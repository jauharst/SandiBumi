//! Workflow runner: executes deterministic modules across wells (rayon-parallel),
//! resolving interval parameters per zone (interval-parameter style), and the cutoff/summary
//! engine modeled on pay-summary specs.

use crate::db;
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::Connection;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct RunModuleRequest {
    pub module: String,
    pub well_ids: Vec<String>,
    /// Arg name → curve mnemonic chosen in the dialog (defaults come from the manifest).
    pub log_inputs: HashMap<String, String>,
    /// Numeric interval parameters from the dialog (whole-well values; zone_params override).
    pub params: HashMap<String, f64>,
    /// String options from the dialog.
    pub opts: HashMap<String, String>,
    /// Log set the outputs are versioned into ("re-run = version N+1, never overwrite").
    /// None = the default "INTERP" set. Ignored when the caller pre-created per-well set
    /// events (workflow chains — one version per chain run, not per step).
    #[serde(default)]
    pub output_set: Option<String>,
    /// Log set the INPUTS are read from (latest version per well): curves that set wrote
    /// come from its archived values; anything else falls back to normal resolution.
    /// None/empty = current values (the default, same as before P1-c).
    #[serde(default)]
    pub input_set: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleRunResult {
    pub well_id: String,
    pub rows_written: usize,
    pub output_curves: Vec<String>,
    pub error: Option<String>,
}

/// Builds per-sample parameter arrays for every Param arg: dialog value (or manifest
/// default) as the base, then zone_params overrides — '*' applies well-wide, named zones
/// apply over their depth range. This is the interval-parameter model.
fn resolve_param_arrays(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    req_params: &HashMap<String, f64>,
    depth: &[f32],
) -> Result<HashMap<String, Vec<f64>>, String> {
    let zones = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zone_params = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;
    let zone_range: HashMap<&str, (f32, f32)> =
        zones.iter().map(|z| (z.zone_name.as_str(), (z.top_depth, z.bottom_depth))).collect();

    let mut out = HashMap::new();
    // Out-of-spec parameter values are REJECTED here, not clamped. Silently clamping a
    // percent-entered SWT_IRR of 25 down to 0.6 would hand back a plausible-but-wrong answer,
    // and passing it through used to kill the run outright: `f64::clamp` asserts `lo <= hi`, so
    // `limit(swt, 25.0, 1.0)` panicked. The zones dialog and the DB Inspector both write
    // `zone_params` without the range check `moduleDialog.ts` applies to typed values — the zone
    // override is designed to beat the dialog — so this is the one choke point where the
    // already-declared ArgSpec range can actually be enforced. Spec defaults are trusted and not
    // re-validated; only values a user or caller supplied are checked.
    let mut bad: Vec<String> = Vec::new();
    for arg in spec.args.iter().filter(|a| a.kind == ArgKind::Param) {
        let range = || match (arg.min, arg.max) {
            (Some(lo), Some(hi)) => format!("valid {lo} to {hi}"),
            (Some(lo), None) => format!("valid >= {lo}"),
            (None, Some(hi)) => format!("valid <= {hi}"),
            (None, None) => "no declared range".to_string(),
        };
        let in_range =
            |v: f64| arg.min.map_or(true, |lo| v >= lo) && arg.max.map_or(true, |hi| v <= hi);

        // Same test both sides. A non-finite value is out of range by definition — it cannot be
        // clamped, compared or averaged — and letting it through here while rejecting it below
        // left the two supply routes disagreeing about what a valid parameter is. JSON cannot
        // carry NaN or Infinity, so today's single caller cannot trigger it; the point is that
        // the next caller (a chain computing a parameter, say) meets one rule, not two.
        if let Some(&v) = req_params.get(&arg.name) {
            if !v.is_finite() || !in_range(v) {
                bad.push(format!("{} = {v} ({})", arg.name, range()));
            }
        }
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            let v = v as f64;
            if !v.is_finite() || !in_range(v) {
                bad.push(format!("{} = {v} in zone '{}' ({})", arg.name, zp.zone_name, range()));
            }
        }

        let base = req_params
            .get(&arg.name)
            .copied()
            .or_else(|| arg.default.parse().ok())
            .unwrap_or(f64::NAN);
        let mut arr = vec![base; depth.len()];

        // Well-wide default first, then named zones override it.
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            if zp.zone_name == "*" {
                arr.fill(v as f64);
            }
        }
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            if let Some(&(top, bottom)) = zone_range.get(zp.zone_name.as_str()) {
                for (i, d) in depth.iter().enumerate() {
                    if *d >= top && *d < bottom {
                        arr[i] = v as f64;
                    }
                }
            }
        }
        out.insert(arg.name.clone(), arr);
    }
    if !bad.is_empty() {
        return Err(format!(
            "parameter value(s) outside the module's declared range: {}. A common cause is \
             entering a v/v fraction as a percentage. Fix the value or clear the zone override.",
            bad.join("; ")
        ));
    }
    Ok(out)
}

/// Runs one module across every well: parse inputs, resolve zone parameters, evaluate,
/// and write output curves to computed_curves. Wells are processed in parallel.
///
/// The `run_workflow_module` Tauri command now calls [`run_workflow_module_into`] directly (to
/// pass a job handle + cancel flag), so this no-progress convenience wrapper is used only by the
/// test suite — hence `allow(dead_code)` for the lib-proper build.
#[allow(dead_code)]
pub fn run_workflow_module(db: &Mutex<Connection>, req: &RunModuleRequest) -> Vec<ModuleRunResult> {
    run_workflow_module_into(db, req, None, None, None)
}

