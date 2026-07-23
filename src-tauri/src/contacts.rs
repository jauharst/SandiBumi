//! Assisted fluid-contact picking. Suggests a contact depth from logs — the Sw=0.5
//! crossover, the deep-resistivity drop (hydrocarbon → water), and the density-neutron gas
//! separation closing (gas-down-to) — each with a confidence; and checks cross-well
//! consistency (a contact is flat in TVDSS, so a least-squares plane through the picked
//! points should have small residuals — wells that disagree are flagged). Read-only: the
//! user accepts/edits the suggestion; nothing is auto-committed.

use crate::db;
use crate::equations::fetch_curve_frame;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Suggestion from logs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ContactSuggestRequest {
    pub well_id: String,
    pub zone_top: f32,
    pub zone_base: f32,
    #[serde(default)]
    pub sw_curve: Option<String>,
    #[serde(default)]
    pub res_curve: Option<String>,
    #[serde(default)]
    pub nphi_curve: Option<String>,
    #[serde(default)]
    pub rhob_curve: Option<String>,
    /// Water-saturation cutoff for the crossover (default 0.5).
    #[serde(default)]
    pub sw_cutoff: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactCandidate {
    /// Suggested type (OWC / GDT) — a hint; the user edits it.
    pub contact_type: String,
    pub depth: f32,
    pub method: String,
    /// 0..1 — how trustworthy the indicator looks (contrast / drop magnitude).
    pub confidence: f32,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactSuggestResult {
    pub candidates: Vec<ContactCandidate>,
    pub error: Option<String>,
}

/// Depths where `v` rises through `cutoff` with increasing depth (finite samples only),
/// each with a contrast confidence. For Sw this is the hydrocarbon→water crossover.
fn upward_crossovers(depth: &[f32], v: &[f32], cutoff: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    for i in 1..depth.len() {
        let (s0, s1) = (v[i - 1], v[i]);
        let (d0, d1) = (depth[i - 1], depth[i]);
        if !s0.is_finite() || !s1.is_finite() || !(d1 > d0) {
            continue;
        }
        if s0 < cutoff && s1 >= cutoff {
            let t = (cutoff - s0) / (s1 - s0);
            let dc = d0 + t * (d1 - d0);
            out.push((dc, window_contrast(depth, v, dc, 5.0)));
        }
    }
    // Collapse near-duplicate crossings from noisy Sw (dithering across the cutoff) — keep the
    // highest-contrast pick in each ~2 m cluster so the candidate list isn't flooded.
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut merged: Vec<(f32, f32)> = Vec::new();
    for (d, c) in out {
        match merged.last_mut() {
            Some(last) if (d - last.0).abs() < 2.0 => {
                if c > last.1 {
                    *last = (d, c);
                }
            }
            _ => merged.push((d, c)),
        }
    }
    merged
}

/// mean(below) − mean(above) of finite `v` within ±`win` of `dc`, clamped to [0,1]. A clean
/// contact (low Sw above, high below) scores near its true contrast; a noisy one scores low.
fn window_contrast(depth: &[f32], v: &[f32], dc: f32, win: f32) -> f32 {
    let (mut sa, mut na, mut sb, mut nb) = (0.0f64, 0u32, 0.0f64, 0u32);
    for (&d, &x) in depth.iter().zip(v) {
        if !x.is_finite() {
            continue;
        }
        if d >= dc - win && d < dc {
            sa += x as f64;
            na += 1;
        } else if d > dc && d <= dc + win {
            sb += x as f64;
            nb += 1;
        }
    }
    if na == 0 || nb == 0 {
        return 0.3;
    }
    ((sb / nb as f64 - sa / na as f64).clamp(0.0, 1.0)) as f32
}

/// Depth of the steepest DOWNWARD step in log10(resistivity) over ~`win` metres — the classic
/// hydrocarbon→water drop. Confidence scales with the decades dropped (≥1.5 decades ⇒ 1.0).
fn resistivity_drop(depth: &[f32], rt: &[f32], win: f32) -> Option<(f32, f32)> {
    let mut best: Option<(usize, usize, f32)> = None; // (i, j, decades)
    for i in 0..depth.len() {
        if !rt[i].is_finite() || rt[i] <= 0.0 {
            continue;
        }
        let target = depth[i] + win;
        let mut j = i;
        while j + 1 < depth.len() && depth[j] < target {
            j += 1;
        }
        if j <= i || !rt[j].is_finite() || rt[j] <= 0.0 {
            continue;
        }
        let drop = (rt[i] as f64).log10() - (rt[j] as f64).log10(); // >0 when Rt falls with depth
        if drop > 0.0 && best.map_or(true, |(_, _, b)| drop as f32 > b) {
            best = Some((i, j, drop as f32));
        }
    }
    let (i, j, decades) = best?;
    // Refine to the midpoint of the sharpest single-sample step inside the winning window, so a
    // sharp contact is reported at its true depth rather than ~win/2 shallow (the window midpoint).
    let (mut bi, mut bd) = (i, f64::MIN);
    for k in i..j {
        if rt[k].is_finite() && rt[k] > 0.0 && rt[k + 1].is_finite() && rt[k + 1] > 0.0 {
            let dd = (rt[k] as f64).log10() - (rt[k + 1] as f64).log10();
            if dd > bd {
                bd = dd;
                bi = k;
            }
        }
    }
    Some(((depth[bi] + depth[bi + 1]) / 2.0, (decades / 1.5).clamp(0.0, 1.0)))
}

/// Gas-down-to from the density-neutron gas crossover closing: in gas, neutron porosity reads
/// well below density porosity (φN − φD ≪ 0); at the gas base the separation returns toward 0.
/// Returns the depth where φN − φD rises through −0.03, and a confidence from the gas excursion.
fn dn_gas_base(depth: &[f32], nphi: &[f32], rhob: &[f32], rho_ma: f32, rho_fl: f32) -> Option<(f32, f32)> {
    // Decide the neutron unit ONCE for the whole curve (PU vs fraction) rather than per sample,
    // so a stray low/high reading isn't misconverted: if the curve's peak exceeds 1.5 it's in PU.
    let pu = nphi.iter().filter(|x| x.is_finite()).cloned().fold(f32::MIN, f32::max) > 1.5;
    let sep: Vec<f32> = nphi
        .iter()
        .zip(rhob)
        .map(|(&n, &r)| {
            if !n.is_finite() || !r.is_finite() {
                f32::NAN
            } else {
                let phi_n = if pu { n / 100.0 } else { n };
                let phi_d = (rho_ma - r) / (rho_ma - rho_fl);
                phi_n - phi_d
            }
        })
        .collect();
    let thr = -0.03f32;
    let (mut min_i, mut min_v) = (None, 0.0f32);
    for (i, &s) in sep.iter().enumerate() {
        if s.is_finite() && s < min_v {
            min_v = s;
            min_i = Some(i);
        }
    }
    let mi = min_i?;
    if min_v > thr {
        return None; // no meaningful gas separation
    }
    for i in (mi + 1)..depth.len() {
        let (s0, s1) = (sep[i - 1], sep[i]);
        if !s0.is_finite() || !s1.is_finite() {
            continue;
        }
        if s0 < thr && s1 >= thr {
            let t = (thr - s0) / (s1 - s0);
            let dc = depth[i - 1] + t * (depth[i] - depth[i - 1]);
            return Some((dc, ((-min_v) / 0.15).clamp(0.0, 1.0)));
        }
    }
    None
}

fn names(user: &Option<String>, defaults: &[&str]) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(u) = user {
        let u = u.trim().to_uppercase();
        if !u.is_empty() {
            v.push(u);
        }
    }
    for d in defaults {
        let d = d.to_uppercase();
        if !v.contains(&d) {
            v.push(d);
        }
    }
    v
}

/// Suggests contact depths for one well within a depth zone. Uses whichever indicators are
/// present (Sw, deep resistivity, density+neutron); returns them sorted by confidence.
pub fn suggest_contacts(conn: &Connection, req: &ContactSuggestRequest) -> ContactSuggestResult {
    let fail = |m: String| ContactSuggestResult { candidates: Vec::new(), error: Some(m) };
    if !(req.zone_base > req.zone_top) {
        return fail("zone base must be below zone top".into());
    }
    let cutoff = req.sw_cutoff.unwrap_or(0.5);

    let sw_names = names(&req.sw_curve, &["SW", "SWT", "SWE", "SW_ARCH", "SWARCH"]);
    let res_names = names(&req.res_curve, &["RES_DEEP", "RT", "ILD", "LLD", "RDEEP"]);
    let nphi_names = names(&req.nphi_curve, &["NPHI", "TNPH", "NPOR"]);
    let rhob_names = names(&req.rhob_curve, &["RHOB", "RHOZ", "DEN"]);

    let all: Vec<String> = sw_names
        .iter()
        .chain(&res_names)
        .chain(&nphi_names)
        .chain(&rhob_names)
        .cloned()
        .collect();
    let (depth_all, curves) = match fetch_curve_frame(conn, &req.well_id, &all) {
        Ok(f) => f,
        Err(e) => return fail(e.to_string()),
    };
    let idx: Vec<usize> = (0..depth_all.len())
        .filter(|&i| depth_all[i] >= req.zone_top && depth_all[i] <= req.zone_base)
        .collect();
    if idx.len() < 4 {
        return fail("not enough samples in the zone".into());
    }
    let depth: Vec<f32> = idx.iter().map(|&i| depth_all[i]).collect();
    let pick = |cands: &[String]| -> Option<Vec<f32>> {
        for n in cands {
            if let Some(v) = curves.get(n) {
                if v.len() == depth_all.len() && v.iter().any(|x| x.is_finite()) {
                    return Some(idx.iter().map(|&i| v[i]).collect());
                }
            }
        }
        None
    };

    let mut cands = Vec::new();
    if let Some(sw) = pick(&sw_names) {
        for (d, c) in upward_crossovers(&depth, &sw, cutoff) {
            cands.push(ContactCandidate {
                contact_type: "OWC".into(),
                depth: d,
                method: format!("Sw={:.2} crossover", cutoff),
                confidence: c,
                detail: "water saturation rises through the cutoff".into(),
            });
        }
    }
    if let Some(rt) = pick(&res_names) {
        if let Some((d, c)) = resistivity_drop(&depth, &rt, 3.0) {
            cands.push(ContactCandidate {
                contact_type: "OWC".into(),
                depth: d,
                method: "resistivity drop".into(),
                confidence: c,
                detail: "deep resistivity falls (hydrocarbon → water)".into(),
            });
        }
    }
    if let (Some(nphi), Some(rhob)) = (pick(&nphi_names), pick(&rhob_names)) {
        if let Some((d, c)) = dn_gas_base(&depth, &nphi, &rhob, 2.65, 1.0) {
            cands.push(ContactCandidate {
                contact_type: "GDT".into(),
                depth: d,
                method: "density-neutron".into(),
                confidence: c,
                detail: "neutron-density gas crossover closes (gas base)".into(),
            });
        }
    }
    if cands.is_empty() {
        return fail("no Sw / resistivity / density-neutron contact indicators in the zone".into());
    }
    cands.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    ContactSuggestResult { candidates: cands, error: None }
}

// ---------------------------------------------------------------------------
// Cross-well consistency (a contact is flat in TVDSS)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct WellResidual {
    pub well_id: String,
    pub well_name: String,
    pub tvdss: f32,
    pub predicted: f32,
    pub residual: f32,
    pub flagged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactConsistency {
    pub contact_type: String,
    pub n: usize,
    pub mean_tvdss: f32,
    pub rms: f32,
    /// z = a + b·x + c·y (a dipping surface) when ≥3 wells have coordinates; else flat mean.
    pub plane: Option<[f64; 3]>,
    pub wells: Vec<WellResidual>,
    pub error: Option<String>,
}

/// Least-squares plane z = a + b·x + c·y through (x,y,z) points, fitted on centred coordinates
/// for conditioning. None if < 3 points or the X-Y spread is degenerate (collinear/coincident).
fn fit_plane(pts: &[(f64, f64, f64)]) -> Option<[f64; 3]> {
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let (mx, my, mz) = (
        pts.iter().map(|p| p.0).sum::<f64>() / n,
        pts.iter().map(|p| p.1).sum::<f64>() / n,
        pts.iter().map(|p| p.2).sum::<f64>() / n,
    );
    let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for &(x, y, z) in pts {
        let (dx, dy, dz) = (x - mx, y - my, z - mz);
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
        sxz += dx * dz;
        syz += dy * dz;
    }
    let det = sxx * syy - sxy * sxy;
    if det.abs() < 1e-6 * (sxx * syy).max(1.0) {
        return None;
    }
    let b = (sxz * syy - syz * sxy) / det;
    let c = (syz * sxx - sxz * sxy) / det;
    let a = mz - b * mx - c * my;
    Some([a, b, c])
}

struct ConsistencyCore {
    plane: Option<[f64; 3]>,
    mean: f32,
    rms: f32,
    predicted: Vec<f32>,
    residuals: Vec<f32>,
    flagged: Vec<bool>,
}

/// Fits a flat-TVDSS surface through the picks and scores each. `flag_abs` is the absolute
/// residual (metres) above which a well is flagged as disagreeing with the group.
fn compute_consistency(pts: &[(Option<f64>, Option<f64>, f32)], flag_abs: f32) -> ConsistencyCore {
    let n = pts.len().max(1);
    let mean = pts.iter().map(|p| p.2).sum::<f32>() / n as f32;
    let coord: Vec<(f64, f64, f64)> = pts
        .iter()
        .filter_map(|p| match (p.0, p.1) {
            (Some(x), Some(y)) => Some((x, y, p.2 as f64)),
            _ => None,
        })
        .collect();
    let plane = if coord.len() >= 3 { fit_plane(&coord) } else { None };
    // ONE consistency baseline. With a dip plane, score ONLY wells that have coordinates
    // (a coordless well can't be placed on a dipping surface, so it is left unscored — NaN —
    // rather than judged against the flat mean, which would false-flag on-trend wells). Without
    // a plane, everyone is scored against the flat mean. RMS is over the scored points only, so
    // it is a coherent statistic and never blends two baselines.
    let predicted: Vec<f32> = pts
        .iter()
        .map(|p| match (plane, p.0, p.1) {
            (Some([a, b, c]), Some(x), Some(y)) => (a + b * x + c * y) as f32,
            (Some(_), _, _) => f32::NAN, // plane in use but this well has no coordinates
            (None, _, _) => mean,
        })
        .collect();
    let residuals: Vec<f32> = pts
        .iter()
        .zip(&predicted)
        .map(|(p, &pr)| if pr.is_finite() { p.2 - pr } else { f32::NAN })
        .collect();
    let scored: Vec<f64> = residuals.iter().filter(|r| r.is_finite()).map(|r| (r * r) as f64).collect();
    let rms = if scored.is_empty() {
        f32::NAN
    } else {
        (scored.iter().sum::<f64>() / scored.len() as f64).sqrt() as f32
    };
    let flagged: Vec<bool> = residuals.iter().map(|r| r.is_finite() && r.abs() > flag_abs).collect();
    ConsistencyCore { plane, mean, rms, predicted, residuals, flagged }
}

fn interp_asc(xs: &[f32], ys: &[f32], x: f32) -> Option<f32> {
    if xs.len() < 2 || x < xs[0] || x > xs[xs.len() - 1] {
        return None;
    }
    let idx = xs.partition_point(|&v| v < x);
    if idx == 0 {
        return Some(ys[0]);
    }
    let (x0, x1) = (xs[idx - 1], xs[idx]);
    let (y0, y1) = (ys[idx - 1], ys[idx]);
    if x1 <= x0 {
        return Some(y0);
    }
    Some(y0 + (y1 - y0) * (x - x0) / (x1 - x0))
}

fn md_to_tvdss(conn: &Connection, well_id: &str, md: f32) -> Option<f32> {
    let path = db::get_well_path(conn, well_id).ok()?;
    let mds: Vec<f32> = path.iter().map(|s| s.md).collect();
    let ss: Vec<f32> = path.iter().map(|s| s.tvdss).collect();
    interp_asc(&mds, &ss, md).filter(|v| v.is_finite())
}

/// Checks whether every well-scoped contact of `contact_type` agrees on a flat TVDSS surface.
/// MD contacts are converted to TVDSS via each well's deviation survey; a dipping plane is
/// fitted when ≥3 wells have coordinates, otherwise the flat mean is used.
pub fn check_contact_consistency(conn: &Connection, contact_type: &str, flag_abs: f32) -> ContactConsistency {
    let none = |msg: &str| ContactConsistency {
        contact_type: contact_type.to_string(),
        n: 0,
        mean_tvdss: f32::NAN,
        rms: f32::NAN,
        plane: None,
        wells: Vec::new(),
        error: Some(msg.to_string()),
    };
    let contacts = match db::list_fluid_contacts(conn) {
        Ok(c) => c,
        Err(e) => return none(&e.to_string()),
    };
    let wells = db::list_wells(conn).unwrap_or_default();
    let wmap: HashMap<String, &db::WellSummary> = wells.iter().map(|w| (w.well_id.clone(), w)).collect();

    let mut meta: Vec<(String, String, Option<f64>, Option<f64>, f32)> = Vec::new();
    for c in contacts
        .iter()
        .filter(|c| c.contact_type.eq_ignore_ascii_case(contact_type) && c.well_id.is_some())
    {
        let wid = c.well_id.clone().unwrap();
        let w = match wmap.get(&wid) {
            Some(w) => w,
            None => continue,
        };
        let tvdss = if c.is_tvdss {
            c.depth as f32
        } else {
            match md_to_tvdss(conn, &wid, c.depth as f32) {
                Some(t) => t,
                None => continue, // no survey to convert MD → TVDSS; skip rather than mislead
            }
        };
        meta.push((wid, w.well_name.clone(), w.surface_x, w.surface_y, tvdss));
    }
    if meta.len() < 2 {
        return none("need at least two well-scoped contacts of this type (MD contacts need a deviation survey)");
    }

    let pts: Vec<(Option<f64>, Option<f64>, f32)> = meta.iter().map(|m| (m.2, m.3, m.4)).collect();
    let core = compute_consistency(&pts, flag_abs);
    let out = meta
        .iter()
        .enumerate()
        .map(|(i, m)| WellResidual {
            well_id: m.0.clone(),
            well_name: m.1.clone(),
            tvdss: m.4,
            predicted: core.predicted[i],
            residual: core.residuals[i],
            flagged: core.flagged[i],
        })
        .collect();
    ContactConsistency {
        contact_type: contact_type.to_string(),
        n: meta.len(),
        mean_tvdss: core.mean,
        rms: core.rms,
        plane: core.plane,
        wells: out,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sw_crossover_finds_known_depth() {
        // Sw ~0.2 in the hydrocarbon leg, ~0.9 below the contact at 2050.
        let depth: Vec<f32> = (0..101).map(|i| 2000.0 + i as f32).collect();
        let sw: Vec<f32> = depth.iter().map(|&d| if d < 2050.0 { 0.2 } else { 0.9 }).collect();
        let xs = upward_crossovers(&depth, &sw, 0.5);
        assert_eq!(xs.len(), 1, "expected one crossover, got {xs:?}");
        let (d, c) = xs[0];
        assert!((d - 2050.0).abs() <= 1.0, "crossover at {d}, expected ~2050");
        assert!(c > 0.6, "confidence too low: {c}");
    }

    #[test]
    fn resistivity_drop_finds_contact() {
        // Rt 50 ohm-m in pay, 2 ohm-m in water, stepping at 2050.
        let depth: Vec<f32> = (0..101).map(|i| 2000.0 + i as f32).collect();
        let rt: Vec<f32> = depth.iter().map(|&d| if d < 2050.0 { 50.0 } else { 2.0 }).collect();
        let (d, c) = resistivity_drop(&depth, &rt, 3.0).expect("a drop");
        assert!((d - 2050.0).abs() <= 3.0, "drop at {d}, expected ~2050");
        assert!(c > 0.8, "≈1.4-decade drop should score high, got {c}");
    }

    #[test]
    fn dn_gas_base_finds_gas_contact() {
        // Gas leg: strong neutron-density crossover (φN ≪ φD) above 2040, closing below.
        let depth: Vec<f32> = (0..101).map(|i| 2000.0 + i as f32).collect();
        let nphi: Vec<f32> = depth.iter().map(|&d| if d < 2040.0 { 0.05 } else { 0.22 }).collect();
        let rhob: Vec<f32> = vec![2.30; depth.len()]; // φD ≈ 0.212 throughout
        let (d, _c) = dn_gas_base(&depth, &nphi, &rhob, 2.65, 1.0).expect("gas base");
        assert!((d - 2040.0).abs() <= 2.0, "gas base at {d}, expected ~2040");
    }

    #[test]
    fn fit_plane_recovers_known_plane() {
        // z = 1000 + 0.01 x + 0.02 y on UTM-scale coordinates.
        let pts: Vec<(f64, f64, f64)> = [(500000.0, 9900000.0), (500500.0, 9900200.0), (500200.0, 9900600.0), (500800.0, 9900100.0)]
            .iter()
            .map(|&(x, y)| (x, y, 1000.0 + 0.01 * (x - 500000.0) + 0.02 * (y - 9900000.0)))
            .collect();
        let [a, b, c] = fit_plane(&pts).expect("plane");
        // predict at a fresh point and compare to the truth
        let (x, y) = (500400.0, 9900300.0);
        let pred = a + b * x + c * y;
        let truth = 1000.0 + 0.01 * (x - 500000.0) + 0.02 * (y - 9900000.0);
        assert!((pred - truth).abs() < 1e-3, "plane off: pred {pred}, truth {truth}");
    }

    #[test]
    fn consistency_flags_the_outlier() {
        // Four wells on a flat 2000 m TVDSS surface, one well 12 m deep — should be flagged.
        let pts = vec![
            (Some(500000.0), Some(9900000.0), 2000.0f32),
            (Some(500500.0), Some(9900000.0), 2000.5),
            (Some(500000.0), Some(9900500.0), 1999.5),
            (Some(500500.0), Some(9900500.0), 2000.0),
            (Some(500250.0), Some(9900250.0), 2012.0), // outlier
        ];
        let core = compute_consistency(&pts, 3.0);
        assert!(core.flagged[4], "outlier well should be flagged (resid {})", core.residuals[4]);
        for i in 0..4 {
            assert!(!core.flagged[i], "inlier {i} wrongly flagged (resid {})", core.residuals[i]);
        }
    }

    #[test]
    fn consistency_leaves_coordless_wells_unscored_under_a_plane() {
        // Three coordinate wells define a plane; a fourth well has no coordinates and sits well
        // off the flat mean. It must NOT be flagged against the mean (it can't be placed on the
        // dipping surface) — it is left unscored so the readout doesn't mislead.
        let pts = vec![
            (Some(500000.0), Some(9900000.0), 2000.0f32),
            (Some(500500.0), Some(9900000.0), 2000.0),
            (Some(500000.0), Some(9900500.0), 2000.0),
            (None, None, 1985.0),
        ];
        let core = compute_consistency(&pts, 3.0);
        assert!(core.plane.is_some(), "3 coordinate wells → a plane");
        assert!(!core.residuals[3].is_finite(), "coordless well should be unscored (NaN residual)");
        assert!(!core.flagged[3], "coordless well must not be flagged against the flat mean");
        assert!(core.rms.is_finite(), "rms should be computed over the scored (coord) wells");
        for i in 0..3 {
            assert!(!core.flagged[i], "inlier {i} wrongly flagged");
        }
    }
}
