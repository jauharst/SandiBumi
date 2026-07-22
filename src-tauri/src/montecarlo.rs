//! Monte Carlo uncertainty (Phase 9, PT06). Puts probability distributions on chosen model
//! parameters (GR endpoints, matrix density, Archie a/m/n, Rw, …), runs N seeded realizations
//! of a module chain, and reports the resulting spread of net pay / NTG / average PHIE /
//! average SWE / hydrocarbon pore volume (HPV) per zone as P10/P50/P90 plus an HPV histogram.
//!
//! **Why it is fast and write-safe:** each realization runs the whole chain *in memory*
//! (`modules::run_module` returns curve vectors; nothing touches `computed_curves`), so the
//! single-writer DuckDB bottleneck measured for field-scale runs never applies. The only
//! database access is the one-time read of the input curves and the zonation up front. 1000
//! realizations across a typical well finish in well under a second. Realizations are
//! rayon-parallel and each is seeded deterministically from `(seed, realization_index)`, so a
//! given request is fully reproducible.
//!
//! Scope of this first increment: uncertain parameters are sampled *well-wide* (one draw per
//! realization, broadcast over depth); non-uncertain parameters still follow the well's zone
//! parameters exactly like a deterministic run. Per-zone parameter distributions and
//! persisted P10/P50/P90 *curves* are deliberate follow-ups — the headline deliverable here is
//! the zonal HPV/pay distribution a client wants.
//!
//! **Sampling (playbook #1.1):** draws are precomputed as an N×P matrix before the realization
//! loop. `sampling: "lhs"` (the default) uses Latin Hypercube Sampling — each parameter's unit
//! interval is split into N equal-probability strata, one jittered draw per stratum, stratum
//! order permuted (McKay, Beckman & Conover 1979, Technometrics 21(2)). `"random"` reproduces
//! the legacy independent-draw sequence byte-for-byte. Optional rank correlations between
//! parameters are induced distribution-free with the Iman–Conover method (Iman & Conover 1982,
//! Commun. Stat. B11(3)): van der Waerden scores are re-colored to the target correlation via
//! Cholesky, then each parameter's draws are rank-matched to its score column — marginals are
//! only reordered, never altered. `converge: true` evaluates realizations in batches and tracks
//! running low/mid/high percentiles of per-well total HPV; in random mode the run stops early
//! once the last checks are stationary (LHS never truncates — the stratified design is only
//! valid at its full size).

use crate::chain::ChainStep;
use crate::db::{self, ZoneEntry, ZoneParamEntry};
use crate::equations;
use crate::modules::{self, ArgKind, ModuleContext};
use duckdb::Connection;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Distribution attached to one uncertain parameter.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Distribution {
    Normal { mean: f64, sd: f64 },
    Uniform { lo: f64, hi: f64 },
    Triangular { lo: f64, mode: f64, hi: f64 },
}

impl Distribution {
    fn sample(&self, rng: &mut Rng) -> f64 {
        match *self {
            Distribution::Normal { mean, sd } => mean + sd * rng.normal(),
            Distribution::Uniform { lo, hi } => lo + (hi - lo) * rng.unit(),
            Distribution::Triangular { lo, mode, hi } => triangular_quantile(lo, mode, hi, rng.unit()),
        }
    }

    /// Central value (median) — the tornado base case.
    fn central(&self) -> f64 {
        match *self {
            Distribution::Normal { mean, .. } => mean,
            Distribution::Uniform { lo, hi } => 0.5 * (lo + hi),
            Distribution::Triangular { lo, mode, hi } => triangular_quantile(lo, mode, hi, 0.5),
        }
    }

    /// Inverse CDF at probability `q` ∈ (0, 1) — the tornado low/high endpoints.
    fn quantile(&self, q: f64) -> f64 {
        let q = q.clamp(1e-6, 1.0 - 1e-6);
        match *self {
            Distribution::Normal { mean, sd } => mean + sd * probit(q),
            Distribution::Uniform { lo, hi } => lo + (hi - lo) * q,
            Distribution::Triangular { lo, mode, hi } => triangular_quantile(lo, mode, hi, q),
        }
    }
}

/// Inverse CDF of a triangular(lo, mode, hi) distribution at `q` ∈ [0, 1]. Also the
/// sampling transform (feed it a U(0,1) draw), so sampling and quantiles stay consistent.
fn triangular_quantile(lo: f64, mode: f64, hi: f64, q: f64) -> f64 {
    if hi <= lo {
        return lo;
    }
    let fc = ((mode - lo) / (hi - lo)).clamp(0.0, 1.0);
    if q < fc {
        lo + (q * (hi - lo) * (mode - lo)).sqrt()
    } else {
        hi - ((1.0 - q) * (hi - lo) * (hi - mode)).sqrt()
    }
}

/// Inverse standard-normal CDF (Acklam's rational approximation; |abs err| < 1.2e-9 over the
/// central region, refined nowhere-critical for our P10/P90 tornado endpoints).
fn probit(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02,
        1.383577518672690e+02, -3.066479806614716e+01, 2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02,
        6.680131188771972e+01, -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00,
        -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00, 3.754408661907416e+00,
    ];
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= phigh {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct McParam {
    /// Module parameter name to vary (e.g. "GR_MA"); applies to every step that has it.
    pub param: String,
    pub dist: Distribution,
}

/// How the N×P draw matrix is generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sampling {
    /// Latin Hypercube (default): N equal-probability strata per parameter, one jittered draw
    /// per stratum, stratum order permuted — the sample CDF matches the target far tighter
    /// than independent draws at the same N (McKay, Beckman & Conover 1979).
    #[default]
    Lhs,
    /// Legacy independent draws, byte-identical to the pre-LHS sequence for a given seed.
    Random,
}

/// Target Spearman rank correlation between two Monte Carlo parameters, induced with the
/// Iman–Conover method. `rho` is clamped to ±0.995; pairs naming unknown parameters are
/// reported in `McResult.notes` and skipped.
#[derive(Debug, Clone, Deserialize)]
pub struct McCorrelation {
    pub param_a: String,
    pub param_b: String,
    pub rho: f64,
}

fn default_low_pctl() -> f64 {
    0.10
}
fn default_high_pctl() -> f64 {
    0.90
}
fn default_converge_tol() -> f64 {
    0.005
}

#[derive(Debug, Clone, Deserialize)]
pub struct McRequest {
    pub well_ids: Vec<String>,
    /// The deterministic chain to run each realization (same shape as a workflow chain).
    pub steps: Vec<ChainStep>,
    pub mc_params: Vec<McParam>,
    pub iterations: usize,
    pub seed: u64,
    // Pay/HPV cutoffs (same semantics as the pay summary).
    pub vsh_max: f64,
    pub phie_min: f64,
    pub swe_max: f64,
    #[serde(default)]
    pub perm_min: Option<f64>,
    /// HPV histogram bin count.
    pub bins: usize,
    /// Low / high output percentiles as fractions in (0, 1) — default 0.10 / 0.90. One control
    /// drives both the reported spread (`Pctl.lo` / `Pctl.hi`) and the tornado's one-at-a-time
    /// input sweep, so P10/P90, P5/P95, P1/P99 … all stay consistent. The median (`Pctl.mid`) is
    /// always reported.
    #[serde(default = "default_low_pctl")]
    pub low_pctl: f64,
    #[serde(default = "default_high_pctl")]
    pub high_pctl: f64,
    /// Retain the per-realization draws and report Spearman rank correlation of each MC
    /// parameter against each output metric (global sensitivity). Off by default → the run is
    /// byte-for-byte identical to before.
    #[serde(default)]
    pub sensitivity: bool,
    /// Also run a one-at-a-time low/base/high sweep per parameter (base = each distribution's
    /// median, low/high = its P10/P90) with the others held at base — the classic tornado range.
    #[serde(default)]
    pub tornado: bool,
    /// Draw-matrix scheme; defaults to Latin Hypercube. `"random"` reproduces the legacy
    /// independent-draw results exactly.
    #[serde(default)]
    pub sampling: Sampling,
    /// Target rank correlations between parameter pairs (Iman–Conover; empty = independent).
    #[serde(default)]
    pub correlations: Vec<McCorrelation>,
    /// Track running low/mid/high convergence of per-well total HPV in batches; in `random`
    /// mode, stop early once the series is stationary. Off by default.
    #[serde(default)]
    pub converge: bool,
    /// Relative stationarity tolerance for the convergence check (default 0.005 = 0.5%).
    #[serde(default = "default_converge_tol")]
    pub converge_tol: f64,
}

