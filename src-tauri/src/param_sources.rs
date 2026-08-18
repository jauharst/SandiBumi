//! Cited competing parameter positions and the interpreter decision made against them
//! (`SB-CORE-013`, `SB-MLA-031`).
//!
//! This module carries values with sources and evidence tiers. It never carries vendor algorithms,
//! lookup tables, or copied help text. A value listed here is disclosure, not a SandiBumi default:
//! the module manifests remain the authority on whether a parameter ships present or `ABSENT`.

use serde::{Deserialize, Serialize};

/// One corpus position on a parameter. `value` is text because a range and an explicit absence are
/// both evidence, and neither may be coerced into a made-up scalar.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParamSource {
    pub product: &'static str,
    pub value: &'static str,
    pub note: &'static str,
    pub source: &'static str,
    /// The exact tier label used by the owning PRD chapter (including refinements such as T1a).
    pub tier: &'static str,
}

/// Owned form persisted with curve ancestry so a future disclosure does not depend on whatever the
/// source registry happens to contain when the project is reopened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterEvidence {
    pub product: String,
    pub value: String,
    pub note: String,
    pub source: String,
    pub tier: String,
}

/// The structured decision attached to the selected parameter value in a run's ancestry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterDecision {
    pub topic: String,
    pub parameter: String,
    pub alternatives: Vec<ParameterEvidence>,
    /// Human-readable `product (value)` identities whose cited scalar/range contains the choice.
    /// Empty means the value is explicitly the interpreter's own decision, not a vendor match.
    pub selected_matches: Vec<String>,
}

impl ParameterDecision {
    /// A complete export-safe rendering. Evidence tiers and sources remain visible rather than being
    /// reduced to a bare "matched vendor X" label.
    pub fn disclosure(&self) -> String {
        let alternatives = self
            .alternatives
            .iter()
            .map(|e| {
                format!(
                    "{} {} [{}; source: {}]",
                    e.product, e.value, e.tier, e.source
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let selected = if self.selected_matches.is_empty() {
            "matches none; interpreter decision".to_string()
        } else {
            format!("agrees with {}", self.selected_matches.join(", "))
        };
        format!(
            "{}: competing positions [{}]; selected value {}",
            self.parameter, alternatives, selected
        )
    }
}

pub const CLUSTER_COUNT: &str = "cluster_count";
pub const GR_CLEAN_ENDPOINT: &str = "gr_clean_endpoint";
pub const GR_SHALE_ENDPOINT: &str = "gr_shale_endpoint";
pub const MATRIX_DENSITY: &str = "matrix_density";
pub const SHALE_DENSITY: &str = "shale_density";
pub const DRY_SHALE_DENSITY: &str = "dry_shale_density";
pub const MATRIX_NEUTRON_ENDPOINT: &str = "matrix_neutron_endpoint";
pub const SHALE_NEUTRON_ENDPOINT: &str = "shale_neutron_endpoint";
// SB-POR-007: the porosity chapter's section 5 rows that a live POR manifest actually exposes.
// A parameter whose section 5 row is absent stays untopiced rather than borrowing a neighbouring
// quantity's evidence.
pub const FLUID_DENSITY: &str = "fluid_density";
pub const FORMATION_WATER_DENSITY: &str = "formation_water_density";
pub const MAX_EFFECTIVE_POROSITY: &str = "max_effective_porosity";
pub const POROSITY_LIMIT_MODE: &str = "porosity_limit_mode";
pub const HIGH_SHALE_BRANCH_THRESHOLD: &str = "high_shale_branch_threshold";
pub const MATRIX_TRANSIT_TIME: &str = "matrix_transit_time";
pub const FLUID_TRANSIT_TIME: &str = "fluid_transit_time";
pub const SHALE_TRANSIT_TIME: &str = "shale_transit_time";
pub const SONIC_COMPACTION_CORRECTION: &str = "sonic_compaction_correction";
pub const ARCHIE_A: &str = "archie_a";
pub const ARCHIE_M: &str = "archie_m";
pub const ARCHIE_N: &str = "archie_n";
pub const FORMATION_WATER_RESISTIVITY: &str = "formation_water_resistivity";
pub const SHALE_RESISTIVITY: &str = "shale_resistivity";
pub const CUTOFF_VSH_MAX: &str = "cutoff_vsh_max";
pub const CUTOFF_PHIE_MIN: &str = "cutoff_phie_min";
pub const CUTOFF_SWE_MAX: &str = "cutoff_swe_max";
/// SB-POR-028: the shale-reduction clamp bounds — MODE-SPECIFIC per
/// `docs/PRD_v2/11_porosity.md` §5 lines 1231-1232 (chart mode clamps both axes,
/// Bateman-Konen clamps the neutron side only, wider).
pub const SHALE_REDUCTION_CLAMP: &str = "shale_reduction_clamp";

// ---------------------------------------------------------------------------
// SB-DBM-025 (DEC-026, answered by DEC-043): a constant that crosses a module
// boundary is REGISTERED with its source, and this registry is the DEFINITION
// point - every consumer re-exports from here, so the registered value and the
// value the physics runs on are the same object by construction, not by test.
// ---------------------------------------------------------------------------

/// The pilot's cross-module PHIE floor. DEC-043 (2026-08-16) ruled 0.001 over
/// `11_porosity.md` SB-POR-045's ship-absent position - the later direct product record
/// supersedes the chapter's unresolved one - and DEC-047 fixed the shape as a cited value
/// with the ruling as its source; the deviation from the chapter's literal MUST is logged
/// at DEC-047, not buried.
pub const PHIE_FLOOR: f64 = 0.001;

/// Geolog's `cgg.h` `MISS_FLOAT` null sentinel, the cited magnitude behind
/// `db::is_large_negative_null`'s computed screen bound (SB-DBM-030 / DEC-022 family).
pub const GEOLOG_MISS_FLOAT: f32 = -1.0e30;

// ---------------------------------------------------------------------------
// SB-CLY-001 (DEC-036, confirmed unchanged by DEC-060(b)): the versioned CLY
// provenance registry. v1 carries EXACTLY six things and no more: method
// identity, missing input, masked/disabled input, ENDPOINT_INVALID, COAL, and
// substitution - the last as its own field, independent of the token, because
// "which method ran and why it could not" and "what was substituted in its
// place" are different statements. Extension by the deferred SB-CLY-031/032 is
// a MIGRATION with a version bump - never a silent re-use or renumbering of an
// existing code.
// ---------------------------------------------------------------------------

pub const CLY_PROV_REGISTRY_VERSION: u32 = 1;

/// One registry code: the number stored in the token curve, its stable name, and what it
/// asserts about the sample.
pub struct ClyProvEntry {
    pub code: f32,
    pub token: &'static str,
    pub meaning: &'static str,
}

pub const CLY_PROV_COMPUTED: f32 = 0.0;
pub const CLY_PROV_MISSING_INPUT: f32 = 1.0;
pub const CLY_PROV_MASKED_INPUT: f32 = 2.0;
pub const CLY_PROV_ENDPOINT_INVALID: f32 = 3.0;
pub const CLY_PROV_COAL: f32 = 4.0;

pub const CLY_PROV_CODES: [ClyProvEntry; 5] = [
    ClyProvEntry {
        code: CLY_PROV_COMPUTED,
        token: "COMPUTED",
        meaning: "a computed value was emitted at this sample",
    },
    ClyProvEntry {
        code: CLY_PROV_MISSING_INPUT,
        token: "MISSING_INPUT",
        meaning: "an input the method needs is absent at this sample",
    },
    ClyProvEntry {
        code: CLY_PROV_MASKED_INPUT,
        token: "MASKED_INPUT",
        meaning: "the sample was excluded by the run's mask (rule 11); written by the runner, which owns the mask",
    },
    ClyProvEntry {
        code: CLY_PROV_ENDPOINT_INVALID,
        token: "ENDPOINT_INVALID",
        meaning: "the endpoint pair is degenerate (clean >= shale), so no computed value exists by SB-CLY-001's own MUST",
    },
    ClyProvEntry {
        code: CLY_PROV_COAL,
        token: "COAL",
        meaning: "reserved for the SB-CLY-036 coal branch - defined in v1 so a later emitter cannot renumber, emitted by no shipped operation yet",
    },
];

/// The registry itself. Method identity and substitution are REGISTRY-STRUCTURE FIELDS, not
/// per-sample codes (DEC-036): "which method the token curve describes" is one statement per
/// registry, and substitution is its own independent channel.
pub struct ClyProvRegistry {
    pub version: u32,
    /// Method identity: the method family whose provenance the v1 token curve describes.
    pub method: &'static str,
    /// DEC-036 constraint 5 pins substitution independence from BOTH sides WHERE a
    /// substitution happens. No substituting operation exists in the approved
    /// CLAY_LINEAR_GR group, so v1 declares NO substitution curve rather than emitting a
    /// dead all-zero channel - the curve ships with the first substituting operation,
    /// carried in this field so its home is already decided.
    pub substitution_curve: Option<&'static str>,
    pub codes: &'static [ClyProvEntry],
}

pub const CLY_PROV_V1: ClyProvRegistry = ClyProvRegistry {
    version: CLY_PROV_REGISTRY_VERSION,
    method: "vsh_gr (the OPT_GR linear-GR transform group)",
    substitution_curve: None,
    codes: &CLY_PROV_CODES,
};

/// Decode one stored code against the registry. `None` is the whole point: an unknown code is
/// NOT a token (DEC-036 constraint 3), and the re-import path refuses on it by name rather
/// than passing a later vocabulary through as whatever v1 happens to assign.
pub fn cly_prov_token(code: f32) -> Option<&'static str> {
    CLY_PROV_V1
        .codes
        .iter()
        .find(|entry| entry.code == code)
        .map(|entry| entry.token)
}

/// One registered cross-module constant: its value lives in the consts above (re-exported by
/// every consumer), and this row carries the audit trail - who reads it and on whose word.
pub struct CrossModuleConstant {
    pub name: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub consumers: &'static str,
    pub source: &'static str,
}

/// SB-DBM-025: the complete selected-pilot inventory of petrophysical constants that cross a
/// module boundary. Adding a shared constant means adding a row HERE with its citation -
/// an uncited entry has no business crossing a boundary.
pub const CROSS_MODULE_CONSTANTS: &[CrossModuleConstant] = &[
    CrossModuleConstant {
        name: "PHIE_FLOOR",
        value: PHIE_FLOOR,
        unit: "v/v",
        consumers: "modules.rs porosity limiting (phi_den/phi_dn/phi_dnbk high-shale kill and PHIE limit); workflow.rs pay paths",
        source: "DEC-043 (2026-08-16), shape per DEC-047; supersedes 11_porosity.md SB-POR-045's ship-absent position",
    },
    CrossModuleConstant {
        name: "GEOLOG_MISS_FLOAT",
        value: GEOLOG_MISS_FLOAT as f64,
        unit: "sentinel",
        consumers: "db.rs large-negative null screen; ingest/dlis/intake flag channels (SB-DBM-030)",
        source: "Geolog cgg.h MISS_FLOAT = -1.0e30; conversion ruling DEC-022 (2026-08-17)",
    },
    CrossModuleConstant {
        name: "C_MAD",
        value: crate::robust::C_MAD,
        unit: "dimensionless",
        consumers: "robust.rs scale estimate; condition.rs Hampel despike; distribution statistics",
        source: crate::robust::C_MAD_SOURCE,
    },
];

/// The cut-off engine is not a module-manifest run, so its ancestry attaches these explicit topic
/// identities after the generic complete-run record is constructed.
pub const PAY_PARAMETER_TOPICS: &[(&str, &str)] = &[
    ("vsh_max", CUTOFF_VSH_MAX),
    ("phie_min", CUTOFF_PHIE_MIN),
    ("swe_max", CUTOFF_SWE_MAX),
];

macro_rules! claim {
    ($product:literal, $value:literal, $note:literal, $source:literal, $tier:literal) => {
        ParamSource {
            product: $product,
            value: $value,
            note: $note,
            source: $source,
            tier: $tier,
        }
    };
}

const CLUSTER_COUNT_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "15-20",
        "advised first-stage count, to be consolidated afterwards",
        "IP cluster_analysis.htm",
        "T2"
    ),
    claim!(
        "Interactive Petrophysics",
        "4-5",
        "advised consolidated count after merging first-stage clusters",
        "IP cluster_analysis.htm",
        "T2"
    ),
    claim!(
        "Techlog",
        "5",
        "shipped default corroborated by two modules",
        "Techlog HRA and petrophysical-groups help pages",
        "T3"
    ),
    claim!(
        "Geolog",
        "none stated",
        "no default or advised count in the Facimage help set",
        "Geolog Facimage help set",
        "T3"
    ),
    claim!(
        "SandiBumi",
        "5",
        "current starting value; not fitted or field-derived",
        "src-tauri/src/facies.rs module manifests",
        "T1"
    ),
];

