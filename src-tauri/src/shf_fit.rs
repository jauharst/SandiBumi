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
//! Height-domain forms fitted here: Cuddy FOIL (BVW), Brooks-Corey, Skelt-Harrison, Thomeer
//! (hyperbola, carbonate standard) and a log-derived Leverett-J (Sw = A·J^B with J from PERM/PHIE
//! and fluid props). All forms accept an optional rock-type curve and then ALSO fit one law per
//! RT class (playbook #4 — the per-rock-type split is the single biggest accuracy win on stacked
//! Mahakam sands). Deferred: SCAL porous-plate / centrifuge importers, MICP-calibrated coeffs.

use crate::satheight::{J_CONST, PSI_PER_FT_PER_SG};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
/// `lo`/`hi` clamp each dimension. Derivative-free — fine for the small 4-parameter Skelt fit
/// (and reused by the 3-parameter Thomeer fit with the spare dimension pinned lo == hi).
pub(crate) fn nelder_mead<F: Fn(&[f64; 4]) -> f64>(
    f: F,
    x0: [f64; 4],
    lo: [f64; 4],
    hi: [f64; 4],
    iters: usize,
) -> [f64; 4] {
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
// Thomeer hyperbola in the height domain (Thomeer 1960, "Introduction of a Pore Geometrical
// Factor Defined by the Capillary Pressure Curve", JPT 12(3) / Trans. AIME 219). The lab form
// is a log-hyperbola in capillary pressure: Bv/B∞ = exp(−G / log10(Pc/Pd)) for Pc > Pd, with
// Pd the displacement (entry) pressure and G the pore-geometrical factor (≈0.1 well-sorted,
// >2 poorly sorted). Reservoir Pc is proportional to height above the FWL (Pc = 0.433·Δρ·h_ft),
// so Pc/Pd = H/Hd and the curve fits directly in height with an entry HEIGHT Hd standing in for
// Pd. With the non-wetting plateau B∞/φ = 1 − Swirr:
//   Sw(H) = 1 − (1 − Swirr)·exp(−G / log10(H/Hd))   for H > Hd;   Sw = 1 at/below Hd.
// --------------------------------------------------------------------------------------------

/// Thomeer height-form fit: entry height Hd (m), pore-geometrical factor G, irreducible Swirr.
#[derive(Debug, Clone, Copy)]
pub struct ThomeerFit {
    pub swirr: f64,
    pub hd: f64,
    pub g: f64,
    pub r2: f64,
    pub n: usize,
}

fn thomeer_sw(swirr: f64, hd: f64, g: f64, h: f64) -> f64 {
    if h <= hd || hd <= 0.0 {
        return 1.0;
    }
    let lg = (h / hd).log10();
    1.0 - (1.0 - swirr) * (-(g / lg)).exp()
}

/// Fits the Thomeer height form by bounded Nelder-Mead on the Sw sum of squares (the spare 4th
/// simplex dimension is pinned lo == hi, per the minimizer's contract). Needs ≥4 usable points.
pub fn fit_thomeer(points: &[(f64, f64)]) -> Option<ThomeerFit> {
    let clean: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(h, sw)| h.is_finite() && sw.is_finite() && h > 0.0 && sw > 0.0 && sw <= 1.0)
        .collect();
    if clean.len() < 4 {
        return None;
    }
    let min_sw = clean.iter().map(|&(_, s)| s).fold(f64::INFINITY, f64::min);
    let hmin = clean.iter().map(|&(h, _)| h).fold(f64::INFINITY, f64::min);
    let hmax = clean.iter().map(|&(h, _)| h).fold(f64::NEG_INFINITY, f64::max);
    if hmax <= 1e-3 {
        // Sub-millimetre height range (e.g. a constant curve mis-picked as TVDSS): no fittable
        // curve, and the Hd bounds below would invert (lo > hi panics f64::clamp).
        return None;
    }
    let sse = |p: &[f64; 4]| -> f64 {
        clean.iter().map(|&(h, sw)| (sw - thomeer_sw(p[0], p[1], p[2], h)).powi(2)).sum()
    };
    // Swirr must sit below the lowest observed Sw (Thomeer approaches it only logarithmically,
    // so leave generous room); the entry height below the shallowest-H sample (points at/below
    // Hd read Sw = 1 and carry no shape information); G spans sorted → poorly-sorted rock.
    let x0 = [(min_sw * 0.6).clamp(0.0, 0.9), (hmin * 0.5).max(1e-3), 0.7, 0.0];
    let lo = [0.0, 1e-3, 0.02, 0.0];
    let hi = [(min_sw * 0.95).max(0.0), hmax, 8.0, 0.0];
    let best = nelder_mead(sse, x0, lo, hi, 600);
    let mean_sw = clean.iter().map(|&(_, s)| s).sum::<f64>() / clean.len() as f64;
    let ss_tot: f64 = clean.iter().map(|&(_, s)| (s - mean_sw).powi(2)).sum();
    let ss_res = sse(&best);
    let r2 = if ss_tot > 0.0 { (1.0 - ss_res / ss_tot).clamp(0.0, 1.0) } else { 1.0 };
    Some(ThomeerFit { swirr: best[0], hd: best[1], g: best[2], r2, n: clean.len() })
}

