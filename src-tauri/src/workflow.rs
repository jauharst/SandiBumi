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
    pub input_set: Option<String>
,
    /// Explicit operator and source/reference note. The operator is entered once per frontend
    /// session and attached to every run; it is never inferred from the Windows account.
    pub custody: equations::RunCustody,
}

#[cfg(test)]
pub(crate) fn test_run_custody() -> equations::RunCustody {
    equations::RunCustody {
        actor: equations::AncestryActor {
            kind: equations::AncestryActorKind::Human,
            identity: "automated-test-fixture".to_string(),
        },
        source_note: "test fixture values declared in the owning test".to_string(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleRunResult {
    pub well_id: String,
    pub rows_written: usize,
    pub output_curves: Vec<String>,
    pub error: Option<String>,
}

/// Whether a deterministic module's result changes when the physical unit of its
/// depth frame changes. This is deliberately exhaustive rather than inferred from
/// argument spelling: `phimax`, for example, can fall back from TVDSS to DEPTH, and
/// `condflag` uses depth only when applying thickness and shoulder contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DepthUnitDependency {
    Declared,
    Independent,
}

/// Machine-readable depth-unit inventory for every deterministic module manifest.
///
/// The explicit independent arm is load-bearing. A wildcard that called an unknown
/// module independent would let the next depth-dependent module silently inherit the
/// legacy metres fallback. The inventory test below therefore checks this function
/// against the live manifest registry.
pub(crate) fn module_depth_unit_dependency(module: &str) -> Result<DepthUnitDependency, String> {
    match module {
        "phimax" | "ftemp_grad" | "precalc" | "condflag" | "depth_shift" | "splice"
        | "despike" | "smooth" | "fill_gaps" | "block" | "bed_detect" | "sw_height" => {
            Ok(DepthUnitDependency::Declared)
        }
        "vsh_gr" | "vsh_dn" | "phi_den" | "phi_dn" | "phi_son" | "ssc" | "sspw"
        | "badhole" | "nphimat" | "gascorr" | "gr_hole_corr" | "nphi_env_corr"
        | "rhob_hole_corr" | "gr_normalize" | "log_predict" | "sw_arch" | "sw_indo"
        | "sw_sim" | "sw_rtc" | "sw_imts" | "perm_wyllie_rose" | "perm_coates"
        | "perm_transform" | "thin_bed_ts" | "clip" | "flip" | "normalize" | "multimin"
        | "midplot" | "rocktyping" | "lucia_rfn" | "pittman_rx" | "rt_cutoff"
        | "electrofacies" | "gmm_facies" | "toc_passey" | "kerogen" | "gip"
        | "brittleness" => Ok(DepthUnitDependency::Independent),
        other => Err(format!(
            "module '{other}' has no depth-unit dependency classification; classify it before it can run"
        )),
    }
}

/// Resolve the typed unit a module run receives. Independent modules may run in a
/// legacy project whose unit is absent because their result cannot consume this
/// placeholder; dependent modules must stop before any input fetch or write.
pub(crate) fn resolve_module_depth_unit(
    conn: &Connection,
    module: &str,
) -> Result<crate::units::DepthUnit, String> {
    let dependency = module_depth_unit_dependency(module)?;
    match crate::units::project_depth_unit(conn)
        .map_err(|error| format!("cannot read the project's declared depth unit: {error}"))?
    {
        Some(unit) => Ok(unit),
        None if dependency == DepthUnitDependency::Declared => {
            Err(format!("{module} requires a declared project depth unit before it can run"))
        }
        None => Ok(crate::units::DepthUnit::Metres),
    }
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
    let mut zoned: Vec<String> = Vec::new();
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

        // A well-scoped parameter refuses a NAMED zone override and accepts the well-wide one.
        // The distinction is the whole rule: `*` gives the well one value, which is what a
        // geothermal trend has, while a named zone gives it a different value part way down —
        // and since the trend is evaluated from surface at every sample, that is a STEP at the
        // formation top rather than a bend (see `ArgSpec::well_scope`). Only an override that
        // would actually apply is refused; one naming a zone this well does not have is inert
        // today and must not start failing runs.
        if arg.well_scope {
            for zp in zone_params.iter().filter(|z| z.param_name == arg.name && z.zone_name != "*") {
                if zp.value_num.is_some() && zone_range.contains_key(zp.zone_name.as_str()) {
                    zoned.push(format!("{} in zone '{}'", arg.name, zp.zone_name));
                }
            }
        }

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
    // Refused by name with the fix, rather than ignored. Silently dropping the override would
    // change the well's temperature — and so its Rw, and so its Sw — with nothing on the log to
    // say why, which is the failure this whole rule exists to prevent.
    if !zoned.is_empty() {
        return Err(format!(
            "these parameters describe one trend for the whole well and cannot be set per zone: \
             {}. The trend is computed from surface at every sample, so a value that changes part \
             way down makes the curve JUMP at that formation top instead of bending — and \
             formation temperature reaches Sw through Rw. Set it once for the well (the '*' scope \
             in the per-well parameter grid, which is still honoured) or clear the zone override.",
            zoned.join("; ")
        ));
    }

    // A synthetic per-sample array naming which zone each sample falls in — ordinal by depth
    // order, NaN outside every zone. Not a Param anyone can set: it rides the same channel
    // because that channel already carries "one value per sample, resolved from this well's
    // zones", which is exactly what this is.
    //
    // `frame::block` needs it to upscale marker-to-marker: a module has no database handle, so
    // without this the only bed definitions expressible would be a fixed interval and a class
    // curve, and "one value per zone" is the coarsening a zone-parameter table or a volumetrics
    // summary actually consumes. Zones are sorted by top depth so the ordinal is monotone with
    // depth — a block index that jumped around would make consecutive beds non-adjacent.
    let mut ordered: Vec<_> = zones.iter().collect();
    ordered.sort_by(|a, b| a.top_depth.partial_cmp(&b.top_depth).unwrap_or(std::cmp::Ordering::Equal));
    let mut zone_index = vec![f64::NAN; depth.len()];
    for (ord, z) in ordered.iter().enumerate() {
        for (i, d) in depth.iter().enumerate() {
            if *d >= z.top_depth && *d < z.bottom_depth {
                zone_index[i] = ord as f64;
            }
        }
    }
    out.insert(ZONE_INDEX_ARG.to_string(), zone_index);
    Ok(out)
}

/// Name of the synthetic per-sample zone-ordinal array (see [`resolve_param_arrays`]). Prefixed
/// like the `__IN_<arg>` mnemonics so it cannot collide with a manifest parameter.
pub(crate) const ZONE_INDEX_ARG: &str = "__ZONE_INDEX";

/// Run option carrying a prefix applied to every output curve name.
///
/// Universal like `MASK` rather than a per-module manifest arg: it is one rule about what a run
/// writes. **Monte Carlo REFUSES a step that sets it** — its plan builder resolves cutoffs and
/// fraction curves from the manifest's declared LogOut names, so a prefixed run would be planned
/// against names it never writes, and the study would come back with plausible percentiles
/// computed from nothing. Refusing by name beats a silently empty answer.
pub(crate) const OUT_PREFIX_OPT: &str = "OUT_PREFIX";

/// Run-option prefix carrying a per-output rename: `__OUT_VSH = VSHALE` makes the run write its
/// `VSH` output as `VSHALE`.
///
/// Double-underscored like `__IN_<arg>` and `__ZONE_INDEX` because it is framework-reserved and
/// can never collide with a manifest option. An absent or blank entry means "the manifest's own
/// default name", which is what every existing run and every saved chain sends — so this is
/// additive by construction.
pub(crate) const OUT_NAME_PREFIX: &str = "__OUT_";

/// The names a run will actually write, one per declared output, in declaration order.
///
/// This is the ONE place a module's output name is decided, and it exists because five modules
/// used to build their own (`log_predict` returned `<target>_SYN`, `phi_cap` returned
/// `<input>_CAP`, Condition returned whatever its `OUT` text field said). Three costs followed
/// from that, and all three are what this closes:
///
/// * The manifest's declared LogOut described a curve the run did not write, so a dialog reading
///   "Outputs: SYN" was telling the user something untrue.
/// * There was no way to offer a rename without re-implementing each module's naming rule in the
///   caller — a second copy that would drift, the standing `composite.rs`-versus-renderer warning.
/// * Nothing checked the name before it was written. `condition.rs` and `frame.rs` each carried
///   their own copy of the shadowing refusal; the other forty modules had none, so a rename could
///   have put `VSH` on `GR` and produced a curve nothing can read.
///
/// Jauhar, 2026-08-05: *"naming each output curve in bulk when modules gonna run"* — the grid in
/// the module pane is a row per entry returned here.
/// The names a run actually WRITES for its class outputs (`SB-MLA-055`).
///
/// Walks `modules::class_outputs` through the same two transforms the write path applies — the
/// per-output rename from `resolve_output_names`, then the universal prefix — so a FACIES the user
/// renamed to LITHO under prefix `TEST_` is declared as `TEST_LITHO`. Deriving it here rather than
/// threading a flag through `ModuleOutputs` keeps the change off every module's return type; the
/// cost is that this must apply the SAME two transforms, which is why it reads them from the same
/// two places rather than restating either.
fn class_output_names(
    module: &str,
    out_names: &[(String, String)],
    opts: &HashMap<String, String>,
) -> Vec<String> {
    let prefix = opts
        .get(OUT_PREFIX_OPT)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_default();
    modules::class_outputs(module)
        .iter()
        .filter_map(|key| out_names.iter().find(|(declared, _)| declared == key).map(|(_, n)| n.clone()))
        .map(|n| format!("{prefix}{n}"))
        .collect()
}

pub(crate) fn resolve_output_names(
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
) -> Result<Vec<(String, String)>, String> {
    let mut resolved: Vec<(String, String)> = Vec::new();
    for arg in spec.args.iter().filter(|a| a.kind == ArgKind::LogOut) {
        // A rename wins over the manifest default; a BLANK rename means "the default", the same
        // reading a blank Text arg gets, so clearing the box returns the original name rather
        // than writing an unnamed curve.
        let typed = opts.get(&format!("{OUT_NAME_PREFIX}{}", arg.name)).map(|s| s.trim()).unwrap_or("");
        let name = if !typed.is_empty() {
            typed.to_uppercase()
        } else if arg.default.is_empty() {
            arg.name.clone()
        } else {
            expand_out_pattern(&arg.default, spec, opts, &resolved).unwrap_or_else(|| arg.name.clone())
        };

        // Whitespace and quotes would survive the write and then break every reader that parses a
        // curve list — refused here, where the user typed it, rather than in a LAS export weeks on.
        if name.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == ',') {
            return Err(format!(
                "Output name '{name}' for {} contains a space or quote. A curve name is used \
                 verbatim in exports and curve lists — use letters, digits and underscores.",
                arg.name
            ));
        }
        if name == "DEPTH" {
            return Err(format!(
                "{} = DEPTH is refused: DEPTH is the reference column of the existing STANDARD \
                 frame. A module must never write back to that frame's reference column; use \
                 Reframe to emit a different depth basis as a new OWN frame.",
                arg.name
            ));
        }
        if crate::schema_vocab::standard_column(&name).is_some() {
            return Err(format!(
                "{} = {name} would be shadowed: {name} is read from the raw log first, so a \
                 computed copy stored under that name is never the one anything reads. Give the \
                 output its own name.",
                arg.name
            ));
        }
        if let Some((other, _)) = resolved.iter().find(|(_, n)| *n == name) {
            return Err(format!(
                "{} and {other} would both be written as {name}. Two outputs under one name means \
                 one of them silently replaces the other — rename one.",
                arg.name
            ));
        }
        resolved.push((arg.name.clone(), name));
    }
    Ok(resolved)
}

/// The run options a module actually sees: manifest defaults, the caller's overrides on top, and
/// each input's resolved mnemonic as `__IN_<arg>`.
///
/// Shared by the runner and by [`preview_output_names`] so a dialog is shown the names the run
/// will really write. Two copies of this assembly would drift in exactly the way that matters —
/// the preview would agree with the run until an argument was added, and then quietly stop.
pub(crate) fn build_opts(
    spec: &modules::ModuleSpec,
    overrides: &HashMap<String, String>,
    log_inputs: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut opts: HashMap<String, String> = spec
        .args
        .iter()
        .filter(|a| a.kind == ArgKind::Option || a.kind == ArgKind::Text)
        .map(|a| (a.name.clone(), a.default.clone()))
        .collect();
    for (k, v) in overrides {
        let value = if spec.args.iter().any(|arg| arg.name == *k && arg.kind == ArgKind::Option) {
            modules::canonical_option_value(&spec.name, k, v)
        } else {
            v.clone()
        };
        opts.insert(k.clone(), value);
    }
    for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
        let mnemonic = log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
        opts.insert(format!("__IN_{}", a.name), mnemonic.trim().to_uppercase());
    }
    opts
}

/// Build the effective configurable-parameter record from one module manifest. The runner has
/// already resolved options with [`build_opts`]; this function records the same values and says
/// whether each came from the request or the manifest. It never supplies a missing numeric
/// default: an uncited/ABSENT manifest entry remains REQUIRED_UNSET.
pub(crate) fn effective_module_parameters(
    spec: &modules::ModuleSpec,
    explicit_params: &HashMap<String, f64>,
    explicit_opts: &HashMap<String, String>,
    effective_opts: &HashMap<String, String>,
    source_note: &str,
    name_prefix: &str,
) -> Result<
    (
        Vec<equations::AncestryParameter>,
        serde_json::Map<String, serde_json::Value>,
    ),
    String,
