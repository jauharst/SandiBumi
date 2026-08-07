//! Electrofacies — unsupervised k-means clustering over chosen log curves, producing a
//! discrete FACIES curve (unsupervised electrofacies).
//!
//! Runs per well through the standard module framework (whole-vector, like `log_predict`):
//! every depth where all *present* input curves exist becomes a sample in a standardized
//! feature space; k-means++ (seeded, dependency-free, best-of-N restarts) partitions those
//! samples into K clusters. Cluster labels are then reordered by the ascending mean of the
//! first supplied input curve (typically GR), so FACIES 0 is the "cleanest" class and the
//! numbering is monotone in shaliness — which gives approximate cross-well comparability
//! even though each well is clustered independently.
//!
//! Field-wide clustering (pooling samples across wells for globally consistent labels) is a
//! deliberate deferred follow-up; this per-well version is the smallest useful increment and
//! reuses the whole existing dialog/chain/mask pipeline for free.

use crate::modules::{log_in, log_out, opt, param, ModuleContext, ModuleOutputs, ModuleSpec};
use std::collections::HashMap;

/// Input curve slots, in priority order. The first slot that carries data defines the
/// cluster-ordering axis. CURVE1 is required; the rest are optional (an absent curve simply
/// drops that feature dimension).
const SLOTS: [&str; 5] = ["CURVE1", "CURVE2", "CURVE3", "CURVE4", "CURVE5"];

// ---------------------------------------------------------------------------
// SB-MLA-023 — ONE k-means definition, whichever engine runs it
// ---------------------------------------------------------------------------
//
// SandiBumi has two k-means engines for platform reasons: this dependency-free native one, which is
// what Electrofacies and GMM Facies run per well, and scikit-learn's, which the ML suite runs when
// the user picks k-means there. They were configured differently — 8 restarts and a 100-iteration
// cap here against scikit-learn's 10 and 300 — and restart count and iteration cap are precisely
// the two knobs that decide WHICH local optimum a k-means lands in. So the same curves, the same K
// and the same seed gave two different facies schemes depending on which door the user came in, and
// nothing on either screen said so.
//
// These four constants are that definition, and `ml.rs` builds its Python runner FROM them rather
// than restating them (`ml::ml_shared_constants_py`), so there is no second copy to drift. Pinned by
// `the_two_kmeans_engines_are_configured_from_one_definition`.
//
// The values are scikit-learn's own documented defaults, adopted rather than invented. Both moves
// are in the safe direction: restarts are kept best-of-N by inertia, so 10 can only find an optimum
// at least as good as 8 ever did, and Lloyd's algorithm decreases inertia monotonically, so raising
// the cap only changes runs that had NOT converged at 100 — where the old cap was silently
// truncating rather than converging.

/// How many k-means++ restarts, keeping the lowest-inertia labelling. scikit-learn's `n_init`.
pub(crate) const KMEANS_RESTARTS: usize = 10;
/// Lloyd-iteration cap per restart. scikit-learn's `max_iter`.
pub(crate) const KMEANS_MAX_ITERS: usize = 300;
/// Convergence tolerance on the centre shift, scaled by the mean feature variance — scikit-learn's
/// `tol` and its scaling rule. Without this the native engine ran to exact label stability, which is
/// a different stopping rule from the Python one and the third divergence the two engines had.
pub(crate) const KMEANS_TOL: f64 = 1e-4;

/// SB-MLA-024 — the one seed default. This module used to fall back to **7** while the ML suite
/// used **42**; neither is wrong, and two values for one concept is the defect. No vendor in the
/// corpus ships a seed control at all, so there is no external value to defer to — 42 wins because
/// it is already the number in `ml.rs`, in the ML dialog and in the leaderboard header.
///
/// This CHANGES the default clustering Electrofacies and GMM Facies produce. A run made before
/// this recorded its seed, so it is still reproducible by typing 7 back in; what moves is the
/// result of pressing Run without touching the field.
pub(crate) const SEED_DEFAULT: f64 = 42.0;

