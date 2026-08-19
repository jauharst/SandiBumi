//! Assisted fluid-contact picking. Suggests a contact depth from logs — the Sw=0.5
//! crossover, the deep-resistivity drop (hydrocarbon → water), and the density-neutron gas
//! separation closing (gas-down-to) — each with a confidence; and checks cross-well
//! consistency (a contact is flat in TVDSS, so a least-squares plane through the picked
//! points should have small residuals — wells that disagree are flagged). Read-only: the
//! user accepts/edits the suggestion; nothing is auto-committed.

use crate::db;
use crate::equations::fetch_curve_frame;
use crate::schema_vocab::DepthDatum;
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

/// One QC group: every well-scoped contact of one TYPE, in one COMPARTMENT, governing the same set
/// of MARKERS.
///
/// All three parts of the key earn their place, and each was a way of pooling surfaces that are not
/// the same surface:
///
/// - **Marker**, because two stacked sands routinely have two different oil-water contacts.
/// - **A SET of markers**, because several stacked sands in one hydraulic unit just as routinely
///   share ONE contact. A single marker column can say the first and not the second.
/// - **Compartment**, because two fault blocks are not in pressure communication and have no reason
///   to sit on the same contact at all.
///
/// A contact stating none of them is its own group — never a member of every group. It cannot be:
/// nothing in it says which sand or which block it belongs to, and folding it in would put a
/// field-wide datum into a reservoir's own fit.
#[derive(Debug, Clone, Serialize)]
pub struct ContactGroup {
    pub contact_type: String,
    /// `None` = the contacts in this group state no compartment.
    pub compartment: Option<String>,
    /// Sorted. Empty = they state no marker; several = one contact shared across a stack.
    pub zones: Vec<String>,
    /// Contacts in the group, all scopes.
    pub n: usize,
    /// Of those, the well-scoped ones — the only ones the consistency check can use.
    pub n_well: usize,
}

/// Normalised group key for one contact: the compartment, and its markers sorted.
fn group_key(c: &db::FluidContact) -> (Option<String>, Vec<String>) {
    let comp = c.compartment.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let mut zones: Vec<String> =
        c.zones.iter().map(|z| z.trim().to_string()).filter(|z| !z.is_empty()).collect();
    // Sorted, so a contact entered as [B, A] and one entered as [A, B] are the same group. Two
    // contacts governing the same sands are the same surface however the user typed them.
    zones.sort();
    zones.dedup();
    (comp, zones)
}

