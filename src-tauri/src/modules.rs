//! Deterministic petrophysics module library, ported from Loglan sources
//! (vsh_gr.lls, vsh_dn.lls, phi_den.lls, phi_dn.lls, sw_arch.lls, sw_indo.lls, sw_sim.lls,
//! perm_wyllie_rose.lls, perm_coates.lls) with the same MISSING semantics (`f32::NAN`),
//! LIMIT clamping, and per-frame evaluation model.
//!
//! Each module carries a manifest (`.info`-style) that the frontend
//! uses to auto-generate its parameter dialog: numeric interval parameters with cited defaults
//! or explicit `ABSENT` state, validation ranges, string options with fixed choices, and
//! input/output logs.
//!
//! Density convention: g/cc (matching LAS field data), not the kg/m3 some suites use.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use crate::units::{convert_depth, DepthUnit};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArgKind {
    /// Numeric interval parameter (per-zone overridable).
    Param,
    /// String option with fixed choices (global per run).
    Option,
    /// Free text (global per run) — travels in `opts` exactly as [`ArgKind::Option`] does, but
    /// renders as a typed field because the valid values are not a list the manifest can hold.
    ///
    /// Added for the Condition family, where the user names the output curve himself (Jauhar,
    /// 2026-08-05). An Option cannot express that: the answer is a mnemonic, and the set of
    /// mnemonics a project might want is the set of all strings.
    Text,
    /// Input log curve (resolved from standard/computed curves).
    LogIn,
    /// Output log curve (written to computed_curves).
    LogOut,
}

/// Semantic role of a binary flag curve. The numeric polarity is deliberately not part of this
/// enum: every role uses the one [`FlagValue`] mapping below, while the role tells a consumer
/// whether the curve is intended to exclude samples or merely explain/diagnose them.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlagKind {
    ExclusionMask,
    DiagnosticIndicator,
}

/// The only binary flag polarity in the deterministic module system. Producers construct this
/// type, never numeric truth values; conversion to the persisted f32 channel occurs in one place.
/// Missing remains `f32::NAN`, in accordance with the project-wide missing-data contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlagValue {
    Missing,
    Clear,
    Flagged,
}

impl FlagValue {
    pub(crate) fn as_f32(self) -> f32 {
        match self {
            Self::Missing => f32::NAN,
            Self::Clear => 0.0,
            Self::Flagged => 1.0,
        }
    }
}

/// Typed construction boundary for one flag channel. A caller cannot select a numeric polarity;
/// it can only mark semantic states and let [`FlagValue::as_f32`] perform the single conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlagCurve {
    values: Vec<FlagValue>,
}

impl FlagCurve {
    pub(crate) fn clear(len: usize) -> Self {
        Self {
            values: vec![FlagValue::Clear; len],
        }
    }

    pub(crate) fn missing(len: usize) -> Self {
        Self {
            values: vec![FlagValue::Missing; len],
        }
    }

    pub(crate) fn set(&mut self, index: usize, value: FlagValue) {
        self.values[index] = value;
    }

    pub(crate) fn get(&self, index: usize) -> FlagValue {
        self.values[index]
    }

    pub(crate) fn is_flagged(&self, index: usize) -> bool {
        self.get(index) == FlagValue::Flagged
    }

    fn validate_f32(values: &[f32], identity: &str) -> Result<(), String> {
        for (index, value) in values.iter().copied().enumerate() {
            if !value.is_nan()
                && value != FlagValue::Clear.as_f32()
                && value != FlagValue::Flagged.as_f32()
            {
                return Err(format!(
                    "flag output '{identity}' produced {value} at sample {index}; the only finite flag values are 0 = clear and 1 = flagged"
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn from_f32(values: Vec<f32>, identity: &str) -> Result<Self, String> {
        Self::validate_f32(&values, identity)?;
        Ok(Self {
            values: values
                .into_iter()
                .map(|value| {
                    if value.is_nan() {
                        FlagValue::Missing
                    } else if value == FlagValue::Clear.as_f32() {
                        FlagValue::Clear
                    } else {
                        FlagValue::Flagged
                    }
                })
                .collect(),
        })
    }

    pub(crate) fn into_f32(self) -> Vec<f32> {
        self.values.into_iter().map(FlagValue::as_f32).collect()
    }
}

pub(crate) fn sample_is_flagged(value: f32) -> bool {
    value == FlagValue::Flagged.as_f32()
}

pub(crate) const fn framework_precondition_flag_kind() -> FlagKind {
    FlagKind::DiagnosticIndicator
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ValidityBranch {
    /// Option/Text argument whose selected id activates this condition.
    pub argument: String,
    /// Exact stable wire id that activates this condition.
    pub equals: String,
}

/// Exact machine-readable token for a numeric parameter that deliberately ships without a
/// default. This is a provenance state, not a citation and not permission to invent a value.
pub const ABSENT_DEFAULT_SOURCE: &str = "ABSENT";

/// The one manifest token for a numeric length expressed in the project's declared depth unit.
/// Fixed-unit parameters such as metre-qualified `SHIFT` deliberately do not use this token.
pub const PROJECT_DEPTH_UNIT_TOKEN: &str = "depth";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ValidityRule {
    /// The argument value must be one of the `ArgSpec::choices` ids.
    Enumeration,
    /// The argument's numeric value (Param) or every finite sample (LogIn) must be in range.
    NumericRange {
        min: Option<f64>,
        max: Option<f64>,
        unit: String,
        #[serde(default)]
        when: Option<ValidityBranch>,
    },
    /// At least one named LogIn argument must contain a finite sample.
    RequiredCompanion {
        any_of: Vec<String>,
        #[serde(default)]
        when: Option<ValidityBranch>,
    },
    /// The argument must carry a finite value when the selected method branch is active.
    /// This keeps an absent branch parameter honest without making unrelated branches unusable.
    RequiredValue {
        #[serde(default)]
        when: Option<ValidityBranch>,
    },
    /// This LogIn must be finite at every sample where another named LogIn is finite.
    ///
    /// Unlike [`ValidityRule::RequiredCompanion`], this is a whole-run refusal rather than an
    /// "at least one finite sample" availability check. It is used when letting one uncovered
    /// sample through would manufacture an unmarked correction-named copy.
    RequiredWhereFinite { input: String },
    /// The argument's numeric value must be strictly below another numeric argument per sample.
    LessThan { other: String },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ValidityCondition {
    /// Stable condition id used in refusals and persisted manifests.
    pub id: String,
    /// Human explanation shown beside the field and repeated in a refusal.
    pub statement: String,
    /// Named source for this condition. Empty strings are rejected by the registry test.
    pub source: String,
    #[serde(flatten)]
    pub rule: ValidityRule,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SourcedGuidance {
    /// Advice about how the interpreter may derive or select a value. This is deliberately
    /// separate from [`ArgSpec::default`]: a convention is not a numeric value.
    pub text: String,
    /// Named source for the advice. Blank sources are rejected by the owned contract test.
    pub source: String,
}

/// Physical quantity carried by a shale/clay-volume curve.
///
/// `v/v` is only a unit: both quantities use it. The producer therefore declares this identity,
/// and the runner carries it independently of the curve name so an output rename cannot turn clay
/// into shale (or vice versa).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ShaleClayQuantity {
    #[serde(rename = "VSH")]
    ShaleVolume,
    #[serde(rename = "VCL")]
    ClayVolume,
}

/// The stable deterministic-porosity family identity. Numerical limits do not live here: DEC-015
/// keeps those method-specific and source-bound so a density rule cannot silently become a sonic
/// rule merely because both outputs carry porosity.
pub const POROSITY_FAMILY_ID: &str = "POR";
pub(crate) const PHIE_DN_LIMITED_DEFAULT: &str = "PHIE_DN_LIM";
pub(crate) const PHIT_DN_LIMITED_DEFAULT: &str = "PHIT_DN_LIM";
pub const POROSITY_LIMITING_CONTRACT: &str = "porosity_method_limit_policy_v1";
pub const POROSITY_FLAG_CONTRACT: &str = "porosity_branch_limit_reason_v1";
pub const POROSITY_OUTPUT_NAMING_CONTRACT: &str = "workflow_resolved_output_name_v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PorosityModuleRole {
    DeterministicMethod,
    ComparisonProducer,
    LimitProducer,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PorosityOutputRole {
    UnlimitedEffective,
    UnlimitedTotal,
    LimitedEffective,
    LimitedTotal,
    ComparisonUnlimitedEffective,
    ComparisonUnlimitedTotal,
    ComparisonLimitedEffective,
    ComparisonLimitedTotal,
    Effective,
    Total,
    FreeFluid,
    Capped,
    Ceiling,
}

/// SB-POR-001 defines the one reason-channel shape. Actual per-sample emission is owned by
/// SB-POR-003, so the manifest must say that it is pending instead of advertising flags that the
/// current evaluators do not yet write.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PorosityFlagEmission {
    PendingSbPor003,
}

/// POR-family custody attached to a declared output. The common envelope contains identities and
/// observability contracts only; it deliberately has no numeric floor, ceiling or correction
/// coefficient field. Those values remain in each method's separately sourced arguments/policy.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PorosityOutputContract {
    pub family: String,
    pub module_role: PorosityModuleRole,
    pub method: String,
    pub convention: String,
    pub output_role: PorosityOutputRole,
    pub limiting_contract: String,
    pub limiting_policy: String,
    pub limiting_policy_source: String,
    pub flag_contract: String,
    pub flag_emission: PorosityFlagEmission,
    pub output_naming_contract: String,
}

/// Source-unit custody for one numeric parameter value. The artefact spelling is preserved while
/// the conversion itself uses the normalized, generated registry identity.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ParameterUnitCustody {
    pub artefact_value: f64,
    pub artefact_unit: String,
    pub canonical_value: f64,
    pub canonical_unit: String,
    pub conversion: crate::curves::NamedUnitConversion,
}

impl ParameterUnitCustody {
    pub(crate) fn new(
        artefact_value: f64,
        artefact_unit: &str,
        canonical_unit: &str,
    ) -> Result<Self, String> {
        let conversion = crate::curves::named_unit_conversion(artefact_unit, canonical_unit)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            artefact_value,
            artefact_unit: artefact_unit.into(),
            canonical_value: conversion.apply(artefact_value),
            canonical_unit: canonical_unit.into(),
            conversion,
        })
    }
}

impl ShaleClayQuantity {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ShaleVolume => "VSH",
            Self::ClayVolume => "VCL",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ArgSpec {
    pub name: String,
    pub desc: String,
    pub unit: String,
    pub kind: ArgKind,
    /// LogOut only: semantic flag role. `None` means an ordinary numeric/class output.
    #[serde(default)]
    pub flag_kind: Option<FlagKind>,
    /// LogIn only: physical quantities this role accepts. Empty means this requirement has no
    /// shale/clay quantity contract. More than one entry is an explicit dual-type consumer, never
    /// a mnemonic fallback.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_shale_clay_quantities: Vec<ShaleClayQuantity>,
    /// LogOut only: producer-owned physical identity persisted beside the resolved output name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_shale_clay_quantity: Option<ShaleClayQuantity>,
    /// LogOut only: the common POR-family envelope plus this method's own policy identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub porosity_output: Option<PorosityOutputContract>,
    /// Default numeric value (Param), default choice (Option), or default curve mnemonic (LogIn).
    pub default: String,
    /// LogIn only: ordered curve mnemonics tried when the interpreter has not selected one
    /// explicitly. The first available curve wins and the resolved mnemonic is recorded in the
    /// run ancestry. Empty means the ordinary single `default` mnemonic is used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preferred_aliases: Vec<String>,
    /// Source-bearing interpreter guidance shown beside the field. It may explain how to pick a
    /// value, but it never supplies [`ArgSpec::default`] and is never consumed by computation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guidance: Vec<SourcedGuidance>,
    /// Source for a numeric Param default, or the exact token [`ABSENT_DEFAULT_SOURCE`] when the
    /// parameter deliberately ships without one. Empty is invalid for every registered Param.
    #[serde(default)]
    pub default_source: String,
    /// Numeric-default custody in the source artefact's unit. `None` for arguments with no numeric
    /// default; explicit run values receive their own canonical-unit identity record at runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_unit_custody: Option<ParameterUnitCustody>,
    /// Valid choices for Option args. **These are stored in `params_json` on every saved run, so
    /// they must never be renamed** — that is what `choice_labels` is for.
    pub choices: Vec<String>,
    /// Optional display text, parallel to `choices`. Empty means "show the id".
    ///
    /// `OPT_GR`'s choices are the bare strings `LARINOV1`, `LARINOV2`, … with no rock age, no
    /// coefficient and no tooltip, so the only place a user was told which is which was the manual
    /// test plan — and the plan had them the wrong way round (`docs/review_triage.md` finding 21).
    /// Picking the wrong one returns 0.33 where 0.216 belongs: a shale volume more than half again
    /// too high through the whole intermediate-GR interval, which is exactly where the VSH cutoff
    /// decides net pay. The curve looks entirely normal and nothing downstream can catch it.
    #[serde(default)]
    pub choice_labels: Vec<String>,
    /// `SB-CORE-013` topic key: the parameter this arg sets is one the corpus records COMPETING
    /// shipped values for, so the editor shows them with their sources at the point of choice
    /// (`param_sources::sources_for`). Empty for the overwhelming majority of args, which is why it
    /// is a key rather than an embedded list — the values belong to the topic, not to the module, and
    /// electrofacies, GMM facies and the ML dialog must not be able to show three different answers
    /// for the same number.
    #[serde(default)]
    pub sources_topic: String,
    /// Source-bearing, machine-readable preconditions evaluated at the public dispatch boundary.
    #[serde(default)]
    pub validity_conditions: Vec<ValidityCondition>,
    /// Validation range for Param args.
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Whether a LogIn is required (missing optional inputs become all-NaN).
    pub required: bool,
    /// LogIn only: other declared input arguments that can satisfy this required role.
    ///
    /// This is an explicit one-of contract, not a mnemonic fallback. `sw_rtc`, for example, can
    /// consume either its SSC `PHIT` input or its separately declared `PHIT_SSPW` input. Marking
    /// the primary optional would allow neither; marking it unconditionally required would reject
    /// the valid SSPW route before the body can combine the two.
    #[serde(default)]
    pub required_any_of: Vec<String>,
    /// LogIn only: resolve from computed provenance (precalc outputs, log sets) and never
    /// the RAW import store — for unit-contract inputs like FTEMP/FPRESS where a raw
    /// curve with the same mnemonic (a commercial LAS export's degF FTEMP) would silently
    /// masquerade as the degC/psi curve the module assumes.
    pub computed_only: bool,
    /// Param only: a NAMED-zone override of this parameter is REFUSED. The `*` well-wide scope
    /// still applies, which is the point — the parameter has one value per well, not one per zone.
    ///
    /// For a parameter defining a TREND against depth this is a physical statement, not a
    /// convenience. `precalc` computes `SURF_TEMP + TEMP_GRAD × TVDSS` from surface at every
    /// sample rather than integrating down through the zones above it, so giving a lower zone its
    /// own gradient makes the temperature profile JUMP at the boundary instead of bending: a 0.03
    /// °C/m well with a 0.035 override below 1500 m stepped **10.5 °C across 100 m** where the
    /// undisturbed trend rises 3.0. Rock temperature is continuous — a 10 °C discontinuity at a
    /// formation top is not something the earth does — and it does not stay in FTEMP, because the
    /// Arps correction turns temperature into Rw and Rw goes straight into Sw.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 6): *"temperature is curves
    /// only"* — the geothermal trend belongs to the well and its product is a curve, so there is
    /// no per-zone gradient to integrate and the question of what temperature each zone starts at
    /// never arises.
    ///
    /// Deliberately NOT applied to `PSURF`/`PGRAD`, whose structure is identical: a pressure step
    /// at a formation top is a pressure compartment, which is a real thing rock does. The
    /// asymmetry is the physics, not an oversight.
    #[serde(default)]
    pub well_scope: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModuleSpec {
    pub name: String,
    pub title: String,
    pub category: String, // "VSH" | "Porosity" | "Saturation" | "Permeability" | "Prep"
    pub doc: String,
    pub args: Vec<ArgSpec>,
}

/// Stable key for the exact validity manifest stored beside each module run's legacy parameter
/// payload. The snapshot prevents a later module manifest from silently rewriting which range,
/// branch, companion, statement or source governed an earlier interpretation.
pub(crate) const MODULE_VALIDITY_MANIFEST_KEY: &str = "_sandibumi_module_validity_v1";
pub(crate) const MODULE_VALIDITY_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SavedValidityArgument {
    pub argument: String,
    pub conditions: Vec<ValidityCondition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModuleValidityManifest {
    pub schema_version: u32,
    pub module: String,
    pub arguments: Vec<SavedValidityArgument>,
}

pub(crate) fn module_validity_manifest(spec: &ModuleSpec) -> ModuleValidityManifest {
    ModuleValidityManifest {
        schema_version: MODULE_VALIDITY_MANIFEST_SCHEMA_VERSION,
        module: spec.name.clone(),
        arguments: spec
            .args
            .iter()
            .filter(|argument| !argument.validity_conditions.is_empty())
            .map(|argument| SavedValidityArgument {
                argument: argument.name.clone(),
                conditions: argument.validity_conditions.clone(),
            })
            .collect(),
    }
}

pub(crate) fn param(
    name: &str,
    desc: &str,
    unit: &str,
    default: f64,
    min: f64,
    max: f64,
    default_source: &str,
) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::Param,
        flag_kind: None,
        accepted_shale_clay_quantities: vec![],
        output_shale_clay_quantity: None,
        porosity_output: None,
        default: default.to_string(),
        preferred_aliases: vec![],
        guidance: vec![],
        default_source: default_source.into(),
        default_unit_custody: None,
        choices: vec![],
        choice_labels: vec![],
        validity_conditions: vec![],
        min: Some(min),
        max: Some(max),
        required: true,
        required_any_of: vec![],
        computed_only: false,
        well_scope: false,
        sources_topic: String::new(),
    }
}

/// A cited numeric default transcribed in the artefact's own unit and converted through the
/// generated registry. The effective canonical default is derived here; callers never repeat it
/// as a second magic literal that could drift away from its custody record.
fn param_from_artefact(
    name: &str,
    desc: &str,
    canonical_unit: &str,
    artefact_value: f64,
    artefact_unit: &str,
    min: f64,
    max: f64,
    default_source: &str,
) -> ArgSpec {
    let custody = ParameterUnitCustody::new(artefact_value, artefact_unit, canonical_unit)
        .unwrap_or_else(|error| panic!("invalid unit custody for {name}: {error}"));
    ArgSpec {
        default_unit_custody: Some(custody.clone()),
        ..param(
            name,
            desc,
            canonical_unit,
            custody.canonical_value,
            min,
            max,
            default_source,
        )
    }
}

pub(crate) fn opt(name: &str, desc: &str, default: &str, choices: &[&str]) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: String::new(),
        kind: ArgKind::Option,
        flag_kind: None,
        accepted_shale_clay_quantities: vec![],
        output_shale_clay_quantity: None,
        porosity_output: None,
        default: default.into(),
        preferred_aliases: vec![],
        guidance: vec![],
        default_source: String::new(),
        default_unit_custody: None,
        choices: choices.iter().map(|s| s.to_string()).collect(),
        choice_labels: Vec::new(),
        validity_conditions: vec![],
        min: None,
        max: None,
        required: true,
        required_any_of: vec![],
        computed_only: false,
        well_scope: false,
        sources_topic: String::new(),
    }
}

/// [`opt`] with display text per choice — same ids on the wire, a readable dropdown on screen.
pub(crate) fn opt_labelled(
    name: &str,
    desc: &str,
    default: &str,
    choices: &[(&str, &str)],
) -> ArgSpec {
    let mut a = opt(name, desc, default, &choices.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    a.choice_labels = choices.iter().map(|(_, label)| (*label).to_string()).collect();
    a
}

fn validity(id: &str, statement: &str, source: &str, rule: ValidityRule) -> ValidityCondition {
    ValidityCondition { id: id.into(), statement: statement.into(), source: source.into(), rule }
}

fn with_validity(mut arg: ArgSpec, conditions: Vec<ValidityCondition>) -> ArgSpec {
    arg.validity_conditions = conditions;
    arg
}

fn with_guidance(mut arg: ArgSpec, items: &[(&str, &str)]) -> ArgSpec {
    arg.guidance = items
        .iter()
        .map(|(text, source)| SourcedGuidance {
            text: (*text).into(),
            source: (*source).into(),
        })
        .collect();
    arg
}

/// A [`param`] with NO default — the field opens EMPTY and the dialog refuses to run until a
/// value is given (`required: true`), or accepts the blank as "no bound on this side"
/// (`required: false`, and the module then sees NaN).
///
/// Jauhar's call for the despike window, 2026-08-05: *"No default — I set it every run."* The
/// reasoning is the provenance rule with teeth. A despike window is a THICKNESS, and what counts
/// as a spike rather than a thin bed is a property of the tool, the sampling and the rock — there
/// is no number that is right in two basins. A shipped default would be somebody's field
/// calibration wearing the authority of a manifest, and a despiked curve looks entirely plausible
/// whichever window produced it. Same family as `gr_normalize`'s reference percentiles, which are
/// pinned as generic precisely so nobody's regression result ships as the default.
///
/// The optional form exists for a genuine two-sided bound (Clip's MIN/MAX): leaving one side empty
/// is a statement that the curve is unbounded there, not an omission.
pub(crate) fn param_open(
    name: &str,
    desc: &str,
    unit: &str,
    min: f64,
    max: f64,
    required: bool,
) -> ArgSpec {
    ArgSpec {
        default: String::new(),
        default_source: ABSENT_DEFAULT_SOURCE.into(),
        required,
        ..param(name, desc, unit, 0.0, min, max, ABSENT_DEFAULT_SOURCE)
    }
}

/// A deliberately absent parameter that is required only on named option branches.
///
/// The field stays optional at the generic [`ArgSpec::required`] layer so an inactive method does
/// not demand parameters it cannot consume. Each active branch is represented by a sourced
/// [`ValidityRule::RequiredValue`] condition and is enforced by the public runner.
pub(crate) fn param_open_when(
    name: &str,
    desc: &str,
    unit: &str,
    min: f64,
    max: f64,
    branches: &[(&str, &str)],
    source: &str,
) -> ArgSpec {
    let conditions = branches
        .iter()
        .map(|(argument, equals)| {
            validity(
                &format!(
                    "{}.required_when_{}",
                    name.to_lowercase(),
                    equals.to_lowercase()
                ),
                &format!("{name} is required when {argument} = {equals}."),
                source,
                ValidityRule::RequiredValue {
                    when: Some(ValidityBranch {
                        argument: (*argument).into(),
                        equals: (*equals).into(),
                    }),
                },
            )
        })
        .collect();
    with_validity(param_open(name, desc, unit, min, max, false), conditions)
}

/// A free-text run option (see [`ArgKind::Text`]). Reaches the module through `opts`.
/// A free-text run option. Currently unused: its only caller was the Condition/Frame families'
/// "Output curve name" field, which the output-name grid replaced (`log_out_as`). Kept because
/// `ArgKind::Text` is a real kind the whole stack already renders, and a manifest wanting a
/// free-string parameter should not have to re-add the plumbing.
#[allow(dead_code)]
pub(crate) fn text(name: &str, desc: &str, default: &str) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: String::new(),
        kind: ArgKind::Text,
        flag_kind: None,
        accepted_shale_clay_quantities: vec![],
        output_shale_clay_quantity: None,
        porosity_output: None,
        default: default.into(),
        preferred_aliases: vec![],
        guidance: vec![],
        default_source: String::new(),
        default_unit_custody: None,
        choices: vec![],
        choice_labels: vec![],
        validity_conditions: vec![],
        min: None,
        max: None,
        required: false,
        required_any_of: vec![],
        computed_only: false,
        well_scope: false,
        sources_topic: String::new(),
    }
}

pub(crate) fn log_in(name: &str, desc: &str, unit: &str, default_curve: &str, required: bool) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::LogIn,
        flag_kind: None,
        accepted_shale_clay_quantities: vec![],
        output_shale_clay_quantity: None,
        porosity_output: None,
        default: default_curve.into(),
        preferred_aliases: vec![],
        guidance: vec![],
        default_source: String::new(),
        default_unit_custody: None,
        choices: vec![],
        choice_labels: vec![],
        validity_conditions: vec![],
        min: None,
        max: None,
        required,
        required_any_of: vec![],
        computed_only: false,
        well_scope: false,
        sources_topic: String::new(),
    }
}

/// A [`param`] the corpus records COMPETING shipped values for (`SB-CORE-013`).
///
/// The editor shows those values with their sources beside the field, and the run records which of
/// them the interpreter's choice agrees with. Reach for this only where
/// [`crate::param_sources::sources_for`] actually has entries — a topic key with nothing behind it
/// renders an empty panel, which reads as "nobody disagrees" and is the opposite of the point.
pub(crate) fn param_sourced(
    name: &str,
    desc: &str,
    unit: &str,
    default: f64,
    min: f64,
    max: f64,
    topic: &str,
    default_source: &str,
) -> ArgSpec {
    ArgSpec {
        sources_topic: topic.into(),
        ..param(name, desc, unit, default, min, max, default_source)
    }
}

/// Add a competing-value topic without changing whether the underlying parameter has a default.
/// Disclosure must never turn an `ABSENT` parameter into a plausible number merely because another
/// product ships one.
pub(crate) fn with_sources(mut arg: ArgSpec, topic: &str) -> ArgSpec {
    debug_assert!(!crate::param_sources::sources_for(topic).is_empty());
    arg.sources_topic = topic.into();
    arg
}

/// A deliberately absent, well-scoped parameter. It retains the depth-trend scope rule while
/// refusing to invent the value that defines that trend. The per-well grid may supply one value,
/// but a zone cannot create a discontinuity part-way down the well.
pub(crate) fn param_open_well(name: &str, desc: &str, unit: &str, min: f64, max: f64) -> ArgSpec {
    ArgSpec {
        well_scope: true,
        ..param_open(name, desc, unit, min, max, true)
    }
}

/// A whole-well absent parameter that is required only on named option branches.
pub(crate) fn param_open_well_when(
    name: &str,
    desc: &str,
    unit: &str,
    min: f64,
    max: f64,
    branches: &[(&str, &str)],
    source: &str,
) -> ArgSpec {
    ArgSpec {
        well_scope: true,
        ..param_open_when(name, desc, unit, min, max, branches, source)
    }
}

/// A [`log_in`] restricted to computed provenance (see [`ArgSpec::computed_only`]).
pub(crate) fn log_in_computed(name: &str, desc: &str, unit: &str, default_curve: &str, required: bool) -> ArgSpec {
    ArgSpec { computed_only: true, ..log_in(name, desc, unit, default_curve, required) }
}

/// A required input role that may be satisfied by this argument or one of the other named LogIn
/// arguments. Every alternative remains independently declared in the manifest and therefore
/// independently selectable and persisted.
pub(crate) fn log_in_one_of(
    name: &str,
    desc: &str,
    unit: &str,
    default_curve: &str,
    alternatives: &[&str],
) -> ArgSpec {
    ArgSpec {
        required_any_of: alternatives.iter().map(|alternative| (*alternative).to_string()).collect(),
        ..log_in(name, desc, unit, default_curve, true)
    }
}

/// An output curve. `default` is EMPTY, which means "written under the declared name" — `VSH`
/// declares `VSH` and writes `VSH`.
///
/// See [`log_out_as`] for the outputs whose name is built from a run's own choices, and
/// [`crate::workflow::resolve_output_names`] for the one place either is turned into the name a
/// run actually writes.
pub(crate) fn log_out(name: &str, desc: &str, unit: &str) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::LogOut,
        flag_kind: None,
        accepted_shale_clay_quantities: vec![],
        output_shale_clay_quantity: None,
        porosity_output: None,
        default: String::new(),
        preferred_aliases: vec![],
        guidance: vec![],
        default_source: String::new(),
        default_unit_custody: None,
        choices: vec![],
        choice_labels: vec![],
        validity_conditions: vec![],
        min: None,
        max: None,
        required: true,
        required_any_of: vec![],
        computed_only: false,
        well_scope: false,
        sources_topic: String::new(),
    }
}

/// An output whose DEFAULT name is built from the run's own choices — `log_predict` writes
/// `<target>_SYN`, `phi_cap` writes `<input>_CAP`, a despiked curve writes `<input>_C`.
///
/// `pattern` is the declared name with `{ARG}` placeholders naming other args of the same module:
/// a LogIn expands to the mnemonic the run chose for it, a LogOut to the name that output already
/// resolved to (declaration order), anything else to its option/text value.
///
/// **The module returns its DECLARED key and never builds this name itself.** Five modules used to
/// `format!` their own output name, which meant the manifest's declared LogOut described a curve
/// the run did not write — so a dialog listing "Outputs: SYN" was lying, and there was no way to
/// offer a rename without a second implementation of each module's naming rule. One expansion, in
/// the framework, is also what lets [`crate::workflow::resolve_output_names`] check every name a
/// run is about to write against the shadowing rule below.
pub(crate) fn log_out_as(name: &str, pattern: &str, desc: &str, unit: &str) -> ArgSpec {
    ArgSpec { default: pattern.into(), ..log_out(name, desc, unit) }
}

/// A flag output declared with its semantic role. This is the only manifest constructor for a
/// flag; ordinary [`log_out`] remains explicitly untyped.
pub(crate) fn log_out_flag(name: &str, desc: &str, kind: FlagKind) -> ArgSpec {
    ArgSpec { flag_kind: Some(kind), ..log_out(name, desc, "") }
}

pub(crate) fn log_out_flag_as(
    name: &str,
    pattern: &str,
    desc: &str,
    kind: FlagKind,
) -> ArgSpec {
    ArgSpec {
        default: pattern.into(),
        ..log_out_flag(name, desc, kind)
    }
}

/// A [`log_in`] whose automatic selection follows a source-defined ordered alias list.
/// Explicit interpreter selections still win; the runner uses these only when no selection was
/// supplied and records the mnemonic it actually found for each well.
fn log_in_preferred(
    name: &str,
    desc: &str,
    unit: &str,
    preferred_aliases: &[&str],
    required: bool,
) -> ArgSpec {
    debug_assert!(!preferred_aliases.is_empty());
    ArgSpec {
        preferred_aliases: preferred_aliases
            .iter()
            .map(|alias| alias.trim().to_uppercase())
            .collect(),
        // Keep the ordinary raw mnemonic as the compatibility default for callers such as the
        // Monte Carlo engine that construct a module context directly. Workflow/preflight paths
        // consume `preferred_aliases` and therefore still resolve corrected-first per well.
        ..log_in(
            name,
            desc,
            unit,
            preferred_aliases.last().expect("preferred aliases are non-empty"),
            required,
        )
    }
}

/// Everything a module needs at run time, resolved by the workflow runner:
/// input logs by arg name, per-sample numeric parameter arrays (zone-resolved),
/// and global string options.
#[derive(Clone)]
pub struct ModuleContext {
    pub n: usize,
    pub logs: HashMap<String, Vec<f32>>,
    pub params: HashMap<String, Vec<f64>>,
    pub opts: HashMap<String, String>,
    /// The unit DEPTH (and any depth-derived param such as a free-water level) is in for
    /// this run — the project's declared unit. Modules whose physics is unit-specific must
    /// consult this rather than assume: the capillary-pressure law in `satheight.rs` is
    /// per FOOT of column, so on a foot-declared project a hardcoded metres→feet multiply
    /// returned a Pc 3.28x too high. A typed field rather than an `opts` key deliberately:
    /// a missing string key would silently mean metres, which is the failure mode itself.
    pub depth_unit: crate::units::DepthUnit,
}

/// Reserved `opts` key: the run's DECLARED class curves, upper-cased and comma-separated
/// (`SB-MLA-055`). Set by `workflow.rs`, which has the connection; read through
/// [`ModuleContext::input_is_class_curve`].
///
/// Carried in `opts` rather than as a `ModuleContext` field, for the reason `MASK` and
/// `OUT_PREFIX` are: it is one cross-cutting rule about a run, not a parameter any module declares,
/// and a new typed field would have to be threaded through all forty-odd context constructions to
/// say nothing in most of them.
pub(crate) const CLASS_CURVES_OPT: &str = "__CLASS_CURVES";
pub(crate) const INPUT_UNIT_OPT_PREFIX: &str = "__UNIT_";

impl ModuleContext {
    /// Whether the curve named by log argument `arg` was DECLARED a class curve.
    ///
    /// Used by the modules that would otherwise average codes — `frame::block`,
    /// `condition::smooth`, `condition::despike`. They refuse rather than silently substituting a
    /// safe statistic: unlike a re-frame, which reports the method it resolved per curve, a module
    /// has no channel to say "I did something other than what you asked", and a coercion nobody can
    /// see is the failure this rule exists to prevent.
    ///
    /// The resolved mnemonic lives under [`Self::in_curve`]'s `__IN_<ARG>` key, never under the bare
    /// arg name — `opts` carries an entry named for the arg only for Option and Text args
    /// (`workflow::build_opts`). Reading `o(arg)` here compiles, returns "" for every run, and
    /// silently disables the whole rule.
    pub(crate) fn input_is_class_curve(&self, arg: &str) -> bool {
        let name = self.in_curve(arg);
        if name.is_empty() {
            return false;
        }
        self.opts
            .get(CLASS_CURVES_OPT)
            .map(|s| s.split(',').any(|c| c.trim() == name))
            .unwrap_or(false)
    }

    /// The MNEMONIC resolved for log argument `arg`, upper-cased — the curve the run actually read,
    /// as opposed to `log(arg)`, which is its samples under the arg's own name.
    ///
    /// One accessor rather than a `format!("__IN_{arg}")` at each call site: the key is a private
    /// convention of `workflow::build_opts`, and a caller that spells it wrong gets an empty string
    /// rather than a compile error.
    pub(crate) fn in_curve(&self, arg: &str) -> String {
        self.o(&format!("__IN_{arg}")).trim().to_uppercase()
    }

    /// Declared unit of the concrete curve resolved for one input argument. The workflow owns
    /// source precedence because it owns the database connection; the module owns whether that
    /// declaration is mandatory for its arithmetic.
    pub(crate) fn input_unit(&self, arg: &str) -> &str {
        self.opts
            .get(&format!("{INPUT_UNIT_OPT_PREFIX}{arg}"))
            .map(String::as_str)
            .unwrap_or("")
    }

    pub(crate) fn log(&self, name: &str) -> Vec<f32> {
        self.logs.get(name).cloned().unwrap_or_else(|| vec![f32::NAN; self.n])
    }
    pub(crate) fn p(&self, name: &str, i: usize) -> f64 {
        record_defaulted_parameter(name, i);
        self.params.get(name).and_then(|v| v.get(i)).copied().unwrap_or(f64::NAN)
    }
    pub(crate) fn o(&self, name: &str) -> &str {
        record_defaulted_option(name);
        self.opts.get(name).map(|s| s.as_str()).unwrap_or("")
    }
}

pub type ModuleOutputs = HashMap<String, Vec<f32>>;

/// Controlled, durable vocabulary for a calculation that returned usable curves but did not
/// produce a clean result. These four members are the complete SB-DBM-039 vocabulary; adding a
/// fifth is a contract change, not an ad-hoc message choice.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunDegradationKind {
    Clamped,
    Defaulted,
    Truncated,
    SubstitutedInput,
}

/// Universal run policy for a source-bearing precondition. The default stays refusal: callers
/// must opt into retaining valid samples because that choice adds a companion curve and a
/// degraded run record.
pub(crate) const PRECONDITION_POLICY_OPT: &str = "__PRECONDITION_POLICY";
pub(crate) const PRECONDITION_POLICY_REFUSE: &str = "REFUSE";
pub(crate) const PRECONDITION_POLICY_FLAG_VALID_SAMPLES: &str = "FLAG_VALID_SAMPLES";
pub(crate) const PRECONDITION_FLAG_OUTPUT_KEY: &str = "__PRECONDITION_FLAG";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreconditionPolicy {
    Refuse,
    FlagValidSamples,
}

pub(crate) fn precondition_policy(
    opts: &HashMap<String, String>,
) -> Result<PreconditionPolicy, String> {
    let value = opts
        .get(PRECONDITION_POLICY_OPT)
        .map(|value| value.trim())
        .unwrap_or("");
    match value {
        "" | PRECONDITION_POLICY_REFUSE => Ok(PreconditionPolicy::Refuse),
        PRECONDITION_POLICY_FLAG_VALID_SAMPLES => Ok(PreconditionPolicy::FlagValidSamples),
        other => Err(format!(
            "precondition policy '{other}' is not recognized; choose {PRECONDITION_POLICY_REFUSE} or {PRECONDITION_POLICY_FLAG_VALID_SAMPLES}"
        )),
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct PreconditionAffectedSample {
    pub index: usize,
    pub offending_value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison_value: Option<f64>,
}

/// Structured runtime evidence for one declared condition. The complete affected-sample list is
/// retained in run provenance; the companion flag is the fast per-depth surface.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct PreconditionViolation {
    pub condition_id: String,
    pub argument: String,
    pub expected: String,
    pub source: String,
    pub statement: String,
    pub unit: String,
    pub affected_samples: Vec<PreconditionAffectedSample>,
}

impl PreconditionViolation {
    pub(crate) fn message(&self, module: &str) -> String {
        let first = self
            .affected_samples
            .first()
            .expect("a precondition violation always has an affected sample");
        let unit = if self.unit.is_empty() {
            String::new()
        } else {
            format!(" {}", self.unit)
        };
        let remainder = if self.affected_samples.len() == 1 {
            String::new()
        } else {
            format!(
                " ({} affected samples; the complete list is stored in run provenance)",
                self.affected_samples.len()
            )
        };
        format!(
            "precondition '{}' on '{}' flagged before {} ran: value {}{} at sample {} is outside {}{}. {} Source: {}",
            self.condition_id,
            self.argument,
            module,
            first.offending_value,
            unit,
            first.index,
            self.expected,
            remainder,
            self.statement,
            self.source
        )
    }
}

impl RunDegradationKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Clamped => "CLAMPED",
            Self::Defaulted => "DEFAULTED",
            Self::Truncated => "TRUNCATED",
            Self::SubstitutedInput => "SUBSTITUTED_INPUT",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "CLAMPED" => Some(Self::Clamped),
            "DEFAULTED" => Some(Self::Defaulted),
            "TRUNCATED" => Some(Self::Truncated),
            "SUBSTITUTED_INPUT" => Some(Self::SubstitutedInput),
            _ => None,
        }
    }
}

/// One structured reason a per-well result is degraded. `occurrences` aggregates repeated sample
/// events so a 100,000-row clamp does not become 100,000 provenance rows.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RunDegradation {
    pub kind: RunDegradationKind,
    pub detail: String,
    pub occurrences: usize,
}

impl RunDegradation {
    pub(crate) fn one(kind: RunDegradationKind, detail: impl Into<String>) -> Self {
        Self { kind, detail: detail.into(), occurrences: 1 }
    }
}

/// Provenance of defaults available to one module evaluation. Numeric defaults are sample-aware:
/// a named-zone or whole-well explicit override clears the flag only where that override applies.
#[derive(Debug, Clone, Default)]
pub(crate) struct DefaultUsage {
    pub(crate) parameter_samples: HashMap<String, Vec<bool>>,
    pub(crate) options: HashSet<String>,
}

#[derive(Default)]
struct DegradationCapture {
    events: BTreeMap<(RunDegradationKind, String), usize>,
    default_usage: DefaultUsage,
    defaulted_parameter_samples: HashSet<(String, usize)>,
    defaulted_options: HashSet<String>,
}

thread_local! {
    /// Rayon evaluates each well on one worker thread, so a thread-local capture keeps concurrent
    /// wells isolated without putting a callback on every `ModuleContext` literal in the scientific
    /// modules. The guard below always restores any prior capture, including on unwind.
    static DEGRADATION_CAPTURE: RefCell<Option<DegradationCapture>> = const { RefCell::new(None) };
}

struct DegradationCaptureGuard {
    previous: Option<DegradationCapture>,
    finished: bool,
}

impl DegradationCaptureGuard {
    fn start(default_usage: DefaultUsage) -> Self {
        let current = DegradationCapture { default_usage, ..Default::default() };
        let previous = DEGRADATION_CAPTURE.with(|slot| slot.replace(Some(current)));
        Self { previous, finished: false }
    }

    fn finish(mut self) -> Vec<RunDegradation> {
        let current = DEGRADATION_CAPTURE.with(|slot| slot.replace(self.previous.take()));
        self.finished = true;
        current
            .map(|capture| {
                capture
                    .events
                    .into_iter()
                    .map(|((kind, detail), occurrences)| RunDegradation {
                        kind,
                        detail,
                        occurrences,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Drop for DegradationCaptureGuard {
    fn drop(&mut self) {
        if !self.finished {
            DEGRADATION_CAPTURE.with(|slot| {
                slot.replace(self.previous.take());
            });
        }
    }
}

fn record_degradation(kind: RunDegradationKind, detail: impl Into<String>) {
    let detail = detail.into();
    DEGRADATION_CAPTURE.with(|slot| {
        if let Some(capture) = slot.borrow_mut().as_mut() {
            *capture.events.entry((kind, detail)).or_insert(0) += 1;
        }
    });
}

fn record_degradation_once(kind: RunDegradationKind, detail: impl Into<String>) {
    let detail = detail.into();
    DEGRADATION_CAPTURE.with(|slot| {
        if let Some(capture) = slot.borrow_mut().as_mut() {
            capture.events.entry((kind, detail)).or_insert(1);
        }
    });
}

fn record_defaulted_parameter(name: &str, sample: usize) {
    DEGRADATION_CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(capture) = slot.as_mut() else { return };
        let is_defaulted = capture
            .default_usage
            .parameter_samples
            .get(name)
            .and_then(|samples| samples.get(sample))
            .copied()
            .unwrap_or(false);
        if is_defaulted
            && capture
                .defaulted_parameter_samples
                .insert((name.to_string(), sample))
        {
            let detail = format!("parameter '{name}' used its sourced module-manifest default");
            *capture.events.entry((RunDegradationKind::Defaulted, detail)).or_insert(0) += 1;
        }
    });
}

fn record_defaulted_option(name: &str) {
    DEGRADATION_CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(capture) = slot.as_mut() else { return };
        if capture.default_usage.options.contains(name)
            && capture.defaulted_options.insert(name.to_string())
        {
            let detail = format!("option '{name}' used its module-manifest default");
            capture.events.insert((RunDegradationKind::Defaulted, detail), 1);
        }
    });
}

/// Evaluate one module while collecting only degradation events that actually occur on this
/// well. Callers that do not need the durable job/run contract keep using [`run_module`].
pub(crate) fn run_module_with_degradations(
    name: &str,
    ctx: &ModuleContext,
    default_usage: DefaultUsage,
) -> Result<
    (
        ModuleOutputs,
        Vec<RunDegradation>,
        Vec<PreconditionViolation>,
        Option<Vec<f32>>,
    ),
    String,
> {
    let capture = DegradationCaptureGuard::start(default_usage);
    let output = (|| {
        if precondition_policy(&ctx.opts)? == PreconditionPolicy::Refuse {
            return run_module(name, ctx).map(|outputs| (outputs, Vec::new(), None));
        }

        if let Some(message) = retired_module(name) {
            return Err(message.to_string());
        }
        let spec = module_catalog()
            .iter()
            .find(|module| module.name == name)
            .ok_or_else(|| format!("unknown module '{name}'"))?;
        let violations = collect_sample_precondition_violations(spec, ctx)?;
        if violations.is_empty() {
            return run_module(name, ctx)
                .map(|outputs| (outputs, Vec::new(), Some(vec![0.0; ctx.n])));
        }

        let affected: HashSet<usize> = violations
            .iter()
            .flat_map(|violation| {
                violation
                    .affected_samples
                    .iter()
                    .map(|sample| sample.index)
            })
            .collect();
        if affected.len() >= ctx.n {
            // With no unaffected sample there is no partial result to retain. Re-run the ordinary
            // validator so the refusal keeps the exact condition/value/range/source payload.
            return run_module(name, ctx).map(|outputs| (outputs, Vec::new(), None));
        }

        let mut sanitized = ctx.clone();
        for (name, values) in &mut sanitized.logs {
            if name != "DEPTH" {
                for index in &affected {
                    if let Some(value) = values.get_mut(*index) {
                        *value = f32::NAN;
                    }
                }
            }
        }
        for values in sanitized.params.values_mut() {
            for index in &affected {
                if let Some(value) = values.get_mut(*index) {
                    *value = f64::NAN;
                }
            }
        }

        validate_declared_preconditions_ignoring(spec, &sanitized, &affected)?;
        let mut outputs = dispatch_module(name, &sanitized)?;
        for values in outputs.values_mut() {
            for index in &affected {
                if let Some(value) = values.get_mut(*index) {
                    *value = f32::NAN;
                }
            }
        }
        validate_flag_outputs(spec, &outputs)?;
        let mut flag = FlagCurve::clear(ctx.n);
        for index in affected {
            flag.set(index, FlagValue::Flagged);
        }
        Ok((outputs, violations, Some(flag.into_f32())))
    })();
    let degradations = capture.finish();
    output.map(|(outputs, violations, flag)| (outputs, degradations, violations, flag))
}

/// The DECLARED output keys of `module` whose values are class identifiers rather than quantities
/// (`SB-MLA-055`). Everything not listed is a continuous curve.
///
/// Keyed by the manifest's declared output key, not by the written curve name — the user can rename
/// an output and add a prefix, and a rule that matched on `FACIES*` would lose the curve the moment
/// they did, while catching an unrelated `FACIES_CONFIDENCE` that is not a class at all.
///
/// `gmm_facies` is the case that makes the distinction load-bearing: it writes a class curve AND a
/// probability curve in the same run. `FPROB` is an ordinary continuous quantity and must stay
/// averageable — a per-module flag, or a name prefix, would wrongly protect it and the user would
/// find their probability curve resampled by MODE.
pub(crate) fn class_outputs(module: &str) -> &'static [&'static str] {
    match module {
        "electrofacies" => &["FACIES"],
        "gmm_facies" => &["FACIES_GMM"],
        "sw_arch" | "sw_indo" | "sw_sim" => &["SW_METHOD"],
        _ => &[],
    }
}

fn limit(v: f64, lo: f64, hi: f64) -> f64 {
    // `f64::clamp` panics when `lo > hi` or either bound is NaN, and the bounds here are module
    // PARAMETERS — a zone override or an unbounded Monte Carlo draw can push one past the other
    // (e.g. SWT_IRR entered as 25 meaning percent gives `clamp(25.0, 1.0)`). The real enforcement
    // is in `workflow::resolve_param_arrays`, which now rejects out-of-spec values; this is the
    // backstop so a future module cannot reintroduce the panic. Release builds set
    // `panic = "abort"`, so an unguarded clamp takes the whole app down rather than failing a run.
    if v.is_nan() || !(lo <= hi) {
        f64::NAN
    } else {
        let limited = v.clamp(lo, hi);
        if limited != v {
            record_degradation(
                RunDegradationKind::Clamped,
                format!("calculated value was clamped to the existing range [{lo}, {hi}]"),
            );
        }
        limited
    }
}

const MISSING: f64 = f64::NAN;

/// Lower bound on the LIMITED effective porosity every porosity module writes as `PHIE`
/// (Jauhar, 2026-08-01: "always limit phie to 0.001" — docs/review_triage.md finding 16).
///
/// A shale-corrected density porosity reads slightly NEGATIVE over a tight streak — a dense
/// carbonate stringer on a sandstone matrix is the ordinary case, not a corrupt curve. Nothing
/// downstream treats that as an error, so the negative volume propagates: the pay summary sums
/// `PHIE·(1−SWE)·h` over net, and the streak's contribution is SUBTRACTED from the zone's
/// hydrocarbon column. Measured, that took a SAND row's HPV more than 20 % below the floored
/// answer while RESERVOIR and PAY stayed byte-identical — the two rows anyone checks first agreed
/// with each other while the third quietly did not.
///
/// **0.001 v/v rather than 0.0**, which is his call and not an arbitrary epsilon: a hard zero is a
/// legitimate reading (shale has no effective porosity, and the ≥95 % VSH branch says so), so
/// flooring at zero would make "no porosity here" and "the arithmetic went below zero"
/// indistinguishable. 0.1 pu is below anything a log can resolve and above anything a physical
/// interpretation would claim, so a PHIE sitting exactly on it is legible as the floor.
///
/// **The floor lands on `PHIE` only.** `PHIE_DEN` / `PHIE_DN` are the declared UNLIMITED twins and
/// stay unclamped, so the negative excursion is still there to be plotted when the question is
/// whether the matrix density is right. Clamping those too would hide the evidence for the very
/// judgement the curve exists to support.
pub(crate) const PHIE_FLOOR: f64 = 0.001;

fn is_missing(v: f64) -> bool {
    v.is_nan()
}

/// Apply the SB-CLY-043 contracts by module argument identity, never by the curve name selected at
/// run time. This inventory is intentionally explicit: a role called `VCLAY` requires VCL while a
/// role called `VSH` requires VSH, and neither decision can be recovered safely from spelling or
/// from the shared `v/v` unit. A dual-type role must be added here only when its owning
/// specification actually declares one.
fn apply_shale_clay_quantity_contracts(modules: &mut [ModuleSpec]) -> Result<(), String> {
    fn argument_mut<'a>(
        modules: &'a mut [ModuleSpec],
        module: &str,
        argument: &str,
    ) -> Result<&'a mut ArgSpec, String> {
        modules
            .iter_mut()
            .find(|spec| spec.name == module)
            .and_then(|spec| spec.args.iter_mut().find(|arg| arg.name == argument))
            .ok_or_else(|| {
                format!(
                    "SB-CLY-043 quantity inventory names missing argument '{module}.{argument}'"
                )
            })
    }

    const VSH_INPUTS: &[(&str, &str)] = &[
        ("phi_den", "VSH"),
        ("phi_dn", "VSH"),
        ("phi_son", "VSH"),
        ("sspw", "VSH"),
        ("sw_indo", "VSH"),
        ("sw_sim", "VSH"),
        ("thin_bed_ts", "VSH"),
        ("rt_cutoff", "VSH"),
    ];
    const VSH_OUTPUTS: &[(&str, &str)] = &[
        ("vsh_gr", "VSH_GR"),
        ("vsh_gr", "VSH"),
        ("vsh_dn", "VSH_DN"),
        ("vsh_dn", "VSH"),
        ("ssc", "VSH_SSC"),
        ("ssc", "VSHGR"),
        ("ssc", "VSHND"),
        ("multimin", "VSH_MM"),
    ];
    const VCL_OUTPUTS: &[(&str, &str)] = &[("multimin", "VOL_CLAY")];

    for (module, argument) in VSH_INPUTS {
        let arg = argument_mut(modules, module, argument)?;
        if arg.kind != ArgKind::LogIn {
            return Err(format!("SB-CLY-043 input contract '{module}.{argument}' is not a LogIn"));
        }
        arg.accepted_shale_clay_quantities = vec![ShaleClayQuantity::ShaleVolume];
    }
    for (module, argument) in VSH_OUTPUTS {
        let arg = argument_mut(modules, module, argument)?;
        if arg.kind != ArgKind::LogOut {
            return Err(format!("SB-CLY-043 output contract '{module}.{argument}' is not a LogOut"));
        }
        arg.output_shale_clay_quantity = Some(ShaleClayQuantity::ShaleVolume);
    }
    for (module, argument) in VCL_OUTPUTS {
        let arg = argument_mut(modules, module, argument)?;
        if arg.kind != ArgKind::LogOut {
            return Err(format!("SB-CLY-043 output contract '{module}.{argument}' is not a LogOut"));
        }
        arg.output_shale_clay_quantity = Some(ShaleClayQuantity::ClayVolume);
    }

    let clay = argument_mut(modules, "brittleness", "VCLAY")?;
    if clay.kind != ArgKind::LogIn {
        return Err("SB-CLY-043 clay quantity contract 'brittleness.VCLAY' is not a LogIn".into());
    }
    clay.accepted_shale_clay_quantities = vec![ShaleClayQuantity::ClayVolume];

    for module in modules {
        for arg in &module.args {
            if !arg.accepted_shale_clay_quantities.is_empty() && arg.kind != ArgKind::LogIn {
                return Err(format!(
                    "SB-CLY-043 accepted quantity contract '{}.{}' is not attached to a LogIn",
                    module.name, arg.name
                ));
            }
            if arg.output_shale_clay_quantity.is_some() && arg.kind != ArgKind::LogOut {
                return Err(format!(
                    "SB-CLY-043 output quantity contract '{}.{}' is not attached to a LogOut",
                    module.name, arg.name
                ));
            }
            let distinct = arg
                .accepted_shale_clay_quantities
                .iter()
                .copied()
                .collect::<HashSet<_>>();
            if distinct.len() != arg.accepted_shale_clay_quantities.len() {
                return Err(format!(
                    "SB-CLY-043 quantity contract '{}.{}' repeats an accepted quantity",
                    module.name, arg.name
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct PorosityOutputRegistration {
    argument: &'static str,
    role: PorosityOutputRole,
    convention: &'static str,
}

#[derive(Clone, Copy)]
struct PorosityModuleRegistration {
    module: &'static str,
    role: PorosityModuleRole,
    method: &'static str,
    limiting_policy: &'static str,
    limiting_policy_source: &'static str,
    outputs: &'static [PorosityOutputRegistration],
}

const fn porosity_output(
    argument: &'static str,
    role: PorosityOutputRole,
    convention: &'static str,
) -> PorosityOutputRegistration {
    PorosityOutputRegistration {
        argument,
        role,
        convention,
    }
}

const PHI_DEN_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output(
        "PHIE_DEN",
        PorosityOutputRole::UnlimitedEffective,
        "DENSITY_SHALE_SUBTRACTIVE_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIT_DEN",
        PorosityOutputRole::UnlimitedTotal,
        "DENSITY_SHALE_SUBTRACTIVE_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIE",
        PorosityOutputRole::LimitedEffective,
        "DENSITY_SHALE_SUBTRACTIVE_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIT",
        PorosityOutputRole::LimitedTotal,
        "DENSITY_SHALE_SUBTRACTIVE_WITH_TOTAL_REBUILD",
    ),
];

const PHI_DN_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output(
        "PHIE_DN",
        PorosityOutputRole::ComparisonUnlimitedEffective,
        "SHALE_REDUCED_COMPARISON_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIT_DN",
        PorosityOutputRole::ComparisonUnlimitedTotal,
        "SHALE_REDUCED_COMPARISON_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIE",
        PorosityOutputRole::ComparisonLimitedEffective,
        "SHALE_REDUCED_COMPARISON_WITH_TOTAL_REBUILD",
    ),
    porosity_output(
        "PHIT",
        PorosityOutputRole::ComparisonLimitedTotal,
        "SHALE_REDUCED_COMPARISON_WITH_TOTAL_REBUILD",
    ),
];

const PHI_SON_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output(
        "PHIT_SON",
        PorosityOutputRole::LimitedTotal,
        "CURRENT_MIXED_SONIC_PENDING_SB_POR_013",
    ),
    porosity_output(
        "PHIE_SON",
        PorosityOutputRole::LimitedEffective,
        "CURRENT_MIXED_SONIC_PENDING_SB_POR_013",
    ),
];

const PHIMAX_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output("PHI_CAP", PorosityOutputRole::Capped, "COMPACTION_CEILING"),
    porosity_output("PHI_MAX", PorosityOutputRole::Ceiling, "COMPACTION_CEILING"),
];

const SSC_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output("PHIT_SSC", PorosityOutputRole::Total, "SSC_BOUND_WATER_SPLIT"),
    porosity_output("PHIE_SSC", PorosityOutputRole::Effective, "SSC_BOUND_WATER_SPLIT"),
    porosity_output("PHIFF_SSC", PorosityOutputRole::FreeFluid, "SSC_BOUND_WATER_SPLIT"),
    porosity_output("PHIFF_GR", PorosityOutputRole::FreeFluid, "SSC_GR_EQUIVALENT"),
    porosity_output("PHIE_GR", PorosityOutputRole::Effective, "SSC_GR_EQUIVALENT"),
    porosity_output("PHIT_GR", PorosityOutputRole::Total, "SSC_GR_EQUIVALENT"),
];

const SSPW_POROSITY_OUTPUTS: &[PorosityOutputRegistration] = &[
    porosity_output("PHIT_SSPW", PorosityOutputRole::Total, "SSPW_BOUND_WATER_SPLIT"),
    porosity_output("PHIE_SSPW", PorosityOutputRole::Effective, "SSPW_BOUND_WATER_SPLIT"),
    porosity_output("PHIFF_SSPW", PorosityOutputRole::FreeFluid, "SSPW_BOUND_WATER_SPLIT"),
];

const POROSITY_MODULE_REGISTRATIONS: &[PorosityModuleRegistration] = &[
    PorosityModuleRegistration {
        module: "phi_den",
        role: PorosityModuleRole::DeterministicMethod,
        method: "DENSITY",
        limiting_policy: "phi_den_effective_floor_and_selected_ceiling",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §5 porosity limits and DEC-015",
        outputs: PHI_DEN_POROSITY_OUTPUTS,
    },
    PorosityModuleRegistration {
        module: "phi_dn",
        role: PorosityModuleRole::ComparisonProducer,
        method: "DENSITY_NEUTRON_COMPARISON",
        limiting_policy: "phi_dn_effective_floor_and_selected_ceiling",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §5 porosity limits and DEC-015",
        outputs: PHI_DN_POROSITY_OUTPUTS,
    },
    PorosityModuleRegistration {
        module: "phi_son",
        role: PorosityModuleRole::DeterministicMethod,
        method: "SONIC_CURRENT_WYLLIE_OR_LEGACY_RHG_TOKEN",
        limiting_policy: "phi_son_unit_interval",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §§3.3, 5.2 and DEC-015",
        outputs: PHI_SON_POROSITY_OUTPUTS,
    },
    PorosityModuleRegistration {
        module: "phimax",
        role: PorosityModuleRole::LimitProducer,
        method: "POROSITY_COMPACTION_CEILING",
        limiting_policy: "phimax_compaction_ceiling",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters and DEC-015",
        outputs: PHIMAX_POROSITY_OUTPUTS,
    },
    PorosityModuleRegistration {
        module: "ssc",
        role: PorosityModuleRole::DeterministicMethod,
        method: "SAND_SILT_CLAY",
        limiting_policy: "ssc_component_and_total_bounds",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §3.8 and DEC-015",
        outputs: SSC_POROSITY_OUTPUTS,
    },
    PorosityModuleRegistration {
        module: "sspw",
        role: PorosityModuleRole::DeterministicMethod,
        method: "SANDSTONE_WORKFLOW_RECONSTRUCTION",
        limiting_policy: "sspw_component_and_total_bounds",
        limiting_policy_source: "docs/PRD_v2/11_porosity.md §3.8 and DEC-015",
        outputs: SSPW_POROSITY_OUTPUTS,
    },
];

fn porosity_contract(
    registration: PorosityModuleRegistration,
    output: PorosityOutputRegistration,
) -> PorosityOutputContract {
    PorosityOutputContract {
        family: POROSITY_FAMILY_ID.into(),
        module_role: registration.role,
        method: registration.method.into(),
        convention: output.convention.into(),
        output_role: output.role,
        limiting_contract: POROSITY_LIMITING_CONTRACT.into(),
        limiting_policy: registration.limiting_policy.into(),
        limiting_policy_source: registration.limiting_policy_source.into(),
        flag_contract: POROSITY_FLAG_CONTRACT.into(),
        flag_emission: PorosityFlagEmission::PendingSbPor003,
        output_naming_contract: POROSITY_OUTPUT_NAMING_CONTRACT.into(),
    }
}

fn apply_porosity_contracts(modules: &mut [ModuleSpec]) -> Result<(), String> {
    for registration in POROSITY_MODULE_REGISTRATIONS.iter().copied() {
        let module = modules
            .iter_mut()
            .find(|module| module.name == registration.module)
            .ok_or_else(|| {
                format!(
                    "SB-POR-001 registry names missing module '{}'",
                    registration.module
                )
            })?;
        for output in registration.outputs.iter().copied() {
            let argument = module
                .args
                .iter_mut()
                .find(|argument| argument.name == output.argument)
                .ok_or_else(|| {
                    format!(
                        "SB-POR-001 registry names missing output '{}.{}'",
                        registration.module, output.argument
                    )
                })?;
            if argument.kind != ArgKind::LogOut {
                return Err(format!(
                    "SB-POR-001 contract '{}.{}' is not a LogOut",
                    registration.module, output.argument
                ));
            }
            if argument.porosity_output.is_some() {
                return Err(format!(
                    "SB-POR-001 contract '{}.{}' is registered more than once",
                    registration.module, output.argument
                ));
            }
            argument.porosity_output = Some(porosity_contract(registration, output));
        }
    }
    validate_porosity_contracts(modules)
}

fn validate_porosity_contracts(modules: &[ModuleSpec]) -> Result<(), String> {
    let expected_modules = POROSITY_MODULE_REGISTRATIONS
        .iter()
        .map(|registration| registration.module)
        .collect::<HashSet<_>>();
    let actual_modules = modules
        .iter()
        .filter(|module| module.category == "Porosity")
        .map(|module| module.name.as_str())
        .collect::<HashSet<_>>();
    let mut failures = Vec::new();
    if actual_modules != expected_modules {
        failures.push(format!(
            "live Porosity modules {:?} do not match registered family {:?}",
            actual_modules, expected_modules
        ));
    }

    let mut actual_method_policies = HashSet::new();
    for module in modules {
        let registration = POROSITY_MODULE_REGISTRATIONS
            .iter()
            .copied()
            .find(|registration| registration.module == module.name);
        if registration.is_none() {
            for argument in module
                .args
                .iter()
                .filter(|argument| argument.porosity_output.is_some())
            {
                failures.push(format!(
                    "{}.{} carries POR metadata outside the Porosity family",
                    module.name, argument.name
                ));
            }
            continue;
        }
        let registration = registration.unwrap();
        if module.category != "Porosity" {
            failures.push(format!(
                "registered POR module '{}' has category '{}'",
                module.name, module.category
            ));
        }

        let expected_outputs = registration
            .outputs
            .iter()
            .map(|output| output.argument)
            .collect::<HashSet<_>>();
        for argument in module.args.iter().filter(|argument| {
            argument.porosity_output.is_some() || expected_outputs.contains(argument.name.as_str())
        }) {
            let Some(output) = registration
                .outputs
                .iter()
                .copied()
                .find(|output| output.argument == argument.name)
            else {
                failures.push(format!(
                    "{}.{} carries POR metadata but is absent from its module policy",
                    module.name, argument.name
                ));
                continue;
            };
            let identity = format!("{}.{}", module.name, argument.name);
            let Some(contract) = argument.porosity_output.as_ref() else {
                failures.push(format!("{identity} is missing its POR output contract"));
                continue;
            };
            if argument.kind != ArgKind::LogOut || argument.unit != "v/v" {
                failures.push(format!(
                    "{identity} must be a v/v LogOut to carry POR output custody"
                ));
            }
            let expected = porosity_contract(registration, output);
            if contract.limiting_policy != expected.limiting_policy {
                failures.push(format!(
                    "{identity} borrows or misstates limit policy '{}' instead of its registered '{}'",
                    contract.limiting_policy, expected.limiting_policy
                ));
            }
            if contract != &expected {
                failures.push(format!(
                    "{identity} does not match the common POR envelope and registered method policy"
                ));
            }
            if registration.role != PorosityModuleRole::LimitProducer {
                actual_method_policies.insert(contract.limiting_policy.as_str());
            }
        }
    }

    let result_producer_count = POROSITY_MODULE_REGISTRATIONS
        .iter()
        .filter(|registration| registration.role != PorosityModuleRole::LimitProducer)
        .count();
    if actual_method_policies.len() != result_producer_count {
        failures.push(format!(
            "one porosity result producer borrows another producer's limit policy: {} distinct policies for {result_producer_count} producers",
            actual_method_policies.len()
        ));
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-POR-001 common-envelope registry gate failed: {}",
            failures.join("; ")
        ))
    }
}

/// Immutable registry of every deterministic module manifest, in workflow order. Monte Carlo and
/// batch chains call `run_module` thousands of times, so rebuilding every manifest at each public
/// dispatch would turn central validation into an avoidable per-realization cost.
fn module_catalog() -> &'static [ModuleSpec] {
    static CATALOG: OnceLock<Vec<ModuleSpec>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut modules = vec![
            vsh_gr_spec(),
            vsh_dn_spec(),
            phi_den_spec(),
            phi_dn_spec(),
            phi_son_spec(),
            phimax_spec(),
            crate::ssc::ssc_spec(),
            crate::ssc::sspw_spec(),
            ftemp_grad_spec(),
            precalc_spec(),
            badhole_spec(),
            condflag_spec(),
            nphimat_spec(),
            gascorr_spec(),
            gr_hole_corr_spec(),
            nphi_env_corr_spec(),
            rhob_hole_corr_spec(),
            gr_normalize_spec(),
            log_predict_spec(),
            sw_arch_spec(),
            sw_indo_spec(),
            sw_sim_spec(),
            crate::lrlc::sw_rtc_spec(),
            crate::lrlc::sw_imts_spec(),
            perm_wyllie_rose_spec(),
            perm_coates_spec(),
            perm_transform_spec(),
            thin_bed_ts_spec(),
            depth_shift_spec(),
            splice_spec(),
            crate::condition::despike_spec(),
            crate::condition::smooth_spec(),
            crate::condition::clip_spec(),
            crate::condition::fill_gaps_spec(),
            crate::condition::flip_spec(),
            crate::condition::normalize_spec(),
            crate::frame::block_spec(),
            crate::frame::bed_detect_spec(),
            crate::multimin::multimin_spec(),
            crate::satheight::sw_height_spec(),
            crate::lithology::midplot_spec(),
            crate::rocktyping::rocktyping_spec(),
            crate::rocktyping::lucia_rfn_spec(),
            crate::rocktyping::pittman_rx_spec(),
            crate::rocktyping::rt_cutoff_spec(),
            crate::facies::electrofacies_spec(),
            crate::facies::gmm_facies_spec(),
            crate::unconventional::toc_passey_spec(),
            crate::unconventional::kerogen_spec(),
            crate::unconventional::gip_spec(),
            crate::unconventional::brittleness_spec(),
        ];
        apply_shale_clay_quantity_contracts(&mut modules).unwrap_or_else(|error| panic!("{error}"));
        apply_porosity_contracts(&mut modules).unwrap_or_else(|error| panic!("{error}"));
        validate_parameter_sources(&modules).unwrap_or_else(|error| panic!("{error}"));
        // SB-CUT-017: the same gate for defaults that are NOT module parameters.
        crate::param_sources::validate_domain_defaults(
            crate::param_sources::CUT_DOMAIN_DEFAULTS,
        )
        .unwrap_or_else(|error| panic!("{error}"));
        validate_clay_unit_contract(&modules).unwrap_or_else(|error| panic!("{error}"));
        validate_flag_declarations(&modules).unwrap_or_else(|error| panic!("{error}"));
        validate_project_depth_unit_tokens(&modules).unwrap_or_else(|error| panic!("{error}"));
        modules
    })
}

/// Registry snapshot returned over IPC. Callers own the serialized copy; execution reads the
/// immutable catalog directly through [`module_catalog`].
pub fn list_modules() -> Vec<ModuleSpec> {
    module_catalog().to_vec()
}

/// Modules retired from the compute path: still returned by `list_modules` (so a saved chain step
/// or a `module:<name>` dockview panel resolves by name and can explain itself), but no longer
/// runnable — `run_module` blocks them with this message instead of silently running superseded
/// physics. Adding a name here is the whole retirement; there is no per-spec flag to thread
/// through the ~40 module literals.
pub(crate) fn retired_module(name: &str) -> Option<&'static str> {
    match name {
        "multimin" => Some(
            "The Multimin module is retired — its fixed 4-component inversion is superseded by \
             SandiMin. Re-run this step with SandiMin (Advance ▸ Mineral Solver).",
        ),
        _ => None,
    }
}

/// Registry build gate for SB-CORE-004. Numeric defaults are admissible only when their own
/// machine-readable source is present; a deliberately absent default uses the exact `ABSENT`
/// token and must not carry a concealed number. SB-CLY-051 additionally requires each shipping
/// CLY default to name a checkable artefact rather than a product label.
pub(crate) fn source_identifies_checkable_artefact(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    let file_or_record = [
        ".info", ".lls", ".html", ".htm", ".json", ".xml", ".md", ".pdf", "doi:",
        "isbn",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || source.contains('/')
        || source.contains('\\')
        || source.contains('§');
    let named_publication = source.split_whitespace().count() >= 2
        && source.split(|character: char| !character.is_ascii_digit())
            .any(|digits| digits.len() == 4 && digits.starts_with(['1', '2']));
    file_or_record || named_publication
}

pub(crate) fn validate_parameter_sources(modules: &[ModuleSpec]) -> Result<(), String> {
    let mut failures = Vec::new();
    for module in modules {
        for arg in &module.args {
            if arg.kind != ArgKind::Param {
                continue;
            }
            let identity = format!("{}.{}", module.name, arg.name);
            let source = arg.default_source.trim();
            if source.is_empty() {
                failures.push(format!(
                    "{identity} has default '{}' but no source",
                    arg.default
                ));
                continue;
            }
            if source == ABSENT_DEFAULT_SOURCE {
                if !arg.default.is_empty() {
                    failures.push(format!(
                        "{identity} declares source ABSENT but still ships default '{}'",
                        arg.default
                    ));
                }
                continue;
            }
            // SB-SAT-038 extends the checkable-artefact rule from VSH to Saturation. The chapter
            // requires a source to be "a file and section, a module and parameter name, or a full
            // literature citation" — a product name alone is not one. The domain's own evidence is
            // the argument: three vendors ship three `Rw` defaults, three `B` method defaults, two
            // `vQ0` values from the same paper, and a Simandoux `a` no cited paper supports — and
            // none of them tells the user.
            if matches!(module.category.as_str(), "VSH" | "Saturation")
                && !source_identifies_checkable_artefact(source)
            {
                failures.push(format!(
                    "{identity} source '{source}' does not identify a checkable artefact locator, named publication, or project record; a product name alone is not a source"
                ));
                continue;
            }
            match arg.default.parse::<f64>() {
                Ok(value) if value.is_finite() => {}
                _ => failures.push(format!(
                    "{identity} cites source '{source}' but has no finite numeric default"
                )),
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-CORE-004 parameter-source build gate failed ({} violation{}): {}",
            failures.len(),
            if failures.len() == 1 { "" } else { "s" },
            failures.join("; ")
        ))
    }
}

/// Fail the immutable registry build if a CLY quantity has no typed unit, or if a numeric default
/// can diverge from its source-unit conversion. This is intentionally scoped to CLY: SB-CLY-054
/// does not authorize retrofitting unrelated domain manifests in the same increment.
fn validate_clay_unit_contract(modules: &[ModuleSpec]) -> Result<(), String> {
    let mut failures = Vec::new();
    for module in modules.iter().filter(|module| module.category == "VSH") {
        for argument in module.args.iter().filter(|argument| {
            matches!(argument.kind, ArgKind::Param | ArgKind::LogIn | ArgKind::LogOut)
        }) {
            let identity = format!("{}.{}", module.name, argument.name);
            if crate::curves::resolve_unit_token(&argument.unit).is_none() {
                failures.push(format!(
                    "{identity} has unregistered unit token '{}'",
                    argument.unit
                ));
                continue;
            }
            if argument.kind != ArgKind::Param {
                continue;
            }
            let Ok(default) = argument.default.parse::<f64>() else {
                if argument.default_unit_custody.is_some() {
                    failures.push(format!(
                        "{identity} has no numeric default but still carries default unit custody"
                    ));
                }
                continue;
            };
            let Some(custody) = argument.default_unit_custody.as_ref() else {
                failures.push(format!(
                    "{identity} ships numeric default {default} {} without artefact-unit custody",
                    argument.unit
                ));
                continue;
            };
            let expected = match ParameterUnitCustody::new(
                custody.artefact_value,
                &custody.artefact_unit,
                &argument.unit,
            ) {
                Ok(expected) => expected,
                Err(error) => {
                    failures.push(format!("{identity} has invalid unit custody: {error}"));
                    continue;
                }
            };
            if custody != &expected {
                failures.push(format!(
                    "{identity} unit custody does not match the named registry conversion"
                ));
            }
            if custody.canonical_unit != argument.unit
                || (custody.canonical_value - default).abs() > 1e-12
            {
                failures.push(format!(
                    "{identity} default {default} {} differs from converted custody value {} {}",
                    argument.unit, custody.canonical_value, custody.canonical_unit
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid CLY unit contract: {}", failures.join("; ")))
    }
}

/// Resolve legacy vendor option tokens to the equation identity persisted by a new run.
///
/// `MODIFIED` is Geolog's name for the Bardon-Pied equation while IP uses "Modified
/// Simandoux" for the Schlumberger equation. Accepting those tokens at the input boundary keeps
/// saved chains runnable; returning only the equation ids prevents that ambiguity from entering
/// new provenance, manifests, or module arithmetic.
pub(crate) fn canonical_option_value(module: &str, argument: &str, value: &str) -> String {
    let trimmed = value.trim();
    match (module, argument, trimmed) {
        ("sw_sim", "OPT_SIM", "MODIFIED" | "SIM_MOD") => "simandoux_bardon_pied".into(),
        ("sw_sim", "OPT_SIM", "SCHLUMBERGER" | "SCHLUM" | "SIM_SCHL") => {
            "simandoux_modified_slb".into()
        }
        _ => trimmed.to_string(),
    }
}

/// Registry build gate for SB-ENV-057. A numeric parameter may name a fixed unit (`m`, `ft`)
/// when its implementation performs an explicit conversion, or use the single project-native
/// token. Ambiguous union spellings cannot say which unit a supplied number is in and are refused.
fn validate_project_depth_unit_tokens(modules: &[ModuleSpec]) -> Result<(), String> {
    let invalid = modules
        .iter()
        .flat_map(|module| {
            module.args.iter().filter_map(move |argument| {
                (argument.kind == ArgKind::Param
                    && matches!(argument.unit.as_str(), "m|ft" | "ft|m"))
                .then(|| format!("{}.{}={}", module.name, argument.name, argument.unit))
            })
        })
        .collect::<Vec<_>>();
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-ENV-057 project-depth-unit registry gate failed: use '{PROJECT_DEPTH_UNIT_TOKEN}' for project-native lengths or one explicit fixed unit with conversion; invalid declarations: {}",
            invalid.join(", ")
        ))
    }
}

/// Registry build gate for SB-ENV-030. A semantic flag role belongs only to an output channel;
/// attaching it to a parameter, option, text field, or input would make the IPC metadata lie.
fn validate_flag_declarations(modules: &[ModuleSpec]) -> Result<(), String> {
    let invalid = modules
        .iter()
        .flat_map(|module| {
            module.args.iter().filter_map(move |arg| {
                (arg.flag_kind.is_some() && arg.kind != ArgKind::LogOut)
                    .then(|| format!("{}.{}", module.name, arg.name))
            })
        })
        .collect::<Vec<_>>();
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-ENV-030 flag-kind registry gate failed: only LogOut arguments may declare a flag kind; invalid declarations: {}",
            invalid.join(", ")
        ))
    }
}

/// Validate the wire/storage representation at the boundary where a typed flag becomes f32.
/// Optional flag outputs may be absent, but an emitted declared flag may contain only MISSING,
/// CLEAR, or FLAGGED from [`FlagValue`].
fn validate_flag_outputs(
    spec: &ModuleSpec,
    outputs: &ModuleOutputs,
) -> Result<(), String> {
    for arg in spec.args.iter().filter(|arg| arg.flag_kind.is_some()) {
        if let Some(values) = outputs.get(&arg.name) {
            FlagCurve::validate_f32(values, &format!("{}.{}", spec.name, arg.name))?;
        }
    }
    Ok(())
}

fn format_numeric_range(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(lo), Some(hi)) => format!("{lo} to {hi}"),
        (Some(lo), None) => format!(">= {lo}"),
        (None, Some(hi)) => format!("<= {hi}"),
        (None, None) => "a finite value".to_string(),
    }
}

fn selected_value(spec: &ModuleSpec, ctx: &ModuleContext, name: &str) -> String {
    let default = spec
        .args
        .iter()
        .find(|arg| arg.name == name)
        .map(|arg| arg.default.as_str())
        .unwrap_or("");
    canonical_option_value(
        &spec.name,
        name,
        ctx.opts.get(name).map(String::as_str).unwrap_or(default),
    )
}

fn numeric_value_at(
    spec: &ModuleSpec,
    ctx: &ModuleContext,
    name: &str,
    index: usize,
) -> Option<f64> {
    let arg = spec.args.iter().find(|arg| arg.name == name)?;
    match arg.kind {
        ArgKind::Param => ctx.params.get(name)?.get(index).copied(),
        ArgKind::LogIn => ctx.logs.get(name)?.get(index).map(|value| *value as f64),
        _ => None,
    }
}

fn condition_is_active(spec: &ModuleSpec, ctx: &ModuleContext, rule: &ValidityRule) -> bool {
    let when = match rule {
        ValidityRule::NumericRange { when, .. }
        | ValidityRule::RequiredCompanion { when, .. }
        | ValidityRule::RequiredValue { when } => when.as_ref(),
        _ => None,
    };
    when.map_or(true, |branch| {
        selected_value(spec, ctx, &branch.argument) == branch.equals
    })
}

/// Collect only conditions whose offending samples can be isolated without discarding the
/// unaffected interval. Whole-run conditions (method ids, required companions/values) remain
/// refusals because no per-sample repair can make them true.
fn collect_sample_precondition_violations(
    spec: &ModuleSpec,
    ctx: &ModuleContext,
) -> Result<Vec<PreconditionViolation>, String> {
    let mut violations = Vec::new();
    for arg in &spec.args {
        for condition in &arg.validity_conditions {
            if condition.id.trim().is_empty()
                || condition.statement.trim().is_empty()
                || condition.source.trim().is_empty()
            {
                return Err(format!(
                    "module '{}' has an invalid validity manifest on '{}': every condition needs a stable id, statement and source",
                    spec.name, arg.name
                ));
            }
            if !condition_is_active(spec, ctx, &condition.rule) {
                continue;
            }
            let (expected, unit, affected_samples) = match &condition.rule {
                ValidityRule::NumericRange { min, max, unit, .. } => {
                    let affected = (0..ctx.n)
                        .filter_map(|index| {
                            let value = numeric_value_at(spec, ctx, &arg.name, index)?;
                            (value.is_finite()
                                && (min.is_some_and(|lo| value < lo)
                                    || max.is_some_and(|hi| value > hi)))
                            .then_some(PreconditionAffectedSample {
                                index,
                                offending_value: value,
                                comparison_value: None,
                            })
                        })
                        .collect::<Vec<_>>();
                    let suffix = if unit.is_empty() {
                        String::new()
                    } else {
                        format!(" {unit}")
                    };
                    (format!("{}{}", format_numeric_range(*min, *max), suffix), unit.clone(), affected)
                }
                ValidityRule::LessThan { other } => {
                    if !spec.args.iter().any(|candidate| candidate.name == *other) {
                        return Err(format!(
                            "module '{}' has an invalid validity manifest: '{}' names unknown comparison argument '{}'",
                            spec.name, condition.id, other
                        ));
                    }
                    let affected = (0..ctx.n)
                        .filter_map(|index| {
                            let value = numeric_value_at(spec, ctx, &arg.name, index)?;
                            let other_value = numeric_value_at(spec, ctx, other, index)?;
                            (value.is_finite() && other_value.is_finite() && value >= other_value)
                                .then_some(PreconditionAffectedSample {
                                    index,
                                    offending_value: value,
                                    comparison_value: Some(other_value),
                                })
                        })
                        .collect::<Vec<_>>();
                    (
                        format!("less than '{other}' at the same sample"),
                        arg.unit.clone(),
                        affected,
                    )
                }
                _ => continue,
            };
            if !affected_samples.is_empty() {
                violations.push(PreconditionViolation {
                    condition_id: condition.id.clone(),
                    argument: arg.name.clone(),
                    expected,
                    source: condition.source.clone(),
                    statement: condition.statement.clone(),
                    unit,
                    affected_samples,
                });
            }
        }
    }
    Ok(violations)
}

/// Enforce the validity conditions already declared by a module manifest before its body runs.
///
/// This deliberately lives at the public dispatch boundary rather than in the dialog or in one
/// workflow caller. Saved chains, Monte Carlo, batch execution and future API callers all reach
/// [`run_module`], so none can turn an unknown option into a module body's `_ => default` arm or
/// feed an out-of-range zone array to arithmetic that returns a plausible number.
fn validate_declared_preconditions(spec: &ModuleSpec, ctx: &ModuleContext) -> Result<(), String> {
    validate_declared_preconditions_ignoring(spec, ctx, &HashSet::new())
}

fn validate_option_value(spec: &ModuleSpec, arg: &ArgSpec, value: &str) -> Result<(), String> {
    if value.is_empty() && !arg.required {
        return Ok(());
    }
    let mut has_sourced_enumeration = false;
    for condition in arg
        .validity_conditions
        .iter()
        .filter(|condition| matches!(condition.rule, ValidityRule::Enumeration))
    {
        has_sourced_enumeration = true;
        if condition.id.trim().is_empty()
            || condition.statement.trim().is_empty()
            || condition.source.trim().is_empty()
        {
            return Err(format!(
                "module '{}' has an invalid validity manifest on '{}': every condition needs a stable id, statement and source",
                spec.name, arg.name
            ));
        }
        if value.is_empty() || !arg.choices.iter().any(|choice| choice == value) {
            return Err(format!(
                "precondition '{}' on '{}' failed before {} ran: value '{}' is not in the permitted set [{}]. {} Source: {}",
                condition.id,
                arg.name,
                spec.name,
                value,
                arg.choices.join(", "),
                condition.statement,
                condition.source
            ));
        }
    }
    if !has_sourced_enumeration
        && (value.is_empty() || !arg.choices.iter().any(|choice| choice == value))
    {
        return Err(format!(
            "precondition '{}' failed before {} ran: option value '{}' is not in the permitted set [{}]. Choose one of the declared method ids.",
            arg.name,
            spec.name,
            value,
            arg.choices.join(", ")
        ));
    }
    Ok(())
}

/// Validate every closed-set selector without reading a curve or allocating an output version.
/// The ordinary module boundary calls the same [`validate_option_value`] helper below. Saved
/// chains use this metadata-only pass over all steps so a typo in a later step cannot leave the
/// earlier step's curve inside a chain that ultimately refused.
pub(crate) fn validate_module_options(
    name: &str,
    opts: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some(message) = retired_module(name) {
        return Err(message.to_string());
    }
    let spec = module_catalog()
        .iter()
        .find(|module| module.name == name)
        .ok_or_else(|| format!("unknown module '{name}'"))?;
    for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Option) {
        let value = opts.get(&arg.name).map(String::as_str).unwrap_or(&arg.default);
        validate_option_value(spec, arg, value)?;
    }
    Ok(())
}

fn validate_declared_preconditions_ignoring(
    spec: &ModuleSpec,
    ctx: &ModuleContext,
    ignored_samples: &HashSet<usize>,
) -> Result<(), String> {
    let populated = |name: &str| {
        ctx.logs
            .get(name)
            .is_some_and(|values| values.iter().take(ctx.n).any(|value| value.is_finite()))
    };

    for arg in &spec.args {
        for condition in &arg.validity_conditions {
            if condition.id.trim().is_empty() || condition.statement.trim().is_empty() || condition.source.trim().is_empty() {
                return Err(format!(
                    "module '{}' has an invalid validity manifest on '{}': every condition needs a stable id, statement and source",
                    spec.name, arg.name
                ));
            }
            if !condition_is_active(spec, ctx, &condition.rule) {
                continue;
            }

            match &condition.rule {
                // Closed-set values are checked once below for every Option, including options
                // whose manifest predates source-bearing validity conditions.
                ValidityRule::Enumeration => {}
                ValidityRule::NumericRange { min, max, unit, .. } => {
                    for index in 0..ctx.n {
                        if ignored_samples.contains(&index) {
                            continue;
                        }
                        let Some(value) = numeric_value_at(spec, ctx, &arg.name, index) else { continue ;
                        };
                        if !value.is_finite() {
                            continue;
                        }
                        if min.map_or(false, |lo| value < lo) || max.map_or(false, |hi| value > hi) {
                            let suffix = if unit.is_empty() { String::new() } else { format!(" {unit}") };
                            return Err(format!(
                                "precondition '{}' on '{}' failed before {} ran: value {}{} at sample {} is outside {}{}. {} Source: {}",
                                condition.id,
                                arg.name,
                                spec.name,
                                value,
                                suffix,
                                index,
                                format_numeric_range(*min, *max),
                                suffix,
                                condition.statement,
                                condition.source
                            ));
                        }
                    }
                }
                ValidityRule::RequiredCompanion { any_of, .. } => {
                    if !any_of.iter().any(|name| populated(name)) {
                        return Err(format!(
                            "precondition '{}' on '{}' failed before {} ran: none of the required companion inputs [{}] has a finite sample. {} Source: {}",
                            condition.id,
                            arg.name,
                            spec.name,
                            any_of.join(", "),
                            condition.statement,
                            condition.source
                        ));
                    }
                }
                ValidityRule::RequiredValue { .. } => {
                    let Some(values) = ctx.params.get(&arg.name) else {
                        return Err(format!(
                            "precondition '{}' on '{}' failed before {} ran: this parameter ships ABSENT because it has no defensible generic default, and the selected method branch requires an interpreter value. {} Source: {}",
                            condition.id,
                            arg.name,
                            spec.name,
                            condition.statement,
                            condition.source
                        ));
                    };
                    if values.len() < ctx.n {
                        return Err(format!(
                            "precondition '{}' on '{}' failed before {} ran: the selected method branch requires {} sample values but only {} were resolved. {} Source: {}",
                            condition.id,
                            arg.name,
                            spec.name,
                            ctx.n,
                            values.len(),
                            condition.statement,
                            condition.source
                        ));
                    }
                    if let Some((index, value)) = values
                        .iter()
                        .copied()
                        .enumerate()
                        .take(ctx.n)
                        .find(|(index, value)| {
                            !ignored_samples.contains(index) && !value.is_finite()
                        })
                    {
                        return Err(format!(
                            "precondition '{}' on '{}' failed before {} ran: the selected method branch requires a finite interpreter value at sample {}, got {}. {} Source: {}",
                            condition.id,
                            arg.name,
                            spec.name,
                            index,
                            value,
                            condition.statement,
                            condition.source
                        ));
                    }
                }
                ValidityRule::RequiredWhereFinite { input } => {
                    let input_arg = spec.args.iter().find(|candidate| candidate.name == *input);
                    if input_arg.is_none_or(|candidate| candidate.kind != ArgKind::LogIn)
                        || arg.kind != ArgKind::LogIn
                    {
                        return Err(format!(
                            "module '{}' has an invalid validity manifest: '{}' requires two declared LogIn arguments but names '{}' and '{}'",
                            spec.name, condition.id, input, arg.name
                        ));
                    }
                    for index in 0..ctx.n {
                        let primary = numeric_value_at(spec, ctx, input, index);
                        if !primary.is_some_and(f64::is_finite) {
                            continue;
                        }
                        let companion = numeric_value_at(spec, ctx, &arg.name, index);
                        if !companion.is_some_and(f64::is_finite) {
                            return Err(format!(
                                "precondition '{}' on '{}' failed before {} ran: '{}' is finite but '{}' is missing at sample {}. {} Source: {}",
                                condition.id,
                                arg.name,
                                spec.name,
                                input,
                                arg.name,
                                index,
                                condition.statement,
                                condition.source
                            ));
                        }
                    }
                }
                ValidityRule::LessThan { other } => {
                    if !spec.args.iter().any(|candidate| candidate.name == *other) {
                        return Err(format!(
                            "module '{}' has an invalid validity manifest: '{}' names unknown comparison argument '{}'",
                            spec.name, condition.id, other
                        ));
                    }
                    for index in 0..ctx.n {
                        if ignored_samples.contains(&index) {
                            continue;
                        }
                        let (Some(value), Some(other_value)) =
                            (
                                numeric_value_at(spec, ctx, &arg.name, index),
                                numeric_value_at(spec, ctx, other, index),
                            )
                        else {
                            continue;
                        };
                        if value.is_finite() && other_value.is_finite() && value >= other_value {
                            return Err(format!(
                                "precondition '{}' on '{}' failed before {} ran: value {} at sample {} is not less than '{}' value {}. {} Source: {}",
                                condition.id,
                                arg.name,
                                spec.name,
                                value,
                                index,
                                other,
                                other_value,
                                condition.statement,
                                condition.source
                            ));
                        }
                    }
                }
            }
        }

        match &arg.kind {
            ArgKind::Option => {
                let value = selected_value(spec, ctx, &arg.name);
                validate_option_value(spec, arg, &value)?;
            }
            ArgKind::Text => {
                let value = selected_value(spec, ctx, &arg.name);
                if arg.required && value.trim().is_empty() {
                    return Err(format!(
                        "precondition '{}' failed before {} ran: a non-empty value is required.",
                        arg.name, spec.name
                    ));
                }
            }
            ArgKind::Param => {
                let Some(values) = ctx.params.get(&arg.name) else {
                    if arg.required {
                        if arg.default_source == ABSENT_DEFAULT_SOURCE {
                            return Err(format!(
                                "precondition '{}' failed before {} ran: this required parameter ships ABSENT because it has no defensible generic default. Supply an interpreter value before running.",
                                arg.name, spec.name
                            ));
                        }
                        return Err(format!(
                            "precondition '{}' failed before {} ran: the required parameter has no resolved per-sample values.",
                            arg.name, spec.name
                        ));
                    }
                    continue;
                };
                if arg.required && values.len() < ctx.n {
                    return Err(format!(
                        "precondition '{}' failed before {} ran: the required parameter has {} resolved sample values but the frame has {}.",
                        arg.name,
                        spec.name,
                        values.len(),
                        ctx.n
                    ));
                }
                for (index, value) in values.iter().copied().enumerate().take(ctx.n) {
                    if ignored_samples.contains(&index) {
                        continue;
                    }
                    if value.is_nan() && !arg.required {
                        continue;
                    }
                    if !value.is_finite() {
                        let unit = if arg.unit.is_empty() { String::new() } else { format!(" {}", arg.unit) };
                        return Err(format!(
                            "precondition '{}' failed before {} ran: value {}{} at sample {} is not finite. Fix the supplied value or its zone override.",
                            arg.name,
                            spec.name,
                            value,
                            unit,
                            index
                        ));
                    }
                }
            }
            ArgKind::LogIn => {
                if arg.required
                    && !populated(&arg.name)
                    && !arg.required_any_of.iter().any(|alternative| populated(alternative))
                {
                    let expected = if arg.required_any_of.is_empty() {
                        arg.name.clone()
                    } else {
                        std::iter::once(arg.name.as_str())
                            .chain(arg.required_any_of.iter().map(String::as_str))
                            .collect::<Vec<_>>()
                            .join(" or ")
                    };
                    return Err(format!(
                        "precondition '{}' failed before {} ran: none of the required input role [{}] has a finite sample. Select a populated curve before running the module.",
                        arg.name, spec.name, expected
                    ));
                }
            }
            ArgKind::LogOut => {}
        }
    }
    Ok(())
}

/// Dispatches a module run by name.
pub fn run_module(name: &str, ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    // Retired modules resolve by name (their spec stays in the catalog) but must not run — a
    // saved chain step that reaches one fails loudly and actionably rather than silently
    // producing superseded results.
    if let Some(msg) = retired_module(name) {
        return Err(msg.to_string());
    }
    let spec = module_catalog()
        .iter()
        .find(|module| module.name == name)
        .ok_or_else(|| format!("unknown module '{name}'"))?;
    validate_declared_preconditions(spec, ctx)?;
    let outputs = dispatch_module(name, ctx)?;
    validate_flag_outputs(spec, &outputs)?;
    Ok(outputs)
}

fn dispatch_module(name: &str, ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    match name {
        "vsh_gr" => Ok(vsh_gr(ctx)),
        "vsh_dn" => Ok(vsh_dn(ctx)),
        "phi_den" => Ok(phi_den(ctx)),
        "phi_dn" => Ok(phi_dn(ctx)),
        "phi_son" => Ok(phi_son(ctx)),
        "phimax" => Ok(phimax(ctx)),
        "ftemp_grad" => Ok(ftemp_grad(ctx)),
        "precalc" => Ok(precalc(ctx)),
        "badhole" => badhole(ctx),
        "condflag" => condflag(ctx),
        "nphimat" => Ok(nphimat(ctx)),
        "gascorr" => gascorr(ctx),
        "gr_hole_corr" => Ok(gr_hole_corr(ctx)),
        "nphi_env_corr" => Ok(nphi_env_corr(ctx)),
        "rhob_hole_corr" => Ok(rhob_hole_corr(ctx)),
        "gr_normalize" => Ok(gr_normalize(ctx)),
        "log_predict" => Ok(log_predict(ctx)),
        "ssc" => Ok(crate::ssc::ssc(ctx)),
        "sspw" => Ok(crate::ssc::sspw(ctx)),
        "sw_rtc" => Ok(crate::lrlc::sw_rtc(ctx)),
        "sw_imts" => Ok(crate::lrlc::sw_imts(ctx)),
        "sw_height" => Ok(crate::satheight::sw_height(ctx)),
        "midplot" => Ok(crate::lithology::midplot(ctx)),
        "rocktyping" => Ok(crate::rocktyping::rocktyping(ctx)),
        "lucia_rfn" => Ok(crate::rocktyping::lucia_rfn_module(ctx)),
        "pittman_rx" => Ok(crate::rocktyping::pittman_rx(ctx)),
        "rt_cutoff" => Ok(crate::rocktyping::rt_cutoff(ctx)),
        "electrofacies" => crate::facies::electrofacies(ctx),
        "gmm_facies" => crate::facies::gmm_facies(ctx),
        "sw_arch" => Ok(sw_arch(ctx)),
        "sw_indo" => Ok(sw_indo(ctx)),
        "sw_sim" => Ok(sw_sim(ctx)),
        "perm_wyllie_rose" => Ok(perm_wyllie_rose(ctx)),
        "perm_coates" => Ok(perm_coates(ctx)),
        "perm_transform" => Ok(perm_transform(ctx)),
        "thin_bed_ts" => Ok(thin_bed_ts(ctx)),
        "depth_shift" => Ok(depth_shift(ctx)),
        "splice" => Ok(splice(ctx)),
        // Condition — the curve-conditioning family. Each returns a Result of its own: a window
        // that was never set, a bound that would be shadowed by a standard curve and a pivot
        // taken as zero are all refusals rather than plausible-looking output.
        "despike" => crate::condition::despike(ctx),
        "smooth" => crate::condition::smooth(ctx),
        "clip" => crate::condition::clip(ctx),
        "fill_gaps" => crate::condition::fill_gaps(ctx),
        "flip" => crate::condition::flip(ctx),
        "normalize" => crate::condition::normalize(ctx),
        // Frame — depth-sampling. Both refuse rather than guess what a bed is.
        "block" => crate::frame::block(ctx),
        "bed_detect" => crate::frame::bed_detect(ctx),
        "toc_passey" => Ok(crate::unconventional::toc_passey(ctx)),
        "kerogen" => Ok(crate::unconventional::kerogen(ctx)),
        "gip" => Ok(crate::unconventional::gip(ctx)),
        "brittleness" => Ok(crate::unconventional::brittleness(ctx)),
        other => Err(format!("unknown module '{other}'")),
    }
}

// ---------------------------------------------------------------------------
// VSH_GR — Volume of shale from gamma ray (Loglan vsh_gr.lls)
// ---------------------------------------------------------------------------

const GR_ENDPOINT_PICKING_GUIDANCE: (&str, &str) = (
    "IP derives endpoints by pooling a Percentile Group, pre-clipping at 0%/98%, computing a selected percentile and linearly extrapolating. Its clay percentile is 130%; its clean percentile is unstated. Techlog offers 5%/95%; P3/P97 is an optional named house preset. Treat these as alternative procedures, not a generic endpoint value.",
    "docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; IP clayparameters.htm (57, 59, 60); Techlog VSH single-log pages; method_workflow_standards.md",
);

const ND_CROSSPLOT_PICKING_GUIDANCE: (&str, &str) = (
    "IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.",
    "docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page",
);

fn vsh_gr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "vsh_gr".into(),
        title: "VSH from Gamma Ray".into(),
        category: "VSH".into(),
        doc: "VSH_GR = (GR - GR_MA) / (GR_SH - GR_MA), with optional non-linear corrections \
              (Stieber, Larionov, Clavier). VSH is the result limited to 0–1."
            .into(),
        args: vec![
            // Labels, not renamed ids: the id is what `params_json` stores on every saved run, and
            // it is what the label leads with so a user reading a stored run still recognises it.
            //
            // The two Larionov forms are the reason this exists. They differ only by a digit in
            // their name and by a factor of more than 1.5 in their answer at mid-range gamma —
            // 0.330 against 0.216 at IGR 0.5 — which lands squarely where the VSH cutoff decides
            // net pay, on a curve that looks entirely normal. The rock-age attributions are the
            // published ones (Larionov 1969) and are pinned against the closed forms by
            // `every_vsh_gr_transform_lands_on_its_published_coefficient`.
            //
            // LARINOV3 is stated by its coefficients rather than attributed: nothing in the repo
            // cites a source for that form, and inventing one is the move the provenance rules
            // forbid.
            with_validity(
                opt_labelled(
                    "OPT_GR",
                    "VSH from gamma ray method",
                    "LINEAR",
                    &[
                        ("LINEAR", "LINEAR — VSH = IGR"),
                        ("STIEBER1", "STIEBER1 — Stieber, IGR/(3−2·IGR)"),
                        ("STIEBER2", "STIEBER2 — Stieber, IGR/(2−IGR)"),
                        ("STIEBER3", "STIEBER3 — Stieber, IGR/(4−3·IGR)"),
                        ("LARINOV1", "LARINOV1 — Larionov, Mesozoic and older"),
                        ("LARINOV2", "LARINOV2 — Larionov, Tertiary / unconsolidated"),
                        ("LARINOV3", "LARINOV3 — 0.127·(3.15^(2·IGR) − 1)"),
                        ("CLAVIER", "CLAVIER — Clavier et al."),
                    ],
                ),
                vec![validity(
                    "vsh_gr.method_id",
                    "The selected GR transform must be one of the method ids declared by the manifest.",
                    "docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.lls L109-L139",
                    ValidityRule::Enumeration,
                )],
            ),
            with_guidance(with_sources(with_validity(
                param_open("GR_MA", "Gamma ray matrix (clean)", "gAPI", 0.0, 200.0, true),
                vec![
                    validity(
                        "vsh_gr.gr_ma_range",
                        "The clean gamma-ray endpoint must remain inside the source manifest range.",
                        "docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.info L48-L49",
                        ValidityRule::NumericRange {
                            min: Some(0.0),
                            max: Some(200.0),
                            unit: "gAPI".into(),
                            when: None,
                        },
                    ),
                    validity(
                        "vsh_gr.endpoint_order",
                        "The clean gamma-ray endpoint must be strictly below the shale endpoint.",
                        "docs/PRD_v2/10_clay-volume.md §3.3 and SB-CLY-001; Geolog vsh_gr.lls L99-L102",
                        ValidityRule::LessThan { other: "GR_SH".into() },
                    ),
                ],
            ), crate::param_sources::GR_CLEAN_ENDPOINT), &[GR_ENDPOINT_PICKING_GUIDANCE]),
            with_guidance(with_sources(with_validity(
                param_open("GR_SH", "Gamma ray shale", "gAPI", 0.0, 1000.0, true),
                vec![validity(
                    "vsh_gr.gr_sh_range",
                    "The shale gamma-ray endpoint must remain inside the source manifest range.",
                    "docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.info L48-L49",
                    ValidityRule::NumericRange {
                        min: Some(0.0),
                        max: Some(1000.0),
                        unit: "gAPI".into(),
                        when: None,
                    },
                )],
            ), crate::param_sources::GR_SHALE_ENDPOINT), &[GR_ENDPOINT_PICKING_GUIDANCE]),
            log_in_preferred(
                "GR",
                "Gamma ray log",
                "gAPI",
                &["GR_COR", "GR_EC", "GR"],
                true,
            ),
            log_out("VSH_GR", "VSH from gamma ray (unlimited)", "v/v"),
            log_out("VSH", "Limited volume of shale", "v/v"),
        ],
    }
}

fn vsh_gr(ctx: &ModuleContext) -> ModuleOutputs {
    let gr = ctx.log("GR");
    let method = ctx.o("OPT_GR").to_string();
    let mut vsh_gr_out = vec![f32::NAN; ctx.n];
    let mut vsh_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let g = gr[i] as f64;
        let gr_ma = ctx.p("GR_MA", i);
        let gr_sh = ctx.p("GR_SH", i);
        if is_missing(g) || is_missing(gr_ma) || is_missing(gr_sh) || gr_ma >= gr_sh {
            continue;
        }
        let mut v = (g - gr_ma) / (gr_sh - gr_ma);
        let unlimited = match method.as_str() {
            "STIEBER1" => {
                v = limit(v, -10.0, 1.49);
                v / (3.0 - 2.0 * v)
            }
            "STIEBER2" => {
                v = limit(v, -10.0, 1.99);
                v / (2.0 - v)
            }
            "STIEBER3" => {
                v = limit(v, -10.0, 1.33);
                v / (4.0 - 3.0 * v)
            }
            "LARINOV1" => 0.33 * (2.0_f64.powf(2.0 * v) - 1.0),
            "LARINOV2" => 0.083 * (2.0_f64.powf(3.7 * v) - 1.0),
            "LARINOV3" => 0.127 * (3.15_f64.powf(2.0 * v) - 1.0),
            "CLAVIER" => {
                v = limit(v, -2.53, 1.13);
                1.7 - (3.38 - (v + 0.7).powi(2)).sqrt()
            }
            _ => v, // LINEAR
        };
        vsh_gr_out[i] = unlimited as f32;
        vsh_out[i] = limit(unlimited, 0.0, 1.0) as f32;
    }

    HashMap::from([("VSH_GR".to_string(), vsh_gr_out), ("VSH".to_string(), vsh_out)])
}

// ---------------------------------------------------------------------------
// VSH_DN — Volume of shale from density-neutron crossplot (Loglan vsh_dn.lls)
// ---------------------------------------------------------------------------

fn vsh_dn_spec() -> ModuleSpec {
    ModuleSpec {
        name: "vsh_dn".into(),
        title: "VSH from Density-Neutron".into(),
        category: "VSH".into(),
        doc: "Two-log crossplot VSH: the (RHOB, NPHI) point's position between the clean \
              matrix line and the shale point. Density in g/cc. CAUTION: the neutron shale \
              response is hydroxyl-driven, so a single NPHI_SH endpoint is clay-type \
              sensitive — a 4-OH clay (illite/smectite) gives ~12 p.u. N-D separation vs \
              ~35 p.u. for an 8-OH clay (kaolinite/chlorite). Supply GR to raise \
              VSH_DN_FLAG where the N-D VSH diverges from the clay-type-insensitive GR VSH \
              (clay-type or gas ambiguity), or falls off the matrix–shale–fluid triangle."
            .into(),
        args: vec![
            with_guidance(
                with_sources(
                    param_open("RHO_MA", "Matrix density", "g/cc", 2.0, 3.2, true),
                    crate::param_sources::MATRIX_DENSITY,
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                with_sources(
                    param_open("RHO_SH", "Shale density", "g/cc", 1.5, 3.0, true),
                    crate::param_sources::SHALE_DENSITY,
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                param_from_artefact(
                    "RHO_FL", "Fluid density", "g/cc", 1000.0, "k/m3", 0.5, 1.5,
                    "Geolog vsh_dn.info RHO_FL DEFAULT 1000 k/m3; Techlog petrophysics-vsh-from-neutrondensity.html RHO fluid 1.0 g/cm3; docs/PRD_v2/10_clay-volume.md §5",
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                with_sources(
                    param_open("NPHI_MA", "Matrix neutron porosity", "v/v", -0.15, 0.5, true),
                    crate::param_sources::MATRIX_NEUTRON_ENDPOINT,
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                with_sources(
                    param_open("NPHI_SH", "Shale neutron porosity", "v/v", 0.0, 0.8, true),
                    crate::param_sources::SHALE_NEUTRON_ENDPOINT,
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                param_from_artefact(
                    "NPHI_FL", "Fluid neutron porosity", "v/v", 1.0, "v/v", 0.5, 1.2,
                    "Geolog vsh_dn.info NPHI_FL 1 v/v; Techlog petrophysics-vsh-from-neutrondensity.html NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5",
                ),
                &[ND_CROSSPLOT_PICKING_GUIDANCE],
            ),
            with_guidance(
                with_sources(
                    param_open("GR_MA", "Clean GR (clay-type cross-check)", "gAPI", 0.0, 150.0, true),
                    crate::param_sources::GR_CLEAN_ENDPOINT,
                ),
                &[GR_ENDPOINT_PICKING_GUIDANCE],
            ),
            with_guidance(
                with_sources(
                    param_open("GR_SH", "Shale GR (clay-type cross-check)", "gAPI", 40.0, 400.0, true),
                    crate::param_sources::GR_SHALE_ENDPOINT,
                ),
                &[GR_ENDPOINT_PICKING_GUIDANCE],
            ),
            param_from_artefact(
                "FLAG_TOL", "Flag |VSH(N-D) − VSH(GR)| above this", "v/v", 0.25, "v/v", 0.05, 1.0,
                "docs/PRD_v2/10_clay-volume.md §5.1 — SandiBumi diagnostic threshold",
            ),
            log_in_preferred(
                "RHOB",
                "Density log",
                "g/cc",
                &["RHO_COR", "RHOB_EC", "RHOB"],
                true,
            ),
            log_in_preferred(
                "NPHI",
                "Neutron porosity log",
                "v/v",
                &["NPHI_COR", "NPHI_EC", "NPHI"],
                true,
            ),
            log_in_preferred(
                "GR",
                "Gamma ray (optional clay-type cross-check)",
                "gAPI",
                &["GR_COR", "GR_EC", "GR"],
                false,
            ),
            log_out("VSH_DN", "VSH from density-neutron (unlimited)", "v/v"),
            log_out("VSH", "Limited volume of shale", "v/v"),
            log_out("VSH_DN_FLAG", "1 where N-D VSH is unreliable (off-model, or diverges from GR VSH)", "flag"),
        ],
    }
}

fn vsh_dn_rearrangement(
    rho_b: f64,
    nphi: f64,
    rho_ma: f64,
    rho_sh: f64,
    rho_fl: f64,
    nphi_ma: f64,
    nphi_sh: f64,
    nphi_fl: f64,
) -> Option<f64> {
    let a = (rho_ma - rho_fl) * (nphi_fl - nphi);
    let b = (rho_b - rho_fl) * (nphi_fl - nphi_ma);
    let c = (rho_ma - rho_fl) * (nphi_fl - nphi_sh);
    let d = (rho_sh - rho_fl) * (nphi_fl - nphi_ma);
    // A degenerate matrix/shale/fluid triangle (near-collinear endpoints — an in-range but
    // physically bad parameter choice) drives (c - d) to ~0, sending the UNLIMITED VSH_DN to
    // +/-Infinity on every sample. Preserve the existing refusal boundary as part of the f64
    // evaluator so the tested algebra and the shipping module cannot drift apart.
    ((c - d).abs() >= 1e-6).then(|| (a - b) / (c - d))
}

fn vsh_dn(ctx: &ModuleContext) -> ModuleOutputs {
    let rho = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let gr = ctx.log("GR");
    let mut vsh_dn_out = vec![f32::NAN; ctx.n];
    let mut vsh_out = vec![f32::NAN; ctx.n];
    let mut flag_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, np) = (rho[i] as f64, nphi[i] as f64);
        let rho_ma = ctx.p("RHO_MA", i);
        let rho_sh = ctx.p("RHO_SH", i);
        let rho_fl = ctx.p("RHO_FL", i);
        let nphi_ma = ctx.p("NPHI_MA", i);
        let nphi_sh = ctx.p("NPHI_SH", i);
        let nphi_fl = ctx.p("NPHI_FL", i);
        if is_missing(r) || is_missing(np) {
            continue;
        }
        let Some(v) = vsh_dn_rearrangement(
            r, np, rho_ma, rho_sh, rho_fl, nphi_ma, nphi_sh, nphi_fl,
        ) else {
            continue;
        };
        vsh_dn_out[i] = v as f32;
        let v_lim = limit(v, 0.0, 1.0);
        vsh_out[i] = v_lim as f32;

        // Clay-type / gas guard. A single N-D VSH is only as trustworthy as NPHI_SH,
        // whose value depends on clay hydroxyl count (Ellis Ch14/21). Flag samples that
        // fall off the matrix–shale–fluid triangle, or — when GR is supplied — whose N-D
        // VSH diverges from the clay-type-insensitive GR VSH.
        let mut unreliable = v < -0.05 || v > 1.05;
        if !unreliable {
            let g = gr[i] as f64;
            let gr_ma = ctx.p("GR_MA", i);
            let gr_sh = ctx.p("GR_SH", i);
            let tol = ctx.p("FLAG_TOL", i);
            if g.is_finite() && tol.is_finite() && (gr_sh - gr_ma).abs() > 1e-6 {
                let vsh_gr = ((g - gr_ma) / (gr_sh - gr_ma)).clamp(0.0, 1.0);
                if (v_lim - vsh_gr).abs() > tol {
                    unreliable = true;
                }
            }
        }
        flag_out[i] = if unreliable { 1.0 } else { 0.0 };
    }

    HashMap::from([
        ("VSH_DN".to_string(), vsh_dn_out),
        ("VSH".to_string(), vsh_out),
        ("VSH_DN_FLAG".to_string(), flag_out),
    ])
}

// ---------------------------------------------------------------------------
// PHI_DEN — Porosity from density log (Loglan phi_den.lls)
// ---------------------------------------------------------------------------

fn phi_den_spec() -> ModuleSpec {
    ModuleSpec {
        name: "phi_den".into(),
        title: "Porosity from Density".into(),
        category: "Porosity".into(),
        doc: "PHIE = (RHO_MA - RHOB)/(RHO_MA - RHO_FL) - VSH*(RHO_MA - RHO_SH)/(RHO_MA - RHO_FL). \
              PHIT = PHIE + VSH*PHIT_SH, where PHIT_SH = (RHO_DSH - RHO_SH)/(RHO_DSH - RHO_W). \
              Above 95% VSH the sample is treated as shale."
            .into(),
        args: vec![
            param_sourced(
                "RHO_MA", "Matrix density", "g/cc", 2.65, 2.0, 3.2,
                crate::param_sources::MATRIX_DENSITY,
                "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.",
            ),
            with_sources(param_open("RHO_SH", "Shale density", "g/cc", 1.5, 3.0, true), crate::param_sources::SHALE_DENSITY),
            with_sources(
                param(
                    "RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5,
                    "IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
                ),
                crate::param_sources::FLUID_DENSITY,
            ),
            with_sources(param_open("RHO_DSH", "Dry shale density", "g/cc", 2.0, 3.2, true), crate::param_sources::DRY_SHALE_DENSITY),
            with_sources(
                param(
                    "RHO_W", "Formation water density", "g/cc", 1.0, 0.8, 1.3,
                    "Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
                ),
                crate::param_sources::FORMATION_WATER_DENSITY,
            ),
            with_sources(
                opt("OPT_PHIEMAX", "PHIE limiting method", "SHALE_REDUCED", &["SHALE_REDUCED", "MAXIMUM"]),
                crate::param_sources::POROSITY_LIMIT_MODE,
            ),
            with_sources(
                param(
                    "PHIE_MAX", "Maximum allowed PHIE", "v/v", 0.3, 0.05, 0.5,
                    "Geolog V14 phi_den.info PHIE_MAX DEFAULT 0.3; docs/PRD_v2/11_porosity.md §5.3",
                ),
                crate::param_sources::MAX_EFFECTIVE_POROSITY,
            ),
            param_sourced(
                "VSH_SHALE",
                "High-shale kill threshold (at or above it: PHIE = 0, PHIT = PHIT_SH)",
                "v/v",
                0.95,
                0.0,
                1.0,
                crate::param_sources::HIGH_SHALE_BRANCH_THRESHOLD,
                "Geolog V14 phi_*.lls hard-coded VSH >= 0.95 (all six modules); docs/PRD_v2/11_porosity.md §5 line 1229 makes it a parameter in SandiBumi defaulting to 0.95 with this source",
            ),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
            log_out("PHIE_DEN", "PHIE from density (unlimited)", "v/v"),
            log_out("PHIT_DEN", "PHIT from density (unlimited)", "v/v"),
            log_out("PHIE", "Limited effective porosity", "v/v"),
            log_out("PHIT", "Limited total porosity", "v/v"),
        ],
    }
}

/// The one clay-bound-water porosity in the product (SB-POR-008):
/// `PHIT_SH = (RHO_DSH - RHO_SH) / (RHO_DSH - RHO_W)`.
///
/// `rho_w` is the FORMATION WATER density and is deliberately not the fluid density: the water
/// filling shale porosity is formation water, while `RHO_FL` describes the fluid the density
/// porosity is computed against. The two ship at the same 1.00 default
/// (`docs/PRD_v2/11_porosity.md` section 5.1), so substituting one for the other is invisible until
/// an interpreter selects salt water at 1.10 — which is exactly why this must exist once rather
/// than be rewritten per module.
///
/// The **shale-subtraction** term `(RHO_MA - RHO_SH)/(RHO_MA - RHO_FL)` is a different quantity and
/// must never share this name (F16).
pub(crate) fn shale_total_porosity(rho_dsh: f64, rho_sh: f64, rho_w: f64) -> f64 {
    (rho_dsh - rho_sh) / (rho_dsh - rho_w)
}

/// Shared PHIT_SH derivation from phi_den/phi_dn: shale total porosity from densities.
fn phit_sh_at(ctx: &ModuleContext, i: usize) -> f64 {
    shale_total_porosity(
        ctx.p("RHO_DSH", i),
        ctx.p("RHO_SH", i),
        ctx.p("RHO_W", i),
    )
}

fn phi_den(ctx: &ModuleContext) -> ModuleOutputs {
    let rho = ctx.log("RHOB");
    let vsh = ctx.log("VSH");
    let shale_reduced = ctx.o("OPT_PHIEMAX") != "MAXIMUM";
    let mut phie_den = vec![f32::NAN; ctx.n];
    let mut phit_den = vec![f32::NAN; ctx.n];
    let mut phie_lim_out = vec![f32::NAN; ctx.n];
    let mut phit_lim_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, v) = (rho[i] as f64, vsh[i] as f64);
        if is_missing(r) || is_missing(v) {
            continue;
        }
        let rho_ma = ctx.p("RHO_MA", i);
        let rho_sh = ctx.p("RHO_SH", i);
        let rho_fl = ctx.p("RHO_FL", i);
        let phie_max = ctx.p("PHIE_MAX", i);
        let phit_sh = phit_sh_at(ctx, i);

        if v >= ctx.p("VSH_SHALE", i) {
            phie_den[i] = 0.0;
            phie_lim_out[i] = PHIE_FLOOR as f32;
            phit_den[i] = phit_sh as f32;
            phit_lim_out[i] = phit_sh as f32;
            continue;
        }

        let pe = (rho_ma - r) / (rho_ma - rho_fl) - v * (rho_ma - rho_sh) / (rho_ma - rho_fl);
        let pt = pe + v * phit_sh;
        let phie_lim = if shale_reduced { phie_max * (1.0 - v) } else { phie_max };
        let pe_l = limit(pe, PHIE_FLOOR, phie_lim);
        phie_den[i] = pe as f32;
        phit_den[i] = pt as f32;
        phie_lim_out[i] = pe_l as f32;
        phit_lim_out[i] = (pe_l + v * phit_sh) as f32;
    }

    HashMap::from([
        ("PHIE_DEN".to_string(), phie_den),
        ("PHIT_DEN".to_string(), phit_den),
        ("PHIE".to_string(), phie_lim_out),
        ("PHIT".to_string(), phit_lim_out),
    ])
}

// ---------------------------------------------------------------------------
// PHI_DN — Porosity from density-neutron (Loglan phi_dn.lls structure; analytic
// crossplot instead of proprietary service-company chart lookups)
// ---------------------------------------------------------------------------

fn phi_dn_spec() -> ModuleSpec {
    ModuleSpec {
        name: "phi_dn".into(),
        title: "Porosity from Density-Neutron".into(),
        category: "Porosity".into(),
        doc: "Shale-corrects RHOB and NPHI to 'shale reduced' values, then combines density \
              porosity and neutron porosity: AVERAGE = (PHID+PHIN)/2, GAS_RMS = sqrt((PHID²+PHIN²)/2) \
              for gas-bearing zones. QUICK-LOOK COMPARISON ONLY - neither combination is a crossplot \
              porosity method: no vendor ships either as one, and IP says of the field shortcuts that \
              they should not be used for anything other than this. The chart-free analytic \
              neutron-density method is a separate contract (SB-POR-021). \
              PHIE = PHIX*(1-VSH); PHIT = PHIE + VSH*PHIT_SH."
            .into(),
        args: vec![
            opt("OPT_XPLOT", "Crossplot combination method", "AVERAGE", &["AVERAGE", "GAS_RMS"]),
            param_sourced(
                "RHO_MA", "Matrix density", "g/cc", 2.65, 2.0, 3.2,
                crate::param_sources::MATRIX_DENSITY,
                "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.",
            ),
            with_sources(param_open("RHO_SH", "Shale density", "g/cc", 1.5, 3.0, true), crate::param_sources::SHALE_DENSITY),
            with_sources(
                param(
                    "RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5,
                    "IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
                ),
                crate::param_sources::FLUID_DENSITY,
            ),
            with_sources(param_open("NPHI_SH", "Shale neutron porosity", "v/v", 0.0, 0.8, true), crate::param_sources::SHALE_NEUTRON_ENDPOINT),
            with_sources(param_open("RHO_DSH", "Dry shale density", "g/cc", 2.0, 3.2, true), crate::param_sources::DRY_SHALE_DENSITY),
            with_sources(
                param(
                    "RHO_W", "Formation water density", "g/cc", 1.0, 0.8, 1.3,
                    "Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
                ),
                crate::param_sources::FORMATION_WATER_DENSITY,
            ),
            with_sources(
                opt("OPT_PHIEMAX", "PHIE limiting method", "SHALE_REDUCED", &["SHALE_REDUCED", "MAXIMUM"]),
                crate::param_sources::POROSITY_LIMIT_MODE,
            ),
            with_sources(
                param(
                    "PHIE_MAX", "Maximum allowed PHIE", "v/v", 0.3, 0.05, 0.5,
                    "Geolog V14 phi_dn.info PHIE_MAX DEFAULT 0.3; docs/PRD_v2/11_porosity.md §5.3",
                ),
                crate::param_sources::MAX_EFFECTIVE_POROSITY,
            ),
            param_sourced(
                "VSH_SHALE",
                "High-shale kill threshold (at or above it: PHIE = 0, PHIT = PHIT_SH)",
                "v/v",
                0.95,
                0.0,
                1.0,
                crate::param_sources::HIGH_SHALE_BRANCH_THRESHOLD,
                "Geolog V14 phi_*.lls hard-coded VSH >= 0.95 (all six modules); docs/PRD_v2/11_porosity.md §5 line 1229 makes it a parameter in SandiBumi defaulting to 0.95 with this source",
            ),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", true),
            log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
            log_out("PHIE_DN", "PHIE from density-neutron (unlimited)", "v/v"),
            log_out("PHIT_DN", "PHIT from density-neutron (unlimited)", "v/v"),
            log_out_as(
                "PHIE",
                PHIE_DN_LIMITED_DEFAULT,
                "Limited effective porosity",
                "v/v",
            ),
            log_out_as(
                "PHIT",
                PHIT_DN_LIMITED_DEFAULT,
                "Limited total porosity",
                "v/v",
            ),
        ],
    }
}

fn phi_dn(ctx: &ModuleContext) -> ModuleOutputs {
    let rho = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let vsh = ctx.log("VSH");
    let gas_rms = ctx.o("OPT_XPLOT") == "GAS_RMS";
    let shale_reduced = ctx.o("OPT_PHIEMAX") != "MAXIMUM";
    let mut phie_dn = vec![f32::NAN; ctx.n];
    let mut phit_dn = vec![f32::NAN; ctx.n];
    let mut phie_lim_out = vec![f32::NAN; ctx.n];
    let mut phit_lim_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, np, v) = (rho[i] as f64, nphi[i] as f64, vsh[i] as f64);
        if is_missing(r) || is_missing(np) || is_missing(v) {
            continue;
        }
        let rho_ma = ctx.p("RHO_MA", i);
        let rho_sh = ctx.p("RHO_SH", i);
        let rho_fl = ctx.p("RHO_FL", i);
        let nphi_sh = ctx.p("NPHI_SH", i);
        let phie_max = ctx.p("PHIE_MAX", i);
        let phit_sh = phit_sh_at(ctx, i);

        if v >= ctx.p("VSH_SHALE", i) {
            phie_dn[i] = 0.0;
            phie_lim_out[i] = PHIE_FLOOR as f32;
            phit_dn[i] = phit_sh as f32;
            phit_lim_out[i] = phit_sh as f32;
            continue;
        }

        // Shale-reduce the input logs (same limits as the Loglan source, in g/cc).
        let rhosr = limit((r - v * rho_sh) / (1.0 - v), 1.95, 3.0);
        let nphisr = limit((np - v * nphi_sh) / (1.0 - v), -0.015, 0.40);

        let phid = (rho_ma - rhosr) / (rho_ma - rho_fl);
        let phix = if gas_rms {
            ((phid * phid + nphisr * nphisr) / 2.0).sqrt()
        } else {
            (phid + nphisr) / 2.0
        };

        let pe = phix * (1.0 - v);
        let pt = pe + v * phit_sh;
        let phie_lim = if shale_reduced { phie_max * (1.0 - v) } else { phie_max };
        let pe_l = limit(pe, PHIE_FLOOR, phie_lim);
        phie_dn[i] = pe as f32;
        phit_dn[i] = pt as f32;
        phie_lim_out[i] = pe_l as f32;
        phit_lim_out[i] = (pe_l + v * phit_sh) as f32;
    }

    HashMap::from([
        ("PHIE_DN".to_string(), phie_dn),
        ("PHIT_DN".to_string(), phit_dn),
        ("PHIE".to_string(), phie_lim_out),
        ("PHIT".to_string(), phit_lim_out),
    ])
}

// ---------------------------------------------------------------------------
// PHI_SON — Porosity from sonic (Wyllie time-average / Raymer-Hunt-Gardner)
// ---------------------------------------------------------------------------

fn phi_son_spec() -> ModuleSpec {
    ModuleSpec {
        name: "phi_son".into(),
        title: "Porosity from Sonic".into(),
        category: "Porosity".into(),
        doc: "WYLLIE: PHIT = (DT - DT_MA)/(DT_FL - DT_MA), shale-corrected for PHIE. \
              RHG (Raymer-Hunt-Gardner): PHIT = 0.625*(DT - DT_MA)/DT. \
              OPT_CP=ON applies the Wyllie lack-of-compaction correction (Cp = DT_SH/100): \
              undercompacted shaly sands (e.g. shallow Mahakam delta) read porosity high on \
              the straight time-average, so the WYLLIE porosity is divided by Cp. RHG is \
              self-compacting and is never Cp-corrected."
            .into(),
        args: vec![
            opt("OPT_SON", "Sonic porosity method", "WYLLIE", &["WYLLIE", "RHG"]),
            with_sources(
                opt("OPT_CP", "Wyllie lack-of-compaction correction (Cp = DT_SH/100)", "OFF", &["OFF", "ON"]),
                crate::param_sources::SONIC_COMPACTION_CORRECTION,
            ),
            with_sources(
                param_open("DT_MA", "Matrix transit time", "us/ft", 40.0, 70.0, true),
                crate::param_sources::MATRIX_TRANSIT_TIME,
            ),
            with_sources(
                param(
                    "DT_FL", "Fluid transit time", "us/ft", 189.0, 150.0, 220.0,
                    "IP swparameters.htm Sonic water Default 189; Geolog phi_son.info DT_FL 620 us/m; docs/PRD_v2/11_porosity.md §5.2",
                ),
                crate::param_sources::FLUID_TRANSIT_TIME,
            ),
            with_sources(
                param_open("DT_SH", "Shale transit time", "us/ft", 60.0, 150.0, true),
                crate::param_sources::SHALE_TRANSIT_TIME,
            ),
            log_in("DT", "Sonic transit time log", "us/ft", "DT", true),
            log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
            log_out("PHIT_SON", "Total porosity from sonic", "v/v"),
            log_out("PHIE_SON", "Effective porosity from sonic", "v/v"),
        ],
    }
}

fn phi_son(ctx: &ModuleContext) -> ModuleOutputs {
    let dt = ctx.log("DT");
    let vsh = ctx.log("VSH");
    let rhg = ctx.o("OPT_SON") == "RHG";
    let cp_on = ctx.o("OPT_CP") == "ON";
    let mut phit_son = vec![f32::NAN; ctx.n];
    let mut phie_son = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (d, v) = (dt[i] as f64, vsh[i] as f64);
        if is_missing(d) {
            continue;
        }
        let dt_ma = ctx.p("DT_MA", i);
        let dt_fl = ctx.p("DT_FL", i);
        let dt_sh = ctx.p("DT_SH", i);

        // Wyllie lack-of-compaction divisor Cp = DT_SH/100 (Hilchie): the whole time-average
        // porosity is scaled by 1/Cp in undercompacted section. RHG is self-compacting, so Cp
        // never applies to it (and a non-positive DT_SH degenerates to no correction).
        let cp = if cp_on && !rhg && dt_sh > 0.0 { dt_sh / 100.0 } else { 1.0 };

        let pt = if rhg {
            0.625 * (d - dt_ma) / d
        } else {
            (d - dt_ma) / (dt_fl - dt_ma) / cp
        };
        phit_son[i] = limit(pt, 0.0, 1.0) as f32;
        if !is_missing(v) {
            // pt already carries the 1/Cp scaling, so the shale term is divided by Cp too —
            // the effective porosity is [raw - Vsh·shale] / Cp, per the standard shaly-sand form.
            let pe = pt - v * (dt_sh - dt_ma) / (dt_fl - dt_ma) / cp;
            // SB-POR-009 / F21: effective porosity can never exceed total porosity. Density and
            // D-N get this free because they rebuild PHIT from the limited PHIE, but sonic
            // computes the two independently, so the ordering has to be imposed here — bounding
            // PHIE by the already-limited PHIT, exactly as `ssc`/`sspw` do.
            //
            // This binds only where the shale term is NEGATIVE, i.e. DT_SH < DT_MA. That is not a
            // hypothetical: DT_MA 70 and DT_SH 60 are both inside the shipped declared ranges, and
            // there the subtraction becomes an addition and effective porosity overtakes total.
            // No new bound is introduced — the ceiling is the sample's own total porosity.
            phie_son[i] = limit(pe, 0.0, phit_son[i] as f64) as f32;
        }
    }

    HashMap::from([("PHIT_SON".to_string(), phit_son), ("PHIE_SON".to_string(), phie_son)])
}

// ---------------------------------------------------------------------------
// PHIMAX — porosity ceiling from a compaction trend (deck slide 64 "max core
// porosity" line). Caps a computed porosity at the field's compaction-controlled
// upper limit, per-zone overridable, as a CONSTANT or a TVDSS trend.
// ---------------------------------------------------------------------------

fn phimax_spec() -> ModuleSpec {
    ModuleSpec {
        name: "phimax".into(),
        title: "Porosity Ceiling (φmax)".into(),
        category: "Porosity".into(),
        doc: "Caps an input porosity at a maximum ceiling — the field's compaction-\
              controlled upper limit (the crossplot 'max core porosity' line). The ceiling \
              is CONSTANT (PHIMAX0, per-zone overridable), or a TVDSS compaction TREND: \
              LINEAR (φmax = PHIMAX0 − PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000) or ATHY \
              exponential (φmax = PHIMAX0·exp(−ATHY_K·(TVDSS − TVDSS_REF)/1000)). TVDSS is a \
              POSITIVE-downward depth-below-datum curve (same convention as precalc), so \
              DEEPER = larger TVDSS = lower ceiling; all four params are per-zone \
              overridable. No TVDSS curve → measured DEPTH is used instead (fine for near-\
              vertical wells; the trend then reads against MD). Writes <PHI>_CAP = \
              min(PHI, φmax) preserving MISSING, and the ceiling curve <PHI>_MAX for QC \
              overlay; the input porosity is never modified. Constant mode ignores TVDSS."
            .into(),
        args: vec![
            opt(
                "MODE",
                "Ceiling model",
                "linear",
                &["constant", "linear", "athy"],
            ),
            param_open(
                "PHIMAX0",
                "φmax at TVDSS_REF (also the CONSTANT cap value)",
                "v/v",
                0.0,
                1.0,
                true,
            ),
            param_open_when(
                "TVDSS_REF",
                "Reference TVDSS where φmax = PHIMAX0",
                PROJECT_DEPTH_UNIT_TOKEN,
                -30000.0,
                30000.0,
                &[("MODE", "linear"), ("MODE", "athy")],
                "docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters",
            ),
            param_open_when(
                "PHIMAX_GRAD",
                "LINEAR: φmax lost per 1000 TVDSS units deeper",
                "v/v per 1000",
                -1.0,
                1.0,
                &[("MODE", "linear")],
                "docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters",
            ),
            param_open_when(
                "ATHY_K",
                "ATHY: compaction coefficient per 1000 TVDSS units",
                "1/1000",
                0.0,
                5.0,
                &[("MODE", "athy")],
                "docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters",
            ),
            log_in("PHI", "Porosity to cap", "v/v", "PHIE", true),
            log_in("TVDSS", "True vertical depth subsea (trend modes)", "ft|m", "TVDSS", false),
            log_out_as("PHI_CAP", "{PHI}_CAP", "Capped porosity", "v/v"),
            log_out_as("PHI_MAX", "{PHI}_MAX", "φmax ceiling curve", "v/v"),
        ],
    }
}

fn phimax(ctx: &ModuleContext) -> ModuleOutputs {
    let phi = ctx.log("PHI");
    let mode = ctx.o("MODE");
    // TVDSS falls back to measured depth as a WHOLE curve, never per sample (matching
    // precalc) — mixing MD and TVD samples would kink the trend. Constant mode never
    // touches it. Positive-downward convention: deeper = larger value = lower ceiling.
    let tvd_in = ctx.log("TVDSS");
    let tvd: Vec<f32> = if tvd_in.iter().any(|v| v.is_finite()) {
        tvd_in
    } else {
        if mode != "constant" {
            record_degradation_once(
                RunDegradationKind::SubstitutedInput,
                "TVDSS was absent, so measured DEPTH supplied the whole compaction-trend frame",
            );
        }
        ctx.log("DEPTH")
    };

    let mut capped = vec![f32::NAN; ctx.n];
    let mut ceiling = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let p0 = ctx.p("PHIMAX0", i);
        let phi_max = match mode {
            "constant" => p0,
            "linear" | "athy" => {
                let d = tvd[i] as f64;
                if is_missing(d) {
                    MISSING
                } else {
                    // dz > 0 when the sample is DEEPER than the reference (larger TVDSS/MD).
                    let dz = (d - ctx.p("TVDSS_REF", i)) / 1000.0;
                    if mode == "linear" {
                        p0 - ctx.p("PHIMAX_GRAD", i) * dz
                    } else {
                        p0 * (-ctx.p("ATHY_K", i) * dz).exp()
                    }
                }
            }
            _ => p0,
        };
        // A porosity ceiling below 0 or above 1 is meaningless; clamp (MISSING passes through).
        let phi_max = limit(phi_max, 0.0, 1.0);
        ceiling[i] = phi_max as f32;

        let pv = phi[i] as f64;
        if is_missing(pv) {
            continue; // capped stays MISSING wherever the input porosity is MISSING
        }
        // Where the ceiling is MISSING (e.g. trend with no depth), pass porosity through uncapped.
        capped[i] = (if is_missing(phi_max) { pv } else { pv.min(phi_max) }) as f32;
    }
    HashMap::from([("PHI_CAP".to_string(), capped), ("PHI_MAX".to_string(), ceiling)])
}

// ---------------------------------------------------------------------------
// FTEMP_GRAD — Formation temperature from gradient or BHT interpolation
// ---------------------------------------------------------------------------

fn ftemp_grad_spec() -> ModuleSpec {
    ModuleSpec {
        name: "ftemp_grad".into(),
        title: "Formation Temperature".into(),
        category: "Prep".into(),
        doc: "GRADIENT: FTEMP = TSURF + TGRAD*depth. BHT: linear interpolation from surface \
              temperature to bottom-hole temperature at TD_BHT."
            .into(),
        args: vec![
            opt("OPT_FT", "Temperature model", "GRADIENT", &["GRADIENT", "BHT"]),
            // All four define ONE temperature profile for the well — see ArgSpec::well_scope.
            param_open_well(
                "TSURF",
                "Surface temperature (whole well)",
                "degC",
                0.0,
                50.0,
            ),
            param_open_well_when(
                "TGRAD",
                "Temperature gradient (whole well)",
                "degC/m",
                0.005,
                0.1,
                &[("OPT_FT", "GRADIENT")],
                "docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters",
            ),
            param_open_well_when(
                "BHT",
                "Bottom hole temperature (whole well)",
                "degC",
                30.0,
                250.0,
                &[("OPT_FT", "BHT")],
                "docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters",
            ),
            param_open_well_when(
                "TD_BHT",
                "Depth of BHT measurement (whole well)",
                "m",
                100.0,
                10000.0,
                &[("OPT_FT", "BHT")],
                "docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters",
            ),
            log_out("FTEMP", "Formation temperature", "degC"),
        ],
    }
}

fn ftemp_grad(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let bht_mode = ctx.o("OPT_FT") == "BHT";
    let mut ftemp = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = convert_depth(depth[i] as f64, ctx.depth_unit, DepthUnit::Metres);
        if is_missing(d) {
            continue;
        }
        let tsurf = ctx.p("TSURF", i);
        let t = if bht_mode {
            let bht = ctx.p("BHT", i);
            let td = ctx.p("TD_BHT", i);
            // A zone override can push TD_BHT <= 0 past the dialog's 100..10000 range; the
            // division would then yield a finite-looking +/-Infinity FTEMP that is_missing()
            // (NaN-only) never catches. Skip such samples (mirrors condflag/gascorr guards).
            if td <= 0.0 {
                continue;
            }
            tsurf + (bht - tsurf) * d / td
        } else {
            tsurf + ctx.p("TGRAD", i) * d
        };
        ftemp[i] = t as f32;
    }
    HashMap::from([("FTEMP".to_string(), ftemp)])
}

// ---------------------------------------------------------------------------
// PRECALC — reservoir-condition pre-calculation: FTEMP, FPRESS, RMF, CT, CXO
// (ROADMAP §4c item 17)
// ---------------------------------------------------------------------------

fn precalc_spec() -> ModuleSpec {
    ModuleSpec {
        name: "precalc".into(),
        title: "Pre-Calculation (P / T / Rmf / Ct / Cxo)".into(),
        category: "Prep".into(),
        doc: "Reservoir-condition inputs for saturation and SandiMin work, from trend fits: \
              formation temperature = SURF_TEMP + TEMP_GRAD*TVDSS and FPRESS = PSURF + \
              PGRAD*TVDSS, both linear in true vertical depth. Gradients — and the TREND \
              fit below — are per depth unit of the TVDSS curve: enter per-metre values \
              (and a metric refit) for metric wells; no study fit ships as a generic default. \
              SURF_TEMP / TEMP_GRAD / RMF_TEMP are entered in OPT_TU \
              units, but the FTEMP curve is always written in degC (the unit every \
              downstream module assumes); FTEMP_F is the same trend in degF for SandiMin \
              fluid-property entry. RMF at formation temperature comes either from a \
              surface mud-filtrate measurement Arps-converted per sample (ARPS) or from a \
              field regression RMF = RMF_A + RMF_B*log10(TVDSS) already fit at formation \
              temperature (TREND, for wells with no mud data). CT = 1000/RT and CXO = \
              1000/RXO are QC/plotting conductivities in mmho/m — SandiMin's CT/CXO tool \
              rows read the RESISTIVITY curves directly and convert internally, so do not \
              feed these curves to them. No TVDSS curve → measured DEPTH is used instead \
              (fine for near-vertical wells)."
            .into(),
        args: vec![
            opt("OPT_TU", "Temperature unit for entered params", "degF", &["degF", "degC"]),
            // The geothermal trend is one trend for the well — a named-zone override would step
            // the temperature at a formation top rather than bend it. See ArgSpec::well_scope.
            // PSURF/PGRAD below deliberately stay per-zone: a pressure compartment is real.
            param_open_well("SURF_TEMP", "Surface temperature (intercept, whole well)", "degF|degC", -50.0, 150.0),
            param_open_well("TEMP_GRAD", "Temperature gradient per TVDSS unit (whole well)", "deg/ft|m", 0.0005, 0.2),
            param_open("PSURF", "Formation pressure intercept", "psi", -500.0, 5000.0, true),
            param_open("PGRAD", "Pressure gradient per TVDSS unit", "psi/ft|m", 0.05, 5.0, true),
            opt("OPT_RMF", "RMF source", "ARPS", &["ARPS", "TREND"]),
            param_open_when(
                "RMF_MEAS", "Rmf measured at surface (ARPS)", "ohmm", 0.001, 20.0,
                &[("OPT_RMF", "ARPS")],
                "docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters",
            ),
            param_open_when(
                "RMF_TEMP", "Rmf measurement temperature (ARPS)", "degF|degC", -50.0, 150.0,
                &[("OPT_RMF", "ARPS")],
                "docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters",
            ),
            param_open_when(
                "RMF_A", "RMF trend intercept (TREND, ft-based fit)", "ohmm", 0.0, 5.0,
                &[("OPT_RMF", "TREND")],
                "docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters",
            ),
            param_open_when(
                "RMF_B",
                "RMF trend slope on log10(TVDSS) (TREND — fit must use the TVDSS curve's depth unit)",
                "ohmm",
                -2.0,
                2.0,
                &[("OPT_RMF", "TREND")],
                "docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters",
            ),
            log_in("TVDSS", "True vertical depth subsea", "ft|m", "TVDSS", false),
            log_in("RT", "Deep resistivity", "ohmm", "RES_DEEP", false),
            log_in("RXO", "Flushed-zone resistivity", "ohmm", "RXO", false),
            log_out("FTEMP", "Formation temperature (always degC)", "degC"),
            log_out("FTEMP_F", "Formation temperature in degF (SandiMin fluid entry)", "degF"),
            log_out("FPRESS", "Formation pressure", "psi"),
            log_out("RMF", "Mud filtrate resistivity at FTEMP", "ohmm"),
            log_out("CT", "Deep conductivity 1000/RT (QC/plotting)", "mmho/m"),
            log_out("CXO", "Flushed conductivity 1000/RXO (QC/plotting)", "mmho/m"),
        ],
    }
}

fn precalc(ctx: &ModuleContext) -> ModuleOutputs {
    // TVDSS falls back to measured depth as a whole curve, never per sample:
    // mixing MD and TVD samples would put artificial kinks in the trends.
    let tvd_in = ctx.log("TVDSS");
    let tvd: Vec<f32> = if tvd_in.iter().any(|v| v.is_finite()) {
        tvd_in
    } else {
        record_degradation_once(
            RunDegradationKind::SubstitutedInput,
            "TVDSS was absent, so measured DEPTH supplied the whole temperature-pressure frame",
        );
        ctx.log("DEPTH")
    };
    let rt = ctx.log("RT");
    let rxo = ctx.log("RXO");
    let degc = ctx.o("OPT_TU") == "degC";
    let trend = ctx.o("OPT_RMF") == "TREND";
    let to_f = |t: f64| if degc { t * 1.8 + 32.0 } else { t };

    let mut ftemp = vec![f32::NAN; ctx.n];
    let mut ftemp_f = vec![f32::NAN; ctx.n];
    let mut fpress = vec![f32::NAN; ctx.n];
    let mut rmf = vec![f32::NAN; ctx.n];
    let mut ct = vec![f32::NAN; ctx.n];
    let mut cxo = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let d = tvd[i] as f64;
        if !is_missing(d) {
            // t is in the entered (OPT_TU) unit; the FTEMP curve is ALWAYS degC —
            // every consumer of the FTEMP mnemonic (nphi_env_corr, Rw resolution,
            // ftemp_grad) assumes degC. FTEMP_F carries the degF twin.
            let t = ctx.p("SURF_TEMP", i) + ctx.p("TEMP_GRAD", i) * d;
            let t_f = to_f(t);
            ftemp[i] = (if degc { t } else { (t - 32.0) / 1.8 }) as f32;
            ftemp_f[i] = t_f as f32;
            fpress[i] = (ctx.p("PSURF", i) + ctx.p("PGRAD", i) * d) as f32;
            let r = if trend {
                if d > 0.0 { ctx.p("RMF_A", i) + ctx.p("RMF_B", i) * d.log10() } else { MISSING }
            } else {
                crate::multimin2::arps_f(ctx.p("RMF_MEAS", i), to_f(ctx.p("RMF_TEMP", i)), t_f)
            };
            // Non-positive resistivity is physically meaningless — leave MISSING.
            if r > 0.0 {
                rmf[i] = r as f32;
            }
        }
        let rd = rt[i] as f64;
        if rd > 0.0 {
            ct[i] = (1000.0 / rd) as f32;
        }
        let rx = rxo[i] as f64;
        if rx > 0.0 {
            cxo[i] = (1000.0 / rx) as f32;
        }
    }

    HashMap::from([
        ("FTEMP".to_string(), ftemp),
        ("FTEMP_F".to_string(), ftemp_f),
        ("FPRESS".to_string(), fpress),
        ("RMF".to_string(), rmf),
        ("CT".to_string(), ct),
        ("CXO".to_string(), cxo),
    ])
}

// ---------------------------------------------------------------------------
// BADHOLE — bad-hole / washout QC flag from density correction and caliper
// ---------------------------------------------------------------------------

fn badhole_spec() -> ModuleSpec {
    ModuleSpec {
        name: "badhole".into(),
        title: "Bad-Hole QC Flag".into(),
        category: "Prep".into(),
        doc: "BADHOLE = 1 where the borehole departs from gauge or the density correction is large \
              enough to distrust the porosity logs: |DRHO| > DRHO_MAX, or |CALI - bit size| > \
              DCAL_MAX. Bit size comes from the BS curve where present, or the interpreter's \
              optional BS_INPUT; no value is substituted when both are absent, so only DRHO can \
              be evaluated. The flag is 0 in good hole and MISSING where no QC criterion can be \
              evaluated. The two \
              BADHOLE_*_EVALUATED companions record criterion availability with 1 = evaluated and \
              0 = unavailable; they are not the separate cause/sign channels. Feed BADHOLE to any \
              module run as a mask so flagged intervals go missing instead of polluting results."
            .into(),
        args: vec![
            ArgSpec {
                required: false,
                ..opt_labelled(
                    "DRHO_MAX_UNIT",
                    "Unit of the density-correction threshold; required when DRHO is present",
                    "",
                    &[("g/cc", "g/cc"), ("kg/m3", "kg/m3")],
                )
            },
            param_open(
                "DRHO_MAX",
                "Max acceptable density correction",
                "",
                0.0,
                0.5,
                true,
            ),
            param_open(
                "DCAL_MAX",
                "Max acceptable absolute caliper departure from bit size",
                "in",
                0.0,
                12.0,
                true,
            ),
            ArgSpec {
                min: None,
                max: None,
                ..param_open(
                    "BS_INPUT",
                    "Optional explicit bit size when the BS curve is absent — blank means unavailable",
                    "in",
                    0.0,
                    0.0,
                    false,
                )
            },
            log_in("DRHO", "Density correction log", "g/cc", "DRHO", false),
            log_in("CALI", "Caliper log", "in", "CALI", false),
            log_in("BS", "Bit size log", "in", "BS", false),
            log_out_flag(
                "BADHOLE",
                "Bad-hole flag (1 = bad, 0 = good)",
                FlagKind::ExclusionMask,
            ),
            log_out_flag(
                "BADHOLE_CALI_EVALUATED",
                "Caliper criterion availability (1 = evaluated, 0 = unavailable)",
                FlagKind::DiagnosticIndicator,
            ),
            log_out_flag(
                "BADHOLE_DRHO_EVALUATED",
                "Density-correction criterion availability (1 = evaluated, 0 = unavailable)",
                FlagKind::DiagnosticIndicator,
            ),
        ],
    }
}

fn badhole(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let drho = ctx.log("DRHO");
    if drho.iter().take(ctx.n).any(|value| value.is_finite()) {
        let curve_unit = ctx.input_unit("DRHO").trim();
        if curve_unit.is_empty() {
            return Err(format!(
                "DRHO unit is missing for input curve '{}'; declare g/cc or kg/m3 before badhole compares it with DRHO_MAX",
                ctx.in_curve("DRHO")
            ));
        }
        let threshold_unit = ctx.o("DRHO_MAX_UNIT").trim();
        if threshold_unit.is_empty() {
            return Err(format!(
                "DRHO_MAX unit is missing while DRHO is present in {curve_unit}; state the threshold unit before badhole runs"
            ));
        }
        let curve_token = crate::curves::resolve_unit_token(curve_unit).ok_or_else(|| {
            format!(
                "DRHO input curve '{}' declares unsupported unit '{curve_unit}'; declare g/cc or kg/m3 before badhole runs",
                ctx.in_curve("DRHO")
            )
        })?;
        let threshold_token = crate::curves::resolve_unit_token(threshold_unit).ok_or_else(|| {
            format!(
                "DRHO_MAX declares unsupported unit '{threshold_unit}'; state g/cc or kg/m3 before badhole runs"
            )
        })?;
        if curve_token.canonical_unit != threshold_token.canonical_unit {
            return Err(format!(
                "DRHO unit mismatch: input curve '{}' is declared {curve_unit}, but DRHO_MAX is declared {threshold_unit}; badhole refused before comparing values",
                ctx.in_curve("DRHO")
            ));
        }
    }
    let cali = ctx.log("CALI");
    let bs = ctx.log("BS");
    let mut flag = FlagCurve::missing(ctx.n);
    let mut cali_evaluated = FlagCurve::clear(ctx.n);
    let mut drho_evaluated = FlagCurve::clear(ctx.n);

    for i in 0..ctx.n {
        let dr = drho[i] as f64;
        let cl = cali[i] as f64;
        let drho_max = ctx.p("DRHO_MAX", i);
        let dcal_max = ctx.p("DCAL_MAX", i);
        let bit = {
            let b = bs[i] as f64;
            if is_missing(b) { ctx.p("BS_INPUT", i) } else { b }
        };

        let mut any = false;
        let mut bad = false;
        if !is_missing(dr) {
            any = true;
            drho_evaluated.set(i, FlagValue::Flagged);
            if dr.abs() > drho_max {
                bad = true;
            }
        }
        if !is_missing(cl) && !is_missing(bit) {
            any = true;
            cali_evaluated.set(i, FlagValue::Flagged);
            if (cl - bit).abs() > dcal_max {
                bad = true;
            }
        }
        if any {
            flag.set(
                i,
                if bad {
                    FlagValue::Flagged
                } else {
                    FlagValue::Clear
                },
            );
        }
    }

    Ok(HashMap::from([
        ("BADHOLE".to_string(), flag.into_f32()),
        (
            "BADHOLE_CALI_EVALUATED".to_string(),
            cali_evaluated.into_f32(),
        ),
        (
            "BADHOLE_DRHO_EVALUATED".to_string(),
            drho_evaluated.into_f32(),
        ),
    ]))
}

// ---------------------------------------------------------------------------
// CONDFLAG — data-conditioning flags: coal / tight / gas crossover, plus a
// shoulder adjustment so lithology transitions don't survive the mask
// ---------------------------------------------------------------------------

fn condflag_spec() -> ModuleSpec {
    ModuleSpec {
        name: "condflag".into(),
        title: "Data Conditioning Flags".into(),
        category: "Prep".into(),
        doc: "Flags samples whose density/neutron readings should not feed porosity or \
              mineral solving. COAL_FLAG: RHOB < COAL_RHOB and NPHI > COAL_NPHI, plus DT > \
              COAL_DT where a sonic exists; a washed-out hole mimics coal, so samples with \
              BADHOLE = 1 are never called coal. TIGHT_FLAG: density porosity (from RHO_MA \
              / RHO_FL — the same parameters, and zone overrides, as the density-porosity \
              modules) and NPHI both below TIGHT_PHI. XOVER_FLAG: gas crossover, density \
              porosity exceeding NPHI by more than XOVER_MIN — coal and bad hole are \
              excluded because they fake the same light-density signature. NPHI must be in \
              matrix units consistent with RHO_MA: limestone-unit neutron against a \
              sandstone RHO_MA reads about 0.04 low in clean water sand, right at the \
              XOVER_MIN threshold — convert the neutron first, then supply a sourced threshold \
              for the declared neutron convention. \
              Flagged beds thinner than MIN_THICK are dropped as spikes (missing samples \
              inside a bed do not split it). SHOULDER_FLAG is the transition adjustment: \
              logs average across bed boundaries, so samples within SHOULDER of a coal / \
              tight bed — or a bad-hole interval at least MIN_THICK thick — still carry \
              mixed readings; masking only the bed itself would leave those shoulder values \
              in the conditioned data. COND_FLAG combines coal, tight, bad hole and \
              shoulder (and crossover when OPT_XCOND = YES — leave NO when gas zones will \
              be corrected rather than discarded); feed it as the Mask on later module \
              runs, but leave the Mask empty on the condflag run itself — masking this run \
              with BADHOLE would blank COND_FLAG exactly where it must read 1. MIN_THICK \
              and SHOULDER are in the depth curve's declared unit and ship absent. Run the \
              badhole module first so its flag is \
              available here."
            .into(),
        args: vec![
            with_sources(
                param(
                    "RHO_MA", "Matrix density", "g/cc", 2.65, 2.0, 3.2,
                    "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.",
                ),
                crate::param_sources::MATRIX_DENSITY,
            ),
            param(
                "RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5,
                "IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1",
            ),
            param_open("COAL_RHOB", "Coal: density below", "g/cc", 1.2, 2.4, true),
            param_open("COAL_NPHI", "Coal: neutron above", "v/v", 0.15, 0.8, true),
            param_open("COAL_DT", "Coal: sonic above (when DT present)", "us/ft", 70.0, 160.0, true),
            param_open("TIGHT_PHI", "Tight: both porosities below", "v/v", 0.0, 0.2, true),
            param_open(
                "XOVER_MIN",
                "Crossover: DPHI - NPHI above (~0.08 for limestone-unit NPHI)",
                "v/v",
                0.0,
                0.3,
                true,
            ),
            param_open(
                "MIN_THICK",
                "Drop flagged beds thinner than",
                PROJECT_DEPTH_UNIT_TOKEN,
                0.0,
                10.0,
                true,
            ),
            param_open(
                "SHOULDER",
                "Shoulder width beyond bed edges",
                PROJECT_DEPTH_UNIT_TOKEN,
                0.0,
                5.0,
                true,
            ),
            opt("OPT_XCOND", "Include gas crossover in COND_FLAG", "NO", &["NO", "YES"]),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity log (matrix units matching RHO_MA)", "v/v", "NPHI", true),
            log_in("DT", "Sonic transit time log", "us/ft", "DT", false),
            log_in("BADHOLE", "Bad-hole flag from the badhole module", "", "BADHOLE", false),
            log_out_flag(
                "COAL_FLAG",
                "Coal flag (1 = coal)",
                FlagKind::DiagnosticIndicator,
            ),
            log_out_flag(
                "TIGHT_FLAG",
                "Tight-zone flag (1 = tight)",
                FlagKind::DiagnosticIndicator,
            ),
            log_out_flag(
                "XOVER_FLAG",
                "Gas crossover flag (1 = crossover)",
                FlagKind::DiagnosticIndicator,
            ),
            log_out_flag(
                "SHOULDER_FLAG",
                "Bed-transition shoulder flag (1 = shoulder)",
                FlagKind::DiagnosticIndicator,
            ),
            log_out_flag(
                "COND_FLAG",
                "Combined conditioning mask (1 = exclude)",
                FlagKind::ExclusionMask,
            ),
        ],
    }
}

/// Consecutive runs of [`FlagValue::Flagged`] (missing breaks a run), as inclusive index pairs.
fn flag_runs(flag: &FlagCurve) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start: Option<usize> = None;
    for (i, state) in flag.values.iter().copied().enumerate() {
        if state == FlagValue::Flagged {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            runs.push((s, i - 1));
        }
    }
    if let Some(s) = start {
        runs.push((s, flag.values.len() - 1));
    }
    runs
}

/// Runs of flag == 1.0 with fragments merged when only missing samples separate
/// them — a null reading inside a bed must not split it into despikable slivers.
fn bridged_runs(flag: &FlagCurve) -> Vec<(usize, usize)> {
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in flag_runs(flag) {
        if let Some(last) = merged.last_mut() {
            if flag.values[last.1 + 1..s]
                .iter()
                .all(|state| *state == FlagValue::Missing)
            {
                last.1 = e;
                continue;
            }
        }
        merged.push((s, e));
    }
    merged
}

/// Median sample spacing of the depth curve (0.0 when it cannot be measured).
fn median_spacing(depth: &[f32]) -> f64 {
    let mut d: Vec<f64> = depth
        .windows(2)
        .filter(|w| w[0].is_finite() && w[1].is_finite())
        .map(|w| (w[1] - w[0]).abs() as f64)
        .collect();
    if d.is_empty() {
        return 0.0;
    }
    d.sort_by(|a, b| a.total_cmp(b));
    d[d.len() / 2]
}

fn condflag(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let depth = ctx.log("DEPTH");
    let rhob = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let dt = ctx.log("DT");
    let bh = FlagCurve::from_f32(ctx.log("BADHOLE"), "condflag.BADHOLE input")?;
    let xcond = ctx.o("OPT_XCOND") == "YES";

    let mut coal = FlagCurve::missing(ctx.n);
    let mut tight = FlagCurve::missing(ctx.n);
    let mut xover = FlagCurve::missing(ctx.n);

    for i in 0..ctx.n {
        let (rb, np) = (rhob[i] as f64, nphi[i] as f64);
        if is_missing(rb) || is_missing(np) {
            continue;
        }
        let washout = bh.is_flagged(i);
        let d = dt[i] as f64;
        let coal_hit = rb < ctx.p("COAL_RHOB", i)
            && np > ctx.p("COAL_NPHI", i)
            && (is_missing(d) || d > ctx.p("COAL_DT", i));
        coal.set(
            i,
            if coal_hit && !washout {
                FlagValue::Flagged
            } else {
                FlagValue::Clear
            },
        );

        // Zone overrides bypass dialog range checks, so a degenerate matrix/fluid
        // pair (den <= 0) can still arrive: DPHI is meaningless then — leave the
        // density-porosity flags missing rather than flagging on +/-inf.
        let den = ctx.p("RHO_MA", i) - ctx.p("RHO_FL", i);
        if den <= 0.0 {
            continue;
        }
        let dphi = (ctx.p("RHO_MA", i) - rb) / den;

        let tp = ctx.p("TIGHT_PHI", i);
        tight.set(
            i,
            if dphi < tp && np < tp {
                FlagValue::Flagged
            } else {
                FlagValue::Clear
            },
        );

        let x_hit = dphi - np > ctx.p("XOVER_MIN", i) && !coal_hit && !washout;
        xover.set(
            i,
            if x_hit {
                FlagValue::Flagged
            } else {
                FlagValue::Clear
            },
        );
    }

    // Spike removal: a one- or two-sample "bed" is log noise, not lithology.
    // Thickness counts one sample spacing beyond the run's depth extent; runs are
    // bridged across missing samples so a null inside a bed can't shave it thin.
    let dz = median_spacing(&depth);
    for flag in [&mut coal, &mut tight, &mut xover] {
        for (s, e) in bridged_runs(flag) {
            let min_thick = ctx.p("MIN_THICK", s);
            let extent = (depth[e] - depth[s]).abs() as f64 + dz;
            if extent.is_finite() && min_thick > 0.0 && extent < min_thick {
                for index in s..=e {
                    if flag.is_flagged(index) {
                        flag.set(index, FlagValue::Clear);
                    }
                }
            }
        }
    }

    // A bad-hole interval only earns shoulders when it is a real bed (>= MIN_THICK):
    // a single-sample DRHO blip masks itself via COND_FLAG, but dilating around it
    // would throw away good rock on both sides.
    let mut bh_bed = FlagCurve::clear(ctx.n);
    for (s, e) in bridged_runs(&bh) {
        let min_thick = ctx.p("MIN_THICK", s);
        let extent = (depth[e] - depth[s]).abs() as f64 + dz;
        if !extent.is_finite() || min_thick <= 0.0 || extent >= min_thick {
            for index in s..=e {
                if bh.is_flagged(index) {
                    bh_bed.set(index, FlagValue::Flagged);
                }
            }
        }
    }

    // Shoulder adjustment: walk outward from every coal/tight/bad-hole bed edge
    // and flag samples still within SHOULDER of the boundary — their readings
    // average the two lithologies and would pollute results if left unmasked.
    let mut bed = FlagCurve::clear(ctx.n);
    for index in 0..ctx.n {
        if coal.is_flagged(index) || tight.is_flagged(index) || bh_bed.is_flagged(index) {
            bed.set(index, FlagValue::Flagged);
        }
    }
    let mut shoulder = FlagCurve::missing(ctx.n);
    for i in 0..ctx.n {
        if depth[i].is_finite() {
            shoulder.set(i, FlagValue::Clear);
        }
    }
    for (s, e) in flag_runs(&bed) {
        let sh_top = ctx.p("SHOULDER", s);
        if sh_top > 0.0 && depth[s].is_finite() {
            let mut j = s;
            while j > 0 {
                j -= 1;
                if bed.is_flagged(j)
                    || !depth[j].is_finite()
                    || ((depth[s] - depth[j]).abs() as f64) > sh_top
                {
                    break;
                }
                shoulder.set(j, FlagValue::Flagged);
            }
        }
        let sh_base = ctx.p("SHOULDER", e);
        if sh_base > 0.0 && depth[e].is_finite() {
            let mut j = e;
            while j + 1 < ctx.n {
                j += 1;
                if bed.is_flagged(j)
                    || !depth[j].is_finite()
                    || ((depth[j] - depth[e]).abs() as f64) > sh_base
                {
                    break;
                }
                shoulder.set(j, FlagValue::Flagged);
            }
        }
    }

    let mut cond = FlagCurve::missing(ctx.n);
    for i in 0..ctx.n {
        let parts = [
            coal.get(i),
            tight.get(i),
            bh.get(i),
            if xcond {
                xover.get(i)
            } else {
                FlagValue::Missing
            },
        ];
        if parts.contains(&FlagValue::Flagged) || shoulder.is_flagged(i) {
            cond.set(i, FlagValue::Flagged);
        } else if parts.iter().any(|state| *state != FlagValue::Missing) {
            // Shoulder alone never marks a sample evaluable: with no QC input at
            // all the combined flag stays MISSING, matching the badhole module.
            cond.set(i, FlagValue::Clear);
        }
    }

    Ok(HashMap::from([
        ("COAL_FLAG".to_string(), coal.into_f32()),
        ("TIGHT_FLAG".to_string(), tight.into_f32()),
        ("XOVER_FLAG".to_string(), xover.into_f32()),
        ("SHOULDER_FLAG".to_string(), shoulder.into_f32()),
        ("COND_FLAG".to_string(), cond.into_f32()),
    ]))
}

// ---------------------------------------------------------------------------
// NPHIMAT — neutron matrix conversion via the chartbook porosity-equivalence
// curves (Por-5 CNL thermal, Por-4 APS epithermal), digitized at vector
// precision into neutron_charts.rs.
// ---------------------------------------------------------------------------

fn nphimat_spec() -> ModuleSpec {
    ModuleSpec {
        name: "nphimat".into(),
        title: "Neutron Matrix Conversion".into(),
        category: "Prep".into(),
        doc: "Converts a neutron porosity log recorded in one matrix convention into all \
              three (NPHI_LS / NPHI_SS / NPHI_DOL), using the chartbook porosity-equivalence \
              curves: Por-5 for the CNL thermal tools (NPHI ratio method; TNPH \
              environmentally corrected, with 0 and 250,000 ppm salinity variants) and Por-4 \
              for the epithermal tools — APLC and FPLC (APS) plus the legacy sidewall SNP. \
              Limestone units ARE apparent limestone porosity — the chart's x-axis, on which \
              calcite is the identity — so an SS or DOL input is first inverted back to that \
              axis, then read out along each matrix curve; the input convention passes \
              through unchanged. Feed the output whose matrix matches your RHO_MA into \
              density-neutron work (NPHI_SS with RHO_MA 2.65) — that removes the \
              limestone-vs-sandstone convention offset before a sourced XOVER_MIN is applied. \
              SALINITY picks the TNPH curve pair only; the other \
              tools have a single chart curve. Apply environmental corrections \
              (nphi_env_corr) before converting — the charts assume corrected logs. The \
              limestone axis and dolomite curves are digitized to about -0.02..0.40; the \
              sandstone curves leave the chart top at 40 pu true porosity (~0.32-0.36 \
              apparent limestone), and beyond the data every curve is extended linearly on \
              its end segment. Note NPHI_LS is also a common raw-log mnemonic: after a run, \
              by-name lookups resolve the computed version first (the raw log keeps its \
              provenance in the Curve Catalog)."
            .into(),
        args: vec![
            opt(
                "TOOL",
                "Neutron measurement the log comes from (TNPH/NPHI: CNL thermal, Por-5; APLC/FPLC: APS epithermal and SNP: sidewall neutron, Por-4)",
                "TNPH",
                &["TNPH", "NPHI", "APLC", "FPLC", "SNP"],
            ),
            opt(
                "SALINITY",
                "Formation salinity (TNPH curves only; SALT_250K = 250,000 ppm)",
                "FRESH",
                &["FRESH", "SALT_250K"],
            ),
            opt("MATRIX_IN", "Matrix convention the input log is recorded in", "LS", &["LS", "SS", "DOL"]),
            log_in("NPHI", "Neutron porosity log (v/v, in MATRIX_IN units)", "v/v", "NPHI", true),
            log_out("NPHI_LS", "Neutron porosity, limestone units (apparent limestone)", "v/v"),
            log_out("NPHI_SS", "Neutron porosity, quartz sandstone units", "v/v"),
            log_out("NPHI_DOL", "Neutron porosity, dolomite units", "v/v"),
        ],
    }
}

/// Piecewise-linear read of a digitized chart curve (strictly increasing in both
/// coordinates). `inverse` swaps the axes — true porosity back to apparent
/// limestone. Outside the tabulated span the end segment's slope is extended.
pub(crate) fn chart_lerp(table: &[(f32, f32)], v: f64, inverse: bool) -> f64 {
    let at = |i: usize| -> (f64, f64) {
        let (x, y) = table[i];
        if inverse { (y as f64, x as f64) } else { (x as f64, y as f64) }
    };
    let seg = |i0: usize, i1: usize| -> f64 {
        let (x0, y0) = at(i0);
        let (x1, y1) = at(i1);
        y0 + (v - x0) * (y1 - y0) / (x1 - x0)
    };
    if v <= at(0).0 {
        return seg(0, 1);
    }
    for i in 1..table.len() {
        if v <= at(i).0 {
            return seg(i - 1, i);
        }
    }
    seg(table.len() - 2, table.len() - 1)
}

/// The (sandstone, dolomite) chart tables for a tool choice.
pub(crate) fn nphimat_tables(tool: &str, salt: bool) -> (&'static [(f32, f32)], &'static [(f32, f32)]) {
    use crate::neutron_charts as nc;
    match tool {
        "NPHI" => (nc::CNL_NPHI_SS, nc::CNL_NPHI_DOL),
        "APLC" => (nc::APS_APLC_SS, nc::APS_APLC_DOL),
        "FPLC" => (nc::APS_FPLC_SS, nc::APS_FPLC_DOL),
        "SNP" => (nc::SNP_SS, nc::SNP_DOL),
        _ if salt => (nc::CNL_TNPH_SALT_SS, nc::CNL_TNPH_SALT_DOL),
        _ => (nc::CNL_TNPH_FRESH_SS, nc::CNL_TNPH_FRESH_DOL),
    }
}

fn nphimat(ctx: &ModuleContext) -> ModuleOutputs {
    let np = ctx.log("NPHI");
    let (t_ss, t_dol) = nphimat_tables(ctx.o("TOOL"), ctx.o("SALINITY") == "SALT_250K");
    let matrix_in = ctx.o("MATRIX_IN");

    let mut ls = vec![f32::NAN; ctx.n];
    let mut ss = vec![f32::NAN; ctx.n];
    let mut dol = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let v = np[i] as f64;
        if is_missing(v) {
            continue;
        }
        let app = match matrix_in {
            "SS" => chart_lerp(t_ss, v, true),
            "DOL" => chart_lerp(t_dol, v, true),
            _ => v,
        };
        // The input convention is copied through untouched — a chart round trip
        // would only add interpolation noise to values we already have.
        ls[i] = if matrix_in == "SS" || matrix_in == "DOL" { app as f32 } else { np[i] };
        ss[i] = if matrix_in == "SS" { np[i] } else { chart_lerp(t_ss, app, false) as f32 };
        dol[i] = if matrix_in == "DOL" { np[i] } else { chart_lerp(t_dol, app, false) as f32 };
    }

    HashMap::from([
        ("NPHI_LS".to_string(), ls),
        ("NPHI_SS".to_string(), ss),
        ("NPHI_DOL".to_string(), dol),
    ])
}

// ---------------------------------------------------------------------------
// GASCORR — iterated gas correction of bulk density (iterated-loop method, see below;
// ROADMAP §4c item 19)
// ---------------------------------------------------------------------------

fn gascorr_spec() -> ModuleSpec {
    let mut args = vec![
        with_sources(
            param(
                "RHO_MA", "Matrix density", "g/cc", 2.65, 2.0, 3.2,
                "IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16.",
            ),
            crate::param_sources::MATRIX_DENSITY,
        ),
        param(
            "RHO_FL", "Liquid (filtrate) density the correction restores", "g/cc", 1.0, 0.8, 1.3,
            "Geolog V14 phi_dnh.info RHO_MF DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.4",
        ),
        param_open("SG_GAS", "Gas specific gravity (air = 1)", "", 0.55, 1.2, true),
        param_open("A", "Tortuosity constant", "", 0.1, 5.0, true),
        param_open("M", "Cementation exponent", "", 1.0, 4.0, true),
        param_open("N", "Saturation exponent", "", 1.0, 4.0, true),
        opt("OPT_GATE", "Where to apply the correction", "FLAGGED", &["FLAGGED", "EVERYWHERE"]),
    ];
    // rw_args carries its own optional FTEMP input; gascorr needs FTEMP as a
    // required input for the gas density, so swap that entry for our own.
    args.extend(rw_args().into_iter().filter(|a| a.name != "FTEMP"));
    args.extend([
        log_in("RHOB", "Bulk density", "g/cc", "RHOB", true),
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        // Computed-only: a raw import named FTEMP/FPRESS may be in degF/kPa — only the
        // precalc outputs (or a log set) satisfy the degC/psi contract.
        log_in_computed("FTEMP", "Formation temperature (precalc)", "degC", "FTEMP", true),
        log_in_computed("FPRESS", "Formation pressure (precalc)", "psi", "FPRESS", true),
        log_in("GAS_FLAG", "Gas-zone flag for FLAGGED gating", "", "XOVER_FLAG", false),
        log_out("RHOB_GC", "Gas-corrected bulk density", "g/cc"),
        log_out("PHIT_GC", "Density porosity from the corrected RHOB (converged)", "v/v"),
        log_out("SWT_GC", "Archie SWT at convergence", "v/v"),
        log_out("GASDEN", "Gas density at reservoir P/T (QC)", "g/cc"),
    ]);
    ModuleSpec {
        name: "gascorr".into(),
        title: "Gas Correction (density, iterated)".into(),
        category: "Prep".into(),
        doc: "Removes the gas effect from RHOB (iterated density-neutron loop): density porosity \
              and Archie SWT are solved from the current density, then RHOB_GC = RHOB + \
              PHIT*(1-SWT)*(RHO_FL - GASDEN) replaces the gas volume with liquid, iterated \
              until PHIT moves less than 1e-4 (max 20 passes; non-converging samples stay \
              MISSING). GASDEN is the real-gas density of an SG_GAS gas at FPRESS/FTEMP \
              (Standing pseudo-criticals + Papay z-factor) — run the precalc module first; \
              samples without P/T, RT or Rw stay MISSING rather than passing through \
              uncorrected. The default OPT_GATE = FLAGGED corrects only where GAS_FLAG > \
              0.5 (chain condflag's XOVER_FLAG, which already excludes coal and washout) \
              and errors if the flag curve has no data. OPT_GATE = EVERYWHERE corrects \
              every sample — beware: high-resistivity low-density beds (coal, resistive \
              washouts) read as gas to the Archie loop and get large spurious corrections. \
              QC per slides 66-67: the detached high-porosity gas cloud on PHIE vs \
              wet-clay collapses after correction. Feed RHOB_GC to phi_den (or use \
              PHIT_GC directly) — NOT to phi_dn or a SandiMin solve that includes NPHI: \
              their gas handling assumes an uncorrected density-neutron pair, so a \
              corrected RHOB with a still-gas-affected NPHI biases porosity low."
            .into(),
        args,
    }
}

/// Standing pseudo-criticals + Papay z-factor → real-gas density in g/cc.
/// Returns MISSING when pressure/temperature/gravity are unusable.
fn gas_density_gcc(sg: f64, press_psi: f64, temp_c: f64) -> f64 {
    if !(press_psi > 0.0) || !(temp_c > -273.15) || !(sg > 0.0) {
        return MISSING;
    }
    let t_r = temp_c * 1.8 + 32.0 + 459.67; // degR
    let tpc = 168.0 + 325.0 * sg - 12.5 * sg * sg; // degR (Standing)
    let ppc = 677.0 + 15.0 * sg - 37.5 * sg * sg; // psia (Standing)
    let tpr = t_r / tpc;
    let ppr = press_psi / ppc;
    // Papay (1968); floored — the correlation goes unphysical far outside its range.
    let z = (1.0 - 3.53 * ppr / 10f64.powf(0.9813 * tpr)
        + 0.274 * ppr * ppr / 10f64.powf(0.8157 * tpr))
        .max(0.1);
    // ρ = P·M/(z·R·T): M = 28.9647·SG lb/lb-mol, R = 10.7316 psia·ft³/(lb-mol·°R),
    // then lb/ft³ → g/cc.
    press_psi * 28.9647 * sg / (z * 10.7316 * t_r) / 62.428
}

fn gascorr(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let rhob = ctx.log("RHOB");
    let rt = ctx.log("RT");
    let ftemp = ctx.log("FTEMP");
    let fpress = ctx.log("FPRESS");
    let flag = ctx.log("GAS_FLAG");
    let gated = ctx.o("OPT_GATE") == "FLAGGED";
    // A FLAGGED run whose flag curve resolved to nothing would silently correct zero
    // samples while reporting success — indistinguishable from "no gas anywhere".
    if gated && !flag.iter().any(|v| !v.is_nan()) {
        let mnem = ctx.o("__IN_GAS_FLAG");
        let name = if mnem.is_empty() { "GAS_FLAG" } else { mnem };
        return Err(format!(
            "OPT_GATE = FLAGGED but the gas flag '{name}' has no data — run condflag first or set OPT_GATE = EVERYWHERE"
        ));
    }

    let mut rhob_gc = vec![f32::NAN; ctx.n];
    let mut phit_gc = vec![f32::NAN; ctx.n];
    let mut swt_gc = vec![f32::NAN; ctx.n];
    let mut rhog_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let rb = rhob[i] as f64;
        if is_missing(rb) {
            continue;
        }
        let rma = ctx.p("RHO_MA", i);
        let rfl = ctx.p("RHO_FL", i);
        // Zone overrides bypass dialog range checks (condflag precedent): a degenerate
        // matrix/fluid pair — or a density below the fluid it would be corrected to —
        // has no meaningful density porosity, so outputs stay MISSING.
        if !(rma - rfl > 0.0) || rb <= rfl {
            continue;
        }
        let dphi = |rho: f64| limit((rma - rho) / (rma - rfl), 0.0, 1.0);
        // Gate on > 0.5, not == 1: depth-shifted flags interpolate fractional values at
        // bed edges. MISSING flags (NaN > 0.5 is false) still pass through untouched.
        if gated && !(flag[i] as f64 > 0.5) {
            rhob_gc[i] = rhob[i];
            phit_gc[i] = dphi(rb) as f32;
            continue;
        }
        let rhog = gas_density_gcc(ctx.p("SG_GAS", i), fpress[i] as f64, ftemp[i] as f64);
        let rw = resolve_rw(ctx, &ftemp, i);
        let r = rt[i] as f64;
        if is_missing(rhog) || is_missing(rw) || rw <= 0.0 || is_missing(r) || r <= 0.0 {
            continue; // corrected outputs stay MISSING — no silent pass-through
        }
        let a = ctx.p("A", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        let archie = |p: f64| -> f64 {
            if p <= 0.0 {
                return 1.0;
            }
            let s = (a * rw / (p.powf(m) * r)).powf(1.0 / n_exp);
            // f64::min would swallow a NaN term (it returns the non-NaN operand) and
            // fabricate Sw = 1 — keep the NaN so the sample fails convergence instead.
            if s.is_finite() { s.min(1.0) } else { f64::NAN }
        };

        let mut rho_c = rb;
        let mut phit = dphi(rb);
        let mut converged = false;
        for _ in 0..20 {
            let swt = archie(phit);
            rho_c = rb + phit * (1.0 - swt) * (rfl - rhog);
            let next = dphi(rho_c);
            let done = (next - phit).abs() < 1e-4;
            phit = next;
            if done {
                converged = true;
                break;
            }
        }
        if !converged {
            // Oscillating/diverging iterate (possible at edge-of-range RHO_MA/RHO_FL or
            // M/N combinations): writing the 20th pass would be an internally
            // inconsistent triple masquerading as a converged answer.
            continue;
        }
        rhob_gc[i] = rho_c as f32;
        phit_gc[i] = phit as f32;
        swt_gc[i] = archie(phit) as f32;
        rhog_out[i] = rhog as f32;
    }

    Ok(HashMap::from([
        ("RHOB_GC".to_string(), rhob_gc),
        ("PHIT_GC".to_string(), phit_gc),
        ("SWT_GC".to_string(), swt_gc),
        ("GASDEN".to_string(), rhog_out),
    ]))
}

// ---------------------------------------------------------------------------
// Environmental corrections (pragmatic analytic set). These are
// linearized, coefficient-driven equivalents of the service-company chartbook
// corrections — the coefficients are parameters that ship absent until a cited tool/chart
// value is supplied. Chart-lookup fidelity comes later (ROADMAP).
// Each writes a corrected copy (<LOG>_EC); inputs are never modified. The private arithmetic
// helpers retain their simple missing-value behavior, while the public dispatcher enforces the
// source-bearing coverage conditions below before any correction-named output can escape.
// ---------------------------------------------------------------------------

fn gr_hole_corr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gr_hole_corr".into(),
        title: "GR Hole-Size Correction".into(),
        category: "Prep".into(),
        doc: "GR_EC = GR * (1 + K_GR*(CALI - BS)): linear borehole-enlargement correction — \
              gamma rays attenuated by the extra mud annulus are restored. Bit size from the \
              BS curve where present, else BS_DEF. The public runner refuses if CALI is missing \
              at any finite GR sample; it never writes an unmarked uncorrected GR_EC copy."
            .into(),
        args: vec![
            param_open(
                "K_GR",
                "Correction per inch of enlargement",
                "1/in",
                0.0,
                0.05,
                true,
            ),
            param_open(
                "BS_DEF",
                "Bit size when BS curve is absent",
                "in",
                3.0,
                30.0,
                true,
            ),
            log_in("GR", "Gamma ray log", "gapi", "GR", true),
            with_validity(
                log_in("CALI", "Caliper log", "in", "CALI", false),
                vec![validity(
                    "gr_hole_corr.caliper_coverage",
                    "Caliper is required at every finite GR sample; without it the correction-named output would be an unmarked input copy.",
                    "docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and section 6.2 T11/T12",
                    ValidityRule::RequiredWhereFinite { input: "GR".into() },
                )],
            ),
            log_in("BS", "Bit size log", "in", "BS", false),
            log_out("GR_EC", "Environmentally corrected gamma ray", "gapi"),
        ],
    }
}

fn gr_hole_corr(ctx: &ModuleContext) -> ModuleOutputs {
    let gr = ctx.log("GR");
    let cali = ctx.log("CALI");
    let bs = ctx.log("BS");
    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let g = gr[i] as f64;
        if is_missing(g) {
            continue;
        }
        let cl = cali[i] as f64;
        if is_missing(cl) {
            out[i] = g as f32; // no caliper: pass through
            continue;
        }
        let bit = {
            let b = bs[i] as f64;
            if is_missing(b) { ctx.p("BS_DEF", i) } else { b }
        };
        let enlargement = (cl - bit).max(0.0); // undersize holes get no correction
        out[i] = (g * (1.0 + ctx.p("K_GR", i) * enlargement)) as f32;
    }
    HashMap::from([("GR_EC".to_string(), out)])
}

fn nphi_env_corr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "nphi_env_corr".into(),
        title: "Neutron Environmental Correction".into(),
        category: "Prep".into(),
        doc: "NPHI_EC = NPHI + K_TEMP*(FTEMP - T_REF) + K_SAL*(SALW/100000): linearized \
              formation-temperature and formation-salinity terms whose coefficients must be \
              supplied from the applicable CNL chart. Requires FTEMP (run Formation Temperature first) for the temperature \
              term; without it only the salinity term applies."
            .into(),
        args: vec![
            param_open("K_TEMP", "Temperature coefficient", "v/v per degC", -0.01, 0.01, true),
            param_open("T_REF", "Chart reference temperature", "degC", 0.0, 100.0, true),
            param_open("K_SAL", "Salinity coefficient per 100 kppm", "v/v", -0.05, 0.05, true),
            param_open("SALW", "Formation water salinity", "ppm", 0.0, 300000.0, true),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", true),
            // FTEMP must come from precalc/ftemp_grad COMPUTED output, not a raw LAS curve — a raw
            // degF FTEMP would otherwise be silently applied as degC. Mirrors gascorr's contract.
            log_in_computed("FTEMP", "Formation temperature (precalc)", "degC", "FTEMP", false),
            log_out("NPHI_EC", "Environmentally corrected neutron porosity", "v/v"),
        ],
    }
}

fn nphi_env_corr(ctx: &ModuleContext) -> ModuleOutputs {
    let nphi = ctx.log("NPHI");
    let ftemp = ctx.log("FTEMP");
    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let np = nphi[i] as f64;
        if is_missing(np) {
            continue;
        }
        let mut corr = ctx.p("K_SAL", i) * ctx.p("SALW", i) / 100000.0;
        let ft = ftemp[i] as f64;
        if !is_missing(ft) {
            corr += ctx.p("K_TEMP", i) * (ft - ctx.p("T_REF", i));
        }
        out[i] = (np + corr) as f32;
    }
    HashMap::from([("NPHI_EC".to_string(), out)])
}

fn rhob_hole_corr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "rhob_hole_corr".into(),
        title: "Density Hole-Size Correction".into(),
        category: "Prep".into(),
        doc: "RHOB_EC = RHOB + K_RHO*(CALI - HD_REF) for CALI beyond HD_REF: in oversize \
              holes the pad reads too much mud, so density is restored upward using supplied, \
              tool-specific chart values. Within gauge RHOB may remain unchanged; the public \
              runner refuses if CALI is missing at any finite RHOB sample. Use with the BADHOLE flag — beyond a \
              few inches of washout no correction is trustworthy."
            .into(),
        args: vec![
            param_open(
                "K_RHO",
                "Correction per inch beyond reference",
                "g/cc/in",
                0.0,
                0.05,
                true,
            ),
            param_open(
                "HD_REF",
                "Hole diameter where correction starts",
                "in",
                4.0,
                20.0,
                true,
            ),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            with_validity(
                log_in("CALI", "Caliper log", "in", "CALI", false),
                vec![validity(
                    "rhob_hole_corr.caliper_coverage",
                    "Caliper is required at every finite RHOB sample; without it the correction-named output would be an unmarked input copy.",
                    "docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and section 6.2 T12",
                    ValidityRule::RequiredWhereFinite { input: "RHOB".into() },
                )],
            ),
            log_out("RHOB_EC", "Environmentally corrected density", "g/cc"),
        ],
    }
}

fn rhob_hole_corr(ctx: &ModuleContext) -> ModuleOutputs {
    let rhob = ctx.log("RHOB");
    let cali = ctx.log("CALI");
    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let r = rhob[i] as f64;
        if is_missing(r) {
            continue;
        }
        let cl = cali[i] as f64;
        let corr = if is_missing(cl) {
            0.0
        } else {
            ctx.p("K_RHO", i) * (cl - ctx.p("HD_REF", i)).max(0.0)
        };
        out[i] = (r + corr) as f32;
    }
    HashMap::from([("RHOB_EC".to_string(), out)])
}

// ---------------------------------------------------------------------------
// Shared Rw resolution (Loglan sw_*.lls): constant, Arps-corrected measurement,
// or salinity conversion (Bateman-Konen / Kennedy).
// ---------------------------------------------------------------------------

fn rw_args() -> Vec<ArgSpec> {
    vec![
        opt(
            "OPT_RW",
            "Formation water resistivity source",
            "CONSTANT",
            &["CONSTANT", "MEASURED", "SALINITY"],
        ),
        with_sources(param_open_when(
            "RW",
            "Rw at formation temperature (CONSTANT)",
            "ohmm",
            0.001,
            20.0,
            &[("OPT_RW", "CONSTANT")],
            "docs/PRD_v2/12_saturation.md §5 formation-water parameters",
        ), crate::param_sources::FORMATION_WATER_RESISTIVITY),
        param_open_when(
            "RWS",
            "Measured water sample resistivity",
            "ohmm",
            0.001,
            20.0,
            &[("OPT_RW", "MEASURED")],
            "docs/PRD_v2/12_saturation.md §5 formation-water parameters",
        ),
        param_open_when(
            "RWT",
            "Temperature of RWS measurement",
            "degC",
            0.0,
            150.0,
            &[("OPT_RW", "MEASURED")],
            "docs/PRD_v2/12_saturation.md §5 formation-water parameters",
        ),
        param_open_when(
            "SALW",
            "Formation water salinity",
            "ppm",
            100.0,
            300000.0,
            &[("OPT_RW", "SALINITY")],
            "docs/PRD_v2/12_saturation.md §5 formation-water parameters",
        ),
        log_in(
            "FTEMP",
            "Formation temperature (for MEASURED/SALINITY)",
            "degC",
            "FTEMP",
            false,
        ),
    ]
}

fn resolve_rw(ctx: &ModuleContext, ftemp: &[f32], i: usize) -> f64 {
    match ctx.o("OPT_RW") {
        "MEASURED" => {
            let ft = ftemp[i] as f64;
            if is_missing(ft) {
                return MISSING;
            }
            ctx.p("RWS", i) * (ctx.p("RWT", i) + 21.5) / (ft + 21.5)
        }
        "SALINITY" => {
            let ft = ftemp[i] as f64;
            let salw = ctx.p("SALW", i);
            if is_missing(ft) || is_missing(salw) {
                return MISSING;
            }
            // Kennedy above 39161 ppm, Bateman-Konen below (Loglan sw_arch.lls).
            if salw > 39161.0 {
                let rw75 = if salw <= 275000.0 {
                    1.0 / (24.30853
                        - 0.0364 * ((salw / 10000.0) - 29.46518957)
                        - 0.02922 * ((salw / 10000.0) - 29.46518957).powi(2))
                } else {
                    0.0412
                };
                rw75 * (75.0 + 6.77) / ((1.8 * ft + 32.0) + 6.77)
            } else {
                let rw75 = 0.0123 + 3647.5 / salw.powf(0.955);
                rw75 * (23.9 + 21.5) / (ft + 21.5)
            }
        }
        _ => ctx.p("RW", i), // CONSTANT
    }
}

// ---------------------------------------------------------------------------
// SW_ARCH — Water saturation, Archie (Loglan sw_arch.lls)
// ---------------------------------------------------------------------------

fn sw_arch_spec() -> ModuleSpec {
    let mut args = vec![
        with_sources(param_open("A", "Tortuosity constant", "", 0.1, 5.0, true), crate::param_sources::ARCHIE_A),
        with_sources(param_open("M", "Cementation exponent", "", 1.0, 4.0, true), crate::param_sources::ARCHIE_M),
        with_sources(param_open("N", "Saturation exponent", "", 1.0, 4.0, true), crate::param_sources::ARCHIE_N),
        param_open(
            "SWT_IRR",
            "Irreducible total water saturation",
            "v/v",
            0.0,
            0.6,
            true,
        ),
    ];
    args.extend(rw_args());
    args.extend([
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        log_in("PHIT", "Limited total porosity", "v/v", "PHIT", true),
        log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
        log_out("SWT_ARCH", "SWT from Archie (unlimited)", "v/v"),
        log_out("SWT", "Limited total water saturation", "v/v"),
        log_out("SWE", "Limited effective water saturation", "v/v"),
        log_out("VOL_UWAT", "Volume of water (unflushed)", "v/v"),
        log_out("SW_METHOD", "Producing saturation equation (categorical method code)", ""),
    ]);
    ModuleSpec {
        name: "sw_arch".into(),
        title: "SW — Archie".into(),
        category: "Saturation".into(),
        doc: "SWT = (A*Rw / (PHIT^M * RT))^(1/N), on total porosity; SWE derived by removing \
              the shale-bound water fraction. Archie (1942)."
            .into(),
        args,
    }
}

fn sw_arch(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    let phit = ctx.log("PHIT");
    let phie = ctx.log("PHIE");
    let ftemp = ctx.log("FTEMP");
    let mut swt_arch = vec![f32::NAN; ctx.n];
    let mut swt_out = vec![f32::NAN; ctx.n];
    let mut swe_out = vec![f32::NAN; ctx.n];
    let mut vol_uwat = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, pt, pe) = (rt[i] as f64, phit[i] as f64, phie[i] as f64);
        if is_missing(pt) {
            continue;
        }
        // Coal / zero porosity: everything water (standard convention). Keyed on total
        // porosity alone — at pt==0 the formation factor a/pt^m blows up to +inf
        // regardless of PHIE, so we must catch it here even when PHIE is absent (NaN);
        // otherwise SWT_ARCH stores +Infinity and poisons catalog stats/autoscale.
        if pt == 0.0 {
            swt_arch[i] = 1.0;
            swt_out[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = 0.0;
            continue;
        }
        let rw = resolve_rw(ctx, &ftemp, i);
        // RT <= 0 is non-physical (typically a null coded as 0): the Archie ratio
        // a*Rw/(phi^m * RT) diverges to +Infinity, and is_missing() screens only NaN,
        // so +inf would flow into SWT_ARCH and poison catalog min/max + plot autoscale.
        // Drop the sample to missing, matching sw_rtc / sw_imts (lrlc.rs) which already
        // guard rt <= 0. (A negative RT would instead give a NaN via powf of a negative
        // base — also caught here so both invalid cases behave identically.)
        if is_missing(r) || r <= 0.0 || is_missing(rw) {
            continue;
        }
        let a = ctx.p("A", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        let swt_irr = ctx.p("SWT_IRR", i);

        let ff = a / pt.powf(m);
        let swt = (ff * rw / r).powf(1.0 / n_exp);
        swt_arch[i] = swt as f32;
        let swt_l = limit(swt, swt_irr, 1.0);
        swt_out[i] = swt_l as f32;

        if !is_missing(pe) {
            let swtsh = 1.0 - pe / pt;
            let swe = if swtsh >= 1.0 {
                1.0
            } else {
                ((swt - swtsh) / (1.0 - swtsh)).max(0.0)
            };
            let swe_irr = if swtsh >= 1.0 { 0.0 } else { ((swt_irr - swtsh) / (1.0 - swtsh)).max(0.0) };
            let mut swe_l = limit(swe, swe_irr, 1.0);
            // Low effective porosity clean-up (convention: PHIE < 0.005 → all water).
            if pe < 0.005 {
                swe_l = 1.0;
                swt_out[i] = 1.0;
            }
            swe_out[i] = swe_l as f32;
            vol_uwat[i] = (pe * swe_l) as f32;
        }
    }

    let method_flag = swt_arch
        .iter()
        .map(|sw| if sw.is_finite() { crate::multimin2::SwModel::ArchieTotal.flag_code() } else { f32::NAN })
        .collect();
    HashMap::from([
        ("SWT_ARCH".to_string(), swt_arch),
        ("SWT".to_string(), swt_out),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
        ("SW_METHOD".to_string(), method_flag),
    ])
}

// ---------------------------------------------------------------------------
// SW_INDO — Water saturation, Indonesia / Poupon-Leveaux (Loglan sw_indo.lls)
// ---------------------------------------------------------------------------

fn sw_indo_spec() -> ModuleSpec {
    let mut args = vec![
        opt(
            "OPT_INDO",
            "Indonesia VSH exponent variant",
            "FULL",
            &["FULL", "SIMPLE", "TAR_SAND"],
        ),
        with_sources(param_open("A", "Tortuosity constant", "", 0.1, 5.0, true), crate::param_sources::ARCHIE_A),
        with_sources(param_open("M", "Cementation exponent", "", 1.0, 4.0, true), crate::param_sources::ARCHIE_M),
        with_sources(param_open("N", "Saturation exponent", "", 1.0, 4.0, true), crate::param_sources::ARCHIE_N),
        with_sources(param_open("RT_SH", "Shale resistivity", "ohmm", 0.1, 500.0, true), crate::param_sources::SHALE_RESISTIVITY),
        param_open(
            "SWE_IRR",
            "Irreducible effective water saturation",
            "v/v",
            0.0,
            0.6,
            true,
        ),
    ];
    args.extend(rw_args());
    args.extend([
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
        log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
        log_out("SWE_INDO", "SWE from Indonesia (unlimited)", "v/v"),
        log_out("SWE", "Limited effective water saturation", "v/v"),
        log_out("VOL_UWAT", "Volume of water (unflushed)", "v/v"),
        log_out("SW_METHOD", "Producing saturation equation (categorical method code)", ""),
    ]);
    ModuleSpec {
        name: "sw_indo".into(),
        title: "SW — Indonesia (Poupon-Leveaux)".into(),
        category: "Saturation".into(),
        doc: "1/RT = (v/RT_SH + PHIE^M/(A*Rw) + 2*sqrt(v*PHIE^M/(A*Rw*RT_SH))) * SW^N, \
              v = VSH^(2-VSH) (FULL), VSH^2 (SIMPLE), VSH^(2-2*VSH) (TAR_SAND). \
              Poupon & Leveaux (1971)."
            .into(),
        args,
    }
}

fn sw_indo(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    let phie = ctx.log("PHIE");
    let vsh = ctx.log("VSH");
    let ftemp = ctx.log("FTEMP");
    let variant = ctx.o("OPT_INDO").to_string();
    let mut swe_indo = vec![f32::NAN; ctx.n];
    let mut swe_out = vec![f32::NAN; ctx.n];
    let mut vol_uwat = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, pe, vs) = (rt[i] as f64, phie[i] as f64, vsh[i] as f64);
        if is_missing(pe) {
            continue;
        }
        // SB-SAT-030: at VSH -> 1 the water and effective porosity both go to zero, so the
        // answer is degenerate even though nothing divides by zero here. The arithmetic is
        // deliberately left alone — the requirement permits Sw = 1, it forbids Sw = 1 UNFLAGGED.
        if vs >= 1.0 {
            record_degradation_once(
                RunDegradationKind::Clamped,
                "indonesia: VSH >= 1 leaves no effective porosity, so the saturation is \
                 degenerate rather than measured",
            );
        }
        if pe < 0.005 {
            swe_indo[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = pe as f32;
            continue;
        }
        let rw = resolve_rw(ctx, &ftemp, i);
        // RT <= 0 is non-physical: 1/(RT*(...)) diverges to +Infinity and is_missing()
        // screens only NaN, so +inf would poison SWE_INDO stats/autoscale. Drop to missing,
        // matching sw_rtc / sw_imts (lrlc.rs) and sw_arch above.
        if is_missing(r) || r <= 0.0 || is_missing(vs) || is_missing(rw) {
            continue;
        }
        let a = ctx.p("A", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        let rt_sh = ctx.p("RT_SH", i);
        let swe_irr = ctx.p("SWE_IRR", i);

        let v = match variant.as_str() {
            "SIMPLE" => vs.powi(2),
            "TAR_SAND" => vs.powf(2.0 - 2.0 * vs),
            _ => vs.powf(2.0 - vs), // FULL
        };
        let ff = a / pe.powf(m);
        let f1 = 1.0 / (ff * rw);
        let f2 = 2.0 * (v / (rw * ff * rt_sh)).sqrt();
        let f3 = v / rt_sh;
        let swe = (1.0 / (r * (f1 + f2 + f3))).powf(1.0 / n_exp);
        swe_indo[i] = swe as f32;
        let swe_l = limit(swe, swe_irr, 1.0);
        swe_out[i] = swe_l as f32;
        vol_uwat[i] = (pe * swe_l) as f32;
    }

    let method_flag = swe_indo
        .iter()
        .map(|sw| if sw.is_finite() { crate::multimin2::SwModel::Indonesia.flag_code() } else { f32::NAN })
        .collect();
    HashMap::from([
        ("SWE_INDO".to_string(), swe_indo),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
        ("SW_METHOD".to_string(), method_flag),
    ])
}

// ---------------------------------------------------------------------------
// SW_SIM — Water saturation, Simandoux (Loglan sw_sim.lls, Newton-Raphson solver)
// ---------------------------------------------------------------------------

fn sw_sim_spec() -> ModuleSpec {
    let mut args =
        vec![
        opt_labelled(
            "OPT_SIM",
            "Equation identity",
            "simandoux_bardon_pied",
            &[
                (
                    "simandoux_bardon_pied",
                    "simandoux_bardon_pied — Simandoux / Bardon-Pied (Geolog MODIFIED)",
                ),
                (
                    "simandoux_modified_slb",
                    "simandoux_modified_slb — Modified Simandoux / Schlumberger (Geolog SCHLUM)",
                ),
            ],
        ),
        param_open("A", "Tortuosity constant", "", 0.1, 5.0, true),
        param_open("M", "Cementation exponent", "", 1.0, 4.0, true),
        param_open("N", "Saturation exponent", "", 1.0, 4.0, true),
        param(
            "C", "VSH exponent (simandoux_modified_slb only)", "", 1.0, 1.0, 2.0,
            "Geolog V14 sw_sim.info C DEFAULT 1 VALIDATION 1:2; docs/PRD_v2/12_saturation.md §5",
        ),
        param_open("RT_SH", "Shale resistivity", "ohmm", 0.1, 500.0, true),
        param_open("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.0, 0.6, true),
    ];
    args.extend(rw_args());
    args.extend([
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
        log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
        log_out("SWE_SIM", "SWE from the selected typed Simandoux equation (unlimited)", "v/v"),
        log_out("SWE", "Limited effective water saturation", "v/v"),
        log_out("VOL_UWAT", "Volume of water (unflushed)", "v/v"),
        log_out("SW_METHOD", "Producing saturation equation (categorical method code)", ""),
    ]);
    ModuleSpec {
        name: "sw_sim".into(),
        title: "SW — typed Simandoux equations".into(),
        category: "Saturation".into(),
        doc: "Each persisted id names one equation. simandoux_bardon_pied: \
              1/RT = PHIE^M*SWE^N/(A*Rw) + VSH*SWE/RT_SH. \
              simandoux_modified_slb: 1/RT = PHIE^M*SWE^N/(A*Rw*(1-VSH)) + \
              VSH^C*SWE/RT_SH. Legacy vendor tokens are accepted only as input aliases."
            .into(),
        args,
    }
}

fn sw_sim(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    let phie = ctx.log("PHIE");
    let vsh = ctx.log("VSH");
    let ftemp = ctx.log("FTEMP");
    let method = canonical_option_value("sw_sim", "OPT_SIM", ctx.o("OPT_SIM"));
    let modified_slb = method == "simandoux_modified_slb";
    let mut swe_sim = vec![f32::NAN; ctx.n];
    let mut swe_out = vec![f32::NAN; ctx.n];
    let mut vol_uwat = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let (r, pe, vs) = (rt[i] as f64, phie[i] as f64, vsh[i] as f64);
        if is_missing(pe) {
            continue;
        }
        if pe < 0.005 {
            swe_sim[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = pe as f32;
            continue;
        }
        let rw = resolve_rw(ctx, &ftemp, i);
        // RT <= 0 is non-physical: g3 = -1/RT becomes -Infinity and the Newton-Raphson
        // solve diverges to a garbage/MISSING value, silently dropping the sample. Screen it
        // explicitly (matching sw_arch / sw_indo / sw_rtc / sw_imts) instead of relying on
        // the solver to diverge.
        if is_missing(r) || r <= 0.0 || is_missing(vs) || is_missing(rw) {
            continue;
        }
        // simandoux_modified_slb carries a 1/(1-VSH) term that is singular at VSH=1; treat pure
        // shale as all water (same convention as the low-PHIE branch above). The shared solver
        // applies the same rule; keeping this explicit preserves the module's PHIE volume output.
        if modified_slb && vs >= 1.0 {
            // SB-SAT-030: returning a plausible number from a singular equation is the
            // fail-silent pattern. The value is still all-water — the chapter permits that —
            // but it MUST NOT leave the run unflagged, because on the log an answer clamped
            // out of a 0/0 is indistinguishable from one the equation actually produced.
            record_degradation_once(
                RunDegradationKind::Clamped,
                "simandoux_modified_slb: VSH >= 1 makes the 1/(1-VSH) term singular; \
                 saturation clamped to all-water rather than computed",
            );
            swe_sim[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = pe as f32;
            continue;
        }
        let a = ctx.p("A", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        let c = ctx.p("C", i);
        let rt_sh = ctx.p("RT_SH", i);
        let swe_irr = ctx.p("SWE_IRR", i);

        let sat = if modified_slb {
            crate::multimin2::sw_simandoux_modified_slb(r, pe, vs, rw, rt_sh, m, n_exp, a, c)
        } else {
            crate::multimin2::sw_simandoux_bardon_pied(r, pe, vs, rw, rt_sh, m, n_exp, a)
        };
        if is_missing(sat) {
            continue;
        }
        swe_sim[i] = sat as f32;
        let swe_l = limit(sat, swe_irr, 1.0);
        swe_out[i] = swe_l as f32;
        vol_uwat[i] = (pe * swe_l) as f32;
    }

    let flag_model = if modified_slb {
        crate::multimin2::SwModel::SimandouxModifiedSlb
    } else {
        crate::multimin2::SwModel::SimandouxBardonPied
    };
    let method_flag = swe_sim
        .iter()
        .map(|sw| if sw.is_finite() { flag_model.flag_code() } else { f32::NAN })
        .collect();
    HashMap::from([
        ("SWE_SIM".to_string(), swe_sim),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
        ("SW_METHOD".to_string(), method_flag),
    ])
}

// ---------------------------------------------------------------------------
// PERM_WYLLIE_ROSE — Permeability, Wyllie-Rose family (Loglan perm_wyllie_rose.lls)
// ---------------------------------------------------------------------------

fn perm_wyllie_rose_spec() -> ModuleSpec {
    ModuleSpec {
        name: "perm_wyllie_rose".into(),
        title: "Permeability — Wyllie-Rose".into(),
        category: "Permeability".into(),
        doc: "PERM = (C * PHIE^D / SWE_IRR^E)^2, mD. Defaults per method: \
              TIMUR C=100 D=2.25 E=1; MORRIS_BIGGS_OIL C=250 D=3 E=1; MORRIS_BIGGS_GAS C=79 D=3 E=1; \
              TIXIER C=250 D=3 E=1."
            .into(),
        args: vec![
            opt("OPT_WR", "Wyllie-Rose variant", "TIMUR", &["TIMUR", "MORRIS_BIGGS_OIL", "MORRIS_BIGGS_GAS", "TIXIER"]),
            param_open("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.01, 0.8, true),
            log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
            log_out("PERM_WR", "Permeability from Wyllie-Rose", "mD"),
            log_out("PERM", "Working permeability", "mD"),
        ],
    }
}

fn perm_wyllie_rose(ctx: &ModuleContext) -> ModuleOutputs {
    let phie = ctx.log("PHIE");
    let (c, d, e) = match ctx.o("OPT_WR") {
        "MORRIS_BIGGS_OIL" => (250.0, 3.0, 1.0),
        "MORRIS_BIGGS_GAS" => (79.0, 3.0, 1.0),
        "TIXIER" => (250.0, 3.0, 1.0),
        _ => (100.0, 2.25, 1.0), // TIMUR
    };
    let mut perm = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let pe = phie[i] as f64;
        let swirr = ctx.p("SWE_IRR", i);
        // Guard negative PHIE explicitly: pe.powf(d) is NaN for the fractional TIMUR exponent
        // (d=2.25) but a finite, plausible-looking value for the integer MORRIS_BIGGS/TIXIER
        // exponent (d=3.0), so without this the four OPT_WR variants disagree on the same
        // non-physical input. Skip it uniformly.
        if is_missing(pe) || pe < 0.0 || is_missing(swirr) || swirr <= 0.0 {
            continue;
        }
        let k = (c * pe.powf(d) / swirr.powf(e)).powi(2);
        perm[i] = k as f32;
    }
    HashMap::from([("PERM_WR".to_string(), perm.clone()), ("PERM".to_string(), perm)])
}

// ---------------------------------------------------------------------------
// PERM_COATES — Permeability, Coates FFI (Loglan perm_coates.lls)
// ---------------------------------------------------------------------------

fn perm_coates_spec() -> ModuleSpec {
    ModuleSpec {
        name: "perm_coates".into(),
        title: "Permeability — Coates".into(),
        category: "Permeability".into(),
        doc: "PERM = (C * PHIE^2 * (1 - SWE_IRR)/SWE_IRR)^2, mD.".into(),
        args: vec![
            param_open("CONST_COATES", "Coates constant", "", 1.0, 1000.0, true),
            param_open(
                "SWE_IRR",
                "Irreducible effective water saturation",
                "v/v",
                0.01,
                0.8,
                true,
            ),
            log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
            log_out("PERM_COATES", "Permeability from Coates", "mD"),
            log_out("PERM", "Working permeability", "mD"),
        ],
    }
}

fn perm_coates(ctx: &ModuleContext) -> ModuleOutputs {
    let phie = ctx.log("PHIE");
    let mut perm = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let pe = phie[i] as f64;
        let c = ctx.p("CONST_COATES", i);
        let swirr = ctx.p("SWE_IRR", i);
        if is_missing(pe) || is_missing(swirr) || swirr <= 0.0 {
            continue;
        }
        let k = c * pe * pe * (1.0 - swirr) / swirr;
        perm[i] = (k * k) as f32;
    }
    HashMap::from([("PERM_COATES".to_string(), perm.clone()), ("PERM".to_string(), perm)])
}

// ---------------------------------------------------------------------------
// PERM_TRANSFORM — Por-perm regression transform (core-calibrated)
// ---------------------------------------------------------------------------

fn perm_transform_spec() -> ModuleSpec {
    ModuleSpec {
        name: "perm_transform".into(),
        title: "Permeability — Por-Perm Transform".into(),
        category: "Permeability".into(),
        doc: "log10(PERM) = PT_A * PHIE + PT_B — the classic core-derived porosity-permeability \
              regression. Calibrate PT_A/PT_B per zone from RCAL data."
            .into(),
        args: vec![
            param_open("PT_A", "Slope", "", 1.0, 100.0, true),
            param_open("PT_B", "Intercept", "", -10.0, 5.0, true),
            log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
            log_out("PERM_XFM", "Permeability from transform", "mD"),
            log_out("PERM", "Working permeability", "mD"),
        ],
    }
}

fn perm_transform(ctx: &ModuleContext) -> ModuleOutputs {
    let phie = ctx.log("PHIE");
    let mut perm = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let pe = phie[i] as f64;
        let a = ctx.p("PT_A", i);
        let b = ctx.p("PT_B", i);
        if is_missing(pe) {
            continue;
        }
        // PT_A up to 100 and PT_B up to 5 are inside the dialog-validated ranges, so a*pe+b can
        // exceed ~38.5 and 10^x overflows the f32 cast to +Infinity (which is_missing, NaN-only,
        // treats as a real value that then flows into pay-flag cutoffs). Emit MISSING instead.
        let k = 10.0_f64.powf(a * pe + b) as f32;
        perm[i] = if k.is_finite() { k } else { f32::NAN };
    }
    HashMap::from([("PERM_XFM".to_string(), perm.clone()), ("PERM".to_string(), perm)])
}

// ---------------------------------------------------------------------------
// THIN_BED_TS — Thomas-Stieber laminated sand-shale decomposition
// (thin-beds model; Thomas & Stieber, 1975)
// ---------------------------------------------------------------------------

fn thin_bed_ts_spec() -> ModuleSpec {
    ModuleSpec {
        name: "thin_bed_ts".into(),
        title: "Thin Beds — Thomas-Stieber".into(),
        category: "ThinBeds".into(),
        doc: "Decomposes bulk VSH into laminar and dispersed shale by comparing the \
              measured (VSH, PHIT) point against the pure-laminated line \
              PHIT = PHI_SD_MAX*(1-VSH) + PHI_SH*VSH and the pure-dispersed line \
              PHIT = PHI_SD_MAX - VSH*(1-PHI_SH). VLAM reduces net sand (VSAND = 1-VLAM); \
              VDISP stays within the sand fraction. PHIE_LAM is the laminar-shale-corrected \
              porosity of the net sand. Structural shale is not modeled."
            .into(),
        args: vec![
            param_open(
                "PHI_SD_MAX",
                "Clean sand porosity (endpoint)",
                "v/v",
                0.05,
                0.45,
                true,
            ),
            param_open(
                "PHI_SH",
                "Shale porosity (endpoint)",
                "v/v",
                0.0,
                0.45,
                true,
            ),
            log_in("PHIT", "Total porosity log", "v/v", "PHIT", true),
            log_in("VSH", "Total (bulk) volume of shale log", "v/v", "VSH", true),
            log_out("VLAM", "Laminar shale volume fraction", "v/v"),
            log_out("VDISP", "Dispersed shale volume fraction", "v/v"),
            log_out("VSAND", "Net sand (non-laminar) fraction", "v/v"),
            log_out("PHIE_LAM", "Laminar-shale-corrected sand porosity", "v/v"),
        ],
    }
}

fn thin_bed_ts(ctx: &ModuleContext) -> ModuleOutputs {
    let phit = ctx.log("PHIT");
    let vsh = ctx.log("VSH");
    let mut vlam_out = vec![f32::NAN; ctx.n];
    let mut vdisp_out = vec![f32::NAN; ctx.n];
    let mut vsand_out = vec![f32::NAN; ctx.n];
    let mut phie_lam_out = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let pt = phit[i] as f64;
        let vs = vsh[i] as f64;
        let phi_sd = ctx.p("PHI_SD_MAX", i);
        let phi_sh = ctx.p("PHI_SH", i);
        if is_missing(pt) || is_missing(vs) || is_missing(phi_sd) || is_missing(phi_sh) {
            continue;
        }
        let vs_c = limit(vs, 0.0, 1.0);
        let lam_line = phi_sd * (1.0 - vs_c) + phi_sh * vs_c;
        let disp_line = phi_sd - vs_c * (1.0 - phi_sh);
        let denom = lam_line - disp_line;
        let f_disp = if denom.abs() > 1e-9 { limit((lam_line - pt) / denom, 0.0, 1.0) } else { 0.0 };

        let vlam = vs_c * (1.0 - f_disp);
        let vdisp = vs_c * f_disp;
        let vsand = 1.0 - vlam;
        vlam_out[i] = vlam as f32;
        vdisp_out[i] = vdisp as f32;
        vsand_out[i] = vsand as f32;
        phie_lam_out[i] =
            if vsand > 1e-6 { limit((pt - vlam * phi_sh) / vsand, 0.0, phi_sd) as f32 } else { f32::NAN };
    }

    HashMap::from([
        ("VLAM".to_string(), vlam_out),
        ("VDISP".to_string(), vdisp_out),
        ("VSAND".to_string(), vsand_out),
        ("PHIE_LAM".to_string(), phie_lam_out),
    ])
}

// ---------------------------------------------------------------------------
// DEPTH_SHIFT — block depth shift of one curve (log splice/shift)
// ---------------------------------------------------------------------------

fn depth_shift_spec() -> ModuleSpec {
    ModuleSpec {
        name: "depth_shift".into(),
        title: "Depth Shift".into(),
        category: "Prep".into(),
        doc: "Shifts CURVE by SHIFT metres (+ = the feature moves DEEPER) and resamples it \
              back onto the well's depth grid by linear interpolation. SHIFT is zone-\
              overridable, so different intervals can take different block shifts. The \
              result is written as <CURVE>_DS; the input curve is never modified."
            .into(),
        args: vec![
            param_open(
                "SHIFT",
                "Depth shift (+ = deeper)",
                "m",
                -1000.0,
                1000.0,
                true,
            ),
            log_in("CURVE", "Curve to shift", "", "GR", true),
            log_out_as("CURVE_DS", "{CURVE}_DS", "Depth-shifted copy", ""),
        ],
    }
}

/// Linear interpolation of `vals` (sampled at ascending `depths`) at `target`.
/// NaN outside the depth range, at NaN neighbours (no interpolation across gaps),
/// or when the frame is empty.
fn interp_at(depths: &[f32], vals: &[f32], target: f64) -> f64 {
    let n = depths.len();
    if n == 0 || target < depths[0] as f64 || target > depths[n - 1] as f64 {
        return MISSING;
    }
    // Binary search for the first sample >= target.
    let mut lo = 0usize;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if (depths[mid] as f64) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (d0, d1) = (depths[lo] as f64, depths[hi] as f64);
    let (v0, v1) = (vals[lo] as f64, vals[hi] as f64);
    if (target - d0).abs() < 1e-9 {
        return v0;
    }
    if (target - d1).abs() < 1e-9 {
        return v1;
    }
    if is_missing(v0) || is_missing(v1) || d1 <= d0 {
        return MISSING;
    }
    v0 + (v1 - v0) * (target - d0) / (d1 - d0)
}

fn depth_shift(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let vals = ctx.log("CURVE");

    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = depth[i] as f64;
        let shift = convert_depth(ctx.p("SHIFT", i), DepthUnit::Metres, ctx.depth_unit);
        if is_missing(d) || is_missing(shift) {
            continue;
        }
        out[i] = interp_at(&depth, &vals, d - shift) as f32;
    }
    HashMap::from([("CURVE_DS".to_string(), out)])
}

// ---------------------------------------------------------------------------
// SPLICE — combine two curves at a splice depth (run-to-run splicing)
// ---------------------------------------------------------------------------

fn splice_spec() -> ModuleSpec {
    ModuleSpec {
        name: "splice".into(),
        title: "Splice Curves".into(),
        category: "Prep".into(),
        doc: "SPLICED = TOP_CURVE above SPLICE_DEPTH, BOT_CURVE at and below it — the \
              classic run-to-run splice. Written as <TOP_CURVE>_SPL; inputs are never \
              modified."
            .into(),
        args: vec![
            param_open(
                "SPLICE_DEPTH",
                "Depth where BOT_CURVE takes over",
                "m",
                0.0,
                20000.0,
                true,
            ),
            log_in(
                "TOP_CURVE",
                "Curve used above the splice depth",
                "",
                "GR",
                true,
            ),
            log_in(
                "BOT_CURVE",
                "Curve used below the splice depth",
                "",
                "GR",
                true,
            ),
            log_out_as("SPLICED", "{TOP_CURVE}_SPL", "Spliced curve", ""),
        ],
    }
}

fn splice(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let top = ctx.log("TOP_CURVE");
    let bot = ctx.log("BOT_CURVE");
    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = convert_depth(depth[i] as f64, ctx.depth_unit, DepthUnit::Metres);
        if is_missing(d) {
            continue;
        }
        out[i] = if d < ctx.p("SPLICE_DEPTH", i) { top[i] } else { bot[i] };
    }
    HashMap::from([("SPLICED".to_string(), out)])
}

// ---------------------------------------------------------------------------
// GR_NORMALIZE — two-point percentile gamma-ray normalization
// ---------------------------------------------------------------------------

const GR_NORMALIZATION_PERCENTILE_GUIDANCE: (&str, &str) = (
    "P3/P97 is a named SandiBumi house preset for selecting well percentiles. It selects positions in the distribution; it is not a gamma-ray endpoint value.",
    "docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5.1; method_workflow_standards.md",
);

const GR_NORMALIZATION_REFERENCE_GUIDANCE: (&str, &str) = (
    "Compute well percentiles over a common reference interval containing comparable rock. Derive one reference pair from the study distribution or an agreed reference, then use that same pair for every well in the study.",
    "docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; method_workflow_standards.md",
);

fn gr_normalize_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gr_normalize".into(),
        title: "GR Normalization (Two-Point Percentile)".into(),
        category: "Prep".into(),
        doc: "GRN = (GR − Plow_well)·(Phigh_ref − Plow_ref)/(Phigh_well − Plow_well) + Plow_ref. \
              The well percentiles are computed from this run's GR samples (mask the run to a \
              common reference interval so every well is measured over comparable rock); the \
              reference percentiles are parameters. \
              SET YOUR OWN FIELD REFERENCE PAIR — that is the entire point of the module. The \
              pair ships absent: a reference pair from one basin is the wrong reference in another. \
              Derive yours from the field's own multi-well GR distribution, or from a reference \
              well everyone agrees on, then use the SAME pair for every well in the study. QC \
              across wells with a GRN histogram overlay — the P3/P97 of every normalized well \
              should coincide."
            .into(),
        args: vec![
            with_guidance(
                param(
                    "P_LOW", "Low percentile", "%", 3.0, 0.0, 50.0,
                    "memory/method_workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3",
                ),
                &[GR_NORMALIZATION_PERCENTILE_GUIDANCE],
            ),
            with_guidance(
                param(
                    "P_HIGH", "High percentile", "%", 97.0, 50.0, 100.0,
                    "memory/method_workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3",
                ),
                &[GR_NORMALIZATION_PERCENTILE_GUIDANCE],
            ),
            with_guidance(
                param_open("GR_LOW_REF", "Reference GR at low percentile", "gapi", 0.0, 1000.0, true),
                &[GR_NORMALIZATION_REFERENCE_GUIDANCE],
            ),
            with_guidance(
                param_open("GR_HIGH_REF", "Reference GR at high percentile", "gapi", 0.0, 1000.0, true),
                &[GR_NORMALIZATION_REFERENCE_GUIDANCE],
            ),
            log_in("GR", "Gamma ray log", "gapi", "GR", true),
            log_out("GRN", "Normalized gamma ray", "gapi"),
        ],
    }
}

/// The GR preset of [`crate::condition::normalize`], kept so saved chains and stored runs still
/// resolve — it is NOT a second implementation.
///
/// Jauhar, 2026-08-05: *"dont dupilcates, normalize tools here should be universal for all
/// logs"*. A two-point percentile map has nothing to do with gamma rays; the same arithmetic
/// normalizes a neutron, a sonic or a density, and every one of them drifts between tools in the
/// same way. So the module became `normalize`, this delegates to it with the GR arg names, and
/// the pickers hide this one (`SUPERSEDED_MODULE_IDS`) so the user sees exactly one Normalize.
///
/// Left RUNNABLE rather than retired like `multimin`: retiring it would fail every saved chain
/// carrying a `gr_normalize` step, and unlike superseded physics the answer here is unchanged.
fn gr_normalize(ctx: &ModuleContext) -> ModuleOutputs {
    let mut opts = ctx.opts.clone();
    opts.insert("OPT_METHOD".into(), "TWO_POINT".into());
    opts.insert("OPT_SPACE".into(), "LINEAR".into());
    let mut params = ctx.params.clone();
    // The GR manifest's own arg names, mapped onto the universal ones.
    if let Some(v) = params.get("GR_LOW_REF").cloned() {
        params.insert("REF_LOW".into(), v);
    }
    if let Some(v) = params.get("GR_HIGH_REF").cloned() {
        params.insert("REF_HIGH".into(), v);
    }
    let mut logs = ctx.logs.clone();
    if let Some(v) = logs.get("GR").cloned() {
        logs.insert("CURVE".into(), v);
    }
    let inner = ModuleContext { n: ctx.n, logs, params, opts, depth_unit: ctx.depth_unit };
    // A refusal from the shared core (no reference pair, a run with nothing in it) leaves GRN
    // MISSING, which is what this module always did — it never returned a Result.
    let out = crate::condition::normalize(&inner)
        .map(|m| m["OUT_CURVE"].clone())
        .unwrap_or_else(|_| vec![f32::NAN; ctx.n]);
    HashMap::from([("GRN".to_string(), out)])
}

// ---------------------------------------------------------------------------
// LOG_PREDICT — synthetic log by K-nearest-neighbour regression (Facimage MRGC
// equivalent: synthetic RHOB from GRN + association, synthetic NPHI from
// RHOB + GRN, synthetic DT/U for multimin coverage)
// ---------------------------------------------------------------------------

fn log_predict_spec() -> ModuleSpec {
    ModuleSpec {
        name: "log_predict".into(),
        title: "Synthetic Log (KNN Predict)".into(),
        category: "Prep".into(),
        doc: "Facimage-style synthetic log: trains on the samples of THIS run where TARGET and \
              every supplied predictor are present, then predicts TARGET everywhere the \
              predictors exist by distance-weighted K-nearest-neighbour regression (predictors \
              z-scored; training set decimated to ≤4000 points). OPT_COMBINE: SYNTHETIC writes \
              the pure prediction; FILL_MISSING keeps the raw value where present; MAX_RAW takes \
              max(raw, synthetic) — the washout rule for RHOB, since bad hole only pushes RHOB \
              down. Output is named <TARGET>_SYN. Mask the run to good-hole intervals so bad \
              samples never train the model."
            .into(),
        args: vec![
            opt("OPT_COMBINE", "How to combine with the raw curve", "SYNTHETIC", &["SYNTHETIC", "FILL_MISSING", "MAX_RAW"]),
            param(
                "K", "Number of neighbours", "", 10.0, 1.0, 50.0,
                "Geolog V14 facimage_05_using_hc.5.05.html Nearest Neighbors Default 10; docs/PRD_v2/24_ml-advanced.md §5",
            ),
            log_in("TARGET", "Curve to predict", "", "RHOB", true),
            log_in("P1", "Predictor 1", "", "GR", true),
            log_in("P2", "Predictor 2 (optional)", "", "NPHI", false),
            log_in("P3", "Predictor 3 (optional)", "", "DT", false),
            log_out_as("SYN", "{TARGET}_SYN", "Synthetic curve", ""),
        ],
    }
}

fn log_predict(ctx: &ModuleContext) -> ModuleOutputs {
    let target = ctx.log("TARGET");
    let combine = ctx.o("OPT_COMBINE").to_string();
    let out_name = "SYN".to_string();
    let mut out = vec![f32::NAN; ctx.n];

    // Use every supplied predictor that carries data.
    let preds: Vec<Vec<f32>> = ["P1", "P2", "P3"]
        .iter()
        .map(|p| ctx.log(p))
        .filter(|v| v.iter().any(|x| !x.is_nan()))
        .collect();
    if preds.is_empty() {
        return HashMap::from([(out_name, out)]);
    }
    let dims = preds.len();

    // Training set: target + all predictors present. The sample index is kept so a
    // sample never predicts from itself (leave-one-out) — otherwise every training
    // sample self-matches at distance 0 and the synthetic just echoes the raw curve,
    // defeating the MAX_RAW washout rule.
    let mut train: Vec<(usize, Vec<f64>, f64)> = Vec::new();
    for i in 0..ctx.n {
        let t = target[i] as f64;
        if is_missing(t) {
            continue;
        }
        let x: Vec<f64> = preds.iter().map(|p| p[i] as f64).collect();
        if x.iter().any(|v| is_missing(*v)) {
            continue;
        }
        train.push((i, x, t));
    }
    let k = (ctx.p("K", 0).max(1.0) as usize).min(train.len());
    if train.len() < 10 {
        return HashMap::from([(out_name, out)]);
    }
    // Decimate a huge training set (keeps the scan O(n·4000)).
    if train.len() > 4000 {
        let stride = train.len() as f64 / 4000.0;
        train = (0..4000)
            .map(|j| train[(j as f64 * stride) as usize].clone())
            .collect();
    }

    // Z-score the predictor space from the training set.
    let mut mean = vec![0.0; dims];
    let mut std = vec![0.0; dims];
    for (_, x, _) in &train {
        for d in 0..dims {
            mean[d] += x[d];
        }
    }
    for m in &mut mean {
        *m /= train.len() as f64;
    }
    for (_, x, _) in &train {
        for d in 0..dims {
            std[d] += (x[d] - mean[d]).powi(2);
        }
    }
    for s in &mut std {
        *s = (*s / train.len() as f64).sqrt();
        // Negated comparison so a NaN std is caught too: `NaN < 1e-9` is false, so the old form
        // let a NaN through and every scaled distance below became NaN.
        if !(*s >= 1e-9) {
            *s = 1.0;
        }
    }
    let scaled: Vec<(usize, Vec<f64>, f64)> = train
        .iter()
        .map(|(i, x, t)| (*i, (0..dims).map(|d| (x[d] - mean[d]) / std[d]).collect(), *t))
        .collect();

    for i in 0..ctx.n {
        let x: Vec<f64> = preds.iter().map(|p| p[i] as f64).collect();
        if x.iter().any(|v| is_missing(*v)) {
            continue;
        }
        let xs: Vec<f64> = (0..dims).map(|d| (x[d] - mean[d]) / std[d]).collect();

        // Keep the K nearest by insertion into a tiny sorted buffer.
        let mut best: Vec<(f64, f64)> = Vec::with_capacity(k + 1); // (dist², value)
        for (ti, tx, tv) in &scaled {
            if *ti == i {
                // Leave-one-out. Without this every training sample self-matches at distance 0,
                // so at k = 1 the synthetic reproduces the raw curve EXACTLY and every predictor
                // set scores perfectly — the trap SB-MLA-050 names, pinned by
                // `a_k1_neighbour_search_that_reproduces_its_training_data_exactly_is_a_failure`,
                // which fails with 60 of 60 samples exact if this line is removed.
                continue;
            }
            let d2: f64 = (0..dims).map(|d| (xs[d] - tx[d]).powi(2)).sum();
            if !d2.is_finite() {
                continue; // a non-finite distance cannot rank; skip rather than sort on it
            }
            if best.len() < k {
                best.push((d2, *tv));
                best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            } else if d2 < best[k - 1].0 {
                best[k - 1] = (d2, *tv);
                best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            }
        }
        let mut wsum = 0.0;
        let mut vsum = 0.0;
        for (d2, v) in &best {
            let w = 1.0 / (d2.sqrt() + 1e-6);
            wsum += w;
            vsum += w * v;
        }
        let syn = vsum / wsum;

        let raw = target[i] as f64;
        out[i] = match combine.as_str() {
            "FILL_MISSING" if !is_missing(raw) => raw as f32,
            "MAX_RAW" if !is_missing(raw) => raw.max(syn) as f32,
            _ => syn as f32,
        };
    }

    HashMap::from([(out_name, out)])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-011. Matrix density must be a single shared
    /// parameter across modules that a documented workflow chains. `gascorr`'s own doc instructs
    /// chaining it with the porosity modules, so the four consumers below are one chain.
    ///
    /// The value is Jauhar's, recorded 2026-08-16: **2.65**, the section 5.1 three-way agreement
    /// across IP MINDEF, Techlog `QM_MineralTable` and SandiMin (tier T3). Section 5.1 also cites
    /// Geolog `phi_den.info`'s shipped 2645 k/m3 (tier T1) and adjudicates neither, so one shared
    /// parameter could not exist until an owner chose. Geolog's position stays visible as evidence
    /// through the shared source topic rather than being deleted.
    #[test]
    fn every_chained_module_reads_one_shared_matrix_density_and_still_discloses_the_position_it_did_not_take(
    ) {
        const CHAINED: &[&str] = &["phi_den", "phi_dn", "condflag", "gascorr"];
        const OWNER_SELECTED: f64 = 2.65;
        const GEOLOG_SHIPPED: f64 = 2.645;

        let modules = module_catalog();
        let mut seen = Vec::new();
        for name in CHAINED {
            let spec = modules
                .iter()
                .find(|spec| spec.name == *name)
                .unwrap_or_else(|| panic!("{name} is not in the shipping catalog"));
            let arg = spec
                .args
                .iter()
                .find(|arg| arg.name == "RHO_MA")
                .unwrap_or_else(|| panic!("{name} declares no RHO_MA"));
            let value: f64 = arg
                .default
                .parse()
                .unwrap_or_else(|_| panic!("{name}.RHO_MA default '{}' is not numeric", arg.default));
            seen.push((*name, value, arg.min, arg.max, arg.sources_topic.clone()));
        }

        // A — one shared parameter means one default. Before this row, `gascorr` shipped 2.65 while
        // the three porosity modules shipped 2.645, and the docs told the user to chain them.
        for (name, value, ..) in &seen {
            assert_eq!(
                *value, OWNER_SELECTED,
                "'{name}' must read the one shared matrix density; a chained module on its own value is the defect"
            );
            assert_ne!(
                *value, GEOLOG_SHIPPED,
                "'{name}' still carries the unchosen Geolog default"
            );
        }
        let ranges: Vec<_> = seen.iter().map(|(_, _, lo, hi, _)| (*lo, *hi)).collect();
        assert!(
            ranges.windows(2).all(|w| w[0] == w[1]),
            "a shared parameter cannot have different validity ranges per module: {ranges:?}"
        );

        // B — the other side, and the one that keeps this honest. Choosing a value must not erase
        // the cited position it was chosen over: every consumer still discloses both, so the
        // interpreter can see that 2.645 exists and who ships it.
        for (name, _, _, _, topic) in &seen {
            let positions = crate::param_sources::sources_for(topic);
            assert!(
                !positions.is_empty(),
                "'{name}' must still disclose the competing matrix-density positions"
            );
            assert!(
                positions
                    .iter()
                    .any(|p| p.value.parse::<f64>() == Ok(GEOLOG_SHIPPED)),
                "the unchosen Geolog 2.645 must remain visible as evidence beside the field, not be deleted"
            );
            assert!(
                positions
                    .iter()
                    .any(|p| p.value.parse::<f64>() == Ok(OWNER_SELECTED)),
                "the selected value must itself be a cited position, not an interpreter invention"
            );
        }

        // C — no other porosity module may quietly reintroduce a second matrix density.
        for spec in modules.iter().filter(|spec| spec.category == "Porosity") {
            if let Some(arg) = spec.args.iter().find(|arg| arg.name == "RHO_MA") {
                if let Ok(value) = arg.default.parse::<f64>() {
                    assert_eq!(
                        value, OWNER_SELECTED,
                        "porosity module '{}' introduces a second matrix density",
                        spec.name
                    );
                }
            }
        }
    }

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-023 and F14. The requirement is explicit
    /// that the arithmetic average and the RMS **MUST NOT** be presented as crossplot porosity
    /// methods, and that the doc string claiming they are "the standard analytic equivalent" of
    /// chart lookups **MUST** be removed. They **MAY** ship as explicitly labelled quick-look
    /// comparison curves, which is what this pins.
    ///
    /// This covers the presentation arm only. The pay-eligibility arm is recorded as a live
    /// contract conflict rather than silently resolved here - see the SB-POR-023 evidence row.
    #[test]
    fn the_neutron_density_shortcuts_are_labelled_quick_look_comparisons_and_never_claim_to_be_a_crossplot_method(
    ) {
        let dn = module_catalog()
            .into_iter()
            .find(|spec| spec.name == "phi_dn")
            .expect("phi_dn is a shipping module");

        // A — the exact claim the requirement orders removed must be gone from every porosity doc,
        // not merely from the one line the chapter cites.
        for spec in module_catalog().iter().filter(|spec| spec.category == "Porosity") {
            let doc = spec.doc.to_lowercase();
            assert!(
                !doc.contains("standard analytic equivalent"),
                "'{}' still claims to be the standard analytic equivalent of a chart lookup",
                spec.name
            );
            assert!(
                !doc.contains("chart lookup") || spec.name == "phi_dn",
                "'{}' should not describe itself against chart lookups",
                spec.name
            );
        }

        // B — the other side. Removing the claim is not enough; the requirement permits these
        // curves only as EXPLICITLY labelled quick-look comparisons, so the label must be present.
        let doc = dn.doc.to_lowercase();
        assert!(
            doc.contains("quick-look comparison only"),
            "the D-N shortcuts must be explicitly labelled quick-look comparisons: {}",
            dn.doc
        );
        assert!(
            doc.contains("not a crossplot") || doc.contains("neither combination is a crossplot"),
            "the doc must say plainly that these are not crossplot porosity methods: {}",
            dn.doc
        );

        // C — pay eligibility, settled by Jauhar on 2026-08-16 as option (b): the D-N limited output
        // IS admitted to pay, so the canonical-first fallback reaching it is the approved contract
        // rather than the leak it looked like. The pay path must therefore consult exactly the
        // closed two-name pair and nothing wider — a family scan here would let any porosity-shaped
        // curve into a reserves number, which is the failure the closed list exists to prevent.
        //
        // `PILOT_SCOPE.md` item 6 still reads "excluded from pay by default" and contradicts this
        // ruling. That file is outside this program's allowed paths and is hash-bound to DEC-018,
        // so the correction is recorded in the SB-POR-023 evidence row rather than made here.
        let pay = include_str!("workflow.rs");
        let candidates = pay
            .split("let phie_candidates = vec![")
            .nth(1)
            .and_then(|tail| tail.split("];").next())
            .expect("run_pay_summary must declare its PHIE candidate list");
        assert!(
            candidates.contains("\"PHIE\"") && candidates.contains("PHIE_DN_LIMITED_DEFAULT"),
            "pay must reach the D-N limited output through the canonical-first pair: {candidates}"
        );
        assert_eq!(
            candidates.matches(',').count(),
            2,
            "the pay fallback must stay a closed two-name pair, not a widening list: {candidates}"
        );

        // D — the registry identity from SB-POR-001 must still type this producer as a comparison,
        // so the label and the machine-readable role cannot drift apart.
        let roles: Vec<_> = dn
            .args
            .iter()
            .filter_map(|arg| arg.porosity_output.as_ref())
            .map(|contract| contract.method.clone())
            .collect();
        assert!(
            !roles.is_empty() && roles.iter().all(|m| m.contains("COMPARISON")),
            "phi_dn outputs must remain typed as a comparison producer: {roles:?}"
        );
    }

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-009 and F21. `PHIT >= PHIE` must hold at
    /// every sample by construction, and the requirement's own words are that the invariant "MUST
    /// additionally be asserted, not merely relied on".
    ///
    /// The adversarial witness is not invented: `DT_MA = 70` and `DT_SH = 60` are both inside the
    /// shipped `phi_son` declared ranges (`DT_MA` 40..70, `DT_SH` 60..150). There the shale term
    /// `(DT_SH - DT_MA)` goes NEGATIVE, so the effective porosity is built by ADDING to the total —
    /// which is how a sonic sample can report more effective than total porosity while every input
    /// is nominally in range. Arm B proves the case really is adversarial, so this cannot pass by
    /// picking inputs that never stress the ordering.
    ///
    /// `ssc`/`sspw` are proved structurally rather than executed: both bound effective porosity with
    /// total porosity as the upper limit, so the invariant holds by construction there. Their file
    /// is protected, and the 2026-08-16 authorization covered SB-POR-008 only.
    #[test]
    fn every_porosity_method_keeps_total_porosity_at_or_above_effective_porosity_at_every_sample() {
        use crate::units::DepthUnit;

        let n = 3usize;
        let logs: HashMap<String, Vec<f32>> = HashMap::from([
            ("DT".into(), vec![80.0, 95.0, 120.0]),
            ("RHOB".into(), vec![2.30, 2.45, 2.10]),
            ("NPHI".into(), vec![0.25, 0.35, 0.15]),
            ("VSH".into(), vec![0.20, 0.60, 0.90]),
        ]);
        // Section 5.1 / 5.2 cited values, except the sonic pair, which is deliberately the
        // in-range-but-inverted case described above.
        let base: &[(&str, f64)] = &[
            ("RHO_MA", 2.65),
            ("RHO_SH", 2.50),
            ("RHO_FL", 1.00),
            ("RHO_DSH", 2.78),
            ("RHO_W", 1.00),
            ("NPHI_SH", 0.40),
            ("PHIE_MAX", 0.30),
            ("VSH_SHALE", 0.95),
            ("DT_FL", 189.0),
            ("DT_MA", 70.0),
            ("DT_SH", 60.0),
        ];
        let params: HashMap<String, Vec<f64>> =
            base.iter().map(|(k, v)| ((*k).to_string(), vec![*v; n])).collect();

        // B — the witness must actually be able to break the ordering, or arm A proves nothing.
        let (dt, vsh) = (120.0_f64, 0.90_f64);
        let raw_total = (dt - 70.0) / (189.0 - 70.0);
        let raw_effective = raw_total - vsh * (60.0 - 70.0) / (189.0 - 70.0);
        assert!(
            raw_effective > raw_total,
            "the chosen in-range sonic parameters must make the unguarded effective porosity exceed the total"
        );

        // A — execute every porosity module that can be driven from these inputs and pair its
        // outputs by declared name, so a limited pair and an unlimited pair are both checked.
        let mut checked = 0usize;
        for module in ["phi_den", "phi_dn", "phi_son"] {
            for opts in [
                HashMap::new(),
                HashMap::from([("OPT_SON".to_string(), "RHG".to_string())]),
                HashMap::from([("OPT_CP".to_string(), "ON".to_string())]),
                HashMap::from([("OPT_PHIEMAX".to_string(), "MAXIMUM".to_string())]),
            ] {
                let ctx = ModuleContext {
                    n,
                    logs: logs.clone(),
                    params: params.clone(),
                    opts,
                    depth_unit: DepthUnit::Metres,
                };
                let outputs = run_module(module, &ctx)
                    .unwrap_or_else(|error| panic!("{module} failed to run: {error}"));
                for (name, effective) in &outputs {
                    let Some(suffix) = name.strip_prefix("PHIE") else { continue };
                    let Some(total) = outputs.get(&format!("PHIT{suffix}")) else { continue };
                    for i in 0..n {
                        let (e, t) = (effective[i], total[i]);
                        if !e.is_finite() || !t.is_finite() {
                            continue;
                        }
                        assert!(
                            t >= e,
                            "{module}: PHIT{suffix} {t} is below PHIE{suffix} {e} at sample {i}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(
            checked >= 30,
            "the sweep must actually compare finite pairs, not silently skip them; compared {checked}"
        );

        // C — the protected methods hold the invariant by construction: effective porosity is
        // bounded above by total porosity at the point it is written.
        let ssc = include_str!("ssc.rs");
        let bounded = ssc
            .lines()
            .filter(|line| line.contains("let phie = limit(") && line.contains(", phit)"))
            .count();
        assert_eq!(
            bounded, 2,
            "ssc and sspw must each bound effective porosity by total porosity; found {bounded}"
        );

        // D — the other side: no registered porosity method may emit a PHIT/PHIE pair that neither
        // arm covers. A new method would otherwise inherit the guarantee without ever being checked.
        let covered = ["phi_den", "phi_dn", "phi_son", "ssc", "sspw"];
        for spec in module_catalog().iter().filter(|spec| spec.category == "Porosity") {
            let emits_pair = spec
                .args
                .iter()
                .any(|arg| arg.kind == ArgKind::LogOut && arg.name.starts_with("PHIE"))
                && spec
                    .args
                    .iter()
                    .any(|arg| arg.kind == ArgKind::LogOut && arg.name.starts_with("PHIT"));
            assert!(
                !emits_pair || covered.contains(&spec.name.as_str()),
                "porosity method '{}' emits a PHIT/PHIE pair that this proof does not cover",
                spec.name
            );
        }
    }

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-008 and F16. The required form
    /// `PHIT_SH = (RHO_DSH - RHO_SH)/(RHO_DSH - RHO_W)` with `RHO_W` the FORMATION WATER density is
    /// the requirement's own words; the divergence magnitude comes from section 5.1, where fluid
    /// density ships fresh `1.00` and salt `1.10` while formation-water density ships `1.00`. That
    /// is why the wrong anchor is invisible at defaults and only appears on saline formation water.
    ///
    /// The witness values below are chosen to separate the two anchors, not read back from the
    /// code: with `RHO_DSH = 2.78` (section 5.1 IP `Rho Dry Clay`), `RHO_SH = 2.50` (section 5.1
    /// Techlog script `DEN_shale`) and `RHO_W = 1.10`, the required form gives `0.28/1.68`, while
    /// the retired fluid-anchored form at `RHO_FL = 1.00` gives `0.28/1.78`. They differ by about
    /// 3.7 percent of the answer, in the direction that overstates shale porosity.
    ///
    /// Owner authorization, 2026-08-16: Jauhar authorized the narrow `ssc.rs` edit this pins.
    /// Investigation then narrowed it further — `ssc`'s own `(rhob_dsi - rhob_wsi)/(rhob_dsi -
    /// rhob_fl)` is a fractional distance along the fluid-anchored projection line `m3` that
    /// defines `rhob_dsi`, so it is a silt geometry term rather than this quantity. Its arithmetic
    /// is deliberately unchanged and only its colliding local name was retired, which is what F16
    /// actually requires.
    #[test]
    fn one_formation_water_clay_bound_water_porosity_serves_every_porosity_method_and_the_silt_and_shale_subtraction_terms_keep_their_own_identities(
    ) {
        // A — the single definition, evaluated from the chapter's own form rather than by calling
        // the code twice. RHO_W is the anchor; RHO_FL does not appear.
        let (rho_dsh, rho_sh, rho_w, rho_fl) = (2.78_f64, 2.50_f64, 1.10_f64, 1.00_f64);
        let required = (rho_dsh - rho_sh) / (rho_dsh - rho_w);
        assert!(
            (shale_total_porosity(rho_dsh, rho_sh, rho_w) - required).abs() < 1e-12,
            "the shared helper must be the chapter's form exactly"
        );

        // B — the two anchors must be distinguishable, or this contract is untestable and the
        // defect it targets would be invisible. This is the whole reason the row existed.
        let fluid_anchored = (rho_dsh - rho_sh) / (rho_dsh - rho_fl);
        assert!(
            (required - fluid_anchored).abs() > 0.005,
            "the witness must actually separate formation water from fluid density"
        );
        assert!(
            fluid_anchored < required,
            "the retired fluid anchor understates the denominator and so overstates shale porosity"
        );

        // C — every porosity method that carries this quantity declares the formation-water
        // parameter it is anchored on. `sspw` is included because SB-POR-008 is what put it there.
        let modules = module_catalog();
        for module in ["phi_den", "phi_dn", "sspw"] {
            let spec = modules
                .iter()
                .find(|spec| spec.name == module)
                .unwrap_or_else(|| panic!("{module} is not in the shipping catalog"));
            assert!(
                spec.args.iter().any(|arg| arg.name == "RHO_W"),
                "'{module}' carries clay-bound-water porosity and must declare RHO_W"
            );
        }

        // D — the other side, and the one a lazy implementation would trip. The quantity must be
        // defined exactly once in the whole tree. A module that re-derives the expression locally
        // would satisfy every assertion above and still be the defect SB-POR-008 exists to close.
        let sources = [
            ("modules.rs", include_str!("modules.rs")),
            ("ssc.rs", include_str!("ssc.rs")),
        ];
        let mut definitions = Vec::new();
        for (file, whole) in sources {
            // Production code only. A test may legitimately write the form out to compare against;
            // the contract is that no shipping module re-derives it.
            let text = whole.split("#[cfg(test)]").next().unwrap_or(whole);
            for (number, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                // The shale form divided by a dry-shale-minus-something denominator.
                let re_derived = (code.contains("rho_dsh -") || code.contains("rhob_dsh -"))
                    && code.contains('/')
                    && (code.contains("rho_sh") || code.contains("rhob_sh"));
                if re_derived {
                    definitions.push(format!("{file}:{}", number + 1));
                }
            }
        }
        assert_eq!(
            definitions.len(),
            1,
            "clay-bound-water porosity must be written exactly once; found {definitions:?}"
        );
        assert!(
            definitions[0].starts_with("modules.rs"),
            "the one definition must be the shared helper, not a module-local copy: {definitions:?}"
        );

        // E — F16's naming rule, from both sides. The shale-SUBTRACTION term is a different
        // quantity and the SSC silt fraction is a third; neither may wear this name.
        let ssc = include_str!("ssc.rs");
        assert!(
            ssc.contains("let silt_water_fraction ="),
            "the SSC silt geometry term must carry its own identity"
        );
        assert!(
            ssc.contains("(rhob_dsi - rhob_wsi) / (rhob_dsi - rhob_fl)"),
            "the SSC silt term's fluid-anchored arithmetic must remain untouched"
        );
    }

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-007. Every expected topic, value, source
    /// and tier below is transcribed from that chapter's section 5 parameter tables (5.1 densities,
    /// 5.2 sonic transit times, 5.3 limits) and its tier key at lines 7-19; no value is read back
    /// from the shipping manifests, so a manifest that merely describes itself cannot pass.
    ///
    /// The requirement is bounded by section 5 in both directions. A parameter section 5 carries a
    /// row for **must** expose its source and tier; a parameter section 5 carries **no** row for —
    /// `OPT_SON`, `OPT_XPLOT`, and `phimax`'s SandiBumi-own compaction trend — must stay untopiced
    /// rather than borrow a neighbouring quantity's evidence. Section 5.3's "IP roll-off triple"
    /// names `PHIMAX` but is a shale roll-off, not `phimax`'s compaction ceiling, and the two are
    /// deliberately not merged here.
    ///
    /// Residual, per the owner's SB-POR-007 scope decision: `ssc` and `sspw` declare their own
    /// parameters inside `src-tauri/src/ssc.rs`, which this program may not edit. Their parameters
    /// therefore remain unsourced and are named rather than silently counted as covered. Both are
    /// excluded from the first pilot, so nothing shipping in it is left without its citation.
    #[test]
    fn every_cited_porosity_parameter_carries_its_section_five_source_and_tier_while_an_absent_default_stays_absent(
    ) {
        use crate::param_sources as sources;

        let modules = module_catalog();
        let arg = |module: &str, argument: &str| -> ArgSpec {
            modules
                .iter()
                .find(|spec| spec.name == module)
                .unwrap_or_else(|| panic!("module '{module}' is not in the shipping catalog"))
                .args
                .iter()
                .find(|a| a.name == argument)
                .unwrap_or_else(|| panic!("'{module}.{argument}' is not a shipping argument"))
                .clone()
        };

        // A — every section 5 row a live POR manifest exposes reaches its parameter.
        let cited: &[(&str, &str, &str)] = &[
            ("phi_den", "RHO_MA", sources::MATRIX_DENSITY),
            ("phi_den", "RHO_SH", sources::SHALE_DENSITY),
            ("phi_den", "RHO_DSH", sources::DRY_SHALE_DENSITY),
            ("phi_den", "RHO_FL", sources::FLUID_DENSITY),
            ("phi_den", "RHO_W", sources::FORMATION_WATER_DENSITY),
            ("phi_den", "PHIE_MAX", sources::MAX_EFFECTIVE_POROSITY),
            ("phi_den", "OPT_PHIEMAX", sources::POROSITY_LIMIT_MODE),
            ("phi_dn", "RHO_MA", sources::MATRIX_DENSITY),
            ("phi_dn", "RHO_SH", sources::SHALE_DENSITY),
            ("phi_dn", "RHO_DSH", sources::DRY_SHALE_DENSITY),
            ("phi_dn", "NPHI_SH", sources::SHALE_NEUTRON_ENDPOINT),
            ("phi_dn", "RHO_FL", sources::FLUID_DENSITY),
            ("phi_dn", "RHO_W", sources::FORMATION_WATER_DENSITY),
            ("phi_dn", "PHIE_MAX", sources::MAX_EFFECTIVE_POROSITY),
            ("phi_dn", "OPT_PHIEMAX", sources::POROSITY_LIMIT_MODE),
            ("phi_son", "DT_MA", sources::MATRIX_TRANSIT_TIME),
            ("phi_son", "DT_FL", sources::FLUID_TRANSIT_TIME),
            ("phi_son", "DT_SH", sources::SHALE_TRANSIT_TIME),
            ("phi_son", "OPT_CP", sources::SONIC_COMPACTION_CORRECTION),
        ];
        for (module, argument, topic) in cited {
            assert_eq!(
                &arg(module, argument).sources_topic,
                topic,
                "'{module}.{argument}' must expose its section 5 evidence under '{topic}'"
            );
        }

        // B — a topic string is not evidence. Every one must resolve to real, completely
        // attributed positions, and every position must be tiered.
        //
        // The eight topics this requirement registers are held to the porosity chapter's own tier
        // key (lines 7-19). The four it *reuses* — matrix, shale and dry-shale density and the
        // shale neutron endpoint — were registered and pinned by earlier CLY/CORE increments that
        // render the same primary tier as `T1′`; re-spelling their evidence to `T1p` here would
        // silently rewrite another requirement's record and break its pinned inventory, so they are
        // held to complete attribution and a non-empty tier instead. The cross-chapter divergence
        // between `T1p`, `T1′` and the frontend fixture's `T1-prime` is recorded, not resolved
        // here: unifying a shared vocabulary is its own change, not a side effect of this one.
        const POR_OWNED_TIERS: &[&str] = &["T1p", "T1", "T2", "T3", "T3-eq", "T4"];
        let por_owned = [
            sources::FLUID_DENSITY,
            sources::FORMATION_WATER_DENSITY,
            sources::MAX_EFFECTIVE_POROSITY,
            sources::POROSITY_LIMIT_MODE,
            sources::MATRIX_TRANSIT_TIME,
            sources::FLUID_TRANSIT_TIME,
            sources::SHALE_TRANSIT_TIME,
            sources::SONIC_COMPACTION_CORRECTION,
        ];
        for (module, argument, topic) in cited {
            let positions = sources::sources_for(topic);
            assert!(
                !positions.is_empty(),
                "'{module}.{argument}' names topic '{topic}', which resolves to no evidence at all"
            );
            assert!(
                sources::parameter_label(topic).is_some(),
                "topic '{topic}' has no human label for the dialog to render"
            );
            for position in positions {
                assert!(
                    !position.product.is_empty()
                        && !position.value.is_empty()
                        && !position.source.is_empty(),
                    "a '{topic}' position is missing its product, value or source: {position:?}"
                );
                assert!(
                    !position.tier.is_empty(),
                    "a '{topic}' position carries no evidence tier: {position:?}"
                );
                if por_owned.contains(topic) {
                    assert!(
                        POR_OWNED_TIERS.contains(&position.tier),
                        "'{topic}' is registered by SB-POR-007 and carries tier '{}', which is not one of the porosity chapter's declared tiers",
                        position.tier
                    );
                }
            }
        }

        // C — the whole point of `with_sources`. Registering an attested vendor value must never
        // give a deliberately absent parameter a number to fall back on. Each of these ships
        // ABSENT per section 5, and each now also ships its competing evidence.
        for (module, argument) in [
            ("phi_den", "RHO_SH"),
            ("phi_den", "RHO_DSH"),
            ("phi_dn", "NPHI_SH"),
            ("phi_son", "DT_MA"),
            ("phi_son", "DT_SH"),
        ] {
            let spec = arg(module, argument);
            assert!(
                !spec.sources_topic.is_empty(),
                "'{module}.{argument}' is the case this rule exists for and must be sourced"
            );
            assert!(
                spec.default.is_empty() && spec.default_source == ABSENT_DEFAULT_SOURCE,
                "'{module}.{argument}' ships ABSENT; disclosing evidence must not create a default"
            );
            assert!(
                sources::sources_for(&spec.sources_topic)
                    .iter()
                    .any(|position| position.value.parse::<f64>().is_ok()),
                "'{module}.{argument}' must still disclose at least one attested number"
            );
        }

        // D — the other side. Section 5 registers no row for these, so nothing may be invented for
        // them here; their sources belong to their own requirements (`OPT_XPLOT` to SB-POR-023
        // under DEC-014, `OPT_SON` to the SB-POR-013..020 sonic group under DEC-017).
        for (module, argument) in [
            ("phi_dn", "OPT_XPLOT"),
            ("phi_son", "OPT_SON"),
            ("phimax", "MODE"),
            ("phimax", "PHIMAX0"),
        ] {
            assert!(
                arg(module, argument).sources_topic.is_empty(),
                "'{module}.{argument}' has no section 5 row; sourcing it would invent a citation"
            );
        }

        // E — the tier has to survive into the run record, not merely into the dialog. This is the
        // exact call the runner makes for every sourced numeric parameter.
        let phie_max = arg("phi_den", "PHIE_MAX");
        let decision = sources::decision_for(
            &phie_max.sources_topic,
            &serde_json::Value::from(0.30_f64),
        )
        .expect("a sourced numeric parameter must produce a run-record decision");
        assert_eq!(decision.parameter, "maximum effective porosity");
        assert!(
            decision
                .alternatives
                .iter()
                .any(|position| position.tier == "T1"
                    && position.value == "0.30"
                    && position.source.contains("phi_*.info PHIE_MAX")),
            "the persisted decision must carry the cited Geolog position with its tier: {:?}",
            decision.alternatives
        );
        assert!(
            decision
                .alternatives
                .iter()
                .any(|position| position.value == "0.35"),
            "the disagreeing Techlog ceiling must remain visible beside the chosen value"
        );
        assert!(
            decision.alternatives.iter().all(|position| !position.tier.is_empty()),
            "a position with no tier is an unranked claim, not evidence"
        );
    }

    /// CORRECTNESS — `docs/PRD_v2/11_porosity.md` SB-POR-001 and SB-POR-T39, adjudicated by
    /// `docs/takeover/DECISIONS.md` DEC-015. The common contract owns family, custody shape,
    /// observable reason schema and output naming; each method retains its own source-bound
    /// numerical policy. `phimax` is deliberately a limit producer rather than a deterministic
    /// interpretation method. The expected inventory is read independently from the shipping
    /// module catalog, so a registry that merely describes itself cannot satisfy this test.
    #[test]
    fn every_porosity_module_uses_one_envelope_while_each_result_producer_keeps_its_own_limit_policy() {
        let modules = module_catalog();
        let live_porosity_modules = modules
            .iter()
            .filter(|module| module.category == "Porosity")
            .map(|module| module.name.as_str())
            .collect::<HashSet<_>>();
        let expected_modules = HashSet::from([
            "phi_den", "phi_dn", "phi_son", "phimax", "ssc", "sspw",
        ]);
        assert_eq!(
            live_porosity_modules, expected_modules,
            "every live Porosity module must be inside the one registered family"
        );

        let expected_outputs = BTreeMap::from([
            (
                "phi_den",
                BTreeMap::from([
                    ("PHIE_DEN", PorosityOutputRole::UnlimitedEffective),
                    ("PHIT_DEN", PorosityOutputRole::UnlimitedTotal),
                    ("PHIE", PorosityOutputRole::LimitedEffective),
                    ("PHIT", PorosityOutputRole::LimitedTotal),
                ]),
            ),
            (
                "phi_dn",
                BTreeMap::from([
                    ("PHIE_DN", PorosityOutputRole::ComparisonUnlimitedEffective),
                    ("PHIT_DN", PorosityOutputRole::ComparisonUnlimitedTotal),
                    ("PHIE", PorosityOutputRole::ComparisonLimitedEffective),
                    ("PHIT", PorosityOutputRole::ComparisonLimitedTotal),
                ]),
            ),
            (
                "phi_son",
                BTreeMap::from([
                    ("PHIE_SON", PorosityOutputRole::LimitedEffective),
                    ("PHIT_SON", PorosityOutputRole::LimitedTotal),
                ]),
            ),
            (
                "phimax",
                BTreeMap::from([
                    ("PHI_CAP", PorosityOutputRole::Capped),
                    ("PHI_MAX", PorosityOutputRole::Ceiling),
                ]),
            ),
            (
                "ssc",
                BTreeMap::from([
                    ("PHIE_GR", PorosityOutputRole::Effective),
                    ("PHIE_SSC", PorosityOutputRole::Effective),
                    ("PHIFF_GR", PorosityOutputRole::FreeFluid),
                    ("PHIFF_SSC", PorosityOutputRole::FreeFluid),
                    ("PHIT_GR", PorosityOutputRole::Total),
                    ("PHIT_SSC", PorosityOutputRole::Total),
                ]),
            ),
            (
                "sspw",
                BTreeMap::from([
                    ("PHIE_SSPW", PorosityOutputRole::Effective),
                    ("PHIFF_SSPW", PorosityOutputRole::FreeFluid),
                    ("PHIT_SSPW", PorosityOutputRole::Total),
                ]),
            ),
        ]);

        let mut method_policies = HashSet::new();
        for module in modules.iter().filter(|module| module.category == "Porosity") {
            let classified = module
                .args
                .iter()
                .filter_map(|argument| {
                    argument.porosity_output.as_ref().map(|contract| {
                        assert_eq!(argument.kind, ArgKind::LogOut);
                        assert_eq!(argument.unit, "v/v");
                        assert_eq!(contract.family, POROSITY_FAMILY_ID);
                        assert_eq!(contract.limiting_contract, POROSITY_LIMITING_CONTRACT);
                        assert_eq!(contract.flag_contract, POROSITY_FLAG_CONTRACT);
                        assert_eq!(
                            contract.flag_emission,
                            PorosityFlagEmission::PendingSbPor003,
                            "SB-POR-001 defines the reason shape; it must not claim SB-POR-003 already emits it"
                        );
                        assert_eq!(
                            contract.output_naming_contract,
                            POROSITY_OUTPUT_NAMING_CONTRACT
                        );
                        assert!(!contract.method.is_empty());
                        assert!(!contract.convention.is_empty());
                        assert!(
                            contract
                                .limiting_policy_source
                                .contains("docs/PRD_v2/11_porosity.md"),
                            "{}.{} limit policy has no chapter source",
                            module.name,
                            argument.name
                        );
                        if contract.module_role != PorosityModuleRole::LimitProducer {
                            method_policies.insert(contract.limiting_policy.as_str());
                        }
                        (argument.name.as_str(), contract.output_role)
                    })
                })
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                classified,
                expected_outputs[module.name.as_str()],
                "{} must classify exactly its live porosity outputs",
                module.name
            );

            let expected_role = match module.name.as_str() {
                "phimax" => PorosityModuleRole::LimitProducer,
                "phi_dn" => PorosityModuleRole::ComparisonProducer,
                _ => PorosityModuleRole::DeterministicMethod,
            };
            assert!(
                module
                    .args
                    .iter()
                    .filter_map(|argument| argument.porosity_output.as_ref())
                    .all(|contract| contract.module_role == expected_role),
                "{} has the wrong POR module role",
                module.name
            );
        }
        assert_eq!(
            method_policies.len(),
            5,
            "density, D-N comparison, sonic, SSC and SSPW must not borrow one another's numerical limit policy"
        );

        assert!(
            modules
                .iter()
                .filter(|module| module.category != "Porosity")
                .flat_map(|module| &module.args)
                .all(|argument| argument.porosity_output.is_none()),
            "POR metadata must not leak onto another module family"
        );

        let mut missing_output = modules.to_vec();
        missing_output
            .iter_mut()
            .find(|module| module.name == "phi_son")
            .unwrap()
            .args
            .iter_mut()
            .find(|argument| argument.name == "PHIE_SON")
            .unwrap()
            .porosity_output = None;
        assert!(
            validate_porosity_contracts(&missing_output)
                .unwrap_err()
                .contains("phi_son.PHIE_SON"),
            "a lazy partial registration must fail the immutable catalog gate"
        );

        let mut borrowed_policy = modules.to_vec();
        let density_policy = borrowed_policy
            .iter()
            .find(|module| module.name == "phi_den")
            .unwrap()
            .args
            .iter()
            .find_map(|argument| argument.porosity_output.as_ref())
            .unwrap()
            .limiting_policy
            .clone();
        for argument in borrowed_policy
            .iter_mut()
            .find(|module| module.name == "phi_son")
            .unwrap()
            .args
            .iter_mut()
            .filter(|argument| argument.porosity_output.is_some())
        {
            argument.porosity_output.as_mut().unwrap().limiting_policy = density_policy.clone();
        }
        assert!(
            validate_porosity_contracts(&borrowed_policy)
                .unwrap_err()
                .contains("borrows"),
            "one universal clamp is not the common contract DEC-015 authorized"
        );

        for module in modules.iter().filter(|module| module.category == "Porosity") {
            let rename_opts = module
                .args
                .iter()
                .filter(|argument| argument.porosity_output.is_some())
                .map(|argument| {
                    (
                        format!("__OUT_{}", argument.name),
                        format!("CHECK_{}_{}", module.name, argument.name),
                    )
                })
                .collect::<HashMap<_, _>>();
            let resolved = crate::workflow::resolve_output_names(module, &rename_opts).unwrap();
            for argument in module.args.iter().filter(|argument| argument.porosity_output.is_some()) {
                let resolved_name = resolved
                    .iter()
                    .find(|(declared, _)| declared == &argument.name)
                    .map(|(_, name)| name)
                    .unwrap();
                assert_eq!(
                    resolved_name,
                    &format!("CHECK_{}_{}", module.name, argument.name).to_uppercase(),
                    "the common output-naming contract must remain user-configurable"
                );
            }
        }
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.3 SB-ENV-030, section 5.2 and
    /// SB-ENV-T38. The source-owned polarity is `1 = true`; the expected inventory names every
    /// ENV/Condition flag-emitting manifest output and deliberately excludes `flip.OUT_FLAG`,
    /// which carries a numeric pivot rather than a flag. The source-declaration count prevents a
    /// second mapping from becoming another convention beside the closed typed one.
    #[test]
    fn every_environment_flag_emitter_uses_the_one_typed_polarity_and_declares_its_flag_kind() {
        let declared: BTreeMap<String, FlagKind> = module_catalog()
            .iter()
            .filter(|module| {
                matches!(
                    module.name.as_str(),
                    "badhole" | "condflag" | "despike" | "smooth" | "clip" | "fill_gaps" | "flip"
                )
            })
            .flat_map(|module| {
                module.args.iter().filter_map(move |argument| {
                    argument
                        .flag_kind
                        .map(|kind| (format!("{}.{}", module.name, argument.name), kind))
                })
            })
            .collect();
        let expected = BTreeMap::from([
            ("badhole.BADHOLE".into(), FlagKind::ExclusionMask),
            (
                "badhole.BADHOLE_CALI_EVALUATED".into(),
                FlagKind::DiagnosticIndicator,
            ),
            (
                "badhole.BADHOLE_DRHO_EVALUATED".into(),
                FlagKind::DiagnosticIndicator,
            ),
            ("condflag.COAL_FLAG".into(), FlagKind::DiagnosticIndicator),
            ("condflag.TIGHT_FLAG".into(), FlagKind::DiagnosticIndicator),
            ("condflag.XOVER_FLAG".into(), FlagKind::DiagnosticIndicator),
            (
                "condflag.SHOULDER_FLAG".into(),
                FlagKind::DiagnosticIndicator,
            ),
            ("condflag.COND_FLAG".into(), FlagKind::ExclusionMask),
            ("despike.OUT_FLAG".into(), FlagKind::DiagnosticIndicator),
            ("smooth.OUT_FLAG".into(), FlagKind::DiagnosticIndicator),
            ("clip.OUT_FLAG".into(), FlagKind::DiagnosticIndicator),
            ("fill_gaps.OUT_FLAG".into(), FlagKind::DiagnosticIndicator),
        ]);
        assert_eq!(declared, expected, "the whole ENV/Condition flag inventory must be typed");
        assert_eq!(
            framework_precondition_flag_kind(),
            FlagKind::DiagnosticIndicator,
            "the framework-owned companion is an indicator, not a user-selected mask"
        );
        assert_eq!(FlagValue::Clear.as_f32(), 0.0);
        assert_eq!(FlagValue::Flagged.as_f32(), 1.0);
        assert!(FlagValue::Missing.as_f32().is_nan());

        let source = include_str!("modules.rs");
        let declaration = ["enum ", "FlagValue"].concat();
        assert_eq!(
            source.matches(&declaration).count(),
            1,
            "a second flag-polarity type would recreate the convention split T38 forbids"
        );
    }

    /// `f64::clamp` panics when the bounds are inverted or non-finite, and module bounds are
    /// themselves parameters — a zone override of SWT_IRR entered as a percentage (25 instead of
    /// 0.25) produced `limit(swt, 25.0, 1.0)` and killed the run. Release builds set
    /// `panic = "abort"`, so this took the whole app down rather than failing one module.
    #[test]
    fn limit_returns_missing_instead_of_panicking_on_bad_bounds() {
        assert_eq!(limit(0.5, 0.0, 1.0), 0.5, "in range, untouched");
        assert_eq!(limit(1.5, 0.0, 1.0), 1.0, "above range, clamped to hi");
        assert_eq!(limit(-0.5, 0.0, 1.0), 0.0, "below range, clamped to lo");
        assert!(limit(f64::NAN, 0.0, 1.0).is_nan(), "missing in, missing out");

        // The three shapes that used to panic.
        assert!(limit(0.5, 25.0, 1.0).is_nan(), "inverted bounds (the percent-entry case)");
        assert!(limit(0.5, f64::NAN, 1.0).is_nan(), "NaN low bound");
        assert!(limit(0.5, 0.0, f64::NAN).is_nan(), "NaN high bound");

        // Equal bounds are degenerate but legal, and must still clamp rather than read missing.
        assert_eq!(limit(0.7, 0.3, 0.3), 0.3, "lo == hi is a valid clamp");
    }

    /// The legacy `multimin` module is retired: it must stay in the catalog so a saved workflow
    /// chain that references it still resolves by name (and can show its stored params), but be
    /// flagged retired so `run_module` blocks it. A live module is not flagged. (The end-to-end
    /// block — a Failed run carrying the SandiMin message through the real runner — is pinned by
    /// `workflow::tests::phase7_generic_store_feeds_modules_and_mask`.)
    #[test]
    fn multimin_is_retired_but_still_cataloged() {
        let msg = retired_module("multimin").expect("multimin must be marked retired");
        assert!(msg.contains("SandiMin"), "retirement message must point at SandiMin: {msg}");
        assert!(retired_module("vsh_gr").is_none(), "a live module must not be flagged retired");
        assert!(
            list_modules().iter().any(|m| m.name == "multimin"),
            "retired multimin must stay in the catalog so saved chains resolve by name"
        );
    }

    /// The KNN z-score floor used `if *s < 1e-9`, which a NaN std slips past (`NaN < x` is false),
    /// making every scaled distance NaN and panicking the neighbour sort on `partial_cmp`.
    #[test]
    fn zscore_std_floor_catches_nan_not_just_small() {
        let floor = |mut s: f64| {
            if !(s >= 1e-9) {
                s = 1.0;
            }
            s
        };
        assert_eq!(floor(2.5), 2.5, "a healthy std is left alone");
        assert_eq!(floor(0.0), 1.0, "a zero-variance predictor is floored");
        assert_eq!(floor(1e-12), 1.0, "a tiny std is floored");
        assert_eq!(floor(f64::NAN), 1.0, "a NaN std is floored — the case the old form missed");
    }

    fn ctx_with(
        n: usize,
        logs: &[(&str, Vec<f32>)],
        params: &[(&str, f64)],
        opts: &[(&str, &str)],
    ) -> ModuleContext {
        ModuleContext {
            n,
            logs: logs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            params: params.iter().map(|(k, v)| (k.to_string(), vec![*v; n])).collect(),
            opts: opts.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            depth_unit: Default::default(),
        }
    }

    /// CORRECTNESS — SB-CORE-003 and `20_envcorr-qc.md` sections 4.1 and 6.1 T01-T05.
    /// The synthetic 8-13/8-18 lb/gal branch ranges are copied from the chapter's explicit
    /// NON-ADOPTABLE verification rows (Geolog `unc_tnph.lls:340,346`); they prove the schema and
    /// are never registered on a shipping module. The live VSH limits come from
    /// `10_clay-volume.md` section 3.2 and Geolog `vsh_gr.info` L48-L49. No uncited physical bound
    /// or product default is introduced here.
    #[test]
    fn source_bearing_precondition_shapes_refuse_before_computation_while_a_valid_public_run_still_computes() {
        let synthetic = ModuleSpec {
            name: "synthetic_branch_validation".into(),
            title: "Synthetic branch validation".into(),
            category: "Test".into(),
            doc: "The cited ranges are verification fixtures, not adopted product values.".into(),
            args: vec![
                with_validity(
                    opt("MUD_TYPE", "Synthetic branch selector", "NORMAL", &["NORMAL", "BARITE"]),
                    vec![validity(
                        "synthetic.mud_type",
                        "The branch selector must name a declared branch.",
                        "docs/PRD_v2/20_envcorr-qc.md §6.1 T01/T03",
                        ValidityRule::Enumeration,
                    )],
                ),
                with_validity(
                    log_in("MUD_WEIGHT", "Synthetic per-sample mud weight", "lb/gal", "MUD_WEIGHT", true),
                    vec![
                        validity(
                            "synthetic.normal_mud_range",
                            "The normal-mud verification branch uses its own stated range.",
                            "Geolog unc_tnph.lls:340 — NON-ADOPTABLE verification fixture",
                            ValidityRule::NumericRange {
                                min: Some(8.0),
                                max: Some(13.0),
                                unit: "lb/gal".into(),
                                when: Some(ValidityBranch { argument: "MUD_TYPE".into(), equals: "NORMAL".into() }),
                            },
                        ),
                        validity(
                            "synthetic.barite_mud_range",
                            "The barite verification branch uses its own stated range.",
                            "Geolog unc_tnph.lls:346 — NON-ADOPTABLE verification fixture",
                            ValidityRule::NumericRange {
                                min: Some(8.0),
                                max: Some(18.0),
                                unit: "lb/gal".into(),
                                when: Some(ValidityBranch { argument: "MUD_TYPE".into(), equals: "BARITE".into() }),
                            },
                        ),
                        validity(
                            "synthetic.caliper_companion",
                            "This synthetic correction cannot be evaluated without a caliper input.",
                            "docs/PRD_v2/20_envcorr-qc.md SB-ENV-001(d) and SB-ENV-016",
                            ValidityRule::RequiredCompanion { any_of: vec!["CALIPER".into()], when: None },
                        ),
                    ],
                ),
                log_in("CALIPER", "Synthetic required companion", "in", "CALIPER", false),
                log_out("CORRECTED", "Synthetic output", "v/v"),
            ],
        };

        let encoded = serde_json::to_string(&synthetic).expect("module manifest serializes");
        let decoded: ModuleSpec = serde_json::from_str(&encoded).expect("module manifest deserializes");
        assert_eq!(decoded, synthetic, "every validity field must survive the manifest round trip");
        assert!(
            synthetic
                .args
                .iter()
                .flat_map(|arg| &arg.validity_conditions)
                .all(|condition| !condition.statement.is_empty() && !condition.source.is_empty()),
            "a bare condition without meaning or source is not a validity contract"
        );

        let synthetic_context = |branch: &str, mud_weight: Vec<f32>, caliper: Option<Vec<f32>>| {
            let mut logs = HashMap::from([("MUD_WEIGHT".into(), mud_weight)]);
            if let Some(values) = caliper {
                logs.insert("CALIPER".into(), values);
            }
            ModuleContext {
                n: 2,
                logs,
                params: HashMap::new(),
                opts: HashMap::from([("MUD_TYPE".into(), branch.into())]),
                depth_unit: Default::default(),
            }
        };

        let normal_error = validate_declared_preconditions(
            &synthetic,
            &synthetic_context("NORMAL", vec![12.0, 14.0], Some(vec![8.5, 8.5])),
        )
        .expect_err("14 lb/gal must fail the 8-13 normal branch at its second sample");
        assert!(normal_error.contains("sample 1") && normal_error.contains("8 to 13"), "per-sample range missing: {normal_error}");
        assert!(normal_error.contains("unc_tnph.lls:340"), "range source missing: {normal_error}");

        validate_declared_preconditions(
            &synthetic,
            &synthetic_context("BARITE", vec![12.0, 14.0], Some(vec![8.5, 8.5])),
        )
        .expect("the same 14 lb/gal sample is valid on the separately declared 8-18 barite branch");

        let companion_error = validate_declared_preconditions(
            &synthetic,
            &synthetic_context("BARITE", vec![12.0, 14.0], None),
        )
        .expect_err("a declared required companion must refuse before computation");
        assert!(companion_error.contains("CALIPER"), "required companion missing: {companion_error}");
        assert!(companion_error.contains("SB-ENV-001(d)"), "companion source missing: {companion_error}");

        let synthetic_option_error = validate_declared_preconditions(
            &synthetic,
            &synthetic_context("TYPO", vec![12.0, 12.0], Some(vec![8.5, 8.5])),
        )
        .expect_err("an undeclared synthetic branch must refuse");
        assert!(synthetic_option_error.contains("TYPO") && synthetic_option_error.contains("NORMAL, BARITE"));
        assert!(synthetic_option_error.contains("§6.1 T01/T03"), "enumeration source missing: {synthetic_option_error}");

        let context = |gr_ma: f64, gr_sh: f64, method: &str, gr: Vec<f32>| {
            ctx_with(
                1,
                &[("GR", gr)],
                &[("GR_MA", gr_ma), ("GR_SH", gr_sh)],
                &[("OPT_GR", method)],
            )
        };

        let option_error = run_module("vsh_gr", &context(20.0, 120.0, "TYPO", vec![70.0]))
            .expect_err("an undeclared method id must never fall through to LINEAR");
        assert!(option_error.contains("vsh_gr.method_id"), "condition id missing: {option_error}");
        assert!(option_error.contains("TYPO"), "offending value missing: {option_error}");
        assert!(option_error.contains("LINEAR"), "permitted set missing: {option_error}");
        assert!(option_error.contains("vsh_gr.lls"), "condition source missing: {option_error}");

        let range_error = run_module("vsh_gr", &context(-1.0, 120.0, "LINEAR", vec![70.0]))
            .expect_err("a value outside the declared GR_MA range must stop before VSH arithmetic");
        assert!(range_error.contains("vsh_gr.gr_ma_range"), "condition id missing: {range_error}");
        assert!(range_error.contains("-1"), "offending value missing: {range_error}");
        assert!(range_error.contains("0") && range_error.contains("200"), "range missing: {range_error}");
        assert!(range_error.contains("vsh_gr.info"), "range source missing: {range_error}");

        let order_error = run_module("vsh_gr", &context(120.0, 20.0, "LINEAR", vec![70.0]))
            .expect_err("inverted endpoints must be refused before the body can silently return NaN");
        assert!(order_error.contains("vsh_gr.endpoint_order"), "relational condition missing: {order_error}");
        assert!(order_error.contains("120") && order_error.contains("20"), "offending pair missing: {order_error}");
        assert!(order_error.contains("SB-CLY-001"), "relational source missing: {order_error}");

        let input_error = run_module("vsh_gr", &context(20.0, 120.0, "LINEAR", vec![f32::NAN]))
            .expect_err("a required curve with no finite sample must refuse instead of returning blank success");
        assert!(input_error.contains("GR"), "required input missing from refusal: {input_error}");

        let mut empty_parameter = context(20.0, 120.0, "LINEAR", vec![70.0]);
        empty_parameter.params.insert("GR_MA".into(), Vec::new());
        let empty_parameter_error = run_module("vsh_gr", &empty_parameter)
            .expect_err("a required parameter with no per-sample values must refuse before computation");
        assert!(
            empty_parameter_error.contains("GR_MA") && empty_parameter_error.contains("frame has 1"),
            "empty parameter refusal is not actionable: {empty_parameter_error}"
        );

        let valid = run_module("vsh_gr", &context(20.0, 120.0, "LINEAR", vec![70.0]))
            .expect("the valid side of the same declared contract must still run");
        assert!((valid["VSH_GR"][0] - 0.5).abs() < 1e-6, "valid LINEAR result changed");
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.1 SB-ENV-009 and section 6.1
    /// SB-ENV-T03. The closed method set is the shipping VSH-GR manifest sourced from Geolog
    /// `vsh_gr.info` / `vsh_gr.lls` and recorded in `10_clay-volume.md` sections 3.2-3.3. The
    /// 20/120/70 gAPI positive control independently gives IGR = (70 - 20) / (120 - 20) = 0.5;
    /// no new endpoint or default is introduced by this test.
    #[test]
    fn an_unknown_method_name_is_refused_with_its_parameter_value_and_permitted_set_before_any_branch_runs() {
        let context = |method: &str| {
            ctx_with(
                1,
                &[("GR", vec![70.0])],
                &[("GR_MA", 20.0), ("GR_SH", 120.0)],
                &[("OPT_GR", method)],
            )
        };

        let error = run_module("vsh_gr", &context("TYPO"))
            .expect_err("an undeclared method must refuse instead of reaching a fallback arm");
        assert!(error.contains("OPT_GR"), "selector name missing: {error}");
        assert!(error.contains("TYPO"), "unrecognised value missing: {error}");
        assert!(
            error.contains(
                "[LINEAR, STIEBER1, STIEBER2, STIEBER3, LARINOV1, LARINOV2, LARINOV3, CLAVIER]"
            ),
            "complete permitted set missing or reordered: {error}"
        );

        let valid = run_module("vsh_gr", &context("LINEAR"))
            .expect("a declared method must remain runnable");
        assert_eq!(valid["VSH_GR"], vec![0.5]);

        // Pin the universal side too. A validator limited to the VSH condition above would still
        // let older Option manifests fall through. Every runnable registered selector must accept
        // its declared default and refuse one value outside its own closed set by name.
        const UNKNOWN: &str = "__UNRECOGNISED_SELECTOR__";
        for spec in module_catalog()
            .iter()
            .filter(|spec| retired_module(&spec.name).is_none())
        {
            validate_module_options(&spec.name, &HashMap::new())
                .unwrap_or_else(|error| panic!("{} rejects its declared option defaults: {error}", spec.name));
            for arg in spec.args.iter().filter(|arg| arg.kind == ArgKind::Option) {
                assert!(!arg.choices.iter().any(|choice| choice == UNKNOWN));
                let error = validate_module_options(
                    &spec.name,
                    &HashMap::from([(arg.name.clone(), UNKNOWN.into())]),
                )
                .unwrap_err();
                assert!(error.contains(&arg.name), "{} selector name missing: {error}", spec.name);
                assert!(error.contains(UNKNOWN), "{} unknown value missing: {error}", spec.name);
                for choice in &arg.choices {
                    assert!(
                        error.contains(choice),
                        "{} permitted choice '{}' missing: {error}",
                        spec.name,
                        choice
                    );
                }
            }
        }
    }

    /// CORRECTNESS — SB-CORE-004 / SB-CORE-T10 and `CONTRACT.md` section 2.
    /// The expected rule comes from the requirement: every shipped numeric default has a
    /// machine-readable source, and an explicit `ABSENT` parameter has no default.
    #[test]
    fn a_registered_default_without_a_source_fails_the_build_gate() {
        let mut bad = ModuleSpec {
            name: "synthetic_unsourced_default".into(),
            title: "Synthetic unsourced default".into(),
            category: "Test".into(),
            doc: "Deliberately invalid registry fixture for SB-CORE-T10.".into(),
            args: vec![param(
                "VALUE",
                "Synthetic default",
                "",
                1.0,
                0.0,
                2.0,
                "docs/PRD_v2/04_CORE_REQUIREMENTS.md SB-CORE-T10 synthetic valid-side source",
            )],
        };
        bad.args[0].default_source.clear();

        let error = validate_parameter_sources(&[bad])
            .expect_err("a numeric default with an empty source must fail the registry build gate");
        assert!(
            error.contains("synthetic_unsourced_default.VALUE"),
            "parameter identity missing: {error}"
        );
        assert!(
            error.contains("default") && error.contains("source"),
            "failure is not actionable: {error}"
        );

        validate_parameter_sources(module_catalog())
            .expect("the complete shipping module registry must contain zero unsourced defaults");
    }

    /// CORRECTNESS — SB-CORE-004 / SB-CORE-T11 and `record_data_tools.md`'s despike-window
    /// decision. `WINDOW` deliberately has no generic value: `ABSENT` is the source-state token,
    /// and supplying a finite interpreter value must make the same module runnable.
    #[test]
    fn an_absent_required_parameter_refuses_until_the_interpreter_supplies_a_value() {
        let spec = module_catalog()
            .iter()
            .find(|module| module.name == "despike")
            .expect("despike is registered");
        let window = spec
            .args
            .iter()
            .find(|arg| arg.name == "WINDOW")
            .expect("WINDOW is declared");
        assert_eq!(window.default_source, ABSENT_DEFAULT_SOURCE);
        assert!(
            window.default.is_empty(),
            "ABSENT must never conceal a numeric default"
        );

        let context = |window: Option<f64>| {
            let mut params = HashMap::from([("K".into(), vec![3.0; 5])]);
            if let Some(value) = window {
                params.insert("WINDOW".into(), vec![value; 5]);
            }
            ModuleContext {
                n: 5,
                logs: HashMap::from([
                    ("DEPTH".into(), vec![0.0, 1.0, 2.0, 3.0, 4.0]),
                    ("CURVE".into(), vec![1.0, 1.0, 10.0, 1.0, 1.0]),
                ]),
                params,
                opts: HashMap::from([("OPT_METHOD".into(), "HAMPEL".into())]),
                depth_unit: Default::default(),
            }
        };

        let error = run_module("despike", &context(None))
            .expect_err("an ABSENT required parameter must refuse before computation");
        assert!(
            error.contains("WINDOW") && error.contains("ABSENT"),
            "refusal is not actionable: {error}"
        );

        // Six depth units cover all five one-unit samples under the module's centred half-open
        // window rule, satisfying the documented five-sample HAMPEL minimum without weakening it.
        let output = run_module("despike", &context(Some(6.0)))
            .expect("supplying the deliberately absent parameter must enable the same run");
        assert!(
            output.values().flatten().any(|value| value.is_finite()),
            "the supplied side must produce a real curve, not blank success"
        );

        // Branch pin, both sides: K is irrelevant to ABS but required by HAMPEL. Source:
        // despike's declared method contract and docs/PRD_v2/20_envcorr-qc.md §5.3.
        let mut abs_context = context(Some(6.0));
        abs_context.opts.insert("OPT_METHOD".into(), "ABS".into());
        abs_context.params.remove("K");
        abs_context.params.insert("THRESH".into(), vec![5.0; 5]);
        run_module("despike", &abs_context)
            .expect("an inactive branch must not demand its deliberately absent parameter");

        let mut hampel_without_k = context(Some(6.0));
        hampel_without_k.params.remove("K");
        let branch_error = run_module("despike", &hampel_without_k)
            .expect_err("the active branch must require its deliberately absent parameter");
        assert!(
            branch_error.contains("K")
                && branch_error.contains("HAMPEL")
                && branch_error.contains("ABSENT"),
            "conditional refusal is not actionable: {branch_error}"
        );
    }

    /// **SB-MLA-050 — the k = 1 self-match trap, as a hard fail.**
    ///
    /// A leave-one-out neighbour search must exclude the held-out sample from its own neighbour
    /// list. Without the exclusion, at `k = 1` every training sample's nearest neighbour is ITSELF
    /// at distance zero: the synthetic reproduces the raw curve exactly, the error is exactly zero,
    /// and every predictor set scores perfectly. Geolog's own reference-HC page documents this trap
    /// for its product, and the requirement takes it as a **hard-fail fixture rather than a
    /// tolerance check** — which is the right call, because the failure is not approximate. It is
    /// exact, and it looks like a triumph.
    ///
    /// The exclusion is one `continue` in `log_predict`. Deleting it broke nothing in this suite
    /// before this test existed; it is the kind of line a later reader removes as redundant, and
    /// the result is a synthetic RHOB that silently echoes the washed-out log it was meant to
    /// replace — defeating the MAX_RAW rule the module exists for.
    ///
    /// Pinned from BOTH sides, and the second side is what makes the first mean anything: a test
    /// asserting only "the error is not zero" passes on a predictor returning garbage. So the
    /// synthetic must ALSO track the target it was fitted on.
    #[test]
    fn a_k1_neighbour_search_that_reproduces_its_training_data_exactly_is_a_failure() {
        // A clean monotone relation on ONE predictor: TARGET = 2*GR + 10, sampled on distinct GR.
        // Each sample's true nearest OTHER neighbour is an adjacent GR, so the honest k = 1
        // prediction is close but never equal.
        let n = 60usize;
        let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32 * 1.5).collect();
        let target: Vec<f32> = gr.iter().map(|g| 2.0 * g + 10.0).collect();
        let ctx = ctx_with(
            n,
            &[("TARGET", target.clone()), ("P1", gr.clone())],
            &[("K", 1.0)],
            &[("OPT_COMBINE", "SYNTHETIC")],
        );
        let out = log_predict(&ctx);
        let syn = &out["SYN"];

        // Side one: NOT a reconstruction. Every sample must differ from its own measured value,
        // because its own value was withheld from the search that produced it.
        let mut exact = 0usize;
        let mut worst = 0.0f32;
        for i in 0..n {
            assert!(syn[i].is_finite(), "sample {i} produced no prediction");
            let err = (syn[i] - target[i]).abs();
            if err == 0.0 {
                exact += 1;
            }
            worst = worst.max(err);
        }
        assert_eq!(
            exact, 0,
            "{exact} of {n} samples were reproduced EXACTLY - the neighbour search is matching each \
             sample against itself, so a k=1 score on this data is meaningless and every predictor \
             set would look perfect"
        );
        // The spacing is 1.5 gAPI, so the nearest other neighbour differs by 3.0 in the target.
        // Anything much below that would mean a self-match is leaking in partially.
        assert!(worst >= 2.9, "the largest error was {worst}, too small for a genuine hold-out");

        // Side two: still a working predictor, or side one is satisfied by returning nonsense.
        // Away from the two ends every neighbour is one step out, so the error stays near 3.0.
        for i in 1..n - 1 {
            let err = (syn[i] - target[i]).abs();
            assert!(
                err <= 4.0,
                "sample {i} missed by {err} - the exclusion must hold out the sample, not break the fit"
            );
        }
    }

    #[test]
    fn precalc_kk_fits_and_conductivities() {
        // Example fits from one study: FTEMP = 77 + 0.0260292*TVDSS [degF],
        // FPRESS = 44.2823 + 0.539812*TVDSS [psi], TVDSS in ft.
        let ctx = ctx_with(
            2,
            &[
                ("TVDSS", vec![5000.0, f32::NAN]),
                ("RT", vec![5.0, 0.0]),
                ("RXO", vec![2.0, f32::NAN]),
            ],
            &[
                ("SURF_TEMP", 77.0),
                ("TEMP_GRAD", 0.0260292),
                ("PSURF", 44.2823),
                ("PGRAD", 0.539812),
                ("RMF_MEAS", 0.2),
                ("RMF_TEMP", 75.0),
            ],
            &[("OPT_TU", "degF"), ("OPT_RMF", "ARPS")],
        );
        let out = precalc(&ctx);
        // Params entered in degF → FTEMP_F carries the fit; FTEMP is the degC twin.
        assert!((out["FTEMP_F"][0] - 207.146).abs() < 0.01, "FTEMP_F {}", out["FTEMP_F"][0]);
        let degc = (207.146 - 32.0) / 1.8;
        assert!((out["FTEMP"][0] as f64 - degc).abs() < 0.01, "FTEMP {}", out["FTEMP"][0]);
        assert!((out["FPRESS"][0] - 2743.34).abs() < 0.1, "FPRESS {}", out["FPRESS"][0]);
        let arps = 0.2 * (75.0 + 6.77) / (207.146 + 6.77);
        assert!((out["RMF"][0] as f64 - arps).abs() < 1e-4, "RMF {}", out["RMF"][0]);
        assert!((out["CT"][0] - 200.0).abs() < 1e-3);
        assert!((out["CXO"][0] - 500.0).abs() < 1e-3);
        // Missing TVDSS sample → trend outputs missing; RT <= 0 / missing RXO → missing.
        assert!(out["FTEMP"][1].is_nan() && out["FTEMP_F"][1].is_nan());
        assert!(out["FPRESS"][1].is_nan() && out["RMF"][1].is_nan());
        assert!(out["CT"][1].is_nan() && out["CXO"][1].is_nan());
    }

    #[test]
    fn phi_son_wyllie_cp_opt_in_only_scales_wyllie() {
        // DT=75, matrix 55.5, fluid 189, shale 90 → raw Wyllie PHIT = 19.5/133.5 = 0.146067.
        let logs = [("DT", vec![75.0f32]), ("VSH", vec![0.0f32])];
        let params = [("DT_MA", 55.5), ("DT_FL", 189.0), ("DT_SH", 90.0)];
        let raw = 19.5 / 133.5;

        // OPT_CP OFF (default) — plain straight time-average, unchanged.
        let off = phi_son(&ctx_with(1, &logs, &params, &[("OPT_SON", "WYLLIE"), ("OPT_CP", "OFF")]));
        assert!((off["PHIT_SON"][0] as f64 - raw).abs() < 1e-5, "OFF {}", off["PHIT_SON"][0]);

        // OPT_CP ON — divided by Cp = DT_SH/100 = 0.9 → ~11% higher porosity.
        let on = phi_son(&ctx_with(1, &logs, &params, &[("OPT_SON", "WYLLIE"), ("OPT_CP", "ON")]));
        assert!((on["PHIT_SON"][0] as f64 - raw / 0.9).abs() < 1e-5, "ON {}", on["PHIT_SON"][0]);

        // RHG is self-compacting: OPT_CP=ON must NOT touch its porosity.
        let rhg_cp = phi_son(&ctx_with(1, &logs, &params, &[("OPT_SON", "RHG"), ("OPT_CP", "ON")]));
        let rhg_expect = 0.625 * (75.0 - 55.5) / 75.0;
        assert!((rhg_cp["PHIT_SON"][0] as f64 - rhg_expect).abs() < 1e-5, "RHG+CP {}", rhg_cp["PHIT_SON"][0]);
    }

    #[test]
    fn vsh_dn_flags_offmodel_and_gr_divergence() {
        // Sample 0: clean sand at the matrix point, GR clean → VSH≈0, consistent → not flagged.
        // Sample 1: N-D reads very shaly (VSH≈0.92) but GR reads clean → divergence → flagged.
        // Sample 2: RHOB well below matrix → VSH off the triangle (<0) → flagged even w/o GR help.
        let ctx = ctx_with(
            3,
            &[
                ("RHOB", vec![2.65, 2.55, 2.20]),
                ("NPHI", vec![0.00, 0.30, 0.00]),
                ("GR", vec![15.0, 15.0, 15.0]),
            ],
            &[
                ("RHO_MA", 2.65),
                ("RHO_SH", 2.50),
                ("RHO_FL", 1.00),
                ("NPHI_MA", 0.00),
                ("NPHI_SH", 0.35),
                ("NPHI_FL", 1.00),
                ("GR_MA", 15.0),
                ("GR_SH", 120.0),
                ("FLAG_TOL", 0.25),
            ],
            &[],
        );
        let out = vsh_dn(&ctx);
        assert_eq!(out["VSH_DN_FLAG"][0], 0.0, "clean & consistent → not flagged");
        assert_eq!(out["VSH_DN_FLAG"][1], 1.0, "N-D shaly but GR clean → flagged");
        assert_eq!(out["VSH_DN_FLAG"][2], 1.0, "off-model crossover → flagged");
        assert!(out["VSH_DN"][2] < 0.0, "sample 2 sits below the matrix line");

        // Without GR the divergence path is inert, but off-model detection still fires.
        let ctx_no_gr = ctx_with(
            2,
            &[("RHOB", vec![2.55, 2.20]), ("NPHI", vec![0.30, 0.00])],
            &[
                ("RHO_MA", 2.65),
                ("RHO_SH", 2.50),
                ("RHO_FL", 1.00),
                ("NPHI_MA", 0.00),
                ("NPHI_SH", 0.35),
                ("NPHI_FL", 1.00),
                ("GR_MA", 15.0),
                ("GR_SH", 120.0),
                ("FLAG_TOL", 0.25),
            ],
            &[],
        );
        let out2 = vsh_dn(&ctx_no_gr);
        assert_eq!(out2["VSH_DN_FLAG"][0], 0.0, "no GR → shaly point not divergence-flagged");
        assert_eq!(out2["VSH_DN_FLAG"][1], 1.0, "off-model still flagged without GR");
    }

    #[test]
    fn vsh_dn_degenerate_triangle_is_missing_not_inf() {
        // Matrix point == shale point (rho AND nphi coincide) collapses the (c - d) denominator
        // to 0, which without the guard sends the UNLIMITED VSH_DN to +/-Infinity on every sample.
        let ctx = ctx_with(
            1,
            &[("RHOB", vec![2.40]), ("NPHI", vec![0.15])],
            &[
                ("RHO_MA", 2.65), ("RHO_SH", 2.65), ("RHO_FL", 1.00),
                ("NPHI_MA", 0.30), ("NPHI_SH", 0.30), ("NPHI_FL", 1.00),
                ("GR_MA", 15.0), ("GR_SH", 120.0), ("FLAG_TOL", 0.25),
            ],
            &[],
        );
        let out = vsh_dn(&ctx);
        assert!(!out["VSH_DN"][0].is_infinite(), "VSH_DN must never be +/-Infinity, was {}", out["VSH_DN"][0]);
        assert!(out["VSH_DN"][0].is_nan(), "degenerate triangle → missing, was {}", out["VSH_DN"][0]);
        assert!(out["VSH"][0].is_nan());
    }

    #[test]
    fn ftemp_grad_bht_nonpositive_td_is_missing() {
        // A zone override can force TD_BHT <= 0 past the dialog range; the BHT interpolation would
        // then divide by <= 0 and yield a finite-looking +/-Infinity FTEMP.
        let bad = ftemp_grad(&ctx_with(
            1,
            &[("DEPTH", vec![1500.0])],
            &[("TSURF", 26.7), ("BHT", 100.0), ("TD_BHT", 0.0), ("TGRAD", 0.03)],
            &[("OPT_FT", "BHT")],
        ));
        assert!(bad["FTEMP"][0].is_nan(), "TD_BHT=0 → missing, was {}", bad["FTEMP"][0]);
        // A valid TD_BHT still interpolates linearly.
        let good = ftemp_grad(&ctx_with(
            1,
            &[("DEPTH", vec![1000.0])],
            &[("TSURF", 26.7), ("BHT", 100.0), ("TD_BHT", 2000.0), ("TGRAD", 0.03)],
            &[("OPT_FT", "BHT")],
        ));
        let expect = 26.7 + (100.0 - 26.7) * 1000.0 / 2000.0;
        assert!((good["FTEMP"][0] as f64 - expect).abs() < 1e-3, "FTEMP {}", good["FTEMP"][0]);
    }

    /// T-PREP-02. Both temperature models, pinned at the anchors that define them.
    ///
    /// GRADIENT is a straight line THROUGH the surface temperature: at zero depth it reads TSURF
    /// exactly and every metre adds TGRAD. BHT is a different statement entirely — an
    /// interpolation onto a temperature somebody MEASURED — so at TD_BHT it must land on BHT
    /// exactly. That landing is the whole reason the mode exists; a BHT run that misses the
    /// measurement it was handed is not a BHT run.
    ///
    /// The two are deliberately given parameters that DISAGREE below surface (86.7 against 100
    /// degC at 2000 m), so an OPT_FT that stopped switching fails here rather than quietly
    /// returning the gradient answer under a BHT label. Nothing on a log would show that: both
    /// curves are smooth, monotonic and entirely plausible, and the error would only surface much
    /// later as an Rw that is wrong by a few percent everywhere.
    ///
    /// Below TD_BHT the interpolation EXTRAPOLATES, and that is pinned rather than assumed. It is
    /// the honest behaviour — the trend is all the evidence there is past the measurement — but it
    /// means FTEMP below TD is no longer anchored on anything, and if that is ever clamped instead,
    /// this test forces the decision into the open rather than letting it change silently.
    #[test]
    fn formation_temperature_lands_on_both_of_its_anchors() {
        let depths = vec![0.0f32, 1000.0, 2000.0, 3000.0, f32::NAN];
        let params = [("TSURF", 26.7), ("TGRAD", 0.03), ("BHT", 100.0), ("TD_BHT", 2000.0)];
        let logs = [("DEPTH", depths)];
        let n = 5;

        let grad = ftemp_grad(&ctx_with(n, &logs, &params, &[("OPT_FT", "GRADIENT")]))["FTEMP"].clone();
        assert!((grad[0] as f64 - 26.7).abs() < 1e-3, "surface must read TSURF, got {}", grad[0]);
        assert!((grad[2] as f64 - 86.7).abs() < 1e-3, "TSURF + 0.03*2000, got {}", grad[2]);
        let (d1, d2) = (grad[1] - grad[0], grad[2] - grad[1]);
        assert!((d1 - d2).abs() < 1e-3, "GRADIENT must be a straight line: {d1} vs {d2}");

        let bht = ftemp_grad(&ctx_with(n, &logs, &params, &[("OPT_FT", "BHT")]))["FTEMP"].clone();
        assert!((bht[0] as f64 - 26.7).abs() < 1e-3, "both modes start at TSURF, got {}", bht[0]);
        assert!(
            (bht[2] as f64 - 100.0).abs() < 1e-3,
            "BHT mode must land ON the measured BHT at TD_BHT, got {}",
            bht[2]
        );
        assert!((bht[1] as f64 - 63.35).abs() < 1e-3, "half way is the mean, got {}", bht[1]);

        // The control: the modes must actually disagree, or the switch proves nothing.
        assert!(
            (bht[2] - grad[2]).abs() > 10.0,
            "OPT_FT stopped switching — both modes returned {} at TD",
            grad[2]
        );

        // Past the measurement the trend simply continues: 26.7 + 73.3*1.5.
        assert!((bht[3] as f64 - 136.65).abs() < 1e-2, "below TD_BHT it extrapolates, got {}", bht[3]);

        // No depth, no temperature — in either mode.
        assert!(grad[4].is_nan() && bht[4].is_nan(), "a missing depth must not produce a temperature");
    }

    /// CORRECTNESS — `crate::units::M_PER_FT` is the exact international foot from
    /// NIST SP 811. The module manifests qualify TGRAD, TD_BHT, SHIFT and SPLICE_DEPTH
    /// in metres, so changing only the project's stored depth unit cannot change the
    /// physical answer.
    #[test]
    fn metre_qualified_depth_parameters_produce_the_same_results_in_foot_and_metre_projects() {
        use crate::units::{DepthUnit, M_PER_FT};

        let as_feet = |depths: &[f32]| {
            depths.iter().map(|d| (*d as f64 / M_PER_FT) as f32).collect::<Vec<_>>()
        };

        let temperature_params = [("TSURF", 20.0), ("TGRAD", 0.03), ("BHT", 80.0), ("TD_BHT", 2000.0)];
        for mode in ["GRADIENT", "BHT"] {
            let metre_depths = vec![0.0, 1000.0, 2000.0];
            let metre = ftemp_grad(&ctx_with(
                3,
                &[("DEPTH", metre_depths.clone())],
                &temperature_params,
                &[("OPT_FT", mode)],
            ));
            let mut foot_ctx = ctx_with(
                3,
                &[("DEPTH", as_feet(&metre_depths))],
                &temperature_params,
                &[("OPT_FT", mode)],
            );
            foot_ctx.depth_unit = DepthUnit::Feet;
            let feet = ftemp_grad(&foot_ctx);
            for i in 0..3 {
                assert!(
                    (metre["FTEMP"][i] - feet["FTEMP"][i]).abs() < 1e-3,
                    "{mode} changed at sample {i}: metre={} foot={}",
                    metre["FTEMP"][i],
                    feet["FTEMP"][i]
                );
            }
        }

        let metre_depths = vec![1000.0, 1001.0, 1002.0, 1003.0, 1004.0];
        let values = vec![0.0, 10.0, 20.0, 30.0, 40.0];
        let metre_shift = depth_shift(&ctx_with(
            5,
            &[("DEPTH", metre_depths.clone()), ("CURVE", values.clone())],
            &[("SHIFT", 1.0)],
            &[],
        ));
        let mut foot_shift_ctx = ctx_with(
            5,
            &[("DEPTH", as_feet(&metre_depths)), ("CURVE", values)],
            &[("SHIFT", 1.0)],
            &[],
        );
        foot_shift_ctx.depth_unit = DepthUnit::Feet;
        let foot_shift = depth_shift(&foot_shift_ctx);
        for i in 0..5 {
            let (metre, feet) = (metre_shift["CURVE_DS"][i], foot_shift["CURVE_DS"][i]);
            assert!(
                (metre.is_nan() && feet.is_nan()) || (metre - feet).abs() < 1e-3,
                "a one-metre shift changed at sample {i}: metre={metre} foot={feet}"
            );
        }

        let top = vec![1.0; 5];
        let bottom = vec![2.0; 5];
        let metre_splice = splice(&ctx_with(
            5,
            &[
                ("DEPTH", metre_depths.clone()),
                ("TOP_CURVE", top.clone()),
                ("BOT_CURVE", bottom.clone()),
            ],
            &[("SPLICE_DEPTH", 1002.0)],
            &[],
        ));
        let mut foot_splice_ctx = ctx_with(
            5,
            &[
                ("DEPTH", as_feet(&metre_depths)),
                ("TOP_CURVE", top),
                ("BOT_CURVE", bottom),
            ],
            &[("SPLICE_DEPTH", 1002.0)],
            &[],
        );
        foot_splice_ctx.depth_unit = DepthUnit::Feet;
        let foot_splice = splice(&foot_splice_ctx);
        assert_eq!(metre_splice["SPLICED"], foot_splice["SPLICED"]);
    }

    /// CORRECTNESS — `20_envcorr-qc.md` SB-ENV-057 / exact SB-ENV-T67 defines `depth` as the
    /// single manifest token for a length expressed in the project's declared depth unit. The
    /// complete inventory applies that contract to the nine current native-depth parameters.
    /// `SHIFT` and `SPLICE_DEPTH` pin the other side: they are explicitly metre-qualified and the
    /// NIST-backed equivalence test above proves that their implementations convert rather than
    /// consume a native project-unit value.
    #[test]
    fn every_project_depth_length_parameter_uses_one_token_while_metre_qualified_parameters_stay_metres(
    ) {
        use std::collections::BTreeSet;

        let expected: BTreeSet<(&str, &str)> = [
            ("despike", "WINDOW"),
            ("smooth", "WINDOW"),
            ("fill_gaps", "MAX_GAP"),
            ("block", "INTERVAL"),
            ("block", "MIN_BED"),
            ("bed_detect", "MIN_BED"),
            ("condflag", "MIN_THICK"),
            ("condflag", "SHOULDER"),
            ("phimax", "TVDSS_REF"),
        ]
        .into_iter()
        .collect();

        let mut declared = BTreeSet::new();
        for module in module_catalog() {
            for argument in module.args.iter().filter(|argument| argument.kind == ArgKind::Param) {
                if expected.contains(&(module.name.as_str(), argument.name.as_str())) {
                    assert_eq!(
                        argument.unit, PROJECT_DEPTH_UNIT_TOKEN,
                        "{}.{} must use the one project-depth-length token",
                        module.name, argument.name
                    );
                    declared.insert((module.name.as_str(), argument.name.as_str()));
                }
                assert!(
                    !matches!(argument.unit.as_str(), "m|ft" | "ft|m"),
                    "{}.{} retains the ambiguous legacy project-depth token {:?}",
                    module.name,
                    argument.name,
                    argument.unit
                );
            }
        }
        assert_eq!(declared, expected, "the complete T43/T67 project-depth inventory changed");

        validate_project_depth_unit_tokens(module_catalog())
            .expect("the shipping registry uses no ambiguous project-depth parameter token");
        let invalid = module_catalog()
            .iter()
            .find(|module| module.name == "condflag")
            .expect("condflag is registered")
            .clone();
        for ambiguous_unit in ["m|ft", "ft|m"] {
            let mut mutated = invalid.clone();
            mutated
                .args
                .iter_mut()
                .find(|argument| argument.name == "MIN_THICK")
                .expect("condflag.MIN_THICK is declared")
                .unit = ambiguous_unit.into();
            let error = validate_project_depth_unit_tokens(&[mutated])
                .expect_err("an ambiguous project-depth token must fail the registry gate");
            assert!(
                error.contains(&format!("condflag.MIN_THICK={ambiguous_unit}")),
                "parameter identity missing: {error}"
            );
        }

        for (module_name, argument_name) in
            [("depth_shift", "SHIFT"), ("splice", "SPLICE_DEPTH")]
        {
            let module = module_catalog()
                .iter()
                .find(|module| module.name == module_name)
                .unwrap_or_else(|| panic!("{module_name} is registered"));
            let argument = module
                .args
                .iter()
                .find(|argument| argument.name == argument_name)
                .unwrap_or_else(|| panic!("{module_name}.{argument_name} is declared"));
            assert_eq!(
                argument.unit, "m",
                "{module_name}.{argument_name} is fixed in metres and must not masquerade as native project depth"
            );
        }
    }

    /// T-PREP-16 steps 1, 2 and 4: the two combine rules that make a synthetic log usable, and
    /// the refusal when there is not enough rock to learn from. (Step 3, the masked-washout case,
    /// lives in `workflow.rs` — the re-blanking happens in the runner, not here.)
    ///
    /// A straight line between the predictor and the target is deliberate. KNN cannot be checked
    /// against a closed form, so the fixture is chosen to make the right answer knowable: a
    /// prediction that lands near the line is following the data, and one that does not is not.
    ///
    /// **FILL_MISSING must return the RAW value bit for bit where the log exists**, not the
    /// prediction. A synthetic that quietly overwrote good measurements with its own smoothed
    /// version would look better than the real log — smoother, no noise, no spikes — which is
    /// exactly why it would never be questioned.
    ///
    /// **MAX_RAW is a one-sided rule and the asymmetry is the physics**: a washed-out hole reads
    /// density LOW because the tool sees mud, never high, so where the prediction exceeds the
    /// measurement the measurement is the suspect one — and where the measurement is higher it
    /// stands, whatever the model thinks. A symmetric rule would let the model erase real tight
    /// streaks, which are exactly the thin beds a synthetic log is worst at reproducing.
    ///
    /// **Under ten training samples writes nothing at all.** A five-point KNN fitted on nine
    /// points is not a model of the formation, it is a model of nine points — and it would
    /// return confident, plausible numbers across the whole well with no sign of how little it
    /// was built from.
    #[test]
    fn a_synthetic_log_fills_gaps_keeps_raw_and_repairs_only_downward() {
        // GR 20..115 by 5; DT = 50 + GR. One predictor is enough to make the relation checkable.
        let gr: Vec<f32> = (0..20).map(|i| 20.0 + i as f32 * 5.0).collect();
        let dt_true: Vec<f32> = gr.iter().map(|g| 50.0 + g).collect();
        let opts = [("OPT_COMBINE", "FILL_MISSING"), ("__IN_TARGET", "DT")];

        // A gap in the middle, so the neighbours straddle it rather than only sitting to one side.
        let mut dt = dt_true.clone();
        dt[9] = f32::NAN;
        dt[10] = f32::NAN;
        let out = log_predict(&ctx_with(
            20,
            &[("TARGET", dt.clone()), ("P1", gr.clone())],
            &[("K", 5.0)],
            &opts,
        ));
        let syn = &out["SYN"];

        for i in 0..20 {
            if dt[i].is_nan() {
                continue;
            }
            assert_eq!(
                syn[i], dt[i],
                "sample {i}: FILL_MISSING must hand back the measurement untouched"
            );
        }
        let (lo, hi) = (dt_true[0], dt_true[19]);
        for i in [9usize, 10] {
            let v = syn[i];
            assert!(!v.is_nan(), "the gap at {i} was not filled");
            assert!(
                v >= lo && v <= hi,
                "sample {i}: {v} is outside the range it was trained on ({lo}..{hi}) — a KNN \
                 average cannot leave the hull of its neighbours, so this is extrapolated nonsense"
            );
            assert!(
                (v - dt_true[i]).abs() < 10.0,
                "sample {i}: {v} does not track the relation it was given (true {})",
                dt_true[i]
            );
        }

        // MAX_RAW: one sample reading far too LOW (a washout) and one reading HIGH.
        let mut dt = dt_true.clone();
        let (washed, high) = (4usize, 14usize);
        dt[washed] = 60.0; // true 90 — the hole, not the rock
        dt[high] = 200.0; // true 140 — high, and therefore trusted
        let out = log_predict(&ctx_with(
            20,
            &[("TARGET", dt.clone()), ("P1", gr.clone())],
            &[("K", 5.0)],
            &[("OPT_COMBINE", "MAX_RAW"), ("__IN_TARGET", "DT")],
        ));
        let syn = &out["SYN"];
        assert!(
            syn[washed] > dt[washed] + 20.0,
            "the depressed sample was not repaired: {} vs raw {}",
            syn[washed],
            dt[washed]
        );
        assert!(
            (syn[washed] - dt_true[washed]).abs() < 15.0,
            "the repair should land near the trend, got {} for a true {}",
            syn[washed],
            dt_true[washed]
        );
        assert_eq!(
            syn[high], dt[high],
            "a reading ABOVE the prediction must stand — bad hole only pushes the log down, so \
             the model has no standing to pull a high measurement back to the trend"
        );

        // Under ten training samples: nothing is written, rather than a confident guess.
        let out = log_predict(&ctx_with(
            6,
            &[
                ("TARGET", dt_true[..6].to_vec()),
                ("P1", gr[..6].to_vec()),
            ],
            &[("K", 5.0)],
            &opts,
        ));
        assert!(
            out["SYN"].iter().all(|v| v.is_nan()),
            "six samples is not a training set — the module must write nothing"
        );
    }

    #[test]
    fn perm_wyllie_rose_negative_phie_missing_across_all_variants() {
        // Negative PHIE is non-physical. TIMUR's fractional exponent already NaN'd it, but the
        // integer MORRIS_BIGGS/TIXIER exponent produced a finite, plausible PERM — all four skip.
        for variant in ["TIMUR", "MORRIS_BIGGS_OIL", "MORRIS_BIGGS_GAS", "TIXIER"] {
            let out = perm_wyllie_rose(&ctx_with(
                1,
                &[("PHIE", vec![-0.1])],
                &[("SWE_IRR", 0.2)],
                &[("OPT_WR", variant)],
            ));
            assert!(out["PERM"][0].is_nan(), "{variant}: negative PHIE must be missing, was {}", out["PERM"][0]);
        }
    }

    #[test]
    fn perm_wyllie_rose_edges() {
        // phi=0 → PERM 0 (not NaN/panic); missing PHIE → NaN; swirr<=0 → NaN; valid → finite +ve.
        let z = perm_wyllie_rose(&ctx_with(1, &[("PHIE", vec![0.0])], &[("SWE_IRR", 0.2)], &[]));
        assert_eq!(z["PERM"][0], 0.0);
        let m = perm_wyllie_rose(&ctx_with(1, &[("PHIE", vec![f32::NAN])], &[("SWE_IRR", 0.2)], &[]));
        assert!(m["PERM"][0].is_nan());
        let s = perm_wyllie_rose(&ctx_with(1, &[("PHIE", vec![0.2])], &[("SWE_IRR", 0.0)], &[]));
        assert!(s["PERM"][0].is_nan());
        let ok = perm_wyllie_rose(&ctx_with(1, &[("PHIE", vec![0.2])], &[("SWE_IRR", 0.2)], &[]));
        assert!(ok["PERM"][0].is_finite() && ok["PERM"][0] > 0.0);
    }

    /// T-PETRO-14 — each Wyllie-Rose variant must carry its OWN published constants, and two of
    /// them are deliberately the same equation.
    ///
    /// `perm_wyllie_rose_edges` and `..._negative_phie_missing_across_all_variants` already pin the
    /// guards; what was never checked is that OPT_WR actually selects different physics. A variant
    /// wired to the wrong constants is the silent kind of wrong — permeability comes back finite,
    /// positive, correctly shaped against porosity, and an order of magnitude off.
    ///
    /// Values hand-derived from PERM = (C·φ^D / Swirr^E)² at the plan's domain-sanity point
    /// φ = 0.25, Swirr = 0.15, with the constants in the module doc:
    ///   TIMUR              C=100 D=2.25 E=1 → (100·0.25^2.25 / 0.15)² = 868.06 mD
    ///   MORRIS_BIGGS_OIL   C=250 D=3    E=1 → (250·0.015625 / 0.15)²  = 678.17 mD
    ///   MORRIS_BIGGS_GAS   C=79  D=3    E=1 → (79·0.015625  / 0.15)²  =  67.72 mD
    ///   TIXIER             C=250 D=3    E=1 → same equation as MORRIS_BIGGS_OIL
    #[test]
    fn the_wyllie_rose_variants_carry_their_own_constants_and_two_are_one_equation() {
        let run = |variant: &str| -> f64 {
            perm_wyllie_rose(&ctx_with(
                1,
                &[("PHIE", vec![0.25f32])],
                &[("SWE_IRR", 0.15)],
                &[("OPT_WR", variant)],
            ))["PERM_WR"][0] as f64
        };
        let timur = run("TIMUR");
        let oil = run("MORRIS_BIGGS_OIL");
        let gas = run("MORRIS_BIGGS_GAS");
        let tixier = run("TIXIER");

        assert!((timur - 868.06).abs() < 0.5, "TIMUR {timur}");
        assert!((oil - 678.17).abs() < 0.5, "MORRIS_BIGGS_OIL {oil}");
        assert!((gas - 67.72).abs() < 0.5, "MORRIS_BIGGS_GAS {gas}");

        // TIXIER is MORRIS_BIGGS_OIL in this port — identical to the last bit, not merely close.
        // Documented in the module doc; asserted here so a future edit to one must touch both.
        assert_eq!(oil, tixier, "TIXIER and MORRIS_BIGGS_OIL are the same C/D/E in this port");

        // The gas variant sits a full decade below the oil one — (250/79)² = 10.01 — which is the
        // whole reason the choice matters. Anything less than a decade means a variant is misread.
        assert!(oil / gas > 9.9, "gas must be ~1 decade below oil: {oil} / {gas} = {}", oil / gas);
        assert!(timur > oil, "TIMUR is the highest of the four at this point");

        // An unknown OPT_WR falls back to TIMUR rather than failing. Pinned so the fallback stays
        // a deliberate choice — a typo in a saved chain must not silently become a different rock.
        assert_eq!(run("NOT_A_VARIANT"), timur, "an unrecognised variant falls back to TIMUR");
    }

    #[test]
    fn perm_coates_computes_and_handles_edges() {
        // PERM = (C*phi^2*(1-swirr)/swirr)^2. C=100, phi=0.2, swirr=0.2 →
        // inner = 100*0.04*0.8/0.2 = 16 → PERM = 256.
        let out = perm_coates(&ctx_with(1, &[("PHIE", vec![0.2])], &[("CONST_COATES", 100.0), ("SWE_IRR", 0.2)], &[]));
        assert!((out["PERM"][0] as f64 - 256.0).abs() < 1e-2, "PERM {}", out["PERM"][0]);
        let z = perm_coates(&ctx_with(1, &[("PHIE", vec![0.0])], &[("CONST_COATES", 100.0), ("SWE_IRR", 0.2)], &[]));
        assert_eq!(z["PERM"][0], 0.0);
        let s = perm_coates(&ctx_with(1, &[("PHIE", vec![0.2])], &[("CONST_COATES", 100.0), ("SWE_IRR", 0.0)], &[]));
        assert!(s["PERM"][0].is_nan());
    }

    #[test]
    fn perm_transform_overflow_is_missing() {
        // PT_A=100, PT_B=5 are inside the dialog ranges; at PHIE=0.5 the exponent is 55 and 10^55
        // overflows the f32 cast to +Infinity — must be emitted as MISSING, not +inf.
        let big = perm_transform(&ctx_with(1, &[("PHIE", vec![0.5])], &[("PT_A", 100.0), ("PT_B", 5.0)], &[]));
        assert!(!big["PERM"][0].is_infinite(), "must not be +inf, was {}", big["PERM"][0]);
        assert!(big["PERM"][0].is_nan());
        // Normal calibration stays finite: 10^(20*0.2 - 3) = 10^1 = 10.
        let ok = perm_transform(&ctx_with(1, &[("PHIE", vec![0.2])], &[("PT_A", 20.0), ("PT_B", -3.0)], &[]));
        assert!((ok["PERM"][0] as f64 - 10.0).abs() < 1e-2, "PERM {}", ok["PERM"][0]);
    }

    #[test]
    fn precalc_rmf_trend_and_depth_fallback() {
        // No TVDSS curve at all → whole-curve fallback to measured DEPTH.
        // TREND regression (RMF = 0.517068 - 0.116517*log10(TVDSS)) is already at FTEMP.
        let ctx = ctx_with(
            1,
            &[("DEPTH", vec![5000.0])],
            &[
                ("SURF_TEMP", 77.0),
                ("TEMP_GRAD", 0.0260292),
                ("PSURF", 44.2823),
                ("PGRAD", 0.539812),
                ("RMF_A", 0.517068),
                ("RMF_B", -0.116517),
            ],
            &[("OPT_TU", "degF"), ("OPT_RMF", "TREND")],
        );
        let out = precalc(&ctx);
        assert!((out["FTEMP_F"][0] - 207.146).abs() < 0.01);
        let expect = 0.517068 - 0.116517 * 5000f64.log10();
        assert!((out["RMF"][0] as f64 - expect).abs() < 1e-4, "RMF {}", out["RMF"][0]);
    }

    #[test]
    fn precalc_trend_guards_nonpositive_depth_and_rmf() {
        // log10 is undefined at TVDSS <= 0 (samples above the subsea datum) and the
        // TREND fit goes non-positive at great depth — both must stay MISSING while
        // FTEMP/FPRESS (plain linear trends) still compute.
        let ctx = ctx_with(
            3,
            &[("TVDSS", vec![0.0, -50.0, 100000.0])],
            &[
                ("SURF_TEMP", 77.0),
                ("TEMP_GRAD", 0.026),
                ("PSURF", 0.0),
                ("PGRAD", 0.433),
                ("RMF_A", 0.517),
                ("RMF_B", -0.1165),
            ],
            &[("OPT_TU", "degF"), ("OPT_RMF", "TREND")],
        );
        let out = precalc(&ctx);
        assert!(out["RMF"][0].is_nan() && out["RMF"][1].is_nan());
        // 0.517 - 0.1165*log10(100000) = 0.517 - 0.5825 < 0 → physically meaningless.
        assert!(out["RMF"][2].is_nan());
        assert!(out["FTEMP"][0].is_finite() && out["FPRESS"][1].is_finite());
    }

    #[test]
    fn precalc_degc_mode_converts_for_arps() {
        // Metric well: 25 degC + 0.03 degC/m at 2000 m → FTEMP 85 degC (= 185 degF).
        // Rmf 0.2 ohmm @ 25 degC (77 degF) Arps-converted to 185 degF.
        let ctx = ctx_with(
            1,
            &[("TVDSS", vec![2000.0])],
            &[
                ("SURF_TEMP", 25.0),
                ("TEMP_GRAD", 0.03),
                ("PSURF", 0.0),
                ("PGRAD", 1.422),
                ("RMF_MEAS", 0.2),
                ("RMF_TEMP", 25.0),
            ],
            &[("OPT_TU", "degC"), ("OPT_RMF", "ARPS")],
        );
        let out = precalc(&ctx);
        assert!((out["FTEMP"][0] - 85.0).abs() < 1e-3, "FTEMP {}", out["FTEMP"][0]);
        assert!((out["FTEMP_F"][0] - 185.0).abs() < 1e-3, "FTEMP_F {}", out["FTEMP_F"][0]);
        let expect = 0.2 * (77.0 + 6.77) / (185.0 + 6.77);
        assert!((out["RMF"][0] as f64 - expect).abs() < 1e-4, "RMF {}", out["RMF"][0]);
    }

    #[test]
    fn phimax_constant_caps_and_preserves_missing() {
        // CONSTANT ceiling 0.40, no TVDSS needed. Output names derive from __IN_PHI.
        let ctx = ctx_with(
            4,
            &[("PHI", vec![0.30, 0.45, f32::NAN, 0.40])],
            &[("PHIMAX0", 0.40), ("TVDSS_REF", 0.0), ("PHIMAX_GRAD", 0.03), ("ATHY_K", 0.10)],
            &[("MODE", "constant"), ("__IN_PHI", "PHIE")],
        );
        let out = phimax(&ctx);
        let cap = &out["PHI_CAP"];
        let mx = &out["PHI_MAX"];
        assert!((cap[0] - 0.30).abs() < 1e-6, "below ceiling unchanged: {}", cap[0]);
        assert!((cap[1] - 0.40).abs() < 1e-6, "above ceiling capped: {}", cap[1]);
        assert!(cap[2].is_nan(), "MISSING input stays MISSING");
        assert!((cap[3] - 0.40).abs() < 1e-6, "exactly at ceiling: {}", cap[3]);
        // Ceiling is a constant, emitted at every sample — even where the input is MISSING.
        for m in mx {
            assert!((m - 0.40).abs() < 1e-6, "constant ceiling everywhere: {m}");
        }
    }

    #[test]
    fn phimax_linear_trend_falls_with_depth() {
        // TVDSS_REF 5000, grad 0.05 per 1000 units. At 5000 → ceiling PHIMAX0=0.40;
        // at 6000 (1000 deeper) → 0.40 − 0.05 = 0.35. Positive-downward TVDSS.
        let ctx = ctx_with(
            2,
            &[("PHI", vec![0.50, 0.50]), ("TVDSS", vec![5000.0, 6000.0])],
            &[("PHIMAX0", 0.40), ("TVDSS_REF", 5000.0), ("PHIMAX_GRAD", 0.05), ("ATHY_K", 0.10)],
            &[("MODE", "linear"), ("__IN_PHI", "PHIE")],
        );
        let out = phimax(&ctx);
        assert!((out["PHI_MAX"][0] - 0.40).abs() < 1e-6, "at ref: {}", out["PHI_MAX"][0]);
        assert!((out["PHI_MAX"][1] - 0.35).abs() < 1e-6, "1000 deeper: {}", out["PHI_MAX"][1]);
        // PHI 0.50 caps to the ceiling at both depths.
        assert!((out["PHI_CAP"][0] - 0.40).abs() < 1e-6);
        assert!((out["PHI_CAP"][1] - 0.35).abs() < 1e-6);
    }

    #[test]
    fn phimax_athy_exponential_and_depth_fallback() {
        // No TVDSS curve → whole-curve fallback to DEPTH (MD). Athy: ceiling = 0.30·exp(−k·dz).
        // TVDSS_REF 0, k 0.5: at MD 0 → 0.30; at MD 1000 → 0.30·exp(−0.5) = 0.181959.
        let ctx = ctx_with(
            2,
            &[("PHI", vec![0.40, 0.40]), ("DEPTH", vec![0.0, 1000.0])],
            &[("PHIMAX0", 0.30), ("TVDSS_REF", 0.0), ("PHIMAX_GRAD", 0.03), ("ATHY_K", 0.5)],
            &[("MODE", "athy"), ("__IN_PHI", "PHIT")],
        );
        let out = phimax(&ctx);
        assert!((out["PHI_MAX"][0] - 0.30).abs() < 1e-6, "at ref: {}", out["PHI_MAX"][0]);
        let deep = 0.30 * (-0.5f64).exp();
        assert!((out["PHI_MAX"][1] as f64 - deep).abs() < 1e-6, "athy decay: {}", out["PHI_MAX"][1]);
        assert!((out["PHI_CAP"][0] - 0.30).abs() < 1e-6, "capped to ceiling at ref");
        assert!((out["PHI_CAP"][1] as f64 - deep).abs() < 1e-6, "capped to decayed ceiling");
    }

    #[test]
    fn phimax_clamps_ceiling_to_unit_range() {
        // A trend that drives the ceiling out of [0,1] is clamped (deliberate guard):
        // sample 0 — linear grad 1.0 per 1000, 30000 deep → raw −29.6 → clamped 0.0 → porosity forced to 0.
        // sample 1 — negative grad lifts the ceiling to 1.5 → clamped 1.0 → 0.80 passes through uncapped.
        let ctx = ctx_with(
            2,
            &[("PHI", vec![0.30, 0.80]), ("TVDSS", vec![30000.0, 1000.0])],
            &[("PHIMAX0", 0.40), ("TVDSS_REF", 0.0), ("PHIMAX_GRAD", 1.0), ("ATHY_K", 0.10)],
            &[("MODE", "linear"), ("__IN_PHI", "PHIE")],
        );
        let out = phimax(&ctx);
        assert!((out["PHI_MAX"][0] - 0.0).abs() < 1e-6, "sub-zero ceiling clamps to 0: {}", out["PHI_MAX"][0]);
        assert!((out["PHI_CAP"][0] - 0.0).abs() < 1e-6, "porosity forced to 0 below crossover");
        // Re-run sample 1 in a config where the ceiling exceeds 1 (negative gradient).
        let ctx2 = ctx_with(
            1,
            &[("PHI", vec![0.80]), ("TVDSS", vec![1000.0])],
            &[("PHIMAX0", 0.50), ("TVDSS_REF", 0.0), ("PHIMAX_GRAD", -1.0), ("ATHY_K", 0.10)],
            &[("MODE", "linear"), ("__IN_PHI", "PHIE")],
        );
        let out2 = phimax(&ctx2);
        assert!((out2["PHI_MAX"][0] - 1.0).abs() < 1e-6, "super-unit ceiling clamps to 1: {}", out2["PHI_MAX"][0]);
        assert!((out2["PHI_CAP"][0] - 0.80).abs() < 1e-6, "0.80 passes through under a 1.0 ceiling");
    }

    #[test]
    fn phimax_partial_nan_tvdss_passes_through_uncapped() {
        // A PARTIALLY finite TVDSS keeps the whole curve (fallback only fires when ALL samples are
        // non-finite): the NaN-depth sample gets a MISSING ceiling and its porosity passes through
        // uncapped — the documented per-sample semantics under the whole-curve fallback policy.
        let ctx = ctx_with(
            2,
            &[("PHI", vec![0.50, 0.50]), ("TVDSS", vec![5000.0, f32::NAN])],
            &[("PHIMAX0", 0.40), ("TVDSS_REF", 5000.0), ("PHIMAX_GRAD", 0.05), ("ATHY_K", 0.10)],
            &[("MODE", "linear"), ("__IN_PHI", "PHIE")],
        );
        let out = phimax(&ctx);
        assert!((out["PHI_MAX"][0] - 0.40).abs() < 1e-6, "finite-depth sample capped: {}", out["PHI_MAX"][0]);
        assert!((out["PHI_CAP"][0] - 0.40).abs() < 1e-6);
        assert!(out["PHI_MAX"][1].is_nan(), "NaN-depth sample → MISSING ceiling");
        assert!((out["PHI_CAP"][1] - 0.50).abs() < 1e-6, "NaN-depth porosity passes through uncapped");
    }

    #[test]
    fn phi_den_shale_branch_limits_and_missing() {
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("VSH_SHALE", 0.95),
        ];
        let phit_sh = 0.15 / 1.65; // (RHO_DSH-RHO_SH)/(RHO_DSH-RHO_W) = (2.65-2.5)/(2.65-1.0)

        // Happy path: RHOB 2.3, VSH 0.2 (unlimited PHIE below both caps → limited == unlimited).
        let out = phi_den(&ctx_with(1, &[("RHOB", vec![2.3]), ("VSH", vec![0.2])], &params, &[]));
        let pe = 0.345 / 1.645 - 0.2 * 0.145 / 1.645;
        assert!((out["PHIE_DEN"][0] as f64 - pe).abs() < 1e-5, "PHIE_DEN {}", out["PHIE_DEN"][0]);
        assert!((out["PHIT_DEN"][0] as f64 - (pe + 0.2 * phit_sh)).abs() < 1e-5, "PHIT_DEN");
        assert!((out["PHIE"][0] as f64 - pe).abs() < 1e-5, "unclamped below cap");

        // VSH ≥ 0.95 → shale: no effective porosity, PHIT = PHIT_SH. The LIMITED curve carries the
        // floor (finding 16) while the unlimited twin keeps the modelled hard zero.
        // (0.95_f32 rounds to 0.9499999_f64 which is just under the threshold, so use 0.96.)
        for v in [0.96f32, 1.0] {
            let out = phi_den(&ctx_with(1, &[("RHOB", vec![2.4]), ("VSH", vec![v])], &params, &[]));
            assert_eq!(out["PHIE"][0], PHIE_FLOOR as f32, "shale PHIE floored at VSH={v}");
            assert_eq!(out["PHIE_DEN"][0], 0.0);
            assert!((out["PHIT"][0] as f64 - phit_sh).abs() < 1e-6, "shale PHIT=PHIT_SH at VSH={v}");
            assert!((out["PHIT_DEN"][0] as f64 - phit_sh).abs() < 1e-6);
        }

        // OPT_PHIEMAX: a high-porosity sand (unlimited PHIE ≈ 0.435) clamps to phie_max·(1−VSH)=0.24
        // under SHALE_REDUCED (default) vs a flat phie_max=0.30 under MAXIMUM.
        let logs = [("RHOB", vec![1.9f32]), ("VSH", vec![0.2f32])];
        let red = phi_den(&ctx_with(1, &logs, &params, &[("OPT_PHIEMAX", "SHALE_REDUCED")]));
        let max = phi_den(&ctx_with(1, &logs, &params, &[("OPT_PHIEMAX", "MAXIMUM")]));
        assert!((red["PHIE"][0] - 0.24).abs() < 1e-5, "SHALE_REDUCED cap 0.24, got {}", red["PHIE"][0]);
        assert!((max["PHIE"][0] - 0.30).abs() < 1e-5, "MAXIMUM cap 0.30, got {}", max["PHIE"][0]);

        // Missing input propagates to all outputs.
        let out = phi_den(&ctx_with(1, &[("RHOB", vec![f32::NAN]), ("VSH", vec![0.2])], &params, &[]));
        assert!(out["PHIE"][0].is_nan() && out["PHIT_DEN"][0].is_nan());
    }

    /// SB-POR-043. `11_porosity.md:1044-1046` states exactly one MUST — the high-shale kill
    /// threshold is a CITED PARAMETER, not a literal — and §5:1229 mandates both the parameter and
    /// its value: *"a parameter in SandiBumi, defaulting to 0.95 with this source"*, tier T1,
    /// Geolog `phi_*.lls`. The step it produces is a discontinuity in PHIE "at a value the analyst
    /// cannot move", so the whole point of the row is that the analyst CAN now move it.
    ///
    /// Pinned from both sides deliberately. Arm A alone would pass an implementation that declares
    /// the argument and then goes on reading the literal — the arg would be inert, the dialog would
    /// show a number that changes nothing, and the discontinuity would still be unmovable. Arm C is
    /// what makes the parameter load-bearing.
    #[test]
    fn the_high_shale_kill_threshold_is_a_cited_parameter_the_analyst_can_move_and_never_a_literal() {
        let modules = module_catalog();
        let arg = |module: &str, argument: &str| -> Option<ArgSpec> {
            modules
                .iter()
                .find(|spec| spec.name == module)
                .unwrap_or_else(|| panic!("module '{module}' is not in the shipping catalog"))
                .args
                .iter()
                .find(|a| a.name == argument)
                .cloned()
        };

        // A — the threshold is a governed argument on both branch-carrying methods, defaulting to
        // the cited 0.95 and pointing at the competing-value topic.
        for module in ["phi_den", "phi_dn"] {
            let a = arg(module, "VSH_SHALE")
                .unwrap_or_else(|| panic!("{module} still hides the high-shale threshold"));
            assert_eq!(a.default, "0.95", "{module}.VSH_SHALE default");
            assert_eq!(
                a.sources_topic,
                crate::param_sources::HIGH_SHALE_BRANCH_THRESHOLD,
                "{module}.VSH_SHALE must disclose the three-way vendor disagreement"
            );
            assert!(
                a.default_source.contains("Geolog"),
                "{module}.VSH_SHALE default_source must name its source, got {:?}",
                a.default_source
            );
        }

        // B — `phi_son` has no high-shale branch at all (§3.5 / :682) and must not grow one just
        // because the parameter now exists.
        assert!(
            arg("phi_son", "VSH_SHALE").is_none(),
            "phi_son has no high-shale branch; adding the parameter would imply one"
        );

        // C — F21 (:488-494) is a genuine three-way disagreement, and all three positions ship.
        let claims = crate::param_sources::sources_for(
            crate::param_sources::HIGH_SHALE_BRANCH_THRESHOLD,
        );
        assert_eq!(claims.len(), 3, "F21 records Geolog, IP and Techlog");
        for (product, tier) in [("Geolog", "T1"), ("IP", "T2"), ("Techlog", "T3")] {
            let c = claims
                .iter()
                .find(|c| c.product == product)
                .unwrap_or_else(|| panic!("F21's {product} position is not disclosed"));
            assert_eq!(c.tier, tier, "{product} tier");
        }
        assert_eq!(
            claims.iter().find(|c| c.product == "Geolog").unwrap().value,
            "0.95"
        );
        assert!(
            claims.iter().any(|c| c.product == "IP" && c.value == "ABSENT"),
            "IP publishes NO numeric default for Vcl Limit — disclosing one would invent it"
        );

        // D — the literal is really gone: the same sample is killed at the default and computes a
        // real porosity once the analyst moves the threshold above it.
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("NPHI_SH", 0.35),
        ];
        let phit_sh = 0.15 / 1.65; // (RHO_DSH-RHO_SH)/(RHO_DSH-RHO_W)
        // `ctx_with` is a bare harness and does NOT materialise manifest defaults, so both arms
        // state the threshold explicitly. The 0.95 default itself is pinned by arm A, and the run
        // path applies it at `workflow.rs:280` / `:700` (user or zone value, else `arg.default`).
        let at_default: Vec<(&str, f64)> = params
            .iter()
            .copied()
            .chain([("VSH_SHALE", 0.95)])
            .collect();
        let moved: Vec<(&str, f64)> = params
            .iter()
            .copied()
            .chain([("VSH_SHALE", 0.99)])
            .collect();

        // phi_den: RHOB 2.4 at VSH 0.96. Killed at 0.95; at 0.99 the unlimited curve carries
        // 0.245/1.645 − 0.96·0.145/1.645 = 0.0643161.
        let logs = [("RHOB", vec![2.4f32]), ("VSH", vec![0.96f32])];
        let killed = phi_den(&ctx_with(1, &logs, &at_default, &[]));
        assert_eq!(killed["PHIE_DEN"][0], 0.0, "default 0.95 still kills VSH 0.96");
        let alive = phi_den(&ctx_with(1, &logs, &moved, &[]));
        assert!(
            (alive["PHIE_DEN"][0] as f64 - 0.0643161).abs() < 1e-5,
            "threshold moved to 0.99 -> the sample is a rock again, got {}",
            alive["PHIE_DEN"][0]
        );

        // phi_dn: the same move must reach the second branch site, not just the first.
        let logs = [
            ("RHOB", vec![2.4f32]),
            ("NPHI", vec![0.30f32]),
            ("VSH", vec![0.96f32]),
        ];
        let killed = phi_dn(&ctx_with(1, &logs, &at_default, &[]));
        assert_eq!(killed["PHIE_DN"][0], 0.0, "default 0.95 still kills VSH 0.96");
        assert!((killed["PHIT_DN"][0] as f64 - phit_sh).abs() < 1e-6, "shale sentinel");
        let alive = phi_dn(&ctx_with(1, &logs, &moved, &[]));
        assert!(
            (alive["PHIT_DN"][0] as f64 - phit_sh).abs() > 1e-6,
            "phi_dn still on the shale branch at VSH_SHALE 0.99 — its literal survived"
        );
    }

    /// SB-SAT-001 (P0). `12_saturation.md:867-890` — every saturation model is identified by a
    /// stable identifier naming its EQUATION, never a vendor's adjective for it, and no selector
    /// may offer a bare `Modified` / `Simandoux` / `Modified Simandoux`.
    ///
    /// The cost of getting this wrong is quantified in the chapter, which is why it is P0:
    /// "Modified" means Geolog's `Vsh·Sw` shale term in one product and IP's/Techlog's `(1−Vcl)`
    /// divisor in another, and selecting by adjective costs **7.3 saturation units and +19 % HCPV**.
    ///
    /// The row's as-built said `multimin2.rs:115,164` mislabel the Schlumberger form as
    /// Bardon-Pied; that is stale — both engines now carry equation ids. What was missing is
    /// anything keeping it that way, so this pins the contract rather than changing behaviour.
    #[test]
    fn every_saturation_model_is_named_by_its_equation_and_no_selector_offers_a_bare_vendor_adjective(
    ) {
        let modules = module_catalog();

        // A — the two Simandoux forms the chapter singles out are offered under equation ids, and
        // each label LEADS with its own id, so a vendor adjective can only ever trail it.
        let opt_sim = modules
            .iter()
            .find(|spec| spec.name == "sw_sim")
            .expect("sw_sim is a shipping module")
            .args
            .iter()
            .find(|a| a.name == "OPT_SIM")
            .expect("sw_sim exposes an equation selector")
            .clone();
        assert!(
            opt_sim.choices.iter().any(|value| value == "simandoux_bardon_pied"),
            "the Bardon-Pied equation must be offered by its equation id"
        );
        assert!(
            opt_sim.choices.iter().any(|value| value == "simandoux_modified_slb"),
            "the Schlumberger equation must be offered by its equation id"
        );
        for (value, label) in opt_sim.choices.iter().zip(opt_sim.choice_labels.iter()) {
            assert!(
                label.starts_with(value.as_str()),
                "a saturation label must lead with its equation id so a result and the selector \
                 match without translating an adjective: {value} -> {label}"
            );
        }

        // B — legacy vendor tokens still resolve, so saved chains keep running, and they resolve
        // the RIGHT way round. This mapping is the whole finding: MODIFIED is GEOLOG's name for
        // Bardon-Pied, not for the Schlumberger form. Swapping these two is the 7.3-saturation-unit
        // error, and it computes and plots either way.
        assert_eq!(
            canonical_option_value("sw_sim", "OPT_SIM", "MODIFIED"),
            "simandoux_bardon_pied",
            "Geolog's MODIFIED is Bardon-Pied — mapping it to the Schlumberger form is the defect"
        );
        assert_eq!(
            canonical_option_value("sw_sim", "OPT_SIM", "SCHLUMBERGER"),
            "simandoux_modified_slb"
        );
        // An already-canonical id passes through untouched, so re-running a new chain is stable.
        assert_eq!(
            canonical_option_value("sw_sim", "OPT_SIM", "simandoux_bardon_pied"),
            "simandoux_bardon_pied"
        );

        // C — universal, so a FUTURE module cannot reintroduce the ambiguity: no shipped option
        // anywhere offers a bare vendor adjective as its stored value.
        let banned = ["MODIFIED", "SIMANDOUX", "MODIFIED SIMANDOUX", "MODIFIED_SIMANDOUX"];
        for spec in modules {
            for arg in spec.args.iter().filter(|a| a.kind == ArgKind::Option) {
                for value in &arg.choices {
                    assert!(
                        !banned.contains(&value.to_ascii_uppercase().as_str()),
                        "{}.{} offers '{value}', a vendor adjective whose meaning changes between \
                         products — name the equation instead",
                        spec.name,
                        arg.name
                    );
                }
            }
        }

        // D — the solver engine agrees with the UI: every catalogue entry is an equation id and
        // its label leads with it, exactly as the selector does. Two engines, one vocabulary.
        for choice in crate::multimin2::sw_model_catalog() {
            assert!(
                choice.label.starts_with(choice.id),
                "solver model '{}' has a label that does not lead with its id: {}",
                choice.id,
                choice.label
            );
            assert!(
                !banned.contains(&choice.id.to_ascii_uppercase().as_str()),
                "solver model id '{}' is a vendor adjective",
                choice.id
            );
        }
    }

    /// SB-POR-056. `11_porosity.md:1118-1121` — porosity is carried internally in `v/v`, transit
    /// time in `µs/ft` and density in `g/cc`, with display units a presentation concern. Geolog
    /// ships `K/M3` and `US/M` internally (F22) and Techlog ships filtrate salinity in four
    /// unit/value combinations (F23); the canonical-unit rule is what keeps an import from either
    /// arriving **1000× out**.
    ///
    /// The row was recorded as depending on SB-POR-004's family typing. That closed, so what was
    /// left is the proof. The load-bearing link is arm B: a porosity OUTPUT is typed `POR` by
    /// `POROSITY_FAMILY_ID`, and `POR` resolves to `v/v` in the unit registry — without that bridge
    /// a computed `PHIE` has no canonical unit for the catalogue or LAS export to resolve, which is
    /// exactly what this row's as-built said was missing.
    #[test]
    fn porosity_transit_time_and_density_each_have_one_canonical_unit_and_a_thousandfold_delivery_is_converted_not_accepted(
    ) {
        use crate::curves::{canonical_unit, convert_to_canonical};

        // A — the three units the chapter names, stated where the product actually reads them.
        assert_eq!(canonical_unit("POR"), Some("v/v"), "porosity is a fraction");
        assert_eq!(canonical_unit("DT"), Some("us/ft"), "transit time");
        assert_eq!(canonical_unit("RHOB"), Some("g/cc"), "bulk density");
        assert_eq!(canonical_unit("NPHI"), Some("v/v"), "neutron is a fraction");

        // B — the bridge SB-POR-004 built: a computed porosity output is typed POR, and POR has a
        // canonical unit. Either half alone leaves an exported PHIE unresolvable.
        assert_eq!(
            canonical_unit(POROSITY_FAMILY_ID),
            Some("v/v"),
            "a computed porosity output must resolve to a canonical unit through its own family"
        );

        // C — the 1000× delivery Geolog ships is CONVERTED, not accepted verbatim. 2300 kg/m3 is
        // 2.3 g/cc; accepted as-is it would be a density no rock has.
        let mut rhob = [2300.0_f32];
        assert!(
            convert_to_canonical("RHOZ", "RHOB", Some("K/M3"), &mut rhob).is_some(),
            "a kg/m3 density must be converted, not passed through"
        );
        assert!((rhob[0] - 2.3).abs() < 1e-4, "kg/m3 -> g/cc, got {}", rhob[0]);

        // 328.084 µs/m × 0.3048 m/ft = 100 µs/ft.
        let mut dt = [328.084_f32];
        assert!(convert_to_canonical("DTCO", "DT", Some("US/M"), &mut dt).is_some());
        assert!((dt[0] - 100.0).abs() < 0.05, "us/m -> us/ft, got {}", dt[0]);

        // D — and the converted value reaches COMPUTE identically to a natively-canonical one.
        // This is the clause that makes the rule worth having: same rock, two deliveries, one
        // answer. Without it the conversion could be correct and still never be applied.
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("VSH_SHALE", 0.95),
        ];
        let logs_native = [("RHOB", vec![2.3f32]), ("VSH", vec![0.2f32])];
        let logs_converted = [("RHOB", vec![rhob[0]]), ("VSH", vec![0.2f32])];
        let native = phi_den(&ctx_with(1, &logs_native, &params, &[]));
        let converted = phi_den(&ctx_with(1, &logs_converted, &params, &[]));
        // Not bit-identical, and asserting that would be a false contract: 2300.0_f32 / 1000.0 is
        // not exactly 2.3_f32, so the two answers differ in the last ULP (measured 0.19209729 vs
        // 0.19209714). What the rule actually protects against is a unit-SCALE error, so the
        // tolerance is set well below any decimal shift and the ratio is pinned separately.
        let (a, b) = (native["PHIE_DEN"][0] as f64, converted["PHIE_DEN"][0] as f64);
        assert!(
            (a - b).abs() < 1e-6,
            "a g/cc delivery and a converted kg/m3 delivery of the same rock disagree by more than \
             float rounding: {a} vs {b}"
        );
        assert!(
            (a / b - 1.0).abs() < 1e-5,
            "the two deliveries differ by a SCALE factor, which is the 1000x/100x failure this rule \
             exists to catch: {a} vs {b}"
        );
        // And the answer is a fraction, not a percentage — the other 100× trap in the same family.
        assert!(
            native["PHIE_DEN"][0] > 0.0 && native["PHIE_DEN"][0] < 1.0,
            "porosity left compute outside v/v: {}",
            native["PHIE_DEN"][0]
        );
    }

    /// SB-POR-049. `11_porosity.md:1071-1073` forbids shipping a hard-coded lithology kill.
    /// Techlog's `φ_n > φ_d ∧ 2.91 ≤ ρ_b ≤ 3.5 ∧ φ_n ≤ 0.04 ⇒ φ = 0` is the only numeric kill any
    /// vendor publishes, and it zeroes real porosity in a tight carbonate with no flag and no
    /// parameter (F24).
    ///
    /// The row is a PROVE, not a fix: the inventory already found no such branch. What was missing
    /// was anything that keeps it that way, so this pins the absence from both directions — the
    /// behaviour today, and the literals that would reintroduce it.
    ///
    /// The distinction being defended is between *the arithmetic went negative* and *a rule zeroed
    /// it*. A tight carbonate genuinely has negative apparent density porosity, and that number is
    /// interpretable — an exact `0.0` is not, because it cannot be told apart from a kill. This is
    /// the same reasoning that keeps `PHIE_FLOOR` off zero.
    #[test]
    fn no_porosity_method_zeroes_a_tight_carbonate_on_a_hard_coded_lithology_rule() {
        // A — a sample squarely inside Techlog's published kill window. RHOB 2.95 is within
        // [2.91, 3.5]; NPHI 0.03 is below 0.04 and above the (negative) density porosity, so all
        // three of Techlog's conjuncts hold. SandiBumi must still compute.
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("NPHI_SH", 0.35),
            ("VSH_SHALE", 0.95),
        ];
        let logs = [
            ("RHOB", vec![2.95f32]),
            ("NPHI", vec![0.03f32]),
            ("VSH", vec![0.05f32]),
        ];

        // phi_den: (2.645−2.95)/1.645 − 0.05·(2.645−2.5)/1.645 = −0.1898176.
        let den = phi_den(&ctx_with(1, &logs, &params, &[]));
        assert!(
            (den["PHIE_DEN"][0] as f64 + 0.1898176).abs() < 1e-5,
            "density porosity must be the computed negative value, not a kill: {}",
            den["PHIE_DEN"][0]
        );
        assert_ne!(
            den["PHIE_DEN"][0], 0.0,
            "an exact zero here cannot be told apart from Techlog's lithology kill"
        );

        // phi_dn: the same window through the crossplot path.
        let dn = phi_dn(&ctx_with(1, &logs, &params, &[]));
        assert_ne!(
            dn["PHIE_DN"][0], 0.0,
            "an exact zero here cannot be told apart from Techlog's lithology kill"
        );
        assert!(
            (dn["PHIE_DN"][0] as f64) < 0.04,
            "a tight carbonate must not read as porous: {}",
            dn["PHIE_DN"][0]
        );

        // B — the registry half, so a FUTURE module cannot quietly reintroduce it. Production
        // source only: the scan is truncated at the test module, or this test's own statement of
        // Techlog's window would match itself.
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/modules.rs"))
            .expect("modules.rs is readable");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(before, _)| before)
            .unwrap_or(&source);
        // Deliberately a whole-file substring scan rather than a scoped one, and the trade-off is
        // stated so a future false positive is SCOPED rather than deleted: "2.91" also matches
        // 12.91 and 2.912, and a non-porosity module could one day need one legitimately. Erring
        // toward a spurious failure is the safe direction here — the alternative is a guard that
        // silently stops covering the thing it was written for. If this fires on innocent code,
        // narrow the slice to the porosity spec bodies; do not remove the assertion.
        assert!(
            !production.contains("2.91"),
            "the lower bound of Techlog's lithology-kill band (SB-POR-049) appeared in production \
             source. If this is an unrelated module's legitimate literal, narrow this scan to the \
             porosity bodies — do not delete it"
        );
        assert!(
            !production.contains("0.04 =>") && !production.contains("<= 0.04"),
            "a hard-coded neutron kill threshold (SB-POR-049) appeared in production source. Same \
             rule: narrow the scan, do not delete the assertion"
        );
    }

    /// A dense stringer must not hand a NEGATIVE porosity to anything downstream — and must still
    /// be visible as one to anybody asking whether the matrix density is right.
    ///
    /// A tight carbonate streak logged against a sandstone matrix reads RHOB above RHO_MA, so the
    /// density porosity comes out below zero. That is a routine artefact of the matrix choice, not
    /// a corrupt curve, and nothing downstream treated it as an error: the pay summary sums
    /// `PHIE·(1−SWE)·h`, so the streak's negative volume was SUBTRACTED from its zone's
    /// hydrocarbon column (`docs/review_triage.md` finding 16, Jauhar 2026-08-01: *"always limit
    /// phie to 0.001"*).
    ///
    /// The split is the point of the test. `PHIE` is floored, because everything downstream reads
    /// it as a physical volume. `PHIE_DEN` is the DECLARED unlimited twin and stays negative,
    /// because the excursion is the evidence for the judgement the curve exists to support —
    /// clamping both would hide the reason to go and check RHO_MA.
    #[test]
    fn a_negative_density_porosity_is_floored_but_stays_visible_in_the_unlimited_twin() {
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("VSH_SHALE", 0.95),
        ];
        // RHOB 2.75 on a 2.645 matrix, clean: pe = (2.645−2.75)/1.645 ≈ −0.0638.
        let out = phi_den(&ctx_with(1, &[("RHOB", vec![2.75]), ("VSH", vec![0.0])], &params, &[]));
        assert!(out["PHIE_DEN"][0] < 0.0, "the unlimited twin must keep the artefact visible");
        assert_eq!(out["PHIE"][0], PHIE_FLOOR as f32, "the limited curve is floored");
        assert!(out["PHIT"][0] >= 0.0, "and PHIT follows it rather than staying negative");

        // Same rock through the density-neutron route, which is the commoner one.
        let dn_params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0), ("NPHI_SH", 0.35),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("VSH_SHALE", 0.95),
        ];
        let dn = phi_dn(&ctx_with(
            1,
            &[("RHOB", vec![2.75]), ("NPHI", vec![-0.01]), ("VSH", vec![0.0])],
            &dn_params,
            &[],
        ));
        assert!(dn["PHIE_DN"][0] < 0.0, "unlimited twin negative here too");
        assert_eq!(dn["PHIE"][0], PHIE_FLOOR as f32, "and the limited curve floored the same way");

        // The floor must stay far below any porosity cutoff anyone would set, or it would stop a
        // stringer being subtracted by quietly promoting it into reservoir instead.
        assert!(PHIE_FLOOR < 0.01, "PHIE_FLOOR {PHIE_FLOOR} is too close to a real cutoff");
    }

    #[test]
    fn phi_dn_crossplot_shale_reduction_and_branches() {
        let params = [
            ("RHO_MA", 2.645), ("RHO_SH", 2.5), ("RHO_FL", 1.0), ("NPHI_SH", 0.35),
            ("RHO_DSH", 2.65), ("RHO_W", 1.0), ("PHIE_MAX", 0.3), ("VSH_SHALE", 0.95),
        ];
        let phit_sh = 0.15 / 1.65;

        // Happy path AVERAGE, shale-corrected: RHOB 2.3, NPHI 0.25, VSH 0.2.
        // rhosr=(2.3−0.5)/0.8=2.25, nphisr=(0.25−0.07)/0.8=0.225 (both in range).
        let out = phi_dn(
            &ctx_with(1, &[("RHOB", vec![2.3]), ("NPHI", vec![0.25]), ("VSH", vec![0.2])], &params, &[]),
        );
        let phid = (2.645 - 2.25) / 1.645;
        let pe = ((phid + 0.225) / 2.0) * 0.8;
        assert!((out["PHIE_DN"][0] as f64 - pe).abs() < 1e-5, "PHIE_DN {}", out["PHIE_DN"][0]);
        assert!((out["PHIT_DN"][0] as f64 - (pe + 0.2 * phit_sh)).abs() < 1e-5, "PHIT_DN");

        // VSH ≥ 0.95 → shale branch.
        let sh = phi_dn(
            &ctx_with(1, &[("RHOB", vec![2.4]), ("NPHI", vec![0.4]), ("VSH", vec![0.97])], &params, &[]),
        );
        // The limited curve carries the PHIE floor (finding 16); PHIT stays the shale's own total.
        assert_eq!(sh["PHIE"][0], PHIE_FLOOR as f32);
        assert_eq!(sh["PHIE_DN"][0], 0.0, "the unlimited twin keeps the modelled hard zero");
        assert!((sh["PHIT"][0] as f64 - phit_sh).abs() < 1e-6);

        // AVERAGE vs GAS_RMS diverge when PHID and NPHI differ (VSH 0 to isolate the combination).
        // RHOB 2.0, NPHI 0.10: phid=(2.645−2.0)/1.645, nphisr=0.10.
        let logs = [("RHOB", vec![2.0f32]), ("NPHI", vec![0.10f32]), ("VSH", vec![0.0f32])];
        let avg = phi_dn(&ctx_with(1, &logs, &params, &[("OPT_XPLOT", "AVERAGE")]));
        let rms = phi_dn(&ctx_with(1, &logs, &params, &[("OPT_XPLOT", "GAS_RMS")]));
        let phid = (2.645 - 2.0) / 1.645;
        let pe_avg = (phid + 0.10) / 2.0;
        let pe_rms = ((phid * phid + 0.10 * 0.10) / 2.0_f64).sqrt();
        assert!((avg["PHIE_DN"][0] as f64 - pe_avg).abs() < 1e-5, "AVERAGE {}", avg["PHIE_DN"][0]);
        assert!((rms["PHIE_DN"][0] as f64 - pe_rms).abs() < 1e-5, "GAS_RMS {}", rms["PHIE_DN"][0]);
        assert!(pe_rms > pe_avg + 0.02, "RMS must exceed AVERAGE when phid≠nphi");

        // Density shale-reduction LOWER clamp bites: RHOB 1.5, VSH 0.2 → (1.5−0.5)/0.8=1.25 clamps
        // up to 1.95, so PHIE_DN uses rhosr=1.95 (unclamped 1.25 would give a much larger value).
        let cl = phi_dn(
            &ctx_with(1, &[("RHOB", vec![1.5]), ("NPHI", vec![0.20]), ("VSH", vec![0.2])], &params, &[]),
        );
        let phid_c = (2.645 - 1.95) / 1.645;
        let nphisr_c = (0.20 - 0.2 * 0.35) / 0.8;
        let pe_c = ((phid_c + nphisr_c) / 2.0) * 0.8;
        assert!((cl["PHIE_DN"][0] as f64 - pe_c).abs() < 1e-5, "clamped rhosr=1.95: {}", cl["PHIE_DN"][0]);

        // Missing input propagates.
        let mn = phi_dn(
            &ctx_with(1, &[("RHOB", vec![2.3]), ("NPHI", vec![f32::NAN]), ("VSH", vec![0.2])], &params, &[]),
        );
        assert!(mn["PHIE"][0].is_nan() && mn["PHIT_DN"][0].is_nan());
    }

    #[test]
    fn vsh_gr_linear_and_limits() {
        let ctx = ctx_with(
            3,
            &[("GR", vec![20.0, 70.0, 150.0])],
            &[("GR_MA", 20.0), ("GR_SH", 120.0)],
            &[("OPT_GR", "LINEAR")],
        );
        let out = vsh_gr(&ctx);
        let vsh = &out["VSH"];
        assert!((vsh[0] - 0.0).abs() < 1e-5);
        assert!((vsh[1] - 0.5).abs() < 1e-5);
        assert!((vsh[2] - 1.0).abs() < 1e-5); // limited from 1.3
        assert!((out["VSH_GR"][2] - 1.3).abs() < 1e-5); // unlimited
    }

    /// The dropdown LABEL and the arithmetic must agree about which Larionov is which.
    ///
    /// The test above ties the code to the closed forms; this ties the closed forms to what the
    /// user is told. Between them the loop is closed, and it needs to be: the manual plan had the
    /// two rock-age attributions the wrong way round, and the dropdown was the only other place a
    /// user could learn which is which (`docs/review_triage.md` finding 21). A label claiming
    /// Tertiary above a coefficient set published for Mesozoic rock is the same defect moved one
    /// layer out — and just as invisible, because the curve looks entirely normal.
    #[test]
    fn the_vsh_gr_labels_agree_with_the_coefficients_they_describe() {
        let spec = list_modules().into_iter().find(|m| m.name == "vsh_gr").unwrap();
        let arg = spec.args.iter().find(|a| a.name == "OPT_GR").unwrap();
        assert_eq!(arg.choices.len(), arg.choice_labels.len(), "every choice needs a label, or none do");

        // Every label leads with its own id. The id is what `params_json` stores, so it is what a
        // user reading a saved run has in front of them — a label that replaced it would leave
        // them unable to match the two.
        for (id, l) in arg.choices.iter().zip(&arg.choice_labels) {
            assert!(l.starts_with(id), "{l} must lead with {id}");
        }
        let label = |id: &str| -> &str {
            let i = arg.choices.iter().position(|c| c == id).expect(id);
            arg.choice_labels[i].as_str()
        };

        // At mid-range IGR the older-rock set is the STEEPER one. That is the fact the two labels
        // encode, and getting it backwards is worth more than half again in shale volume.
        let v = 0.5f64;
        let older = 0.33 * (2.0f64.powf(2.0 * v) - 1.0); // 0.330
        let tertiary = 0.083 * (2.0f64.powf(3.7 * v) - 1.0); // 0.216
        assert!(older > tertiary, "sanity: {older} vs {tertiary}");
        assert!(label("LARINOV1").contains("Mesozoic and older"), "{}", label("LARINOV1"));
        assert!(label("LARINOV2").contains("Tertiary"), "{}", label("LARINOV2"));
        assert!(!label("LARINOV1").contains("Tertiary"), "both must not claim Tertiary");

        // LARINOV3 claims no rock age, because nothing in the repo cites a source for that form.
        // Inventing an attribution to make the dropdown look complete is the move the provenance
        // rules forbid — it would read exactly as authoritative as the two that are real.
        assert!(
            !label("LARINOV3").contains("Mesozoic") && !label("LARINOV3").contains("Tertiary"),
            "{}",
            label("LARINOV3")
        );
        assert!(label("LARINOV3").contains("0.127"), "it states its coefficient instead: {}", label("LARINOV3"));
    }

    /// T-PETRO-02. Every `OPT_GR` transform at the same mid-range gamma ray, against the
    /// published coefficient it implements. `vsh_gr_linear_and_limits` above covers LINEAR only,
    /// and a wrong nonlinear transform is the archetype of what this pile exists to catch: VSH
    /// comes out plausible at both endpoints and wrong through the whole shaly section, which is
    /// exactly where the net-pay cutoff sits.
    ///
    /// The expected values are the closed forms in `vsh_gr` (`modules.rs:337-355`), which carry
    /// the standard published coefficients and are inherited from Jauhar's own `vsh_gr.lls` —
    /// they are re-derived here by hand rather than copied from a run, so this is a check and
    /// not a snapshot.
    ///
    /// GR_MA 20 / GR_SH 120, so GR 70 is IGR = 0.5 exactly.
    #[test]
    fn every_vsh_gr_transform_lands_on_its_published_coefficient() {
        let vsh_at = |method: &str, gr: f32| -> (f32, f32) {
            let ctx = ctx_with(
                1,
                &[("GR", vec![gr])],
                &[("GR_MA", 20.0), ("GR_SH", 120.0)],
                &[("OPT_GR", method)],
            );
            let out = vsh_gr(&ctx);
            (out["VSH"][0], out["VSH_GR"][0])
        };

        // At IGR = 0.5. Each expectation is the transform evaluated by hand.
        let expected: [(&str, f64); 8] = [
            ("LINEAR", 0.5),
            // Stieber (1970) and its two variants: IGR / (k - (k-1)*IGR).
            ("STIEBER1", 0.5 / (3.0 - 2.0 * 0.5)),           // 0.250000
            ("STIEBER2", 0.5 / (2.0 - 0.5)),                 // 0.333333
            ("STIEBER3", 0.5 / (4.0 - 3.0 * 0.5)),           // 0.200000
            // Larionov (1969), OLDER rocks (Mesozoic and older): 0.33*(2^(2*IGR) - 1).
            ("LARINOV1", 0.33 * (2.0f64.powf(1.0) - 1.0)),   // 0.330000
            // Larionov (1969), TERTIARY / unconsolidated: 0.083*(2^(3.7*IGR) - 1).
            ("LARINOV2", 0.083 * (2.0f64.powf(1.85) - 1.0)), // 0.216248
            ("LARINOV3", 0.127 * (3.15f64.powf(1.0) - 1.0)), // 0.273050
            // Clavier (1971): 1.7 - sqrt(3.38 - (IGR + 0.7)^2).
            ("CLAVIER", 1.7 - (3.38f64 - 1.2f64.powi(2)).sqrt()), // 0.307161
        ];
        for (method, want) in expected {
            let (vsh, _) = vsh_at(method, 70.0);
            assert!(
                (vsh as f64 - want).abs() < 1e-5,
                "{method} at IGR 0.5 gave {vsh}, expected {want:.6}"
            );
        }

        // The domain claim the plan makes: every correction is concave, so at intermediate GR
        // every nonlinear form reads BELOW the linear one. A transform that came out above it
        // would be overstating shale in exactly the interval the cutoff decides.
        let linear = vsh_at("LINEAR", 70.0).0;
        for (method, _) in expected.iter().skip(1) {
            assert!(vsh_at(method, 70.0).0 < linear, "{method} must read below LINEAR at IGR 0.5");
        }

        // A clean matrix reads zero on every transform — including Clavier, where 1.7 - sqrt(2.89)
        // cancels exactly rather than approximately.
        for (method, _) in expected {
            let (vsh, raw) = vsh_at(method, 20.0);
            assert!(vsh.abs() < 1e-6, "{method} at IGR 0 gave {vsh}");
            assert!(raw.abs() < 1e-6, "{method} at IGR 0 gave raw {raw}");
        }

        // The top endpoint is NOT 1.0 for the Larionov forms, and that is the published
        // coefficients rather than a defect: they are empirical fits, never normalised to close
        // at pure shale. LARINOV1 stops at 0.99, LARINOV2 at 0.9957, and LARINOV3 OVERSHOOTS to
        // 1.133. VSH_GR keeps the raw number and VSH clamps it, which is the whole reason the
        // module emits both.
        for (method, want_raw) in [("LARINOV1", 0.99f64), ("LARINOV2", 0.995671), ("LARINOV3", 1.133155)] {
            let (vsh, raw) = vsh_at(method, 120.0);
            assert!((raw as f64 - want_raw).abs() < 1e-4, "{method} raw at IGR 1 gave {raw}");
            assert!((0.0..=1.0).contains(&vsh), "{method} limited VSH left 0..1: {vsh}");
        }
        for method in ["LINEAR", "STIEBER1", "STIEBER2", "STIEBER3", "CLAVIER"] {
            let (_, raw) = vsh_at(method, 120.0);
            assert!((raw - 1.0).abs() < 1e-5, "{method} does close at 1.0 at pure shale: {raw}");
        }

        // Past pure shale every limited VSH stays inside 0..1 while the raw curve runs on.
        for (method, _) in expected {
            let (vsh, raw) = vsh_at(method, 200.0);
            assert!((0.0..=1.0).contains(&vsh), "{method} limited VSH left 0..1 above GR_SH: {vsh}");
            assert!(raw > 1.0 || method == "LARINOV1", "{method} raw {raw}");
        }

        // Monotone in GR: more gamma ray is never less shale. A transform with a sign or
        // bracket slip can still hit the endpoints and fail only in between.
        for (method, _) in expected {
            let mut prev = f32::NEG_INFINITY;
            for gr in [20.0f32, 40.0, 60.0, 80.0, 100.0, 120.0] {
                let v = vsh_at(method, gr).1;
                assert!(v > prev - 1e-6, "{method} is not monotone at GR {gr}: {v} after {prev}");
                prev = v;
            }
        }
    }

    #[test]
    fn sw_arch_clean_sand() {
        // Classic Archie check: A=1, M=N=2, Rw=0.1, PHIT=0.25, RT=10 →
        // FF = 16, SWT = sqrt(16*0.1/10) = 0.4
        let ctx = ctx_with(
            1,
            &[("RT", vec![10.0]), ("PHIT", vec![0.25]), ("PHIE", vec![0.25])],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("SWT_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT")],
        );
        let out = sw_arch(&ctx);
        assert!((out["SWT"][0] - 0.4).abs() < 1e-4, "SWT was {}", out["SWT"][0]);
        assert!((out["SWE"][0] - 0.4).abs() < 1e-4);
    }

    #[test]
    fn sw_arch_zero_porosity_missing_phie_is_all_water_not_inf() {
        // PHIT=0 (coal/tight) with PHIE absent (NaN): the formation factor a/pt^m blows up.
        // The guard must key on pt==0 regardless of PHIE, so SWT_ARCH reads 1.0 (all water),
        // NOT +Infinity — otherwise the raw curve poisons catalog min/max and plot autoscale.
        let ctx = ctx_with(
            1,
            &[("RT", vec![10.0]), ("PHIT", vec![0.0]), ("PHIE", vec![f32::NAN])],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("SWT_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT")],
        );
        let out = sw_arch(&ctx);
        assert!(out["SWT_ARCH"][0].is_finite(), "SWT_ARCH must be finite, was {}", out["SWT_ARCH"][0]);
        assert_eq!(out["SWT_ARCH"][0], 1.0);
        assert_eq!(out["SWT"][0], 1.0);
    }

    #[test]
    fn sw_arch_nonpositive_rt_is_missing_not_inf() {
        // RT = 0 (a null coded as zero) makes the Archie ratio diverge to +Infinity; RT < 0
        // (bad processing) makes it NaN. Neither may leak into SWT_ARCH — a +inf there poisons
        // catalog min/max and plot autoscale. Both drop to missing, per the sw_rtc/sw_imts rule.
        let ctx = ctx_with(
            2,
            &[("RT", vec![0.0, -5.0]), ("PHIT", vec![0.25, 0.25]), ("PHIE", vec![0.25, 0.25])],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("SWT_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT")],
        );
        let out = sw_arch(&ctx);
        for k in ["SWT_ARCH", "SWT", "SWE", "VOL_UWAT"] {
            for i in 0..2 {
                assert!(!out[k][i].is_infinite(), "{k}[{i}] must never be +/-Infinity, was {}", out[k][i]);
                assert!(out[k][i].is_nan(), "{k}[{i}] must be missing (NaN), was {}", out[k][i]);
            }
        }
    }

    /// SB-SAT-047 (P0). `12_saturation.md:1832-1846` — a named saturation model **MUST** return the
    /// same value from the deterministic module and from the mineral solver, given the same inputs
    /// and parameters, to a stated tolerance.
    ///
    /// It is P0 because the product was failing it in the most expensive possible way: the two
    /// engines computed **different Simandoux equations under the same name**, 7.3 saturation units
    /// apart. SB-SAT-001 fixed the naming this session; this fixes the thing naming alone cannot —
    /// that the two engines actually agree on the number.
    ///
    /// **Tolerance is stated, not implied:** 1e-6 in saturation units. Both engines solve the same
    /// closed forms in f64 here, so anything looser would hide a real divergence, and demanding bit
    /// equality would fail on ordering alone.
    #[test]
    fn a_named_saturation_model_returns_one_number_from_either_engine() {
        const TOL: f64 = 1e-6;
        let (a, m, n_exp, rw, rsh) = (1.0_f64, 2.0_f64, 2.0_f64, 0.05_f64, 4.0_f64);
        let (rt, phie, vsh) = (8.0_f64, 0.22_f64, 0.25_f64);
        let params = [
            ("A", a), ("M", m), ("N", n_exp), ("RW", rw), ("RT_SH", rsh),
            ("C", 1.0), ("SWE_IRR", 0.0), ("SWT_IRR", 0.0),
        ];
        let logs = [
            ("RT", vec![rt as f32]),
            ("PHIT", vec![phie as f32]),
            ("PHIE", vec![phie as f32]),
            ("VSH", vec![vsh as f32]),
        ];

        // A — Archie. The module's unclipped diagnostic is the comparable curve; the clipped one
        // carries irreducible-saturation bounds the solver form does not know about.
        let arch = sw_arch(&ctx_with(1, &logs, &params, &[("OPT_RW", "CONSTANT")]));
        let solver_arch = crate::multimin2::sw_archie(rt, phie, rw, m, n_exp, a);
        assert!(
            (arch["SWT_ARCH"][0] as f64 - solver_arch).abs() < TOL,
            "archie disagrees between engines: module {} vs solver {solver_arch}",
            arch["SWT_ARCH"][0]
        );

        // B — both Simandoux forms, which is where the 7.3 su divergence lived. Each module branch
        // must match its OWN solver counterpart, and the test names which is which.
        for (option, solver) in [
            (
                "simandoux_bardon_pied",
                crate::multimin2::sw_simandoux_bardon_pied(rt, phie, vsh, rw, rsh, m, n_exp, a),
            ),
            (
                "simandoux_modified_slb",
                crate::multimin2::sw_simandoux_modified_slb(rt, phie, vsh, rw, rsh, m, n_exp, a, 1.0),
            ),
        ] {
            let out = sw_sim(&ctx_with(
                1,
                &logs,
                &params,
                &[("OPT_RW", "CONSTANT"), ("OPT_SIM", option)],
            ));
            assert!(
                (out["SWE_SIM"][0] as f64 - solver).abs() < TOL,
                "{option} disagrees between engines: module {} vs solver {solver}",
                out["SWE_SIM"][0]
            );
        }

        // C — and the two Simandoux forms are genuinely different numbers on this sample. Without
        // this arm, arm B would still pass if BOTH engines had collapsed onto one equation — which
        // is precisely the failure that made this row P0, just relocated.
        let bp = crate::multimin2::sw_simandoux_bardon_pied(rt, phie, vsh, rw, rsh, m, n_exp, a);
        let slb = crate::multimin2::sw_simandoux_modified_slb(rt, phie, vsh, rw, rsh, m, n_exp, a, 1.0);
        assert!(
            (bp - slb).abs() > 1e-3,
            "the two Simandoux forms returned the same number ({bp} vs {slb}); agreement between \
             engines means nothing if the engines have collapsed the two equations into one"
        );
    }

    /// SB-SAT-038 (P0). `12_saturation.md:1522-1537` — every saturation parameter **MUST** resolve
    /// to either a value with a **non-empty, checkable** source string or the explicit `NoDefault`
    /// state, and a default with an empty source **MUST fail the build**. A checkable reference is
    /// a file and section, a module and parameter name, or a full literature citation — a product
    /// name alone is not one.
    ///
    /// The domain's own evidence is the argument: three vendors ship three `Rw` defaults, three `B`
    /// method defaults, two `vQ0` values from the same paper, and a Simandoux `a` no cited paper
    /// supports — and none of them tells the user. A plausible-but-wrong endpoint computes, plots,
    /// and ships into a reserves number without failing.
    ///
    /// The as-built said no parameter carries a source and `ArgSpec` has no field for one. Both are
    /// stale — `default_source` exists and `validate_parameter_sources` already gates every module.
    /// What was missing is that the **checkable-artefact** rule was scoped to `VSH` alone; this
    /// increment extends it to `Saturation`, which the whole shipping catalogue already satisfies.
    #[test]
    fn a_saturation_default_without_a_checkable_source_fails_the_build() {
        // A — the shipping catalogue satisfies the stricter rule today.
        validate_parameter_sources(&module_catalog())
            .expect("every shipping saturation parameter must carry a checkable source or be ABSENT");

        let saturation_param = |source: &str, default: f64| ModuleSpec {
            name: "sw_probe".into(),
            title: "probe".into(),
            category: "Saturation".into(),
            doc: String::new(),
            args: vec![param("PROBE", "probe", "", default, 0.0, 10.0, source)],
        };

        // B — a product name alone is refused. This is the clause that makes the rule bite: it is
        // exactly how an uncited vendor default looks when someone writes it down in good faith.
        let bare_vendor = validate_parameter_sources(&[saturation_param("Geolog", 2.0)]);
        assert!(
            bare_vendor.is_err(),
            "a bare product name passed as a source for a saturation default"
        );
        assert!(
            format!("{bare_vendor:?}").contains("checkable"),
            "the refusal must say WHY, so the fix is obvious: {bare_vendor:?}"
        );

        // C — an empty source is refused even though the number looks ordinary.
        assert!(
            validate_parameter_sources(&[saturation_param("", 2.0)]).is_err(),
            "a saturation default with no source at all passed the build gate"
        );

        // D — and a properly cited one is accepted, or the rule would just block everything and
        // teach the next author to route around it.
        validate_parameter_sources(&[saturation_param(
            "docs/PRD_v2/12_saturation.md §5 formation-water parameters",
            2.0,
        )])
        .expect("a file-and-section citation is a checkable source");
    }

    /// SB-SAT-034 (P0). `12_saturation.md:1470-1487` — `a`, `m`, `n` and the Waxman-Smits /
    /// dual-water `m*`, `n*` **MUST** ship as `NoDefault`, a first-class state distinct from any
    /// numeric value, and a run requesting a saturation model without them **MUST** fail with a
    /// message naming the missing parameter.
    ///
    /// IP publishes **no default for a/m/n at all** — the 1.0/2.0/2.0 commonly quoted are Basic
    /// Log Analysis values only. A cementation exponent is a rock property measured on core, and
    /// the chapter is blunt about the stake: **a shipped exponent is the highest-consequence
    /// silent default in petrophysics.**
    ///
    /// Every module already complied; the survivor was the solver defaulting `archie_a` to 1.0,
    /// which served Indonesia and Simandoux where `a` is a free parameter, not only the
    /// dual-water forms where `a = 1` is physical.
    #[test]
    fn no_saturation_engine_ships_a_cementation_or_tortuosity_exponent_the_user_did_not_supply() {
        // A - no module ships a numeric default for any of the five.
        let mut checked = 0;
        for spec in module_catalog().iter().filter(|s| s.category == "Saturation") {
            for arg in spec.args.iter().filter(|a| {
                matches!(a.name.as_str(), "A" | "M" | "N" | "MSTAR" | "NSTAR")
            }) {
                checked += 1;
                assert_eq!(
                    arg.default, "",
                    "{}.{} ships exponent default {:?} - these come from core, not from us",
                    spec.name, arg.name, arg.default
                );
                assert_eq!(
                    arg.default_source, ABSENT_DEFAULT_SOURCE,
                    "{}.{} must DECLARE its absence, not merely leave the field blank",
                    spec.name, arg.name
                );
            }
        }
        assert!(
            checked >= 8,
            "expected these on several saturation modules, found {checked} - a pass would be vacuous"
        );

        // B - the solver refuses rather than defaulting, and NAMES what is missing. That is the
        // clause the chapter states explicitly, and a stronger guarantee than a blank field:
        // a caller cannot forget to set it.
        let payload = r#"{"rw":0.1,"rw_temp_f":77,"rmf":0.1,"rmf_temp_f":62,"ftemp_f":148,"m":2,"n":2,"mud_type":"WBM","rsh":4,"indonesia_k":1,"simandoux_c":1,"phit_sh":0.1,"ws_b":0}"#;
        let without_a = serde_json::from_str::<crate::multimin2::FluidProps>(payload);
        assert!(
            without_a.is_err(),
            "the solver accepted a fluid model with no tortuosity factor - it must refuse"
        );
        assert!(
            format!("{:?}", without_a.unwrap_err()).contains("archie_a"),
            "the refusal must NAME the missing parameter, or the user cannot act on it"
        );
    }

    /// SB-SAT-031 (P0). `12_saturation.md:1442-1456` — `Rw` **MUST** ship as `NoDefault` in every
    /// saturation module and in the solver. SandiBumi **MUST NOT** inherit IP's `0.1` or Techlog's
    /// `0.03`, and **MUST NOT** substitute a value derived from a formation-water environment band.
    ///
    /// IP's 0.1 and Techlog's 0.03 differ by **1.83× on Sw** at m = n = 2. The dossier explicitly
    /// **withdrew** a project-kb `Rw ≈ 0.21` as unsound corroboration, so no default rests on it
    /// either.
    ///
    /// The as-built said `modules.rs` ships 0.1 and `lrlc.rs` ships 0.3, leaving the two engines
    /// `√3 = 1.73×` apart before the user touches anything. Both are stale — every site now uses
    /// the `param_open` family. The row was a PROVE, and what it pins is that no future edit can
    /// quietly reintroduce a number here.
    #[test]
    fn no_saturation_engine_ships_a_formation_water_resistivity_the_user_did_not_supply() {
        let modules = module_catalog();

        // The values this domain must never inherit: IP's, Techlog's, the LRLC figure the as-built
        // reported, and the withdrawn project-kb corroboration.
        let rejected = ["0.1", "0.10", "0.03", "0.3", "0.30", "0.21"];

        let mut checked = 0;
        for spec in modules.iter().filter(|s| s.category == "Saturation") {
            for arg in spec.args.iter().filter(|a| a.name == "RW" || a.name == "RWS") {
                checked += 1;
                assert_eq!(
                    arg.default, "",
                    "{}.{} ships a formation-water resistivity default of {:?} — every Rw must be \
                     supplied by the user",
                    spec.name, arg.name, arg.default
                );
                assert_eq!(
                    arg.default_source, ABSENT_DEFAULT_SOURCE,
                    "{}.{} must declare its default ABSENT, not merely leave it blank",
                    spec.name, arg.name
                );
                assert!(
                    !rejected.contains(&arg.default.as_str()),
                    "{}.{} carries a rejected vendor value",
                    spec.name, arg.name
                );
            }
        }
        assert!(
            checked >= 3,
            "expected the saturation family to expose Rw on several modules, found {checked} — a \
             pass here would otherwise be vacuous"
        );

        // The solver has no default either, and it proves it by REFUSING to deserialize without
        // one. That is a stronger guarantee than a blank field: a caller cannot forget to set it.
        let missing_rw = serde_json::from_str::<crate::multimin2::FluidProps>("{}");
        assert!(
            missing_rw.is_err(),
            "the solver accepted a fluid model with no Rw — it must refuse rather than default"
        );
        assert!(
            format!("{:?}", missing_rw.unwrap_err()).contains("rw"),
            "the solver's refusal must name Rw so the user knows what to supply"
        );
    }

    /// SB-SAT-030. `12_saturation.md:1427-1440` — when `Vsh → 1` in `simandoux_modified_slb`
    /// (whose `1/(1−Vsh)` term is singular) or in `indonesia` (where water and effective porosity
    /// both go to zero), the run **MUST** raise a flagged condition. It **MAY** additionally
    /// return `Sw = 1`; it **MUST NOT** return `Sw = 1` unflagged.
    ///
    /// Techlog Elan is the only vendor documenting this failure mode. Returning a plausible number
    /// from a singular equation is the fail-silent pattern: on the log, a saturation clamped out of
    /// a `0/0` is indistinguishable from one the equation actually produced.
    ///
    /// The values are deliberately unchanged — all-water is permitted. What this pins is that the
    /// run says so.
    #[test]
    fn a_pure_shale_saturation_is_flagged_rather_than_quietly_returned_as_water() {
        let params = [
            ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.05),
            ("RT_SH", 4.0), ("C", 1.0), ("SWE_IRR", 0.0),
        ];
        // Each module gets the case the chapter documents for it. For
        // `simandoux_modified_slb` the singularity is in the `1/(1-VSH)` term and is
        // independent of porosity. For `indonesia` the chapter is explicit that water AND
        // effective porosity both go to zero — at VSH=1 with a healthy PHIE it returns a
        // COMPUTED value (measured: 0.373), so pairing VSH=1 with a real porosity would be
        // asserting a physically inconsistent sample, not the documented degeneracy.
        for (module, phie, opts) in [
            (("sw_sim"), 0.10f32, vec![("OPT_RW", "CONSTANT"), ("OPT_SIM", "simandoux_modified_slb")]),
            (("sw_indo"), 0.002f32, vec![("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL")]),
        ] {
            let logs = [
                ("RT", vec![8.0f32]),
                ("PHIE", vec![phie]),
                ("VSH", vec![1.0f32]),
            ];
            let ctx = ctx_with(1, &logs, &params, &opts);
            let (out, degradations, _, _) =
                run_module_with_degradations(module, &ctx, DefaultUsage::default())
                    .unwrap_or_else(|e| panic!("{module} failed to run: {e}"));

            // A — the condition IS raised. This is the whole requirement.
            assert!(
                degradations.iter().any(|d| d.kind == RunDegradationKind::Clamped
                    && d.detail.contains("VSH >= 1")),
                "{module} returned a pure-shale saturation with no flagged condition: {degradations:?}"
            );

            // B — and the answer is still all water, so the flag did not come at the cost of the
            // value the chapter permits. Asserting only arm A would pass a module that flagged and
            // then emitted something else entirely.
            let swe = out["SWE"][0];
            assert!(
                (swe - 1.0).abs() < 1e-6,
                "{module}: pure shale must still read all water, got {swe}"
            );
        }

        // C — the flag is specific to the singularity, not raised on every run. A clean sand must
        // come back unflagged, or the condition carries no information.
        let clean = ctx_with(
            1,
            &[("RT", vec![20.0f32]), ("PHIE", vec![0.25f32]), ("VSH", vec![0.10f32])],
            &params,
            &[("OPT_RW", "CONSTANT"), ("OPT_SIM", "simandoux_modified_slb")],
        );
        let (_, degradations, _, _) =
            run_module_with_degradations("sw_sim", &clean, DefaultUsage::default())
                .expect("clean sand runs");
        assert!(
            !degradations.iter().any(|d| d.detail.contains("VSH >= 1")),
            "a clean sand must not raise the pure-shale condition: {degradations:?}"
        );
    }

    /// SB-SAT-029. `12_saturation.md:1412-1425` — the documented guard rails, **including the
    /// volume detail**: `φe < 0.005 ⇒ all saturations 1` **and `VOL_UWAT = φe, not 0`**;
    /// `φe = φt = 0 ⇒ all saturations 1, all volumes 0`; `Rt` missing or ≤ 0 ⇒ every saturation
    /// output null.
    ///
    /// The volume detail is the one the chapter says bites (dossier MN-4): zeroing volumes there
    /// would silently zero bulk-volume water over tight streaks that still carry porosity. The
    /// interval is declared **wet**, not declared **empty** — and those are different answers that
    /// look identical in a summation.
    ///
    /// Rule 4 (variable-`m` guard) is vacuous by construction — no variable-`m` route exists — so
    /// it is deliberately not asserted rather than faked with a placeholder.
    #[test]
    fn every_standalone_saturation_guard_declares_a_tight_streak_wet_rather_than_empty() {
        let base = [
            ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.05),
            ("RT_SH", 4.0), ("SWT_IRR", 0.0), ("SWE_IRR", 0.0),
        ];
        let opts = [("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL"), ("OPT_SIM", "simandoux_bardon_pied")];

        // A — the volume detail, across every standalone saturation module. A tight streak with a
        // little porosity is ALL WATER, and its water volume is that porosity, never zero.
        let tight = 0.002_f32; // below the documented 0.005 rule
        for (name, run) in [
            ("sw_arch", sw_arch as fn(&ModuleContext) -> ModuleOutputs),
            ("sw_indo", sw_indo as fn(&ModuleContext) -> ModuleOutputs),
            ("sw_sim", sw_sim as fn(&ModuleContext) -> ModuleOutputs),
        ] {
            let out = run(&ctx_with(
                1,
                &[
                    ("RT", vec![8.0]),
                    ("PHIT", vec![0.05]),
                    ("PHIE", vec![tight]),
                    ("VSH", vec![0.3]),
                ],
                &base,
                &opts,
            ));
            assert_eq!(out["SWE"][0], 1.0, "{name}: a tight streak is all water");
            assert!(
                (out["VOL_UWAT"][0] - tight).abs() < 1e-6,
                "{name}: VOL_UWAT must be PHIE ({tight}), not 0 — zeroing it declares the streak \
                 EMPTY when it is merely WET, and the two are indistinguishable in a summation. \
                 Got {}",
                out["VOL_UWAT"][0]
            );
        }

        // B — the neighbouring rule must NOT be satisfied by the same shortcut: at zero porosity
        // there is genuinely nothing there, so the volume IS zero. If a module returned `phie`
        // unconditionally it would pass arm A and fail here.
        let out = sw_arch(&ctx_with(
            1,
            &[("RT", vec![8.0]), ("PHIT", vec![0.0]), ("PHIE", vec![0.0])],
            &base,
            &opts,
        ));
        assert_eq!(out["SWT"][0], 1.0, "zero porosity is all water");
        assert_eq!(out["SWE"][0], 1.0);
        assert_eq!(out["VOL_UWAT"][0], 0.0, "at zero porosity the water volume really is zero");

        // C — a non-physical resistivity nulls the saturation outputs rather than emitting an
        // infinity. RT <= 0 is typically a null coded as 0.
        for rt in [0.0_f32, -1.0, f32::NAN] {
            let out = sw_arch(&ctx_with(
                1,
                &[("RT", vec![rt]), ("PHIT", vec![0.20]), ("PHIE", vec![0.18])],
                &base,
                &opts,
            ));
            for curve in ["SWT", "SWE", "SWT_ARCH", "VOL_UWAT"] {
                assert!(
                    out[curve][0].is_nan(),
                    "sw_arch at RT={rt}: {curve} must be MISSING, was {}",
                    out[curve][0]
                );
            }
        }
    }

    /// SB-SAT-006. `12_saturation.md:908-920` — Indonesia is `v = Vsh^(2 − k·Vsh)` with
    /// `SWE = (1/(Rt·(1/(ff·Rw) + 2√(v/(Rw·ff·Rsh)) + v/Rsh)))^(1/n)`, `ff = a/φe^m`, exposing `k`
    /// with presets `FULL (k=1)`, `SIMPLE (k=0)` and `TAR_SAND/Woodhouse (k=2)`. **Both the
    /// deterministic module and the solver MUST use the same parameterised form.**
    ///
    /// The as-built said `multimin2.rs` hard-codes `k = 1` so the solver cannot run SIMPLE or
    /// TAR_SAND. That is stale: `multimin2.rs:277` reads `vsh.powf(1.0 - k * vsh / 2.0)` — the
    /// same family, spelled for the `1/√Rt` row, so squaring it returns `Vsh^(2 − k·Vsh)`. The row
    /// was a PROVE.
    ///
    /// Expectations are evaluated from the CHAPTER's equation with an explicit `k`, never read back
    /// from the module — that is what makes this a check of the named presets rather than a
    /// restatement of whatever the code happens to do.
    #[test]
    fn the_three_indonesia_presets_are_the_chapter_k_values_and_the_solver_shares_the_same_form() {
        let (a, m, n_exp, rw, rsh) = (1.0_f64, 2.0_f64, 2.0_f64, 0.1_f64, 5.0_f64);
        let (rt, phie, vsh) = (10.0_f64, 0.20_f64, 0.30_f64);

        // The chapter's equation, with k supplied rather than inferred.
        let chapter_swe = |k: f64| -> f64 {
            let v = vsh.powf(2.0 - k * vsh);
            let ff = a / phie.powf(m);
            let denom = 1.0 / (ff * rw) + 2.0 * (v / (rw * ff * rsh)).sqrt() + v / rsh;
            (1.0 / (rt * denom)).powf(1.0 / n_exp)
        };

        // A — each named preset IS its cited k. FULL=1, SIMPLE=0, TAR_SAND=2.
        for (variant, k) in [("FULL", 1.0), ("SIMPLE", 0.0), ("TAR_SAND", 2.0)] {
            let out = sw_indo(&ctx_with(
                1,
                &[("RT", vec![rt as f32]), ("PHIE", vec![phie as f32]), ("VSH", vec![vsh as f32])],
                &[("A", a), ("M", m), ("N", n_exp), ("RW", rw), ("RT_SH", rsh), ("SWE_IRR", 0.0)],
                &[("OPT_RW", "CONSTANT"), ("OPT_INDO", variant)],
            ));
            let expected = chapter_swe(k);
            assert!(
                (out["SWE_INDO"][0] as f64 - expected).abs() < 1e-5,
                "{variant} must be k={k} in Vsh^(2-k*Vsh): got {} expected {expected}",
                out["SWE_INDO"][0]
            );
        }

        // B — and the three are genuinely different answers, so arm A cannot be satisfied by a
        // module that ignores the option and returns one curve for all three.
        let (full, simple, tar) = (chapter_swe(1.0), chapter_swe(0.0), chapter_swe(2.0));
        assert!(
            (full - simple).abs() > 1e-3 && (full - tar).abs() > 1e-3 && (simple - tar).abs() > 1e-3,
            "the presets must separate: FULL {full} SIMPLE {simple} TAR_SAND {tar}"
        );

        // C — the solver shares the form. Its row is written for 1/√Rt, so its shale factor is
        // `Vsh^(1 - k·Vsh/2)`; squared, that is the module's `Vsh^(2 - k·Vsh)`. Pinning the
        // identity at every preset is what "the same parameterised form" means here.
        for k in [0.0_f64, 1.0, 2.0] {
            let solver_sq = vsh.powf(1.0 - k * vsh / 2.0).powi(2);
            let module_v = vsh.powf(2.0 - k * vsh);
            assert!(
                (solver_sq - module_v).abs() < 1e-12,
                "solver and module disagree on the Indonesia shale term at k={k}: \
                 {solver_sq} vs {module_v}"
            );
        }

        // D — an unconfigured run uses the cited FULL preset, so a module run and a solve
        // that were never configured cannot silently pick different variants. The solver's own
        // default is documented as k=1 at `multimin2.rs:523-525`; it is not asserted here because
        // reaching it means deserializing `FluidProps`, which has many required fields, and a test
        // that constructs a whole fluid model to read one default would break for reasons that have
        // nothing to do with this contract.
        let indo_default = module_catalog()
            .iter()
            .find(|spec| spec.name == "sw_indo")
            .expect("sw_indo is a shipping module")
            .args
            .iter()
            .find(|a| a.name == "OPT_INDO")
            .expect("sw_indo exposes the variant selector")
            .default
            .clone();
        assert_eq!(indo_default, "FULL", "the cited default preset is FULL (k=1)");
    }

    #[test]
    fn sw_indo_nonpositive_rt_is_missing_not_inf() {
        // 1/(RT*(...)) diverges to +Infinity at RT=0 — must not reach SWE_INDO.
        let ctx = ctx_with(
            1,
            &[("RT", vec![0.0]), ("PHIE", vec![0.2]), ("VSH", vec![0.3])],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("RT_SH", 5.0), ("SWE_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL")],
        );
        let out = sw_indo(&ctx);
        for k in ["SWE_INDO", "SWE", "VOL_UWAT"] {
            assert!(!out[k][0].is_infinite(), "{k} must never be +/-Infinity, was {}", out[k][0]);
            assert!(out[k][0].is_nan(), "{k} must be missing (NaN), was {}", out[k][0]);
        }
    }

    #[test]
    fn sw_sim_nonpositive_rt_is_missing_not_inf() {
        // g3 = -1/RT becomes -Infinity at RT=0 and the Newton-Raphson solve diverges; the
        // explicit guard drops the sample to missing instead of relying on that divergence.
        let ctx = ctx_with(
            1,
            &[("RT", vec![0.0]), ("PHIE", vec![0.22]), ("VSH", vec![0.25])],
            &[
                ("A", 1.0), ("M", 2.0), ("N", 2.0), ("C", 1.0),
                ("RW", 0.05), ("RT_SH", 4.0), ("SWE_IRR", 0.0),
            ],
            &[("OPT_RW", "CONSTANT"), ("OPT_SIM", "MODIFIED")],
        );
        let out = sw_sim(&ctx);
        for k in ["SWE_SIM", "SWE", "VOL_UWAT"] {
            assert!(!out[k][0].is_infinite(), "{k} must never be +/-Infinity, was {}", out[k][0]);
            assert!(out[k][0].is_nan(), "{k} must be missing (NaN), was {}", out[k][0]);
        }
    }

    #[test]
    fn sw_sim_schlumberger_pure_shale_is_all_water() {
        // SCHLUMBERGER g1 carries a 1/(1-VSH) term singular at VSH=1; pure shale must resolve
        // to all-water (SWE=1) instead of dividing by zero and silently dropping the sample.
        let ctx = ctx_with(
            1,
            &[("RT", vec![5.0]), ("PHIE", vec![0.2]), ("VSH", vec![1.0])],
            &[
                ("A", 1.0),
                ("M", 2.0),
                ("N", 2.0),
                ("RW", 0.1),
                ("C", 2.0),
                ("RT_SH", 4.0),
                ("SWE_IRR", 0.0),
            ],
            &[("OPT_RW", "CONSTANT"), ("OPT_SIM", "SCHLUMBERGER")],
        );
        let out = sw_sim(&ctx);
        assert!(out["SWE"][0].is_finite(), "SWE must be finite at VSH=1, was {}", out["SWE"][0]);
        assert_eq!(out["SWE"][0], 1.0);
    }

    #[test]
    fn sw_indo_full_vs_simple() {
        let logs: Vec<(&str, Vec<f32>)> =
            vec![("RT", vec![10.0]), ("PHIE", vec![0.2]), ("VSH", vec![0.3])];
        let params = [
            ("A", 1.0),
            ("M", 2.0),
            ("N", 2.0),
            ("RW", 0.1),
            ("RT_SH", 5.0),
            ("SWE_IRR", 0.0),
        ];
        let full = sw_indo(&ctx_with(1, &logs, &params, &[("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL")]));
        let simple = sw_indo(&ctx_with(1, &logs, &params, &[("OPT_RW", "CONSTANT"), ("OPT_INDO", "SIMPLE")]));
        let (sf, ss) = (full["SWE"][0], simple["SWE"][0]);
        assert!(sf > 0.0 && sf <= 1.0);
        assert!(ss > 0.0 && ss <= 1.0);
        // FULL uses VSH^(2-VSH) > VSH^2, so its shale conductivity term is larger → lower SW.
        assert!(sf < ss, "full={sf} simple={ss}");
    }

    #[test]
    fn sw_sim_matches_quadratic_solution() {
        // MODIFIED Simandoux with N=2 is a quadratic we can solve analytically.
        let (a, m, rw, rt, rt_sh, pe, vs): (f64, f64, f64, f64, f64, f64, f64) =
            (1.0, 2.0, 0.05, 8.0, 4.0, 0.22, 0.25);
        let g1: f64 = pe.powf(m) / (a * rw);
        let g2: f64 = vs / rt_sh;
        let g3: f64 = -1.0 / rt;
        let expected = (-g2 + (g2 * g2 - 4.0 * g1 * g3).sqrt()) / (2.0 * g1);

        let ctx = ctx_with(
            1,
            &[("RT", vec![rt as f32]), ("PHIE", vec![pe as f32]), ("VSH", vec![vs as f32])],
            &[("A", a), ("M", m), ("N", 2.0), ("RW", rw), ("RT_SH", rt_sh), ("SWE_IRR", 0.0), ("C", 1.0)],
            &[("OPT_RW", "CONSTANT"), ("OPT_SIM", "MODIFIED")],
        );
        let out = sw_sim(&ctx);
        assert!(
            (out["SWE_SIM"][0] as f64 - expected).abs() < 1e-4,
            "newton={} quadratic={}",
            out["SWE_SIM"][0],
            expected
        );
    }

    #[test]
    fn every_model_shared_by_the_module_and_solver_engines_returns_one_number_for_one_typed_fixture() {
        // CORRECTNESS — SB-CORE-T17 and docs/PRD_v2/12_saturation.md SB-SAT-T09/T30. Archie,
        // Indonesia k=0/1/2, and both typed Simandoux equations are implemented in both engines;
        // this independently evaluates each specified equation and requires both engines to meet it.
        let arch_ctx = ctx_with(
            1,
            &[("RT", vec![10.0]), ("PHIT", vec![0.25]), ("PHIE", vec![0.25])],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("SWT_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT")],
        );
        let arch_module = sw_arch(&arch_ctx)["SWT_ARCH"][0] as f64;
        let arch_solver = crate::multimin2::sw_archie(10.0, 0.25, 0.1, 2.0, 2.0, 1.0);
        assert!((arch_module - 0.4).abs() <= 1e-6, "archie_total module={arch_module}");
        assert!((arch_solver - 0.4).abs() <= 1e-12, "archie_total solver={arch_solver}");
        assert!((arch_module - arch_solver).abs() <= 1e-6);

        let (rt, phie, vsh, rw, rsh, a, m, n) = (8.0_f64, 0.20_f64, 0.30_f64, 0.25_f64, 3.0_f64, 1.0_f64, 2.0_f64, 2.0_f64);
        for (variant, k) in [("SIMPLE", 0.0_f64), ("FULL", 1.0_f64), ("TAR_SAND", 2.0_f64)] {
            let indo_ctx = ctx_with(
                1,
                &[("RT", vec![rt as f32]), ("PHIE", vec![phie as f32]), ("VSH", vec![vsh as f32])],
                &[("A", a), ("M", m), ("N", n), ("RW", rw), ("RT_SH", rsh), ("SWE_IRR", 0.0)],
                &[("OPT_RW", "CONSTANT"), ("OPT_INDO", variant)],
            );
            let module_sw = sw_indo(&indo_ctx)["SWE_INDO"][0] as f64;
            let solver_sw = crate::multimin2::sw_indonesia(rt, phie, vsh, rw, rsh, m, n, a, k);
            let v = vsh.powf(2.0 - k * vsh);
            let ff = a / phie.powf(m);
            let conductance = 1.0 / (ff * rw)
                + 2.0 * (v / (rw * ff * rsh)).sqrt()
                + v / rsh;
            let expected = (1.0 / (rt * conductance)).powf(1.0 / n);
            assert!((module_sw - expected).abs() <= 1e-6, "indonesia {variant} module={module_sw}, expected={expected}");
            assert!((solver_sw - expected).abs() <= 1e-12, "indonesia {variant} solver={solver_sw}, expected={expected}");
            assert!((module_sw - solver_sw).abs() <= 1e-6, "indonesia {variant}: module={module_sw}, solver={solver_sw}");
        }

        // docs/PRD_v2/12_saturation.md §2.2 cites 0.625 for Bardon-Pied and 0.5524 for
        // modified-SLB at this exact fixture (C=1).
        let sim_ctx = |method: &'static str| ctx_with(
            1,
            &[("RT", vec![rt as f32]), ("PHIE", vec![phie as f32]), ("VSH", vec![vsh as f32])],
            &[
                ("A", a),
                ("M", m),
                ("N", n),
                ("C", 1.0),
                ("RW", rw),
                ("RT_SH", rsh),
                ("SWE_IRR", 0.0),
            ],
            &[("OPT_RW", "CONSTANT"), ("OPT_SIM", method)],
        );
        let bardon_module = sw_sim(&sim_ctx("simandoux_bardon_pied"))["SWE_SIM"][0] as f64;
        let bardon_solver = crate::multimin2::sw_simandoux_bardon_pied(rt, phie, vsh, rw, rsh, m, n, a);
        assert!((bardon_module - 0.625).abs() < 5e-4, "Bardon-Pied module={bardon_module}");
        assert!((bardon_solver - 0.625).abs() < 5e-4, "Bardon-Pied solver={bardon_solver}");
        assert!((bardon_module - bardon_solver).abs() <= 1e-6);

        let slb_module = sw_sim(&sim_ctx("simandoux_modified_slb"))["SWE_SIM"][0] as f64;
        let slb_solver = crate::multimin2::sw_simandoux_modified_slb(rt, phie, vsh, rw, rsh, m, n, a, 1.0);
        assert!((slb_module - 0.5524).abs() < 1e-4, "modified-SLB module={slb_module}");
        assert!((slb_solver - 0.5524).abs() < 1e-4, "modified-SLB solver={slb_solver}");
        assert!((slb_module - slb_solver).abs() <= 1e-6);
    }

    #[test]
    fn every_saturation_method_surface_resolves_to_the_same_equation_identifier() {
        // CORRECTNESS — SB-CORE-T18 and docs/PRD_v2/12_saturation.md SB-SAT-001/T02. Bare vendor
        // adjectives are forbidden because Geolog and IP attach "Modified" to different equations.
        let spec = sw_sim_spec();
        let method = spec.args.iter().find(|arg| arg.name == "OPT_SIM").unwrap();
        assert_eq!(
            method.choices,
            ["simandoux_bardon_pied", "simandoux_modified_slb"],
            "the persisted choice is the equation id, never MODIFIED or SCHLUMBERGER"
        );
        assert_eq!(method.choices.len(), method.choice_labels.len());
        for (id, label) in method.choices.iter().zip(&method.choice_labels) {
            assert!(label.starts_with(id), "the UI label '{label}' must lead with its recorded id '{id}'");
            assert!(spec.doc.contains(id), "the module documentation must name {id}");
        }
        assert!(!method.choice_labels.iter().any(|label| {
            matches!(label.as_str(), "Modified" | "Simandoux" | "Modified Simandoux")
        }));

        let solver_catalog = crate::multimin2::sw_model_catalog();
        let mut flag_codes = std::collections::HashSet::new();
        for entry in &solver_catalog {
            assert!(
                flag_codes.insert(entry.flag_code.to_bits()),
                "two equation ids share method-flag code {}",
                entry.flag_code
            );
            assert_eq!(
                crate::multimin2::sw_model_id_from_flag(entry.flag_code),
                Some(entry.id),
                "flag code {} must resolve back to {}",
                entry.flag_code,
                entry.id
            );
        }
        for id in &method.choices {
            let solver = solver_catalog.iter().find(|entry| entry.id == id).unwrap_or_else(|| {
                panic!("the solver catalog has no entry for module equation id {id}")
            });
            assert!(solver.label.starts_with(id), "solver/UI label '{}' does not lead with {id}", solver.label);
        }
        assert_eq!(crate::multimin2::SwModel::SimandouxBardonPied.id(), "simandoux_bardon_pied");
        assert_eq!(crate::multimin2::SwModel::SimandouxModifiedSlb.id(), "simandoux_modified_slb");
        let legacy_solver: crate::multimin2::SwModel = serde_json::from_str("\"simandoux\"").unwrap();
        assert_eq!(legacy_solver.id(), "simandoux_modified_slb", "the old solver id keeps its old equation");
        let legacy_archie: crate::multimin2::SwModel = serde_json::from_str("\"archie\"").unwrap();
        assert_eq!(legacy_archie.id(), "archie_total", "the old Archie id keeps its total-porosity equation");

        for legacy in ["MODIFIED", "SIM_MOD"] {
            assert_eq!(canonical_option_value("sw_sim", "OPT_SIM", legacy), "simandoux_bardon_pied");
        }
        for legacy in ["SCHLUMBERGER", "SCHLUM", "SIM_SCHL"] {
            assert_eq!(canonical_option_value("sw_sim", "OPT_SIM", legacy), "simandoux_modified_slb");
        }

        let request = crate::workflow::RunModuleRequest {
            module: "sw_sim".into(),
            well_ids: vec![],
            log_inputs: HashMap::new(),
            params: HashMap::new(),
            opts: HashMap::from([("OPT_SIM".into(), "SCHLUM".into())]),
            output_set: None,
            input_set: None
        ,
            custody: crate::workflow::test_run_custody(),
        };
        let built = crate::workflow::build_opts(&spec, &request.opts, &request.log_inputs);
        assert_eq!(built["OPT_SIM"], "simandoux_modified_slb");
        let recorded: serde_json::Value = serde_json::from_str(
            &crate::workflow::recorded_module_params(&request, &spec, &built),
        )
        .unwrap();
        assert_eq!(recorded["method_id"], "simandoux_modified_slb");
        assert_eq!(recorded["OPT_SIM"], "simandoux_modified_slb");

        let flag_for = |id: &str| {
            solver_catalog
                .iter()
                .find(|entry| entry.id == id)
                .unwrap_or_else(|| panic!("no method-flag code for {id}"))
                .flag_code
        };
        let assert_module_flag = |module: &str, output: ModuleOutputs, saturation: &str, id: &str| {
            assert!(
                class_outputs(module).contains(&"SW_METHOD"),
                "{module} must declare its method flag as a class curve"
            );
            let values = output.get("SW_METHOD").unwrap_or_else(|| panic!("{module} emitted no SW_METHOD curve"));
            let saturation_values = &output[saturation];
            assert_eq!(values.len(), saturation_values.len());
            for (flag, sw) in values.iter().zip(saturation_values) {
                if sw.is_finite() {
                    assert_eq!(*flag, flag_for(id), "{module} finite sample does not identify {id}");
                    assert_eq!(crate::multimin2::sw_model_id_from_flag(*flag), Some(id));
                } else {
                    assert!(flag.is_nan(), "{module} must not claim a producer for a missing result");
                }
            }
        };

        let arch = ctx_with(
            2,
            &[
                ("RT", vec![10.0, f32::NAN]),
                ("PHIT", vec![0.25, 0.25]),
                ("PHIE", vec![0.25, 0.25]),
            ],
            &[("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1), ("SWT_IRR", 0.0)],
            &[("OPT_RW", "CONSTANT")],
        );
        assert_module_flag("sw_arch", sw_arch(&arch), "SWT_ARCH", "archie_total");

        let indo = ctx_with(
            2,
            &[
                ("RT", vec![8.0, f32::NAN]),
                ("PHIE", vec![0.20, 0.20]),
                ("VSH", vec![0.30, 0.30]),
            ],
            &[
                ("A", 1.0),
                ("M", 2.0),
                ("N", 2.0),
                ("RW", 0.25),
                ("RT_SH", 3.0),
                ("SWE_IRR", 0.0),
            ],
            &[("OPT_RW", "CONSTANT"), ("OPT_INDO", "FULL")],
        );
        assert_module_flag("sw_indo", sw_indo(&indo), "SWE_INDO", "indonesia");

        for id in ["simandoux_bardon_pied", "simandoux_modified_slb"] {
            let sim = ctx_with(
                2,
                &[
                    ("RT", vec![8.0, f32::NAN]),
                    ("PHIE", vec![0.20, 0.20]),
                    ("VSH", vec![0.30, 0.30]),
                ],
                &[
                    ("A", 1.0),
                    ("M", 2.0),
                    ("N", 2.0),
                    ("C", 1.0),
                    ("RW", 0.25),
                    ("RT_SH", 3.0),
                    ("SWE_IRR", 0.0),
                ],
                &[("OPT_RW", "CONSTANT"), ("OPT_SIM", id)],
            );
            assert_module_flag("sw_sim", sw_sim(&sim), "SWE_SIM", id);
        }

        let (sandimin_flag_name, sandimin_flags) = crate::multimin2::saturation_method_flag_curve(
            "MM",
            crate::multimin2::SwModel::SimandouxModifiedSlb,
            &[true, false, true],
        );
        assert_eq!(sandimin_flag_name, "MM_SW_METHOD");
        assert_eq!(sandimin_flags[0], flag_for("simandoux_modified_slb"));
        assert!(sandimin_flags[1].is_nan());
        assert_eq!(sandimin_flags[2], flag_for("simandoux_modified_slb"));
    }

    #[test]
    fn missing_propagates() {
        let ctx = ctx_with(
            2,
            &[("GR", vec![f32::NAN, 70.0])],
            &[("GR_MA", 20.0), ("GR_SH", 120.0)],
            &[("OPT_GR", "LINEAR")],
        );
        let out = vsh_gr(&ctx);
        assert!(out["VSH"][0].is_nan());
        assert!(!out["VSH"][1].is_nan());
    }

    #[test]
    fn depth_shift_resamples_onto_grid() {
        // Grid 1000..1010 step 1 m, value = 2·depth. Shift +2 m moves the feature deeper:
        // out(d) = value at (d − 2) = 2(d − 2); the top two samples fall before the data.
        let depths: Vec<f32> = (0..11).map(|i| 1000.0 + i as f32).collect();
        let vals: Vec<f32> = depths.iter().map(|d| 2.0 * d).collect();
        let ctx = ctx_with(
            11,
            &[("DEPTH", depths.clone()), ("CURVE", vals.clone())],
            &[("SHIFT", 2.0)],
            &[("__IN_CURVE", "GR")],
        );
        let out = depth_shift(&ctx);
        let s = &out["CURVE_DS"];
        assert!(s[0].is_nan() && s[1].is_nan(), "samples shifted in from above the log top must be missing");
        assert!((s[2] as f64 - 2000.0).abs() < 1e-3);
        assert!((s[10] as f64 - 2016.0).abs() < 1e-3);

        // Fractional shift interpolates linearly: at 1001 with +0.5 → value at 1000.5.
        let ctx_frac = ctx_with(
            11,
            &[("DEPTH", depths), ("CURVE", vals)],
            &[("SHIFT", 0.5)],
            &[("__IN_CURVE", "GR")],
        );
        let f = &depth_shift(&ctx_frac)["CURVE_DS"];
        assert!((f[1] as f64 - 2001.0).abs() < 1e-3);
    }

    #[test]
    fn splice_switches_at_depth() {
        let depths: Vec<f32> = (0..6).map(|i| 1000.0 + i as f32).collect();
        let ctx = ctx_with(
            6,
            &[("DEPTH", depths), ("TOP_CURVE", vec![1.0; 6]), ("BOT_CURVE", vec![2.0; 6])],
            &[("SPLICE_DEPTH", 1003.0)],
            &[("__IN_TOP_CURVE", "RES_RUN1")],
        );
        let out = splice(&ctx);
        let s = &out["SPLICED"];
        assert_eq!(s[2], 1.0, "above the splice depth the top curve wins");
        assert_eq!(s[3], 2.0, "at/below the splice depth the bottom curve wins");
    }

    /// T-PREP-18. `splice_switches_at_depth` above pins the handover itself. The plan promises
    /// something further: where the CONTRIBUTING run is missing, the output is missing — no fill
    /// from the other run.
    ///
    /// That is worth a test even though `splice` has no fallback branch to get wrong, because
    /// the tempting "helpful" rewrite is the wrong one. A gap in the top run above the splice
    /// depth is rock that run did not log. Reaching down to the bottom run to fill it would be a
    /// SECOND splice, at a depth the user never chose and cannot see on the log — and the joined
    /// curve would look continuous, which is precisely why nothing downstream would catch it.
    ///
    /// Four quadrants, because only two of them are load-bearing: a gap in the run that is
    /// contributing must survive, and a gap in the run that is NOT contributing must be
    /// irrelevant. Both directions are checked, and in both the other run holds a real value at
    /// that depth — a gap opposite a gap would prove nothing.
    #[test]
    fn a_gap_in_the_contributing_run_stays_a_gap() {
        // 1000..1005 at 1 m, splice at 1003 → indices 0,1,2 take TOP; 3,4,5 take BOT.
        let depths: Vec<f32> = (0..6).map(|i| 1000.0 + i as f32).collect();
        //                    1000  1001(hole)  1002  1003  1004  1005
        let top = vec![1.0, f32::NAN, 1.0, 1.0, 1.0, 1.0];
        let bot = vec![2.0, 2.0, f32::NAN, 2.0, f32::NAN, 2.0];
        let ctx = ctx_with(
            6,
            &[("DEPTH", depths), ("TOP_CURVE", top), ("BOT_CURVE", bot)],
            &[("SPLICE_DEPTH", 1003.0)],
            &[("__IN_TOP_CURVE", "RES_RUN1")],
        );
        let s = &splice(&ctx)["SPLICED"];

        assert!(
            s[1].is_nan(),
            "the top run has no value at 1001 and the bottom run's 2.0 must NOT be borrowed to fill it"
        );
        assert_eq!(s[2], 1.0, "the bottom run's gap at 1002 is above the splice — it contributes nothing there");
        assert!(
            s[4].is_nan(),
            "the bottom run has no value at 1004 and the top run's 1.0 must NOT be borrowed to fill it"
        );
        assert_eq!(s[0], 1.0);
        assert_eq!(s[3], 2.0, "the sample exactly ON the splice depth belongs to the bottom run");
        assert_eq!(s[5], 2.0);
    }

    /// A sample with no depth cannot be placed on either side of the splice, so it is missing —
    /// never assigned to a side by default. Also pins the fallback output name: with no resolved
    /// input mnemonic to build `<top>_SPL` from, the curve is plain `SPLICED` rather than a name
    /// with an empty prefix (`_SPL`), which would collide across runs.
    #[test]
    fn a_sample_with_no_depth_is_not_assigned_to_a_side() {
        let ctx = ctx_with(
            3,
            &[
                ("DEPTH", vec![1000.0, f32::NAN, 1010.0]),
                ("TOP_CURVE", vec![1.0, 1.0, 1.0]),
                ("BOT_CURVE", vec![2.0, 2.0, 2.0]),
            ],
            &[("SPLICE_DEPTH", 1005.0)],
            &[],
        );
        let out = splice(&ctx);
        let s = &out["SPLICED"];
        assert_eq!(s[0], 1.0);
        assert!(s[1].is_nan(), "a sample with no depth is on neither side of the splice");
        assert_eq!(s[2], 2.0);
    }

    #[test]
    fn badhole_flags_washout_and_drho() {
        // `20_envcorr-qc.md` section 5.2 cites 0.02 g/cc and 2 in; section 4.3 uses
        // a real 6 in slim-hole example. The other values only bracket those supplied inputs.
        let ctx = ctx_with(
            4,
            &[
                ("DRHO", vec![0.01, 0.03, 0.01, f32::NAN]),
                ("CALI", vec![6.2, 6.2, 9.0, f32::NAN]),
                ("BS", vec![6.0, 6.0, 6.0, f32::NAN]),
            ],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0)],
            &[
                ("__IN_DRHO", "DRHO"),
                ("__UNIT_DRHO", "g/cc"),
                ("DRHO_MAX_UNIT", "g/cc"),
            ],
        );
        let out = badhole(&ctx).expect("matching declared units must run");
        let f = &out["BADHOLE"];
        assert_eq!(f[0], 0.0, "good hole");
        assert_eq!(f[1], 1.0, "big DRHO");
        assert_eq!(f[2], 1.0, "washout");
        assert!(f[3].is_nan(), "no QC curves at all -> missing");
    }

    /// CORRECTNESS — `10_clay-volume.md` SB-CLY-035 / exact T36 supplies CALI = 6.0 in,
    /// BS = 8.5 in and DCAL_MAX = 1.0 in and requires the under-gauge side to fire. The
    /// over-gauge sample is the equal 2.5 in departure on the other side; 7.5/9.5 in are
    /// independently derived strict boundaries and 8.5 in is the zero-departure control.
    #[test]
    fn under_gauge_and_over_gauge_hole_both_fire_while_both_strict_boundaries_and_in_gauge_do_not() {
        let ctx = ctx_with(
            5,
            &[
                ("DRHO", vec![f32::NAN; 5]),
                ("CALI", vec![6.0, 7.5, 8.5, 9.5, 11.0]),
                ("BS", vec![8.5; 5]),
            ],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 1.0)],
            &[],
        );

        let out = badhole(&ctx).expect("a complete caliper discriminator must run");
        assert_eq!(
            out["BADHOLE"],
            [1.0, 0.0, 0.0, 0.0, 1.0],
            "equal-magnitude under- and over-gauge departures must be treated symmetrically"
        );
        assert_eq!(
            out["BADHOLE_CALI_EVALUATED"],
            [1.0; 5],
            "every finite caliper/bit-size pair was actually evaluated"
        );
    }

    /// CORRECTNESS — `20_envcorr-qc.md` section 4.3 SB-ENV-021 and section 6.3 T32,
    /// sourced to Geolog `badhole.lls:88-101`, require independent caliper/DRHO availability,
    /// an explicit record of which terms were evaluated, and MISSING rather than a false good-hole
    /// zero when neither is evaluable. The supplied 0.02 g/cc and 2 in thresholds are cited in
    /// section 5.2; every expected flag follows directly from those inequalities.
    #[test]
    fn a_bad_hole_flag_uses_each_available_term_records_which_was_evaluated_and_stays_missing_when_neither_was_evaluable() {
        let ctx = ctx_with(
            5,
            &[
                ("DRHO", vec![f32::NAN, 0.03, 0.01, f32::NAN, f32::NAN]),
                ("CALI", vec![9.0, f32::NAN, 6.2, f32::NAN, 6.2]),
                ("BS", vec![6.0, f32::NAN, 6.0, f32::NAN, 6.0]),
            ],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0)],
            &[
                ("__IN_DRHO", "DRHO"),
                ("__UNIT_DRHO", "g/cc"),
                ("DRHO_MAX_UNIT", "g/cc"),
            ],
        );

        let out = run_module("badhole", &ctx)
            .expect("each available criterion must run independently");
        assert_eq!(out["BADHOLE"][..3], [1.0, 1.0, 0.0]);
        assert!(out["BADHOLE"][3].is_nan(), "neither evaluable must remain MISSING");
        assert_eq!(out["BADHOLE"][4], 0.0, "one evaluated good criterion is a genuine zero");
        assert_eq!(
            out["BADHOLE_CALI_EVALUATED"],
            [1.0, 0.0, 1.0, 0.0, 1.0]
        );
        assert_eq!(
            out["BADHOLE_DRHO_EVALUATED"],
            [0.0, 1.0, 1.0, 0.0, 0.0]
        );
    }

    /// CORRECTNESS — `20_envcorr-qc.md` sections 4.3 SB-ENV-024, 5.2 and 6.3 T33.
    /// Both bad-hole thresholds are required to ship `ABSENT`; the valid-side values are the
    /// chapter's cited 0.02 g/cc tight-tolerance and 2 in delivered-study precedents. Presets are
    /// not offered here because section 7.2 ESC-1 leaves their shipped names and set unresolved.
    #[test]
    fn both_bad_hole_thresholds_ship_absent_and_each_must_be_explicitly_supplied_before_the_algorithm_can_run() {
        let spec = module_catalog()
            .iter()
            .find(|module| module.name == "badhole")
            .expect("badhole is registered");
        for name in ["DRHO_MAX", "DCAL_MAX"] {
            let threshold = spec
                .args
                .iter()
                .find(|arg| arg.name == name)
                .unwrap_or_else(|| panic!("{name} is declared"));
            assert!(threshold.required, "{name} must remain a required interpreter decision");
            assert_eq!(threshold.default_source, ABSENT_DEFAULT_SOURCE);
            assert!(
                threshold.default.is_empty(),
                "{name} must not conceal a numeric default behind ABSENT"
            );
        }

        let missing_drho_threshold = ctx_with(
            1,
            &[("DRHO", vec![0.03])],
            &[("DCAL_MAX", 2.0)],
            &[],
        );
        let error = run_module("badhole", &missing_drho_threshold)
            .expect_err("a missing DRHO threshold must refuse before computation");
        assert!(
            error.contains("DRHO_MAX") && error.contains("ABSENT"),
            "DRHO threshold refusal is not actionable: {error}"
        );

        let missing_caliper_threshold = ctx_with(
            1,
            &[("DRHO", vec![0.03])],
            &[("DRHO_MAX", 0.02)],
            &[],
        );
        let error = run_module("badhole", &missing_caliper_threshold)
            .expect_err("a missing differential-caliper threshold must refuse before computation");
        assert!(
            error.contains("DCAL_MAX") && error.contains("ABSENT"),
            "differential-caliper threshold refusal is not actionable: {error}"
        );

        // CALI and BS are absent, so no bit-size value enters this DRHO-only public control and no
        // uncited geometry fixture is needed.
        let explicit = ctx_with(
            2,
            &[("DRHO", vec![0.01, 0.03])],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0)],
            &[
                ("__IN_DRHO", "DRHO"),
                ("__UNIT_DRHO", "g/cc"),
                ("DRHO_MAX_UNIT", "g/cc"),
            ],
        );
        let output = run_module("badhole", &explicit)
            .expect("explicitly supplying both required thresholds must enable public dispatch");
        assert_eq!(
            output["BADHOLE"],
            [0.0, 1.0],
            "explicit cited thresholds must distinguish below-threshold and above-threshold samples"
        );
    }

    /// CORRECTNESS — `20_envcorr-qc.md` sections 4.3 SB-ENV-025, 5.2 and 6.3 T34.
    /// The 0.02 g/cc and 2 in thresholds are the chapter's cited selectable values; the 6 in bit
    /// size is its explicit slim-hole example. CALI 8/10 in are independently derived as one/two
    /// threshold-widths above 6 in, so the strict boundary and firing sides do not mirror the code.
    #[test]
    fn bit_size_has_no_default_and_missing_geometry_disables_only_caliper_while_curve_and_explicit_entry_remain_available() {
        let spec = module_catalog()
            .iter()
            .find(|module| module.name == "badhole")
            .expect("badhole is registered");
        assert!(
            spec.args.iter().all(|arg| arg.name != "BS_DEF"),
            "a fallback named BS_DEF is still a default source of invented geometry"
        );
        let explicit_bit_size = spec
            .args
            .iter()
            .find(|arg| arg.name == "BS_INPUT")
            .expect("an explicit user entry remains available when no BS curve exists");
        assert_eq!(explicit_bit_size.default_source, ABSENT_DEFAULT_SOURCE);
        assert!(explicit_bit_size.default.is_empty());
        assert!(!explicit_bit_size.required, "missing bit size must not block the DRHO term");
        assert_eq!(explicit_bit_size.min, None, "the chapter cites no physical lower bound");
        assert_eq!(explicit_bit_size.max, None, "the chapter cites no physical upper bound");

        let no_bit_size = ctx_with(
            3,
            &[
                ("DRHO", vec![0.03, 0.01, f32::NAN]),
                ("CALI", vec![6.2, 6.2, 6.2]),
            ],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0)],
            &[
                ("__IN_DRHO", "DRHO"),
                ("__UNIT_DRHO", "g/cc"),
                ("DRHO_MAX_UNIT", "g/cc"),
            ],
        );
        let output = run_module("badhole", &no_bit_size)
            .expect("missing geometry must degrade to the independently evaluable DRHO term");
        assert_eq!(output["BADHOLE"][..2], [1.0, 0.0]);
        assert!(output["BADHOLE"][2].is_nan(), "neither evaluable term must remain MISSING");
        assert_eq!(output["BADHOLE_CALI_EVALUATED"], [0.0, 0.0, 0.0]);
        assert_eq!(output["BADHOLE_DRHO_EVALUATED"], [1.0, 1.0, 0.0]);

        let from_curve = ctx_with(
            2,
            &[
                ("DRHO", vec![f32::NAN; 2]),
                ("CALI", vec![8.0, 10.0]),
                ("BS", vec![6.0, 6.0]),
            ],
            &[("DRHO_MAX", 0.02), ("DCAL_MAX", 2.0)],
            &[],
        );
        let output = run_module("badhole", &from_curve).expect("a measured BS curve must enable caliper QC");
        assert_eq!(output["BADHOLE"], [0.0, 1.0]);
        assert_eq!(output["BADHOLE_CALI_EVALUATED"], [1.0, 1.0]);

        let from_explicit_entry = ctx_with(
            2,
            &[
                ("DRHO", vec![f32::NAN; 2]),
                ("CALI", vec![8.0, 10.0]),
            ],
            &[
                ("DRHO_MAX", 0.02),
                ("DCAL_MAX", 2.0),
                ("BS_INPUT", 6.0),
            ],
            &[],
        );
        let output = run_module("badhole", &from_explicit_entry)
            .expect("an explicit interpreter entry must enable caliper QC without becoming a default");
        assert_eq!(output["BADHOLE"], [0.0, 1.0]);
        assert_eq!(output["BADHOLE_CALI_EVALUATED"], [1.0, 1.0]);
    }

    /// Full parameter set for condflag tests; individual tests override entries.
    fn condflag_params() -> Vec<(&'static str, f64)> {
        vec![
            ("RHO_MA", 2.645),
            ("RHO_FL", 1.0),
            ("COAL_RHOB", 1.9),
            ("COAL_NPHI", 0.35),
            ("COAL_DT", 100.0),
            ("TIGHT_PHI", 0.05),
            ("XOVER_MIN", 0.04),
            ("MIN_THICK", 0.0),
            ("SHOULDER", 0.0),
        ]
    }

    #[test]
    fn condflag_detects_coal_tight_and_crossover() {
        // Half-metre sampling; despike and shoulder disabled to isolate detection.
        let ctx = ctx_with(
            6,
            &[
                ("DEPTH", vec![1000.0, 1000.5, 1001.0, 1001.5, 1002.0, 1002.5]),
                //             shale   coal    fast-DT tight   gas     no-NPHI
                ("RHOB", vec![2.45, 1.40, 1.40, 2.62, 2.20, 2.30]),
                ("NPHI", vec![0.30, 0.45, 0.45, 0.02, 0.15, f32::NAN]),
                ("DT", vec![90.0, 120.0, 60.0, 55.0, 80.0, 85.0]),
            ],
            &condflag_params(),
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][1], 1.0, "light + hydrogen-rich + slow sonic = coal");
        assert_eq!(out["COAL_FLAG"][2], 0.0, "fast sonic vetoes the coal call");
        assert_eq!(out["TIGHT_FLAG"][3], 1.0, "DPHI 0.015 and NPHI 0.02 both under cutoff");
        assert_eq!(out["XOVER_FLAG"][4], 1.0, "DPHI 0.271 vs NPHI 0.15 = gas crossover");
        assert_eq!(out["XOVER_FLAG"][1], 0.0, "coal is not crossover");
        assert_eq!(out["XOVER_FLAG"][0], 0.0, "shale: NPHI over DPHI, no crossover");
        assert_eq!(out["COAL_FLAG"][0], 0.0);
        assert_eq!(out["TIGHT_FLAG"][0], 0.0);
        assert!(out["COAL_FLAG"][5].is_nan(), "missing NPHI -> flags missing");
        // Default OPT_XCOND=NO keeps gas crossover out of the combined mask.
        assert_eq!(out["COND_FLAG"][1], 1.0);
        assert_eq!(out["COND_FLAG"][4], 0.0, "crossover excluded from mask by default");
        assert!(out["COND_FLAG"][5].is_nan());
    }

    #[test]
    fn condflag_washout_is_not_coal_and_xcond_option() {
        // Sample 0 reads exactly like coal but sits in washed-out hole.
        let ctx = ctx_with(
            4,
            &[
                ("DEPTH", vec![1000.0, 1000.5, 1001.0, 1001.5]),
                ("RHOB", vec![1.40, 1.40, 2.20, 2.45]),
                ("NPHI", vec![0.45, 0.45, 0.15, 0.30]),
                ("DT", vec![120.0, 120.0, 80.0, 90.0]),
                ("BADHOLE", vec![1.0, 0.0, 0.0, f32::NAN]),
            ],
            &condflag_params(),
            &[("OPT_XCOND", "YES")],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][0], 0.0, "washout mimics coal -> not coal");
        assert_eq!(out["XOVER_FLAG"][0], 0.0, "washout mimics crossover -> not crossover");
        assert_eq!(out["COND_FLAG"][0], 1.0, "still masked, via the bad-hole flag");
        assert_eq!(out["COAL_FLAG"][1], 1.0, "same readings in gauge hole = coal");
        assert_eq!(out["COND_FLAG"][2], 1.0, "OPT_XCOND=YES pulls crossover into the mask");
        assert_eq!(out["COND_FLAG"][3], 0.0, "clean shale, missing BADHOLE -> good");
    }

    #[test]
    fn condflag_min_thick_drops_spikes() {
        let mut params = condflag_params();
        for p in &mut params {
            if p.0 == "MIN_THICK" {
                p.1 = 0.6;
            }
        }
        // One-sample coal spike (0.5 m counting sample spacing) vs a two-sample
        // bed (1.0 m) at half-metre sampling.
        let coal_rb = 1.4_f32;
        let coal_np = 0.45_f32;
        let ctx = ctx_with(
            9,
            &[
                ("DEPTH", (0..9).map(|i| 1000.0 + 0.5 * i as f32).collect()),
                ("RHOB", vec![2.45, 2.45, coal_rb, 2.45, 2.45, coal_rb, coal_rb, 2.45, 2.45]),
                ("NPHI", vec![0.30, 0.30, coal_np, 0.30, 0.30, coal_np, coal_np, 0.30, 0.30]),
            ],
            &params,
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][2], 0.0, "one-sample spike dropped by MIN_THICK");
        assert_eq!(out["COND_FLAG"][2], 0.0);
        assert_eq!(out["COAL_FLAG"][5], 1.0, "real two-sample bed survives");
        assert_eq!(out["COAL_FLAG"][6], 1.0);
    }

    #[test]
    fn condflag_shoulder_extends_past_bed_edges() {
        let mut params = condflag_params();
        for p in &mut params {
            if p.0 == "SHOULDER" {
                p.1 = 1.0;
            }
        }
        // Coal bed at samples 4-6; SHOULDER 1.0 at 0.5 m sampling reaches two
        // samples beyond each edge.
        let n = 11;
        let mut rb = vec![2.45_f32; n];
        let mut np = vec![0.30_f32; n];
        for i in 4..=6 {
            rb[i] = 1.4;
            np[i] = 0.45;
        }
        let ctx = ctx_with(
            n,
            &[
                ("DEPTH", (0..n).map(|i| 1000.0 + 0.5 * i as f32).collect()),
                ("RHOB", rb),
                ("NPHI", np),
            ],
            &params,
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        let sh = &out["SHOULDER_FLAG"];
        assert_eq!(&sh[..], &[0., 0., 1., 1., 0., 0., 0., 1., 1., 0., 0.]);
        for i in 2..=8 {
            assert_eq!(out["COND_FLAG"][i], 1.0, "bed + shoulders all masked (i={i})");
        }
        assert_eq!(out["COND_FLAG"][1], 0.0, "1.5 m from the bed edge is beyond SHOULDER");
        assert_eq!(out["COND_FLAG"][9], 0.0);
    }

    #[test]
    fn condflag_coal_without_sonic_and_missing_inputs() {
        // No DT curve at all -> coal decided on density/neutron alone.
        let ctx = ctx_with(
            3,
            &[
                ("DEPTH", vec![1000.0, 1000.5, 1001.0]),
                ("RHOB", vec![1.40, 2.45, f32::NAN]),
                ("NPHI", vec![0.45, 0.30, 0.30]),
            ],
            &condflag_params(),
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][0], 1.0, "no sonic -> two-criteria coal call");
        assert_eq!(out["COAL_FLAG"][1], 0.0);
        assert!(out["COAL_FLAG"][2].is_nan(), "missing RHOB -> flags missing");
        assert!(out["TIGHT_FLAG"][2].is_nan());
        assert!(out["XOVER_FLAG"][2].is_nan());
        assert!(out["COND_FLAG"][2].is_nan(), "nothing evaluable -> mask missing");
    }

    #[test]
    fn condflag_null_inside_bed_does_not_split_it() {
        let mut params = condflag_params();
        for p in &mut params {
            if p.0 == "MIN_THICK" {
                p.1 = 0.6;
            }
        }
        // 3-sample coal bed with a null NPHI in the middle: each fragment alone
        // (0.5 m) is thinner than MIN_THICK, but the bridged bed (1.5 m) survives.
        let ctx = ctx_with(
            7,
            &[
                ("DEPTH", (0..7).map(|i| 1000.0 + 0.5 * i as f32).collect()),
                ("RHOB", vec![2.45, 2.45, 1.40, 1.40, 1.40, 2.45, 2.45]),
                ("NPHI", vec![0.30, 0.30, 0.45, f32::NAN, 0.45, 0.30, 0.30]),
            ],
            &params,
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][2], 1.0, "bed fragment kept despite the null between");
        assert_eq!(out["COAL_FLAG"][4], 1.0);
        assert!(out["COAL_FLAG"][3].is_nan(), "the null sample itself stays missing");
    }

    #[test]
    fn condflag_badhole_blip_masks_itself_but_earns_no_shoulder() {
        let mut params = condflag_params();
        for p in &mut params {
            if p.0 == "MIN_THICK" {
                p.1 = 0.6;
            }
            if p.0 == "SHOULDER" {
                p.1 = 1.0;
            }
        }
        // Single-sample bad-hole blip at i2; real washout bed at i6-i8.
        let n = 12;
        let mut bh = vec![0.0_f32; n];
        bh[2] = 1.0;
        for i in 6..=8 {
            bh[i] = 1.0;
        }
        let ctx = ctx_with(
            n,
            &[
                ("DEPTH", (0..n).map(|i| 1000.0 + 0.5 * i as f32).collect()),
                ("RHOB", vec![2.45; n]),
                ("NPHI", vec![0.30; n]),
                ("BADHOLE", bh),
            ],
            &params,
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COND_FLAG"][2], 1.0, "the blip sample itself stays masked");
        assert_eq!(out["SHOULDER_FLAG"][1], 0.0, "no dilation around a one-sample blip");
        assert_eq!(out["SHOULDER_FLAG"][3], 0.0);
        assert_eq!(out["SHOULDER_FLAG"][5], 1.0, "the real washout bed dilates");
        assert_eq!(out["SHOULDER_FLAG"][4], 1.0);
        assert_eq!(out["SHOULDER_FLAG"][9], 1.0);
        assert_eq!(out["SHOULDER_FLAG"][10], 1.0);
        assert_eq!(out["SHOULDER_FLAG"][11], 0.0, "1.5 m out is beyond SHOULDER");
    }

    #[test]
    fn condflag_degenerate_density_params_skip_dphi_flags() {
        // Zone overrides bypass dialog validation, so RHO_MA <= RHO_FL can reach
        // the module: DPHI-based flags must go missing, never fire on +/-inf.
        let mut params = condflag_params();
        for p in &mut params {
            if p.0 == "RHO_MA" {
                p.1 = 1.0;
            }
            if p.0 == "RHO_FL" {
                p.1 = 1.2;
            }
        }
        let ctx = ctx_with(
            2,
            &[
                ("DEPTH", vec![1000.0, 1000.5]),
                ("RHOB", vec![1.40, 2.45]),
                ("NPHI", vec![0.45, 0.30]),
            ],
            &params,
            &[],
        );
        let out = condflag(&ctx).expect("condflag");
        assert_eq!(out["COAL_FLAG"][0], 1.0, "coal needs no DPHI and still works");
        assert!(out["TIGHT_FLAG"][1].is_nan(), "degenerate RHO_MA/RHO_FL -> no tight call");
        assert!(out["XOVER_FLAG"][1].is_nan(), "degenerate RHO_MA/RHO_FL -> no crossover call");
    }

    #[test]
    fn nphimat_tables_are_strictly_monotone() {
        // chart_lerp inverts the tables, so both coordinates must strictly
        // increase — this guards regressions in the generated neutron_charts.rs.
        use crate::neutron_charts as nc;
        for (name, t) in [
            ("CNL_NPHI_SS", nc::CNL_NPHI_SS),
            ("CNL_NPHI_DOL", nc::CNL_NPHI_DOL),
            ("CNL_TNPH_FRESH_SS", nc::CNL_TNPH_FRESH_SS),
            ("CNL_TNPH_FRESH_DOL", nc::CNL_TNPH_FRESH_DOL),
            ("CNL_TNPH_SALT_SS", nc::CNL_TNPH_SALT_SS),
            ("CNL_TNPH_SALT_DOL", nc::CNL_TNPH_SALT_DOL),
            ("APS_APLC_SS", nc::APS_APLC_SS),
            ("APS_APLC_DOL", nc::APS_APLC_DOL),
            ("APS_FPLC_SS", nc::APS_FPLC_SS),
            ("APS_FPLC_DOL", nc::APS_FPLC_DOL),
            ("SNP_SS", nc::SNP_SS),
            ("SNP_DOL", nc::SNP_DOL),
        ] {
            assert!(t.len() >= 40, "{name}: only {} points", t.len());
            for w in t.windows(2) {
                assert!(
                    w[1].0 > w[0].0 && w[1].1 > w[0].1,
                    "{name}: not strictly increasing at x={}",
                    w[1].0
                );
            }
        }
    }

    #[test]
    fn nphimat_reproduces_por5_worked_example() {
        // Printed on Por-5: quartz sandstone, TNPH = 18 pu apparent limestone,
        // formation salinity 250,000 ppm -> sandstone porosity 24 pu.
        let ctx = ctx_with(
            2,
            &[("NPHI", vec![0.18, f32::NAN])],
            &[],
            &[("TOOL", "TNPH"), ("SALINITY", "SALT_250K"), ("MATRIX_IN", "LS")],
        );
        let out = nphimat(&ctx);
        assert_eq!(out["NPHI_LS"][0], 0.18, "input convention passes through untouched");
        assert!((out["NPHI_SS"][0] - 0.24).abs() < 0.006, "book says 24 pu, got {}", out["NPHI_SS"][0]);
        assert!(out["NPHI_DOL"][0] < 0.18, "dolomite reads below limestone");
        for k in ["NPHI_LS", "NPHI_SS", "NPHI_DOL"] {
            assert!(out[k][1].is_nan(), "{k}: missing input stays missing");
        }
    }

    #[test]
    fn nphimat_round_trip_ss_back_to_ls() {
        // Inverse direction: a sandstone-unit log at the digitized SS reading for
        // 18 pu apparent limestone must come back to 0.18 on the limestone axis.
        let ctx = ctx_with(
            1,
            &[("NPHI", vec![0.2396])],
            &[],
            &[("TOOL", "TNPH"), ("SALINITY", "SALT_250K"), ("MATRIX_IN", "SS")],
        );
        let out = nphimat(&ctx);
        assert_eq!(out["NPHI_SS"][0], 0.2396, "input convention passes through untouched");
        assert!((out["NPHI_LS"][0] - 0.18).abs() < 1e-4, "got {}", out["NPHI_LS"][0]);
    }

    #[test]
    fn nphimat_thermal_dolomite_bow_and_salinity_scope() {
        // CNL ratio-method NPHI: the big thermal dolomite effect — 20 pu apparent
        // limestone reads ~11.8 pu in dolomite (digitized table nodes).
        let opts: &[(&str, &str)] = &[("TOOL", "NPHI"), ("SALINITY", "FRESH"), ("MATRIX_IN", "LS")];
        let ctx = ctx_with(1, &[("NPHI", vec![0.20])], &[], opts);
        let out = nphimat(&ctx);
        assert!((out["NPHI_DOL"][0] - 0.1181).abs() < 2e-4, "got {}", out["NPHI_DOL"][0]);
        assert!((out["NPHI_SS"][0] - 0.2460).abs() < 2e-4, "got {}", out["NPHI_SS"][0]);
        // SALINITY only selects between TNPH curve pairs — other tools ignore it.
        let ctx_salt = ctx_with(
            1,
            &[("NPHI", vec![0.20])],
            &[],
            &[("TOOL", "NPHI"), ("SALINITY", "SALT_250K"), ("MATRIX_IN", "LS")],
        );
        assert_eq!(nphimat(&ctx_salt)["NPHI_DOL"][0], out["NPHI_DOL"][0]);
    }

    #[test]
    fn nphimat_dolomite_input_inverts_through_the_chart() {
        let ctx = ctx_with(
            1,
            &[("NPHI", vec![0.05])],
            &[],
            &[("TOOL", "NPHI"), ("SALINITY", "FRESH"), ("MATRIX_IN", "DOL")],
        );
        let out = nphimat(&ctx);
        assert_eq!(out["NPHI_DOL"][0], 0.05, "input convention passes through untouched");
        // The recovered apparent limestone must land back on the dolomite curve...
        let back = chart_lerp(crate::neutron_charts::CNL_NPHI_DOL, out["NPHI_LS"][0] as f64, false);
        assert!((back - 0.05).abs() < 1e-6, "inverse then forward must close ({back})");
        // ...and sit above the dolomite value (dolomite reads low on the chart).
        assert!(out["NPHI_LS"][0] > 0.05 && out["NPHI_SS"][0] > out["NPHI_LS"][0]);
    }

    #[test]
    fn nphimat_extends_beyond_the_chart_span() {
        // 45 pu apparent limestone is past the digitized span — the conversion
        // continues on the end-segment slope instead of clamping or blanking.
        let ctx = ctx_with(
            1,
            &[("NPHI", vec![0.45])],
            &[],
            &[("TOOL", "TNPH"), ("SALINITY", "FRESH"), ("MATRIX_IN", "LS")],
        );
        let out = nphimat(&ctx);
        let t = crate::neutron_charts::CNL_TNPH_FRESH_SS;
        assert!(out["NPHI_SS"][0] > t[t.len() - 1].1, "extends past the last table node");
        assert!(out["NPHI_SS"][0] < 0.60, "with a sane end-segment slope");
    }

    #[test]
    fn env_corrections_move_the_right_way() {
        // GR: enlargement increases GR_EC; in-gauge (or no caliper) leaves it alone.
        let ctx = ctx_with(
            3,
            &[("GR", vec![100.0, 100.0, 100.0]), ("CALI", vec![8.5, 12.5, f32::NAN])],
            &[("K_GR", 0.0075), ("BS_DEF", 8.5)],
            &[],
        );
        let gr = gr_hole_corr(&ctx);
        assert_eq!(gr["GR_EC"][0], 100.0);
        assert!((gr["GR_EC"][1] - 103.0).abs() < 1e-3, "4 in enlargement -> +3%");
        assert_eq!(gr["GR_EC"][2], 100.0, "no caliper passes through");

        // RHOB: only beyond the reference diameter, and upward.
        let ctx = ctx_with(
            2,
            &[("RHOB", vec![2.30, 2.30]), ("CALI", vec![9.0, 14.0])],
            &[("K_RHO", 0.004), ("HD_REF", 10.0)],
            &[],
        );
        let rb = rhob_hole_corr(&ctx);
        assert_eq!(rb["RHOB_EC"][0], 2.30, "in gauge: unchanged");
        assert!((rb["RHOB_EC"][1] - 2.316).abs() < 1e-4, "4 in over reference -> +0.016");

        // NPHI: salinity term applies even without FTEMP; temperature term needs it.
        let ctx = ctx_with(
            1,
            &[("NPHI", vec![0.30]), ("FTEMP", vec![104.0])],
            &[("K_TEMP", 0.0001), ("T_REF", 24.0), ("K_SAL", -0.002), ("SALW", 100000.0)],
            &[],
        );
        let np = nphi_env_corr(&ctx);
        // 0.30 - 0.002*(100000/100000) + 0.0001*(104-24) = 0.30 - 0.002 + 0.008 = 0.306
        assert!((np["NPHI_EC"][0] - 0.306).abs() < 1e-4, "got {}", np["NPHI_EC"][0]);
    }

    /// CORRECTNESS — SB-ENV-006 / SB-ENV-T12, `docs/PRD_v2/20_envcorr-qc.md` section 6.2.
    /// The numbers below are synthetic non-zero algebra fixtures, never product defaults. The
    /// sourced expectation is relational: with a correction input missing, every registered
    /// `*_EC` producer either refuses or returns values that are not an unmarked input copy; with
    /// the complete fixture it remains runnable, so a blanket retirement cannot satisfy the test.
    #[test]
    fn every_ec_module_with_a_missing_correction_input_refuses_or_changes_the_curve_and_complete_inputs_still_run(
    ) {
        let ec_modules = list_modules()
            .into_iter()
            .filter(|module| {
                module.args.iter().any(|arg| {
                    arg.kind == ArgKind::LogOut && arg.name.ends_with("_EC")
                })
            })
            .map(|module| module.name)
            .collect::<Vec<_>>();
        assert!(!ec_modules.is_empty(), "the universal guard must exercise a real *_EC producer");

        for module in ec_modules {
            match module.as_str() {
                "gr_hole_corr" => {
                    let missing = ctx_with(
                        2,
                        &[("GR", vec![80.0, 90.0])],
                        &[("K_GR", 0.01), ("BS_DEF", 8.5)],
                        &[],
                    );
                    assert!(
                        run_module(&module, &missing).is_err(),
                        "{module} must not return an unmarked GR_EC copy without caliper"
                    );

                    let complete = ctx_with(
                        2,
                        &[("GR", vec![80.0, 90.0]), ("CALI", vec![10.5, 11.5])],
                        &[("K_GR", 0.01), ("BS_DEF", 8.5)],
                        &[],
                    );
                    let out = run_module(&module, &complete)
                        .expect("complete GR correction inputs must remain runnable");
                    assert_ne!(out["GR_EC"], complete.logs["GR"]);
                }
                "nphi_env_corr" => {
                    let salinity_only = ctx_with(
                        2,
                        &[("NPHI", vec![0.20, 0.25])],
                        &[
                            ("K_TEMP", 0.001),
                            ("T_REF", 25.0),
                            ("K_SAL", 0.01),
                            ("SALW", 100_000.0),
                        ],
                        &[],
                    );
                    let partial = run_module(&module, &salinity_only)
                        .expect("a non-zero salinity correction is not an uncorrected copy");
                    assert_ne!(partial["NPHI_EC"], salinity_only.logs["NPHI"]);

                    let complete = ctx_with(
                        2,
                        &[
                            ("NPHI", vec![0.20, 0.25]),
                            ("FTEMP", vec![80.0, 90.0]),
                        ],
                        &[
                            ("K_TEMP", 0.001),
                            ("T_REF", 25.0),
                            ("K_SAL", 0.01),
                            ("SALW", 100_000.0),
                        ],
                        &[],
                    );
                    let out = run_module(&module, &complete)
                        .expect("complete neutron correction inputs must remain runnable");
                    assert_ne!(out["NPHI_EC"], complete.logs["NPHI"]);
                }
                "rhob_hole_corr" => {
                    let missing = ctx_with(
                        2,
                        &[("RHOB", vec![2.30, 2.35])],
                        &[("K_RHO", 0.01), ("HD_REF", 8.5)],
                        &[],
                    );
                    assert!(
                        run_module(&module, &missing).is_err(),
                        "{module} must not return an unmarked RHOB_EC copy without caliper"
                    );

                    let complete = ctx_with(
                        2,
                        &[("RHOB", vec![2.30, 2.35]), ("CALI", vec![10.5, 11.5])],
                        &[("K_RHO", 0.01), ("HD_REF", 8.5)],
                        &[],
                    );
                    let out = run_module(&module, &complete)
                        .expect("complete density correction inputs must remain runnable");
                    assert_ne!(out["RHOB_EC"], complete.logs["RHOB"]);
                }
                unexpected => panic!(
                    "registered *_EC producer '{unexpected}' has no SB-ENV-T12 missing-input fixture"
                ),
            }
        }
    }

    #[test]
    fn gascorr_spec_shape() {
        // The spec swaps rw_args' optional FTEMP for a required one — no arg
        // name may appear twice or the dialog renders two conflicting rows.
        let spec = gascorr_spec();
        let mut seen = std::collections::HashSet::new();
        for a in &spec.args {
            assert!(seen.insert(a.name.clone()), "duplicate arg {}", a.name);
        }
        let ftemp = spec.args.iter().find(|a| a.name == "FTEMP").unwrap();
        assert!(matches!(ftemp.kind, ArgKind::LogIn) && ftemp.required);
        // A raw import named FTEMP/FPRESS may be degF/kPa — precalc outputs only.
        assert!(ftemp.computed_only, "FTEMP must not resolve from the RAW store");
        assert!(spec.args.iter().find(|a| a.name == "FPRESS").unwrap().computed_only);
        let flag = spec.args.iter().find(|a| a.name == "GAS_FLAG").unwrap();
        assert!(!flag.required, "gate flag is optional");
        let gate = spec.args.iter().find(|a| a.name == "OPT_GATE").unwrap();
        assert_eq!(gate.default, "FLAGGED", "safe default: EVERYWHERE overcorrects coal/washout");
        assert!(spec.args.iter().any(|a| a.name == "OPT_RW"), "rw_args merged in");
        // Manifest JSON for the frontend render check (visible with --nocapture).
        println!("GASCORR_SPEC_JSON {}", serde_json::to_string(&spec).unwrap());
    }

    #[test]
    fn gascorr_papay_gas_density_pinned() {
        // SG 0.65 at the KK example's reservoir conditions (2743.34 psi, 93.9 degC
        // = 5000 ft TVDSS on the KK trends): Standing Tpc 373.97 R / Ppc 670.91 psia,
        // Papay z ~0.899 -> rho_g ~0.1297 g/cc. Hand-computed independently.
        let rhog = gas_density_gcc(0.65, 2743.34, 93.9);
        assert!((rhog - 0.1297).abs() < 0.001, "rhog {}", rhog);
        // Low pressure -> near-ideal gas, much lighter.
        let light = gas_density_gcc(0.65, 500.0, 93.9);
        assert!((light - 0.0222).abs() < 0.001, "rhog {}", light);
        // Unusable P/T -> MISSING.
        assert!(gas_density_gcc(0.65, f64::NAN, 93.9).is_nan());
        assert!(gas_density_gcc(0.65, 2743.34, f64::NAN).is_nan());
        assert!(gas_density_gcc(0.65, -5.0, 93.9).is_nan());
    }

    #[test]
    fn gascorr_converges_on_gas_sand_and_skips_water() {
        // Forward-model a gas sand: true phit 0.30, Swt 0.40, gas at the KK
        // reservoir conditions. RT is chosen so Archie (A 1, M 2, N 2, Rw 0.1)
        // returns exactly Sw 0.40 at phit 0.30: RT = 0.1/(0.09*0.16) = 6.9444.
        let rhog = gas_density_gcc(0.65, 2743.34, 93.9);
        let rb_gas = 0.70 * 2.65 + 0.30 * 0.40 * 1.0 + 0.30 * 0.60 * rhog;
        // Water zone at the same porosity: rb = 2.155, RT low -> Archie Sw >= 1.
        let ctx = ctx_with(
            2,
            &[
                ("RHOB", vec![rb_gas as f32, 2.155]),
                ("RT", vec![6.9444, 1.0]),
                ("FTEMP", vec![93.9, 93.9]),
                ("FPRESS", vec![2743.34, 2743.34]),
            ],
            &[
                ("RHO_MA", 2.65),
                ("RHO_FL", 1.0),
                ("SG_GAS", 0.65),
                ("A", 1.0),
                ("M", 2.0),
                ("N", 2.0),
                ("RW", 0.1),
            ],
            &[("OPT_GATE", "EVERYWHERE"), ("OPT_RW", "CONSTANT")],
        );
        let out = gascorr(&ctx).unwrap();
        // Uncorrected density porosity is inflated to ~0.395; the loop must pull
        // it back to the true 0.30 and land on the liquid-replaced density 2.155.
        assert!((out["PHIT_GC"][0] - 0.300).abs() < 1e-3, "phit {}", out["PHIT_GC"][0]);
        assert!((out["SWT_GC"][0] - 0.400).abs() < 2e-3, "swt {}", out["SWT_GC"][0]);
        assert!((out["RHOB_GC"][0] - 2.155).abs() < 2e-3, "rhob_gc {}", out["RHOB_GC"][0]);
        assert!((out["GASDEN"][0] - 0.1297).abs() < 0.001);
        // Water zone: Sw clamps to 1, correction is exactly zero.
        assert_eq!(out["RHOB_GC"][1], 2.155);
        assert!((out["PHIT_GC"][1] - 0.30).abs() < 1e-4);
        assert_eq!(out["SWT_GC"][1], 1.0);
    }

    #[test]
    fn gascorr_flag_gate_and_missing_inputs() {
        // FLAGGED: only flag == 1 corrects; 0 and MISSING pass RHOB through.
        let ctx = ctx_with(
            3,
            &[
                ("RHOB", vec![2.0, 2.0, 2.0]),
                ("RT", vec![6.9444, 6.9444, 6.9444]),
                ("FTEMP", vec![93.9, 93.9, 93.9]),
                ("FPRESS", vec![2743.34, 2743.34, 2743.34]),
                ("GAS_FLAG", vec![1.0, 0.0, f32::NAN]),
            ],
            &[
                ("RHO_MA", 2.65),
                ("RHO_FL", 1.0),
                ("SG_GAS", 0.65),
                ("A", 1.0),
                ("M", 2.0),
                ("N", 2.0),
                ("RW", 0.1),
            ],
            &[("OPT_GATE", "FLAGGED"), ("OPT_RW", "CONSTANT")],
        );
        let out = gascorr(&ctx).unwrap();
        assert!(out["RHOB_GC"][0] > 2.0, "flagged sample corrected upward");
        assert_eq!(out["RHOB_GC"][1], 2.0, "flag 0 passes through");
        assert_eq!(out["RHOB_GC"][2], 2.0, "flag MISSING passes through");
        assert!(out["SWT_GC"][1].is_nan() && out["GASDEN"][1].is_nan());
        // Missing FPRESS (precalc not run) -> corrected outputs stay MISSING,
        // never a silent uncorrected pass-through.
        let ctx = ctx_with(
            1,
            &[("RHOB", vec![2.0]), ("RT", vec![6.9444]), ("FTEMP", vec![93.9])],
            &[("RHO_MA", 2.65), ("RHO_FL", 1.0), ("SG_GAS", 0.65), ("RW", 0.1)],
            &[("OPT_GATE", "EVERYWHERE"), ("OPT_RW", "CONSTANT")],
        );
        let out = gascorr(&ctx).unwrap();
        assert!(out["RHOB_GC"][0].is_nan() && out["PHIT_GC"][0].is_nan());
    }

    #[test]
    fn gascorr_guards_stay_missing_or_error() {
        let base_params: &[(&str, f64)] = &[
            ("RHO_MA", 2.65),
            ("RHO_FL", 1.0),
            ("SG_GAS", 0.65),
            ("A", 1.0),
            ("M", 2.0),
            ("N", 2.0),
            ("RW", 0.1),
        ];
        // FLAGGED with a flag curve that resolved to nothing: a silent zero-correction
        // run would be indistinguishable from "no gas anywhere" — must error instead.
        let ctx = ctx_with(
            2,
            &[
                ("RHOB", vec![2.0, 2.0]),
                ("RT", vec![6.9444, 6.9444]),
                ("FTEMP", vec![93.9, 93.9]),
                ("FPRESS", vec![2743.34, 2743.34]),
            ],
            base_params,
            &[("OPT_GATE", "FLAGGED"), ("OPT_RW", "CONSTANT")],
        );
        assert!(gascorr(&ctx).is_err(), "all-NaN flag under FLAGGED must be loud");

        // Fractional flags (depth-shifted bed edges): > 0.5 corrects, <= 0.5 passes.
        let ctx = ctx_with(
            2,
            &[
                ("RHOB", vec![2.0, 2.0]),
                ("RT", vec![6.9444, 6.9444]),
                ("FTEMP", vec![93.9, 93.9]),
                ("FPRESS", vec![2743.34, 2743.34]),
                ("GAS_FLAG", vec![0.9, 0.4]),
            ],
            base_params,
            &[("OPT_GATE", "FLAGGED"), ("OPT_RW", "CONSTANT")],
        );
        let out = gascorr(&ctx).unwrap();
        assert!(out["RHOB_GC"][0] > 2.0, "flag 0.9 corrects");
        assert_eq!(out["RHOB_GC"][1], 2.0, "flag 0.4 passes through");

        // Degenerate zone overrides (bypass dialog ranges) and unphysical densities:
        // RHO_FL >= RHO_MA, RHOB below RHO_FL, and a negative Rw all stay MISSING
        // instead of writing plausible-looking garbage (condflag precedent).
        for (params, what) in [
            (&[("RHO_MA", 2.65), ("RHO_FL", 2.65), ("SG_GAS", 0.65), ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1)][..], "RHO_FL == RHO_MA"),
            (&[("RHO_MA", 1.0), ("RHO_FL", 2.65), ("SG_GAS", 0.65), ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", 0.1)][..], "RHO_MA < RHO_FL"),
            (&[("RHO_MA", 2.65), ("RHO_FL", 1.0), ("SG_GAS", 0.65), ("A", 1.0), ("M", 2.0), ("N", 2.0), ("RW", -0.05)][..], "negative RW"),
        ] {
            let ctx = ctx_with(
                1,
                &[
                    ("RHOB", vec![2.0]),
                    ("RT", vec![200.0]),
                    ("FTEMP", vec![93.9]),
                    ("FPRESS", vec![2743.34]),
                ],
                params,
                &[("OPT_GATE", "EVERYWHERE"), ("OPT_RW", "CONSTANT")],
            );
            let out = gascorr(&ctx).unwrap();
            assert!(
                out["RHOB_GC"][0].is_nan() && out["PHIT_GC"][0].is_nan() && out["SWT_GC"][0].is_nan(),
                "{what} must leave outputs MISSING"
            );
        }

        // A washout reading below the restored fluid density has no meaningful
        // density porosity — MISSING, not a phantom 100%-porosity gas sand.
        let ctx = ctx_with(
            1,
            &[
                ("RHOB", vec![0.95]),
                ("RT", vec![200.0]),
                ("FTEMP", vec![93.9]),
                ("FPRESS", vec![2743.34]),
            ],
            base_params,
            &[("OPT_GATE", "EVERYWHERE"), ("OPT_RW", "CONSTANT")],
        );
        let out = gascorr(&ctx).unwrap();
        assert!(out["RHOB_GC"][0].is_nan() && out["PHIT_GC"][0].is_nan());
    }

    /// T-PREP-13, the half `gascorr_guards_stay_missing_or_error` leaves open: that refusal
    /// only asserts `is_err()`. Two things about the message itself carry the weight.
    ///
    /// **It must name the curve the USER picked, not the slot.** The gate flag defaults to
    /// XOVER_FLAG but any flag can be chosen, and someone sent to look at "GAS_FLAG" when they
    /// selected something else is being sent to a curve that does not exist in their project.
    /// They will conclude the message is wrong before they conclude their flag is empty.
    ///
    /// **The remedy the message offers must actually work.** It tells the user to set
    /// OPT_GATE = EVERYWHERE, so that path is exercised with the same empty flag: it must
    /// correct rather than refuse. A refusal that recommends a fix nobody tested is worse than
    /// a bare refusal — it spends the user's trust on advice that sends them in a circle.
    #[test]
    fn the_empty_flag_refusal_names_the_users_curve_and_its_remedy_works() {
        let params: &[(&str, f64)] = &[
            ("RHO_MA", 2.65),
            ("RHO_FL", 1.0),
            ("SG_GAS", 0.65),
            ("A", 1.0),
            ("M", 2.0),
            ("N", 2.0),
            ("RW", 0.1),
        ];
        let logs = [
            ("RHOB", vec![2.0f32]),
            ("RT", vec![6.9444f32]),
            ("FTEMP", vec![93.9f32]),
            ("FPRESS", vec![2743.34f32]),
            ("GAS_FLAG", vec![f32::NAN]),
        ];

        let err = gascorr(&ctx_with(
            1,
            &logs,
            params,
            &[("OPT_GATE", "FLAGGED"), ("OPT_RW", "CONSTANT"), ("__IN_GAS_FLAG", "MY_GAS_ZONES")],
        ))
        .expect_err("an empty flag under FLAGGED must refuse");
        assert!(
            err.contains("MY_GAS_ZONES"),
            "the refusal must name the flag the user chose, got: {err}"
        );
        assert!(err.contains("EVERYWHERE"), "the refusal must state the remedy, got: {err}");

        // Nothing chosen — the message falls back to the slot name rather than an empty quote.
        let err = gascorr(&ctx_with(
            1,
            &logs,
            params,
            &[("OPT_GATE", "FLAGGED"), ("OPT_RW", "CONSTANT")],
        ))
        .expect_err("still a refusal with no chosen mnemonic");
        assert!(err.contains("GAS_FLAG"), "no chosen curve must not leave a blank name: {err}");

        // The remedy, on the same empty flag: EVERYWHERE corrects instead of refusing.
        let out = gascorr(&ctx_with(
            1,
            &logs,
            params,
            &[("OPT_GATE", "EVERYWHERE"), ("OPT_RW", "CONSTANT"), ("__IN_GAS_FLAG", "MY_GAS_ZONES")],
        ))
        .expect("EVERYWHERE is the documented escape hatch — it must not refuse too");
        assert!(
            out["RHOB_GC"][0] > 2.0,
            "the remedy must actually correct, got {}",
            out["RHOB_GC"][0]
        );
    }

    /// A GR normalization reference pair is interpretation data, not a generic default. A pair
    /// from another study still makes a smooth, plausible curve, so both ends must ship absent.
    #[test]
    fn gr_normalize_reference_pair_ships_absent_not_as_a_field_calibration() {
        let spec = gr_normalize_spec();
        for name in ["GR_LOW_REF", "GR_HIGH_REF"] {
            let arg = spec.args.iter().find(|a| a.name == name).unwrap();
            assert!(
                arg.default.is_empty(),
                "{name} must not conceal a numeric reference"
            );
            assert_eq!(arg.default_source, ABSENT_DEFAULT_SOURCE);
            assert!(arg.required, "both ends of the reference pair are required");
        }
    }

    /// CORRECTNESS — SB-CLY-042 and `10_clay-volume.md` sections 3.5 F15/F17, 4.3,
    /// 5 and 5.1. IP's percentile pipeline, Techlog's quantile habit and the two
    /// documented N-D clean-line constructions are advice for choosing endpoints, not endpoint
    /// values. SB-CLY-050 separately withdraws the disputed RHO_MA value while keeping the agreed
    /// RHO_FL/NPHI_FL defaults and the named P3/P97 normalization preset as positive controls, so a lazy
    /// implementation that simply erases every number.
    #[test]
    fn documented_picking_conventions_are_sourced_help_and_never_numeric_defaults() {
        let catalog = module_catalog();
        let argument = |module_name: &str, argument_name: &str| {
            catalog
                .iter()
                .find(|module| module.name == module_name)
                .unwrap_or_else(|| panic!("missing module {module_name}"))
                .args
                .iter()
                .find(|arg| arg.name == argument_name)
                .unwrap_or_else(|| panic!("missing argument {module_name}.{argument_name}"))
        };

        for (module_name, argument_name) in [
            ("vsh_gr", "GR_MA"),
            ("vsh_gr", "GR_SH"),
            ("vsh_dn", "RHO_MA"),
            ("vsh_dn", "RHO_SH"),
            ("vsh_dn", "NPHI_MA"),
            ("vsh_dn", "NPHI_SH"),
            ("vsh_dn", "GR_MA"),
            ("vsh_dn", "GR_SH"),
            ("gr_normalize", "GR_LOW_REF"),
            ("gr_normalize", "GR_HIGH_REF"),
        ] {
            let arg = argument(module_name, argument_name);
            assert!(
                arg.default.is_empty(),
                "{module_name}.{argument_name} must not turn picking advice into a value"
            );
            assert_eq!(
                arg.default_source,
                ABSENT_DEFAULT_SOURCE,
                "{module_name}.{argument_name} must declare the missing value explicitly"
            );
            assert!(
                !arg.guidance.is_empty(),
                "{module_name}.{argument_name} must carry its picking convention beside the field"
            );
            assert!(
                arg.guidance.iter().all(|item| {
                    !item.text.trim().is_empty()
                        && !item.source.trim().is_empty()
                        && item.source.contains("10_clay-volume.md")
                }),
                "{module_name}.{argument_name} guidance must state both advice and source"
            );
        }

        for argument_name in ["RHO_FL", "NPHI_FL"] {
            let arg = argument("vsh_dn", argument_name);
            assert!(
                arg.default.parse::<f64>().is_ok(),
                "the independently cited {argument_name} default must not be erased"
            );
            assert_ne!(arg.default_source, ABSENT_DEFAULT_SOURCE);
            assert!(
                !arg.guidance.is_empty(),
                "{argument_name} still participates in the documented crossplot construction"
            );
        }

        for argument_name in ["P_LOW", "P_HIGH"] {
            let arg = argument("gr_normalize", argument_name);
            assert!(
                arg.default.parse::<f64>().is_ok(),
                "the named P3/P97 preset is cited and must remain selectable"
            );
            assert!(arg.default_source.contains("method_workflow_standards.md"));
            assert!(
                !arg.guidance.is_empty(),
                "the preset must be described as a named convention rather than an endpoint"
            );
        }
    }

    /// CORRECTNESS — SB-CLY-051 / SB-CLY-T33 and `10_clay-volume.md` sections 4.5, 5
    /// and 6. The chapter identifies the exact Geolog `.info`, Techlog HTML and SandiBumi
    /// section locators; a product name by itself is explicitly rejected there. The exact
    /// three-default inventory prevents an implementation from validating only one convenient
    /// field, while the effective-parameter assertion proves the locator reaches the run record.
    #[test]
    fn every_shipping_clay_default_names_a_checkable_artefact_and_a_product_name_alone_fails_before_the_run_record() {
        let clay_modules = module_catalog()
            .iter()
            .filter(|module| module.category == "VSH")
            .collect::<Vec<_>>();
        assert_eq!(
            clay_modules
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>(),
            vec!["vsh_gr", "vsh_dn"],
            "the source audit must cover every shipping CLY module"
        );

        let shipping_defaults = clay_modules
            .iter()
            .flat_map(|module| {
                module.args.iter().filter_map(move |arg| {
                    (arg.kind == ArgKind::Param && arg.default.parse::<f64>().is_ok()).then(|| {
                        (
                            format!("{}.{}", module.name, arg.name),
                            arg.default_source.as_str(),
                        )
                    })
                })
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            shipping_defaults.keys().cloned().collect::<Vec<_>>(),
            vec![
                "vsh_dn.FLAG_TOL".to_string(),
                "vsh_dn.NPHI_FL".to_string(),
                "vsh_dn.RHO_FL".to_string(),
            ],
            "no shipping clay default may escape the source audit"
        );
        for (identity, locator) in &shipping_defaults {
            let expected_locator = match identity.as_str() {
                "vsh_dn.RHO_FL" => "vsh_dn.info RHO_FL",
                "vsh_dn.NPHI_FL" => "vsh_dn.info NPHI_FL",
                "vsh_dn.FLAG_TOL" => "docs/PRD_v2/10_clay-volume.md §5.1",
                _ => unreachable!("the exact inventory assertion above closed this vocabulary"),
            };
            assert!(
                locator.contains(expected_locator),
                "{identity} must name its checkable artefact locator, got: {locator}"
            );
        }

        let mut product_only = vsh_dn_spec();
        product_only
            .args
            .iter_mut()
            .find(|arg| arg.name == "RHO_FL")
            .unwrap()
            .default_source = "Techlog".into();
        let registry_error = validate_parameter_sources(std::slice::from_ref(&product_only))
            .expect_err("a product name alone must fail the CLY registry gate");
        assert!(registry_error.contains("vsh_dn.RHO_FL"));
        assert!(
            registry_error.contains("checkable artefact"),
            "the refusal must state the missing evidence, got: {registry_error}"
        );
        let run_error = crate::workflow::effective_module_parameters(
            &product_only,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            "explicit values sourced by this test",
            "",
        )
        .expect_err("the same invalid manifest must fail before constructing a run record");
        assert!(run_error.contains("vsh_dn.RHO_FL"));

        let vsh_dn = clay_modules
            .iter()
            .find(|module| module.name == "vsh_dn")
            .unwrap();
        let (recorded, _) = crate::workflow::effective_module_parameters(
            vsh_dn,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            "explicit values sourced by this test",
            "",
        )
        .expect("the cited shipping manifest must construct a run record");
        let recorded_defaults = recorded
            .iter()
            .filter(|parameter| {
                parameter.resolution == Some(crate::equations::ParameterResolution::Defaulted)
                    && !parameter.name.ends_with("@unit_custody")
            })
            .map(|parameter| (parameter.name.as_str(), parameter.source.as_str()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            recorded_defaults,
            BTreeMap::from([
                ("FLAG_TOL", shipping_defaults["vsh_dn.FLAG_TOL"]),
                ("NPHI_FL", shipping_defaults["vsh_dn.NPHI_FL"]),
                ("RHO_FL", shipping_defaults["vsh_dn.RHO_FL"]),
            ]),
            "every shipping clay default must retain its primary source in the run record"
        );
    }

    /// CORRECTNESS — SB-CLY-054, SB-CLY-T21 and the physical-unit limb of SB-CLY-T42.
    /// The exact CLY argument inventory and units come from `10_clay-volume.md` sections 4.5/5;
    /// `2645 k/m3 -> 2.645 g/cc` and the `1e-9` tolerance come from T42. The two bilinear
    /// expressions and the 10,000-case/two-unit-system oracle come independently from T21 and
    /// dossier section 2.7, not from the implementation under test. SB-CLY-045's separate
    /// Vsh/Vcl semantic bridge is deliberately not claimed here.
    #[test]
    fn every_clay_quantity_is_unit_typed_and_named_conversions_preserve_bilinear_results_and_run_custody() {
        let clay_quantities = module_catalog()
            .iter()
            .filter(|module| module.category == "VSH")
            .flat_map(|module| {
                module.args.iter().filter_map(move |argument| {
                    matches!(argument.kind, ArgKind::Param | ArgKind::LogIn | ArgKind::LogOut)
                        .then(|| {
                            (
                                format!("{}.{}", module.name, argument.name),
                                argument.unit.clone(),
                            )
                        })
                })
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            clay_quantities,
            BTreeMap::from([
                ("vsh_dn.FLAG_TOL".into(), "v/v".into()),
                ("vsh_dn.GR".into(), "gAPI".into()),
                ("vsh_dn.GR_MA".into(), "gAPI".into()),
                ("vsh_dn.GR_SH".into(), "gAPI".into()),
                ("vsh_dn.NPHI".into(), "v/v".into()),
                ("vsh_dn.NPHI_FL".into(), "v/v".into()),
                ("vsh_dn.NPHI_MA".into(), "v/v".into()),
                ("vsh_dn.NPHI_SH".into(), "v/v".into()),
                ("vsh_dn.RHOB".into(), "g/cc".into()),
                ("vsh_dn.RHO_FL".into(), "g/cc".into()),
                ("vsh_dn.RHO_MA".into(), "g/cc".into()),
                ("vsh_dn.RHO_SH".into(), "g/cc".into()),
                ("vsh_dn.VSH".into(), "v/v".into()),
                ("vsh_dn.VSH_DN".into(), "v/v".into()),
                ("vsh_dn.VSH_DN_FLAG".into(), "flag".into()),
                ("vsh_gr.GR".into(), "gAPI".into()),
                ("vsh_gr.GR_MA".into(), "gAPI".into()),
                ("vsh_gr.GR_SH".into(), "gAPI".into()),
                ("vsh_gr.VSH".into(), "v/v".into()),
                ("vsh_gr.VSH_GR".into(), "v/v".into()),
            ]),
            "every shipping CLY quantity must declare the chapter unit with no spelling drift"
        );
        for (identity, unit) in &clay_quantities {
            assert!(
                crate::curves::resolve_unit_token(unit).is_some(),
                "{identity} uses unregistered unit token '{unit}'"
            );
        }

        let density_rule = crate::curves::UNIT_RULES
            .iter()
            .find(|rule| rule.from_unit == "kg/m3" && rule.to_unit == "g/cc")
            .expect("the cited density conversion is registered");
        let converted_matrix_density = 2645.0_f64 * density_rule.factor as f64;
        assert!(
            (converted_matrix_density - 2.645).abs() <= 1e-9,
            "T42 requires 2645 k/m3 -> 2.645 g/cc within 1e-9, got {converted_matrix_density}"
        );
        assert!(
            crate::curves::validate_unit_bridge("kg/m3", "v/v")
                .unwrap_err()
                .to_string()
                .contains("quantity-kind mismatch"),
            "a density source unit must be refused for a fraction quantity"
        );

        let vsh_dn = module_catalog()
            .iter()
            .find(|module| module.name == "vsh_dn")
            .expect("vsh_dn is registered");
        let default_custody = vsh_dn
            .args
            .iter()
            .filter(|argument| {
                argument.kind == ArgKind::Param && argument.default.parse::<f64>().is_ok()
            })
            .map(|argument| {
                let encoded = serde_json::to_value(argument).expect("ArgSpec serializes over IPC");
                let custody = encoded
                    .get("default_unit_custody")
                    .unwrap_or_else(|| panic!("{}.{} lacks unit custody", vsh_dn.name, argument.name));
                (argument.name.as_str(), custody.clone())
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            default_custody.keys().copied().collect::<Vec<_>>(),
            vec!["FLAG_TOL", "NPHI_FL", "RHO_FL"],
            "every and only cited shipping defaults carry artefact-unit custody"
        );
        let rho_fl_custody = &default_custody["RHO_FL"];
        assert_eq!(rho_fl_custody["artefact_value"].as_f64(), Some(1000.0));
        assert_eq!(rho_fl_custody["artefact_unit"], "k/m3");
        assert_eq!(rho_fl_custody["canonical_value"].as_f64(), Some(1.0));
        assert_eq!(rho_fl_custody["canonical_unit"], "g/cc");
        assert_eq!(
            rho_fl_custody["conversion"]["identity"],
            "curve-units-v2:kg/m3->g/cc"
        );
        assert_eq!(
            rho_fl_custody["conversion"]["factor"].as_f64(),
            Some(0.001)
        );
        assert!(rho_fl_custody["conversion"]["derivation"]
            .as_str()
            .is_some_and(|text| text.contains("1000 kg/m3")));

        let (recorded, _) = crate::workflow::effective_module_parameters(
            vsh_dn,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            "explicit values sourced by this test",
            "",
        )
        .expect("the unit-typed CLY manifest constructs a run record");
        let recorded_rho_fl = recorded
            .iter()
            .find(|parameter| parameter.name == "RHO_FL@unit_custody")
            .expect("RHO_FL artefact unit and conversion must survive into run custody");
        assert_eq!(recorded_rho_fl.value, rho_fl_custody.clone());
        assert_eq!(
            recorded_rho_fl.source,
            vsh_dn
                .args
                .iter()
                .find(|argument| argument.name == "RHO_FL")
                .unwrap()
                .default_source
        );
        assert_eq!(
            recorded
                .iter()
                .filter(|parameter| parameter.name.ends_with("@unit_custody"))
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "RHO_FL@unit_custody",
                "NPHI_FL@unit_custody",
                "FLAG_TOL@unit_custody",
            ],
            "every and only effective defaulted CLY parameter must carry run unit custody"
        );

        let explicit_source = "interpreter supplied canonical values for this test";
        let (explicit_recorded, _) = crate::workflow::effective_module_parameters(
            vsh_dn,
            &std::collections::HashMap::from([("RHO_MA".to_string(), 2.645)]),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            explicit_source,
            "",
        )
        .expect("an explicit canonical CLY value constructs identity unit custody");
        let explicit_custody = explicit_recorded
            .iter()
            .find(|parameter| parameter.name == "RHO_MA@unit_custody")
            .expect("an explicit numeric CLY parameter must record canonical-unit custody");
        assert_eq!(explicit_custody.value["artefact_value"].as_f64(), Some(2.645));
        assert_eq!(explicit_custody.value["artefact_unit"], "g/cc");
        assert_eq!(explicit_custody.value["canonical_value"].as_f64(), Some(2.645));
        assert_eq!(explicit_custody.value["canonical_unit"], "g/cc");
        assert_eq!(
            explicit_custody.value["conversion"]["identity"],
            "curve-units-v2:g/cc->g/cc"
        );
        assert_eq!(explicit_custody.source, explicit_source);
        assert!(
            explicit_recorded
                .iter()
                .all(|parameter| parameter.name != "RHO_SH@unit_custody"),
            "an ABSENT parameter has no value and must not receive invented unit custody"
        );

        let mut state = 0x5A17_C1A7_054_u64;
        let mut next_fraction = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((state >> 11) as f64) / ((1_u64 << 53) as f64)
        };
        let ranged = |fraction: f64, low: f64, high: f64| low + fraction * (high - low);
        let bilinear = |rho_b: f64,
                        nphi: f64,
                        rho_ma: f64,
                        rho_sh: f64,
                        rho_fl: f64,
                        nphi_ma: f64,
                        nphi_sh: f64,
                        nphi_fl: f64| {
            ((rho_fl - rho_ma) * (nphi - nphi_ma)
                - (nphi_fl - nphi_ma) * (rho_b - rho_ma))
                / ((rho_fl - rho_ma) * (nphi_sh - nphi_ma)
                    - (nphi_fl - nphi_ma) * (rho_sh - rho_ma))
        };
        let mut checked = 0_usize;
        while checked < 10_000 {
            let rho_ma = ranged(next_fraction(), 2.55, 2.85);
            let rho_sh = ranged(next_fraction(), 2.15, 2.55);
            let rho_fl = ranged(next_fraction(), 0.85, 1.15);
            let nphi_ma = ranged(next_fraction(), -0.10, 0.05);
            let nphi_sh = ranged(next_fraction(), 0.20, 0.55);
            let nphi_fl = ranged(next_fraction(), 0.85, 1.15);
            let rho_b = ranged(next_fraction(), 1.80, 2.95);
            let nphi = ranged(next_fraction(), -0.15, 0.70);
            let denominator = (rho_fl - rho_ma) * (nphi_sh - nphi_ma)
                - (nphi_fl - nphi_ma) * (rho_sh - rho_ma);
            if denominator.abs() < 0.02 {
                continue;
            }
            let canonical = bilinear(
                rho_b, nphi, rho_ma, rho_sh, rho_fl, nphi_ma, nphi_sh, nphi_fl,
            );
            let shipped = vsh_dn_rearrangement(
                rho_b, nphi, rho_ma, rho_sh, rho_fl, nphi_ma, nphi_sh, nphi_fl,
            )
            .expect("the swept fixture excludes degenerate endpoint geometry");
            assert!((canonical - shipped).abs() <= 1e-12, "g/cc case {checked}");

            let source_scale = 1.0 / density_rule.factor as f64;
            let converted = |density: f64| density * source_scale * density_rule.factor as f64;
            let converted_canonical = bilinear(
                converted(rho_b),
                nphi,
                converted(rho_ma),
                converted(rho_sh),
                converted(rho_fl),
                nphi_ma,
                nphi_sh,
                nphi_fl,
            );
            let converted_shipped = vsh_dn_rearrangement(
                converted(rho_b),
                nphi,
                converted(rho_ma),
                converted(rho_sh),
                converted(rho_fl),
                nphi_ma,
                nphi_sh,
                nphi_fl,
            )
            .expect("unit conversion preserves non-degenerate endpoint geometry");
            assert!(
                (converted_canonical - converted_shipped).abs() <= 1e-12,
                "k/m3 conversion case {checked}"
            );
            checked += 1;
        }
    }

    /// CORRECTNESS — SB-CLY-050 / SB-CLY-T18. The empty/default disposition and the
    /// two endpoint sets are copied from `10_clay-volume.md` sections 4.5, 5 and 6.
    /// The expected VSH values are the chapter's independently evaluated Geolog
    /// `vsh_dn.lls` equation, not values derived through this implementation.
    #[test]
    fn when_vendors_disagree_the_parameter_opens_empty_shows_all_sources_and_refuses_before_arithmetic() {
        let catalog = module_catalog();
        let argument = |module_name: &str, argument_name: &str| {
            catalog
                .iter()
                .find(|module| module.name == module_name)
                .unwrap_or_else(|| panic!("missing module {module_name}"))
                .args
                .iter()
                .find(|arg| arg.name == argument_name)
                .unwrap_or_else(|| panic!("missing argument {module_name}.{argument_name}"))
        };

        let disputed = [
            ("vsh_gr", "GR_MA", crate::param_sources::GR_CLEAN_ENDPOINT),
            ("vsh_gr", "GR_SH", crate::param_sources::GR_SHALE_ENDPOINT),
            ("vsh_dn", "RHO_MA", crate::param_sources::MATRIX_DENSITY),
            ("vsh_dn", "RHO_SH", crate::param_sources::SHALE_DENSITY),
            ("vsh_dn", "NPHI_MA", "matrix_neutron_endpoint"),
            ("vsh_dn", "NPHI_SH", crate::param_sources::SHALE_NEUTRON_ENDPOINT),
            ("vsh_dn", "GR_MA", crate::param_sources::GR_CLEAN_ENDPOINT),
            ("vsh_dn", "GR_SH", crate::param_sources::GR_SHALE_ENDPOINT),
        ];
        for (module_name, argument_name, topic) in disputed {
            let arg = argument(module_name, argument_name);
            assert!(
                arg.default.is_empty(),
                "{module_name}.{argument_name} must not silently select a vendor position"
            );
            assert_eq!(arg.default_source, ABSENT_DEFAULT_SOURCE);
            assert!(arg.required, "{module_name}.{argument_name} must be set before evaluation");
            assert_eq!(
                arg.sources_topic, topic,
                "{module_name}.{argument_name} must show its competing evidence beside the field"
            );
            assert!(
                crate::param_sources::sources_for(topic).len() >= 2,
                "{module_name}.{argument_name} must expose every held vendor position"
            );
        }

        for (topic, product, value, source_fragment, tier) in [
            (crate::param_sources::GR_CLEAN_ENDPOINT, "Techlog", "10", "gamma-ray.html", "T1′"),
            (crate::param_sources::GR_SHALE_ENDPOINT, "Techlog", "100", "gamma-ray.html", "T1′"),
            (crate::param_sources::MATRIX_DENSITY, "Geolog", "2.645", "vsh_dn.info", "T1"),
            (crate::param_sources::MATRIX_DENSITY, "Techlog documentation", "2.65", "neutrondensity.html", "T1′"),
            (crate::param_sources::SHALE_DENSITY, "Techlog documentation", "2.4", "neutrondensity.html", "T1′"),
            (crate::param_sources::SHALE_DENSITY, "Techlog template", "2.45", "C2_method_defaults.json", "T3"),
            ("matrix_neutron_endpoint", "Techlog documentation", "-0.1", "neutrondensity.html", "T1′"),
            ("matrix_neutron_endpoint", "Techlog template", "0", "C2_method_defaults.json", "T3"),
            (crate::param_sources::SHALE_NEUTRON_ENDPOINT, "Techlog documentation", "0.40", "neutrondensity.html", "T1′"),
        ] {
            assert!(
                crate::param_sources::sources_for(topic)
                    .iter()
                    .any(|row| {
                        row.product == product
                            && row.value == value
                            && row.source.contains(source_fragment)
                            && row.tier == tier
                    }),
                "{topic} must expose {product} position {value} from {source_fragment} at {tier}"
            );
        }

        // Positive controls: these three values are agreed/cited in the same chapter and are not
        // to be erased by an implementation that treats every clay number as disputed.
        for (name, expected) in [("RHO_FL", 1.0), ("NPHI_FL", 1.0), ("FLAG_TOL", 0.25)] {
            let arg = argument("vsh_dn", name);
            assert!((arg.default.parse::<f64>().unwrap() - expected).abs() < f64::EPSILON);
            assert_ne!(arg.default_source, ABSENT_DEFAULT_SOURCE);
        }

        let incomplete = ctx_with(
            1,
            &[("RHOB", vec![2.35]), ("NPHI", vec![0.30])],
            &[
                ("RHO_SH", 2.45),
                ("RHO_FL", 1.0),
                ("NPHI_MA", 0.0),
                ("NPHI_SH", 0.4),
                ("NPHI_FL", 1.0),
                ("GR_MA", 10.0),
                ("GR_SH", 100.0),
                ("FLAG_TOL", 0.25),
            ],
            &[],
        );
        let error = run_module("vsh_dn", &incomplete).unwrap_err();
        assert!(error.contains("RHO_MA"), "the refusal must name the unset parameter: {error}");
        assert!(
            error.contains("ABSENT"),
            "the refusal must happen at source/default validation before arithmetic: {error}"
        );

        let run = |rho_sh: f64, nphi_ma: f64| {
            run_module(
                "vsh_dn",
                &ctx_with(
                    1,
                    &[("RHOB", vec![2.35]), ("NPHI", vec![0.30])],
                    &[
                        ("RHO_MA", 2.65),
                        ("RHO_SH", rho_sh),
                        ("RHO_FL", 1.0),
                        ("NPHI_MA", nphi_ma),
                        ("NPHI_SH", 0.4),
                        ("NPHI_FL", 1.0),
                        ("GR_MA", 10.0),
                        ("GR_SH", 100.0),
                        ("FLAG_TOL", 0.25),
                    ],
                    &[],
                ),
            )
            .unwrap()["VSH_DN"][0]
        };
        let template = run(2.45, 0.0);
        let documentation = run(2.40, -0.1);
        assert!((template - 0.4239).abs() <= 1e-4, "Techlog template set: {template}");
        assert!((documentation - 0.6000).abs() <= 1e-4, "Techlog documentation set: {documentation}");
        assert!(
            (template - documentation).abs() > 0.17,
            "the two source positions must remain distinct rather than silently averaged"
        );
    }

    #[test]
    fn gr_normalize_maps_well_percentiles_to_reference() {
        // GR uniform 0..100 → P3_well = 3, P97_well = 97. After normalization those
        // must land exactly on the reference values 53.68 / 133.93.
        let gr: Vec<f32> = (0..=100).map(|i| i as f32).collect();
        let ctx = ctx_with(
            101,
            &[("GR", gr)],
            &[("P_LOW", 3.0), ("P_HIGH", 97.0), ("GR_LOW_REF", 53.68), ("GR_HIGH_REF", 133.93)],
            &[],
        );
        let out = gr_normalize(&ctx);
        let grn = &out["GRN"];
        assert!((grn[3] as f64 - 53.68).abs() < 1e-3, "P3 sample → ref P3, got {}", grn[3]);
        assert!((grn[97] as f64 - 133.93).abs() < 1e-3, "P97 sample → ref P97, got {}", grn[97]);
        // Affine: midpoint maps to the midpoint of the reference span.
        assert!((grn[50] as f64 - (53.68 + 133.93) / 2.0).abs() < 0.5);
    }

    #[test]
    fn log_predict_learns_association_and_fills_gaps() {
        // TARGET = 2·P1 + 10 on the training half; the second half has no target.
        let n = 200;
        let p1: Vec<f32> = (0..n).map(|i| (i % 100) as f32).collect();
        let target: Vec<f32> =
            (0..n).map(|i| if i < 100 { 2.0 * (i as f32) + 10.0 } else { f32::NAN }).collect();
        let ctx = ctx_with(
            n,
            &[("TARGET", target), ("P1", p1)],
            &[("K", 3.0)],
            &[("OPT_COMBINE", "SYNTHETIC"), ("__IN_TARGET", "RHOB")],
        );
        let out = log_predict(&ctx);
        let syn = &out["SYN"];
        // Sample 150 has P1 = 50 → prediction ≈ 110.
        assert!((syn[150] - 110.0).abs() < 3.0, "KNN should recover the trend, got {}", syn[150]);
        assert!(!syn[0].is_nan(), "training samples get predictions too");
    }

    #[test]
    fn log_predict_max_raw_keeps_raw_where_higher() {
        // Washout rule: raw RHOB above the synthetic is trusted.
        let n = 50;
        let p1: Vec<f32> = (0..n).map(|i| i as f32).collect();
        // Constant target 2.5 everywhere except one washout-low sample.
        let mut target: Vec<f32> = vec![2.5; n];
        target[25] = 2.0; // washed out: raw below trend
        let ctx = ctx_with(
            n,
            &[("TARGET", target), ("P1", p1)],
            &[("K", 5.0)],
            &[("OPT_COMBINE", "MAX_RAW"), ("__IN_TARGET", "RHOB")],
        );
        let out = log_predict(&ctx);
        let syn = &out["SYN"];
        assert!(syn[25] > 2.3, "washout sample must be pulled up toward the trend, got {}", syn[25]);
        assert!((syn[10] - 2.5).abs() < 1e-3, "good samples keep raw (raw ≥ synthetic)");
    }

    #[test]
    fn thin_bed_ts_pure_laminated_and_dispersed() {
        let phi_sd = 0.30;
        let phi_sh = 0.10;
        let vs = 0.4;
        // Point exactly on the laminated line -> VLAM == VSH, VDISP == 0, VSAND == 1-VSH.
        let lam_phit = phi_sd * (1.0 - vs) + phi_sh * vs;
        let ctx_lam = ctx_with(
            1,
            &[("PHIT", vec![lam_phit as f32]), ("VSH", vec![vs as f32])],
            &[("PHI_SD_MAX", phi_sd), ("PHI_SH", phi_sh)],
            &[],
        );
        let out_lam = thin_bed_ts(&ctx_lam);
        assert!((out_lam["VLAM"][0] as f64 - vs).abs() < 1e-4);
        assert!(out_lam["VDISP"][0].abs() < 1e-4);
        assert!((out_lam["VSAND"][0] as f64 - (1.0 - vs)).abs() < 1e-4);

        // Point exactly on the dispersed line -> VDISP == VSH, VLAM == 0, VSAND == 1.
        let disp_phit = phi_sd - vs * (1.0 - phi_sh);
        let ctx_disp = ctx_with(
            1,
            &[("PHIT", vec![disp_phit as f32]), ("VSH", vec![vs as f32])],
            &[("PHI_SD_MAX", phi_sd), ("PHI_SH", phi_sh)],
            &[],
        );
        let out_disp = thin_bed_ts(&ctx_disp);
        assert!(out_disp["VLAM"][0].abs() < 1e-4);
        assert!((out_disp["VDISP"][0] as f64 - vs).abs() < 1e-4);
        assert!((out_disp["VSAND"][0] as f64 - 1.0).abs() < 1e-4);
    }

    /// CHARACTERIZATION — SB-PLT-035 requires the interactive clay plot to call the
    /// governed batch equation. The current UI instead duplicates the two equations below.
    /// Their endpoint identity is algebraic — at VSH=PHI_SD, PHI_SD−VSH·(1−PHI_SH)
    /// reduces to PHI_SD·PHI_SH — so no uncited endpoint value enters this test.
    #[test]
    fn characterizes_the_interactive_clay_overlay_as_a_duplicate_formula_matching_batch_endpoints() {
        let spec = thin_bed_ts_spec();
        assert!(spec
            .doc
            .contains("PHIT = PHI_SD_MAX*(1-VSH) + PHI_SH*VSH"));
        assert!(spec
            .doc
            .contains("PHIT = PHI_SD_MAX - VSH*(1-PHI_SH)"));

        let ui = include_str!("../../src/ui/crossplotPanel.ts");
        assert!(ui.contains("const vMin = Math.min(1, phiSd)"));
        assert!(ui.contains("[vMin, phiSd * phiSh]"));
        assert!(ui.contains("PHIT = PHI_SD − VSH·(1−PHI_SH)"));
        assert!(ui.contains("[1, phiSh]"));
        assert!(
            !ui.contains("runModule(\"thin_bed_ts\"")
                && !ui.contains("invoke(\"thin_bed_ts\""),
            "the PARTIAL UI does not yet call the governed batch equation"
        );
    }
}
