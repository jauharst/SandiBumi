//! Electrofacies tie-in QC (Wave B item 8, increment 2) — cross-tabulate a predicted log-domain
//! rock-type curve (e.g. RT_LOG from `rt_cutoff`) against a reference/core rock-type curve at
//! matched depths, and report the confusion matrix + dominant-class purity (ref_rocktyping_shf.md
//! §Cutoff-based electrofacies tie-in: "accept the mapping if dominant-class purity is above a
//! threshold"). The two curves need NOT share a labelling scheme — rows are reference classes,
//! columns predicted classes, and purity measures how cleanly each reference class maps to one
//! predicted class.

use crate::equations::fetch_curve_frame;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct FaciesConfusionRequest {
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
        error: Some(msg.to_string()),
    }
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
        error: None,
    }
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
    {
        let conn = db.lock().unwrap();
        let names = vec![pred.clone(), refc.clone()];
        for well_id in &req.well_ids {
            let Ok((_d, cols)) = fetch_curve_frame(&conn, well_id, &names) else { continue };
            let (Some(pv), Some(rv)) = (cols.get(&pred), cols.get(&refc)) else { continue };
            let n = pv.len().min(rv.len());
            for i in 0..n {
                let (p, r) = (pv[i] as f64, rv[i] as f64);
                if p.is_finite() && r.is_finite() {
                    pairs.push((r.round() as i64, p.round() as i64));
                }
            }
        }
    }
    build_confusion(&pairs)
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
}
