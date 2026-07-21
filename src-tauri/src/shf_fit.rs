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
// Height-domain SHF forms fitted to a log-derived Sw-vs-H cloud (Wave B item 8, increment 2):
// Brooks-Corey and Skelt-Harrison, complementing the FOIL/BVW fit above. Both take (H, Sw) points
// (H = height above FWL) and return parameters + R² + a sampled fitted curve for the crossplot.
// --------------------------------------------------------------------------------------------

/// Brooks-Corey height form: Sw = Swirr + (1−Swirr)·(He/H)^λ for H ≥ He (Sw = 1 below He).
#[derive(Debug, Clone, Copy)]
pub struct BrooksCoreyFit {
    pub swirr: f64,
    pub he: f64,
    pub lambda: f64,
    pub r2: f64,
    pub n: usize,
}

/// Fits Brooks-Corey by gridding Swirr and, for each, a log-log linear fit of the effective
/// saturation Se=(Sw−Swirr)/(1−Swirr) against H (log Se = λ·log He − λ·log H). Picks the Swirr
/// with the best R². Robust and derivative-free. Needs ≥3 usable points.
pub fn fit_brooks_corey(points: &[(f64, f64)]) -> Option<BrooksCoreyFit> {
    let clean: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(h, sw)| h.is_finite() && sw.is_finite() && h > 0.0 && sw > 0.0 && sw <= 1.0)
        .collect();
    if clean.len() < 3 {
        return None;
    }
    let min_sw = clean.iter().map(|&(_, s)| s).fold(f64::INFINITY, f64::min);
    let mut best: Option<BrooksCoreyFit> = None;
    // Swirr must be below the lowest observed Sw; sweep a grid up to 95% of it.
    let hi = (min_sw * 0.95).max(0.0);
    let steps = 40;
    for i in 0..=steps {
        let swirr = hi * i as f64 / steps as f64;
        // Linear fit of log10 Se vs log10 H over points with 0 < Se < 1.
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for &(h, sw) in &clean {
            let se = (sw - swirr) / (1.0 - swirr);
            if se > 1e-6 && se < 1.0 {
                xs.push(h.log10());
                ys.push(se.log10());
            }
        }
        let n = xs.len();
        if n < 3 {
            continue;
        }
        let mx = xs.iter().sum::<f64>() / n as f64;
        let my = ys.iter().sum::<f64>() / n as f64;
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for k in 0..n {
            let dx = xs[k] - mx;
            let dy = ys[k] - my;
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
        }
        if sxx <= 0.0 {
            continue;
        }
        let slope = sxy / sxx; // = −λ
        let lambda = -slope;
        if lambda <= 0.0 {
            continue; // Se must fall with height
        }
        let intercept = my - slope * mx; // = λ·log10 He
        let he = 10f64.powf(intercept / lambda);
        let r2 = if syy > 0.0 { (sxy * sxy) / (sxx * syy) } else { 1.0 };
        let cand = BrooksCoreyFit { swirr, he, lambda, r2: r2.clamp(0.0, 1.0), n };
        if best.map(|b| cand.r2 > b.r2).unwrap_or(true) {
            best = Some(cand);
        }
    }
    best
}

/// Skelt-Harrison height form: Sw(H) = 1 − A·exp(−(B/(H+D))^C).
#[derive(Debug, Clone, Copy)]
pub struct SkeltFit {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub r2: f64,
    pub n: usize,
}

fn skelt_sw(a: f64, b: f64, c: f64, d: f64, h: f64) -> f64 {
    let hd = h + d;
    if hd <= 0.0 {
        return 1.0;
    }
    1.0 - a * (-(b / hd).powf(c)).exp()
}