> {
    let manifest_version = crate::parameter_pack::module_parameter_schema_from_spec(spec)?
        .module_schema_version;
    let manifest_source = format!("module manifest {manifest_version}");
    let mut parameters = Vec::new();
    let mut legacy = serde_json::Map::new();

    for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Param) {
        let (value, source, resolution, value_manifest_version) =
            if let Some(value) = explicit_params.get(&arg.name) {
                (
                    serde_json::json!(value),
                    source_note.to_string(),
                    Some(equations::ParameterResolution::Explicit),
                    None,
                )
            } else if let Ok(value) = arg.default.parse::<f64>() {
                (
                    serde_json::json!(value),
                    arg.default_source.clone(),
                    Some(equations::ParameterResolution::Defaulted),
                    Some(manifest_version.clone()),
                )
            } else {
                (
                    serde_json::json!(modules::ABSENT_DEFAULT_SOURCE),
                    modules::ABSENT_DEFAULT_SOURCE.to_string(),
                    None,
                    None,
                )
            };
        legacy.insert(arg.name.clone(), value.clone());
        let decision = crate::param_sources::decision_for(&arg.sources_topic, &value);
        parameters.push(equations::AncestryParameter {
            name: format!("{name_prefix}{}", arg.name),
            value,
            source,
            resolution,
            manifest_version: value_manifest_version,
            decision,
        });
    }

    for arg in spec
        .args
        .iter()
        .filter(|arg| arg.kind == ArgKind::Option || arg.kind == ArgKind::Text)
    {
        if let Some(value) = effective_opts.get(&arg.name) {
            let explicit = explicit_opts.contains_key(&arg.name);
            legacy.insert(arg.name.clone(), serde_json::json!(value));
            parameters.push(equations::AncestryParameter {
                name: format!("{name_prefix}{}", arg.name),
                value: serde_json::json!(value),
                source: if explicit {
                    source_note.to_string()
                } else {
                    manifest_source.clone()
                },
                resolution: Some(if explicit {
                    equations::ParameterResolution::Explicit
                } else {
                    equations::ParameterResolution::Defaulted
                }),
                manifest_version: (!explicit).then(|| manifest_version.clone()),
                decision: None,
            });
        }
    }

    Ok((parameters, legacy))
}

/// Test-only view of the legacy flat parameter payload. Existing module tests pin the
/// stable saturation method identifiers through this view; production persistence uses
/// `complete_module_log_spec`, which adds a source to every recorded value.
#[cfg(test)]
pub(crate) fn recorded_module_params(
    req: &RunModuleRequest,
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
) -> String {
    let mut recorded = serde_json::Map::new();
    for (name, value) in &req.params {
        recorded.insert(name.clone(), serde_json::json!(value));
    }
    let method_id = match req.module.as_str() {
        "sw_arch" => Some("archie_total"),
        "sw_indo" => Some("indonesia"),
        "sw_sim" => opts.get("OPT_SIM").map(String::as_str),
        _ => None,
    };
    if let Some(id) = method_id {
        for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Option) {
            if let Some(value) = opts.get(&arg.name) {
                recorded.insert(arg.name.clone(), serde_json::json!(value));
            }
        }
        recorded.insert("method_id".into(), serde_json::json!(id));
    }
    serde_json::Value::Object(recorded).to_string()
}

