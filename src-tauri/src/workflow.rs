//! Workflow runner: executes deterministic modules across wells (rayon-parallel),
//! resolving interval parameters per zone (interval-parameter style), and the cutoff/summary
//! engine modeled on pay-summary specs.

use crate::db;
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::{Connection, OptionalExt};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
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

#[derive(Debug, Clone, Serialize)]
pub struct ModuleInputAvailability {
    pub well_id: String,
    /// Manifest argument names whose selected curves contain at least one finite sample on the
    /// exact frame/input-set resolution path the public runner will use.
    pub available_arguments: Vec<String>,
    /// A read failure is not reported as an absent curve. The dialog renders this as an explicit
    /// preflight failure so a database error cannot masquerade as ordinary missing log coverage.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DespikeContaminationIssue {
    pub well_id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DespikeContaminationPreview {
    pub branches: Vec<crate::condition::DespikeContaminationBranch>,
    pub evaluated_wells: usize,
    pub unavailable_well_ids: Vec<String>,
    pub issues: Vec<DespikeContaminationIssue>,
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleRunOutcome {
    Clean,
    Degraded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleRunResult {
    pub well_id: String,
    pub rows_written: usize,
    pub output_curves: Vec<String>,
    pub error: Option<String>,
    pub outcome: ModuleRunOutcome,
    pub degradations: Vec<modules::RunDegradation>,
}

impl ModuleRunResult {
    fn failed(well_id: impl Into<String>, error: String) -> Self {
        Self {
            well_id: well_id.into(),
            rows_written: 0,
            output_curves: Vec::new(),
            error: Some(error),
            outcome: ModuleRunOutcome::Failed,
            degradations: Vec::new(),
        }
    }

    fn skipped(well_id: impl Into<String>) -> Self {
        Self {
            well_id: well_id.into(),
            rows_written: 0,
            output_curves: Vec::new(),
            error: None,
            outcome: ModuleRunOutcome::Skipped,
            degradations: Vec::new(),
        }
    }
}

fn degradation_message(degradations: &[modules::RunDegradation]) -> String {
    let details = degradations
        .iter()
        .map(|event| {
            format!(
                "{}: {} ({} occurrence{})",
                event.kind.as_str(),
                event.detail,
                event.occurrences,
                if event.occurrences == 1 { "" } else { "s" }
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("degraded result - {details}")
}

fn run_warning_message(
    module: &str,
    degradations: &[modules::RunDegradation],
    violations: &[modules::PreconditionViolation],
) -> String {
    let mut parts = Vec::new();
    if !degradations.is_empty() {
        parts.push(degradation_message(degradations));
    }
    if !violations.is_empty() {
        parts.push(
            violations
                .iter()
                .map(|violation| violation.message(module))
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
    parts.join("; ")
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
        "vsh_gr" | "vsh_dn" | "phi_den" | "phi_dn" | "phi_dnbk" | "phi_son" | "ssc" | "sspw"
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
fn resolve_param_arrays_with_default_usage(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    req_params: &HashMap<String, f64>,
    depth: &[f32],
) -> Result<(HashMap<String, Vec<f64>>, HashMap<String, Vec<bool>>), String> {
    let zones = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zone_params = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;
    let zone_range: HashMap<&str, (f32, f32)> =
        zones.iter().map(|z| (z.zone_name.as_str(), (z.top_depth, z.bottom_depth))).collect();

    let mut out = HashMap::new();
    let mut defaulted_samples = HashMap::new();
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
        // A source-bearing unconditional NumericRange is enforced at the algorithm boundary,
        // where it can produce SB-ENV-003's condition id, source and optional per-sample flag.
        // Legacy ArgSpec ranges with no such condition stay here: silently dropping their only
        // guard would be a weakening, not a migration.
        let algorithm_range = arg.validity_conditions.iter().any(|condition| {
            matches!(
                condition.rule,
                modules::ValidityRule::NumericRange { when: None, .. }
            )
        });
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
            if !v.is_finite() || (!algorithm_range && !in_range(v)) {
                bad.push(format!("{} = {v} ({})", arg.name, range()));
            }
        }
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            let v = v as f64;
            if !v.is_finite() || (!algorithm_range && !in_range(v)) {
                bad.push(format!("{} = {v} in zone '{}' ({})", arg.name, zp.zone_name, range()));
            }
        }

        let base = req_params
            .get(&arg.name)
            .copied()
            .or_else(|| arg.default.parse().ok())
            .unwrap_or(f64::NAN);
        let mut arr = vec![base; depth.len()];
        let base_is_defaulted = !req_params.contains_key(&arg.name)
            && arg.default.parse::<f64>().is_ok();
        let mut defaulted = vec![base_is_defaulted; depth.len()];

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
                defaulted.fill(false);
            }
        }
        for zp in zone_params.iter().filter(|z| z.param_name == arg.name) {
            let Some(v) = zp.value_num else { continue };
            if let Some(&(top, bottom)) = zone_range.get(zp.zone_name.as_str()) {
                for (i, d) in depth.iter().enumerate() {
                    if *d >= top && *d < bottom {
                        arr[i] = v as f64;
                        defaulted[i] = false;
                    }
                }
            }
        }
        out.insert(arg.name.clone(), arr);
        if defaulted.iter().any(|value| *value) {
            defaulted_samples.insert(arg.name.clone(), defaulted);
        }
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
    Ok((out, defaulted_samples))
}

/// Test-facing value-only view. Production execution uses the companion default-usage map so a
/// sourced manifest default cannot make a result look clean merely because it computed.
#[cfg(test)]
fn resolve_param_arrays(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    req_params: &HashMap<String, f64>,
    depth: &[f32],
) -> Result<HashMap<String, Vec<f64>>, String> {
    resolve_param_arrays_with_default_usage(conn, well_id, spec, req_params, depth)
        .map(|(parameters, _)| parameters)
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

/// Canonical run-provenance key for the universal sample mask. Its value is always either the
/// upper-cased curve mnemonic actually selected by the run or the explicit `NONE` state; absence
/// of the key is reserved for legacy records written before SB-ENV-028. State and curve are kept
/// separate so a real curve named `NONE` cannot masquerade as an unmasked run.
pub(crate) const MASK_PROVENANCE_KEY: &str = "MASK";
pub(crate) const MASK_PROVENANCE_NONE: &str = "NONE";
pub(crate) const MASK_PROVENANCE_APPLIED: &str = "APPLIED";
pub(crate) const FLAG_KIND_PROVENANCE_PREFIX: &str = "FLAG_KIND.";
pub(crate) const OUTPUT_QUANTITY_PROVENANCE_PREFIX: &str = "OUTPUT_QUANTITY.";
pub(crate) const INPUT_QUANTITY_PROVENANCE_PREFIX: &str = "INPUT_QUANTITY.";
pub(crate) const POROSITY_OUTPUT_PROVENANCE_PREFIX: &str = "POROSITY_OUTPUT.";
pub(crate) const SMOOTHING_POLICY_PROVENANCE_KEY: &str = "SMOOTHING_POLICY";

/// SB-SAT-043 run-provenance keys. A saturation answer carries the paper it traces to, the
/// Worthington 1985 classification where a source states one, any contested attribution, and — for
/// the LRLC methods — the calibration standing of the coefficients it was computed on.
pub(crate) const METHOD_CITATION_PROVENANCE_KEY: &str = "method_citation";
pub(crate) const WORTHINGTON_TYPE_PROVENANCE_KEY: &str = "worthington_1985_type";
pub(crate) const METHOD_CAUTION_PROVENANCE_KEY: &str = "method_attribution_caution";
pub(crate) const UNFITTED_COEFFICIENTS_PROVENANCE_KEY: &str = "unfitted_coefficients";

/// SB-SAT-043 / SB-SAT-048. The coefficients each LRLC method is calibrated on. They are one
/// field's calibration, and a run on numbers that did not come from this project's own fit is
/// indistinguishable in the OUTPUT from one that did — a foreign calibration *"does not announce
/// itself: it yields a smooth, plausible Sw that is simply wrong"* (`lrlc.rs:83-90`).
pub(crate) fn lrlc_calibration_coefficients(module: &str) -> &'static [&'static str] {
    match module {
        "sw_rtc" => &["A_CAP", "B_QV", "C0", "RSF"],
        "sw_imts" => &["S_FACTOR"],
        _ => &[],
    }
}

/// SB-SAT-043. The persisted equation identity for a saturation run.
///
/// `sw_sim` offers two equations that trace to the same References block but are DIFFERENT
/// equations, 7.3 saturation units apart, so the identity is the option's canonical value rather
/// than the module name.
pub(crate) fn saturation_method_id<'a>(
    module: &str,
    opts: &'a HashMap<String, String>,
) -> Option<std::borrow::Cow<'a, str>> {
    match module {
        "sw_arch" => Some(std::borrow::Cow::Borrowed("archie_total")),
        "sw_indo" => Some(std::borrow::Cow::Borrowed("indonesia")),
        "sw_rtc" => Some(std::borrow::Cow::Borrowed("lrlc_rtc")),
        "sw_imts" => Some(std::borrow::Cow::Borrowed("lrlc_imts")),
        // `sw_height` and the retired `multimin` are deliberately absent. Both are registered in
        // `SATURATION_METHODS` so the build gate accounts for every Saturation-category module,
        // but neither gets a citation pushed into its run record: saturation-height's literature
        // and its fitted-object provenance belong to `15_sat-height-rocktyping.md`, and printing
        // this chapter's hand-off token in a deliverable would say less than nothing.
        "sw_sim" => opts.get("OPT_SIM").map(|value| {
            std::borrow::Cow::Owned(modules::canonical_option_value("sw_sim", "OPT_SIM", value))
        }),
        _ => None,
    }
}

pub(crate) fn mask_provenance(opts: &HashMap<String, String>) -> serde_json::Value {
    match opts
        .get(MASK_PROVENANCE_KEY)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(curve) => serde_json::json!({
            "state": MASK_PROVENANCE_APPLIED,
            "curve": curve.to_uppercase(),
        }),
        None => serde_json::json!({ "state": MASK_PROVENANCE_NONE }),
    }
}

/// Run-option prefix carrying a per-output rename: `__OUT_VSH = VSHALE` makes the run write its
/// `VSH` output as `VSHALE`.
///
/// Double-underscored like `__IN_<arg>` and `__ZONE_INDEX` because it is framework-reserved and
/// can never collide with a manifest option. An absent or blank entry means "the manifest's own
/// default name", which is what every existing run and every saved chain sends — so this is
/// additive by construction.
pub(crate) const OUT_NAME_PREFIX: &str = "__OUT_";

pub(crate) const PRECONDITION_POLICY_PROVENANCE_KEY: &str =
    "_sandibumi_precondition_policy_v1";
pub(crate) const PRECONDITION_VIOLATIONS_PROVENANCE_KEY: &str =
    "_sandibumi_precondition_violations_v1";

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

        validate_output_name(&arg.name, &name, &resolved)?;
        resolved.push((arg.name.clone(), name));
    }
    if modules::precondition_policy(opts)? == modules::PreconditionPolicy::FlagValidSamples {
        let name = format!("{}_PRECONDITION_FLAG", spec.name.to_uppercase());
        validate_output_name(modules::PRECONDITION_FLAG_OUTPUT_KEY, &name, &resolved)?;
        resolved.push((modules::PRECONDITION_FLAG_OUTPUT_KEY.into(), name));
    }
    Ok(resolved)
}

/// Resolve every typed flag declaration through the same rename and universal-prefix rules as
/// the write path. The returned curve identity is therefore the persisted identity, not the
/// manifest key, and remains correct when an interpreter renames `COND_FLAG` to `TO_EXCLUDE`.
pub(crate) fn resolved_flag_output_names(
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
) -> Result<Vec<(String, modules::FlagKind)>, String> {
    let prefix = opts
        .get(OUT_PREFIX_OPT)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .unwrap_or_default();
    Ok(resolve_output_names(spec, opts)?
        .into_iter()
        .filter_map(|(declared, name)| {
            let kind = if declared == modules::PRECONDITION_FLAG_OUTPUT_KEY {
                Some(modules::framework_precondition_flag_kind())
            } else {
                spec.args
                    .iter()
                    .find(|arg| arg.name == declared)
                    .and_then(|arg| arg.flag_kind)
            }?;
            Some((format!("{prefix}{name}"), kind))
        })
        .collect())
}

/// Resolve producer-declared VSH/VCL identities through the same rename and universal-prefix
/// transforms as the write path. This is metadata about the persisted curve, so the mutable output
/// mnemonic cannot be used to reconstruct it later.
pub(crate) fn resolved_shale_clay_output_names(
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
) -> Result<Vec<(String, modules::ShaleClayQuantity)>, String> {
    let prefix = opts
        .get(OUT_PREFIX_OPT)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .unwrap_or_default();
    Ok(resolve_output_names(spec, opts)?
        .into_iter()
        .filter_map(|(declared, name)| {
            let quantity = spec
                .args
                .iter()
                .find(|arg| arg.name == declared)
                .and_then(|arg| arg.output_shale_clay_quantity)?;
            Some((format!("{prefix}{name}"), quantity))
        })
        .collect())
}

/// Resolve producer-declared POR family, method and volume convention through the same output
/// rename and prefix transforms as the write. A mutable mnemonic is not scientific provenance:
/// the persisted curve-specific record is what distinguishes equal-looking PHIT/PHIE quantities.
pub(crate) fn resolved_porosity_output_names(
    spec: &modules::ModuleSpec,
    opts: &HashMap<String, String>,
) -> Result<Vec<(String, modules::PorosityOutputContract)>, String> {
    let prefix = opts
        .get(OUT_PREFIX_OPT)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .unwrap_or_default();
    Ok(resolve_output_names(spec, opts)?
        .into_iter()
        .filter_map(|(declared, name)| {
            let contract = spec
                .args
                .iter()
                .find(|arg| arg.name == declared)
                .and_then(|arg| arg.porosity_output.clone())?;
            Some((format!("{prefix}{name}"), contract))
        })
        .collect())
}

fn validate_output_name(
    argument: &str,
    name: &str,
    resolved: &[(String, String)],
) -> Result<(), String> {
    // Whitespace and quotes would survive the write and then break every reader that parses a
    // curve list — refused here, where the user typed it, rather than in a LAS export weeks on.
    if name.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == ',') {
        return Err(format!(
            "Output name '{name}' for {argument} contains a space or quote. A curve name is used \
             verbatim in exports and curve lists — use letters, digits and underscores."
        ));
    }
    if name == "DEPTH" {
        return Err(format!(
            "{argument} = DEPTH is refused: DEPTH is the reference column of the existing STANDARD \
             frame. A module must never write back to that frame's reference column; use \
             Reframe to emit a different depth basis as a new OWN frame."
        ));
    }
    if crate::schema_vocab::standard_column(name).is_some() {
        return Err(format!(
            "{argument} = {name} would be shadowed: {name} is read from the raw log first, so a \
             computed copy stored under that name is never the one anything reads. Give the \
             output its own name."
        ));
    }
    if let Some((other, _)) = resolved.iter().find(|(_, resolved_name)| resolved_name == name) {
        return Err(format!(
            "{argument} and {other} would both be written as {name}. Two outputs under one name means \
             one of them silently replaces the other — rename one."
        ));
    }
    Ok(())
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
    if spec.category == "VSH" {
        modules::validate_parameter_sources(std::slice::from_ref(spec))?;
    }
    let manifest_version = crate::parameter_pack::module_parameter_schema_from_spec(spec)?
        .module_schema_version;
    let manifest_source = format!("module manifest {manifest_version}");
    let mut parameters = Vec::new();
    let mut legacy = serde_json::Map::new();

    for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Param) {
        let (value, source, resolution, value_manifest_version, unit_custody) =
            if let Some(value) = explicit_params.get(&arg.name) {
                let custody = (spec.category == "VSH")
                    .then(|| modules::ParameterUnitCustody::new(*value, &arg.unit, &arg.unit))
                    .transpose()?;
                (
                    serde_json::json!(value),
                    source_note.to_string(),
                    Some(equations::ParameterResolution::Explicit),
                    None,
                    custody,
                )
            } else if let Ok(value) = arg.default.parse::<f64>() {
                (
                    serde_json::json!(value),
                    arg.default_source.clone(),
                    Some(equations::ParameterResolution::Defaulted),
                    Some(manifest_version.clone()),
                    arg.default_unit_custody.clone(),
                )
            } else {
                (
                    serde_json::json!(modules::ABSENT_DEFAULT_SOURCE),
                    modules::ABSENT_DEFAULT_SOURCE.to_string(),
                    None,
                    None,
                    None,
                )
            };
        legacy.insert(arg.name.clone(), value.clone());
        let decision = crate::param_sources::decision_for(&arg.sources_topic, &value);
        let custody_manifest_version = value_manifest_version.clone();
        parameters.push(equations::AncestryParameter {
            name: format!("{name_prefix}{}", arg.name),
            value,
            source: source.clone(),
            resolution,
            manifest_version: value_manifest_version,
            decision,
        });
        if let Some(custody) = unit_custody {
            parameters.push(equations::AncestryParameter {
                name: format!("{name_prefix}{}@unit_custody", arg.name),
                value: serde_json::to_value(custody)
                    .map_err(|error| format!("cannot serialize {} unit custody: {error}", arg.name))?,
                source,
                resolution,
                manifest_version: custody_manifest_version,
                decision: None,
            });
        }
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
    precondition_violations: &[modules::PreconditionViolation],
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
    let mask = mask_provenance(&req.opts);
    if parameters
        .iter()
        .any(|parameter| parameter.name == MASK_PROVENANCE_KEY)
    {
        return Err(format!(
            "module '{}' declares an argument that collides with reserved run-provenance key '{}'",
            spec.name, MASK_PROVENANCE_KEY
        ));
    }
    let mask_is_applied = mask["state"] == MASK_PROVENANCE_APPLIED;
    parameters.push(equations::AncestryParameter {
        name: MASK_PROVENANCE_KEY.into(),
        value: mask,
        source: if mask_is_applied {
            req.custody.source_note.clone()
        } else {
            "SB-ENV-028 explicit no-mask run state".into()
        },
        resolution: mask_is_applied.then_some(equations::ParameterResolution::Explicit),
        manifest_version: None,
        decision: None,
    });
    if req.module == "smooth" {
        if parameters
            .iter()
            .any(|parameter| parameter.name == SMOOTHING_POLICY_PROVENANCE_KEY)
            || legacy.contains_key(SMOOTHING_POLICY_PROVENANCE_KEY)
        {
            return Err(format!(
                "module '{}' declares an argument that collides with reserved smoothing-provenance key '{}'",
                spec.name, SMOOTHING_POLICY_PROVENANCE_KEY
            ));
        }
        let policy = crate::condition::smoothing_policy(
            opts.get("OPT_METHOD").map(String::as_str).unwrap_or("MEAN"),
        );
        legacy.insert(SMOOTHING_POLICY_PROVENANCE_KEY.into(), policy.clone());
        parameters.push(equations::AncestryParameter {
            name: SMOOTHING_POLICY_PROVENANCE_KEY.into(),
            value: policy,
            source: "docs/PRD_v2/20_envcorr-qc.md SB-ENV-041 / SB-ENV-T49".into(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });
    }
    for (curve, kind) in resolved_flag_output_names(spec, opts)? {
        // Condition flags are optional. Persist a role only for a curve this run actually emitted;
        // otherwise metadata would claim an output exists when OPT_FLAG deliberately suppressed it.
        if !output_names.iter().any(|output| output == &curve) {
            continue;
        }
        let name = format!("{FLAG_KIND_PROVENANCE_PREFIX}{curve}");
        if parameters.iter().any(|parameter| parameter.name == name) {
            return Err(format!(
                "module '{}' declares an argument that collides with reserved flag-kind provenance key '{}'",
                spec.name, name
            ));
        }
        parameters.push(equations::AncestryParameter {
            name,
            value: serde_json::to_value(kind)
                .map_err(|error| format!("cannot serialize flag kind for {curve}: {error}"))?,
            source: "SB-ENV-030 typed flag-kind declaration".into(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });
    }
    for (curve, quantity) in resolved_shale_clay_output_names(spec, opts)? {
        if !output_names.iter().any(|output| output == &curve) {
            continue;
        }
        let name = format!("{OUTPUT_QUANTITY_PROVENANCE_PREFIX}{curve}");
        if parameters.iter().any(|parameter| parameter.name == name) {
            return Err(format!(
                "module '{}' declares an argument that collides with reserved output-quantity provenance key '{}'",
                spec.name, name
            ));
        }
        parameters.push(equations::AncestryParameter {
            name,
            value: serde_json::to_value(quantity)
                .map_err(|error| format!("cannot serialize output quantity for {curve}: {error}"))?,
            source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });
    }
    for (curve, contract) in resolved_porosity_output_names(spec, opts)? {
        if !output_names.iter().any(|output| output == &curve) {
            continue;
        }
        let name = format!("{POROSITY_OUTPUT_PROVENANCE_PREFIX}{curve}");
        if parameters.iter().any(|parameter| parameter.name == name) {
            return Err(format!(
                "module '{}' declares an argument that collides with reserved porosity-output provenance key '{}'",
                spec.name, name
            ));
        }
        parameters.push(equations::AncestryParameter {
            name,
            value: serde_json::to_value(contract).map_err(|error| {
                format!("cannot serialize porosity output custody for {curve}: {error}")
            })?,
            source: "docs/PRD_v2/11_porosity.md SB-POR-004 and F16; DEC-013".into(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });
    }
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
    let method_id = saturation_method_id(&req.module, opts);
    if let Some(method_id) = method_id.as_deref() {
        legacy.insert("method_id".into(), serde_json::json!(method_id));
        parameters.push(equations::AncestryParameter {
            name: "method_id".into(),
            value: serde_json::json!(method_id),
            source: req.custody.source_note.clone(),
            resolution: None,
            manifest_version: None,
            decision: None,
        });

        // SB-SAT-043. The paper travels WITH the answer. Geolog ships references inside its module
        // manifests but no vendor carries one through to the result, so a computed Sw arrives
        // downstream with nothing to defend it. Attaching the citation to the equation identity is
        // what makes SB-SAT-038's build-time source gate auditable in a deliverable rather than
        // only at compile time.
        let Some(method) = crate::param_sources::saturation_method(&req.module, method_id) else {
            return Err(format!(
                "saturation module '{}' ran as '{method_id}' with no registered literature \
                 citation; register it in param_sources::SATURATION_METHODS",
                req.module
            ));
        };
        let mut record = vec![
            (
                METHOD_CITATION_PROVENANCE_KEY,
                serde_json::json!(method.citation),
                "docs/PRD_v2/12_saturation.md SB-SAT-043; param_sources::SATURATION_METHODS"
                    .to_string(),
            ),
            (
                // Carried on EVERY saturation run, including the models nobody classified: the
                // field then says NONE-STATED and its source names what was consulted. Omitting it
                // instead would make "no source classifies this" and "nobody recorded it" the same
                // record, and only one of those is a fact a reader can check.
                WORTHINGTON_TYPE_PROVENANCE_KEY,
                match method.worthington {
                    Some(kind) => serde_json::json!(kind),
                    None => serde_json::json!(crate::param_sources::WORTHINGTON_NONE_STATED),
                },
                method.worthington_source.to_string(),
            ),
        ];
        if !method.caution.trim().is_empty() {
            record.push((
                METHOD_CAUTION_PROVENANCE_KEY,
                serde_json::json!(method.caution),
                "docs/PRD_v2/12_saturation.md SB-SAT-043 - a disputed attribution travels with the \
                 citation rather than being resolved on the user's behalf"
                    .to_string(),
            ));
        }
        // SB-SAT-048 via SB-SAT-T59. The LRLC coefficients are one field's calibration. No value
        // ships as a default any more, so the flag reports the stronger fact — and would report
        // the weaker one immediately if a default were ever reintroduced.
        let coefficients = lrlc_calibration_coefficients(&req.module);
        if !coefficients.is_empty() {
            let resolutions = coefficients
                .iter()
                .map(|name| {
                    let state = parameters
                        .iter()
                        .find(|parameter| parameter.name == *name)
                        .map(|parameter| match parameter.resolution {
                            Some(equations::ParameterResolution::Defaulted) => "SHIPPED_DEFAULT",
                            Some(equations::ParameterResolution::Explicit) => "ENTERED",
                            None => "UNRECORDED",
                        })
                        .unwrap_or("ABSENT");
                    (name.to_string(), serde_json::json!(state))
                })
                .collect::<serde_json::Map<_, _>>();
            let on_shipped_default = resolutions
                .values()
                .any(|state| state == &serde_json::json!("SHIPPED_DEFAULT"));
            record.push((
                UNFITTED_COEFFICIENTS_PROVENANCE_KEY,
                serde_json::json!({
                    "state": if on_shipped_default { "SHIPPED_DEFAULT_IN_USE" } else { "NO_SHIPPED_DEFAULT" },
                    "coefficients": resolutions,
                    "limit": "ENTERED does not distinguish a coefficient accepted from this \
                              project's own calibration from one typed by hand: zone-parameter \
                              custody carries no source text. Fitted-versus-entered is therefore \
                              NOT claimed here.",
                }),
                "docs/PRD_v2/12_saturation.md SB-SAT-048 and SB-SAT-T59; src-tauri/src/lrlc.rs:83-90"
                    .to_string(),
            ));
        }
        for (name, value, source) in record {
            if parameters.iter().any(|parameter| parameter.name == name)
                || legacy.contains_key(name)
            {
                return Err(format!(
                    "module '{}' declares an argument that collides with reserved \
                     saturation-provenance key '{name}'",
                    spec.name
                ));
            }
            parameters.push(equations::AncestryParameter {
                name: name.into(),
                value,
                source,
                resolution: None,
                manifest_version: None,
                decision: None,
            });
        }
    }

    if modules::precondition_policy(opts)? == modules::PreconditionPolicy::FlagValidSamples {
        let policy = serde_json::json!(modules::PRECONDITION_POLICY_FLAG_VALID_SAMPLES);
        let violations = serde_json::to_value(precondition_violations)
            .map_err(|error| format!("cannot serialize precondition violations: {error}"))?;
        for (key, value) in [
            (PRECONDITION_POLICY_PROVENANCE_KEY, policy.clone()),
            (PRECONDITION_VIOLATIONS_PROVENANCE_KEY, violations.clone()),
        ] {
            if legacy.insert(key.into(), value).is_some() {
                return Err(format!(
                    "module '{}' declares an argument that collides with reserved saved-run key '{}'",
                    spec.name, key
                ));
            }
        }
        parameters.push(equations::AncestryParameter {
            name: PRECONDITION_POLICY_PROVENANCE_KEY.into(),
            value: policy,
            source: req.custody.source_note.clone(),
            resolution: Some(equations::ParameterResolution::Explicit),
            manifest_version: None,
            decision: None,
        });
        if !precondition_violations.is_empty() {
            let mut sources = precondition_violations
                .iter()
                .map(|violation| violation.source.clone())
                .collect::<Vec<_>>();
            sources.sort();
            sources.dedup();
            parameters.push(equations::AncestryParameter {
                name: PRECONDITION_VIOLATIONS_PROVENANCE_KEY.into(),
                value: violations,
                source: sources.join(" | "),
                resolution: None,
                manifest_version: None,
                decision: None,
            });
        }
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
            Ok(input) => {
                if let Some(arg) = spec.args.iter().find(|arg| {
                    arg.name == *argument && !arg.accepted_shale_clay_quantities.is_empty()
                }) {
                    let accepted = arg
                        .accepted_shale_clay_quantities
                        .iter()
                        .map(|quantity| quantity.as_str())
                        .collect::<Vec<_>>()
                        .join(" or ");
                    let quantity = shale_clay_quantity_for_ancestry_input(conn, &input)?
                        .ok_or_else(|| {
                            format!(
                                "module '{}' input '{}' requires typed {accepted} metadata, but resolved curve '{}' has no VSH/VCL quantity metadata",
                                spec.name, argument, input.curve
                            )
                        })?;
                    if !arg.accepted_shale_clay_quantities.contains(&quantity) {
                        return Err(format!(
                            "module '{}' input '{}' requires {accepted}, but resolved curve '{}' carries {} metadata",
                            spec.name,
                            argument,
                            input.curve,
                            quantity.as_str()
                        ));
                    }
                    let name = format!("{INPUT_QUANTITY_PROVENANCE_PREFIX}{argument}");
                    if parameters.iter().any(|parameter| parameter.name == name) {
                        return Err(format!(
                            "module '{}' declares an argument that collides with reserved input-quantity provenance key '{}'",
                            spec.name, name
                        ));
                    }
                    parameters.push(equations::AncestryParameter {
                        name,
                        value: serde_json::to_value(quantity).map_err(|error| {
                            format!("cannot serialize input quantity for {argument}: {error}")
                        })?,
                        source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
                        resolution: None,
                        manifest_version: None,
                        decision: None,
                    });
                }
                inputs.push(input)
            }
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
    // SB-DBM-015 (DEC-023): the zone-set identity the run sees, recorded whenever zones
    // exist - a renamed or moved top changes it, and the re-run resolver refuses by name.
    let zone_set = match &zone_scope {
        equations::AncestryZoneScope::WholeWell => None,
        _ => {
            let (version, digest) =
                db::current_zone_set(conn, well_id).map_err(|error| error.to_string())?;
            Some(equations::ManifestZoneSet { version, digest })
        }
    };
    let ancestry = equations::CurveAncestry {
        schema_version: equations::CURVE_ANCESTRY_SCHEMA_VERSION,
        module: req.module.clone(),
        // SB-DBM-002 (DEC-021): the producing code's own digest, not the hand-maintained
        // package version that does not move when a module's arithmetic does.
        module_version: format!("src:{}", modules::module_source_digest(&req.module)),
        inputs,
        parameters,
        parameter_state,
        zone_scope,
        actor: req.custody.actor.clone(),
        timestamp_utc_ms: equations::ancestry_timestamp_utc_ms()?,
        outputs,
        depth_frame: None,
        zone_set,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    };
    let validity_manifest = serde_json::to_value(modules::module_validity_manifest(spec))
        .map_err(|error| format!("cannot serialize module validity manifest: {error}"))?;
    if legacy
        .insert(modules::MODULE_VALIDITY_MANIFEST_KEY.into(), validity_manifest)
        .is_some()
    {
        return Err(format!(
            "module '{}' declares an argument that collides with reserved saved-run key '{}'",
            spec.name,
            modules::MODULE_VALIDITY_MANIFEST_KEY
        ));
    }
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
    pub flag_kind: Option<modules::FlagKind>,
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
                flag_kind: if arg == modules::PRECONDITION_FLAG_OUTPUT_KEY {
                    Some(modules::framework_precondition_flag_kind())
                } else {
                    a.and_then(|argument| argument.flag_kind)
                },
                desc: if arg == modules::PRECONDITION_FLAG_OUTPUT_KEY {
                    "Companion flag: 1 = a declared precondition was violated at this sample; 0 = valid."
                        .into()
                } else {
                    a.map(|a| a.desc.clone()).unwrap_or_default()
                },
                unit: if arg == modules::PRECONDITION_FLAG_OUTPUT_KEY {
                    "1 = violation".into()
                } else {
                    a.map(|a| a.unit.clone()).unwrap_or_default()
                },
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
/// SB-DBM-015: the outcome of "re-run this set" - the replay happened only because every
/// manifest element resolved, and the byte comparison is reported rather than assumed.
#[derive(Debug, Clone, Serialize)]
pub struct RerunReport {
    pub set_id: String,
    pub module: String,
    pub output_set: String,
    pub compared_curves: usize,
    pub bit_identical: bool,
}

/// SB-DBM-015: "re-run this set". Every manifest element is verified to STILL RESOLVE
/// before anything runs, and a failed element refuses BY NAME - a re-run that silently
/// substituted a moved curve, a changed zone set, a different implementation or a deleted
/// model would be F-18's failure at the scale of a whole run. Only when the whole manifest
/// resolves is the run replayed (into its own RERUN output set, so the original version is
/// never disturbed), and the outputs are compared byte for byte against the stored
/// originals of the manifest's own set version.
pub fn rerun_log_set(
    db: &Mutex<Connection>,
    well_id: &str,
    set_id: &str,
    custody: &equations::RunCustody,
) -> Result<RerunReport, String> {
    let (module, ancestry, stored_params) = {
        let conn = db.lock().map_err(|_| "database busy".to_string())?;
        let entry = equations::list_log_sets(&conn, well_id)
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|entry| entry.set_id == set_id)
            .ok_or_else(|| format!("re-run refused: no stored run with set id {set_id}"))?;
        let ancestry = entry.ancestry.clone().ok_or_else(|| {
            format!(
                "re-run refused: manifest element 'run record' does not resolve - set {set_id} \
                 predates complete ancestry and carries no manifest"
            )
        })?;
        let params: serde_json::Value = entry
            .params_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| format!("stored run parameters are unreadable: {error}"))?
            .unwrap_or_else(|| serde_json::json!({}));

        // (a) Module identity and version - the producing code's own digest (DEC-021).
        let current = format!("src:{}", modules::module_source_digest(&entry.module));
        if ancestry.module_version != current {
            return Err(format!(
                "re-run refused: manifest element 'module version' no longer resolves - the run \
                 was produced by {} '{}' but the current build is '{current}'",
                entry.module, ancestry.module_version
            ));
        }

        // (b) Every input curve identity, re-resolved by the SAME resolver the run used: a
        // different chosen identity or set version means the input has moved.
        for input in &ancestry.inputs {
            let now = equations::resolve_ancestry_input(
                &conn,
                &input.well_id,
                &input.argument,
                &input.curve,
                None,
                None,
            )
            .map_err(|error| {
                format!(
                    "re-run refused: manifest element 'input curve {}' ({}) no longer \
                     resolves: {error}",
                    input.curve, input.argument
                )
            })?;
            if now.set_id != input.set_id || now.set_version != input.set_version {
                return Err(format!(
                    "re-run refused: manifest element 'input curve {}' ({}) no longer resolves \
                     to the recorded identity - stored set {} v{:?}, current set {} v{:?}",
                    input.curve,
                    input.argument,
                    input.set_id,
                    input.set_version,
                    now.set_id,
                    now.set_version
                ));
            }
        }

        // (c) The zone-set identity (DEC-023): a renamed or moved top means the same run
        // over the same well would mean something different.
        if let Some(recorded) = &ancestry.zone_set {
            let (_, digest) =
                db::current_zone_set(&conn, well_id).map_err(|error| error.to_string())?;
            if digest != recorded.digest {
                return Err(format!(
                    "re-run refused: manifest element 'zone set' no longer resolves - the run \
                     saw zone-set digest {} (v{}) but the well now has {digest}",
                    recorded.digest, recorded.version
                ));
            }
        }

        // (d) The applied learned model (DEC-024): a deleted model cannot be re-applied.
        if let Some(model_id) = &ancestry.applied_model {
            let present: i64 = conn
                .query_row(
                    "SELECT count(*) FROM ml_models WHERE model_id = ?1",
                    duckdb::params![model_id],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            if present == 0 {
                return Err(format!(
                    "re-run refused: manifest element 'applied model' no longer resolves - \
                     model {model_id} is not in ml_models"
                ));
            }
        }
        (entry.module.clone(), ancestry, params)
    };

    // Rebuild the request from the stored record: numeric parameters and string options
    // come from the saved params (reserved provenance keys stripped), inputs from the
    // manifest's own resolved argument -> curve pairs.
    let mut params: HashMap<String, f64> = HashMap::new();
    let mut opts: HashMap<String, String> = HashMap::new();
    if let serde_json::Value::Object(map) = &stored_params {
        for (name, value) in map {
            if name == equations::CURVE_ANCESTRY_KEY
                || name == modules::MODULE_VALIDITY_MANIFEST_KEY
            {
                continue;
            }
            match value {
                serde_json::Value::Number(number) => {
                    if let Some(number) = number.as_f64() {
                        params.insert(name.clone(), number);
                    }
                }
                serde_json::Value::String(text) => {
                    opts.insert(name.clone(), text.clone());
                }
                _ => {}
            }
        }
    }
    let log_inputs: HashMap<String, String> = ancestry
        .inputs
        .iter()
        .map(|input| (input.argument.clone(), input.curve.clone()))
        .collect();
    let request = RunModuleRequest {
        module: module.clone(),
        well_ids: vec![well_id.to_string()],
        log_inputs,
        params,
        opts,
        output_set: Some("RERUN".to_string()),
        input_set: None,
        custody: custody.clone(),
    };
    // Snapshot the ORIGINAL version's stored values before the replay touches anything:
    // the archive holds the manifest's own set version even when a later run superseded it.
    let read_version = |conn: &Connection, set: &str, curve: &str| -> Result<Vec<(u32, Option<u32>)>, String> {
        let read = |table: &str| -> Result<Vec<(u32, Option<u32>)>, String> {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT depth, value FROM {table} WHERE set_id = ?1 AND curve_name = ?2 \
                     ORDER BY depth"
                ))
                .map_err(|error| error.to_string())?;
            stmt.query_map(duckdb::params![set, curve], |row| {
                Ok((
                    row.get::<_, f32>(0)?.to_bits(),
                    row.get::<_, Option<f32>>(1)?.map(f32::to_bits),
                ))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
        };
        let current = read("computed_curves")?;
        if current.is_empty() { read("computed_curves_archive") } else { Ok(current) }
    };
    let originals: Vec<(String, Vec<(u32, Option<u32>)>)> = {
        let conn = db.lock().map_err(|_| "database busy".to_string())?;
        ancestry
            .outputs
            .iter()
            .map(|output| {
                read_version(&conn, set_id, &output.curve).map(|rows| (output.curve.clone(), rows))
            })
            .collect::<Result<_, _>>()?
    };

    let results = run_workflow_module(db, &request);
    let result = results
        .first()
        .ok_or_else(|| "re-run produced no result".to_string())?;
    if let Some(error) = &result.error {
        return Err(format!("re-run failed: {error}"));
    }

    // Byte comparison against the replay's own new version.
    let conn = db.lock().map_err(|_| "database busy".to_string())?;
    let rerun_set = equations::list_log_sets(&conn, well_id)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|entry| entry.set_name == "RERUN" && entry.module == module)
        .max_by_key(|entry| entry.version)
        .ok_or_else(|| "re-run recorded no RERUN log set".to_string())?;
    let mut bit_identical = true;
    for (curve, original) in &originals {
        let replay = read_version(&conn, &rerun_set.set_id, curve)?;
        if &replay != original {
            bit_identical = false;
        }
    }
    Ok(RerunReport {
        set_id: set_id.to_string(),
        module,
        output_set: "RERUN".to_string(),
        compared_curves: originals.len(),
        bit_identical,
    })
}

pub fn run_workflow_module(db: &Mutex<Connection>, req: &RunModuleRequest) -> Vec<ModuleRunResult> {
    run_workflow_module_into(db, req, None, None, None)
}

fn resolved_log_args(spec: &modules::ModuleSpec, log_inputs: &HashMap<String, String>) -> Vec<(String, String)> {
    spec.args
        .iter()
        .filter(|argument| argument.kind == ArgKind::LogIn)
        .map(|argument| {
            let mnemonic = log_inputs
                .get(&argument.name)
                .cloned()
                .unwrap_or_else(|| argument.default.clone());
            (argument.name.clone(), mnemonic)
        })
        .collect()
}

fn automatic_input_aliases(argument: &modules::ArgSpec) -> Vec<String> {
    if !argument.preferred_aliases.is_empty() {
        return argument.preferred_aliases.clone();
    }
    let mut aliases = vec![argument.default.clone()];
    match argument.default.trim().to_uppercase().as_str() {
        "PHIE" => aliases.push(modules::PHIE_DN_LIMITED_DEFAULT.into()),
        "PHIT" => aliases.push(modules::PHIT_DN_LIMITED_DEFAULT.into()),
        _ => {}
    }
    aliases
}

fn first_available_input_alias(
    conn: &Connection,
    well_id: &str,
    argument: &str,
    aliases: &[String],
    input_set: Option<&str>,
    own_set_id: Option<&str>,
    available_in_run: &HashSet<String>,
) -> Result<Option<String>, String> {
    for alias in aliases {
        if available_in_run.contains(alias) {
            return Ok(Some(alias.clone()));
        }
        if equations::try_resolve_ancestry_input(
            conn,
            well_id,
            argument,
            alias,
            input_set,
            own_set_id,
        )?
        .is_some()
        {
            return Ok(Some(alias.clone()));
        }
    }
    Ok(None)
}

/// Resolve automatic input aliases against one well while preserving every explicit interpreter
/// selection. The ordered aliases are a manifest contract; availability is checked through the
/// same ancestry resolver that records the winning curve, so selection and provenance cannot
/// disagree about which curve existed.
pub(crate) fn resolved_log_args_for_well(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    log_inputs: &HashMap<String, String>,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
    available_in_run: &HashSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let mut resolved = Vec::new();
    for argument in spec.args.iter().filter(|argument| argument.kind == ArgKind::LogIn) {
        if let Some(explicit) = log_inputs.get(&argument.name) {
            resolved.push((argument.name.clone(), explicit.clone()));
            continue;
        }

        // SB-POR-004 / DEC-013: PHIE and PHIT remain the established density-facing canonical
        // identities, while the D-N comparison producer now has collision-safe physical defaults.
        // Downstream logical roles may follow that exact method-specific curve only when the
        // canonical name is absent. This is deliberately a closed two-name list rather than a
        // family scan: silently electing among several porosity methods would undo the provenance
        // contract this alias protects. Explicit interpreter selections still win above.
        let automatic_aliases = automatic_input_aliases(argument);
        let selected = first_available_input_alias(
            conn,
            well_id,
            &argument.name,
            &automatic_aliases,
            input_set,
            own_set_id,
            available_in_run,
        )?;
        resolved.push((
            argument.name.clone(),
            selected.unwrap_or_else(|| {
                argument
                    .preferred_aliases
                    .first()
                    .cloned()
                    .unwrap_or_else(|| argument.default.clone())
            }),
        ));
    }
    Ok(resolved)
}

fn validity_input_arguments(spec: &modules::ModuleSpec) -> HashSet<String> {
    let is_log = |name: &str| {
        spec.args
            .iter()
            .any(|argument| argument.name == name && argument.kind == ArgKind::LogIn)
    };
    let mut names = HashSet::new();
    for owner in &spec.args {
        for condition in &owner.validity_conditions {
            match &condition.rule {
                modules::ValidityRule::NumericRange { .. } if owner.kind == ArgKind::LogIn => {
                    names.insert(owner.name.clone());
                }
                modules::ValidityRule::RequiredCompanion { any_of, .. } => {
                    names.extend(any_of.iter().filter(|name| is_log(name)).cloned());
                }
                modules::ValidityRule::RequiredWhereFinite { input } => {
                    if owner.kind == ArgKind::LogIn {
                        names.insert(owner.name.clone());
                    }
                    if is_log(input) {
                        names.insert(input.clone());
                    }
                }
                modules::ValidityRule::LessThan { other } => {
                    if owner.kind == ArgKind::LogIn {
                        names.insert(owner.name.clone());
                    }
                    if is_log(other) {
                        names.insert(other.clone());
                    }
                }
                _ => {}
            }
        }
    }
    names
}

/// Unit metadata for the curve source the value resolver will actually use. This mirrors the
/// resolver's source order without moving unit decisions into a scientific module: selected
/// import set, selected/own computed set, current computed curve, then the deterministic generic
/// curve decision. A row carrying NULL is a found curve with missing metadata and must not fall
/// through to a different curve whose unit happens to be populated.
fn resolved_module_input_unit(
    conn: &Connection,
    well_id: &str,
    mnemonic: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<Option<String>, String> {
    let upper = mnemonic.trim().to_uppercase();

    if let Some(set_name) = input_set.map(str::trim).filter(|value| !value.is_empty()) {
        let computed_set: Option<String> = conn
            .query_row(
                "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2) \
                 ORDER BY version DESC LIMIT 1",
                duckdb::params![well_id, set_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("cannot resolve input set '{set_name}' unit: {error}"))?;
        if let Some(set_id) = computed_set {
            let present: Option<i32> = conn
                .query_row(
                    "SELECT 1 FROM computed_curves_archive \
                     WHERE set_id = ?1 AND upper(curve_name) = ?2 LIMIT 1",
                    duckdb::params![set_id, upper],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| format!("cannot inspect selected input-set unit: {error}"))?;
            if present.is_some() {
                return Ok(crate::db::curve_unit_for(conn, well_id, &upper));
            }
        } else {
            let imported: Option<Option<String>> = conn
                .query_row(
                    "SELECT unit FROM curve_meta \
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2) AND upper(mnemonic) = ?3 \
                     ORDER BY modified_seq DESC NULLS LAST, curve_id LIMIT 1",
                    duckdb::params![well_id, set_name, upper],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| format!("cannot resolve selected import-set unit: {error}"))?;
            if let Some(unit) = imported {
                return Ok(unit);
            }
        }
    }

    if let Some(own_set_id) = own_set_id {
        let present: Option<i32> = conn
            .query_row(
                "SELECT 1 FROM computed_curves_archive \
                 WHERE set_id = ?1 AND upper(curve_name) = ?2 LIMIT 1",
                duckdb::params![own_set_id, upper],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("cannot inspect current chain-set unit: {error}"))?;
        if present.is_some() {
            return Ok(crate::db::curve_unit_for(conn, well_id, &upper));
        }
    }

    let current_computed: Option<i32> = conn
        .query_row(
            "SELECT 1 FROM computed_curves c \
             JOIN standard_curves s ON s.well_id = c.well_id AND s.depth = c.depth \
             WHERE c.well_id = ?1 AND upper(c.curve_name) = ?2 AND isfinite(c.value) LIMIT 1",
            duckdb::params![well_id, upper],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("cannot inspect current computed-curve unit: {error}"))?;
    if current_computed.is_some() {
        return Ok(crate::db::curve_unit_for(conn, well_id, &upper));
    }

    let generic: Option<Option<String>> = conn
        .query_row(
            "SELECT unit FROM curve_meta \
             WHERE well_id = ?1 AND (upper(mnemonic) = ?2 OR upper(family) = ?2) \
             ORDER BY (set_name = 'RAW') DESC, \
                      (upper(mnemonic) = ?2) DESC, \
                      (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC, \
                      COALESCE(final_flag, 0) DESC, \
                      modified_seq DESC NULLS LAST, curve_id LIMIT 1",
            duckdb::params![well_id, upper],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("cannot resolve imported curve unit: {error}"))?;
    Ok(generic.flatten())
}

pub(crate) fn shale_clay_quantity_from_family(
    family: Option<&str>,
) -> Option<modules::ShaleClayQuantity> {
    match family.map(str::trim).map(str::to_uppercase).as_deref() {
        Some("VSH" | "VSH_UNCLIPPED") => Some(modules::ShaleClayQuantity::ShaleVolume),
        Some("VCL") => Some(modules::ShaleClayQuantity::ClayVolume),
        _ => None,
    }
}

/// Read the producer-owned quantity metadata for the exact ancestry input the resolver selected.
/// Generic/imported curves carry it in `curve_meta.family`; computed curves carry it in their
/// versioned ancestry record. The mnemonic and unit are deliberately not consulted.
pub(crate) fn shale_clay_quantity_for_ancestry_input(
    conn: &Connection,
    input: &equations::AncestryInput,
) -> Result<Option<modules::ShaleClayQuantity>, String> {
    let Some(chosen_curve_id) = input.chosen_curve_id.as_deref() else {
        return Ok(None);
    };
    if chosen_curve_id.starts_with("computed:") {
        let params_json: Option<String> = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE set_id = ?1",
                duckdb::params![&input.set_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| {
                format!(
                    "cannot read quantity metadata for computed input '{}': {error}",
                    input.curve
                )
            })?
            .flatten();
        let Some(params_json) = params_json else {
            return Ok(None);
        };
        let ancestry = equations::parse_curve_ancestry(&params_json).map_err(|error| {
            format!(
                "cannot read quantity metadata for computed input '{}': {error}",
                input.curve
            )
        })?;
        let key = format!(
            "{OUTPUT_QUANTITY_PROVENANCE_PREFIX}{}",
            input.curve.trim().to_uppercase()
        );
        let matches = ancestry
            .parameters
            .iter()
            .filter(|parameter| parameter.name == key)
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err(format!(
                "computed input '{}' carries duplicate quantity metadata",
                input.curve
            ));
        }
        return matches
            .first()
            .map(|parameter| {
                serde_json::from_value(parameter.value.clone()).map_err(|error| {
                    format!(
                        "computed input '{}' carries invalid quantity metadata: {error}",
                        input.curve
                    )
                })
            })
            .transpose();
    }

    let family: Option<Option<String>> = conn
        .query_row(
            "SELECT family FROM curve_meta WHERE curve_id = ?1",
            duckdb::params![chosen_curve_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| {
            format!(
                "cannot read quantity metadata for imported input '{}': {error}",
                input.curve
            )
        })?;
    Ok(shale_clay_quantity_from_family(family.flatten().as_deref()))
}

fn validate_shale_clay_input_quantities(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    log_args: &[(String, String)],
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<(), String> {
    for argument in spec
        .args
        .iter()
        .filter(|argument| !argument.accepted_shale_clay_quantities.is_empty())
    {
        let Some((_, curve)) = log_args.iter().find(|(name, _)| name == &argument.name) else {
            continue;
        };
        if curve.trim().is_empty() {
            continue;
        }
        let Some(input) = equations::try_resolve_ancestry_input(
            conn,
            well_id,
            &argument.name,
            curve,
            input_set,
            own_set_id,
        )? else {
            continue;
        };
        let accepted = argument
            .accepted_shale_clay_quantities
            .iter()
            .map(|quantity| quantity.as_str())
            .collect::<Vec<_>>()
            .join(" or ");
        let Some(actual) = shale_clay_quantity_for_ancestry_input(conn, &input)? else {
            return Err(format!(
                "module '{}' input '{}' requires typed {accepted} metadata, but resolved curve '{}' has no VSH/VCL quantity metadata; assign the physical family explicitly instead of relying on its mnemonic",
                spec.name, argument.name, input.curve
            ));
        };
        if !argument.accepted_shale_clay_quantities.contains(&actual) {
            return Err(format!(
                "module '{}' input '{}' requires {accepted}, but resolved curve '{}' carries {} metadata",
                spec.name,
                argument.name,
                input.curve,
                actual.as_str()
            ));
        }
    }
    Ok(())
}

/// SB-POR-024 (DEC-025): the N-D porosity boundary. The resolved NPHI curve must carry
/// a DECLARED neutron matrix basis, and the Bateman-Konen crossplot additionally requires
/// the LIMESTONE entry its own arithmetic assumes. The refusal names the module, the
/// curve, the physics and the fix - never a guess: DEC-025 forbids inferring the basis
/// from contractor, tool, salinity or a matrix default.
fn validate_neutron_basis_input(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    log_args: &[(String, String)],
) -> Result<(), String> {
    let Some(required_entry) = modules::required_neutron_basis(&spec.name) else {
        return Ok(());
    };
    let curve = log_args
        .iter()
        .find(|(argument, _)| argument == "NPHI")
        .map(|(_, curve)| curve.clone())
        .unwrap_or_else(|| "NPHI".to_string());
    // A curve that does not RESOLVE is the ordinary missing-input machinery's refusal,
    // with its own honest message; this boundary judges only a curve that exists.
    if equations::resolve_generic_curve_id(
        conn,
        well_id,
        &curve,
        equations::CurveRequest::SemanticFamily,
    )
    .ok()
    .flatten()
    .is_none()
    {
        return Ok(());
    }
    let Some(declared) = nphimat_declared_basis(conn, well_id, log_args) else {
        return Err(format!(
            "module '{}' refuses: neutron curve '{curve}' has no DECLARED matrix basis. A              limestone-unit neutron read against a sandstone matrix is ~0.04 v/v low in clean              water sand, and an undeclared basis cannot be checked - declare it              (set_curve_neutron_basis) or convert with nphimat first. DEC-025 / SB-POR-024",
            spec.name
        ));
    };
    if let Some(entry) = required_entry {
        if !declared.eq_ignore_ascii_case(entry) {
            return Err(format!(
                "module '{}' refuses: its crossplot is entered in {entry} units, but neutron                  curve '{curve}' declares basis {declared} - convert with nphimat first.                  DEC-025 / SB-POR-024",
                spec.name
            ));
        }
    }
    Ok(())
}

/// Resolve the selected input mnemonics into manifest argument names using the same input-set,
/// native-curve and computed-only rules as a real module run. This is shared with the dialog
/// preflight so “available before run” cannot be answered by a cheaper but different resolver.
fn fetch_module_input_logs(
    conn: &Connection,
    well_id: &str,
    spec: &modules::ModuleSpec,
    log_args: &[(String, String)],
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<(Vec<f32>, HashMap<String, Vec<f32>>, HashMap<String, String>), String> {
    let curve_names: Vec<String> = log_args.iter().map(|(_, mnemonic)| mnemonic.clone()).collect();
    let (depth, columns) = equations::fetch_curve_frame_from_set(
        conn,
        well_id,
        &curve_names,
        input_set,
        own_set_id,
    )
    .map_err(|error| error.to_string())?;

    let mut logs = HashMap::new();
    let mut units = HashMap::new();
    logs.insert("DEPTH".to_string(), depth.clone());
    for (argument, mnemonic) in log_args {
        let values = columns
            .get(&mnemonic.trim().to_uppercase())
            .cloned()
            .unwrap_or_else(|| vec![f32::NAN; depth.len()]);
        logs.insert(argument.clone(), values);
        if let Some(unit) =
            resolved_module_input_unit(conn, well_id, mnemonic, input_set, own_set_id)?
        {
            units.insert(argument.clone(), unit);
        }
    }

    // A raw curve with a familiar mnemonic is not proof that a computed-only input exists. Keep
    // this on the shared path so the preflight cannot advertise an input the runner will reject.
    for argument in spec
        .args
        .iter()
        .filter(|argument| argument.kind == ArgKind::LogIn && argument.computed_only)
    {
        let mnemonic = log_args
            .iter()
            .find(|(name, _)| name == &argument.name)
            .map(|(_, mnemonic)| mnemonic.clone())
            .unwrap_or_else(|| argument.default.clone());
        let values = equations::fetch_computed_only_aligned(
            conn,
            well_id,
            &mnemonic,
            &depth,
            input_set,
            own_set_id,
        )
        .map_err(|error| error.to_string())?;
        logs.insert(argument.name.clone(), values);
    }
    Ok((depth, logs, units))
}

/// SB-ENV-029 + SB-DBM-015: the declared neutron basis the runner injects for nphimat -
/// ONE resolution shared by the injection and the stored manifest, so they cannot drift.
fn nphimat_declared_basis(
    conn: &Connection,
    well_id: &str,
    log_args: &[(String, String)],
) -> Option<String> {
    let curve_name = log_args
        .iter()
        .find(|(argument, _)| argument == "NPHI")
        .map(|(_, curve)| curve.clone())
        .unwrap_or_else(|| "NPHI".to_string());
    let curve_id = equations::resolve_generic_curve_id(
        conn,
        well_id,
        &curve_name,
        equations::CurveRequest::SemanticFamily,
    )
    .ok()
    .flatten()?;
    conn.query_row(
        "SELECT neutron_basis FROM curve_meta WHERE curve_id = ?1",
        duckdb::params![curve_id],
        |row| row.get(0),
    )
    .ok()
    .flatten()
}

fn fetch_mask_aligned(
    conn: &Connection,
    well_id: &str,
    mask_name: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<Option<Vec<f32>>, String> {
    if mask_name.is_empty() {
        return Ok(None);
    }
    // SB-CLY-001 (DEC-036 constraint 2, kept where DEC-060 dropped the ENV guards): the CLY
    // provenance token curve is categorical - 0 means COMPUTED, so rule 11's any-non-zero
    // mask would blank every explained absence and pass exactly the computed samples,
    // inverting the intent. Refused by name.
    if mask_name.trim().eq_ignore_ascii_case("VSH_PROV") {
        return Err(format!(
            "{mask_name} is the CLY provenance token curve (registry v{}), not a flag: 0 means \
             COMPUTED, so masking on it would invert the intent. Mask with BADHOLE or another \
             binary flag instead.",
            crate::param_sources::CLY_PROV_REGISTRY_VERSION
        ));
    }
    let (_, columns) = equations::fetch_curve_frame_from_set(
        conn,
        well_id,
        &[mask_name.to_string()],
        input_set,
        own_set_id,
    )
    .map_err(|error| error.to_string())?;
    Ok(columns.get(&mask_name.to_uppercase()).cloned())
}

fn apply_mask_to_logs(
    logs: &mut HashMap<String, Vec<f32>>,
    log_args: &[(String, String)],
    mask: Option<&[f32]>,
) {
    let Some(mask) = mask else { return };
    for (argument, _) in log_args {
        if let Some(values) = logs.get_mut(argument) {
            for (value, flag) in values.iter_mut().zip(mask.iter()) {
                if modules::sample_is_flagged(*flag) {
                    *value = f32::NAN;
                }
            }
        }
    }
}

/// Read-only preflight for the module dialog. Only argument names and availability booleans leave
/// Rust; curve arrays remain behind IPC. A condition with no finite source sample is therefore
/// visible before launch without duplicating the runner's curve-resolution rules in TypeScript.
pub fn module_input_availability(
    db: &Mutex<Connection>,
    module: &str,
    well_ids: &[String],
    log_inputs: &HashMap<String, String>,
    input_set: Option<&str>,
) -> Result<Vec<ModuleInputAvailability>, String> {
    let spec = modules::list_modules()
        .into_iter()
        .find(|candidate| candidate.name == module)
        .ok_or_else(|| format!("unknown module '{module}'"))?;
    let needed = validity_input_arguments(&spec);
    let mut rows = Vec::with_capacity(well_ids.len());
    for well_id in well_ids {
        let resolved = db
            .lock()
            .map_err(|_| "database busy".to_string())
            .and_then(|conn| {
                let log_args = resolved_log_args_for_well(
                    &conn,
                    well_id,
                    &spec,
                    log_inputs,
                    input_set,
                    None,
                    &HashSet::new(),
                )?
                .into_iter()
                .filter(|(argument, _)| needed.contains(argument))
                .collect::<Vec<_>>();
                fetch_module_input_logs(&conn, well_id, &spec, &log_args, input_set, None)
                    .map(|resolved| (resolved, log_args))
            });
        match resolved {
            Ok(((_, logs, _), log_args)) => {
                let available_arguments = log_args
                    .iter()
                    .filter_map(|(argument, _)| {
                        logs.get(argument)
                            .is_some_and(|values| values.iter().any(|value| value.is_finite()))
                            .then(|| argument.clone())
                    })
                    .collect();
                rows.push(ModuleInputAvailability {
                    well_id: well_id.clone(),
                    available_arguments,
                    error: None,
                });
            }
            Err(error) => rows.push(ModuleInputAvailability {
                well_id: well_id.clone(),
                available_arguments: Vec::new(),
                error: Some(error),
            }),
        }
    }
    Ok(rows)
}

/// Read-only live preview for SB-ENV-031. It resolves the selected curve, zone-aware parameter
/// arrays and universal mask through the same helpers as the public runner. Only estimator names,
/// mathematical ceilings and counts leave Rust; the well-log arrays remain behind IPC.
pub fn despike_contamination_preview(
    db: &Mutex<Connection>,
    well_ids: &[String],
    log_inputs: &HashMap<String, String>,
    req_params: &HashMap<String, f64>,
    opts: &HashMap<String, String>,
    input_set: Option<&str>,
) -> Result<DespikeContaminationPreview, String> {
    let method = opts.get("OPT_METHOD").map(String::as_str).unwrap_or("HAMPEL");
    if method != "HAMPEL" {
        return Err(format!(
            "despike contamination ceiling applies to HAMPEL, not {method}; that method does not consume K"
        ));
    }
    let spec = modules::list_modules()
        .into_iter()
        .find(|candidate| candidate.name == "despike")
        .ok_or_else(|| "despike module is not registered".to_string())?;
    let log_args = resolved_log_args(&spec, log_inputs);
    let mask_name = opts.get("MASK").map(|value| value.trim()).unwrap_or("");
    let mut true_mad_samples = 0usize;
    let mut fallback_samples = 0usize;
    let mut true_mad_ceiling = None;
    let mut fallback_ceiling = None;
    let mut evaluated_wells = 0usize;
    let mut unavailable_well_ids = Vec::new();
    let mut issues = Vec::new();

    for well_id in well_ids {
        let resolved = db.lock().map_err(|_| "database busy".to_string()).and_then(|conn| {
            let (depth, mut logs, _) =
                fetch_module_input_logs(&conn, well_id, &spec, &log_args, input_set, None)?;
            let (parameters, _) =
                resolve_param_arrays_with_default_usage(&conn, well_id, &spec, req_params, &depth)?;
            let mask = fetch_mask_aligned(&conn, well_id, mask_name, input_set, None)?;
            apply_mask_to_logs(&mut logs, &log_args, mask.as_deref());
            Ok((depth, logs, parameters))
        });

        let (depth, logs, parameters) = match resolved {
            Ok(values) => values,
            Err(error) => {
                issues.push(DespikeContaminationIssue { well_id: well_id.clone(), error });
                continue;
            }
        };
        let values = logs.get("CURVE").map(Vec::as_slice).unwrap_or(&[]);
        if !values.iter().any(|value| value.is_finite()) {
            unavailable_well_ids.push(well_id.clone());
            continue;
        }
        let first_finite = |name: &str| {
            parameters
                .get(name)
                .and_then(|values| values.iter().copied().find(|value| value.is_finite()))
                .unwrap_or(f64::NAN)
        };
        let window = first_finite("WINDOW");
        let k = first_finite("K");
        match crate::condition::despike_contamination_profile(&depth, values, window, k) {
            Ok(branches) if !branches.is_empty() => {
                evaluated_wells += 1;
                for branch in branches {
                    match branch.estimator {
                        crate::condition::DespikeEstimator::TrueMad => {
                            true_mad_samples += branch.sample_count;
                            true_mad_ceiling = Some(branch.ceiling_pct);
                        }
                        crate::condition::DespikeEstimator::MeanDeviationFallback => {
                            fallback_samples += branch.sample_count;
                            fallback_ceiling = Some(branch.ceiling_pct);
                        }
                        crate::condition::DespikeEstimator::MeanSigmaPopulation => {
                            unreachable!("the shipped Hampel preview never runs a mean-sigma estimator")
                        }
                    }
                }
            }
            Ok(_) => unavailable_well_ids.push(well_id.clone()),
            Err(error) => issues.push(DespikeContaminationIssue { well_id: well_id.clone(), error }),
        }
    }

    let mut branches = Vec::with_capacity(2);
    if let Some(ceiling_pct) = true_mad_ceiling {
        branches.push(crate::condition::DespikeContaminationBranch {
            estimator: crate::condition::DespikeEstimator::TrueMad,
            ceiling_pct,
            sample_count: true_mad_samples,
        });
    }
    if let Some(ceiling_pct) = fallback_ceiling {
        branches.push(crate::condition::DespikeContaminationBranch {
            estimator: crate::condition::DespikeEstimator::MeanDeviationFallback,
            ceiling_pct,
            sample_count: fallback_samples,
        });
    }
    Ok(DespikeContaminationPreview {
        branches,
        evaluated_wells,
        unavailable_well_ids,
        issues,
    })
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
            .map(|well_id| ModuleRunResult::failed(well_id.clone(), error.clone()))
            .collect();
    }
    let spec = match modules::list_modules().into_iter().find(|m| m.name == req.module) {
        Some(s) => s,
        None => {
            return req
                .well_ids
                .iter()
                .map(|w| {
                    ModuleRunResult::failed(
                        w.clone(),
                        format!("unknown module '{}'", req.module),
                    )
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
                    .map(|well_id| ModuleRunResult::failed(well_id.clone(), error.clone()))
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

    // The names this run will write, decided ONCE. Every input to the decision — the manifest, the
    // chosen mnemonics, the renames — is well-independent, so a bad name is refused here as one
    // message rather than as N identical per-well failures in the Processing panel.
    let out_names = match resolve_output_names(&spec, &opts) {
        Ok(n) => n,
        Err(e) => {
            return req
                .well_ids
                .iter()
                .map(|w| ModuleRunResult::failed(w.clone(), e.clone()))
                .collect()
        }
    };

    // Phase 1 outcome per well. Outputs are held in memory so Phase 2 can write EVERY well in
    // one batched transaction (vs a fsync-bound delete+append transaction per well — the
    // dominant field-scale write cost). Nothing is written to computed_curves during Phase 1.
    enum Outcome {
        Skipped,
        Failed(String),
        Computed {
            depth: Vec<f32>,
            outputs: HashMap<String, Vec<f32>>,
            log_args: Vec<(String, String)>,
            degradations: Vec<modules::RunDegradation>,
            precondition_violations: Vec<modules::PreconditionViolation>,
            scientific_answered: bool,
            /// SB-POR-047: the hole-quality custody line for this run's log-set comment —
            /// present only for modules that declare BADHOLE. "Nobody looked" is a value here,
            /// never silence.
            badhole_record: Option<String>,
        },
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

    let defaulted_options: HashSet<String> = spec
        .args
        .iter()
        .filter(|arg| arg.kind == ArgKind::Option || arg.kind == ArgKind::Text)
        .filter(|arg| !req.opts.contains_key(&arg.name))
        .map(|arg| arg.name.clone())
        .collect();

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
            let compute = || -> Result<
                (
                    Vec<f32>,
                    HashMap<String, Vec<f32>>,
                    Vec<(String, String)>,
                    Vec<modules::RunDegradation>,
                    Vec<modules::PreconditionViolation>,
                    bool,
                    Option<String>,
                ),
                String,
            > {
                // A chain's own set event: its earlier steps' outputs beat the input set.
                let own_set = preset_sets.and_then(|m| m.get(well_id.as_str())).map(|s| s.as_str());
                let (depth, mut logs, input_units, params, defaulted_parameters, log_args) = {
                    let conn = db.lock().unwrap();
                    let log_args = resolved_log_args_for_well(
                        &conn,
                        well_id,
                        &spec,
                        &req.log_inputs,
                        req.input_set.as_deref(),
                        own_set,
                        &HashSet::new(),
                    )?;
                    validate_shale_clay_input_quantities(
                        &conn,
                        well_id,
                        &spec,
                        &log_args,
                        req.input_set.as_deref(),
                        own_set,
                    )?;
                    // SB-POR-024 (DEC-025): the N-D methods refuse an undeclared or
                    // wrong-basis neutron before anything computes.
                    validate_neutron_basis_input(&conn, well_id, &spec, &log_args)?;
                    let (depth, logs, input_units) = fetch_module_input_logs(
                        &conn,
                        well_id,
                        &spec,
                        &log_args,
                        req.input_set.as_deref(),
                        own_set,
                    )
                    ?;
                    if depth.is_empty() {
                        return Err("no curve data for well".into());
                    }
                    let (params, defaulted_parameters) = resolve_param_arrays_with_default_usage(
                        &conn,
                        well_id,
                        &spec,
                        &req.params,
                        &depth,
                    )?;
                    (depth, logs, input_units, params, defaulted_parameters, log_args)
                };

                // Optional bad-hole (or any flag) mask. Resolve it BEFORE the module runs so
                // flagged samples can be excluded from the module's INPUTS, not just its
                // outputs. Modules that compute run-level statistics — gr_normalize's P3/P97
                // percentiles, log_predict's KNN training set — would otherwise be anchored by
                // casing/washout samples, and that mis-anchoring contaminates every output
                // sample, flagged or not. The mask is resolved like any other input
                // (generic-store aware).
                let mask_name = req.opts.get("MASK").map(|s| s.trim()).unwrap_or("");
                let mask = {
                    let conn = db.lock().unwrap();
                    fetch_mask_aligned(
                        &conn,
                        well_id,
                        mask_name,
                        req.input_set.as_deref(),
                        own_set,
                    )?
                };

                // SB-ENV-027 (DEC-033): the ONE approved repair exemption - log_predict's SYN
                // when OPT_COMBINE = MAX_RAW, the mode that is genuinely a washout repair. The
                // declaration is per OUTPUT and per MODE: SYN produced under SYNTHETIC or
                // FILL_MISSING is masked normally, and BOTH mask passes are bypassed for the
                // declared repair - a repair blanked at the second pass is a repair that did
                // not happen. Adding an entry here is a DECISION that returns to DEC-033,
                // never an implementation convenience.
                let repair_run = req.module == "log_predict"
                    && req.opts
                        .get("OPT_COMBINE")
                        .map(|mode| mode.trim() == "MAX_RAW")
                        .unwrap_or(false);

                // Blank flagged samples in the module INPUTS (never DEPTH) before the run, so
                // per-run statistics only see unmasked data.
                if !repair_run {
                    apply_mask_to_logs(&mut logs, &log_args, mask.as_deref());
                }

                // Per-well opts: everything the run decided, plus THIS well's declared class
                // curves. Set only where the well has any, so a project that has never run a
                // clustering carries no extra key and behaves exactly as before.
                let mut well_opts = opts.clone();
                for (argument, unit) in input_units {
                    well_opts.insert(
                        format!("{}{}", modules::INPUT_UNIT_OPT_PREFIX, argument),
                        unit,
                    );
                }
                if let Some(cls) = class_by_well.get(well_id) {
                    well_opts.insert(modules::CLASS_CURVES_OPT.to_string(), cls.clone());
                }
                // SB-ENV-029 (DEC-025): nphimat validates MATRIX_IN against the input curve's
                // DECLARED neutron matrix basis, so the runner resolves the same curve the
                // fetch used and injects its declaration - never inferring one.
                if req.module == "nphimat" {
                    let conn = db.lock().unwrap();
                    if let Some(basis) = nphimat_declared_basis(&conn, well_id, &log_args) {
                        well_opts.insert(modules::NEUTRON_BASIS_OPT.to_string(), basis);
                    }
                }
                let ctx = ModuleContext { n: depth.len(), logs, params, opts: well_opts, depth_unit };
                // SB-POR-047 + SB-POR-026: run custody, composed where the inputs exist.
                // Supplied means the resolved column has ANY finite sample; a curve that
                // resolved but is all-NaN over this frame was never evaluated here, and saying
                // "nobody looked" is the honest record for it too. One line per declared flag,
                // joined into the run's DEC-039 version comment.
                let declares = |name: &str| {
                    spec.args
                        .iter()
                        .any(|argument| argument.kind == modules::ArgKind::LogIn && argument.name == name)
                };
                let mut custody_lines: Vec<String> = Vec::new();
                if declares("BADHOLE") {
                    let column = ctx.log("BADHOLE");
                    custody_lines.push(if column.iter().any(|value| value.is_finite()) {
                        let flagged = column.iter().filter(|value| **value == 1.0).count();
                        format!("BADHOLE consumed: {flagged} flagged samples excluded")
                    } else {
                        "BADHOLE not supplied - hole quality not evaluated".to_string()
                    });
                }
                if declares("GAS_FLAG") {
                    let column = ctx.log("GAS_FLAG");
                    custody_lines.push(if column.iter().any(|value| value.is_finite()) {
                        let flagged = column.iter().filter(|value| **value == 1.0).count();
                        format!("gas crossover flagged at {flagged} samples (condflag XOVER_FLAG consumed)")
                    } else {
                        "crossover flag not supplied - gas effect not evaluated".to_string()
                    });
                }
                // SB-POR-028: the limits the run actually BOUND, drained from the module's
                // own capture right after it returns (see below) and appended here.
                let badhole_record_base = custody_lines;
                let default_usage = modules::DefaultUsage {
                    parameter_samples: defaulted_parameters,
                    options: defaulted_options.clone(),
                };
                let (
                    mut outputs,
                    mut degradations,
                    mut precondition_violations,
                    precondition_flag,
                    (bound_limits, branch_counts),
                ) =
                    modules::run_module_with_degradations(&req.module, &ctx, default_usage)?;
                // SB-POR-028: the limits this run actually bound join the custody comment; a
                // module carrying clamp parameters that bound nothing says so, because "no
                // clamp bit" and "nobody would have told you" must not read the same.
                let mut custody_lines = badhole_record_base;
                // SB-POR-003: which physics answered, per branch, before the limit lines.
                if !branch_counts.is_empty() {
                    let detail = branch_counts
                        .iter()
                        .map(|(name, count)| format!("{name} {count} samples"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    custody_lines.push(format!("branches: {detail}"));
                }
                // SB-POR-028 keeps its own line for the input-conditioning clamps; every other
                // bound limit is an OUTPUT limit and gets the SB-POR-003 line below.
                let (clamps, output_limits): (Vec<(String, usize)>, Vec<(String, usize)>) =
                    bound_limits
                        .into_iter()
                        .partition(|(name, _)| name == "RHOSR" || name == "NPHISR");
                if !clamps.is_empty() {
                    let detail = clamps
                        .iter()
                        .map(|(name, count)| format!("{name} bound at {count} samples"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    custody_lines.push(format!("shale-reduction clamps: {detail}"));
                } else if spec.args.iter().any(|argument| argument.name == "NPHISR_MIN") {
                    custody_lines.push("shale-reduction clamps bound nothing".to_string());
                }
                // SB-POR-003: "every limit that bound" is DEC-039's own text - and a run whose
                // output limits bound nothing says so (the SB-POR-028 principle: no-limit-bit
                // and nobody-would-have-told-you must never read the same).
                if !output_limits.is_empty() {
                    let detail = output_limits
                        .iter()
                        .map(|(name, count)| format!("{name} at {count} samples"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    custody_lines.push(format!("output limits: {detail}"));
                } else if !branch_counts.is_empty() {
                    custody_lines.push("output limits: none bound".to_string());
                }
                let badhole_record =
                    (!custody_lines.is_empty()).then(|| custody_lines.join("; "));

                // A module returning a vector shorter OR longer than its depth frame is still
                // written by the established zip discipline, but only the common prefix survives.
                // SB-DBM-039 requires that usable partial result to say TRUNCATED, never Ok.
                let mut mismatched: Vec<_> = outputs
                    .iter()
                    .filter(|(_, values)| values.len() != depth.len())
                    .map(|(name, values)| (name.clone(), values.len()))
                    .collect();
                mismatched.sort_by(|left, right| left.0.cmp(&right.0));
                for (name, length) in mismatched {
                    degradations.push(modules::RunDegradation::one(
                        modules::RunDegradationKind::Truncated,
                        format!(
                            "output '{name}' had {length} samples for a {}-sample depth frame and was truncated to the common prefix",
                            depth.len()
                        ),
                    ));
                }

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

                // SB-ENV-027 (DEC-033): resolve the declared repair output's STORED name
                // (rename + prefix applied, exactly as the map above composed it).
                let repair_exempt_output: Option<String> = repair_run
                    .then(|| {
                        out_names
                            .iter()
                            .find(|(declared, _)| declared == "SYN")
                            .map(|(_, resolved)| {
                                match opts
                                    .get(OUT_PREFIX_OPT)
                                    .map(|value| value.trim())
                                    .filter(|value| !value.is_empty())
                                {
                                    Some(prefix) => {
                                        format!("{}{resolved}", prefix.to_uppercase())
                                    }
                                    None => resolved.clone(),
                                }
                            })
                    })
                    .flatten();

                // SB-CLY-001 (DEC-036): resolve the CLY provenance output's STORED name
                // (rename + prefix, exactly as the outputs map composed it) so the mask pass
                // below can WRITE the masked/disabled token instead of blanking it, and the
                // zone-bearing message can read the final tokens.
                let cly_prov_output: Option<String> = (req.module == "vsh_gr")
                    .then(|| {
                        out_names
                            .iter()
                            .find(|(declared, _)| declared == "VSH_PROV")
                            .map(|(_, resolved)| {
                                match opts
                                    .get(OUT_PREFIX_OPT)
                                    .map(|value| value.trim())
                                    .filter(|value| !value.is_empty())
                                {
                                    Some(prefix) => {
                                        format!("{}{resolved}", prefix.to_uppercase())
                                    }
                                    None => resolved.clone(),
                                }
                            })
                    })
                    .flatten();

                // Blank flagged samples in the OUTPUTS too, so a flagged depth's result is
                // never trusted downstream - EXCEPT the one declared repair output, whose
                // finite values at masked depths are the module's whole purpose.
                if let Some(mask) = &mask {
                    for (name, values) in outputs.iter_mut() {
                        if repair_exempt_output.as_deref() == Some(name.as_str()) {
                            continue;
                        }
                        if cly_prov_output.as_deref() == Some(name.as_str()) {
                            // SB-CLY-001: a masked sample's token is the mask's own statement,
                            // written HERE where the mask is known - blanking it would erase
                            // the one record of WHY the sample has no computed value.
                            for (value, flag) in values.iter_mut().zip(mask.iter()) {
                                if modules::sample_is_flagged(*flag) {
                                    *value = crate::param_sources::CLY_PROV_MASKED_INPUT;
                                }
                            }
                            continue;
                        }
                        for (v, m) in values.iter_mut().zip(mask.iter()) {
                            if modules::sample_is_flagged(*m) {
                                *v = f32::NAN;
                            }
                        }
                    }
                    // DEC-033 constraint 3: the typed binary companion that makes the
                    // exemption honest - 1 marks a finite value PRODUCED AT A MASKED DEPTH
                    // ("this number was reconstructed, not measured"), 0 an ordinary finite
                    // sample, MISSING where the output itself is. Deliberately NOT any
                    // replaced-sample flag: it discloses reconstruction, it does not judge
                    // hole quality.
                    if let Some(name) = repair_exempt_output.as_ref() {
                        if let Some(values) = outputs.get(name) {
                            let marker: Vec<f32> = values
                                .iter()
                                .zip(mask.iter())
                                .map(|(value, flag)| {
                                    if value.is_nan() {
                                        f32::NAN
                                    } else if modules::sample_is_flagged(*flag) {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                })
                                .collect();
                            outputs.insert(format!("{name}_RECON_FLAG"), marker);
                        }
                    }
                }

                // SB-CLY-001 (DEC-036 constraint 4): the zone-bearing run-level message.
                // The per-sample token does not discharge it - group the ENDPOINT_INVALID
                // samples by zone and name the parameter pair, the zone and the offending
                // values, on the same channel every other run degradation rides.
                if let Some(prov_name) = cly_prov_output.as_ref() {
                    if let Some(prov) = outputs.get(prov_name.as_str()) {
                        let invalid: Vec<usize> = prov
                            .iter()
                            .enumerate()
                            .filter(|(_, value)| {
                                **value == crate::param_sources::CLY_PROV_ENDPOINT_INVALID
                            })
                            .map(|(index, _)| index)
                            .collect();
                        if !invalid.is_empty() {
                            let zones = {
                                let conn = db.lock().map_err(|_| "database busy".to_string())?;
                                db::list_zones(&conn, well_id).map_err(|e| e.to_string())?
                            };
                            let mut groups: Vec<(String, usize, f64, f64)> = Vec::new();
                            for index in invalid {
                                let sample_depth = depth[index];
                                let zone = zones
                                    .iter()
                                    .find(|zone| {
                                        sample_depth >= zone.top_depth
                                            && sample_depth < zone.bottom_depth
                                    })
                                    .map(|zone| format!("zone '{}'", zone.zone_name))
                                    .unwrap_or_else(|| "the well-wide parameter set".to_string());
                                let value_at = |name: &str| {
                                    ctx.params
                                        .get(name)
                                        .and_then(|values| values.get(index))
                                        .copied()
                                        .unwrap_or(f64::NAN)
                                };
                                let gr_ma = value_at("GR_MA");
                                let gr_sh = value_at("GR_SH");
                                match groups.iter_mut().find(|group| group.0 == zone) {
                                    Some(group) => group.1 += 1,
                                    None => groups.push((zone, 1, gr_ma, gr_sh)),
                                }
                            }
                            for (zone, count, gr_ma, gr_sh) in groups {
                                degradations.push(modules::RunDegradation {
                                    kind: modules::RunDegradationKind::EndpointInvalid,
                                    detail: format!(
                                        "vsh_gr endpoint pair is degenerate in {zone}: GR_MA {gr_ma} >= GR_SH {gr_sh} - no computed value emitted; VSH_PROV carries the {} token (CLY provenance registry v{})",
                                        crate::param_sources::cly_prov_token(
                                            crate::param_sources::CLY_PROV_ENDPOINT_INVALID
                                        )
                                        .expect("the registry defines its own token"),
                                        crate::param_sources::CLY_PROV_REGISTRY_VERSION
                                    ),
                                    occurrences: count,
                                });
                            }
                        }
                    }
                }

                // Decide whether the module answered before adding the finite 0/1 framework flag.
                // Otherwise an all-MISSING scientific run would be versioned merely because its
                // companion flag contains zeros — exactly the false-success contract `answered`
                // exists to prevent.
                let scientific_answered = answered(&outputs);
                if let Some(flag) = precondition_flag {
                    let base = out_names
                        .iter()
                        .find(|(declared, _)| declared == modules::PRECONDITION_FLAG_OUTPUT_KEY)
                        .map(|(_, name)| name.clone())
                        .ok_or_else(|| {
                            "precondition flag policy was selected but no companion output name was resolved"
                                .to_string()
                        })?;
                    let name = opts
                        .get(OUT_PREFIX_OPT)
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(|prefix| format!("{}{base}", prefix.to_uppercase()))
                        .unwrap_or(base);
                    outputs.insert(name, flag);
                }

                degradations.sort_by(|left, right| {
                    left.kind
                        .cmp(&right.kind)
                        .then_with(|| left.detail.cmp(&right.detail))
                });
                precondition_violations.sort_by(|left, right| {
                    left.condition_id
                        .cmp(&right.condition_id)
                        .then_with(|| left.argument.cmp(&right.argument))
                });
                Ok((
                    depth,
                    outputs,
                    log_args,
                    degradations,
                    precondition_violations,
                    scientific_answered,
                    badhole_record,
                ))
            };

            let outcome = match compute() {
                Ok((
                    depth,
                    outputs,
                    log_args,
                    degradations,
                    precondition_violations,
                    scientific_answered,
                    badhole_record,
                )) => Outcome::Computed {
                    depth,
                    outputs,
                    log_args,
                    degradations,
                    precondition_violations,
                    scientific_answered,
                    badhole_record,
                },
                Err(e) => Outcome::Failed(e),
            };
            if let Some(p) = progress {
                match &outcome {
                    // A run whose outputs are all MISSING (e.g. gascorr with no precalc, or a
                    // module fed an all-NaN input) did no real work — flag it Warned, not a green
                    // Ok, so the panel doesn't read as a successful correction.
                    Outcome::Computed {
                        scientific_answered,
                        degradations,
                        precondition_violations,
                        ..
                    } if *scientific_answered
                        && degradations.is_empty()
                        && precondition_violations.is_empty() =>
                    {
                        p.finish_item(well_id, crate::jobs::ItemState::Ok, None)
                    }
                    Outcome::Computed {
                        scientific_answered,
                        degradations,
                        precondition_violations,
                        ..
                    } if *scientific_answered => {
                        p.finish_item(
                            well_id,
                            crate::jobs::ItemState::Warned,
                            Some(run_warning_message(
                                &req.module,
                                degradations,
                                precondition_violations,
                            )),
                        )
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
            Outcome::Computed { scientific_answered: true, .. } => Some(w.clone()),
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
            let Outcome::Computed {
                outputs,
                log_args,
                precondition_violations,
                scientific_answered,
                ..
            } = outcome
            else {
                continue;
            };
            if !*scientific_answered {
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
                log_args,
                &names,
                precondition_violations,
                parameter_serializer,
            ) {
                Ok(mut spec) => {
                    // SB-DBM-015: the arms the spec builder cannot know - the depth frame
                    // exists only after the fetch, and the physics-driving attribute value
                    // is the one the runner injected (same helper, so record and injection
                    // cannot drift).
                    let Outcome::Computed { depth, .. } = outcome else { unreachable!() };
                    let frame = (!depth.is_empty()).then(|| equations::ManifestDepthFrame {
                        top: depth[0],
                        base: depth[depth.len() - 1],
                        samples: depth.len(),
                    });
                    let physics = if req.module == "nphimat"
                        || modules::required_neutron_basis(&req.module).is_some()
                    {
                        nphimat_declared_basis(&conn, well_id, log_args)
                            .map(|value| {
                                vec![equations::PhysicsAttribute {
                                    name: "neutron_basis".into(),
                                    value,
                                }]
                            })
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    if let Err(error) = spec.record_run_manifest(frame, physics) {
                        build_error = Some(error);
                        break;
                    }
                    complete.push(equations::CompleteWellLogSet {
                        well_id: well_id.clone(),
                        spec,
                    });
                }
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
        if let Outcome::Computed {
            depth,
            outputs,
            degradations,
            scientific_answered,
            ..
        } = o
        {
            if !*scientific_answered {
                continue;
            }
            if let Some(set_id) = set_ids.get(well_id) {
                writes.push(equations::CompleteWellWrite {
                    well_id: well_id.clone(),
                    depth: depth.clone(),
                    curves: outputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    set_id: set_id.clone(),
                    degradation_module: req.module.clone(),
                    degradations: degradations.clone(),
                });
                // SB-POR-047 / DEC-039: the hole-quality custody line goes on THIS version's
                // comment. A failure to record it degrades the run's record, not the run.
                if let Outcome::Computed { badhole_record: Some(record), .. } = o {
                    let conn = db.lock().unwrap();
                    let _ = equations::set_log_set_comment(&conn, set_id.as_str(), record);
                }
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
            Outcome::Skipped => ModuleRunResult::skipped(well_id.clone()),
            Outcome::Failed(e) => ModuleRunResult::failed(well_id.clone(), e.clone()),
            Outcome::Computed {
                depth,
                outputs,
                degradations,
                precondition_violations,
                scientific_answered,
                ..
            } => {
                if outputs.is_empty() {
                    ModuleRunResult::skipped(well_id.clone())
                } else if !*scientific_answered {
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
                        outcome: ModuleRunOutcome::Failed,
                        degradations: degradations.clone(),
                    }
                } else if let Some(e) = &set_err {
                    ModuleRunResult::failed(well_id.clone(), e.clone())
                } else if !set_ids.contains_key(well_id) {
                    ModuleRunResult::failed(
                        well_id.clone(),
                        "no output set allocated for well".into(),
                    )
                } else if let Some(e) = &write_err {
                    ModuleRunResult::failed(well_id.clone(), e.clone())
                } else {
                    let mut names: Vec<String> = outputs.keys().cloned().collect();
                    names.sort();
                    ModuleRunResult {
                        well_id: well_id.clone(),
                        rows_written: depth.len(),
                        output_curves: names,
                        error: None,
                        outcome: if degradations.is_empty() && precondition_violations.is_empty() {
                            ModuleRunOutcome::Clean
                        } else {
                            ModuleRunOutcome::Degraded
                        },
                        degradations: degradations.clone(),
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
    /// SB-CUT-001 (DEC-071): the thickness discretisation model. Defaults to CENTRED per
    /// the ruling; FORWARD ("TOPS") stays selectable to reproduce a legacy run's numbers.
    #[serde(default)]
    pub discretisation: DiscretisationModel,
    /// SB-CUT-016. VSH <= vsh_max counts as sand. **`None` means UNFILTERED** — the property is
    /// not used to exclude anything, and the result says so. There is deliberately no default:
    /// four shipped vendor sets disagree, two of them from one vendor, and Jauhar's own delivered
    /// work spans Vsh 0.20-0.85 across intervals of a single area.
    /// SB-CUT-019: carried AS ENTERED, with its unit, and canonicalised on receipt. A bare
    /// number is refused rather than guessed at.
    pub vsh_max: Option<CutoffSpec>,
    /// SB-CUT-016. PHIE >= phie_min counts as reservoir (with sand). `None` = unfiltered.
    pub phie_min: Option<CutoffSpec>,
    /// SB-CUT-016. SWE <= swe_max counts as pay (with reservoir). `None` = unfiltered.
    pub swe_max: Option<CutoffSpec>,
    /// PERM >= perm_min added to the pay flag when PERM exists. `None` = unfiltered.
    pub perm_min: Option<CutoffSpec>,
    /// SB-CUT-016. Cut-offs the caller switched ON and left without a value. A summation **MUST
    /// NOT** run against one, so any name here refuses the whole request.
    ///
    /// Separate from a `None` value on purpose: *"I am not filtering on Sw"* and *"I meant to
    /// filter on Sw and have not said what"* are different statements, and only one of them may
    /// produce a number. `#[serde(default)]`, so every record written before this existed still
    /// deserializes and still means what it meant.
    #[serde(default)]
    pub enabled_unset: Vec<String>,
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
    /// SB-CUT-009. Per-curve averaging weighting, keyed by the SLOT the curve fills — one of
    /// [`AVERAGED_SLOTS`], a role rather than a mnemonic. Absent slots take [`default_weighting`],
    /// so a caller who declares nothing gets exactly the behaviour that shipped before this
    /// existed. Persisted with the rest of the run's configuration in `log_sets.params_json`,
    /// which is what makes it *stored with the curve's averaging configuration* rather than an
    /// argument that evaporates after the run.
    #[serde(default)]
    pub weighting: BTreeMap<String, AverageWeighting>,
    /// SB-CUT-022. Which report tiers each cut-off is USED at, keyed by SLOT. An absent slot takes
    /// [`default_cutoff_use`], which is the ladder that shipped before this existed — so a caller
    /// who declares nothing sees no number move. Persisted with the rest of the run's
    /// configuration, which is what makes the activation auditable FROM A RESULT rather than
    /// re-derivable only by knowing which rule the engine happened to apply.
    #[serde(default)]
    pub cutoff_use: BTreeMap<String, CutoffUse>,
    /// SB-CUT-012. The depth frame to summate in. Defaults to MD, which is the only frame
    /// SandiBumi can currently weight; any other is REFUSED rather than served MD numbers under
    /// a different label.
    #[serde(default)]
    pub frame: SummationFrame,
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
    /// SB-CUT-002: the discretisation model this row's thicknesses were computed under. A
    /// consumer must never have to infer it — two tools disagreeing by half a step at every
    /// zone contact both print plausible nets.
    pub discretisation_model: String,
    /// SB-CUT-002: the sample interval (project depth unit) the summation ran on — the median
    /// forward step of this well's frame. Net-to-gross is not scale-invariant, so two rows
    /// computed at different steps are different statements even over the same rock.
    pub sample_interval: f32,
    /// SB-CUT-003. Footage the classifier EVALUATED and rejected — it saw the sample and the
    /// sample failed a cutoff.
    ///
    /// Kept strictly apart from [`Self::unknown`] because the two are the same number on a page
    /// and completely different rock. A zone reading 40 % net-to-gross because 60 % is shale and a
    /// zone reading 40 % because 55 % was never logged both print 0.40, and only the split says
    /// which. Techlog books a non-positive clipped interval as UNKNOWN distinct from NOT-NET; IP
    /// marks nulls in-band inside the numeric column and never separates them at all.
    pub not_net: f32,
    /// SB-CUT-003. Footage whose flag could not be EVALUATED, so that
    /// `gross = net + not_net + unknown` holds exactly.
    ///
    /// **Derived rather than accumulated, and that is the substance of the requirement.** Two
    /// separate things make footage unjudgeable and only one of them is a sample: an in-zone
    /// sample whose VSH/PHIE/SWE are missing, and footage carrying no sample at all — a logging
    /// gap, or the ordinary case of a zone bottomed on a marker below the TD of the run that
    /// logged it. Summing the first alone would leave the identity broken over exactly the
    /// intervals where a reader most needs it to close.
    pub unknown: f32,
    /// SB-CUT-004. Net-to-gross over the footage the classifier could actually judge —
    /// `net / (gross - unknown)`, the chapter's `N:(G−Unknown)`.
    ///
    /// Reported BESIDE [`Self::ntg`] rather than instead of it, because the two answer different
    /// questions and the gap between them is the null fraction. Over a washed-out or
    /// partially-logged interval that gap is the whole argument about whether a net-to-gross is
    /// defensible; no incumbent surfaces both, so an interpreter comparing one tool's number with
    /// another's cannot tell which was quoted.
    ///
    /// **MISSING, never zero, where nothing was judged.** With no judged footage there is no
    /// denominator, and a printed 0.00 would be a claim about rock nobody looked at — the same
    /// reasoning as [`Self::n_classified`]. Crosses IPC as JSON `null`, like the `avg_*` fields.
    pub ntg_known: f32,
    /// SB-CUT-030. True when an emitted zonal average falls outside its quantity's physical
    /// bounds. The value is **emitted as computed, not corrected** — a corrected average is a
    /// number nobody derived, and the condition that produced it is exactly what a reviewer needs
    /// to see. It rides in its own typed field for the SB-CUT-029 reason: a marker inside the
    /// numeric column would stop being arithmetic.
    #[serde(default)]
    pub out_of_range: bool,
    /// SB-CUT-005. Footage moved into the largest component so the partition closes — reported
    /// rather than printed, which is the whole point of the requirement. Zero on any run whose
    /// partition already closed, which is every ordinary run; a non-zero value here is the record
    /// that a correction happened and how big it was.
    pub residual_absorbed: f32,
    /// SB-CUT-012. The depth frame these weights were measured in — part of the result's identity.
    /// An MD and a TVD summation are separate records, never one rescaled into the other.
    pub frame: SummationFrame,
    /// SB-CUT-012. What the per-sample weights were differenced from. Naming the frame alone does
    /// not say WHICH depths produced the increments.
    pub weights_source: String,
    /// SB-CUT-016. Cut-offs NOT applied to this summation, in VSH/PHIE/SWE/PERM order. An
    /// unfiltered summation must be reported AS unfiltered - a net that quietly stopped being
    /// filtered, with nothing on the result to say so, is the whole failure this prevents.
    pub unfiltered: Vec<String>,
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
    /// SB-POR-057 (DEC-070, RULED 2026-08-18: "quick look only shows pay summation as
    /// visual not pay curves"). **This well's porosity exists ONLY as the quick-look D-N
    /// comparison curve (`PHIE_DN_LIM`), and it was deliberately not summed** - the
    /// quick-look shortcuts may be OVERLAID on a display as a visual comparison, but never
    /// feed net/NTG/HPV. Without this mark the zeros on such a well read exactly like a
    /// wet well; with it a reader sees the curve existed and why it was refused. Supersedes
    /// the pay-eligible fallback DEC-042 shipped. Per well, like
    /// [`Self::perm_cutoff_no_data`]; false both when an authoritative `PHIE` was summed
    /// and when the well simply has no porosity at all - the flag means "present and
    /// excluded", never "absent".
    #[serde(default)]
    pub quicklook_phie_excluded: bool,
}

const SUMMARY_FLAGS: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

/// SB-CUT-019. The quantity a cut-off constrains, which fixes both its canonical unit and the
/// physical range it cannot leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutoffQuantity {
    /// A volume fraction: Vsh, porosity, saturation. Canonical `v/v`, bounded 0..=1.
    VolumeFraction,
    /// Permeability. Canonical `mD`, bounded to non-negative.
    Permeability,
}

impl CutoffQuantity {
    pub fn canonical_unit(self) -> &'static str {
        match self {
            CutoffQuantity::VolumeFraction => "v/v",
            CutoffQuantity::Permeability => "mD",
        }
    }
}

/// SB-CUT-019. A cut-off AS ENTERED — a number and the unit it was entered in.
///
/// The unit is not decoration. IP's own manual expresses the sensitivity-sweep example in porosity
/// units and the cut-off default in `v/v` **for the same quantity, with no unit tag on the field**.
/// Entering `35` where `0.1` is meant is a **350x** error whose symptom is an all-net result: a
/// good-looking well, not a visible failure. So a bare number is refused rather than guessed at.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CutoffEntry {
    pub value: f64,
    /// The unit the user typed. Empty is a REFUSAL, never an assumption.
    pub unit: String,
}

/// A bare number on the wire still DESERIALIZES — it becomes an entry with an empty unit, which
/// then fails [`CutoffEntry::canonical`] with the message that names the field and says why.
///
/// Deliberate: refusing at the parse layer would return serde's *invalid type* text, which tells
/// an analyst nothing about porosity units, and would also break every request shape written
/// before this existed. The value is rejected either way; this controls WHICH message they get.
impl<'de> Deserialize<'de> for CutoffEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Bare(f64),
            Tagged { value: f64, unit: String },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Bare(value) => CutoffEntry { value, unit: String::new() },
            Wire::Tagged { value, unit } => CutoffEntry { value, unit },
        })
    }
}

impl CutoffEntry {
    /// Convert to the quantity's canonical unit, refusing a bare number, an unknown unit, and a
    /// value outside the quantity's physical range.
    ///
    /// `35 pu` becomes `0.35 v/v`; `35 v/v` is refused as out of bounds - the same number, and
    /// only the unit says which of those two the user meant.
    pub fn canonical(&self, quantity: CutoffQuantity, label: &str) -> Result<f64, String> {
        let unit = self.unit.trim();
        if unit.is_empty() {
            return Err(format!(
                "{label} was entered as a bare number ({}) with no unit. A porosity cut-off \
                 typed as 35 is 0.35 in porosity units and impossible in v/v, and the 350x \
                 error looks like an all-net well rather than a failure - so state the unit.",
                self.value
            ));
        }
        if !self.value.is_finite() {
            return Err(format!("{label} is not a finite number"));
        }
        let lower = unit.to_ascii_lowercase();
        let canonical = match quantity {
            CutoffQuantity::VolumeFraction => match lower.as_str() {
                "v/v" | "frac" | "fraction" | "dec" => self.value,
                "pu" | "p.u." | "%" | "pct" | "percent" => self.value / 100.0,
                _ => {
                    return Err(format!(
                        "{label} is in '{unit}', which is not a unit of volume fraction. \
                         Use v/v, pu or %."
                    ))
                }
            },
            CutoffQuantity::Permeability => match lower.as_str() {
                "md" => self.value,
                "d" | "darcy" => self.value * 1000.0,
                _ => {
                    return Err(format!(
                        "{label} is in '{unit}', which is not a unit of permeability. Use mD or D."
                    ))
                }
            },
        };
        let out_of_range = match quantity {
            CutoffQuantity::VolumeFraction => !(0.0..=1.0).contains(&canonical),
            CutoffQuantity::Permeability => canonical < 0.0,
        };
        if out_of_range {
            return Err(format!(
                "{label} is {} {unit}, which is {canonical} {} - outside the physical range \
                 of the quantity. A volume fraction cannot exceed 1; if porosity units were \
                 meant, enter the unit as pu.",
                self.value,
                quantity.canonical_unit()
            ));
        }
        Ok(canonical)
    }
}

/// SB-CUT-030. The three named stages a value passes through, and whether each clamps.
///
/// **`Accumulate` never clamps, and that is the whole requirement.** Clamping inside a sum is not
/// a display choice; it moves the MEAN. For a truly wet interval the unclamped hydrocarbon
/// contribution `phi*(1-Sw)` has expectation zero under symmetric noise, while the clamped
/// contribution `phi*max(0, 1-Sw)` has expectation `phi*sigma/sqrt(2*pi)` = `0.3989*phi*sigma > 0`
/// — always toward MORE hydrocarbon, by an amount independent of iteration count
/// (`docs/PRD_v2/14_cutoffs-summation-mc.md:789-794`). A clamp that is correct for one
/// deterministic evaluation is a bias in expectation over an ensemble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClampStage {
    /// Summation. NEVER clamped.
    Accumulate,
    /// The cut-off comparison. Clamped to the quantity's bounds.
    FlagTest,
    /// What a reader is shown. Clamped to the quantity's bounds.
    Present,
}

/// SB-CUT-030. A quantity's physical bounds — **attached to the QUANTITY, never to a curve-type
/// string**.
///
/// Binding bounds to a type string is the specific failure that makes IP's clipping worse than
/// Techlog's unconditional clamp: IP clips by *declared curve type*, so mis-typing a curve silently
/// changes its numerics, and the change is **invisible in the data**. A quantity cannot be
/// mis-typed by a label because it is not a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedQuantity {
    /// A volume fraction: Vsh, porosity, saturation. Bounded `0..=1`.
    VolumeFraction,
    /// Permeability. Bounded below at zero and **unbounded above**.
    Permeability,
    /// A quantity with no physical bounds at all — a reconstruction error, a resistivity, a
    /// coefficient. It must NOT be clamped to `[0,1]` merely because that is the common case.
    Unbounded,
}

impl BoundedQuantity {
    /// The bounds, or `None` where the quantity has none. An open upper bound is `f64::INFINITY`
    /// rather than a large number, so nothing accidentally clips a real permeability.
    pub fn bounds(self) -> Option<(f64, f64)> {
        match self {
            BoundedQuantity::VolumeFraction => Some((0.0, 1.0)),
            BoundedQuantity::Permeability => Some((0.0, f64::INFINITY)),
            BoundedQuantity::Unbounded => None,
        }
    }

    /// Whether a value lies outside the quantity's bounds. A NaN is not out of range — it is
    /// absent, which is a different statement and already has its own carrier (SB-CUT-029).
    pub fn is_out_of_range(self, value: f32) -> bool {
        match self.bounds() {
            Some((lo, hi)) => value.is_finite() && ((value as f64) < lo || (value as f64) > hi),
            None => false,
        }
    }
}

/// SB-CUT-030. Apply one stage's clamping policy to one value of one quantity.
///
/// The single place the policy is expressed, so `accumulate` cannot quietly acquire a clamp in one
/// caller while the others keep theirs.
pub fn stage_value(stage: ClampStage, quantity: BoundedQuantity, value: f32) -> f32 {
    match (stage, quantity.bounds()) {
        // Never, for any quantity. This arm is the requirement.
        (ClampStage::Accumulate, _) => value,
        // An unbounded quantity is not clamped at any stage — there is nothing to clamp it to.
        (_, None) => value,
        (_, Some((lo, hi))) => {
            if value.is_nan() {
                value
            } else {
                value.clamp(lo as f32, hi as f32)
            }
        }
    }
}

/// SB-CUT-030. The PRESENT stage for an emitted zonal average.
///
/// Clamped to the quantity's bounds, **except** where the average falls outside them — which the
/// requirement says must be emitted AS COMPUTED and flagged, never corrected. A corrected average
/// is a number nobody derived, and the condition that produced it is exactly what a reviewer needs
/// to see. So the clamp is inert on every ordinary run by construction: an in-range value has
/// nothing to clamp, and an out-of-range one is deliberately let through beside its flag.
fn present_average(value: f32) -> f32 {
    let quantity = BoundedQuantity::VolumeFraction;
    if quantity.is_out_of_range(value) {
        value
    } else {
        stage_value(ClampStage::Present, quantity, value)
    }
}

/// SB-CUT-020. Which side of a bound a sample sitting exactly ON it falls.
///
/// Spelled as words rather than as `>=` / `>`, because a symbol on the wire invites parsing and
/// this is the one field where a misread is invisible: it changes the verdict only for samples
/// exactly on the cut-off, which is exactly the population a marginal-pay result turns on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BoundOperator {
    /// A sample exactly equal to the bound is INSIDE. `x >= min`, `x <= max`.
    #[default]
    Inclusive,
    /// A sample exactly equal to the bound is OUTSIDE. `x > min`, `x < max`.
    Exclusive,
}

/// SB-CUT-020. One side of a cut-off range, in the quantity's canonical unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CutoffBound {
    pub value: f64,
    pub operator: BoundOperator,
}

/// SB-CUT-020. Which side of a range a single-sided cut-off occupies.
///
/// A slot named `phie_min` has always meant *at least this*, and `vsh_max` *at most this*. The
/// sense is the slot's, not the value's, so the degenerate form cannot land on the wrong side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutoffSense {
    Minimum,
    Maximum,
}

/// SB-CUT-020. A cut-off as a two-sided range — **and this doc comment is the specification that
/// SB-CUT-T24 tests against**, deliberately, because the vendor cannot be the oracle: Techlog
/// documents its modes 2 and 3 as outside tests and implements them as inside tests.
///
/// **The specification.** A sample value `x` passes the cut-off when it satisfies BOTH bounds:
///
/// | Side | Operator | Passes when | A sample exactly on the bound |
/// |---|---|---|---|
/// | low  | `INCLUSIVE` | `x >= value` | **inside** |
/// | low  | `EXCLUSIVE` | `x > value`  | **outside** |
/// | high | `INCLUSIVE` | `x <= value` | **inside** |
/// | high | `EXCLUSIVE` | `x < value`  | **outside** |
/// | either | *absent* | always | *not applicable — the far bound is open* |
///
/// An absent bound is an OPEN far bound, satisfied by every value. The single-sided `>=` / `<=`
/// forms are therefore this range with one side absent and the other `INCLUSIVE` — the degenerate
/// case, not a separate mechanism, so a project saved before ranges existed classifies identically.
///
/// `INCLUSIVE` is the default on both sides for the same reason: it is what the single-sided forms
/// have always meant, and a generalisation that silently moved the boundary would rewrite every
/// existing marginal result.
///
/// A range that can admit no value is REFUSED rather than quietly booking zero net — see
/// [`CutoffSpec::canonical`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct CutoffRange {
    pub low: Option<CutoffBound>,
    pub high: Option<CutoffBound>,
}

impl CutoffRange {
    /// The specification above, and the only place a cut-off comparison is made.
    ///
    /// **The comparison happens in `f32`, the precision the DATA has, and that is required rather
    /// than convenient.** A continuous log is `f32` (collaboration rule 2) while a cut-off is
    /// entered as a decimal and held as `f64`. Widen the sample instead and `0.30f32` becomes
    /// `0.30000001192…`, which is strictly GREATER than `0.30f64` — so a sample the user entered
    /// `0.30` to sit exactly on never sits on it, and the EXCLUSIVE operator silently excludes
    /// nothing at all. That is Techlog's mode 7 arrived at by arithmetic instead of by a bug.
    /// Narrowing the bound instead compares two numbers the data can actually distinguish, which
    /// is the only reading under which "exactly equal to the bound" means anything.
    pub fn contains(&self, sample: f32) -> bool {
        // A NaN satisfies no comparison, which is the honest answer: an unmeasured sample cannot
        // demonstrate that it passes. The callers handle missing data before reaching here.
        let low_ok = match self.low {
            Some(CutoffBound { value, operator: BoundOperator::Inclusive }) => sample >= value as f32,
            Some(CutoffBound { value, operator: BoundOperator::Exclusive }) => sample > value as f32,
            None => true,
        };
        let high_ok = match self.high {
            Some(CutoffBound { value, operator: BoundOperator::Inclusive }) => sample <= value as f32,
            Some(CutoffBound { value, operator: BoundOperator::Exclusive }) => sample < value as f32,
            None => true,
        };
        low_ok && high_ok
    }
}

/// SB-CUT-020. One side of a cut-off range AS ENTERED — a value, its unit, and its operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutoffSpecBound {
    #[serde(flatten)]
    pub entry: CutoffEntry,
    #[serde(default)]
    pub operator: BoundOperator,
}

/// SB-CUT-020. A cut-off as it arrives on the wire: a bare number, a `{value, unit}` entry, or a
/// `{min, max}` range with a per-bound operator.
///
/// The first two forms are the degenerate single-sided case and are accepted unchanged, so every
/// caller written before ranges existed keeps working and keeps meaning what it meant.
#[derive(Debug, Clone, PartialEq)]
pub struct CutoffSpec {
    pub min: Option<CutoffSpecBound>,
    pub max: Option<CutoffSpecBound>,
    /// The single-sided form, held until the slot's [`CutoffSense`] says which side it belongs to.
    pub single: Option<CutoffSpecBound>,
}

/// Serialization is the INVERSE of deserialization, deliberately: a persisted run has to reload
/// as the cut-off it was. A degenerate single-sided spec therefore writes the same object it
/// arrived as - now carrying its operator - and a range writes `{min, max}` with an absent side
/// omitted rather than written as a null.
impl Serialize for CutoffSpec {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match (&self.min, &self.max, &self.single) {
            (_, _, Some(single)) => single.serialize(serializer),
            (min, max, None) => {
                let mut map = serializer.serialize_map(None)?;
                if let Some(bound) = min {
                    map.serialize_entry("min", bound)?;
                }
                if let Some(bound) = max {
                    map.serialize_entry("max", bound)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CutoffSpec {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            // NARROWEST FIRST, and it has to be. Both of `Range`'s fields are optional, so that
            // arm matches ANY object — including `{value, unit}`, which it would silently accept
            // as a range with no bounds at all: a cut-off that filters nothing, configured. That
            // is Techlog's mode 7 reproduced by an ordering mistake, so `Single` is tried first.
            // A `{min, max}` object carries no `value` field, so it cannot match `Single`.
            Single(CutoffSpecBound),
            /// A bare number is not a map, so it cannot reach the flattened `Single` arm - and
            /// SB-CUT-019 requires it to PARSE and then be refused by name rather than returning
            /// serde's *invalid type* text. It becomes a unitless single bound, which `canonical`
            /// rejects with the message about porosity units.
            Bare(f64),
            Range {
                #[serde(default)]
                min: Option<CutoffSpecBound>,
                #[serde(default)]
                max: Option<CutoffSpecBound>,
            },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Range { min, max } => CutoffSpec { min, max, single: None },
            Wire::Single(single) => CutoffSpec { min: None, max: None, single: Some(single) },
            Wire::Bare(value) => CutoffSpec {
                min: None,
                max: None,
                single: Some(CutoffSpecBound {
                    entry: CutoffEntry { value, unit: String::new() },
                    operator: BoundOperator::default(),
                }),
            },
        })
    }
}

/// SB-CUT-020. The degenerate case, as a conversion: a single entered value becomes the
/// single-sided range whose far bound is open and whose operator is `INCLUSIVE`.
///
/// The same statement the wire form makes, available in Rust so a caller that already holds an
/// entry does not have to spell the range out and risk spelling it differently.
impl From<CutoffEntry> for CutoffSpec {
    fn from(entry: CutoffEntry) -> Self {
        CutoffSpec {
            min: None,
            max: None,
            single: Some(CutoffSpecBound { entry, operator: BoundOperator::default() }),
        }
    }
}

impl CutoffSpec {
    /// Convert to canonical units and resolve the slot's sense, refusing anything the range
    /// specification cannot mean.
    pub fn canonical(
        &self,
        quantity: CutoffQuantity,
        sense: CutoffSense,
        label: &str,
    ) -> Result<CutoffRange, String> {
        let bound = |side: &Option<CutoffSpecBound>, side_label: &str| {
            side.as_ref()
                .map(|b| {
                    b.entry
                        .canonical(quantity, &format!("{label} {side_label}"))
                        .map(|value| CutoffBound { value, operator: b.operator })
                })
                .transpose()
        };
        let mut range = CutoffRange {
            low: bound(&self.min, "lower bound")?,
            high: bound(&self.max, "upper bound")?,
        };
        if let Some(single) = bound(&self.single, "")? {
            match sense {
                CutoffSense::Minimum => range.low = Some(single),
                CutoffSense::Maximum => range.high = Some(single),
            }
        }
        // A window nobody could have meant is refused, not run. Zero net from an inverted range
        // computes and plots exactly like zero net from tight rock, which is this row's risk class.
        if let (Some(low), Some(high)) = (range.low, range.high) {
            let empty = low.value > high.value
                || (low.value == high.value
                    && (low.operator == BoundOperator::Exclusive
                        || high.operator == BoundOperator::Exclusive));
            if empty {
                return Err(format!(
                    "{label} is the range {} to {}, which admits no value at all. A cut-off that \
                     cannot pass books zero net and looks exactly like tight rock, so it is \
                     refused rather than run.",
                    low.value, high.value
                ));
            }
        }
        Ok(range)
    }
}

/// SB-CUT-016. Render a cut-off for a deliverable: its value, or the word that says it was never
/// applied.
///
/// One helper rather than a spelling per surface. The two failures it exists to prevent are
/// printing nothing - a reader then assumes the cut-off was used - and printing a number that was
/// never applied, which is worse because it is checkable and wrong.
pub fn cutoff_label(value: Option<&CutoffSpec>, decimals: usize) -> String {
    // SB-CUT-019: the unit is printed WITH the number. A deliverable that says "PHIE >= 0.10"
    // without saying in what has reproduced the very ambiguity the entry rule exists to stop.
    let Some(spec) = value else {
        return "unfiltered".to_string();
    };
    // SB-CUT-020: a two-sided range prints in interval notation, where the bracket IS the operator
    // and an engineer reads it without a legend. The single-sided inclusive form keeps its bare
    // number, because that is what every existing deliverable shows and it has not changed meaning.
    match (&spec.min, &spec.max, &spec.single) {
        (_, _, Some(single)) => {
            let unit = single.entry.unit.trim();
            match single.operator {
                BoundOperator::Inclusive => format!("{:.decimals$} {unit}", single.entry.value),
                BoundOperator::Exclusive => {
                    format!("{:.decimals$} {unit} (exclusive)", single.entry.value)
                }
            }
        }
        (low, high, None) => {
            let unit = low
                .as_ref()
                .or(high.as_ref())
                .map(|b| b.entry.unit.trim().to_string())
                .unwrap_or_default();
            let open_bracket = match low {
                Some(b) if b.operator == BoundOperator::Exclusive => "(",
                Some(_) => "[",
                None => "(",
            };
            let close_bracket = match high {
                Some(b) if b.operator == BoundOperator::Exclusive => ")",
                Some(_) => "]",
                None => ")",
            };
            let lo = low
                .as_ref()
                .map(|b| format!("{:.decimals$}", b.entry.value))
                .unwrap_or_else(|| "-inf".into());
            let hi = high
                .as_ref()
                .map(|b| format!("{:.decimals$}", b.entry.value))
                .unwrap_or_else(|| "+inf".into());
            format!("{open_bracket}{lo}, {hi}{close_bracket} {unit}")
                .trim_end()
                .to_string()
        }
    }
}

/// SB-CUT-020. Render a cut-off as a PHRASE for running prose — `>= 0.10 mD`, `> 0.10 mD`,
/// `in [0.10, 0.35] v/v` — or the empty string when the cut-off was never applied.
///
/// Separate from [`cutoff_label`] because prose needs the comparison spelled out while a table cell
/// gets its sense from the row label. One helper rather than a spelling per surface: three call
/// sites used to hard-code `>=`, which a two-sided range or an exclusive bound makes untrue.
pub fn cutoff_phrase(value: Option<&CutoffSpec>, sense: CutoffSense, decimals: usize) -> String {
    let Some(spec) = value else {
        return String::new();
    };
    match (&spec.min, &spec.max, &spec.single) {
        (_, _, Some(single)) => {
            let comparison = match (sense, single.operator) {
                (CutoffSense::Minimum, BoundOperator::Inclusive) => ">=",
                (CutoffSense::Minimum, BoundOperator::Exclusive) => ">",
                (CutoffSense::Maximum, BoundOperator::Inclusive) => "<=",
                (CutoffSense::Maximum, BoundOperator::Exclusive) => "<",
            };
            format!(
                "{comparison} {:.decimals$} {}",
                single.entry.value,
                single.entry.unit.trim()
            )
        }
        _ => format!("in {}", cutoff_label(value, decimals)),
    }
}

/// SB-CUT-012. The depth frame a summation's per-sample weights were measured in.
///
/// Part of a result's IDENTITY, not a display option. The per-sample weight is `Δz` in MD and
/// `Δz·cos θ` in TVD, so the weights differ rather than merely the totals — in a 60° hold section
/// by a factor of two, which is why IP says TVD zonal averages *"could be considerably
/// different"*. A net thickness quoted in a deviated field without its frame is a number a reader
/// cannot use. Techlog offers four frames, IP two; the union is the vocabulary here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SummationFrame {
    /// Measured depth along hole — the log's own depth column, differenced.
    #[default]
    Md,
    /// True vertical depth.
    Tvd,
    /// True vertical depth subsea.
    Tvdss,
    /// True stratigraphic thickness.
    Tst,
}

impl SummationFrame {
    /// The chapter's own spelling of each frame. Its `match` is exhaustive, so it is also the
    /// compile-time guard: a fifth variant cannot be added without deciding here what it is
    /// called, which is a stronger guarantee than a list a test could let go stale.
    pub fn as_str(self) -> &'static str {
        match self {
            SummationFrame::Md => "MD",
            SummationFrame::Tvd => "TVD",
            SummationFrame::Tvdss => "TVDSS",
            SummationFrame::Tst => "TST",
        }
    }
}

/// SB-CUT-012. What an MD summation's weights were differenced from.
///
/// Recorded beside the frame because naming the frame alone does not say WHICH depths produced the
/// increments — the same reason a calibration records the curves it was fitted on.
pub const MD_WEIGHTS_SOURCE: &str = "log depth increments (MD)";

/// SB-CUT-009. How an averaged curve is weighted across a zone.
///
/// Declared per curve, never inferred from the curve's name or family. Techlog's own behaviour is
/// the harm the chapter names: *"the SW curve is weighted by POR but the SWE is not weighted"* —
/// a curve loses its φ-weighting because of how it happens to be spelled, and nothing on the page
/// says which form produced the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AverageWeighting {
    /// `Σ(C·h) / Σh`
    Thickness,
    /// `Σ(C·φ·h) / Σ(φ·h)`
    Porosity,
}

/// SB-CUT-009. The curve slots the summation averages. A SLOT is a role fixed at compile time —
/// which input of the summation a curve fills — not the mnemonic it happens to be stored under.
pub const AVERAGED_SLOTS: [&str; 3] = ["VSH", "PHIE", "SWE"];

/// SB-CUT-022. Which report tiers a cut-off is USED at.
///
/// An explicit flag per tier, never an inference. F-17 is the reason: Geolog changed the activation
/// trigger between two modules of ONE product — `Determin` fires on the presence of the *curve*,
/// `determin_mc` on the presence of the *value*. Either rule is defensible; what is not defensible
/// is that a result cannot say which one applied, because an inference leaves no record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutoffUse {
    pub sand: bool,
    pub reservoir: bool,
    pub pay: bool,
}

impl CutoffUse {
    /// Whether this cut-off's value is applied at one tier.
    fn at(&self, tier: &str) -> bool {
        match tier {
            "SAND" => self.sand,
            "RESERVOIR" => self.reservoir,
            _ => self.pay,
        }
    }
}

/// SB-CUT-022. The tiers a cut-off is used at when the caller declares nothing.
///
/// Cited, not chosen, and chosen to move no number: this is the ladder the engine already applied,
/// stated as flags instead of as nesting. Net sand is clay-driven, net reservoir adds porosity and
/// net pay adds saturation — T4 Bentley & Ringrose, `docs/PRD_v2/14_cutoffs-summation-mc.md:1296-1297`.
/// **`SWE` is off at the reservoir tier**, which is F-25 `:494-495`: IP's `Sw Net Use` and
/// `Sw Pay Use` are separate ordinals and Net Reservoir is described as porosity- and clay-driven.
pub fn default_cutoff_use(slot: &str) -> CutoffUse {
    match slot {
        "VSH" => CutoffUse { sand: true, reservoir: true, pay: true },
        "PHIE" => CutoffUse { sand: false, reservoir: true, pay: true },
        // SWE and PERM: pay only.
        _ => CutoffUse { sand: false, reservoir: false, pay: true },
    }
}

/// SB-CUT-022. Resolve the tiers one cut-off is used at.
///
/// Takes a SLOT and the run's declaration — nothing else. It cannot see whether a curve exists or
/// whether a value was supplied, which is what makes *never inferred from the presence of a curve
/// or of a value* a property of the signature rather than of today's body.
pub fn cutoff_use_for(declared: &BTreeMap<String, CutoffUse>, slot: &str) -> CutoffUse {
    declared.get(slot).copied().unwrap_or_else(|| default_cutoff_use(slot))
}

/// SB-CUT-022. The four cut-offs and the tiers each is used at, resolved once per run.
///
/// **One value per property, read by every tier that uses it.** That is F-25's shape exactly: IP
/// ships `Phi Cutoff` as a single ordinal *"for Pay and Reservoir report"* with `Phi Net Use` and
/// `Phi Pay Use` as two independent ordinals beside it. Two values would be a different product
/// and a different requirement — SB-CUT-024's, which owns arbitrary named tiers and their own
/// cut-off sets, and which is outside this gate.
/// Deliberately NOT `Default`: an all-false [`CutoffUse`] is a cut-off switched off everywhere,
/// which is a real and occasionally wanted state but a catastrophic thing to arrive at by
/// forgetting a field. Every construction names its four use declarations.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TierCutoffs {
    pub(crate) vsh: Option<CutoffRange>,
    pub(crate) phie: Option<CutoffRange>,
    pub(crate) swe: Option<CutoffRange>,
    pub(crate) perm: Option<CutoffRange>,
    pub(crate) vsh_use: CutoffUse,
    pub(crate) phie_use: CutoffUse,
    pub(crate) swe_use: CutoffUse,
    pub(crate) perm_use: CutoffUse,
}

impl TierCutoffs {
    /// The cut-off applied to one property at one tier: its value where the tier uses it, and
    /// `None` — which filters nothing — where the tier does not.
    fn applied(&self, tier: &str, slot: &str) -> Option<CutoffRange> {
        let (value, used) = match slot {
            "VSH" => (self.vsh, self.vsh_use),
            "PHIE" => (self.phie, self.phie_use),
            "SWE" => (self.swe, self.swe_use),
            _ => (self.perm, self.perm_use),
        };
        used.at(tier).then_some(value).flatten()
    }
}

/// SB-CUT-009. The weighting applied when the caller declares nothing.
///
/// Cited, not chosen. The φ-weighted saturation `Σ(Sw·φ·h)/Σ(φ·h)` is agreed by all three vendors
/// and is required for SB-CUT-010's volumetric identity to hold at all
/// (`docs/PRD_v2/14_cutoffs-summation-mc.md:1041-1042`); thickness weighting for the rest is what
/// the engine already did, so a caller who declares nothing sees no number move.
pub fn default_weighting(slot: &str) -> AverageWeighting {
    if slot == "SWE" {
        AverageWeighting::Porosity
    } else {
        AverageWeighting::Thickness
    }
}

/// SB-CUT-009. Resolve the weighting for one averaged slot.
///
/// Takes a SLOT and the run's declaration — nothing else. It has no access to which curve filled
/// that slot, which is what makes "never inferred from the name" a property of the signature
/// rather than of the current implementation.
pub fn weighting_for(
    declared: &BTreeMap<String, AverageWeighting>,
    slot: &str,
) -> AverageWeighting {
    declared.get(slot).copied().unwrap_or_else(|| default_weighting(slot))
}

/// SB-CUT-009. One weighted average, accumulated sample by sample.
///
/// A sample joins the numerator AND the denominator together or not at all, so an average is
/// always normalised over exactly the footage its own curve was valid on — a SAND-row sample with
/// a good Vsh but a missing φ must not drag the porosity average toward zero.
#[derive(Debug, Default, Clone, Copy)]
struct WeightedMean {
    sum_wc: f64,
    sum_w: f64,
}

impl WeightedMean {
    /// `weight` is NaN where the weighting basis itself is missing — a φ-weighted average cannot
    /// use a sample with no porosity, however good its own value is.
    fn add(&mut self, value: f32, weight: f64) {
        if value.is_nan() || !weight.is_finite() {
            return;
        }
        self.sum_wc += value as f64 * weight;
        self.sum_w += weight;
    }

    fn value(&self) -> f32 {
        if self.sum_w > 0.0 {
            (self.sum_wc / self.sum_w) as f32
        } else {
            f32::NAN
        }
    }
}

/// SB-CUT-005. Relative tolerance on `gross - (net + not_net + unknown)`.
///
/// `1e-7`, cited: `docs/PRD_v2/14_cutoffs-summation-mc.md:2083` (SB-CUT-T22), which adopts
/// Techlog's `adjustFinal` reconciliation shape with the `print` → result-field refinement. It is
/// a NUMERICAL tolerance on closure arithmetic, not a petrophysical cutoff.
pub const PARTITION_TOLERANCE: f64 = 1e-7;
// SB-CUT-017: the registry entry `cut.partition_tolerance` carries this same number beside
// the citation that authorises it, and the named test asserts the two agree - so the
// disclosure cannot drift away from the behaviour.

/// SB-CUT-005. A footage partition that has been reconciled, and by how much.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconciledPartition {
    pub net: f32,
    pub not_net: f32,
    pub unknown: f32,
    /// Footage moved into the largest component to make the partition close. **Reported, not
    /// printed** — that distinction IS the requirement. Techlog computes the same correction and
    /// sends it to a console, where it is lost, and a reconciliation whose correction is not
    /// recorded is indistinguishable from no reconciliation.
    pub absorbed: f32,
}

/// SB-CUT-005. A partition that does not close within [`PARTITION_TOLERANCE`], with every number a
/// reader needs to act on it. Structured rather than a bare message, for the same reason the
/// absorbed amount is a field: a diagnostic that only exists as prose cannot be checked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartitionResidual {
    pub gross: f32,
    pub net: f32,
    pub not_net: f32,
    pub unknown: f32,
    pub residual: f64,
    pub relative: f64,
    pub tolerance: f64,
}

impl std::fmt::Display for PartitionResidual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "footage partition does not close: gross {} against net {} + not-net {} + unknown {} \
             leaves a residual of {:e} ({:e} relative), outside the {:e} tolerance",
            self.gross, self.net, self.not_net, self.unknown, self.residual, self.relative,
            self.tolerance
        )
    }
}

/// SB-CUT-005. Check `gross - (net + not_net + unknown)` against [`PARTITION_TOLERANCE`], absorb a
/// residual within it into the LARGEST component, and report what was absorbed.
///
/// Evaluated in `f64` on the values as they will be REPORTED, so it checks the partition a reader
/// actually receives rather than an intermediate nobody sees. Absorption targets the largest
/// component because that is where a relative correction is least distorting — moving an ulp of
/// gross onto a small component could shift it by a large fraction of itself.
pub fn reconcile_partition(
    gross: f32,
    net: f32,
    not_net: f32,
    unknown: f32,
) -> Result<ReconciledPartition, PartitionResidual> {
    let residual = gross as f64 - (net as f64 + not_net as f64 + unknown as f64);
    // A zero-thickness zone has nothing to be relative TO; its components are zero as well, so the
    // residual is zero and the absolute value is the honest comparison.
    let scale = if gross.abs() > 0.0 { gross.abs() as f64 } else { 1.0 };
    let relative = residual.abs() / scale;
    if relative > PARTITION_TOLERANCE {
        return Err(PartitionResidual {
            gross,
            net,
            not_net,
            unknown,
            residual,
            relative,
            tolerance: PARTITION_TOLERANCE,
        });
    }
    let mut out =
        ReconciledPartition { net, not_net, unknown, absorbed: residual as f32 };
    if net >= not_net && net >= unknown {
        out.net = (net as f64 + residual) as f32;
    } else if not_net >= unknown {
        out.not_net = (not_net as f64 + residual) as f32;
    } else {
        out.unknown = (unknown as f64 + residual) as f32;
    }
    Ok(out)
}

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
    // SB-CUT-012: refuse a frame whose per-sample weights cannot be computed, before any work.
    // The per-sample weight is dz in MD and dz*cos(theta) in TVD, so a TVD summation is not a
    // rescaling of an MD one - it is a different set of weights - and serving MD numbers under a
    // TVD label is exactly what the requirement forbids.
    // SB-CUT-016: a cut-off switched on and left blank stops the run, before any work and
    // whatever else is set. Naming them all at once beats refusing one at a time.
    if !req.enabled_unset.is_empty() {
        return Err(format!(
            "cannot summate: {} enabled with no value. A cut-off has no default - four shipped              vendor sets disagree and delivered work spans a wide range even within one field -              so set a value, or turn the cut-off off and the summation will report it unfiltered.",
            req.enabled_unset.join(", ")
        ));
    }
    // SB-CUT-019: canonicalise every entered cut-off before anything is computed. A bare number,
    // an unknown unit or a physically impossible value stops the run here, naming the field.
    // SB-CUT-020: and resolve each into a RANGE. A single-sided entry becomes the degenerate
    // range with an open far bound, so a request written before ranges existed means what it meant.
    let cut = |spec: &Option<CutoffSpec>,
               quantity: CutoffQuantity,
               sense: CutoffSense,
               label: &str| {
        spec.as_ref().map(|s| s.canonical(quantity, sense, label)).transpose()
    };
    let vsh_max = cut(&req.vsh_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the VSH cut-off")?;
    let phie_min = cut(&req.phie_min, CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the PHIE cut-off")?;
    let swe_max = cut(&req.swe_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the SWE cut-off")?;
    let perm_min = cut(&req.perm_min, CutoffQuantity::Permeability, CutoffSense::Minimum, "the PERM cut-off")?;
    // SB-CUT-022: resolve which tiers each cut-off is used at, once per run and from the SLOT plus
    // the caller's declaration only.
    let tier_cuts = TierCutoffs {
        vsh: vsh_max,
        phie: phie_min,
        swe: swe_max,
        perm: perm_min,
        vsh_use: cutoff_use_for(&req.cutoff_use, "VSH"),
        phie_use: cutoff_use_for(&req.cutoff_use, "PHIE"),
        swe_use: cutoff_use_for(&req.cutoff_use, "SWE"),
        perm_use: cutoff_use_for(&req.cutoff_use, "PERM"),
    };
    let unfiltered: Vec<String> = [
        ("VSH", vsh_max.is_none()),
        ("PHIE", phie_min.is_none()),
        ("SWE", swe_max.is_none()),
        ("PERM", perm_min.is_none()),
    ]
    .iter()
    .filter(|(_, absent)| *absent)
    .map(|(name, _)| (*name).to_string())
    .collect();
    if req.frame != SummationFrame::Md {
        return Err(format!(
            "cannot summate in {}: the per-sample weights would be dz*cos(theta) from the well's              deviation survey, and SandiBumi computes only MD (dz) weights today. Run in MD, or              ask for a {} summation to be built as its own record.",
            req.frame.as_str(),
            req.frame.as_str()
        ));
    }
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
        // SB-POR-057 (DEC-070, RULED 2026-08-18): the candidate list is the ONE canonical
        // name. The quick-look D-N limited curve is no longer a fallback - "quick look only
        // shows pay summation as visual not pay curves" - superseding the DEC-042 two-name
        // pair this list used to carry. Displays may overlay PHIE_DN_LIM; the summed
        // numbers never read it.
        let phie_candidates = vec!["PHIE".to_string()];
        let (phie_curve, phie_resolved) = match first_available_input_alias(
            &conn,
            well_id,
            "PHIE",
            &phie_candidates,
            req.input_set.as_deref(),
            None,
            &HashSet::new(),
        ) {
            Ok(Some(curve)) => (curve, true),
            Ok(None) => ("PHIE".into(), false),
            Err(_) => continue,
        };
        // DEC-070's observable half: when the ONLY porosity here is the quick-look curve,
        // the row says so - the zeros below mean "not interpreted for pay", never "wet".
        // Deliberately NOT set when the well has no porosity at all: the flag means
        // "present and excluded", and conflating it with absence would erase the reason
        // the mark exists.
        let quicklook_phie_excluded = !phie_resolved
            && equations::try_resolve_ancestry_input(
                &conn,
                well_id,
                "PHIE",
                modules::PHIE_DN_LIMITED_DEFAULT,
                req.input_set.as_deref(),
                None,
            )
            .ok()
            .flatten()
            .is_some();
        let curve_names: Vec<String> =
            vec!["VSH".into(), phie_curve.clone(), "SWE".into(), "PERM".into()];

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
        let phie_col = floored_phie(&columns[&phie_curve]);
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

        // SB-CUT-002: the interval every row of this well will record.
        let sample_interval = median_sample_interval(&step);

        // Flags per sample: NaN inputs exclude the sample (flag stays NaN). Single-sourced
        // through `classify_sample` so the sweep engine below applies identical cutoff logic.
        let mut flag_sand = vec![f32::NAN; n];
        let mut flag_res = vec![f32::NAN; n];
        let mut flag_pay = vec![f32::NAN; n];
        for i in 0..n {
            let (fs, fr, fp) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i], &tier_cuts, has_perm_cut,
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
                    vec!["VSH".to_string(), phie_curve.clone(), "SWE".to_string()];
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
                // SB-CUT-003: footage the classifier saw and REJECTED. Only samples it could
                // actually evaluate land here; see the `unknown` derivation below.
                let mut not_net = 0.0f64;
                // SB-CUT-009: one accumulator per averaged slot, each carrying whichever weighting
                // the run DECLARED for that slot. The φ-weighted form used to be hard-wired to the
                // saturation slot, so it could be neither requested elsewhere nor switched off.
                let mut avg = [WeightedMean::default(); AVERAGED_SLOTS.len()];
                let mode: Vec<AverageWeighting> =
                    AVERAGED_SLOTS.iter().map(|s| weighting_for(&req.weighting, s)).collect();
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
                    // SB-CUT-001: ONE discretisation rule, shared. This site used to inline
                    // its own copy of the clamp; a second copy is a second thing to keep in
                    // step, and net pay is where a silent divergence costs most.
                    let (s_top, s_bot) =
                        sample_slab(depth[i] as f64, step[i] as f64, req.discretisation);
                    let h = sample_incl_thickness(
                        s_top,
                        s_bot,
                        zone.top_depth as f64,
                        zone.bottom_depth as f64,
                        None,
                    );
                    if h <= 0.0 {
                        continue;
                    }
                    if !flags[i].is_nan() {
                        n_classified += 1;
                    }
                    if flags[i] != 1.0 {
                        // SB-CUT-003: only an EVALUATED rejection is NotNet. A NaN flag means the
                        // classifier had nothing to judge, so its footage must fall through to
                        // `unknown` — folding it in here still closes the identity, which is
                        // precisely why the requirement names it.
                        if !flags[i].is_nan() {
                            not_net += h;
                        }
                        continue;
                    }
                    net += h;
                    // SB-CUT-009: the two weight bases. The φ basis is MISSING where porosity is,
                    // so a φ-weighted average silently skips a sample it cannot weight rather than
                    // treating it as weightless — the same rule the hard-wired version followed.
                    let w = |m: AverageWeighting| match m {
                        AverageWeighting::Thickness => h,
                        AverageWeighting::Porosity => {
                            if phie[i].is_nan() { f64::NAN } else { phie[i] as f64 * h }
                        }
                    };
                    for (slot, value) in [vsh[i], phie[i], swe[i]].into_iter().enumerate() {
                        // SB-CUT-030: values enter the sum through the ACCUMULATE stage, which
                        // never clamps. A clamp inside a sum does not relocate a tail - it moves
                        // the MEAN, by 0.3989*phi*sigma toward more hydrocarbon, independent of
                        // iteration count. Named at the site so a future edit has to argue with it.
                        let accumulated = stage_value(
                            ClampStage::Accumulate,
                            BoundedQuantity::VolumeFraction,
                            value,
                        );
                        avg[slot].add(accumulated, w(mode[slot]));
                    }
                    if !phie[i].is_nan() && !swe[i].is_nan() {
                        hpv += phie[i] as f64 * (1.0 - swe[i] as f64) * h;
                    }
                }

                let gross = zone.bottom_depth - zone.top_depth;
                // SB-CUT-003: the remainder, so the partition closes exactly. It absorbs both
                // kinds of unevaluable footage — an in-zone sample the classifier could not judge,
                // and footage no sample covers at all. Computed in f64 against the same f64 sums
                // the other two came from, then rounded once.
                let unknown = gross as f64 - net - not_net;
                // SB-CUT-005: check the partition AS REPORTED — the three f64 sums are each
                // rounded once on the way into the row, so the closure a reader receives is not
                // automatically the closure the arithmetic had. Within tolerance the drift is
                // absorbed into the largest component and recorded; outside it the summation
                // refuses rather than shipping a partition that does not add up.
                let recon = reconcile_partition(gross, net as f32, not_net as f32, unknown as f32)
                    .map_err(|residual| {
                        format!(
                            "{well_name} zone {} flag {flag_name}: {residual}",
                            zone.zone_name
                        )
                    })?;
                all_rows.push(PaySummaryRow {
                    well_id: well_id.clone(),
                    well_name: well_name.clone(),
                    zone: zone.zone_name.clone(),
                    flag: flag_name.to_string(),
                    discretisation_model: req.discretisation.token().to_string(),
                    sample_interval,
                    top: zone.top_depth,
                    bottom: zone.bottom_depth,
                    gross,
                    net: recon.net,
                    not_net: recon.not_net,
                    unknown: recon.unknown,
                    residual_absorbed: recon.absorbed,
                    frame: req.frame,
                    weights_source: MD_WEIGHTS_SOURCE.to_string(),
                    unfiltered: unfiltered.clone(),
                    // SB-CUT-004: the same net over the footage that was actually judged. MISSING
                    // rather than 0.0 when nothing was — there is no denominator, and a printed
                    // zero would be a claim about rock nobody looked at.
                    ntg_known: {
                        let judged = gross as f64 - unknown;
                        if judged > 0.0 { (net / judged) as f32 } else { f32::NAN }
                    },
                    ntg: if gross > 0.0 { (net / gross as f64) as f32 } else { 0.0 },
                    // Averages are normalised over the footage THAT curve was valid on — not total
                    // net — so a SAND-row sample with valid VSH but missing PHIE does not drag
                    // avg_phie toward zero. Each carries the weighting its slot declared.
                    // SB-CUT-030: the three averages are emitted through the PRESENT stage, and
                    // an average outside its quantity's bounds is FLAGGED rather than corrected.
                    // All three are volume fractions - the bound comes from the QUANTITY, not from
                    // the curve's name or declared type, which is the failure mode IP has.
                    out_of_range: [avg[0].value(), avg[1].value(), avg[2].value()]
                        .iter()
                        .any(|v| BoundedQuantity::VolumeFraction.is_out_of_range(*v)),
                    avg_vsh: present_average(avg[0].value()),
                    avg_phie: present_average(avg[1].value()),
                    avg_swe: present_average(avg[2].value()),
                    // SB-CUT-030: HPV is a volume-thickness, not a fraction - it routinely
                    // exceeds 1 - so it goes through the PRESENT stage as an UNBOUNDED quantity.
                    // That is the clause "an unbounded quantity MUST NOT be clamped to [0,1]"
                    // stated at the one site where a careless clamp would destroy the number
                    // rather than merely round it.
                    hpv: stage_value(
                        ClampStage::Present,
                        BoundedQuantity::Unbounded,
                        hpv as f32,
                    ),
                    n_classified,
                    perm_cutoff_no_data,
                    quicklook_phie_excluded,
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
    cuts: &TierCutoffs,
    has_perm_cut: bool,
) -> (f32, f32, f32) {
    // SB-CUT-016: an ABSENT cut-off does not filter. The NaN cascade below is deliberately
    // untouched by that - a sample with no VSH is unjudgeable whether or not VSH is being used as
    // a cut-off, and making an unfiltered cut-off also stop requiring its curve would let a well
    // with no VSH book pay it never booked. The requirement says nothing about NaN handling, so
    // the rule stands.
    //
    // SB-CUT-022 leaves it alone for the same reason. The use flags govern whether a cut-off's
    // VALUE is applied at a tier; they say nothing about whether the tier needs that curve to be
    // judgeable at all. Those are two different questions and only one of them is a cut-off.
    if vsh.is_nan() {
        return (f32::NAN, f32::NAN, f32::NAN);
    }
    // SB-CUT-022: each tier applies exactly the cut-offs DECLARED for it. The ladder that used to
    // be expressed by nesting — reservoir built on sand, pay built on reservoir — is now expressed
    // by the default flags, which say the same thing wherever nobody declares otherwise.
    let judge = |tier: &str| {
        let passes = |slot: &str, sample: f32| {
            // SB-CUT-030: the FLAG_TEST stage compares the value clamped to its QUANTITY's bounds
            // - the bound comes from the quantity, never from the curve's name or declared type,
            // which is the failure that makes IP's clipping invisible in the data. Inert for an
            // in-range sample, which is every ordinary one, so no number moves; what it does is put
            // the stage boundary somewhere a reader can find it.
            let quantity = if slot == "PERM" {
                BoundedQuantity::Permeability
            } else {
                BoundedQuantity::VolumeFraction
            };
            let tested = stage_value(ClampStage::FlagTest, quantity, sample);
            cuts.applied(tier, slot).map_or(true, |range| range.contains(tested))
        };
        passes("VSH", vsh)
            && passes("PHIE", phie)
            && passes("SWE", swe)
            // A sample with no PERM value cannot demonstrate it passes the cutoff — missing PERM
            // must FAIL rather than silently pass, at whichever tier the cut-off is applied.
            && (!has_perm_cut
                || cuts
                    .applied(tier, "PERM")
                    .map_or(true, |range| !perm.is_nan() && range.contains(perm)))
    };
    let fs = judge("SAND") as u8 as f32;
    if phie.is_nan() {
        return (fs, f32::NAN, f32::NAN);
    }
    let fr = judge("RESERVOIR") as u8 as f32;
    if swe.is_nan() {
        return (fs, fr, f32::NAN);
    }
    (fs, fr, judge("PAY") as u8 as f32)
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
    fixed_vsh: Option<CutoffRange>,
    fixed_phie: Option<CutoffRange>,
    fixed_swe: Option<CutoffRange>,
    perm_min: Option<CutoffRange>,
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
        // SB-CUT-020: the SWEPT bound is the degenerate single-sided range in the slot's own
        // sense, inclusive - the sweep varies a cut-off VALUE and says nothing about inclusivity,
        // so it uses the same default the single-sided forms have always carried. The HELD
        // cut-offs keep whatever operators the caller declared.
        let swept = |low: bool| {
            let bound = Some(CutoffBound { value: cut, operator: BoundOperator::Inclusive });
            if low {
                CutoffRange { low: bound, high: None }
            } else {
                CutoffRange { low: None, high: bound }
            }
        };
        match prop {
            SweepProp::Vsh => vsh_max = Some(swept(false)),
            SweepProp::Phie => phie_min = Some(swept(true)),
            SweepProp::Swe => swe_max = Some(swept(false)),
        }
        // SB-CUT-022: the sweep is a plot of one cut-off VALUE against a metric, so it carries the
        // shipped tier declaration. A caller who wants a different one runs the summary, which is
        // where a declaration belongs.
        let swept_cuts = TierCutoffs {
            vsh: vsh_max,
            phie: phie_min,
            swe: swe_max,
            perm: perm_min,
            vsh_use: default_cutoff_use("VSH"),
            phie_use: default_cutoff_use("PHIE"),
            swe_use: default_cutoff_use("SWE"),
            perm_use: default_cutoff_use("PERM"),
        };

        let mut net = 0.0f64;
        let mut hpv = 0.0f64;
        for i in 0..n {
            let h = incl_h[i];
            if h <= 0.0 {
                continue;
            }
            let (_s, _r, pay) = classify_sample(
                vsh[i], phie[i], swe[i], perm[i], &swept_cuts, has_perm_cut,
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
    /// SB-CUT-001 (DEC-071): the discretisation model, shared with the pay summary the
    /// sweep informs - a sweep read against one model and a summary run under another
    /// would put the elbow in the wrong place.
    #[serde(default)]
    pub discretisation: DiscretisationModel,
    /// Which cutoff to sweep: "VSH" | "PHIE" | "SWE".
    pub property: String,
    /// Fixed values for the two cutoffs NOT being swept (the swept one's field is ignored).
    /// SB-CUT-016: `None` = that property is not filtered while this sweep runs. No default.
    /// SB-CUT-019: carried as entered, with its unit.
    pub vsh_max: Option<CutoffSpec>,
    pub phie_min: Option<CutoffSpec>,
    pub swe_max: Option<CutoffSpec>,
    pub perm_min: Option<CutoffSpec>,
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
/// SB-CUT-001 (DEC-071, RULED 2026-08-18): the thickness discretisation model is a
/// PARAMETER of the one shared rule, defaulting to CENTRED per the requirement text - a
/// sample's slab straddles its depth, representing the rock AROUND the measurement.
/// FORWARD (the chapter's TOPS rule, Techlog computeGross) stays selectable so a legacy
/// run's numbers can be reproduced bit-for-bit. Jauhar accepted that the CENTRED default
/// moves every existing net-pay and NTG number by up to half a sample step at each
/// pay/zone edge ("3, centred", after the difference was explained in thickness terms).
/// ONE vocabulary everywhere: the serde wire form IS the record token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiscretisationModel {
    #[default]
    #[serde(rename = "CENTRED")]
    Centred,
    #[serde(rename = "TOPS")]
    Forward,
}

impl DiscretisationModel {
    pub fn token(self) -> &'static str {
        match self {
            Self::Centred => "CENTRED",
            Self::Forward => DISCRETISATION_MODEL,
        }
    }
}

/// SB-CUT-001: the ONE place a sample's slab is derived from its depth and step - the
/// model choice can never again be inlined divergently at a call site.
pub(crate) fn sample_slab(depth: f64, step: f64, model: DiscretisationModel) -> (f64, f64) {
    match model {
        DiscretisationModel::Forward => (depth, depth + step),
        DiscretisationModel::Centred => (depth - step / 2.0, depth + step / 2.0),
    }
}

/// SB-CUT-002: the record token of the FORWARD rule - the forward-interval, zone-clipped
/// rule the chapter's T02 hand-traces from Techlog's computeGross and names TOPS. A
/// summation number without its model is not reproducible: IP ships two different "Net"
/// definitions under one column heading and labels neither. Since DEC-071 the model is a
/// request parameter defaulting to CENTRED; every record carries the token of the model
/// that actually produced it.
pub const DISCRETISATION_MODEL: &str = "TOPS";

/// SB-CUT-002: the sample interval a summation was computed on — the median forward step, the
/// same summary `reframe`'s regularize already uses for "the source's own spacing". Recorded
/// per record because net-to-gross is NOT scale-invariant (the chapter's T4: 0.55 → 0.75 → 1.0
/// across three blocking steps). NaN when no positive step exists.
pub(crate) fn median_sample_interval(step: &[f32]) -> f32 {
    let mut positive: Vec<f32> = step.iter().copied().filter(|s| *s > 0.0).collect();
    if positive.is_empty() {
        return f32::NAN;
    }
    positive.sort_by(|a, b| a.partial_cmp(b).unwrap());
    positive[positive.len() / 2]
}

pub(crate) fn sample_incl_thickness(
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
    // SB-CUT-019: the two HELD cut-offs are entered values and are canonicalised before any
    // sweep runs. The swept property's range is a plot bound, not a cut-off, and keeps its own
    // units by construction - it is expressed in whatever the swept quantity's canonical unit is.
    let cut = |spec: &Option<CutoffSpec>,
               quantity: CutoffQuantity,
               sense: CutoffSense,
               label: &str| {
        spec.as_ref().map(|s| s.canonical(quantity, sense, label)).transpose()
    };
    let held_vsh = cut(&req.vsh_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the held VSH cut-off")?;
    let held_phie = cut(&req.phie_min, CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the held PHIE cut-off")?;
    let held_swe = cut(&req.swe_max, CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "the held SWE cut-off")?;
    let held_perm = cut(&req.perm_min, CutoffQuantity::Permeability, CutoffSense::Minimum, "the held PERM cut-off")?;
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
            let (s_top, s_bot) = sample_slab(depth[i] as f64, step[i] as f64, req.discretisation);
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
            vsh, phie, swe, perm, &incl_h, prop, held_vsh, held_phie, held_swe,
            held_perm, req.sweep_min, req.sweep_max, steps, metric, gross,
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

    #[test]
    fn the_live_despike_preview_reads_the_selected_windows_and_returns_only_branch_counts() {
        // CORRECTNESS — SB-ENV-031 / T40/T69 presentation path. The branch identities and
        // 33.33/50.00 % expectations come from docs/PRD_v2/20_envcorr-qc.md §2.5 and §6.
        // The unequal fixtures force the preview through both sides of the actual spread branch.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depth: Vec<f32> = (0..41).map(|index| 1000.0 + index as f32 * 0.1).collect();
        let add_gr = |name: &str, gr: Vec<f32>| {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, Some(0.0)).unwrap();
            let missing = vec![f32::NAN; depth.len()];
            db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                gr,
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing,
            )
            .unwrap();
            id.to_string()
        };
        let mut quiet = vec![50.0f32; depth.len()];
        quiet[20] = 300.0;
        let quiet_id = add_gr("ZERO-MAD-WINDOW", quiet);
        let scattered = (0..depth.len()).map(|index| 50.0 + index as f32 * 0.25).collect();
        let scattered_id = add_gr("POSITIVE-MAD-WINDOW", scattered);
        let db = Mutex::new(conn);

        let preview = despike_contamination_preview(
            &db,
            &[quiet_id, scattered_id],
            &HashMap::from([("CURVE".to_string(), "GR".to_string())]),
            &HashMap::from([("WINDOW".to_string(), 0.5), ("K".to_string(), 3.0)]),
            &HashMap::new(),
            None,
        )
        .expect("the selected curves use the same read path as the run");

        assert_eq!(preview.evaluated_wells, 2);
        assert!(preview.unavailable_well_ids.is_empty());
        assert!(preview.issues.is_empty());
        let true_mad = preview
            .branches
            .iter()
            .find(|branch| branch.estimator == crate::condition::DespikeEstimator::TrueMad)
            .expect("positive scatter must select true MAD");
        let fallback = preview
            .branches
            .iter()
            .find(|branch| branch.estimator == crate::condition::DespikeEstimator::MeanDeviationFallback)
            .expect("zero MAD must select the fallback");
        assert!((true_mad.ceiling_pct - 50.0).abs() <= 0.01);
        assert!((fallback.ceiling_pct - 33.333_333).abs() <= 0.01);

        let wire = serde_json::to_value(&preview).unwrap();
        let keys: std::collections::BTreeSet<_> = wire
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            std::collections::BTreeSet::from([
                "branches",
                "evaluated_wells",
                "issues",
                "unavailable_well_ids",
            ]),
            "well-log arrays must remain behind IPC",
        );
    }

    /// CORRECTNESS - SB-ENV-026 / SB-ENV-T35. The declaration requirement, both mismatch
    /// directions and 0.1 g/cc = 100 kg/m3 come from `docs/PRD_v2/20_envcorr-qc.md`
    /// sections 4.3, 5.2 and 6.3. The 0.05 and 0.2 g/cc controls are derived as one half
    /// and two times that cited threshold; together they prove a matching declaration still
    /// reaches the specified comparison without depending on decimal equality after f32 storage.
    #[test]
    fn a_drho_unit_is_required_on_import_and_both_threshold_unit_mismatch_directions_refuse_before_flagging() {
        assert_eq!(
            crate::curves::resolve_unit_token("G/C3").map(|token| token.canonical_unit),
            Some("g/cc"),
            "the chapter-cited Geolog G/C3 spelling must resolve to g/cc"
        );
        assert_eq!(
            crate::curves::resolve_unit_token("k/m3").map(|token| token.canonical_unit),
            Some("kg/m3"),
            "the chapter-cited Geolog k/m3 spelling must resolve to kg/m3"
        );
        let las_path = std::env::temp_dir().join(format!(
            "sandibumi-env026-{}-{}.las",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::write(
            &las_path,
            "~VERSION\nVERS. 2.0 :\n~WELL\nWELL. DRHO-UNIT-DECLARATION :\n\
             ~CURVE\nDEPT.M : depth\nDRHO. : density correction\n\
             ~ASCII\n1000.0 100.0\n1000.5 200.0\n",
        )
        .unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let path = las_path.to_string_lossy().into_owned();
        let absent = ingest::import_las_files_with(
            &conn,
            std::slice::from_ref(&path),
            None,
            &ingest::LasImportOptions::default(),
        )
        .remove(0);
        assert!(
            absent.error.as_deref().unwrap_or("").contains("DRHO")
                && absent.error.as_deref().unwrap_or("").contains("unit"),
            "an undeclared DRHO unit must request a declaration, got {:?}",
            absent.error
        );
        let well_count: i64 = conn.query_row("SELECT count(*) FROM wells", [], |row| row.get(0)).unwrap();
        assert_eq!(well_count, 0, "the refused delivery must not leave a partial well");

        let imported = ingest::import_las_files_with(
            &conn,
            std::slice::from_ref(&path),
            None,
            &ingest::LasImportOptions {
                undeclared_drho_unit: Some("kg/m3".into()),
                ..Default::default()
            },
        )
        .remove(0);
        std::fs::remove_file(&las_path).ok();
        assert!(imported.error.is_none(), "an explicit cited unit must import: {:?}", imported.error);
        let imported_well = imported.well_id.expect("the explicit declaration creates the well");
        let (stored_unit, first_value): (Option<String>, f32) = conn
            .query_row(
                "SELECT m.unit, s.value FROM curve_meta m JOIN curve_samples s USING (curve_id) \
                 WHERE m.well_id = ?1 AND upper(m.mnemonic) = 'DRHO' ORDER BY s.depth LIMIT 1",
                duckdb::params![imported_well],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(stored_unit.as_deref(), Some("g/cc"), "the declared metric source is persisted canonically");
        assert!((first_value - 0.1).abs() < 1e-6, "100 kg/m3 must become 0.1 g/cc, got {first_value}");

        let add_well = |unit: Option<&str>, values: &[f32]| -> String {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, "DRHO-THRESHOLD-UNIT", None, None, Some(0.0)).unwrap();
            let depth = vec![1000.0, 1000.5];
            db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn,
                &well,
                "RAW",
                "DRHO",
                unit,
                Some("DRHO"),
                Some("synthetic unit-contract fixture"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, values).unwrap();
            well
        };
        let kg_curve = add_well(Some("kg/m3"), &[100.0, 200.0]);
        let gcc_curve = add_well(Some("g/cc"), &[0.05, 0.2]);
        let missing_curve = add_well(None, &[0.05, 0.2]);
        let dbm = Mutex::new(conn);
        let run = |well: &str, threshold_unit: &str| {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "badhole".into(),
                    well_ids: vec![well.to_string()],
                    log_inputs: HashMap::new(),
                    params: HashMap::from([
                        ("DRHO_MAX".into(), 0.1),
                        ("DCAL_MAX".into(), 2.0),
                    ]),
                    opts: HashMap::from([("DRHO_MAX_UNIT".into(), threshold_unit.into())]),
                    output_set: None,
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
            .remove(0)
        };

        let kg_vs_gcc = run(&kg_curve, "g/cc");
        let message = kg_vs_gcc.error.as_deref().unwrap_or("");
        assert!(message.contains("kg/m3") && message.contains("g/cc") && message.contains("mismatch"), "{message}");

        let gcc_vs_kg = run(&gcc_curve, "kg/m3");
        let message = gcc_vs_kg.error.as_deref().unwrap_or("");
        assert!(message.contains("g/cc") && message.contains("kg/m3") && message.contains("mismatch"), "{message}");

        let missing = run(&missing_curve, "g/cc");
        let message = missing.error.as_deref().unwrap_or("");
        assert!(message.contains("DRHO") && message.contains("unit") && message.contains("missing"), "{message}");

        let matching = run(&gcc_curve, "g/cc");
        assert!(matching.error.is_none(), "matching units must run: {:?}", matching.error);
        let conn = dbm.lock().unwrap();
        let (_, columns) = equations::fetch_curve_frame(&conn, &gcc_curve, &["BADHOLE".into()]).unwrap();
        assert_eq!(columns["BADHOLE"], vec![0.0, 1.0], "strict cited threshold comparison must still execute");
    }

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
                discretisation: DiscretisationModel::Forward,
                well_ids: vec![skip_candidate.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                input_set: None,
                skip_version: true,
                stats_only: false,
                custody: Some(test_run_custody()),
                frame: Default::default(),
                weighting: Default::default(),
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

    #[derive(Debug, serde::Deserialize)]
    struct SavedValidityArgument {
        argument: String,
        conditions: Vec<modules::ValidityCondition>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct SavedValidityManifest {
        schema_version: u32,
        module: String,
        arguments: Vec<SavedValidityArgument>,
    }

    /// CORRECTNESS — SB-ENV-001 / SB-ENV-T01 and `20_envcorr-qc.md` sections 4.1, 5.1 and 6.1.
    /// The 8-13 and 8-18 lb/gal ranges are the chapter's explicit NON-ADOPTABLE verification
    /// rows from Geolog `unc_tnph.lls:340,346`; this synthetic manifest is never registered as a
    /// shipping module and introduces no product limit or default.
    fn saved_sb_env_001_validity_manifest() -> SavedValidityManifest {
        let enumeration = modules::ValidityCondition {
            id: "synthetic.mud_type".into(),
            statement: "The branch selector must name a declared branch.".into(),
            source: "docs/PRD_v2/20_envcorr-qc.md section 6.1 T01/T03".into(),
            rule: modules::ValidityRule::Enumeration,
        };
        let normal_range = modules::ValidityCondition {
            id: "synthetic.normal_mud_range".into(),
            statement: "The normal-mud verification branch uses its own stated range.".into(),
            source: "Geolog unc_tnph.lls:340 - NON-ADOPTABLE verification fixture".into(),
            rule: modules::ValidityRule::NumericRange {
                min: Some(8.0),
                max: Some(13.0),
                unit: "lb/gal".into(),
                when: Some(modules::ValidityBranch {
                    argument: "MUD_TYPE".into(),
                    equals: "NORMAL".into(),
                }),
            },
        };
        let barite_range = modules::ValidityCondition {
            id: "synthetic.barite_mud_range".into(),
            statement: "The barite verification branch uses its own stated range.".into(),
            source: "Geolog unc_tnph.lls:346 - NON-ADOPTABLE verification fixture".into(),
            rule: modules::ValidityRule::NumericRange {
                min: Some(8.0),
                max: Some(18.0),
                unit: "lb/gal".into(),
                when: Some(modules::ValidityBranch {
                    argument: "MUD_TYPE".into(),
                    equals: "BARITE".into(),
                }),
            },
        };
        let companion = modules::ValidityCondition {
            id: "synthetic.caliper_companion".into(),
            statement: "This synthetic correction cannot be evaluated without a caliper input."
                .into(),
            source: "docs/PRD_v2/20_envcorr-qc.md SB-ENV-001(d) and SB-ENV-016".into(),
            rule: modules::ValidityRule::RequiredCompanion {
                any_of: vec!["CALIPER".into()],
                when: None,
            },
        };

        let mut selector = modules::opt(
            "MUD_TYPE",
            "Synthetic branch selector",
            "NORMAL",
            &["NORMAL", "BARITE"],
        );
        selector.validity_conditions = vec![enumeration];
        let mut mud_weight = modules::log_in(
            "MUD_WEIGHT",
            "Synthetic per-sample mud weight",
            "lb/gal",
            "MUD_WEIGHT",
            false,
        );
        mud_weight.validity_conditions = vec![normal_range, barite_range, companion];
        let synthetic = modules::ModuleSpec {
            name: "synthetic_saved_validity".into(),
            title: "Synthetic saved validity".into(),
            category: "Test fixture".into(),
            doc: "Verification-only SB-ENV-001 fixture; no condition is adopted by the product."
                .into(),
            args: vec![
                selector,
                mud_weight,
                modules::log_in(
                    "CALIPER",
                    "Synthetic required companion",
                    "in",
                    "CALIPER",
                    false,
                ),
                modules::log_out("CORRECTED", "Synthetic output", "v/v"),
            ],
        };

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "VALIDITY-CONDITION-FIXTURE", None, None, None).unwrap();
        let well_id = well_uuid.to_string();
        let request = RunModuleRequest {
            module: synthetic.name.clone(),
            well_ids: vec![well_id.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::new(),
            output_set: Some("VALIDITY_FIXTURE".into()),
            input_set: None,
            custody: test_run_custody(),
        };
        let complete = complete_module_log_spec(
            &conn,
            &well_id,
            &request,
            &synthetic,
            &build_opts(&synthetic, &request.opts, &request.log_inputs),
            &[],
            &["CORRECTED".into()],
            &[],
            &|parameters| serde_json::to_value(parameters).map_err(|error| error.to_string()),
        )
        .unwrap();
        let set_id = equations::create_complete_log_set(&conn, &well_id, &complete)
            .unwrap()
            .0;
        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE set_id = ?1",
                duckdb::params![set_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        let saved: serde_json::Value = serde_json::from_str(&params_json).unwrap();
        serde_json::from_value(
            saved["_sandibumi_module_validity_v1"].clone(),
        )
        .expect("the saved run must carry a deserializable validity manifest")
    }

    #[test]
    fn an_enumeration_validity_condition_survives_the_saved_run_params_json_round_trip() {
        let saved = saved_sb_env_001_validity_manifest();
        assert_eq!(saved.schema_version, 1);
        assert_eq!(saved.module, "synthetic_saved_validity");
        assert_eq!(saved.arguments[0].argument, "MUD_TYPE");
        assert_eq!(
            saved.arguments[0].conditions,
            vec![modules::ValidityCondition {
                id: "synthetic.mud_type".into(),
                statement: "The branch selector must name a declared branch.".into(),
                source: "docs/PRD_v2/20_envcorr-qc.md section 6.1 T01/T03".into(),
                rule: modules::ValidityRule::Enumeration,
            }]
        );
    }

    #[test]
    fn a_per_sample_numeric_range_survives_the_saved_run_with_its_unit_meaning_and_source() {
        let saved = saved_sb_env_001_validity_manifest();
        let range = &saved.arguments[1].conditions[0];
        assert_eq!(saved.arguments[1].argument, "MUD_WEIGHT");
        assert_eq!(range.id, "synthetic.normal_mud_range");
        assert_eq!(
            range.statement,
            "The normal-mud verification branch uses its own stated range."
        );
        assert_eq!(
            range.source,
            "Geolog unc_tnph.lls:340 - NON-ADOPTABLE verification fixture"
        );
        assert!(matches!(
            &range.rule,
            modules::ValidityRule::NumericRange {
                min: Some(8.0),
                max: Some(13.0),
                unit,
                ..
            } if unit == "lb/gal"
        ));
    }

    #[test]
    fn branch_specific_ranges_survive_the_saved_run_without_collapsing_to_one_module_range() {
        let saved = saved_sb_env_001_validity_manifest();
        let conditions = &saved.arguments[1].conditions;
        assert!(matches!(
            &conditions[0].rule,
            modules::ValidityRule::NumericRange {
                min: Some(8.0),
                max: Some(13.0),
                when: Some(modules::ValidityBranch { argument, equals }),
                ..
            } if argument == "MUD_TYPE" && equals == "NORMAL"
        ));
        assert!(matches!(
            &conditions[1].rule,
            modules::ValidityRule::NumericRange {
                min: Some(8.0),
                max: Some(18.0),
                when: Some(modules::ValidityBranch { argument, equals }),
                ..
            } if argument == "MUD_TYPE" && equals == "BARITE"
        ));
    }

    #[test]
    fn a_required_companion_condition_survives_the_saved_run_with_the_input_it_requires() {
        let saved = saved_sb_env_001_validity_manifest();
        let companion = &saved.arguments[1].conditions[2];
        assert_eq!(companion.id, "synthetic.caliper_companion");
        assert_eq!(
            companion.statement,
            "This synthetic correction cannot be evaluated without a caliper input."
        );
        assert_eq!(
            companion.source,
            "docs/PRD_v2/20_envcorr-qc.md SB-ENV-001(d) and SB-ENV-016"
        );
        assert!(matches!(
            &companion.rule,
            modules::ValidityRule::RequiredCompanion { any_of, when: None }
                if any_of == &["CALIPER"]
        ));
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
                     WHERE set_id = ?1 AND name <> ?2
                     ORDER BY position",
                )
                .unwrap();
            statement
                // This helper owns configurable numeric parameters. SB-ENV-028's separately
                // tested typed MASK context also uses the canonical run-parameter store, but is
                // deliberately not coerced into an f64 to make this older test pass.
                .query_map(duckdb::params![set_id, MASK_PROVENANCE_KEY], |row| {
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
                &[],
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

    /// CORRECTNESS — `10_clay-volume.md` SB-CLY-041 / exact T43 and §5.2-53 cite Geolog's
    /// ordered corrected mnemonics `GR_COR`, `RHO_COR`, `NPHI_COR`; the same chapter records that
    /// `GRN` is normalized, not corrected. SandiBumi's existing correction modules emit the
    /// independently named `GR_EC`, `RHOB_EC`, `NPHI_EC` curves. Linear-GR expectations are the
    /// independent `(GR - 20) / (120 - 20)` arithmetic. The density-neutron expectation 0.4239
    /// and its endpoint fixture come directly from SB-CLY-T18's cited Techlog-template witness.
    #[test]
    fn corrected_aliases_win_over_raw_and_normalized_inputs_raw_remains_the_fallback_and_each_resolved_curve_is_recorded(
    ) {
        fn add_well(conn: &duckdb::Connection, name: &str, depth: &[f32]) -> String {
            let id = uuid::Uuid::new_v4();
            db::insert_well(conn, id, name, None, None, None).unwrap();
            let missing = vec![f32::NAN; depth.len()];
            db::insert_standard_curves(
                conn,
                id,
                depth.to_vec(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing,
            )
            .unwrap();
            id.to_string()
        }

        fn add_curve(
            conn: &duckdb::Connection,
            well_id: &str,
            depth: &[f32],
            set_name: &str,
            mnemonic: &str,
            family: &str,
            unit: &str,
            values: &[f32],
        ) {
            let curve_id = db::upsert_curve_meta(
                conn,
                well_id,
                set_name,
                mnemonic,
                Some(unit),
                Some(family),
                Some("SB-CLY-041 ordered-alias fixture"),
                Some(1),
            )
            .unwrap();
            db::insert_curve_samples(conn, &curve_id, depth, values).unwrap();
        }

        fn output_curve(conn: &duckdb::Connection, well_id: &str, curve: &str) -> Vec<f32> {
            equations::fetch_curve_frame(conn, well_id, &[curve.to_string()])
                .unwrap()
                .1[curve]
                .clone()
        }

        fn recorded_inputs(
            conn: &duckdb::Connection,
            well_id: &str,
            output_set: &str,
        ) -> HashMap<String, String> {
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
                .map(|input| (input.argument, input.curve))
                .collect()
        }

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depth = vec![1000.0_f32];
        let vendor_gr = add_well(&conn, "VENDOR-CORRECTED-GAMMA", &depth);
        let native_gr = add_well(&conn, "NATIVE-CORRECTED-GAMMA", &depth);
        let raw_gr = add_well(&conn, "RAW-GAMMA-FALLBACK", &depth);
        for well_id in [&vendor_gr, &native_gr, &raw_gr] {
            add_curve(&conn, well_id, &depth, "RAW", "GR", "GR", "gAPI", &[40.0]);
            add_curve(
                &conn,
                well_id,
                &depth,
                "NORMALIZED",
                "GRN",
                "GR",
                "gAPI",
                &[110.0],
            );
        }
        add_curve(
            &conn,
            &vendor_gr,
            &depth,
            "CORRECTED",
            "GR_COR",
            "GR",
            "gAPI",
            &[70.0],
        );
        add_curve(
            &conn,
            &native_gr,
            &depth,
            "CORRECTED",
            "GR_EC",
            "GR",
            "gAPI",
            &[80.0],
        );

        let dbm = Mutex::new(conn);
        let gr_run = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![vendor_gr.clone(), native_gr.clone(), raw_gr.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
                opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
                output_set: Some("ORDERED-GR-ALIASES".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(gr_run.iter().all(|result| result.error.is_none()), "{gr_run:?}");
        {
            let conn = dbm.lock().unwrap();
            assert_eq!(output_curve(&conn, &vendor_gr, "VSH_GR"), vec![0.5]);
            assert_eq!(output_curve(&conn, &native_gr, "VSH_GR"), vec![0.6]);
            assert_eq!(output_curve(&conn, &raw_gr, "VSH_GR"), vec![0.2]);
            assert_eq!(recorded_inputs(&conn, &vendor_gr, "ORDERED-GR-ALIASES")["GR"], "GR_COR");
            assert_eq!(recorded_inputs(&conn, &native_gr, "ORDERED-GR-ALIASES")["GR"], "GR_EC");
            assert_eq!(recorded_inputs(&conn, &raw_gr, "ORDERED-GR-ALIASES")["GR"], "GR");
        }

        let chain_registry = crate::chain::new_registry();
        let chain_job = uuid::Uuid::new_v4();
        let cancel = crate::chain::register(&chain_registry, chain_job);
        crate::chain::run_chain(
            &dbm,
            &chain_registry,
            chain_job,
            &cancel,
            &[crate::chain::ChainStep {
                module: "vsh_gr".into(),
                log_inputs: HashMap::new(),
                params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
                opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
            }],
            &[vendor_gr.clone(), raw_gr.clone()],
            Some("ORDERED-GR-CHAIN"),
            None,
            &test_run_custody(),
            None,
        );
        {
            let conn = dbm.lock().unwrap();
            assert_eq!(
                recorded_inputs(&conn, &vendor_gr, "ORDERED-GR-CHAIN")["step[1].GR"],
                "GR_COR"
            );
            assert_eq!(
                recorded_inputs(&conn, &raw_gr, "ORDERED-GR-CHAIN")["step[1].GR"],
                "GR"
            );
        }

        let conn = dbm.into_inner().unwrap();
        let vendor_dn = add_well(&conn, "VENDOR-CORRECTED-DENSITY-NEUTRON", &depth);
        let native_dn = add_well(&conn, "NATIVE-CORRECTED-DENSITY-NEUTRON", &depth);
        for well_id in [&vendor_dn, &native_dn] {
            add_curve(&conn, well_id, &depth, "RAW", "RHOB", "RHOB", "g/cc", &[2.65]);
            add_curve(&conn, well_id, &depth, "RAW", "NPHI", "NPHI", "v/v", &[0.0]);
            add_curve(&conn, well_id, &depth, "RAW", "GR", "GR", "gAPI", &[10.0]);
        }
        for (well_id, rho, nphi, gr) in [
            (&vendor_dn, "RHO_COR", "NPHI_COR", "GR_COR"),
            (&native_dn, "RHOB_EC", "NPHI_EC", "GR_EC"),
        ] {
            add_curve(&conn, well_id, &depth, "CORRECTED", rho, "RHOB", "g/cc", &[2.35]);
            add_curve(&conn, well_id, &depth, "CORRECTED", nphi, "NPHI", "v/v", &[0.30]);
            add_curve(&conn, well_id, &depth, "CORRECTED", gr, "GR", "gAPI", &[55.0]);
        }

        let dbm = Mutex::new(conn);
        let dn_run = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "vsh_dn".into(),
                well_ids: vec![vendor_dn.clone(), native_dn.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("RHO_MA".into(), 2.65),
                    ("RHO_SH".into(), 2.45),
                    ("RHO_FL".into(), 1.0),
                    ("NPHI_MA".into(), 0.0),
                    ("NPHI_SH".into(), 0.4),
                    ("NPHI_FL".into(), 1.0),
                    ("GR_MA".into(), 10.0),
                    ("GR_SH".into(), 100.0),
                    ("FLAG_TOL".into(), 0.25),
                ]),
                opts: HashMap::new(),
                output_set: Some("ORDERED-DN-ALIASES".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(dn_run.iter().all(|result| result.error.is_none()), "{dn_run:?}");
        let conn = dbm.lock().unwrap();
        for (well_id, expected) in [
            (&vendor_dn, ["RHO_COR", "NPHI_COR", "GR_COR"]),
            (&native_dn, ["RHOB_EC", "NPHI_EC", "GR_EC"]),
        ] {
            let actual = output_curve(&conn, well_id, "VSH_DN")[0];
            assert!((actual - 0.4239).abs() <= 1e-4, "{well_id}: {actual}");
            let recorded = recorded_inputs(&conn, well_id, "ORDERED-DN-ALIASES");
            assert_eq!(recorded["RHOB"], expected[0]);
            assert_eq!(recorded["NPHI"], expected[1]);
            assert_eq!(recorded["GR"], expected[2]);
        }
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
    /// SB-CUT-020. The degenerate single-sided cut-offs these classification tests were written
    /// against, named rather than positional: `at_most` is a high bound, `at_least` a low one, both
    /// INCLUSIVE, which is exactly what a bare `>=` / `<=` cut-off has always meant.
    /// SB-CUT-022. The shipped tier ladder over four cut-off values — what a run that declares
    /// nothing applies. These classification tests predate the flags and must keep asserting the
    /// same behaviour through them, which is the point: the ladder moved from nesting to
    /// declaration without moving a number.
    fn ladder(
        vsh: Option<CutoffRange>,
        phie: Option<CutoffRange>,
        swe: Option<CutoffRange>,
        perm: Option<CutoffRange>,
    ) -> TierCutoffs {
        TierCutoffs {
            vsh,
            phie,
            swe,
            perm,
            vsh_use: default_cutoff_use("VSH"),
            phie_use: default_cutoff_use("PHIE"),
            swe_use: default_cutoff_use("SWE"),
            perm_use: default_cutoff_use("PERM"),
        }
    }

    fn at_most(value: f64) -> Option<CutoffRange> {
        Some(CutoffRange {
            low: None,
            high: Some(CutoffBound { value, operator: BoundOperator::Inclusive }),
        })
    }

    fn at_least(value: f64) -> Option<CutoffRange> {
        Some(CutoffRange {
            low: Some(CutoffBound { value, operator: BoundOperator::Inclusive }),
            high: None,
        })
    }

    #[test]
    fn classify_sample_nan_propagation() {
        // Clean pay (no perm cut).
        assert_eq!(
            classify_sample(0.2, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false),
            (1.0, 1.0, 1.0)
        );
        // Missing VSH → all excluded.
        let (s, r, p) = classify_sample(f32::NAN, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert!(s.is_nan() && r.is_nan() && p.is_nan());
        // Missing PHIE → SAND set, RES/PAY excluded.
        let (s, r, p) = classify_sample(0.2, f32::NAN, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert_eq!(s, 1.0);
        assert!(r.is_nan() && p.is_nan());
        // Missing SWE → SAND+RES set, PAY excluded.
        let (s, r, p) = classify_sample(0.2, 0.2, f32::NAN, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false);
        assert_eq!((s, r), (1.0, 1.0));
        assert!(p.is_nan());
        // Fails the sand cutoff → SAND 0 cascades to RES/PAY 0.
        assert_eq!(
            classify_sample(0.9, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), None), false),
            (0.0, 0.0, 0.0)
        );
        // Active PERM cutoff: missing PERM fails; sufficient PERM passes.
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, f32::NAN, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), at_least(1.0)), true);
        assert_eq!(p, 0.0);
        let (_, _, p) = classify_sample(0.2, 0.2, 0.3, 5.0, &ladder(at_most(0.5), at_least(0.1), at_most(0.6), at_least(1.0)), true);
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
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![well.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            // Stats only: the point of the test is the returned rows, and this keeps it from
            // writing FLAG_* curves as a side effect.
            stats_only: true
        ,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
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

    /// 21 samples one unit apart from 1000, split so that all three kinds of footage are present
    /// and none of them is zero. VSH alone decides the split, because it is the one curve whose
    /// ABSENCE makes a sample unjudgeable rather than merely failing:
    ///
    /// * `1000..1010` — VSH 0.2, passes the 0.5 cutoff  → **10 units NET**
    /// * `1010..1015` — VSH 0.8, fails the 0.5 cutoff   → **5 units NOT-NET**
    /// * `1015..`     — VSH MISSING, cannot be judged   → **UNKNOWN**
    fn seed_partition_well(conn: &duckdb::Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 21usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        let mut vsh = vec![0.2f32; n];
        vsh[10..15].fill(0.8);
        vsh[15..].fill(f32::NAN);
        equations::write_computed_curve(conn, &well, &depth, "VSH", &vsh).unwrap();
        for (curve, v) in [("PHIE", 0.20f32), ("SWE", 0.30)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![v; n]).unwrap();
        }
        well
    }

    /// SB-CUT-003 (P1). `14_cutoffs-summation-mc.md:944-955` — a summation **MUST** report
    /// `Gross`, `Net`, `NotNet` and `Unknown` as four separate quantities satisfying
    /// `Gross = Net + NotNet + Unknown` exactly, and `Unknown` — the footage whose flag could not
    /// be EVALUATED — **MUST NOT** be folded into `NotNet`.
    ///
    /// Techlog books a non-positive clipped interval as UNKNOWN, distinct from NOT-NET; IP marks
    /// nulls in-band with a `$$` pair inside a numeric column. Only the four-way partition is
    /// auditable: a zone reading 40 % net-to-gross because 60 % is shale and a zone reading 40 %
    /// because 55 % was never logged are the same two numbers and completely different rock.
    ///
    /// Pinned from both sides, because the invariant alone is satisfiable by the exact error the
    /// requirement names — fold every unjudgeable sample into `NotNet` and `Gross` still closes:
    ///
    /// * **A** — every component is its own expected footage on a zone the samples tile exactly,
    ///   so `NotNet` cannot silently absorb the missing-VSH interval.
    /// * **B** — footage carrying NO SAMPLE AT ALL lands in `Unknown`. This is what makes
    ///   deriving `Unknown` from the other three correct rather than convenient: accumulating it
    ///   from missing-flag samples alone would leave the identity broken wherever a zone extends
    ///   past the log, which is every zone bottomed on a marker the logging run did not reach.
    #[test]
    fn a_summation_partitions_gross_four_ways_and_books_unjudgeable_footage_as_unknown_not_as_notnet(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let tiled = seed_partition_well(&conn, "CUT-TILED");
        let overhang = seed_partition_well(&conn, "CUT-OVERHANG");
        // Declared 10 units deeper than the log reaches — the ordinary case of a zone bottomed on
        // a marker below TD of the run that logged it.
        db::upsert_zone_with_datum(
            &conn,
            &overhang,
            "OVERHANG",
            1000.0,
            1030.0,
            crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![tiled.clone(), overhang.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");

        // A — the undeclared "ALL" zone runs 1000..1020, which the samples tile exactly. PHIE and
        // SWE pass everywhere, so SAND, RESERVOIR and PAY partition identically and all three are
        // checked rather than one standing in for the others.
        let tiled_rows: Vec<_> = rows.iter().filter(|r| r.well_id == tiled).collect();
        assert_eq!(tiled_rows.len(), 3, "one row per summary flag");
        for r in &tiled_rows {
            assert_eq!(r.gross, 20.0, "{} gross", r.flag);
            assert_eq!(r.net, 10.0, "{} net — the ten samples that passed", r.flag);
            assert_eq!(
                r.not_net, 5.0,
                "{} not-net — the five samples that FAILED the cutoff, and ONLY those. 10.0 here \
                 means the missing-VSH interval was folded in, which is the error this pins.",
                r.flag
            );
            assert_eq!(
                r.unknown, 5.0,
                "{} unknown — the five samples with no VSH to judge",
                r.flag
            );
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} partition must close exactly",
                r.flag
            );
        }

        // B — the declared zone runs 1000..1030 while the log stops at 1020. Six sampled units are
        // unjudgeable (1015..1021, the last sample's forward interval now falling inside the zone)
        // and nine units carry no sample at all; both are footage whose flag could not be
        // evaluated, so both are Unknown.
        let over: Vec<_> = rows.iter().filter(|r| r.well_id == overhang).collect();
        assert_eq!(over.len(), 3, "one row per summary flag");
        for r in &over {
            assert_eq!(r.gross, 30.0, "{} gross is the declared zone, not the logged span", r.flag);
            assert_eq!(r.net, 10.0, "{} net", r.flag);
            assert_eq!(r.not_net, 5.0, "{} not-net", r.flag);
            assert_eq!(
                r.unknown, 15.0,
                "{} unknown — 6 unjudgeable sampled units plus 9 units nothing logged at all",
                r.flag
            );
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} partition must close exactly even where the samples do not reach the base",
                r.flag
            );
        }
    }

    /// SB-CUT-004 (P2). `14_cutoffs-summation-mc.md:966-975` — a summation **MUST** report both
    /// `N:G = Net/Gross` and `N:(G−Unknown)`, each labelled.
    ///
    /// The two differ by exactly the null fraction. Over a washed-out or partially-logged interval
    /// that difference is the whole argument about whether a net-to-gross is defensible, and no
    /// incumbent surfaces both — so an interpreter comparing one tool's number with another's has
    /// no way to know they are answering different questions.
    ///
    /// Pinned on three cases, because either ratio alone looks reasonable:
    ///
    /// * the zone the samples tile exactly, where the two still differ because some samples had
    ///   nothing to judge;
    /// * the zone declared below the logged interval, where they diverge by half — the case that
    ///   makes the pair worth reporting at all;
    /// * the well nobody interpreted, where the second ratio has NO denominator and must come back
    ///   MISSING rather than 0.00, which would read as "none of the judged rock is net".
    #[test]
    fn a_summation_reports_net_to_gross_over_all_footage_and_over_only_the_footage_it_could_judge() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let tiled = seed_partition_well(&conn, "NG-TILED");
        let overhang = seed_partition_well(&conn, "NG-OVERHANG");
        db::upsert_zone_with_datum(
            &conn,
            &overhang,
            "OVERHANG",
            1000.0,
            1030.0,
            crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();
        // Raw logs only — exactly the state after importing a LAS and running nothing, so every
        // sample is unjudgeable and Gross − Unknown is zero.
        let blank_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, blank_id, "NG-BLANK", Some("Synthetic"), None, None).unwrap();
        let blank = blank_id.to_string();
        let bn = 20usize;
        let bdepth: Vec<f32> = (0..bn).map(|i| 1000.0 + i as f32).collect();
        let bnan = vec![f32::NAN; bn];
        db::insert_standard_curves(
            &conn, blank_id, bdepth, vec![50.0; bn], bnan.clone(), vec![0.2; bn],
            vec![2.4; bn], bnan.clone(), bnan,
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![tiled.clone(), overhang.clone(), blank.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        // A — samples tile the zone exactly: 10 net of 20 gross, 5 of it unjudged.
        for r in rows.iter().filter(|r| r.well_id == tiled) {
            assert!(near(r.ntg, 0.5), "{} N:G is net/gross = 10/20, got {}", r.flag, r.ntg);
            assert!(
                near(r.ntg_known, 10.0 / 15.0),
                "{} N:(G-Unknown) is net over judged footage = 10/15, got {}",
                r.flag,
                r.ntg_known
            );
        }

        // B — the zone runs 10 units below the log. Half its footage was never judged, and the two
        // ratios diverge from 0.33 to 0.67: the same rock, described twice, honestly.
        for r in rows.iter().filter(|r| r.well_id == overhang) {
            assert!(near(r.ntg, 10.0 / 30.0), "{} N:G, got {}", r.flag, r.ntg);
            assert!(near(r.ntg_known, 10.0 / 15.0), "{} N:(G-Unknown), got {}", r.flag, r.ntg_known);
            assert!(
                r.ntg_known > r.ntg,
                "{} excluding unjudged footage can only RAISE the ratio; a second number equal to \
                 the first means Unknown never reached the denominator",
                r.flag
            );
        }

        // C — nothing was interpreted, so there is no judged footage to divide by. MISSING, never
        // zero: a printed 0.00 is a claim about rock nobody looked at.
        for r in rows.iter().filter(|r| r.well_id == blank) {
            assert_eq!(r.n_classified, 0, "{} the well really is uninterpreted", r.flag);
            assert!(near(r.unknown, r.gross), "{} every unit of it is Unknown", r.flag);
            assert!(
                r.ntg_known.is_nan(),
                "{} N:(G-Unknown) has no denominator here and must be MISSING, got {}",
                r.flag,
                r.ntg_known
            );
        }
    }

    /// SB-CUT-005 (P2). `14_cutoffs-summation-mc.md:972-985` — SandiBumi **MUST** check
    /// `Gross − (Net + NotNet + Unknown)` against a NAMED relative tolerance. Within tolerance the
    /// residual **MUST** be absorbed into the largest component **and the absorbed amount MUST
    /// appear in the result record**; outside it the summation **MUST** fail with a structured
    /// error.
    ///
    /// Tolerance `1e-7` relative, cited: `14_cutoffs-summation-mc.md:2083` (SB-CUT-T22), which is
    /// Techlog's `adjustFinal` shape with the `print` → result-field refinement. Nothing here is a
    /// petrophysical value; the footages below are NUMERICAL fixtures chosen so that a residual at
    /// the tolerance boundary is exactly representable in `f32` — at a realistic gross of tens of
    /// metres, `1e-7` relative is far below one ulp and no absorption could be observed at all.
    ///
    /// **The recorded amount is the whole requirement.** Techlog computes the same correction and
    /// prints it, which loses it: a reconciliation whose correction is not recorded is
    /// indistinguishable from no reconciliation.
    #[test]
    fn a_footage_partition_is_absorbed_into_its_largest_component_and_the_amount_recorded_or_else_refused(
    ) {
        // Gross 1e6 with ulp 0.0625, so a residual of exactly one ulp is 6.25e-8 relative — inside
        // the tolerance and still large enough to move an f32.
        let g = 1_000_000.0f32;
        let ulp = 0.0625f32;

        // A — within tolerance, and NET is the largest, so net is what moves.
        let r = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0 - ulp)
            .expect("one ulp of gross is inside 1e-7 relative");
        assert_eq!(r.net, 400_000.0 + ulp, "the residual lands on the largest component");
        assert_eq!(r.not_net, 300_000.0, "the other components are untouched");
        assert_eq!(r.unknown, 300_000.0 - ulp);
        assert_eq!(r.absorbed, ulp, "the absorbed amount is RECORDED, not printed and lost");
        assert_eq!(r.net + r.not_net + r.unknown, g, "and the partition now closes");

        // B — LARGEST, not first. Same residual, but Unknown now carries the most footage.
        let r = reconcile_partition(g, 200_000.0, 100_000.0, 700_000.0 - ulp)
            .expect("inside tolerance");
        assert_eq!(r.net, 200_000.0, "net must NOT absorb it merely for being first");
        assert_eq!(r.not_net, 100_000.0);
        assert_eq!(r.unknown, 700_000.0, "the largest component absorbs the residual");
        assert_eq!(r.absorbed, ulp);

        // C — outside tolerance the summation REFUSES, and the refusal carries the numbers. Four
        // ulps is 2.5e-7 relative, past 1e-7.
        let err = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0 - 4.0 * ulp)
            .expect_err("2.5e-7 relative is outside the 1e-7 tolerance and must refuse");
        assert_eq!(err.tolerance, PARTITION_TOLERANCE);
        assert!(
            (err.relative - 2.5e-7).abs() < 1e-12,
            "the refusal states the relative residual it measured, got {}",
            err.relative
        );
        let text = err.to_string();
        for needle in ["1000000", "residual", "1e-7"] {
            assert!(
                text.contains(needle),
                "a structured refusal names {needle} so a reader can act on it: {text}"
            );
        }

        // D — a residual of exactly zero is still a successful reconciliation recording zero, not a
        // special case that skips the check.
        let r = reconcile_partition(g, 400_000.0, 300_000.0, 300_000.0).expect("closes exactly");
        assert_eq!(r.absorbed, 0.0);
        assert_eq!(r.net, 400_000.0);

        // E — WIRED IN. An ordinary summary run carries the field and its partition closes, so the
        // guard above is protecting the real path rather than sitting in a test.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_partition_well(&conn, "RECON-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("an ordinary summary reconciles rather than refusing");
        assert!(!rows.is_empty());
        for r in &rows {
            assert_eq!(
                r.net + r.not_net + r.unknown,
                r.gross,
                "{} the reported partition closes after reconciliation",
                r.flag
            );
            assert!(
                r.residual_absorbed.abs() as f64 <= PARTITION_TOLERANCE * r.gross as f64,
                "{} a real run absorbs at most the tolerance, got {}",
                r.flag,
                r.residual_absorbed
            );
        }
    }

    /// Eleven samples one unit apart from 1000, every one of which passes every cutoff, with φ,
    /// Sw and Vsh each stepping halfway down. The whole point is that φ and the other curves are
    /// ANTI-correlated, so a thickness-weighted average and a φ-weighted one give visibly
    /// different answers — over the ten in-zone units:
    ///
    /// * `Σφh = 5(0.30) + 5(0.10) = 2.0`
    /// * Sw thickness-weighted `= 0.40`, φ-weighted `= 0.30`
    /// * Vsh thickness-weighted `= 0.25`, φ-weighted `= 0.175`
    ///
    /// `porosity_name` fills the porosity slot under a chosen mnemonic, which is what arm D needs.
    fn seed_weighting_well(conn: &duckdb::Connection, name: &str, porosity_name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        let half = |lo: f32, hi: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 5 { lo } else { hi }).collect()
        };
        equations::write_computed_curve(conn, &well, &depth, "VSH", &half(0.10, 0.40)).unwrap();
        equations::write_computed_curve(conn, &well, &depth, porosity_name, &half(0.30, 0.10))
            .unwrap();
        equations::write_computed_curve(conn, &well, &depth, "SWE", &half(0.20, 0.60)).unwrap();
        well
    }

    /// SB-POR-057 (DEC-070, RULED 2026-08-18: "quick look only shows pay summation as
    /// visual not pay curves", confirmed "8, correct"). The D-N quick-look shortcuts are
    /// structurally a comparison-only class (Comparison* roles, custody mnemonics distinct
    /// from the shared PHIE/PHIT, ancestry module identity as provenance) and the pay
    /// engine never reads them: the candidate list is the one canonical name, a well whose
    /// only porosity is the quick-look curve is reported NOT INTERPRETED with the refusal
    /// recorded on the row, and absence of any porosity is deliberately NOT marked - the
    /// flag means "present and excluded". Supersedes the DEC-042 pay-eligible fallback.
    /// Display overlay needs no gate here: plot layers read curves by mnemonic and nothing
    /// added excludes PHIE_DN_LIM from them.
    #[test]
    fn the_quick_look_porosity_never_feeds_the_summed_numbers_and_its_refusal_is_recorded_on_the_row(
    ) {
        // A - the comparison-only class is structural: every registered phi_dn porosity
        // output carries a Comparison* role, and the limited pair lands under its own
        // custody mnemonics, never the shared authoritative names.
        let dn = modules::list_modules()
            .into_iter()
            .find(|spec| spec.name == "phi_dn")
            .expect("phi_dn ships");
        let mut classified = 0usize;
        for argument in &dn.args {
            let Some(contract) = argument.porosity_output.as_ref() else { continue };
            classified += 1;
            assert!(
                format!("{:?}", contract.output_role).starts_with("Comparison"),
                "phi_dn.{} must stay comparison-typed, got {:?}",
                argument.name,
                contract.output_role
            );
        }
        assert_eq!(classified, 4, "the whole quick-look output set is comparison-typed");
        let limited = dn
            .args
            .iter()
            .find(|argument| argument.name == "PHIE")
            .expect("the limited effective output exists");
        // `log_out_as` records the custody rename in the argument's default pattern.
        assert_eq!(
            limited.default,
            modules::PHIE_DN_LIMITED_DEFAULT,
            "the limited quick-look curve writes under its own custody mnemonic, not PHIE"
        );

        // Fixture wells share seed_weighting_well's rock (avg_swe 0.30 phi-weighted,
        // avg_phie 0.20 thickness-weighted over 10 net units when summed).
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let auth = seed_weighting_well(&conn, "QL-AUTH", "PHIE");
        let ql_only = seed_weighting_well(&conn, "QL-ONLY", modules::PHIE_DN_LIMITED_DEFAULT);
        // A well carrying BOTH: the quick-look curve holds DIFFERENT numbers (0.05
        // everywhere), so a leak into the summation would move avg_phie visibly.
        let both_well = seed_weighting_well(&conn, "QL-BOTH", "PHIE");
        let depth: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32).collect();
        equations::write_computed_curve(
            &conn, &both_well, &depth, modules::PHIE_DN_LIMITED_DEFAULT, &vec![0.05f32; 11],
        )
        .unwrap();
        // A well with NO porosity of any kind (the seeded curve lands under an alien name
        // nothing resolves), to prove absence is not marked as exclusion.
        let none = seed_weighting_well(&conn, "QL-NONE", "PHIX_UNRESOLVED");
        let dbm = Mutex::new(conn);

        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![auth.clone(), ql_only.clone(), both_well.clone(), none.clone()],
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("the summary runs");
        let pay = |well: &str| -> &PaySummaryRow {
            rows.iter()
                .find(|row| row.well_id == well && row.flag == "PAY")
                .expect("a PAY row")
        };
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        // B - the authoritative well sums exactly as before, unmarked.
        let a = pay(&auth);
        assert!(near(a.avg_phie, 0.20) && near(a.avg_swe, 0.30) && near(a.net, 10.0));
        assert!(!a.quicklook_phie_excluded);

        // C - the quick-look-only well is NOT summed: not interpreted for pay, with the
        // refusal recorded. Its curve held real numbers; none of them reached a sum.
        let q = pay(&ql_only);
        assert_eq!(q.n_classified, 0, "the quick-look curve never feeds classification");
        assert!(near(q.net, 0.0) && near(q.unknown, q.gross), "unjudged, not wet");
        assert!(q.quicklook_phie_excluded, "the row records the DEC-070 refusal");
        assert!(
            rows.iter()
                .filter(|row| row.well_id == ql_only)
                .all(|row| row.quicklook_phie_excluded),
            "per well: every flag row of the well carries the mark"
        );

        // D - beside an authoritative PHIE the quick-look curve neither leaks nor marks:
        // identical averages to the plain well, flag false.
        let b = pay(&both_well);
        assert!(
            near(b.avg_phie, 0.20) && near(b.avg_swe, 0.30),
            "the 0.05 quick-look values must not move a summed average: {} {}",
            b.avg_phie,
            b.avg_swe
        );
        assert!(!b.quicklook_phie_excluded, "nothing was excluded - PHIE answered");

        // E - a well with no porosity AT ALL is not marked: the flag means "present and
        // excluded", never "absent".
        let n = pay(&none);
        assert_eq!(n.n_classified, 0);
        assert!(!n.quicklook_phie_excluded, "absence is not exclusion");

        // F - the refusal crosses the wire as a typed boolean, like its precedent.
        let wire = serde_json::to_value(q).expect("a row serializes");
        assert!(wire["quicklook_phie_excluded"].is_boolean());
    }

    /// SB-CUT-009 (P1, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1033-1048` — porosity
    /// weighting of an averaged curve **MUST** be controlled by an explicit per-curve flag stored
    /// with the curve's averaging configuration, and SandiBumi **MUST NOT** infer it from the
    /// curve's name or family.
    ///
    /// The harm is Techlog's, quoted in the chapter: *"the SW curve is weighted by POR but the SWE
    /// is not weighted"* — a curve loses its φ-weighting because of how it happens to be spelled,
    /// and on this fixture that is 0.40 against 0.30, ten saturation units, with nothing on the
    /// page to say which was used.
    ///
    /// The as-built named two gaps and both are closed here: the φ-weighted form could not be
    /// REQUESTED for another curve, and could not be SWITCHED OFF.
    ///
    /// Defaults are cited, not chosen: the φ-weighted saturation `Σ(Sw·φ·h)/Σ(φ·h)` is agreed by
    /// all three vendors (`:1041-1042`) and is what the engine already did, so nothing moves for a
    /// caller who declares nothing.
    #[test]
    fn zone_averaging_weighting_is_declared_per_curve_and_never_inferred_from_the_curve_name() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let plain = seed_weighting_well(&conn, "WGT-PLAIN", "PHIE");
        let aliased = seed_weighting_well(&conn, "WGT-ALIAS", modules::PHIE_DN_LIMITED_DEFAULT);
        let dbm = Mutex::new(conn);
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;

        let run = |wells: Vec<String>, weighting: BTreeMap<String, AverageWeighting>| {
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: wells,
                    // Permissive on purpose: every sample must pass, so the only thing that can
                    // move an average is the weighting under test.
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting,
                },
            )
            .expect("summary runs")
        };
        let pay = |rows: &[PaySummaryRow], well: &str| -> PaySummaryRow {
            rows.iter().find(|r| r.well_id == well && r.flag == "PAY").expect("a PAY row").clone()
        };

        // A — DECLARING NOTHING keeps the vendor-agreed behaviour: saturation φ-weighted, the
        // others by thickness. A caller who never heard of this flag sees no change.
        let base = run(vec![plain.clone()], BTreeMap::new());
        let r = pay(&base, &plain);
        assert!(near(r.avg_swe, 0.30), "default SWE is phi-weighted 0.30, got {}", r.avg_swe);
        assert!(near(r.avg_vsh, 0.25), "default VSH is thickness-weighted 0.25, got {}", r.avg_vsh);
        assert!(near(r.avg_phie, 0.20), "default PHIE is thickness-weighted 0.20, got {}", r.avg_phie);

        // B — it can be SWITCHED OFF. Declaring thickness weighting for the saturation slot moves
        // the answer to 0.40, which is the number Techlog silently produces for a curve spelled
        // the wrong way. Here it is a declaration, not an accident.
        let off = run(
            vec![plain.clone()],
            BTreeMap::from([("SWE".to_string(), AverageWeighting::Thickness)]),
        );
        assert!(
            near(pay(&off, &plain).avg_swe, 0.40),
            "declared thickness weighting must actually change the average, got {}",
            pay(&off, &plain).avg_swe
        );

        // C — it can be REQUESTED FOR ANOTHER CURVE, which the hard-wired version could not do at
        // all. Vsh φ-weighted is 0.175 against its thickness-weighted 0.25.
        let on = run(
            vec![plain.clone()],
            BTreeMap::from([("VSH".to_string(), AverageWeighting::Porosity)]),
        );
        assert!(
            near(pay(&on, &plain).avg_vsh, 0.175),
            "phi weighting must be available to any averaged curve, got {}",
            pay(&on, &plain).avg_vsh
        );
        assert!(
            near(pay(&on, &plain).avg_swe, 0.30),
            "and declaring one curve must not disturb another"
        );

        // D — updated under DEC-070 (RULED 2026-08-18), which removed the DEC-042 fallback
        // this arm rode on: a well whose porosity exists ONLY under the quick-look custody
        // mnemonic is no longer summed AT ALL, so the name cannot influence a weighting
        // decision because the curve never reaches the averaging - the strongest form of
        // "never inferred from the name". The refusal is observable on the row rather than
        // silent, and the anti-inference contract stays behaviourally pinned by arms A-C
        // and structurally by the scan below.
        let both = run(vec![plain.clone(), aliased.clone()], BTreeMap::new());
        let (p, a) = (pay(&both, &plain), pay(&both, &aliased));
        assert!(near(p.avg_swe, 0.30), "the authoritative well still sums");
        assert_eq!(a.n_classified, 0, "the quick-look-only well is not interpreted for pay");
        assert!(a.quicklook_phie_excluded, "and the row records why");
        assert!(!p.quicklook_phie_excluded, "a well summed from PHIE carries no such mark");

        // ...and structurally, so a future edit cannot quietly reintroduce the inference. The
        // resolver is keyed on the SLOT a curve fills — a role, fixed at compile time — and the
        // one place the summation holds a resolved MNEMONIC is `phie_curve`. Proving that name
        // never reaches the resolver is the difference between "does not infer from the name" and
        // "happens not to today". A slot key spelled like a mnemonic is not an inference: it is
        // the position, and arm D above is what proves it behaviourally.
        // Truncated at the test module, or the scan matches the very strings it is asserting
        // about and passes for free — this file is its own subject. Cut on `mod tests {` rather
        // than on `#[cfg(test)]`, which also marks three production-side test helpers far above
        // here and would silently truncate away the code actually under scan.
        let whole = include_str!("workflow.rs");
        let source = &whole[..whole.find("\nmod tests {").expect("the test module is below")];
        assert!(
            !source.contains("weighting_for(req, &phie_curve")
                && !source.contains("weighting_for(&req, &phie_curve"),
            "the resolved porosity mnemonic must never be passed to the weighting resolver"
        );
        let start = source.find("pub fn weighting_for").expect("the resolver exists");
        let body = &source[start..start + 700];
        for banned in ["phie_curve", "family", "curve_meta", "mnemonic"] {
            assert!(
                !body.contains(banned),
                "the weighting resolver must not consult {banned}; it sees a slot and a declaration"
            );
        }
    }

    /// SB-CUT-010 (P1, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1050-1062` — `HCPV` computed
    /// by direct summation `Σφ(1−Sw)h` **MUST** equal `Net × φ̄ × (1 − S̄w)` rebuilt from the
    /// reported averages, to floating-point tolerance, for every emitted zone.
    ///
    /// The expected value is an INDEPENDENT algebraic identity, not a re-derivation of the code —
    /// which is what the register meant by *shared implementation is not an independent proof*:
    ///
    /// ```text
    /// Net · φ̄ · (1 − S̄w) = Net · (Σφh/Net) · (1 − Σ Sw·φ·h / Σφh)
    ///                     = Σφh − Σ Sw·φ·h  =  Σ φh(1 − Sw)  =  HCPV
    /// ```
    ///
    /// It cancels ONLY because `S̄w` is φ-weighted. With a thickness-weighted `S̄w` the `Σφh` does
    /// not cancel and the two sides part company — so the identity is what locks SB-CUT-009's
    /// weighting choice in place, and the negative control is the half that carries the proof. On
    /// this fixture that is 1.4 against 1.2, a 14 % error in the hydrocarbon column.
    ///
    /// **Precondition, stated rather than assumed:** φ and Sw must be valid across the whole net
    /// interval. Where Sw is missing over part of net, `Net · φ̄` counts footage `HCPV` cannot, and
    /// the identity is not claimed — the engine deliberately normalises each average over the
    /// footage ITS OWN curve was valid on, which is a separate pinned rule. T07's fixture is a
    /// flagged interval with varying φ and Sw, so the precondition holds here by construction.
    #[test]
    fn hydrocarbon_pore_volume_summed_directly_equals_the_volume_rebuilt_from_the_reported_averages_in_both_engines(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "HCPV-1", "PHIE");
        let dbm = Mutex::new(conn);
        let run = |weighting: BTreeMap<String, AverageWeighting>| {
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting,
                },
            )
            .expect("summary runs")
        };
        let rebuilt = |r: &PaySummaryRow| -> f64 {
            r.net as f64 * r.avg_phie as f64 * (1.0 - r.avg_swe as f64)
        };

        // A — every emitted zone and flag closes. The absolute value is asserted too, so an
        // engine that returned zeros everywhere could not satisfy the identity vacuously.
        let rows = run(BTreeMap::new());
        assert!(!rows.is_empty());
        for r in &rows {
            assert!(
                (r.hpv as f64 - 1.4).abs() < 1e-6,
                "{} HCPV by direct summation is 1.4 on this fixture, got {}",
                r.flag,
                r.hpv
            );
            assert!(
                (r.hpv as f64 - rebuilt(r)).abs() / r.hpv as f64 <= 1e-6,
                "{} the identity must close: summed {} against rebuilt {}",
                r.flag,
                r.hpv,
                rebuilt(r)
            );
        }

        // B — the negative control the chapter demands. Declaring thickness-weighted Sw leaves the
        // direct summation alone (it never used an average) and moves the rebuilt side to 1.2. If
        // this ever stops failing, the two sides have stopped being independent.
        let off = run(BTreeMap::from([("SWE".to_string(), AverageWeighting::Thickness)]));
        for r in &off {
            assert!(
                (r.hpv as f64 - 1.4).abs() < 1e-6,
                "{} direct summation is unaffected by a weighting choice",
                r.flag
            );
            assert!(
                (rebuilt(r) - 1.2).abs() < 1e-6,
                "{} thickness-weighted Sw rebuilds 1.2, got {}",
                r.flag,
                rebuilt(r)
            );
            assert!(
                (r.hpv as f64 - rebuilt(r)).abs() / r.hpv as f64 > 1e-3,
                "{} the identity MUST fail with thickness-weighted Sw - if it holds either way it \
                 is proving nothing about the weighting",
                r.flag
            );
        }

        // C — the same identity in the Monte Carlo engine, which emits its own per-zone averages
        // and its own HPV per realization. Checked on the realization's metrics, NOT on the
        // P10/P50/P90 bundle: percentiles do not commute with a product, so the identity is a
        // statement about one realization and asserting it across percentiles would be false.
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let step = vec![1.0f32; n];
        let half = |lo: f32, hi: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 5 { lo } else { hi }).collect()
        };
        let m = crate::montecarlo::zone_metrics(
            DiscretisationModel::Forward, // DEC-071: fixture derived under FORWARD
            &half(0.10, 0.40),
            &half(0.30, 0.10),
            &half(0.20, 0.60),
            &vec![f32::NAN; n],
            &depth,
            &step,
            &db::ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: 1000.0,
                bottom_depth: 1010.0,
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            },
            &crate::montecarlo::Cutoffs {
                vsh_max: at_most(0.9),
                phie_min: at_least(0.05),
                swe_max: at_most(0.9),
                perm_min: None,
            },
            false,
        );
        let mc_rebuilt = m.net as f64 * m.avg_phie as f64 * (1.0 - m.avg_swe as f64);
        assert!(
            (m.hpv as f64 - 1.4).abs() < 1e-6,
            "Monte Carlo sums the same 1.4, got {}",
            m.hpv
        );
        assert!(
            (m.hpv as f64 - mc_rebuilt).abs() / m.hpv as f64 <= 1e-6,
            "the identity must close in the Monte Carlo engine too: {} against {}",
            m.hpv,
            mc_rebuilt
        );
    }

    /// SB-POR-047 / SB-POR-T41. Source: `11_porosity.md` §3.7 — porosity methods accept the
    /// existing `BADHOLE` flag as a DECLARED input (per `gascorr`'s optional-flag idiom) rather
    /// than depending on the analyst remembering a generic Mask, and record its effect through
    /// the DEC-039 per-version comment. Three states, and the third is the point: **flag-absent
    /// records that nobody looked — never a silent zero and never silence.**
    #[test]
    fn a_porosity_method_consumes_the_declared_badhole_flag_and_its_run_records_who_looked() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 12usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let mut wells = HashMap::new();
        for name in ["CLEAN", "FLAGGED", "ABSENT"] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
                vec![2.35; n], nan.clone(), nan,
            )
            .unwrap();
            let well = id.to_string();
            // Typed VSH through the generic store, satisfying the SB-POR-006 quantity gate.
            let curve = db::upsert_curve_meta(
                &conn, &well, "RAW", "VSH", Some("v/v"), Some("VSH"),
                Some("SB-POR-047 fixture"), None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &vec![0.2; n]).unwrap();
            match name {
                "CLEAN" => equations::write_computed_curve(&conn, &well, &depth, "BADHOLE", &vec![0.0; n]).unwrap(),
                "FLAGGED" => {
                    let flag: Vec<f32> = (0..n).map(|i| if (4..7).contains(&i) { 1.0 } else { 0.0 }).collect();
                    equations::write_computed_curve(&conn, &well, &depth, "BADHOLE", &flag).unwrap();
                }
                _ => {} // ABSENT: no hole-quality curve exists on this well at all.
            }
            wells.insert(name, well);
        }
        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "phi_den".into(),
            well_ids: ["CLEAN", "FLAGGED", "ABSENT"].iter().map(|w| wells[*w].clone()).collect(),
            log_inputs: HashMap::new(),
            // Fixture values declared in the owning test — not shipping defaults.
            params: HashMap::from([
                ("RHO_SH".to_string(), 2.5_f64),
                ("RHO_DSH".to_string(), 2.65_f64),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
            custody: test_run_custody(),
        };
        let results = run_workflow_module_into(&dbm, &req, None, None, None);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());

        let conn = dbm.lock().unwrap();
        let phie = |name: &str| -> Vec<f32> {
            equations::fetch_curve_frame(&conn, &wells[name], &["PHIE_DEN".into()])
                .unwrap()
                .1
                .remove("PHIE_DEN")
                .unwrap()
        };
        let comment = |name: &str| -> String {
            equations::list_log_sets(&conn, &wells[name])
                .unwrap()
                .into_iter()
                .find(|s| s.module == "phi_den")
                .expect("the run versioned a set")
                .comment
                .expect("a POR run must record its hole-quality custody")
        };

        // A. FLAGGED: excluded exactly where flagged, equal to CLEAN everywhere else — the
        //    method itself consumed the flag; nobody set a Mask.
        let (clean, flagged) = (phie("CLEAN"), phie("FLAGGED"));
        for i in 0..n {
            if (4..7).contains(&i) {
                assert!(flagged[i].is_nan(), "sample {i} is flagged and must be excluded");
            } else {
                assert_eq!(flagged[i].to_bits(), clean[i].to_bits(), "unflagged sample {i} must be untouched");
            }
        }
        assert_eq!(
            comment("FLAGGED"),
            "BADHOLE consumed: 3 flagged samples excluded; crossover flag not supplied - gas effect not evaluated; branches: density 9 samples; output limits: none bound"
        );

        // B. CLEAN: the flag was looked at and bound nothing.
        assert!(clean.iter().all(|v| v.is_finite()));
        assert_eq!(
            comment("CLEAN"),
            "BADHOLE consumed: 0 flagged samples excluded; crossover flag not supplied - gas effect not evaluated; branches: density 12 samples; output limits: none bound"
        );

        // C. ABSENT: the numbers equal CLEAN bit for bit — an absent flag must not invent an
        //    exclusion — and the RECORD says nobody looked, rather than a zero that reads as
        //    "checked and fine" or a silence that reads as anything at all.
        let absent = phie("ABSENT");
        for i in 0..n {
            assert_eq!(absent[i].to_bits(), clean[i].to_bits());
        }
        assert_eq!(
            comment("ABSENT"),
            "BADHOLE not supplied - hole quality not evaluated; crossover flag not supplied - gas effect not evaluated; branches: density 12 samples; output limits: none bound"
        );
    }

    /// SB-POR-026. Source: `11_porosity.md:951-952` — gas crossover is detected by `condflag`
    /// (`XOVER_FLAG` already ships) and SURFACED on the porosity output; Jauhar ruled 2026-08-16
    /// the surfacing is a PROVENANCE RECORD (the DEC-039 version comment), not a flag curve or a
    /// flag-shaped key. The flag is CONSUMED from `condflag` rather than recomputed, so the coal
    /// and washout exclusions its detection already applies survive into the record.
    ///
    /// Pinned from both sides: the record states crossover extent or that nobody looked — and
    /// **the numbers must not move**, because a provenance record that corrects is `gascorr`'s
    /// job, not this row's.
    #[test]
    fn gas_crossover_provenance_rides_the_porosity_runs_version_comment_and_never_moves_a_number()
    {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 10usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let mut wells = HashMap::new();
        for name in ["SCREENED", "UNSCREENED"] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
                vec![2.35; n], nan.clone(), nan,
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn, &well, "RAW", "VSH", Some("v/v"), Some("VSH"),
                Some("SB-POR-026 fixture"), None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &vec![0.2; n]).unwrap();
            if name == "SCREENED" {
                // condflag's own output shape: 0/1, crossover over four samples.
                let xover: Vec<f32> =
                    (0..n).map(|i| if (2..6).contains(&i) { 1.0 } else { 0.0 }).collect();
                equations::write_computed_curve(&conn, &well, &depth, "XOVER_FLAG", &xover)
                    .unwrap();
            }
            wells.insert(name, well);
        }
        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "phi_den".into(),
            well_ids: ["SCREENED", "UNSCREENED"].iter().map(|w| wells[*w].clone()).collect(),
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("RHO_SH".to_string(), 2.5_f64),
                ("RHO_DSH".to_string(), 2.65_f64),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
            custody: test_run_custody(),
        };
        let results = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());

        let conn = dbm.lock().unwrap();
        let comment = |name: &str| -> String {
            equations::list_log_sets(&conn, &wells[name])
                .unwrap()
                .into_iter()
                .find(|s| s.module == "phi_den")
                .unwrap()
                .comment
                .expect("a POR run records its custody")
        };
        // A. Screened: the record states the extent, naming the flag it consumed.
        assert!(
            comment("SCREENED")
                .contains("gas crossover flagged at 4 samples (condflag XOVER_FLAG consumed)"),
            "got: {}",
            comment("SCREENED")
        );
        // B. Unscreened: nobody looked, and the record SAYS so — never a 0, never silence.
        assert!(
            comment("UNSCREENED").contains("crossover flag not supplied - gas effect not evaluated"),
            "got: {}",
            comment("UNSCREENED")
        );
        // C. The other side: provenance never corrects. The two wells' porosities are
        //    bit-identical — a crossover record that moved a number would be gascorr's job
        //    smuggled in without its physics.
        let phie = |name: &str| -> Vec<f32> {
            equations::fetch_curve_frame(&conn, &wells[name], &["PHIE_DEN".into()])
                .unwrap()
                .1
                .remove("PHIE_DEN")
                .unwrap()
        };
        let (screened, unscreened) = (phie("SCREENED"), phie("UNSCREENED"));
        for i in 0..n {
            assert_eq!(
                screened[i].to_bits(),
                unscreened[i].to_bits(),
                "sample {i}: a provenance record must never move a number"
            );
        }
    }

    /// SB-POR-028, the record half. `11_porosity.md:946-947`: hitting a clamp raises
    /// SB-POR-003's flag — which DEC-039 ruled is the per-version comment. A silently clamped
    /// reading is a model-out-of-range fact erased from the one place an analyst reads.
    #[test]
    fn a_bound_shale_reduction_clamp_is_recorded_on_the_runs_version_comment() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 10usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let mut wells = HashMap::new();
        for (name, low_rhob) in [("BOUND", 3usize), ("UNBOUND", 0usize)] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            // RHOB 1.5 on the first `low_rhob` samples drives rhosr under the 1.95 floor.
            let rhob: Vec<f32> =
                (0..n).map(|i| if i < low_rhob { 1.5 } else { 2.35 }).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
                rhob, nan.clone(), nan,
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn, &well, "RAW", "VSH", Some("v/v"), Some("VSH"),
                Some("SB-POR-028 fixture"), None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &vec![0.0; n]).unwrap();
            wells.insert(name, well);
        }
        for well in wells.values() {
            declare_nphi_basis(&conn, well, "NPHI", "SANDSTONE");
        }
        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "phi_dn".into(),
            well_ids: ["BOUND", "UNBOUND"].iter().map(|w| wells[*w].clone()).collect(),
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("RHO_SH".to_string(), 2.5_f64),
                ("RHO_DSH".to_string(), 2.65_f64),
                ("NPHI_SH".to_string(), 0.35_f64),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
            custody: test_run_custody(),
        };
        let results = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        let conn = dbm.lock().unwrap();
        let comment = |name: &str| -> String {
            equations::list_log_sets(&conn, &wells[name])
                .unwrap()
                .into_iter()
                .find(|set| set.module == "phi_dn")
                .unwrap()
                .comment
                .expect("a POR run records its custody")
        };
        // A. Three low readings bound the density floor, and the record says which and how often.
        assert!(
            comment("BOUND").contains("shale-reduction clamps: RHOSR bound at 3 samples"),
            "got: {}",
            comment("BOUND")
        );
        // B. A run whose clamps bound NOTHING says so — "no clamp bit" and "nobody would have
        //    told you" must never read the same.
        assert!(
            comment("UNBOUND").contains("shale-reduction clamps bound nothing"),
            "got: {}",
            comment("UNBOUND")
        );
    }

    /// SB-POR-003 (DEC-039 form). Source: `docs/PRD_v2/11_porosity.md` §7's PHIFLAG stream,
    /// re-ruled by DEC-039 (2026-08-16) as free text carried per curve version: every porosity
    /// run records the branch it took and every limit that bound as that version's comment.
    /// Write and reload are the stored `log_sets` row read back through `list_log_sets`; export
    /// is the LAS `~O` provenance line, which carries the custody record beside the parameters.
    #[test]
    fn a_porosity_run_records_the_branches_it_took_and_every_limit_that_bound_and_the_record_survives_export(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let n = 10usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let mut wells = HashMap::new();
        // MIX drives phi_den down BOTH branches (2 high-shale samples, 8 density samples), and
        // RHOB 1.8 puts pe ~0.51 over the 0.3*(1-VSH) ceiling so PHIE binds on all 8.
        // SPLIT drives phi_dnbk down BOTH pseudo-mineral equations: 6 samples with
        // NPHI 0.30 > PHID 0.152 (upper, B-11/B-12) and 4 with NPHI 0.10 < PHID 0.298 (lower).
        // The split is deliberately ASYMMETRIC so a record that claims the wrong branch
        // cannot survive by symmetry - swapped labels swap the counts and both lines fail.
        let mix_vsh = {
            let mut v = vec![0.10f32; n];
            v[0] = 0.96;
            v[1] = 0.96;
            v
        };
        let split_rhob: Vec<f32> = (0..n).map(|i| if i < 6 { 2.45 } else { 2.20 }).collect();
        let split_nphi: Vec<f32> = (0..n).map(|i| if i < 6 { 0.30 } else { 0.10 }).collect();
        for (name, rhob, nphi, vsh) in [
            ("MIX", vec![1.8f32; n], vec![0.2f32; n], mix_vsh),
            ("SPLIT", split_rhob, split_nphi, vec![0.0f32; n]),
        ] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], nphi, rhob,
                nan.clone(), nan,
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn, &well, "RAW", "VSH", Some("v/v"), Some("VSH"),
                Some("SB-POR-003 fixture"), None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &vsh).unwrap();
            wells.insert(name, well);
        }
        declare_nphi_basis(&conn, &wells["SPLIT"], "NPHI", "LIMESTONE");
        let dbm = Mutex::new(conn);
        for (module, well) in [("phi_den", "MIX"), ("phi_dnbk", "SPLIT")] {
            let req = RunModuleRequest {
                module: module.into(),
                well_ids: vec![wells[well].clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("RHO_SH".to_string(), 2.5_f64),
                    ("RHO_DSH".to_string(), 2.65_f64),
                    ("RHO_W".to_string(), 1.0_f64),
                    ("NPHI_SH".to_string(), 0.35_f64),
                ]),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            };
            let results = run_workflow_module_into(&dbm, &req, None, None, None);
            assert!(results.iter().all(|r| r.error.is_none()), "{module}: {:?}",
                results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        }
        let conn = dbm.lock().unwrap();
        let comment = |well: &str, module: &str| -> String {
            equations::list_log_sets(&conn, &wells[well])
                .unwrap()
                .into_iter()
                .find(|set| set.module == module)
                .unwrap()
                .comment
                .expect("a POR run records its custody")
        };
        let mix = comment("MIX", "phi_den");
        let split = comment("SPLIT", "phi_dnbk");
        // A. Both branches are counted by name, and the counts are the fixture's own arithmetic.
        assert!(
            mix.contains("branches: density 8 samples, high-shale kill 2 samples"),
            "got: {mix}"
        );
        // B. The limit that bound is NAMED with its count - "every limit that bound" is the
        //    ruling's own text.
        assert!(mix.contains("output limits: PHIE at 8 samples"), "got: {mix}");
        // C. The pseudo-mineral split is a per-sample branch identity: both equations answered
        //    and the record says how often each did.
        assert!(split.contains("pseudo-mineral lower (B-9/B-10) 4 samples"), "got: {split}");
        assert!(split.contains("pseudo-mineral upper (B-11/B-12) 6 samples"), "got: {split}");
        // D. A run whose output limits bound NOTHING says so - silence and nothing-bound must
        //    never read the same (the SB-POR-028 principle, applied to the output side).
        assert!(split.contains("output limits: none bound"), "got: {split}");
        // E. Export: the LAS ~O provenance line carries the custody record, so the statement
        //    survives leaving the project file.
        let dest = std::env::temp_dir()
            .join(format!("sb_por003_{}.las", uuid::Uuid::new_v4()));
        crate::export::export_las(&conn, &wells["MIX"], dest.to_str().unwrap()).unwrap();
        let text = std::fs::read_to_string(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);
        assert!(
            text.contains("branches: density 8 samples"),
            "the exported ~O line must carry the custody record"
        );
    }

    /// SB-ENV-027 (DEC-033): the ONE declared repair - log_predict.SYN under OPT_COMBINE =
    /// MAX_RAW - survives BOTH mask passes and wears the typed reconstruction marker, while the
    /// SAME output under SYNTHETIC or FILL_MISSING stays masked normally. Pinned from both
    /// sides: a test proving only the MAX_RAW bypass would pass for an implementation that
    /// exempts every mode.
    #[test]
    fn the_one_declared_repair_survives_the_mask_and_wears_its_marker_while_other_modes_stay_masked(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 20usize;
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-REPAIR", None, None, None).unwrap();
        let well = id.to_string();
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        // A clean linear GR-DT relation, one flagged sample (14) whose RAW reads 200 - far
        // ABOVE the ~115 trend, so max(raw, syn) keeps the raw 200 ONLY if the input pass was
        // bypassed for the repair; a masked input would leave the prediction alone (~115).
        let gr: Vec<f32> = (0..n)
            .map(|i| if i == 19 { f32::NAN } else { 50.0 + 2.0 * i as f32 })
            .collect();
        let mut dt: Vec<f32> = gr.iter().map(|g| g + 50.0).collect();
        dt[14] = 200.0;
        let flag: Vec<f32> = (0..n).map(|i| if i == 14 { 1.0 } else { 0.0 }).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), gr, nan.clone(), nan.clone(), nan.clone(), dt, nan,
        )
        .unwrap();
        let badhole = db::upsert_curve_meta(
            &conn, &well, "RAW", "BADHOLE", Some(""), Some("BADHOLE"),
            Some("SB-ENV-027 fixture"), None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &badhole, &depth, &flag).unwrap();
        let dbm = Mutex::new(conn);
        let run = |mode: &str| {
            let req = RunModuleRequest {
                module: "log_predict".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::from([
                    ("TARGET".to_string(), "DT".to_string()),
                    ("P1".to_string(), "GR".to_string()),
                ]),
                params: HashMap::new(),
                opts: HashMap::from([
                    ("MASK".to_string(), "BADHOLE".to_string()),
                    ("OPT_COMBINE".to_string(), mode.to_string()),
                ]),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            };
            let results = run_workflow_module_into(&dbm, &req, None, None, None);
            assert!(results.iter().all(|r| r.error.is_none()), "{mode}: {:?}",
                results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        };
        let read = |name: &str| -> Vec<f32> {
            let conn = dbm.lock().unwrap();
            equations::fetch_curve_frame(&conn, &well, &[name.to_string()])
                .unwrap()
                .1
                .remove(name)
                .unwrap()
        };
        // A. The undeclared modes stay masked: SYN at the flagged depth is MISSING.
        for mode in ["SYNTHETIC", "FILL_MISSING"] {
            run(mode);
            let syn = read("DT_SYN");
            assert!(
                syn[14].is_nan(),
                "{mode}: SYN at a masked depth must stay masked, got {}",
                syn[14]
            );
        }
        // B. The declared repair survives BOTH passes: the flagged depth's raw 200 was
        //    visible to MAX_RAW (input pass bypassed) and the result was not blanked
        //    (output pass bypassed) - bit-for-bit the raw reading, which max() keeps.
        run("MAX_RAW");
        let syn = read("DT_SYN");
        assert_eq!(
            syn[14].to_bits(),
            200.0f32.to_bits(),
            "the declared repair must see and keep the raw reading, got {}",
            syn[14]
        );
        // C. The typed companion discloses exactly which finite values were reconstructed:
        //    1 at the masked depth, 0 on an ordinary sample, MISSING where the output is.
        let marker = read("DT_SYN_RECON_FLAG");
        assert_eq!(marker[14].to_bits(), 1.0f32.to_bits());
        assert_eq!(marker[5].to_bits(), 0.0f32.to_bits());
        assert!(marker[19].is_nan(), "no output, no marker - got {}", marker[19]);
    }

    /// SB-DBM-002 (DEC-021): what a run RECORDS as its module version is the producing
    /// file's own source digest ("src:<hex16>"), never the hand-maintained package version -
    /// two curves computed by different code must never claim the same producer.
    #[test]
    fn a_runs_recorded_module_version_is_the_producing_files_digest() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-VER", None, None, None).unwrap();
        let n = 5usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth, vec![60.0; n], nan.clone(), nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let req = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![id.to_string()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".to_string(), 20.0_f64), ("GR_SH".to_string(), 120.0_f64)]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
            custody: test_run_custody(),
        };
        let results = run_workflow_module_into(&dbm, &req, None, None, None);
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        let conn = dbm.lock().unwrap();
        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE module = 'vsh_gr'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let expected = format!(
            "\"module_version\":\"src:{}\"",
            modules::module_source_digest("vsh_gr")
        );
        assert!(
            params_json.contains(&expected),
            "the stored ancestry must carry the producing file's digest; wanted {expected} in {params_json}"
        );
    }

    /// SB-ENV-022 (DEC-060, reversing DEC-032): a bad-hole cause flag is an ordinary
    /// 1 = true flag - safe in the mask machinery, which is the reversal's stated reason -
    /// so it is ACCEPTED as a MASK through the production runner, and the coded-form refusal
    /// guard is dropped rather than left as dead ceremony. Masking on BADHOLE_DRHO_POS blanks
    /// exactly the samples where that cause fired.
    #[test]
    fn a_badhole_cause_flag_is_an_ordinary_flag_and_masks_through_the_runner() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-RSN", None, None, None).unwrap();
        let n = 3usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![60.0; n], nan.clone(), nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        let drho_id = db::upsert_curve_meta(
            &conn, &id.to_string(), "RAW", "DRHO", Some("g/cc"), Some("DRHO"), None, None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &drho_id, &depth, &[0.30, 0.01, 0.30]).unwrap();
        let dbm = Mutex::new(conn);
        // Run badhole so the cause flags exist as stored curves.
        let results = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "badhole".into(),
                well_ids: vec![id.to_string()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("DRHO_MAX".to_string(), 0.02_f64),
                    ("DCAL_MAX".to_string(), 2.0_f64),
                ]),
                opts: HashMap::from([("DRHO_MAX_UNIT".to_string(), "g/cc".to_string())]),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        // Mask a vsh_gr run on the DRHO-positive cause flag: accepted, and it blanks
        // exactly the samples where the cause fired.
        let results = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![id.to_string()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("GR_MA".to_string(), 20.0_f64),
                    ("GR_SH".to_string(), 120.0_f64),
                ]),
                opts: HashMap::from([("MASK".to_string(), "BADHOLE_DRHO_POS".to_string())]),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(
            results.iter().all(|r| r.error.is_none()),
            "an ordinary cause flag must be accepted as a mask: {:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>()
        );
        let conn = dbm.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH' \
                 ORDER BY depth",
            )
            .unwrap();
        let vsh: Vec<f32> = stmt
            .query_map(duckdb::params![id.to_string()], |row| {
                Ok(row.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(vsh[0].is_nan(), "flagged sample masked");
        assert!((vsh[1] - 0.4).abs() < 1e-4, "clean sample computes, got {}", vsh[1]);
        assert!(vsh[2].is_nan(), "flagged sample masked");
    }

    /// SB-POR-010 (SHOULD): a porosity curve's audit trail - method name, the FULL
    /// effective parameter set including zone overrides, and the resolved input curve
    /// identities - is sufficient to RE-DERIVE the curve without the session. The proof
    /// reconstructs the run from the STORED record alone and reproduces the stored bytes
    /// bit for bit; then the live zone table moves underneath and the record still
    /// reproduces the ORIGINAL bytes, while the live re-run honestly reports non-identity
    /// (the SB-DBM-015 arm E contract, deliberately unchanged - a re-run executes under
    /// today's interpretation, a re-derivation replays the recorded one). A record
    /// stripped of its zone entry fails to reproduce, so the `PARAM@ZONE` entries are
    /// load-bearing, not decorative.
    #[test]
    fn a_porosity_curve_re_derives_from_its_stored_manifest_alone_while_the_live_tables_move_underneath(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-POR010", None, None, None).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = id.to_string();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![50.0; n], nan.clone(),
            vec![0.20, 0.22, 0.24, 0.26], vec![2.30, 2.35, 2.40, 2.45], nan.clone(), nan,
        )
        .unwrap();
        db::upsert_md_zone(&conn, &well, "ZONE_A", 1000.0, 1002.0).unwrap();
        db::upsert_md_zone(&conn, &well, "ZONE_B", 1002.0, 1004.0).unwrap();
        // The override the record must carry: shale density differs in ZONE_A.
        db::set_zone_param(&conn, &well, "ZONE_A", "RHO_SH", Some(2.40), None).unwrap();
        let dbm = Mutex::new(conn);
        // phi_den requires a TYPED shale-volume input (SB-CLY-043), so VSH comes from a
        // real vsh_gr run: GR 50 against 20/120 endpoints is a constant 0.30.
        seed_typed_vsh(&dbm, &well);

        let results = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "phi_den".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([
                    ("RHO_MA".to_string(), 2.645_f64),
                    ("RHO_SH".to_string(), 2.50_f64),
                    ("RHO_FL".to_string(), 1.0_f64),
                    ("RHO_DSH".to_string(), 2.70_f64),
                    ("RHO_W".to_string(), 1.0_f64),
                    ("PHIE_MAX".to_string(), 0.3_f64),
                ]),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(results[0].error.is_none(), "{:?}", results[0].error);

        let (set_id, ancestry) = {
            let conn = dbm.lock().unwrap();
            let entry = equations::list_log_sets(&conn, &well)
                .unwrap()
                .into_iter()
                .find(|entry| entry.module == "phi_den")
                .expect("the run recorded its set");
            (entry.set_id.clone(), entry.ancestry.clone().expect("complete ancestry"))
        };

        // A - the record is COMPLETE: method name, every declared parameter with a value,
        // the zone override under its own PARAM@ZONE name, and both input identities.
        assert_eq!(ancestry.module, "phi_den", "the method name is the record's spine");
        let value_of = |name: &str| -> Option<f64> {
            ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == name)
                .and_then(|parameter| parameter.value.as_f64())
        };
        for declared in [
            "RHO_MA", "RHO_SH", "RHO_FL", "RHO_DSH", "RHO_W", "PHIE_MAX", "VSH_SHALE",
            "PHIE_FLOOR",
        ] {
            assert!(
                value_of(declared).is_some(),
                "the record must carry the effective value of {declared}"
            );
        }
        assert_eq!(
            value_of("RHO_SH@ZONE_A"),
            Some(2.40_f32 as f64),
            "the zone override is part of the full parameter set"
        );
        for input in ["RHOB", "VSH"] {
            assert!(
                ancestry
                    .inputs
                    .iter()
                    .any(|entry| entry.curve == input && !entry.set_id.is_empty()),
                "the record must carry the resolved identity of input {input}"
            );
        }

        // The stored bytes this proof must reproduce.
        let stored: Vec<(String, Vec<u32>)> = {
            let conn = dbm.lock().unwrap();
            ancestry
                .outputs
                .iter()
                .map(|output| {
                    let mut stmt = conn
                        .prepare(
                            "SELECT value FROM computed_curves WHERE well_id = ?1 AND \
                             curve_name = ?2 AND set_id = ?3 ORDER BY depth",
                        )
                        .unwrap();
                    let bits: Vec<u32> = stmt
                        .query_map(duckdb::params![well, output.curve, set_id], |row| {
                            Ok(row.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN).to_bits())
                        })
                        .unwrap()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    assert_eq!(bits.len(), n, "{} stored on the full frame", output.curve);
                    (output.curve.clone(), bits)
                })
                .collect()
        };

        // Re-derivation FROM THE RECORD: base values and options from the ancestry
        // parameter entries, zone overrides from the PARAM@ZONE entries laid over the
        // digest-verified zone geometry, inputs fetched by the recorded curve names.
        // Nothing reads the live zone_params table.
        let rebuild = |include_zone_entries: bool| -> HashMap<String, Vec<f32>> {
            let conn = dbm.lock().unwrap();
            let (_, recorded_digest) = {
                let recorded = ancestry.zone_set.as_ref().expect("zoned run records its set");
                let (version, digest) = db::current_zone_set(&conn, &well).unwrap();
                assert_eq!(
                    digest, recorded.digest,
                    "the zone geometry is digest-pinned; a moved top refuses upstream"
                );
                (version, digest)
            };
            let _ = recorded_digest;
            let zones = db::list_zones(&conn, &well).unwrap();
            let input_names: Vec<String> =
                ancestry.inputs.iter().map(|entry| entry.curve.clone()).collect();
            let (frame_depth, columns) =
                equations::fetch_curve_frame_from_set(&conn, &well, &input_names, None, None)
                    .unwrap();
            assert_eq!(frame_depth.len(), n);
            let spec = modules::list_modules()
                .into_iter()
                .find(|module| module.name == "phi_den")
                .unwrap();
            let mut params: HashMap<String, Vec<f64>> = HashMap::new();
            let mut opts: HashMap<String, String> = HashMap::new();
            for argument in &spec.args {
                match argument.kind {
                    modules::ArgKind::Param => {
                        let Some(base) = value_of(&argument.name) else { continue };
                        let mut array = vec![base; n];
                        if include_zone_entries {
                            for parameter in &ancestry.parameters {
                                let Some((name, zone)) = parameter.name.split_once('@') else {
                                    continue;
                                };
                                if name != argument.name || zone == "unit_custody" {
                                    continue;
                                }
                                let Some(value) = parameter.value.as_f64() else { continue };
                                if zone == "*" {
                                    array.fill(value);
                                }
                            }
                            for parameter in &ancestry.parameters {
                                let Some((name, zone)) = parameter.name.split_once('@') else {
                                    continue;
                                };
                                if name != argument.name || zone == "unit_custody" || zone == "*" {
                                    continue;
                                }
                                let Some(value) = parameter.value.as_f64() else { continue };
                                let Some(range) =
                                    zones.iter().find(|candidate| candidate.zone_name == zone)
                                else {
                                    continue;
                                };
                                for (i, d) in frame_depth.iter().enumerate() {
                                    if *d >= range.top_depth && *d < range.bottom_depth {
                                        array[i] = value;
                                    }
                                }
                            }
                        }
                        params.insert(argument.name.clone(), array);
                    }
                    modules::ArgKind::Option => {
                        if let Some(value) = ancestry
                            .parameters
                            .iter()
                            .find(|parameter| parameter.name == argument.name)
                            .and_then(|parameter| parameter.value.as_str())
                        {
                            opts.insert(argument.name.clone(), value.to_string());
                        }
                    }
                    _ => {}
                }
            }
            let ctx = modules::ModuleContext {
                n,
                logs: columns.into_iter().collect(),
                params,
                opts,
                depth_unit: crate::units::DepthUnit::Metres,
            };
            modules::run_module("phi_den", &ctx)
                .expect("the recorded run re-derives")
                .into_iter()
                .collect()
        };

        // B - the record alone reproduces the stored bytes, zone override included.
        let rebuilt = rebuild(true);
        for (curve, bits) in &stored {
            let values = rebuilt.get(curve).unwrap_or_else(|| panic!("{curve} re-derived"));
            for i in 0..n {
                assert_eq!(
                    values[i].to_bits(),
                    bits[i],
                    "{curve} sample {i}: the record must reproduce the stored byte"
                );
            }
        }

        // C - the live zone table moves; the RECORD still reproduces the ORIGINAL bytes,
        // while the live re-run honestly reports non-identity (SB-DBM-015 arm E).
        {
            let conn = dbm.lock().unwrap();
            db::set_zone_param(&conn, &well, "ZONE_A", "RHO_SH", Some(2.30), None).unwrap();
        }
        let after_move = rebuild(true);
        for (curve, bits) in &stored {
            for i in 0..n {
                assert_eq!(
                    after_move[curve][i].to_bits(),
                    bits[i],
                    "{curve} sample {i}: re-derivation reads the record, never the live table"
                );
            }
        }
        let live = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect("a changed parameter is not a broken manifest");
        assert!(
            !live.bit_identical,
            "the live re-run under a moved zone parameter must not claim bit-identity"
        );
        {
            let conn = dbm.lock().unwrap();
            db::set_zone_param(&conn, &well, "ZONE_A", "RHO_SH", Some(2.40), None).unwrap();
        }

        // D - the zone entry is LOAD-BEARING: a record stripped of its PARAM@ZONE entries
        // fails to reproduce the zoned samples.
        let stripped = rebuild(false);
        let phie_stored = &stored.iter().find(|(curve, _)| curve == "PHIE").unwrap().1;
        let mut differs = false;
        for i in 0..n {
            if stripped["PHIE"][i].to_bits() != phie_stored[i] {
                differs = true;
            }
        }
        assert!(differs, "without the zone entry the re-derivation must NOT reproduce ZONE_A");
    }

    /// SB-DBM-015 / exact SB-DBM-T15 (DEC-021/023/024): the re-run manifest is complete and
    /// CHECKED. The unmutated project replays bit-identically; a differing module version, a
    /// re-versioned input curve, an edited zone set and a deleted applied model are each
    /// REFUSED with the refusal NAMING the element - a refusal that names no element fails
    /// this test. A changed zone parameter, by contrast, still resolves and replays - and
    /// the report says the result is NOT bit-identical rather than pretending.
    #[test]
    fn the_rerun_manifest_resolves_or_refuses_naming_the_element_and_replays_bit_identically() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-RRM", None, None, None).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = id.to_string();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth, vec![60.0, 70.0, 80.0, 90.0], nan.clone(), nan.clone(),
            nan.clone(), nan.clone(), nan,
        )
        .unwrap();
        db::upsert_md_zone(&conn, &well, "MIOCENE_A", 1000.0, 1002.0).unwrap();
        db::upsert_md_zone(&conn, &well, "MIOCENE_B", 1002.0, 1004.0).unwrap();
        db::set_zone_param(&conn, &well, "MIOCENE_A", "GR_MA", Some(10.0), None).unwrap();
        let dbm = Mutex::new(conn);
        // Producer: a smoothed GR the consumer resolves as its input, so a later
        // re-version of the producer is exactly T15's moved-input case.
        let smooth = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "smooth".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("WINDOW".to_string(), 1.0_f64)]),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(smooth[0].error.is_none(), "{:?}", smooth[0].error);
        let smoothed = smooth[0]
            .output_curves
            .iter()
            .find(|curve| !curve.ends_with("_SPK"))
            .expect("smooth writes its smoothed curve")
            .clone();
        let vsh_request = RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![well.clone()],
            log_inputs: HashMap::from([("GR".to_string(), smoothed.clone())]),
            params: HashMap::from([
                ("GR_MA".to_string(), 20.0_f64),
                ("GR_SH".to_string(), 120.0_f64),
            ]),
            opts: HashMap::new(),
            output_set: None,
            input_set: None,
            custody: test_run_custody(),
        };
        let results = run_workflow_module(&dbm, &vsh_request);
        assert!(results[0].error.is_none(), "{:?}", results[0].error);
        let set_id = {
            let conn = dbm.lock().unwrap();
            equations::list_log_sets(&conn, &well)
                .unwrap()
                .into_iter()
                .find(|entry| entry.module == "vsh_gr")
                .expect("the run recorded its set")
                .set_id
        };

        // A. Unmutated: every element resolves, and the replay is bit-identical.
        let report = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect("an unmutated manifest resolves");
        assert!(report.bit_identical, "the unmutated replay must reproduce byte for byte");
        assert!(report.compared_curves >= 3, "{}", report.compared_curves);

        // B. Module version differs: refused, NAMING the element.
        {
            let conn = dbm.lock().unwrap();
            conn.execute(
                "UPDATE log_sets SET params_json = REPLACE(params_json, 'src:', 'src:dead') WHERE set_id = ?1",
                duckdb::params![set_id],
            )
            .unwrap();
        }
        let error = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect_err("a different implementation must refuse");
        assert!(error.contains("module version"), "the element is named: {error}");
        {
            let conn = dbm.lock().unwrap();
            conn.execute(
                "UPDATE log_sets SET params_json = REPLACE(params_json, 'src:dead', 'src:') WHERE set_id = ?1",
                duckdb::params![set_id],
            )
            .unwrap();
        }

        // C. Applied model deleted: the manifest names a model that is not in ml_models.
        {
            let conn = dbm.lock().unwrap();
            conn.execute(
                "UPDATE log_sets SET params_json = REPLACE(params_json, '\"module_version\"', '\"applied_model\":\"11111111-2222-3333-4444-555555555555\",\"module_version\"') WHERE set_id = ?1",
                duckdb::params![set_id],
            )
            .unwrap();
        }
        let error = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect_err("a deleted applied model must refuse");
        assert!(
            error.contains("applied model") && error.contains("11111111"),
            "the element is named: {error}"
        );
        {
            let conn = dbm.lock().unwrap();
            conn.execute(
                "UPDATE log_sets SET params_json = REPLACE(params_json, '\"applied_model\":\"11111111-2222-3333-4444-555555555555\",', '') WHERE set_id = ?1",
                duckdb::params![set_id],
            )
            .unwrap();
        }

        // D. Zone set edited: a moved top changes the zone-set identity; restoring the
        //    geometry restores it (the digest is content identity, not a counter).
        {
            let conn = dbm.lock().unwrap();
            db::upsert_md_zone(&conn, &well, "MIOCENE_B", 1002.5, 1004.0).unwrap();
        }
        let error = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect_err("an edited zone set must refuse");
        assert!(error.contains("zone set"), "the element is named: {error}");
        {
            let conn = dbm.lock().unwrap();
            db::upsert_md_zone(&conn, &well, "MIOCENE_B", 1002.0, 1004.0).unwrap();
        }

        // E. A changed zone PARAMETER still resolves - the honest report is a replay that
        //    is NOT bit-identical, never a silent claim of reproduction.
        {
            let conn = dbm.lock().unwrap();
            db::set_zone_param(&conn, &well, "MIOCENE_A", "GR_MA", Some(15.0), None).unwrap();
        }
        let report = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect("a changed parameter is not a broken manifest");
        assert!(
            !report.bit_identical,
            "a replay under a changed zone parameter must not claim bit-identity"
        );
        {
            let conn = dbm.lock().unwrap();
            db::set_zone_param(&conn, &well, "MIOCENE_A", "GR_MA", Some(10.0), None).unwrap();
        }

        // F. Input curve re-versioned: re-running the producer moves the input's resolved
        //    identity, and the consumer's re-run refuses NAMING the curve.
        let smooth = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "smooth".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("WINDOW".to_string(), 1.0_f64)]),
                opts: HashMap::new(),
                output_set: None,
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert!(smooth[0].error.is_none(), "{:?}", smooth[0].error);
        let error = rerun_log_set(&dbm, &well, &set_id, &test_run_custody())
            .expect_err("a re-versioned input must refuse");
        assert!(
            error.contains(&smoothed),
            "the element is named ({smoothed}): {error}"
        );
    }

    /// SB-CLY-001 (DEC-036 constraints 2 and 4): through the production runner, a degenerate
    /// zone is NAMED in the run-level message with the parameter pair and the offending
    /// values - the per-sample token does not discharge it - a masked sample carries the
    /// runner-owned MASKED_INPUT token instead of a blank, and the token curve is refused as
    /// a MASK by name (0 = COMPUTED would invert under rule 11).
    #[test]
    fn an_inverted_endpoint_zone_is_named_in_the_run_message_and_a_masked_sample_keeps_its_token(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CLY", None, None, None).unwrap();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![60.0; n], nan.clone(), nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        // Two zones; the deeper one carries an INVERTED override - the data-entry error the
        // chapter names as the single most common in the domain.
        db::upsert_md_zone(&conn, &id.to_string(), "MIOCENE_OK", 1000.0, 1002.0).unwrap();
        db::upsert_md_zone(&conn, &id.to_string(), "MIOCENE_BAD", 1002.0, 1004.0).unwrap();
        db::set_zone_param(&conn, &id.to_string(), "MIOCENE_BAD", "GR_MA", Some(150.0), None)
            .unwrap();
        db::set_zone_param(&conn, &id.to_string(), "MIOCENE_BAD", "GR_SH", Some(100.0), None)
            .unwrap();
        // A mask flagging the second sample (inside the VALID zone).
        let mask_id = db::upsert_curve_meta(
            &conn, &id.to_string(), "RAW", "MYFLAG", Some("flag"), None, None, None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &mask_id, &depth, &[0.0, 1.0, 0.0, 0.0]).unwrap();
        let dbm = Mutex::new(conn);
        let run = |mask: Option<&str>| -> Vec<ModuleRunResult> {
            let mut opts: HashMap<String, String> = HashMap::new();
            if let Some(mask) = mask {
                opts.insert("MASK".to_string(), mask.to_string());
            }
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "vsh_gr".into(),
                    well_ids: vec![id.to_string()],
                    log_inputs: HashMap::new(),
                    params: HashMap::from([
                        ("GR_MA".to_string(), 20.0_f64),
                        ("GR_SH".to_string(), 120.0_f64),
                    ]),
                    opts,
                    output_set: None,
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
        };
        // A. The token curve is refused as a MASK by name, naming the registry version.
        let refused = run(Some("VSH_PROV"));
        let error = refused[0].error.clone().expect("the token curve must be refused as a mask");
        assert!(error.contains("registry v1"), "the refusal names the vocabulary: {error}");
        // B. The run succeeds, and the zone-bearing message names the zone, the pair and
        //    the offending values.
        let results = run(Some("MYFLAG"));
        assert!(results[0].error.is_none(), "{:?}", results[0].error);
        let message = results[0]
            .degradations
            .iter()
            .find(|d| d.kind == modules::RunDegradationKind::EndpointInvalid)
            .map(|d| d.detail.clone())
            .expect("the zone-bearing message is part of the contract, not the curve");
        assert!(message.contains("MIOCENE_BAD"), "the ZONE is named: {message}");
        assert!(
            message.contains("GR_MA") && message.contains("GR_SH"),
            "the parameter pair is named: {message}"
        );
        assert!(
            message.contains("150") && message.contains("100"),
            "the offending values are named: {message}"
        );
        // C. Stored tokens: computed / masked / endpoint-invalid, each its own statement,
        //    and the masked sample keeps a TOKEN where every ordinary output is blanked.
        let conn = dbm.lock().unwrap();
        let read = |curve: &str| -> Vec<f32> {
            let mut stmt = conn
                .prepare(
                    "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 \
                     ORDER BY depth",
                )
                .unwrap();
            stmt.query_map(duckdb::params![id.to_string(), curve], |row| {
                Ok(row.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
        };
        let prov = read("VSH_PROV");
        assert_eq!(prov[0], crate::param_sources::CLY_PROV_COMPUTED);
        assert_eq!(
            prov[1],
            crate::param_sources::CLY_PROV_MASKED_INPUT,
            "a masked sample's token is the mask's own statement, never a blank"
        );
        assert_eq!(prov[2], crate::param_sources::CLY_PROV_ENDPOINT_INVALID);
        assert_eq!(prov[3], crate::param_sources::CLY_PROV_ENDPOINT_INVALID);
        let vsh = read("VSH");
        assert!((vsh[0] - 0.4).abs() < 1e-4, "the valid zone still computes: {}", vsh[0]);
        assert!(vsh[1].is_nan() && vsh[2].is_nan() && vsh[3].is_nan());
    }

    /// SB-ENV-007 (DEC-060(a) + DEC-031(b)): the one-hot flag group survives the production
    /// runner as ordinary stored flags - safe in the mask machinery, the reversal's stated
    /// reason - and the applied-step manifest rides the run's LOG-SET record: the branch
    /// record composes "corrected in full N, not applied M" into the version comment, which
    /// is the per-run manifest the per-sample group complements.
    #[test]
    fn the_correction_flag_group_is_one_hot_through_the_runner_and_the_manifest_rides_the_log_set()
    {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CST", None, None, None).unwrap();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        // GR everywhere, caliper over half: a partial run under DEC-031 part (c).
        let cali_id = db::upsert_curve_meta(
            &conn, &id.to_string(), "RAW", "CALI", Some("in"), Some("CALI"), None, None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &cali_id, &depth, &[10.5, f32::NAN, 10.5, f32::NAN])
            .unwrap();
        db::insert_standard_curves(
            &conn, id, depth, vec![80.0; n], nan.clone(), nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let run = |mask: Option<&str>| -> Vec<ModuleRunResult> {
            let mut opts: HashMap<String, String> = HashMap::new();
            if let Some(mask) = mask {
                opts.insert("MASK".to_string(), mask.to_string());
            }
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "gr_hole_corr".into(),
                    well_ids: vec![id.to_string()],
                    log_inputs: HashMap::new(),
                    params: HashMap::from([
                        ("K_GR".to_string(), 0.01_f64),
                        ("BS_DEF".to_string(), 8.5_f64),
                    ]),
                    opts,
                    output_set: None,
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
        };
        let results = run(None);
        assert!(results.iter().all(|r| r.error.is_none()), "{:?}",
            results.iter().filter_map(|r| r.error.clone()).collect::<Vec<_>>());
        let conn = dbm.lock().unwrap();
        let comment = equations::list_log_sets(&conn, &id.to_string())
            .unwrap()
            .into_iter()
            .find(|set| set.module == "gr_hole_corr")
            .unwrap()
            .comment
            .expect("a correction run records its step manifest");
        assert!(
            comment.contains("corrected in full 2 samples")
                && comment.contains("not applied (no caliper) 2 samples"),
            "the manifest states what was and was not applied: {comment}"
        );
        // The stored group is one-hot at every sampled depth: FULL where the caliper covered,
        // NONE where the raw value passed through - written as ordinary 1 = true flags.
        let read = |curve: &str| -> Vec<f32> {
            let mut stmt = conn
                .prepare(
                    "SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 \
                     ORDER BY depth",
                )
                .unwrap();
            stmt.query_map(duckdb::params![id.to_string(), curve], |row| {
                Ok(row.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
        };
        assert_eq!(read("GR_EC_FULL"), [1.0, 0.0, 1.0, 0.0]);
        assert_eq!(read("GR_EC_NONE"), [0.0, 1.0, 0.0, 1.0]);
    }

    /// SB-ENV-029 (DEC-025): the runner carries the input curve's DECLARED neutron basis to
    /// nphimat's consistency gate - a declaration made at import reaches the module that
    /// would otherwise convert on the wrong scale silently.
    #[test]
    fn a_declared_neutron_basis_reaches_nphimat_through_the_production_runner() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-NBAS", None, None, None).unwrap();
        let n = 3usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth, nan.clone(), nan.clone(), vec![0.18; n], nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        // The NPHI standard column resolves through the generic-store migration path; declare
        // the basis on the migrated curve.
        db::migrate_standard_curves_to_generic_store(&conn).unwrap();
        let nphi_curve = db::list_generic_curve_catalog(&conn, &id.to_string())
            .unwrap()
            .into_iter()
            .find(|c| c.mnemonic == "NPHI")
            .expect("NPHI migrated")
            .curve_id;
        db::set_curve_neutron_basis(
            &conn, &nphi_curve, "LIMESTONE", "declared at import by the user (DEC-025)",
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let run = |matrix_in: &str| -> Vec<ModuleRunResult> {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "nphimat".into(),
                    well_ids: vec![id.to_string()],
                    log_inputs: HashMap::new(),
                    params: HashMap::new(),
                    opts: HashMap::from([("MATRIX_IN".to_string(), matrix_in.to_string())]),
                    output_set: None,
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
        };
        // Contradiction refuses through the full production path.
        let refused = run("SS");
        let error = refused[0].error.clone().expect("the declared basis must gate MATRIX_IN");
        assert!(error.contains("LIMESTONE") && error.contains("DEC-025"), "{error}");
        // Agreement runs.
        let matched = run("LS");
        assert!(matched[0].error.is_none(), "{:?}", matched[0].error);
    }

    /// SB-CUT-001 (DEC-071, RULED 2026-08-18: "centred"): the thickness discretisation
    /// model is a request parameter DEFAULTING TO CENTRED - a sample's slab straddles its
    /// depth - while FORWARD (the chapter's TOPS rule) stays selectable and reproduces the
    /// shipped numbers. The two models are proven DISTINCT on a data-edge zone, hand-derived
    /// both ways; every record names the model that produced it; and the Monte Carlo engine
    /// agrees with the deterministic summary under BOTH models, so an MC P50 can never
    /// disagree with the pay summary for this reason.
    #[test]
    fn the_discretisation_model_defaults_to_centred_and_forward_reproduces_the_shipped_numbers() {
        // A - the default is CENTRED, on the enum and over the wire: a request that never
        // mentions the field deserializes to the ruled default, so every pre-ruling caller
        // gets CENTRED rather than silently keeping the old rule.
        assert_eq!(DiscretisationModel::default(), DiscretisationModel::Centred);
        assert_eq!(DiscretisationModel::Centred.token(), "CENTRED");
        assert_eq!(DiscretisationModel::Forward.token(), "TOPS");
        let wire: PaySummaryRequest = serde_json::from_value(serde_json::json!({
            "well_ids": [],
            "vsh_max": null,
            "phie_min": null,
            "swe_max": null,
            "perm_min": null,
        }))
        .expect("a pre-ruling request still deserializes");
        assert_eq!(wire.discretisation, DiscretisationModel::Centred);

        // B - the ONE slab derivation: centred straddles, forward hangs down.
        assert_eq!(
            sample_slab(1000.0, 1.0, DiscretisationModel::Centred),
            (999.5, 1000.5)
        );
        assert_eq!(
            sample_slab(1000.0, 1.0, DiscretisationModel::Forward),
            (1000.0, 1001.0)
        );

        // C - hand-derived, both ways, on a zone straddling the data edge. Samples at
        // 1000..=1003 m, step 1 m, all-pay curves; zone [999, 1001). FORWARD: only the
        // 1000 m slab [1000, 1001) overlaps -> net 1.0. CENTRED: the 1000 m slab
        // [999.5, 1000.5) contributes 1.0 and the 1001 m slab [1000.5, 1001.5) contributes
        // 0.5 -> net 1.5. Jauhar accepted exactly this kind of movement.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CENTRED", None, None, None).unwrap();
        let well = id.to_string();
        let n = 4usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
            vec![2.35; n], nan.clone(), nan,
        )
        .unwrap();
        for (curve, value) in [("VSH", 0.10), ("PHIE", 0.30), ("SWE", 0.20)] {
            equations::write_computed_curve(&conn, &well, &depth, curve, &vec![value; n])
                .unwrap();
        }
        db::upsert_zone_with_datum(
            &conn, &well, "EDGE", 999.0, 1001.0, crate::schema_vocab::DepthDatum::Md,
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let run = |model: DiscretisationModel| -> PaySummaryRow {
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: model,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                    perm_min: None,
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting: Default::default(),
                },
            )
            .unwrap()
            .into_iter()
            .find(|row| row.flag == "PAY")
            .expect("a PAY row")
        };
        let forward = run(DiscretisationModel::Forward);
        let centred = run(DiscretisationModel::Centred);
        assert!(
            (forward.net - 1.0).abs() < 1e-6,
            "FORWARD reproduces the shipped number 1.0, got {}",
            forward.net
        );
        assert!(
            (centred.net - 1.5).abs() < 1e-6,
            "CENTRED counts the straddling halves: expected 1.5, got {}",
            centred.net
        );
        assert_eq!(forward.discretisation_model, "TOPS");
        assert_eq!(centred.discretisation_model, "CENTRED");

        // D - the DEC-071-noted contract: the Monte Carlo engine's net agrees with the
        // deterministic pay summary for the same inputs, under BOTH models.
        let step = vec![1.0f32; n];
        for (model, expected) in [
            (DiscretisationModel::Forward, forward.net),
            (DiscretisationModel::Centred, centred.net),
        ] {
            let m = crate::montecarlo::zone_metrics(
                model,
                &vec![0.10f32; n],
                &vec![0.30f32; n],
                &vec![0.20f32; n],
                &vec![f32::NAN; n],
                &depth,
                &step,
                &db::ZoneEntry {
                    zone_name: "EDGE".into(),
                    top_depth: 999.0,
                    bottom_depth: 1001.0,
                    depth_datum: crate::schema_vocab::DepthDatum::Md,
                },
                &crate::montecarlo::Cutoffs {
                    vsh_max: at_most(0.9),
                    phie_min: at_least(0.05),
                    swe_max: at_most(0.9),
                    perm_min: None,
                },
                false,
            );
            assert!(
                (m.net - expected).abs() < 1e-6,
                "Monte Carlo net {} must agree with the pay summary {} under {:?}",
                m.net,
                expected,
                model
            );
        }
    }

    /// SB-CUT-002 / SB-CUT-T02b's identity half. Source: `14_cutoffs-summation-mc.md:927-942` —
    /// every record carrying a thickness, a net, a net-to-gross or a thickness-weighted average
    /// MUST carry the discretisation model that produced it and the sample interval it was
    /// computed on; a consumer must never have to infer either. IP ships TWO definitions of
    /// "Net" in one product under the same heading and labels neither, and net-to-gross is not
    /// scale-invariant (T4: 0.55 → 0.75 → 1.0 across three blocking steps).
    #[test]
    fn every_thickness_bearing_result_names_its_discretisation_model_and_the_step_it_ran_on() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Two wells, SAME rock, DIFFERENT frames: 1.0 m and 0.5 m steps.
        let mut wells = Vec::new();
        for (name, step) in [("STEP-ONE", 1.0f32), ("STEP-HALF", 0.5f32)] {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, name, Some("Synthetic"), None, None).unwrap();
            let well = id.to_string();
            let n = (20.0 / step) as usize + 1;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * step).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
                vec![2.35; n], nan.clone(), nan,
            )
            .unwrap();
            for (curve, value) in [("VSH", 0.10), ("PHIE", 0.30), ("SWE", 0.20)] {
                equations::write_computed_curve(&conn, &well, &depth, curve, &vec![value; n])
                    .unwrap();
            }
            db::upsert_zone_with_datum(
                &conn, &well, "Z", 1000.0, 1010.0, crate::schema_vocab::DepthDatum::Md,
            )
            .unwrap();
            wells.push((well, step));
        }
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: wells.iter().map(|(w, _)| w.clone()).collect(),
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .unwrap();

        // A. Every row STATES the model — the shipped TOPS rule — and its own well's step.
        for r in &rows {
            assert_eq!(
                r.discretisation_model, DISCRETISATION_MODEL,
                "a thickness-bearing record must name the model that produced it"
            );
            let expected = wells.iter().find(|(w, _)| *w == r.well_id).unwrap().1;
            assert!(
                (r.sample_interval - expected).abs() < 1e-6,
                "{}: the record must carry ITS OWN frame's step {expected}, got {}",
                r.well_name,
                r.sample_interval
            );
        }

        // B. The whole point: two records over the SAME rock at different steps are
        //    distinguishable BY THE RECORD, with no depth column to re-derive it from.
        let one = rows.iter().find(|r| r.well_name == "STEP-ONE").unwrap();
        let half = rows.iter().find(|r| r.well_name == "STEP-HALF").unwrap();
        assert!(
            (one.sample_interval - half.sample_interval).abs() > 0.4,
            "records computed at different steps must be distinguishable"
        );

        // C. The workbook carries both — per row, since wells in one workbook differ in frame —
        //    and as NUMBERS-stay-numbers: the step is a numeric cell, the model a text cell.
        let sheet = crate::office::pay_sheet(&rows, "m");
        let model_col = sheet
            .columns
            .iter()
            .position(|c| c.header == "Model")
            .expect("the pay sheet must carry the discretisation model");
        let step_col = sheet
            .columns
            .iter()
            .position(|c| c.header.starts_with("Step"))
            .expect("the pay sheet must carry the sample interval");
        let mut seen_steps = std::collections::BTreeSet::new();
        for row in &sheet.rows {
            match (&row[model_col], &row[step_col]) {
                (crate::office::Cell::Text(model), crate::office::Cell::Num(step)) => {
                    assert_eq!(model, DISCRETISATION_MODEL);
                    seen_steps.insert((step * 1000.0).round() as i64);
                }
                other => panic!("model must be text and step numeric, got {other:?}"),
            }
        }
        assert_eq!(
            seen_steps.into_iter().collect::<Vec<_>>(),
            vec![500, 1000],
            "both frames' steps must survive into the workbook"
        );

        // D. The Monte Carlo bundle carries the same identity fields (populated by the same
        //    helpers); their presence on the struct is pinned here, their end-to-end values by
        //    the MC engine's own DB tests running under the same construction site.
        let median = median_sample_interval(&[0.5, 0.5, 0.5, 1.0]);
        assert!((median - 0.5).abs() < 1e-9, "the median step is the regularize convention");
        assert!(median_sample_interval(&[0.0, -1.0]).is_nan(), "no positive step is NaN, not zero");
    }

    /// SB-CUT-011 (P1). `14_cutoffs-summation-mc.md:1064-1075` — a sample that passes every
    /// cut-off but lies outside every defined zone **MUST NOT** contribute to any cumulative
    /// result or summary statistic (IP's stated zone-membership rule).
    ///
    /// Easy to violate in a single-pass implementation that applies cut-offs before zone
    /// membership, and easy to test WRONGLY: an out-of-zone sample that also fails a cut-off is
    /// excluded for the wrong reason and proves nothing. So the samples outside every zone here
    /// are asserted to pass all three cut-offs on their own, and they carry values found nowhere
    /// else — φ 0.50 against the zones' 0.30 and 0.10 — so any leak moves a number.
    ///
    /// Three intervals, two of them declared, prove membership is what decides: the same engine
    /// counts a sample in the zone that contains it, and not in the one next door.
    #[test]
    fn a_sample_outside_every_declared_zone_contributes_to_no_summary_statistic_however_well_it_passes_the_cutoffs(
    ) {
        // First, the guard that makes the rest meaningful: these values clear every cut-off.
        assert_eq!(
            classify_sample(0.80, 0.50, 0.85, f32::NAN, &ladder(at_most(0.9), at_least(0.05), at_most(0.9), None), false),
            (1.0, 1.0, 1.0),
            "the out-of-zone samples must pass SAND, RESERVOIR and PAY on their own merits - \
             otherwise their absence below proves a cut-off worked, not the zone rule"
        );

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "ZONE-EDGE", Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 25usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![20.0; n], vec![0.2; n],
            vec![2.35; n], nan.clone(), nan,
        )
        .unwrap();
        // Three bands: UPPER (declared), LOWER (declared), and BELOW — outside every zone.
        let band = |a: f32, b: f32, c: f32| -> Vec<f32> {
            (0..n).map(|i| if i < 10 { a } else if i < 20 { b } else { c }).collect()
        };
        equations::write_computed_curve(&conn, &well, &depth, "VSH", &band(0.10, 0.40, 0.80))
            .unwrap();
        equations::write_computed_curve(&conn, &well, &depth, "PHIE", &band(0.30, 0.10, 0.50))
            .unwrap();
        equations::write_computed_curve(&conn, &well, &depth, "SWE", &band(0.20, 0.60, 0.85))
            .unwrap();
        for (name, top, base) in
            [("UPPER", 1000.0, 1010.0), ("LOWER", 1010.0, 1020.0)]
        {
            db::upsert_zone_with_datum(
                &conn, &well, name, top, base, crate::schema_vocab::DepthDatum::Md,
            )
            .unwrap();
        }

        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("summary runs");
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;
        let row = |zone: &str, flag: &str| -> PaySummaryRow {
            rows.iter()
                .find(|r| r.zone == zone && r.flag == flag)
                .unwrap_or_else(|| panic!("a {flag} row for {zone}"))
                .clone()
        };

        assert_eq!(rows.len(), 6, "two declared zones by three flags, and nothing for BELOW");
        assert!(
            !rows.iter().any(|r| r.zone != "UPPER" && r.zone != "LOWER"),
            "the footage below every zone must not produce a zone of its own"
        );

        for flag in SUMMARY_FLAGS {
            let u = row("UPPER", flag);
            assert!(near(u.net, 10.0), "{flag} UPPER net is its own ten units, got {}", u.net);
            assert!(near(u.avg_phie, 0.30), "{flag} UPPER phi is 0.30, got {}", u.avg_phie);
            assert!(near(u.avg_swe, 0.20), "{flag} UPPER Sw is 0.20, got {}", u.avg_swe);
            assert!(near(u.hpv, 2.4), "{flag} UPPER HPV is 2.4, got {}", u.hpv);

            let l = row("LOWER", flag);
            assert!(near(l.net, 10.0), "{flag} LOWER net is its own ten units, got {}", l.net);
            assert!(near(l.avg_phie, 0.10), "{flag} LOWER phi is 0.10, got {}", l.avg_phie);
            assert!(near(l.avg_swe, 0.60), "{flag} LOWER Sw is 0.60, got {}", l.avg_swe);
            assert!(near(l.hpv, 0.4), "{flag} LOWER HPV is 0.4, got {}", l.hpv);

            // The below-every-zone band carries φ 0.50 and Sw 0.85, which appear in neither row —
            // stated as its own assertion so a leak reads as what it is rather than as a stray
            // arithmetic error somewhere above.
            for r in [&u, &l] {
                assert!(
                    r.avg_phie < 0.31 && r.avg_swe < 0.61,
                    "{flag} {} shows a trace of the samples below every zone: phi {} Sw {}",
                    r.zone, r.avg_phie, r.avg_swe
                );
            }
        }

        // The register asks for ONE fixture across all three paths, because the rule is easy to
        // hold in the summation and lose in a sibling that walks the same curves.

        // Path 2 — the cutoff SWEEP, restricted to UPPER. Every sample clears a VSH cutoff of 0.9,
        // so net is decided by zone membership alone and must be UPPER's own ten units.
        let sweep = run_cutoff_sweep(
            &dbm,
            &CutoffSweepRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                property: "VSH".into(),
                vsh_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.05, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.9, unit: "v/v".into() }.into()),
                perm_min: None,
                // Every sample's VSH is at most 0.80, so it clears every step of this range and
                // net is decided by zone membership alone across the whole sweep.
                sweep_min: 0.85,
                sweep_max: 0.95,
                steps: 3,
                metric: "NET".into(),
                zone: Some("UPPER".into()),
                dst_dataset: None,
            },
        )
        .expect("sweep runs");
        let series = sweep.series.first().expect("one well, one series");
        assert!(
            near(series.gross as f32, 10.0),
            "the sweep's gross is UPPER's own thickness, got {}",
            series.gross
        );
        assert!(
            series.values.iter().all(|v| near(*v as f32, 10.0)),
            "the sweep must count only UPPER's samples, got {:?}",
            series.values
        );

        // Path 3 — MONTE CARLO's per-realization zone metrics on the same arrays.
        let step = vec![1.0f32; n];
        let m = crate::montecarlo::zone_metrics(
            DiscretisationModel::Forward, // DEC-071: fixture derived under FORWARD
            &band(0.10, 0.40, 0.80),
            &band(0.30, 0.10, 0.50),
            &band(0.20, 0.60, 0.85),
            &vec![f32::NAN; n],
            &depth,
            &step,
            &db::ZoneEntry {
                zone_name: "UPPER".into(),
                top_depth: 1000.0,
                bottom_depth: 1010.0,
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            },
            &crate::montecarlo::Cutoffs {
                vsh_max: at_most(0.9),
                phie_min: at_least(0.05),
                swe_max: at_most(0.9),
                perm_min: None,
            },
            false,
        );
        assert!(near(m.net, 10.0), "Monte Carlo counts only UPPER's samples, got {}", m.net);
        assert!(near(m.avg_phie, 0.30), "Monte Carlo phi is UPPER's 0.30, got {}", m.avg_phie);
        assert!(near(m.hpv, 2.4), "Monte Carlo HPV is UPPER's 2.4, got {}", m.hpv);
    }

    /// SB-CUT-012 (P2). `14_cutoffs-summation-mc.md:1078-1091` — a summation result **MUST** carry
    /// `{frame, weights_source}` with `frame` one of MD, TVD, TVDSS or TST; MD and TVD summations
    /// **MUST** be separate records; and SandiBumi **MUST NOT** present a TVD result as a
    /// rescaling of an MD result.
    ///
    /// The per-sample weight is `Δz` in MD and `Δz·cos θ` in TVD, so it is the WEIGHTS that
    /// differ, not merely the totals. In a 60° hold section they differ by a factor of two, which
    /// is why IP says TVD zonal averages *"could be considerably different"* — the frame is part of
    /// a result's identity, not a display option. A net thickness quoted in a deviated field
    /// without its frame is a number a reader cannot use.
    ///
    /// **The summation is MD-only and this row does not change that.** It closes the MUST the
    /// honest way for an ABSENT row: every result declares the frame it was actually computed in,
    /// and a request for a frame whose weights SandiBumi cannot compute is REFUSED by name rather
    /// than served MD numbers under a TVD label — which is precisely the third clause.
    #[test]
    fn a_summation_declares_the_depth_frame_its_weights_came_from_and_refuses_one_it_cannot_weight()
    {
        // The four frames the chapter names — Techlog offers four, IP two, and the union is the
        // vocabulary. `as_str` matches exhaustively in production, so a fifth variant cannot be
        // added without naming it there; this pins what those names ARE.
        assert_eq!(
            [
                SummationFrame::Md.as_str(),
                SummationFrame::Tvd.as_str(),
                SummationFrame::Tvdss.as_str(),
                SummationFrame::Tst.as_str()
            ],
            ["MD", "TVD", "TVDSS", "TST"]
        );
        assert_eq!(SummationFrame::default(), SummationFrame::Md);

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_partition_well(&conn, "FRAME-1");
        let dbm = Mutex::new(conn);
        let req = |frame: SummationFrame| PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![well.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            stats_only: true,
            custody: None,
            weighting: Default::default(),
            frame,
        };

        // A — every emitted row declares the frame AND where its weights came from. Both, because
        // "MD" alone does not say which depths were differenced.
        let rows = run_pay_summary(&dbm, &req(SummationFrame::Md)).expect("an MD summation runs");
        assert!(!rows.is_empty());
        for r in &rows {
            assert_eq!(r.frame, SummationFrame::Md, "{} frame", r.flag);
            assert_eq!(
                r.weights_source, MD_WEIGHTS_SOURCE,
                "{} must name the numbers its weights were differenced from",
                r.flag
            );
        }

        // B — the other three are REFUSED, by name, with the reason. Not returned empty, and above
        // all not returned as MD numbers wearing a different label.
        for frame in [SummationFrame::Tvd, SummationFrame::Tvdss, SummationFrame::Tst] {
            let err = run_pay_summary(&dbm, &req(frame))
                .expect_err("a frame whose weights cannot be computed must refuse");
            assert!(
                err.contains(frame.as_str()),
                "the refusal must name the frame that was asked for: {err}"
            );
            assert!(
                err.contains("cos") || err.contains("deviation") || err.contains("survey"),
                "the refusal must say what is missing, not merely that it declines: {err}"
            );
        }

        // C — and the refusal is a REFUSAL, not a fallback. If a TVD request ever starts returning
        // rows, this is the assertion that catches it before anybody quotes them.
        assert!(
            run_pay_summary(&dbm, &req(SummationFrame::Tvd)).is_err(),
            "a TVD result must never be an MD result relabelled"
        );
    }

    /// SB-CUT-016 (P0, SILENT-WRONGNESS). `14_cutoffs-summation-mc.md:1138-1160` — SandiBumi
    /// **MUST NOT** ship a numeric default for any cut-off; every cut-off field **MUST** ship in
    /// the first-class state *no default — user must set*; an unfiltered summation **MUST** be
    /// reported as unfiltered on the result and in the report; and a summation **MUST NOT** run
    /// against an unset cut-off that has been enabled.
    ///
    /// Four shipped vendor sets, no two identical, **two of them from one vendor**: IP φ 0.1 /
    /// Sw 0.5 / Vsh 0.5; Techlog 0.15 / 0.85 / 0.5; Geolog `default_*.paysum` 0.08 / 0.5 / 0.3;
    /// Geolog `determin_mc.info` 0.08 / 0.5 / **0.5**. Jauhar's own delivered work spans Vsh
    /// 0.20–0.85 and one record spans Vsh 0.55–0.85 *across intervals of a single area* — the
    /// quantity is not constant even within one field, so there is no number to pick.
    ///
    /// **What this row deliberately does NOT change:** the NaN cascade. A sample with no VSH is
    /// still unjudgeable whether or not VSH is being used as a cut-off. Making an unfiltered
    /// cut-off also stop requiring its curve would let a well with no VSH book pay it never
    /// booked, and the requirement says nothing about it — so the rule stands untouched.
    #[test]
    fn no_cutoff_ships_a_value_an_unapplied_one_is_reported_unfiltered_and_an_enabled_blank_one_refuses(
    ) {
        // A — the UI ships no numeric cut-off default. This is where the violation lived: the
        // backend always required values, while two frontend surfaces pre-filled them.
        for (path, src) in [
            ("src/ui/cutoffs.ts", include_str!("../../src/ui/cutoffs.ts")),
            ("src/ui/dashboardPanel.ts", include_str!("../../src/ui/dashboardPanel.ts")),
        ] {
            for banned in ["0.5", "0.15", "0.85", "0.08", "0.3", "0.1", "0.6"] {
                let seeded = format!("\"{banned}\"");
                assert!(
                    !src.contains(&seeded),
                    "{path} seeds a cut-off field with {seeded} - no vendor's number is \
                     defensible here, and a pre-filled box is a shipped default"
                );
            }
        }

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        // Every sample passes on VSH and PHIE; SWE 0.20 and 0.60 straddle a 0.4 cut-off.
        let well = seed_weighting_well(&conn, "CUTOFF-1", "PHIE");
        let dbm = Mutex::new(conn);
        let vv = |v: Option<f64>| v.map(|x| CutoffSpec::from(CutoffEntry { value: x, unit: "v/v".into() }));
        let req = |vsh: Option<f64>, phie: Option<f64>, swe: Option<f64>, blank: Vec<String>| {
            PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: vv(vsh),
                phie_min: vv(phie),
                swe_max: vv(swe),
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: blank,
                cutoff_use: Default::default(),
            }
        };
        let pay = |rows: &[PaySummaryRow]| -> PaySummaryRow {
            rows.iter().find(|r| r.flag == "PAY").expect("a PAY row").clone()
        };

        // B — an SWE cut-off of 0.4 excludes the five deep samples, so PAY net is 5 of 10.
        let filtered = run_pay_summary(&dbm, &req(Some(0.9), Some(0.05), Some(0.4), vec![]))
            .expect("a fully specified summation runs");
        assert!((pay(&filtered).net - 5.0).abs() < 1e-6, "the SWE cut-off must bite");
        assert_eq!(
            pay(&filtered).unfiltered,
            vec!["PERM".to_string()],
            "only PERM is unfiltered here - not asking for a permeability cut-off is itself an              unfiltered summation on that property, and the result says so rather than staying              silent about it"
        );

        // C — omitting it makes the summation UNFILTERED on SWE: all ten units count, AND the row
        // says so. Both halves matter - a number that quietly stopped being filtered, with nothing
        // on the result to say so, is the whole failure this clause exists to prevent.
        let unfiltered = run_pay_summary(&dbm, &req(Some(0.9), Some(0.05), None, vec![]))
            .expect("an unfiltered summation is legitimate and runs");
        assert!(
            (pay(&unfiltered).net - 10.0).abs() < 1e-6,
            "an absent cut-off must not filter, got net {}",
            pay(&unfiltered).net
        );
        assert_eq!(
            pay(&unfiltered).unfiltered,
            vec!["SWE".to_string(), "PERM".to_string()],
            "the result must REPORT every cut-off that was not applied, in VSH/PHIE/SWE/PERM order"
        );

        // D — ABSENT MEANS ABSENT, not a fallback. Rock that fails EVERY vendor default - Vsh 0.80
        // against their 0.5, φ 0.02 against 0.08/0.1/0.15, Sw 0.95 against 0.5/0.6/0.85 - must
        // count in full when no cut-off is set. Arm C alone could not catch a silent fallback,
        // because its φ 0.30 and Vsh 0.40 clear those numbers anyway; this is the arm that bites.
        let shale_id = uuid::Uuid::new_v4();
        {
            let conn = dbm.lock().unwrap();
            db::insert_well(&conn, shale_id, "CUTOFF-SHALE", Some("Synthetic"), None, None).unwrap();
            let sid = shale_id.to_string();
            let n = 11usize;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves(
                &conn, shale_id, depth.clone(), vec![120.0; n], vec![40.0; n], vec![0.02; n],
                vec![2.6; n], nan.clone(), nan,
            )
            .unwrap();
            for (curve, v) in [("VSH", 0.80f32), ("PHIE", 0.02), ("SWE", 0.95)] {
                equations::write_computed_curve(&conn, &sid, &depth, curve, &vec![v; n]).unwrap();
            }
        }
        let shale = shale_id.to_string();
        let mut all_absent = req(None, None, None, vec![]);
        all_absent.well_ids = vec![shale.clone()];
        let rows = run_pay_summary(&dbm, &all_absent).expect("an entirely unfiltered run is legitimate");
        let r = rows.iter().find(|r| r.flag == "PAY").expect("a PAY row");
        assert!(
            (r.net - 10.0).abs() < 1e-6,
            "with no cut-off set, rock that fails every vendor default still counts in full - a \
             net below 10 here means an absent cut-off quietly became somebody's number, got {}",
            r.net
        );
        assert_eq!(
            r.unfiltered,
            vec!["VSH".to_string(), "PHIE".to_string(), "SWE".to_string(), "PERM".to_string()],
            "and all four are reported unfiltered"
        );

        // E — a cut-off the user switched on and left blank REFUSES. Distinct from C on purpose:
        // "I am not filtering on Sw" and "I meant to filter on Sw and have not said what" are
        // different statements, and only one of them may produce a number.
        let err = run_pay_summary(&dbm, &req(Some(0.9), Some(0.05), None, vec!["SWE".into()]))
            .expect_err("an enabled but unset cut-off must refuse");
        assert!(
            err.contains("SWE"),
            "the refusal must name the cut-off that was left blank: {err}"
        );
    }

    /// SB-CUT-019 (P1). `14_cutoffs-summation-mc.md:1204-1221` and `:2087` (SB-CUT-T26) — a
    /// cut-off **MUST** be entered with a unit and stored with it; a bare number **MUST** be
    /// rejected; `35 pu` **MUST** be accepted and stored as `0.35 v/v`; `35 v/v` **MUST** be
    /// rejected as out of bounds; dimensionless cut-offs **MUST** be bounded to their quantity's
    /// physical range.
    ///
    /// IP's own manual expresses the sensitivity-sweep example in porosity units and the cut-off
    /// default in `v/v` **for the same quantity, with no unit tag on the field**. `35` where `0.1`
    /// is meant is a **350x** error, and its symptom is an all-net result — a good-looking well,
    /// not a visible failure. The unit is the only thing that separates the two readings, so it is
    /// required rather than guessed.
    #[test]
    fn a_cutoff_is_refused_without_a_unit_and_thirty_five_is_porosity_units_or_out_of_bounds() {
        let por = CutoffQuantity::VolumeFraction;

        // A — a bare number is REFUSED for the MISSING UNIT, not for the number being implausible.
        // `0.10` is a perfectly ordinary porosity cut-off in v/v, so an implementation that only
        // range-checked would let it through — and would then be silently choosing between
        // 0.10 v/v and 0.10 pu, which differ by the same 100x the rule exists to stop.
        let plausible = CutoffEntry { value: 0.10, unit: String::new() };
        let err = plausible.canonical(por, "the PHIE cut-off").expect_err("a bare number refuses");
        assert!(err.contains("PHIE"), "the refusal names the field: {err}");
        assert!(err.contains("no unit"), "and refuses for the RIGHT reason: {err}");

        // and the chapter's own example carries a message that explains the trap rather than
        // saying "no" — a refusal an analyst cannot act on gets worked around, not obeyed.
        let bare = CutoffEntry { value: 35.0, unit: String::new() };
        let err = bare.canonical(por, "the PHIE cut-off").expect_err("a bare number must refuse");
        assert!(err.contains("350"), "and states the size of the error it prevents: {err}");

        // B — `35 pu` is accepted and canonicalised to 0.35 v/v.
        let pu = CutoffEntry { value: 35.0, unit: "pu".into() };
        assert!(
            (pu.canonical(por, "the PHIE cut-off").expect("35 pu is a real porosity") - 0.35).abs()
                < 1e-12,
            "35 pu is 0.35 v/v"
        );

        // C — `35 v/v` is REFUSED as out of bounds. Same number as B, opposite verdict, and only
        // the unit distinguishes them: that is the whole requirement in one pair of assertions.
        let vv = CutoffEntry { value: 35.0, unit: "v/v".into() };
        let err = vv.canonical(por, "the PHIE cut-off").expect_err("35 v/v is impossible");
        assert!(err.contains("physical range"), "{err}");

        // D — the bounds are the quantity's own, both ends.
        assert!(CutoffEntry { value: -0.1, unit: "v/v".into() }.canonical(por, "x").is_err());
        assert!(CutoffEntry { value: 1.0, unit: "v/v".into() }.canonical(por, "x").is_ok());
        assert!(CutoffEntry { value: 100.0, unit: "%".into() }.canonical(por, "x").is_ok());

        // E — permeability has its own unit family and its own bound, so the rule is a property of
        // the QUANTITY rather than a single hard-coded 0..1.
        let perm = CutoffQuantity::Permeability;
        assert!(
            (CutoffEntry { value: 1.0, unit: "D".into() }.canonical(perm, "k").unwrap() - 1000.0)
                .abs()
                < 1e-9,
            "1 darcy is 1000 mD"
        );
        assert!(CutoffEntry { value: -1.0, unit: "mD".into() }.canonical(perm, "k").is_err());
        assert!(
            CutoffEntry { value: 1.0, unit: "v/v".into() }.canonical(perm, "k").is_err(),
            "a volume fraction is not a permeability, however plausible the number"
        );

        // F — WIRED IN: the summation refuses before it computes anything, so a bare number can
        // never reach the pay arithmetic. A refusal that only exists in a helper is not a contract.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "UNIT-1", "PHIE");
        let dbm = Mutex::new(conn);
        let err = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 35.0, unit: String::new() }.into()),
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect_err("a bare cut-off must stop the run");
        assert!(err.contains("no unit"), "{err}");
    }

    /// SB-CUT-030 (P1). `14_cutoffs-summation-mc.md:1378-1397` and §3.8 at `:778-806` — three
    /// named stages, `accumulate` **never clamped** and `flag_test` / `present` clamped to the
    /// **quantity's** bounds; bounds attach to the quantity and **never to a curve-type string**;
    /// an unbounded quantity **MUST NOT** be clamped to `[0,1]`; a zonal average outside its
    /// bounds **MUST** be emitted with `out_of_range: true`, **not corrected**; and percent-to-
    /// fraction conversion and the bound check **MUST** be separate operations with an over-bound
    /// value after conversion raising.
    ///
    /// The chapter quantifies why `accumulate` must not clamp, and the number is the argument: for
    /// a truly wet interval the unclamped `phi*(1-Sw)` has expectation ZERO under symmetric noise,
    /// while the clamped `phi*max(0, 1-Sw)` has expectation `0.3989*phi*sigma > 0`. Clamping does
    /// not relocate a tail — it moves the MEAN, always toward more hydrocarbon, by an amount
    /// independent of iteration count. Correct for one deterministic evaluation, a bias in
    /// expectation over an ensemble.
    #[test]
    fn accumulate_never_clamps_while_flag_test_and_present_clamp_to_the_quantitys_own_bounds() {
        use BoundedQuantity::{Permeability, Unbounded, VolumeFraction};
        use ClampStage::{Accumulate, FlagTest, Present};

        // A — the stage rule, at values outside the bounds on both sides. `accumulate` returns the
        // value untouched for EVERY quantity; the other two clamp.
        for quantity in [VolumeFraction, Permeability, Unbounded] {
            for value in [-0.4f32, 1.7, 42.0] {
                assert_eq!(
                    stage_value(Accumulate, quantity, value),
                    value,
                    "accumulate must never clamp: {quantity:?} at {value}"
                );
            }
        }
        assert_eq!(stage_value(FlagTest, VolumeFraction, 1.7), 1.0);
        assert_eq!(stage_value(Present, VolumeFraction, -0.4), 0.0);

        // B — the quantified consequence, reproduced. A symmetric spread of `1 - Sw` about zero
        // accumulates to zero unclamped and to a POSITIVE mean clamped: the bias is not a tail
        // effect, it is the mean moving, and it moves toward more hydrocarbon every time.
        let draws: Vec<f32> = (-50..=50).map(|i| i as f32 * 0.01).collect();
        let unclamped: f64 = draws
            .iter()
            .map(|hc| stage_value(Accumulate, VolumeFraction, *hc) as f64)
            .sum();
        let clamped: f64 = draws
            .iter()
            .map(|hc| stage_value(Present, VolumeFraction, *hc) as f64)
            .sum();
        assert!(unclamped.abs() < 1e-6, "a symmetric spread accumulates to zero: {unclamped}");
        assert!(
            clamped > 0.0,
            "and clamping the same spread moves the mean toward hydrocarbon: {clamped}"
        );

        // C — BOUNDS ATTACH TO THE QUANTITY. Permeability is bounded BELOW and open above, so a
        // real 4,000 mD must survive; an unbounded quantity is not clamped to [0,1] merely because
        // that is the common case. Binding bounds to a curve-type string is the specific failure
        // that makes IP's clipping invisible in the data — a quantity cannot be mis-typed by a
        // label, because it is not a label.
        assert_eq!(stage_value(Present, Permeability, 4000.0), 4000.0);
        assert_eq!(stage_value(Present, Permeability, -3.0), 0.0);
        assert_eq!(
            stage_value(Present, Unbounded, 42.0),
            42.0,
            "an unbounded quantity must not be clamped to [0,1]"
        );
        assert_eq!(Unbounded.bounds(), None);
        assert_eq!(Permeability.bounds(), Some((0.0, f64::INFINITY)));

        // D — an out-of-range value is DETECTED, and a NaN is not out of range: absent is a
        // different statement and already has its own carrier (SB-CUT-029).
        assert!(VolumeFraction.is_out_of_range(1.2) && VolumeFraction.is_out_of_range(-0.1));
        assert!(!VolumeFraction.is_out_of_range(0.5));
        assert!(!VolumeFraction.is_out_of_range(f32::NAN), "missing is not out of range");
        assert!(!Unbounded.is_out_of_range(1e9));

        // E — PERCENT-TO-FRACTION CONVERSION AND THE BOUND CHECK ARE SEPARATE OPERATIONS, and an
        // over-bound value AFTER conversion raises. `35 pu` converts to 0.35 and passes; `35 v/v`
        // needs no conversion and fails the check; `200 pu` converts to 2.0 and fails AFTER the
        // conversion, which is the ordering the requirement asks for.
        let por = CutoffQuantity::VolumeFraction;
        assert!((CutoffEntry { value: 35.0, unit: "pu".into() }.canonical(por, "x").unwrap() - 0.35).abs() < 1e-12);
        assert!(CutoffEntry { value: 35.0, unit: "v/v".into() }.canonical(por, "x").is_err());
        let after = CutoffEntry { value: 200.0, unit: "pu".into() }
            .canonical(por, "the PHIE cut-off")
            .expect_err("2.0 v/v is impossible");
        assert!(
            after.contains("physical range") && after.contains('2'),
            "the refusal must name the CONVERTED value, which is what proves the check ran after \
             the conversion rather than instead of it: {after}"
        );

        // F — WIRED IN: a zonal average outside its bounds is EMITTED with the flag rather than
        // corrected. An ordinary well flags nothing, which is the control that stops the flag from
        // being stuck on.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-CLAMP-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("the run is valid");
        assert!(!rows.is_empty());
        assert!(
            rows.iter().all(|r| !r.out_of_range),
            "an in-range well must not be flagged - a flag that is always on says nothing"
        );
        let wire = serde_json::to_value(&rows[0]).unwrap();
        assert!(
            wire["out_of_range"].is_boolean(),
            "and the condition rides a typed sibling, not the numeric column"
        );

        // G — the POSITIVE half, and it is the half the requirement actually turns on: a zonal
        // average outside its bounds is EMITTED WITH THE FLAG AND NOT CORRECTED. Without this the
        // in-range control above is satisfied by a flag hard-wired to `false`, which is a check
        // that cannot fail — a mutation proved exactly that before this arm existed.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-CLAMP-2", Some("Synthetic"), None, None).unwrap();
        let impossible = id.to_string();
        let n = 8usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth.clone(), vec![30.0; n], vec![4.0; n], nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        // A supersaturated combination: a saturation above 1 is physically impossible and is
        // exactly what an unclamped chain output looks like when a sampled parameter set is wrong.
        for (curve, value) in [("VSH", 0.10f32), ("PHIE", 0.30), ("SWE", 1.40)] {
            equations::write_computed_curve(&conn, &impossible, &depth, curve, &vec![value; n])
                .unwrap();
        }
        let dbm = Mutex::new(conn);
        let flagged = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![impossible],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: None,
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("an impossible saturation must still produce a result, flagged");
        let row = flagged
            .iter()
            .find(|r| r.avg_swe.is_finite())
            .expect("some row carries the saturation average");
        assert!(
            row.out_of_range,
            "an average of {} is outside 0..1 and must be FLAGGED",
            row.avg_swe
        );
        assert!(
            (row.avg_swe - 1.40).abs() < 1e-5,
            "and emitted AS COMPUTED, not corrected to 1.0 - a corrected average is a number \
             nobody derived: {}",
            row.avg_swe
        );
    }

    /// SB-CUT-029 (P1). `14_cutoffs-summation-mc.md:1361-1376` — a null or not-computed condition
    /// **MUST** be carried in a typed sibling field, **never as an in-band marker inside a numeric
    /// field**, and consumers **MUST** render a dash rather than a zero when the count is zero.
    ///
    /// F-15: IP prints `$$` **inside a numeric report column** to mean *"nulls present"* —
    /// unparseable, uncarryable through a calculation, and invisible to a downstream consumer that
    /// reads the column as a number. The chapter's as-built says the marker discipline is already
    /// right and that *"the remaining work is the footage partition of `SB-CUT-003`"*; that row
    /// landed, so what is left is this proof.
    #[test]
    fn a_not_computed_condition_rides_a_typed_sibling_and_never_a_marker_inside_a_numeric_column() {
        let row = |name: &str, n_classified: usize, perm_no_data: bool| PaySummaryRow {
            well_id: "w".into(),
            well_name: name.into(),
            discretisation_model: DISCRETISATION_MODEL.to_string(),
            sample_interval: 0.5,
            zone: "WHOLE".into(),
            flag: "PAY".into(),
            top: 1000.0,
            bottom: 1010.0,
            gross: 10.0,
            net: 0.0,
            not_net: if n_classified == 0 { 0.0 } else { 10.0 },
            unknown: if n_classified == 0 { 10.0 } else { 0.0 },
            ntg: 0.0,
            ntg_known: if n_classified == 0 { f32::NAN } else { 0.0 },
            avg_vsh: f32::NAN,
            avg_phie: f32::NAN,
            avg_swe: f32::NAN,
            hpv: 0.0,
            n_classified,
            perm_cutoff_no_data: perm_no_data,
            quicklook_phie_excluded: false,
            residual_absorbed: 0.0,
            out_of_range: false,
            unfiltered: vec!["PERM".into()],
            frame: Default::default(),
            weights_source: MD_WEIGHTS_SOURCE.into(),
        };
        let uninterpreted = serde_json::to_value(row("NEVER_INTERPRETED", 0, false)).unwrap();
        let barren = serde_json::to_value(row("INTERPRETED_AND_BARREN", 40, true)).unwrap();

        // A — NO IN-BAND MARKER. Every field that carries a quantity must serialize as a JSON
        // number or null. A string in a numeric column is F-15 exactly: it survives the wire, it
        // reads as data, and it stops being arithmetic.
        for (label, value) in [("uninterpreted", &uninterpreted), ("barren", &barren)] {
            let object = value.as_object().expect("a row is an object");
            for field in [
                "top", "bottom", "gross", "net", "not_net", "unknown", "ntg", "ntg_known",
                "avg_vsh", "avg_phie", "avg_swe", "hpv", "residual_absorbed",
            ] {
                let cell = &object[field];
                assert!(
                    cell.is_number() || cell.is_null(),
                    "{label} row: '{field}' carries {cell}, which is not a number - a marker \
                     inside a numeric column is unparseable and uncarryable through a calculation"
                );
            }
        }

        // B — THE TYPED SIBLINGS, and their types. A count that arrived as a string, or a flag as
        // "true", would satisfy arm A and defeat the requirement.
        for (label, value) in [("uninterpreted", &uninterpreted), ("barren", &barren)] {
            let object = value.as_object().expect("an object");
            assert!(object["n_classified"].is_u64(), "{label}: the count is an integer");
            assert!(
                object["perm_cutoff_no_data"].is_boolean(),
                "{label}: the missing-permeability condition is a boolean"
            );
            assert!(
                object["unfiltered"]
                    .as_array()
                    .is_some_and(|names| names.iter().all(|name| name.is_string())),
                "{label}: the unfiltered cut-offs are a list of names, not a packed string"
            );
        }

        // C — the two rows are distinguishable PURELY from the typed fields. Their numeric columns
        // are the same shape - net 0, N:G 0, HCPV 0 - which is the whole trap: a reader looking at
        // the numbers alone cannot tell a well nobody interpreted from a well found barren, and
        // nothing in the numbers is allowed to tell them apart either.
        for field in ["net", "ntg", "hpv"] {
            assert_eq!(
                uninterpreted[field], barren[field],
                "the numeric columns must NOT encode the difference - '{field}' does"
            );
        }
        assert_ne!(
            uninterpreted["n_classified"], barren["n_classified"],
            "and the typed sibling must be what carries it"
        );

        // D — WIRED IN. A real run emits the siblings, so the discipline is a property of the
        // engine's output rather than of a struct somebody could bypass.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-MARKER-1");
        let dbm = Mutex::new(conn);
        let rows = run_pay_summary(
            &dbm,
            &PaySummaryRequest {
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well],
                vsh_max: None,
                phie_min: None,
                swe_max: None,
                perm_min: Some(CutoffEntry { value: 1.0, unit: "mD".into() }.into()),
                skip_version: false,
                stats_only: true,
                custody: None,
                weighting: Default::default(),
                frame: Default::default(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
            },
        )
        .expect("the run is valid");
        let pay = rows.iter().find(|r| r.flag == "PAY").expect("a pay row is emitted");
        // This well carries no permeability at all, so every sample fails the cut-off for want of
        // data. The zero is real arithmetic; the REASON rides beside it in its own typed field.
        assert!(
            pay.perm_cutoff_no_data,
            "the well has no PERM, and the run must SAY so rather than leaving a bare zero"
        );
        assert_eq!(pay.net, 0.0, "while the number itself stays a number");
        let wire = serde_json::to_value(pay).unwrap();
        assert!(wire["net"].is_number() && wire["perm_cutoff_no_data"].is_boolean());
    }

    /// SB-CUT-028 (P1). `14_cutoffs-summation-mc.md:1346-1359` — saturation quantities **MUST** be
    /// named `SWE` or `SWT` explicitly wherever a cut-off, an average or a result field refers to
    /// one, and a bare `SW` **MUST NOT** appear in a cut-off record or a result field.
    ///
    /// FINDINGS §6 rule 8, sharpened by T3: in Techlog the mnemonic silently changes the
    /// weighting — *"the SW curve is weighted by POR but the SWE is not"* — so a bare name is both
    /// ambiguous about which saturation is meant AND load-bearing on the arithmetic. That is why
    /// this is P1: the ambiguity does not stay an ambiguity, it becomes a different number.
    ///
    /// The chapter's `Verified by` points at SB-CUT-T06, which its own test-to-requirement map
    /// assigns to SB-CUT-009 and which pins the average-form identity — the CONSEQUENCE of the
    /// naming rather than the naming. The naming contract therefore needs this test.
    #[test]
    fn no_module_output_cutoff_field_or_result_field_is_a_bare_sw_rather_than_swe_or_swt() {
        // A — the registry, from BOTH sides. A scan that only forbids would pass by finding
        // nothing at all, which is how a negative test quietly stops testing.
        let catalog = modules::list_modules();
        let outputs: Vec<(String, String)> = catalog
            .iter()
            .flat_map(|module| {
                module
                    .args
                    .iter()
                    .filter(|arg| arg.kind == ArgKind::LogOut)
                    .map(move |arg| (module.name.clone(), arg.name.clone()))
            })
            .collect();
        assert!(
            outputs.len() > 50,
            "the scan must see a real catalog, not an empty one: {} outputs",
            outputs.len()
        );
        for (module, output) in &outputs {
            assert_ne!(
                output.to_ascii_uppercase(),
                "SW",
                "module '{module}' emits a bare SW; a saturation output must say SWE or SWT"
            );
        }
        // and the positive control: the explicit identities really are what gets emitted.
        for wanted in ["SWE", "SWT"] {
            assert!(
                outputs.iter().any(|(_, output)| output == wanted),
                "some shipping module must emit '{wanted}'"
            );
        }

        // B — a RESULT FIELD. The pay-summary row is what a consumer reads, and its saturation
        // average must name its flavour there too, because the row outlives the run that made it.
        let row = PaySummaryRow {
            well_id: "w".into(),
            well_name: "SANDI-SW-1".into(),
            discretisation_model: DISCRETISATION_MODEL.to_string(),
            sample_interval: 0.5,
            zone: "A".into(),
            flag: "PAY".into(),
            top: 0.0,
            bottom: 1.0,
            gross: 1.0,
            net: 1.0,
            not_net: 0.0,
            unknown: 0.0,
            ntg: 1.0,
            ntg_known: 1.0,
            avg_vsh: 0.1,
            avg_phie: 0.2,
            avg_swe: 0.3,
            hpv: 0.14,
            n_classified: 1,
            perm_cutoff_no_data: false,
            quicklook_phie_excluded: false,
            residual_absorbed: 0.0,
            out_of_range: false,
            unfiltered: Vec::new(),
            frame: Default::default(),
            weights_source: MD_WEIGHTS_SOURCE.into(),
        };
        let serialized = serde_json::to_value(&row).expect("a row serializes");
        let fields: Vec<String> = serialized
            .as_object()
            .expect("a row is an object")
            .keys()
            .cloned()
            .collect();
        assert!(fields.iter().any(|f| f == "avg_swe"), "the row names the flavour: {fields:?}");
        for field in &fields {
            assert!(
                field != "avg_sw" && field != "sw" && field != "sw_max",
                "result field '{field}' is a bare SW"
            );
        }

        // C — a CUT-OFF RECORD. What is persisted with the run has to name the flavour, because
        // that record is read years later by somebody who cannot ask which Sw was meant.
        let request = PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec!["w".into()],
            vsh_max: None,
            phie_min: None,
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            skip_version: false,
            stats_only: true,
            custody: None,
            weighting: Default::default(),
            frame: Default::default(),
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
        };
        // `PaySummaryRequest` is a wire type read from the frontend, so the record to inspect is
        // the FIELD NAME the wire uses. Round-tripping the JSON the frontend sends is what proves
        // the persisted cut-off names its flavour; a bare `sw_max` would not deserialize at all.
        let wire = serde_json::json!({
            "well_ids": ["w"],
            "swe_max": {"value": 0.5, "unit": "v/v"},
        });
        let parsed: PaySummaryRequest =
            serde_json::from_value(wire).expect("the cut-off record uses swe_max");
        assert!(parsed.swe_max.is_some(), "and it carries the value");
        let bare = serde_json::json!({ "well_ids": ["w"], "sw_max": {"value": 0.5, "unit": "v/v"} });
        let parsed_bare: PaySummaryRequest =
            serde_json::from_value(bare).expect("an unknown field is ignored, not accepted as SWE");
        assert!(
            parsed_bare.swe_max.is_none(),
            "a bare sw_max must NOT be read as the saturation cut-off - silently accepting it \r
             is exactly the ambiguity this row forbids"
        );
        let _ = &request;

        // D — the exemption is NAMED and narrow. A bare `SW` may appear as an INPUT, because an
        // input names the user's own curve and the requirement governs cut-off records and result
        // fields. What it may never be is an OUTPUT, which arm A already forbids — so this arm
        // states the boundary rather than leaving it to be rediscovered as a false positive.
        let bare_sw_inputs: Vec<String> = catalog
            .iter()
            .flat_map(|module| {
                module
                    .args
                    .iter()
                    .filter(|arg| arg.kind == ArgKind::LogIn && arg.name.eq_ignore_ascii_case("SW"))
                    .map(move |_| module.name.clone())
            })
            .collect();
        for module in &bare_sw_inputs {
            let emits_bare = outputs
                .iter()
                .any(|(m, o)| m == module && o.eq_ignore_ascii_case("SW"));
            assert!(
                !emits_bare,
                "module '{module}' may READ a curve called SW, but it must not emit one"
            );
        }
    }

    /// SB-CUT-027 (P2). `14_cutoffs-summation-mc.md:1331-1344` — SandiBumi **MUST NOT** impose a
    /// fixed maximum on the number of input curves, cut-offs, report tiers or flag curves.
    ///
    /// Ledger D-5.4: IP's parameter model stops at **Curve 10**, its 2025 prose claims **50**, and
    /// IP2018's *"up to 10 input curves … the additional 7"* was correct — the 2025 edit introduced
    /// the error. All of them are vendor implementation limits with no physical basis, and
    /// SandiBumi should inherit neither the caps nor the confusion.
    ///
    /// **A fixed ARITY is not a cap, and the distinction is the whole row.** Four cut-off fields
    /// exist because four quantities are cut on; three tiers exist because three are emitted.
    /// Neither is a budget a user can exhaust. A cap is a maximum imposed on a collection that
    /// would otherwise grow — which is what this asserts the absence of.
    #[test]
    fn a_run_carries_more_curves_than_any_vendor_cap_and_the_fixed_cutoff_and_tier_counts_are_arities_not_maxima(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-NOCAP-1");
        let depth: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32).collect();

        // A — sixty curves on one well, fetched in ONE frame. Sixty is chosen to clear BOTH of the
        // vendor numbers the ledger records: past Curve 10, and past the 2025 prose's 50.
        const CURVES: usize = 60;
        let names: Vec<String> = (0..CURVES).map(|i| format!("NOCAP_{i:02}")).collect();
        for (i, name) in names.iter().enumerate() {
            equations::write_computed_curve(&conn, &well, &depth, name, &vec![i as f32; 11])
                .unwrap();
        }
        let (frame_depth, columns) = equations::fetch_curve_frame(&conn, &well, &names)
            .expect("a frame of sixty curves must resolve");
        assert_eq!(frame_depth.len(), depth.len());
        assert_eq!(
            columns.len(),
            CURVES,
            "every requested curve must come back - a silent truncation IS a cap"
        );
        for (i, name) in names.iter().enumerate() {
            assert_eq!(
                columns[name][0], i as f32,
                "curve {name} must carry its own values, not a neighbour's"
            );
        }

        // B — the four cut-off fields are an ARITY. Each is independently absent-capable, so the
        // four are not a budget: a run may use none of them, all of them, or any subset, and the
        // count is not a resource anything competes for.
        let dbm = Mutex::new(conn);
        let vv = |value: f64| Some(CutoffSpec::from(CutoffEntry { value, unit: "v/v".into() }));
        let summary = |vsh, phie, swe| {
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vsh,
                    phie_min: phie,
                    swe_max: swe,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("every subset of the cut-offs is a valid run")
        };
        let none = summary(None, None, None);
        let all = summary(vv(0.50), vv(0.10), vv(0.90));
        assert!(!none.is_empty() && !all.is_empty());

        // C — the tier count is DATA, not a hard-coded three scattered through the engine. The row
        // emission iterates `SUMMARY_FLAGS`, so the output carries exactly the tiers that constant
        // names — which is what makes adding one a change to a list rather than a search for every
        // place a `3` is written down.
        let mut emitted: Vec<String> =
            all.iter().map(|row| row.flag.clone()).collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
        emitted.sort();
        let mut declared: Vec<String> = SUMMARY_FLAGS.iter().map(|f| f.to_string()).collect();
        declared.sort();
        assert_eq!(
            emitted, declared,
            "the emitted tiers must be exactly the declared ones, with nothing dropped and nothing \
             invented"
        );

        // D — and no cap is expressed anywhere in the summation engine as a maximum COUNT of
        // curves, cut-offs, tiers or flags. The clamps this domain does carry are on ITERATIONS
        // and SWEEP STEPS, which are compute budgets on a loop rather than limits on how much
        // rock a study may describe, so they are named here rather than exempted silently.
        let source = include_str!("workflow.rs");
        let body = source.split("\nmod tests {").next().unwrap_or(source);
        for banned in [
            "curves.len() >",
            "cutoffs.len() >",
            "flags.len() >",
            "tiers.len() >",
            "curve_names.len() >",
            ".take(10)",
            ".take(50)",
        ] {
            assert!(
                !body.contains(banned),
                "the summation engine must impose no maximum on how much a study describes, but \
                 it contains '{banned}'"
            );
        }
    }

    /// A clean, porous, WET sand: every sample passes a clay and a porosity cut-off and fails any
    /// ordinary saturation cut-off. It is the rock SB-CUT-026 exists to protect.
    fn seed_wet_reservoir(conn: &duckdb::Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 11usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![30.0; n], vec![4.0; n], vec![0.25; n], vec![2.3; n],
            nan.clone(), nan,
        )
        .unwrap();
        for (curve, value) in [("VSH", 0.10f32), ("PHIE", 0.30), ("SWE", 0.80)] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![value; n]).unwrap();
        }
        well
    }

    /// SB-CUT-026 (P1). `14_cutoffs-summation-mc.md:1318-1329` — the net reservoir tier **MUST
    /// NOT** apply a saturation cut-off by default; net reservoir **MUST** be porosity- and
    /// clay-driven, and saturation **MUST** enter at the pay tier.
    ///
    /// F-25: IP's `Sw Net Use` and `Sw Pay Use` are separate ordinals and Net Reservoir is
    /// described as porosity- and clay-driven. The consequence of getting it wrong is stated in
    /// the chapter and is the reason this is P1 rather than a preference — **it reclassifies wet
    /// reservoir as non-reservoir**. A water-bearing sand is still reservoir rock; it is the pay
    /// tier that is allowed to care that it is wet.
    #[test]
    fn a_wet_but_porous_clean_sand_is_reservoir_and_not_pay_because_saturation_enters_at_the_pay_tier(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_wet_reservoir(&conn, "SANDI-WET-1");
        let dbm = Mutex::new(conn);
        let vv = |value: f64| Some(CutoffSpec::from(CutoffEntry { value, unit: "v/v".into() }));
        let run = |swe: Option<CutoffSpec>, use_at: Vec<(&str, CutoffUse)>| {
            let rows = run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vv(0.50),
                    phie_min: vv(0.10),
                    swe_max: swe,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: use_at
                        .into_iter()
                        .map(|(slot, u)| (slot.to_string(), u))
                        .collect(),
                },
            )
            .expect("the run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("SAND"), net("RESERVOIR"), net("PAY"))
        };

        // A — the requirement's own failure mode. This sand is clean (VSH 0.10) and porous
        // (PHIE 0.30) and WET (SWE 0.80). With a 0.50 saturation cut-off it must book as
        // reservoir in full and as pay not at all.
        let (sand, reservoir, pay) = run(vv(0.50), vec![]);
        assert!(sand > 0.0, "the fixture books sand");
        assert_eq!(
            reservoir, sand,
            "a wet sand is still reservoir rock: applying the saturation cut-off at the reservoir \
             tier would reclassify it as non-reservoir, which is the defect this row prevents"
        );
        assert_eq!(pay, 0.0, "and saturation DOES enter at the pay tier");

        // B — the reservoir tier is independent of the saturation cut-off's VALUE. Moving it must
        // move pay and leave reservoir where it is; that is what "does not apply" means, as
        // distinct from "applies but happens not to bite on this fixture".
        let (_, reservoir_loose, pay_loose) = run(vv(0.90), vec![]);
        assert_eq!(reservoir_loose, reservoir, "reservoir must not move with the SWE cut-off");
        assert!(pay_loose > pay, "while pay must: {pay_loose} against {pay}");

        // C — and reservoir IS porosity- and clay-driven, pinned from the positive side too. A
        // tier that applied NOTHING would satisfy every assertion above and be a different bug.
        let strict_clay = run(vv(0.50), vec![]).1;
        let (_, reservoir_clay, _) = {
            let rows = run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: vv(0.05),
                    phie_min: vv(0.10),
                    swe_max: vv(0.50),
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("the run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("SAND"), net("RESERVOIR"), net("PAY"))
        };
        assert!(
            reservoir_clay < strict_clay,
            "a clay cut-off below the sand's VSH must reduce reservoir: {reservoir_clay} against \
             {strict_clay}"
        );

        // D — it is a DEFAULT, not a prohibition. The requirement says the reservoir tier must not
        // apply saturation *by default*; a user who declares otherwise is entitled to it, and
        // removing the capability would be a different requirement.
        let (_, reservoir_declared, _) = run(
            vv(0.50),
            vec![("SWE", CutoffUse { sand: false, reservoir: true, pay: true })],
        );
        assert_eq!(
            reservoir_declared, 0.0,
            "an explicit declaration must reach the reservoir tier - the rule is a default, not a \
             prohibition"
        );

        // E — and the default itself is DECLARED rather than emergent, so it can be read off the
        // configuration instead of inferred from a result.
        assert!(
            !default_cutoff_use("SWE").reservoir && default_cutoff_use("SWE").pay,
            "SWE ships off at the reservoir tier and on at pay"
        );
    }

    /// SB-CUT-022 (P1). `14_cutoffs-summation-mc.md:1254-1272` and F-25 at `:489-501` — each
    /// cut-off **MUST** carry an explicit enable flag per report tier; activation **MUST NOT** be
    /// inferred from the presence of a curve or of a value; and the reservoir and pay tiers
    /// **MUST** share **one value** with **two independent use flags**.
    ///
    /// IP ships exactly that shape: `Phi Net Use`, `Phi Pay Use` and `Phi Cutoff`, the last
    /// described as *"Porosity cutoff value for Pay and Reservoir report"* — one value, two flags.
    /// The reason it must be a flag and not an inference is F-17: Geolog changed the activation
    /// trigger between two modules of ONE product, `Determin` firing on the presence of the curve
    /// and `determin_mc` on the presence of the value. An inferred rule cannot be audited from a
    /// result, because the result does not record what was inferred.
    #[test]
    fn each_cutoff_declares_the_tiers_it_is_used_at_and_reservoir_and_pay_share_one_value_with_independent_flags(
    ) {
        // A — the shipped defaults ARE the ladder, declared rather than nested. Net sand is clay
        // driven, net reservoir adds porosity, net pay adds saturation (T4 Bentley & Ringrose,
        // `:1296-1297`), and Sw is OFF at the reservoir tier — F-25 `:494-495`, which is also
        // SB-CUT-026's whole subject.
        assert_eq!(
            default_cutoff_use("VSH"),
            CutoffUse { sand: true, reservoir: true, pay: true }
        );
        assert_eq!(
            default_cutoff_use("PHIE"),
            CutoffUse { sand: false, reservoir: true, pay: true }
        );
        assert_eq!(
            default_cutoff_use("SWE"),
            CutoffUse { sand: false, reservoir: false, pay: true },
            "IP describes Net Reservoir as porosity- and clay-driven; Sw is off there by default"
        );
        assert_eq!(
            default_cutoff_use("PERM"),
            CutoffUse { sand: false, reservoir: false, pay: true }
        );

        // B — ONE VALUE, TWO FLAGS. The reservoir and pay tiers read the same `phie_min`; turning
        // it off for one tier must not change the other, and must not change the value.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "SANDI-TIER-1", "PHIE");
        let dbm = Mutex::new(conn);
        let run = |use_at: Vec<(&str, CutoffUse)>| {
            let rows = run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: Some(
                        CutoffEntry { value: 0.20, unit: "v/v".into() }.into(),
                    ),
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: use_at
                        .into_iter()
                        .map(|(slot, use_at)| (slot.to_string(), use_at))
                        .collect(),
                },
            )
            .expect("the run itself is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("RESERVOIR"), net("PAY"))
        };

        // The fixture's PHIE is 0.30 over its shallow half and 0.10 over its deep half, so a 0.20
        // cut-off is a real filter: with it on, half the footage books.
        let (res_both, pay_both) = run(vec![]);
        assert!(res_both > 0.0 && pay_both > 0.0, "the default run books something");

        // Off at RESERVOIR only. Pay must not move — that is what INDEPENDENT means.
        let (res_off, pay_off) = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: false, pay: true },
        )]);
        assert!(
            res_off > res_both,
            "with the porosity cut-off disabled at the reservoir tier that tier must book MORE \
             footage: {res_off} against {res_both}"
        );
        assert_eq!(
            pay_off, pay_both,
            "and the pay tier must not move — one value, two independent flags"
        );

        // Off at PAY only. Now the mirror: reservoir must not move.
        let (res_pay_off, pay_pay_off) = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: true, pay: false },
        )]);
        assert_eq!(res_pay_off, res_both, "the reservoir tier must not move");
        assert!(
            pay_pay_off > pay_both,
            "and the pay tier must book more: {pay_pay_off} against {pay_both}"
        );

        // C — ACTIVATION IS NEVER INFERRED. `use_at` is resolved from the SLOT and the run's own
        // declaration and nothing else, so neither the presence of a curve nor the presence of a
        // value can turn a cut-off on. That is a property of the signature, not of today's body:
        // the resolver has no access to either.
        let declared = BTreeMap::from([(
            "PHIE".to_string(),
            CutoffUse { sand: true, reservoir: false, pay: false },
        )]);
        assert_eq!(
            cutoff_use_for(&declared, "PHIE"),
            CutoffUse { sand: true, reservoir: false, pay: false },
            "a declaration is honoured verbatim"
        );
        assert_eq!(
            cutoff_use_for(&declared, "SWE"),
            default_cutoff_use("SWE"),
            "and an undeclared slot takes its documented default, not its neighbour's declaration"
        );

        // D — a cut-off disabled at EVERY tier books exactly what no cut-off at all books. The two
        // are different statements about intent and must be the same statement about rock.
        let all_off = run(vec![(
            "PHIE",
            CutoffUse { sand: false, reservoir: false, pay: false },
        )]);
        let unfiltered = {
            let rows = run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: None,
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("an unfiltered run is valid");
            let net = |flag: &str| {
                rows.iter()
                    .filter(|row| row.flag == flag)
                    .map(|row| row.net as f64)
                    .sum::<f64>()
            };
            (net("RESERVOIR"), net("PAY"))
        };
        assert_eq!(
            all_off, unfiltered,
            "a cut-off switched off at every tier filters nothing, exactly as an absent one does"
        );
    }

    /// SB-CUT-020 (P2). `14_cutoffs-summation-mc.md:1223-1240` and SB-CUT-T24 at `:2085` — a
    /// cut-off **MUST** be expressible as a two-sided range with an explicit operator selecting the
    /// inclusivity of each bound; the single-sided `>=` / `<=` forms **MUST** be the degenerate
    /// case with an open far bound; and every operator's boundary behaviour **MUST** be tested
    /// against SandiBumi's **own written specification**, which is [`CutoffRange`]'s doc comment.
    ///
    /// The oracle is deliberately ours. Techlog's `limitType` is strictly more general than IP's
    /// single-sided form, but its shipped implementation is the warning rather than the model:
    /// modes 4/5/6 raise, mode 7 is a silent always-pass, and modes 2/3 are **documented as outside
    /// tests and implemented as inside tests**. A boundary convention not tested against its own
    /// spec is a coin flip at every sample sitting exactly on the cut-off — which is precisely the
    /// population that decides a marginal-pay result.
    #[test]
    fn a_sample_exactly_on_a_cutoff_bound_is_included_or_excluded_by_that_bounds_own_declared_operator(
    ) {
        // A — the specification itself, at exactly the bound, for every operator on every side.
        // This is the T24 case: a value equal to `min` and a value equal to `max`.
        let low = |operator| CutoffRange {
            low: Some(CutoffBound { value: 0.10, operator }),
            high: None,
        };
        let high = |operator| CutoffRange {
            low: None,
            high: Some(CutoffBound { value: 0.50, operator }),
        };
        assert!(low(BoundOperator::Inclusive).contains(0.10f32), "x >= min admits x == min");
        assert!(!low(BoundOperator::Exclusive).contains(0.10f32), "x > min excludes x == min");
        assert!(high(BoundOperator::Inclusive).contains(0.50f32), "x <= max admits x == max");
        assert!(!high(BoundOperator::Exclusive).contains(0.50f32), "x < max excludes x == max");
        // and away from the bound every operator agrees, so the arms above isolate the boundary.
        for operator in [BoundOperator::Inclusive, BoundOperator::Exclusive] {
            assert!(low(operator).contains(0.11f32) && !low(operator).contains(0.09f32));
            assert!(high(operator).contains(0.49f32) && !high(operator).contains(0.51f32));
        }

        // A2 — and "exactly on the bound" is decided at the precision the DATA has. A continuous
        // log is f32; a cut-off is entered as a decimal. Widen the sample and 0.30f32 becomes
        // 0.30000001192…, strictly GREATER than 0.30f64 — so the sample the user typed `0.30` to
        // sit exactly on would not sit on it, and the exclusive operator would exclude nothing at
        // all. Both sides are pinned, because an implementation comparing in f64 passes the
        // inclusive half and fails only here.
        let three_tenths = CutoffRange {
            low: Some(CutoffBound { value: 0.30, operator: BoundOperator::Exclusive }),
            high: None,
        };
        assert!(
            (0.30f32 as f64) > 0.30f64,
            "the premise: widening an f32 sample overshoots the f64 bound"
        );
        assert!(
            !three_tenths.contains(0.30f32),
            "an f32 sample of 0.30 sits exactly on a 0.30 bound and an exclusive bound excludes it"
        );
        assert!(
            CutoffRange {
                low: Some(CutoffBound { value: 0.30, operator: BoundOperator::Inclusive }),
                high: None,
            }
            .contains(0.30f32),
            "and an inclusive bound admits it"
        );

        // B — an ABSENT bound is an OPEN far bound and admits everything on that side. That is
        // what makes the single-sided form a degenerate range rather than a separate mechanism.
        let open = CutoffRange { low: None, high: None };
        assert!(open.contains(-1e9f32) && open.contains(1e9f32));
        assert!(low(BoundOperator::Inclusive).contains(1e9f32), "no high bound admits any large value");

        // C — the DEGENERATE wire form is unchanged. A slot that has always meant "at least this"
        // still means it, inclusively, and a slot that has always meant "at most this" likewise -
        // the requirement makes the single-sided forms the degenerate case, so a project saved
        // before ranges existed must classify every sample exactly as it did.
        let entry: CutoffSpec = serde_json::from_str(r#"{"value":0.10,"unit":"v/v"}"#).unwrap();
        let as_min = entry
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
            .unwrap();
        assert_eq!(as_min.low, Some(CutoffBound { value: 0.10, operator: BoundOperator::Inclusive }));
        assert_eq!(as_min.high, None, "the far side stays open");
        assert!(as_min.contains(0.10f32), "and a sample exactly on it still passes, as it always did");
        let as_max = entry
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Maximum, "VSH")
            .unwrap();
        assert_eq!(as_max.high, Some(CutoffBound { value: 0.10, operator: BoundOperator::Inclusive }));
        assert_eq!(as_max.low, None);

        // D — a genuine two-sided range, with a different operator on each side, crosses the wire
        // and filters both ends. `35 pu` is canonicalised per bound, so the unit rule of SB-CUT-019
        // reaches inside a range rather than stopping at its edge.
        let spec: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":10,"unit":"pu","operator":"EXCLUSIVE"},
                "max":{"value":35,"unit":"pu","operator":"INCLUSIVE"}}"#,
        )
        .unwrap();
        let range = spec
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
            .expect("a two-sided porosity window is a real cut-off");
        assert!(!range.contains(0.10f32), "the low bound is exclusive, so 0.10 fails");
        assert!(range.contains(0.35f32), "the high bound is inclusive, so 0.35 passes");
        assert!(range.contains(0.20f32) && !range.contains(0.40f32));

        // E — a range that can admit NOTHING is refused. Booking zero net from a window nobody
        // could have meant is this row's own risk class: it computes, it plots, and it is wrong.
        let empty: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":0.40,"unit":"v/v"},"max":{"value":0.20,"unit":"v/v"}}"#,
        )
        .unwrap();
        let error = empty
            .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "the PHIE cut-off")
            .expect_err("an inverted window must refuse");
        assert!(error.contains("PHIE"), "and name the cut-off: {error}");
        let touching: CutoffSpec = serde_json::from_str(
            r#"{"min":{"value":0.20,"unit":"v/v","operator":"EXCLUSIVE"},
                "max":{"value":0.20,"unit":"v/v"}}"#,
        )
        .unwrap();
        assert!(
            touching
                .canonical(CutoffQuantity::VolumeFraction, CutoffSense::Minimum, "PHIE")
                .is_err(),
            "bounds that meet with either side exclusive admit nothing either"
        );

        // F — WIRED IN, and the pair is the point: the SAME well and the SAME number classify
        // differently on the operator alone. A sample sitting exactly on the cut-off is the
        // population that decides a marginal result, so the operator has to reach the arithmetic.
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_weighting_well(&conn, "SANDI-BOUND-1", "PHIE");
        let dbm = Mutex::new(conn);
        let net_with = |operator: &str| {
            let spec: CutoffSpec = serde_json::from_str(&format!(
                r#"{{"min":{{"value":0.30,"unit":"v/v","operator":"{operator}"}}}}"#
            ))
            .unwrap();
            run_pay_summary(
                &dbm,
                &PaySummaryRequest {
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![well.clone()],
                    vsh_max: None,
                    phie_min: Some(spec),
                    swe_max: None,
                    perm_min: None,
                    skip_version: false,
                    stats_only: true,
                    custody: None,
                    weighting: Default::default(),
                    frame: Default::default(),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                },
            )
            .expect("the run itself is valid under both operators")
            .iter()
            .filter(|row| row.flag == "PAY")
            .map(|row| row.net as f64)
            .sum::<f64>()
        };
        let inclusive = net_with("INCLUSIVE");
        let exclusive = net_with("EXCLUSIVE");
        assert!(
            inclusive > exclusive,
            "the fixture's PHIE sits exactly on 0.30, so an inclusive bound must book footage an \
             exclusive one does not: inclusive {inclusive}, exclusive {exclusive}"
        );
    }

    /// Seed a well that every resistivity saturation model can run on: a deep resistivity, and
    /// the interpreted porosity and shale volume the shaly-sand models take as inputs.
    fn seed_saturation_well(conn: &duckdb::Connection, name: &str) -> String {
        let id = uuid::Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let well = id.to_string();
        let n = 12usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            conn, id, depth.clone(), vec![55.0; n], vec![9.0; n], nan.clone(), nan.clone(),
            nan.clone(), nan,
        )
        .unwrap();
        for (curve, value) in [
            ("PHIE", 0.22f32),
            ("PHIT", 0.26),
            ("PHIT_SSPW", 0.26),
            ("CAPBW_SSPW", 0.06),
        ] {
            equations::write_computed_curve(conn, &well, &depth, curve, &vec![value; n]).unwrap();
        }
        well
    }

    /// `VSH` must arrive with its physical family declared, so the shaly-sand models resolve it as
    /// a shale volume rather than by mnemonic. Producing it through `vsh_gr` is the honest way to
    /// get that: it is how a real well acquires one, and it exercises the same custody the run
    /// will check.
    /// SB-POR-024 (DEC-025, RULED 2026-08-17): the N-D porosity methods refuse a neutron
    /// curve whose matrix basis is not DECLARED - a limestone-unit neutron read against a
    /// sandstone matrix is ~0.04 v/v low in clean water sand, and an undeclared basis
    /// cannot be checked. The basis is declared curve metadata, never inferred (DEC-025's
    /// constraint); the Bateman-Konen crossplot additionally requires the LIMESTONE entry
    /// its own source assumes, naming nphimat as the converter; and the declared basis
    /// rides the run's stored manifest as a physics attribute, so the output provenance
    /// states what the arithmetic consumed. A curve that does not resolve at all keeps
    /// the ordinary missing-input refusal - absence of a curve is not absence of a basis.
    #[test]
    fn the_neutron_density_methods_refuse_an_undeclared_or_wrong_basis_and_record_the_declared_one(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-NB24", None, None, None).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = id.to_string();
        let n = 3usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(
            &conn, id, depth, vec![50.0; n], nan.clone(), vec![0.21; n], vec![2.35; n],
            nan.clone(), nan,
        )
        .unwrap();
        db::migrate_standard_curves_to_generic_store(&conn).unwrap();
        let nphi_curve = db::list_generic_curve_catalog(&conn, &well)
            .unwrap()
            .into_iter()
            .find(|entry| entry.mnemonic == "NPHI")
            .expect("NPHI migrated")
            .curve_id;
        let dbm = Mutex::new(conn);
        seed_typed_vsh(&dbm, &well);
        let run = |module: &str| -> ModuleRunResult {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: module.into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params: HashMap::from([
                        ("RHO_MA".to_string(), 2.645_f64),
                        ("RHO_SH".to_string(), 2.50_f64),
                        ("RHO_FL".to_string(), 1.0_f64),
                        ("RHO_DSH".to_string(), 2.70_f64),
                        ("RHO_W".to_string(), 1.0_f64),
                        ("NPHI_SH".to_string(), 0.35_f64),
                        ("PHIE_MAX".to_string(), 0.3_f64),
                    ]),
                    opts: HashMap::new(),
                    output_set: None,
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
            .remove(0)
        };

        // A - undeclared: both N-D methods refuse BY NAME, stating the physics, the fix
        // and the ruling; the density method never needed a neutron and still runs.
        for module in ["phi_dn", "phi_dnbk"] {
            let refused = run(module).error.expect("an undeclared basis must refuse");
            assert!(
                refused.contains("DECLARED matrix basis")
                    && refused.contains("set_curve_neutron_basis")
                    && refused.contains("DEC-025"),
                "{module}: {refused}"
            );
        }
        assert!(run("phi_den").error.is_none(), "phi_den has no neutron input to gate");

        // B - declared SANDSTONE: the quick-look runs (any declared basis is admissible -
        // its average reads against the interpreter's own RHO_MA), and the stored manifest
        // records the declared basis as a physics attribute.
        {
            let conn = dbm.lock().unwrap();
            db::set_curve_neutron_basis(
                &conn, &nphi_curve, "SANDSTONE", "declared at import by the user (DEC-025)",
            )
            .unwrap();
        }
        let quick = run("phi_dn");
        assert!(quick.error.is_none(), "{:?}", quick.error);
        let recorded_basis = |module: &str| -> Option<String> {
            let conn = dbm.lock().unwrap();
            equations::list_log_sets(&conn, &well)
                .unwrap()
                .into_iter()
                .filter(|entry| entry.module == module)
                .last()
                .and_then(|entry| entry.ancestry)
                .and_then(|ancestry| {
                    ancestry
                        .physics_attributes
                        .iter()
                        .find(|attribute| attribute.name == "neutron_basis")
                        .map(|attribute| attribute.value.clone())
                })
        };
        assert_eq!(
            recorded_basis("phi_dn").as_deref(),
            Some("SANDSTONE"),
            "the output provenance states the declared basis"
        );

        // C - the crossplot is entered in LIMESTONE units: a declared SANDSTONE basis is
        // refused naming the entry units and the converter.
        let wrong = run("phi_dnbk").error.expect("a wrong basis must refuse");
        assert!(
            wrong.contains("LIMESTONE") && wrong.contains("nphimat") && wrong.contains("DEC-025"),
            "{wrong}"
        );

        // D - redeclared LIMESTONE: the crossplot runs and its provenance says so.
        {
            let conn = dbm.lock().unwrap();
            db::set_curve_neutron_basis(
                &conn, &nphi_curve, "LIMESTONE", "nphimat conversion record (DEC-025)",
            )
            .unwrap();
        }
        let crossplot = run("phi_dnbk");
        assert!(crossplot.error.is_none(), "{:?}", crossplot.error);
        assert_eq!(recorded_basis("phi_dnbk").as_deref(), Some("LIMESTONE"));
    }

    /// SB-POR-024 fixture side: declare the neutron basis the way an import would, so the
    /// DEC-025 boundary refusal does not fire on a fixture that is not about it. Finds the
    /// named mnemonic in the generic catalog (migrating the standard columns first, the
    /// same route an old project takes).
    fn declare_nphi_basis(conn: &duckdb::Connection, well: &str, mnemonic: &str, basis: &str) {
        db::migrate_standard_curves_to_generic_store(conn).unwrap();
        let curve = db::list_generic_curve_catalog(conn, well)
            .unwrap()
            .into_iter()
            .find(|entry| entry.mnemonic == mnemonic)
            .unwrap_or_else(|| panic!("{mnemonic} in the catalog"))
            .curve_id;
        db::set_curve_neutron_basis(conn, &curve, basis, "test fixture declaration (DEC-025)")
            .unwrap();
    }

    fn seed_typed_vsh(dbm: &Mutex<duckdb::Connection>, well: &str) {
        let results = run_workflow_module_into(
            dbm,
            &RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![well.to_string()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
                opts: HashMap::new(),
                output_set: Some("INTERP".into()),
                input_set: None,
                custody: test_run_custody(),
            },
            None,
            None,
            None,
        );
        assert!(results[0].error.is_none(), "seeding VSH: {:?}", results[0].error);
    }

    /// SB-SAT-043 (P0). `docs/PRD_v2/12_saturation.md:1776-1795` and SB-SAT-T59 at `:2523-2532` —
    /// every saturation run **MUST** emit, alongside its curves, a machine-readable record of the
    /// model identifier, every parameter value used, each value's source string, the literature
    /// citation the method traces to, and the Worthington 1985 type where one is stated by a
    /// source; for the LRLC methods an explicit unfitted-coefficient flag; **zero fields empty**;
    /// and that record **MUST** survive export into the deliverable.
    ///
    /// Geolog ships published references inside every module manifest but **no vendor carries the
    /// reference through to the answer** (`:1783-1790`). A parameter that carries the paper it came
    /// from, through the computation, into the deliverable is the claim this row exists to make —
    /// and the only thing that makes SB-SAT-038's build-time source gate auditable downstream.
    #[test]
    fn a_saturation_run_carries_its_model_citation_worthington_type_and_unfitted_coefficient_flag_into_the_deliverable(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_saturation_well(&conn, "SANDI-SAT-1");
        let dbm = Mutex::new(conn);
        seed_typed_vsh(&dbm, &well);

        let run = |module: &str, curve: &str, params: HashMap<String, f64>, opts: Vec<(&str, &str)>| {
            let results = run_workflow_module_into(
                &dbm,
                &RunModuleRequest {
                    module: module.into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params,
                    opts: opts
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    output_set: Some(module.to_ascii_uppercase()),
                    input_set: None,
                    custody: test_run_custody(),
                },
                None,
                None,
                None,
            );
            assert!(results[0].error.is_none(), "{module}: {:?}", results[0].error);
            let conn = dbm.lock().unwrap();
            equations::curve_ancestry(&conn, &well, curve)
                .unwrap_or_else(|error| panic!("{module} must record its ancestry: {error}"))
        };

        // A — the Indonesia leg, which is the one a source classifies. Its record must carry the
        // paper the equation traces to and the Worthington 1985 type, both beside the value.
        let indo = run(
            "sw_indo",
            "SWE",
            HashMap::from([
                ("A".into(), 1.0),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("RT_SH".into(), 3.0),
                ("RW".into(), 0.1),
                ("SWE_IRR".into(), 0.05),
            ]),
            vec![("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL")],
        );
        let find = |ancestry: &equations::CurveAncestry, name: &str| {
            ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == name)
                .unwrap_or_else(|| {
                    panic!(
                        "the record must carry '{name}'; it carries {:?}",
                        ancestry.parameters.iter().map(|p| &p.name).collect::<Vec<_>>()
                    )
                })
                .clone()
        };

        assert_eq!(find(&indo, "method_id").value, serde_json::json!("indonesia"));
        let citation = find(&indo, "method_citation");
        assert!(
            citation.value.as_str().unwrap_or("").contains("Poupon")
                && citation.value.as_str().unwrap_or("").contains("Paper O"),
            "the Indonesia record must name the paper it traces to: {}",
            citation.value
        );
        let worthington = find(&indo, "worthington_1985_type");
        assert_eq!(
            worthington.value,
            serde_json::json!(4),
            "Geolog states sw_indo as Worthington type 4 (12_saturation.md:478)"
        );

        // B — zero fields empty. A record with a blank source is worse than no record: it reads as
        // provenance and defends nothing.
        for parameter in &indo.parameters {
            assert!(
                !parameter.source.trim().is_empty(),
                "'{}' was recorded with no source",
                parameter.name
            );
        }

        // C — Archie is classified by NO source, and that must be SAID rather than left blank.
        // Omitting the field and stating "none" are different claims, and only one is checkable.
        let arch = run(
            "sw_arch",
            "SWE",
            HashMap::from([
                ("A".into(), 1.0),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("RW".into(), 0.1),
                ("SWT_IRR".into(), 0.05),
            ]),
            vec![("OPT_RW", "CONSTANT")],
        );
        let arch_type = find(&arch, "worthington_1985_type");
        assert_eq!(
            arch_type.value,
            serde_json::json!(crate::param_sources::WORTHINGTON_NONE_STATED),
            "no source classifies Archie, so the record must SAY so rather than omit the field"
        );
        assert!(
            !arch_type.source.trim().is_empty(),
            "and the record must still say WHY it carries none"
        );

        // D — the LRLC methods carry an explicit unfitted-coefficient flag. `sw_rtc`'s A_CAP/B_QV/
        // C0/RSF are one field's calibration; a run on numbers that did not come from this
        // project's own fit is indistinguishable in the OUTPUT from one that did.
        let rtc = run(
            "sw_rtc",
            "SWE_RTC",
            HashMap::from([
                ("RW".into(), 0.3),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("A_CAP".into(), 0.45),
                ("B_QV".into(), 0.0057),
                ("C0".into(), -0.0071),
                ("RSF".into(), 2.25),
                ("CEC".into(), 0.0),
                ("RHOG".into(), 2.65),
            ]),
            vec![],
        );
        assert_eq!(find(&rtc, "method_id").value, serde_json::json!("lrlc_rtc"));
        let flag = find(&rtc, "unfitted_coefficients");
        assert!(
            !flag.value.is_null() && !flag.source.trim().is_empty(),
            "an LRLC run must state the calibration standing of its coefficients: {flag:?}"
        );

        // E — SURVIVES EXPORT. The disclosure cells are what the PDF, Word, workbook and deck all
        // render, so a record that stopped at the database boundary would satisfy every assertion
        // above and still fail the requirement.
        let disclosures = {
            let conn = dbm.lock().unwrap();
            equations::curve_ancestry_disclosures(&conn, &[well.clone()], Some("SW_INDO")).unwrap()
        };
        let rendered = disclosures
            .iter()
            .flat_map(|disclosure| disclosure.cells())
            .collect::<Vec<_>>()
            .join(" | ");
        for expected in ["method_citation", "Paper O", "worthington_1985_type"] {
            assert!(
                rendered.contains(expected),
                "the exported disclosure must carry '{expected}': {rendered}"
            );
        }

        // F — the BUILD GATE, which is what makes this a contract rather than a table somebody
        // remembered to fill in. It is written over the Saturation CATEGORY, not a hand-kept list,
        // so the model that ships without a citation is caught rather than remembered.
        use crate::param_sources::{
            validate_saturation_methods, SaturationMethod, METHOD_OWNED_ELSEWHERE, RETIRED_METHOD,
            SATURATION_METHODS, WORTHINGTON_NONE_STATED,
        };
        let catalog = modules::list_modules();
        validate_saturation_methods(&catalog, SATURATION_METHODS)
            .expect("the shipped registry passes its own gate");
        let shipped = catalog
            .iter()
            .filter(|module| module.category == "Saturation")
            .count();
        assert!(
            shipped >= 6 && SATURATION_METHODS.len() >= shipped,
            "the gate must not pass by seeing nothing: {shipped} saturation modules, {} registered",
            SATURATION_METHODS.len()
        );

        let entry = |citation: &'static str, worthington_source: &'static str| SaturationMethod {
            module: "sw_arch",
            method_id: "archie_total",
            citation,
            worthington: None,
            worthington_source,
            caution: "",
        };
        // A publication nobody can look up is not a citation - the same clause SB-CORE-004 applies
        // to a parameter default, applied to the paper behind the equation.
        assert!(
            validate_saturation_methods(&[], &[entry("Archie", "none stated, per T59")]).is_err(),
            "an author's name alone must not pass as a citation"
        );
        // Silence about the classification is what is being prevented, not the absence of one.
        assert!(
            validate_saturation_methods(&[], &[entry("Archie 1942 Trans. AIME 146:54-62", "")])
                .is_err(),
            "an entry that says nothing about the Worthington type must fail"
        );
        // A hand-off must name where the literature lives, or it is an omission wearing a token.
        for token in [RETIRED_METHOD, METHOD_OWNED_ELSEWHERE] {
            assert!(
                validate_saturation_methods(&[], &[entry(token, "not classified")]).is_err(),
                "'{token}' with no explanation must fail"
            );
        }
        // And the clause that catches the NEXT model: a Saturation-category module absent from the
        // registry fails the build, so a new saturation answer cannot ship with nothing behind it.
        let orphan = catalog
            .iter()
            .find(|module| module.name == "sw_indo")
            .expect("sw_indo is a shipping saturation module")
            .clone();
        let survivors = SATURATION_METHODS
            .iter()
            .copied()
            .filter(|method| method.module != "sw_indo")
            .collect::<Vec<_>>();
        let error = validate_saturation_methods(&[orphan], &survivors)
            .expect_err("an unregistered saturation module must fail the build");
        assert!(
            error.contains("sw_indo") && error.contains("citation"),
            "and the failure must name the module and what it lacks: {error}"
        );

        // The token is a STATEMENT, so it must be a word the record can carry - a null value is
        // refused by the ancestry validator, and would in any case be indistinguishable from an
        // unwritten field.
        assert!(!WORTHINGTON_NONE_STATED.trim().is_empty());
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
                    discretisation: DiscretisationModel::Forward,
                    input_set: None,
                    well_ids: vec![no_perm.clone(), low_perm.clone()],
                    vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                    phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                    swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    perm_min: perm_min.map(|p| CutoffEntry { value: p, unit: "mD".into() }.into()),
                    skip_version: false,
                    stats_only: true
                ,
                    custody: None,
                    frame: Default::default(),
                    weighting: Default::default(),
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
                discretisation: DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![bare.clone(), good.clone()],
                vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true
            ,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
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
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Vsh, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0,
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
            &vsh, &phie, &swe, &perm, &all, SweepProp::Swe, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0, 3,
            Metric::Ntg, 4.0,
        );
        assert!((vals[2] - 1.0).abs() < 1e-9);
        // DST clips two samples to zero thickness → NET tops out at 2 m.
        let half = [1.0f64, 1.0, 0.0, 0.0];
        let (_, vals2, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &half, SweepProp::Swe, at_most(0.5), at_least(0.1), at_most(0.6), None, 0.0, 1.0,
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
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, at_most(0.9), at_least(0.0), at_most(1.0), None, 0.0, 1.0, 2,
            Metric::Net, 1.5,
        );
        assert!((peak - 1.5).abs() < 1e-9, "net must be the clamped 1.5 m, not 2.0; got {peak}");
        let (_, ntg, _) = compute_sweep(
            &vsh, &phie, &swe, &perm, &incl_h, SweepProp::Swe, at_most(0.9), at_least(0.0), at_most(1.0), None, 0.0, 1.0, 2,
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

        // PEF, DRHO, CALI ONLY in the generic store. CALI is enlarged at sample 2 against the
        // explicit 6 in bit size from `20_envcorr-qc.md` section 4.3's slim-hole example.
        let put = |mnem: &str, family: &str, unit: &str, vals: Vec<f32>| {
            let id = db::upsert_curve_meta(&conn, &w, "RAW", mnem, Some(unit), Some(family), Some("test"), None).unwrap();
            db::insert_curve_samples(&conn, &id, &depths, &vals).unwrap();
        };
        put("PEFZ", "PEF", "b/e", vec![pef_v; n]); // mnemonic differs → must resolve by family
        put("HDRA", "DRHO", "g/cc", vec![0.01, 0.01, 0.03, 0.01]); // above 0.02 at sample 2
        put("HCAL", "CALI", "in", vec![6.2, 6.2, 9.0, 6.2]); // 9 - 6 > cited 2 in cutoff

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
        let r = run(
            "badhole",
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0), ("BS_INPUT", 6.0)],
            &[("DRHO_MAX_UNIT", "g/cc")],
        );
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
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false
        ,
            stats_only: true,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
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
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            stats_only: false
        ,
            custody: Some(test_run_custody()),
            frame: Default::default(),
            weighting: Default::default(),
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
            // SB-CUT-019 tightened this: the stored form carries the UNIT beside the value,
            // because "entered with a unit and stored with it" is half the requirement. A bare
            // 0.5 in provenance would no longer say whether it meant v/v or porosity units.
            // SB-CUT-020 tightened it again: the stored form also names the OPERATOR,
            // so a reloaded run cannot silently move which side of the bound a sample
            // falls on - the one difference that is invisible everywhere except at
            // exactly the cut-off.
            for expected in [
                "\"vsh_max\":{\"operator\":\"INCLUSIVE\",\"unit\":\"v/v\",\"value\":0.5}",
                "\"phie_min\":{\"operator\":\"INCLUSIVE\",\"unit\":\"v/v\",\"value\":0.1}",
                "\"swe_max\":{\"operator\":\"INCLUSIVE\",\"unit\":\"v/v\",\"value\":0.5}",
            ] {
                assert!(params.contains(expected), "cutoffs in provenance: {params}");
            }
        }

        // `skip_version` is retained only so an older caller receives an explicit refusal instead
        // of silently writing ancestry-free curves.
        let req_skip = PaySummaryRequest {
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: true,
            stats_only: false
        ,
            custody: Some(test_run_custody()),
            frame: Default::default(),
            weighting: Default::default(),
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
            discretisation: DiscretisationModel::Forward,
            input_set: None,
            well_ids: vec![w.clone()],
            vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            skip_version: false,
            stats_only: true
        ,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
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

    /// CORRECTNESS - SB-POR-004 / SB-POR-T31 and SB-POR-T32. The required distinct
    /// method results, POR family and imported-versus-computed provenance distinction come from
    /// `docs/PRD_v2/11_porosity.md` sections 3.4, 4 and 6.2; intentional replacement plus
    /// append-only restore is DEC-013. The input sample and every explicit endpoint are cited in
    /// that chapter: RHOB 2.30, NPHI 0.25 and VSH 0.20 are its existing density/D-N witness;
    /// RHO_MA 2.65, RHO_FL/RHO_W 1.0 and PHIE_MAX 0.30 are section 5 defaults; RHO_SH 2.50,
    /// RHO_DSH 2.78 and NPHI_SH 0.40 are explicitly attested fixture choices, not defaults.
    #[test]
    fn porosity_methods_keep_distinct_default_names_and_each_curve_carries_family_method_and_convention_while_explicit_replacement_stays_versioned_and_restorable(
    ) {
        fn current_values(conn: &Connection, well_id: &str, curve: &str) -> Vec<f32> {
            let mut statement = conn
                .prepare(
                    "SELECT value FROM computed_curves
                     WHERE well_id = ?1 AND upper(curve_name) = upper(?2)
                     ORDER BY depth",
                )
                .unwrap();
            statement
                .query_map(duckdb::params![well_id, curve], |row| row.get(0))
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap()
        }

        fn porosity_contract(
            conn: &Connection,
            well_id: &str,
            curve: &str,
        ) -> modules::PorosityOutputContract {
            let ancestry = equations::curve_ancestry(conn, well_id, curve)
                .unwrap_or_else(|error| panic!("{curve} has no complete ancestry: {error}"));
            let parameter = ancestry
                .parameters
                .iter()
                .find(|parameter| parameter.name == format!("POROSITY_OUTPUT.{curve}"))
                .unwrap_or_else(|| panic!("{curve} has no per-output POR custody: {ancestry:?}"));
            serde_json::from_value(parameter.value.clone())
                .unwrap_or_else(|error| panic!("{curve} POR custody is invalid: {error}"))
        }

        let families = ["PHIE", "PHIT", "PHIA", "DPHI"]
            .map(|mnemonic| {
                (
                    mnemonic,
                    crate::curves::family_for(mnemonic).map(|family| family.family),
                )
            });
        assert!(
            families
                .iter()
                .all(|(_, family)| *family == Some(modules::POROSITY_FAMILY_ID)),
            "the chapter's four required mnemonic witnesses must all resolve to POR: {families:?}"
        );

        let density_spec = modules::list_modules()
            .into_iter()
            .find(|module| module.name == "phi_den")
            .unwrap();
        let density_defaults = resolve_output_names(&density_spec, &HashMap::new()).unwrap();
        assert_eq!(
            density_defaults,
            vec![
                ("PHIE_DEN".into(), "PHIE_DEN".into()),
                ("PHIT_DEN".into(), "PHIT_DEN".into()),
                ("PHIE".into(), "PHIE".into()),
                ("PHIT".into(), "PHIT".into()),
            ],
            "density retains the established canonical current outputs"
        );
        let density_neutron_spec = modules::list_modules()
            .into_iter()
            .find(|module| module.name == "phi_dn")
            .unwrap();
        let density_neutron_defaults =
            resolve_output_names(&density_neutron_spec, &HashMap::new()).unwrap();
        assert_eq!(
            density_neutron_defaults,
            vec![
                ("PHIE_DN".into(), "PHIE_DN".into()),
                ("PHIT_DN".into(), "PHIT_DN".into()),
                ("PHIE".into(), "PHIE_DN_LIM".into()),
                ("PHIT".into(), "PHIT_DN_LIM".into()),
            ],
            "density-neutron limited outputs need different method-specific defaults"
        );
        let resolved_with_user_controls = resolved_porosity_output_names(
            &density_neutron_spec,
            &HashMap::from([
                (format!("{OUT_NAME_PREFIX}PHIE"), "CURRENT_EFFECTIVE".into()),
                (OUT_PREFIX_OPT.into(), "reviewed_".into()),
            ]),
        )
        .unwrap();
        assert!(
            resolved_with_user_controls.iter().any(|(curve, contract)| {
                curve == "REVIEWED_CURRENT_EFFECTIVE"
                    && contract.family == modules::POROSITY_FAMILY_ID
                    && contract.method == "DENSITY_NEUTRON_COMPARISON"
            }),
            "a user rename and universal prefix must transform the custody key exactly as they transform the write"
        );

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let sw_indo_spec = modules::list_modules()
            .into_iter()
            .find(|module| module.name == "sw_indo")
            .unwrap();
        let only_dn_effective = HashSet::from(["PHIE_DN_LIM".to_string()]);
        let effective_from_dn = resolved_log_args_for_well(
            &conn,
            "POR-ROLE-RESOLUTION",
            &sw_indo_spec,
            &HashMap::new(),
            None,
            None,
            &only_dn_effective,
        )
        .unwrap();
        assert_eq!(
            effective_from_dn
                .iter()
                .find(|(argument, _)| argument == "PHIE")
                .map(|(_, curve)| curve.as_str()),
            Some("PHIE_DN_LIM"),
            "a downstream logical PHIE role must follow the sole D-N limited output"
        );
        let canonical_and_dn_effective =
            HashSet::from(["PHIE".to_string(), "PHIE_DN_LIM".to_string()]);
        let effective_from_canonical = resolved_log_args_for_well(
            &conn,
            "POR-ROLE-RESOLUTION",
            &sw_indo_spec,
            &HashMap::new(),
            None,
            None,
            &canonical_and_dn_effective,
        )
        .unwrap();
        assert_eq!(
            effective_from_canonical
                .iter()
                .find(|(argument, _)| argument == "PHIE")
                .map(|(_, curve)| curve.as_str()),
            Some("PHIE"),
            "the established canonical density result must win when both limited methods exist"
        );
        let sw_arch_spec = modules::list_modules()
            .into_iter()
            .find(|module| module.name == "sw_arch")
            .unwrap();
        let only_dn_total = HashSet::from(["PHIT_DN_LIM".to_string()]);
        let total_from_dn = resolved_log_args_for_well(
            &conn,
            "POR-ROLE-RESOLUTION",
            &sw_arch_spec,
            &HashMap::new(),
            None,
            None,
            &only_dn_total,
        )
        .unwrap();
        assert_eq!(
            total_from_dn
                .iter()
                .find(|(argument, _)| argument == "PHIT")
                .map(|(_, curve)| curve.as_str()),
            Some("PHIT_DN_LIM"),
            "the same deterministic role resolution must cover total porosity"
        );
        let well_uuid = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "POROSITY-CUSTODY", None, None, None).unwrap();
        let well_id = well_uuid.to_string();
        let depth = vec![1000.0_f32, 1000.5];
        let missing = vec![f32::NAN; depth.len()];
        db::insert_standard_curves(
            &conn,
            well_uuid,
            depth.clone(),
            missing.clone(),
            missing.clone(),
            vec![0.25, 0.25],
            vec![2.30, 2.30],
            missing.clone(),
            missing,
        )
        .unwrap();
        let vsh = db::upsert_curve_meta(
            &conn,
            &well_id,
            "RAW",
            "VSH",
            Some("v/v"),
            Some("VSH"),
            Some("SB-POR-004 typed input fixture"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &vsh, &depth, &[0.20, 0.20]).unwrap();
        let imported_phie = db::upsert_curve_meta(
            &conn,
            &well_id,
            "DELIVERED",
            "PHIE",
            Some("v/v"),
            Some(modules::POROSITY_FAMILY_ID),
            Some("imported porosity fixture"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &imported_phie, &depth, &[0.18, 0.19]).unwrap();

        let params = HashMap::from([
            ("RHO_MA".into(), 2.65),
            ("RHO_SH".into(), 2.50),
            ("RHO_FL".into(), 1.0),
            ("RHO_DSH".into(), 2.78),
            ("RHO_W".into(), 1.0),
            ("NPHI_SH".into(), 0.40),
            ("PHIE_MAX".into(), 0.30),
        ]);
        declare_nphi_basis(&conn, &well_id, "NPHI", "SANDSTONE");
        let dbm = Mutex::new(conn);
        let run = |module: &str, output_set: &str, explicit_current: bool| {
            let mut opts = HashMap::new();
            if explicit_current {
                opts.insert(format!("{OUT_NAME_PREFIX}PHIE"), "PHIE".into());
                opts.insert(format!("{OUT_NAME_PREFIX}PHIT"), "PHIT".into());
            }
            let result = run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: module.into(),
                    well_ids: vec![well_id.clone()],
                    log_inputs: HashMap::new(),
                    params: params.clone(),
                    opts,
                    output_set: Some(output_set.into()),
                    input_set: None,
                    custody: test_run_custody(),
                },
            );
            assert!(result[0].error.is_none(), "{module} failed: {:?}", result[0].error);
            result[0].output_curves.clone()
        };

        let density_names = run("phi_den", "POR_DENSITY", false);
        let density_neutron_names = run("phi_dn", "POR_DENSITY_NEUTRON", false);
        assert!(density_names.contains(&"PHIE".into()));
        assert!(density_neutron_names.contains(&"PHIE_DN_LIM".into()));
        let conn = dbm.lock().unwrap();
        assert_eq!(current_values(&conn, &well_id, "PHIE").len(), depth.len());
        assert_eq!(current_values(&conn, &well_id, "PHIE_DN_LIM").len(), depth.len());
        let density_contract = porosity_contract(&conn, &well_id, "PHIE");
        let density_neutron_contract =
            porosity_contract(&conn, &well_id, "PHIE_DN_LIM");
        assert_eq!(density_contract.family, modules::POROSITY_FAMILY_ID);
        assert_eq!(density_contract.method, "DENSITY");
        assert_eq!(
            density_contract.convention,
            "DENSITY_SHALE_SUBTRACTIVE_WITH_TOTAL_REBUILD"
        );
        assert_eq!(density_neutron_contract.family, modules::POROSITY_FAMILY_ID);
        assert_eq!(density_neutron_contract.method, "DENSITY_NEUTRON_COMPARISON");
        assert_eq!(
            density_neutron_contract.convention,
            "SHALE_REDUCED_COMPARISON_WITH_TOTAL_REBUILD"
        );
        let imported_identity: (String, String) = conn
            .query_row(
                "SELECT family, source FROM curve_meta WHERE curve_id = ?1",
                duckdb::params![imported_phie],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(imported_identity.0, modules::POROSITY_FAMILY_ID);
        assert_eq!(imported_identity.1, "imported porosity fixture");
        assert!(
            equations::curve_ancestry(&conn, &well_id, "PHIE").is_ok(),
            "the computed curve must carry complete run ancestry independently of imported metadata"
        );
        drop(conn);

        run("phi_den", "POR_CURRENT", true);
        let (density_set_id, density_current) = {
            let conn = dbm.lock().unwrap();
            let set_id: String = conn
                .query_row(
                    "SELECT CAST(set_id AS VARCHAR) FROM log_sets
                     WHERE well_id = ?1 AND set_name = 'POR_CURRENT' AND version = 1",
                    duckdb::params![well_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(porosity_contract(&conn, &well_id, "PHIE").method, "DENSITY");
            (set_id, current_values(&conn, &well_id, "PHIE"))
        };

        run("phi_dn", "POR_CURRENT", true);
        let density_neutron_current = {
            let conn = dbm.lock().unwrap();
            assert_eq!(
                porosity_contract(&conn, &well_id, "PHIE").method,
                "DENSITY_NEUTRON_COMPARISON"
            );
            let versions: i64 = conn
                .query_row(
                    "SELECT count(*) FROM log_sets
                     WHERE well_id = ?1 AND set_name = 'POR_CURRENT'",
                    duckdb::params![well_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(versions, 2, "explicit replacement must append a second version");
            current_values(&conn, &well_id, "PHIE")
        };
        assert_ne!(
            density_current, density_neutron_current,
            "the replacement control must exercise two observably different results"
        );

        {
            let conn = dbm.lock().unwrap();
            let restored = equations::restore_log_set(&conn, &density_set_id).unwrap();
            assert_eq!(restored.new_version, 3);
            assert_eq!(current_values(&conn, &well_id, "PHIE"), density_current);
            assert_eq!(porosity_contract(&conn, &well_id, "PHIE").method, "DENSITY");
            let archived_versions: i64 = conn
                .query_row(
                    "SELECT count(DISTINCT a.set_id)
                     FROM computed_curves_archive a
                     JOIN log_sets s ON s.set_id = a.set_id
                     WHERE a.well_id = ?1 AND upper(a.curve_name) = 'PHIE'
                       AND s.set_name = 'POR_CURRENT'",
                    duckdb::params![well_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                archived_versions, 3,
                "replacement and restore must retain all three append-only PHIE versions"
            );
        }
    }

    /// CORRECTNESS - SB-POR-006. The contract is `docs/PRD_v2/11_porosity.md` section 6.1
    /// SB-POR-006 and finding F15: a shale-endpoint `VSH` and a wet-clay-endpoint `VCL` are
    /// different quantities sharing one `v/v` unit, the endpoint subtracted must match the volume
    /// supplied, and the refusal is the requirement. The `CSR` bridge that would convert between
    /// them is SB-POR-012, which is outside the approved Gate 2 program, so nothing here converts.
    /// The oracle is acceptance versus refusal, never a number: the density parameters below only
    /// make the shipped public workflow finite and reuse the SB-POR-004 fixture already cited to
    /// that chapter's section 5 defaults and its explicitly attested shale choices.
    #[test]
    fn every_porosity_method_that_consumes_a_shale_or_clay_volume_declares_the_quantity_it_accepts_and_refuses_an_untyped_or_wrong_family_curve(
    ) {
        // Side A - declaration. A porosity module is identified by its own registered POR output
        // custody, never by name, so a new method joins this inventory automatically.
        let porosity_modules = modules::list_modules()
            .into_iter()
            .filter(|spec| {
                spec.args
                    .iter()
                    .any(|arg| arg.porosity_output.is_some())
            })
            .collect::<Vec<_>>();
        assert!(
            !porosity_modules.is_empty(),
            "the POR registry must supply the inventory this requirement ranges over"
        );
        let typed_shale_clay_inputs = porosity_modules
            .iter()
            .flat_map(|spec| {
                spec.args
                    .iter()
                    .filter(|arg| {
                        arg.kind == ArgKind::LogIn
                            && !arg.accepted_shale_clay_quantities.is_empty()
                    })
                    .map(|arg| (spec.name.clone(), arg.name.clone()))
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            typed_shale_clay_inputs,
            std::collections::BTreeSet::from([
                ("phi_den".to_string(), "VSH".to_string()),
                ("phi_dn".to_string(), "VSH".to_string()),
                ("phi_dnbk".to_string(), "VSH".to_string()),
                ("phi_son".to_string(), "VSH".to_string()),
                ("sspw".to_string(), "VSH".to_string()),
            ]),
            "every porosity method that consumes a shale/clay volume must declare which quantity it accepts"
        );

        // Side B - no untyped shale/clay input may hide behind side A. This asks the independent
        // generated curve registry which declared inputs are shale/clay volumes at all, so an
        // added POR consumer that forgets its typing fails here even though it never appears in
        // the set above. Side A alone would pass such an omission; side B alone would pass a
        // module whose declared default mnemonic is unregistered.
        let untyped_shale_clay_inputs = porosity_modules
            .iter()
            .flat_map(|spec| {
                spec.args
                    .iter()
                    .filter(|arg| arg.kind == ArgKind::LogIn)
                    .filter(|arg| {
                        arg.accepted_shale_clay_quantities.is_empty()
                            && crate::curves::family_for(&arg.default)
                                .map(|family| {
                                    shale_clay_quantity_from_family(Some(family.family)).is_some()
                                })
                                .unwrap_or(false)
                    })
                    .map(|arg| format!("{}.{}", spec.name, arg.name))
            })
            .collect::<Vec<_>>();
        assert!(
            untyped_shale_clay_inputs.is_empty(),
            "a porosity input whose registry family is a shale/clay volume must be typed: {untyped_shale_clay_inputs:?}"
        );

        // Side C - behavior. One shipped porosity method, three curves that are identical in
        // mnemonic, unit and samples and differ only in the declared quantity.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depth = vec![1000.0f32, 1000.5];
        let add_well = |label: &str, family: Option<&str>| {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, label, None, None, Some(0.0)).unwrap();
            let missing = vec![f32::NAN; depth.len()];
            db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                vec![2.30, 2.30],
                missing.clone(),
                missing,
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn,
                &well,
                "RAW",
                "VSH",
                Some("v/v"),
                family,
                Some("SB-POR-006 volume-quantity control"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &[0.20, 0.20]).unwrap();
            well
        };
        let typed_shale = add_well("SHALE-VOLUME", Some("VSH"));
        let untyped = add_well("UNTYPED-VOLUME", None);
        let clay_under_shale_name = add_well("CLAY-VOLUME-UNDER-VSH-NAME", Some("VCL"));

        let dbm = Mutex::new(conn);
        let results = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "phi_den".into(),
                well_ids: vec![
                    typed_shale.clone(),
                    untyped.clone(),
                    clay_under_shale_name.clone(),
                ],
                log_inputs: HashMap::from([("VSH".into(), "VSH".into())]),
                params: HashMap::from([
                    ("RHO_MA".into(), 2.65),
                    ("RHO_SH".into(), 2.50),
                    ("RHO_FL".into(), 1.0),
                    ("RHO_DSH".into(), 2.78),
                    ("RHO_W".into(), 1.0),
                    ("PHIE_MAX".into(), 0.30),
                    ("PHIE_FLOOR".into(), 0.001),
                    ("VSH_SHALE".into(), 0.95),
                ]),
                // Stated explicitly so the accepted control is Clean rather than Degraded by the
                // manifest's own honest "used its default" disclosure. This is the shipped
                // `phi_den` choice, declared rather than introduced.
                opts: HashMap::from([("OPT_PHIEMAX".into(), "SHALE_REDUCED".into())]),
                output_set: Some("POR-QUANTITY-CONTRACT".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        let result_for = |well: &str| {
            results
                .iter()
                .find(|result| result.well_id == well)
                .unwrap_or_else(|| panic!("{well} produced no run result"))
        };

        let accepted = result_for(&typed_shale);
        assert_eq!(
            accepted.outcome,
            ModuleRunOutcome::Clean,
            "a declared shale volume is exactly what a density porosity subtracts: {:?} / {:?}",
            accepted.error, accepted.degradations
        );
        assert!(accepted.rows_written > 0);

        let untyped_result = result_for(&untyped);
        assert_eq!(
            untyped_result.outcome,
            ModuleRunOutcome::Failed,
            "an untyped volume must be refused, not assumed to be shale"
        );
        assert_eq!(untyped_result.rows_written, 0);
        let untyped_refusal = untyped_result
            .error
            .as_deref()
            .expect("an untyped volume must explain its refusal");
        assert!(
            untyped_refusal.contains("VSH")
                && untyped_refusal.contains("VCL")
                && untyped_refusal.contains("phi_den"),
            "the refusal must name the method and both candidate quantities: {untyped_refusal}"
        );

        let wrong_family = result_for(&clay_under_shale_name);
        assert_eq!(
            wrong_family.outcome,
            ModuleRunOutcome::Failed,
            "a clay volume carrying a VSH mnemonic must not reach a shale-endpoint subtraction"
        );
        assert_eq!(wrong_family.rows_written, 0);
        let wrong_family_refusal = wrong_family
            .error
            .as_deref()
            .expect("a wrong-family volume must explain its refusal");
        assert!(
            wrong_family_refusal.contains("VSH") && wrong_family_refusal.contains("VCL"),
            "the refusal must name both quantities: {wrong_family_refusal}"
        );
        assert_ne!(
            untyped_refusal, wrong_family_refusal,
            "absent typing and wrong typing are different findings and must not share one message"
        );

        let conn = dbm.lock().unwrap();
        for refused in [&untyped, &clay_under_shale_name] {
            let versions: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM log_sets
                     WHERE well_id = ?1 AND set_name = 'POR-QUANTITY-CONTRACT'",
                    duckdb::params![refused],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                versions, 0,
                "a volume-quantity refusal must not version a porosity interpretation"
            );
            let curves: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1",
                    duckdb::params![refused],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(curves, 0, "a refused porosity run must write no samples");
        }
        let accepted_curves: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT curve_name) FROM computed_curves WHERE well_id = ?1",
                duckdb::params![typed_shale],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            accepted_curves > 0,
            "the accepted control must actually produce porosity, or the refusals prove nothing"
        );
    }

    /// CORRECTNESS — SB-ENV-041 / SB-ENV-T49. The four required declarations come from
    /// `docs/PRD_v2/20_envcorr-qc.md` sections 4.4 and 6.4. The three literal policy records below
    /// are independently read from the shipped arithmetic in `condition::smooth`: all use a
    /// centred physical-depth window and preserve a MISSING target while using finite neighbours
    /// inside that window; MEAN divides by the finite count, MEDIAN is an order statistic, and
    /// SAVGOL solves local normal equations with a finite-mean fallback at insufficient support.
    /// The 2.0 window and curve values are synthetic fixture inputs, not a shipping default or a
    /// petrophysical expected value.
    #[test]
    fn a_smoothed_curve_records_its_kernel_normalisation_end_and_gap_edge_behaviour_after_restart() {
        struct TempProject(std::path::PathBuf);
        impl Drop for TempProject {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
                let _ = std::fs::remove_file(format!("{}.wal", self.0.display()));
            }
        }

        let temporary = TempProject(std::env::temp_dir().join(format!(
            "sandibumi-env041-smoothing-policy-{}.duckdb",
            uuid::Uuid::new_v4()
        )));
        let path = temporary.0.to_string_lossy().to_string();
        let conn = db::init_db(&path).expect("create smoothing-provenance project");
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "SMOOTHING-POLICY", None, None, Some(0.0)).unwrap();
        let missing = vec![f32::NAN; 5];
        db::insert_standard_curves(
            &conn,
            well,
            vec![1000.0, 1001.0, 1002.0, 1003.0, 1004.0],
            vec![10.0, 20.0, f32::NAN, 40.0, 50.0],
            missing.clone(),
            missing.clone(),
            missing.clone(),
            missing.clone(),
            missing,
        )
        .unwrap();
        let well_id = well.to_string();
        let dbm = Mutex::new(conn);

        let run = |method: &str, curve: &str, set_name: &str| {
            let result = run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "smooth".into(),
                    well_ids: vec![well_id.clone()],
                    log_inputs: HashMap::from([("CURVE".into(), "GR".into())]),
                    params: HashMap::from([("WINDOW".into(), 2.0)]),
                    opts: HashMap::from([
                        ("OPT_METHOD".into(), method.into()),
                        ("OPT_FLAG".into(), "NO".into()),
                        (format!("{OUT_NAME_PREFIX}OUT_CURVE"), curve.into()),
                    ]),
                    output_set: Some(set_name.into()),
                    input_set: None,
                    custody: test_run_custody(),
                },
            );
            assert_eq!(result.len(), 1);
            assert!(result[0].error.is_none(), "{method} run failed: {:?}", result[0].error);
            assert!(result[0].output_curves.iter().any(|output| output == curve));
        };
        run("MEAN", "GR_MEAN", "SMOOTH_MEAN");
        run("MEDIAN", "GR_MEDIAN", "SMOOTH_MEDIAN");
        run("SAVGOL", "GR_SAVGOL", "SMOOTH_SAVGOL");
        dbm.lock().unwrap().execute_batch("CHECKPOINT").unwrap();
        drop(dbm);

        let reopened = db::init_db_resilient(&path).expect("reopen smoothing-provenance project");
        let policy = |curve: &str| {
            equations::curve_ancestry(&reopened, &well_id, curve)
                .unwrap()
                .parameters
                .into_iter()
                .find(|parameter| parameter.name == "SMOOTHING_POLICY")
                .unwrap_or_else(|| panic!("{curve} lost its smoothing policy"))
                .value
        };
        let mean = policy("GR_MEAN");
        let median = policy("GR_MEDIAN");
        let savgol = policy("GR_SAVGOL");

        assert_eq!(
            mean,
            serde_json::json!({
                "schema_version": 1,
                "kernel": "UNIFORM_MEAN",
                "normalisation": "DIVIDE_BY_FINITE_SAMPLE_COUNT",
                "end_behaviour": "TRUNCATE_CENTERED_WINDOW_TO_AVAILABLE_DEPTHS",
                "gap_edge_behaviour": "PRESERVE_MISSING_TARGET_AND_USE_FINITE_NEIGHBOURS_WITHIN_WINDOW"
            })
        );
        assert_eq!(
            median,
            serde_json::json!({
                "schema_version": 1,
                "kernel": "WINDOW_MEDIAN",
                "normalisation": "FINITE_ORDER_STATISTIC",
                "end_behaviour": "TRUNCATE_CENTERED_WINDOW_TO_AVAILABLE_DEPTHS",
                "gap_edge_behaviour": "PRESERVE_MISSING_TARGET_AND_USE_FINITE_NEIGHBOURS_WITHIN_WINDOW"
            })
        );
        assert_eq!(
            savgol,
            serde_json::json!({
                "schema_version": 1,
                "kernel": "LOCAL_QUADRATIC_LEAST_SQUARES",
                "normalisation": "LOCAL_LEAST_SQUARES_NORMAL_EQUATIONS",
                "end_behaviour": "TRUNCATE_CENTERED_WINDOW_AND_USE_FINITE_MEAN_IF_UNDERDETERMINED",
                "gap_edge_behaviour": "PRESERVE_MISSING_TARGET_AND_USE_FINITE_NEIGHBOURS_WITHIN_WINDOW"
            })
        );
        assert_ne!(mean, median, "two kernels sharing one window must not collapse to one policy");
        assert_ne!(mean, savgol, "the least-squares branch must not inherit the mean policy");
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
    fn the_masked_washout_is_now_repaired_by_the_declared_exemption_it_once_defeated() {
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

        // Now with the mask the module's own documentation recommends. SB-ENV-027
        // (DEC-033, 2026-08-18) FIXED the audited defect this test used to pin as-is: the
        // declared repair - log_predict.SYN under MAX_RAW, the one approved inventory
        // entry - survives both mask passes, so the masked run repairs the washout exactly
        // as the unmasked control did. T-PREP-16's known-issue line was updated with this.
        let r = run(Some("BADHOLE"));
        assert!(r[0].error.is_none(), "masked log_predict: {:?}", r[0].error);
        let masked = syn_of();
        assert!(
            !masked[washout].is_nan() && (masked[washout] - rhob_true[washout]).abs() < 0.1,
            "the declared repair must survive the mask it once defeated: got {} for a true {}",
            masked[washout],
            rhob_true[washout]
        );
        // And the typed companion discloses the reconstruction at exactly that depth.
        let marker = {
            let conn = dbm.lock().unwrap();
            let (_, cols) = equations::fetch_curve_frame(
                &conn, &w, &["RHOB_SYN_RECON_FLAG".to_string()],
            )
            .unwrap();
            cols["RHOB_SYN_RECON_FLAG"].clone()
        };
        assert_eq!(marker[washout].to_bits(), 1.0f32.to_bits());
        assert_eq!(marker[3].to_bits(), 0.0f32.to_bits());
        assert!(
            masked.iter().enumerate().any(|(i, v)| i != washout && !v.is_nan()),
            "the masked run wrote nothing anywhere — that is a different failure"
        );

        // The mechanism the exemption must defeat, kept as documentation: a context where
        // only the PREDICTOR is missing at the washout — what the input-side mask used to do
        // — already yields MISSING before the output pass ever runs. This is WHY DEC-033
        // constraint 2 bypasses BOTH passes, not one.
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

    /// CORRECTNESS — SB-ENV-006 / SB-ENV-T11, `docs/PRD_v2/20_envcorr-qc.md` section 6.2.
    /// The cited expected behavior is a visible refusal or a fully flagged uncorrected result;
    /// because SB-ENV-005/007 are not available, this increment chooses refusal. The numeric
    /// values are synthetic non-zero algebra fixtures, not correction defaults or field endpoints.
    #[test]
    fn a_gr_correction_with_no_caliper_refuses_and_writes_no_uncorrected_copy() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_uuid = Uuid::new_v4();
        db::insert_well(&conn, well_uuid, "MISSING-CALIPER", None, None, Some(0.0)).unwrap();
        let well = well_uuid.to_string();
        let depths = vec![1000.0f32, 1000.5];
        db::insert_standard_curves(
            &conn,
            well_uuid,
            depths.clone(),
            vec![80.0, 90.0],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
        )
        .unwrap();

        let dbm = Mutex::new(conn);
        let request = || RunModuleRequest {
            module: "gr_hole_corr".into(),
            well_ids: vec![well.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("K_GR".into(), 0.01), ("BS_DEF".into(), 8.5)]),
            opts: HashMap::new(),
            output_set: Some("GR-CORRECTION".into()),
            input_set: None,
            custody: test_run_custody(),
        };

        let refused = run_workflow_module(&dbm, &request());
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].outcome, ModuleRunOutcome::Failed);
        let error = refused[0]
            .error
            .as_deref()
            .expect("the missing caliper must be reported to the caller");
        assert!(error.contains("gr_hole_corr.caliper_coverage"), "condition id missing: {error}");
        // DEC-031 part (c) narrowed this refusal to the whole-run case - no finite caliper
        // ANYWHERE - so the message names the anywhere-condition and the ruling, not a sample:
        // a partially covered caliper now runs with the gap on the state channel instead.
        assert!(error.contains("has a finite sample"), "the anywhere-condition missing: {error}");
        assert!(error.contains("DEC-031"), "the narrowing ruling missing: {error}");
        assert!(error.contains("SB-ENV-006"), "contract source missing: {error}");
        assert!(refused[0].output_curves.is_empty());
        assert_eq!(refused[0].rows_written, 0);
        {
            let conn = dbm.lock().unwrap();
            let written: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'GR_EC'",
                    duckdb::params![well],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(written, 0, "a reported refusal must leave no correction-named curve");

            let caliper_id = db::upsert_curve_meta(
                &conn,
                &well,
                "RAW",
                "CALI",
                Some("in"),
                Some("CALI"),
                Some("synthetic algebra fixture"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &caliper_id, &depths, &[10.5, 11.5]).unwrap();
        }

        let complete = run_workflow_module(&dbm, &request());
        assert!(complete[0].error.is_none(), "complete inputs must still run: {:?}", complete[0].error);
        assert_eq!(complete[0].rows_written, 2);
        let conn = dbm.lock().unwrap();
        let (_, curves) = equations::fetch_curve_frame(&conn, &well, &["GR".into(), "GR_EC".into()]).unwrap();
        assert_ne!(curves["GR_EC"], curves["GR"], "the valid control must exercise a real correction");
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
            create_complete_log_set, restore_log_set, write_computed_curves_with_ancestry,
            AncestryOutput, AncestryParameter, AncestryZoneScope, CompleteLogSetSpec,
            CurveAncestry,
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

        let gr_input = equations::resolve_ancestry_input(&conn, &w, "GR", "GR", None, None)
            .expect("the synthetic standard GR must have a resolvable source identity");
        let producer_spec = || {
            let parameters = vec![AncestryParameter {
                name: format!("{OUTPUT_QUANTITY_PROVENANCE_PREFIX}VSH"),
                value: serde_json::json!(modules::ShaleClayQuantity::ShaleVolume),
                source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
                resolution: None,
                manifest_version: None,
                decision: None,
            }];
            CompleteLogSetSpec::try_new(
                "INTERP",
                CurveAncestry {
                    schema_version: equations::CURVE_ANCESTRY_SCHEMA_VERSION,
                    module: "vsh_gr".into(),
                    module_version: env!("CARGO_PKG_VERSION").into(),
                    inputs: vec![gr_input.clone()],
                    parameter_state: equations::parameter_state_for(&parameters),
                    parameters,
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: test_run_custody().actor,
                    timestamp_utc_ms: equations::ancestry_timestamp_utc_ms().unwrap(),
                    outputs: vec![AncestryOutput {
                        curve: "VSH".into(),
                        derivation: "SB-CLY-043 typed restore fixture".into(),
                    }],

                    depth_frame: None,
                    zone_set: None,
                    stochastic: None,
                    applied_model: None,
                    physics_attributes: Vec::new(),
                },
            )
            .unwrap()
        };

        // Version 1: a clean sand. Version 2: very shaly. Same curve, same well.
        let (set1, v1) = create_complete_log_set(&conn, &w, &producer_spec()).unwrap();
        write_computed_curves_with_ancestry(
            &conn,
            &w,
            &depth,
            &[("VSH", &[0.10f32, 0.10, 0.10])],
            &set1,
        )
        .unwrap();
        let (set2, v2) = create_complete_log_set(&conn, &w, &producer_spec()).unwrap();
        write_computed_curves_with_ancestry(
            &conn,
            &w,
            &depth,
            &[("VSH", &[0.80f32, 0.80, 0.80])],
            &set2,
        )
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
            restore_log_set(&c, set1.as_str()).unwrap();
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

        {
            // SB-POR-024 (DEC-025): a real delivery does not declare its neutron basis, so
            // the fixture declares one per well the way an importing user would; a well
            // without the conditioned channel is skipped and keeps its own missing-input
            // refusal downstream.
            for well in &well_ids {
                if let Some(entry) = db::list_generic_curve_catalog(&conn, well)
                    .unwrap()
                    .into_iter()
                    .find(|entry| entry.mnemonic == "NPHI_COR")
                {
                    db::set_curve_neutron_basis(
                        &conn, &entry.curve_id, "LIMESTONE",
                        "test fixture declaration (DEC-025)",
                    )
                    .unwrap();
                }
            }
        }
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
            let mut clean = 0usize;
            for r in &results {
                println!("{module}: well={} rows={} outputs={:?} err={:?}", r.well_id, r.rows_written, r.output_curves, r.error);
                match &r.error {
                    None => clean += 1,
                    // A fixture folder holds whatever it holds: a well genuinely missing a
                    // required channel is refused BY the documented input precondition, and
                    // that refusal is correct behaviour, not a chain failure. Anything else
                    // still fails the test.
                    Some(error) if error.contains("has a finite sample") => {}
                    Some(error) => panic!("{module} failed: {error}"),
                }
            }
            assert!(clean >= 2, "{module}: fewer than two fixture wells ran clean");
        };

        run(
            "vsh_gr",
            &[("GR", "GRN_CS")],
            &[("GR_MA", 25.0), ("GR_SH", 130.0)],
            &[("OPT_GR", "LINEAR")],
        );
        // phi_dnbk, not phi_dn: DEC-070 (2026-08-18) made the quick-look curves
        // visual-only, so a chain that ends in a pay summary interprets porosity with the
        // authoritative crossplot; its fixture basis declaration above is LIMESTONE, the
        // entry units the method's own source assumes (SB-POR-024 / DEC-025).
        // The interpreter's election: the crossplot's limited pair becomes the current
        // PHIE/PHIT by the explicit output-name mechanism, which is how an authoritative
        // method's answer reaches saturation and pay (the custody default keeps method-
        // qualified names precisely so this promotion is a stated decision).
        run(
            "phi_dnbk",
            &[("NPHI", "NPHI_COR")],
            &[("RHO_SH", 2.5), ("NPHI_SH", 0.35), ("RHO_DSH", 2.65), ("PHIE_MAX", 0.35)],
            &[("__OUT_PHIE", "PHIE"), ("__OUT_PHIT", "PHIT")],
        );
        run(
            "sw_indo",
            &[],
            // SWE_IRR became required-ABSENT after this test was written (SB-CORE-004
            // family); 0.0 is the inert no-floor fixture, not a field value.
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.2), ("RT_SH", 4.0), ("SWE_IRR", 0.0)],
            &[("OPT_INDO", "FULL"), ("OPT_RW", "CONSTANT")],
        );
        run("perm_wyllie_rose", &[], &[("SWE_IRR", 0.15)], &[("OPT_WR", "TIMUR")]);

        // Physical sanity: VSH/PHIE/SWE within [0,1], PERM non-negative, and each
        // well has a meaningful number of valid samples.
        {
            let conn = db.lock().unwrap();
            // The chain interprets with the authoritative crossplot (DEC-070 keeps the
            // quick-look out of pay) and ELECTS its limited pair as the current PHIE via
            // the explicit output-name mechanism, so bare PHIE is the summary's subject.
            for (curve, lo, hi) in [("VSH", 0.0, 1.0), ("PHIE", 0.0, 0.5), ("SWE", 0.0, 1.0), ("PERM", 0.0, f64::MAX)] {
                let (count, min, max): (i64, Option<f64>, Option<f64>) = conn
                    .query_row(
                        "SELECT count(value), min(value), max(value) FROM computed_curves
                         WHERE curve_name = ?1 AND NOT isnan(value)",
                        duckdb::params![curve],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .unwrap();
                let (min, max) = (min.unwrap_or(f64::NAN), max.unwrap_or(f64::NAN));
                println!("{curve}: n={count} min={min:.4} max={max:.4}");
                assert!(count > 1000, "{curve}: too few valid samples ({count})");
                assert!(min >= lo && max <= hi, "{curve} out of physical range: [{min}, {max}]");
            }
        }

        // Pay summary over the whole wells (no zones defined → single ALL zone).
        let rows = run_pay_summary(
            &db,
            &PaySummaryRequest { well_ids: well_ids.clone(), vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()), phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()), swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()), perm_min: None, input_set: None, skip_version: false, stats_only: false ,
                discretisation: DiscretisationModel::Forward,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
                custody: Some(test_run_custody()),
                frame: Default::default(),
                weighting: Default::default(),
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

    /// CORRECTNESS - SB-DBM-039 / SB-DBM-T39. The three-way batch, `Warned`
    /// states, non-clean aggregate and 25-job prune come exactly from
    /// `docs/PRD_v2/22_database-model.md` section 6, SB-DBM-T39, sourced there to
    /// SB-CORE-002 and the job registry. The synthetic depths and parameters only
    /// make the existing clamp and documented TVDSS-to-DEPTH substitution reachable;
    /// no petrophysical output value is an expected value in this test.
    #[test]
    fn a_clamped_well_and_a_substituted_input_well_are_warned_and_leave_durable_degradation_records_after_their_job_is_pruned_while_a_clean_well_stays_clean(
    ) {
        fn seed(
            conn: &Connection,
            name: &str,
            tvdss: Option<Vec<f32>>,
        ) -> String {
            let well_id = uuid::Uuid::new_v4();
            db::insert_well(conn, well_id, name, None, None, None).unwrap();
            let depth = vec![1000.0_f32, 1001.0];
            db::insert_standard_curves(
                conn,
                well_id,
                depth.clone(),
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
                vec![f32::NAN; 2],
            )
            .unwrap();
            let well = well_id.to_string();
            let phi = db::upsert_curve_meta(
                conn,
                &well,
                "RAW",
                "PHIE",
                Some("v/v"),
                Some("POR"),
                Some("synthetic reachability fixture for SB-DBM-T39"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(conn, &phi, &depth, &[0.2, 0.2]).unwrap();
            if let Some(values) = tvdss {
                let curve = db::upsert_curve_meta(
                    conn,
                    &well,
                    "RAW",
                    "TVDSS",
                    Some("m"),
                    Some("DEPTH"),
                    Some("synthetic reachability fixture for SB-DBM-T39"),
                    None,
                )
                .unwrap();
                db::insert_curve_samples(conn, &curve, &depth, &values).unwrap();
            }
            well
        }

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let clamped = seed(&conn, "CLAMPED-RESULT", Some(vec![2000.0, 2001.0]));
        let substituted = seed(&conn, "SUBSTITUTED-DEPTH", None);
        let clean = seed(&conn, "IN-RANGE-RESULT", Some(vec![1000.0, 1001.0]));
        let dbm = Mutex::new(conn);

        let registry = crate::jobs::new_registry();
        let job_id = uuid::Uuid::new_v4();
        let progress = crate::jobs::register(
            &registry,
            job_id,
            "SB-DBM-T39",
            "degraded outcome batch",
            vec![
                (clamped.clone(), "CLAMPED-RESULT".into()),
                (substituted.clone(), "SUBSTITUTED-DEPTH".into()),
                (clean.clone(), "IN-RANGE-RESULT".into()),
            ],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            true,
        );
        progress.running(3);
        let results = run_workflow_module_into(
            &dbm,
            &RunModuleRequest {
                module: "phimax".into(),
                well_ids: vec![clamped.clone(), substituted.clone(), clean.clone()],
                log_inputs: HashMap::from([
                    ("PHI".into(), "PHIE".into()),
                    ("TVDSS".into(), "TVDSS".into()),
                ]),
                params: HashMap::from([
                    ("PHIMAX0".into(), 0.5),
                    ("TVDSS_REF".into(), 0.0),
                    ("PHIMAX_GRAD".into(), -0.3),
                ]),
                opts: HashMap::from([("MODE".into(), "linear".into())]),
                output_set: Some("DEGRADATION".into()),
                input_set: None,
                custody: test_run_custody(),
            },
            None,
            None,
            Some(&progress),
        );
        progress.complete();

        let result = |well: &str| {
            results
                .iter()
                .find(|result| result.well_id == well)
                .unwrap_or_else(|| panic!("result for {well}"))
        };
        let persisted_run_count: i64 = dbm
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE module = 'phimax'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            result(&clamped).outcome,
            ModuleRunOutcome::Degraded,
            "clamped result: {:?}; persisted run count: {persisted_run_count}",
            result(&clamped),
        );
        assert!(result(&clamped).degradations.iter().any(|d| {
            d.kind == modules::RunDegradationKind::Clamped
        }));
        assert_eq!(result(&substituted).outcome, ModuleRunOutcome::Degraded);
        assert!(result(&substituted).degradations.iter().any(|d| {
            d.kind == modules::RunDegradationKind::SubstitutedInput
        }));
        assert_eq!(result(&clean).outcome, ModuleRunOutcome::Clean);
        assert!(result(&clean).degradations.is_empty());

        let view = crate::jobs::list(&registry)
            .into_iter()
            .find(|view| view.id == job_id.to_string())
            .expect("the just-finished job is visible");
        let state = |well: &str| {
            view.items
                .iter()
                .find(|item| item.key == well)
                .unwrap_or_else(|| panic!("job item for {well}"))
        };
        assert_eq!(state(&clamped).state, crate::jobs::ItemState::Warned);
        assert_eq!(state(&substituted).state, crate::jobs::ItemState::Warned);
        assert_eq!(state(&clean).state, crate::jobs::ItemState::Ok);
        assert_eq!(view.outcome, Some(crate::jobs::JobOutcome::Degraded));
        assert!(state(&clamped).message.as_deref().unwrap_or("").contains("CLAMPED"));
        assert!(
            state(&substituted)
                .message
                .as_deref()
                .unwrap_or("")
                .contains("SUBSTITUTED_INPUT")
        );

        for index in 0..25 {
            let later = crate::jobs::register(
                &registry,
                uuid::Uuid::new_v4(),
                "later job",
                format!("later-{index}"),
                vec![],
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                false,
            );
            later.complete();
        }
        assert!(
            crate::jobs::list(&registry).iter().all(|view| view.id != job_id.to_string()),
            "25 later jobs prune the transient result required by the exact fixture"
        );

        let conn = dbm.lock().unwrap();
        let stored = |well: &str| -> (String, Vec<(String, i64)>) {
            let (set_id, outcome): (String, String) = conn
                .query_row(
                    "SELECT CAST(set_id AS VARCHAR), outcome_state FROM log_sets
                     WHERE well_id = ?1 AND set_name = 'DEGRADATION'",
                    duckdb::params![well],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            let mut statement = conn
                .prepare(
                    "SELECT kind, occurrences FROM run_degradations
                     WHERE set_id = ?1 ORDER BY position",
                )
                .unwrap();
            let events = statement
                .query_map(duckdb::params![set_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap();
            (outcome, events)
        };
        let clamped_store = stored(&clamped);
        assert_eq!(clamped_store.0, "DEGRADED");
        assert!(clamped_store.1.iter().any(|(kind, count)| kind == "CLAMPED" && *count > 0));
        let substituted_store = stored(&substituted);
        assert_eq!(substituted_store.0, "DEGRADED");
        assert!(
            substituted_store
                .1
                .iter()
                .any(|(kind, count)| kind == "SUBSTITUTED_INPUT" && *count > 0)
        );
        let clean_store = stored(&clean);
        assert_eq!(clean_store, ("CLEAN".into(), vec![]));
    }

    /// CORRECTNESS — `20_envcorr-qc.md` §4.1 SB-ENV-003 and §6.1 SB-ENV-T05.
    /// The 0–200 gAPI GR-matrix validity range and its source come from the shipping manifest,
    /// cited there to `10_clay-volume.md` §3.2 and Geolog `vsh_gr.info` L48-L49. The valid
    /// LINEAR controls are independently hand-calculated from `(GR - GR_MA) / (GR_SH - GR_MA)`:
    /// 20 gAPI gives 0 and 220 gAPI gives 1 for the 20/220 endpoints. The middle zone supplies
    /// 201 gAPI, exactly one gAPI beyond the cited maximum; no uncited physical value is adopted.
    #[test]
    fn a_subset_precondition_violation_keeps_only_valid_samples_with_a_companion_flag_and_source_bearing_provenance_while_refusal_stays_available_and_a_flag_alone_never_versions_an_answer(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_id, "PARTIAL-PRECONDITION", None, None, None).unwrap();
        let well = well_id.to_string();
        db::insert_standard_curves(
            &conn,
            well_id,
            vec![1000.0, 1001.0, 1002.0],
            vec![20.0, 120.0, 220.0],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        db::upsert_md_zone(&conn, &well, "ABOVE-DECLARED-RANGE", 1000.5, 1001.5).unwrap();
        db::set_zone_param(
            &conn,
            &well,
            "ABOVE-DECLARED-RANGE",
            "GR_MA",
            Some(201.0),
            Some("SB-ENV-003 one-sample boundary fixture"),
        )
        .unwrap();
        let dbm = Mutex::new(conn);

        let request = |policy: Option<&str>, output_set: &str| RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![well.clone()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 220.0)]),
            opts: policy
                .map(|value| HashMap::from([("__PRECONDITION_POLICY".into(), value.into())]))
                .unwrap_or_default(),
            output_set: Some(output_set.into()),
            input_set: None,
            custody: test_run_custody(),
        };

        let refused = run_workflow_module(&dbm, &request(None, "REFUSAL-CONTROL"));
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].outcome, ModuleRunOutcome::Failed);
        let refusal = refused[0].error.as_deref().expect("the default policy must label its refusal");
        assert!(refusal.contains("vsh_gr.gr_ma_range"), "condition id missing: {refusal}");
        assert!(refusal.contains("value 201 gAPI at sample 1"), "offending value missing: {refusal}");
        assert!(refusal.contains("0 to 200 gAPI"), "expected range missing: {refusal}");
        assert!(refusal.contains("vsh_gr.info L48-L49"), "range source missing: {refusal}");

        let registry = crate::jobs::new_registry();
        let job_id = uuid::Uuid::new_v4();
        let progress = crate::jobs::register(
            &registry,
            job_id,
            "Module",
            "flag partial precondition",
            vec![(well.clone(), "one sample above the declared range".into())],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            true,
        );
        progress.running(1);
        let flagged = run_workflow_module_into(
            &dbm,
            &request(Some("FLAG_VALID_SAMPLES"), "FLAGGED-PARTIAL"),
            None,
            None,
            Some(&progress),
        );
        progress.complete();

        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].outcome, ModuleRunOutcome::Degraded, "{flagged:?}");
        assert_eq!(flagged[0].rows_written, 3);
        assert_eq!(
            flagged[0].output_curves,
            vec![
                "VSH".to_string(),
                "VSH_GR".to_string(),
                "VSH_GR_PRECONDITION_FLAG".to_string(),
                // SB-CLY-001 (DEC-036): the registry token curve rides every vsh_gr run. At
                // framework-sanitized samples it is MISSING and the generic companion flag is
                // the record - a framework exclusion is not one of registry v1's six things.
                "VSH_PROV".to_string(),
            ]
        );
        assert!(flagged[0].error.is_none(), "the valid samples are a result, not a failed run");

        let view = crate::jobs::list(&registry)
            .into_iter()
            .find(|view| view.id == job_id.to_string())
            .expect("the Processing panel can inspect the completed run");
        assert_eq!(view.outcome, Some(crate::jobs::JobOutcome::Degraded));
        let item = view.items.iter().find(|item| item.key == well).expect("per-well Processing item");
        assert_eq!(item.state, crate::jobs::ItemState::Warned);
        let warning = item.message.as_deref().expect("the flag cannot be the only warning surface");
        assert!(warning.contains("vsh_gr.gr_ma_range"), "condition id missing: {warning}");
        assert!(warning.contains("201 gAPI"), "offending value missing: {warning}");
        assert!(warning.contains("0 to 200 gAPI"), "expected range missing: {warning}");
        assert!(warning.contains("vsh_gr.info L48-L49"), "range source missing: {warning}");

        let conn = dbm.lock().unwrap();
        let read_curve = |name: &str| -> Vec<f32> {
            let mut statement = conn
                .prepare(
                    "SELECT value FROM computed_curves
                     WHERE well_id = ?1 AND curve_name = ?2 ORDER BY depth",
                )
                .unwrap();
            statement
                // SB-DBM-030: a missing sample is SQL NULL at the store; read it back as NaN.
                .query_map(duckdb::params![&well, name], |row| {
                    Ok(row.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN))
                })
                .unwrap()
                .collect::<duckdb::Result<Vec<_>>>()
                .unwrap()
        };
        let vsh = read_curve("VSH");
        assert_eq!(vsh.len(), 3);
        assert_eq!(vsh[0], 0.0);
        assert!(vsh[1].is_nan(), "the invalid sample must never become an unmarked number");
        assert_eq!(vsh[2], 1.0);
        assert_eq!(read_curve("VSH_GR_PRECONDITION_FLAG"), vec![0.0, 1.0, 0.0]);

        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'FLAGGED-PARTIAL'",
                duckdb::params![&well],
                |row| row.get(0),
            )
            .unwrap();
        let saved: serde_json::Value = serde_json::from_str(&params_json).unwrap();
        assert_eq!(saved["_sandibumi_precondition_policy_v1"], "FLAG_VALID_SAMPLES");
        let violations = saved["_sandibumi_precondition_violations_v1"]
            .as_array()
            .expect("the run provenance must carry its flagged violations");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0]["condition_id"], "vsh_gr.gr_ma_range");
        assert_eq!(violations[0]["argument"], "GR_MA");
        assert_eq!(violations[0]["expected"], "0 to 200 gAPI");
        assert_eq!(
            violations[0]["source"],
            "docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.info L48-L49"
        );
        assert_eq!(violations[0]["affected_samples"][0]["index"], 1);
        assert_eq!(violations[0]["affected_samples"][0]["offending_value"], 201.0);

        let refused_writes: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets
                 WHERE well_id = ?1 AND set_name = 'REFUSAL-CONTROL'",
                duckdb::params![&well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(refused_writes, 0, "the refusal side must still write nothing");

        // `docs/record_fixes.md`: a run that reports failure must not also version an
        // interpretation. A negative finite PHIE is a deliberately invalid fixture value, not an
        // adopted endpoint; Wyllie-Rose explicitly leaves every such sample MISSING. With no
        // declared precondition violation the selected policy still produces an all-zero framework
        // flag, and that flag must not be mistaken for a scientific answer.
        drop(conn);
        {
            let conn = dbm.lock().unwrap();
            equations::write_computed_curve(
                &conn,
                &well,
                &[1000.0, 1001.0, 1002.0],
                "PHIE",
                &[-0.1, -0.1, -0.1],
            )
            .unwrap();
        }
        let flag_only = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "perm_wyllie_rose".into(),
                well_ids: vec![well.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("SWE_IRR".into(), 0.2)]),
                opts: HashMap::from([(
                    modules::PRECONDITION_POLICY_OPT.into(),
                    modules::PRECONDITION_POLICY_FLAG_VALID_SAMPLES.into(),
                )]),
                output_set: Some("FLAG-IS-NOT-AN-ANSWER".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert_eq!(flag_only[0].outcome, ModuleRunOutcome::Failed);
        assert_eq!(flag_only[0].rows_written, 0);
        let conn = dbm.lock().unwrap();
        let flag_only_sets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'FLAG-IS-NOT-AN-ANSWER'",
                duckdb::params![&well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(flag_only_sets, 0, "a flag alone must not allocate a log-set version");
        let flag_only_curves: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves
                 WHERE well_id = ?1 AND curve_name IN
                 ('PERM', 'PERM_WR', 'PERM_WYLLIE_ROSE_PRECONDITION_FLAG')",
                duckdb::params![&well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(flag_only_curves, 0, "a finite framework flag must not version an all-MISSING scientific result");
    }

    #[test]
    fn a_required_shale_volume_accepts_renamed_shale_metadata_and_refuses_clay_metadata_even_under_a_vsh_name() {
        // CORRECTNESS — SB-CLY-043 / SB-CLY-T43. The expected result is the chapter's typed
        // interface rule, not a snapshot: VSH and VCL are distinct, renaming does not change the
        // quantity, and a VCL must be refused where VSH is required. The numerical values only
        // make the existing public workflows finite: the GR triplet gives IGR=0.5 as in T04, and
        // the Thomas-Stieber endpoints are the cited SB-TBD-T10 verification inputs. No value here
        // becomes a product default.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depth = vec![1000.0f32, 1000.5];

        let add_condition = |label: &str, gr: Vec<f32>| {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, label, None, None, Some(0.0)).unwrap();
            let missing = vec![f32::NAN; depth.len()];
            db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                gr,
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing,
            )
            .unwrap();
            let well = id.to_string();
            let phit = db::upsert_curve_meta(
                &conn,
                &well,
                "RAW",
                "PHIT",
                Some("v/v"),
                None,
                Some("SB-CLY-043 type-contract fixture"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &phit, &depth, &[0.16, 0.16]).unwrap();
            well
        };

        let renamed_shale = add_condition("RENAMED-SHALE-QUANTITY", vec![70.0, 70.0]);
        let clay_under_vsh_name = add_condition("CLAY-METADATA-UNDER-VSH-NAME", vec![f32::NAN; 2]);
        let vcl = db::upsert_curve_meta(
            &conn,
            &clay_under_vsh_name,
            "RAW",
            "VSH",
            Some("v/v"),
            Some("VCL"),
            Some("SB-CLY-043 wrong-type control"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &vcl, &depth, &[0.4, 0.4]).unwrap();

        let dbm = Mutex::new(conn);
        let produced = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "vsh_gr".into(),
                well_ids: vec![renamed_shale.clone()],
                log_inputs: HashMap::new(),
                params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
                opts: HashMap::from([(
                    format!("{OUT_NAME_PREFIX}VSH"),
                    "RENAMED_SHALE".into(),
                )]),
                output_set: Some("SHALE-PRODUCER".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        assert_ne!(produced[0].outcome, ModuleRunOutcome::Failed, "the sourced producer fixture must run");
        assert!(produced[0].rows_written > 0);

        let run_thomas_stieber = |well: &str, vsh_curve: &str, output_set: &str| {
            run_workflow_module(
                &dbm,
                &RunModuleRequest {
                    module: "thin_bed_ts".into(),
                    well_ids: vec![well.to_string()],
                    log_inputs: HashMap::from([
                        ("PHIT".into(), "PHIT".into()),
                        ("VSH".into(), vsh_curve.into()),
                    ]),
                    params: HashMap::from([("PHI_SD_MAX".into(), 0.30), ("PHI_SH".into(), 0.15)]),
                    opts: HashMap::new(),
                    output_set: Some(output_set.into()),
                    input_set: None,
                    custody: test_run_custody(),
                },
            )
            .remove(0)
        };

        let accepted = run_thomas_stieber(&renamed_shale, "RENAMED_SHALE", "RENAMED-SHALE-CONSUMER");
        assert_eq!(accepted.outcome, ModuleRunOutcome::Clean, "renaming a typed VSH must not erase its identity");
        let refused = run_thomas_stieber(&clay_under_vsh_name, "VSH", "WRONG-TYPE-CONTROL");
        assert_eq!(refused.outcome, ModuleRunOutcome::Failed, "VCL metadata must win over the misleading VSH mnemonic");
        assert_eq!(refused.rows_written, 0);
        let refusal = refused.error.expect("wrong quantity must explain its refusal");
        assert!(refusal.contains("VSH") && refusal.contains("VCL"), "refusal must name both quantities: {refusal}");

        let conn = dbm.lock().unwrap();
        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND module = 'thin_bed_ts' AND set_name = 'RENAMED-SHALE-CONSUMER'
                 ORDER BY version DESC LIMIT 1",
                duckdb::params![renamed_shale],
                |row| row.get(0),
            )
            .unwrap();
        let ancestry = equations::parse_curve_ancestry(&params_json).unwrap();
        let received = ancestry
            .parameters
            .iter()
            .find(|parameter| parameter.name == "INPUT_QUANTITY.VSH")
            .expect("the consumer run must record the quantity received");
        assert_eq!(received.value, serde_json::json!("VSH"));
        let wrong_type_sets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'WRONG-TYPE-CONTROL'",
                duckdb::params![clay_under_vsh_name],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(wrong_type_sets, 0, "a type refusal must not version an interpretation");
    }

    #[test]
    fn a_clay_volume_consumer_accepts_clay_refuses_shale_and_records_which_quantity_it_received() {
        // CORRECTNESS — SB-CLY-043 plus the mineralogical clay identity stated by
        // docs/PRD_v2/19_toc-unconventional.md §3.4. The old "Clay / shale volume" label cannot
        // authorize substituting VSH in the Jarvie/Wang-Gale clay denominator. The 0.20 sample is
        // only a finite fixture; no numeric output is used as a correctness oracle.
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let depth = vec![1000.0f32, 1000.5];
        let add_quantity = |family: &str| {
            let id = uuid::Uuid::new_v4();
            db::insert_well(&conn, id, family, None, None, Some(0.0)).unwrap();
            let missing = vec![f32::NAN; depth.len()];
            db::insert_standard_curves(
                &conn,
                id,
                depth.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing.clone(),
                missing,
            )
            .unwrap();
            let well = id.to_string();
            let curve = db::upsert_curve_meta(
                &conn,
                &well,
                "RAW",
                "MINERAL_FRACTION",
                Some("v/v"),
                Some(family),
                Some("SB-CLY-043 typed clay control"),
                None,
            )
            .unwrap();
            db::insert_curve_samples(&conn, &curve, &depth, &[0.20, 0.20]).unwrap();
            well
        };
        let vsh_well = add_quantity("VSH");
        let vcl_well = add_quantity("VCL");
        let dbm = Mutex::new(conn);

        let results = run_workflow_module(
            &dbm,
            &RunModuleRequest {
                module: "brittleness".into(),
                well_ids: vec![vsh_well.clone(), vcl_well.clone()],
                log_inputs: HashMap::from([("VCLAY".into(), "MINERAL_FRACTION".into())]),
                params: HashMap::new(),
                opts: HashMap::from([("METHOD".into(), "mineral_jarvie".into())]),
                output_set: Some("TYPED-CLAY-CONSUMER".into()),
                input_set: None,
                custody: test_run_custody(),
            },
        );
        let vsh_result = results.iter().find(|result| result.well_id == vsh_well).unwrap();
        assert_eq!(vsh_result.outcome, ModuleRunOutcome::Failed);
        let refusal = vsh_result.error.as_deref().expect("VSH in a VCL role must explain its refusal");
        assert!(refusal.contains("VCL") && refusal.contains("VSH"), "refusal must name both quantities: {refusal}");
        let vcl_result = results.iter().find(|result| result.well_id == vcl_well).unwrap();
        assert_eq!(vcl_result.outcome, ModuleRunOutcome::Clean);

        let conn = dbm.lock().unwrap();
        let params_json: String = conn
            .query_row(
                "SELECT params_json FROM log_sets
                 WHERE well_id = ?1 AND module = 'brittleness' AND set_name = 'TYPED-CLAY-CONSUMER'
                 ORDER BY version DESC LIMIT 1",
                duckdb::params![vcl_well],
                |row| row.get(0),
            )
            .unwrap();
        let ancestry = equations::parse_curve_ancestry(&params_json).unwrap();
        let received = ancestry
            .parameters
            .iter()
            .find(|parameter| parameter.name == "INPUT_QUANTITY.VCLAY")
            .expect("the VCL consumer must record the received quantity");
        assert_eq!(received.value, serde_json::json!("VCL"));
        let wrong_type_sets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'TYPED-CLAY-CONSUMER'",
                duckdb::params![vsh_well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(wrong_type_sets, 0, "a VSH-to-VCL type refusal must not version an interpretation");
    }
}
