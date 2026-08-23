//! Workflow runner: executes deterministic modules across wells (rayon-parallel),
//! resolving interval parameters per zone (interval-parameter style), and the cutoff/summary
//! engine modeled on pay-summary specs.

use crate::db;
use crate::ancestry;
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::{Connection, OptionalExt};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    /// Explicit operator and source/reference note. The operator is entered once per frontend
    /// session and attached to every run; it is never inferred from the Windows account.
    pub custody: ancestry::RunCustody,
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
pub(crate) fn test_run_custody() -> ancestry::RunCustody {
    ancestry::RunCustody {
        actor: ancestry::AncestryActor {
            kind: ancestry::AncestryActorKind::Human,
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

/// The run's output-name prefix, read the one way.
///
/// Trimmed, empty means none, and UPPERCASED - a stored curve name is upper case, so a prefix
/// typed in lower case must not produce a curve the catalog resolves differently from the one the
/// run declared. Eight sites used to state this rule for themselves, against `class_output_names`'
/// own claim to read the name "from the same two places rather than restating either".
pub(crate) fn output_prefix(opts: &HashMap<String, String>) -> String {
    opts.get(OUT_PREFIX_OPT)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .unwrap_or_default()
}

/// A resolved output name carrying the run's prefix. Empty prefix means the name is unchanged, so
/// this is the identity on every run that did not ask for one - which is every existing run and
/// every saved chain.
pub(crate) fn prefixed_output(opts: &HashMap<String, String>, name: &str) -> String {
    format!("{}{name}", output_prefix(opts))
}

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
        "sw_imts" => &["S_FACTOR_GW"],
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
    let prefix = output_prefix(opts);
    modules::class_outputs(module)
        .iter()
        .filter_map(|key| out_names.iter().find(|(declared, _)| declared == key).map(|(_, n)| n.clone()))
        .map(|n| format!("{prefix}{n}"))
        .collect()
}

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
    let prefix = output_prefix(opts);
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
    let prefix = output_prefix(opts);
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
    let prefix = output_prefix(opts);
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
/// AUDIT-2026-08-20 finding 54. ONE reserved-provenance-key guard.
///
/// This was written out nine times in three different shapes, and only two copies also consulted
/// `legacy` - which read as a decision nobody could check. It is neither a decision nor a hole:
/// [`effective_module_parameters`] puts every declared argument into BOTH maps, `legacy` under the
/// bare name and `parameters` under `name_prefix + name`, and `complete_module_log_spec` passes an
/// EMPTY prefix - so today the two lookups are the same lookup. `legacy` is nonetheless the one
/// that stays correct if a prefix is ever passed, because a reserved key is always a bare name.
/// So every site now checks both, and the question stops existing.
///
/// `kind` is the adjective the message already carried; the wording is unchanged at every site,
/// because these strings are what a user reads when a module cannot be saved.
fn reject_reserved_key(
    parameters: &[ancestry::AncestryParameter],
    legacy: &serde_json::Map<String, serde_json::Value>,
    module: &str,
    kind: &str,
    name: &str,
) -> Result<(), String> {
    if parameters.iter().any(|parameter| parameter.name == name) || legacy.contains_key(name) {
        return Err(format!(
            "module '{module}' declares an argument that collides with reserved {kind} key '{name}'"
        ));
    }
    Ok(())
}

pub(crate) fn effective_module_parameters(
    spec: &modules::ModuleSpec,
    explicit_params: &HashMap<String, f64>,
    explicit_opts: &HashMap<String, String>,
    effective_opts: &HashMap<String, String>,
    source_note: &str,
    name_prefix: &str,
) -> Result<
    (
        Vec<ancestry::AncestryParameter>,
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
                    Some(ancestry::ParameterResolution::Explicit),
                    None,
                    custody,
                )
            } else if let Ok(value) = arg.default.parse::<f64>() {
                (
                    serde_json::json!(value),
                    arg.default_source.clone(),
                    Some(ancestry::ParameterResolution::Defaulted),
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
        parameters.push(ancestry::AncestryParameter {
            name: format!("{name_prefix}{}", arg.name),
            value,
            source: source.clone(),
            resolution,
            manifest_version: value_manifest_version,
            decision,
        });
        if let Some(custody) = unit_custody {
            parameters.push(ancestry::AncestryParameter {
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
            parameters.push(ancestry::AncestryParameter {
                name: format!("{name_prefix}{}", arg.name),
                value: serde_json::json!(value),
                source: if explicit {
                    source_note.to_string()
                } else {
                    manifest_source.clone()
                },
                resolution: Some(if explicit {
                    ancestry::ParameterResolution::Explicit
                } else {
                    ancestry::ParameterResolution::Defaulted
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
    // AUDIT-2026-08-20 finding 50(b): the identity comes from the ONE production registry, so
    // this test view cannot answer differently from the run it is a view of. It used to hold a
    // private copy that skipped `canonical_option_value`, which is a second answer for `sw_sim`.
    let method_id = saturation_method_id(&req.module, opts);
    if let Some(id) = method_id.as_deref() {
        for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Option) {
            if let Some(value) = opts.get(&arg.name) {
                recorded.insert(arg.name.clone(), serde_json::json!(value));
            }
        }
        recorded.insert("method_id".into(), serde_json::json!(id));
    }
    serde_json::Value::Object(recorded).to_string()
}

// The test seam for `serialize_module_parameters`, below. A plain comment rather than a doc
// comment: rustdoc carries nothing from a macro invocation, so `///` here is an unused doc.
#[cfg(test)]
thread_local! {
    static FORCED_PARAMETER_SERIALIZATION_FAILURE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

/// Fails `serialize_module_parameters` for as long as the returned guard lives, on THIS thread.
#[cfg(test)]
pub(crate) struct ForcedParameterSerializationFailure;

#[cfg(test)]
impl ForcedParameterSerializationFailure {
    pub(crate) const MESSAGE: &'static str = "injected parameter serialization failure";

    pub(crate) fn arm() -> Self {
        FORCED_PARAMETER_SERIALIZATION_FAILURE.with(|forced| forced.set(true));
        Self
    }
}

#[cfg(test)]
impl Drop for ForcedParameterSerializationFailure {
    fn drop(&mut self) {
        FORCED_PARAMETER_SERIALIZATION_FAILURE.with(|forced| forced.set(false));
    }
}

/// The one production serialization of a run's legacy parameter map.
///
/// `serde_json::to_value` of a map that already deserialized cannot fail, so the failure this
/// guards is not otherwise producible - and it has to be proved, because a run that cannot record
/// its parameters must abort rather than version an interpretation it cannot describe. That proof
/// used to be a generic closure threaded through THREE production signatures, one of them public,
/// for the sake of a single test. The seam is a thread-local now: production carries no parameter
/// at all, and a test can only ever affect its own thread, which matters in a suite this parallel.
fn serialize_module_parameters(
    parameters: &serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    #[cfg(test)]
    {
        if FORCED_PARAMETER_SERIALIZATION_FAILURE.with(std::cell::Cell::get) {
            return Err(ForcedParameterSerializationFailure::MESSAGE.to_string());
        }
    }
    serde_json::to_value(parameters).map_err(|error| error.to_string())
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
) -> Result<ancestry::CompleteLogSetSpec, String> {
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
    reject_reserved_key(&parameters, &legacy, &spec.name, "run-provenance", MASK_PROVENANCE_KEY)?;
    let mask_is_applied = mask["state"] == MASK_PROVENANCE_APPLIED;
    parameters.push(ancestry::AncestryParameter {
        name: MASK_PROVENANCE_KEY.into(),
        value: mask,
        source: if mask_is_applied {
            req.custody.source_note.clone()
        } else {
            "SB-ENV-028 explicit no-mask run state".into()
        },
        resolution: mask_is_applied.then_some(ancestry::ParameterResolution::Explicit),
        manifest_version: None,
        decision: None,
    });
    if modules::runner_declarations(&req.module).records_smoothing_policy {
        reject_reserved_key(
            &parameters,
            &legacy,
            &spec.name,
            "smoothing-provenance",
            SMOOTHING_POLICY_PROVENANCE_KEY,
        )?;
        let policy = crate::condition::smoothing_policy(
            opts.get("OPT_METHOD").map(String::as_str).unwrap_or("MEAN"),
        );
        legacy.insert(SMOOTHING_POLICY_PROVENANCE_KEY.into(), policy.clone());
        parameters.push(ancestry::AncestryParameter {
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
        reject_reserved_key(&parameters, &legacy, &spec.name, "flag-kind provenance", &name)?;
        parameters.push(ancestry::AncestryParameter {
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
        reject_reserved_key(&parameters, &legacy, &spec.name, "output-quantity provenance", &name)?;
        parameters.push(ancestry::AncestryParameter {
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
        reject_reserved_key(&parameters, &legacy, &spec.name, "porosity-output provenance", &name)?;
        parameters.push(ancestry::AncestryParameter {
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
            parameters.push(ancestry::AncestryParameter {
                name: format!("{}@{}", arg.name, zone_value.zone_name),
                value: serde_json::json!(value),
                source: source.to_string(),
                resolution: Some(ancestry::ParameterResolution::Explicit),
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
        parameters.push(ancestry::AncestryParameter {
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
                            Some(ancestry::ParameterResolution::Defaulted) => "SHIPPED_DEFAULT",
                            Some(ancestry::ParameterResolution::Explicit) => "ENTERED",
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
            reject_reserved_key(&parameters, &legacy, &spec.name, "saturation-provenance", name)?;
            parameters.push(ancestry::AncestryParameter {
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
            reject_reserved_key(&parameters, &legacy, &spec.name, "saved-run", key)?;
            legacy.insert(key.into(), value);
        }
        parameters.push(ancestry::AncestryParameter {
            name: PRECONDITION_POLICY_PROVENANCE_KEY.into(),
            value: policy,
            source: req.custody.source_note.clone(),
            resolution: Some(ancestry::ParameterResolution::Explicit),
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
            parameters.push(ancestry::AncestryParameter {
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
        match ancestry::resolve_ancestry_input(
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
                    let quantity = checked_shale_clay_quantity(
                        arg,
                        shale_clay_quantity_for_ancestry_input(conn, &input)?,
                        &spec.name,
                        argument,
                        QuantityOrigin {
                            curve_phrase: &format!("resolved curve '{}'", input.curve),
                            missing_advice: "",
                        },
                    )?;
                    let name = format!("{INPUT_QUANTITY_PROVENANCE_PREFIX}{argument}");
                    reject_reserved_key(
                        &parameters,
                        &legacy,
                        &spec.name,
                        "input-quantity provenance",
                        &name,
                    )?;
                    parameters.push(shale_clay_quantity_parameter(name, argument, quantity)?);
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
        ancestry::AncestryZoneScope::WholeWell
    } else {
        ancestry::AncestryZoneScope::Defined(
            zones
                .into_iter()
                .map(|zone| ancestry::AncestryZone {
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
        .map(|curve| ancestry::AncestryOutput {
            curve: curve.clone(),
            derivation: format!("{}:{curve}", req.module),
        })
        .collect();
    let parameter_state = ancestry::parameter_state_for(&parameters);
    // SB-DBM-015 (DEC-023): the zone-set identity the run sees, recorded whenever zones
    // exist - a renamed or moved top changes it, and the re-run resolver refuses by name.
    let zone_set = match &zone_scope {
        ancestry::AncestryZoneScope::WholeWell => None,
        _ => {
            let (version, digest) =
                db::current_zone_set(conn, well_id).map_err(|error| error.to_string())?;
            Some(ancestry::ManifestZoneSet { version, digest })
        }
    };
    let ancestry = ancestry::CurveAncestry {
        schema_version: ancestry::CURVE_ANCESTRY_SCHEMA_VERSION,
        method_derivation: ancestry::method_derivation_citation(&req.module),
        module: req.module.clone(),
        // SB-DBM-002 (DEC-021): the producing code's own digest, not the hand-maintained
        // package version that does not move when a module's arithmetic does.
        module_version: format!("src:{}", modules::module_source_digest(&req.module)),
        inputs,
        parameters,
        parameter_state,
        zone_scope,
        actor: req.custody.actor.clone(),
        timestamp_utc_ms: ancestry::ancestry_timestamp_utc_ms()?,
        outputs,
        depth_frame: None,
        zone_set,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    };
    let validity_manifest = serde_json::to_value(modules::module_validity_manifest(spec))
        .map_err(|error| format!("cannot serialize module validity manifest: {error}"))?;
    // `parameters` has moved into `ancestry` by here, which is why this site could only ever
    // consult one map - not a divergence, a consequence. The vector is the same one, so the guard
    // reads it back off the struct and stays identical to the other eight.
    reject_reserved_key(
        &ancestry.parameters,
        &legacy,
        &spec.name,
        "saved-run",
        modules::MODULE_VALIDITY_MANIFEST_KEY,
    )?;
    legacy.insert(modules::MODULE_VALIDITY_MANIFEST_KEY.into(), validity_manifest);
    let legacy = serialize_module_parameters(&legacy)
        .map_err(|error| format!("cannot serialize module parameters: {error}"))?;
    ancestry::CompleteLogSetSpec::try_new_with_legacy(
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
    custody: &ancestry::RunCustody,
) -> Result<RerunReport, String> {
    let (module, ancestry, stored_params) = {
        let conn = db.lock().map_err(|_| "database busy".to_string())?;
        let entry = ancestry::list_log_sets(&conn, well_id)
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
            let now = ancestry::resolve_ancestry_input(
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
            if name == ancestry::CURVE_ANCESTRY_KEY
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
    let rerun_set = ancestry::list_log_sets(&conn, well_id)
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

pub(crate) fn first_available_input_alias(
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
        if ancestry::try_resolve_ancestry_input(
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
                modules::ValidityRule::LessThan { other }
                | modules::ValidityRule::NotAbove { other } => {
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
/// How the caller found the curve whose quantity metadata is being checked, in the words the
/// refusal will use. "chain-produced curve 'VSH1'" and "resolved curve 'VSH1'" send a user to two
/// different places to fix the same complaint, and only the second can be fixed by assigning a
/// family - a curve a chain produced a moment ago takes its quantity from the module that made it,
/// so telling its user to go and assign one would be advice they cannot act on.
pub(crate) struct QuantityOrigin<'a> {
    pub(crate) curve_phrase: &'a str,
    pub(crate) missing_advice: &'a str,
}

/// AUDIT-2026-08-20 finding 76: the SB-CLY-043 input-quantity contract, checked in ONE place.
///
/// A clay-volume consumer declares which quantity it accepts - VSH (shale) or VCL (clay) - and a
/// curve carrying the other one is a DIFFERENT physical quantity under a compatible-looking
/// mnemonic. This was written out FOUR times: twice in `chain.rs` (once for a curve the chain had
/// just produced, once for one resolved out of the project) and twice here (the run's ancestry
/// build and the pre-flight validation). They differed only in where `actual` came from and in
/// what the refusal called the curve. Four copies is four places for an accepted list to be
/// widened in three, and the widening is silent: the wrong quantity computes, plots and ships.
pub(crate) fn checked_shale_clay_quantity(
    contract: &modules::ArgSpec,
    actual: Option<modules::ShaleClayQuantity>,
    module: &str,
    arg_name: &str,
    origin: QuantityOrigin<'_>,
) -> Result<modules::ShaleClayQuantity, String> {
    let QuantityOrigin {
        curve_phrase,
        missing_advice,
    } = origin;
    let accepted = contract
        .accepted_shale_clay_quantities
        .iter()
        .map(|quantity| quantity.as_str())
        .collect::<Vec<_>>()
        .join(" or ");
    let actual = actual.ok_or_else(|| {
        format!(
            "module '{module}' input '{arg_name}' requires typed {accepted} metadata, but {curve_phrase} has no VSH/VCL quantity metadata{missing_advice}"
        )
    })?;
    if !contract.accepted_shale_clay_quantities.contains(&actual) {
        return Err(format!(
            "module '{module}' input '{arg_name}' requires {accepted}, but {curve_phrase} carries {} metadata",
            actual.as_str()
        ));
    }
    Ok(actual)
}

/// The provenance row recording WHICH quantity a run accepted for one input. Separate from the
/// check above because the pre-flight validation refuses without recording anything, while the
/// three producing paths record - and one of them has to clear the reserved-key guard between the
/// two steps, so they cannot be one call.
pub(crate) fn shale_clay_quantity_parameter(
    name: String,
    arg_name: &str,
    quantity: modules::ShaleClayQuantity,
) -> Result<ancestry::AncestryParameter, String> {
    Ok(ancestry::AncestryParameter {
        name,
        value: serde_json::to_value(quantity)
            .map_err(|error| format!("cannot serialize input quantity for {arg_name}: {error}"))?,
        source: "docs/PRD_v2/10_clay-volume.md SB-CLY-043".into(),
        resolution: None,
        manifest_version: None,
        decision: None,
    })
}

pub(crate) fn shale_clay_quantity_for_ancestry_input(
    conn: &Connection,
    input: &ancestry::AncestryInput,
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
        let ancestry = ancestry::parse_curve_ancestry(&params_json).map_err(|error| {
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
                "computed input '{}' carries duplicate quantity metadata. Two records claim \
                 the same parameter with different units, so which one the run should honour \
                 cannot be decided. Re-run the module that produced this curve so its \
                 metadata is written once.",
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
        let Some(input) = ancestry::try_resolve_ancestry_input(
            conn,
            well_id,
            &argument.name,
            curve,
            input_set,
            own_set_id,
        )? else {
            continue;
        };
        // Refuses without recording: this pass runs before the ancestry record exists.
        checked_shale_clay_quantity(
            argument,
            shale_clay_quantity_for_ancestry_input(conn, &input)?,
            &spec.name,
            &argument.name,
            QuantityOrigin {
                curve_phrase: &format!("resolved curve '{}'", input.curve),
                missing_advice: "; assign the physical family explicitly instead of relying on its mnemonic",
            },
        )?;
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
            "module '{}' refuses: neutron curve '{curve}' has no DECLARED matrix basis. A limestone-unit neutron read against a sandstone matrix is ~0.04 v/v low in clean water sand, and an undeclared basis cannot be checked - declare it (set_curve_neutron_basis) or convert with nphimat first. DEC-025 / SB-POR-024",
            spec.name
        ));
    };
    if let Some(entry) = required_entry {
        if !declared.eq_ignore_ascii_case(entry) {
            return Err(format!(
                "module '{}' refuses: its crossplot is entered in {entry} units, but neutron curve '{curve}' declares basis {declared} - convert with nphimat first. DEC-025 / SB-POR-024",
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

pub(crate) fn fetch_mask_aligned(
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

pub(crate) fn apply_mask_to_logs(
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

/// Blanks flagged samples in a step's OUTPUTS, so a flagged depth's result is never trusted
/// downstream. The sibling of [`apply_mask_to_logs`], and like it shared by the deterministic
/// runner and the Monte Carlo engine.
///
/// AUDIT-2026-08-20 finding 12. This lived inline in `run_workflow_module`, and the Monte Carlo
/// engine carried each step's MASK and never read it - so a washout was interpreted as rock in
/// EVERY realization, which is the direction that adds pay, and a batch study is what gets
/// quoted. Extracted rather than copied because the pin test that disclosed the gap says exactly
/// why: whoever fixes it must extend BOTH, or the mask silently blanks nothing.
///
/// The two exemptions are passed in BY NAME rather than derived here, because the two engines
/// name their outputs differently - the deterministic runner has resolved names and prefixes by
/// this point, the Monte Carlo engine's `run_module` returns DECLARED keys. The rule is shared;
/// resolving the names stays with whoever knows them.
pub(crate) fn apply_mask_to_outputs(
    outputs: &mut HashMap<String, Vec<f32>>,
    mask: &[f32],
    repair_exempt_output: Option<&str>,
    cly_prov_output: Option<&str>,
) {
    for (name, values) in outputs.iter_mut() {
        // DEC-033: the one declared repair output, whose finite values at masked depths are the
        // module's whole purpose. A repair blanked at this pass is a repair that did not happen.
        if repair_exempt_output == Some(name.as_str()) {
            continue;
        }
        if cly_prov_output == Some(name.as_str()) {
            // SB-CLY-001: a masked sample's token is the mask's own statement, written HERE
            // where the mask is known - blanking it would erase the one record of WHY the
            // sample has no computed value.
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
    preset_sets: Option<&HashMap<String, ancestry::CompleteSetId>>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    progress: Option<&crate::jobs::JobHandle>,
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

    /// AUDIT-2026-08-20 finding 50(a): what one module run produced, before it becomes an
    /// [`Outcome`].
    ///
    /// This was an anonymous SEVEN-element tuple, restated three times: a type list on the
    /// closure's signature, a positional construction at its end, and a positional destructuring
    /// at the call. The seven types happen to be distinct today, so the compiler does catch a
    /// swapped pair - but that is an accident of the current field list, not a property of the
    /// shape, and it stops holding the moment a second `Option<String>` or a second `bool` joins
    /// it. Named, both ends of the hand-off are checked by FIELD in either direction, and adding
    /// a field to one list without the other is a compile error rather than a reading exercise.
    struct ComputedRun {
        depth: Vec<f32>,
        outputs: HashMap<String, Vec<f32>>,
        log_args: Vec<(String, String)>,
        degradations: Vec<modules::RunDegradation>,
        precondition_violations: Vec<modules::PreconditionViolation>,
        scientific_answered: bool,
        badhole_record: Option<String>,
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
            let compute = || -> Result<ComputedRun, String> {
                #[cfg(test)]
                let _phase_well = crate::lock_probe::well();
                // A chain's own set event: its earlier steps' outputs beat the input set.
                let own_set = preset_sets.and_then(|m| m.get(well_id.as_str())).map(|s| s.as_str());
                let (depth, mut logs, input_units, params, defaulted_parameters, log_args) = {
                    #[cfg(test)]
                    let conn = { let _phase_wait = crate::lock_probe::wait(); db.lock().unwrap() };
                    #[cfg(not(test))]
                    let conn = db.lock().unwrap();
                    #[cfg(test)]
                    let _phase_read = crate::lock_probe::read();
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
                    #[cfg(test)]
                    let conn = { let _phase_wait = crate::lock_probe::wait(); db.lock().unwrap() };
                    #[cfg(not(test))]
                    let conn = db.lock().unwrap();
                    #[cfg(test)]
                    let _phase_read = crate::lock_probe::read();
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
                let declared = modules::runner_declarations(&req.module);
                let repair_run = declared.mask_repair.is_some_and(|repair| {
                    req.opts
                        .get(repair.option)
                        .map(|mode| mode.trim() == repair.value)
                        .unwrap_or(false)
                });

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
                if declared.reads_input_neutron_basis {
                    #[cfg(test)]
                    let conn = { let _phase_wait = crate::lock_probe::wait(); db.lock().unwrap() };
                    #[cfg(not(test))]
                    let conn = db.lock().unwrap();
                    #[cfg(test)]
                    let _phase_read = crate::lock_probe::read();
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
                let prefix = output_prefix(&opts);
                if !prefix.is_empty() {
                    outputs = outputs
                        .into_iter()
                        .map(|(name, values)| (format!("{prefix}{name}"), values))
                        .collect();
                }

                // SB-ENV-027 (DEC-033): resolve the declared repair output's STORED name
                // (rename + prefix applied, exactly as the map above composed it).
                let repair_exempt_output: Option<String> = declared
                    .mask_repair
                    .filter(|_| repair_run)
                    .and_then(|repair| {
                        out_names
                            .iter()
                            .find(|(name, _)| name == repair.output)
                            .map(|(_, resolved)| prefixed_output(&opts, resolved))
                    });

                // SB-CLY-001 (DEC-036): resolve the CLY provenance output's STORED name
                // (rename + prefix, exactly as the outputs map composed it) so the mask pass
                // below can WRITE the masked/disabled token instead of blanking it, and the
                // zone-bearing message can read the final tokens.
                let cly_prov_output: Option<String> = declared
                    .provenance_output
                    .and_then(|provenance| {
                        out_names
                            .iter()
                            .find(|(name, _)| name == provenance)
                            .map(|(_, resolved)| prefixed_output(&opts, resolved))
                    });

                // Blank flagged samples in the OUTPUTS too, so a flagged depth's result is
                // never trusted downstream - EXCEPT the one declared repair output, whose
                // finite values at masked depths are the module's whole purpose.
                if let Some(mask) = &mask {
                    apply_mask_to_outputs(
                        &mut outputs,
                        mask,
                        repair_exempt_output.as_deref(),
                        cly_prov_output.as_deref(),
                    );
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
                                #[cfg(test)]
                                let _phase_read = crate::lock_probe::read();
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
                    let name = prefixed_output(&opts, &base);
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
                Ok(ComputedRun {
                    depth,
                    outputs,
                    log_args,
                    degradations,
                    precondition_violations,
                    scientific_answered,
                    badhole_record,
                })
            };

            let outcome = match compute() {
                Ok(ComputedRun {
                    depth,
                    outputs,
                    log_args,
                    degradations,
                    precondition_violations,
                    scientific_answered,
                    badhole_record,
                }) => Outcome::Computed {
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
    let set_ids: HashMap<String, ancestry::CompleteSetId> = if succ_ids.is_empty() {
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
            ) {
                Ok(mut spec) => {
                    // SB-DBM-015: the arms the spec builder cannot know - the depth frame
                    // exists only after the fetch, and the physics-driving attribute value
                    // is the one the runner injected (same helper, so record and injection
                    // cannot drift).
                    let Outcome::Computed { depth, .. } = outcome else { unreachable!() };
                    let frame = (!depth.is_empty()).then(|| ancestry::ManifestDepthFrame {
                        top: depth[0],
                        base: depth[depth.len() - 1],
                        samples: depth.len(),
                    });
                    let physics = if modules::runner_declarations(&req.module)
                        .reads_input_neutron_basis
                        || modules::required_neutron_basis(&req.module).is_some()
                    {
                        nphimat_declared_basis(&conn, well_id, log_args)
                            .map(|value| {
                                vec![ancestry::PhysicsAttribute {
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
                    complete.push(ancestry::CompleteWellLogSet {
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
            || ancestry::create_complete_log_sets_batch(&conn, &complete),
            |error| Err(error),
        ) {
            Ok(m) => m,
            Err(error) => {
                set_err = Some(error);
                HashMap::new()
            }
        }
    };

    let mut writes: Vec<ancestry::CompleteWellWrite> = Vec::with_capacity(succ_ids.len());
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
                writes.push(ancestry::CompleteWellWrite {
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
                    let _ = ancestry::set_log_set_comment(&conn, set_id.as_str(), record);
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
        #[cfg(test)]
        let _phase_write = crate::lock_probe::write();
        let err = ancestry::write_computed_curves_with_ancestry_batch(&conn, &writes).err();
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

#[cfg(test)]
mod tests {
    use super::*;
    // AUDIT-2026-08-20 finding 49: four tests here exercise a module run AND the
    // pay summary that consumes it, which is the seam itself and not a filing error.
    use crate::paysummary::{run_pay_summary, CutoffEntry, DiscretisationModel, PaySummaryRequest};
    use crate::ingest;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    /// AUDIT-2026-08-20 finding 50(b). The generic runner had grown four - measured five -
    /// special cases reached by matching a module NAME on the run path, which is the one dispatch
    /// the module framework exists to make unnecessary: a module ships a manifest, the runner
    /// reads it, and nothing in the run path should know that `nphimat` is the module needing its
    /// input's neutron basis. Each literal is also a silent trap for a rename, because a module
    /// renamed in its manifest keeps running here under the old name and quietly loses its
    /// special case.
    ///
    /// The needle is a comparison of THIS RUN's module against a literal. A named registry keyed
    /// on a `module: &str` parameter - `saturation_method_id`, `lrlc_calibration_coefficients`,
    /// `modules::required_neutron_basis` - is the declaration this finding asks for, already
    /// outside the run path, and is deliberately not counted.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. Deleting all
    /// five behaviours would empty the run path of module names and break DEC-025, DEC-033,
    /// DEC-036 and SB-ENV-041 while doing it; a declaration table full of misspelled curve names
    /// would keep the runner clean and declare nothing that resolves. So: the run path matches no
    /// module name, AND every arm of the table names a real module whose manifest really declares
    /// the output and option that arm cites.
    #[test]
    fn the_runner_matches_no_module_name_and_every_declaration_resolves_against_a_real_manifest() {
        // Production half only, needles assembled, comment lines dropped - so this test is never
        // an occurrence of what it counts, and the prose explaining a decision is not the decision.
        // Line endings normalised: these files are CRLF on disk, so a boundary spelled with
        // a bare newline silently matches nothing and the slice runs to the end of the file.
        let runner = include_str!("workflow.rs").replace("\r\n", "\n");
        let production = runner
            .split("
mod tests")
            .next()
            .expect("a split always yields one piece");
        // Against a LITERAL. Holding one entry's module against another's is an ordinary
        // comparison; naming a module in the runner's own source is the dispatch being removed.
        let by_name = [".module", " == \""].concat();
        let by_match = [".module", ".as_str()"].concat();
        let matched: Vec<&str> = production
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter(|line| line.contains(by_name.as_str()) || line.contains(by_match.as_str()))
            .collect();
        assert!(
            matched.is_empty(),
            "the runner must reach a module's special case through modules::runner_declarations, never by matching its name: {matched:?}",
        );

        // Every arm of the declaration table, read off the table's own source rather than a second
        // list that could drift from it.
        let manifests = include_str!("modules.rs").replace("\r\n", "\n");
        let table = manifests
            .split(["fn runner_dec", "larations("].concat().as_str())
            .nth(1)
            .expect("the declaration table exists")
            // Bounded by the blank line AFTER the function - the table itself has none. A
            // brace-free boundary on purpose: an unbalanced brace in a test's own source ends
            // core_ancestry_tests' cfg(test) skip early and leaks this whole module into its
            // production scan, which is what a needle spelling the closing brace did here.
            .split("

")
            .next()
            .expect("a split always yields one piece");
        let declared_modules: Vec<&str> = table
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter_map(|line| line.strip_prefix('"'))
            .filter_map(|rest| rest.split_once('"'))
            .filter(|(_, after)| after.trim_start().starts_with("=>"))
            .map(|(name, _)| name)
            .collect();
        assert_eq!(
            declared_modules.len(),
            4,
            "the five run-path behaviours are declared by four modules; a change here is a DECISION about DEC-025, DEC-033, DEC-036 or SB-ENV-041, not a refactor: {declared_modules:?}",
        );

        let specs = modules::list_modules();
        for module in declared_modules {
            let spec = specs
                .iter()
                .find(|spec| spec.name == module)
                .unwrap_or_else(|| panic!("{module} is declared for but is not a module; a renamed module silently loses its special case"));
            let names = |kind: modules::ArgKind| -> Vec<&str> {
                spec.args
                    .iter()
                    .filter(|arg| arg.kind == kind)
                    .map(|arg| arg.name.as_str())
                    .collect()
            };
            let declared = modules::runner_declarations(module);
            if let Some(repair) = declared.mask_repair {
                let outputs = names(modules::ArgKind::LogOut);
                assert!(
                    outputs.contains(&repair.output),
                    "{module} declares the mask repair on output {}, which its manifest does not produce: {outputs:?}",
                    repair.output,
                );
                let options = names(modules::ArgKind::Option);
                assert!(
                    options.contains(&repair.option),
                    "{module} declares the mask repair behind option {}, which its manifest does not offer: {options:?}",
                    repair.option,
                );
            }
            if let Some(provenance) = declared.provenance_output {
                let outputs = names(modules::ArgKind::LogOut);
                assert!(
                    outputs.contains(&provenance),
                    "{module} declares provenance on output {provenance}, which its manifest does not produce: {outputs:?}",
                );
            }
        }
    }

    /// AUDIT-2026-08-20 finding 76. The SB-CLY-043 input-quantity contract - a clay-volume
    /// consumer declares whether it accepts VSH or VCL, and the other one is a DIFFERENT physical
    /// quantity wearing a compatible-looking mnemonic - was checked in FOUR places: twice in
    /// `chain.rs` (a curve the chain had just produced, and one resolved out of the project) and
    /// twice here (the run ancestry build, and the pre-flight validation). Four copies is four
    /// places for an accepted list to be widened in three, and the widening is silent: the wrong
    /// quantity computes, plots and ships into a report.
    ///
    /// Pinned from both sides, because either half alone has a lazier way to pass. The check is
    /// written ONCE and all four callers reach it, AND the refusal still describes the curve the
    /// way the caller found it. Folding the wording into one shared string would satisfy the count
    /// and send a user chasing the wrong fix: a curve the chain produced a moment ago takes its
    /// quantity from the module that made it, so "assign the physical family" is advice they
    /// cannot act on, while for a curve resolved out of the project it is the whole answer.
    #[test]
    fn the_shale_clay_quantity_contract_is_checked_once_and_still_names_where_the_curve_came_from()
    {
        // Counted over the production half of each file, with every needle assembled, so that
        // this test is never an occurrence of what it counts.
        let here = include_str!("workflow.rs");
        let chain = include_str!("chain.rs");
        let before_tests = |source: &'static str| {
            source.split("\nmod tests").next().expect("a split always yields one piece")
        };
        let production = [before_tests(here), before_tests(chain)].concat();
        let refusal = ["has no VSH/VCL", " quantity metadata"].concat();
        assert_eq!(
            production.matches(refusal.as_str()).count(),
            1,
            "the quantity contract is one check; a second is a second accepted list",
        );
        // Scoped to the INPUT path on purpose: the same document also decides which quantity a
        // module PRODUCES, and that provenance is a separate concern with its own two writers.
        assert_eq!(
            production.matches(["cannot serialize input", " quantity for"].concat().as_str())
                .count(),
            1,
            "and one statement of the provenance row that records what was accepted",
        );
        assert_eq!(
            production.matches(["checked_shale_clay_quantity", "("].concat().as_str()).count(),
            5,
            "one declaration and the four callers that reach it",
        );
        // Two of those four found the curve somewhere a user cannot assign a family, and must
        // therefore withhold that advice: the chain-produced curve, and the run's ancestry build.
        let quote = '"';
        assert_eq!(
            production.matches(format!("missing_advice: {quote}{quote},").as_str()).count(),
            2,
            "the advice follows the origin; a caller that hands out unactionable advice is a bug",
        );

        // brittleness.VCLAY is a real SB-CLY-043 consumer: it accepts CLAY volume and nothing else.
        let catalog = modules::list_modules();
        let contract = catalog
            .iter()
            .find(|spec| spec.name == "brittleness")
            .and_then(|spec| spec.args.iter().find(|arg| arg.name == "VCLAY"))
            .expect("brittleness.VCLAY carries the clay-quantity contract");
        assert_eq!(
            contract.accepted_shale_clay_quantities,
            vec![modules::ShaleClayQuantity::ClayVolume],
        );

        let chain_produced = || QuantityOrigin {
            curve_phrase: "chain-produced curve 'VCL1'",
            missing_advice: "",
        };
        let resolved = || QuantityOrigin {
            curve_phrase: "resolved curve 'VCL1'",
            missing_advice: "; assign the physical family explicitly instead of relying on its mnemonic",
        };

        // The accepted quantity passes and is handed back for the provenance row.
        assert_eq!(
            checked_shale_clay_quantity(
                contract,
                Some(modules::ShaleClayQuantity::ClayVolume),
                "brittleness",
                "VCLAY",
                resolved(),
            )
            .expect("clay volume is what this consumer accepts"),
            modules::ShaleClayQuantity::ClayVolume,
        );

        // The other quantity is refused by name, and the refusal says which one arrived.
        let wrong = checked_shale_clay_quantity(
            contract,
            Some(modules::ShaleClayQuantity::ShaleVolume),
            "brittleness",
            "VCLAY",
            resolved(),
        )
        .expect_err("shale volume is a different physical quantity");
        assert_eq!(
            wrong,
            "module 'brittleness' input 'VCLAY' requires VCL, but resolved curve 'VCL1' carries VSH metadata",
        );

        // No metadata at all is refused rather than assumed - and the advice follows the ORIGIN.
        let unresolved = checked_shale_clay_quantity(
            contract, None, "brittleness", "VCLAY", resolved(),
        )
        .expect_err("an untyped curve is refused, never guessed from its mnemonic");
        assert!(
            unresolved.ends_with(
                "; assign the physical family explicitly instead of relying on its mnemonic"
            ),
            "a curve resolved out of the project can be fixed by assigning a family: {unresolved}",
        );
        let unresolved = checked_shale_clay_quantity(
            contract, None, "brittleness", "VCLAY", chain_produced(),
        )
        .expect_err("an untyped chain-produced curve is refused too");
        assert_eq!(
            unresolved,
            "module 'brittleness' input 'VCLAY' requires typed VCL metadata, but chain-produced curve 'VCL1' has no VSH/VCL quantity metadata",
            "a curve the chain just produced takes its quantity from the module that made it, so \
             that advice would be unactionable and is not given",
        );

        // The provenance row records WHICH quantity was accepted, and cites what decided it.
        let parameter = shale_clay_quantity_parameter(
            "step[2].INPUT_QUANTITY.VCLAY".into(),
            "VCLAY",
            modules::ShaleClayQuantity::ClayVolume,
        )
        .expect("the accepted quantity is recordable");
        assert_eq!(parameter.value, serde_json::json!("VCL"));
        assert_eq!(parameter.source, "docs/PRD_v2/10_clay-volume.md SB-CLY-043");
    }

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
            db::insert_standard_curves_as_opened_project(
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
            db::insert_standard_curves_as_opened_project(
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::curve_ancestry(&conn, &control, "VSH")
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
        db::insert_standard_curves_as_opened_project(
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
        let equation_ancestry = ancestry::parse_curve_ancestry(&equation_params).unwrap();
        assert!(equation_ancestry.parameters.is_empty(), "equation metadata is not a parameter");
        assert_eq!(
            equation_ancestry.parameter_state,
            Some(crate::schema_vocab::ProvenanceAbsentState::NotApplicable),
            "a genuine no-parameter run has the specified named state"
        );
        let equation_json: serde_json::Value = serde_json::from_str(&equation_params).unwrap();
        assert_eq!(
            equation_json[ancestry::CURVE_ANCESTRY_KEY]["parameter_state"],
            "NOT_APPLICABLE",
            "the persisted reader surface carries the state verbatim"
        );
        let mut legacy_equation_json = equation_json.clone();
        legacy_equation_json[ancestry::CURVE_ANCESTRY_KEY]["schema_version"] =
            serde_json::json!(2);
        legacy_equation_json[ancestry::CURVE_ANCESTRY_KEY]
            .as_object_mut()
            .unwrap()
            .remove("parameter_state");
        let legacy_ancestry =
            ancestry::parse_curve_ancestry(&legacy_equation_json.to_string()).unwrap();
        assert_eq!(
            legacy_ancestry.parameter_state,
            Some(crate::schema_vocab::ProvenanceAbsentState::LegacyUnrecorded),
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
        let forced = ForcedParameterSerializationFailure::arm();
        let module_result = run_workflow_module_into(&dbm, &module_request, None, None, None);
        drop(forced);
        assert_eq!(module_result.len(), 1);
        let error = module_result[0]
            .error
            .as_deref()
            .expect("a parameter serialization failure must fail the module run");
        assert!(error.contains(ForcedParameterSerializationFailure::MESSAGE), "{error}");

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
        )
        .unwrap();
        let set_id = ancestry::create_complete_log_set(&conn, &well_id, &complete)
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
        ) -> ancestry::CompleteSetId {
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
            )
            .unwrap();
            ancestry::create_complete_log_set(conn, well_id, &complete)
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
        ) -> ancestry::AncestryInput {
            let params_json: String = conn
                .query_row(
                    "SELECT params_json FROM log_sets
                     WHERE well_id = ?1 AND set_name = ?2
                     ORDER BY version DESC LIMIT 1",
                    duckdb::params![well_id, output_set],
                    |row| row.get(0),
                )
                .unwrap();
            ancestry::parse_curve_ancestry(&params_json)
                .unwrap()
                .inputs
                .into_iter()
                .find(|input| input.argument == "CURVE")
                .expect("the flip run records its CURVE input")
        }

        fn rejected_identities(
            input: &ancestry::AncestryInput,
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
            assert_eq!(input.rule, Some(ancestry::CurveResolutionRule::FinalFlag));
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
        assert_eq!(input.rule, Some(ancestry::CurveResolutionRule::FinalFlag));
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
        let mut incomplete = ancestry::parse_curve_ancestry(&params_json).unwrap();
        incomplete.inputs[0].chosen_curve_id = None;
        assert!(
            ancestry::CompleteLogSetSpec::try_new("INCOMPLETE", incomplete).is_err(),
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::parse_curve_ancestry(&params_json)
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &wells[name])
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &wells[name])
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &wells[name])
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
            db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &wells[well])
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            let entry = ancestry::list_log_sets(&conn, &well)
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
        db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &well)
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
        let comment = ancestry::list_log_sets(&conn, &id.to_string())
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
        db::insert_standard_curves_as_opened_project(
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
    /// AUDIT-2026-08-20 finding 74. A run may prefix the curve names it writes. The rule is
    /// trimmed, empty-means-none and UPPERCASED - a stored curve name is upper case, so `rev_` and
    /// `REV_` have to name the same curve or a run writes one name while the catalog looks for
    /// another. Eight sites in this file stated that rule for themselves, against
    /// `class_output_names`' own claim to read a name "from the same two places rather than
    /// restating either". Both sides: the rule, and a count that stops a ninth being written.
    #[test]
    fn one_reading_of_the_output_prefix_and_nothing_else_reads_the_option() {
        let none: HashMap<String, String> = HashMap::new();
        assert_eq!(output_prefix(&none), "", "no prefix asked for is no prefix");
        assert_eq!(prefixed_output(&none, "PHIE"), "PHIE", "and the declared name is untouched");

        for entered in ["", "   "] {
            let blank = HashMap::from([(OUT_PREFIX_OPT.to_string(), entered.to_string())]);
            assert_eq!(output_prefix(&blank), "", "a blank entry is not a prefix: {entered:?}");
            assert_eq!(prefixed_output(&blank, "PHIE"), "PHIE");
        }

        let lower = HashMap::from([(OUT_PREFIX_OPT.to_string(), "  rev_  ".to_string())]);
        let upper = HashMap::from([(OUT_PREFIX_OPT.to_string(), "REV_".to_string())]);
        assert_eq!(output_prefix(&lower), "REV_", "trimmed and upper-cased");
        assert_eq!(prefixed_output(&lower, "PHIE"), "REV_PHIE");
        assert_eq!(
            prefixed_output(&lower, "PHIE"),
            prefixed_output(&upper, "PHIE"),
            "the same prefix typed either way must name the same curve",
        );

        // The other side. The needle is assembled rather than written out, so this test is not an
        // offender against its own count.
        let needle = ["get(", "OUT_PREFIX_OPT)"].concat();
        assert_eq!(
            include_str!("workflow.rs").matches(needle.as_str()).count(),
            1,
            "the run's output prefix is read in exactly one place",
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            ancestry::list_log_sets(&conn, &well)
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
            ancestry::curve_ancestry(&conn, &well, curve)
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
        let find = |ancestry: &ancestry::CurveAncestry, name: &str| {
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
            ancestry::curve_ancestry_disclosures(&conn, &[well.clone()], Some("SW_INDO")).unwrap()
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
                    input_set: None,
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
        db::insert_standard_curves_as_opened_project(
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
                input_set: None,
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
        db::insert_standard_curves_as_opened_project(
            &conn, dead, depths.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        ).unwrap();

        // Live well: a real GR that clusters and computes a real VSH.
        let live = Uuid::new_v4();
        db::insert_well(&conn, live, "LIVE-1", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            input_set: None,
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
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let vsh = [0.1; 4];
        let phie = [0.2; 4];
        let swe = [0.3; 4];
        let perm = [f32::NAN; 4];
        let input_spec = ancestry::LogSetSpec {
            set_name: "TEST_INPUTS".into(),
            module: "test_fixture".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (input_set_id, _) = ancestry::create_log_set(&conn, &w, &input_spec).unwrap();
        ancestry::write_computed_curves_versioned(
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
            stats_only: false,
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
            stats_only: false,
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
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depths.clone(),
            vec![50.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let vsh = [0.1; 4];
        let phie = [0.2; 4];
        let swe = [0.3; 4];
        let perm = [f32::NAN; 4];
        let input_spec = ancestry::LogSetSpec {
            set_name: "TEST_INPUTS".into(),
            module: "test_fixture".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (input_set_id, _) = ancestry::create_log_set(&conn, &w, &input_spec).unwrap();
        ancestry::write_computed_curves_versioned(
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
            stats_only: true,
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
            db::insert_standard_curves_as_opened_project(
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
            input_set: None,
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
            db::insert_standard_curves_as_opened_project(
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
                input_set: None,
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
            let Ok(out) = modules::run_module(&spec.name, &ctx) else { continue;
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
            let ancestry = ancestry::curve_ancestry(conn, well_id, curve)
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
        db::insert_standard_curves_as_opened_project(
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
            ancestry::curve_ancestry(&conn, &well_id, "PHIE").is_ok(),
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
            let restored = ancestry::restore_log_set(&conn, &density_set_id).unwrap();
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
            db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            ancestry::curve_ancestry(&reopened, &well_id, curve)
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            input_set: None,
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

        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
        use crate::ancestry::{AncestryOutput, AncestryParameter, AncestryZoneScope, CompleteLogSetSpec, CurveAncestry, create_complete_log_set, restore_log_set, write_computed_curves_with_ancestry};

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "SANDI-VER", None, None, None).unwrap();
        let w = id.to_string();

        // RHOB is phi_den's other input; hold it constant so VSH is the only thing that moves.
        let n = 3usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves_as_opened_project(
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

        let gr_input = ancestry::resolve_ancestry_input(&conn, &w, "GR", "GR", None, None)
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
                    schema_version: ancestry::CURVE_ANCESTRY_SCHEMA_VERSION,
                    method_derivation: None,
                    module: "vsh_gr".into(),
                    module_version: env!("CARGO_PKG_VERSION").into(),
                    inputs: vec![gr_input.clone()],
                    parameter_state: ancestry::parameter_state_for(&parameters),
                    parameters,
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: test_run_custody().actor,
                    timestamp_utc_ms: ancestry::ancestry_timestamp_utc_ms().unwrap(),
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
            input_set: None,
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
        db::insert_standard_curves_as_opened_project(
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
                input_set: None,
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
                input_set: None,
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
        db::insert_standard_curves_as_opened_project(
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
            input_set: None,
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
            db::insert_standard_curves_as_opened_project(
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
        let imported =
            ingest::import_deviation_csv(&conn, &dev, csv.to_str().unwrap(), Some(25.0), None, None);
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
            input_set: None,
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
        db::insert_standard_curves_as_opened_project(
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
                input_set: None,
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
            db::insert_standard_curves_as_opened_project(
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
            input_set: None,
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
                input_set: None,
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
            &PaySummaryRequest { well_ids: well_ids.clone(), vsh_max: Some(CutoffEntry { value: 0.5, unit: "v/v".into() }.into()), phie_min: Some(CutoffEntry { value: 0.1, unit: "v/v".into() }.into()), swe_max: Some(CutoffEntry { value: 0.6, unit: "v/v".into() }.into()), perm_min: None, input_set: None, skip_version: false, stats_only: false,
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
            db::insert_standard_curves_as_opened_project(
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
        db::insert_standard_curves_as_opened_project(
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
            db::insert_standard_curves_as_opened_project(
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
        let ancestry = ancestry::parse_curve_ancestry(&params_json).unwrap();
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
            db::insert_standard_curves_as_opened_project(
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
        let ancestry = ancestry::parse_curve_ancestry(&params_json).unwrap();
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
    /// AUDIT-2026-08-20 finding 54. The reserved-provenance-key guard was written out NINE times
    /// in three different shapes, and only two copies also consulted `legacy` - so whether that
    /// was a decision or an omission could not be read off the code.
    ///
    /// It was neither. `effective_module_parameters` puts every declared argument into BOTH maps -
    /// `legacy` under the bare name, `parameters` under `name_prefix + name` - and this call site
    /// passes an empty prefix, so the two lookups are the same lookup today. `legacy` is the one
    /// that stays correct if a prefix is ever passed, because a reserved key is always a bare
    /// name. Every site checks both now.
    ///
    /// Pinned from BOTH sides: dropping either lookup has to fail, or "checks both" is a comment
    /// rather than a behaviour.
    #[test]
    fn a_reserved_provenance_key_is_refused_from_either_map_by_one_guard() {
        let param = |name: &str| ancestry::AncestryParameter {
            name: name.into(),
            value: serde_json::json!(1.0),
            source: "test".into(),
            resolution: None,
            manifest_version: None,
            decision: None,
        };
        let empty = serde_json::Map::new();

        // A - declared as a PARAMETER.
        let declared = vec![param(MASK_PROVENANCE_KEY)];
        let refusal = reject_reserved_key(
            &declared, &empty, "sw_arch", "run-provenance", MASK_PROVENANCE_KEY,
        ).expect_err("a parameter colliding with a reserved key must be refused");
        assert!(
            refusal.contains(MASK_PROVENANCE_KEY) && refusal.contains("run-provenance"),
            "the refusal must name the key and what kind it is, got: {refusal}"
        );

        // B - declared as an OPTION, which reaches `legacy` under its bare name. This is the half
        // six of the nine copies did not look at.
        let mut legacy = serde_json::Map::new();
        legacy.insert(MASK_PROVENANCE_KEY.into(), serde_json::json!("MEAN"));
        reject_reserved_key(&[], &legacy, "sw_arch", "run-provenance", MASK_PROVENANCE_KEY)
            .expect_err("a legacy entry colliding with a reserved key must be refused too");

        // C - and an ordinary module is not refused.
        reject_reserved_key(&[param("M")], &empty, "sw_arch", "run-provenance", MASK_PROVENANCE_KEY)
            .expect("an unrelated argument must not trip the guard");

        // D - and it is still ONE guard. Nine hand-written copies is nine places for the next one
        // to diverge, which is what this finding was.
        let source = include_str!("workflow.rs");
        // Split so this line is not itself an occurrence of what it counts.
        let needle = format!("{} that collides with reserved", "declares an argument");
        assert_eq!(
            source.matches(needle.as_str()).count(),
            1,
            "the reserved-key refusal must be written in exactly one place"
        );
    }

    /// #129 stage 2 named its blocker with this, and this is now what shows the blocker is gone.
    /// `PERF-ATTEMPTS.md` §4: pooled reader connections made 99 of 100 wells fail with a COMMIT
    /// error reported out of a READ, and six hypotheses were ruled out before this bisect found
    /// the cause - `ancestry::try_resolve_ancestry_input` ran
    /// `db::migrate_standard_curves_to_generic_store`, the project-wide back-fill, from inside the
    /// read whenever a curve was missing, and N connections meant N threads each running the whole
    /// write. The read no longer repairs anything; the back-fill belongs to
    /// `project::open_and_migrate`.
    ///
    /// What the four arms said then, and say now:
    ///
    ///   - DEPTH walks the statement groups on an UN-BACKFILLED project - a legacy project that
    ///     was never opened. It used to lose 7 of 8 wells at group 1 to a duplicate `well_id`.
    ///     Now: 0 failed, and 0 resolved. **Both halves of that matter.** Zero failures is the
    ///     fix; zero resolutions is why the failure count alone would be worthless here, because
    ///     a read that resolves nothing cannot collide with anything.
    ///   - CONFIRM runs the back-fill once up front and repeats the same concurrent read. Clean
    ///     then and now; it is what turned a first-failing group into a named cause.
    ///   - CLEAN-STORE re-walks every group on the back-filled project - the state every
    ///     production open leaves behind. 0 failed and 8 of 8 resolved, which is the arm that
    ///     says the concurrency is genuinely usable rather than merely quiet.
    ///   - CONTROL runs the whole sequence serially. Clean, so the probe is measuring concurrency
    ///     rather than a broken fixture.
    ///
    /// STORE closes it: the un-backfilled project holds 0 `curve_meta` rows after every read
    /// above. Before the fix those rows arrived from eight threads racing to write them.
    ///
    /// The fixtures here deliberately DO NOT use
    /// `db::insert_standard_curves_as_opened_project` - an un-opened project is the state this
    /// exists to exercise, and handing it a repaired one would make every arm vacuously clean.
    ///
    /// Kept rather than deleted because the next stage-2 attempt needs exactly this to verify
    /// itself, and re-deriving it cost six experiments. `#[ignore]`d: it builds two projects and
    /// runs rayon over both, which the green gate must not wait on. It asserts nothing - it
    /// PRINTS, for a human to read - so it is never evidence that anything passes.
    #[test]
    #[ignore]
    fn the_only_write_on_the_module_input_read_path_is_the_generic_store_back_fill() {
        use rayon::prelude::*;

        let dir = std::env::temp_dir().join("sandibumi_pool_bisect");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("p.duckdb");
        let base = crate::db::init_db(path.to_str().unwrap()).unwrap();

        let wells = 8usize;
        let n = 200usize;
        let mut ids = Vec::new();
        for i in 0..wells {
            let id = uuid::Uuid::new_v4();
            crate::db::insert_well(&base, id, &format!("SANDI-{i}"), None, None, None).unwrap();
            let depth: Vec<f32> = (0..n).map(|k| 1000.0 + k as f32).collect();
            crate::db::insert_standard_curves(
                &base, id, depth,
                vec![40.0; n], vec![f32::NAN; n], vec![0.2; n],
                vec![2.4; n], vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            ids.push(id.to_string());
        }

        let spec = modules::list_modules().into_iter().find(|m| m.name == "vsh_gr").unwrap();
        let no_logs: HashMap<String, String> = HashMap::new();
        let params: HashMap<String, f64> =
            HashMap::from([("GR_MA".to_string(), 20.0), ("GR_SH".to_string(), 120.0)]);

        // depth==5 means "run everything"; lower numbers stop earlier, so the first depth that
        // errors names the statement.
        let names = [
            "1 resolved_log_args_for_well",
            "2 + validate_shale_clay_input_quantities",
            "3 + validate_neutron_basis_input",
            "4 + fetch_module_input_logs",
            "5 + resolve_param_arrays_with_default_usage",
        ];

        for (index, label) in names.iter().enumerate() {
            let depth_limit = index + 1;
            // Fresh clones each round, one per well, each used by exactly one rayon thread.
            let work: Vec<(String, Connection)> = ids
                .iter()
                .map(|id| (id.clone(), base.try_clone().expect("try_clone")))
                .collect();

            let outcomes: Vec<Result<bool, String>> = work
                .into_par_iter()
                .map(|(well_id, conn)| {
                    let attempt = || -> Result<bool, String> {
                        let resolved = crate::ancestry::try_resolve_ancestry_input(
                            &conn, &well_id, "GR", "GR", None, None,
                        )?
                        .is_some();
                        let log_args = resolved_log_args_for_well(
                            &conn, &well_id, &spec, &no_logs, None, None, &HashSet::new(),
                        )?;
                        if depth_limit == 1 {
                            return Ok(resolved);
                        }
                        validate_shale_clay_input_quantities(
                            &conn, &well_id, &spec, &log_args, None, None,
                        )?;
                        if depth_limit == 2 {
                            return Ok(resolved);
                        }
                        validate_neutron_basis_input(&conn, &well_id, &spec, &log_args)?;
                        if depth_limit == 3 {
                            return Ok(resolved);
                        }
                        let (depth, _logs, _units) = fetch_module_input_logs(
                            &conn, &well_id, &spec, &log_args, None, None,
                        )?;
                        if depth_limit == 4 {
                            return Ok(resolved);
                        }
                        let _ = resolve_param_arrays_with_default_usage(
                            &conn, &well_id, &spec, &params, &depth,
                        )?;
                        Ok(resolved)
                    };
                    attempt()
                })
                .collect();

            let errors: Vec<String> = outcomes
                .iter()
                .filter_map(|outcome| outcome.as_ref().err().cloned())
                .collect();
            let resolved = outcomes.iter().filter(|outcome| matches!(outcome, Ok(true))).count();
            println!(
                "DEPTH {label}: {} of {wells} failed, {resolved} of {wells} resolved GR{}",
                errors.len(),
                errors.first().map(|e| format!(" - first: {e}")).unwrap_or_default()
            );
        }

        // CONFIRMATION: the suspect is the lazy back-fill that try_resolve_ancestry_input runs when
        // a curve is missing from the generic store. Run it ONCE up front and the same concurrent
        // read must be clean - which is the difference between naming a cause and guessing one.
        {
            let fresh = dir.join("q.duckdb");
            let base2 = crate::db::init_db(fresh.to_str().unwrap()).unwrap();
            let mut ids2 = Vec::new();
            for i in 0..wells {
                let id = uuid::Uuid::new_v4();
                crate::db::insert_well(&base2, id, &format!("SANDI-B{i}"), None, None, None).unwrap();
                let depth: Vec<f32> = (0..n).map(|k| 1000.0 + k as f32).collect();
                crate::db::insert_standard_curves(
                    &base2, id, depth,
                    vec![40.0; n], vec![f32::NAN; n], vec![0.2; n],
                    vec![2.4; n], vec![f32::NAN; n], vec![f32::NAN; n],
                )
                .unwrap();
                ids2.push(id.to_string());
            }
            crate::db::migrate_standard_curves_to_generic_store(&base2).unwrap();
            let work: Vec<(String, Connection)> = ids2
                .iter()
                .map(|id| (id.clone(), base2.try_clone().expect("try_clone")))
                .collect();
            let errors: Vec<String> = work
                .into_par_iter()
                .filter_map(|(well_id, conn)| {
                    resolved_log_args_for_well(
                        &conn, &well_id, &spec, &no_logs, None, None, &HashSet::new(),
                    )
                    .err()
                })
                .collect();
            println!(
                "CONFIRM back-fill run first, then the same concurrent read: {} of {wells} failed{}",
                errors.len(),
                errors.first().map(|e| format!(" - first: {e}")).unwrap_or_default()
            );

            // And every OTHER statement group, on the pre-migrated store. The first loop above
            // could not test these honestly: its own round 1 ran the back-fill, so rounds 2-5 were
            // measuring an already-migrated project. If a SECOND lazy write hides further down the
            // path, this is where it shows.
            for (index, label) in names.iter().enumerate() {
                let depth_limit = index + 1;
                let work: Vec<(String, Connection)> = ids2
                    .iter()
                    .map(|id| (id.clone(), base2.try_clone().expect("try_clone")))
                    .collect();
                let outcomes: Vec<Result<bool, String>> = work
                    .into_par_iter()
                    .map(|(well_id, conn)| {
                        let attempt = || -> Result<bool, String> {
                            let resolved = crate::ancestry::try_resolve_ancestry_input(
                                &conn, &well_id, "GR", "GR", None, None,
                            )?
                            .is_some();
                            let log_args = resolved_log_args_for_well(
                                &conn, &well_id, &spec, &no_logs, None, None, &HashSet::new(),
                            )?;
                            if depth_limit == 1 {
                                return Ok(resolved);
                            }
                            validate_shale_clay_input_quantities(
                                &conn, &well_id, &spec, &log_args, None, None,
                            )?;
                            if depth_limit == 2 {
                                return Ok(resolved);
                            }
                            validate_neutron_basis_input(&conn, &well_id, &spec, &log_args)?;
                            if depth_limit == 3 {
                                return Ok(resolved);
                            }
                            let (depth, _l, _u) = fetch_module_input_logs(
                                &conn, &well_id, &spec, &log_args, None, None,
                            )?;
                            if depth_limit == 4 {
                                return Ok(resolved);
                            }
                            let _ = resolve_param_arrays_with_default_usage(
                                &conn, &well_id, &spec, &params, &depth,
                            )?;
                            Ok(resolved)
                        };
                        attempt()
                    })
                    .collect();
                let errors: Vec<String> = outcomes
                    .iter()
                    .filter_map(|outcome| outcome.as_ref().err().cloned())
                    .collect();
                let resolved =
                    outcomes.iter().filter(|outcome| matches!(outcome, Ok(true))).count();
                println!(
                    "CLEAN-STORE {label}: {} of {wells} failed, {resolved} of {wells} resolved GR{}",
                    errors.len(),
                    errors.first().map(|e| format!(" - first: {e}")).unwrap_or_default()
                );
            }
        }

        // Control: the whole sequence on ONE connection, serially. Must be clean, or the probe is
        // measuring a broken fixture rather than concurrency.
        let mut serial_errors = 0usize;
        for well_id in &ids {
            let attempt = || -> Result<(), String> {
                let log_args = resolved_log_args_for_well(
                    &base, well_id, &spec, &no_logs, None, None, &HashSet::new(),
                )?;
                validate_shale_clay_input_quantities(&base, well_id, &spec, &log_args, None, None)?;
                validate_neutron_basis_input(&base, well_id, &spec, &log_args)?;
                let (depth, _l, _u) =
                    fetch_module_input_logs(&base, well_id, &spec, &log_args, None, None)?;
                let _ = resolve_param_arrays_with_default_usage(
                    &base, well_id, &spec, &params, &depth,
                )?;
                Ok(())
            };
            if attempt().is_err() {
                serial_errors += 1;
            }
        }
        println!("CONTROL serial on the base connection: {serial_errors} of {wells} failed");

        // And the whole reason the read stopped repairing: `base` was never opened through
        // `project::open_and_migrate`, so its generic store is empty - and every concurrent read
        // above must have LEFT it empty. Before #129's fix this count was non-zero and the rows
        // arrived from eight threads racing to write them.
        let store_rows: i64 = base
            .query_row("SELECT COUNT(*) FROM curve_meta", [], |row| row.get(0))
            .expect("counting curve_meta");
        println!(
            "STORE un-backfilled project holds {store_rows} curve_meta rows after every read above (0 = the read never wrote)"
        );
    }
}
