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
#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    /// SB-CUT-001 (DEC-071): the thickness discretisation model, shared with the
    /// deterministic pay summary so an MC net can never disagree with it for this reason.
    #[serde(default)]
    pub discretisation: crate::paysummary::DiscretisationModel,
    /// The deterministic chain to run each realization (same shape as a workflow chain).
    pub steps: Vec<ChainStep>,
    pub mc_params: Vec<McParam>,
    pub iterations: usize,
    pub seed: u64,
    #[serde(default)]
    pub custody: Option<crate::ancestry::RunCustody>,
    // Pay/HPV cutoffs (same semantics as the pay summary).
    /// SB-CUT-016. `None` = UNFILTERED on this property. Absent-capable for the same reason the
    /// deterministic summation is: an MC run silently using a shipped 0.5 while the pay summary
    /// reports the property unfiltered is a disagreement nobody could reconcile.
    /// SB-CUT-019: carried as entered, with its unit, and canonicalised before any realization.
    pub vsh_max: Option<crate::paysummary::CutoffSpec>,
    pub phie_min: Option<crate::paysummary::CutoffSpec>,
    pub swe_max: Option<crate::paysummary::CutoffSpec>,
    #[serde(default)]
    pub perm_min: Option<crate::paysummary::CutoffSpec>,
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
    /// SB-CUT-002: the discretisation model and the sample interval these percentile bundles
    /// were computed under — the same identity the deterministic pay summary records.
    pub discretisation_model: String,
    pub sample_interval: f32,
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

#[derive(Default, Debug, Clone, Serialize)]
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
    /// AUDIT-2026-08-20 finding 12. The step's resolved MASK flag curve, on the well's own
    /// depth frame. Fetched ONCE per well here rather than per realization: a flag curve is a
    /// property of the hole, not of the draw, so it is identical in all N realizations.
    mask: Option<Vec<f32>>,
}

/// Cutoffs bundled for the per-zone pay/HPV accumulation.
#[derive(Clone, Copy)]
pub(crate) struct Cutoffs {
    pub(crate) vsh_max: Option<crate::paysummary::CutoffRange>,
    pub(crate) phie_min: Option<crate::paysummary::CutoffRange>,
    pub(crate) swe_max: Option<crate::paysummary::CutoffRange>,
    pub(crate) perm_min: Option<crate::paysummary::CutoffRange>,
}

