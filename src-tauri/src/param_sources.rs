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
}
