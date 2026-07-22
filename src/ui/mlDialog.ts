import {
  listCurveCatalog,
  listWells,
  runMl,
  runMlEval,
  type MlEvalResult,
  type MlEvalRow,
  type MlRequest,
  type MlResult,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, defaultRunWellIds, filterByActiveGroup } from "../state";
import { formRow } from "./modal";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/** Machine-learning dialog (Phase 10-4): one entry point for the whole catalog —
 *  supervised regression/classification (fit on labelled train wells, predict on apply
 *  wells) and unsupervised clustering/dimensionality-reduction (fit on the pooled apply
 *  wells, so clustering is field-wide with globally consistent ids). Models run in the
 *  scikit-learn subprocess; results land in computed_curves like any module output.
 *
 *  Algorithm ids must stay in sync with ML_RUNNER in src-tauri/src/ml.rs. */

interface ParamSpec {
  key: string;
  label: string;
  kind: "num" | "text" | "select";
  def: number | string;
  options?: string[];
}

interface AlgoSpec {
  id: string;
  label: string;
  desc: string;
  params: ParamSpec[];
  /** reduction only: overrides the task's default output base name. */
  out?: string;
}

interface TaskSpec {
  id: MlRequest["task"];
  label: string;
  supervised: boolean;
  defaultOut: string;
  algos: AlgoSpec[];
}

const num = (key: string, label: string, def: number): ParamSpec => ({ key, label, kind: "num", def });
const txt = (key: string, label: string, def: string): ParamSpec => ({ key, label, kind: "text", def });
const sel = (key: string, label: string, def: string, options: string[]): ParamSpec => ({
  key, label, kind: "select", def, options,
});

const TASKS: TaskSpec[] = [
  {
    id: "regression",
    label: "Predict a continuous log (regression)",
    supervised: true,
    defaultOut: "ML_PRED",
    algos: [
      { id: "rf", label: "Random Forest Regressor",
        desc: "Ensemble of averaged decision trees — non-linear and resistant to overfitting.",
        params: [num("n_estimators", "trees", 200), num("max_depth", "max depth (0 = none)", 0)] },
      { id: "gbdt", label: "Gradient Boosting (XGBoost)",
        desc: "Sequential trees minimizing error — highest accuracy in complex settings. Falls back to sklearn boosting if xgboost isn't installed.",
        params: [num("n_estimators", "trees", 300), num("learning_rate", "learning rate", 0.1), num("max_depth", "max depth", 4)] },
      { id: "svr", label: "Support Vector Regression",
        desc: "Margin-of-tolerance hyperplane — performs well on smaller, localized datasets.",
        params: [num("C", "C", 10), num("epsilon", "epsilon", 0.1)] },
      { id: "ann", label: "Neural Network (MLP)",
        desc: "Multi-layer perceptron for complex multi-curve patterns; needs plenty of data.",
        params: [txt("hidden", "hidden layers", "64,32"), num("max_iter", "max iterations", 500)] },
      { id: "linear", label: "Linear / Polynomial",
        desc: "Baseline relationships between curves (degree 1 = plain linear regression).",
        params: [num("degree", "polynomial degree", 1)] },
    ],
  },
  {
    id: "classification",
    label: "Predict a discrete log (classification)",
    supervised: true,
    defaultOut: "ML_CLASS",
    algos: [
      { id: "svm", label: "Support Vector Machine",
        desc: "Non-linear separator for distinct rock types via high-dimensional mapping.",
        params: [num("C", "C", 10)] },
      { id: "knn", label: "K-Nearest Neighbors",
        desc: "Labels each sample by the most common class among its nearest neighbours.",
        params: [num("n_neighbors", "neighbours", 7)] },
      { id: "rf", label: "Random Forest Classifier",
        desc: "Ensemble trees — robust against noisy or incomplete log data.",
        params: [num("n_estimators", "trees", 200)] },
      { id: "gnb", label: "Gaussian Naive Bayes",
        desc: "Fast probabilistic classifier — lithology prediction with minimal tuning.",
        params: [] },
      { id: "logreg", label: "Logistic Regression",
        desc: "Probability baseline for binary or multi-class decisions (sand vs shale).",
        params: [num("C", "C", 1)] },
    ],
  },
  {
    id: "clustering",
    label: "Electrofacies clustering (unsupervised)",
    supervised: false,
    defaultOut: "FACIES_ML",
    algos: [
      { id: "kmeans", label: "K-Means",
        desc: "Industry standard for rapid electrofacies — pick the number of classes K.",
        params: [num("k", "K classes", 5)] },
      { id: "gmm", label: "Gaussian Mixture (GMM)",
        desc: "Probabilistic clusters — best for overlapping or transitional facies; writes a _PROB confidence curve.",
        params: [num("k", "K classes", 5)] },
      { id: "hier", label: "Hierarchical (Agglomerative)",
        desc: "Builds a cluster tree and cuts it at K groups — good for adjusting facies granularity.",
        params: [num("k", "K classes", 5), sel("linkage", "linkage", "ward", ["ward", "complete", "average"])] },
      { id: "dbscan", label: "DBSCAN (density)",
        desc: "Density-based groups — isolates rare anomalies; noise samples stay empty (NaN).",
        params: [num("eps", "eps (std-dev units)", 0.5), num("min_samples", "min samples", 10)] },
    ],
  },
  {
    id: "reduction",
    label: "Dimensionality reduction (PCA / t-SNE)",
    supervised: false,
    defaultOut: "PC",
    algos: [
      { id: "pca", label: "Principal Component Analysis", out: "PC",
        desc: "Combines correlated logs into orthogonal components (writes PC1, PC2, …).",
        params: [num("n_components", "components", 3)] },
      { id: "tsne", label: "t-SNE (2-D map)", out: "TSNE",
        desc: "Maps multi-curve responses into a 2-D space — crossplot TSNE1 vs TSNE2 (max 20000 samples).",
        params: [num("perplexity", "perplexity", 30)] },
    ],
  },
];