pub fn electrofacies_spec() -> ModuleSpec {
    ModuleSpec {
        name: "electrofacies".into(),
        title: "Electrofacies (K-means)".into(),
        category: "Facies".into(),
        doc: "Unsupervised electrofacies: k-means clusters the samples of THIS well in the \
              space of the supplied curves (each feature z-scored by default, so mixed units \
              are comparable) into K facies. Any curve slot with no data is dropped; a sample \
              missing any present curve gets FACIES = MISSING. Cluster labels are ordered by \
              the mean of the first supplied curve (usually GR), so FACIES 0 is the cleanest \
              class and the numbering is monotone in shaliness. Clustering is per well and \
              deterministic for a given seed. Output: FACIES (integer 0..K-1)."
            .into(),
        args: vec![
            param("K", "Number of facies (clusters)", "", 5.0, 2.0, 12.0),
            param("SEED", "Random seed (reproducibility)", "", SEED_DEFAULT, 0.0, 1e9),
            opt("OPT_STANDARDIZE", "Feature scaling", "ZSCORE", &["ZSCORE", "NONE"]),
            log_in("CURVE1", "Curve 1 (also orders the facies)", "", "GR", true),
            log_in("CURVE2", "Curve 2 (optional)", "", "RHOB", false),
            log_in("CURVE3", "Curve 3 (optional)", "", "NPHI", false),
            log_in("CURVE4", "Curve 4 (optional)", "", "DT", false),
            log_in("CURVE5", "Curve 5 (optional)", "", "SP", false),
            log_out("FACIES", "Electrofacies cluster index (0..K-1)", ""),
        ],
    }
}

/// Shared sample preparation for the clustering modules: present input slots, complete
/// samples (with source depth indices), optional per-dimension z-scoring.
struct Prep {
    present: Vec<Vec<f32>>,
    idx: Vec<usize>,
    pts: Vec<Vec<f64>>,
    dims: usize,
    k: usize,
    seed: u64,
}

/// SB-MLA-013. Refuses by name rather than returning `None`.
///
/// An all-NaN facies track is visually indistinguishable from one that was never computed, so
/// writing it as a success does not merely fail silently - it disguises the failure as an absence
/// of work. The message separates the two causes because they need different fixes: no input curve
/// carries data at all (load or map a curve), or the curves are there and too few samples survive
/// complete-case selection to support the requested cluster count (lower K, or find what is
/// NaN-ing the rows out).
fn prep_samples(ctx: &ModuleContext) -> Result<Prep, String> {
    let n = ctx.n;
    // Keep only slots that carry data, preserving priority order.
    let present: Vec<Vec<f32>> = SLOTS
        .iter()
        .map(|s| ctx.log(s))
        .filter(|v| v.iter().any(|x| !x.is_nan()))
        .collect();
    if present.is_empty() {
        return Err(format!(
            "no input curve carries any data in this well - clustering looked for {}",
            SLOTS.join(", ")
        ));
    }
    let dims = present.len();

    let k = (ctx.p("K", 0).round().max(2.0) as usize).min(12);
    let seed = {
        let s = ctx.p("SEED", 0);
        if s.is_finite() { s.max(0.0) as u64 } else { SEED_DEFAULT as u64 }
    };
    let zscore = ctx.o("OPT_STANDARDIZE") != "NONE";

    // Collect complete samples (all present curves non-NaN), keeping the source depth index.
    let mut idx: Vec<usize> = Vec::new();
    let mut pts: Vec<Vec<f64>> = Vec::new();
    for i in 0..n {
        let x: Vec<f64> = present.iter().map(|c| c[i] as f64).collect();
        if x.iter().any(|v| v.is_nan()) {
            continue;
        }
        idx.push(i);
        pts.push(x);
    }
    if pts.len() < k {
        return Err(format!(
            "{} sample(s) carry all {} input curve(s), fewer than the {k} clusters requested - lower K, or find what is masked or missing over this interval",
            pts.len(),
            dims
        ));
    }

    // Standardize each dimension (z-score) so no single curve's raw magnitude dominates.
    if zscore {
        let m = pts.len() as f64;
        let mut mean = vec![0.0; dims];
        for x in &pts {
            for d in 0..dims {
                mean[d] += x[d];
            }
        }
        for mv in &mut mean {
            *mv /= m;
        }
        let mut std = vec![1.0; dims];
        let mut var = vec![0.0; dims];
        for x in &pts {
            for d in 0..dims {
                var[d] += (x[d] - mean[d]).powi(2);
            }
        }
        for d in 0..dims {
            std[d] = (var[d] / m).sqrt();
            if std[d] < 1e-9 {
                std[d] = 1.0;
            }
        }
        for x in &mut pts {
            for d in 0..dims {
                x[d] = (x[d] - mean[d]) / std[d];
            }
        }
    }

    Ok(Prep { present, idx, pts, dims, k, seed })
}

