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

use crate::equations::{fetch_curve_frame, write_computed_curves_versioned};
use crate::python_engine::{find_python, hide_console};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// Python side of the bridge. Keep messages ASCII (Windows console encodings) and keep
/// the algorithm ids in sync with the catalog in `src/ui/mlDialog.ts`.
const ML_RUNNER: &str = r#"
import sys, json
import numpy as np

def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
task = header["task"]; algo = header["algorithm"]; p = header["params"] or {}
d = header["d"]; n_train = header["n_train"]; has_y = header["has_target"]; n_apply = header["n_apply"]
total = n_train * d + (n_train if has_y else 0) + n_apply * d
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

try:
    import sklearn  # noqa: F401
except ImportError:
    fail("scikit-learn is not installed for this Python - run: pip install scikit-learn")
from sklearn.preprocessing import StandardScaler

seed = int(p.get("seed", 42))
supervised = task in ("regression", "classification")
metrics = {}
if bool(p.get("standardize", True)):
    scaler = StandardScaler().fit(X if supervised else A)
    Xs = scaler.transform(X) if n_train else X
    As = scaler.transform(A) if n_apply else A
else:
    Xs, As = X, A

def cv_score(model, scoring, key):
    if n_train >= 30:
        try:
            from sklearn.model_selection import cross_val_score
            metrics[key] = float(np.mean(cross_val_score(model, Xs, y if scoring == "r2" else y.astype(int), cv=5, scoring=scoring)))
        except Exception as e:
            metrics["cv_error"] = str(e)

outs = []
if task == "regression":
    if algo == "rf":
        from sklearn.ensemble import RandomForestRegressor
        model = RandomForestRegressor(n_estimators=int(p.get("n_estimators", 200)),
                                      max_depth=int(p.get("max_depth", 0)) or None,
                                      random_state=seed, n_jobs=-1)
    elif algo == "gbdt":
        try:
            from xgboost import XGBRegressor
            model = XGBRegressor(n_estimators=int(p.get("n_estimators", 300)),
                                 learning_rate=float(p.get("learning_rate", 0.1)),
                                 max_depth=int(p.get("max_depth", 4)), random_state=seed, verbosity=0)
        except ImportError:
            from sklearn.ensemble import HistGradientBoostingRegressor
            model = HistGradientBoostingRegressor(max_iter=int(p.get("n_estimators", 300)),
                                                  learning_rate=float(p.get("learning_rate", 0.1)),
                                                  max_depth=int(p.get("max_depth", 4)) or None,
                                                  random_state=seed)
            metrics["note"] = "xgboost not installed - used sklearn HistGradientBoosting (pip install xgboost)"
    elif algo == "svr":
        from sklearn.svm import SVR
        model = SVR(C=float(p.get("C", 10.0)), epsilon=float(p.get("epsilon", 0.1)))
    elif algo == "ann":
        from sklearn.neural_network import MLPRegressor
        hidden = tuple(int(t) for t in str(p.get("hidden", "64,32")).replace(" ", "").split(",") if t)
        model = MLPRegressor(hidden_layer_sizes=hidden or (64, 32),
                             max_iter=int(p.get("max_iter", 500)), random_state=seed)
    elif algo == "linear":
        from sklearn.linear_model import LinearRegression
        deg = int(p.get("degree", 1))
        if deg > 1:
            from sklearn.pipeline import make_pipeline
            from sklearn.preprocessing import PolynomialFeatures
            model = make_pipeline(PolynomialFeatures(deg), LinearRegression())
        else:
            model = LinearRegression()
    else:
        fail("unknown regression algorithm '" + algo + "'")
    cv_score(model, "r2", "r2_cv5")
    model.fit(Xs, y)
    pred = model.predict(Xs)
    ss_res = float(np.sum((y - pred) ** 2)); ss_tot = max(float(np.sum((y - np.mean(y)) ** 2)), 1e-12)
    metrics["r2_train"] = 1.0 - ss_res / ss_tot
    metrics["rmse_train"] = float(np.sqrt(np.mean((y - pred) ** 2)))
    metrics["n_train"] = n_train
    outs.append(("", model.predict(As).astype(np.float32)))