const GR_CLEAN_ENDPOINT_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog",
        "10",
        "starting range only, not a universal clean endpoint",
        "Techlog petrophysics-vsh-from-gamma-ray.html",
        "T1′"
    ),
    claim!(
        "Geolog",
        "none stated",
        "defers to a well constant; no numeric default",
        "Geolog vsh_gr.info",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "auto-picked",
        "picked from the curve rather than shipped as one scalar",
        "IP clayparameters.htm",
        "T2"
    ),
];

const GR_SHALE_ENDPOINT_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog",
        "100",
        "starting range only, not a universal shale endpoint",
        "Techlog petrophysics-vsh-from-gamma-ray.html",
        "T1′"
    ),
    claim!(
        "Geolog",
        "none stated",
        "defers to a well constant; no numeric default",
        "Geolog vsh_gr.info",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "auto-picked",
        "picked from the curve rather than shipped as one scalar",
        "IP clayparameters.htm",
        "T2"
    ),
];

const MATRIX_DENSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog documentation",
        "2.65",
        "documented VSH density-neutron matrix endpoint",
        "Techlog petrophysics-vsh-from-neutrondensity.html 2.65 g/cm3",
        "T1′"
    ),
    claim!(
        "Interactive Petrophysics / SandiMin",
        "2.65",
        "sandstone or quartz endpoint corroborating the Techlog position",
        "ip_ingest/E_threeway_endpoint_compare.json",
        "T3"
    ),
    claim!(
        "Geolog",
        "2.645",
        "shipped sandstone matrix-density default",
        "Geolog vsh_dn.info and phi_den.info RHO_MA DEFAULT 2645 k/m3",
        "T1"
    ),
];

const SHALE_DENSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "none stated",
        "requires an entered value when density is selected",
        "IP swparameters.htm",
        "T2"
    ),
    claim!(
        "Geolog",
        "none stated",
        "references the RHO_SH well constant without a number",
        "Geolog vsh_dn.info and phi_den.info",
        "T1"
    ),
    claim!(
        "Techlog documentation",
        "2.4",
        "documented shale density",
        "Techlog petrophysics-vsh-from-neutrondensity.html; effective-porosity-from-density.html",
        "T1′"
    ),
    claim!(
        "Techlog template",
        "2.45",
        "shipped VSH density-neutron shale endpoint",
        "Techlog C2_method_defaults.json RHOB_shale = 2.45 g/cm3 and all four Q*_PR.xml",
        "T3"
    ),
    claim!(
        "Techlog script",
        "2.5",
        "shipped script parameter; differs from its documentation",
        "Techlog PorosityAndLithologyComputation.py DEN_shale",
        "T1"
    ),
];

const DRY_SHALE_DENSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "2.78",
        "Rho Dry Clay",
        "IP 2018/2025 porosity parameter help",
        "T2"
    ),
    claim!(
        "Techlog",
        "2.85",
        "dry shale; the source names a different quantity from IP",
        "Techlog PorosityAndLithologyComputation.py DEN_dryshale",
        "T1"
    ),
    claim!(
        "Geolog",
        "none stated",
        "RHO_DSH has validation but no numeric default",
        "Geolog phi_den.info",
        "T1"
    ),
];

const MATRIX_NEUTRON_ENDPOINT_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog documentation",
        "-0.1",
        "documented matrix neutron endpoint",
        "Techlog petrophysics-vsh-from-neutrondensity.html and petrophysics-vsh-from-thermal-neutron.html",
        "T1′"
    ),
    claim!(
        "Techlog template",
        "0",
        "shipped template matrix neutron endpoint",
        "Techlog C2_method_defaults.json NPHI_matrix = 0 and all four Q*_PR.xml",
        "T3"
    ),
    claim!(
        "Geolog",
        "none stated",
        "well constant with validation but no numeric default",
        "Geolog vsh_dn.info NPHI_MA validation -0.2:0.5",
        "T1"
    ),
];

const SHALE_REDUCTION_CLAMP_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "[1.950, 3.000] g/cc and [-0.015, 0.40] v/v",
        "chart-mode shale-reduced clamps, density and neutron",
        "Geolog V14 phi_dn.lls L292-295; docs/PRD_v2/11_porosity.md section 5 line 1231-1232",
        "T1"
    ),
    claim!(
        "Geolog",
        "[-0.015, 1.0] v/v, no density clamp",
        "Bateman-Konen-mode shale-reduced clamp, neutron side only",
        "Geolog V14 phi_dnbk.lls; docs/PRD_v2/11_porosity.md section 5 line 1231-1232",
        "T1"
    ),
];

const SHALE_NEUTRON_ENDPOINT_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog documentation",
        "0.40",
        "documented shale neutron endpoint",
        "Techlog petrophysics-vsh-from-neutrondensity.html",
        "T1′"
    ),
    claim!(
        "Techlog template",
        "0.4",
        "shipped template shale neutron endpoint; agrees with the documentation",
        "Techlog C2_method_defaults.json NPHI_shale = 0.4",
        "T3"
    ),
    claim!(
        "Techlog script",
        "0.45",
        "shipped script parameter; differs from its documentation",
        "Techlog PorosityAndLithologyComputation.py NEUT_shale",
        "T1"
    ),
    claim!(
        "Geolog",
        "none stated",
        "no numeric default",
        "Geolog vsh_dn.info and phi_dn.info",
        "T1"
    ),
];

const ARCHIE_A_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "none stated",
        "no factory value for a",
        "IP 2018 PhiSw and shaly-sand model help",
        "T2"
    ),
    claim!("Geolog", "1", "shipped default", "Geolog sw_*.info", "T1"),
    claim!("Techlog", "1", "shipped value", "Techlog Quanti parameter table", "T3"),
];

const ARCHIE_M_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "none stated",
        "no factory value for m",
        "IP 2018 PhiSw and shaly-sand model help",
        "T2"
    ),
    claim!("Geolog", "2", "shipped default", "Geolog sw_*.info", "T1"),
    claim!("Techlog", "2", "shipped value", "Techlog Quanti parameter table", "T3"),
];

