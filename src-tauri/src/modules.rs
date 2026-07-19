//! Deterministic petrophysics module library, ported from Geolog-V14 Loglan sources
//! (vsh_gr.lls, vsh_dn.lls, phi_den.lls, phi_dn.lls, sw_arch.lls, sw_indo.lls, sw_sim.lls,
//! perm_wyllie_rose.lls, perm_coates.lls) with the same MISSING semantics (`f32::NAN`),
//! LIMIT clamping, and per-frame evaluation model.
//!
//! Each module carries a manifest (mirroring Geolog's `.info` files) that the frontend
//! uses to auto-generate its parameter dialog: numeric interval parameters with defaults
//! and validation ranges, string options with fixed choices, and input/output logs.
//!
//! Density convention: g/cc (matching LAS field data), not Geolog's kg/m3.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArgKind {
    /// Numeric interval parameter (per-zone overridable).
    Param,
    /// String option with fixed choices (global per run).
    Option,
    /// Input log curve (resolved from standard/computed curves).
    LogIn,
    /// Output log curve (written to computed_curves).
    LogOut,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgSpec {
    pub name: String,
    pub desc: String,
    pub unit: String,
    pub kind: ArgKind,
    /// Default numeric value (Param), default choice (Option), or default curve mnemonic (LogIn).
    pub default: String,
    /// Valid choices for Option args.
    pub choices: Vec<String>,
    /// Validation range for Param args.
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Whether a LogIn is required (missing optional inputs become all-NaN).
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleSpec {
    pub name: String,
    pub title: String,
    pub category: String, // "VSH" | "Porosity" | "Saturation" | "Permeability" | "Prep"
    pub doc: String,
    pub args: Vec<ArgSpec>,
}

pub(crate) fn param(name: &str, desc: &str, unit: &str, default: f64, min: f64, max: f64) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::Param,
        default: default.to_string(),
        choices: vec![],
        min: Some(min),
        max: Some(max),
        required: true,
    }
}

pub(crate) fn opt(name: &str, desc: &str, default: &str, choices: &[&str]) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: String::new(),
        kind: ArgKind::Option,
        default: default.into(),
        choices: choices.iter().map(|s| s.to_string()).collect(),
        min: None,
        max: None,
        required: true,
    }
}

pub(crate) fn log_in(name: &str, desc: &str, unit: &str, default_curve: &str, required: bool) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::LogIn,
        default: default_curve.into(),
        choices: vec![],
        min: None,
        max: None,
        required,
    }
}

pub(crate) fn log_out(name: &str, desc: &str, unit: &str) -> ArgSpec {
    ArgSpec {
        name: name.into(),
        desc: desc.into(),
        unit: unit.into(),
        kind: ArgKind::LogOut,
        default: String::new(),
        choices: vec![],
        min: None,
        max: None,
        required: true,
    }
}

/// Everything a module needs at run time, resolved by the workflow runner:
/// input logs by arg name, per-sample numeric parameter arrays (zone-resolved),
/// and global string options.
pub struct ModuleContext {
    pub n: usize,
    pub logs: HashMap<String, Vec<f32>>,
    pub params: HashMap<String, Vec<f64>>,
    pub opts: HashMap<String, String>,
}

impl ModuleContext {
    pub(crate) fn log(&self, name: &str) -> Vec<f32> {
        self.logs.get(name).cloned().unwrap_or_else(|| vec![f32::NAN; self.n])
    }
    pub(crate) fn p(&self, name: &str, i: usize) -> f64 {
        self.params.get(name).and_then(|v| v.get(i)).copied().unwrap_or(f64::NAN)
    }
    pub(crate) fn o(&self, name: &str) -> &str {
        self.opts.get(name).map(|s| s.as_str()).unwrap_or("")
    }
}

pub type ModuleOutputs = HashMap<String, Vec<f32>>;

fn limit(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_nan() {
        v
    } else {
        v.clamp(lo, hi)
    }
}

const MISSING: f64 = f64::NAN;

fn is_missing(v: f64) -> bool {
    v.is_nan()
}

/// Registry of every deterministic module manifest, in workflow order.
pub fn list_modules() -> Vec<ModuleSpec> {
    vec![
        vsh_gr_spec(),
        vsh_dn_spec(),
        phi_den_spec(),
        phi_dn_spec(),
        phi_son_spec(),
        crate::ssc::ssc_spec(),
        crate::ssc::sspw_spec(),
        ftemp_grad_spec(),
        badhole_spec(),
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
        crate::multimin::multimin_spec(),
        crate::satheight::sw_height_spec(),
        crate::facies::electrofacies_spec(),
        crate::facies::gmm_facies_spec(),
    ]
}

