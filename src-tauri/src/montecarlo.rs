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
    /// Restrict this entry's draw to one named zone (None = well-wide, the default). The same
    /// parameter may appear in several entries with different zones; outside every scoped zone
    /// it follows its deterministic zone-resolved base values. Entries apply in list order, so
    /// a later entry wins where zones overlap. An unknown zone name leaves the parameter at
    /// base values and adds a note.
    #[serde(default)]
    pub zone: Option<String>,
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
    /// Persist per-sample uncertainty curves to a versioned "MONTECARLO" log set: for each
    /// tracked output the chain produces (VSH/PHIE/SWE/PERM), `MC_<KEY>_LOW` / `_P50` /
    /// `_HIGH` (per-sample percentiles at the request's low/0.50/high fractions across
    /// realizations) and `MC_<KEY>_BASE` (one deterministic run at every parameter's median).
    /// Samples with too few finite realizations stay NaN. Off by default.
    #[serde(default)]
    pub persist: bool,
    /// Also store the per-sample REALIZATION MATRIX itself into `array_logs` as
    /// `MC_<KEY>_REAL`, so the log view can draw an adjustable band, a spaghetti overlay or a
    /// density heat map from one stored run — the percentiles become a display setting you
    /// change instead of a reason to re-run the study. Requires `persist` (it rides the same
    /// pass over the kept realizations). Off by default: this is the only output here whose
    /// size scales with iteration count.
    #[serde(default)]
    pub persist_realizations: bool,
    /// How many realizations `persist_realizations` stores per depth (default 256, clamped to
    /// 8..=1024). The full kept set can reach 1024, which at ~2000 samples is ~8 MB per curve
    /// per well — 3 GB across a 100-well field, the exact kind of growth that produced the
    /// 2.5 GB field report. 256 draws put P10/P90 within a hair of the full set at a quarter
    /// the size; when the cap bites, the run says so rather than letting the band quietly
    /// disagree with the persisted MC_*_LOW/_HIGH curves.
    #[serde(default)]
    pub realization_cap: Option<u32>,
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

