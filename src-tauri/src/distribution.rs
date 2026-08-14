//! Distribution summaries for depth-binned displays — box plots, whiskers, histograms.
//!
//! Deliberately **source-agnostic**: everything here takes a bare slice of values and knows
//! nothing about where they came from. That is the whole point. Core plugs gathered into a
//! depth interval, XRD sample counts, and the N realizations of a Monte Carlo array log at a
//! single depth are the SAME statistical operation once the values are in hand — they differ
//! only in how they were gathered. Both the point-data track and (later) array logs feed
//! this module; neither owns it.
//!
//! Mirrored in `src/distribution.ts` for the WebGPU log view, which cannot call Rust per
//! frame. **Keep the two in agreement** — same percentile definition, same whisker rules —
//! the way `FACIES_PALETTE` is kept in sync between `plotCanvas.ts` and `composite.rs`.

use serde::{Deserialize, Serialize};

/// How the whiskers are defined. This is a real interpretive choice, not a cosmetic one:
/// Tukey answers "which plugs are unusual for this interval", a percentile pair answers
/// "where do 80% of the plugs lie" — different questions, different pictures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Whisker {
    /// The most extreme sample still within `k` × IQR of the box. Anything past it is an
    /// outlier and is drawn individually. k = 1.5 is the convention.
    Tukey(f32),
    /// A percentile pair (e.g. 10/90). The whiskers ARE those percentiles; no outliers,
    /// because nothing is "unusual" under this definition — it is a coverage statement.
    Percentile(f32, f32),
    /// The full observed range. No outliers.
    MinMax,
}

/// One depth bin's summary. All values are in the curve's own units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxStats {
    pub n: usize,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    /// Lower box edge (default P25).
    pub lo: f32,
    /// Median (P50) — always P50 regardless of the chosen box edges.
    pub med: f32,
    /// Upper box edge (default P75).
    pub hi: f32,
    pub whisker_lo: f32,
    pub whisker_hi: f32,
    /// Samples beyond the whiskers, drawn individually. Empty for non-Tukey rules.
    pub outliers: Vec<f32>,
}

/// What a Ward partition was run OVER — the thing that makes one application different from
/// another, and the only part of `SB-MLA-025` that is not shared code.
///
/// The criterion is one criterion. What differs between its uses is the ORDER the values were put
/// in before it ran, and that changes the geological question completely: an optimal split of
/// FZI sorted by value answers "how many rock types are in this core", while the same arithmetic
/// over the depth-ordered profile answers "where are the flow-unit boundaries in this well". A
/// user must be able to tell which they ran, so the caller DECLARES it and it travels with the
/// result rather than being inferred from which module happened to call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WardOrder {
    /// Values sorted ascending; clusters are intervals of the VALUE. `hfu.rs`.
    SortedValue,
    /// Values left in depth order; clusters are intervals of DEPTH. `lorenz.rs`.
    Depth,
    /// No ordering constraint — any point may join any cluster. Agglomerative Ward in the ML pane
    /// (`ml.rs`), which is scikit-learn's implementation and not this DP. Listed here because this
    /// enum is the one place the three applications are named, and a variant missing from the list
    /// is how a fourth one gets added without anybody noticing there were already three.
    Free,
}

impl WardOrder {
    /// The name that goes in provenance. Prefixed so the three read as variants of one criterion
    /// rather than as three unrelated methods.
    pub fn name(self) -> &'static str {
        match self {
            WardOrder::SortedValue => "ward:sorted-value",
            WardOrder::Depth => "ward:depth-contiguous",
            WardOrder::Free => "ward:free",
        }
    }
}