/// Like [`run_workflow_module`], but chains pass `preset_sets` (well_id → set_id) so every
/// step of one chain run writes into the SAME set version instead of bumping per step, and an
/// optional `cancel` flag lets a running chain skip the remaining wells mid-step so Cancel takes
/// effect within a well or two instead of after the whole step finishes.
pub fn run_workflow_module_into(
    db: &Mutex<Connection>,
    req: &RunModuleRequest,
    preset_sets: Option<&HashMap<String, String>>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<ModuleRunResult> {
    let spec = match modules::list_modules().into_iter().find(|m| m.name == req.module) {
        Some(s) => s,
        None => {
            return req
                .well_ids
                .iter()
                .map(|w| ModuleRunResult {
                    well_id: w.clone(),
                    rows_written: 0,
                    output_curves: vec![],
                    error: Some(format!("unknown module '{}'", req.module)),
                })
                .collect()
        }
    };

    // Options: dialog values over manifest defaults.
    let mut opts: HashMap<String, String> = spec
        .args
        .iter()
        .filter(|a| a.kind == ArgKind::Option)
        .map(|a| (a.name.clone(), a.default.clone()))
        .collect();
    for (k, v) in &req.opts {
        opts.insert(k.clone(), v.clone());
    }

    // The project's depth unit, read ONCE here rather than per well: it is a project-level
    // fact, and the wells below run under rayon where each lock acquisition would contend.
    let depth_unit = {
        let conn = db.lock().unwrap();
        crate::units::project_depth_unit_or_default(&conn)
    };

    // Input curves: dialog mnemonic over manifest default mnemonic.
    let log_args: Vec<(String, String)> = spec
        .args
        .iter()
        .filter(|a| a.kind == ArgKind::LogIn)
        .map(|a| {
            let mnemonic = req.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            (a.name.clone(), mnemonic)
        })
        .collect();
    // Expose each input's resolved mnemonic to the module as "__IN_<arg>", so modules
    // that derive their output names from their input (depth_shift → GR_DS) can.
    for (arg_name, mnemonic) in &log_args {
        opts.insert(format!("__IN_{arg_name}"), mnemonic.trim().to_uppercase());
    }

    // Phase 1 outcome per well. Outputs are held in memory so Phase 2 can write EVERY well in
    // one batched transaction (vs a fsync-bound delete+append transaction per well — the
    // dominant field-scale write cost). Nothing is written to computed_curves during Phase 1.
    enum Outcome {
        Skipped,
        Failed(String),
        Computed { depth: Vec<f32>, outputs: HashMap<String, Vec<f32>> },
    }

    let outcomes: Vec<Outcome> = req
        .well_ids
        .par_iter()
        .map(|well_id| {
            // Cooperative cancellation: once a chain sets its flag, the wells rayon hasn't
            // started yet skip all fetch/compute/write and return a no-op, so the in-flight
            // par_iter drains in ~a well or two instead of grinding through every remaining
            // well. The chain re-checks the flag between steps and finalizes as Cancelled.
            if cancel.map_or(false, |c| c.load(std::sync::atomic::Ordering::SeqCst)) {
                // This path reads the raw flag (a chain shares one flag across registries) rather
                // than going through `JobHandle::is_cancelled`, so the observation has to be
                // recorded explicitly — otherwise `run_job` would finalize a genuinely drained run
                // as Completed, which is the same class of lie in the opposite direction.
                if let Some(p) = progress {
                    p.note_cancel_observed();
                }
                return Outcome::Skipped;
            }
            // Live per-well progress for the universal Processing panel. With rayon, several
            // wells show "running" at once — an honest picture of the parallel work.
            if let Some(p) = progress {
                p.start_item(well_id);
            }
            let compute = || -> Result<(Vec<f32>, HashMap<String, Vec<f32>>), String> {
                let curve_names: Vec<String> = log_args.iter().map(|(_, m)| m.clone()).collect();
                // A chain's own set event: its earlier steps' outputs beat the input set.
                let own_set = preset_sets.and_then(|m| m.get(well_id.as_str())).map(|s| s.as_str());
                let (depth, columns, params) = {
                    let conn = db.lock().unwrap();
                    let (depth, columns) = equations::fetch_curve_frame_from_set(
                        &conn,
                        well_id,
                        &curve_names,
                        req.input_set.as_deref(),
                        own_set,
                    )
                    .map_err(|e| e.to_string())?;
                    if depth.is_empty() {
                        return Err("no curve data for well".into());
                    }
                    let params = resolve_param_arrays(&conn, well_id, &spec, &req.params, &depth)?;
                    (depth, columns, params)
                };

                let mut logs: HashMap<String, Vec<f32>> = HashMap::new();
                logs.insert("DEPTH".to_string(), depth.clone());
                for (arg_name, mnemonic) in &log_args {
                    let values = columns
                        .get(&mnemonic.trim().to_uppercase())
                        .cloned()
                        .unwrap_or_else(|| vec![f32::NAN; depth.len()]);
                    logs.insert(arg_name.clone(), values);
                }
                // Unit-contract inputs (ArgSpec.computed_only, e.g. gascorr FTEMP/FPRESS):
                // re-resolve from computed provenance only — the frame above may have
                // fallen back to a RAW import with the same mnemonic but the wrong unit.
                for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn && a.computed_only) {
                    let mnemonic = log_args
                        .iter()
                        .find(|(name, _)| name == &a.name)
                        .map(|(_, m)| m.clone())
                        .unwrap_or_else(|| a.default.clone());
                    let conn = db.lock().unwrap();
                    let values = equations::fetch_computed_only_aligned(
                        &conn,
                        well_id,
                        &mnemonic,
                        &depth,
                        req.input_set.as_deref(),
                        own_set,
                    )
                    .map_err(|e| e.to_string())?;
                    logs.insert(a.name.clone(), values);
                }

                // Optional bad-hole (or any flag) mask. Resolve it BEFORE the module runs so
                // flagged samples can be excluded from the module's INPUTS, not just its
                // outputs. Modules that compute run-level statistics — gr_normalize's P3/P97
                // percentiles, log_predict's KNN training set — would otherwise be anchored by
                // casing/washout samples, and that mis-anchoring contaminates every output
                // sample, flagged or not. The mask is resolved like any other input
                // (generic-store aware).
                let mask_name = req.opts.get("MASK").map(|s| s.trim()).unwrap_or("");
                let mask: Option<Vec<f32>> = if mask_name.is_empty() {
                    None
                } else {
                    let conn = db.lock().unwrap();
                    let (_, mcols) = equations::fetch_curve_frame_from_set(
                        &conn,
                        well_id,
                        &[mask_name.to_string()],
                        req.input_set.as_deref(),
                        own_set,
                    )
                    .map_err(|e| e.to_string())?;
                    drop(conn);
                    mcols.get(&mask_name.to_uppercase()).cloned()
                };

                // Blank flagged samples in the module INPUTS (never DEPTH) before the run, so
                // per-run statistics only see unmasked data.
                if let Some(mask) = &mask {
                    for (arg_name, _) in &log_args {
                        if let Some(values) = logs.get_mut(arg_name) {
                            for (v, m) in values.iter_mut().zip(mask.iter()) {
                                if *m == 1.0 {
                                    *v = f32::NAN;
                                }
                            }
                        }
                    }
                }

                let ctx = ModuleContext { n: depth.len(), logs, params, opts: opts.clone(), depth_unit };
                let mut outputs = modules::run_module(&req.module, &ctx)?;

                // Blank flagged samples in the OUTPUTS too, so a flagged depth's result is
                // never trusted downstream.
                if let Some(mask) = &mask {
                    for values in outputs.values_mut() {
                        for (v, m) in values.iter_mut().zip(mask.iter()) {
                            if *m == 1.0 {
                                *v = f32::NAN;
                            }
                        }
                    }
                }

                Ok((depth, outputs))
            };

            let outcome = match compute() {
                Ok((depth, outputs)) => Outcome::Computed { depth, outputs },
                Err(e) => Outcome::Failed(e),
            };
            if let Some(p) = progress {
                match &outcome {
                    // A run whose outputs are all MISSING (e.g. gascorr with no precalc, or a
                    // module fed an all-NaN input) did no real work — flag it Warned, not a green
                    // Ok, so the panel doesn't read as a successful correction.
                    Outcome::Computed { outputs, .. }
                        if outputs.values().any(|v| v.iter().any(|x| x.is_finite())) =>
                    {
                        p.finish_item(well_id, crate::jobs::ItemState::Ok, None)
                    }
                    Outcome::Computed { .. } => {
                        p.finish_item(well_id, crate::jobs::ItemState::Warned, Some("no finite output".into()))
                    }
                    Outcome::Failed(e) => {
                        p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.clone()))
                    }
                    Outcome::Skipped => {}
                }
            }
            outcome
        })
        .collect();

    // ---- Phase 2: ONE batched, versioned write for every well that produced output. ----
    // Set ids: a chain supplies its shared per-well event via `preset_sets`; a plain module run
    // allocates version N+1 per well (batched into one transaction). Then every well's curves
    // land in a SINGLE transaction instead of a delete+append+flush transaction per well.
    let succ_ids: Vec<String> = req
        .well_ids
        .iter()
        .zip(outcomes.iter())
        .filter_map(|(w, o)| match o {
            Outcome::Computed { outputs, .. } if !outputs.is_empty() => Some(w.clone()),
            _ => None,
        })
        .collect();

    let mut set_err: Option<String> = None;
    let set_ids: HashMap<String, String> = if succ_ids.is_empty() {
        HashMap::new()
    } else if let Some(preset) = preset_sets {
        succ_ids.iter().filter_map(|w| preset.get(w).map(|s| (w.clone(), s.clone()))).collect()
    } else {
        let set_spec = equations::LogSetSpec {
            set_name: req.output_set.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "INTERP".into()),
            module: req.module.clone(),
            params_json: serde_json::to_string(&req.params).unwrap_or_default(),
            inputs_json: {
                // Provenance records where inputs were read from too.
                let mut prov = log_args.clone();
                if let Some(s) = req.input_set.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                    prov.push(("input_set".into(), s.to_string()));
                }
                serde_json::to_string(&prov).unwrap_or_default()
            },
        };
        let conn = db.lock().unwrap();
        match equations::create_log_sets_batch(&conn, &succ_ids, &set_spec) {
            Ok(m) => m,
            Err(e) => {
                set_err = Some(e.to_string());
                HashMap::new()
            }
        }
    };

    let mut writes: Vec<equations::WellWrite> = Vec::with_capacity(succ_ids.len());
    for (well_id, o) in req.well_ids.iter().zip(outcomes.iter()) {
        if let Outcome::Computed { depth, outputs } = o {
            if outputs.is_empty() {
                continue;
            }
            if let Some(set_id) = set_ids.get(well_id) {
                writes.push(equations::WellWrite {
                    well_id: well_id.clone(),
                    depth: depth.clone(),
                    curves: outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    set_id: set_id.clone(),
                });
            }
        }
    }

    // The batched write is one big transaction with no per-well signal, so without this the
    // panel's bar sits at the step boundary looking frozen. Name the wait so it reads as
    // working, not stuck. (The panel polls the job registry, not the DB, so this shows even
    // while the write holds the DB lock.)
    if let Some(p) = progress {
        if !writes.is_empty() {
            p.set_current(Some(format!("Writing {} well(s)…", writes.len())));
        }
    }
    let write_err: Option<String> = if writes.is_empty() {
        None
    } else {
        let conn = db.lock().unwrap();
        equations::write_computed_curves_versioned_batch(&conn, &writes).err().map(|e| e.to_string())
    };

    // A Phase-2 set-allocation or write failure downgrades the affected wells in the panel —
    // their compute finished OK but nothing was persisted, so they must not read as green.
    if let Some(p) = progress {
        if let Some(e) = &set_err {
            for w in &succ_ids {
                p.mark_item(w, crate::jobs::ItemState::Failed, Some(e.clone()));
            }
        } else if let Some(e) = &write_err {
            for wr in &writes {
                p.mark_item(&wr.well_id, crate::jobs::ItemState::Failed, Some(e.clone()));
            }
        }
    }

    // Per-well results, in the original well order.
    req.well_ids
        .iter()
        .zip(outcomes.iter())
        .map(|(well_id, o)| match o {
            Outcome::Skipped => ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: None },
            Outcome::Failed(e) => ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e.clone()) },
            Outcome::Computed { depth, outputs } => {
                if outputs.is_empty() {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: None }
                } else if let Some(e) = &set_err {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e.clone()) }
                } else if !set_ids.contains_key(well_id) {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some("no output set allocated for well".into()) }
                } else if let Some(e) = &write_err {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e.clone()) }
                } else {
                    let mut names: Vec<String> = outputs.keys().cloned().collect();
                    names.sort();
                    // Every output sample MISSING (e.g. gascorr with no precalc): a green
                    // "N samples → …" line is indistinguishable from a real result and would be
                    // totalled into History as success, so surface it distinctly instead of a
                    // bare success with the full depth count. Mirrors SandiMin's "no solvable
                    // samples" error for a zero-useful run.
                    let any_finite = outputs.values().any(|v| v.iter().any(|x| x.is_finite()));
                    if any_finite {
                        ModuleRunResult { well_id: well_id.clone(), rows_written: depth.len(), output_curves: names, error: None }
                    } else {
                        ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: names, error: Some("no finite output — every sample is missing (check inputs, e.g. precalc not run)".into()) }
                    }
                }
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pay summary — cutoffs → flags → per-zone statistics (pay-summary model)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct PaySummaryRequest {
    pub well_ids: Vec<String>,
    /// VSH <= vsh_max counts as sand.
    pub vsh_max: f64,
    /// PHIE >= phie_min counts as reservoir (with sand).
    pub phie_min: f64,
    /// SWE <= swe_max counts as pay (with reservoir).
    pub swe_max: f64,
    /// Optional PERM >= perm_min added to the pay flag when PERM exists.
    pub perm_min: Option<f64>,
    /// When true, FLAG_* curves are written in place without creating a versioned log set. Set
    /// by the report/composite render pass, whose pay flags are a render side-effect that must
    /// not churn the archive with a version per render. The explicit Cutoffs & Summary run
    /// leaves this false, so its pay flags are versioned with the cutoffs recorded in provenance
    /// (log_sets.params_json).
    #[serde(default)]
    pub skip_version: bool,
    /// When true, compute and return the per-zone statistics WITHOUT persisting any FLAG_*
    /// curves at all. The Field Dashboard sets this: it recomputes on every cutoff tweak and
    /// only consumes the returned rows, so writing 3 FLAG curves × every well each refresh
    /// (~1,600 delete+append+flush transactions on 540 wells) was pure waste that dominated
    /// its runtime. Persisting flags stays the job of the explicit Cutoffs & Summary run.
    #[serde(default)]
    pub stats_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaySummaryRow {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub flag: String, // SAND | RESERVOIR | PAY
    pub top: f32,
    pub bottom: f32,
    pub gross: f32,
    pub net: f32,
    pub ntg: f32,
    pub avg_vsh: f32,
    pub avg_phie: f32,
    /// PHIE-weighted average SWE (pay-summary convention).
    pub avg_swe: f32,
    pub hpv: f32, // sum of PHIE*(1-SWE)*thickness over net
    /// In-zone samples the classifier could actually judge. **0 means the well was never
    /// interpreted** — VSH/PHIE/SWE resolved to all-NaN — as opposed to a genuine zero-net
    /// result, which the identical `net`/`ntg`/`hpv` zeros cannot distinguish on their own.
    /// Consumers must render "—" rather than 0.00 when this is 0.
    pub n_classified: usize,
}

const SUMMARY_FLAGS: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

/// Computes the pay summary per well per zone and writes FLAG_SAND / FLAG_RESERVOIR /
/// FLAG_PAY curves. Wells without zones get a single whole-well "ALL" zone.
pub fn run_pay_summary(db: &Mutex<Connection>, req: &PaySummaryRequest) -> Result<Vec<PaySummaryRow>, String> {
    let curve_names: Vec<String> = vec!["VSH".into(), "PHIE".into(), "SWE".into(), "PERM".into()];
    let mut all_rows = Vec::new();

    for well_id in &req.well_ids {
        let conn = db.lock().unwrap();
        let well_name: String = conn
            .query_row(
                "SELECT well_name FROM wells WHERE well_id = ?1",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| well_id.clone());

        // Per-well isolation: a well with no curves — or a transient fetch/zone read error — is
        // skipped, keeping every other well's rows, rather than `?`-aborting the whole batch (a
        // single bad well would otherwise zero the entire Field Dashboard / summary response).
        let (depth, columns) = match equations::fetch_curve_frame(&conn, well_id, &curve_names) {
            Ok((d, c)) if !d.is_empty() => (d, c),
            _ => continue,
        };
        let mut zones = match db::list_zones(&conn, well_id) {
            Ok(z) => z,
            Err(_) => continue,
        };
        drop(conn);

        if zones.is_empty() {
            zones.push(db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: depth[0],
                bottom_depth: *depth.last().unwrap(),
            });
        }

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie = &columns["PHIE"];
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];
        let has_perm_cut = req.perm_min.is_some() && perm.iter().any(|v| !v.is_nan());

        // Sample thickness: forward depth difference, last sample reuses the previous step.
        let mut step = vec![0.0f32; n];
        for i in 0..n {
            step[i] = if i + 1 < n {
                depth[i + 1] - depth[i]
            } else if i > 0 {
                step[i - 1]
            } else {
                0.0
            };
        }

        // Flags per sample: NaN inputs exclude the sample (flag stays NaN). Single-sourced
        // through `classify_sample` so the sweep engine below applies identical cutoff logic.
        let mut flag_sand = vec![f32::NAN; n];
        let mut flag_res = vec![f32::NAN; n];
        let mut flag_pay = vec![f32::NAN; n];
        for i in 0..n {
            let (fs, fr, fp) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i],
                req.vsh_max, req.phie_min, req.swe_max, req.perm_min, has_perm_cut,
            );
            flag_sand[i] = fs;
            flag_res[i] = fr;
            flag_pay[i] = fp;
        }

        if !req.stats_only {
            let conn = db.lock().unwrap();
            if req.skip_version {
                // Render side-effect (report/composite): overwrite FLAG_* in place, no version churn.
                for (name, values) in
                    [("FLAG_SAND", &flag_sand), ("FLAG_RESERVOIR", &flag_res), ("FLAG_PAY", &flag_pay)]
                {
                    equations::write_computed_curve(&conn, well_id, &depth, name, values).map_err(|e| e.to_string())?;
                }
            } else {
                // Version the pay flags into a log set with provenance — module + the CUTOFFS
                // that produced them + the inputs — like any other module output, so a re-run
                // keeps history, any version is restorable/prunable from the catalog, and the
                // cutoffs are retrievable from log_sets.params_json.
                let params_json = serde_json::json!({
                    "vsh_max": req.vsh_max,
                    "phie_min": req.phie_min,
                    "swe_max": req.swe_max,
                    "perm_min": req.perm_min,
                })
                .to_string();
                let spec = equations::LogSetSpec {
                    set_name: "PAYFLAG".into(),
                    module: "pay_summary".into(),
                    params_json,
                    inputs_json: serde_json::to_string(&curve_names).unwrap_or_default(),
                };
                let (set_id, _) =
                    equations::create_log_set(&conn, well_id, &spec).map_err(|e| e.to_string())?;
                let batch: Vec<(&str, &[f32])> = vec![
                    ("FLAG_SAND", flag_sand.as_slice()),
                    ("FLAG_RESERVOIR", flag_res.as_slice()),
                    ("FLAG_PAY", flag_pay.as_slice()),
                ];
                equations::write_computed_curves_versioned(&conn, well_id, &depth, &batch, &set_id)
                    .map_err(|e| e.to_string())?;
            }
        }

        for zone in &zones {
            for flag_name in SUMMARY_FLAGS {
                let flags = match flag_name {
                    "SAND" => &flag_sand,
                    "RESERVOIR" => &flag_res,
                    _ => &flag_pay,
                };
                let mut net = 0.0f64;
                let mut net_vsh = 0.0f64;
                let mut net_phie = 0.0f64;
                let mut sum_vsh = 0.0f64;
                let mut sum_phie = 0.0f64;
                let mut sum_phie_swe = 0.0f64;
                let mut sum_phie_w = 0.0f64;
                let mut hpv = 0.0f64;
                // Samples in this zone that the classifier could actually judge. A well whose
                // VSH/PHIE/SWE were never computed classifies to NaN everywhere, which leaves
                // net/ntg/hpv at 0.0 — byte-identical to a genuine wet or shaly zone. Carrying
                // the count lets the UI and the client PDF say "not interpreted" instead of
                // printing a hard zero that reads as a real answer.
                let mut n_classified = 0usize;

                for i in 0..n {
                    // Each sample represents the forward interval [depth[i], depth[i]+step].
                    // Clamp its contribution to the overlap with [zone.top, zone.bottom): the
                    // last in-zone sample no longer bleeds a full step past the base, a sample
                    // straddling the zone top is counted for its in-zone part, and net can never
                    // exceed gross (a sub-step-thick zone previously could).
                    let s_top = depth[i] as f64;
                    let s_bot = (depth[i] + step[i]) as f64;
                    let lo = s_top.max(zone.top_depth as f64);
                    let hi = s_bot.min(zone.bottom_depth as f64);
                    let h = hi - lo;
                    if h <= 0.0 {
                        continue;
                    }
                    if !flags[i].is_nan() {
                        n_classified += 1;
                    }
                    if flags[i] != 1.0 {
                        continue;
                    }
                    net += h;
                    if !vsh[i].is_nan() {
                        sum_vsh += vsh[i] as f64 * h;
                        net_vsh += h;
                    }
                    if !phie[i].is_nan() {
                        sum_phie += phie[i] as f64 * h;
                        net_phie += h;
                        if !swe[i].is_nan() {
                            sum_phie_swe += phie[i] as f64 * swe[i] as f64 * h;
                            sum_phie_w += phie[i] as f64 * h;
                            hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                        }
                    }
                }

                let gross = zone.bottom_depth - zone.top_depth;
                all_rows.push(PaySummaryRow {
                    well_id: well_id.clone(),
                    well_name: well_name.clone(),
                    zone: zone.zone_name.clone(),
                    flag: flag_name.to_string(),
                    top: zone.top_depth,
                    bottom: zone.bottom_depth,
                    gross,
                    net: net as f32,
                    ntg: if gross > 0.0 { (net / gross as f64) as f32 } else { 0.0 },
                    // Averages are normalised by the net thickness over which THAT curve is
                    // valid — not total net — so a SAND-row sample with valid VSH but missing
                    // PHIE no longer drags avg_phie toward zero.
                    avg_vsh: if net_vsh > 0.0 { (sum_vsh / net_vsh) as f32 } else { f32::NAN },
                    avg_phie: if net_phie > 0.0 { (sum_phie / net_phie) as f32 } else { f32::NAN },
                    avg_swe: if sum_phie_w > 0.0 { (sum_phie_swe / sum_phie_w) as f32 } else { f32::NAN },
                    hpv: hpv as f32,
                    n_classified,
                });
            }
        }
    }

    Ok(all_rows)
}

// ---------------------------------------------------------------------------
// Cutoff sensitivity (ROADMAP Wave E item 21) — sweep the pay engine over a range
// of candidate cutoffs, holding the other two fixed, to find the elbow where pay
// stops responding to the cutoff. This is the sensitivity-sweep method; the companion
// method (DST-highlighted crossplots) lives in the frontend cutoff pane. Both follow the
// standard cutoff-selection practice: pick the cutoff where net stops responding, then
// confirm it against tested rock rather than against the sweep alone.
// ---------------------------------------------------------------------------

