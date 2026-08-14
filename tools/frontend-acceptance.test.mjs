import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
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

test("a_user_axis_range_wins_and_without_it_the_header_range_wins_in_the_rendered_label_and_export_while_validity_never_becomes_display", async () => {
  // CORRECTNESS — SB-PLT-002 / SB-PLT-T01/T02. The precedence and provenance tiers
  // come from docs/PRD_v2/23_plotting-interactivity.md §4.1 and §6, citing the
  // plotting dossier §§2.2 and 5.3. These four unequal ranges are discriminator
  // fixtures only; they are not petrophysical limits or product defaults.
  const {
    axisRangeExportRecord,
    formatAxisRangeLabel,
    resolveAxisRange,
  } = await load("/src/ui/axisRange.ts");
  const candidates = {
    user: { min: 10, max: 20 },
    headerDisplay: { min: 1, max: 2 },
    auditedFamilyDisplay: { min: 3, max: 4 },
    finiteData: { min: 5, max: 6 },
    validity: { min: 100, max: 200 },
  };

  const user = resolveAxisRange(candidates);
  assert.deepEqual(user, { min: 10, max: 20, tier: "user" });
  assert.equal(formatAxisRangeLabel("X", user), "X range: user · 10 → 20");
  assert.deepEqual(axisRangeExportRecord("x", user), {
    axis: "x",
    min: 10,
    max: 20,
    tier: "user",
  });

  const header = resolveAxisRange({ ...candidates, user: null });
  assert.deepEqual(header, { min: 1, max: 2, tier: "header_display" });
  assert.equal(formatAxisRangeLabel("X", header), "X range: header display · 1 → 2");
  assert.deepEqual(axisRangeExportRecord("x", header), {
    axis: "x",
    min: 1,
    max: 2,
    tier: "header_display",
  });

  assert.equal(
    resolveAxisRange({
      user: null,
      headerDisplay: null,
      auditedFamilyDisplay: null,
      finiteData: null,
      validity: { min: 100, max: 200 },
    }),
    null,
    "a scientific validity range is never promoted to a display range",
  );

  // The functional resolver alone would let a lazy implementation pass while every live
  // quantitative surface kept its private defaults. Pin the five governed adapters as the
  // other side of the contract: each resolves through the shared chain, renders its winning
  // tier, and sends the same range record to export.
  for (const panel of [
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
    "correlationPanel.ts",
    "vegaPanel.ts",
  ]) {
    const source = await readFile(new URL(`../src/ui/${panel}`, import.meta.url), "utf8");
    assert.match(source, /resolveBoundAxisRange/, `${panel} must use the governed precedence chain`);
    assert.match(source, /formatAxisRangeSummary/, `${panel} must show the winning range tier`);
    assert.match(source, /axisRanges: state\.axis_ranges/, `${panel} must export the rendered range custody`);
  }
});