const ARCHIE_N_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "none stated",
        "no factory value for n",
        "IP 2018 PhiSw and shaly-sand model help",
        "T2"
    ),
    claim!("Geolog", "2", "shipped default", "Geolog sw_*.info", "T1"),
    claim!("Techlog", "2", "shipped value", "Techlog Quanti parameter table", "T3"),
];

const FORMATION_WATER_RESISTIVITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "none stated",
        "RW/RWS/SALW are required inputs",
        "Geolog sw_*.info",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "0.1",
        "factory value accompanied by a warning to adjust it",
        "IP2018 §3.1 water-resistivity parameter help",
        "T2"
    ),
    claim!(
        "Techlog",
        "0.03",
        "factory value without the IP warning",
        "Techlog Quanti parameter table",
        "T3"
    ),
];

const SHALE_RESISTIVITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "none stated",
        "input log with no numeric default",
        "Geolog shaly-sand saturation manifests",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "interpreter-picked",
        "picked by the interpreter rather than defaulted",
        "IP shaly-sand parameter help",
        "T2"
    ),
    claim!(
        "Techlog",
        "5",
        "shipped shale-resistivity value",
        "Techlog Quanti parameter table",
        "T3"
    ),
];

const CUTOFF_VSH_MAX_SOURCES: &[ParamSource] = &[
    claim!("Interactive Petrophysics", "0.5", "manual report cutoff", "IP Reports 1-4 help", "T2"),
    claim!("Techlog", "0.5", "shipped VSH_max", "Techlog SummariesMonteCarlo.py", "T1a"),
    claim!("Geolog", "0.3", "vshale-only pay-summary configuration", "Geolog vshale-only_*.paysum", "T1b"),
    claim!("Geolog", "0.5", "deterministic Monte Carlo cutoff", "Geolog determin_mc.info", "T1b"),
];

const CUTOFF_PHIE_MIN_SOURCES: &[ParamSource] = &[
    claim!("Interactive Petrophysics", "0.1", "manual report cutoff", "IP Reports 1-4 help", "T2"),
    claim!("Techlog", "0.15", "shipped POR_min", "Techlog SummariesMonteCarlo.py", "T1a"),
    claim!("Geolog", "0.08", "pay-summary and deterministic Monte Carlo cutoff", "Geolog default_*.paysum and determin_mc.info", "T1b"),
    claim!("Geolog", "0", "permissive sensitivity cutoff", "Geolog tp_pay_sensitivity.info", "T1b"),
];

const CUTOFF_SWE_MAX_SOURCES: &[ParamSource] = &[
    claim!("Interactive Petrophysics", "0.5", "manual report cutoff", "IP Reports 1-4 help", "T2"),
    claim!("Techlog", "0.85", "shipped SW_max", "Techlog SummariesMonteCarlo.py", "T1a"),
    claim!("Geolog", "0.5", "deterministic Monte Carlo cutoff", "Geolog determin_mc.info", "T1b"),
    claim!("Geolog", "1", "permissive sensitivity cutoff", "Geolog tp_pay_sensitivity.info", "T1b"),
];

// SB-POR-007 / docs/PRD_v2/11_porosity.md section 5. Every claim below is transcribed from a
// section 5 row, including the rows whose SandiBumi parameter deliberately ships ABSENT: an
// attested vendor value is disclosure, never a default, and `with_sources` exists precisely so
// registering one cannot change whether the parameter has a default.
const FLUID_DENSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "1.00",
        "fresh water",
        "IP basicloganalysis.htm verbatim: Defaults to 1.0 gm/cc for fresh water",
        "T2"
    ),
    claim!(
        "Geolog",
        "1.00",
        "RHO_FL DEFAULT 1000 k/m3",
        "Geolog V14 phi_den.info RHO_FL",
        "T1"
    ),
    claim!(
        "Techlog",
        "1",
        "RHOB_fluid on the effective-porosity-from-density page",
        "Techlog 2018 petrophysics-effective-porosity-from-density.html",
        "T3"
    ),
    claim!(
        "Interactive Petrophysics",
        "1.10",
        "salt water; a different formation, not a competing default",
        "IP basicloganalysis.htm verbatim: Set to 1.1 gm/cc for salt water",
        "T2"
    ),
];

const FORMATION_WATER_DENSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "1.00",
        "RHO_W DEFAULT 1000 k/m3, validation 500:2000",
        "Geolog V14 phi_den.info RHO_W",
        "T1"
    ),
    claim!(
        "Techlog",
        "1",
        "the neutron-density crossplot anchors on MUD FILTRATE density, a different quantity that happens to share the value",
        "Techlog 2018 petrophysics-porosity-neutrondensity-crossplot.html rho_mf",
        "T3-eq"
    ),
];

const MAX_EFFECTIVE_POROSITY_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "0.30",
        "PHIE_MAX in phi_den.info, phi_dn.info and phi_son.info alike",
        "Geolog V14 phi_*.info PHIE_MAX",
        "T1"
    ),
    claim!(
        "Techlog",
        "0.35",
        "PHImax caps TOTAL porosity, so it is not the same ceiling",
        "Techlog 2018 PorosityAndLithologyComputation.py PHImax",
        "T1"
    ),
];

/// SB-POR-043 / F21 (`11_porosity.md:488-494`). Three products, three different behaviours and
/// no shared convention: a hard literal, a defaultless user parameter, and off. The tiers differ
/// too, which is the point of disclosing them - only the Geolog position is executable source.
const HIGH_SHALE_BRANCH_THRESHOLD_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "0.95",
        "hard-coded VSH >= 0.95 in all six phi_* modules => PHIE = 0, PHIT = PHIT_SH, MTH_PHI = SHALE",
        "Geolog V14 phi_*.lls; docs/PRD_v2/11_porosity.md F21 and section 5 line 1229",
        "T1"
    ),
    claim!(
        "IP",
        "ABSENT",
        "a user parameter Vcl Limit => Phie = 0.0001, all Sw = 1.0, PHIFLAG 9; IP publishes NO numeric default",
        "IP2018 A_porosity_sw.md Vcl Limit; docs/PRD_v2/11_porosity.md F21",
        "T2"
    ),
    claim!(
        "Techlog",
        "OFF",
        "LimitPhi defaults to Do Not Constrain Porosity, so no high-shale branch runs at all",
        "Techlog 2018 LimitPhi default; docs/PRD_v2/11_porosity.md F21",
        "T3"
    ),
];

const POROSITY_LIMIT_MODE_SOURCES: &[ParamSource] = &[
    claim!(
        "Geolog",
        "SHALE_REDUCED",
        "OPT_PHIEMAX DEFAULT across every porosity module",
        "Geolog V14 phi_*.info OPT_PHIEMAX",
        "T1"
    ),
    claim!(
        "Techlog",
        "no constraint",
        "Techlog applies no porosity ceiling by default; recorded, not adopted",
        "Techlog 2018 PorosityAndLithologyComputation.py",
        "T1"
    ),
];

const MATRIX_TRANSIT_TIME_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "56.0",
        "sandstone, Wyllie; corroborated by PhiSw.hlp",
        "IP swparameters.htm Default 56",
        "T2"
    ),
    claim!(
        "Geolog",
        "55.5",
        "sandstone; DT_MA 182.1 us/m, inherited by Wyllie and paired with EXP_AFF 1.60 in the AFF table",
        "Geolog V14 phi_son.info DT_MA and phi_son.lls AFF table",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "49.0",
        "limestone, Wyllie",
        "IP swparameters.htm Sonic Lime Default 49",
        "T2"
    ),
    claim!(
        "Geolog",
        "47.6",
        "limestone, AFF/Raiga, paired with EXP_AFF 1.76; the field-observed table instead gives 49.0",
        "Geolog V14 phi_son.lls AFF table",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "44.0",
        "dolomite, Wyllie",
        "IP swparameters.htm Sonic Dol Default 44",
        "T2"
    ),
    claim!(
        "Geolog",
        "43.5",
        "dolomite, AFF/Raiga, paired with EXP_AFF 2.00; the field-observed table instead gives 44.0",
        "Geolog V14 phi_son.lls AFF table",
        "T1"
    ),
];

const FLUID_TRANSIT_TIME_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "189",
        "fresh",
        "IP swparameters.htm Sonic water Default 189",
        "T2"
    ),
    claim!(
        "Geolog",
        "189",
        "DT_FL 620 us/m",
        "Geolog V14 phi_son.info DT_FL",
        "T1"
    ),
    claim!(
        "Techlog",
        "189",
        "shipped value on the sonic porosity page",
        "Techlog 2018 petrophysics-effective-porosity-from-sonic.html",
        "T3"
    ),
    claim!(
        "Interactive Petrophysics",
        "174",
        "salt-saturated formation water, stated as approximate",
        "IP basicloganalysis.htm verbatim: For salt-saturated formation water use about 174 usec/ft",
        "T2"
    ),
];

const SHALE_TRANSIT_TIME_SOURCES: &[ParamSource] = &[
    claim!(
        "Techlog",
        "100",
        "DTshale in the shipped script",
        "Techlog 2018 PorosityAndLithologyComputation.py DTshale",
        "T1"
    ),
    claim!(
        "Geolog",
        "none stated",
        "validation 150:600 us/m with no numeric default",
        "Geolog V14 phi_son.info DT_SH",
        "T1"
    ),
    claim!(
        "Interactive Petrophysics",
        "none stated",
        "a value must be entered",
        "IP swparameters.htm",
        "T2"
    ),
];

