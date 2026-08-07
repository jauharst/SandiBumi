import {
  applyMlModel,
  deleteMlModel,
  listCurveCatalog,
  listMlModels,
  listWells,
  mlDeterminismNote,
  mlModelWarnings,
  renameMlModel,
  runMl,
  runMlEval,
  type MlEvalResult,
  type MlEvalRow,
  type MlModelInfo,
  type MlRequest,
  type MlResult,
  type SplitBalance,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, defaultRunWellIds, filterByActiveGroup } from "../state";
import { buildLogSetPicker } from "./logSetPicker";
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

/** SB-MLA-009 — what a model or a curve says about how well it travels.
 *
 *  Written once by the fitting run (`ml.rs::blind_record`), stored in the model's metrics and on
 *  every curve's log-set record, and read here. `performed: false` carries NO value, deliberately:
 *  the whole point of the requirement is that a training score must never stand in for a blind one.
 *  The cautionary case is a delivered project where a predicted curve reached a training
 *  correlation of 0.99 against a blind-well range of 0.31–0.70. */
export interface BlindRecord {
  performed: boolean;
  metric?: string;
  value?: number;
  protocol?: string;
  /** True only where WHOLE WELLS were held back. A random-row split scores the model on depths
   *  centimetres from ones it was fitted on, so its number does not answer "will this work on the
   *  next well" — and the two must never be quoted as each other. */
  answers_new_well?: boolean;
  n_blind_wells?: number;
  n_blind_rows?: number;
  n_fit_rows?: number;
  seed?: number;
  why?: string;
}

/** One reader for every surface that shows a blind score, so the wording and the fallbacks cannot
 *  drift between the model list, the run panel and the report. */
export function readBlind(json: string | null | undefined): BlindRecord | null {
  if (!json) return null;
  try {
    const v = JSON.parse(json) as Record<string, unknown>;
    const b = (v.blind ?? null) as BlindRecord | null;
    return b && typeof b === "object" ? b : null;
  } catch {
    return null;
  }
}

/** SB-MLA-002 + SB-MLA-004 — one entry per well that actually contributed rows to the fit.
 *  Written by `ml.rs::assemble_training`; `trained_on` answers "which wells", this answers
 *  "which rock". */
export interface TrainWellRecord {
  well_id: string;
  well: string;
  rows: number;
  masked: number;
  incomplete: number;
  set_name?: string | null;
  set_id?: string | null;
  set_version?: number | null;
}

/** SB-MLA-004 — the run-level mask wrapped around the per-well roster. `mask_curve: null` means no
 *  mask was used, which is a different fact from a mask that was applied and flagged nothing (that
 *  reads as a name with every `masked` at zero, and an all-zero bad-hole flag across a field usually
 *  means the flag was never computed). Mirrors `ml.rs::TrainingRecord`. */
export interface TrainingRecord {
  mask_curve: string | null;
  wells: TrainWellRecord[];
}

function readTraining(json: string | null | undefined): TrainingRecord | null {
  if (!json) return null;
  try {
    const v = JSON.parse(json) as TrainingRecord;
    return v && typeof v === "object" && Array.isArray(v.wells) ? v : null;
  } catch {
    return null;
  }
}

function readRoster(json: string | null | undefined): TrainWellRecord[] {
  return readTraining(json)?.wells ?? [];
}

/** Which log set the training rows were read from. A model with no set was fitted from the CURRENT
 *  store, and that is weaker provenance rather than a missing field: the values can move under the
 *  model without anything changing name or version, so it is said in those words. */
export function describeTrainingSets(json: string | null | undefined): string {
  const roster = readRoster(json);
  if (roster.length === 0) return "not recorded (saved before this was kept)";
  const named = roster.filter((r) => r.set_name);
  if (named.length === 0) return "the current store — no frozen log set, so these values can move under the model";
  const sets = [...new Set(named.map((r) => `${r.set_name}${r.set_version ? ` v${r.set_version}` : ""}`))];
  const live = named.length === roster.length ? "" : ` (${roster.length - named.length} well(s) from the current store)`;
  return sets.join(", ") + live;
}

/** How much rock the run mask took out. Reported with the WORST well named: a mask that removed a
 *  fifth of the field evenly and one that emptied a single well are different situations with the
 *  same total. */
export function describeMaskEffect(json: string | null | undefined): string {
  const rec = readTraining(json);
  if (!rec || rec.wells.length === 0) return "not recorded";
  const roster = rec.wells;
  // The curve is named first, because "which flag was this" is the question a reviewer asks before
  // "how much did it take". An absent mask says so outright rather than reading as a missing field.
  if (!rec.mask_curve) return "no mask was applied";
  const masked = roster.reduce((a, r) => a + (r.masked || 0), 0);
  // A mask that flagged nothing is NOT the same as no mask: an all-zero bad-hole flag across a whole
  // field is usually a flag nobody computed, and that is worth reading as a fact about the run.
  if (masked === 0) return `${rec.mask_curve} — applied, and it excluded nothing`;
  const total = roster.reduce((a, r) => a + (r.rows || 0) + (r.masked || 0) + (r.incomplete || 0), 0);
  const worst = roster.reduce((a, r) => ((r.masked || 0) > (a.masked || 0) ? r : a), roster[0]);
  const pct = total > 0 ? Math.round((100 * masked) / total) : 0;
  return `${rec.mask_curve} — ${masked} of ${total} sample(s) excluded (${pct}%), most from ${worst.well} (${worst.masked})`;
}

/** SB-MLA-005 — the library set the artifact was written under. Falls back to the standalone
 *  scikit-learn column for models saved before the full record existed, rather than reading blank. */