/// Low / median / high percentile (at the request's `low_pctl` / 0.50 / `high_pctl`) plus mean/sd
/// for one metric across realizations.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Pctl {
    pub lo: f32,
    pub mid: f32,
    pub hi: f32,
    pub mean: f32,
    pub sd: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct McZoneResult {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub top: f32,
    pub bottom: f32,
    pub gross: f32,
    pub iterations: usize,
    pub net: Pctl,
    pub ntg: Pctl,
    pub avg_phie: Pctl,
    pub avg_swe: Pctl,
    pub hpv: Pctl,
    /// HPV histogram: `bins` counts spanning [`hist_lo`, `hist_lo` + bins*`hist_w`].
    pub hpv_hist: Vec<u32>,
    pub hist_lo: f32,
    pub hist_w: f32,
}

/// One output-metric bundle (net pay / NTG / avg PHIE / avg SWE / HPV) for a single realization
/// or sweep point — the unit the tornado and Spearman results are expressed in.
#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct MetricSet {
    pub net: f32,
    pub ntg: f32,
    pub avg_phie: f32,
    pub avg_swe: f32,
    pub hpv: f32,
}

/// Sensitivity of one Monte Carlo parameter, within one zone.
#[derive(Debug, Clone, Serialize)]
pub struct SensParam {
    pub param: String,
    /// Spearman rank correlation of the sampled parameter vs each output metric across all
    /// realizations (−1..+1; NaN when the parameter or the metric has no spread). `None` unless
    /// `sensitivity` was requested.
    pub spearman: Option<MetricSet>,
    /// One-at-a-time sweep: output metrics with this parameter at its P10 / median / P90, all
    /// other MC parameters held at their medians. `None` unless `tornado` was requested.
    pub oat_low: Option<MetricSet>,
    pub oat_base: Option<MetricSet>,
    pub oat_high: Option<MetricSet>,
}

/// Per-zone parameter-sensitivity block, parallel to [`McZoneResult`].
#[derive(Debug, Clone, Serialize)]
pub struct McSensZone {
    pub well_id: String,
    pub well_name: String,
    pub zone: String,
    pub params: Vec<SensParam>,
}

/// One convergence checkpoint: running low/mid/high percentiles of per-well total HPV after
/// `at` realizations.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ConvCheck {
    pub at: u32,
    pub lo: f32,
    pub mid: f32,
    pub hi: f32,
}