/// Least-squares power law y = A·x^B in ln-ln space over strictly positive points — the
/// Leverett 1941 J-function form when fed (J, Sw) (Leverett, "Capillary behaviour in porous
/// solids", Trans. AIME 142). Returns (A, B, R² in ln space, n). None below 3 usable points.
fn fit_power_lnln(points: &[(f64, f64)]) -> Option<(f64, f64, f64, usize)> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for &(x, y) in points {
        if x.is_finite() && y.is_finite() && x > 0.0 && y > 0.0 {
            xs.push(x.ln());
            ys.push(y.ln());
        }
    }
    let n = xs.len();
    if n < 3 {
        return None;
    }
    let nf = n as f64;
    let mx = xs.iter().sum::<f64>() / nf;
    let my = ys.iter().sum::<f64>() / nf;
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let dx = xs[i] - mx;
        let dy = ys[i] - my;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx <= 0.0 {
        return None;
    }
    let b = sxy / sxx;
    let a = (my - b * mx).exp();
    let r2 = if syy > 0.0 { ((sxy * sxy) / (sxx * syy)).clamp(0.0, 1.0) } else { 1.0 };
    Some((a, b, r2, n))
}

// --------------------------------------------------------------------------------------------
// DB-backed command: pool computed PHIE/SW/TVDSS across wells, fit the field-wide FOIL, and
// (optionally) scan for the common FWL. Returns the (H, BVW) scatter for the crossplot too.
// --------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CuddyFoilRequest {
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
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
    /// Optional rock-type/facies curve — when set, ALSO fit one FOIL law per rounded RT class.
    #[serde(default)]
    pub rt_curve: String,
}

#[derive(Debug, Serialize)]
pub struct FoilPoint {
    pub h: f64,
    pub bvw: f64,
    pub well_id: String,
    /// Rounded rock-type class of the sample (null when no RT curve was supplied / RT is NaN).
    pub rt: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct FwlScanOut {
    pub fwl: f64,
    pub residual: f64,
}

/// One per-rock-type FOIL law (BVW = a·H^b fitted over that RT class only).
#[derive(Debug, Serialize)]
pub struct FoilGroupFit {
    pub rt: i32,
    pub a: f64,
    pub b: f64,
    pub r2: f64,
    pub n_points: usize,
    pub error: Option<String>,
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
    /// Per-rock-type fits when an RT curve was supplied (ascending RT class).
    pub groups: Vec<FoilGroupFit>,
    /// (reason, count) of candidate samples excluded from the fit — never drop silently.
    pub excluded: Vec<(String, usize)>,
    pub notes: Vec<String>,
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
        groups: vec![],
        excluded: vec![],
        notes: vec![],
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

    let rt_name = req.rt_curve.trim().to_uppercase();

