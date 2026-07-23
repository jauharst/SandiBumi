//! Petrel-style tops tooling: stratigraphic-order (crossing) checks and log-based
//! marker autocorrelation across wells.

use crate::db;
use crate::equations::fetch_curve_frame;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compares this well's top ordering (by depth) against every other well that shares
/// the same top names. A pair that is stratigraphically reversed relative to the
/// majority of other wells produces a human-readable warning — the field-standard
/// "your marker crosses" check done at pick time.
pub fn check_top_order(conn: &Connection, well_id: &str) -> Result<Vec<String>, String> {
    let this_tops = db::list_tops(conn, well_id).map_err(|e| e.to_string())?;
    if this_tops.len() < 2 {
        return Ok(Vec::new());
    }

    // name(upper) -> depth for every OTHER well.
    let mut others: Vec<HashMap<String, f32>> = Vec::new();
    for well in db::list_wells(conn).map_err(|e| e.to_string())? {
        if well.well_id == well_id {
            continue;
        }
        let tops = db::list_tops(conn, &well.well_id).map_err(|e| e.to_string())?;
        if tops.len() >= 2 {
            others.push(tops.into_iter().map(|t| (t.top_name.to_uppercase(), t.depth)).collect());
        }
    }
    if others.is_empty() {
        return Ok(Vec::new());
    }

    let mut warnings = Vec::new();
    for i in 0..this_tops.len() {
        for j in (i + 1)..this_tops.len() {
            let (upper, lower) = (&this_tops[i], &this_tops[j]); // list_tops orders by depth
            let (mut same, mut reversed) = (0u32, 0u32);
            for other in &others {
                if let (Some(&du), Some(&dl)) =
                    (other.get(&upper.top_name.to_uppercase()), other.get(&lower.top_name.to_uppercase()))
                {
                    if du <= dl {
                        same += 1;
                    } else {
                        reversed += 1;
                    }
                }
            }
            if reversed > same {
                warnings.push(format!(
                    "'{}' is above '{}' here, but below it in {} of {} other wells",
                    upper.top_name,
                    lower.top_name,
                    reversed,
                    same + reversed
                ));
            }
        }
    }
    Ok(warnings)
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutoCorrRequest {
    pub source_well_id: String,
    pub top_name: String,
    /// Log used for pattern matching (GR is the field standard).
    pub curve: String,
    /// Half-length (depth units) of the correlation window around the marker.
    pub half_window: f32,
    /// How far above/below the initial guess to search in each target well.
    pub search_range: f32,
    pub target_well_ids: Vec<String>,
    /// "shift" (rigid best-lag, the fast default) or "warp" (elastic depth-warp
    /// refinement of the rigid pick). Absent/unknown ⇒ "shift".
    #[serde(default)]
    pub method: Option<String>,
    /// Warp only: elasticity control (≥1). Sizes the warp search window — bounding the
    /// marker's total displacement from the rigid pick — and scales the per-step stretch
    /// penalty (larger ⇒ more elastic). A soft control, not a hard per-sample slope cap.
    /// Absent ⇒ 1.5.
    #[serde(default)]
    pub max_stretch: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoCorrProposal {
    pub well_id: String,
    /// The target well's existing depth for this top (None = top not picked there yet).
    pub current_depth: Option<f32>,
    pub proposed_depth: Option<f32>,
    /// Pearson correlation of the log shapes at the proposed depth (−1..1).
    pub correlation: f32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoCorrResult {
    pub proposals: Vec<AutoCorrProposal>,
    pub error: Option<String>,
}

/// Linear interpolation of `vals` (aligned with ascending `depth`) at `x`.
/// NaN if outside the sampled range or if either bracketing sample is NaN.
fn interp(depth: &[f32], vals: &[f32], x: f32) -> f32 {
    if depth.is_empty() || x < depth[0] || x > depth[depth.len() - 1] {
        return f32::NAN;
    }
    let idx = depth.partition_point(|&d| d < x);
    if idx == 0 {
        return vals[0];
    }
    if idx >= depth.len() {
        return vals[depth.len() - 1];
    }
    let (d0, d1) = (depth[idx - 1], depth[idx]);
    let (v0, v1) = (vals[idx - 1], vals[idx]);
    if !v0.is_finite() || !v1.is_finite() {
        return f32::NAN;
    }
    if d1 <= d0 {
        return v0;
    }
    v0 + (v1 - v0) * (x - d0) / (d1 - d0)
}

/// Pearson correlation over index pairs where both series are finite.
/// Returns (r, n_used).
fn pearson(a: &[f32], b: &[f32]) -> (f32, usize) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (&x, &y) in a.iter().zip(b) {
        if x.is_finite() && y.is_finite() {
            xs.push(x as f64);
            ys.push(y as f64);
        }
    }
    let n = xs.len();
    if n < 4 {
        return (f32::NAN, n);
    }
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (dx, dy) = (xs[i] - mx, ys[i] - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return (f32::NAN, n);
    }
    ((sxy / (sxx * syy).sqrt()) as f32, n)
}

/// Proposes the marker depth in each target well by sliding the source well's log
/// shape (a ±half_window window around the marker) over the target's log within
/// ±search_range of the initial guess (the target's existing pick, else the source
/// depth) and keeping the depth of maximum Pearson correlation. The user reviews and
/// applies proposals in the dialog — nothing is written here.
pub fn autocorrelate_top(conn: &Connection, req: &AutoCorrRequest) -> AutoCorrResult {
    let fail = |msg: String| AutoCorrResult { proposals: Vec::new(), error: Some(msg) };

    if !(req.half_window > 0.0) || !(req.search_range > 0.0) {
        return fail("window and search range must be positive".into());
    }
    let source_tops = match db::list_tops(conn, &req.source_well_id) {
        Ok(t) => t,
        Err(e) => return fail(e.to_string()),
    };
    let source_depth = match source_tops
        .iter()
        .find(|t| t.top_name.eq_ignore_ascii_case(&req.top_name))
    {
        Some(t) => t.depth,
        None => return fail(format!("top '{}' is not picked in the source well", req.top_name)),
    };

    let curve_names = vec![req.curve.to_uppercase()];
    let (src_depth, src_curves) = match fetch_curve_frame(conn, &req.source_well_id, &curve_names) {
        Ok(f) => f,
        Err(e) => return fail(e.to_string()),
    };
    let src_vals = match src_curves.get(&curve_names[0]) {
        Some(v) if !v.is_empty() => v,
        _ => return fail(format!("source well has no '{}' data", req.curve)),
    };

    // Template: the source log sampled on a uniform offset grid across the window.
    let step = median_step(&src_depth).max(1e-3);
    let k = ((2.0 * req.half_window / step).round() as usize).clamp(16, 512);
    let offsets: Vec<f32> = (0..=k)
        .map(|i| -req.half_window + (2.0 * req.half_window) * i as f32 / k as f32)
        .collect();
    let template: Vec<f32> = offsets.iter().map(|&o| interp(&src_depth, src_vals, source_depth + o)).collect();
    let finite = template.iter().filter(|v| v.is_finite()).count();
    if finite < template.len() * 2 / 3 {
        return fail(format!(
            "not enough '{}' data around {:.1} in the source well",
            req.curve, source_depth
        ));
    }

    let warp = req.method.as_deref().map_or(false, |m| m.eq_ignore_ascii_case("warp"));
    let max_stretch = req.max_stretch.unwrap_or(1.5).clamp(1.0, 3.0);

    let mut proposals = Vec::new();
    for target_id in &req.target_well_ids {
        if target_id == &req.source_well_id {
            continue;
        }
        let mut proposal = AutoCorrProposal {
            well_id: target_id.clone(),
            current_depth: None,
            proposed_depth: None,
            correlation: f32::NAN,
            error: None,
        };
        proposal.current_depth = db::list_tops(conn, target_id)
            .ok()
            .and_then(|tops| tops.into_iter().find(|t| t.top_name.eq_ignore_ascii_case(&req.top_name)))
            .map(|t| t.depth);

        match fetch_curve_frame(conn, target_id, &curve_names) {
            Ok((tgt_depth, tgt_curves)) => {
                let tgt_vals = match tgt_curves.get(&curve_names[0]) {
                    Some(v) if !v.is_empty() => v,
                    _ => {
                        proposal.error = Some(format!("no '{}' data", req.curve));
                        proposals.push(proposal);
                        continue;
                    }
                };
                let guess = proposal.current_depth.unwrap_or(source_depth);
                let scan_step = (step / 2.0).max(1e-3);
                match best_shift(&template, &offsets, &tgt_depth, tgt_vals, guess, req.search_range, scan_step) {
                    None => proposal.error = Some(format!("no overlapping '{}' data in the search range", req.curve)),
                    Some((dc, rc)) => {
                        // Warp refines the rigid pick with a monotone depth warp. Keep the warp
                        // only when it fits at least as well as the rigid pick (same Pearson
                        // metric, small epsilon for reconstruction noise); otherwise — or if the
                        // warp degenerated / placed the marker in a data gap — fall back to the
                        // rigid answer, so warp can never silently regress a good rigid match.
                        let picked = if warp {
                            match warp_refine(&template, &offsets, &tgt_depth, tgt_vals, dc, max_stretch, step) {
                                Some((wd, wr)) if wr + 0.01 >= rc => (wd, wr),
                                _ => (dc, rc),
                            }
                        } else {
                            (dc, rc)
                        };
                        proposal.proposed_depth = Some(picked.0);
                        proposal.correlation = picked.1;
                    }
                }
            }
            Err(e) => proposal.error = Some(e.to_string()),
        }
        proposals.push(proposal);
    }
    AutoCorrResult { proposals, error: None }
}

fn median_step(depth: &[f32]) -> f32 {
    if depth.len() < 2 {
        return 0.5;
    }
    let mut steps: Vec<f32> = depth.windows(2).map(|w| w[1] - w[0]).filter(|s| *s > 0.0).collect();
    if steps.is_empty() {
        return 0.5;
    }
    steps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    steps[steps.len() / 2]
}

/// Rigid best-lag: slides `template` (sampled on `offsets` about a centre depth) over
/// the target log within ±`search_range` of `guess`, at `scan_step`, keeping the depth
/// of maximum Pearson correlation. Returns (depth, r) or None if nothing overlapped.
fn best_shift(
    template: &[f32],
    offsets: &[f32],
    tgt_depth: &[f32],
    tgt_vals: &[f32],
    guess: f32,
    search_range: f32,
    scan_step: f32,
) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    let steps = ((2.0 * search_range) / scan_step).round() as i64;
    for s in 0..=steps {
        let c = guess - search_range + s as f32 * scan_step;
        let window: Vec<f32> = offsets.iter().map(|&o| interp(tgt_depth, tgt_vals, c + o)).collect();
        let (r, n) = pearson(template, &window);
        if !r.is_finite() || n < template.len() * 2 / 3 {
            continue;
        }
        if best.map_or(true, |(_, br)| r > br) {
            best = Some((c, r));
        }
    }
    best
}

/// Linear-interpolated percentile (0–100) of an ascending-sorted slice.
fn pctl(sorted: &[f32], p: f32) -> f32 {
    let n = sorted.len();
    if n == 0 {
        return f32::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let rank = (p / 100.0).clamp(0.0, 1.0) * (n - 1) as f32;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo as f32)
}

/// Maps the finite samples of `w` onto ~[0,1] using its own P3/P97 (the gr_normalize
/// two-point idea, applied window-locally) so the amplitude-based warp cost compares
/// SHAPE, not tool calibration or datum. No-op on flat/short windows; NaNs stay NaN.
fn normalize_window(w: &mut [f32]) {
    let mut finite: Vec<f32> = w.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.len() < 2 {
        return;
    }
    finite.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lo = pctl(&finite, 3.0);
    let hi = pctl(&finite, 97.0);
    if !(hi - lo > 1e-6) {
        return;
    }
    for v in w.iter_mut() {
        if v.is_finite() {
            *v = ((*v - lo) / (hi - lo)).clamp(-0.25, 1.25);
        }
    }
}

/// Open-begin / open-end subsequence DTW: warps source template `a` (fully consumed)
/// onto the best-fitting stretch of the longer target `b`. Step set is (1,1) diagonal,
/// (1,0) source-advance = compression, (0,1) target-advance = stretch — which makes the
/// path strictly monotone and non-inverting; each non-diagonal step adds `penalty`, so
/// the alignment hugs slope 1 unless the data clearly warps. Returns the optimal path as
/// (source_idx, target_idx) pairs (ascending) and the mean per-sample cost. Non-finite
/// samples contribute a mild fixed cost rather than aborting.
fn subseq_dtw(a: &[f32], b: &[f32], penalty: f32) -> Option<(Vec<(usize, usize)>, f32)> {
    let n = a.len();
    let m = b.len();
    if n == 0 || m < n {
        return None;
    }
    let big = f32::INFINITY;
    let dist = |i: usize, j: usize| -> f32 {
        match (a[i], b[j]) {
            (x, y) if x.is_finite() && y.is_finite() => (x - y).abs(),
            // NaN pair: a moderate fixed cost so the path is discouraged from — but not
            // forbidden to — cross short data gaps. Normalized amplitudes span ~[0,1].
            _ => 0.50,
        }
    };
    let mut d = vec![vec![big; m]; n];
    let mut back = vec![vec![0u8; m]; n]; // 0 = diag, 1 = up (1,0), 2 = left (0,1)
    for j in 0..m {
        d[0][j] = dist(0, j); // free start: the template may begin anywhere in b
    }
    for i in 1..n {
        for j in 0..m {
            let diag = if j >= 1 { d[i - 1][j - 1] } else { big };
            let up = d[i - 1][j] + penalty;
            let left = if j >= 1 { d[i][j - 1] + penalty } else { big };
            let (mut bestv, mut bk) = (diag, 0u8);
            if up < bestv {
                bestv = up;
                bk = 1;
            }
            if left < bestv {
                bestv = left;
                bk = 2;
            }
            if bestv.is_finite() {
                d[i][j] = bestv + dist(i, j);
                back[i][j] = bk;
            }
        }
    }
    // Free end: cheapest cell on the last source row.
    let (mut jstar, mut bestend) = (0usize, big);
    for j in 0..m {
        if d[n - 1][j] < bestend {
            bestend = d[n - 1][j];
            jstar = j;
        }
    }
    if !bestend.is_finite() {
        return None;
    }
    let mut path = Vec::with_capacity(n);
    let (mut i, mut j) = (n - 1, jstar);
    loop {
        path.push((i, j));
        if i == 0 {
            break;
        }
        match back[i][j] {
            0 => {
                i -= 1;
                j -= 1;
            }
            1 => i -= 1,
            2 => j -= 1,
            _ => break,
        }
    }
    path.reverse();
    Some((path, bestend / n as f32))
}

/// Elastic-warp refinement of the rigid pick `dc`. Builds a target window sized for a
/// `max_stretch` warp of the marker's position, P3/P97-normalizes both logs, runs
/// subsequence DTW, and reads off the target depth the marker (offset 0, template centre)
/// warps to. The reported r is Pearson of the template against the target warped back onto
/// the source grid — the same metric as the rigid r, so the caller can compare them.
/// Returns None (⇒ caller keeps the rigid pick) when the window is too sparse to warp OR
/// the marker warps into a data gap OR the fit is unscoreable.
fn warp_refine(
    template: &[f32],
    offsets: &[f32],
    tgt_depth: &[f32],
    tgt_vals: &[f32],
    dc: f32,
    max_stretch: f32,
    step: f32,
) -> Option<(f32, f32)> {
    let n = template.len();
    let half_win = offsets.last().copied().unwrap_or(0.0).abs();
    if !(half_win > 0.0) || !(step > 0.0) {
        return None;
    }
    // Target window centred on the rigid pick, long enough to hold a max_stretch warp.
    let half_span = half_win * max_stretch + 2.0 * step;
    let b_start = dc - half_span;
    let m = ((2.0 * half_span / step).round() as usize).max(n + 2);
    let b_raw: Vec<f32> = (0..=m).map(|jj| interp(tgt_depth, tgt_vals, b_start + jj as f32 * step)).collect();
    if b_raw.iter().filter(|v| v.is_finite()).count() < b_raw.len() / 2 {
        return None;
    }

    let mut a = template.to_vec();
    normalize_window(&mut a);
    let mut b = b_raw.clone();
    normalize_window(&mut b);
    let penalty = (0.30 / max_stretch).clamp(0.05, 0.30);
    let (path, _cost) = subseq_dtw(&a, &b, penalty)?;

    // Marker = the template sample nearest offset 0.
    let center = offsets
        .iter()
        .enumerate()
        .min_by(|(_, x), (_, y)| x.abs().partial_cmp(&y.abs()).unwrap())
        .map(|(idx, _)| idx)?;

    // Mean target index aligned to each source index (the path may map several j to one i).
    let mean_j_at = |i: usize| -> Option<f32> {
        let js: Vec<usize> = path.iter().filter(|(pi, _)| *pi == i).map(|(_, j)| *j).collect();
        if js.is_empty() {
            None
        } else {
            Some(js.iter().map(|&j| j as f32).sum::<f32>() / js.len() as f32)
        }
    };
    let mean_j = mean_j_at(center)?;
    // Reject a marker that warped into a data gap (NaN target sample): the rigid pick,
    // scored on real overlap, is safer there than a depth pulled across a null interval.
    let marker_idx = (mean_j.round() as usize).min(b_raw.len() - 1);
    if !b_raw[marker_idx].is_finite() {
        return None;
    }
    let proposed = b_start + mean_j * step;

    // Quality: warp the target back onto the source grid and Pearson-score against the template.
    let mut tgt_on_src = vec![f32::NAN; n];
    for (i, cell) in tgt_on_src.iter_mut().enumerate() {
        if let Some(mj) = mean_j_at(i) {
            let idx = (mj.round() as usize).min(b_raw.len() - 1);
            *cell = b_raw[idx];
        }
    }
    let (r, _) = pearson(template, &tgt_on_src);
    if !r.is_finite() {
        return None; // unscoreable warp → caller keeps the rigid pick
    }
    Some((proposed, r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn open_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        conn
    }

    /// Seeds a well whose GR is a smooth sine plus one sharp spike at `spike_depth`,
    /// giving the correlator a unique feature to lock onto.
    fn seed_gr_well(conn: &Connection, name: &str, spike_depth: f32) -> String {
        let id = Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let n = 400usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = depth
            .iter()
            .map(|&d| {
                let base = 60.0 + 25.0 * (d / 15.0).sin();
                let spike = 80.0 * (-((d - spike_depth) / 2.0).powi(2)).exp();
                base + spike
            })
            .collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan)
            .unwrap();
        id.to_string()
    }

    /// A GR pattern with a sinusoid backbone plus two sharp Gaussian markers, evaluated
    /// on a "pattern-depth" axis. Warping a target = feeding it a stretched pattern-depth.
    fn base_gr(x: f32) -> f32 {
        60.0 + 25.0 * ((x - 1000.0) / 8.0).sin()
            + 70.0 * (-(((x - 1090.0) / 3.0).powi(2))).exp()
            + 55.0 * (-(((x - 1116.0) / 3.0).powi(2))).exp()
    }

    /// Seeds a well whose GR at well-depth d is `base_gr(coord(d))`. `coord` is identity
    /// for the source and the inverse warp for a stretched target.
    fn seed_pattern_well(conn: &Connection, name: &str, coord: impl Fn(f32) -> f32) -> String {
        let id = Uuid::new_v4();
        db::insert_well(conn, id, name, Some("Synthetic"), None, None).unwrap();
        let n = 440usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = depth.iter().map(|&d| base_gr(coord(d))).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan)
            .unwrap();
        id.to_string()
    }

    #[test]
    fn crossing_check_flags_reversed_pair() {
        let conn = open_db();
        let w1 = seed_gr_well(&conn, "ORDER-1", 1050.0);
        let w2 = seed_gr_well(&conn, "ORDER-2", 1050.0);
        let w3 = seed_gr_well(&conn, "ORDER-3", 1050.0);
        for w in [&w1, &w2] {
            db::upsert_top(&conn, w, "TOP_A", 1000.0, None).unwrap();
            db::upsert_top(&conn, w, "TOP_B", 1100.0, None).unwrap();
        }
        // Reversed in w3: B above A.
        db::upsert_top(&conn, &w3, "TOP_A", 1120.0, None).unwrap();
        db::upsert_top(&conn, &w3, "TOP_B", 1080.0, None).unwrap();

        let w3_warnings = check_top_order(&conn, &w3).unwrap();
        assert_eq!(w3_warnings.len(), 1, "expected one crossing warning: {w3_warnings:?}");
        assert!(w3_warnings[0].contains("TOP_B") && w3_warnings[0].contains("2 of 2"), "{}", w3_warnings[0]);
        // The conforming wells are outvoted 1:1? No — each sees one reversed (w3) vs one same,
        // tie means "same or better" wins and no warning is raised.
        assert!(check_top_order(&conn, &w1).unwrap().is_empty());
    }

    #[test]
    fn autocorrelate_finds_shifted_marker() {
        let conn = open_db();
        let src = seed_gr_well(&conn, "AC-SRC", 1100.0);
        let tgt = seed_gr_well(&conn, "AC-TGT", 1107.5); // same spike, 7.5 m deeper
        db::upsert_top(&conn, &src, "MFS-8", 1100.0, None).unwrap();

        let result = autocorrelate_top(
            &conn,
            &AutoCorrRequest {
                source_well_id: src.clone(),
                top_name: "MFS-8".into(),
                curve: "GR".into(),
                half_window: 10.0,
                search_range: 20.0,
                target_well_ids: vec![tgt.clone()],
                method: None,
                max_stretch: None,
            },
        );
        assert!(result.error.is_none(), "{:?}", result.error);
        let p = &result.proposals[0];
        assert!(p.error.is_none(), "{:?}", p.error);
        let proposed = p.proposed_depth.expect("proposal expected");
        assert!(
            (proposed - 1107.5).abs() <= 1.0,
            "proposed {proposed}, expected ~1107.5 (r={})",
            p.correlation
        );
        assert!(p.correlation > 0.9, "correlation too low: {}", p.correlation);
        assert_eq!(p.current_depth, None);
    }

    #[test]
    fn autocorrelate_reports_missing_curve_and_top() {
        let conn = open_db();
        let src = seed_gr_well(&conn, "AC-ERR-SRC", 1100.0);
        // Target well with no curve data at all.
        let bare = Uuid::new_v4();
        db::insert_well(&conn, bare, "AC-ERR-TGT", None, None, None).unwrap();
        db::upsert_top(&conn, &src, "TOP_X", 1100.0, None).unwrap();

        let result = autocorrelate_top(
            &conn,
            &AutoCorrRequest {
                source_well_id: src.clone(),
                top_name: "TOP_X".into(),
                curve: "GR".into(),
                half_window: 10.0,
                search_range: 20.0,
                target_well_ids: vec![bare.to_string()],
                method: None,
                max_stretch: None,
            },
        );
        assert!(result.error.is_none());
        assert!(result.proposals[0].error.is_some(), "bare well should error");

        // Unpicked source top → request-level error.
        let missing = autocorrelate_top(
            &conn,
            &AutoCorrRequest {
                source_well_id: src,
                top_name: "NOT_PICKED".into(),
                curve: "GR".into(),
                half_window: 10.0,
                search_range: 20.0,
                target_well_ids: vec![],
                method: None,
                max_stretch: None,
            },
        );
        assert!(missing.error.is_some());
    }

    #[test]
    fn autocorrelate_warp_recovers_stretched_section() {
        let conn = open_db();
        // Source: identity pattern; marker picked at pattern-depth 1100.
        let src = seed_pattern_well(&conn, "WARP-SRC", |d| d);
        // Target: two-piece monotone warp of the pattern axis. Above pattern-depth 1050
        // it is ×1.2 about 1000; below it is ×1.5, continuous at the kink
        // (kink_src 1050 → kink_tgt 1000 + 1.2·50 = 1060). We seed with the INVERSE
        // (target-well-depth → pattern-depth) so the target log is the stretched section.
        let inv_warp = |dt: f32| -> f32 {
            let kink_t = 1060.0;
            if dt <= kink_t {
                1000.0 + (dt - 1000.0) / 1.2
            } else {
                1050.0 + (dt - kink_t) / 1.5
            }
        };
        let tgt = seed_pattern_well(&conn, "WARP-TGT", inv_warp);
        db::upsert_top(&conn, &src, "MFS", 1100.0, None).unwrap();
        // True marker depth in target: warp(1100) = 1060 + 1.5·(1100−1050) = 1135.
        let truth = 1135.0f32;

        let base = AutoCorrRequest {
            source_well_id: src.clone(),
            top_name: "MFS".into(),
            curve: "GR".into(),
            half_window: 25.0,
            search_range: 60.0,
            target_well_ids: vec![tgt.clone()],
            method: None,
            max_stretch: None,
        };
        // Rigid shift cannot stretch, so its window-centre is biased off the true depth.
        let rigid = autocorrelate_top(&conn, &base);
        let rigid_d = rigid.proposals[0].proposed_depth.expect("rigid depth");

        // Warp aligns both markers under the ×1.5 stretch → recovers ~1135.
        let warp = autocorrelate_top(
            &conn,
            &AutoCorrRequest { method: Some("warp".into()), max_stretch: Some(1.8), ..base },
        );
        let p = &warp.proposals[0];
        assert!(p.error.is_none(), "{:?}", p.error);
        let warp_d = p.proposed_depth.expect("warp depth");
        assert!(
            (warp_d - truth).abs() <= 4.0,
            "warp {warp_d} not within 4 m of {truth} (rigid was {rigid_d}, r={})",
            p.correlation
        );
        // Warp is at least as accurate as rigid (strictly better in practice here).
        assert!(
            (warp_d - truth).abs() <= (rigid_d - truth).abs() + 0.5,
            "warp {warp_d} no better than rigid {rigid_d} vs truth {truth}"
        );
        assert!(p.correlation > 0.85, "warp r too low: {}", p.correlation);
    }

    #[test]
    fn autocorrelate_warp_matches_rigid_on_pure_shift() {
        // No stretch, just a 7.5 m shift. The better-of guard must keep warp from
        // regressing the rigid answer — warp should still land on the shifted marker.
        let conn = open_db();
        let src = seed_gr_well(&conn, "WS-SRC", 1100.0);
        let tgt = seed_gr_well(&conn, "WS-TGT", 1107.5);
        db::upsert_top(&conn, &src, "MFS-8", 1100.0, None).unwrap();
        let result = autocorrelate_top(
            &conn,
            &AutoCorrRequest {
                source_well_id: src.clone(),
                top_name: "MFS-8".into(),
                curve: "GR".into(),
                half_window: 10.0,
                search_range: 20.0,
                target_well_ids: vec![tgt.clone()],
                method: Some("warp".into()),
                max_stretch: Some(1.5),
            },
        );
        let p = &result.proposals[0];
        assert!(p.error.is_none(), "{:?}", p.error);
        let d = p.proposed_depth.expect("depth");
        assert!((d - 1107.5).abs() <= 1.5, "warp {d} should match the pure-shift ~1107.5 (r={})", p.correlation);
        assert!(p.correlation > 0.9, "r too low: {}", p.correlation);
    }

    #[test]
    fn subseq_dtw_path_is_monotone_and_complete() {
        // Source: a sine. Target: a ~×1.5 stretched, deterministically jittered version.
        let a: Vec<f32> = (0..60).map(|i| ((i as f32) / 6.0).sin()).collect();
        let b: Vec<f32> = (0..96)
            .map(|i| {
                let jitter = 0.04 * (((i * 7) % 5) as f32 - 2.0); // deterministic pseudo-noise
                ((i as f32) / 9.0).sin() + jitter
            })
            .collect();
        let (path, cost) = subseq_dtw(&a, &b, 0.1).expect("dtw path");
        assert!(cost.is_finite());
        // Monotone non-decreasing in both axes — no depth inversion.
        for w in path.windows(2) {
            assert!(w[1].0 >= w[0].0 && w[1].1 >= w[0].1, "path inverted {:?}->{:?}", w[0], w[1]);
        }
        // Every source sample consumed, first→last, all indices in range.
        assert_eq!(path.first().unwrap().0, 0);
        assert_eq!(path.last().unwrap().0, a.len() - 1);
        assert!(path.iter().all(|&(i, j)| i < a.len() && j < b.len()));
    }
}