/// The exact optimal K-partition of a sequence minimising total within-cluster sum of squares —
/// the Ward criterion — as ONE implementation (`SB-MLA-025`, `SB-CORE-006`).
///
/// This lived twice: `hfu::ward_partition` over sorted FZI and `lorenz::segment_dp` over the
/// depth-ordered slope profile. The two were byte-for-byte the same dynamic program with different
/// return shapes — one backtracked internally for a single k, the other returned the table so the
/// caller could choose k first. Two copies of one criterion is two places for it to drift, and the
/// drift would be silent: both would still produce a plausible partition.
///
/// Kept as a struct rather than a function because the two uses genuinely need different things
/// from the same table. Building it is the expensive part (O(k·m²)); reading an assignment out of
/// it afterwards is free, so a caller that wants to compare several k values pays once.
///
/// The partition is always CONTIGUOUS in the order supplied — that is what makes the exact DP
/// possible at all. Free-ordering Ward is a different algorithm (agglomerative), which is why
/// [`WardOrder::Free`] is a name here and not a code path.
pub struct WardDp {
    /// `sse_by_k[j]` = minimum total within-cluster sum of squares using j clusters. Index 0 unused.
    sse_by_k: Vec<f64>,
    /// Backtracking table: `arg[j][i]` is where the j-th cluster starts when the first i elements
    /// are split into j.
    arg: Vec<Vec<usize>>,
    m: usize,
    order: WardOrder,
}

impl WardDp {
    /// Builds the table for every k up to `kmax`. O(kmax·m²).
    pub fn new(vals: &[f64], kmax: usize, order: WardOrder) -> Self {
        let m = vals.len();
        let k = kmax.clamp(1, m.max(1));
        // Prefix sums for O(1) segment SS: cost[a,b) = Σx² − (Σx)²/(b−a).
        let mut ps = vec![0.0f64; m + 1];
        let mut ps2 = vec![0.0f64; m + 1];
        for i in 0..m {
            ps[i + 1] = ps[i] + vals[i];
            ps2[i + 1] = ps2[i] + vals[i] * vals[i];
        }
        let cost = |a: usize, b: usize| -> f64 {
            let cnt = (b - a) as f64;
            if cnt <= 0.0 {
                return 0.0;
            }
            let s = ps[b] - ps[a];
            // Clamped at zero: the identity is exact in real arithmetic, and in floating point a
            // segment of identical values can land a few ulps below it.
            (ps2[b] - ps2[a] - s * s / cnt).max(0.0)
        };
        let inf = f64::INFINITY;
        let mut dp = vec![vec![inf; m + 1]; k + 1];
        let mut arg = vec![vec![0usize; m + 1]; k + 1];
        dp[0][0] = 0.0;
        for j in 1..=k {
            for i in j..=m {
                for t in (j - 1)..i {
                    let c = dp[j - 1][t] + cost(t, i);
                    if c < dp[j][i] {
                        dp[j][i] = c;
                        arg[j][i] = t;
                    }
                }
            }
        }
        let sse_by_k = (0..=k).map(|j| dp[j][m]).collect();
        WardDp { sse_by_k, arg, m, order }
    }

    /// Minimum total within-cluster sum of squares for each k. Index 0 is unused; index 1 is the
    /// one-cluster total, which is what an elbow rule normalises against.
    pub fn sse_by_k(&self) -> &[f64] {
        &self.sse_by_k
    }

    /// The 0-based cluster id of each element, in the order supplied. Clusters are numbered in that
    /// same order, so with [`WardOrder::SortedValue`] the ids ascend with the value and with
    /// [`WardOrder::Depth`] they ascend with depth.
    pub fn assign(&self, k: usize) -> Vec<usize> {
        let mut assign = vec![0usize; self.m];
        let k = k.clamp(1, self.sse_by_k.len().saturating_sub(1));
        if k <= 1 || self.m == 0 {
            return assign;
        }
        let mut i = self.m;
        for j in (1..=k).rev() {
            let t = self.arg[j][i];
            for a in assign.iter_mut().take(i).skip(t) {
                *a = j - 1;
            }
            i = t;
        }
        assign
    }

    /// The variant name for provenance — see [`WardOrder`].
    pub fn variant(&self) -> &'static str {
        self.order.name()
    }
}

/// Linear-interpolated percentile of an ASCENDING-sorted slice (the R type-7 / NumPy
/// `percentile` default, and what Excel's PERCENTILE returns). `p` is 0–100 and is clamped.
/// Chosen because it is what every reference a petrophysicist would check against uses — a
/// nearest-rank definition would disagree with the same numbers computed in a spreadsheet.
pub fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return f32::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let p = p.clamp(0.0, 100.0) as f64;
    let pos = (p / 100.0) * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let frac = (pos - lo as f64) as f32;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