const SONIC_COMPACTION_CORRECTION_SOURCES: &[ParamSource] = &[
    claim!(
        "Interactive Petrophysics",
        "1.0",
        "the compaction factor itself",
        "IP swparameters.htm Sonic Cp (Default 1.0)",
        "T2"
    ),
    claim!("Geolog", "1", "BCP DEFAULT", "Geolog V14 phi_son.info BCP", "T1"),
    claim!(
        "Interactive Petrophysics",
        "Cp = DTSH/100 only when DTSH > 100 us/ft",
        "the greater-than-100 guard is part of the cited rule, not an addition to it",
        "IP basicloganalysis.htm rule of thumb",
        "T2"
    ),
    claim!(
        "Geolog",
        "PHIE_SON scaled by 328.084/DT_SH only when DT_SH > 328.084 us/m",
        "the same guarded rule expressed at the 100 us/ft equivalent",
        "Geolog V14 phi_son.lls",
        "T1"
    ),
];

pub fn sources_for(topic: &str) -> &'static [ParamSource] {
    match topic {
        CLUSTER_COUNT => CLUSTER_COUNT_SOURCES,
        GR_CLEAN_ENDPOINT => GR_CLEAN_ENDPOINT_SOURCES,
        GR_SHALE_ENDPOINT => GR_SHALE_ENDPOINT_SOURCES,
        MATRIX_DENSITY => MATRIX_DENSITY_SOURCES,
        SHALE_DENSITY => SHALE_DENSITY_SOURCES,
        DRY_SHALE_DENSITY => DRY_SHALE_DENSITY_SOURCES,
        MATRIX_NEUTRON_ENDPOINT => MATRIX_NEUTRON_ENDPOINT_SOURCES,
        SHALE_NEUTRON_ENDPOINT => SHALE_NEUTRON_ENDPOINT_SOURCES,
        FLUID_DENSITY => FLUID_DENSITY_SOURCES,
        FORMATION_WATER_DENSITY => FORMATION_WATER_DENSITY_SOURCES,
        MAX_EFFECTIVE_POROSITY => MAX_EFFECTIVE_POROSITY_SOURCES,
        POROSITY_LIMIT_MODE => POROSITY_LIMIT_MODE_SOURCES,
        HIGH_SHALE_BRANCH_THRESHOLD => HIGH_SHALE_BRANCH_THRESHOLD_SOURCES,
        MATRIX_TRANSIT_TIME => MATRIX_TRANSIT_TIME_SOURCES,
        FLUID_TRANSIT_TIME => FLUID_TRANSIT_TIME_SOURCES,
        SHALE_TRANSIT_TIME => SHALE_TRANSIT_TIME_SOURCES,
        SONIC_COMPACTION_CORRECTION => SONIC_COMPACTION_CORRECTION_SOURCES,
        ARCHIE_A => ARCHIE_A_SOURCES,
        ARCHIE_M => ARCHIE_M_SOURCES,
        ARCHIE_N => ARCHIE_N_SOURCES,
        FORMATION_WATER_RESISTIVITY => FORMATION_WATER_RESISTIVITY_SOURCES,
        SHALE_RESISTIVITY => SHALE_RESISTIVITY_SOURCES,
        CUTOFF_VSH_MAX => CUTOFF_VSH_MAX_SOURCES,
        CUTOFF_PHIE_MIN => CUTOFF_PHIE_MIN_SOURCES,
        CUTOFF_SWE_MAX => CUTOFF_SWE_MAX_SOURCES,
        SHALE_REDUCTION_CLAMP => SHALE_REDUCTION_CLAMP_SOURCES,
        _ => &[],
    }
}

pub fn parameter_label(topic: &str) -> Option<&'static str> {
    Some(match topic {
        CLUSTER_COUNT => "cluster count",
        GR_CLEAN_ENDPOINT => "clean gamma-ray endpoint",
        GR_SHALE_ENDPOINT => "shale gamma-ray endpoint",
        MATRIX_DENSITY => "matrix density",
        SHALE_DENSITY => "shale density",
        DRY_SHALE_DENSITY => "dry shale or dry clay density",
        MATRIX_NEUTRON_ENDPOINT => "matrix neutron endpoint",
        SHALE_NEUTRON_ENDPOINT => "shale neutron endpoint",
        FLUID_DENSITY => "fluid density",
        FORMATION_WATER_DENSITY => "formation-water density",
        MAX_EFFECTIVE_POROSITY => "maximum effective porosity",
        POROSITY_LIMIT_MODE => "porosity limiting mode",
        HIGH_SHALE_BRANCH_THRESHOLD => "high-shale branch threshold",
        MATRIX_TRANSIT_TIME => "matrix transit time",
        FLUID_TRANSIT_TIME => "fluid transit time",
        SHALE_TRANSIT_TIME => "shale transit time",
        SONIC_COMPACTION_CORRECTION => "sonic compaction correction",
        ARCHIE_A => "Archie a",
        ARCHIE_M => "Archie m",
        ARCHIE_N => "Archie n",
        FORMATION_WATER_RESISTIVITY => "formation-water resistivity",
        SHALE_RESISTIVITY => "shale resistivity",
        CUTOFF_VSH_MAX => "maximum VSH cutoff",
        CUTOFF_PHIE_MIN => "minimum PHIE cutoff",
        CUTOFF_SWE_MAX => "maximum SWE cutoff",
        _ => return None,
    })
}

/// Stable inventory used by the acceptance test and by any future source-browser surface.
// ---------------------------------------------------------------------------
// SB-CUT-017 — every default this domain ships carries a machine-readable source
// ---------------------------------------------------------------------------

/// SB-CUT-017. A default SandiBumi SHIPS, paired with the citation that authorises it.
///
/// **The value lives in the entry.** That is the whole design: a shipped default cannot exist
/// without a source because there is nowhere else to put the number. It is the same structural
/// property `ArgSpec.default_source` already gives module parameters — the difference between a
/// convention and a contract is whether a machine enforces it — extended to the defaults that are
/// NOT module parameters, which is where this domain's gap was: the pay summary is not a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainDefault {
    /// Stable machine identity, `<domain>.<name>`.
    pub id: &'static str,
    /// The value as shipped, spelled as the code spells it.
    pub value: &'static str,
    /// The requirement that owns this value. Never blank: a number nobody owns is a number nobody
    /// can defend in a client review.
    pub owner: &'static str,
    /// A citation identifying a checkable artefact, or the exact token `ABSENT` when the shipped
    /// value has none — in which case `divergence` must say so.
    pub source: &'static str,
    /// Empty when `source` is a citation. When the source is `ABSENT`, this states what is known
    /// and who has to settle it, so an unsourced default is DISCLOSED rather than merely tolerated.
    pub divergence: &'static str,
}

/// SB-CUT-017. Every default the cut-offs / summation / Monte Carlo domain ships.
///
/// The cut-offs themselves are absent since SB-CUT-016, so they appear nowhere here — there is
/// nothing to source about a value that is not shipped. What remains is the handful of decisions
/// this domain still makes for the user.
pub const CUT_DOMAIN_DEFAULTS: &[DomainDefault] = &[
    DomainDefault {
        id: "cut.partition_tolerance",
        value: "1e-7",
        owner: "SB-CUT-005",
        source: "docs/PRD_v2/14_cutoffs-summation-mc.md:2083 (SB-CUT-T22); Techlog adjustFinal shape with the print-to-result-field refinement",
        divergence: "",
    },
    DomainDefault {
        id: "cut.saturation_average_weighting",
        value: "porosity",
        owner: "SB-CUT-009",
        source: "docs/PRD_v2/14_cutoffs-summation-mc.md:1041-1042 — all three vendors agree on the phi-weighted form",
        divergence: "",
    },
    DomainDefault {
        id: "cut.summation_frame",
        value: "MD",
        owner: "SB-CUT-012",
        source: "docs/PRD_v2/14_cutoffs-summation-mc.md:1078-1091 — the only frame whose weights SandiBumi can compute; any other refuses",
        divergence: "",
    },
    DomainDefault {
        id: "cut.mc_auto_stop_tolerance",
        value: "0.005",
        owner: "SB-CUT-039",
        source: ABSENT_DOMAIN_SOURCE,
        divergence: "The chapter's parameter table cites IP `define_monte_carlo_parameters.htm` at 0.1 %; SandiBumi ships 0.5 %, a five-fold divergence with no source of its own. SB-CUT-039 owns setting this from the cited source and sits OUTSIDE the Gate 2 scope, so it is disclosed here rather than adopted — changing it would alter when auto-stop fires.",
    },
    DomainDefault {
        id: "cut.mc_reported_percentiles",
        value: "0.10 / 0.90",
        owner: "SB-CUT-039",
        source: ABSENT_DOMAIN_SOURCE,
        divergence: "P10/P90 is the chapter's reporting vocabulary throughout, but no row cites it as SandiBumi's shipped default, and the chapter's parameter table does not carry a row for it. Registered as unsourced rather than back-filled from usage.",
    },
];

/// SB-CUT-017. The exact token a [`DomainDefault`] uses when its shipped value has no citation.
/// Deliberately the same word `ArgSpec` uses, so one grep finds every unsourced number.
pub const ABSENT_DOMAIN_SOURCE: &str = "ABSENT";

