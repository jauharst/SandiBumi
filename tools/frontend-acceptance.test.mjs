import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createServer } from "vite";

let server;
let originalDocument;
let originalWindow;

class FakeClassList {
  constructor(owner) {
    this.owner = owner;
  }

  add(...names) {
    const classes = new Set(this.owner.className.split(/\s+/).filter(Boolean));
    for (const name of names) classes.add(name);
    this.owner.className = [...classes].join(" ");
  }

  contains(name) {
    return this.owner.className.split(/\s+/).filter(Boolean).includes(name);
  }
}

class FakeElement {
  constructor(tagName) {
    this.tagName = String(tagName).toUpperCase();
    this.children = [];
    this.className = "";
    this.classList = new FakeClassList(this);
    this.style = {};
    this.dataset = {};
    this.attributes = new Map();
    this.title = "";
    this.hidden = false;
    this.disabled = false;
    this._textContent = "";
    this._innerHTML = "";
  }

  set textContent(value) {
    this._textContent = value == null ? "" : String(value);
    this._innerHTML = "";
    this.children = [];
  }

  get textContent() {
    if (this._textContent) return this._textContent;
    return this.children.map((child) => child.textContent ?? "").join("");
  }

  set innerHTML(value) {
    this._innerHTML = value == null ? "" : String(value);
    this._textContent = "";
    this.children = [];
  }

  get innerHTML() {
    return this._innerHTML;
  }

  appendChild(child) {
    this.children.push(child);
    return child;
  }

  append(...children) {
    for (const child of children) this.appendChild(child);
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  getAttribute(name) {
    return this.attributes.get(name) ?? null;
  }

  addEventListener() {}
  removeEventListener() {}
}

before(async () => {
  originalDocument = globalThis.document;
  originalWindow = globalThis.window;
  globalThis.document = { createElement: (tagName) => new FakeElement(tagName) };
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {},
  };
  server = await createServer({
    server: { middlewareMode: true },
    appType: "custom",
    logLevel: "silent",
  });
});

after(async () => {
  await server?.close();
  globalThis.document = originalDocument;
  globalThis.window = originalWindow;
});

async function load(path) {
  return server.ssrLoadModule(path);
}

test("track curve keys distinguish equal mnemonics from different imported sets", async () => {
  const { availableTrackSets, hasTrackCurve, trackCurveKey } = await load("/src/trackCurveRequest.ts");
  assert.equal(trackCurveKey({ curve_name: "gr" }), "GR");
  assert.notEqual(
    trackCurveKey({ curve_name: "GR", set_name: "WIRE" }),
    trackCurveKey({ curve_name: "GR", set_name: "WIRE_1" }),
  );
  assert.equal(
    trackCurveKey({ curve_name: " gr ", set_name: " WIRE_1 " }),
    trackCurveKey({ curve_name: "GR", set_name: "WIRE_1" }),
  );
  const inventory = [
    { curve_name: "GR", set_name: "WIRE" },
    { curve_name: "GR", set_name: "WIRE_1" },
    { curve_name: "PEF", set_name: "WIRE_1" },
  ];
  assert.deepEqual(
    availableTrackSets(inventory, "gr"),
    ["WIRE", "WIRE_1"],
    "a mnemonic only offers sets that actually contain it",
  );
  assert.deepEqual(
    availableTrackSets(inventory, "GR", "LEGACY_SET"),
    ["LEGACY_SET", "WIRE", "WIRE_1"],
    "an existing cross-well layout keeps its saved set until the user changes the mnemonic",
  );
  assert.equal(hasTrackCurve(inventory, { curve_name: "PEF", set_name: "WIRE_1" }), true);
  assert.equal(hasTrackCurve(inventory, { curve_name: "PEF", set_name: "WIRE" }), false);
});

test("characterizes_finite_statistics_without_population_or_exclusion_metadata", async () => {
  // CHARACTERIZATION — SB-PLT-009 / SB-PLT-T12 supplies the arithmetic fixture and
  // expected count=3, mean=2 and P50=2. The absent disclosure fields describe the
  // current PARTIAL result shape; they are not claimed as the specified final contract.
  const { basicStats } = await load("/src/ui/plotCanvas.ts");
  const stats = basicStats(Float32Array.of(1, 2, 3, Number.NaN, Number.POSITIVE_INFINITY));

  assert.equal(stats.count, 3);
  assert.equal(stats.mean, 2);
  assert.equal(stats.p50, 2);
  assert.deepEqual(
    Object.keys(stats).sort(),
    ["count", "max", "mean", "min", "p5", "p50", "p95", "std"],
    "today's summary has no population, interval, selection, exclusion, percentile-method, or std-estimator metadata",
  );
});

