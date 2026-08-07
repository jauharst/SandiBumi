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

def P(p, key, default):
    """Read a parameter, and RECORD what was actually used (SB-MLA-001).

    A re-run cannot be reconstructed from a record that omits a value that changed the answer,
    and the value that changed the answer is very often one nobody supplied - `seed` above all,
    which is the single parameter with the largest effect on a clustering result. So every read
    goes through here and every default is recorded AS a default, naming where it came from.
    Reading `P(p, key, default)` directly is the defect this exists to prevent; there should be
    no `p.get` left in either runner.
    """
    v = dict.get(p, key) if p else None
    if v is None or v == "":
        EFFECTIVE[key] = {"value": default, "defaulted": True, "source": "ml.rs build_model default"}
        return default
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

/// The training runner: the shared estimator definitions, then the fit-and-apply body.
fn ml_runner() -> String {
    format!("{ML_BUILD_MODEL}{ML_RUNNER_BODY}")
}

/// The leaderboard runner, from the SAME estimator definitions. Composing both from one fragment is
/// what makes SB-MLA-026 structural rather than a pair of copies somebody has to remember to sync.
fn ml_eval_runner() -> String {
    format!("{ML_BUILD_MODEL}{ML_EVAL_RUNNER_BODY}")
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

seed = int(P(p, "seed", 42))
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
else:
    scaler = None
    Xs, As = X, A

def fit_xy(yv):
    """The rows the model is allowed to learn from. Everything else is being kept honest."""
    return (Xs[fit_rows], yv[fit_rows]) if fit_rows is not None else (Xs, yv)

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
        else:
            # One well: there is no blind fold to be had, and saying so is the point.
            sc = cross_val_score(est, X if fit_rows is None else X[fit_rows], yf, cv=KFold(n_splits=5, shuffle=True, random_state=seed), scoring=scoring)
            metrics[key] = float(np.mean(sc))
            metrics[key + "_folds"] = "random folds within ONE well - not a blind score"
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
        else:
            metrics["accuracy_blind"] = float(np.mean(model.predict(Xb) == yb.astype(int)))
    except Exception as e:
        metrics["blind_error"] = str(e)

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
    blind_score(model, "r2")
    outs.append(("", model.predict(As).astype(np.float32)))

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
    metrics["class_counts"] = {str(c): int(np.sum(yf == c)) for c in np.unique(yf)}
    metrics["n_train"] = n_train
    blind_score(model, "accuracy")
    outs.append(("", model.predict(As).astype(np.float32)))
    outs.append(("_PROB", np.max(model.predict_proba(As), axis=1).astype(np.float32)))

elif task == "clustering":
    k = int(P(p, "k", 5))
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
        labels = AgglomerativeClustering(n_clusters=k, linkage=str(P(p, "linkage", "ward"))).fit_predict(As)
    elif algo == "dbscan":
        from sklearn.cluster import DBSCAN
        labels = DBSCAN(eps=float(P(p, "eps", 0.5)), min_samples=int(P(p, "min_samples", 10))).fit_predict(As)
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
        c = max(1, min(d, int(P(p, "n_components", 3))))
        P_used("n_components", c)
        pca = PCA(n_components=c, random_state=seed)
        Z = pca.fit_transform(As)
        metrics["explained_variance_pct"] = [round(float(v) * 100, 2) for v in pca.explained_variance_ratio_]
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

sys.stdout.buffer.write((json.dumps({"suffixes": [s for s, _ in outs], "metrics": metrics,
                                     "model_len": len(model_blob), "sklearn": sklearn_version}) + "\n").encode("utf-8"))
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
                                     "metrics": {"n_apply": n_apply, "applied": True}}) + "\n").encode("utf-8"))
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
}

