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

pub fn run_ml(db: &Mutex<Connection>, req: &MlRequest) -> MlResult {
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
            for well_id in &req.train_well_ids {
                let Ok((depth, cols)) = fetch_curve_frame(&conn, well_id, &fetch_names) else { continue };
                let Some(tv) = cols.get(&tgt) else { continue };
                let Some(fcols) = features.iter().map(|f| cols.get(f)).collect::<Option<Vec<_>>>() else { continue };
                for i in 0..depth.len() {
                    if tv[i].is_finite() && fcols.iter().all(|c| c[i].is_finite()) {
                        for c in &fcols {
                            x_train.push(c[i]);
                        }
                        y_train.push(tv[i]);
                    }
                }
            }
        }
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame(&conn, well_id, &features) {
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
                    let mut idx = Vec::new();
                    for i in 0..depth.len() {
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
        return fail("no complete samples in the apply wells (every row has at least one missing input)");
    }

    let y_opt = if supervised { Some(y_train.as_slice()) } else { None };
    match exec_ml(&python, &req.task, &req.algorithm, &req.params, d, &x_train, y_opt, &x_apply, n_apply) {
        Err(e) => fail(&e),
        Ok((metrics, outs)) => {
            let out_names: Vec<String> = outs.iter().map(|(s, _)| format!("{base}{s}")).collect();
            let mut wells = Vec::new();
            let conn = db.lock().unwrap();
            let mut start = 0usize;
            for aw in &apply {
                if let Some(e) = &aw.error {
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
                    Ok(()) => wells.push(MlWellResult {
                        well_id: aw.well_id.clone(),
                        rows_predicted: m,
                        error: (m == 0).then(|| "no complete samples in this well".to_string()),
                    }),
                    Err(e) => wells.push(MlWellResult {
                        well_id: aw.well_id.clone(),
                        rows_predicted: 0,
                        error: Some(e.to_string()),
                    }),
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
}