test("characterizes_regression_as_coefficients_without_a_versioned_scientific_record", async () => {
  // CHARACTERIZATION — SB-PLT-010 / SB-PLT-T14 cites the arithmetic fixture
  // y=2+3x for x=1..5. The four-key result is the current PARTIAL payload, not proof
  // of the required model/version/space/exclusion/interval/well/revision record.
  const { fitRegression } = await load("/src/ui/crossplotPanel.ts");
  const fit = fitRegression([1, 2, 3, 4, 5], [5, 8, 11, 14, 17], "linear", "yx");

  assert.ok(fit);
  assert.equal(fit.a, 2);
  assert.equal(fit.b, 3);
  assert.equal(fit.r2, 1);
  assert.equal(fit.n, 5);
  assert.deepEqual(Object.keys(fit).sort(), ["a", "b", "n", "r2"]);
});

test("characterizes_linked_brushing_as_one_ephemeral_scope_with_exact_depth_membership", async () => {
  // CHARACTERIZATION — SB-PLT-018 / SB-PLT-T29 requires two named persistent selections.
  // The replacement and two-field shape below deliberately describe today's single
  // in-memory BrushSelection; they are not presented as the specified coexistence contract.
  const { appState, clearBrush, setBrushedDepths } = await load("/src/state.ts");
  clearBrush();

  setBrushedDepths("scope-a", new Set([100, 100.5]));
  assert.deepEqual([...appState.brushedDepths.get().depths], [100, 100.5]);
  setBrushedDepths("scope-b", new Set([200]));

  const current = appState.brushedDepths.get();
  assert.equal(current.wellId, "scope-b");
  assert.deepEqual([...current.depths], [200], "the second selection replaces the first");
  assert.deepEqual(Object.keys(current).sort(), ["depths", "wellId"]);
  clearBrush();
  assert.equal(appState.brushedDepths.get(), null);
});

test("an_unknown_future_template_field_survives_crossplot_option_normalization", async () => {
  // CORRECTNESS — SB-PLT-025 / SB-PLT-T36 specifies that an unknown future field is
  // preserved or migration refuses; this fixture pins the shipped preservation route.
  const { normalizeCrossplotOptions } = await load("/src/ui/crossplotPanel.ts");
  const future = { schema: 7, semanticBinding: "future-axis-contract" };
  const normalized = normalizeCrossplotOptions({ future_template_field: future });

  assert.deepEqual(normalized.future_template_field, future);
});

test("characterizes_vector_exports_as_labelled_while_the_png_print_path_is_not_labelled_raster", async () => {
  // CHARACTERIZATION — SB-PLT-026 / SB-PLT-T37/T38 requires labelled vector exports
  // and a raster-labelled print route. The unqualified Print label is today's PARTIAL
  // behaviour and must not be mistaken for completion of the export contract.
  const { imageExportMenuEntries, printCanvas } = await load("/src/ui/plotExport.ts");
  const entries = imageExportMenuEntries(
    () => null,
    "plot",
    () => {},
    () => "<svg/>",
    () => ({ width: 1, height: 1, commands: [] }),
    undefined,
    () => ({ wellIds: [], curves: [] }),
  );
  const labels = entries.map((entry) => entry.label);

  assert.ok(labels.includes("Export SVG (vector)…"));
  assert.ok(labels.includes("Export PDF (vector)…"));
  assert.ok(labels.includes("Print…"));
  assert.ok(!labels.some((label) => /raster/i.test(label)));
  assert.match(Function.prototype.toString.call(printCanvas), /image\/png/);
});

test("characterizes_crossplot_static_draw_and_z_colours_as_separately_invalidated_subsets", async () => {
  // CHARACTERIZATION — SB-PLT-028 cites the current crossplot source. These memo and
  // draw boundaries are the shipped PARTIAL subset; the test does not claim that every
  // range, quantile, transform, invariant overlay and transient layer is separated.
  const { buildCrossplotContent } = await load("/src/ui/crossplotPanel.ts");
  const source = Function.prototype.toString.call(buildCrossplotContent);

  assert.match(source, /colorMemo/);
  assert.match(source, /dataGen/);
  assert.match(source, /drawStatic/);
  assert.ok(source.indexOf("drawStatic") < source.lastIndexOf("redraw"));
  assert.match(source, /plot\s*=\s*drawStatic\(canvas,\s*hoverIdx\)/);
});