/// Summarizes one set of values. Non-finite samples are dropped (never counted, never
/// treated as zero); `None` when nothing finite is left, so a caller draws no glyph rather
/// than an empty one at a false position.
pub fn box_stats(values: &[f32], box_lo: f32, box_hi: f32, whisker: Whisker) -> Option<BoxStats> {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).expect("non-finite values already filtered out"));
    let (box_lo, box_hi) = if box_lo <= box_hi { (box_lo, box_hi) } else { (box_hi, box_lo) };
    let lo = percentile(&v, box_lo);
    let hi = percentile(&v, box_hi);
    let med = percentile(&v, 50.0);
    let mean = v.iter().map(|x| *x as f64).sum::<f64>() as f32 / v.len() as f32;

    let (whisker_lo, whisker_hi, outliers) = match whisker {
        Whisker::MinMax => (v[0], v[v.len() - 1], Vec::new()),
        Whisker::Percentile(a, b) => {
            let (a, b) = if a <= b { (a, b) } else { (b, a) };
            (percentile(&v, a), percentile(&v, b), Vec::new())
        }
        Whisker::Tukey(k) => {
            // Fences from the BOX edges, whatever the user set those to — with the default
            // 25/75 this is textbook Tukey; with 10/90 it is the same idea on a wider box.
            let iqr = hi - lo;
            let (fence_lo, fence_hi) = (lo - k * iqr, hi + k * iqr);
            let w_lo = *v.iter().find(|x| **x >= fence_lo).unwrap_or(&v[0]);
            let w_hi = *v.iter().rev().find(|x| **x <= fence_hi).unwrap_or(&v[v.len() - 1]);
            let out: Vec<f32> = v.iter().copied().filter(|x| *x < w_lo || *x > w_hi).collect();
            (w_lo, w_hi, out)
        }
    };

    Some(BoxStats {
        n: v.len(),
        min: v[0],
        max: v[v.len() - 1],
        mean,
        lo,
        med,
        hi,
        whisker_lo,
        whisker_hi,
        outliers,
    })
}

/// Counts values into `bins` equal-width bins spanning [min, max]. Values outside the range
/// are DROPPED, not clamped into the end bins — a clamped sample would invent a count the
/// data never had at that value. Handles a reversed range (min > max), which is normal for a
/// porosity axis.
pub const HISTOGRAM_BINS_MIN: usize = 1;
pub const HISTOGRAM_BINS_MAX: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramContract {
    pub counts: Vec<u32>,
    pub displayed_total: u32,
    pub non_finite_excluded: usize,
}

pub fn canonical_histogram(
    values: &[f32],
    min: f32,
    max: f32,
    bins: usize,
) -> HistogramContract {
    let bins = bins.clamp(HISTOGRAM_BINS_MIN, HISTOGRAM_BINS_MAX);
    let mut counts = vec![0u32; bins];
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    let mut non_finite_excluded = 0;
    if hi > lo {
        for &value in values {
            if !value.is_finite() {
                non_finite_excluded += 1;
                continue;
            }
            if value < lo || value > hi {
                continue;
            }
            let index = (((value - lo) / (hi - lo)) * bins as f32) as usize;
            counts[index.min(bins - 1)] += 1;
        }
    } else {
        non_finite_excluded = values.iter().filter(|value| !value.is_finite()).count();
    }
    HistogramContract {
        displayed_total: counts.iter().sum(),
        counts,
        non_finite_excluded,
    }
}

/// Counts-only compatibility for non-reporting micro-glyphs. Arithmetic remains canonical.
pub fn histogram(values: &[f32], min: f32, max: f32, bins: usize) -> Vec<u32> {
    canonical_histogram(values, min, max, bins).counts
}

/// Groups depth-tagged samples into equal-height depth bins, returning
/// `(bin_top, bin_base, values)` for every bin that has at least one sample. Empty bins are
/// omitted entirely, so a sparse core section draws no glyphs rather than a row of blanks.
/// `bin` is in the project's depth unit; a non-positive height yields no bins.
pub fn bin_by_depth(depth: &[f32], value: &[f32], bin: f32) -> Vec<(f32, f32, Vec<f32>)> {
    if !(bin > 0.0) {
        return Vec::new();
    }
    let n = depth.len().min(value.len());
    let mut keyed: Vec<(i64, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let (d, v) = (depth[i], value[i]);
        if !d.is_finite() || !v.is_finite() {
            continue;
        }
        keyed.push(((d / bin).floor() as i64, v));
    }
    keyed.sort_by_key(|(k, _)| *k);
    let mut out: Vec<(f32, f32, Vec<f32>)> = Vec::new();
    for (k, v) in keyed {
        match out.last_mut() {
            Some((top, _, vals)) if (*top / bin).floor() as i64 == k => vals.push(v),
            _ => {
                let top = k as f32 * bin;
                out.push((top, top + bin, vec![v]));
            }
        }
    }
    out
}