/// Dispatches a module run by name.
pub fn run_module(name: &str, ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    match name {
        "vsh_gr" => Ok(vsh_gr(ctx)),
        "vsh_dn" => Ok(vsh_dn(ctx)),
        "phi_den" => Ok(phi_den(ctx)),
        "phi_dn" => Ok(phi_dn(ctx)),
        "phi_son" => Ok(phi_son(ctx)),
        "ftemp_grad" => Ok(ftemp_grad(ctx)),
        "badhole" => Ok(badhole(ctx)),
        "gr_hole_corr" => Ok(gr_hole_corr(ctx)),
        "nphi_env_corr" => Ok(nphi_env_corr(ctx)),
        "rhob_hole_corr" => Ok(rhob_hole_corr(ctx)),
        "gr_normalize" => Ok(gr_normalize(ctx)),
        "log_predict" => Ok(log_predict(ctx)),
        "ssc" => Ok(crate::ssc::ssc(ctx)),
        "sspw" => Ok(crate::ssc::sspw(ctx)),
        "sw_rtc" => Ok(crate::lrlc::sw_rtc(ctx)),
        "sw_imts" => Ok(crate::lrlc::sw_imts(ctx)),
        "multimin" => Ok(crate::multimin::multimin(ctx)),
        "sw_height" => Ok(crate::satheight::sw_height(ctx)),
        "electrofacies" => Ok(crate::facies::electrofacies(ctx)),
        "gmm_facies" => Ok(crate::facies::gmm_facies(ctx)),
        "sw_arch" => Ok(sw_arch(ctx)),
        "sw_indo" => Ok(sw_indo(ctx)),
        "sw_sim" => Ok(sw_sim(ctx)),
        "perm_wyllie_rose" => Ok(perm_wyllie_rose(ctx)),
        "perm_coates" => Ok(perm_coates(ctx)),
        "perm_transform" => Ok(perm_transform(ctx)),
        "thin_bed_ts" => Ok(thin_bed_ts(ctx)),
        "depth_shift" => Ok(depth_shift(ctx)),
        "splice" => Ok(splice(ctx)),
        other => Err(format!("unknown module '{other}'")),
    }
}

// ---------------------------------------------------------------------------
// VSH_GR — Volume of shale from gamma ray (Geolog vsh_gr.lls)
// ---------------------------------------------------------------------------

fn vsh_gr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "vsh_gr".into(),
        title: "VSH from Gamma Ray".into(),
        category: "VSH".into(),
        doc: "VSH_GR = (GR - GR_MA) / (GR_SH - GR_MA), with optional non-linear corrections \
              (Stieber, Larionov, Clavier). VSH is the result limited to 0–1."
            .into(),
        args: vec![
            opt(
                "OPT_GR",
                "VSH from gamma ray method",
                "LINEAR",
                &["LINEAR", "STIEBER1", "STIEBER2", "STIEBER3", "LARINOV1", "LARINOV2", "LARINOV3", "CLAVIER"],
            ),
            param("GR_MA", "Gamma ray matrix (clean)", "gapi", 20.0, 0.0, 200.0),
            param("GR_SH", "Gamma ray shale", "gapi", 120.0, 0.0, 1000.0),
            log_in("GR", "Gamma ray log", "gapi", "GR", true),
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
// VSH_DN — Volume of shale from density-neutron crossplot (Geolog vsh_dn.lls)
// ---------------------------------------------------------------------------

fn vsh_dn_spec() -> ModuleSpec {
    ModuleSpec {
        name: "vsh_dn".into(),
        title: "VSH from Density-Neutron".into(),
        category: "VSH".into(),
        doc: "Two-log crossplot VSH: the (RHOB, NPHI) point's position between the clean \
              matrix line and the shale point. Density in g/cc."
            .into(),
        args: vec![
            param("RHO_MA", "Matrix density", "g/cc", 2.645, 2.0, 3.2),
            param("RHO_SH", "Shale density", "g/cc", 2.5, 1.5, 3.0),
            param("RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5),
            param("NPHI_MA", "Matrix neutron porosity", "v/v", -0.02, -0.15, 0.5),
            param("NPHI_SH", "Shale neutron porosity", "v/v", 0.35, 0.0, 0.8),
            param("NPHI_FL", "Fluid neutron porosity", "v/v", 1.0, 0.5, 1.2),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", true),
            log_out("VSH_DN", "VSH from density-neutron (unlimited)", "v/v"),
            log_out("VSH", "Limited volume of shale", "v/v"),
        ],
    }
}

fn vsh_dn(ctx: &ModuleContext) -> ModuleOutputs {
    let rho = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let mut vsh_dn_out = vec![f32::NAN; ctx.n];
    let mut vsh_out = vec![f32::NAN; ctx.n];

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
        let a = (rho_ma - rho_fl) * (nphi_fl - np);
        let b = (r - rho_fl) * (nphi_fl - nphi_ma);
        let c = (rho_ma - rho_fl) * (nphi_fl - nphi_sh);
        let d = (rho_sh - rho_fl) * (nphi_fl - nphi_ma);
        let v = (a - b) / (c - d);
        vsh_dn_out[i] = v as f32;
        vsh_out[i] = limit(v, 0.0, 1.0) as f32;
    }

    HashMap::from([("VSH_DN".to_string(), vsh_dn_out), ("VSH".to_string(), vsh_out)])
}

// ---------------------------------------------------------------------------
// PHI_DEN — Porosity from density log (Geolog phi_den.lls)
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
            param("RHO_MA", "Matrix density", "g/cc", 2.645, 2.0, 3.2),
            param("RHO_SH", "Shale density", "g/cc", 2.5, 1.5, 3.0),
            param("RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5),
            param("RHO_DSH", "Dry shale density", "g/cc", 2.65, 2.0, 3.2),
            param("RHO_W", "Formation water density", "g/cc", 1.0, 0.8, 1.3),
            opt("OPT_PHIEMAX", "PHIE limiting method", "SHALE_REDUCED", &["SHALE_REDUCED", "MAXIMUM"]),
            param("PHIE_MAX", "Maximum allowed PHIE", "v/v", 0.3, 0.05, 0.5),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
            log_out("PHIE_DEN", "PHIE from density (unlimited)", "v/v"),
            log_out("PHIT_DEN", "PHIT from density (unlimited)", "v/v"),
            log_out("PHIE", "Limited effective porosity", "v/v"),
            log_out("PHIT", "Limited total porosity", "v/v"),
        ],
    }
}