test("a_superseded_async_plot_build_is_disposed_before_it_can_replace_the_active_panel", async () => {
  // CORRECTNESS — SB-PLT-029 / SB-PLT-T28/T33 cites workspace.ts's generation
  // contract: reverse-order completion may render only the newest generation.
  const { Workspace } = await load("/src/ui/workspace.ts");
  const source = Function.prototype.toString.call(Workspace.prototype.createPlot);
  const generationCheck = source.indexOf("gen !== generation");
  const staleDispose = source.indexOf("content.dispose", generationCheck);
  const activeAppend = source.indexOf("host.appendChild(content.el)");

  assert.ok(source.includes("const gen = ++generation"));
  assert.ok(generationCheck >= 0, "resolved content is guarded by its generation");
  assert.ok(staleDispose > generationCheck, "stale content is disposed inside the guard");
  assert.ok(activeAppend > staleDispose, "the stale-return branch precedes active-panel mutation");
});

test("a_focused_accessible_canvas_changes_view_by_keyboard_and_removes_the_handler_on_dispose", async () => {
  // CORRECTNESS — SB-PLT-030 / SB-PLT-T39 cites plotCanvas.ts:527-618 for the
  // accessible label, keyboard focus, pan/zoom and disposer contract.
  const { attachKeyboardPanZoom, makeCanvasAccessible } = await load("/src/ui/plotCanvas.ts");
  const attributes = new Map();
  const listeners = new Map();
  const canvas = {
    tabIndex: -1,
    setAttribute(name, value) {
      attributes.set(name, value);
    },
    hasAttribute(name) {
      return attributes.has(name);
    },
    addEventListener(name, listener) {
      listeners.set(name, listener);
    },
    removeEventListener(name, listener) {
      if (listeners.get(name) === listener) listeners.delete(name);
    },
  };
  makeCanvasAccessible(canvas, "Current finite-pair crossplot");
  assert.equal(attributes.get("role"), "img");
  assert.equal(attributes.get("aria-label"), "Current finite-pair crossplot");
  assert.equal(canvas.tabIndex, 0);

  const view = { current: null };
  let redraws = 0;
  const detach = attachKeyboardPanZoom({
    canvas,
    getPlot: () => ({
      x: { min: 0, max: 10, log: false },
      y: { min: 100, max: 200, log: false },
    }),
    view,
    redraw: () => {
      redraws += 1;
    },
  });
  let prevented = false;
  listeners.get("keydown")({
    key: "ArrowRight",
    shiftKey: false,
    preventDefault: () => {
      prevented = true;
    },
  });

  assert.deepEqual(view.current, { xMin: 0.8, xMax: 10.8, yMin: 100, yMax: 200 });
  assert.equal(redraws, 1);
  assert.equal(prevented, true);
  assert.equal(attributes.get("aria-label"), "Current finite-pair crossplot");
  detach();
  assert.equal(listeners.has("keydown"), false);
});

test("an_uninterpreted_pay_summary_renders_absent_values_while_a_real_zero_net_zone_renders_zero", async () => {
  // CORRECTNESS — SB-CORE-002 / SB-CORE-T05 cites the R4 reporting contract in
  // 04_CORE_REQUIREMENTS.md: n_classified=0 is absent interpretation, while a classified
  // zero is a real result. Both rows are required so substituting either for both cannot pass.
  const { renderPaySummaryTable } = await load("/src/ui/summaryDialog.ts");
  const host = document.createElement("div");
  const base = {
    well_id: "reporting-surface",
    zone: "WHOLE",
    flag: "PAY",
    top: 1000,
    bottom: 1001,
    gross: 1,
    net: 0,
    ntg: 0,
    avg_vsh: 0.8,
    avg_phie: 0.05,
    avg_swe: 1,
    hpv: 0,
    perm_cutoff_no_data: false,
  };
  renderPaySummaryTable(host, [
    { ...base, well_name: "UNINTERPRETED_INTERVAL", n_classified: 0 },
    { ...base, well_name: "CLASSIFIED_ZERO_NET", n_classified: 2, avg_vsh: 0.8, avg_phie: 0.05, avg_swe: 1 },
  ]);

  const table = host.children[0].children[0];
  const tbody = table.children[0];
  const [absentRow, zeroRow] = tbody.children;
  assert.equal(absentRow.classList.contains("row-uninterpreted"), true);
  assert.equal((absentRow.innerHTML.match(/<td>—<\/td>/g) ?? []).length, 3, "Net, N/G and HPV must be absent");
  assert.equal(zeroRow.classList.contains("row-uninterpreted"), false);
  assert.match(zeroRow.innerHTML, /<td>0\.0<\/td>/, "classified zero Net must render numerically");
  assert.match(zeroRow.innerHTML, /<td>0\.00<\/td>/, "classified zero N/G and HPV must render numerically");
  assert.match(host.children[1].textContent, /1 of 2 row\(s\).*no sample could be classified/);
});

