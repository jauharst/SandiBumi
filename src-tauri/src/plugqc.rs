//! Pairing two measurements made on the SAME plug, so one can be checked against the other.
//!
//! The petrography measurements now in the project (`VPORE_TS`, `PORE_D50`, …) are numbers nobody
//! has yet plotted against anything independent. A thin-section pore area estimating a volume
//! fraction by the Delesse relation is a *claim*, and the only way to find out whether it holds on
//! this rock is to put it beside the helium porosity measured on the plug the section was cut
//! from. Same for a pore-body diameter beside the throat radii the capillary-pressure curve
//! reports. This module does the pairing and the arithmetic; the pane does the picture.
//!
//! Four rules hold it together.
//!
//! **A pair is two measurements of the same plug, and nothing else.** Depths are matched within a
//! tolerance and a sample with no partner inside it is DROPPED and COUNTED — never snapped to the
//! nearest one, which is the same rule the S-factor calibration follows and for the same reason: a
//! shift that is a whole number of sample intervals is invisible to any tolerance check, so
//! loosening the tolerance to "get more points" quietly pairs a plug with its neighbour. Register
//! the core first; that is what `registration.rs` is for.
//!
//! **A measurement may be used ONCE.** Two thin sections cut a centimetre apart would otherwise
//! both claim the one plug porosity nearest them, and that single core value would appear twice in
//! the cloud and twice in the correlation, tightening it for free. Pairing is therefore greedy on
//! the closest pair first, and each side is consumed when it is used.
//!
//! **Both a linear and a rank correlation are reported, because the two answer different
//! questions.** Pearson asks "is this a straight line", which is the right question when the axes
//! are the same quantity measured twice — a section's pore area against the plug's helium
//! porosity. Spearman asks only "do they move together", which is the right question when they are
//! not: pore BODIES and pore THROATS are different lengths and should never fall on one line, but a
//! rock with bigger bodies had better have bigger throats. Spearman is also invariant to any
//! monotone transform, so it does not change when the pane switches an axis to log — which keeps
//! the number in the table from disagreeing with the picture.
//!
//! **Nothing here converts a unit.** Point data is stored verbatim (the core-extras rule), so a
//! core porosity delivered in percent stays in percent. The result therefore reports the median of
//! each axis: a 0.18 beside an 18.2 is a unit mismatch the user can see at a glance, which beats a
//! guess about which of the two was meant.

use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::distribution;

/// Points sent to the frontend. Beyond this the cloud is decimated evenly — a 2000-well project
/// can pair a million plugs, and an IPC payload that size buys nothing a reader can see. The
/// STATISTICS are always computed on every pair, before any decimation.
const MAX_POINTS: usize = 5000;

/// Default depth tolerance ON A METRE PROJECT. One standard 6-inch core sample, the same default
/// the S-factor calibration uses for the same pairing problem.
///
/// The suffix is load-bearing, and so is `#[cfg(test)]`. This shipped as a bare `pub const` named
/// `DEFAULT_DEPTH_TOL` and compared against depths in whatever unit the project stores, so a FOOT
/// project silently paired at 0.15 ft — 1.8 inches, about a third of a sample — and reported the
/// plugs it then failed to reach as "no plug within the depth tolerance". No live path may reach
/// for a fixed number again: they resolve [`default_depth_tol`], which asks the project. What is
/// left is the value a metre-project TEST states outright, and the compiler now enforces that.
#[cfg(test)]
const DEFAULT_DEPTH_TOL_METRES: f32 = 0.15;

/// The default pairing tolerance in the PROJECT's own unit — 0.15 m or 0.5 ft, both one standard
/// 6-inch sample. See [`crate::units::same_depth_tolerance`] for why the two numbers are not a
/// conversion of each other.
///
/// An undeclared project falls back to metres rather than refusing: this is the value used when
/// the CALLER supplied nothing, `DepthUnit::default()` is documented as metres, and failing a
/// whole QC run over an unset preference would be worse than the behaviour this replaces.
pub fn default_depth_tol(conn: &Connection) -> f32 {
    let unit = crate::units::project_depth_unit(conn).ok().flatten().unwrap_or_default();
    crate::units::same_depth_tolerance(unit) as f32
}

/// psi to pascal, so the Washburn algebra below can be written in the units it is stated in.
const PSI_TO_PA: f64 = 6894.757;

/// The mercury saturation a throat radius is quoted at by default: 35%, the Kolodzie (1980) /
/// Winland **r35** convention already used by `rocktyping.rs`. It is a NAMED convention, not a
/// fitted number, and it is the one that makes this plot directly comparable to the R35 curve the
/// rock-typing module predicts from porosity and permeability.
pub const DEFAULT_HG_SATURATION: f32 = 0.35;

/// Where one axis of the comparison comes from.
///
/// Deliberately flat rather than a tagged enum: the pane builds it from two selects, and a
/// stringly-typed `kind` that fails with a named message beats a serde union that fails with a
/// deserialization error the user cannot act on.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlugSource {
    /// `"core"` (a routine-core-analysis column), `"aux"` (one item of one point dataset) or
    /// `"scal_throat"` (a pore-throat radius read off the plug's own Pc curve).
    pub kind: String,
    /// `aux` only — the point dataset. Its ACTIVE delivery is what gets read.
    #[serde(default)]
    pub dataset: String,
    /// `core`: CPOR / CPERM / CGD / CSW. `aux`: the measurement name.
    #[serde(default)]
    pub item: String,
    /// `scal_throat` only — the non-wetting (mercury) saturation the radius is quoted at.
    #[serde(default)]
    pub saturation: f32,
}

