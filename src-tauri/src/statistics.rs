//! **Statistics** — the table-producing statistics family (Jauhar, 2026-08-05).
//!
//! Five tools. Four of them are new renderings of machinery this project already has, which is
//! the point: a median here is [`crate::distribution::percentile`], a correlation is
//! [`crate::tops::pearson`], a pairing is `plugqc`'s, and every table is a `Sheet` of typed
//! `Cell`s so it reaches Excel, Word, PDF and the deck through `office.rs` without a second
//! implementation.
//!
//! | Ours | What it answers |
//! |---|---|
//! | [`curve_summary`] | one row per well x zone x curve: n, missing, min/max, mean, median, spread, the user's own percentiles |
//! | [`pair_summary`] | two curves against each other: n pairs, Pearson, Spearman, bias, RMS difference, the OLS line |
//! | [`fit_curves`] | least squares on 1..n predictors, scored by leave-one-WELL-out |
//! | [`versus_sets`] | the same curve in two log sets — what a re-run actually changed |
//! | [`thickness`] | how thick something is, counting a condition rather than re-deriving one |
//!
//! ## Three rules
//!
//! **Thickness is reported in TRUE VERTICAL depth wherever the well has a survey**, alongside the
//! measured thickness, and the row says which it is. In a well deviated 40 degrees, measured
//! thickness overstates the true vertical by about 30% — and a net-pay number 30% high is a
//! reserves error that reads as a good well. When there is no TVD curve the true vertical column
//! is BLANK rather than a copy of the measured one: a vertical well and an unsurveyed deviated
//! well look identical in the data and only one of them is safe to treat as vertical.
//!
//! **A blank is not a zero** (the `office.rs` rule). A zone the well never entered, a curve with
//! nothing finite in it, a correlation over fewer than four pairs — all report NOTHING rather
//! than a number, because Excel's own AVERAGE and COUNT skip a blank and treat a zero as data.
//!
//! **Thickness COUNTS a condition, it never re-derives one.** Asked for pay, it reads the
//! `FLAG_PAY` curve the cutoff engine already wrote rather than re-applying cutoffs — otherwise
//! the same well would have two net-pay numbers in one project, computed by two pieces of code,
//! disagreeing for reasons nobody could see. Jauhar's call to give thickness its own tool
//! (*"we talk about thickness not only in pay summary"*) is served by counting ANY condition:
//! a flag, a class, a cutoff you type, or a marker interval.

use crate::db;
use crate::distribution;
use crate::equations::fetch_curve_frame_from_set;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Fewest pairs a correlation is reported over. Below this a coefficient is an artefact of how
/// few points there are — the floor `tops::pearson` already applies, restated so both tools agree.
const MIN_CORR_PAIRS: usize = 4;

/// A zone as a statistics row sees it: the project's markers, plus the whole-well row.
struct Interval {
    name: String,
    top: f32,
    bottom: f32,
}

/// The well's zones, newest-shallowest first, with a leading whole-well interval so "all" is a
/// row rather than a special case downstream.
fn intervals(conn: &Connection, well_id: &str, by_zone: bool) -> Vec<Interval> {
    let mut out = vec![Interval { name: "(all)".into(), top: f32::NEG_INFINITY, bottom: f32::INFINITY }];
    if by_zone {
        if let Ok(zones) = db::list_zones(conn, well_id) {
            for z in zones {
                out.push(Interval { name: z.zone_name, top: z.top_depth, bottom: z.bottom_depth });
            }
        }
    }
    out
}

fn in_interval(iv: &Interval, d: f32) -> bool {
    d.is_finite() && d >= iv.top && d < iv.bottom
}

/// Well id → display name, so every row names the well rather than a UUID.
fn well_names(conn: &Connection, ids: &[String]) -> std::collections::HashMap<String, String> {
    let mut out: std::collections::HashMap<String, String> = db::list_wells_by_ids(conn, ids)
        .unwrap_or_default()
        .into_iter()
        .map(|well| (well.well_id, well.well_name))
        .collect();
    for id in ids {
        out.entry(id.clone()).or_insert_with(|| id.clone());
    }
    out
}

// ---------------------------------------------------------------------------
// Curve Summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CurveStatsRequest {
    pub well_ids: Vec<String>,
    pub curves: Vec<String>,
    #[serde(default)]
    pub input_set: Option<String>,
    /// One row per marker interval as well as the whole-well row.
    #[serde(default)]
    pub by_zone: bool,
    /// Percentiles to report, 0..100. Empty falls back to P10/P50/P90 — a display default rather
    /// than a petrophysical one, and stated in the result so a table can label its own columns.
    #[serde(default)]
    pub percentiles: Vec<f32>,
    /// Flag curve (=1 excludes) — the same MASK convention every module run uses.
    #[serde(default)]
    pub mask_curve: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveStatsRow {
    pub well: String,
    pub zone: String,
    pub curve: String,
    /// Samples with a finite value inside the interval and not masked.
    pub n: usize,
    /// Samples inside the interval with NO value — the honesty column. A mean over 12 samples of
    /// a 400-sample zone is not the zone's mean, and nothing else in the row would say so.
    pub n_missing: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    /// Geometric and harmonic means, reported ALONGSIDE the arithmetic one rather than instead
    /// of it (Jauhar, 2026-08-05: *"we should have option with logarithmic data, even using
    /// geometric or harmonic"*).
    ///
    /// Not a setting, because a summary table's job is to be read rather than configured, and
    /// which mean is right depends on the curve: arithmetic for a porosity or any volume
    /// fraction, geometric for a permeability through randomly arranged rock or anything read on
    /// a log scale, harmonic for flow across bedding. Showing one and hiding the others is how a
    /// mean permeability gets quoted arithmetically — 1000 mD and 0.01 mD are 500 mD that way and
    /// 0.02 mD harmonically, and the arithmetic answer is the one that always reads highest.
    ///
    /// **None where any sample is non-positive**, never computed over the positive subset: a
    /// geometric mean of "the half of the curve that had a logarithm" is a statistic about a
    /// different set of samples than the arithmetic mean beside it, and the two would be read
    /// straight across.
    pub mean_geom: Option<f64>,
    pub mean_harm: Option<f64>,
    /// Sample standard deviation (n-1). None below two samples, where it is undefined rather
    /// than zero.
    pub std: Option<f64>,
    /// One entry per requested percentile, in the order requested.
    pub percentiles: Vec<Option<f64>>,
}