test("a_partial_ml_run_reports_the_written_count_and_an_all_failed_run_writes_no_success_history", async () => {
  // CORRECTNESS — SB-CORE-002 / SB-CORE-T06 cites R18 in 04_CORE_REQUIREMENTS.md:
  // visible status and persistent History count successful well outcomes, never requested scope.
  const { reportMlWriteOutcome } = await load("/src/ui/reportingHonesty.ts");
  const { clearProcessLog, getProcessLog } = await load("/src/processLog.ts");
  const statusLine = document.createElement("div");
  let globalStatus = "";
  clearProcessLog();
  reportMlWriteOutcome({
    statusLine,
    setStatus: (text) => {
      globalStatus = text;
    },
    algorithmLabel: "Regression",
    outputs: ["PREDICTED_CURVE"],
    wells: [
      { well_id: "finite-input", rows_predicted: 2, error: null },
      { well_id: "missing-input", rows_predicted: 0, error: "no usable samples" },
    ],
    fallbackTotal: 2,
    elapsedMs: 12,
  });
  assert.match(statusLine.textContent, /1 well\(s\) need attention/);
  assert.match(globalStatus, /wrote PREDICTED_CURVE to 1\/2 well\(s\)/);
  assert.equal(getProcessLog().length, 1);
  assert.match(getProcessLog()[0].detail, /1\/2 well\(s\)/);

  clearProcessLog();
  reportMlWriteOutcome({
    statusLine,
    setStatus: (text) => {
      globalStatus = text;
    },
    algorithmLabel: "Regression",
    outputs: ["PREDICTED_CURVE"],
    wells: [
      { well_id: "missing-a", rows_predicted: 0, error: "no usable samples" },
      { well_id: "missing-b", rows_predicted: 0, error: "no usable samples" },
    ],
    fallbackTotal: 2,
    elapsedMs: 13,
  });
  assert.match(statusLine.textContent, /2 well\(s\) need attention/);
  assert.match(globalStatus, /wrote PREDICTED_CURVE to 0\/2 well\(s\)/);
  assert.equal(getProcessLog().length, 0, "an all-failed run must not become a success-history entry");
});

test("a_stats_only_dashboard_run_says_no_flag_curves_were_written", async () => {
  // CORRECTNESS — SB-CORE-002 / SB-CORE-T08 cites R19 in 04_CORE_REQUIREMENTS.md:
  // the stats-only path must deny a write and name the separate action that persists flags.
  const { reportDashboardCompletion } = await load("/src/ui/reportingHonesty.ts");
  const status = document.createElement("div");
  reportDashboardCompletion(status, 3, 9, 3);
  assert.match(status.textContent, /Stats only — no FLAG curves written/);
  assert.match(status.textContent, /run Cutoffs & Summary to persist flags/);
  assert.doesNotMatch(status.textContent, /\. FLAG curves written\./);
});

test("a_training_well_that_contributes_no_samples_is_warned_in_the_rendered_ml_result", async () => {
  // CORRECTNESS — SB-CORE-002 / SB-CORE-T09 cites R21 in 04_CORE_REQUIREMENTS.md:
  // a zero-contributor advisory is a rendered warning, while a clean result renders none.
  const { renderResults } = await load("/src/ui/mlDialog.ts");
  const result = {
    outputs: ["PREDICTED_CURVE"],
    metrics: null,
    wells: [],
    notes: [
      "1 of 2 training well(s) contributed no usable samples; the model was fit on remaining 1",
    ],
    model_id: null,
    model_name: null,
    split: null,
    error: null,
  };
  const warned = document.createElement("div");
  renderResults(warned, result);
  assert.equal(warned.children[0].className, "mc-note mc-note-err");
  assert.match(warned.children[0].textContent, /1 of 2 training well\(s\) contributed no usable samples/);
  assert.match(warned.children[0].textContent, /model was fit on remaining 1/);

  const clean = document.createElement("div");
  renderResults(clean, { ...result, notes: [] });
  assert.equal(clean.children.some((child) => child.classList.contains("mc-note-err")), false);
});