/// One thing the axis pickers can offer, counted over the wells in scope.
#[derive(Debug, Clone, Serialize)]
pub struct PlugChoice {
    pub kind: String,
    pub dataset: String,
    pub item: String,
    pub label: String,
    /// Samples carrying a NUMBER across the scoped wells. A descriptive item cannot be an axis.
    pub n: usize,
    pub wells: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlugQcRequest {
    pub well_ids: Vec<String>,
    pub x: PlugSource,
    pub y: PlugSource,
    /// Two measurements further apart than this are not the same plug.
    #[serde(default)]
    pub depth_tol: f32,
}

/// One paired plug. Both depths ride along: when they differ the user is looking at how well the
/// two deliveries are registered against each other, which is not visible from the values.
#[derive(Debug, Clone, Serialize)]
pub struct PlugQcPoint {
    pub well_id: String,
    pub x: f32,
    pub y: f32,
    pub x_depth: f32,
    pub y_depth: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlugQcResult {
    pub points: Vec<PlugQcPoint>,
    /// Pairs found, which is NOT `points.len()` once the cloud is decimated.
    pub n_pairs: usize,
    pub n_wells: usize,
    /// Straight-line agreement. Meaningful when the two axes are the same quantity.
    pub pearson: f32,
    /// Rank agreement — "do they move together", regardless of the shape or the axis scale.
    pub spearman: f32,
    pub x_label: String,
    pub y_label: String,
    pub x_median: f32,
    pub y_median: f32,
    /// Samples that could not be paired or could not be read, with the reason.
    pub excluded: Vec<(String, usize)>,
    pub notes: Vec<String>,
}

/// Washburn: the throat radius a given capillary pressure has just entered.
///
/// `r = 2·σcosθ / Pc`. The interfacial term arrives in dyn/cm (= mN/m = 1e-3 N/m) as the lab
/// recorded it, Pc in psi, and the answer is wanted in micrometres — the unit factors are written
/// out rather than folded into one constant so the algebra stays checkable. It comes to
/// `0.29008·σcosθ/Pc`, which is the familiar `r(µm) ≈ 107/Pc(psi)` once σcosθ = 367 dyn/cm for the
/// mercury-air system (the same constant `thomeer.rs` standardizes to).
pub fn throat_radius_um(ift_dyn_cm: f64, pc_psi: f64) -> f64 {
    if !(pc_psi > 0.0) || !(ift_dyn_cm > 0.0) {
        return f64::NAN;
    }
    2.0 * (ift_dyn_cm * 1e-3) / (pc_psi * PSI_TO_PA) * 1e6
}

/// One measurement at one depth.
#[derive(Debug, Clone, Copy)]
struct Sample {
    depth: f32,
    value: f32,
}

fn bump(v: &mut Vec<(String, usize)>, why: &str) {
    match v.iter_mut().find(|(w, _)| w == why) {
        Some((_, n)) => *n += 1,
        None => v.push((why.to_string(), 1)),
    }
}

/// The mercury saturation a throat radius will actually be read at — the r35 convention unless
/// the caller named a usable one. Resolved in ONE place so the label can never disagree with the
/// number: a plot captioned "at 0% mercury" would be a lie told by a defaulted field.
fn resolved_saturation(src: &PlugSource) -> f32 {
    if src.saturation.is_finite() && src.saturation > 0.0 && src.saturation < 1.0 {
        src.saturation
    } else {
        DEFAULT_HG_SATURATION
    }
}

/// Human-readable name for an axis.
pub fn label_for(src: &PlugSource) -> String {
    match src.kind.as_str() {
        "core" => format!("{} — core plugs", src.item),
        "aux" => format!("{} — {}", src.item, src.dataset),
        "scal_throat" => format!(
            "Pore-throat radius at {:.0}% mercury (µm)",
            (resolved_saturation(src) * 100.0).round()
        ),
        _ => src.item.clone(),
    }
}

/// Every sample of one source in one well, depth-ordered, with what had to be left out.
fn samples_for(
    conn: &Connection,
    well_id: &str,
    src: &PlugSource,
    excluded: &mut Vec<(String, usize)>,
) -> Result<Vec<Sample>, String> {
    let mut out: Vec<Sample> = Vec::new();
    match src.kind.as_str() {
        "core" => {
            let want = src.item.to_uppercase();
            // NULL cells are already dropped by the reader rather than read as zeros.
            for (item, depth, value) in
                crate::db::get_core_point_series(conn, well_id).map_err(|e| e.to_string())?
            {
                if item.eq_ignore_ascii_case(&want) && depth.is_finite() && value.is_finite() {
                    out.push(Sample { depth, value });
                }
            }
        }
        "aux" => {
            let rows = crate::db::list_aux_data(conn, well_id, Some(&src.dataset))
                .map_err(|e| e.to_string())?;
            for r in rows {
                if !r.item.eq_ignore_ascii_case(&src.item) {
                    continue;
                }
                let Some(v) = r.value_num.filter(|v| v.is_finite()) else {
                    bump(excluded, "sample(s) whose value is text, not a number");
                    continue;
                };
                // An interval sample is anchored at its MIDDLE — the same convention the point
                // tracks draw it at, so the plot and the log view agree about where it is.
                let depth = match r.depth_base.filter(|b| b.is_finite()) {
                    Some(b) => (r.depth_top + b) * 0.5,
                    None => r.depth_top,
                };
                if depth.is_finite() {
                    out.push(Sample { depth, value: v });
                }
            }
        }
        "scal_throat" => out = throat_samples(conn, well_id, resolved_saturation(src), excluded)?,
        other => return Err(format!("unknown measurement source '{other}'")),
    }
    out.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(Ordering::Equal));
    Ok(out)
}

/// One throat radius per SCAL plug, read off that plug's own capillary-pressure curve.
fn throat_samples(
    conn: &Connection,
    well_id: &str,
    saturation: f32,
    excluded: &mut Vec<(String, usize)>,
) -> Result<Vec<Sample>, String> {
    let rows = crate::db::get_scal_pc(conn, well_id).map_err(|e| e.to_string())?;
    // A plug is identified by its sample number where the lab gave one and by its depth otherwise;
    // keying on both means a sample number reused at two depths stays two plugs.
    let mut plugs: Vec<((Option<i32>, u32), Option<f32>, Option<f32>, Vec<(f64, f64)>)> = Vec::new();
    for r in rows {
        let key = (r.sample_no, r.depth.unwrap_or(f32::NAN).to_bits());
        let idx = match plugs.iter().position(|(k, _, _, _)| *k == key) {
            Some(i) => i,
            None => {
                plugs.push((key, r.depth, None, Vec::new()));
                plugs.len() - 1
            }
        };
        if plugs[idx].2.is_none() {
            plugs[idx].2 = r.ift.filter(|v| v.is_finite() && *v > 0.0);
        }
        // `sw` is the WETTING-phase saturation, so the injected mercury is 1 − sw — the same
        // reading `thomeer.rs` takes when it forms the invaded bulk volume.
        let (pc, sw) = (r.pc as f64, r.sw as f64);
        if pc.is_finite() && pc > 0.0 && sw.is_finite() && (0.0..=1.0).contains(&sw) {
            plugs[idx].3.push((pc, 1.0 - sw));
        }
    }

    let target = saturation as f64;
    let mut out = Vec::new();
    for (_, depth, ift, mut pts) in plugs {
        let Some(depth) = depth.filter(|d| d.is_finite()) else {
            bump(excluded, "SCAL plug(s) with no depth to pair on");
            continue;
        };
        let Some(ift) = ift else {
            // Without the lab's σcosθ there is no radius, only a pressure. `thomeer.rs` takes the
            // same line: a plug with no recorded interfacial tension gets no mercury-system answer.
            bump(excluded, "SCAL plug(s) with no recorded interfacial tension");
            continue;
        };
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        if pts.len() < 2 {
            bump(excluded, "SCAL plug(s) with too few Pc points");
            continue;
        }
        let Some(pc) = pc_at_saturation(&pts, target) else {
            // NEVER extrapolated: a curve that stopped at 20% mercury cannot state r35, and a
            // radius invented past the last measured step is the strongest number on the plot.
            bump(excluded, "SCAL plug(s) whose Pc curve never reached that saturation");
            continue;
        };
        let r = throat_radius_um(ift as f64, pc);
        if r.is_finite() && r > 0.0 {
            out.push(Sample { depth, value: r as f32 });
        }
    }
    Ok(out)
}

/// The capillary pressure at a given non-wetting saturation, interpolated **in log Pc**.
///
/// Pc spans decades across one curve, so interpolating it linearly between a 10 psi step and a
/// 1000 psi step would put the answer an order of magnitude out. Returns `None` when the target
/// lies outside the measured saturations — the caller reports that rather than extrapolating.
fn pc_at_saturation(pts: &[(f64, f64)], target: f64) -> Option<f64> {
    for w in pts.windows(2) {
        let ((p0, s0), (p1, s1)) = (w[0], w[1]);
        let (lo, hi) = if s0 <= s1 { (s0, s1) } else { (s1, s0) };
        if target >= lo && target <= hi {
            if (s1 - s0).abs() < 1e-9 {
                return Some(p0);
            }
            let f = (target - s0) / (s1 - s0);
            let l = p0.log10() + f * (p1.log10() - p0.log10());
            return Some(10f64.powf(l));
        }
    }
    None
}

/// Closest-first pairing with each sample used at most once.
///
/// Greedy on the closest pair rather than an optimal assignment: with plugs and the sections cut
/// from them the depths agree to a centimetre or they are not the same plug, so the two agree —
/// and greedy is the version whose behaviour can be stated in one sentence when it does not.
fn pair_samples(xs: &[Sample], ys: &[Sample], tol: f32) -> Vec<(usize, usize)> {
    let mut cands: Vec<(f32, usize, usize)> = Vec::new();
    let mut lo = 0usize;
    for (i, x) in xs.iter().enumerate() {
        while lo < ys.len() && ys[lo].depth < x.depth - tol {
            lo += 1;
        }
        let mut j = lo;
        while j < ys.len() && ys[j].depth <= x.depth + tol {
            cands.push(((x.depth - ys[j].depth).abs(), i, j));
            j += 1;
        }
    }
    cands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal).then(a.1.cmp(&b.1)));
    let mut used_x = vec![false; xs.len()];
    let mut used_y = vec![false; ys.len()];
    let mut out = Vec::new();
    for (_, i, j) in cands {
        if used_x[i] || used_y[j] {
            continue;
        }
        used_x[i] = true;
        used_y[j] = true;
        out.push((i, j));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Mid-ranks, so tied values share a rank instead of taking an arbitrary order.
fn ranks(v: &[f32]) -> Vec<f32> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap_or(Ordering::Equal));
    let mut out = vec![0f32; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let r = (i + j) as f32 / 2.0 + 1.0;
        for k in idx.iter().take(j + 1).skip(i) {
            out[*k] = r;
        }
        i = j + 1;
    }
    out
}