/// SB-CUT-017. The build gate. A default with no source fails it; a default that DECLARES its
/// source absent must name an owner and state the divergence, so "unsourced" can never be silent.
pub fn validate_domain_defaults(defaults: &[DomainDefault]) -> Result<(), String> {
    let mut failures = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for entry in defaults {
        if entry.id.trim().is_empty() {
            failures.push("a domain default has no id".to_string());
            continue;
        }
        if seen.contains(&entry.id) {
            failures.push(format!("{} is registered twice", entry.id));
        }
        seen.push(entry.id);
        if entry.value.trim().is_empty() {
            failures.push(format!("{} has no value", entry.id));
        }
        if entry.owner.trim().is_empty() {
            failures.push(format!("{} names no owning requirement", entry.id));
        }
        if entry.source == ABSENT_DOMAIN_SOURCE {
            if entry.divergence.trim().is_empty() {
                failures.push(format!(
                    "{} declares its source ABSENT but says nothing about what is known - an                      unsourced default must be disclosed, not merely tolerated",
                    entry.id
                ));
            }
        } else if !crate::modules::source_identifies_checkable_artefact(entry.source) {
            failures.push(format!(
                "{} source '{}' does not identify a checkable artefact locator, named publication                  or project record; a product name alone is not a source",
                entry.id, entry.source
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-CUT-017 domain-default source gate failed ({} violation{}): {}",
            failures.len(),
            if failures.len() == 1 { "" } else { "s" },
            failures.join("; ")
        ))
    }
}

// ---------------------------------------------------------------------------
// SB-SAT-043 — a saturation answer carries the paper it traces to
// ---------------------------------------------------------------------------

/// SB-SAT-043. One saturation equation, with the literature it traces to and the Worthington 1985
/// classification where a source states one.
///
/// Geolog ships published references inside every module manifest, and **no vendor carries the
/// reference through to the answer** (`docs/PRD_v2/12_saturation.md:481-484`). That is the whole
/// point of this registry: the citation is attached to the METHOD IDENTITY, so it travels with the
/// run into the ancestry record and out into the deliverable rather than sitting in a doc string
/// nobody exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaturationMethod {
    /// The module that produces the answer.
    pub module: &'static str,
    /// The persisted equation identity. A module offering two equations has one entry per
    /// equation, because the two trace to different places and the adjective in the UI does not
    /// disambiguate them — `MODIFIED` means opposite things in Geolog and IP (`:135-146`).
    pub method_id: &'static str,
    /// The literature the equation traces to, or [`RETIRED_METHOD`] / [`METHOD_OWNED_ELSEWHERE`].
    pub citation: &'static str,
    /// Worthington 1985 type where a source states one. `None` is a STATEMENT, not a gap: the
    /// record still carries the field and `worthington_source` says why it carries no number.
    pub worthington: Option<u8>,
    /// Where the classification comes from, or why none is carried. Never blank.
    pub worthington_source: &'static str,
    /// A contested or divergent attribution that must travel WITH the citation, or empty. A
    /// citation the corpus disputes is still the citation; hiding the dispute is what makes it a
    /// problem later.
    pub caution: &'static str,
}

/// SB-SAT-043. The token for a module kept only so a saved chain resolves, which computes nothing.
pub const RETIRED_METHOD: &str = "RETIRED";

/// SB-SAT-043. What the Worthington field CARRIES when no source classifies the model.
///
/// A word rather than a null, and that is not cosmetic: `CurveAncestry::validate` refuses a
/// parameter with no recorded value, and rightly — but more to the point, "no source classifies
/// this model" and "nobody recorded a classification" are different claims, and only one of them
/// is checkable. The field's `source` then says which source was consulted to reach it.
pub const WORTHINGTON_NONE_STATED: &str = "NONE-STATED";

/// SB-SAT-043. The token for a saturation module whose literature belongs to another chapter's
/// requirements. Its `caution` names the chapter, so the entry is an explicit hand-off rather than
/// an omission.
pub const METHOD_OWNED_ELSEWHERE: &str = "OWNED-ELSEWHERE";

/// SB-SAT-043. Every module in the `Saturation` category, with the paper its answer traces to.
///
/// Sourcing note: the citations are the References blocks the corpus records Geolog shipping per
/// module (`docs/PRD_v2/12_saturation.md:469-479`), quoted rather than reassembled. Where a module
/// offers two equations, both entries carry that module's References block and the equation
/// identity distinguishes them — the chapter attributes the EQUATIONS at `:137-142` and the
/// REFERENCES at `:470-471`, and it does not pair one paper to one branch. Inventing that pairing
/// would be the same class of error as inventing a default.
pub const SATURATION_METHODS: &[SaturationMethod] = &[
    SaturationMethod {
        module: "sw_arch",
        method_id: "archie_total",
        citation: "Archie 1942 Trans. AIME 146:54-62 (Geolog sw_arch.info References block; docs/PRD_v2/12_saturation.md:470)",
        worthington: None,
        worthington_source: "No source classifies Archie. Geolog states a Worthington 1985 type for sw_indo, sw_sim, sw_ws, sw_juha, sw_dual and sw_tot only (docs/PRD_v2/12_saturation.md:478-479), and SB-SAT-T59 lists archie_* as carrying none.",
        caution: "",
    },
    SaturationMethod {
        module: "sw_indo",
        method_id: "indonesia",
        citation: "Poupon & Leveaux 1971 SPWLA 12th Paper O (Geolog sw_indo.info References block; docs/PRD_v2/12_saturation.md:472)",
        worthington: Some(4),
        worthington_source: "Geolog states sw_indo as type 4, noting that Worthington fixes the saturation exponent N at 2 unlike the original formulae (docs/PRD_v2/12_saturation.md:478, :1910-1912)",
        caution: "IP cites the same Indonesia paper for its NIGERIA module (docs/PRD_v2/12_saturation.md:421); the paper attaches to this equation, not to that one.",
    },
    SaturationMethod {
        module: "sw_sim",
        method_id: "simandoux_bardon_pied",
        citation: "Simandoux 1963 Revue de l'IFP (SPWLA 'Shaly Sand' Reprint Volume 1982 translation); Bardon & Pied 1969 SPWLA 10th Paper Z (Geolog sw_sim.info References block; docs/PRD_v2/12_saturation.md:470-471, :158)",
        worthington: Some(2),
        worthington_source: "Geolog states sw_sim as type 2 (docs/PRD_v2/12_saturation.md:478-479); SB-SAT-T59 lists simandoux_* as type 2",
        caution: "This is Geolog's OPT_SIM=MODIFIED and IP's plain 'Simandoux' - the same adjective names the OTHER equation in IP and Techlog (docs/PRD_v2/12_saturation.md:137-146). The References block does not attribute the shipped a = 0.8 to either paper (:157-159).",
    },
    SaturationMethod {
        module: "sw_sim",
        method_id: "simandoux_modified_slb",
        citation: "Simandoux 1963 Revue de l'IFP (SPWLA 'Shaly Sand' Reprint Volume 1982 translation); Bardon & Pied 1969 SPWLA 10th Paper Z (Geolog sw_sim.info References block; docs/PRD_v2/12_saturation.md:470-471, :158)",
        worthington: Some(2),
        worthington_source: "Geolog states sw_sim as type 2 (docs/PRD_v2/12_saturation.md:478-479); SB-SAT-T59 lists simandoux_* as type 2",
        caution: "This is Geolog's OPT_SIM=SCHLUM and IP's/Techlog's 'Modified Simandoux' - the (1-Vsh) divisor form (docs/PRD_v2/12_saturation.md:139-142). The chapter lists Schlumberger 1989 among Geolog's references but does not attach it to this branch, so it is not claimed here.",
    },
    SaturationMethod {
        module: "sw_rtc",
        method_id: "lrlc_rtc",
        citation: "SandiBumi LRLC research, 'Study of LRLC caused by High Clay Volume and Microporosity in Pertamina Fields' (PHE UI + LAPI ITB); method math at docs/method_lrlc_rtc_imts.md, RtC sections; src-tauri/src/lrlc.rs:1-13",
        worthington: None,
        worthington_source: "No source classifies it. RtC is SandiBumi's own method, so no vendor classification applies (docs/PRD_v2/12_saturation.md:1890-1891), and Worthington 1985 predates it.",
        caution: "",
    },
    SaturationMethod {
        module: "sw_imts",
        method_id: "lrlc_imts",
        citation: "SandiBumi LRLC research, 'Study of LRLC caused by High Clay Volume and Microporosity in Pertamina Fields' (PHE UI + LAPI ITB); docs/method_lrlc_rtc_imts.md, IMTS sections; src-tauri/src/lrlc.rs:1-13. Waxman-Smits-family conductivity after Waxman & Smits 1968 SPEJ and Waxman & Thomas 1974 SPEJ, with the excess-conductivity coefficient after Juhasz 1979 SPWLA 20th Paper AA and 1981 SPWLA 22nd (docs/PRD_v2/12_saturation.md:473)",
        worthington: None,
        worthington_source: "No source classifies IMTS itself. Geolog classifies its own sw_ws as type 2 (docs/PRD_v2/12_saturation.md:478-479), but IMTS is SandiBumi's own mineral-textural scaling of that family and no source states a type for it; carrying sw_ws's number across would be a classification nobody published.",
        caution: "Contested authorship, shipped unresolved: IP attributes the clay-bound-water relation to Hill, Shirley & Klein 1979 SPWLA 20th Paper AA while Geolog attributes a paper of that exact title, same symposium, same year, same paper letter, to Juhasz; Techlog cites Juhasz 1979 The Log Analyst p 3-14. Both readings ship and neither is chosen (docs/PRD_v2/12_saturation.md:486-491, ESC-1).",
    },
    SaturationMethod {
        module: "sw_height",
        method_id: "saturation_height",
        citation: METHOD_OWNED_ELSEWHERE,
        worthington: None,
        worthington_source: "Worthington 1985 classifies resistivity saturation models; a saturation-height function is not one.",
        caution: "Saturation-height belongs to docs/PRD_v2/15_sat-height-rocktyping.md, which owns the Leverett-J and Skelt-Harrison families and their fitted-object provenance; src-tauri/src/satheight.rs:122 already cites that chapter's parameter section. Registered here so a Saturation-category module cannot go unaccounted for, not to import that chapter's requirements.",
    },
    SaturationMethod {
        module: "multimin",
        method_id: "multimin_retired",
        citation: RETIRED_METHOD,
        worthington: None,
        worthington_source: "A retired step computes no saturation, so there is nothing to classify.",
        caution: "Retired and superseded by SandiMin (src-tauri/src/multimin.rs:10-13). The spec is kept only so a saved workflow chain resolves by name and can show its stored parameters; running the step returns a message and writes no curve, so no run of it can reach the provenance record.",
    },
];

