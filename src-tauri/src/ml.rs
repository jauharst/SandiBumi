//! Machine-learning bridge (Phase 10-4): supervised regression / classification and
//! unsupervised clustering / dimensionality reduction over well-log curves, powered by
//! scikit-learn through the same subprocess protocol as `python_engine.rs` (JSON header
//! line + raw little-endian f32 blocks on stdin/stdout, stderr's last line = the error).
//!
//! Division of labour: Rust owns data plumbing — pooling complete samples across wells,
//! masking missing values, scattering predictions back onto each well's full depth grid
//! (NaN where any input was missing) and writing `computed_curves`. Python owns the
//! models. Supervised tasks fit on labelled TRAIN wells and predict on APPLY wells;
//! unsupervised tasks fit directly on the pooled APPLY samples — which makes clustering
//! field-wide by construction (one model, globally consistent cluster ids).
//!
//! Cluster ids are reordered by ascending mean of the FIRST feature curve, matching the
//! native k-means/GMM facies modules (put GR first → class 0 = cleanest).

use crate::equations::{fetch_curve_frame_from_set, write_computed_curves_versioned};

/// The log set ML output lands in when the caller names none — the value that used to be
/// hardcoded, so an older payload writes exactly where it always did.
const DEFAULT_ML_SET: &str = "ML";
use crate::python_engine::{find_python, hide_console};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// Python side of the bridge. Keep messages ASCII (Windows console encodings) and keep
/// the algorithm ids in sync with the catalog in `src/ui/mlDialog.ts`.
/// ONE definition of every supported supervised estimator, embedded verbatim in the training runner
/// and in the leaderboard runner.
///
/// SB-MLA-026. Declared twice and independently, the two drifted, and every divergence flattered or
/// misrepresented the ranking the user chooses from: the leaderboard ranked `degree = 3` polynomial
/// regression **as a straight line**, ran the gradient-boosting fallback at 100 iterations against
/// the run's 300, and built `SVC` without `probability=True` — not a cosmetic difference, because
/// that flag makes scikit-learn fit internal Platt scaling and changes the estimator. The leaderboard
/// also accepted no parameter map at all, so a user's own hyperparameters were ranked as defaults.
///
/// A ranking of models nobody will fit is not a degraded ranking; it is a ranking of the wrong
/// things, presented cleanly. Syncing two copies would have fixed those three and left the mechanism
/// that produced them, so there is one copy and both runners concatenate it.
const ML_BUILD_MODEL: &str = r#"
EFFECTIVE = {}

# SB-MLA-057. The values a log file uses to mean "no reading". A parameter is a THRESHOLD or a
# LIMIT, and one of these arriving as a threshold is never a threshold - it is an absence that lost
# its type somewhere upstream. They are worth refusing by name because they compute: -999.25 as a
# DBSCAN eps produces one enormous cluster and no error at all.
NULL_SENTINELS = (-999.25, -999.0, -9999.0, -99999.0, -9999.25)

# SB-MLA-030. What a `_PROB` curve actually IS, per estimator. These are not interchangeable and a
# reader cannot tell them apart from the track: a calibrated posterior answers "how likely is this
# class", a Platt-scaled SVM distance answers "how far inside the margin is it", and a k-NN vote
# fraction answers "how many of the seven nearest agreed" - which on k=7 can only ever take seven
# values. The dossier records that both IP and Geolog emit relative-only probabilities and SAY so;
# emitting one under the same convention as a posterior without saying so is the interoperability
# defect this closes.
PROB_MEANING = {
    "rf": "the fraction of trees voting for the winning class - a vote share, not a calibrated "
          "posterior, and it is optimistic near the training data",
    "knn": "the fraction of the k nearest neighbours agreeing on the winning class - it can only "
           "take k+1 distinct values, so it is coarse by construction",
    "gaussian_nb": "the winning class's posterior under the naive-Bayes independence assumption - "
                   "calibrated only to the extent the inputs really are independent, which log "
                   "curves are not",
    "logreg": "the winning class's logistic posterior - the best calibrated of these, and still "
              "conditional on the model being right",
    "svm": "the winning class's Platt-scaled score - a monotone squashing of distance from the "
           "decision boundary fitted by internal cross-validation, NOT a posterior",
}

def P(p, key, default):
    """Read a parameter, and RECORD what was actually used (SB-MLA-001).

    A re-run cannot be reconstructed from a record that omits a value that changed the answer,
    and the value that changed the answer is very often one nobody supplied - `seed` above all,
    which is the single parameter with the largest effect on a clustering result. So every read
    goes through here and every default is recorded AS a default, naming where it came from.
    Reading `P(p, key, default)` directly is the defect this exists to prevent; there should be
    no `p.get` left in either runner.

    SB-MLA-057 is enforced here too, for the same reason: this is the ONE door every parameter
    comes through, so a check here cannot be forgotten by the next parameter somebody adds.
    "No value" is already a distinct state - it returns the declared default and is recorded as
    defaulted - so a missing-data sentinel arriving as a value can only be a mistake.
    """
    v = dict.get(p, key) if p else None
    if v is None or v == "":
        EFFECTIVE[key] = {"value": default, "defaulted": True, "source": "ml.rs build_model default"}
        return default
    if isinstance(v, float) and v != v:
        fail("parameter '" + key + "' is not-a-number. Leave it blank to use the default (" +
             str(default) + ") - blank is a real state here and means 'use the default', which NaN "
             "cannot say.")
    if isinstance(v, (int, float)) and not isinstance(v, bool) and float(v) in NULL_SENTINELS:
        fail("parameter '" + key + "' was given " + str(v) + ", which is a missing-data sentinel, "
             "not a setting. It would compute rather than fail. Leave it blank to use the default (" +
             str(default) + ").")
    EFFECTIVE[key] = {"value": v, "defaulted": False}
    return v

def P_used(key, value):
    """Note the value a parameter was CLAMPED to, beside the one that was asked for. A request
    the code silently narrowed is a parameter the record would otherwise misstate."""
    if key in EFFECTIVE and EFFECTIVE[key].get("value") != value:
        EFFECTIVE[key]["used"] = value

def build_model(task, algo, p, seed):
    p = p or {}
    if task == "regression":
        if algo == "rf":
            from sklearn.ensemble import RandomForestRegressor
            return RandomForestRegressor(n_estimators=int(P(p, "n_estimators", 200)),
                                         max_depth=int(P(p, "max_depth", 0)) or None,
                                         random_state=seed, n_jobs=-1), None
        if algo == "gbdt":
            try:
                from xgboost import XGBRegressor
                return XGBRegressor(n_estimators=int(P(p, "n_estimators", 300)),
                                    learning_rate=float(P(p, "learning_rate", 0.1)),
                                    max_depth=int(P(p, "max_depth", 4)),
                                    random_state=seed, verbosity=0), None
            except ImportError:
                from sklearn.ensemble import HistGradientBoostingRegressor
                return HistGradientBoostingRegressor(max_iter=int(P(p, "n_estimators", 300)),
                                                     learning_rate=float(P(p, "learning_rate", 0.1)),
                                                     max_depth=int(P(p, "max_depth", 4)) or None,
                                                     random_state=seed), \
                    "xgboost not installed - used sklearn HistGradientBoosting (pip install xgboost)"
        if algo == "svr":
            from sklearn.svm import SVR
            return SVR(C=float(P(p, "C", 10.0)), epsilon=float(P(p, "epsilon", 0.1))), None
        if algo == "ann":
            from sklearn.neural_network import MLPRegressor
            hidden = tuple(int(t) for t in str(P(p, "hidden", "64,32")).replace(" ", "").split(",") if t)
            return MLPRegressor(hidden_layer_sizes=hidden or (64, 32),
                                max_iter=int(P(p, "max_iter", 500)), random_state=seed), None
        if algo == "linear":
            from sklearn.linear_model import LinearRegression
            deg = int(P(p, "degree", 1))
            if deg > 1:
                from sklearn.pipeline import make_pipeline
                from sklearn.preprocessing import PolynomialFeatures
                return make_pipeline(PolynomialFeatures(deg), LinearRegression()), None
            return LinearRegression(), None
    elif task == "classification":
        if algo == "svm":
            from sklearn.svm import SVC
            return SVC(C=float(P(p, "C", 10.0)), probability=True, random_state=seed), None
        if algo == "knn":
            from sklearn.neighbors import KNeighborsClassifier
            return KNeighborsClassifier(n_neighbors=int(P(p, "n_neighbors", 7))), None
        if algo == "rf":
            from sklearn.ensemble import RandomForestClassifier
            return RandomForestClassifier(n_estimators=int(P(p, "n_estimators", 200)),
                                          random_state=seed, n_jobs=-1), None
        if algo == "gnb":
            from sklearn.naive_bayes import GaussianNB
            return GaussianNB(), None
        if algo == "logreg":
            from sklearn.linear_model import LogisticRegression
            return LogisticRegression(C=float(P(p, "C", 1.0)), max_iter=1000), None
    return None, None
"#;

// ---------------------------------------------------------------------------
// SB-MLA-035 — a transformed quantity is a distinct quantity
// ---------------------------------------------------------------------------

/// The suffix a log-space prediction carries. It is part of the MNEMONIC, not a flag on the curve:
/// a flag can be lost, ignored or not looked at, and a name travels with the curve into the log
/// view, the LAS export, the workbook and the deck.
pub(crate) const LOG10_SUFFIX: &str = "_LOG10";

/// Marks the back-transform inside the runner's suffix slot. It is NOT a mnemonic suffix and never
/// reaches a curve name: `out_names` maps it to the base name itself, because the back-transform is
/// the quantity the user asked for and the log-space prediction is the derived one. A control
/// character keeps it from ever colliding with a real suffix (`""`, `_PROB`, `1`..`n`).
const BACK_SUFFIX: &str = "\u{1}BACK";

/// The suffix a SPECTRALLY TEXTURED prediction carries (round-3 item 5).
///
/// Jauhar, 2026-08-07, asked for two versions and got a name for each, which is the same argument
/// `LOG10_SUFFIX` makes: the difference between these two curves cannot live in a dialog the reader
/// never saw. `_SIM` rather than `_SPEC` because the property that matters to whoever picks the
/// curve up is not that a spectrum was involved — it is that the detail was SIMULATED. The plain
/// prediction keeps the base name, so the defensible curve is the one you get by default and the
/// textured one has to be asked for by name.
pub(crate) const SIM_SUFFIX: &str = "_SIM";

/// The unit a quantity is in, from wherever the catalog happens to record it.
///
/// Four stores can answer, consulted in order of how SPECIFIC the answer is: a unit DECLARED by
/// whatever wrote the curve (`curve_unit`), then the unit an import recorded for the mnemonic
/// (`curve_meta`), then the unit an equation declared for its output, then the canonical unit of
/// the family the mnemonic belongs to — which is the unit SandiBumi stores that family in, so it is
/// a fact about the store rather than a guess about the data.
///
/// Field-wide rather than per-well on purpose: the transform is applied to ONE pooled training set,
/// so "what unit is this quantity in" is a question about the quantity, not about a well.
fn catalog_unit(conn: &Connection, curve: &str) -> Option<String> {
    let c = curve.trim().to_uppercase();
    if c.is_empty() {
        return None;
    }
    let one = |sql: &str| -> Option<String> {
        conn.query_row(sql, duckdb::params![c], |r| r.get::<_, Option<String>>(0))
            .ok()
            .flatten()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    one("SELECT unit FROM curve_unit WHERE upper(curve_name) = ? AND unit IS NOT NULL LIMIT 1")
        .or_else(|| one("SELECT unit FROM curve_meta WHERE upper(mnemonic) = ? AND unit IS NOT NULL LIMIT 1"))
        .or_else(|| {
            one("SELECT output_units FROM equations WHERE upper(output_curve) = ? AND output_units IS NOT NULL LIMIT 1")
        })
        .or_else(|| crate::curves::family_for(&c).map(|f| f.canonical_unit.to_string()))
}

/// Applies the target transform to the assembled training rows, dropping any sample the transform
/// has no answer for and reporting how many.
///
/// Rows are dropped from `x`, `y` and `groups` TOGETHER. They are three parallel views of the same
/// samples — drop a row from `y` alone and every feature after it belongs to a different depth, and
/// the model fits confidently on scrambled pairs.
///
/// A permeability of exactly 0 is a real reading (a seal) and has no logarithm. It is dropped and
/// COUNTED rather than floored to some small number: a floor is an invented parameter, it would
/// anchor the low end of the fit, and the count is the honest thing to show the user.
fn apply_target_transform(kind: &str, d: usize, x: &mut Vec<f32>, y: &mut Vec<f32>, groups: &mut Vec<f32>) -> usize {
    if kind != "log10" {
        return 0;
    }
    let keep: Vec<bool> = y.iter().map(|v| v.is_finite() && *v > 0.0).collect();
    let dropped = keep.iter().filter(|k| !**k).count();
    if dropped > 0 {
        let mut nx: Vec<f32> = Vec::with_capacity((keep.len() - dropped) * d);
        for (i, k) in keep.iter().enumerate() {
            if *k {
                nx.extend_from_slice(&x[i * d..(i + 1) * d]);
            }
        }
        *x = nx;
        let mut i = 0;
        groups.retain(|_| {
            i += 1;
            keep[i - 1]
        });
        let mut j = 0;
        y.retain(|_| {
            j += 1;
            keep[j - 1]
        });
    }
    for v in y.iter_mut() {
        *v = v.log10();
    }
    dropped
}

/// The unit a transformed prediction is in, and the unit its back-transform is in.
///
/// `log10(mD)` rather than `mD`, and `log10` alone where the target's own unit is unknown — an
/// unknown unit must not be allowed to erase the one thing that IS known about the quantity.
fn transformed_unit(kind: &str, target_unit: Option<&str>) -> String {
    match (kind, target_unit.map(str::trim).filter(|u| !u.is_empty())) {
        ("log10", Some(u)) => format!("log10({u})"),
        ("log10", None) => "log10".to_string(),
        (_, Some(u)) => u.to_string(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// SB-MLA-003 — a saved model identifies the exact training rows
// ---------------------------------------------------------------------------

/// A content fingerprint of the assembled training matrix: the feature names in order, the feature
/// values, the target values, the well index of each row, and the row order.
///
/// `trained_on` plus `n_train` narrows a re-run but does not pin it. The same wells at a later log
/// set version are DIFFERENT ROWS with the same names and very possibly the same count — an edited
/// curve, a re-run of the module that produced the target, a changed mask. A hash is the only record
/// that distinguishes "these are the rows" from "these are the wells", and it is what makes the
/// provenance claim checkable rather than merely asserted.
///
/// FNV-1a/64, written out rather than taken as a dependency. The threat model is an ACCIDENT — two
/// training sets that differ and are reported as the same — not an adversary constructing a
/// collision, so a cryptographic digest would buy nothing a project this size can spend. `DefaultHasher`
/// is explicitly not stable across Rust releases, which for a value written into a project file
/// would mean the same rows hashing differently after a toolchain upgrade.
///
/// Two canonicalisations, both required for "numerically identical must hash identically": every
/// NaN collapses to one bit pattern (an f32 NaN has millions), and −0.0 collapses to 0.0.
fn training_fingerprint(features: &[String], d: usize, x: &[f32], y: &[f32], groups: &[f32]) -> String {
    struct Fnv(u64);
    impl Fnv {
        fn eat(&mut self, bytes: &[u8]) {
            for b in bytes {
                self.0 ^= *b as u64;
                self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        fn val(&mut self, v: f32) {
            // NaN -> one canonical NaN; -0.0 -> 0.0. Without both, two numerically identical
            // matrices can hash differently, and a re-fit that changed nothing reads as a
            // different training set — which would make the whole record untrustworthy.
            let c = if v.is_nan() { f32::NAN } else if v == 0.0 { 0.0 } else { v };
            self.eat(&c.to_le_bytes());
        }
    }
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    // The names ride along because the same numbers under different mnemonics are a different
    // training set — and a feature list reordered is a different model (the ordering contract).
    for f in features {
        h.eat(f.as_bytes());
        h.eat(b"\x1f");
    }
    h.eat(&(d as u64).to_le_bytes());
    h.eat(&(y.len() as u64).to_le_bytes());
    for v in x {
        h.val(*v);
    }
    for v in y {
        h.val(*v);
    }
    for g in groups {
        h.val(*g);
    }
    format!("{:016x}", h.0)
}

// ---------------------------------------------------------------------------
// SB-MLA-009 — blind-well performance travels with the curve
// ---------------------------------------------------------------------------

/// What a curve must be able to say about the model that made it: how well that model performed on
/// data it was not fitted on, by what protocol, and over how many wells.
///
/// A net-pay number computed from a predicted permeability whose blind-well R² was 0.31 is a
/// different claim from one computed from a measured permeability, and nothing downstream can tell
/// which it received unless the curve says so. The cautionary case is a delivered project where a
/// predicted NPHI reached a training correlation of 0.99 against a blind-well range of 0.31–0.70 —
/// a factor of three between the number an analyst sees by default and the number that describes
/// what the curve can actually predict.
///
/// So where no blind evaluation was performed this returns `performed: false` WITH NO NUMBER. The
/// requirement is explicit that the absence must be carried rather than filled: a training metric
/// standing in for a blind one is the 0.99 above, and it is worse than a blank because it reads as
/// an answer.
///
/// Built once, in the fitting run, and stored in the model's metrics — so the apply path copies it
/// verbatim instead of re-deriving it. Two derivations of one fact is two things to keep in step.
fn blind_record(metrics: &serde_json::Value, split: Option<&SplitReport>, task: &str) -> serde_json::Value {
    let clf = task == "classification";
    let key = if clf { "accuracy_blind" } else { "r2_blind" };
    let name = if clf { "accuracy" } else { "R2" };
    let value = metrics.get(key).and_then(|v| v.as_f64());
    match (split, value) {
        (Some(sp), Some(v)) => serde_json::json!({
            "performed": true,
            "metric": name,
            "value": v,
            // The protocol is part of the claim, not a footnote. A random-row split scores the
            // model on depths a few centimetres from ones it was fitted on, so its number does not
            // answer "will this work on the next well" — and quoting it as if it did is the whole
            // reason this field exists.
            "protocol": if sp.mode == "sample" { "random rows, stratified" } else { "whole wells" },
            "answers_new_well": sp.mode != "sample",
            "n_blind_wells": sp.blind_wells.len(),
            "n_blind_rows": sp.blind_rows,
            "n_fit_rows": sp.fit_rows,
            "seed": sp.seed,
        }),
        _ => serde_json::json!({
            "performed": false,
            "why": "no blind test was requested for this fit, so nothing here describes how the model travels to data it was not fitted on. The cross-validation score is over folds of the same wells.",
        }),
    }
}

// ---------------------------------------------------------------------------
// SB-MLA-010 — the deliverable carries the ML provenance block
// ---------------------------------------------------------------------------

/// One ML-produced log set live on a well, described the way a deliverable has to describe it.
///
/// Every field is a string because this is a ROW in a report table, not a computation — and because
/// the alternative, letting each renderer format its own numbers, is how the PDF and the Word twin
/// come to disagree about the same study.
#[derive(Debug, Clone)]
pub struct MlProvenanceRow {
    pub curves: String,
    pub model: String,
    pub algorithm: String,
    /// ORDERED. The order is part of the apply contract, so a provenance block that reordered it
    /// would document a model nobody could rebuild.
    pub features: String,
    pub target: String,
    pub training: String,
    /// The SB-MLA-009 statement, in words — including "not blind-tested", which is the case this
    /// block exists for.
    pub blind: String,
    pub log_set: String,
    pub run_date: String,
    pub train_hash: String,
}

/// Renders the blind record as the sentence a deliverable prints.
///
/// The Rust counterpart of `mlDialog.ts::readBlind`'s tooltip wording — the two are deliberately
/// separate because a screen tooltip and a printed report are different registers, but they must
/// agree on the FACTS, and above all on the same refusal: where nothing was held back, both say so
/// and neither shows a training score in its place.
fn blind_sentence(blind: Option<&serde_json::Value>) -> String {
    let Some(b) = blind else {
        return "not recorded (this curve predates the blind-performance record)".into();
    };
    if b.get("performed").and_then(|v| v.as_bool()) != Some(true) {
        return "not blind-tested - nothing was held back, so there is no measurement of how this model performs on data it has not seen".into();
    }
    let metric = b.get("metric").and_then(|v| v.as_str()).unwrap_or("score");
    let value = b.get("value").and_then(|v| v.as_f64()).map(|v| format!("{v:.3}")).unwrap_or_else(|| "-".into());
    let wells = b.get("n_blind_wells").and_then(|v| v.as_u64()).unwrap_or(0);
    let rows = b.get("n_blind_rows").and_then(|v| v.as_u64()).unwrap_or(0);
    if b.get("answers_new_well").and_then(|v| v.as_bool()) == Some(true) {
        format!("{metric} {value} on {wells} well(s) held back whole ({rows} samples the model never saw)")
    } else {
        // Said in full every time. A reader who takes this for a whole-well number reads it as an
        // answer to "will this work on the next well", which it is not.
        format!("{metric} {value} on {rows} random rows drawn from the same wells the model trained on - not a measure of transfer to a new well")
    }
}

/// Every ML-produced log set whose curves are CURRENTLY live on this well.
///
/// Driven from `computed_curves.set_id`, so it reports what the report will actually print rather
/// than every ML run the well has ever seen — a superseded version is not in the deliverable and
/// listing it would be a provenance block describing somebody else's numbers.
///
/// This is the point of the whole provenance group: a parameter that carries the paper it came from,
/// through the computation, into the deliverable. Until now the lineage stopped at the database.
/// One live curve that names a saved model as the thing that produced it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelCitation {
    pub well_name: String,
    pub set_name: String,
    pub curves: Vec<String>,
}

/// Which delivered curves would be orphaned by deleting this model (SB-MLA-007).
///
/// A saved model is the answer to "which model produced this curve", and that is the entire reason
/// artifacts exist here. Deleting one silently does not corrupt anything — the curve keeps its
/// numbers — it does something quieter and worse: the curve goes on citing a model id that resolves
/// to nothing, so the provenance block in a delivered report names a model nobody can produce. The
/// failure surfaces in front of a client, months later, as a question that cannot be answered.
///
/// Driven from `computed_curves.set_id` like `ml_provenance`, so it counts what a deliverable would
/// actually PRINT rather than every run the project has ever seen. A superseded version is not in
/// the deliverable, and refusing a deletion to protect a curve nobody will ever read would make this
/// check the thing people learn to force past.
pub fn model_citations(conn: &Connection, model_id: &str) -> Vec<ModelCitation> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT w.well_name, ls.set_name, ls.set_id
         FROM log_sets ls
         JOIN wells w ON w.well_id = ls.well_id
         WHERE ls.module LIKE 'ml:%'
           AND ls.params_json LIKE ?1
           AND EXISTS (SELECT 1 FROM computed_curves cc WHERE cc.set_id = ls.set_id)
         ORDER BY w.well_name, ls.set_name",
    ) else {
        return Vec::new();
    };
    // Matched on the id as it appears in the recorded JSON. A LIKE rather than a JSON extract
    // because the reference sits at two different depths - the ordinary path writes `model_id` at
    // the top level, the coverage path records one per segment - and a query that knew only one
    // shape would silently report "not cited" for the other.
    let needle = format!("%\"{model_id}\"%");
    let rows = stmt.query_map(duckdb::params![needle], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    });
    let Ok(rows) = rows else { return Vec::new() };

    let mut out = Vec::new();
    for (well_name, set_name, set_id) in rows.flatten() {
        let curves: Vec<String> = conn
            .prepare("SELECT DISTINCT curve_name FROM computed_curves WHERE set_id = ?1 ORDER BY curve_name")
            .and_then(|mut s| {
                s.query_map(duckdb::params![set_id], |r| r.get::<_, String>(0))
                    .map(|it| it.flatten().collect())
            })
            .unwrap_or_default();
        if !curves.is_empty() {
            out.push(ModelCitation { well_name, set_name, curves });
        }
    }
    out
}

pub fn ml_provenance(conn: &Connection, well_id: &str) -> Vec<MlProvenanceRow> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT ls.set_id, ls.set_name, ls.module, ls.params_json, ls.inputs_json,
                strftime(ls.created_at, '%Y-%m-%d %H:%M')
         FROM log_sets ls
         WHERE ls.module LIKE 'ml:%' AND ls.well_id = ?1
           AND EXISTS (SELECT 1 FROM computed_curves cc WHERE cc.set_id = ls.set_id)
         ORDER BY ls.created_at",
    ) else {
        return Vec::new();
    };
    let rows = stmt.query_map(duckdb::params![well_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, String>(5)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };

    let mut out = Vec::new();
    for row in rows.flatten() {
        let (set_id, set_name, module, params_json, inputs_json, created_at) = row;
        let p: serde_json::Value =
            params_json.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(serde_json::Value::Null);
        let curves: Vec<String> = conn
            .prepare("SELECT DISTINCT curve_name FROM computed_curves WHERE set_id = ?1 ORDER BY curve_name")
            .and_then(|mut s| {
                s.query_map(duckdb::params![set_id], |r| r.get::<_, String>(0))
                    .map(|it| it.flatten().collect())
            })
            .unwrap_or_default();
        if curves.is_empty() {
            continue;
        }
        let features: Vec<String> =
            inputs_json.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();

        // The model record answers "how much rock trained this", which the curve's own row cannot.
        // A model that has since been deleted leaves the name still on the curve, and that is the
        // truth: the curve was made by it, and it is gone.
        let model_id = p.get("model_id").and_then(|v| v.as_str());
        let info = model_id.and_then(|id| crate::db::get_ml_model(conn, id).ok().map(|(i, _)| i));
        // SB-MLA-007's second half: a curve whose model has been force-deleted must SAY the
        // reference is unresolvable. Printing the name alone reads as a live reference, and a
        // deliverable that names a model nobody can produce asserts an audit trail it cannot
        // honour - which is the whole hazard the deletion guard exists for.
        //
        // Derived HERE, at read time, rather than stamped onto the citing rows when the deletion
        // happens. Two reasons. A stamp can be missed - a project restored from a backup taken
        // before the deletion carries the curve and not the mark - whereas resolving the id every
        // time cannot go stale. And params_json is the RUN RECORD, a statement of what was
        // configured when the run happened; editing it afterwards to describe a later event is the
        // same category of error as the one being guarded against.
        let unresolved = model_id.is_some() && info.is_none();
        let model = match p.get("model_name").and_then(|v| v.as_str()) {
            Some(name) if unresolved => format!(
                "{name} - DELETED from this project, so this curve cannot be re-applied or re-examined"
            ),
            Some(name) => name.to_string(),
            None if unresolved => {
                "a model that has since been DELETED from this project (its name was not recorded)".into()
            }
            None => "not kept (this fit was not saved as a model)".into(),
        };
        let target = info
            .as_ref()
            .and_then(|i| i.target_curve.clone())
            .or_else(|| p.get("target").and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or_else(|| "-".into());
        let training = match &info {
            Some(i) => format!("{} samples from {} well(s)", i.n_train, i.trained_on.len()),
            None => p
                .get("trained_on")
                .and_then(|v| v.as_array())
                .map(|a| format!("{} well(s)", a.len()))
                .unwrap_or_else(|| "not recorded".into()),
        };
        // SB-MLA-011. Appended rather than given its own column, so the fact reaches the PDF, the
        // Word twin and the workbook without changing a table shape four renderers agree on. It
        // belongs beside the training description because it qualifies it: "300 samples from 8
        // wells" reads very differently once you know THIS well was not one of them.
        let training = match p.get("well_role").and_then(|v| v.as_str()) {
            Some(role) => format!("{training}; this well: {role}"),
            None => training,
        };
        out.push(MlProvenanceRow {
            curves: curves.join(", "),
            model,
            algorithm: module.strip_prefix("ml:").unwrap_or(&module).to_string(),
            features: if features.is_empty() { "-".into() } else { features.join(", ") },
            target,
            training,
            blind: blind_sentence(p.get("blind")),
            log_set: set_name,
            run_date: created_at,
            train_hash: p
                .get("train_hash")
                .and_then(|v| v.as_str())
                .or(info.as_ref().and_then(|i| i.train_hash.as_deref()))
                .unwrap_or("not recorded")
                .to_string(),
        });
    }
    out
}

/// The provenance block's column headings, HERE rather than in each renderer.
///
/// Same argument as `office.rs`'s shared `Sheet`s: the PDF and the Word twin are two renderings of
/// ONE decision, not two implementations that can drift. A reader comparing the two documents for a
/// study is comparing them because something disagreed, and a column that says "Trained on" in one
/// and "Training data" in the other is a disagreement about wording pretending to be one about facts.
pub const ML_PROV_HEADERS: [&str; 6] = [
    "Curve(s)",
    "Model / algorithm",
    "Inputs, in order",
    "Trained on",
    "Blind performance",
    "Log set / date / rows",
];

/// The requirement's binding sentence, printed rather than assumed.
///
/// SB-MLA-010: a report MUST NOT present a model-derived curve as though it were measured or
/// deterministically computed. A reader cannot tell from a track — a predicted PERM is a smooth,
/// plausible curve — and by the time the number has been through a net-pay cutoff and a volumetric,
/// nobody downstream can either. ASCII on purpose: the PDF writer is Helvetica/WinAnsi and replaces
/// every non-ASCII character with a hyphen, so an em dash here would print as one anyway while the
/// Word twin kept the dash — the same sentence, set differently, in two documents of one study.
pub const ML_PROV_CAVEAT: &str = "The curves listed below were PREDICTED by a fitted model, not \
     measured and not computed by a deterministic petrophysical equation. Every number derived from \
     them - net pay, porosity-thickness, hydrocarbon pore volume - inherits the blind performance \
     stated here.";

impl MlProvenanceRow {
    /// The six printed cells, in `ML_PROV_HEADERS` order.
    ///
    /// Multi-line cells: a renderer wraps on `\n`, so the composition lives here and not in the two
    /// (soon three) places that draw it.
    pub fn cells(&self) -> [String; 6] {
        [
            // The target is named beside the curve, because "PERM_EST" alone does not say what
            // quantity it stands in for — and what it stands in for is the measured thing the
            // reader will otherwise assume it is.
            if self.target == "-" {
                self.curves.clone()
            } else {
                format!("{}\n(a prediction of {})", self.curves, self.target)
            },
            format!("{}\n{}", self.model, self.algorithm),
            self.features.clone(),
            self.training.clone(),
            self.blind.clone(),
            format!("{}\n{}\n{}", self.log_set, self.run_date, self.train_hash),
        ]
    }
}

/// SB-MLA-023 / SB-MLA-024 — the product's k-means and seed definitions, EMITTED into the Python
/// runners from the Rust constants that the native engine runs on.
///
/// `facies.rs` holds the definition because it holds the implementation; this function is what stops
/// there being a second copy of it written in Python. Restating `n_init=10, max_iter=300` in the
/// runner text would compile, run and look right — and would go stale the day somebody changed the
/// native side, which is exactly the history this requirement exists to end.
///
/// Emitted as module-level Python names rather than substituted at each call site so that the value
/// appears once in the generated source too, and so the conformance test has something to read.
fn ml_shared_constants_py() -> String {
    format!(
        "# Generated from facies.rs - SandiBumi's one k-means definition (SB-MLA-023).\n\
         KMEANS_N_INIT = {}\n\
         KMEANS_MAX_ITER = {}\n\
         KMEANS_TOL = {:e}\n\
         SEED_DEFAULT = {}\n\
         # SB-MLA-021 - the class code for a sample an algorithm REJECTED, as opposed to one it was\n\
         # never given. Emitted rather than written here so the runner, the log-view block track and\n\
         # the print path cannot disagree about which code means 'not one of the clusters'.\n\
         CLUSTER_REJECT = {}\n\
         # Round-3 item 5 - the suffix the spectrally textured prediction is emitted under. Emitted\n\
         # so the runner cannot spell it differently from the name resolver that has to place it.\n\
         SIM_SUFFIX = \"{}\"\n",
        crate::facies::KMEANS_RESTARTS,
        crate::facies::KMEANS_MAX_ITERS,
        crate::facies::KMEANS_TOL,
        crate::facies::SEED_DEFAULT as i64,
        CLUSTER_REJECT,
        SIM_SUFFIX,
    )
}

/// SB-MLA-021 — the class code meaning "this sample was evaluated and belongs to no cluster".
///
/// Distinct from `NaN`, which in a class curve now means one thing only: never evaluated. A sample
/// DBSCAN rejects was measured, standardized and tested, and found not to belong to anything — that
/// is a finding about the rock, and storing it as missing throws the finding away.
///
/// Negative on purpose. Cluster ids run `0..K-1` ordered by ascending first-feature mean, so a reject
/// class appended after them would sit at the shaly end of an ordering it is not part of, and anyone
/// averaging a curve by facies code would read it as the shaliest rock in the well. A negative sorts
/// below every cluster and belongs to no part of the ramp.
///
/// Both renderers treat ANY negative class as rejected rather than testing this exact value, so the
/// display cannot silently mis-colour a code it does not recognise.
pub const CLUSTER_REJECT: i64 = -1;

/// SB-MLA-005 — the runtime probe, shared by the fitting runner and the apply runner.
///
/// ONE definition, because the whole point is to compare a model's recorded runtime against the one
/// about to load it, and two probes that named their components differently would report a mismatch
/// between `scikit-learn` and `sklearn` on an identical machine.
///
/// The set is not arbitrary. `joblib` is the SERIALISER — a pickle written by one version and read
/// by another is precisely the failure this record exists to name, and it is the component most
/// often overlooked because nobody thinks of it as participating in the fit. `scipy` is in because
/// scikit-learn's solvers reach into it, `numpy` because the arrays are its, and the interpreter
/// itself because a pickle protocol is a property of it.
///
/// `xgboost` is in for a different reason from the rest: when it is present it IS the estimator for
/// `gbdt`, so a model fitted under it and applied under a later one is exactly the case the
/// requirement names — "every library that participated in fitting". When it is absent the runner
/// substitutes a scikit-learn estimator (see SB-MLA-012), and its absence from the record is then
/// correct rather than a gap.
///
/// A missing package is written as an explicit JSON **null**, never omitted and never `""`. The three
/// states are genuinely different and the comparison needs all of them: a KEY THAT IS ABSENT means
/// this build never probed that component, so nothing can be said about it; `null` means it was
/// probed and was not installed; a string is the version. Only the middle one lets the drift check
/// report the case that matters most here — a model fitted with no `xgboost` (and therefore, per
/// SB-MLA-012, a substituted scikit-learn estimator) now being applied on a machine that has it.
/// An empty string would read as "version unknown", which calls for a different response again.
///
/// The probe IMPORTS the module rather than asking `importlib.metadata` for a distribution version:
/// the distribution is `scikit-learn` and the module is `sklearn`, and a probe naming it the first
/// way while the runner named it the second would report a mismatch on an identical machine.
const ML_RUNTIME_PY: &str = r#"
def _runtime():
    import sys as _s
    out = {"python": "%d.%d.%d" % _s.version_info[:3]}
    for _n in ("numpy", "scipy", "sklearn", "joblib", "xgboost"):
        try:
            out[_n] = __import__(_n).__version__
        except Exception:
            out[_n] = None
    return out
"#;

/// The training runner: the shared constants, the runtime probe, the shared estimator definitions,
/// then the fit-and-apply body.
fn ml_runner() -> String {
    format!("{}{ML_RUNTIME_PY}{ML_BUILD_MODEL}{ML_RUNNER_BODY}", ml_shared_constants_py())
}

/// The leaderboard runner, from the SAME estimator definitions. Composing both from one fragment is
/// what makes SB-MLA-026 structural rather than a pair of copies somebody has to remember to sync.
fn ml_eval_runner() -> String {
    format!("{}{ML_BUILD_MODEL}{ML_EVAL_RUNNER_BODY}", ml_shared_constants_py())
}

/// The apply runner, carrying the same runtime probe so its answer can be compared with the model's.
fn ml_apply_runner() -> String {
    format!("{ML_RUNTIME_PY}{ML_APPLY_RUNNER}")
}

const ML_RUNNER_BODY: &str = r#"
import sys, json
import numpy as np

def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
task = header["task"]; algo = header["algorithm"]; p = header["params"] or {}
d = header["d"]; n_train = header["n_train"]; has_y = header["has_target"]; n_apply = header["n_apply"]
save_model = bool(header.get("save_model", False))
feature_names = header.get("features") or []
has_groups = bool(header.get("has_groups", False))
# The blind set arrives as a per-ROW mask in the binary payload, not as a list of well indices.
# Both split modes reduce to the same thing here - holding out whole wells is one particular row
# mask - so the runner has one code path and cannot behave differently for the two. Sending it as
# a payload column rather than a JSON list also keeps the header small when 30% of 200 000 rows
# are held out.
has_blind = bool(header.get("has_blind", False))
total = (n_train * d + (n_train if has_y else 0) + n_apply * d
         + (n_train if has_groups else 0) + (n_train if has_blind else 0))
raw = sys.stdin.buffer.read(4 * total)
if len(raw) != 4 * total:
    fail("truncated input stream")
off = 0
def take(count):
    global off
    a = np.frombuffer(raw, dtype=np.float32, count=count, offset=4 * off).copy()
    off += count
    return a
X = take(n_train * d).reshape(n_train, d).astype(np.float64)
y = take(n_train).astype(np.float64) if has_y else None
A = take(n_apply * d).reshape(n_apply, d).astype(np.float64)
# One well index per training row. CROSS-VALIDATION always holds out whole WELLS with it, in both
# split modes and deliberately: a random fold puts the same well on both sides, and the model has
# then seen the interval it is being scored on. So a sample-mode run still gets one score that
# cannot leak, beside the sample-blind score it asked for.
groups = take(n_train).astype(np.int64) if has_groups else None
# Which rows are blind was decided in Rust - see split_blind_wells / split_blind_samples. The
# runner never draws the split itself: it has to be reported and re-runnable whatever the
# subprocess does with it.
blind = take(n_train) > 0.5 if has_blind else None
fit_rows = ~blind if blind is not None else None

try:
    import sklearn  # noqa: F401
except ImportError:
    fail("scikit-learn is not installed for this Python - run: pip install scikit-learn")
from sklearn.preprocessing import StandardScaler

seed = int(P(p, "seed", SEED_DEFAULT))
supervised = task in ("regression", "classification")
metrics = {}
if bool(P(p, "standardize", True)):
    # Fitted on the FIT rows only when a blind split is in force. A scaler that has seen the
    # blind wells' mean and scale makes the blind score optimistic by construction - the same
    # leak SB-MLA-028 closed in the leaderboard, and it would arrive here through the back door.
    basis = X[fit_rows] if (supervised and fit_rows is not None) else (X if supervised else A)
    scaler = StandardScaler().fit(basis)
    Xs = scaler.transform(X) if n_train else X
    As = scaler.transform(A) if n_apply else A
    # SB-MLA-034 / SB-MLA-032. The pre-transform is ANNOUNCED, and so is the basis it was fitted on.
    # Standardisation is not cosmetic: it is what makes a DBSCAN eps meaningful, what stops a
    # resistivity in ohm-m dominating a porosity in v/v on any distance-based method, and it is
    # fitted on a particular set of rows - so the same model applied to a different well set is
    # standing on a different mean and scale. A user reading "eps = 0.5" needs to know that 0.5 is in
    # standard deviations of THIS basis.
    metrics["pre_transform"] = (
        "inputs standardised to zero mean and unit variance, fitted on %s (%d row(s)). Any parameter "
        "in distance units - DBSCAN eps above all - is in standard deviations of that basis, not in "
        "the curves' own units." % (
            "the FIT rows only, so the blind wells' mean and scale never reach the model"
            if (supervised and fit_rows is not None) else
            ("the training rows" if supervised else "the wells being clustered"),
            int(len(basis)),
        )
    )
    metrics["standardize_basis_mean"] = [round(float(v), 6) for v in np.atleast_1d(scaler.mean_)]
    metrics["standardize_basis_scale"] = [round(float(v), 6) for v in np.atleast_1d(scaler.scale_)]
else:
    scaler = None
    Xs, As = X, A
    # Stated as a choice rather than left as an absence: on any distance-based method this is the
    # difference between clustering rock and clustering whichever curve has the largest numbers.
    metrics["pre_transform"] = (
        "inputs used in their own units, NOT standardised - on a distance-based method (k-means, "
        "GMM, DBSCAN, k-NN, SVM) the curve with the largest numeric range will dominate the result"
    )

def fit_xy(yv):
    """The rows the model is allowed to learn from. Everything else is being kept honest."""
    return (Xs[fit_rows], yv[fit_rows]) if fit_rows is not None else (Xs, yv)

def name_protocol(key, sentence):
    """SB-MLA-027 - a score is a claim, and a claim without its protocol is not checkable.

    R-squared over the fitted rows, over folds of the same wells, and over wells the model never saw
    are three different numbers that answer three different questions, and they are routinely quoted
    as one. Kept as DATA next to the score rather than as prose in a note, so a renderer that prints
    the number can always find the sentence that qualifies it.
    """
    metrics.setdefault("score_protocols", {})[key] = sentence

def cv_score(model, scoring, key):
    """Validation score over the FIT wells.

    Folds are whole wells (GroupKFold) whenever the caller supplied groups. A plain `cv=5`
    splits pooled samples, so consecutive depths from one well land on both sides of the fold
    and the model is scored on rock it has already seen a metre away - the number that comes
    back is a smoothness measure, not a validation. The scaler is refitted inside each fold
    for the same reason.
    """
    Xf, yf = fit_xy(y if scoring == "r2" else y.astype(int))
    if len(yf) < 30:
        return
    try:
        from sklearn.model_selection import cross_val_score, GroupKFold, KFold
        from sklearn.pipeline import make_pipeline
        gf = groups[fit_rows] if (groups is not None and fit_rows is not None) else groups
        ng = int(len(np.unique(gf))) if gf is not None else 0
        est = make_pipeline(StandardScaler(), model) if scaler is not None else model
        if ng >= 2:
            nsp = min(5, ng)
            sc = cross_val_score(est, X[fit_rows] if fit_rows is not None else X, yf,
                                 cv=GroupKFold(n_splits=nsp), groups=gf, scoring=scoring)
            metrics[key] = float(np.mean(sc))
            metrics[key + "_folds"] = "%d wells held out one at a time" % nsp if nsp == ng else "%d well groups" % nsp
            name_protocol(key, "whole wells held out (%s) - this answers 'will it work on the next well'"
                          % metrics[key + "_folds"])
        else:
            # One well: there is no blind fold to be had, and saying so is the point.
            sc = cross_val_score(est, X if fit_rows is None else X[fit_rows], yf, cv=KFold(n_splits=5, shuffle=True, random_state=seed), scoring=scoring)
            metrics[key] = float(np.mean(sc))
            metrics[key + "_folds"] = "random folds within ONE well - not a blind score"
            # SB-MLA-019. The protocol DEGRADED, and the number it produced sits under the same key a
            # sound protocol would have used. Random folds within one well score the model on rock a
            # few centimetres from rock it was fitted on, so the number is a smoothness measure, not
            # a validation - and it reads HIGH, which is the wrong direction for a caveat to fail in.
            # Flagged as data rather than left in prose so a renderer cannot print the score without
            # being able to find the qualification.
            metrics["cv_degraded"] = True
            metrics[key + "_degraded"] = True
            name_protocol(key, "random folds inside ONE well - the model is scored on rock centimetres "
                               "from rock it was fitted on, so this measures smoothness, not validity, "
                               "and it reads higher than a real blind score would")
    except Exception as e:
        metrics["cv_error"] = str(e)

def blind_score(model, kind):
    """Score on the rows held out of the fit."""
    if blind is None or not int(np.sum(blind)):
        return
    Xb, yb = Xs[blind], y[blind]
    metrics["n_blind"] = int(np.sum(blind))
    metrics["n_fit"] = int(np.sum(fit_rows))
    if groups is not None:
        metrics["n_blind_wells"] = int(len(np.unique(groups[blind])))
        metrics["n_fit_wells"] = int(len(np.unique(groups[fit_rows])))
    # SB-MLA-027. Whole wells and drawn rows are not the same claim, and only the first answers
    # "will this work on the next well" - a row split leaves the held-out samples centimetres from
    # fitted ones. The protocol is stated with the score rather than inferred from the split mode.
    whole_wells = False
    if groups is not None:
        # Disjoint well sets is the property that matters, not the requested mode: it is what makes
        # the held-out rows rock the model has never been near.
        whole_wells = not (set(np.unique(groups[blind])) & set(np.unique(groups[fit_rows])))
    if whole_wells:
        blind_protocol = (
            "%d row(s) from %d WHOLE well(s) the model never saw - this answers 'will it work on the "
            "next well'" % (int(np.sum(blind)), int(metrics.get("n_blind_wells", 0)))
        )
    else:
        blind_protocol = (
            "%d row(s) drawn out of wells the model was also fitted on - held-out samples sit "
            "centimetres from fitted ones, so this reads higher than a whole-well score would"
            % int(np.sum(blind))
        )
    # How alike the two sides are, per feature and on the target. This is the evidence for
    # "similar statistics", and it is reported rather than asserted: a stratified draw is
    # SUPPOSED to make these match, so a pair that does not match is the signal that the strata
    # were wrong - a class with three samples in it cannot be split representatively.
    try:
        cmp = []
        for j, nm in enumerate(feature_names or [("x%d" % j) for j in range(d)]):
            cf, cb = X[fit_rows, j], X[blind, j]
            cmp.append({"name": nm, "fit_mean": float(np.mean(cf)), "blind_mean": float(np.mean(cb)),
                        "fit_sd": float(np.std(cf)), "blind_sd": float(np.std(cb))})
        yf_, yb_ = y[fit_rows], y[blind]
        cmp.append({"name": "(target)", "fit_mean": float(np.mean(yf_)), "blind_mean": float(np.mean(yb_)),
                    "fit_sd": float(np.std(yf_)), "blind_sd": float(np.std(yb_))})
        metrics["split_balance"] = cmp
    except Exception:
        pass
    try:
        if kind == "r2":
            pb = model.predict(Xb)
            ss_res = float(np.sum((yb - pb) ** 2)); ss_tot = max(float(np.sum((yb - np.mean(yb)) ** 2)), 1e-12)
            metrics["r2_blind"] = 1.0 - ss_res / ss_tot
            metrics["rmse_blind"] = float(np.sqrt(np.mean((yb - pb) ** 2)))
            for kk in ("r2_blind", "rmse_blind"):
                name_protocol(kk, blind_protocol)
        else:
            metrics["accuracy_blind"] = float(np.mean(model.predict(Xb) == yb.astype(int)))
            name_protocol("accuracy_blind", blind_protocol)
    except Exception as e:
        metrics["blind_error"] = str(e)

SILHOUETTE_CAP = 5000

def note_convergence(n_iter, cap, converged):
    """SB-MLA-016 - did the fit STOP, or did it merely run out of iterations?

    Two runs that both return labels, plot identically, and mean completely different things. An
    exhausted fit is a partial answer presented as a final one, and scikit-learn's own signal for it
    is a warning nobody sees from a subprocess.
    """
    metrics["converged"] = bool(converged)
    metrics["n_iter"] = int(n_iter)
    metrics["max_iter"] = int(cap)
    if not converged:
        metrics["convergence_note"] = (
            "the fit did NOT converge: it stopped after hitting the %d-iteration cap, so this is "
            "where the optimiser had got to and not where it was going. Raise max_iter, or take the "
            "result as provisional." % int(cap)
        )

SPEC_GRID = np.linspace(0.0, 0.5, 257)

def target_spectrum(measured, gf):
    """The measured target's amplitude density, averaged over the fit wells.

    PER WELL, not over the pooled matrix. The fit rows are many wells stacked end to end, and an
    FFT across that stack reads every well boundary as a step - the spectrum would then be
    dominated by the joins, which are an artifact of how the matrix was assembled and not a
    property of any rock. Averaged on a common normalised-frequency grid because the wells have
    different lengths, and normalised per sample so a long well does not outvote a short one.
    """
    dens = []
    wells = np.unique(gf) if gf is not None else [None]
    for w in wells:
        yw = measured if w is None else measured[gf == w]
        yw = yw[np.isfinite(yw)]
        if len(yw) < 32:
            continue
        s = yw - np.mean(yw)
        a = np.abs(np.fft.rfft(s)) / float(len(s))
        dens.append(np.interp(SPEC_GRID, np.fft.rfftfreq(len(s)), a))
    if not dens:
        return None, 0
    return np.mean(np.asarray(dens), axis=0), len(dens)

SPEC_SMOOTH = 5

def smooth_band(power, w):
    """Boxcar-average a periodogram over neighbouring frequencies.

    A raw periodogram FLUCTUATES about the true spectral density - it is an inconsistent estimator,
    its variance does not fall with sample count. About half its bins therefore read low by chance,
    and because the deficit below is rectified at zero, every one of those becomes energy that gets
    ADDED. Left unsmoothed, a prediction that already had exactly its target's resolution came back
    measurably rougher than the log it was matched to.

    The window makes the two sides comparable rather than tuning a result: the target density is
    already averaged over every fit well, while a single segment's periodogram has no averaging at
    all, so one side was a density and the other was noise around one. Measured on synthetic logs at
    widths 1/5/9/17/33 - 5 reproduced the target's roughness to within 0.2%, and wider windows were
    worse in both directions because they flatten the peaks the deficit is measured against.
    """
    if w <= 1:
        return power
    pad = w // 2
    return np.convolve(np.pad(power, pad, mode="edge"), np.ones(w) / float(w), mode="valid")[:len(power)]

def spectral_texture(pred, measured, gf, seed):
    """Round-3 item 5 - give the prediction the frequency content its target has and it lacks.

    A regression predicts the CONDITIONAL MEAN, so it is smooth by construction: it can only carry
    through detail its inputs contain, and a curve read over feet cannot produce detail measured
    over inches. Writing that smooth curve under the target's name overstates what was resolved.

    What this does is add a seeded random-phase realisation whose amplitude spectrum makes up the
    DEFICIT between the measured target's density and the prediction's own. So the result has the
    target's frequency content while keeping the prediction's low frequencies untouched - the
    deficit is zero wherever the prediction already has as much energy as the target, and the DC
    term is forced to zero so the mean never moves.

    **The added detail is not a measurement.** It is one plausible realisation of infinitely many:
    correct in its statistics, arbitrary in its placement. That is why this is off by default, why
    it is written under its OWN curve name, and why the note says so in the words a reader needs -
    no bed in the added detail should be correlated between wells.

    Applied only across gap-free runs of 32+ samples. An FFT spanning a gap reads the gap as
    periodicity and stamps that invented period across the whole segment.
    """
    dens, n_wells = target_spectrum(measured, gf)
    if dens is None:
        return None, "not applied: no fit well has 32+ measured target samples to take a spectrum from"
    out = np.array(pred, dtype=np.float64)
    ok = np.isfinite(out)
    rng = np.random.RandomState(int(seed) & 0x7FFFFFFF)
    applied = 0
    runs = 0
    i = 0
    n = len(out)
    while i < n:
        if not ok[i]:
            i += 1
            continue
        j = i
        while j < n and ok[j]:
            j += 1
        length = j - i
        if length >= 32:
            seg = out[i:j]
            centred = seg - np.mean(seg)
            have = smooth_band(np.abs(np.fft.rfft(centred)) ** 2, SPEC_SMOOTH) ** 0.5
            want = np.interp(np.fft.rfftfreq(length), SPEC_GRID, dens) * float(length)
            deficit = np.sqrt(np.maximum(0.0, want ** 2 - have ** 2))
            deficit[0] = 0.0
            phase = rng.uniform(0.0, 2.0 * np.pi, len(deficit))
            resid = np.fft.irfft(deficit * (np.cos(phase) + 1j * np.sin(phase)), length)
            out[i:j] = seg + resid
            applied += length
            runs += 1
        i = j
    if applied == 0:
        return None, "not applied: no gap-free run of 32 or more samples to take a spectrum across"
    return out.astype(np.float32), (
        "%d sample(s) across %d gap-free run(s) carry ADDED detail, matched to the amplitude "
        "spectrum of the measured target over %d fit well(s). The added detail is consistent with "
        "the target's frequency content and is NOT a measurement: it is one realisation of many, "
        "correct in its statistics and arbitrary in its placement. Do not correlate a bed seen only "
        "in this curve between wells, and quote the plain prediction for anything that has to be "
        "defended sample by sample." % (applied, runs, n_wells)
    )

outs = []
if task == "regression":
    model, build_note = build_model(task, algo, p, seed)
    if model is None:
        fail("unknown regression algorithm '" + algo + "'")
    if build_note:
        metrics["note"] = build_note
    cv_score(model, "r2", "r2_cv")
    Xf, yf = fit_xy(y)
    # NOT refitted on the blind wells afterwards. The blind wells still get their curve, so the
    # prediction there can be laid against core and looked at - which is the whole reason a
    # petrophysicist holds a well back. Refitting would make that curve in-sample and leave the
    # reported score describing a model that no longer exists.
    model.fit(Xf, yf)
    pred = model.predict(Xf)
    ss_res = float(np.sum((yf - pred) ** 2)); ss_tot = max(float(np.sum((yf - np.mean(yf)) ** 2)), 1e-12)
    metrics["r2_train"] = 1.0 - ss_res / ss_tot
    metrics["rmse_train"] = float(np.sqrt(np.mean((yf - pred) ** 2)))
    metrics["n_train"] = n_train
    # SB-MLA-027. The in-sample score. It is the one most likely to be quoted as an answer and the
    # one that answers least: a model with enough capacity drives it toward 1 by memorising, so it
    # measures capacity, not skill.
    for kk in ("r2_train", "rmse_train"):
        name_protocol(kk, "the rows the model was FITTED ON - in-sample, so it measures how much the "
                          "model could memorise and not how it will behave on rock it has not seen")
    blind_score(model, "r2")
    base_pred = model.predict(As).astype(np.float32)
    outs.append(("", base_pred))
    # Round-3 item 5, second half. OFF by default: the plain prediction is the defensible curve,
    # and a textured one that arrived without being asked for would be quoted as a measurement.
    if bool(P(p, "spectral_texture", False)):
        gf_spec = groups[fit_rows] if (groups is not None and fit_rows is not None) else groups
        sim, spec_note = spectral_texture(base_pred, yf, gf_spec, seed)
        if sim is not None:
            outs.append((SIM_SUFFIX, sim))
        metrics["spectral_texture"] = spec_note

elif task == "classification":
    yi = y.astype(int)
    model, build_note = build_model(task, algo, p, seed)
    if model is None:
        fail("unknown classification algorithm '" + algo + "'")
    if build_note:
        metrics["note"] = build_note
    cv_score(model, "accuracy", "accuracy_cv")
    Xf, yf = fit_xy(yi)
    model.fit(Xf, yf)
    metrics["accuracy_train"] = float(np.mean(model.predict(Xf) == yf))
    name_protocol("accuracy_train", "the rows the model was FITTED ON - in-sample, so it measures how "
                                    "much the model could memorise and not how it will behave on rock "
                                    "it has not seen")
    metrics["class_counts"] = {str(c): int(np.sum(yf == c)) for c in np.unique(yf)}
    metrics["n_train"] = n_train
    blind_score(model, "accuracy")
    outs.append(("", model.predict(As).astype(np.float32)))
    outs.append(("_PROB", np.max(model.predict_proba(As), axis=1).astype(np.float32)))
    # SB-MLA-030. A `_PROB` curve is not one quantity across this product, and the differences
    # matter: a calibrated posterior, a distance-derived score squeezed through Platt scaling, and a
    # k-NN vote fraction are read the same way off a track and mean different things. Declared per
    # run, in words, rather than left to a mnemonic that cannot carry the distinction.
    metrics["prob_definition"] = PROB_MEANING.get(
        algo, "the winning class's score from %s, normalised across classes to sum to 1" % algo)
    metrics["prob_normalisation"] = "across the classes at each depth, summing to 1"

elif task == "clustering":
    k = int(P(p, "k", 5))
    prob = None
    # SB-MLA-014. k cannot exceed the number of samples there are to cluster, and scikit-learn
    # would raise rather than explain. Reported as a CLAMP, not silently substituted: "you asked
    # for 12 and got 4" is a fact about the data the user needs, and a run that quietly returned
    # 4 clusters under a request for 12 would be read as 12 clusters that happened to merge.
    if n_apply and k > n_apply:
        k = max(1, int(n_apply))
        P_used("k", k)
        metrics["k_clamped"] = "k was reduced to %d: there are only %d samples to cluster" % (k, n_apply)
    if algo == "kmeans":
        from sklearn.cluster import KMeans
        # SB-MLA-023: n_init / max_iter / tol come from facies.rs, not from scikit-learn's defaults
        # and not from a number typed here. Restart count and iteration cap decide WHICH local
        # optimum k-means lands in, so two engines configured differently are two methods.
        km = KMeans(n_clusters=k, n_init=KMEANS_N_INIT, max_iter=KMEANS_MAX_ITER,
                    tol=KMEANS_TOL, random_state=seed)
        labels = km.fit_predict(As)
        # SB-MLA-016. A run that stopped because it converged and one that stopped because it hit
        # the iteration cap are different results, and both return labels that plot identically.
        # The second is a partial answer presented as a final one.
        note_convergence(int(km.n_iter_), int(KMEANS_MAX_ITER), int(km.n_iter_) < int(KMEANS_MAX_ITER))
    elif algo == "gmm":
        from sklearn.mixture import GaussianMixture
        gm_max = int(P(p, "max_iter", 100))
        gm = GaussianMixture(n_components=k, random_state=seed, max_iter=gm_max).fit(As)
        note_convergence(int(gm.n_iter_), gm_max, bool(gm.converged_))
        resp = gm.predict_proba(As)
        labels = np.argmax(resp, axis=1); prob = np.max(resp, axis=1)
        # SB-MLA-015. A component the fit drove to (almost) no weight is not a cluster the rock has;
        # it is the mixture telling you k was too high. Silently leaving it in the count makes a
        # 6-component answer out of a 5-component one.
        tiny = [int(i) for i, w in enumerate(gm.weights_) if float(w) < 0.01]
        if tiny:
            metrics["degenerate_components"] = (
                "%d of %d mixture component(s) hold under 1%% of the weight - the fit is telling you "
                "k is higher than the data supports" % (len(tiny), k)
            )
    elif algo == "hier":
        from sklearn.cluster import AgglomerativeClustering
        # SB-MLA-046. The linkage names are scikit-learn's own enumeration, and 'ward' is its
        # default; it is the only one that minimises within-cluster variance, which is the same
        # criterion facies.rs's k-means and hfu.rs's Ward partition use - so it is the choice that
        # keeps the three consistent rather than an arbitrary pick.
        link = str(P(p, "linkage", "ward"))
        if link not in ("ward", "complete", "average", "single"):
            fail("unknown linkage '" + link + "' - one of: ward, complete, average, single")
        labels = AgglomerativeClustering(n_clusters=k, linkage=link).fit_predict(As)
    elif algo == "dbscan":
        from sklearn.cluster import DBSCAN
        eps = float(P(p, "eps", 0.5))
        # SB-MLA-053. `eps` is a DISTANCE, and what one unit of it means is decided entirely by the
        # pre-transform. Standardised, it is a multiple of a standard deviation and the same 0.5
        # means the same thing on any field. Un-standardised, it is in the mixed units of whatever
        # curves were picked - a deep resistivity in ohm-m and a porosity in v/v are three orders of
        # magnitude apart, so the resistivity alone decides every neighbourhood and the porosity
        # contributes nothing. The result is not an error; it is one huge cluster, or noise
        # everywhere, and nothing says why.
        #
        # The name stays `eps` because that is scikit-learn's own and renaming it would fork the
        # vocabulary. What it multiplies is DECLARED instead, and the meaningless case is called out
        # rather than left for the user to infer from a bad answer.
        metrics["eps_unit"] = (
            "standard deviations of the standardisation basis" if scaler is not None
            else "the RAW mixed units of the selected curves"
        )
        if scaler is None:
            metrics["eps_warning"] = (
                "eps = %g is being applied in the curves' own units because standardisation is off. "
                "Whichever input has the largest numeric range decides every neighbourhood on its "
                "own, and the others contribute nothing - this usually returns one huge cluster or "
                "noise everywhere, with no error. Turn standardisation on, or set eps in the units "
                "of your largest-range curve deliberately." % eps
            )
        labels = DBSCAN(eps=eps, min_samples=int(P(p, "min_samples", 10))).fit_predict(As)
    else:
        fail("unknown clustering algorithm '" + algo + "'")
    # SB-MLA-021. Real clusters get ids ordered by first-feature mean; a sample the algorithm
    # REJECTED (DBSCAN noise) is written as CLUSTER_REJECT (-1), not as NaN.
    #
    # "this sample is an outlier the model refuses to classify" and "this sample had no RHOB" are
    # different statements about the rock, and leaving both missing conflates them - the rejected
    # sample was measured, evaluated, and found not to belong to anything, which is a finding. NaN
    # in this curve now means one thing only: never evaluated.
    #
    # -1 rather than an id after the clusters, because cluster ids are ordered by ascending
    # first-feature mean and appending the reject class would put it at the shaly end of an ordering
    # it is not part of - anyone averaging a curve by facies code would read it as the shaliest rock
    # in the well. A negative sorts below every cluster and belongs to no part of the ramp.
    ids = [int(c) for c in np.unique(labels) if c >= 0]
    if not ids:
        fail("clustering found no clusters (DBSCAN: widen eps / lower min_samples)")
    order = sorted(ids, key=lambda c: float(np.mean(A[labels == c, 0])))
    remap = {c: i for i, c in enumerate(order)}
    out = np.full(n_apply, np.nan, dtype=np.float32)
    for c, i in remap.items():
        out[labels == c] = i
    n_reject = int(np.sum(labels < 0))
    if n_reject:
        out[labels < 0] = CLUSTER_REJECT
    metrics["cluster_sizes"] = {str(remap[c]): int(np.sum(labels == c)) for c in order}
    if algo == "dbscan":
        metrics["noise_pct"] = round(float(np.mean(labels < 0) * 100), 2)
        metrics["n_rejected"] = n_reject
        metrics["reject_code"] = CLUSTER_REJECT
    # SB-MLA-014, the other half. Fewer clusters came back than were asked for. For k-means that is
    # an empty cluster; for DBSCAN it is the density parameters. Either way the answer is not the
    # one requested and the count is what a reader will quote.
    if algo != "dbscan" and len(ids) < k:
        metrics["k_short"] = (
            "%d cluster(s) came back out of the %d asked for - the rest were empty, which means the "
            "data does not separate that far" % (len(ids), k)
        )
    if len(ids) > 1:
        try:
            from sklearn.metrics import silhouette_score
            keep = np.where(labels >= 0)[0]
            n_scored = len(keep)
            # SB-MLA-020. Subsampled because the score is O(n^2); the CAP is stated with the number
            # so it is never read as a whole-field figure beside metrics that are.
            if len(keep) > SILHOUETTE_CAP:
                keep = np.random.RandomState(seed).choice(keep, SILHOUETTE_CAP, replace=False)
            metrics["silhouette"] = round(float(silhouette_score(As[keep], labels[keep])), 4)
            metrics["silhouette_basis"] = (
                "all %d clustered sample(s)" % n_scored if n_scored <= SILHOUETTE_CAP else
                "a seeded random %d of %d clustered sample(s) - this score is a sample, unlike the "
                "counts beside it" % (SILHOUETTE_CAP, n_scored)
            )
        except Exception as e:
            metrics["silhouette_error"] = str(e)
    outs.append(("", out))
    if prob is not None:
        # SB-MLA-030. GMM's is the one genuinely calibrated posterior here, and it deserves to be
        # distinguished from the classifier scores rather than sharing an undifferentiated name.
        metrics["prob_definition"] = (
            "the winning mixture component's RESPONSIBILITY - a true posterior over the components. "
            "1.0 is unambiguous; about 1/K means the sample sits on a boundary between components"
        )
        metrics["prob_normalisation"] = "across the K mixture components at each depth, summing to 1"
        outs.append(("_PROB", prob.astype(np.float32)))

elif task == "reduction":
    if algo == "pca":
        from sklearn.decomposition import PCA
        c = max(1, min(d, int(P(p, "n_components", 3))))
        P_used("n_components", c)
        pca = PCA(n_components=c, random_state=seed)
        Z = pca.fit_transform(As)
        # SB-MLA-048. A principal component is only defined up to its SIGN - the eigenvector solver
        # may return either, and which one it returns can change with the sample set, the LAPACK
        # build or the scikit-learn version. Left alone, the same wells re-run next month give a PC1
        # that is the mirror of the one in last month's report: every crossplot reversed, every
        # "high PC1 is the clean sand" statement inverted, and nothing to show anything changed.
        #
        # The convention: each component is oriented so its loading on the FIRST feature curve is
        # non-negative. Anchored to the user's own first input rather than to the largest loading,
        # because the largest loading can itself change between runs and a rule that moves is not a
        # convention. Same reasoning as the cluster ordering - put the curve you want to read the
        # component against first.
        flip = np.where(pca.components_[:, 0] < 0.0, -1.0, 1.0)
        Z = Z * flip[np.newaxis, :]
        loadings = pca.components_ * flip[:, np.newaxis]
        metrics["explained_variance_pct"] = [round(float(v) * 100, 2) for v in pca.explained_variance_ratio_]
        # SB-MLA-047. The variance ratios say how much each component carries; the LOADINGS say what
        # it is made of, which is the half a petrophysicist reads. Reported per component as
        # {curve: weight}, so "PC1 is mostly density against neutron" is answerable without
        # re-deriving it.
        # The header's feature list is the authority; the positional fallback keeps a loadings table
        # readable rather than absent if it ever arrives short.
        fname = lambda j: str(feature_names[j]) if j < len(feature_names) else ("x%d" % j)
        metrics["loadings"] = {
            str(i + 1): {fname(j): round(float(loadings[i, j]), 4) for j in range(loadings.shape[1])}
            for i in range(loadings.shape[0])
        }
        metrics["sign_convention"] = (
            "each component is oriented so its loading on %s is non-negative, so a re-run cannot "
            "silently mirror a crossplot" % fname(0)
        )
    elif algo == "tsne":
        if n_apply > 20000:
            fail("t-SNE is limited to 20000 samples (got " + str(n_apply) + ") - select fewer wells")
        from sklearn.manifold import TSNE
        perp = min(float(P(p, "perplexity", 30.0)), max(5.0, (n_apply - 1) / 3.0))
        P_used("perplexity", perp)
        ts = TSNE(n_components=2, perplexity=perp, random_state=seed)
        Z = ts.fit_transform(As)
        if hasattr(ts, "kl_divergence_"):
            metrics["kl_divergence"] = round(float(ts.kl_divergence_), 4)
    elif algo == "autoencoder":
        fail("autoencoders need PyTorch and are not wired up yet - use PCA for now")
    else:
        fail("unknown reduction algorithm '" + algo + "'")
    for i in range(Z.shape[1]):
        outs.append((str(i + 1), Z[:, i].astype(np.float32)))
else:
    fail("unknown task '" + task + "'")

metrics["n_apply"] = n_apply
# SB-MLA-001: the EFFECTIVE parameter set, defaults included and marked as defaults. Rust cannot
# assemble this - it does not know which of the caller's keys this algorithm actually read, nor
# what the runner substituted for the ones it did not send.
metrics["effective_params"] = EFFECTIVE

# A fitted model is an ARTIFACT, not a by-product of one run: dump it so the SAME model can be
# applied to other wells later. The SCALER goes with it - refitting a StandardScaler on the apply
# wells would be a different transform, and the predictions would be quietly wrong rather than
# obviously broken. The feature NAMES go with it too, so the apply side can prove it is feeding
# the columns the model was fitted on, in the order it was fitted on them.
model_blob = b""
sklearn_version = ""
if save_model and supervised:
    try:
        import io as _io, joblib, sklearn as _sk
        buf = _io.BytesIO()
        joblib.dump({"scaler": scaler, "model": model, "features": feature_names,
                     "task": task, "algorithm": algo}, buf, compress=3)
        model_blob = buf.getvalue()
        sklearn_version = _sk.__version__
    except Exception as e:
        # Never lose the RUN because the artifact could not be saved - the curves are already
        # computed. Report it and let the caller say so.
        metrics["model_save_error"] = str(e)

# SB-MLA-005. Reported on EVERY run, not only when a model is saved: the curve is as much a product
# of this library set as the artifact is, and a run that saved nothing still has to be reproducible.
# joblib is in here because it is the SERIALISER - a pickle written by one version and read by
# another is the failure this record exists to name - and scipy because sklearn's solvers reach into
# it. Missing entries are omitted rather than written empty, so "not installed" cannot be read as
# "version unknown".
sys.stdout.buffer.write((json.dumps({"suffixes": [s for s, _ in outs], "metrics": metrics,
                                     "model_len": len(model_blob), "sklearn": sklearn_version,
                                     "runtime": _runtime()}) + "\n").encode("utf-8"))
for _, arr in outs:
    sys.stdout.buffer.write(np.ascontiguousarray(arr, dtype=np.float32).tobytes())
if model_blob:
    sys.stdout.buffer.write(model_blob)
"#;

/// Applies an ALREADY FITTED model. It loads the artifact and predicts — it never fits, which
/// is the whole point: a refit on different data is a different model.
const ML_APPLY_RUNNER: &str = r#"
import sys, json
import numpy as np

def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
d = header["d"]; n_apply = header["n_apply"]; model_len = header["model_len"]
want = header.get("features") or []

blob = sys.stdin.buffer.read(model_len)
if len(blob) != model_len:
    fail("truncated model stream")
raw = sys.stdin.buffer.read(4 * n_apply * d)
if len(raw) != 4 * n_apply * d:
    fail("truncated input stream")
A = np.frombuffer(raw, dtype=np.float32, count=n_apply * d).reshape(n_apply, d).astype(np.float64)

try:
    import io as _io, joblib
    import sklearn  # noqa: F401
except ImportError:
    fail("scikit-learn and joblib are needed to apply a saved model - run: pip install scikit-learn joblib")
try:
    bundle = joblib.load(_io.BytesIO(blob))
except Exception as e:
    import sklearn as _sk
    fail("could not load the saved model (this project's scikit-learn is " + _sk.__version__ + "): " + str(e))

have = bundle.get("features") or []
# The ordering contract, checked inside the artifact itself. Feeding a model trained on
# [GR, RHOB, NPHI] a matrix ordered [GR, NPHI, RHOB] produces confident nonsense that nothing
# downstream can catch, so it must be impossible rather than unlikely.
if want and have and list(want) != list(have):
    fail("this model was fitted on " + ", ".join(have) + " - refusing to apply it to " + ", ".join(want))
if have and len(have) != d:
    fail("this model expects " + str(len(have)) + " input curve(s), got " + str(d))

scaler = bundle.get("scaler")
model = bundle["model"]
task = bundle.get("task", "regression")
As = scaler.transform(A) if scaler is not None else A

outs = [("", model.predict(As).astype(np.float32))]
if task == "classification" and hasattr(model, "predict_proba"):
    outs.append(("_PROB", np.max(model.predict_proba(As), axis=1).astype(np.float32)))

sys.stdout.buffer.write((json.dumps({"suffixes": [s for s, _ in outs],
                                     "metrics": {"n_apply": n_apply, "applied": True},
                                     "runtime": _runtime()}) + "\n").encode("utf-8"))
for _, arr in outs:
    sys.stdout.buffer.write(np.ascontiguousarray(arr, dtype=np.float32).tobytes())
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct MlRequest {
    /// "regression" | "classification" | "clustering" | "reduction"
    pub task: String,
    pub algorithm: String,
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
    pub feature_curves: Vec<String>,
    #[serde(default)]
    pub target_curve: Option<String>,
    /// Optional flag curve: samples where the mask == 1.0 are excluded from training AND left
    /// MISSING (NaN) in the prediction — the same 0/1 convention as the module MASK (workflow.rs).
    #[serde(default)]
    pub mask_curve: Option<String>,
    #[serde(default)]
    pub train_well_ids: Vec<String>,
    pub apply_well_ids: Vec<String>,
    pub output_curve: String,
    /// Keep the fitted model under this name so it can be applied to other wells later, instead
    /// of dying with the subprocess. Supervised tasks only — clustering and reduction are fitted
    /// on the wells they are applied to by construction, so there is no separate model to reuse.
    #[serde(default)]
    pub save_model_as: Option<String>,
    #[serde(default)]
    pub model_note: Option<String>,
    /// Hold roughly this fraction of the pooled training SAMPLES back from the fit and score the
    /// model on them.
    ///
    /// A share of the data, held back as whole wells — see `split_blind_wells` for why those are
    /// two separate decisions and why the achieved share is reported beside the requested one.
    /// Supervised tasks only: clustering and reduction are fitted on the very wells they are
    /// applied to, so "held out" would not mean anything there. `None` keeps the old behaviour
    /// exactly, which is what lets every saved workflow and older IPC payload run unchanged.
    #[serde(default)]
    pub blind_fraction: Option<f64>,
    /// Seed for the draw. Fixed by default so the same request re-runs to the same split — a blind
    /// score that moves when nothing changed cannot be cited (`SB-MLA-008`).
    #[serde(default)]
    pub split_seed: Option<u64>,
    /// How the blind set is drawn: `"well"` (default) holds back whole wells; `"sample"` draws
    /// individual rows, stratified on the target.
    ///
    /// They answer different questions and neither is a better version of the other. **By well**
    /// asks "will this model work on the next well I drill?" — the only question a field study
    /// actually has, and the only split that cannot leak. **By sample** asks "has this model
    /// learned the relationship present in these wells?" — the conventional ML hold-out, exact in
    /// its percentage and balanced in its statistics, and optimistic on log data because
    /// consecutive depths are near-duplicates of each other.
    ///
    /// Defaults to `"well"`, which is what every older payload sends by omission.
    #[serde(default)]
    pub split_mode: Option<String>,
    /// Fit on a TRANSFORM of the target instead of the target itself — currently `"log10"`, or
    /// `None`/`"none"` for the raw quantity (SB-MLA-035).
    ///
    /// Permeability is the reason this exists: it spans orders of magnitude, so a least-squares fit
    /// in linear space is dominated by the few highest-permeability samples and an R² of 0.9 can
    /// coexist with an order-of-magnitude error through the whole reservoir-quality range. Fitting
    /// log10(k) is the standard practice, not an option to be clever with.
    ///
    /// What the requirement is actually about is what happens NEXT. A permeability predicted in
    /// log10 space is a DIFFERENT QUANTITY from a permeability in mD, and if it is written under the
    /// same mnemonic with the same unit then a reported mean of −0.4 reads as a permeability
    /// instead of as 0.398 mD in log units. It renders, it prints, it reaches a client deck, and the
    /// only reader who can catch it is one who already knows the transform was set. So the
    /// log-space prediction gets its own name and its own unit, the scores say which space they were
    /// computed in, and the back-transform is a separate declared curve rather than an invisible
    /// step.
    #[serde(default)]
    pub target_transform: Option<String>,
    /// Read every feature, target and mask curve from THIS log set's stored values (latest
    /// version per well) instead of whatever the current values happen to be. Curves the set
    /// never wrote fall back to normal resolution.
    ///
    /// Jauhar, 2026-08-05: *"each tools or modules should give user freedom to define input and
    /// output log set ... and their own curves"*. Without it a model trained today and one
    /// trained after the next porosity re-run are fitted on different rock with nothing in either
    /// artifact able to say so — which is exactly the provenance saving a model was for.
    #[serde(default)]
    pub input_set: Option<String>,
    /// Version the predicted curves into this log set. Defaults to `ML` — what was hardcoded
    /// before — so an older payload behaves identically.
    #[serde(default)]
    pub output_set: Option<String>,
    /// Fit a separate model per pattern of AVAILABLE inputs, instead of one model over the rows
    /// where every input exists.
    ///
    /// A depth is normally used only where every selected curve has a value, so one curve logged
    /// over half the well deletes the other half of every other curve too — in training and in
    /// prediction. With this on, the half carrying four curves is predicted by a four-curve model
    /// and the half carrying three by a three-curve model, each fitted and scored on its own terms.
    ///
    /// Off by default: it changes what a run means. One model over one feature set is a simpler
    /// object to defend in a report, and a user who has not asked for two should not silently get
    /// two.
    #[serde(default)]
    pub coverage_segments: bool,
    /// Write the prediction at this vertical resolution: one value per `output_step`-thick interval,
    /// held across the interval, on the well's own depths.
    ///
    /// A model fitted against a target sampled every 0.5 m predicts at every INPUT depth, so it
    /// emits a value every 0.1524 m — a curve claiming three times the resolution anything it
    /// learned from ever had, and nothing downstream can tell. Set to the target's own sampling and
    /// the curve stops overstating itself.
    ///
    /// The frame does NOT change — only the values, held in blocks — because `computed_curves` are
    /// read back by exact depth match, so a curve written at its own coarser sampling would land on
    /// depths the well does not have and read back all-missing. Re-framing is `reframe.rs`'s job.
    ///
    /// `None` writes at the input frame, which is what every run did before this existed. Declared
    /// rather than inferred: a run that quietly coarsened its own output would be changing the
    /// answer on the user's behalf.
    #[serde(default)]
    pub output_step: Option<f64>,
    /// Confine BOTH the fit and the prediction to this depth window. Open by default, so every
    /// pre-existing payload keeps running over the whole well.
    #[serde(default)]
    pub interval: DepthWindow,
}

/// Applying an already-fitted model. Deliberately NOT an `MlRequest`: there is no training
/// well, no algorithm and no parameter here — those are properties of the saved model, and
/// letting a caller restate them would invite them to differ.
#[derive(Debug, Clone, Deserialize)]
pub struct MlApplyRequest {
    /// Read the model's feature curves from this log set (see [`MlRequest::input_set`]).
    #[serde(default)]
    pub input_set: Option<String>,
    /// Confine the prediction to this depth window. Open by default. NOT inherited from the model:
    /// the interval a model was FITTED over is a statement about where it learned, and the interval
    /// it is being applied to is a separate decision the user makes per distribution — propagating a
    /// model fitted in one formation into a different one is a choice, and usually a wrong one, but
    /// it is theirs to make and to see.
    #[serde(default)]
    pub interval: DepthWindow,
    /// Version the applied curves into this log set (default `ML`).
    #[serde(default)]
    pub output_set: Option<String>,
    pub model_id: String,
    pub apply_well_ids: Vec<String>,
    pub output_curve: String,
    #[serde(default)]
    pub mask_curve: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MlWellResult {
    pub well_id: String,
    /// Samples that actually got a prediction (rows where every input curve was present).
    pub rows_predicted: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MlResult {
    /// Curve names written (base name + per-output suffix, e.g. FACIES_ML, FACIES_ML_PROB).
    pub outputs: Vec<String>,
    pub metrics: serde_json::Value,
    pub wells: Vec<MlWellResult>,
    /// Advisories that qualify a successful run — e.g. training wells that contributed no usable
    /// samples, so a 20-well selection was really fit on 3. Empty on a fully clean run.
    pub notes: Vec<String>,
    /// Set when the fit was kept as a reusable artifact (`save_model_as`).
    pub model_id: Option<String>,
    /// The name it was actually stored under — an existing name is auto-suffixed rather than
    /// overwritten, so this can differ from what was asked for.
    pub model_name: Option<String>,
    /// Which wells were fitted on and which were held blind. `None` when no split was asked for.
    pub split: Option<SplitReport>,
    pub error: Option<String>,
}

/// The split as it was actually performed, not as it was requested.
///
/// The requested fraction is kept beside the ACHIEVED one on purpose, and both row counts beside
/// both. The fraction asks for a share of the DATA, but the thing held back is a whole well (see
/// `split_blind_wells`), and whole wells are lumpy: five wells cannot usually be divided into
/// exactly 30% of their pooled samples. Reporting only the request would make the blind score a
/// claim about an unstated amount of rock. Well NAMES, because the next question after a blind
/// score is always "which ones?".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SplitReport {
    /// In `sample` mode both lists are empty — every well contributes to both sides, so naming
    /// them would say nothing. The row counts carry the whole answer there.
    pub fit_wells: Vec<String>,
    pub blind_wells: Vec<String>,
    /// Usable training rows on each side — what the fraction is really a fraction of.
    pub fit_rows: usize,
    pub blind_rows: usize,
    pub requested_fraction: f64,
    /// `blind_rows / (fit_rows + blind_rows)`. What the user actually got.
    pub achieved_fraction: f64,
    pub seed: u64,
    /// `"well"` or `"sample"` — see [`MlRequest::split_mode`]. Recorded because the two are
    /// different claims and a score quoted without it cannot be read.
    pub mode: String,
    /// How many wells contributed rows. Present in both modes: in `sample` mode it is the answer
    /// to "how much rock is this really?", which the well lists no longer give.
    pub wells_pooled: usize,
}

/// The depth window a run is confined to.
///
/// Jauhar, 2026-08-07: *"it should be tops bounded as well by user"*. A model fitted over a whole
/// well learns one relation for every formation it passed through, and a shale-prone deltaic sand
/// and the carbonate below it do not share a porosity-permeability transform. Confining the fit to
/// the interval the interpreter actually means is the difference between a model and an average.
///
/// **Each side is independent, and an open side is open.** That is the same convention
/// `TopInterval` already follows in the frontend — the last top in a well runs to TD, which is
/// expressed as no base rather than as a guessed one. Treating a missing base as "no window" would
/// silently widen the run back to the whole well.
///
/// Applied like the run MASK, and for the same reason: a sample outside the window is not sent to
/// python, so the scatter-back leaves it NaN. A depth outside the interval was not interpreted, and
/// an empty sample says exactly that.
#[derive(Debug, Clone, Copy, Default, Deserialize, serde::Serialize)]
pub struct DepthWindow {
    #[serde(default)]
    pub top: Option<f64>,
    #[serde(default)]
    pub base: Option<f64>,
}

impl DepthWindow {
    /// True when this window constrains nothing — used to keep the notes quiet on an ordinary run.
    pub fn is_open(&self) -> bool {
        self.top.is_none() && self.base.is_none()
    }

    /// Inclusive at the top, EXCLUSIVE at the base — so two abutting intervals cannot both claim
    /// the sample sitting exactly on their shared marker, which would double-count it in any run
    /// that swept a well zone by zone.
    pub fn contains(&self, d: f32) -> bool {
        let d = d as f64;
        if !d.is_finite() {
            return false;
        }
        self.top.map_or(true, |t| d >= t) && self.base.map_or(true, |b| d < b)
    }

    /// How the window reads in a note, for a run that has to say what it was confined to.
    pub fn describe(&self) -> String {
        match (self.top, self.base) {
            (Some(t), Some(b)) => format!("{t} to {b}"),
            (Some(t), None) => format!("{t} to TD"),
            (None, Some(b)) => format!("the top of the log to {b}"),
            (None, None) => "the whole well".to_string(),
        }
    }
}

fn fail(msg: &str) -> MlResult {
    MlResult {
        outputs: vec![],
        metrics: serde_json::Value::Null,
        wells: vec![],
        notes: vec![],
        split: None,
        model_id: None,
        model_name: None,
        error: Some(msg.to_string()),
    }
}

/// Pools labelled training rows across the training wells and reports which wells contributed
/// ZERO usable samples — unreadable, missing the target or an input curve (`fetch_curve_frame`
/// returns an all-NaN column for a curve the well lacks, so a wrong target mnemonic lands here
/// rather than as an error), or fully masked. That list is the honesty signal the caller
/// surfaces, so a 20-well selection cannot silently be fit on 3.
/// Choose which wells are held blind, deterministically from `seed`, so that the rows they carry
/// land as near as possible to `fraction` of the pooled training rows. `counts[i]` is the number
/// of usable samples well `i` contributed.
///
/// **The fraction is a share of the DATA; the thing held back is a whole WELL.** Those are two
/// different statements and both are deliberate.
///
/// A share of the data, because that is what the user is deciding — "hold 30% of what these five
/// wells gave me" (Jauhar, 2026-08-07: *"not 30% of wells, but from 30% of total data those 5
/// wells gave"*). Counting wells instead would make the same 30% mean 6% of the rock when the two
/// wells drawn happen to be short re-entries, and 55% when they are the deep ones — a blind score
/// whose meaning moves with the draw.
///
/// A whole well, because splitting pooled SAMPLES 70/30 puts consecutive depths from one well on
/// both sides of the line. At a 0.1524 m sampling the row above and the row below are all but the
/// same rock, so the model is scored on what it already saw a few centimetres away and the blind
/// score is optimistic by construction — the same failure `SB-MLA-028` closed in the leaderboard.
///
/// So the fraction is a TARGET rather than a count. Walk the wells in seeded-shuffled order and
/// take one whenever taking it moves the running row total CLOSER to the target; the whole list is
/// scanned rather than stopped at the first well that overshoots, so one big well early does not
/// force the split — a smaller one later can still fill the gap. Whole wells are lumpy and the
/// target is often unreachable; the caller reports what was achieved beside what was asked
/// (`SplitReport::achieved_fraction`), because a miss the user cannot see is a blind score about
/// an unstated amount of rock.
///
/// At least one well stays on each side whenever there are two to divide: a request for a blind
/// test that silently produces no blind well is the kind of clean-looking nothing `SB-CORE-002`
/// exists to forbid.
fn split_blind_wells(counts: &[usize], fraction: f64, seed: u64) -> Vec<usize> {
    let n = counts.len();
    let total: usize = counts.iter().sum();
    if n < 2 || total == 0 || !(fraction > 0.0) {
        return Vec::new();
    }
    let target = (total as f64) * fraction.min(1.0);
    let mut next = splitmix(seed);
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let mut blind: Vec<usize> = Vec::new();
    let mut acc = 0f64;
    for &i in &order {
        let c = counts[i] as f64;
        if (acc + c - target).abs() < (acc - target).abs() {
            blind.push(i);
            acc += c;
        }
    }
    // Both guards restore the ONE well per side floor the doc comment promises. Empty happens when
    // every well overshoots on its own (one well holds more than twice the target); full happens at
    // a fraction near 1. In each case the shuffled order decides, so the result stays seeded.
    if blind.is_empty() {
        // The well that lands nearest on its own — not simply the first, which on a lopsided field
        // can be the one well that overshoots worst.
        let pick = *order
            .iter()
            .min_by(|&&a, &&b| {
                let da = (counts[a] as f64 - target).abs();
                let db = (counts[b] as f64 - target).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        blind.push(pick);
    } else if blind.len() == n {
        // Give back whichever well leaves the remainder nearest the target.
        let drop_at = (0..blind.len())
            .min_by(|&a, &b| {
                let da = (acc - counts[blind[a]] as f64 - target).abs();
                let db = (acc - counts[blind[b]] as f64 - target).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        blind.remove(drop_at);
    }
    blind.sort_unstable();
    blind
}

/// SplitMix64 — the one seeded generator in this repo (`facies.rs`, `split_blind_wells`).
/// Reproducible across platforms in a way a hash-map iteration order is not.
fn splitmix(seed: u64) -> impl FnMut() -> u64 {
    let mut s = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    move || {
        s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Draw `fraction` of the ROWS at random as the blind set, STRATIFIED so the draw carries the same
/// distribution as the whole (Jauhar, 2026-08-07: *"random sample 3000 data from there with
/// similar statistic taken to be tested as blind"*).
///
/// Returns a 0/1 mask, one entry per training row.
///
/// **Stratified, not plain random, because "similar statistic" is the requirement.** A flat 30%
/// draw over 10 000 rows has no obligation to be representative: a thin coal of forty samples can
/// land wholly on one side, and a classifier's rarest facies can miss the blind set entirely — at
/// which point the blind accuracy is an average over the classes that happened to be drawn.
/// Drawing 30% WITHIN each stratum forces both sides to carry the same mix. `strata` is the class
/// for a classifier and a decile of the target for a regressor; either way the split is balanced on
/// the quantity being predicted, which is the one that decides the score.
///
/// Each stratum contributes `round(fraction * size)`, then the remainder is corrected against the
/// exact target so the total lands on the requested count rather than accumulating rounding error
/// across ten strata. Every stratum with two or more rows keeps at least one on each side; a
/// stratum of one cannot be divided and goes to the fit, because a blind class the model was never
/// shown scores zero for a reason that is not the model's.
///
/// **This split leaks and the caller says so.** Consecutive depths at a 0.1524 m sampling are all
/// but the same rock, so a row drawn blind usually has its neighbour in the fit set and the score
/// is optimistic. That is a property of the method, not a defect in it — the well-grouped
/// cross-validation score is reported beside it precisely so the two can be compared.
fn split_blind_samples(strata: &[i64], fraction: f64, seed: u64) -> Vec<f32> {
    let n = strata.len();
    let mut mask = vec![0f32; n];
    if n < 2 || !(fraction > 0.0) {
        return mask;
    }
    let f = fraction.min(1.0);
    let want_total = ((n as f64) * f).round() as usize;
    let want_total = want_total.clamp(1, n - 1);

    // Rows grouped by stratum, in first-seen order so the result does not depend on hash ordering.
    let mut order: Vec<i64> = Vec::new();
    let mut buckets: std::collections::HashMap<i64, Vec<usize>> = std::collections::HashMap::new();
    for (i, s) in strata.iter().enumerate() {
        buckets.entry(*s).or_insert_with(|| {
            order.push(*s);
            Vec::new()
        });
        buckets.get_mut(s).unwrap().push(i);
    }

    let mut next = splitmix(seed);
    let mut taken = 0usize;
    for s in &order {
        let rows = buckets.get_mut(s).unwrap();
        let m = rows.len();
        if m < 2 {
            continue; // cannot divide a stratum of one; it stays in the fit set.
        }
        let want = (((m as f64) * f).round() as usize).clamp(1, m - 1);
        for i in (1..m).rev() {
            let j = (next() % (i as u64 + 1)) as usize;
            rows.swap(i, j);
        }
        for &r in rows.iter().take(want) {
            mask[r] = 1.0;
            taken += 1;
        }
    }

    // Correct the total. Per-stratum rounding drifts (ten strata of 55 rows at 30% round to 17
    // each, 170 against a target of 165), and a user who asked for 3000 of 10000 should get 3000.
    // Corrections are drawn from the shuffled rows so they stay seeded and stay spread across
    // strata rather than all coming out of the first one.
    let mut adjust = |want_more: bool, count: usize| {
        let mut left = count;
        for s in &order {
            if left == 0 {
                break;
            }
            let rows = &buckets[s];
            let held: usize = rows.iter().filter(|&&r| mask[r] > 0.5).count();
            for &r in rows.iter() {
                if left == 0 {
                    break;
                }
                let is_blind = mask[r] > 0.5;
                // Never empty a side of a stratum that had two rows to divide.
                if want_more && !is_blind && held + 1 < rows.len() {
                    mask[r] = 1.0;
                    left -= 1;
                } else if !want_more && is_blind && held > 1 {
                    mask[r] = 0.0;
                    left -= 1;
                }
            }
        }
    };
    if taken < want_total {
        adjust(true, want_total - taken);
    } else if taken > want_total {
        adjust(false, taken - want_total);
    }

    // The same floor the well split holds: a blind test that silently produces no blind row, or
    // leaves nothing to fit on, is the clean-looking nothing SB-CORE-002 forbids.
    let held: usize = mask.iter().filter(|v| **v > 0.5).count();
    if held == 0 {
        mask[0] = 1.0;
    } else if held == n {
        mask[0] = 0.0;
    }
    mask
}

/// The strata `split_blind_samples` balances on: the class itself for a classifier, a decile of the
/// target for a regressor.
///
/// Deciles rather than raw values because a continuous target has as many strata as rows otherwise,
/// and every stratum of one would fall through the `m < 2` guard and land in the fit set. Ten is
/// the conventional choice and is enough to hold the shape of a distribution without making the
/// strata too thin to divide.
fn strata_for(y: &[f32], classification: bool) -> Vec<i64> {
    if classification {
        return y.iter().map(|v| if v.is_finite() { *v as i64 } else { i64::MIN }).collect();
    }
    let mut sorted: Vec<f32> = y.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() {
        return vec![0; y.len()];
    }
    let cuts: Vec<f32> = (1..10).map(|k| crate::distribution::percentile(&sorted, k as f32 * 10.0)).collect();
    y.iter()
        .map(|v| {
            if !v.is_finite() {
                return i64::MIN;
            }
            cuts.iter().filter(|c| *v > **c).count() as i64
        })
        .collect()
}

/// SB-MLA-002 + SB-MLA-004 — what ONE training well contributed, and what it was read from.
///
/// `trained_on` answers "which wells". This answers "which rock", which is the question a re-run has
/// to match and the one a well list cannot reach: the same well at a later log-set version is
/// different rows under the same name.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainWellRecord {
    pub well_id: String,
    pub well: String,
    /// Rows that reached X/y.
    pub rows: usize,
    /// Rows the run mask removed (SB-MLA-004). The mask decides which rows trained the model, so
    /// under SB-MLA-003 it is part of the model's identity — and it is the parameter most likely to
    /// differ between an analyst's run and a reviewer's re-run, because a bad-hole flag is itself a
    /// computed curve somebody else owns.
    pub masked: usize,
    /// Rows dropped for a missing target or input, which the mask had nothing to do with. Recorded
    /// SEPARATELY because the two call for opposite fixes — widen the mask, or go and find the
    /// missing curve — and a single "rows not used" number reads as the mask's doing.
    pub incomplete: usize,
    /// SB-MLA-002. The log set this well's frame was read from. `None` means the CURRENT store,
    /// which is a different and weaker provenance: the values can move under the model without
    /// anything changing name or version.
    pub set_name: Option<String>,
    pub set_id: Option<String>,
    pub set_version: Option<i64>,
}

/// SB-MLA-004 — the whole training record: the run-level mask, and the per-well roster.
///
/// The mask is wrapped around the roster rather than repeated on each well because it is one
/// decision applied to the whole run, and a field copied onto ninety rows is ninety chances for a
/// reader to find two of them disagreeing.
///
/// **`mask_curve: None` is a positive statement, not a blank.** The requirement asks for the mask
/// "or its explicit absence", and the distinction it is reaching for is real: no mask at all, versus
/// a mask that was applied and flagged nothing. The second reads as `Some(name)` with every `masked`
/// at zero — and an all-zero bad-hole flag across a field is usually a sign the flag was never
/// computed, which is worth noticing rather than reading as clean hole.
///
/// The shape is versionless on purpose: `training_json` is written by this build and has never
/// shipped in another, so there is no earlier form to tolerate. A model saved before the column
/// existed has NULL, which every reader already treats as "not recorded".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainingRecord {
    /// The run mask, by curve name, uppercased as the run applied it. `None` = no mask was used.
    pub mask_curve: Option<String>,
    /// One entry per well that actually contributed rows. A well that gave nothing is not part of
    /// the training rock, and listing it would make this record disagree with SB-MLA-003's
    /// fingerprint about what the model was fitted on.
    pub wells: Vec<TrainWellRecord>,
}

/// SB-MLA-005 — every recorded runtime component that differs from the one about to load the model.
///
/// Named component by component rather than as "the runtime differs", because the responses are not
/// the same: a numpy step is usually nothing, a **joblib** step is the one that unpickles the blob,
/// and a scikit-learn step can change an estimator's arithmetic without changing its name.
///
/// Three recorded states, and the difference between two of them is the whole point (see
/// `ML_RUNTIME_PY`). A recorded VERSION that has changed or gone is the ordinary case. A recorded
/// **null** — probed at fit time, not installed — that is now present is reported too, because for
/// `xgboost` that means the fit used a substituted scikit-learn estimator and this machine would not.
/// A key MISSING from the record is silent: the model predates the probe knowing about that
/// component, and inventing a mismatch out of that would cry wolf on every older model.
fn runtime_drift(recorded: Option<&str>, current: &serde_json::Value) -> Vec<String> {
    let Some(rec) = recorded.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()) else {
        return Vec::new();
    };
    let (Some(rec), Some(cur)) = (rec.as_object(), current.as_object()) else {
        return Vec::new();
    };
    // A key absent from `current` is treated as "this probe did not ask", so it cannot manufacture a
    // step; only a key the CURRENT probe answered null for is read as "not installed".
    let mut diffs: Vec<String> = Vec::new();
    for (name, was) in rec {
        let Some(now) = cur.get(name) else { continue };
        if now == was {
            continue;
        }
        let show = |v: &serde_json::Value| v.as_str().unwrap_or("not installed").to_string();
        diffs.push(format!("{name} {} -> {}", show(was), show(now)));
    }
    if diffs.is_empty() {
        return Vec::new();
    }
    diffs.sort();
    vec![format!(
        "this model was fitted under a different runtime ({}). The artifact is a pickle, so it is loadable only under a compatible set - it loaded here, but a prediction made under a changed library is not guaranteed to be the one the model was validated on",
        diffs.join(", ")
    )]
}

/// SB-MLA-005 — the runtime a model would be applied under, probed ONCE per session.
///
/// The requirement says the warning must come BEFORE the model is applied, and the apply run reports
/// its own runtime only once it has already predicted. So the interpreter is asked separately, at the
/// moment the user is looking at a list of models deciding which one to push across fifty wells.
///
/// Cached like `python_status`: the answer cannot change while the app is running, and probing per
/// row would spawn a subprocess per model in the list.
pub fn ml_runtime() -> serde_json::Value {
    static RUNTIME: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            let Some(python) = find_python() else { return serde_json::Value::Null };
            // The SAME `_runtime()` the runners use — a second probe naming its components
            // differently would report a mismatch between `scikit-learn` and `sklearn` on one
            // machine, which is the failure mode this whole comparison exists to avoid.
            let script = format!("{ML_RUNTIME_PY}\nimport json, sys\nsys.stdout.write(json.dumps(_runtime()))\n");
            let mut cmd = Command::new(&python);
            cmd.args(["-c", &script]).stdout(Stdio::piped()).stderr(Stdio::null());
            hide_console(&mut cmd);
            cmd.output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| serde_json::from_slice(&o.stdout).ok())
                .unwrap_or(serde_json::Value::Null)
        })
        .clone()
}

/// The curve name one runner output lands under, and the back-transform companion where a transform
/// ran.
///
/// SB-MLA-035 lives here: under a transform the model's own output is `<base>_LOG10` and the
/// back-transform is `<base>`, never the other way round and never in place. Shared by the ordinary
/// path and the coverage-segmented one, because two spellings of this rule would let one path write
/// a log-space curve under the name the other reserves for millidarcies.
fn out_name_for(base: &str, suffix: &str, transform: &str) -> String {
    match (transform.is_empty(), suffix) {
        (false, "") => format!("{base}{LOG10_SUFFIX}"),
        (false, BACK_SUFFIX) => base.to_string(),
        // The textured curve is a texturing OF the model's own output, so under a transform it is
        // named after the log-space curve it was made from — `PERM_LOG10_SIM`, never `PERM_SIM`,
        // which would read as millidarcies and be out by orders of magnitude on a plot.
        (false, SIM_SUFFIX) => format!("{base}{LOG10_SUFFIX}{SIM_SUFFIX}"),
        _ => format!("{base}{suffix}"),
    }
}

/// The unit an output carries, or `None` where there is nothing true to say.
///
/// A blank stays blank: "we do not know this curve's unit" must never be written as though it were
/// "this curve is dimensionless". A class code and a probability have no unit at all, so they are
/// left out of the declaration entirely rather than declared empty.
fn unit_for_output(suffix: &str, transform: &str, target_unit: Option<&str>) -> Option<String> {
    let tu = target_unit.unwrap_or_default();
    // The textured curve is the model's own output with detail added, so it is in whatever space
    // that output is in — the same unit as the base curve, transformed or not. Leaving it undeclared
    // would export a curve in log space with no unit beside one that has millidarcies.
    let base_space = suffix.is_empty() || suffix == SIM_SUFFIX;
    let u = if base_space && !transform.is_empty() {
        transformed_unit(transform, Some(tu))
    } else if base_space || suffix == BACK_SUFFIX {
        tu.to_string()
    } else {
        return None;
    };
    (!u.is_empty()).then_some(u)
}

/// Whether an output carries CLASS CODES rather than a quantity.
///
/// A class code is a name written as a number, and the mean of facies 1 and facies 4 is 2.5, which
/// is not a facies — the same rule `frame::block` refuses an averaging statistic under (SB-MLA-055).
/// The distinction is by (task, suffix), not by inspecting the values: a well whose classes happen
/// to come out {0, 1} is indistinguishable from a probability by inspection, and guessing wrong in
/// that direction averages codes silently.
fn output_is_class(task: &str, suffix: &str) -> bool {
    match task {
        // The base curve is the predicted class; `_PROB` beside it is a confidence, which is a real
        // quantity and averages honestly.
        "classification" | "clustering" => suffix.is_empty(),
        // Regression predicts a quantity, and reduction's PC1… are continuous scores.
        _ => false,
    }
}

/// Hold one value per `step`-thick interval, across the interval, ON THE WELL'S OWN DEPTHS.
///
/// Jauhar, 2026-08-07: *"sampling rate, each log has different resolution … Result should adjust
/// their frequency to log target"*, then *"writing output at target sampling"*. A model fitted
/// against a target sampled every 0.5 m predicts at every input depth, so it emits a value every
/// 0.1524 m — a curve claiming three times the vertical resolution any of its training data had.
/// Nothing downstream can tell; it plots as a detailed log.
///
/// **The frame does not change, only the values.** This is `frame::block`'s discipline and it is not
/// a stylistic choice: `computed_curves` are read back by EXACT depth match, so a curve written at
/// its own coarser sampling would land on depths the well does not have and read back all-missing.
/// Re-framing is `reframe.rs`'s job precisely because it cannot be done here. The consequence for
/// the reader is that the curve needs `draw_style: "step"` in the layout, or the log view draws a
/// gradient between two block values that nothing ever measured — the run says so in its notes.
///
/// Blocks are anchored on an ABSOLUTE grid (`floor(depth / step)`), not on each well's first sample.
/// Anchoring per well would give two wells the same block THICKNESS at different block BOUNDARIES,
/// so a bed straddling a boundary in one well would sit mid-block in the next and the two would not
/// be comparable — the same trap `TargetSpec.align` exists for.
///
/// Returns the number of distinct blocks the well's live samples fell into.
fn block_to_step(depth: &[f32], values: &mut [f32], step: f64, class_curve: bool) -> usize {
    if !(step > 0.0) {
        return 0;
    }
    // The bin each sample belongs to, computed once. A closure over `values` would borrow it for as
    // long as it lives and block the write-back below.
    let bins: Vec<Option<i64>> = (0..values.len())
        .map(|i| {
            let d = depth.get(i).copied().unwrap_or(f32::NAN) as f64;
            (d.is_finite() && values[i].is_finite()).then(|| (d / step).floor() as i64)
        })
        .collect();
    let mut buckets: std::collections::BTreeMap<i64, Vec<f32>> = Default::default();
    for (i, k) in bins.iter().enumerate() {
        if let Some(k) = k {
            buckets.entry(*k).or_default().push(values[i]);
        }
    }
    let answer: std::collections::BTreeMap<i64, f32> = buckets
        .iter()
        .map(|(k, v)| {
            let a = if class_curve {
                // The block's commonest code, ties to the shallowest — `frame::block`'s MODE rule,
                // deliberately the same one, so "upscale a class curve" has a single definition.
                let mut best = (v[0], 0usize);
                for x in v {
                    let c = v.iter().filter(|y| (**y - *x).abs() < 1e-6).count();
                    if c > best.1 {
                        best = (*x, c);
                    }
                }
                best.0
            } else {
                // Arithmetic, and that is the right mean for a volume fraction. Under a log10
                // transform this runs in LOG space, which makes it the geometric mean of the
                // millidarcies — the standard permeability upscale, for free and by construction.
                (v.iter().map(|x| *x as f64).sum::<f64>() / v.len() as f64) as f32
            };
            (*k, a)
        })
        .collect();
    for (i, k) in bins.iter().enumerate() {
        if let Some(a) = k.and_then(|k| answer.get(&k)) {
            values[i] = *a;
        }
    }
    answer.len()
}

/// How rough a curve is: the mean absolute step between neighbouring live samples, divided by the
/// curve's own standard deviation.
///
/// Dividing by the spread is what makes two curves comparable — a permeability in millidarcies and a
/// density in g/cc have wildly different step sizes and the question is not how big the steps are but
/// how big they are RELATIVE to the range the curve covers. Gaps are skipped rather than bridged: a
/// step across a washout is not a measurement of anything.
///
/// Deliberately not an FFT. The question a petrophysicist is asking here is "does this log wiggle
/// like the log it is predicting", and the mean absolute difference answers it in a number that can
/// be explained, checked by hand, and computed on a curve with holes in it. A power spectrum needs a
/// regular grid, and these curves have gaps.
fn roughness(values: &[f32]) -> Option<f64> {
    let live: Vec<f64> = values.iter().filter(|v| v.is_finite()).map(|v| *v as f64).collect();
    if live.len() < 8 {
        return None;
    }
    let mean = live.iter().sum::<f64>() / live.len() as f64;
    let var = live.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / live.len() as f64;
    let sd = var.sqrt();
    if !(sd > 0.0) {
        return None;
    }
    // Steps taken only between samples that are BOTH live, so a gap contributes nothing rather than
    // one enormous jump that would read as high-frequency detail.
    let mut steps = 0usize;
    let mut total = 0f64;
    for w in values.windows(2) {
        if w[0].is_finite() && w[1].is_finite() {
            total += (w[1] - w[0]).abs() as f64;
            steps += 1;
        }
    }
    (steps >= 4).then(|| total / steps as f64 / sd)
}

/// What a predicted curve's vertical resolution is worth, measured against the target it learned
/// from.
///
/// Jauhar, 2026-08-07: *"rhob and dres with same sampling at 0.5 f can have different resolution or
/// curve/wave frequency, if we wanna predict rhob, predicted rhob log frequency should follow
/// original rhob as well"*. He is right, and it is the harder half of the sampling question: two
/// curves on one 0.5 ft frame can carry completely different vertical resolution, because a density
/// pad reads a few inches and a deep induction reads several feet.
///
/// **A prediction is always smoother than its target, and that is physics, not a bug.** The model
/// can only carry through the detail its INPUTS contain; asked to predict a sharp density from a
/// smooth resistivity, it returns the sharpness the resistivity had. So this reports the shortfall
/// rather than correcting it: restoring the missing detail means SYNTHESIZING it, which produces a
/// curve that looks better resolved without being better known, and that is a decision for the
/// interpreter to make explicitly rather than a default to inherit.
///
/// Returned as a ratio and a sentence. Below 1 the prediction is smoother than the measured log by
/// roughly that factor.
fn resolution_note(target_train: &[f32], predicted: &[f32], target_name: &str) -> Option<String> {
    let (rt, rp) = (roughness(target_train)?, roughness(predicted)?);
    if !(rt > 0.0) {
        return None;
    }
    let ratio = rp / rt;
    // A prediction within a quarter of its target's roughness is doing as well as this measure can
    // tell. Saying so on every run would train the eye to skip the line that matters.
    if ratio >= 0.75 {
        return None;
    }
    Some(format!(
        "vertical resolution: this prediction varies about {:.0}% as much between neighbouring samples as the measured {target_name} it learned from, so it is a SMOOTHER log than the one it is standing in for. That is the resolution its inputs carry, not a fault in the fit - a curve read over feet cannot produce detail measured over inches. Read thin beds off it with that in mind; nothing here has invented the missing detail, and nothing should without saying so",
        ratio * 100.0
    ))
}

/// Most feature subsets one run will fit models for.
///
/// A field where every well is missing a different curve can present a great many availability
/// patterns, and one fit per pattern is one subprocess per pattern. The cap keeps a pathological
/// delivery from turning a click into forty fits — and the depths it leaves unclaimed are REPORTED
/// rather than dropped quietly, because the whole point of this mode is that no depth goes missing
/// without saying so.
const MAX_COVERAGE_SEGMENTS: usize = 6;

/// Fewest training rows a segment will be fitted on. Below this the segment is skipped BY NAME —
/// a model fitted on forty rows over three curves is not a weaker answer, it is a different kind of
/// object, and quietly producing one under the same curve name as a well-fitted segment would make
/// the curve's own quality vary along its length with nothing recording where.
const MIN_SEGMENT_ROWS: usize = 30;

/// Which model predicts which depth, decided from the patterns of available inputs alone.
///
/// `avail` is one bitmask per depth, per well: bit *k* set means feature *k* has a value there.
/// Returns the candidate subsets largest-first, and per well per depth the index of the candidate
/// that depth belongs to — `None` where no kept candidate is a subset of what the depth carries.
///
/// Three decisions live here and nowhere else:
///
/// **Candidates are the patterns that OCCUR, never all 2^n subsets.** A four-curve run whose curves
/// are either all present or all absent is one model, not sixteen; enumerating subsets would fit
/// fifteen models for rock that does not exist.
///
/// **The cap rations by how much rock a pattern covers, NOT by how many curves it has.** These are
/// two different orderings and using one for both is the trap: the largest patterns are typically
/// the rarest — all-curves-present, plus a handful of near-complete oddities — so keeping the six
/// biggest would blank most of the well. It also makes the fallback below unreachable, because every
/// surviving candidate would then be at least as large as every cut one, and a subset of equal size
/// is the same set.
///
/// **A depth is claimed by the LARGEST candidate whose curves it carries.** Predicting a four-curve
/// depth with the three-curve model would throw away a log sitting right there. The containment test
/// (`c & a == c`) is what makes "carries" mean carries-at-least rather than carries-exactly — so a
/// depth whose own pattern lost the cap is still predicted by the biggest kept subset it can feed,
/// instead of going blank. Only a depth carrying no kept subset at all is left unclaimed.
///
/// Every tie is broken on the data (rows, then curve count, then the pattern itself), never on hash
/// order, which is what makes the same run twice produce the same segments.
fn coverage_plan(avail: &[&[u32]], max_segments: usize) -> (Vec<u32>, Vec<Vec<Option<usize>>>) {
    let mut pattern_rows: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for w in avail {
        for &a in w.iter() {
            if a != 0 {
                *pattern_rows.entry(a).or_insert(0) += 1;
            }
        }
    }
    // Which patterns survive the cap: the ones covering the most rock.
    let mut candidates: Vec<u32> = pattern_rows.keys().copied().collect();
    candidates.sort_by(|a, b| {
        pattern_rows[b].cmp(&pattern_rows[a]).then_with(|| b.count_ones().cmp(&a.count_ones())).then(a.cmp(b))
    });
    candidates.truncate(max_segments);
    // Which of the survivors claims a depth: the one using the most curves.
    candidates.sort_by(|a, b| {
        b.count_ones().cmp(&a.count_ones()).then_with(|| pattern_rows[b].cmp(&pattern_rows[a])).then(a.cmp(b))
    });

    let assigned: Vec<Vec<Option<usize>>> = avail
        .iter()
        .map(|w| {
            w.iter()
                .map(|&a| {
                    // First match wins and `candidates` is ordered largest-first, so this IS
                    // "the largest model whose inputs this depth carries".
                    (a != 0).then(|| candidates.iter().position(|&c| c & a == c)).flatten()
                })
                .collect()
        })
        .collect();
    (candidates, assigned)
}

/// A run segmented by which inputs are actually present at each depth.
///
/// The problem it solves, in Jauhar's words (2026-08-07): *"assume user have 4 curves, model should
/// still run even 1 curves only half depth coverage, (model only predict using 3 curves on the
/// other half depth coverage)"*. The ordinary path uses a depth only where EVERY input has a value,
/// so a curve logged over half the well deletes the other half of every other curve as well — in the
/// fit and in the prediction. On a field where each well is missing something different, the
/// intersection can be nearly empty while every individual curve looks well covered.
///
/// **The rule is: each depth is predicted by the largest model whose inputs it carries.** Candidate
/// subsets are the availability patterns that actually occur, largest first; a depth goes to the
/// first candidate it can satisfy. Candidates are the OBSERVED patterns rather than every subset of
/// the feature list, which would be 2^n and mostly hypothetical.
///
/// **A segment trains on every row carrying its subset, not only on rows matching its pattern
/// exactly.** The three-curve model should learn from the four-curve half too — those rows carry all
/// three of its inputs, and withholding them would fit it on less data than exists for no reason.
///
/// **Each segment keeps its own blind score and its own saved model.** They are different models on
/// different feature sets, and a single number over both would describe neither. That is also why
/// the segments are reported individually rather than summarised: the curve is one curve, and how
/// well it is known genuinely varies along its length.
fn run_ml_coverage(
    db: &Mutex<Connection>,
    req: &MlRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> MlResult {
    let features: Vec<String> =
        req.feature_curves.iter().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty()).collect();
    if features.len() < 2 {
        return fail("coverage segmentation needs at least 2 input curves - with one curve there is no smaller model to fall back to");
    }
    if features.len() > 16 {
        return fail("coverage segmentation is limited to 16 input curves");
    }
    let base = req.output_curve.trim().to_uppercase();
    if base.is_empty() {
        return fail("output curve name is empty");
    }
    let Some(target) = req.target_curve.as_deref().map(|t| t.trim().to_uppercase()).filter(|t| !t.is_empty())
    else {
        return fail("supervised learning needs a target curve");
    };
    if req.train_well_ids.is_empty() {
        return fail("supervised learning needs at least one training well");
    }
    if req.apply_well_ids.is_empty() {
        return fail("select at least one well to apply to");
    }
    let Some(python) = find_python() else {
        return fail("no Python with numpy found - install Python 3.10+ with numpy + scikit-learn, or set SANDIBUMI_PYTHON to its python.exe");
    };
    let mask_curve =
        req.mask_curve.as_deref().map(|m| m.trim().to_uppercase()).filter(|m| !m.is_empty());
    let out_set = req
        .output_set
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ML_SET)
        .to_string();

    // --- Which inputs each depth of each apply well actually carries -------------
    struct CovWell {
        well_id: String,
        depth: Vec<f32>,
        /// Bitmask of present features per depth; 0 where the row is masked or carries nothing.
        avail: Vec<u32>,
        /// Feature values per depth, in `features` order. NaN where absent.
        cols: Vec<Vec<f32>>,
        masked: usize,
        error: Option<String>,
    }
    let mut wells_data: Vec<CovWell> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let mut fetch = features.clone();
        if let Some(mk) = &mask_curve {
            fetch.push(mk.clone());
        }
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None) {
                Ok((depth, cols)) => {
                    // A curve absent from this well entirely is an all-NaN column, NOT a reason to
                    // refuse the well — that is the whole point of this mode.
                    let fcols: Vec<Vec<f32>> = features
                        .iter()
                        .map(|f| cols.get(f).cloned().unwrap_or_else(|| vec![f32::NAN; depth.len()]))
                        .collect();
                    let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
                    let mut avail = vec![0u32; depth.len()];
                    let mut masked = 0usize;
                    for i in 0..depth.len() {
                        if mcol.is_some_and(|m| m[i] == 1.0) || !req.interval.contains(depth[i]) {
                            masked += 1;
                            continue;
                        }
                        let mut bits = 0u32;
                        for (k, c) in fcols.iter().enumerate() {
                            if c[i].is_finite() {
                                bits |= 1 << k;
                            }
                        }
                        avail[i] = bits;
                    }
                    wells_data.push(CovWell {
                        well_id: well_id.clone(),
                        depth,
                        avail,
                        cols: fcols,
                        masked,
                        error: None,
                    });
                }
                Err(e) => wells_data.push(CovWell {
                    well_id: well_id.clone(),
                    depth: vec![],
                    avail: vec![],
                    cols: vec![],
                    masked: 0,
                    error: Some(e.to_string()),
                }),
            }
        }
    }

    let avail_per_well: Vec<&[u32]> = wells_data.iter().map(|w| w.avail.as_slice()).collect();
    let (candidates, assigned) = coverage_plan(&avail_per_well, MAX_COVERAGE_SEGMENTS);
    if candidates.is_empty() {
        return fail("no apply row carries even one of the selected input curves");
    }

    let mut notes: Vec<String> = Vec::new();
    let mut segments: Vec<CoverageSegment> = Vec::new();
    // Per well, per output name, the accumulating curve. One curve, contributed to by several
    // models — which is the point, and why the write happens once at the end rather than per
    // segment (a per-segment write would DELETE the previous segment's rows for the same name).
    let mut acc: Vec<std::collections::BTreeMap<String, Vec<f32>>> =
        wells_data.iter().map(|_| Default::default()).collect();
    let mut predicted_rows: Vec<usize> = vec![0; wells_data.len()];
    let mut out_names_all: Vec<String> = Vec::new();
    let mut class_names: std::collections::BTreeSet<String> = Default::default();
    let mut units: Vec<(String, String)> = Vec::new();
    // Same rule as the ordinary path: a step that is not a thickness is dropped with a note rather
    // than failing a run that has already fitted every segment.
    let out_step: Option<f64> = match req.output_step {
        Some(s) if s.is_finite() && s > 0.0 => Some(s),
        Some(s) => {
            notes.push(format!(
                "an output resolution of {s} is not a thickness, so the curves are written at the input sampling instead"
            ));
            None
        }
        None => None,
    };
    let mut blocks_written = 0usize;
    let target_unit = {
        let conn = db.lock().unwrap();
        catalog_unit(&conn, &target)
    };

    for (ci, &cand) in candidates.iter().enumerate() {
        let sub: Vec<String> =
            features.iter().enumerate().filter(|(k, _)| cand & (1 << k) != 0).map(|(_, f)| f.clone()).collect();
        let sd = sub.len();
        let n_here: usize = assigned
            .iter()
            .map(|a| a.iter().filter(|v| **v == Some(ci)).count())
            .sum();
        if n_here == 0 {
            continue; // a pattern entirely absorbed by a larger one — not a segment, just a subset
        }
        if let Some(p) = progress {
            p.set_current(Some(format!("Fitting {} on {} curve(s)…", req.algorithm, sd)));
        }

        // Trains on every row carrying this subset, whatever else those rows carry.
        let (mut x_train, mut y_train, mut groups, _empty, roster) = {
            let conn = db.lock().unwrap();
            assemble_training(&conn, &req.train_well_ids, &sub, &target, mask_curve.as_ref(), req.input_set.as_deref(), req.interval)
        };
        let transform = req
            .target_transform
            .as_deref()
            .map(str::trim)
            .map(str::to_lowercase)
            .filter(|t| !t.is_empty() && t != "none")
            .unwrap_or_default();
        if !transform.is_empty() {
            if transform != "log10" {
                return fail(&format!("unknown target transform '{transform}' - use log10, or none"));
            }
            apply_target_transform(&transform, sd, &mut x_train, &mut y_train, &mut groups);
        }
        let n_train = y_train.len();
        if n_train < MIN_SEGMENT_ROWS {
            segments.push(CoverageSegment {
                features: sub,
                n_predicted: 0,
                n_train,
                blind: serde_json::Value::Null,
                model_name: None,
                skipped: Some(format!(
                    "only {n_train} training row(s) carry these curves - below {MIN_SEGMENT_ROWS} this segment is not fitted, and the {n_here} depth(s) it would have covered are left blank rather than predicted by a model nobody could defend"
                )),
            });
            continue;
        }

        // This segment's own blind split, over its own rows.
        let split_seed = req.split_seed.unwrap_or(crate::facies::SEED_DEFAULT as u64);
        let mut row_counts: std::collections::BTreeMap<usize, usize> = Default::default();
        for g in &groups {
            *row_counts.entry(*g as usize).or_insert(0) += 1;
        }
        let contributing: Vec<usize> = row_counts.keys().copied().collect();
        let counts: Vec<usize> = row_counts.values().copied().collect();
        let by_sample =
            req.split_mode.as_deref().map(str::trim).unwrap_or("well").eq_ignore_ascii_case("sample");
        let mut blind_mask: Vec<f32> = Vec::new();
        if let Some(f) = req.blind_fraction.filter(|f| *f > 0.0) {
            if by_sample {
                let strata = strata_for(&y_train, req.task == "classification");
                blind_mask = split_blind_samples(&strata, f, split_seed);
            } else {
                let pos = split_blind_wells(&counts, f, split_seed);
                let bg: Vec<usize> = pos.iter().map(|&i| contributing[i]).collect();
                blind_mask =
                    groups.iter().map(|g| if bg.contains(&(*g as usize)) { 1.0 } else { 0.0 }).collect();
            }
        }

        // The apply matrix for the depths THIS segment owns.
        let mut x_apply: Vec<f32> = Vec::new();
        let mut idx_per_well: Vec<Vec<usize>> = Vec::with_capacity(wells_data.len());
        for (wi, w) in wells_data.iter().enumerate() {
            let mut idx = Vec::new();
            for i in 0..w.depth.len() {
                if assigned[wi][i] != Some(ci) {
                    continue;
                }
                for (k, c) in w.cols.iter().enumerate() {
                    if cand & (1 << k) != 0 {
                        x_apply.push(c[i]);
                    }
                }
                idx.push(i);
            }
            idx_per_well.push(idx);
        }
        let n_apply = x_apply.len() / sd.max(1);

        let save_name = req
            .save_model_as
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            // One model per segment, and the name says which — a saved artifact whose feature list
            // the user cannot see from its name is one they will apply to the wrong thing.
            .map(|s| format!("{s}_{sd}CURVE"));
        let save_features = save_name.as_ref().map(|_| sub.as_slice());

        let run = match exec_ml_full(
            &python,
            &req.task,
            &req.algorithm,
            &req.params,
            sd,
            &x_train,
            Some(&y_train),
            &x_apply,
            n_apply,
            save_features,
            Some(&groups),
            &blind_mask,
        ) {
            Ok(r) => r,
            Err(e) => {
                segments.push(CoverageSegment {
                    features: sub,
                    n_predicted: 0,
                    n_train,
                    blind: serde_json::Value::Null,
                    model_name: None,
                    skipped: Some(format!("this segment failed to fit: {e}")),
                });
                continue;
            }
        };
        // This segment's own split report, so `blind_record` describes the rows THIS model was
        // scored on rather than the run as a whole.
        let blind_rows = blind_mask.iter().filter(|v| **v > 0.5).count();
        let seg_split = req.blind_fraction.filter(|f| *f > 0.0 && blind_rows > 0).map(|f| SplitReport {
            mode: if by_sample { "sample".into() } else { "well".into() },
            requested_fraction: f,
            achieved_fraction: blind_rows as f64 / n_train.max(1) as f64,
            // Named per segment rather than listed: the well NAMES are the same list for every
            // segment, and repeating them once per model would say the segments differ in which
            // wells they used when they differ in which curves.
            fit_wells: vec![],
            blind_wells: vec![],
            fit_rows: n_train - blind_rows,
            blind_rows,
            seed: split_seed,
            wells_pooled: contributing.len(),
        });
        let blind = blind_record(&run.metrics, seg_split.as_ref(), &req.task);

        // Save this segment's model, with the log set and mask record every other path writes.
        let mut model_name = None;
        if let Some(name) = &save_name {
            let conn = db.lock().unwrap();
            let trained_on: Vec<String> = roster.iter().filter(|r| r.rows > 0).map(|r| r.well.clone()).collect();
            let training_json = serde_json::to_string(&TrainingRecord {
                mask_curve: mask_curve.clone(),
                wells: roster.iter().filter(|r| r.rows > 0).cloned().collect(),
            })
            .ok();
            let runtime_json = (!run.runtime.is_null()).then(|| run.runtime.to_string());
            let mut m = run.metrics.clone();
            if let Some(o) = m.as_object_mut() {
                o.insert("blind".into(), blind.clone());
            }
            match crate::db::insert_ml_model(
                &conn,
                &crate::db::NewMlModel {
                    name,
                    task: &req.task,
                    algorithm: &req.algorithm,
                    feature_curves: &sub,
                    target_curve: Some(&target),
                    params_json: &serde_json::to_string(&req.params).unwrap_or_default(),
                    metrics_json: &serde_json::to_string(&m).unwrap_or_default(),
                    trained_on: &trained_on,
                    n_train,
                    standardize: req.params.get("standardize").and_then(|v| v.as_bool()).unwrap_or(true),
                    note: req.model_note.as_deref(),
                    data: &run.model_blob,
                    train_hash: Some(&training_fingerprint(&sub, sd, &x_train, &y_train, &groups)),
                    training_json: training_json.as_deref(),
                    runtime_json: runtime_json.as_deref(),
                    sklearn_version: (!run.sklearn.is_empty()).then_some(run.sklearn.as_str()),
                },
            ) {
                Ok((_, stored)) => model_name = Some(stored),
                Err(e) => notes.push(format!("the {sd}-curve segment's model was NOT saved: {e} - its curves are still written")),
            }
        }

        // SB-MLA-035, the same two curves the ordinary path writes: the model's own output in log
        // space and its back-transform in the target's unit, separately named so neither can be
        // read as the other.
        let mut seg_outs = run.outs;
        if !transform.is_empty() {
            if let Some((_, native)) = seg_outs.iter().find(|(s, _)| s.is_empty()) {
                let back: Vec<f32> =
                    native.iter().map(|v| if v.is_finite() { 10f32.powf(*v) } else { f32::NAN }).collect();
                seg_outs.push((BACK_SUFFIX.to_string(), back));
            }
        }

        // Scatter into the accumulating curves.
        let mut start = 0usize;
        for (wi, idx) in idx_per_well.iter().enumerate() {
            let m = idx.len();
            for (suffix, values) in &seg_outs {
                let name = out_name_for(&base, suffix, &transform);
                if !out_names_all.contains(&name) {
                    out_names_all.push(name.clone());
                    // Recorded HERE, where the suffix is still in hand. By the time the accumulated
                    // curves are blocked they are names only, and re-deriving "is this the class
                    // curve or the probability beside it" from a name would be guessing at exactly
                    // the point where guessing wrong averages class codes.
                    if output_is_class(&req.task, suffix) {
                        class_names.insert(name.clone());
                    }
                    // Only a regression predicts a quantity. A class code has no unit, and a blank
                    // declared for one would be a claim rather than an absence.
                    if req.task == "regression" {
                        if let Some(u) = unit_for_output(suffix, &transform, target_unit.as_deref()) {
                            units.push((name.clone(), u));
                        }
                    }
                }
                let slot = acc[wi].entry(name).or_insert_with(|| vec![f32::NAN; wells_data[wi].depth.len()]);
                for (j, &i) in idx.iter().enumerate() {
                    slot[i] = values[start + j];
                }
            }
            predicted_rows[wi] += m;
            start += m;
        }
        segments.push(CoverageSegment {
            features: sub,
            n_predicted: n_here,
            n_train,
            blind,
            model_name,
            skipped: None,
        });
    }

    // --- Write once per well ------------------------------------------------------
    let unclaimed: usize = assigned
        .iter()
        .zip(&wells_data)
        .map(|(a, w)| a.iter().zip(&w.avail).filter(|(s, av)| s.is_none() && **av != 0).count())
        .sum();
    if unclaimed > 0 {
        notes.push(format!(
            "{unclaimed} depth(s) carry a combination of inputs no fitted segment covers and were left blank - the cap is {MAX_COVERAGE_SEGMENTS} segments per run"
        ));
    }
    let fitted: Vec<&CoverageSegment> = segments.iter().filter(|s| s.skipped.is_none()).collect();
    if fitted.is_empty() {
        return fail("no coverage segment had enough training rows to fit - check the target's coverage in the training wells");
    }
    notes.push(format!(
        "fitted {} model(s), one per pattern of available inputs: {}",
        fitted.len(),
        fitted
            .iter()
            .map(|s| format!("{} over {} depth(s)", s.features.join("+"), s.n_predicted))
            .collect::<Vec<_>>()
            .join("; ")
    ));

    let mut wells_out: Vec<MlWellResult> = Vec::new();
    {
        let conn = db.lock().unwrap();
        for (wi, w) in wells_data.iter().enumerate() {
            if let Some(p) = progress {
                p.start_item(&w.well_id);
            }
            if let Some(e) = &w.error {
                if let Some(p) = progress {
                    p.finish_item(&w.well_id, crate::jobs::ItemState::Failed, Some(e.clone()));
                }
                wells_out.push(MlWellResult { well_id: w.well_id.clone(), rows_predicted: 0, error: Some(e.clone()) });
                continue;
            }
            if predicted_rows[wi] == 0 {
                // Same rule the ordinary path follows: refuse before a log set is allocated rather
                // than write an all-NaN curve that looks like a track nobody computed.
                let msg = if w.masked > 0 {
                    format!("no depth carries a usable set of inputs ({} row(s) excluded by the mask)", w.masked)
                } else {
                    "no depth carries a usable set of inputs".to_string()
                };
                if let Some(p) = progress {
                    p.finish_item(&w.well_id, crate::jobs::ItemState::Failed, Some(msg.clone()));
                }
                wells_out.push(MlWellResult { well_id: w.well_id.clone(), rows_predicted: 0, error: Some(msg) });
                continue;
            }
            // Blocked AFTER every segment has contributed, never per segment. A block spanning the
            // boundary between two segments' rock holds samples from both models, and averaging
            // them is the honest answer for that interval — blocking each segment separately would
            // instead give that block two competing values and let write order pick one.
            let mut blocked: Vec<(String, Vec<f32>)> = Vec::new();
            if let Some(step) = out_step {
                for (name, values) in &acc[wi] {
                    let mut v = values.clone();
                    let n = block_to_step(&w.depth, &mut v, step, class_names.contains(name));
                    blocks_written = blocks_written.max(n);
                    blocked.push((name.clone(), v));
                }
            }
            let curves: Vec<(&str, &[f32])> = if blocked.is_empty() {
                acc[wi].iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect()
            } else {
                blocked.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect()
            };
            let spec = crate::equations::LogSetSpec {
                set_name: out_set.clone(),
                module: format!("ml:{}:{}", req.task, req.algorithm),
                // The provenance records EVERY segment, because the curve genuinely was made by
                // several models and "which model produced this curve" has more than one answer
                // along its length. A single model reference here would name one of them and be
                // wrong about the rest.
                params_json: serde_json::to_string(&serde_json::json!({
                    "algorithm": req.algorithm,
                    "params": req.params,
                    "coverage_segments": segments,
                }))
                .unwrap_or_default(),
                inputs_json: serde_json::to_string(&features).unwrap_or_default(),
            };
            let done = crate::equations::create_log_set(&conn, &w.well_id, &spec).and_then(|(set_id, _)| {
                write_computed_curves_versioned(&conn, &w.well_id, &w.depth, &curves, &set_id)
            });
            match done {
                Ok(()) => {
                    if !units.is_empty() {
                        let _ = crate::db::declare_curve_units(&conn, &w.well_id, &units);
                    }
                    if let Some(p) = progress {
                        p.finish_item(&w.well_id, crate::jobs::ItemState::Ok, None);
                    }
                    wells_out.push(MlWellResult {
                        well_id: w.well_id.clone(),
                        rows_predicted: predicted_rows[wi],
                        error: None,
                    });
                }
                Err(e) => {
                    if let Some(p) = progress {
                        p.finish_item(&w.well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                    }
                    wells_out.push(MlWellResult {
                        well_id: w.well_id.clone(),
                        rows_predicted: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    if let Some(step) = out_step {
        notes.push(if blocks_written > 0 {
            format!(
                "written at {step} resolution: one value per {step}-thick interval, held across it, on each well's own depths (up to {blocks_written} blocks in a well). Blocks are taken AFTER every segment has contributed, so an interval spanning two segments' rock averages both rather than one of them winning. Set this curve's draw style to Step in the curve editor"
            )
        } else {
            format!(
                "an output resolution of {step} was asked for but no well had a live prediction to block, so the curves are at the input sampling"
            )
        });
    }

    MlResult {
        outputs: out_names_all,
        metrics: serde_json::json!({ "coverage_segments": segments, "output_step": out_step }),
        wells: wells_out,
        notes,
        model_id: None,
        model_name: None,
        split: None,
        error: None,
    }
}

/// The ONE log set a model's training rows all came from, or `None`.
///
/// `None` covers three different situations on purpose, because all three mean the same thing to a
/// caller: the model predates the record, it was fitted from the live store, or its wells were read
/// from more than one set. Only an unambiguous single set can be reused without choosing on the
/// user's behalf — and a model whose wells came from FINAL and RAW has no single answer to inherit.
fn training_sets(training_json: Option<&str>) -> Option<String> {
    let rec: TrainingRecord = serde_json::from_str(training_json?).ok()?;
    let mut names: Vec<&str> = rec.wells.iter().filter_map(|w| w.set_name.as_deref()).collect();
    // Every contributing well must have been read from a set, or the ones that were not came from
    // the live store and there is no single provenance to carry forward.
    if names.len() != rec.wells.len() || names.is_empty() {
        return None;
    }
    names.sort_unstable();
    names.dedup();
    match names.as_slice() {
        [one] => Some((*one).to_string()),
        _ => None,
    }
}

/// SB-MLA-008 — what would stop THIS configuration reproducing, said before the run.
///
/// The requirement's escape clause asks the product to name the source of any non-determinism rather
/// than offer a guarantee that silently does not hold. The honest scope of that is what the product
/// can OBSERVE — it is not a place to restate second-hand claims about which library is deterministic
/// on which machine, because a claim nobody here can check is worth less than no claim.
///
/// One case qualifies, and it is SandiBumi's own code rather than anybody else's: `gbdt` fits
/// `XGBRegressor` where `xgboost` is installed and substitutes scikit-learn's
/// `HistGradientBoosting` where it is not (`ML_BUILD_MODEL`). Same request, same seed, same rows,
/// two different estimators depending on the machine — and the request is recorded as `gbdt` either
/// way, which is why SB-MLA-012 stays open. Here the point is narrower: the user is about to press
/// Run, and this is the one thing about to happen that a re-run elsewhere would not repeat.
///
/// Everything else the product can see is a CROSS-RUN fact rather than a property of the algorithm —
/// the runtime has moved, the rows have changed, the input set has been superseded — and each is
/// already named where it belongs, on the model row (`model_warnings`) and in the run result.
pub fn determinism_note(task: &str, algorithm: &str) -> Option<String> {
    if task != "regression" || algorithm != "gbdt" {
        return None;
    }
    let rt = ml_runtime();
    // Absent because no Python was found at all, or absent because the probe asked and did not find
    // it. Only the second is evidence; the first means the run is not going to start anyway.
    let asked = rt.get("xgboost");
    match asked {
        Some(serde_json::Value::Null) => Some(
            "xgboost is not installed, so this run will fit scikit-learn's HistGradientBoosting \
             instead. It is recorded as 'gbdt' either way, so the same request on a machine that \
             HAS xgboost fits a different estimator and will not reproduce these curves. \
             Installing xgboost, or choosing an algorithm that does not substitute, removes the \
             ambiguity."
                .to_string(),
        ),
        _ => None,
    }
}

/// SB-MLA-002 + SB-MLA-005 — everything a saved model's row should warn about, per model.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelWarnings {
    pub model_id: String,
    pub notes: Vec<String>,
}

/// The same two checks the apply path runs, asked BEFORE anything is applied.
///
/// Computed here rather than in the picker so each check has ONE implementation and one wording. A
/// warning that reads differently in the list and in the run result reads as two different problems,
/// and the model has not changed between those two moments.
///
/// Only models with something to say appear. A list that returned a row per model, most of them
/// empty, would put the burden of finding the two that matter back on the caller.
pub fn model_warnings(conn: &Connection) -> Vec<ModelWarnings> {
    let now = ml_runtime();
    crate::db::list_ml_models(conn)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| {
            let mut notes = Vec::new();
            if !now.is_null() {
                notes.extend(runtime_drift(m.runtime_json.as_deref(), &now));
            }
            notes.extend(training_set_drift(conn, m.training_json.as_deref()));
            (!notes.is_empty()).then(|| ModelWarnings { model_id: m.model_id, notes })
        })
        .collect()
}

/// SB-MLA-002 — the recorded training log set no longer exists, or has been superseded.
///
/// The point is not that applying the model is wrong; it is that the rock the model learned from is
/// no longer what that set name returns, so a re-fit today would not reproduce this model and a
/// reviewer comparing the two would have no way to see why. Reported BY NAME, with the well, because
/// "a training set was superseded" is not actionable and "PHIE_FINAL v2 on SANDI-7 is now v3" is.
///
/// Wells are summarised rather than listed one per line past a handful: on a field-scale model this
/// is the difference between a note and a wall.
fn training_set_drift(conn: &Connection, training_json: Option<&str>) -> Vec<String> {
    let Some(rec) = training_json.and_then(|s| serde_json::from_str::<TrainingRecord>(s).ok()) else {
        return Vec::new();
    };
    let mut gone: Vec<String> = Vec::new();
    let mut superseded: Vec<String> = Vec::new();
    for r in &rec.wells {
        let (Some(set_name), Some(set_id), Some(version)) = (&r.set_name, &r.set_id, r.set_version)
        else {
            continue;
        };
        let exists: bool = conn
            .query_row("SELECT 1 FROM log_sets WHERE set_id = ?1", duckdb::params![set_id], |_| Ok(()))
            .is_ok();
        if !exists {
            gone.push(format!("{} ({set_name})", r.well));
            continue;
        }
        if let Some((_, latest)) = resolve_input_set(conn, &r.well_id, set_name) {
            if latest > version {
                superseded.push(format!("{} ({set_name} v{version} -> v{latest})", r.well));
            }
        }
    }
    let say = |what: &str, list: Vec<String>| -> Option<String> {
        if list.is_empty() {
            return None;
        }
        let shown = if list.len() > 4 {
            format!("{}, and {} more", list[..4].join(", "), list.len() - 4)
        } else {
            list.join(", ")
        };
        Some(format!("the log set this model was trained from {what}: {shown}. Re-fitting today would read different rock, so this model can no longer be reproduced from that set name"))
    };
    [say("no longer exists", gone), say("has been superseded", superseded)].into_iter().flatten().collect()
}

/// The `log_sets` row a named input set resolves to for one well — the same "latest version wins"
/// rule `fetch_curve_frame_from_set` reads by, asked separately so the answer can be RECORDED.
fn resolve_input_set(conn: &Connection, well_id: &str, set_name: &str) -> Option<(String, i64)> {
    conn.query_row(
        "SELECT set_id, version FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
         ORDER BY version DESC LIMIT 1",
        duckdb::params![well_id, set_name],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
    )
    .ok()
}

fn assemble_training(
    conn: &Connection,
    train_well_ids: &[String],
    features: &[String],
    tgt: &str,
    mask_curve: Option<&String>,
    input_set: Option<&str>,
    window: DepthWindow,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<String>, Vec<TrainWellRecord>) {
    let mut fetch_names = features.to_vec();
    fetch_names.push(tgt.to_string());
    if let Some(mk) = mask_curve {
        fetch_names.push(mk.clone());
    }
    let set_name = input_set.map(str::trim).filter(|s| !s.is_empty());
    let mut x_train: Vec<f32> = Vec::new();
    let mut y_train: Vec<f32> = Vec::new();
    // One well index per row, so the runner can hold out whole wells rather than samples.
    let mut groups: Vec<f32> = Vec::new();
    let mut empty_train: Vec<String> = Vec::new();
    let mut roster: Vec<TrainWellRecord> = Vec::new();
    for (g, well_id) in train_well_ids.iter().enumerate() {
        let before = y_train.len();
        let mut masked = 0usize;
        let mut incomplete = 0usize;
        if let Ok((depth, cols)) = fetch_curve_frame_from_set(conn, well_id, &fetch_names, input_set, None) {
            if let (Some(tv), Some(fcols)) = (
                cols.get(tgt),
                features.iter().map(|f| cols.get(f)).collect::<Option<Vec<_>>>(),
            ) {
                let mcol = mask_curve.and_then(|mk| cols.get(mk));
                for i in 0..depth.len() {
                    // MASK convention (workflow.rs): a mask value of exactly 1.0 excludes the
                    // sample from X/y; 0.0 / NaN / absent keeps it.
                    if mcol.map_or(false, |m| m[i] == 1.0) {
                        masked += 1;
                        continue;
                    }
                    // Counted with the masked rows rather than the incomplete ones: both are
                    // deliberate exclusions, whereas `incomplete` means a curve was never measured
                    // there, and a well refused for having no rows has to name which it was.
                    if !window.contains(depth[i]) {
                        masked += 1;
                        continue;
                    }
                    if tv[i].is_finite() && fcols.iter().all(|c| c[i].is_finite()) {
                        for c in &fcols {
                            x_train.push(c[i]);
                        }
                        y_train.push(tv[i]);
                        groups.push(g as f32);
                    } else {
                        incomplete += 1;
                    }
                }
            } else {
                // The well has depths but not the columns, so every one of them is incomplete.
                // Counted rather than left at zero: "0 masked, 0 incomplete, 0 rows" would read as
                // an empty well when what happened is that a curve is missing.
                incomplete = depth.len();
            }
        }
        let rows = y_train.len() - before;
        // A well that moved y_train not at all contributed nothing — unreadable, lacking the
        // target/feature, or fully masked. Record it instead of dropping it invisibly.
        if rows == 0 {
            empty_train.push(well_id.clone());
        }
        let resolved = set_name.and_then(|s| resolve_input_set(conn, well_id, s));
        roster.push(TrainWellRecord {
            well_id: well_id.clone(),
            well: well_name(conn, well_id),
            rows,
            masked,
            incomplete,
            set_name: set_name.map(str::to_string),
            set_id: resolved.as_ref().map(|(id, _)| id.clone()),
            set_version: resolved.map(|(_, v)| v),
        });
    }
    (x_train, y_train, groups, empty_train, roster)
}

/// A well's name, falling back to its id. Used wherever a record is written for a person to read —
/// a UUID in a provenance table is a value nobody can act on.
fn well_name(conn: &Connection, well_id: &str) -> String {
    conn.query_row("SELECT well_name FROM wells WHERE well_id = ?1", duckdb::params![well_id], |r| {
        r.get::<_, String>(0)
    })
    .unwrap_or_else(|_| well_id.to_string())
}

struct ApplyWell {
    well_id: String,
    depth: Vec<f32>,
    /// Row indices (into `depth`) of the complete samples sent to python, in order.
    idx: Vec<usize>,
    /// Rows the run mask excluded. Kept so a well with nothing to predict can name WHICH
    /// emptiness it is: masked out, or never measured. They call for opposite fixes -
    /// widen the mask, or go and find the missing curve (SB-MLA-013).
    masked: usize,
    error: Option<String>,
}

/// Why a well produced no rows to predict. Only called when `idx` is empty, so the well is
/// being refused either way; this decides what the refusal says.
fn no_rows_reason(aw: &ApplyWell) -> String {
    if aw.masked == 0 {
        "no depth in this well carries every input curve at once".to_string()
    } else if aw.masked == aw.depth.len() {
        format!("the run mask excluded all {} depths in this well", aw.depth.len())
    } else {
        format!(
            "no depth carries every input curve at once, after the run mask excluded {} of {}",
            aw.masked,
            aw.depth.len()
        )
    }
}

/// One segment of a coverage-segmented run: a feature subset, and what it was worth.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoverageSegment {
    /// The inputs this segment's model was fitted on, in order.
    pub features: Vec<String>,
    /// Depths across the apply wells this segment predicted.
    pub n_predicted: usize,
    pub n_train: usize,
    /// This segment's OWN blind record. Never averaged with another segment's: a three-curve model
    /// and a four-curve model are different models, and one number over both would describe neither.
    /// Carries `performed: false` where nothing was held back, never a training score in its place.
    pub blind: serde_json::Value,
    pub model_name: Option<String>,
    /// Why this segment produced nothing, where it did not.
    pub skipped: Option<String>,
}

/// Refuses a fit that would LEARN FROM manufactured detail.
///
/// A `_SIM` curve carries a spectrally simulated high-frequency component: right in its statistics,
/// arbitrary in its placement. Reading it back in as a feature or a target launders that into
/// something a model treats as measurement, and the provenance chain records only that a curve named
/// `X_SIM` was an input — which is true and tells the reader nothing about what happened.
///
/// This is the failure this whole two-curve design exists to prevent, and a naming convention alone
/// does not prevent it: the checkbox list offers every curve in the well, and `PERM_SIM` sorts
/// directly beside `PERM`. The refusal is here rather than only in the pane because the pane is one
/// caller — a workflow chain or a future batch route would otherwise walk straight past it.
///
/// Deliberately a REFUSAL and not a warning. There is no defensible reason to fit against invented
/// detail, so there is nothing for the user to weigh, and a warning would simply be clicked through.
fn refuse_simulated_inputs(features: &[String], target: Option<&str>) -> Option<String> {
    let is_sim = |c: &str| c.trim().to_uppercase().ends_with(SIM_SUFFIX);
    let mut named: Vec<String> =
        features.iter().filter(|c| is_sim(c)).map(|c| c.trim().to_uppercase()).collect();
    if target.is_some_and(is_sim) {
        named.push(target.unwrap_or_default().trim().to_uppercase());
    }
    if named.is_empty() {
        return None;
    }
    named.sort();
    named.dedup();
    Some(format!(
        "{} carries simulated detail, not measurement - it is a prediction with high-frequency \
         content added to match a target's spectrum, correct in its statistics and arbitrary in its \
         placement. A model fitted against it would learn that invented detail and report the usual \
         scores for it. Use the plain curve (the same name without {SIM_SUFFIX}) instead.",
        named.join(" and ")
    ))
}

/// The runner source, written to a temp file for the life of one run and deleted after.
///
/// **This exists because `python -c <source>` has a hard ceiling and we hit it.** Windows caps a
/// command line at about 32 KB, and the runner had grown past it — every ML feature failed at once
/// with `The filename or extension is too long. (os error 206)`, a message naming neither Python nor
/// machine learning, triggered by nothing more than added comments. Nothing guarded it, so the
/// ceiling was invisible right up to the moment it was a total outage.
///
/// Passing a path removes the ceiling rather than raising it, so the runner can be commented and
/// extended like ordinary code. `-c` is kept only for the one-line probes (`import numpy`), which
/// cannot approach any limit.
///
/// Deleted on `Drop`, so an early return or a panic cannot leave the file behind. It holds only the
/// runner's own source: no well data, no curve values, nothing client-identifying.
struct ScriptFile(std::path::PathBuf);

impl ScriptFile {
    fn new(tag: &str, source: &str) -> std::io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        // Process id AND a counter: two runs in one session must not collide, and neither must two
        // copies of the app open at once.
        let name = format!("sandibumi-{tag}-{}-{}.py", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed));
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, source)?;
        Ok(Self(path))
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for ScriptFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub fn run_ml(db: &Mutex<Connection>, req: &MlRequest, progress: Option<&crate::jobs::JobHandle>) -> MlResult {
    let supervised = matches!(req.task.as_str(), "regression" | "classification");
    // Jauhar, 2026-08-07: *"model should still run even 1 curves only half depth coverage, (model
    // only predict using 3 curves on the other half depth coverage)"*. A separate path rather than a
    // branch through this one: every stage below is written for ONE feature set and one fitted
    // model, and threading a second set through the transform, the split, the fingerprint, the model
    // save and the provenance would leave five places where a segment could silently inherit
    // another's record.
    // Checked BEFORE the coverage path branches away, or one of the two routes into a fit would not
    // be guarded at all.
    if let Some(refusal) = refuse_simulated_inputs(&req.feature_curves, req.target_curve.as_deref()) {
        return fail(&refusal);
    }
    if req.coverage_segments && supervised {
        return run_ml_coverage(db, req, progress);
    }
    let features: Vec<String> =
        req.feature_curves.iter().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty()).collect();
    if features.is_empty() {
        return fail("select at least one input curve");
    }
    if req.apply_well_ids.is_empty() {
        return fail("select at least one well to apply to");
    }
    let base = req.output_curve.trim().to_uppercase();
    if base.is_empty() {
        return fail("output curve name is empty");
    }
    let target = req.target_curve.as_deref().map(|t| t.trim().to_uppercase());
    let mask_curve =
        req.mask_curve.as_deref().map(|m| m.trim().to_uppercase()).filter(|m| !m.is_empty());
    if supervised {
        if target.as_deref().map_or(true, str::is_empty) {
            return fail("supervised learning needs a target curve");
        }
        if req.train_well_ids.is_empty() {
            return fail("supervised learning needs at least one training well");
        }
    }
    let Some(python) = find_python() else {
        return fail("no Python with numpy found - install Python 3.10+ with numpy + scikit-learn, or set SANDIBUMI_PYTHON to its python.exe");
    };

    let d = features.len();
    // Where the predictions land. Resolved once, so every well of a run is versioned into the
    // same set — a run that scattered its wells across two set names could not afterwards be
    // read back as one interpretation.
    let out_set = req
        .output_set
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ML_SET)
        .to_string();
    let mut x_train: Vec<f32> = Vec::new();
    let mut y_train: Vec<f32> = Vec::new();
    let mut groups: Vec<f32> = Vec::new();
    let mut empty_train: Vec<String> = Vec::new();
    let mut roster: Vec<TrainWellRecord> = Vec::new();
    let mut apply: Vec<ApplyWell> = Vec::new();
    let mut x_apply: Vec<f32> = Vec::new();
    {
        let conn = db.lock().unwrap();
        if supervised {
            let tgt = target.clone().unwrap();
            let (xt, yt, gt, empty, rec) =
                assemble_training(&conn, &req.train_well_ids, &features, &tgt, mask_curve.as_ref(), req.input_set.as_deref(), req.interval);
            x_train = xt;
            y_train = yt;
            groups = gt;
            empty_train = empty;
            roster = rec;
        }
        let mut apply_fetch = features.clone();
        if let Some(mk) = &mask_curve {
            apply_fetch.push(mk.clone());
        }
        let window = req.interval;
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame_from_set(&conn, well_id, &apply_fetch, req.input_set.as_deref(), None) {
                Ok((depth, cols)) => {
                    let fcols: Vec<&Vec<f32>> = features.iter().filter_map(|f| cols.get(f)).collect();
                    if fcols.len() != d || depth.is_empty() {
                        apply.push(ApplyWell {
                            well_id: well_id.clone(),
                            depth,
                            idx: vec![],
                            masked: 0,
                            error: Some("missing input curve data".into()),
                        });
                        continue;
                    }
                    let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
                    let mut idx = Vec::new();
                    let mut masked = 0usize;
                    for i in 0..depth.len() {
                        // Masked apply rows (mask == 1.0) are never sent to python, so scatter-back
                        // leaves them NaN — the OUTPUT-blanking half of the module MASK convention.
                        // A depth outside the run's interval takes the same route: it was never
                        // interpreted, and an empty sample says exactly that.
                        if mcol.map_or(false, |m| m[i] == 1.0) || !window.contains(depth[i]) {
                            masked += 1;
                            continue;
                        }
                        if fcols.iter().all(|c| c[i].is_finite()) {
                            for c in &fcols {
                                x_apply.push(c[i]);
                            }
                            idx.push(i);
                        }
                    }
                    apply.push(ApplyWell { well_id: well_id.clone(), depth, idx, masked, error: None });
                }
                Err(e) => apply.push(ApplyWell {
                    well_id: well_id.clone(),
                    depth: vec![],
                    idx: vec![],
                    masked: 0,
                    error: Some(e.to_string()),
                }),
            }
        }
    }

    // SB-MLA-035. The transform is applied HERE, on the assembled rows, so everything downstream —
    // the blind split, the strata, the folds, the scores — is in one space and there is no point at
    // which half the pipeline is in mD and half in log10.
    let transform = req
        .target_transform
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase)
        .filter(|t| !t.is_empty() && t != "none")
        .unwrap_or_default();
    let mut transform_notes: Vec<String> = Vec::new();
    if !transform.is_empty() {
        if !supervised || req.task != "regression" {
            return fail(&format!(
                "a {transform} target transform only applies to regression - a class label has no logarithm"
            ));
        }
        if transform != "log10" {
            return fail(&format!("unknown target transform '{transform}' - use log10, or none"));
        }
        let dropped = apply_target_transform(&transform, d, &mut x_train, &mut y_train, &mut groups);
        if dropped > 0 {
            transform_notes.push(format!(
                "{dropped} training sample(s) had a target of zero or less and have no logarithm, so they were dropped rather than floored - a floor would be an invented number anchoring the low end of the fit"
            ));
        }
    }

    let n_train = y_train.len();
    if supervised && n_train < 10 {
        return fail(&format!(
            "only {n_train} labelled training samples - need at least 10 (input curves + target must overlap in the training wells)"
        ));
    }
    // SB-MLA-003. Fingerprint the rows HERE — after the transform, before the blind split. After
    // the transform because a log-fitted model was fitted on different numbers, and a record that
    // could not tell those apart would be the SB-MLA-035 defect wearing a hash. Before the split
    // because the split is a deterministic function of these rows and the recorded seed and mode,
    // so this one value plus those two pin the fit rows exactly — and hashing only the fit side
    // would make an otherwise identical run look like a different training set the moment somebody
    // changed the blind percentage.
    let train_hash = supervised.then(|| training_fingerprint(&features, d, &x_train, &y_train, &groups));
    // Surface training wells that contributed nothing (wrong target mnemonic, missing input, or
    // fully masked). Without this, a 20-well selection fit on 3 wells looks like a clean 20-well
    // run — the exact silent-degradation the app's cardinal rule forbids.
    let mut notes: Vec<String> = transform_notes;
    if supervised && !empty_train.is_empty() {
        let requested = req.train_well_ids.len();
        notes.push(format!(
            "{} of {requested} training well(s) contributed no usable samples (missing the target or an input curve, or fully masked); the model was fit on the remaining {}",
            empty_train.len(),
            requested - empty_train.len()
        ));
    }
    // SB-MLA-004. "A run whose mask removed samples MUST report that count to the user." Reported
    // as a TOTAL with the worst well named, not as a per-well list: on a field run that list is
    // hundreds of lines, and the number that changes an interpreter's mind is how much rock left
    // the fit and whether it left one well in particular.
    if supervised {
        let masked: usize = roster.iter().map(|r| r.masked).sum();
        if masked > 0 {
            let total = masked + roster.iter().map(|r| r.rows + r.incomplete).sum::<usize>();
            let share = if total > 0 { 100.0 * masked as f64 / total as f64 } else { 0.0 };
            let worst = roster.iter().max_by_key(|r| r.masked).map(|r| (r.well.clone(), r.masked));
            let mut msg = format!(
                "the mask curve {} excluded {masked} of {total} training sample(s) ({share:.0}%)",
                mask_curve.as_deref().unwrap_or("MASK"),
            );
            if let Some((w, m)) = worst {
                // A mask that removed a fifth of the field evenly and one that emptied a single
                // well are different situations with the same total.
                msg.push_str(&format!("; most from {w} ({m})"));
            }
            notes.push(msg);
        }
    }
    // SB-MLA-002, the fit-side half. A named input set that a well does not have is NOT an error —
    // `fetch_curve_frame_from_set` degrades to the current store — but it is a silent change of
    // provenance for that well, and a model recorded as "trained from FINAL" whose rows partly came
    // from live values is exactly the confusion the requirement exists to prevent.
    if supervised {
        if let Some(set) = req.input_set.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let missing: Vec<&str> =
                roster.iter().filter(|r| r.set_id.is_none()).map(|r| r.well.as_str()).collect();
            if !missing.is_empty() {
                notes.push(format!(
                    "{} of {} training well(s) have no log set named '{set}', so their rows were read from the CURRENT store instead: {}",
                    missing.len(),
                    roster.len(),
                    missing.join(", ")
                ));
            }
        }
    }
    let n_apply = x_apply.len() / d;
    if n_apply == 0 {
        // Masking is a second, independent way to empty the pool — don't blame missing inputs.
        let cause = if mask_curve.is_some() {
            "every row is missing an input or excluded by the mask"
        } else {
            "every row has at least one missing input"
        };
        return fail(&format!("no complete samples in the apply wells ({cause})"));
    }

    // The fit + predict is one opaque subprocess; show it as an indeterminate phase, then report
    // the per-well writeback (the panel's items are the apply wells).
    if let Some(p) = progress {
        p.set_current(Some(format!("Training {} model on {} samples…", req.algorithm, n_train)));
    }
    let y_opt = if supervised { Some(y_train.as_slice()) } else { None };
    // Only a supervised fit is a reusable artifact; asking to save a clustering would promise
    // something the design does not mean (see MlRequest::save_model_as).
    let save_name = req
        .save_model_as
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && supervised)
        .map(str::to_string);
    let save_features = save_name.as_ref().map(|_| features.as_slice());

    // The blind split is decided HERE, not in the runner: the assignment has to be reported and
    // re-runnable whatever the subprocess does with it. Only wells that actually contributed
    // rows can be held out — holding back a well that turned out to be empty would reserve a
    // blind set of nothing and score the model on it.
    let split_seed = req.split_seed.unwrap_or(crate::facies::SEED_DEFAULT as u64);
    // Rows per contributing well, in one pass — the fraction is a share of THESE, so the counts
    // have to come from the pooled matrix rather than from the requested well list (a well that
    // contributed nothing is not 20% of a five-well field).
    let mut row_counts: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    for g in &groups {
        *row_counts.entry(*g as usize).or_insert(0) += 1;
    }
    let contributing: Vec<usize> = row_counts.keys().copied().collect();
    let counts: Vec<usize> = row_counts.values().copied().collect();
    let by_sample = req.split_mode.as_deref().map(str::trim).unwrap_or("well").eq_ignore_ascii_case("sample");
    let asked = req.blind_fraction.filter(|f| supervised && *f > 0.0);

    // Both modes end as ONE row mask over the pooled training rows, which is what the runner
    // takes. Holding out whole wells is just a particular mask, so the subprocess has a single
    // code path and the two modes cannot diverge in how the fit is done — only in what is drawn.
    let mut blind_mask: Vec<f32> = Vec::new();
    let mut blind_groups: Vec<usize> = Vec::new();
    if let Some(f) = asked {
        if by_sample {
            let strata = strata_for(&y_train, req.task == "classification");
            blind_mask = split_blind_samples(&strata, f, split_seed);
        } else {
            let blind_pos = split_blind_wells(&counts, f, split_seed);
            blind_groups = blind_pos.iter().map(|&i| contributing[i]).collect();
            blind_mask = groups
                .iter()
                .map(|g| if blind_groups.contains(&(*g as usize)) { 1.0 } else { 0.0 })
                .collect();
        }
    }

    let split = asked.map(|f| {
        let blind_rows = blind_mask.iter().filter(|v| **v > 0.5).count();
        let total_rows: usize = counts.iter().sum();
        SplitReport {
            // Named only where naming them means something — in sample mode every well is on
            // both sides, and printing all of them under "Held blind" would be a lie of layout.
            fit_wells: if by_sample {
                Vec::new()
            } else {
                contributing
                    .iter()
                    .filter(|g| !blind_groups.contains(g))
                    .filter_map(|&g| req.train_well_ids.get(g).cloned())
                    .collect()
            },
            blind_wells: if by_sample {
                Vec::new()
            } else {
                blind_groups.iter().filter_map(|&g| req.train_well_ids.get(g).cloned()).collect()
            },
            fit_rows: total_rows - blind_rows,
            blind_rows,
            requested_fraction: f,
            achieved_fraction: if total_rows == 0 {
                0.0
            } else {
                blind_rows as f64 / total_rows as f64
            },
            seed: split_seed,
            mode: if by_sample { "sample".into() } else { "well".into() },
            wells_pooled: contributing.len(),
        }
    });

    match exec_ml_full(
        &python,
        &req.task,
        &req.algorithm,
        &req.params,
        d,
        &x_train,
        y_opt,
        &x_apply,
        n_apply,
        save_features,
        if supervised { Some(groups.as_slice()) } else { None },
        &blind_mask,
    ) {
        Err(e) => fail(&e),
        Ok(MlRun { mut metrics, outs, model_blob, sklearn, runtime }) => {
            // SB-MLA-001. The runner reports what IT defaulted; the choices Rust made are Rust's
            // to record, and the blind-split seed is one of them — defaulted here, and the single
            // parameter that decides which wells the reported blind score is a score of.
            if let Some(eff) = metrics.get_mut("effective_params").and_then(|v| v.as_object_mut()) {
                if asked.is_some() {
                    eff.insert(
                        "split_seed".into(),
                        serde_json::json!({
                            "value": split_seed,
                            "defaulted": req.split_seed.is_none(),
                            "source": "ml.rs run_ml default",
                        }),
                    );
                    // The mode changes what the blind score is a claim ABOUT, so it belongs in the
                    // record beside the seed — a re-run from a record that omits it would produce
                    // the same number meaning something else.
                    eff.insert(
                        "split_mode".into(),
                        serde_json::json!({
                            "value": if by_sample { "sample" } else { "well" },
                            "defaulted": req.split_mode.is_none(),
                            "source": "ml.rs run_ml default",
                        }),
                    );
                }
            }
            // What gets PERSISTED is the effective set, not the supplied one — a re-run cannot be
            // built from a record that omits the values nobody typed.
            let params_record = metrics
                .get("effective_params")
                .cloned()
                .unwrap_or_else(|| serde_json::Value::Object(req.params.clone()));
            let params_json = serde_json::to_string(&params_record).unwrap_or_default();

            // SB-MLA-035. Under a transform the model's own output is in log space and is NAMED so:
            // `<base>_LOG10`, never `<base>`. The back-transform is written too, because a
            // permeability is what the user asked for — but as a SECOND, separately named and
            // separately united curve, so the two can never be mistaken for one another. That is the
            // "explicit, logged step" the requirement asks for; an in-place back-transform would be
            // the invisible one.
            let mut outs = outs;
            if !transform.is_empty() {
                if let Some((_, native)) = outs.iter().find(|(s, _)| s.is_empty()) {
                    let back: Vec<f32> =
                        native.iter().map(|v| if v.is_finite() { 10f32.powf(*v) } else { f32::NAN }).collect();
                    outs.push((BACK_SUFFIX.to_string(), back));
                }
            }
            let out_names: Vec<String> =
                outs.iter().map(|(s, _)| out_name_for(&base, s, &transform)).collect();
            // The vertical resolution the curves will be written at. Resolved HERE rather than at
            // the write loop because the model is saved before the curves are, and the resolution a
            // model was made to write at has to travel with it — an apply run that quietly reverted
            // to the input sampling would produce a curve at a different resolution from the one the
            // fit was reviewed at, under the same model's name.
            //
            // A non-positive or non-finite step is dropped rather than refused: the fit has already
            // happened, and losing a field's predictions over a mistyped box is the more expensive
            // failure. It says so instead.
            let out_step: Option<f64> = match req.output_step {
                Some(s) if s.is_finite() && s > 0.0 => Some(s),
                Some(s) => {
                    notes.push(format!(
                        "an output resolution of {s} is not a thickness, so the curves are written at the input sampling instead"
                    ));
                    None
                }
                None => None,
            };
            if let Some(step) = out_step {
                metrics["output_step"] = serde_json::json!(step);
            }
            if !req.interval.is_open() {
                // Said on every confined run, whether or not it changed the row count. A model
                // fitted over one zone and read as a whole-well answer is the error this prevents,
                // and it is only preventable if the confinement is on the record beside the score.
                notes.push(format!(
                    "confined to {} - both the rows this was fitted on and the depths it wrote. Its scores describe that interval and nothing above or below it",
                    req.interval.describe()
                ));
            }
            if !req.interval.is_open() {
                metrics["interval"] = serde_json::json!({ "top": req.interval.top, "base": req.interval.base });
            }
            // Measured on the model's OWN output (the untransformed one), against the target rows it
            // was fitted on. Reported, never corrected: see `resolution_note`.
            if let Some(tgt) = target.as_deref() {
                if let Some((_, native)) = outs.iter().find(|(sfx, _)| sfx.is_empty()) {
                    if let Some(n) = resolution_note(&y_train, native, tgt) {
                        notes.push(n);
                    }
                }
            }
            let mut blocks_written = 0usize;
            let mut wells = Vec::new();
            let conn = db.lock().unwrap();
            // The unit of every curve about to be written, so a reader can tell log10(mD) from mD
            // (SB-MLA-035). A blank target unit stays blank: "we do not know" must not be dressed
            // up as "dimensionless".
            let target_unit = target
                .as_deref()
                .and_then(|t| catalog_unit(&conn, t))
                .unwrap_or_default();
            // Declared for a REGRESSION run whether or not a transform ran. Untransformed, the
            // prediction is in the target's own unit and saying so costs nothing; transformed, the
            // two curves carry different units and saying so is the whole requirement. A classifier
            // predicts a class code, which has no unit — and a blank here would be a claim, not an
            // absence, so those curves are left out of the declaration entirely.
            let units: Vec<(String, String)> = if req.task != "regression" {
                Vec::new()
            } else {
                out_names
                    .iter()
                    .zip(&outs)
                    .filter_map(|(name, (s, _))| {
                        unit_for_output(s, &transform, Some(&target_unit)).map(|u| (name.clone(), u))
                    })
                    .collect()
            };
            if !transform.is_empty() {
                let log_name = format!("{base}{LOG10_SUFFIX}");
                let shown = transformed_unit(&transform, Some(&target_unit));
                notes.push(format!(
                    "fitted on {transform}(target): {log_name} is the model's own output in {shown}, and {base} is its back-transform. Every score below is in {shown} - an R2 in log space is not the same claim as an R2 in {}",
                    if target_unit.is_empty() { "the target's own unit".into() } else { target_unit.clone() },
                ));
                metrics["target_transform"] = serde_json::json!(transform);
                metrics["metric_space"] = serde_json::json!(shown);
            }
            // SB-MLA-009. Built here, stored in the metrics that go into the model record, and put
            // on every curve this run writes — so the apply path can copy it rather than re-derive
            // it, and one definition serves both.
            let blind = supervised.then(|| blind_record(&metrics, split.as_ref(), &req.task));
            if let Some(b) = &blind {
                metrics["blind"] = b.clone();
            }
            // Keep the fit as an artifact, BEFORE the curves are written (SB-MLA-006).
            //
            // This used to run after the well loop, so a storage problem here could not cost the
            // predictions. It still cannot: every failure below is a NOTE, never a return, and the
            // curves are written either way. What the old ordering did cost was the provenance -
            // the model id did not exist yet when each log set was created, so a curve could not
            // name the model that made it. The asymmetry was backwards: the apply path, the cheap
            // case where the model is already named, recorded it, while the fit path - whose
            // configuration is by far the harder one to reconstruct - did not.
            let (mut model_id, mut model_name) = (None, None);
            if let Some(name) = &save_name {
                if model_blob.is_empty() {
                    let why = metrics
                        .get("model_save_error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("the Python side returned no model (is joblib installed?)");
                    notes.push(format!("the model was NOT saved: {why}"));
                } else {
                    let trained_on: Vec<String> = req
                        .train_well_ids
                        .iter()
                        .filter(|id| !empty_train.contains(id))
                        .map(|id| {
                            conn.query_row(
                                "SELECT well_name FROM wells WHERE well_id = ?1",
                                duckdb::params![id],
                                |r| r.get::<_, String>(0),
                            )
                            .unwrap_or_else(|_| id.clone())
                        })
                        .collect();
                    // SB-MLA-002 + SB-MLA-004. The roster is stored for the wells that actually
                    // CONTRIBUTED — a well that gave the fit nothing was not part of the training
                    // rock, and listing it with `rows: 0` beside the others would make `n_train`
                    // and the roster disagree about how many wells trained the model.
                    let training_json = serde_json::to_string(&TrainingRecord {
                        // SB-MLA-004. The mask BY NAME, or an explicit `None` — the two are
                        // different facts about the run, not a value and its absence.
                        mask_curve: mask_curve.clone(),
                        wells: roster.iter().filter(|r| r.rows > 0).cloned().collect(),
                    })
                    .ok();
                    let runtime_json = (!runtime.is_null()).then(|| runtime.to_string());
                    match crate::db::insert_ml_model(
                        &conn,
                        &crate::db::NewMlModel {
                            name,
                            task: &req.task,
                            algorithm: &req.algorithm,
                            feature_curves: &features,
                            target_curve: target.as_deref(),
                            params_json: &params_json,
                            metrics_json: &serde_json::to_string(&metrics).unwrap_or_default(),
                            trained_on: &trained_on,
                            n_train,
                            standardize: req
                                .params
                                .get("standardize")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true),
                            note: req.model_note.as_deref(),
                            data: &model_blob,
                            train_hash: train_hash.as_deref(),
                            training_json: training_json.as_deref(),
                            runtime_json: runtime_json.as_deref(),
                            sklearn_version: (!sklearn.is_empty()).then_some(sklearn.as_str()),
                        },
                    ) {
                        Ok((id, stored)) => {
                            if &stored != name {
                                notes.push(format!(
                                    "a model named '{name}' already exists, so this one was saved as '{stored}' - retraining makes a NEW model rather than replacing the one an existing curve was made with"
                                ));
                            }
                            model_id = Some(id);
                            model_name = Some(stored);
                        }
                        Err(e) => notes.push(format!("the model was NOT saved: {e} - the curves are still written, and name the algorithm rather than a model")),
                    }
                }
            }
            let mut start = 0usize;
            // SB-MLA-017. The set ids this run actually wrote, and whether a cancel cut it short.
            let mut written_sets: Vec<String> = Vec::new();
            let mut cancelled_wells = 0usize;
            if let Some(p) = progress {
                p.set_current(Some("Writing predictions…".into()));
            }
            for aw in &apply {
                // Cancel before this well's predictions are written. The sklearn fit upstream is a
                // blocking child process and is not interruptible, but the write-back loop is, so
                // a late Cancel at least stops the remaining wells getting curves they should not.
                if progress.map_or(false, |p| p.is_cancelled()) {
                    if let Some(p) = progress {
                        p.finish_item(&aw.well_id, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                    }
                    cancelled_wells += 1;
                    wells.push(MlWellResult {
                        well_id: aw.well_id.clone(),
                        rows_predicted: 0,
                        error: Some("cancelled".into()),
                    });
                    continue;
                }
                if let Some(p) = progress {
                    p.start_item(&aw.well_id);
                }
                if let Some(e) = &aw.error {
                    if let Some(p) = progress {
                        p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(e.clone()));
                    }
                    wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: 0, error: Some(e.clone()) });
                    continue;
                }
                let m = aw.idx.len();
                // A well with nothing to predict is REFUSED here, before a log set is allocated
                // and before anything is written. Writing the all-NaN curve first and reporting
                // the failure afterwards is the SB-MLA-013 defect: on the log view an all-missing
                // track is indistinguishable from one that was never computed, so the failure is
                // not merely silent, it is disguised as an absence of work.
                if m == 0 {
                    let msg = no_rows_reason(aw);
                    if let Some(p) = progress {
                        p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(msg.clone()));
                    }
                    wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: 0, error: Some(msg) });
                    continue;
                }
                let mut curves: Vec<(String, Vec<f32>)> = Vec::with_capacity(outs.len());
                for ((suffix, values), name) in outs.iter().zip(&out_names) {
                    let mut full = vec![f32::NAN; aw.depth.len()];
                    for (j, &i) in aw.idx.iter().enumerate() {
                        full[i] = values[start + j];
                    }
                    if let Some(step) = out_step {
                        let n = block_to_step(&aw.depth, &mut full, step, output_is_class(&req.task, suffix));
                        blocks_written = blocks_written.max(n);
                    }
                    curves.push((name.clone(), full));
                }
                let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
                // SB-MLA-006. The model reference rides in `params_json` beside the effective
                // parameters, in the SAME shape the apply path writes, so one reader answers
                // "which model made this curve?" for both paths. `module` keeps its existing
                // `ml:<task>:<algorithm>` spelling — a curve made by a fit and a curve made by an
                // apply are different events and the catalog should keep saying which.
                //
                // Where no model was kept the reference is absent rather than empty. That IS the
                // answer for such a curve: it was made by a fit nobody preserved, and a null
                // model_id says so without inviting a lookup that must fail.
                let spec = crate::equations::LogSetSpec {
                    set_name: out_set.clone(),
                    module: format!("ml:{}:{}", req.task, req.algorithm),
                    // SB-MLA-009 rides in the same object as SB-MLA-006's model reference, and it
                    // rides on EVERY fit-path curve — including one whose model was not kept, since
                    // "how well does this travel" is a question about the curve, not about whether
                    // anybody saved the fit that made it.
                    params_json: {
                        let mut rec = serde_json::json!({
                            "algorithm": req.algorithm,
                            "params": params_record,
                        });
                        // SB-MLA-011. Whether THIS well trained the model or only received its
                        // predictions is the difference between an interpolation and an
                        // extrapolation, and it is the first thing a reviewer asks. It was visible
                        // only as a run-time warning, which is to say it was visible for as long as
                        // the pane stayed open; on the curve it was invisible. A well selected for
                        // training that contributed nothing is recorded as its own case rather than
                        // folded into "applied only" — the user believed it was training rock, and
                        // the record should say the fit disagreed.
                        rec["well_role"] = serde_json::json!(if !req.train_well_ids.contains(&aw.well_id) {
                            "applied only - this well did not train the model, so its curve is an extrapolation from other wells"
                        } else if empty_train.contains(&aw.well_id) {
                            "selected for training but contributed no usable rows - it did NOT train the model, so its curve is an extrapolation"
                        } else {
                            "trained and applied - this well's own rock is part of what the model learned from"
                        });
                        rec["n_trained_wells"] = serde_json::json!(
                            req.train_well_ids.iter().filter(|id| !empty_train.contains(*id)).count()
                        );
                        rec["n_applied_wells"] = serde_json::json!(apply.len());
                        if let (Some(id), Some(nm)) = (&model_id, &model_name) {
                            rec["model_id"] = serde_json::json!(id);
                            rec["model_name"] = serde_json::json!(nm);
                        }
                        if let Some(b) = &blind {
                            rec["blind"] = b.clone();
                        }
                        if let Some(h) = &train_hash {
                            rec["train_hash"] = serde_json::json!(h);
                        }
                        serde_json::to_string(&rec).unwrap_or_else(|_| params_json.clone())
                    },
                    inputs_json: serde_json::to_string(&req.feature_curves).unwrap_or_default(),
                };
                let versioned = crate::equations::create_log_set(&conn, &aw.well_id, &spec).and_then(|(set_id, _)| {
                    write_computed_curves_versioned(&conn, &aw.well_id, &aw.depth, &refs, &set_id).map(|()| set_id)
                });
                match versioned {
                    Ok(set_id) => {
                        // SB-MLA-017. Kept so a cancel arriving later can stamp the sets this run
                        // already wrote. A set that failed to write is deliberately not in here:
                        // there is nothing to qualify.
                        written_sets.push(set_id);
                        // SB-MLA-035. The unit is declared with the curve, not left to be inferred
                        // from the mnemonic later. Like the model save above, a failure here is a
                        // note rather than a return — the prediction is written either way.
                        if !units.is_empty() {
                            if let Err(e) = crate::db::declare_curve_units(&conn, &aw.well_id, &units) {
                                notes.push(format!("the unit of the new curves was not recorded: {e}"));
                            }
                        }
                        if let Some(p) = progress {
                            p.finish_item(&aw.well_id, crate::jobs::ItemState::Ok, None);
                        }
                        wells.push(MlWellResult {
                            well_id: aw.well_id.clone(),
                            rows_predicted: m,
                            error: None,
                        });
                    }
                    Err(e) => {
                        if let Some(p) = progress {
                            p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                        }
                        wells.push(MlWellResult {
                            well_id: aw.well_id.clone(),
                            rows_predicted: 0,
                            error: Some(e.to_string()),
                        });
                    }
                }
                start += m;
            }

            // SB-MLA-017. A partially written set is the worst artifact this pane can leave: on the
            // Wells pane it is indistinguishable from a completed run over a smaller well selection,
            // because the set name and the module string are the ones a complete run writes. So the
            // sets that DID get written say what they are.
            if cancelled_wells > 0 && !written_sets.is_empty() {
                let n = mark_cancelled_sets(&conn, &written_sets, written_sets.len(), apply.len());
                notes.push(format!(
                    "this run was CANCELLED after {} of {} well(s) - the {n} log set(s) already written are marked as coming from a cancelled run, so they are not mistaken for a complete run over fewer wells. The wells that were cut are listed above with 'cancelled'",
                    written_sets.len(),
                    apply.len()
                ));
                metrics["cancelled"] = serde_json::json!({
                    "wells_written": written_sets.len(),
                    "wells_in_scope": apply.len(),
                });
            }

            if let Some(step) = out_step {
                // Stated whether or not it changed anything, and the "no blocks" case is stated too:
                // a resolution setting that silently did nothing is the one a reader would go on
                // trusting. The step-draw reminder is here for the same reason `frame::block` puts
                // it in its own doc — the layout is where draw style lives, and a blocked curve
                // drawn as a line shows a gradient between two block values that nothing measured.
                notes.push(if blocks_written > 0 {
                    format!(
                        "written at {step} resolution: one value per {step}-thick interval, held across it, on each well's own depths (up to {blocks_written} blocks in a well). Set this curve's draw style to Step in the curve editor, or the log view draws a gradient between two block values that nothing measured"
                    )
                } else {
                    format!(
                        "an output resolution of {step} was asked for but no well had a live prediction to block, so the curves are at the input sampling"
                    )
                });
                metrics["output_step"] = serde_json::json!(step);
            }

            MlResult { outputs: out_names, metrics, wells, notes, model_id, model_name, split, error: None }
        }
    }
}

/// SB-MLA-017 — stamps the log sets a cancelled run DID write with the fact that it was cancelled.
///
/// Returns how many sets were successfully marked, which is not always `set_ids.len()`: this runs
/// after the curves are already stored, so a failure here must cost the mark, never the work.
///
/// **Written after the fact, and that is not the objection it looks like.** Marking a curve's run
/// record months later to describe a separate event — a model deleted in a different session — would
/// be rewriting history, which is why `ml_provenance` derives that case at read time instead. A
/// cancellation is not a separate event: it is how THIS run ended, and the run record is not complete
/// until the run is. Stamping it here finishes the record rather than revising it.
fn mark_cancelled_sets(conn: &Connection, set_ids: &[String], written: usize, in_scope: usize) -> usize {
    let mut done = 0usize;
    for set_id in set_ids {
        let current: Option<String> = conn
            .query_row("SELECT params_json FROM log_sets WHERE set_id = ?1", duckdb::params![set_id], |r| r.get(0))
            .ok()
            .flatten();
        let mut rec: serde_json::Value = current
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        if !rec.is_object() {
            // A params record that is not an object cannot carry the mark, and replacing it would
            // throw away whatever it was. Leave it and let the count say fewer were marked.
            continue;
        }
        rec["cancelled"] = serde_json::json!({
            "wells_written": written,
            "wells_in_scope": in_scope,
            // In words, because this object is read by a person deciding whether to deliver the
            // curve, not only by code deciding whether to show a badge.
            "note": format!(
                "this run was cancelled after {written} of {in_scope} well(s); the field is covered in part, and the wells missing this set were cut, not excluded"
            ),
        });
        let Ok(text) = serde_json::to_string(&rec) else { continue };
        if conn
            .execute("UPDATE log_sets SET params_json = ?1 WHERE set_id = ?2", duckdb::params![text, set_id])
            .is_ok()
        {
            done += 1;
        }
    }
    done
}

/// Applies a saved model to wells it has never seen. Never fits anything.
pub fn apply_ml_model(
    db: &Mutex<Connection>,
    req: &MlApplyRequest,
    progress: Option<&crate::jobs::JobHandle>,
) -> MlResult {
    if req.apply_well_ids.is_empty() {
        return fail("select at least one well to apply to");
    }
    let base = req.output_curve.trim().to_uppercase();
    if base.is_empty() {
        return fail("output curve name is empty");
    }
    let Some(python) = find_python() else {
        return fail("no Python with numpy found - install Python 3.10+ with numpy + scikit-learn, or set SANDIBUMI_PYTHON to its python.exe");
    };
    let mask_curve = req.mask_curve.as_deref().map(|m| m.trim().to_uppercase()).filter(|m| !m.is_empty());
    let window = req.interval;

    let (info, blob) = {
        let conn = db.lock().unwrap();
        match crate::db::get_ml_model(&conn, &req.model_id) {
            Ok(v) => v,
            Err(e) => return fail(&format!("saved model not found: {e}")),
        }
    };
    // The model's OWN feature list drives the fetch. The caller never restates it, so it cannot
    // reorder it — the ordering contract is enforced by construction here and re-checked inside
    // the artifact by the runner.
    let features = info.feature_curves.clone();
    let d = features.len();
    if d == 0 {
        return fail("this saved model records no input curves");
    }

    // ...and so does its log set, for exactly the same reason the feature ORDER does.
    //
    // Jauhar, 2026-08-07: *"user dont need to re input well, data, rerun model again to
    // propagate"*. The features were already locked to the artifact; the SET they are read from was
    // not, so a model fitted on FINAL porosity could be applied against the live store and nothing
    // anywhere would say so. That is the same class of defect as a reordered matrix — it computes,
    // it plots, and the curve is quietly a different quantity — and it was the one half of the
    // contract still taken from the caller.
    //
    // The caller may still override. What it may not do is override SILENTLY.
    let mut notes: Vec<String> = Vec::new();
    let model_set: Option<String> = training_sets(info.training_json.as_deref());
    let input_set: Option<String> = match (req.input_set.as_deref().map(str::trim).filter(|s| !s.is_empty()), &model_set) {
        // Nothing asked for, and the model remembers: use what it was fitted on.
        (None, Some(set)) => {
            notes.push(format!(
                "read from log set '{set}', the set this model was fitted on - propagating it does not need the inputs restated"
            ));
            Some(set.clone())
        }
        (Some(asked), Some(set)) if !asked.eq_ignore_ascii_case(set) => {
            notes.push(format!(
                "this model was fitted on log set '{set}' and is being applied against '{asked}'. The curves may carry the same names and different values, so its blind score does not describe this run"
            ));
            Some(asked.to_string())
        }
        (asked, _) => asked.map(str::to_string),
    };
    // ...and so does the resolution it was made to write at. The same argument again: a fit reviewed
    // as a 0.5 m answer, propagated at the input sampling, is a curve at a different vertical
    // resolution from the one that was signed off, carrying the same model's name. The model records
    // it, so the apply path does not have to be told.
    let out_step: Option<f64> = serde_json::from_str::<serde_json::Value>(&info.metrics_json)
        .ok()
        .and_then(|m| m.get("output_step").and_then(|v| v.as_f64()))
        .filter(|s| s.is_finite() && *s > 0.0);
    if let Some(step) = out_step {
        notes.push(format!(
            "written at {step} resolution, the resolution this model was fitted to write at - one value per {step}-thick interval, held across it, on each well's own depths. Set the curve's draw style to Step in the curve editor"
        ));
    }
    let mut blocks_written = 0usize;

    let mut apply: Vec<ApplyWell> = Vec::new();
    let mut x_apply: Vec<f32> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let mut fetch = features.clone();
        if let Some(mk) = &mask_curve {
            fetch.push(mk.clone());
        }
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame_from_set(&conn, well_id, &fetch, input_set.as_deref(), None) {
                Ok((depth, cols)) => {
                    // Name the curve that is missing. "missing input curve data" sends somebody
                    // hunting through five mnemonics; the model knows exactly which it needs.
                    let absent: Vec<&str> =
                        features.iter().filter(|f| !cols.contains_key(*f)).map(String::as_str).collect();
                    if !absent.is_empty() || depth.is_empty() {
                        let msg = if depth.is_empty() {
                            "this well has no curve data".to_string()
                        } else {
                            format!("missing input curve(s): {}", absent.join(", "))
                        };
                        apply.push(ApplyWell {
                            well_id: well_id.clone(),
                            depth,
                            idx: vec![],
                            masked: 0,
                            error: Some(msg),
                        });
                        continue;
                    }
                    let fcols: Vec<&Vec<f32>> = features.iter().filter_map(|f| cols.get(f)).collect();
                    let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
                    let mut idx = Vec::new();
                    let mut masked = 0usize;
                    for i in 0..depth.len() {
                        if mcol.map_or(false, |m| m[i] == 1.0) || !window.contains(depth[i]) {
                            masked += 1;
                            continue;
                        }
                        if fcols.iter().all(|c| c[i].is_finite()) {
                            for c in &fcols {
                                x_apply.push(c[i]);
                            }
                            idx.push(i);
                        }
                    }
                    apply.push(ApplyWell { well_id: well_id.clone(), depth, idx, masked, error: None });
                }
                Err(e) => apply.push(ApplyWell {
                    well_id: well_id.clone(),
                    depth: vec![],
                    idx: vec![],
                    masked: 0,
                    error: Some(e.to_string()),
                }),
            }
        }
    }

    let n_apply = x_apply.len() / d;
    if n_apply == 0 {
        return fail("no complete samples in the apply wells (every row is missing one of the model's input curves, or is masked)");
    }
    if let Some(p) = progress {
        p.set_current(Some(format!("Applying '{}' to {n_apply} samples…", info.name)));
    }

    let header = serde_json::json!({
        "d": d, "n_apply": n_apply, "model_len": blob.len(), "features": features,
    });
    let script = match ScriptFile::new("apply", &ml_apply_runner()) {
        Ok(s) => s,
        Err(e) => return fail(&format!("could not write the runner to a temporary file: {e}")),
    };
    let mut cmd = Command::new(&python);
    cmd.arg(script.path()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return fail(&format!("failed to start python: {e}")),
    };
    {
        let Some(stdin) = child.stdin.as_mut() else { return fail("failed to open python stdin") };
        let mut write = || -> std::io::Result<()> {
            stdin.write_all(header.to_string().as_bytes())?;
            stdin.write_all(b"\n")?;
            stdin.write_all(&blob)?;
            stdin.write_all(bytemuck::cast_slice(&x_apply))
        };
        if let Err(e) = write() {
            return fail(&e.to_string());
        }
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return fail(&e.to_string()),
    };
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("applying the model failed");
        return fail(last.trim());
    }
    let Some(nl) = output.stdout.iter().position(|&b| b == b'\n') else {
        return fail("python returned no result header");
    };
    #[derive(Deserialize)]
    struct OutHeader {
        suffixes: Vec<String>,
        metrics: serde_json::Value,
        #[serde(default)]
        runtime: serde_json::Value,
    }
    let hdr: OutHeader = match serde_json::from_slice(&output.stdout[..nl]) {
        Ok(h) => h,
        Err(e) => return fail(&format!("bad apply result header: {e}")),
    };
    let body = &output.stdout[nl + 1..];
    if body.len() != hdr.suffixes.len() * n_apply * 4 {
        return fail(&format!("python returned {} result bytes, expected {}", body.len(), hdr.suffixes.len() * n_apply * 4));
    }
    let mut outs: Vec<(String, Vec<f32>)> = Vec::with_capacity(hdr.suffixes.len());
    for (i, s) in hdr.suffixes.iter().enumerate() {
        let mut vals = vec![0f32; n_apply];
        bytemuck::cast_slice_mut::<f32, u8>(&mut vals)
            .copy_from_slice(&body[i * n_apply * 4..(i + 1) * n_apply * 4]);
        outs.push((s.clone(), vals));
    }

    let out_names: Vec<String> = outs.iter().map(|(s, _)| format!("{base}{s}")).collect();
    let mut wells = Vec::new();
    let conn = db.lock().unwrap();
    let mut start = 0usize;
    if let Some(p) = progress {
        p.set_current(Some("Writing predictions…".into()));
    }
    for aw in &apply {
        if let Some(p) = progress {
            p.start_item(&aw.well_id);
        }
        if let Some(e) = &aw.error {
            if let Some(p) = progress {
                p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(e.clone()));
            }
            wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: 0, error: Some(e.clone()) });
            continue;
        }
        let m = aw.idx.len();
        // Refused before anything is written — see the same guard on the fit path (SB-MLA-013).
        if m == 0 {
            let msg = no_rows_reason(aw);
            if let Some(p) = progress {
                p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(msg.clone()));
            }
            wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: 0, error: Some(msg) });
            continue;
        }
        let mut curves: Vec<(String, Vec<f32>)> = Vec::with_capacity(outs.len());
        for ((suffix, values), name) in outs.iter().zip(&out_names) {
            let mut full = vec![f32::NAN; aw.depth.len()];
            for (j, &i) in aw.idx.iter().enumerate() {
                full[i] = values[start + j];
            }
            if let Some(step) = out_step {
                let n = block_to_step(&aw.depth, &mut full, step, output_is_class(&info.task, suffix));
                blocks_written = blocks_written.max(n);
            }
            curves.push((name.clone(), full));
        }
        let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
        // Provenance names the MODEL, not just the algorithm: "which model produced this curve"
        // is the question saving them was meant to answer.
        let spec = crate::equations::LogSetSpec {
            set_name: req.output_set.as_deref().map(str::trim).filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_ML_SET).to_string(),
            module: format!("ml:apply:{}", info.name),
            // SB-MLA-009 / SB-MLA-003. The blind record is COPIED from the model rather than
            // re-derived: it is a property of the fit, and this path has no fit. Copying is also
            // what keeps a curve made by applying a model saying the same thing as a curve made by
            // the run that fitted it. A model saved before either field existed carries neither,
            // and the absence is what such a model honestly has to offer.
            params_json: serde_json::to_string(&{
                let mut rec = serde_json::json!({
                    "model_id": info.model_id, "model_name": info.name,
                    "algorithm": info.algorithm, "trained_on": info.trained_on,
                });
                if let Some(h) = &info.train_hash {
                    rec["train_hash"] = serde_json::json!(h);
                }
                if let Some(b) = serde_json::from_str::<serde_json::Value>(&info.metrics_json)
                    .ok()
                    .and_then(|m| m.get("blind").cloned())
                {
                    rec["blind"] = b;
                }
                rec
            })
            .unwrap_or_default(),
            inputs_json: serde_json::to_string(&features).unwrap_or_default(),
        };
        let versioned = crate::equations::create_log_set(&conn, &aw.well_id, &spec)
            .and_then(|(set_id, _)| write_computed_curves_versioned(&conn, &aw.well_id, &aw.depth, &refs, &set_id));
        match versioned {
            Ok(()) => {
                if let Some(p) = progress {
                    p.finish_item(&aw.well_id, crate::jobs::ItemState::Ok, None);
                }
                wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: m, error: None });
            }
            Err(e) => {
                if let Some(p) = progress {
                    p.finish_item(&aw.well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                }
                wells.push(MlWellResult { well_id: aw.well_id.clone(), rows_predicted: 0, error: Some(e.to_string()) });
            }
        }
        start += m;
    }
    let mut notes = {
        let mut n = vec![format!(
            "applied the saved model '{}' ({} on {}), trained on {} well(s) - nothing was refitted",
            info.name,
            info.algorithm,
            info.target_curve.clone().unwrap_or_else(|| "-".into()),
            info.trained_on.len()
        )];
        // Which set the inputs were read from, decided above rather than by the caller.
        n.extend(notes);
        n
    };
    // SB-MLA-005 and SB-MLA-002, both checked HERE because this is the moment the artifact is
    // actually used. A warning at save time would name a runtime that had not yet diverged, and a
    // training set that is fine today can be superseded tomorrow by somebody re-running porosity.
    notes.extend(runtime_drift(info.runtime_json.as_deref(), &hdr.runtime));
    notes.extend(training_set_drift(&conn, info.training_json.as_deref()));
    MlResult {
        outputs: out_names,
        metrics: hdr.metrics,
        wells,
        notes,
        model_id: Some(info.model_id),
        model_name: Some(info.name),
        // Applying a saved model fits nothing, so there is no split to report. The split that
        // produced the model is part of the MODEL's record, not of this run.
        split: None,
        error: None,
    }
}

/// One python round-trip: returns (metrics, [(suffix, values-per-pooled-apply-sample)]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn exec_ml(
    python: &PathBuf,
    task: &str,
    algorithm: &str,
    params: &serde_json::Map<String, serde_json::Value>,
    d: usize,
    x_train: &[f32],
    y_train: Option<&[f32]>,
    x_apply: &[f32],
    n_apply: usize,
) -> Result<(serde_json::Value, Vec<(String, Vec<f32>)>), String> {
    exec_ml_full(python, task, algorithm, params, d, x_train, y_train, x_apply, n_apply, None, None, &[])
        .map(|r| (r.metrics, r.outs))
}

/// What one fitting subprocess returned. A named struct rather than the 4-tuple this used to be:
/// SB-MLA-005 adds a fifth member, and a five-element tuple of two JSON values and two strings is a
/// transposition waiting to happen at the one call site that has to get provenance right.
pub(crate) struct MlRun {
    pub metrics: serde_json::Value,
    pub outs: Vec<(String, Vec<f32>)>,
    pub model_blob: Vec<u8>,
    /// Kept separate from `runtime` because it is stored in its own column and was there first.
    pub sklearn: String,
    /// SB-MLA-005 — the interpreter and libraries that actually ran this fit.
    pub runtime: serde_json::Value,
}

/// As `exec_ml`, but when `save_features` is `Some(names)` the fitted scaler + estimator come
/// back as a joblib blob (with the scikit-learn version that wrote it, so a later load failure
/// can name the mismatch instead of being a mystery).
#[allow(clippy::too_many_arguments)]
pub(crate) fn exec_ml_full(
    python: &PathBuf,
    task: &str,
    algorithm: &str,
    params: &serde_json::Map<String, serde_json::Value>,
    d: usize,
    x_train: &[f32],
    y_train: Option<&[f32]>,
    x_apply: &[f32],
    n_apply: usize,
    save_features: Option<&[String]>,
    // `groups` is one well index per training row. Without it the runner has no way to hold out a
    // WELL, and every validation number it reports is a random-sample fold — see `cv_score`.
    // `blind_mask` is 1.0 for a row held out of the fit, one entry per training row; empty = no
    // split was asked for. A MASK rather than a list of wells, so that holding out whole wells and
    // drawing individual rows reach the runner as the same thing and cannot diverge there.
    groups: Option<&[f32]>,
    blind_mask: &[f32],
) -> Result<MlRun, String> {
    let n_train = if d == 0 { 0 } else { x_train.len() / d };
    let groups = groups.filter(|g| g.len() == n_train);
    // A mask of the wrong length is dropped rather than sent: the runner reads a fixed byte count
    // from the payload, so a short one would silently shift every column after it.
    let blind_mask = if blind_mask.len() == n_train { blind_mask } else { &[][..] };
    let header = serde_json::json!({
        "task": task, "algorithm": algorithm, "params": params,
        "d": d, "n_train": n_train, "has_target": y_train.is_some(), "n_apply": n_apply,
        "save_model": save_features.is_some(),
        "features": save_features.unwrap_or(&[]),
        "has_groups": groups.is_some(),
        "has_blind": !blind_mask.is_empty(),
    });

    let script = ScriptFile::new("fit", &ml_runner())
        .map_err(|e| format!("could not write the runner to a temporary file: {e}"))?;
    let mut cmd = Command::new(python);
    cmd.arg(script.path()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.write_all(bytemuck::cast_slice(x_train)).map_err(|e| e.to_string())?;
        if let Some(y) = y_train {
            stdin.write_all(bytemuck::cast_slice(y)).map_err(|e| e.to_string())?;
        }
        stdin.write_all(bytemuck::cast_slice(x_apply)).map_err(|e| e.to_string())?;
        if let Some(g) = groups {
            stdin.write_all(bytemuck::cast_slice(g)).map_err(|e| e.to_string())?;
        }
        // LAST, matching the runner's `take` order — X, y, A, groups, blind.
        if !blind_mask.is_empty() {
            stdin.write_all(bytemuck::cast_slice(blind_mask)).map_err(|e| e.to_string())?;
        }
    } // drop closes stdin

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("python ML run failed");
        return Err(last.trim().to_string());
    }
    let nl = output.stdout.iter().position(|&b| b == b'\n').ok_or("python returned no result header")?;
    #[derive(Deserialize)]
    struct OutHeader {
        suffixes: Vec<String>,
        metrics: serde_json::Value,
        #[serde(default)]
        model_len: usize,
        #[serde(default)]
        sklearn: String,
        #[serde(default)]
        runtime: serde_json::Value,
    }
    let hdr: OutHeader =
        serde_json::from_slice(&output.stdout[..nl]).map_err(|e| format!("bad ML result header: {e}"))?;
    let body = &output.stdout[nl + 1..];
    let expect = hdr.suffixes.len() * n_apply * 4;
    if body.len() != expect + hdr.model_len {
        return Err(format!(
            "python returned {} result bytes, expected {}",
            body.len(),
            expect + hdr.model_len
        ));
    }
    let mut outs = Vec::with_capacity(hdr.suffixes.len());
    for (i, s) in hdr.suffixes.iter().enumerate() {
        let mut vals = vec![0f32; n_apply];
        bytemuck::cast_slice_mut::<f32, u8>(&mut vals).copy_from_slice(&body[i * n_apply * 4..(i + 1) * n_apply * 4]);
        outs.push((s.clone(), vals));
    }
    Ok(MlRun {
        metrics: hdr.metrics,
        outs,
        model_blob: body[expect..].to_vec(),
        sklearn: hdr.sklearn,
        runtime: hdr.runtime,
    })
}

// ---------------------------------------------------------------------------------------------
// Model-comparison leaderboard (Wave B item 3): evaluate algorithm x feature-subset combos with
// BLIND-WELL cross-validation (whole wells held out via GroupKFold — the plain random 5-fold in
// ML_RUNNER leaks depth correlation because adjacent samples from one well land in both folds),
// plus permutation feature importance and a confusion matrix. One python round-trip evaluates
// every combo (single sklearn import); no curves are written — this ranks approaches to pick from.
// ---------------------------------------------------------------------------------------------

const ML_EVAL_RUNNER_BODY: &str = r#"
import sys, json
import numpy as np

def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
task = header["task"]; d = header["d"]; n = header["n_train"]
folds = int(header.get("folds", 5)); seed = int(header.get("seed", SEED_DEFAULT))
standardize = bool(header.get("standardize", True)); combos = header["combos"]
p = header.get("params") or {}
params_for = header.get("params_for")
total = n * d + n + n
raw = sys.stdin.buffer.read(4 * total)
if len(raw) != 4 * total:
    fail("truncated input stream")
X = np.frombuffer(raw, dtype=np.float32, count=n * d, offset=0).reshape(n, d).astype(np.float64)
y = np.frombuffer(raw, dtype=np.float32, count=n, offset=4 * n * d).astype(np.float64)
groups = np.frombuffer(raw, dtype=np.float32, count=n, offset=4 * (n * d + n)).astype(np.int64)

try:
    import sklearn  # noqa: F401
except ImportError:
    fail("scikit-learn is not installed for this Python - run: pip install scikit-learn")
from sklearn.preprocessing import StandardScaler
from sklearn.model_selection import GroupKFold, KFold
from sklearn.inspection import permutation_importance
from sklearn.metrics import r2_score, accuracy_score, f1_score, confusion_matrix

# Nothing is standardized here. A transform fitted before the split has seen the held-out well,
# so every score reported as blind is optimistic by construction (SB-MLA-028). The scalers are
# fitted per fold, on the fold's training rows only, further down.

# The estimator a leaderboard row scores is built by the SAME build_model the training run calls,
# from the SAME parameter map (SB-MLA-026). A row must describe the model the user will actually fit.
def make_model(algo):
    return build_model(task, algo, p if algo == params_for else {}, seed)[0]

ng = int(len(np.unique(groups)))
use_group = ng >= 2
nsplits = min(folds, ng) if use_group else min(folds, n)
nsplits = max(2, nsplits)
splitter = GroupKFold(n_splits=nsplits) if use_group else KFold(n_splits=nsplits, shuffle=True, random_state=seed)
SP = list(splitter.split(X, y, groups)) if use_group else list(splitter.split(X, y))

# One scaler per fold, fitted on that fold's TRAINING rows and on nothing else.
# StandardScaler is per-column, so a single fit over every column serves every feature subset:
# subselecting standardized columns == standardizing the subset. That commutation is why this is
# one fit per fold rather than one per (fold x subset). It is not a licence to fit outside the
# fold - the rows are what must not leak, and the rows are what this restricts.
SC = []
for tr, te in SP:
    if standardize:
        s = StandardScaler().fit(X[tr])
        SC.append((s.mean_, s.scale_))
    else:
        SC.append((None, None))

def fold_xy(k, fidx):
    tr, te = SP[k]
    a = X[tr][:, fidx]; b = X[te][:, fidx]
    mu, sd = SC[k]
    if mu is not None:
        a = (a - mu[fidx]) / sd[fidx]
        b = (b - mu[fidx]) / sd[fidx]
    return tr, te, a, b

clf = task == "classification"
yt = y.astype(int) if clf else y
labels = sorted(int(v) for v in np.unique(yt)) if clf else None
scoring = "accuracy" if clf else "r2"

# Rows carried back for the predicted-vs-actual crossplot. Every point is an OUT-OF-FOLD
# prediction - made by a model that did not see that row - so the picture answers the same
# question the score does. A crossplot of fitted values would look better and mean nothing.
#
# Sampled EVENLY over the pooled row order rather than taking the first N: the rows are ordered
# well by well, so the first N would be one or two wells and the picture would describe them
# instead of the field. This is also what keeps the payload a bounded diagnostic rather than the
# curve data rule 3 is about - the cap is the reason it is not that.
CROSSPLOT_MAX = 2000
_step = max(1, int(np.ceil(n / float(CROSSPLOT_MAX))))
XP = np.arange(0, n, _step)[:CROSSPLOT_MAX]

def _finite(a):
    # NaN is not valid JSON, and serde rejects what Python's json emits for it. A row no fold
    # could predict comes back as null, which is the honest value: not zero, not omitted.
    return [None if not np.isfinite(v) else round(float(v), 6) for v in a]

rows = []
for combo in combos:
    algo = combo["algorithm"]; fidx = combo["feat_idx"]
    oof = np.full(n, np.nan)
    fold_scores = []
    fold_imps = []
    err = None
    try:
        for k in range(len(SP)):
            m = make_model(algo)
            if m is None:
                err = "unknown algorithm '" + str(algo) + "'"; break
            tr, te, Xtr, Xte = fold_xy(k, fidx)
            m.fit(Xtr, yt[tr])
            pred = m.predict(Xte)
            oof[te] = pred
            fold_scores.append(accuracy_score(yt[te], pred.astype(int)) if clf else r2_score(y[te], pred))
            # Importance is measured on the SAME held-out rows the score is, by the SAME model.
            # Permuting a column of the training data would report how much the model leaned on a
            # feature to memorize, not how much that feature carried to a well it had never seen -
            # and the two numbers sit in one leaderboard row, so they must answer one question.
            try:
                pi = permutation_importance(m, Xte, yt[te], n_repeats=5,
                                            random_state=seed, scoring=scoring)
                fold_imps.append(np.asarray(pi.importances_mean, dtype=np.float64))
            except Exception:
                pass
    except Exception as e:
        err = str(e)
    if err is not None:
        rows.append({"algorithm": algo, "feat_idx": fidx, "error": err})
        continue
    metrics = {}
    if clf:
        oofi = oof.astype(int)
        score = float(accuracy_score(yt, oofi))
        metrics["macro_f1"] = float(f1_score(yt, oofi, average="macro", zero_division=0))
        conf = confusion_matrix(yt, oofi, labels=labels).tolist()
        labs = labels
    else:
        score = float(r2_score(y, oof))
        metrics["rmse"] = float(np.sqrt(np.mean((y - oof) ** 2)))
        conf = None; labs = None
    imp = [float("nan")] * len(fidx)
    imp_std = [float("nan")] * len(fidx)
    if fold_imps:
        M = np.vstack(fold_imps)
        imp = [float(v) for v in M.mean(axis=0)]
        # Spread ACROSS FOLDS, not across permutation repeats: it says whether a feature carried
        # to every held-out well or only to one. A mean of 0.30 +/- 0.28 is a different finding
        # from 0.30 +/- 0.02, and only the second one names a predictor.
        imp_std = [float(v) for v in M.std(axis=0)] if M.shape[0] > 1 else [0.0] * len(fidx)
    rows.append({"algorithm": algo, "feat_idx": fidx, "score": score,
                 "score_std": float(np.std(fold_scores)), "metrics": metrics,
                 "importances": imp, "importances_std": imp_std,
                 "n_imp_folds": int(len(fold_imps)), "confusion": conf, "labels": labs,
                 "blind_pred": _finite(oof[XP])})

# The actual values and the well each sampled row came from are the SAME for every model, so they
# ride once at the top rather than being repeated per row: fifteen copies of one column is fifteen
# chances for a reader to wonder which is authoritative.
out = {"rows": rows, "n_groups": ng, "n_splits": int(nsplits),
       "blind_actual": _finite(y[XP]), "blind_group": [int(v) for v in groups[XP]],
       "blind_sampled": int(len(XP)), "blind_total": int(n),
       "cv": "blind-well GroupKFold" if use_group else "random KFold"}
sys.stdout.buffer.write((json.dumps(out) + "\n").encode("utf-8"))
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct MlEvalRequest {
    /// Read the feature and target curves from THIS log set (see [`MlRequest::input_set`]). A
    /// leaderboard scored against a different version of the same curves is not comparable with
    /// the run it is meant to inform.
    #[serde(default)]
    pub input_set: Option<String>,
    /// "regression" | "classification" (supervised only — the leaderboard needs a target).
    pub task: String,
    pub feature_curves: Vec<String>,
    pub target_curve: String,
    pub train_well_ids: Vec<String>,
    /// Algorithm ids to compare (same ids as the training runner); empty → nothing to do.
    pub algorithms: Vec<String>,
    /// The SAME hyperparameter map the training run will be given. Without it the leaderboard
    /// ranked every candidate at its defaults, so a user who set `degree = 3` or `n_estimators`
    /// chose from a table describing models they were not about to fit (SB-MLA-026).
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
    /// Which algorithm `params` belongs to. The dialog holds one algorithm's settings at a time, so
    /// the map is applied to that row and every other row is scored at its defaults — which is what
    /// the run would fit for them, since the user has configured nothing else. Applied to all rows
    /// instead, an `C` set for SVR would silently re-rank logistic regression against a value
    /// nobody chose for it. `None` → every row at defaults.
    #[serde(default)]
    pub params_for: Option<String>,
    /// Feature subsets to try (each a subset of feature_curves by name); empty → the full set only.
    #[serde(default)]
    pub subsets: Vec<Vec<String>>,
    #[serde(default)]
    pub standardize: bool,
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub folds: Option<usize>,
    /// Optional flag curve: samples where the mask == 1.0 are excluded from the pooled CV set, so
    /// the leaderboard scores the same (unmasked) population the real ML run trains on.
    #[serde(default)]
    pub mask_curve: Option<String>,
    /// Score the candidates in a TRANSFORMED target space — the same [`MlRequest::target_transform`]
    /// the run will be given. A leaderboard is only worth reading if it ranks the model the run will
    /// fit, and a model fitted on log10(k) is a different model from one fitted on k: in linear
    /// space an R2 over four decades of permeability is dominated by the few highest values, so the
    /// winner there is routinely not the winner in log space.
    #[serde(default)]
    pub target_transform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MlEvalRow {
    pub algorithm: String,
    pub features: Vec<String>,
    pub score: Option<f64>,
    pub score_std: Option<f64>,
    pub metrics: serde_json::Value,
    /// Permutation importance, measured on each fold's HELD-OUT rows by the model that fold fitted,
    /// then averaged. It answers the same question `score` does — what carried to a well the model
    /// had not seen — so the two can be read in one row.
    pub importances: Vec<f64>,
    /// Spread of `importances` ACROSS FOLDS. A feature that matters in one well and nowhere else
    /// has a large one, and is not a predictor however high its mean.
    pub importances_std: Vec<f64>,
    /// How many folds contributed an importance measurement. Below `n_splits`, some fold could not
    /// be permuted (a single-class or single-sample held-out partition), and the mean is over fewer
    /// wells than the score is.
    pub n_imp_folds: usize,
    pub confusion: Option<Vec<Vec<i64>>>,
    pub labels: Option<Vec<i64>>,
    /// SB-MLA-027 — this model's OUT-OF-FOLD prediction for each sampled row, aligned position for
    /// position with [`MlEvalResult::blind_actual`].
    ///
    /// Out-of-fold, so every point was predicted by a model that had not seen that row: the picture
    /// answers the same question the score does. A crossplot of fitted values would look better and
    /// say nothing. `None` where no fold could predict that row — never zero, which is a value.
    pub blind_pred: Vec<Option<f64>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MlEvalResult {
    pub rows: Vec<MlEvalRow>,
    pub n_train: usize,
    pub n_groups: usize,
    pub cv: String,
    pub n_splits: usize,
    pub note: Option<String>,
    /// Echo of the request's `params_for`: which row was scored with the user's own settings. The
    /// requirement is that the leaderboard says so rather than presenting a mixed table cleanly —
    /// every other row is at library defaults, and a reader cannot tell by looking.
    pub params_for: Option<String>,
    /// The measured value at each sampled row — the x-axis of every model's crossplot.
    ///
    /// Carried ONCE rather than per row because it is the same column for every model: fifteen
    /// copies of one truth is fifteen chances for a reader to wonder which is authoritative, and
    /// one place to get an alignment wrong instead of fifteen.
    pub blind_actual: Vec<Option<f64>>,
    /// Which well each sampled row came from, BY NAME. A crossplot coloured by well is how an
    /// interpreter sees that a model is carried by two wells and fails on the third — the aggregate
    /// R² above it cannot show that, and it is the reading that decides whether the curve ships.
    pub blind_well: Vec<String>,
    /// How many rows the picture is drawn from, and how many there were. Stated because the sample
    /// is capped: a scatter that silently showed 2,000 of 60,000 points would read as all of them,
    /// and density is the first thing anybody judges from a crossplot.
    pub blind_sampled: usize,
    pub blind_total: usize,
    pub error: Option<String>,
}

fn eval_fail(msg: &str) -> MlEvalResult {
    MlEvalResult {
        rows: vec![],
        n_train: 0,
        n_groups: 0,
        cv: String::new(),
        n_splits: 0,
        note: None,
        params_for: None,
        blind_actual: vec![],
        blind_well: vec![],
        blind_sampled: 0,
        blind_total: 0,
        error: Some(msg.to_string()),
    }
}

/// Cap on total (algorithm x subset) combos evaluated in one leaderboard run — a full-subset
/// sweep over many curves would otherwise fit thousands of models. Excess is dropped with a note.
const MAX_COMBOS: usize = 80;

pub fn run_ml_eval(db: &Mutex<Connection>, req: &MlEvalRequest) -> MlEvalResult {
    if !matches!(req.task.as_str(), "regression" | "classification") {
        return eval_fail("the leaderboard is for supervised tasks (regression or classification)");
    }
    let features: Vec<String> =
        req.feature_curves.iter().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty()).collect();
    if features.is_empty() {
        return eval_fail("select at least one input curve");
    }
    let target = req.target_curve.trim().to_uppercase();
    if target.is_empty() {
        return eval_fail("choose a target curve to compare against");
    }
    let mask_curve =
        req.mask_curve.as_deref().map(|m| m.trim().to_uppercase()).filter(|m| !m.is_empty());
    let algos: Vec<String> =
        req.algorithms.iter().map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect();
    if algos.is_empty() {
        return eval_fail("no algorithms selected to compare");
    }
    if req.train_well_ids.len() < 2 {
        return eval_fail("blind-well comparison needs at least 2 training wells (whole wells are held out)");
    }
    let Some(python) = find_python() else {
        return eval_fail("no Python with numpy found - install Python 3.10+ with numpy + scikit-learn");
    };

    // Pool complete labelled samples across the training wells, tracking each sample's well index
    // so python can hold out whole wells (GroupKFold).
    let d = features.len();
    let mut x_train: Vec<f32> = Vec::new();
    let mut y_train: Vec<f32> = Vec::new();
    let mut groups: Vec<f32> = Vec::new();
    // Names for the crossplot's per-point well label, taken here because this is where the
    // connection is held. Index-aligned with `req.train_well_ids`, which is what the runner's
    // `groups` column indexes into.
    let train_names: Vec<String>;
    {
        let conn = db.lock().unwrap();
        train_names = req.train_well_ids.iter().map(|id| well_name(&conn, id)).collect();
        let mut fetch_names = features.clone();
        fetch_names.push(target.clone());
        if let Some(mk) = &mask_curve {
            fetch_names.push(mk.clone());
        }
        for (g, well_id) in req.train_well_ids.iter().enumerate() {
            let Ok((depth, cols)) = fetch_curve_frame_from_set(&conn, well_id, &fetch_names, req.input_set.as_deref(), None) else { continue };
            let Some(tv) = cols.get(&target) else { continue };
            let Some(fcols) = features.iter().map(|f| cols.get(f)).collect::<Option<Vec<_>>>() else { continue };
            let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
            for i in 0..depth.len() {
                // Exclude masked (== 1.0) samples from the CV pool, matching run_ml.
                if mcol.map_or(false, |m| m[i] == 1.0) {
                    continue;
                }
                if tv[i].is_finite() && fcols.iter().all(|c| c[i].is_finite()) {
                    for c in &fcols {
                        x_train.push(c[i]);
                    }
                    y_train.push(tv[i]);
                    groups.push(g as f32);
                }
            }
        }
    }
    // SB-MLA-035 / SB-MLA-026. The leaderboard must rank the model THE RUN WILL FIT, and a run
    // fitted on log10(target) is a different model from one fitted on the target. Rank them in the
    // untransformed space and the table is a ranking of models nobody is about to fit: on a
    // permeability spanning four decades, R2 in linear space is dominated by the handful of highest
    // values, so the winner there is routinely not the winner in log space.
    let transform = req
        .target_transform
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase)
        .filter(|t| !t.is_empty() && t != "none")
        .unwrap_or_default();
    let mut eval_note = None;
    if !transform.is_empty() {
        if req.task != "regression" {
            return eval_fail(&format!(
                "a {transform} target transform only applies to regression - a class label has no logarithm"
            ));
        }
        if transform != "log10" {
            return eval_fail(&format!("unknown target transform '{transform}' - use log10, or none"));
        }
        let dropped = apply_target_transform(&transform, d, &mut x_train, &mut y_train, &mut groups);
        if dropped > 0 {
            eval_note = Some(format!(
                "scored in {transform} space; {dropped} sample(s) of zero or less have no logarithm and were dropped"
            ));
        } else {
            eval_note = Some(format!("scored in {transform} space - these scores are not comparable with untransformed ones"));
        }
    }

    let n_train = y_train.len();
    if n_train < 20 {
        return eval_fail(&format!(
            "only {n_train} labelled samples across the training wells - need at least 20 for cross-validation"
        ));
    }
    // Build the (algorithm x subset) combos as feature-index lists into `features`.
    let idx_of = |name: &str| features.iter().position(|f| f == name);
    let mut subset_idx: Vec<Vec<usize>> = Vec::new();
    if req.subsets.is_empty() {
        subset_idx.push((0..d).collect());
    } else {
        let mut seen: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();
        for sub in &req.subsets {
            let mut idx: Vec<usize> =
                sub.iter().filter_map(|n| idx_of(&n.trim().to_uppercase())).collect();
            idx.sort_unstable();
            idx.dedup();
            if !idx.is_empty() && seen.insert(idx.clone()) {
                subset_idx.push(idx);
            }
        }
        if subset_idx.is_empty() {
            subset_idx.push((0..d).collect());
        }
    }
    let mut combos: Vec<(String, Vec<usize>)> = Vec::new();
    for a in &algos {
        for s in &subset_idx {
            combos.push((a.clone(), s.clone()));
        }
    }
    // Both notes matter and neither may silently replace the other: a truncated leaderboard reads
    // as "all of them", and a log-space score reads as a linear-space one.
    let mut notes: Vec<String> = eval_note.into_iter().collect();
    if combos.len() > MAX_COMBOS {
        notes.push(format!(
            "evaluated the first {MAX_COMBOS} of {} algorithm×subset combos (cap) — narrow the algorithms or subsets",
            combos.len()
        ));
        combos.truncate(MAX_COMBOS);
    }
    let mut note = (!notes.is_empty()).then(|| notes.join(" • "));

    let seed = req.seed.unwrap_or(crate::facies::SEED_DEFAULT as i64);
    let folds = req.folds.unwrap_or(5).clamp(2, 10);
    match exec_ml_eval(&python, &req.task, d, n_train, &x_train, &y_train, &groups, &combos, req.standardize, seed, folds, &req.params, req.params_for.as_deref()) {
        Err(e) => eval_fail(&e),
        Ok(py) => {
            // `py.n_groups` is the number of wells that ACTUALLY contributed samples (masking can
            // empty a whole well), i.e. what Python used for GroupKFold — report THAT, not the
            // requested count. Warn when masking collapses it below 2, since Python then silently
            // falls back to a leaky (depth-adjacent) random KFold.
            let requested = req.train_well_ids.len();
            if py.n_groups < requested {
                let deg = if py.n_groups < 2 {
                    format!(
                        "only {} training well contributed samples after masking (of {requested}) — blind-well CV needs \u{2265}2 wells, so scores fell back to random KFold and may be optimistic",
                        py.n_groups
                    )
                } else {
                    format!(
                        "{} of {requested} training well(s) contributed no samples after masking; blind-well CV ran over the remaining {}",
                        requested - py.n_groups,
                        py.n_groups
                    )
                };
                note = Some(match note {
                    Some(existing) => format!("{existing}. {deg}"),
                    None => deg,
                });
            }
            let mut rows: Vec<MlEvalRow> = py
                .rows
                .into_iter()
                .map(|r| MlEvalRow {
                    algorithm: r.algorithm,
                    features: r.feat_idx.iter().filter_map(|&i| features.get(i).cloned()).collect(),
                    score: r.score,
                    score_std: r.score_std,
                    metrics: r.metrics,
                    importances: r.importances,
                    importances_std: r.importances_std,
                    n_imp_folds: r.n_imp_folds,
                    confusion: r.confusion,
                    labels: r.labels,
                    blind_pred: r.blind_pred,
                    error: r.error,
                })
                .collect();
            // Best first: successful rows by score desc, errored rows last.
            rows.sort_by(|a, b| match (a.score, b.score) {
                (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
            // A group index the runner reports but the caller's list does not cover would be a
            // bug rather than a well, so it is named as unknown rather than silently indexed.
            let blind_well: Vec<String> = py
                .blind_group
                .iter()
                .map(|&g| train_names.get(g).cloned().unwrap_or_else(|| "?".to_string()))
                .collect();
            MlEvalResult {
                rows,
                n_train,
                n_groups: py.n_groups,
                cv: py.cv,
                n_splits: py.n_splits,
                note,
                params_for: req.params_for.clone(),
                blind_actual: py.blind_actual,
                blind_well,
                blind_sampled: py.blind_sampled,
                blind_total: py.blind_total,
                error: None,
            }
        }
    }
}

#[derive(Deserialize)]
struct PyEvalRow {
    algorithm: String,
    feat_idx: Vec<usize>,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    score_std: Option<f64>,
    #[serde(default)]
    metrics: serde_json::Value,
    #[serde(default)]
    importances: Vec<f64>,
    #[serde(default)]
    importances_std: Vec<f64>,
    #[serde(default)]
    n_imp_folds: usize,
    #[serde(default)]
    confusion: Option<Vec<Vec<i64>>>,
    #[serde(default)]
    labels: Option<Vec<i64>>,
    #[serde(default)]
    blind_pred: Vec<Option<f64>>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct PyEvalOut {
    rows: Vec<PyEvalRow>,
    n_groups: usize,
    n_splits: usize,
    cv: String,
    #[serde(default)]
    blind_actual: Vec<Option<f64>>,
    #[serde(default)]
    blind_group: Vec<usize>,
    #[serde(default)]
    blind_sampled: usize,
    #[serde(default)]
    blind_total: usize,
}

/// One python round-trip evaluating every combo. Returns parsed rows (feature INDICES; the caller
/// maps them back to names).
#[allow(clippy::too_many_arguments)]
pub(crate) fn exec_ml_eval(
    python: &PathBuf,
    task: &str,
    d: usize,
    n_train: usize,
    x_train: &[f32],
    y_train: &[f32],
    groups: &[f32],
    combos: &[(String, Vec<usize>)],
    standardize: bool,
    seed: i64,
    folds: usize,
    params: &serde_json::Map<String, serde_json::Value>,
    params_for: Option<&str>,
) -> Result<PyEvalOut, String> {
    let combos_json: Vec<serde_json::Value> = combos
        .iter()
        .map(|(a, idx)| serde_json::json!({ "algorithm": a, "feat_idx": idx }))
        .collect();
    let header = serde_json::json!({
        "task": task, "d": d, "n_train": n_train,
        "standardize": standardize, "seed": seed, "folds": folds, "combos": combos_json,
        "params": params, "params_for": params_for,
    });

    let script = ScriptFile::new("eval", &ml_eval_runner())
        .map_err(|e| format!("could not write the runner to a temporary file: {e}"))?;
    let mut cmd = Command::new(python);
    cmd.arg(script.path()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.write_all(bytemuck::cast_slice(x_train)).map_err(|e| e.to_string())?;
        stdin.write_all(bytemuck::cast_slice(y_train)).map_err(|e| e.to_string())?;
        stdin.write_all(bytemuck::cast_slice(groups)).map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("python ML comparison failed");
        return Err(last.trim().to_string());
    }
    let nl = output.stdout.iter().position(|&b| b == b'\n').unwrap_or(output.stdout.len());
    serde_json::from_slice(&output.stdout[..nl]).map_err(|e| format!("bad ML comparison result: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn python_with_sklearn() -> Option<PathBuf> {
        let p = find_python()?;
        let mut cmd = Command::new(&p);
        cmd.args(["-c", "import sklearn"]).stdout(Stdio::null()).stderr(Stdio::null());
        hide_console(&mut cmd);
        cmd.status().ok()?.success().then_some(p)
    }

    fn params(pairs: &[(&str, serde_json::Value)]) -> serde_json::Map<String, serde_json::Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    /// **SB-MLA-023, the half that fails the build.** The product has two k-means engines for
    /// platform reasons, and they were configured differently — 8 restarts and 100 iterations in
    /// the native one against scikit-learn's 10 and 300, with no tolerance at all on the native
    /// side. Restart count and iteration cap are precisely the two knobs that select WHICH local
    /// optimum k-means lands in, so the same curves, K and seed gave two different facies schemes
    /// depending on which door the user came in.
    ///
    /// This is the structural check, and it is the one that matters: the numbers must reach Python
    /// FROM the Rust definition, so there is no second copy of them to go stale. A test that merely
    /// asserted both said 10 would pass a pair of literals sitting in two files, which is the state
    /// this requirement exists to end.
    ///
    /// Needs no Python, so a divergence fails the gate — which is what the requirement asks for and
    /// what an end-to-end comparison alone could not deliver, since that one has to skip where
    /// scikit-learn is absent.
    #[test]
    fn the_two_kmeans_engines_are_configured_from_one_definition() {
        let emitted = ml_shared_constants_py();
        for (name, want) in [
            ("KMEANS_N_INIT", crate::facies::KMEANS_RESTARTS.to_string()),
            ("KMEANS_MAX_ITER", crate::facies::KMEANS_MAX_ITERS.to_string()),
            ("SEED_DEFAULT", (crate::facies::SEED_DEFAULT as i64).to_string()),
        ] {
            assert!(
                emitted.contains(&format!("{name} = {want}")),
                "the runner preamble must carry the native value of {name} ({want}):\n{emitted}",
            );
        }
        // The tolerance is emitted in scientific notation, so match it by parsing rather than by
        // spelling — `1e-4` and `0.0001` are the same number and either would be correct.
        let tol: f64 = emitted
            .lines()
            .find_map(|l| l.strip_prefix("KMEANS_TOL = "))
            .and_then(|v| v.trim().parse().ok())
            .expect("the preamble declares a tolerance");
        assert!((tol - crate::facies::KMEANS_TOL).abs() < 1e-12, "tolerance emitted as {tol}");

        // Both runners get the preamble, and both use the NAMES. A literal here would compile, run
        // and look right while silently forking the definition again.
        for (which, src) in [("train", ml_runner()), ("leaderboard", ml_eval_runner())] {
            assert!(src.contains("KMEANS_N_INIT = "), "{which} runner is missing the preamble");
            assert!(
                !src.contains("n_init=10") && !src.contains("max_iter=300"),
                "{which} runner still hardcodes a k-means constant beside the shared one",
            );
        }
        assert!(
            ML_RUNNER_BODY.contains("n_init=KMEANS_N_INIT")
                && ML_RUNNER_BODY.contains("max_iter=KMEANS_MAX_ITER")
                && ML_RUNNER_BODY.contains("tol=KMEANS_TOL"),
            "the KMeans call must read all three from the shared definition",
        );
        // SB-MLA-024 — one seed default, and neither runner may write its own.
        assert!(
            !ML_RUNNER_BODY.contains("\"seed\", 42") && !ML_EVAL_RUNNER_BODY.contains("\"seed\", 42"),
            "a seed default restated in Python is a second definition of the same concept",
        );
    }

    /// The end-to-end half of SB-MLA-023: with one definition behind both, the two engines must
    /// actually agree on data whose answer is not in doubt.
    ///
    /// Deliberately a THREE-blob fixture with wide separation. k-means is only well-posed where the
    /// clustering is unambiguous, and a fixture with overlapping groups would be testing whose
    /// pseudo-random draw got luckier — the native engine seeds from SplitMix64 and scikit-learn
    /// from NumPy's Mersenne Twister, so identical labelling in general is not on offer and
    /// claiming it would be a false pin. What IS on offer, and what a user notices, is that both
    /// find the same obvious answer with the same number of restarts and the same stopping rule.
    ///
    /// Skips where scikit-learn is absent, so the gate never depends on it — the structural test
    /// above is the one that fails the build.
    #[test]
    fn the_two_kmeans_engines_label_the_same_data_the_same_way() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // Three separated blobs in (GR-like, RHOB-like), interleaved so neither engine can be right
        // by accident of row order. Jitter is deterministic — a fixture that moved between runs
        // could not tell a divergence from a coincidence.
        let centres = [(25.0f32, 2.62f32), (75.0, 2.45), (140.0, 2.30)];
        let (mut gr, mut rhob, mut x_apply) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..60 {
            for (cx, cy) in centres {
                let j = (i % 7) as f32;
                let (a, b) = (cx + j * 0.3, cy + j * 0.002);
                gr.push(a);
                rhob.push(b);
                x_apply.extend_from_slice(&[a, b]);
            }
        }
        let n = gr.len();

        let mut logs = std::collections::HashMap::new();
        logs.insert("CURVE1".to_string(), gr);
        logs.insert("CURVE2".to_string(), rhob);
        let mut mod_params = std::collections::HashMap::new();
        mod_params.insert("K".to_string(), vec![3.0; n]);
        mod_params.insert("SEED".to_string(), vec![crate::facies::SEED_DEFAULT; n]);
        let ctx = crate::modules::ModuleContext {
            n,
            logs,
            params: mod_params,
            opts: std::collections::HashMap::new(),
            depth_unit: Default::default(),
        };
        let native = crate::facies::electrofacies(&ctx).expect("native clustering ran");
        let native = &native["FACIES"];

        let (_, outs) = exec_ml(
            &py,
            "clustering",
            "kmeans",
            &params(&[
                ("k", serde_json::json!(3)),
                ("seed", serde_json::json!(crate::facies::SEED_DEFAULT as i64)),
            ]),
            2,
            &[],
            None,
            &x_apply,
            n,
        )
        .expect("scikit-learn clustering ran");
        let sk = &outs[0].1;

        // Both order their cluster ids by ascending mean of the FIRST feature, so the labels are
        // directly comparable — no permutation matching, which would let a genuine disagreement
        // about WHICH samples group together pass as a relabelling.
        assert_eq!(native.len(), sk.len());
        let disagree = (0..n).filter(|&i| native[i] != sk[i]).count();
        assert_eq!(
            disagree, 0,
            "the two k-means engines disagreed on {disagree} of {n} samples of a three-blob \
             fixture — one definition means one answer here",
        );
    }

    #[test]
    fn regression_linear_recovers_line() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // y = 2x + 1 exactly; linear regression must recover it and predict new points.
        let x_train: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let y_train: Vec<f32> = x_train.iter().map(|&x| 2.0 * x + 1.0).collect();
        let x_apply: Vec<f32> = vec![10.0, 50.5, 99.0];
        let (metrics, outs) = exec_ml(
            &py, "regression", "linear", &params(&[("standardize", serde_json::json!(false))]),
            1, &x_train, Some(&y_train), &x_apply, 3,
        )
        .expect("regression run failed");
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].0, "");
        let pred = &outs[0].1;
        assert!((pred[0] - 21.0).abs() < 1e-3, "got {}", pred[0]);
        assert!((pred[1] - 102.0).abs() < 1e-3, "got {}", pred[1]);
        assert!((pred[2] - 199.0).abs() < 1e-3, "got {}", pred[2]);
        let r2 = metrics["r2_train"].as_f64().unwrap();
        assert!(r2 > 0.999, "r2_train = {r2}");

        // SB-MLA-001, from both sides on a real run. `standardize` was supplied and must be
        // recorded as supplied; `seed` and `degree` were never sent and must be recorded WITH the
        // fact that they were defaulted — a re-run cannot be built from a record that omits the
        // values nobody typed, and `seed` is the one with the largest effect on the answer.
        let eff = metrics["effective_params"].as_object().expect("effective params are reported");
        assert_eq!(eff["standardize"]["defaulted"], serde_json::json!(false));
        assert_eq!(eff["standardize"]["value"], serde_json::json!(false));
        assert_eq!(eff["seed"]["defaulted"], serde_json::json!(true), "seed was never supplied");
        assert_eq!(eff["seed"]["value"], serde_json::json!(42));
        assert!(eff["seed"]["source"].is_string(), "a defaulted value names where the default came from");
        assert_eq!(eff["degree"]["defaulted"], serde_json::json!(true), "the algorithm's own default counts too");
    }

    fn mk_req(task: &str, features: &[&str], target: Option<&str>, train: &[String], apply: &[String]) -> MlRequest {
        MlRequest {
            input_set: None,
            output_set: None,
            blind_fraction: None,
            split_seed: None,
            split_mode: None,
            task: task.into(),
            algorithm: if task == "clustering" { "kmeans".into() } else { "linear".into() },
            params: serde_json::Map::new(),
            feature_curves: features.iter().map(|s| s.to_string()).collect(),
            target_curve: target.map(|s| s.to_string()),
            mask_curve: None,
            train_well_ids: train.to_vec(),
            apply_well_ids: apply.to_vec(),
            output_curve: "PRED".into(),
            save_model_as: None,
            model_note: None,
            target_transform: None,
            coverage_segments: false,
            output_step: None,
            interval: DepthWindow::default(),
        }
    }

    /// run_ml's own DB-integration guards (the pure exec_ml tests never reach them): the early
    /// request-shape refusals need no python; the <10-training-samples and n_apply==0 refusals
    /// fire after find_python but BEFORE sklearn, so they need only numpy.
    #[test]
    fn run_ml_guards_reject_bad_requests() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 5usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();

        // Train well: only 5 complete GR + RHOB(target) samples.
        let train = Uuid::new_v4();
        db::insert_well(&conn, train, "TR-1", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, train, depths.clone(),
            (0..n).map(|i| 20.0 + i as f32 * 5.0).collect(), // GR
            vec![f32::NAN; n], vec![f32::NAN; n],
            (0..n).map(|i| 2.2 + i as f32 * 0.05).collect(), // RHOB (target)
            vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        // Apply well with a real GR.
        let apply = Uuid::new_v4();
        db::insert_well(&conn, apply, "AP-1", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, apply, depths.clone(),
            (0..n).map(|i| 30.0 + i as f32).collect(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        // Apply well whose GR is entirely missing → no complete apply samples.
        let empty = Uuid::new_v4();
        db::insert_well(&conn, empty, "AP-EMPTY", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, empty, depths.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();

        let db = Mutex::new(conn);
        let (tr, ap, em) = (train.to_string(), apply.to_string(), empty.to_string());

        // Early request-shape guards (no python needed).
        assert!(run_ml(&db, &mk_req("regression", &[], Some("RHOB"), &[tr.clone()], &[ap.clone()]), None).error.is_some(), "empty features");
        assert!(run_ml(&db, &mk_req("regression", &["GR"], Some("RHOB"), &[tr.clone()], &[]), None).error.is_some(), "empty apply");
        assert!(run_ml(&db, &mk_req("regression", &["GR"], None, &[tr.clone()], &[ap.clone()]), None).error.is_some(), "supervised without target");
        assert!(run_ml(&db, &mk_req("regression", &["GR"], Some("RHOB"), &[], &[ap.clone()]), None).error.is_some(), "supervised without train wells");

        if find_python().is_none() {
            eprintln!("skipping python-dependent run_ml guards: no python+numpy");
            return;
        }
        // Only 5 labelled samples → the <10 refusal (fires before sklearn).
        let r = run_ml(&db, &mk_req("regression", &["GR"], Some("RHOB"), &[tr.clone()], &[ap.clone()]), None);
        assert!(
            r.error.as_deref().unwrap_or("").contains("labelled training samples"),
            "expected <10-samples refusal, got {:?}",
            r.error,
        );
        // Unsupervised clustering on a well with no complete samples → n_apply==0 refusal.
        let r = run_ml(&db, &mk_req("clustering", &["GR"], None, &[], &[em.clone()]), None);
        assert!(
            r.error.as_deref().unwrap_or("").contains("no complete samples"),
            "expected n_apply==0 refusal, got {:?}",
            r.error,
        );
    }

    /// A MASK curve (== 1.0 excludes) reaches the APPLY pool before python: an all-1.0 mask starves
    /// the pool → the n_apply==0 refusal, while the same data with no mask does not. Needs numpy.
    #[test]
    fn run_ml_mask_excludes_apply_samples() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 6usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "MASK-AP", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, well, depths.clone(),
            (0..n).map(|i| 30.0 + i as f32).collect(), // GR present on every row
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        crate::equations::write_computed_curve(&conn, &well.to_string(), &depths, "MASK", &vec![1.0f32; n]).unwrap();

        let ids = well.to_string();
        let db = Mutex::new(conn);
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }
        let mut masked = mk_req("clustering", &["GR"], None, &[], &[ids.clone()]);
        masked.mask_curve = Some("MASK".into());
        let r = run_ml(&db, &masked, None);
        let msg = r.error.as_deref().unwrap_or("").to_string();
        assert!(
            msg.contains("no complete samples") && msg.contains("mask"),
            "all-1.0 mask must starve the apply pool AND name the mask as the cause, got {:?}",
            r.error,
        );
        // Control: identical data, no mask → the pool is NOT empty (never hits that refusal).
        let ctrl = run_ml(&db, &mk_req("clustering", &["GR"], None, &[], &[ids]), None);
        assert!(
            !ctrl.error.as_deref().unwrap_or("").contains("no complete samples"),
            "without a mask the well has complete samples, got {:?}",
            ctrl.error,
        );
    }

    /// **SB-MLA-006 — a curve made by a fitted model names that model.**
    ///
    /// The asymmetry was backwards. The APPLY path — the cheap case, where the model already exists
    /// and is named — recorded it on every curve; the FIT path, whose configuration is by far the
    /// harder one to reconstruct, recorded only the algorithm. And it could not have done otherwise:
    /// the model was persisted AFTER the well loop, so its id did not exist yet when each log set
    /// was created.
    ///
    /// The fix moves the save ahead of the loop, which sounds like it trades away the "a storage
    /// problem must not cost the predictions" rule. It does not: every failure in that block is a
    /// note rather than a return, and the curves are written either way — the second half of this
    /// test is what holds that line, because a run that saved nothing must still write its curves,
    /// citing no model at all rather than one that does not exist.
    #[test]
    fn a_curve_from_a_fitting_run_names_the_model_and_a_run_that_kept_none_names_none() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let Some(_) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 40usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-PROV", None, None, Some(0.0)).unwrap();
        let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32 * 2.0).collect();
        // A target that is a clean function of GR, so the fit is never the thing under test here.
        let rhob: Vec<f32> = gr.iter().map(|g| 2.65 - g * 0.001).collect();
        // (gr, res_deep, nphi, rhob, dt, sp) — RHOB is the fourth curve column.
        db::insert_standard_curves(
            &conn, well, depths.clone(), gr,
            vec![f32::NAN; n], vec![f32::NAN; n], rhob, vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();

        let ids = well.to_string();
        let db = Mutex::new(conn);
        let cited = |db: &Mutex<Connection>, set: &str| -> Option<String> {
            let c = db.lock().unwrap();
            let js: String = c
                .query_row(
                    "SELECT params_json FROM log_sets WHERE well_id = ?1 AND set_name = ?2 \
                     ORDER BY version DESC LIMIT 1",
                    duckdb::params![&ids, set],
                    |r| r.get(0),
                )
                .expect("the run created a log set");
            serde_json::from_str::<serde_json::Value>(&js)
                .ok()?
                .get("model_id")?
                .as_str()
                .map(str::to_string)
        };

        let mut saved = mk_req("regression", &["GR"], Some("RHOB"), &[ids.clone()], &[ids.clone()]);
        saved.save_model_as = Some("PROV_TEST".into());
        saved.output_set = Some("ML_SAVED".into());
        let r = run_ml(&db, &saved, None);
        assert!(r.error.is_none(), "the fitting run failed: {:?}", r.error);
        let model_id = r.model_id.clone().expect("the model was kept");
        assert_eq!(
            cited(&db, "ML_SAVED").as_deref(),
            Some(model_id.as_str()),
            "the curve must cite the model that produced it, not merely the algorithm",
        );
        // And the citation must resolve — a provenance string pointing at nothing asserts an audit
        // trail it cannot honour.
        {
            let c = db.lock().unwrap();
            assert!(db::get_ml_model(&c, &model_id).is_ok(), "the cited model must exist");
        }

        // The other side. No model asked for, so the curves are still written and cite NO model —
        // absent, not an empty string, because a null id invites no lookup that has to fail.
        let mut unsaved = mk_req("regression", &["GR"], Some("RHOB"), &[ids.clone()], &[ids.clone()]);
        unsaved.output_set = Some("ML_UNSAVED".into());
        let r2 = run_ml(&db, &unsaved, None);
        assert!(r2.error.is_none(), "a run that keeps no model must still write its curves: {:?}", r2.error);
        assert!(r2.model_id.is_none(), "nothing was asked to be saved");
        assert_eq!(cited(&db, "ML_UNSAVED"), None, "a curve must never cite a model that was not kept");
    }

    /// **SB-MLA-035, the half that needs no Python.** Three columns describe the same samples —
    /// features, target, well index — and the transform drops rows from the target's column. Drop
    /// them from `y` alone and every feature row after the first drop belongs to a different depth,
    /// and the model fits, scores and reports confidently on scrambled pairs. Nothing downstream
    /// can catch that; the row count still agrees with itself.
    ///
    /// Pinned from both sides. A zero permeability is a real reading and has no logarithm, so it is
    /// dropped and COUNTED rather than floored to some small number — a floor is an invented
    /// parameter that anchors the low end of the fit. And with no transform asked for, nothing is
    /// dropped and nothing is rewritten: the transform must be something the user chose, never
    /// something the pipeline decided a permeability-shaped curve deserved.
    #[test]
    fn a_log_transform_drops_a_row_from_every_column_or_from_none() {
        let d = 2;
        // Row i carries feature pair (i, i+100) and well index i, so a mis-drop is visible as a
        // pairing that no longer matches rather than merely as a wrong length.
        let mk = || {
            let y = vec![10.0f32, 0.0, 100.0, -3.0, 1000.0];
            let x: Vec<f32> = (0..y.len()).flat_map(|i| [i as f32, i as f32 + 100.0]).collect();
            let g: Vec<f32> = (0..y.len()).map(|i| i as f32).collect();
            (x, y, g)
        };

        let (mut x, mut y, mut g) = mk();
        let dropped = apply_target_transform("log10", d, &mut x, &mut y, &mut g);
        assert_eq!(dropped, 2, "a zero and a negative target both have no logarithm");
        assert_eq!(y.len(), 3);
        assert_eq!(g, vec![0.0, 2.0, 4.0], "the surviving rows must keep their own well index");
        assert_eq!(x, vec![0.0, 100.0, 2.0, 102.0, 4.0, 104.0], "features must ride with their target");
        for (got, want) in y.iter().zip([1.0f32, 2.0, 3.0]) {
            assert!((got - want).abs() < 1e-6, "target must be in log10 space, got {got}");
        }

        // The other side: no transform, no drop, no rewrite.
        let (mut x2, mut y2, mut g2) = mk();
        let (x0, y0, g0) = mk();
        assert_eq!(apply_target_transform("none", d, &mut x2, &mut y2, &mut g2), 0);
        assert_eq!((x2, y2, g2), (x0, y0, g0), "an unasked-for transform must change nothing");

        // The unit names the space, and an unknown target unit must not erase what IS known.
        assert_eq!(transformed_unit("log10", Some("mD")), "log10(mD)");
        assert_eq!(transformed_unit("log10", Some("  ")), "log10");
        assert_eq!(transformed_unit("log10", None), "log10");
    }

    /// **Sampling is not resolution, and a prediction says which one it fell short on.**
    ///
    /// Jauhar, 2026-08-07: *"rhob and dres with same sampling at 0.5 f can have different resolution
    /// or curve/wave frequency"*. He is right, and it is the harder half of the sampling question: a
    /// density pad reads a few inches and a deep induction reads several feet, so two curves on one
    /// 0.5 ft frame carry completely different vertical resolution. Blocking to a thickness cannot
    /// see that at all - both curves have the same sample spacing.
    ///
    /// A prediction is always smoother than its target, because the model can only carry through the
    /// detail its INPUTS contain. So this is measured and reported, never corrected: restoring the
    /// missing detail means synthesizing it, and a curve that looks better resolved without being
    /// better known is the more expensive failure.
    ///
    /// Pinned from both sides. A smooth prediction of a rough target must be REPORTED, and a
    /// prediction that already matches its target's roughness must be SILENT - a line printed on
    /// every run is a line the eye learns to skip, and this one has to be read when it appears.
    #[test]
    fn a_smooth_prediction_of_a_rough_log_says_so_and_a_faithful_one_stays_quiet() {
        // A rough target: fine detail on a trend, as a density pad reads it.
        let rough: Vec<f32> = (0..200)
            .map(|i| 2.4 + 0.02 * ((i % 2) as f32) + 0.0005 * i as f32)
            .collect();
        // A smooth prediction of the same thing: the trend, none of the detail. This is what a model
        // fed only low-resolution curves returns, and it plots as a perfectly plausible density.
        let smooth: Vec<f32> = (0..200).map(|i| 2.41 + 0.0005 * i as f32).collect();

        let note = resolution_note(&rough, &smooth, "RHOB").expect("a smoother prediction must be reported");
        assert!(note.contains("vertical resolution"), "{note}");
        assert!(note.contains("RHOB"), "the note must name the target it fell short of: {note}");
        assert!(note.contains("SMOOTHER"), "{note}");

        // The other side: a prediction that wiggles like its target says nothing.
        assert!(
            resolution_note(&rough, &rough, "RHOB").is_none(),
            "a prediction matching its target's roughness must not print a warning"
        );

        // Roughness is RELATIVE to the curve's own spread, or a permeability in millidarcies would
        // always look rougher than a porosity in fractions and the two could never be compared.
        let big: Vec<f32> = rough.iter().map(|v| v * 1000.0).collect();
        let (a, b) = (roughness(&rough).unwrap(), roughness(&big).unwrap());
        assert!((a - b).abs() < 1e-4, "scaling a curve must not change how rough it is: {a} vs {b}");

        // A gap contributes no step. Bridged, the jump across it would read as fine detail - the
        // exact opposite of the truth, since nothing was measured there at all.
        let mut holed = rough.clone();
        for i in 90..110 {
            holed[i] = f32::NAN;
        }
        let r = roughness(&holed).expect("a curve with a gap still has a roughness");
        assert!(r < a * 1.5, "a gap must not be counted as a step: {r} against {a}");

        // Too little to say anything is None, not zero: zero would read as "perfectly smooth".
        assert!(roughness(&[1.0, 2.0, 3.0]).is_none());
        assert!(roughness(&[2.5f32; 50]).is_none(), "a flat curve has no spread to be rough relative to");
    }

    /// **A tops-bounded run is bounded on BOTH sides of the work: what it learns from, and what it
    /// writes.**
    ///
    /// Jauhar, 2026-08-07: *"it should be tops bounded as well by user"*. A model fitted over a
    /// whole well learns one relation for every formation it passed through, and a deltaic sand and
    /// the carbonate below it do not share a porosity-permeability transform.
    ///
    /// Three things here are the ones that go wrong quietly:
    ///
    /// An open side must stay open. The last top in a well runs to TD, expressed as no base rather
    /// than a guessed one — read as "no window", the run silently widens back to the whole well and
    /// the interpreter gets a field-wide model under a zone's name.
    ///
    /// The base is EXCLUSIVE while the top is inclusive, so two abutting zones cannot both claim the
    /// sample sitting exactly on their shared marker. Swept zone by zone, an inclusive base would
    /// count that sample twice — once in each model, in both scores.
    ///
    /// And a NaN depth is in no window at all. `contains` on a non-finite depth has to be false, or
    /// a comparison that is false in both directions lets it fall through whichever branch was
    /// written second.
    #[test]
    fn a_tops_bounded_run_is_confined_on_both_sides_and_an_open_side_stays_open() {
        let both = DepthWindow { top: Some(2000.0), base: Some(2100.0) };
        assert!(!both.is_open());
        assert!(both.contains(2000.0), "the top marker's own sample is INSIDE its zone");
        assert!(both.contains(2099.9));
        assert!(!both.contains(2100.0), "the base marker belongs to the zone BELOW, or it is counted twice");
        assert!(!both.contains(1999.9));

        // An open side is open, not zero and not the top of the log.
        let to_td = DepthWindow { top: Some(2000.0), base: None };
        assert!(to_td.contains(9999.0), "no base means run to TD");
        assert!(!to_td.contains(1999.0));
        let from_top = DepthWindow { top: None, base: Some(2100.0) };
        assert!(from_top.contains(0.0), "no top means start at the top of the log");
        assert!(!from_top.contains(2100.0));

        // The default constrains nothing, which is what keeps every pre-existing payload whole.
        let open = DepthWindow::default();
        assert!(open.is_open());
        assert!(open.contains(-1.0) && open.contains(1e9));

        // A depth that is not a number is in no window, including the open one.
        assert!(!both.contains(f32::NAN));
        assert!(!open.contains(f32::NAN));

        // The description is what the run's note quotes, so an open side must not read as a number.
        assert_eq!(both.describe(), "2000 to 2100");
        assert_eq!(to_td.describe(), "2000 to TD");
        assert_eq!(from_top.describe(), "the top of the log to 2100");
        assert_eq!(open.describe(), "the whole well");
    }

    /// **A depth is predicted by the largest model whose curves it carries, and by nothing else.**
    ///
    /// Jauhar's cross-check (2026-08-07): four input curves, one of them logged over half the well.
    /// The ordinary path uses a depth only where every input has a value, so that one short curve
    /// deletes the other half of all four. The answer is one model per observed pattern — and the
    /// two ways of getting it wrong are opposite, so this pins both.
    ///
    /// Predict a four-curve depth with the three-curve model and a log that is sitting right there
    /// goes unused. Predict a three-curve depth with the four-curve model and the model is being fed
    /// a curve that does not exist at that depth. Neither shows up as an error — both produce a full,
    /// plausible curve — so the assignment is pinned from both sides here rather than trusted.
    ///
    /// The cap is pinned the same way. It rations by rock covered, not by curve count: keeping the
    /// biggest patterns would keep the rarest ones, and a depth whose own pattern lost the cap must
    /// fall back to the largest kept SUBSET rather than go blank. Only a depth that can feed no kept
    /// model at all is left unclaimed — never quietly handed to one it cannot feed.
    #[test]
    fn a_depth_is_predicted_by_the_largest_model_whose_curves_it_carries() {
        // Four features; bit k = feature k is present. GR=0, RHOB=1, NPHI=2, RT=3.
        const ALL: u32 = 0b1111;
        const NO_RT: u32 = 0b0111;
        const GR_ONLY: u32 = 0b0001;
        let a: Vec<u32> = vec![ALL, ALL, ALL, ALL, NO_RT, NO_RT, NO_RT];
        let b: Vec<u32> = vec![ALL, ALL, GR_ONLY, 0];
        let wells: Vec<&[u32]> = vec![&a, &b];

        let (cands, asg) = coverage_plan(&wells, 6);
        assert_eq!(
            cands,
            vec![ALL, NO_RT, GR_ONLY],
            "candidates must be the patterns that OCCUR (3 of them), never all 15 non-empty subsets, \
             and ordered largest-first for assignment"
        );
        assert_eq!(
            asg[0],
            vec![Some(0), Some(0), Some(0), Some(0), Some(1), Some(1), Some(1)],
            "the four-curve depths must go to the four-curve model even though the three-curve one \
             also fits them, and the three-curve depths must NOT go to the four-curve model"
        );
        assert_eq!(
            asg[1],
            vec![Some(0), Some(0), Some(2), None],
            "a depth carrying one curve is predicted by the one-curve model; a depth carrying none \
             is left unclaimed rather than handed to a model it cannot feed"
        );

        // The cap keeps what covers the most rock, and a cut pattern falls back to a kept subset.
        // NO_RT covers 3 rows and ALL covers 2, so ranking by curve count would keep the wrong one.
        let c: Vec<u32> = vec![NO_RT, NO_RT, NO_RT, ALL, ALL, GR_ONLY];
        let one: Vec<&[u32]> = vec![&c];
        let (cands1, asg1) = coverage_plan(&one, 1);
        assert_eq!(cands1, vec![NO_RT], "the cap must keep the pattern covering the most rock");
        assert_eq!(
            asg1[0],
            vec![Some(0), Some(0), Some(0), Some(0), Some(0), None],
            "a depth whose own pattern lost the cap must fall back to the largest kept SUBSET it can \
             feed - the four-curve depths here carry all three of NO_RT's curves - while a depth \
             carrying no kept subset stays unclaimed"
        );
    }

    /// **A prediction written at the target's sampling stops claiming the inputs' resolution.**
    ///
    /// Jauhar, 2026-08-07: *"sampling rate, each log has different resolution … Result should adjust
    /// their frequency to log target"*, then *"writing output at target sampling"*. A model fitted
    /// against a target read every 0.5 m predicts at every INPUT depth, so it emits a value every
    /// 0.1524 m. Nothing downstream can tell that curve from one a tool actually logged at that rate.
    ///
    /// Four things could go wrong here and three of them are silent:
    ///
    /// The frame must NOT change. `computed_curves` are read back by exact depth match, so a curve
    /// written at its own coarser sampling lands on depths the well does not have and reads back
    /// all-missing — a whole run lost with nothing saying so.
    ///
    /// A class curve must take the block's MODE, never its mean. The mean of facies 1 and facies 4
    /// is 2.5, which is not a facies and which nothing downstream can reject (SB-MLA-055).
    ///
    /// A depth with no prediction must stay MISSING. Filling it from its block would invent an answer
    /// exactly where the model declined to give one.
    ///
    /// And blocks must be anchored on an ABSOLUTE grid, not on each well's first sample. Per-well
    /// anchoring gives two wells the same block THICKNESS at different block BOUNDARIES, so a bed
    /// mid-block in one well straddles a boundary in the next — the numbers stay plausible and stop
    /// being comparable, which is the trap `TargetSpec.align` exists for.
    #[test]
    fn a_prediction_written_at_the_target_sampling_never_claims_the_inputs_resolution() {
        // 0.25 m sampling, blocked to 0.5 m. Bins are floor(d / 0.5): 0,0,1,1,2,2.
        let depth: Vec<f32> = vec![0.0, 0.25, 0.5, 0.75, 1.0, 1.25];
        let mut v: Vec<f32> = vec![1.0, 3.0, 10.0, 20.0, 5.0, f32::NAN];
        let n = block_to_step(&depth, &mut v, 0.5, false);
        assert_eq!(n, 3, "three 0.5 m blocks carry live samples");
        assert_eq!(v.len(), depth.len(), "the FRAME must not change - only the values");
        assert_eq!(&v[..5], &[2.0, 2.0, 15.0, 15.0, 5.0], "each block's mean, held across the block");
        assert!(v[5].is_nan(), "a depth the model did not answer must stay missing, not inherit its block");

        // A class curve. Same block, codes 1, 1, 4.
        let cd: Vec<f32> = vec![0.0, 0.1, 0.2];
        let mut cv: Vec<f32> = vec![1.0, 1.0, 4.0];
        block_to_step(&cd, &mut cv, 0.5, true);
        assert_eq!(cv, vec![1.0, 1.0, 1.0], "a class block takes its commonest CODE, never a mean");
        assert!(!cv.contains(&2.0), "the mean of facies 1 and facies 4 is 2.5 and is not a facies");

        // The absolute grid. These two depths straddle 0.5, so they are two blocks and keep their
        // own values. Anchored on this well's first sample (0.4) they would both land in one block
        // and be averaged to 5 — plausible, and not the same rock as the next well's blocks.
        let ad: Vec<f32> = vec![0.4, 0.6];
        let mut av: Vec<f32> = vec![2.0, 8.0];
        assert_eq!(block_to_step(&ad, &mut av, 0.5, false), 2);
        assert_eq!(av, vec![2.0, 8.0], "blocks are anchored on an absolute grid, not on the well's first sample");

        // A step that is not a thickness changes nothing rather than dividing by it.
        let mut zv: Vec<f32> = vec![1.0, 2.0];
        assert_eq!(block_to_step(&[0.0, 1.0], &mut zv, 0.0, false), 0);
        assert_eq!(zv, vec![1.0, 2.0]);

        // And the class/quantity split is decided by (task, suffix), never by looking at values: a
        // classifier's own output is codes, the `_PROB` beside it is a real number that averages.
        assert!(output_is_class("classification", ""));
        assert!(!output_is_class("classification", "_PROB"));
        assert!(output_is_class("clustering", ""));
        assert!(!output_is_class("regression", ""));
        assert!(!output_is_class("reduction", "1"));
    }

    /// **SB-MLA-035 — a transformed quantity is a distinct quantity, with its own name and its own
    /// unit.** Permeability is fitted in log10 space because that is where the relation is linear;
    /// what the model then predicts is log10(mD), and until now it was written under the name the
    /// user typed, in a registry that had nowhere to record a unit at all. The failure is quiet and
    /// expensive: a mean of −0.4 under a header reading mD is not a wrong-looking number, it is
    /// 0.398 mD reported as a negative permeability, and every average, cutoff and net-pay sum
    /// built on it inherits the error.
    ///
    /// So the two quantities become two curves. The model's own output keeps the log unit under a
    /// name that says so; the back-transform is written as a SEPARATE, separately-united curve and
    /// announced in the notes, which is the "explicit step" the requirement asks for — an in-place
    /// back-transform would be the invisible one.
    ///
    /// Skips where scikit-learn is absent, so the green gate never depends on it; the row-lockstep
    /// half above is the one that fails the build.
    #[test]
    fn a_log_fitted_prediction_and_its_back_transform_are_two_curves_with_two_units() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let Some(_) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 60usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-LOG10", None, None, Some(0.0)).unwrap();
        let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32).collect();
        db::insert_standard_curves(
            &conn, well, depths.clone(), gr.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        // A permeability spanning four decades — the shape that makes the log fit the right one —
        // carried in the generic store WITH ITS UNIT, which is where an imported core measurement
        // actually arrives.
        let wid = well.to_string();
        let perm: Vec<f32> = gr.iter().map(|g| 10f32.powf(g / 15.0)).collect();
        let cid = db::upsert_curve_meta(&conn, &wid, "RAW", "PERM", Some("mD"), None, Some("test"), None).unwrap();
        db::insert_curve_samples(&conn, &cid, &depths, &perm).unwrap();

        let dbm = Mutex::new(conn);
        let mut req = mk_req("regression", &["GR"], Some("PERM"), &[wid.clone()], &[wid.clone()]);
        req.output_curve = "PERM_EST".into();
        req.output_set = Some("ML_LOG10".into());
        req.target_transform = Some("log10".into());
        let r = run_ml(&dbm, &req, None);
        assert!(r.error.is_none(), "the log-fitted run failed: {:?}", r.error);

        // Two curves, two names, and the plain name is NOT the log-space one.
        assert!(
            r.outputs.contains(&"PERM_EST_LOG10".to_string()) && r.outputs.contains(&"PERM_EST".to_string()),
            "a transformed run must write both quantities, got {:?}",
            r.outputs,
        );

        let conn = dbm.lock().unwrap();
        assert_eq!(
            db::curve_unit_for(&conn, &wid, "PERM_EST_LOG10").as_deref(),
            Some("log10(mD)"),
            "the model's own output carries the unit of the space it was fitted in",
        );
        assert_eq!(
            db::curve_unit_for(&conn, &wid, "PERM_EST").as_deref(),
            Some("mD"),
            "the back-transform carries the target's own unit",
        );

        let read = |name: &str| -> Vec<f32> {
            let mut st = conn
                .prepare("SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 ORDER BY depth")
                .unwrap();
            st.query_map(duckdb::params![&wid, name], |r| r.get::<_, f32>(0))
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        };
        let logs = read("PERM_EST_LOG10");
        let lin = read("PERM_EST");
        assert_eq!(logs.len(), n);
        assert_eq!(lin.len(), n);
        for (l, v) in logs.iter().zip(&lin) {
            assert!(
                (10f32.powf(*l) - v).abs() <= v.abs() * 1e-3 + 1e-6,
                "the back-transform must be 10^(log-space prediction): 10^{l} vs {v}",
            );
        }
        // The trap the requirement names: the curve wearing the mD header must not carry log-space
        // numbers. Across four decades of permeability a mean below 1 would be exactly that.
        let mean_lin = lin.iter().sum::<f32>() / lin.len() as f32;
        assert!(mean_lin > 1.0, "the mD curve reads {mean_lin} - that is a log-space number under an mD header");
        let (max_log, max_lin) = (
            logs.iter().cloned().fold(f32::MIN, f32::max),
            lin.iter().cloned().fold(f32::MIN, f32::max),
        );
        assert!(
            max_log < max_lin / 10.0,
            "the two curves must be in genuinely different spaces, got {max_log} and {max_lin} - \
             a copy of one curve under two names would pass every other check here",
        );

        // And the report says which space its scores are in, so an R2 cannot be quoted as a claim
        // about the other one.
        assert_eq!(r.metrics.get("target_transform").and_then(|v| v.as_str()), Some("log10"));
        assert_eq!(r.metrics.get("metric_space").and_then(|v| v.as_str()), Some("log10(mD)"));
        assert!(
            r.notes.iter().any(|nt| nt.contains("PERM_EST_LOG10") && nt.contains("log10(mD)")),
            "the transform must be announced by name, got {:?}",
            r.notes,
        );

        // The requirement is about the DELIVERABLE, so it is checked there: the LAS a client
        // receives must not carry a log-space column under a permeability header. That is the
        // failure in its most expensive form — the number leaves the building attached to the
        // wrong unit, and the reader has no way to tell.
        let dir = std::env::temp_dir().join(format!("sandibumi-mla035-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let las = dir.join("out.las");
        crate::export::export_las(&conn, &wid, las.to_str().unwrap()).unwrap();
        let text = std::fs::read_to_string(&las).unwrap();
        let unit_of = |mnemonic: &str| -> String {
            text.lines()
                .find(|l| l.trim_start().starts_with(&format!("{mnemonic} ")) || l.trim_start().starts_with(&format!("{mnemonic}.")))
                .and_then(|l| l.split_once('.'))
                .map(|(_, rest)| rest.split_whitespace().next().unwrap_or("").to_string())
                .unwrap_or_default()
        };
        assert_eq!(unit_of("PERM_EST_LOG10"), "log10(mD)", "the LAS header must name the log space:\n{text}");
        assert_eq!(unit_of("PERM_EST"), "mD", "the back-transform must be exported in the target's unit:\n{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **SB-MLA-003 — a saved model identifies the exact training ROWS, not merely the wells.**
    ///
    /// `trained_on` plus `n_train` narrows a re-run but does not pin it: the same wells at a later
    /// log-set version are different rows with the same names and very possibly the same count. So
    /// the pin is from both sides. Identical rows must give an identical hash — otherwise every
    /// re-fit reads as a new training set and the record becomes noise nobody checks — and a single
    /// changed VALUE must give a different one, even where the well list, the sample count and the
    /// feature list are untouched, which is exactly the case `trained_on` cannot see.
    ///
    /// The two canonicalisations are pinned as well, because both are ways for "nothing changed" to
    /// hash as "something changed": an f32 NaN has millions of bit patterns and −0.0 is not 0.0's
    /// bit pattern, so hashing the raw bytes of numerically identical matrices would not be stable.
    #[test]
    fn a_training_fingerprint_is_stable_for_the_same_rows_and_changes_for_one_different_value() {
        let feats = vec!["GR".to_string(), "RHOB".to_string()];
        let x: Vec<f32> = (0..20).map(|i| i as f32 * 0.5).collect();
        let y: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let g: Vec<f32> = vec![0.0; 10];
        let base = training_fingerprint(&feats, 2, &x, &y, &g);
        assert_eq!(base, training_fingerprint(&feats, 2, &x, &y, &g), "the same rows must fingerprint the same");
        assert_eq!(base.len(), 16, "a fixed-width hex digest, so it can be shown and compared by eye");

        // One value, in the middle, changed by an amount no well list or count would notice.
        let mut x2 = x.clone();
        x2[7] += 0.001;
        assert_ne!(base, training_fingerprint(&feats, 2, &x2, &y, &g), "one changed feature value is a different training set");
        let mut y2 = y.clone();
        y2[3] += 0.001;
        assert_ne!(base, training_fingerprint(&feats, 2, &x, &y2, &g), "one changed target value is a different training set");

        // Order is part of the matrix. The same rows shuffled fit the same model only for
        // order-independent algorithms, and the record cannot know which was used.
        let mut y3 = y.clone();
        y3.swap(0, 9);
        assert_ne!(base, training_fingerprint(&feats, 2, &x, &y3, &g), "row order is part of the record");

        // The names ride along: identical numbers under different mnemonics are a different set,
        // and a reordered feature list is a different model under this repo's ordering contract.
        assert_ne!(
            base,
            training_fingerprint(&["GR".into(), "NPHI".into()], 2, &x, &y, &g),
            "the feature names are part of the fingerprint",
        );
        assert_ne!(
            base,
            training_fingerprint(&["RHOB".into(), "GR".into()], 2, &x, &y, &g),
            "feature ORDER is part of the fingerprint",
        );

        // And the canonicalisations, from both sides: numerically identical must hash identically.
        let pos = vec![0.0f32; 10];
        let neg = vec![-0.0f32; 10];
        assert_eq!(
            training_fingerprint(&feats, 1, &pos, &y, &g),
            training_fingerprint(&feats, 1, &neg, &y, &g),
            "-0.0 and 0.0 are the same number and must be the same row",
        );
        let nan_a = vec![f32::NAN; 4];
        let nan_b = vec![f32::from_bits(0x7fc0_0001); 4];
        assert!(nan_b[0].is_nan());
        assert_eq!(
            training_fingerprint(&feats, 1, &nan_a, &y, &g),
            training_fingerprint(&feats, 1, &nan_b, &y, &g),
            "two NaNs are the same missing value however they were produced",
        );
    }

    /// **SB-MLA-009 — blind-well performance travels with the curve, and its ABSENCE travels too.**
    ///
    /// A net-pay number computed from a predicted permeability whose blind-well R² was 0.31 is a
    /// different claim from one computed from a measured permeability, and nothing downstream can
    /// tell which it received unless the curve says. The cautionary case is a delivered project
    /// where a predicted curve reached a training correlation of 0.99 against a blind-well range of
    /// 0.31–0.70 — a factor of three between the number an analyst sees by default and the number
    /// that describes what the curve can actually predict.
    ///
    /// Which is why the second half matters more than the first: where no blind test was run the
    /// record must say so and carry NO NUMBER. A training metric standing in for a blind one is
    /// that 0.99, and it is worse than a blank because it reads as an answer.
    ///
    /// Pure — `blind_record` is the one place the decision is made, so it can be pinned without a
    /// fit. The end-to-end half rides on the SB-MLA-006 provenance test's log-set reader.
    #[test]
    fn a_curve_carries_the_blind_score_or_says_there_was_none_and_never_a_training_one() {
        let sp = SplitReport {
            fit_wells: vec!["SANDI-1".into(), "SANDI-2".into()],
            blind_wells: vec!["SANDI-3".into()],
            fit_rows: 900,
            blind_rows: 300,
            requested_fraction: 0.3,
            achieved_fraction: 0.25,
            seed: 42,
            mode: "well".into(),
            wells_pooled: 3,
        };
        // A flattering training score sits right beside the blind one in the same object — which is
        // the whole hazard, and why the record names the key it took.
        let m = serde_json::json!({ "r2_train": 0.99, "r2_cv": 0.80, "r2_blind": 0.31 });

        let rec = blind_record(&m, Some(&sp), "regression");
        assert_eq!(rec["performed"], serde_json::json!(true));
        assert_eq!(rec["value"], serde_json::json!(0.31), "the BLIND score, never the training one");
        assert_eq!(rec["metric"], serde_json::json!("R2"));
        assert_eq!(rec["protocol"], serde_json::json!("whole wells"));
        assert_eq!(rec["answers_new_well"], serde_json::json!(true));
        assert_eq!(rec["n_blind_wells"], serde_json::json!(1));

        // Sample mode scores the model on depths centimetres from ones it was fitted on, so it
        // does NOT answer "will this work on the next well" — and the record has to say which
        // question was answered, or the number gets quoted as the other one.
        let by_sample = SplitReport { mode: "sample".into(), blind_wells: vec![], ..sp };
        let rec2 = blind_record(&m, Some(&by_sample), "regression");
        assert_eq!(rec2["protocol"], serde_json::json!("random rows, stratified"));
        assert_eq!(rec2["answers_new_well"], serde_json::json!(false));

        // The other side, and the one that matters most. No split → no number at all.
        let none = blind_record(&m, None, "regression");
        assert_eq!(none["performed"], serde_json::json!(false));
        assert!(none.get("value").is_none(), "a run with no blind test must carry no score: {none}");
        assert!(none["why"].as_str().unwrap_or("").contains("no blind test"), "the absence is stated, not implied");

        // A split whose score never arrived is the same absence, not a half-answer.
        let no_score = blind_record(&serde_json::json!({ "r2_train": 0.99 }), Some(&by_sample), "regression");
        assert_eq!(no_score["performed"], serde_json::json!(false));
        assert!(no_score.get("value").is_none(), "an unscored split must not borrow the training number");

        // A classifier reports accuracy, and must not be handed the regression key.
        let cm = serde_json::json!({ "accuracy_train": 0.98, "accuracy_blind": 0.62, "r2_blind": 0.31 });
        let rc = blind_record(&cm, Some(&by_sample), "classification");
        assert_eq!(rc["metric"], serde_json::json!("accuracy"));
        assert_eq!(rc["value"], serde_json::json!(0.62));
    }

    /// **SB-MLA-010 — the deliverable names every model-derived curve it prints, and no other.**
    ///
    /// Pinned from both sides, because either half alone would pass with the wrong implementation.
    ///
    /// The obvious half is that an ML curve appears, with the model, the ORDERED inputs and the blind
    /// sentence. The half that decides whether the block is worth printing is the SECOND: a reader
    /// checks a provenance table to answer "is the PERM in this report measured?", so a table listing
    /// a run whose curves were superseded hours ago is worse than no table — it names a model that
    /// did not make the number on the page. Driving it from `computed_curves.set_id` is what makes
    /// the two agree; a query over `log_sets` alone reads identically and is wrong.
    ///
    /// Third, a deterministic module's log set must NOT be swept in. VSH from a Larionov equation is
    /// not a prediction, and putting it under a heading that says "PREDICTED by a fitted model" would
    /// be the requirement's own defect committed in the opposite direction.
    #[test]
    fn a_deliverable_names_every_model_derived_curve_it_prints_and_no_superseded_one() {
        use crate::equations::{create_log_set, write_computed_curves_versioned, LogSetSpec};
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, well, "SANDI-1", None, None, Some(0.0)).unwrap();
        let id = well.to_string();
        let depth: Vec<f32> = (0..5).map(|i| 1000.0 + i as f32).collect();
        let vals = vec![1.0f32; 5];

        let ml_params = |score: f64| {
            serde_json::json!({
                "algorithm": "rf",
                "model_name": "PERM_FROM_CORE",
                "target": "PERM_CORE",
                "train_hash": "0123456789abcdef",
                "blind": { "performed": true, "metric": "R2", "value": score,
                           "protocol": "whole wells", "answers_new_well": true,
                           "n_blind_wells": 2, "n_blind_rows": 300 },
            })
            .to_string()
        };
        // The order is the apply contract, so it is the order the block must print.
        let inputs = serde_json::to_string(&["GR", "RHOB", "NPHI"]).unwrap();

        // Run 1, then run 2 over the SAME curve name: the second write deletes the first's rows.
        let mk = |score: f64| {
            let spec = LogSetSpec {
                set_name: "ML".into(),
                module: "ml:regression:rf".into(),
                params_json: ml_params(score),
                inputs_json: inputs.clone(),
            };
            let (set_id, _) = create_log_set(&conn, &id, &spec).unwrap();
            write_computed_curves_versioned(&conn, &id, &depth, &[("PERM_ML", &vals[..])], &set_id).unwrap();
        };
        mk(0.11);
        mk(0.62);

        // A deterministic module, live on the same well.
        let eq = LogSetSpec {
            set_name: "VSH".into(),
            module: "equation:vsh_linear".into(),
            params_json: "{}".into(),
            inputs_json: "[\"GR\"]".into(),
        };
        let (eq_set, _) = create_log_set(&conn, &id, &eq).unwrap();
        write_computed_curves_versioned(&conn, &id, &depth, &[("VSH", &vals[..])], &eq_set).unwrap();

        let rows = ml_provenance(&conn, &id);
        assert_eq!(rows.len(), 1, "one live ML curve, one row - not one per run ever made: {rows:?}");
        let r = &rows[0];
        assert_eq!(r.curves, "PERM_ML");
        assert_eq!(r.model, "PERM_FROM_CORE");
        assert_eq!(r.features, "GR, RHOB, NPHI", "printed in the order the model was fitted in");
        assert!(r.blind.contains("0.620"), "the SURVIVING run's blind score: {}", r.blind);
        assert!(!r.blind.contains("0.110"), "the superseded run's score must not be on the page");
        assert!(r.train_hash.starts_with("0123"), "the rows that made it are quotable: {}", r.train_hash);

        // The printed cells, which is what both renderers consume.
        let cells = r.cells();
        assert_eq!(cells.len(), ML_PROV_HEADERS.len(), "a cell per column, or one renderer drops a fact");
        assert!(cells[0].contains("a prediction of PERM_CORE"), "the target is named beside the curve: {}", cells[0]);
        assert!(ML_PROV_CAVEAT.contains("PREDICTED"), "the caveat is the requirement's own sentence");
        assert!(ML_PROV_CAVEAT.is_ascii(), "the PDF writer replaces non-ASCII, so the two documents would differ");

        // The other side: a well with no ML curve gets no block at all, rather than an empty table
        // under a heading that implies there is a model somewhere.
        let clean = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, clean, "SANDI-2", None, None, Some(0.0)).unwrap();
        assert!(ml_provenance(&conn, &clean.to_string()).is_empty());
    }

    /// **SB-MLA-060 — no vendor model or weight file is read, converted or imported.**
    ///
    /// This one is already true, so the test is not a fix: it is the LOCK. The boundary would be
    /// crossed for an entirely reasonable-sounding product reason — "let the customer keep using the
    /// model they already trained" — and the cost of crossing it is not recoverable, so it must fail
    /// the build rather than be caught in review. Reading a weight file to apply it is using the
    /// capability; reading it to understand it is reconstruction. Neither is available.
    ///
    /// Three checks, because there are three doors. A DEPENDENCY that can parse a model artifact is
    /// the widest one — a crate added for some other reason brings the capability with it. A Python
    /// IMPORT is the same door on the other side of the subprocess boundary, where `cargo` cannot
    /// see it. And the SOURCE of model bytes is the invariant itself: the only bytes any runner
    /// deserializes come from a buffer handed to it on stdin, which came from SandiBumi's own
    /// `ml_models` table — no runner opens a file at all.
    ///
    /// Interchange with an incumbent stays possible where the requirement allows it: the vendor's
    /// *outputs*, exported as ordinary curves, come in through LAS and DLIS like any other log.
    #[test]
    fn no_code_path_reads_a_vendor_model_or_weight_file() {
        use std::path::Path;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));

        // 1. Dependencies. A model-format reader in the manifest is the capability, whether or not
        //    anything calls it yet.
        let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
        for crate_name in [
            "onnxruntime", "tract-onnx", "tract-core", "tch", "candle-core", "burn",
            "tensorflow", "safetensors", "hdf5", "netcdf",
        ] {
            assert!(
                !manifest.lines().any(|l| l.trim_start().starts_with(crate_name)),
                "'{crate_name}' can read a trained-model artifact - SB-MLA-060 forbids the capability, \
                 not merely its use",
            );
        }

        // 2. The subprocess side. cargo cannot see a Python import, and the runners are strings.
        for (which, src) in [
            ("train", ml_runner()),
            ("leaderboard", ml_eval_runner()),
            ("apply", ml_apply_runner()),
        ] {
            for module in ["torch", "tensorflow", "keras", "onnx", "h5py", "tflite"] {
                assert!(
                    !src.contains(&format!("import {module}")),
                    "the {which} runner imports {module}, which exists to read somebody else's weights",
                );
            }
            // 3. The invariant. No runner opens a file — every byte it deserializes arrived on
            //    stdin from SandiBumi's own table.
            assert!(
                !src.contains("open("),
                "the {which} runner opens a file; model bytes must arrive on stdin from ml_models",
            );
        }
        assert!(
            ML_APPLY_RUNNER.contains("joblib.load(_io.BytesIO(blob))"),
            "the apply runner must deserialize from the in-memory blob it was handed, never a path",
        );
        assert_eq!(
            ML_APPLY_RUNNER.matches("joblib.load").count(),
            1,
            "one load site, or the one that is checked is not the only one",
        );

        // 4. And no file-format handler anywhere names a model artifact. A dialog filter is how the
        //    first one would arrive: the reader gets written because the picker already offers it.
        let mut offenders: Vec<String> = Vec::new();
        let scan = |dir: std::path::PathBuf, ext: &str, out: &mut Vec<String>| -> usize {
            let Ok(entries) = std::fs::read_dir(&dir) else { return 0 };
            let mut seen = 0usize;
            for e in entries.flatten() {
                let p = e.path();
                if !p.extension().map(|x| x == ext).unwrap_or(false) {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&p) else { continue };
                seen += 1;
                for bad in [".onnx", ".hdf5", ".caffemodel", ".tflite", ".safetensors", ".ckpt", ".pth"] {
                    // This very test names them, so it cannot be its own offender.
                    if text.contains(bad) && !p.ends_with("ml.rs") {
                        out.push(format!("{} names {bad}", p.display()));
                    }
                }
            }
            seen
        };
        let rs = scan(root.join("src"), "rs", &mut offenders);
        let ts = scan(root.parent().expect("repo root").join("src").join("ui"), "ts", &mut offenders);
        assert!(offenders.is_empty(), "a vendor model artifact is named in the source: {offenders:?}");
        // A file scan that found nothing passes for the wrong reason, which on a check this
        // load-bearing is worse than a failure — it would go on passing after the tree moved.
        assert!(rs > 40 && ts > 20, "the scan reached {rs} Rust and {ts} TypeScript files - it is looking in the wrong place");
    }

    /// SB-MLA-001. Every parameter the runner reads must go through `P`, which records what was
    /// actually used and whether it was a default. A stray `p.get` is the whole defect — it reads
    /// a value, changes the answer with it, and leaves nothing in the record — and `cargo check`
    /// cannot see it, because the runner is a string.
    ///
    /// The second assertion is the one that would have caught a real mistake made writing this:
    /// a blanket rewrite of `p.get(` also rewrites the one inside `P` itself, and `P` calling `P`
    /// is a stack overflow at RUN time, in a subprocess, on a machine with scikit-learn — which
    /// the green gate would never reach.
    #[test]
    fn every_parameter_the_runner_reads_is_recorded_as_supplied_or_defaulted() {
        for (name, src) in [("run", ml_runner()), ("eval", ml_eval_runner())] {
            assert!(
                !src.contains("p.get("),
                "{name} runner reads a parameter without recording it - use P(p, key, default)",
            );
            assert!(src.contains("def P(p, key, default):"), "{name} runner is missing the recorder");
        }
        // P must not call itself. `dict.get(p, key)` is the un-rewritable spelling.
        let body = ml_runner();
        let start = body.find("def P(p, key, default):").expect("recorder present");
        let end = body[start..].find("def P_used").expect("P_used follows P") + start;
        assert!(
            body[start..end].contains("dict.get(p, key)"),
            "the recorder must read the dict directly, or a rewrite of p.get turns it into infinite recursion",
        );
        // And the record has to reach the caller, or none of the above is observable.
        assert!(body.contains(r#"metrics["effective_params"] = EFFECTIVE"#), "the record is never emitted");
    }

    /// The percentage is a share of the DATA; the thing held back is a whole WELL. The lopsided
    /// case is the one that matters and the one a well-count split got wrong: five wells of
    /// 3000/1000/500/300/200 rows asked for 30% must land near 1500 ROWS, where "two of the five
    /// wells" would give either 12% or 68% of the rock depending on which two the shuffle drew.
    ///
    /// Pinned from both sides. Too small an ask still yields ONE blind well rather than none, and
    /// 1.0 still leaves one well to fit on — a blind test that silently produces no blind well is
    /// the clean-looking nothing SB-CORE-002 forbids, and a "split" with nothing to train on is
    /// that failure mirrored. And a well holding most of the field is NOT held out for a 30% test:
    /// without that, the guarantee could be met by a fallback that grabs the first shuffled well
    /// and calls 97% a 30% split.
    #[test]
    fn a_share_of_the_samples_is_reached_with_whole_wells_and_always_leaves_one_on_each_side() {
        let rows = |c: &[usize], b: &[usize]| -> usize { b.iter().map(|&i| c[i]).sum() };

        // Five EQUAL wells: 30% of the data is 1.5 wells' worth, so either 1 or 2 lands 10 points
        // off and the pick is the seed's. What must hold is that it is one of those two - never 0,
        // never 3.
        let equal = [1000usize; 5];
        let b = split_blind_wells(&equal, 0.3, 42);
        assert!((1..=2).contains(&b.len()), "1.5 wells' worth is 1 or 2, got {}", b.len());

        // Five LOPSIDED wells - the case that made the old well-count split wrong. Asking for 30%
        // of 5000 rows (1500) must land near 1500 rows, NOT on "two of the five wells", which here
        // would be either 600 rows (12%) or 3400 (68%) depending on the draw.
        let lop = [3000usize, 1000, 500, 300, 200];
        for seed in 1..30u64 {
            let b = split_blind_wells(&lop, 0.3, seed);
            let got = rows(&lop, &b);
            assert!(
                (900..=2100).contains(&got),
                "seed {seed}: asked 1500 rows, got {got} from wells {b:?}"
            );
        }

        // A well that is most of the field cannot be held out for a 30% test, and the fallback is
        // the CLOSEST well rather than the first shuffled one.
        let whale = [10_000usize, 400, 300];
        for seed in 1..30u64 {
            let b = split_blind_wells(&whale, 0.3, seed);
            assert!(!b.contains(&0), "seed {seed}: the 10k-row well is not 30% of anything");
        }

        // Floors and degenerate asks.
        assert_eq!(split_blind_wells(&equal, 0.01, 42).len(), 1, "a tiny ask still holds one well");
        assert_eq!(split_blind_wells(&equal, 1.0, 42).len(), 4, "all-blind still leaves one to fit on");
        assert!(split_blind_wells(&[1000], 0.5, 42).is_empty(), "one well cannot be split");
        assert!(split_blind_wells(&equal, 0.0, 42).is_empty(), "no split asked for");
        assert!(split_blind_wells(&[0, 0, 0], 0.3, 42).is_empty(), "no rows, nothing to divide");

        // Every index is a real well, and no well is on both sides of the line.
        let seven = [900usize, 100, 400, 250, 800, 150, 600];
        let b = split_blind_wells(&seven, 0.4, 1);
        assert!(b.iter().all(|&i| i < 7));
        let mut u = b.clone();
        u.dedup();
        assert_eq!(u, b, "a well is held out once or not at all");
    }

    /// Jauhar, 2026-08-07: *"real 30% of data, from existing assume 10000 of data, random sample
    /// 3000 data from there with similar statistic taken to be tested as blind"*. Two claims, and
    /// the test pins both because either alone would pass a wrong implementation.
    ///
    /// EXACTLY 3000 of 10000 — a plain per-stratum rounding drifts (ten strata of 1000 at 30% is
    /// fine, but ten of 55 rounds to 170 against a target of 165), and a user who asked for 3000
    /// should get 3000, not "about 3000".
    ///
    /// And REPRESENTATIVE — every stratum contributes its own 30%, which is the entire difference
    /// between this and a flat random draw. Pinned on a deliberately lopsided population where a
    /// flat draw would routinely miss the rare stratum: without stratification the rarest class
    /// lands wholly on one side often enough to make a blind accuracy meaningless, and nothing
    /// downstream could tell.
    #[test]
    fn a_sample_split_draws_the_exact_count_and_keeps_every_stratum_in_proportion() {
        // 10 000 rows, ten strata, deliberately unequal: 5 000 / 2 000 / 1 000 / … / 20.
        let sizes = [5000usize, 2000, 1000, 700, 500, 400, 200, 120, 60, 20];
        let mut strata: Vec<i64> = Vec::new();
        for (s, n) in sizes.iter().enumerate() {
            strata.extend(std::iter::repeat(s as i64).take(*n));
        }
        assert_eq!(strata.len(), 10_000);

        let mask = split_blind_samples(&strata, 0.30, 42);
        let held = mask.iter().filter(|v| **v > 0.5).count();
        assert_eq!(held, 3000, "asked for 3000 of 10000 rows");

        // Each stratum contributes its own share, within one row of rounding plus the total
        // correction. A flat random draw would satisfy the count above and fail this.
        let mut at = 0usize;
        for (s, n) in sizes.iter().enumerate() {
            let got = mask[at..at + n].iter().filter(|v| **v > 0.5).count();
            let want = (*n as f64 * 0.30).round() as usize;
            assert!(
                got.abs_diff(want) <= 2,
                "stratum {s} ({n} rows): held {got}, proportional share is {want}"
            );
            assert!(got > 0 && got < *n, "stratum {s} keeps rows on both sides");
            at += n;
        }
    }

    /// A stratum of ONE cannot be divided, and the row goes to the FIT side. Blind is where a score
    /// is computed, and a class the model was never shown scores zero for a reason that is not the
    /// model's — so the undividable row is spent on training, where it at least teaches something.
    ///
    /// The floors from the well split hold here too, and for the same reason: a blind test that
    /// produces no blind row, or leaves nothing to fit on, is the clean-looking nothing
    /// SB-CORE-002 forbids.
    #[test]
    fn an_undividable_stratum_goes_to_the_fit_side_and_both_sides_keep_rows() {
        // Four singleton strata beside one real one.
        let strata: Vec<i64> = vec![0, 1, 2, 3, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];
        let mask = split_blind_samples(&strata, 0.5, 7);
        for i in 0..4 {
            assert!(mask[i] < 0.5, "a stratum of one row is not held blind (row {i})");
        }
        let held = mask.iter().filter(|v| **v > 0.5).count();
        assert!(held > 0 && held < strata.len(), "both sides carry rows");

        // Degenerate asks, pinned from both sides.
        assert!(split_blind_samples(&[0, 0, 0], 0.0, 1).iter().all(|v| *v < 0.5), "no split asked for");
        assert!(split_blind_samples(&[0], 0.5, 1).iter().all(|v| *v < 0.5), "one row cannot be split");
        let all = split_blind_samples(&vec![0i64; 20], 1.0, 1);
        assert!(all.iter().any(|v| *v < 0.5), "all-blind still leaves something to fit on");

        // Seeded and reproducible - SB-MLA-008.
        assert_eq!(split_blind_samples(&strata, 0.5, 7), split_blind_samples(&strata, 0.5, 7));
        let pop: Vec<i64> = (0..500).map(|i| i % 5).collect();
        assert_ne!(
            split_blind_samples(&pop, 0.3, 1),
            split_blind_samples(&pop, 0.3, 2),
            "the seed must actually choose the rows"
        );
    }

    /// A classifier stratifies on the CLASS; a regressor has no classes, so it stratifies on
    /// deciles of the target. Raw values would give a continuous target as many strata as rows, and
    /// every one of them would be a stratum of one — which the guard above sends to the fit side,
    /// leaving nothing blind at all. This pins the boundary between the two readings.
    #[test]
    fn a_continuous_target_is_stratified_by_decile_and_a_class_target_by_class() {
        let classes: Vec<f32> = vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0];
        assert_eq!(strata_for(&classes, true), vec![0, 0, 1, 1, 2, 2]);

        // 100 distinct porosities: as classes that would be 100 strata of one.
        let cont: Vec<f32> = (0..100).map(|i| i as f32 * 0.003).collect();
        assert_eq!(strata_for(&cont, true).len(), 100);
        let dec = strata_for(&cont, false);
        let mut uniq: Vec<i64> = dec.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert!(uniq.len() <= 10 && uniq.len() >= 8, "deciles, got {} strata", uniq.len());
        assert!(dec[0] < dec[99], "the strata follow the target, low to high");

        // And the whole point: the continuous target still splits.
        let mask = split_blind_samples(&dec, 0.3, 3);
        let held = mask.iter().filter(|v| **v > 0.5).count();
        assert_eq!(held, 30, "30 of 100 rows");
    }

    /// The fraction the user typed and the fraction they got are different numbers whenever whole
    /// wells cannot divide the rows exactly — which is most of the time. Reporting only the request
    /// would make the blind score a claim about an unstated amount of rock, so the two are pinned
    /// to travel together and the achieved one is pinned to be the truth about the ROWS, not a copy
    /// of the ask.
    #[test]
    fn the_split_reports_the_share_of_data_it_reached_not_the_share_it_was_asked_for() {
        let counts = [3000usize, 1000, 500, 300, 200];
        let total: usize = counts.iter().sum();
        let blind = split_blind_wells(&counts, 0.3, 42);
        let blind_rows: usize = blind.iter().map(|&i| counts[i]).sum();
        let achieved = blind_rows as f64 / total as f64;

        assert!(blind_rows > 0 && blind_rows < total, "both sides carry rows");
        assert!(
            (achieved - 0.3).abs() < 0.2,
            "asked 30% of the data, reached {:.1}% - the target is on ROWS",
            achieved * 100.0
        );
        // The whole point: it is allowed to miss, and the miss must be visible rather than rounded
        // away into the requested number.
        assert!(
            (achieved * (total as f64) - blind_rows as f64).abs() < 1e-9,
            "the achieved fraction is computed from the rows, not restated from the request"
        );
    }

    /// A blind score that moves when nothing changed cannot be cited, so the shuffle is seeded and
    /// the seed travels in the request (SB-MLA-008). The second half matters as much: if the seed
    /// were ignored the split would be a fixed prefix, and "random" would be a claim rather than a
    /// behaviour — every study in the field would hold out the same wells.
    #[test]
    fn the_same_seed_splits_the_same_wells_and_a_different_seed_does_not() {
        let c = [700usize, 300, 1200, 450, 900, 150, 600, 800, 250, 1100, 350, 500];
        assert_eq!(split_blind_wells(&c, 0.25, 7), split_blind_wells(&c, 0.25, 7));
        let mut differs = false;
        for s in 1..40u64 {
            if split_blind_wells(&c, 0.25, s) != split_blind_wells(&c, 0.25, 7) {
                differs = true;
                break;
            }
        }
        assert!(differs, "the seed must actually choose the wells, not decorate a fixed order");
    }

    /// SB-MLA-T13 on the python path. When SOME apply wells have data the run proceeds, so the
    /// whole-run `n_apply == 0` refusal never fires and the empty wells fall through to the
    /// write loop — the case that was writing an all-NaN curve and calling it a warning. The
    /// refusal must name WHICH emptiness it is: masked out and never measured call for opposite
    /// fixes (widen the mask, or go and find the curve), and "no complete samples" said both.
    #[test]
    fn a_well_with_nothing_to_predict_names_which_emptiness_it_is() {
        let aw = |depth: usize, masked: usize| ApplyWell {
            well_id: "W".into(),
            depth: vec![0.0; depth],
            idx: vec![],
            masked,
            error: None,
        };

        let never_measured = no_rows_reason(&aw(100, 0));
        assert!(
            never_measured.contains("input curve") && !never_measured.contains("mask"),
            "nothing masked -> the cause is the data, not the mask: {never_measured:?}",
        );

        let all_masked = no_rows_reason(&aw(100, 100));
        assert!(
            all_masked.contains("mask") && all_masked.contains("100"),
            "everything masked -> the cause is the mask, and it states how many: {all_masked:?}",
        );

        let both = no_rows_reason(&aw(100, 40));
        assert!(
            both.contains("mask") && both.contains("40") && both.contains("input curve"),
            "a partial mask leaves both causes in play, so both are named: {both:?}",
        );
    }

    /// The blind wells are wells the model never saw. Four wells at 50% leaves two on each side,
    /// and the run must report by NAME which two were fitted and which two were scored — a blind
    /// score is a claim about specific rock, and "70%" does not say whose. Needs sklearn.
    #[test]
    #[ignore]
    fn a_blind_split_scores_the_model_on_wells_it_was_never_fitted_on() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 60usize;
        let mut ids = Vec::new();
        for w in 0..4 {
            let id = Uuid::new_v4();
            db::insert_well(&conn, id, &format!("SANDI-{w}"), None, None, Some(0.0)).unwrap();
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            // Each well spans its own GR window, so a model fitted on two of them is genuinely
            // extrapolating into the other two - a split by SAMPLE would hide that completely.
            let gr: Vec<f32> = (0..n).map(|i| 20.0 + (w * 60) as f32 + i as f32).collect();
            let rhob: Vec<f32> = (0..n).map(|i| 2.0 + (i % 7) as f32 * 0.05).collect();
            db::insert_standard_curves(
                &conn, id, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
                rhob.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            let t: Vec<f32> = (0..n).map(|i| 0.4 - 0.002 * gr[i] + 0.05 * rhob[i]).collect();
            crate::equations::write_computed_curve(&conn, &id.to_string(), &depths, "PHIT_CORE", &t).unwrap();
            ids.push(id.to_string());
        }
        let dbm = Mutex::new(conn);
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }

        let mut req = mk_req("regression", &["GR", "RHOB"], Some("PHIT_CORE"), &ids, &ids);
        req.blind_fraction = Some(0.5);
        req.split_seed = Some(11);
        let r = run_ml(&dbm, &req, None);
        assert!(r.error.is_none(), "run failed: {:?}", r.error);

        let sp = r.split.expect("a run with a blind fraction reports the split it performed");
        assert_eq!(sp.blind_wells.len(), 2, "4 wells at 50% is 2 blind");
        assert_eq!(sp.fit_wells.len(), 2);
        for b in &sp.blind_wells {
            assert!(!sp.fit_wells.contains(b), "a well is fitted on or held blind, never both");
        }

        let m = &r.metrics;
        assert!(m.get("r2_blind").and_then(|v| v.as_f64()).is_some(), "a blind score is reported: {m}");
        // The counts are the split, restated from the runner's own view of the rows it received.
        // If they disagreed with the well lists above, one of the two would be describing a run
        // that did not happen.
        assert_eq!(m["n_blind_wells"].as_u64(), Some(2));
        assert_eq!(m["n_fit_wells"].as_u64(), Some(2));
        assert_eq!(
            m["n_fit"].as_u64().unwrap() + m["n_blind"].as_u64().unwrap(),
            m["n_train"].as_u64().unwrap(),
            "every labelled row is on exactly one side of the split",
        );
        // The relation is exactly linear, so a correct blind fit still recovers it. This is what
        // stops the test passing on a run that held out the wells and then quietly fitted on them.
        let r2b = m["r2_blind"].as_f64().unwrap();
        assert!(r2b > 0.99, "the underlying relation is linear, so the blind r2 should be high: {r2b}");
    }

    /// The sample-mode twin of the test above, on the same four wells, and the contrast between
    /// them is the point. Jauhar asked for a real row-level draw: 30% of the pooled samples, taken
    /// with similar statistics, whatever well each row came from.
    ///
    /// Three things must hold that the well-mode path cannot show. The count is EXACT (four wells
    /// of 60 rows is 240; 30% is 72, not "about 70"). Both sides draw on EVERY well, so the well
    /// lists are deliberately empty rather than listing all four under both headings. And the
    /// balance table comes back, because "similar statistic" is a claim that has to be evidenced
    /// rather than asserted — each well here occupies its own GR window, so an unstratified draw
    /// would show visibly different means. Needs sklearn.
    #[test]
    #[ignore]
    fn a_sample_split_draws_rows_from_every_well_and_reports_how_alike_the_two_sides_are() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 60usize;
        let mut ids = Vec::new();
        for w in 0..4 {
            let id = Uuid::new_v4();
            db::insert_well(&conn, id, &format!("SANDI-{w}"), None, None, Some(0.0)).unwrap();
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            let gr: Vec<f32> = (0..n).map(|i| 20.0 + (w * 60) as f32 + i as f32).collect();
            let rhob: Vec<f32> = (0..n).map(|i| 2.0 + (i % 7) as f32 * 0.05).collect();
            db::insert_standard_curves(
                &conn, id, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
                rhob.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            let t: Vec<f32> = (0..n).map(|i| 0.4 - 0.002 * gr[i] + 0.05 * rhob[i]).collect();
            crate::equations::write_computed_curve(&conn, &id.to_string(), &depths, "PHIT_CORE", &t).unwrap();
            ids.push(id.to_string());
        }
        let dbm = Mutex::new(conn);
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }

        let mut req = mk_req("regression", &["GR", "RHOB"], Some("PHIT_CORE"), &ids, &ids);
        req.blind_fraction = Some(0.30);
        req.split_seed = Some(11);
        req.split_mode = Some("sample".into());
        let r = run_ml(&dbm, &req, None);
        assert!(r.error.is_none(), "run failed: {:?}", r.error);

        let sp = r.split.expect("a run with a blind fraction reports the split it performed");
        assert_eq!(sp.mode, "sample");
        assert_eq!(sp.blind_rows + sp.fit_rows, 240, "four wells of 60 labelled rows");
        assert_eq!(sp.blind_rows, 72, "30% of 240 is exact, not approximate");
        assert!((sp.achieved_fraction - 0.30).abs() < 1e-9, "a row draw hits the fraction exactly");
        assert!(
            sp.blind_wells.is_empty() && sp.fit_wells.is_empty(),
            "every well is on both sides, so naming them would say nothing",
        );
        assert_eq!(sp.wells_pooled, 4, "how much rock this is, which the well lists no longer say");

        let m = &r.metrics;
        assert_eq!(m["n_blind"].as_u64(), Some(72), "the runner received the mask Rust drew");
        assert_eq!(m["n_fit"].as_u64(), Some(168));
        // Cross-validation stays grouped by WELL in sample mode, deliberately - so the run carries
        // one score that cannot leak beside the one that can.
        assert!(m.get("r2_cv").and_then(|v| v.as_f64()).is_some(), "a well-grouped CV score too: {m}");

        // "Similar statistic" - evidenced. Each well spans its own GR window, so an unstratified
        // draw would show the two sides' means pulling apart.
        let bal = m["split_balance"].as_array().expect("the balance table travels with the split");
        assert!(bal.iter().any(|e| e["name"] == "(target)"), "the target is compared, not only the inputs");
        for e in bal {
            let (fm, bm) = (e["fit_mean"].as_f64().unwrap(), e["blind_mean"].as_f64().unwrap());
            let sd = e["fit_sd"].as_f64().unwrap().max(1e-9);
            assert!(
                (fm - bm).abs() < 0.25 * sd,
                "{}: fit mean {fm:.4} against blind mean {bm:.4} is not a representative draw",
                e["name"],
            );
        }
    }

    /// SB-MLA-T13 end to end: a two-well clustering run where one well is clusterable and one
    /// carries no reading. The good well is written; the empty well is REFUSED, and refused
    /// before a log set is allocated — a run that reports failure must not also version an
    /// interpretation. Needs sklearn.
    #[test]
    #[ignore]
    fn an_empty_well_beside_a_good_one_is_refused_and_writes_nothing() {
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 40usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let nan = || vec![f32::NAN; n];

        let good = Uuid::new_v4();
        db::insert_well(&conn, good, "SANDI-GOOD", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, good, depths.clone(),
            (0..n).map(|i| 20.0 + (i as f32 * 3.0) % 110.0).collect(),
            nan(), nan(), nan(), nan(), nan(),
        )
        .unwrap();

        let empty = Uuid::new_v4();
        db::insert_well(&conn, empty, "SANDI-EMPTY", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(&conn, empty, depths.clone(), nan(), nan(), nan(), nan(), nan(), nan())
            .unwrap();

        let (gid, eid) = (good.to_string(), empty.to_string());
        let dbm = Mutex::new(conn);
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }
        let r = run_ml(&dbm, &mk_req("clustering", &["GR"], None, &[], &[gid.clone(), eid.clone()]), None);
        assert!(r.error.is_none(), "the run itself succeeds - one well had data: {:?}", r.error);

        let good_res = r.wells.iter().find(|w| w.well_id == gid).expect("good well reported");
        assert!(good_res.error.is_none() && good_res.rows_predicted > 0, "good well: {good_res:?}");

        let empty_res = r.wells.iter().find(|w| w.well_id == eid).expect("empty well reported");
        let msg = empty_res.error.clone().unwrap_or_default();
        assert!(
            msg.contains("input curve") || msg.contains("curve data"),
            "the empty well must be refused BY NAME, not reported clean: {empty_res:?}",
        );
        assert_eq!(empty_res.rows_predicted, 0);

        // And it wrote nothing: no curve, and no log-set version allocated for it.
        let conn = dbm.lock().unwrap();
        let written: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves WHERE well_id = ?",
                duckdb::params![&eid],
                |r| r.get(0),
            )
            .unwrap_or(0);
        assert_eq!(written, 0, "a refused well must not leave an all-missing curve behind");
    }

    // --- Saved models ------------------------------------------------------

    /// Two wells with a clean linear relation: WELL A is cored (has the target), WELL B is not.
    /// Returns (db, id_a, id_b).
    fn two_well_db() -> (std::sync::Mutex<duckdb::Connection>, String, String) {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 40usize;
        let mk = |name: &str, with_target: bool| -> String {
            let id = Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, Some(0.0)).unwrap();
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32).collect();
            let rhob: Vec<f32> = (0..n).map(|i| 2.0 + (i % 7) as f32 * 0.05).collect();
            db::insert_standard_curves(
                &conn, id, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
                rhob.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            if with_target {
                // PHIT = 0.4 - 0.002*GR + 0.05*RHOB — recoverable exactly by a linear fit, so a
                // correctly applied model must reproduce it and a wrongly-ordered one cannot.
                let t: Vec<f32> =
                    (0..n).map(|i| 0.4 - 0.002 * gr[i] + 0.05 * rhob[i]).collect();
                crate::equations::write_computed_curve(&conn, &id.to_string(), &depths, "PHIT_CORE", &t).unwrap();
            }
            id.to_string()
        };
        let a = mk("CORED-1", true);
        let b = mk("BLIND-1", false);
        (std::sync::Mutex::new(conn), a, b)
    }

    /// **SB-MLA-005 — a runtime step is named component by component, and a missing record is not
    /// a mismatch.**
    ///
    /// Pinned from both sides. The obvious half is that a changed version is reported. The half that
    /// decides whether anybody keeps reading these warnings is the second: a model saved before the
    /// record existed must produce NO warning at all. A check that treated "not recorded" as "does
    /// not match" would fire on every model in every project that predates today, and a warning that
    /// fires on everything is one nobody reads when it finally means something.
    #[test]
    fn a_runtime_step_is_named_component_by_component_and_an_unrecorded_one_is_not_a_mismatch() {
        // What the real probe writes: every component it asked about, `null` where it was absent.
        let now = serde_json::json!({
            "python": "3.12.4", "numpy": "2.1.0", "scipy": null,
            "sklearn": "1.6.0", "joblib": "1.4.2", "xgboost": "2.1.0",
        });
        let same = serde_json::json!({
            "python": "3.12.4", "numpy": "2.1.0", "scipy": null,
            "sklearn": "1.5.0", "joblib": "1.4.2", "xgboost": "2.1.0",
        })
        .to_string();
        let notes = runtime_drift(Some(&same), &now);
        assert_eq!(notes.len(), 1, "one note, not one per component");
        assert!(notes[0].contains("sklearn 1.5.0 -> 1.6.0"), "the component is named: {}", notes[0]);
        assert!(!notes[0].contains("numpy"), "a component that matches is not listed: {}", notes[0]);
        assert!(!notes[0].contains("scipy"), "absent then and absent now is not a step: {}", notes[0]);

        // joblib is the SERIALISER. A step here is the one that unpickles the blob, so it must be
        // named rather than folded into "the runtime differs".
        let jl = serde_json::json!({ "joblib": "1.2.0" }).to_string();
        assert!(runtime_drift(Some(&jl), &now)[0].contains("joblib 1.2.0 -> 1.4.2"));

        // Recorded, and now gone: install it, do not match a version.
        let gone = serde_json::json!({ "scipy": "1.14.0" }).to_string();
        assert!(runtime_drift(Some(&gone), &now)[0].contains("scipy 1.14.0 -> not installed"));

        // The reverse, and the case xgboost exists in this record for: the fit had no xgboost, so it
        // ran on the substituted scikit-learn estimator (SB-MLA-012). This machine HAS xgboost, so
        // the same request would fit a different algorithm — the one step a naive "compare the
        // versions we both have" check cannot see, because one side has no version at all.
        let subbed = serde_json::json!({ "xgboost": null }).to_string();
        assert!(
            runtime_drift(Some(&subbed), &now)[0].contains("xgboost not installed -> 2.1.0"),
            "an absence that has become a presence is a runtime step"
        );

        // The other side, and the one that keeps the warning worth reading.
        assert!(runtime_drift(None, &now).is_empty(), "a model with no record cannot have drifted");
        assert!(
            runtime_drift(Some(&now.to_string()), &now).is_empty(),
            "an identical runtime says nothing"
        );
        // A component present now but never recorded is silent: the model predates the probe asking
        // about it, so there is no evidence either way.
        let partial = serde_json::json!({ "python": "3.12.4" }).to_string();
        assert!(runtime_drift(Some(&partial), &now).is_empty());
        // And the mirror: a component the CURRENT probe did not ask about cannot manufacture a step.
        let older_probe = serde_json::json!({ "python": "3.12.4" });
        assert!(runtime_drift(Some(&same), &older_probe).is_empty());
    }

    /// **A model carries the log set it was fitted on, so propagating it needs nothing restated.**
    ///
    /// Jauhar, 2026-08-07: *"user dont need to re input well, data, rerun model again to
    /// propagate"*. The feature list was already locked to the artifact and its ORDER enforced by
    /// the runner; the set those features are READ from was still taken from the caller, so a model
    /// fitted on FINAL porosity could be applied against the live store with nothing saying so —
    /// the same class of defect as a reordered matrix, and the last half of the contract still
    /// outside it.
    ///
    /// Pinned from both sides. A single recorded set is inherited. Every case where there is no ONE
    /// answer returns `None` rather than choosing on the user's behalf — and the three that produce
    /// it are genuinely different situations that happen to demand the same silence: no record at
    /// all, a well read from the live store, and wells read from two different sets.
    #[test]
    fn a_model_carries_the_log_set_it_was_fitted_on_and_never_guesses_between_two() {
        let w = |set: Option<&str>, name: &str| TrainWellRecord {
            well_id: format!("id-{name}"),
            well: name.into(),
            rows: 100,
            masked: 0,
            incomplete: 0,
            set_name: set.map(str::to_string),
            set_id: set.map(|_| "sid".to_string()),
            set_version: set.map(|_| 1),
        };
        let json = |wells: Vec<TrainWellRecord>| {
            serde_json::to_string(&TrainingRecord { mask_curve: None, wells }).unwrap()
        };

        // Every well from one set: that is the set to inherit.
        let one = json(vec![w(Some("FINAL"), "SANDI-1"), w(Some("FINAL"), "SANDI-2")]);
        assert_eq!(training_sets(Some(&one)).as_deref(), Some("FINAL"));

        // Two sets. There is no single answer, and picking one would silently decide which rock the
        // propagation reads - so it declines and the caller's own choice stands.
        let two = json(vec![w(Some("FINAL"), "SANDI-1"), w(Some("RAW"), "SANDI-2")]);
        assert!(training_sets(Some(&two)).is_none(), "two sets have no one answer to inherit");

        // One well read from the live store. The model is NOT "trained on FINAL" - part of it was
        // trained on values that can move - so there is nothing safe to carry forward.
        let mixed = json(vec![w(Some("FINAL"), "SANDI-1"), w(None, "SANDI-2")]);
        assert!(training_sets(Some(&mixed)).is_none(), "a live-store well leaves no frozen set");

        // Nothing recorded, and nothing at all.
        assert!(training_sets(None).is_none(), "a model saved before the record existed");
        assert!(training_sets(Some(&json(vec![]))).is_none(), "no wells, no set");
        assert!(training_sets(Some("not json")).is_none());
    }

    /// **SB-MLA-002 — the training log set is recorded, and a set that has moved is named.**
    ///
    /// The point is not that applying the model is wrong. It is that the rock the model learned from
    /// is no longer what that set name returns, so a re-fit today would not reproduce this model and
    /// a reviewer comparing the two would have nothing to point at. Both drift cases are pinned, and
    /// so is the case that must stay quiet.
    #[test]
    fn a_model_records_the_log_set_its_rows_came_from_and_names_it_when_it_has_moved() {
        use crate::equations::{create_log_set, LogSetSpec};
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, well, "SANDI-1", None, None, Some(0.0)).unwrap();
        let id = well.to_string();
        let spec = LogSetSpec {
            set_name: "FINAL".into(),
            module: "equation:phie".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (v1, ver1) = create_log_set(&conn, &id, &spec).unwrap();
        assert_eq!(ver1, 1);

        let roster = |set_id: Option<&str>, version: Option<i64>| {
            serde_json::to_string(&TrainingRecord {
                mask_curve: None,
                wells: vec![TrainWellRecord {
                    well_id: id.clone(),
                    well: "SANDI-1".into(),
                    rows: 500,
                    masked: 0,
                    incomplete: 0,
                    set_name: Some("FINAL".into()),
                    set_id: set_id.map(str::to_string),
                    set_version: version,
                }],
            })
            .unwrap()
        };

        // Nothing has changed: silence.
        assert!(training_set_drift(&conn, Some(&roster(Some(&v1), Some(1)))).is_empty());

        // Somebody re-ran porosity. The set name still resolves — to different rock.
        create_log_set(&conn, &id, &spec).unwrap();
        let moved = training_set_drift(&conn, Some(&roster(Some(&v1), Some(1))));
        assert_eq!(moved.len(), 1);
        assert!(moved[0].contains("superseded"), "{}", moved[0]);
        assert!(moved[0].contains("SANDI-1 (FINAL v1 -> v2)"), "named, not merely counted: {}", moved[0]);

        // A set id that is not there at all is a different situation and says so.
        let vanished = training_set_drift(&conn, Some(&roster(Some("no-such-set"), Some(1))));
        assert_eq!(vanished.len(), 1);
        assert!(vanished[0].contains("no longer exists"), "{}", vanished[0]);

        // A model fitted from the CURRENT store has no set to check, and must not be warned about:
        // it never claimed to come from a frozen set, and inventing a warning would train the reader
        // to ignore the ones that mean something.
        assert!(training_set_drift(&conn, Some(&roster(None, None))).is_empty());
        assert!(training_set_drift(&conn, None).is_empty(), "a model saved before the record existed");
    }

    /// **SB-MLA-004 — the mask's effect is recorded per well, and never confused with a missing
    /// curve.**
    ///
    /// A single "rows not used" count would satisfy a careless reading of the requirement and be
    /// useless: masked rows and incomplete rows call for OPPOSITE fixes — widen the mask, or go and
    /// find the missing curve. So both are counted, and the test drives one well of each kind plus
    /// one that is both, and checks the two numbers never borrow from each other.
    #[test]
    fn the_mask_effect_is_recorded_per_well_and_is_never_confused_with_a_missing_curve() {
        use crate::db;
        use uuid::Uuid;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 10usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let gr: Vec<f32> = (0..n).map(|i| 20.0 + i as f32).collect();

        // One well with a complete frame, and a BADHOLE flag over its first three depths.
        let a = Uuid::new_v4();
        db::insert_well(&conn, a, "SANDI-1", None, None, Some(0.0)).unwrap();
        let mut rhob: Vec<f32> = vec![2.3; n];
        // ... and two depths where the TARGET was never measured. Nothing to do with the mask.
        rhob[8] = f32::NAN;
        rhob[9] = f32::NAN;
        db::insert_standard_curves(
            &conn, a, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n], rhob,
            vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let flag: Vec<f32> = (0..n).map(|i| if i < 3 { 1.0 } else { 0.0 }).collect();
        crate::equations::write_computed_curve(&conn, &a.to_string(), &depths, "BADHOLE", &flag).unwrap();

        let features = vec!["GR".to_string()];
        let ids = vec![a.to_string()];
        let (_x, y, _g, empty, roster) =
            assemble_training(&conn, &ids, &features, "RHOB", Some(&"BADHOLE".to_string()), None, DepthWindow::default());

        assert!(empty.is_empty(), "the well contributed, so it is not an empty well");
        assert_eq!(y.len(), 5, "10 depths, 3 masked, 2 with no target");
        let r = &roster[0];
        assert_eq!(r.rows, 5);
        assert_eq!(r.masked, 3, "exactly the flagged depths");
        assert_eq!(r.incomplete, 2, "the unmeasured target is NOT the mask's doing");
        assert_eq!(r.rows + r.masked + r.incomplete, n, "every depth is accounted for");
        assert_eq!(r.well, "SANDI-1", "a UUID in a provenance record is not actionable");

        // The other side: the same well with no mask loses nothing to one.
        let (_x2, y2, _g2, _e2, roster2) =
            assemble_training(&conn, &ids, &features, "RHOB", None, None, DepthWindow::default());
        assert_eq!(y2.len(), 8, "the three flagged depths are ordinary rows without a mask");
        assert_eq!(roster2[0].masked, 0, "no mask, nothing attributed to one");
        assert_eq!(roster2[0].incomplete, 2);

        // And the half the counts alone cannot carry: WHICH flag this was. A model recording only
        // "3 samples excluded" cannot be re-run, because the next analyst has no way to know whether
        // that was BADHOLE, a coal flag or a hand-drawn interval — and the requirement asks for the
        // curve "or its explicit absence", which are two different facts rather than a value and a
        // blank. `null` here is the second of them, not a missing field.
        let named = serde_json::to_string(&TrainingRecord {
            mask_curve: Some("BADHOLE".into()),
            wells: roster.clone(),
        })
        .unwrap();
        let back: TrainingRecord = serde_json::from_str(&named).unwrap();
        assert_eq!(back.mask_curve.as_deref(), Some("BADHOLE"));
        assert_eq!(back.wells[0].masked, 3, "the roster survives the wrapper");

        let unmasked =
            serde_json::to_string(&TrainingRecord { mask_curve: None, wells: roster2 }).unwrap();
        let back2: TrainingRecord = serde_json::from_str(&unmasked).unwrap();
        assert!(back2.mask_curve.is_none(), "no mask is recorded as no mask");
        assert!(
            unmasked.contains("\"mask_curve\":null"),
            "written explicitly, so a reader can tell it from a field that was never set: {unmasked}"
        );
    }

    /// A saved model with the boring fields filled in. Deliberately NOT a `Default` impl on
    /// `NewMlModel`: requiring every field at the real call site is the property that makes the
    /// struct worth having, and a `..Default::default()` would hand the omission back.
    fn model_fixture<'a>(
        name: &'a str,
        feature_curves: &'a [String],
        trained_on: &'a [String],
        data: &'a [u8],
        train_hash: Option<&'a str>,
    ) -> crate::db::NewMlModel<'a> {
        crate::db::NewMlModel {
            name,
            task: "regression",
            algorithm: "rf",
            feature_curves,
            target_curve: Some("PERM"),
            params_json: "{}",
            metrics_json: "{}",
            trained_on,
            n_train: 100,
            standardize: true,
            note: None,
            data,
            train_hash,
            training_json: None,
            runtime_json: None,
            sklearn_version: Some("1.5.0"),
        }
    }

    #[test]
    fn a_retrained_model_never_overwrites_the_one_a_delivered_curve_was_made_with() {
        use crate::db;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let feats = vec!["GR".to_string()];
        let blob = vec![1u8, 2, 3];
        let one: Vec<String> = vec!["A".into()];
        let two: Vec<String> = vec!["A".into(), "B".into()];
        let (_, first) = db::insert_ml_model(&conn, &model_fixture("PERM_RF", &feats, &one, &blob, Some("aaaa"))).unwrap();
        let (_, second) = db::insert_ml_model(&conn, &model_fixture("PERM_RF", &feats, &two, &blob, Some("bbbb"))).unwrap();
        assert_eq!(first, "PERM_RF");
        assert_eq!(second, "PERM_RF_1", "a second fit is a NEW model, not a replacement");
        assert_eq!(db::list_ml_models(&conn).unwrap().len(), 2);
    }

    /// **SB-MLA-007 — a model a delivered curve cites is not deletable without a word, and one
    /// nothing cites is.**
    ///
    /// Deleting a cited model corrupts nothing: the curve keeps its numbers. It does something
    /// quieter and more expensive — the curve goes on naming a model id that resolves to nothing, so
    /// the provenance block in a report names something nobody can produce, and the failure surfaces
    /// in front of a client months later as a question that cannot be answered.
    ///
    /// Pinned from both sides, because the second is what decides whether the first is any use. A
    /// check that flagged every model would be one people learn to click past, so a model nothing
    /// cites must come back clean — and so must one whose log set carries no curves, since a
    /// superseded version is not in the deliverable and protecting it would make the refusal noise.
    #[test]
    fn a_model_a_delivered_curve_cites_is_not_deletable_without_a_word() {
        use crate::db;
        use crate::equations::{create_log_set, write_computed_curves_versioned, LogSetSpec};
        use uuid::Uuid;

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-CITE", None, None, Some(0.0)).unwrap();
        let well_id = well.to_string();

        let feats: Vec<String> = vec!["GR".into()];
        let on: Vec<String> = vec!["SANDI-CITE".into()];
        let blob = vec![1u8, 2, 3];
        let (cited_id, _) = db::insert_ml_model(&conn, &model_fixture("CITED", &feats, &on, &blob, None)).unwrap();
        let (lonely_id, _) = db::insert_ml_model(&conn, &model_fixture("LONELY", &feats, &on, &blob, None)).unwrap();

        // A curve made by CITED, recorded the way the fit path records it.
        let depths: Vec<f32> = (0..20).map(|i| 1000.0 + i as f32).collect();
        let vals: Vec<f32> = (0..20).map(|i| 0.1 + i as f32 * 0.001).collect();
        let spec = LogSetSpec {
            set_name: "ML".into(),
            module: "ml:regression:rf".into(),
            params_json: serde_json::json!({ "algorithm": "rf", "model_id": cited_id, "model_name": "CITED" })
                .to_string(),
            inputs_json: "[\"GR\"]".into(),
        };
        let (set_id, _) = create_log_set(&conn, &well_id, &spec).unwrap();
        write_computed_curves_versioned(&conn, &well_id, &depths, &[("PHIT_ML", vals.as_slice())], &set_id).unwrap();

        // The cited one is found, and the citation says WHERE — a refusal naming no curve is one
        // nobody can act on.
        let cited = model_citations(&conn, &cited_id);
        assert_eq!(cited.len(), 1, "the delivered curve must be found: {cited:?}");
        assert_eq!(cited[0].well_name, "SANDI-CITE");
        assert_eq!(cited[0].set_name, "ML");
        assert_eq!(cited[0].curves, vec!["PHIT_ML".to_string()]);

        // The other side. A model nothing cites is clean, or the check becomes noise.
        assert!(
            model_citations(&conn, &lonely_id).is_empty(),
            "a model no curve names must not be protected, or the refusal is one people click past"
        );

        // A log set with no rows is not in any deliverable. Protecting it would flag a model whose
        // curves were re-run and superseded, which is the commonest case of all.
        let empty_spec = LogSetSpec {
            set_name: "ML".into(),
            module: "ml:regression:rf".into(),
            params_json: serde_json::json!({ "model_id": lonely_id }).to_string(),
            inputs_json: "[]".into(),
        };
        create_log_set(&conn, &well_id, &empty_spec).unwrap();
        assert!(
            model_citations(&conn, &lonely_id).is_empty(),
            "a set carrying no curves is not in a deliverable and must not protect its model"
        );
    }

    /// **SB-MLA-007, second half — a curve whose model was force-deleted says the reference is
    /// unresolvable, and one whose model is still there does not.**
    ///
    /// The refusal can be overridden, so the deleted case is reachable by design. Once it happens the
    /// provenance block is the last line of defence: printing the model NAME alone reads as a live
    /// reference, and a report naming a model nobody can produce asserts an audit trail it cannot
    /// honour — which is the hazard the guard exists for, arriving by the one route the guard allows.
    ///
    /// The second assertion is the load-bearing one. A block that marked every row would tell a
    /// reader nothing, so the live case must come back with the bare name.
    #[test]
    fn a_curve_whose_model_was_deleted_says_so_and_one_whose_model_remains_does_not() {
        use crate::db;
        use crate::equations::{create_log_set, write_computed_curves_versioned, LogSetSpec};
        use uuid::Uuid;

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-GONE", None, None, Some(0.0)).unwrap();
        let well_id = well.to_string();

        let feats: Vec<String> = vec!["GR".into()];
        let on: Vec<String> = vec!["SANDI-GONE".into()];
        let blob = vec![9u8; 8];
        let (doomed, _) = db::insert_ml_model(&conn, &model_fixture("DOOMED", &feats, &on, &blob, None)).unwrap();
        let (kept, _) = db::insert_ml_model(&conn, &model_fixture("KEPT", &feats, &on, &blob, None)).unwrap();

        let depths: Vec<f32> = (0..10).map(|i| 900.0 + i as f32).collect();
        let vals: Vec<f32> = (0..10).map(|i| 0.2 + i as f32 * 0.001).collect();
        for (name, id, curve) in [("ML_A", &doomed, "PERM_A"), ("ML_B", &kept, "PERM_B")] {
            let spec = LogSetSpec {
                set_name: name.into(),
                module: "ml:regression:rf".into(),
                params_json: serde_json::json!({ "model_id": id, "model_name": name }).to_string(),
                inputs_json: "[\"GR\"]".into(),
            };
            let (set_id, _) = create_log_set(&conn, &well_id, &spec).unwrap();
            write_computed_curves_versioned(&conn, &well_id, &depths, &[(curve, vals.as_slice())], &set_id).unwrap();
        }

        // Both models are live: neither row may claim otherwise.
        let before = ml_provenance(&conn, &well_id);
        assert_eq!(before.len(), 2, "both ML log sets carry curves: {before:?}");
        assert!(
            before.iter().all(|r| !r.model.contains("DELETED")),
            "a live model must print as a bare name, or the mark tells a reader nothing: {before:?}"
        );

        db::delete_ml_model(&conn, &doomed).unwrap();

        let after = ml_provenance(&conn, &well_id);
        let gone = after.iter().find(|r| r.curves.contains("PERM_A")).expect("the curve outlives its model");
        let still = after.iter().find(|r| r.curves.contains("PERM_B")).expect("the untouched set is unchanged");
        assert!(
            gone.model.contains("DELETED"),
            "the deliverable must say the reference is unresolvable, not just name it: {}",
            gone.model
        );
        assert!(gone.model.contains("ML_A"), "and must still say WHICH model, or the record is useless");
        assert_eq!(still.model, "ML_B", "the surviving model's row is untouched");
    }

    /// **SB-MLA-017 — a log set written before a cancel says it came from a cancelled run, and one
    /// from a run that finished says nothing.**
    ///
    /// A partially written set is the worst artifact this pane can leave. It is not corrupt and it is
    /// not empty: on the Wells pane it carries the same set name and the same module string a
    /// complete run writes, so it reads as a finished interpretation over a smaller well selection —
    /// and the wells that were cut look like wells somebody chose to exclude.
    ///
    /// The third assertion is the one that would actually bite. The mark shares `params_json` with
    /// the model reference (SB-MLA-006) and the blind record (SB-MLA-009), so a stamp that rebuilt
    /// the object instead of adding to it would erase the provenance it was written to qualify.
    #[test]
    fn a_log_set_written_before_a_cancel_says_so_and_a_completed_one_stays_silent() {
        use crate::db;
        use crate::equations::{create_log_set, LogSetSpec};
        use uuid::Uuid;

        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-CUT", None, None, Some(0.0)).unwrap();
        let well_id = well.to_string();

        let spec = |set: &str| LogSetSpec {
            set_name: set.into(),
            module: "ml:regression:rf".into(),
            params_json: serde_json::json!({
                "algorithm": "rf",
                "model_id": "m-123",
                "model_name": "PERM_RF",
                "blind": { "performed": true, "metric": "r2", "value": 0.81 },
            })
            .to_string(),
            inputs_json: "[\"GR\"]".into(),
        };
        let (cut, _) = create_log_set(&conn, &well_id, &spec("ML_CUT")).unwrap();
        let (whole, _) = create_log_set(&conn, &well_id, &spec("ML_WHOLE")).unwrap();

        // Two of five wells were written before the user pressed Cancel.
        let marked = mark_cancelled_sets(&conn, std::slice::from_ref(&cut), 2, 5);
        assert_eq!(marked, 1, "the set that was written must be marked");

        let read = |set_id: &str| -> serde_json::Value {
            let s: Option<String> = conn
                .query_row("SELECT params_json FROM log_sets WHERE set_id = ?1", duckdb::params![set_id], |r| r.get(0))
                .unwrap();
            serde_json::from_str(&s.unwrap()).unwrap()
        };

        let c = read(&cut);
        assert_eq!(c["cancelled"]["wells_written"], 2);
        assert_eq!(c["cancelled"]["wells_in_scope"], 5);
        assert!(
            c["cancelled"]["note"].as_str().unwrap_or("").contains("cut, not excluded"),
            "the mark is read by a person deciding whether to deliver the curve, so it says so in words: {}",
            c["cancelled"]
        );

        // The mark QUALIFIES the provenance; it must not replace it.
        assert_eq!(c["model_id"], "m-123", "the model reference must survive the stamp");
        assert_eq!(c["blind"]["value"], 0.81, "and so must the blind record");
        assert_eq!(c["algorithm"], "rf");

        // The other side: a run that finished leaves nothing to explain, and a mark on every set
        // would tell a reader nothing.
        let w = read(&whole);
        assert!(w.get("cancelled").is_none(), "a completed run's set must stay silent: {w}");
    }

    /// **SB-MLA-021 — a rejected sample is a class, and it is never drawn as one of the clusters.**
    ///
    /// "This sample is an outlier the model refuses to classify" and "this sample had no RHOB" are
    /// different statements about the rock. Writing both as missing left the class curve unable to
    /// say which, and the aggregate `noise_pct` could only say how much, never where.
    ///
    /// The colour half is the part that would have shipped wrong. Both palette lookups fold an index
    /// back into range with `((i % n) + n) % n`, so `-1` would have painted as a real cluster's
    /// colour — an outlier drawn as a legitimate facies on the log view and in the printed
    /// deliverable, which is worse than the gap it replaced.
    #[test]
    fn a_rejected_sample_is_a_class_of_its_own_and_is_never_coloured_as_a_cluster() {
        // One definition, emitted into the runner rather than written there — the same rule the
        // k-means constants follow, for the same reason: a literal would run and look right.
        let preamble = ml_shared_constants_py();
        assert!(
            preamble.contains(&format!("CLUSTER_REJECT = {CLUSTER_REJECT}")),
            "the runner preamble must carry the reject code:\n{preamble}"
        );
        assert!(CLUSTER_REJECT < 0, "the reject code must sort below every cluster id");
        assert!(
            ML_RUNNER_BODY.contains("out[labels < 0] = CLUSTER_REJECT"),
            "the runner must WRITE the reject code, not leave the sample missing"
        );
        assert!(
            !ML_RUNNER_BODY.contains("# DBSCAN noise (-1) stays NaN"),
            "the old conflating behaviour must be gone, not merely supplemented"
        );

        // The colour separation. A reject must not collide with ANY cluster colour, including the
        // one the modulo wrap would have handed it.
        let reject = crate::composite::facies_color_for_test(CLUSTER_REJECT);
        for c in 0..24i64 {
            assert_ne!(
                crate::composite::facies_color_for_test(c),
                reject,
                "cluster {c} shares the reject colour, so an outlier prints as real rock"
            );
        }
        // Any negative, not only this code: an unrecognised class must not be painted as rock.
        assert_eq!(crate::composite::facies_color_for_test(-7), reject);
    }

    /// **Round-3 item 5 — the textured prediction is a SECOND curve, named for what it is, and it is
    /// never what you get by default.**
    ///
    /// Jauhar asked for two versions. The reason two names matter rather than two dialog settings is
    /// that the dialog is seen once and the curve is read for years: a smooth prediction and one
    /// carrying simulated detail plot identically to a reader who was not told which is which, and
    /// the simulated one looks BETTER — more detailed, more like a real log — which is exactly
    /// backwards from how much it can be trusted.
    ///
    /// Under a transform the texturing is of the model's own log-space output, so it must be named
    /// after that curve. `PERM_SIM` beside a `PERM` in millidarcies would be read as millidarcies and
    /// plot orders of magnitude out.
    #[test]
    fn a_spectrally_textured_prediction_is_a_second_named_curve_and_never_the_default() {
        // Named for the simulation, not the method: what matters to whoever picks the curve up is
        // that the detail was invented, not that a Fourier transform was involved.
        assert_eq!(out_name_for("PERM", SIM_SUFFIX, ""), "PERM_SIM");
        assert_eq!(out_name_for("PERM", "", ""), "PERM", "the plain prediction keeps the base name");
        assert_eq!(
            out_name_for("PERM", SIM_SUFFIX, "log10"),
            "PERM_LOG10_SIM",
            "under a transform the texturing is OF the log-space curve and must say so"
        );

        // Same space as the curve it was made from, so an export cannot put an undeclared log-space
        // curve beside one in millidarcies.
        assert_eq!(unit_for_output(SIM_SUFFIX, "", Some("mD")).as_deref(), Some("mD"));
        assert_eq!(
            unit_for_output(SIM_SUFFIX, "log10", Some("mD")),
            unit_for_output("", "log10", Some("mD")),
            "the textured curve carries whatever unit the model's own output carries"
        );

        // One spelling, emitted, for the same reason the reject code is.
        assert!(ml_shared_constants_py().contains(&format!("SIM_SUFFIX = \"{SIM_SUFFIX}\"")));
        assert!(
            ML_RUNNER_BODY.contains("outs.append((SIM_SUFFIX, sim))"),
            "the runner must emit under the shared name, not a literal it could misspell"
        );

        // OFF unless asked for. The plain prediction is the defensible curve; a textured one that
        // arrived unrequested would be quoted as a measurement by somebody who never chose it.
        assert!(
            ML_RUNNER_BODY.contains("P(p, \"spectral_texture\", False)"),
            "spectral texture must default to off, and go through P so the record says it was off"
        );
        // The added detail must never move the answer the plain curve already gives.
        assert!(
            ML_RUNNER_BODY.contains("deficit[0] = 0.0"),
            "the DC term must be forced to zero or the textured curve shifts the mean"
        );
        assert!(
            ML_RUNNER_BODY.contains("np.maximum(0.0, want ** 2 - have ** 2)"),
            "only the DEFICIT is added, so a prediction already as rough as its target is left alone"
        );
        // ...and the deficit is measured against a SMOOTHED periodogram. Without this the estimator
        // is inconsistent, half its bins read low by chance, and rectifying at zero turns every one
        // of those into energy added to a curve that did not need it — measured: a prediction
        // already at its target's resolution came back rougher than the log it was matched to.
        assert!(
            ML_RUNNER_BODY.contains("smooth_band(np.abs(np.fft.rfft(centred)) ** 2, SPEC_SMOOTH)"),
            "the segment's spectrum must be smoothed, or the deficit is measured against noise"
        );
    }

    /// **A model may not be fitted against a curve carrying simulated detail.**
    ///
    /// This is the failure the two-curve design exists to prevent, and it is the one a naming
    /// convention does NOT prevent on its own: the input list offers every curve in the well, and
    /// `PERM_SIM` sorts directly beside `PERM`. Tick the wrong one and the model learns invented
    /// high-frequency detail, reports the usual scores for it, and the provenance records only that
    /// a curve named `PERM_SIM` was an input — true, and silent about what it means.
    ///
    /// Pinned from both sides. A refusal that also caught ordinary curves would be one people route
    /// around, and `SIMPSON` or `MAX_SIM_1` must go through: the rule is a SUFFIX, not a substring.
    #[test]
    fn a_model_may_not_be_fitted_against_a_curve_carrying_simulated_detail() {
        let sim = |v: &[&str], t: Option<&str>| {
            refuse_simulated_inputs(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>(), t)
        };

        // As a feature, as a target, and named in the refusal so the user knows which to swap.
        let f = sim(&["GR", "perm_sim"], None).expect("a simulated feature is refused");
        assert!(f.contains("PERM_SIM"), "the refusal must name the offender: {f}");
        assert!(f.contains("without _SIM"), "and must say what to use instead: {f}");
        assert!(sim(&["GR"], Some("PHIE_SIM")).is_some(), "a simulated TARGET is refused too");

        // Both offenders named, once each, rather than only the first.
        let both = sim(&["A_SIM", "GR", "A_SIM", "B_SIM"], None).unwrap();
        assert!(both.contains("A_SIM") && both.contains("B_SIM"), "{both}");
        assert_eq!(both.matches("A_SIM").count(), 1, "deduped: {both}");

        // The other side. A refusal that fired on ordinary curves would be routed around, and the
        // rule is a suffix — a curve that merely CONTAINS the letters must run.
        assert!(sim(&["GR", "RHOB", "NPHI"], Some("PERM")).is_none());
        assert!(sim(&["SIMPSON", "MAX_SIM_1", "SIMILARITY"], Some("PERM")).is_none());
        assert!(sim(&[], None).is_none());
    }

    /// **The runner is launched from a FILE, so its length is not a cliff — and a temp file cannot
    /// outlive the run.**
    ///
    /// This is a scar, not a precaution. `python -c <source>` was the launch mechanism, Windows caps
    /// a command line near 32 KB, and the runner crossed it — every ML feature failed at once with
    /// `The filename or extension is too long. (os error 206)`, a message naming neither Python nor
    /// machine learning. The trigger was added comments. Nothing guarded it, so the ceiling was
    /// invisible until it was a total outage, and the natural "fix" under it — delete some comments —
    /// would have restored service while leaving the cliff exactly where it was.
    ///
    /// So the assertion is not "the runner is short enough". It is that **no runner is launched with
    /// `-c` at all**, which is the property that has no ceiling. The runners are deliberately checked
    /// against the old limit too, purely to record that they are past it: this is not a hypothetical.
    #[test]
    fn a_runner_is_launched_from_a_file_so_its_length_is_not_a_cliff() {
        const CMDLINE_CEILING: usize = 32_767;

        // The evidence. If these ever fall back under the ceiling the test still passes — it is the
        // launch mechanism that matters — but the numbers are here so nobody re-litigates this.
        let sizes = [("fit", ml_runner().len()), ("apply", ml_apply_runner().len()), ("eval", ml_eval_runner().len())];
        assert!(
            sizes.iter().any(|(_, n)| *n > CMDLINE_CEILING),
            "a runner used to exceed {CMDLINE_CEILING} chars and that is why this exists: {sizes:?}"
        );

        // The property that actually holds: no runner rides on the command line.
        let src = include_str!("ml.rs");
        for bad in ["-c\", &ml_runner()", "-c\", &ml_apply_runner()", "-c\", &ml_eval_runner()"] {
            assert!(
                !src.contains(bad),
                "a runner is being passed with -c again, which reinstates a ~{CMDLINE_CEILING}-char \
                 cliff that fails with an error naming neither Python nor ML: {bad}"
            );
        }

        // And the file does not outlive the run — a fit per well across a field would otherwise
        // leave one temp file per run behind for the session.
        let path = {
            let s = ScriptFile::new("selftest", "print('hi')\n").expect("temp script");
            assert!(s.path().exists(), "the script must exist while the run holds it");
            s.path().to_path_buf()
        };
        assert!(!path.exists(), "the temp script must be removed when the run drops it");
    }

    /// **SB-MLA-022 — the ordered-feature contract is checked on the DEFAULT gate.**
    ///
    /// The runtime refusal lives inside the joblib artifact and needs a real interpreter, so the test
    /// that exercises it is `#[ignore]`d and legitimately so. That left one of the four things this
    /// tree does better than any incumbent resting on somebody remembering to run the ignored set.
    ///
    /// The gap is closable without scikit-learn because the strongest guarantee here is STRUCTURAL,
    /// not behavioural: an apply request has no feature list at all, so a caller cannot state an
    /// order to get wrong. `apply_ml_model` drives the fetch from the artifact's own list. A refusal
    /// catches a bad order; having nowhere to express one means it cannot arise from this product at
    /// all, and that property is checkable here.
    #[test]
    fn an_apply_request_cannot_state_a_feature_order_for_the_model_to_refuse() {
        // An EXHAUSTIVE struct literal, deliberately. If a `feature_curves` field is ever added to
        // the apply request this stops COMPILING with "missing field", which is a stronger and
        // earlier guard than any runtime assertion — and the reason is here for whoever hits it: a
        // caller-supplied order is a second place to state the feature list, therefore a second
        // place to get it wrong, and a model fed [RHOB, GR] where it was fitted on [GR, RHOB]
        // returns confident nonsense that nothing downstream can catch.
        let _shape = MlApplyRequest {
            input_set: None,
            interval: DepthWindow::default(),
            output_set: None,
            model_id: "m-1".into(),
            apply_well_ids: vec!["w-1".into()],
            output_curve: "PERM_ML".into(),
            mask_curve: None,
        };

        // And a feature list offered over IPC is ignored rather than honoured — an older or
        // hand-built caller cannot smuggle one in.
        let json = serde_json::json!({
            "model_id": "m-1",
            "apply_well_ids": ["w-1"],
            "output_curve": "PERM_ML",
            "feature_curves": ["RHOB", "GR"],
        });
        let req: MlApplyRequest =
            serde_json::from_value(json).expect("an unknown feature list must be ignored, not fatal");
        assert_eq!(req.model_id, "m-1", "the model id is the only thing that selects features");

        // And the runner still refuses a mismatch, for models applied by anything that is not this
        // request type. Source-level, which is the weaker kind of assertion — the behavioural test
        // is `a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order`, which needs sklearn.
        let src = ml_apply_runner();
        assert!(
            src.contains("features") && (src.contains("!=") || src.contains("fail(")),
            "the apply runner must still check the artifact's feature list"
        );
    }

    #[test]
    fn listing_models_never_carries_their_bytes() {
        use crate::db;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let blob = vec![7u8; 4096];
        let feats: Vec<String> = vec!["GR".into()];
        let on: Vec<String> = vec!["A".into()];
        let (id, _) = db::insert_ml_model(&conn, &model_fixture("M", &feats, &on, &blob, None)).unwrap();
        let listed = db::list_ml_models(&conn).unwrap();
        assert_eq!(listed[0].bytes, 4096, "the size is reported so the picker can show it");
        assert_eq!(listed[0].feature_curves, vec!["GR".to_string()]);
        let (_, bytes) = db::get_ml_model(&conn, &id).unwrap();
        assert_eq!(bytes.len(), 4096, "only the apply path fetches the model itself");
    }

    /// End to end: fit on the cored well SAVING the model, then apply that same model to a well
    /// it has never seen — no refit — and check it reproduces the relation. Needs sklearn+joblib.
    #[test]
    #[ignore]
    fn a_saved_model_applies_to_an_unseen_well_without_refitting() {
        let (dbm, cored, blind) = two_well_db();
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }
        let mut req = mk_req("regression", &["GR", "RHOB"], Some("PHIT_CORE"), &[cored.clone()], &[cored.clone()]);
        req.output_curve = "PHIT_ML".into();
        req.save_model_as = Some("PHIT_FROM_CORE".into());
        let fit = run_ml(&dbm, &req, None);
        assert!(fit.error.is_none(), "fit failed: {:?}", fit.error);
        let model_id = fit.model_id.clone().expect("the model was kept");
        assert_eq!(fit.model_name.as_deref(), Some("PHIT_FROM_CORE"));

        // The blind well has no target at all — only a saved model can give it one.
        let applied = apply_ml_model(
            &dbm,
            &MlApplyRequest {
                input_set: None,
                output_set: None,
                model_id,
                apply_well_ids: vec![blind.clone()],
                output_curve: "PHIT_ML".into(),
                mask_curve: None,
                interval: DepthWindow::default(),
            },
            None,
        );
        assert!(applied.error.is_none(), "apply failed: {:?}", applied.error);
        assert_eq!(applied.wells.len(), 1);
        assert!(applied.wells[0].error.is_none(), "{:?}", applied.wells[0].error);
        assert_eq!(applied.wells[0].rows_predicted, 40);

        // The prediction must reproduce the relation the model was fitted on.
        let conn = dbm.lock().unwrap();
        let (_, cols) = crate::equations::fetch_curve_frame(&conn, &blind, &["GR".into(), "RHOB".into(), "PHIT_ML".into()]).unwrap();
        let gr = cols.get("GR").unwrap();
        let rhob = cols.get("RHOB").unwrap();
        let pred = cols.get("PHIT_ML").unwrap();
        for i in 0..gr.len() {
            let want = 0.4 - 0.002 * gr[i] + 0.05 * rhob[i];
            assert!((pred[i] - want).abs() < 1e-3, "row {i}: predicted {} want {want}", pred[i]);
        }
    }

    /// **End to end, the whole round-2/round-3 workflow in one run: bound the fit to an interval,
    /// fit a model per available-input pattern, write at a stated resolution, save the artifact, and
    /// propagate it to an unseen well over a DIFFERENT interval.**
    ///
    /// Each of those has its own unit test. This one exists because they interact, and the
    /// interactions are where the silent failures live: an interval applied to the fit but not to
    /// the write, a coverage segment that ignores the interval, a block that spans the interval
    /// edge, a distribution that inherits the fit's window instead of taking its own. Every one of
    /// those produces a full, plausible curve.
    ///
    /// Needs sklearn + joblib, so it is `#[ignore]`d and the green gate can never depend on it.
    #[test]
    #[ignore]
    fn the_whole_ml_workflow_holds_together_from_bounded_fit_to_distribution() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 240usize;
        // RES_DEEP stops half way down, so the deep half can only be predicted without it - the
        // coverage case. Depths run 1000..1239 so the interval below cuts a real boundary.
        let mk = |name: &str, with_target: bool| -> String {
            let id = Uuid::new_v4();
            db::insert_well(&conn, id, name, None, None, Some(0.0)).unwrap();
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
            let gr: Vec<f32> = (0..n).map(|i| 20.0 + (i % 53) as f32).collect();
            let rhob: Vec<f32> = (0..n).map(|i| 2.0 + (i % 7) as f32 * 0.05).collect();
            let rt: Vec<f32> = (0..n)
                .map(|i| if i >= n / 2 { f32::NAN } else { 5.0 + (i % 11) as f32 })
                .collect();
            db::insert_standard_curves(
                &conn, id, depths.clone(), gr.clone(), rt.clone(), vec![f32::NAN; n],
                rhob.clone(), vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();
            if with_target {
                let t: Vec<f32> = (0..n).map(|i| 0.4 - 0.002 * gr[i] + 0.05 * rhob[i]).collect();
                crate::equations::write_computed_curve(&conn, &id.to_string(), &depths, "PHIT_CORE", &t).unwrap();
            }
            id.to_string()
        };
        let cored = mk("SANDI-E2E-1", true);
        let blind = mk("SANDI-E2E-2", false);
        let dbm = std::sync::Mutex::new(conn);

        // --- 1. Bounded, segmented, blocked fit -------------------------------
        const TOP: f64 = 1020.0;
        const BASE: f64 = 1200.0;
        let mut req = mk_req(
            "regression",
            &["GR", "RHOB", "RES_DEEP"],
            Some("PHIT_CORE"),
            &[cored.clone()],
            &[cored.clone()],
        );
        req.output_curve = "PHIT_ML".into();
        req.save_model_as = Some("PHIT_E2E".into());
        req.coverage_segments = true;
        req.output_step = Some(4.0);
        req.interval = DepthWindow { top: Some(TOP), base: Some(BASE) };
        let fit = run_ml(&dbm, &req, None);
        assert!(fit.error.is_none(), "fit failed: {:?}", fit.error);
        assert!(fit.wells[0].error.is_none(), "{:?}", fit.wells[0].error);

        // Two segments: one where RES_DEEP exists, one where it does not. Neither averaged.
        let segs = fit.metrics.get("coverage_segments").and_then(|v| v.as_array()).expect("segments reported");
        let fitted: Vec<&serde_json::Value> =
            segs.iter().filter(|s| s.get("skipped").is_some_and(|x| x.is_null())).collect();
        assert_eq!(fitted.len(), 2, "one model with RES_DEEP, one without: {segs:#?}");
        let widths: Vec<usize> =
            fitted.iter().map(|s| s["features"].as_array().unwrap().len()).collect();
        assert!(widths.contains(&3) && widths.contains(&2), "expected a 3-curve and a 2-curve model, got {widths:?}");

        // --- 2. The interval bounded the WRITE, not only the fit ---------------
        {
            let conn = dbm.lock().unwrap();
            let (depth, cols) =
                crate::equations::fetch_curve_frame(&conn, &cored, &["PHIT_ML".into()]).unwrap();
            let pred = cols.get("PHIT_ML").unwrap();
            let mut inside = 0usize;
            for i in 0..depth.len() {
                let d = depth[i] as f64;
                if d < TOP || d >= BASE {
                    assert!(
                        !pred[i].is_finite(),
                        "depth {d} is outside {TOP}..{BASE} and must be MISSING, got {}",
                        pred[i]
                    );
                } else if pred[i].is_finite() {
                    inside += 1;
                }
            }
            assert!(inside > 100, "the interval should carry most of its depths, got {inside}");

            // --- 3. The block held one value across each 4 m interval ----------
            // Anchored on an absolute grid, so blocks are [1020,1024), [1024,1028)... Neighbours
            // inside one block must be identical; a block boundary is where a change is allowed.
            let mut same_within = 0usize;
            for i in 1..depth.len() {
                if !pred[i].is_finite() || !pred[i - 1].is_finite() {
                    continue;
                }
                let (a, b) = ((depth[i - 1] as f64 / 4.0).floor(), (depth[i] as f64 / 4.0).floor());
                if a == b {
                    assert!(
                        (pred[i] - pred[i - 1]).abs() < 1e-6,
                        "depths {} and {} are in one 4 m block and must hold one value",
                        depth[i - 1], depth[i]
                    );
                    same_within += 1;
                }
            }
            assert!(same_within > 50, "the run should have produced many within-block pairs, got {same_within}");
        }

        // --- 4. The artifacts, and what they carry -----------------------------
        let models = {
            let conn = dbm.lock().unwrap();
            db::list_ml_models(&conn).unwrap()
        };
        assert_eq!(
            models.len(),
            2,
            "one saved model per fitted segment: {:?}",
            models.iter().map(|m| m.name.clone()).collect::<Vec<_>>()
        );
        let three = models
            .iter()
            .find(|m| m.feature_curves.len() == 3)
            .expect("the three-curve segment kept a model");
        assert!(three.name.contains("3CURVE"), "a saved model names the curves it needs: {}", three.name);

        // --- 5. Distribute to an unseen well, over its OWN interval ------------
        // Deliberately a different window: the apply path must NOT inherit the fit's.
        const DTOP: f64 = 1100.0;
        let applied = apply_ml_model(
            &dbm,
            &MlApplyRequest {
                input_set: None,
                output_set: None,
                model_id: three.model_id.clone(),
                apply_well_ids: vec![blind.clone()],
                output_curve: "PHIT_DIST".into(),
                mask_curve: None,
                interval: DepthWindow { top: Some(DTOP), base: None },
            },
            None,
        );
        assert!(applied.error.is_none(), "distribution failed: {:?}", applied.error);
        assert!(applied.wells[0].error.is_none(), "{:?}", applied.wells[0].error);

        let conn = dbm.lock().unwrap();
        let (depth, cols) = crate::equations::fetch_curve_frame(
            &conn,
            &blind,
            &["GR".into(), "RHOB".into(), "PHIT_DIST".into()],
        )
        .unwrap();
        let gr = cols.get("GR").unwrap();
        let rhob = cols.get("RHOB").unwrap();
        let pred = cols.get("PHIT_DIST").unwrap();
        let mut checked = 0usize;
        for i in 0..depth.len() {
            let d = depth[i] as f64;
            if d < DTOP {
                assert!(!pred[i].is_finite(), "depth {d} is above the distribution top and must be MISSING");
                continue;
            }
            // The three-curve model needs RES_DEEP, which this well lacks below the halfway point,
            // so a missing prediction there is correct - the model was never fed, not misapplied.
            if !pred[i].is_finite() {
                continue;
            }
            let want = 0.4 - 0.002 * gr[i] + 0.05 * rhob[i];
            assert!(
                (pred[i] - want).abs() < 5e-3,
                "row {i} at {d}: distributed {} want {want}",
                pred[i]
            );
            checked += 1;
        }
        // Exactly 20, and the number is the whole point of the case. The distribution starts at
        // 1100; RES_DEEP, which this model needs, runs out after 1119. So the three-curve model can
        // answer 1100..1119 and nothing else — bounded above by the interval the user chose and
        // below by the curve the model requires. A looser floor here would pass just as happily if
        // the interval were being ignored and the whole well came back predicted.
        assert_eq!(
            checked, 20,
            "the distribution is bounded above by its own interval (1100) and below by where \
             RES_DEEP stops (1119), so it must answer exactly those 20 depths"
        );
    }

    /// The ordering contract. A model fitted on [GR, RHOB] fed a matrix ordered [RHOB, GR] would
    /// produce confident nonsense that nothing downstream can catch, so the artifact carries its
    /// own feature list and REFUSES. This drives the runner directly, because the Rust side makes
    /// the mistake impossible by construction — which is exactly why the runner must still check.
    #[test]
    #[ignore]
    fn a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order() {
        use std::io::Write as _;
        let (dbm, cored, _blind) = two_well_db();
        if find_python().is_none() {
            eprintln!("skipping: no python+numpy");
            return;
        }
        let mut req = mk_req("regression", &["GR", "RHOB"], Some("PHIT_CORE"), &[cored.clone()], &[cored.clone()]);
        req.output_curve = "PHIT_ML".into();
        req.save_model_as = Some("ORDER_GUARD".into());
        let fit = run_ml(&dbm, &req, None);
        let model_id = fit.model_id.clone().expect("model saved");
        let (_, blob) = {
            let conn = dbm.lock().unwrap();
            crate::db::get_ml_model(&conn, &model_id).unwrap()
        };

        let python = find_python().unwrap();
        let x: Vec<f32> = vec![30.0, 2.1, 40.0, 2.2];
        let header = serde_json::json!({
            "d": 2, "n_apply": 2, "model_len": blob.len(), "features": ["RHOB", "GR"],
        });
        let script = ScriptFile::new("apply-test", &ml_apply_runner()).unwrap();
        let mut cmd = Command::new(&python);
        cmd.arg(script.path()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        hide_console(&mut cmd);
        let mut child = cmd.spawn().unwrap();
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(header.to_string().as_bytes()).unwrap();
            stdin.write_all(b"\n").unwrap();
            stdin.write_all(&blob).unwrap();
            stdin.write_all(bytemuck::cast_slice(&x)).unwrap();
        }
        let out = child.wait_with_output().unwrap();
        assert!(!out.status.success(), "a reordered matrix must be refused, not predicted");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("fitted on"), "the refusal names the fitted order: {err}");
    }

    /// **SB-MLA-008 — the same run twice is the same curves, byte for byte, for every algorithm.**
    ///
    /// The dossier's finding is that no incumbent offers this: an unseeded k-means at K = 15 over a
    /// pooled five-well set returns different cluster ids every time, so a facies track in a
    /// delivered report cannot be reproduced. Every algorithm here goes through `P(p, "seed", …)`,
    /// which means a seed is always on the record even when nobody typed one — but that is the
    /// mechanism, not the evidence. This is the evidence, and it is measured rather than asserted.
    ///
    /// **Compared on the BITS, not the values.** A tolerance would hide exactly the drift this
    /// exists to catch, and `f32::NAN != f32::NAN` under `==` — a run that turned a cluster into
    /// noise on the second pass would slip through a value comparison as "both NaN, both fine".
    /// `to_bits` makes a missing sample compare equal to a missing sample and unequal to anything
    /// else, which is the byte-identity the requirement asks for.
    ///
    /// The metrics are compared too, because the requirement says "every reported metric" — a
    /// silhouette or an explained-variance vector that moved is the same instability arriving by a
    /// different door, and it is the number that ends up in a report's methodology table.
    ///
    /// `autoencoder` is excluded because it refuses (PyTorch is not wired up), and `dbscan` because
    /// its parameters are data-scaled — the fixture's spread would return a single cluster and prove
    /// nothing about determinism.
    ///
    /// **What this test proves depends on what is installed, and that is the point rather than a
    /// weakness.** `gbdt` fits `XGBRegressor` where `xgboost` is present and scikit-learn's
    /// `HistGradientBoosting` where it is not, so the case that passes here is whichever estimator
    /// this machine actually runs — on the reference machine, with no `xgboost`, that is the
    /// substitute. A test asserting determinism for an estimator that was never executed would be
    /// the kind of guarantee SB-MLA-008's escape clause exists to prevent. `determinism_note` says
    /// the same thing to the user before the run. Needs sklearn.
    #[test]
    #[ignore]
    fn the_same_run_twice_produces_byte_identical_curves_for_every_algorithm() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // Four separable blobs in two dimensions, with a target that is a genuine function of both:
        // enough structure for clustering to find groups and for a regressor to have something to
        // learn, and small enough that sixteen fits run in seconds.
        let n = 200usize;
        let d = 2usize;
        let mut x: Vec<f32> = Vec::with_capacity(n * d);
        let mut y_reg: Vec<f32> = Vec::with_capacity(n);
        let mut y_cls: Vec<f32> = Vec::with_capacity(n);
        for i in 0..n {
            let blob = i % 4;
            let t = (i / 4) as f32;
            let a = blob as f32 * 30.0 + (t * 0.37).sin() * 4.0;
            let b = blob as f32 * 0.4 + (t * 0.71).cos() * 0.05;
            x.push(a);
            x.push(b);
            y_reg.push(0.4 - 0.002 * a + 0.05 * b);
            y_cls.push(blob as f32);
        }

        let cases: &[(&str, &str, Option<&[f32]>)] = &[
            ("regression", "linear", Some(&[])),
            ("regression", "rf", Some(&[])),
            ("regression", "gbdt", Some(&[])),
            ("regression", "svr", Some(&[])),
            ("regression", "ann", Some(&[])),
            ("classification", "rf", Some(&[])),
            ("classification", "knn", Some(&[])),
            ("classification", "svm", Some(&[])),
            ("classification", "gnb", Some(&[])),
            ("classification", "logreg", Some(&[])),
            ("clustering", "kmeans", None),
            ("clustering", "gmm", None),
            ("clustering", "hier", None),
            ("reduction", "pca", None),
            ("reduction", "tsne", None),
        ];

        for (task, algo, supervised) in cases {
            let target: Option<&[f32]> = supervised.map(|_| {
                if *task == "classification" { &y_cls[..] } else { &y_reg[..] }
            });
            let params = serde_json::Map::new();
            let once = exec_ml(&py, task, algo, &params, d, &x, target, &x, n)
                .unwrap_or_else(|e| panic!("{task}/{algo} failed: {e}"));
            let twice = exec_ml(&py, task, algo, &params, d, &x, target, &x, n)
                .unwrap_or_else(|e| panic!("{task}/{algo} failed on the second run: {e}"));

            assert_eq!(
                once.1.len(),
                twice.1.len(),
                "{task}/{algo}: the two runs produced different numbers of curves",
            );
            for ((s1, v1), (s2, v2)) in once.1.iter().zip(twice.1.iter()) {
                assert_eq!(s1, s2, "{task}/{algo}: the curve suffixes came back in a different order");
                assert_eq!(v1.len(), v2.len(), "{task}/{algo}{s1}: different sample counts");
                let differing = v1
                    .iter()
                    .zip(v2.iter())
                    .enumerate()
                    .find(|(_, (a, b))| a.to_bits() != b.to_bits());
                assert!(
                    differing.is_none(),
                    "{task}/{algo}{s1}: sample {} is {:?} then {:?} - not byte-identical",
                    differing.unwrap().0,
                    differing.unwrap().1 .0,
                    differing.unwrap().1 .1,
                );
            }

            // "every reported metric", minus the two that describe the RUN rather than the answer:
            // the effective-parameter record and the runtime both legitimately restate themselves,
            // and neither is a number anybody would notice moving.
            let strip = |m: &serde_json::Value| {
                let mut m = m.clone();
                if let Some(o) = m.as_object_mut() {
                    o.remove("runtime");
                }
                m
            };
            assert_eq!(
                strip(&once.0),
                strip(&twice.0),
                "{task}/{algo}: the metrics moved between two identical runs",
            );
        }
    }

    /// MASK excludes flagged rows from the FIT: a training well whose one flagged row carries a wild
    /// outlier target must not bend the regression — the masked fit recovers the clean line.
    #[test]
    fn run_ml_mask_excludes_training_outlier() {
        let Some(_py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 13usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let gr: Vec<f32> = (0..n).map(|i| 10.0 + i as f32).collect();
        let mut rhob: Vec<f32> = gr.iter().map(|g| 2.0 * g + 1.0).collect();
        rhob[n - 1] = 9999.0; // wild outlier target on the last row
        let mut mask = vec![0.0f32; n];
        mask[n - 1] = 1.0; // flag exactly that row

        let train = Uuid::new_v4();
        db::insert_well(&conn, train, "TR-M", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, train, depths.clone(), gr.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], rhob, vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        crate::equations::write_computed_curve(&conn, &train.to_string(), &depths, "MASK", &mask).unwrap();
        let apply = Uuid::new_v4();
        db::insert_well(&conn, apply, "AP-M", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, apply, depths.clone(), gr,
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();

        let (tr, ap) = (train.to_string(), apply.to_string());
        let db = Mutex::new(conn);
        let mut req = mk_req("regression", &["GR"], Some("RHOB"), &[tr], &[ap]);
        req.mask_curve = Some("MASK".into());
        let r = run_ml(&db, &req, None);
        assert!(r.error.is_none(), "masked regression should run: {:?}", r.error);
        let r2 = r.metrics.get("r2_train").and_then(|v| v.as_f64()).unwrap_or(0.0);
        assert!(r2 > 0.999, "outlier masked out of the fit → clean line; r2_train = {r2}");
    }

    /// A training well whose target curve is absent (a wrong mnemonic, or an older well) must be
    /// REPORTED as contributing zero samples, not silently pooled away — the pure core of the
    /// honesty fix, exercised without python. `fetch_curve_frame` hands back an all-NaN column
    /// for the missing target, so the well reads as "no usable samples", and the run must not
    /// present a 2-well selection as if both trained the model.
    #[test]
    fn assemble_training_flags_wells_with_no_target() {
        use crate::db;
        use duckdb::Connection;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 20usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let gr: Vec<f32> = (0..n).map(|i| 10.0 + i as f32).collect();
        let rhob: Vec<f32> = gr.iter().map(|g| 2.0 * g + 1.0).collect();

        // GOOD: has both GR (feature) and RHOB (target).
        let good = Uuid::new_v4();
        db::insert_well(&conn, good, "GOOD", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, good, depths.clone(), gr.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], rhob, vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();

        // BAD: has GR but NO RHOB (target all-NaN) — the wrong-target-mnemonic case.
        let bad = Uuid::new_v4();
        db::insert_well(&conn, bad, "BAD", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(
            &conn, bad, depths.clone(), gr,
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();

        let features = vec!["GR".to_string()];
        let ids = vec![good.to_string(), bad.to_string()];
        let (_x, y, groups, empty, roster) =
            assemble_training(&conn, &ids, &features, "RHOB", None, None, DepthWindow::default());
        assert_eq!(groups.len(), y.len(), "every training row carries the well it came from");

        assert_eq!(y.len(), n, "the well with the target contributes all its rows");
        assert_eq!(
            empty,
            vec![bad.to_string()],
            "the target-less well is flagged empty, not silently dropped"
        );
        // SB-MLA-004. The empty well's rows were lost to a MISSING TARGET, not to a mask — and the
        // record says which, because the two call for opposite fixes.
        let bad_rec = roster.iter().find(|r| r.well_id == bad.to_string()).unwrap();
        assert_eq!(bad_rec.rows, 0);
        assert_eq!(bad_rec.masked, 0, "no mask was given, so nothing may be attributed to one");
        assert_eq!(bad_rec.incomplete, n, "the rows are accounted for, not merely absent");
    }

    /// Masking that empties a whole training well must be reported truthfully: the leaderboard
    /// shows the POST-mask contributing-well count and warns that blind-well CV fell back to
    /// random KFold — not the pre-mask request count, which hid the collapse.
    #[test]
    fn run_ml_eval_mask_collapse_is_reported() {
        let Some(_py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        use crate::db;
        use duckdb::Connection;
        use std::sync::Mutex;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let n = 40usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32).collect();
        let gr: Vec<f32> = (0..n).map(|i| 10.0 + (i % 20) as f32).collect();
        let rhob: Vec<f32> = gr.iter().map(|g| 2.0 * g + 1.0).collect();

        // Well A: 40 good rows, mask all 0. Well B: 40 rows, mask ALL 1 (fully excluded).
        let a = Uuid::new_v4();
        db::insert_well(&conn, a, "EV-A", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(&conn, a, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n], rhob.clone(), vec![f32::NAN; n], vec![f32::NAN; n]).unwrap();
        crate::equations::write_computed_curve(&conn, &a.to_string(), &depths, "MASK", &vec![0.0f32; n]).unwrap();
        let b = Uuid::new_v4();
        db::insert_well(&conn, b, "EV-B", None, None, Some(0.0)).unwrap();
        db::insert_standard_curves(&conn, b, depths.clone(), gr.clone(), vec![f32::NAN; n], vec![f32::NAN; n], rhob, vec![f32::NAN; n], vec![f32::NAN; n]).unwrap();
        crate::equations::write_computed_curve(&conn, &b.to_string(), &depths, "MASK", &vec![1.0f32; n]).unwrap();

        let (ida, idb) = (a.to_string(), b.to_string());
        let db = Mutex::new(conn);
        let req = MlEvalRequest {
            input_set: None,
            task: "regression".into(),
            feature_curves: vec!["GR".into()],
            target_curve: "RHOB".into(),
            train_well_ids: vec![ida, idb],
            algorithms: vec!["linear".into()],
            params: Default::default(),
            params_for: None,
            subsets: vec![],
            standardize: false,
            seed: Some(42),
            folds: Some(5),
            mask_curve: Some("MASK".into()),
            target_transform: None,
        };
        let r = run_ml_eval(&db, &req);
        assert!(r.error.is_none(), "eval should run: {:?}", r.error);
        // Only well A contributed after masking → the truthful count is 1, not the requested 2.
        assert_eq!(r.n_groups, 1, "post-mask contributing wells (was reporting the pre-mask 2)");
        assert!(r.cv.to_lowercase().contains("random"), "cv should reflect the KFold fallback: {}", r.cv);
        assert!(
            r.note.as_deref().unwrap_or("").contains("masking"),
            "a masking-collapse note is expected, got {:?}",
            r.note,
        );
    }

    #[test]
    fn classification_knn_labels_blobs_confidently() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // Two tight 2-D blobs labelled 0 / 1; apply points near each centre must get the
        // right label with probability ~1 and a "_PROB" output alongside the class.
        let mut x_train = Vec::new();
        let mut y_train = Vec::new();
        for i in 0..40 {
            let j = (i % 8) as f32 * 0.01;
            x_train.extend_from_slice(&[j, j]);
            y_train.push(0.0);
            x_train.extend_from_slice(&[10.0 + j, 10.0 + j]);
            y_train.push(1.0);
        }
        let x_apply = vec![0.02f32, 0.03, 10.05, 10.02];
        let (_, outs) = exec_ml(
            &py, "classification", "knn", &params(&[("n_neighbors", serde_json::json!(5))]),
            2, &x_train, Some(&y_train), &x_apply, 2,
        )
        .expect("classification run failed");
        assert_eq!(outs.len(), 2);
        assert_eq!(outs[1].0, "_PROB");
        assert_eq!(outs[0].1[0], 0.0);
        assert_eq!(outs[0].1[1], 1.0);
        assert!(outs[1].1[0] > 0.99 && outs[1].1[1] > 0.99);
    }

    #[test]
    fn clustering_kmeans_orders_by_first_feature() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // Blob A (low feature-0) interleaved with blob B (high) — cluster ids must come
        // back 0 for the LOW-mean blob regardless of k-means' internal label order.
        let mut x_apply = Vec::new();
        for i in 0..50 {
            let j = (i % 10) as f32 * 0.02;
            x_apply.extend_from_slice(&[50.0 + j, 1.0 + j]); // high blob first on purpose
            x_apply.extend_from_slice(&[1.0 + j, 50.0 + j]);
        }
        let (metrics, outs) = exec_ml(
            &py, "clustering", "kmeans", &params(&[("k", serde_json::json!(2))]),
            2, &[], None, &x_apply, 100,
        )
        .expect("clustering run failed");
        let labels = &outs[0].1;
        for s in 0..100 {
            let expected = if s % 2 == 0 { 1.0 } else { 0.0 }; // even rows = high blob = class 1
            assert_eq!(labels[s], expected, "sample {s}");
        }
        let sizes = &metrics["cluster_sizes"];
        assert_eq!(sizes["0"].as_i64(), Some(50));
        assert_eq!(sizes["1"].as_i64(), Some(50));
    }

    #[test]
    fn pca_returns_numbered_components() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // 3 features, the third a copy of the first: 2 components must carry ~all variance.
        let mut x_apply = Vec::new();
        for i in 0..200 {
            let a = (i as f32 * 0.37).sin();
            let b = (i as f32 * 0.11).cos();
            x_apply.extend_from_slice(&[a, b, a]);
        }
        let (metrics, outs) = exec_ml(
            &py, "reduction", "pca", &params(&[("n_components", serde_json::json!(2))]),
            3, &[], None, &x_apply, 200,
        )
        .expect("pca run failed");
        assert_eq!(outs.len(), 2);
        assert_eq!(outs[0].0, "1");
        assert_eq!(outs[1].0, "2");
        let ev = metrics["explained_variance_pct"].as_array().unwrap();
        let total: f64 = ev.iter().map(|v| v.as_f64().unwrap()).sum();
        assert!(total > 99.0, "explained variance = {total}%");
    }

    #[test]
    fn eval_leaderboard_blind_well_cv_ranks_linear_top() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // 3 wells (groups), single feature, exact y = 2x + 1. Under blind-well GroupKFold the
        // linear model must generalize (R^2 ~ 1) to the held-out well; compare linear vs rf.
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut g = Vec::new();
        for well in 0..3 {
            for i in 0..40 {
                let xv = (well * 40 + i) as f32 * 0.5;
                x.push(xv);
                y.push(2.0 * xv + 1.0);
                g.push(well as f32);
            }
        }
        let combos = vec![("linear".to_string(), vec![0usize]), ("rf".to_string(), vec![0usize])];
        let out = exec_ml_eval(&py, "regression", 1, y.len(), &x, &y, &g, &combos, false, 42, 3, &Default::default(), None)
            .expect("eval run failed");
        assert_eq!(out.cv, "blind-well GroupKFold");
        assert_eq!(out.n_groups, 3);
        let lin = out.rows.iter().find(|r| r.algorithm == "linear").expect("linear row");
        assert!(lin.error.is_none(), "linear errored: {:?}", lin.error);
        assert!(lin.score.unwrap() > 0.999, "linear blind-well R2 = {:?}", lin.score);
        assert_eq!(lin.importances.len(), 1);

        // The crossplot's alignment contract. `blind_actual[i]` and `blind_pred[i]` must be the same
        // ROW, and `blind_group[i]` must be that row's well — a misalignment produces a scatter that
        // looks entirely plausible and describes nothing, which nothing downstream could catch.
        //
        // Pinned on an EXACT relationship, so alignment and correctness are separable: y = 2x + 1
        // means a correctly aligned linear prediction lands on its own actual to within rounding,
        // while any shuffle of one array against the other scatters immediately. The runner rounds
        // to six decimals for the wire, so the tolerance is about that and nothing else.
        assert_eq!(
            lin.blind_pred.len(),
            out.blind_actual.len(),
            "one prediction per sampled row, or the two arrays cannot be indexed together",
        );
        assert_eq!(out.blind_group.len(), out.blind_actual.len(), "one well per sampled row");
        assert_eq!(out.blind_total, y.len(), "the total is the population, not the sample");
        assert!(out.blind_sampled > 0 && out.blind_sampled <= out.blind_total);
        let mut checked = 0;
        for (i, (a, p)) in out.blind_actual.iter().zip(lin.blind_pred.iter()).enumerate() {
            let (Some(a), Some(p)) = (a, p) else { continue };
            assert!(
                (a - p).abs() < 1e-3,
                "row {i}: actual {a} against prediction {p} - the two arrays are not the same rows",
            );
            assert!(out.blind_group[i] < 3, "row {i} names well {} of 3", out.blind_group[i]);
            checked += 1;
        }
        assert!(checked > 10, "only {checked} rows carried both an actual and a prediction");
    }

    #[test]
    fn eval_leaderboard_classification_returns_confusion() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // 3 wells, two separable 2-D blobs per well labelled 0/1. Blind-well accuracy ~1 and a
        // 2x2 confusion matrix must come back with the class labels.
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut g = Vec::new();
        for well in 0..3 {
            for i in 0..30 {
                let j = (i % 6) as f32 * 0.01;
                x.extend_from_slice(&[j, j]);
                y.push(0.0);
                g.push(well as f32);
                x.extend_from_slice(&[10.0 + j, 10.0 + j]);
                y.push(1.0);
                g.push(well as f32);
            }
        }
        let combos = vec![("knn".to_string(), vec![0usize, 1usize])];
        let out = exec_ml_eval(&py, "classification", 2, y.len(), &x, &y, &g, &combos, true, 42, 3, &Default::default(), None)
            .expect("eval run failed");
        let row = &out.rows[0];
        assert!(row.error.is_none(), "knn errored: {:?}", row.error);
        assert!(row.score.unwrap() > 0.99, "blind-well accuracy = {:?}", row.score);
        let conf = row.confusion.as_ref().expect("confusion matrix");
        assert_eq!(conf.len(), 2);
        assert_eq!(row.labels.as_ref().unwrap(), &vec![0i64, 1]);
    }

    /// SB-MLA-028 / SB-MLA-T29. The spec asks for a direct structural assertion — that no transform
    /// is fitted on data outside the fold's training partition — and deliberately does not assert a
    /// magnitude, because the size of the leak is fixture-dependent while the pipeline order is not.
    ///
    /// Pinned from BOTH sides on purpose. Asserting only "a scaler is fitted per fold" would pass on
    /// an implementation that ALSO standardized globally and then re-standardized inside the fold;
    /// asserting only "nothing is fitted before the split" would pass on one that never standardized
    /// at all. The two halves together admit one arrangement.
    #[test]
    fn no_transform_is_fitted_outside_the_folds_training_rows() {
        let src = ml_eval_runner();
        let src = src.as_str();

        // Side one: nothing is fitted before the split, and the split sees the RAW matrix.
        assert!(
            !src.contains("fit_transform"),
            "a fit_transform survives in the leaderboard runner; a transform fitted over the whole \
             matrix has seen the held-out well"
        );
        assert!(
            src.contains("splitter.split(X, y, groups)") && src.contains("splitter.split(X, y)"),
            "the splitter must partition the raw matrix - splitting a pre-transformed one is the \
             leak with an extra step"
        );

        // Side two: the scaler that does exist is fitted on the fold's TRAINING rows, and does so
        // after the folds are known.
        let fit = src.find("StandardScaler().fit(X[tr])").expect(
            "the per-fold scaler must be fitted on the fold's training rows, X[tr], and nothing else",
        );
        let split = src.find("splitter.split(").expect("splitter");
        assert!(
            fit > split,
            "the scaler is fitted at byte {fit}, before the folds exist at byte {split}"
        );

        // And the importance is measured on the held-out rows, by the fold's own model - not on a
        // second model refitted over everything, which is what this file did until 2026-08-07.
        assert!(
            src.contains("permutation_importance(m, Xte, yt[te]"),
            "permutation importance must be measured on the held-out partition, so it answers the \
             same question the blind score does"
        );
    }

    /// SB-MLA-026 / SB-MLA-T27. The requirement asks for a test asserting that the evaluation and
    /// training paths construct identical estimators for every supported algorithm. The strongest
    /// form of that assertion is that there is only one construction to compare: both runners are
    /// composed from `ML_BUILD_MODEL`, so an estimator cannot be declared in one and not the other.
    ///
    /// Pinned from both sides. "Both embed the fragment" alone would pass on a runner that embedded
    /// it and then shadowed it with a local constructor — which is exactly the shape the defect had,
    /// two independent declarations — so the second half asserts no estimator is constructed outside
    /// the fragment at all.
    #[test]
    fn the_leaderboard_builds_the_same_estimators_the_run_will_fit() {
        // Every estimator this product can fit. A new algorithm added to one runner and not the
        // other is the defect this test exists to prevent, so the list is spelled out.
        const ESTIMATORS: &[&str] = &[
            "RandomForestRegressor(",
            "XGBRegressor(",
            "HistGradientBoostingRegressor(",
            "SVR(",
            "MLPRegressor(",
            "LinearRegression(",
            "PolynomialFeatures(",
            "SVC(",
            "KNeighborsClassifier(",
            "RandomForestClassifier(",
            "GaussianNB(",
            "LogisticRegression(",
        ];

        for name in ESTIMATORS {
            assert!(
                ML_BUILD_MODEL.contains(name),
                "{name} is not in the shared estimator definitions"
            );
            // Side one: neither runner body declares its own.
            assert!(
                !ML_RUNNER_BODY.contains(name),
                "the training runner constructs {name} itself instead of calling build_model - that \
                 is the second declaration the leaderboard used to drift from"
            );
            assert!(
                !ML_EVAL_RUNNER_BODY.contains(name),
                "the leaderboard runner constructs {name} itself instead of calling build_model"
            );
        }

        // Side two: both composed runners actually carry the shared definitions and call them.
        for (who, src) in [("training", ml_runner()), ("leaderboard", ml_eval_runner())] {
            assert!(src.contains(ML_BUILD_MODEL), "the {who} runner does not embed build_model");
            assert!(
                src.contains("build_model(task, algo, p, seed)"),
                "the {who} runner never calls build_model with the run's own parameter map"
            );
        }

        // And the leaderboard reads a parameter map at all — without it every candidate is ranked
        // at its defaults however the run is configured.
        assert!(
            ML_EVAL_RUNNER_BODY.contains("p = header.get(\"params\")"),
            "the leaderboard runner ignores the hyperparameters the run will use"
        );
    }

    /// The behavioural half of SB-MLA-T27. `degree` is the divergence with the largest consequence:
    /// the leaderboard used to rank a cubic fit as a straight line, so a user comparing `linear`
    /// against anything else was reading the wrong row entirely.
    #[test]
    fn a_polynomial_degree_is_ranked_as_a_polynomial_not_as_a_line() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // y = x^2 over three wells. A straight line cannot fit it; a cubic-capable pipeline can.
        // Same data, same algorithm id, one parameter apart.
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut g = Vec::new();
        for well in 0..3 {
            for i in 0..40 {
                let xv = (well * 40 + i) as f32 * 0.05;
                x.push(xv);
                y.push(xv * xv);
                g.push(well as f32);
            }
        }
        let combos = vec![("linear".to_string(), vec![0usize])];
        let run = |params: serde_json::Map<String, serde_json::Value>| {
            // Scoped to "linear", the way the dialog scopes the settings it is showing.
            exec_ml_eval(
                &py, "regression", 1, y.len(), &x, &y, &g, &combos, false, 42, 3, &params,
                Some("linear"),
            )
            .expect("eval run failed")
        };

        let straight = run(Default::default());
        let mut cubic_params = serde_json::Map::new();
        cubic_params.insert("degree".into(), serde_json::json!(3));
        let cubic = run(cubic_params);

        let s = straight.rows[0].score.expect("straight score");
        let c = cubic.rows[0].score.expect("cubic score");
        assert!(
            c > s,
            "degree=3 scored {c} against a straight line's {s} - the leaderboard is still ranking \
             the same estimator for both, so the parameter never reached it"
        );
    }

    /// The other half of SB-MLA-T29, and the one that needs the optional package. A fixture where
    /// one well sits far from the other two is the case the leak flatters: pooled centring drags the
    /// outlier toward the middle before the model is asked to be blind to it.
    #[test]
    fn a_shifted_well_is_standardized_by_the_wells_that_trained_on_it() {
        let Some(py) = python_with_sklearn() else {
            eprintln!("skipping: no python+sklearn on this machine");
            return;
        };
        // Three wells on one line, y = 2x + 1, but well 2's feature range is a thousand-fold away.
        // The relationship is identical, so a correctly-standardized blind fold still recovers it;
        // what changes is which rows supplied the centring.
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut g = Vec::new();
        for well in 0..3 {
            let scale = if well == 2 { 1000.0f32 } else { 1.0 };
            for i in 0..40 {
                let xv = (i as f32 * 0.5 + 1.0) * scale;
                x.push(xv);
                y.push(2.0 * xv + 1.0);
                g.push(well as f32);
            }
        }
        let combos = vec![("linear".to_string(), vec![0usize])];
        let out = exec_ml_eval(&py, "regression", 1, y.len(), &x, &y, &g, &combos, true, 42, 3, &Default::default(), None)
            .expect("eval run failed");
        assert_eq!(out.cv, "blind-well GroupKFold");
        let row = &out.rows[0];
        assert!(row.error.is_none(), "linear errored: {:?}", row.error);

        // Every fold contributed an importance, which is only possible if importance is measured
        // per fold. A single refit over everything could contribute exactly one.
        assert_eq!(
            row.n_imp_folds, out.n_splits,
            "importance came from {} folds but the score came from {}",
            row.n_imp_folds, out.n_splits
        );
        assert_eq!(row.importances.len(), 1);
        assert_eq!(row.importances_std.len(), 1);
        assert!(
            row.importances[0].is_finite() && row.importances_std[0].is_finite(),
            "importance {:?} +/- {:?}",
            row.importances[0],
            row.importances_std[0]
        );
    }
}