/// Everything the two axis pickers can offer over the wells in scope.
pub fn list_plug_choices(conn: &Connection, well_ids: &[String]) -> Result<Vec<PlugChoice>, String> {
    let mut out: Vec<PlugChoice> = Vec::new();
    let mut push = |kind: &str, dataset: &str, item: &str, n: usize| {
        if n == 0 {
            return;
        }
        match out
            .iter_mut()
            .find(|c| c.kind == kind && c.dataset == dataset && c.item == item)
        {
            Some(c) => {
                c.n += n;
                c.wells += 1;
            }
            None => out.push(PlugChoice {
                kind: kind.into(),
                dataset: dataset.into(),
                item: item.into(),
                label: String::new(),
                n,
                wells: 1,
            }),
        }
    };

    for wid in well_ids {
        let mut counts: Vec<(String, String, String, usize)> = Vec::new();
        let mut tally = |kind: &str, dataset: &str, item: &str| {
            match counts
                .iter_mut()
                .find(|(k, d, i, _)| k == kind && d == dataset && i == item)
            {
                Some((_, _, _, n)) => *n += 1,
                None => counts.push((kind.into(), dataset.into(), item.into(), 1)),
            }
        };
        for (item, _, _) in crate::db::get_core_point_series(conn, wid).map_err(|e| e.to_string())? {
            tally("core", "", &item);
        }
        for r in crate::db::list_aux_data(conn, wid, None).map_err(|e| e.to_string())? {
            // Only what could be an axis. A lithology description is real data and belongs in the
            // point track; it is not a coordinate.
            if r.value_num.map(|v| v.is_finite()) == Some(true) {
                tally("aux", &r.dataset, &r.item);
            }
        }
        for (kind, dataset, item, n) in counts {
            push(&kind, &dataset, &item, n);
        }
        let scal = crate::db::get_scal_pc(conn, wid).map_err(|e| e.to_string())?;
        if !scal.is_empty() {
            push("scal_throat", "", "", scal.len());
        }
    }

    for c in &mut out {
        c.label = match c.kind.as_str() {
            "core" => format!("Core plugs — {} ({} sample(s))", c.item, c.n),
            "aux" => format!("{} — {} ({} sample(s))", c.dataset, c.item, c.n),
            _ => format!("SCAL — pore-throat radius ({} Pc point(s))", c.n),
        };
    }
    out.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.dataset.cmp(&b.dataset)).then(a.item.cmp(&b.item)));
    Ok(out)
}