/// Low / median / high percentile of ONE distribution, in a single call.
///
/// This is what an uncertainty band is made of: at each depth an array log holds a whole set
/// of values, and the band is three percentiles through it. Kept here rather than at the two
/// call sites so the interactive viewer and the print exporter cannot drift on how a band is
/// derived — the same reason `box_stats` lives here.
///
/// Non-finite values are dropped, exactly as everywhere else in this module; `None` means the
/// depth had nothing finite to summarise and should be a GAP in the band, not a zero.
pub fn band(values: &[f32], lo_p: f32, hi_p: f32) -> Option<(f32, f32, f32)> {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (lo_p, hi_p) = if lo_p <= hi_p { (lo_p, hi_p) } else { (hi_p, lo_p) };
    Some((percentile(&v, lo_p), percentile(&v, 50.0), percentile(&v, hi_p)))
}

/// Picks `want` indices spread EVENLY across `total`, for drawing a readable subset of a large
/// set of realizations.
///
/// Evenly rather than the first `want`: a Latin-hypercube design lays its draws out in
/// stratified order, so the first N of them are a biased corner of the sampled space and a
/// spaghetti plot built from them would understate the true spread. Returns every index when
/// `want >= total`, and an empty vector when either is zero.
pub fn even_indices(total: usize, want: usize) -> Vec<usize> {
    if total == 0 || want == 0 {
        return Vec::new();
    }
    if want >= total {
        return (0..total).collect();
    }
    // Mid-point sampling of `want` equal strata: symmetric, and never returns index `total`.
    (0..want).map(|k| ((k as f64 + 0.5) * total as f64 / want as f64) as usize).map(|i| i.min(total - 1)).collect()
}

#[cfg(test)]
mod ward_tests {
    use super::*;