/// Shared PHIT_SH derivation from phi_den/phi_dn: shale total porosity from densities.
fn phit_sh_at(ctx: &ModuleContext, i: usize) -> f64 {
    let rho_dsh = ctx.p("RHO_DSH", i);
    let rho_sh = ctx.p("RHO_SH", i);
    let rho_w = ctx.p("RHO_W", i);
    (rho_dsh - rho_sh) / (rho_dsh - rho_w)
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

        if v >= 0.95 {
            phie_den[i] = 0.0;
            phie_lim_out[i] = 0.0;
            phit_den[i] = phit_sh as f32;
            phit_lim_out[i] = phit_sh as f32;
            continue;
        }

        let pe = (rho_ma - r) / (rho_ma - rho_fl) - v * (rho_ma - rho_sh) / (rho_ma - rho_fl);
        let pt = pe + v * phit_sh;
        let phie_lim = if shale_reduced { phie_max * (1.0 - v) } else { phie_max };
        let pe_l = limit(pe, 0.0, phie_lim);
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
// PHI_DN — Porosity from density-neutron (Geolog phi_dn.lls structure; analytic
// crossplot instead of proprietary service-company chart lookups)
// ---------------------------------------------------------------------------

fn phi_dn_spec() -> ModuleSpec {
    ModuleSpec {
        name: "phi_dn".into(),
        title: "Porosity from Density-Neutron".into(),
        category: "Porosity".into(),
        doc: "Shale-corrects RHOB and NPHI to 'shale reduced' values, then combines density \
              porosity and neutron porosity: AVERAGE = (PHID+PHIN)/2, GAS_RMS = sqrt((PHID²+PHIN²)/2) \
              for gas-bearing zones. (Geolog uses service-company chart lookups here; this is the \
              standard analytic equivalent.) PHIE = PHIX*(1-VSH); PHIT = PHIE + VSH*PHIT_SH."
            .into(),
        args: vec![
            opt("OPT_XPLOT", "Crossplot combination method", "AVERAGE", &["AVERAGE", "GAS_RMS"]),
            param("RHO_MA", "Matrix density", "g/cc", 2.645, 2.0, 3.2),
            param("RHO_SH", "Shale density", "g/cc", 2.5, 1.5, 3.0),
            param("RHO_FL", "Fluid density", "g/cc", 1.0, 0.5, 1.5),
            param("NPHI_SH", "Shale neutron porosity", "v/v", 0.35, 0.0, 0.8),
            param("RHO_DSH", "Dry shale density", "g/cc", 2.65, 2.0, 3.2),
            param("RHO_W", "Formation water density", "g/cc", 1.0, 0.8, 1.3),
            opt("OPT_PHIEMAX", "PHIE limiting method", "SHALE_REDUCED", &["SHALE_REDUCED", "MAXIMUM"]),
            param("PHIE_MAX", "Maximum allowed PHIE", "v/v", 0.3, 0.05, 0.5),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", true),
            log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
            log_out("PHIE_DN", "PHIE from density-neutron (unlimited)", "v/v"),
            log_out("PHIT_DN", "PHIT from density-neutron (unlimited)", "v/v"),
            log_out("PHIE", "Limited effective porosity", "v/v"),
            log_out("PHIT", "Limited total porosity", "v/v"),
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

        if v >= 0.95 {
            phie_dn[i] = 0.0;
            phie_lim_out[i] = 0.0;
            phit_dn[i] = phit_sh as f32;
            phit_lim_out[i] = phit_sh as f32;
            continue;
        }

        // Shale-reduce the input logs (same limits as the Geolog source, in g/cc).
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
        let pe_l = limit(pe, 0.0, phie_lim);
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
              RHG (Raymer-Hunt-Gardner): PHIT = 0.625*(DT - DT_MA)/DT."
            .into(),
        args: vec![
            opt("OPT_SON", "Sonic porosity method", "WYLLIE", &["WYLLIE", "RHG"]),
            param("DT_MA", "Matrix transit time", "us/ft", 55.5, 40.0, 70.0),
            param("DT_FL", "Fluid transit time", "us/ft", 189.0, 150.0, 220.0),
            param("DT_SH", "Shale transit time", "us/ft", 90.0, 60.0, 150.0),
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

        let pt = if rhg {
            0.625 * (d - dt_ma) / d
        } else {
            (d - dt_ma) / (dt_fl - dt_ma)
        };
        phit_son[i] = limit(pt, 0.0, 1.0) as f32;
        if !is_missing(v) {
            let pe = pt - v * (dt_sh - dt_ma) / (dt_fl - dt_ma);
            phie_son[i] = limit(pe, 0.0, 1.0) as f32;
        }
    }

    HashMap::from([("PHIT_SON".to_string(), phit_son), ("PHIE_SON".to_string(), phie_son)])
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
            param("TSURF", "Surface temperature", "degC", 26.7, 0.0, 50.0),
            param("TGRAD", "Temperature gradient", "degC/m", 0.03, 0.005, 0.1),
            param("BHT", "Bottom hole temperature", "degC", 100.0, 30.0, 250.0),
            param("TD_BHT", "Depth of BHT measurement", "m", 2000.0, 100.0, 10000.0),
            log_out("FTEMP", "Formation temperature", "degC"),
        ],
    }
}

fn ftemp_grad(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let bht_mode = ctx.o("OPT_FT") == "BHT";
    let mut ftemp = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = depth[i] as f64;
        if is_missing(d) {
            continue;
        }
        let tsurf = ctx.p("TSURF", i);
        let t = if bht_mode {
            let bht = ctx.p("BHT", i);
            let td = ctx.p("TD_BHT", i);
            tsurf + (bht - tsurf) * d / td
        } else {
            tsurf + ctx.p("TGRAD", i) * d
        };
        ftemp[i] = t as f32;
    }
    HashMap::from([("FTEMP".to_string(), ftemp)])
}

