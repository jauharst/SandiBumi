import {
  applyMlModel,
  deleteMlModel,
  listCurveCatalog,
  listMlModels,
  listTops,
  listWells,
  mlDeterminismNote,
  mlModelWarnings,
  renameMlModel,
  runMl,
  runMlEval,
  statsCurveSummary,
  curveSampling,
  type CoverageSegment,
  type CurveSampling,
  type CurveStatsRow,
  type MlEvalResult,
  type MlEvalRow,
  type MlModelInfo,
  type MlRequest,
  type MlResult,
  type SplitBalance,
  type TopEntry,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, defaultRunWellIds, filterByActiveGroup, setStatus } from "../state";
import { buildLogSetPicker } from "./logSetPicker";
import { FACIES_PALETTE } from "./plotCanvas";
import { formRow } from "./modal";
import { buildParamSources } from "./paramSources";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";
import { buildImageExportButtons } from "./plotExport";

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
  /** Entries sharing a family across BOTH supervised tasks are ONE entry in the picker, under
   *  Universal. Random Forest is `rf` in both; Support Vector is `svr` fitting a value and `svm`
   *  fitting a class — different estimators, one idea, and listing them as two algorithms claimed a
   *  choice the user does not actually have to make until they know what they are predicting. */
  family?: string;
  /** The name the family goes under when it is listed once. Set on the regression side. */
  familyLabel?: string;
}

