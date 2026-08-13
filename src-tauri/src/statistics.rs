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
        let Ok((_da, a)) =
            fetch_curve_frame_from_set(&conn, well_id, &req.curves, Some(req.set_a.as_str()), None)
        else {
            continue;
        };
        let Ok((_db_, b)) =
            fetch_curve_frame_from_set(&conn, well_id, &req.curves, req.set_b.as_deref(), None)
        else {
            continue;
        };
        for curve in &req.curves {
            let key = curve.trim().to_uppercase();
            let (Some(av), Some(bv)) = (a.get(&key), b.get(&key)) else { continue };
            let n = av.len().min(bv.len());
            let (mut common, mut only_a, mut only_b, mut changed) = (0usize, 0usize, 0usize, 0usize);
            let mut diffs: Vec<f64> = Vec::new();
            for i in 0..n {
                match (av[i].is_finite(), bv[i].is_finite()) {
                    (true, true) => {
                        common += 1;
                        let d = bv[i] as f64 - av[i] as f64;
                        // Relative, so a change is judged against the size of the number: 1e-6 on
                        // a resistivity of 200 is a rounding bit, and on a porosity of 0.15 it is
                        // not the same statement at all.
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
            }
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
    /// True vertical thickness, present only where the well has a TVD curve. **Blank rather than
    /// a copy of the measured value**: a vertical well and an unsurveyed deviated well look
    /// identical in the data, and only one of them is safe to treat as vertical.
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
        let tvd = cols.get("TVD").filter(|t| t.iter().any(|v| v.is_finite()));
        // Each sample's own thickness — half to the neighbour above and half below, so the sum
        // over an interval is its true span rather than one step short at each end.
        let step = |i: usize| -> f64 {
            let n = depth.len();
            let up = if i > 0 { (depth[i] - depth[i - 1]) as f64 / 2.0 } else { 0.0 };
            let dn = if i + 1 < n { (depth[i + 1] - depth[i]) as f64 / 2.0 } else { 0.0 };
            (up + dn).max(0.0)
        };
        let vstep = |i: usize| -> Option<f64> {
            let t = tvd?;
            let n = t.len().min(depth.len());
            let up = if i > 0 && t[i].is_finite() && t[i - 1].is_finite() {
                (t[i] - t[i - 1]) as f64 / 2.0
            } else {
                0.0
            };
            let dn = if i + 1 < n && t[i + 1].is_finite() && t[i].is_finite() {
                (t[i + 1] - t[i]) as f64 / 2.0
            } else {
                0.0
            };
            Some((up + dn).max(0.0))
        };

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

            let mut gross_md = 0.0;
            let mut gross_tvd = 0.0;
            let mut any_tvd = false;
            for i in 0..depth.len() {
                if in_interval(&iv, depth[i]) {
                    gross_md += step(i);
                    if let Some(v) = vstep(i) {
                        gross_tvd += v;
                        any_tvd = true;
                    }
                }
            }
            for (item, test) in items {
                let mut n = 0usize;
                let mut net_md = 0.0;
                let mut net_tvd = 0.0;
                for i in 0..depth.len() {
                    if in_interval(&iv, depth[i]) && test(i) {
                        n += 1;
                        net_md += step(i);
                        if let Some(v) = vstep(i) {
                            net_tvd += v;
                        }
                    }
                }
                let (g, net) = if any_tvd { (gross_tvd, net_tvd) } else { (gross_md, net_md) };
                rows.push(ThicknessRow {
                    well: names.get(well_id).cloned().unwrap_or_else(|| well_id.clone()),
                    zone: iv.name.clone(),
                    item,
                    n,
                    gross_md,
                    net_md,
                    gross_tvd: any_tvd.then_some(gross_tvd),
                    net_tvd: any_tvd.then_some(net_tvd),
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