/// SB-SAT-043. The build gate. Every module in the `Saturation` category must be registered, and
/// every registered method must carry a citation identifying a checkable artefact and a statement
/// of its Worthington classification - including the statement that it has none.
///
/// The gate is over the CATEGORY rather than a hand-written list, for the same reason SB-CUT-018's
/// pane enumeration discovers rather than lists: a hand-maintained list goes stale the day someone
/// adds a model, and the model that ships without a citation would be the one nobody remembered.
pub fn validate_saturation_methods(
    modules: &[crate::modules::ModuleSpec],
    methods: &[SaturationMethod],
) -> Result<(), String> {
    let mut failures = Vec::new();
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for entry in methods {
        let identity = format!("{}/{}", entry.module, entry.method_id);
        if entry.module.trim().is_empty() || entry.method_id.trim().is_empty() {
            failures.push("a saturation method entry has no identity".to_string());
            continue;
        }
        if seen.contains(&(entry.module, entry.method_id)) {
            failures.push(format!("{identity} is registered twice"));
        }
        seen.push((entry.module, entry.method_id));
        if entry.worthington_source.trim().is_empty() {
            failures.push(format!(
                "{identity} states no Worthington classification and no reason for carrying none - \
                 an empty field reads as an oversight, which is exactly what a reader cannot check"
            ));
        }
        match entry.citation {
            RETIRED_METHOD | METHOD_OWNED_ELSEWHERE => {
                if entry.caution.trim().is_empty() {
                    failures.push(format!(
                        "{identity} carries '{}' instead of a citation but says nothing about why; \
                         a hand-off must name where the method's literature lives",
                        entry.citation
                    ));
                }
            }
            citation if !crate::modules::source_identifies_checkable_artefact(citation) => {
                failures.push(format!(
                    "{identity} citation '{citation}' does not identify a checkable artefact \
                     locator or named publication; an author's name alone is not a citation"
                ));
            }
            _ => {}
        }
    }
    for module in modules
        .iter()
        .filter(|module| module.category == "Saturation")
    {
        if !methods.iter().any(|entry| entry.module == module.name) {
            failures.push(format!(
                "saturation module '{}' ships with no registered literature citation; a saturation \
                 answer that cannot name the paper it traces to is the gap SB-SAT-043 closes",
                module.name
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "SB-SAT-043 saturation-citation gate failed ({} violation{}): {}",
            failures.len(),
            if failures.len() == 1 { "" } else { "s" },
            failures.join("; ")
        ))
    }
}

/// SB-SAT-043. The registered method for one run, by module and persisted equation identity.
pub fn saturation_method(module: &str, method_id: &str) -> Option<&'static SaturationMethod> {
    SATURATION_METHODS
        .iter()
        .find(|entry| entry.module == module && entry.method_id == method_id)
}

#[cfg(test)]
pub fn topics() -> &'static [&'static str] {
    &[
        CLUSTER_COUNT,
        GR_CLEAN_ENDPOINT,
        GR_SHALE_ENDPOINT,
        MATRIX_DENSITY,
        SHALE_DENSITY,
        DRY_SHALE_DENSITY,
        MATRIX_NEUTRON_ENDPOINT,
        SHALE_NEUTRON_ENDPOINT,
        FLUID_DENSITY,
        FORMATION_WATER_DENSITY,
        MAX_EFFECTIVE_POROSITY,
        POROSITY_LIMIT_MODE,
        HIGH_SHALE_BRANCH_THRESHOLD,
        MATRIX_TRANSIT_TIME,
        FLUID_TRANSIT_TIME,
        SHALE_TRANSIT_TIME,
        SONIC_COMPACTION_CORRECTION,
        ARCHIE_A,
        ARCHIE_M,
        ARCHIE_N,
        FORMATION_WATER_RESISTIVITY,
        SHALE_RESISTIVITY,
        CUTOFF_VSH_MAX,
        CUTOFF_PHIE_MIN,
        CUTOFF_SWE_MAX,
    ]
}

pub fn decision_for(topic: &str, value: &serde_json::Value) -> Option<ParameterDecision> {
    let selected = value.as_f64()?;
    let alternatives = sources_for(topic);
    if alternatives.is_empty() {
        return None;
    }
    let selected_matches = alternatives
        .iter()
        .filter(|entry| value_agrees(entry.value, selected))
        .map(|entry| format!("{} ({})", entry.product, entry.value))
        .collect();
    Some(ParameterDecision {
        topic: topic.to_string(),
        parameter: parameter_label(topic)?.to_string(),
        alternatives: alternatives
            .iter()
            .map(|entry| ParameterEvidence {
                product: entry.product.to_string(),
                value: entry.value.to_string(),
                note: entry.note.to_string(),
                source: entry.source.to_string(),
                tier: entry.tier.to_string(),
            })
            .collect(),
        selected_matches,
    })
}

/// Compatibility note for the ML run record. Scientific curve ancestry uses the structured record
/// above; this string is retained because the ML result metadata predates curve ancestry.
pub fn decision_note(topic: &str, value: f64) -> Option<String> {
    let decision = decision_for(topic, &serde_json::json!(value))?;
    Some(format!("{} = {value}; {}", decision.parameter, decision.disclosure()))
}

