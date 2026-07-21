//! Saturation-height FUNCTION fitting (Wave B item 8, SHF side) — the Cuddy FOIL / fractal BVW
//! method, grounded in Cuddy et al. 1993 (SPWLA, read in full) and Cuddy 2017 (fractal update).
//!
//! FOIL: bulk-volume water is a porosity- and permeability-independent power law of height above
//! the free-water level, BVW = Sw·φ = a·H^b (b negative). Fitted as a straight line in log10
//! space: log10(BVW) = log10(a) + b·log10(H), by ordinary least squares (Cuddy fits BVW as the
//! independent variable — we follow the standard log-log regression here). This is the pragmatic
//! Mahakam route when SCAL Pc is sparse: it needs only the computed PHIE, SW and TVDSS logs.
//!
//! Also provides the Cuddy Eq 19 FWL scan: step a candidate common FWL through a window and pick
//! the depth that minimizes the pooled log-residual of the FOIL fit (sharpest, best-correlated fit).
//!
//! Deferred to later increments: Brooks-Corey / Thomeer / Skelt-Harrison / Leverett-J-from-SCAL
//! fitting, per-rock-type lambda variants, SCAL porous-plate / centrifuge importers.

use crate::equations::fetch_curve_frame;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Result of a FOIL fit: BVW = a·H^b with coefficient of determination in log space.
#[derive(Debug, Clone, Copy)]
pub struct FoilFit {
    pub a: f64,
    pub b: f64,
    pub r2: f64,
    pub n: usize,
}

/// Fits BVW = a·H^b to (H, BVW) points via least squares on (log10 H, log10 BVW).
/// Only points with H > 0 and BVW > 0 are used. Returns None if fewer than 2 usable points
/// or the heights are all equal (degenerate — no slope).
pub fn fit_foil(points: &[(f64, f64)]) -> Option<FoilFit> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for &(h, bvw) in points {
        if h.is_finite() && bvw.is_finite() && h > 0.0 && bvw > 0.0 {
            xs.push(h.log10());
            ys.push(bvw.log10());
        }
    }
    let n = xs.len();
    if n < 2 {
        return None;
    }
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut syy = 0.0;
    for i in 0..n {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx <= 0.0 {
        return None; // all heights equal — slope undefined
    }
    let b = sxy / sxx;
    let log_a = mean_y - b * mean_x;
    let a = 10f64.powf(log_a);
    // R^2 in log space.
    let r2 = if syy > 0.0 { (sxy * sxy) / (sxx * syy) } else { 1.0 };
    Some(FoilFit { a, b, r2: r2.clamp(0.0, 1.0), n })
}

/// Pooled sum of squared log10 residuals of the FOIL fit for a given set of (H, BVW) points —
/// the Cuddy Eq 19 quality metric (lower = tighter fit). Returns +inf if the fit is degenerate.
fn foil_residual(points: &[(f64, f64)]) -> f64 {
    let Some(fit) = fit_foil(points) else { return f64::INFINITY };
    let log_a = fit.a.log10();
    let mut ss = 0.0;
    let mut n = 0usize;
    for &(h, bvw) in points {
        if h.is_finite() && bvw.is_finite() && h > 0.0 && bvw > 0.0 {
            let pred = log_a + fit.b * h.log10();
            let res = bvw.log10() - pred;
            ss += res * res;
            n += 1;
        }
    }
    if n == 0 {
        f64::INFINITY
    } else {
        ss / n as f64
    }
}

/// One point on the FWL-scan quality curve: candidate free-water level (TVDSS) and the mean-squared
/// log residual of the FOIL fit recomputed at that FWL.
#[derive(Debug, Clone, Copy)]
pub struct FwlScanPoint {
    pub fwl: f64,
    pub residual: f64,
}