/// Pairs the two measurements plug by plug across the scoped wells.
pub fn run_plug_qc(conn: &Connection, req: &PlugQcRequest) -> Result<PlugQcResult, String> {
    if req.well_ids.is_empty() {
        return Err("no wells selected".into());
    }
    let tol = if req.depth_tol.is_finite() && req.depth_tol > 0.0 {
        req.depth_tol
    } else {
        default_depth_tol(conn)
    };

    let mut res = PlugQcResult {
        x_label: label_for(&req.x),
        y_label: label_for(&req.y),
        ..Default::default()
    };
    let mut all: Vec<PlugQcPoint> = Vec::new();
    let mut wells_with_pairs = 0usize;
    let mut wells_missing = 0usize;

    for wid in &req.well_ids {
        let xs = samples_for(conn, wid, &req.x, &mut res.excluded)?;
        let ys = samples_for(conn, wid, &req.y, &mut res.excluded)?;
        if xs.is_empty() || ys.is_empty() {
            wells_missing += 1;
            continue;
        }
        let pairs = pair_samples(&xs, &ys, tol);
        if !pairs.is_empty() {
            wells_with_pairs += 1;
        }
        for (i, j) in &pairs {
            all.push(PlugQcPoint {
                well_id: wid.clone(),
                x: xs[*i].value,
                y: ys[*j].value,
                x_depth: xs[*i].depth,
                y_depth: ys[*j].depth,
            });
        }
        let unpaired = xs.len() + ys.len() - 2 * pairs.len();
        for _ in 0..unpaired {
            bump(&mut res.excluded, "sample(s) with no partner within the depth tolerance");
        }
    }

    res.n_pairs = all.len();
    res.n_wells = wells_with_pairs;
    if wells_missing > 0 {
        res.notes.push(format!(
            "{wells_missing} well(s) in scope carry only one of the two measurements."
        ));
    }
    if all.is_empty() {
        res.notes.push(format!(
            "No plug carried both measurements within {tol}. If the two deliveries came from \
             different depth references, register the core first (Data ▸ Tools ▾ ▸ Register Depth…) \
             rather than widening the tolerance."
        ));
        return Ok(res);
    }

    // Statistics on EVERY pair, before the cloud is thinned for the wire.
    let xv: Vec<f32> = all.iter().map(|p| p.x).collect();
    let yv: Vec<f32> = all.iter().map(|p| p.y).collect();
    // `tops::pearson` refuses fewer than four points, and both correlations inherit that floor.
    // A blank cell with no reason reads as a bug; say which it is.
    res.pearson = crate::tops::pearson(&xv, &yv).0;
    res.spearman = crate::tops::pearson(&ranks(&xv), &ranks(&yv)).0;
    if !res.pearson.is_finite() {
        res.notes.push(format!(
            "{} pair(s) — a correlation needs at least four, so both are left blank.",
            all.len()
        ));
    }
    let mut xs_sorted = xv.clone();
    let mut ys_sorted = yv.clone();
    xs_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    ys_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    res.x_median = distribution::percentile(&xs_sorted, 50.0);
    res.y_median = distribution::percentile(&ys_sorted, 50.0);

    if all.len() > MAX_POINTS {
        // Spread evenly rather than the first N — the first N of a depth-ordered list is the top
        // of the well, which is a biased corner of the field, not a sample of it.
        let keep = distribution::even_indices(all.len(), MAX_POINTS);
        res.points = keep.into_iter().map(|i| all[i].clone()).collect();
        res.notes.push(format!(
            "{} pairs found; {MAX_POINTS} drawn, spread evenly. Every number above uses all of them.",
            all.len()
        ));
    } else {
        res.points = all;
    }
    Ok(res)
}