// ---------------------------------------------------------------------------
// BADHOLE — bad-hole / washout QC flag from density correction and caliper
// ---------------------------------------------------------------------------

fn badhole_spec() -> ModuleSpec {
    ModuleSpec {
        name: "badhole".into(),
        title: "Bad-Hole QC Flag".into(),
        category: "Prep".into(),
        doc: "BADHOLE = 1 where the borehole is enlarged or the density correction is large \
              enough to distrust the porosity logs: |DRHO| > DRHO_MAX, or (CALI - bit size) > \
              DCAL_MAX. Bit size comes from the BS curve where present, else BS_DEF. The flag is \
              0 in good hole and MISSING where no QC curve exists. Feed it to any module run as a \
              mask so flagged intervals go missing instead of polluting results."
            .into(),
        args: vec![
            param("DRHO_MAX", "Max acceptable density correction", "g/cc", 0.05, 0.0, 0.5),
            param("DCAL_MAX", "Max acceptable (caliper - bit size)", "in", 1.0, 0.0, 12.0),
            param("BS_DEF", "Bit size when BS curve is absent", "in", 8.5, 3.0, 30.0),
            log_in("DRHO", "Density correction log", "g/cc", "DRHO", false),
            log_in("CALI", "Caliper log", "in", "CALI", false),
            log_in("BS", "Bit size log", "in", "BS", false),
            log_out("BADHOLE", "Bad-hole flag (1 = bad, 0 = good)", ""),
        ],
    }
}

fn badhole(ctx: &ModuleContext) -> ModuleOutputs {
    let drho = ctx.log("DRHO");
    let cali = ctx.log("CALI");
    let bs = ctx.log("BS");
    let mut flag = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        let dr = drho[i] as f64;
        let cl = cali[i] as f64;
        let drho_max = ctx.p("DRHO_MAX", i);
        let dcal_max = ctx.p("DCAL_MAX", i);
        let bit = {
            let b = bs[i] as f64;
            if is_missing(b) { ctx.p("BS_DEF", i) } else { b }
        };

        let mut any = false;
        let mut bad = false;
        if !is_missing(dr) {
            any = true;
            if dr.abs() > drho_max {
                bad = true;
            }
        }
        if !is_missing(cl) && !is_missing(bit) {
            any = true;
            if cl - bit > dcal_max {
                bad = true;
            }
        }
        if any {
            flag[i] = if bad { 1.0 } else { 0.0 };
        }
    }

    HashMap::from([("BADHOLE".to_string(), flag)])
}

// ---------------------------------------------------------------------------
// Environmental corrections (Geolog PT03, pragmatic analytic set). These are
// linearized, coefficient-driven equivalents of the service-company chartbook
// corrections — the coefficients are parameters with chartbook-magnitude defaults,
// so they can be tuned per tool/field. Chart-lookup fidelity comes later (ROADMAP).
// Each writes a corrected copy (<LOG>_EC); inputs are never modified, and a missing
// QC input (e.g. no caliper) passes the log through uncorrected rather than blanking.
// ---------------------------------------------------------------------------