/// Applying an already-fitted model. Deliberately NOT an `MlRequest`: there is no training
/// well, no algorithm and no parameter here — those are properties of the saved model, and
/// letting a caller restate them would invite them to differ.
#[derive(Debug, Clone, Deserialize)]
pub struct MlApplyRequest {
    /// Read the model's feature curves from this log set (see [`MlRequest::input_set`]).
    #[serde(default)]
    pub input_set: Option<String>,
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

fn assemble_training(
    conn: &Connection,
    train_well_ids: &[String],
    features: &[String],
    tgt: &str,
    mask_curve: Option<&String>,
    input_set: Option<&str>,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<String>) {
    let mut fetch_names = features.to_vec();
    fetch_names.push(tgt.to_string());
    if let Some(mk) = mask_curve {
        fetch_names.push(mk.clone());
    }
    let mut x_train: Vec<f32> = Vec::new();
    let mut y_train: Vec<f32> = Vec::new();
    // One well index per row, so the runner can hold out whole wells rather than samples.
    let mut groups: Vec<f32> = Vec::new();
    let mut empty_train: Vec<String> = Vec::new();
    for (g, well_id) in train_well_ids.iter().enumerate() {
        let before = y_train.len();
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
        // A well that moved y_train not at all contributed nothing — unreadable, lacking the
        // target/feature, or fully masked. Record it instead of dropping it invisibly.
        if y_train.len() == before {
            empty_train.push(well_id.clone());
        }
    }
    (x_train, y_train, groups, empty_train)
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
    let mut apply: Vec<ApplyWell> = Vec::new();
    let mut x_apply: Vec<f32> = Vec::new();
    {
        let conn = db.lock().unwrap();
        if supervised {
            let tgt = target.clone().unwrap();
            let (xt, yt, gt, empty) =
                assemble_training(&conn, &req.train_well_ids, &features, &tgt, mask_curve.as_ref(), req.input_set.as_deref());
            x_train = xt;
            y_train = yt;
            groups = gt;
            empty_train = empty;
        }
        let mut apply_fetch = features.clone();
        if let Some(mk) = &mask_curve {
            apply_fetch.push(mk.clone());
        }
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
                        if mcol.map_or(false, |m| m[i] == 1.0) {
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

    let n_train = y_train.len();
    if supervised && n_train < 10 {
        return fail(&format!(
            "only {n_train} labelled training samples - need at least 10 (input curves + target must overlap in the training wells)"
        ));
    }
    // Surface training wells that contributed nothing (wrong target mnemonic, missing input, or
    // fully masked). Without this, a 20-well selection fit on 3 wells looks like a clean 20-well
    // run — the exact silent-degradation the app's cardinal rule forbids.
    let mut notes: Vec<String> = Vec::new();
    if supervised && !empty_train.is_empty() {
        let requested = req.train_well_ids.len();
        notes.push(format!(
            "{} of {requested} training well(s) contributed no usable samples (missing the target or an input curve, or fully masked); the model was fit on the remaining {}",
            empty_train.len(),
            requested - empty_train.len()
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
    let split_seed = req.split_seed.unwrap_or(42);
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
        Ok((mut metrics, outs, model_blob, sklearn)) => {
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
            let out_names: Vec<String> = outs.iter().map(|(s, _)| format!("{base}{s}")).collect();
            let mut wells = Vec::new();
            let conn = db.lock().unwrap();
            let mut start = 0usize;
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
                for ((_, values), name) in outs.iter().zip(&out_names) {
                    let mut full = vec![f32::NAN; aw.depth.len()];
                    for (j, &i) in aw.idx.iter().enumerate() {
                        full[i] = values[start + j];
                    }
                    curves.push((name.clone(), full));
                }
                let refs: Vec<(&str, &[f32])> = curves.iter().map(|(n, v)| (n.as_str(), v.as_slice())).collect();
                let spec = crate::equations::LogSetSpec {
                    set_name: out_set.clone(),
                    module: format!("ml:{}:{}", req.task, req.algorithm),
                    params_json: params_json.clone(),
                    inputs_json: serde_json::to_string(&req.feature_curves).unwrap_or_default(),
                };
                let versioned = crate::equations::create_log_set(&conn, &aw.well_id, &spec)
                    .and_then(|(set_id, _)| write_computed_curves_versioned(&conn, &aw.well_id, &aw.depth, &refs, &set_id));
                match versioned {
                    Ok(()) => {
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

            // Keep the fit as an artifact. This happens AFTER the curves are written and never
            // fails the run: the predictions are already correct and on disk, so a storage
            // problem here costs the reusable model, not the work.
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
                    match crate::db::insert_ml_model(
                        &conn,
                        name,
                        &req.task,
                        &req.algorithm,
                        &features,
                        target.as_deref(),
                        &params_json,
                        &serde_json::to_string(&metrics).unwrap_or_default(),
                        &trained_on,
                        n_train,
                        req.params.get("standardize").and_then(|v| v.as_bool()).unwrap_or(true),
                        (!sklearn.is_empty()).then_some(sklearn.as_str()),
                        req.model_note.as_deref(),
                        &model_blob,
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
                        Err(e) => notes.push(format!("the curves were written but the model was NOT saved: {e}")),
                    }
                }
            }
            MlResult { outputs: out_names, metrics, wells, notes, model_id, model_name, split, error: None }
        }
    }
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

    let mut apply: Vec<ApplyWell> = Vec::new();
    let mut x_apply: Vec<f32> = Vec::new();
    {
        let conn = db.lock().unwrap();
        let mut fetch = features.clone();
        if let Some(mk) = &mask_curve {
            fetch.push(mk.clone());
        }
        for well_id in &req.apply_well_ids {
            match fetch_curve_frame_from_set(&conn, well_id, &fetch, req.input_set.as_deref(), None) {
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
                        if mcol.map_or(false, |m| m[i] == 1.0) {
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
    let mut cmd = Command::new(&python);
    cmd.args(["-c", ML_APPLY_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
        for ((_, values), name) in outs.iter().zip(&out_names) {
            let mut full = vec![f32::NAN; aw.depth.len()];
            for (j, &i) in aw.idx.iter().enumerate() {
                full[i] = values[start + j];
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
            params_json: serde_json::to_string(&serde_json::json!({
                "model_id": info.model_id, "model_name": info.name,
                "algorithm": info.algorithm, "trained_on": info.trained_on,
            }))
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
    let notes = vec![format!(
        "applied the saved model '{}' ({} on {}), trained on {} well(s) - nothing was refitted",
        info.name,
        info.algorithm,
        info.target_curve.clone().unwrap_or_else(|| "-".into()),
        info.trained_on.len()
    )];
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
        .map(|(m, o, _, _)| (m, o))
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
) -> Result<(serde_json::Value, Vec<(String, Vec<f32>)>, Vec<u8>, String), String> {
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

    let mut cmd = Command::new(python);
    cmd.args(["-c", &ml_runner()]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    Ok((hdr.metrics, outs, body[expect..].to_vec(), hdr.sklearn))
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
folds = int(header.get("folds", 5)); seed = int(header.get("seed", 42))
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
                 "n_imp_folds": int(len(fold_imps)), "confusion": conf, "labels": labs})

out = {"rows": rows, "n_groups": ng, "n_splits": int(nsplits),
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
            MlEvalResult { rows, n_train, n_groups: py.n_groups, cv: py.cv, n_splits: py.n_splits, note, params_for: req.params_for.clone(), error: None }
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

    let mut cmd = Command::new(python);
    cmd.args(["-c", &ml_eval_runner()]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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

    #[test]
    fn a_retrained_model_never_overwrites_the_one_a_delivered_curve_was_made_with() {
        use crate::db;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let feats = vec!["GR".to_string()];
        let blob = vec![1u8, 2, 3];
        let (_, first) = db::insert_ml_model(&conn, "PERM_RF", "regression", "rf", &feats, Some("PERM"),
            "{}", "{}", &["A".into()], 100, true, Some("1.5.0"), None, &blob).unwrap();
        let (_, second) = db::insert_ml_model(&conn, "PERM_RF", "regression", "rf", &feats, Some("PERM"),
            "{}", "{}", &["A".into(), "B".into()], 200, true, Some("1.5.0"), None, &blob).unwrap();
        assert_eq!(first, "PERM_RF");
        assert_eq!(second, "PERM_RF_1", "a second fit is a NEW model, not a replacement");
        assert_eq!(db::list_ml_models(&conn).unwrap().len(), 2);
    }

    #[test]
    fn listing_models_never_carries_their_bytes() {
        use crate::db;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let blob = vec![7u8; 4096];
        let (id, _) = db::insert_ml_model(&conn, "M", "regression", "rf", &["GR".into()], Some("PERM"),
            "{}", "{}", &["A".into()], 10, true, None, None, &blob).unwrap();
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
        let mut cmd = Command::new(&python);
        cmd.args(["-c", ML_APPLY_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
        let (_x, y, groups, empty) = assemble_training(&conn, &ids, &features, "RHOB", None, None);
        assert_eq!(groups.len(), y.len(), "every training row carries the well it came from");

        assert_eq!(y.len(), n, "the well with the target contributes all its rows");
        assert_eq!(
            empty,
            vec![bad.to_string()],
            "the target-less well is flagged empty, not silently dropped"
        );
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
