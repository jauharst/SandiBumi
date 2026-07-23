//! Generalized Multimin — user-defined multi-mineral / fluid optimizer, modeled on
//! the reference multi-mineral solver and IP's Mineral Solver (spec extracted from both installs, see
//! docs/multimin_ref_spec.md and docs/multimin_ip_spec.md).
//!
//! Formulation (standard convention):
//! - One volume vector per depth frame: minerals + clays (common to both zones) plus
//!   flushed-zone (X / Sxo) and unflushed-zone (U / Sw) fluid sets.
//! - Every tool responds to X-zone fluids except the deep conductivity CT, which sees
//!   the U zone; CXO (flushed conductivity) sees the X zone.
//! - Resistivity enters as CONDUCTIVITY via the DUAL WATER LINEAR transform: with
//!   w = 0.75·m + 0.25·n the response row  Ct^(1/w) = Σ v_i · C_i^(1/w)  is linear in
//!   the volumes (C_i = fluid conductivity endpoints; minerals/hydrocarbons are 0).
//!   The measured resistivity curve is converted (C = 1/R) and transformed before entering
//!   the system. Row uncertainty: U_ct = 0.03·Cw^(1/w), U_cxo = 0.03·Cmf^(1/w).
//! - Hard constraints: Σ(minerals + clays + U-zone fluids) = 1 (UNITY, X fluids excluded)
//!   and per-component box bounds 0 ≤ v ≤ max_vol (fluids default 0.5).
//! - Soft "Tool" constraints at σ = 0.01 (treated as pseudo-measurements):
//!   POROSITY (Σ X fluids = Σ U fluids) and BNDWAT (bound water tied to clay volumes via
//!   k = 96·CEC·ρ_clay / (T°C + 298) · α, the Dual-Water clay-bound-water multiplier).
//!   WATER MUD (Sxo ≥ Sw for water-based mud) is enforced by a re-solve when violated.
//!
//! The solver is a bounded, equality-constrained active-set least squares (KKT system with
//! a unity Lagrange multiplier; components can be fixed at 0 or at their upper bound).
//! RECON is the INCOHERENCE — the σ-weighted RMS of (reconstructed − measured) over the live tool
//! rows (Quanti.Elan "incoherence" function, Eq 79; see docs/multimin_geolog_spec.md) — so a high
//! value flags a model that cannot reproduce the logs. With `recon_qc` the reconstruction is
//! decomposed per tool: `<prefix>_<KEY>_REC` (measurement rebuilt from the volumes, display units)
//! and `<prefix>_<KEY>_DIF` (that tool's σ-unit residual, whose RMS over tools is RECON), so the
//! user can see WHICH log the model fails to honour. The reconstruction only discriminates when the
//! system is over-determined — the reported `dof` says whether that holds.

use crate::equations::{fetch_curve_frame, write_computed_curves_versioned};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Request / result types
// ---------------------------------------------------------------------------

/// One mineral, clay, or fluid component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    /// "mineral" | "clay" | "fluid"
    #[serde(default)]
    pub kind: String,
    /// Fluids only: "X" (flushed / Sxo), "U" (unflushed / Sw), or "" (seen by both zones).
    #[serde(default)]
    pub zone: String,
    /// Fluids only: "water" | "bound_water" | "oil" | "gas".
    #[serde(default)]
    pub fluid_type: String,
    /// Tool-key → endpoint response, in display units (g/cc, v/v, us/ft, API, ...).
    pub endpoints: HashMap<String, f64>,
    /// Cation exchange capacity, meq/g (clays; drives the bound-water constraint).
    #[serde(default)]
    pub cec: f64,
    /// Upper volume bound (default: 1.0 minerals, 0.5 fluids).
    #[serde(default = "default_one")]
    pub max_vol: f64,
}

fn default_one() -> f64 {
    1.0
}

/// A tool (input log) in the inversion. Keys "CT"/"CXO" are conductivity rows: `curve`
/// is a RESISTIVITY curve (ohmm) converted to conductivity (mho/m) per sample; their
/// endpoints come from the fluid properties, not from the endpoints table. `sigma <= 0`
/// on CT/CXO means "auto" (0.03·C^(1/w)).
#[derive(Debug, Clone, Deserialize)]
pub struct ToolSpec {
    pub key: String,
    pub curve: String,
    pub sigma: f64,
}

/// Saturation model for SandiMin's deep/flushed conductivity tools (Jauhar's Sw-equation request).
///
/// `linear_dw` (default) is the current in-inversion linearised dual-water `Ct^(1/w) = Σ v·C^(1/w)`
/// with a single exponent `w = 0.75m + 0.25n`; it is linear in the volume vector and leaves every
/// already-reviewed number untouched. The shaly-sand forms are **post-solve**: the mineral inversion
/// runs on the lithology tools with NO conductivity row, then Sw is computed from the closed form
/// using the solved effective porosity + shale volume and the deep resistivity, and the U-zone
/// water/HC volumes are redistributed to honour it (so PHIE is unchanged and SWE = the model Sw).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SwModel {
    /// Linearised dual-water, in-inversion. Default — nothing moves.
    #[default]
    LinearDw,
    /// Clavier-Coates-Dumanoir dual-water, exact form honouring m and n separately (Newton form
    /// solved by bisection). Post-solve; the bound-water saturation comes from the solved v_bw.
    DualWaterNonlinear,
    /// Archie (1942) clean-sand, total-porosity form. Post-solve; ignores the clay conductivity term.
    Archie,
    /// Poupon-Leveaux "Indonesia" (1971), effective-porosity form. Non-linear in Sw, post-solve.
    Indonesia,
    /// Modified Simandoux (Bardon-Pied), effective-porosity form. Non-linear in Sw, post-solve.
    Simandoux,
}

impl SwModel {
    /// Every model except the default linearised dual-water replaces the in-inversion conductivity row
    /// with a post-solve Sw computed from the solved volumes.
    fn is_post_solve(self) -> bool {
        !matches!(self, SwModel::LinearDw)
    }
}

/// Poupon-Leveaux ("Indonesia", 1971) water saturation, effective-porosity form, solved for Sw∈[0,1]:
///   1/√Rt = [ Vsh^(1 − Vsh/2)/√Rsh + √(φe^m / (a·Rw)) ] · Sw^(n/2)
/// Rw and Rsh are at formation temperature. Returns NaN on non-physical inputs.
pub fn sw_indonesia(rt: f64, phie: f64, vsh: f64, rw: f64, rsh: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(rt > 0.0) || !(phie > 0.0) || !(rw > 0.0) || !(n > 0.0) {
        return f64::NAN;
    }
    let vsh = vsh.clamp(0.0, 1.0);
    let a = a.max(1e-9);
    let term_sh = if rsh > 0.0 { vsh.powf(1.0 - vsh / 2.0) / rsh.sqrt() } else { 0.0 };
    let term_sand = (phie.powf(m) / (a * rw)).sqrt();
    let denom = term_sh + term_sand;
    if !(denom > 0.0) {
        return f64::NAN;
    }
    let sw_half = (1.0 / rt).sqrt() / denom; // = Sw^(n/2)
    sw_half.powf(2.0 / n).clamp(0.0, 1.0)
}