elif task == "classification":
    yi = y.astype(int)
    if algo == "svm":
        from sklearn.svm import SVC
        model = SVC(C=float(p.get("C", 10.0)), probability=True, random_state=seed)
    elif algo == "knn":
        from sklearn.neighbors import KNeighborsClassifier
        model = KNeighborsClassifier(n_neighbors=int(p.get("n_neighbors", 7)))
    elif algo == "rf":
        from sklearn.ensemble import RandomForestClassifier
        model = RandomForestClassifier(n_estimators=int(p.get("n_estimators", 200)), random_state=seed, n_jobs=-1)
    elif algo == "gnb":
        from sklearn.naive_bayes import GaussianNB
        model = GaussianNB()
    elif algo == "logreg":
        from sklearn.linear_model import LogisticRegression
        model = LogisticRegression(C=float(p.get("C", 1.0)), max_iter=1000)
    else:
        fail("unknown classification algorithm '" + algo + "'")
    cv_score(model, "accuracy", "accuracy_cv5")
    model.fit(Xs, yi)
    metrics["accuracy_train"] = float(np.mean(model.predict(Xs) == yi))
    metrics["class_counts"] = {str(c): int(np.sum(yi == c)) for c in np.unique(yi)}
    metrics["n_train"] = n_train
    outs.append(("", model.predict(As).astype(np.float32)))
    outs.append(("_PROB", np.max(model.predict_proba(As), axis=1).astype(np.float32)))

elif task == "clustering":
    k = int(p.get("k", 5))
    prob = None
    if algo == "kmeans":
        from sklearn.cluster import KMeans
        labels = KMeans(n_clusters=k, n_init=10, random_state=seed).fit_predict(As)
    elif algo == "gmm":
        from sklearn.mixture import GaussianMixture
        gm = GaussianMixture(n_components=k, random_state=seed).fit(As)
        resp = gm.predict_proba(As)
        labels = np.argmax(resp, axis=1); prob = np.max(resp, axis=1)
    elif algo == "hier":
        from sklearn.cluster import AgglomerativeClustering
        labels = AgglomerativeClustering(n_clusters=k, linkage=str(p.get("linkage", "ward"))).fit_predict(As)
    elif algo == "dbscan":
        from sklearn.cluster import DBSCAN
        labels = DBSCAN(eps=float(p.get("eps", 0.5)), min_samples=int(p.get("min_samples", 10))).fit_predict(As)
    else:
        fail("unknown clustering algorithm '" + algo + "'")
    # DBSCAN noise (-1) stays NaN; real clusters get ids ordered by first-feature mean.
    ids = [int(c) for c in np.unique(labels) if c >= 0]
    if not ids:
        fail("clustering found no clusters (DBSCAN: widen eps / lower min_samples)")
    order = sorted(ids, key=lambda c: float(np.mean(A[labels == c, 0])))
    remap = {c: i for i, c in enumerate(order)}
    out = np.full(n_apply, np.nan, dtype=np.float32)
    for c, i in remap.items():
        out[labels == c] = i
    metrics["cluster_sizes"] = {str(remap[c]): int(np.sum(labels == c)) for c in order}
    if algo == "dbscan":
        metrics["noise_pct"] = round(float(np.mean(labels < 0) * 100), 2)
    if len(ids) > 1:
        try:
            from sklearn.metrics import silhouette_score
            keep = np.where(labels >= 0)[0]
            if len(keep) > 5000:
                keep = np.random.RandomState(seed).choice(keep, 5000, replace=False)
            metrics["silhouette"] = round(float(silhouette_score(As[keep], labels[keep])), 4)
        except Exception:
            pass
    outs.append(("", out))
    if prob is not None:
        outs.append(("_PROB", prob.astype(np.float32)))

elif task == "reduction":
    if algo == "pca":
        from sklearn.decomposition import PCA
        c = max(1, min(d, int(p.get("n_components", 3))))
        pca = PCA(n_components=c, random_state=seed)
        Z = pca.fit_transform(As)
        metrics["explained_variance_pct"] = [round(float(v) * 100, 2) for v in pca.explained_variance_ratio_]
    elif algo == "tsne":
        if n_apply > 20000:
            fail("t-SNE is limited to 20000 samples (got " + str(n_apply) + ") - select fewer wells")
        from sklearn.manifold import TSNE
        perp = min(float(p.get("perplexity", 30.0)), max(5.0, (n_apply - 1) / 3.0))
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
sys.stdout.buffer.write((json.dumps({"suffixes": [s for s, _ in outs], "metrics": metrics}) + "\n").encode("utf-8"))
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
    pub error: Option<String>,
}