pub fn electrofacies(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let n = ctx.n;
    let mut out = vec![f32::NAN; n];
    let Prep { present, idx, pts, dims, k, seed } = prep_samples(ctx)?;

    // Best-of-N k-means++ restarts, keeping the lowest-inertia labelling.
    let mut best_labels: Vec<usize> = Vec::new();
    let mut best_inertia = f64::INFINITY;
    for r in 0..KMEANS_RESTARTS {
        let mut rng = Rng::new(seed ^ (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1));
        let (labels, inertia) = kmeans_once(&pts, k, dims, &mut rng);
        if inertia < best_inertia {
            best_inertia = inertia;
            best_labels = labels;
        }
    }

    // Reorder cluster labels by ascending mean of the first (priority) curve, so the
    // numbering is interpretable and roughly consistent between wells.
    let order = order_by_first_curve(&best_labels, &idx, &present[0], k);
    for (s, &i) in idx.iter().enumerate() {
        out[i] = order[best_labels[s]] as f32;
    }

    Ok(HashMap::from([("FACIES".to_string(), out)]))
}

pub fn gmm_facies_spec() -> ModuleSpec {
    ModuleSpec {
        name: "gmm_facies".into(),
        title: "Electrofacies (GMM, soft)".into(),
        category: "Facies".into(),
        doc: "Soft electrofacies: a Gaussian mixture model (diagonal covariance, EM, \
              initialized from k-means) clusters this well's samples in the space of the \
              supplied curves. Unlike k-means, every sample gets a membership PROBABILITY \
              per facies — FPROB is the winning facies' posterior (1.0 = unambiguous, \
              ~1/K = boundary/mixed sample), so transitional beds are visible instead of \
              being forced into a class. Labels are ordered by the mean of the first \
              supplied curve (usually GR). Deterministic for a given seed. Outputs: \
              FACIES_GMM (integer 0..K-1), FPROB (max posterior, 0-1)."
            .into(),
        args: vec![
            param("K", "Number of facies (mixture components)", "", 5.0, 2.0, 12.0),
            param("SEED", "Random seed (reproducibility)", "", SEED_DEFAULT, 0.0, 1e9),
            opt("OPT_STANDARDIZE", "Feature scaling", "ZSCORE", &["ZSCORE", "NONE"]),
            log_in("CURVE1", "Curve 1 (also orders the facies)", "", "GR", true),
            log_in("CURVE2", "Curve 2 (optional)", "", "RHOB", false),
            log_in("CURVE3", "Curve 3 (optional)", "", "NPHI", false),
            log_in("CURVE4", "Curve 4 (optional)", "", "DT", false),
            log_in("CURVE5", "Curve 5 (optional)", "", "SP", false),
            log_out("FACIES_GMM", "GMM facies index (0..K-1)", ""),
            log_out("FPROB", "Posterior probability of the winning facies", "v/v"),
        ],
    }
}