    /// One criterion, and the ORDER is what makes an application different.
    ///
    /// `SB-MLA-025`. The same dynamic program lived twice — over FZI sorted by value in `hfu.rs`
    /// and over the depth-ordered slope profile in `lorenz.rs` — so this pins that the shared
    /// implementation still finds the known optimum, and that the two orderings genuinely disagree
    /// on the same numbers. If they agreed, merging them would have been the whole story; they do
    /// not, which is why the variant has to reach the user.
    #[test]
    fn one_ward_criterion_gives_different_answers_under_different_orderings_and_names_which() {
        // Deliberately NOT sorted: two low values sit between the highs. Read in the order given
        // (depth), the optimal 2-split is a contiguous run; read sorted by value it is not.
        let raw = [10.0, 10.2, 1.0, 1.1, 9.8, 10.1];
        let depth = WardDp::new(&raw, 3, WardOrder::Depth);
        assert_eq!(
            depth.assign(2),
            vec![0, 0, 1, 1, 1, 1],
            "in depth order the split has to be one contiguous cut, wherever the values sit",
        );

        let mut sorted = raw;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let byval = WardDp::new(&sorted, 3, WardOrder::SortedValue);
        assert_eq!(
            byval.assign(2),
            vec![0, 0, 1, 1, 1, 1],
            "sorted by value the two low samples group together however far apart in depth",
        );
        // The same two samples, opposite conclusions: in depth order sample 4 (9.8) joins the LOW
        // group because it is contiguous with it; sorted by value it joins the high group.
        assert!(
            depth.sse_by_k()[2] > byval.sse_by_k()[2],
            "the value-sorted split must fit better, or the fixture does not separate the two \
             questions: depth {:?} vs sorted {:?}",
            depth.sse_by_k()[2],
            byval.sse_by_k()[2],
        );

        // Each carries the name that goes into provenance, and the three are distinct.
        assert_eq!(depth.variant(), "ward:depth-contiguous");
        assert_eq!(byval.variant(), "ward:sorted-value");
        assert_eq!(WardOrder::Free.name(), "ward:free");
        let names = [WardOrder::SortedValue, WardOrder::Depth, WardOrder::Free].map(WardOrder::name);
        let mut uniq = names.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), 3, "three applications, three names: {names:?}");
    }

    /// The table is built once and read many times, so a k read out of it must equal what a table
    /// built for that k alone would give. Without this, the shared struct could return a partition
    /// for the wrong k and every caller would still get a plausible-looking answer.
    #[test]
    fn an_assignment_read_from_a_larger_table_matches_one_built_for_that_k_alone() {
        let vals = [1.0, 1.2, 5.0, 5.1, 5.2, 9.0, 9.4, 9.5];
        let big = WardDp::new(&vals, 5, WardOrder::SortedValue);
        for k in 1..=5 {
            let alone = WardDp::new(&vals, k, WardOrder::SortedValue);
            assert_eq!(big.assign(k), alone.assign(k), "k={k}");
            assert!(
                (big.sse_by_k()[k] - alone.sse_by_k()[k]).abs() < 1e-12,
                "k={k} SSE differs",
            );
        }
        assert_eq!(big.assign(3), vec![0, 0, 1, 1, 1, 2, 2, 2], "three obvious groups");
        // k above the sample count, and k below 1, are clamped rather than panicking.
        assert_eq!(WardDp::new(&vals, 99, WardOrder::Depth).assign(99).len(), vals.len());
        assert_eq!(WardDp::new(&vals, 3, WardOrder::Depth).assign(0), vec![0; vals.len()]);
        assert!(WardDp::new(&[], 3, WardOrder::Depth).assign(2).is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_band_is_three_percentiles_of_one_depths_realizations() {
        // 101 realizations 0.00..1.00: P10/P50/P90 land exactly on 0.10 / 0.50 / 0.90.
        let vals: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0).collect();
        let (lo, med, hi) = band(&vals, 10.0, 90.0).unwrap();
        assert!((lo - 0.10).abs() < 1e-6, "lo {lo}");
        assert!((med - 0.50).abs() < 1e-6, "med {med}");
        assert!((hi - 0.90).abs() < 1e-6, "hi {hi}");
    }

    #[test]
    fn a_band_ignores_failed_realizations_and_reports_nothing_when_all_failed() {
        let mixed = [f32::NAN, 0.2, f32::NAN, 0.4, 0.6];
        let (lo, med, hi) = band(&mixed, 0.0, 100.0).unwrap();
        assert_eq!((lo, med, hi), (0.2, 0.4, 0.6), "NaN realizations must not shift the band");
        // A depth where nothing converged is a GAP, never a zero-width band at 0.0.
        assert!(band(&[f32::NAN, f32::NAN], 10.0, 90.0).is_none());
        assert!(band(&[], 10.0, 90.0).is_none());
    }

    #[test]
    fn band_edges_given_backwards_still_come_back_low_first() {
        let vals: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0).collect();
        assert_eq!(band(&vals, 90.0, 10.0).unwrap(), band(&vals, 10.0, 90.0).unwrap());
    }

    #[test]
    fn spaghetti_traces_are_drawn_from_across_the_sample_space_not_its_first_corner() {
        // 8 of 1000: evenly spread, so the drawn subset spans the design rather than sitting in
        // the low corner an LHS design happens to start in.
        let idx = even_indices(1000, 8);
        assert_eq!(idx, vec![62, 187, 312, 437, 562, 687, 812, 937]);
        assert_eq!(even_indices(5, 5), vec![0, 1, 2, 3, 4], "asking for all gives all, in order");
        assert_eq!(even_indices(3, 10), vec![0, 1, 2], "never more than exist");
        assert!(even_indices(0, 4).is_empty() && even_indices(4, 0).is_empty());
        // Never returns an out-of-bounds index, at any ratio.
        for total in 1..64usize {
            for want in 1..64usize {
                assert!(even_indices(total, want).iter().all(|i| *i < total), "{total}/{want}");
            }
        }
    }

    #[test]
    fn percentile_matches_the_spreadsheet_definition() {
        let v = [1.0f32, 2.0, 3.0, 4.0];
        // Excel PERCENTILE / numpy.percentile on 1,2,3,4: P25 = 1.75, P50 = 2.5, P75 = 3.25.
        assert_eq!(percentile(&v, 25.0), 1.75);
        assert_eq!(percentile(&v, 50.0), 2.5);
        assert_eq!(percentile(&v, 75.0), 3.25);
        assert_eq!(percentile(&v, 0.0), 1.0);
        assert_eq!(percentile(&v, 100.0), 4.0);
    }

    #[test]
    fn a_single_plug_summarizes_to_itself_rather_than_failing() {
        // One plug in a depth bin is common in a sparse cored interval. It must produce a
        // degenerate box at its own value, not a NaN glyph at a false position.
        let s = box_stats(&[0.23], 25.0, 75.0, Whisker::Tukey(1.5)).unwrap();
        assert_eq!((s.n, s.lo, s.med, s.hi), (1, 0.23, 0.23, 0.23));
        assert!(s.outliers.is_empty());
    }

    #[test]
    fn non_finite_samples_are_dropped_not_counted() {
        let s = box_stats(&[1.0, f32::NAN, 3.0, f32::INFINITY], 25.0, 75.0, Whisker::MinMax).unwrap();
        assert_eq!(s.n, 2);
        assert_eq!((s.min, s.max), (1.0, 3.0));
        assert!(box_stats(&[f32::NAN, f32::NAN], 25.0, 75.0, Whisker::MinMax).is_none());
    }

    #[test]
    fn tukey_whiskers_stop_at_real_samples_and_flag_the_rest() {
        // 1..9 plus a far outlier. IQR fences must exclude 100 and the whisker must land on
        // an ACTUAL sample (9), not on the fence value itself.
        let v = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let s = box_stats(&v, 25.0, 75.0, Whisker::Tukey(1.5)).unwrap();
        assert_eq!(s.whisker_lo, 1.0);
        assert_eq!(s.whisker_hi, 9.0);
        assert_eq!(s.outliers, vec![100.0]);
    }

    #[test]
    fn a_percentile_whisker_rule_reports_no_outliers() {
        let v = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let s = box_stats(&v, 25.0, 75.0, Whisker::Percentile(10.0, 90.0)).unwrap();
        assert!(s.outliers.is_empty(), "a coverage rule declares nothing unusual");
        assert_eq!(s.whisker_lo, percentile(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0], 10.0));
    }

    #[test]
    fn histogram_drops_out_of_range_instead_of_clamping() {
        let counts = histogram(&[-1.0, 0.0, 0.5, 1.0, 2.0], 0.0, 1.0, 2);
        // 0.0 and 0.5 land in the two bins, 1.0 lands in the last; -1 and 2 are dropped.
        assert_eq!(counts, vec![1, 2]);
        assert_eq!(counts.iter().sum::<u32>(), 3);
    }

    #[test]
    fn histogram_accepts_a_reversed_axis() {
        // Porosity tracks routinely run 0.5 → 0.0; the counts must not silently come out empty.
        assert_eq!(histogram(&[0.1, 0.4], 0.5, 0.0, 5).iter().sum::<u32>(), 2);
    }

    #[test]
    fn depth_bins_skip_empty_intervals_and_keep_their_samples() {
        // Plugs at 1000.1/1000.4 and 1010.2 with a 5 m bin: two bins, and the 5 m of barren
        // hole between them contributes no bin at all.
        let d = [1000.1f32, 1000.4, 1010.2];
        let v = [0.2f32, 0.24, 0.31];
        let bins = bin_by_depth(&d, &v, 5.0);
        assert_eq!(bins.len(), 2);
        assert_eq!(bins[0].2, vec![0.2, 0.24]);
        assert_eq!(bins[1].2, vec![0.31]);
        assert_eq!((bins[0].0, bins[0].1), (1000.0, 1005.0));
    }

    #[test]
    fn a_monte_carlo_realization_set_summarizes_through_the_same_call() {
        // The reason this module takes a bare slice: 1000 PHIE realizations at ONE depth are
        // the same operation as plugs gathered over an interval. If this ever needs its own
        // code path, the abstraction has been broken.
        let realizations: Vec<f32> = (0..1000).map(|i| 0.20 + (i as f32) * 0.0001).collect();
        let s = box_stats(&realizations, 10.0, 90.0, Whisker::Percentile(10.0, 90.0)).unwrap();
        assert_eq!(s.n, 1000);
        assert!((s.med - 0.24995).abs() < 1e-4);
        assert!(s.lo < s.med && s.med < s.hi);
    }
}