/// Per-well convergence trace (present only when `converge` was requested).
#[derive(Debug, Clone, Serialize)]
pub struct McConvergence {
    pub well_id: String,
    pub well_name: String,
    /// Running-percentile checkpoints — the UI sparkline.
    pub checks: Vec<ConvCheck>,
    /// True when the trailing checkpoints were stationary within `converge_tol`.
    pub converged: bool,
    pub used_iterations: usize,
    pub requested_iterations: usize,
    /// Explains an early stop, or why one was not allowed (LHS designs never truncate).
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McResult {
    pub zones: Vec<McZoneResult>,
    /// Per-zone parameter sensitivity (empty unless `sensitivity` or `tornado` was requested).
    pub sensitivity: Vec<McSensZone>,
    /// The output percentiles actually used (echoed from the request, clamped) so the UI can label
    /// the lo/hi columns — e.g. 0.10 / 0.90.
    pub low_pctl: f64,
    pub high_pctl: f64,
    /// Sampling scheme actually used ("lhs" / "random"), echoed for the UI badge.
    pub sampling: String,
    /// Per-well convergence traces (empty unless `converge` was requested).
    pub convergence: Vec<McConvergence>,
    /// Non-fatal advisories (skipped correlation pairs, degenerate targets, …).
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Deterministic PRNG (SplitMix64 + Box–Muller) — dependency-free, seedable.
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Standard normal via Box–Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------------------
// Draw matrix: LHS / random sampling + Iman–Conover rank-correlation induction.
// ---------------------------------------------------------------------------

/// Precomputes the full N×P draw matrix (`draws[realization][param]`). Random mode replays the
/// legacy per-index RNG sequence exactly, so an uncorrelated `"random"` run is byte-identical
/// to the pre-LHS implementation. LHS draws one jittered sample per equal-probability stratum
/// and permutes the stratum order per parameter (independent RNG stream per column).
fn build_draws(
    params: &[McParam],
    n: usize,
    seed: u64,
    sampling: Sampling,
    correlations: &[McCorrelation],
    notes: &mut Vec<String>,
) -> Vec<Vec<f64>> {
    let p = params.len();
    let mut draws = vec![vec![0.0f64; p]; n];
    match sampling {
        Sampling::Random => {
            for (r, row) in draws.iter_mut().enumerate() {
                let mut rng = Rng::new(seed ^ (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
                for (j, prm) in params.iter().enumerate() {
                    row[j] = prm.dist.sample(&mut rng);
                }
            }
        }
        Sampling::Lhs => {
            let nn = n as f64;
            for (j, prm) in params.iter().enumerate() {
                let mut rng = Rng::new(seed ^ (j as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407));
                // One jittered quantile per stratum i: u ∈ [i/n, (i+1)/n).
                let mut u: Vec<f64> = (0..n).map(|i| (i as f64 + rng.unit()) / nn).collect();
                // Fisher–Yates permutation so strata pair randomly across parameters.
                for i in (1..n).rev() {
                    let k = (rng.next_u64() % (i as u64 + 1)) as usize;
                    u.swap(i, k);
                }
                for (r, ur) in u.iter().enumerate() {
                    draws[r][j] = prm.dist.quantile(*ur);
                }
            }
        }
    }
    if !correlations.is_empty() {
        if p >= 2 && n >= 10 {
            iman_conover(&mut draws, params, correlations, seed, notes);
        } else {
            notes.push("correlation targets ignored: need at least 2 MC parameters and 10 iterations".into());
        }
    }
    draws
}

/// Iman–Conover (1982) distribution-free rank-correlation induction. Builds a van der Waerden
/// score matrix with the target correlation (Cholesky re-coloring, targets pre-adjusted by the
/// Spearman→Pearson map 2·sin(πρ/6) so the achieved rank correlation centers on the request),
/// then reorders each parameter's draws so their ranks match the score column's ranks. Values
/// are permuted within a column, never altered — every marginal distribution survives exactly.
fn iman_conover(
    draws: &mut [Vec<f64>],
    params: &[McParam],
    corrs: &[McCorrelation],
    seed: u64,
    notes: &mut Vec<String>,
) {
    let n = draws.len();
    let p = params.len();

    // Target correlation matrix: identity plus the requested pairs.
    let mut c = vec![vec![0.0f64; p]; p];
    for (i, row) in c.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    let mut any = false;
    let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
    for cr in corrs {
        let ia = params.iter().position(|q| q.param == cr.param_a);
        let ib = params.iter().position(|q| q.param == cr.param_b);
        match (ia, ib) {
            (Some(a), Some(b)) if a != b => {
                if !seen_pairs.insert((a.min(b), a.max(b))) {
                    notes.push(format!(
                        "correlation pair '{}' / '{}' specified more than once — the last entry (ρ = {:.3}) wins",
                        cr.param_a, cr.param_b, cr.rho
                    ));
                }
                let rho_s = cr.rho.clamp(-0.995, 0.995);
                // Rank-matching to normal scores attenuates a Pearson target ρ to (6/π)·asin(ρ/2)
                // in rank space; pre-adjust with the inverse map so the ACHIEVED Spearman centers
                // on the requested value instead of landing systematically low (Iman & Conover
                // 1982, the Spearman↔Pearson conversion for normal scores).
                let rho = 2.0 * (std::f64::consts::PI * rho_s / 6.0).sin();
                c[a][b] = rho;
                c[b][a] = rho;
                any = true;
            }
            _ => notes.push(format!(
                "correlation pair '{}' / '{}' ignored (parameter not in the study, or a self-pair)",
                cr.param_a, cr.param_b
            )),
        }
    }
    if !any {
        return;
    }
    let Some(l) = cholesky(&c) else {
        notes.push("correlation targets are jointly inconsistent (matrix not positive-definite); correlations skipped".into());
        return;
    };

    // Score matrix M: van der Waerden scores Φ⁻¹(i/(n+1)), independently shuffled per column.
    let mut m = vec![vec![0.0f64; p]; n];
    for j in 0..p {
        let mut scores: Vec<f64> = (0..n).map(|i| probit((i as f64 + 1.0) / (n as f64 + 1.0))).collect();
        let mut rng = Rng::new(seed ^ (j as u64 + 101).wrapping_mul(0xD6E8_FEB8_6659_FD93));
        for i in (1..n).rev() {
            let k = (rng.next_u64() % (i as u64 + 1)) as usize;
            scores.swap(i, k);
        }
        for (i, s) in scores.into_iter().enumerate() {
            m[i][j] = s;
        }
    }

    // Re-color: T = M·Q⁻ᵀ·Lᵀ where E = corr(M) = QQᵀ removes the shuffle's incidental
    // correlation, then L imprints the target: corr(T) = L·Q⁻¹·E·Q⁻ᵀ·Lᵀ = L·Lᵀ = C.
    let e = corr_matrix(&m);
    let Some(q) = cholesky(&e) else {
        notes.push("Iman–Conover score matrix degenerate; correlations skipped".into());
        return;
    };
    let mut t = vec![vec![0.0f64; p]; n];
    for i in 0..n {
        let z = forward_solve(&q, &m[i]);
        for a in 0..p {
            let mut s = 0.0;
            for (b, zb) in z.iter().enumerate().take(a + 1) {
                s += l[a][b] * zb;
            }
            t[i][a] = s;
        }
    }

    // Rank-match: the row holding the k-th smallest score gets the k-th smallest draw.
    for j in 0..p {
        let mut vals: Vec<f64> = draws.iter().map(|r| r[j]).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut idx: Vec<usize> = (0..n).collect();
        idx.sort_by(|&a, &b| t[a][j].partial_cmp(&t[b][j]).unwrap_or(std::cmp::Ordering::Equal));
        for (k, &row) in idx.iter().enumerate() {
            draws[row][j] = vals[k];
        }
    }
}

/// Cholesky factor L (lower-triangular, A = L·Lᵀ) of a small symmetric matrix; None when the
/// matrix is not positive-definite.
fn cholesky(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let p = a.len();
    let mut l = vec![vec![0.0f64; p]; p];
    for i in 0..p {
        for j in 0..=i {
            let mut s = a[i][j];
            for k in 0..j {
                s -= l[i][k] * l[j][k];
            }
            if i == j {
                if s <= 1e-12 {
                    return None;
                }
                l[i][j] = s.sqrt();
            } else {
                l[i][j] = s / l[j][j];
            }
        }
    }
    Some(l)
}

/// Solves L·x = b for lower-triangular L by forward substitution.
fn forward_solve(l: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let p = b.len();
    let mut x = vec![0.0f64; p];
    for i in 0..p {
        let mut s = b[i];
        for k in 0..i {
            s -= l[i][k] * x[k];
        }
        x[i] = s / l[i][i];
    }
    x
}

/// Pearson correlation matrix of the columns of a row-major N×P matrix.
fn corr_matrix(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = m.len();
    let p = m[0].len();
    let mut mean = vec![0.0f64; p];
    for row in m {
        for (j, v) in row.iter().enumerate() {
            mean[j] += v;
        }
    }
    for v in mean.iter_mut() {
        *v /= n as f64;
    }
    let mut cov = vec![vec![0.0f64; p]; p];
    for row in m {
        for a in 0..p {
            let da = row[a] - mean[a];
            for b in 0..=a {
                cov[a][b] += da * (row[b] - mean[b]);
            }
        }
    }
    let mut c = vec![vec![0.0f64; p]; p];
    for a in 0..p {
        for b in 0..=a {
            let den = (cov[a][a] * cov[b][b]).sqrt();
            let v = if den > 0.0 {
                cov[a][b] / den
            } else if a == b {
                1.0
            } else {
                0.0
            };
            c[a][b] = v;
            c[b][a] = v;
        }
    }
    c
}

/// Minimum realizations before an early stop may trigger.
const CONV_MIN_ITER: usize = 200;

/// True when the trailing three checkpoints agree within `tol` relative to the running median
/// (or the band width, whichever is larger) — the "stationary series" criterion.
fn conv_stable(checks: &[ConvCheck], tol: f64) -> bool {
    let k = checks.len();
    if k < 4 {
        return false;
    }
    (k - 2..k).all(|i| {
        let a = checks[i - 1];
        let b = checks[i];
        let scale = (b.mid.abs() as f64).max((b.hi - b.lo).abs() as f64).max(1e-9);
        let d = (b.lo - a.lo).abs().max((b.mid - a.mid).abs()).max((b.hi - a.hi).abs()) as f64;
        d.is_finite() && d <= tol * scale
    })
}

// ---------------------------------------------------------------------------
// Per-step execution plan, precomputed once per well (outside the realization loop).
// ---------------------------------------------------------------------------

struct StepPlan {
    module: String,
    opts: HashMap<String, String>,
    /// (arg name, resolved input curve name UPPERCASE).
    log_args: Vec<(String, String)>,
    param_args: Vec<String>,
    /// Zone-resolved base value arrays for parameters NOT under Monte Carlo.
    base_params: HashMap<String, Vec<f64>>,
}

/// Cutoffs bundled for the per-zone pay/HPV accumulation.
#[derive(Clone, Copy)]
struct Cutoffs {
    vsh_max: f64,
    phie_min: f64,
    swe_max: f64,
    perm_min: Option<f64>,
}

#[derive(Clone, Copy)]
struct ZoneMetrics {
    net: f32,
    ntg: f32,
    avg_phie: f32,
    avg_swe: f32,
    hpv: f32,
}

/// Zone-resolved parameter array: manifest/step base, then well-wide '*' then named zones.
fn resolve_zone_param(
    name: &str,
    base: f64,
    zones: &[ZoneEntry],
    zone_params: &[ZoneParamEntry],
    depth: &[f32],
) -> Vec<f64> {
    let mut arr = vec![base; depth.len()];
    for zp in zone_params.iter().filter(|z| z.param_name == name) {
        if let (Some(v), "*") = (zp.value_num, zp.zone_name.as_str()) {
            arr.fill(v as f64);
        }
    }
    for zp in zone_params.iter().filter(|z| z.param_name == name) {
        let Some(v) = zp.value_num else { continue };
        if let Some(z) = zones.iter().find(|z| z.zone_name == zp.zone_name) {
            for (i, d) in depth.iter().enumerate() {
                if *d >= z.top_depth && *d < z.bottom_depth {
                    arr[i] = v as f64;
                }
            }
        }
    }
    arr
}

/// Per-zone net-pay / NTG / averages / HPV from a realization's output curves.
fn zone_metrics(
    vsh: &[f32],
    phie: &[f32],
    swe: &[f32],
    perm: &[f32],
    depth: &[f32],
    step: &[f32],
    zone: &ZoneEntry,
    cut: &Cutoffs,
    has_perm_cut: bool,
) -> ZoneMetrics {
    let mut net = 0.0f64;
    let mut sum_phie = 0.0f64;
    let mut sum_phie_swe = 0.0f64;
    let mut sum_phie_w = 0.0f64;
    let mut hpv = 0.0f64;
    for i in 0..depth.len() {
        // Clamp each sample's forward interval to its overlap with the zone (mirrors
        // run_pay_summary): the last in-zone sample no longer bleeds a full step past the
        // base, and net can never exceed gross — so MC P10/P50/P90 net/NTG/HPV agree with
        // the pay summary.
        let s_top = depth[i] as f64;
        let s_bot = (depth[i] + step[i]) as f64;
        let lo = s_top.max(zone.top_depth as f64);
        let hi = s_bot.min(zone.bottom_depth as f64);
        let h = hi - lo;
        if h <= 0.0 {
            continue;
        }
        let v = vsh[i] as f64;
        let p = phie[i] as f64;
        let s = swe[i] as f64;
        if v.is_nan() || p.is_nan() || s.is_nan() {
            continue;
        }
        let mut pay = v <= cut.vsh_max && p >= cut.phie_min && s <= cut.swe_max;
        if has_perm_cut {
            // A sample with no PERM value cannot demonstrate it passes the cutoff — missing
            // PERM must fail, not silently pass (matches run_pay_summary's classify_sample, so
            // MC and the pay summary agree for identical cutoffs).
            pay = pay && !perm[i].is_nan() && (perm[i] as f64) >= cut.perm_min.unwrap();
        }
        if !pay {
            continue;
        }
        net += h;
        sum_phie += p * h;
        sum_phie_swe += p * s * h;
        sum_phie_w += p * h;
        hpv += p * (1.0 - s) * h;
    }
    let gross = (zone.bottom_depth - zone.top_depth) as f64;
    ZoneMetrics {
        net: net as f32,
        ntg: if gross > 0.0 { (net / gross) as f32 } else { 0.0 },
        avg_phie: if net > 0.0 { (sum_phie / net) as f32 } else { f32::NAN },
        avg_swe: if sum_phie_w > 0.0 { (sum_phie_swe / sum_phie_w) as f32 } else { f32::NAN },
        hpv: hpv as f32,
    }
}

/// Percentile (linear interpolation, type-7) over finite values; NaN if none finite.
fn percentile(sorted: &[f32], p: f64) -> f32 {
    if sorted.is_empty() {
        return f32::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = p * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = (rank - lo as f64) as f32;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

fn summarize(values: &[f32], lo_p: f64, hi_p: f64) -> Pctl {
    let mut finite: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        // No-data metric (e.g. avg_phie/avg_swe in a dry zone) → NaN, matching percentile()'s
        // empty convention, so the UI renders "—" rather than a fabricated "0.00".
        return Pctl {
            lo: f32::NAN,
            mid: f32::NAN,
            hi: f32::NAN,
            mean: f32::NAN,
            sd: f32::NAN,
        };
    }
    finite.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = finite.len() as f64;
    let mean = finite.iter().map(|v| *v as f64).sum::<f64>() / n;
    let var = finite.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n;
    Pctl {
        lo: percentile(&finite, lo_p),
        mid: percentile(&finite, 0.50),
        hi: percentile(&finite, hi_p),
        mean: mean as f32,
        sd: var.sqrt() as f32,
    }
}

fn histogram(values: &[f32], bins: usize) -> (Vec<u32>, f32, f32) {
    let bins = bins.max(1);
    let finite: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return (vec![0; bins], 0.0, 0.0);
    }
    let lo = finite.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = finite.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut counts = vec![0u32; bins];
    if hi <= lo {
        counts[0] = finite.len() as u32; // degenerate: all identical
        return (counts, lo, 0.0);
    }
    let w = (hi - lo) / bins as f32;
    for v in finite {
        let mut b = ((v - lo) / w) as usize;
        if b >= bins {
            b = bins - 1;
        }
        counts[b] += 1;
    }
    (counts, lo, w)
}

// ---------------------------------------------------------------------------
// Sensitivity: Spearman rank correlation (dependency-free, average-rank ties).
// ---------------------------------------------------------------------------

/// Average (fractional) ranks of `v`, 1-based, ties sharing the mean of their positions.
fn average_ranks(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && v[idx[j]] == v[idx[i]] {
            j += 1;
        }
        // Positions i..j (0-based) → 1-based ranks (i+1)..=j; their mean is ((i+1)+j)/2.
        let avg = ((i + 1 + j) as f64) / 2.0;
        for &k in &idx[i..j] {
            ranks[k] = avg;
        }
        i = j;
    }
    ranks
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len();
    if n < 2 {
        return f64::NAN;
    }
    let inv = 1.0 / n as f64;
    let ma = a.iter().sum::<f64>() * inv;
    let mb = b.iter().sum::<f64>() * inv;
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let da = a[i] - ma;
        let db = b[i] - mb;
        sab += da * db;
        saa += da * da;
        sbb += db * db;
    }
    let den = (saa * sbb).sqrt();
    if den <= 0.0 {
        f64::NAN
    } else {
        sab / den
    }
}

/// Spearman rank correlation of parameter draws `x` against metric values `y`, over the
/// realizations where the metric is finite (draws are always finite). NaN if < 3 valid pairs
/// or either side has no spread.
fn spearman(x: &[f64], y: &[f32]) -> f32 {
    let mut xs = Vec::with_capacity(x.len());
    let mut ys = Vec::with_capacity(x.len());
    for (&a, &b) in x.iter().zip(y) {
        if a.is_finite() && b.is_finite() {
            xs.push(a);
            ys.push(b as f64);
        }
    }
    if xs.len() < 3 {
        return f32::NAN;
    }
    pearson(&average_ranks(&xs), &average_ranks(&ys)) as f32
}

/// Runs one in-memory realization at a fixed set of parameter values and returns the per-zone
/// output metrics — the workhorse for the one-at-a-time tornado sweep.
#[allow(clippy::too_many_arguments)]
fn metrics_for_values(
    plans: &[StepPlan],
    raw_pool: &HashMap<String, Vec<f32>>,
    depth: &[f32],
    step_thick: &[f32],
    zones: &[ZoneEntry],
    cut: &Cutoffs,
    has_perm_cut: bool,
    values: &HashMap<String, f64>,
    n: usize,
) -> Vec<MetricSet> {
    let pool = run_realization(plans, raw_pool, depth, values, n);
    let nanv = vec![f32::NAN; n];
    let vsh = pool.get("VSH").unwrap_or(&nanv);
    let phie = pool.get("PHIE").unwrap_or(&nanv);
    let swe = pool.get("SWE").unwrap_or(&nanv);
    let perm = pool.get("PERM").unwrap_or(&nanv);
    zones
        .iter()
        .map(|z| {
            let m = zone_metrics(vsh, phie, swe, perm, depth, step_thick, z, cut, has_perm_cut);
            MetricSet { net: m.net, ntg: m.ntg, avg_phie: m.avg_phie, avg_swe: m.avg_swe, hpv: m.hpv }
        })
        .collect()
}

/// Builds the per-step plan for one well: resolves inputs, options, and zone-resolved base
/// parameter arrays. Returns the plans plus the raw input-curve pool and depth/step arrays.
fn build_plans(
    conn: &Connection,
    well_id: &str,
    steps: &[ChainStep],
    specs: &HashMap<String, modules::ModuleSpec>,
) -> Result<(Vec<StepPlan>, HashMap<String, Vec<f32>>, Vec<f32>, Vec<f32>, Vec<ZoneEntry>), String> {
    // Curves produced somewhere in the chain don't need a DB read — they come from the pool.
    let mut produced: HashSet<String> = HashSet::new();
    for step in steps {
        let spec = specs.get(&step.module).ok_or_else(|| format!("unknown module '{}'", step.module))?;
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogOut) {
            produced.insert(a.name.to_uppercase());
        }
    }

    // External inputs = LogIn mnemonics not produced by any step.
    let mut external: HashSet<String> = HashSet::new();
    for step in steps {
        let spec = &specs[&step.module];
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
            let mnem = step.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            let up = mnem.trim().to_uppercase();
            if !produced.contains(&up) {
                external.insert(up);
            }
        }
    }