const DEFAULT_FEATURES = ["GR", "NPHI", "RHOB", "RES_DEEP", "DT"];

/** Hosted as a dock pane (workspace component "ml"), not a popup. */
export async function buildMlContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const [wells, catalog] = await Promise.all([
    listWells().then(filterByActiveGroup).catch(() => [] as WellSummary[]),
    listCurveCatalog().catch(() => []),
  ]);
  const curveNames = catalog.map((c) => c.name);
  const selected = appState.selectedWell.get();
  // Apply wells (the run scope) come from the shared scope selector; Train wells stay a
  // checklist below — a distinct labelled-data pick, not the run coverage.
  const scope = await buildWellScope();

  let task = TASKS[0];
  let algo = task.algos[0];

  const content = document.createElement("div");
  content.className = "mc-dialog";

  // --- Task + algorithm ----------------------------------------------------
  const taskSel = document.createElement("select");
  for (const t of TASKS) {
    const o = document.createElement("option");
    o.value = t.id;
    o.textContent = t.label;
    taskSel.appendChild(o);
  }
  content.appendChild(formRow("Task", taskSel));

  const algoSel = document.createElement("select");
  const algoDesc = document.createElement("div");
  algoDesc.className = "mc-chain-note";
  content.appendChild(formRow("Algorithm", algoSel));
  content.appendChild(algoDesc);

  // --- Input curves + target ----------------------------------------------
  const featBox = document.createElement("div");
  featBox.className = "mc-wells";
  const featChecks = new Map<string, HTMLInputElement>();
  for (const name of curveNames) {
    const label = document.createElement("label");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.value = name;
    cb.checked = DEFAULT_FEATURES.includes(name);
    featChecks.set(name, cb);
    label.append(cb, document.createTextNode(` ${name}`));
    featBox.appendChild(label);
  }
  content.appendChild(
    formRow("Input curves", featBox, "Order matters for clustering: class 0 = lowest mean of the FIRST checked curve (put GR first)."),
  );

  const targetSel = document.createElement("select");
  for (const name of curveNames) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = name;
    targetSel.appendChild(o);
  }
  if (curveNames.includes("FACIES")) targetSel.value = "FACIES";
  const targetRow = formRow(
    "Target curve", targetSel,
    "The labelled 'ground truth' to learn (core-calibrated curve, interpreted facies, …).",
  );
  content.appendChild(targetRow);

  // Optional MASK curve — kept visible for ALL tasks (it also governs the unsupervised fit pool),
  // default "(none)" so data is never silently dropped.
  const maskSel = document.createElement("select");
  const maskNone = document.createElement("option");
  maskNone.value = "";
  maskNone.textContent = "(none)";
  maskSel.appendChild(maskNone);
  for (const name of curveNames) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = name;
    maskSel.appendChild(o);
  }
  content.appendChild(
    formRow(
      "Mask (exclude)", maskSel,
      "Optional 0/1 flag curve: samples where the mask = 1 are excluded from training and left blank (NaN) in the output — bad-hole, coal, casing.",
    ),
  );

  // --- Wells ---------------------------------------------------------------
  function wellBox(defaultAll: boolean): { el: HTMLElement; checks: Map<string, HTMLInputElement> } {
    const box = document.createElement("div");
    box.className = "mc-wells";
    const checks = new Map<string, HTMLInputElement>();
    const runDefaults = defaultRunWellIds(wells);
    if (runDefaults.size === 0 && selected) runDefaults.add(selected.well_id);
    for (const w of wells) {
      const label = document.createElement("label");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.value = w.well_id;
      cb.checked = defaultAll || runDefaults.has(w.well_id);
      checks.set(w.well_id, cb);
      label.append(cb, document.createTextNode(` ${w.well_name}`));
      box.appendChild(label);
    }
    return { el: box, checks };
  }
  const train = wellBox(false);
  const trainRow = formRow("Train wells", train.el, "Wells whose labelled samples fit the model.");
  content.appendChild(trainRow);
  // Apply wells = the run scope. Unsupervised models also FIT on these (pooled — field-wide).
  content.appendChild(scope.el);

  // --- Hyperparameters + output -------------------------------------------
  const paramsGrid = document.createElement("div");
  paramsGrid.className = "mc-settings";
  content.appendChild(formRow("Parameters", paramsGrid));
  let paramInputs: { spec: ParamSpec; get: () => number | string }[] = [];

  const outInput = document.createElement("input");
  outInput.type = "text";
  outInput.value = task.defaultOut;
  let outEdited = false;
  outInput.addEventListener("input", () => (outEdited = true));
  content.appendChild(
    formRow("Output curve", outInput, "Extra outputs get suffixes: _PROB (confidence), or PC1/PC2… for reduction."),
  );

  const stdCb = document.createElement("input");
  stdCb.type = "checkbox";
  stdCb.checked = true;
  const seedInput = document.createElement("input");
  seedInput.type = "number";
  seedInput.value = "42";
  const commonWrap = document.createElement("div");
  commonWrap.className = "mc-settings";
  const stdLabel = document.createElement("label");
  stdLabel.className = "mc-field";
  stdLabel.append(stdCb, document.createTextNode(" Standardize inputs (z-score)"));
  const seedLabel = document.createElement("label");
  seedLabel.className = "mc-field";
  const seedText = document.createElement("span");
  seedText.textContent = "Seed";
  seedLabel.append(seedText, seedInput);
  commonWrap.append(stdLabel, seedLabel);
  content.appendChild(formRow("Common", commonWrap));

  function renderParams(): void {
    paramsGrid.innerHTML = "";
    paramInputs = [];
    if (algo.params.length === 0) {
      const none = document.createElement("span");
      none.className = "mc-empty";
      none.textContent = "No tuning parameters.";
      paramsGrid.appendChild(none);
      return;
    }
    for (const spec of algo.params) {
      const label = document.createElement("label");
      label.className = "mc-field";
      const t = document.createElement("span");
      t.textContent = spec.label;
      let input: HTMLInputElement | HTMLSelectElement;
      if (spec.kind === "select") {
        input = document.createElement("select");
        for (const opt of spec.options ?? []) {
          const o = document.createElement("option");
          o.value = opt;
          o.textContent = opt;
          input.appendChild(o);
        }
        input.value = String(spec.def);
      } else {
        input = document.createElement("input");
        input.type = spec.kind === "num" ? "number" : "text";
        if (spec.kind === "num") input.step = "any";
        input.value = String(spec.def);
      }
      label.append(t, input);
      paramsGrid.appendChild(label);
      paramInputs.push({
        spec,
        get: () => (spec.kind === "num" ? parseFloat(input.value) || 0 : input.value),
      });
    }
  }

  function refreshAlgos(): void {
    algoSel.innerHTML = "";
    for (const a of task.algos) {
      const o = document.createElement("option");
      o.value = a.id;
      o.textContent = a.label;
      algoSel.appendChild(o);
    }
    algo = task.algos[0];
    algoSel.value = algo.id;
    syncAlgo();
  }

  function syncAlgo(): void {
    algoDesc.textContent = algo.desc;
    targetRow.style.display = task.supervised ? "" : "none";
    trainRow.style.display = task.supervised ? "" : "none";
    compareRow.style.display = task.supervised ? "" : "none";
    if (!outEdited) outInput.value = algo.out ?? task.defaultOut;
    renderParams();
  }

  taskSel.addEventListener("change", () => {
    task = TASKS.find((t) => t.id === taskSel.value) ?? TASKS[0];
    refreshAlgos();
  });
  algoSel.addEventListener("change", () => {
    algo = task.algos.find((a) => a.id === algoSel.value) ?? task.algos[0];
    syncAlgo();
  });

  // --- Run + results -------------------------------------------------------
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Run Model";
  runBtn.classList.add("primary");
  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  runRow.append(runBtn, statusLine);
  content.appendChild(runRow);

  // --- Compare (leaderboard) — supervised only ------------------------------
  const subsetSel = document.createElement("select");
  for (const [val, lbl] of [
    ["full", "Full set only"],
    ["loco", "Leave-one-curve-out"],
    ["singles", "Full + each single curve"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = lbl;
    subsetSel.appendChild(o);
  }
  const compareBtn = document.createElement("button");
  compareBtn.type = "button";
  compareBtn.textContent = "Compare algorithms";
  compareBtn.title = "Rank every algorithm (× curve subsets) by blind-well cross-validation — writes no curves";
  const compareStatus = document.createElement("div");
  compareStatus.className = "mc-status";
  const compareRow = formRow("Compare", (() => {
    const wrap = document.createElement("div");
    wrap.className = "ml-compare-row";
    wrap.append(subsetSel, compareBtn, compareStatus);
    return wrap;
  })(), "Leaderboard: blind-well GroupKFold CV (whole wells held out) + permutation importance + confusion matrix. Needs ≥2 train wells.");
  content.appendChild(compareRow);

  const results = document.createElement("div");
  results.className = "mc-results";
  content.appendChild(results);

  const buildSubsets = (features: string[], strategy: string): string[][] => {
    const full = features.slice();
    if (strategy === "loco" && features.length > 1) {
      return [full, ...features.map((_, i) => features.filter((_, j) => j !== i))];
    }
    if (strategy === "singles" && features.length > 1) {
      return [full, ...features.map((f) => [f])];
    }
    return [full];
  };

  compareBtn.addEventListener("click", async () => {
    if (!task.supervised) return;
    const features = [...featChecks.entries()].filter(([, cb]) => cb.checked).map(([n]) => n);
    const trainIds = [...train.checks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id);
    if (features.length === 0) {
      setStatus("Check at least one input curve");
      return;
    }
    if (trainIds.length < 2) {
      setStatus("Blind-well comparison needs at least 2 training wells");
      return;
    }
    compareBtn.disabled = true;
    runBtn.disabled = true;
    compareStatus.textContent = "Comparing… (blind-well CV over all combos)";
    const t0 = performance.now();
    try {
      const res = await runMlEval({
        task: task.id as "regression" | "classification",
        feature_curves: features,
        target_curve: targetSel.value,
        mask_curve: maskSel.value || null,
        train_well_ids: trainIds,
        algorithms: task.algos.map((a) => a.id),
        subsets: buildSubsets(features, subsetSel.value),
        standardize: stdCb.checked,
        seed: Math.round(parseFloat(seedInput.value) || 42),
        folds: 5,
      });
      const ms = Math.round(performance.now() - t0);
      if (res.error) {
        compareStatus.textContent = `Failed: ${res.error}`;
      } else {
        compareStatus.textContent =
          `${res.rows.length} combos • ${res.cv} (${res.n_splits} folds, ${res.n_groups} wells) • ${ms} ms` +
          (res.note ? ` • ${res.note}` : "");
        recordProcess("ML", `Leaderboard: compared ${task.algos.length} algorithms on ${features.length} curves`);
      }
      renderLeaderboard(results, res, task.id === "classification");
    } catch (e) {
      compareStatus.textContent = `Failed: ${e}`;
    } finally {
      compareBtn.disabled = false;
      runBtn.disabled = false;
    }
  });

  const hint = document.createElement("div");
  hint.className = "mc-chain-note";
  hint.textContent = "Needs Python with numpy + scikit-learn (pip install scikit-learn); xgboost optional.";
  content.appendChild(hint);

  runBtn.addEventListener("click", async () => {
    const features = [...featChecks.entries()].filter(([, cb]) => cb.checked).map(([n]) => n);
    const applyIds = scope.getWellIds();
    const trainIds = [...train.checks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id);
    if (features.length === 0) {
      setStatus("Check at least one input curve");
      return;
    }
    if (applyIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    const params: Record<string, number | string | boolean> = {
      standardize: stdCb.checked,
      seed: Math.round(parseFloat(seedInput.value) || 42),
    };
    for (const { spec, get } of paramInputs) params[spec.key] = get();
    const req: MlRequest = {
      task: task.id,
      algorithm: algo.id,
      params,
      feature_curves: features,
      target_curve: task.supervised ? targetSel.value : null,
      mask_curve: maskSel.value || null,
      train_well_ids: task.supervised ? trainIds : [],
      apply_well_ids: applyIds,
      output_curve: outInput.value,
    };
    runBtn.disabled = true;
    statusLine.textContent = "Running…";
    const t0 = performance.now();
    try {
      const res = await runMl(req);
      const ms = Math.round(performance.now() - t0);
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
      } else {
        statusLine.textContent = `Done in ${ms} ms → ${res.outputs.join(", ")}`;
        setStatus(`${algo.label}: wrote ${res.outputs.join(", ")} to ${applyIds.length} well(s)`);
        recordProcess("ML", `${algo.label}: wrote ${res.outputs.join(", ")} to ${applyIds.length} well(s)`);
        bumpDataVersion(); // ML wrote curves — refresh open plots/log views/catalog
      }
      renderResults(results, res);
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  refreshAlgos();
  return { el: content, dispose: () => scope.dispose() };
}

function fmtMetric(v: unknown): string {
  if (typeof v === "number") {
    return Number.isInteger(v) ? String(v) : v.toFixed(4);
  }
  if (Array.isArray(v)) return v.map(fmtMetric).join(", ");
  if (v !== null && typeof v === "object") {
    return Object.entries(v as Record<string, unknown>)
      .map(([k, x]) => `${k}: ${fmtMetric(x)}`)
      .join("  ·  ");
  }
  return String(v);
}

function renderResults(host: HTMLElement, res: MlResult): void {
  host.innerHTML = "";
  if (res.error) return;

  if (res.metrics && typeof res.metrics === "object") {
    const table = document.createElement("table");
    table.className = "mc-table";
    for (const [key, value] of Object.entries(res.metrics)) {
      const tr = document.createElement("tr");
      const th = document.createElement("th");
      th.textContent = key;
      const td = document.createElement("td");
      td.textContent = fmtMetric(value);
      tr.append(th, td);
      table.appendChild(tr);
    }
    host.appendChild(table);
  }

  const wellsTable = document.createElement("table");
  wellsTable.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["Well", "Predicted samples", "Error"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  wellsTable.appendChild(head);
  for (const w of res.wells) {
    const tr = document.createElement("tr");
    for (const c of [w.well_id, String(w.rows_predicted), w.error ?? "—"]) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    wellsTable.appendChild(tr);
  }
  host.appendChild(wellsTable);
}

/** Leaderboard table (best first) + a details panel (permutation importance + confusion matrix)
 *  for the selected row. Backend already sorts rows by blind-well score descending. */
function renderLeaderboard(host: HTMLElement, res: MlEvalResult, isClf: boolean): void {
  host.innerHTML = "";
  if (res.error || !res.rows.length) return;
  const scoreLabel = isClf ? "Accuracy" : "R²";
  const secLabel = isClf ? "macro-F1" : "RMSE";

  const table = document.createElement("table");
  table.className = "mc-table ml-leaderboard";
  const head = document.createElement("tr");
  for (const h of ["#", "Algorithm", "Curves", scoreLabel, "±", secLabel]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const detail = document.createElement("div");
  detail.className = "ml-detail";

  const firstOkRow = res.rows.find((r) => !r.error) ?? null;
  res.rows.forEach((row, i) => {
    const tr = document.createElement("tr");
    const sec = isClf ? row.metrics?.["macro_f1"] : row.metrics?.["rmse"];
    const cells = row.error
      ? [String(i + 1), row.algorithm, row.features.join(", "), "error", "—", row.error]
      : [
          String(i + 1),
          row.algorithm,
          row.features.join(", "),
          row.score != null ? row.score.toFixed(4) : "—",
          row.score_std != null ? `±${row.score_std.toFixed(3)}` : "",
          typeof sec === "number" ? sec.toFixed(4) : "—",
        ];
    for (const c of cells) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    if (!row.error && i === 0) tr.classList.add("mc-best");
    if (!row.error) {
      tr.classList.add("ml-lb-row");
      if (row === firstOkRow) tr.classList.add("ml-sel");
      tr.addEventListener("click", () => {
        for (const r of table.querySelectorAll(".ml-sel")) r.classList.remove("ml-sel");
        tr.classList.add("ml-sel");
        renderEvalDetail(detail, row, isClf);
      });
    }
    table.appendChild(tr);
  });

  host.appendChild(table);
  host.appendChild(detail);
  if (firstOkRow) renderEvalDetail(detail, firstOkRow, isClf);
}

function renderEvalDetail(host: HTMLElement, row: MlEvalRow, isClf: boolean): void {
  host.innerHTML = "";
  if (row.importances?.length) {
    const title = document.createElement("div");
    title.className = "mc-chain-note";
    title.textContent = `Permutation importance — ${row.algorithm} (${row.features.join(", ")})`;
    host.appendChild(title);
    const maxAbs = Math.max(1e-9, ...row.importances.map((v) => Math.abs(v)));
    row.features.forEach((f, i) => {
      const v = row.importances[i] ?? 0;
      const line = document.createElement("div");
      line.className = "ml-imp-row";
      const name = document.createElement("span");
      name.className = "ml-imp-name";
      name.textContent = f;
      const barWrap = document.createElement("div");
      barWrap.className = "ml-imp-bar-wrap";
      const bar = document.createElement("div");
      bar.className = "ml-imp-bar";
      bar.style.width = `${(Math.max(0, v) / maxAbs) * 100}%`;
      const val = document.createElement("span");
      val.className = "ml-imp-val";
      val.textContent = Number.isFinite(v) ? v.toFixed(4) : "—";
      barWrap.appendChild(bar);
      line.append(name, barWrap, val);
      host.appendChild(line);
    });
  }
  if (isClf && row.confusion && row.labels) {
    const cap = document.createElement("div");
    cap.className = "mc-chain-note";
    cap.textContent = "Confusion matrix (row = actual, col = predicted)";
    host.appendChild(cap);
    const t = document.createElement("table");
    t.className = "mc-table ml-confusion";
    const head = document.createElement("tr");
    head.appendChild(document.createElement("th"));
    for (const l of row.labels) {
      const th = document.createElement("th");
      th.textContent = String(l);
      head.appendChild(th);
    }
    t.appendChild(head);
    row.confusion.forEach((rowArr, r) => {
      const tr = document.createElement("tr");
      const rh = document.createElement("th");
      rh.textContent = String(row.labels?.[r] ?? r);
      tr.appendChild(rh);
      rowArr.forEach((n, c) => {
        const td = document.createElement("td");
        td.textContent = String(n);
        if (r === c) td.className = "ml-diag";
        tr.appendChild(td);
      });
      t.appendChild(tr);
    });
    host.appendChild(t);
  }
}