/// Compact bounded Nelder-Mead simplex minimizer (fixed iteration budget). `f` returns the loss;
/// `lo`/`hi` clamp each dimension. Derivative-free — fine for the small 4-parameter Skelt fit.
fn nelder_mead<F: Fn(&[f64; 4]) -> f64>(f: F, x0: [f64; 4], lo: [f64; 4], hi: [f64; 4], iters: usize) -> [f64; 4] {
    let clamp = |x: [f64; 4]| -> [f64; 4] {
        let mut o = x;
        for i in 0..4 {
            o[i] = o[i].clamp(lo[i], hi[i]);
        }
        o
    };
    // Build the initial simplex (5 vertices).
    let mut simplex: Vec<[f64; 4]> = vec![clamp(x0)];
    for i in 0..4 {
        let mut v = x0;
        let step = (hi[i] - lo[i]) * 0.1 + 1e-3;
        v[i] += step;
        simplex.push(clamp(v));
    }
    let mut fvals: Vec<f64> = simplex.iter().map(|v| f(v)).collect();
    for _ in 0..iters {
        // Order by loss.
        let mut idx: Vec<usize> = (0..simplex.len()).collect();
        idx.sort_by(|&a, &b| fvals[a].partial_cmp(&fvals[b]).unwrap_or(std::cmp::Ordering::Equal));
        let best = idx[0];
        let worst = idx[idx.len() - 1];
        let second = idx[idx.len() - 2];
        // Centroid of all but the worst.
        let mut cen = [0.0; 4];
        for (&j, _) in idx.iter().zip(0..idx.len() - 1) {
            for k in 0..4 {
                cen[k] += simplex[j][k];
            }
        }
        for k in 0..4 {
            cen[k] /= (idx.len() - 1) as f64;
        }
        let reflect = |coef: f64| -> [f64; 4] {
            let mut r = [0.0; 4];
            for k in 0..4 {
                r[k] = cen[k] + coef * (cen[k] - simplex[worst][k]);
            }
            clamp(r)
        };
        let xr = reflect(1.0);
        let fr = f(&xr);
        if fr < fvals[best] {
            let xe = reflect(2.0);
            let fe = f(&xe);
            if fe < fr {
                simplex[worst] = xe;
                fvals[worst] = fe;
            } else {
                simplex[worst] = xr;
                fvals[worst] = fr;
            }
        } else if fr < fvals[second] {
            simplex[worst] = xr;
            fvals[worst] = fr;
        } else {
            let xc = reflect(0.5);
            let fc = f(&xc);
            if fc < fvals[worst] {
                simplex[worst] = xc;
                fvals[worst] = fc;
            } else {
                // Shrink toward the best.
                for &j in idx.iter().skip(1) {
                    for k in 0..4 {
                        simplex[j][k] = simplex[best][k] + 0.5 * (simplex[j][k] - simplex[best][k]);
                    }
                    simplex[j] = clamp(simplex[j]);
                    fvals[j] = f(&simplex[j]);
                }
            }
        }
    }
    let bi = (0..fvals.len()).min_by(|&a, &b| fvals[a].partial_cmp(&fvals[b]).unwrap()).unwrap();
    simplex[bi]
}

/// Fits Skelt-Harrison by Nelder-Mead on the sum of squared Sw residuals. Needs ≥4 usable points.
pub fn fit_skelt(points: &[(f64, f64)]) -> Option<SkeltFit> {
    let clean: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(h, sw)| h.is_finite() && sw.is_finite() && h > 0.0 && sw > 0.0 && sw <= 1.0)
        .collect();
    if clean.len() < 4 {
        return None;
    }
    let min_sw = clean.iter().map(|&(_, s)| s).fold(f64::INFINITY, f64::min);
    let mut hs: Vec<f64> = clean.iter().map(|&(h, _)| h).collect();
    hs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let hmed = hs[hs.len() / 2];
    let sse = |p: &[f64; 4]| -> f64 {
        clean.iter().map(|&(h, sw)| (sw - skelt_sw(p[0], p[1], p[2], p[3], h)).powi(2)).sum()
    };
    let x0 = [(1.0 - min_sw).clamp(0.05, 1.0), hmed.max(1.0), 1.0, 0.0];
    let lo = [0.05, 1e-3, 0.1, 0.0];
    let hi = [1.0, hmed * 20.0 + 100.0, 10.0, hmed.max(10.0)];
    let best = nelder_mead(sse, x0, lo, hi, 400);
    // R² from the residual sum of squares vs the total.
    let mean_sw = clean.iter().map(|&(_, s)| s).sum::<f64>() / clean.len() as f64;
    let ss_tot: f64 = clean.iter().map(|&(_, s)| (s - mean_sw).powi(2)).sum();
    let ss_res = sse(&best);
    let r2 = if ss_tot > 0.0 { (1.0 - ss_res / ss_tot).clamp(0.0, 1.0) } else { 1.0 };
    Some(SkeltFit { a: best[0], b: best[1], c: best[2], d: best[3], r2, n: clean.len() })
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