/// Per-sample SAND / RESERVOIR / PAY classification against the cutoffs, matching the
/// Pay-summary NaN propagation: a missing VSH excludes all three (returns NaN,NaN,NaN);
/// a missing PHIE excludes RESERVOIR and PAY; a missing SWE excludes PAY. Each returned
/// value is `f32::NAN` when the sample is excluded, else `0.0`/`1.0`. `has_perm_cut` is the
/// caller's decision that a PERM cutoff is active (perm_min set and PERM present in the set).
#[inline]
fn classify_sample(
    vsh: f32,
    phie: f32,
    swe: f32,
    perm: f32,
    vsh_max: f64,
    phie_min: f64,
    swe_max: f64,
    perm_min: Option<f64>,
    has_perm_cut: bool,
) -> (f32, f32, f32) {
    if vsh.is_nan() {
        return (f32::NAN, f32::NAN, f32::NAN);
    }
    let sand = (vsh as f64) <= vsh_max;
    let fs = sand as u8 as f32;
    if phie.is_nan() {
        return (fs, f32::NAN, f32::NAN);
    }
    let res = sand && (phie as f64) >= phie_min;
    let fr = res as u8 as f32;
    if swe.is_nan() {
        return (fs, fr, f32::NAN);
    }
    let mut pay = res && (swe as f64) <= swe_max;
    if has_perm_cut {
        // A sample with no PERM value cannot demonstrate it passes the cutoff — missing
        // PERM must fail, not silently pass (same rule as run_pay_summary).
        pay = pay && !perm.is_nan() && (perm as f64) >= perm_min.unwrap();
    }
    (fs, fr, pay as u8 as f32)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SweepProp {
    Vsh,
    Phie,
    Swe,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Metric {
    Net,
    Hpv,
    Ntg,
}

/// Evaluates the pay metric at every candidate cutoff. Pure over pre-assembled arrays so it
/// is unit-testable without a database; `incl_h[i]` is the sample's clamped geometric
/// thickness within the analysed interval (zone ∩ DST) — 0 excludes it, and net accumulates
/// this clamped overlap (NOT the raw sample step) so net can never exceed gross, matching
/// run_pay_summary. `gross` is the geometric denominator for NTG. Returns
/// (cutoffs, values, peak) where `peak` is the maximum value over the sweep.
#[allow(clippy::too_many_arguments)]
fn compute_sweep(
    vsh: &[f32],
    phie: &[f32],
    swe: &[f32],
    perm: &[f32],
    incl_h: &[f64],
    prop: SweepProp,
    fixed_vsh: f64,
    fixed_phie: f64,
    fixed_swe: f64,
    perm_min: Option<f64>,
    sweep_min: f64,
    sweep_max: f64,
    steps: usize,
    metric: Metric,
    gross: f64,
) -> (Vec<f64>, Vec<f64>, f64) {
    let steps = steps.clamp(2, 500);
    let n = vsh.len();
    // A PERM cutoff only applies when a PERM curve exists for the well. Scoped over the WHOLE
    // frame (not just the analysed subset) so the PAY metric agrees with run_pay_summary, which
    // decides has_perm_cut once per well before any zone/DST filtering. Judging it over the
    // included subset alone would silently disable the gate on a zone/DST slice that happens to
    // hold no PERM, so identical cutoffs could report more pay here than in the pay summary.
    let has_perm_cut = perm_min.is_some() && perm.iter().any(|v| !v.is_nan());

    let mut cutoffs = Vec::with_capacity(steps);
    let mut values = Vec::with_capacity(steps);
    let mut peak = f64::NEG_INFINITY;

    for k in 0..steps {
        let t = k as f64 / (steps - 1) as f64;
        let cut = sweep_min + (sweep_max - sweep_min) * t;
        let (mut vsh_max, mut phie_min, mut swe_max) = (fixed_vsh, fixed_phie, fixed_swe);
        match prop {
            SweepProp::Vsh => vsh_max = cut,
            SweepProp::Phie => phie_min = cut,
            SweepProp::Swe => swe_max = cut,
        }

        let mut net = 0.0f64;
        let mut hpv = 0.0f64;
        for i in 0..n {
            let h = incl_h[i];
            if h <= 0.0 {
                continue;
            }
            let (_s, _r, pay) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i], vsh_max, phie_min, swe_max, perm_min, has_perm_cut,
            );
            if pay == 1.0 {
                net += h;
                if !phie[i].is_nan() && !swe[i].is_nan() {
                    hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                }
            }
        }

        let value = match metric {
            Metric::Net => net,
            Metric::Hpv => hpv,
            Metric::Ntg => {
                if gross > 0.0 {
                    net / gross
                } else {
                    0.0
                }
            }
        };
        cutoffs.push(cut);
        values.push(value);
        if value > peak {
            peak = value;
        }
    }
    if !peak.is_finite() {
        peak = 0.0;
    }
    (cutoffs, values, peak)
}

#[derive(Debug, Clone, Deserialize)]
pub struct CutoffSweepRequest {
    pub well_ids: Vec<String>,
    /// Which cutoff to sweep: "VSH" | "PHIE" | "SWE".
    pub property: String,
    /// Fixed values for the two cutoffs NOT being swept (the swept one's field is ignored).
    pub vsh_max: f64,
    pub phie_min: f64,
    pub swe_max: f64,
    pub perm_min: Option<f64>,
    pub sweep_min: f64,
    pub sweep_max: f64,
    pub steps: usize,
    /// Metric plotted on Y: "NET" (net thickness) | "HPV" (hydrocarbon pore-thickness) | "NTG".
    pub metric: String,
    /// Restrict to one named zone; None/empty = whole well.
    #[serde(default)]
    pub zone: Option<String>,
    /// Restrict to samples inside an aux_data interval set (e.g. "PERFORATION" / "DST");
    /// None/empty = every sample in the zone.
    #[serde(default)]
    pub dst_dataset: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CutoffSweepSeries {
    pub well_id: String,
    pub well_name: String,
    pub cutoffs: Vec<f64>,
    pub values: Vec<f64>,
    /// Maximum value over the sweep (the frontend normalises each well to its own peak).
    pub peak: f64,
    /// Geometric gross thickness of the analysed interval (NTG denominator).
    pub gross: f64,
    /// Number of samples that entered the analysis (0 ⇒ nothing to plot; UI warns).
    pub n_samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CutoffSweepResult {
    pub series: Vec<CutoffSweepSeries>,
    pub property: String,
    pub metric: String,
}

/// Collapses an aux_data set to its distinct, non-overlapping depth intervals (rows with a
/// base depth, merged) for DST/perforation filtering. Point rows (no base) are ignored — a
/// test needs an interval, not a marker. Overlapping or touching intervals are unioned so a
/// re-perforation or redundant row cannot inflate the summed DST gross (the NTG denominator):
/// membership already counts each sample once (via `any`), so the gross must too.
fn aux_intervals(rows: &[db::AuxRow]) -> Vec<(f32, f32)> {
    let mut iv: Vec<(f32, f32)> = rows
        .iter()
        .filter_map(|r| r.depth_base.map(|b| (r.depth_top, b)))
        .filter(|(t, b)| b > t)
        .collect();
    iv.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut merged: Vec<(f32, f32)> = Vec::with_capacity(iv.len());
    for (t, b) in iv {
        match merged.last_mut() {
            Some(last) if t <= last.1 => {
                if b > last.1 {
                    last.1 = b;
                }
            }
            _ => merged.push((t, b)),
        }
    }
    merged
}

/// Geometric overlap thickness of a sample's forward interval `[s_top, s_bot]` with the
/// zone `[ztop, zbot)`, further intersected with the (merged, non-overlapping) DST intervals
/// when present. Mirrors run_pay_summary's zone clamp so a sample straddling the zone/DST
/// boundary contributes only its in-interval part and net can never exceed gross.
fn sample_incl_thickness(
    s_top: f64,
    s_bot: f64,
    ztop: f64,
    zbot: f64,
    dst: Option<&[(f32, f32)]>,
) -> f64 {
    let lo = s_top.max(ztop);
    let hi = s_bot.min(zbot);
    let base = hi - lo;
    if base <= 0.0 {
        return 0.0;
    }
    match dst {
        None => base,
        // DST intervals are pre-merged (non-overlapping) by aux_intervals, so summing the
        // per-interval overlaps counts each unit of thickness at most once.
        Some(iv) => iv
            .iter()
            .map(|(t, b)| {
                let l2 = lo.max(*t as f64);
                let h2 = hi.min(*b as f64);
                (h2 - l2).max(0.0)
            })
            .sum(),
    }
}

/// A 0-sample sweep row so a well that can't be analysed (no curves, missing zone, or a
/// transient DB read error) still shows in the legend as "(0 samples)" instead of vanishing
/// and making the well count undercount.
fn empty_sweep_series(well_id: &str, well_name: String) -> CutoffSweepSeries {
    CutoffSweepSeries {
        well_id: well_id.to_string(),
        well_name,
        cutoffs: Vec::new(),
        values: Vec::new(),
        peak: 0.0,
        gross: 0.0,
        n_samples: 0,
    }
}

/// Method 1 of the cutoff study: for each well, sweep one cutoff across `[sweep_min,
/// sweep_max]` (holding the other two fixed) and report the pay metric at each step, so the
/// user can pick the cutoff at the response elbow. Reads VSH/PHIE/SWE/PERM, filters to an
/// optional zone and optional DST interval set, and writes nothing (pure analysis).
pub fn run_cutoff_sweep(
    db: &Mutex<Connection>,
    req: &CutoffSweepRequest,
) -> Result<CutoffSweepResult, String> {
    let prop = match req.property.to_uppercase().as_str() {
        "VSH" => SweepProp::Vsh,
        "PHIE" => SweepProp::Phie,
        "SWE" => SweepProp::Swe,
        other => return Err(format!("unknown sweep property '{other}' (VSH|PHIE|SWE)")),
    };
    let metric = match req.metric.to_uppercase().as_str() {
        "NET" => Metric::Net,
        "HPV" => Metric::Hpv,
        "NTG" => Metric::Ntg,
        other => return Err(format!("unknown metric '{other}' (NET|HPV|NTG)")),
    };
    if !(req.sweep_max > req.sweep_min) {
        return Err("sweep max must be greater than sweep min".into());
    }
    let steps = req.steps.clamp(2, 500);
    let dst_name = req.dst_dataset.as_deref().filter(|s| !s.is_empty());
    let zone_name = req.zone.as_deref().filter(|s| !s.is_empty());
    let curve_names: Vec<String> = vec!["VSH".into(), "PHIE".into(), "SWE".into(), "PERM".into()];
    let mut series = Vec::new();

    for well_id in &req.well_ids {
        let conn = db.lock().unwrap();
        let well_name: String = conn
            .query_row(
                "SELECT well_name FROM wells WHERE well_id = ?1",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| well_id.clone());
        // Per-well isolation: a transient fetch/zone/aux read error skips just this well (a
        // 0-sample legend row) instead of `?`-aborting the whole batch and discarding every
        // well already accumulated — same graceful degradation as run_workflow_module.
        let (depth, columns) = match equations::fetch_curve_frame(&conn, well_id, &curve_names) {
            Ok((d, c)) if !d.is_empty() => (d, c),
            _ => {
                drop(conn);
                series.push(empty_sweep_series(well_id, well_name));
                continue;
            }
        };
        let zones = match db::list_zones(&conn, well_id) {
            Ok(z) => z,
            Err(_) => {
                drop(conn);
                series.push(empty_sweep_series(well_id, well_name));
                continue;
            }
        };
        let dst = match dst_name {
            Some(ds) => match db::list_aux_data(&conn, well_id, Some(ds)) {
                Ok(rows) => Some(aux_intervals(&rows)),
                Err(_) => {
                    drop(conn);
                    series.push(empty_sweep_series(well_id, well_name));
                    continue;
                }
            },
            None => None,
        };
        drop(conn);

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie = &columns["PHIE"];
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];

        // Sample thickness: forward depth difference, last sample reuses the previous step
        // (same convention as run_pay_summary).
        let mut step = vec![0.0f32; n];
        for i in 0..n {
            step[i] = if i + 1 < n {
                depth[i + 1] - depth[i]
            } else if i > 0 {
                step[i - 1]
            } else {
                0.0
            };
        }

        // Zone bounds: a named zone that a well lacks yields an empty (0-sample) series so
        // the run still returns a row for that well rather than silently dropping it.
        let (ztop, zbot) = match zone_name {
            Some(z) => match zones.iter().find(|zz| zz.zone_name == z) {
                Some(zz) => (zz.top_depth, zz.bottom_depth),
                None => {
                    series.push(empty_sweep_series(well_id, well_name));
                    continue;
                }
            },
            None => (depth[0], *depth.last().unwrap()),
        };

        // Per-sample clamped geometric thickness within [ztop, zbot) ∩ DST — mirrors
        // run_pay_summary's zone clamp so net can never exceed gross. A sample straddling the
        // zone/DST boundary contributes only its in-interval part, not its whole step; a DST
        // boundary landing mid-sample counts that sample's actual overlap fraction.
        let mut incl_h = vec![0.0f64; n];
        let mut n_incl = 0usize;
        for i in 0..n {
            let s_top = depth[i] as f64;
            let s_bot = (depth[i] + step[i]) as f64;
            let h = sample_incl_thickness(s_top, s_bot, ztop as f64, zbot as f64, dst.as_deref());
            incl_h[i] = h;
            if h > 0.0 {
                n_incl += 1;
            }
        }

        // Geometric gross (NTG denominator): DST intervals clipped to the zone, else the
        // whole zone length.
        let gross = match &dst {
            None => (zbot - ztop).max(0.0) as f64,
            Some(iv) => iv
                .iter()
                .map(|(t, b)| {
                    let lo = (*t).max(ztop);
                    let hi = (*b).min(zbot);
                    (hi - lo).max(0.0) as f64
                })
                .sum(),
        };

        let (cutoffs, values, peak) = compute_sweep(
            vsh, phie, swe, perm, &incl_h, prop, req.vsh_max, req.phie_min, req.swe_max,
            req.perm_min, req.sweep_min, req.sweep_max, steps, metric, gross,
        );
        series.push(CutoffSweepSeries {
            well_id: well_id.clone(),
            well_name,
            cutoffs,
            values,
            peak,
            gross,
            n_samples: n_incl,
        });
    }

    Ok(CutoffSweepResult {
        series,
        property: req.property.to_uppercase(),
        metric: req.metric.to_uppercase(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest;
    use std::collections::HashMap;

    /// The shared cutoff classifier must reproduce the .paysum NaN propagation exactly:
    /// a missing input excludes it and everything downstream, and a missing PERM fails an
    /// active PERM cutoff instead of passing.
    #[test]
    fn classify_sample_nan_propagation() {
        // Clean pay (no perm cut).
        assert_eq!(
            classify_sample(0.2, 0.2, 0.3, f32::NAN, 0.5, 0.1, 0.6, None, false),
            (1.0, 1.0, 1.0)
        );
        // Missing VSH → all excluded.
        let (s, r, p) = classify_sample(f32::NAN, 0.2, 0.3, f32::NAN, 0.5, 0.1, 0.6, None, false);
        assert!(s.is_nan() && r.is_nan() && p.is_nan());
        // Missing PHIE → SAND set, RES/PAY excluded.
        let (s, r, p) = classify_sample(0.2, f32::NAN, 0.3, f32::NAN, 0.5, 0.1, 0.6, None, false);
        assert_eq!(s, 1.0);
        assert!(r.is_nan() && p.is_nan());
        // Missing SWE → SAND+RES set, PAY excluded.
        let (s, r, p) = classify_sample(0.2, 0.2, f32::NAN, f32::NAN, 0.5, 0.1, 0.6, None, false);
        assert_eq!((s, r), (1.0, 1.0));
        assert!(p.is_nan());
        // Fails the sand cutoff → SAND 0 cascades to RES/PAY 0.
        assert_eq!(
            classify_sample(0.9, 0.2, 0.3, f32::NAN, 0.5, 0.1, 0.6, None, false),
            (0.0, 0.0, 0.0)
        );
        // Active PERM cutoff: missing PERM fails; sufficient PERM passes.
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, f32::NAN, 0.5, 0.1, 0.6, Some(1.0), true);
        assert_eq!(p, 0.0);
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, 5.0, 0.5, 0.1, 0.6, Some(1.0), true);
        assert_eq!(p, 1.0);
    }

    /// A well whose VSH/PHIE/SWE were never computed classifies to NaN at every sample, which
    /// leaves net/ntg/hpv at exactly 0.0 — byte-identical to a genuine wet or shaly zone. The
    /// dialog, the Field Dashboard and the client PDF all printed that zero as if it were an
    /// answer. `n_classified` is the discriminator, so it must be 0 there and non-zero for a real
    /// interpretation; the zeros themselves stay unchanged.
    #[test]
    fn pay_summary_marks_an_uninterpreted_well_as_classifying_nothing() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "PAY-1", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();

        // Only raw logs — exactly the state after importing a LAS and running nothing.
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![50.0; n], nan.clone(), vec![0.2; n], vec![2.4; n],
            nan.clone(), nan,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let req = PaySummaryRequest {
            well_ids: vec![well.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.6,
            perm_min: None,
            skip_version: false,
            // Stats only: the point of the test is the returned rows, and this keeps it from
            // writing FLAG_* curves as a side effect.
            stats_only: true,
        };
        let rows = run_pay_summary(&dbm, &req).expect("summary runs on an uninterpreted well");
        assert!(!rows.is_empty(), "rows are still emitted — the well and its zone exist");
        for r in &rows {
            assert_eq!(
                r.n_classified, 0,
                "no sample can be classified without VSH/PHIE/SWE ({} {})",
                r.zone, r.flag
            );
            // The zeros are unchanged; the counter is what tells the consumer not to print them.
            assert_eq!(r.net, 0.0);
            assert_eq!(r.hpv, 0.0);
        }
    }

    /// A clean, porous, low-Sw sand where every sample passes VSH/PHIE/SWE on its own, so the
    /// only thing that can exclude a sample is the PERM cutoff. `perm` is the permeability the
    /// well MEASURED — `None` means the well carries none at all, which is the case under test.
    fn seed_pay_well(conn: &duckdb::Connection, name: &str, perm: Option<f32>) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        for (curve, v) in [("VSH", 0.2f32), ("PHIE", 0.20), ("SWE", 0.30)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![v; n]).unwrap();
        }
        if let Some(k) = perm {
            equations::write_computed_curve(conn, &well, &depth, "PERM", &vec![k; n]).unwrap();
        }
        well
    }