    // Pool (well, TVDSS, BVW=Sw·φ, RT) samples across the wells, counting what gets dropped.
    let mut samples: Vec<(String, f64, f64, Option<i32>)> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let (mut ex_phi, mut ex_sw_hi, mut ex_sw_lo) = (0usize, 0usize, 0usize);
    let mut empty_wells: Vec<String> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let mut names = vec![phie.clone(), sw.clone(), tvdss.clone()];
        if !rt_name.is_empty() {
            names.push(rt_name.clone());
        }
        for well_id in &req.well_ids {
            let before = samples.len();
            'well: {
                let Ok((_depth, cols)) = crate::equations::fetch_curve_frame_from_set(&conn, well_id, &names, req.input_set.as_deref(), None) else { break 'well };
                let (Some(pv), Some(sv), Some(tv)) = (cols.get(&phie), cols.get(&sw), cols.get(&tvdss)) else {
                    break 'well;
                };
            let rv = cols.get(&rt_name);
            let n = pv.len().min(sv.len()).min(tv.len());
            for i in 0..n {
                let (p, s, t) = (pv[i] as f64, sv[i] as f64, tv[i] as f64);
                if !(p.is_finite() && s.is_finite() && t.is_finite()) {
                    continue; // incomplete inputs (outside the computed interval) — not a candidate
                }
                if !(p > req.min_phi && p > 0.0) {
                    ex_phi += 1;
                    continue;
                }
                if s > 1.0 {
                    ex_sw_hi += 1; // non-physical Sw — refuse, don't fold into BVW
                    continue;
                }
                if s <= 0.0 {
                    ex_sw_lo += 1;
                    continue;
                }
                let rt = rv
                    .and_then(|r| r.get(i))
                    .map(|v| *v as f64)
                    .filter(|v| v.is_finite())
                    .map(|v| v.round() as i32);
                samples.push((well_id.clone(), t, p * s, rt));
            }
            } // 'well
            if samples.len() == before {
                empty_wells.push(well_id.clone());
            }
        }
    }
    // A scoped well that contributed NOTHING must be called out — a "field-wide" FOIL fitted
    // from a subset of the scoped wells is not field-wide.
    if !empty_wells.is_empty() {
        let shown: Vec<String> = empty_wells.iter().take(8).cloned().collect();
        let more = if empty_wells.len() > 8 { format!(" (+{} more)", empty_wells.len() - 8) } else { String::new() };
        notes.push(format!(
            "{} of {} scoped wells contributed no usable samples: {}{}",
            empty_wells.len(),
            req.well_ids.len(),
            shown.join(", "),
            more
        ));
    }
    let base_excluded = |ex_below_fwl: usize| -> Vec<(String, usize)> {
        let mut out = Vec::new();
        for (reason, cnt) in [
            ("below φ cutoff", ex_phi),
            ("Sw > 1 (non-physical)", ex_sw_hi),
            ("Sw ≤ 0", ex_sw_lo),
            ("at/below the FWL", ex_below_fwl),
        ] {
            if cnt > 0 {
                out.push((reason.to_string(), cnt));
            }
        }
        out
    };
    if samples.len() < 2 {
        // Keep the exclusion breakdown + notes on the error — they say WHY the cloud vanished.
        let mut res = foil_err("not enough complete PHIE/SW/TVDSS samples above the porosity cutoff");
        res.excluded = base_excluded(0);
        res.notes = notes;
        return res;
    }

    // Optional FWL scan over a common contact; otherwise use the requested FWL.
    let mut scan_out = Vec::new();
    let mut fwl_best = None;
    let fwl_used = if req.scan {
        let ts: Vec<(f64, f64)> = samples.iter().map(|(_, t, b, _)| (*t, *b)).collect();
        let (curve, best) = foil_fwl_scan(&ts, req.scan_lo, req.scan_hi, req.scan_step);
        scan_out = curve.iter().map(|p| FwlScanOut { fwl: p.fwl, residual: p.residual }).collect();
        fwl_best = best;
        best.unwrap_or(req.fwl)
    } else {
        req.fwl
    };

    // Height above FWL and the pooled (all-samples) fit.
    let above: Vec<&(String, f64, f64, Option<i32>)> =
        samples.iter().filter(|(_, t, _, _)| fwl_used - t > 0.0).collect();
    let ex_below_fwl = samples.len() - above.len();
    let pts: Vec<(f64, f64)> = above.iter().map(|(_, t, b, _)| (fwl_used - t, *b)).collect();
    let Some(fit) = fit_foil(&pts) else {
        // Keep the breakdown here too — "everything sits at/below the FWL" is exactly what the
        // counters were built to explain.
        let mut res = foil_err("FOIL fit failed — no samples above the FWL, or heights all equal");
        res.excluded = base_excluded(ex_below_fwl);
        res.notes = notes;
        return res;
    };

    // Per-rock-type laws (over the same FWL). Groups that cannot fit are reported, not dropped.
    let mut groups: Vec<FoilGroupFit> = Vec::new();
    if !rt_name.is_empty() {
        let mut by_rt: BTreeMap<i32, Vec<(f64, f64)>> = BTreeMap::new();
        let mut no_rt = 0usize;
        for (_, t, b, rt) in above.iter() {
            match rt {
                Some(r) => by_rt.entry(*r).or_default().push((fwl_used - t, *b)),
                None => no_rt += 1,
            }
        }
        if no_rt > 0 {
            notes.push(format!("{no_rt} samples have no {rt_name} value and joined only the pooled fit"));
        }
        for (rt, gp) in by_rt {
            match fit_foil(&gp) {
                Some(f) => groups.push(FoilGroupFit { rt, a: f.a, b: f.b, r2: f.r2, n_points: f.n, error: None }),
                None => groups.push(FoilGroupFit {
                    rt,
                    a: f64::NAN,
                    b: f64::NAN,
                    r2: f64::NAN,
                    n_points: gp.len(),
                    error: Some("too few usable points for a FOIL fit".into()),
                }),
            }
        }
    }

    let excluded = base_excluded(ex_below_fwl);

    // Scatter for the crossplot (decimated).
    let stride = (above.len() / MAX_FOIL_POINTS).max(1);
    let points: Vec<FoilPoint> = above
        .iter()
        .step_by(stride)
        .map(|(w, t, b, rt)| FoilPoint { h: fwl_used - t, bvw: *b, well_id: w.clone(), rt: *rt })
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
        groups,
        excluded,
        notes,
        error: None,
    }
}