#[derive(Clone, Copy)]
pub(crate) struct ZoneMetrics {
    pub(crate) net: f32,
    pub(crate) ntg: f32,
    pub(crate) avg_phie: f32,
    pub(crate) avg_swe: f32,
    pub(crate) hpv: f32,
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
///
/// `pub(crate)` for SB-CUT-010: the volumetric identity `HCPV = Net.phi_bar.(1 - Sw_bar)` is
/// asserted against THIS function, because it is a statement about one realization and
/// percentiles do not commute with a product - the P10/P50/P90 bundle cannot carry it.
pub(crate) fn zone_metrics(
    model: crate::paysummary::DiscretisationModel,
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
    // The PHIE floor is SB-CUT-001's sibling and belongs in the same place for the same reason.
    // `paysummary::floored_phie`'s own doc already says it - one function rather than a copy in
    // each pay path - and this was the pay path it had never reached: a Monte Carlo study that
    // varies m/n/Rw over `sw_indo` against a DELIVERED vendor PHIE runs no porosity module, so
    // nothing floored the curve, and a slightly negative streak over tight carbonate (a routine
    // artefact of a sandstone-matrix density porosity, not a corrupt curve) SUBTRACTED its
    // PHIE*(1-SWE)*h from HPV in every realization. The deterministic pay summary floors it and
    // the MC P50 did not, so the two disagreed on the same wells at the same cutoffs and the
    // gap read as uncertainty.
    //
    // Applied HERE rather than at the callers because `v.max(FLOOR)` is idempotent - a caller
    // that already floored loses nothing - so this covers every pay path there is and leaves no
    // second site to forget.
    let phie = crate::paysummary::floored_phie(phie);
    let phie = &phie[..];

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
        // SB-CUT-001: the ONE discretisation rule, shared with run_pay_summary. This was a
        // third inline copy; three copies of a net-pay clamp is three places for it to drift,
        // and a Monte Carlo P50 disagreeing with the deterministic pay summary for that
        // reason would look like uncertainty rather than a bug. Narrow edit under DEC-048.
        let (s_top, s_bot) =
            crate::paysummary::sample_slab(depth[i] as f64, step[i] as f64, model);
        let h = crate::paysummary::sample_incl_thickness(
            s_top,
            s_bot,
            zone.top_depth as f64,
            zone.bottom_depth as f64,
            None,
        );
        if h <= 0.0 {
            continue;
        }
        let v = vsh[i] as f64;
        let p = phie[i] as f64;
        let s = swe[i] as f64;
        if v.is_nan() || p.is_nan() || s.is_nan() {
            continue;
        }
        // SB-CUT-016: an absent cut-off does not filter. The NaN guard above is untouched.
        let mut pay = cut.vsh_max.map_or(true, |r| r.contains(vsh[i]))
            && cut.phie_min.map_or(true, |r| r.contains(phie[i]))
            && cut.swe_max.map_or(true, |r| r.contains(swe[i]));
        if has_perm_cut {
            // A sample with no PERM value cannot demonstrate it passes the cutoff — missing
            // PERM must fail, not silently pass (matches run_pay_summary's classify_sample, so
            // MC and the pay summary agree for identical cutoffs).
            pay = pay && !perm[i].is_nan() && cut.perm_min.unwrap().contains(perm[i]);
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
/// AUDIT-2026-08-20 finding 44. This was a second copy of `distribution`'s R type 7, differing
/// only in taking a fraction and in not clamping it. It is now the shared one — kept as a
/// one-line alias so the call sites below still read as percentiles of a realization set rather
/// than as conversions.
fn percentile(sorted: &[f32], p: f64) -> f32 {
    crate::distribution::percentile_fraction(sorted, p)
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
    model: crate::paysummary::DiscretisationModel,
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
            let m = zone_metrics(model, vsh, phie, swe, perm, depth, step_thick, z, cut, has_perm_cut);
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
    ancestry_inputs: Vec<(String, String, String)>,
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
    let mut ancestry_candidates = Vec::new();
    for (step_index, step ) in steps .iter().enumerate() {
        let spec = &specs[&step.module];
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::LogIn) {
            let mnem = step.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone());
            let up = mnem.trim().to_uppercase();
            if !produced.contains(&up) {
                external.insert(up.clone());
                ancestry_candidates.push((
                    format!("step_{}:{}:{}", step_index + 1, step.module, a.name),
                    up,
                ));
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

    let mut ancestry_inputs = Vec::new();
    for (argument, curve) in ancestry_candidates {
        if crate::ancestry::try_resolve_ancestry_input(conn, well_id, &argument, &curve, None, None)?
            .is_some()
        {
            ancestry_inputs.push((well_id.to_string(), argument, curve));
        }
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
        // Audit finding #3 (AUDIT-2026-08-20): the deterministic runner rejects out-of-spec
        // parameter values at `workflow::resolve_param_arrays`, but a Monte Carlo study resolves
        // its base values HERE, one call site over, and used to skip that guard entirely — so a
        // percent-entered zone override (SWT_IRR = 25) reached `f64::clamp`, whose lo <= hi
        // assert aborts the whole process under the release profile's panic = "abort". Same rule,
        // same message family, same exemption as the deterministic guard: an arg whose range is
        // enforced at the algorithm boundary by an unconditional NumericRange condition is
        // checked there, where it can produce its condition id; spec defaults are trusted and
        // only values a user supplied (step params, zone overrides) are checked.
        let mut bad: Vec<String> = Vec::new();
        for a in spec.args.iter().filter(|a| a.kind == ArgKind::Param) {
            let algorithm_range = a.validity_conditions.iter().any(|condition| {
                matches!(condition.rule, modules::ValidityRule::NumericRange { when: None, .. })
            });
            let range = || match (a.min, a.max) {
                (Some(lo), Some(hi)) => format!("valid {lo} to {hi}"),
                (Some(lo), None) => format!("valid >= {lo}"),
                (None, Some(hi)) => format!("valid <= {hi}"),
                (None, None) => "no declared range".to_string(),
            };
            let in_range =
                |v: f64| a.min.map_or(true, |lo| v >= lo) && a.max.map_or(true, |hi| v <= hi);
            if let Some(&v) = step.params.get(&a.name) {
                if !v.is_finite() || (!algorithm_range && !in_range(v)) {
                    bad.push(format!("{} = {v} ({})", a.name, range()));
                }
            }
            for zp in zone_params.iter().filter(|z| z.param_name == a.name) {
                let Some(v) = zp.value_num else { continue };
                let v = v as f64;
                if !v.is_finite() || (!algorithm_range && !in_range(v)) {
                    bad.push(format!("{} = {v} in zone '{}' ({})", a.name, zp.zone_name, range()));
                }
            }
            param_args.push(a.name.clone());
            let base = step.params.get(&a.name).copied().or_else(|| a.default.parse().ok()).unwrap_or(f64::NAN);
            base_params.insert(a.name.clone(), resolve_zone_param(&a.name, base, &zones_raw, &zone_params, &depth));
        }
        if !bad.is_empty() {
            return Err(format!(
                "step \"{}\": parameter value(s) outside the module's declared range: {}. A \
                 common cause is entering a v/v fraction as a percentage. Fix the value or clear \
                 the zone override.",
                step.module,
                bad.join("; ")
            ));
        }

        // The same resolution the deterministic runner uses, including its VSH_PROV refusal -
        // a Monte Carlo chain must not accept a mask the real chain would reject by name.
        let mask = crate::workflow::fetch_mask_aligned(
            conn,
            well_id,
            opts.get("MASK").map(|s| s.trim()).unwrap_or(""),
            None,
            None,
        )?;
        plans.push(StepPlan {
            module: step.module.clone(),
            opts,
            depth_unit: crate::workflow::resolve_module_depth_unit(conn, &step.module)?,
            log_args,
            param_args,
            base_params,
            mask,
        });
    }

    Ok(WellPlan { plans, raw_pool, ancestry_inputs,
        depth, step_thick, zones: zones_raw, produced })
}

/// Audit finding #3 (AUDIT-2026-08-20): draws and tornado sweep points bypass the declared-range
/// guard in `workflow::resolve_param_arrays` — they are written straight into the parameter
/// arrays after `build_plans`, so nothing between the distribution and the module arithmetic
/// checked them. An unbounded Normal, a mis-typed Uniform bound or a percent-entered mean then
/// reaches code that trusts its declared range; `f64::clamp` panics when the range inverts, and
/// the release profile's panic = "abort" takes the whole process down. Every value the study can
/// apply — each draw, and the tornado's low/base/high points when requested — is checked here
/// against every consuming step's declared ArgSpec range before any realization runs. Args whose
/// range is enforced at the algorithm boundary (unconditional NumericRange) are exempt from the
/// range test but not the finiteness test, exactly as in the deterministic guard. One exemplar
/// violation per parameter is reported: the fix is the distribution, not the realization.
fn sampled_value_violations(
    mc_params: &[McParam],
    steps: &[ChainStep],
    specs: &HashMap<String, modules::ModuleSpec>,
    draws: &[Vec<f64>],
    tornado_pctls: Option<(f64, f64)>,
) -> Vec<String> {
    let mut bad: Vec<String> = Vec::new();
    for (j, mp) in mc_params.iter().enumerate() {
        // Every consuming step's declared range — a parameter name may be shared by several
        // modules with different ranges (sw_indo's SWE_IRR tops out at 0.6, perm_coates' at 0.8),
        // and a value the study applies must satisfy all of them, exactly as a typed value must.
        let mut ranges: Vec<(&str, Option<f64>, Option<f64>, bool)> = Vec::new();
        for step in steps {
            let Some(spec) = specs.get(&step.module) else { continue };
            for a in spec.args.iter().filter(|a| a.kind == ArgKind::Param && a.name == mp.param) {
                let algorithm_range = a.validity_conditions.iter().any(|condition| {
                    matches!(condition.rule, modules::ValidityRule::NumericRange { when: None, .. })
                });
                ranges.push((step.module.as_str(), a.min, a.max, algorithm_range));
            }
        }
        if ranges.is_empty() {
            continue; // not consumed by any step — the draw never reaches a module
        }
        let mut check = |v: f64, whence: &str| -> bool {
            for &(module, min, max, algorithm_range) in &ranges {
                let in_range =
                    min.map_or(true, |lo| v >= lo) && max.map_or(true, |hi| v <= hi);
                if !v.is_finite() || (!algorithm_range && !in_range) {
                    let range = match (min, max) {
                        (Some(lo), Some(hi)) => format!("valid {lo} to {hi}"),
                        (Some(lo), None) => format!("valid >= {lo}"),
                        (None, Some(hi)) => format!("valid <= {hi}"),
                        (None, None) => "no declared range".to_string(),
                    };
                    bad.push(format!("{} = {v} ({module}: {range}) {whence}", mp.param));
                    return true;
                }
            }
            false
        };
        let mut hit = false;
        for (r, row) in draws.iter().enumerate() {
            if check(row[j], &format!("at realization {} of {}", r + 1, draws.len())) {
                hit = true;
                break;
            }
        }
        if !hit {
            if let Some((lo_p, hi_p)) = tornado_pctls {
                for (v, whence) in [
                    (mp.dist.central(), "at the tornado base point"),
                    (mp.dist.quantile(lo_p), "at the tornado low sweep point"),
                    (mp.dist.quantile(hi_p), "at the tornado high sweep point"),
                ] {
                    if check(v, whence) {
                        break;
                    }
                }
            }
        }
    }
    bad
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

/// One tracked curve family's in-memory persistence payload. Keeping this assembly separate from
/// the database write lets the data-custody rule be proved without manufacturing scientifically
/// invalid module inputs merely to make a deterministic base curve missing.
struct PersistedCurveSummary {
    curves: Vec<(String, Vec<f32>)>,
    array: Option<(String, Vec<f32>, Vec<Vec<f32>>)>,
    note: Option<String>,
}

/// Assemble one MC_<KEY> family from realization snapshots plus its deterministic base run.
/// This is the former inline persistence calculation: samples with fewer than eight finite
/// realizations remain missing; a missing/all-missing base drops BASE only when percentile curves
/// still exist; realization order (including NaNs) is preserved in the optional array payload.
fn summarize_persisted_curve(
    key: &str,
    well_name: &str,
    depth: &[f32],
    snapshots: &[&[f32]],
    base: Option<&[f32]>,
    lo_p: f64,
    hi_p: f64,
    persist_realizations: bool,
    real_cap: usize,
) -> PersistedCurveSummary {
    let n = depth.len();
    let mut lo_c = vec![f32::NAN; n];
    let mut mid_c = vec![f32::NAN; n];
    let mut hi_c = vec![f32::NAN; n];
    let mut buf: Vec<f32> = Vec::with_capacity(snapshots.len());
    let cap = real_cap.min(snapshots.len());
    let mut arr_depths: Vec<f32> = Vec::new();
    let mut arr_vals: Vec<Vec<f32>> = Vec::new();
    for i in 0..n {
        buf.clear();
        buf.extend(snapshots.iter().filter_map(|snapshot| snapshot.get(i).copied()).filter(|v| v.is_finite()));
        if persist_realizations {
            // REALIZATION ORDER, NaNs included: index r must mean the same realization at every
            // depth. The sorted `buf` is for percentiles only and must not be stored.
            let col: Vec<f32> = snapshots
                .iter()
                .take(cap)
                .map(|snapshot| snapshot.get(i).copied().unwrap_or(f32::NAN))
                .collect();
            // AUDIT-2026-08-20 finding 39: gated on the SAME population as the percentile curves
            // below, not on the stored column's own finite count. The stored column is a PREFIX
            // of the realizations, so its finite count can only be lower — above the cap a depth
            // can hold plenty of finite realizations overall and fewer than eight inside the
            // prefix, and the store then carried percentile curves at a depth with NO matrix row.
            // The band display reads a missing row as "nothing converged here" and breaks, while
            // the curves in the same track carry values, which is the one thing the two are
            // supposed to agree about.
            //
            // The prefix itself stays: the realizations are Fisher-Yates shuffled per column, so
            // the first `cap` of them are an unbiased subsample rather than a corner of the
            // sampled space. What the cap costs is POPULATION, not representativeness — above it
            // the band summarises fewer realizations than the curves beside it, and the run says
            // so rather than leaving the reader to infer it.
            if buf.len() >= 8 {
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
    let finite_base = base.filter(|curve| curve.iter().any(|v| v.is_finite()));
    let mut curves = Vec::new();
    if have_pct {
        curves.push((format!("MC_{key}_LOW"), lo_c));
        curves.push((format!("MC_{key}_P50"), mid_c));
        curves.push((format!("MC_{key}_HIGH"), hi_c));
    }
    let note = match finite_base {
        Some(curve) => {
            curves.push((format!("MC_{key}_BASE"), curve.to_vec()));
            None
        }
        None if have_pct => Some(format!(
            "{well_name}: MC_{key}_BASE skipped — the all-median base run produced no finite {key}"
        )),
        None => None,
    };
    let array = if arr_depths.is_empty() {
        None
    } else {
        Some((format!("MC_{key}_REAL"), arr_depths, arr_vals))
    };

    PersistedCurveSummary { curves, array, note }
}

/// The scheme name echoed back for the UI badge. One spelling: three results used to write the
/// same two-arm match, so a renamed scheme could reach the badge from one of them and not another.
fn sampling_label(sampling: Sampling) -> &'static str {
    match sampling {
        Sampling::Lhs => "lhs",
        Sampling::Random => "random",
    }
}

/// SB-CUT-019: a cut-off goes into provenance as ENTERED - value AND unit - because "stored with
/// it" is half the requirement. An ABSENT cut-off is recorded as the token, never as JSON `null`:
/// `null` is what a reader sees when a field was not recorded at all, and "we did not filter on
/// this" and "we did not write this down" are different statements about a study.
fn recorded_cutoff(entered: &Option<crate::paysummary::CutoffSpec>) -> serde_json::Value {
    entered
        .as_ref()
        .map(|spec| serde_json::json!(spec))
        .unwrap_or_else(|| serde_json::json!("ABSENT"))
}

/// The four pay cut-offs as they go into a run's provenance record, in one place so a fifth
/// cannot be added past the rule. Pinned against `McRequest`'s own declaration by
/// `every_pay_cutoff_records_its_absence_as_the_token_rather_than_a_bare_null`.
fn recorded_cutoffs(req: &McRequest) -> [(&'static str, serde_json::Value); 4] {
    [
        ("vsh_max", recorded_cutoff(&req.vsh_max)),
        ("phie_min", recorded_cutoff(&req.phie_min)),
        ("swe_max", recorded_cutoff(&req.swe_max)),
        ("perm_min", recorded_cutoff(&req.perm_min)),
    ]
}

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
        // SB-ENV-027 (DEC-033): the ONE approved repair exemption, stated here in the same terms
        // the deterministic runner states it - log_predict's SYN under OPT_COMBINE = MAX_RAW,
        // the mode that genuinely IS a washout repair. Both mask passes are bypassed for it,
        // because a repair blanked at the second pass is a repair that did not happen.
        let repair_run = plan.module == "log_predict"
            && plan.opts.get("OPT_COMBINE").map(|mode| mode.trim() == "MAX_RAW").unwrap_or(false);
        let mut logs: HashMap<String, Vec<f32>> = HashMap::with_capacity(plan.log_args.len() + 1);
        logs.insert("DEPTH".to_string(), depth.to_vec());
        for (arg, mnem) in &plan.log_args {
            let v = pool.get(mnem).cloned().unwrap_or_else(|| vec![f32::NAN; n]);
            logs.insert(arg.clone(), v);
        }
        // AUDIT-2026-08-20 finding 12. Blank flagged samples in the INPUTS before the run, for
        // the reason the deterministic runner states: a module computing a run-level statistic
        // (gr_normalize's P3/P97, log_predict's KNN training set) would otherwise be ANCHORED by
        // casing and washout samples, and that mis-anchoring contaminates every output sample,
        // flagged or not - once per realization. DEPTH is never masked.
        if let Some(mask) = &plan.mask {
            if !repair_run {
                crate::workflow::apply_mask_to_logs(&mut logs, &plan.log_args, Some(mask));
            }
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
                let mut outputs: HashMap<String, Vec<f32>> =
                    outputs.into_iter().map(|(k, v)| (k.to_uppercase(), v)).collect();
                // ...and in the OUTPUTS, so a flagged depth's result is never trusted
                // downstream. `run_module` returns DECLARED keys here (unlike the deterministic
                // runner's resolved, prefixed names), so the two exemptions are named in that
                // vocabulary. The DEC-033 `_RECON_FLAG` companion is deliberately NOT emitted:
                // it discloses reconstruction in a WRITTEN curve, and a realization writes none.
                if let Some(mask) = &plan.mask {
                    crate::workflow::apply_mask_to_outputs(
                        &mut outputs,
                        mask,
                        repair_run.then_some("SYN"),
                        (plan.module == "vsh_gr").then_some("VSH_PROV"),
                    );
                }
                for (k, v) in outputs {
                    pool.insert(k, v);
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
        let Some(spec) = specs.get(&step.module) else { continue;
        };
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
/// SB-CUT-019. Canonicalise every entered cut-off, refusing a bare number, an unknown unit or a
/// physically impossible value.
///
/// Called at the ENTRY POINT - the Tauri command - so the user gets the message. `run_monte_carlo`
/// calls it again and produces nothing on failure, because the job registry fixes that function's
/// return type and a silent guess is the one outcome that must not happen.
pub fn validate_cutoffs(req: &McRequest) -> Result<Cutoffs, String> {
    use crate::paysummary::{CutoffQuantity, CutoffSense};
    let entered = |e: &Option<crate::paysummary::CutoffSpec>,
                   q: CutoffQuantity,
                   sense: CutoffSense,
                   label: &str| {
        e.as_ref().map(|x| x.canonical(q, sense, label)).transpose()
    };
    Ok(Cutoffs {
        vsh_max: entered(
            &req.vsh_max,
            CutoffQuantity::VolumeFraction,
            CutoffSense::Maximum,
            "the VSH cut-off",
        )?,
        phie_min: entered(
            &req.phie_min,
            CutoffQuantity::VolumeFraction,
            CutoffSense::Minimum,
            "the PHIE cut-off",
        )?,
        swe_max: entered(
            &req.swe_max,
            CutoffQuantity::VolumeFraction,
            CutoffSense::Maximum,
            "the SWE cut-off",
        )?,
        perm_min: entered(
            &req.perm_min,
            CutoffQuantity::Permeability,
            CutoffSense::Minimum,
            "the PERM cut-off",
        )?,
    })
}

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
    if req.persist {
        let validation = req
            .custody
            .as_ref()
            .ok_or_else(|| {
                "run refused: enter custody before persisting Monte Carlo curves".to_string()
            })
            .and_then(crate::ancestry::RunCustody::validate);
        if let Err(error) = validation {
            return McResult {
                low_pctl: lo_p,
                high_pctl: hi_p,
                sampling: sampling_label(req.sampling).into(),
                errors: vec![error],
                ..Default::default()
            };
        }
    }
    let specs: HashMap<String, modules::ModuleSpec> =
        modules::list_modules().into_iter().map(|s| (s.name.clone(), s)).collect();
    // SB-CUT-019: the cut-offs were canonicalised and validated by `validate_cutoffs` at the
    // entry point. A run reaching here with an unusable entry produces NOTHING rather than
    // guessing - `run_monte_carlo` cannot return an error because the job registry fixes its
    // return type, so the refusal lives where the user's value first arrives.
    let cut = match validate_cutoffs(req) {
        Ok(cut) => cut,
        Err(_) => return McResult::default(),
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

    // Refuse the whole study, not per well: the draw matrix is shared across wells, so every
    // well would apply the same out-of-range values. Same message family as the deterministic
    // runner's declared-range guard; refused rather than clamped, because silently clamping a
    // percent-entered Normal(25, 1) to 0.6 would hand back plausible-but-wrong percentiles.
    let bad_draws = sampled_value_violations(
        &req.mc_params,
        &req.steps,
        &specs,
        &draws_all,
        if req.tornado { Some((lo_p, hi_p)) } else { None },
    );
    if !bad_draws.is_empty() {
        return McResult {
            low_pctl: lo_p,
            high_pctl: hi_p,
            sampling: sampling_label(req.sampling).into(),
            notes,
            errors: vec![format!(
                "Monte Carlo draw(s) outside the module's declared range: {}. A common cause is \
                 entering a v/v fraction as a percentage in the distribution. Fix the \
                 distribution's parameters so every value it can apply stays inside the declared \
                 range.",
                bad_draws.join("; ")
            )],
            ..Default::default()
        };
    }

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
        let WellPlan { plans, raw_pool, ancestry_inputs,
            depth, step_thick, mut zones, produced } = wp;
        let n = depth.len();
        let had_declared_zones = !zones.is_empty();
        if zones.is_empty() {
            zones.push(ZoneEntry {
                zone_name: "ALL".into(),
                top_depth: depth[0],
                bottom_depth: *depth.last().unwrap(),
                depth_datum: crate::schema_vocab::DepthDatum::Md,
            });
        }
        // First module failure anywhere in this well's sweep. A failed step used to be dropped,
        // leaving the pool unchanged so every downstream step read NaN and the well came back as
        // a P10=P50=P90 table of zeros — a confident-looking uncertainty study of a chain that
        // never ran. Reported per well after the sweep.
        let step_err: std::sync::OnceLock<String> = std::sync::OnceLock::new();

        // A requested permeability cutoff is ALWAYS active — DEC-084 (2026-08-20), the same
        // rule run_pay_summary follows. Whether a step PRODUCES perm or the project carries it
        // decides what the samples read, never whether the cutoff applies: finding 8 closed the
        // produced-curve half in 2026-08-01, but a run with no PERM anywhere still exempted
        // itself — the less-data-books-more-pay inversion, closed at the well level in
        // workflow.rs (finding 7) and closed here for Monte Carlo. Every sample without a PERM
        // value fails the cutoff in `zone_metrics` for want of evidence, and the advisory below
        // is what separates that zero from a wet reservoir on the result.
        let has_perm_cut = req.perm_min.is_some();
        if has_perm_cut
            && !produced.contains("PERM")
            && !raw_pool.get("PERM").map(|c| c.iter().any(|v| !v.is_nan())).unwrap_or(false)
        {
            notes.push(format!(
                "{well_name}: permeability cutoff is active but no PERM exists on this well \
                 (no curve, no model step) — every sample fails it, so zero net/HPV here is \
                 absence of evidence, not a wet reservoir"
            ));
        }

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
                .map(|z| zone_metrics(req.discretisation, vsh, phie, swe, perm, &depth, &step_thick, z, &cut, has_perm_cut))
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
                    req.discretisation,
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
                    req.discretisation,
                        &plans, &raw_pool, &depth, &step_thick, &zones, &cut, has_perm_cut,
                        &req.mc_params, &spans, &lv, n, &step_err,
                    ));
                    high.push(metrics_for_values(
                    req.discretisation,
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
                discretisation_model: req.discretisation.token().to_string(),
                sample_interval: crate::paysummary::median_sample_interval(&step_thick),
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
            // AUDIT-2026-08-20 finding 39. Above the cap the stored matrix and the percentile
            // curves describe DIFFERENT POPULATIONS — same depths, same realizations by identity,
            // but the band is drawn over `real_cap` of them and MC_*_LOW/_HIGH over all `kept`.
            // At matching percentiles the two will then be close but not equal, and a reader
            // comparing a band against the curve beside it deserves to know why rather than
            // hunting for a bug. Said once per run, not once per curve.
            if req.persist_realizations && real_cap < kept {
                notes.push(format!(
                    "{well_name}: realization matrices store {real_cap} of {kept} realizations \
                     (realization_cap); a band drawn from them summarises that subsample, while \
                     MC_*_LOW/_P50/_HIGH summarise all {kept}"
                ));
            }
            let mut out: Vec<(String, Vec<f32>)> = Vec::new();
            // (curve name, depths, per-depth realization vectors) for the array store.
            let mut arrays: Vec<(String, Vec<f32>, Vec<Vec<f32>>)> = Vec::new();
            for (t, key) in TRACKED.iter().enumerate() {
                if !produced.contains(*key) {
                    continue;
                }
                let snaps: Vec<&[f32]> = per_real
                    .iter()
                    .filter_map(|m| m.2.as_ref().map(|s| s[t].as_slice()))
                    .collect();
                let summary = summarize_persisted_curve(
                    key,
                    &well_name,
                    &depth,
                    &snaps,
                    base_pool.get(*key).map(Vec::as_slice),
                    lo_p,
                    hi_p,
                    req.persist_realizations,
                    real_cap,
                );
                if summary.curves.is_empty() {
                    continue;
                }
                out.extend(summary.curves);
                if let Some(note) = summary.note {
                    notes.push(note);
                }
                if let Some(array) = summary.array {
                    arrays.push(array);
                }
            }
            if out.is_empty() {
                notes.push(format!("{well_name}: nothing to persist — no tracked output curve had finite values"));
            } else {
                let conn = db.lock().unwrap();
                let mut parameters = serde_json::json!({
                        "iterations": used_iterations,
                        "requested_iterations": req.iterations,
                    "seed": req.seed,
                        "sampling": req.sampling,
                        "low_pctl": lo_p,
                        "high_pctl": hi_p,
                        "kept_realizations": kept,
                        "mc_params": req.mc_params,
                    "correlations": req.correlations, "steps":req.steps,
                    "bins": req.bins,
                    "sensitivity": req.sensitivity,
                    "tornado": req.tornado,
                    "converge": req.converge,
                    "converge_tol": req.converge_tol,
                    "persist_realizations": req.persist_realizations,
                    "realization_cap": req.realization_cap.map(serde_json::Value::from).unwrap_or_else(|| serde_json::json!("ABSENT")),
                });
                // Every pay cut-off records its absence the same way. Written through the shared
                // rule rather than four times into the literal above, which is how three of them
                // came to serialize as a bare `null` while the fourth carried the token.
                if let Some(record) = parameters.as_object_mut() {
                    for (key, value) in recorded_cutoffs(req) {
                        record.insert(key.to_string(), value);
                    }
                }
                let zone_scope = if had_declared_zones {
                    crate::ancestry::AncestryZoneScope::Defined(
                        zones
                            .iter()
                            .filter(|zone| zone.top_depth < zone.bottom_depth)
                            .map(|zone| crate::ancestry::AncestryZone {
                                name: zone.zone_name.clone(),
                                top: zone.top_depth,
                                base: zone.bottom_depth,
                                source: req
                                    .custody
                                    .as_ref()
                                    .expect("persistence custody validated")
                                    .source_note
                                    .clone(),
                            })
                            .collect(),
                    )
                } else {
                    crate::ancestry::AncestryZoneScope::WholeWell
                };
                let output_names = out.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
                let spec = crate::ancestry::complete_curve_run_spec(
                    &conn,
                    well_id,
                    "MONTECARLO",
                    "montecarlo",
                    req.custody.as_ref().expect("persistence custody validated"),
                    &ancestry_inputs,
                    None,
                    parameters,
                    zone_scope,
                    &output_names,
                );
                // Reclaim the ENTIRE MC_* family in the current store first: a previous
                // MONTECARLO version may have written keys this run doesn't (e.g. PERM dropped
                // from the chain), and the versioned writer deletes only the names it writes —
                // stale cross-version rows would otherwise keep serving next to this run's
                // curves and keep the old version flagged current. The archive keeps every
                // version restorable.
                let family: Vec<String> = TRACKED
                    .iter()
                    .flat_map(|k| {
                        ["LOW", "P50", "HIGH", "BASE"].into_iter().map(move |s| format!("MC_{k}_{s}"))})
                    .collect();
                let write = spec.and_then(|spec| {
                        let (set_id, version) =
                        crate::ancestry::create_complete_log_set(&conn, well_id, &spec)?;
                    let refs: Vec<(&str, &[f32])> = out.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
                        crate::ancestry::write_computed_curves_with_ancestry_clearing(&conn, well_id, &depth, &refs, &family, &set_id)
                            ?;
                    Ok(version)
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
                                Err(e) => {
                            notes.push(format!("{well_name}: {name} not stored — {e}"));
                            }
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
        sampling: sampling_label(req.sampling).into(),
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
        // CHARACTERIZATION fixtures: these values are the pre-SB-CORE-004 manifest inputs that
        // existing Monte Carlo tests previously consumed implicitly. They are explicit here so
        // the tests continue to isolate sampling, persistence, masking, and cutoff behavior; none
        // of them is restored as a shipping default.
        let params = match module {
            "vsh_gr" => HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            // phi_den, not phi_dn: DEC-070 (2026-08-18) made the D-N quick-look curve
            // visual-only, so a chain that feeds pay uses the authoritative density
            // method - these fixtures are about sampling/masking/cutoffs, not the ruling.
            "phi_den" => HashMap::from([
                ("RHO_MA".into(), 2.645),
                ("RHO_SH".into(), 2.5),
                ("RHO_FL".into(), 1.0),
                ("RHO_DSH".into(), 2.65),
                ("RHO_W".into(), 1.0),
                ("PHIE_MAX".into(), 0.3),
            ]),
            "sw_indo" => HashMap::from([
                ("A".into(), 1.0),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("RT_SH".into(), 5.0),
                ("SWE_IRR".into(), 0.0),
                ("RW".into(), 0.1),
            ]),
            "perm_coates" => {
                HashMap::from([("CONST_COATES".into(), 100.0), ("SWE_IRR".into(), 0.15)])
            }
            "rocktyping" => HashMap::from([("PS_EXP".into(), 3.0)]),
            _ => HashMap::new(),
        };
        ChainStep {
            module: module.into(),
            log_inputs: HashMap::new(),
            params,
            opts: HashMap::new(),
        }
    }

    /// A clean-ish sand: low GR, moderate porosity, low water saturation, so vsh_gr → phi_den →
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
        db::insert_standard_curves_as_opened_project(conn, id, depth, gr, res, nphi, rhob, dt, sp).unwrap();
        id.to_string()
    }

    /// AUDIT-2026-08-20 finding 82. SB-CUT-019 says an absent cut-off is recorded as the token
    /// ABSENT - and the rule was stated on `perm_min` alone, so the other three serialized as a
    /// bare `null`. `null` is what a reader sees when a field was never recorded at all, and "we
    /// did not filter on this property" and "we did not write this down" are different statements
    /// about a study. Both sides: an entered cut-off must still record its value and unit.
    #[test]
    fn every_pay_cutoff_records_its_absence_as_the_token_rather_than_a_bare_null() {
        // Three entered, one absent - the shape a real run routinely has.
        let req = base_request("SANDI-1", Vec::new(), 8, 1);
        let recorded = recorded_cutoffs(&req);
        assert_eq!(
            recorded.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            ["vsh_max", "phie_min", "swe_max", "perm_min"],
            "the record names every pay cut-off the request carries",
        );
        for (key, value) in &recorded {
            assert!(!value.is_null(), "{key} serialized as a bare null instead of stating its absence");
        }
        assert_eq!(recorded[3].1, serde_json::json!("ABSENT"), "an absent cut-off records the token");
        assert_eq!(recorded[1].1["value"], serde_json::json!(0.08), "an entered cut-off keeps its value");
        assert_eq!(recorded[1].1["unit"], serde_json::json!("v/v"), "and the unit it was entered in");

        // A fifth cut-off added to the request would otherwise skip the rule entirely, which is
        // exactly how three of these four came to write a bare null in the first place.
        let source = include_str!("montecarlo.rs");
        let start = source.find("pub struct McRequest {").expect("the request is declared here");
        // The needle is a newline and a closing brace, with no trailing newline: this source is
        // read exactly as it sits on disk, and a working tree checked out with CRLF would
        // otherwise never find the end of the declaration at all. Spelling the pair out in prose
        // rather than in backticks is deliberate - `production_rust` decides where this file's
        // test module ends by counting braces, and a brace in a comment counts.
        let body = &source[start..][..source[start..].find("\n}").expect("the declaration closes")];
        assert_eq!(
            body.matches("Option<crate::paysummary::CutoffSpec>").count(),
            recorded.len(),
            "a pay cut-off was added to the request without a place in the provenance record",
        );
    }

    fn base_request(well: &str, mc: Vec<McParam>, iterations: usize, seed: u64) -> McRequest {
        McRequest {
            well_ids: vec![well.into()],
            // DEC-071: MC fixtures keep their hand-derived FORWARD expectations.
            discretisation: crate::paysummary::DiscretisationModel::Forward,
            steps: vec![step("vsh_gr"), step("phi_den"), step("sw_indo")],
            mc_params: mc,
            iterations,
            seed,
            custody: Some(crate::workflow::test_run_custody()),
            vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(crate::paysummary::CutoffEntry { value: 0.08, unit: "v/v".into() }.into()),
            swe_max: Some(crate::paysummary::CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
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

    /// Audit finding #3 (AUDIT-2026-08-20), the plan-time side. The deterministic runner refuses
    /// an out-of-spec parameter at `workflow::resolve_param_arrays`; a Monte Carlo study resolves
    /// its base values in `build_plans` and used to skip that guard entirely, so a
    /// percent-entered zone override (SWE_IRR = 25 against sw_indo's declared 0..0.6) reached
    /// `f64::clamp`, whose lo <= hi assert aborts the whole process under the release profile's
    /// panic = "abort". Both sides: the override refuses by name as a per-well error, and the
    /// same override at a legal value runs to a full result — so neither an always-refuse nor a
    /// never-refuse mutation passes.
    #[test]
    fn a_percent_entered_zone_override_refuses_the_study_instead_of_aborting_the_process() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // The classic typo: an irreducible saturation entered as a percentage. The zones dialog
        // and the DB Inspector both write zone_params without the range check moduleDialog.ts
        // applies to typed values — the override is designed to beat the dialog — so this is the
        // supply route that really reaches a study unchecked.
        db::set_zone_param(&conn, &well, "*", "SWE_IRR", Some(25.0), None).unwrap();
        let db_mutex = Mutex::new(conn);
        let mc = vec![McParam {
            param: "M".into(),
            dist: Distribution::Uniform { lo: 1.8, hi: 2.2 },
            zone: None,
        }];
        let req = base_request(&well, mc, 20, 42);
        let res = run_monte_carlo(&db_mutex, &req, None);
        assert!(res.zones.is_empty(), "an out-of-range override must not produce percentiles");
        assert_eq!(res.errors.len(), 1, "expected one per-well refusal, got {:?}", res.errors);
        let msg = &res.errors[0];
        assert!(
            msg.contains("SWE_IRR = 25") && msg.contains("declared range"),
            "the refusal must name the parameter, the value and the rule: {msg}"
        );
        assert!(msg.contains("percentage"), "the refusal must state the common cause: {msg}");

        // The same override at a legal value is applied, not refused.
        {
            let conn = db_mutex.lock().unwrap();
            db::set_zone_param(&conn, &well, "*", "SWE_IRR", Some(0.3), None).unwrap();
        }
        let ok = run_monte_carlo(&db_mutex, &req, None);
        assert!(ok.errors.is_empty(), "a legal override must run clean: {:?}", ok.errors);
        assert!(!ok.zones.is_empty(), "a legal override must produce results");
    }

    /// Audit finding #3, the draw side. A distribution is user input like any typed value, but
    /// its draws are written into the parameter arrays after `build_plans`, bypassing every
    /// declared-range check — a Normal mean entered as a percentage sent SWE_IRR ~ 25 into the
    /// same aborting clamp, and an honestly-meant wide Normal can stray past a bound on any
    /// realization. The study refuses up front, study-wide (the draw matrix is shared across
    /// wells), naming the parameter, the consuming module, its declared range and an exemplar
    /// realization; the identical study drawn inside the range runs to a full result.
    #[test]
    fn a_draw_that_leaves_the_declared_range_refuses_the_study_by_name() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let db_mutex = Mutex::new(conn);
        let mc = vec![McParam {
            param: "SWE_IRR".into(),
            dist: Distribution::Normal { mean: 25.0, sd: 1.0 },
            zone: None,
        }];
        let res = run_monte_carlo(&db_mutex, &base_request(&well, mc, 50, 42), None);
        assert!(res.zones.is_empty(), "out-of-range draws must not produce percentiles");
        assert_eq!(res.errors.len(), 1, "expected one study-wide refusal, got {:?}", res.errors);
        let msg = &res.errors[0];
        assert!(
            msg.contains("SWE_IRR") && msg.contains("sw_indo") && msg.contains("0.6"),
            "the refusal must name the parameter, the consuming module and the declared range: {msg}"
        );
        assert!(msg.contains("realization"), "the refusal must locate an exemplar draw: {msg}");

        // The same study drawn inside the declared range runs to a full result. Uniform on
        // purpose — its support is bounded, so this side can never flake on a tail draw.
        let mc_ok = vec![McParam {
            param: "SWE_IRR".into(),
            dist: Distribution::Uniform { lo: 0.05, hi: 0.25 },
            zone: None,
        }];
        let ok = run_monte_carlo(&db_mutex, &base_request(&well, mc_ok, 50, 42), None);
        assert!(ok.errors.is_empty(), "an in-range distribution must run clean: {:?}", ok.errors);
        assert!(!ok.zones.is_empty(), "an in-range distribution must produce results");
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

        let run = |steps: Vec<ChainStep>, perm_min: Option<crate::paysummary::CutoffSpec>| -> McResult {
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
        let reads_perm = || vec![step("vsh_gr"), step("phi_den"), step("sw_indo"), step("rocktyping")];
        let a_open = run(reads_perm(), None);
        let a_cut = run(reads_perm(), Some(crate::paysummary::CutoffEntry { value: 1.0e9, unit: "mD".into() }.into()));
        assert!(a_open.zones[0].net.mid > 0.0, "the well must have pay before any cutoff");
        assert_eq!(a_cut.zones[0].net.mid, 0.0, "1 mD cannot pass a 1e9 mD cutoff — the cutoff works here");

        // Chain B — the SAME chain with a permeability model inserted ahead of it. PERM is now a
        // PRODUCED curve, so it never enters the external pool at all; the cutoff has to find it
        // in `produced` instead.
        let makes_perm =
            || vec![step("vsh_gr"), step("phi_den"), step("sw_indo"), step("perm_coates"), step("rocktyping")];
        let b_open = run(makes_perm(), None);
        let b_cut = run(makes_perm(), Some(crate::paysummary::CutoffEntry { value: 1.0e9, unit: "mD".into() }.into()));
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
        let b_loose = run(makes_perm(), Some(crate::paysummary::CutoffEntry { value: 1.0e-9, unit: "mD".into() }.into()));
        assert_eq!(
            b_loose.zones[0].net.mid, bo.net.mid,
            "a cutoff the modelled permeability passes must not remove pay"
        );
    }

    /// DEC-084 (2026-08-20, Jauhar: a well with no perm cannot escape the cutoff, and the
    /// cutoff is independent of the chain) — a Monte Carlo run with NO permeability anywhere
    /// still answers to an active permeability cutoff.
    ///
    /// The 2026-08-01 fix above taught `has_perm_cut` to see PRODUCED perm, but the gate still
    /// asked whether perm exists at all, so a run with no PERM curve and no perm model quietly
    /// exempted itself — the same less-data-books-more-pay inversion `run_pay_summary` closed at
    /// the well level (finding 7). Now the request alone decides: every sample without a PERM
    /// value fails the cutoff for want of evidence, and the advisory note is what separates that
    /// zero from a wet reservoir. The control half (no cutoff → pay survives) keeps this from
    /// being satisfied by a chain that is simply broken.
    #[test]
    fn a_run_with_no_permeability_anywhere_still_answers_to_an_active_perm_cutoff() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // Deliberately NO PERM curve in the project and no permeability model in the chain.
        let dbm = Mutex::new(conn);

        let run = |perm_min: Option<crate::paysummary::CutoffSpec>| -> McResult {
            let mc = vec![McParam {
                param: "GR_MA".into(),
                dist: Distribution::Normal { mean: 25.0, sd: 5.0 },
                zone: None,
            }];
            let mut req = base_request(&well, mc, 24, 42);
            req.steps = vec![step("vsh_gr"), step("phi_den"), step("sw_indo")];
            req.perm_min = perm_min;
            let res = run_monte_carlo(&dbm, &req, None);
            assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
            res
        };

        // Control: the same chain with no cutoff has pay — so the zero below is the cutoff's
        // verdict, not a chain that never ran.
        let open = run(None);
        assert!(open.zones[0].net.mid > 0.0, "the well must have pay before any cutoff");
        assert!(
            open.notes.iter().all(|n| !n.contains("permeability cutoff")),
            "no advisory without a cutoff: {:?}",
            open.notes
        );

        // Active cutoff, no PERM anywhere: every sample fails for want of evidence.
        let cut = run(Some(crate::paysummary::CutoffEntry { value: 0.1, unit: "mD".into() }.into()));
        assert_eq!(cut.zones[0].net.mid, 0.0, "missing permeability cannot pass an active cutoff");
        assert_eq!(cut.zones[0].hpv.mid, 0.0, "and books no hydrocarbon volume on missing data");
        assert!(
            cut.notes.iter().any(|n| n.contains("permeability cutoff") && n.contains("absence of evidence")),
            "the advisory must separate absence of evidence from a wet reservoir: {:?}",
            cut.notes
        );
    }

    /// T-BATCH-17 / AUDIT-2026-08-20 finding 12 — CLOSED. A step's MASK is now READ by the
    /// Monte Carlo engine, not merely carried.
    ///
    /// It used to be carried and ignored, so the engine interpreted washout as rock and
    /// reported MORE pay than the very same chain did — the dangerous direction, since a batch
    /// study is what gets quoted. There were TWO causes and the fix had to close both, exactly
    /// as this test's earlier doc block warned: `build_plans` never fetched the flag curve
    /// (`external` is assembled from LogIn mnemonics and MASK is an Option), and
    /// `run_realization` never blanked. `StepPlan.mask` closes the first, once per well rather
    /// than once per realization; the two shared `workflow::apply_mask_to_*` functions close the
    /// second. Shared, not copied, because two copies of a mask rule is two places for it to
    /// drift — which is how this gap opened.
    ///
    /// The test now pins AGREEMENT: the masked chain and the masked Monte Carlo must land on the
    /// same net, and — the half that makes that meaningful — dropping the mask must CHANGE the
    /// Monte Carlo answer. An engine that blanked everything, or nothing, would satisfy one of
    /// those and not the other.
    #[test]
    fn a_monte_carlo_chain_honours_a_step_mask_exactly_as_the_real_chain_does() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);

        // Cased-off over the shallow third — the same grid `seed_well` laid down.
        //
        // Deliberately NOT BADHOLE. SB-POR-047 made hole quality a DECLARED input on the
        // porosity methods, so `build_plans` fetches BADHOLE from the LogIn list and `phi_den`
        // excludes the washout natively, mask or no mask - which is why this test could once
        // report agreement while the MASK mechanism was completely inert. CASING is a flag no
        // module declares, so the MASK option is the ONLY route it can take, and the test is
        // about the mechanism again rather than about one module's argument list.
        let n = 300usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let cased: Vec<f32> = (0..n).map(|i| if i < 100 { 1.0 } else { 0.0 }).collect();
        crate::equations::write_computed_curve(&conn, &well, &depth, "CASING", &cased).unwrap();
        let dbm = Mutex::new(conn);

        let masked = || HashMap::from([("MASK".to_string(), "CASING".to_string())]);
        let modules_in_chain = ["vsh_gr", "phi_den", "sw_indo"];

        // The real chain, one masked step at a time, then the pay summary over what it wrote.
        for m in modules_in_chain {
            let params = step(m).params;
            let res = crate::workflow::run_workflow_module(
                &dbm,
                &crate::workflow::RunModuleRequest {
                    module: m.into(),
                    well_ids: vec![well.clone()],
                    log_inputs: HashMap::new(),
                    params,
                    opts: masked(),
                    output_set: None,
                    input_set: None,
                    custody: crate::workflow::test_run_custody(),
                },
            );
            assert!(res[0].error.is_none(), "{m} failed: {:?}", res[0].error);
        }
        let chain_rows = crate::paysummary::run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &crate::paysummary::PaySummaryRequest {
                // DEC-071: compared against the FORWARD MC fixture above.
                discretisation: crate::paysummary::DiscretisationModel::Forward,
                input_set: None,
                well_ids: vec![well.clone()],
                vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                phie_min: Some(crate::paysummary::CutoffEntry { value: 0.08, unit: "v/v".into() }.into()),
                swe_max: Some(crate::paysummary::CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                perm_min: None,
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                skip_version: false,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
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
        // The mask EXCLUDES the cased-off interval; it does not blank the well. An engine that
        // over-applies - masking every sample, or masking DEPTH along with the logs - would
        // satisfy "less pay than unmasked" perfectly while destroying the study.
        assert!(
            mc_net > 0.0,
            "the masked Monte Carlo must still find pay below the casing, got net {mc_net}"
        );

        // SB-POR-047 CLOSED this observable for chains carrying a porosity module: hole quality
        // is a DECLARED input now, and `build_plans` assembles its fetch list from LogIn
        // mnemonics — so the flag curve rides into the realization and phi_den excludes the
        // washout natively, mask or no mask. The MASK-ignoring MECHANISM this test's doc block
        // describes still exists (`run_realization` still never blanks on MASK); it simply no
        // longer has anything left to inflate here, and the equality is the proof.
        assert!(
            (mc_net - chain_net).abs() < 1e-3,
            "the masked chain and the masked Monte Carlo must land on the same net: MC {mc_net} vs chain {chain_net}"
        );

        // ...and the half that makes that agreement worth anything: dropping the mask must
        // CHANGE the Monte Carlo answer, upwards. An engine that blanked EVERYTHING would pass
        // the agreement above (both zero) and fail here; an engine that blanked NOTHING - the
        // defect - passes here only if the numbers happen to coincide, which they do not,
        // because the cased-off third carries pay when nothing excludes it.
        let mut unmasked = req.clone();
        unmasked.steps = modules_in_chain.iter().map(|m| step(m)).collect();
        let mc_unmasked = run_monte_carlo(&dbm, &unmasked, None).zones[0].net.mid;
        assert!(
            mc_unmasked > mc_net + 1e-3,
            "ignoring the mask must interpret the cased-off interval as rock and report MORE pay: unmasked {mc_unmasked} vs masked {mc_net}"
        );

        // Cause two was "the flag curve never enters the pool" - `external` is assembled from
        // LogIn mnemonics and MASK is an Option, so no module declaring CASING means nothing
        // fetched it. StepPlan.mask resolves it directly, once per well.
        {
            let conn = dbm.lock().unwrap();
            let specs: HashMap<String, modules::ModuleSpec> =
                modules::list_modules().into_iter().map(|s| (s.name.clone(), s)).collect();
            let wp = build_plans(&conn, &well, &req.steps, &specs).expect("plans build");
            assert!(
                !wp.raw_pool.contains_key("CASING"),
                "CASING is not a declared input of any step, so it must NOT arrive through the LogIn pool - if it did, this test would no longer be about the MASK mechanism"
            );
            let mask = wp.plans[0].mask.as_ref().expect("the step's MASK must be RESOLVED, not merely carried");
            assert_eq!(mask.len(), n, "the flag curve arrives on the well's own depth frame");
            assert!(
                mask.iter().take(100).all(|f| modules::sample_is_flagged(*f))
                    && mask.iter().skip(100).all(|f| !modules::sample_is_flagged(*f)),
                "and it is the curve that was written, not a placeholder"
            );
        }
    }

    #[test]
    fn hpv_distribution_is_ordered_and_reproducible() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        // The test's subject is percentile ordering and seed reproducibility, not an unbounded
        // Gaussian prior. Use the original fixture's mean ± one stated SD as an explicitly bounded
        // interval so no realization violates `vsh_gr`'s declared GR_MA range. Gaussian truncation
        // is the separate, deferred SB-CUT-038 contract and must not be invented here.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 17.0, hi: 33.0 }, zone: None }];
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
        // Sensitivity reproducibility is the subject. The bounded interval is the original 25 ± 8
        // fixture; an unbounded Gaussian would require the separately deferred SB-CUT-038
        // truncation policy.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 17.0, hi: 33.0 }, zone: None }];
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
        db::upsert_md_zone(&conn, &well, "UPPER", 1000.0, 1075.0).unwrap();
        db::upsert_md_zone(&conn, &well, "LOWER", 1075.0, 1150.0).unwrap();
        let dbm = Mutex::new(conn);

        let mc = vec![McParam {
            param: "GR_MA".into(),
            // Preserve the original fixture's mean ± stated SD as a bounded input; the subject is
            // zone scoping, while Gaussian truncation remains SB-CUT-038.
            dist: Distribution::Uniform { lo: 10.0, hi: 40.0 },
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
            dist: Distribution::Uniform { lo: 10.0, hi: 40.0 },
            zone: Some("NOPE".into()),
        }];
        let res2 = run_monte_carlo(&dbm, &base_request(&well, mc2, 100, 42), None);
        assert!(res2.notes.iter().any(|n| n.contains("NOPE")), "unknown zone should note: {:?}", res2.notes);
        for z in &res2.zones {
            assert_eq!(z.hpv.lo, z.hpv.hi, "unknown zone scope must not vary anything");
        }
    }

    fn seed_computed(conn: &Connection, well: &str, name: &str, value: f32) {
        let depth = (0..300)
            .map(|i| 1000.0 + i as f32 * 0.5)
            .collect::<Vec<_>>();
        let values = vec![value; depth.len()];
        equations::write_computed_curves_batch(conn, well, &depth, &[(name, &values)]).unwrap();
    }

    #[test]
    fn inverted_zone_notes_instead_of_panicking() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        // Storable through the DB inspector (no validation on that path): top ≥ bottom.
        db::upsert_md_zone(&conn, &well, "BAD", 1100.0, 1050.0).unwrap();
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

    /// AUDIT-2026-08-20 finding 39. `CLAUDE.md` states the stored matrix and the percentile
    /// curves "never disagree about where an answer exists". That held only BELOW the storage
    /// cap - the only regime the pin above exercises, which is how it stayed unnoticed. The
    /// matrix was gated on the finite count inside the stored PREFIX and the curves on the count
    /// across every realization, and a prefix can only hold fewer. Above the cap the store
    /// therefore carried percentile curves at depths with NO matrix row, and the band display
    /// reads a missing row as "nothing converged here" while the curve beside it plots a value.
    ///
    /// Driven directly rather than through a run: the disagreement needs realizations that fail
    /// at SOME depths and not others, which a well-behaved fixture never produces - and a test
    /// that cannot reach the condition it names is the reason this survived a pin already.
    ///
    /// Pinned from both sides: the depth sets must match, AND the matrix must still be capped -
    /// otherwise simply storing every realization would satisfy the first half.
    #[test]
    fn the_stored_matrix_covers_every_depth_the_curves_answer_even_above_the_storage_cap() {
        let depth = [1000.0f32, 1000.5];
        // Twenty realizations, eight of them stored. At the SHALLOW depth only three of the first
        // eight converged, while fifteen of the twenty did - so the curves answer there and the
        // stored prefix, on its own, does not. The deep sample converged everywhere.
        let snaps: Vec<Vec<f32>> = (0..20)
            .map(|r| {
                let shallow = if r < 8 && r % 3 != 0 { f32::NAN } else { 0.30 + r as f32 * 0.001 };
                vec![shallow, 0.40 + r as f32 * 0.001]
            })
            .collect();
        let refs: Vec<&[f32]> = snaps.iter().map(|s| s.as_slice()).collect();
        assert_eq!(
            refs.iter().filter(|s| s[0].is_finite()).count(),
            15,
            "fixture: the shallow depth must be answerable across all realizations"
        );
        assert_eq!(
            refs.iter().take(8).filter(|s| s[0].is_finite()).count(),
            3,
            "fixture: and NOT answerable from the stored prefix alone - that is the whole case"
        );

        let summary =
            summarize_persisted_curve("SWE", "SANDI-39", &depth, &refs, None, 0.10, 0.90, true, 8);
        let (_, arr_depths, arr_vals) = summary.array.expect("the matrix must be produced");
        let p50 = summary
            .curves
            .iter()
            .find(|(name, _)| name == "MC_SWE_P50")
            .map(|(_, values)| values.clone())
            .expect("the percentile curves must be produced");

        // B first, because it is what makes A meaningful: this really is the capped regime.
        assert!(
            arr_vals.iter().all(|col| col.len() == 8),
            "every stored depth must keep exactly the capped eight realization slots"
        );

        // A. Every depth the curves answer has a matrix row, and no row exists anywhere else.
        let answered: Vec<f32> = depth
            .iter()
            .zip(p50.iter())
            .filter(|(_, value)| value.is_finite())
            .map(|(d, _)| *d)
            .collect();
        assert_eq!(answered.len(), 2, "both depths must carry a percentile answer");
        assert_eq!(
            arr_depths, answered,
            "a depth with a percentile answer and no matrix row is the disagreement this pins"
        );
    }



    #[test]
    fn persist_reclaims_stale_family_and_degenerate_base_drops_only_base() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = seed_well(&conn);
        let dbm = Mutex::new(conn);

        // Run 1: full chain → MC_VSH/PHIE/SWE families all written (v1).
        // Persistence is the subject. Keep the original 25 ± 8 fixture inside an explicit bounded
        // distribution rather than silently adding the deferred SB-CUT-038 truncation policy.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 17.0, hi: 33.0 }, zone: None }];
        let mut req1 = base_request(&well, mc, 200, 42);
        req1.persist = true;
        let res1 = run_monte_carlo(&dbm, &req1, None);
        assert!(res1.persisted.iter().any(|c| c.starts_with("MC_PHIE_")));

        // Run 2: a valid vsh_gr-only chain proves that the persistence transaction reclaims
        // families which the new chain no longer produces. Degenerate-base assembly is exercised
        // separately below without asking an invalid GR_MA >= GR_SH pair to compute.
        let mut s = step("vsh_gr");
        s.params.insert("GR_SH".into(), 100.0);
        let mc2 = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 20.0, hi: 30.0 }, zone: None }];
        let mut req2 = base_request(&well, mc2, 300, 7);
        req2.steps = vec![s];
        req2.persist = true;
        let res2 = run_monte_carlo(&dbm, &req2, None);
        assert!(res2.errors.is_empty(), "unexpected errors: {:?}", res2.errors);
        assert!(res2.persisted.contains(&"MC_VSH_LOW".to_string()), "persisted: {:?}", res2.persisted);
        assert!(res2.persisted.contains(&"MC_VSH_BASE".to_string()), "valid base missing: {:?}", res2.persisted);

        // A persistence-layer degenerate base is a data-custody condition, not permission to make
        // a scientific module accept invalid inputs. Eight finite realization snapshots prove the
        // percentile curves survive; an all-missing deterministic base must drop only BASE and
        // retain the exact explanatory note. These values are synthetic array fixtures, not
        // petrophysical endpoints or defaults.
        let snapshots: Vec<Vec<f32>> = (1..=8).map(|v| vec![v as f32]).collect();
        let snapshot_refs: Vec<&[f32]> = snapshots.iter().map(Vec::as_slice).collect();
        let degenerate = summarize_persisted_curve(
            "VSH",
            "Synthetic",
            &[1000.0],
            &snapshot_refs,
            None,
            0.10,
            0.90,
            false,
            8,
        );
        let names: Vec<&str> = degenerate.curves.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"MC_VSH_LOW"), "percentile curves must survive: {names:?}");
        assert!(names.contains(&"MC_VSH_P50"), "percentile curves must survive: {names:?}");
        assert!(names.contains(&"MC_VSH_HIGH"), "percentile curves must survive: {names:?}");
        assert!(!names.contains(&"MC_VSH_BASE"), "degenerate base must skip only BASE: {names:?}");
        assert!(
            degenerate.note.as_deref().is_some_and(|n| n.contains("MC_VSH_BASE skipped")),
            "notes: {:?}",
            degenerate.note
        );

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

        // Persistence is the subject. Keep the original 25 ± 8 fixture inside an explicit bounded
        // distribution rather than silently adding the deferred SB-CUT-038 truncation policy.
        let mc = vec![McParam { param: "GR_MA".into(), dist: Distribution::Uniform { lo: 17.0, hi: 33.0 }, zone: None }];
        let mut req = base_request(&well, mc, 300, 42);
        req.persist = true;
        let res = run_monte_carlo(&dbm, &req, None);
        assert!(res.errors.is_empty(), "unexpected errors: {:?}", res.errors);
        // The chain (vsh_gr → phi_den → sw_indo) produces VSH/PHIE/SWE but no PERM.
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
    /// AUDIT-2026-08-20 finding 10. CLAUDE.md: PHIE is floored "at the curve and at EVERY pay
    /// path". The Monte Carlo pay path was the one it had never reached - a study that varies
    /// m/n/Rw over a DELIVERED vendor PHIE runs no porosity module, so nothing floored the curve.
    ///
    /// Pinned from BOTH sides, because a lazier floor passes exactly one of them:
    ///   A - a negative streak must floor, so HPV cannot be SUBTRACTED from.
    ///   B - a MISSING sample must stay missing. `f32::max` returns the other side when one is
    ///       NaN, so the bare `v.max(FLOOR)` a hurried fix reaches for turns a washout into a
    ///       real 0.001 and books its whole slab into net - a well that was never interpreted
    ///       over that interval would report pay there.
    #[test]
    fn a_monte_carlo_realization_floors_a_negative_porosity_and_still_leaves_a_missing_one_missing() {
        let n = 5usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let step = vec![1.0f32; n];
        let zone = ZoneEntry {
            zone_name: "ALL".into(),
            top_depth: 1000.0,
            bottom_depth: 1005.0,
            depth_datum: crate::schema_vocab::DepthDatum::Md,
        };
        // No cut-offs: every sample counts, so the arithmetic isolates the floor itself.
        let cut = Cutoffs { vsh_max: None, phie_min: None, swe_max: None, perm_min: None };
        let run = |phie: &[f32]| {
            zone_metrics(
                crate::paysummary::DiscretisationModel::Forward,
                &vec![0.10f32; n],
                phie,
                &vec![0.20f32; n],
                &vec![f32::NAN; n],
                &depth,
                &step,
                &zone,
                &cut,
                false,
            )
        };
        let floor = crate::modules::PHIE_FLOOR as f32;

        // A - a vendor density porosity that went slightly negative over tight rock.
        let neg = run(&vec![-0.02f32; n]);
        assert!(
            neg.hpv > 0.0 && (neg.hpv - 5.0 * floor * 0.8).abs() < 1e-6,
            "a negative PHIE must floor, not subtract from HPV; got {}",
            neg.hpv
        );
        assert!(
            (neg.avg_phie - floor).abs() < 1e-6,
            "the average must report the floored porosity, got {}",
            neg.avg_phie
        );

        // B - one washed-out sample in the middle of a good interval.
        let mut holed = vec![0.20f32; n];
        holed[2] = f32::NAN;
        let miss = run(&holed);
        assert!(
            (miss.net - 4.0).abs() < 1e-6,
            "a MISSING sample must not become a real {} and book its slab into net; got net {}",
            floor,
            miss.net
        );
    }
}