/// Physical-plausibility diagnostic for one well (playbook #1 residual, "reject/flag impossible
/// combos"). Counts how often a sampled parameter combination drove the petrophysics out of
/// physical bounds — the raw `Sw > 1` / `PHIE < 0` seen on the chain's UNLIMITED companion curves
/// (`PHIE_DN`, `SWT_ARCH`, `SWE_INDO`, …) before each module's limit clamps them into `[0, 1]`.
///
/// These realizations are REPORTED, never excluded: the clamp already gives an impossible draw the
/// physically-correct volumetric answer (an over-dense matrix → zero effective porosity, a
/// supersaturated combo → fully wet), so they remain valid low/high tails of the distribution —
/// dropping them would bias P10/P90. A large `fraction` instead means the input distributions are
/// straining physics and should be narrowed.
#[derive(Debug, Clone, Serialize)]
pub struct McPlausibility {
    pub well_id: String,
    pub well_name: String,
    /// Realizations with at least one physically-impossible in-zone sample.
    pub impossible_realizations: u32,
    /// Realizations evaluated for this well.
    pub realizations: u32,
    /// `impossible_realizations / realizations` (0 when no realizations ran).
    pub fraction: f32,
    /// False when the well produced no finite porosity/saturation samples to judge (e.g. a missing
    /// density log) — so the UI shows "not checked" instead of a fabricated clean pass.
    pub checked: bool,
    /// Human-readable breakdown, e.g. "Sw>1 in 41/500 (612 samples); PHIE<0 or >1 in 7/500 (9 samples)".
    pub detail: String,
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
    /// Curve names written to the versioned MONTECARLO log set (empty unless `persist`).
    pub persisted: Vec<String>,
    /// Per-well physical-plausibility diagnostics: the fraction of realizations whose sampled combo
    /// produced an impossible Sw>1 / PHIE<0 (reported, not excluded). Empty when the chain produces
    /// no porosity/saturation (v/v) curves.
    pub plausibility: Vec<McPlausibility>,
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
                // Zone-scoped duplicates share a name but draw independently; a name-keyed
                // target can only bind ONE of them, so say which.
                for nm in [&cr.param_a, &cr.param_b] {
                    if params.iter().filter(|q| &q.param == nm).count() > 1 {
                        let msg = format!(
                            "parameter '{nm}' appears in several zone-scoped entries — the ρ target binds only the first entry"
                        );
                        if !notes.contains(&msg) {
                            notes.push(msg);
                        }
                    }
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
            (Some(_), Some(_)) => notes.push(format!(
                "correlation pair '{}' / '{}' ignored — both names resolve to the same entry (zone-scoped entries of one parameter cannot yet be correlated with each other)",
                cr.param_a, cr.param_b
            )),
            _ => notes.push(format!(
                "correlation pair '{}' / '{}' ignored (parameter not in the study)",
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

/// Tolerance for the physical-plausibility bound check: a porosity/saturation sample counts as
/// impossible only when it leaves `[0, 1]` by more than this, so floating-point fuzz at exactly 0
/// or 1 (a limit landing on the boundary) never trips it.
const PHYS_TOL: f64 = 1e-3;

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
    /// The project's depth unit, captured here so the in-memory realization loop (which
    /// runs with no DB connection) can hand it to every `ModuleContext`.
    depth_unit: crate::units::DepthUnit,
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
/// output metrics — the workhorse for the one-at-a-time tornado sweep. `values` is indexed
/// like `mc_params` (zone scopes respected via `spans`).
#[allow(clippy::too_many_arguments)]
fn metrics_for_values(
    plans: &[StepPlan],
    raw_pool: &HashMap<String, Vec<f32>>,
    depth: &[f32],
    step_thick: &[f32],
    zones: &[ZoneEntry],
    cut: &Cutoffs,
    has_perm_cut: bool,
    mc_params: &[McParam],
    spans: &[ParamSpan],
    values: &[f64],
    n: usize,
    step_err: &std::sync::OnceLock<String>,
) -> Vec<MetricSet> {
    let pool = run_realization(plans, raw_pool, depth, mc_params, spans, values, n, step_err);
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

/// One well's precomputed execution state: step plans, the raw input-curve pool, the depth /
/// sample-thickness grids, zonation, and the UPPERCASE set of LogOut mnemonics some step
/// PRODUCES (the persist gate — inputs the chain merely consumes must never be written back
/// as MC uncertainty products).
struct WellPlan {
    plans: Vec<StepPlan>,
    raw_pool: HashMap<String, Vec<f32>>,
    depth: Vec<f32>,
    step_thick: Vec<f32>,
    zones: Vec<ZoneEntry>,
    produced: HashSet<String>,
}

/// Builds the per-step plan for one well: resolves inputs, options, and zone-resolved base
/// parameter arrays. Returns the plans plus the raw input-curve pool and depth/step arrays.
fn build_plans(
    conn: &Connection,
    well_id: &str,
    steps: &[ChainStep],
    specs: &HashMap<String, modules::ModuleSpec>,
) -> Result<WellPlan, String> {
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

        // A step that renames its outputs writes curves the plan builder cannot see: cutoffs and
        // the fraction-curve lists below are resolved from the manifest's declared LogOut names,
        // so the study would be planned against names the run never produces and would return
        // plausible percentiles computed from nothing.
        //
        // Both forms of the rename are refused, and BOTH have to be: a prefix and a per-curve name
        // are one freedom offered two ways (`OUT_PREFIX_OPT`, `OUT_NAME_PREFIX`), so catching only
        // the one that happened to ship first would leave the same silent failure reachable
        // through the grid. Refused by name rather than ignored — a study is not the place to
        // discover that a setting was quietly dropped.
        let renamed = step
            .opts
            .iter()
            .find(|(k, v)| k.starts_with(crate::workflow::OUT_NAME_PREFIX) && !v.trim().is_empty())
            .map(|(k, _)| k.trim_start_matches(crate::workflow::OUT_NAME_PREFIX).to_string());
        if step.opts.get(crate::workflow::OUT_PREFIX_OPT).is_some_and(|p| !p.trim().is_empty()) || renamed.is_some() {
            let what = match &renamed {
                Some(out) => format!("renames its {out} output"),
                None => "sets an output prefix".to_string(),
            };
            return Err(format!(
                "Step \"{}\" {what}, and a Monte Carlo study cannot follow that: its cutoffs and \
                 volume curves are resolved from the module's declared output names, so it would \
                 be planning against curves this run never writes. Clear the renaming on that \
                 step, or run it outside the study.",
                step.module
            ));
        }

        // Options: manifest defaults, then step overrides, plus __IN_<arg> mnemonics. Text args
        // travel here too — same channel as an Option, per `ArgKind::Text`.
        let mut opts: HashMap<String, String> = spec
            .args
            .iter()
            .filter(|a| a.kind == ArgKind::Option || a.kind == ArgKind::Text)
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

        plans.push(StepPlan {
            module: step.module.clone(),
            opts,
            depth_unit: crate::workflow::resolve_module_depth_unit(conn, &step.module)?,
            log_args,
            param_args,
            base_params,
        });
    }

    Ok(WellPlan { plans, raw_pool, depth, step_thick, zones: zones_raw, produced })
}

/// Contiguous index span of a zone-scoped MC parameter on the well's depth grid; None = the
/// whole well.
type ParamSpan = Option<(usize, usize)>;

/// One realization's output: (draws, per-zone metrics, optional tracked-curve snapshot for the
/// persist path in [`TRACKED`] order, physical-plausibility tally `(poro_violations,
/// sat_violations)` = distinct in-zone samples with PHIE / Sw outside [0,1] on the chain's
/// unlimited curves, plus the count of in-zone samples that had any finite porosity/saturation
/// value to judge).
type RealOut = (Vec<f64>, Vec<ZoneMetrics>, Option<Vec<Vec<f32>>>, (u32, u32, u32));

/// Output curves eligible for MC_*_LOW/P50/HIGH/BASE persistence, in snapshot order.
const TRACKED: [&str; 4] = ["VSH", "PHIE", "SWE", "PERM"];

/// Runs the chain in memory for one realization and returns the resulting curve pool.
/// `values[j]` is the draw for `mc_params[j]`, applied over `spans[j]` on top of that
/// parameter's zone-resolved base array (list order — a later entry wins where spans overlap).
fn run_realization(
    plans: &[StepPlan],
    raw_pool: &HashMap<String, Vec<f32>>,
    depth: &[f32],
    mc_params: &[McParam],
    spans: &[ParamSpan],
    values: &[f64],
    n: usize,
    // First module error seen anywhere in the sweep. `OnceLock` because the failure is identical
    // on every realization and this is written from rayon threads — first writer wins.
    step_err: &std::sync::OnceLock<String>,
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
            let mut arr = match plan.base_params.get(pname) {
                Some(base) => base.clone(),
                None => vec![f64::NAN; n],
            };
            for (j, mp) in mc_params.iter().enumerate() {
                if &mp.param == pname {
                    match spans[j] {
                        None => arr.fill(values[j]),
                        Some((s, e)) => arr[s..e].fill(values[j]),
                    }
                }
            }
            params.insert(pname.clone(), arr);
        }
        let ctx = ModuleContext { n, logs, params, opts: plan.opts.clone(), depth_unit: plan.depth_unit };
        match modules::run_module(&plan.module, &ctx) {
            Ok(outputs) => {
                for (k, v) in outputs {
                    pool.insert(k.to_uppercase(), v);
                }
            }
            Err(e) => {
                // Record the first failure instead of dropping it. Swallowing it left the pool
                // unchanged, so every downstream step read NaN and the study came back as a
                // P10=P50=P90 table of zeros with nothing to explain it — the same
                // silent-success failure that gascorr's own guard was written to prevent,
                // reintroduced one call site away. Identical on every realization, so first
                // writer wins and the rest are no-ops.
                let _ = step_err.set(format!("{}: {e}", plan.module));
            }
        }
    }
    pool
}

/// The chain's produced porosity / saturation output curves (unit `v/v`, UPPERCASE name starting
/// `PHI` / `SW`) — the physical-plausibility check targets, returned as (porosity, saturation).
/// Spec-driven so any conforming module is covered without hardcoded names; includes both the
/// limited and unlimited companions (the limited ones are clamped into range and never trip, so
/// scanning them is harmless).
fn fraction_output_curves(
    steps: &[ChainStep],
    specs: &HashMap<String, modules::ModuleSpec>,
) -> (Vec<String>, Vec<String>) {
    let (mut poro, mut sat) = (Vec::new(), Vec::new());
    for step in steps {
        let Some(spec) = specs.get(&step.module) else { continue };
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogOut) {
            if !a.unit.eq_ignore_ascii_case("v/v") {
                continue;
            }
            let up = a.name.to_uppercase();
            if up.starts_with("PHI") {
                if !poro.contains(&up) {
                    poro.push(up);
                }
            } else if up.starts_with("SW") && !sat.contains(&up) {
                sat.push(up);
            }
        }
    }
    (poro, sat)
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
    let mut plaus_out: Vec<McPlausibility> = Vec::new();
    let mut persisted: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    // Persist path: keep the FIRST `keep_cap` realizations' tracked curves. Draws are iid
    // (random) or a random permutation of strata (LHS), so a prefix is an unbiased subsample —
    // and unlike a stride precomputed from the REQUESTED count, it cannot collapse to a few
    // dozen snapshots when a convergence early stop truncates the run.
    let keep_cap = if req.persist { req.iterations.clamp(1, 100_000).min(1024) } else { 0 };
    let want_sens = req.sensitivity || req.tornado;
    let conv_tol = req.converge_tol.clamp(1e-5, 0.2);

    // The draw matrix depends only on (params, iterations, seed, sampling, correlations) — build
    // it once and share across wells. LHS stratification and Iman–Conover both need every
    // realization's draw jointly, which is why sampling is hoisted out of the rayon loop.
    let draws_all = build_draws(&req.mc_params, iterations, req.seed, req.sampling, &req.correlations, &mut notes);

    // Physical-plausibility candidates: the chain's produced porosity/saturation outputs. The
    // chain is identical across wells, so resolve them once. The UNLIMITED companions (PHIE_DN,
    // SWT_ARCH, …) carry the raw Sw>1 / PHIE<0 the module limits clamp away.
    let (poro_curves, sat_curves) = fraction_output_curves(&req.steps, &specs);

    for (wi, well_id) in req.well_ids.iter().enumerate() {
        if let Some(p) = progress {
            if p.is_cancelled() {
                break;
            }
            p.set_current(Some(format!("Monte Carlo: well {}/{}", wi + 1, req.well_ids.len())));
            p.start_item(well_id);
        }
        // One-time DB phase: build plans + read inputs + zonation.
        let (wp, well_name) = {
            let conn = db.lock().unwrap();
            let well_name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", duckdb::params![well_id], |r| r.get(0))
                .unwrap_or_else(|_| well_id.clone());
            match build_plans(&conn, well_id, &req.steps, &specs) {
                Ok(wp) => (wp, well_name),
                Err(e) => {
                    if let Some(p) = progress {
                        p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                    }
                    errors.push(format!("{well_id}: {e}"));
                    continue;
                }
            }
        };
        let WellPlan { plans, raw_pool, depth, step_thick, mut zones, produced } = wp;
        let n = depth.len();
        if zones.is_empty() {
            zones.push(ZoneEntry { zone_name: "ALL".into(), top_depth: depth[0], bottom_depth: *depth.last().unwrap() });
        }
        // First module failure anywhere in this well's sweep. A failed step used to be dropped,
        // leaving the pool unchanged so every downstream step read NaN and the well came back as
        // a P10=P50=P90 table of zeros — a confident-looking uncertainty study of a chain that
        // never ran. Reported per well after the sweep.
        let step_err: std::sync::OnceLock<String> = std::sync::OnceLock::new();

        // A permeability cutoff has to survive a chain that MODELS permeability.
        //
        // `raw_pool` holds only EXTERNAL inputs — mnemonics no step produces — so the moment a
        // `perm_coates` (or any other permeability model) is inserted into the chain, PERM leaves
        // that pool and the cutoff switched itself off silently. Exactly backwards: a study that
        // models permeability is the study whose permeability cutoff matters
        // (`docs/review_triage.md` finding 8). The realization pool carries produced curves, so
        // PERM really is there when `zone_metrics` reads it.
        let has_perm_cut = req.perm_min.is_some()
            && (produced.contains("PERM")
                || raw_pool.get("PERM").map(|c| c.iter().any(|v| !v.is_nan())).unwrap_or(false));

        // In-zone mask (union of the reported zone windows) for the physical-plausibility scan —
        // out-of-zone samples never enter the volumetrics, so they should not count either.
        let in_zone: Vec<bool> = depth
            .iter()
            .map(|d| zones.iter().any(|z| z.top_depth < z.bottom_depth && *d >= z.top_depth && *d < z.bottom_depth))
            .collect();

        // Zone-scoped MC parameters: resolve each entry to a contiguous index span on this
        // well's (sorted) depth grid, matching the `d >= top && d < bottom` zone convention.
        // An unknown zone name leaves the parameter at base values (empty span) with a note.
        let spans: Vec<ParamSpan> = req
            .mc_params
            .iter()
            .map(|p| match &p.zone {
                None => None,
                Some(zname) => match zones.iter().find(|z| &z.zone_name == zname) {
                    // Inverted zones (top ≥ bottom) must yield an EMPTY span, matching how every
                    // deterministic consumer treats them (resolve_zone_param matches no samples,
                    // zone_metrics skips h ≤ 0) — an unguarded s > e would panic the slice fill.
                    Some(z) if z.top_depth < z.bottom_depth => Some((
                        depth.partition_point(|d| *d < z.top_depth),
                        depth.partition_point(|d| *d < z.bottom_depth),
                    )),
                    Some(_) => {
                        notes.push(format!(
                            "{well_name}: zone '{zname}' has top ≥ bottom — parameter '{}' left at base values",
                            p.param
                        ));
                        Some((0, 0))
                    }
                    None => {
                        notes.push(format!(
                            "{well_name}: zone '{zname}' not found for parameter '{}' — left at base values",
                            p.param
                        ));
                        Some((0, 0))
                    }
                },
            })
            .collect();

        // Parallel realizations over the precomputed draw matrix (ordered like `req.mc_params`,
        // so sensitivity can correlate draws against outputs; rows are empty when no parameters
        // vary). Random mode replays the legacy per-index sequence, so results are unchanged.
        let eval = |r: usize| -> RealOut {
            let draws = &draws_all[r];
            let pool = run_realization(&plans, &raw_pool, &depth, &req.mc_params, &spans, draws, n, &step_err);
            let nanv = vec![f32::NAN; n];
            let vsh = pool.get("VSH").unwrap_or(&nanv);
            let phie = pool.get("PHIE").unwrap_or(&nanv);
            let swe = pool.get("SWE").unwrap_or(&nanv);
            let perm = pool.get("PERM").unwrap_or(&nanv);
            let zm = zones
                .iter()
                .map(|z| zone_metrics(vsh, phie, swe, perm, &depth, &step_thick, z, &cut, has_perm_cut))
                .collect();
            // Physical-plausibility tally: distinct in-zone samples whose porosity or saturation
            // leaves [0,1] on the chain's curves. The unlimited companions (PHIE_DN, SWT_ARCH, …)
            // carry the raw Sw>1 / PHIE<0 the module limits clamp away; the limited PHIE/SWE stay
            // in range and never trip. Counted, never excluded (see McPlausibility).
            let oob = |v: f32| {
                let v = v as f64;
                v.is_finite() && (v < -PHYS_TOL || v > 1.0 + PHYS_TOL)
            };
            let poro_slices: Vec<&Vec<f32>> = poro_curves.iter().filter_map(|nm| pool.get(nm)).collect();
            let sat_slices: Vec<&Vec<f32>> = sat_curves.iter().filter_map(|nm| pool.get(nm)).collect();
            let (mut poro_bad, mut sat_bad, mut checked) = (0u32, 0u32, 0u32);
            for i in 0..n {
                if !in_zone[i] {
                    continue;
                }
                // "checked" = an in-zone sample that actually had a finite porosity/saturation
                // value to judge, so a well that produced none (e.g. missing RHOB) reports "not
                // checked" rather than a fabricated clean pass.
                if poro_slices.iter().chain(sat_slices.iter()).any(|c| (c[i] as f64).is_finite()) {
                    checked += 1;
                }
                if poro_slices.iter().any(|c| oob(c[i])) {
                    poro_bad += 1;
                }
                if sat_slices.iter().any(|c| oob(c[i])) {
                    sat_bad += 1;
                }
            }
            let snap = if r < keep_cap {
                // Snapshot only curves the chain PRODUCES — inputs it merely consumes are not
                // Monte Carlo products (empty slots keep TRACKED indexing intact).
                Some(
                    TRACKED
                        .iter()
                        .map(|k| {
                            if produced.contains(*k) {
                                pool.get(*k).cloned().unwrap_or_else(|| vec![f32::NAN; n])
                            } else {
                                Vec::new()
                            }
                        })
                        .collect(),
                )
            } else {
                None
            };
            (draws.clone(), zm, snap, (poro_bad, sat_bad, checked))
        };
        let per_real: Vec<RealOut> = if req.converge {
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
            let mut acc: Vec<RealOut> = Vec::with_capacity(iterations);
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
        // then each ENTRY swept to its P10/P90 with the rest held at base (values indexed like
        // `mc_params`, so two zone-scoped entries of the same parameter sweep independently).
        // Cheap — one chain run per endpoint. `oat[param][zone]` for low/base/high.
        let (oat_base, oat_low, oat_high): (Vec<MetricSet>, Vec<Vec<MetricSet>>, Vec<Vec<MetricSet>>) =
            if req.tornado && !req.mc_params.is_empty() {
                let base_vals: Vec<f64> = req.mc_params.iter().map(|p| p.dist.central()).collect();
                let base = metrics_for_values(
                    &plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut,
                    &req.mc_params, &spans, &base_vals, n, &step_err,
                );
                let mut low = Vec::with_capacity(req.mc_params.len());
                let mut high = Vec::with_capacity(req.mc_params.len());
                for (pj, p) in req.mc_params.iter().enumerate() {
                    let mut lv = base_vals.clone();
                    lv[pj] = p.dist.quantile(lo_p);
                    let mut hv = base_vals.clone();
                    hv[pj] = p.dist.quantile(hi_p);
                    low.push(metrics_for_values(
                        &plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut,
                        &req.mc_params, &spans, &lv, n, &step_err,
                    ));
                    high.push(metrics_for_values(
                        &plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut,
                        &req.mc_params, &spans, &hv, n, &step_err,
                    ));
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
                            // Zone-scoped entries are labeled "PARAM @ ZONE" so two rows for
                            // the same parameter stay distinguishable in the tornado.
                            param: match &p.zone {
                                Some(z) => format!("{} @ {}", p.param, z),
                                None => p.param.clone(),
                            },
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
        // Physical-plausibility rollup (playbook #1 residual): fraction of realizations whose
        // sampled combo produced an impossible Sw>1 / PHIE<0 on the unlimited curves. Reported,
        // NOT excluded — the module limits already clamp these to the correct volumetrics, so they
        // stay valid low/high tails (see McPlausibility). A large fraction warns that the input
        // distributions are straining physics.
        if !poro_curves.is_empty() || !sat_curves.is_empty() {
            let realz = per_real.len();
            let mut impossible = 0u32;
            let (mut poro_realz, mut sat_realz) = (0u32, 0u32);
            let (mut poro_tot, mut sat_tot) = (0u64, 0u64);
            let mut checked_total = 0u64;
            for m in &per_real {
                let (pb, sb, ck) = m.3;
                checked_total += ck as u64;
                if pb + sb > 0 {
                    impossible += 1;
                }
                if pb > 0 {
                    poro_realz += 1;
                    poro_tot += pb as u64;
                }
                if sb > 0 {
                    sat_realz += 1;
                    sat_tot += sb as u64;
                }
            }
            let fraction = if realz > 0 { impossible as f32 / realz as f32 } else { 0.0 };
            let checked = checked_total > 0;
            let mut parts: Vec<String> = Vec::new();
            if sat_realz > 0 {
                parts.push(format!("Sw>1 in {sat_realz}/{realz} ({sat_tot} samples)"));
            }
            if poro_realz > 0 {
                parts.push(format!("PHIE<0 or >1 in {poro_realz}/{realz} ({poro_tot} samples)"));
            }
            let detail = if !checked {
                "no porosity/saturation samples to check".to_string()
            } else if parts.is_empty() {
                "all realizations physically in bounds".to_string()
            } else {
                parts.join("; ")
            };
            plaus_out.push(McPlausibility {
                well_id: well_id.clone(),
                well_name: well_name.clone(),
                impossible_realizations: impossible,
                realizations: realz as u32,
                fraction,
                checked,
                detail,
            });
        }

        // Persist per-sample uncertainty curves to a fresh version of the MONTECARLO log set:
        // MC_<KEY>_LOW/_P50/_HIGH from the kept realizations' per-sample spread, MC_<KEY>_BASE
        // from one deterministic run at every entry's median. Only curves the chain PRODUCES
        // (LogOut of some step) are eligible — inputs it merely consumes would come back as
        // zero-width fake uncertainty bands. Samples with < 8 finite realizations stay NaN. A
        // degenerate base run skips only BASE (with a note), never the percentile curves.
        let mut persist_warn: Option<String> = None;
        if req.persist {
            let centrals: Vec<f64> = req.mc_params.iter().map(|p| p.dist.central()).collect();
            let base_pool = run_realization(&plans, &raw_pool, &depth, &req.mc_params, &spans, &centrals, n, &step_err);
            let kept = per_real.iter().filter(|m| m.2.is_some()).count();
            let real_cap = req.realization_cap.unwrap_or(256).clamp(8, 1024) as usize;
            let mut out: Vec<(String, Vec<f32>)> = Vec::new();
            // (curve name, depths, per-depth realization vectors) for the array store.
            let mut arrays: Vec<(String, Vec<f32>, Vec<Vec<f32>>)> = Vec::new();
            for (t, key) in TRACKED.iter().enumerate() {
                if !produced.contains(*key) {
                    continue;
                }
                let snaps: Vec<&Vec<f32>> = per_real.iter().filter_map(|m| m.2.as_ref().map(|s| &s[t])).collect();
                let mut lo_c = vec![f32::NAN; n];
                let mut mid_c = vec![f32::NAN; n];
                let mut hi_c = vec![f32::NAN; n];
                let mut buf: Vec<f32> = Vec::with_capacity(snaps.len());
                let cap = real_cap.min(snaps.len());
                let mut arr_depths: Vec<f32> = Vec::new();
                let mut arr_vals: Vec<Vec<f32>> = Vec::new();
                for i in 0..n {
                    buf.clear();
                    buf.extend(snaps.iter().map(|s| s[i]).filter(|v| v.is_finite()));
                    if req.persist_realizations {
                        // REALIZATION ORDER, NaNs included: index r must mean the same
                        // realization at every depth or a spaghetti trace is not a trace. The
                        // sorted `buf` above is for percentiles only and must not be stored.
                        // The >= 8 floor matches the percentile curves', so a stored depth and
                        // the MC_*_LOW/_HIGH curves are never present at different depths.
                        let col: Vec<f32> = snaps.iter().take(cap).map(|s| s[i]).collect();
                        if col.iter().filter(|v| v.is_finite()).count() >= 8 {
                            arr_depths.push(depth[i]);
                            arr_vals.push(col);
                        }
                    }
                    if buf.len() < 8 {
                        continue;
                    }
                    buf.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    lo_c[i] = percentile(&buf, lo_p);
                    mid_c[i] = percentile(&buf, 0.50);
                    hi_c[i] = percentile(&buf, hi_p);
                }
                let have_pct = mid_c.iter().any(|v| v.is_finite());
                let base = base_pool.get(*key).filter(|c| c.iter().any(|v| v.is_finite())).cloned();
                if !have_pct && base.is_none() {
                    continue;
                }
                if have_pct {
                    out.push((format!("MC_{key}_LOW"), lo_c));
                    out.push((format!("MC_{key}_P50"), mid_c));
                    out.push((format!("MC_{key}_HIGH"), hi_c));
                }
                match base {
                    Some(b) => out.push((format!("MC_{key}_BASE"), b)),
                    None => notes.push(format!(
                        "{well_name}: MC_{key}_BASE skipped — the all-median base run produced no finite {key}"
                    )),
                }
                if !arr_depths.is_empty() {
                    arrays.push((format!("MC_{key}_REAL"), arr_depths, arr_vals));
                }
            }
            if out.is_empty() {
                notes.push(format!("{well_name}: nothing to persist — no tracked output curve had finite values"));
            } else {
                let spec = equations::LogSetSpec {
                    set_name: "MONTECARLO".into(),
                    module: "montecarlo".into(),
                    params_json: serde_json::json!({
                        "iterations": used_iterations,
                        "seed": req.seed,
                        "sampling": match req.sampling { Sampling::Lhs => "lhs", Sampling::Random => "random" },
                        "low_pctl": lo_p,
                        "high_pctl": hi_p,
                        "kept_realizations": kept,
                        "params": req.mc_params.iter().map(|p| match &p.zone {
                            Some(z) => format!("{} @ {}", p.param, z),
                            None => p.param.clone(),
                        }).collect::<Vec<_>>(),
                    })
                    .to_string(),
                    inputs_json: serde_json::json!(req.steps.iter().map(|s| s.module.clone()).collect::<Vec<_>>())
                        .to_string(),
                };
                let conn = db.lock().unwrap();
                // Reclaim the ENTIRE MC_* family in the current store first: a previous
                // MONTECARLO version may have written keys this run doesn't (e.g. PERM dropped
                // from the chain), and the versioned writer deletes only the names it writes —
                // stale cross-version rows would otherwise keep serving next to this run's
                // curves and keep the old version flagged current. The archive keeps every
                // version restorable.
                let family: Vec<String> = TRACKED
                    .iter()
                    .flat_map(|k| ["LOW", "P50", "HIGH", "BASE"].into_iter().map(move |s| format!("MC_{k}_{s}")))
                    .collect();
                let ph = std::iter::repeat("?").take(family.len()).collect::<Vec<_>>().join(", ");
                let mut del_params: Vec<String> = Vec::with_capacity(family.len() + 1);
                del_params.push(well_id.clone());
                del_params.extend(family);
                let write = conn
                    .execute(
                        &format!("DELETE FROM computed_curves WHERE well_id = ? AND upper(curve_name) IN ({ph})"),
                        duckdb::params_from_iter(del_params),
                    )
                    .and_then(|_| equations::create_log_set(&conn, well_id, &spec))
                    .and_then(|(set_id, version)| {
                        let refs: Vec<(&str, &[f32])> = out.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
                        equations::write_computed_curves_versioned(&conn, well_id, &depth, &refs, &set_id)
                            .map(|()| version)
                    });
                match write {
                    Ok(version) => {
                        if used_iterations > kept {
                            notes.push(format!(
                                "{well_name}: percentile curves estimated from the first {kept} realizations"
                            ));
                        }
                        notes.push(format!("{well_name}: persisted {} curves to log set MONTECARLO v{version}", out.len()));
                        for (k, _) in &out {
                            if !persisted.contains(k) {
                                persisted.push(k.clone());
                            }
                        }
                        // Realization matrices go to `array_logs`, NOT to the versioned archive
                        // that holds the curves. That is deliberate: the archive exists so a
                        // re-run is non-destructive, and keeping every version of a matrix this
                        // size would balloon the project file. The matrix is the WORKING data a
                        // display reads and is replaced by its own re-run; the percentile CURVES
                        // it produced are what stay versioned and restorable.
                        for (name, ds, vals) in &arrays {
                            // No axis: realization 7 is not a measurement at 7 of anything, and writing an
                            // index there would invite a reader to plot it against one.
                            match db::write_array_log(&conn, well_id, "MONTECARLO", name, ds, vals, None) {
                                Ok(rows) => {
                                    notes.push(format!(
                                        "{well_name}: stored {name} — {rows} depths x {} realizations",
                                        vals.first().map_or(0, Vec::len)
                                    ));
                                    if !persisted.contains(name) {
                                        persisted.push(name.clone());
                                    }
                                }
                                Err(e) => notes.push(format!("{well_name}: {name} not stored — {e}")),
                            }
                        }
                        if !arrays.is_empty() && kept > real_cap {
                            notes.push(format!(
                                "{well_name}: realizations stored are the first {real_cap} of {kept}, so a band drawn from them can differ slightly from MC_*_LOW/_HIGH"
                            ));
                        }
                    }
                    Err(e) => {
                        let msg = format!("persist failed: {e}");
                        errors.push(format!("{well_id}: {msg}"));
                        persist_warn = Some(msg);
                    }
                }
            }
        }

        // A chain step that failed on every realization means this well's whole study is built on
        // a chain that never ran: the pool kept its NaNs and the volumetrics came out as zeros.
        // Surface the module's own message — it is the actionable one (e.g. gascorr telling you
        // OPT_GATE is FLAGGED but condflag was never run) — instead of letting the study read as
        // a confident P10=P50=P90 table of zeros.
        let chain_err = step_err.get().cloned();
        if let Some(m) = &chain_err {
            errors.push(format!("{well_id}: chain step failed on every realization — {m}"));
        }

        if let Some(p) = progress {
            // A well whose study succeeded but whose persist write failed finishes WARNED, not
            // Ok — the same convention every other curve writer follows.
            match (&chain_err, &persist_warn) {
                (Some(m), _) => p.finish_item(
                    well_id,
                    crate::jobs::ItemState::Failed,
                    Some(format!("chain step failed on every realization — {m}")),
                ),
                (None, Some(m)) => p.finish_item(well_id, crate::jobs::ItemState::Warned, Some(m.clone())),
                (None, None) => p.finish_item(well_id, crate::jobs::ItemState::Ok, None),
            }
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
        persisted,
        plausibility: plaus_out,
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
            persist: false,
            persist_realizations: false,
            realization_cap: None,
        }
    }

    /// T-BATCH-16 — adding a permeability MODEL to a Monte Carlo chain silently switches the
    /// permeability CUTOFF off.
    ///
    /// `has_perm_cut` asks whether PERM is in `raw_pool`, and `build_plans` fills `raw_pool`
    /// only from LogIn mnemonics that **no step produces**. So a chain that reads PERM from the
    /// project (rocktyping takes it as an input) gets a working cutoff — but the moment a
    /// permeability model is added ahead of it, PERM becomes a produced curve, drops out of the
    /// external set, and the cutoff goes quiet. Both chains are shown here, one after the other,
    /// on the same well with the same PERM curve in the project and the same cutoff.
    ///
    /// That was the wrong way round — a study that models permeability is exactly the study whose
    /// permeability cutoff matters, and there was nothing in the result to say it had been
    /// skipped, so the numbers looked like a cutoff that was applied and happened not to bite.
    ///
    /// Fixed 2026-08-01 (`docs/review_triage.md` finding 8) by asking `produced` as well as
    /// `raw_pool`. The realization pool carries produced curves, so PERM really is there when
    /// `zone_metrics` reads it — this is not a case of turning on a cutoff with no data behind it.
    ///
    /// Both chains stay in the test. Chain A is the control: without it, the assertions on B
    /// would pass just as well against a cutoff that was broken everywhere.
    #[test]
    fn a_permeability_cutoff_survives_a_chain_that_models_permeability() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // A measured permeability in the project, far below any cutoff used here.
        let n = 300usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        crate::equations::write_computed_curve(&conn, &well, &depth, "PERM", &vec![1.0f32; n]).unwrap();
        let dbm = Mutex::new(conn);

        let run = |steps: Vec<ChainStep>, perm_min: Option<f64>| -> McResult {
            let mc = vec![McParam {
                param: "GR_MA".into(),
                dist: Distribution::Normal { mean: 25.0, sd: 5.0 },
                zone: None,
            }];
            let mut req = base_request(&well, mc, 24, 42);
            req.steps = steps;
            req.perm_min = perm_min;
            let res = run_monte_carlo(&dbm, &req, None);
            assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
            res
        };

        // Chain A — reads permeability from the project (rocktyping consumes PERM, nothing
        // produces it). This is the case where the cutoff works, and it is the control: without
        // it, the equality below would prove only that the cutoff is broken everywhere.
        let reads_perm = || vec![step("vsh_gr"), step("phi_dn"), step("sw_indo"), step("rocktyping")];
        let a_open = run(reads_perm(), None);
        let a_cut = run(reads_perm(), Some(1.0e9));
        assert!(a_open.zones[0].net.mid > 0.0, "the well must have pay before any cutoff");
        assert_eq!(a_cut.zones[0].net.mid, 0.0, "1 mD cannot pass a 1e9 mD cutoff — the cutoff works here");

        // Chain B — the SAME chain with a permeability model inserted ahead of it. PERM is now a
        // PRODUCED curve, so it never enters the external pool at all; the cutoff has to find it
        // in `produced` instead.
        let makes_perm =
            || vec![step("vsh_gr"), step("phi_dn"), step("sw_indo"), step("perm_coates"), step("rocktyping")];
        let b_open = run(makes_perm(), None);
        let b_cut = run(makes_perm(), Some(1.0e9));
        let (bo, bc) = (&b_open.zones[0], &b_cut.zones[0]);
        assert!(bo.net.mid > 0.0, "chain B must have pay to lose");
        assert_eq!(bc.net.mid, 0.0, "a 1e9 mD cutoff must bite in chain B exactly as in chain A");
        assert_eq!(bc.hpv.mid, 0.0, "and take HPV with it");
        assert_eq!(bc.ntg.mid, 0.0, "and N:G");

        // Stated as the comparison a user would actually make: same well, same cutoff, same
        // project — the two chains must now AGREE, where before one reported no pay and the other
        // reported all of it.
        assert_eq!(
            bc.net.mid, a_cut.zones[0].net.mid,
            "modelling permeability must not change whether the permeability cutoff applies"
        );

        // And the cutoff is still a cutoff rather than a switch that now deletes everything: a
        // threshold the modelled rock CLEARS must leave the pay alone. Without this, setting
        // `has_perm_cut` unconditionally would pass every assertion above.
        let b_loose = run(makes_perm(), Some(1.0e-9));
        assert_eq!(
            b_loose.zones[0].net.mid, bo.net.mid,
            "a cutoff the modelled permeability passes must not remove pay"
        );
    }

    /// T-BATCH-17 — a step's MASK is carried into the Monte Carlo plan and then never read.
    ///
    /// The real chain runner blanks every masked input before a module runs and blanks the
    /// outputs after (`workflow.rs`). `run_realization` does neither, so the Monte Carlo engine
    /// interprets washout as rock and reports MORE pay than the very same chain does — the
    /// dangerous direction, since a batch study is what gets quoted.
    ///
    /// There are TWO causes, not the one the audit names, and this test pins both: even if
    /// `run_realization` learned to blank, `build_plans` never fetches the flag curve, because
    /// `external` is assembled from LogIn mnemonics and MASK is an Option. Whoever fixes this
    /// must extend both or the mask will silently blank nothing.
    #[test]
    fn the_monte_carlo_chain_ignores_a_step_mask_the_real_chain_honours() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);

        // Bad hole over the shallow third — the same grid `seed_well` laid down.
        let n = 300usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let badhole: Vec<f32> = (0..n).map(|i| if i < 100 { 1.0 } else { 0.0 }).collect();
        crate::equations::write_computed_curve(&conn, &well, &depth, "BADHOLE", &badhole).unwrap();
        let dbm = Mutex::new(conn);

        let masked = || HashMap::from([("MASK".to_string(), "BADHOLE".to_string())]);
        let modules_in_chain = ["vsh_gr", "phi_dn", "sw_indo"];

        // The real chain, one masked step at a time, then the pay summary over what it wrote.
        for m in modules_in_chain {
            let res = crate::workflow::run_workflow_module(
                &dbm,
                &crate::workflow::RunModuleRequest {
                    module: m.into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params: HashMap::new(),
                    opts: masked(),
                    output_set: None,
                    input_set: None,
                },
            );
            assert!(res[0].error.is_none(), "{m} failed: {:?}", res[0].error);
        }
        let chain_rows = crate::workflow::run_pay_summary(
            &dbm,
            &crate::workflow::PaySummaryRequest {
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: 0.5,
                phie_min: 0.08,
                swe_max: 0.6,
                perm_min: None,
                skip_version: false,
                stats_only: true,
            },
        )
        .expect("pay summary runs");
        let chain_net = chain_rows.iter().find(|r| r.flag == "PAY").expect("a PAY row").net;
        assert!(chain_net > 0.0, "the masked chain must still leave pay, or the comparison is empty");

        // Monte Carlo over the SAME chain with the SAME mask and no uncertainty at all. It claims
        // to run the same chain, so it should land on the same number.
        let mut req = base_request(&well, Vec::new(), 4, 42);
        req.steps = modules_in_chain
            .iter()
            .map(|m| {
                let mut s = step(m);
                s.opts = masked();
                s
            })
            .collect();
        let mc = run_monte_carlo(&dbm, &req, None);
        assert!(mc.errors.is_empty(), "unexpected errors: {:?}", mc.errors);
        let mc_net = mc.zones[0].net.mid;

        assert!(
            mc_net > chain_net,
            "MC counted the washout as rock: MC net {mc_net} vs the same chain's {chain_net}"
        );

        // The mask is inert, not partially applied: dropping it changes nothing.
        let mut unmasked = req.clone();
        unmasked.steps = modules_in_chain.iter().map(|m| step(m)).collect();
        let mc_unmasked = run_monte_carlo(&dbm, &unmasked, None);
        assert_eq!(
            mc_net, mc_unmasked.zones[0].net.mid,
            "setting a MASK on the step made no difference to the Monte Carlo answer"
        );

        // Cause two: the flag curve never even enters the pool the realizations run on, though
        // the option itself is carried all the way into the plan.
        {
            let conn = dbm.lock().unwrap();
            let specs: HashMap<String, modules::ModuleSpec> =
                modules::list_modules().into_iter().map(|s| (s.name.clone(), s)).collect();
            let wp = build_plans(&conn, &well, &req.steps, &specs).expect("plans build");
            assert!(
                !wp.raw_pool.contains_key("BADHOLE"),
                "the flag curve is never fetched: {:?}",
                wp.raw_pool.keys().collect::<Vec<_>>()
            );
            assert_eq!(
                wp.plans[0].opts.get("MASK").map(String::as_str),
                Some("BADHOLE"),
                "the MASK option IS carried into the plan — it is simply never read"
            );
        }
    }

    #[test]
    fn hpv_distribution_is_ordered_and_reproducible() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 }, zone: None }];
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
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 }, zone: None }];
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
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 }, zone: None }];
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
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 6.0 }, zone: None }],
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
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 }, zone: None }],
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
            vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 6.0 }, zone: None }],
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
        let params = vec![McParam { param: "X".into(), dist: Distribution::Uniform { lo: 2.0, hi: 5.0 }, zone: None }];
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
            McParam { param: "A".into(), dist: Distribution::Normal { mean: 0.0, sd: 1.0 }, zone: None },
            McParam { param: "B".into(), dist: Distribution::Uniform { lo: 10.0, hi: 20.0 }, zone: None },
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
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 }, zone: None }];
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
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 0.0 }, zone: None }];
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
    fn zone_scoped_param_only_moves_its_zone() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // Two zones over the 1000–1150 m column; the MC entry is scoped to UPPER only.
        db::upsert_zone(&conn, &well, "UPPER", 1000.0, 1075.0).unwrap();
        db::upsert_zone(&conn, &well, "LOWER", 1075.0, 1150.0).unwrap();
        let dbm = Mutex::new(conn);

        let mc = vec![McParam {
            param: "GR_MA".into(),
            dist: Distribution::Normal { mean: 25.0, sd: 15.0 },
            zone: Some("UPPER".into()),
        }];
        let mut req = base_request(&well, mc, 400, 42);
        req.sensitivity = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        let upper = res.zones.iter().find(|z| z.zone == "UPPER").expect("UPPER zone");
        let lower = res.zones.iter().find(|z| z.zone == "LOWER").expect("LOWER zone");
        assert!(upper.hpv.hi > upper.hpv.lo, "scoped uncertainty must spread its own zone");
        assert_eq!(lower.hpv.lo, lower.hpv.hi, "the unscoped zone must stay deterministic");
        assert!(lower.hpv.sd.abs() < 1e-6, "no variance may leak outside the scoped zone");
        // The sensitivity row carries the zone-qualified label.
        assert_eq!(res.sensitivity[0].params[0].param, "GR_MA @ UPPER");

        // An unknown zone name leaves the parameter at base values, with a note.
        let mc2 = vec![McParam {
            param: "GR_MA".into(),
            dist: Distribution::Normal { mean: 25.0, sd: 15.0 },
            zone: Some("NOPE".into()),
        }];
        let res2 = run_monte_carlo(&dbm, &base_request(&well, mc2, 100, 42), None);
        assert!(res2.notes.iter().any(|n| n.contains("NOPE")), "unknown zone should note: {:?}", res2.notes);
        for z in &res2.zones {
            assert_eq!(z.hpv.lo, z.hpv.hi, "unknown zone scope must not vary anything");
        }
    }

    fn seed_computed(conn: &Connection, well: &str, name: &str, value: f32) {
        for i in 0..300 {
            let d = 1000.0 + i as f32 * 0.5;
            conn.execute(
                "INSERT INTO computed_curves (well_id, depth, curve_name, value) VALUES (?1, ?2, ?3, ?4)",
                duckdb::params![well, d, name, value],
            )
            .unwrap();
        }
    }

    #[test]
    fn inverted_zone_notes_instead_of_panicking() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // Storable through the DB inspector (no validation on that path): top ≥ bottom.
        db::upsert_zone(&conn, &well, "BAD", 1100.0, 1050.0).unwrap();
        let dbm = Mutex::new(conn);
        let mc = vec![McParam {
            param: "GR_MA".into(),
            dist: Distribution::Normal { mean: 25.0, sd: 15.0 },
            zone: Some("BAD".into()),
        }];
        // Must complete (pre-fix: arr[s..e] with s > e panicked the rayon loop) and behave like
        // every deterministic consumer: the inverted zone contributes nothing.
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 100, 42), None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert!(res.notes.iter().any(|n| n.contains("top ≥ bottom")), "expected inverted-zone note: {:?}", res.notes);
        for z in &res.zones {
            assert_eq!(z.hpv.lo, z.hpv.hi, "inverted-zone scope must not vary anything");
        }
    }

    #[test]
    fn persist_skips_chain_inputs() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // PHIE/VSH exist as previously computed curves — sw_indo CONSUMES them.
        seed_computed(&conn, &well, "PHIE", 0.2);
        seed_computed(&conn, &well, "VSH", 0.1);
        let dbm = Mutex::new(conn);
        let mc = vec![McParam { param: "A".into(), dist: Distribution::Normal { mean: 1.0, sd: 0.15 }, zone: None }];
        let mut req = base_request(&well, mc, 200, 42);
        req.steps = vec![step("sw_indo")];
        req.persist = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        // Only curves the chain PRODUCES may be persisted — inputs it merely reads must not
        // come back as zero-width fake uncertainty bands.
        assert!(res.persisted.iter().any(|c| c.starts_with("MC_SWE_")), "persisted: {:?}", res.persisted);
        assert!(
            !res.persisted.iter().any(|c| c.starts_with("MC_PHIE_") || c.starts_with("MC_VSH_")),
            "chain inputs must not be persisted: {:?}",
            res.persisted
        );
    }

    #[test]
    fn persist_realizations_stores_a_matrix_that_reproduces_the_percentile_curves() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        seed_computed(&conn, &well, "PHIE", 0.2);
        seed_computed(&conn, &well, "VSH", 0.1);
        let dbm = Mutex::new(conn);
        let mc = vec![McParam { param: "A".into(), dist: Distribution::Normal { mean: 1.0, sd: 0.15 }, zone: None }];
        let mut req = base_request(&well, mc, 64, 42);
        req.steps = vec![step("sw_indo")];
        req.persist = true;
        req.persist_realizations = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert!(res.persisted.iter().any(|c| c == "MC_SWE_REAL"), "persisted: {:?}", res.persisted);

        let conn = dbm.lock().unwrap();
        let rows = db::read_array_log(&conn, &well, Some("MONTECARLO"), "MC_SWE_REAL").unwrap();
        assert!(!rows.is_empty(), "the matrix must reach the store");
        assert!(rows.iter().all(|r| r.samples.len() == 64), "every depth keeps every realization slot");

        // The stored matrix must reproduce the curves written beside it: below the storage cap
        // they are the same realizations, so a band drawn from the matrix and the persisted
        // MC_SWE_LOW/_HIGH curves are the SAME numbers, not merely similar ones.
        let curves = equations::fetch_curve_frame(
            &conn,
            &well,
            &["MC_SWE_LOW".into(), "MC_SWE_P50".into(), "MC_SWE_HIGH".into()],
        )
        .unwrap();
        let (depth, cols) = (curves.0, curves.1);
        let idx = depth.iter().position(|d| (*d - rows[0].depth).abs() < 1e-4).expect("stored depth is a curve depth");
        let (lo, med, hi) = crate::distribution::band(&rows[0].samples, 10.0, 90.0).unwrap();
        for (name, want) in [("MC_SWE_LOW", lo), ("MC_SWE_P50", med), ("MC_SWE_HIGH", hi)] {
            let got = cols.get(name).and_then(|c| c.get(idx).copied()).unwrap();
            assert!((got - want).abs() < 1e-5, "{name}: curve {got} vs matrix {want}");
        }
    }

    #[test]
    fn persist_reclaims_stale_family_and_degenerate_base_drops_only_base() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        // Run 1: full chain → MC_VSH/PHIE/SWE families all written (v1).
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 }, zone: None }];
        let mut req1 = base_request(&well, mc, 200, 42);
        req1.persist = true;
        let res1 = run_monte_carlo(&dbm, &req1, None);
        assert!(res1.persisted.iter().any(|c| c.starts_with("MC_PHIE_")));

        // Run 2: vsh_gr only, with GR_SH pinned to the distribution's central value — the
        // all-median base run degenerates (GR_MA == GR_SH), but half the draws stay finite.
        let mut s = step("vsh_gr");
        s.params.insert("GR_SH".into(), 100.0);
        let mc2 = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 99.0, hi: 101.0 }, zone: None }];
        let mut req2 = base_request(&well, mc2, 300, 7);
        req2.steps = vec![s];
        req2.persist = true;
        let res2 = run_monte_carlo(&dbm, &req2, None);
        assert!(res2.errors.is_empty(), "unexpected errors: {:?}", res2.errors);
        // Percentile curves survive a degenerate base; only BASE is skipped, with a note.
        assert!(res2.persisted.contains(&"MC_VSH_LOW".to_string()), "persisted: {:?}", res2.persisted);
        assert!(
            !res2.persisted.contains(&"MC_VSH_BASE".to_string()),
            "degenerate base must skip only BASE: {:?}",
            res2.persisted
        );
        assert!(res2.notes.iter().any(|n| n.contains("MC_VSH_BASE skipped")), "notes: {:?}", res2.notes);

        let conn = dbm.lock().unwrap();
        // Stale family reclaim: run 2 wrote no PHIE curves, so v1's MC_PHIE_* rows must be gone
        // from the CURRENT store (the archive keeps v1 restorable).
        let current_phie: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name LIKE 'MC_PHIE%'",
                duckdb::params![well],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(current_phie, 0, "stale MC_PHIE_* rows must be reclaimed from the current store");
        let archive_phie: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves_archive WHERE well_id = ?1 AND curve_name LIKE 'MC_PHIE%'",
                duckdb::params![well],
                |r| r.get(0),
            )
            .unwrap();
        assert!(archive_phie > 0, "the archive must keep v1's curves restorable");
        let versions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'MONTECARLO'",
                duckdb::params![well],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(versions, 2);
    }

    #[test]
    fn persist_writes_versioned_low_base_high_curves() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 }, zone: None }];
        let mut req = base_request(&well, mc, 300, 42);
        req.persist = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        // The chain (vsh_gr → phi_dn → sw_indo) produces VSH/PHIE/SWE but no PERM.
        assert!(res.persisted.contains(&"MC_PHIE_LOW".to_string()), "persisted: {:?}", res.persisted);
        assert!(res.persisted.contains(&"MC_PHIE_BASE".to_string()));
        assert!(!res.persisted.iter().any(|c| c.contains("PERM")), "no PERM in this chain");

        {
            let conn = dbm.lock().unwrap();
            // Versioned log set registered.
            let (set_name, version): (String, i64) = conn
                .query_row(
                    "SELECT set_name, version FROM log_sets WHERE well_id = ?1",
                    duckdb::params![well],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(set_name, "MONTECARLO");
            assert_eq!(version, 1);
            // Per-sample ordering LOW ≤ P50 ≤ HIGH wherever finite, and the band is real.
            let (_, cols) = equations::fetch_curve_frame(
                &conn,
                &well,
                &["MC_PHIE_LOW".into(), "MC_PHIE_P50".into(), "MC_PHIE_HIGH".into(), "MC_PHIE_BASE".into()],
            )
            .unwrap();
            let lo = &cols["MC_PHIE_LOW"];
            let mid = &cols["MC_PHIE_P50"];
            let hi = &cols["MC_PHIE_HIGH"];
            let mut checked = 0;
            for i in 0..lo.len() {
                if lo[i].is_finite() && mid[i].is_finite() && hi[i].is_finite() {
                    assert!(lo[i] <= mid[i] && mid[i] <= hi[i], "percentile order broken at {i}");
                    checked += 1;
                }
            }
            assert!(checked > 100, "expected finite percentile curves, got {checked} samples");
            assert!(cols["MC_PHIE_BASE"].iter().any(|v| v.is_finite()), "BASE curve must exist");
        }

        // A second persist run never overwrites — it lands as version 2.
        let res2 = run_monte_carlo(&dbm, &req, None);
        assert!(res2.errors.is_empty());
        let conn = dbm.lock().unwrap();
        let versions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM log_sets WHERE well_id = ?1 AND set_name = 'MONTECARLO'",
                duckdb::params![well],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(versions, 2, "persist must version, not overwrite");
    }

    #[test]
    fn configurable_percentiles_widen_the_spread() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 8.0 }, zone: None }];

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

    #[test]
    fn impossible_combo_guard_flags_negative_porosity() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // Matrix density fixed BELOW the measured RHOB (2.35): density porosity goes negative, so
        // the unlimited PHIE_DN is < 0 on every sample — a physically impossible combo the limits
        // clamp to 0 in the volumetrics but which the guard must surface.
        let mc = vec![McParam { param: "RHO_MA".into(), dist: Distribution::Normal { mean: 2.0, sd: 0.0 }, zone: None }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 100, 1), None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        assert_eq!(res.plausibility.len(), 1, "one well → one plausibility row");
        let pl = &res.plausibility[0];
        assert_eq!(pl.realizations, 100);
        assert_eq!(pl.impossible_realizations, 100, "every fixed-below-RHOB realization is impossible");
        assert!((pl.fraction - 1.0).abs() < 1e-6, "fraction should be 1.0, got {}", pl.fraction);
        assert!(pl.detail.contains("PHIE"), "detail should name the porosity violation: {}", pl.detail);
        // The headline study still runs — the guard reports, it never corrupts the result.
        assert_eq!(res.zones.len(), 1);
    }

    #[test]
    fn impossible_combo_guard_flags_supersaturation() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // Cementation exponent pinned at its max drives Indonesia SWE > 1 on the clean-sand samples
        // (the unlimited SWE_INDO), while porosity stays in range — so only the Sw>1 bucket trips.
        let mc = vec![McParam { param: "M".into(), dist: Distribution::Normal { mean: 4.0, sd: 0.0 }, zone: None }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 100, 3), None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        let pl = &res.plausibility[0];
        assert_eq!(pl.impossible_realizations, 100, "every high-M realization is impossible");
        assert!(pl.detail.contains("Sw>1"), "detail should name the Sw>1 violation: {}", pl.detail);
        assert!(!pl.detail.contains("PHIE"), "porosity should stay in bounds: {}", pl.detail);
    }

    #[test]
    fn impossible_combo_guard_clean_run_reports_zero() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);
        // A normal GR-endpoint study on a clean sand: no sample leaves [0,1].
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Normal { mean: 25.0, sd: 6.0 }, zone: None }];
        let res = run_monte_carlo(&dbm, &base_request(&well, mc, 200, 5), None);
        let pl = &res.plausibility[0];
        assert_eq!(pl.impossible_realizations, 0, "clean sand should have no impossible realizations");
        assert_eq!(pl.fraction, 0.0);
        assert!(pl.detail.contains("in bounds"), "clean detail: {}", pl.detail);
    }
}