/// Modified Simandoux (Bardon-Pied) water saturation, effective-porosity form, solved for Sw∈[0,1]:
///   1/Rt = φe^m·Sw^n / (a·Rw·(1 − Vsh)) + Vsh·Sw / Rsh
/// Closed-form quadratic when n == 2; monotone bisection otherwise. Rw/Rsh at formation temperature.
pub fn sw_simandoux(rt: f64, phie: f64, vsh: f64, rw: f64, rsh: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(rt > 0.0) || !(phie > 0.0) || !(rw > 0.0) || !(n > 0.0) {
        return f64::NAN;
    }
    let vsh = vsh.clamp(0.0, 0.999);
    let a = a.max(1e-9);
    let ct = 1.0 / rt;
    let coef_sand = phie.powf(m) / (a * rw * (1.0 - vsh)); // coefficient of Sw^n
    let coef_sh = if rsh > 0.0 { vsh / rsh } else { 0.0 }; // coefficient of Sw^1
    if !(coef_sand > 0.0) {
        // Degenerate: no sand term — the shale term alone gives Sw (or NaN if no shale either).
        return if coef_sh > 0.0 { (ct / coef_sh).clamp(0.0, 1.0) } else { f64::NAN };
    }
    if (n - 2.0).abs() < 1e-9 {
        // coef_sand·Sw² + coef_sh·Sw − ct = 0
        let disc = coef_sh * coef_sh + 4.0 * coef_sand * ct;
        if disc < 0.0 {
            return f64::NAN;
        }
        return ((-coef_sh + disc.sqrt()) / (2.0 * coef_sand)).clamp(0.0, 1.0);
    }
    // General n: f(Sw) = coef_sand·Sw^n + coef_sh·Sw − ct is increasing on [0,1]; f(0) = −ct < 0.
    let f = |sw: f64| coef_sand * sw.powf(n) + coef_sh * sw - ct;
    if f(1.0) <= 0.0 {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (0.5 * (lo + hi)).clamp(0.0, 1.0)
}

/// Clavier-Coates-Dumanoir dual-water TOTAL water saturation (SPEJ 1984), exact form honouring m and
/// n separately (unlike the linearised in-inversion row, which folds them into w = 0.75m+0.25n):
///   Ct = (φt^m · Swt^n / a) · [ Cw + (Cwb − Cw)·Swb/Swt ]
/// rearranged to the increasing-in-Swt root equation (Geolog sw_dual CALC_SW):
///   Cw·Swt^n + Swb·(Cwb − Cw)·Swt^(n−1) − a·Ct/φt^m = 0,   Ct = 1/Rt.
/// `swb` is the bound-water saturation (v_bw/φt) taken from the solved bound-water volume — so no Qv
/// is needed. Conductivities are mho/m at formation temperature. Returns SWT∈[0,1] (bisection; the
/// n==2 case closes to a quadratic). NaN on non-physical inputs.
pub fn sw_dual_nonlinear(rt: f64, phit: f64, swb: f64, cw: f64, cwb: f64, m: f64, n: f64, a: f64) -> f64 {
    // n < 1 is non-physical (saturation exponents are ≥ 1) AND breaks this solver: the Swt^(n−1) term
    // diverges at Swt→0, so g(0) blows up and the bisection/short-circuit logic (which assumes g rises
    // from g(0)=−rhs<0) collapses SWT to 0 regardless of Rt. Reject it — the post-solve caller's
    // is_finite() check then leaves the linear inversion's split untouched rather than silently zeroing.
    if !(rt > 0.0) || !(phit > 0.0) || !(cw > 0.0) || !(n >= 1.0) {
        return f64::NAN;
    }
    let swb = swb.clamp(0.0, 1.0);
    let a = a.max(1e-9);
    let ct = 1.0 / rt;
    let rhs = a * ct / phit.powf(m); // the constant term a·Ct/φt^m (> 0)
    let lin = swb * (cwb - cw); // coefficient of Swt^(n−1)
    if (n - 2.0).abs() < 1e-9 {
        // cw·Swt² + lin·Swt − rhs = 0. disc = lin² + 4·cw·rhs is always ≥ 0 (cw>0, rhs>0), so the
        // positive root exists; cw>0 makes it the physical branch.
        let disc = lin * lin + 4.0 * cw * rhs;
        return ((-lin + disc.sqrt()) / (2.0 * cw)).clamp(0.0, 1.0);
    }
    // General n: g(Swt) = cw·Swt^n + lin·Swt^(n−1) − rhs. g(0)=−rhs<0; if g(1)≤0 the rock is at/above
    // Swt=1. Between, g is continuous — bisect (cw>0 keeps the high-Swt branch increasing).
    let g = |swt: f64| cw * swt.powf(n) + lin * swt.powf(n - 1.0) - rhs;
    if g(1.0) <= 0.0 {
        return 1.0;
    }
    if g(0.0) > 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if g(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (0.5 * (lo + hi)).clamp(0.0, 1.0)
}

/// Archie (1942) clean-sand TOTAL water saturation — no shale term:
///   Swt = ( a·Rw / (φt^m · Rt) )^(1/n)
/// The exactly-invertible base case (so there is no separate "linear/non-linear" Archie). Rw at
/// formation temperature. Returns SWT∈[0,1]; NaN on non-physical inputs.
pub fn sw_archie(rt: f64, phit: f64, rw: f64, m: f64, n: f64, a: f64) -> f64 {
    if !(rt > 0.0) || !(phit > 0.0) || !(rw > 0.0) || !(n > 0.0) {
        return f64::NAN;
    }
    let a = a.max(1e-9);
    ((a * rw) / (phit.powf(m) * rt)).powf(1.0 / n).clamp(0.0, 1.0)
}

/// Fluid / saturation parameters (needed when CT or CXO participates).
#[derive(Debug, Clone, Deserialize)]
pub struct FluidProps {
    /// Formation water resistivity sample (ohmm) + its temperature (°F).
    pub rw: f64,
    pub rw_temp_f: f64,
    /// Mud filtrate resistivity sample (ohmm) + its temperature (°F).
    pub rmf: f64,
    pub rmf_temp_f: f64,
    /// Formation temperature (°F) at the interval of interest.
    pub ftemp_f: f64,
    /// Archie/dual-water exponents; w = 0.75·m + 0.25·n.
    pub m: f64,
    pub n: f64,
    /// "WATER" | "OIL"
    #[serde(default = "default_mud")]
    pub mud_type: String,
    /// Shale resistivity Rsh (ohmm) at formation temperature — the 100%-shale Rt used by the
    /// shaly-sand Sw models (Indonesia/Simandoux). Ignored by the dual-water models. Default 4.0.
    #[serde(default = "default_rsh")]
    pub rsh: f64,
    /// Archie tortuosity factor a (Indonesia/Simandoux). The dual-water models use a = 1. Default 1.0.
    #[serde(default = "default_archie_a")]
    pub archie_a: f64,
}

fn default_mud() -> String {
    "WATER".into()
}
fn default_rsh() -> f64 {
    4.0
}
fn default_archie_a() -> f64 {
    1.0
}

/// Derived fluid quantities (also exposed to the dialog via `multimin_fluid_calc`).
#[derive(Debug, Clone, Serialize)]
pub struct FluidCalc {
    pub w: f64,
    /// Formation-water / mud-filtrate conductivity at formation temperature (mho/m).
    pub cw: f64,
    pub cmf: f64,
    /// Clay-bound-water conductivity (mho/m) and its per-zone α-reduced values.
    pub cbw: f64,
    pub cbw_x: f64,
    pub cbw_u: f64,
    pub alpha_x: f64,
    pub alpha_u: f64,
    pub salinity_w_ppm: f64,
    pub salinity_mf_ppm: f64,
    /// Auto uncertainties for the transformed CT/CXO rows.
    pub u_ct: f64,
    pub u_cxo: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultiminRequest {
    pub components: Vec<Component>,
    pub tools: Vec<ToolSpec>,
    pub apply_well_ids: Vec<String>,
    #[serde(default = "default_prefix")]
    pub output_prefix: String,
    #[serde(default = "default_true")]
    pub unity: bool,
    /// Required when CT or CXO is among the tools.
    #[serde(default)]
    pub fluid: Option<FluidProps>,
    /// Emit per-tool reconstruction-QC curves: for each active tool a `<prefix>_<KEY>_REC`
    /// (measurement rebuilt from the solved volumes, in the tool's display units) and
    /// `<prefix>_<KEY>_DIF` (the σ-unit residual = that tool's term of RECON). Off by default
    /// to keep the curve set lean.
    #[serde(default)]
    pub recon_qc: bool,
    /// Saturation model for the conductivity tools. Default `linear_dw` = the current in-inversion
    /// linearised dual-water (nothing moves). `indonesia`/`simandoux` are post-solve shaly-sand forms.
    #[serde(default)]
    pub sw_model: SwModel,
}

fn default_prefix() -> String {
    "MM".into()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct MultiminWellResult {
    pub well_id: String,
    pub rows_solved: usize,
    pub mean_recon: f32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MultiminResult {
    pub outputs: Vec<String>,
    pub wells: Vec<MultiminWellResult>,
    /// Model degrees of freedom = (tools + soft constraints + unity) − components. 0 = exactly
    /// determined (residuals are forced to ~0 and can't validate the model); >0 = over-determined,
    /// so RECON/incoherence is a real fit-quality signal.
    pub dof: i64,
    /// Set when `dof == 0` — a heads-up that the reconstruction can't discriminate the model.
    pub dof_note: Option<String>,
    pub error: Option<String>,
}

fn fail(msg: &str) -> MultiminResult {
    MultiminResult { outputs: vec![], wells: vec![], dof: 0, dof_note: None, error: Some(msg.to_string()) }
}

/// Curve-safe token for a component name: uppercase, non-alphanumeric → '_'.
fn curve_token(name: &str) -> String {
    let t: String = name
        .trim()
        .to_uppercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    t.trim_matches('_').to_string()
}

// ---------------------------------------------------------------------------
// Fluid property calculations (reference fluid formulas / IP)
// ---------------------------------------------------------------------------

/// Arps temperature conversion of a resistivity (°F form). Shared with the
/// precalc Prep module in modules.rs.
pub(crate) fn arps_f(r: f64, from_f: f64, to_f: f64) -> f64 {
    r * (from_f + 6.77) / (to_f + 6.77)
}

/// Equivalent NaCl salinity (ppm) from water resistivity at 75 °F (Bateman-Konen inverse).
fn salinity_ppm(r_sample: f64, t_sample_f: f64) -> f64 {
    let r75 = arps_f(r_sample, t_sample_f, 75.0);
    if r75 <= 0.0124 {
        return 400_000.0; // saturated brine guard
    }
    10f64.powf((3.562 - (r75 - 0.0123).log10()) / 0.955)
}

/// Dual-water diffuse-layer expansion factor: α = sqrt(20455 / S) below 20,455 ppm NaCl.
fn alpha_expansion(salinity: f64) -> f64 {
    if salinity > 0.0 && salinity < 20_455.0 {
        (20_455.0 / salinity).sqrt().min(5.0)
    } else {
        1.0
    }
}

pub fn fluid_calc(p: &FluidProps) -> FluidCalc {
    let w = 0.75 * p.m + 0.25 * p.n;
    let w = if w.is_finite() && w > 0.5 { w } else { 2.0 };
    let cw = 1.0 / arps_f(p.rw, p.rw_temp_f, p.ftemp_f).max(1e-4);
    let cmf = 1.0 / arps_f(p.rmf, p.rmf_temp_f, p.ftemp_f).max(1e-4);
    let t_c = (p.ftemp_f - 32.0) * 5.0 / 9.0;
    let cbw = 0.0007 * (t_c + 8.5) * (t_c + 298.0);
    let sal_w = salinity_ppm(p.rw, p.rw_temp_f);
    let sal_mf = salinity_ppm(p.rmf, p.rmf_temp_f);
    let alpha_u = alpha_expansion(sal_w);
    // Oil mud: no filtrate invasion of water — X zone driven by formation water too.
    let alpha_x = if p.mud_type.eq_ignore_ascii_case("OIL") { alpha_u } else { alpha_expansion(sal_mf) };
    FluidCalc {
        w,
        cw,
        cmf,
        cbw,
        cbw_x: cbw / alpha_x,
        cbw_u: cbw / alpha_u,
        alpha_x,
        alpha_u,
        salinity_w_ppm: sal_w,
        salinity_mf_ppm: sal_mf,
        u_ct: 0.03 * cw.powf(1.0 / w),
        u_cxo: 0.03 * cmf.powf(1.0 / w),
    }
}

/// Clay-bound-water multiplier k so that v_bw = k · v_dryclay (reference spec 5.03):
/// k = α · 96 · CEC[meq/g] · ρ_clay[g/cc] / (T°C + 298).
fn bndwat_multiplier(cec: f64, rho_gcc: f64, t_c: f64, alpha: f64) -> f64 {
    alpha * 96.0 * cec * rho_gcc / (t_c + 298.0)
}

// ---------------------------------------------------------------------------
// Wet-clay → dry-clay endpoint conversion (KKT ONWJ Multimin Parameters.xlsx)
// ---------------------------------------------------------------------------

/// Wet-clay log readings picked in a shale interval, plus the assumed dry-clay
/// density (2.70 marine / 2.78 deltaic in the KKT ONWJ study).
#[derive(Debug, Clone, Deserialize)]
pub struct WetClayInput {
    pub rhob_wet: f64,
    pub nphi_wet: f64,
    pub gr_wet: f64,
    #[serde(default)]
    pub dt_wet: Option<f64>,
    pub rho_dry: f64,
    /// Current fluid properties — the CEC equivalent uses their T and α_u so the
    /// solver's BNDWAT constraint reproduces the φ_clay bookkeeping exactly.
    #[serde(default)]
    pub fluid: Option<FluidProps>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryClayCalc {
    /// Clay-bound porosity — the water fraction of the WET clay volume.
    pub phi_clay: f64,
    pub rhob_dry: f64,
    pub nphi_dry: f64,
    pub gr_dry: f64,
    pub dt_dry: Option<f64>,
    /// Bound-water tie for the dry-clay framework: v_bw = cbw_ratio · v_dryclay
    /// (CBW = φ_clay · V_wetclay and V_wetclay = V_dryclay / (1 − φ_clay)).
    pub cbw_ratio: f64,
    /// CEC (meq/g) that makes the solver's Dual-Water BNDWAT constraint enforce
    /// exactly cbw_ratio at the given fluid conditions — set it on the clay
    /// component together with the dry endpoints.
    pub cec_equiv: f64,
}

/// The xlsx conversion, verbatim (water at 1.00 g/cc and 189 µs/ft):
///   φ_clay   = (ρ_dry − ρ_wet) / (ρ_dry − 1.0)
///   NPHI_dry = (NPHI_wet − φ_clay) / (1 − φ_clay)
///   GR_dry   =  GR_wet / (1 − φ_clay)
///   DT_dry   = (DT_wet − 189·φ_clay) / (1 − φ_clay)
/// Deck slide 59 confirms the bookkeeping: bound water is an explicit solved
/// fluid volume (SWB = V_bw / PHIT), which is what cbw_ratio/cec_equiv feed.
pub fn dry_clay_calc(inp: &WetClayInput) -> Result<DryClayCalc, String> {
    const RHO_W: f64 = 1.0;
    const DT_W: f64 = 189.0;
    if !(inp.rhob_wet > RHO_W) {
        return Err("wet-clay RHOB must exceed 1.0 g/cc (the water density)".into());
    }
    if !(inp.rho_dry > inp.rhob_wet) {
        return Err("dry-clay density must exceed the wet-clay RHOB reading".into());
    }
    if !(inp.nphi_wet > 0.0 && inp.nphi_wet <= 1.0) {
        return Err("wet-clay NPHI must be a fraction in (0, 1] v/v — not percent".into());
    }
    if !(inp.gr_wet > 0.0) {
        return Err("wet-clay GR must be positive".into());
    }
    let phi = (inp.rho_dry - inp.rhob_wet) / (inp.rho_dry - RHO_W);
    let dry = 1.0 - phi;
    if let Some(d) = inp.dt_wet {
        if !(d > DT_W * phi) {
            return Err(format!(
                "wet-clay DT must exceed the water term 189·φ_clay = {:.1} µs/ft",
                DT_W * phi
            ));
        }
    }
    let cbw_ratio = phi / dry;
    let (t_c, alpha) = match &inp.fluid {
        Some(p) => ((p.ftemp_f - 32.0) * 5.0 / 9.0, fluid_calc(p).alpha_u),
        None => (25.0, 1.0),
    };
    // Invert bndwat_multiplier (the clay's RHOB endpoint becomes ρ_dry on apply).
    let cec_equiv = cbw_ratio * (t_c + 298.0) / (alpha * 96.0 * inp.rho_dry);
    Ok(DryClayCalc {
        phi_clay: phi,
        rhob_dry: inp.rho_dry,
        nphi_dry: (inp.nphi_wet - phi) / dry,
        gr_dry: inp.gr_wet / dry,
        dt_dry: inp.dt_wet.map(|d| (d - DT_W * phi) / dry),
        cbw_ratio,
        cec_equiv,
    })
}

// ---------------------------------------------------------------------------
// Fluid-property autofill from the precalc module's output curves
// ---------------------------------------------------------------------------

/// Zone-averaged fluid entries read from precalc outputs (FTEMP_F in °F and RMF
/// in ohmm at formation temperature). `None` when the curve has no finite sample
/// in the interval — precalc has not been run, or the zone is empty.
#[derive(Debug, Clone, Serialize)]
pub struct PrecalcFluid {
    pub ftemp_f: Option<f64>,
    pub rmf: Option<f64>,
    pub n_ftemp: usize,
    pub n_rmf: usize,
}

pub fn fluid_from_precalc(
    db: &Mutex<Connection>,
    well_id: &str,
    top: Option<f64>,
    bottom: Option<f64>,
) -> Result<PrecalcFluid, String> {
    let conn = db.lock().unwrap();
    let names = vec!["FTEMP_F".to_string(), "RMF".to_string()];
    let (depth, cols) = fetch_curve_frame(&conn, well_id, &names).map_err(|e| e.to_string())?;
    let in_range = |d: f32| -> bool {
        let d = d as f64;
        top.is_none_or(|t| d >= t) && bottom.is_none_or(|b| d <= b)
    };
    let mean_of = |name: &str| -> (Option<f64>, usize) {
        let col = cols.get(name).expect("requested curve present");
        let mut s = 0.0;
        let mut n = 0usize;
        for (i, v) in col.iter().enumerate() {
            if v.is_finite() && depth[i].is_finite() && in_range(depth[i]) {
                s += *v as f64;
                n += 1;
            }
        }
        (if n > 0 { Some(s / n as f64) } else { None }, n)
    };
    let (ftemp_f, n_ftemp) = mean_of("FTEMP_F");
    let (rmf, n_rmf) = mean_of("RMF");
    Ok(PrecalcFluid { ftemp_f, rmf, n_ftemp, n_rmf })
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

const SIGMA_CONSTRAINT: f64 = 0.01; // nominal Tool-constraint uncertainty

struct ZoneSets {
    /// Unity coefficients (1 for minerals/clays/U-and-shared fluids, 0 for X fluids).
    unity: Vec<f64>,
    clays: Vec<usize>,
    x_water: Vec<usize>,
    u_water: Vec<usize>,
    x_hc: Vec<usize>,
    u_hc: Vec<usize>,
    x_bw: Vec<usize>,
    u_bw: Vec<usize>,
    x_fluids: Vec<usize>,
    u_fluids: Vec<usize>,
    has_split: bool,
}

fn classify(comps: &[Component]) -> ZoneSets {
    let n = comps.len();
    let mut z = ZoneSets {
        unity: vec![0.0; n],
        clays: vec![],
        x_water: vec![],
        u_water: vec![],
        x_hc: vec![],
        u_hc: vec![],
        x_bw: vec![],
        u_bw: vec![],
        x_fluids: vec![],
        u_fluids: vec![],
        has_split: false,
    };
    for (i, c) in comps.iter().enumerate() {
        let is_fluid = c.kind.eq_ignore_ascii_case("fluid");
        let zone = c.zone.trim().to_uppercase();
        if !is_fluid {
            z.unity[i] = 1.0;
            if c.kind.eq_ignore_ascii_case("clay") {
                z.clays.push(i);
            }
            continue;
        }
        let in_x = zone != "U"; // X or shared
        let in_u = zone != "X"; // U or shared
        if in_u {
            z.unity[i] = 1.0;
            z.u_fluids.push(i);
        }
        if in_x {
            z.x_fluids.push(i);
        }
        match c.fluid_type.trim().to_lowercase().as_str() {
            "bound_water" => {
                if in_x {
                    z.x_bw.push(i);
                }
                if in_u {
                    z.u_bw.push(i);
                }
            }
            "oil" | "gas" => {
                if in_x {
                    z.x_hc.push(i);
                }
                if in_u {
                    z.u_hc.push(i);
                }
            }
            _ => {
                if in_x {
                    z.x_water.push(i);
                }
                if in_u {
                    z.u_water.push(i);
                }
            }
        }
    }
    // A real X/U split exists only if some fluid is exclusive to each zone.
    let has_x_only = comps.iter().any(|c| c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("X"));
    let has_u_only = comps.iter().any(|c| c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U"));
    z.has_split = has_x_only && has_u_only;
    z
}

/// Scale the volumes at `idx` so they sum to `target`, preserving their relative split (even split
/// when they are all ~0). The post-solve Sw models use it to impose Sw·φe on the water set and
/// (1−Sw)·φe on the HC set without changing φe — so PHIE and hard unity stay exactly as solved.
fn set_group(x: &mut [f64], idx: &[usize], target: f64) {
    if idx.is_empty() {
        return;
    }
    let cur: f64 = idx.iter().map(|&c| x[c]).sum();
    if cur > 1e-12 {
        let s = target / cur;
        for &c in idx {
            x[c] *= s;
        }
    } else {
        let each = target / idx.len() as f64;
        for &c in idx {
            x[c] = each;
        }
    }
}

pub fn run_multimin(
    db: &Mutex<Connection>,
    req: &MultiminRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> MultiminResult {
    let n = req.components.len();
    if n < 2 {
        return fail("select at least two components");
    }
    let model = req.sw_model;
    let post_solve = model.is_post_solve();
    let tools: Vec<&ToolSpec> = req.tools.iter().filter(|t| !t.curve.trim().is_empty()).collect();
    if tools.is_empty() {
        return fail("select at least one input log");
    }
    if req.apply_well_ids.is_empty() {
        return fail("select at least one well to apply to");
    }

    let has_cond = tools.iter().any(|t| is_cond_key(&t.key));
    let fluid = if has_cond {
        match &req.fluid {
            Some(p) => Some(fluid_calc(p)),
            None => return fail("CT/CXO selected but fluid properties (Rw, Rmf, temperature, m, n) are missing"),
        }
    } else {
        req.fluid.as_ref().map(fluid_calc)
    };

    let zs = classify(&req.components);
    if has_cond && zs.x_water.is_empty() && zs.u_water.is_empty() && zs.x_bw.is_empty() && zs.u_bw.is_empty() {
        return fail("CT/CXO selected but no water component is in the model");
    }
    // Post-solve shaly-sand Sw (Indonesia/Simandoux): the conductivity tool STAYS in the inversion —
    // dropping it would leave the U-zone water/HC split collinear (both invisible to the nuclear
    // tools) and the solve singular. We keep the well-posed linear solve and only REPLACE the
    // reported Sw with the closed form, reading Rt/Rxo straight from the conductivity tools' columns.
    // Sw redistributes φe into (Sw·φe water, (1−Sw)·φe HC), so a U-zone HC component must be present.
    let ct_tool_idx = tools.iter().position(|t| t.key.trim().eq_ignore_ascii_case("CT"));
    let cxo_tool_idx = tools.iter().position(|t| t.key.trim().eq_ignore_ascii_case("CXO"));
    // A shared-zone (zone "") water/HC sits in BOTH the x_* and u_* sets, so the U then X overrides
    // would scale it twice and corrupt PHIE/SWE/unity. The flushed-zone override therefore runs only
    // when the X and U fluid sets are disjoint (the standard zone-exclusive Sxo/Sw model); a shared
    // fluid means no invasion (Sxo = Sw), so leaving the X split as solved is already correct.
    let post_zones_disjoint = zs.x_water.iter().all(|i| !zs.u_water.contains(i))
        && zs.x_hc.iter().all(|i| !zs.u_hc.contains(i));
    if post_solve {
        if ct_tool_idx.is_none() {
            return fail("the non-linear Sw models need the deep-resistivity tool (CT) — add it");
        }
        if zs.u_water.is_empty() || zs.u_hc.is_empty() {
            return fail(
                "the non-linear Sw models need both a U-zone water and a U-zone hydrocarbon component",
            );
        }
    }

    // Static per-tool data: weights, endpoint rows (conductivity rows built from fluid calc).
    let t_c = req.fluid.as_ref().map(|p| (p.ftemp_f - 32.0) * 5.0 / 9.0).unwrap_or(25.0);
    let mut weights: Vec<f64> = Vec::with_capacity(tools.len());
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(tools.len());
    let mut tkind: Vec<TKind> = Vec::with_capacity(tools.len());
    for t in &tools {
        let key = t.key.trim().to_uppercase();
        if is_cond_key(&key) {
            let fc = fluid.as_ref().unwrap();
            let inv_w = 1.0 / fc.w;
            let mut row = vec![0.0f64; n];
            if key == "CT" {
                for &i in &zs.u_water {
                    row[i] = fc.cw.powf(inv_w);
                }
                for &i in &zs.u_bw {
                    row[i] = fc.cbw_u.powf(inv_w);
                }
            } else {
                for &i in &zs.x_water {
                    row[i] = fc.cmf.powf(inv_w);
                }
                for &i in &zs.x_bw {
                    row[i] = fc.cbw_x.powf(inv_w);
                }
            }
            // An all-zero conductivity row is the bogus equation 0 = Ct^(1/w): it happens when
            // the model has no water/bound-water in this tool's zone (e.g. CT but only X-zone
            // water). The whole-model no-water case is caught earlier; this catches the
            // per-zone case that slips past it.
            if row.iter().all(|&e| e == 0.0) {
                let need = if key == "CT" { "U-zone (deep) water or bound-water" } else { "X-zone (flushed) water or bound-water" };
                return fail(&format!(
                    "{key} selected but the model has no {need} component — its response row is all zero"
                ));
            }
            let auto_sigma = if key == "CT" { fc.u_ct } else { fc.u_cxo };
            let sigma = if t.sigma > 0.0 { t.sigma } else { auto_sigma };
            weights.push(1.0 / sigma.max(1e-9));
            rows.push(row);
            tkind.push(TKind::Cond(fc.w));
        } else if is_pef_key(&key) {
            // PEF is a PER-ELECTRON index and does NOT mix by volume; only U = Pe·ρe
            // does. Build the mixing row from the U endpoints and convert the measured
            // PEF curve to U per sample (see the sample loop / rho_e).
            if t.sigma <= 0.0 {
                return fail(&format!("tool {key} needs a positive uncertainty"));
            }
            let row: Vec<f64> = req
                .components
                .iter()
                .map(|c| {
                    if c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U") {
                        0.0
                    } else {
                        *c.endpoints.get("U").unwrap_or(&f64::NAN)
                    }
                })
                .collect();
            if row.iter().any(|e| !e.is_finite()) {
                return fail(
                    "PEF selected but a component is missing its U (b/cc) endpoint — PEF is converted to U before mixing",
                );
            }
            weights.push(1.0 / t.sigma); // base; the per-sample weight also divides by ρe
            rows.push(row);
            tkind.push(TKind::Pef(t.sigma));
        } else {
            if t.sigma <= 0.0 {
                return fail(&format!("tool {key} needs a positive uncertainty"));
            }
            let row: Vec<f64> = req
                .components
                .iter()
                .map(|c| {
                    // U-zone-exclusive fluids are invisible to all non-CT tools.
                    if c.kind.eq_ignore_ascii_case("fluid") && c.zone.eq_ignore_ascii_case("U") {
                        0.0
                    } else {
                        *c.endpoints.get(&key).unwrap_or(&f64::NAN)
                    }
                })
                .collect();
            if row.iter().any(|e| !e.is_finite()) {
                return fail(&format!("tool {key} has a missing endpoint — fill the endpoints table"));
            }
            weights.push(1.0 / t.sigma);
            rows.push(row);
            tkind.push(TKind::Plain);
        }
    }

    // Soft constraint rows (built once; appended after the live tool rows each sample).
    let mut soft: Vec<(Vec<f64>, f64)> = Vec::new();
    if zs.has_split {
        let mut row = vec![0.0f64; n];
        for &i in &zs.x_fluids {
            row[i] += 1.0;
        }
        for &i in &zs.u_fluids {
            row[i] -= 1.0;
        }
        soft.push((row, 0.0)); // POROSITY: Σ X fluids − Σ U fluids = 0
    }
    if !zs.clays.is_empty() {
        let fc_alpha = |x: bool| fluid.as_ref().map(|f| if x { f.alpha_x } else { f.alpha_u }).unwrap_or(1.0);
        let mut bw_rows: Vec<(Vec<usize>, f64)> = Vec::new();
        if !zs.x_bw.is_empty() && zs.x_bw != zs.u_bw {
            bw_rows.push((zs.x_bw.clone(), fc_alpha(true)));
        }
        if !zs.u_bw.is_empty() {
            bw_rows.push((zs.u_bw.clone(), fc_alpha(false)));
        }
        for (bw_idx, alpha) in bw_rows {
            let mut row = vec![0.0f64; n];
            let mut any = false;
            for &ci in &zs.clays {
                let c = &req.components[ci];
                if c.cec > 0.0 {
                    let rho = *c.endpoints.get("RHOB").unwrap_or(&2.65);
                    row[ci] = bndwat_multiplier(c.cec, rho, t_c, alpha);
                    any = true;
                }
            }
            if any {
                for &bi in &bw_idx {
                    row[bi] = -1.0;
                }
                soft.push((row, 0.0)); // BNDWAT: Σ k·v_clay − v_bw = 0
            }
        }
    }
    let soft_weight = 1.0 / SIGMA_CONSTRAINT;

    // WATER MUD row (used only on violation re-solve): Σ X waters − Σ U waters = 0.
    let water_mud_row: Option<Vec<f64>> = if zs.has_split
        && req.fluid.as_ref().map(|p| !p.mud_type.eq_ignore_ascii_case("OIL")).unwrap_or(true)
    {
        let mut row = vec![0.0f64; n];
        for &i in zs.x_water.iter().chain(&zs.x_bw) {
            row[i] += 1.0;
        }
        for &i in zs.u_water.iter().chain(&zs.u_bw) {
            row[i] -= 1.0;
        }
        Some(row)
    } else {
        None
    };

    let hi: Vec<f64> = req.components.iter().map(|c| if c.max_vol > 0.0 { c.max_vol.min(1.0) } else { 1.0 }).collect();
    let unity_c = if req.unity { Some(zs.unity.clone()) } else { None };

    // Minimum live tool rows per sample: volumes minus unity/soft-constraint degrees of freedom.
    let n_extra = soft.len() + usize::from(req.unity);
    let min_tools = n.saturating_sub(n_extra).max(1);
    if tools.len() < min_tools {
        return fail(&format!(
            "need at least {min_tools} input logs to constrain {n} components (have {})",
            tools.len()
        ));
    }
    // Model degrees of freedom with every tool live: equations (tools + soft + unity) − unknowns.
    let dof = tools.len() as i64 + n_extra as i64 - n as i64;
    let dof_note = (dof == 0).then(|| {
        format!(
            "exactly determined: {} equation(s) for {n} component(s), so RECON is forced to ~0 and \
             cannot validate the model — add an input log for a real reconstruction check",
            tools.len() + n_extra
        )
    });
    let recon_qc = req.recon_qc;

    let comp_tokens: Vec<String> = req.components.iter().map(|c| curve_token(&c.name)).collect();
    let vol_names: Vec<String> = comp_tokens.iter().map(|t| format!("VOL_{t}")).collect();
    // Uppercase the output prefix so a re-cased prefix (e.g. "mm" after "MM") can't leave a stale
    // case-shadow row: every computed-curve reader resolves case-insensitively but the delete-then-
    // append writer deletes by exact curve_name. Matches curve_token()'s uppercasing of components.
    let prefix_upper = req.output_prefix.trim().to_uppercase();
    let prefix = if prefix_upper.is_empty() { "MM" } else { prefix_upper.as_str() };

    let fetch_names: Vec<String> = tools.iter().map(|t| t.curve.trim().to_uppercase()).collect();
    // PEF→U conversion needs the density curve even when RHOB is not itself a tool.
    // Resolve the density source and fetch it alongside (without disturbing tool_cols).
    let density_name: Option<String> = if tkind.iter().any(|k| matches!(k, TKind::Pef(_))) {
        Some(
            tools
                .iter()
                .find(|t| t.key.trim().eq_ignore_ascii_case("RHOB"))
                .map(|t| t.curve.trim().to_uppercase())
                .unwrap_or_else(|| "RHOB".to_string()),
        )
    } else {
        None
    };
    let mut all_fetch = fetch_names.clone();
    if let Some(d) = &density_name {
        if !all_fetch.contains(d) {
            all_fetch.push(d.clone());
        }
    }

    let mut out_names: Vec<String> = Vec::new();
    let mut wells: Vec<MultiminWellResult> = Vec::new();

    let n_wells = req.apply_well_ids.len();
    let conn = db.lock().unwrap();
    for (wi, well_id) in req.apply_well_ids.iter().enumerate() {
        // Cooperative cancel + live per-well progress into the Processing panel. The panel polls
        // the (separate) jobs registry, so these updates land even while this holds the DB lock.
        if let Some(p) = progress {
            if p.is_cancelled() {
                break;
            }
            p.set_current(Some(format!("SandiMin: well {}/{}", wi + 1, n_wells)));
            p.start_item(well_id);
        }
        let (depth, cols) = match fetch_curve_frame(&conn, well_id, &all_fetch) {
            Ok(v) => v,
            Err(e) => {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                }
                wells.push(MultiminWellResult {
                    well_id: well_id.clone(),
                    rows_solved: 0,
                    mean_recon: f32::NAN,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };
        let ns = depth.len();
        let tool_cols: Vec<&Vec<f32>> = fetch_names.iter().map(|nm| cols.get(nm).unwrap()).collect();
        let rhob_col: Option<&Vec<f32>> = density_name.as_ref().and_then(|d| cols.get(d));

        let mut vol: Vec<Vec<f32>> = vec![vec![f32::NAN; ns]; n];
        let mut recon = vec![f32::NAN; ns];
        let mut solved = 0usize;
        let mut recon_sum = 0.0f64;
        // Per-tool reconstruction QC (recon_qc): reconstructed measurement + σ-unit residual.
        let mut tool_rec: Vec<Vec<f32>> = if recon_qc { vec![vec![f32::NAN; ns]; tools.len()] } else { Vec::new() };
        let mut tool_dif: Vec<Vec<f32>> = if recon_qc { vec![vec![f32::NAN; ns]; tools.len()] } else { Vec::new() };

        for i in 0..ns {
            let mut a: Vec<Vec<f64>> = Vec::with_capacity(tools.len() + soft.len());
            let mut b: Vec<f64> = Vec::with_capacity(tools.len() + soft.len());
            let mut live_tools = 0usize;
            // (tool index, measured value in solve domain, weight) for the reconstruction QC.
            let mut live: Vec<(usize, f64, f64)> = Vec::new();
            for (t, tcol) in tool_cols.iter().enumerate() {
                let raw = tcol[i] as f64;
                if !raw.is_finite() {
                    continue;
                }
                let (v, w) = match tkind[t] {
                    TKind::Plain => (raw, weights[t]),
                    TKind::Cond(w_exp) => {
                        // Resistivity (ohmm) → conductivity (mho/m) → ^(1/w) transform.
                        if raw <= 1e-4 {
                            continue;
                        }
                        ((1.0 / raw).powf(1.0 / w_exp), weights[t])
                    }
                    TKind::Pef(sig) => {
                        // U = Pe·ρe (volumetric); its uncertainty in U space is σ_PEF·ρe.
                        let rhob = match rhob_col.map(|c| c[i] as f64) {
                            Some(rb) if rb.is_finite() && rb > 0.0 => rb,
                            _ => continue,
                        };
                        let re = rho_e(rhob);
                        (raw * re, 1.0 / (sig * re).max(1e-9))
                    }
                };
                a.push(rows[t].iter().map(|e| e * w).collect());
                b.push(v * w);
                if recon_qc {
                    live.push((t, v, w));
                }
                live_tools += 1;
            }
            if live_tools < min_tools {
                continue;
            }
            let n_tool_rows = a.len();
            for (row, rhs) in &soft {
                a.push(row.iter().map(|e| e * soft_weight).collect());
                b.push(rhs * soft_weight);
            }

            let mut x = match solve_bounded_lsq(&a, &b, unity_c.as_deref(), &hi, n) {
                Some(x) => x,
                None => continue,
            };
            // WATER MUD: for WBM, flushed-zone water cannot be less than unflushed water.
            if let Some(wm) = &water_mud_row {
                let s: f64 = wm.iter().zip(&x).map(|(c, v)| c * v).sum();
                if s < -1e-6 {
                    let mut a2 = a.clone();
                    let mut b2 = b.clone();
                    a2.push(wm.iter().map(|e| e * soft_weight).collect());
                    b2.push(0.0);
                    if let Some(x2) = solve_bounded_lsq(&a2, &b2, unity_c.as_deref(), &hi, n) {
                        x = x2;
                    }
                }
            }

            // Post-solve shaly-sand Sw (Indonesia/Simandoux): the linear inversion above fixed φe and
            // Vsh; now REPLACE the water/HC split with the closed-form Sw. φe is preserved, so PHIE
            // and hard unity are untouched and SWE becomes the model Sw. RECON is computed AFTER this,
            // so for these models it also measures how well the chosen Sw coheres with every tool.
            if post_solve {
                let fc = fluid.as_ref().unwrap();
                let fp = req.fluid.as_ref().unwrap();
                let (m_exp, n_exp, a_arch, rsh) = (fp.m, fp.n, fp.archie_a, fp.rsh);
                let vsh = zs.clays.iter().chain(&zs.u_bw).map(|&c| x[c]).sum::<f64>();
                let read_res = |idx: Option<usize>| -> Option<f64> {
                    idx.map(|t| tool_cols[t][i] as f64).filter(|v| v.is_finite() && *v > 0.0)
                };
                // Returns the EFFECTIVE water fraction (free water / φe) so the φe redistribution below is
                // one code path for every model. Indonesia/Simandoux read Rw = 1/cw; the dual-water form
                // additionally uses the zone's clay-bound-water conductivity `cwb` and solved v_bw.
                let sw_of = |rt: f64, phie: f64, cw: f64, cwb: f64, v_bw: f64| -> f64 {
                    match model {
                        SwModel::Indonesia => {
                            sw_indonesia(rt, phie, vsh, 1.0 / cw.max(1e-9), rsh, m_exp, n_exp, a_arch)
                        }
                        SwModel::Simandoux => {
                            sw_simandoux(rt, phie, vsh, 1.0 / cw.max(1e-9), rsh, m_exp, n_exp, a_arch)
                        }
                        SwModel::DualWaterNonlinear => {
                            // Total-basis dual water: bound-water saturation from the solved v_bw, then
                            // convert Swt back to a free-water/φe fraction. φe (= φt − v_bw) is preserved,
                            // so PHIT/PHIE/unity stay as solved and only the water/HC split moves.
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swb = (v_bw / phit).clamp(0.0, 1.0);
                            let swt = sw_dual_nonlinear(rt, phit, swb, cw, cwb, m_exp, n_exp, a_arch);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            ((swt * phit - v_bw) / phie).clamp(0.0, 1.0)
                        }
                        SwModel::Archie => {
                            // Clean-sand Archie on total porosity, then free-water/φe (same conversion as
                            // dual water; the total water Swt·φt includes the solved v_bw as bound water).
                            let phit = phie + v_bw;
                            if !(phit > 1e-9) || !(phie > 1e-9) {
                                return f64::NAN;
                            }
                            let swt = sw_archie(rt, phit, 1.0 / cw.max(1e-9), m_exp, n_exp, a_arch);
                            if !swt.is_finite() {
                                return f64::NAN;
                            }
                            ((swt * phit - v_bw) / phie).clamp(0.0, 1.0)
                        }
                        SwModel::LinearDw => f64::NAN,
                    }
                };
                // U zone (deep): Rt against Rw.
                let phie_u = zs.u_water.iter().chain(&zs.u_hc).map(|&c| x[c]).sum::<f64>();
                if phie_u > 1e-6 {
                    if let Some(rt) = read_res(ct_tool_idx) {
                        let v_bw_u = zs.u_bw.iter().map(|&c| x[c]).sum::<f64>();
                        let sw = sw_of(rt, phie_u, fc.cw, fc.cbw_u, v_bw_u);
                        if sw.is_finite() {
                            set_group(&mut x, &zs.u_water, sw * phie_u);
                            set_group(&mut x, &zs.u_hc, (1.0 - sw) * phie_u);
                        }
                    }
                }
                // X zone (flushed): Rxo against Rmf — only with a real, zone-disjoint X/U split and an
                // X-zone HC (post_zones_disjoint keeps a shared-zone fluid from being scaled twice).
                if zs.has_split && post_zones_disjoint && !zs.x_hc.is_empty() {
                    let phie_x = zs.x_water.iter().chain(&zs.x_hc).map(|&c| x[c]).sum::<f64>();
                    if phie_x > 1e-6 {
                        if let Some(rxo) = read_res(cxo_tool_idx) {
                            let v_bw_x = zs.x_bw.iter().map(|&c| x[c]).sum::<f64>();
                            let sxo = sw_of(rxo, phie_x, fc.cmf, fc.cbw_x, v_bw_x);
                            if sxo.is_finite() {
                                set_group(&mut x, &zs.x_water, sxo * phie_x);
                                set_group(&mut x, &zs.x_hc, (1.0 - sxo) * phie_x);
                            }
                        }
                    }
                }
            }

            // Weighted RMS residual over the live tool rows only (σ units).
            let mut sse = 0.0;
            for (row, &bi) in a.iter().zip(&b).take(n_tool_rows) {
                let pred: f64 = row.iter().zip(&x).map(|(ai, xi)| ai * xi).sum();
                let d = pred - bi;
                sse += d * d;
            }
            let rerr = (sse / n_tool_rows as f64).sqrt();

            for c in 0..n {
                vol[c][i] = x[c] as f32;
            }
            recon[i] = rerr as f32;
            recon_sum += rerr;
            solved += 1;

            // Per-tool reconstruction: rebuild each live tool's reading from the solved volumes.
            // rec_native = rows[t]·x is in the tool's SOLVE domain; the σ-unit residual
            // (rec_native − v)·w is exactly that tool's term of RECON (Σ term² / n_tool_rows = rerr²).
            if recon_qc {
                let rhob_i = rhob_col.map(|c| c[i] as f64);
                for &(t, v, w) in &live {
                    let rec_native: f64 = rows[t].iter().zip(&x).map(|(e, xi)| e * xi).sum();
                    tool_dif[t][i] = ((rec_native - v) * w) as f32;
                    tool_rec[t][i] = recon_display(&tkind[t], rec_native, rhob_i) as f32;
                }
            }
        }

        // Derived output curves from the solved volumes.
        let sum_over = |idx: &[usize], i: usize| -> f32 { idx.iter().map(|&c| vol[c][i]).sum() };
        let make = |f: &dyn Fn(usize) -> f32| -> Vec<f32> {
            (0..ns).map(|i| if vol[0][i].is_finite() { f(i) } else { f32::NAN }).collect()
        };

        let mut curves: Vec<(String, Vec<f32>)> = Vec::with_capacity(n + 10);
        for (name, values) in vol_names.iter().zip(&vol) {
            curves.push((name.clone(), values.clone()));
        }
        let has_u_fluids = !zs.u_fluids.is_empty();
        if has_u_fluids {
            let phie = make(&|i| sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i));
            let phit = make(&|i| sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i) + sum_over(&zs.u_bw, i));
            let swe = make(&|i| {
                let p = sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i);
                if p > 1e-6 { sum_over(&zs.u_water, i) / p } else { f32::NAN }
            });
            let swt = make(&|i| {
                let p = sum_over(&zs.u_water, i) + sum_over(&zs.u_hc, i) + sum_over(&zs.u_bw, i);
                if p > 1e-6 { (sum_over(&zs.u_water, i) + sum_over(&zs.u_bw, i)) / p } else { f32::NAN }
            });
            curves.push((format!("{prefix}_PHIE"), phie));
            curves.push((format!("{prefix}_PHIT"), phit));
            curves.push((format!("{prefix}_SWE"), swe));
            curves.push((format!("{prefix}_SWT"), swt));
        }
        if zs.has_split {
            let sxot = make(&|i| {
                let p = sum_over(&zs.x_water, i) + sum_over(&zs.x_hc, i) + sum_over(&zs.x_bw, i);
                if p > 1e-6 { (sum_over(&zs.x_water, i) + sum_over(&zs.x_bw, i)) / p } else { f32::NAN }
            });
            curves.push((format!("{prefix}_SXOT"), sxot));
            if !zs.u_hc.is_empty() || !zs.x_hc.is_empty() {
                let moved = make(&|i| sum_over(&zs.u_hc, i) - sum_over(&zs.x_hc, i));
                curves.push((format!("{prefix}_MOVEDHC"), moved));
            }
        }
        if !zs.clays.is_empty() {
            let vsh = make(&|i| sum_over(&zs.clays, i) + sum_over(&zs.u_bw, i));
            curves.push((format!("{prefix}_VSH"), vsh));
        }
        // RECON = the incoherence (σ-weighted RMS residual over live tool rows; Quanti.Elan Eq 79).
        curves.push((format!("{prefix}_RECON"), recon));
        // Per-tool reconstruction QC decomposition (opt-in): rebuilt reading + σ-unit residual.
        if recon_qc {
            for (t, tool) in tools.iter().enumerate() {
                let tag = curve_token(&tool.key);
                curves.push((format!("{prefix}_{tag}_REC"), std::mem::take(&mut tool_rec[t])));
                curves.push((format!("{prefix}_{tag}_DIF"), std::mem::take(&mut tool_dif[t])));
            }
        }

        if out_names.is_empty() {
            out_names = curves.iter().map(|(n, _)| n.clone()).collect();
        }
        let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
        let spec = crate::equations::LogSetSpec {
            set_name: "SANDIMIN".into(),
            module: "sandimin".into(),
            params_json: serde_json::to_string(&serde_json::json!({
                "components": req.components.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
                "prefix": prefix,
                "sw_model": match model {
                    SwModel::LinearDw => "linear_dw",
                    SwModel::DualWaterNonlinear => "dual_water_nonlinear",
                    SwModel::Archie => "archie",
                    SwModel::Indonesia => "indonesia",
                    SwModel::Simandoux => "simandoux",
                },
            }))
            .unwrap_or_default(),
            inputs_json: serde_json::to_string(&req.tools.iter().map(|t| t.curve.as_str()).collect::<Vec<_>>())
                .unwrap_or_default(),
        };
        let write_err = crate::equations::create_log_set(&conn, well_id, &spec)
            .and_then(|(set_id, _)| write_computed_curves_versioned(&conn, well_id, &depth, &refs, &set_id))
            .err()
            .map(|e| e.to_string());
        if let Some(p) = progress {
            // A write failure is Failed; a solve that produced nothing is a Warned caveat; else Ok.
            let (state, msg) = if let Some(e) = &write_err {
                (crate::jobs::ItemState::Failed, Some(e.clone()))
            } else if solved == 0 {
                (crate::jobs::ItemState::Warned, Some("no solvable samples (too few live input logs)".to_string()))
            } else {
                (crate::jobs::ItemState::Ok, None)
            };
            p.finish_item(well_id, state, msg);
        }
        wells.push(MultiminWellResult {
            well_id: well_id.clone(),
            rows_solved: solved,
            mean_recon: if solved > 0 { (recon_sum / solved as f64) as f32 } else { f32::NAN },
            error: write_err.or_else(|| (solved == 0).then(|| "no solvable samples (too few live input logs)".to_string())),
        });
    }

    MultiminResult { outputs: out_names, wells, dof, dof_note, error: None }
}

/// Rebuilds a tool's measurement in its DISPLAY domain from the solved native prediction:
/// Plain tools are already physical; a conductivity row predicts C^(1/w) → resistivity = pred^−w;
/// a PEF row predicts U = Pe·ρe → PEF = U/ρe.
fn recon_display(kind: &TKind, native: f64, rhob: Option<f64>) -> f64 {
    match *kind {
        TKind::Plain => native,
        TKind::Cond(w) => {
            if native > 1e-12 {
                native.powf(-w)
            } else {
                f64::NAN
            }
        }
        TKind::Pef(_) => match rhob {
            Some(rb) if rb.is_finite() && rb > 0.0 => native / rho_e(rb),
            _ => f64::NAN,
        },
    }
}

fn is_cond_key(key: &str) -> bool {
    let k = key.trim().to_uppercase();
    k == "CT" || k == "CXO"
}

fn is_pef_key(key: &str) -> bool {
    key.trim().eq_ignore_ascii_case("PEF")
}

/// Litho-Density electron density from bulk density: inverse of ρₐ = 1.0704·ρₑ − 0.1883.
fn rho_e(rhob: f64) -> f64 {
    (rhob + 0.1883) / 1.0704
}

/// Volumetric photoelectric factor U = Pe·ρₑ — the quantity that mixes linearly by
/// volume. Per-electron PEF does NOT, so a measured PEF reading is converted to U here
/// before it enters the linear system. `None` for a non-physical RHOB.
#[allow(dead_code)] // exercised by the pef_converts_to_u_before_mixing test; inline in the hot path
fn pef_to_u(pef: f64, rhob: f64) -> Option<f64> {
    (pef.is_finite() && rhob.is_finite() && rhob > 0.0).then(|| pef * rho_e(rhob))
}

/// How a tool's measurement enters the least-squares system.
#[derive(Clone, Copy)]
enum TKind {
    /// Endpoint response mixes linearly (RHOB, NPHI, DT, GR, U, …).
    Plain,
    /// Resistivity → conductivity^(1/w) transform (CT/CXO); carries w.
    Cond(f64),
    /// PEF → U = Pe·ρe conversion before mixing; carries the PEF-space σ.
    Pef(f64),
}

// ---------------------------------------------------------------------------
// Solver — bounded (0 ≤ v ≤ hi), optionally equality-constrained (cᵀv = 1)
// active-set least squares.
// ---------------------------------------------------------------------------

/// A^T (A v − b), the gradient of ½‖Av−b‖².
fn grad(a: &[Vec<f64>], b: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    let m = a.len();
    let mut g = vec![0.0; n];
    for i in 0..m {
        let pred: f64 = (0..n).map(|c| a[i][c] * v[c]).sum();
        let r = pred - b[i];
        for (c, gc) in g.iter_mut().enumerate() {
            *gc += a[i][c] * r;
        }
    }
    g
}

#[derive(Clone, Copy, PartialEq)]
enum BState {
    Free,
    AtLo,
    AtHi,
}

/// min‖Av−b‖² s.t. 0 ≤ v ≤ hi and (optionally) cᵀv = 1.
/// Active-set: solve the KKT system on the free set (fixed components folded into the RHS),
/// line-search from the current feasible point to the first bound crossing, and release
/// bound components whose KKT multiplier has the wrong sign.
fn solve_bounded_lsq(
    a: &[Vec<f64>],
    b: &[f64],
    unity_c: Option<&[f64]>,
    hi: &[f64],
    n: usize,
) -> Option<Vec<f64>> {
    if a.is_empty() {
        return None;
    }
    // Feasible start: spread the unity budget over the unity components, capped by hi;
    // non-unity (X-zone fluid) components start mid-box.
    let mut v = vec![0.0f64; n];
    match unity_c {
        Some(c) => {
            let k = c.iter().filter(|&&ci| ci != 0.0).count().max(1);
            let mut budget = 1.0f64;
            let share = 1.0 / k as f64;
            for i in 0..n {
                if c[i] != 0.0 {
                    v[i] = share.min(hi[i]);
                    budget -= c[i] * v[i];
                }
            }
            // Distribute any remainder (from hi caps) over uncapped unity components.
            if budget.abs() > 1e-12 {
                for i in 0..n {
                    if c[i] != 0.0 && v[i] < hi[i] {
                        let add = (budget / c[i]).min(hi[i] - v[i]).max(0.0);
                        v[i] += add;
                        budget -= c[i] * add;
                        if budget.abs() < 1e-12 {
                            break;
                        }
                    }
                }
            }
            for i in 0..n {
                if c[i] == 0.0 {
                    v[i] = (0.25f64).min(hi[i]);
                }
            }
        }
        None => {
            for i in 0..n {
                v[i] = (0.25f64).min(hi[i]);
            }
        }
    }

    let mut state = vec![BState::Free; n];
    let mut solved_once = false;
    let max_outer = 8 * n + 12;

    for _ in 0..max_outer {
        let free: Vec<usize> = (0..n).filter(|&c| state[c] == BState::Free).collect();
        if free.is_empty() {
            return if solved_once { Some(v) } else { None };
        }

        // Contribution of components fixed at their upper bound.
        let fixed_hi: Vec<usize> = (0..n).filter(|&c| state[c] == BState::AtHi).collect();
        let m = a.len();
        let k = free.len();
        let with_unity = unity_c.is_some();
        let dim = k + usize::from(with_unity);
        let mut mat = vec![vec![0.0f64; dim]; dim];
        let mut rhs = vec![0.0f64; dim];
        // b' = b − A_H v_H
        let bprime: Vec<f64> = (0..m)
            .map(|i| b[i] - fixed_hi.iter().map(|&c| a[i][c] * hi[c]).sum::<f64>())
            .collect();
        for p in 0..k {
            for q in 0..k {
                let g: f64 = (0..m).map(|i| a[i][free[p]] * a[i][free[q]]).sum();
                mat[p][q] = 2.0 * g;
            }
            rhs[p] = 2.0 * (0..m).map(|i| a[i][free[p]] * bprime[i]).sum::<f64>();
        }
        if let Some(c) = unity_c {
            for p in 0..k {
                mat[p][k] = c[free[p]];
                mat[k][p] = c[free[p]];
            }
            rhs[k] = 1.0 - fixed_hi.iter().map(|&i| c[i] * hi[i]).sum::<f64>();
        }
        let sol = match solve_linear_opt(mat, rhs) {
            Some(s) if s.iter().all(|x| x.is_finite()) => s,
            _ => return if solved_once { Some(v) } else { None },
        };
        let mut u = vec![0.0f64; n];
        for (idx, &c) in free.iter().enumerate() {
            u[c] = sol[idx];
        }
        for &c in &fixed_hi {
            u[c] = hi[c];
        }
        let mu = if with_unity { sol[k] } else { 0.0 };

        let feasible = free.iter().all(|&c| u[c] >= -1e-12 && u[c] <= hi[c] + 1e-12);
        if feasible {
            for c in 0..n {
                v[c] = u[c].clamp(0.0, hi[c]);
            }
            solved_once = true;
            // KKT sign check on bound components: at-lo needs 2g+μc ≥ 0, at-hi needs ≤ 0.
            let g = grad(a, b, &v, n);
            let cvec = unity_c;
            let mut release = None;
            let mut worst = 1e-9;
            for c in 0..n {
                let cc = cvec.map(|cv| cv[c]).unwrap_or(0.0);
                let kkt = 2.0 * g[c] + mu * cc;
                match state[c] {
                    BState::AtLo if -kkt > worst => {
                        worst = -kkt;
                        release = Some(c);
                    }
                    BState::AtHi if kkt > worst => {
                        worst = kkt;
                        release = Some(c);
                    }
                    _ => {}
                }
            }
            match release {
                Some(c) => state[c] = BState::Free,
                None => return Some(v),
            }
        } else {
            // Step from feasible v toward u until the first free component hits a bound.
            let mut alpha: f64 = 1.0;
            for &c in &free {
                let d = u[c] - v[c];
                if d < -1e-18 && u[c] < 0.0 {
                    alpha = alpha.min(v[c] / -d);
                } else if d > 1e-18 && u[c] > hi[c] {
                    alpha = alpha.min((hi[c] - v[c]) / d);
                }
            }
            let mut any_fixed = false;
            for c in 0..n {
                v[c] += alpha * (u[c] - v[c]);
            }
            for &c in &free {
                if v[c] <= 1e-10 {
                    v[c] = 0.0;
                    state[c] = BState::AtLo;
                    any_fixed = true;
                } else if v[c] >= hi[c] - 1e-10 {
                    v[c] = hi[c];
                    state[c] = BState::AtHi;
                    any_fixed = true;
                }
            }
            if !any_fixed {
                return if solved_once { Some(v) } else { None };
            }
        }
    }
    if solved_once {
        Some(v)
    } else {
        None
    }
}

/// Gaussian elimination with partial pivoting. Returns None for a singular system.
fn solve_linear_opt(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let k = b.len();
    for col in 0..k {
        let mut pivot = col;
        for r in (col + 1)..k {
            if a[r][col].abs() > a[pivot][col].abs() {
                pivot = r;
            }
        }
        if a[pivot][col].abs() < 1e-15 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for r in (col + 1)..k {
            let f = a[r][col] / a[col][col];
            for c in col..k {
                a[r][c] -= f * a[col][c];
            }
            b[r] -= f * b[col];
        }
    }
    let mut x = vec![0.0; k];
    for col in (0..k).rev() {
        let mut s = b[col];
        for c in (col + 1)..k {
            s -= a[col][c] * x[c];
        }
        x[col] = s / a[col][col];
    }
    Some(x)
}

// ---------------------------------------------------------------------------
// Endpoint library — reference §6.2 + IP2018 MINDEF.PAR defaults, merged.
// Display units: RHOB g/cc, NPHI v/v, DT us/ft, GR API, PEF b/e, U b/cc,
// THOR ppm, POTA %, URAN ppm, VP km/s, VS km/s, EPT ns/m, EATT dB/m, SIGMA c.u.
// CT/CXO endpoints are computed from the fluid properties at run time.
// ---------------------------------------------------------------------------

#[allow(dead_code)] // canonical tool-key ordering, referenced by docs/spec + future validation
pub const TOOL_KEYS: [&str; 14] =
    ["RHOB", "NPHI", "DT", "GR", "PEF", "U", "THOR", "POTA", "URAN", "VP", "VS", "EPT", "EATT", "SIGMA"];

struct LibRow {
    name: &'static str,
    kind: &'static str,
    zone: &'static str,
    fluid_type: &'static str,
    cec: f64,
    max_vol: f64,
    /// [RHOB, NPHI, DT, GR, PEF, U, THOR, POTA, URAN, EPT, SIGMA]  (VP/VS derived from DT)
    v: [f64; 11],
}

const fn m(name: &'static str, v: [f64; 11]) -> LibRow {
    LibRow { name, kind: "mineral", zone: "", fluid_type: "", cec: 0.0, max_vol: 1.0, v }
}
const fn clay(name: &'static str, cec: f64, v: [f64; 11]) -> LibRow {
    LibRow { name, kind: "clay", zone: "", fluid_type: "", cec, max_vol: 1.0, v }
}
const fn fl(name: &'static str, zone: &'static str, fluid_type: &'static str, v: [f64; 11]) -> LibRow {
    LibRow { name, kind: "fluid", zone, fluid_type, cec: 0.0, max_vol: 0.5, v }
}

/// Merged reference/IP default library, in IP's mineral-dropdown order (Jauhar's screenshot).
#[rustfmt::skip]
const LIB: &[LibRow] = &[
    //                       RHOB   NPHI    DT     GR    PEF     U    THOR  POTA  URAN   EPT  SIGMA
    m("Calcite",           [2.71,  0.000,  47.5, 11.0,  5.08, 13.8,  0.0,  0.00,  1.4,  9.1,  7.4]),
    m("Quartz",            [2.65, -0.050,  55.5,  1.0,  1.81,  4.8,  0.0,  0.00,  0.1,  7.2,  4.7]),
    m("Dolomite",          [2.85,  0.025,  43.5,  8.0,  3.14,  9.0,  0.1,  0.00,  0.9,  8.7,  6.9]),
    m("Orthoclase",        [2.57, -0.010,  69.0,171.0,  2.86,  8.7,  1.1, 10.21,  0.4,  7.6, 15.3]),
    m("Albite",            [2.60, -0.005,  49.0,  8.0,  1.68,  5.6,  0.0,  0.50,  0.0,  7.6, 11.4]),
    m("Anhydrite",         [2.98, -0.020,  50.0,  5.0,  5.05, 14.95, 0.2,  0.00,  0.4,  8.4, 12.0]),
    m("Halite",            [2.04, -0.030,  67.0,  5.0,  4.65,  9.7,  0.2,  0.00,  0.0,  8.2,750.0]),
    m("Gypsum",            [2.35,  0.540,  52.0,  5.0,  3.99,  9.46, 0.0,  0.00,  0.3,  6.8, 20.0]),
    m("Pyrite",            [4.99,  0.000,  39.2,  5.0, 16.97, 82.0,  0.0,  0.00,  0.0,  0.0, 90.0]),
    m("Siderite",          [3.88,  0.180,  44.0,  6.0, 14.70, 72.0,  0.4,  0.00,  0.5,  8.9, 54.2]),
    m("Muscovite",         [2.85,  0.240,  49.0,130.0,  2.40, 11.5,  0.0,  7.80,  0.7,  8.9, 95.3]),
    m("Biotite",           [3.04,  0.130,  50.8,127.0,  6.27, 21.6,  1.5,  7.20,  0.7,  7.8, 54.1]),
    clay("Glauconite", 0.20, [2.96, 0.410,  49.4,150.0,  5.32, 16.5,  2.8,  5.60,  5.1, 12.0, 89.6]),
    clay("Kaolinite",  0.10, [2.62, 0.451,  85.3,104.0,  1.83,  5.38,18.9,  0.08,  3.1,  8.0, 20.1]),
    clay("Chlorite",   0.15, [2.81, 0.520,  85.3, 56.0,  6.30, 21.7, 11.0,  0.67,  3.5,  8.0, 43.7]),
    clay("Illite",     0.25, [2.78, 0.247,  85.3,160.0,  4.00, 11.12,12.3,  4.48,  4.8,  8.0, 40.6]),
    clay("Montmorillonite",1.0,[2.63,0.218, 85.3,168.0,  2.70,  7.61,20.6,  0.58,  7.1,  8.0, 20.2]),
    clay("Clay",       0.00, [2.65, 0.350, 100.0,152.0,  3.50, 10.0,  6.0,  2.00, 12.0,  8.0, 30.0]),
    m("Coal",              [1.19,  0.520, 160.0, 10.0,  0.20,  0.24, 0.0,  0.00,  0.0,  0.0,  0.0]),
    m("Kerogen",           [1.10,  0.600, 150.0,100.0,  0.24,  0.26, 0.0,  0.00, 10.0,  0.0,  0.0]),
    fl("Water Sxo", "X", "water",       [1.00, 1.00, 189.0, 0.0, 0.36, 0.40, 0.0, 0.0, 0.0, 29.0, 50.0]),
    fl("Water Sw",  "U", "water",       [1.00, 1.00, 189.0, 0.0, 0.36, 0.40, 0.0, 0.0, 0.0, 29.0, 50.0]),
    fl("BoundWater", "", "bound_water", [1.00, 1.00, 189.0, 0.0, 0.36, 0.39, 0.0, 0.0, 0.0, 30.0, 50.0]),
    fl("Oil Sxo",   "X", "oil",         [0.80, 1.00, 189.0, 0.0, 0.12, 0.11, 0.0, 0.0, 0.0,  5.0, 21.0]),
    fl("Oil Sw",    "U", "oil",         [0.80, 1.00, 189.0, 0.0, 0.12, 0.11, 0.0, 0.0, 0.0,  5.0, 21.0]),
    fl("Gas Sxo",   "X", "gas",         [0.20, 0.44, 250.0, 0.0, 0.09, 0.02, 0.0, 0.0, 0.0,  3.3,  5.0]),
    fl("Gas Sw",    "U", "gas",         [0.20, 0.44, 250.0, 0.0, 0.09, 0.02, 0.0, 0.0, 0.0,  3.3,  5.0]),
];

pub fn multimin_library() -> Vec<Component> {
    LIB.iter()
        .map(|r| {
            let mut endpoints: HashMap<String, f64> = HashMap::new();
            let [rhob, nphi, dt, gr, pef, u, thor, pota, uran, ept, sigma] = r.v;
            let is_fluid = r.kind == "fluid";
            let vp = if dt > 0.0 { 304.8 / dt } else { 0.0 };
            let vs = if is_fluid { 0.0 } else { vp / 1.7 };
            for (k, val) in [
                ("RHOB", rhob),
                ("NPHI", nphi),
                ("DT", dt),
                ("GR", gr),
                ("PEF", pef),
                ("U", u),
                ("THOR", thor),
                ("POTA", pota),
                ("URAN", uran),
                ("VP", (vp * 100.0).round() / 100.0),
                ("VS", (vs * 100.0).round() / 100.0),
                ("EPT", ept),
                ("EATT", 0.0),
                ("SIGMA", sigma),
            ] {
                endpoints.insert(k.to_string(), val);
            }
            Component {
                name: r.name.to_string(),
                kind: r.kind.to_string(),
                zone: r.zone.to_string(),
                fluid_type: r.fluid_type.to_string(),
                endpoints,
                cec: r.cec,
                max_vol: r.max_vol,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib_get(name: &str) -> Component {
        multimin_library().into_iter().find(|c| c.name == name).unwrap()
    }

    /// Weighted rows for a set of components over plain (non-conductivity) tools.
    fn weighted(comps: &[&Component], keys: &[&str], sigmas: &[f64], meas: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for (t, key) in keys.iter().enumerate() {
            let w = 1.0 / sigmas[t];
            let row: Vec<f64> = comps
                .iter()
                .map(|c| {
                    if c.kind == "fluid" && c.zone == "U" {
                        0.0
                    } else {
                        c.endpoints[&key.to_string()] * w
                    }
                })
                .collect();
            a.push(row);
            b.push(meas[t] * w);
        }
        (a, b)
    }

    fn unity_of(comps: &[&Component]) -> Vec<f64> {
        comps
            .iter()
            .map(|c| if c.kind == "fluid" && c.zone == "X" { 0.0 } else { 1.0 })
            .collect()
    }

    #[test]
    fn recon_qc_emits_per_tool_curves_and_flags_endpoint_error() {
        // Forward-model a 24-sample quartz/illite/water well from the library's own endpoints, so a
        // clean solve reconstructs the logs exactly (incoherence ~0) and a wrong endpoint inflates it.
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-RECON", None, None, None).unwrap();
        let ids = wid.to_string();
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new(); // shared water: in unity + seen by every tool
        let ep = |c: &Component, k: &str| c.endpoints[k];
        let n = 24usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let (mut gr, mut nphi, mut rhob, mut dt) = (vec![], vec![], vec![], vec![]);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let vi = 0.05 + 0.35 * (t * 3.14).sin().powi(2);
            let vw = 0.05 + 0.20 * t;
            let vq = (1.0 - vi - vw).max(0.0);
            let s = vq + vi + vw;
            let (vq, vi, vw) = (vq / s, vi / s, vw / s);
            let mix = |k: &str| vq * ep(&q, k) + vi * ep(&ill, k) + vw * ep(&wat, k);
            gr.push(mix("GR") as f32);
            nphi.push(mix("NPHI") as f32);
            rhob.push(mix("RHOB") as f32);
            dt.push(mix("DT") as f32);
        }
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), gr, vec![2.0f32; n], nphi, rhob.clone(), dt, vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        let tools = || {
            vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 2.0 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
            ]
        };
        let run = |comps: Vec<Component>, prefix: &str| -> MultiminResult {
            run_multimin(
                &db,
                &MultiminRequest {
                    components: comps,
                    tools: tools(),
                    apply_well_ids: vec![ids.clone()],
                    output_prefix: prefix.into(),
                    unity: true,
                    fluid: None,
                    recon_qc: true,
                    sw_model: SwModel::LinearDw,
                },
                None,
            )
        };
        let read = |names: &[&str]| -> HashMap<String, Vec<f32>> {
            let c = db.lock().unwrap();
            fetch_curve_frame(&c, &ids, &names.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap().1
        };
        let rms = |v: &[f32]| {
            let fin: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            (fin.iter().map(|x| x * x).sum::<f32>() / fin.len().max(1) as f32).sqrt()
        };

        // Clean model: 4 tools + unity − 3 components = 2 DOF, so RECON is a real signal.
        let clean = run(vec![q.clone(), ill.clone(), wat.clone()], "MM");
        assert!(clean.error.is_none(), "err={:?}", clean.error);
        assert_eq!(clean.dof, 2, "dof = 4 tools + unity − 3 comps");
        assert!(clean.dof_note.is_none());
        let mean_clean = clean.wells[0].mean_recon;
        assert!(mean_clean < 0.1, "a perfect forward model should reconstruct exactly, incoherence={mean_clean}");
        for name in ["MM_RHOB_REC", "MM_RHOB_DIF", "MM_NPHI_REC", "MM_DT_REC", "MM_GR_REC"] {
            assert!(clean.outputs.iter().any(|o| o == name), "missing output {name}; got {:?}", clean.outputs);
        }
        // Reconstructed RHOB recovers the measured RHOB.
        let cols = read(&["MM_RHOB_REC"]);
        let maxdiff = cols["MM_RHOB_REC"]
            .iter()
            .zip(&rhob)
            .filter(|(r, _)| r.is_finite())
            .map(|(r, m)| (r - m).abs())
            .fold(0.0f32, f32::max);
        assert!(maxdiff < 0.02, "reconstructed RHOB should match measured, max diff {maxdiff}");

        // Inject a wrong illite density (+0.4 g/cc) → incoherence rises and the density residual is real.
        let mut ill_bad = ill.clone();
        *ill_bad.endpoints.get_mut("RHOB").unwrap() += 0.4;
        let bad = run(vec![q.clone(), ill_bad, wat.clone()], "MB");
        assert!(bad.error.is_none(), "err={:?}", bad.error);
        let mean_bad = bad.wells[0].mean_recon;
        assert!(
            mean_bad > mean_clean * 3.0 + 0.1,
            "an endpoint error should inflate incoherence: clean {mean_clean}, bad {mean_bad}"
        );
        assert!(rms(&read(&["MB_RHOB_DIF"])["MB_RHOB_DIF"]) > 0.1, "the density misfit should show in MB_RHOB_DIF");
    }

    #[test]
    fn dof_note_set_when_exactly_determined() {
        // 3 components, 2 tools + unity = 3 equations → dof 0 → a note, and RECON ~0 regardless.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-DOF", None, None, None).unwrap();
        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn, wid, depth, vec![40.0; n], vec![2.0; n], vec![0.2; n], vec![2.45; n], vec![80.0; n], vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let res = run_multimin(
            &db,
            &MultiminRequest {
                components: vec![q, ill, wat],
                tools: vec![
                    ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                    ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ],
                apply_well_ids: vec![wid.to_string()],
                output_prefix: "MM".into(),
                unity: true,
                fluid: None,
                recon_qc: false,
                sw_model: SwModel::LinearDw,
            },
            None,
        );
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert_eq!(res.dof, 0);
        assert!(res.dof_note.is_some(), "exactly-determined model should carry a dof note");
    }

    #[test]
    fn recovers_known_three_mineral_mix() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        // Zone-less variant of water so it appears in unity and all tools.
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let truth = [0.55, 0.15, 0.30];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k.to_string()] * v).sum::<f64>())
            .collect();
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &meas);
        let hi = vec![1.0; 3];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.02, "got {v:?}, want {truth:?}");
        }
    }

    #[test]
    fn unity_is_exact_and_bounds_hold() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &[2.40, 0.20, 80.0, 60.0]);
        let hi = vec![1.0, 1.0, 0.25]; // cap water below its natural solution
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        let sum: f64 = v.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "unity violated: {sum}");
        assert!(v[2] <= 0.25 + 1e-9, "upper bound violated: {v:?}");
        assert!(v.iter().all(|&x| x >= -1e-9), "lower bound violated: {v:?}");
    }

    #[test]
    fn nonneg_holds_when_truth_is_a_boundary() {
        let (q, ill) = (lib_get("Quartz"), lib_get("Illite"));
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &wat];
        let truth = [0.75, 0.0, 0.25];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k.to_string()] * v).sum::<f64>())
            .collect();
        let sig = [0.03, 0.03, 5.0, 15.0];
        let (a, b) = weighted(&comps, &keys, &sig, &meas);
        let hi = vec![1.0; 3];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, 3).unwrap();
        assert!(v[1] >= -1e-9 && v[1] < 0.02, "illite should be ~0, got {}", v[1]);
        assert!((v[0] - 0.75).abs() < 0.02 && (v[2] - 0.25).abs() < 0.02, "got {v:?}");
    }

    #[test]
    fn xu_split_recovers_sw_and_sxo_from_conductivity() {
        // Shaly sand with invasion: quartz 0.55, illite 0.15, porosity 0.30.
        // U zone: Sw = 0.40 → water_sw 0.12, oil_sw 0.18.
        // X zone: Sxo = 0.80 → water_sxo 0.24, oil_sxo 0.06.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let wx = lib_get("Water Sxo");
        let wu = lib_get("Water Sw");
        let ox = lib_get("Oil Sxo");
        let ou = lib_get("Oil Sw");
        let comps = [&q, &ill, &wx, &wu, &ox, &ou];
        let n = comps.len();
        let truth = [0.55, 0.15, 0.24, 0.12, 0.06, 0.18];

        let props = FluidProps {
            rw: 0.43,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let fc = fluid_calc(&props);
        let inv_w = 1.0 / fc.w;

        // Nuclear tools (X-zone fluids visible, U-zone fluids invisible).
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.0264, 0.014, 1.951, 6.0];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k| {
                comps
                    .iter()
                    .zip(truth)
                    .map(|(c, v)| if c.zone == "U" { 0.0 } else { c.endpoints[&k.to_string()] * v })
                    .sum::<f64>()
            })
            .collect();
        let (mut a, mut b) = weighted(&comps, &keys, &sig, &meas);

        // CT row (U-zone water) and CXO row (X-zone water), dual-water linear transform.
        let ct_t = fc.cw.powf(inv_w) * truth[3]; // only water_sw conducts
        let cxo_t = fc.cmf.powf(inv_w) * truth[2];
        let wct = 1.0 / fc.u_ct;
        let wcxo = 1.0 / fc.u_cxo;
        let mut ct_row = vec![0.0; n];
        ct_row[3] = fc.cw.powf(inv_w) * wct;
        let mut cxo_row = vec![0.0; n];
        cxo_row[2] = fc.cmf.powf(inv_w) * wcxo;
        a.push(ct_row);
        b.push(ct_t * wct);
        a.push(cxo_row);
        b.push(cxo_t * wcxo);

        // POROSITY soft row: X fluids − U fluids = 0.
        let sw = 1.0 / SIGMA_CONSTRAINT;
        a.push(vec![0.0, 0.0, sw, -sw, sw, -sw]);
        b.push(0.0);

        let unity = unity_of(&comps); // X fluids excluded
        let hi = vec![1.0, 1.0, 0.5, 0.5, 0.5, 0.5];
        let v = solve_bounded_lsq(&a, &b, Some(&unity), &hi, n).unwrap();
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.02, "got {v:?}, want {truth:?}");
        }
        let phie = v[3] + v[5];
        let sw_calc = v[3] / phie;
        let sxo_calc = v[2] / (v[2] + v[4]);
        assert!((sw_calc - 0.40).abs() < 0.05, "Sw {sw_calc}");
        assert!((sxo_calc - 0.80).abs() < 0.05, "Sxo {sxo_calc}");
    }

    #[test]
    fn bound_water_tracks_clay_volume() {
        // BNDWAT soft constraint: v_bw ≈ k·v_illite with k = 96·CEC·ρ/(T°C+298).
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let bw = lib_get("BoundWater");
        let mut wat = lib_get("Water Sxo");
        wat.zone = "".into();
        let comps = [&q, &ill, &bw, &wat];
        let n = comps.len();
        let t_c = (148.0 - 32.0) * 5.0 / 9.0;
        let k = bndwat_multiplier(ill.cec, ill.endpoints["RHOB"], t_c, 1.0);
        assert!((k - 0.184).abs() < 0.01, "reference multiplier: got {k}");

        let truth = [0.60, 0.20, k * 0.20, 1.0 - 0.60 - 0.20 - k * 0.20];
        let keys = ["RHOB", "NPHI", "DT", "GR"];
        let sig = [0.0264, 0.014, 1.951, 6.0];
        let meas: Vec<f64> = keys
            .iter()
            .map(|k2| comps.iter().zip(truth).map(|(c, v)| c.endpoints[&k2.to_string()] * v).sum::<f64>())
            .collect();
        let (mut a, mut b) = weighted(&comps, &keys, &sig, &meas);
        let sw = 1.0 / SIGMA_CONSTRAINT;
        a.push(vec![0.0, k * sw, -sw, 0.0]);
        b.push(0.0);
        let hi = vec![1.0, 1.0, 0.5, 0.5];
        let v = solve_bounded_lsq(&a, &b, Some(&unity_of(&comps)), &hi, n).unwrap();
        assert!((v[2] - k * v[1]).abs() < 0.02, "bound water should track clay: {v:?}");
        for (got, want) in v.iter().zip(truth) {
            assert!((got - want).abs() < 0.03, "got {v:?}, want {truth:?}");
        }
    }

    #[test]
    fn fluid_calc_matches_reference() {
        // reference default-model example: Rw 0.43 @ 77F, Rmf 0.10 @ 62F, FT 148F, m=n=2.
        let props = FluidProps {
            rw: 0.43,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let fc = fluid_calc(&props);
        assert!((fc.w - 2.0).abs() < 1e-9);
        // The reference shows U FreeW CT response 4.298 mho/m and salinity ~13,048 ppm.
        assert!((fc.cw - 4.3).abs() < 0.2, "Cw {}", fc.cw);
        assert!((fc.salinity_w_ppm - 13_048.0).abs() < 800.0, "S {}", fc.salinity_w_ppm);
        assert!(fc.cmf > fc.cw, "filtrate is saltier here: Cmf {} Cw {}", fc.cmf, fc.cw);
        assert!(fc.u_ct > 0.0 && fc.u_cxo > 0.0);
    }

    #[test]
    fn library_has_expected_shape() {
        let lib = multimin_library();
        assert_eq!(lib.len(), 27, "IP mineral-dropdown parity");
        for c in &lib {
            for k in TOOL_KEYS {
                assert!(c.endpoints.contains_key(k), "{} missing {k}", c.name);
            }
        }
        assert_eq!(lib.iter().filter(|c| c.kind == "clay").count(), 6);
        assert_eq!(lib.iter().filter(|c| c.kind == "fluid").count(), 7);
        let ill = lib.iter().find(|c| c.name == "Illite").unwrap();
        assert!((ill.cec - 0.25).abs() < 1e-9);
        let wsxo = lib.iter().find(|c| c.name == "Water Sxo").unwrap();
        assert_eq!(wsxo.zone, "X");
        assert!((wsxo.max_vol - 0.5).abs() < 1e-9);
    }

    #[test]
    fn pef_converts_to_u_before_mixing() {
        // U mixes volumetrically; PEF does not. Use a pyritic sand — pyrite's high ρe
        // makes the two paths diverge sharply (a quartz/calcite pair nearly coincides
        // because their ρe are almost equal). The true U is the volume average, and the
        // PEF a tool would read is U/ρe. Converting that PEF back through rho_e must
        // recover U, while linearly mixing the raw PEF endpoints gives a wrong answer.
        let (q, py) = (lib_get("Quartz"), lib_get("Pyrite"));
        let (vq, vp) = (0.85, 0.15);
        let u_true = vq * q.endpoints["U"] + vp * py.endpoints["U"];
        let rhob = vq * q.endpoints["RHOB"] + vp * py.endpoints["RHOB"];
        let pef_read = u_true / rho_e(rhob);

        let u_back = pef_to_u(pef_read, rhob).unwrap();
        assert!((u_back - u_true).abs() < 1e-9, "U round-trip: {u_back} vs {u_true}");

        let pef_linear = vq * q.endpoints["PEF"] + vp * py.endpoints["PEF"];
        assert!(
            (pef_linear - pef_read).abs() > 0.5,
            "raw-PEF volumetric mixing ({pef_linear:.3}) must differ from the U path ({pef_read:.3})"
        );
        assert!(pef_to_u(5.0, -1.0).is_none(), "non-physical RHOB rejected");
        assert!(pef_to_u(f64::NAN, 2.5).is_none(), "non-finite PEF rejected");
    }

    #[test]
    fn rejects_underdetermined_request() {
        // 4 components under hard unity need at least n−1 = 3 independent tool logs; offering
        // only 2 leaves the system under-determined (a whole subspace of vertex solutions).
        // The solver must refuse the run up front rather than emit arbitrary volumes. The
        // gate fires on request shape alone, before any DB access, so an empty db is fine.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let cal = lib_get("Calcite");
        let mut wat = lib_get("Water Sxo");
        wat.zone = String::new();
        let req = MultiminRequest {
            components: vec![q, ill, cal, wat],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
            ],
            apply_well_ids: vec!["dummy-well".into()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: None,
            recon_qc: false,
            sw_model: SwModel::LinearDw,
        };
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        let res = run_multimin(&conn, &req, None);
        let err = res.error.as_deref().unwrap_or("");
        assert!(
            err.contains("need at least"),
            "expected an under-determination refusal, got {:?}",
            res.error
        );
        assert!(res.wells.is_empty(), "no wells should be processed on a refused request");
    }

    #[test]
    fn rejects_all_zero_conductivity_row() {
        // CT reads U-zone (deep) water, but the model's only water is X-zone (Water Sxo), so
        // the CT response row is all zero — a bogus 0 = Ct^(1/w) equation. The whole-model
        // no-water guard passes (X water exists), so this per-zone case must be caught here.
        let q = lib_get("Quartz");
        let ill = lib_get("Illite");
        let wx = lib_get("Water Sxo"); // zone X only — nothing in the U (deep) zone
        let props = FluidProps {
            rw: 0.3,
            rw_temp_f: 77.0,
            rmf: 0.1,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let req = MultiminRequest {
            components: vec![q, ill, wx],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.03 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.03 },
                ToolSpec { key: "CT".into(), curve: "RT".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec!["dummy-well".into()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            recon_qc: false,
            sw_model: SwModel::LinearDw,
        };
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        let res = run_multimin(&conn, &req, None);
        let err = res.error.as_deref().unwrap_or("");
        assert!(
            err.contains("all zero"),
            "expected an all-zero conductivity-row refusal, got {:?}",
            res.error
        );
    }

    /// End-to-end smoke test for both reference fixes, driven through the actual DB path
    /// (fetch_curve_frame → run_multimin with a PEF tool → write; and run_module vsh_dn
    /// with GR). #[ignore] so the normal suite never touches it. If SANDIBUMI_E2E_DB
    /// points at a project.duckdb copy with a RHOB+NPHI+GR well, it runs on that REAL
    /// field data; otherwise it seeds a synthetic well through the real schema + write
    /// path so the new run_multimin PEF branch and vsh_dn output are still exercised
    /// against DB-resident curves. Run with:
    ///   [SANDIBUMI_E2E_DB=<copy.duckdb>] cargo test --lib -- \
    ///       --ignored --nocapture e2e_pef_and_vsh_on_real_well
    #[test]
    #[ignore]
    fn e2e_pef_and_vsh_on_real_well() {
        // Best RHOB+NPHI+GR well in a DB, preferring one that also has PEF.
        fn pick_well(db: &Mutex<Connection>, wells: &[crate::db::WellSummary]) -> Option<(String, [usize; 5])> {
            let need: Vec<String> =
                ["RHOB", "NPHI", "DT", "GR", "PEF"].iter().map(|s| s.to_string()).collect();
            let cnt = |cols: &HashMap<String, Vec<f32>>, k: &str| {
                cols.get(k).map(|v| v.iter().filter(|x| x.is_finite()).count()).unwrap_or(0)
            };
            let mut best: Option<(String, [usize; 5], usize)> = None;
            for w in wells {
                let cols = match {
                    let c = db.lock().unwrap();
                    fetch_curve_frame(&c, &w.well_id, &need)
                } {
                    Ok((_d, cols)) => cols,
                    Err(_) => continue,
                };
                let cov = [cnt(&cols, "RHOB"), cnt(&cols, "NPHI"), cnt(&cols, "DT"), cnt(&cols, "GR"), cnt(&cols, "PEF")];
                if cov[0] == 0 || cov[1] == 0 || cov[3] == 0 {
                    continue;
                }
                let score = cov[0] + cov[1] + cov[3] + if cov[4] > 0 { 10_000_000 } else { 0 };
                if best.as_ref().map(|b| score > b.2).unwrap_or(true) {
                    best = Some((w.well_id.clone(), cov, score));
                }
            }
            best.map(|(id, cov, _)| (id, cov))
        }

        // Seed a synthetic well by forward-modeling a quartz/illite/water mix with the
        // library's own endpoints, so run_multimin should recover it near-exactly and the
        // PEF curve (stored as U/ρe, the tool reading) round-trips through the PEF→U path.
        fn build_synthetic() -> (Mutex<Connection>, String, bool) {
            let conn = Connection::open_in_memory().expect("in-memory db");
            crate::db::create_schema(&conn).expect("schema");
            let wid = "11111111-1111-1111-1111-111111111111";
            conn.execute_batch(&format!(
                "INSERT INTO wells (well_id, well_name, field_name) VALUES ('{wid}','SYNTH-1','E2E');"
            ))
            .expect("insert well");
            let lib = multimin_library();
            let ep = |nm: &str, k: &str| lib.iter().find(|c| c.name == nm).unwrap().endpoints[k];
            let (n, top, step) = (300usize, 2000.0f64, 0.5f64);
            // fetch_curve_frame reads gr/res_deep/nphi/rhob as NON-optional f32, so
            // res_deep must be non-null (every real well has resistivity); give it a constant.
            let mut sc = String::from("INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob, dt) VALUES ");
            let mut pf = String::from("INSERT INTO computed_curves (well_id, depth, curve_name, value) VALUES ");
            for i in 0..n {
                let depth = top + i as f64 * step;
                let t = i as f64 / (n - 1) as f64;
                // Smoothly varying, in-bounds mix (water ≤ 0.35, illite ≤ 0.5).
                let vi = 0.05 + 0.40 * (t * 9.42).sin().powi(2);
                let vw = 0.05 + 0.25 * ((t * 6.28).cos() * 0.5 + 0.5);
                let vq = (1.0 - vi - vw).max(0.0);
                let s = vq + vi + vw;
                let (vq, vi, vw) = (vq / s, vi / s, vw / s);
                let mix = |k: &str| vq * ep("Quartz", k) + vi * ep("Illite", k) + vw * ep("Water Sxo", k);
                let (gr, nphi, rhob, dt, u) = (mix("GR"), mix("NPHI"), mix("RHOB"), mix("DT"), mix("U"));
                let pef = u / rho_e(rhob);
                sc += &format!("('{wid}',{depth},{gr},2.0,{nphi},{rhob},{dt}),");
                pf += &format!("('{wid}',{depth},'PEF',{pef}),");
            }
            sc.pop();
            sc.push(';');
            pf.pop();
            pf.push(';');
            conn.execute_batch(&sc).expect("insert standard_curves");
            conn.execute_batch(&pf).expect("insert PEF");
            (Mutex::new(conn), wid.to_string(), true)
        }

        let mut synthetic = false;
        let picked = std::env::var("SANDIBUMI_E2E_DB").ok().and_then(|path| match Connection::open(&path) {
            Ok(conn) => {
                let db = Mutex::new(conn);
                let wells = { let c = db.lock().unwrap(); crate::db::list_wells(&c).unwrap_or_default() };
                match pick_well(&db, &wells) {
                    Some((wid, cov)) => {
                        eprintln!("using REAL db {path}: {} wells; well {wid} cov RHOB/NPHI/DT/GR/PEF = {cov:?}", wells.len());
                        Some((db, wid, cov[4] > 0))
                    }
                    None => {
                        eprintln!("real db {path} opened but has no RHOB+NPHI+GR well — using synthetic");
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("could not open SANDIBUMI_E2E_DB ({e}) — using synthetic");
                None
            }
        });
        let (db, well_id, has_pef) = picked.unwrap_or_else(|| {
            synthetic = true;
            let s = build_synthetic();
            eprintln!("using SYNTHETIC forward-modeled well {} (300 samples, quartz/illite/water)", s.1);
            s
        });

        {
            let c = db.lock().unwrap();
            let raw: i64 = c
                .query_row("SELECT count(*) FROM standard_curves WHERE well_id = ?1", duckdb::params![well_id], |r| r.get(0))
                .unwrap_or(-1);
            let (nd, nr, np) = fetch_curve_frame(&c, &well_id, &["RHOB".into(), "PEF".into()])
                .map(|(d, cols)| {
                    let fin = |k: &str| cols.get(k).map(|v| v.iter().filter(|x| x.is_finite()).count()).unwrap_or(0);
                    (d.len(), fin("RHOB"), fin("PEF"))
                })
                .unwrap_or((999_999, 0, 0));
            eprintln!("debug: standard_curves rows={raw}; fetch depths={nd} finiteRHOB={nr} finitePEF={np}");
        }

        // ---- Fix 1: multimin with a PEF tool (converted to U per sample) ----
        let lib = multimin_library();
        let get = |nm: &str| lib.iter().find(|c| c.name == nm).cloned().unwrap();
        let q = get("Quartz");
        let ill = get("Illite");
        let mut wat = get("Water Sxo");
        wat.zone = String::new(); // shared water: enters unity + seen by all tools
        let comps = vec![q, ill, wat];

        let base_tools = || {
            vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
            ]
        };
        let run = |prefix: &str, with_pef: bool| -> MultiminWellResult {
            let mut tools = base_tools();
            if with_pef {
                tools.push(ToolSpec { key: "PEF".into(), curve: "PEF".into(), sigma: 0.3 });
            }
            let req = MultiminRequest {
                components: comps.clone(),
                tools,
                apply_well_ids: vec![well_id.clone()],
                output_prefix: prefix.into(),
                unity: true,
                fluid: None,
                recon_qc: false,
                sw_model: SwModel::LinearDw,
            };
            let res = run_multimin(&db, &req, None);
            assert!(res.error.is_none(), "run_multimin error: {:?}", res.error);
            res.wells.into_iter().next().unwrap()
        };

        let no_pef = run("MMNOPEF", false);
        eprintln!(
            "multimin (no PEF): solved={} mean_recon={:.4} err={:?}",
            no_pef.rows_solved, no_pef.mean_recon, no_pef.error
        );
        assert!(no_pef.rows_solved > 0, "no samples solved without PEF");

        if has_pef {
            let with_pef = run("MMPEF", true);
            eprintln!(
                "multimin (+PEF→U): solved={} mean_recon={:.4}",
                with_pef.rows_solved, with_pef.mean_recon
            );
            assert!(with_pef.rows_solved > 0, "PEF run solved no samples");
            assert!(with_pef.mean_recon.is_finite(), "PEF run recon not finite");
            if synthetic {
                // PEF was stored as U_true/ρe; the PEF→U path must reconstruct U_true,
                // so the converted row fits the true volumes and RECON stays ~0. A wrong
                // (raw-PEF) mix would leave this row inconsistent and inflate RECON.
                assert!(
                    with_pef.mean_recon < 0.05,
                    "synthetic PEF→U row should fit near-perfectly; recon={}",
                    with_pef.mean_recon
                );
            }
            // Hard-unity guarantees Σvol=1; verify on the written curves.
            let cols = {
                let c = db.lock().unwrap();
                fetch_curve_frame(
                    &c,
                    &well_id,
                    &["VOL_QUARTZ".into(), "VOL_ILLITE".into(), "VOL_WATER_SXO".into()],
                )
                .unwrap()
                .1
            };
            let (mut n_ok, mut sum_err_max) = (0usize, 0.0f32);
            for i in 0..cols["VOL_QUARTZ"].len() {
                let s = cols["VOL_QUARTZ"][i] + cols["VOL_ILLITE"][i] + cols["VOL_WATER_SXO"][i];
                if s.is_finite() {
                    n_ok += 1;
                    sum_err_max = sum_err_max.max((s - 1.0).abs());
                }
            }
            eprintln!("  VOL rows summing to 1: {n_ok}, max |Σvol−1| = {sum_err_max:.2e}");
            assert!(n_ok > 0 && sum_err_max < 1e-3, "unity violated on written VOL curves");
        }

        // ---- Fix 2: vsh_dn clay-type guard on real GR ----
        let (depth, cols) = {
            let c = db.lock().unwrap();
            fetch_curve_frame(&c, &well_id, &["RHOB".into(), "NPHI".into(), "GR".into()]).unwrap()
        };
        let nsamp = depth.len();
        let logs: HashMap<String, Vec<f32>> = ["RHOB", "NPHI", "GR"]
            .iter()
            .map(|k| (k.to_string(), cols[*k].clone()))
            .collect();
        let params: HashMap<String, Vec<f64>> = [
            ("RHO_MA", 2.645),
            ("RHO_SH", 2.5),
            ("RHO_FL", 1.0),
            ("NPHI_MA", -0.02),
            ("NPHI_SH", 0.35),
            ("NPHI_FL", 1.0),
            ("GR_MA", 15.0),
            ("GR_SH", 120.0),
            ("FLAG_TOL", 0.25),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), vec![*v; nsamp]))
        .collect();
        let ctx = crate::modules::ModuleContext { n: nsamp, logs, params, opts: HashMap::new() };
        let out = crate::modules::run_module("vsh_dn", &ctx).expect("vsh_dn run");
        let flag = &out["VSH_DN_FLAG"];
        let vsh = &out["VSH"];
        let (mut n_flag, mut n_eval, mut vsh_min, mut vsh_max) = (0usize, 0usize, f32::MAX, f32::MIN);
        for i in 0..nsamp {
            if flag[i].is_finite() {
                n_eval += 1;
                if flag[i] == 1.0 {
                    n_flag += 1;
                }
                vsh_min = vsh_min.min(vsh[i]);
                vsh_max = vsh_max.max(vsh[i]);
            }
        }
        eprintln!(
            "vsh_dn: evaluated={n_eval} flagged={n_flag} ({:.1}%), VSH∈[{vsh_min:.3},{vsh_max:.3}]",
            100.0 * n_flag as f32 / n_eval.max(1) as f32
        );
        assert!(n_eval > 0, "vsh_dn produced no evaluated samples");
        assert!(vsh_min >= 0.0 && vsh_max <= 1.0, "limited VSH out of [0,1]");
    }

    #[test]
    fn dry_clay_matches_the_kkt_example() {
        // Multimin Parameters.xlsx, KK-1 Post Main: dry clay 2.70, wet 2.18333,
        // wet NPHI 0.489583, wet GR 110 -> phi_clay 0.3039, NPHI 0.2667, GR 158.0.
        let dc = dry_clay_calc(&WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        })
        .unwrap();
        assert!((dc.phi_clay - 0.303922).abs() < 1e-5, "phi_clay {}", dc.phi_clay);
        assert!((dc.nphi_dry - 0.2667).abs() < 3e-4, "nphi_dry {}", dc.nphi_dry);
        assert!((dc.gr_dry - 158.0).abs() < 0.1, "gr_dry {}", dc.gr_dry);
        assert_eq!(dc.rhob_dry, 2.70);
        assert!(dc.dt_dry.is_none(), "no wet DT picked -> no dry DT");
        assert!((dc.cbw_ratio - dc.phi_clay / (1.0 - dc.phi_clay)).abs() < 1e-12);
    }

    #[test]
    fn dry_clay_cec_reproduces_the_bndwat_tie() {
        // The equivalent CEC must make the solver's BNDWAT multiplier equal the
        // phi_clay ratio at the SAME fluid conditions (T and alpha_u), and the DT
        // conversion must follow the 189 us/ft water line.
        let fluid = FluidProps {
            rw: 0.305,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 1.86,
            n: 1.78,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let dc = dry_clay_calc(&WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: Some(110.0),
            rho_dry: 2.70,
            fluid: Some(fluid.clone()),
        })
        .unwrap();
        let fc = fluid_calc(&fluid);
        let t_c = (fluid.ftemp_f - 32.0) * 5.0 / 9.0;
        let k = bndwat_multiplier(dc.cec_equiv, dc.rhob_dry, t_c, fc.alpha_u);
        assert!((k - dc.cbw_ratio).abs() < 1e-9, "k {} vs ratio {}", k, dc.cbw_ratio);
        let phi = dc.phi_clay;
        let want_dt = (110.0 - 189.0 * phi) / (1.0 - phi);
        assert!((dc.dt_dry.unwrap() - want_dt).abs() < 1e-9);
    }

    #[test]
    fn dry_clay_rejects_degenerate_densities() {
        let base = WetClayInput {
            rhob_wet: 2.18,
            nphi_wet: 0.49,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        };
        let wet_too_light = WetClayInput { rhob_wet: 0.9, ..base.clone() };
        assert!(dry_clay_calc(&wet_too_light).is_err(), "wet RHOB below water density");
        let dry_below_wet = WetClayInput { rho_dry: 2.10, ..base.clone() };
        assert!(dry_clay_calc(&dry_below_wet).is_err(), "dry density below wet reading");
        assert!(dry_clay_calc(&base).is_ok());
    }

    #[test]
    fn dry_clay_rejects_unphysical_picks() {
        // percent-entry habit, blank-field zero coercion, and a wet DT
        // below the 189·φ water term must all error instead of producing
        // negative dry endpoints silently.
        let base = WetClayInput {
            rhob_wet: 2.18333,
            nphi_wet: 0.489583,
            gr_wet: 110.0,
            dt_wet: None,
            rho_dry: 2.70,
            fluid: None,
        };
        let pct = WetClayInput { nphi_wet: 48.9583, ..base.clone() };
        assert!(dry_clay_calc(&pct).is_err(), "percent NPHI entry");
        let blank_nphi = WetClayInput { nphi_wet: 0.0, ..base.clone() };
        assert!(dry_clay_calc(&blank_nphi).is_err(), "blank NPHI coerced to 0");
        let blank_gr = WetClayInput { gr_wet: 0.0, ..base.clone() };
        assert!(dry_clay_calc(&blank_gr).is_err(), "blank GR coerced to 0");
        // phi_clay ~0.3039 -> water term 189*phi ~57.4 us/ft.
        let dt_low = WetClayInput { dt_wet: Some(50.0), ..base.clone() };
        assert!(dry_clay_calc(&dt_low).is_err(), "DT below the water term");
        let dt_ok = WetClayInput { dt_wet: Some(110.0), ..base.clone() };
        assert!(dry_clay_calc(&dt_ok).is_ok());
    }

    // --- Saturation models (Jauhar's Sw-equation request) --------------------

    #[test]
    fn sw_indonesia_round_trips() {
        // Forward-model Rt from a known Sw via the Indonesia equation, then recover it.
        let (phie, vsh, rw, rsh, m, n, a): (f64, f64, f64, f64, f64, f64, f64) =
            (0.22, 0.18, 0.08, 3.5, 2.0, 2.0, 1.0);
        let d = vsh.powf(1.0 - vsh / 2.0) / rsh.sqrt() + (phie.powf(m) / (a * rw)).sqrt();
        for sw_true in [0.15f64, 0.35, 0.55, 0.8, 1.0] {
            let rt = 1.0 / (d * d * sw_true.powf(n)); // 1/Rt = D²·Sw^n
            let sw = sw_indonesia(rt, phie, vsh, rw, rsh, m, n, a);
            assert!((sw - sw_true).abs() < 1e-6, "Indonesia round-trip: got {sw}, want {sw_true}");
        }
        // A non-2 saturation exponent inverts exactly too (Sw^(n/2) is isolated in closed form).
        let (n2, sw_true): (f64, f64) = (1.8, 0.4);
        let rt = 1.0 / (d * d * sw_true.powf(n2));
        assert!((sw_indonesia(rt, phie, vsh, rw, rsh, m, n2, a) - sw_true).abs() < 1e-6);
    }

    #[test]
    fn sw_simandoux_round_trips() {
        // Forward-model Rt from a known Sw via modified Simandoux, then recover it — exercising
        // both the n==2 quadratic path and the general-n bisection path.
        let (phie, vsh, rw, rsh, m, a): (f64, f64, f64, f64, f64, f64) = (0.25, 0.2, 0.06, 4.0, 2.0, 1.0);
        for &n in &[2.0f64, 1.7, 2.3] {
            for sw_true in [0.2f64, 0.45, 0.7, 0.95] {
                let ct = phie.powf(m) * sw_true.powf(n) / (a * rw * (1.0 - vsh)) + vsh * sw_true / rsh;
                let sw = sw_simandoux(1.0 / ct, phie, vsh, rw, rsh, m, n, a);
                assert!((sw - sw_true).abs() < 1e-4, "Simandoux n={n} round-trip: got {sw}, want {sw_true}");
            }
        }
    }

    #[test]
    fn sw_equations_reject_nonphysical_inputs() {
        assert!(sw_indonesia(-1.0, 0.2, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0).is_nan(), "Rt<=0");
        assert!(sw_indonesia(10.0, 0.0, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0).is_nan(), "phie<=0");
        assert!(sw_simandoux(-1.0, 0.2, 0.1, 0.1, 4.0, 2.0, 2.0, 1.0).is_nan(), "Rt<=0");
        assert!(sw_simandoux(10.0, 0.2, 0.1, 0.0, 4.0, 2.0, 2.0, 1.0).is_nan(), "Rw<=0");
        // A very conductive Rt (fresh, high-φ) clamps Sw to 1, never above.
        assert!((sw_indonesia(0.01, 0.3, 0.0, 0.1, 4.0, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
        assert!((sw_simandoux(0.01, 0.3, 0.0, 0.1, 4.0, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sw_equations_match_hand_computed_points() {
        // INDEPENDENT of the round-trip tests: the expected Sw values are hand-computed NUMERIC
        // LITERALS (not built from the functions' own expressions), so a shale-term or exponent
        // transcription error would fail here instead of being confirmed by a self-referential forward.

        // Clean sand (Vsh=0, m=n=2, a=1) reduces to Archie: Sw² = Rw/(φ²·Rt).
        // φ=0.2, Rw=0.05, Rt=20 ⇒ Sw² = 0.05/(0.04·20) = 0.0625 ⇒ Sw = 0.25 (exact by hand).
        assert!((sw_indonesia(20.0, 0.2, 0.0, 0.05, 4.0, 2.0, 2.0, 1.0) - 0.25).abs() < 1e-6, "Indonesia→Archie");
        assert!((sw_simandoux(20.0, 0.2, 0.0, 0.05, 4.0, 2.0, 2.0, 1.0) - 0.25).abs() < 1e-6, "Simandoux→Archie");

        // Indonesia WITH shale (exercises Vsh^(1−Vsh/2)/√Rsh): Vsh=0.5, Rsh=4, φ=0.2, Rw=0.1, m=n=2.
        //   term_sh = 0.5^0.75/2 = 0.297302 ; term_sand = √(0.04/0.1) = 0.632456 ; denom = 0.929758
        //   Sw=0.4 ⇒ 1/√Rt = 0.929758·0.4 = 0.371903 ⇒ Rt = 7.230045 (hand-computed).
        assert!((sw_indonesia(7.230045, 0.2, 0.5, 0.1, 4.0, 2.0, 2.0, 1.0) - 0.4).abs() < 1e-3, "Indonesia shale point");

        // Modified Simandoux WITH shale (exercises the Vsh·Sw/Rsh term): Vsh=0.4, Rsh=3, φ=0.25,
        // Rw=0.08, m=n=2. coef_sand=0.0625/0.048=1.302083 ; coef_sh=0.133333 ; Sw=0.5 ⇒
        //   1/Rt = 1.302083·0.25 + 0.133333·0.5 = 0.392188 ⇒ Rt = 2.549795 (hand-computed).
        assert!((sw_simandoux(2.549795, 0.25, 0.4, 0.08, 3.0, 2.0, 2.0, 1.0) - 0.5).abs() < 1e-3, "Simandoux shale point");
    }

    #[test]
    fn indonesia_post_solve_recovers_known_sw() {
        // Full X/U model: the nuclear tools see the flushed (X) fluids and fix φe; the deep
        // resistivity is forward-modelled from a known deep Sw via Indonesia (Vsh=0 ⇒ Archie). CT
        // stays in the inversion (keeping the U-split well-posed); the Indonesia model then post-solve
        // OVERRIDES SWE = Sw, leaving PHIE (= 1 − quartz) untouched.
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15); // flushed Sxo = 0.5
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10); // deep Sw = 0.35; Rw at formation T
        let d = (phie.powf(2.0) / (1.0 * rw)).sqrt(); // Vsh=0 ⇒ Indonesia = Archie
        let rt = 1.0 / (d * d * sw_true.powf(2.0));

        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-IND", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); n],
            vec![rt as f32; n],
            vec![mix("NPHI"); n],
            vec![mix("RHOB"); n],
            vec![mix("DT"); n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);

        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let req = MultiminRequest {
            components: vec![q, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            recon_qc: false,
            sw_model: SwModel::Indonesia,
        };
        let res = run_multimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into()]).unwrap().1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }

    #[test]
    fn sw_dual_nonlinear_hand_computed_and_conversion() {
        // Hand-computed NUMERIC LITERAL (independent of the function's own expression) — a coefficient
        // or exponent slip fails here rather than being confirmed self-referentially.
        // φt=0.3, Swb=0.2, Cw=2, Cwb=5, m=n=2, a=1, SWT_true=0.6:
        //   CT = (φt^m·SWT^n/a)·(Cw + (Cwb−Cw)·Swb/SWT) = (0.09·0.36)·(2 + 3·0.2/0.6) = 0.0324·3 = 0.0972
        //   ⇒ Rt = 1/0.0972 = 10.2880658…
        let swt = sw_dual_nonlinear(10.2880658436, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0);
        assert!((swt - 0.6).abs() < 1e-4, "dual-water SWT {swt}, want 0.6");

        // The effective conversion the run path applies: v_bw = Swb·φt = 0.06 ⇒ φe = 0.24;
        // free water = (SWT−Swb)·φt = 0.4·0.3 = 0.12 ⇒ SWE = 0.12/0.24 = 0.5 (hand-computed).
        let (phit, v_bw): (f64, f64) = (0.3, 0.06);
        let swe = ((swt * phit - v_bw) / (phit - v_bw)).clamp(0.0, 1.0);
        assert!((swe - 0.5).abs() < 1e-3, "dual-water SWE {swe}, want 0.5");

        // General n exercises the bisection branch: forward-model CT, invert, recover SWT.
        let (m, n, a, cw, cwb): (f64, f64, f64, f64, f64) = (1.8, 2.3, 1.0, 1.5, 4.0);
        let (phit2, swb2, swt2): (f64, f64, f64) = (0.28, 0.15, 0.45);
        let ct = (phit2.powf(m) * swt2.powf(n) / a) * (cw + (cwb - cw) * swb2 / swt2);
        let back = sw_dual_nonlinear(1.0 / ct, phit2, swb2, cw, cwb, m, n, a);
        assert!((back - swt2).abs() < 1e-3, "dual-water general-n round trip {back}, want {swt2}");

        // High conductivity (very low Rt) saturates to SWT = 1; non-physical inputs → NaN.
        assert!((sw_dual_nonlinear(0.01, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(sw_dual_nonlinear(-1.0, 0.3, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_dual_nonlinear(10.0, 0.0, 0.2, 2.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_dual_nonlinear(10.0, 0.3, 0.2, 0.0, 5.0, 2.0, 2.0, 1.0).is_nan());
        // Sub-linear n<1 (non-physical; would diverge at Swt→0 and silently zero SWE) → NaN, not 0.
        assert!(sw_dual_nonlinear(4.63, 0.3, 0.2, 2.0, 5.0, 2.0, 0.5, 1.0).is_nan(), "n<1 must be rejected");
    }

    #[test]
    fn sw_archie_hand_computed() {
        // Hand-computed literals: Swt = (a·Rw/(φt^m·Rt))^(1/n).
        // φt=0.2, Rw=0.1, m=n=2, a=1 ⇒ Swt²=0.1/(0.04·Rt); Rt=10 ⇒ Swt²=0.25 ⇒ Swt=0.5.
        assert!((sw_archie(10.0, 0.2, 0.1, 2.0, 2.0, 1.0) - 0.5).abs() < 1e-9, "Archie n=2");
        // n=3: Swt³=0.1/(0.04·Rt); Rt=20 ⇒ Swt³=0.125 ⇒ Swt=0.5.
        assert!((sw_archie(20.0, 0.2, 0.1, 2.0, 3.0, 1.0) - 0.5).abs() < 1e-9, "Archie n=3");
        // Clean 100% water (very low Rt) clamps to 1; non-physical inputs → NaN.
        assert!((sw_archie(0.01, 0.2, 0.1, 2.0, 2.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(sw_archie(-1.0, 0.2, 0.1, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_archie(10.0, 0.0, 0.1, 2.0, 2.0, 1.0).is_nan());
        assert!(sw_archie(10.0, 0.2, 0.0, 2.0, 2.0, 1.0).is_nan());
        // Archie ≡ Indonesia with Vsh=0 (both reduce to the clean-sand power law).
        let arch = sw_archie(15.0, 0.22, 0.08, 2.0, 2.0, 1.0);
        let indo0 = sw_indonesia(15.0, 0.22, 0.0, 0.08, 4.0, 2.0, 2.0, 1.0);
        assert!((arch - indo0).abs() < 1e-9, "Archie vs Indonesia(Vsh=0): {arch} vs {indo0}");
    }

    #[test]
    fn dual_water_nonlinear_post_solve_recovers_known_sw() {
        // Same forward model as indonesia_post_solve_recovers_known_sw but sw_model = DualWaterNonlinear.
        // With no clay/bound-water component Swb=0, so the dual-water form reduces to Archie; a deep Sw
        // forward-modelled through Archie must come back as SWE, PHIE untouched. Exercises the run-path
        // wiring (post-solve gate, CT-stays-in inversion, φe redistribution) for the new model.
        let q = lib_get("Quartz");
        let wsxo = lib_get("Water Sxo");
        let osxo = lib_get("Oil Sxo");
        let wsw = lib_get("Water Sw");
        let osw = lib_get("Oil Sw");
        let ep = |c: &Component, k: &str| c.endpoints[&k.to_string()];
        let (vq, vwx, vox) = (0.70, 0.15, 0.15);
        let (phie, sw_true, rw): (f64, f64, f64) = (0.30, 0.35, 0.10);
        let d = (phie.powf(2.0) / (1.0 * rw)).sqrt();
        let rt = 1.0 / (d * d * sw_true.powf(2.0));

        let n = 6usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let mix = |k: &str| (vq * ep(&q, k) + vwx * ep(&wsxo, k) + vox * ep(&osxo, k)) as f32;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "MM-DWNL", None, None, None).unwrap();
        crate::db::insert_standard_curves(
            &conn,
            wid,
            depth,
            vec![mix("GR"); n],
            vec![rt as f32; n],
            vec![mix("NPHI"); n],
            vec![mix("RHOB"); n],
            vec![mix("DT"); n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let db = Mutex::new(conn);
        let props = FluidProps {
            rw,
            rw_temp_f: 100.0,
            rmf: 0.1,
            rmf_temp_f: 100.0,
            ftemp_f: 100.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
            rsh: 4.0,
            archie_a: 1.0,
        };
        let req = MultiminRequest {
            components: vec![q, wsxo, osxo, wsw, osw],
            tools: vec![
                ToolSpec { key: "RHOB".into(), curve: "RHOB".into(), sigma: 0.0264 },
                ToolSpec { key: "NPHI".into(), curve: "NPHI".into(), sigma: 0.014 },
                ToolSpec { key: "DT".into(), curve: "DT".into(), sigma: 1.951 },
                ToolSpec { key: "GR".into(), curve: "GR".into(), sigma: 6.0 },
                ToolSpec { key: "CT".into(), curve: "RES_DEEP".into(), sigma: 0.0 },
            ],
            apply_well_ids: vec![wid.to_string()],
            output_prefix: "MM".into(),
            unity: true,
            fluid: Some(props),
            recon_qc: false,
            sw_model: SwModel::DualWaterNonlinear,
        };
        let res = run_multimin(&db, &req, None);
        assert!(res.error.is_none(), "err={:?}", res.error);
        assert!(res.wells[0].rows_solved > 0, "no samples solved");
        let c = db.lock().unwrap();
        let cols = fetch_curve_frame(&c, &wid.to_string(), &["MM_SWE".into(), "MM_PHIE".into()]).unwrap().1;
        let mean = |v: &[f32]| {
            let f: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            assert!(!f.is_empty(), "no finite samples");
            f.iter().sum::<f32>() / f.len() as f32
        };
        let swe = mean(&cols["MM_SWE"]);
        let phie_out = mean(&cols["MM_PHIE"]);
        assert!((swe - sw_true as f32).abs() < 0.02, "post-solve SWE {swe}, want {sw_true}");
        assert!((phie_out - phie as f32).abs() < 0.02, "PHIE {phie_out}, want {phie}");
    }
}
