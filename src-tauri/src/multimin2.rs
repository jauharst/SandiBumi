//! Generalized Multimin — user-defined multi-mineral / fluid optimizer, modeled on
//! Geolog Multimin and IP's Mineral Solver (spec extracted from both installs, see
//! docs/multimin_geolog_spec.md and docs/multimin_ip_spec.md).
//!
//! Formulation (Geolog convention):
//! - One volume vector per depth frame: minerals + clays (common to both zones) plus
//!   flushed-zone (X / Sxo) and unflushed-zone (U / Sw) fluid sets.
//! - Every tool responds to X-zone fluids except the deep conductivity CT, which sees
//!   the U zone; CXO (flushed conductivity) sees the X zone.
//! - Resistivity enters as CONDUCTIVITY via the DUAL WATER LINEAR transform: with
//!   w = 0.75·m + 0.25·n the response row  Ct^(1/w) = Σ v_i · C_i^(1/w)  is linear in
//!   the volumes (C_i = fluid conductivity endpoints; minerals/hydrocarbons are 0).
//!   The measured resistivity curve is converted (C = 1/R) and transformed before entering
//!   the system. Row uncertainty (Geolog): U_ct = 0.03·Cw^(1/w), U_cxo = 0.03·Cmf^(1/w).
//! - Hard constraints: Σ(minerals + clays + U-zone fluids) = 1 (UNITY, X fluids excluded)
//!   and per-component box bounds 0 ≤ v ≤ max_vol (fluids default 0.5).
//! - Soft "Tool" constraints at σ = 0.01 (Geolog treats these as pseudo-measurements):
//!   POROSITY (Σ X fluids = Σ U fluids) and BNDWAT (bound water tied to clay volumes via
//!   k = 96·CEC·ρ_clay / (T°C + 298) · α, the Dual-Water clay-bound-water multiplier).
//!   WATER MUD (Sxo ≥ Sw for water-based mud) is enforced by a re-solve when violated.
//!
//! The solver is a bounded, equality-constrained active-set least squares (KKT system with
//! a unity Lagrange multiplier; components can be fixed at 0 or at their upper bound).
//! RECON (RMS weighted residual over the live tool rows, σ units) flags model failure.

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
    /// Upper volume bound (Geolog default: 1.0 minerals, 0.5 fluids).
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
}

fn default_mud() -> String {
    "WATER".into()
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
    pub error: Option<String>,
}

