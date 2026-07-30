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
pub fn histogram(values: &[f32], min: f32, max: f32, bins: usize) -> Vec<u32> {
    let bins = bins.max(1);
    let mut out = vec![0u32; bins];
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if !(hi > lo) {
        return out;
    }
    for &x in values {
        if !x.is_finite() || x < lo || x > hi {
            continue;
        }
        let idx = (((x - lo) / (hi - lo)) * bins as f32) as usize;
        out[idx.min(bins - 1)] += 1;
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

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