// --------------------------------------------------------------------------------------------
// DB-backed command for the height-domain forms (Brooks-Corey / Skelt-Harrison): pool the
// log-derived Sw-vs-H cloud and fit the chosen form. Returns the scatter + a sampled fit curve.
// --------------------------------------------------------------------------------------------

fn d_rho_w() -> f64 {
    1.0
}
fn d_rho_hc() -> f64 {
    0.7
}
fn d_ift_res() -> f64 {
    26.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShfFitRequest {
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
    pub well_ids: Vec<String>,
    pub phie_curve: String,
    pub sw_curve: String,
    pub tvdss_curve: String,
    pub fwl: f64,
    #[serde(default)]
    pub min_phi: f64,
    /// "brooks_corey" | "skelt" | "thomeer" | "leverett_j".
    pub method: String,
    /// Working permeability curve — required by leverett_j only ("" = unset).
    #[serde(default)]
    pub perm_curve: String,
    /// Fluid properties for the height→Pc→J conversion (leverett_j only). Defaults are Tier-A
    /// seeds, per-run overridable: water 1.0 g/cc; HC 0.7 g/cc (Techlog sand-summary default,
    /// techlog_ingest/FINDINGS.md §C); σ·cosθ 26 dyn/cm ≈ IP Cap_Pressure_Fluid_Prop_Defaults
    /// Res(Water-Oil) σ 30 dyn/cm · cos 30° (ip_ingest/C_toplevel_par_defaults.json). A vendor
    /// default is a seed, not field truth.
    #[serde(default = "d_rho_w")]
    pub rho_w: f64,
    #[serde(default = "d_rho_hc")]
    pub rho_hc: f64,
    #[serde(default = "d_ift_res")]
    pub ift_res: f64,
    /// Optional rock-type/facies curve — when set, ALSO fit one law per rounded RT class.
    #[serde(default)]
    pub rt_curve: String,
}

/// One pooled log sample above the FWL, ready for any of the height-domain fits.
struct Sample {
    well: String,
    h: f64,
    sw: f64,
    phi: f64,
    /// Leverett J of this sample (needs PERM + fluid props), None when unavailable.
    j: Option<f64>,
    /// √(k/φ) of this sample — kept so the display curve can use a representative value.
    sqrt_kphi: Option<f64>,
    rt: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ShfPoint {
    pub h: f64,
    pub sw: f64,
    pub well_id: String,
    /// Rounded rock-type class of the sample (null when no RT curve was supplied / RT is NaN).
    pub rt: Option<i32>,
}

/// One per-rock-type SHF law of the requested family.
#[derive(Debug, Serialize)]
pub struct ShfGroupFit {
    pub rt: i32,
    pub params: Vec<(String, f64)>,
    pub r2: f64,
    pub n_points: usize,
    /// Sampled fitted Sw(H) curve over this group's height range.
    pub curve: Vec<(f64, f64)>,
    pub error: Option<String>,
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
    /// Per-rock-type fits when an RT curve was supplied (ascending RT class).
    pub groups: Vec<ShfGroupFit>,
    /// (reason, count) of candidate samples excluded from the fit — never drop silently.
    pub excluded: Vec<(String, usize)>,
    pub notes: Vec<String>,
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
        groups: vec![],
        excluded: vec![],
        notes: vec![],
        error: Some(msg.to_string()),
    }
}

/// Fits one height-domain family over a set of samples; shared by the pooled fit and every
/// per-rock-type group. Returns (named params, R², n fitted, sampled Sw(H) display curve).
#[allow(clippy::type_complexity)]
fn fit_height_method(
    method: &str,
    samples: &[&Sample],
    req: &ShfFitRequest,
    // Height `h` here is a DEPTH DIFFERENCE, so it is in the project's depth unit — the
    // Leverett-J branch needs it in feet for the 0.433 psi/ft/SG constant.
    depth_unit: crate::units::DepthUnit,
) -> Result<(Vec<(String, f64)>, f64, usize, Vec<(f64, f64)>), String> {
    let pts: Vec<(f64, f64)> = samples.iter().map(|s| (s.h, s.sw)).collect();
    if pts.is_empty() {
        return Err("no usable samples".into());
    }
    let hmin = pts.iter().map(|&(h, _)| h).fold(f64::INFINITY, f64::min).max(1e-3);
    let hmax = pts.iter().map(|&(h, _)| h).fold(f64::NEG_INFINITY, f64::max);

    let (params, r2, n, model): (Vec<(String, f64)>, f64, usize, Box<dyn Fn(f64) -> f64>) = match method {
        "brooks_corey" => {
            let Some(f) = fit_brooks_corey(&pts) else {
                return Err("Brooks-Corey fit failed (too few points in the transition zone)".into());
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
                return Err("Skelt-Harrison fit failed".into());
            };
            let (a, b, c, d) = (f.a, f.b, f.c, f.d);
            (
                vec![("A".into(), f.a), ("B".into(), f.b), ("C".into(), f.c), ("D".into(), f.d)],
                f.r2,
                f.n,
                Box::new(move |h: f64| skelt_sw(a, b, c, d, h)),
            )
        }
        "thomeer" => {
            let Some(f) = fit_thomeer(&pts) else {
                return Err("Thomeer fit failed (too few usable points)".into());
            };
            let (swirr, hd, g) = (f.swirr, f.hd, f.g);
            (
                vec![("swirr".into(), f.swirr), ("hd".into(), f.hd), ("g".into(), f.g)],
                f.r2,
                f.n,
                Box::new(move |h: f64| thomeer_sw(swirr, hd, g, h)),
            )
        }
        "leverett_j" => {
            let jpts: Vec<(f64, f64)> = samples.iter().filter_map(|s| s.j.map(|j| (j, s.sw))).collect();
            if jpts.len() < 3 {
                return Err("Leverett-J needs ≥3 samples with valid permeability".into());
            }
            let Some((a, b, r2, n)) = fit_power_lnln(&jpts) else {
                return Err("Leverett-J fit failed (degenerate J range)".into());
            };
            // The fitted law is per-sample in J; a single display curve needs one representative
            // rock quality — use the median √(k/φ) of the fitted samples (echoed as a param).
            let mut kp: Vec<f64> = samples.iter().filter_map(|s| s.sqrt_kphi).collect();
            kp.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            let med = kp[kp.len() / 2];
            let (rho_w, rho_hc, ift) = (req.rho_w, req.rho_hc, req.ift_res);
            (
                vec![("A".into(), a), ("B".into(), b), ("sqrt_k_phi_med".into(), med)],
                r2,
                n,
                Box::new(move |h: f64| {
                    let pc = PSI_PER_FT_PER_SG * (rho_w - rho_hc) * crate::units::to_feet(h, depth_unit);
                    if pc <= 0.0 {
                        return 1.0;
                    }
                    let j = J_CONST * pc / ift * med;
                    (a * j.powf(b)).clamp(0.0, 1.0)
                }),
            )
        }
        other => return Err(format!("unknown SHF method '{other}'")),
    };

    // RMS of the Sw residual over the fitted samples — a readout the dialog shows next to R².
    // For Leverett-J this uses each sample's own J (the real model), not the median-√(k/φ)
    // display curve.
    let rms = if method == "leverett_j" {
        let a = params[0].1;
        let b = params[1].1;
        let (ss, cnt) = samples
            .iter()
            .filter_map(|s| s.j.map(|j| (s.sw - (a * j.powf(b)).clamp(0.0, 1.0)).powi(2)))
            .fold((0.0, 0usize), |(ss, c), e| (ss + e, c + 1));
        if cnt > 0 { (ss / cnt as f64).sqrt() } else { f64::NAN }
    } else {
        let (ss, cnt) = pts
            .iter()
            .map(|&(h, sw)| (sw - model(h).clamp(0.0, 1.0)).powi(2))
            .fold((0.0, 0usize), |(ss, c), e| (ss + e, c + 1));
        if cnt > 0 { (ss / cnt as f64).sqrt() } else { f64::NAN }
    };
    let mut params = params;
    params.push(("rms".into(), rms));

    // Sampled fit curve across the observed H range.
    let steps = 60usize;
    let curve: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            let h = hmin + (hmax - hmin) * i as f64 / steps as f64;
            (h, model(h).clamp(0.0, 1.0))
        })
        .collect();
    Ok((params, r2, n, curve))
}