pub fn gmm_facies(ctx: &ModuleContext) -> Result<ModuleOutputs, String> {
    let n = ctx.n;
    let mut out = vec![f32::NAN; n];
    let mut prob = vec![f32::NAN; n];
    let Prep { present, idx, pts, dims, k, seed } = prep_samples(ctx)?;
    let m = pts.len();

    // Initialize from the best k-means run (same restarts as electrofacies, so the two
    // modules agree on well-separated data and only diverge where GMM's soft view matters).
    let mut init_labels: Vec<usize> = Vec::new();
    let mut best_inertia = f64::INFINITY;
    for r in 0..KMEANS_RESTARTS {
        let mut rng = Rng::new(seed ^ (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1));
        let (labels, inertia) = kmeans_once(&pts, k, dims, &mut rng);
        if inertia < best_inertia {
            best_inertia = inertia;
            init_labels = labels;
        }
    }

    // Component parameters from the k-means partition: weight, mean, diagonal variance.
    const VAR_FLOOR: f64 = 1e-4;
    let mut weight = vec![0.0f64; k];
    let mut mu = vec![vec![0.0f64; dims]; k];
    let mut var = vec![vec![0.0f64; dims]; k];
    {
        let mut counts = vec![0usize; k];
        for (i, p) in pts.iter().enumerate() {
            let c = init_labels[i];
            counts[c] += 1;
            for d in 0..dims {
                mu[c][d] += p[d];
            }
        }
        for c in 0..k {
            let cnt = counts[c].max(1) as f64;
            for d in 0..dims {
                mu[c][d] /= cnt;
            }
            weight[c] = counts[c] as f64 / m as f64;
        }
        for (i, p) in pts.iter().enumerate() {
            let c = init_labels[i];
            for d in 0..dims {
                var[c][d] += (p[d] - mu[c][d]).powi(2);
            }
        }
        for c in 0..k {
            let cnt = counts[c].max(1) as f64;
            for d in 0..dims {
                var[c][d] = (var[c][d] / cnt).max(VAR_FLOOR);
            }
        }
    }

    // EM until the mean log-likelihood stops improving. `resp[i]` = posterior over components.
    let ln2pi = (2.0 * std::f64::consts::PI).ln();
    let mut resp = vec![vec![0.0f64; k]; m];
    let mut prev_ll = f64::NEG_INFINITY;
    for _ in 0..KMEANS_MAX_ITERS {
        // E-step (log-space with log-sum-exp for stability).
        let mut ll = 0.0;
        for (i, p) in pts.iter().enumerate() {
            let mut logp = vec![0.0f64; k];
            for c in 0..k {
                let mut lp = weight[c].max(1e-12).ln();
                for d in 0..dims {
                    lp += -0.5 * (ln2pi + var[c][d].ln()) - (p[d] - mu[c][d]).powi(2) / (2.0 * var[c][d]);
                }
                logp[c] = lp;
            }
            let mx = logp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let sum: f64 = logp.iter().map(|&v| (v - mx).exp()).sum();
            ll += mx + sum.ln();
            for c in 0..k {
                resp[i][c] = (logp[c] - mx).exp() / sum;
            }
        }
        // M-step.
        for c in 0..k {
            let nk: f64 = resp.iter().map(|r| r[c]).sum();
            weight[c] = (nk / m as f64).max(1e-12);
            if nk < 1e-8 {
                continue; // degenerate component — keep its parameters frozen
            }
            for d in 0..dims {
                let mean_d: f64 = pts.iter().enumerate().map(|(i, p)| resp[i][c] * p[d]).sum::<f64>() / nk;
                mu[c][d] = mean_d;
            }
            for d in 0..dims {
                let v: f64 =
                    pts.iter().enumerate().map(|(i, p)| resp[i][c] * (p[d] - mu[c][d]).powi(2)).sum::<f64>() / nk;
                var[c][d] = v.max(VAR_FLOOR);
            }
        }
        if (ll - prev_ll).abs() < 1e-6 * m as f64 {
            break;
        }
        prev_ll = ll;
    }

    // Hard label = argmax posterior; confidence = that posterior. Reorder by first curve.
    let labels: Vec<usize> = resp
        .iter()
        .map(|r| {
            let mut best = 0;
            for c in 1..k {
                if r[c] > r[best] {
                    best = c;
                }
            }
            best
        })
        .collect();
    let order = order_by_first_curve(&labels, &idx, &present[0], k);
    for (s, &i) in idx.iter().enumerate() {
        out[i] = order[labels[s]] as f32;
        prob[i] = resp[s][labels[s]] as f32;
    }

    Ok(HashMap::from([("FACIES_GMM".to_string(), out), ("FPROB".to_string(), prob)]))
}