interface TaskSpec {
  id: MlRequest["task"];
  label: string;
  /** Heading this task's algorithms sit under in the single grouped picker.
   *
   *  Jauhar, 2026-08-07: *"just split algorithm for continuous log or discrete log, its okay if
   *  there are 1 alg that can used for those 2 shown together"*. So the question the picker asks is
   *  **what are you predicting**, not "pick a task, now pick an algorithm from what that leaves" —
   *  and Random Forest legitimately appears under both continuous and discrete, because it is two
   *  estimators with one idea rather than one estimator doing two jobs. The old cascade also reset
   *  the algorithm every time the task changed, which is a click nobody asked for.
   *
   *  Phrased as the OUTPUT rather than as the statistical name: an interpreter picks by what they
   *  want out of it, and "regression" is the method's name for the answer, not the answer. */
  group: string;
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
    group: "Continuous log  —  predict a value",
    supervised: true,
    defaultOut: "ML_PRED",
    algos: [
      { id: "rf", label: "Random Forest Regressor", family: "rf", familyLabel: "Random Forest",
        desc: "Ensemble of averaged decision trees — non-linear and resistant to overfitting.",
        params: [num("n_estimators", "trees", 200), num("max_depth", "max depth (0 = none)", 0)] },
      { id: "gbdt", label: "Gradient Boosting (XGBoost)",
        desc: "Sequential trees minimizing error — highest accuracy in complex settings. Falls back to sklearn boosting if xgboost isn't installed.",
        params: [num("n_estimators", "trees", 300), num("learning_rate", "learning rate", 0.1), num("max_depth", "max depth", 4)] },
      { id: "svr", label: "Support Vector Regression", family: "svec", familyLabel: "Support Vector",
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
    group: "Discrete log  —  predict a class",
    supervised: true,
    defaultOut: "ML_CLASS",
    algos: [
      { id: "svm", label: "Support Vector Machine", family: "svec",
        desc: "Non-linear separator for distinct rock types via high-dimensional mapping.",
        params: [num("C", "C", 10)] },
      { id: "knn", label: "K-Nearest Neighbors",
        desc: "Labels each sample by the most common class among its nearest neighbours.",
        params: [num("n_neighbors", "neighbours", 7)] },
      { id: "rf", label: "Random Forest Classifier", family: "rf",
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
    group: "Electrofacies  —  no target curve",
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
    group: "Reduction  —  no target curve",
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
  /** What was being predicted before the last picker change — see the change handler. */
  let prevTask = task;

  const content = document.createElement("div");
  content.className = "mc-dialog ml-pane";

  // --- The five sections ----------------------------------------------------
  //
  // Jauhar, 2026-08-07: *"better to split into subpanes inside ML, for input, data qc, model,
  // result visualization"*. One scrolling column had grown to a dozen form rows in no particular
  // order — the algorithm at the top, its parameters two thirds of the way down, the output curve
  // below that — so setting up a run meant scrolling past everything twice.
  //
  // The order is the order the work is done in, and each section answers one question: what am I
  // learning from, is that data fit to learn from, what shall I fit, what came out, and — once I
  // believe it — where else does it go. A segmented strip rather than a new tab component —
  // `.seg`/`.seg-opt` is already this app's segmented control (Organic increment 2), and a second
  // mechanism for "pick one of these" would be a second thing to keep in agreement.
  //
  // Distribution sits AFTER Results (Jauhar, 2026-08-07: *"swap model dist and result position"*),
  // and the reason is more than tidiness: propagating a model you have not looked at is the one
  // move this pane should not make easy. The blind-well scores and the predicted-vs-measured
  // crossplot are what decide whether a model is fit to leave the wells it was trained on, so the
  // section that spends that judgement comes after the section that supplies it.
  const SECTIONS = [
    ["input", "Input", "Which curves, which wells, over which interval, and which stored values to read them from."],
    ["qc", "Data QC", "Whether the data you just chose can support the model you are about to fit."],
    ["model", "Model", "What to fit, how it is validated, and what the outputs are called. Run Model lives here."],
    ["results", "Results", "What the run produced, how the models compare, and the models you have kept."],
    ["dist", "Model Distribution", "Propagate a kept model to the rest of the field — its own wells, interval and names."],
  ] as const;
  type SectionId = (typeof SECTIONS)[number][0];

  const tabStrip = document.createElement("div");
  tabStrip.className = "seg";
  // The sticky bar is a WRAPPER, not the pill group itself. `.seg` sizes to its content, so making
  // it sticky pinned a background only as wide as the four tabs and let the scrolling form show
  // through beside them — the segmented control and the fields underneath drawn on top of each
  // other. Same failure as any sticky header with a background narrower than its container.
  const tabBar = document.createElement("div");
  tabBar.className = "ml-sections";
  tabBar.appendChild(tabStrip);
  const panels = new Map<SectionId, HTMLElement>();
  const tabs = new Map<SectionId, HTMLButtonElement>();
  const sectionHost = document.createElement("div");
  sectionHost.className = "ml-section-host";
  let activeSection: SectionId = "input";

  function showSection(id: SectionId): void {
    activeSection = id;
    for (const [k, el] of tabs) el.setAttribute("aria-pressed", String(k === id));
    for (const [k, el] of panels) el.hidden = k !== id;
    // Data QC is a measurement over the CURRENT selection, so it is taken when the section is
    // opened rather than kept live: recomputing on every checkbox click would spawn a query per
    // keystroke, and a stale answer beside a changed selection is worse than an absent one.
    if (id === "qc") void refreshQc();
  }

  for (const [id, label, title] of SECTIONS) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.textContent = label;
    b.title = title;
    b.setAttribute("aria-pressed", String(id === activeSection));
    b.addEventListener("click", () => showSection(id));
    tabs.set(id, b);
    tabStrip.appendChild(b);

    const panel = document.createElement("div");
    panel.className = "ml-section";
    panel.hidden = id !== activeSection;
    panels.set(id, panel);
    sectionHost.appendChild(panel);
  }
  content.append(tabBar, sectionHost);
  const sIn = panels.get("input") as HTMLElement;
  const sQc = panels.get("qc") as HTMLElement;
  const sModel = panels.get("model") as HTMLElement;
  const sDist = panels.get("dist") as HTMLElement;
  const sRes = panels.get("results") as HTMLElement;

  // --- Algorithm: THREE groups, by what kind of log comes out ---------------
  //
  // Jauhar, 2026-08-07: *"for model, i want only 3 option, Universal for continous & discrete,
  // continous only, and discrete only"*. The picker had four groups, one per task, which asked the
  // user to know the statistical name of what they wanted before they could find it.
  //
  // The three groups are derived, not hand-listed, so they cannot drift from what the runner
  // actually supports:
  //
  // - **Universal** — the families the runner fits BOTH ways. Random Forest is `rf` in each;
  //   Support Vector is `svr` for a value and `svm` for a class. Listed ONCE, with a
  //   Continuous/Discrete choice appearing beside it, because until you know what you are predicting
  //   there is no choice to make between them.
  // - **Continuous only** / **Discrete only** — everything else, grouped by the kind of log it
  //   writes. That puts electrofacies clustering under Discrete (it writes class codes) and PCA /
  //   t-SNE under Continuous (they write component curves), which loses no capability and drops the
  //   two groups whose headings named a method rather than an answer. An entry needing no target
  //   says so in its own label, since it sits beside ones that do.
  const algoSel = document.createElement("select");
  algoSel.className = "form-control";
  const reg = TASKS.find((t) => t.id === "regression") as TaskSpec;
  const cls = TASKS.find((t) => t.id === "classification") as TaskSpec;
  /** Families the runner fits both ways — the Universal group, and nothing else. */
  const universal = reg.algos.filter((a) => a.family && cls.algos.some((b) => b.family === a.family));
  const isUniversal = (a: AlgoSpec) => !!a.family && universal.some((u) => u.family === a.family);
  const writesDiscrete = (t: TaskSpec) => t.id === "classification" || t.id === "clustering";
  const GROUPS: [string, [TaskSpec, AlgoSpec][]][] = [
    [
      "Universal  —  continuous or discrete",
      universal.map((a) => [reg, a] as [TaskSpec, AlgoSpec]),
    ],
    [
      "Continuous only  —  predicts a value",
      TASKS.filter((t) => !writesDiscrete(t)).flatMap((t) =>
        t.algos.filter((a) => !isUniversal(a)).map((a) => [t, a] as [TaskSpec, AlgoSpec]),
      ),
    ],
    [
      "Discrete only  —  predicts a class",
      TASKS.filter(writesDiscrete).flatMap((t) =>
        t.algos.filter((a) => !isUniversal(a)).map((a) => [t, a] as [TaskSpec, AlgoSpec]),
      ),
    ],
  ];
  for (const [heading, entries] of GROUPS) {
    if (!entries.length) continue;
    const g = document.createElement("optgroup");
    g.label = heading;
    for (const [t, a] of entries) {
      const o = document.createElement("option");
      // task:algo, because an algorithm id alone is ambiguous once `rf` is in two tasks.
      o.value = `${t.id}:${a.id}`;
      // A universal family goes under its family name; the per-task labels ("… Regressor",
      // "… Classifier") would each be half a truth in a group that covers both.
      o.textContent = isUniversal(a)
        ? (a.familyLabel ?? a.label)
        : t.supervised
          ? a.label
          : `${a.label} — no target curve`;
      g.appendChild(o);
    }
    algoSel.appendChild(g);
  }
  /**
   * The option that stands for a (task, algorithm) pair.
   *
   * A universal family is listed ONCE, under its regression-side value, so `classification:svm` is
   * not an option that exists — assigning it silently blanked the picker while the run underneath
   * was correctly configured. The select names the FAMILY; the Predicting control beside it carries
   * the task. This is the one place that knows that, so the two cannot disagree.
   */
  const pickerValue = (t: TaskSpec, a: AlgoSpec): string => {
    if (isUniversal(a)) {
      const twin = reg.algos.find((x) => x.family === a.family);
      if (twin) return `${reg.id}:${twin.id}`;
    }
    return `${t.id}:${a.id}`;
  };
  algoSel.value = pickerValue(task, algo);
  const algoDesc = document.createElement("div");
  algoDesc.className = "mc-chain-note";
  sModel.appendChild(formRow("Algorithm", algoSel));
  sModel.appendChild(algoDesc);

  // The Continuous/Discrete choice a universal family needs, and only it. Shown beside the picker
  // rather than as a separate concept: it is the second half of "which algorithm", not a new
  // setting. Switching it swaps to the same family's estimator in the other task — Random Forest
  // stays Random Forest, and the parameter grid follows because the two take different parameters.
  const kindSeg = document.createElement("div");
  kindSeg.className = "seg";
  const kindBtns = new Map<MlRequest["task"], HTMLButtonElement>();
  for (const [id, label] of [
    ["regression", "Continuous"],
    ["classification", "Discrete"],
  ] as [MlRequest["task"], string][]) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.textContent = label;
    b.addEventListener("click", () => {
      const t = TASKS.find((x) => x.id === id) as TaskSpec;
      const twin = t.algos.find((x) => x.family === algo.family);
      if (!twin) return;
      task = t;
      algo = twin;
      prevTask = t;
      algoSel.value = pickerValue(task, algo);
      syncAlgo();
    });
    kindBtns.set(id, b);
    kindSeg.appendChild(b);
  }
  const kindRow = formRow("Predicting", kindSeg);
  sModel.appendChild(kindRow);
  function syncKind(): void {
    const uni = isUniversal(algo);
    kindRow.style.display = uni ? "" : "none";
    for (const [id, b] of kindBtns) b.setAttribute("aria-pressed", String(uni && task.id === id));
  }

  // --- Also run (comparison) ------------------------------------------------
  // Jauhar, 2026-08-07: *"in model user should have option to run multiple model simultaneously, so
  // later in result it can be compared at instant"*. Compare algorithms already scored many
  // estimators — but scoring is not shipping. This runs them for real: each writes its own curve,
  // keeps its own blind score and saves its own model, so the comparison is over the curves that
  // would actually be delivered rather than over a cross-validation number.
  //
  // They run one after another, not concurrently. DuckDB is a single writer and each fit is its own
  // Python subprocess, so concurrency would buy little — and "simultaneously" here means one click,
  // not one instant. What makes them COMPARABLE is that they share the run's seed, split fraction
  // and split mode, so every model is fitted and scored on exactly the same rows.
  const alsoWrap = document.createElement("div");
  alsoWrap.className = "ml-also";
  const alsoChecks = new Map<string, HTMLInputElement>();
  const alsoNote = document.createElement("div");
  alsoNote.className = "ml-norm-why";
  // Where the ONE open parameter panel renders. One at a time rather than all of them inline: the
  // whole complaint about this control was its height, and four expanded parameter grids would be
  // the same problem with more in it.
  const alsoParamsHost = document.createElement("div");
  alsoParamsHost.className = "ml-also-params";
  alsoParamsHost.hidden = true;
  // ONE block child, not three siblings. `.form-row` is a flex ROW, so appending the note and the
  // parameter panel beside the chips made all three compete for the same horizontal space — the
  // chips shrank to their widest item and every one of them wrapped onto its own line, rebuilding
  // the stacked column this was meant to replace. Same shape as the Output-resolution row above.
  const alsoStack = document.createElement("div");
  alsoStack.className = "ml-also-stack";
  alsoStack.append(alsoWrap, alsoNote, alsoParamsHost);
  const alsoRow = formRow("Also run", alsoStack);
  sModel.appendChild(alsoRow);

  /**
   * Per-algorithm parameter OVERRIDES for the also-run models, keyed by algorithm id.
   *
   * Jauhar, 2026-08-07: *"where is customized parameter for each alg?"* — it did not exist. Every
   * co-run model took its manifest defaults, and the note said so, which is honest and is not the
   * same as being able to do the thing: comparing a tuned Random Forest against a default XGBoost
   * is not a comparison of the two methods.
   *
   * **Only CHANGED values are stored, and only changed values are sent.** Sending a parameter at its
   * own default would make `P()` record it as user-supplied, and SB-MLA-001's whole point is that
   * the run record distinguishes a value somebody chose from one nobody touched. Kept by algorithm
   * id so switching the primary algorithm back and forth does not discard tuning.
   */
  const alsoParams = new Map<string, Map<string, number | string>>();
  let alsoOpen: string | null = null;
  /** Per-chip appearance refresh, so a parameter edit can update its own chip without a rebuild —
   *  rebuilding the chip row from inside an input handler would destroy the field being typed in. */
  const alsoChipSync = new Map<string, () => void>();

  /** The params to SEND for one co-run algorithm: the shared run settings plus its own overrides. */
  const alsoParamsFor = (a: AlgoSpec): Record<string, number | string | boolean> => {
    const out: Record<string, number | string | boolean> = {
      standardize: stdCb.checked,
      seed: Math.round(parseFloat(seedInput.value) || 42),
      spectral_texture: task.id === "regression" && specCb.checked,
    };
    for (const [k, v] of alsoParams.get(a.id) ?? []) out[k] = v;
    return out;
  };

  function renderAlsoParams(): void {
    alsoParamsHost.innerHTML = "";
    const a = alsoOpen ? task.algos.find((x) => x.id === alsoOpen) : null;
    alsoParamsHost.hidden = !a;
    if (!a) return;
    const head = document.createElement("div");
    head.className = "ml-also-params-head";
    head.textContent = a.params.length
      ? `${a.label} — anything left untouched runs at its default, and the run records it as one.`
      : `${a.label} has nothing to tune — it is fitted from the data alone.`;
    alsoParamsHost.appendChild(head);
    const grid = document.createElement("div");
    grid.className = "mc-settings";
    const store = alsoParams.get(a.id) ?? new Map<string, number | string>();
    alsoParams.set(a.id, store);
    for (const spec of a.params) {
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
      } else {
        input = document.createElement("input");
        input.type = spec.kind === "num" ? "number" : "text";
        if (spec.kind === "num") input.step = "any";
      }
      input.className = "form-control";
      input.value = String(store.get(spec.key) ?? spec.def);
      input.title = `${spec.label} — default ${spec.def}`;
      const mark = () => label.classList.toggle("mc-field-changed", String(input.value) !== String(spec.def));
      input.addEventListener("input", () => {
        // Back to the default means REMOVE the override, not store the default — otherwise the run
        // record would report a value the user had merely typed back.
        if (String(input.value) === String(spec.def) || input.value === "") store.delete(spec.key);
        else store.set(spec.key, spec.kind === "num" ? Number(input.value) : input.value);
        mark();
        alsoChipSync.get(a.id)?.();
        syncAlso();
      });
      input.addEventListener("change", () => input.dispatchEvent(new Event("input")));
      mark();
      label.append(t, input);
      grid.appendChild(label);
    }
    alsoParamsHost.appendChild(grid);
  }

  function renderAlso(): void {
    // Snapshot BEFORE clearing: this now re-runs when a gear is opened, not only when the algorithm
    // changes, and rebuilding from an already-cleared map would silently untick every model the user
    // had chosen — a click that appears to open a panel and quietly cancels four runs.
    const wasChecked = new Set(
      [...alsoChecks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id),
    );
    alsoWrap.innerHTML = "";
    alsoChecks.clear();
    alsoChipSync.clear();
    const others = task.algos.filter((a) => a.id !== algo.id);
    alsoRow.style.display = others.length ? "" : "none";
    for (const a of others) {
      // One compact chip per algorithm instead of a stacked checkbox row per algorithm. Same
      // controls, a quarter of the height, and it wraps at whatever width the pane happens to be.
      const chip = document.createElement("label");
      chip.className = "ml-also-chip";
      chip.title = a.desc;
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = wasChecked.has(a.id);
      const name = document.createElement("span");
      name.textContent = a.label;
      // The gear appears only once the model is actually going to run, because parameters for a
      // model nobody ticked are a control with no effect.
      const gear = document.createElement("button");
      gear.type = "button";
      gear.className = "ml-also-gear";
      gear.textContent = "⚙";
      gear.title = `Parameters for ${a.label}`;
      gear.hidden = true;
      gear.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        alsoOpen = alsoOpen === a.id ? null : a.id;
        renderAlso();
        renderAlsoParams();
      });
      const sync = () => {
        gear.hidden = !cb.checked;
        chip.classList.toggle("ml-also-on", cb.checked);
        chip.classList.toggle("ml-also-tuned", (alsoParams.get(a.id)?.size ?? 0) > 0);
        gear.classList.toggle("ml-also-gear-open", alsoOpen === a.id);
      };
      cb.addEventListener("change", () => {
        if (!cb.checked && alsoOpen === a.id) alsoOpen = null;
        sync();
        renderAlsoParams();
        syncAlso();
      });
      sync();
      alsoChipSync.set(a.id, sync);
      chip.append(cb, name, gear);
      alsoChecks.set(a.id, cb);
      alsoWrap.appendChild(chip);
    }
    syncAlso();
  }
  /** Every algorithm this click will fit, the picked one first. */
  const alsoSelected = (): AlgoSpec[] => [
    algo,
    ...task.algos.filter((a) => a.id !== algo.id && alsoChecks.get(a.id)?.checked),
  ];
  function syncAlso(): void {
    const picked = alsoSelected();
    // Jauhar, 2026-08-07: *"assume there are 7 alg than can be used for continuous, i only see also
    // run for other 4"*. The picker groups by the KIND OF LOG that comes out, so "Continuous"
    // holds the regressions and PCA/t-SNE together; Also run offers only the same TASK, because a
    // reduction has no target and writes component curves, so there is no shared curve to compare
    // against and no score that means the same thing. Correct, and it was invisible — a user counts
    // the dropdown, counts the checkboxes, and is left to guess which of the two is wrong.
    // The picker's grouping, not TaskSpec.group: "Continuous only" spans regression AND reduction,
    // which is exactly why the counts differ.
    const writesDiscreteTask = (t: TaskSpec) => t.id === "classification" || t.id === "clustering";
    const otherTasks = TASKS.filter(
      (t) => t.id !== task.id && writesDiscreteTask(t) === writesDiscreteTask(task) && !t.supervised,
    ).flatMap((t) => t.algos.map((a) => a.label));
    const cannot = otherTasks.length
      ? ` ${otherTasks.join(" and ")} are in the same group in the list above but cannot be co-run here: they have no target curve, so there is nothing to compare a prediction of ${targetSel.value || "the target"} against.`
      : "";
    const tuned = picked.slice(1).filter((a) => (alsoParams.get(a.id)?.size ?? 0) > 0).map((a) => a.label);
    alsoNote.textContent =
      picked.length === 1
        ? `One model, writing one curve. Tick others to fit them on the same rows, with the same split and the same seed, and compare the curves you would actually deliver.${cannot}`
        : `${picked.length} models on the same rows, the same split and the same seed — so the scores are comparable. Each writes its own curve suffixed with its name (${picked
            .map((a) => `${outInput.value || "PRED"}_${a.id.toUpperCase()}`)
            .join(", ")}) and saves its own model, because ${picked.length} models cannot share one curve name. Use ⚙ to tune any of them; ${
            tuned.length ? `${tuned.join(", ")} ${tuned.length === 1 ? "carries" : "carry"} your own settings, and the rest run` : "untouched ones run"
          } at their defaults, which the run records as defaults.${cannot}`;
  }

  // SB-MLA-008. What about the chosen configuration would not reproduce on another machine, said
  // BEFORE the run rather than discovered when somebody cannot repeat the result. Hidden unless
  // there is something to say — today that is only the gbdt estimator substitution, and a line that
  // is present but empty most of the time teaches the eye to skip the place it appears.
  const detNote = document.createElement("div");
  detNote.className = "ml-determinism-note";
  detNote.hidden = true;
  sModel.appendChild(detNote);

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
  // The fixed-limits table is per input curve, so it follows the ticks. Delegated on the box rather
  // than bound per checkbox: the list carries every imported log on a real delivery, and one
  // listener that outlives the rows is cheaper than several hundred that do not.
  featBox.addEventListener("change", () => renderBasisLimits());
  sIn.appendChild(
    formRow(
      "Input curves",
      featBox,
      "Tick any curve the model should learn from — standard columns, computed curves and imported logs alike. Order matters for clustering: class 0 = lowest mean of the FIRST checked curve (put GR first).",
    ),
  );

  const targetSel = document.createElement("select");
  targetSel.className = "form-control";
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
  sIn.appendChild(targetRow);

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
  sModel.append(xfRow, xfEcho);

  // Optional MASK curve — kept visible for ALL tasks (it also governs the unsupervised fit pool),
  // default "(none)" so data is never silently dropped.
  const maskSel = document.createElement("select");
  maskSel.className = "form-control";
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
  sIn.appendChild(
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
  sIn.appendChild(trainRow);

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
  sModel.appendChild(splitRow);
  echoSplit();

  // Apply wells = the run scope. Unsupervised models also FIT on these (pooled — field-wide).
  sIn.appendChild(scope.el);

  // --- Hyperparameters + output -------------------------------------------
  const paramsGrid = document.createElement("div");
  paramsGrid.className = "mc-settings";
  sModel.appendChild(formRow("Parameters", paramsGrid));
  let paramInputs: {
    spec: ParamSpec;
    get: () => number | string;
    /** Whether this field still holds the algorithm's default — the distinction SB-MLA-001 records
     *  and the form previously did not show. */
    changed: () => boolean;
    reset: () => void;
  }[] = [];

  const outInput = document.createElement("input");
  outInput.type = "text";
  outInput.className = "form-control";
  outInput.value = task.defaultOut;
  let outEdited = false;
  outInput.addEventListener("input", () => {
    outEdited = true;
    // The transform echo names the two curves it will write, so it has to follow the name.
    echoTransform();
  });
  sModel.appendChild(
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
  sModel.appendChild(formRow("Common", commonWrap));

  // --- Coverage segmentation ------------------------------------------------
  // Jauhar, 2026-08-07: *"assume user have 4 curves, model should still run even 1 curves only half
  // depth coverage, (model only predict using 3 curves on the other half depth coverage)"*. Off by
  // default, because on it the curve is made by more than one model and a reader has to be told —
  // which is exactly what the notes and the per-segment scores do, and what silently defaulting it
  // on would skip.
  const covCb = document.createElement("input");
  covCb.type = "checkbox";
  const covLabel = document.createElement("label");
  covLabel.className = "mc-field";
  covLabel.append(covCb, document.createTextNode(" Fit a model per available-input pattern"));
  const covWhy = document.createElement("div");
  covWhy.className = "ml-norm-why";
  // A column, not the usual `.mc-settings` flex row: `.mc-field` stacks its checkbox ABOVE its text,
  // which reads fine for a two-word caption sitting beside the Seed box and turns a sentence-length
  // one into three wrapped lines in a narrow column. The explanation goes below both, full width.
  const covWrap = document.createElement("div");
  covWrap.className = "ml-cov";
  covWrap.append(covLabel, covWhy);
  const covRow = formRow("Partial coverage", covWrap);
  sModel.appendChild(covRow);
  function syncCoverage(): void {
    // `style.display` rather than the `hidden` attribute, matching every other row here — a CSS
    // `display` rule overrides `hidden`, and this repo has been bitten by that twice.
    covRow.style.display = task.supervised ? "" : "none";
    covWhy.textContent = covCb.checked
      ? "Each depth is predicted by the largest model whose inputs it carries: where all your curves exist, a model on all of them; where one is short, a smaller model on the rest. Every segment gets its own blind score and its own saved model, because they are different models — one number over both would describe neither."
      : "One model over the depths where EVERY input has a value. A curve logged over half the interval therefore removes the other half of all the others too. Turn this on to keep that rock.";
  }
  // --- Output resolution ----------------------------------------------------
  // Jauhar, 2026-08-07: *"sampling rate, each log has different resolution … Result should adjust
  // their frequency to log target"*, then *"writing output at target sampling"*. A model fitted
  // against a 0.5 m target predicts at every INPUT depth, so it emits a value every 0.1524 m — a
  // curve claiming three times the vertical resolution anything it learned from ever had.
  //
  // Declared, never inferred. The app's own rule everywhere else (LONG/WIDE is declared, the light
  // is declared): a run that quietly coarsened its own output would be changing the answer on the
  // user's behalf. The measured target step is FILLED IN so the choice costs one click, not a
  // lookup — but it is still a choice, and it is still editable.
  const resSeg = document.createElement("div");
  resSeg.className = "seg";
  let resMode: "input" | "step" = "input";
  const resBtns = new Map<typeof resMode, HTMLButtonElement>();
  for (const [k, label] of [
    ["input", "As predicted"],
    ["step", "Target sampling"],
  ] as [typeof resMode, string][]) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.textContent = label;
    b.setAttribute("aria-pressed", String(k === resMode));
    b.addEventListener("click", () => {
      resMode = k;
      for (const [kk, el] of resBtns) el.setAttribute("aria-pressed", String(kk === k));
      syncRes();
    });
    resBtns.set(k, b);
    resSeg.appendChild(b);
  }
  const resStep = document.createElement("input");
  resStep.type = "number";
  resStep.step = "any";
  resStep.className = "form-control";
  const resStepLabel = document.createElement("label");
  resStepLabel.className = "mc-field ml-res-step";
  const resStepText = document.createElement("span");
  resStepText.textContent = "Block thickness";
  resStepLabel.append(resStepText, resStep);
  const resWhy = document.createElement("div");
  resWhy.className = "ml-norm-why";
  const resWrap = document.createElement("div");
  resWrap.className = "ml-cov";
  const resTop = document.createElement("div");
  resTop.className = "mc-settings";
  resTop.append(resSeg, resStepLabel);
  resWrap.append(resTop, resWhy);
  const resRow = formRow("Output resolution", resWrap);
  sModel.appendChild(resRow);
  /** The target's own median sampling, measured over the training wells. Null until QC has run. */
  let targetStep: number | null = null;
  function syncRes(): void {
    resStepLabel.style.display = resMode === "step" ? "" : "none";
    if (resMode === "step" && !resStep.value && targetStep) resStep.value = String(targetStep);
    const shown = Number(resStep.value);
    resWhy.textContent =
      resMode === "input"
        ? targetStep
          ? `The curve gets a value at every depth its INPUTS have — finer than the ${targetStep} spacing of ${targetSel.value || "the target"}, so it will look more detailed than anything it learned from.`
          : "The curve gets a value at every depth its inputs have. Where the target was logged more coarsely than the inputs, that is more vertical resolution than the model can actually support."
        : shown > 0
          ? `One value per ${shown} interval, held across it, on each well's own depths — so the curve stops claiming resolution it does not have. The depth frame is unchanged; set the curve's draw style to Step in the curve editor, or the log view draws a gradient between two block values that nothing measured.`
          : "Enter the thickness one value should cover. Open Data QC and the target's own measured sampling is filled in here.";
  }
  resStep.addEventListener("input", syncRes);

  // --- The second, textured curve (round-3 item 5) ---------------------------
  //
  // Sampling and resolution are different things, so this sits beside the sampling control rather
  // than inside it: blocking to the target's step stops a curve OVERSTATING its resolution, and this
  // addresses the opposite complaint — a prediction that is smoother than the log it was fitted
  // against, because a regression predicts the conditional mean and can only carry through detail
  // its inputs contain.
  //
  // OFF by default, and it writes a SECOND curve rather than changing the first. The plain
  // prediction is the defensible one; a textured curve looks more like a real log, which is exactly
  // backwards from how much it can be trusted, so it has to be asked for and it has to be named.
  const specCb = document.createElement("input");
  specCb.type = "checkbox";
  const specLabel = document.createElement("label");
  specLabel.className = "mc-field ml-cov";
  const specText = document.createElement("span");
  specText.textContent = "Also write a spectrally textured copy";
  specLabel.append(specCb, specText);
  const specWhy = document.createElement("div");
  specWhy.className = "ml-norm-why";
  const specWrap = document.createElement("div");
  specWrap.className = "ml-cov";
  specWrap.append(specLabel, specWhy);
  const specRow = formRow("Missing detail", specWrap);
  sModel.appendChild(specRow);
  function syncSpec(): void {
    const base = (outInput.value.trim() || task.defaultOut).toUpperCase();
    specRow.style.display = task.id === "regression" ? "" : "none";
    specWhy.textContent = specCb.checked
      ? `A second curve, ${base}_SIM, gets the frequency content ${targetSel.value || "the target"} has and the prediction lacks — matched to the measured target's own spectrum, well by well. ${base} itself is untouched. The added detail is NOT a measurement: it is one realisation of many, right in its statistics and arbitrary in its placement, so do not correlate a bed seen only in ${base}_SIM between wells.`
      : `${base} will be smoother than ${targetSel.value || "the target"}, because a model can only carry through detail its inputs contain. Tick this to also write a copy carrying the detail the target has — as a separate curve, never in place of this one.`;
  }
  specCb.addEventListener("change", syncSpec);

  covCb.addEventListener("change", () => {
    syncCoverage();
    void refreshQc();
  });

  // --- Input / output log set (`logSetPicker.ts`). A model fitted on today's PHIE and one fitted
  // after the next porosity re-run are fitted on different rock; naming the set is what lets a
  // saved model say which (Jauhar, 2026-08-05). Output defaults to ML, which is where every
  // prediction went before this was selectable.
  const setPicker = buildLogSetPicker({ write: "ML" });
  for (const row of setPicker.rows) sIn.appendChild(row);

  // Which interval the FIT learns from. In Input, beside the wells and curves, because it is part
  // of "what am I learning from" rather than of "what shall I fit".
  const fitInterval = buildIntervalPicker("Interval");
  sIn.appendChild(fitInterval.row);

  // Every parameter the runner reads through `P(p, key, default)` has a field here, and always did.
  // What it did not have was any way to see WHICH of them you had changed — a grid of numbers looks
  // identical whether they are your settings or the library's, and SB-MLA-001 exists because that
  // distinction decides whether a run can be reproduced. The record has kept it since; the form did
  // not show it. Now a changed field marks itself and offers its default back.
  const paramsReset = document.createElement("button");
  paramsReset.type = "button";
  paramsReset.className = "ml-param-reset";
  paramsReset.textContent = "Reset to defaults";
  paramsReset.hidden = true;

  function renderParams(): void {
    paramsGrid.innerHTML = "";
    paramInputs = [];
    paramsReset.hidden = true;
    if (algo.params.length === 0) {
      const none = document.createElement("span");
      none.className = "mc-empty";
      // Naming the estimator matters: "no tuning parameters" under a bare heading reads as a
      // failure to load the form. Under the algorithm's own name it reads as a fact about it.
      none.textContent = `${algo.label} has nothing to tune — it is fitted from the data alone.`;
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
      // The default is on the field itself rather than in a separate column: a "default" column
      // doubles the width of the grid to carry a number that is only interesting when it differs
      // from the one beside it.
      input.title = `${spec.label} — default ${spec.def}. The run records whether this value was yours or the default (SB-MLA-001).`;
      const mark = () => {
        label.classList.toggle("mc-field-changed", String(input.value) !== String(spec.def));
        paramsReset.hidden = !paramInputs.some((p) => p.changed());
      };
      input.addEventListener("input", mark);
      input.addEventListener("change", mark);
      label.append(t, input);
      paramsGrid.appendChild(label);
      paramInputs.push({
        spec,
        get: () => (spec.kind === "num" ? parseFloat(input.value) || 0 : input.value),
        changed: () => String(input.value) !== String(spec.def),
        reset: () => {
          input.value = String(spec.def);
          mark();
        },
      });
    }
    paramsGrid.appendChild(paramsReset);
    // SB-CORE-013. Appended BELOW the grid rather than into a cell: the grid is a compact
    // two-column form and a four-row citation panel inside one of its cells would wreck it. The
    // panel is per PARAMETER, so it is attached where that parameter is set — here, the cluster
    // count, which is the corpus's densest disagreement and the same number `facies.rs` sets.
    if (algo.params.some((p) => p.key === "k")) {
      paramsGrid.appendChild(buildParamSources("cluster_count"));
    }
  }
  paramsReset.addEventListener("click", () => {
    for (const p of paramInputs) p.reset();
    paramsReset.hidden = true;
  });

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
    syncKind();
    renderParams();
    renderAlso();
    syncNorm();
    syncCoverage();
    // "Target sampling" has no meaning without a target, so the whole row is supervised-only rather
    // than offering a choice one of whose options cannot apply.
    resRow.style.display = task.supervised ? "" : "none";
    syncRes();
    syncSpec();
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

  algoSel.addEventListener("change", () => {
    const [taskId, algoId] = algoSel.value.split(":");
    task = TASKS.find((t) => t.id === taskId) ?? TASKS[0];
    algo = task.algos.find((a) => a.id === algoId) ?? task.algos[0];
    // A universal family is listed on its regression side, so choosing it would always land on
    // Continuous — silently discarding a Discrete choice the user had already made and left set.
    // Picking Random Forest while predicting a facies log should keep predicting a facies log.
    if (isUniversal(algo) && prevTask.id === "classification") {
      const twin = cls.algos.find((a) => a.family === algo.family);
      if (twin) {
        task = cls;
        algo = twin;
      }
    }
    prevTask = task;
    syncAlgo();
  });

  // --- Data QC --------------------------------------------------------------
  //
  // Jauhar, 2026-08-07: *"for data qc, adjust based on ML choosen, i.e. trees, clustering, etc"*.
  // The checks are not generic, because what makes data unfit depends entirely on what is about to
  // be fitted: a random forest does not care that RHOB is 2.5 and GR is 90, and k-means cares about
  // nothing else. A fixed checklist would either warn about scale on a tree model (noise) or stay
  // quiet about it on a distance model (the failure this section exists to catch).
  //
  // Everything here comes from `stats_curve_summary`, which already reports n, n_missing, min/max
  // and standard deviation per well per curve, and already honours the input set and the mask. A
  // second statistics path for the same numbers would be a second convention to keep in agreement.
  const qcBtn = document.createElement("button");
  qcBtn.type = "button";
  qcBtn.className = "form-run-btn";
  qcBtn.textContent = "Check the data";
  const qcHead = document.createElement("div");
  qcHead.className = "mc-chain-note";
  const qcOut = document.createElement("div");
  qcOut.className = "ml-qc-out";

  // Normalization, shown where the reason to change it appears.
  //
  // Jauhar, 2026-08-07: *"in data qc, can we also provide normalization of data input, since
  // different log and scale will provide different weight as well, mainly for several algorithm not
  // all"*. The setting itself already existed in Model ▸ Common. What did not exist was any way to
  // decide it: the scale finding is measured here, and the control was two sections away.
  //
  // **This is one setting with two views, never two checkboxes.** A second control holding its own
  // copy of the same state is how the two come to disagree, and a run would then normalize or not
  // depending on which section you looked at last. Both read and write `stdCb`.
  const normWrap = document.createElement("div");
  normWrap.className = "ml-norm";
  const normCb = document.createElement("input");
  normCb.type = "checkbox";
  const normLab = document.createElement("label");
  normLab.append(normCb, document.createTextNode(" Standardize inputs (z-score)"));
  const normWhy = document.createElement("div");
  normWhy.className = "ml-norm-why";

  // SB-MLA-033 — what the feature space is normalized AGAINST.
  //
  // On a data-derived basis, adding one well to the build set recomputes every mean and scale, so
  // every boundary expressed in it moves in the wells that were ALREADY there. Nothing looks wrong
  // afterwards, which is why this needs a control rather than a warning.
  //
  // The limits are never filled in for the user. A GR normalized 0-150 and the same GR normalized
  // 0-200 give different clusters and both look right, so the range is the analyst's statement
  // about their field (SB-CORE-004) and the run refuses until every input has one.
  let normBasis: "data" | "limits" = "data";
  const basisLimits = new Map<string, { lo: string; hi: string }>();
  const basisSeg = document.createElement("div");
  basisSeg.className = "seg ml-basis-seg";
  const basisTable = document.createElement("div");
  basisTable.className = "ml-basis-limits";
  basisTable.hidden = true;

  /** Rebuilds the limits table for the currently ticked inputs, keeping anything already typed. */
  function renderBasisLimits(): void {
    const feats = [...featChecks.entries()].filter(([, cb]) => cb.checked).map(([n]) => n);
    basisTable.hidden = normBasis !== "limits";
    if (normBasis !== "limits") return;
    basisTable.innerHTML = "";
    if (feats.length === 0) {
      const empty = document.createElement("div");
      empty.className = "ml-norm-why";
      empty.textContent = "Tick some input curves first — the limits are per curve.";
      basisTable.appendChild(empty);
      return;
    }
    for (const name of feats) {
      const row = document.createElement("div");
      row.className = "ml-basis-row";
      const label = document.createElement("span");
      label.className = "ml-basis-curve";
      label.textContent = name;
      row.appendChild(label);
      const held = basisLimits.get(name) ?? { lo: "", hi: "" };
      for (const end of ["lo", "hi"] as const) {
        const box = document.createElement("input");
        box.type = "number";
        box.className = "form-control ml-basis-num";
        box.placeholder = end === "lo" ? "low" : "high";
        box.value = held[end];
        box.addEventListener("input", () => {
          const cur = basisLimits.get(name) ?? { lo: "", hi: "" };
          cur[end] = box.value;
          basisLimits.set(name, cur);
        });
        row.appendChild(box);
      }
      basisTable.appendChild(row);
    }
  }

  for (const [id, label] of [
    ["data", "From the data"],
    ["limits", "Fixed limits"],
  ] as ["data" | "limits", string][]) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.setAttribute("aria-pressed", String(id === normBasis));
    b.textContent = label;
    b.addEventListener("click", () => {
      normBasis = id;
      for (const o of Array.from(basisSeg.children)) {
        o.setAttribute("aria-pressed", String((o as HTMLElement).textContent === label));
      }
      renderBasisLimits();
      syncNorm();
    });
    basisSeg.appendChild(b);
  }
  const basisWhy = document.createElement("div");
  basisWhy.className = "ml-norm-why";
  normWrap.append(normLab, normWhy, basisSeg, basisWhy, basisTable);

  /** Says what standardizing would do FOR THE CHOSEN ALGORITHM, which is the only form of the
   *  question with an answer. Mirrors the Model section's state in both directions. */
  function syncNorm(): void {
    normCb.checked = stdCb.checked;
    normWhy.textContent = SCALE_FREE.has(algo.id)
      ? `${algo.label} is unaffected by it — it treats each curve on its own terms, so the z-score changes ` +
        "nothing it predicts. Harmless to leave on; it is recorded either way."
      : `${algo.label} weighs every input together, so without this the widest-ranging curve dominates ` +
        "whether or not it carries information. The scaler is fitted on the FIT rows only and stored with " +
        "the model, so an apply run reuses the same transform rather than refitting one on different wells.";
    normWhy.classList.toggle("ml-norm-off", !stdCb.checked && !SCALE_FREE.has(algo.id));
    // The basis only means anything when something is being normalized.
    basisSeg.hidden = !stdCb.checked;
    basisWhy.hidden = !stdCb.checked;
    if (!stdCb.checked) {
      basisTable.hidden = true;
    } else {
      basisWhy.textContent =
        normBasis === "data"
          ? "Mean and spread come from the wells in THIS run. Add a well and they are recomputed — " +
            "which moves every boundary expressed in them, including in the wells you did not touch. " +
            "A retrain reports how far the space moved."
          : "Each curve is normalized onto 0–1 against limits you set, so the basis does not move when " +
            "the well set does. Give every input a low and a high — SandiBumi will not choose them, " +
            "because the same curve over two ranges gives two answers and both look right.";
      renderBasisLimits();
    }
  }
  normCb.addEventListener("change", () => {
    stdCb.checked = normCb.checked;
    syncNorm();
    void refreshQc();
  });
  stdCb.addEventListener("change", syncNorm);

  sQc.append(
    formRow("Normalization", normWrap, "The same setting as Model ▸ Common — shown here because this is where the reason to change it is measured."),
    formRow("Fitness", qcBtn, "Measures the curves and wells currently selected, for the model currently chosen."),
    qcHead,
    qcOut,
  );
  let qcGen = 0;

  /** True where the estimator's PREDICTIONS are invariant to rescaling an input.
   *
   *  Jauhar asked for this to be cross-checked rather than assumed (2026-08-07), so the reasoning is
   *  written down per estimator against what `ML_BUILD_MODEL` actually constructs — not against what
   *  the algorithm is called, which is where this kind of table usually goes wrong.
   *
   *  **Invariant, and why each one is:**
   *  - `rf`, `gbdt` — trees split one feature at a time on a threshold. Any monotone rescaling maps
   *    every candidate split to an equivalent one, so the tree is the same tree.
   *  - `gnb` — GaussianNB fits a mean and variance PER FEATURE independently. Scaling feature *j*
   *    by *a* scales its fitted mean and standard deviation by *a* too, and the log-likelihood
   *    changes by −log *a*, which is identical for every class. The argmax is untouched.
   *  - `linear` — scikit-learn's `LinearRegression` is unregularised OLS, so rescaling changes the
   *    coefficients and not one predicted value. (At `degree > 1` this stays true algebraically;
   *    unscaled inputs raised to a power are merely harder to condition numerically.)
   *
   *  **Not invariant, and the reason is not always distance:**
   *  - `svr`, `svm` (RBF kernel), `knn`, `kmeans`, `gmm`, `hier`, `dbscan`, `tsne` — all measure a
   *    distance across every feature at once, so the widest-ranging curve decides the answer whether
   *    or not it carries information.
   *  - `ann` — gradient descent on unscaled inputs converges to a different place in a fixed
   *    iteration budget.
   *  - `logreg` — the trap in this list. Its arithmetic looks linear, but scikit-learn's
   *    `LogisticRegression` is L2-penalised by default: a widely-ranging feature earns a small
   *    coefficient, a small coefficient is penalised less, and the fit therefore moves with the
   *    scale. It is deliberately NOT in the invariant set.
   *  - `pca` — the most scale-sensitive thing here. Components follow variance, so on unscaled data
   *    the first component is essentially whichever curve has the largest units. */
  const SCALE_FREE = new Set(["rf", "gbdt", "gnb", "linear"]);

  async function refreshQc(): Promise<void> {
    const gen = ++qcGen;
    const feats = [...featChecks.entries()].filter(([, cb]) => cb.checked).map(([n]) => n);
    const wellIds = task.supervised
      ? [...train.checks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id)
      : scope.getWellIds();
    qcOut.innerHTML = "";
    if (feats.length === 0 || wellIds.length === 0) {
      qcHead.textContent =
        "Pick input curves and wells on the Input section first — this measures that selection, not the project.";
      return;
    }
    qcHead.textContent = "Measuring…";
    const curves = task.supervised ? [...feats, targetSel.value] : feats;
    let rows: CurveStatsRow[];
    try {
      [rows] = await statsCurveSummary({
        well_ids: wellIds,
        curves,
        input_set: setPicker.inputSet(),
        mask_curve: maskSel.value || null,
        percentiles: [],
      });
    } catch (e) {
      if (gen !== qcGen) return;
      qcHead.textContent = `Could not measure the data: ${e}`;
      return;
    }
    if (gen !== qcGen) return;
    // The sampling probe is a second round trip, deliberately: it answers a question the coverage
    // numbers raise rather than one they answer, and asking it always would put a per-curve query
    // behind every QC open. Failing it is not fatal — the coverage findings still stand.
    let sampling: [string, CurveSampling[]][] = [];
    try {
      sampling = await curveSampling(wellIds, curves);
    } catch {
      /* the sampling findings are simply absent, not wrong */
    }
    if (gen !== qcGen) return;
    // The target's own sampling, so the Output resolution box can be filled in rather than looked
    // up. MEDIAN across the training wells, not the mean: one well logged at a different rate would
    // drag a mean to a spacing no tool ever ran at, and this number is offered as a default the user
    // may accept without checking.
    if (task.supervised && targetSel.value) {
      const steps = sampling
        .flatMap(([, cs]) => cs)
        .filter((s) => s.curve === targetSel.value.toUpperCase() && s.step != null && (s.step as number) > 0)
        .map((s) => s.step as number)
        .sort((a, b) => a - b);
      targetStep = steps.length ? Number(steps[Math.floor((steps.length - 1) / 2)].toFixed(4)) : null;
      if (targetStep && !resStep.value) resStep.value = String(targetStep);
      syncRes();
    }
    renderQc(qcHead, qcOut, rows, sampling, {
      curves,
      inputs: feats,
      target: task.supervised ? targetSel.value : null,
      wells: wellIds.length,
      algorithm: algo.id,
      algorithmLabel: algo.label,
      taskId: task.id,
      scaleFree: SCALE_FREE.has(algo.id),
      k: Number(paramInputs.find((p) => p.spec.key === "k")?.get() ?? 0) || null,
      standardize: stdCb.checked,
      masked: !!maskSel.value,
      coverage: task.supervised && covCb.checked,
    });
  }
  qcBtn.addEventListener("click", () => void refreshQc());

  // --- Run: inside the Model section ---------------------------------------
  //
  // Jauhar, 2026-08-07: *"run model should only shown on model tab, and only applied to defined
  // input wells and data that shown in qc"*. It was a pane FOOTER, visible from every section,
  // which is what made the second half of that sentence a fair question: a button standing under
  // Data QC reads as acting on what Data QC is showing, and a button standing under Results reads
  // as re-running whatever produced them.
  //
  // It fits, and fitting is the Model section's subject. Propagating a fitted model is a different
  // action with a different scope, and it now has its own section and its own button — which is the
  // real answer to "what does this apply to": each button sits with the choices it consumes.
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Run Model";
  runBtn.classList.add("primary");
  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row ml-footer";
  runRow.append(runBtn, statusLine);

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
  sModel.appendChild(saveRow);
  // Last in the section, because it consumes everything above it.
  sModel.appendChild(runRow);

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
  sRes.appendChild(compareRow);

  const results = document.createElement("div");
  results.className = "mc-results";
  sRes.appendChild(results);

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
  sRes.appendChild(hint);

  // --- Model Distribution ---------------------------------------------------
  //
  // Jauhar, 2026-08-07: *"for propagation, add new subpanes there, use phrase Model Distribution, so
  // final well selection, interval selection, set/cons name, and log name behave like other
  // modules"*. Propagating was previously an Apply button on a row in the saved-models list, which
  // made it look like a property of that row rather than a run of its own — and it silently borrowed
  // the FIT's wells, interval and names, so "apply this to the rest of the field" meant editing the
  // Input section until it no longer described the fit that had been reviewed.
  //
  // So it is its own section with its own scope, its own interval and its own names, shaped like
  // every other batch run in the application. The one thing it does NOT restate is the model's
  // features and the log set they came from: those travel inside the artifact, and letting a caller
  // restate them would invite them to differ (SB-MLA-006).
  const distModel = document.createElement("select");
  distModel.className = "form-control";
  const distModelNote = document.createElement("div");
  distModelNote.className = "mc-chain-note";
  sDist.appendChild(formRow("Model", distModel));
  sDist.appendChild(distModelNote);

  const distScope = await buildWellScope();
  sDist.appendChild(distScope.el);
  const distInterval = buildIntervalPicker("Interval");
  sDist.appendChild(distInterval.row);

  const distSetPicker = buildLogSetPicker({ write: "ML" });
  for (const row of distSetPicker.rows) sDist.appendChild(row);

  const distOut = document.createElement("input");
  distOut.className = "form-control";
  distOut.placeholder = "e.g. RHOB_PRED";
  sDist.appendChild(
    formRow(
      "Output curve",
      distOut,
      "The name the propagated log is written under. Give it its own name — a distribution written over the curve the fit produced would overwrite the one you reviewed.",
    ),
  );

  const distMask = document.createElement("select");
  distMask.className = "form-control";
  {
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    distMask.appendChild(none);
    for (const name of curveNames) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      distMask.appendChild(o);
    }
  }
  sDist.appendChild(
    formRow(
      "Mask (exclude)",
      distMask,
      "Its own mask, not the fit's: the wells being propagated to are different wells, and a bad-hole flag is a property of the hole it was computed in.",
    ),
  );

  const distBtn = document.createElement("button");
  distBtn.type = "button";
  distBtn.textContent = "Distribute Model";
  distBtn.classList.add("primary");
  const distStatus = document.createElement("div");
  distStatus.className = "mc-status";
  const distRun = document.createElement("div");
  distRun.className = "mc-run-row ml-footer";
  distRun.append(distBtn, distStatus);
  sDist.appendChild(distRun);
  const distResults = document.createElement("div");
  sDist.appendChild(distResults);

  /** Kept in step with the saved-model list, which is the only source of models to distribute. */
  let distModels: MlModelInfo[] = [];
  function syncDistModels(models: MlModelInfo[]): void {
    distModels = models;
    const keep = distModel.value;
    distModel.innerHTML = "";
    if (models.length === 0) {
      const o = document.createElement("option");
      o.value = "";
      o.textContent = "(no saved models yet)";
      distModel.appendChild(o);
      distModelNote.textContent =
        "Fit something on the Model section with a name in Save model as, and it will appear here. " +
        "Distribution deliberately runs from a SAVED model rather than from the last run: a refit on " +
        "different data is a different model, and a curve that cannot name the model it came from " +
        "cannot be defended in a report.";
      return;
    }
    for (const m of models) {
      const o = document.createElement("option");
      o.value = m.model_id;
      o.textContent = `${m.name}  —  ${m.algorithm} on ${m.target_curve ?? "?"}`;
      distModel.appendChild(o);
    }
    if (models.some((m) => m.model_id === keep)) distModel.value = keep;
    syncDistNote();
  }
  function syncDistNote(): void {
    const m = distModels.find((x) => x.model_id === distModel.value);
    if (!m) {
      distModelNote.textContent = "";
      return;
    }
    // The features and their ORDER are the apply contract and are read from the artifact, so they
    // are stated here rather than offered as choices. A model fitted on [GR, RHOB] fed [RHOB, GR]
    // returns confident nonsense nothing downstream can catch.
    distModelNote.textContent =
      `Needs ${m.feature_curves.join(", ")} — in that order, from the artifact, not from Input. ` +
      `Fitted on ${m.n_train.toLocaleString()} rows from ${m.trained_on.length} well(s)` +
      (m.sklearn_version ? ` with scikit-learn ${m.sklearn_version}` : "") +
      ". Leave the input log set on (current values) and it reads the set it was fitted on.";
    if (!distOut.value.trim()) distOut.value = `${m.target_curve ?? "ML"}_DIST`;
  }
  distModel.addEventListener("change", syncDistNote);

  distBtn.addEventListener("click", async () => {
    const m = distModels.find((x) => x.model_id === distModel.value);
    if (!m) {
      distStatus.textContent = "Pick a saved model first.";
      return;
    }
    const wellIds = distScope.getWellIds();
    if (wellIds.length === 0) {
      distStatus.textContent = "No wells in scope — pick a group, pin or select wells, or choose All.";
      return;
    }
    if (!distOut.value.trim()) {
      distStatus.textContent = "Give the distributed curve a name.";
      return;
    }
    distBtn.disabled = true;
    distStatus.textContent = `Distributing '${m.name}' to ${wellIds.length} well(s)…`;
    const t0 = performance.now();
    try {
      const res = await applyMlModel({
        model_id: m.model_id,
        apply_well_ids: wellIds,
        output_curve: distOut.value.trim(),
        input_set: distSetPicker.inputSet(),
        output_set: distSetPicker.outputSet(),
        mask_curve: distMask.value || null,
        interval: distInterval.getWindow(),
      });
      const ms = Math.round(performance.now() - t0);
      if (res.error) {
        distStatus.textContent = `Failed: ${res.error}`;
      } else {
        const total = res.wells.length || wellIds.length;
        const ok = res.wells.filter((w) => !w.error).length;
        const outs = res.outputs.join(", ");
        distStatus.textContent =
          `Done in ${ms} ms → ${outs}` + (ok < total ? ` — ${total - ok} well(s) need attention` : "");
        if (ok > 0) {
          recordProcess("ML", `Distributed model '${m.name}' → ${outs} on ${ok}/${total} well(s)`);
          setStatus(`Distributed '${m.name}': ${outs} on ${ok}/${total} well(s)`);
          bumpDataVersion();
        }
      }
      renderResults(distResults, res, nameOf);
    } catch (e) {
      distStatus.textContent = `Failed: ${e}`;
    } finally {
      distBtn.disabled = false;
    }
  });

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
  sRes.appendChild(savedWrap);

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
      syncDistModels([]);
      return;
    }
    // ONE fetch feeds both the list and the distribution picker. Two calls would let the two
    // disagree about which models exist — a model deleted here and still offered there.
    syncDistModels(models);
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
        // SB-MLA-007. The backend REFUSES while a live curve names this model, and its refusal
        // lists what would be orphaned — so the second question quotes that list rather than the
        // generic warning above. A curve citing a model id that resolves to nothing is a provenance
        // block in a report naming something nobody can produce, and it surfaces months later.
        try {
          await deleteMlModel(m.model_id);
          recordProcess("ML", `Deleted saved model '${m.name}' (no delivered curve cited it)`);
        } catch (e) {
          const msg = String(e);
          // Only a citation refusal deserves a second question; anything else is a real failure.
          if (!msg.includes("name this model")) {
            savedNote.textContent = `Delete failed: ${e}`;
            return;
          }
          if (!window.confirm(`${msg}\n\nDelete '${m.name}' anyway?`)) {
            savedNote.textContent = "Kept.";
            return;
          }
          try {
            await deleteMlModel(m.model_id, true);
          } catch (e2) {
            savedNote.textContent = `Delete failed: ${e2}`;
            return;
          }
          // SB-MLA-007. A forced deletion is the one that changed what the project can defend, so
          // it goes in the permanent record with the reason it was refused — the curves are still
          // there and still cite it, and six months on the only question worth answering is whether
          // somebody knew that at the time. The refusal text carries the wells and curves, so the
          // record names them too rather than saying a deletion merely happened.
          recordProcess("ML", `Force-deleted saved model '${m.name}' while curves still cited it — ${msg}`);
        }
        await refreshSaved();
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
      // Regression only: there is no "missing frequency content" in a class code.
      spectral_texture: task.id === "regression" && specCb.checked,
    };
    for (const { spec, get } of paramInputs) params[spec.key] = get();
    const picked = alsoSelected();
    const multi = picked.length > 1;
    const base = outInput.value.trim() || task.defaultOut;
    const saveBase = saveInput.value.trim();
    // SB-MLA-033, refused here as well as in Rust — and this is the case Rust cannot catch on its
    // own. `Number("")` is 0, not NaN, so a blank low box would reach the backend as a perfectly
    // valid limit of zero and normalize the curve against a range nobody typed.
    if (stdCb.checked && normBasis === "limits") {
      const bad = features.filter((c) => {
        const l = basisLimits.get(c);
        if (!l || l.lo.trim() === "" || l.hi.trim() === "") return true;
        const [lo, hi] = [Number(l.lo), Number(l.hi)];
        return !Number.isFinite(lo) || !Number.isFinite(hi) || hi <= lo;
      });
      if (bad.length > 0) {
        setStatus(
          `Fixed limits need a low and a high above it for every input — ${bad.join(", ")} ${bad.length === 1 ? "does" : "do"} not have one yet`,
        );
        statusLine.textContent = `Set the limits for ${bad.join(", ")}, or switch the basis back to the data.`;
        return;
      }
    }
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
      coverage_segments: task.supervised && covCb.checked,
      output_step:
        task.supervised && resMode === "step" && Number(resStep.value) > 0 ? Number(resStep.value) : null,
      interval: fitInterval.getWindow(),
      // SB-MLA-033. Only sent when the user actually chose fixed limits, so an omitted field keeps
      // meaning "the data-derived basis" for every payload written before this existed. A blank box
      // travels as NaN and the backend refuses by name rather than treating it as zero.
      norm_basis: normBasis === "limits" ? "limits" : null,
      norm_limits:
        normBasis === "limits"
          ? features.map((curve) => {
              const l = basisLimits.get(curve) ?? { lo: "", hi: "" };
              return { curve, low: Number(l.lo), high: Number(l.hi) };
            })
          : [],
    };
    runBtn.disabled = true;
    statusLine.textContent = "Running…";
    const t0 = performance.now();
    try {
      // One request per algorithm, run in order. Everything that decides WHICH ROWS a model sees —
      // the wells, the curves, the mask, the split fraction, the split mode and the seed — is shared
      // verbatim, so the scores are comparable; only the algorithm, its parameters and the names it
      // writes under differ.
      const runs: { algo: AlgoSpec; res: MlResult }[] = [];
      for (const [i, a] of picked.entries()) {
        if (multi) statusLine.textContent = `Running ${a.label} (${i + 1} of ${picked.length})…`;
        // Distinct names once there is more than one model. Every run is suffixed, including the
        // picked one — privileging it with the bare name would make "which curve is the comparison
        // baseline" a question about the order boxes were ticked in.
        const one: MlRequest = {
          ...req,
          algorithm: a.id,
          // The picked algorithm takes the form's fields; every other takes its own ⚙ overrides
          // merged over the shared run settings. Only values the user actually CHANGED are sent —
          // passing a parameter at its own default would make the runner record it as
          // user-supplied, and SB-MLA-001 exists to keep those two apart.
          params: a.id === algo.id ? params : alsoParamsFor(a),
          output_curve: multi ? `${base}_${a.id.toUpperCase()}` : base,
          save_model_as: task.supervised && saveBase ? (multi ? `${saveBase}_${a.id.toUpperCase()}` : saveBase) : null,
        };
        runs.push({ algo: a, res: await runMl(one) });
      }
      const res = runs[0].res;
      const ms = Math.round(performance.now() - t0);
      if (multi) {
        const okRuns = runs.filter((r) => !r.res.error);
        const wrote = okRuns.flatMap((r) => r.res.outputs).join(", ");
        statusLine.textContent =
          `Done in ${ms} ms → ${okRuns.length}/${runs.length} model(s)` + (wrote ? ` → ${wrote}` : "");
        if (okRuns.length > 0) {
          recordProcess("ML", `Compared ${okRuns.length} model(s) on ${req.target_curve ?? "-"}: wrote ${wrote}`);
          setStatus(`Ran ${okRuns.length} model(s): ${wrote}`);
        }
        for (const r of runs) {
          if (r.res.model_name) {
            recordProcess("ML", `Saved model '${r.res.model_name}' (${r.algo.label} on ${req.target_curve ?? "-"})`);
          }
        }
        if (okRuns.some((r) => r.res.model_name)) {
          saveInput.value = "";
          void refreshSaved();
        }
        if (okRuns.length > 0) bumpDataVersion();
        renderMultiResults(results, runs, nameOf);
        if (okRuns.length > 0) showSection("results");
        return;
      }
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
      // Show what came out, since Run Model is on the Model tab and the answer is rendered two
      // sections away. Only on SUCCESS: `statusLine` sits in this section, so a failed run must
      // leave the user where its message is, rather than switching them to an empty Results panel
      // and reporting the failure on a tab they can no longer see.
      if (!res.error) showSection("results");
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  syncAlgo();
  return {
    el: content,
    dispose: () => {
      scope.dispose();
      distScope.dispose();
      fitInterval.dispose();
      distInterval.dispose();
    },
  };
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

  const segs = (res.metrics as Record<string, unknown> | null)?.["coverage_segments"];
  if (Array.isArray(segs)) renderCoverageSegments(host, segs as CoverageSegment[]);

  if (res.metrics && typeof res.metrics === "object") {
    renderEffectiveParams(host, res.metrics as Record<string, unknown>);
    const table = document.createElement("table");
    table.className = "mc-table";
    for (const [key, value] of Object.entries(res.metrics)) {
      if (key === "effective_params") continue; // shown as its own table above
      if (key === "coverage_segments") continue; // its own table above; `fmtMetric` would print JSON
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
 * Several models run in one click, compared at the top and detailed underneath.
 *
 * The comparison is over the CURVES, not over a cross-validation score — that is what separates this
 * from Compare algorithms. Every row here is a model that actually wrote a log and saved an artifact,
 * so the reading is "which of these would I deliver", not "which scores best on paper".
 *
 * The scores are comparable because the runs share the wells, curves, mask, split fraction, split
 * mode and seed; only the estimator differs. The caption says so, because a table of scores with no
 * statement of what was held constant is a table nobody can act on.
 *
 * A failed run keeps its row. Dropping it would leave a comparison that looks complete and quietly
 * excludes the model that could not fit — usually the most interesting fact on the screen.
 */
function renderMultiResults(
  host: HTMLElement,
  runs: { algo: AlgoSpec; res: MlResult }[],
  nameOf?: (id: string) => string,
): void {
  host.innerHTML = "";
  const cap = document.createElement("div");
  cap.className = "ml-seg-cap";
  cap.textContent =
    `${runs.length} models on the same wells, the same curves, the same split and the same seed — only the ` +
    "estimator differs, which is what makes these scores comparable. Each wrote its own curve and saved its " +
    "own model, so this compares what you would deliver rather than a cross-validation number.";
  host.appendChild(cap);

  const table = document.createElement("table");
  table.className = "mc-table ml-multi-table";
  const head = document.createElement("tr");
  for (const h of ["Model", "Curve", "Wells", "Blind", "Saved as"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);
  // Best blind score marked, and ONLY when more than one run produced one — a "best" badge on a
  // field of one is a ranking of nothing.
  const scoreOf = (r: MlResult): number | null => {
    const b = (r.metrics as Record<string, unknown> | null)?.["blind"] as Record<string, unknown> | undefined;
    return b?.["performed"] === true && typeof b["value"] === "number" ? (b["value"] as number) : null;
  };
  const scored = runs.map((r) => scoreOf(r.res)).filter((v): v is number => v != null);
  const best = scored.length > 1 ? Math.max(...scored) : null;

  for (const { algo, res } of runs) {
    const tr = document.createElement("tr");
    if (res.error) tr.classList.add("ml-well-refused");
    const s = scoreOf(res);
    const ok = res.wells.filter((w) => !w.error).length;
    const cells = res.error
      ? [algo.label, "—", "—", "—", "—"]
      : [
          algo.label,
          res.outputs.join(", ") || "—",
          `${ok}/${res.wells.length}`,
          s == null ? "not requested" : s.toFixed(4),
          res.model_name ?? "not saved",
        ];
    for (const [i, c] of cells.entries()) {
      const td = document.createElement("td");
      td.textContent = c;
      if (i === 3 && s != null && best != null && s === best) {
        td.classList.add("ml-diag");
        td.title = "Highest blind score of these runs — read it beside the crossplot before trusting it";
      }
      tr.appendChild(td);
    }
    table.appendChild(tr);
    if (res.error) {
      const why = document.createElement("tr");
      const td = document.createElement("td");
      td.colSpan = 5;
      td.className = "ml-seg-why";
      td.textContent = res.error;
      why.appendChild(td);
      table.appendChild(why);
    }
  }
  host.appendChild(table);

  // The full per-run result, collapsed. Everything the single-model view shows is still reachable —
  // notes, split report, per-well outcomes — without burying the comparison under five copies of it.
  for (const { algo, res } of runs) {
    const box = document.createElement("details");
    box.className = "ml-eff";
    const sum = document.createElement("summary");
    sum.textContent = `${algo.label} — full result`;
    box.appendChild(sum);
    const inner = document.createElement("div");
    if (res.error) {
      inner.className = "mc-note mc-note-err";
      inner.textContent = res.error;
    } else {
      renderResults(inner, res, nameOf);
    }
    box.appendChild(inner);
    host.appendChild(box);
  }
}

/**
 * The models a coverage-segmented run fitted, one CARD each, never summarised into one.
 *
 * The output is a single curve, and the temptation is to report it with a single score. There isn't
 * one. A four-curve model and a three-curve model fitted on different rows are different models; an
 * R² over both would be a number that describes neither, and it would describe the shallow half of
 * the well and the deep half equally when the whole point of the run is that they are not equally
 * known.
 *
 * Cards rather than a table, and not only because a six-column table does not fit a form column. A
 * table invites reading DOWN a column and comparing rows, which is the one reading these numbers
 * must not get — 0.81 beside 0.64 is not a ranking, it is two models answering over different rock,
 * and the segment with the lower score is not the worse model, it is the one that had less to work
 * with. Cards carry that separation in the layout instead of only in a caption.
 *
 * A segment that was NOT fitted keeps its card and states why in full. A skipped model that simply
 * vanished would leave blank rock with no visible cause.
 */
function renderCoverageSegments(host: HTMLElement, segments: CoverageSegment[]): void {
  if (!segments.length) return;
  const fitted = segments.filter((s) => !s.skipped);
  const totalDepths = fitted.reduce((a, s) => a + s.n_predicted, 0);

  const box = document.createElement("div");
  box.className = "ml-seg";
  const cap = document.createElement("div");
  cap.className = "ml-seg-cap";
  cap.textContent =
    fitted.length === 1
      ? "One model covered every predicted depth — the selected inputs are present or absent together."
      : `${fitted.length} models, one per pattern of available inputs. Each is scored on its own rows. They are not ranked against one another: the smaller model is not worse, it is the one that had fewer curves to work with.`;
  box.appendChild(cap);

  for (const s of segments) {
    const card = document.createElement("div");
    card.className = s.skipped ? "ml-seg-card ml-seg-card-skip" : "ml-seg-card";

    const title = document.createElement("div");
    title.className = "ml-seg-title";
    const names = document.createElement("span");
    names.className = "ml-seg-names";
    names.textContent = s.features.join(" + ");
    const count = document.createElement("span");
    count.className = "ml-seg-count";
    count.textContent = `${s.features.length} curve${s.features.length === 1 ? "" : "s"}`;
    title.append(names, count);
    card.appendChild(title);

    if (s.skipped) {
      const why = document.createElement("div");
      why.className = "ml-seg-why";
      why.textContent = s.skipped;
      card.appendChild(why);
    } else {
      const blind = (s.blind ?? {}) as Record<string, unknown>;
      // "Not requested" and "requested, and it scored 0.61" are different statements and a dash for
      // both would merge them. The protocol rides along, because a random-row split does not answer
      // the question a whole-well one does — the same rule the run-level split report follows.
      const score =
        blind["performed"] === true
          ? `${String(blind["metric"] ?? "")} ${fmtMetric(blind["value"])}`
          : "not requested";
      const scoreNote =
        blind["performed"] === true
          ? blind["answers_new_well"] === true
            ? "on wells held out whole"
            : "on held-out rows — not a new-well score"
          : "no blind test was asked for";
      const stats = document.createElement("div");
      stats.className = "ml-seg-stats";
      const share = totalDepths > 0 ? Math.round((s.n_predicted / totalDepths) * 100) : null;
      const cells: [string, string, string][] = [
        ["Predicts", s.n_predicted.toLocaleString(), share == null ? "depths" : `depths — ${share}% of the curve`],
        ["Fitted on", s.n_train.toLocaleString(), "training rows"],
        ["Blind", score, scoreNote],
      ];
      for (const [k, v, note] of cells) {
        const cell = document.createElement("div");
        cell.className = "ml-seg-cell";
        const kk = document.createElement("span");
        kk.className = "ml-seg-k";
        kk.textContent = k;
        const vv = document.createElement("span");
        vv.className = "ml-seg-v";
        vv.textContent = v;
        const nn = document.createElement("span");
        nn.className = "ml-seg-n";
        nn.textContent = note;
        cell.append(kk, vv, nn);
        stats.appendChild(cell);
      }
      card.appendChild(stats);

      const foot = document.createElement("div");
      foot.className = "ml-seg-foot";
      foot.textContent = s.model_name
        ? `Saved as ${s.model_name}`
        : "Not saved — this segment's model exists only for this run";
      card.appendChild(foot);
    }
    box.appendChild(card);
  }
  host.appendChild(box);
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
  renderScoreChart(host, res, isClf, tied);
  renderMetricBars(host, res, isClf);
  renderBlindCrossplot(host, res, isClf);
  renderPredictorConsensus(host, res);
  host.appendChild(detail);
  if (firstOkRow) renderEvalDetail(detail, firstOkRow, isClf);
}

/**
 * The two headline scores side by side, one bar chart each.
 *
 * Jauhar, 2026-08-07: *"visualization should also provide histogram of r2 and rmse comparison
 * between models"*. They are drawn as a PAIR and never on one axis, because they are different
 * quantities in different units pointing in opposite directions — R² is dimensionless and higher is
 * better, RMSE is in the target's own units and lower is better. Sharing an axis would put the best
 * model at opposite ends of the same picture.
 *
 * Both are sorted by R² — the same order as the table and the score chart — rather than each by its
 * own metric. Two differently-ordered charts side by side invite the reader to compare rows by
 * position, and here position would mean two different things.
 *
 * The disagreement between them is the reason to draw both. R² is a share of variance explained, so
 * it flatters a model tested over a wide range and punishes one tested over a narrow one; RMSE says
 * how wrong the prediction is in the units the answer will be quoted in. A model that wins on R² and
 * loses on RMSE has been scored on a broader spread of rock, not fitted better.
 *
 * Exported for the vite dev server.
 */
export function renderMetricBars(host: HTMLElement, res: MlEvalResult, isClf: boolean): void {
  const ok = res.rows.filter((r) => !r.error && r.score != null);
  if (ok.length < 2) return;
  const secKey = isClf ? "macro_f1" : "rmse";
  const secLabel = isClf ? "macro-F1" : "RMSE";
  // "Higher is better" for accuracy and macro-F1; for RMSE it is the opposite, and the caption has
  // to say so or the longest bar reads as the winner.
  const secBetterHigh = isClf;
  const sec = ok.map((r) => {
    const v = r.metrics?.[secKey];
    return typeof v === "number" && Number.isFinite(v) ? v : null;
  });
  if (sec.every((v) => v == null)) return;

  const wrap = document.createElement("div");
  wrap.className = "ml-chart ml-metric-bars";
  const cap = document.createElement("div");
  cap.className = "mc-chain-note";
  cap.textContent = `${isClf ? "Accuracy" : "R²"} and ${secLabel} across models — both in the score order above`;
  wrap.appendChild(cap);

  const grid = document.createElement("div");
  grid.className = "ml-bars-grid";

  const panel = (
    title: string,
    values: (number | null)[],
    betterHigh: boolean,
    fmt: (v: number) => string,
  ): HTMLElement => {
    const col = document.createElement("div");
    col.className = "ml-bars-col";
    const h = document.createElement("div");
    h.className = "ml-bars-title";
    h.textContent = `${title} — ${betterHigh ? "higher is better" : "lower is better"}`;
    col.appendChild(h);
    const finite = values.filter((v): v is number => v != null);
    const max = Math.max(...finite, 0);
    const min = Math.min(...finite, 0);
    // Bars from a common zero, so length is proportional to the value rather than to its distance
    // from an arbitrary floor. A truncated axis is how two near-identical models come to look like a
    // landslide, which is the exact misreading `tiedAtTheTop` exists to prevent.
    const span = Math.max(Math.abs(max), Math.abs(min), 1e-9);
    // The best value on THIS metric, which need not be the top row.
    const best = finite.length ? (betterHigh ? Math.max(...finite) : Math.min(...finite)) : null;
    ok.forEach((r, i) => {
      const v = values[i];
      const line = document.createElement("div");
      line.className = "ml-bar-row";
      const name = document.createElement("span");
      name.className = "ml-bar-name";
      name.textContent = r.algorithm;
      name.title = r.features.join(", ");
      const track = document.createElement("div");
      track.className = "ml-bar-track";
      const bar = document.createElement("div");
      bar.className = "ml-bar";
      // A negative R² is a real and important reading — the model is worse than predicting the
      // mean — so it is drawn, on its own side of zero, rather than clamped to nothing.
      if (v != null && v < 0) bar.classList.add("ml-bar-neg");
      bar.style.width = v == null ? "0%" : `${(Math.abs(v) / span) * 100}%`;
      if (v != null && best != null && v === best) bar.classList.add("ml-bar-best");
      track.appendChild(bar);
      const val = document.createElement("span");
      val.className = "ml-bar-val";
      val.textContent = v == null ? "—" : fmt(v);
      line.append(name, track, val);
      col.appendChild(line);
    });
    return col;
  };

  grid.append(
    panel(isClf ? "Accuracy" : "R²", ok.map((r) => r.score as number), true, (v) => v.toFixed(3)),
    panel(secLabel, sec, secBetterHigh, (v) => (Math.abs(v) >= 0.01 ? v.toFixed(4) : v.toExponential(2))),
  );
  wrap.appendChild(grid);

  // The two metrics disagreeing is a finding, not a glitch, and it is the one an experienced eye is
  // looking for. Said only when it happens.
  const bestScoreIdx = 0; // rows arrive sorted by score
  let bestSecIdx = -1;
  sec.forEach((v, i) => {
    if (v == null) return;
    const cur = bestSecIdx >= 0 ? sec[bestSecIdx] : null;
    if (cur == null || (secBetterHigh ? v > cur : v < cur)) bestSecIdx = i;
  });
  if (bestSecIdx >= 0 && bestSecIdx !== bestScoreIdx) {
    const note = document.createElement("div");
    note.className = "mc-chain-note ml-consensus-note";
    note.textContent = isClf
      ? `${ok[bestScoreIdx].algorithm} has the best accuracy and ${ok[bestSecIdx].algorithm} the best macro-F1. ` +
        `Accuracy is dominated by the commonest class, macro-F1 weights every class equally — so the second ` +
        `model is doing better on the thin facies, which is usually the ones worth predicting.`
      : `${ok[bestScoreIdx].algorithm} has the best R² and ${ok[bestSecIdx].algorithm} the lowest RMSE. ` +
        `R² is a share of variance explained and rewards a model tested over a wide range; RMSE is the error ` +
        `in the units the answer gets quoted in. Where they disagree, RMSE is the one a volumetric inherits.`;
    wrap.appendChild(note);
  }
  host.appendChild(wrap);
}

/**
 * Predicted against measured, on rows the model did not see.
 *
 * Jauhar, 2026-08-07: *"result visualization should provide xplot prediction vs blind test either
 * for model user wanna see (provide all, but shows depend on user)"*. So every model's points are
 * computed and carried back; the picker chooses which one is drawn. Computing them all costs
 * nothing extra — the out-of-fold predictions already existed inside the cross-validation and were
 * being discarded once the score was taken from them.
 *
 * **The 1:1 line is the subject, not decoration.** A score says how close the cloud is to it; the
 * picture says HOW it misses, and those are different findings with different fixes. A model that
 * is tight but rotated off 1:1 is mis-scaled and can be corrected; one that is flat at the mean has
 * learned nothing and cannot; one that tracks well then saturates at the top end has run out of
 * calibration range, which is a coring problem rather than a modelling one. All three can share an
 * R².
 *
 * **Coloured by well, because that is the reading the aggregate cannot give.** A blind R² of 0.7
 * over three wells can be 0.9, 0.85 and 0.1 — and the third well is the one that says whether this
 * curve travels. Ordered by first appearance so a well keeps its colour between models.
 *
 * Exported for the vite dev server.
 */
export function renderBlindCrossplot(host: HTMLElement, res: MlEvalResult, isClf: boolean): void {
  // A classifier's predicted-vs-actual is the confusion matrix, which the detail panel already
  // draws. Plotting class codes on two continuous axes would invite reading the distance between
  // facies 1 and facies 4 as three of something.
  if (isClf) return;
  const ok = res.rows.filter((r) => !r.error && r.blind_pred?.length);
  if (ok.length === 0 || !res.blind_actual?.length) return;

  const wrap = document.createElement("div");
  wrap.className = "ml-chart ml-xplot";
  const bar = document.createElement("div");
  bar.className = "ml-xplot-bar";
  const cap = document.createElement("div");
  cap.className = "mc-chain-note";
  const pick = document.createElement("select");
  pick.className = "form-control ml-xplot-pick";
  ok.forEach((r, i) => {
    const o = document.createElement("option");
    o.value = String(i);
    o.textContent = `${r.algorithm} · ${r.features.length} curve(s) · R² ${r.score?.toFixed(3) ?? "—"}`;
    pick.appendChild(o);
  });
  bar.append(cap, pick);
  wrap.appendChild(bar);
  const plotHost = document.createElement("div");
  wrap.appendChild(plotHost);

  const wells = [...new Set(res.blind_well)];
  const draw = (i: number): void => {
    const row = ok[i];
    plotHost.innerHTML = "";
    cap.textContent =
      `Predicted vs measured on held-out rows · ${res.blind_sampled.toLocaleString()} of ` +
      `${res.blind_total.toLocaleString()} shown · both axes on one scale, so the dashed line is 1:1`;
    const scatter = blindScatterSvg(res, row, wells);
    plotHost.appendChild(scatter);
    // Re-attached on every redraw, because the export has to copy the model currently on screen. A
    // toolbar built once beside a picker that swaps the chart under it would quietly go on
    // exporting whichever model happened to be drawn first.
    attachChartExport(plotHost, scatter, `ML predicted vs measured - ${row.algorithm}`);
    const legend = document.createElement("div");
    legend.className = "ml-xplot-legend";
    wells.forEach((w, wi) => {
      const chip = document.createElement("span");
      chip.className = "ml-xplot-chip";
      const dot = document.createElement("span");
      dot.className = "ml-xplot-dot";
      dot.style.background = wellColor(wi);
      chip.append(dot, document.createTextNode(w));
      legend.appendChild(chip);
    });
    plotHost.appendChild(legend);
  };
  pick.addEventListener("change", () => draw(Number(pick.value)));
  draw(0);
  host.appendChild(wrap);
}

/** Per-well colour for the crossplot. Reads from the app's own facies palette so a well keeps a
 *  recognisable colour beside the facies tracks, and cycles rather than running out. */
function wellColor(i: number): string {
  return FACIES_PALETTE[i % FACIES_PALETTE.length];
}

function blindScatterSvg(res: MlEvalResult, row: MlEvalRow, wells: string[]): SVGSVGElement {
  const pts: { x: number; y: number; w: number }[] = [];
  for (let i = 0; i < res.blind_actual.length; i++) {
    const x = res.blind_actual[i];
    const y = row.blind_pred[i];
    if (x == null || y == null || !Number.isFinite(x) || !Number.isFinite(y)) continue;
    pts.push({ x, y, w: Math.max(0, wells.indexOf(res.blind_well[i] ?? "")) });
  }
  const size = 360;
  const pad = 42;
  const svg = svgEl("svg", { viewBox: `0 0 ${size} ${size}`, class: "ml-xplot-svg", width: "100%" });
  if (pts.length === 0) return svg;

  // ONE range for both axes. A predicted-vs-measured plot is read against the 1:1 line, and two
  // independently scaled axes would draw that line at 45° whatever the model did — turning a
  // systematic bias into a picture of a perfect fit.
  const lo = Math.min(...pts.map((p) => Math.min(p.x, p.y)));
  const hi = Math.max(...pts.map((p) => Math.max(p.x, p.y)));
  const m = (hi - lo) * 0.05 || 1e-6;
  const a = lo - m;
  const b = hi + m;
  const X = (v: number) => pad + ((v - a) / (b - a)) * (size - pad - 10);
  const Y = (v: number) => size - pad - ((v - a) / (b - a)) * (size - pad - 10);

  svg.appendChild(svgEl("rect", { x: pad, y: 10, width: size - pad - 10, height: size - pad - 10, class: "ml-xp-frame" }));
  svg.appendChild(svgEl("line", { x1: X(a), y1: Y(a), x2: X(b), y2: Y(b), class: "ml-xp-unity" }));
  for (const p of pts) {
    const c = svgEl("circle", { cx: X(p.x), cy: Y(p.y), r: 1.9, class: "ml-xp-pt" });
    c.setAttribute("fill", wellColor(p.w));
    svg.appendChild(c);
  }
  const lab = (x: number, y: number, s: string, cls: string, anchor = "middle") => {
    const t = svgEl("text", { x, y, class: cls, "text-anchor": anchor });
    t.textContent = s;
    svg.appendChild(t);
  };
  const f = (v: number) => (Math.abs(v) >= 0.01 && Math.abs(v) < 1e5 ? v.toFixed(2) : v.toExponential(1));
  lab(X(a), size - pad + 14, f(a), "ml-xp-axis", "start");
  lab(X(b), size - pad + 14, f(b), "ml-xp-axis", "end");
  lab(size / 2, size - 6, "measured", "ml-xp-axis");
  const yl = svgEl("text", { x: 12, y: size / 2, class: "ml-xp-axis", "text-anchor": "middle", transform: `rotate(-90 12 ${size / 2})` });
  yl.textContent = "predicted (out of fold)";
  svg.appendChild(yl);
  // Only the y MAX is labelled. Both axes deliberately share one range — that is what makes the
  // dashed line 1:1 — so the y minimum is the same number as the x minimum, printed a few pixels
  // away from it in the corner where the two axes meet. Two identical numbers touching read as a
  // rendering fault, and the second one carries nothing the first did not.
  lab(pad - 5, Y(b) + 4, f(b), "ml-xp-axis", "end");
  return svg;
}

/** What the Data QC section was asked about. Passed as one object because every check needs several
 *  of these and a nine-argument call is a transposition waiting to happen. */
export interface QcContext {
  /** Every curve the run reads — inputs AND the target. A row reaches the fit only where all of
   *  them exist, so this is the set the coverage question is asked over. */
  curves: string[];
  /** The inputs alone. Kept SEPARATE from `curves` because the scale question is about what the
   *  estimator is fed, and the target is not fed to it: a permeability target spanning four decades
   *  cannot swamp a distance calculation it never enters. Reported against the full list, that check
   *  raises a confident, plausible, wrong alert on every core-calibrated regression — which is worse
   *  than not checking, because the fix it names (standardize) would not change the answer. */
  inputs: string[];
  /** Null for an unsupervised run, where there is no target to be short of. */
  target: string | null;
  wells: number;
  algorithm: string;
  algorithmLabel: string;
  taskId: string;
  /** True where the estimator ignores the scale of its inputs (trees, naive Bayes, linear). */
  scaleFree: boolean;
  /** Requested cluster count, where the algorithm has one. */
  k: number | null;
  standardize: boolean;
  masked: boolean;
  /** True when the run will fit one model per pattern of available inputs instead of one model over
   *  the intersection. It changes what a short curve COSTS, so several findings below read
   *  differently with it on — and a checklist that ignored the setting would keep recommending a fix
   *  the user has already applied. */
  coverage: boolean;
}

interface QcFinding {
  level: "ok" | "warn" | "alert";
  title: string;
  detail: string;
}

/**
 * Whether the selected data can support the selected model.
 *
 * Every finding here is a statement about the pair, never about the data alone, because "is this
 * data good" has no answer — a spread of four orders of magnitude between two curves is fatal to
 * k-means and completely irrelevant to a random forest, and a checklist that reported it either way
 * would be wrong half the time in whichever direction the reader was not expecting.
 *
 * Findings are RANKED, worst first, and a clean run says so rather than showing nothing: an empty
 * panel reads as "the check did not run".
 *
 * Exported so it can be driven with synthetic rows over the vite dev server.
 */
export function qcFindings(
  rows: CurveStatsRow[],
  sampling: [string, CurveSampling[]][],
  ctx: QcContext,
): QcFinding[] {
  const out: QcFinding[] = [];

  // --- 0. Sampling, before anything that depends on it ----------------------
  //
  // Jauhar, 2026-08-07: *"each log has different resolution, sometimes it looks low frequency such
  // as resistivity, sometimes high such as rxo, gr, or nphi"*. Every read in this application aligns
  // curves onto the well's frame by EXACT depth match, which is right and cheap when the curves came
  // from one delivery — and returns nothing at all when they did not. A resistivity delivered on a
  // 0.5 m grid, joined onto a 0.1524 m frame, coincides at no depth: fully logged, fully stored, and
  // it reads as absent.
  //
  // Reported FIRST and separately from coverage, because "missing" and "present on another grid"
  // send an interpreter in opposite directions — and the second one previously reported itself as
  // the first, which sends them looking for a log they already have.
  const offGrid = new Map<string, { wells: string[]; own: number; step: number | null }>();
  const steps = new Map<string, number[]>();
  for (const [, curves] of sampling) {
    for (const s of curves) {
      if (s.step != null && s.step > 0) {
        const list = steps.get(s.curve) ?? [];
        list.push(s.step);
        steps.set(s.curve, list);
      }
      // The signature of the fault: real samples, none of which land on the frame. A curve with a
      // FEW landing is a different situation (partial overlap) and is left to the coverage check,
      // which measures it properly.
      if (s.n_own > 0 && s.n_on_frame === 0) {
        const e = offGrid.get(s.curve) ?? { wells: [], own: 0, step: s.step };
        e.wells.push("");
        e.own += s.n_own;
        offGrid.set(s.curve, e);
      }
    }
  }
  for (const [, curves] of sampling) {
    for (const s of curves) {
      const e = offGrid.get(s.curve);
      if (e && s.n_own > 0 && s.n_on_frame === 0 && e.step == null) e.step = s.step;
    }
  }
  for (const [curve, e] of offGrid) {
    out.push({
      level: "alert",
      title: `${curve} is logged but sits on a different depth grid`,
      detail:
        `It holds ${e.own.toLocaleString()} samples${e.step ? ` at about ${e.step.toFixed(4)} spacing` : ""}, ` +
        "and not one of them falls on a depth this well's other curves use. Curves are joined by exact " +
        "depth, so every read of it comes back blank and every row it touches is dropped. This is not a " +
        "missing curve — do not go looking for it. Re-frame it onto the well's own sampling " +
        "(Data ▸ Frame ▸ Resample) and it will come straight in.",
    });
  }
  // The softer version: the curves ARE on one grid, but they were logged at different rates. Nothing
  // is lost here — the join still works — but a curve carrying a value every 0.5 m beside one
  // carrying a value every 0.1 m contributes a fifth as many rows, and its influence on the fit is
  // the same fraction. Worth knowing, not worth stopping for.
  const rates = [...steps.entries()]
    .map(([curve, list]) => ({ curve, step: list.reduce((a, b) => a + b, 0) / list.length }))
    .filter((r) => r.step > 0);
  if (rates.length > 1) {
    const fine = rates.reduce((a, b) => (b.step < a.step ? b : a));
    const coarse = rates.reduce((a, b) => (b.step > a.step ? b : a));
    const ratio = coarse.step / fine.step;
    if (ratio > 1.5 && !offGrid.has(coarse.curve)) {
      out.push({
        level: "warn",
        title: `${coarse.curve} is sampled about ${ratio < 10 ? ratio.toFixed(1) : Math.round(ratio)}× more coarsely than ${fine.curve}`,
        detail:
          `${coarse.step.toFixed(4)} against ${fine.step.toFixed(4)}. Both are on the well's frame, so nothing is ` +
          `lost — but ${coarse.curve} answers for a fraction of the depths, and every row where it is blank is ` +
          `dropped from the fit entirely, taking the finely-sampled curves at that depth with it. Sampling is ` +
          "not the same as vertical resolution: how often a tool was read tells you nothing about how thin a bed " +
          "it can see, and neither number is something SandiBumi can infer for you.",
      });
    }
  }

  const byCurve = new Map<string, CurveStatsRow[]>();
  for (const r of rows) {
    const list = byCurve.get(r.curve) ?? [];
    list.push(r);
    byCurve.set(r.curve, list);
  }
  const total = (c: string, f: (r: CurveStatsRow) => number) =>
    (byCurve.get(c) ?? []).reduce((a, r) => a + f(r), 0);

  // --- 1. Is there anything to fit at all -----------------------------------
  // A model needs every input AND the target present at the SAME depth, and a run where each curve
  // is 90% complete but their gaps do not overlap has far fewer usable rows than any single column
  // suggests. The intersection cannot be recovered from per-curve totals, so what is reported is the
  // LIMIT it cannot exceed, named as such.
  //
  // This one is pushed FIRST and stays first whatever its level, because it is the scale everything
  // below it is read against. Sorted with the rest it would sink under three warnings whenever the
  // data was fine, which is exactly when a reader most needs to know they have 140 rows and not
  // 14,000.
  const counts = ctx.curves.map((c) => ({ curve: c, n: total(c, (r) => r.n) }));
  const thinnest = counts.reduce((a, b) => (b.n < a.n ? b : a), counts[0] ?? { curve: "", n: 0 });
  // Rows per input rather than an absolute floor: 500 rows is comfortable for two curves and thin
  // for eight, and a fixed number would be wrong at both ends.
  const perInput = thinnest.n / Math.max(1, ctx.inputs.length);
  const headline: QcFinding =
    thinnest.n === 0
      ? {
          level: "alert",
          title: `${thinnest.curve} has no samples in the selected wells`,
          detail:
            "Nothing can be fitted. Either the curve is absent from these wells, the mask excluded all of it, " +
            "or the input log set does not carry it.",
        }
      : perInput < 30
        ? {
            level: "alert",
            title: `At most ${thinnest.n.toLocaleString()} rows for ${ctx.inputs.length} input(s) — limited by ${thinnest.curve}`,
            detail:
              `About ${Math.round(perInput)} rows per input curve. At that ratio almost any algorithm here will fit ` +
              "the noise and score well doing it. Use fewer inputs, or more wells.",
          }
        : // Under coverage segmentation the cap belongs to the segment using every input — but only
          // when the capping curve IS an input. The target is needed by every segment by definition,
          // so a target-capped run is capped everywhere, and saying otherwise would promise rows that
          // segmentation cannot produce. Core permeability is usually the thinnest curve in a run,
          // which makes this the common case rather than the corner one.
          !ctx.coverage
          ? {
              level: "ok",
              title: `At most ${thinnest.n.toLocaleString()} rows can reach the fit`,
              detail:
                `Capped by ${thinnest.curve}, the shortest of the selected curves — about ${Math.round(perInput)} rows ` +
                "per input. The real count is lower wherever the curves' gaps do not line up, and the run reports it.",
            }
          : thinnest.curve === ctx.target
            ? {
                level: "ok",
                title: `At most ${thinnest.n.toLocaleString()} rows can reach ANY model`,
                detail:
                  `Capped by ${thinnest.curve} — about ${Math.round(perInput)} rows per input. Fitting a model per ` +
                  "available-input pattern does not lift this one: the target is what every model is fitted against, so " +
                  "no segment can see a depth it does not reach. What segmentation buys here is coverage of the " +
                  "PREDICTION, not more training data.",
              }
            : {
                level: "ok",
                title: `At most ${thinnest.n.toLocaleString()} rows can reach the FULLEST model`,
                detail:
                  `Capped by ${thinnest.curve}, the shortest of the selected curves — about ${Math.round(perInput)} rows ` +
                  "per input. The smaller models fitted alongside it are not bound by that curve at all; each one's own " +
                  "row count is reported with its blind score in the result.",
              };

  // --- 2. Coverage per curve ------------------------------------------------
  for (const c of ctx.curves) {
    const n = total(c, (r) => r.n);
    const miss = total(c, (r) => r.n_missing);
    const denom = n + miss;
    if (denom === 0) continue;
    const share = miss / denom;
    const empties = (byCurve.get(c) ?? []).filter((r) => r.n === 0);
    if (empties.length > 0) {
      out.push({
        level: "warn",
        title: `${c} is missing entirely from ${empties.length} of ${ctx.wells} well(s)`,
        detail:
          `${empties.map((r) => r.well).slice(0, 6).join(", ")}${empties.length > 6 ? `, and ${empties.length - 6} more` : ""}. ` +
          "Those wells contribute nothing to the fit and are not a smaller contribution — they are absent. " +
          "Drop the curve or drop the wells; leaving both narrows the model to whichever wells happen to have everything.",
      });
    } else if (share > 0.3) {
      // The cost is stated in ROWS THE OTHER CURVES LOSE, not as a coverage percentage. A curve
      // covering 48% of the well sounds like a partial input; what it actually does is delete 52% of
      // every OTHER curve's rows as well, because a depth is used only where every input has a
      // value. The second framing is the one that changes a decision — usually to drop the curve.
      const best = Math.max(...counts.map((x) => x.n));
      const lost = Math.max(0, best - n);
      out.push(
        ctx.coverage
          ? {
              // With segmentation on, a short curve stops being a choice between keeping it and
              // keeping the rock. It is still worth SAYING, because the interval now carries two
              // models and how well the answer is known genuinely varies down the well.
              level: "ok",
              title: `${c} covers ${Math.round((1 - share) * 100)}% of the interval — the run will fit a model with it and a model without`,
              detail:
                `The ${lost.toLocaleString()} depths ${c} does not reach keep their other curves and are predicted by a ` +
                "separate, smaller model, so no rock is dropped for want of one log. Read the two blind scores in the " +
                "result separately: they are different models on different inputs, and the interval is better known " +
                "where the fuller one answers.",
            }
          : {
              level: "warn",
              title: `${c} covers ${Math.round((1 - share) * 100)}% of the interval, and costs the run ${lost.toLocaleString()} rows`,
              detail:
                `A depth is used only where EVERY input has a value, so the ${lost.toLocaleString()} depths ${c} does not ` +
                "reach are dropped whole — taking the curves that were logged there with them. Three ways out: drop " +
                `the curve and gain ${lost.toLocaleString()} rows, drop the wells it is short in, or turn on "Fit a model per ` +
                'available-input pattern" in Model, which fits one model where it exists and a smaller one where it ' +
                "does not. The leaderboard is the honest way to choose between the first two.",
            },
      );
    }
  }

  // --- 3. Scale, and ONLY where scale matters -------------------------------
  // Over `inputs`, never `curves` — the target is not fed to the estimator, so it cannot dominate a
  // distance. See the field's own note.
  const spreads = ctx.inputs
    .map((c) => {
      const rs = (byCurve.get(c) ?? []).filter((r) => r.std != null && (r.std as number) > 0);
      if (rs.length === 0) return null;
      // Pooled by well count rather than by sample count: this is a question about the size of the
      // numbers, not a precise statistic, and one enormous well should not decide it alone.
      return { curve: c, sd: rs.reduce((a, r) => a + (r.std as number), 0) / rs.length };
    })
    .filter((v): v is { curve: string; sd: number } => v != null);
  if (spreads.length > 1) {
    const hi = spreads.reduce((a, b) => (b.sd > a.sd ? b : a));
    const lo = spreads.reduce((a, b) => (b.sd < a.sd ? b : a));
    const ratio = hi.sd / lo.sd;
    if (ctx.scaleFree) {
      out.push({
        level: "ok",
        title: `Scale does not matter to ${ctx.algorithmLabel}`,
        detail:
          `${hi.curve} varies about ${ratio < 10 ? ratio.toFixed(1) : Math.round(ratio)}× as widely as ${lo.curve}, ` +
          "which would dominate a distance-based model. This one splits one curve at a time on a threshold, " +
          "so it is unaffected either way.",
      });
    } else if (ratio > 20 && !ctx.standardize) {
      out.push({
        level: "alert",
        title: `${hi.curve} would swamp every other input`,
        detail:
          `It varies about ${Math.round(ratio)}× as widely as ${lo.curve}, and ${ctx.algorithmLabel} measures ` +
          "distance across all inputs at once — so that curve decides the answer whether or not it carries any " +
          "information. Turn on Standardize inputs in the Model section.",
      });
    } else if (ratio > 20) {
      out.push({
        level: "ok",
        title: "Standardizing is doing real work here",
        detail:
          `${hi.curve} varies about ${Math.round(ratio)}× as widely as ${lo.curve}. Without the z-score it would ` +
          `decide every distance ${ctx.algorithmLabel} measures. The scaler is stored with the model, so an ` +
          "apply run uses the same transform rather than refitting one.",
      });
    }
  }

  // --- 4. The target, where there is one ------------------------------------
  if (ctx.target) {
    const t = byCurve.get(ctx.target) ?? [];
    const tn = t.reduce((a, r) => a + r.n, 0);
    const wellsWithTarget = t.filter((r) => r.n > 0).length;
    if (tn === 0) {
      out.push({
        level: "alert",
        title: `The target ${ctx.target} has no samples at all`,
        detail: "Nothing is labelled, so there is nothing to learn. Pick a target the training wells actually carry.",
      });
    } else if (wellsWithTarget < 2) {
      out.push({
        level: "alert",
        title: `Only ${wellsWithTarget} well carries ${ctx.target}`,
        detail:
          "A model fitted in one well has no way to be validated on another, and its cross-validation score is " +
          "measured on folds of the same well. Nothing here can tell you whether it travels.",
      });
    } else if (wellsWithTarget < ctx.wells) {
      out.push({
        level: "warn",
        title: `${ctx.target} is present in ${wellsWithTarget} of ${ctx.wells} training wells`,
        detail:
          "The rest contribute no labelled rows. That is not an error — it is the usual shape of core coverage — " +
          "but the model is learning from those wells only, and a blind test can hold back at most " +
          `${wellsWithTarget - 1} of them.`,
      });
    }

    // A classifier fitted on a continuous target is a common and expensive mistake: it computes,
    // labels every distinct value a class, and the accuracy is meaningless. The tell is the number
    // of distinct values, which the summary cannot give directly — but a target whose range spans
    // far more than a handful of integers is not a class code.
    if (ctx.taskId === "classification") {
      const lo = Math.min(...t.filter((r) => r.min != null).map((r) => r.min as number));
      const hi = Math.max(...t.filter((r) => r.max != null).map((r) => r.max as number));
      if (Number.isFinite(lo) && Number.isFinite(hi) && (hi - lo > 50 || !Number.isInteger(lo) || !Number.isInteger(hi))) {
        out.push({
          level: "alert",
          title: `${ctx.target} does not look like a class code`,
          detail:
            `It runs from ${lo.toFixed(3)} to ${hi.toFixed(3)}. A classifier will treat every distinct value as its ` +
            "own class, fit happily, and report an accuracy that means nothing. If this is a continuous log, " +
            "choose a Continuous log algorithm instead.",
        });
      }
    }
    if (ctx.taskId === "regression") {
      const lo = Math.min(...t.filter((r) => r.min != null).map((r) => r.min as number));
      const hi = Math.max(...t.filter((r) => r.max != null).map((r) => r.max as number));
      if (Number.isFinite(lo) && Number.isFinite(hi) && hi > 0 && lo >= 0 && hi / Math.max(lo, 1e-9) > 1000) {
        out.push({
          level: "warn",
          title: `${ctx.target} spans more than three orders of magnitude`,
          detail:
            `${lo.toExponential(1)} to ${hi.toExponential(1)}. Fitted as measured, the largest values dominate the ` +
            "error and the low end is fitted almost not at all. Set Fit target as → log10 in the Model section, " +
            "which is the space this relation is usually linear in.",
        });
      }
    }
  }

  // --- 5. Cluster count against what the data can carry ---------------------
  if (ctx.k && ctx.k > 1) {
    const perWell = thinnest.n / Math.max(1, ctx.wells);
    if (thinnest.n / ctx.k < 30) {
      out.push({
        level: "alert",
        title: `${ctx.k} classes over about ${thinnest.n.toLocaleString()} rows`,
        detail:
          `That is roughly ${Math.round(thinnest.n / ctx.k)} samples per class. Below about thirty, a class is a ` +
          "handful of depths and its mean is not a rock property. Lower K or add wells.",
      });
    } else {
      out.push({
        level: "ok",
        title: `${ctx.k} classes over about ${thinnest.n.toLocaleString()} rows`,
        detail:
          `Roughly ${Math.round(thinnest.n / ctx.k)} samples per class, about ${Math.round(perWell)} rows per well. ` +
          "Class numbering is ordered by the mean of the FIRST checked input curve, so put GR first if you want " +
          "0 to be the cleanest sand.",
      });
    }
  }

  const rank = { alert: 0, warn: 1, ok: 2 } as const;
  out.sort((a, b) => rank[a.level] - rank[b.level]);
  return [headline, ...out];
}

function renderQc(
  head: HTMLElement,
  host: HTMLElement,
  rows: CurveStatsRow[],
  sampling: [string, CurveSampling[]][],
  ctx: QcContext,
): void {
  host.innerHTML = "";
  const findings = qcFindings(rows, sampling, ctx);
  const alerts = findings.filter((f) => f.level === "alert").length;
  const warns = findings.filter((f) => f.level === "warn").length;
  head.textContent =
    (alerts
      ? `${alerts} thing${alerts === 1 ? "" : "s"} to fix before fitting ${ctx.algorithmLabel}`
      : warns
        ? `Nothing blocking, ${warns} thing${warns === 1 ? "" : "s"} worth knowing`
        : `Nothing found against ${ctx.algorithmLabel}`) +
    ` · ${ctx.curves.length} curve(s), ${ctx.wells} well(s)` +
    (ctx.masked ? ", mask applied" : "");

  for (const f of findings) {
    const card = document.createElement("div");
    card.className = `ml-qc-item ml-qc-${f.level}`;
    const t = document.createElement("div");
    t.className = "ml-qc-title";
    t.textContent = f.title;
    const d = document.createElement("div");
    d.className = "ml-qc-detail";
    d.textContent = f.detail;
    card.append(t, d);
    host.appendChild(card);
  }
}

const SVG_NS = "http://www.w3.org/2000/svg";

function svgEl<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number>,
): SVGElementTagNameMap[K] {
  const el = document.createElementNS(SVG_NS, tag);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, String(v));
  return el;
}

/** A depth window, as the backend's `DepthWindow` takes it. An open side stays null. */
interface IntervalPick {
  top: number | null;
  base: number | null;
}

/**
 * "Which interval" — by marker, or by typed depths, or neither.
 *
 * Jauhar, 2026-08-07: *"it should be tops bounded as well by user"*. A model fitted over a whole
 * well learns one relation for every formation it passed through, and a deltaic sand and the
 * carbonate below it do not share a porosity-permeability transform.
 *
 * The markers come from ONE well — whichever is selected — and are then applied as DEPTHS to every
 * well in the run. That is a real limitation and the control says so, because the alternative would
 * be worse in a way that is hard to see: resolving "Gumai" per well sounds more correct, but a well
 * that lacks the marker would silently fall back to its whole length and join the fit as a
 * different population. A depth window is at least the same window everywhere.
 *
 * Returns `getWindow`, which is what the request carries, and a `dispose` for the well subscription.
 */
function buildIntervalPicker(
  label: string,
): { row: HTMLElement; getWindow: () => IntervalPick; dispose: () => void } {
  let tops: TopEntry[] = [];
  const sel = document.createElement("select");
  sel.className = "form-control ml-interval-sel";
  const topIn = document.createElement("input");
  const baseIn = document.createElement("input");
  for (const i of [topIn, baseIn]) {
    i.type = "number";
    i.step = "any";
    i.className = "form-control";
  }
  const mkNum = (text: string, input: HTMLInputElement) => {
    const l = document.createElement("label");
    l.className = "mc-field ml-interval-num";
    const s = document.createElement("span");
    s.textContent = text;
    l.append(s, input);
    return l;
  };
  const why = document.createElement("div");
  why.className = "ml-norm-why";
  const wrap = document.createElement("div");
  wrap.className = "ml-cov";
  const line = document.createElement("div");
  line.className = "mc-settings";
  line.append(sel, mkNum("Top", topIn), mkNum("Base", baseIn));
  wrap.append(line, why);
  const row = formRow(label, wrap);

  const describe = () => {
    const t = topIn.value.trim();
    const b = baseIn.value.trim();
    why.textContent =
      !t && !b
        ? "The whole logged interval of every well in this run."
        : `${t || "the top of the log"} to ${b || "TD"}, applied as DEPTHS to every well here — the marker list is read from the selected well only, so check it suits the others.`;
  };
  sel.addEventListener("change", () => {
    // A marker fills the boxes and then lets go. The depths stay editable, and a run always sends
    // numbers — so what was actually used is recoverable from the record even after the tops move.
    const i = Number(sel.value);
    if (!Number.isFinite(i) || i < 0) {
      topIn.value = "";
      baseIn.value = "";
    } else {
      topIn.value = String(tops[i].depth);
      // The interval is this marker down to the NEXT one; the deepest marker runs to TD, which is
      // an EMPTY base rather than a guessed number.
      baseIn.value = i + 1 < tops.length ? String(tops[i + 1].depth) : "";
    }
    describe();
  });
  for (const i of [topIn, baseIn]) {
    i.addEventListener("input", () => {
      sel.value = "-1";
      describe();
    });
  }

  const load = async (): Promise<void> => {
    const w = appState.selectedWell.get();
    sel.innerHTML = "";
    const none = document.createElement("option");
    none.value = "-1";
    none.textContent = w ? "(whole well)" : "(no well selected — type depths)";
    sel.appendChild(none);
    tops = w ? await listTops(w.well_id).catch(() => [] as TopEntry[]) : [];
    tops.sort((a, b) => a.depth - b.depth);
    tops.forEach((t, i) => {
      const o = document.createElement("option");
      o.value = String(i);
      o.textContent = `${t.top_name} (${t.depth})`;
      sel.appendChild(o);
    });
    sel.value = "-1";
    describe();
  };
  void load();
  const off = appState.selectedWell.subscribe(() => void load());
  describe();

  return {
    row,
    getWindow: () => ({
      top: topIn.value.trim() === "" ? null : Number(topIn.value),
      base: baseIn.value.trim() === "" ? null : Number(baseIn.value),
    }),
    dispose: off,
  };
}

/** The presentation properties a standalone SVG has to carry itself. */
const SVG_PAINT = [
  "fill", "fill-opacity", "stroke", "stroke-width", "stroke-dasharray", "stroke-opacity",
  "stroke-linecap", "opacity", "font-family", "font-size", "font-weight", "text-anchor",
  "dominant-baseline",
] as const;

/**
 * A copy of an ML chart that survives leaving the application.
 *
 * These charts are drawn as live SVG and painted from the stylesheet — `var(--text)`, `--accent`,
 * the class rules in `styles.css`. Serialize one as it stands and every one of those references
 * dangles: the file opens as black-on-transparent line art, or as nothing. So each element's
 * COMPUTED paint is written onto the element before serializing. The result is a file that looks
 * the same in Illustrator, in a browser, and pasted into a report — which is the only reason to
 * export a vector rather than a picture of one.
 *
 * The theme is baked in, deliberately. An export is a copy of what was on the screen, and a figure
 * that silently re-themed itself in somebody else's document would not be that copy.
 */
function inlineSvgPaint(src: SVGSVGElement): string {
  const clone = src.cloneNode(true) as SVGSVGElement;
  const from = src.querySelectorAll<SVGElement>("*");
  const to = clone.querySelectorAll<SVGElement>("*");
  const apply = (a: Element, b: SVGElement) => {
    const cs = getComputedStyle(a);
    for (const p of SVG_PAINT) {
      const v = cs.getPropertyValue(p);
      if (v && v !== "none" && v !== "normal") b.style.setProperty(p, v);
    }
  };
  apply(src, clone);
  for (let i = 0; i < from.length && i < to.length; i++) apply(from[i], to[i]);
  // An explicit ground: SVG defaults to transparent, which reads as black once the file lands in a
  // document with a dark page or a slide with a photo behind it.
  const bg = getComputedStyle(document.documentElement).getPropertyValue("--bg-panel").trim();
  if (bg) clone.style.setProperty("background", bg);
  clone.setAttribute("xmlns", SVG_NS);
  const box = src.getBoundingClientRect();
  if (box.width > 0 && box.height > 0) {
    clone.setAttribute("width", String(Math.round(box.width)));
    clone.setAttribute("height", String(Math.round(box.height)));
  }
  return new XMLSerializer().serializeToString(clone);
}

/** The same chart rasterized, for the clipboard, a PNG and the printer — all three of which take a
 *  canvas. Drawn from the self-contained SVG above, so the picture carries the chart's real colours
 *  rather than the browser's defaults. */
async function svgToCanvas(src: SVGSVGElement, scale = 2): Promise<HTMLCanvasElement> {
  const box = src.getBoundingClientRect();
  const w = Math.max(1, Math.round(box.width));
  const h = Math.max(1, Math.round(box.height));
  const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(inlineSvgPaint(src))}`;
  const img = new Image();
  await new Promise<void>((res, rej) => {
    img.onload = () => res();
    img.onerror = () => rej(new Error("the chart could not be rasterized"));
    img.src = url;
  });
  const canvas = document.createElement("canvas");
  canvas.width = w * scale;
  canvas.height = h * scale;
  const ctx = canvas.getContext("2d");
  if (ctx) {
    // Opaque, for the same reason the SVG gets a ground: a transparent PNG pasted into a dark slide
    // becomes an unreadable figure, and nobody re-exports it because it looked fine in the preview.
    const bg = getComputedStyle(document.documentElement).getPropertyValue("--bg-panel").trim();
    if (bg) {
      ctx.fillStyle = bg;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
    }
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
  }
  return canvas;
}

/**
 * Copy / PNG / SVG / Print on an ML chart — the same four the canvas plots have carried since
 * `plotExport.ts` (Jauhar, 2026-08-07: *"add option to print to cliboard, svg, etc such other
 * visualization"*).
 *
 * The actions themselves are `plotExport`'s, not re-implemented: a second definition of "save this
 * plot" is a second place for the file naming, the status wording and the Processing-history entry
 * to drift. What is new here is only the bridge from an SVG chart to the canvas those actions take.
 */
function attachChartExport(host: HTMLElement, svg: SVGSVGElement, name: string): HTMLElement {
  let lastRaster: HTMLCanvasElement | null = null;
  const bar = buildImageExportButtons(
    // Rasterization is async and `imageAction` wants a canvas now, so the most recent raster is
    // kept and refreshed on hover — the first click after the chart appears is the only one that
    // could miss, and the buttons are unreachable without passing over them.
    () => lastRaster,
    name,
    setStatus,
    () => inlineSvgPaint(svg),
  );
  const refresh = () => {
    void svgToCanvas(svg).then((c) => (lastRaster = c)).catch(() => undefined);
  };
  refresh();
  bar.addEventListener("pointerenter", refresh);
  bar.classList.add("ml-chart-export");
  host.appendChild(bar);
  return bar;
}

/**
 * The leaderboard, drawn.
 *
 * The table above it already carries every number, so this is not a second copy of the data — it is
 * the one thing the table cannot show: **whether the gap between two models is bigger than the
 * uncertainty in either of them.** A column of scores four decimal places wide invites the eye to
 * read 0.7412 as beating 0.7385, and the `tiedAtTheTop` note says otherwise in a sentence somebody
 * has to stop and parse. Here the whiskers either overlap or they do not.
 *
 * Sorted best-first like the table, and deliberately the same order — a chart that re-sorted would
 * make the reader map one to the other.
 *
 * The tie band is shaded across the whole plot rather than drawn per row, because what it marks is a
 * REGION of the score axis inside which this run cannot distinguish anything, not a property of the
 * rows that happen to fall in it. A fifth model landing there later is equally undecided.
 *
 * Errored rows are omitted rather than drawn at zero: a model that failed to fit has no score, and a
 * dot at the bottom of the axis is a score of zero, which is a different and much stronger claim.
 *
 * Exported so it can be driven with synthetic rows over the vite dev server — the whisker geometry
 * and the band are wrong in ways a screenshot shows and a type check does not.
 */
export function renderScoreChart(
  host: HTMLElement,
  res: MlEvalResult,
  isClf: boolean,
  tied: number,
): void {
  const ok = res.rows.filter((r) => !r.error && r.score != null);
  // One model is not a comparison. Two is the smallest one worth a picture.
  if (ok.length < 2) return;

  const wrap = document.createElement("div");
  wrap.className = "ml-chart";
  const cap = document.createElement("div");
  cap.className = "mc-chain-note";
  cap.textContent = `${isClf ? "Accuracy" : "R²"} by model — dot is the mean across folds, bar is the spread between them`;
  wrap.appendChild(cap);

  const rowH = 22;
  const padL = 156;
  const padR = 62;
  const padT = 8;
  const width = 620;
  const height = padT + ok.length * rowH + 22;

  const lo = Math.min(...ok.map((r) => (r.score as number) - (r.score_std ?? 0)));
  const hi = Math.max(...ok.map((r) => (r.score as number) + (r.score_std ?? 0)));
  // A flat run (every model identical) would divide by zero; a hair of span keeps the dots visible
  // and, correctly, keeps them on top of each other.
  const pad = Math.max((hi - lo) * 0.08, 1e-6);
  const x0 = lo - pad;
  const x1 = hi + pad;
  const X = (v: number) => padL + ((v - x0) / (x1 - x0)) * (width - padL - padR);

  const svg = svgEl("svg", { viewBox: `0 0 ${width} ${height}`, class: "ml-score-chart", width: "100%" });

  // The undecided region, drawn once. It spans from the leader's lower whisker to the top of
  // whatever the tie reaches, which is exactly the interval `tiedAtTheTop` tests against.
  if (tied > 1) {
    const band = ok.slice(0, tied);
    const bLo = Math.min(...band.map((r) => (r.score as number) - (r.score_std ?? 0)));
    const bHi = Math.max(...band.map((r) => (r.score as number) + (r.score_std ?? 0)));
    svg.appendChild(
      svgEl("rect", {
        x: X(bLo),
        y: padT - 2,
        width: Math.max(1, X(bHi) - X(bLo)),
        height: ok.length * rowH + 4,
        class: "ml-sc-tieband",
      }),
    );
  }

  ok.forEach((r, i) => {
    const cy = padT + i * rowH + rowH / 2;
    const score = r.score as number;
    const sd = r.score_std ?? 0;

    const label = svgEl("text", { x: padL - 8, y: cy + 4, class: "ml-sc-label", "text-anchor": "end" });
    // The curve COUNT rather than the curve names: the table beside this carries the names, and a
    // list of six mnemonics at this size is unreadable. How many curves a model needed is the thing
    // being traded against its score, and that is a number.
    label.textContent = `${r.algorithm} · ${r.features.length} curve${r.features.length === 1 ? "" : "s"}`;
    const t = document.createElementNS(SVG_NS, "title");
    t.textContent = r.features.join(", ");
    label.appendChild(t);
    svg.appendChild(label);

    if (sd > 0) {
      svg.appendChild(svgEl("line", { x1: X(score - sd), y1: cy, x2: X(score + sd), y2: cy, class: "ml-sc-whisker" }));
      for (const e of [score - sd, score + sd]) {
        svg.appendChild(svgEl("line", { x1: X(e), y1: cy - 4, x2: X(e), y2: cy + 4, class: "ml-sc-cap" }));
      }
    }
    const dot = svgEl("circle", { cx: X(score), cy, r: 4.5, class: "ml-sc-dot" });
    if (i < tied) dot.setAttribute("class", tied === 1 ? "ml-sc-dot ml-sc-best" : "ml-sc-dot ml-sc-tied");
    svg.appendChild(dot);

    const val = svgEl("text", { x: width - padR + 6, y: cy + 4, class: "ml-sc-value" });
    val.textContent = sd > 0 ? `${score.toFixed(3)} ±${sd.toFixed(3)}` : score.toFixed(3);
    svg.appendChild(val);
  });

  for (const [v, anchor] of [
    [x0, "start"],
    [x1, "end"],
  ] as const) {
    const t = svgEl("text", { x: X(v), y: height - 5, class: "ml-sc-axis", "text-anchor": anchor });
    t.textContent = v.toFixed(2);
    svg.appendChild(t);
  }

  wrap.appendChild(svg);
  attachChartExport(wrap, svg, "ML model scores");
  host.appendChild(wrap);
}

/**
 * Which curve is actually carrying the prediction — asked across every model rather than inside one.
 *
 * The importance panel below answers "what did THIS model lean on". That is a different and weaker
 * question: a curve can top one algorithm's importance and vanish from the next, and when it does,
 * what has been measured is the algorithm rather than the rock. The question an interpreter is
 * really asking before committing a logging programme — *is GR carrying this, or is it just the
 * random forest?* — needs every model in the run to vote.
 *
 * **Importance is normalised WITHIN each model before anything is compared.** Permutation importance
 * is a drop in that model's own score, so a random forest's and an SVR's are not the same quantity
 * and averaging them raw would produce a number with no meaning that still sorts. Each model's
 * importances are scaled to its own maximum, which makes every vote a share of that model's total
 * reliance — comparable because it is dimensionless, not because the units happen to look alike.
 *
 * **The denominator is models that USED the curve, not models in the run.** The leaderboard ranks
 * algorithm × curve subsets, so most curves are absent from most rows. Counting a curve as scoring
 * zero in a model that was never offered it would bury exactly the curve a small subset was built
 * to test.
 *
 * **A model whose importance could not be measured does not vote.** `n_imp_folds == 0` means no fold
 * could be permuted; its zeros are an absence of measurement, and letting them average in would drag
 * every curve toward "does not carry".
 *
 * The spread across models is the point of the whole panel, so it is stated: a curve at 0.9 in one
 * model and 0.1 in another has the same mean as one at 0.5 twice, and only the second is a finding.
 */
export function renderPredictorConsensus(host: HTMLElement, res: MlEvalResult): void {
  const voters = res.rows.filter(
    (r) => !r.error && r.n_imp_folds > 0 && r.importances?.length === r.features.length,
  );
  // Two models cannot disagree usefully, and one cannot disagree at all.
  if (voters.length < 2) return;

  const shares = new Map<string, number[]>();
  for (const r of voters) {
    // Scaled to this model's own maximum: a share of what THIS model leaned on.
    const max = Math.max(...r.importances.map((v) => Math.abs(v)), 0);
    if (!(max > 0)) continue; // a model that leaned on nothing has no opinion to record
    r.features.forEach((f, i) => {
      const s = Math.max(0, r.importances[i] ?? 0) / max;
      const list = shares.get(f) ?? [];
      list.push(s);
      shares.set(f, list);
    });
  }
  if (shares.size === 0) return;

  const stats = [...shares.entries()]
    .map(([curve, vals]) => {
      const n = vals.length;
      const mean = vals.reduce((a, b) => a + b, 0) / n;
      const spread = Math.max(...vals) - Math.min(...vals);
      return { curve, n, mean, spread, lo: Math.min(...vals), hi: Math.max(...vals) };
    })
    .sort((a, b) => b.mean - a.mean);

  const wrap = document.createElement("div");
  wrap.className = "ml-chart ml-consensus";
  const cap = document.createElement("div");
  cap.className = "mc-chain-note";
  cap.textContent = `Which curve carries — across ${voters.length} models, each scaled to its own strongest input`;
  wrap.appendChild(cap);

  for (const s of stats) {
    const line = document.createElement("div");
    line.className = "ml-imp-row";
    const name = document.createElement("span");
    name.className = "ml-imp-name";
    name.textContent = s.curve;
    const barWrap = document.createElement("div");
    barWrap.className = "ml-imp-bar-wrap";
    const bar = document.createElement("div");
    bar.className = "ml-imp-bar";
    bar.style.width = `${s.mean * 100}%`;
    barWrap.appendChild(bar);
    // The range across models, drawn over the mean. Where it is wide the mean is not the finding.
    const range = document.createElement("div");
    range.className = "ml-imp-whisker";
    range.style.left = `${s.lo * 100}%`;
    range.style.width = `${Math.max(0.5, (s.hi - s.lo) * 100)}%`;
    barWrap.appendChild(range);
    const val = document.createElement("span");
    val.className = "ml-imp-val";
    val.textContent = `${s.mean.toFixed(2)} · ${s.n}/${voters.length}`;
    line.append(name, barWrap, val);

    // Two different reasons a row should not be read as a predictor, and they are not the same
    // finding, so they are not the same message.
    if (s.n < voters.length / 2) {
      line.classList.add("ml-imp-unclear");
      line.title =
        `Offered to ${s.n} of ${voters.length} models, so most of this run never tested it. ` +
        `A high share here is a statement about those ${s.n} runs, not about the field.`;
    } else if (s.spread > 0.5) {
      line.classList.add("ml-imp-unclear");
      line.title =
        `Ranges ${s.lo.toFixed(2)} to ${s.hi.toFixed(2)} across the models that used it. ` +
        `A curve that carries in one algorithm and not another is telling you about the algorithm. ` +
        `The mean is not the finding here — the spread is.`;
    } else {
      line.title =
        `${s.lo.toFixed(2)} to ${s.hi.toFixed(2)} across ${s.n} models. ` +
        `Consistent across algorithms, which is what makes it a property of the rock rather than of the fit.`;
    }
    wrap.appendChild(line);
  }

  // The panel below this one shows what the SELECTED model leaned on, and the two will often
  // disagree — which is the most useful thing either of them says and the easiest to mistake for a
  // bug. A reader who sees RHOB on top here and GR on top there concludes one panel is broken,
  // stops trusting both, and goes back to reading the score column alone.
  //
  // So the reconciliation is stated rather than left to be worked out. Both outcomes are worth
  // printing: disagreement means the leader is that algorithm's preference, and agreement is the
  // stronger result — the same curve wins whichever way the problem is fitted.
  const best = res.rows.find((r) => !r.error && r.score != null && r.n_imp_folds > 0);
  const bestTop =
    best && best.importances?.length
      ? best.features[best.importances.indexOf(Math.max(...best.importances))]
      : null;
  const consensusTop = stats[0]?.curve ?? null;
  if (bestTop && consensusTop) {
    const note = document.createElement("div");
    note.className = "mc-chain-note ml-consensus-note";
    note.textContent =
      bestTop === consensusTop
        ? `${consensusTop} leads both the best-scoring model and the run as a whole — the same curve wins ` +
          `whichever way the problem is fitted, which is the strongest form this evidence takes.`
        : `The best-scoring model (${best?.algorithm}) leans hardest on ${bestTop}, but ${consensusTop} is what ` +
          `carries across the run. Not a contradiction — ${bestTop} is that algorithm's preference, and a curve ` +
          `only one algorithm relies on is a fact about the fit rather than about the rock. Both are worth ` +
          `logging; only ${consensusTop} is worth planning around.`;
    wrap.appendChild(note);
  }
  host.appendChild(wrap);
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
