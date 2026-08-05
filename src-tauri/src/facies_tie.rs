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
    /// Reference rock-type curve, e.g. a core-derived RT or a rock-typing RT.
    pub ref_curve: String,
}

#[derive(Debug, Serialize)]
pub struct RefClassRow {
    pub ref_label: i64,
    pub dominant_pred: i64,
    pub purity: f64,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct FaciesConfusionResult {
    pub ref_labels: Vec<i64>,
    pub pred_labels: Vec<i64>,
    /// matrix[i][j] = count where reference == ref_labels[i] and prediction == pred_labels[j].
    pub matrix: Vec<Vec<i64>>,
    pub per_ref: Vec<RefClassRow>,
    /// Σ over reference classes of the dominant-cell count / total pairs.
    pub overall_purity: f64,
    pub n: usize,
    /// ANOVA-style variance reduction of log10(core k) when grouped by the PREDICTED class:
    /// 1 − SS_within/SS_total. 1 = the typing explains all core-perm variance, 0 = none.
    /// NaN (→ JSON null) when no core plugs match or fewer than 2 classes carry plugs.
    pub k_var_reduction: f64,
    /// Core plugs that contributed to `k_var_reduction` (valid k, matched to a predicted class).
    pub n_core_plugs: usize,
    pub error: Option<String>,
}

fn confusion_err(msg: &str) -> FaciesConfusionResult {
    FaciesConfusionResult {
        ref_labels: vec![],
        pred_labels: vec![],
        matrix: vec![],
        per_ref: vec![],
        overall_purity: f64::NAN,
        n: 0,
        k_var_reduction: f64::NAN,
        n_core_plugs: 0,
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
    for (i, &rl) in ref_labels.iter().enumerate() {
        let row = &matrix[i];
        let rowsum: i64 = row.iter().sum();
        let (jmax, &vmax) = row.iter().enumerate().max_by_key(|&(_, &v)| v).unwrap();
        dominant_total += vmax;
        per_ref.push(RefClassRow {
            ref_label: rl,
            dominant_pred: pred_labels[jmax],
            purity: if rowsum > 0 { vmax as f64 / rowsum as f64 } else { 0.0 },
            count: rowsum,
        });
    }
    let overall_purity = dominant_total as f64 / pairs.len() as f64;

    FaciesConfusionResult {
        ref_labels,
        pred_labels,
        matrix,
        per_ref,
        overall_purity,
        n: pairs.len(),
        k_var_reduction: f64::NAN,
        n_core_plugs: 0,
        error: None,
    }
}

/// Max |depth difference| (m) for matching a core plug to the nearest log sample.
const CORE_MATCH_TOL_M: f32 = 1.0;

/// Index of the sample in ascending `depth` nearest to `target`, if within tolerance.
fn nearest_within(depth: &[f32], target: f32) -> Option<usize> {
    if depth.is_empty() || !target.is_finite() {
        return None;
    }
    let pos = depth.partition_point(|&d| d < target);
    let mut best: Option<usize> = None;
    for cand in [pos.wrapping_sub(1), pos] {
        if cand < depth.len() {
            let dd = (depth[cand] - target).abs();
            if dd <= CORE_MATCH_TOL_M && best.map_or(true, |b| dd < (depth[b] - target).abs()) {
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
    {
        let conn = db.lock().unwrap();
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
            if let Ok(plugs) = crate::db::get_core_plugs(&conn, well_id) {
                for plug in plugs {
                    let k = plug.cperm as f64;
                    if !(k.is_finite() && k > 0.0) {
                        continue;
                    }
                    let Some(idx) = nearest_within(&d, plug.depth) else { continue };
                    if idx < pv.len() && (pv[idx] as f64).is_finite() {
                        core_groups.push(((pv[idx] as f64).round() as i64, k.log10()));
                    }
                }
            }
        }
    }
    let mut res = build_confusion(&pairs);
    res.k_var_reduction = variance_reduction(&core_groups);
    res.n_core_plugs = core_groups.len();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // plus one invalid-k plug that must be skipped.
        let cd: Vec<f32> = vec![2000.2, 2001.2, 2002.2, 2005.7, 2006.2, 2007.7, 2050.0];
        let ck: Vec<f32> = vec![100.0, 110.0, 90.0, 1.0, 1.1, 0.9, -5.0];
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
}