export function describeRuntime(
  json: string | null | undefined,
  sklearnVersion: string | null | undefined,
): string {
  if (json) {
    try {
      const v = JSON.parse(json) as Record<string, string | null>;
      // A null is "probed, not installed" and is worth showing — for xgboost it says which estimator
      // actually fitted the model. Only a component that was never probed is left out.
      const parts = Object.entries(v).map(([name, ver]) => `${name} ${ver || "not installed"}`);
      if (parts.length > 0) return parts.sort().join(", ");
    } catch {
      /* fall through to the older single-version record */
    }
  }
  return sklearnVersion ? `scikit-learn ${sklearnVersion} (only this was recorded)` : "not recorded";
}

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
  // Results come back keyed by well id; every table the user reads shows the well's NAME.
  const wellNames = new Map(wells.map((w) => [w.well_id, w.well_name]));
  const nameOf = (id: string) => wellNames.get(id) ?? id;
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

  // SB-MLA-008. What about the chosen configuration would not reproduce on another machine, said
  // BEFORE the run rather than discovered when somebody cannot repeat the result. Hidden unless
  // there is something to say — today that is only the gbdt estimator substitution, and a line that
  // is present but empty most of the time teaches the eye to skip the place it appears.
  const detNote = document.createElement("div");
  detNote.className = "ml-determinism-note";
  detNote.hidden = true;
  content.appendChild(detNote);

  // --- Input curves + target ----------------------------------------------
  const featBox = document.createElement("div");
  featBox.className = "mc-wells";
  const featChecks = new Map<string, HTMLInputElement>();
  for (const c of catalog) {
    const label = document.createElement("label");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.value = c.name;
    cb.checked = DEFAULT_FEATURES.includes(c.name);
    featChecks.set(c.name, cb);
    // The unit and where the curve came from, beside the name. The list now carries every
    // IMPORTED log as well as the standard columns and the computed ones, so on a real delivery
    // it is long — and "which RHOB is this" is the first question a long list raises.
    const tail = document.createElement("span");
    tail.className = "mc-curve-src";
    tail.textContent = `${c.units ? ` ${c.units}` : ""} · ${c.source.toLowerCase()}`;
    label.append(cb, document.createTextNode(` ${c.name}`), tail);
    featBox.appendChild(label);
  }
  content.appendChild(
    formRow(
      "Input curves",
      featBox,
      "Tick any curve the model should learn from — standard columns, computed curves and imported logs alike. Order matters for clustering: class 0 = lowest mean of the FIRST checked curve (put GR first).",
    ),
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

  // --- Target transform (SB-MLA-035) --------------------------------------
  // Permeability spans decades and is fitted in log10 space because that is where the relation is
  // linear. What the model then predicts is log10(mD), NOT mD — so the choice is made here, in the
  // open, rather than by the user log-transforming a curve by hand and then forgetting which one
  // they did it to. Two curves come back: the model's own output under `<name>_LOG10` in log units,
  // and its back-transform under `<name>` in the target's own units.
  const xfSeg = document.createElement("div");
  xfSeg.className = "seg ml-target-xf";
  let targetXf: "" | "log10" = "";
  const xfBtns = new Map<string, HTMLButtonElement>();
  for (const [id, label, title] of [
    ["", "As measured", "Fit the target in the units it is stored in. Correct for anything roughly linear in the inputs — porosity, saturation, a sonic slowness."],
    ["log10", "log10", "Fit log10(target). The right choice for permeability and anything else spanning decades. Samples of zero or less have no logarithm and are dropped, and the count is reported."],
  ] as const) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.setAttribute("aria-pressed", String(id === ""));
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", () => {
      targetXf = id;
      for (const [k, el] of xfBtns) el.setAttribute("aria-pressed", String(k === id));
      echoTransform();
    });
    xfBtns.set(id, b);
    xfSeg.appendChild(b);
  }
  const xfEcho = document.createElement("div");
  xfEcho.className = "mc-chain-note";
  /** Name the two curves BEFORE the run, because after it the names are the only warning left. */
  function echoTransform(): void {
    const base = (outInput.value || "PRED").trim().toUpperCase();
    xfEcho.textContent = targetXf
      ? `Writes ${base}_LOG10 (the model's own output, in log units) and ${base} (its back-transform, in the target's units). Scores are reported in log space.`
      : "";
  }
  const xfRow = formRow(
    "Fit target as", xfSeg,
    "A transformed quantity is a different quantity. Whichever space the model is fitted in is the space its scores describe.",
  );
  content.append(xfRow, xfEcho);

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

  // --- Blind test split ----------------------------------------------------
  // The percentage is a share of the DATA — of the samples these wells actually gave, not of the
  // well count (Jauhar, 2026-08-07). But what gets held back is still a WHOLE WELL: splitting
  // pooled samples would put consecutive depths from one well on both sides of the line, and the
  // model would be scored on rock it saw a few centimetres away.
  //
  // So the control asks for a target and cannot promise to hit it — whole wells are lumpy, and how
  // many samples each well will contribute is not known until the curves are read and masked. The
  // echo therefore says what will be AIMED at; the result panel says what was reached.
  const splitWrap = document.createElement("div");
  splitWrap.className = "ml-split-ctl";
  const splitOn = document.createElement("input");
  splitOn.type = "checkbox";
  const splitOnLabel = document.createElement("label");
  splitOnLabel.append(splitOn, document.createTextNode(" Hold wells back as a blind test"));
  const splitPct = document.createElement("input");
  splitPct.type = "number";
  splitPct.min = "5";
  splitPct.max = "80";
  splitPct.step = "5";
  splitPct.value = "30";
  splitPct.disabled = true;
  const splitSeed = document.createElement("input");
  splitSeed.type = "number";
  splitSeed.min = "0";
  splitSeed.step = "1";
  splitSeed.value = "42";
  splitSeed.disabled = true;
  splitSeed.title = "Which wells are chosen. Same seed, same wells — so a blind score can be quoted and re-run.";
  const splitEcho = document.createElement("div");
  splitEcho.className = "ml-split-echo";
  const splitFields = document.createElement("div");
  splitFields.className = "ml-split-fields";
  const pctLab = document.createElement("label");
  pctLab.append(splitPct, document.createTextNode(" % of samples held blind"));
  const seedLab = document.createElement("label");
  seedLab.append(document.createTextNode("seed "), splitSeed);

  // The two modes answer different questions and neither is a better version of the other, so this
  // is a choice rather than a default with an override. `.seg`/`.seg-opt` is the app's segmented
  // control (Organic increment 2) — the same component the Field Dashboard's Flag/Metric pills use.
  const modeSeg = document.createElement("div");
  modeSeg.className = "seg ml-split-mode";
  let splitMode: "well" | "sample" = "well";
  const modeBtns = new Map<string, HTMLButtonElement>();
  for (const [id, label, title] of [
    ["well", "Whole wells", "Hold back entire wells. Answers: will this model work on the next well I drill? Cannot leak."],
    ["sample", "Random rows", "Draw individual samples, stratified so the blind set carries the same distribution. Exact percentage; optimistic on log data, because the depth above and below a held-out sample are usually in the fit set."],
  ] as const) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    // `aria-pressed` is what `.seg-opt` styles off, not a class — the convention the Field
    // Dashboard's pills already set. One selected-state mechanism, not two.
    b.setAttribute("aria-pressed", String(id === "well"));
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", () => {
      splitMode = id;
      for (const [k, el] of modeBtns) el.setAttribute("aria-pressed", String(k === id));
      echoSplit();
    });
    modeBtns.set(id, b);
    modeSeg.appendChild(b);
  }
  const modeLab = document.createElement("label");
  modeLab.append(document.createTextNode("held back as "), modeSeg);

  splitFields.append(pctLab, modeLab, seedLab);
  splitWrap.append(splitOnLabel, splitFields, splitEcho);

  /** Say what the percentage is aiming at, and what it CANNOT promise, before the run.
   *
   *  Deliberately no predicted well count. The old echo mirrored `split_blind_wells` and printed
   *  "3 wells fitted, 2 held blind"; that arithmetic no longer exists here, because the answer now
   *  depends on how many usable samples each well contributes — which is known only after the
   *  curves are read and the mask applied. A number the frontend guessed and the backend then
   *  contradicted would be worse than no number.
   */
  function echoSplit(): void {
    const n = [...train.checks.values()].filter((c) => c.checked).length;
    if (!splitOn.checked) {
      splitEcho.textContent = n
        ? `All ${n} training well(s) are fitted on. The only validation is cross-validation over folds of those wells.`
        : "";
      splitEcho.classList.remove("ml-split-thin");
      return;
    }
    if (n < 2) {
      splitEcho.textContent = "A split needs at least 2 training wells — a single well cannot be divided by well.";
      splitEcho.classList.add("ml-split-thin");
      return;
    }
    const pct = Math.max(0, Math.min(100, Number(splitPct.value) || 0));
    if (splitMode === "sample") {
      splitEcho.textContent =
        `SandiBumi draws exactly ${pct}% of the pooled samples at random from all ${n} wells, stratified on the ` +
        `target so the blind set carries the same distribution as the whole. Every well is on both sides. ` +
        `This scores optimistically on log data — the depths either side of a held-out sample are usually in ` +
        `the fit set — so read it beside the cross-validation score, which stays grouped by well.`;
      splitEcho.classList.add("ml-split-thin");
      return;
    }
    splitEcho.textContent =
      `SandiBumi picks whole wells from the ${n} selected until about ${pct}% of their pooled samples are held back. ` +
      `Whole wells rarely divide the data exactly — the share actually reached is reported with the score.` +
      (n < 4 ? " With few wells the steps are coarse, so expect to land some way off." : "");
    splitEcho.classList.toggle("ml-split-thin", n < 4);
  }
  splitOn.addEventListener("change", () => {
    splitPct.disabled = splitSeed.disabled = !splitOn.checked;
    for (const el of modeBtns.values()) el.disabled = !splitOn.checked;
    echoSplit();
  });
  for (const el of modeBtns.values()) el.disabled = true;
  splitPct.addEventListener("input", echoSplit);
  for (const cb of train.checks.values()) cb.addEventListener("change", echoSplit);

  const splitRow = formRow(
    "Blind test",
    splitWrap,
    "Wells kept out of the fit and used to score it. They still get their predicted curve, so you can lay it against core.",
  );
  content.appendChild(splitRow);
  echoSplit();

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
  outInput.addEventListener("input", () => {
    outEdited = true;
    // The transform echo names the two curves it will write, so it has to follow the name.
    echoTransform();
  });
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

  // --- Input / output log set (`logSetPicker.ts`). A model fitted on today's PHIE and one fitted
  // after the next porosity re-run are fitted on different rock; naming the set is what lets a
  // saved model say which (Jauhar, 2026-08-05). Output defaults to ML, which is where every
  // prediction went before this was selectable.
  const setPicker = buildLogSetPicker({ write: "ML" });
  for (const row of setPicker.rows) content.appendChild(row);

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
    // Clustering and reduction are fitted on the very wells they are applied to, so there is no
    // "held out" to be had — offering the control there would promise a validation that cannot exist.
    splitRow.style.display = task.supervised ? "" : "none";
    compareRow.style.display = task.supervised ? "" : "none";
    // Only a supervised fit is a reusable artifact. Clustering and reduction are fitted on the
    // very wells they are applied to, so "apply it later" would mean something different.
    saveRow.style.display = task.supervised ? "" : "none";
    // Only a continuous target has a logarithm. Offered on a classifier the control would be an
    // invitation to a refusal, and offered on clustering it would name a target that does not exist.
    const canTransform = task.id === "regression";
    xfRow.style.display = canTransform ? "" : "none";
    xfEcho.style.display = canTransform ? "" : "none";
    if (!canTransform && targetXf) {
      targetXf = "";
      for (const [k, el] of xfBtns) el.setAttribute("aria-pressed", String(k === ""));
    }
    if (!outEdited) outInput.value = algo.out ?? task.defaultOut;
    echoTransform();
    renderParams();
    // Asked per selection rather than once: the answer depends on which algorithm is chosen, and
    // the backend caches the runtime probe, so every call after the first is free. A generation
    // counter drops a stale answer — the user can change the algorithm faster than the round trip.
    const gen = ++detGen;
    detNote.hidden = true;
    void mlDeterminismNote(task.id, algo.id)
      .then((note) => {
        if (gen !== detGen) return;
        detNote.textContent = note ?? "";
        detNote.hidden = !note;
      })
      .catch(() => {
        /* a probe that could not run is not a claim that the run is non-deterministic */
      });
  }
  let detGen = 0;

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

  // --- Keep the fitted model ------------------------------------------------
  // Until now the fit died with the subprocess: you could not train on the cored wells and
  // apply THAT SAME model to the rest of the field later. Naming it here makes it an artifact
  // a delivered curve can cite.
  const saveInput = document.createElement("input");
  saveInput.className = "form-control";
  saveInput.placeholder = "leave blank to not keep the model";
  const saveRow = formRow(
    "Save model as",
    saveInput,
    "Keeps the fitted model (and its scaler) so it can be applied to other wells later, without refitting",
  );
  content.appendChild(saveRow);

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
        input_set: setPicker.inputSet(),
        algorithms: task.algos.map((a) => a.id),
        // The settings currently on screen, and the algorithm they belong to. The leaderboard must
        // rank the model the run will fit, not the same id at library defaults.
        params: Object.fromEntries(paramInputs.map(({ spec, get }) => [spec.key, get()])),
        params_for: algo.id,
        subsets: buildSubsets(features, subsetSel.value),
        standardize: stdCb.checked,
        seed: Math.round(parseFloat(seedInput.value) || 42),
        folds: 5,
        // The leaderboard has to rank the model the run will fit. Ranked in linear space while the
        // run fits log10, the table would recommend a different winner than the one that wins.
        target_transform: task.id === "regression" && targetXf ? targetXf : null,
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

  // --- Saved models ---------------------------------------------------------
  // A trained model is a named, dated, citable artifact here: apply it to new wells without
  // refitting, because a refit on different data is a different model.
  const savedWrap = document.createElement("div");
  savedWrap.className = "mc-section";
  const savedHead = document.createElement("h4");
  savedHead.textContent = "Saved models";
  const savedList = document.createElement("div");
  savedList.className = "mc-saved-list";
  const savedNote = document.createElement("div");
  savedNote.className = "mc-chain-note";
  savedWrap.append(savedHead, savedList, savedNote);
  content.appendChild(savedWrap);

  // SB-MLA-002 + SB-MLA-005. Fetched alongside the list rather than after it: the first call spawns
  // the runtime probe, and running the two in flight together means the list appears at the speed of
  // the slower one instead of their sum. A failure here must not take the model list down — a picker
  // that shows nothing because a warning could not be computed is worse than one showing no warning.
  let warnings = new Map<string, string[]>();

  const refreshSaved = async (): Promise<void> => {
    let models: MlModelInfo[];
    try {
      const [list, warned] = await Promise.all([listMlModels(), mlModelWarnings().catch(() => [])]);
      models = list;
      warnings = new Map(warned.map((w) => [w.model_id, w.notes]));
    } catch (e) {
      savedNote.textContent = `Could not list saved models: ${e}`;
      return;
    }
    savedList.innerHTML = "";
    if (models.length === 0) {
      savedNote.textContent =
        "None yet. Run a supervised model with a name in “Save model as”, then apply it here to wells it has never seen.";
      return;
    }
    savedNote.textContent =
      "Applying uses the model's OWN input curves, in the order it was fitted on — a well missing one is reported by name rather than predicted from the wrong columns.";
    for (const m of models) {
      const row = document.createElement("div");
      row.className = "mc-saved-row";
      const desc = document.createElement("div");
      desc.className = "mc-saved-desc";
      const mb = m.bytes >= 1024 * 1024 ? `${(m.bytes / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(m.bytes / 1024))} kB`;
      desc.textContent =
        `${m.name} — ${m.algorithm} ${m.task}` +
        (m.target_curve ? ` → ${m.target_curve}` : "") +
        `  ·  ${m.feature_curves.join(", ")}` +
        `  ·  ${m.n_train} samples from ${m.trained_on.length} well(s)  ·  ${m.created_at}  ·  ${mb}`;
      desc.title =
        `Trained on: ${m.trained_on.join(", ") || "—"}\n` +
        `Standardized: ${m.standardize ? "yes (the scaler is stored with the model)" : "no"}\n` +
        // SB-MLA-003. The wells and the count do not pin a re-run: the same wells at a later log-set
        // version are different rows with the same names and often the same count.
        `Training rows: ${m.train_hash ?? "not recorded (saved before this was kept)"}\n` +
        // SB-MLA-002 + SB-MLA-004. Which log set the rows were read from, and how much rock the
        // mask took out — the two facts a re-run has to match and neither of which is in the
        // well list.
        `Read from: ${describeTrainingSets(m.training_json)}\n` +
        `Mask: ${describeMaskEffect(m.training_json)}\n` +
        // SB-MLA-005. The artifact is a pickle, so it is loadable only under a compatible set —
        // and joblib, the component that actually unpickles it, is the one nobody thinks of.
        `Runtime: ${describeRuntime(m.runtime_json, m.sklearn_version)}`;

      // SB-MLA-009. How well the model travels, stated on the row you pick it from — because this
      // is the moment somebody decides to apply it to fifty wells they have no core in. A model
      // that was never blind-tested says so; it must never show a training score here instead.
      const blindTag = document.createElement("span");
      blindTag.className = "ml-blind-tag";
      const b = readBlind(m.metrics_json);
      if (b?.performed) {
        const v = typeof b.value === "number" ? b.value.toFixed(2) : "—";
        blindTag.textContent = `blind ${b.metric ?? ""} ${v}`.trim();
        blindTag.dataset.grade = typeof b.value === "number" ? (b.value >= 0.7 ? "good" : b.value >= 0.4 ? "fair" : "weak") : "none";
        blindTag.title =
          `Measured on ${b.n_blind_rows ?? "?"} row(s) in ${b.n_blind_wells ?? 0} well(s) the model was not fitted on, held back as ${b.protocol ?? "?"}.\n` +
          (b.answers_new_well
            ? "Whole wells were held back, so this is what the model does on a well it has never seen."
            : "Rows were drawn from wells the model also trained on, so this says the relationship is learnable here — not that it travels to a new well.");
      } else {
        blindTag.textContent = "not blind-tested";
        blindTag.dataset.grade = "none";
        blindTag.title =
          "This model was fitted without holding anything back, so there is no measurement of how it performs on data it has not seen. " +
          "Its training score is not that measurement and is deliberately not shown here.";
      }
      // SB-MLA-002 + SB-MLA-005. Only where something has actually MOVED under the model — a badge
      // on every row would be a badge nobody reads, and the one row that matters would be lost in a
      // column of reassurance. The wording comes from the backend, so this row and the run result
      // that follows it cannot describe the same problem two different ways.
      const notes = warnings.get(m.model_id) ?? [];
      let driftTag: HTMLSpanElement | null = null;
      if (notes.length > 0) {
        driftTag = document.createElement("span");
        driftTag.className = "ml-drift-tag";
        driftTag.textContent = notes.length > 1 ? `${notes.length} warnings` : "has drifted";
        driftTag.title = notes.join("\n\n");
      }

      const applyBtn = document.createElement("button");
      applyBtn.type = "button";
      applyBtn.textContent = "Apply to scope";
      const renameBtn = document.createElement("button");
      renameBtn.type = "button";
      renameBtn.textContent = "Rename";
      const delBtn = document.createElement("button");
      delBtn.type = "button";
      delBtn.textContent = "Delete";
      row.append(desc, blindTag);
      if (driftTag) row.appendChild(driftTag);
      row.append(applyBtn, renameBtn, delBtn);
      savedList.appendChild(row);

      applyBtn.addEventListener("click", async () => {
        const applyIds = scope.getWellIds();
        if (applyIds.length === 0) {
          setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
          return;
        }
        applyBtn.disabled = true;
        statusLine.textContent = `Applying '${m.name}' to ${applyIds.length} well(s)…`;
        try {
          const res: MlResult = await applyMlModel({
            model_id: m.model_id,
            apply_well_ids: applyIds,
            output_curve: outInput.value,
            input_set: setPicker.inputSet(),
            output_set: setPicker.outputSet(),
            mask_curve: maskSel.value || null,
          });
          if (res.error) {
            statusLine.textContent = `Failed: ${res.error}`;
          } else {
            const total = res.wells.length || applyIds.length;
            const ok = res.wells.filter((w) => !w.error).length;
            const outs = res.outputs.join(", ");
            statusLine.textContent =
              `Applied '${m.name}' → ${outs}` + (ok < total ? ` — ${total - ok} well(s) need attention` : "");
            if (ok > 0) {
              setStatus(`Applied model ${m.name}: wrote ${outs} to ${ok}/${total} well(s)`);
              recordProcess("ML", `Applied saved model ${m.name}: wrote ${outs} to ${ok}/${total} well(s)`);
              bumpDataVersion();
            }
          }
          renderResults(results, res, nameOf);
        } catch (e) {
          statusLine.textContent = `Failed: ${e}`;
        } finally {
          applyBtn.disabled = false;
        }
      });

      renameBtn.addEventListener("click", async () => {
        const next = window.prompt("New name for this model", m.name);
        if (!next || next.trim() === m.name) return;
        try {
          const stored = await renameMlModel(m.model_id, next.trim());
          savedNote.textContent = stored === next.trim() ? "" : `Name in use — saved as '${stored}'.`;
          await refreshSaved();
        } catch (e) {
          savedNote.textContent = `Rename failed: ${e}`;
        }
      });

      delBtn.addEventListener("click", async () => {
        // A deleted model cannot be rebuilt from the curves it produced, so confirm by name.
        if (!window.confirm(`Delete the saved model '${m.name}'?\n\nCurves it already produced are kept, but they can no longer be reproduced from this model.`)) {
          return;
        }
        try {
          await deleteMlModel(m.model_id);
          await refreshSaved();
        } catch (e) {
          savedNote.textContent = `Delete failed: ${e}`;
        }
      });
    }
  };
  void refreshSaved();

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
      save_model_as: task.supervised && saveInput.value.trim() ? saveInput.value.trim() : null,
      blind_fraction: task.supervised && splitOn.checked ? Number(splitPct.value) / 100 : null,
      split_seed: task.supervised && splitOn.checked ? Number(splitSeed.value) || 0 : null,
      split_mode: task.supervised && splitOn.checked ? splitMode : null,
      target_transform: task.id === "regression" && targetXf ? targetXf : null,
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
        // Count what actually landed. `applyIds.length` is the SCOPE, not the outcome: a well
        // with no usable feature curves comes back carrying an error and gets no curve, and one
        // with no complete samples gets an all-NaN curve — both flagged per-well by the backend.
        // Reporting the scope count made the status line and, worse, the permanent History entry
        // claim every well was written. moduleDialog already reports "ok/total"; match it.
        const total = res.wells.length || applyIds.length;
        const ok = res.wells.filter((w) => !w.error).length;
        const outs = res.outputs.join(", ");
        const scope = ok === total ? `${total} well(s)` : `${ok}/${total} well(s)`;
        const needAttention = total - ok;
        statusLine.textContent =
          `Done in ${ms} ms → ${outs}` +
          (needAttention > 0 ? ` — ${needAttention} well(s) need attention` : "");
        setStatus(`${algo.label}: wrote ${outs} to ${scope}`);
        // A run that wrote nothing is not a process worth recording as if it had succeeded.
        if (ok > 0) {
          recordProcess("ML", `${algo.label}: wrote ${outs} to ${scope}`);
        }
        if (res.model_name) {
          statusLine.textContent += ` · model saved as '${res.model_name}'`;
          recordProcess("ML", `Saved model '${res.model_name}' (${algo.label} on ${req.target_curve ?? "-"})`);
          saveInput.value = "";
          void refreshSaved();
        }
        bumpDataVersion(); // ML wrote curves — refresh open plots/log views/catalog
      }
      renderResults(results, res, nameOf);
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