    let names: Vec<String> = external.iter().cloned().collect();
    let (depth, columns) = equations::fetch_curve_frame(conn, well_id, &names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("no curve data for well".into());
    }
    let n = depth.len();

    // Raw pool keyed UPPERCASE, matching how chain steps resolve their inputs.
    let mut raw_pool: HashMap<String, Vec<f32>> = HashMap::new();
    for name in &names {
        let v = columns.get(&name.to_uppercase()).cloned().unwrap_or_else(|| vec![f32::NAN; n]);
        raw_pool.insert(name.to_uppercase(), v);
    }

    let zones_raw = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zone_params = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;

    // Sample thickness: forward difference, last reuses previous step.
    let mut step_thick = vec![0.0f32; n];
    for i in 0..n {
        step_thick[i] = if i + 1 < n {
            depth[i + 1] - depth[i]
        } else if i > 0 {
            step_thick[i - 1]
        } else {
            0.0
        };
    }

    let mut plans = Vec::with_capacity(steps.len());
    for step in steps {
        let spec = &specs[&step.module];

        // Options: manifest defaults, then step overrides, plus __IN_<arg> mnemonics.
        let mut opts: HashMap<String, String> = spec
            .args
            .iter()
            .filter(|a| a.kind == ArgKind::Option)
            .map(|a| (a.name.clone(), a.default.clone()))
            .collect();
        for (k, v) in &step.opts {
            opts.insert(k.clone(), v.clone());
        }

        let mut log_args = Vec::new();
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
            let mnem = step.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            let up = mnem.trim().to_uppercase();
            opts.insert(format!("__IN_{}", a.name), up.clone());
            log_args.push((a.name.clone(), up));
        }