/// One measurement the caller already has in hand, not yet stored.
///
/// Deliberately a bare depth-and-value rather than anything petrographic. The pairing below is
/// about depths; a scoring routine that knew what a pore fraction *was* would have to be written
/// again for the next measurement that wants checking.
#[derive(Debug, Clone, Copy)]
pub struct MeasuredSample {
    pub depth: f32,
    pub value: f32,
}

/// How a run agrees with a measurement this app did not produce.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Agreement {
    pub reference_label: String,
    /// Plugs carrying BOTH. Not the number of plates measured, and the difference is the point:
    /// two runs that refused different plates are scored on different rock and their coefficients
    /// are not directly comparable. Reported beside every number for that reason.
    pub n_pairs: usize,
    /// Measurements on either side that found no partner inside the tolerance.
    pub n_unpaired: usize,
    /// Straight-line agreement — the right question when both axes are porosity.
    pub pearson: f32,
    /// Rank agreement — "does it order the plugs the way the laboratory does". The question to
    /// read when choosing between two settings, because it survives the systematic offset a
    /// section-versus-plug comparison always carries (a point count reads well below helium on a
    /// microporous carbonate without being wrong about which plug is the better rock).
    pub spearman: f32,
    pub measured_median: f32,
    pub reference_median: f32,
    pub notes: Vec<String>,
}