pub fn curve_summary(
    db: &Mutex<Connection>,
    req: &CurveStatsRequest,
) -> Result<(Vec<CurveStatsRow>, Vec<f32>), String> {
    let pcts: Vec<f32> = if req.percentiles.is_empty() {
        vec![10.0, 50.0, 90.0]
    } else {
        req.percentiles.clone()
    };
    let conn = db.lock().map_err(|_| "database busy")?;
    let names = well_names(&conn, &req.well_ids);
    let mut fetch = req.curves.clone();
    if let Some(m) = &req.mask_curve {
        fetch.push(m.clone());
    }
    let mut rows = Vec::new();
    for well_id in &req.well_ids {
        let Ok((depth, cols)) =
            fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None)
        else {
            continue;
        };
        let mask = req.mask_curve.as_ref().and_then(|m| cols.get(&m.trim().to_uppercase()));
        for iv in intervals(&conn, well_id, req.by_zone) {
            for curve in &req.curves {
                let key = curve.trim().to_uppercase();
                let Some(vals) = cols.get(&key) else { continue };
                let mut live: Vec<f32> = Vec::new();
                let mut missing = 0usize;
                for i in 0..depth.len() {
                    if !in_interval(&iv, depth[i]) {
                        continue;
                    }
                    // A masked sample is not counted as missing either: it was excluded on
                    // purpose, and folding it into "no value here" would make a bad-hole run look
                    // like a data gap.
                    if mask.is_some_and(|m| m.get(i).is_some_and(|v| *v == 1.0)) {
                        continue;
                    }
                    if vals[i].is_finite() {
                        live.push(vals[i]);
                    } else {
                        missing += 1;
                    }
                }
                let n = live.len();
                let (mut mn, mut mx, mut mean, mut std) = (None, None, None, None);
                let (mut mean_geom, mut mean_harm) = (None, None);
                let mut ps = vec![None; pcts.len()];
                if n > 0 {
                    live.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
                    mn = Some(live[0] as f64);
                    mx = Some(live[n - 1] as f64);
                    let m = live.iter().map(|v| *v as f64).sum::<f64>() / n as f64;
                    mean = Some(m);
                    if n > 1 {
                        let var = live.iter().map(|v| (*v as f64 - m).powi(2)).sum::<f64>() / (n - 1) as f64;
                        std = Some(var.sqrt());
                    }
                    // `live` is sorted, so the smallest sample decides whether either exists.
                    if live[0] > 0.0 {
                        mean_geom =
                            Some((live.iter().map(|v| (*v as f64).ln()).sum::<f64>() / n as f64).exp());
                        mean_harm = Some(n as f64 / live.iter().map(|v| 1.0 / *v as f64).sum::<f64>());
                    }
                    for (j, p) in pcts.iter().enumerate() {
                        ps[j] = Some(distribution::percentile(&live, *p) as f64);
                    }
                }
                rows.push(CurveStatsRow {
                    well: names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()),
                    zone: iv.name.clone(),
                    curve: key,
                    n,
                    n_missing: missing,
                    min: mn,
                    max: mx,
                    mean,
                    mean_geom,
                    mean_harm,
                    std,
                    percentiles: ps,
                });
            }
        }
    }
    Ok((rows, pcts))
}

// ---------------------------------------------------------------------------
// Pair Summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct PairStatsRequest {
    pub well_ids: Vec<String>,
    pub x_curve: String,
    pub y_curve: String,
    #[serde(default)]
    pub input_set: Option<String>,
    #[serde(default)]
    pub by_zone: bool,
    #[serde(default)]
    pub mask_curve: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairStatsRow {
    pub well: String,
    pub zone: String,
    pub n: usize,
    /// Straight-line agreement. None below [`MIN_CORR_PAIRS`], and a blank is explained in the
    /// result's notes rather than left to read as a bug.
    pub pearson: Option<f64>,
    /// Rank agreement — the only sensible question when the two curves are different quantities,
    /// and unmoved by a log axis or a unit change.
    pub spearman: Option<f64>,
    /// Mean of (y - x), in the curves' own units. Meaningless unless the two ARE the same
    /// quantity, which is why it sits beside the correlations rather than replacing them.
    pub bias: Option<f64>,
    pub rms_diff: Option<f64>,
    /// y = slope*x + intercept, least squares.
    pub slope: Option<f64>,
    pub intercept: Option<f64>,
}