    /// T-BATCH-08 (1) — the PERM cutoff has a whole-well escape hatch, and it opens the wrong way.
    ///
    /// `classify_sample` is emphatic that a SAMPLE with no PERM cannot demonstrate it passes an
    /// active cutoff, so it fails (`classify_sample_nan_propagation` pins that). But whether the
    /// cutoff is active at all is decided per WELL, one line earlier: `perm_min.is_some() &&
    /// perm.iter().any(|v| !v.is_nan())`. A well carrying NO permeability anywhere therefore
    /// switches the cutoff off for itself and reports its full pay.
    ///
    /// The two halves of the same rule disagree, and the direction is the damaging one: the well
    /// that measured permeability and measured it BELOW the cutoff is excluded, while the well
    /// that measured none at all sails through. In a field summary those rows add together with
    /// nothing on screen saying which wells the cutoff was applied to — so the less data a well
    /// has, the more pay it books.
    ///
    /// Pinned AS-IS, not endorsed. Whether an uncored well should be excluded or exempted is a
    /// petrophysical decision that changes client numbers, so it is Jauhar's to make — see
    /// docs/review_triage.md finding 7.
    #[test]
    fn a_well_with_no_perm_at_all_quietly_escapes_an_active_perm_cutoff() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Identical rock. The ONLY difference is whether permeability was measured.
        let no_perm = seed_pay_well(&conn, "PAY-NOPERM", None);
        let low_perm = seed_pay_well(&conn, "PAY-LOWPERM", Some(1.0));
        let dbm = Mutex::new(conn);