/// Scores measurements the caller is holding against a stored plug measurement, in one well.
///
/// The same pairing rule as [`run_plug_qc`] and literally the same code — closest pair first, each
/// side consumed once, nothing snapped beyond the tolerance. The only difference is that one axis
/// arrives as a slice instead of a database read, which is what lets a run be scored BEFORE it is
/// stored: tuning that had to be saved first would leave a trail of half-judged answers in the
/// project, which is the same reason `set_name` is optional on a pore run.
pub fn score_against_plugs(
    conn: &Connection,
    well_id: &str,
    measured: &[MeasuredSample],
    reference: &PlugSource,
    depth_tol: f32,
) -> Result<Agreement, String> {
    let tol = if depth_tol.is_finite() && depth_tol > 0.0 {
        depth_tol
    } else {
        default_depth_tol(conn)
    };
    let mut res = Agreement {
        reference_label: label_for(reference),
        ..Default::default()
    };

    let mut excluded = Vec::new();
    let refs = samples_for(conn, well_id, reference, &mut excluded)?;
    if refs.is_empty() {
        res.notes.push(format!(
            "This well carries no {}, so there is nothing to check the run against.",
            res.reference_label
        ));
        return Ok(res);
    }

    let mut xs: Vec<Sample> = measured
        .iter()
        .filter(|m| m.depth.is_finite() && m.value.is_finite())
        .map(|m| Sample { depth: m.depth, value: m.value })
        .collect();
    xs.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(Ordering::Equal));

    let pairs = pair_samples(&xs, &refs, tol);
    res.n_pairs = pairs.len();
    res.n_unpaired = xs.len() + refs.len() - 2 * pairs.len();
    if pairs.is_empty() {
        // The tolerance is NOT the thing to widen. A core off by a whole sample interval passes any
        // tolerance check, so a looser one quietly pairs each section with its neighbour's plug and
        // returns a confident number about the wrong rock.
        res.notes.push(format!(
            "No plate found a plug within {tol}. If the sections and the plugs came from different \
             depth references, register the core first (Data ▸ Tools ▾ ▸ Register Depth…) rather \
             than widening the tolerance."
        ));
        return Ok(res);
    }

    let xv: Vec<f32> = pairs.iter().map(|(i, _)| xs[*i].value).collect();
    let yv: Vec<f32> = pairs.iter().map(|(_, j)| refs[*j].value).collect();
    res.pearson = crate::tops::pearson(&xv, &yv).0;
    res.spearman = crate::tops::pearson(&ranks(&xv), &ranks(&yv)).0;
    if !res.pearson.is_finite() {
        res.notes.push(format!(
            "{} plug(s) — an agreement needs at least four, so both are left blank.",
            pairs.len()
        ));
    }
    // Reported for the same reason `run_plug_qc` reports them: point data is stored verbatim, so a
    // 0.12 beside a 24.8 is a fraction-versus-percent delivery visible at a glance. It does NOT
    // move the rank agreement, which is why that is the number to choose a setting on.
    let mut xs_sorted = xv.clone();
    let mut ys_sorted = yv.clone();
    xs_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    ys_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    res.measured_median = distribution::percentile(&xs_sorted, 50.0);
    res.reference_median = distribution::percentile(&ys_sorted, 50.0);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{AuxRow, ScalPcRow};

    fn mem() -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-QC", None, None, None).unwrap();
        (conn, wid.to_string())
    }

    fn core(conn: &Connection, w: &str, depths: &[f32], por: &[f32]) {
        let nan = vec![f32::NAN; depths.len()];
        crate::db::insert_core_data(conn, w, "RAW", None, depths, por, &nan, &nan, &nan).unwrap();
    }

    fn aux(conn: &Connection, w: &str, item: &str, rows: &[(f32, f32)]) {
        let rows: Vec<AuxRow> = rows
            .iter()
            .map(|(d, v)| AuxRow {
                dataset: "PETROGRAPHY".into(),
                depth_top: *d,
                depth_base: None,
                item: item.into(),
                value_num: Some(*v),
                value_text: None,
            })
            .collect();
        crate::db::insert_aux_data(conn, w, "PETROGRAPHY", "TS", None, &rows).unwrap();
    }

    fn src_core(item: &str) -> PlugSource {
        PlugSource { kind: "core".into(), item: item.into(), ..Default::default() }
    }

    fn src_aux(item: &str) -> PlugSource {
        PlugSource {
            kind: "aux".into(),
            dataset: "PETROGRAPHY".into(),
            item: item.into(),
            ..Default::default()
        }
    }

    /// "One standard 6-inch sample" is a physical length, and the tolerance is compared against
    /// depths in the PROJECT's unit — so it shipped as a bare `0.15` that meant six inches on a
    /// metre project and 1.8 inches on a foot one, roughly a third of a sample. A foot project
    /// then reported most of its plugs as having no partner, and the S-factor calibration next
    /// door was fitted on whatever handful survived.
    ///
    /// Both halves are pinned. The foot project must get a real six inches, and the metre
    /// project's number must NOT move — 0.15 m is what shipped and what the documentation says,
    /// and re-deriving it as 0.1524 would change every existing project's pairing for a
    /// millimetre of no petrophysical content.
    #[test]
    fn a_plug_pairing_tolerance_is_one_sample_in_the_projects_own_unit() {
        let pair_at = |unit: crate::units::DepthUnit, gap: f32| -> usize {
            let (conn, w) = mem();
            crate::units::set_project_depth_unit(&conn, unit).unwrap();
            // Plugs 5 apart, so an offset that misses its own plug cannot land on the next one
            // instead — which would answer "did it pair?" with the wrong pairing.
            core(&conn, &w, &[2000.0, 2005.0], &[0.20, 0.24]);
            aux(&conn, &w, "VPORE_TS", &[(2000.0 + gap, 0.19), (2005.0 + gap, 0.25)]);
            run_plug_qc(
                &conn,
                &PlugQcRequest {
                    well_ids: vec![w.clone()],
                    x: src_core("CPOR"),
                    y: src_aux("VPORE_TS"),
                    // Nothing supplied — this is the path that resolves the project's default.
                    depth_tol: 0.0,
                },
            )
            .unwrap()
            .n_pairs
        };

        // A foot project: 0.3 ft is well inside one 6-inch sample and must pair. Under the old
        // bare 0.15 it was outside the tolerance and both sides were dropped.
        assert_eq!(
            pair_at(crate::units::DepthUnit::Feet, 0.3),
            2,
            "0.3 ft is inside one 6-inch sample — a foot project must not pair at 0.15 ft"
        );
        // …and it is still a tolerance, not a licence: 0.9 ft is nearly two samples away.
        assert_eq!(pair_at(crate::units::DepthUnit::Feet, 0.9), 0, "0.9 ft is not the same plug");

        // A metre project is untouched: 0.1 m pairs, 0.2 m does not, exactly as before.
        assert_eq!(pair_at(crate::units::DepthUnit::Metres, 0.1), 2, "0.1 m is inside 0.15 m");
        assert_eq!(
            pair_at(crate::units::DepthUnit::Metres, 0.1524),
            0,
            "0.1524 m is outside the shipped 0.15 m — re-deriving the metre default from six \
             inches would silently move every existing project's pairing"
        );
    }

    /// The whole point of the pane: a plate measurement beside the plug it was cut from.
    #[test]
    fn a_section_pairs_with_the_plug_it_was_cut_from() {
        let (conn, w) = mem();
        core(&conn, &w, &[2000.0, 2001.0, 2002.0, 2003.0, 2004.0, 2005.0], &[
            0.20, 0.24, 0.18, 0.26, 0.15, 0.22,
        ]);
        aux(&conn, &w, "VPORE_TS", &[
            (2000.02, 0.19),
            (2001.0, 0.25),
            (2002.05, 0.17),
            (2003.01, 0.25),
            (2004.0, 0.14),
            (2005.03, 0.21),
        ]);

        let res = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_core("CPOR"),
                y: src_aux("VPORE_TS"),
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();
        assert_eq!(res.n_pairs, 6, "every plate found its plug");
        assert_eq!(res.n_wells, 1);
        assert!(res.excluded.is_empty(), "nothing dropped: {:?}", res.excluded);
        assert!(res.pearson > 0.9, "the two porosities track: r = {}", res.pearson);
        // The medians are reported so a percent-vs-fraction delivery is visible at a glance.
        assert!((res.x_median - 0.21).abs() < 1e-5, "median CPOR = {}", res.x_median);
    }

    /// A plate further than the tolerance from any plug is DROPPED and COUNTED, never snapped to
    /// the nearest one — snapping is how a core that needs registering gets silently accepted.
    #[test]
    fn a_sample_with_no_partner_is_dropped_and_counted() {
        let (conn, w) = mem();
        core(&conn, &w, &[2000.0, 2001.0], &[0.20, 0.24]);
        aux(&conn, &w, "VPORE_TS", &[(2000.0, 0.19), (2050.0, 0.31)]);

        let res = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_core("CPOR"),
                y: src_aux("VPORE_TS"),
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();
        assert_eq!(res.n_pairs, 1);
        let dropped: usize = res.excluded.iter().map(|(_, n)| *n).sum();
        assert_eq!(dropped, 2, "the far plate and the plug it left behind: {:?}", res.excluded);
        assert!(res.excluded.iter().any(|(w, _)| w.contains("no partner")));
    }

    /// Two sections cut close together must not both claim the one plug nearest them: that single
    /// core porosity would then appear twice in the cloud and tighten the correlation for free.
    #[test]
    fn one_plug_cannot_be_claimed_by_two_sections() {
        let (conn, w) = mem();
        core(&conn, &w, &[2000.0], &[0.20]);
        aux(&conn, &w, "VPORE_TS", &[(2000.01, 0.19), (2000.03, 0.21)]);

        let res = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_core("CPOR"),
                y: src_aux("VPORE_TS"),
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();
        assert_eq!(res.n_pairs, 1, "the plug is used once");
        // …and the closer of the two sections is the one that got it.
        assert!((res.points[0].y - 0.19).abs() < 1e-6, "the nearest section won the pairing");
        assert_eq!(res.excluded.iter().map(|(_, n)| *n).sum::<usize>(), 1);
    }

    /// Pore bodies against pore throats are different lengths on purpose. Pearson asks whether
    /// they fall on a straight line, which they should not; Spearman asks whether they move
    /// together, which they should — and a rank measure does not change when an axis goes log.
    #[test]
    fn a_curved_but_monotone_relation_reads_as_rank_agreement_not_a_straight_line() {
        let (conn, w) = mem();
        let depths: Vec<f32> = (0..12).map(|i| 2000.0 + i as f32).collect();
        // y = x^4: strictly increasing, badly non-linear.
        let x: Vec<f32> = (1..=12).map(|i| i as f32).collect();
        core(&conn, &w, &depths, &x);
        let pts: Vec<(f32, f32)> =
            depths.iter().zip(&x).map(|(d, v)| (*d, v.powi(4))).collect();
        aux(&conn, &w, "PORE_D50", &pts);

        let res = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_core("CPOR"),
                y: src_aux("PORE_D50"),
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();
        assert_eq!(res.n_pairs, 12);
        assert!((res.spearman - 1.0).abs() < 1e-4, "perfect rank agreement: {}", res.spearman);
        assert!(res.pearson < 0.95, "and NOT a straight line: r = {}", res.pearson);
    }

    /// Washburn against the number every MICP report quotes: mercury-air (σcosθ = 367 dyn/cm) puts
    /// r ≈ 1.07 µm at 100 psi, the familiar r(µm) ≈ 107/Pc(psi).
    #[test]
    fn the_throat_radius_matches_the_textbook_mercury_relation() {
        let r = throat_radius_um(367.0, 100.0);
        assert!((r - 1.0651).abs() < 1e-3, "r = {r}");
        // And it scales as 1/Pc, which is the whole content of the relation.
        assert!((throat_radius_um(367.0, 1000.0) - r / 10.0).abs() < 1e-6);
    }

    /// Pc runs over decades, so the interpolation is in LOG Pc. Halfway in saturation between
    /// 10 psi and 1000 psi is 100 psi, not 505.
    #[test]
    fn the_pc_interpolation_is_logarithmic_and_never_extrapolates() {
        let pts = [(10.0, 0.2), (1000.0, 0.6)];
        let mid = pc_at_saturation(&pts, 0.4).unwrap();
        assert!((mid - 100.0).abs() < 1e-6, "log-interpolated Pc = {mid}");
        // A curve that stopped at 60% mercury cannot state a radius at 70%.
        assert!(pc_at_saturation(&pts, 0.7).is_none());
        assert!(pc_at_saturation(&pts, 0.1).is_none());
    }

    /// A plug whose Pc report carries no interfacial tension has a pressure but no radius, and is
    /// reported as such rather than converted with an assumed mercury system.
    #[test]
    fn a_scal_plug_without_an_interfacial_tension_is_excluded_by_name() {
        let (conn, w) = mem();
        let mk = |sample: i32, depth: f32, ift: Option<f32>| -> Vec<ScalPcRow> {
            [(10.0f32, 0.9f32), (100.0, 0.5), (1000.0, 0.2)]
                .iter()
                .map(|(pc, sw)| ScalPcRow {
                    sample_no: Some(sample),
                    depth: Some(depth),
                    perm: f32::NAN,
                    poro: f32::NAN,
                    pc: *pc,
                    sw: *sw,
                    system: Some("hg_air".into()),
                    ift,
                })
                .collect()
        };
        let mut rows = mk(1, 2000.0, Some(367.0));
        rows.extend(mk(2, 2001.0, None));
        crate::db::insert_scal_pc(&conn, &w, "RAW", None, &rows).unwrap();
        core(&conn, &w, &[2000.0, 2001.0], &[0.20, 0.24]);

        let res = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_core("CPOR"),
                y: PlugSource {
                    kind: "scal_throat".into(),
                    saturation: DEFAULT_HG_SATURATION,
                    ..Default::default()
                },
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();
        assert_eq!(res.n_pairs, 1, "only the plug with a recorded ift yields a radius");
        assert!(
            res.excluded.iter().any(|(w, _)| w.contains("interfacial tension")),
            "the reason is named: {:?}",
            res.excluded
        );
        assert!(res.y_label.contains("35% mercury"), "{}", res.y_label);
        assert!(res.points[0].y > 0.0 && res.points[0].y < 100.0, "µm, not psi: {}", res.points[0].y);
    }

    /// Scoring a run the user has NOT saved must give the same answer as saving it and plotting it.
    ///
    /// This is the whole reason `score_against_plugs` sits in this module instead of in the
    /// petrography dialog. Two pairing implementations would drift, and the drift would be silent —
    /// both return a plausible correlation, and nothing on screen says which rule produced it. The
    /// test is the proof that there is one rule: identical values, one read from the project and
    /// one held in hand, must come out identical to the last decimal.
    #[test]
    fn scoring_a_run_in_hand_matches_scoring_it_after_it_is_saved() {
        let (conn, w) = mem();
        let depths: [f32; 8] = [2000.0, 2001.0, 2002.0, 2003.0, 2004.0, 2005.0, 2006.0, 2007.0];
        let por = [0.20, 0.24, 0.18, 0.26, 0.15, 0.22, 0.29, 0.11];
        core(&conn, &w, &depths, &por);
        // What the plates measured — near the plugs, and not on top of them.
        let plate_depths: [f32; 8] =
            [2000.02, 2001.0, 2002.05, 2003.01, 2004.0, 2005.03, 2006.02, 2007.01];
        let plate_values = [0.19, 0.25, 0.17, 0.25, 0.14, 0.21, 0.27, 0.12];

        let stored: Vec<(f32, f32)> =
            plate_depths.iter().zip(&plate_values).map(|(d, v)| (*d, *v)).collect();
        aux(&conn, &w, "VPORE_TS", &stored);
        let saved = run_plug_qc(
            &conn,
            &PlugQcRequest {
                well_ids: vec![w.clone()],
                x: src_aux("VPORE_TS"),
                y: src_core("CPOR"),
                depth_tol: DEFAULT_DEPTH_TOL_METRES,
            },
        )
        .unwrap();

        let in_hand: Vec<MeasuredSample> = plate_depths
            .iter()
            .zip(&plate_values)
            .map(|(d, v)| MeasuredSample { depth: *d, value: *v })
            .collect();
        let live = score_against_plugs(&conn, &w, &in_hand, &src_core("CPOR"), DEFAULT_DEPTH_TOL_METRES)
            .unwrap();

        assert_eq!(live.n_pairs, saved.n_pairs, "same pairing");
        assert_eq!(live.n_pairs, 8);
        assert!((live.pearson - saved.pearson).abs() < 1e-6, "{} vs {}", live.pearson, saved.pearson);
        assert!((live.spearman - saved.spearman).abs() < 1e-6);
        assert!((live.reference_median - saved.y_median).abs() < 1e-6);
    }

    /// A plate with no plug inside the tolerance is DROPPED and COUNTED, never snapped to the
    /// nearest one — the rule the rest of this module already follows, inherited rather than
    /// rewritten. Snapping is how a core that needs registering gets silently accepted, and here it
    /// would be worse than usual: the number is being used to decide whether a setting is good, so
    /// a pairing that is wrong for every plate would still look like a verdict.
    #[test]
    fn a_plate_that_found_no_plug_is_counted_not_snapped() {
        let (conn, w) = mem();
        core(&conn, &w, &[2000.0, 2001.0], &[0.20, 0.24]);
        let in_hand = [
            MeasuredSample { depth: 2000.0, value: 0.19 },
            MeasuredSample { depth: 2400.0, value: 0.31 },
        ];
        let res =
            score_against_plugs(&conn, &w, &in_hand, &src_core("CPOR"), DEFAULT_DEPTH_TOL_METRES).unwrap();
        assert_eq!(res.n_pairs, 1);
        assert_eq!(res.n_unpaired, 2, "the stranded plate and the plug it left behind");
        // Four pairs are the floor both correlations inherit, and a blank with no reason reads as
        // a bug rather than as "not enough plugs".
        assert!(!res.pearson.is_finite());
        assert!(res.notes.iter().any(|n| n.contains("at least four")), "{:?}", res.notes);
    }

    /// The number a setting is chosen on has to survive the offset this comparison always carries.
    ///
    /// A section's pore area and a plug's helium porosity are not the same measurement: helium
    /// fills micropores an optical section cannot resolve, so on a carbonate it reads far higher.
    /// A delivery stored as a percent instead of a fraction does the same thing again, a hundred
    /// times over. Neither says the tool ordered the plugs wrongly — which is why the RANK
    /// agreement is the one the dialog leads with, and why the two medians are reported: they are
    /// what makes a unit mismatch visible instead of mysterious.
    #[test]
    fn a_scale_difference_moves_the_medians_and_not_the_rank_agreement() {
        let (conn, w) = mem();
        let depths: Vec<f32> = (0..8).map(|i| 2000.0 + i as f32).collect();
        let por = [0.20, 0.24, 0.18, 0.26, 0.15, 0.22, 0.29, 0.11];
        core(&conn, &w, &depths, &por);

        let mk = |scale: f32| -> Vec<MeasuredSample> {
            depths
                .iter()
                .zip(&por)
                // Same ordering, systematically lower, and curved — a section under-reads most on
                // the most microporous rock.
                .map(|(d, v)| MeasuredSample { depth: *d, value: v.powf(1.4) * scale })
                .collect()
        };
        let a = score_against_plugs(&conn, &w, &mk(1.0), &src_core("CPOR"), DEFAULT_DEPTH_TOL_METRES)
            .unwrap();
        let b = score_against_plugs(&conn, &w, &mk(100.0), &src_core("CPOR"), DEFAULT_DEPTH_TOL_METRES)
            .unwrap();

        assert!((a.spearman - 1.0).abs() < 1e-4, "the ordering is perfect: {}", a.spearman);
        assert!((a.spearman - b.spearman).abs() < 1e-6, "and a unit change does not touch it");
        assert!(a.pearson < 0.999, "while the straight-line fit does feel the curvature");
        // The medians are what show the user which of the two they are looking at.
        assert!(b.measured_median > 50.0 * a.measured_median);
        assert!((a.reference_median - b.reference_median).abs() < 1e-6);
    }

    /// A well with no core carries no verdict, and says so instead of returning a zero. A 0.00
    /// agreement would read as "this setting is useless" rather than "nothing was compared".
    #[test]
    fn a_well_with_nothing_to_check_against_says_so() {
        let (conn, w) = mem();
        let in_hand = [MeasuredSample { depth: 2000.0, value: 0.2 }];
        let res =
            score_against_plugs(&conn, &w, &in_hand, &src_core("CPOR"), DEFAULT_DEPTH_TOL_METRES).unwrap();
        assert_eq!(res.n_pairs, 0);
        assert!(res.notes.iter().any(|n| n.contains("nothing to check")), "{:?}", res.notes);
    }

    /// Tied values share a mid-rank, so a run of equal readings cannot invent an ordering that a
    /// rank correlation would then reward.
    #[test]
    fn tied_values_share_a_rank() {
        let r = ranks(&[3.0, 1.0, 1.0, 5.0]);
        assert_eq!(r, vec![3.0, 1.5, 1.5, 4.0]);
    }
}