/// Per-rock-type fits: one law of the requested family per rounded RT class. Groups whose fit
/// fails are reported with the failure, never silently dropped.
fn fit_groups(
    samples: &[Sample],
    method: &str,
    req: &ShfFitRequest,
    depth_unit: crate::units::DepthUnit,
) -> Vec<ShfGroupFit> {
    let mut by_rt: BTreeMap<i32, Vec<&Sample>> = BTreeMap::new();
    for s in samples {
        if let Some(rt) = s.rt {
            by_rt.entry(rt).or_default().push(s);
        }
    }
    by_rt
        .into_iter()
        .map(|(rt, gp)| match fit_height_method(method, &gp, req, depth_unit) {
            Ok((params, r2, n, curve)) => ShfGroupFit { rt, params, r2, n_points: n, curve, error: None },
            Err(e) => ShfGroupFit {
                rt,
                params: vec![],
                r2: f64::NAN,
                n_points: gp.len(),
                curve: vec![],
                error: Some(e),
            },
        })
        .collect()
}

/// Buckles sanity check (Buckles 1965, "Correlating and averaging connate water saturation
/// data"): above the transition zone BVW = φ·Sw should be roughly one constant. Looks at the
/// top-height quartile of the used samples; a wide relative spread flags an inconsistent
/// irreducible plateau (mixed rock types or residual shale effects) — a note, not a failure.
fn buckles_note(samples: &[Sample]) -> Option<String> {
    if samples.len() < 20 {
        return None;
    }
    let mut hs: Vec<f64> = samples.iter().map(|s| s.h).collect();
    hs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let h_q3 = hs[(hs.len() * 3) / 4];
    let mut bvw: Vec<f64> = samples.iter().filter(|s| s.h >= h_q3).map(|s| s.phi * s.sw).collect();
    if bvw.len() < 8 {
        return None;
    }
    bvw.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = bvw[bvw.len() / 2];
    let iqr = bvw[(bvw.len() * 3) / 4] - bvw[bvw.len() / 4];
    if med > 0.0 && iqr / med > 0.6 {
        Some(format!(
            "Buckles check: BVW in the top-height quartile is not one constant (median {med:.3}, IQR {iqr:.3}) — consider per-rock-type fits or a stricter φ cutoff"
        ))
    } else {
        None
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
    let leverett = req.method == "leverett_j";
    let perm = req.perm_curve.trim().to_uppercase();
    if leverett {
        if perm.is_empty() {
            return shf_err(&req.method, "Leverett-J needs a permeability curve");
        }
        if req.ift_res <= 0.0 {
            return shf_err(&req.method, "reservoir σ·cosθ must be positive");
        }
        if req.rho_w <= req.rho_hc {
            return shf_err(&req.method, "water density must exceed hydrocarbon density (Pc would be ≤ 0)");
        }
    }
    let rt_name = req.rt_curve.trim().to_uppercase();
    // `h` below is (FWL - TVDSS), a depth difference, so it carries the project's depth
    // unit. The Leverett-J Pc law is per foot of column, so it needs the unit to convert.
    let depth_unit = {
        let conn = db.lock().unwrap();
        match crate::units::require_project_depth_unit(&conn, "saturation-height fitting") {
            Ok(unit) => unit,
            Err(error) => return shf_err(&req.method, &error),
        }
    };

    // Pool samples above the FWL / porosity cutoff, counting what gets dropped and why.
    let mut samples: Vec<Sample> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let (mut ex_phi, mut ex_below_fwl, mut ex_sw_hi, mut ex_sw_lo, mut ex_no_perm) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut empty_wells: Vec<String> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let mut names = vec![phie.clone(), sw.clone(), tvdss.clone()];
        if leverett {
            names.push(perm.clone());
        }
        if !rt_name.is_empty() {
            names.push(rt_name.clone());
        }
        for well_id in &req.well_ids {
            let before = samples.len();
            'well: {
                let Ok((_d, cols)) = crate::equations::fetch_curve_frame_from_set(&conn, well_id, &names, req.input_set.as_deref(), None) else { break 'well };
                let (Some(pv), Some(sv), Some(tv)) = (cols.get(&phie), cols.get(&sw), cols.get(&tvdss)) else {
                    break 'well;
                };
            let kv = cols.get(&perm);
            let rv = cols.get(&rt_name);
            let n = pv.len().min(sv.len()).min(tv.len());
            for i in 0..n {
                let (p, s, t) = (pv[i] as f64, sv[i] as f64, tv[i] as f64);
                if !(p.is_finite() && s.is_finite() && t.is_finite()) {
                    continue; // incomplete inputs (outside the computed interval) — not a candidate
                }
                if !(p > req.min_phi && p > 0.0) {
                    ex_phi += 1;
                    continue;
                }
                let h = req.fwl - t;
                if h <= 0.0 {
                    ex_below_fwl += 1;
                    continue;
                }
                if s > 1.0 {
                    ex_sw_hi += 1; // playbook: refuse non-physical Sw, and say so
                    continue;
                }
                if s <= 0.0 {
                    ex_sw_lo += 1;
                    continue;
                }
                // Leverett J for this sample (log-derived): Pc from height, J from PERM/PHIE.
                let (j, sqrt_kphi) = if leverett {
                    let k = kv.and_then(|v| v.get(i)).map(|v| *v as f64).unwrap_or(f64::NAN);
                    if k.is_finite() && k > 0.0 {
                        let pc =
                            PSI_PER_FT_PER_SG * (req.rho_w - req.rho_hc) * crate::units::to_feet(h, depth_unit);
                        let skp = (k / p).sqrt();
                        (Some(J_CONST * pc / req.ift_res * skp), Some(skp))
                    } else {
                        ex_no_perm += 1;
                        continue; // a Leverett fit cannot use a perm-less sample
                    }
                } else {
                    (None, None)
                };
                let rt = rv
                    .and_then(|r| r.get(i))
                    .map(|v| *v as f64)
                    .filter(|v| v.is_finite())
                    .map(|v| v.round() as i32);
                samples.push(Sample { well: well_id.clone(), h, sw: s, phi: p, j, sqrt_kphi, rt });
            }
            } // 'well
            if samples.len() == before {
                empty_wells.push(well_id.clone());
            }
        }
    }
    // A scoped well that contributed NOTHING (curve absent under the chosen names, or entirely
    // outside the FWL/cutoff window) must be called out — a "field-wide" law fitted from a
    // subset of the scoped wells is not field-wide.
    if !empty_wells.is_empty() {
        let shown: Vec<String> = empty_wells.iter().take(8).cloned().collect();
        let more = if empty_wells.len() > 8 { format!(" (+{} more)", empty_wells.len() - 8) } else { String::new() };
        notes.push(format!(
            "{} of {} scoped wells contributed no usable samples: {}{}",
            empty_wells.len(),
            req.well_ids.len(),
            shown.join(", "),
            more
        ));
    }
    let mut excluded: Vec<(String, usize)> = Vec::new();
    for (reason, cnt) in [
        ("below φ cutoff", ex_phi),
        ("at/below the FWL", ex_below_fwl),
        ("Sw > 1 (non-physical)", ex_sw_hi),
        ("Sw ≤ 0", ex_sw_lo),
        ("no permeability (Leverett-J)", ex_no_perm),
    ] {
        if cnt > 0 {
            excluded.push((reason.to_string(), cnt));
        }
    }
    if samples.len() < 4 {
        // Keep the exclusion breakdown + notes on the error — they say WHY the cloud vanished.
        let mut res = shf_err(&req.method, "not enough Sw samples above the FWL / porosity cutoff");
        res.excluded = excluded;
        res.notes = notes;
        return res;
    }

    // Pooled (all-samples) fit of the requested family.
    let all: Vec<&Sample> = samples.iter().collect();
    let (params, r2, n, curve) = match fit_height_method(&req.method, &all, req, depth_unit) {
        Ok(out) => out,
        Err(e) => {
            let mut res = shf_err(&req.method, &e);
            res.excluded = excluded;
            res.notes = notes;
            return res;
        }
    };

    // Per-rock-type laws + honesty notes.
    let mut groups: Vec<ShfGroupFit> = Vec::new();
    if !rt_name.is_empty() {
        let no_rt = samples.iter().filter(|s| s.rt.is_none()).count();
        if no_rt > 0 {
            notes.push(format!("{no_rt} samples have no {rt_name} value and joined only the pooled fit"));
        }
        groups = fit_groups(&samples, &req.method, req, depth_unit);
    }
    if let Some(nb) = buckles_note(&samples) {
        notes.push(nb);
    }

    // Decimated scatter.
    let stride = (samples.len() / MAX_FOIL_POINTS).max(1);
    let points: Vec<ShfPoint> = samples
        .iter()
        .step_by(stride)
        .map(|s| ShfPoint { h: s.h, sw: s.sw, well_id: s.well.clone(), rt: s.rt })
        .collect();

    ShfFitResult {
        method: req.method.clone(),
        params,
        r2,
        n_points: n,
        points,
        curve,
        groups,
        excluded,
        notes,
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
        assert!(fit_thomeer(&[(10.0, 0.5), (20.0, 0.4), (30.0, 0.3)]).is_none());
    }

    #[test]
    fn thomeer_recovers_known_g_and_entry_height() {
        // Sw = 1 − 0.8·exp(−0.8 / log10(H/3)) sampled exactly over 4–900 m. The fit must reach
        // an essentially perfect R² and land near the generating (Swirr, Hd, G).
        let (swirr, hd, g) = (0.2f64, 3.0f64, 0.8f64);
        let hs = [4.0f64, 6.0, 9.0, 14.0, 22.0, 35.0, 55.0, 90.0, 150.0, 250.0, 500.0, 900.0];
        let pts: Vec<(f64, f64)> = hs.iter().map(|&h| (h, thomeer_sw(swirr, hd, g, h))).collect();
        let fit = fit_thomeer(&pts).expect("thomeer fit");
        assert!(fit.r2 > 0.999, "r2={}", fit.r2);
        assert!((fit.hd - hd).abs() < 1.5, "hd={}", fit.hd);
        assert!((fit.g - g).abs() < 0.4, "g={}", fit.g);
        assert!((fit.swirr - swirr).abs() < 0.1, "swirr={}", fit.swirr);
        // Drainage shape: fitted Sw falls with height and Sw = 1 at/below the entry height.
        assert_eq!(thomeer_sw(fit.swirr, fit.hd, fit.g, fit.hd * 0.5), 1.0);
        assert!(thomeer_sw(fit.swirr, fit.hd, fit.g, 10.0) > thomeer_sw(fit.swirr, fit.hd, fit.g, 500.0));
    }

    #[test]
    fn thomeer_rejects_submillimetre_height_range_instead_of_panicking() {
        // All samples within 1 mm of the FWL (e.g. a constant curve mis-picked as TVDSS): the
        // Hd bounds would invert (lo > hi) and f64::clamp would panic — must return None.
        assert!(fit_thomeer(&[(5e-4, 0.9), (6e-4, 0.8), (7e-4, 0.7), (8e-4, 0.6)]).is_none());
    }

    #[test]
    fn leverett_j_cloud_fit_recovers_power_law() {
        // Sw = 0.4·J^−0.5 with J built exactly the way run_shf_fit builds it (height → Pc → J at
        // constant rock quality); the ln-ln regression must give the generating law back.
        let (a_true, b_true) = (0.4f64, -0.5f64);
        let (rho_w, rho_hc, ift) = (1.0f64, 0.7f64, 26.0f64);
        let sqrt_kphi = (200.0f64 / 0.25).sqrt();
        let mut pts = Vec::new();
        for i in 1..=40 {
            let h = i as f64 * 4.0;
            // This synthetic well's heights are metres, stated explicitly rather than
            // baked into a bare 3.28084 multiply.
            let pc = PSI_PER_FT_PER_SG
                * (rho_w - rho_hc)
                * crate::units::to_feet(h, crate::units::DepthUnit::Metres);
            let j = J_CONST * pc / ift * sqrt_kphi;
            let sw = a_true * j.powf(b_true);
            if sw < 1.0 {
                pts.push((j, sw));
            }
        }
        assert!(pts.len() >= 30, "synthetic cloud too small: {}", pts.len());
        let (a, b, r2, n) = fit_power_lnln(&pts).expect("lnln fit");
        assert!((a - a_true).abs() < 0.02, "A={a}");
        assert!((b - b_true).abs() < 0.02, "B={b}");
        assert!(r2 > 0.999, "r2={r2}");
        assert_eq!(n, pts.len());
    }

    fn dummy_req(method: &str) -> ShfFitRequest {
        ShfFitRequest {
            input_set: None,
            well_ids: vec![],
            phie_curve: "PHIE".into(),
            sw_curve: "SWE".into(),
            tvdss_curve: "TVDSS".into(),
            fwl: 0.0,
            min_phi: 0.0,
            method: method.into(),
            perm_curve: String::new(),
            rho_w: d_rho_w(),
            rho_hc: d_rho_hc(),
            ift_res: d_ift_res(),
            rt_curve: "RT".into(),
        }
    }

    #[test]
    fn per_rock_type_split_yields_two_distinct_laws() {
        // Two facies drawn from clearly different Brooks-Corey laws. Grouped fitting must
        // return one law per RT class and the fitted λ must be distinct — the playbook's
        // "≥2 distinct laws on a 2-facies synthetic" acceptance test.
        let mk = |rt: i32, swirr: f64, he: f64, lambda: f64, hs: &[f64]| -> Vec<Sample> {
            hs.iter()
                .map(|&h| Sample {
                    well: "W1".into(),
                    h,
                    sw: swirr + (1.0 - swirr) * (he / h).powf(lambda),
                    phi: 0.25,
                    j: None,
                    sqrt_kphi: None,
                    rt: Some(rt),
                })
                .collect()
        };
        let mut samples = mk(1, 0.10, 3.0, 0.45, &[4.0, 6.0, 9.0, 14.0, 22.0, 35.0, 55.0, 90.0]);
        samples.extend(mk(2, 0.30, 8.0, 1.3, &[10.0, 14.0, 20.0, 30.0, 45.0, 70.0, 110.0, 170.0]));
        let req = dummy_req("brooks_corey");
        let groups = fit_groups(&samples, "brooks_corey", &req, Default::default());
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|gp| gp.error.is_none()), "groups: {groups:?}");
        assert_eq!((groups[0].rt, groups[1].rt), (1, 2));
        let lam = |gp: &ShfGroupFit| gp.params.iter().find(|(k, _)| k == "lambda").expect("lambda").1;
        let (l1, l2) = (lam(&groups[0]), lam(&groups[1]));
        assert!((l1 - 0.45).abs() < 0.15, "λ1={l1}");
        assert!((l2 - 1.3).abs() < 0.35, "λ2={l2}");
        assert!((l2 - l1).abs() > 0.4, "laws must be distinct: {l1} vs {l2}");
        // Each group also carries its own display curve over its own height range.
        assert!(!groups[0].curve.is_empty() && !groups[1].curve.is_empty());
    }
}
