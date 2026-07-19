//! Multimin — constrained weighted least-squares mineral/fluid inversion
//! (Geolog PT07 equivalent). A four-component model (SAND, CLAY, WATER, HYDROCARBON)
//! is solved per depth sample against whichever of RHOB/NPHI/DT/PEF are present, with a
//! non-negativity constraint (NNLS, Lawson-Hanson active set) and a heavily-weighted
//! unity equation (sum of volumes = 1) folded in as a soft constraint — the standard
//! optimizer trick that keeps the solve a single NNLS rather than a QP.
//!
//! Each tool equation is scaled by 1/sigma so tools of different magnitude (g/cc vs
//! us/ft) are weighted by their measurement uncertainty. The reconstruction error curve
//! (RMS residual in sigma units over the used tools) is the QC that tells you where the
//! model does not explain the logs.
//!
//! This is the deterministic per-sample solver; `inversion.rs` holds the separate
//! background stochastic-inversion job machinery.

use crate::modules::{log_in, log_out, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

const N: usize = 4; // SAND, CLAY, WATER, HC
const SAND: usize = 0;
const CLAY: usize = 1;
const WATER: usize = 2;
const HC: usize = 3;

pub(crate) fn multimin_spec() -> ModuleSpec {
    ModuleSpec {
        name: "multimin".into(),
        title: "Multimin — Mineral Inversion".into(),
        category: "Saturation".into(),
        doc: "Weighted least-squares inversion for SAND/CLAY/WATER/HYDROCARBON volumes from \
              RHOB, NPHI, DT and PEF (any subset present is used). Non-negative volumes, with a \
              soft unity constraint. Outputs the four volumes plus PHIT_MM (=water+hc), \
              VSH_MM (=clay), SWT_MM (=water/PHIT) and RECON_ERR (RMS log-reconstruction error \
              in sigma units — high where the model fails). Endpoints and per-tool sigmas are \
              parameters so they can be tuned per field/zone."
            .into(),
        args: vec![
            // Endpoint responses per component (matrix rows are built from these).
            param("RHOB_SAND", "Sand grain density", "g/cc", 2.65, 2.0, 3.2),
            param("RHOB_CLAY", "Clay density", "g/cc", 2.55, 2.0, 3.2),
            param("RHOB_WATER", "Water density", "g/cc", 1.0, 0.8, 1.3),
            param("RHOB_HC", "Hydrocarbon density", "g/cc", 0.8, 0.1, 1.1),
            param("NPHI_SAND", "Sand neutron", "v/v", -0.02, -0.15, 0.5),
            param("NPHI_CLAY", "Clay neutron", "v/v", 0.30, 0.0, 0.8),
            param("NPHI_WATER", "Water neutron", "v/v", 1.0, 0.5, 1.2),
            param("NPHI_HC", "Hydrocarbon neutron", "v/v", 0.55, 0.0, 1.2),
            param("DT_SAND", "Sand transit time", "us/ft", 55.5, 40.0, 70.0),
            param("DT_CLAY", "Clay transit time", "us/ft", 90.0, 60.0, 150.0),
            param("DT_WATER", "Water transit time", "us/ft", 189.0, 150.0, 220.0),
            param("DT_HC", "Hydrocarbon transit time", "us/ft", 210.0, 150.0, 260.0),
            param("PEF_SAND", "Sand photoelectric factor", "b/e", 1.81, 1.0, 6.0),
            param("PEF_CLAY", "Clay photoelectric factor", "b/e", 3.10, 1.0, 6.0),
            param("PEF_WATER", "Water photoelectric factor", "b/e", 0.36, 0.0, 2.0),
            param("PEF_HC", "Hydrocarbon photoelectric factor", "b/e", 0.12, 0.0, 2.0),
            // Per-tool measurement sigma (equation weight = 1/sigma).
            param("SIG_RHOB", "RHOB uncertainty", "g/cc", 0.03, 0.005, 0.5),
            param("SIG_NPHI", "NPHI uncertainty", "v/v", 0.03, 0.005, 0.5),
            param("SIG_DT", "DT uncertainty", "us/ft", 5.0, 0.5, 50.0),
            param("SIG_PEF", "PEF uncertainty", "b/e", 0.30, 0.02, 3.0),
            param("W_UNITY", "Unity-constraint weight", "", 1000.0, 1.0, 1e6),
            log_in("RHOB", "Density log", "g/cc", "RHOB", false),
            log_in("NPHI", "Neutron porosity log", "v/v", "NPHI", false),
            log_in("DT", "Sonic transit time log", "us/ft", "DT", false),
            log_in("PEF", "Photoelectric factor log", "b/e", "PEF", false),
            log_out("VOL_SAND", "Sand (quartz) volume", "v/v"),
            log_out("VOL_CLAY", "Clay volume", "v/v"),
            log_out("VOL_WATER", "Water volume", "v/v"),
            log_out("VOL_HC", "Hydrocarbon volume", "v/v"),
            log_out("PHIT_MM", "Total porosity (water + hc)", "v/v"),
            log_out("VSH_MM", "Shale volume (= clay)", "v/v"),
            log_out("SWT_MM", "Total water saturation", "v/v"),
            log_out("RECON_ERR", "Reconstruction error (sigma units)", ""),
        ],
    }
}

pub(crate) fn multimin(ctx: &ModuleContext) -> ModuleOutputs {
    let rhob = ctx.log("RHOB");
    let nphi = ctx.log("NPHI");
    let dt = ctx.log("DT");
    let pef = ctx.log("PEF");

    let mut vol_sand = vec![f32::NAN; ctx.n];
    let mut vol_clay = vec![f32::NAN; ctx.n];
    let mut vol_water = vec![f32::NAN; ctx.n];
    let mut vol_hc = vec![f32::NAN; ctx.n];
    let mut phit_mm = vec![f32::NAN; ctx.n];
    let mut vsh_mm = vec![f32::NAN; ctx.n];
    let mut swt_mm = vec![f32::NAN; ctx.n];
    let mut recon = vec![f32::NAN; ctx.n];

    for i in 0..ctx.n {
        // Assemble the tool equations that have a live log value at this sample.
        // Each entry: (endpoint responses per component, measured value, weight = 1/sigma).
        let mut rows: Vec<([f64; N], f64, f64)> = Vec::with_capacity(5);
        let mut push_tool = |val: f32, ends: [f64; N], sigma: f64| {
            let v = val as f64;
            if v.is_finite() && sigma > 0.0 {
                rows.push((ends, v, 1.0 / sigma));
            }
        };
        push_tool(
            rhob[i],
            [ctx.p("RHOB_SAND", i), ctx.p("RHOB_CLAY", i), ctx.p("RHOB_WATER", i), ctx.p("RHOB_HC", i)],
            ctx.p("SIG_RHOB", i),
        );
        push_tool(
            nphi[i],
            [ctx.p("NPHI_SAND", i), ctx.p("NPHI_CLAY", i), ctx.p("NPHI_WATER", i), ctx.p("NPHI_HC", i)],
            ctx.p("SIG_NPHI", i),
        );
        push_tool(
            dt[i],
            [ctx.p("DT_SAND", i), ctx.p("DT_CLAY", i), ctx.p("DT_WATER", i), ctx.p("DT_HC", i)],
            ctx.p("SIG_DT", i),
        );
        push_tool(
            pef[i],
            [ctx.p("PEF_SAND", i), ctx.p("PEF_CLAY", i), ctx.p("PEF_WATER", i), ctx.p("PEF_HC", i)],
            ctx.p("SIG_PEF", i),
        );

        // Need at least two tool equations to constrain a 4-component model with unity.
        let n_tools = rows.len();
        if n_tools < 2 {
            continue;
        }

        // Weighted design matrix A (rows scaled by weight), rhs b. Unity row appended last.
        let w_unity = ctx.p("W_UNITY", i);
        let mut a: Vec<[f64; N]> = Vec::with_capacity(n_tools + 1);
        let mut b: Vec<f64> = Vec::with_capacity(n_tools + 1);
        for (ends, val, w) in &rows {
            a.push(ends.map(|e| e * w));
            b.push(val * w);
        }
        a.push([w_unity; N]);
        b.push(w_unity);

        let x = nnls(&a, &b);

        // Reconstruction error: RMS residual over the tool rows only, in sigma units
        // (weights are 1/sigma, so the weighted residual is already dimensionless).
        let mut sse = 0.0;
        for r in 0..n_tools {
            let pred: f64 = (0..N).map(|c| a[r][c] * x[c]).sum();
            let d = pred - b[r];
            sse += d * d;
        }
        let rerr = (sse / n_tools as f64).sqrt();

        let (vs, vc, vw, vh) = (x[SAND], x[CLAY], x[WATER], x[HC]);
        let phit = vw + vh;
        vol_sand[i] = vs as f32;
        vol_clay[i] = vc as f32;
        vol_water[i] = vw as f32;
        vol_hc[i] = vh as f32;
        phit_mm[i] = phit as f32;
        vsh_mm[i] = vc as f32;
        swt_mm[i] = if phit > 1e-6 { (vw / phit) as f32 } else { f32::NAN };
        recon[i] = rerr as f32;
    }

    HashMap::from([
        ("VOL_SAND".to_string(), vol_sand),
        ("VOL_CLAY".to_string(), vol_clay),
        ("VOL_WATER".to_string(), vol_water),
        ("VOL_HC".to_string(), vol_hc),
        ("PHIT_MM".to_string(), phit_mm),
        ("VSH_MM".to_string(), vsh_mm),
        ("SWT_MM".to_string(), swt_mm),
        ("RECON_ERR".to_string(), recon),
    ])
}

// ---------------------------------------------------------------------------
// NNLS (Lawson & Hanson 1974, active-set) for the fixed 4-column model.
// ---------------------------------------------------------------------------

/// Solves min ||A x - b||_2 subject to x >= 0. `a` is m rows of N columns.
fn nnls(a: &[[f64; N]], b: &[f64]) -> [f64; N] {
    let m = a.len();
    let mut x = [0.0f64; N];
    let mut passive = [false; N]; // columns free to be non-zero
    let max_outer = 3 * N;

    // A^T r for the current residual r = b - A x.
    let grad = |x: &[f64; N]| -> [f64; N] {
        let mut resid = vec![0.0; m];
        for i in 0..m {
            let pred: f64 = (0..N).map(|c| a[i][c] * x[c]).sum();
            resid[i] = b[i] - pred;
        }
        let mut w = [0.0; N];
        for (c, wc) in w.iter_mut().enumerate() {
            *wc = (0..m).map(|i| a[i][c] * resid[i]).sum();
        }
        w
    };

    for _ in 0..max_outer {
        let w = grad(&x);
        // Bring in the active column with the largest positive gradient.
        let mut t = None;
        let mut best = 1e-9;
        for c in 0..N {
            if !passive[c] && w[c] > best {
                best = w[c];
                t = Some(c);
            }
        }
        let Some(t) = t else { break };
        passive[t] = true;

        // Inner loop: solve the unconstrained LS on the passive set, back out any
        // column that went non-positive, until the passive solution is all positive.
        for _ in 0..(2 * N) {
            let cols: Vec<usize> = (0..N).filter(|&c| passive[c]).collect();
            let z_p = lstsq_passive(a, b, &cols);
            let mut z = [0.0f64; N];
            for (k, &c) in cols.iter().enumerate() {
                z[c] = z_p[k];
            }
            if cols.iter().all(|&c| z[c] > 0.0) {
                for c in 0..N {
                    x[c] = if passive[c] { z[c] } else { 0.0 };
                }
                break;
            }
            // Step only partway toward z so a violating column hits zero.
            let mut alpha = f64::INFINITY;
            for &c in &cols {
                if z[c] <= 0.0 {
                    let denom = x[c] - z[c];
                    if denom.abs() > 1e-18 {
                        alpha = alpha.min(x[c] / denom);
                    }
                }
            }
            if !alpha.is_finite() {
                alpha = 0.0;
            }
            for c in 0..N {
                if passive[c] {
                    x[c] += alpha * (z[c] - x[c]);
                }
            }
            for c in 0..N {
                if passive[c] && x[c] <= 1e-12 {
                    passive[c] = false;
                    x[c] = 0.0;
                }
            }
        }
    }
    x
}

/// Ordinary least squares over the passive columns via the normal equations
/// (A_p^T A_p) z = A_p^T b, solved by Gaussian elimination with partial pivoting.
/// The passive set here is at most N=4 wide, so this is trivially cheap.
fn lstsq_passive(a: &[[f64; N]], b: &[f64], cols: &[usize]) -> Vec<f64> {
    let k = cols.len();
    let m = a.len();
    let mut ata = vec![vec![0.0f64; k]; k];
    let mut atb = vec![0.0f64; k];
    for p in 0..k {
        for q in 0..k {
            ata[p][q] = (0..m).map(|i| a[i][cols[p]] * a[i][cols[q]]).sum();
        }
        atb[p] = (0..m).map(|i| a[i][cols[p]] * b[i]).sum();
    }
    solve_linear(ata, atb)
}

/// Gaussian elimination with partial pivoting. Returns zeros for a singular system.
fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Vec<f64> {
    let k = b.len();
    for col in 0..k {
        let mut pivot = col;
        for r in (col + 1)..k {
            if a[r][col].abs() > a[pivot][col].abs() {
                pivot = r;
            }
        }
        if a[pivot][col].abs() < 1e-15 {
            return vec![0.0; k];
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
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ModuleContext;
    use std::collections::HashMap;

    fn ctx_one(logs: &[(&str, f32)]) -> ModuleContext {
        // Build a 1-sample context with all the spec's default params applied.
        let spec = multimin_spec();
        let params: HashMap<String, Vec<f64>> = spec
            .args
            .iter()
            .filter(|a| a.kind == crate::modules::ArgKind::Param)
            .map(|a| (a.name.clone(), vec![a.default.parse().unwrap()]))
            .collect();
        let logs_map: HashMap<String, Vec<f32>> =
            logs.iter().map(|(k, v)| (k.to_string(), vec![*v])).collect();
        ModuleContext { n: 1, logs: logs_map, params, opts: HashMap::new() }
    }

    #[test]
    fn nnls_solves_nonneg_least_squares() {
        // A well-posed positive system: diagonal, answer must be exact and >= 0.
        let a = [[2.0, 0.0, 0.0, 0.0], [0.0, 3.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 4.0]];
        let b = [4.0, 9.0, 5.0, 8.0];
        let x = nnls(&a, &b);
        assert!((x[0] - 2.0).abs() < 1e-9);
        assert!((x[1] - 3.0).abs() < 1e-9);
        assert!((x[2] - 5.0).abs() < 1e-9);
        assert!((x[3] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn nnls_clamps_negative_to_zero() {
        // Unconstrained solution would be negative; NNLS must return 0 there.
        let a = [[1.0, 0.0, 0.0, 0.0]];
        let b = [-5.0];
        let x = nnls(&a, &b);
        assert_eq!(x[0], 0.0);
    }

    #[test]
    fn multimin_recovers_known_clean_wet_sand() {
        // Forward-model a clean wet sand: 70% sand, 30% water, no clay/hc, using the
        // default endpoints, then check the inversion recovers those volumes.
        let (vs, vw) = (0.70, 0.30);
        let rhob = vs * 2.65 + vw * 1.0;
        let nphi = vs * -0.02 + vw * 1.0;
        let dt = vs * 55.5 + vw * 189.0;
        let pef = vs * 1.81 + vw * 0.36;
        let ctx = ctx_one(&[
            ("RHOB", rhob as f32),
            ("NPHI", nphi as f32),
            ("DT", dt as f32),
            ("PEF", pef as f32),
        ]);
        let out = multimin(&ctx);
        assert!((out["VOL_SAND"][0] - 0.70).abs() < 0.02, "sand={}", out["VOL_SAND"][0]);
        assert!((out["VOL_WATER"][0] - 0.30).abs() < 0.02, "water={}", out["VOL_WATER"][0]);
        assert!(out["VOL_CLAY"][0] < 0.02, "clay leaked: {}", out["VOL_CLAY"][0]);
        assert!(out["VOL_HC"][0] < 0.02, "hc leaked: {}", out["VOL_HC"][0]);
        assert!((out["PHIT_MM"][0] - 0.30).abs() < 0.02);
        assert!((out["SWT_MM"][0] - 1.0).abs() < 0.02, "clean wet sand SWT should be ~1");
        assert!(out["RECON_ERR"][0] < 0.5, "perfect data should reconstruct well");
    }

    #[test]
    fn multimin_skips_when_too_few_tools() {
        let ctx = ctx_one(&[("RHOB", 2.4)]); // only one tool → underdetermined, skip
        let out = multimin(&ctx);
        assert!(out["VOL_SAND"][0].is_nan());
    }
}
