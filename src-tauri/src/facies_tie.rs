//! Electrofacies tie-in QC (Wave B item 8, increment 2) — cross-tabulate a predicted log-domain
//! rock-type curve (e.g. RT_LOG from `rt_cutoff`) against a reference/core rock-type curve at
//! matched depths, and report the confusion matrix + dominant-class purity (ref_rocktyping_shf.md
//! §Cutoff-based electrofacies tie-in: "accept the mapping if dominant-class purity is above a
//! threshold"). The two curves need NOT share a labelling scheme — rows are reference classes,
//! columns predicted classes, and purity measures how cleanly each reference class maps to one
//! predicted class.

use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct FaciesConfusionRequest {
    /// Read the curves this run consumes from THIS log set's stored values (latest version per
    /// well) rather than from whatever the current values are. Curves the set never wrote fall
    /// back to normal resolution; an empty name means "current values", which is what every
    /// caller did before this existed (Jauhar, 2026-08-05).
    #[serde(default)]
    pub input_set: Option<String>,
    pub well_ids: Vec<String>,
    /// Predicted (log-domain) rock-type curve, e.g. RT_LOG.
    pub pred_curve: String,
    /// Reference rock-type curve, e.g. a core-derived rock type or a rocktyping RT_CLASS.
    pub ref_curve: String,
    /// Dominant-class purity at or above which the mapping is ACCEPTED, as a fraction 0..1.
    ///
    /// **Ships with no default and stays `None` until the user states one** (SB-MLA-052,
    /// SB-CORE-004). The method note this module implements says to accept the mapping when
    /// dominant-class purity is above a threshold and states no value, and no source in the
    /// corpus states one either. Electing a number here would put SandiBumi's guess behind the
    /// method's silence, and a mapping stamped "accepted" against an invented bar is worse than
    /// one carrying a bare purity the reader has to judge. `#[serde(default)]`, so every caller
    /// that predates this still deserializes.
    #[serde(default)]
    pub accept_threshold: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RefClassRow {
    pub ref_label: i64,
    pub dominant_pred: i64,
    /// ROW-normalised: of the samples of this reference class, the fraction the model put in
    /// `dominant_pred`. See `row_axis` — this number is meaningless without its axis.
    pub purity: f64,
    pub count: i64,
}

/// The column-wise counterpart of `RefClassRow` — Geolog's "recognition rate" axis.
#[derive(Debug, Serialize)]
pub struct PredClassRow {
    pub pred_label: i64,
    pub dominant_ref: i64,
    /// COLUMN-normalised: of the samples the model called this predicted class, the fraction that
    /// really were `dominant_ref`. See `col_axis`.
    pub recognition: f64,
    pub count: i64,
}

/// What a ROW-normalised cell divides by, in words. Emitted with the result so a payload cannot be
/// read with the wrong denominator (SB-MLA-051).
pub const ROW_AXIS: &str =
    "row (by reference class): of the samples this reference class holds, the fraction the model \
     assigned to each predicted class - answers whether the model FINDS this rock";
/// What a COLUMN-normalised cell divides by, in words.
pub const COL_AXIS: &str =
    "column (by predicted class): of the samples the model assigned to this predicted class, the \
     fraction that really were each reference class - answers whether the label can be TRUSTED";

#[derive(Debug, Serialize)]
pub struct FaciesConfusionResult {
    pub ref_labels: Vec<i64>,
    pub pred_labels: Vec<i64>,
    /// matrix[i][j] = count where reference == ref_labels[i] and prediction == pred_labels[j].
    pub matrix: Vec<Vec<i64>>,
    /// `matrix` divided by its ROW sums, as fractions 0..1. Labelled by `row_axis`.
    pub row_pct: Vec<Vec<f64>>,
    /// `matrix` divided by its COLUMN sums, as fractions 0..1. Labelled by `col_axis`.
    pub col_pct: Vec<Vec<f64>>,
    /// Prose statement of what `row_pct` (and `per_ref[].purity`) divides by.
    pub row_axis: String,
    /// Prose statement of what `col_pct` (and `per_pred[].recognition`) divides by.
    pub col_axis: String,
    pub per_ref: Vec<RefClassRow>,
    pub per_pred: Vec<PredClassRow>,
    /// Σ over reference classes of the dominant-cell count / total pairs. ROW-normalised.
    pub overall_purity: f64,
    /// The threshold the USER stated, echoed so the result records the decision it was judged
    /// against. `None` means none was stated — see `accept_note`.
    pub accept_threshold: Option<f64>,
    /// `overall_purity >= accept_threshold`, or `None` when no threshold was stated. A mapping is
    /// never judged against a number SandiBumi chose.
    pub accepted: Option<bool>,
    /// Why there is no verdict, when there is none.
    pub accept_note: Option<String>,
    pub n: usize,
    /// ANOVA-style variance reduction of log10(core k) when grouped by the PREDICTED class:
    /// 1 − SS_within/SS_total. 1 = the typing explains all core-perm variance, 0 = none.
    /// NaN (→ JSON null) when no core plugs match or fewer than 2 classes carry plugs.
    pub k_var_reduction: f64,
    /// Core plugs that contributed to `k_var_reduction` (valid k, matched to a predicted class).
    pub n_core_plugs: usize,
    /// Plugs with a usable permeability that found NO log sample inside the join tolerance, and so
    /// contributed to nothing. Reported rather than absorbed: a variance reduction computed on
    /// nine of ninety plugs is a different statement from one computed on ninety (SB-MLA-054).
    pub n_core_unmatched: usize,
    /// How a core plug was put on the log's depth frame, in words — the method and its tolerance.
    pub core_match_note: String,
    pub error: Option<String>,
}

/// How the core-to-log depth join is done, stated with the result rather than left in the source.
///
/// The distance is quoted in the unit the PROJECT stores its depths in, because that is the unit
/// the join measured in. `None` is the honest reading where no join was attempted - an argument
/// refusal, or a result built before any plug was read: there is no project unit in hand at that
/// point, and quoting a number for a comparison that never ran would state a tolerance nothing
/// was measured against. One sentence either way, because two copies of it are two places for the
/// wording to drift.
fn core_match_note(unit: Option<crate::units::DepthUnit>) -> String {
    let distance = match unit {
        Some(unit) => format!("{:.2} {}", core_match_tol_in(unit), unit.label()),
        None => format!("{CORE_MATCH_TOL_METRES:.2} m of hole, restated in the project's own depth unit"),
    };
    format!("core plugs joined to the NEAREST log sample within {distance}; no interpolation, and a plug with no sample inside that distance is dropped rather than stretched to one")
}

fn confusion_err(msg: &str) -> FaciesConfusionResult {
    FaciesConfusionResult {
        ref_labels: vec![],
        pred_labels: vec![],
        matrix: vec![],
        row_pct: vec![],
        col_pct: vec![],
        row_axis: ROW_AXIS.to_string(),
        col_axis: COL_AXIS.to_string(),
        per_ref: vec![],
        per_pred: vec![],
        overall_purity: f64::NAN,
        accept_threshold: None,
        accepted: None,
        accept_note: None,
        n: 0,
        k_var_reduction: f64::NAN,
        n_core_plugs: 0,
        n_core_unmatched: 0,
        core_match_note: core_match_note(None),
        error: Some(msg.to_string()),
    }
}

/// ANOVA variance reduction of y grouped by class: 1 − SS_within/SS_total. NaN when fewer than
/// 2 populated classes or SS_total ~ 0 (no variance to explain).
pub fn variance_reduction(groups: &[(i64, f64)]) -> f64 {
    if groups.len() < 2 {
        return f64::NAN;
    }
    let n = groups.len() as f64;
    let mean = groups.iter().map(|&(_, y)| y).sum::<f64>() / n;
    let ss_total: f64 = groups.iter().map(|&(_, y)| (y - mean).powi(2)).sum();
    if ss_total < 1e-12 {
        return f64::NAN;
    }
    let mut by_class: std::collections::HashMap<i64, Vec<f64>> = std::collections::HashMap::new();
    for &(c, y) in groups {
        by_class.entry(c).or_default().push(y);
    }
    if by_class.len() < 2 {
        return f64::NAN;
    }
    let ss_within: f64 = by_class
        .values()
        .map(|ys| {
            let m = ys.iter().sum::<f64>() / ys.len() as f64;
            ys.iter().map(|y| (y - m).powi(2)).sum::<f64>()
        })
        .sum();
    (1.0 - ss_within / ss_total).clamp(0.0, 1.0)
}

/// Builds the confusion matrix + purity from (reference_class, predicted_class) integer pairs.
pub fn build_confusion(pairs: &[(i64, i64)]) -> FaciesConfusionResult {
    if pairs.is_empty() {
        return confusion_err("no matched samples where both curves are present");
    }
    let ref_labels: Vec<i64> = pairs.iter().map(|&(r, _)| r).collect::<BTreeSet<_>>().into_iter().collect();
    let pred_labels: Vec<i64> = pairs.iter().map(|&(_, p)| p).collect::<BTreeSet<_>>().into_iter().collect();
    let ri: std::collections::HashMap<i64, usize> = ref_labels.iter().enumerate().map(|(i, &l)| (l, i)).collect();
    let pi: std::collections::HashMap<i64, usize> = pred_labels.iter().enumerate().map(|(i, &l)| (l, i)).collect();

    let mut matrix = vec![vec![0i64; pred_labels.len()]; ref_labels.len()];
    for &(r, p) in pairs {
        matrix[ri[&r]][pi[&p]] += 1;
    }

    let mut per_ref = Vec::new();
    let mut dominant_total = 0i64;
    let mut row_pct = vec![vec![0.0f64; pred_labels.len()]; ref_labels.len()];
    for (i, &rl) in ref_labels.iter().enumerate() {
        let row = &matrix[i];
        let rowsum: i64 = row.iter().sum();
        let (jmax, &vmax) = row.iter().enumerate().max_by_key(|&(_, &v)| v).unwrap();
        dominant_total += vmax;
        if rowsum > 0 {
            for (j, &v) in row.iter().enumerate() {
                row_pct[i][j] = v as f64 / rowsum as f64;
            }
        }
        per_ref.push(RefClassRow {
            ref_label: rl,
            dominant_pred: pred_labels[jmax],
            purity: if rowsum > 0 { vmax as f64 / rowsum as f64 } else { 0.0 },
            count: rowsum,
        });
    }
    let overall_purity = dominant_total as f64 / pairs.len() as f64;

    // The other axis. Same cells, different denominator, and a different question — one says
    // whether the model finds a rock, the other whether its label can be trusted (SB-MLA-051).
    let mut per_pred = Vec::new();
    let mut col_pct = vec![vec![0.0f64; pred_labels.len()]; ref_labels.len()];
    for (j, &pl) in pred_labels.iter().enumerate() {
        let colsum: i64 = (0..ref_labels.len()).map(|i| matrix[i][j]).sum();
        let (imax, vmax) = (0..ref_labels.len())
            .map(|i| (i, matrix[i][j]))
            .max_by_key(|&(_, v)| v)
            .unwrap();
        if colsum > 0 {
            for i in 0..ref_labels.len() {
                col_pct[i][j] = matrix[i][j] as f64 / colsum as f64;
            }
        }
        per_pred.push(PredClassRow {
            pred_label: pl,
            dominant_ref: ref_labels[imax],
            recognition: if colsum > 0 { vmax as f64 / colsum as f64 } else { 0.0 },
            count: colsum,
        });
    }

    FaciesConfusionResult {
        ref_labels,
        pred_labels,
        matrix,
        row_pct,
        col_pct,
        row_axis: ROW_AXIS.to_string(),
        col_axis: COL_AXIS.to_string(),
        per_ref,
        per_pred,
        overall_purity,
        accept_threshold: None,
        accepted: None,
        accept_note: None,
        n: pairs.len(),
        k_var_reduction: f64::NAN,
        n_core_plugs: 0,
        n_core_unmatched: 0,
        core_match_note: core_match_note(None),
        error: None,
    }
}

/// Applies the user's acceptance threshold, or records that they stated none (SB-MLA-052).
fn judge(res: &mut FaciesConfusionResult, threshold: Option<f64>) {
    match threshold {
        Some(t) if t.is_finite() && (0.0..=1.0).contains(&t) => {
            res.accept_threshold = Some(t);
            res.accepted = Some(res.overall_purity >= t);
        }
        Some(_) => {
            res.accept_note = Some(
                "the acceptance threshold must be a dominant-class purity between 0 and 1; the \
                 mapping was not judged"
                    .into(),
            );
        }
        None => {
            res.accept_note = Some(
                "no acceptance threshold was stated, so the mapping is reported and not judged. \
                 The method note says to accept above a threshold and states no value, and neither \
                 does any source SandiBumi holds - the bar is yours to set for this field."
                    .into(),
            );
        }
    }
}

/// Max |depth difference| for matching a core plug to the nearest log sample, as a PHYSICAL SIZE:
/// one metre of hole, about three log samples at a normal half-foot sampling.
///
/// It is never compared against a depth directly. AUDIT-2026-08-20 finding 16: this shipped as a
/// bare `CORE_MATCH_TOL_M: f32 = 1.0` measured against depths in whatever unit the project stores,
/// so a FOOT project paired plugs within 1 ft - a third of the rock the note promises - and booked
/// every plug in the other two thirds as unmatched, which reads as a core delivery that misses the
/// log rather than as a tolerance that shrank. `plugqc`'s `default_depth_tol` is the same fix for
/// the same class (finding 17); the two differ only in that six inches is a real anchor there and
/// one metre is not one here, so this converts EXACTLY rather than picking a round foot number.
const CORE_MATCH_TOL_METRES: f64 = 1.0;

/// [`CORE_MATCH_TOL_METRES`] restated in the unit a project stores its depths in.
fn core_match_tol_in(unit: crate::units::DepthUnit) -> f32 {
    crate::units::metres_in(CORE_MATCH_TOL_METRES, unit) as f32
}

/// Index of the sample in ascending `depth` nearest to `target`, if within `tol`.
///
/// `tol` is passed rather than read from a constant so it arrives in the SAME unit as `depth`.
fn nearest_within(depth: &[f32], target: f32, tol: f32) -> Option<usize> {
    if depth.is_empty() || !target.is_finite() {
        return None;
    }
    let pos = depth.partition_point(|&d| d < target);
    let mut best: Option<usize> = None;
    for cand in [pos.wrapping_sub(1), pos] {
        if cand < depth.len() {
            let dd = (depth[cand] - target).abs();
            if dd <= tol && best.map_or(true, |b| dd < (depth[b] - target).abs()) {
                best = Some(cand);
            }
        }
    }
    best
}

pub fn run_facies_confusion(db: &Mutex<Connection>, req: &FaciesConfusionRequest) -> FaciesConfusionResult {
    if req.well_ids.is_empty() {
        return confusion_err("select at least one well");
    }
    let pred = req.pred_curve.trim().to_uppercase();
    let refc = req.ref_curve.trim().to_uppercase();
    if pred.is_empty() || refc.is_empty() {
        return confusion_err("both the predicted and reference rock-type curves are required");
    }
    if pred == refc {
        return confusion_err("predicted and reference curves are the same");
    }

    let mut pairs: Vec<(i64, i64)> = Vec::new();
    // (predicted class, log10 core k) at core-plug depths — for the k-variance-reduction QC.
    let mut core_groups: Vec<(i64, f64)> = Vec::new();
    let mut core_unmatched = 0usize;
    // Wells whose core could not be READ at all — a cross-datum delivery, chiefly. Kept apart
    // from `core_unmatched`, which counts plugs that WERE read and found no log sample nearby.
    let mut core_refusals: Vec<String> = Vec::new();
    // The unit this project stores its depths in, read once and carried out of the lock: it sets
    // the join tolerance below AND the note that reports it, so the two cannot disagree.
    let depth_unit;
    {
        let conn = db.lock().unwrap();
        depth_unit = crate::units::project_depth_unit(&conn).ok().flatten().unwrap_or_default();
        // AUDIT-2026-08-20 finding 16: one metre of hole, restated before it meets a depth.
        let core_tol = core_match_tol_in(depth_unit);
        let names = vec![pred.clone(), refc.clone()];
        for well_id in &req.well_ids {
            let Ok((d, cols)) = crate::equations::fetch_curve_frame_from_set(&conn, well_id, &names, req.input_set.as_deref(), None) else { continue };
            let (Some(pv), Some(rv)) = (cols.get(&pred), cols.get(&refc)) else { continue };
            let n = pv.len().min(rv.len());
            for i in 0..n {
                let (p, r) = (pv[i] as f64, rv[i] as f64);
                if p.is_finite() && r.is_finite() {
                    pairs.push((r.round() as i64, p.round() as i64));
                }
            }
            // Does the predicted typing explain core permeability? Sample the PREDICTED class at
            // each plug depth (nearest log sample within tolerance) and pool (class, log10 k).
            // A refusal here is REPORTED, never swallowed. `k_var_reduction` is NaN both when no
            // plug matched and when the core could not be read at all, and those are different
            // statements — one says the typing explains nothing, the other says nothing was
            // compared. The reason rides on `core_match_note`, whose whole job is to say how the
            // plugs were put on the log's depth frame; here they were not.
            let plugs = match crate::db::get_core_plugs(&conn, well_id) {
                Ok(p) => p,
                Err(e) => {
                    // Named by WELL, never by the UUID the user has never seen.
                    let well: String = conn
                        .query_row("SELECT well_name FROM wells WHERE well_id = ?1", [well_id], |r| {
                            r.get(0)
                        })
                        .unwrap_or_else(|_| well_id.clone());
                    core_refusals.push(format!("{well}: {e}"));
                    continue;
                }
            };
            for plug in plugs {
                let k = plug.cperm as f64;
                if !(k.is_finite() && k > 0.0) {
                    continue;
                }
                let Some(idx) = nearest_within(&d, plug.depth, core_tol) else {
                    core_unmatched += 1;
                    continue;
                };
                if idx < pv.len() && (pv[idx] as f64).is_finite() {
                    core_groups.push(((pv[idx] as f64).round() as i64, k.log10()));
                } else {
                    core_unmatched += 1;
                }
            }
        }
    }
    let mut res = build_confusion(&pairs);
    // This run DID have a project unit in hand, so the note quotes the distance the join actually
    // measured. `build_confusion` cannot: it is handed pairs, never a connection.
    res.core_match_note = core_match_note(Some(depth_unit));
    res.k_var_reduction = variance_reduction(&core_groups);
    res.n_core_plugs = core_groups.len();
    res.n_core_unmatched = core_unmatched;
    if !core_refusals.is_empty() {
        res.core_match_note =
            format!("{} — core NOT read for: {}", res.core_match_note, core_refusals.join("; "));
    }
    judge(&mut res, req.accept_threshold);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::DepthUnit;

    /// AUDIT-2026-08-20 finding 16. Pinned from BOTH sides, because the lazier implementation -
    /// the one that shipped - passes the metre half perfectly: it returned 1.0 always.
    #[test]
    fn the_core_match_tolerance_is_one_metre_of_hole_in_whatever_unit_the_project_stores() {
        // A metre project states it unchanged.
        assert_eq!(core_match_tol_in(DepthUnit::Metres), 1.0);

        // A foot project states the same PHYSICAL DISTANCE, which is a different NUMBER. Reusing
        // 1.0 there paired plugs within one foot - under a third of the rock the note promises -
        // and booked every plug in the other two thirds as unmatched, which reads as a core
        // delivery that misses the log rather than as a tolerance that quietly shrank.
        let ft = core_match_tol_in(DepthUnit::Feet);
        assert!((ft - 3.280_84).abs() < 1e-4, "one metre in feet, got {ft}");
        assert!(ft > 3.0, "an exact conversion, never the metre number reused");

        // And the note reports the distance in the unit the join measured in, both ways - so a
        // reader cannot be told "within 1 m" about a comparison that ran in feet.
        assert!(core_match_note(Some(DepthUnit::Metres)).contains("within 1.00 m;"));
        assert!(core_match_note(Some(DepthUnit::Feet)).contains("within 3.28 ft;"));

        // Where no join was attempted there is no project unit, and the note quotes no number it
        // could not have measured against.
        let unjoined = core_match_note(None);
        assert!(unjoined.contains("restated in the project's own depth unit"));
        assert!(!unjoined.contains("within 1.00 m;"), "no distance for a join that never ran");
    }

    #[test]
    fn confusion_tallies_and_scores_purity() {
        // Reference class 1 maps cleanly to predicted 2 (4/4); reference 2 splits 3-vs-1.
        let pairs = [(1, 2), (1, 2), (1, 2), (1, 2), (2, 3), (2, 3), (2, 3), (2, 1)];
        let res = build_confusion(&pairs);
        assert_eq!(res.ref_labels, vec![1, 2]);
        assert_eq!(res.pred_labels, vec![1, 2, 3]);
        assert_eq!(res.n, 8);
        // Row for ref 1: all 4 in the pred-2 column.
        let r1 = res.per_ref.iter().find(|r| r.ref_label == 1).unwrap();
        assert_eq!(r1.dominant_pred, 2);
        assert!((r1.purity - 1.0).abs() < 1e-9);
        // Row for ref 2: 3/4 map to pred 3.
        let r2 = res.per_ref.iter().find(|r| r.ref_label == 2).unwrap();
        assert_eq!(r2.dominant_pred, 3);
        assert!((r2.purity - 0.75).abs() < 1e-9);
        // Overall dominant purity = (4 + 3) / 8.
        assert!((res.overall_purity - 7.0 / 8.0).abs() < 1e-9);
    }

    #[test]
    fn confusion_rejects_empty() {
        assert!(build_confusion(&[]).error.is_some());
    }

    #[test]
    fn a_confusion_cell_carries_both_normalisations_and_names_which_axis_each_divides_by() {
        // The imbalance that makes this a correctness rule rather than a display one: reference
        // class 1 is small and perfectly found, but the predicted-1 label is mostly reference 2.
        // Row says 100 %, column says 18 %, and both are true of the same cell.
        let mut pairs: Vec<(i64, i64)> = vec![(1, 1); 10];
        pairs.extend(vec![(2, 1); 45]);
        pairs.extend(vec![(2, 2); 45]);
        let res = build_confusion(&pairs);

        assert_eq!(res.matrix, vec![vec![10, 0], vec![45, 45]]);
        // ROW: of reference class 1's samples, all went to predicted 1.
        assert!((res.row_pct[0][0] - 1.0).abs() < 1e-12);
        // COLUMN: of the samples called predicted 1, only 10 of 55 really were reference 1.
        assert!((res.col_pct[0][0] - 10.0 / 55.0).abs() < 1e-12);
        // Neither alone would pass: emitting one matrix under both names must fail here.
        assert!((res.row_pct[0][0] - res.col_pct[0][0]).abs() > 0.5);
        // And they must not be swapped — rows sum to 1 across, columns sum to 1 down.
        for row in &res.row_pct {
            assert!((row.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
        for j in 0..res.pred_labels.len() {
            let s: f64 = (0..res.ref_labels.len()).map(|i| res.col_pct[i][j]).sum();
            assert!((s - 1.0).abs() < 1e-12);
        }
        // Each summary row states its own axis's answer, and they disagree by construction.
        let ref1 = res.per_ref.iter().find(|r| r.ref_label == 1).unwrap();
        assert!((ref1.purity - 1.0).abs() < 1e-12);
        let pred1 = res.per_pred.iter().find(|p| p.pred_label == 1).unwrap();
        assert_eq!(pred1.dominant_ref, 2);
        assert!((pred1.recognition - 45.0 / 55.0).abs() < 1e-12);
        assert_eq!(pred1.count, 55);
        // A percentage travels with the denominator it was divided by, never bare.
        assert!(res.row_axis.contains("reference class") && res.row_axis.contains("row"));
        assert!(res.col_axis.contains("predicted class") && res.col_axis.contains("column"));
        assert_ne!(res.row_axis, res.col_axis);
    }

    #[test]
    fn a_mapping_is_never_judged_against_a_threshold_sandibumi_chose() {
        let pairs: Vec<(i64, i64)> = {
            let mut p = vec![(1, 1); 10];
            p.extend(vec![(2, 1); 45]);
            p.extend(vec![(2, 2); 45]);
            p
        };
        // Stating nothing yields no verdict and says why — not a silent pass against a default.
        let mut none = build_confusion(&pairs);
        judge(&mut none, None);
        assert!((none.overall_purity - 0.55).abs() < 1e-12);
        assert_eq!(none.accepted, None);
        assert_eq!(none.accept_threshold, None);
        assert!(none.accept_note.as_deref().unwrap().contains("yours to set"));

        // The other side: a threshold the user DID state is honoured and recorded with the result,
        // so a stored verdict can always be read back against the bar it was measured on.
        let mut pass = build_confusion(&pairs);
        judge(&mut pass, Some(0.5));
        assert_eq!(pass.accepted, Some(true));
        assert_eq!(pass.accept_threshold, Some(0.5));
        assert!(pass.accept_note.is_none());

        let mut fail = build_confusion(&pairs);
        judge(&mut fail, Some(0.9));
        assert_eq!(fail.accepted, Some(false));
        assert_eq!(fail.accept_threshold, Some(0.9));

        // A purity is a fraction; 90 typed for 90 % is refused rather than read as "never accept".
        let mut bad = build_confusion(&pairs);
        judge(&mut bad, Some(90.0));
        assert_eq!(bad.accepted, None);
        assert!(bad.accept_note.as_deref().unwrap().contains("between 0 and 1"));
    }

    #[test]
    fn variance_reduction_scores_separation() {
        // Two classes with distinct means and zero within-class spread → full reduction (1.0).
        let perfect: Vec<(i64, f64)> = vec![(1, 0.0), (1, 0.0), (2, 3.0), (2, 3.0)];
        assert!((variance_reduction(&perfect) - 1.0).abs() < 1e-9);
        // One class → NaN (nothing to compare).
        assert!(variance_reduction(&[(1, 0.0), (1, 1.0)]).is_nan());
        // Classes with identical distributions → ~0 reduction.
        let none: Vec<(i64, f64)> = vec![(1, 0.0), (1, 2.0), (2, 0.0), (2, 2.0)];
        assert!(variance_reduction(&none) < 1e-9);
        // No variance at all → NaN, not a fake 1.0.
        assert!(variance_reduction(&[(1, 5.0), (2, 5.0), (2, 5.0)]).is_nan());
    }

    #[test]
    fn run_confusion_reports_core_k_variance_reduction() {
        use duckdb::Connection;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        crate::db::insert_well(&conn, id, "FT-1", None, None, None).unwrap();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![40.0; n],
            vec![10.0; n],
            vec![0.2; n],
            vec![2.4; n],
            vec![80.0; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let ids = id.to_string();
        // Predicted RT: class 1 over the top half, class 2 over the bottom; reference identical.
        let rt: Vec<f32> = (0..n).map(|i| if i < 10 { 1.0 } else { 2.0 }).collect();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_LOG", &rt).unwrap();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_REF", &rt).unwrap();
        // Core plugs: class-1 depths carry k≈100 mD, class-2 depths k≈1 mD (well separated),
        // plus one invalid-k plug that must be skipped, and one valid-k plug 40 m below the
        // logged interval that must be COUNTED as unmatched rather than absorbed.
        let cd: Vec<f32> = vec![2000.2, 2001.2, 2002.2, 2005.7, 2006.2, 2007.7, 2050.0, 2049.0];
        let ck: Vec<f32> = vec![100.0, 110.0, 90.0, 1.0, 1.1, 0.9, -5.0, 50.0];
        let cp = vec![0.2f32; cd.len()];
        let nanv = vec![f32::NAN; cd.len()];
        crate::db::insert_core_data(&conn, &ids, "RAW", None, &cd, &cp, &ck, &nanv, &nanv).unwrap();

        let db = Mutex::new(conn);
        let res = run_facies_confusion(
            &db,
            &FaciesConfusionRequest {
                well_ids: vec![ids],
                pred_curve: "RT_LOG".into(),
                ref_curve: "RT_REF".into(),
                input_set: None,
                accept_threshold: None,
            },
        );
        assert!(res.error.is_none(), "err={:?}", res.error);
        // 6 valid plugs matched (the 2050 m plug has k<0 AND is out of range anyway).
        assert_eq!(res.n_core_plugs, 6);
        // Tight within-class k, far-apart class means → strong variance reduction.
        assert!(res.k_var_reduction > 0.95, "R²k={}", res.k_var_reduction);
        // Identical curves → perfect purity, untouched by the core extension.
        assert!((res.overall_purity - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_core_plug_with_no_log_sample_in_tolerance_is_counted_not_absorbed() {
        use duckdb::Connection;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        crate::db::insert_well(&conn, id, "FT-2", None, None, None).unwrap();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn,
            id,
            depth.clone(),
            vec![40.0; n],
            vec![10.0; n],
            vec![0.2; n],
            vec![2.4; n],
            vec![80.0; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let ids = id.to_string();
        let rt: Vec<f32> = (0..n).map(|i| if i < 10 { 1.0 } else { 2.0 }).collect();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_LOG", &rt).unwrap();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_REF", &rt).unwrap();
        // Two plugs inside the logged interval, three valid-k plugs far below it.
        let cd: Vec<f32> = vec![2000.2, 2006.2, 2100.0, 2200.0, 2300.0];
        let ck: Vec<f32> = vec![100.0, 1.0, 50.0, 60.0, 70.0];
        let cp = vec![0.2f32; cd.len()];
        let nanv = vec![f32::NAN; cd.len()];
        crate::db::insert_core_data(&conn, &ids, "RAW", None, &cd, &cp, &ck, &nanv, &nanv).unwrap();

        let db = Mutex::new(conn);
        let res = run_facies_confusion(
            &db,
            &FaciesConfusionRequest {
                well_ids: vec![ids],
                pred_curve: "RT_LOG".into(),
                ref_curve: "RT_REF".into(),
                input_set: None,
                accept_threshold: None,
            },
        );
        assert!(res.error.is_none(), "err={:?}", res.error);
        // Neither side is allowed to swallow the three: they are not in the statistic...
        assert_eq!(res.n_core_plugs, 2);
        // ...and they are not forgotten either. Five plugs went in, five are accounted for.
        assert_eq!(res.n_core_unmatched, 3);
        // The join rule travels with the number it produced.
        assert!(res.core_match_note.contains("NEAREST") && res.core_match_note.contains(" m"));
    }

    /// Codex whole-repository review, P1, second half: adding the cross-datum guard to
    /// `get_core_plugs` achieves nothing here while the caller swallows it.
    ///
    /// `if let Ok(plugs) = ...` turned every read failure into an empty plug list, so a core
    /// delivery quoted on TVDSS produced exactly what a well with no core produces — a NaN
    /// variance reduction over zero plugs — and nothing said which had happened. That is the
    /// build record's own "reported success having done nothing", so the reason now rides on
    /// `core_match_note`, whose stated job is to say how the plugs were put on the log's frame.
    ///
    /// Pinned from both sides: the CONFUSION half of the result must survive untouched, because
    /// class-against-class never reads a plug. Failing the whole analysis would be the opposite
    /// error — withholding an answer that was never in question.
    #[test]
    fn a_core_read_refused_for_its_datum_is_named_in_the_match_note_rather_than_left_as_a_bare_nan() {
        use duckdb::Connection;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = Uuid::new_v4();
        crate::db::insert_well(&conn, id, "FT-DATUM", None, None, None).unwrap();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 2000.0 + i as f32 * 0.5).collect();
        crate::db::insert_standard_curves(
            &conn, id, depth.clone(), vec![40.0; n], vec![10.0; n], vec![0.2; n], vec![2.4; n],
            vec![80.0; n], vec![f32::NAN; n],
        )
        .unwrap();
        let ids = id.to_string();
        let rt: Vec<f32> = (0..n).map(|i| if i < 10 { 1.0 } else { 2.0 }).collect();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_LOG", &rt).unwrap();
        crate::equations::write_computed_curve(&conn, &ids, &depth, "RT_REF", &rt).unwrap();
        // Plugs that WOULD tie perfectly — so a silent skip cannot hide behind "nothing matched".
        let cd: Vec<f32> = vec![2000.2, 2001.2, 2006.2, 2007.2];
        let ck: Vec<f32> = vec![100.0, 120.0, 1.0, 1.2];
        let cp = vec![0.2f32; cd.len()];
        let nanv = vec![f32::NAN; cd.len()];
        crate::db::insert_core_data(&conn, &ids, "RAW", None, &cd, &cp, &ck, &nanv, &nanv).unwrap();
        crate::db::declare_set_datum(&conn, "core_sets", &ids, None, "RAW", "TVDSS").unwrap();

        let db = Mutex::new(conn);
        let req = FaciesConfusionRequest {
            well_ids: vec![ids],
            pred_curve: "RT_LOG".into(),
            ref_curve: "RT_REF".into(),
            input_set: None,
            accept_threshold: None,
        };
        let res = run_facies_confusion(&db, &req);

        // The confusion half is untouched — it never reads a plug.
        assert!(res.error.is_none(), "the class comparison still answers: {:?}", res.error);
        assert!((res.overall_purity - 1.0).abs() < 1e-9, "purity {}", res.overall_purity);

        // The core half withholds, and SAYS SO by well and by datum.
        assert_eq!(res.n_core_plugs, 0, "no plug may be tied across datums");
        assert!(res.k_var_reduction.is_nan(), "and no statistic is reported over none");
        assert!(
            res.core_match_note.contains("FT-DATUM")
                && res.core_match_note.contains("TVDSS")
                && res.core_match_note.contains("MD"),
            "the reason travels with the result, naming the well and both datums: {}",
            res.core_match_note
        );
        // Not counted as unmatched: those plugs were never read, let alone measured against a
        // tolerance, and booking them there would misreport the reason.
        assert_eq!(res.n_core_unmatched, 0, "a refused read is not a failed depth match");
    }
}