fn fail(msg: &str) -> MultiminResult {
    MultiminResult { outputs: vec![], wells: vec![], error: Some(msg.to_string()) }
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
// Fluid property calculations (Geolog RF04 §5 / IP formulas)
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

/// Clay-bound-water multiplier k so that v_bw = k · v_dryclay (Geolog RF04 5.03):
/// k = α · 96 · CEC[meq/g] · ρ_clay[g/cc] / (T°C + 298).
fn bndwat_multiplier(cec: f64, rho_gcc: f64, t_c: f64, alpha: f64) -> f64 {
    alpha * 96.0 * cec * rho_gcc / (t_c + 298.0)
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

const SIGMA_CONSTRAINT: f64 = 0.01; // Geolog's nominal Tool-constraint uncertainty

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

pub fn run_multimin(db: &Mutex<Connection>, req: &MultiminRequest) -> MultiminResult {
    let n = req.components.len();
    if n < 2 {
        return fail("select at least two components");
    }
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

    // Static per-tool data: weights, endpoint rows (conductivity rows built from fluid calc).
    let t_c = req.fluid.as_ref().map(|p| (p.ftemp_f - 32.0) * 5.0 / 9.0).unwrap_or(25.0);
    let mut weights: Vec<f64> = Vec::with_capacity(tools.len());
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(tools.len());
    let mut cond_w: Vec<Option<f64>> = Vec::with_capacity(tools.len()); // Some(w) → transform measurement
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
            let auto_sigma = if key == "CT" { fc.u_ct } else { fc.u_cxo };
            let sigma = if t.sigma > 0.0 { t.sigma } else { auto_sigma };
            weights.push(1.0 / sigma.max(1e-9));
            rows.push(row);
            cond_w.push(Some(fc.w));
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
            cond_w.push(None);
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

    let comp_tokens: Vec<String> = req.components.iter().map(|c| curve_token(&c.name)).collect();
    let vol_names: Vec<String> = comp_tokens.iter().map(|t| format!("VOL_{t}")).collect();
    let prefix = req.output_prefix.trim();
    let prefix = if prefix.is_empty() { "MM" } else { prefix };

    let fetch_names: Vec<String> = tools.iter().map(|t| t.curve.trim().to_uppercase()).collect();

    let mut out_names: Vec<String> = Vec::new();
    let mut wells: Vec<MultiminWellResult> = Vec::new();

    let conn = db.lock().unwrap();
    for well_id in &req.apply_well_ids {
        let (depth, cols) = match fetch_curve_frame(&conn, well_id, &fetch_names) {
            Ok(v) => v,
            Err(e) => {
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

        let mut vol: Vec<Vec<f32>> = vec![vec![f32::NAN; ns]; n];
        let mut recon = vec![f32::NAN; ns];
        let mut solved = 0usize;
        let mut recon_sum = 0.0f64;

        for i in 0..ns {
            let mut a: Vec<Vec<f64>> = Vec::with_capacity(tools.len() + soft.len());
            let mut b: Vec<f64> = Vec::with_capacity(tools.len() + soft.len());
            let mut live_tools = 0usize;
            for (t, tcol) in tool_cols.iter().enumerate() {
                let mut v = tcol[i] as f64;
                if !v.is_finite() {
                    continue;
                }
                if let Some(w_exp) = cond_w[t] {
                    // Resistivity (ohmm) → conductivity (mho/m) → ^(1/w) transform.
                    if v <= 1e-4 {
                        continue;
                    }
                    v = (1.0 / v).powf(1.0 / w_exp);
                }
                let w = weights[t];
                a.push(rows[t].iter().map(|e| e * w).collect());
                b.push(v * w);
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
        curves.push((format!("{prefix}_RECON"), recon));

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
            }))
            .unwrap_or_default(),
            inputs_json: serde_json::to_string(&req.tools.iter().map(|t| t.curve.as_str()).collect::<Vec<_>>())
                .unwrap_or_default(),
        };
        let write_err = crate::equations::create_log_set(&conn, well_id, &spec)
            .and_then(|(set_id, _)| write_computed_curves_versioned(&conn, well_id, &depth, &refs, &set_id))
            .err()
            .map(|e| e.to_string());
        wells.push(MultiminWellResult {
            well_id: well_id.clone(),
            rows_solved: solved,
            mean_recon: if solved > 0 { (recon_sum / solved as f64) as f32 } else { f32::NAN },
            error: write_err.or_else(|| (solved == 0).then(|| "no solvable samples (too few live input logs)".to_string())),
        });
    }

    MultiminResult { outputs: out_names, wells, error: None }
}

fn is_cond_key(key: &str) -> bool {
    let k = key.trim().to_uppercase();
    k == "CT" || k == "CXO"
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
// Endpoint library — Geolog RF04 §6.2 + IP2018 MINDEF.PAR defaults, merged.
// Display units: RHOB g/cc, NPHI v/v, DT us/ft, GR API, PEF b/e, U b/cc,
// THOR ppm, POTA %, URAN ppm, VP km/s, VS km/s, EPT ns/m, EATT dB/m, SIGMA c.u.
// CT/CXO endpoints are computed from the fluid properties at run time.
// ---------------------------------------------------------------------------

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

/// Merged Geolog/IP default library, in IP's mineral-dropdown order (Jauhar's screenshot).
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
        assert!((k - 0.184).abs() < 0.01, "Geolog reference multiplier: got {k}");

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
    fn fluid_calc_matches_geolog_reference() {
        // Geolog's default-model example: Rw 0.43 @ 77F, Rmf 0.10 @ 62F, FT 148F, m=n=2.
        let props = FluidProps {
            rw: 0.43,
            rw_temp_f: 77.0,
            rmf: 0.10,
            rmf_temp_f: 62.0,
            ftemp_f: 148.0,
            m: 2.0,
            n: 2.0,
            mud_type: "WATER".into(),
        };
        let fc = fluid_calc(&props);
        assert!((fc.w - 2.0).abs() < 1e-9);
        // Geolog shows U FreeW CT response 4.298 mho/m and salinity ~13,048 ppm.
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
}