        let mut param_args = Vec::new();
        let mut base_params = HashMap::new();
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::Param) {
            param_args.push(a.name.clone());
            let base = step.params.get(&a.name).copied().or_else(|| a.default.parse().ok()).unwrap_or(f64::NAN);
            base_params.insert(a.name.clone(), resolve_zone_param(&a.name, base, &zones_raw, &zone_params, &depth));
        }

        plans.push(StepPlan { module: step.module.clone(), opts, log_args, param_args, base_params });
    }

    Ok((plans, raw_pool, depth, step_thick, zones_raw))
}

/// Runs the chain in memory for one realization and returns the resulting curve pool.
fn run_realization(
    plans: &[StepPlan],
    raw_pool: &HashMap<String, Vec<f32>>,
    depth: &[f32],
    mc_values: &HashMap<String, f64>,
    n: usize,
) -> HashMap<String, Vec<f32>> {
    let mut pool = raw_pool.clone();
    for plan in plans {
        let mut logs: HashMap<String, Vec<f32>> = HashMap::with_capacity(plan.log_args.len() + 1);
        logs.insert("DEPTH".to_string(), depth.to_vec());
        for (arg, mnem) in &plan.log_args {
            let v = pool.get(mnem).cloned().unwrap_or_else(|| vec![f32::NAN; n]);
            logs.insert(arg.clone(), v);
        }
        let mut params: HashMap<String, Vec<f64>> = HashMap::with_capacity(plan.param_args.len());
        for pname in &plan.param_args {
            if let Some(v) = mc_values.get(pname) {
                params.insert(pname.clone(), vec![*v; n]);
            } else if let Some(base) = plan.base_params.get(pname) {
                params.insert(pname.clone(), base.clone());
            }
        }
        let ctx = ModuleContext { n, logs, params, opts: plan.opts.clone() };
        if let Ok(outputs) = modules::run_module(&plan.module, &ctx) {
            for (k, v) in outputs {
                pool.insert(k.to_uppercase(), v);
            }
        }
        // A failed step just leaves the pool unchanged; downstream sees NaNs.
    }
    pool
}