fn fail(msg: &str) -> MlResult {
    MlResult { outputs: vec![], metrics: serde_json::Value::Null, wells: vec![], error: Some(msg.to_string()) }
}

struct ApplyWell {
    well_id: String,
    depth: Vec<f32>,
    /// Row indices (into `depth`) of the complete samples sent to python, in order.
    idx: Vec<usize>,
    error: Option<String>,
}

pub fn run_ml(db: &Mutex<Connection>, req: &MlRequest, progress: Option<&crate::jobs::JobHandle>) -> MlResult {
    let supervised = matches!(req.task.as_str(), "regression" | "classification");
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
        return fail("no Python with numpy found - install Python 3.10+ with numpy + scikit-learn, or set ARSHILLA_PYTHON to its python.exe");
    };

    let d = features.len();
    let mut x_train: Vec<f32> = Vec::new();
    let mut y_train: Vec<f32> = Vec::new();
    let mut apply: Vec<ApplyWell> = Vec::new();
    let mut x_apply: Vec<f32> = Vec::new();
    {
        let conn = db.lock().unwrap();
        if supervised {
            let tgt = target.clone().unwrap();
            let mut fetch_names = features.clone();
            fetch_names.push(tgt.clone());
            if let Some(mk) = &mask_curve {
                fetch_names.push(mk.clone());
            }
            for well_id in &req.train_well_ids {
                let Ok((depth, cols)) = fetch_curve_frame(&conn, well_id, &fetch_names) else { continue };
                let Some(tv) = cols.get(&tgt) else { continue };
                let Some(fcols) = features.iter().map(|f| cols.get(f)).collect::<Option<Vec<_>>>() else { continue };
                let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
                for i in 0..depth.len() {
                    // MASK convention (workflow.rs): a mask value of exactly 1.0 excludes the
                    // sample from X/y; 0.0 / NaN / absent keeps it.
                    if mcol.map_or(false, |m| m[i] == 1.0) {
                        continue;
                    }
                    if tv[i].is_finite() && fcols.iter().all(|c| c[i].is_finite()) {
                        for c in &fcols {
                            x_train.push(c[i]);
                        }
                        y_train.push(tv[i]);
                    }
                }
            }
        }
        let mut apply_fetch = features.clone();
        if let Some(mk) = &mask_curve {
            apply_fetch.push(mk.clone());
        }
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame(&conn, well_id, &apply_fetch) {
                Ok((depth, cols)) => {
                    let fcols: Vec<&Vec<f32>> = features.iter().filter_map(|f| cols.get(f)).collect();
                    if fcols.len() != d || depth.is_empty() {
                        apply.push(ApplyWell {
                            well_id: well_id.clone(),
                            depth,
                            idx: vec![],
                            error: Some("missing input curve data".into()),
                        });
                        continue;
                    }
                    let mcol = mask_curve.as_ref().and_then(|mk| cols.get(mk));
                    let mut idx = Vec::new();
                    for i in 0..depth.len() {
                        // Masked apply rows (mask == 1.0) are never sent to python, so scatter-back
                        // leaves them NaN — the OUTPUT-blanking half of the module MASK convention.
                        if mcol.map_or(false, |m| m[i] == 1.0) {
                            continue;
                        }
                        if fcols.iter().all(|c| c[i].is_finite()) {
                            for c in &fcols {
                                x_apply.push(c[i]);
                            }
                            idx.push(i);
                        }
                    }
                    apply.push(ApplyWell { well_id: well_id.clone(), depth, idx, error: None });
                }
                Err(e) => apply.push(ApplyWell {
                    well_id: well_id.clone(),
                    depth: vec![],
                    idx: vec![],
                    error: Some(e.to_string()),
                }),
            }
        }
    }

    let n_train = y_train.len();
    if supervised && n_train < 10 {
        return fail(&format!(
            "only {n_train} labelled training samples - need at least 10 (input curves + target must overlap in the training wells)"
        ));
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
    match exec_ml(&python, &req.task, &req.algorithm, &req.params, d, &x_train, y_opt, &x_apply, n_apply) {
        Err(e) => fail(&e),
        Ok((metrics, outs)) => {
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
                let mut curves: Vec<(String, Vec<f32>)> = Vec::with_capacity(outs.len());
                for ((_, values), name) in outs.iter().zip(&out_names) {
                    let mut full = vec![f32::NAN; aw.depth.len()];
                    for (j, &i) in aw.idx.iter().enumerate() {
                        full[i] = values[start + j];
                    }
                    curves.push((name.clone(), full));
                }
                let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
                let spec = crate::equations::LogSetSpec {
                    set_name: "ML".into(),
                    module: format!("ml:{}:{}", req.task, req.algorithm),
                    params_json: serde_json::to_string(&req.params).unwrap_or_default(),
                    inputs_json: serde_json::to_string(&req.feature_curves).unwrap_or_default(),
                };
                let versioned = crate::equations::create_log_set(&conn, &aw.well_id, &spec)
                    .and_then(|(set_id, _)| write_computed_curves_versioned(&conn, &aw.well_id, &aw.depth, &refs, &set_id));
                match versioned {
                    Ok(()) => {
                        if let Some(p) = progress {
                            let (st, msg) = if m == 0 {
                                (crate::jobs::ItemState::Warned, Some("no complete samples in this well".to_string()))
                            } else {
                                (crate::jobs::ItemState::Ok, None)
                            };
                            p.finish_item(&aw.well_id, st, msg);
                        }
                        wells.push(MlWellResult {
                            well_id: aw.well_id.clone(),
                            rows_predicted: m,
                            error: (m == 0).then(|| "no complete samples in this well".to_string()),
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
            MlResult { outputs: out_names, metrics, wells, error: None }
        }
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
    let n_train = if d == 0 { 0 } else { x_train.len() / d };
    let header = serde_json::json!({
        "task": task, "algorithm": algorithm, "params": params,
        "d": d, "n_train": n_train, "has_target": y_train.is_some(), "n_apply": n_apply,
    });

    let mut cmd = Command::new(python);
    cmd.args(["-c", ML_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    }
    let hdr: OutHeader =
        serde_json::from_slice(&output.stdout[..nl]).map_err(|e| format!("bad ML result header: {e}"))?;
    let body = &output.stdout[nl + 1..];
    let expect = hdr.suffixes.len() * n_apply * 4;
    if body.len() != expect {
        return Err(format!("python returned {} result bytes, expected {}", body.len(), expect));
    }
    let mut outs = Vec::with_capacity(hdr.suffixes.len());
    for (i, s) in hdr.suffixes.iter().enumerate() {
        let mut vals = vec![0f32; n_apply];
        bytemuck::cast_slice_mut::<f32, u8>(&mut vals).copy_from_slice(&body[i * n_apply * 4..(i + 1) * n_apply * 4]);
        outs.push((s.clone(), vals));
    }
    Ok((hdr.metrics, outs))
}

// ---------------------------------------------------------------------------------------------
// Model-comparison leaderboard (Wave B item 3): evaluate algorithm x feature-subset combos with
// BLIND-WELL cross-validation (whole wells held out via GroupKFold — the plain random 5-fold in
// ML_RUNNER leaks depth correlation because adjacent samples from one well land in both folds),
// plus permutation feature importance and a confusion matrix. One python round-trip evaluates
// every combo (single sklearn import); no curves are written — this ranks approaches to pick from.
// ---------------------------------------------------------------------------------------------

const ML_EVAL_RUNNER: &str = r#"
import sys, json
import numpy as np

def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)

header = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
task = header["task"]; d = header["d"]; n = header["n_train"]
folds = int(header.get("folds", 5)); seed = int(header.get("seed", 42))
standardize = bool(header.get("standardize", True)); combos = header["combos"]
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

# StandardScaler is per-column, so subselecting standardized columns == standardizing the subset.
Xs = StandardScaler().fit_transform(X) if standardize else X

def make_model(algo):
    if task == "regression":
        if algo == "rf":
            from sklearn.ensemble import RandomForestRegressor
            return RandomForestRegressor(n_estimators=200, random_state=seed, n_jobs=-1)
        if algo == "gbdt":
            try:
                from xgboost import XGBRegressor
                return XGBRegressor(n_estimators=300, learning_rate=0.1, max_depth=4, random_state=seed, verbosity=0)
            except ImportError:
                from sklearn.ensemble import HistGradientBoostingRegressor
                return HistGradientBoostingRegressor(random_state=seed)
        if algo == "svr":
            from sklearn.svm import SVR
            return SVR(C=10.0, epsilon=0.1)
        if algo == "ann":
            from sklearn.neural_network import MLPRegressor
            return MLPRegressor(hidden_layer_sizes=(64, 32), max_iter=500, random_state=seed)
        if algo == "linear":
            from sklearn.linear_model import LinearRegression
            return LinearRegression()
    else:
        if algo == "svm":
            from sklearn.svm import SVC
            return SVC(C=10.0, random_state=seed)
        if algo == "knn":
            from sklearn.neighbors import KNeighborsClassifier
            return KNeighborsClassifier(n_neighbors=7)
        if algo == "rf":
            from sklearn.ensemble import RandomForestClassifier
            return RandomForestClassifier(n_estimators=200, random_state=seed, n_jobs=-1)
        if algo == "gnb":
            from sklearn.naive_bayes import GaussianNB
            return GaussianNB()
        if algo == "logreg":
            from sklearn.linear_model import LogisticRegression
            return LogisticRegression(C=1.0, max_iter=1000)
    return None

ng = int(len(np.unique(groups)))
use_group = ng >= 2
nsplits = min(folds, ng) if use_group else min(folds, n)
nsplits = max(2, nsplits)
splitter = GroupKFold(n_splits=nsplits) if use_group else KFold(n_splits=nsplits, shuffle=True, random_state=seed)
SP = list(splitter.split(Xs, y, groups)) if use_group else list(splitter.split(Xs, y))

clf = task == "classification"
yt = y.astype(int) if clf else y
labels = sorted(int(v) for v in np.unique(yt)) if clf else None
scoring = "accuracy" if clf else "r2"

rows = []
for combo in combos:
    algo = combo["algorithm"]; fidx = combo["feat_idx"]
    Xc = Xs[:, fidx]
    oof = np.full(n, np.nan)
    fold_scores = []
    err = None
    try:
        for tr, te in SP:
            m = make_model(algo)
            if m is None:
                err = "unknown algorithm '" + str(algo) + "'"; break
            m.fit(Xc[tr], yt[tr])
            pred = m.predict(Xc[te])
            oof[te] = pred
            fold_scores.append(accuracy_score(yt[te], pred.astype(int)) if clf else r2_score(y[te], pred))
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
    try:
        mfull = make_model(algo)
        mfull.fit(Xc, yt)
        pi = permutation_importance(mfull, Xc, yt, n_repeats=5, random_state=seed, scoring=scoring)
        imp = [float(v) for v in pi.importances_mean]
    except Exception:
        pass
    rows.append({"algorithm": algo, "feat_idx": fidx, "score": score,
                 "score_std": float(np.std(fold_scores)), "metrics": metrics,
                 "importances": imp, "confusion": conf, "labels": labs})

out = {"rows": rows, "n_groups": ng, "n_splits": int(nsplits),
       "cv": "blind-well GroupKFold" if use_group else "random KFold"}
sys.stdout.buffer.write((json.dumps(out) + "\n").encode("utf-8"))
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct MlEvalRequest {
    /// "regression" | "classification" (supervised only — the leaderboard needs a target).
    pub task: String,
    pub feature_curves: Vec<String>,
    pub target_curve: String,
    pub train_well_ids: Vec<String>,
    /// Algorithm ids to compare (same ids as ML_RUNNER); empty → nothing to do.
    pub algorithms: Vec<String>,
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
}

#[derive(Debug, Serialize)]
pub struct MlEvalRow {
    pub algorithm: String,
    pub features: Vec<String>,
    pub score: Option<f64>,
    pub score_std: Option<f64>,
    pub metrics: serde_json::Value,
    pub importances: Vec<f64>,
    pub confusion: Option<Vec<Vec<i64>>>,
    pub labels: Option<Vec<i64>>,
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
    {
        let conn = db.lock().unwrap();
        let mut fetch_names = features.clone();
        fetch_names.push(target.clone());
        if let Some(mk) = &mask_curve {
            fetch_names.push(mk.clone());
        }
        for (g, well_id) in req.train_well_ids.iter().enumerate() {
            let Ok((depth, cols)) = fetch_curve_frame(&conn, well_id, &fetch_names) else { continue };
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
    let mut note = None;
    if combos.len() > MAX_COMBOS {
        note = Some(format!(
            "evaluated the first {MAX_COMBOS} of {} algorithm×subset combos (cap) — narrow the algorithms or subsets",
            combos.len()
        ));
        combos.truncate(MAX_COMBOS);
    }

    let seed = req.seed.unwrap_or(42);
    let folds = req.folds.unwrap_or(5).clamp(2, 10);
    match exec_ml_eval(&python, &req.task, d, n_train, &x_train, &y_train, &groups, &combos, req.standardize, seed, folds) {
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
                    confusion: r.confusion,
                    labels: r.labels,
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
            MlEvalResult { rows, n_train, n_groups: py.n_groups, cv: py.cv, n_splits: py.n_splits, note, error: None }
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
    confusion: Option<Vec<Vec<i64>>>,
    #[serde(default)]
    labels: Option<Vec<i64>>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct PyEvalOut {
    rows: Vec<PyEvalRow>,
    n_groups: usize,
    n_splits: usize,
    cv: String,
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
) -> Result<PyEvalOut, String> {
    let combos_json: Vec<serde_json::Value> = combos
        .iter()
        .map(|(a, idx)| serde_json::json!({ "algorithm": a, "feat_idx": idx }))
        .collect();
    let header = serde_json::json!({
        "task": task, "d": d, "n_train": n_train,
        "standardize": standardize, "seed": seed, "folds": folds, "combos": combos_json,
    });

    let mut cmd = Command::new(python);
    cmd.args(["-c", ML_EVAL_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    }

    fn mk_req(task: &str, features: &[&str], target: Option<&str>, train: &[String], apply: &[String]) -> MlRequest {
        MlRequest {
            task: task.into(),
            algorithm: if task == "clustering" { "kmeans".into() } else { "linear".into() },
            params: serde_json::Map::new(),
            feature_curves: features.iter().map(|s| s.to_string()).collect(),
            target_curve: target.map(|s| s.to_string()),
            mask_curve: None,
            train_well_ids: train.to_vec(),
            apply_well_ids: apply.to_vec(),
            output_curve: "PRED".into(),
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
            task: "regression".into(),
            feature_curves: vec!["GR".into()],
            target_curve: "RHOB".into(),
            train_well_ids: vec![ida, idb],
            algorithms: vec!["linear".into()],
            subsets: vec![],
            standardize: false,
            seed: Some(42),
            folds: Some(5),
            mask_curve: Some("MASK".into()),
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
        let out = exec_ml_eval(&py, "regression", 1, y.len(), &x, &y, &g, &combos, false, 42, 3)
            .expect("eval run failed");
        assert_eq!(out.cv, "blind-well GroupKFold");
        assert_eq!(out.n_groups, 3);
        let lin = out.rows.iter().find(|r| r.algorithm == "linear").expect("linear row");
        assert!(lin.error.is_none(), "linear errored: {:?}", lin.error);
        assert!(lin.score.unwrap() > 0.999, "linear blind-well R2 = {:?}", lin.score);
        assert_eq!(lin.importances.len(), 1);
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
        let out = exec_ml_eval(&py, "classification", 2, y.len(), &x, &y, &g, &combos, true, 42, 3)
            .expect("eval run failed");
        let row = &out.rows[0];
        assert!(row.error.is_none(), "knn errored: {:?}", row.error);
        assert!(row.score.unwrap() > 0.99, "blind-well accuracy = {:?}", row.score);
        let conf = row.confusion.as_ref().expect("confusion matrix");
        assert_eq!(conf.len(), 2);
        assert_eq!(row.labels.as_ref().unwrap(), &vec![0i64, 1]);
    }
}
