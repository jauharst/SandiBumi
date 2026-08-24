//! Curve ancestry and log-set versioning: what a computed curve is a record OF.
//!
//! AUDIT-2026-08-20 finding 53. This was ~2,900 lines in the middle of `equations.rs`, which had
//! grown from 2,246 to 6,892 lines. The file's own job is curve RESOLUTION - fetch a mnemonic,
//! decide which set answers it, list the catalog, run an equation - and the highest-traffic
//! question in the repository ("how does a mnemonic resolve across sets?") had ended up behind
//! the entire custody model.
//!
//! The two concerns touch at ONE seam and it is a value type: the write path takes a
//! `CompleteLogSetSpec`.

use duckdb::{params, params_from_iter, Connection, OptionalExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

// The resolution side of the seam, in the direction the audit named: the write path takes a
// value type from here, and the ancestry recorder asks the resolver which curve answered a
// mnemonic. Three names, all from `equations`.
use crate::equations::{resolve_generic_curve_decision, CurveRequest, GenericCurveDecision};

// ---------------------------------------------------------------------------
// P1-c log-set versioning: run events + append-only history (never overwrite)
// ---------------------------------------------------------------------------

/// Provenance of one run event into a named log set.
#[derive(Debug, Clone)]
pub struct LogSetSpec {
    pub set_name: String,
    pub module: String,
    pub params_json: String,
    pub inputs_json: String,
}

// ---------------------------------------------------------------------------
// SB-ENV-005 (DEC-031(b), signed DRAFT_ENV005 under DEC-076): the applied-step
// manifest - the ordered list of steps actually applied to a log-set version,
// retrievable without re-running anything. It rides the version row itself
// (`log_sets.applied_steps_json`), written in the same transaction that
// allocates the version. One vocabulary answers SB-ENV-028's mask record and
// SB-ENV-042's edit provenance too: a mask application is a step of kind
// "mask", an interactive edit a step of kind "edit" naming its recovery curve.
// ---------------------------------------------------------------------------

/// Schema version stamped into every written manifest. A stored manifest whose
/// `v` this build does not know REFUSES interpretation (naming the version) while
/// the curves themselves still read - the column is consulted by nothing else.
pub(crate) const APPLIED_STEPS_SCHEMA_VERSION: u32 = 1;

/// Per-step outcome counts, copied from the run's own DEC-060 one-hot flag group
/// (`<OUT>_FULL/_PARTIAL/_NONE/_REFUSED`) at the moment the run resolved them -
/// never re-derived later. Absent (`None` on the step) where the writer did not
/// have the counts: omission is representable, fabrication is not.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedStepOutcome {
    pub full: u64,
    pub partial: u64,
    pub none: u64,
    pub refused: u64,
}

/// One applied step. Every field is COPIED from what the run already resolved;
/// a step the runner cannot fully describe writes the fields it has and omits
/// the rest as `null`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedStep {
    pub seq: u32,
    /// "module" | "correction" | "mask" | "edit".
    pub kind: String,
    /// Module/equation identity; absent only for kind "edit".
    #[serde(default)]
    pub module: Option<String>,
    /// SHA-256 hex digest of the run's resolved `params_json` (which stays on the
    /// same row) - makes "same step re-applied?" decidable without parsing.
    #[serde(default)]
    pub params_digest: Option<String>,
    /// Resolved input mnemonics with set qualification ("NPHI@RAW").
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outcome: Option<AppliedStepOutcome>,
    /// The rule-11 mask flag curve this step consumed, where one was.
    #[serde(default)]
    pub mask: Option<String>,
    /// The SB-ENV-037 bit-exact recovery record curve, where one exists.
    #[serde(default)]
    pub recovery: Option<String>,
}

/// The manifest: versioned JSON `{"v": 1, "steps": [...]}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedStepsManifest {
    pub v: u32,
    pub steps: Vec<AppliedStep>,
}

/// What a retrieval returns. `Unknown` is a pre-contract version whose step
/// history cannot be recovered - deliberately NOT an empty step list, because an
/// empty list claims "nothing was applied", which is an answer, not an absence.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum AppliedStepsRecord {
    Manifest { manifest: AppliedStepsManifest },
    Unknown,
}