/// One k-means run: k-means++ seeding + Lloyd iterations. Returns (labels, inertia).
///
/// Stops on [`KMEANS_TOL`], scaled by the mean feature variance — scikit-learn's rule, so the two
/// engines stop at the same place and not merely after the same number of iterations. The
/// no-label-changed break stays as a fast path; it can only fire where the centres did not move at
/// all, which is a strict subset of what the tolerance already catches.
fn kmeans_once(pts: &[Vec<f64>], k: usize, dims: usize, rng: &mut Rng) -> (Vec<usize>, f64) {
    let m = pts.len();
    // The tolerance is relative because it is compared against a distance in FEATURE space, and an
    // absolute 1e-4 would mean "converged" on z-scored curves and "keep going" on raw resistivity.
    let tol = {
        let mut mean_var = 0.0;
        for d in 0..dims {
            let mu = pts.iter().map(|p| p[d]).sum::<f64>() / m as f64;
            mean_var += pts.iter().map(|p| (p[d] - mu).powi(2)).sum::<f64>() / m as f64;
        }
        KMEANS_TOL * (mean_var / dims.max(1) as f64)
    };
    // --- k-means++ seeding ---
    let mut centers: Vec<Vec<f64>> = Vec::with_capacity(k);
    centers.push(pts[(rng.unit() * m as f64) as usize % m].clone());
    let mut d2: Vec<f64> = pts.iter().map(|p| dist2(p, &centers[0])).collect();
    while centers.len() < k {
        let total: f64 = d2.iter().sum();
        let mut chosen = m - 1;
        if total > 0.0 {
            let target = rng.unit() * total;
            let mut acc = 0.0;
            for (i, &w) in d2.iter().enumerate() {
                acc += w;
                if acc >= target {
                    chosen = i;
                    break;
                }
            }
        } else {
            chosen = (rng.unit() * m as f64) as usize % m;
        }
        centers.push(pts[chosen].clone());
        for (i, p) in pts.iter().enumerate() {
            let nd = dist2(p, centers.last().unwrap());
            if nd < d2[i] {
                d2[i] = nd;
            }
        }
    }

    // --- Lloyd iterations ---
    let mut labels = vec![0usize; m];
    for _ in 0..KMEANS_MAX_ITERS {
        let mut changed = false;
        for (i, p) in pts.iter().enumerate() {
            let mut best = 0;
            let mut bestd = f64::INFINITY;
            for (c, ctr) in centers.iter().enumerate() {
                let dd = dist2(p, ctr);
                if dd < bestd {
                    bestd = dd;
                    best = c;
                }
            }
            if labels[i] != best {
                labels[i] = best;
                changed = true;
            }
        }
        // Recompute centers; reseed any empty cluster to the point farthest from its center.
        let mut sums = vec![vec![0.0; dims]; k];
        let mut counts = vec![0usize; k];
        for (i, p) in pts.iter().enumerate() {
            let c = labels[i];
            counts[c] += 1;
            for d in 0..dims {
                sums[c][d] += p[d];
            }
        }
        let mut shift = 0.0f64;
        for c in 0..k {
            if counts[c] > 0 {
                for d in 0..dims {
                    let moved = sums[c][d] / counts[c] as f64;
                    shift += (moved - centers[c][d]).powi(2);
                    centers[c][d] = moved;
                }
            } else {
                // Empty cluster: grab the globally worst-fit point.
                let mut worst = 0;
                let mut worstd = -1.0;
                for (i, p) in pts.iter().enumerate() {
                    let dd = dist2(p, &centers[labels[i]]);
                    if dd > worstd {
                        worstd = dd;
                        worst = i;
                    }
                }
                centers[c] = pts[worst].clone();
                labels[worst] = c;
                changed = true;
                // A reseeded centre is a jump, not a step. Folding it into the shift would let one
                // empty cluster's relocation read as "not converged" for another iteration, or —
                // worse on a tiny distance scale — mask a real move. Force another pass instead.
                shift = f64::INFINITY;
            }
        }
        if !changed || shift <= tol {
            break;
        }
    }

    let inertia: f64 = pts.iter().enumerate().map(|(i, p)| dist2(p, &centers[labels[i]])).sum();
    (labels, inertia)
}