test("display_clipping_counts_hidden_points_without_changing_analysis_while_explicit_validity_changes_and_discloses_n_statistics_and_fit_inputs_on_every_pilot_plot", async () => {
  // CORRECTNESS — SB-PLT-004. docs/PRD_v2/23_plotting-interactivity.md
  // §§2.2 and 4.1 require display clipping to leave the analysis population alone,
  // while explicit validity exclusion changes and reports n, statistics and fits.
  // The unequal 0..4, 1..3 and 2..4 bounds are discriminator fixtures only; they
  // are not petrophysical limits or product defaults.
  const { applyPlotRangePolicy, formatPlotRangePolicySummary } = await load("/src/ui/plotRangePolicy.ts");
  const { basicStats } = await load("/src/ui/plotCanvas.ts");
  const x = Float32Array.of(0, 1, 2, 3, 4);
  const y = Float32Array.of(10, 11, 12, 13, 14);

  const clipped = applyPlotRangePolicy([
    { values: x, display: { min: 1, max: 3 }, validity: { min: 2, max: 4 } },
    { values: y, display: null, validity: null },
  ], false);
  assert.deepEqual(clipped.indices, [0, 1, 2, 3, 4]);
  assert.equal(clipped.analysisCount, 5);
  assert.equal(clipped.displayHidden, 2);
  assert.equal(clipped.validityExcluded, 0);
  assert.equal(basicStats(Float32Array.from(clipped.indices.map((index) => x[index]))).mean, 2);
  assert.equal(
    formatPlotRangePolicySummary(clipped, { statistics: true, fitInputs: clipped.analysisCount }),
    "n=5 · non-finite excluded=0 · log-domain excluded=0 · display hidden=2 · validity excluded=0 · statistics n=5 · fit inputs=5",
  );

  const filtered = applyPlotRangePolicy([
    { values: x, display: { min: 0, max: 4 }, validity: { min: 2, max: 4 } },
    { values: y, display: null, validity: null },
  ], true);
  assert.deepEqual(filtered.indices, [2, 3, 4]);
  assert.equal(filtered.analysisCount, 3);
  assert.equal(filtered.displayHidden, 0);
  assert.equal(filtered.validityExcluded, 2);
  assert.equal(basicStats(Float32Array.from(filtered.indices.map((index) => x[index]))).mean, 3);
  assert.equal(
    formatPlotRangePolicySummary(filtered, { statistics: true, fitInputs: filtered.analysisCount }),
    "n=3 · non-finite excluded=0 · log-domain excluded=0 · display hidden=0 · validity excluded=2 · statistics n=3 · fit inputs=3",
  );

  // Exercise each panel's real population adapter from both sides. A dead import or an
  // unused shared helper cannot satisfy this inventory.
  const { screenCrossplotPopulation } = await load("/src/ui/crossplotPanel.ts");
  const { DEFAULT_HISTOGRAM_OPTIONS, screenHistogramPopulation } = await load("/src/ui/histogramPanel.ts");
  const { screenPickettPopulation } = await load("/src/ui/pickettPanel.ts");
  const { DEFAULT_CORRELATION_OPTIONS, screenCorrelationPopulation } = await load("/src/ui/correlationPanel.ts");
  const { screenVegaPopulation } = await load("/src/ui/vegaPanel.ts");
  assert.equal(typeof screenCrossplotPopulation, "function", "Crossplot must expose its live population adapter");

  const makePanelReports = (validity) => [
    ["Crossplot", screenCrossplotPopulation(
      x, y, validity, { min: 2, max: 4 }, null, { min: validity ? 0 : 1, max: validity ? 4 : 3 }, null,
    )],
    ["Histogram", screenHistogramPopulation(
      x,
      { ...DEFAULT_HISTOGRAM_OPTIONS, validityFilter: validity, validMin: 2, validMax: 4 },
      { min: validity ? 0 : 1, max: validity ? 4 : 3 },
    )],
    ["Pickett", screenPickettPopulation(
      Float32Array.of(1, 2, 3, 4, 5),
      y,
      { validityFilter: validity, rtValidMin: 3, rtValidMax: 5 },
      { min: validity ? 1 : 2, max: validity ? 5 : 4 },
      null,
    )],
    ["Correlation", screenCorrelationPopulation(
      x,
      { ...DEFAULT_CORRELATION_OPTIONS, validityFilter: validity, validMin: 2, validMax: 4 },
      { min: validity ? 0 : 1, max: validity ? 4 : 3 },
    )],
    ["Vega", screenVegaPopulation(
      Array.from(x, (value, index) => ({ depth: index, x: value, y: y[index] })),
      "scatter",
      { apply: validity, x: { min: 2, max: 4 }, y: null },
      { min: validity ? 0 : 1, max: validity ? 4 : 3 },
      null,
    )],
  ];
  for (const [panel, report] of makePanelReports(false)) {
    assert.equal(report.analysisCount, 5, `${panel}: display clipping must preserve n`);
    assert.equal(report.displayHidden, 2, `${panel}: display clipping must count hidden samples`);
    assert.equal(report.validityExcluded, 0, `${panel}: disabled validity must exclude nothing`);
  }
  for (const [panel, report] of makePanelReports(true)) {
    assert.equal(report.analysisCount, 3, `${panel}: explicit validity must change n`);
    assert.equal(report.displayHidden, 0, `${panel}: full display must hide nothing`);
    assert.equal(report.validityExcluded, 2, `${panel}: explicit validity must count exclusions`);
  }

  // Build-time adapter registry only: correctness comes from the executed reports above.
  // This inventory ensures a live pilot panel cannot quietly drop the shared disclosure path.
  for (const [panel, adapter] of [
    ["crossplotPanel.ts", "screenCrossplotPopulation"],
    ["histogramPanel.ts", "screenHistogramPopulation"],
    ["pickettPanel.ts", "screenPickettPopulation"],
    ["correlationPanel.ts", "screenCorrelationPopulation"],
    ["vegaPanel.ts", "screenVegaPopulation"],
  ]) {
    const source = await readFile(new URL(`../src/ui/${panel}`, import.meta.url), "utf8");
    assert.match(source, new RegExp(`${adapter}\\(`), `${panel} must call its executed policy adapter`);
    assert.match(source, /formatPlotRangePolicySummary\(/, `${panel} must disclose the shared policy summary`);
  }
});

test("every_shipped_unit_limit_row_is_source_owned_and_dimensionally_screened_while_the_documented_6_56x_pair_and_unknown_units_stay_disabled_with_reasons", async () => {
  // CORRECTNESS — SB-PLT-005 / SB-PLT-T05. docs/PRD_v2/23_plotting-interactivity.md
  // §§2.2, 4.1, 6 and 7.1 O-2 cite dossier §3.3a: 1 international foot is 0.3048 m,
  // the accepted screen is 15%, RHOB 1.95..2.95 g/cc ↔ 1950..2950 kg/m3 is exact,
  // DTC 240..40 us/ft ↔ 780..120 us/m is deliberately rounded but inside that screen,
  // and attenuation 0..100 dB/ft ↔ 0..50 dB/m is 6.56× divergent. These are cited
  // incumbent display rows under audit, not physical-family bounds or new defaults.
  const {
    UNIT_LIMIT_ROWS,
    auditUnitLimitRow,
    auditedFamilyDisplayDecision,
    axisRangeExportRecord,
    formatAxisRangeLabel,
    resolveBoundAxisRange,
  } = await load("/src/ui/axisRange.ts");

  assert.deepEqual(
    UNIT_LIMIT_ROWS.map((row) => row.id),
    [
      "GR:gAPI",
      "RHOB:g/cc",
      "RHOB:kg/m3",
      "NPHI:v/v",
      "PEF:b/e",
      "PHIE:v/v",
      "SW:v/v",
      "DT:us/ft",
      "DT:us/m",
      "ACOUSTIC_ATTENUATION_RATE:dB/m",
    ],
    "the activation gate must enumerate the complete screened seed set plus the cited refusal",
  );
  for (const row of UNIT_LIMIT_ROWS) {
    assert.ok(row.source.length > 0, `${row.id} must carry its numeric source`);
    const audit = auditUnitLimitRow(row);
    if (row.familyDefault) {
      assert.equal(audit.enabled, true, `${row.id} must pass before the family-default tier can use it`);
    }
  }

  const density = auditedFamilyDisplayDecision({ mnemonic: "RHOB", display_unit: "kg/m3" });
  assert.equal(density.enabled, true);
  assert.deepEqual(density.range, { min: 1950, max: 2950 });
  assert.match(density.reason, /exact registered conversion/u);

  const slowness = auditedFamilyDisplayDecision({ mnemonic: "DTC", display_unit: "us/m" });
  assert.equal(slowness.enabled, true);
  assert.deepEqual(slowness.range, { min: 780, max: 120 });
  assert.match(slowness.reason, /within the cited 15% screen/u);

  const attenuationRow = UNIT_LIMIT_ROWS.find((row) => row.id === "ACOUSTIC_ATTENUATION_RATE:dB/m");
  const attenuation = auditUnitLimitRow(attenuationRow);
  assert.equal(attenuation.enabled, false);
  assert.equal(Math.round(attenuation.divergenceFactor * 100) / 100, 6.56);
  assert.match(attenuation.reason, /disabled.*exceeds the cited 15% screen/u);

  const unknown = auditedFamilyDisplayDecision({ mnemonic: "RHOB", display_unit: "g/c3" });
  assert.equal(unknown.enabled, false);
  assert.equal(unknown.range, null);
  assert.match(unknown.reason, /disabled.*no audited unit-limit row/u);

  const fallback = resolveBoundAxisRange({
    binding: { resolved: [{ mnemonic: "RHOB", display_unit: "g/c3" }] },
    user: null,
    finiteData: { min: 2.1, max: 2.8 },
    validity: null,
  });
  assert.equal(fallback.tier, "finite_data");
  assert.equal(fallback.familyLimitAudit.enabled, false);
  assert.match(formatAxisRangeLabel("X", fallback), /family limit disabled/u);
  assert.deepEqual(axisRangeExportRecord("x", fallback).familyLimitAudit, fallback.familyLimitAudit);

  for (const panel of [
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
    "correlationPanel.ts",
    "vegaPanel.ts",
  ]) {
    const source = await readFile(new URL(`../src/ui/${panel}`, import.meta.url), "utf8");
    assert.match(source, /resolveBoundAxisRange/, `${panel} must execute the audited activation gate`);
    assert.match(source, /formatAxisRangeSummary/, `${panel} must disclose a disabled row's reason`);
    assert.match(source, /axisRanges: state\.axis_ranges/, `${panel} must export the audit custody`);
  }
});

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