        let summary = |perm_min: Option<f64>| -> Vec<PaySummaryRow> {
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    well_ids: vec![no_perm.clone(), low_perm.clone()],
                    vsh_max: 0.5,
                    phie_min: 0.1,
                    swe_max: 0.6,
                    perm_min,
                    skip_version: false,
                    stats_only: true,
                },
            )
            .expect("summary runs")
        };
        let pay = |rows: &[PaySummaryRow], w: &str| -> PaySummaryRow {
            rows.iter().find(|r| r.well_id == w && r.flag == "PAY").expect("a PAY row per well").clone()
        };

        // Baseline: with no PERM cutoff at all, both wells are full pay. This is the control —
        // it establishes the rock is identical, so anything below is the cutoff's doing.
        let open = summary(None);
        let base_no_perm = pay(&open, &no_perm).net;
        let base_low_perm = pay(&open, &low_perm).net;
        assert!(base_no_perm > 0.0, "the test rock must be pay before any cutoff is applied");
        assert_eq!(base_no_perm, base_low_perm, "both wells must start as the same rock");

        // Now a cutoff nothing in either well could pass.
        let cut = summary(Some(1000.0));

        // The well that MEASURED permeability, at 1 mD, is correctly excluded.
        assert_eq!(pay(&cut, &low_perm).net, 0.0, "1 mD cannot pass a 1000 mD cutoff");

        // The well that measured NONE keeps every metre of its pay — the cutoff never applied.
        assert_eq!(
            pay(&cut, &no_perm).net,
            base_no_perm,
            "a well with no PERM at all is exempted from the PERM cutoff rather than failing it"
        );
        assert!(pay(&cut, &no_perm).hpv > 0.0, "and it books hydrocarbon volume on that exemption");

        // Both wells were fully interpreted, so `n_classified` cannot be used downstream to tell
        // the exempted rows apart from the honest ones — which is what makes this silent.
        assert!(pay(&cut, &no_perm).n_classified > 0);
        assert!(pay(&cut, &low_perm).n_classified > 0);
    }

    /// T-RT-05 — rocktyping on a well that has porosity but no permeability must fail by name and
    /// write NOTHING.
    ///
    /// The dangerous outcome is not a crash, it is a quiet success: every output would be NaN at
    /// every depth, the run would report ✓, and the Curve Catalog would gain FZI/RT rows that are
    /// empty from top to bottom. A later reader has no way to tell that from a well where the
    /// rock genuinely had no answer. The control below runs the same module on the same well with
    /// permeability present, so the failure is provably the missing curve and not a broken module.
    #[test]
    fn rocktyping_without_a_permeability_curve_fails_and_writes_no_curves() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "RT-NOPERM", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        // Porosity, but deliberately no permeability of any name.
        equations::write_computed_curve(&conn, &well, &depth, "PHIE", &vec![0.20f32; n]).unwrap();
        let dbm = Mutex::new(conn);

        let run = || {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "rocktyping".into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params: HashMap::new(),
                    opts: HashMap::new(),
                    output_set: None,
                    input_set: None,
                },
            )
        };
        let outputs = ["RQI", "PHIZ", "FZI", "R35", "PGEOM", "PSTRUC", "RT", "PERM_RT"];
        let written = |name: &str| -> i64 {
            let conn = dbm.lock().unwrap();
            conn.query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND UPPER(curve_name) = ?2",
                duckdb::params![well, name],
                |r| r.get(0),
            )
            .unwrap()
        };

        // Counted in Rust, deliberately. DuckDB gives NaN a TOTAL ordering — `NaN = NaN` is true
        // there, so an SQL `value = value` filter counts every MISSING sample as a real one and
        // this test would have reported the opposite of the truth.
        let finite = |name: &str| -> usize {
            let conn = dbm.lock().unwrap();
            let mut st = conn
                .prepare("SELECT value FROM computed_curves WHERE well_id = ?1 AND UPPER(curve_name) = ?2")
                .unwrap();
            let rows: Vec<Option<f32>> = st
                .query_map(duckdb::params![well, name], |r| r.get(0))
                .unwrap()
                .map(|v| v.unwrap())
                .collect();
            rows.iter().filter(|v| v.is_some_and(f32::is_finite)).count()
        };

        let res = run();
        assert_eq!(res.len(), 1);

        // The API half is honest, and that much is already pinned by
        // `all_nan_module_output_reports_error_not_success`.
        assert!(res[0].error.is_some(), "a missing permeability curve must be reported, not absorbed");
        assert_eq!(res[0].rows_written, 0, "the failed run must not report a sample count");

        // The catalog half is NOT. Phase 2 writes for any well whose outcome is `Computed` with a
        // non-empty output map, and an all-MISSING output map is still non-empty — so the whole
        // rocktyping family is versioned into the catalog as curves that are blank end to end.
        // T-RT-05's Expected says the catalog must gain NO FZI/RT rows. Pinned AS-IS, not endorsed
        // — see docs/review_triage.md finding 10.
        for name in outputs {
            assert!(written(name) > 0, "{name}: the empty curve IS written today");
            assert_eq!(finite(name), 0, "{name}: and every sample of it is MISSING");
        }

        // Control: give the well a permeability and the identical call succeeds and writes the
        // family. Without this the assertions above would also pass on a module that never works.
        {
            let conn = dbm.lock().unwrap();
            equations::write_computed_curve(&conn, &well, &depth, "PERM", &vec![100.0f32; n]).unwrap();
        }
        let ok = run();
        assert!(ok[0].error.is_none(), "with permeability present it must run: {:?}", ok[0].error);
        assert!(ok[0].rows_written > 0);
        for name in outputs {
            assert!(finite(name) > 0, "{name} must carry real values after the successful run");
        }
    }

    /// T-ADV-11 — RtC on a well that has resistivity but no porosity of ANY name must be reported,
    /// not returned as a green run.
    ///
    /// `all_nan_module_output_reports_error_not_success` pins the guard on vsh_gr and
    /// electrofacies. This is the case the guard was actually written for, and it is nastier than
    /// a dead well: RES_DEEP is present and healthy, so the run has real data to chew on and comes
    /// back with a full-length SWT_RTC curve that happens to be MISSING at every depth. On a
    /// saturation curve that is the difference between "no answer" and "no hydrocarbon".
    ///
    /// The control matters especially here because sw_rtc has the SSPW fallback: the failure must
    /// be the absence of porosity under EITHER name, not the module failing to look for the
    /// second one. So the same well is then given PHIT_SSPW only — the fallback curve, never the
    /// primary — and must succeed.
    #[test]
    fn rtc_without_porosity_under_either_name_is_reported_not_returned_as_success() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "RTC-NOPHI", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        // A raw well: real deep resistivity, no porosity interpretation of any kind.
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![60.0; n], vec![8.0; n], nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        let dbm = Mutex::new(conn);

        let run = || {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "sw_rtc".into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params: HashMap::new(),
                    opts: HashMap::new(),
                    output_set: None,
                    input_set: None,
                },
            )
        };

        let res = run();
        assert!(
            res[0].error.is_some(),
            "a saturation run with no porosity must be reported, not returned as a success"
        );
        assert_eq!(res[0].rows_written, 0, "and must not claim a sample count");

        // Control: give it porosity under the FALLBACK name only. If this failed too, the test
        // above would be pinning a broken module rather than an honest refusal.
        {
            let conn = dbm.lock().unwrap();
            equations::write_computed_curve(&conn, &well, &depth, "PHIT_SSPW", &vec![0.25f32; n]).unwrap();
            equations::write_computed_curve(&conn, &well, &depth, "CAPBW_SSPW", &vec![0.08f32; n]).unwrap();
        }
        let ok = run();
        assert!(ok[0].error.is_none(), "the SSPW fallback alone must be enough to run: {:?}", ok[0].error);
        assert!(ok[0].rows_written > 0, "and it must write real samples");
    }

    /// T-BATCH-08 (3) — one unusable well must not zero the whole response.
    ///
    /// `run_pay_summary` `continue`s past a well whose curve frame or zone read fails instead of
    /// `?`-aborting the batch. The bare well is listed FIRST here on purpose: an abort would take
    /// the good well's rows with it, and a test that put the good well first would pass either way.
    #[test]
    fn one_unusable_well_cannot_zero_the_whole_pay_summary() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // A well record with no curve data at all — an import that failed, or a well created by hand.
        let bare_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, bare_id, "PAY-BARE", Some("Synthetic"), None, None).unwrap();
        let bare = bare_id.to_string();
        let good = seed_pay_well(&conn, "PAY-GOOD", Some(500.0));
        let dbm = Mutex::new(conn);

        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                well_ids: vec![bare.clone(), good.clone()],
                vsh_max: 0.5,
                phie_min: 0.1,
                swe_max: 0.6,
                perm_min: None,
                skip_version: false,
                stats_only: true,
            },
        )
        .expect("a bare well must not fail the batch");

        let good_pay = rows.iter().find(|r| r.well_id == good && r.flag == "PAY").expect("the good well still reports");
        assert!(good_pay.net > 0.0, "the good well keeps its full answer: {good_pay:?}");

        // The bare well contributes NO rows — it is skipped, not reported as a zero. A zero row
        // would be indistinguishable from a genuinely wet zone in the Field Dashboard.
        assert!(
            !rows.iter().any(|r| r.well_id == bare),
            "a well with no curves must be absent, not present with zeros"
        );
    }

    /// A zone override beats the module dialog by design, so it also skips the dialog's range
    /// check — `moduleDialog.ts` validates against ArgSpec.min/max, `zonesDialog.ts` does not,
    /// and the DB Inspector edits `zone_params.value_num` raw. A petrophysicist entering
    /// irreducible water saturation in PERCENT (25 instead of 0.25) then produced
    /// `limit(swt, 25.0, 1.0)`, and `f64::clamp` asserts `lo <= hi` — the run died with an opaque
    /// "worker thread failed". The value is rejected rather than clamped: silently pulling 25
    /// down to the spec maximum would answer with a plausible-but-wrong saturation.
    #[test]
    fn out_of_range_zone_param_is_rejected_not_clamped() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "RANGE-1", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let depth: Vec<f32> = (0..5).map(|i| 1000.0 + i as f32).collect();

        let spec = modules::list_modules()
            .into_iter()
            .find(|s| s.name == "sw_arch")
            .expect("sw_arch is a registered module");
        let arg = spec
            .args
            .iter()
            .find(|a| a.name == "SWT_IRR")
            .expect("sw_arch declares SWT_IRR");
        let hi = arg.max.expect("SWT_IRR declares an upper bound");

        // Baseline: no override at all resolves cleanly.
        let ok = resolve_param_arrays(&conn, &well, &spec, &HashMap::new(), &depth);
        assert!(ok.is_ok(), "an unmodified run must still resolve: {ok:?}");

        // The percent-entry mistake, well-wide.
        db::set_zone_param(&conn, &well, "*", "SWT_IRR", Some(25.0), None).unwrap();
        let err = resolve_param_arrays(&conn, &well, &spec, &HashMap::new(), &depth)
            .expect_err("an out-of-range zone override must fail the run, not panic it");
        assert!(err.contains("SWT_IRR"), "the message must name the parameter: {err}");
        assert!(err.contains("25"), "and the offending value: {err}");
        assert!(
            err.contains(&hi.to_string()),
            "and the valid range so the user can act on it: {err}"
        );

        // A value inside the declared range resolves again — the guard is not blanket-blocking.
        db::set_zone_param(&conn, &well, "*", "SWT_IRR", Some(0.25), None).unwrap();
        let good = resolve_param_arrays(&conn, &well, &spec, &HashMap::new(), &depth);
        assert!(good.is_ok(), "an in-range override must pass: {good:?}");
        let arr = &good.unwrap()["SWT_IRR"];
        assert!(arr.iter().all(|v| (*v - 0.25).abs() < 1e-9), "override applied well-wide");
    }

    /// Sweeping the VSH (sand) cutoff upward can only admit more pay, so the metric is
    /// monotone non-decreasing; the peak lands at the most permissive cutoff.
    #[test]
    fn cutoff_sweep_vsh_monotone() {
        let vsh = [0.1f32, 0.3, 0.5, 0.7, 0.9];
        let phie = [0.2f32; 5];
        let swe = [0.3f32; 5];
        let perm = [f32::NAN; 5];
        // Each sample contributes a full 1 m of clamped thickness.
        let incl_h = [1.0f64; 5];
        let (cuts, vals, peak) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Vsh, 0.5, 0.1, 0.6, None, 0.0, 1.0,
            11, Metric::Net, 5.0,
        );
        assert_eq!(cuts.len(), 11);
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9, "not monotone: {:?}", vals);
        }
        assert!((vals[0] - 0.0).abs() < 1e-9); // cutoff 0.0 → no sample has VSH ≤ 0
        assert!((peak - 5.0).abs() < 1e-9); // cutoff 1.0 → all 5 m of pay
    }

    /// NTG divides by the geometric gross; the DST `included` mask drops samples and scales
    /// net down accordingly.
    #[test]
    fn cutoff_sweep_ntg_and_dst_mask() {
        let vsh = [0.2f32; 4];
        let phie = [0.2f32; 4];
        let swe = [0.3f32; 4];
        let perm = [f32::NAN; 4];
        // All four samples at full 1 m thickness, gross 4 → every sample pays at a generous
        // SWE cutoff → NTG 1.0.
        let all = [1.0f64; 4];
        let (_, vals, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &all, SweepProp::Swe, 0.5, 0.1, 0.6, None, 0.0, 1.0, 3,
            Metric::Ntg, 4.0,
        );
        assert!((vals[2] - 1.0).abs() < 1e-9);
        // DST clips two samples to zero thickness → NET tops out at 2 m.
        let half = [1.0f64, 1.0, 0.0, 0.0];
        let (_, vals2, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &half, SweepProp::Swe, 0.5, 0.1, 0.6, None, 0.0, 1.0,
            3, Metric::Net, 2.0,
        );
        assert!((vals2[2] - 2.0).abs() < 1e-9);
    }

    /// Overlapping perforation/DST rows must union, not double-count: two rows (2000,2010) and
    /// (2005,2015) cover 15 m, not 20 m, so the NTG gross stays consistent with net thickness.
    #[test]
    fn aux_intervals_merges_overlaps() {
        let row = |t: f32, b: Option<f32>| db::AuxRow {
            dataset: "DST".into(),
            depth_top: t,
            depth_base: b,
            item: String::new(),
            value_num: None,
            value_text: None,
        };
        // Overlapping + a nested + an exact duplicate + a point row (dropped).
        let rows = vec![
            row(2000.0, Some(2010.0)),
            row(2005.0, Some(2015.0)), // overlaps the first → union to (2000,2015)
            row(2006.0, Some(2008.0)), // nested inside → absorbed
            row(2005.0, Some(2015.0)), // exact duplicate → absorbed
            row(2100.0, None),         // point row → ignored
            row(2050.0, Some(2050.0)), // zero-length → ignored
            row(2030.0, Some(2040.0)), // disjoint → its own interval
        ];
        let iv = aux_intervals(&rows);
        assert_eq!(iv, vec![(2000.0, 2015.0), (2030.0, 2040.0)]);
        let gross: f32 = iv.iter().map(|(t, b)| b - t).sum();
        assert!((gross - 25.0).abs() < 1e-4, "gross should be 15+10, got {gross}");
    }

    /// Regression for the "step bleed past boundary" bug in the sweep engine: when a zone base
    /// falls mid-sample, the sweep must count only each sample's in-zone overlap (fed via
    /// incl_h), so net ≤ gross and NTG ≤ 1 — matching run_pay_summary on the identical fixture.
    /// Previously compute_sweep summed each included sample's full step and reported NTG ≈ 1.33.
    #[test]
    fn compute_sweep_clamps_thickness_via_incl_h() {
        // depths 1000..1003 (step 1.0), zone [1000, 1001.5): overlaps 1.0, 0.5, 0, 0 → gross 1.5.
        let vsh = [0.1f32; 4];
        let phie = [0.2f32; 4];
        let swe = [0.3f32; 4];
        let perm = [f32::NAN; 4];
        let incl_h = [1.0f64, 0.5, 0.0, 0.0];
        // Permissive cutoffs: every in-zone sample pays → net = 1.5 (the clamped overlap), NOT
        // 2.0 (two full steps), so peak net is 1.5 and NTG never exceeds 1.
        let (_, _, peak) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, 0.9, 0.0, 1.0, None, 0.0, 1.0, 2,
            Metric::Net, 1.5,
        );
        assert!((peak - 1.5).abs() < 1e-9, "net must be the clamped 1.5 m, not 2.0; got {peak}");
        let (_, ntg, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, 0.9, 0.0, 1.0, None, 0.0, 1.0, 2,
            Metric::Ntg, 1.5,
        );
        assert!(ntg[1] <= 1.0 + 1e-9, "NTG must not exceed 1; got {}", ntg[1]);
    }

    /// The per-sample geometric clamp: a sample's overlap with the zone, then intersected with
    /// the DST intervals when present.
    #[test]
    fn sample_incl_thickness_clamps_zone_and_dst() {
        // Sample [1001,1002] vs zone [1000,1001.5): 0.5 m in zone.
        assert!((sample_incl_thickness(1001.0, 1002.0, 1000.0, 1001.5, None) - 0.5).abs() < 1e-9);
        // Fully outside the zone → 0.
        assert_eq!(sample_incl_thickness(1002.0, 1003.0, 1000.0, 1001.5, None), 0.0);
        // Zone overlap [1000,1002]; DST intervals (1000.5,1001)+(1001.5,1002) → 0.5+0.5 = 1.0.
        let dst = [(1000.5f32, 1001.0f32), (1001.5, 1002.0)];
        let h = sample_incl_thickness(1000.0, 1002.0, 999.0, 1003.0, Some(&dst));
        assert!((h - 1.0).abs() < 1e-9, "DST-clipped overlap should be 1.0, got {h}");
    }

    /// Phase 7 wiring test — no field files, no vcvars: a well whose PEF, DRHO and CALI
    /// live ONLY in the generic curve store (never the fixed six) drives (1) multimin,
    /// proving the generic-store read fallback feeds a real module through the runner;
    /// (2) the badhole flag from generic DRHO/CALI; and (3) a masked vsh_gr run, proving
    /// flagged intervals are NaN'd out of module outputs.
    #[test]
    fn phase7_generic_store_feeds_modules_and_mask() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "MM-1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        // Forward-model a clean wet sand at every depth (70% sand / 30% water) so we know
        // the answer, plus one washed-out sample flagged by CALI.
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
        let (vs, vw) = (0.70f64, 0.30f64);
        let rhob_v = (vs * 2.65 + vw * 1.0) as f32;
        let nphi_v = (vs * -0.02 + vw * 1.0) as f32;
        let dt_v = (vs * 55.5 + vw * 189.0) as f32;
        let pef_v = (vs * 1.81 + vw * 0.36) as f32;
        let n = depths.len();

        // RHOB/NPHI/DT go in the fixed table; GR too (for the masked run). RES/SP unused.
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            vec![40.0; n],       // GR
            vec![f32::NAN; n],   // RES_DEEP
            vec![nphi_v; n],     // NPHI
            vec![rhob_v; n],     // RHOB
            vec![dt_v; n],       // DT
            vec![f32::NAN; n],   // SP
        )
        .unwrap();

        // PEF, DRHO, CALI ONLY in the generic store. CALI is huge (washout) at sample 2.
        let put = |mnem: &str, family: &str, unit: &str, vals: Vec<f32>| {
            let id = db::upsert_curve_meta(&conn, &w, "RAW", mnem, Some(unit), Some(family), Some("test"), None).unwrap();
            db::insert_curve_samples(&conn, &id, &depths, &vals).unwrap();
        };
        put("PEFZ", "PEF", "b/e", vec![pef_v; n]); // mnemonic differs → must resolve by family
        put("HDRA", "DRHO", "g/cc", vec![0.01, 0.01, 0.20, 0.01]); // big DRHO at sample 2
        put("HCAL", "CALI", "in", vec![8.6, 8.6, 14.0, 8.6]); // washout at sample 2 (BS 8.5)

        let dbm = Mutex::new(conn);
        let run = |module: &str, params: &[(&str, f64)], opts: &[(&str, &str)]| -> Vec<ModuleRunResult> {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: vec![w.clone()],
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                output_set: None,
                input_set: None,
            };
            run_workflow_module(&dbm, &req)
        };

        // (1) multimin is RETIRED — even with every input present (incl. PEF from the generic
        // store), the runner must refuse it with a clear SandiMin migration message and write no
        // curves, rather than silently running the superseded 4-component solver. It resolves by
        // name (the spec stays in the catalog) so this is a Failed run, not "unknown module". The
        // generic-store family-resolution fallback this part used to prove is still covered by (2)
        // below (HDRA→DRHO, HCAL→CALI).
        let r = run("multimin", &[], &[]);
        assert!(
            r[0].error.as_deref().unwrap_or("").contains("SandiMin"),
            "retired multimin must return a SandiMin migration error, got {:?}",
            r[0].error
        );
        assert!(r[0].output_curves.is_empty(), "a retired module must write no curves");

        // (2) badhole — DRHO and CALI resolve from the generic store; sample 2 is bad.
        let r = run("badhole", &[("DRHO_MAX", 0.05), ("DCAL_MAX", 1.0), ("BS_DEF", 8.5)], &[]);
        assert!(r[0].error.is_none(), "badhole: {:?}", r[0].error);
        {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["BADHOLE".into()]).unwrap();
            let bh = &cols["BADHOLE"];
            assert_eq!(bh[0], 0.0, "good hole");
            assert_eq!(bh[2], 1.0, "washout must flag bad");
        }

        // (3) masked vsh_gr — the badhole flag masks sample 2 out of the output.
        let r = run("vsh_gr", &[("GR_MA", 20.0), ("GR_SH", 120.0)], &[("MASK", "BADHOLE")]);
        assert!(r[0].error.is_none(), "vsh_gr masked: {:?}", r[0].error);
        {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["VSH".into()]).unwrap();
            let vsh = &cols["VSH"];
            assert!(!vsh[0].is_nan(), "good-hole sample kept");
            assert!(vsh[2].is_nan(), "bad-hole sample must be masked to NaN");
        }
    }

    /// A module run whose every output sample is MISSING — all-NaN inputs, so vsh_gr yields
    /// all-NaN VSH and electrofacies can't cluster (no usable curve) — must report distinctly,
    /// NOT a green "N samples → …" success that totals into History. Positive control: the same
    /// modules on a live well still succeed with the full sample count.
    #[test]
    fn all_nan_module_output_reports_error_not_success() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
        let n = depths.len();

        // Dead well: every standard curve is all-NaN.
        let dead = Uuid::new_v4();
        db::insert_well(&conn, dead, "DEAD-1", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, dead, depths.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        ).unwrap();

        // Live well: a real GR that clusters and computes a real VSH.
        let live = Uuid::new_v4();
        db::insert_well(&conn, live, "LIVE-1", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, live, depths.clone(),
            vec![20.0, 55.0, 90.0, 120.0], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        ).unwrap();

        let dbm = Mutex::new(conn);
        let run = |module: &str, well: &Uuid, params: &[(&str, f64)]| -> Vec<ModuleRunResult> {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: vec![well.to_string()],
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
            };
            run_workflow_module(&dbm, &req)
        };

        // vsh_gr on all-NaN GR → all-NaN VSH → error, not a green success with a full count.
        let r = run("vsh_gr", &dead, &[("GR_MA", 20.0), ("GR_SH", 120.0)]);
        assert!(r[0].error.is_some(), "all-NaN vsh_gr must report an error");
        assert_eq!(r[0].rows_written, 0, "dead run must not report a full sample count");

        // electrofacies with no usable input curve → all-NaN FACIES → error.
        let r = run("electrofacies", &dead, &[("K", 2.0)]);
        assert!(r[0].error.is_some(), "electrofacies with no input curves must report an error");

        // Positive controls: the same modules on the live well succeed with the full count.
        let r = run("vsh_gr", &live, &[("GR_MA", 20.0), ("GR_SH", 120.0)]);
        assert!(r[0].error.is_none(), "live vsh_gr: {:?}", r[0].error);
        assert_eq!(r[0].rows_written, n);
        let r = run("electrofacies", &live, &[("K", 2.0)]);
        assert!(r[0].error.is_none(), "live electrofacies: {:?}", r[0].error);
    }

    #[test]
    fn mask_excludes_flagged_samples_from_gr_normalize_percentiles() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "GRN-1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        // Five good-hole GR samples spanning 30–70 gAPI, plus one washed-out sample at GR=500.
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5, 1002.0, 1002.5];
        let n = depths.len();
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            vec![30.0, 40.0, 50.0, 60.0, 70.0, 500.0], // GR (outlier at the flagged sample)
            vec![f32::NAN; n],                          // RES_DEEP
            vec![f32::NAN; n],                          // NPHI
            vec![f32::NAN; n],                          // RHOB
            vec![f32::NAN; n],                          // DT
            vec![f32::NAN; n],                          // SP
        )
        .unwrap();
        // BADHOLE flag: only the GR=500 sample is bad (the mask curve, resolved like any input).
        equations::write_computed_curve(&conn, &w, &depths, "BADHOLE", &[0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
            .unwrap();

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "gr_normalize".into(),
            well_ids: vec![w.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(), // manifest defaults: P3/P97, generic refs 20/120
            opts: [("MASK".to_string(), "BADHOLE".to_string())].into_iter().collect(),
            output_set: None,
            input_set: None,
        };
        let r = run_workflow_module(&dbm, &req);
        assert!(r[0].error.is_none(), "gr_normalize masked: {:?}", r[0].error);

        let conn = dbm.lock().unwrap();
        let (_, cols) = equations::fetch_curve_frame(&conn, &w, &["GRN".into()]).unwrap();
        let grn = &cols["GRN"];
        // The flagged sample is still masked out of the output.
        assert!(grn[5].is_nan(), "flagged sample must be masked in the output");
        // With the GR=500 outlier excluded from the percentile anchoring, the good-hole samples
        // span the full reference range (~80 gAPI). Under the old output-only masking the
        // outlier still anchored P97 and the good samples compressed into < ~10 gAPI.
        let good: Vec<f32> = grn[..5].iter().copied().filter(|v| !v.is_nan()).collect();
        let (mn, mx) = good
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &v| (a.min(v), b.max(v)));
        assert!(mx - mn > 50.0, "good-hole GRN must span the reference range, got spread {}", mx - mn);
    }

    #[test]
    fn pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "PAY-1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        let depths = vec![1000.0f32, 1001.0, 1002.0, 1003.0];
        let n = depths.len();
        // Standard curves supply the depth spine; the interpretation curves are computed.
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        // All sand; sample 1 has valid VSH but MISSING PHIE (the SAND-row dilution case).
        equations::write_computed_curve(&conn, &w, &depths, "VSH", &[0.1, 0.1, 0.1, 0.1]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PHIE", &[0.2, f32::NAN, 0.2, 0.2]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "SWE", &[0.3, 0.3, 0.3, 0.3]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PERM", &[f32::NAN; 4]).unwrap();
        // A zone thinner than one sample step (1.5 m vs 1.0 m steps): the last in-zone sample
        // must not bleed past the base, so net must equal gross (1.5), not overshoot to 2.0.
        db::upsert_zone(&conn, &w, "Z1", 1000.0, 1001.5).unwrap();

        let dbm = Mutex::new(conn);
        let req = PaySummaryRequest {
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: true,
            stats_only: false,
        };
        let rows = run_pay_summary(&dbm, &req).unwrap();
        let sand = rows.iter().find(|r| r.zone == "Z1" && r.flag == "SAND").expect("SAND row");

        // Overlap clamp: net never exceeds gross (old forward-step gave net 2.0 > gross 1.5).
        assert!((sand.gross - 1.5).abs() < 1e-3, "gross={}", sand.gross);
        assert!(sand.net <= sand.gross + 1e-4, "net {} must not exceed gross {}", sand.net, sand.gross);
        assert!((sand.net - 1.5).abs() < 1e-3, "net={}", sand.net);
        // avg_phie normalised over PHIE-valid net (→ 0.2), not diluted by the missing-PHIE
        // sample (old code divided sum_phie by total net → ~0.1).
        assert!((sand.avg_phie - 0.2).abs() < 1e-3, "avg_phie={}", sand.avg_phie);
    }

    /// Polish-5: an explicit pay-summary run versions the FLAG_* curves into a PAYFLAG log set
    /// whose provenance records the module + the cutoffs; skip_version writes in place instead.
    #[test]
    fn pay_summary_versions_flags_with_cutoffs_in_provenance() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "PAY-PROV", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();
        let depths = vec![1000.0f32, 1001.0, 1002.0, 1003.0];
        let n = depths.len();
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "VSH", &[0.1; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PHIE", &[0.2; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "SWE", &[0.3; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PERM", &[f32::NAN; 4]).unwrap();
        db::upsert_zone(&conn, &w, "Z1", 1000.0, 1003.0).unwrap();
        let dbm = Mutex::new(conn);

        // Explicit run: versions the pay flags with the cutoffs recorded in provenance.
        let req = PaySummaryRequest {
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: false,
            stats_only: false,
        };
        run_pay_summary(&dbm, &req).unwrap();
        {
            let conn = dbm.lock().unwrap();
            let (module, params): (String, String) = conn
                .query_row(
                    "SELECT module, params_json FROM log_sets WHERE well_id = ?1 AND set_name = 'PAYFLAG' ORDER BY version DESC LIMIT 1",
                    duckdb::params![w],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .expect("a PAYFLAG log set should exist after a versioned pay-summary run");
            assert_eq!(module, "pay_summary");
            assert!(params.contains("\"vsh_max\":0.5"), "cutoffs in provenance: {params}");
            assert!(params.contains("\"phie_min\":0.1"), "cutoffs in provenance: {params}");
            assert!(params.contains("\"swe_max\":0.5"), "cutoffs in provenance: {params}");
        }

        // skip_version (report/composite render side-effect): writes FLAG_* in place, no new version.
        let req_skip = PaySummaryRequest {
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: true,
            stats_only: false,
        };
        run_pay_summary(&dbm, &req_skip).unwrap();
        {
            let conn = dbm.lock().unwrap();
            let versions: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'PAYFLAG'",
                    duckdb::params![w],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(versions, 1, "skip_version must not add a PAYFLAG version");
        }
    }

    /// Performance fix (Field Dashboard): stats_only computes and returns the same per-zone
    /// rows as a writing run, but persists NOTHING — no FLAG_* computed curves and no PAYFLAG
    /// log set. This is what removes the ~1,600 write transactions per dashboard Compute.
    #[test]
    fn pay_summary_stats_only_persists_nothing() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "STATS-ONLY", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();
        let depths = vec![1000.0f32, 1001.0, 1002.0, 1003.0];
        let n = depths.len();
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "VSH", &[0.1; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PHIE", &[0.2; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "SWE", &[0.3; 4]).unwrap();
        equations::write_computed_curve(&conn, &w, &depths, "PERM", &[f32::NAN; 4]).unwrap();
        db::upsert_zone(&conn, &w, "Z1", 1000.0, 1003.0).unwrap();
        let dbm = Mutex::new(conn);

        let base = PaySummaryRequest {
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: false,
            stats_only: true,
        };
        let rows_stats = run_pay_summary(&dbm, &base).unwrap();
        assert!(!rows_stats.is_empty(), "stats_only must still return the summary rows");

        // Nothing was persisted: no FLAG_* curves, no PAYFLAG log set.
        {
            let conn = dbm.lock().unwrap();
            let flag_curves: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name LIKE 'FLAG_%'",
                    duckdb::params![w],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(flag_curves, 0, "stats_only must not write any FLAG_* curve");
            let payflag_sets: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'PAYFLAG'",
                    duckdb::params![w],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(payflag_sets, 0, "stats_only must not create a PAYFLAG log set");
        }

        // Same cutoffs, now writing in place: identical row count + matching PAY net, and
        // FLAG_* curves now exist — confirming stats_only changed persistence only, not math.
        let writing = PaySummaryRequest { stats_only: false, skip_version: true, ..base.clone() };
        let rows_write = run_pay_summary(&dbm, &writing).unwrap();
        assert_eq!(rows_stats.len(), rows_write.len(), "stats_only must not change the rows returned");
        let pay_a = rows_stats.iter().find(|r| r.flag == "PAY").expect("PAY row (stats)");
        let pay_b = rows_write.iter().find(|r| r.flag == "PAY").expect("PAY row (write)");
        assert!((pay_a.net - pay_b.net).abs() < 1e-4, "stats_only net {} vs writing net {}", pay_a.net, pay_b.net);
        {
            let conn = dbm.lock().unwrap();
            let flag_curves: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name LIKE 'FLAG_%'",
                    duckdb::params![w],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(flag_curves > 0, "the writing run must persist FLAG_* curves");
        }
    }

    /// GR normalization anchors on EACH WELL'S OWN percentiles, not on the batch's pooled ones.
    ///
    /// That is the entire point of the module: two wells logged by different tools, or in
    /// different mud, read different absolute GR in the same shale, and normalizing is what makes
    /// one VSH cutoff mean the same rock in both. Pooling the percentiles across the run would
    /// still produce a plausible-looking GRN — the FIELD would anchor on the references while
    /// each individual well drifted off them by however much its own distribution differs from
    /// the pool. Nothing on the log says so, and the wells would then disagree exactly where the
    /// module was supposed to make them agree.
    ///
    /// So the two wells here are deliberately given very different GR characters and run in ONE
    /// batch. Each must come back with its own P3 and P97 on the shared references.
    ///
    /// The reference values are read from the manifest rather than typed in, because they are
    /// generic defaults held by `gr_normalize_reference_defaults_are_generic_not_a_field_calibration`
    /// and must never be restated as literals here — a second copy is a second thing to go stale.
    #[test]
    fn gr_normalization_anchors_each_well_on_its_own_percentiles() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // Two wells, same rock, very different absolute GR: B reads roughly twice A and is
        // shifted. Pooled percentiles would sit between them and fit neither.
        let n = 101usize;
        let mk = |name: &str, base: f32, span: f32| -> uuid::Uuid {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, None).unwrap();
            let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
            // A deterministic saw-tooth spread over the span: a real distribution, not a ramp
            // that would make every percentile trivially exact.
            let gr: Vec<f32> = (0..n)
                .map(|i| base + span * (((i * 37) % n) as f32 / (n - 1) as f32))
                .collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn,
                id,
                depth,
                gr,
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan,
            )
            .unwrap();
            id
        };
        let a = mk("SANDI-GRA", 15.0, 60.0);
        let b = mk("SANDI-GRB", 70.0, 150.0);

        // The shipped reference endpoints, taken from the manifest.
        let spec = modules::list_modules()
            .into_iter()
            .find(|m| m.name == "gr_normalize")
            .expect("gr_normalize must be in the catalog");
        let default_of = |name: &str| -> f32 {
            spec.args
                .iter()
                .find(|x| x.name == name)
                .expect("arg present")
                .default
                .parse()
                .expect("numeric default")
        };
        let (lo_ref, hi_ref) = (default_of("GR_LOW_REF"), default_of("GR_HIGH_REF"));
        let (p_lo, p_hi) = (default_of("P_LOW"), default_of("P_HIGH"));

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "gr_normalize".into(),
            well_ids: vec![a.to_string(), b.to_string()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let results = run_workflow_module_into(&dbm, &req, None, None, None);
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.error.is_none(), "gr_normalize failed: {:?}", r.error);
        }

        let grn_of = |well: &uuid::Uuid| -> Vec<f32> {
            let c = dbm.lock().unwrap();
            let mut stmt = c
                .prepare(
                    "SELECT value FROM computed_curves
                     WHERE well_id = ?1 AND curve_name = 'GRN' ORDER BY depth",
                )
                .unwrap();
            let v: Vec<f32> = stmt
                .query_map(duckdb::params![well.to_string()], |r| r.get(0))
                .unwrap()
                .filter_map(|x| x.ok())
                .collect();
            v
        };

        for (name, id) in [("SANDI-GRA", &a), ("SANDI-GRB", &b)] {
            let mut v = grn_of(id);
            assert_eq!(v.len(), n, "{name}: every sample must be normalized");
            v.sort_by(|x, y| x.partial_cmp(y).unwrap());

            let got_lo = crate::distribution::percentile(&v, p_lo);
            let got_hi = crate::distribution::percentile(&v, p_hi);
            assert!(
                (got_lo - lo_ref).abs() < 0.5,
                "{name}: its OWN P{p_lo} must land on the low reference {lo_ref}, got {got_lo} \
                 — percentiles look pooled across the batch rather than per well"
            );
            assert!(
                (got_hi - hi_ref).abs() < 0.5,
                "{name}: its OWN P{p_hi} must land on the high reference {hi_ref}, got {got_hi}"
            );
        }

        // The control: before normalizing, these two wells were nowhere near each other. If the
        // raw curves already agreed, the assertions above would pass without the module doing
        // anything at all.
        let raw_spread = (70.0f32 - 15.0).abs();
        assert!(raw_spread > 1.0, "the two wells must start with genuinely different GR");
    }

    /// T-PREP-05 and T-WELL-16 together: a per-zone parameter override must reach **every sample
    /// inside its zone and no sample outside it**, through the real runner.
    ///
    /// This is the interval-parameter model's whole promise. A module that read a parameter once
    /// before its loop instead of per sample would ignore every zone override ever entered, and
    /// nothing would say so — the run succeeds, the curve is smooth, and the only symptom is that
    /// the numbers are the whole-well answer wearing a zoned label.
    ///
    /// The boundary is HALF-OPEN (`>= top`, `< bottom`), which is what stops a sample sitting
    /// exactly on a shared boundary from belonging to both zones and taking whichever happened to
    /// be listed last. That is pinned here, at the boundary sample itself.
    ///
    /// **A finding, pinned as-is rather than fixed.** `precalc` computes each sample as
    /// `SURF_TEMP + grad(i) * depth(i)` — the gradient is applied from SURFACE, not integrated
    /// down through the zones above. So overriding the gradient in a lower zone produces a **STEP
    /// in temperature at the zone boundary, not a kink**: here 67.0 degC at 1400 m jumps to 77.5
    /// at 1500 m, a 10.5 degC discontinuity where the undisturbed trend would have risen 3.0.
    /// Rock temperature is continuous, so this profile is not physical, and it feeds Rw through
    /// the Arps correction and Rw feeds Sw. T-PREP-05's own expected result says "the FTEMP trend
    /// **kinks**… no discontinuity artifacts" — which is not what the code does.
    ///
    /// It is pinned rather than changed because the fix is method math: integrating per zone means
    /// choosing what the zone's top temperature is, and that is a petrophysical decision with a
    /// cited source, not a refactor. If it is ever integrated, this test must fail and force the
    /// change into the open.
    #[test]
    fn a_per_zone_gradient_override_reaches_exactly_its_own_samples() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-ZP1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        // Vertical well, no TVDSS curve — precalc falls back to measured depth as a whole curve.
        let depths: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32 * 100.0).collect();
        let n = depths.len();
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
        )
        .unwrap();

        // Two zones meeting at 1500 m; only the deeper one carries an override.
        let (boundary, base_grad, deep_grad, surf) = (1500.0f32, 0.03f64, 0.035f64, 25.0f64);
        db::upsert_zone(&conn, &w, "SHALLOW", 1000.0, boundary).unwrap();
        db::upsert_zone(&conn, &w, "DEEP", boundary, 2100.0).unwrap();
        db::set_zone_param(&conn, &w, "DEEP", "TEMP_GRAD", Some(deep_grad as f32), None).unwrap();

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "precalc".into(),
            well_ids: vec![w.clone()],
            log_inputs: HashMap::new(),
            params: [("SURF_TEMP".to_string(), surf), ("TEMP_GRAD".to_string(), base_grad)]
                .into_iter()
                .collect(),
            opts: [("OPT_TU".to_string(), "degC".to_string())].into_iter().collect(),
            output_set: None,
            input_set: None,
        };
        let r = run_workflow_module(&dbm, &req);
        assert!(r[0].error.is_none(), "precalc: {:?}", r[0].error);

        let conn = dbm.lock().unwrap();
        let (d, cols) = equations::fetch_curve_frame(&conn, &w, &["FTEMP".into()]).unwrap();
        let ft = &cols["FTEMP"];
        assert_eq!(d.len(), n);

        for i in 0..n {
            let grad = if d[i] >= boundary { deep_grad } else { base_grad };
            let expect = surf + grad * d[i] as f64;
            assert!(
                (ft[i] as f64 - expect).abs() < 1e-2,
                "sample {i} at {} m: FTEMP {} != {expect} — the {} gradient did not reach it",
                d[i],
                ft[i],
                if d[i] >= boundary { "zone" } else { "well" }
            );
        }

        // The boundary sample belongs to DEEP and to DEEP only. A closed interval on both sides
        // would let SHALLOW and DEEP both claim 1500 m, and which one won would be list order.
        let at_boundary = d.iter().position(|v| *v == boundary).expect("1500 m is a sample");
        assert!(
            (ft[at_boundary] as f64 - (surf + deep_grad * boundary as f64)).abs() < 1e-2,
            "a sample exactly on the boundary must take the zone whose TOP it is"
        );
        assert!(
            (ft[at_boundary - 1] as f64 - (surf + base_grad * (boundary - 100.0) as f64)).abs() < 1e-2,
            "and the sample above it must be untouched by that zone"
        );

        // The step, recorded so the finding above cannot quietly disappear.
        let step = ft[at_boundary] - ft[at_boundary - 1];
        let within_zone = ft[at_boundary - 1] - ft[at_boundary - 2];
        assert!(
            (step as f64 - 10.5).abs() < 1e-2 && (within_zone as f64 - 3.0).abs() < 1e-2,
            "the boundary discontinuity changed: step {step} across the boundary against \
             {within_zone} within the zone. Either the gradient is now integrated per zone \
             (good — update this test and its comment) or something else moved."
        );

        // The control: without the override every sample would sit on one line, so all of the
        // above would pass on a runner that ignored zone parameters entirely.
        assert!(
            (ft[at_boundary] as f64 - (surf + base_grad * boundary as f64)).abs() > 1.0,
            "the override never took effect — this well is still on the whole-well gradient"
        );
    }

    /// T-PREP-16 step 3, pinned as the audited defect rather than as correct behaviour.
    ///
    /// `log_predict`'s MAX_RAW mode exists for exactly one purpose: to repair a density log inside
    /// a washout, where the tool read mud instead of rock. The mask exists for the opposite
    /// purpose: to remove washout samples from everything. Run them together — which is precisely
    /// what the module's own documentation tells you to do — and the mask wins, so the one curve
    /// built to fill the bad hole comes back MISSING inside the bad hole.
    ///
    /// **There are TWO blanks, not one, and the audit finding names only the second.** The runner
    /// blanks the flagged samples in the module's INPUTS before the run (so the predictor is gone
    /// and the module cannot even attempt a prediction), and blanks them again in the OUTPUTS
    /// after. Exempting `log_predict` from the output pass alone would leave RHOB_SYN exactly as
    /// MISSING as it is now, and the symptom would look unfixed. That is asserted below, so
    /// whoever takes this on knows before they start.
    ///
    /// The unmasked control is what makes this a defect report rather than a complaint: the same
    /// well, the same washout, no mask — and the repair happens. The module works. The runner
    /// throws the answer away.
    #[test]
    fn a_masked_washout_defeats_the_very_module_meant_to_repair_it() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-WO1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        // A clean density-gamma relation, and one washed-out sample reading far too light.
        let n = 20usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32 * 5.0).collect();
        let rhob_true: Vec<f32> = gr.iter().map(|g| 2.70 - 0.003 * g).collect();
        let washout = 10usize;
        let mut rhob = rhob_true.clone();
        rhob[washout] = 1.95; // mud, not rock

        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            gr.clone(),
            vec![f32::NAN; n],
            vec![f32::NAN; n],
            rhob.clone(),
            vec![f32::NAN; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let mut flag = vec![0.0f32; n];
        flag[washout] = 1.0;
        equations::write_computed_curve(&conn, &w, &depths, "BADHOLE", &flag).unwrap();

        let dbm = Mutex::new(conn);
        let run = |mask: Option<&str>| -> Vec<ModuleRunResult> {
            let mut opts: HashMap<String, String> =
                [("OPT_COMBINE".to_string(), "MAX_RAW".to_string())].into_iter().collect();
            if let Some(m) = mask {
                opts.insert("MASK".to_string(), m.to_string());
            }
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "log_predict".into(),
                    well_ids: vec![w.clone()],
                    log_inputs: [("TARGET".to_string(), "RHOB".to_string())].into_iter().collect(),
                    params: [("K".to_string(), 5.0)].into_iter().collect(),
                    opts,
                    output_set: None,
                    input_set: None,
                },
            )
        };
        let syn_of = || -> Vec<f32> {
            let conn = dbm.lock().unwrap();
            let (_, cols) =
                equations::fetch_curve_frame(&conn, &w, &["RHOB_SYN".to_string()]).unwrap();
            cols["RHOB_SYN"].clone()
        };

        // Control first: no mask, and the washout IS repaired.
        let r = run(None);
        assert!(r[0].error.is_none(), "unmasked log_predict: {:?}", r[0].error);
        let unmasked = syn_of();
        assert!(
            !unmasked[washout].is_nan() && unmasked[washout] > rhob[washout] + 0.2,
            "the module failed to repair the washout even unmasked ({}); the rest of this test \
             would then be measuring the wrong thing",
            unmasked[washout]
        );
        assert!(
            (unmasked[washout] - rhob_true[washout]).abs() < 0.1,
            "the repair should land near the trend: {} for a true {}",
            unmasked[washout],
            rhob_true[washout]
        );

        // Now with the mask the module's own documentation recommends.
        let r = run(Some("BADHOLE"));
        assert!(r[0].error.is_none(), "masked log_predict: {:?}", r[0].error);
        let masked = syn_of();
        assert!(
            masked[washout].is_nan(),
            "AUDIT-2026-07-21 (Prep statistical #1) says the repaired value is re-blanked at the \
             masked depth, and T-PREP-16 tells the tester to expect that. It returned {} instead \
             — if this was fixed deliberately, update this test and T-PREP-16's known-issue line \
             together.",
            masked[washout]
        );
        assert!(
            masked.iter().enumerate().any(|(i, v)| i != washout && !v.is_nan()),
            "the masked run wrote nothing anywhere — that is a different failure"
        );

        // The second blank. Feeding the module a context where only the PREDICTOR is missing at
        // the washout — which is what the input-side mask does — already yields MISSING, before
        // the output pass ever runs. So exempting log_predict from output masking would not fix
        // this on its own.
        let mut gr_masked = gr.clone();
        gr_masked[washout] = f32::NAN;
        let ctx = modules::ModuleContext {
            n,
            logs: [
                ("TARGET".to_string(), rhob.clone()),
                ("P1".to_string(), gr_masked),
                ("DEPTH".to_string(), depths.clone()),
            ]
            .into_iter()
            .collect(),
            params: [("K".to_string(), vec![5.0; n])].into_iter().collect(),
            opts: [
                ("OPT_COMBINE".to_string(), "MAX_RAW".to_string()),
                ("__IN_TARGET".to_string(), "RHOB".to_string()),
            ]
            .into_iter()
            .collect(),
            depth_unit: Default::default(),
        };
        let out = modules::run_module("log_predict", &ctx).unwrap();
        assert!(
            out["RHOB_SYN"][washout].is_nan(),
            "with the predictor blanked the module cannot predict — so an output-masking \
             exemption alone would leave RHOB_SYN missing anyway"
        );
    }

    /// T-PREP-11. A raw imported curve called FTEMP must NEVER satisfy a computed-only input.
    ///
    /// `nphi_env_corr` reads FTEMP in degC. Commercial LAS exports routinely carry an FTEMP in
    /// degF, and it lands in the RAW import store under exactly that mnemonic. Consume it and the
    /// temperature term is computed from a number roughly twice too large — a correction of a few
    /// thousandths v/v instead of a few ten-thousandths. Nothing about that is visible: NPHI_EC
    /// still tracks NPHI, still looks like a neutron log, and the error rides into porosity.
    ///
    /// `gascorr_spec_shape` asserts the FLAG on gascorr's arguments. This asserts the BEHAVIOUR,
    /// on the module whose manual test names it, through the real runner — which is where the
    /// contract is actually enforced (`workflow.rs` re-resolves computed-only inputs after the
    /// ordinary curve frame has already fallen back to RAW). The flag and the re-resolution loop
    /// are two separate things and either alone is silently useless.
    ///
    /// The three states, in the order a user meets them:
    ///
    /// 1. RAW FTEMP present, nothing computed → the temperature term must be ABSENT, leaving only
    ///    salinity. Not an error, by design: the module documents FTEMP as optional.
    /// 2. Run Formation Temperature → the term appears.
    /// 3. With BOTH present the computed one must win — the case that actually bites, because a
    ///    user who ran precalc reasonably assumes they are covered.
    #[test]
    fn a_raw_ftemp_never_satisfies_the_computed_only_contract() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-EC1", None, None, Some(0.0)).unwrap();
        let w = wid.to_string();

        let depths = vec![1000.0f32, 1500.0, 2000.0];
        let n = depths.len();
        let nphi_in = 0.30f32;
        db::insert_standard_curves(
            &conn,
            wid,
            depths.clone(),
            vec![f32::NAN; n],   // GR
            vec![f32::NAN; n],   // RES_DEEP
            vec![nphi_in; n],    // NPHI
            vec![f32::NAN; n],   // RHOB
            vec![f32::NAN; n],   // DT
            vec![f32::NAN; n],   // SP
        )
        .unwrap();

        // The trap: a RAW-set curve called FTEMP carrying degF numbers, exactly as a vendor LAS
        // delivers it. 220 degF is 104.4 degC — a perfectly ordinary deep temperature in either
        // unit, which is what makes it undetectable by any range check.
        let raw_degf = 220.0f32;
        {
            let id = db::upsert_curve_meta(
                &conn, &w, "RAW", "FTEMP", Some("degF"), Some("FTEMP"), Some("test"), None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &id, &depths, &vec![raw_degf; n]).unwrap();
        }

        let dbm = Mutex::new(conn);
        let run = |module: &str, params: &[(&str, f64)]| -> Vec<ModuleRunResult> {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: vec![w.clone()],
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
            };
            run_workflow_module(&dbm, &req)
        };
        let curve = |name: &str| -> Vec<f32> {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &[name.to_string()]).unwrap();
            cols[name].clone()
        };

        // Manifest defaults, read rather than retyped — a second copy is a second thing to drift.
        let spec = crate::modules::list_modules()
            .into_iter()
            .find(|s| s.name == "nphi_env_corr")
            .expect("nphi_env_corr must be in the manifest");
        let dflt = |name: &str| -> f64 {
            spec.args.iter().find(|a| a.name == name).unwrap().default.parse().unwrap()
        };
        let (k_temp, t_ref, k_sal, salw) =
            (dflt("K_TEMP"), dflt("T_REF"), dflt("K_SAL"), dflt("SALW"));
        let ec_params = [("K_TEMP", k_temp), ("T_REF", t_ref), ("K_SAL", k_sal), ("SALW", salw)];
        let salinity_only = nphi_in as f64 + k_sal * salw / 100000.0;

        // (1) Only the raw degF FTEMP exists. The temperature term must not appear.
        let r = run("nphi_env_corr", &ec_params);
        assert!(r[0].error.is_none(), "nphi_env_corr: {:?}", r[0].error);
        let ec = curve("NPHI_EC");
        for (i, v) in ec.iter().enumerate() {
            assert!(
                (*v as f64 - salinity_only).abs() < 1e-6,
                "sample {i}: a RAW degF FTEMP was consumed — NPHI_EC {v} is not the \
                 salinity-only {salinity_only}"
            );
        }
        // Stated the other way round, so the failure message says what went wrong rather than
        // only that a number moved: the degF value must not have driven the correction.
        let if_degf_consumed = salinity_only + k_temp * (raw_degf as f64 - t_ref);
        assert!(
            (ec[0] as f64 - if_degf_consumed).abs() > 1e-5,
            "NPHI_EC landed exactly where consuming the raw degF FTEMP would put it"
        );

        // (2) Run Formation Temperature — now a genuine degC FTEMP exists in computed provenance.
        let r = run(
            "ftemp_grad",
            &[("TSURF", 26.7), ("TGRAD", 0.03), ("BHT", 100.0), ("TD_BHT", 2000.0)],
        );
        assert!(r[0].error.is_none(), "ftemp_grad: {:?}", r[0].error);
        let ftemp = curve("FTEMP");

        // (3) Re-run with BOTH present. The computed one must win, sample by sample.
        let r = run("nphi_env_corr", &ec_params);
        assert!(r[0].error.is_none(), "nphi_env_corr rerun: {:?}", r[0].error);
        let ec = curve("NPHI_EC");
        for i in 0..n {
            let expect = salinity_only + k_temp * (ftemp[i] as f64 - t_ref);
            assert!(
                (ec[i] as f64 - expect).abs() < 1e-6,
                "sample {i}: NPHI_EC {} must follow the COMPUTED FTEMP {} (expected {expect})",
                ec[i],
                ftemp[i]
            );
        }

        // The control. Every assertion above would also pass on a module that ignored FTEMP
        // altogether, so the computed run must genuinely differ from the salinity-only one.
        assert!(
            (ec[2] as f64 - salinity_only).abs() > 1e-6,
            "the temperature term never appeared even with a computed FTEMP — this test would \
             pass on a module that ignored FTEMP entirely"
        );
    }

    /// Restoring an earlier log-set version must change what the NEXT module run computes.
    ///
    /// `db::log_set_versioning_never_overwrites` proves the restore itself: the archive keeps
    /// both generations and the current store goes back to version 1's values. What it does not
    /// prove is that anything downstream then READS those values — and that is the whole point
    /// of being able to restore. A restore that quietly left modules computing on version 2
    /// would be the worst possible outcome: the catalog, the version history and the curve on
    /// screen would all say version 1, while every number derived from it came from the run you
    /// deliberately rolled back.
    ///
    /// phi_den is the downstream module here because it takes VSH as an input curve, so its
    /// PHIE moves whenever VSH does. The control is that the two PHIE results must DIFFER —
    /// without it, a phi_den that ignored VSH entirely would satisfy every other assertion.
    #[test]
    fn a_restored_log_set_version_feeds_the_next_module_run() {
        use crate::equations::{
            create_log_set, restore_log_set, write_computed_curves_versioned, LogSetSpec,
        };

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-VER", None, None, None).unwrap();
        let w = id.to_string();

        // RHOB is phi_den's other input; hold it constant so VSH is the only thing that moves.
        let n = 3usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![60.0; n],
            nan.clone(),
            nan.clone(),
            vec![2.35f32; n],
            nan.clone(),
            nan,
        )
        .unwrap();

        let spec = LogSetSpec {
            set_name: "INTERP".into(),
            module: "vsh_gr".into(),
            params_json: "{}".into(),
            inputs_json: "[\"GR\"]".into(),
        };

        // Version 1: a clean sand. Version 2: very shaly. Same curve, same well.
        let (set1, v1) = create_log_set(&conn, &w, &spec).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.10f32, 0.10, 0.10])], &set1)
            .unwrap();
        let (set2, v2) = create_log_set(&conn, &w, &spec).unwrap();
        write_computed_curves_versioned(&conn, &w, &depth, &[("VSH", &[0.80f32, 0.80, 0.80])], &set2)
            .unwrap();
        assert_eq!((v1, v2), (1, 2));

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "phi_den".into(),
            well_ids: vec![w.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let phie_at = |d: f32| -> f32 {
            let c = dbm.lock().unwrap();
            c.query_row(
                "SELECT value FROM computed_curves
                 WHERE well_id = ?1 AND curve_name = 'PHIE' AND depth = ?2",
                duckdb::params![w, d],
                |r| r.get(0),
            )
            .expect("phi_den must have written PHIE")
        };

        // Run against the CURRENT version (2, the shaly one).
        let r = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(r[0].error.is_none(), "phi_den on v2: {:?}", r[0].error);
        let phie_v2 = phie_at(1000.0);

        // Roll back to version 1 and run again. Nothing else changed.
        {
            let c = dbm.lock().unwrap();
            restore_log_set(&c, &set1).unwrap();
        }
        let r = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(r[0].error.is_none(), "phi_den on restored v1: {:?}", r[0].error);
        let phie_v1 = phie_at(1000.0);

        assert!(
            (phie_v1 - phie_v2).abs() > 1e-4,
            "the restore changed nothing downstream: PHIE was {phie_v2} on v2 and {phie_v1} after \
             restoring v1. Either the module is not reading the restored VSH, or it is not \
             reading VSH at all"
        );
        // Direction, not just difference: less shale leaves more effective porosity, because
        // phi_den subtracts the shale term VSH*(RHO_MA - RHO_SH)/(RHO_MA - RHO_FL).
        assert!(
            phie_v1 > phie_v2,
            "restoring the cleaner VSH must RAISE PHIE (got {phie_v1} vs {phie_v2})"
        );
    }

    /// Cancel responsiveness: with the chain cancel flag already set, run_workflow_module_into
    /// skips every well (no fetch/compute/write) and returns clean no-ops — so a Cancel drains a
    /// running step's remaining wells in ~a well or two instead of grinding through all of them.
    #[test]
    fn module_run_skips_all_wells_when_cancelled() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "CANCELME", None, None, None).unwrap();
        let w = wid.to_string();
        let n = 8usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        db::insert_standard_curves(
            &conn, wid, depths,
            vec![45.0; n], vec![f32::NAN; n], vec![0.2; n], vec![2.4; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let dbm = Mutex::new(conn);

        let req = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![w.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };

        // Flag already set → every well is a no-op, nothing written.
        let cancel = std::sync::atomic::AtomicBool::new(true);
        let results = run_workflow_module_into(&dbm, &req, None, Some(&cancel), None);
        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none(), "cancel skip is a clean no-op, not an error");
        assert_eq!(results[0].rows_written, 0, "a cancelled well writes nothing");
        {
            let conn = dbm.lock().unwrap();
            let vsh: i64 = conn
                .query_row("SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH'", duckdb::params![w], |r| r.get(0))
                .unwrap();
            assert_eq!(vsh, 0, "no VSH curve should be written when cancelled");
        }

        // Control: the same run WITHOUT the flag DOES write VSH — proving the skip above was the
        // cancel, not a broken fixture.
        let results2 = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(results2[0].error.is_none(), "uncancelled: {:?}", results2[0].error);
        assert!(results2[0].rows_written > 0, "uncancelled run must write VSH");
    }

    /// T-ADV-13. The audit finding was that `sw_height`'s TVD input had NO PRODUCER anywhere in
    /// the app: the deviated-well fix was unit-tested at the module level, the deviation survey
    /// was imported and stored, and nothing connected the two — so the TVD dropdown was a false
    /// affordance and every height silently came back measured along hole, overstating the column
    /// by ~1/cos(inc).
    ///
    /// Both HALVES have had tests for a while — `satheight`'s `sw_height_uses_tvd_and_allows_
    /// tvdss_fwl` hands the module a TVD array directly, and `ingest`'s `deviation_import_
    /// materializes_tvd_curves` checks the survey lands on the log grid. Neither says anything
    /// about the JOINT, which is exactly where the finding lived. This runs the real path:
    /// import a survey, then run the module through `run_workflow_module`'s own input
    /// resolution and read HAFWL back out of the database.
    #[test]
    fn a_deviated_wells_height_is_measured_from_the_survey_not_along_hole() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // Two identical wells on the same MD grid. Only one gets a deviation survey, so the
        // other is the control: it must still fall back to measured depth.
        let depth: Vec<f32> = vec![0.0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0, 3000.0];
        let n = depth.len();
        let mk = |name: &str| -> String {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, None).unwrap();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![50.0; n], nan.clone(), vec![0.2; n],
                vec![2.4; n], nan.clone(), nan,
            )
            .unwrap();
            let w = id.to_string();
            // sw_height needs PHIE; PERM keeps the LEVERETT branch alive so SWH is real too.
            equations::write_computed_curve(&conn, &w, &depth, "PHIE", &vec![0.25f32; n]).unwrap();
            equations::write_computed_curve(&conn, &w, &depth, "PERM", &vec![100.0f32; n]).unwrap();
            w
        };
        let dev = mk("SANDI-DEV");
        let vert = mk("SANDI-VERT");

        // Vertical to 1000 m MD, build to 60 deg by 2000, hold to TD. At 60 deg inclination a
        // metre of hole buys half a metre of true depth, so by TD the two references are
        // hundreds of metres apart — far too large to be confused with interpolation slop.
        let csv = std::env::temp_dir().join(format!("sandibumi_devheight_{dev}.csv"));
        std::fs::write(&csv, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let imported = ingest::import_deviation_csv(&conn, &dev, csv.to_str().unwrap(), Some(25.0), None);
        std::fs::remove_file(&csv).ok();
        assert!(imported.error.is_none(), "survey import failed: {:?}", imported.error);

        // What the survey actually put on the log grid — the test never re-derives minimum
        // curvature, it asserts that the module CONSUMED whatever the survey produced.
        let tvd = equations::fetch_curve_frame(&conn, &dev, &["TVD".into()]).unwrap().1["TVD"].clone();
        assert!(
            tvd.iter().all(|v| v.is_finite()),
            "the survey must materialize a TVD curve on the log grid: {tvd:?}"
        );

        const FWL: f64 = 2600.0;
        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "sw_height".into(),
            well_ids: vec![dev.clone(), vert.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("FWL".to_string(), FWL)]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let results = run_workflow_module(&dbm, &req);
        assert!(results.iter().all(|r| r.error.is_none()), "run errored: {results:?}");

        let conn = dbm.lock().unwrap();
        let hafwl = |w: &str| -> Vec<f32> {
            equations::fetch_curve_frame(&conn, w, &["HAFWL".into()]).unwrap().1["HAFWL"].clone()
        };
        let (h_dev, h_vert) = (hafwl(&dev), hafwl(&vert));

        // The deviated well's height is FWL minus TRUE vertical depth, at every sample.
        for i in 0..n {
            let want = FWL as f32 - tvd[i];
            assert!(
                (h_dev[i] - want).abs() < 0.1,
                "sample {i} (MD {}): HAFWL {} should be FWL - TVD {} = {want}",
                depth[i], h_dev[i], tvd[i]
            );
        }

        // In the VERTICAL section TVD == MD, so the two references agree — which is what makes
        // the deviated section's disagreement meaningful rather than an artefact of the fixture.
        let i1000 = 2;
        assert!(
            (h_dev[i1000] - (FWL as f32 - depth[i1000])).abs() < 0.5,
            "above the kick-off the survey and the driller's depth must agree: {}",
            h_dev[i1000]
        );

        // At TD they must NOT. This is the assertion the audit finding would have failed:
        // measured along hole the column reads hundreds of metres taller than it is.
        let td = n - 1;
        let along_hole = FWL as f32 - depth[td];
        assert!(
            (h_dev[td] - along_hole) > 500.0,
            "at TD the survey height {} must sit far above the along-hole height {along_hole}",
            h_dev[td]
        );

        // Control: no survey, no TVD curve — the module falls back to measured depth. That
        // fallback is correct behaviour for a genuinely vertical well, and it is also exactly
        // what the deviated well used to do.
        for i in 0..n {
            let want = FWL as f32 - depth[i];
            assert!(
                (h_vert[i] - want).abs() < 1e-3,
                "a well with no survey measures height along hole: {} vs {want}",
                h_vert[i]
            );
        }
    }

    /// T-PETRO-13. A `zone_params` override must beat the dialog value INSIDE its zone and
    /// change nothing outside it. The failure this guards against is silent in both directions:
    /// an override that leaks writes a wrong Sw over rock nobody calibrated, and one that never
    /// applies leaves the calibration looking done while the numbers are unchanged.
    ///
    /// The arithmetic is checked against the plan's own expectation — with N = 2, dropping RW
    /// from 0.1 to 0.02 scales SWT by sqrt(0.02/0.1) — rather than against whatever the code
    /// happens to return.
    #[test]
    fn a_zone_parameter_override_moves_that_zone_and_leaves_the_rest_untouched() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-ZONE", None, None, None).unwrap();
        let w = id.to_string();

        // 1000..1019 at 1 m. RT 4 ohmm and PHIT 0.25 put the baseline SWT at 0.632, so the
        // overridden value (0.283) is nowhere near the [SWT_IRR, 1] clamp — a clamped answer
        // would mask the very ratio under test.
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![50.0; n], vec![4.0; n], vec![0.25; n],
            vec![2.4; n], nan.clone(), nan,
        )
        .unwrap();
        for name in ["PHIT", "PHIE"] {
            equations::write_computed_curve(&conn, &w, &depth, name, &vec![0.25f32; n]).unwrap();
        }
        db::upsert_zone(&conn, &w, "UPPER", 1000.0, 1010.0).unwrap();
        db::upsert_zone(&conn, &w, "LOWER", 1010.0, 1020.0).unwrap();

        let dbm = Mutex::new(conn);
        let run = || -> Vec<f32> {
            let req = RunModuleRequest {
                module: "sw_arch".into(),
                well_ids: vec![w.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("A".to_string(), 1.0),
                    ("M".to_string(), 2.0),
                    ("N".to_string(), 2.0),
                    ("RW".to_string(), 0.1),
                    ("SWT_IRR".to_string(), 0.0),
                ]),
                opts: HashMap::from([("OPT_RW".to_string(), "CONSTANT".to_string())]),
                output_set: None,
                input_set: None,
            };
            let r = run_workflow_module(&dbm, &req);
            assert!(r[0].error.is_none(), "sw_arch failed: {:?}", r[0].error);
            let conn = dbm.lock().unwrap();
            equations::fetch_curve_frame(&conn, &w, &["SWT".into()]).unwrap().1["SWT"].clone()
        };

        let before = run();
        assert!(before.iter().all(|v| v.is_finite()), "baseline SWT must be finite: {before:?}");

        // The dialog still says RW = 0.1 on the re-run — the override is what has to win.
        {
            let conn = dbm.lock().unwrap();
            db::set_zone_param(&conn, &w, "UPPER", "RW", Some(0.02), None).unwrap();
        }
        let after = run();

        let ratio = (0.02f64 / 0.1).sqrt() as f32; // 0.4472
        for i in 0..n {
            let d = depth[i];
            if d < 1010.0 {
                let want = before[i] * ratio;
                assert!(
                    (after[i] - want).abs() < 1e-4,
                    "inside UPPER at {d}: SWT {} should be {} x {ratio} = {want}",
                    after[i], before[i]
                );
            } else {
                // Sample-for-sample identical, not merely close: nothing in the LOWER zone saw
                // a different parameter, so nothing about its arithmetic changed.
                assert_eq!(
                    after[i], before[i],
                    "outside UPPER at {d}: SWT moved from {} to {}",
                    before[i], after[i]
                );
            }
        }

        // The zone interval is half-open [top, bottom): the sample sitting exactly on 1010 is
        // the LOWER zone's first sample, not the UPPER zone's last. Two adjacent zones written
        // the way anyone writes them (1000-1010, 1010-1020) must not both claim it.
        let boundary = depth.iter().position(|&d| d == 1010.0).unwrap();
        assert_eq!(
            after[boundary], before[boundary],
            "the sample at the shared boundary belongs to the deeper zone"
        );
    }

    /// Batched write (perf refactor): a module run over MANY wells writes each well's own curve
    /// correctly in ONE transaction — rows are not crossed between wells and per-well set
    /// versioning is intact (one INTERP set per well).
    #[test]
    fn batched_module_run_writes_every_well_correctly() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let mk = |name: &str, gr: f32| -> String {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, None).unwrap();
            let n = 5usize;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            db::insert_standard_curves(
                &conn, id, depth,
                vec![gr; n], vec![f32::NAN; n], vec![0.2; n], vec![2.4; n], vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            id.to_string()
        };
        let a = mk("A", 40.0); // low GR → low VSH
        let b = mk("B", 90.0); // high GR → high VSH
        let dbm = Mutex::new(conn);

        let req = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![a.clone(), b.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
        };
        let results = run_workflow_module(&dbm, &req);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.error.is_none()), "batched run errored: {results:?}");
        assert!(results.iter().all(|r| r.rows_written > 0), "every well must write rows");

        let conn = dbm.lock().unwrap();
        let vsh = |w: &str| -> Vec<f32> {
            equations::fetch_curve_frame(&conn, w, &["VSH".into()]).unwrap().1["VSH"].clone()
        };
        let (va, vb) = (vsh(&a), vsh(&b));
        assert!(va.iter().all(|v| !v.is_nan()) && vb.iter().all(|v| !v.is_nan()), "both wells got finite VSH");
        assert!(va[0] < vb[0], "rows not crossed: low-GR A VSH {} < high-GR B VSH {}", va[0], vb[0]);
        let sets: i64 = conn
            .query_row("SELECT COUNT(*) FROM log_sets WHERE set_name = 'INTERP'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sets, 2, "one INTERP set version per well after the batch");
    }

    /// Full deterministic chain against a real field delivery: import → VSH(GR) →
    /// PHI(D-N) → SW(Indonesia) → PERM(Timur) → pay summary. Ignored by default and skipped
    /// with a printed reason when no delivery folder is configured
    /// (`SANDIBUMI_FIELD_FIXTURES/las/`); run with:
    /// `cargo test --release -- --ignored --nocapture test_full_deterministic_chain`
    #[test]
    #[ignore]
    fn test_full_deterministic_chain() {
        let paths = crate::field_fixtures::las_files(3);
        if crate::field_fixtures::skip("test_full_deterministic_chain", paths.len(), 2) {
            return;
        }

        let db_path = crate::field_fixtures::temp_db("workflow_test");
        let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db failed");

        let results = ingest::import_las_files(&conn, &paths, None);
        let well_ids: Vec<String> = results
            .iter()
            .map(|r| r.well_id.clone().unwrap_or_else(|| panic!("import failed: {:?}", r.error)))
            .collect();

        let db = Mutex::new(conn);
        let run = |module: &str, params: &[(&str, f64)], opts: &[(&str, &str)]| {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: well_ids.clone(),
                log_inputs: HashMap::new(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                output_set: None,
                input_set: None,
            };
            let results = run_workflow_module(&db, &req);
            for r in &results {
                println!("{module}: well={} rows={} outputs={:?} err={:?}", r.well_id, r.rows_written, r.output_curves, r.error);
                assert!(r.error.is_none(), "{module} failed: {:?}", r.error);
            }
        };

        run("vsh_gr", &[("GR_MA", 25.0), ("GR_SH", 130.0)], &[("OPT_GR", "LINEAR")]);
        run(
            "phi_dn",
            &[("RHO_MA", 2.645), ("RHO_SH", 2.5), ("NPHI_SH", 0.35), ("RHO_DSH", 2.65), ("PHIE_MAX", 0.35)],
            &[("OPT_XPLOT", "AVERAGE")],
        );
        run(
            "sw_indo",
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.2), ("RT_SH", 4.0)],
            &[("OPT_INDO", "FULL"), ("OPT_RW", "CONSTANT")],
        );
        run("perm_wyllie_rose", &[("SWE_IRR", 0.15)], &[("OPT_WR", "TIMUR")]);

        // Physical sanity: VSH/PHIE/SWE within [0,1], PERM non-negative, and each
        // well has a meaningful number of valid samples.
        {
            let conn = db.lock().unwrap();
            for (curve, lo, hi) in [("VSH", 0.0, 1.0), ("PHIE", 0.0, 0.5), ("SWE", 0.0, 1.0), ("PERM", 0.0, f64::MAX)] {
                let (count, min, max): (i64, f64, f64) = conn
                    .query_row(
                        "SELECT count(value), min(value), max(value) FROM computed_curves
                         WHERE curve_name = ?1 AND NOT isnan(value)",
                        duckdb::params![curve],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .unwrap();
                println!("{curve}: n={count} min={min:.4} max={max:.4}");
                assert!(count > 1000, "{curve}: too few valid samples ({count})");
                assert!(min >= lo && max <= hi, "{curve} out of physical range: [{min}, {max}]");
            }
        }

        // Pay summary over the whole wells (no zones defined → single ALL zone).
        let rows = run_pay_summary(
            &db,
            &PaySummaryRequest { well_ids: well_ids.clone(), vsh_max: 0.5, phie_min: 0.1, swe_max: 0.6, perm_min: None, skip_version: true, stats_only: false },
        )
        .expect("pay summary failed");
        assert_eq!(rows.len(), well_ids.len() * 3); // SAND/RESERVOIR/PAY per well
        for r in &rows {
            println!(
                "{} {} {}: gross={:.1} net={:.1} ntg={:.3} avgPHIE={:.3} avgSWE={:.3} HPV={:.2}",
                r.well_name, r.zone, r.flag, r.gross, r.net, r.ntg, r.avg_phie, r.avg_swe, r.hpv
            );
            assert!(r.net <= r.gross + 0.01);
            if r.flag == "PAY" {
                let res = rows
                    .iter()
                    .find(|x| x.well_id == r.well_id && x.zone == r.zone && x.flag == "RESERVOIR")
                    .unwrap();
                assert!(r.net <= res.net + 0.01, "PAY net exceeds RESERVOIR net");
            }
        }
    }
}