// --------------------------------------------------------------------------------------------
// DB-backed command for the height-domain forms (Brooks-Corey / Skelt-Harrison): pool the
// log-derived Sw-vs-H cloud and fit the chosen form. Returns the scatter + a sampled fit curve.
// --------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ShfFitRequest {
    pub well_ids: Vec<String>,
    pub phie_curve: String,
    pub sw_curve: String,
    pub tvdss_curve: String,
    pub fwl: f64,
    #[serde(default)]
    pub min_phi: f64,
    /// "brooks_corey" | "skelt".
    pub method: String,
}

#[derive(Debug, Serialize)]
pub struct ShfPoint {
    pub h: f64,
    pub sw: f64,
    pub well_id: String,
}

#[derive(Debug, Serialize)]
pub struct ShfFitResult {
    pub method: String,
    pub params: Vec<(String, f64)>,
    pub r2: f64,
    pub n_points: usize,
    pub points: Vec<ShfPoint>,
    /// Sampled fitted Sw(H) curve for the overlay: (H, Sw) pairs across the data range.
    pub curve: Vec<(f64, f64)>,
    pub error: Option<String>,
}

fn shf_err(method: &str, msg: &str) -> ShfFitResult {
    ShfFitResult {
        method: method.to_string(),
        params: vec![],
        r2: f64::NAN,
        n_points: 0,
        points: vec![],
        curve: vec![],
        error: Some(msg.to_string()),
    }
}