/// Every `(type, compartment, marker set)` combination in the project, so a QC pane can check them
/// all rather than making the user name each one.
pub fn contact_groups(conn: &Connection) -> Vec<ContactGroup> {
    let contacts = db::list_fluid_contacts(conn).unwrap_or_default();
    let mut out: Vec<ContactGroup> = Vec::new();
    for c in &contacts {
        // Case-insensitive on the TYPE (a user typing "owc" means OWC) but exact on the marker and
        // compartment names, because those are data the user chose and two names differing only in
        // case are two names as far as everything else in this project is concerned.
        let (comp, zones) = group_key(c);
        match out.iter_mut().find(|g| {
            g.contact_type.eq_ignore_ascii_case(&c.contact_type)
                && g.compartment == comp
                && g.zones == zones
        }) {
            Some(g) => {
                g.n += 1;
                if c.well_id.is_some() {
                    g.n_well += 1;
                }
            }
            None => out.push(ContactGroup {
                contact_type: c.contact_type.to_uppercase(),
                compartment: comp,
                zones,
                n: 1,
                n_well: usize::from(c.well_id.is_some()),
            }),
        }
    }
    out.sort_by(|a, b| {
        a.contact_type
            .cmp(&b.contact_type)
            .then_with(|| a.compartment.cmp(&b.compartment))
            .then_with(|| a.zones.cmp(&b.zones))
    });
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactConsistency {
    pub contact_type: String,
    /// The compartment this check was restricted to; `None` = the contacts stating none.
    pub compartment: Option<String>,
    /// The marker set this check was restricted to; empty = the contacts stating none.
    pub zones: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DepthComparison {
    pub left: f32,
    pub right: f32,
    pub difference: f32,
    pub datum: DepthDatum,
}

/// Compares a stored zone top with a stored contact without erasing either declared datum.
/// The active survey is the per-well reference frame for the only cross-datum transform this
/// product currently owns: MD to TVDSS (or the reverse comparison). Other pairs stay refused
/// until their own reference transform is stored rather than inferred.
pub fn compare_zone_top_to_contact(
    conn: &Connection,
    well_id: &str,
    zone_name: &str,
    contact_id: &str,
) -> Result<DepthComparison, String> {
    let zone = db::list_zones(conn, well_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|z| z.zone_name == zone_name)
        .ok_or_else(|| format!("zone '{zone_name}' does not exist on the selected well"))?;
    let contact = db::list_fluid_contacts(conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|c| c.contact_id == contact_id)
        .ok_or_else(|| format!("contact '{contact_id}' does not exist"))?;
    if let Some(contact_well) = contact.well_id.as_deref() {
        if contact_well != well_id {
            return Err(format!("contact '{contact_id}' belongs to a different well"));
        }
    }

    let left_datum = zone.depth_datum;
    let right_datum = contact.depth_datum;
    let (left, right, datum) = if left_datum == right_datum {
        (zone.top_depth, contact.depth as f32, left_datum)
    } else {
        match (left_datum, right_datum) {
            (DepthDatum::Md, DepthDatum::Tvdss) => {
                let converted = md_to_tvdss(conn, well_id, zone.top_depth).ok_or_else(|| {
                    format!(
                        "cannot compare {} with {}: this well has no reference frame covering the MD value",
                        left_datum.as_str(),
                        right_datum.as_str()
                    )
                })?;
                (converted, contact.depth as f32, DepthDatum::Tvdss)
            }
            (DepthDatum::Tvdss, DepthDatum::Md) => {
                let converted = md_to_tvdss(conn, well_id, contact.depth as f32).ok_or_else(|| {
                    format!(
                        "cannot compare {} with {}: this well has no reference frame covering the MD value",
                        left_datum.as_str(),
                        right_datum.as_str()
                    )
                })?;
                (zone.top_depth, converted, DepthDatum::Tvdss)
            }
            _ => {
                return Err(format!(
                    "cannot compare {} with {}: this well has no stored transform between those datums",
                    left_datum.as_str(),
                    right_datum.as_str()
                ));
            }
        }
    };
    Ok(DepthComparison { left, right, difference: left - right, datum })
}

/// Checks whether every well-scoped contact of `contact_type` **in one marker** agrees on a flat
/// TVDSS surface. MD contacts are converted to TVDSS via each well's deviation survey; a dipping
/// plane is fitted when ≥3 wells have coordinates, otherwise the flat mean is used.
///
/// **The compartment and the marker set are part of the GROUP, not filters you may omit.** Passing
/// `None`/empty checks the contacts that state none — it does NOT mean "all of them". Two sands, or
/// two fault blocks, pooled into one fit produce a surface neither is on and then flag every well
/// as disagreeing with it, which is the exact opposite of what a QC is for.
pub fn check_contact_consistency(
    conn: &Connection,
    contact_type: &str,
    compartment: Option<&str>,
    zones: &[String],
    flag_abs: f32,
    well_ids: &[String],
) -> ContactConsistency {
    let want_comp =
        compartment.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let mut want_zones: Vec<String> =
        zones.iter().map(|z| z.trim().to_string()).filter(|z| !z.is_empty()).collect();
    want_zones.sort();
    want_zones.dedup();

    let none = |msg: &str| ContactConsistency {
        contact_type: contact_type.to_string(),
        compartment: want_comp.clone(),
        zones: want_zones.clone(),
        n: 0,
        mean_tvdss: f32::NAN,
        rms: f32::NAN,
        plane: None,
        wells: Vec::new(),
        error: Some(msg.to_string()),
    };
    let contacts = match db::list_fluid_contacts_for_wells(conn, well_ids) {
        Ok(c) => c,
        Err(e) => return none(&e.to_string()),
    };
    let wells = db::list_wells_by_ids(conn, well_ids).unwrap_or_default();
    let wmap: HashMap<String, &db::WellSummary> = wells.iter().map(|w| (w.well_id.clone(), w)).collect();

    let mut meta: Vec<(String, String, Option<f64>, Option<f64>, f32)> = Vec::new();
    for c in contacts.iter().filter(|c| {
        let (comp, z) = group_key(c);
        c.contact_type.eq_ignore_ascii_case(contact_type)
            && c.well_id.is_some()
            && comp == want_comp
            && z == want_zones
    }) {
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
        compartment: want_comp,
        zones: want_zones,
        n: meta.len(),
        mean_tvdss: core.mean,
        rms: core.rms,
        plane: core.plane,
        wells: out,
        error: None,
    }
}

// ---------------------------------------------------------------------------
// The two FWLs
// ---------------------------------------------------------------------------

/// The parameter a saturation-height run reads its free-water level from.
const FWL_PARAM: &str = "FWL";

/// One well/marker where the picked FWL contact and the FWL the arithmetic uses do not agree.
///
/// **This is the defect the marker column exists to expose.** A free-water level lives in two
/// places: `fluid_contacts`, where it is picked and drawn on the correlation panel, and
/// `zone_params`, where `sw_height` actually reads it. Nothing reconciled them, so the log could
/// show one surface while every saturation in the report was computed from another — silently, and
/// with both numbers looking entirely reasonable.
#[derive(Debug, Clone, Serialize)]
pub struct FwlCheck {
    pub well_id: String,
    pub well_name: String,
    pub zone_name: String,
    /// The picked contact's depth, in the reference it declares.
    pub contact_depth: f32,
    pub contact_is_tvdss: bool,
    /// What `zone_params` holds for this well and marker, if anything.
    pub param_value: Option<f32>,
    /// `contact - param`, NaN when there is no parameter or the two cannot be compared.
    pub difference: f32,
    /// What the user should do about it, in words.
    pub verdict: String,
    /// True when this row can be written to `zone_params` — i.e. the comparison is meaningful.
    pub can_apply: bool,
}

/// Compares every marker-tagged FWL contact against the parameter the arithmetic reads.
///
/// **An MD contact is reported, never converted.** The stored parameter carries no reference of its
/// own — `satheight.rs` documents FWL as "the same reference as the vertical-depth input", which is
/// a property of the run, not of the number. Converting the contact to TVDSS to force a comparison
/// would be asserting something about the parameter that nothing in the project actually says, and
/// the failure would be silent. Saying so and stopping is the honest move.
///
/// **A contact with no marker is skipped**, because there is no `zone_params` row it could
/// correspond to: the parameter is keyed by marker, and `*` is a whole-well value rather than this
/// contact's.
///
/// **A contact governing SEVERAL markers produces one row per marker.** Stacked sands in one
/// hydraulic unit share a contact, and the parameter they are computed from is per marker — so a
/// shared contact has to be checked against, and written to, every sand it governs. Reporting one
/// row for the contact would hide a sand whose parameter had drifted.
pub fn check_fwl_agreement(
    conn: &Connection,
    tolerance: f32,
    well_ids: &[String],
) -> Vec<FwlCheck> {
    let contacts = db::list_fluid_contacts_for_wells(conn, well_ids).unwrap_or_default();
    let wells = db::list_wells_by_ids(conn, well_ids).unwrap_or_default();
    let wmap: HashMap<&str, &db::WellSummary> =
        wells.iter().map(|w| (w.well_id.as_str(), w)).collect();

    let mut out = Vec::new();
    for c in contacts.iter().filter(|c| c.contact_type.eq_ignore_ascii_case(FWL_PARAM)) {
        let Some(wid) = c.well_id.as_deref() else { continue };
        let Some(w) = wmap.get(wid) else { continue };
        let (_, zones) = group_key(c);
        if zones.is_empty() {
            continue;
        }
        let params = db::list_zone_params(conn, wid).unwrap_or_default();

        for zone in &zones {
            let param = params
                .iter()
                .find(|z| &z.zone_name == zone && z.param_name == FWL_PARAM)
                .and_then(|z| z.value_num);

            let mut row = FwlCheck {
                well_id: wid.to_string(),
                well_name: w.well_name.clone(),
                zone_name: zone.clone(),
                contact_depth: c.depth as f32,
                contact_is_tvdss: c.is_tvdss,
                param_value: param,
                difference: f32::NAN,
                verdict: String::new(),
                can_apply: false,
            };

            if !c.is_tvdss {
                row.verdict =
                    "This contact is on MEASURED depth. The stored parameter carries no reference \
                     of its own - a run's FWL is on whatever reference its vertical-depth input \
                     was - so the two cannot be compared without asserting something the project \
                     does not say. Re-pick it in TVDSS, or set the parameter by hand."
                        .into();
                out.push(row);
                continue;
            }

            match param {
                None => {
                    row.verdict =
                        "Picked, but nothing computes from it: this marker has no FWL parameter, \
                         so a saturation-height run here would fall back to the module default."
                            .into();
                    row.can_apply = true;
                }
                Some(p) => {
                    let d = row.contact_depth - p;
                    row.difference = d;
                    if d.abs() <= tolerance.max(0.0) {
                        row.verdict = "Agrees with the value the arithmetic uses.".into();
                    } else {
                        row.verdict = format!(
                            "DISAGREES by {d:+.2}. The correlation panel draws {:.2} and every \
                             saturation-height run on this marker computes from {p:.2}.",
                            row.contact_depth
                        );
                        row.can_apply = true;
                    }
                }
            }
            out.push(row);
        }
    }
    out.sort_by(|a, b| a.well_name.cmp(&b.well_name).then_with(|| a.zone_name.cmp(&b.zone_name)));
    out
}

/// Copies picked FWL contacts into `zone_params`, so the arithmetic reads what the panel draws.
///
/// **An explicit copy, never a live read.** The alternative — having `sw_height` resolve its FWL
/// from the contact table at run time — would give the project two sources of truth reconciled
/// invisibly at the moment of calculation, and no stored run could afterwards say which it used.
/// This is the shape every other calibration in the app takes: fit, look, Apply, one transaction,
/// one undo (`calibrationApply.ts`).
///
/// Rows are grouped per marker because `set_zone_param_batch` writes one zone at a time; the whole
/// operation is still one call per marker rather than one per well.
pub fn apply_fwl_to_zone_params(
    conn: &mut Connection,
    picks: &[(String, String, f32)],
) -> Result<usize, String> {
    let mut by_zone: HashMap<&str, Vec<(String, String, Option<f32>)>> = HashMap::new();
    for (well_id, zone, depth) in picks {
        if !depth.is_finite() {
            return Err(format!("{zone}: a non-finite depth cannot be written as a parameter"));
        }
        by_zone
            .entry(zone.as_str())
            .or_default()
            .push((well_id.clone(), FWL_PARAM.to_string(), Some(*depth)));
    }
    let mut n = 0;
    for (zone, entries) in by_zone {
        n += db::set_zone_param_batch(conn, zone, &entries).map_err(|e| e.to_string())?;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_vocab::DepthDatum;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        conn
    }

    /// `zone_params.well_id` is a UUID column, so a well id has to be a real UUID string.
    fn add_well(conn: &Connection, name: &str, x: f64, y: f64) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO wells (well_id, well_name, surface_x, surface_y) VALUES (?1, ?2, ?3, ?4)",
            duckdb::params![id, name, x, y],
        )
        .unwrap();
        id
    }

    fn all_well_ids(conn: &Connection) -> Vec<String> {
        let mut stmt = conn.prepare("SELECT CAST(well_id AS VARCHAR) FROM wells ORDER BY well_id").unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<duckdb::Result<Vec<_>>>()
            .unwrap()
    }

    fn add_contact(conn: &Connection, well: &str, kind: &str, zones: &[&str], depth: f64) {
        add_in(conn, well, kind, None, zones, depth);
    }

    fn add_in(
        conn: &Connection,
        well: &str,
        kind: &str,
        compartment: Option<&str>,
        zones: &[&str],
        depth: f64,
    ) {
        let z: Vec<String> = zones.iter().map(|s| s.to_string()).collect();
        db::upsert_fluid_contact_with_datum(
            conn,
            &uuid::Uuid::new_v4().to_string(),
            None,
            Some(well),
            kind,
            depth,
            DepthDatum::Tvdss,
            None,
            None,
            compartment,
            &z,
        )
        .unwrap();
    }

    /// CORRECTNESS — F-17 and SB-DBM-T31 in `docs/PRD_v2/22_database-model.md`:
    /// TVDSS is positive down, elevation is positive up, and unlike datums cannot be compared
    /// until this well has a declared reference frame. The hand-derived control is
    /// 1000 m MD/TVD - 100 m elevation = 900 m TVDSS.
    #[test]
    fn an_md_zone_top_and_a_tvdss_contact_are_refused_without_a_frame_and_compare_with_positive_down_tvdss_with_one(
    ) {
        let conn = db();
        let well = add_well(&conn, "DEPTH-REFERENCE", 0.0, 0.0);
        db::upsert_zone_with_datum(
            &conn,
            &well,
            "REFERENCE_INTERVAL",
            1000.0,
            1100.0,
            DepthDatum::Md,
        )
        .unwrap();
        db::upsert_fluid_contact_with_datum(
            &conn,
            "reference-contact",
            None,
            Some(&well),
            "FWL",
            900.0,
            DepthDatum::Tvdss,
            None,
            None,
            None,
            &["REFERENCE_INTERVAL".to_string()],
        )
        .unwrap();

        let refusal = compare_zone_top_to_contact(
            &conn,
            &well,
            "REFERENCE_INTERVAL",
            "reference-contact",
        )
        .unwrap_err();
        assert!(refusal.contains("MD"), "the refusal must name MD: {refusal}");
        assert!(refusal.contains("TVDSS"), "the refusal must name TVDSS: {refusal}");

        let stations = crate::deviation::minimum_curvature(
            &[0.0, 1000.0, 1100.0],
            &[0.0, 0.0, 0.0],
            &[0.0, 0.0, 0.0],
            100.0,
        );
        assert!((stations[1].tvdss - 900.0).abs() < 1e-6, "TVDSS is positive down");
        db::insert_well_path(
            &conn,
            &well,
            "REFERENCE_FRAME",
            Some("SB-DBM-T31 fixture"),
            Some(100.0),
            &stations,
        )
        .unwrap();

        let comparison = compare_zone_top_to_contact(
            &conn,
            &well,
            "REFERENCE_INTERVAL",
            "reference-contact",
        )
        .unwrap();
        assert_eq!(comparison.datum, DepthDatum::Tvdss);
        assert!((comparison.left - 900.0).abs() < 1e-6);
        assert!((comparison.right - 900.0).abs() < 1e-6);
        assert!((comparison.difference).abs() < 1e-6);

        // The same contract must hold after opening a project written under the old, reversed
        // convention. Both the stored frame and an explicitly TVDSS contact are converted once;
        // running the migration again must not flip either value back.
        let legacy = db();
        let legacy_well = add_well(&legacy, "LEGACY-DEPTH-REFERENCE", 0.0, 0.0);
        let legacy_stations = [crate::deviation::Station {
            md: 1000.0,
            inc: 0.0,
            azi: 0.0,
            tvd: 1000.0,
            tvdss: -900.0,
        }];
        db::insert_well_path(
            &legacy,
            &legacy_well,
            "LEGACY_FRAME",
            Some("pre-SB-DBM-031 fixture"),
            Some(100.0),
            &legacy_stations,
        )
        .unwrap();
        db::upsert_fluid_contact_with_datum(
            &legacy,
            "legacy-contact",
            None,
            Some(&legacy_well),
            "FWL",
            -900.0,
            DepthDatum::Tvdss,
            None,
            None,
            None,
            &["REFERENCE_INTERVAL".to_string()],
        )
        .unwrap();
        legacy
            .execute(
                "INSERT INTO computed_curves (well_id, depth, curve_name, value)
                 VALUES (?1, 1000.0, 'TVDSS', -900.0)",
                duckdb::params![legacy_well],
            )
            .unwrap();
        legacy
            .execute(
                "INSERT INTO zones (well_id, zone_name, top_depth, bottom_depth)
                 VALUES (?1, 'UNDECLARED_INTERVAL', 1000.0, 1100.0)",
                duckdb::params![legacy_well],
            )
            .unwrap();

        db::migrate_tvdss_positive_down(&legacy, None).unwrap();
        db::migrate_tvdss_positive_down(&legacy, None).unwrap();
        assert!((db::get_well_path(&legacy, &legacy_well).unwrap()[0].tvdss - 900.0).abs() < 1e-6);
        assert!(
            (db::list_fluid_contacts(&legacy).unwrap()[0].depth - 900.0).abs() < 1e-6,
            "the declared TVDSS contact is converted once"
        );
        let stored_curve: f32 = legacy
            .query_row(
                "SELECT value FROM computed_curves
                 WHERE well_id = ?1 AND curve_name = 'TVDSS'",
                duckdb::params![legacy_well],
                |row| row.get(0),
            )
            .unwrap();
        assert!((stored_curve - 900.0).abs() < 1e-6, "the materialized TVDSS curve is converted once");
        let legacy_zone_datum: Option<String> = legacy
            .query_row(
                "SELECT depth_datum FROM zones WHERE well_id = ?1 AND zone_name = 'UNDECLARED_INTERVAL'",
                duckdb::params![legacy_well],
                |row| row.get(0),
            )
            .unwrap();
        assert!(legacy_zone_datum.is_none(), "an untyped legacy depth must not be relabelled MD");
        let legacy_refusal = db::list_zones(&legacy, &legacy_well).unwrap_err().to_string();
        assert!(
            legacy_refusal.contains("no declared depth datum"),
            "legacy use must refuse instead of inferring MD: {legacy_refusal}"
        );
    }

    /// The defect the marker column exists to fix. Two stacked sands each have their own oil-water
    /// contact, and both are perfectly flat within themselves. Pooled into one plane fit they
    /// produce a surface neither sand is on, and every well is then flagged as disagreeing with a
    /// contact that was never there.
    ///
    /// Asserted from BOTH sides: each marker must come back tight AND the unfiltered pooling must
    /// be the wide answer, or a version that had simply stopped fitting anything would pass.
    #[test]
    fn two_sands_are_two_contacts_and_are_never_pooled_into_one_surface() {
        let conn = db();
        let a = add_well(&conn, "SANDI-1", 0.0, 0.0);
        let b = add_well(&conn, "SANDI-2", 1000.0, 0.0);
        let c = add_well(&conn, "SANDI-3", 0.0, 1000.0);
        // Upper sand: a flat OWC at -2000. Lower sand: a flat OWC at -2400.
        for w in [&a, &b, &c] {
            add_contact(&conn, w, "OWC", &["UPPER"], -2000.0);
            add_contact(&conn, w, "OWC", &["LOWER"], -2400.0);
        }

        let upper = check_contact_consistency(&conn, "OWC", None, &["UPPER".into()], 3.0, &all_well_ids(&conn));
        assert_eq!(upper.n, 3, "the upper sand sees only its own three picks");
        assert!(upper.rms < 0.01, "and they are flat: rms {}", upper.rms);
        assert!(upper.wells.iter().all(|w| !w.flagged), "so nothing is flagged");

        let lower = check_contact_consistency(&conn, "OWC", None, &["LOWER".into()], 3.0, &all_well_ids(&conn));
        assert_eq!(lower.n, 3);
        assert!(lower.rms < 0.01, "rms {}", lower.rms);
        assert!((lower.mean_tvdss - (-2400.0)).abs() < 0.01, "on its own surface, not a blend");

        // The two groups are reported separately rather than as one.
        let groups = contact_groups(&conn);
        assert_eq!(groups.len(), 2, "two markers, two groups: {groups:?}");
        assert!(groups.iter().all(|g| g.n_well == 3 && g.contact_type == "OWC"));

        // The control, and the reason the marker matters: an unmarked pick is its OWN group and is
        // never folded into a marker's fit. Add TWO at a third depth (a single pick cannot be
        // checked against anything) and neither sand moves.
        add_contact(&conn, &a, "OWC", &[], -2200.0);
        add_contact(&conn, &b, "OWC", &[], -2200.0);
        let upper2 = check_contact_consistency(&conn, "OWC", None, &["UPPER".into()], 3.0, &all_well_ids(&conn));
        assert_eq!(upper2.n, 3, "the unmarked picks did not join the upper sand");
        assert!((upper2.mean_tvdss - (-2000.0)).abs() < 0.01);
        let unmarked = check_contact_consistency(&conn, "OWC", None, &[], 3.0, &all_well_ids(&conn));
        assert_eq!(unmarked.n, 2, "they are their own group");
        assert!((unmarked.mean_tvdss - (-2200.0)).abs() < 0.01);
    }

    /// Stacked sands in ONE hydraulic unit share one contact. That is the case a single marker
    /// column cannot express, and it is why the markers are a link table: the contact governs a SET
    /// of sands, and the set is what identifies it.
    ///
    /// The order the markers were entered in must not matter — two picks governing the same sands
    /// are the same surface however they were typed.
    #[test]
    fn stacked_sands_can_share_one_contact_whatever_order_they_were_entered_in() {
        let conn = db();
        let a = add_well(&conn, "SANDI-1", 0.0, 0.0);
        let b = add_well(&conn, "SANDI-2", 1000.0, 0.0);
        add_contact(&conn, &a, "OWC", &["A", "B", "C"], -2000.0);
        add_contact(&conn, &b, "OWC", &["C", "B", "A"], -2001.0);

        let groups = contact_groups(&conn);
        assert_eq!(groups.len(), 1, "one shared contact, one group: {groups:?}");
        assert_eq!(groups[0].zones, vec!["A", "B", "C"], "sorted, so entry order cannot split it");
        assert_eq!(groups[0].n_well, 2);

        let g = &groups[0];
        let chk = check_contact_consistency(&conn, "OWC", g.compartment.as_deref(), &g.zones, 3.0, &all_well_ids(&conn));
        assert_eq!(chk.n, 2, "both wells are in the same group");
        assert!(chk.rms < 1.0);

        // And a sand OUTSIDE the shared unit is a different surface, not part of it.
        add_contact(&conn, &a, "OWC", &["D"], -2400.0);
        assert_eq!(contact_groups(&conn).len(), 2, "the lone sand is its own group");
    }

    /// Two fault blocks are not in pressure communication and have no reason to sit on the same
    /// contact. Without the compartment they pool, the fit lands between them, and BOTH blocks are
    /// flagged as disagreeing with a surface neither is on.
    ///
    /// Asserted from both sides: each compartment must come back tight AND the pooled version must
    /// be visibly wrong, or an implementation that had stopped fitting anything would pass.
    #[test]
    fn two_compartments_are_two_contacts_even_in_the_same_sand() {
        let conn = db();
        let a = add_well(&conn, "SANDI-1", 0.0, 0.0);
        let b = add_well(&conn, "SANDI-2", 100.0, 0.0);
        let c = add_well(&conn, "SANDI-3", 5000.0, 0.0);
        let d = add_well(&conn, "SANDI-4", 5100.0, 0.0);
        // One sand, two blocks, 60 m apart across the fault.
        add_in(&conn, &a, "OWC", Some("NORTH"), &["UPPER"], -2000.0);
        add_in(&conn, &b, "OWC", Some("NORTH"), &["UPPER"], -2000.5);
        add_in(&conn, &c, "OWC", Some("SOUTH"), &["UPPER"], -2060.0);
        add_in(&conn, &d, "OWC", Some("SOUTH"), &["UPPER"], -2060.5);

        let groups = contact_groups(&conn);
        assert_eq!(groups.len(), 2, "one sand, two compartments, two groups: {groups:?}");

        let north =
            check_contact_consistency(&conn, "OWC", Some("NORTH"), &["UPPER".into()], 3.0, &all_well_ids(&conn));
        assert_eq!(north.n, 2);
        assert!(north.rms < 1.0, "the north block is flat within itself: {}", north.rms);
        assert!((north.mean_tvdss - (-2000.25)).abs() < 0.5);

        let south =
            check_contact_consistency(&conn, "OWC", Some("SOUTH"), &["UPPER".into()], 3.0, &all_well_ids(&conn));
        assert!((south.mean_tvdss - (-2060.25)).abs() < 0.5, "and sits on its own level");

        // The control: the compartment is doing the work. Strip it and all four pool into one fit
        // whose spread is the fault throw rather than the pick uncertainty.
        for w in [&a, &b, &c, &d] {
            add_contact(&conn, w, "GWC", &["UPPER"], if w == &a || w == &b { -2000.0 } else { -2060.0 });
        }
        let pooled = check_contact_consistency(&conn, "GWC", None, &["UPPER".into()], 3.0, &all_well_ids(&conn));
        assert_eq!(pooled.n, 4);
        assert!(
            pooled.rms > 10.0,
            "pooling two blocks gives a surface neither is on: rms {}",
            pooled.rms
        );
    }

    /// The two-FWL split, which is the whole reason this went in. A free-water level lives in
    /// `fluid_contacts` (drawn on the panel) and in `zone_params` (what `sw_height` computes from).
    /// Nothing reconciled them, so the log could show one surface while every saturation came from
    /// another — both entirely plausible numbers.
    #[test]
    fn a_picked_contact_and_the_computed_one_are_compared_and_can_be_reconciled() {
        let mut conn = db();
        let w = add_well(&conn, "SANDI-1", 0.0, 0.0);
        add_contact(&conn, &w, "FWL", &["UPPER"], -2000.0);
        // The arithmetic is using a different level.
        db::set_zone_param_batch(&mut conn, "UPPER", &[(w.clone(), "FWL".into(), Some(-2035.0))])
            .unwrap();

        let checks = check_fwl_agreement(&conn, 0.1, &all_well_ids(&conn));
        assert_eq!(checks.len(), 1);
        let c = &checks[0];
        assert!((c.difference - 35.0).abs() < 0.01, "the gap is reported: {}", c.difference);
        assert!(c.verdict.contains("DISAGREES"), "{}", c.verdict);
        assert!(c.can_apply);

        apply_fwl_to_zone_params(&mut conn, &[(w.clone(), "UPPER".into(), -2000.0)]).unwrap();
        let after = check_fwl_agreement(&conn, 0.1, &all_well_ids(&conn));
        assert!(after[0].verdict.starts_with("Agrees"), "{}", after[0].verdict);
        assert!(!after[0].can_apply);
        // And it really went where the module reads it from.
        let p = db::list_zone_params(&conn, &w).unwrap();
        let fwl = p.iter().find(|z| z.zone_name == "UPPER" && z.param_name == "FWL").unwrap();
        assert!((fwl.value_num.unwrap() - (-2000.0)).abs() < 0.01);
    }

    /// An MD contact is REPORTED, never converted. The stored parameter carries no reference of its
    /// own — `satheight.rs` documents FWL as "the same reference as the vertical-depth input",
    /// which is a property of the run — so converting the contact to force a comparison would
    /// assert something the project never said, and would do it silently.
    #[test]
    fn a_measured_depth_contact_is_refused_rather_than_converted() {
        let conn = db();
        let w = add_well(&conn, "SANDI-1", 0.0, 0.0);
        db::upsert_fluid_contact_with_datum(
            &conn,
            "c1",
            None,
            Some(&w),
            "FWL",
            2100.0,
            DepthDatum::Md,
            None,
            None,
            None,
            &["UPPER".to_string()],
        )
        .unwrap();
        let checks = check_fwl_agreement(&conn, 0.1, &all_well_ids(&conn));
        assert_eq!(checks.len(), 1);
        assert!(!checks[0].can_apply, "nothing may be written from it");
        assert!(checks[0].verdict.contains("MEASURED depth"), "{}", checks[0].verdict);
        assert!(checks[0].difference.is_nan(), "and no difference is invented");
    }

    /// A contact with no marker has no `zone_params` row it could correspond to — the parameter is
    /// keyed by marker, and `*` is a whole-well value rather than this contact's. Skipping it is
    /// the honest answer; matching it to `*` would let one pick silently rewrite every zone.
    #[test]
    fn an_unmarked_contact_is_not_matched_against_the_whole_well_value() {
        let mut conn = db();
        let w = add_well(&conn, "SANDI-1", 0.0, 0.0);
        add_contact(&conn, &w, "FWL", &[], -2000.0);
        db::set_zone_param_batch(&mut conn, "*", &[(w.clone(), "FWL".into(), Some(-2500.0))])
            .unwrap();
        assert!(check_fwl_agreement(&conn, 0.1, &all_well_ids(&conn)).is_empty(), "no marker, no comparison");
    }

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