/// Maps original cluster id -> rank (0 = lowest mean of the ordering curve).
fn order_by_first_curve(labels: &[usize], idx: &[usize], first: &[f32], k: usize) -> Vec<usize> {
    let mut sum = vec![0.0f64; k];
    let mut cnt = vec![0usize; k];
    for (s, &i) in idx.iter().enumerate() {
        let v = first[i] as f64;
        if v.is_finite() {
            sum[labels[s]] += v;
            cnt[labels[s]] += 1;
        }
    }
    let means: Vec<f64> = (0..k)
        .map(|c| if cnt[c] > 0 { sum[c] / cnt[c] as f64 } else { f64::INFINITY })
        .collect();
    let mut ids: Vec<usize> = (0..k).collect();
    ids.sort_by(|&a, &b| means[a].partial_cmp(&means[b]).unwrap_or(std::cmp::Ordering::Equal));
    // ids[rank] = original cluster; invert to original -> rank.
    let mut rank = vec![0usize; k];
    for (r, &c) in ids.iter().enumerate() {
        rank[c] = r;
    }
    rank
}

fn dist2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

// SplitMix64 — dependency-free, seedable (same generator used in montecarlo.rs).
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
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(logs: HashMap<String, Vec<f32>>, k: f64, n: usize) -> ModuleContext {
        let mut params = HashMap::new();
        params.insert("K".to_string(), vec![k; n]);
        params.insert("SEED".to_string(), vec![7.0; n]);
        ModuleContext { n, logs, params, opts: HashMap::new(), depth_unit: Default::default() }
    }

    #[test]
    fn separates_two_obvious_clusters() {
        // Two well-separated blobs in (GR, RHOB): clean sand vs shale.
        let mut gr = Vec::new();
        let mut rhob = Vec::new();
        for _ in 0..50 {
            gr.push(20.0);
            rhob.push(2.65);
        }
        for _ in 0..50 {
            gr.push(120.0);
            rhob.push(2.45);
        }
        let logs = HashMap::from([
            ("CURVE1".to_string(), gr),
            ("CURVE2".to_string(), rhob),
        ]);
        let out = electrofacies(&ctx(logs, 2.0, 100)).expect("clustering should succeed")["FACIES"].clone();
        // First 50 (low GR) should all be facies 0, next 50 facies 1 (ordering by GR mean).
        assert!(out[..50].iter().all(|&v| v == 0.0), "clean sand -> facies 0");
        assert!(out[50..].iter().all(|&v| v == 1.0), "shale -> facies 1");
    }

    #[test]
    fn missing_inputs_yield_missing_facies() {
        let gr = vec![20.0, f32::NAN, 30.0, 120.0, 110.0, 25.0];
        let logs = HashMap::from([("CURVE1".to_string(), gr)]);
        let out = electrofacies(&ctx(logs, 2.0, 6)).expect("clustering should succeed")["FACIES"].clone();
        assert!(out[1].is_nan(), "sample with missing input stays MISSING");
        assert!(out.iter().filter(|v| !v.is_nan()).count() == 5);
    }

    /// SB-MLA-T13 case (a). A well where no input curve carries a single reading cannot be
    /// clustered, and both engines must say so BY NAME. The failure mode this pins is not a
    /// crash but a courtesy: returning the pre-allocated all-NaN vector as `Ok`. On a log view
    /// an all-missing FACIES track looks exactly like one that was never computed, so the run
    /// reports success while the user sees an empty track and has no way to tell which it is.
    #[test]
    fn a_well_with_no_input_data_is_refused_by_name_not_returned_as_a_clean_curve() {
        let logs = HashMap::from([("CURVE1".to_string(), vec![f32::NAN; 20])]);
        for (engine, out) in [
            ("electrofacies", electrofacies(&ctx(logs.clone(), 3.0, 20))),
            ("gmm_facies", gmm_facies(&ctx(logs.clone(), 3.0, 20))),
        ] {
            let msg = out.expect_err(&format!("{engine} must refuse a well with no input data"));
            assert!(
                msg.contains("no input curve carries any data"),
                "{engine} must name the cause, got {msg:?}",
            );
        }
    }

    /// SB-MLA-T13 case (b), pinned from BOTH sides: 4 complete samples with `K = 5` is refused
    /// and the message states both numbers, while 5 complete samples with `K = 5` succeeds. The
    /// second half is what stops the refusal being satisfied by an engine that has simply become
    /// timid — refusing every sparse well would pass the first assertion on its own.
    #[test]
    fn fewer_complete_samples_than_clusters_is_refused_naming_the_count_and_k() {
        let short = vec![10.0, 40.0, 70.0, 100.0];
        let logs = HashMap::from([("CURVE1".to_string(), short)]);
        for (engine, out) in [
            ("electrofacies", electrofacies(&ctx(logs.clone(), 5.0, 4))),
            ("gmm_facies", gmm_facies(&ctx(logs.clone(), 5.0, 4))),
        ] {
            let msg = out.expect_err(&format!("{engine} must refuse 4 samples with K=5"));
            assert!(
                msg.contains('4') && msg.contains('5'),
                "{engine} must state the sample count and the requested cluster count, got {msg:?}",
            );
        }

        // The other side: one more sample and the same request is answerable, so it is answered.
        let enough = vec![10.0, 40.0, 70.0, 100.0, 130.0];
        let logs = HashMap::from([("CURVE1".to_string(), enough)]);
        for (engine, key, out) in [
            ("electrofacies", "FACIES", electrofacies(&ctx(logs.clone(), 5.0, 5))),
            ("gmm_facies", "FACIES_GMM", gmm_facies(&ctx(logs.clone(), 5.0, 5))),
        ] {
            let outs = out.unwrap_or_else(|e| panic!("{engine} must cluster 5 samples into 5: {e}"));
            assert!(
                outs[key].iter().any(|v| !v.is_nan()),
                "{engine} returned a labelling, so it must not be all-missing",
            );
        }
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let gr: Vec<f32> = (0..200).map(|i| (i as f32 * 1.7) % 140.0).collect();
        let logs = HashMap::from([("CURVE1".to_string(), gr)]);
        let a = electrofacies(&ctx(logs.clone(), 4.0, 200)).expect("clustering should succeed")["FACIES"].clone();
        let b = electrofacies(&ctx(logs, 4.0, 200)).expect("clustering should succeed")["FACIES"].clone();
        assert_eq!(a, b, "same seed -> identical labels");
    }

    #[test]
    fn gmm_agrees_with_kmeans_on_separated_blobs_and_is_confident() {
        let mut gr = Vec::new();
        let mut rhob = Vec::new();
        for _ in 0..50 {
            gr.push(20.0);
            rhob.push(2.65);
        }
        for _ in 0..50 {
            gr.push(120.0);
            rhob.push(2.45);
        }
        let logs = HashMap::from([("CURVE1".to_string(), gr), ("CURVE2".to_string(), rhob)]);
        let res = gmm_facies(&ctx(logs, 2.0, 100)).expect("clustering should succeed");
        let fac = &res["FACIES_GMM"];
        let prob = &res["FPROB"];
        assert!(fac[..50].iter().all(|&v| v == 0.0), "clean sand -> facies 0");
        assert!(fac[50..].iter().all(|&v| v == 1.0), "shale -> facies 1");
        // Well-separated blobs: the winning posterior should be essentially certain.
        assert!(prob.iter().all(|&p| p > 0.99), "separated clusters -> confident posteriors");
    }

    #[test]
    fn gmm_boundary_samples_get_lower_confidence() {
        // A continuous GR ramp split into K=2 components: samples at the decision
        // boundary between the two fitted Gaussians must be genuinely ambiguous
        // (posterior near 0.5) while the ramp's extremes are near-certain.
        let gr: Vec<f32> = (0..=200).map(|i| i as f32 * 0.5).collect(); // 0..100
        let n = gr.len();
        let logs = HashMap::from([("CURVE1".to_string(), gr)]);
        let res = gmm_facies(&ctx(logs, 2.0, n)).expect("clustering should succeed");
        let prob = &res["FPROB"];
        let min_prob = prob.iter().cloned().fold(1.0f32, f32::min);
        assert!(min_prob < 0.75, "boundary of a ramp is ambiguous, got min {}", min_prob);
        assert!(prob[0] > 0.99 && prob[n - 1] > 0.99, "ramp extremes are near-certain");
        // The least-confident sample sits at the label transition.
        let fac = &res["FACIES_GMM"];
        let argmin = (0..n).min_by(|&a, &b| prob[a].partial_cmp(&prob[b]).unwrap()).unwrap();
        assert!(
            fac[argmin.saturating_sub(2)] != fac[(argmin + 2).min(n - 1)],
            "min-confidence sample lies at the facies boundary"
        );
    }

    #[test]
    fn gmm_deterministic_and_missing_propagates() {
        let gr: Vec<f32> =
            (0..200).map(|i| if i == 13 { f32::NAN } else { (i as f32 * 1.7) % 140.0 }).collect();
        let logs = HashMap::from([("CURVE1".to_string(), gr)]);
        let a = gmm_facies(&ctx(logs.clone(), 3.0, 200)).expect("clustering should succeed");
        let b = gmm_facies(&ctx(logs, 3.0, 200)).expect("clustering should succeed");
        // NaN != NaN, so compare element-wise treating NaN==NaN as equal.
        let same = a["FACIES_GMM"]
            .iter()
            .zip(b["FACIES_GMM"].iter())
            .all(|(&x, &y)| (x.is_nan() && y.is_nan()) || x == y);
        assert!(same, "same seed -> identical labels");
        assert!(a["FACIES_GMM"][13].is_nan() && a["FPROB"][13].is_nan());
    }

    #[test]
    fn labels_are_ordered_by_first_curve() {
        // Three GR levels; facies index must increase with GR.
        let mut gr = Vec::new();
        for lvl in [15.0f32, 70.0, 130.0] {
            for _ in 0..40 {
                gr.push(lvl);
            }
        }
        let logs = HashMap::from([("CURVE1".to_string(), gr)]);
        let out = electrofacies(&ctx(logs, 3.0, 120)).expect("clustering should succeed")["FACIES"].clone();
        assert!(out[..40].iter().all(|&v| v == 0.0));
        assert!(out[40..80].iter().all(|&v| v == 1.0));
        assert!(out[80..].iter().all(|&v| v == 2.0));
    }
}