fn gr_hole_corr_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gr_hole_corr".into(),
        title: "GR Hole-Size Correction".into(),
        category: "Prep".into(),
        doc: "GR_EC = GR * (1 + K_GR*(CALI - BS)): linear borehole-enlargement correction — \
              gamma rays attenuated by the extra mud annulus are restored. Bit size from the \
              BS curve where present, else BS_DEF. No caliper → GR passes through uncorrected."
            .into(),
        args: vec![
            param("K_GR", "Correction per inch of enlargement", "1/in", 0.0075, 0.0, 0.05),
            param("BS_DEF", "Bit size when BS curve is absent", "in", 8.5, 3.0, 30.0),
            log_in("GR", "Gamma ray log", "gapi", "GR", true),
            log_in("CALI", "Caliper log", "in", "CALI", false),
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
              formation-temperature and formation-salinity terms at CNL chartbook magnitudes \
              (defaults). Requires FTEMP (run Formation Temperature first) for the temperature \
              term; without it only the salinity term applies."
            .into(),
        args: vec![
            param("K_TEMP", "Temperature coefficient", "v/v per degC", 0.0001, -0.01, 0.01),
            param("T_REF", "Chart reference temperature", "degC", 24.0, 0.0, 100.0),
            param("K_SAL", "Salinity coefficient per 100 kppm", "v/v", -0.002, -0.05, 0.05),
            param("SALW", "Formation water salinity", "ppm", 20000.0, 0.0, 300000.0),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", true),
            log_in("FTEMP", "Formation temperature", "degC", "FTEMP", false),
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
              holes the pad reads too much mud, so density is restored upward at chartbook \
              magnitude (default 0.004 g/cc per inch beyond 10\"). Within gauge, or with no \
              caliper, RHOB passes through unchanged. Use with the BADHOLE flag — beyond a \
              few inches of washout no correction is trustworthy."
            .into(),
        args: vec![
            param("K_RHO", "Correction per inch beyond reference", "g/cc/in", 0.004, 0.0, 0.05),
            param("HD_REF", "Hole diameter where correction starts", "in", 10.0, 4.0, 20.0),
            log_in("RHOB", "Density log", "g/cc", "RHOB", true),
            log_in("CALI", "Caliper log", "in", "CALI", false),
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
// Shared Rw resolution (Geolog sw_*.lls): constant, Arps-corrected measurement,
// or salinity conversion (Bateman-Konen / Kennedy).
// ---------------------------------------------------------------------------

fn rw_args() -> Vec<ArgSpec> {
    vec![
        opt("OPT_RW", "Formation water resistivity source", "CONSTANT", &["CONSTANT", "MEASURED", "SALINITY"]),
        param("RW", "Rw at formation temperature (CONSTANT)", "ohmm", 0.1, 0.001, 20.0),
        param("RWS", "Measured water sample resistivity", "ohmm", 0.1, 0.001, 20.0),
        param("RWT", "Temperature of RWS measurement", "degC", 24.0, 0.0, 150.0),
        param("SALW", "Formation water salinity", "ppm", 20000.0, 100.0, 300000.0),
        log_in("FTEMP", "Formation temperature (for MEASURED/SALINITY)", "degC", "FTEMP", false),
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
            // Kennedy above 39161 ppm, Bateman-Konen below (Geolog sw_arch.lls).
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
// SW_ARCH — Water saturation, Archie (Geolog sw_arch.lls)
// ---------------------------------------------------------------------------

fn sw_arch_spec() -> ModuleSpec {
    let mut args = vec![
        param("A", "Tortuosity constant", "", 1.0, 0.1, 5.0),
        param("M", "Cementation exponent", "", 2.0, 1.0, 4.0),
        param("N", "Saturation exponent", "", 2.0, 1.0, 4.0),
        param("SWT_IRR", "Irreducible total water saturation", "v/v", 0.0, 0.0, 0.6),
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
        // Coal / zero porosity: everything water (Geolog convention).
        if pt == 0.0 && pe == 0.0 {
            swt_arch[i] = 1.0;
            swt_out[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = 0.0;
            continue;
        }
        let rw = resolve_rw(ctx, &ftemp, i);
        if is_missing(r) || is_missing(rw) {
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
            // Low effective porosity clean-up (Geolog: PHIE < 0.005 → all water).
            if pe < 0.005 {
                swe_l = 1.0;
                swt_out[i] = 1.0;
            }
            swe_out[i] = swe_l as f32;
            vol_uwat[i] = (pe * swe_l) as f32;
        }
    }

    HashMap::from([
        ("SWT_ARCH".to_string(), swt_arch),
        ("SWT".to_string(), swt_out),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
    ])
}

// ---------------------------------------------------------------------------
// SW_INDO — Water saturation, Indonesia / Poupon-Leveaux (Geolog sw_indo.lls)
// ---------------------------------------------------------------------------

fn sw_indo_spec() -> ModuleSpec {
    let mut args = vec![
        opt("OPT_INDO", "Indonesia VSH exponent variant", "FULL", &["FULL", "SIMPLE", "TAR_SAND"]),
        param("A", "Tortuosity constant", "", 1.0, 0.1, 5.0),
        param("M", "Cementation exponent", "", 2.0, 1.0, 4.0),
        param("N", "Saturation exponent", "", 2.0, 1.0, 4.0),
        param("RT_SH", "Shale resistivity", "ohmm", 5.0, 0.1, 500.0),
        param("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.0, 0.0, 0.6),
    ];
    args.extend(rw_args());
    args.extend([
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
        log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
        log_out("SWE_INDO", "SWE from Indonesia (unlimited)", "v/v"),
        log_out("SWE", "Limited effective water saturation", "v/v"),
        log_out("VOL_UWAT", "Volume of water (unflushed)", "v/v"),
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
        if pe < 0.005 {
            swe_indo[i] = 1.0;
            swe_out[i] = 1.0;
            vol_uwat[i] = pe as f32;
            continue;
        }
        let rw = resolve_rw(ctx, &ftemp, i);
        if is_missing(r) || is_missing(vs) || is_missing(rw) {
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

    HashMap::from([
        ("SWE_INDO".to_string(), swe_indo),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
    ])
}

// ---------------------------------------------------------------------------
// SW_SIM — Water saturation, Simandoux (Geolog sw_sim.lls, Newton-Raphson solver)
// ---------------------------------------------------------------------------

fn sw_sim_spec() -> ModuleSpec {
    let mut args = vec![
        opt("OPT_SIM", "Simandoux variant", "MODIFIED", &["MODIFIED", "SCHLUMBERGER"]),
        param("A", "Tortuosity constant", "", 1.0, 0.1, 5.0),
        param("M", "Cementation exponent", "", 2.0, 1.0, 4.0),
        param("N", "Saturation exponent", "", 2.0, 1.0, 4.0),
        param("C", "VSH exponent (SCHLUMBERGER variant)", "", 1.0, 0.5, 2.0),
        param("RT_SH", "Shale resistivity", "ohmm", 5.0, 0.1, 500.0),
        param("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.0, 0.0, 0.6),
    ];
    args.extend(rw_args());
    args.extend([
        log_in("RT", "True formation resistivity", "ohmm", "RES_DEEP", true),
        log_in("PHIE", "Limited effective porosity", "v/v", "PHIE", true),
        log_in("VSH", "Limited volume of shale", "v/v", "VSH", true),
        log_out("SWE_SIM", "SWE from Simandoux (unlimited)", "v/v"),
        log_out("SWE", "Limited effective water saturation", "v/v"),
        log_out("VOL_UWAT", "Volume of water (unflushed)", "v/v"),
    ]);
    ModuleSpec {
        name: "sw_sim".into(),
        title: "SW — Simandoux".into(),
        category: "Saturation".into(),
        doc: "Solves g1*SW^N + g2*SW - 1/RT = 0 by Newton-Raphson (20 iterations, tol 1e-5). \
              MODIFIED: g1 = PHIE^M/(A*Rw), g2 = VSH/RT_SH. \
              SCHLUMBERGER: g1 = PHIE^M/(A*Rw*(1-VSH)), g2 = VSH^C/RT_SH."
            .into(),
        args,
    }
}

/// Newton-Raphson solve of g1*s^n + g2*s + g3 = 0, exactly as Geolog's CALC_SW subroutine.
fn calc_sw(g1: f64, g2: f64, g3: f64, n: f64) -> f64 {
    let mut sat = 0.5_f64;
    for _ in 0..20 {
        let fx = g1 * sat.powf(n) + g2 * sat + g3;
        let fxp = n * g1 * sat.powf(n - 1.0) + g2;
        let del = fx / fxp;
        sat = (sat - del).max(0.0);
        if del.abs() < 0.00001 {
            return sat;
        }
    }
    MISSING
}

fn sw_sim(ctx: &ModuleContext) -> ModuleOutputs {
    let rt = ctx.log("RT");
    let phie = ctx.log("PHIE");
    let vsh = ctx.log("VSH");
    let ftemp = ctx.log("FTEMP");
    let modified = ctx.o("OPT_SIM") != "SCHLUMBERGER";
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
        if is_missing(r) || is_missing(vs) || is_missing(rw) {
            continue;
        }
        let a = ctx.p("A", i);
        let m = ctx.p("M", i);
        let n_exp = ctx.p("N", i);
        let c = ctx.p("C", i);
        let rt_sh = ctx.p("RT_SH", i);
        let swe_irr = ctx.p("SWE_IRR", i);

        let ff = a / pe.powf(m);
        let (g1, g2) = if modified {
            (1.0 / (ff * rw), vs / rt_sh)
        } else {
            (1.0 / (ff * rw * (1.0 - vs)), vs.powf(c) / rt_sh)
        };
        let g3 = -1.0 / r;
        let sat = calc_sw(g1, g2, g3, n_exp);
        if is_missing(sat) {
            continue;
        }
        swe_sim[i] = sat as f32;
        let swe_l = limit(sat, swe_irr, 1.0);
        swe_out[i] = swe_l as f32;
        vol_uwat[i] = (pe * swe_l) as f32;
    }

    HashMap::from([
        ("SWE_SIM".to_string(), swe_sim),
        ("SWE".to_string(), swe_out),
        ("VOL_UWAT".to_string(), vol_uwat),
    ])
}

// ---------------------------------------------------------------------------
// PERM_WYLLIE_ROSE — Permeability, Wyllie-Rose family (Geolog perm_wyllie_rose.lls)
// ---------------------------------------------------------------------------

fn perm_wyllie_rose_spec() -> ModuleSpec {
    ModuleSpec {
        name: "perm_wyllie_rose".into(),
        title: "Permeability — Wyllie-Rose".into(),
        category: "Permeability".into(),
        doc: "PERM = (C * PHIE^D / SWE_IRR^E)^2, mD. Defaults per method from Geolog: \
              TIMUR C=100 D=2.25 E=1; MORRIS_BIGGS_OIL C=250 D=3 E=1; MORRIS_BIGGS_GAS C=79 D=3 E=1; \
              TIXIER C=250 D=3 E=1."
            .into(),
        args: vec![
            opt("OPT_WR", "Wyllie-Rose variant", "TIMUR", &["TIMUR", "MORRIS_BIGGS_OIL", "MORRIS_BIGGS_GAS", "TIXIER"]),
            param("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.15, 0.01, 0.8),
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
        if is_missing(pe) || is_missing(swirr) || swirr <= 0.0 {
            continue;
        }
        let k = (c * pe.powf(d) / swirr.powf(e)).powi(2);
        perm[i] = k as f32;
    }
    HashMap::from([("PERM_WR".to_string(), perm.clone()), ("PERM".to_string(), perm)])
}

// ---------------------------------------------------------------------------
// PERM_COATES — Permeability, Coates FFI (Geolog perm_coates.lls)
// ---------------------------------------------------------------------------

fn perm_coates_spec() -> ModuleSpec {
    ModuleSpec {
        name: "perm_coates".into(),
        title: "Permeability — Coates".into(),
        category: "Permeability".into(),
        doc: "PERM = (C * PHIE^2 * (1 - SWE_IRR)/SWE_IRR)^2, mD.".into(),
        args: vec![
            param("CONST_COATES", "Coates constant", "", 100.0, 1.0, 1000.0),
            param("SWE_IRR", "Irreducible effective water saturation", "v/v", 0.15, 0.01, 0.8),
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
            param("PT_A", "Slope", "", 20.0, 1.0, 100.0),
            param("PT_B", "Intercept", "", -3.0, -10.0, 5.0),
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
        perm[i] = 10.0_f64.powf(a * pe + b) as f32;
    }
    HashMap::from([("PERM_XFM".to_string(), perm.clone()), ("PERM".to_string(), perm)])
}

// ---------------------------------------------------------------------------
// THIN_BED_TS — Thomas-Stieber laminated sand-shale decomposition
// (Geolog PT09_ThinBeds equivalent; Thomas & Stieber, 1975)
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
            param("PHI_SD_MAX", "Clean sand porosity (endpoint)", "v/v", 0.30, 0.05, 0.45),
            param("PHI_SH", "Shale porosity (endpoint)", "v/v", 0.15, 0.0, 0.45),
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
// DEPTH_SHIFT — block depth shift of one curve (Geolog SpliceLogs/shift equivalent)
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
            param("SHIFT", "Depth shift (+ = deeper)", "m", 0.0, -1000.0, 1000.0),
            log_in("CURVE", "Curve to shift", "", "GR", true),
            log_out("CURVE_DS", "Depth-shifted copy (named <input>_DS)", ""),
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
    let src = ctx.o("__IN_CURVE");
    let out_name = if src.is_empty() { "SHIFTED".to_string() } else { format!("{src}_DS") };

    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = depth[i] as f64;
        let shift = ctx.p("SHIFT", i);
        if is_missing(d) || is_missing(shift) {
            continue;
        }
        out[i] = interp_at(&depth, &vals, d - shift) as f32;
    }
    HashMap::from([(out_name, out)])
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
            param("SPLICE_DEPTH", "Depth where BOT_CURVE takes over", "m", 1000.0, 0.0, 20000.0),
            log_in("TOP_CURVE", "Curve used above the splice depth", "", "GR", true),
            log_in("BOT_CURVE", "Curve used below the splice depth", "", "GR", true),
            log_out("SPLICED", "Spliced curve (named <top input>_SPL)", ""),
        ],
    }
}

fn splice(ctx: &ModuleContext) -> ModuleOutputs {
    let depth = ctx.log("DEPTH");
    let top = ctx.log("TOP_CURVE");
    let bot = ctx.log("BOT_CURVE");
    let src = ctx.o("__IN_TOP_CURVE");
    let out_name = if src.is_empty() { "SPLICED".to_string() } else { format!("{src}_SPL") };

    let mut out = vec![f32::NAN; ctx.n];
    for i in 0..ctx.n {
        let d = depth[i] as f64;
        if is_missing(d) {
            continue;
        }
        out[i] = if d < ctx.p("SPLICE_DEPTH", i) { top[i] } else { bot[i] };
    }
    HashMap::from([(out_name, out)])
}

// ---------------------------------------------------------------------------
// GR_NORMALIZE — two-point percentile gamma-ray normalization (BLSO/Rokan standard)
// ---------------------------------------------------------------------------

/// Linear-interpolated percentile (0–100) of the finite values in `vals`.
fn percentile_of(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return MISSING;
    }
    if n == 1 {
        return sorted[0];
    }
    let rank = (p / 100.0).clamp(0.0, 1.0) * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo as f64)
}

fn gr_normalize_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gr_normalize".into(),
        title: "GR Normalization (Two-Point Percentile)".into(),
        category: "Prep".into(),
        doc: "GRN = (GR − Plow_well)·(Phigh_ref − Plow_ref)/(Phigh_well − Plow_well) + Plow_ref. \
              The well percentiles are computed from this run's GR samples (mask the run to the \
              reference interval to reproduce the field standard), the reference percentiles are \
              parameters. Defaults are the Rokan regional calibration from 562 wells: P3 = 53.68, \
              P97 = 133.93 gAPI. QC across wells with a GRN histogram overlay — the P3/P97 of \
              every normalized well should coincide."
            .into(),
        args: vec![
            param("P_LOW", "Low percentile", "%", 3.0, 0.0, 50.0),
            param("P_HIGH", "High percentile", "%", 97.0, 50.0, 100.0),
            param("GR_LOW_REF", "Reference GR at low percentile", "gapi", 53.68, 0.0, 1000.0),
            param("GR_HIGH_REF", "Reference GR at high percentile", "gapi", 133.93, 0.0, 1000.0),
            log_in("GR", "Gamma ray log", "gapi", "GR", true),
            log_out("GRN", "Normalized gamma ray", "gapi"),
        ],
    }
}

fn gr_normalize(ctx: &ModuleContext) -> ModuleOutputs {
    let gr = ctx.log("GR");
    let mut out = vec![f32::NAN; ctx.n];

    let mut valid: Vec<f64> = gr.iter().map(|v| *v as f64).filter(|v| !is_missing(*v)).collect();
    valid.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if valid.len() < 2 {
        return HashMap::from([("GRN".to_string(), out)]);
    }
    // Percentile levels are global per run: read them at the first valid sample.
    let i0 = (0..ctx.n).find(|&i| !is_missing(gr[i] as f64)).unwrap_or(0);
    let p_lo_well = percentile_of(&valid, ctx.p("P_LOW", i0));
    let p_hi_well = percentile_of(&valid, ctx.p("P_HIGH", i0));
    if is_missing(p_lo_well) || is_missing(p_hi_well) || p_hi_well - p_lo_well <= 1e-9 {
        return HashMap::from([("GRN".to_string(), out)]);
    }

    for i in 0..ctx.n {
        let g = gr[i] as f64;
        let lo_ref = ctx.p("GR_LOW_REF", i);
        let hi_ref = ctx.p("GR_HIGH_REF", i);
        if is_missing(g) || is_missing(lo_ref) || is_missing(hi_ref) {
            continue;
        }
        out[i] = ((g - p_lo_well) * (hi_ref - lo_ref) / (p_hi_well - p_lo_well) + lo_ref) as f32;
    }
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
            param("K", "Number of neighbours", "", 5.0, 1.0, 50.0),
            log_in("TARGET", "Curve to predict", "", "RHOB", true),
            log_in("P1", "Predictor 1", "", "GR", true),
            log_in("P2", "Predictor 2 (optional)", "", "NPHI", false),
            log_in("P3", "Predictor 3 (optional)", "", "DT", false),
            log_out("SYN", "Synthetic curve (named <target>_SYN)", ""),
        ],
    }
}

fn log_predict(ctx: &ModuleContext) -> ModuleOutputs {
    let target = ctx.log("TARGET");
    let combine = ctx.o("OPT_COMBINE").to_string();
    let src = ctx.o("__IN_TARGET");
    let out_name = if src.is_empty() { "SYN".to_string() } else { format!("{src}_SYN") };
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
        if *s < 1e-9 {
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
                continue; // leave-one-out
            }
            let d2: f64 = (0..dims).map(|d| (xs[d] - tx[d]).powi(2)).sum();
            if best.len() < k {
                best.push((d2, *tv));
                best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            } else if d2 < best[k - 1].0 {
                best[k - 1] = (d2, *tv);
                best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
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
        }
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
        let s = &out["GR_DS"];
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
        let f = &depth_shift(&ctx_frac)["GR_DS"];
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
        let s = &out["RES_RUN1_SPL"];
        assert_eq!(s[2], 1.0, "above the splice depth the top curve wins");
        assert_eq!(s[3], 2.0, "at/below the splice depth the bottom curve wins");
    }

    #[test]
    fn badhole_flags_washout_and_drho() {
        let ctx = ctx_with(
            4,
            &[
                ("DRHO", vec![0.01, 0.20, 0.01, f32::NAN]),
                ("CALI", vec![8.6, 8.6, 14.0, f32::NAN]),
            ],
            &[("DRHO_MAX", 0.05), ("DCAL_MAX", 1.0), ("BS_DEF", 8.5)],
            &[],
        );
        let out = badhole(&ctx);
        let f = &out["BADHOLE"];
        assert_eq!(f[0], 0.0, "good hole");
        assert_eq!(f[1], 1.0, "big DRHO");
        assert_eq!(f[2], 1.0, "washout");
        assert!(f[3].is_nan(), "no QC curves at all -> missing");
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
        let syn = &out["RHOB_SYN"];
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
        let syn = &out["RHOB_SYN"];
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
}