interface EffectiveParam {
  value: unknown;
  defaulted?: boolean;
  source?: string;
  used?: unknown;
}

/**
 * What the run actually used — every parameter, defaults included and marked as defaults.
 *
 * `SB-MLA-001`. The panel used to show the settings you typed, which is the one set of numbers
 * that needs no reporting: you have them. The values that decide a result and are NOT on screen
 * anywhere are the ones nobody supplied — `seed` above all, which chooses the clustering you got
 * out of the several the data supports. A record you cannot re-run from is not a record.
 *
 * Defaulted rows are marked rather than hidden: the difference between "I chose 200 trees" and
 * "something chose 200 trees for me" is the difference between a decision and an accident, and
 * six months later they look identical in a report.
 */
function renderEffectiveParams(host: HTMLElement, metrics: Record<string, unknown>): void {
  const eff = metrics["effective_params"];
  if (!eff || typeof eff !== "object") return;
  const entries = Object.entries(eff as Record<string, EffectiveParam>).filter(
    ([, v]) => v && typeof v === "object" && "value" in v,
  );
  if (!entries.length) return;

  const box = document.createElement("details");
  box.className = "ml-eff";
  const sum = document.createElement("summary");
  const nDef = entries.filter(([, v]) => v.defaulted).length;
  sum.textContent = nDef
    ? `Settings this run actually used — ${entries.length} parameter(s), ${nDef} defaulted`
    : `Settings this run actually used — ${entries.length} parameter(s), all supplied`;
  box.appendChild(sum);

  const t = document.createElement("table");
  t.className = "mc-table ml-eff-table";
  const head = document.createElement("tr");
  for (const h of ["Parameter", "Value", "Where it came from"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  t.appendChild(head);
  // Defaulted first: they are the rows the user has not seen anywhere else.
  const sorted = [...entries].sort((a, b) => Number(!!b[1].defaulted) - Number(!!a[1].defaulted));
  for (const [key, v] of sorted) {
    const tr = document.createElement("tr");
    if (v.defaulted) tr.classList.add("ml-eff-defaulted");
    const kd = document.createElement("td");
    kd.textContent = key;
    const vd = document.createElement("td");
    // A clamped value states both numbers: a request the code narrowed is a parameter the
    // record would otherwise misstate (t-SNE perplexity against a small sample count).
    vd.textContent =
      v.used !== undefined && v.used !== v.value ? `${String(v.used)} (asked for ${String(v.value)})` : String(v.value);
    const sd = document.createElement("td");
    sd.className = "ml-eff-src";
    sd.textContent = v.defaulted ? `default — ${v.source ?? "unrecorded"}` : "you set this";
    tr.append(kd, vd, sd);
    t.appendChild(tr);
  }
  box.appendChild(t);
  host.appendChild(box);
}

/**
 * The blind split, and the three scores side by side.
 *
 * An experienced eye does not read `r2_train` — it reads the GAP between train and blind. A model
 * at 0.98 on the wells it was fitted on and 0.41 on the wells it was not has memorised those
 * wells, and either number quoted alone hides that. So the three are shown together, labelled by
 * what they are a score OF, with the gap called out when it is large.
 *
 * The wells are named, not counted. "70% held out" is not an answer to "which wells?", and a
 * blind score is a claim about specific rock.
 */
function renderSplit(host: HTMLElement, res: MlResult, nameOf?: (id: string) => string): void {
  const sp = res.split;
  if (!sp) return;
  const m = (res.metrics ?? {}) as Record<string, unknown>;
  const num = (k: string): number | null => {
    const v = m[k];
    return typeof v === "number" && Number.isFinite(v) ? v : null;
  };
  const name = (id: string) => nameOf?.(id) ?? id;

  const box = document.createElement("div");
  box.className = "ml-split-report";

  const askedPct = sp.requested_fraction * 100;
  const gotPct = sp.achieved_fraction * 100;
  const bySample = sp.mode === "sample";
  const head = document.createElement("div");
  head.className = "ml-split-head";
  head.textContent =
    `Blind test — ${gotPct.toFixed(1)}% of the data held back, ` +
    (bySample ? "drawn as random rows" : "held back as whole wells") +
    ` (asked for ${Math.round(askedPct)}%, seed ${sp.seed})`;
  box.appendChild(head);

  for (const [label, ids, rows, cls] of [
    ["Fitted on", sp.fit_wells, sp.fit_rows, "ml-split-fit"],
    ["Held blind", sp.blind_wells, sp.blind_rows, "ml-split-blind"],
  ] as const) {
    const row = document.createElement("div");
    row.className = `ml-split-row ${cls}`;
    const k = document.createElement("span");
    k.className = "ml-split-key";
    k.textContent = label;
    const v = document.createElement("span");
    // Samples beside the names, because the names alone do not say how much rock this is — two
    // wells can be a third of the field or a twentieth of it. In sample mode there are no names
    // to print (every well is on both sides), so the row count carries the whole answer.
    v.textContent = ids.length
      ? `${ids.map(name).join(", ")} — ${rows.toLocaleString()} samples`
      : bySample
        ? `${rows.toLocaleString()} samples, drawn from all ${sp.wells_pooled} well(s)`
        : "—";
    row.append(k, v);
    box.appendChild(row);
  }

  if (bySample) {
    // Not a warning about a mistake — a label on what the number means. The user chose this mode
    // knowing what it does; what they must not do is quote the score without the qualifier.
    const g = document.createElement("div");
    g.className = "ml-split-gap ml-split-gap-warn";
    g.textContent =
      "A row drawn blind usually has the depth above and below it in the fit set, so this score is optimistic — " +
      "it says the model learned the relationship in these wells, not that it will hold in the next one. " +
      "The cross-validation row below is still grouped by well and does not have that problem; read the two together.";
    box.appendChild(g);
  } else {
    // Whole wells are lumpy, so the request is a target and missing it is normal. Say so when the
    // miss is big enough to change what the score means — silently printing the achieved number
    // beside the requested one would leave the user to notice the gap themselves.
    const miss = Math.abs(gotPct - askedPct);
    if (miss >= 5) {
      const g = document.createElement("div");
      g.className = miss >= 15 ? "ml-split-gap ml-split-gap-warn" : "ml-split-gap";
      g.textContent =
        `Whole wells could not divide these samples at ${Math.round(askedPct)}% — the nearest reachable split holds ` +
        `${gotPct.toFixed(1)}%. Wells are held back whole so the model is never scored on rock it saw a few centimetres away; ` +
        `that is what makes the share coarse.`;
      box.appendChild(g);
    }
  }

  // Regression and classification report different things; show whichever pair exists.
  const isClf = num("accuracy_train") != null;
  const trainV = isClf ? num("accuracy_train") : num("r2_train");
  const cvV = isClf ? num("accuracy_cv") : num("r2_cv");
  const blindV = isClf ? num("accuracy_blind") : num("r2_blind");
  const unit = isClf ? "accuracy" : "R²";

  // SB-MLA-035. Every score below was computed in the space the model was fitted in, and an R² in
  // log space is not the same claim as an R² in mD — it is usually the lower of the two, because
  // the log fit is not being rewarded for getting the few largest values roughly right. So the
  // space is stated once, above the numbers, rather than left for the reader to remember.
  const space = typeof m.metric_space === "string" ? m.metric_space : null;
  if (space) {
    const sl = document.createElement("div");
    sl.className = "ml-split-gap";
    sl.textContent = `Scored in ${space} — the space the model was fitted in. Not comparable with a score on the untransformed target.`;
    box.appendChild(sl);
  }

  const scores = document.createElement("table");
  scores.className = "mc-table ml-score-table";
  // Mode-aware, because the same row means a different thing in each and the wrong wording is a
  // false claim rather than a clumsy one: a sample split never held out a well, so calling its
  // score a score on "blind wells" states exactly what did not happen.
  const blindWells = num("n_blind_wells");
  const rows: [string, number | null, string][] = [
    [`${unit} on the rows it was fitted on`, trainV, "in-sample — always the flattering one"],
    [`${unit} in cross-validation`, cvV, String(m[isClf ? "accuracy_cv_folds" : "r2_cv_folds"] ?? "folds of the fitted wells")],
    [
      bySample ? `${unit} on the blind rows` : `${unit} on the blind wells`,
      blindV,
      bySample
        ? `${num("n_blind") ?? 0} rows drawn from wells the model also trained on`
        : `${num("n_blind") ?? 0} samples in ${blindWells ?? 0} well(s) the model never saw`,
    ],
  ];
  for (const [label, v, note] of rows) {
    if (v == null) continue;
    const tr = document.createElement("tr");
    const th = document.createElement("th");
    th.textContent = label;
    const td = document.createElement("td");
    td.textContent = v.toFixed(4);
    const nd = document.createElement("td");
    nd.className = "ml-score-note";
    nd.textContent = note;
    tr.append(th, td, nd);
    scores.appendChild(tr);
  }
  box.appendChild(scores);

  if (trainV != null && blindV != null) {
    const gap = trainV - blindV;
    const warn = gap > 0.15;
    const g = document.createElement("div");
    g.className = warn ? "ml-split-gap ml-split-gap-warn" : "ml-split-gap";
    // In sample mode a small gap says nothing about travelling to a new WELL — the blind rows came
    // from the same wells. Claiming otherwise would be the single most misleading sentence on the
    // panel, so the two modes get different readings of the same number.
    g.textContent = warn
      ? bySample
        ? `The model scores ${gap.toFixed(3)} better on the rows it was fitted on than on the rows it was not — and both came from the same wells. A gap this size on a within-well split usually means memorisation.`
        : `The model scores ${gap.toFixed(3)} better on the wells it was fitted on than on the wells it was not. That gap is the part of the fit that does not travel.`
      : bySample
        ? `Fitted and blind rows agree to within ${Math.abs(gap).toFixed(3)}. That says the model learned the relationship present in these wells — not that it travels to a new one. Compare the cross-validation row for that.`
        : `Train and blind agree to within ${Math.abs(gap).toFixed(3)} — the fit travels to wells it has not seen.`;
    box.appendChild(g);
  }
  if (sp.blind_wells.length === 1) {
    const thin = document.createElement("div");
    thin.className = "ml-split-gap ml-split-gap-warn";
    thin.textContent = "One blind well is one opinion. It says the model is not broken; it does not say the score is stable.";
    box.appendChild(thin);
  }

  // "Similar statistics" is a claim, so it is evidenced rather than asserted. A stratified draw is
  // SUPPOSED to make these match — so a row that does not match is the useful one: it means that
  // stratum was too thin to divide representatively, and the blind score leans on it.
  const bal = Array.isArray(m.split_balance) ? (m.split_balance as SplitBalance[]) : null;
  if (bal && bal.length) {
    const cap = document.createElement("div");
    cap.className = "ml-split-head ml-balance-head";
    cap.textContent = "How alike the two sides are";
    box.appendChild(cap);

    const t = document.createElement("table");
    t.className = "mc-table ml-balance-table";
    const hr = document.createElement("tr");
    for (const h of ["", "fitted mean", "blind mean", "difference"]) {
      const th = document.createElement("th");
      th.textContent = h;
      hr.appendChild(th);
    }
    t.appendChild(hr);
    for (const b of bal) {
      // Scaled by the fitted side's own spread, because a 0.02 gap is nothing on GR and everything
      // on porosity — an absolute difference cannot be compared across curves.
      const sd = Math.max(Math.abs(b.fit_sd), 1e-9);
      const z = Math.abs(b.fit_mean - b.blind_mean) / sd;
      const tr = document.createElement("tr");
      if (z > 0.25) tr.className = "ml-balance-off";
      const cells = [
        b.name,
        b.fit_mean.toPrecision(4),
        b.blind_mean.toPrecision(4),
        `${z < 0.005 ? "<0.01" : z.toFixed(2)} sd`,
      ];
      cells.forEach((c, i) => {
        const el = document.createElement(i === 0 ? "th" : "td");
        el.textContent = c;
        tr.appendChild(el);
      });
      t.appendChild(tr);
    }
    box.appendChild(t);

    const worst = Math.max(
      ...bal.map((b) => Math.abs(b.fit_mean - b.blind_mean) / Math.max(Math.abs(b.fit_sd), 1e-9)),
    );
    const note = document.createElement("div");
    note.className = worst > 0.25 ? "ml-score-note ml-split-gap-warn" : "ml-score-note";
    note.textContent =
      worst > 0.25
        ? `The two sides differ by up to ${worst.toFixed(2)} standard deviations on one input — the blind set is not a ` +
          `representative sample of the whole, so its score is partly a statement about which rows happened to be drawn.`
        : "Every input and the target agree between the two sides to well within a quarter of a standard deviation — " +
          "the blind set is a representative sample of the whole.";
    box.appendChild(note);
  }
  host.appendChild(box);
}

/**
 * Per-well outcome of a run.
 *
 * A refused well is a RESULT, not a footnote. Since SB-MLA-013 a well that could not be labelled
 * or predicted fails by name instead of quietly receiving an all-missing curve, so this table is
 * now the only place the user learns that 3 of their 40 wells produced nothing — and why. It
 * therefore leads with the count, puts the refused wells FIRST, and names them the way the Wells
 * pane does. A UUID in this column is a well nobody can go and look at.
 *
 * Exported for the same reason as `renderLeaderboard` below — driving it with synthetic wells over
 * the vite dev server is this repo's only way to see that a refused well actually reads as refused.
 */
export function renderResults(host: HTMLElement, res: MlResult, nameOf?: (id: string) => string): void {
  host.innerHTML = "";
  if (res.error) return;

  // Backend advisories (e.g. training wells that contributed no usable samples) — shown at the
  // top so a partially-degraded run can't read as a clean one from the metrics alone.
  for (const note of res.notes ?? []) {
    const warn = document.createElement("div");
    warn.className = "mc-note mc-note-err";
    warn.textContent = `⚠ ${note}`;
    host.appendChild(warn);
  }

  if (res.split) renderSplit(host, res, nameOf);

  if (res.metrics && typeof res.metrics === "object") {
    renderEffectiveParams(host, res.metrics as Record<string, unknown>);
    const table = document.createElement("table");
    table.className = "mc-table";
    for (const [key, value] of Object.entries(res.metrics)) {
      if (key === "effective_params") continue; // shown as its own table above
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

  const refused = res.wells.filter((w) => w.error);
  if (res.wells.length) {
    const tally = document.createElement("div");
    tally.className = refused.length ? "ml-tally ml-tally-warn" : "ml-tally";
    const written = res.wells.length - refused.length;
    const rows = res.wells.reduce((a, w) => a + w.rows_predicted, 0);
    tally.textContent = refused.length
      ? `${written} of ${res.wells.length} wells written (${rows.toLocaleString()} samples) — ${refused.length} refused, listed first below`
      : `${written} well(s) written — ${rows.toLocaleString()} samples`;
    host.appendChild(tally);
  }

  const wellsTable = document.createElement("table");
  wellsTable.className = "mc-table ml-well-table";
  const head = document.createElement("tr");
  for (const h of ["Well", "Samples", "Outcome"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  wellsTable.appendChild(head);
  // Refused wells first: they are the rows that need a decision, and on a field-scale run they
  // would otherwise be a handful of lines somewhere inside two hundred.
  for (const w of [...refused, ...res.wells.filter((w) => !w.error)]) {
    const tr = document.createElement("tr");
    if (w.error) tr.classList.add("ml-well-refused");
    const name = nameOf?.(w.well_id) ?? w.well_id;
    for (const c of [name, w.rows_predicted ? w.rows_predicted.toLocaleString() : "—", w.error ?? "written"]) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    tr.title = w.well_id;
    wellsTable.appendChild(tr);
  }
  host.appendChild(wellsTable);
}

/**
 * How many leading rows are statistically indistinguishable from the top one.
 *
 * A leaderboard sorted by score always has a first row, which is not the same as having a winner.
 * Each score is a mean over folds and carries `score_std` across those folds, so two rows separated
 * by less than their combined spread are one result reported twice. Crowning the first of them is
 * SB-CORE-002 in a table: a degraded answer presented as a clean one.
 *
 * The test is deliberately the crude one — a gap smaller than the sum of the two spreads — rather
 * than a paired t-test. Folds are wells, there are rarely more than a handful, and a test that
 * needs assumptions the data cannot support would be a second false precision on top of the first.
 * Returns 1 when the lead is real.
 */
function tiedAtTheTop(rows: MlEvalRow[]): number {
  const ok = rows.filter((r) => !r.error && r.score != null);
  if (ok.length < 2) return ok.length;
  const lead = ok[0];
  let n = 1;
  for (let i = 1; i < ok.length; i++) {
    const gap = (lead.score as number) - (ok[i].score as number);
    const spread = (lead.score_std ?? 0) + (ok[i].score_std ?? 0);
    if (gap > spread) break;
    n++;
  }
  return n;
}

/** Leaderboard table (best first) + a details panel (permutation importance + confusion matrix)
 *  for the selected row. Backend already sorts rows by blind-well score descending.
 *
 *  Exported so it can be driven with synthetic rows over the vite dev server, which is this repo's
 *  only route to exercising frontend logic — there is no TS test runner, and the tie rule and the
 *  whisker geometry are both wrong in ways a screenshot shows and a type check does not. */
export function renderLeaderboard(host: HTMLElement, res: MlEvalResult, isClf: boolean): void {
  host.innerHTML = "";
  if (res.error || !res.rows.length) return;
  const scoreLabel = isClf ? "Accuracy" : "R²";
  const secLabel = isClf ? "macro-F1" : "RMSE";

  const table = document.createElement("table");
  table.className = "mc-table ml-leaderboard";
  const head = document.createElement("tr");
  for (const h of ["#", "Algorithm", "Settings", "Curves", scoreLabel, "±", secLabel]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const detail = document.createElement("div");
  detail.className = "ml-detail";

  const firstOkRow = res.rows.find((r) => !r.error) ?? null;
  const tied = tiedAtTheTop(res.rows);
  res.rows.forEach((row, i) => {
    const tr = document.createElement("tr");
    const sec = isClf ? row.metrics?.["macro_f1"] : row.metrics?.["rmse"];
    // Which estimator this row actually describes. One row carries the settings on screen; the rest
    // are at library defaults, which is what the run would fit for them — but only if it is said.
    const settings = res.params_for && row.algorithm === res.params_for ? "yours" : "defaults";
    const cells = row.error
      ? [String(i + 1), row.algorithm, settings, row.features.join(", "), "error", "—", row.error]
      : [
          String(i + 1),
          row.algorithm,
          settings,
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
    // Only mark a winner the fold spread can actually separate. Where it cannot, every row in the
    // tie is marked instead of the first one, so the table stops answering a question it can't.
    if (!row.error && i < tied) tr.classList.add(tied === 1 ? "mc-best" : "ml-lb-tied");
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
  if (tied > 1) {
    const note = document.createElement("div");
    note.className = "mc-chain-note ml-tie-note";
    note.textContent =
      `Top ${tied} are within their own fold-to-fold spread — this run does not separate them. ` +
      `Choose on grounds it can support: fewer curves, a cheaper model, or one whose ` +
      `importances hold across wells.`;
    host.appendChild(note);
  }
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

    // Say where the number came from. Measured on held-out rows it answers the same question the
    // blind score does; measured on fewer folds than the score, it answers it over fewer wells.
    const prov = document.createElement("div");
    prov.className = "ml-imp-prov";
    prov.textContent =
      row.n_imp_folds > 0
        ? `Measured on held-out wells, ${row.n_imp_folds} fold${row.n_imp_folds === 1 ? "" : "s"}` +
          ` — the same rows the score above is measured on. The bar is the mean; the whisker is` +
          ` the spread between wells.`
        : "No fold could be permuted, so no importance was measured.";
    host.appendChild(prov);

    // Scale to mean + spread so a whisker cannot run off the end of its own track.
    const reach = row.features.map(
      (_, i) => Math.abs(row.importances[i] ?? 0) + Math.abs(row.importances_std?.[i] ?? 0),
    );
    const maxAbs = Math.max(1e-9, ...reach);
    row.features.forEach((f, i) => {
      const v = row.importances[i] ?? 0;
      const sd = row.importances_std?.[i] ?? Number.NaN;
      const line = document.createElement("div");
      line.className = "ml-imp-row";
      const name = document.createElement("span");
      name.className = "ml-imp-name";
      name.textContent = f;
      const barWrap = document.createElement("div");
      barWrap.className = "ml-imp-bar-wrap";
      const bar = document.createElement("div");
      bar.className = "ml-imp-bar";
      const pct = (x: number) => `${(Math.max(0, x) / maxAbs) * 100}%`;
      bar.style.width = pct(v);
      barWrap.appendChild(bar);
      if (Number.isFinite(sd) && sd > 0) {
        // A whisker that reaches back past zero is the whole point: this feature carried in some
        // wells and not others, and its mean is not evidence of a predictor.
        const wh = document.createElement("div");
        wh.className = "ml-imp-whisker";
        wh.style.left = pct(Math.max(0, v - sd));
        wh.style.width = pct(Math.min(v, sd) + sd);
        barWrap.appendChild(wh);
      }
      const val = document.createElement("span");
      val.className = "ml-imp-val";
      val.textContent = Number.isFinite(v)
        ? Number.isFinite(sd)
          ? `${v.toFixed(4)} ±${sd.toFixed(4)}`
          : v.toFixed(4)
        : "—";
      // Inside its own spread of zero, a feature has not been shown to carry anything.
      if (Number.isFinite(v) && Number.isFinite(sd) && v <= sd) {
        line.classList.add("ml-imp-unclear");
        line.title = "Mean is within its own spread across wells — not separated from no effect.";
      }
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