fn complete_module_log_spec(
    conn: &Connection,
    well_id: &str,
    req: &RunModuleRequest,
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
    log_args: &[(String, String)],
    output_names: &[String],
    parameter_serializer: &impl Fn(
        &serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String>,
) -> Result<equations::CompleteLogSetSpec, String> {
    req.custody.validate()?;

    let zone_params = db::list_zone_params(conn, well_id).map_err(|error| error.to_string())?;
    let (mut parameters, mut legacy) = effective_module_parameters(
        spec,
        &req.params,
        &req.opts,
        opts,
        req.custody.source_note.trim(),
        "",
    )?;
    for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Param) {
        for zone_value in zone_params
            .iter()
            .filter(|entry| entry.param_name == arg.name)
        {
            let Some(value) = zone_value.value_num else {
                continue;
            };
            let source = zone_value
                .value_text
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or(req.custody.source_note.trim());
            parameters.push(equations::AncestryParameter {
                name: format!("{}@{}", arg.name, zone_value.zone_name),
                value: serde_json::json!(value),
                source: source.to_string(),
                resolution: Some(equations::ParameterResolution::Explicit),
                manifest_version: None,
                decision: crate::param_sources::decision_for(
                    &arg.sources_topic,
                    &serde_json::json!(value),
                ),
            });
        }
    }
    // Saturation outputs retain the stable equation identity in addition to the
    // selected option. This is deliberately explicit: a downstream reviewer must
    // not need to decode a vendor adjective to know which equation produced SWE.
    let method_id = match req.module.as_str() {
        "sw_arch" => Some("archie_total"),
        "sw_indo" => Some("indonesia"),
        "sw_sim" => opts.get("OPT_SIM").map(String::as_str),
        _ => None,
    };
    if let Some(method_id) = method_id {
        legacy.insert("method_id".into(), serde_json::json!(method_id));
        parameters.push(equations::AncestryParameter {
            name: "method_id".into(),
            value: serde_json::json!(method_id),
            source: req.custody.source_note.clone(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });
    }

    let mut inputs = Vec::new();
    let mut missing = HashMap::new();
    for (argument, curve) in log_args
        .iter()
        .filter(|(_, curve)| !curve.trim().is_empty())
    {
        match equations::resolve_ancestry_input(
            conn,
            well_id,
            argument,
            curve,
            req.input_set.as_deref(),
            None,
        ) {
            Ok(input) => inputs.push(input),
            Err(error) => {
                missing.insert(argument.as_str(), error);
            }
        }
    }
    for arg in spec
        .args
        .iter()
        .filter(|arg| arg.kind == ArgKind::LogIn && arg.required)
    {
        let present = inputs.iter().any(|input| input.argument == arg.name)
            || arg
                .required_any_of
                .iter()
                .any(|alternate| inputs.iter().any(|input| input.argument == *alternate));
        if !present {
            let detail = missing
                .get(arg.name.as_str())
                .cloned()
                .unwrap_or_else(|| format!("required input '{}' was not selected", arg.name));
            return Err(detail);
        }
    }

    let zones = db::list_zones(conn, well_id).map_err(|error| error.to_string())?;
    let zone_scope = if zones.is_empty() {
        equations::AncestryZoneScope::WholeWell
    } else {
        equations::AncestryZoneScope::Defined(
            zones
                .into_iter()
                .map(|zone| equations::AncestryZone {
                    name: zone.zone_name,
                    top: zone.top_depth,
                    base: zone.bottom_depth,
                    source: req.custody.source_note.clone(),
                })
                .collect(),
        )
    };
    let outputs = output_names
        .iter()
        .map(|curve| equations::AncestryOutput {
            curve: curve.clone(),
            derivation: format!("{}:{curve}", req.module),
        })
        .collect();
    let parameter_state = equations::parameter_state_for(&parameters);
    let ancestry = equations::CurveAncestry {
        schema_version: equations::CURVE_ANCESTRY_SCHEMA_VERSION,
        module: req.module.clone(),
        module_version: env!("CARGO_PKG_VERSION").into(),
        inputs,
        parameters,
        parameter_state,
        zone_scope,
        actor: req.custody.actor.clone(),
        timestamp_utc_ms: equations::ancestry_timestamp_utc_ms()?,
        outputs,
    };
    let legacy = parameter_serializer(&legacy)
        .map_err(|error| format!("cannot serialize module parameters: {error}"))?;
    equations::CompleteLogSetSpec::try_new_with_legacy(
        req.output_set
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("INTERP"),
        ancestry,
        legacy,
        &serde_json::to_string(log_args).map_err(|error| error.to_string())?,
    )
}

/// One declared output and the curve name a run with these settings would write it under.
#[derive(Debug, Clone, Serialize)]
pub struct OutputName {
    pub arg: String,
    pub desc: String,
    pub unit: String,
    pub name: String,
}

/// What the module pane's output grid is filled from: the names this module would write, given the
/// inputs and renames chosen so far.
///
/// The dialog asks rather than working it out, because working it out means expanding
/// [`modules::log_out_as`] patterns — and a second expansion in TypeScript would be a copy of a
/// naming rule that has already been wrong once. The same question the runner asks, answered by
/// the same code.
pub fn preview_output_names(
    module: &str,
    log_inputs: &HashMap<String, String>,
    overrides: &HashMap<String, String>,
) -> Result<Vec<OutputName>, String> {
    let spec = modules::list_modules()
        .into_iter()
        .find(|m| m.name == module)
        .ok_or_else(|| format!("unknown module '{module}'"))?;
    let opts = build_opts(&spec, overrides, log_inputs);
    let resolved = resolve_output_names(&spec, &opts)?;
    Ok(resolved
        .into_iter()
        .map(|(arg, name)| {
            let a = spec.args.iter().find(|a| a.name == arg);
            OutputName {
                desc: a.map(|a| a.desc.clone()).unwrap_or_default(),
                unit: a.map(|a| a.unit.clone()).unwrap_or_default(),
                arg,
                name,
            }
        })
        .collect())
}

/// Expands `{ARG}` placeholders in a [`modules::log_out_as`] pattern.
///
/// A token names another arg of the same module: a LogIn expands to the mnemonic this run chose
/// for it (already in `opts` as `__IN_<arg>`), an earlier LogOut to the name it resolved to,
/// anything else to its option or text value. Returns `None` when a token is unknown or expands
/// to nothing — an optional input the user cleared — and the caller then falls back to the
/// declared name, which is exactly what `log_predict` did by hand.
fn expand_out_pattern(
    pattern: &str,
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
    resolved: &[(String, String)],
) -> Option<String> {
    let mut out = String::with_capacity(pattern.len());
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}')? + open;
        let token = &rest[open + 1..close];
        let value = if let Some((_, name)) = resolved.iter().find(|(declared, _)| declared == token) {
            name.clone()
        } else if spec.args.iter().any(|a| a.name == token && a.kind == ArgKind::LogIn) {
            opts.get(&format!("__IN_{token}"))?.trim().to_uppercase()
        } else {
            opts.get(token)?.trim().to_uppercase()
        };
        if value.is_empty() {
            return None;
        }
        out.push_str(&value);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Some(out)
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
    preset_sets: Option<&HashMap<String, equations::CompleteSetId>>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<ModuleRunResult> {
    run_workflow_module_into_with_parameter_serializer(
        db,
        req,
        preset_sets,
        cancel,
        progress,
        &|parameters| serde_json::to_value(parameters).map_err(|error| error.to_string()),
    )
}

fn run_workflow_module_into_with_parameter_serializer(
    db: &Mutex<Connection>,
    req: &RunModuleRequest,
    preset_sets: Option<&HashMap<String, equations::CompleteSetId>>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    progress: Option<&crate::jobs::JobHandle>,
    parameter_serializer: &impl Fn(
        &serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String>,
) -> Vec<ModuleRunResult> {
    if let Err(error) = req.custody.validate() {
        return req
            .well_ids
            .iter()
            .map(|well_id| ModuleRunResult {
                well_id: well_id.clone(),
                rows_written: 0,
                output_curves: vec![],
                error: Some(error.clone()),
            })
            .collect();
    }
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

    // Options: dialog values over manifest defaults, plus each input's resolved mnemonic as
    // `__IN_<arg>` (which is how a `log_out_as` pattern names its curve after its input). Text
    // args ride the same channel as Options — they are strings chosen per run, and the only
    // difference is that their valid values are not a list the manifest could hold.
    let opts = build_opts(&spec, &req.opts, &req.log_inputs);

    // The project's depth unit, read ONCE here rather than per well: it is a project-level
    // fact, and the wells below run under rayon where each lock acquisition would contend.
    // The declared CLASS curves (SB-MLA-055) are read in the same lock and for the same reason —
    // per WELL, because a facies curve exists on the wells that were clustered and not on others.
    let (depth_unit, class_by_well) = {
        let conn = db.lock().unwrap();
        let unit = match resolve_module_depth_unit(&conn, &req.module) {
            Ok(unit) => unit,
            Err(error) => {
                return req
                    .well_ids
                    .iter()
                    .map(|well_id| ModuleRunResult {
                        well_id: well_id.clone(),
                        rows_written: 0,
                        output_curves: vec![],
                        error: Some(error.clone()),
                    })
                    .collect();
            }
        };
        let map: HashMap<String, String> = req
            .well_ids
            .iter()
            .filter_map(|w| {
                let set = crate::db::class_curves_for_well(&conn, w).ok()?;
                if set.is_empty() {
                    return None;
                }
                let mut v: Vec<String> = set.into_iter().collect();
                v.sort(); // deterministic, so the same run builds the same opts string
                Some((w.clone(), v.join(",")))
            })
            .collect();
        (unit, map)
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
    // The names this run will write, decided ONCE. Every input to the decision — the manifest, the
    // chosen mnemonics, the renames — is well-independent, so a bad name is refused here as one
    // message rather than as N identical per-well failures in the Processing panel.
    let out_names = match resolve_output_names(&spec, &opts) {
        Ok(n) => n,
        Err(e) => {
            return req
                .well_ids
                .iter()
                .map(|w| ModuleRunResult {
                    well_id: w.clone(),
                    rows_written: 0,
                    output_curves: vec![],
                    error: Some(e.clone()),
                })
                .collect()
        }
    };

    // Phase 1 outcome per well. Outputs are held in memory so Phase 2 can write EVERY well in
    // one batched transaction (vs a fsync-bound delete+append transaction per well — the
    // dominant field-scale write cost). Nothing is written to computed_curves during Phase 1.
    enum Outcome {
        Skipped,
        Failed(String),
        Computed { depth: Vec<f32>, outputs: HashMap<String, Vec<f32>> },
    }

    /// Did the run answer ANYWHERE? An output map that is present but entirely MISSING is a run
    /// that could not answer, not an interpretation.
    ///
    /// One helper because this decides four things that must agree: the Processing panel's item
    /// state, whether a log-set version is allocated, whether anything is WRITTEN, and what the
    /// result reports. Phase 2 used to write for any well whose outcome was `Computed` with a
    /// non-empty output map — and an all-MISSING map is still non-empty — so rocktyping on a well
    /// with porosity but no permeability reported its failure AND versioned the whole family
    /// (RQI, PHIZ, FZI, R35, PGEOM, PSTRUC, RT, PERM_RT) into the Curve Catalog as curves blank
    /// from top to bottom (`docs/review_triage.md` finding 10).
    ///
    /// The rule is not "drop blank curves" — it is **a run that reports failure must not also
    /// version an interpretation**. A single all-MISSING output ALONGSIDE finite ones is kept, and
    /// deliberately: a flag curve nothing triggered is a real answer, and dropping one output of a
    /// run would leave the written set inconsistent with the one the module declares.
    fn answered(outputs: &HashMap<String, Vec<f32>>) -> bool {
        outputs.values().any(|v| v.iter().any(|x| x.is_finite()))
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

                // Per-well opts: everything the run decided, plus THIS well's declared class
                // curves. Set only where the well has any, so a project that has never run a
                // clustering carries no extra key and behaves exactly as before.
                let mut well_opts = opts.clone();
                if let Some(cls) = class_by_well.get(well_id) {
                    well_opts.insert(modules::CLASS_CURVES_OPT.to_string(), cls.clone());
                }
                let ctx = ModuleContext { n: depth.len(), logs, params, opts: well_opts, depth_unit };
                let mut outputs = modules::run_module(&req.module, &ctx)?;

                // Declared key → the name this run writes (`resolve_output_names`). A module
                // returns the key its manifest declares and never builds a name of its own, so
                // this is a plain lookup — and an emitted key that no declared output claims is a
                // module bug, left untouched here and caught by
                // `every_module_returns_the_output_keys_its_manifest_declares`.
                outputs = outputs
                    .into_iter()
                    .map(|(key, values)| {
                        let name = out_names
                            .iter()
                            .find(|(declared, _)| *declared == key)
                            .map(|(_, n)| n.clone())
                            .unwrap_or(key);
                        (name, values)
                    })
                    .collect();

                // Universal output prefix — the bulk form of the same freedom (Jauhar,
                // 2026-08-05: *"each tools or modules should give user freedom to define input
                // and output log set ... and their own curves"*), applied AFTER the renames so
                // the two compose: VSH renamed to VSHALE under prefix TEST_ is TEST_VSHALE.
                //
                // Handled HERE rather than per module, for the reason MASK is: it is one rule
                // about what a run writes, and forty copies of it would be forty places to get it
                // wrong.
                //
                // Empty means unchanged, which is what every existing run and every saved chain
                // sends — so this is additive by construction.
                if let Some(prefix) = opts.get(OUT_PREFIX_OPT).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    let prefix = prefix.to_uppercase();
                    outputs = outputs
                        .into_iter()
                        .map(|(name, values)| (format!("{prefix}{name}"), values))
                        .collect();
                }

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
                    Outcome::Computed { outputs, .. } if answered(outputs) => {
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
            Outcome::Computed { outputs, .. } if answered(outputs) => Some(w.clone()),
            _ => None,
        })
        .collect();

    let mut set_err: Option<String> = None;
    let set_ids: HashMap<String, equations::CompleteSetId> = if succ_ids.is_empty() {
        HashMap::new()
    } else if let Some(preset) = preset_sets {
        succ_ids.iter().filter_map(|w| preset.get(w).map(|s| (w.clone(), s.clone()))).collect()
    } else {
        let conn = db.lock().unwrap();
        let mut complete = Vec::with_capacity(succ_ids.len());
        let mut build_error = None;
        for (well_id, outcome) in req.well_ids.iter().zip(outcomes.iter()) {
            let Outcome::Computed { outputs, .. } = outcome else {
                continue;
            };
            if !answered(outputs) {
                continue;
            }
            let mut names: Vec<String> = outputs.keys().cloned().collect();
            names.sort();
            match complete_module_log_spec(
                &conn,
                well_id,
                req,
                &spec,
                &opts,
                &log_args,
                &names,
                parameter_serializer,
            ) {
                Ok(spec) => complete.push(equations::CompleteWellLogSet {
                    well_id: well_id.clone(),
                    spec,
                }),
                Err(error) => {
                    build_error = Some(error);
                    break;
                }
            }
        }
        match build_error.map_or_else(
            || equations::create_complete_log_sets_batch(&conn, &complete),
            |error| Err(error),
        ) {
            Ok(m) => m,
            Err(error) => {
                set_err = Some(error);
                HashMap::new()
            }
        }
    };

    let mut writes: Vec<equations::CompleteWellWrite> = Vec::with_capacity(succ_ids.len());
    for (well_id, o) in req.well_ids.iter().zip(outcomes.iter()) {
        if let Outcome::Computed { depth, outputs } = o {
            if !answered(outputs) {
                continue;
            }
            if let Some(set_id) = set_ids.get(well_id) {
                writes.push(equations::CompleteWellWrite {
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
        let err = equations::write_computed_curves_with_ancestry_batch(&conn, &writes).err();
        // SB-MLA-055. Record which of these curves hold CLASS CODES, so a later re-frame or block
        // cannot average them into a value that is not any class. Declared from the manifest's
        // output keys and resolved through the same rename + prefix the write itself used, so a
        // renamed FACIES is still protected.
        //
        // After the write and never in place of it: a declaration is metadata about a curve, so
        // failing to record it must cost the metadata, not the run. It is also idempotent, which is
        // what lets a re-run re-declare rather than needing to know whether it already had.
        if err.is_none() {
            let class_names = class_output_names(&req.module, &out_names, &opts);
            if !class_names.is_empty() {
                for wr in &writes {
                    let _ = crate::db::declare_class_curves(&conn, &wr.well_id, &class_names, &req.module);
                }
            }
        }
        err
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
                } else if !answered(outputs) {
                    // Every output sample MISSING (e.g. gascorr with no precalc, rocktyping with
                    // no permeability). Checked BEFORE the set/write branches, because this well
                    // was deliberately given no output set — reporting "no output set allocated"
                    // would name the mechanism instead of the cause.
                    //
                    // A green "N samples → …" line here would be indistinguishable from a real
                    // result and would total into History as a success; nothing is written either,
                    // so the catalog can still tell "never run" from "ran and could not answer".
                    let mut names: Vec<String> = outputs.keys().cloned().collect();
                    names.sort();
                    ModuleRunResult {
                        well_id: well_id.clone(),
                        rows_written: 0,
                        output_curves: names,
                        error: Some(
                            "no finite output — every sample is missing (check inputs, e.g. precalc not run)".into(),
                        ),
                    }
                } else if let Some(e) = &set_err {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e.clone()) }
                } else if !set_ids.contains_key(well_id) {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some("no output set allocated for well".into()) }
                } else if let Some(e) = &write_err {
                    ModuleRunResult { well_id: well_id.clone(), rows_written: 0, output_curves: vec![], error: Some(e.clone()) }
                } else {
                    let mut names: Vec<String> = outputs.keys().cloned().collect();
                    names.sort();
                    ModuleRunResult { well_id: well_id.clone(), rows_written: depth.len(), output_curves: names, error: None }
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
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
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
    pub stats_only: bool
,
    #[serde(default)]
    pub custody: Option<equations::RunCustody>,
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
    /// **A permeability cutoff is active and this well carries no PERM at all**, so every sample
    /// failed it for want of data and the zero below is an absence of evidence, not a dry zone.
    /// Per well, so it is the same on every zone row of that well.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 7): *"no relation between em,
    /// wells still can have perm curves"* — whether a cutoff applies has no relation to whether
    /// this particular well was cored, and permeability can be MODELLED where it was not measured
    /// (`perm_coates`, `perm_timur`, the rocktyping family), so lacking a measured PERM is not a
    /// reason to be let off. The cutoff is now active whenever it is requested.
    ///
    /// That settles a rule the code used to hold in two contradictory halves. At the SAMPLE level
    /// a missing PERM correctly FAILED an active cutoff — confirmed `[x]` in `REVIEW.md` — but a
    /// well with no PERM anywhere switched the cutoff off for ITSELF one line earlier. Two wells
    /// of identical rock reported 0 and full net pay with `n_classified > 0` on both, and in a
    /// field roll-up they simply added together: **the less permeability data a well had, the more
    /// pay it booked.** The well-level test is gone and the sample-level rule now does the work.
    ///
    /// The flag survives the change with its meaning inverted, because the reader's problem is
    /// unchanged and only its direction moved. A well that books zero net pay across every zone
    /// looks exactly like a wet well; this is what says the interpretation never had the curve the
    /// cutoff asks about. **It means "a cutoff was requested and this well has nothing to answer it
    /// with", never "this well has no permeability"** — with no cutoff asked for there is nothing
    /// to report, and a flag that fired anyway would appear on every report anyone ever ran.
    #[serde(default)]
    pub perm_cutoff_no_data: bool,
}

const SUMMARY_FLAGS: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

/// PHIE as a pay calculation is allowed to read it: never negative, MISSING preserved.
///
/// The porosity modules already floor what they WRITE (`modules::PHIE_FLOOR`), but the motivating
/// case never passes through one — `docs/review_triage.md` finding 16. A vendor PHIE arriving by
/// LAS reads slightly negative over a tight carbonate streak, which is a routine artefact of a
/// sandstone-matrix density porosity rather than a corrupt curve. That streak reads low GR, clears
/// the VSH cutoff and is flagged SAND, and its `PHIE·(1−SWE)·h` is then SUBTRACTED from the SAND
/// row's hydrocarbon column. Measured, that took HPV more than 20 % below the floored answer while
/// RESERVOIR and PAY stayed byte-identical — so the two rows anyone checks first agreed with each
/// other while the third quietly did not, and the understatement was in the reassuring direction.
///
/// Applied ONCE per well so every consumer downstream sees one number: `hpv`, `avg_phie` and the
/// classifier cannot end up disagreeing about what the porosity at a depth was.
///
/// **`f32::max` returns the other side when one is NaN**, so the guard is load-bearing rather than
/// defensive: without it a MISSING sample would become a real 0.001 and start counting toward
/// `n_classified`, which is the one field that says whether the well was interpreted at all.
///
/// One function rather than a copy in each pay path — the cutoff SWEEP and the summary must agree
/// at the same cutoffs, and two copies is two places for the rule to drift.
fn floored_phie(raw: &[f32]) -> Vec<f32> {
    raw.iter().map(|&v| if v.is_nan() { v } else { v.max(modules::PHIE_FLOOR as f32) }).collect()
}

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
        // The cutoffs decide net pay, so WHICH version of PHIE and SWE they read is part of the
        // answer — a summary that cannot name its inputs' version cannot be reproduced.
        let (depth, columns) = match equations::fetch_curve_frame_from_set(
            &conn, well_id, &curve_names, req.input_set.as_deref(), None,
        ) {
            Ok((d, c)) if !d.is_empty() => (d, c),
            _ => continue,
        };
        let mut zones = match db::list_zones(&conn, well_id) {
            Ok(z) => z,
            Err(_) => continue,
        };
        drop(conn);

        let had_declared_zones = !zones.is_empty();
        if zones.is_empty() {
            zones.push(db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: depth[0],
                bottom_depth: *depth.last().unwrap(),
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            });
        }

        let n = depth.len();
        let vsh = &columns["VSH"];
        let phie_col = floored_phie(&columns["PHIE"]);
        let phie = &phie_col;
        let swe = &columns["SWE"];
        let perm = &columns["PERM"];
        // A requested cutoff is ALWAYS active — see `PaySummaryRow::perm_cutoff_no_data` for why
        // the well-level "does this well have any PERM?" test was removed. `classify_sample` fails
        // a sample whose PERM is missing, which is now the only rule in play.
        let has_perm_cut = req.perm_min.is_some();
        let perm_cutoff_no_data = has_perm_cut && !perm.iter().any(|v| !v.is_nan());

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
                return Err("pay-summary write refused: skip_version would create ancestry-free FLAG curves; use a versioned run"
                        .into());
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
                let custody = req.custody.as_ref().ok_or_else(|| {
                    "pay-summary write refused: explicit run custody is required".to_string()
                })?;
                let mut ancestry_curves =
                    vec!["VSH".to_string(), "PHIE".to_string(), "SWE".to_string()];
                if req.perm_min.is_some() && perm.iter().any(|value| value.is_finite()) {
                    ancestry_curves.push("PERM".into());
                }
                let inputs = ancestry_curves
                    .iter()
                    .map(|curve| (well_id.clone(), curve.clone(), curve.clone()))
                    .collect::<Vec<_>>() ;
                let zone_scope = if had_declared_zones {
                    equations::AncestryZoneScope::Defined(
                        zones
                            .iter()
                            .filter(|zone| zone.top_depth < zone.bottom_depth)
                            .map(|zone| equations::AncestryZone {
                                name: zone.zone_name.clone(),
                                top: zone.top_depth,
                                base: zone.bottom_depth,
                                source: custody.source_note.clone(),
                            })
                            .collect(),
                    )
                } else {
                    equations::AncestryZoneScope::WholeWell
                };
                let output_names = vec![
                    "FLAG_SAND".into(),
                    "FLAG_RESERVOIR".into(),
                    "FLAG_PAY".into(),
                ];
                let mut complete =
                    equations::complete_curve_run_spec(&conn, well_id, &spec.set_name,
                    &spec.module,
                    custody,
                    &inputs,
                    req.input_set.as_deref(),
                    serde_json::from_str(&spec.params_json).map_err(|error| {
                        format!("cannot record pay-summary parameters: {error}")
                    })?,
                    zone_scope,
                    &output_names,
                )?;
                complete.record_parameter_decisions(crate::param_sources::PAY_PARAMETER_TOPICS)?;
                // Previewing and then exporting the same report must not create two
                // indistinguishable PAYFLAG versions. Reuse is allowed only when every
                // material part of the live record matches; a changed input version,
                // value/source, zone/source, operator, output, or implementation creates
                // a new append-only version as usual.
                let already_current = output_names.iter().all(|curve| {
                    equations::curve_ancestry(&conn, well_id, curve)
                        .is_ok_and(|existing| existing.same_computation(complete.ancestry()))
                });
                if !already_current {
                    let (set_id, _) =
                        equations::create_complete_log_set(&conn, well_id, &complete)?;
                let batch: Vec<(&str, &[f32])> = vec![
                    ("FLAG_SAND", flag_sand.as_slice()),
                    ("FLAG_RESERVOIR", flag_res.as_slice()),
                    ("FLAG_PAY", flag_pay.as_slice()),
                ];
                equations::write_computed_curves_with_ancestry(&conn, well_id, &depth, &batch, &set_id)
                    ?;
            }
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
                    perm_cutoff_no_data,
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
    /// Sweep the cutoffs against THIS log set's stored curves rather than the current values —
    /// the same freedom the pay summary it informs has (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
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
        let (depth, columns) = match equations::fetch_curve_frame_from_set(
            &conn, well_id, &curve_names, req.input_set.as_deref(), None,
        ) {
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
        let phie_col = floored_phie(&columns["PHIE"]);
        let phie = &phie_col;
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
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    /// CORRECTNESS - SB-DBM-013 / SB-DBM-T13. The expected atomic refusal, Failed well
    /// state and configuration inventory come from `docs/PRD_v2/22_database-model.md`
    /// section 6, SB-DBM-T13, sourced there to F-03. The curve values are synthetic fixture
    /// inputs; this test asserts custody and rollback, not a petrophysical expected value.
    #[test]
    fn provenance_cannot_be_switched_off_and_a_failed_record_fails_the_write() {
        fn add_well(conn: &Connection, name: &str) -> String {
            let id = uuid::Uuid::new_v4();
            db::insert_well(conn, id, name, None, None, None).unwrap();
            db::insert_standard_curves(
                conn,
                id,
                vec![1000.0, 1001.0],
                vec![20.0, 120.0],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
            )
            .unwrap();
            id.to_string()
        }

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let control = add_well(&conn, "VERSIONED-CONTROL");
        let first_fault = add_well(&conn, "RECORD-FAILURE-ONE");
        let second_fault = add_well(&conn, "RECORD-FAILURE-TWO");
        let skip_candidate = add_well(&conn, "SWITCH-REFUSAL");
        let dbm = Mutex::new(conn);
        let request = |well_ids: Vec<String>, output_set: &str| RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids,
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            opts: HashMap::new(),
            output_set: Some(output_set.into()),
            input_set: None,
            custody: test_run_custody(),
        };

        // Positive side: ordinary execution must write both the curve and its complete record.
        let control_result =
            run_workflow_module_into(&dbm, &request(vec![control.clone()], "CONTROL"), None, None, None);
        assert_eq!(control_result.len(), 1);
        assert!(control_result[0].error.is_none(), "{:?}", control_result[0].error);
        {
            let conn = dbm.lock().unwrap();
            let paired: (i64, i64) = conn
                .query_row(
                    "SELECT
                         (SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH'),
                         (SELECT count(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'CONTROL')",
                    duckdb::params![control],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(paired, (2, 1), "a successful output is inseparable from one run record");
            equations::curve_ancestry(&conn, &control, "VSH")
                .expect("the successful control curve must resolve its complete ancestry");
        }

        // Fault side: constrain this test database so the second FAULT set at version 1 rejects.
        // The first insert has already happened inside create_complete_log_sets_batch when the
        // second fails, so only a real transaction rollback can leave both wells untouched.
        dbm.lock()
            .unwrap()
            .execute(
                "CREATE UNIQUE INDEX sb_dbm_t13_fault ON log_sets(set_name, version)",
                [],
            )
            .unwrap();
        let registry = crate::jobs::new_registry();
        let job_id = uuid::Uuid::new_v4();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let progress = crate::jobs::register(
            &registry,
            job_id,
            "SB-DBM-T13",
            "Injected run-record failure",
            vec![
                (first_fault.clone(), "RECORD-FAILURE-ONE".into()),
                (second_fault.clone(), "RECORD-FAILURE-TWO".into()),
            ],
            cancel,
            true,
        );
        progress.running(2);
        let failed = run_workflow_module_into(
            &dbm,
            &request(vec![first_fault.clone(), second_fault.clone()], "FAULT"),
            None,
            None,
            Some(&progress),
        );
        assert_eq!(failed.len(), 2);
        assert!(
            failed.iter().all(|result| result.error.is_some() && result.rows_written == 0),
            "each affected well must report the record failure: {failed:?}"
        );
        let job = crate::jobs::list(&registry).pop().unwrap();
        assert_eq!(job.items.len(), 2);
        assert!(
            job.items
                .iter()
                .all(|item| item.state == crate::jobs::ItemState::Failed),
            "the serialized processing surface must report both wells Failed: {:?}",
            job.items
        );
        {
            let conn = dbm.lock().unwrap();
            let (sets, curves): (i64, i64) = conn
                .query_row(
                    "SELECT
                         (SELECT count(*) FROM log_sets WHERE set_name = 'FAULT'),
                         (SELECT count(*) FROM computed_curves WHERE well_id IN (?1, ?2))",
                    duckdb::params![first_fault, second_fault],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(sets, 0, "the first run-record insert must roll back with the second");
            assert_eq!(curves, 0, "a failed run record must leave no computed curve rows");
        }

        // The one request field whose old name suggests an ancestry-free mode must refuse, not
        // silently write. These legacy fixture curves are inputs only; the asserted output count
        // is zero, so they are not evidence for a shipping unversioned writer.
        {
            let conn = dbm.lock().unwrap();
            for (curve, values) in [
                ("VSH", [0.1, 0.1]),
                ("PHIE", [0.2, 0.2]),
                ("SWE", [0.3, 0.3]),
                ("PERM", [f32::NAN, f32::NAN]),
            ] {
                equations::write_computed_curve(
                    &conn,
                    &skip_candidate,
                    &[1000.0, 1001.0],
                    curve,
                    &values,
                )
                .unwrap();
            }
        }
        let refusal = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                well_ids: vec![skip_candidate.clone()],
                vsh_max: 0.5,
                phie_min: 0.1,
                swe_max: 0.5,
                perm_min: None,
                input_set: None,
                skip_version: true,
                stats_only: false,
                custody: Some(test_run_custody()),
            },
        )
        .expect_err("skip_version must be an explicit refusal rather than a provenance switch");
        assert!(refusal.contains("ancestry-free"), "{refusal}");
        let conn = dbm.lock().unwrap();
        let pay_rows: (i64, i64) = conn
            .query_row(
                "SELECT
                     (SELECT count(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'PAYFLAG'),
                     (SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name LIKE 'FLAG_%')",
                duckdb::params![skip_candidate],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(pay_rows, (0, 0), "the refused switch must write neither record nor curve");
        drop(conn);

        // Enumerate, rather than assume, every production environment/preference read. Because
        // the independent whole-corpus writer inventory finds no raw or legacy computed writer,
        // none of these values can select a provenance-free write path.
        let configuration = crate::core_ancestry_tests::production_configuration_read_inventory();
        assert!(
            configuration.iter().any(|line| line.contains("SANDIBUMI_DB_MEMORY"))
                && configuration.iter().any(|line| line.contains("localStorage.getItem"))
                && configuration.iter().any(|line| line.contains("sessionStorage.getItem"))
                && configuration.iter().any(|line| line.contains("current_setting("))
                && configuration.iter().any(|line| line.contains("FROM documents"))
                && configuration.iter().any(|line| line.contains("read_user_settings(")),
            "the inventory must cover environment, database, project, installed, persisted and session configuration reads: {configuration:?}"
        );
        let violations = crate::core_ancestry_tests::production_ancestry_bypass_violations();
        assert!(
            violations.is_empty(),
            "no configuration may select a legacy or raw computed writer:\n{}",
            violations.join("\n")
        );
    }

    /// CORRECTNESS - SB-DBM-007 / SB-DBM-T09. The expected `NOT_APPLICABLE` state and
    /// fail-closed serialization behavior are specified verbatim by
    /// `docs/PRD_v2/22_database-model.md` section 6, SB-DBM-T09. Numeric values below are
    /// synthetic fixture inputs; no petrophysical default, limit, or expected result is asserted.
    #[test]
    fn absent_is_a_named_state_never_an_empty_string() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "PARAMETER-STATE-FIXTURE", None, None, None).unwrap();
        let well_id = well_uuid.to_string();
        let depth = vec![1000.0_f32, 1001.0, 1002.0];
        db::insert_standard_curves(
            &conn,
            well_uuid,
            depth.clone(),
            vec![20.0, 40.0, 60.0],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
        )
        .unwrap();
        let dbm = Mutex::new(conn);

        let equation = equations::EquationDef {
            equation_id: uuid::Uuid::new_v4().to_string(),
            name: "PARAMETERLESS_EQUATION".into(),
            description: Some("Synthetic SB-DBM-T09 fixture".into()),
            script: "gr / 2.0".into(),
            input_curves: vec!["GR".into()],
            output_curve: "GR_HALF".into(),
            output_units: Some("gAPI".into()),
            language: "rhai".into(),
        };
        let equation_result = equations::run_equation(
            &dbm,
            &equation,
            std::slice::from_ref(&well_id),
            &test_run_custody(),
            None,
        );
        assert_eq!(equation_result.len(), 1);
        assert!(equation_result[0].error.is_none(), "{:?}", equation_result[0].error);

        let equation_params: String = dbm
            .lock()
            .unwrap()
            .query_row(
                "SELECT params_json FROM log_sets WHERE well_id = ?1 AND module = ?2",
                duckdb::params![well_id, "equation:PARAMETERLESS_EQUATION"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!equation_params.is_empty(), "no-parameter provenance must not be an empty string");
        let equation_ancestry = equations::parse_curve_ancestry(&equation_params).unwrap();
        assert!(equation_ancestry.parameters.is_empty(), "equation metadata is not a parameter");
        assert_eq!(
            equation_ancestry.parameter_state,
            Some(equations::ProvenanceAbsentState::NotApplicable),
            "a genuine no-parameter run has the specified named state"
        );
        let equation_json: serde_json::Value = serde_json::from_str(&equation_params).unwrap();
        assert_eq!(
            equation_json[equations::CURVE_ANCESTRY_KEY]["parameter_state"],
            "NOT_APPLICABLE",
            "the persisted reader surface carries the state verbatim"
        );
        let mut legacy_equation_json = equation_json.clone();
        legacy_equation_json[equations::CURVE_ANCESTRY_KEY]["schema_version"] =
            serde_json::json!(2);
        legacy_equation_json[equations::CURVE_ANCESTRY_KEY]
            .as_object_mut()
            .unwrap()
            .remove("parameter_state");
        let legacy_ancestry =
            equations::parse_curve_ancestry(&legacy_equation_json.to_string()).unwrap();
        assert_eq!(
            legacy_ancestry.parameter_state,
            Some(equations::ProvenanceAbsentState::LegacyUnrecorded),
            "an old empty collection must not be rewritten as known NOT_APPLICABLE"
        );

        let module_request = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![well_id.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            opts: HashMap::new(),
            output_set: Some("SERIALIZATION-REFUSAL".into()),
            input_set: None,
            custody: test_run_custody(),
        };
        let module_result = run_workflow_module_into_with_parameter_serializer(
            &dbm,
            &module_request,
            None,
            None,
            None,
            &|_| Err("injected parameter serialization failure".into()),
        );
        assert_eq!(module_result.len(), 1);
        let error = module_result[0]
            .error
            .as_deref()
            .expect("a parameter serialization failure must fail the module run");
        assert!(error.contains("injected parameter serialization failure"), "{error}");

        let conn = dbm.lock().unwrap();
        let module_sets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND module = 'vsh_gr'",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap();
        let module_curves: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH'",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(module_sets, 0, "a serialization failure must not leave a run record");
        assert_eq!(module_curves, 0, "a serialization failure must not leave computed values");
    }

    /// CORRECTNESS — SB-DBM-004 / SB-DBM-T06, sourced to SB-CORE-011 and F-18 / ledger R-10.
    /// The five values below are synthetic fixture inputs, not petrophysical defaults: the proof
    /// is that the saved run contains the complete effective set, distinguishes the two explicit
    /// values from the three defaults, and retains the exact manifest identity that supplied those
    /// defaults after a later manifest changes.
    #[test]
    fn a_run_records_all_effective_parameters_and_keeps_the_default_manifest_version_after_that_manifest_changes(
    ) {
        fn manifest(defaults: [f64; 5]) -> modules::ModuleSpec {
            let arguments = ["P_ALPHA", "P_BETA", "P_GAMMA", "P_DELTA", "P_EPSILON"];
            modules::ModuleSpec {
                name: "sb_dbm_t06_fixture".into(),
                title: "Synthetic effective-parameter fixture".into(),
                category: "Test fixture".into(),
                doc: "SB-DBM-T06 synthetic fixture".into(),
                args: arguments
                    .into_iter()
                    .zip(defaults)
                    .map(|(name, default)| {
                        let mut argument = modules::param(
                            name,
                            "Synthetic scalar",
                            "unitless",
                            default,
                            -1.0,
                            1.0,
                            "docs/PRD_v2/22_database-model.md SB-DBM-T06 synthetic fixture",
                        );
                        // Ranges are irrelevant to this persistence contract. Removing them keeps
                        // the fixture from resembling a product validity limit.
                        argument.min = None;
                        argument.max = None;
                        argument
                    })
                    .collect(),
            }
        }

        fn manifest_version(spec: &modules::ModuleSpec) -> String {
            let configurable = spec
                .args
                .iter()
                .filter(|argument| {
                    matches!(argument.kind, ArgKind::Param | ArgKind::Option | ArgKind::Text)
                })
                .collect::<Vec<_>>();
            let canonical = serde_json::to_vec(&(spec.name.as_str(), &configurable)).unwrap();
            let digest = Sha256::digest(canonical);
            format!(
                "sha256:{}",
                digest
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            )
        }

        fn recorded(
            conn: &duckdb::Connection,
            set_id: &str,
        ) -> Vec<(String, f64, String, Option<String>)> {
            let mut statement = conn
                .prepare(
                    "SELECT name, value_json, resolution, manifest_version
                     FROM run_parameters
                     WHERE set_id = ?1
                     ORDER BY position",
                )
                .unwrap();
            statement
                .query_map(duckdb::params![set_id], |row| {
                    let value_json: String = row.get(1)?;
                    let value = serde_json::from_str::<f64>(&value_json).unwrap();
                    Ok((row.get(0)?, value, row.get(2)?, row.get(3)?))
                })
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap()
        }

        fn save(
            conn: &duckdb::Connection,
            well_id: &str,
            spec: &modules::ModuleSpec,
        ) -> equations::CompleteSetId {
            let request = RunModuleRequest {
                module: spec.name.clone(),
                well_ids: vec![well_id.to_string()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("P_ALPHA".into(), 11.0), ("P_DELTA".into(), 44.0)]),
                opts: HashMap::new(),
                output_set: Some("EFFECTIVE_PARAMETERS".into()),
                input_set: None,
                custody: test_run_custody(),
            };
            let complete = complete_module_log_spec(
                conn,
                well_id,
                &request,
                spec,
                &build_opts(spec, &request.opts, &request.log_inputs),
                &[],
                &["FIXTURE_RESULT".into()],
                &|parameters| serde_json::to_value(parameters).map_err(|error| error.to_string()),
            )
            .unwrap();
            equations::create_complete_log_set(conn, well_id, &complete)
                .unwrap()
                .0
        }

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "SYNTHETIC", Some("Synthetic"), None, None).unwrap();
        let well_id = well_uuid.to_string();

        let first_manifest = manifest([10.0, 20.0, 30.0, 40.0, 50.0]);
        let first_version = manifest_version(&first_manifest);
        let first_set = save(&conn, &well_id, &first_manifest);
        let original = recorded(&conn, first_set.as_str());
        assert_eq!(
            original,
            vec![
                ("P_ALPHA".into(), 11.0, "EXPLICIT".into(), None),
                ("P_BETA".into(), 20.0, "DEFAULTED".into(), Some(first_version.clone())),
                ("P_GAMMA".into(), 30.0, "DEFAULTED".into(), Some(first_version.clone())),
                ("P_DELTA".into(), 44.0, "EXPLICIT".into(), None),
                ("P_EPSILON".into(), 50.0, "DEFAULTED".into(), Some(first_version.clone())),
            ]
        );

        let changed_manifest = manifest([10.0, 20.0, 300.0, 40.0, 50.0]);
        let changed_version = manifest_version(&changed_manifest);
        assert_ne!(changed_version, first_version, "changing a default changes manifest identity");
        let changed_set = save(&conn, &well_id, &changed_manifest);
        let changed = recorded(&conn, changed_set.as_str());
        assert_eq!(changed[2].1, 300.0);
        assert_eq!(changed[2].2, "DEFAULTED");
        assert_eq!(changed[2].3.as_deref(), Some(changed_version.as_str()));

        assert_eq!(
            recorded(&conn, first_set.as_str()),
            original,
            "a later manifest must not reinterpret the original run"
        );
    }

    /// CORRECTNESS — SB-DBM-006 / SB-DBM-T08. Source: `22_database-model.md` F-04,
    /// SB-DBM-006 and SB-DBM-T08. The three GR arrays are synthetic fixture inputs; the flip
    /// outputs are independently derived around each array's arithmetic mean. The contract is the
    /// stored decision: exact chosen identity plus set version, declared FINAL_FLAG rule and both
    /// rejected identities and set versions. Reflagging selects the opposite identity so a mnemonic-only snapshot, a
    /// winner-only record or a resolver disconnected from the numeric reader cannot pass.
    #[test]
    fn a_module_run_records_the_final_curve_identity_and_both_rejected_candidates_then_records_the_reflagged_choice(
    ) {
        fn recorded_input(
            conn: &duckdb::Connection,
            well_id: &str,
            output_set: &str,
        ) -> equations::AncestryInput {
            let params_json: String = conn
                .query_row(
                    "SELECT params_json FROM log_sets
                     WHERE well_id = ?1 AND set_name = ?2
                     ORDER BY version DESC LIMIT 1",
                    duckdb::params![well_id, output_set],
                    |row| row.get(0),
                )
                .unwrap();
            equations::parse_curve_ancestry(&params_json)
                .unwrap()
                .inputs
                .into_iter()
                .find(|input| input.argument == "CURVE")
                .expect("the flip run records its CURVE input")
        }

        fn rejected_identities(
            input: &equations::AncestryInput,
        ) -> std::collections::BTreeSet<(String, String, Option<i64>)> {
            input
                .rejected_candidates
                .iter()
                .map(|candidate| {
                    (
                        candidate.curve_id.clone(),
                        candidate.log_set.clone(),
                        candidate.set_version,
                    )
                })
                .collect()
        }

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "RESOLUTION-FIXTURE", None, None, None).unwrap();
        let well_id = well_uuid.to_string();
        let depth = vec![1000.0_f32, 1001.0, 1002.0];
        db::insert_standard_curves(
            &conn,
            well_uuid,
            depth.clone(),
            vec![1.0, 2.0, 3.0],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
            vec![f32::NAN; depth.len()],
        )
        .unwrap();

        let first = db::upsert_curve_meta(
            &conn,
            &well_id,
            "PASS_A",
            "GR",
            Some("gAPI"),
            Some("GR"),
            Some("SB-DBM-T08 fixture"),
            Some(1),
        )
        .unwrap();
        let second = db::upsert_curve_meta(
            &conn,
            &well_id,
            "PASS_A",
            "GR",
            Some("gAPI"),
            Some("GR"),
            Some("SB-DBM-T08 fixture"),
            Some(2),
        )
        .unwrap();
        let initially_final = db::upsert_curve_meta(
            &conn,
            &well_id,
            "PASS_B",
            "GR",
            Some("gAPI"),
            Some("GR"),
            Some("SB-DBM-T08 fixture"),
            Some(1),
        )
        .unwrap();
        db::insert_curve_samples(&conn, &first, &depth, &[10.0, 20.0, 30.0]).unwrap();
        db::insert_curve_samples(&conn, &second, &depth, &[40.0, 50.0, 60.0]).unwrap();
        db::insert_curve_samples(&conn, &initially_final, &depth, &[70.0, 80.0, 90.0])
            .unwrap();
        assert_eq!(
            db::set_generic_curve_final(&conn, &initially_final, true).unwrap(),
            None,
            "the fixture starts without an implicit Final decision"
        );

        let dbm = Mutex::new(conn);
        let run = |output_set: &str| {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "flip".into(),
                    well_ids: vec![well_id.clone()],
                    log_inputs: HashMap::from([("CURVE".into(), "GR".into())]),
                    params: HashMap::new(),
                    opts: HashMap::from([
                        ("OPT_PIVOT".into(), "MEAN".into()),
                        ("OPT_FLAG".into(), "NO".into()),
                    ]),
                    output_set: Some(output_set.into()),
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
        };

        let first_run = run("RESOLUTION_A");
        assert!(first_run[0].error.is_none(), "first run: {:?}", first_run[0].error);
        {
            let conn = dbm.lock().unwrap();
            let input = recorded_input(&conn, &well_id, "RESOLUTION_A");
            assert_eq!(input.chosen_curve_id.as_deref(), Some(initially_final.as_str()));
            assert_ne!(input.chosen_curve_id.as_deref(), Some("GR"), "a mnemonic is not an identity");
            assert_eq!(input.log_set, "PASS_B");
            assert_eq!(input.set_version, Some(2));
            assert_eq!(input.rule, Some(equations::CurveResolutionRule::FinalFlag));
            assert_eq!(
                rejected_identities(&input),
                std::collections::BTreeSet::from([
                    (first.clone(), "PASS_A".into(), Some(1)),
                    (second.clone(), "PASS_A".into(), Some(1)),
                ])
            );
            let (_, columns) = equations::fetch_curve_frame(&conn, &well_id, &["GR_C".into()]).unwrap();
            assert_eq!(columns["GR_C"], vec![90.0, 80.0, 70.0]);

            assert_eq!(
                db::set_generic_curve_final(&conn, &first, true).unwrap().as_deref(),
                Some(initially_final.as_str()),
                "reflagging reports the displaced identity so the edit is undoable"
            );
        }

        let second_run = run("RESOLUTION_B");
        assert!(second_run[0].error.is_none(), "second run: {:?}", second_run[0].error);
        let conn = dbm.lock().unwrap();
        let input = recorded_input(&conn, &well_id, "RESOLUTION_B");
        assert_eq!(input.chosen_curve_id.as_deref(), Some(first.as_str()));
        assert_eq!(input.log_set, "PASS_A");
        assert_eq!(input.set_version, Some(2));
        assert_eq!(input.rule, Some(equations::CurveResolutionRule::FinalFlag));
        assert_eq!(
            rejected_identities(&input),
            std::collections::BTreeSet::from([
                (second, "PASS_A".into(), Some(1)),
                (initially_final, "PASS_B".into(), Some(3)),
            ])
        );
        let (_, columns) = equations::fetch_curve_frame(&conn, &well_id, &["GR_C".into()]).unwrap();
        assert_eq!(columns["GR_C"], vec![30.0, 20.0, 10.0]);

        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'RESOLUTION_B'
                 ORDER BY version DESC LIMIT 1",
                duckdb::params![well_id],
                |row| row.get(0),
            )
            .unwrap();
        let mut incomplete = equations::parse_curve_ancestry(&params_json).unwrap();
        incomplete.inputs[0].chosen_curve_id = None;
        assert!(
            equations::CompleteLogSetSpec::try_new("INCOMPLETE", incomplete).is_err(),
            "schema v2 must not accept rejected candidates beside a mnemonic-only input"
        );
    }

    /// SB-MLA-055, the declaration half. A class output is registered under the name the run
    /// actually wrote — through the per-output rename AND the universal prefix — because a
    /// declaration filed under the manifest key would protect a curve nobody has.
    ///
    /// Pinned from BOTH sides, and the second side is the one that matters. `gmm_facies` writes
    /// FACIES_GMM beside FPROB, and FPROB is an ordinary probability: averageable, interpolatable,
    /// continuous. An implementation that flagged the whole module, or matched on a `FACIES`
    /// prefix, would pass the first assertion and quietly resample the user's probability curve by
    /// MODE. Neither half alone would catch that.
    #[test]
    fn a_class_output_is_declared_under_the_name_the_run_wrote_and_a_probability_output_is_not() {
        let out_names = vec![
            ("FACIES_GMM".to_string(), "LITHO".to_string()), // renamed by the user
            ("FPROB".to_string(), "FPROB".to_string()),
        ];
        let mut opts: HashMap<String, String> = HashMap::new();
        opts.insert(OUT_PREFIX_OPT.to_string(), "test_".into());

        let got = class_output_names("gmm_facies", &out_names, &opts);
        assert_eq!(got, vec!["TEST_LITHO".to_string()], "the rename and the prefix both apply");
        assert!(!got.iter().any(|n| n.contains("FPROB")), "a probability curve is continuous and stays averageable");

        // Without a prefix, and without a rename.
        let plain = vec![("FACIES".to_string(), "FACIES".to_string())];
        assert_eq!(
            class_output_names("electrofacies", &plain, &HashMap::new()),
            vec!["FACIES".to_string()]
        );
        // A module with no class outputs declares nothing — the flag is opt-in, so a new module
        // cannot inherit protection it never asked for.
        assert!(class_output_names("vsh", &plain, &HashMap::new()).is_empty());
    }

    /// The enforcement half. A DECLARED class curve is resampled by a class-safe method whatever
    /// was asked for; an undeclared curve is left exactly as the user set it.
    ///
    /// That second half is not politeness — `reframe`'s own doc promises it. A caliper logged in
    /// whole inches passes `looks_discrete`, and the user must be able to say MEAN and get MEAN.
    /// So a guess may pick the DEFAULT and only a declaration may override a decision; a test that
    /// pinned the coercion alone would be satisfied by a heuristic that breaks that promise.
    #[test]
    fn a_declared_class_curve_is_never_averaged_and_an_undeclared_one_keeps_the_method_asked_for() {
        use crate::reframe::{class_safe_method, Method};

        // Everything that can invent a value becomes MODE, or NEAREST where it was point-wise.
        assert_eq!(class_safe_method(Method::Mean), Method::Mode);
        assert_eq!(class_safe_method(Method::Geometric), Method::Mode);
        assert_eq!(class_safe_method(Method::Harmonic), Method::Mode);
        // MEDIAN is in the list on purpose: reframe's `combine` takes it through R-type-7
        // percentile, so an even-count interval of {1, 2} returns 1.5 - not a class.
        assert_eq!(class_safe_method(Method::Median), Method::Mode);
        assert_eq!(class_safe_method(Method::Interpolate), Method::Nearest);
        // Already safe, and unchanged - a coercion that moved these would be churn the user sees.
        assert_eq!(class_safe_method(Method::Nearest), Method::Nearest);
        assert_eq!(class_safe_method(Method::Mode), Method::Mode);
        assert_eq!(class_safe_method(Method::Auto), Method::Mode);

        // The other side: a curve nobody declared is not touched by any of this. `class_safe_method`
        // is only ever reached through the registry lookup, so an undeclared FACIES-looking curve
        // still gets the method the request named.
        let plain = vec![("FACIES".to_string(), "FACIES".to_string())];
        assert!(
            class_output_names("smooth", &plain, &HashMap::new()).is_empty(),
            "nothing declares a curve a class curve except the module that produced it as one",
        );
    }

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
            input_set: None,
            well_ids: vec![well.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.6,
            perm_min: None,
            skip_version: false,
            // Stats only: the point of the test is the returned rows, and this keeps it from
            // writing FLAG_* curves as a side effect.
            stats_only: true
        ,
            custody: None,
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

    /// T-BATCH-08 (1) — a permeability cutoff applies to every well it is asked for, including the
    /// ones with no permeability.
    ///
    /// `classify_sample` is emphatic that a SAMPLE with no PERM cannot demonstrate it passes an
    /// active cutoff, so it fails (`classify_sample_nan_propagation` pins that, and it is a
    /// confirmed `[x]` in REVIEW.md). Until 2026-08-01 whether the cutoff was active at all was
    /// decided per WELL one line earlier — `perm_min.is_some() && perm.iter().any(|v| !v.is_nan())`
    /// — so a well carrying NO permeability anywhere switched the cutoff off for itself and
    /// reported its full pay. Two halves of one rule, disagreeing in the damaging direction: the
    /// well that measured 1 mD against a 1000 mD cutoff was excluded while the well that measured
    /// nothing sailed through, and in a field roll-up those rows added together.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 7): *"no relation between em,
    /// wells still can have perm curves"* — a cutoff's applicability has no relation to whether
    /// this well happened to be cored, and permeability can be modelled where it was not measured.
    /// The well-level test is gone; the sample-level rule is the only one left.
    ///
    /// Both halves of the outcome are asserted, because the reason this needed a decision at all is
    /// that the safe-looking half is only half: the uncored well now books zero, which on a page is
    /// indistinguishable from a wet well. `perm_cutoff_no_data` is what separates them, and it is
    /// asserted here rather than left to the report to remember.
    #[test]
    fn a_well_with_no_perm_fails_the_cutoff_and_says_why() {
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
                    input_set: None,
                    well_ids: vec![no_perm.clone(), low_perm.clone()],
                    vsh_max: 0.5,
                    phie_min: 0.1,
                    swe_max: 0.6,
                    perm_min,
                    skip_version: false,
                    stats_only: true
                ,
                    custody: None,
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

        // And so is the well that measured none — it cannot be SHOWN to pass, which is the same
        // test the sample-level rule already applied. The two halves now agree.
        assert_eq!(
            pay(&cut, &no_perm).net,
            0.0,
            "a well with no PERM must fail an active cutoff, not be exempted from it"
        );
        assert_eq!(pay(&cut, &no_perm).hpv, 0.0, "and it books no hydrocarbon volume on missing data");

        // Both wells were fully interpreted, so `n_classified` is > 0 on both and cannot say why
        // either one came back at zero. It never could — which is why a SECOND discriminator was
        // needed rather than a cleverer reading of this one.
        assert!(pay(&cut, &no_perm).n_classified > 0);
        assert!(pay(&cut, &low_perm).n_classified > 0);

        // `perm_cutoff_no_data` is that discriminator, and it is the whole reason a zero here is
        // readable: the uncored well's zero means "nothing to judge with", the cored well's means
        // "judged and failed". Identical numbers, opposite statements.
        assert!(pay(&cut, &no_perm).perm_cutoff_no_data, "the well with no data must be marked");
        assert!(!pay(&cut, &low_perm).perm_cutoff_no_data, "the well that was judged must not be");

        // And it means "a cutoff was requested and this well has nothing to answer it with" — not
        // "this well has no permeability". With no cutoff asked for there is nothing to report, and
        // a flag that fired anyway would appear on every report anyone ever ran without one.
        assert!(!pay(&open, &no_perm).perm_cutoff_no_data, "no cutoff requested, nothing to say");
        assert_eq!(pay(&open, &no_perm).net, base_no_perm, "and with no cutoff the pay is untouched");
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
                    input_set: None
                ,
                    custody: test_run_custody(),
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

        // And the catalog half is now honest too (`docs/review_triage.md` finding 10, fixed
        // 2026-08-01). Phase 2 used to write for any well whose outcome was `Computed` with a
        // non-empty output map — and an all-MISSING map is still non-empty — so the whole
        // rocktyping family was versioned in as curves blank end to end. The cost was not corrupt
        // values (they were honestly MISSING); it was that the catalog stopped distinguishing
        // "this was never run" from "this was run and could not answer", and burned a log-set
        // version recording the second as though it were an interpretation. T-RT-05's Expected
        // says the catalog must gain no FZI/RT rows, and now it does not.
        for name in outputs {
            assert_eq!(written(name), 0, "{name}: a run that reported failure must not version a curve");
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
            // CHARACTERIZATION fixture: these are the pre-SB-CORE-004 manifest values used to
            // keep this test about missing porosity, not about parameter-source policy. They are
            // explicit test inputs and are not shipping defaults.
            let params = HashMap::from([
                ("RW".to_string(), 0.3),
                ("M".to_string(), 2.0),
                ("N".to_string(), 2.0),
                ("A_CAP".to_string(), 0.45),
                ("B_QV".to_string(), 0.0057),
                ("C0".to_string(), -0.0071),
                ("RSF".to_string(), 2.25),
                ("CEC".to_string(), 0.0),
                ("RHOG".to_string(), 2.65),
            ]);
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "sw_rtc".into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params,
                    opts: HashMap::new(),
                    output_set: None,
                    input_set: None
                ,
                    custody: test_run_custody(),
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
                input_set: None,
                well_ids: vec![bare.clone(), good.clone()],
                vsh_max: 0.5,
                phie_min: 0.1,
                swe_max: 0.6,
                perm_min: None,
                skip_version: false,
                stats_only: true
            ,
                custody: None,
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

    /// T-PREP-05 — a geothermal gradient belongs to the WELL, so a per-zone override is refused.
    ///
    /// `precalc` computes `SURF_TEMP + TEMP_GRAD × TVDSS` from surface at EVERY sample rather than
    /// integrating down through the zones above it. Giving a lower zone its own gradient therefore
    /// did not bend the temperature profile, it STEPPED it: a 0.03 °C/m well with a 0.035 override
    /// below 1500 m jumped **10.5 °C across 100 m** where the undisturbed trend rises 3.0. Rock
    /// temperature is continuous — a 10 °C discontinuity at a formation top is not something the
    /// earth does — and it does not stay in FTEMP, because the Arps correction turns temperature
    /// into Rw and Rw goes straight into Sw.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 6): *"temperature is curves
    /// only"* — the trend belongs to the well and its product is a curve, so there is no per-zone
    /// gradient to integrate and the question of what temperature each zone STARTS at never arises.
    ///
    /// REFUSED rather than silently ignored, which is the half worth defending: quietly dropping
    /// the override would change the well's temperature, and so its Sw, with nothing on the log to
    /// say why. And refused per NAMED zone only — `*` still applies, because that gives the well
    /// one trend, which is exactly what a geothermal gradient is. Wells in one field genuinely do
    /// have different gradients, and the per-well parameter grid writes `*`.
    #[test]
    fn a_geothermal_gradient_is_refused_per_zone_and_accepted_per_well() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-TEMP", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let depth: Vec<f32> = (0..5).map(|i| 1400.0 + i as f32 * 50.0).collect();
        db::upsert_md_zone(&conn, &well, "LOWER", 1500.0, 2000.0).unwrap();

        let spec = modules::list_modules()
            .into_iter()
            .find(|s| s.name == "precalc")
            .expect("precalc is a registered module");
        assert!(
            spec.args.iter().find(|a| a.name == "TEMP_GRAD").expect("declared").well_scope,
            "TEMP_GRAD must be declared well-scoped, or nothing below this line is being tested"
        );
        // CHARACTERIZATION fixture: these pre-SB-CORE-004 manifest values are supplied only so
        // this test isolates well versus zone scope; they are not shipping defaults.
        let base = HashMap::from([
            ("TEMP_GRAD".to_string(), 0.026),
            ("PGRAD".to_string(), 0.433),
        ]);

        // Baseline: an unmodified run resolves.
        assert!(resolve_param_arrays(&conn, &well, &spec, &base, &depth).is_ok());

        // The override that made the step.
        db::set_zone_param(&conn, &well, "LOWER", "TEMP_GRAD", Some(0.035), None).unwrap();
        let err = resolve_param_arrays(&conn, &well, &spec, &base, &depth)
            .expect_err("a named-zone gradient must be refused, not silently dropped");
        assert!(err.contains("TEMP_GRAD"), "the message must name the parameter: {err}");
        assert!(err.contains("LOWER"), "and the zone it is on: {err}");
        assert!(err.contains('*'), "and the scope that does still work: {err}");

        // Cleared, it resolves again — the guard is not blanket-blocking.
        db::set_zone_param(&conn, &well, "LOWER", "TEMP_GRAD", None, None).unwrap();
        assert!(resolve_param_arrays(&conn, &well, &spec, &base, &depth).is_ok());

        // An override naming a zone this well does not have never applied and must not start
        // failing runs — only an override that would actually bite is refused.
        db::set_zone_param(&conn, &well, "NOT-A-ZONE-HERE", "TEMP_GRAD", Some(0.05), None).unwrap();
        assert!(
            resolve_param_arrays(&conn, &well, &spec, &base, &depth).is_ok(),
            "an inert override must stay inert rather than becoming a new failure"
        );

        // The well-wide scope survives, and applies everywhere.
        db::set_zone_param(&conn, &well, "*", "TEMP_GRAD", Some(0.035), None).unwrap();
        let good = resolve_param_arrays(&conn, &well, &spec, &base, &depth)
            .expect("a well-wide gradient is one trend and must be honoured");
        assert!(
            good["TEMP_GRAD"].iter().all(|v| (*v - 0.035).abs() < 1e-9),
            "the '*' scope must reach every sample"
        );

        // PGRAD has the identical shape and is deliberately NOT well-scoped: a pressure step at a
        // formation top is a pressure compartment, which is a real thing rock does. The asymmetry
        // is the physics, and asserting it here is what stops someone "tidying" it away.
        db::set_zone_param(&conn, &well, "LOWER", "PGRAD", Some(0.5), None).unwrap();
        let mixed = resolve_param_arrays(&conn, &well, &spec, &base, &depth)
            .expect("a per-zone pressure gradient stays legal");
        assert!((mixed["PGRAD"][0] - 0.433).abs() < 1e-9, "above the zone, the trend default");
        assert!((mixed["PGRAD"][4] - 0.5).abs() < 1e-9, "inside it, the override");
    }

    /// The NaN guard in `floored_phie` is load-bearing rather than defensive: `f32::max` returns
    /// the OTHER side when one is NaN, so without it a MISSING porosity would come back as a real
    /// 0.001 and start counting toward `n_classified` — the one field that says whether the well
    /// was interpreted at all.
    #[test]
    fn flooring_phie_leaves_missing_missing() {
        let out = floored_phie(&[-0.05, 0.0, f32::NAN, 0.25]);
        let floor = modules::PHIE_FLOOR as f32;
        assert_eq!(out[0], floor, "a negative porosity is floored");
        assert_eq!(out[1], floor, "and so is a hard zero — the floor is 0.001, not 0.0");
        assert!(out[2].is_nan(), "MISSING must stay MISSING");
        assert_eq!(out[3], 0.25, "a real porosity is untouched");
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
                input_set: None
            ,
                    custody: test_run_custody(),
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
                input_set: None
            ,
                custody: test_run_custody(),
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
            // CORRECTNESS fixture: P3/P97 are the cited chapter values. The reference pair is
            // explicit test arithmetic, not a product calibration or shipping default.
            params: HashMap::from([
                ("P_LOW".to_string(), 3.0),
                ("P_HIGH".to_string(), 97.0),
                ("GR_LOW_REF".to_string(), 20.0),
                ("GR_HIGH_REF".to_string(), 120.0),
            ]),
            opts: [("MASK".to_string(), "BADHOLE".to_string())]
                .into_iter()
                .collect(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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
        db::upsert_md_zone(&conn, &w, "Z1", 1000.0, 1001.5).unwrap();

        let dbm = Mutex::new(conn);
        let req = PaySummaryRequest {
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: false
        ,
            stats_only: true,
            custody: None,
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
        let vsh = [0.1; 4];
        let phie = [0.2; 4];
        let swe = [0.3; 4];
        let perm = [f32::NAN; 4];
        let input_spec = equations::LogSetSpec {
            set_name: "TEST_INPUTS".into(),
            module: "test_fixture".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (input_set_id, _) = equations::create_log_set(&conn, &w, &input_spec).unwrap();
        equations::write_computed_curves_versioned(
            &conn,
            &w,
            &depths,
            &[
                ("VSH", &vsh),
                ("PHIE", &phie),
                ("SWE", &swe),
                ("PERM", &perm),
            ],
            &input_set_id,
        ).unwrap();
        db::upsert_md_zone(&conn, &w, "Z1", 1000.0, 1003.0).unwrap();
        let dbm = Mutex::new(conn);

        // Explicit run: versions the pay flags with the cutoffs recorded in provenance.
        let req = PaySummaryRequest {
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: false,
            stats_only: false
        ,
            custody: Some(test_run_custody()),
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

        // `skip_version` is retained only so an older caller receives an explicit refusal instead
        // of silently writing ancestry-free curves.
        let req_skip = PaySummaryRequest {
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: true,
            stats_only: false
        ,
            custody: Some(test_run_custody()),
        };
        let refusal =
            run_pay_summary(&dbm, &req_skip).expect_err("skip_version must not bypass ancestry");
        assert!(
            refusal.contains("ancestry-free"),
            "the refusal names the broken custody contract: {refusal}"
        );
        {
            let conn = dbm.lock().unwrap();
            let versions: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'PAYFLAG'",
                    duckdb::params![w],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(versions, 1, "the refused bypass must not add a PAYFLAG version"
            );
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
        let vsh = [0.1; 4];
        let phie = [0.2; 4];
        let swe = [0.3; 4];
        let perm = [f32::NAN; 4];
        let input_spec = equations::LogSetSpec {
            set_name: "TEST_INPUTS".into(),
            module: "test_fixture".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (input_set_id, _) = equations::create_log_set(&conn, &w, &input_spec).unwrap();
        equations::write_computed_curves_versioned(
            &conn,
            &w,
            &depths,
            &[
                ("VSH", &vsh),
                ("PHIE", &phie),
                ("SWE", &swe),
                ("PERM", &perm),
            ],
            &input_set_id,
        ).unwrap();
        db::upsert_md_zone(&conn, &w, "Z1", 1000.0, 1003.0).unwrap();
        let dbm = Mutex::new(conn);

        let base = PaySummaryRequest {
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.5,
            perm_min: None,
            skip_version: false,
            stats_only: true
        ,
            custody: None,
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
        let writing = PaySummaryRequest { stats_only: false, skip_version: false,
            custody: Some(test_run_custody()), ..base.clone() };
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
    /// P3/P97 are read from the cited manifest entries. The 20/120 reference pair is explicit
    /// arithmetic for this normalization-is-per-well test; SB-CORE-004 deliberately removed it
    /// from the product manifest because no source supports shipping it as a default.
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
        let (p_lo, p_hi) = (default_of("P_LOW"), default_of("P_HIGH"));
        // CHARACTERIZATION fixture: explicit arithmetic endpoints, not shipping defaults.
        let (lo_ref, hi_ref) = (20.0f32, 120.0f32);

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "gr_normalize".into(),
            well_ids: vec![a.to_string(), b.to_string()],
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("P_LOW".to_string(), p_lo as f64),
                ("P_HIGH".to_string(), p_hi as f64),
                ("GR_LOW_REF".to_string(), lo_ref as f64),
                ("GR_HIGH_REF".to_string(), hi_ref as f64),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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
    /// **An output prefix renames every curve a run writes, and nothing else.**
    ///
    /// The general form of the Condition and Frame families' own OUT field (Jauhar, 2026-08-05:
    /// *"each tools or modules should give user freedom to define ... their own curves"*). Most
    /// modules produce a curve whose NAME is the answer — `VSH`, `PHIE` — so renaming them one at
    /// a time is not the shape of the freedom; putting a whole trial run under a prefix is, and
    /// it leaves the interpretation the field is already using untouched.
    ///
    /// Handled once in the runner rather than in forty modules, for the reason `MASK` is. The
    /// control matters as much as the case: an EMPTY prefix must leave the names byte-identical,
    /// or every saved chain and every layout in every existing project would be pointing at
    /// curves that no longer exist.
    #[test]
    fn an_output_prefix_renames_every_curve_a_run_writes_and_an_empty_one_changes_nothing() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let run_with = |prefix: Option<&str>| -> Vec<String> {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-PFX", None, None, Some(0.0)).unwrap();
            let n = 5usize;
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            db::insert_standard_curves(
                &conn,
                wid,
                depths,
                (0..n).map(|i| 20.0 + i as f32 * 10.0).collect(), // GR
                vec![f32::NAN; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
            )
            .unwrap();
            let dbm = Mutex::new(conn);
            let mut opts: HashMap<String, String> = HashMap::new();
            if let Some(p) = prefix {
                opts.insert(OUT_PREFIX_OPT.to_string(), p.to_string());
            }
            let req = RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![wid.to_string()],
                log_inputs: HashMap::new(),
                // CHARACTERIZATION fixture: former manifest endpoints, now explicit test inputs.
                params: HashMap::from([("GR_MA".to_string(), 20.0), ("GR_SH".to_string(), 120.0)]),
                opts,
                output_set: None,
                input_set: None
            ,
                custody: test_run_custody(),
            };
            let r = run_workflow_module(&dbm, &req);
            assert!(r[0].error.is_none(), "vsh_gr: {:?}", r[0].error);
            let mut names = r[0].output_curves.clone();
            names.sort();
            names
        };

        let plain = run_with(None);
        assert!(!plain.is_empty(), "the module must write something for this test to mean anything");
        assert!(plain.iter().any(|n| n == "VSH"), "unprefixed run writes VSH: {plain:?}");

        // An empty prefix is the same as none — the case every existing project sends.
        assert_eq!(run_with(Some("   ")), plain, "a blank prefix must change nothing at all");

        let prefixed = run_with(Some("test_"));
        assert_eq!(
            prefixed,
            plain.iter().map(|n| format!("TEST_{n}")).collect::<Vec<_>>(),
            "every output is prefixed, upper-cased, and none is left behind"
        );
        assert!(
            !prefixed.iter().any(|n| n == "VSH"),
            "and the interpretation's own VSH is untouched — that is the whole point of a trial run"
        );
    }

    /// **Every module writes the keys its manifest declares.** The invariant the whole naming
    /// system rests on: [`resolve_output_names`] decides a name per DECLARED output and the runner
    /// then looks the emitted key up, so a module that invents a key of its own is written under a
    /// name no rename can reach and no dialog ever showed.
    ///
    /// Five modules used to do exactly that (`log_predict` returned `<target>_SYN`, `phi_cap`
    /// `<input>_CAP`, `splice` `<top>_SPL`, `depth_shift` `<input>_DS`, and every Condition and
    /// Frame module returned whatever its `OUT` text field said). The manifest said one thing and
    /// the run wrote another, which is why a dialog reading "Outputs: SYN" was telling the user
    /// something untrue. This is the check that stops it coming back — cheaply, because it needs
    /// no database: every module runs against one synthetic frame and only its KEYS are examined.
    #[test]
    fn every_module_returns_the_output_keys_its_manifest_declares() {
        let n = 8usize;
        let mut checked = 0usize;
        for spec in modules::list_modules() {
            let declared: Vec<String> = spec
                .args
                .iter()
                .filter(|a| a.kind == ArgKind::LogOut)
                .map(|a| a.name.clone())
                .collect();

            // A frame every module can read something from: depth, plus a plausible ramp for each
            // declared input under its own ARG name (which is how `ModuleContext::log` reads it).
            let mut logs: HashMap<String, Vec<f32>> = HashMap::new();
            logs.insert("DEPTH".into(), (0..n).map(|i| 1000.0 + i as f32).collect());
            let mut params: HashMap<String, Vec<f64>> = HashMap::new();
            let mut opts: HashMap<String, String> = HashMap::new();
            for a in &spec.args {
                match a.kind {
                    ArgKind::LogIn => {
                        logs.insert(a.name.clone(), (0..n).map(|i| 0.1 + i as f32 * 0.05).collect());
                        opts.insert(format!("__IN_{}", a.name), a.default.to_uppercase());
                    }
                    ArgKind::Param => {
                        let v = a.default.parse::<f64>().unwrap_or(1.0);
                        params.insert(a.name.clone(), vec![v; n]);
                    }
                    ArgKind::Option | ArgKind::Text => {
                        opts.insert(a.name.clone(), a.default.clone());
                    }
                    ArgKind::LogOut => {}
                }
            }
            params.insert(ZONE_INDEX_ARG.to_string(), vec![0.0; n]);

            let ctx = ModuleContext { n, logs, params, opts, depth_unit: Default::default() };
            // A module is free to REFUSE this synthetic frame — a despike window of zero, a bed
            // definition with no beds. What it may not do is answer under a name of its own.
            let Ok(out) = modules::run_module(&spec.name, &ctx) else { continue ;
            };
            checked += 1;
            for key in out.keys() {
                assert!(
                    declared.contains(key),
                    "{} wrote '{key}', which its manifest does not declare (declared: {declared:?}). \
                     Return the declared key and give the arg a log_out_as pattern instead.",
                    spec.name
                );
            }
        }
        assert!(checked > 20, "only {checked} modules actually ran — the check proved little");
    }

    /// **A pattern is the DEFAULT name; a rename replaces it; a blank rename means the default.**
    ///
    /// Jauhar, 2026-08-05: *"naming each output curve in bulk when modules gonna run"*. The three
    /// states are separate on purpose — clearing the box has to give the original name back rather
    /// than write an unnamed curve, which is what an "empty means empty" reading would do.
    #[test]
    fn an_output_pattern_is_the_default_name_and_a_rename_replaces_it() {
        let spec = modules::list_modules().into_iter().find(|m| m.name == "log_predict").unwrap();
        let named = |renames: &[(&str, &str)]| -> Vec<(String, String)> {
            let mut opts: HashMap<String, String> = HashMap::new();
            opts.insert("__IN_TARGET".into(), "RHOB".into());
            for (k, v) in renames {
                opts.insert(format!("{OUT_NAME_PREFIX}{k}"), (*v).to_string());
            }
            resolve_output_names(&spec, &opts).unwrap()
        };

        assert_eq!(named(&[])[0].1, "RHOB_SYN", "the pattern names it after the curve predicted");
        assert_eq!(named(&[("SYN", "  ")])[0].1, "RHOB_SYN", "a blank rename is the default, not a blank name");
        assert_eq!(named(&[("SYN", "perm_est")])[0].1, "PERM_EST", "a rename replaces the whole name");

        // A pattern whose token has nothing behind it (an optional input the user cleared) falls
        // back to the declared name rather than to a dangling `_SYN`.
        let bare = resolve_output_names(&spec, &HashMap::new()).unwrap();
        assert_eq!(bare[0].1, "SYN", "no input mnemonic, no pattern — the declared name stands");

        // A pattern may name an earlier output, which is how a Condition flag rides its curve.
        let despike = modules::list_modules().into_iter().find(|m| m.name == "despike").unwrap();
        let mut opts: HashMap<String, String> = HashMap::new();
        opts.insert("__IN_CURVE".into(), "GR".into());
        let plain = resolve_output_names(&despike, &opts).unwrap();
        assert_eq!(plain[0].1, "GR_C");
        assert_eq!(plain[1].1, "GR_C_SPK", "the flag follows the curve it belongs to");
        opts.insert(format!("{OUT_NAME_PREFIX}OUT_CURVE"), "GR_ED".into());
        let renamed = resolve_output_names(&despike, &opts).unwrap();
        assert_eq!(renamed[1].1, "GR_ED_SPK", "and follows it through a rename, rather than stranding GR_C_SPK");
    }

    /// **A name that would be shadowed, or collide, is refused BEFORE a single well runs.**
    ///
    /// `fetch_curve_frame` resolves the six standard mnemonics from `standard_curves` first, so a
    /// computed curve stored as `GR` is written, counted, reported — and never the one anything
    /// reads back. `condition.rs` and `frame.rs` each carried their own copy of this refusal and
    /// the other forty modules had none; now there is one check in front of all of them, and it
    /// runs on the spec rather than per well so the user gets one message instead of N.
    ///
    /// The collision half matters just as much: two outputs under one name means one silently
    /// replaces the other, and which one survives depends on hash order.
    #[test]
    fn an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs() {
        let despike = modules::list_modules().into_iter().find(|m| m.name == "despike").unwrap();
        let with = |renames: &[(&str, &str)]| {
            let mut opts: HashMap<String, String> = HashMap::new();
            opts.insert("__IN_CURVE".into(), "GR".into());
            for (k, v) in renames {
                opts.insert(format!("{OUT_NAME_PREFIX}{k}"), (*v).to_string());
            }
            resolve_output_names(&despike, &opts)
        };

        let err = with(&[("OUT_CURVE", "gr")]).expect_err("a standard mnemonic must be refused");
        assert!(err.contains("GR"), "the refusal names the curve: {err}");
        assert!(err.contains("shadow"), "and says why: {err}");

        let err = with(&[("OUT_FLAG", "rhob")]).expect_err("every output is checked, not just the first");
        assert!(err.contains("RHOB"), "{err}");

        let err = with(&[("OUT_FLAG", "GR_C")]).expect_err("two outputs on one name must be refused");
        assert!(err.contains("GR_C"), "the refusal names the collision: {err}");

        let err = with(&[("OUT_CURVE", "GR ED")]).expect_err("a space would survive into every export");
        assert!(err.contains("space"), "{err}");

        // And the control: an ordinary rename is accepted, so none of the above is a blanket ban.
        assert_eq!(with(&[("OUT_CURVE", "gr_ed")]).unwrap()[0].1, "GR_ED");
    }

    /// CORRECTNESS — SB-DBM-029 / SB-DBM-T28. The refusal, named frame, immobility
    /// assertion and OWN-frame control come from `docs/PRD_v2/22_database-model.md`
    /// section 6, SB-DBM-T28, sourced there to F-16 and T2 `O` section 2.5: a module
    /// must "never write back to the Depth curve". Curve values and the 1.0 depth step
    /// are synthetic fixture inputs, not petrophysical expected values or product defaults.
    #[test]
    fn a_module_cannot_write_an_existing_reference_column_and_a_different_depth_basis_is_a_new_own_frame() {
        fn frame_snapshot(
            conn: &Connection,
            well_id: &str,
        ) -> (
            Vec<(
                u32,
                Option<u32>,
                Option<u32>,
                Option<u32>,
                Option<u32>,
                Option<u32>,
                Option<u32>,
            )>,
            Vec<(Option<String>, u32, String, u32)>,
        ) {
            let mut standard_stmt = conn
                .prepare(
                    "SELECT depth, gr, res_deep, nphi, rhob, dt, sp
                     FROM standard_curves
                     WHERE well_id = ?1
                     ORDER BY depth",
                )
                .unwrap();
            let standard = standard_stmt
                .query_map(duckdb::params![well_id], |row| {
                    Ok((
                        row.get::<_, f32>(0)?.to_bits(),
                        row.get::<_, Option<f32>>(1)?.map(f32::to_bits),
                        row.get::<_, Option<f32>>(2)?.map(f32::to_bits),
                        row.get::<_, Option<f32>>(3)?.map(f32::to_bits),
                        row.get::<_, Option<f32>>(4)?.map(f32::to_bits),
                        row.get::<_, Option<f32>>(5)?.map(f32::to_bits),
                        row.get::<_, Option<f32>>(6)?.map(f32::to_bits),
                    ))
                })
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap();

            let mut computed_stmt = conn
                .prepare(
                    "SELECT set_id, depth, curve_name, value
                     FROM computed_curves
                     WHERE well_id = ?1
                     ORDER BY set_id, depth, curve_name",
                )
                .unwrap();
            let computed = computed_stmt
                .query_map(duckdb::params![well_id], |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, f32>(1)?.to_bits(),
                        row.get(2)?,
                        row.get::<_, f32>(3)?.to_bits(),
                    ))
                })
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap();
            (standard, computed)
        }

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(
            &conn,
            well_uuid,
            "REFERENCE-FRAME-CHECK",
            Some("Synthetic"),
            None,
            None,
        )
        .unwrap();
        let well_id = well_uuid.to_string();
        let depth = vec![1000.0, 1000.5, 1001.0, 1001.5];
        db::insert_standard_curves(
            &conn,
            well_uuid,
            depth.clone(),
            vec![11.0, 12.0, 13.0, 14.0],
            vec![21.0, 22.0, 23.0, 24.0],
            vec![31.0, 32.0, 33.0, 34.0],
            vec![41.0, 42.0, 43.0, 44.0],
            vec![51.0, 52.0, 53.0, 54.0],
            vec![61.0, 62.0, 63.0, 64.0],
        )
        .unwrap();
        equations::write_computed_curve(
            &conn,
            &well_id,
            &depth,
            "PEER_SENTINEL",
            &[71.0, 72.0, 73.0, 74.0],
        )
        .unwrap();
        let before = frame_snapshot(&conn, &well_id);

        let request = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![well_id.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::from([(format!("{OUT_NAME_PREFIX}VSH"), "DEPTH".into())]),
            output_set: Some("REFERENCE-REFUSAL".into()),
            input_set: None,
            custody: test_run_custody(),
        };
        let database = Mutex::new(conn);
        let refused = run_workflow_module_into(&database, &request, None, None, None);
        let error = refused[0]
            .error
            .as_deref()
            .expect("the API boundary must refuse the reference-column write");
        assert!(error.contains("DEPTH"), "the refusal must name the reference column: {error}");
        assert!(error.contains("STANDARD frame"), "the refusal must name the protected frame: {error}");
        {
            let conn = database.lock().unwrap();
            assert_eq!(
                frame_snapshot(&conn, &well_id),
                before,
                "the refused module must not move any raw or computed peer curve on the frame"
            );
        }

        let conn = database.into_inner().unwrap();
        crate::reframe::save_curve_selection(
            &conn,
            &crate::reframe::CurveSelection {
                name: "REFERENCE-FRAME-CURVES".into(),
                mode: crate::reframe::CurveSelectionMode::Selected,
                members: vec!["GR".into()],
            },
        )
        .unwrap();
        let reframed = crate::reframe::run_reframe(
            &conn,
            &crate::reframe::ReframeRequest {
                well_ids: vec![well_id.clone()],
                source: crate::reframe::SourceSpec { kind: "standard".into(), name: None },
                selection_name: "REFERENCE-FRAME-CURVES".into(),
                substitutions: vec![],
                target: crate::reframe::TargetSpec {
                    kind: "step".into(),
                    step: Some(1.0),
                    align: false,
                    well_id: None,
                    set_name: None,
                    top: None,
                    base: None,
                },
                methods: HashMap::new(),
                default_method: crate::reframe::Method::Mean,
                output_set: "DIFFERENT-BASIS".into(),
                preview: false,
                custody: Some(test_run_custody()),
            },
        );
        assert!(reframed[0].error.is_none(), "the explicit new-frame path must run: {reframed:?}");

        let (set_id, frame): (String, String) = conn
            .query_row(
                "SELECT set_id, frame
                 FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'DIFFERENT-BASIS'
                 ORDER BY version DESC
                 LIMIT 1",
                duckdb::params![well_id.clone()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(frame, "OWN", "a different depth basis must be declared as an OWN frame");
        let own_depth = {
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT depth
                     FROM computed_curves_archive
                     WHERE set_id = ?1
                     ORDER BY depth",
                )
                .unwrap();
            stmt.query_map(duckdb::params![set_id], |row| row.get::<_, f32>(0))
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap()
        };
        assert_eq!(own_depth, vec![1000.0, 1001.0], "the declared 1.0 fixture step defines a distinct basis");
        assert_eq!(
            frame_snapshot(&conn, &well_id),
            before,
            "writing the OWN frame must leave the existing STANDARD frame byte-identical"
        );
    }

    /// **The parameter under test is PGRAD, and the choice is the point.** This test used to drive
    /// TEMP_GRAD, and doing so exposed finding 6: `precalc` computes each sample as
    /// `intercept + grad(i) * depth(i)` from SURFACE rather than integrating down through the zones
    /// above, so a per-zone gradient produced a STEP at the boundary rather than a kink — 10.5 degC
    /// across 100 m where the trend rises 3.0. Rock temperature is continuous and that reaches Sw
    /// through Rw, so as of 2026-08-01 a per-zone TEMPERATURE gradient is refused outright
    /// (`a_geothermal_gradient_is_refused_per_zone_and_accepted_per_well`).
    ///
    /// PRESSURE is the same arithmetic and the opposite physics: a pressure step at a formation top
    /// is a pressure compartment, which is a real thing rock does. So PGRAD stays zoneable, and it
    /// is the honest subject for a test of the interval-parameter model — the discontinuity it
    /// produces is the answer rather than an artefact.
    #[test]
    fn a_per_zone_pressure_gradient_reaches_exactly_its_own_samples() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
        let (boundary, base_grad, deep_grad, psurf) = (1500.0f32, 0.433f64, 0.5f64, 0.0f64);
        db::upsert_md_zone(&conn, &w, "SHALLOW", 1000.0, boundary).unwrap();
        db::upsert_md_zone(&conn, &w, "DEEP", boundary, 2100.0).unwrap();
        db::set_zone_param(&conn, &w, "DEEP", "PGRAD", Some(deep_grad as f32), None).unwrap();

        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "precalc".into(),
            well_ids: vec![w.clone()],
            log_inputs: HashMap::new(),
            // CHARACTERIZATION fixture: explicit pre-SB-CORE-004 inputs keep this test about the
            // half-open zone boundary. None of these values is restored as a shipping default.
            params: HashMap::from([
                ("SURF_TEMP".to_string(), 25.0),
                ("TEMP_GRAD".to_string(), 0.03),
                ("PSURF".to_string(), psurf),
                ("PGRAD".to_string(), base_grad),
                ("RMF_MEAS".to_string(), 0.2),
                ("RMF_TEMP".to_string(), 25.0),
            ]),
            opts: [("OPT_TU".to_string(), "degC".to_string())].into_iter().collect(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
        };
        let r = run_workflow_module(&dbm, &req);
        assert!(r[0].error.is_none(), "precalc: {:?}", r[0].error);

        let conn = dbm.lock().unwrap();
        let (d, cols) = equations::fetch_curve_frame(&conn, &w, &["FPRESS".into()]).unwrap();
        let fp = &cols["FPRESS"];
        assert_eq!(d.len(), n);

        for i in 0..n {
            let grad = if d[i] >= boundary { deep_grad } else { base_grad };
            let expect = psurf + grad * d[i] as f64;
            assert!(
                (fp[i] as f64 - expect).abs() < 1e-2,
                "sample {i} at {} m: FPRESS {} != {expect} — the {} gradient did not reach it",
                d[i],
                fp[i],
                if d[i] >= boundary { "zone" } else { "well" }
            );
        }

        // The boundary sample belongs to DEEP and to DEEP only. A closed interval on both sides
        // would let SHALLOW and DEEP both claim 1500 m, and which one won would be list order.
        let at_boundary = d.iter().position(|v| *v == boundary).expect("1500 m is a sample");
        assert!(
            (fp[at_boundary] as f64 - (psurf + deep_grad * boundary as f64)).abs() < 1e-2,
            "a sample exactly on the boundary must take the zone whose TOP it is"
        );
        assert!(
            (fp[at_boundary - 1] as f64 - (psurf + base_grad * (boundary - 100.0) as f64)).abs() < 1e-2,
            "and the sample above it must be untouched by that zone"
        );

        // The step across the boundary, which here is the ANSWER rather than an artefact — a
        // pressure compartment. Recorded so the deliberate temperature/pressure asymmetry is
        // visible from this end too, not only from the refusal test.
        let step = fp[at_boundary] - fp[at_boundary - 1];
        let within_zone = fp[at_boundary - 1] - fp[at_boundary - 2];
        assert!(
            step > 3.0 * within_zone,
            "a zoned pressure gradient must still step at the boundary: {step} across it against \
             {within_zone} within the zone above"
        );

        // The control: without the override every sample would sit on one line, so all of the
        // above would pass on a runner that ignored zone parameters entirely.
        assert!(
            (fp[at_boundary] as f64 - (psurf + base_grad * boundary as f64)).abs() > 1.0,
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
                    input_set: None
                ,
                    custody: test_run_custody(),
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
            out["SYN"][washout].is_nan(),
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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
                input_set: None
            ,
                custody: test_run_custody(),
            };
            run_workflow_module(&dbm, &req)
        };
        let curve = |name: &str| -> Vec<f32> {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(&conn, &w, &[name.to_string()]).unwrap();
            cols[name].clone()
        };

        // CHARACTERIZATION fixture: these are the former manifest inputs used only to isolate
        // computed-versus-raw provenance. SB-CORE-004 deliberately does not ship them as defaults.
        let (k_temp, t_ref, k_sal, salw) = (0.001, 25.0, 0.01, 100_000.0);
        let ec_params = [
            ("K_TEMP", k_temp),
            ("T_REF", t_ref),
            ("K_SAL", k_sal),
            ("SALW", salw),
        ];
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
            // CHARACTERIZATION fixture: explicit values isolate restored-curve consumption.
            params: HashMap::from([
                ("RHO_MA".to_string(), 2.645),
                ("RHO_SH".to_string(), 2.5),
                ("RHO_FL".to_string(), 1.0),
                ("RHO_DSH".to_string(), 2.65),
                ("RHO_W".to_string(), 1.0),
                ("PHIE_MAX".to_string(), 0.3),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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

    /// SB-CORE-T02. CORRECTNESS. Source: `docs/PRD_v2/04_CORE_REQUIREMENTS.md`,
    /// SB-CORE-001. The dependent side uses `depth_shift`, not the historically
    /// special-cased `sw_height`, so a one-name guard cannot pass. The independent
    /// side uses the same undeclared project and real stored samples so a blanket
    /// refusal cannot pass either. The exact dependent set comes from inspection of
    /// the live module algorithms and manifests: each member consumes DEPTH/TVD/TVDSS
    /// in a depth-unit-qualified equation, distance, thickness, or window.
    #[test]
    fn a_depth_dependent_module_refuses_an_undeclared_unit_while_an_independent_module_runs() {
        let expected_dependent = vec![
            "phimax",
            "ftemp_grad",
            "precalc",
            "condflag",
            "depth_shift",
            "splice",
            "despike",
            "smooth",
            "fill_gaps",
            "block",
            "bed_detect",
            "sw_height",
        ];
        let mut actual_dependent = Vec::new();
        for spec in modules::list_modules() {
            let dependency = module_depth_unit_dependency(&spec.name)
                .unwrap_or_else(|error| panic!("{}: {error}", spec.name));
            if dependency == DepthUnitDependency::Declared {
                actual_dependent.push(spec.name);
            }
        }
        assert_eq!(actual_dependent, expected_dependent, "the live module registry and its depth-unit inventory must agree");
        assert!(
            module_depth_unit_dependency("unregistered_depth_consumer").is_err(),
            "a future module must be classified explicitly, never assumed independent"
        );

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_id, "UNDECLARED-DEPTH-UNIT", None, None, None).unwrap();
        let depth = vec![1000.0_f32, 1000.5, 1001.0];
        db::insert_standard_curves(
            &conn,
            well_id,
            depth,
            vec![40.0, 70.0, 100.0],
            vec![2.0; 3],
            vec![0.2; 3],
            vec![2.4; 3],
            vec![80.0; 3],
            vec![10.0; 3],
        )
        .unwrap();
        assert_eq!(crate::units::project_depth_unit(&conn).unwrap(), None);

        let well = well_id.to_string();
        let dbm = Mutex::new(conn);
        let refused = run_workflow_module_into(
            &dbm,
            &RunModuleRequest {
                module: "depth_shift".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::new(),
                opts: HashMap::new(),
                output_set: None,
                input_set: None
            ,
                custody: test_run_custody(),
            },
            None,
            None,
            None,
        );

        assert_eq!(refused.len(), 1);
        let error = refused[0].error.as_deref().expect("an undeclared unit must refuse depth_shift");
        assert!(error.contains("depth_shift requires a declared project depth unit"), "{error}");

        let independent = run_workflow_module_into(
            &dbm,
            &RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                // CHARACTERIZATION fixture: explicit endpoints keep this side depth-independent.
                params: HashMap::from([("GR_MA".to_string(), 20.0), ("GR_SH".to_string(), 120.0)]),
                opts: HashMap::new(),
                output_set: None,
                input_set: None
            ,
                custody: test_run_custody(),
            },
            None,
            None,
            None,
        );
        assert_eq!(independent.len(), 1);
        assert!(independent[0].error.is_none(), "a depth-independent run stays available: {:?}", independent[0].error);
        assert_eq!(independent[0].rows_written, 3);

        let conn = dbm.lock().unwrap();
        let shifted: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'GR_DS'",
                duckdb::params![well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(shifted, 0, "a refused dependent run must not write its output");
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
            // CHARACTERIZATION fixture: former manifest endpoints, now explicit test inputs.
            params: HashMap::from([("GR_MA".to_string(), 20.0), ("GR_SH".to_string(), 120.0)]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();

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
            // CHARACTERIZATION fixture: former manifest values are explicit so the test remains
            // about consuming the deviation survey, not about parameter-source policy.
            params: HashMap::from([
                ("FWL".to_string(), FWL),
                ("RHO_HC".to_string(), 0.8),
                ("IFT_RES".to_string(), 26.0),
                ("SWH_A".to_string(), 0.5),
                ("SWH_B".to_string(), -0.4),
                ("SWT_IRR".to_string(), 0.0),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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
        db::upsert_md_zone(&conn, &w, "UPPER", 1000.0, 1010.0).unwrap();
        db::upsert_md_zone(&conn, &w, "LOWER", 1010.0, 1020.0).unwrap();

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
                input_set: None
            ,
                custody: test_run_custody(),
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
            // CHARACTERIZATION fixture: former manifest endpoints, now explicit test inputs.
            params: HashMap::from([("GR_MA".to_string(), 20.0), ("GR_SH".to_string(), 120.0)]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None
        ,
            custody: test_run_custody(),
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
        let run = |module: &str,
                   log_inputs: &[(&str, &str)],
                   params: &[(&str, f64)],
                   opts: &[(&str, &str)]| {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: well_ids.clone(),
                log_inputs: log_inputs.iter().map(|(arg, curve)| (arg.to_string(), curve.to_string())).collect(),
                params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                output_set: None,
                input_set: None
            ,
                custody: test_run_custody(),
            };
            let results = run_workflow_module(&db, &req);
            for r in &results {
                println!("{module}: well={} rows={} outputs={:?} err={:?}", r.well_id, r.rows_written, r.output_curves, r.error);
                assert!(r.error.is_none(), "{module} failed: {:?}", r.error);
            }
        };

        run(
            "vsh_gr",
            &[("GR", "GRN_CS")],
            &[("GR_MA", 25.0), ("GR_SH", 130.0)],
            &[("OPT_GR", "LINEAR")],
        );
        run(
            "phi_dn",
            &[("NPHI", "NPHI_COR")],
            &[("RHO_MA", 2.645), ("RHO_SH", 2.5), ("NPHI_SH", 0.35), ("RHO_DSH", 2.65), ("PHIE_MAX", 0.35)],
            &[("OPT_XPLOT", "AVERAGE")],
        );
        run(
            "sw_indo",
            &[],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.2), ("RT_SH", 4.0)],
            &[("OPT_INDO", "FULL"), ("OPT_RW", "CONSTANT")],
        );
        run("perm_wyllie_rose", &[], &[("SWE_IRR", 0.15)], &[("OPT_WR", "TIMUR")]);

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
            &PaySummaryRequest { well_ids: well_ids.clone(), vsh_max: 0.5, phie_min: 0.1, swe_max: 0.6, perm_min: None, input_set: None, skip_version: false, stats_only: false ,
                custody: Some(test_run_custody()),
            },
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