fn value_agrees(cited: &str, value: f64) -> bool {
    let text = cited.trim();
    if let Some((low, high)) = text.split_once('-') {
        if let (Ok(low), Ok(high)) = (low.trim().parse::<f64>(), high.trim().parse::<f64>()) {
            return value >= low && value <= high;
        }
        return false;
    }
    text.parse::<f64>()
        .map(|candidate| (candidate - value).abs() < 1e-9)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {

    /// SB-CUT-018 (P0). `14_cutoffs-summation-mc.md:1181-1200` — every user-facing surface that
    /// accepts or displays a cut-off **MUST** resolve it from a single shared module, no pane may
    /// hard-code a cut-off literal, and **a test MUST enumerate the panes and fail when one
    /// bypasses the authority**.
    ///
    /// The drift is documented in SandiBumi's own source: TWO disagreeing sets were copy-pasted
    /// across six panes — VSH 0.5 / PHIE **0.08** / SWE **0.5** in `monteCarloDialog` and
    /// `resultsQcPanel` against VSH 0.5 / PHIE **0.1** / SWE **0.6** in four others — while the
    /// Monte Carlo tooltip claimed *"Cutoffs match the pay summary"* when they did not.
    ///
    /// **The enumeration DISCOVERS panes rather than listing them.** A hand-maintained list is a
    /// list that goes stale the day somebody adds a pane, which is exactly how six copies happened.
    /// Any file under `src/ui` that names a cut-off field must route through `./cutoffs`; the two
    /// exemptions are explicit and are the authority itself and the source-topic table.
    #[test]
    fn every_pane_that_touches_a_cutoff_resolves_it_from_the_one_shared_authority() {
        // The authority module, and the topic table that names cut-offs without carrying values.
        const EXEMPT: [&str; 2] = ["cutoffs.ts", "paramSources.ts"];
        let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ui");
        let mut checked = 0;
        let mut panes = Vec::new();
        for entry in std::fs::read_dir(&ui).expect("the ui directory is beside the crate") {
            let path = entry.expect("a readable entry").path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if !name.ends_with(".ts") {
                continue;
            }
            let src = std::fs::read_to_string(&path).expect("a readable pane");
            if !["vsh_max", "phie_min", "swe_max"].iter().any(|f| src.contains(f)) {
                continue;
            }
            checked += 1;
            if EXEMPT.contains(&name.as_str()) {
                continue;
            }
            panes.push(name.clone());
            assert!(
                src.contains("from \"./cutoffs\""),
                "{name} accepts or displays a cut-off but does not resolve it from the shared                  authority - this is the bypass that let two disagreeing sets ship at once"
            );
            // No pane may hard-code a cut-off literal. These are the exact numbers the two
            // copy-pasted sets used, plus the other vendors' published values. Scoped to lines
            // that actually NAME a cut-off: a sweep range or a plot bound is not a cut-off and
            // legitimately has a default, and `sweepMaxIn.value = "0.3"` is one - which is why
            // "sweep" is excluded rather than the whole file being scanned flat.
            for line in src.lines() {
                let lower = line.to_ascii_lowercase();
                if lower.contains("sweep") {
                    continue;
                }
                if !["vsh", "phie", "swe"].iter().any(|f| lower.contains(f)) {
                    continue;
                }
                for banned in ["\"0.5\"", "\"0.08\"", "\"0.1\"", "\"0.6\"", "\"0.15\"", "\"0.85\"", "\"0.3\""] {
                    assert!(
                        !line.contains(banned),
                        "{name} seeds a cut-off field with {banned}; a pane may READ the                          authority, never carry its own copy: {line}"
                    );
                }
            }
        }
        assert!(
            checked >= 8,
            "expected the cut-off surfaces to be discovered, found {checked} - a pass over one or              two files would prove nothing"
        );
        assert!(
            panes.iter().any(|p| p == "dashboardPanel.ts"),
            "the Field Dashboard must be among the enumerated panes; it was the last bypass"
        );
    }


    /// SB-CUT-017 (P0). `14_cutoffs-summation-mc.md:1161-1174` — every default SandiBumi ships in
    /// this domain **MUST** carry a machine-readable source identifying the file, section or
    /// citation it came from; a default with no source **MUST FAIL THE BUILD**; and a module
    /// requiring a source-less parameter **MUST** refuse at run time with an actionable message.
    ///
    /// This is `SB-CORE-004`'s domain-level discharge, and the chapter is explicit that **the
    /// build gate IS the requirement** rather than an implementation detail: the difference
    /// between a convention and a contract is whether a machine enforces it. `ArgSpec` already
    /// gives module parameters that property structurally — the field is not optional. The gap
    /// was the defaults that are NOT module parameters, because the pay summary is not a module.
    ///
    /// The cut-off values themselves are absent since SB-CUT-016, so nothing about them needs
    /// sourcing: there is nothing to defend about a number that is not shipped.
    #[test]
    fn every_default_this_domain_ships_carries_a_checkable_source_or_declares_its_absence_and_owner()
    {
        // A — the live registry passes its own gate, and is not vacuously empty.
        validate_domain_defaults(CUT_DOMAIN_DEFAULTS).expect("the shipped registry passes");
        assert!(
            CUT_DOMAIN_DEFAULTS.len() >= 4,
            "a registry of one or two entries would pass without proving anything"
        );

        // B — the value in the registry IS the value the code ships, so the disclosure cannot
        // drift away from the behaviour it describes. Checked against the constant itself.
        let tol = CUT_DOMAIN_DEFAULTS
            .iter()
            .find(|d| d.id == "cut.partition_tolerance")
            .expect("the partition tolerance is registered");
        assert_eq!(
            tol.value.parse::<f64>().expect("a numeric value"),
            crate::workflow::PARTITION_TOLERANCE,
            "the registered value and the shipped constant must be the same number"
        );

        // C — a product name is not a source. This is the clause that makes it a gate: without it
        // "Techlog" would pass and nobody could check anything.
        let vague = [DomainDefault {
            id: "cut.probe",
            value: "0.5",
            owner: "SB-CUT-017",
            source: "Techlog",
            divergence: "",
        }];
        let err = validate_domain_defaults(&vague).expect_err("a bare product name must fail");
        assert!(err.contains("checkable artefact"), "{err}");

        // D — a default may DECLARE its source absent, but then it must name an owner and say what
        // is known. Silence is the thing being prevented, not absence.
        let silent = [DomainDefault {
            id: "cut.probe",
            value: "0.5",
            owner: "SB-CUT-017",
            source: ABSENT_DOMAIN_SOURCE,
            divergence: "",
        }];
        let err = validate_domain_defaults(&silent).expect_err("silent absence must fail");
        assert!(err.contains("disclosed"), "{err}");

        // E — and a number nobody owns fails, because a number nobody owns is a number nobody can
        // defend in a client review.
        let orphan = [DomainDefault {
            id: "cut.probe",
            value: "0.5",
            owner: "",
            source: "docs/PRD_v2/14_cutoffs-summation-mc.md:1",
            divergence: "",
        }];
        let err = validate_domain_defaults(&orphan).expect_err("an unowned default must fail");
        assert!(err.contains("owning requirement"), "{err}");

        // F — the two unsourced entries are the Monte Carlo pair owned by SB-CUT-039, which is
        // OUTSIDE the Gate 2 scope. They are disclosed rather than adopted, and this asserts the
        // disclosure actually says the useful thing: which row owns it, and by how much SandiBumi
        // diverges from the cited value. Registering them was the alternative to inventing a
        // source for a number that has none.
        let unsourced: Vec<&DomainDefault> = CUT_DOMAIN_DEFAULTS
            .iter()
            .filter(|d| d.source == ABSENT_DOMAIN_SOURCE)
            .collect();
        assert!(!unsourced.is_empty(), "the divergence is real and must stay visible");
        for entry in unsourced {
            assert!(
                entry.owner.starts_with("SB-"),
                "{} must name the requirement that owns it",
                entry.id
            );
            assert!(
                entry.divergence.len() > 40,
                "{} must say what is known, not merely that something is missing",
                entry.id
            );
        }
    }

    use super::*;

    /// CORRECTNESS — the five recorded cluster-count positions and their evidence hierarchy come from
    /// `24_ml-advanced.md` §5.23 and the original SB-MLA-031 / SB-CORE-013 source-panel contract.
    #[test]
    fn every_cluster_count_position_names_its_product_source_and_tier_and_keeps_explicit_absence() {
        let sources = sources_for(CLUSTER_COUNT);
        assert_eq!(sources.len(), 5, "the registry includes two IP stages, Techlog, Geolog, and SandiBumi");
        for source in sources {
            assert!(!source.product.is_empty(), "an unattributed value is not evidence");
            assert!(!source.value.is_empty(), "an omitted position hides disagreement");
            assert!(!source.note.is_empty(), "a bare value has no usable context");
            assert!(!source.source.is_empty(), "every position names its source");
            assert!(!source.tier.is_empty(), "every source carries its evidence tier");
        }
        assert!(
            sources
                .iter()
                .any(|source| source.product == "Geolog" && source.value == "none stated"),
            "a product stating no default is a finding, not a gap to omit"
        );
        let ours = sources
            .iter()
            .position(|source| source.product == "SandiBumi")
            .expect("the shipped starting value remains visible");
        assert!(ours > 0, "the shipped value must not be presented as the authority");
        assert_eq!(sources[ours].value, "5");
        assert!(sources[ours]
            .note
            .contains("not fitted or field-derived"));
        assert!(sources_for("no_such_topic").is_empty());
    }

    /// CORRECTNESS — expected inventory and values come from the cited §5 rows in
    /// `10_clay-volume.md`, `11_porosity.md`, `12_saturation.md`, `14_cutoffs-summation-mc.md`,
    /// and `24_ml-advanced.md`; persistence is the `SB-CORE-013` requirement itself.
    #[test]
    fn every_pilot_parameter_with_competing_values_shows_its_sources_and_tiers_and_persists_the_interpreters_choice() {
        let expected = [
            CLUSTER_COUNT,
            GR_CLEAN_ENDPOINT,
            GR_SHALE_ENDPOINT,
            MATRIX_DENSITY,
            SHALE_DENSITY,
            DRY_SHALE_DENSITY,
            MATRIX_NEUTRON_ENDPOINT,
            SHALE_NEUTRON_ENDPOINT,
            // SB-POR-007 added the porosity chapter's remaining section 5 disagreements. The
            // inventory is still exact; it grew because more of the pilot's cited conflicts are
            // now disclosed, not because the rule was relaxed.
            FLUID_DENSITY,
            FORMATION_WATER_DENSITY,
            MAX_EFFECTIVE_POROSITY,
            POROSITY_LIMIT_MODE,
            HIGH_SHALE_BRANCH_THRESHOLD,
            MATRIX_TRANSIT_TIME,
            FLUID_TRANSIT_TIME,
            SHALE_TRANSIT_TIME,
            SONIC_COMPACTION_CORRECTION,
            ARCHIE_A,
            ARCHIE_M,
            ARCHIE_N,
            FORMATION_WATER_RESISTIVITY,
            SHALE_RESISTIVITY,
            CUTOFF_VSH_MAX,
            CUTOFF_PHIE_MIN,
            CUTOFF_SWE_MAX,
        ];
        assert_eq!(topics(), expected, "the DEC-003 pilot disagreement inventory is exact");
        for topic in expected {
            let rows = sources_for(topic);
            assert!(rows.len() >= 2, "{topic} must show a real disagreement");
            for row in rows {
                assert!(!row.product.is_empty(), "{topic}: product is required");
                assert!(!row.value.is_empty(), "{topic}: value or explicit absence is required");
                assert!(!row.note.is_empty(), "{topic}: a bare number has no usable context");
                assert!(!row.source.is_empty(), "{topic}: source is required for every position");
                assert!(!row.tier.is_empty(), "{topic}: evidence tier is required");
            }
        }

        // Both sides: a cited exact value and range are matched, while an uncited choice remains
        // explicitly the interpreter's own rather than being assigned to the nearest vendor.
        let rw = decision_for(FORMATION_WATER_RESISTIVITY, &serde_json::json!(0.1)).unwrap();
        assert_eq!(rw.selected_matches, ["Interactive Petrophysics (0.1)"]);
        assert!(rw.disclosure().contains("T1"));
        assert!(rw.disclosure().contains("T2"));
        assert!(rw.disclosure().contains("T3"));
        let own = decision_for(FORMATION_WATER_RESISTIVITY, &serde_json::json!(0.2)).unwrap();
        assert!(own.selected_matches.is_empty());
        assert!(own.disclosure().contains("interpreter decision"));
        let ranged = decision_for(CLUSTER_COUNT, &serde_json::json!(17)).unwrap();
        assert!(ranged.selected_matches.iter().any(|m| m.contains("15-20")));
        assert!(decision_for("not_registered", &serde_json::json!(1)).is_none());
        assert!(decision_for(GR_CLEAN_ENDPOINT, &serde_json::json!("ABSENT")).is_none());

        // Every selected pilot module field points at the same registry used above. An editor that
        // forgot the topic would still compute correctly, so this must be pinned separately.
        let manifests = crate::modules::list_modules();
        let expected_fields = [
            ("vsh_gr", "GR_MA", GR_CLEAN_ENDPOINT),
            ("vsh_gr", "GR_SH", GR_SHALE_ENDPOINT),
            ("vsh_dn", "RHO_MA", MATRIX_DENSITY),
            ("vsh_dn", "RHO_SH", SHALE_DENSITY),
            ("vsh_dn", "NPHI_MA", MATRIX_NEUTRON_ENDPOINT),
            ("vsh_dn", "NPHI_SH", SHALE_NEUTRON_ENDPOINT),
            ("vsh_dn", "GR_MA", GR_CLEAN_ENDPOINT),
            ("vsh_dn", "GR_SH", GR_SHALE_ENDPOINT),
            ("phi_den", "RHO_MA", MATRIX_DENSITY),
            ("phi_den", "RHO_SH", SHALE_DENSITY),
            ("phi_den", "RHO_DSH", DRY_SHALE_DENSITY),
            ("phi_dn", "RHO_MA", MATRIX_DENSITY),
            ("phi_dn", "RHO_SH", SHALE_DENSITY),
            ("phi_dn", "RHO_DSH", DRY_SHALE_DENSITY),
            ("phi_dn", "NPHI_SH", SHALE_NEUTRON_ENDPOINT),
            ("sw_arch", "A", ARCHIE_A),
            ("sw_arch", "M", ARCHIE_M),
            ("sw_arch", "N", ARCHIE_N),
            ("sw_arch", "RW", FORMATION_WATER_RESISTIVITY),
            ("sw_indo", "A", ARCHIE_A),
            ("sw_indo", "M", ARCHIE_M),
            ("sw_indo", "N", ARCHIE_N),
            ("sw_indo", "RW", FORMATION_WATER_RESISTIVITY),
            ("sw_indo", "RT_SH", SHALE_RESISTIVITY),
        ];
        for (module, argument, topic) in expected_fields {
            let manifest = manifests.iter().find(|item| item.name == module).unwrap();
            let arg = manifest.args.iter().find(|item| item.name == argument).unwrap();
            assert_eq!(arg.sources_topic, topic, "{module}.{argument} source context");
        }
        assert_eq!(
            PAY_PARAMETER_TOPICS,
            [
                ("vsh_max", CUTOFF_VSH_MAX),
                ("phie_min", CUTOFF_PHIE_MIN),
                ("swe_max", CUTOFF_SWE_MAX),
            ]
        );

        // Persist through the real deterministic writer, then reopen the current curve's ancestry.
        // The first run matches a cited value; the second deliberately matches none. Either half
        // alone would allow an implementation that always labels the choice one way.
        use std::collections::HashMap;
        use std::sync::Mutex;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well_id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, well_id, "SYNTHETIC", None, None, None).unwrap();
        let depth = vec![1000.0_f32, 1000.5, 1001.0];
        let nan = vec![f32::NAN; depth.len()];
        crate::db::insert_standard_curves(
            &conn,
            well_id,
            depth,
            vec![10.0, 55.0, 100.0],
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan.clone(),
            nan,
        )
        .unwrap();
        let db = Mutex::new(conn);
        let run = |clean: f64| crate::workflow::RunModuleRequest {
            module: "vsh_gr".into(),
            well_ids: vec![well_id.to_string()],
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), clean), ("GR_SH".into(), 100.0)]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
            output_set: Some("INTERPRETATION".into()),
            input_set: None,
            custody: crate::workflow::test_run_custody(),
        };
        let first = crate::workflow::run_workflow_module(&db, &run(10.0));
        assert!(first[0].error.is_none(), "{}", first[0].error.as_deref().unwrap_or(""));
        let first_ancestry = crate::equations::curve_ancestry(
            &db.lock().unwrap(),
            &well_id.to_string(),
            "VSH",
        )
        .unwrap();
        let first_clean = first_ancestry.parameters.iter().find(|p| p.name == "GR_MA").unwrap();
        assert_eq!(
            first_clean.decision.as_ref().unwrap().selected_matches,
            ["Techlog (10)"]
        );
        assert_eq!(first_clean.source, crate::workflow::test_run_custody().source_note);

        let second = crate::workflow::run_workflow_module(&db, &run(11.0));
        assert!(second[0].error.is_none(), "{}", second[0].error.as_deref().unwrap_or(""));
        let second_ancestry = crate::equations::curve_ancestry(
            &db.lock().unwrap(),
            &well_id.to_string(),
            "VSH",
        )
        .unwrap();
        let second_clean = second_ancestry.parameters.iter().find(|p| p.name == "GR_MA").unwrap();
        assert!(second_clean.decision.as_ref().unwrap().selected_matches.is_empty());

        // The non-manifest pay engine uses the same structured record for all three cutoffs.
        let custody = crate::workflow::test_run_custody();
        let mut pay = crate::equations::complete_curve_run_spec(
            &db.lock().unwrap(),
            &well_id.to_string(),
            "PAYFLAG",
            "pay_summary",
            &custody,
            &[],
            None,
            serde_json::json!({"vsh_max": 0.5, "phie_min": 0.1, "swe_max": 0.6}),
            crate::equations::AncestryZoneScope::WholeWell,
            &["FLAG_PAY".into()],
        )
        .unwrap();
        pay.record_parameter_decisions(PAY_PARAMETER_TOPICS).unwrap();
        for name in ["vsh_max", "phie_min", "swe_max"] {
            assert!(
                pay.ancestry()
                    .parameters
                    .iter()
                    .find(|p| p.name == name)
                    .and_then(|p| p.decision.as_ref())
                    .is_some(),
                "pay-summary {name} choice is a persisted decision"
            );
        }
    }

    #[test]
    fn the_recorded_cluster_choice_distinguishes_a_cited_range_from_an_interpreters_own_value() {
        let seventeen = decision_note(CLUSTER_COUNT, 17.0).unwrap();
        assert!(seventeen.contains("Interactive Petrophysics (15-20)"));
        let nine = decision_note(CLUSTER_COUNT, 9.0).unwrap();
        assert!(nine.contains("interpreter decision"));
        assert!(!nine.contains("Geolog (none stated)"));
    }

    /// SB-DBM-025 (DEC-026 via DEC-043): a constant that crosses a module boundary is
    /// registered with its source, the registry is the DEFINITION its consumers re-export,
    /// and the inventory is COMPLETE - exactly the pilot's three, each carrying the authority
    /// it stands on. Pinned from both sides: the values are pinned absolutely (a drifted
    /// registry cannot hide behind its own re-exports), and the consumer consts are pinned
    /// bit-equal to the registry (a consumer re-literalled away from it cannot hide either).
    #[test]
    fn every_cross_module_constant_is_registered_with_its_source_and_the_registry_is_what_runs() {
        assert_eq!(CROSS_MODULE_CONSTANTS.len(), 3, "the inventory is complete and exact");
        let entry = |name: &str| {
            CROSS_MODULE_CONSTANTS
                .iter()
                .find(|c| c.name == name)
                .unwrap_or_else(|| panic!("{name} must be registered"))
        };
        // PHIE_FLOOR: DEC-043's 0.001, and the modules const IS the registry value.
        let floor = entry("PHIE_FLOOR");
        assert_eq!(floor.value.to_bits(), 0.001f64.to_bits());
        assert_eq!(crate::modules::PHIE_FLOOR.to_bits(), floor.value.to_bits());
        assert!(floor.source.contains("DEC-043"), "the ruling is the source: {}", floor.source);
        // GEOLOG_MISS_FLOAT: cgg.h's sentinel, one object with the db screen's constant.
        let miss = entry("GEOLOG_MISS_FLOAT");
        // the registry row widens the f32 sentinel to f64, so the pin compares the same cast
        assert_eq!(miss.value.to_bits(), ((-1.0e30f32) as f64).to_bits());
        assert_eq!(crate::db::GEOLOG_MISS_FLOAT.to_bits(), (-1.0e30f32).to_bits());
        assert!(miss.source.contains("cgg.h"), "{}", miss.source);
        // C_MAD: the robust scale constant rides its own long-standing source record.
        let cmad = entry("C_MAD");
        assert_eq!(cmad.value.to_bits(), crate::robust::C_MAD.to_bits());
        assert_eq!(cmad.source, crate::robust::C_MAD_SOURCE);
        // No entry crosses a boundary uncited or unclaimed.
        for constant in CROSS_MODULE_CONSTANTS {
            assert!(!constant.source.trim().is_empty(), "{} is uncited", constant.name);
            assert!(!constant.consumers.trim().is_empty(), "{} names no consumer", constant.name);
        }
    }
}