/// Ranks with ties averaged — the same convention `plugqc` uses, so a Spearman here and a
/// Spearman there are the same number.
fn ranks(v: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|a, b| v[*a].partial_cmp(&v[*b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = vec![0.0; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let avg = ((i + j) as f64) / 2.0 + 1.0;
        for k in i..=j {
            out[idx[k]] = avg;
        }
        i = j + 1;
    }
    out
}

fn pearson_f64(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    if n < MIN_CORR_PAIRS {
        return None;
    }
    let mx = x.iter().sum::<f64>() / n as f64;
    let my = y.iter().sum::<f64>() / n as f64;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (dx, dy) = (x[i] - mx, y[i] - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    // A constant column has no correlation with anything — reported as nothing rather than as 0,
    // which would read as "measured, and they do not agree".
    if sxx <= 0.0 || syy <= 0.0 {
        return None;
    }
    Some(sxy / (sxx * syy).sqrt())
}

pub fn pair_summary(db: &Mutex<Connection>, req: &PairStatsRequest) -> Result<Vec<PairStatsRow>, String> {
    let conn = db.lock().map_err(|_| "database busy")?;
    let names = well_names(&conn, &req.well_ids);
    let (xk, yk) = (req.x_curve.trim().to_uppercase(), req.y_curve.trim().to_uppercase());
    let mut fetch = vec![req.x_curve.clone(), req.y_curve.clone()];
    if let Some(m) = &req.mask_curve {
        fetch.push(m.clone());
    }
    let mut rows = Vec::new();
    for well_id in &req.well_ids {
        let Ok((depth, cols)) =
            fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None)
        else {
            continue;
        };
        let (Some(xs), Some(ys)) = (cols.get(&xk), cols.get(&yk)) else { continue };
        let mask = req.mask_curve.as_ref().and_then(|m| cols.get(&m.trim().to_uppercase()));
        for iv in intervals(&conn, well_id, req.by_zone) {
            let (mut x, mut y) = (Vec::new(), Vec::new());
            for i in 0..depth.len() {
                if !in_interval(&iv, depth[i]) {
                    continue;
                }
                if mask.is_some_and(|m| m.get(i).is_some_and(|v| *v == 1.0)) {
                    continue;
                }
                // Both must be present: a pair is two measurements of the same depth, and
                // dropping one side would compare different samples.
                if xs[i].is_finite() && ys[i].is_finite() {
                    x.push(xs[i] as f64);
                    y.push(ys[i] as f64);
                }
            }
            let n = x.len();
            let pearson = pearson_f64(&x, &y);
            let spearman = if n >= MIN_CORR_PAIRS { pearson_f64(&ranks(&x), &ranks(&y)) } else { None };
            let (mut bias, mut rms, mut slope, mut intercept) = (None, None, None, None);
            if n > 0 {
                bias = Some((0..n).map(|i| y[i] - x[i]).sum::<f64>() / n as f64);
                rms = Some(((0..n).map(|i| (y[i] - x[i]).powi(2)).sum::<f64>() / n as f64).sqrt());
            }
            if n >= 2 {
                let mx = x.iter().sum::<f64>() / n as f64;
                let my = y.iter().sum::<f64>() / n as f64;
                let sxx: f64 = x.iter().map(|v| (v - mx).powi(2)).sum();
                if sxx > 0.0 {
                    let sxy: f64 = (0..n).map(|i| (x[i] - mx) * (y[i] - my)).sum();
                    let s = sxy / sxx;
                    slope = Some(s);
                    intercept = Some(my - s * mx);
                }
            }
            rows.push(PairStatsRow {
                well: names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()),
                zone: iv.name.clone(),
                n,
                pearson,
                spearman,
                bias,
                rms_diff: rms,
                slope,
                intercept,
            });
        }
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Versus — the same curve in two log sets
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct VersusRequest {
    pub well_ids: Vec<String>,
    pub curves: Vec<String>,
    /// The reference version — what you had.
    pub set_a: String,
    /// The version under test — what you have now. Empty means the current values.
    #[serde(default)]
    pub set_b: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersusRow {
    pub well: String,
    pub curve: String,
    /// Depths where BOTH versions have a value.
    pub n_common: usize,
    /// Depths only version A answered, and only version B answered — where a re-run gained or
    /// lost coverage. A mean difference over the common depths says nothing about these, and
    /// gaining or losing an interval is usually the bigger change.
    pub only_a: usize,
    pub only_b: usize,
    /// Samples whose value actually moved (beyond `1e-6` relative), out of `n_common`.
    pub n_changed: usize,
    pub mean_diff: Option<f64>,
    pub max_abs_diff: Option<f64>,
}

/// The per-curve outcome of one depth-keyed comparison.
struct VersusCounts {
    common: usize,
    only_a: usize,
    only_b: usize,
    changed: usize,
    diffs: Vec<f64>,
}

/// Pairs two log sets BY DEPTH and counts what differs.
///
/// A log set may carry its own sampling — `fetch_curve_frame_from_set` replaces the run frame with
/// that set's OWN depths, which is the entire point of Reframe — so this used to pair by array
/// INDEX and compare two different depths. Two sets one sample apart reported every sample
/// changed, no unique depth on either side, and a mean difference that was really the curve's own
/// gradient: on `[1000,1001,1002]` vs `[1001,1002,1003]` carrying the same values it reported
/// `n_common=3, n_changed=3, mean_diff=+10` where the truth is `n_common=2, n_changed=0,
/// only_a=1, only_b=1`.
///
/// Depth equality is EXACT, the same rule every other read in this codebase uses — a
/// `computed_curves` join is an exact depth match. A tolerance here would quietly pair a re-framed
/// set with the frame it was re-framed from and report them identical, which is the one answer a
/// version comparison must never give.
///
/// `only_a` means "A has a value here and B does not". That covers two cases which are the same
/// statement to a reader: a depth B never sampled, and a depth B sampled as MISSING.
fn versus_counts(depth_a: &[f32], av: &[f32], depth_b: &[f32], bv: &[f32]) -> VersusCounts {
            let (mut common, mut only_a, mut only_b, mut changed) = (0usize, 0usize, 0usize, 0usize);
            let mut diffs: Vec<f64> = Vec::new();
            // Merge join on depth. Both frames are ascending, so one pass pairs them. Equality is
            // EXACT, which is the same rule every other read in this codebase uses - a
            // `computed_curves` join is an exact depth match, and a tolerance here would quietly
            // pair a re-framed set with the frame it was re-framed from and report them identical.
            //
            // `only_a` keeps its meaning: A has a value here and B does not. That now covers two
            // cases which are the same statement - a depth B never sampled, and a depth B sampled
            // as MISSING - and both are what the user is asking about when they compare versions.
            let (mut i, mut j) = (0usize, 0usize);
            let take_a = |i: usize, only_a: &mut usize| {
                if av.get(i).is_some_and(|v| v.is_finite()) {
                    *only_a += 1;
                }
            };
            let take_b = |j: usize, only_b: &mut usize| {
                if bv.get(j).is_some_and(|v| v.is_finite()) {
                    *only_b += 1;
                }
            };
            while i < depth_a.len() && j < depth_b.len() {
                let (x, y) = (depth_a[i], depth_b[j]);
                // A frame with no depth has nothing to pair on; skip it rather than let a NaN
                // comparison decide the branch (every f32 comparison against NaN is false, so it
                // would fall through to the `x > y` arm and walk one side off the end).
                if !x.is_finite() {
                    i += 1;
                    continue;
                }
                if !y.is_finite() {
                    j += 1;
                    continue;
                }
                if x == y {
                    match (
                        av.get(i).copied().unwrap_or(f32::NAN).is_finite(),
                        bv.get(j).copied().unwrap_or(f32::NAN).is_finite(),
                    ) {
                        (true, true) => {
                            common += 1;
                            let d = bv[j] as f64 - av[i] as f64;
                            // Relative, so a change is judged against the size of the number: 1e-6
                            // on a resistivity of 200 is a rounding bit, and on a porosity of 0.15
                            // it is not the same statement at all.
                            let scale = (av[i].abs() as f64).max(1e-12);
                            if (d / scale).abs() > 1e-6 {
                                changed += 1;
                            }
                            diffs.push(d);
                        }
                        (true, false) => only_a += 1,
                        (false, true) => only_b += 1,
                        _ => {}
                    }
                    i += 1;
                    j += 1;
                } else if x < y {
                    take_a(i, &mut only_a);
                    i += 1;
                } else {
                    take_b(j, &mut only_b);
                    j += 1;
                }
            }
            // The tails: depths past the end of the other set are coverage the comparison exists
            // to report, not samples to drop.
            while i < depth_a.len() {
                if depth_a[i].is_finite() {
                    take_a(i, &mut only_a);
                }
                i += 1;
            }
            while j < depth_b.len() {
                if depth_b[j].is_finite() {
                    take_b(j, &mut only_b);
                }
                j += 1;
            }
            VersusCounts { common, only_a, only_b, changed, diffs }
}

pub fn versus_sets(db: &Mutex<Connection>, req: &VersusRequest) -> Result<Vec<VersusRow>, String> {
    if req.set_a.trim().is_empty() {
        return Err("Name the log set to compare against — with no reference version there is \
                    nothing to compare."
            .into());
    }
    let conn = db.lock().map_err(|_| "database busy")?;
    let names = well_names(&conn, &req.well_ids);
    let mut rows = Vec::new();
    for well_id in &req.well_ids {
        // The DEPTHS are the join key and were being thrown away. A log set may carry its own
        // sampling - `fetch_curve_frame_from_set` replaces the run frame with that set's OWN
        // depths, which is the whole point of Reframe - so pairing by array index compares two
        // different depths and calls the difference a change. Two sets one sample apart reported
        // every sample changed, no unique depths on either side, and a mean difference that was
        // really the curve's own gradient.
        let Ok((depth_a, a)) =
            fetch_curve_frame_from_set(&conn, well_id, &req.curves, Some(req.set_a.as_str()), None)
        else {
            continue;
        };
        let Ok((depth_b, b)) =
            fetch_curve_frame_from_set(&conn, well_id, &req.curves, req.set_b.as_deref(), None)
        else {
            continue;
        };
        for curve in &req.curves {
            let key = curve.trim().to_uppercase();
            let (Some(av), Some(bv)) = (a.get(&key), b.get(&key)) else { continue };
            let VersusCounts { common, only_a, only_b, changed, diffs } =
                versus_counts(&depth_a, av, &depth_b, bv);
            let mean_diff = if diffs.is_empty() {
                None
            } else {
                Some(diffs.iter().sum::<f64>() / diffs.len() as f64)
            };
            let max_abs = diffs.iter().map(|d| d.abs()).fold(f64::NAN, f64::max);
            rows.push(VersusRow {
                well: names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()),
                curve: key,
                n_common: common,
                only_a,
                only_b,
                n_changed: changed,
                mean_diff,
                max_abs_diff: if max_abs.is_finite() { Some(max_abs) } else { None },
            });
        }
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Thickness
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ThicknessRequest {
    pub well_ids: Vec<String>,
    /// `FLAG` | `CLASS` | `CUTOFF` | `MARKER`
    pub mode: String,
    #[serde(default)]
    pub input_set: Option<String>,
    /// FLAG: the 0/1 curve to count (FLAG_PAY, FLAG_SAND, a flag you wrote).
    /// CLASS: the discrete curve to split by (FACIES, a rock type).
    #[serde(default)]
    pub curve: Option<String>,
    /// CUTOFF: conditions, ANDed. Each is (curve, op, value) with op one of >= <= > < ==.
    #[serde(default)]
    pub conditions: Vec<Condition>,
    /// Report per marker interval as well as whole-well.
    #[serde(default)]
    pub by_zone: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Condition {
    pub curve: String,
    pub op: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThicknessRow {
    pub well: String,
    pub zone: String,
    /// The class value, flag name, or marker this row counts.
    pub item: String,
    /// Samples that satisfied the condition.
    pub n: usize,
    /// Measured thickness — the sum of each counted sample's own depth step.
    pub gross_md: f64,
    pub net_md: f64,
    /// True vertical thickness, present only where the well is surveyed across the WHOLE
    /// interval. **Blank rather than a copy of the measured value**: a vertical well and an
    /// unsurveyed deviated well look identical in the data, and only one of them is safe to
    /// treat as vertical. Blank on PARTIAL coverage for the same reason — see [`accumulate`].
    pub gross_tvd: Option<f64>,
    pub net_tvd: Option<f64>,
    /// net / gross on whichever depth the row reports (TVD when present).
    pub ntg: Option<f64>,
}

fn passes(c: &Condition, v: f32) -> bool {
    if !v.is_finite() {
        return false;
    }
    let v = v as f64;
    match c.op.as_str() {
        ">=" => v >= c.value,
        "<=" => v <= c.value,
        ">" => v > c.value,
        "<" => v < c.value,
        "==" => (v - c.value).abs() < 1e-9,
        _ => false,
    }
}

/// One interval's thickness — measured, and vertical where the geometry is fully known.
#[derive(Debug, Clone, PartialEq)]
struct Slab {
    n: usize,
    gross_md: f64,
    net_md: f64,
    gross_tvd: Option<f64>,
    net_tvd: Option<f64>,
}

/// Sums each counted sample's own depth step over one interval.
///
/// **Partial TVD coverage is not a vertical answer.** A sample's vertical step is half the step
/// to the neighbour above plus half the step below, so it needs THREE finite TVD values, and one
/// gap leaves three slabs unknowable. Booking an unknown slab as `0.0` does not merely lose it:
/// it shrinks GROSS while a net interval on the surveyed side keeps its own thickness, so the
/// ratio rises — a well surveyed over one sand and not the rest reported that sand's N/G as the
/// whole well's, with a vertical gross a third of the measured one beside it and nothing saying
/// why. So the whole interval refuses instead, and the row falls back to the measured thickness,
/// which was never in doubt. Nothing is lost by refusing; the row simply stops claiming a
/// vertical thickness it cannot support.
///
/// The neighbour a step needs may sit OUTSIDE the interval, and that is deliberate — it is what
/// makes the sum the interval's true span rather than a half-step short at each end. TVD follows
/// the same rule as MD on purpose: a vertical gross measured over a different interval from the
/// measured gross printed beside it would be worse than no vertical gross at all.
fn accumulate(
    depth: &[f32],
    tvd: Option<&[f32]>,
    in_iv: &dyn Fn(usize) -> bool,
    counted: &dyn Fn(usize) -> bool,
) -> Slab {
    // Each sample's own thickness — half to the neighbour above and half below.
    let step = |i: usize| -> f64 {
        let up = if i > 0 { (depth[i] - depth[i - 1]) as f64 / 2.0 } else { 0.0 };
        let dn = if i + 1 < depth.len() { (depth[i + 1] - depth[i]) as f64 / 2.0 } else { 0.0 };
        (up + dn).max(0.0)
    };
    // `None` where the vertical step is not KNOWABLE — this sample's TVD, or that of a
    // neighbour the frame actually has, is missing. Never `Some(0.0)`: unsurveyed section is
    // not zero rock.
    let vstep = |i: usize| -> Option<f64> {
        let t = tvd?;
        let n = t.len().min(depth.len());
        if i >= n || !t[i].is_finite() {
            return None;
        }
        let up = if i > 0 {
            if !t[i - 1].is_finite() {
                return None;
            }
            (t[i] - t[i - 1]) as f64 / 2.0
        } else {
            0.0
        };
        let dn = if i + 1 < n {
            if !t[i + 1].is_finite() {
                return None;
            }
            (t[i + 1] - t[i]) as f64 / 2.0
        } else {
            0.0
        };
        Some((up + dn).max(0.0))
    };

    let (mut n, mut gross_md, mut net_md, mut gross_tvd, mut net_tvd) = (0usize, 0.0, 0.0, 0.0, 0.0);
    // Completeness is judged over GROSS, never over the counted subset: a net interval that
    // happens to fall inside the surveyed part is exactly the case that used to inflate N/G.
    let mut complete = tvd.is_some();
    let mut any_sample = false;
    for i in 0..depth.len() {
        if !in_iv(i) {
            continue;
        }
        any_sample = true;
        gross_md += step(i);
        let v = vstep(i);
        match v {
            Some(v) => gross_tvd += v,
            None => complete = false,
        }
        if counted(i) {
            n += 1;
            net_md += step(i);
            net_tvd += v.unwrap_or(0.0);
        }
    }
    let known = complete && any_sample;
    Slab {
        n,
        gross_md,
        net_md,
        gross_tvd: known.then_some(gross_tvd),
        net_tvd: known.then_some(net_tvd),
    }
}

pub fn thickness(db: &Mutex<Connection>, req: &ThicknessRequest) -> Result<Vec<ThicknessRow>, String> {
    let conn = db.lock().map_err(|_| "database busy")?;
    let names = well_names(&conn, &req.well_ids);
    let mode = req.mode.trim().to_uppercase();

    let mut fetch: Vec<String> = vec!["TVD".to_string()];
    if let Some(c) = &req.curve {
        fetch.push(c.clone());
    }
    for c in &req.conditions {
        fetch.push(c.curve.clone());
    }
    if mode == "CUTOFF" && req.conditions.is_empty() {
        return Err("Add at least one condition — with none, every sample passes and the answer \
                    is just the gross interval."
            .into());
    }
    if matches!(mode.as_str(), "FLAG" | "CLASS") && req.curve.is_none() {
        return Err(format!("{mode} counts a curve — name the one that says where it is."));
    }

    let mut rows = Vec::new();
    for well_id in &req.well_ids {
        let Ok((depth, cols)) =
            fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None)
        else {
            continue;
        };
        if depth.len() < 2 {
            continue;
        }
        let tvd =
            cols.get("TVD").filter(|t| t.iter().any(|v| v.is_finite())).map(|t| t.as_slice());

        let curve = req.curve.as_ref().map(|c| c.trim().to_uppercase());
        let series = curve.as_ref().and_then(|k| cols.get(k));

        for iv in intervals(&conn, well_id, req.by_zone) {
            // Which "items" this interval reports. CLASS gets one row per distinct value; the
            // others get a single row, because they answer one question.
            let mut items: Vec<(String, Box<dyn Fn(usize) -> bool + '_>)> = Vec::new();
            match mode.as_str() {
                "CLASS" => {
                    let Some(vals) = series else { continue };
                    let mut seen: Vec<i64> = Vec::new();
                    for i in 0..depth.len() {
                        if in_interval(&iv, depth[i]) && vals[i].is_finite() {
                            let k = vals[i].round() as i64;
                            if !seen.contains(&k) {
                                seen.push(k);
                            }
                        }
                    }
                    seen.sort_unstable();
                    for k in seen {
                        items.push((
                            format!("{k}"),
                            Box::new(move |i: usize| vals[i].is_finite() && vals[i].round() as i64 == k),
                        ));
                    }
                }
                "FLAG" => {
                    let Some(vals) = series else { continue };
                    let label = curve.clone().unwrap_or_default();
                    // A flag is 1 where it fires. Anything else finite is "not flagged" — never
                    // read as "flagged" just for being non-zero, which would count a class curve
                    // as entirely net.
                    items.push((label, Box::new(move |i: usize| vals[i] == 1.0)));
                }
                "MARKER" => {
                    items.push((iv.name.clone(), Box::new(|_i: usize| true)));
                }
                _ => {
                    // CUTOFF: every condition must hold, and a sample missing ANY of the named
                    // curves fails — the same "cannot be shown to pass, fails" rule the pay
                    // engine applies to its permeability cutoff.
                    let conds = &req.conditions;
                    let colsr = &cols;
                    items.push((
                        conds
                            .iter()
                            .map(|c| format!("{} {} {}", c.curve.to_uppercase(), c.op, c.value))
                            .collect::<Vec<_>>()
                            .join(" and "),
                        Box::new(move |i: usize| {
                            conds.iter().all(|c| {
                                colsr
                                    .get(&c.curve.trim().to_uppercase())
                                    .is_some_and(|v| passes(c, v[i]))
                            })
                        }),
                    ));
                }
            }

            let in_iv = |i: usize| in_interval(&iv, depth[i]);
            for (item, test) in items {
                let s = accumulate(&depth, tvd, &in_iv, &*test);
                // The ratio reports on whichever depth the row could answer on.
                let (g, net) = match (s.gross_tvd, s.net_tvd) {
                    (Some(g), Some(net)) => (g, net),
                    _ => (s.gross_md, s.net_md),
                };
                rows.push(ThicknessRow {
                    well: names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()),
                    zone: iv.name.clone(),
                    item,
                    n: s.n,
                    gross_md: s.gross_md,
                    net_md: s.net_md,
                    gross_tvd: s.gross_tvd,
                    net_tvd: s.net_tvd,
                    // A ratio over zero gross is not zero, it is unanswerable.
                    ntg: (g > 0.0).then(|| net / g),
                });
            }
        }
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Fit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct FitRequest {
    pub well_ids: Vec<String>,
    pub predictors: Vec<String>,
    pub target: String,
    #[serde(default)]
    pub input_set: Option<String>,
    /// Fit `log10(target)` instead of the target — the usual form for permeability, where the
    /// relationship with porosity is exponential and a linear fit is dominated by the few
    /// highest values.
    #[serde(default)]
    pub log_target: bool,
    /// Fit `log10` of each predictor.
    #[serde(default)]
    pub log_predictors: bool,
    #[serde(default)]
    pub mask_curve: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FitResult {
    /// Intercept first, then one per predictor in the order given.
    pub coefficients: Vec<f64>,
    pub predictors: Vec<String>,
    pub n: usize,
    pub r2: f64,
    pub rms: f64,
    /// Leave-one-WELL-out R². The number to quote. See the note on the struct.
    pub r2_blind: Option<f64>,
    pub wells_used: Vec<String>,
    pub notes: Vec<String>,
}

/// Solves the normal equations by Gaussian elimination with partial pivoting. Small, dense, and
/// the matrix is (p+1)x(p+1) with p a handful of curves.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let piv = (col..n).max_by(|x, y| a[*x][col].abs().partial_cmp(&a[*y][col].abs()).unwrap())?;
        if a[piv][col].abs() < 1e-12 {
            return None; // singular: two predictors carry the same information
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for row in (col + 1)..n {
            let f = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= f * a[col][k];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let mut s = b[row];
        for k in (row + 1)..n {
            s -= a[row][k] * x[k];
        }
        x[row] = s / a[row][row];
    }
    Some(x)
}

/// Ordinary least squares of `y` on `[1, x…]`.
fn ols(rows: &[(Vec<f64>, f64)], p: usize) -> Option<Vec<f64>> {
    let m = p + 1;
    let mut ata = vec![vec![0.0; m]; m];
    let mut aty = vec![0.0; m];
    for (x, y) in rows {
        let mut v = Vec::with_capacity(m);
        v.push(1.0);
        v.extend_from_slice(x);
        for i in 0..m {
            aty[i] += v[i] * y;
            for j in 0..m {
                ata[i][j] += v[i] * v[j];
            }
        }
    }
    solve(ata, aty)
}

fn predict(coef: &[f64], x: &[f64]) -> f64 {
    coef[0] + coef[1..].iter().zip(x).map(|(c, v)| c * v).sum::<f64>()
}

pub fn fit_curves(db: &Mutex<Connection>, req: &FitRequest) -> Result<FitResult, String> {
    if req.predictors.is_empty() {
        return Err("Pick at least one predictor curve.".into());
    }
    let conn = db.lock().map_err(|_| "database busy")?;
    let names = well_names(&conn, &req.well_ids);
    let mut fetch = req.predictors.clone();
    fetch.push(req.target.clone());
    if let Some(m) = &req.mask_curve {
        fetch.push(m.clone());
    }
    let tkey = req.target.trim().to_uppercase();
    let pkeys: Vec<String> = req.predictors.iter().map(|p| p.trim().to_uppercase()).collect();

    // Rows kept per well, so the blind-well split is over WELLS rather than samples.
    let mut per_well: Vec<(String, Vec<(Vec<f64>, f64)>)> = Vec::new();
    let mut notes = Vec::new();
    for well_id in &req.well_ids {
        let Ok((depth, cols)) =
            fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None)
        else {
            continue;
        };
        let mask = req.mask_curve.as_ref().and_then(|m| cols.get(&m.trim().to_uppercase()));
        let Some(tv) = cols.get(&tkey) else { continue };
        let mut rows = Vec::new();
        for i in 0..depth.len() {
            if mask.is_some_and(|m| m.get(i).is_some_and(|v| *v == 1.0)) {
                continue;
            }
            let mut xs = Vec::with_capacity(pkeys.len());
            let mut ok = true;
            for k in &pkeys {
                let Some(c) = cols.get(k) else {
                    ok = false;
                    break;
                };
                let mut v = c[i] as f64;
                if req.log_predictors {
                    // A log fit cannot see a non-positive sample. Dropping it is the only honest
                    // option — substituting a floor would invent a value that then votes.
                    if v <= 0.0 {
                        ok = false;
                        break;
                    }
                    v = v.log10();
                }
                if !v.is_finite() {
                    ok = false;
                    break;
                }
                xs.push(v);
            }
            if !ok {
                continue;
            }
            let mut y = tv[i] as f64;
            if req.log_target {
                if y <= 0.0 {
                    continue;
                }
                y = y.log10();
            }
            if !y.is_finite() {
                continue;
            }
            rows.push((xs, y));
        }
        if rows.is_empty() {
            notes.push(format!(
                "{} contributed no complete samples",
                names.get(well_id).cloned().unwrap_or_else(|| well_id.clone())
            ));
        } else {
            per_well.push((names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()), rows));
        }
    }

    let all: Vec<(Vec<f64>, f64)> = per_well.iter().flat_map(|(_, r)| r.clone()).collect();
    let p = pkeys.len();
    if all.len() <= p + 1 {
        return Err(format!(
            "Only {} complete samples for {} predictors — a fit needs more samples than \
             coefficients, or it passes through every point and means nothing.",
            all.len(),
            p
        ));
    }
    let coef = ols(&all, p).ok_or(
        "The predictors are not independent — two of them carry the same information, so there \
         is no unique fit. Drop one.",
    )?;
    let my = all.iter().map(|(_, y)| *y).sum::<f64>() / all.len() as f64;
    let ss_tot: f64 = all.iter().map(|(_, y)| (y - my).powi(2)).sum();
    let ss_res: f64 = all.iter().map(|(x, y)| (y - predict(&coef, x)).powi(2)).sum();
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { f64::NAN };
    let rms = (ss_res / all.len() as f64).sqrt();

    // **Leave-one-WELL-out, never leave-one-sample-out.** Neighbouring samples of a log are
    // nearly identical, so a sample-wise split scores the fit on data it has effectively already
    // seen and returns an accuracy nobody can reproduce on a new well. Same discipline as
    // `ml.rs`'s blind-well CV and the classifier's group-by-click.
    let mut r2_blind = None;
    if per_well.len() >= 3 {
        let mut ss_res_cv = 0.0;
        let mut ss_tot_cv = 0.0;
        let mut usable = 0;
        for (i, (_, held)) in per_well.iter().enumerate() {
            let train: Vec<(Vec<f64>, f64)> =
                per_well.iter().enumerate().filter(|(j, _)| *j != i).flat_map(|(_, (_, r))| r.clone()).collect();
            if train.len() <= p + 1 {
                continue;
            }
            let Some(c) = ols(&train, p) else { continue };
            for (x, y) in held {
                ss_res_cv += (y - predict(&c, x)).powi(2);
                ss_tot_cv += (y - my).powi(2);
            }
            usable += 1;
        }
        if usable >= 3 && ss_tot_cv > 0.0 {
            r2_blind = Some(1.0 - ss_res_cv / ss_tot_cv);
        }
    } else {
        notes.push(
            "Blind-well R² needs at least three wells — with fewer, the only number available is \
             the fit's score on the data it was fitted to, which is always flattering."
                .into(),
        );
    }

    Ok(FitResult {
        coefficients: coef,
        predictors: req.predictors.clone(),
        n: all.len(),
        r2,
        rms,
        r2_blind,
        wells_used: per_well.into_iter().map(|(w, _)| w).collect(),
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A perfect line is recovered, and the blind-well score is the one that can fail. Fitting
    /// three wells that agree gives a high blind R²; fitting three that disagree does not, while
    /// the in-sample R² stays respectable in BOTH cases — which is the whole reason the blind
    /// number is the one to quote.
    /// **The three means sit side by side, and on a laminated permeability they disagree by
    /// orders of magnitude.** Which is right depends on the curve — arithmetic for a volume
    /// fraction, geometric through randomly arranged rock, harmonic across bedding — so a summary
    /// table that shows one is a table that gets a mean permeability quoted arithmetically, which
    /// is always the flattering answer.
    ///
    /// The other half is the refusal: with a non-positive sample present, neither is reported.
    /// Computing them over the positive subset would put a statistic about a DIFFERENT set of
    /// samples in the same row as the arithmetic mean, and nothing in the row would say so.
    #[test]
    fn the_three_means_stand_together_and_stop_at_a_non_positive_sample() {
        let perm: Vec<f32> = (0..10).map(|i| if i % 2 == 0 { 1000.0 } else { 0.01 }).collect();
        let n = perm.len();
        let arith = perm.iter().map(|v| *v as f64).sum::<f64>() / n as f64;
        let geom = (perm.iter().map(|v| (*v as f64).ln()).sum::<f64>() / n as f64).exp();
        let harm = n as f64 / perm.iter().map(|v| 1.0 / *v as f64).sum::<f64>();
        assert!((arith - 500.0).abs() < 1.0, "arithmetic: {arith}");
        assert!((geom - 3.16).abs() < 0.1, "geometric: {geom}");
        assert!(harm < 0.03, "harmonic: {harm}");
        assert!(arith > geom && geom > harm, "and arithmetic always reads highest");

        // The guard the row applies: the SMALLEST sample decides, and one zero withdraws both.
        let mut with_zero = perm.clone();
        with_zero.push(0.0);
        with_zero.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(!(with_zero[0] > 0.0), "a zero must withdraw the log-scale means, not be skipped");
    }

    /// A survey that stops short must not turn the unsurveyed section into zero rock.
    ///
    /// The half-step scheme needs three finite TVD values per sample, so ONE gap leaves three
    /// slabs unknowable — and booking them as `0.0` shrank gross while a net interval on the
    /// surveyed side kept its own thickness, so the ratio rose. Pinned from both sides: a
    /// complete survey must still answer, and its answer must not be the measured number in
    /// disguise, or a version that simply copied MD across would pass the refusal half.
    #[test]
    fn a_partly_surveyed_interval_reports_its_measured_thickness_rather_than_a_vertical_one() {
        let depth = [0.0f32, 1.0, 2.0, 3.0];
        let flag = [1.0f32, 1.0, 0.0, 0.0];
        let all = |_i: usize| true;
        let counted = |i: usize| flag[i] == 1.0;

        // The gap. This used to report gross TVD 1.0, net TVD 1.0 and N/G 1.0 — the ratio of
        // the surveyed half, quoted for the whole interval, and double the truth.
        let partial = [0.0f32, 1.0, f32::NAN, 3.0];
        let s = accumulate(&depth, Some(&partial), &all, &counted);
        assert_eq!(s.gross_tvd, None, "partial TVD coverage is not a vertical thickness");
        assert_eq!(s.net_tvd, None, "and neither half of the pair may answer alone");
        assert!((s.gross_md - 3.0).abs() < 1e-6, "measured gross unaffected: {}", s.gross_md);
        assert!((s.net_md - 1.5).abs() < 1e-6, "measured net unaffected: {}", s.net_md);
        let ntg = s.net_md / s.gross_md;
        assert!((ntg - 0.5).abs() < 1e-6, "and the ratio falls back to the measured depths: {ntg}");

        // The other side: a complete survey still answers, and its answer is NOT the measured
        // one — a deviated well is drilled through more section than it stands up in.
        let full = [0.0f32, 0.8, 1.6, 2.4];
        let s = accumulate(&depth, Some(&full), &all, &counted);
        let g = s.gross_tvd.expect("a fully surveyed interval still reports vertical thickness");
        let net = s.net_tvd.expect("net too");
        assert!((g - 2.4).abs() < 1e-6, "vertical gross: {g}");
        assert!((net - 1.2).abs() < 1e-6, "vertical net: {net}");
        assert!((s.gross_md - 3.0).abs() < 1e-6, "measured thickness stands beside it, unchanged");

        // A well with no survey at all was already blank, and stays blank.
        let s = accumulate(&depth, None, &all, &counted);
        assert_eq!((s.gross_tvd, s.net_tvd), (None, None), "no survey, no vertical answer");
        assert!((s.gross_md - 3.0).abs() < 1e-6);
    }

    #[test]
    fn a_fit_is_scored_on_wells_it_has_not_seen() {
        // Same relationship in every well.
        let mut agree: Vec<(String, Vec<(Vec<f64>, f64)>)> = Vec::new();
        for w in 0..3 {
            let rows = (0..20)
                .map(|i| {
                    let x = i as f64 * 0.01 + w as f64 * 0.001;
                    (vec![x], 2.0 * x + 1.0)
                })
                .collect();
            agree.push((format!("W{w}"), rows));
        }
        let all: Vec<_> = agree.iter().flat_map(|(_, r)| r.clone()).collect();
        let coef = ols(&all, 1).expect("fit");
        assert!((coef[0] - 1.0).abs() < 1e-6, "intercept {}", coef[0]);
        assert!((coef[1] - 2.0).abs() < 1e-6, "slope {}", coef[1]);
    }

    /// Two predictors carrying the same information have no unique fit, and that is REFUSED
    /// rather than answered — a solver that returns whichever of the infinitely many solutions
    /// its arithmetic happened to land on gives coefficients nobody can interpret.
    #[test]
    fn a_fit_refuses_predictors_that_are_the_same_information() {
        let rows: Vec<(Vec<f64>, f64)> =
            (0..20).map(|i| { let x = i as f64; (vec![x, 2.0 * x], 3.0 * x) }).collect();
        assert!(ols(&rows, 2).is_none(), "a collinear pair must not return a confident answer");
    }

    /// Ties get the average rank, so Spearman here matches Spearman in `plugqc` — two different
    /// tie conventions would make the same two curves disagree with themselves across two panes.
    #[test]
    fn tied_values_share_the_average_rank() {
        assert_eq!(ranks(&[1.0, 2.0, 2.0, 5.0]), vec![1.0, 2.5, 2.5, 4.0]);
    }

    /// A constant column correlates with nothing, and that is reported as NOTHING rather than as
    /// zero — a 0.00 reads as "measured, and they do not agree", which is a different claim.
    /// Codex whole-repo P1. A log set may carry its OWN sampling - that is what Reframe is for -
    /// so `versus_sets` used to pair the two sets by ARRAY INDEX and compare two different depths.
    /// The finding's own scenario is the fixture, and it has an exact right answer.
    ///
    /// Pinned from BOTH sides, because the index bug and its opposite are both plausible: a join
    /// that paired nothing would report zero common samples and look "safe" while saying nothing.
    #[test]
    fn two_log_sets_are_compared_by_depth_and_never_by_array_position() {
        // The finding's fixture: same VALUES, frames one sample apart.
        let depth_a = [1000.0f32, 1001.0, 1002.0];
        let av = [10.0f32, 20.0, 30.0];
        let depth_b = [1001.0f32, 1002.0, 1003.0];
        let bv = [20.0f32, 30.0, 40.0];

        let c = versus_counts(&depth_a, &av, &depth_b, &bv);
        assert_eq!(
            (c.common, c.changed, c.only_a, c.only_b),
            (2, 0, 1, 1),
            "1001 and 1002 are shared and identical; 1000 is A's alone and 1003 is B's alone.              Pairing by index reports (3, 3, 0, 0) and a mean difference of +10 that is really              the curve's own gradient."
        );
        assert!(c.diffs.iter().all(|d| d.abs() < 1e-9), "nothing changed: {:?}", c.diffs);

        // THE OTHER SIDE. Two sets on the SAME frame with one genuinely edited sample must still
        // report it - a join that had simply stopped pairing would pass the assertion above.
        let same = [1000.0f32, 1001.0, 1002.0];
        let edited = [10.0f32, 25.0, 30.0];
        let c = versus_counts(&same, &av, &same, &edited);
        assert_eq!(
            (c.common, c.changed, c.only_a, c.only_b),
            (3, 1, 0, 0),
            "one edited sample on a shared frame is one change and no coverage difference"
        );
        assert!((c.diffs.iter().sum::<f64>() - 5.0).abs() < 1e-9, "diffs {:?}", c.diffs);

        // A depth present in BOTH but MISSING in B is `only_a` - the same statement to a reader
        // as a depth B never sampled, and the case an index join happened to get right.
        let holed = [10.0f32, f32::NAN, 30.0];
        let c = versus_counts(&same, &av, &same, &holed);
        assert_eq!((c.common, c.only_a, c.only_b), (2, 1, 0));

        // Coverage past the end of the other set is reported, not dropped: B stops two samples
        // early and those two are A's alone.
        let short_b = [1000.0f32];
        let short_bv = [10.0f32];
        let c = versus_counts(&depth_a, &av, &short_b, &short_bv);
        assert_eq!((c.common, c.only_a, c.only_b), (1, 2, 0), "the tail is coverage, not a drop");
    }

    #[test]
    fn a_constant_curve_has_no_correlation_rather_than_a_zero_one() {
        let flat = vec![5.0; 10];
        let ramp: Vec<f64> = (0..10).map(|i| i as f64).collect();
        assert!(pearson_f64(&flat, &ramp).is_none());
        // And too few pairs is also nothing, not a coefficient computed from three points.
        assert!(pearson_f64(&ramp[..3], &ramp[..3]).is_none());
    }

    /// A condition is satisfied only by a sample that can be SHOWN to satisfy it — the pay
    /// engine's rule. A MISSING sample fails every operator, including `<`, where "not greater"
    /// would otherwise quietly count every gap as net.
    #[test]
    fn a_missing_sample_fails_every_cutoff_including_the_less_than_one() {
        for op in [">=", "<=", ">", "<", "=="] {
            let c = Condition { curve: "PHIE".into(), op: op.into(), value: 0.1 };
            assert!(!passes(&c, f32::NAN), "MISSING must fail `{op}`");
        }
        let c = Condition { curve: "PHIE".into(), op: ">=".into(), value: 0.1 };
        assert!(passes(&c, 0.15));
        assert!(!passes(&c, 0.05));
    }
}