pub fn run_shf_fit(db: &Mutex<Connection>, req: &ShfFitRequest) -> ShfFitResult {
    if req.well_ids.is_empty() {
        return shf_err(&req.method, "select at least one well");
    }
    let phie = req.phie_curve.trim().to_uppercase();
    let sw = req.sw_curve.trim().to_uppercase();
    let tvdss = req.tvdss_curve.trim().to_uppercase();
    if phie.is_empty() || sw.is_empty() || tvdss.is_empty() {
        return shf_err(&req.method, "PHIE, SW and TVDSS curves are all required");
    }

    // Pool (well, H=FWL−TVDSS, Sw) above the FWL, above the porosity cutoff.
    let mut samples: Vec<(String, f64, f64)> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let names = vec![phie.clone(), sw.clone(), tvdss.clone()];
        for well_id in &req.well_ids {
            let Ok((_d, cols)) = fetch_curve_frame(&conn, well_id, &names) else { continue };
            let (Some(pv), Some(sv), Some(tv)) = (cols.get(&phie), cols.get(&sw), cols.get(&tvdss)) else {
                continue;
            };
            let n = pv.len().min(sv.len()).min(tv.len());
            for i in 0..n {
                let (p, s, t) = (pv[i] as f64, sv[i] as f64, tv[i] as f64);
                let h = req.fwl - t;
                if p.is_finite() && s.is_finite() && t.is_finite() && p > req.min_phi && h > 0.0 && s > 0.0 && s <= 1.0 {
                    samples.push((well_id.clone(), h, s));
                }
            }
        }
    }
    if samples.len() < 4 {
        return shf_err(&req.method, "not enough Sw samples above the FWL / porosity cutoff");
    }

    let pts: Vec<(f64, f64)> = samples.iter().map(|(_, h, s)| (*h, *s)).collect();
    let hmin = pts.iter().map(|&(h, _)| h).fold(f64::INFINITY, f64::min).max(1e-3);
    let hmax = pts.iter().map(|&(h, _)| h).fold(f64::NEG_INFINITY, f64::max);

    let (params, r2, n, model): (Vec<(String, f64)>, f64, usize, Box<dyn Fn(f64) -> f64>) =
        match req.method.as_str() {
            "brooks_corey" => {
                let Some(f) = fit_brooks_corey(&pts) else {
                    return shf_err(&req.method, "Brooks-Corey fit failed (too few points in the transition zone)");
                };
                let (swirr, he, lambda) = (f.swirr, f.he, f.lambda);
                (
                    vec![("swirr".into(), f.swirr), ("he".into(), f.he), ("lambda".into(), f.lambda)],
                    f.r2,
                    f.n,
                    Box::new(move |h: f64| if h >= he { swirr + (1.0 - swirr) * (he / h).powf(lambda) } else { 1.0 }),
                )
            }
            "skelt" => {
                let Some(f) = fit_skelt(&pts) else {
                    return shf_err(&req.method, "Skelt-Harrison fit failed");
                };
                let (a, b, c, d) = (f.a, f.b, f.c, f.d);
                (
                    vec![("A".into(), f.a), ("B".into(), f.b), ("C".into(), f.c), ("D".into(), f.d)],
                    f.r2,
                    f.n,
                    Box::new(move |h: f64| skelt_sw(a, b, c, d, h)),
                )
            }
            other => return shf_err(&req.method, &format!("unknown SHF method '{other}'")),
        };

    // Sampled fit curve across the observed H range.
    let steps = 60usize;
    let curve: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            let h = hmin + (hmax - hmin) * i as f64 / steps as f64;
            (h, model(h).clamp(0.0, 1.0))
        })
        .collect();

    // Decimated scatter.
    let stride = (samples.len() / MAX_FOIL_POINTS).max(1);
    let points: Vec<ShfPoint> = samples
        .iter()
        .step_by(stride)
        .map(|(w, h, s)| ShfPoint { h: *h, sw: *s, well_id: w.clone() })
        .collect();

    ShfFitResult { method: req.method.clone(), params, r2, n_points: n, points, curve, error: None }
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

    #[test]
    fn brooks_corey_recovers_synthetic_curve() {
        // Sw = 0.15 + 0.85·(5/H)^0.5 sampled above the entry height; fit must recover the params.
        let (swirr, he, lambda): (f64, f64, f64) = (0.15, 5.0, 0.5);
        let pts: Vec<(f64, f64)> = [8.0f64, 10.0, 20.0, 40.0, 80.0, 160.0, 320.0]
            .iter()
            .map(|&h| (h, swirr + (1.0 - swirr) * (he / h).powf(lambda)))
            .collect();
        let fit = fit_brooks_corey(&pts).expect("bc fit");
        assert!(fit.r2 > 0.995, "r2={}", fit.r2);
        assert!((fit.lambda - lambda).abs() < 0.1, "lambda={}", fit.lambda);
        assert!((fit.swirr - swirr).abs() < 0.03, "swirr={}", fit.swirr);
        assert!((fit.he - he).abs() < 1.5, "he={}", fit.he);
    }

    #[test]
    fn skelt_fits_synthetic_curve_well() {
        // Sw = 1 − 0.85·exp(−(20/H)^1.5); Nelder-Mead must reach a high-R² fit (params are not
        // uniquely identifiable, so assert fit quality + monotonic decrease, not exact params).
        let pts: Vec<(f64, f64)> = [5.0, 10.0, 20.0, 40.0, 80.0, 160.0]
            .iter()
            .map(|&h| (h, 1.0 - 0.85 * (-(20.0f64 / h).powf(1.5)).exp()))
            .collect();
        let fit = fit_skelt(&pts).expect("skelt fit");
        assert!(fit.r2 > 0.98, "r2={}", fit.r2);
        // Fitted Sw must fall from shallow (small H) to deep (large H).
        let hi = skelt_sw(fit.a, fit.b, fit.c, fit.d, 10.0);
        let lo = skelt_sw(fit.a, fit.b, fit.c, fit.d, 160.0);
        assert!(hi > lo, "Sw not decreasing: {hi} !> {lo}");
    }

    #[test]
    fn height_fits_reject_too_few_points() {
        assert!(fit_brooks_corey(&[(10.0, 0.5), (20.0, 0.4)]).is_none());
        assert!(fit_skelt(&[(10.0, 0.5), (20.0, 0.4), (30.0, 0.3)]).is_none());
    }
}