/// Runs the Monte Carlo study across the requested wells. All computation is in memory; the
/// only DB access is the per-well input read in [`build_plans`].
pub fn run_monte_carlo(
    db: &Mutex<Connection>,
    req: &McRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> McResult {
    let iterations = req.iterations.clamp(1, 100_000);
    // One percentile pair drives the reported output spread AND the tornado's input sweep, so
    // the whole study stays consistent. Clamp to a sane open interval and keep lo < hi.
    let lo_p = req.low_pctl.clamp(0.001, 0.499);
    let hi_p = req.high_pctl.clamp(0.501, 0.999);
    let specs: HashMap<String, modules::ModuleSpec> =
        modules::list_modules().into_iter().map(|s| (s.name.clone(), s)).collect();
    let cut = Cutoffs {
        vsh_max: req.vsh_max,
        phie_min: req.phie_min,
        swe_max: req.swe_max,
        perm_min: req.perm_min,
    };

    let mut zones_out: Vec<McZoneResult> = Vec::new();
    let mut sens_out: Vec<McSensZone> = Vec::new();
    let mut conv_out: Vec<McConvergence> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let want_sens = req.sensitivity || req.tornado;
    let conv_tol = req.converge_tol.clamp(1e-5, 0.2);

    // The draw matrix depends only on (params, iterations, seed, sampling, correlations) — build
    // it once and share across wells. LHS stratification and Iman–Conover both need every
    // realization's draw jointly, which is why sampling is hoisted out of the rayon loop.
    let draws_all = build_draws(&req.mc_params, iterations, req.seed, req.sampling, &req.correlations, &mut notes);

    for (wi, well_id) in req.well_ids.iter().enumerate() {
        if let Some(p) = progress {
            if p.is_cancelled() {
                break;
            }
            p.set_current(Some(format!("Monte Carlo: well {}/{}", wi + 1, req.well_ids.len())));
            p.start_item(well_id);
        }
        // One-time DB phase: build plans + read inputs + zonation.
        let (plans, raw_pool, depth, step_thick, mut zones, well_name) = {
            let conn = db.lock().unwrap();
            let well_name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", duckdb::params![well_id], |r| r.get(0))
                .unwrap_or_else(|_| well_id.clone());
            match build_plans(&conn, well_id, &req.steps, &specs) {
                Ok((p, rp, d, st, z)) => (p, rp, d, st, z, well_name),
                Err(e) => {
                    if let Some(p) = progress {
                        p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                    }
                    errors.push(format!("{well_id}: {e}"));
                    continue;
                }
            }
        };
        let n = depth.len();
        if zones.is_empty() {
            zones.push(ZoneEntry { zone_name: "ALL".into(), top_depth: depth[0], bottom_depth: *depth.last().unwrap() });
        }
        let has_perm_cut =
            req.perm_min.is_some() && raw_pool.get("PERM").map(|c| c.iter().any(|v| !v.is_nan())).unwrap_or(false);

        // Parallel realizations over the precomputed draw matrix (ordered like `req.mc_params`,
        // so sensitivity can correlate draws against outputs; rows are empty when no parameters
        // vary). Random mode replays the legacy per-index sequence, so results are unchanged.
        let eval = |r: usize| -> (Vec<f64>, Vec<ZoneMetrics>) {
            let draws = &draws_all[r];
            let mc_values: HashMap<String, f64> =
                req.mc_params.iter().zip(draws).map(|(p, &v)| (p.param.clone(), v)).collect();
            let pool = run_realization(&plans, &raw_pool, &depth, &mc_values, n);
            let nanv = vec![f32::NAN; n];
            let vsh = pool.get("VSH").unwrap_or(&nanv);
            let phie = pool.get("PHIE").unwrap_or(&nanv);
            let swe = pool.get("SWE").unwrap_or(&nanv);
            let perm = pool.get("PERM").unwrap_or(&nanv);
            let zm = zones
                .iter()
                .map(|z| zone_metrics(vsh, phie, swe, perm, &depth, &step_thick, z, &cut, has_perm_cut))
                .collect();
            (draws.clone(), zm)
        };
        let per_real: Vec<(Vec<f64>, Vec<ZoneMetrics>)> = if req.converge {
            // Batched evaluation with running low/mid/high checkpoints of per-well total HPV.
            // Early stop is legal only in random mode — truncating an LHS design would leave
            // some strata unsampled and break the stratification guarantee.
            let batch = (iterations / 12).clamp(64, 512).min(iterations);
            // Checkpoint boundaries: even batches with the remainder folded into the FINAL one,
            // so every delta the stationarity verdict sees spans at least a full batch. A runt
            // tail of a few realizations would shift the running percentiles vacuously little
            // and inflate `converged` regardless of actual stationarity.
            let n_batches = (iterations / batch).max(1);
            let mut bounds: Vec<usize> = (1..=n_batches).map(|k| k * batch).collect();
            *bounds.last_mut().unwrap() = iterations;
            let mut acc: Vec<(Vec<f64>, Vec<ZoneMetrics>)> = Vec::with_capacity(iterations);
            let mut checks: Vec<ConvCheck> = Vec::new();
            let mut stopped_early = false;
            let mut done = 0usize;
            for &end in &bounds {
                let mut chunk: Vec<_> = (done..end).into_par_iter().map(eval).collect();
                acc.append(&mut chunk);
                done = end;
                let mut scal: Vec<f32> = acc
                    .iter()
                    .map(|m| m.1.iter().map(|z| z.hpv).sum::<f32>())
                    .filter(|v| v.is_finite())
                    .collect();
                scal.sort_by(|a, b| a.partial_cmp(b).unwrap());
                checks.push(ConvCheck {
                    at: done as u32,
                    lo: percentile(&scal, lo_p),
                    mid: percentile(&scal, 0.50),
                    hi: percentile(&scal, hi_p),
                });
                if req.sampling == Sampling::Random
                    && done >= CONV_MIN_ITER
                    && done < iterations
                    && conv_stable(&checks, conv_tol)
                {
                    stopped_early = true;
                    break;
                }
            }
            let converged = stopped_early || conv_stable(&checks, conv_tol);
            let note = if stopped_early {
                Some(format!("stationary after {done} of {iterations} realizations — stopped early"))
            } else if req.sampling == Sampling::Lhs {
                Some("LHS design size is fixed — early stop disabled; the trace is informational".into())
            } else if !converged {
                Some("not stationary at the final check — consider more iterations".into())
            } else {
                None
            };
            conv_out.push(McConvergence {
                well_id: well_id.clone(),
                well_name: well_name.clone(),
                checks,
                converged,
                used_iterations: acc.len(),
                requested_iterations: iterations,
                note,
            });
            acc
        } else {
            (0..iterations).into_par_iter().map(eval).collect()
        };
        let used_iterations = per_real.len();

        // One-at-a-time tornado sweep (once per well): base = medians of every MC parameter,
        // then each parameter swept to its P10/P90 with the rest held at base. Cheap — one chain
        // run per endpoint. `oat[param][zone]` for low/base/high.
        let (oat_base, oat_low, oat_high): (Vec<MetricSet>, Vec<Vec<MetricSet>>, Vec<Vec<MetricSet>>) =
            if req.tornado && !req.mc_params.is_empty() {
                let base_vals: HashMap<String, f64> =
                    req.mc_params.iter().map(|p| (p.param.clone(), p.dist.central())).collect();
                let base = metrics_for_values(&plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut, &base_vals, n);
                let mut low = Vec::with_capacity(req.mc_params.len());
                let mut high = Vec::with_capacity(req.mc_params.len());
                for p in &req.mc_params {
                    let mut lv = base_vals.clone();
                    lv.insert(p.param.clone(), p.dist.quantile(lo_p));
                    let mut hv = base_vals.clone();
                    hv.insert(p.param.clone(), p.dist.quantile(hi_p));
                    low.push(metrics_for_values(&plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut, &lv, n));
                    high.push(metrics_for_values(&plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut, &hv, n));
                }
                (base, low, high)
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            };

        // Transpose (realization × zone) → per-zone metric vectors, then summarize.
        for (zi, zone) in zones.iter().enumerate() {
            let net: Vec<f32> = per_real.iter().map(|m| m.1[zi].net).collect();
            let ntg: Vec<f32> = per_real.iter().map(|m| m.1[zi].ntg).collect();
            let avg_phie: Vec<f32> = per_real.iter().map(|m| m.1[zi].avg_phie).collect();
            let avg_swe: Vec<f32> = per_real.iter().map(|m| m.1[zi].avg_swe).collect();
            let hpv: Vec<f32> = per_real.iter().map(|m| m.1[zi].hpv).collect();

            // Per-zone parameter sensitivity: Spearman of each param's draws vs each metric, and
            // the one-at-a-time low/base/high tornado points captured above.
            if want_sens && !req.mc_params.is_empty() {
                let params = req
                    .mc_params
                    .iter()
                    .enumerate()
                    .map(|(pj, p)| {
                        let spearman = if req.sensitivity {
                            let draws: Vec<f64> = per_real.iter().map(|m| m.0[pj]).collect();
                            Some(MetricSet {
                                net: spearman(&draws, &net),
                                ntg: spearman(&draws, &ntg),
                                avg_phie: spearman(&draws, &avg_phie),
                                avg_swe: spearman(&draws, &avg_swe),
                                hpv: spearman(&draws, &hpv),
                            })
                        } else {
                            None
                        };
                        SensParam {
                            param: p.param.clone(),
                            spearman,
                            oat_low: oat_low.get(pj).map(|z| z[zi]),
                            oat_base: oat_base.get(zi).copied(),
                            oat_high: oat_high.get(pj).map(|z| z[zi]),
                        }
                    })
                    .collect();
                sens_out.push(McSensZone {
                    well_id: well_id.clone(),
                    well_name: well_name.clone(),
                    zone: zone.zone_name.clone(),
                    params,
                });
            }

            let (hpv_hist, hist_lo, hist_w) = histogram(&hpv, req.bins);
            zones_out.push(McZoneResult {
                well_id: well_id.clone(),
                well_name: well_name.clone(),
                zone: zone.zone_name.clone(),
                top: zone.top_depth,
                bottom: zone.bottom_depth,
                gross: zone.bottom_depth - zone.top_depth,
                iterations: used_iterations,
                net: summarize(&net, lo_p, hi_p),
                ntg: summarize(&ntg, lo_p, hi_p),
                avg_phie: summarize(&avg_phie, lo_p, hi_p),
                avg_swe: summarize(&avg_swe, lo_p, hi_p),
                hpv: summarize(&hpv, lo_p, hi_p),
                hpv_hist,
                hist_lo,
                hist_w,
            });
        }
        if let Some(p) = progress {
            p.finish_item(well_id, crate::jobs::ItemState::Ok, None);
        }
    }

    McResult {
        zones: zones_out,
        sensitivity: sens_out,
        low_pctl: lo_p,
        high_pctl: hi_p,
        sampling: match req.sampling {
            Sampling::Lhs => "lhs".into(),
            Sampling::Random => "random".into(),
        },
        convergence: conv_out,
        notes,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn step(module: &str) -> ChainStep {
        ChainStep { module: module.into(), log_inputs: HashMap::new(), params: HashMap::new(), opts: HashMap::new() }
    }

    /// A clean-ish sand: low GR, moderate porosity, low water saturation, so vsh_gr → phi_dn →
    /// sw_indo yields real pay and a positive HPV.
    fn seed_well(conn: &Connection) -> String {
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "MC_TEST", Some("Synthetic"), None, None).unwrap();
        let n = 300usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 25.0 + (i % 30) as f32).collect();
        let res: Vec<f32> = vec![20.0; n];
        let nphi: Vec<f32> = vec![0.20; n];
        let rhob: Vec<f32> = vec![2.35; n];
        let dt: Vec<f32> = vec![80.0; n];
        let sp: Vec<f32> = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, res, nphi, rhob, dt, sp).unwrap();
        id.to_string()
    }

    fn base_request(well: &str, mc: Vec<McParam>, iterations: usize, seed: u64) -> McRequest {
        McRequest {
            well_ids: vec![well.into()],
            steps: vec![step("vsh_gr"), step("phi_dn"), step("sw_indo")],
            mc_params: mc,
            iterations,
            seed,
            vsh_max: 0.5,
            phie_min: 0.08,
            swe_max: 0.6,
            perm_min: None,
            bins: 10,
            low_pctl: 0.10,
            high_pctl: 0.90,
            sensitivity: false,
            tornado: false,
            // Legacy sampling path by default so the pre-LHS assertions keep their meaning;
            // the LHS/correlation/convergence tests below opt in explicitly.
            sampling: Sampling::Random,
            correlations: Vec::new(),
            converge: false,
            converge_tol: 0.005,
        }
    }

    #[test]
    fn hpv_distribution_is_ordered_and_reproducible() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 } }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc.clone(), 500, 42), None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert_eq!(res.zones.len(), 1);
        let z = &res.zones[0];
        // Percentiles ordered, spread present, HPV positive, histogram counts all iterations.
        assert!(z.hpv.lo <= z.hpv.mid && z.hpv.mid <= z.hpv.hi, "HPV percentiles unordered: {:?}", z.hpv);
        assert!(z.hpv.hi > z.hpv.lo, "expected spread from GR_MA uncertainty");
        assert!(z.hpv.mid > 0.0, "expected positive HPV in a clean sand");
        assert!(z.net.mid > 0.0 && z.avg_phie.mid > 0.0 && z.avg_swe.mid > 0.0);
        assert_eq!(z.hpv_hist.iter().sum::<u32>(), 500);

        // Same seed → identical result (reproducible).
        let res2 = run_monte_carlo(&dbm, &base_request(&well, mc, 500, 42), None);
        assert_eq!(res.zones[0].hpv.mid, res2.zones[0].hpv.mid);
        assert_eq!(res.zones[0].hpv.mean, res2.zones[0].hpv.mean);
    }

    #[test]
    fn zero_variance_param_collapses_distribution() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        // sd = 0 → every realization identical → P10 == P50 == P90, sd ≈ 0.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 } }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 200, 7), None);
        let z = &res.zones[0];
        assert_eq!(z.hpv.lo, z.hpv.hi, "no variance should collapse the spread");
        assert!(z.hpv.sd.abs() < 1e-6, "sd should be ~0, got {}", z.hpv.sd);
    }

    #[test]
    fn uniform_and_triangular_sample_in_range() {
        let mut rng = Rng::new(123);
        for _ in 0..10_000 {
            let u = Distribution::Uniform { lo: 2.0, hi: 5.0 }.sample(&mut rng);
            assert!((2.0..=5.0).contains(&u), "uniform out of range: {u}");
            let t = Distribution::Triangular { lo: 0.0, mode: 3.0, hi: 4.0 }.sample(&mut rng);
            assert!((0.0..=4.0).contains(&t), "triangular out of range: {t}");
        }
    }

    #[test]
    fn sensitivity_off_by_default_and_reproducible() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 } }];
        // Default request has sensitivity=false → no sensitivity block, and the headline zones
        // are byte-identical to a run that DID ask for sensitivity (draws don't perturb the rng).
        let plain = run_monte_carlo(&dbm, &base_request(&well, mc.clone(), 400, 42), None);
        assert!(plain.sensitivity.is_empty(), "sensitivity should be empty when not requested");
        let mut with = base_request(&well, mc, 400, 42);
        with.sensitivity = true;
        let sens = run_monte_carlo(&dbm, &with, None);
        assert_eq!(plain.zones[0].hpv.mid, sens.zones[0].hpv.mid, "asking for sensitivity must not change the MC result");
    }

    #[test]
    fn spearman_ranks_a_monotone_parameter() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // GR_MA ↑ → VSH ↓ → PHIE ↑ → HPV ↑ : a strong positive, monotone influence.
        let mut req = base_request(
            &well,
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 6.0 } }],
            1000,
            11,
        );
        req.sensitivity = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert_eq!(res.sensitivity.len(), 1);
        let sp = res.sensitivity[0].params[0].spearman.expect("spearman requested");
        assert!(sp.hpv.is_finite() && sp.hpv > 0.5, "expected strong positive HPV sensitivity, got {}", sp.hpv);

        // Zero-variance parameter → no rank spread → NaN Spearman.
        let mut req0 = base_request(
            &well,
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 } }],
            300,
            11,
        );
        req0.sensitivity = true;
        let res0 = run_monte_carlo(&dbm, &req0, None);
        assert!(res0.sensitivity[0].params[0].spearman.unwrap().hpv.is_nan(), "no spread → NaN Spearman");
    }

    #[test]
    fn tornado_low_base_high_are_ordered() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        let mut req = base_request(
            &well,
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 6.0 } }],
            50,
            5,
        );
        req.tornado = true;
        let res = run_monte_carlo(&dbm, &req, None);
        let p = &res.sensitivity[0].params[0];
        let (lo, base, hi) = (p.oat_low.unwrap(), p.oat_base.unwrap(), p.oat_high.unwrap());
        // GR_MA at P10 (lower) → lower HPV; P90 → higher HPV; base in between.
        assert!(lo.hpv <= base.hpv + 1e-4 && base.hpv <= hi.hpv + 1e-4, "tornado HPV not ordered: {} {} {}", lo.hpv, base.hpv, hi.hpv);
        assert!(hi.hpv > lo.hpv, "expected a tornado spread, got low {} high {}", lo.hpv, hi.hpv);
        assert!(p.spearman.is_none(), "spearman should be absent when only tornado was requested");
    }

    #[test]
    fn request_without_new_fields_deserializes_with_lhs_defaults() {
        // The current dialog sends no sampling/correlations/converge fields — serde must fill
        // LHS + empty + off + 0.005 so pre-1.1 request shapes keep working over IPC.
        let json = r#"{
            "well_ids": ["w1"], "steps": [], "mc_params": [],
            "iterations": 100, "seed": 1,
            "vsh_max": 0.5, "phie_min": 0.08, "swe_max": 0.6, "perm_min": null, "bins": 10
        }"#;
        let req: McRequest = serde_json::from_str(json).expect("legacy request shape must parse");
        assert_eq!(req.sampling, Sampling::Lhs, "LHS is the documented default");
        assert!(req.correlations.is_empty());
        assert!(!req.converge);
        assert!((req.converge_tol - 0.005).abs() < 1e-12);
        // And the explicit strings round-trip the enum tags.
        let j2 = r#"{"well_ids":[],"steps":[],"mc_params":[],"iterations":1,"seed":1,
            "vsh_max":1,"phie_min":0,"swe_max":1,"perm_min":null,"bins":1,"sampling":"random"}"#;
        let r2: McRequest = serde_json::from_str(j2).unwrap();
        assert_eq!(r2.sampling, Sampling::Random);
    }

    #[test]
    fn lhs_stratifies_and_hits_the_analytic_mean() {
        // One Uniform(2,5) parameter, 200 LHS realizations: exactly one draw must land in each
        // of the 200 equal-probability strata, and the sample mean must sit on the analytic
        // mean far tighter than independent draws could at the same N.
        let params = vec![McParam { param: "X".into(), dist: Distribution::Uniform { lo: 2.0, hi: 5.0 } }];
        let mut notes = Vec::new();
        let draws = build_draws(&params, 200, 42, Sampling::Lhs, &[], &mut notes);
        assert!(notes.is_empty(), "unexpected notes: {notes:?}");
        let vals: Vec<f64> = draws.iter().map(|r| r[0]).collect();
        let mut counts = vec![0usize; 200];
        for v in &vals {
            assert!((2.0..=5.0).contains(v), "LHS uniform draw out of range: {v}");
            let cell = (((v - 2.0) / 3.0) * 200.0).floor() as usize;
            counts[cell.min(199)] += 1;
        }
        assert!(counts.iter().all(|&c| c == 1), "LHS must place exactly one draw per stratum");
        let mean = vals.iter().sum::<f64>() / 200.0;
        assert!((mean - 3.5).abs() < 0.01, "LHS mean {mean} should be ~3.5 (analytic)");
        // Same seed → identical matrix (reproducible).
        let again = build_draws(&params, 200, 42, Sampling::Lhs, &[], &mut notes);
        assert_eq!(draws, again, "LHS draws must be deterministic for a given seed");
    }

    #[test]
    fn iman_conover_induces_target_rank_correlation() {
        // Mixed marginals (normal + uniform) — rank correlation is distribution-free, so the
        // achieved Spearman must land on the target for both signs, and the marginals must be
        // pure reorderings of the uncorrelated draws.
        let params = vec![
            McParam { param: "A".into(), dist: Distribution::Normal { mean: 0.0, sd: 1.0 } },
            McParam { param: "B".into(), dist: Distribution::Uniform { lo: 10.0, hi: 20.0 } },
        ];
        let mut notes = Vec::new();
        let target = vec![McCorrelation { param_a: "A".into(), param_b: "B".into(), rho: 0.8 }];
        let draws = build_draws(&params, 500, 7, Sampling::Lhs, &target, &mut notes);
        assert!(notes.is_empty(), "unexpected notes: {notes:?}");
        let a: Vec<f64> = draws.iter().map(|r| r[0]).collect();
        let b: Vec<f32> = draws.iter().map(|r| r[1] as f32).collect();
        let rho = spearman(&a, &b);
        // The Spearman→Pearson pre-adjustment centers the achieved value on the target (without
        // it, 0.8 lands systematically near 0.786 — the (6/π)·asin(ρ/2) attenuation).
        assert!((rho - 0.8).abs() < 0.05, "achieved rank correlation {rho}, target 0.8");

        let neg = vec![McCorrelation { param_a: "A".into(), param_b: "B".into(), rho: -0.6 }];
        let d2 = build_draws(&params, 500, 7, Sampling::Lhs, &neg, &mut notes);
        let a2: Vec<f64> = d2.iter().map(|r| r[0]).collect();
        let b2: Vec<f32> = d2.iter().map(|r| r[1] as f32).collect();
        let rho2 = spearman(&a2, &b2);
        assert!((rho2 + 0.6).abs() < 0.05, "achieved rank correlation {rho2}, target -0.6");

        // Marginal preservation: sorting each column must reproduce the uncorrelated draws.
        let plain = build_draws(&params, 500, 7, Sampling::Lhs, &[], &mut notes);
        for j in 0..2 {
            let mut got: Vec<f64> = draws.iter().map(|r| r[j]).collect();
            let mut want: Vec<f64> = plain.iter().map(|r| r[j]).collect();
            got.sort_by(|x, y| x.partial_cmp(y).unwrap());
            want.sort_by(|x, y| x.partial_cmp(y).unwrap());
            assert_eq!(got, want, "Iman–Conover must only reorder draws, never change values");
        }

        // A pair naming an unknown parameter is skipped with a note, not an error.
        let bad = vec![McCorrelation { param_a: "A".into(), param_b: "NOPE".into(), rho: 0.5 }];
        let mut notes2 = Vec::new();
        build_draws(&params, 100, 7, Sampling::Lhs, &bad, &mut notes2);
        assert_eq!(notes2.len(), 1, "unknown-parameter pair should produce one note");

        // A conflicting duplicate (same unordered pair, reversed names) must NOT resolve
        // silently: the last entry wins AND a note says so.
        let dup = vec![
            McCorrelation { param_a: "A".into(), param_b: "B".into(), rho: 0.9 },
            McCorrelation { param_a: "B".into(), param_b: "A".into(), rho: -0.9 },
        ];
        let mut notes3 = Vec::new();
        let d3 = build_draws(&params, 400, 7, Sampling::Lhs, &dup, &mut notes3);
        assert_eq!(notes3.len(), 1, "duplicate pair should produce a note, got {notes3:?}");
        assert!(notes3[0].contains("more than once"), "note should name the conflict: {}", notes3[0]);
        let a3: Vec<f64> = d3.iter().map(|r| r[0]).collect();
        let b3: Vec<f32> = d3.iter().map(|r| r[1] as f32).collect();
        assert!(spearman(&a3, &b3) < -0.8, "the LAST entry (−0.9) must win");
    }

    #[test]
    fn convergence_early_stops_on_a_stationary_series() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // sd = 0 → every realization identical → the running percentiles are flat from the very
        // first batch → random-mode early stop long before the requested 5000.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 } }];
        let mut req = base_request(&well, mc, 5000, 42);
        req.converge = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert_eq!(res.convergence.len(), 1);
        let c = &res.convergence[0];
        assert!(c.converged, "a flat series must be reported as converged");
        assert!(c.used_iterations < 5000, "expected an early stop, used {}", c.used_iterations);
        assert!(c.used_iterations >= CONV_MIN_ITER);
        assert!(c.checks.len() >= 4, "need enough checkpoints to judge stationarity");
        // The rest of the result reflects the truncated run consistently.
        assert_eq!(res.zones[0].iterations, c.used_iterations);
        assert_eq!(res.zones[0].hpv_hist.iter().sum::<u32>() as usize, c.used_iterations);
    }

    #[test]
    fn lhs_design_never_truncates() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // Even a perfectly flat series must not shorten an LHS design — truncation would leave
        // strata unsampled. The trace is still reported for the sparkline.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 } }];
        let mut req = base_request(&well, mc, 1000, 42);
        req.sampling = Sampling::Lhs;
        req.converge = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert_eq!(res.sampling, "lhs");
        let c = &res.convergence[0];
        assert_eq!(c.used_iterations, 1000, "LHS must always run the full design");
        assert!(c.converged, "the flat series should still be reported as stationary");
        assert!(c.note.as_deref().unwrap_or("").contains("LHS"), "note should explain why no early stop");
        assert_eq!(res.zones[0].iterations, 1000);
        // No runt checkpoint: the remainder folds into the final batch, so every delta the
        // stationarity verdict judges spans at least one full batch (83 for 1000 iterations).
        assert_eq!(c.checks.last().unwrap().at, 1000);
        let mut prev = 0u32;
        for chk in &c.checks {
            assert!(chk.at - prev >= 64, "checkpoint span {} too small (runt batch)", chk.at - prev);
            prev = chk.at;
        }
    }

    #[test]
    fn configurable_percentiles_widen_the_spread() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 } }];

        // Default P10/P90.
        let base = run_monte_carlo(&dbm, &base_request(&well, mc.clone(), 1000, 42), None);
        assert_eq!(base.low_pctl, 0.10);
        assert_eq!(base.high_pctl, 0.90);
        let d80 = base.zones[0].hpv.hi - base.zones[0].hpv.lo;

        // P1/P99 — same draws (same seed), but a wider low↔high band and an unchanged median.
        let mut wide = base_request(&well, mc, 1000, 42);
        wide.low_pctl = 0.01;
        wide.high_pctl = 0.99;
        let res = run_monte_carlo(&dbm, &wide, None);
        assert_eq!(res.low_pctl, 0.01);
        assert_eq!(res.high_pctl, 0.99);
        let d98 = res.zones[0].hpv.hi - res.zones[0].hpv.lo;
        assert!(d98 > d80, "P1/P99 band ({d98}) should exceed P10/P90 band ({d80})");
        assert_eq!(base.zones[0].hpv.mid, res.zones[0].hpv.mid, "median must not depend on the chosen percentiles");
    }
}