/// Cuddy Eq 19 FWL scan over a common free-water level. `samples` are (TVDSS, BVW) for every
/// pooled sample (BVW already = Sw·φ). For each candidate FWL, height H = FWL − TVDSS (samples
/// below FWL, H ≤ 0, drop out), the FOIL fit is recomputed and its residual recorded. Returns the
/// full quality curve and the argmin FWL. `lo`/`hi` are TVDSS bounds; `step` the increment.
pub fn foil_fwl_scan(samples: &[(f64, f64)], lo: f64, hi: f64, step: f64) -> (Vec<FwlScanPoint>, Option<f64>) {
    let step = if step.abs() < 1e-9 { 0.5 } else { step.abs() };
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    let mut curve = Vec::new();
    let mut best: Option<(f64, f64)> = None; // (fwl, residual)
    let steps = (((hi - lo) / step).floor() as i64).max(0);
    for s in 0..=steps {
        let fwl = lo + s as f64 * step;
        let pts: Vec<(f64, f64)> = samples.iter().map(|&(tvdss, bvw)| (fwl - tvdss, bvw)).collect();
        let res = foil_residual(&pts);
        if res.is_finite() {
            curve.push(FwlScanPoint { fwl, residual: res });
            if best.map(|(_, br)| res < br).unwrap_or(true) {
                best = Some((fwl, res));
            }
        }
    }
    (curve, best.map(|(f, _)| f))
}

// --------------------------------------------------------------------------------------------
// DB-backed command: pool computed PHIE/SW/TVDSS across wells, fit the field-wide FOIL, and
// (optionally) scan for the common FWL. Returns the (H, BVW) scatter for the crossplot too.
// --------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CuddyFoilRequest {
    pub well_ids: Vec<String>,
    pub phie_curve: String,
    pub sw_curve: String,
    pub tvdss_curve: String,
    /// Common free-water level (TVDSS) used for the fit when the scan is off, or the scan centre.
    pub fwl: f64,
    /// Exclude non-net porosity below this (net-reservoir rule, Cuddy 1993). 0 = keep all.
    #[serde(default)]
    pub min_phi: f64,
    #[serde(default)]
    pub scan: bool,
    #[serde(default)]
    pub scan_lo: f64,
    #[serde(default)]
    pub scan_hi: f64,
    #[serde(default)]
    pub scan_step: f64,
}

#[derive(Debug, Serialize)]
pub struct FoilPoint {
    pub h: f64,
    pub bvw: f64,
    pub well_id: String,
}

#[derive(Debug, Serialize)]
pub struct FwlScanOut {
    pub fwl: f64,
    pub residual: f64,
}

#[derive(Debug, Serialize)]
pub struct CuddyFoilResult {
    pub a: f64,
    pub b: f64,
    pub r2: f64,
    pub n_points: usize,
    pub fwl_used: f64,
    pub fwl_best: Option<f64>,
    pub points: Vec<FoilPoint>,
    pub scan: Vec<FwlScanOut>,
    pub error: Option<String>,
}

fn foil_err(msg: &str) -> CuddyFoilResult {
    CuddyFoilResult {
        a: f64::NAN,
        b: f64::NAN,
        r2: f64::NAN,
        n_points: 0,
        fwl_used: f64::NAN,
        fwl_best: None,
        points: vec![],
        scan: vec![],
        error: Some(msg.to_string()),
    }
}

/// At most this many scatter points are returned to the UI (uniformly decimated beyond it).
const MAX_FOIL_POINTS: usize = 4000;