/// SHA-256 hex of the resolved parameter record - the manifest references the
/// `params_json` already on the same row rather than duplicating it.
pub(crate) fn params_digest_hex(params_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(params_json.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// The rule-11 mask is an opt the run resolved (`opts["MASK"]`), persisted inside
/// `params_json`. Copying it out is transcription of the run's own record; a
/// params record without the key simply has no mask step to report.
fn mask_from_params(params_json: &str) -> Option<String> {
    let params: serde_json::Value = serde_json::from_str(params_json).ok()?;
    params.get("MASK")?.as_str().map(str::to_string)
}

/// Derives the one-step manifest a complete run can honestly state about itself:
/// its module/equation identity, its parameter digest, its resolved inputs with
/// set qualification, and its mask. Outcome counts and recovery records belong
/// to the rows that own them (SB-ENV-010/011, SB-ENV-037) and are omitted here,
/// never invented.
pub(crate) fn derive_applied_steps(spec: &CompleteLogSetSpec) -> AppliedStepsManifest {
    AppliedStepsManifest {
        v: APPLIED_STEPS_SCHEMA_VERSION,
        steps: vec![AppliedStep {
            seq: 1,
            kind: "module".to_string(),
            module: Some(spec.storage.module.clone()),
            params_digest: Some(params_digest_hex(&spec.storage.params_json)),
            inputs: spec
                .ancestry
                .inputs
                .iter()
                .map(|input| format!("{}@{}", input.curve, input.log_set))
                .collect(),
            outcome: None,
            mask: mask_from_params(&spec.storage.params_json),
            recovery: None,
        }],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SetWriteDiscipline {
    sampling_style: crate::schema_vocab::SamplingStyle,
}

impl Default for SetWriteDiscipline {
    fn default() -> Self {
        // Existing module/equation outputs are continuous. Until SB-DBM-028 verifies regularity,
        // IRREGULAR is the conservative declaration: it promises no increment the writer has not
        // checked, while still enforcing depth uniqueness.
        Self {
            sampling_style: crate::schema_vocab::SamplingStyle::ContinuousIrregular,
        }
    }
}

/// SB-DBM-005: the signed-map citation for a run's method, copied at registration so it
/// travels with the numbers. Non-catalog identities (user equations, TVD materialization,
/// core-photo traces) resolve to `None` - honest absence, never an invented marker.
pub(crate) fn method_derivation_citation(module: &str) -> Option<String> {
    crate::modules::method_derivation_for(module).map(|(_, _, source)| (*source).to_string())
}

/// Stable schema key embedded in `log_sets.params_json` without adding a second write path or
/// changing the deliberately PK-less `computed_curves` table. Existing top-level parameter keys
/// remain readable; the complete record travels with the same log-set row every current/archive
/// curve already cites.
pub(crate) const CURVE_ANCESTRY_KEY: &str = "_sandibumi_curve_ancestry_v1";
// Schema v4 (SB-DBM-015): the re-run manifest arms - depth frame, zone set, stochastic
// identity, applied model, physics attributes - all serde-defaulted so v1..v3 history reads.
pub(crate) const CURVE_ANCESTRY_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AncestryActorKind {
    Human,
    Automated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryActor {
    pub kind: AncestryActorKind,
    /// Explicit session identity. It is never inferred from a Windows account and is separate
    /// from a report's optional "Prepared by" field.
    pub identity: String,
}

/// User-supplied custody attached to a computation request. The backend supplies the timestamp and
/// resolves curve/set identities from the project; the frontend cannot fabricate either. One
/// source/reference note may cover the explicit values in a run, while manifest defaults and
/// stored zone/plot values retain their own more specific sources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunCustody {
    pub actor: AncestryActor,
    pub source_note: String,
}

impl RunCustody {
    pub fn validate(&self) -> Result<(), String> {
        if self.actor.identity.trim().is_empty() {
            return Err("run refused: enter the session operator identity before computing".into());
        }
        if self.source_note.trim().is_empty() {
            return Err(
                "run refused: enter a source/reference note for the explicit run values".into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CurveResolutionRule {
    ExplicitName,
    WorkingInputSet,
    AliasOff,
    AliasManual,
    AliasAutomatic,
    FinalFlag,
    CurveTypeMru,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectedCurveCandidate {
    pub curve_id: String,
    pub log_set: String,
    pub set_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryInput {
    pub well_id: String,
    pub argument: String,
    pub curve: String,
    pub log_set: String,
    pub set_version: Option<i64>,
    pub set_id: String,
    /// The exact stored curve identity that supplied this input. Imported curves use their native
    /// curve UUID; computed curves use the resolvable `computed:<set UUID>:<curve name>` composite
    /// because the current computed store has no standalone curve UUID. Absent only on readable
    /// schema-v1 history written before SB-DBM-006; every schema-v2 writer is fail-closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_curve_id: Option<String>,
    /// The declared resolution stage that selected `chosen_curve_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<CurveResolutionRule>,
    /// Every candidate considered by the same resolver after the winner, in deterministic order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_candidates: Vec<RejectedCurveCandidate>,
}

/// The one controlled vocabulary for provenance that is absent for a known reason. These are
/// states, not substitute values: sample absence remains `f32::NAN`, and a serialization failure
/// remains an error that prevents the run record from being written.
pub use crate::schema_vocab::ProvenanceAbsentState;

pub(crate) const REQUIRED_UNSET_PARAMETER_STATE: &str =
    ProvenanceAbsentState::RequiredUnset.as_str();

pub(crate) fn parameter_state_for(
    parameters: &[AncestryParameter],
) -> Option<ProvenanceAbsentState> {
    parameters.is_empty().then_some(ProvenanceAbsentState::NotApplicable)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParameterResolution {
    Explicit,
    Defaulted,
}

impl ParameterResolution {
    fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "EXPLICIT",
            Self::Defaulted => "DEFAULTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AncestryParameter {
    pub name: String,
    pub value: serde_json::Value,
    pub source: String,
    /// How a declared module parameter obtained its effective value. `None` is retained only for
    /// schema-v1 legacy rows and derived metadata such as `method_id`, neither of which may be
    /// relabelled as a user decision after the fact.
    pub resolution: Option<ParameterResolution>,
    /// Present only for DEFAULTED values and identifies the exact module manifest that supplied
    /// the default. Historical runs therefore cannot be reinterpreted by a later manifest.
    pub manifest_version: Option<String>,
    /// Present only when the corpus records competing positions for this parameter. Optional so
    /// schema-v1 ancestry written before SB-CORE-013 remains readable without being relabelled.
    pub decision: Option<crate::param_sources::ParameterDecision>,
}

#[derive(Serialize, Deserialize)]
struct AncestryParameterWire {
    name: String,
    value: Option<serde_json::Value>,
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state: Option<ProvenanceAbsentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolution: Option<ParameterResolution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    manifest_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decision: Option<crate::param_sources::ParameterDecision>,
}

impl AncestryParameter {
    fn is_required_unset(&self) -> bool {
        self.value.as_str() == Some(crate::modules::ABSENT_DEFAULT_SOURCE)
            && self.source == crate::modules::ABSENT_DEFAULT_SOURCE
    }
}

impl Serialize for AncestryParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let required_unset = self.is_required_unset();
        AncestryParameterWire {
            name: self.name.clone(),
            value: (!required_unset).then(|| self.value.clone()),
            source: (!required_unset).then(|| self.source.clone()),
            state: required_unset.then_some(ProvenanceAbsentState::RequiredUnset),
            resolution: self.resolution,
            manifest_version: self.manifest_version.clone(),
            decision: self.decision.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AncestryParameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = AncestryParameterWire::deserialize(deserializer)?;
        match wire.state {
            Some(ProvenanceAbsentState::RequiredUnset) => {
                if wire.value.is_some()
                    || wire.source.is_some()
                    || wire.resolution.is_some()
                    || wire.manifest_version.is_some()
                {
                    return Err(serde::de::Error::custom(
                        "REQUIRED_UNSET parameter must have null value, source, resolution, and manifest version",
                    ));
                }
                Ok(Self {
                    name: wire.name,
                    value: serde_json::json!(crate::modules::ABSENT_DEFAULT_SOURCE),
                    source: crate::modules::ABSENT_DEFAULT_SOURCE.to_string(),
                    resolution: None,
                    manifest_version: None,
                    decision: wire.decision,
                })
            }
            Some(other) => Err(serde::de::Error::custom(format!(
                "invalid state {other:?} for a named parameter"
            ))),
            None => {
                let value = wire
                    .value
                    .ok_or_else(|| serde::de::Error::custom("sourced parameter is missing value"))?;
                let source = wire
                    .source
                    .ok_or_else(|| serde::de::Error::custom("sourced parameter is missing source"))?;
                match (wire.resolution, wire.manifest_version.as_deref()) {
                    (Some(ParameterResolution::Explicit), Some(_)) => {
                        return Err(serde::de::Error::custom(
                            "EXPLICIT parameter must not name a default manifest version",
                        ));
                    }
                    (Some(ParameterResolution::Defaulted), Some(version))
                        if !version.trim().is_empty() => {}
                    (Some(ParameterResolution::Defaulted), _) => {
                        return Err(serde::de::Error::custom(
                            "DEFAULTED parameter must name a non-empty manifest version",
                        ));
                    }
                    (None, Some(_)) => {
                        return Err(serde::de::Error::custom(
                            "legacy parameter without a resolution cannot name a manifest version",
                        ));
                    }
                    _ => {}
                }
                Ok(Self {
                    name: wire.name,
                    value,
                    source,
                    resolution: wire.resolution,
                    manifest_version: wire.manifest_version,
                    decision: wire.decision,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AncestryZone {
    pub name: String,
    pub top: f32,
    pub base: f32,
    /// Source/reference note for the numeric zone definition. A blank note is not silently
    /// replaced by "operator input" because that would fabricate custody.
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "kind",
    content = "definitions",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum AncestryZoneScope {
    WholeWell,
    Defined(Vec<AncestryZone>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryOutput {
    pub curve: String,
    pub derivation: String,
}

/// Complete SB-CORE-010 record attached to one per-well log-set version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurveAncestry {
    pub schema_version: u32,
    pub module: String,
    pub module_version: String,
    pub inputs: Vec<AncestryInput>,
    pub parameters: Vec<AncestryParameter>,
    /// Present exactly when the current run genuinely has no parameters. Schema-v1/v2 records
    /// omitted this field; readers classify an empty legacy list as `LEGACY_UNRECORDED` rather
    /// than guessing that it meant `NOT_APPLICABLE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_state: Option<ProvenanceAbsentState>,
    pub zone_scope: AncestryZoneScope,
    pub actor: AncestryActor,
    pub timestamp_utc_ms: u64,
    pub outputs: Vec<AncestryOutput>,
    /// SB-DBM-015: the run's depth frame and its sampling - part of what must be pinned
    /// for a byte-identical replay. Absent only on pre-manifest (schema <= 3) records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth_frame: Option<ManifestDepthFrame>,
    /// SB-DBM-015 (DEC-023): the zone-set identity the run saw, where zone-scoped
    /// parameters could apply. A renamed or moved top changes this, and a re-run must say
    /// so rather than silently meaning something different.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_set: Option<ManifestZoneSet>,
    /// SB-DBM-015 (DEC-024): the stochastic-draw identity - what actually determines a
    /// draw, never a run label. None on deterministic runs; SB-DBM-014 inherits this field
    /// when scheduled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stochastic: Option<StochasticIdentity>,
    /// SB-DBM-015 (DEC-024): the `ml_models` identity of any learned model applied. None
    /// for ordinary module runs; SB-DBM-020 inherits this field when scheduled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_model: Option<String>,
    /// SB-DBM-015 (SB-DBM-017): run-time values of attributes that drive physics - e.g.
    /// the declared neutron matrix basis the runner injected. Empty when none applied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub physics_attributes: Vec<PhysicsAttribute>,
    /// SB-DBM-005 (signed derivation map under DEC-076): the method's own derivation
    /// citation, copied from `modules::METHOD_DERIVATIONS` at run registration so it
    /// travels WITH the numbers into every ancestry-carrying deliverable (the LAS
    /// provenance sidecar embeds the whole ancestry, so this field reaches the client
    /// verbatim). `None` on pre-contract history and on non-catalog runs (user
    /// equations, fixtures) - absence is honest there, never an invented marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_derivation: Option<String>,
}

/// SB-DBM-015: the depth frame arm of the re-run manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestDepthFrame {
    pub top: f32,
    pub base: f32,
    pub samples: usize,
}

/// SB-DBM-015 (DEC-023): the zone-set arm - identity and version from `db::current_zone_set`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestZoneSet {
    pub version: i64,
    pub digest: String,
}

/// SB-DBM-015 (DEC-024): the stochastic arm - the facts that determine a draw.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StochasticIdentity {
    pub seed: u64,
    pub scheme: String,
    pub correlations: String,
    pub iterations: u64,
}

/// SB-DBM-015 (SB-DBM-017): one physics-driving attribute value at run time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhysicsAttribute {
    pub name: String,
    pub value: String,
}

impl CurveAncestry {
    fn validate(&self) -> Result<(), String> {
        let required = [
            ("module", self.module.as_str()),
            ("module version", self.module_version.as_str()),
            ("actor identity", self.actor.identity.as_str()),
        ];
        for (field, value) in required {
            if value.trim().is_empty() {
                return Err(format!("complete curve ancestry is missing {field}"));
            }
        }
        if !(1..=CURVE_ANCESTRY_SCHEMA_VERSION).contains(&self.schema_version) {
            return Err(format!(
                "unsupported curve ancestry schema version {}",
                self.schema_version
            ));
        }
        if self.timestamp_utc_ms == 0 {
            return Err("complete curve ancestry is missing its timestamp".into());
        }
        for input in &self.inputs {
            for (field, value) in [
                ("input well identity", input.well_id.as_str()),
                ("input argument", input.argument.as_str()),
                ("input curve", input.curve.as_str()),
                ("input log set", input.log_set.as_str()),
                ("input set identity", input.set_id.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(format!("complete curve ancestry is missing {field}"));
                }
            }
            if input.set_version.is_some_and(|version| version < 1) {
                return Err(format!(
                    "input '{}' has an invalid log-set version",
                    input.curve
                ));
            }
            match (input.chosen_curve_id.as_deref(), input.rule.as_ref()) {
                (Some(curve_id), Some(_)) if !curve_id.trim().is_empty() => {}
                (None, None) if self.schema_version == 1 => {}
                (Some(_), None) | (None, Some(_)) => {
                    return Err(format!(
                        "input '{}' has an incomplete curve-resolution decision",
                        input.curve
                    ));
                }
                _ => {
                    return Err(format!(
                        "input '{}' has no chosen curve identity",
                        input.curve
                    ));
                }
            }
            let chosen = input.chosen_curve_id.as_deref();
            if chosen.is_none() && !input.rejected_candidates.is_empty() {
                return Err(format!(
                    "input '{}' has rejected candidates without a chosen curve identity",
                    input.curve
                ));
            }
            let mut rejected_ids = std::collections::HashSet::new();
            for candidate in &input.rejected_candidates {
                if candidate.curve_id.trim().is_empty() || candidate.log_set.trim().is_empty() {
                    return Err(format!(
                        "input '{}' has an incomplete rejected curve identity",
                        input.curve
                    ));
                }
                if candidate.set_version.is_some_and(|version| version < 1) {
                    return Err(format!(
                        "input '{}' has a rejected candidate with an invalid set version",
                        input.curve
                    ));
                }
                if chosen == Some(candidate.curve_id.as_str()) {
                    return Err(format!(
                        "input '{}' lists its chosen curve as rejected",
                        input.curve
                    ));
                }
                if !rejected_ids.insert(candidate.curve_id.as_str()) {
                    return Err(format!(
                        "input '{}' repeats a rejected curve identity",
                        input.curve
                    ));
                }
            }
        }
        for parameter in &self.parameters {
            if parameter.name.trim().is_empty() {
                return Err("complete curve ancestry contains an unnamed parameter".into());
            }
            if parameter.is_required_unset() {
                if parameter.resolution.is_some() || parameter.manifest_version.is_some() {
                    return Err(format!(
                        "parameter '{}' has provenance on a REQUIRED_UNSET state",
                        parameter.name
                    ));
                }
                continue;
            }
            if parameter.source == crate::modules::ABSENT_DEFAULT_SOURCE {
                return Err(format!(
                    "parameter '{}' has an incomplete REQUIRED_UNSET state",
                    parameter.name
                ));
            }
            if parameter.value.is_null() {
                return Err(format!(
                    "parameter '{}' has no recorded value",
                    parameter.name
                ));
            }
            if parameter.source.trim().is_empty() {
                return Err(format!(
                    "parameter '{}' has no source string",
                    parameter.name
                ));
            }
            match (parameter.resolution, parameter.manifest_version.as_deref()) {
                (Some(ParameterResolution::Explicit), Some(_)) => {
                    return Err(format!(
                        "explicit parameter '{}' names a default manifest version",
                        parameter.name
                    ));
                }
                (Some(ParameterResolution::Defaulted), Some(version))
                    if !version.trim().is_empty() => {}
                (Some(ParameterResolution::Defaulted), _) => {
                    return Err(format!(
                        "defaulted parameter '{}' has no manifest version",
                        parameter.name
                    ));
                }
                (None, Some(_)) => {
                    return Err(format!(
                        "legacy parameter '{}' names a manifest version without a resolution",
                        parameter.name
                    ));
                }
                _ => {}
            }
            if parameter
                .value
                .as_f64()
                .is_some_and(|value| !value.is_finite())
            {
                return Err(format!(
                    "parameter '{}' has a non-finite recorded value",
                    parameter.name
                ));
            }
        }
        match (self.parameters.is_empty(), self.parameter_state) {
            (true, Some(ProvenanceAbsentState::NotApplicable)) => {}
            (true, Some(ProvenanceAbsentState::LegacyUnrecorded)) if self.schema_version < 3 => {}
            (true, None) if self.schema_version < 3 => {}
            (true, Some(state)) => {
                return Err(format!(
                    "an empty parameter set has invalid provenance state {state:?}"
                ));
            }
            (true, None) => {
                return Err(
                    "a current empty parameter set must be named NOT_APPLICABLE".into(),
                );
            }
            (false, None) => {}
            (false, Some(state)) => {
                return Err(format!(
                    "a populated parameter set must not also claim absent state {state:?}"
                ));
            }
        }
        if let AncestryZoneScope::Defined(zones) = &self.zone_scope {
            if zones.is_empty() {
                return Err("defined zone ancestry contains no zone definitions".into());
            }
            for zone in zones {
                if zone.name.trim().is_empty()
                    || !zone.top.is_finite()
                    || !zone.base.is_finite()
                    || zone.top >= zone.base
                {
                    return Err(
                        "complete curve ancestry contains an invalid zone definition".into(),
                    );
                }
                if zone.source.trim().is_empty() {
                    return Err(format!("zone '{}' has no source string", zone.name));
                }
            }
        }
        if self.outputs.is_empty() {
            return Err("complete curve ancestry has no output derivations".into());
        }
        for output in &self.outputs {
            if output.curve.trim().is_empty() || output.derivation.trim().is_empty() {
                return Err(
                    "complete curve ancestry contains an incomplete output derivation".into(),
                );
            }
        }
        Ok(())
    }

    /// Whether two records describe the same deterministic computation. The timestamp
    /// identifies the event and is intentionally excluded; every scientifically material
    /// input, value/source, zone/source, actor, output, and implementation identity remains.
    pub(crate) fn same_computation(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.module == other.module
            && self.module_version == other.module_version
            && self.inputs == other.inputs
            && self.parameters == other.parameters
            && self.parameter_state == other.parameter_state
            && self.zone_scope == other.zone_scope
            && self.actor == other.actor
            && self.outputs == other.outputs
    }
}

pub(crate) fn ancestry_timestamp_utc_ms() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("cannot record curve ancestry timestamp: {error}"))?
        .as_millis()
        .try_into()
        .map_err(|_| "curve ancestry timestamp exceeds u64".to_string())
}

/// Resolves one effective input using the same precedence as `fetch_curve_frame`: the current
/// chain/run set, then an explicitly named input set, then the current computed store, then the
/// imported generic store. A standard-only legacy project is migrated through the existing
/// idempotent generic-store migration before the final lookup; no invented RAW identity is used.
pub(crate) fn try_resolve_ancestry_input(
    conn: &Connection,
    well_id: &str,
    argument: &str,
    curve: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<Option<AncestryInput>, String> {
    let upper = curve.trim().to_uppercase();
    let computed_curve_id = |set_id: &str| format!("computed:{set_id}:{upper}");
    let from_log_set = |
        set_id: &str,
        rule: CurveResolutionRule,
        rejected_candidates: Vec<RejectedCurveCandidate>,
    | -> Result<AncestryInput, String> {
        let (set_name, version): (String, i64) = conn
            .query_row(
                "SELECT set_name, version FROM log_sets WHERE set_id = ?1",
                params![set_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| {
                format!("input curve '{curve}' cites a missing log-set record: {error}")
            })?;
        Ok(AncestryInput {
            well_id: well_id.to_string(),
            argument: argument.to_string(),
            curve: upper.clone(),
            log_set: set_name,
            set_version: Some(version),
            set_id: set_id.to_string(),
            chosen_curve_id: Some(computed_curve_id(set_id)),
            rule: Some(rule),
            rejected_candidates,
        })
    };

    if let Some(set_id) = own_set_id {
        let found = conn
            .query_row(
                "SELECT 1 FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2 LIMIT 1",
                params![set_id, upper],
                |_| Ok(()),
            )
            .is_ok();
        if found {
            return from_log_set(
                set_id,
                CurveResolutionRule::WorkingInputSet,
                Vec::new(),
            )
            .map(Some);
        }
    }
    if let Some(set_name) = input_set.map(str::trim).filter(|value| !value.is_empty()) {
        let selected: Vec<(String, String, i64)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT s.set_id, s.set_name, s.version FROM log_sets s
                     WHERE s.well_id = ?1 AND upper(s.set_name) = upper(?2)
                       AND EXISTS (SELECT 1 FROM computed_curves_archive a
                                   WHERE a.set_id = s.set_id AND upper(a.curve_name) = ?3)
                     ORDER BY s.version DESC, s.set_id",
                )
                .map_err(|error| error.to_string())?;
            stmt.query_map(params![well_id, set_name, upper], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<duckdb::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?
        };
        if let Some((set_id, _, _)) = selected.first() {
            let rejected_candidates = selected[1..]
                .iter()
                .map(|(candidate_id, candidate_set, version)| RejectedCurveCandidate {
                    curve_id: computed_curve_id(candidate_id),
                    log_set: candidate_set.clone(),
                    set_version: Some(*version),
                })
                .collect();
            return from_log_set(
                set_id,
                CurveResolutionRule::WorkingInputSet,
                rejected_candidates,
            )
            .map(Some);
        }
    }

    let set_ids: Vec<Option<String>> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves
                 WHERE well_id = ?1 AND upper(curve_name) = ?2",
            )
            .map_err(|error| error.to_string())?;
        stmt.query_map(params![well_id, upper], |row| row.get(0))
            .map_err(|error| error.to_string())?
            .collect::<duckdb::Result<_>>()
            .map_err(|error| error.to_string())?
    };
    if !set_ids.is_empty() {
        if set_ids.len() != 1 || set_ids[0].is_none() {
            return Err(format!(
                "input computed curve '{curve}' has no single live ancestry record"
            ));
        }
        return from_log_set(
            set_ids[0].as_deref().expect("checked above"),
            CurveResolutionRule::ExplicitName,
            Vec::new(),
        )
        .map(Some);
    }

    // SB-DIO-034: provenance must resolve with the SAME request type the reader used
    // (SemanticFamily), or a run could record one curve while calculating from another.
    //
    // #129: there is deliberately NO repair here. A curve missing from the generic store used to
    // trigger `db::migrate_standard_curves_to_generic_store` - the whole project's legacy
    // back-fill, a WRITE - from inside this read, and then retry. Behind the one shared connection
    // that ran once, committed, and was invisible; on N reader connections, N rayon threads each
    // ran the whole back-fill and collided on `curve_meta`'s primary key, which is what broke the
    // connection pool. `PERF-ATTEMPTS.md` §4 has the bisect that named it.
    //
    // The back-fill belongs to the open, and already runs there: `project::open_and_migrate` is
    // the route every production open takes, and it runs the back-fill before any well can be
    // read. A LAS import marks its own wells done inside the import transaction, so a well that
    // arrives mid-session is not owed one either. A curve still absent after that is absent - this
    // returns None, the caller tries the next alias, and an input that resolves nowhere is a named
    // missing-input error rather than a silent substitution.
    let imported =
        resolve_generic_curve_decision(conn, well_id, &upper, CurveRequest::SemanticFamily)
            .map_err(|error| error.to_string())?;
    let Some(GenericCurveDecision {
        chosen,
        rule,
        rejected,
    }) = imported
    else {
        return Ok(None);
    };
    let rule = rule.ok_or_else(|| {
        format!(
            "input curve '{curve}' has tied legacy candidates with no recorded modification order"
        )
    })?;
    let rejected_candidates = rejected
        .into_iter()
        .map(|candidate| RejectedCurveCandidate {
            curve_id: candidate.curve_id,
            log_set: candidate.set_name,
            set_version: Some(candidate.set_version),
        })
        .collect();
    let chosen_curve_id = chosen.curve_id.clone();
    Ok(Some(AncestryInput {
        well_id: well_id.to_string(),
        argument: argument.to_string(),
        curve: upper,
        log_set: chosen.set_name,
        set_version: Some(chosen.set_version),
        set_id: chosen_curve_id.clone(),
        chosen_curve_id: Some(chosen_curve_id),
        rule: Some(rule),
        rejected_candidates,
    }))
}

/// [`resolve_ancestry_input`] for many requests at once, as a BATCH FAST PATH over the single
/// dominant case, with the per-call function itself as the fallback for everything else.
///
/// It is deliberately not a re-implementation. `try_resolve_ancestry_input` has four resolution
/// paths in priority order - a working set, a named input set, an explicitly named computed curve,
/// and the generic store - and a batch that reproduced all four from scratch would be four chances
/// to diverge in a PROVENANCE record, which is the kind of wrong answer that computes, plots and
/// ships. So this collapses only the third, which is the one a chain-fed run takes, and hands
/// anything else to the original function unchanged.
///
/// The fast path fires ONLY when all of these hold, and each is the condition under which the
/// per-call function is known to return exactly this value:
///
/// - no `input_set` is in force, so the higher-priority paths cannot apply (the working-set path
///   needs an `own_set_id`, which this entry point does not take at all);
/// - the pair resolves to EXACTLY ONE non-NULL `set_id` in `computed_curves`;
/// - that set's `log_sets` row exists.
///
/// Zero set ids means the generic-store path, more than one (or a NULL one) means the per-call
/// function raises a named error, and a missing `log_sets` row means it raises a different one.
/// All three fall back rather than being reproduced, so the messages stay identical too.
///
/// Order is preserved: the returned vector matches `requests` element for element.
///
/// Pinned against the per-call function by
/// `the_batched_input_resolution_answers_exactly_what_asking_one_at_a_time_answers`.
pub(crate) fn resolve_ancestry_inputs_batch(
    conn: &Connection,
    requests: &[(String, String, String)],
    input_set: Option<&str>,
) -> Result<Vec<AncestryInput>, String> {
    let named_set = input_set.map(str::trim).filter(|value| !value.is_empty());
    if requests.is_empty() || named_set.is_some() {
        // A named input set outranks everything this batches, so there is nothing to collapse.
        return requests
            .iter()
            .map(|(well_id, argument, curve)| {
                resolve_ancestry_input(conn, well_id, argument, curve, input_set, None)
            })
            .collect();
    }

    let mut wells: Vec<String> = requests.iter().map(|(well, _, _)| well.clone()).collect();
    wells.sort_unstable();
    wells.dedup();
    let mut curves: Vec<String> =
        requests.iter().map(|(_, _, curve)| curve.trim().to_uppercase()).collect();
    curves.sort_unstable();
    curves.dedup();

    // The per-call form's third path asks `WHERE well_id = ? AND upper(curve_name) = ?`, so
    // grouping by `upper(curve_name)` here is what it already does - unlike `curve_ancestry_batch`
    // above, which must group by the RAW name to keep a mixed-case well a refusal.
    let mut set_ids: HashMap<(String, String), Vec<Option<String>>> = HashMap::new();
    {
        let wph = std::iter::repeat("?").take(wells.len()).collect::<Vec<_>>().join(", ");
        let cph = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
        let mut binds: Vec<String> = Vec::with_capacity(wells.len() + curves.len());
        binds.extend(wells.iter().cloned());
        binds.extend(curves.iter().cloned());
        let mut stmt = conn
            .prepare(&format!(
                "SELECT well_id, upper(curve_name), CAST(set_id AS VARCHAR) FROM computed_curves
                 WHERE well_id IN ({wph}) AND upper(curve_name) IN ({cph})
                 GROUP BY well_id, upper(curve_name), set_id"
            ))
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(binds), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (well_id, curve, set_id) = row.map_err(|error| error.to_string())?;
            set_ids.entry((well_id, curve)).or_default().push(set_id);
        }
    }

    let mut records: HashMap<String, (String, i64)> = HashMap::new();
    let wanted: Vec<String> = set_ids
        .values()
        .filter(|found| found.len() == 1)
        .filter_map(|found| found[0].clone())
        .collect();
    if !wanted.is_empty() {
        let sph = std::iter::repeat("?").take(wanted.len()).collect::<Vec<_>>().join(", ");
        let mut stmt = conn
            .prepare(&format!(
                "SELECT CAST(set_id AS VARCHAR), set_name, version FROM log_sets
                 WHERE CAST(set_id AS VARCHAR) IN ({sph})"
            ))
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(wanted), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (set_id, set_name, version) = row.map_err(|error| error.to_string())?;
            records.insert(set_id, (set_name, version));
        }
    }

    requests
        .iter()
        .map(|(well_id, argument, curve)| {
            let upper = curve.trim().to_uppercase();
            let found = set_ids.get(&(well_id.clone(), upper.clone()));
            let fast = found
                .filter(|found| found.len() == 1)
                .and_then(|found| found[0].as_deref())
                .and_then(|set_id| records.get(set_id).map(|record| (set_id, record)));
            match fast {
                Some((set_id, (set_name, version))) => Ok(AncestryInput {
                    well_id: well_id.clone(),
                    argument: argument.clone(),
                    curve: upper.clone(),
                    log_set: set_name.clone(),
                    set_version: Some(*version),
                    set_id: set_id.to_string(),
                    chosen_curve_id: Some(format!("computed:{set_id}:{upper}")),
                    rule: Some(CurveResolutionRule::ExplicitName),
                    rejected_candidates: Vec::new(),
                }),
                None => resolve_ancestry_input(conn, well_id, argument, curve, input_set, None),
            }
        })
        .collect()
}

/// Strict ancestry resolution for a curve that materially participated in a computation.
/// Optional module inputs use [`try_resolve_ancestry_input`] so a declared-but-absent input is
/// not falsely recorded as project data; a present curve with malformed ancestry still errors.
pub(crate) fn resolve_ancestry_input(
    conn: &Connection,
    well_id: &str,
    argument: &str,
    curve: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<AncestryInput, String> {
    try_resolve_ancestry_input(conn, well_id, argument, curve, input_set, own_set_id)?
        .ok_or_else(|| format!("input curve '{curve}' has no resolvable log-set identity"))
}

/// A log-set specification that cannot be constructed until the complete record validates.
/// Production writers accept this type, not raw JSON strings.
#[derive(Debug, Clone)]
pub struct CompleteLogSetSpec {
    pub(crate) storage: LogSetSpec,
    pub(crate) ancestry: CurveAncestry,
    discipline: SetWriteDiscipline,
}

/// The three refusals a [`CompleteLogSetSpec::restamp_ancestry`] caller carries in with it.
/// Passed rather than shared: each names the work that was in progress, and one common wording
/// would tell a user the equation engine failed when the pay-summary engine did.
#[derive(Clone, Copy)]
pub(crate) struct RestampMessages {
    pub(crate) parse: &'static str,
    pub(crate) non_object: &'static str,
    pub(crate) serialize: &'static str,
}

impl CompleteLogSetSpec {
    /// AUDIT-2026-08-20 finding 77: validate the ancestry, parse the stored parameter record,
    /// insert the manifest under `CURVE_ANCESTRY_KEY`, re-stringify. Three producers each carried
    /// their own copy of this, and it is the ORDER that is load-bearing rather than the lines: a
    /// copy that validated AFTER inserting, or re-stringified before it, would store a manifest
    /// that never passed validation - and every reader downstream would take it as one that had.
    fn restamp_ancestry(&mut self, messages: RestampMessages) -> Result<(), String> {
        self.ancestry.validate()?;
        let mut stored: serde_json::Value = serde_json::from_str(&self.storage.params_json)
            .map_err(|error| format!("{}: {error}", messages.parse))?;
        let object = stored
            .as_object_mut()
            .ok_or_else(|| messages.non_object.to_string())?;
        object.insert(
            CURVE_ANCESTRY_KEY.into(),
            serde_json::to_value(&self.ancestry)
                .map_err(|error| format!("{}: {error}", messages.serialize))?,
        );
        self.storage.params_json = stored.to_string();
        Ok(())
    }

    #[cfg(test)]
    pub fn try_new(set_name: &str, ancestry: CurveAncestry) -> Result<Self, String> {
        Self::try_new_with_legacy(
            set_name,
            ancestry,
            serde_json::Value::Object(Default::default()),
            "[]",
        )
    }

    pub fn try_new_with_legacy(
        set_name: &str,
        ancestry: CurveAncestry,
        legacy_parameters: serde_json::Value,
        legacy_inputs_json: &str,
    ) -> Result<Self, String> {
        ancestry.validate()?;
        if set_name.trim().is_empty() {
            return Err("complete curve ancestry is missing its output log-set name".into());
        }
        let mut parameters = match legacy_parameters {
            serde_json::Value::Object(map) => map,
            other => {
                let mut map = serde_json::Map::new();
                map.insert("legacy_parameters".into(), other);
                map
            }
        };
        if parameters.contains_key(CURVE_ANCESTRY_KEY) {
            return Err(format!(
                "legacy parameters may not replace reserved key '{CURVE_ANCESTRY_KEY}'"
            ));
        }
        parameters.insert(
            CURVE_ANCESTRY_KEY.into(),
            serde_json::to_value(&ancestry)
                .map_err(|error| format!("cannot serialize curve ancestry: {error}"))?,
        );
        let inputs: serde_json::Value = serde_json::from_str(legacy_inputs_json)
            .map_err(|error| format!("cannot record invalid input JSON: {error}"))?;
        let storage = LogSetSpec {
            set_name: set_name.trim().to_string(),
            module: ancestry.module.clone(),
            params_json: serde_json::Value::Object(parameters).to_string(),
            inputs_json: inputs.to_string(),
        };
        Ok(Self {
            storage,
            ancestry,
            discipline: SetWriteDiscipline::default(),
        })
    }

    pub fn ancestry(&self) -> &CurveAncestry {
        &self.ancestry
    }

    #[cfg(test)]
    pub(crate) fn with_sampling_style(
        mut self,
        sampling_style: crate::schema_vocab::SamplingStyle,
    ) -> Self {
        self.discipline = SetWriteDiscipline {
            sampling_style,
        };
        self
    }

    /// Attach source-comparison decisions to named parameters and refresh the already-validated
    /// serialized ancestry in the storage payload. This keeps specialized producers such as the
    /// pay-summary engine on the same whitelisted complete-record path; it does not create a second
    /// writer or any duplicate-tolerant database behavior.
    /// SB-DBM-015: record the manifest arms the spec builder cannot know - the depth frame
    /// exists only once the runner has fetched the well, and the physics-driving attribute
    /// values are the ones the runner actually injected. Mutates the ancestry AND
    /// re-serializes the stored record, exactly as `record_parameter_decisions` does, so the
    /// stored manifest and the validated ancestry cannot drift.
    pub(crate) fn record_run_manifest(
        &mut self,
        depth_frame: Option<ManifestDepthFrame>,
        physics_attributes: Vec<PhysicsAttribute>,
    ) -> Result<(), String> {
        if depth_frame.is_some() {
            self.ancestry.depth_frame = depth_frame;
        }
        if !physics_attributes.is_empty() {
            self.ancestry.physics_attributes = physics_attributes;
        }
        self.restamp_ancestry(RestampMessages {
            parse: "cannot refresh curve ancestry manifest JSON",
            non_object: "cannot refresh curve ancestry in a non-object parameter record",
            serialize: "cannot serialize curve ancestry manifest",
        })
    }

    pub(crate) fn record_parameter_decisions(
        &mut self,
        topics: &[(&str, &str)],
    ) -> Result<(), String> {
        for parameter in &mut self.ancestry.parameters {
            if let Some((_, topic)) = topics
                .iter()
                .find(|(name, _)| parameter.name.eq_ignore_ascii_case(name))
            {
                parameter.decision = crate::param_sources::decision_for(topic, &parameter.value);
            }
        }
        self.restamp_ancestry(RestampMessages {
            parse: "cannot refresh curve ancestry parameter JSON",
            non_object: "cannot refresh curve ancestry in a non-object parameter record",
            serialize: "cannot serialize curve ancestry decision",
        })
    }

    /// Retain non-parameter run metadata in the legacy payload while naming the canonical
    /// parameter collection as genuinely not applicable. This is used by user equations: their
    /// definition is provenance, but it is not a configurable petrophysical parameter set.
    pub(crate) fn record_parameters_not_applicable(&mut self) -> Result<(), String> {
        self.ancestry.parameters.clear();
        self.ancestry.parameter_state = Some(ProvenanceAbsentState::NotApplicable);
        self.restamp_ancestry(RestampMessages {
            parse: "cannot name the equation parameter state",
            non_object: "cannot name parameters in a non-object provenance record",
            serialize: "cannot serialize the equation parameter state",
        })
    }
}

/// Builds the complete record for a run whose inputs are project curves and whose explicit
/// controls share one user-supplied source/reference note. More specialized producers (for
/// example, a photograph or deviation survey) construct [`CurveAncestry`] directly so their
/// non-curve input identity is recorded truthfully instead of being disguised as a log curve.
pub(crate) fn complete_curve_run_spec(
    conn: &Connection,
    output_well_id: &str,
    set_name: &str,
    module: &str,
    custody: &RunCustody,
    inputs: &[(String, String, String)],
    input_set: Option<&str>,
    legacy_parameters: serde_json::Value,
    zone_scope: AncestryZoneScope,
    outputs: &[String],
) -> Result<CompleteLogSetSpec, String> {
    custody.validate()?;
    if output_well_id.trim().is_empty() {
        return Err("complete curve ancestry is missing its output well identity".into());
    }
    let resolved_inputs = inputs
        .iter()
        .map(|(well_id, argument, curve)| {
            resolve_ancestry_input(conn, well_id, argument, curve, input_set, None)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let legacy_input_curves: Vec<String> =
        inputs.iter().map(|(_, _, curve)| curve.clone()).collect();
    complete_curve_run_spec_resolved(
        output_well_id,
        set_name,
        module,
        custody,
        resolved_inputs,
        &legacy_input_curves,
        legacy_parameters,
        zone_scope,
        outputs,
    )
}

/// [`complete_curve_run_spec`] with its inputs ALREADY resolved, so a caller that resolved a whole
/// field's worth in one batch does not resolve them a second time one at a time.
///
/// This is the same body, not a second copy of it - `complete_curve_run_spec` resolves and then
/// delegates here, so there is exactly one place that builds a `CurveAncestry` from inputs. It
/// re-validates custody and the output well identity rather than trusting the caller: the wrapper
/// checks them BEFORE resolving so that an invalid custody still reports itself first, and a direct
/// caller must not be able to skip the check by taking the shorter door.
pub(crate) fn complete_curve_run_spec_resolved(
    output_well_id: &str,
    set_name: &str,
    module: &str,
    custody: &RunCustody,
    resolved_inputs: Vec<AncestryInput>,
    legacy_input_curves: &[String],
    legacy_parameters: serde_json::Value,
    zone_scope: AncestryZoneScope,
    outputs: &[String],
) -> Result<CompleteLogSetSpec, String> {
    custody.validate()?;
    if output_well_id.trim().is_empty() {
        return Err("complete curve ancestry is missing its output well identity".into());
    }
    let parameters = match &legacy_parameters {
        serde_json::Value::Object(values) => values
            .iter()
            .map(|(name, value)| AncestryParameter {
                name: name.clone(),
                value: if value.is_null() {
                    serde_json::json!("ABSENT")
                } else {
                    value.clone()
                },
                source: custody.source_note.trim().to_string(),
                resolution: Some(ParameterResolution::Explicit),
                manifest_version: None,
                decision: None,
            })
            .collect(),
        serde_json::Value::Null => Vec::new(),
        value => vec![AncestryParameter {
            name: "request".into(),
            value: value.clone(),
            source: custody.source_note.trim().to_string(),
            resolution: Some(ParameterResolution::Explicit),
            manifest_version: None,
            decision: None,
        }],
    };
    let parameter_state = parameter_state_for(&parameters);
    let ancestry = CurveAncestry {
        schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
        method_derivation: method_derivation_citation(module.trim()),
        module: module.trim().to_string(),
        // SB-DBM-002 (DEC-021): equation runs carry the equation engine's own source digest.
        module_version: format!("src:{}", crate::modules::module_source_digest("equation:run")),
        inputs: resolved_inputs,
        parameters,
        parameter_state,
        zone_scope,
        actor: custody.actor.clone(),
        timestamp_utc_ms: ancestry_timestamp_utc_ms()?,
        outputs: outputs
            .iter()
            .map(|curve| AncestryOutput {
                curve: curve.clone(),
                derivation: format!("{}:{}", module.trim(), curve),
            })
            .collect(),
        // SB-DBM-015: the frame is recorded by the runner once the depth exists (see
        // CompleteLogSetSpec::record_run_manifest); the seams stay None until the deferred
        // rows that own them are scheduled (DEC-024).
        depth_frame: None,
        zone_set: None,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    };
    let legacy_inputs =
        serde_json::to_string(&legacy_input_curves.iter().collect::<Vec<_>>())
            .map_err(|error| format!("cannot record run inputs: {error}"))?;
    CompleteLogSetSpec::try_new_with_legacy(set_name, ancestry, legacy_parameters, &legacy_inputs)
}

/// Opaque proof that a stored set has a complete, validated record. No production writer accepts
/// an arbitrary string as a set id.
#[derive(Debug, Clone)]
pub struct CompleteSetId {
    value: String,
    well_id: String,
    outputs: Vec<String>,
}

impl CompleteSetId {
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

pub(crate) struct CompleteWellLogSet {
    pub well_id: String,
    pub spec: CompleteLogSetSpec,
}

pub(crate) struct CompleteWellWrite {
    pub well_id: String,
    pub depth: Vec<f32>,
    pub curves: Vec<(String, Vec<f32>)>,
    pub set_id: CompleteSetId,
    /// The module step that produced these events. A chain's log-set module names the whole
    /// workflow, so this field preserves which individual step degraded the well.
    ///
    /// `None` means this caller does not classify its runs, and Phase 4 leaves `outcome_state`
    /// alone. It is not an oversight to be tidied into `Some(String::new())`: the pay summary has
    /// no degradation vocabulary and its single-well write has never classified a PAYFLAG version,
    /// so batching it had to keep that true or the speed-up would have quietly started marking
    /// those versions CLEAN in the catalog. A run that HAS a degradation vocabulary passes `Some`
    /// and is classified exactly as before.
    pub degradation_module: Option<String>,
    pub degradations: Option<Vec<crate::modules::RunDegradation>>,
}

/// One version row of `log_sets`, named field by field.
///
/// AUDIT-2026-08-20 finding 70. The INSERT was typed out FOUR times, and two of the four had
/// stopped agreeing on which columns they wrote. One divergence was deliberate and carried its
/// reason; the other carried nothing, and nobody could tell them apart, because an omitted
/// column and a deliberately NULL column look identical in SQL. Naming every field makes the
/// difference visible: a writer that means NULL now says `None` where the reader can see it.
pub(crate) struct LogSetRow<'a> {
    pub set_id: &'a str,
    pub well_id: &'a str,
    pub set_name: &'a str,
    pub version: i64,
    pub module: &'a str,
    pub params_json: Option<&'a str>,
    pub inputs_json: Option<&'a str>,
    /// `LogSetFrame` as stored: STANDARD (the well's own grid) or OWN (the set carries depths).
    pub frame: &'a str,
    pub sampling_style: Option<&'a str>,
    pub duplicate_resolution: Option<&'a str>,
    pub outcome_state: Option<&'a str>,
    /// SB-ENV-005. `None` writes SQL NULL, which the reader returns as UNKNOWN - "the step
    /// history cannot be recovered", never an empty step list. A writer that passes `None` is
    /// making that statement on purpose and must say why.
    pub applied_steps_json: Option<&'a str>,
}

/// The ONE place a `log_sets` version row is written. Every allocator goes through it, so a new
/// column reaches every writer or none.
pub(crate) fn insert_log_set(conn: &Connection, row: LogSetRow<'_>) -> duckdb::Result<()> {
    conn.execute(
        "INSERT INTO log_sets
            (set_id, well_id, set_name, version, module, params_json, inputs_json, frame,
             sampling_style, duplicate_resolution, outcome_state, applied_steps_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            row.set_id,
            row.well_id,
            row.set_name,
            row.version,
            row.module,
            row.params_json,
            row.inputs_json,
            row.frame,
            row.sampling_style,
            row.duplicate_resolution,
            row.outcome_state,
            row.applied_steps_json,
        ],
    )?;
    Ok(())
}

pub(crate) const LOG_SET_RESTORE_KEY: &str = "_sandibumi_restore_v1";
pub(crate) const RUN_OUTCOME_CLEAN: &str = "CLEAN";
pub(crate) const RUN_OUTCOME_DEGRADED: &str = "DEGRADED";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StoredRunDegradation {
    pub module: String,
    pub kind: crate::modules::RunDegradationKind,
    pub detail: String,
    pub occurrences: usize,
}

/// Structured link carried by a restore run. The restored version keeps the source calculation's
/// ancestry and parameters, while this record says which immutable historical version supplied
/// its rows. A second restore points at the version it actually copied; history is never rewound.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogSetRestoreRecord {
    pub schema_version: u32,
    pub source_set_id: String,
    pub source_version: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreLogSetResult {
    pub rows_restored: usize,
    pub new_set_id: String,
    pub new_version: i64,
    pub restored_from: LogSetRestoreRecord,
}

/// One version of a log set as listed in the catalog / Sets manager.
#[derive(Debug, Clone, Serialize)]
pub struct LogSetEntry {
    pub set_id: String,
    pub set_name: String,
    pub version: i64,
    pub module: String,
    pub params_json: Option<String>,
    pub inputs_json: Option<String>,
    pub created_at: String,
    pub curve_names: Vec<String>,
    pub is_current: bool,
    pub ancestry: Option<CurveAncestry>,
    pub restored_from: Option<LogSetRestoreRecord>,
    /// `None` is a pre-contract run whose result cannot honestly be classified after the fact.
    pub outcome_state: Option<String>,
    pub degradations: Vec<StoredRunDegradation>,
    /// DEC-045/DEC-039: this version's own free-text comment. Versions never inherit it.
    pub comment: Option<String>,
}

fn restore_record(params_json: Option<&str>) -> Option<LogSetRestoreRecord> {
    let value: serde_json::Value = serde_json::from_str(params_json?).ok()?;
    serde_json::from_value(value.get(LOG_SET_RESTORE_KEY)?.clone()).ok()
}

fn params_json_with_restore_record(
    params_json: Option<&str>,
    restored_from: &LogSetRestoreRecord,
) -> Result<String, String> {
    let mut parameters = match params_json {
        None => serde_json::Map::new(),
        Some(raw) => match serde_json::from_str::<serde_json::Value>(raw)
            .map_err(|error| format!("cannot restore a run with invalid parameter JSON: {error}"))?
        {
            serde_json::Value::Object(map) => map,
            legacy => {
                let mut map = serde_json::Map::new();
                map.insert("legacy_parameters".into(), legacy);
                map
            }
        },
    };
    parameters.insert(
        LOG_SET_RESTORE_KEY.into(),
        serde_json::to_value(restored_from)
            .map_err(|error| format!("cannot serialize restore provenance: {error}"))?,
    );
    Ok(serde_json::Value::Object(parameters).to_string())
}

/// DEC-045/DEC-039 (SB-POR-003/026/028/047/048's shared seam): record the per-VERSION free-text
/// comment — the branch a module took and every limit that bound, stated as text rather than as an
/// enumerated vocabulary, which is exactly what the DEC-039 ruling replaced the categorical stream
/// with. One comment describes ONE run: writing to a version never touches any other version, and
/// an empty text is refused — "no comment" is the NULL the row already has, never an empty string
/// that reads as a recorded nothing.
pub(crate) fn set_log_set_comment(
    conn: &Connection,
    set_id: &str,
    comment: &str,
) -> Result<(), String> {
    if comment.trim().is_empty() {
        return Err("a version comment must say something; absence is the NULL it already has".into());
    }
    let updated = conn
        .execute(
            "UPDATE log_sets SET comment = ?2 WHERE set_id = ?1",
            params![set_id, comment],
        )
        .map_err(|error| error.to_string())?;
    if updated == 0 {
        return Err(format!("no log-set version {set_id} to comment"));
    }
    Ok(())
}

/// SB-ENV-005 retrieval: the manifest for one log-set version, without re-running
/// anything - the manifest is the record, not a recipe re-executed. Three honest
/// answers: the manifest; UNKNOWN for a pre-contract NULL (never an empty step
/// list); a refusal naming the schema version this build does not know, while the
/// version's curves still read (nothing else consults the column).
pub(crate) fn get_applied_steps(
    conn: &Connection,
    set_id: &str,
) -> Result<AppliedStepsRecord, String> {
    let stored: Option<Option<String>> = conn
        .query_row(
            "SELECT applied_steps_json FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(stored) = stored else {
        return Err(format!("no log-set version {set_id}: a manifest exists only on the version row it describes"));
    };
    let Some(json) = stored else {
        return Ok(AppliedStepsRecord::Unknown);
    };
    let manifest: AppliedStepsManifest = serde_json::from_str(&json)
        .map_err(|error| format!("applied-step manifest on version {set_id} does not parse: {error}"))?;
    if manifest.v != APPLIED_STEPS_SCHEMA_VERSION {
        return Err(format!(
            "applied-step manifest on version {set_id} carries schema v{}, and this build reads \
             v{APPLIED_STEPS_SCHEMA_VERSION}: the step history is refused rather than misread; \
             the version's curves are unaffected (SB-ENV-005)",
            manifest.v
        ));
    }
    Ok(AppliedStepsRecord::Manifest { manifest })
}

/// Registers a new run event: version = 1 + the well's highest version of `set_name`
/// (so a re-run NEVER replaces — it becomes version N+1). Returns (set_id, version).
fn create_log_set_raw(
    conn: &Connection,
    well_id: &str,
    spec: &LogSetSpec,
    discipline: SetWriteDiscipline,
    outcome_state: Option<&str>,
    applied_steps_json: Option<&str>,
) -> duckdb::Result<(String, i64)> {
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
        params![well_id, spec.set_name],
        |r| r.get(0),
    )?;
    let set_id = Uuid::new_v4().to_string();
    // SB-ENV-005: the manifest lands in the SAME INSERT that allocates the version -
    // the two exist atomically or not at all. NULL is the legacy fixture path only.
    insert_log_set(
        conn,
        LogSetRow {
            set_id: &set_id,
            well_id,
            set_name: &spec.set_name,
            version,
            module: &spec.module,
            params_json: Some(&spec.params_json),
            inputs_json: Some(&spec.inputs_json),
            frame: crate::schema_vocab::LogSetFrame::Standard.as_str(),
            sampling_style: Some(discipline.sampling_style.as_str()),
            duplicate_resolution: Some(
                crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str(),
            ),
            outcome_state,
            applied_steps_json,
        },
    )?;
    Ok((set_id, version))
}

fn write_run_parameters(
    conn: &Connection,
    set_id: &str,
    parameters: &[AncestryParameter],
) -> duckdb::Result<()> {
    for (position, parameter) in parameters.iter().enumerate() {
        let (value_json, source, state): (Option<String>, Option<&str>, Option<&str>) =
            if parameter.is_required_unset() {
                (None, None, Some(REQUIRED_UNSET_PARAMETER_STATE))
            } else {
                (Some(parameter.value.to_string()), Some(parameter.source.as_str()), None)
            };
        conn.execute(
            "INSERT INTO run_parameters
                (set_id, position, name, value_json, source, state, resolution, manifest_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                set_id,
                position as i64,
                parameter.name,
                value_json,
                source,
                state,
                parameter.resolution.map(ParameterResolution::as_str),
                parameter.manifest_version
            ],
        )?;
    }
    Ok(())
}

/// Legacy test-fixture entry point. Production code is inventoried by SB-CORE-T14 and must use
/// [`create_complete_log_set`] so it cannot obtain a writable set id from partial JSON.
/// SB-ENV-005: writes NO manifest on purpose - this path simulates the pre-contract rows
/// whose step history is unknown.
#[cfg(test)]
pub(crate) fn create_log_set(
    conn: &Connection,
    well_id: &str,
    spec: &LogSetSpec,
) -> duckdb::Result<(String, i64)> {
    create_log_set_raw(conn, well_id, spec, SetWriteDiscipline::default(), None, None)
}

pub(crate) fn create_complete_log_set(
    conn: &Connection,
    well_id: &str,
    spec: &CompleteLogSetSpec,
) -> Result<(CompleteSetId, i64), String> {
    spec.ancestry.validate()?;
    validate_set_write_discipline(spec.discipline)?;
    // SB-ENV-005: every complete (production) write is manifest-era - the manifest is
    // derived from what this run already resolved and rides the version's own INSERT.
    let applied_steps = serde_json::to_string(&derive_applied_steps(spec))
        .map_err(|error| format!("cannot serialize applied-step manifest: {error}"))?;
    let (value, version) = crate::db::with_txn(conn, |conn| {
        let created = create_log_set_raw(
            conn,
            well_id,
            &spec.storage,
            spec.discipline,
            Some(RUN_OUTCOME_CLEAN),
            Some(&applied_steps),
        )?;
        write_run_parameters(conn, &created.0, &spec.ancestry.parameters)?;
        Ok::<_, duckdb::Error>(created)
    })
    .map_err(|error| error.to_string())?;
    Ok((
        CompleteSetId {
            value,
            well_id: well_id.to_string(),
            outputs: spec
                .ancestry
                .outputs
                .iter()
                .map(|output| output.curve.to_uppercase())
                .collect(),
        },
        version,
    ))
}

/// The set's declared sampling style, refused unless the set is live and the declaration parses.
///
/// A POINT set is refused outright: point deliveries have their own store, which declares and logs
/// how it resolves a repeated depth. So is a set whose duplicate-depth resolution is anything but
/// REFUSE - a continuous curve that silently picks a winner at a repeated depth has decided which
/// of two measurements is the rock.
fn load_set_write_discipline(
    conn: &Connection,
    set_id: &str,
) -> Result<SetWriteDiscipline, String> {
    let stored: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT sampling_style, duplicate_resolution
             FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| format!("log-set write discipline is not live: {error}"))?;
    let sampling_style = stored
        .0
        .as_deref()
        .and_then(crate::schema_vocab::SamplingStyle::parse)
        .ok_or_else(|| {
            "log-set write refused: this set's sampling style is unrecorded or unreadable. A \
             version cannot be allocated for a set whose depths have no declared meaning, \
             because every later read would inherit the guess. Re-import the set with its \
             sampling style declared."
                .to_string()
        })?;
    let duplicate_resolution = stored
        .1
        .as_deref()
        .and_then(crate::schema_vocab::DuplicateDepthResolution::parse)
        .ok_or_else(|| {
            "log-set write refused: this set's duplicate-depth resolution is unrecorded or \
             unreadable. Whether two rows at one depth are an error or a correction changes \
             what every later read returns, so it cannot be assumed. Re-import the set with \
             its duplicate-depth resolution declared."
                .to_string()
        })?;
    if duplicate_resolution != crate::schema_vocab::DuplicateDepthResolution::Refuse {
        return Err("log-set write refused: a continuous set must declare duplicate-depth \
                    resolution REFUSE, and this one declares something else. On a continuous \
                    log two rows at one depth are a delivery fault, not a choice between \
                    readings. Re-import the set with REFUSE declared."
            .into());
    }
    let discipline = SetWriteDiscipline { sampling_style };
    validate_set_write_discipline(discipline)?;
    Ok(discipline)
}

fn validate_set_write_discipline(discipline: SetWriteDiscipline) -> Result<(), String> {
    match discipline.sampling_style {
        crate::schema_vocab::SamplingStyle::ContinuousRegular
        | crate::schema_vocab::SamplingStyle::ContinuousIrregular => Ok(()),
        crate::schema_vocab::SamplingStyle::Point => Err(
            "POINT data must use the point-delivery store, which declares and logs its resolution"
                .into(),
        ),
    }
}

fn depth_identity(depth: f32) -> u32 {
    if depth == 0.0 {
        0.0_f32.to_bits()
    } else {
        depth.to_bits()
    }
}

fn validate_continuous_depth_uniqueness(
    depth: &[f32],
    curves: &[(&str, &[f32])],
) -> Result<(), String> {
    for (curve, values) in curves {
        let mut first_rows = HashMap::<u32, usize>::new();
        for (index, value) in depth.iter().take(values.len()).enumerate() {
            let key = depth_identity(*value);
            if let Some(first) = first_rows.insert(key, index) {
                return Err(format!(
                    "continuous depth uniqueness refused for curve '{curve}' at depth {value}: source rows {} and {} share one depth",
                    first + 1,
                    index + 1
                ));
            }
        }
    }
    Ok(())
}

fn validate_archived_continuous_depth_uniqueness(
    conn: &Connection,
    set_id: &str,
) -> Result<(), String> {
    let mut statement = conn
        .prepare(
            "SELECT curve_name, depth,
                    row_number() OVER (
                        PARTITION BY upper(curve_name) ORDER BY rowid
                    ) AS source_row
             FROM computed_curves_archive
             WHERE set_id = ?1
             ORDER BY upper(curve_name), source_row",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![set_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f32>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut first_rows = HashMap::<(String, u32), i64>::new();
    for row in rows {
        let (curve, depth, source_row) = row.map_err(|error| error.to_string())?;
        let curve_key = curve.to_ascii_uppercase();
        let key = (curve_key, depth_identity(depth));
        if let Some(first) = first_rows.insert(key, source_row) {
            return Err(format!(
                "continuous depth uniqueness refused for curve '{curve}' at depth {depth}: source rows {first} and {source_row} share one depth"
            ));
        }
    }
    Ok(())
}

/// Versioned batch write, and THE discipline the PK-less `computed_curves` table rests on:
/// refreshes the CURRENT store (delete-then-append, rows tagged with `set_id`) and appends the
/// identical rows to the append-only archive, all inside one transaction. Prior versions' archive
/// rows are untouched — that is the "never overwrite" guarantee; any version can be restored via
/// `restore_log_set`.
///
/// EVERY production write arrives here: [`write_computed_curves_with_ancestry`], its `_batch` and
/// `_clearing` siblings verify the complete-ancestry set and then call this. The `#[cfg(test)]`
/// fixtures `equations::write_computed_curves_batch` and [`write_computed_curves_versioned`] are
/// shorter doors onto the same function, not second implementations - which is why a comment that
/// names one of THEM as the write discipline is naming a fixture. Pinned by
/// `the_pk_less_write_discipline_names_the_function_that_performs_it`.
fn write_versioned_rows_raw(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &str,
) -> Result<(), String> {
    if curves.is_empty() {
        return Ok(());
    }
    load_set_write_discipline(conn, set_id)?;
    validate_continuous_depth_uniqueness(depth, curves)?;
    // Atomic: DELETE current + append current + append archive must land as one unit, so a
    // crash can't strand the DELETE with the current-store append lost.
    crate::db::with_txn(conn, |conn| {
        let placeholders = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM computed_curves WHERE well_id = ? AND upper(curve_name) IN ({placeholders})");
        // Bind UPPERCASED names so a re-cased write reclaims any prior-casing rows: every reader
        // resolves curve_name case-insensitively via upper(), but an exact-case DELETE would leave
        // a stale shadow row (e.g. old 'phie' after a rewrite to 'PHIE') that can silently win.
        let mut del_params: Vec<String> = Vec::with_capacity(curves.len() + 1);
        del_params.push(well_id.to_string());
        for (name, _) in curves {
            del_params.push(name.to_uppercase());
        }
        conn.execute(&sql, params_from_iter(del_params))?;

        let mut current = conn.appender("computed_curves")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                // SB-DBM-030: a missing sample is SQL NULL at the store, never a float a query could read.
                let stored: Option<f32> = (!v.is_nan()).then(|| *v);
                current.append_row(params![well_id, d, name, stored, set_id])?;
            }
        }
        current.flush()?;

        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                let stored: Option<f32> = (!v.is_nan()).then(|| *v);
                archive.append_row(params![set_id, well_id, d, name, stored])?;
            }
        }
        archive.flush()?;
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

/// Legacy test-fixture entry point. A production caller would be able to pair arbitrary rows with
/// arbitrary partial metadata, so SB-CORE-T14 forbids calls outside test code.
#[cfg(test)]
pub(crate) fn write_computed_curves_versioned(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &str,
) -> Result<(), String> {
    write_versioned_rows_raw(conn, well_id, depth, curves, set_id)
}

/// AUDIT-2026-08-20 finding 77: the gate every complete-ancestry write passes before it touches a
/// row. It was written out three times - character-identical, refusal text included - in the
/// single-well writer, the clearing writer and the batch writer.
///
/// All four checks earn their place and none covers another. The set must belong to THIS well,
/// because a set id is not well-scoped by construction. Every curve about to be written must
/// already be a declared output of the record, because a curve stored under a set that never
/// claimed it can never say where it came from. The set must still be live, and the manifest it
/// stores must parse. A fourth writer that inherited three of the four would compile and would
/// write; what it would lose is the one guarantee the complete-ancestry set exists for.
pub(crate) fn verify_complete_set_covers<'a>(
    conn: &Connection,
    well_id: &str,
    curve_names: impl Iterator<Item = &'a str>,
    set_id: &CompleteSetId,
) -> Result<(), String> {
    if set_id.well_id != well_id {
        return Err("complete ancestry set belongs to a different well".into());
    }
    for name in curve_names {
        if !set_id
            .outputs
            .iter()
            .any(|output| output.eq_ignore_ascii_case(name))
        {
            return Err(format!(
                "computed curve '{name}' has no output derivation in its ancestry record"
            ));
        }
    }
    let stored: Option<String> = conn
        .query_row(
            "SELECT params_json FROM log_sets WHERE set_id = ?1 AND well_id = ?2",
            params![set_id.value, well_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("complete ancestry set is not live: {error}"))?;
    parse_curve_ancestry(stored.as_deref().unwrap_or_default())?;
    Ok(())
}

pub(crate) fn write_computed_curves_with_ancestry(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &CompleteSetId,
) -> Result<(), String> {
    verify_complete_set_covers(conn, well_id, curves.iter().map(|(name, _)| *name), set_id)?;
    write_versioned_rows_raw(conn, well_id, depth, curves, &set_id.value)
}

/// Complete write that ALSO retires a declared family of stale current curves, in the same
/// transaction as the ordinary write discipline — it clears the curves it is about to write AND
/// the declared family, which is what "also" means here.
///
/// `computed_curves` carries no primary key by design, so uniqueness rests entirely on
/// DELETE-then-append. Monte Carlo passes the extra family when a later run stops producing a
/// previously persisted key; the archive remains append-only and no duplicate-tolerant/upsert
/// path is introduced.
pub(crate) fn write_computed_curves_with_ancestry_clearing(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    clear_names: &[String],
    set_id: &CompleteSetId,
) -> Result<(), String> {
    verify_complete_set_covers(conn, well_id, curves.iter().map(|(name, _)| *name), set_id)?;
    load_set_write_discipline(conn, &set_id.value)?;
    validate_continuous_depth_uniqueness(depth, curves)?;
    crate::db::with_txn(conn, |conn| {
        // The DELETE covers the union of the declared stale family AND the curves this call is
        // about to write. Clearing only `clear_names` would let every written curve accumulate a
        // SECOND set of rows on a re-run, and `computed_curves` has no primary key to object -
        // the duplicate would just sit there, doubling whatever a reader averages. Today's sole
        // caller passes a family that happens to cover its own outputs, so the union changes
        // nothing for it; what it removes is the latent case, and it is the same discipline
        // `write_versioned_rows_raw` follows.
        //
        // Uppercased for that function's reason too: every reader resolves curve_name through
        // upper(), so an exact-case DELETE would leave a stale shadow row (old 'phie' after a
        // rewrite to 'PHIE') that can silently win.
        let mut targets: Vec<String> = Vec::with_capacity(clear_names.len() + curves.len());
        for name in clear_names
            .iter()
            .map(|name| name.to_uppercase())
            .chain(curves.iter().map(|(name, _)| name.to_uppercase()))
        {
            if !targets.contains(&name) {
                targets.push(name);
            }
        }
        if !targets.is_empty() {
            let placeholders = std::iter::repeat("?").take(targets.len()).collect::<Vec<_>>().join(", ");
            let sql = format!(
                "DELETE FROM computed_curves WHERE well_id = ? AND upper(curve_name) IN ({placeholders})"
            );
            let mut values = Vec::with_capacity(targets.len() + 1);
            values.push(well_id.to_string());
            values.extend(targets);
            conn.execute(&sql, params_from_iter(values))?;
        }
        let mut current = conn.appender("computed_curves")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                // SB-DBM-030: a missing sample is SQL NULL at the store, never a float a query could read.
                let stored: Option<f32> = (!value.is_nan()).then(|| *value);
                current.append_row(params![well_id, d, name, stored, set_id.value])?;
            }
        }
        current.flush()?;
        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                let stored: Option<f32> = (!value.is_nan()).then(|| *value);
                archive.append_row(params![set_id.value, well_id, d, name, stored])?;
            }
        }
        archive.flush()?;
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

/// Writes a complete version whose archive carries an independent depth frame. Reframe sets must
/// not enter `computed_curves`: that table is aligned to the well's live frame, so doing so would
/// replace a readable interpretation with rows that no current-frame reader can align.
pub(crate) fn write_complete_own_frame(
    conn: &Connection,
    well_id: &str,
    spec: &CompleteLogSetSpec,
    depth: &[f32],
    curves: &[(String, Vec<f32>)],
) -> Result<i64, String> {
    spec.ancestry.validate()?;
    for (curve, _) in curves {
        if !spec
            .ancestry
            .outputs
            .iter()
            .any(|output| output.curve.eq_ignore_ascii_case(curve))
        {
            return Err(format!(
                "computed curve '{curve}' has no output derivation in its ancestry record"
            ));
        }
    }
    validate_set_write_discipline(spec.discipline)?;
    let continuous_curves = curves
        .iter()
        .map(|(name, values)| (name.as_str(), values.as_slice()))
        .collect::<Vec<_>>();
    validate_continuous_depth_uniqueness(depth, &continuous_curves)?;
    // SB-ENV-005: a reframe is a production manifest-era write like any other.
    let applied_steps = serde_json::to_string(&derive_applied_steps(spec))
        .map_err(|error| format!("cannot serialize applied-step manifest: {error}"))?;
    crate::db::with_txn(conn, |conn| {
        let (set_id, version) =
            create_log_set_raw(
                conn,
                well_id,
                &spec.storage,
                spec.discipline,
                Some(RUN_OUTCOME_CLEAN),
                Some(&applied_steps),
            )?;
        conn.execute(
            "UPDATE log_sets SET frame = ?2 WHERE set_id = ?1",
            params![set_id, crate::schema_vocab::LogSetFrame::Own.as_str()],
        )?;
        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                // SB-DBM-030: a missing sample is SQL NULL at the store, never a float a query could read.
                let stored: Option<f32> = (!value.is_nan()).then(|| *value);
                archive.append_row(params![set_id, well_id, d, name, stored])?;
            }
        }
        archive.flush()?;
        Ok::<i64, duckdb::Error>(version)
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn parse_curve_ancestry(params_json: &str) -> Result<CurveAncestry, String> {
    let parameters: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|error| format!("curve ancestry parameter JSON is invalid: {error}"))?;
    let record = parameters
        .get(CURVE_ANCESTRY_KEY)
        .ok_or_else(|| "computed curve has no complete ancestry record".to_string())?;
    let mut ancestry: CurveAncestry = serde_json::from_value(record.clone())
        .map_err(|error| format!("curve ancestry record is invalid: {error}"))?;
    if ancestry.schema_version < 3
        && ancestry.parameters.is_empty()
        && ancestry.parameter_state.is_none()
    {
        ancestry.parameter_state = Some(ProvenanceAbsentState::LegacyUnrecorded);
    }
    ancestry.validate()?;
    Ok(ancestry)
}

pub(crate) const LEGACY_UNRECORDED: &str = ProvenanceAbsentState::LegacyUnrecorded.as_str();

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputedProvenanceClass {
    Recorded,
    LegacyUnrecorded,
}

#[derive(Debug, Clone)]
pub(crate) struct ComputedProvenanceGroup {
    pub curve_name: String,
    pub set_id: Option<String>,
    pub provenance_class: ComputedProvenanceClass,
    pub row_count: i64,
}

/// Classifies every live computed row by its actual join to `log_sets`. A non-NULL UUID whose
/// target record is missing is no more provenanced than a NULL UUID, so both enter the explicit
/// legacy class. Grouping by set identity preserves the one-hop record for every valid row while
/// retaining an exact count for every unrecorded group.
pub(crate) fn computed_provenance_groups(
    conn: &Connection,
    well_id: &str,
) -> Result<Vec<ComputedProvenanceGroup>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT cc.curve_name, CAST(s.set_id AS VARCHAR),
                    s.set_id IS NOT NULL,
                    COUNT(*)
             FROM computed_curves cc
             LEFT JOIN log_sets s ON s.set_id = cc.set_id
             WHERE cc.well_id = ?1
             GROUP BY cc.curve_name, s.set_id
             ORDER BY upper(cc.curve_name), s.set_id NULLS FIRST",
        )
        .map_err(|error| error.to_string())?;
    stmt.query_map(params![well_id], |row| {
        let recorded = row.get::<_, bool>(2)?;
        Ok(ComputedProvenanceGroup {
            curve_name: row.get(0)?,
            set_id: if recorded { row.get(1)? } else { None },
            provenance_class: if recorded {
                ComputedProvenanceClass::Recorded
            } else {
                ComputedProvenanceClass::LegacyUnrecorded
            },
            row_count: row.get(3)?,
        })
    })
    .map_err(|error| error.to_string())?
    .collect::<duckdb::Result<Vec<_>>>()
    .map_err(|error| error.to_string())
}

/// Resolves the one live record attached to a computed curve. Multiple or NULL set identities are
/// refused rather than selecting whichever row DuckDB happens to return first.
///
/// **`#[cfg(test)]` because it is now the SPECIFICATION rather than a caller's route.**
/// `curve_ancestry_batch` replaced its last production caller, and the gate correctly flagged it as
/// dead. It is kept - and kept exactly as it was - because the batch is a fast path whose only
/// safety argument is that it agrees with this function, including where this function refuses;
/// `the_batched_ancestry_lookup_answers_exactly_what_asking_one_at_a_time_answers` runs both over
/// the same fixture and compares them. Deleting it would leave the batch pinned against nothing,
/// and rewriting the batch to be its own reference is precisely the circularity to avoid in a
/// provenance path. It costs nothing in a shipped build.
#[cfg(test)]
pub(crate) fn curve_ancestry(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
) -> Result<CurveAncestry, String> {
    let groups = computed_provenance_groups(conn, well_id)?
        .into_iter()
        .filter(|group| group.curve_name.eq_ignore_ascii_case(curve_name))
        .collect::<Vec<_>>();
    if groups.len() != 1 || groups[0].provenance_class != ComputedProvenanceClass::Recorded {
        return Err(format!(
            "computed curve '{curve_name}' has no single live ancestry record"
        ));
    }
    let set_id = groups[0].set_id.as_deref().expect("recorded groups carry a set id");
    let params_json: Option<String> = conn
        .query_row(
            "SELECT params_json FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            format!("computed curve '{curve_name}' cites a missing ancestry record: {error}")
        })?;
    parse_curve_ancestry(params_json.as_deref().unwrap_or_default())
}

/// `curve_ancestry` for many (well, curve) pairs in TWO queries instead of one pair at a time.
///
/// The per-call form runs `computed_provenance_groups`, which is a WHOLE-WELL `GROUP BY`, and then
/// throws away every group but one - so asking three curves about one well ran that same whole-well
/// query three times. Over a 100-well pay summary that was 300 whole-well scans plus 300 `log_sets`
/// lookups. This asks once for every well's groups and once for every set's parameters.
///
/// **A pair is ABSENT from the map exactly where `curve_ancestry` returns `Err`**, and the three
/// ways that happens are reproduced deliberately rather than approximated:
///
/// - the pair has no single live group - note the SQL groups by the RAW `curve_name`, not
///   `upper(curve_name)`, because a well carrying both `PHIE` and `phie` is two groups to the
///   per-call form and must stay two here; folding them in SQL would turn a refusal into an answer;
/// - the one group it has is `LegacyUnrecorded`;
/// - the cited `log_sets` row is missing, or its parameters do not parse as an ancestry record.
///
/// Pinned against the per-call function by
/// `the_batched_ancestry_lookup_answers_exactly_what_asking_one_at_a_time_answers`.
pub(crate) fn curve_ancestry_batch(
    conn: &Connection,
    well_ids: &[String],
    curve_names: &[String],
) -> Result<HashMap<(String, String), CurveAncestry>, String> {
    if well_ids.is_empty() || curve_names.is_empty() {
        return Ok(HashMap::new());
    }
    let wph = std::iter::repeat("?").take(well_ids.len()).collect::<Vec<_>>().join(", ");
    let cph = std::iter::repeat("?").take(curve_names.len()).collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT cc.well_id, cc.curve_name, CAST(s.set_id AS VARCHAR), s.set_id IS NOT NULL
         FROM computed_curves cc
         LEFT JOIN log_sets s ON s.set_id = cc.set_id
         WHERE cc.well_id IN ({wph}) AND upper(cc.curve_name) IN ({cph})
         GROUP BY cc.well_id, cc.curve_name, s.set_id"
    );
    let mut binds: Vec<String> = Vec::with_capacity(well_ids.len() + curve_names.len());
    binds.extend(well_ids.iter().cloned());
    binds.extend(curve_names.iter().map(|curve| curve.trim().to_uppercase()));

    // (well, UPPER curve) -> every group it has. More than one, or one that is unrecorded, is the
    // per-call function's refusal and must stay a refusal.
    let mut groups: HashMap<(String, String), Vec<Option<String>>> = HashMap::new();
    {
        let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(binds), |row| {
                let recorded = row.get::<_, bool>(3)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    if recorded { row.get::<_, Option<String>>(2)? } else { None },
                ))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (well_id, curve_name, set_id) = row.map_err(|error| error.to_string())?;
            groups
                .entry((well_id, curve_name.trim().to_uppercase()))
                .or_default()
                .push(set_id);
        }
    }

    let wanted: Vec<String> = groups
        .values()
        .filter(|found| found.len() == 1)
        .filter_map(|found| found[0].clone())
        .collect();
    if wanted.is_empty() {
        return Ok(HashMap::new());
    }
    let mut parameters: HashMap<String, Option<String>> = HashMap::new();
    {
        let sph = std::iter::repeat("?").take(wanted.len()).collect::<Vec<_>>().join(", ");
        let mut stmt = conn
            .prepare(&format!(
                "SELECT CAST(set_id AS VARCHAR), params_json FROM log_sets WHERE CAST(set_id AS VARCHAR) IN ({sph})"
            ))
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(wanted), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (set_id, params_json) = row.map_err(|error| error.to_string())?;
            parameters.insert(set_id, params_json);
        }
    }

    let mut resolved = HashMap::new();
    for (key, found) in groups {
        if found.len() != 1 {
            continue;
        }
        let Some(set_id) = found[0].as_deref() else {
            continue;
        };
        // A cited-but-missing record is the per-call function's error, so it stays absent here.
        let Some(params_json) = parameters.get(set_id) else {
            continue;
        };
        if let Ok(ancestry) = parse_curve_ancestry(params_json.as_deref().unwrap_or_default()) {
            resolved.insert(key, ancestry);
        }
    }
    Ok(resolved)
}

/// One complete, human-readable ancestry record ready for a catalog or number-carrying
/// deliverable. `set_id` is retained internally by the query but deliberately not exposed here:
/// downstream files need the stable set name/version plus the complete input identities, not an
/// opaque database implementation detail as their only explanation.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CurveAncestryDisclosure {
    pub well_id: String,
    pub curve_name: String,
    pub provenance_class: ComputedProvenanceClass,
    pub provenance_row_count: i64,
    pub set_name: Option<String>,
    pub version: Option<i64>,
    pub ancestry: Option<CurveAncestry>,
}

impl CurveAncestryDisclosure {
    /// Full disclosure columns shared by PDF, Word, workbook and deck surfaces. No field is
    /// summarized away: an input's well/set identity, a value's source, and a zone's source all
    /// remain in the exported text.
    pub(crate) fn cells(&self) -> [String; 7] {
        let Some(ancestry) = self.ancestry.as_ref() else {
            let label = format!(
                "{} / {} ({} rows)",
                self.curve_name, LEGACY_UNRECORDED, self.provenance_row_count
            );
            return [
                label,
                LEGACY_UNRECORDED.into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
            ];
        };
        let inputs = ancestry
            .inputs
            .iter()
            .map(|input| {
                format!(
                    "{}={} [well {}; set {}{}; id {}]",
                    input.argument,
                    input.curve,
                    input.well_id,
                    input.log_set,
                    input
                        .set_version
                        .map(|version| format!(" v{version}"))
                        .unwrap_or_default(),
                    input.set_id,
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let parameters = ancestry
            .parameters
            .iter()
            .map(|parameter| {
                let base = format!(
                    "{}={} [source: {}]",
                    parameter.name, parameter.value, parameter.source
                );
                parameter
                    .decision
                    .as_ref()
                    .map(|decision| format!("{base} [decision: {}]", decision.disclosure()))
                    .unwrap_or(base)
            })
            .collect::<Vec<_>>()
            .join("; ");
        let zones = match &ancestry.zone_scope {
            AncestryZoneScope::WholeWell => "WHOLE WELL".to_string(),
            AncestryZoneScope::Defined(zones) => zones
                .iter()
                .map(|zone| {
                    format!(
                        "{} {}-{} [source: {}]",
                        zone.name, zone.top, zone.base, zone.source
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
        };
        let actor_kind = match ancestry.actor.kind {
            AncestryActorKind::Human => "HUMAN",
            AncestryActorKind::Automated => "AUTOMATED",
        };
        let custody = format!(
            "{} {} at {} UTC-ms",
            actor_kind, ancestry.actor.identity, ancestry.timestamp_utc_ms
        );
        let derivation = ancestry
            .outputs
            .iter()
            .find(|output| output.curve.eq_ignore_ascii_case(&self.curve_name))
            .map(|output| output.derivation.clone())
            .unwrap_or_else(|| {
                ancestry.outputs
                    .iter()
                    .map(|output| format!("{}={}", output.curve, output.derivation))
                    .collect::<Vec<_>>()
                    .join("; ")
            });
        [
            format!(
                "{} / {} v{}",
                self.curve_name,
                self.set_name.as_deref().expect("recorded disclosure has a set name"),
                self.version.expect("recorded disclosure has a version")
            ),
            format!(
                "{} @ {}",
                ancestry.module, ancestry.module_version
            ),
            if inputs.is_empty() {
                "NO CURVE INPUTS".into()
            } else {
                inputs
            },
            if parameters.is_empty() {
                "NO EXPLICIT PARAMETERS".into()
            } else {
                parameters
            },
            zones,
            custody,
            derivation,
        ]
    }
}

fn push_recorded_disclosures(
    disclosures: &mut Vec<CurveAncestryDisclosure>,
    seen: &mut std::collections::BTreeSet<(String, String, String)>,
    well_id: &str,
    set_id: String,
    set_name: String,
    version: i64,
    params_json: Option<String>,
    curves: Vec<(String, i64)>,
) -> Result<(), String> {
    let ancestry = parse_curve_ancestry(params_json.as_deref().unwrap_or_default()).map_err(|error| {
        format!(
            "computed set '{set_name}' v{version} for well '{well_id}' cannot travel into a deliverable: {error}"
        )
    })?;
    for (curve_name, row_count) in curves {
        let key = (
            well_id.to_string(),
            curve_name.to_uppercase(),
            set_id.clone(),
        );
        if !seen.insert(key) {
            continue;
        }
        if !ancestry
            .outputs
            .iter()
            .any(|output| output.curve.eq_ignore_ascii_case(&curve_name))
        {
            return Err(format!(
                "computed curve '{curve_name}' is absent from its set ancestry output derivations"
            ));
        }
        disclosures.push(CurveAncestryDisclosure {
            well_id: well_id.to_string(),
            curve_name,
            provenance_class: ComputedProvenanceClass::Recorded,
            provenance_row_count: row_count,
            set_name: Some(set_name.clone()),
            version: Some(version),
            ancestry: Some(ancestry.clone()),
        });
    }
    Ok(())
}

/// Returns an explicit provenance disclosure for every current computed row in `well_ids`. When a
/// deliverable names an input set, its latest version is included too because those archived values
/// may replace current values while rendering. Rows with no resolvable run record remain visible as
/// `LEGACY_UNRECORDED` with an exact count; no ancestry is inferred for them.
pub(crate) fn curve_ancestry_disclosures(
    conn: &Connection,
    well_ids: &[String],
    input_set: Option<&str>,
) -> Result<Vec<CurveAncestryDisclosure>, String> {
    let mut disclosures = Vec::new();
    let mut seen = std::collections::BTreeSet::<(String, String, String)>::new();

    for well_id in well_ids {
        for group in computed_provenance_groups(conn, well_id)? {
            if group.provenance_class == ComputedProvenanceClass::LegacyUnrecorded {
                let key = (
                    well_id.clone(),
                    group.curve_name.to_uppercase(),
                    LEGACY_UNRECORDED.to_string(),
                );
                if seen.insert(key) {
                    disclosures.push(CurveAncestryDisclosure {
                        well_id: well_id.clone(),
                        curve_name: group.curve_name,
                        provenance_class: ComputedProvenanceClass::LegacyUnrecorded,
                        provenance_row_count: group.row_count,
                        set_name: None,
                        version: None,
                        ancestry: None,
                    });
                }
                continue;
            }
            let set_id = group.set_id.expect("recorded groups carry a set id");
            let (set_name, version, params_json): (String, i64, Option<String>) = conn
                .query_row(
                    "SELECT set_name, version, params_json FROM log_sets WHERE set_id = ?1",
                    params![set_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| error.to_string())?;
            push_recorded_disclosures(
                &mut disclosures,
                &mut seen,
                well_id,
                set_id,
                set_name,
                version,
                params_json,
                vec![(group.curve_name, group.row_count)],
            )?;
        }

        if let Some(input_set) = input_set.map(str::trim).filter(|value| !value.is_empty()) {
            let selected: Option<(String, String, i64, Option<String>)> = conn
                .query_row(
                    "SELECT set_id, set_name, version, params_json FROM log_sets
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, input_set],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .optional()
                .map_err(|error| error.to_string())?;
            if let Some((set_id, set_name, version, params_json)) = selected {
                let mut stmt = conn
                    .prepare(
                        "SELECT curve_name, COUNT(*) FROM computed_curves_archive
                         WHERE set_id = ?1 GROUP BY curve_name ORDER BY curve_name",
                    )
                    .map_err(|error| error.to_string())?;
                let curves = stmt
                    .query_map(params![set_id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                    })
                    .map_err(|error| error.to_string())?
                    .collect::<duckdb::Result<Vec<_>>>()
                    .map_err(|error| error.to_string())?;
                push_recorded_disclosures(
                    &mut disclosures,
                    &mut seen,
                    well_id,
                    set_id,
                    set_name,
                    version,
                    params_json,
                    curves,
                )?;
            }
        }
    }
    disclosures.sort_by(|a, b| {
        (&a.well_id, &a.curve_name, &a.set_name, &a.version).cmp(&(
            &b.well_id,
            &b.curve_name,
            &b.set_name,
            &b.version,
        ))
    });
    Ok(disclosures)
}

/// One well's versioned output, for the batched multi-well writer.
pub(crate) struct WellWrite {
    pub well_id: String,
    pub depth: Vec<f32>,
    pub curves: Vec<(String, Vec<f32>)>,
    pub set_id: String,
    /// `None` only for the legacy test-fixture writer. Complete production writes always carry
    /// an explicit module plus the (possibly empty) structured degradation list.
    pub degradation_module: Option<String>,
    pub degradations: Option<Vec<crate::modules::RunDegradation>>,
}

/// Batched [`create_log_set`]: registers one run event per well inside a SINGLE transaction
/// instead of one auto-committed INSERT (= one WAL fsync) per well. Returns well_id → set_id.
/// Versioning is identical (each well gets 1 + its own MAX(version) for `set_name`).
#[cfg(test)]
pub(crate) fn create_log_sets_batch(
    conn: &Connection,
    well_ids: &[String],
    spec: &LogSetSpec,
) -> duckdb::Result<HashMap<String, String>> {
    // Plan every well's version from the CURRENT committed state FIRST (reads only — wells are
    // distinct so there is no cross-well dependency), THEN INSERT them all in one transaction.
    // Reading MAX(version) *after* an INSERT inside the same transaction trips a DuckDB internal
    // error, so the reads deliberately precede all writes.
    let mut planned: Vec<(String, i64, String)> = Vec::with_capacity(well_ids.len());
    for well_id in well_ids {
        let version: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, spec.set_name],
            |r| r.get(0),
        )?;
        planned.push((well_id.clone(), version, Uuid::new_v4().to_string()));
    }
    crate::db::with_txn(conn, |conn| {
        for (well_id, version, set_id) in &planned {
            insert_log_set(
                conn,
                LogSetRow {
                    set_id,
                    well_id,
                    set_name: &spec.set_name,
                    version: *version,
                    module: &spec.module,
                    params_json: Some(&spec.params_json),
                    inputs_json: Some(&spec.inputs_json),
                    frame: crate::schema_vocab::LogSetFrame::Standard.as_str(),
                    sampling_style: Some(SetWriteDiscipline::default().sampling_style.as_str()),
                    duplicate_resolution: Some(
                        crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str(),
                    ),
                    outcome_state: None,
                    // SB-ENV-005: NULL on purpose. This legacy batch entry point is
                    // test-exercised only and simulates pre-contract rows, whose step history
                    // genuinely cannot be recovered - which is exactly what UNKNOWN says.
                    applied_steps_json: None,
                },
            )?;
        }
        Ok::<(), duckdb::Error>(())
    })?;
    Ok(planned.into_iter().map(|(w, _, s)| (w, s)).collect())
}

/// A workflow chain can cite an output from an earlier step in the same not-yet-registered set.
/// Its final stored identity must be exact, but it must also survive a deterministic replay of the
/// same well, set name and version. A UUIDv8-shaped SHA-256 digest supplies that internal key; an
/// ordinary module set without a self-reference keeps the existing random UUID allocation.
fn deterministic_chain_set_id(well_id: &str, set_name: &str, version: i64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(well_id.as_bytes());
    hasher.update([0]);
    hasher.update(set_name.as_bytes());
    hasher.update([0]);
    hasher.update(version.to_le_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes).to_string()
}

/// Per-well complete batch registration. Unlike the legacy batch API, every well carries its own
/// zone/input resolution snapshot while the inserts still share one transaction.
pub(crate) fn create_complete_log_sets_batch(
    conn: &Connection,
    wells: &[CompleteWellLogSet],
) -> Result<HashMap<String, CompleteSetId>, String> {
    let mut planned: Vec<(String, i64, String, CompleteLogSetSpec)> =
        Vec::with_capacity(wells.len());
    for well in wells {
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
                params![well.well_id, well.spec.storage.set_name],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        let mut spec = well.spec.clone();
        let had_self_inputs = spec
            .ancestry
            .inputs
            .iter()
            .any(|input| input.set_id == "SELF");
        let set_id = if had_self_inputs {
            deterministic_chain_set_id(&well.well_id, &spec.storage.set_name, version)
        } else {
            Uuid::new_v4().to_string()
        };
        if had_self_inputs {
            for input in spec
                .ancestry
                .inputs
                .iter_mut()
                .filter(|input| input.set_id == "SELF")
            {
                input.log_set = spec.storage.set_name.clone();
                input.set_version = Some(version);
                input.set_id = set_id.clone();
                input.chosen_curve_id = Some(format!("computed:{set_id}:{}", input.curve));
            }
            let mut params: serde_json::Value = serde_json::from_str(&spec.storage.params_json)
                .map_err(|error| format!("cannot bind chain input identities: {error}"))?;
            let object = params.as_object_mut().ok_or_else(|| {
                "cannot bind chain input identities in a non-object parameter record".to_string()
            })?;
            object.insert(
                CURVE_ANCESTRY_KEY.into(),
                serde_json::to_value(&spec.ancestry)
                    .map_err(|error| format!("cannot bind chain ancestry: {error}"))?,
            );
            spec.storage.params_json = params.to_string();
            // Workflow-chain legacy input JSON is the same AncestryInput array. Keep it aligned
            // with the complete record rather than persisting the planning-only SELF marker.
            spec.storage.inputs_json = serde_json::to_string(&spec.ancestry.inputs)
                .map_err(|error| format!("cannot bind chain legacy inputs: {error}"))?;
        }
        spec.ancestry.validate()?;
        validate_set_write_discipline(spec.discipline)?;
        planned.push((
            well.well_id.clone(),
            version,
            set_id,
            spec,
        ));
    }
    // SB-ENV-005: derive each well's manifest OUTSIDE the transaction (pure), so a
    // serialization failure aborts before anything is written.
    let mut manifests: Vec<String> = Vec::with_capacity(planned.len());
    for (_, _, _, spec) in &planned {
        manifests.push(
            serde_json::to_string(&derive_applied_steps(spec))
                .map_err(|error| format!("cannot serialize applied-step manifest: {error}"))?,
        );
    }
    crate::db::with_txn(conn, |conn| {
        for ((well_id, version, set_id, spec), applied_steps) in planned.iter().zip(&manifests) {
            insert_log_set(
                conn,
                LogSetRow {
                    set_id,
                    well_id,
                    set_name: &spec.storage.set_name,
                    version: *version,
                    module: &spec.storage.module,
                    params_json: Some(&spec.storage.params_json),
                    inputs_json: Some(&spec.storage.inputs_json),
                    frame: crate::schema_vocab::LogSetFrame::Standard.as_str(),
                    sampling_style: Some(spec.discipline.sampling_style.as_str()),
                    duplicate_resolution: Some(
                        crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str(),
                    ),
                    outcome_state: Some(RUN_OUTCOME_CLEAN),
                    applied_steps_json: Some(&applied_steps),
                },
            )?;
            write_run_parameters(conn, set_id, &spec.ancestry.parameters)?;
        }
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())?;
    Ok(planned
        .into_iter()
        .map(|(well_id, _, value, spec)| {
            let outputs = spec
                .ancestry
                .outputs
                .iter()
                .map(|output| output.curve.to_uppercase())
                .collect();
            (
                well_id.clone(),
                CompleteSetId {
                    value,
                    well_id,
                    outputs,
                },
            )
        })
        .collect())
}

/// Batched [`write_computed_curves_versioned`]: writes MANY wells' outputs in ONE transaction.
///
/// The earlier version mirrored the single-well path per well — one DELETE + a fresh current
/// appender + a fresh archive appender for every well. At field scale (544 wells) that is 544
/// full-table DELETE scans of `computed_curves` plus 1088 appender open/flush/drop cycles, and it
/// dominated the between-step "pause" the user saw. This version keeps the identical semantics
/// (same delete-then-append discipline, same current+archive double-write, each well's rows still
/// carrying that well's own `set_id`) but restructures the work so it runs in seconds:
///
///   Phase 1 — clear the CURRENT store. Wells are grouped by their exact curve-set (every well in
///   a workflow step writes the same curves, so this is normally ONE group), and each group is
///   cleared with a single `DELETE ... WHERE well_id IN (…) AND curve_name IN (…)` — one table
///   pass for the whole batch instead of one per well. Deleting the exact (wells × curves) cross
///   product is safe *because every well in a group has exactly that curve-set*.
///
///   Phase 2/3 — append. With every DELETE already done, ONE appender per table may span all
///   wells: the DuckDB "appender can't span DML on the same table" constraint only forbids
///   interleaving a DELETE while an appender is open, which never happens here.
fn write_versioned_rows_batch_raw(conn: &Connection, wells: &[WellWrite]) -> Result<(), String> {
    if wells.iter().all(|w| w.curves.is_empty()) {
        return Ok(());
    }
    #[cfg(test)]
    let _phase_validate = crate::lock_probe::w_validate();
    for well in wells {
        load_set_write_discipline(conn, &well.set_id)?;
        match (&well.degradation_module, &well.degradations) {
            (None, None) => {}
            (Some(module), Some(events)) => {
                if module.trim().is_empty() {
                    return Err("a durable degradation record is missing its module".into());
                }
                for event in events {
                    if event.detail.trim().is_empty() || event.occurrences == 0 {
                        return Err(format!(
                            "{} degradation must have non-empty detail and a positive occurrence count",
                            event.kind.as_str()
                        ));
                    }
                }
            }
            _ => {
                return Err(
                    "a complete write must carry both its degradation module and event list".into(),
                )
            }
        }
        let curves = well
            .curves
            .iter()
            .map(|(name, values)| (name.as_str(), values.as_slice()))
            .collect::<Vec<_>>();
        validate_continuous_depth_uniqueness(&well.depth, &curves)?;
    }
    #[cfg(test)]
    drop(_phase_validate);
    crate::db::with_txn(conn, |conn| {
        // Phase 1: group wells by identical curve-set, then one DELETE per group.
        use std::collections::BTreeMap;
        let mut groups: BTreeMap<Vec<&str>, Vec<&str>> = BTreeMap::new();
        for w in wells {
            if w.curves.is_empty() {
                continue;
            }
            let mut names: Vec<&str> = w.curves.iter().map(|(n, _)| n.as_str()).collect();
            names.sort_unstable();
            names.dedup();
            groups.entry(names).or_default().push(w.well_id.as_str());
        }
        #[cfg(test)]
        let _phase_delete = crate::lock_probe::w_delete();
        for (curves, well_ids) in &groups {
            let wph = std::iter::repeat("?").take(well_ids.len()).collect::<Vec<_>>().join(", ");
            let cph = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
            let sql =
                format!("DELETE FROM computed_curves WHERE well_id IN ({wph}) AND upper(curve_name) IN ({cph})");
            // Uppercase curve names (not well_ids) so a re-cased write reclaims prior-casing rows.
            let mut p: Vec<String> = Vec::with_capacity(well_ids.len() + curves.len());
            p.extend(well_ids.iter().map(|w| w.to_string()));
            p.extend(curves.iter().map(|c| c.to_uppercase()));
            conn.execute(&sql, params_from_iter(p))?;
        }
        #[cfg(test)]
        drop(_phase_delete);

        // Phase 2: one appender for the CURRENT store across every well.
        {
            #[cfg(test)]
            let _phase_current = crate::lock_probe::w_current();
            let mut current = conn.appender("computed_curves")?;
            for w in wells {
                for (name, values) in &w.curves {
                    for (d, v) in w.depth.iter().zip(values.iter()) {
                        // SB-DBM-030: a missing sample is SQL NULL at the store, never a float a query could read.
                        let stored: Option<f32> = (!v.is_nan()).then(|| *v);
                        current.append_row(params![w.well_id, d, name, stored, w.set_id])?;
                    }
                }
            }
            current.flush()?;
        }

        // Phase 3: the append-only ARCHIVE, copied table-to-table by the engine rather than pushed
        // through a second appender. Every row it needs was just written to the current store by
        // Phase 2, so re-serializing the same values out of Rust memory is work already done once.
        // The WHERE is Phase 1's own grouping: that DELETE removed exactly these (well, curve)
        // pairs and Phase 2 refilled them, so this selects the rows THIS call wrote and nothing
        // else - carrying each row's own set_id, which is what the appender wrote per well. A
        // missing sample is SQL NULL in both stores, so it copies across as NULL untouched.
        {
            #[cfg(test)]
            let _phase_archive = crate::lock_probe::w_archive();
            for (curves, well_ids) in &groups {
                let wph = std::iter::repeat("?").take(well_ids.len()).collect::<Vec<_>>().join(", ");
                let cph = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
                let sql = format!(
                    "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
                     SELECT set_id, well_id, depth, curve_name, value FROM computed_curves
                     WHERE well_id IN ({wph}) AND upper(curve_name) IN ({cph})"
                );
                let mut p: Vec<String> = Vec::with_capacity(well_ids.len() + curves.len());
                p.extend(well_ids.iter().map(|w| w.to_string()));
                p.extend(curves.iter().map(|c| c.to_uppercase()));
                conn.execute(&sql, params_from_iter(p))?;
            }
        }

        // Phase 4: classify the run and append its structured reasons in the SAME transaction as
        // current + archive rows. A curve can therefore never commit while the warning that
        // qualifies it is lost. Workflow steps reuse one set_id, so new events append after prior
        // positions and a later clean step never erases an earlier DEGRADED state.
        #[cfg(test)]
        let _phase_degrade = crate::lock_probe::w_degrade();
        let mut pending_degradations: Vec<(String, i64, String, &str, String, i64)> = Vec::new();
        let mut next_position: HashMap<String, i64> = HashMap::new();
        for well in wells {
            let (Some(module), Some(events)) =
                (&well.degradation_module, &well.degradations)
            else {
                continue; // legacy test-fixture writer; its run remains unclassified
            };
            let state = if events.is_empty() {
                RUN_OUTCOME_CLEAN
            } else {
                RUN_OUTCOME_DEGRADED
            };
            let updated = if events.is_empty() {
                conn.execute(
                    "UPDATE log_sets SET outcome_state = COALESCE(outcome_state, ?2)
                     WHERE set_id = ?1 AND well_id = ?3",
                    params![well.set_id, state, well.well_id],
                )
                .map_err(|error| {
                    duckdb::Error::InvalidParameterName(format!(
                        "classifying clean run {} failed: {error}",
                        well.set_id
                    ))
                })?
            } else {
                conn.execute(
                    "UPDATE log_sets SET outcome_state = ?2
                     WHERE set_id = ?1 AND well_id = ?3",
                    params![well.set_id, state, well.well_id],
                )
                .map_err(|error| {
                    duckdb::Error::InvalidParameterName(format!(
                        "classifying degraded run {} failed: {error}",
                        well.set_id
                    ))
                })?
            };
            if updated != 1 {
                return Err(duckdb::Error::InvalidParameterName(format!(
                    "complete degradation record has no live log-set row for {}",
                    well.set_id
                )));
            }
            // The seed comes from the table on first sight of a set and from memory afterwards.
            // The rows this loop plans are not in the table yet - they are appended once, below -
            // so a set_id appearing twice in one batch would otherwise read the same seed twice
            // and plan two rows at the same position, which the primary key would then reject.
            // Re-reading was what made the row-by-row version safe against that; carrying the
            // counter forward is what replaces it.
            let mut position = match next_position.get(&well.set_id) {
                Some(next) => *next,
                None => conn
                    .query_row(
                        "SELECT position FROM run_degradations
                         WHERE set_id = ?1 ORDER BY position DESC LIMIT 1",
                        params![well.set_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|error| {
                        duckdb::Error::InvalidParameterName(format!(
                            "locating the next degradation position for {} failed: {error}",
                            well.set_id
                        ))
                    })?
                    .map_or(0, |last| last + 1),
            };
            for event in events {
                pending_degradations.push((
                    well.set_id.clone(),
                    position,
                    module.clone(),
                    event.kind.as_str(),
                    event.detail.clone(),
                    event.occurrences as i64,
                ));
                position += 1;
            }
            next_position.insert(well.set_id.clone(), position);
        }

        // Phase 5: ONE appender for every degradation row of every well, for the same reason
        // phases 2 and 3 use one - and here it is the difference between a chain step that takes
        // 46 seconds and one that does not.
        //
        // Measured 2026-08-23 (`PERF-PHI-DEN-2026-08-23.md`): `phi_den` writes 896 degradation
        // rows per well because it clamps PHIE to [PHIE_FLOOR, PHIT] and PHIT differs at every
        // sample, so the `(kind, detail)` aggregation in `modules.rs` never collapses them. At 100
        // wells that was 89,600 separate `conn.execute` calls at about half a millisecond each,
        // holding the single shared connection throughout - 45.7 s of a 60.4 s module run, against
        // `sw_indo` writing the same 624,800 CURVE rows in 5.3 s.
        //
        // Nothing about what is recorded changes: same rows, same positions, same order, same
        // occurrences. Verified byte-for-byte by
        // `the_batched_degradation_write_stores_exactly_what_the_row_by_row_write_stored`.
        //
        // The appender is safe here, which was measured rather than assumed: `run_degradations`
        // carries CHECK constraints on `kind` and `occurrences > 0` and a PRIMARY KEY on
        // (set_id, position), and a probe confirmed the appender REFUSES all three violations
        // exactly as the statement did. Pinned by
        // `the_batched_degradation_write_still_refuses_what_the_schema_forbids`.
        //
        // The per-well UPDATE and position lookup above are deliberately NOT batched: they are two
        // statements per well against 896, so at 100 wells they are ~200 statements out of 89,800.
        // Batching them would trade the per-well `updated != 1` check - which names the well that
        // lost its log-set row - for a group count that does not.
        if !pending_degradations.is_empty() {
            let count = pending_degradations.len();
            let mut degradations = conn.appender("run_degradations")?;
            for (set_id, position, module, kind, detail, occurrences) in &pending_degradations {
                degradations
                    .append_row(params![set_id, position, module, kind, detail, occurrences])
                    .map_err(|error| {
                        duckdb::Error::InvalidParameterName(format!(
                            "persisting {kind} degradation for {set_id} failed: {error}"
                        ))
                    })?;
            }
            degradations.flush().map_err(|error| {
                duckdb::Error::InvalidParameterName(format!(
                    "persisting {count} degradation row(s) failed: {error}"
                ))
            })?;
        }
        #[cfg(test)]
        drop(_phase_degrade);
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) fn write_computed_curves_versioned_batch(
    conn: &Connection,
    wells: &[WellWrite],
) -> Result<(), String> {
    write_versioned_rows_batch_raw(conn, wells)
}

pub(crate) fn write_computed_curves_with_ancestry_batch(
    conn: &Connection,
    wells: &[CompleteWellWrite],
) -> Result<(), String> {
    let mut raw = Vec::with_capacity(wells.len());
    for well in wells {
        verify_complete_set_covers(
            conn,
            &well.well_id,
            well.curves.iter().map(|(curve, _)| curve.as_str()),
            &well.set_id,
        )?;
        raw.push(WellWrite {
            well_id: well.well_id.clone(),
            depth: well.depth.clone(),
            curves: well.curves.clone(),
            set_id: well.set_id.value.clone(),
            degradation_module: well.degradation_module.clone(),
            degradations: well.degradations.clone(),
        });
    }
    write_versioned_rows_batch_raw(conn, &raw)
}


/// Every run event for a well, newest first, with the curves it wrote and whether any of
/// its rows still provide the current values.
pub(crate) fn list_log_sets(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<LogSetEntry>> {
    let mut stmt = conn.prepare(
        "SELECT s.set_id, s.set_name, s.version, s.module, s.params_json, s.inputs_json,
                strftime(s.created_at, '%Y-%m-%d %H:%M'),
                EXISTS (SELECT 1 FROM computed_curves cc WHERE cc.set_id = s.set_id),
                s.outcome_state, s.comment
         FROM log_sets s
         WHERE s.well_id = ?1
         ORDER BY s.set_name, s.version DESC",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        let params_json: Option<String> = r.get(4)?;
        let ancestry = params_json
            .as_deref()
            .and_then(|text| parse_curve_ancestry(text).ok());
        let restored_from = restore_record(params_json.as_deref());
        Ok(LogSetEntry {
            set_id: r.get(0)?,
            set_name: r.get(1)?,
            version: r.get(2)?,
            module: r.get(3)?,
            params_json,
            inputs_json: r.get(5)?,
            created_at: r.get(6)?,
            curve_names: Vec::new(),
            is_current: r.get(7)?,
            ancestry,
            restored_from,
            outcome_state: r.get(8)?,
            degradations: Vec::new(),
            comment: r.get(9)?,
        })
    })?;
    let mut entries = Vec::new();
    for r in rows {
        entries.push(r?);
    }
    // Curve names per set from the archive (one query, folded in Rust).
    let mut stmt = conn.prepare(
        "SELECT DISTINCT set_id, curve_name FROM computed_curves_archive WHERE well_id = ?1 ORDER BY curve_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut by_set: HashMap<String, Vec<String>> = HashMap::new();
    for r in rows {
        let (sid, name) = r?;
        by_set.entry(sid).or_default().push(name);
    }
    for e in &mut entries {
        if let Some(names) = by_set.remove(&e.set_id) {
            e.curve_names = names;
        }
    }

    let mut statement = conn.prepare(
        "SELECT CAST(d.set_id AS VARCHAR), d.module, d.kind, d.detail, d.occurrences
         FROM run_degradations d
         JOIN log_sets s ON s.set_id = d.set_id
         WHERE s.well_id = ?1
         ORDER BY d.set_id, d.position",
    )?;
    let rows = statement.query_map(params![well_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;
    let mut degradations: HashMap<String, Vec<StoredRunDegradation>> = HashMap::new();
    for row in rows {
        let (set_id, module, kind, detail, occurrences) = row?;
        let kind = crate::modules::RunDegradationKind::parse(&kind).ok_or_else(|| {
            duckdb::Error::InvalidParameterName(format!(
                "run degradation for set {set_id} has unknown kind '{kind}'"
            ))
        })?;
        let occurrences = usize::try_from(occurrences).ok().filter(|value| *value > 0).ok_or_else(
            || {
                duckdb::Error::InvalidParameterName(format!(
                    "run degradation for set {set_id} has invalid occurrence count {occurrences}"
                ))
            },
        )?;
        degradations.entry(set_id).or_default().push(StoredRunDegradation {
            module,
            kind,
            detail,
            occurrences,
        });
    }
    for entry in &mut entries {
        if let Some(events) = degradations.remove(&entry.set_id) {
            entry.degradations = events;
        }
    }
    Ok(entries)
}

/// Distinct constellation (log-set) names across the whole project, alphabetical. The
/// module and workflow dialogs run across many wells at once, so their input/output
/// pickers need the project-wide name list — a single well's `list_log_sets` would miss
/// names that only exist on other wells.
pub(crate) fn list_log_set_names(conn: &Connection) -> duckdb::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT set_name FROM log_sets ORDER BY set_name")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut names = Vec::new();
    for r in rows {
        names.push(r?);
    }
    Ok(names)
}

/// Copies an archived version into a new run event, then makes that new version current for exactly
/// the curves the source version wrote. The source and every intervening version remain unchanged;
/// restoring is another append-only version, never a history rewind.
pub(crate) fn restore_log_set(
    conn: &Connection,
    set_id: &str,
) -> Result<RestoreLogSetResult, String> {
    load_set_write_discipline(conn, set_id)?;
    validate_archived_continuous_depth_uniqueness(conn, set_id)?;
    let source: Option<(
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn
        .query_row(
            "SELECT well_id, set_name, version, params_json, inputs_json, frame,
                    sampling_style, duplicate_resolution, outcome_state
             FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((
        well_id,
        set_name,
        source_version,
        source_params_json,
        inputs_json,
        frame,
        sampling_style,
        duplicate_resolution,
        outcome_state,
    )) = source
    else {
        return Err(format!("log-set version '{set_id}' does not exist"));
    };
    let source_rows: i64 = conn
        .query_row(
            "SELECT count(*) FROM computed_curves_archive WHERE set_id = ?1",
            params![set_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if source_rows == 0 {
        return Err(format!(
            "log-set '{set_name}' version {source_version} has no archived rows to restore"
        ));
    }

    let restored_from = LogSetRestoreRecord {
        schema_version: 1,
        source_set_id: set_id.to_string(),
        source_version,
    };
    let restored_params_json =
        params_json_with_restore_record(source_params_json.as_deref(), &restored_from)?;
    let new_set_id = Uuid::new_v4().to_string();

    // Atomic: append the run record and archive copy, then replace only the current projection.
    // A crash can therefore expose neither a half-created version nor current rows without their
    // immutable archive counterpart.
    let (rows_restored, new_version) = crate::db::with_txn(conn, |conn| {
        let new_version: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets
             WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, set_name],
            |row| row.get(0),
        )?;
        insert_log_set(
            conn,
            LogSetRow {
                set_id: &new_set_id,
                well_id: &well_id,
                set_name: &set_name,
                version: new_version,
                module: "restore",
                params_json: Some(&restored_params_json),
                inputs_json: inputs_json.as_deref(),
                frame: &frame,
                sampling_style: sampling_style.as_deref(),
                duplicate_resolution: duplicate_resolution.as_deref(),
                outcome_state: outcome_state.as_deref(),
                // SB-ENV-005, and OPEN: a restored version is not pre-contract, so UNKNOWN is
                // not strictly true of it - the steps that produced these values are recorded
                // on the source row, which `params_json`'s restore record names. It cannot say
                // so yet: the manifest's `params_digest` is pinned to the digest of the ROW'S
                // OWN `params_json`, and a restore appends its record to that, so the source
                // manifest cannot be copied across without breaking that invariant. Stating it
                // properly needs a step of kind "restore", which would extend SB-ENV-005's
                // signed vocabulary ("module" | "correction" | "mask" | "edit") - Jauhar's
                // ruling, not this refactor's. Explicitly `None` until then, rather than an
                // omitted column nobody can distinguish from an oversight.
                applied_steps_json: None,
            },
        )?;
        conn.execute(
            "INSERT INTO run_parameters
                (set_id, position, name, value_json, source, state, resolution, manifest_version)
             SELECT ?1, position, name, value_json, source, state, resolution, manifest_version
             FROM run_parameters WHERE set_id = ?2",
            params![new_set_id, set_id],
        )?;
        conn.execute(
            "INSERT INTO run_degradations
                (set_id, position, module, kind, detail, occurrences)
             SELECT ?1, position, module, kind, detail, occurrences
             FROM run_degradations WHERE set_id = ?2",
            params![new_set_id, set_id],
        )?;
        conn.execute(
            "DELETE FROM computed_curves
             WHERE well_id = ?2
               AND upper(curve_name) IN (SELECT DISTINCT upper(curve_name) FROM computed_curves_archive WHERE set_id = ?1)",
            params![set_id, well_id],
        )?;
        let restored = conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             SELECT well_id, depth, curve_name, value, ?2
             FROM computed_curves_archive WHERE set_id = ?1",
            params![set_id, new_set_id],
        )?;
        conn.execute(
            "INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
             SELECT ?2, well_id, depth, curve_name, value
             FROM computed_curves_archive WHERE set_id = ?1",
            params![set_id, new_set_id],
        )?;
        Ok::<(usize, i64), duckdb::Error>((restored, new_version))
    })
    .map_err(|error| error.to_string())?;

    Ok(RestoreLogSetResult {
        rows_restored,
        new_set_id,
        new_version,
        restored_from,
    })
}

/// Ordinary version deletion is not a retention policy. Keep this explicit refusal behind the
/// command so a stale frontend or saved UI cannot revive the former destructive path.
pub(crate) fn delete_log_set(_conn: &Connection, _set_id: &str) -> Result<(), String> {
    Err(
        "computed curve history is append-only; deleting a log-set version is refused. No explicit, user-visible and logged version-retention policy is configured"
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    /// CLAUDE.md states the `computed_curves` uniqueness contract - the table carries NO primary
    /// key, so uniqueness rests entirely on delete-then-append - and for a long time CLAUDE.md, the
    /// schema comment beside the table, `frame.rs`'s module doc and two more doc blocks all named
    /// `equations::write_computed_curves_batch` as the function that performs it. That function is
    /// `#[cfg(test)]`: it builds a TEST_FIXTURE ancestry and delegates, so it is a fixture front
    /// door, and a reader sent there to check or change the discipline lands in test code.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. "Name the
    /// production writer everywhere" is satisfied by pointing the prose at any production function
    /// at all; "the writer performs the DELETE" is satisfied while the prose still names the
    /// fixture. So: the fixture is still gated and is never named in production prose WITHOUT being
    /// called a fixture, AND the function the schema comment does name really performs the
    /// delete-then-append.
    #[test]
    fn the_pk_less_write_discipline_names_the_function_that_performs_it() {
        // Needles assembled, comment lines only, production halves only - so this test is never an
        // occurrence of what it counts.
        let fixture = ["write_computed_curves", "_batch"].concat();
        let writer = ["write_versioned", "_rows_raw"].concat();

        // Arm A, first half: the fixture is still gated, so naming it really would send a reader
        // into test code. House convention puts `#[cfg(test)]` on the line before the `fn`.
        let equations: Vec<&str> = include_str!("equations.rs").lines().map(str::trim).collect();
        let declared = ["pub(crate) fn ", fixture.as_str()].concat();
        let at = equations
            .iter()
            .position(|line| line.starts_with(declared.as_str()))
            .expect("the fixture is declared");
        assert_eq!(
            equations[at - 1], "#[cfg(test)]",
            "{fixture} is no longer test-gated; if it has become a production writer this whole check needs rethinking, not relaxing",
        );

        // Arm A, second half: no production comment names the fixture as the discipline. Naming it
        // is allowed only while calling it what it is.
        for (name, source) in [
            ("db.rs", include_str!("db.rs")),
            ("frame.rs", include_str!("frame.rs")),
            ("ancestry.rs", include_str!("ancestry.rs")),
            ("equations.rs", include_str!("equations.rs")),
        ] {
            let production = source
                .split("
mod tests")
                .next()
                .expect("a split always yields one piece");
            let unlabelled: Vec<&str> = production
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("//"))
                .filter(|line| line.contains(fixture.as_str()))
                .filter(|line| !line.to_ascii_lowercase().contains("fixture"))
                .collect();
            assert!(
                unlabelled.is_empty(),
                "{name} names the test fixture where the write discipline belongs: {unlabelled:?}",
            );
        }

        // Arm B: the function the schema comment names is production, and it carries the DELETE.
        assert!(
            include_str!("db.rs")
                .contains(["WRITE DISCIPLINE: `ancestry::", writer.as_str()].concat().as_str()),
            "the comment beside the PK-less table must name the writer that upholds it",
        );
        let here: Vec<&str> = include_str!("ancestry.rs").lines().map(str::trim).collect();
        let start = here
            .iter()
            .position(|line| line.starts_with(["fn ", writer.as_str()].concat().as_str()))
            .expect("the production writer is declared");
        assert_ne!(
            here[start - 1], "#[cfg(test)]",
            "the named write discipline must not itself be test-gated",
        );
        let end = here[start + 1..]
            .iter()
            .position(|line| line.starts_with("fn ") || line.starts_with("pub(crate) fn "))
            .map(|offset| start + 1 + offset)
            .unwrap_or(here.len());
        let body = here[start..end].join(" ");
        let delete = ["DELETE FROM computed", "_curves WHERE well_id"].concat();
        assert!(
            body.contains(delete.as_str()),
            "{writer} is named as the delete-then-append discipline but does not delete",
        );
    }

    /// AUDIT-2026-08-20 finding 53. `equations.rs` had grown from 2,246 to 6,892 lines because it
    /// absorbed this subsystem whole, and the highest-traffic question in the repository - how a
    /// mnemonic resolves across sets - ended up behind ~2,900 lines of custody model.
    ///
    /// Unlike the pay-summary split, this is not a one-way boundary and should not be pinned as
    /// one: `equations.rs` is a heavy CONSUMER of the ancestry API by design, so counting that
    /// direction would only measure how much work the write path does. What is worth holding is
    /// the other direction and the absence of a second copy.
    ///
    /// Both sides, because either alone has a lazier way to pass. Arm A stops this file quietly
    /// acquiring more of the resolution half - the audit's stated seam is a VALUE TYPE plus the
    /// resolver, and a widening import is how a moved subsystem drifts back into being one file
    /// in two places. Arm B stops the opposite failure: satisfying arm A by declaring a second
    /// custody model over in `equations.rs`, which is the shape every duplicated-core finding in
    /// this audit took.
    #[test]
    fn the_custody_model_reaches_the_resolver_through_one_import_and_exists_in_only_one_file() {
        let mine = include_str!("ancestry.rs");
        let production = mine.split("
mod tests").next().expect("a split always yields one piece");
        // CODE lines only; the module doc names `equations` while explaining the seam.
        let reached: Vec<&str> = production
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter(|line| line.contains(["equations", "::"].concat().as_str()))
            .collect();
        assert_eq!(
            reached.len(),
            1,
            "the resolution half is reached through ONE import; a wider seam means the split has stopped being a boundary: {reached:?}",
        );
        for item in ["resolve_generic_curve_decision", "CurveRequest", "GenericCurveDecision"] {
            assert!(reached[0].contains(item), "the seam still names {item}: {}", reached[0]);
        }

        // Arm B: the model is declared HERE and nowhere else. A second declaration compiles fine
        // and is exactly how two authoritative copies of a contract come to exist.
        let resolver = include_str!("equations.rs");
        let resolver_production =
            resolver.split("
mod tests").next().expect("a split always yields one piece");
        for kind in ["CurveAncestry", "CompleteLogSetSpec", "AncestryParameter", "LogSetSpec"] {
            assert!(
                !resolver_production.contains(["struct ", kind, " {"].concat().as_str()),
                "{kind} must be declared once, in this file",
            );
            assert!(
                production.contains(["struct ", kind, " {"].concat().as_str()),
                "{kind} is declared here",
            );
        }
    }

    /// DEC-045 / DEC-039 — the per-version comment column, the shared seam SB-POR-003, 026, 028,
    /// 047 and 048 were each blocked on ("there is nowhere to write it today"). Source: DEC-039
    /// (2026-08-16) records the branch-and-limit state as a COMMENT ON THE CURVE carried per
    /// curve version; DEC-045 authorizes exactly this column in `db.rs`.
    #[test]
    fn a_log_set_version_carries_its_own_comment_and_never_lends_it_to_another_version() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well_id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, well_id, "COMMENT-COL", None, None, Some(0.0)).unwrap();
        let well_id = well_id.to_string();
        let spec = LogSetSpec {
            set_name: "PHIE_RUNS".into(),
            module: "phi_den".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (v1, _) = create_log_set(&conn, &well_id, &spec).unwrap();
        let (v2, _) = create_log_set(&conn, &well_id, &spec).unwrap();

        // A. A comment lands on ITS version and reloads through the catalog listing.
        set_log_set_comment(&conn, &v1, "gas branch; PHIE ceiling bound at 2 samples").unwrap();
        let sets = list_log_sets(&conn, &well_id).unwrap();
        let find = |id: &str| sets.iter().find(|s| s.set_id == id).unwrap();
        assert_eq!(
            find(&v1).comment.as_deref(),
            Some("gas branch; PHIE ceiling bound at 2 samples"),
            "the comment must survive write and reload"
        );

        // B. Versions never inherit: the sibling run stays NULL — a comment describes ONE run.
        assert_eq!(find(&v2).comment, None, "a version must never lend its comment to another");

        // C. An empty text is refused: "no comment" is the NULL the row already has, and an
        //    empty string would read as a recorded nothing.
        assert!(set_log_set_comment(&conn, &v1, "   ").is_err());
        // D. A comment on a version that does not exist is an error, not a silent no-op.
        assert!(set_log_set_comment(&conn, "no-such-set", "text").is_err());
    }

    /// CORRECTNESS — SB-DBM-035 / SB-DBM-T35. The exact v1/v2 archive plus current v3,
    /// refused archive UPDATE/DELETE, restore-to-v4, source-version record and unchanged v1-v3
    /// expectations come from `22_database-model.md` §6 T35, sourced there to SB-CORE-010,
    /// F-06 and the shipped archive-purpose statement. The numeric rows are non-physical fixture
    /// labels, not petrophysical values or defaults.
    #[test]
    fn archive_updates_and_deletes_are_refused_and_restoring_version_one_creates_version_four_without_changing_versions_one_through_three() {
        use crate::db;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = uuid::Uuid::new_v4();
        db::insert_well(
            &conn,
            well_id,
            "ARCHIVE-RESTORE",
            None,
            None,
            Some(0.0),
        )
        .unwrap();
        let well_id = well_id.to_string();
        let depth = [1000.0_f32, 1000.5];
        let spec = LogSetSpec {
            set_name: "HISTORY".into(),
            module: "SB-DBM-T35 fixture".into(),
            params_json: "{\"fixture\":\"versioned rows\"}".into(),
            inputs_json: "[]".into(),
        };
        let generations = [[0.1_f32, 0.2], [0.4, 0.5], [0.7, 0.8]];
        let mut set_ids = Vec::new();
        for (index, values) in generations.iter().enumerate() {
            let (set_id, version) = create_log_set(&conn, &well_id, &spec).unwrap();
            assert_eq!(version, index as i64 + 1);
            write_computed_curves_versioned(
                &conn,
                &well_id,
                &depth,
                &[("FIXTURE_CODE", values)],
                &set_id,
            )
            .unwrap();
            set_ids.push(set_id);
        }

        let archived_rows = |set_id: &str| -> Vec<(f32, String, f32)> {
            let mut statement = conn
                .prepare(
                    "SELECT depth, curve_name, value FROM computed_curves_archive
                     WHERE set_id = ?1 ORDER BY depth, curve_name",
                )
                .unwrap();
            statement
                .query_map(duckdb::params![set_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        let before: Vec<Vec<(f32, String, f32)>> =
            set_ids.iter().map(|set_id| archived_rows(set_id)).collect();

        for sql in [
            "UPDATE computed_curves_archive SET value = 99",
            "DELETE FROM computed_curves_archive",
        ] {
            let error = db::run_readonly_query(&conn, sql, 100)
                .expect_err("the SQL reporting surface must refuse archive mutation");
            assert!(error.contains("only SELECT"), "archive mutation refusal: {error}");
        }
        let delete_error = delete_log_set(&conn, &set_ids[1])
            .expect_err("ordinary log-set deletion must not mutate append-only history");
        assert!(
            delete_error.to_string().contains("append-only"),
            "ordinary archive deletion must name the append-only rule: {delete_error}"
        );

        let receipt = restore_log_set(&conn, &set_ids[0]).unwrap();
        assert_eq!(receipt.rows_restored, 2);
        assert_eq!(receipt.new_version, 4);
        assert_eq!(receipt.restored_from.source_version, 1);
        assert_eq!(receipt.restored_from.source_set_id, set_ids[0]);
        let sets = list_log_sets(&conn, &well_id).unwrap();
        assert_eq!(
            sets.iter().map(|set| set.version).collect::<Vec<_>>(),
            vec![4, 3, 2, 1],
            "restoring v1 while v3 is current must append v4"
        );
        let restored = &sets[0];
        assert_eq!(restored.set_id, receipt.new_set_id);
        assert_eq!(restored.module, "restore");
        assert_eq!(restored.restored_from, Some(receipt.restored_from.clone()));
        let record: serde_json::Value =
            serde_json::from_str(restored.params_json.as_deref().unwrap()).unwrap();
        assert_eq!(record["_sandibumi_restore_v1"]["source_version"], 1);
        assert_eq!(
            record["_sandibumi_restore_v1"]["source_set_id"],
            set_ids[0]
        );

        let current: Vec<(String, f32, f32)> = {
            let mut statement = conn
                .prepare(
                    "SELECT set_id, depth, value FROM computed_curves
                     WHERE well_id = ?1 AND curve_name = 'FIXTURE_CODE' ORDER BY depth",
                )
                .unwrap();
            statement
                .query_map(duckdb::params![well_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        assert_eq!(
            current,
            vec![
                (restored.set_id.clone(), 1000.0, 0.1),
                (restored.set_id.clone(), 1000.5, 0.2),
            ],
            "current rows carry v1 values under the new v4 identity"
        );
        for (index, set_id) in set_ids.iter().enumerate() {
            assert_eq!(
                archived_rows(set_id),
                before[index],
                "source archive version {} changed during restore",
                index + 1
            );
        }
        assert_eq!(
            archived_rows(&restored.set_id),
            before[0],
            "v4 is an appended copy of restored v1"
        );
    }

    /// CORRECTNESS — SB-DBM-026 / SB-DBM-T25. Dossier invariant 12 and T-DB-20 require
    /// continuous sets to refuse a duplicate depth with both source rows, while POINT sets keep
    /// legitimate duplicates. F-26 cites IP's explicit 0.01 ft FPRESS perturbation; it is fixture
    /// input here, never a SandiBumi default. The PK-less store rationale is `db.rs:292-305`.
    #[test]
    fn continuous_duplicates_name_both_source_rows_while_point_duplicates_require_and_record_their_resolution() {
        use crate::db;
        use crate::schema_vocab::{DuplicateDepthResolution, SamplingStyle};
        use crate::units::{set_project_depth_unit, DepthOffset, DepthUnit};

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        set_project_depth_unit(&conn, DepthUnit::Feet).unwrap();
        let well_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_id, "DUPLICATE-DEPTH-FIXTURE", None, None, Some(0.0)).unwrap();
        let well_id = well_id.to_string();
        let depths = [1000.0_f32, 1000.0];
        let values = [0.17_f32, 0.19];

        let make_spec = |set_name: &str, curve: &str| {
            CompleteLogSetSpec::try_new(
                set_name,
                CurveAncestry {
                    schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
                    method_derivation: None,
                    module: "SB-DBM-T25 fixture".into(),
                    module_version: "fixture-build".into(),
                    inputs: Vec::new(),
                    parameters: Vec::new(),
                    parameter_state: Some(ProvenanceAbsentState::NotApplicable),
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: AncestryActor {
                        kind: AncestryActorKind::Automated,
                        identity: "SB-DBM-T25".into(),
                    },
                    timestamp_utc_ms: 1,
                    outputs: vec![AncestryOutput {
                        curve: curve.into(),
                        derivation: "SB-DBM-T25 fixture".into(),
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

        for (style, curve) in [
            (SamplingStyle::ContinuousRegular, "REGULAR_DUP"),
            (SamplingStyle::ContinuousIrregular, "IRREGULAR_DUP"),
        ] {
            let spec = make_spec(style.as_str(), curve).with_sampling_style(style);
            let (set_id, _) = create_complete_log_set(&conn, &well_id, &spec).unwrap();
            let error = write_computed_curves_with_ancestry(
                &conn,
                &well_id,
                &depths,
                &[(curve, &values)],
                &set_id,
            )
            .expect_err("a continuous duplicate must be refused");
            assert!(error.contains(curve), "{error}");
            assert!(error.contains("1000"), "{error}");
            assert!(error.contains("source rows 1 and 2"), "{error}");
            let written: i64 = conn
                .query_row(
                    "SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2",
                    duckdb::params![well_id, curve],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(written, 0, "refusal must precede any current-store mutation");
        }

        let restore_curve = "RESTORE_DUP";
        let restore_spec = make_spec("RESTORE_DUP_SET", restore_curve)
            .with_sampling_style(SamplingStyle::ContinuousRegular);
        let (restore_set, _) =
            create_complete_log_set(&conn, &well_id, &restore_spec).unwrap();
        conn.execute(
            "INSERT INTO computed_curves_archive
                (set_id, well_id, depth, curve_name, value)
             VALUES (?1, ?2, 1000.0, ?3, 0.17),
                    (?1, ?2, 1000.0, ?3, 0.19)",
            duckdb::params![restore_set.as_str(), well_id, restore_curve],
        )
        .unwrap();
        let restore_error = restore_log_set(&conn, restore_set.as_str())
            .expect_err("an archive restore is still a continuous write boundary");
        assert!(restore_error.contains(restore_curve), "{restore_error}");
        assert!(restore_error.contains("1000"), "{restore_error}");
        assert!(
            restore_error.contains("source rows 1 and 2"),
            "{restore_error}"
        );
        let restored: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves
                 WHERE well_id = ?1 AND curve_name = ?2",
                duckdb::params![well_id, restore_curve],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restored, 0, "restore refusal must precede current-store mutation");

        let point_rows = [
            db::AuxRow {
                dataset: "PRESSURE".into(),
                depth_top: 1000.0,
                depth_base: None,
                item: "FPRESS".into(),
                value_num: Some(0.17),
                value_text: None,
            },
            db::AuxRow {
                dataset: "PRESSURE".into(),
                depth_top: 1000.0,
                depth_base: None,
                item: "FPRESS".into(),
                value_num: Some(0.19),
                value_text: None,
            },
        ];
        db::insert_aux_data(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_PRESERVED",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
        )
        .expect("the shipped point-data writer must accept legitimate duplicates");
        let preserved_rows: Vec<(f32, f32)> = {
            let mut statement = conn
                .prepare(
                    "SELECT depth_top, value_num FROM aux_data
                     WHERE dataset = 'PRESSURE' AND set_name = 'POINT_PRESERVED'
                     ORDER BY value_num",
                )
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(preserved_rows, vec![(1000.0, 0.17), (1000.0, 0.19)]);
        let preserved_declaration: (String, String, i64, i64) = conn
            .query_row(
                "SELECT s.sampling_style, s.duplicate_resolution,
                        count(r.source_row), count(r.perturbation_value)
                 FROM aux_sets s
                 LEFT JOIN aux_duplicate_depth_resolutions r
                   ON r.well_id = s.well_id AND r.dataset = s.dataset AND r.set_name = s.set_name
                 WHERE s.set_name = 'POINT_PRESERVED'
                 GROUP BY s.sampling_style, s.duplicate_resolution",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            preserved_declaration,
            ("POINT".into(), "PRESERVE".into(), 2, 0),
            "preservation is declared and logged for both duplicate source rows without a made-up offset"
        );

        let missing_offset = db::insert_aux_data_with_resolution(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_NO_DEFAULT",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
            DuplicateDepthResolution::Perturb,
            None,
        )
            .expect_err("perturbation ships with no default");
        assert!(missing_offset.to_string().contains("unit-typed offset"), "{missing_offset}");

        let explicit_offset = DepthOffset {
            value: 0.01,
            unit: DepthUnit::Feet,
        };
        db::insert_aux_data_with_resolution(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_PERTURBED",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
            DuplicateDepthResolution::Perturb,
            Some(explicit_offset),
        )
        .unwrap();
        let perturbed_depths: Vec<f32> = {
            let mut statement = conn
                .prepare(
                    "SELECT depth_top FROM aux_data
                     WHERE dataset = 'PRESSURE' AND set_name = 'POINT_PERTURBED'
                     ORDER BY value_num",
                )
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(perturbed_depths, vec![1000.0, 1000.01]);
        let log: Vec<(i64, f32, f32, f64, String)> = {
            let mut statement = conn
                .prepare(
                    "SELECT r.source_row, r.original_depth, r.stored_depth,
                            r.perturbation_value, r.perturbation_unit
                     FROM aux_duplicate_depth_resolutions r
                     WHERE r.dataset = 'PRESSURE' AND r.set_name = 'POINT_PERTURBED'
                     ORDER BY source_row",
                )
                .unwrap();
            statement
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                })
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(
            log,
            vec![
                (1, 1000.0, 1000.0, 0.01, "FT".into()),
                (2, 1000.0, 1000.01, 0.01, "FT".into()),
            ]
        );
        let point_current: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves WHERE curve_name = 'FPRESS'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(point_current, 0, "POINT duplicates must never enter the current aligned store");
        let primary_keys: i64 = conn
            .query_row(
                "SELECT count(*) FROM duckdb_constraints()
                 WHERE table_name = 'computed_curves' AND constraint_type = 'PRIMARY KEY'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(primary_keys, 0, "the discipline must not be replaced by a computed_curves PK");
    }

    /// AUDIT-2026-08-20 finding 70. The `log_sets` INSERT was typed out FOUR times, and two of
    /// the four had stopped writing the same columns. One omission was deliberate and carried an
    /// SB-ENV-005 note; the other carried nothing. Nobody could tell them apart, because in SQL
    /// an OMITTED column and a deliberately NULL one are the same row.
    ///
    /// That matters for this column specifically. A NULL `applied_steps_json` is not "no steps" -
    /// the reader returns it as UNKNOWN, "the step history cannot be recovered". A writer that
    /// leaves the column out is therefore making a claim about the version it just allocated,
    /// whether it meant to or not.
    ///
    /// Pinned from BOTH sides, because either alone is satisfiable by the wrong implementation:
    /// there is exactly ONE INSERT, so a new column reaches every writer or none; and every
    /// writer that passes `None` states WHY within sight of the field, so a silent NULL cannot
    /// slip back in through the shared writer.
    #[test]
    fn one_writer_allocates_every_log_set_version_and_a_deliberate_null_manifest_says_why() {
        let source = include_str!("ancestry.rs");

        // A - one INSERT, and it belongs to `insert_log_set`. The needle is assembled at runtime
        // so this test does not count its own literal as a second writer.
        let needle = format!("INSERT INTO {}", "log_sets");
        let inserts: Vec<usize> = source
            .split('\n')
            .enumerate()
            .filter(|(_, line)| line.contains(&needle))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            inserts.len(),
            1,
            "a second hand-typed log_sets INSERT is how the four copies diverged; \
             allocate through insert_log_set instead (lines {inserts:?})"
        );
        let lines: Vec<&str> = source.split('\n').collect();
        let owner = lines[..inserts[0]]
            .iter()
            .rev()
            .find(|line| line.starts_with("pub(crate) fn ") || line.starts_with("fn "))
            .expect("the INSERT sits inside some function");
        assert!(
            owner.contains("fn insert_log_set("),
            "the one INSERT must be the shared writer's, found it in: {owner}"
        );

        // B - and every deliberate NULL manifest states its reason BESIDE the field, not
        // somewhere up the function where the next editor will not see it. Assembled at
        // runtime for the same reason as the needle above.
        let null_field = format!("applied_steps_json: {},", "None");
        let mut unexplained = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if line.trim() != null_field {
                continue;
            }
            let window = lines[index.saturating_sub(6)..index].join("\n");
            if !window.contains("SB-ENV-005") {
                unexplained.push(index + 1);
            }
        }
        assert!(
            unexplained.is_empty(),
            "a NULL manifest claims the step history cannot be recovered, so the writer must say \
             why it is making that claim (SB-ENV-005), at lines {unexplained:?}"
        );
        assert!(
            lines.iter().any(|line| line.trim() == null_field),
            "both writers that mean NULL now say so by name; if neither does, this side of the \
             pin has stopped watching anything"
        );
    }

    /// T-PETRO-02, the versioning half. Re-running a module must land as version N+1 and never
    /// overwrite the previous run, because that history is the only way to answer "which OPT_GR
    /// produced the VSH in this report" after the fact. Five runs of `vsh_gr` under one set name
    /// have to be five versions carrying five different parameter records.
    ///
    /// The per-well independence is the part with a real bug behind it. `create_log_sets_batch`
    /// pre-computes each well's next version because reading `MAX(version)` after an INSERT
    /// inside the same transaction trips a DuckDB internal error (`equations.rs:671`) — and a
    /// pre-computation that took ONE number for the whole batch would give a freshly added well
    /// its neighbours' version. Its history would then start at 7, and every earlier version of
    /// it would appear to exist and be missing.
    /// The batch is a FAST PATH, so the only thing that makes it safe is that it agrees with the
    /// function it replaces - including where that function REFUSES.
    ///
    /// Pinned from both sides deliberately. A batch that returned an empty map would satisfy
    /// "never disagrees" perfectly and be useless, so the well that does have a record is asserted
    /// present and equal; and a batch that answered everything would hide the refusals, so the
    /// mixed-spelling well and the unrecorded well are asserted absent from both.
    #[test]
    fn the_batched_ancestry_lookup_answers_exactly_what_asking_one_at_a_time_answers() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wells = [
            "aaaaaaaa-0000-0000-0000-000000000001",
            "aaaaaaaa-0000-0000-0000-000000000002",
            "aaaaaaaa-0000-0000-0000-000000000003",
        ];
        for (i, well) in wells.iter().enumerate() {
            conn.execute_batch(&format!(
                "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'SANDI-A{}');",
                i + 1
            ))
            .unwrap();
        }
        let spec = || {
            CompleteLogSetSpec::try_new(
                "PAYFLAG",
                CurveAncestry {
                    schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
                    method_derivation: None,
                    module: "TEST_FIXTURE".into(),
                    module_version: env!("CARGO_PKG_VERSION").into(),
                    inputs: Vec::new(),
                    parameters: Vec::new(),
                    parameter_state: Some(ProvenanceAbsentState::NotApplicable),
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: AncestryActor {
                        kind: AncestryActorKind::Automated,
                        identity: "rust-test-fixture".into(),
                    },
                    timestamp_utc_ms: ancestry_timestamp_utc_ms().expect("fixture timestamp"),
                    outputs: vec![AncestryOutput {
                        curve: "FLAG_PAY".into(),
                        derivation: "test_fixture:FLAG_PAY".into(),
                    }],
                    depth_frame: None,
                    zone_set: None,
                    stochastic: None,
                    applied_model: None,
                    physics_attributes: Vec::new(),
                },
            )
            .expect("complete fixture ancestry")
        };

        // Wells 1 and 2: an ordinary recorded FLAG_PAY.
        for well in &wells[..2] {
            let (set_id, _) = create_complete_log_set(&conn, well, &spec()).unwrap();
            conn.execute(
                "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
                 VALUES (?1, 1000.0, 'FLAG_PAY', 1.0, ?2)",
                params![well, set_id.value],
            )
            .unwrap();
        }
        // Well 2 ALSO carries the same curve spelled differently under a second set. That is two
        // groups to the per-call form and must stay two here - folding the spellings in SQL would
        // turn a refusal into an answer.
        let (other, _) = create_complete_log_set(&conn, wells[1], &spec()).unwrap();
        conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             VALUES (?1, 1001.0, 'flag_pay', 1.0, ?2)",
            params![wells[1], other.value],
        )
        .unwrap();
        // Well 3: rows with no log set at all - legacy, unrecorded.
        conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             VALUES (?1, 1000.0, 'FLAG_PAY', 1.0, NULL)",
            params![wells[2]],
        )
        .unwrap();

        let well_ids: Vec<String> = wells.iter().map(|well| well.to_string()).collect();
        let curves = vec!["FLAG_PAY".to_string()];
        let batched = curve_ancestry_batch(&conn, &well_ids, &curves).expect("batch resolves");

        let mut agreed = 0;
        for well in &well_ids {
            let one_at_a_time = curve_ancestry(&conn, well, "FLAG_PAY");
            match (one_at_a_time, batched.get(&(well.clone(), "FLAG_PAY".to_string()))) {
                (Ok(single), Some(many)) => {
                    assert_eq!(&single, many, "batch and per-call disagree for {well}");
                    agreed += 1;
                }
                (Err(_), None) => {}
                (single, many) => panic!(
                    "batch and per-call disagree about whether {well} HAS a record: {} vs {}",
                    single.is_ok(),
                    many.is_some()
                ),
            }
        }
        assert_eq!(agreed, 1, "exactly the one ordinary well must resolve, or this proves nothing");
        assert!(
            !batched.contains_key(&(wells[1].to_string(), "FLAG_PAY".to_string())),
            "two spellings under two sets is a refusal, not an answer"
        );
        assert!(
            !batched.contains_key(&(wells[2].to_string(), "FLAG_PAY".to_string())),
            "an unrecorded curve has no ancestry to report"
        );
    }

    /// Same argument for the input side: the fast path must produce what the per-call resolver
    /// produces, and must hand back anything it does not cover rather than guessing at it.
    #[test]
    fn the_batched_input_resolution_answers_exactly_what_asking_one_at_a_time_answers() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wells = [
            "bbbbbbbb-0000-0000-0000-000000000001",
            "bbbbbbbb-0000-0000-0000-000000000002",
        ];
        for (i, well) in wells.iter().enumerate() {
            conn.execute_batch(&format!(
                "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'SANDI-B{}');",
                i + 1
            ))
            .unwrap();
        }
        let spec = CompleteLogSetSpec::try_new(
            "INTERP",
            CurveAncestry {
                schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
                method_derivation: None,
                module: "TEST_FIXTURE".into(),
                module_version: env!("CARGO_PKG_VERSION").into(),
                inputs: Vec::new(),
                parameters: Vec::new(),
                parameter_state: Some(ProvenanceAbsentState::NotApplicable),
                zone_scope: AncestryZoneScope::WholeWell,
                actor: AncestryActor {
                    kind: AncestryActorKind::Automated,
                    identity: "rust-test-fixture".into(),
                },
                timestamp_utc_ms: ancestry_timestamp_utc_ms().expect("fixture timestamp"),
                outputs: vec![AncestryOutput {
                    curve: "PHIE".into(),
                    derivation: "test_fixture:PHIE".into(),
                }],
                depth_frame: None,
                zone_set: None,
                stochastic: None,
                applied_model: None,
                physics_attributes: Vec::new(),
            },
        )
        .expect("complete fixture ancestry");

        for well in &wells {
            let (set_id, _) = create_complete_log_set(&conn, well, &spec).unwrap();
            conn.execute(
                "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
                 VALUES (?1, 1000.0, 'PHIE', 0.2, ?2)",
                params![well, set_id.value],
            )
            .unwrap();
        }

        let requests: Vec<(String, String, String)> = wells
            .iter()
            .map(|well| (well.to_string(), "PHIE".to_string(), "PHIE".to_string()))
            .collect();
        let batched =
            resolve_ancestry_inputs_batch(&conn, &requests, None).expect("batch resolves");
        assert_eq!(batched.len(), requests.len(), "order and length are the contract");
        for (i, (well, argument, curve)) in requests.iter().enumerate() {
            let one_at_a_time =
                resolve_ancestry_input(&conn, well, argument, curve, None, None).expect("resolves");
            assert_eq!(batched[i], one_at_a_time, "batch and per-call disagree for {well}");
        }

        // A curve the fast path cannot cover falls back to the per-call resolver, which reports the
        // absence BY NAME rather than the batch inventing an identity for it.
        let missing = vec![(wells[0].to_string(), "SWE".to_string(), "SWE".to_string())];
        let refusal = resolve_ancestry_inputs_batch(&conn, &missing, None)
            .expect_err("an unresolvable input is refused, not guessed");
        assert!(
            refusal.contains("no resolvable log-set identity"),
            "the fallback must carry the per-call refusal: {refusal}"
        );
    }

    #[test]
    fn re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let a = "33333333-3333-3333-3333-333333333333";
        let b = "44444444-4444-4444-4444-444444444444";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{a}', 'SANDI-V1'), ('{b}', 'SANDI-V2');"
        ))
        .unwrap();

        let spec = |opt: &str| LogSetSpec {
            set_name: "INTERP".into(),
            module: "vsh_gr".into(),
            params_json: format!("{{\"OPT_GR\":\"{opt}\"}}"),
            inputs_json: "[\"GR\"]".into(),
        };

        // The plan's own sequence: five runs, one per OPT_GR, all into the set named INTERP.
        let opts = ["LINEAR", "LARINOV1", "LARINOV2", "STIEBER1", "CLAVIER"];
        let mut ids = Vec::new();
        for (i, opt) in opts.iter().enumerate() {
            let (set_id, version) = create_log_set(&conn, a, &spec(opt)).unwrap();
            assert_eq!(version, i as i64 + 1, "run {} of vsh_gr must be version {}", i + 1, i + 1);
            ids.push(set_id);
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), opts.len(), "each run needs its own set id, or the runs share history");

        // Every earlier run survives, and each carries the parameters that produced it — which
        // is what makes the version list answerable rather than just countable.
        let sets = list_log_sets(&conn, a).unwrap();
        let interp: Vec<_> = sets.iter().filter(|s| s.set_name == "INTERP").collect();
        assert_eq!(interp.len(), 5, "a re-run must never overwrite the version before it");
        for opt in opts {
            assert!(
                interp.iter().any(|s| s.params_json.as_deref().unwrap_or("").contains(opt)),
                "no version records OPT_GR {opt}; the tooltip could not tell them apart"
            );
        }

        // A DIFFERENT set name on the same well versions independently — INTERP's five runs must
        // not push a first run of FINAL to version 6.
        let (_, first_final) = create_log_set(&conn, a, &LogSetSpec { set_name: "FINAL".into(), ..spec("LINEAR") }).unwrap();
        assert_eq!(first_final, 1, "a set's version counts that set's own runs");

        // And a well that has never been run starts at 1, whatever its neighbours are on.
        let (_, first_b) = create_log_set(&conn, b, &spec("LINEAR")).unwrap();
        assert_eq!(first_b, 1, "version is per well, not per project");

        // The batch path must agree with the single path, per well. Well A is on 5, well B on 1,
        // so one shared number for the batch would be wrong for at least one of them.
        let batch = create_log_sets_batch(&conn, &[a.to_string(), b.to_string()], &spec("LARINOV2")).unwrap();
        assert_eq!(batch.len(), 2);
        let version_of = |well: &str, set_id: &str| -> i64 {
            list_log_sets(&conn, well).unwrap().into_iter().find(|s| s.set_id == *set_id).unwrap().version
        };
        assert_eq!(version_of(a, &batch[a]), 6, "well A had 5 INTERP runs, so the batch is its 6th");
        assert_eq!(version_of(b, &batch[b]), 2, "well B had 1, so the batch is its 2nd");
    }

    /// CORRECTNESS — `22_database-model.md` SB-DBM-003 and §6 SB-DBM-T05/T30.
    /// The values are synthetic fixture inputs, not petrophysical defaults. F-11 is the cited
    /// source for keeping an absent parameter distinct from a missing curve sample.
    ///
    /// Removing the relational row write, its state index, the NULL value/source pair, the
    /// positive sourced row, or the write refusal must fail this one contract from opposite sides.
    /// AUDIT-2026-08-20 finding 77. Two sequences were each written out three times in this file:
    /// the ancestry re-stamp (validate, parse the parameter record, insert `CURVE_ANCESTRY_KEY`,
    /// re-stringify) and the complete-write gate (right well, every curve declared, set still
    /// live, manifest parses). In both, the ORDER and the COMPLETENESS carry the guarantee rather
    /// than the lines - a copy that validated after inserting would store a manifest that never
    /// passed validation, and a copy missing one of the four checks would still compile and still
    /// write.
    ///
    /// Pinned from both sides, because either half alone has a lazier way to pass. The sequence is
    /// written once AND every producer reaches it; the gate keeps all four checks AND still names
    /// which one refused. Sharing one wording across the three producers would satisfy the count
    /// and lose the thing this repository refuses by: a user told only that curve ancestry could
    /// not be refreshed cannot tell whether the pay summary or their own equation was being
    /// recorded.
    #[test]
    fn the_ancestry_restamp_and_the_complete_write_gate_are_each_stated_once() {
        // Counted over the production half of the file only, and with every needle assembled, so
        // that this test is never an occurrence of what it counts.
        let source = include_str!("ancestry.rs");
        let production = source.split("\nmod tests").next().expect("the test module opens");
        let restamp = ["self.storage.params_json = ", "stored.to_string();"].concat();
        assert_eq!(
            production.matches(restamp.as_str()).count(),
            1,
            "the re-stamp is one sequence; a second is an order that can differ silently",
        );
        assert_eq!(
            production
                .matches(["self.restamp_ancestry(", "RestampMessages"].concat().as_str())
                .count(),
            3,
            "and all three producers reach it",
        );
        let gate = ["complete ancestry set is not", " live"].concat();
        assert_eq!(
            production.matches(gate.as_str()).count(),
            1,
            "the complete-write gate is one statement; a second can lose a check",
        );
        assert_eq!(
            production.matches(["verify_complete_set_covers", "("].concat().as_str()).count(),
            3,
            "and all three complete writers pass through it",
        );

        let fixture = || {
            CompleteLogSetSpec::try_new(
                "RESTAMP",
                CurveAncestry {
                    schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
                    method_derivation: None,
                    module: "TEST_FIXTURE".into(),
                    module_version: env!("CARGO_PKG_VERSION").into(),
                    inputs: Vec::new(),
                    parameters: Vec::new(),
                    parameter_state: Some(ProvenanceAbsentState::NotApplicable),
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: AncestryActor {
                        kind: AncestryActorKind::Automated,
                        identity: "rust-test-fixture".into(),
                    },
                    timestamp_utc_ms: ancestry_timestamp_utc_ms().expect("fixture timestamp"),
                    outputs: vec![AncestryOutput {
                        curve: "PHIE".into(),
                        derivation: "test_fixture:PHIE".into(),
                    }],
                    depth_frame: None,
                    zone_set: None,
                    stochastic: None,
                    applied_model: None,
                    physics_attributes: Vec::new(),
                },
            )
            .expect("complete fixture ancestry")
        };

        // Each producer still refuses in its OWN words. The stored parameter record is
        // unparseable here, so all three reach the same failure by three different routes - and
        // each has to say which route it was.
        let unparseable = || {
            let mut spec = fixture();
            spec.storage.params_json = "not json at all".into();
            spec
        };
        let refusals = [
            unparseable().record_run_manifest(None, Vec::new()).unwrap_err(),
            unparseable().record_parameter_decisions(&[]).unwrap_err(),
            unparseable().record_parameters_not_applicable().unwrap_err(),
        ];
        for (refusal, expected) in refusals.iter().zip([
            "cannot refresh curve ancestry manifest JSON",
            "cannot refresh curve ancestry parameter JSON",
            "cannot name the equation parameter state",
        ]) {
            assert!(
                refusal.starts_with(expected),
                "each producer refuses in its own words: expected {expected}, got {refusal}",
            );
        }

        // The gate keeps all four checks, and each one names itself.
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = "77777777-7777-7777-7777-777777777777";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'SANDI-77');"
        ))
        .unwrap();
        let spec = fixture();
        let (set_id, _) =
            create_complete_log_set(&conn, well, &spec).expect("create the fixture set");
        verify_complete_set_covers(&conn, well, ["PHIE"].into_iter(), &set_id)
            .expect("a declared output of a live set on its own well passes");
        for (well, curve, expected) in [
            ("66666666-6666-6666-6666-666666666666", "PHIE", "belongs to a different well"),
            (well, "SWE", "has no output derivation in its ancestry record"),
        ] {
            let refusal = verify_complete_set_covers(&conn, well, [curve].into_iter(), &set_id)
                .expect_err("the gate refuses");
            assert!(refusal.contains(expected), "expected {expected}, got {refusal}");
        }

        // A live set whose stored manifest carries no ancestry is refused rather than written
        // under - the fourth check, and the only one a reader could not detect afterwards.
        conn.execute(
            "UPDATE log_sets SET params_json = '{}' WHERE set_id = ?1",
            params![set_id.as_str()],
        )
        .unwrap();
        let refusal = verify_complete_set_covers(&conn, well, ["PHIE"].into_iter(), &set_id)
            .expect_err("a manifest with no ancestry is refused");
        assert!(
            refusal.contains("no complete ancestry record"),
            "expected the missing-record refusal, got {refusal}",
        );

        // And a set that is no longer there is not live.
        conn.execute("DELETE FROM log_sets WHERE set_id = ?1", params![set_id.as_str()]).unwrap();
        let refusal = verify_complete_set_covers(&conn, well, ["PHIE"].into_iter(), &set_id)
            .expect_err("a set that is gone is refused");
        assert!(refusal.contains("not live"), "expected not live, got {refusal}");
    }

    #[test]
    fn a_parameter_without_a_source_is_queryable_required_unset_and_never_a_number() {
        use crate::db;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SOURCE-STATE", None, None, Some(0.0)).unwrap();

        let ancestry = CurveAncestry {
            schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
            method_derivation: None,
            module: "synthetic_source_state_fixture".into(),
            module_version: "fixture-build".into(),
            inputs: Vec::new(),
            parameters: vec![
                AncestryParameter {
                    name: "SOURCED_FIXTURE".into(),
                    value: serde_json::json!(2.0),
                    source: "22_database-model.md §6 SB-DBM-T05 fixture input".into(),
                    resolution: Some(ParameterResolution::Explicit),
                    manifest_version: None,
                    decision: None,
                },
                AncestryParameter {
                    name: "REQUIRED_INPUT".into(),
                    value: serde_json::json!("ABSENT"),
                    source: crate::modules::ABSENT_DEFAULT_SOURCE.into(),
                    resolution: None,
                    manifest_version: None,
                    decision: None,
                },
            ],
            parameter_state: None,
            zone_scope: AncestryZoneScope::WholeWell,
            actor: AncestryActor {
                kind: AncestryActorKind::Automated,
                identity: "SB-DBM-T05".into(),
            },
            timestamp_utc_ms: 1,
            outputs: vec![AncestryOutput {
                curve: "SOURCE_STATE_RESULT".into(),
                derivation: "SB-DBM-T05 fixture".into(),
            }],

            depth_frame: None,
            zone_set: None,
            stochastic: None,
            applied_model: None,
            physics_attributes: Vec::new(),
        };
        let spec = CompleteLogSetSpec::try_new("SOURCE_STATE", ancestry).unwrap();
        let (set_id, _) = create_complete_log_set(&conn, &well_id.to_string(), &spec).unwrap();

        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM duckdb_indexes() WHERE index_name = 'idx_run_parameters_state'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, 1, "the unset-state query key must be indexed");

        let sourced: (String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT value_json, source, state FROM run_parameters
                 WHERE set_id = ?1 AND name = 'SOURCED_FIXTURE'",
                duckdb::params![set_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(&sourced.0).unwrap(), serde_json::json!(2.0));
        assert_eq!(sourced.1.as_deref(), Some("22_database-model.md §6 SB-DBM-T05 fixture input"));
        assert_eq!(sourced.2, None, "a present sourced value is not an absent state");

        let mut unset = conn
            .prepare(
                "SELECT name, value_json, source FROM run_parameters
                 WHERE state = 'REQUIRED_UNSET' ORDER BY set_id, position",
            )
            .unwrap();
        let rows: Vec<(String, Option<String>, Option<String>)> = unset
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        assert_eq!(rows, vec![("REQUIRED_INPUT".into(), None, None)]);

        let stored: String = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE set_id = ?1",
                duckdb::params![set_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        let payload: serde_json::Value = serde_json::from_str(&stored).unwrap();
        let required = &payload[CURVE_ANCESTRY_KEY]["parameters"][1];
        assert_eq!(required["state"], "REQUIRED_UNSET");
        assert!(required["value"].is_null(), "no parameter is not a numeric value");
        assert!(required["source"].is_null(), "no parameter has no invented source");

        let mut unsourced = spec.ancestry.clone();
        unsourced.parameters[0].source.clear();
        let error = CompleteLogSetSpec::try_new("SOURCE_STATE", unsourced)
            .expect_err("a UI-supplied numeric value without a source must be refused");
        assert!(error.contains("SOURCED_FIXTURE") && error.contains("source"), "{error}");

        // The migration side: a project written before the relational index existed already has
        // the source state in ancestry JSON. Re-opening must index that fact instead of silently
        // treating only future runs as queryable.
        let legacy = Connection::open_in_memory().unwrap();
        db::create_schema(&legacy).unwrap();
        let legacy_well = Uuid::new_v4();
        db::insert_well(&legacy, legacy_well, "PRE-INDEX-STATE", None, None, Some(0.0)).unwrap();
        legacy.execute_batch("DROP TABLE run_parameters").unwrap();
        let legacy_payload = serde_json::json!({
            CURVE_ANCESTRY_KEY: {
                "parameters": [{
                    "name": "LEGACY_REQUIRED_INPUT",
                    "value": "ABSENT",
                    "source": "ABSENT"
                }]
            }
        });
        let (legacy_set, _) = create_log_set(
            &legacy,
            &legacy_well.to_string(),
            &LogSetSpec {
                set_name: "PRE_INDEX".into(),
                module: "synthetic_pre_index_fixture".into(),
                params_json: legacy_payload.to_string(),
                inputs_json: "[]".into(),
            },
        )
        .unwrap();
        db::create_schema(&legacy).unwrap();
        let migrated: (Option<String>, Option<String>, String) = legacy
            .query_row(
                "SELECT value_json, source, state FROM run_parameters
                 WHERE set_id = ?1 AND name = 'LEGACY_REQUIRED_INPUT'",
                duckdb::params![legacy_set],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(migrated, (None, None, "REQUIRED_UNSET".into()));
    }


    /// PERF-PHI-DEN-2026-08-23. The degradation rows used to go in one `INSERT` at a time -
    /// 89,600 of them for a 100-well `phi_den` run, because that module clamps PHIE to PHIT and
    /// PHIT differs at every sample, so the `(kind, detail)` aggregation never collapses. They now
    /// go through ONE appender, the way the curve rows and the archive rows above them already do.
    ///
    /// Speed is the only thing that was allowed to change, so this pins what must not: the same
    /// rows, in the same order, at the same positions, and the same outcome state beside them.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. An appender
    /// that wrote nothing at all would satisfy "no row is wrong"; a flat 0,1,2,3,4 across the whole
    /// batch would satisfy "every event is present". So the CONTENT and the POSITION SEQUENCE are
    /// asserted separately, and a clean well is asserted to contribute no rows while still being
    /// classified.
    #[test]
    fn the_batched_degradation_write_records_every_event_in_its_own_position() {
        use crate::modules::{RunDegradation, RunDegradationKind};

        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let a = "55555555-5555-5555-5555-555555555555";
        let b = "66666666-6666-6666-6666-666666666666";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{a}', 'SANDI-D1'), ('{b}', 'SANDI-D2');"
        ))
        .unwrap();

        let spec = LogSetSpec {
            set_name: "INTERP".into(),
            module: "phi_den".into(),
            params_json: "{}".into(),
            inputs_json: "[\"RHOB\"]".into(),
        };
        let (set_a, _) = create_log_set(&conn, a, &spec).unwrap();
        let (set_b, _) = create_log_set(&conn, b, &spec).unwrap();

        // Synthetic fixture values, not petrophysical defaults - the point is that two events with
        // the SAME kind and different detail must stay two rows, which is exactly the shape
        // `phi_den` produces and the reason the row count is large enough to matter.
        let events = vec![
            RunDegradation {
                kind: RunDegradationKind::Clamped,
                detail: "PHIE above PHIT 0.184".into(),
                occurrences: 3,
            },
            RunDegradation {
                kind: RunDegradationKind::Clamped,
                detail: "PHIE above PHIT 0.191".into(),
                occurrences: 1,
            },
            RunDegradation {
                kind: RunDegradationKind::Defaulted,
                detail: "RHOMA".into(),
                occurrences: 7,
            },
        ];
        let well_write =
            |well_id: &str, set_id: &str, curve: &str, events: Vec<RunDegradation>| WellWrite {
                well_id: well_id.into(),
                depth: vec![1000.0, 1000.5],
                curves: vec![(curve.into(), vec![0.2, 0.21])],
                set_id: set_id.into(),
                degradation_module: Some("phi_den".into()),
                degradations: Some(events),
            };

        // Well A degraded, well B clean, in ONE batch - so a clean well sitting between degraded
        // ones cannot shift anybody's positions.
        write_computed_curves_versioned_batch(
            &conn,
            &[
                well_write(a, &set_a, "PHIE", events.clone()),
                well_write(b, &set_b, "PHIE", Vec::new()),
            ],
        )
        .unwrap();

        let stored = |set_id: &str| -> Vec<(i64, String, String, String, i64)> {
            let mut statement = conn
                .prepare(
                    "SELECT position, module, kind, detail, occurrences
                     FROM run_degradations WHERE set_id = ?1 ORDER BY position",
                )
                .unwrap();
            let rows = statement
                .query_map(params![set_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                })
                .unwrap();
            rows.map(|row| row.unwrap()).collect()
        };

        // Arm 1 - CONTENT. Every field of every event survives the appender unchanged, including
        // the two details that differ only in their last digit.
        assert_eq!(
            stored(&set_a),
            vec![
                (0, "phi_den".into(), "CLAMPED".into(), "PHIE above PHIT 0.184".into(), 3),
                (1, "phi_den".into(), "CLAMPED".into(), "PHIE above PHIT 0.191".into(), 1),
                (2, "phi_den".into(), "DEFAULTED".into(), "RHOMA".into(), 7),
            ],
            "the batched write must store exactly the events it was handed, in order"
        );
        assert!(stored(&set_b).is_empty(), "a clean run records no degradation row");

        let outcome = |set_id: &str| -> Option<String> {
            conn.query_row(
                "SELECT outcome_state FROM log_sets WHERE set_id = ?1",
                params![set_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap()
        };
        assert_eq!(outcome(&set_a).as_deref(), Some(RUN_OUTCOME_DEGRADED));
        assert_eq!(
            outcome(&set_b).as_deref(),
            Some(RUN_OUTCOME_CLEAN),
            "a well with no events is still classified - silence is not the same as unrecorded"
        );

        // Arm 2 - POSITION SEQUENCE, which the appender is the reason to doubt. The row-by-row
        // version re-read the last position before every single insert; the batch reads it once
        // per set and carries the counter forward in memory. Two cases distinguish those:
        //
        //   (a) a LATER batch continues where the earlier one stopped - the seed still comes from
        //       the table, so this fails if the read were dropped;
        //   (b) one set appearing TWICE in a single batch - those rows are not in the table yet,
        //       so this fails if the counter were not carried forward, and it fails loudly as a
        //       primary-key violation rather than quietly as an overwrite.
        write_computed_curves_versioned_batch(
            &conn,
            &[well_write(a, &set_a, "PHIT", vec![events[0].clone()])],
        )
        .unwrap();
        assert_eq!(
            stored(&set_a).iter().map(|row| row.0).collect::<Vec<_>>(),
            vec![0, 1, 2, 3],
            "a second write into the same set continues the sequence rather than restarting it"
        );

        write_computed_curves_versioned_batch(
            &conn,
            &[
                well_write(a, &set_a, "VSH", vec![events[2].clone()]),
                well_write(a, &set_a, "SWE", vec![events[1].clone()]),
            ],
        )
        .unwrap();
        assert_eq!(
            stored(&set_a).iter().map(|row| row.0).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5],
            "one set twice in one batch must not plan two rows at the same position"
        );
    }

    /// The companion to the contract above: a batch is refused WHOLE. The events are validated
    /// before anything is written, so an impossible one costs the whole batch and not just the
    /// rows that happened to follow it - which matters more now than it did, because the rows are
    /// no longer inserted as the loop walks them.
    ///
    /// `occurrences: 0` is the case a reader cannot see is wrong: `run_degradations` carries a
    /// CHECK on it, so the appender would refuse it too (measured, not assumed), but by then a
    /// well's curve rows are already in the transaction. The guard says so first, by name.
    #[test]
    fn a_batch_carrying_one_impossible_degradation_writes_none_of_them() {
        use crate::modules::{RunDegradation, RunDegradationKind};

        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let a = "77777777-7777-7777-7777-777777777777";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{a}', 'SANDI-D3');"
        ))
        .unwrap();
        let (set_a, _) = create_log_set(
            &conn,
            a,
            &LogSetSpec {
                set_name: "INTERP".into(),
                module: "phi_den".into(),
                params_json: "{}".into(),
                inputs_json: "[\"RHOB\"]".into(),
            },
        )
        .unwrap();

        let error = write_computed_curves_versioned_batch(
            &conn,
            &[WellWrite {
                well_id: a.into(),
                depth: vec![1000.0],
                curves: vec![("PHIE".into(), vec![0.2])],
                set_id: set_a.clone(),
                degradation_module: Some("phi_den".into()),
                degradations: Some(vec![
                    RunDegradation {
                        kind: RunDegradationKind::Clamped,
                        detail: "PHIE above PHIT".into(),
                        occurrences: 2,
                    },
                    RunDegradation {
                        kind: RunDegradationKind::Defaulted,
                        detail: "RHOMA".into(),
                        occurrences: 0,
                    },
                ]),
            }],
        )
        .expect_err("an event that occurred zero times is not an event");
        assert!(
            error.contains("DEFAULTED"),
            "the refusal must name which kind was impossible, got: {error}"
        );

        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM run_degradations WHERE set_id = ?1",
                params![set_a],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rows, 0, "the valid event beside the impossible one must not survive either");
    }

    /// #129: the module-input read used to REPAIR the project when a curve was missing from the
    /// generic store - `db::migrate_standard_curves_to_generic_store`, the whole project's legacy
    /// back-fill, run lazily from inside a read and then retried. One shared connection ran it
    /// once and nobody noticed; N reader connections each ran the whole thing and collided on
    /// `curve_meta`'s primary key, which is what broke the connection pool
    /// (`PERF-ATTEMPTS.md` §4).
    ///
    /// The back-fill belongs to the open. Pinned from BOTH sides, because either half alone
    /// passes for the wrong reason: "a legacy well's curve still resolves" passed on the old code
    /// too - the lazy repair is what made it pass - and "the read writes nothing" is satisfied
    /// perfectly by a read that resolves nothing at all.
    #[test]
    fn a_legacy_curve_resolves_because_the_open_backfilled_it_and_never_because_the_read_did() {
        let dir = std::env::temp_dir().join("sandibumi_ancestry_backfill_boundary");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp directory");

        // A project shaped like one made before the generic store existed: standard columns only,
        // and no `curve_migration_done` row to say the back-fill has already run.
        let legacy = |name: &str| -> (String, String) {
            let text = dir
                .join(name)
                .to_str()
                .expect("a temp path is valid UTF-8")
                .to_string();
            let conn = crate::db::init_db(&text).expect("a fresh project");
            let well = uuid::Uuid::new_v4();
            crate::db::insert_well(&conn, well, "SANDI-1", None, None, None).expect("a well");
            let n = 8usize;
            let depth: Vec<f32> = (0..n).map(|k| 1000.0 + k as f32).collect();
            let nan = vec![f32::NAN; n];
            crate::db::insert_standard_curves(
                &conn,
                well,
                depth,
                vec![40.0; n],
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan.clone(),
                nan,
            )
            .expect("standard curves");
            (text, well.to_string())
        };
        let curves_in_store = |conn: &Connection| -> i64 {
            conn.query_row("SELECT COUNT(*) FROM curve_meta", [], |row| row.get(0))
                .expect("counting curve_meta")
        };

        // Side 1 - opened the way every production open opens, the legacy curve resolves.
        let (opened_path, opened_well) = legacy("opened.duckdb");
        let opened = crate::project::open_and_migrate(&opened_path).expect("the project opens");
        assert!(
            try_resolve_ancestry_input(&opened, &opened_well, "GR", "GR", None, None)
                .expect("resolution must not error")
                .is_some(),
            "the open-time back-fill is what makes a legacy well's GR resolvable"
        );

        // Side 2 - the same project WITHOUT that open. The read must decline, and must leave the
        // store exactly as it found it. On the old code this call repaired the project and
        // returned Some, so this is the half that fails if the lazy write ever comes back.
        let (bare_path, bare_well) = legacy("bare.duckdb");
        let bare = crate::db::init_db(&bare_path).expect("the same project, unmigrated");
        let before = curves_in_store(&bare);
        assert!(
            try_resolve_ancestry_input(&bare, &bare_well, "GR", "GR", None, None)
                .expect("resolution must not error")
                .is_none(),
            "an un-backfilled store has no curve to resolve, and the read must say so"
        );
        assert_eq!(
            curves_in_store(&bare),
            before,
            "the module-input read path must never write - the back-fill belongs to the open"
        );
    }
}