pub fn run_cuddy_foil(db: &Mutex<Connection>, req: &CuddyFoilRequest) -> CuddyFoilResult {
    if req.well_ids.is_empty() {
        return foil_err("select at least one well");
    }
    let phie = req.phie_curve.trim().to_uppercase();
    let sw = req.sw_curve.trim().to_uppercase();
    let tvdss = req.tvdss_curve.trim().to_uppercase();
    if phie.is_empty() || sw.is_empty() || tvdss.is_empty() {
        return foil_err("PHIE, SW and TVDSS curves are all required");
    }

    // Pool (well, TVDSS, BVW=Sw·φ) samples across the wells.
    let mut samples: Vec<(String, f64, f64)> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let names = vec![phie.clone(), sw.clone(), tvdss.clone()];
        for well_id in &req.well_ids {
            let Ok((_depth, cols)) = fetch_curve_frame(&conn, well_id, &names) else { continue };
            let (Some(pv), Some(sv), Some(tv)) = (cols.get(&phie), cols.get(&sw), cols.get(&tvdss)) else {
                continue;
            };
            let n = pv.len().min(sv.len()).min(tv.len());
            for i in 0..n {
                let (p, s, t) = (pv[i] as f64, sv[i] as f64, tv[i] as f64);
                if p.is_finite() && s.is_finite() && t.is_finite() && p > req.min_phi && p > 0.0 && s > 0.0 {
                    samples.push((well_id.clone(), t, p * s));
                }
            }
        }
    }
    if samples.len() < 2 {
        return foil_err("not enough complete PHIE/SW/TVDSS samples above the porosity cutoff");
    }

    // Optional FWL scan over a common contact; otherwise use the requested FWL.
    let mut scan_out = Vec::new();
    let mut fwl_best = None;
    let fwl_used = if req.scan {
        let ts: Vec<(f64, f64)> = samples.iter().map(|(_, t, b)| (*t, *b)).collect();
        let (curve, best) = foil_fwl_scan(&ts, req.scan_lo, req.scan_hi, req.scan_step);
        scan_out = curve.iter().map(|p| FwlScanOut { fwl: p.fwl, residual: p.residual }).collect();
        fwl_best = best;
        best.unwrap_or(req.fwl)
    } else {
        req.fwl
    };

    // Height above FWL and the fit.
    let pts: Vec<(f64, f64)> =
        samples.iter().map(|(_, t, b)| (fwl_used - t, *b)).filter(|(h, _)| *h > 0.0).collect();
    let Some(fit) = fit_foil(&pts) else {
        return foil_err("FOIL fit failed — no samples above the FWL, or heights all equal");
    };

    // Scatter for the crossplot (decimated).
    let above: Vec<&(String, f64, f64)> = samples.iter().filter(|(_, t, _)| fwl_used - t > 0.0).collect();
    let stride = (above.len() / MAX_FOIL_POINTS).max(1);
    let points: Vec<FoilPoint> = above
        .iter()
        .step_by(stride)
        .map(|(w, t, b)| FoilPoint { h: fwl_used - t, bvw: *b, well_id: w.clone() })
        .collect();

    CuddyFoilResult {
        a: fit.a,
        b: fit.b,
        r2: fit.r2,
        n_points: fit.n,
        fwl_used,
        fwl_best,
        points,
        scan: scan_out,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_recovers_known_power_law() {
        // BVW = 0.05 * H^-0.8 sampled exactly → fit must return a≈0.05, b≈-0.8, R²≈1.
        let pts: Vec<(f64, f64)> = (1..=50).map(|i| { let h = i as f64; (h, 0.05 * h.powf(-0.8)) }).collect();
        let fit = fit_foil(&pts).expect("fit");
        assert!((fit.a - 0.05).abs() < 1e-6, "a={}", fit.a);
        assert!((fit.b - (-0.8)).abs() < 1e-6, "b={}", fit.b);
        assert!(fit.r2 > 0.9999, "r2={}", fit.r2);
        assert_eq!(fit.n, 50);
    }

    #[test]
    fn fit_rejects_degenerate_and_nonpositive() {
        assert!(fit_foil(&[]).is_none());
        assert!(fit_foil(&[(10.0, 0.02)]).is_none()); // one point
        assert!(fit_foil(&[(5.0, 0.03), (5.0, 0.04)]).is_none()); // all same H
        // Non-positive H/BVW are filtered, leaving too few points.
        assert!(fit_foil(&[(-1.0, 0.02), (2.0, -0.03)]).is_none());
    }

    #[test]
    fn fwl_scan_finds_the_true_contact() {
        // Build samples whose BVW is an exact FOIL of height above a TRUE FWL of 2000 m TVDSS:
        // BVW = 0.05*(FWL_true - TVDSS)^-0.8 for TVDSS above the contact. The scan over candidate
        // FWLs must land on ~2000 (the residual is ~0 only when H uses the true contact).
        let fwl_true = 2000.0;
        let samples: Vec<(f64, f64)> = (1..=40)
            .map(|i| { let tvdss = fwl_true - i as f64 * 5.0; let h = fwl_true - tvdss; (tvdss, 0.05 * h.powf(-0.8)) })
            .collect();
        let (curve, best) = foil_fwl_scan(&samples, 1990.0, 2010.0, 0.5);
        assert!(!curve.is_empty());
        let best = best.expect("best fwl");
        assert!((best - fwl_true).abs() <= 0.5, "best fwl = {best}");
    }
}
