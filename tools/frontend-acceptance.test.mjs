import assert from "node:assert/strict";
import { createHash } from "node:crypto";
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
    this.listeners = new Map();
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

  addEventListener(name, listener) {
    const listeners = this.listeners.get(name) ?? new Set();
    listeners.add(listener);
    this.listeners.set(name, listeners);
  }

  removeEventListener(name, listener) {
    this.listeners.get(name)?.delete(listener);
  }

  dispatchEvent(event) {
    for (const listener of this.listeners.get(event.type) ?? []) listener(event);
    return true;
  }

  click() {
    this.dispatchEvent({ type: "click" });
  }

  getContext(kind) {
    if (this.tagName !== "CANVAS" || kind !== "2d") return null;
    return {
      font: "10px sans-serif",
      textAlign: "start",
      textBaseline: "alphabetic",
      measureText(text) {
        // T37/T38 discriminator metric: deliberately wider than the old
        // 0.6-em character-count estimate, so a self-certified crop proof
        // cannot pass without consuming real TextMetrics-style bounds.
        const width = String(text).length * 9;
        const left = this.textAlign === "center" ? width / 2 : this.textAlign === "right" || this.textAlign === "end" ? width : 0;
        return {
          width,
          actualBoundingBoxLeft: left,
          actualBoundingBoxRight: width - left,
          actualBoundingBoxAscent: 8,
          actualBoundingBoxDescent: 2,
        };
      },
    };
  }
}

before(async () => {
  originalDocument = globalThis.document;
  originalWindow = globalThis.window;
  globalThis.document = { createElement: (tagName) => new FakeElement(tagName) };
  const windowListeners = new Map();
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {},
    addEventListener(name, listener) {
      const listeners = windowListeners.get(name) ?? new Set();
      listeners.add(listener);
      windowListeners.set(name, listeners);
    },
    removeEventListener(name, listener) {
      windowListeners.get(name)?.delete(listener);
    },
    dispatchEvent(event) {
      for (const listener of windowListeners.get(event.type) ?? []) listener(event);
      return true;
    },
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

test("non_finite_log_xy_z_and_waveform_values_follow_one_non_mutating_reported_channel_policy", async () => {
  // CORRECTNESS — SB-PLT-013. docs/PRD_v2/23_plotting-interactivity.md §4.3
  // cites dossier §§2.7 and 5.3: non-finite values are excluded; non-positive values
  // are excluded on log channels; X/Y overflow is clipped and counted; Z overflow is
  // clamped, edge-marked and counted; array-waveform overflow is clamped and counted;
  // source samples never change. The 0..10 range is arithmetic test input, not a
  // petrophysical limit or product default.
  const { applyPlotChannelPolicy } = await load("/src/ui/plotTypes.ts");
  const { applyPlotRangePolicy } = await load("/src/ui/plotRangePolicy.ts");
  const { buildPickettColourPolicy } = await load("/src/ui/pickettPanel.ts");
  const { applyVegaColourPolicy } = await load("/src/ui/vegaPanel.ts");
  const { applyArrayWaveformPolicy } = await load("/src/ui/logViewPanel.ts");
  const source = Float32Array.of(-1, 5, 20, Number.NaN);
  const sourceBits = Array.from(source, (value) => new Uint32Array(new Float32Array([value]).buffer)[0]);
  const display = { min: 0, max: 10 };

  const xy = applyPlotChannelPolicy(source, "cartesian", display, true);
  assert.equal(xy.nonFiniteExcluded, 1);
  assert.equal(xy.logDomainExcluded, 1);
  assert.equal(xy.displayClipped, 1);
  assert.equal(xy.clamped, 0);
  assert.equal(xy.values[2], 20, "Cartesian clipping must not rewrite the source value");

  const aligned = applyPlotRangePolicy([
    { values: source, display, validity: null, log: true },
  ], false);
  assert.deepEqual(aligned.indices, [1, 2]);
  assert.equal(aligned.nonFiniteExcluded, 1);
  assert.equal(aligned.logDomainExcluded, 1);
  assert.equal(aligned.displayHidden, 1);

  const colour = applyPlotChannelPolicy(source, "colour", display, false);
  assert.equal(colour.nonFiniteExcluded, 1);
  assert.equal(colour.logDomainExcluded, 0);
  assert.equal(colour.displayClipped, 0);
  assert.equal(colour.clamped, 2);
  assert.deepEqual(colour.edgeMarks, ["low", "none", "high", "none"]);
  assert.deepEqual(Array.from(colour.values.slice(0, 3)), [0, 5, 10]);

  const pickett = buildPickettColourPolicy(source, display, false, "viridis");
  assert.equal(pickett.excluded, 1);
  assert.equal(pickett.clamped, 2);
  assert.deepEqual(pickett.edgeMarks, ["low", "none", "high", "none"]);
  assert.equal(pickett.colors[3], "rgba(0,0,0,0)");
  const pickettLog = buildPickettColourPolicy(source, { min: 1, max: 10 }, true, "viridis");
  assert.equal(pickettLog.nonFiniteExcluded, 1);
  assert.equal(pickettLog.logDomainExcluded, 1);
  assert.equal(pickettLog.clamped, 1);
  assert.deepEqual(pickettLog.edgeMarks, ["none", "none", "high", "none"]);

  const vega = applyVegaColourPolicy(
    Array.from(source, (z, depth) => ({ x: depth + 1, y: depth + 2, z, depth })),
    display,
  );
  assert.equal(vega.excluded, 1);
  assert.equal(vega.clamped, 2);
  assert.deepEqual(vega.rows.map((row) => row.zDisplay), [0, 5, 10, undefined]);
  assert.deepEqual(vega.rows.map((row) => row.zEdge), ["low", "none", "high", "none"]);

  const waveform = applyArrayWaveformPolicy(source, display, false);
  assert.equal(waveform.nonFiniteExcluded, 1);
  assert.equal(waveform.clamped, 2);
  assert.deepEqual(Array.from(waveform.values.slice(0, 3)), [0, 5, 10]);
  const waveformLog = applyArrayWaveformPolicy(source, { min: 1, max: 10 }, true);
  assert.equal(waveformLog.nonFiniteExcluded, 1);
  assert.equal(waveformLog.logDomainExcluded, 1);
  assert.equal(waveformLog.clamped, 1);
  assert.deepEqual(waveformLog.edgeMarks, ["none", "none", "high", "none"]);
  assert.deepEqual(
    Array.from(source, (value) => new Uint32Array(new Float32Array([value]).buffer)[0]),
    sourceBits,
    "no display policy may mutate the source curve",
  );

  // The pure arithmetic alone would let dead helpers pass. Pin both sides of the live inventory:
  // every ordinary population adapter delegates to the same channel policy; every Z consumer
  // uses the colour policy and edge marks; the screen and composite waveform paths use the same
  // clamp/count policy and disclose it.
  const sources = Object.fromEntries(await Promise.all([
    "plotRangePolicy.ts",
    "crossplotPanel.ts",
    "pickettPanel.ts",
    "vegaPanel.ts",
    "logViewPanel.ts",
  ].map(async (file) => [file, await readFile(new URL(`../src/ui/${file}`, import.meta.url), "utf8")])));
  assert.match(sources["plotRangePolicy.ts"], /applyPlotChannelPolicy\(/);
  assert.match(sources["crossplotPanel.ts"], /applyPlotChannelPolicy\([\s\S]*?"colour"/);
  assert.match(sources["crossplotPanel.ts"], /drawDiamonds\(/);
  assert.match(sources["pickettPanel.ts"], /buildPickettColourPolicy\(/);
  assert.match(sources["pickettPanel.ts"], /drawDiamonds\(/);
  assert.match(sources["vegaPanel.ts"], /applyVegaColourPolicy\(/);
  assert.match(sources["vegaPanel.ts"], /zDisplay/);
  assert.match(sources["vegaPanel.ts"], /zEdge/);
  assert.match(sources["logViewPanel.ts"], /applyArrayWaveformPolicy\(/);
  assert.match(sources["logViewPanel.ts"], /waveform clamped=/);
  const composite = await readFile(new URL("../src-tauri/src/composite.rs", import.meta.url), "utf8");
  assert.match(composite, /apply_plot_channel_policy\(/);
  assert.match(composite, /waveform clamped=/);
});

test("context_plot_decimation_uses_one_shared_index_retains_both_endpoints_and_exports_counts_algorithm_stride_and_forced_endpoint_without_calling_the_view_complete", async () => {
  // CORRECTNESS — SB-PLT-015 / SB-PLT-T21/T22. docs/PRD_v2/23_plotting-interactivity.md
  // §§4.3 and 6 cite dossier §§2.9, 4.2 and 5.3–5.5: eligible indices 0..10 at
  // stride 4 must yield 0,4,8,10 with the final endpoint forced, and depth/X/Y/Z
  // must use that same source index. These sequences are independent arithmetic
  // fixtures, not petrophysical values or product defaults.
  const { decimateSharedChannels } = await load("/src/ui/plotTypes.ts");
  const { contextReductionExport, describeContextOutcome } = await load("/src/ui/plotCommon.ts");
  const sourceIndices = Array.from({ length: 11 }, (_value, index) => index);
  const depth = Float32Array.from(sourceIndices);
  const x = Float32Array.from(sourceIndices, (index) => 100 + index);
  const y = Float32Array.from(sourceIndices, (index) => 200 + index);
  const z = Float32Array.from(sourceIndices, (index) => 300 + index);
  const reduced = decimateSharedChannels([depth, x, y, z], sourceIndices, 4);

  assert.deepEqual(reduced.manifest.sourceIndices, [0, 4, 8, 10]);
  assert.equal(reduced.manifest.originalCount, 11);
  assert.equal(reduced.manifest.displayedCount, 4);
  assert.equal(reduced.manifest.algorithm, "stride_from_first_with_forced_final_endpoint");
  assert.equal(reduced.manifest.stride, 4);
  assert.equal(reduced.manifest.endpointsForced, true);
  assert.deepEqual(reduced.channels.map((channel) => Array.from(channel)), [
    [0, 4, 8, 10],
    [100, 104, 108, 110],
    [200, 204, 208, 210],
    [300, 304, 308, 310],
  ]);
  for (let mark = 0; mark < reduced.manifest.displayedCount; mark++) {
    const index = reduced.manifest.sourceIndices[mark];
    assert.equal(reduced.channels[0][mark], depth[index]);
    assert.equal(reduced.channels[1][mark], x[index]);
    assert.equal(reduced.channels[2][mark], y[index]);
    assert.equal(reduced.channels[3][mark], z[index]);
  }

  const outcome = {
    layers: [{
      wellId: "represented-well",
      name: "represented well",
      color: "#000000",
      depth: reduced.channels[0],
      series: new Map([
        ["X", reduced.channels[1]],
        ["Y", reduced.channels[2]],
        ["Z", reduced.channels[3]],
      ]),
      reduction: reduced.manifest,
      depthStep: {
        coarsestStep: 1,
        decimationFactors: [1, 1, 1],
        mode: "unchanged",
        intervalClosure: "[lo,hi)",
      },
    }],
    shown: 4,
    decimated: true,
    skipped: 0,
    absent: [],
    refusal: null,
  };
  const disclosure = describeContextOutcome(outcome);
  assert.match(disclosure, /reduced 11→4; stride 4; final endpoint forced in 1 well/u);
  assert.doesNotMatch(disclosure, /complete/iu);

  const exported = contextReductionExport("crossplot", outcome, 1);
  assert.ok(exported);
  assert.deepEqual(exported.items[0], {
    subject_kind: "points",
    subject_id: "represented-well",
    original_count: 11,
    displayed_count: 4,
    algorithm: "stride_from_first_with_forced_final_endpoint",
    stride: 4,
    endpoints_forced: true,
  });
  const serialized = JSON.stringify(exported);
  assert.match(serialized, /"stride":4/u);
  assert.match(serialized, /"endpoints_forced":true/u);

  // The pure helper would let a dead implementation pass. Pin one live panel plus the
  // whitelisted export route, then inventory the other two shared context consumers.
  const sources = Object.fromEntries(await Promise.all([
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
  ].map(async (file) => [file, await readFile(new URL(`../src/ui/${file}`, import.meta.url), "utf8")])));
  for (const source of Object.values(sources)) {
    assert.match(source, /contextReductionExport\(/u);
    assert.match(source, /\(\) => ctxReductionManifest/u);
  }
  const exportSource = await readFile(new URL("../src/ui/plotExport.ts", import.meta.url), "utf8");
  assert.match(exportSource, /savePlotReductionManifest\(dest, JSON\.stringify\(manifest\)\)/u);
});

test("every_registered_plot_record_limit_reports_original_and_displayed_counts_or_refuses_instead_of_returning_a_prefix", async () => {
  // CORRECTNESS — SB-PLT-031 / SB-PLT-T40. docs/PRD_v2/23_plotting-interactivity.md
  // §4.6 requires every load/point/well/facet/legend/visual record limit to report before/after
  // counts and export its manifest, or refuse a hard maximum. §5 cites the current 60,000-point
  // budget and 8-load concurrency. Other maxima are retained as-built inputs from 3a4723b6; this
  // test proves behavior immediately below/above the configured value without claiming that the
  // retained number is an independently validated product default.
  const {
    PLOT_RECORD_LIMITS,
    applyPlotRecordLimit,
    plotRecordLimit,
    reducePlotLabel,
  } = await load("/src/ui/plotLimits.ts");
  const { contextReductionExport } = await load("/src/ui/plotCommon.ts");

  assert.deepEqual(PLOT_RECORD_LIMITS.map((limit) => limit.id), [
    "context_fetch_concurrency",
    "context_point_budget",
    "well_scope_name_preview_rows",
    "context_well_legend_rows",
    "context_well_name_characters",
    "fit_scatter_legend_rows",
    "vega_categorical_groups",
  ]);
  assert.deepEqual(
    [...new Set(PLOT_RECORD_LIMITS.map((limit) => limit.subject_kind))].sort(),
    ["facets", "legend", "load", "points", "visual", "wells"],
  );
  assert.equal(plotRecordLimit("context_point_budget").maximum, 60_000);
  assert.equal(plotRecordLimit("context_fetch_concurrency").maximum, 8);

  const legendLimit = plotRecordLimit("context_well_legend_rows");
  const exactLegend = applyPlotRecordLimit(
    "context_well_legend_rows",
    Array.from({ length: legendLimit.maximum }, (_value, index) => index),
    "well_legend",
  );
  assert.equal(exactLegend.item, null, "an exact-boundary legend is not falsely reported as reduced");
  assert.equal(exactLegend.refusal, null);
  assert.equal(exactLegend.displayed.length, legendLimit.maximum);
  const reducedLegend = applyPlotRecordLimit(
    "context_well_legend_rows",
    Array.from({ length: legendLimit.maximum + 2 }, (_value, index) => index),
    "well_legend",
  );
  assert.equal(reducedLegend.displayed.length, legendLimit.maximum);
  assert.deepEqual(reducedLegend.item, {
    subject_kind: "legend",
    subject_id: "well_legend",
    original_count: legendLimit.maximum + 2,
    displayed_count: legendLimit.maximum,
    algorithm: "first_context_well_rows_with_reported_remainder",
    stride: null,
    endpoints_forced: null,
  });
  assert.equal(reducedLegend.refusal, null);

  const labelLimit = plotRecordLimit("context_well_name_characters");
  const exactLabel = "x".repeat(labelLimit.maximum);
  assert.deepEqual(reducePlotLabel("context_well_name_characters", exactLabel, "well-a"), {
    displayed: exactLabel,
    item: null,
  });
  const longLabel = "x".repeat(labelLimit.maximum + 3);
  const reducedLabel = reducePlotLabel("context_well_name_characters", longLabel, "well-a");
  assert.equal(Array.from(reducedLabel.displayed).length, labelLimit.maximum);
  assert.ok(reducedLabel.displayed.endsWith("…"));
  assert.deepEqual(reducedLabel.item, {
    subject_kind: "visual",
    subject_id: "context_well_name:well-a",
    original_count: labelLimit.maximum + 3,
    displayed_count: labelLimit.maximum,
    algorithm: "leading_characters_with_ellipsis_and_reported_remainder",
    stride: null,
    endpoints_forced: null,
  });

  const hardMaximum = plotRecordLimit("vega_categorical_groups");
  const refused = applyPlotRecordLimit(
    "vega_categorical_groups",
    Array.from({ length: hardMaximum.maximum + 1 }, (_value, index) => index),
    "categorical_groups",
  );
  assert.deepEqual(refused.displayed, [], "a hard maximum never returns a plausible prefix");
  assert.equal(refused.item.original_count, hardMaximum.maximum + 1);
  assert.equal(refused.item.displayed_count, 0);
  assert.match(refused.refusal, new RegExp(`exceeds hard maximum ${hardMaximum.maximum}`));
  const accepted = applyPlotRecordLimit(
    "vega_categorical_groups",
    Array.from({ length: hardMaximum.maximum }, (_value, index) => index),
    "categorical_groups",
  );
  assert.equal(accepted.refusal, null);
  assert.equal(accepted.item, null);
  assert.equal(accepted.displayed.length, hardMaximum.maximum);

  const pointLimit = plotRecordLimit("context_point_budget");
  const previewLimit = plotRecordLimit("well_scope_name_preview_rows");
  const layers = Array.from(
    { length: legendLimit.maximum + 1 },
    (_value, index) => ({
      wellId: `well-${index}`,
      name: index === 0 ? longLabel : `well-${index}`,
      color: "#000000",
      depth: Float32Array.from([0]),
      series: new Map(),
      reduction: {
        originalCount: index === 0 ? pointLimit.maximum + 1 : 1,
        displayedCount: index === 0 ? pointLimit.maximum : 1,
        algorithm: "stride_from_first_with_forced_final_endpoint",
        stride: index === 0 ? 2 : 1,
        endpointsForced: index === 0,
        sourceIndices: [0],
      },
      depthStep: { coarsestStep: 1, decimationFactors: [1], mode: "unchanged", intervalClosure: "[lo,hi)" },
    }),
  );
  const manifest = contextReductionExport(
    "crossplot",
    { layers, shown: pointLimit.maximum + legendLimit.maximum, decimated: true, skipped: 0, absent: [], depthReframeHandoffs: [], refusal: null },
    previewLimit.maximum + 1,
    { wellId: "active-well", name: longLabel },
  );
  assert.ok(manifest);
  assert.deepEqual(
    [...new Set(manifest.items.map((item) => item.subject_kind))].sort(),
    ["legend", "points", "visual", "wells"],
  );
  assert.ok(manifest.items.every((item) => item.original_count >= item.displayed_count));
  assert.ok(manifest.items.some((item) => item.subject_id === "context_well_name:well-0"));
  assert.ok(manifest.items.some((item) => item.subject_id === "context_well_name:active-well"));

  const consumers = Object.fromEntries(await Promise.all([
    "plotCommon.ts",
    "wellScope.ts",
    "fitScatter.ts",
    "vegaPanel.ts",
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
  ].map(async (file) => [file, await readFile(new URL(`../src/ui/${file}`, import.meta.url), "utf8")])));
  for (const limit of PLOT_RECORD_LIMITS) {
    for (const file of limit.consumers) {
      assert.match(consumers[file], new RegExp(`\\"${limit.id}\\"`), `${file} must resolve ${limit.id} through the registry`);
    }
  }
  for (const [file, source] of Object.entries(consumers)) {
    assert.doesNotMatch(source, /\.slice\(0,/u, `${file} may not hide a prefix outside the plot-limit registry`);
  }
  assert.doesNotMatch(consumers["crossplotPanel.ts"], /MAX_CONTEXT_POINTS/u);
  assert.doesNotMatch(consumers["histogramPanel.ts"], /MAX_CONTEXT_POINTS/u);
  assert.doesNotMatch(consumers["pickettPanel.ts"], /MAX_CONTEXT_POINTS/u);
  assert.doesNotMatch(consumers["vegaPanel.ts"], /MAX_GROUPS/u);
  assert.match(consumers["vegaPanel.ts"], /\(\) => reductionManifest/u);
  const exportSource = await readFile(new URL("../src/ui/plotExport.ts", import.meta.url), "utf8");
  assert.match(exportSource, /savePlotReductionManifest\(dest, JSON\.stringify\(manifest\)\)/u);
});

test("equal_and_exact_multiple_regular_depth_grids_proceed_with_reported_factors_while_non_integer_or_irregular_grids_refuse_with_an_explicit_reframe_action_and_intervals_stay_half_open", async () => {
  // CORRECTNESS — SB-PLT-016 / SB-PLT-T23–T26. docs/PRD_v2/23_plotting-interactivity.md
  // §§4.3, 6 and 7.3 R-8 cite the plotting dossier §§2.12, 5.1 and 5.3: equal steps keep factor 1,
  // 0.5 and 1.0 decimate to the coarsest grid with factor 2, 0.5 and 0.8 refuse,
  // and [100,101) retains 100 and 100.5 but excludes 101. docs/record_data_tools.md
  // makes Reframe the only user-owned operation allowed to create a new depth frame.
  const { halfOpenDepthIndices, reconcileDepthChannels } = await load("/src/ui/plotTypes.ts");

  const equal = reconcileDepthChannels([
    { depth: Float32Array.from([0, 0.5, 1]), values: Float32Array.from([10, 11, 12]) },
    { depth: Float32Array.from([0, 0.5, 1]), values: Float32Array.from([20, 21, 22]) },
  ]);
  assert.deepEqual(Array.from(equal.depth), [0, 0.5, 1]);
  assert.deepEqual(equal.channels.map((channel) => Array.from(channel)), [
    [10, 11, 12],
    [20, 21, 22],
  ]);
  assert.deepEqual(equal.decimationFactors, [1, 1]);
  assert.equal(equal.mode, "unchanged");

  const exactMultiple = reconcileDepthChannels([
    {
      depth: Float32Array.from([0, 0.5, 1, 1.5, 2]),
      values: Float32Array.from([10, 11, 12, 13, 14]),
    },
    { depth: Float32Array.from([0, 1, 2]), values: Float32Array.from([20, 21, 22]) },
  ]);
  assert.deepEqual(Array.from(exactMultiple.depth), [0, 1, 2]);
  assert.deepEqual(exactMultiple.channels.map((channel) => Array.from(channel)), [
    [10, 12, 14],
    [20, 21, 22],
  ]);
  assert.deepEqual(exactMultiple.decimationFactors, [2, 1]);
  assert.equal(exactMultiple.mode, "decimated_to_coarsest");

  let irregularError;
  try {
    reconcileDepthChannels([
      { depth: Float32Array.from([0, 0.5, 1.25]), values: Float32Array.from([10, 11, 12]) },
      { depth: Float32Array.from([0, 0.5, 1.25]), values: Float32Array.from([20, 21, 22]) },
    ]);
  } catch (error) {
    irregularError = error;
  }
  assert.ok(irregularError, "an identical but irregular grid must not bypass regularity validation");
  assert.equal(irregularError.name, "DepthGridReconciliationError");
  assert.equal(irregularError.route, "reframe");
  assert.equal(irregularError.actionLabel, "Open Reframe");
  assert.equal(irregularError.automaticResampling, false);

  let nonIntegerError;
  try {
    reconcileDepthChannels([
      { depth: Float32Array.from([0, 0.5, 1]), values: Float32Array.from([10, 11, 12]) },
      { depth: Float32Array.from([0, 0.8, 1.6]), values: Float32Array.from([20, 21, 22]) },
    ]);
  } catch (error) {
    nonIntegerError = error;
  }
  assert.ok(nonIntegerError, "non-integer step ratios must refuse instead of being aligned or resampled");
  assert.equal(nonIntegerError.name, "DepthGridReconciliationError");
  assert.equal(nonIntegerError.route, "reframe");
  assert.equal(nonIntegerError.actionLabel, "Open Reframe");
  assert.equal(nonIntegerError.automaticResampling, false);

  assert.deepEqual(
    halfOpenDepthIndices(Float32Array.from([100, 100.5, 101]), 100, 101),
    [0, 1],
  );

  const plotCommon = await load("/src/ui/plotCommon.ts");
  const handoff = plotCommon.depthReframeHandoff?.(
    nonIntegerError,
    ["active-well"],
    ["NPHI", "RHOB"],
  );
  assert.deepEqual(handoff, {
    event: "sandibumi:open-reframe",
    actionLabel: "Open Reframe",
    reason: nonIntegerError.message,
    wellIds: ["active-well"],
    curves: ["NPHI", "RHOB"],
    automaticResampling: false,
  });
  assert.equal(
    plotCommon.depthReframeHandoff?.(new Error("ordinary fetch failure"), ["active-well"], ["NPHI"]),
    null,
    "an unrelated error must not masquerade as a depth-grid handoff",
  );

  const statuses = [];
  const control = plotCommon.buildDepthReframeHandoff?.((text) => statuses.push(text));
  assert.ok(control, "the plot route must expose a real Reframe handoff control");
  assert.equal(control.el.style.display, "none");
  control.show(handoff);
  assert.equal(control.el.style.display, "");
  assert.match(control.el.textContent, /Plot refused: .* No data were resampled\./u);
  const button = control.el.children.at(-1);
  assert.equal(button.textContent, "Open Reframe");

  let opened = 0;
  const unregister = plotCommon.registerDepthReframeRoute?.(() => {
    opened += 1;
  });
  assert.equal(typeof unregister, "function");
  button.click();
  assert.equal(opened, 1, "the refusal action must open the explicit Reframe workflow once");
  assert.match(statuses.at(-1), /no plot data were resampled/iu);
  control.clear();
  assert.equal(control.el.style.display, "none");
  unregister();
  button.click();
  assert.equal(opened, 1, "the registered route must be removable without leaving a hidden listener");

  // The helpers above are necessary but a disconnected refusal/action would still pass. Inventory
  // every pilot consumer and the one shell route; no plotting consumer may invoke Reframe itself.
  const sources = Object.fromEntries(await Promise.all([
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
    "plotCommon.ts",
  ].map(async (file) => [file, await readFile(new URL(`../src/ui/${file}`, import.meta.url), "utf8")])));
  for (const panel of ["crossplotPanel.ts", "histogramPanel.ts", "pickettPanel.ts"]) {
    assert.match(sources[panel], /buildDepthReframeHandoff\(/u, `${panel} must render the refusal action`);
    assert.match(sources[panel], /\.show\(/u, `${panel} must disclose the typed handoff`);
    assert.doesNotMatch(
      sources[panel],
      /runReframe|run_reframe/u,
      `${panel} must not resample merely because it encountered an incompatible plot grid`,
    );
  }
  assert.match(
    sources["plotCommon.ts"],
    /depthReframeByIndex\[i\] = depthReframeHandoff\(error, \[ids\[i\]\], curves\)/u,
    "the shared multi-well loader must retain an actionable handoff for the affected well",
  );
  const ribbonSource = await readFile(new URL("../src/ui/ribbon.ts", import.meta.url), "utf8");
  assert.match(
    ribbonSource,
    /registerDepthReframeRoute\(\(\) => workspace\.openReframe\(\)\)/u,
    "the shell must open the existing explicit Reframe workflow when the user chooses the action",
  );
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

  const unknown = auditedFamilyDisplayDecision({ mnemonic: "RHOB", display_unit: "lb/ft3" });
  assert.equal(unknown.enabled, false);
  assert.equal(unknown.range, null);
  assert.match(unknown.reason, /disabled.*no audited unit-limit row/u);

  const fallback = resolveBoundAxisRange({
    binding: { resolved: [{ mnemonic: "RHOB", display_unit: "lb/ft3" }] },
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

test("every_pilot_histogram_uses_half_open_bins_with_a_closed_final_endpoint_counts_non_finite_samples_separately_and_displays_the_sum_of_bin_counts", async () => {
  // CORRECTNESS - SB-PLT-006 / SB-PLT-T06/T07. docs/PRD_v2/23_plotting-interactivity.md
  // sections 4.2, 5 and 6 cite the plotting dossier sections 2.4, 2.7 and 5.1-5.3:
  // [0,1,2,3] over [0,3] in three bins is [1,1,2], while [0,NaN,+Inf,1]
  // over [0,1] has displayed total 2 and non-finite-excluded count 2. The chapter's
  // sourced product bounds are exactly default=50, minimum=1 and maximum=200 bins.
  const {
    HISTOGRAM_BINS_DEFAULT,
    HISTOGRAM_BINS_MAX,
    HISTOGRAM_BINS_MIN,
    canonicalHistogram,
    normalizeHistogramBinCount,
  } = await load("/src/distribution.ts");
  const { computeHistogram } = await load("/src/ui/histogramPanel.ts");
  const { computeMarginalHistogram } = await load("/src/ui/crossplotPanel.ts");
  const { buildVegaHistogramData } = await load("/src/ui/vegaPanel.ts");

  assert.equal(HISTOGRAM_BINS_DEFAULT, 50);
  assert.equal(HISTOGRAM_BINS_MIN, 1);
  assert.equal(HISTOGRAM_BINS_MAX, 200);
  assert.equal(normalizeHistogramBinCount(0), 1);
  assert.equal(normalizeHistogramBinCount(201), 200);
  assert.equal(normalizeHistogramBinCount(Number.NaN), 50);

  const endpoints = [0, 1, 2, 3];
  const missing = [0, Number.NaN, Number.POSITIVE_INFINITY, 1];
  const adapters = [
    ["canonical", (values, min, max) => canonicalHistogram(values, min, max, 3)],
    ["Histogram", (values, min, max) => computeHistogram(values, min, max, 3)],
    ["Crossplot marginals", (values, min, max) => computeMarginalHistogram(values, min, max, 3, false)],
    ["Vega", (values, min, max) => buildVegaHistogramData(values, min, max, 3)],
  ];
  for (const [surface, run] of adapters) {
    const endpointResult = run(endpoints, 0, 3);
    assert.deepEqual(endpointResult.counts, [1, 1, 2], `${surface}: the final upper endpoint belongs to the final bin`);
    assert.deepEqual(endpointResult.edges, [0, 1, 2, 3], `${surface}: the three equal bins retain their exact edges`);
    assert.equal(endpointResult.displayedTotal, 4, `${surface}: displayed total is the sum of counts`);
    assert.equal(endpointResult.counts.reduce((sum, count) => sum + count, 0), endpointResult.displayedTotal);
    assert.equal(endpointResult.nonFiniteExcluded, 0);

    const missingResult = run(missing, 0, 1);
    assert.equal(missingResult.displayedTotal, 2, `${surface}: only finite in-range samples are displayed`);
    assert.equal(missingResult.counts.reduce((sum, count) => sum + count, 0), 2);
    assert.equal(missingResult.nonFiniteExcluded, 2, `${surface}: NaN and infinity are counted separately`);
  }

  const vega = buildVegaHistogramData(endpoints, 0, 3, 3);
  assert.deepEqual(
    vega.rows,
    [
      { binStart: 0, binEnd: 1, count: 1 },
      { binStart: 1, binEnd: 2, count: 1 },
      { binStart: 2, binEnd: 3, count: 2 },
    ],
    "Vega receives the already-governed bins instead of choosing a private bin transform",
  );

  // Executed adapters above prove arithmetic. This second side inventories the live draw/export
  // routes so a dead shared helper cannot pass while a panel keeps private binning.
  const histogramSource = await readFile(new URL("../src/ui/histogramPanel.ts", import.meta.url), "utf8");
  const crossplotSource = await readFile(new URL("../src/ui/crossplotPanel.ts", import.meta.url), "utf8");
  const vegaSource = await readFile(new URL("../src/ui/vegaPanel.ts", import.meta.url), "utf8");
  const logViewSource = await readFile(new URL("../src/ui/logViewPanel.ts", import.meta.url), "utf8");
  assert.match(histogramSource, /canonicalHistogram\(/, "the primary draw and its static vector redraw share the contract");
  assert.match(histogramSource, /displayed n=\$\{displayedTotal\} of analysis n=/, "the primary axis discloses the bar population");
  assert.match(crossplotSource, /computeMarginalHistogram\(/, "crossplot marginals execute the tested adapter");
  assert.match(crossplotSource, /marginal displayed n X=/, "crossplot discloses both marginal bar populations");
  assert.match(crossplotSource, /drawStatic[\s\S]*drawCrossplot\(/, "crossplot vector export reruns the same marginal draw");
  assert.match(vegaSource, /buildVegaHistogramData\(/, "the live Vega grammar executes the tested pre-bin adapter");
  assert.match(vegaSource, /displayed total=\$\{histogram\.displayedTotal\}/, "Vega discloses the pre-binned bar population");
  assert.doesNotMatch(vegaSource, /field:\s*"x",\s*bin:\s*true/, "Vega may not select a private implicit bin contract");
  assert.match(vegaSource, /current\.view\.toSVG\(\)/, "Vega SVG export uses the rendered governed view");
  assert.match(logViewSource, /canonicalHistogram\(/, "log-view point and array histogram glyphs use the same contract");
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

test("every_plot_statistic_records_its_population_interval_selection_finite_pairs_exclusions_percentile_interpolation_and_standard_deviation_choice", async () => {
  // CORRECTNESS — SB-PLT-009 / SB-PLT-T12/T13. docs/PRD_v2/23_plotting-interactivity.md
  // §4.2 requires each record to disclose population, interval, selection, finite-pair count,
  // exclusion counts, percentile interpolation and sample/population standard-deviation choice.
  // T12 independently supplies count=3, mean=2, P50=2 and two non-finite exclusions for
  // [1,2,3,NaN,+Inf]. T13 and the cited §5.2 box parameters supply P5/P25/P50/P75/P95 over 0..100.
  const { applyPlotRangePolicy } = await load("/src/ui/plotRangePolicy.ts");
  const {
    buildPlotStatisticsRecord,
    formatPlotStatisticsRecord,
    plotStatisticsInterval,
  } = await load("/src/ui/plotCanvas.ts");
  const {
    DEFAULT_HISTOGRAM_OPTIONS,
    buildHistogramStatisticsRecord,
  } = await load("/src/ui/histogramPanel.ts");
  const {
    DEFAULT_CROSSPLOT_OPTIONS,
    buildCrossplotStatisticsRecords,
  } = await load("/src/ui/crossplotPanel.ts");
  const {
    buildVegaBoxStatistics,
    buildVegaStatisticsRecords,
  } = await load("/src/ui/vegaPanel.ts");
  const { buildPickettStatisticsRecords } = await load("/src/ui/pickettPanel.ts");
  const {
    DEFAULT_CORRELATION_OPTIONS,
    buildCorrelationStatisticsRecord,
  } = await load("/src/ui/correlationPanel.ts");

  const input = Float32Array.of(1, 2, 3, Number.NaN, Number.POSITIVE_INFINITY);
  const policy = applyPlotRangePolicy([{ values: input, display: null, validity: null }], false);
  const eligible = Float32Array.from(policy.indices.map((index) => input[index]));
  const context = {
    binding_channel: "x",
    channel: "value",
    population: "active_well",
    well_ids: ["scope-a"],
    interval: { low: 100, high: 101, closure: "[lo,hi)" },
    selection: { kind: "all_eligible", selection_id: null, label: "all eligible", applied: false },
    policy,
    selection_excluded: 0,
    unpaired_or_unclassified_excluded: 0,
  };
  const sample = buildPlotStatisticsRecord(eligible, { ...context, standard_deviation: "sample_n_minus_one" });
  assert.equal(sample.values.count, 3);
  assert.equal(sample.values.mean, 2);
  assert.equal(sample.values.p50, 2);
  assert.equal(sample.values.std, 1, "sample standard deviation divides by n-1");
  assert.equal(sample.population, "active_well");
  assert.equal(sample.channel, "value");
  assert.equal(sample.binding_channel, "x");
  assert.deepEqual(sample.well_ids, ["scope-a"]);
  assert.deepEqual(sample.interval, { low: 100, high: 101, closure: "[lo,hi)" });
  assert.deepEqual(sample.selection, { kind: "all_eligible", selection_id: null, label: "all eligible", applied: false });
  assert.equal(sample.finite_pair_count, 3);
  assert.deepEqual(sample.exclusions, {
    input_count: 5,
    non_finite: 2,
    log_domain: 0,
    validity: 0,
    selection: 0,
    unpaired_or_unclassified: 0,
    display_hidden: 0,
  });
  assert.equal(sample.percentile_interpolation, "linear_index_n_minus_one");
  assert.equal(sample.standard_deviation, "sample_n_minus_one");
  assert.match(formatPlotStatisticsRecord(sample), /active well.*interval=\[100,101\).*selection=all eligible.*finite pairs=3.*non-finite=2.*percentile=linear index \(n-1\).*std=sample \(n-1\)/u);

  const population = buildPlotStatisticsRecord(eligible, {
    ...context,
    population: "pooled",
    well_ids: ["scope-a", "scope-b"],
    selection: { kind: "named", selection_id: "sel-1", label: "selected samples", applied: true },
    standard_deviation: "population_n",
  });
  assert.equal(population.values.std, Math.sqrt(2 / 3), "population standard deviation divides by n");
  assert.equal(population.population, "pooled");
  assert.equal(population.selection.selection_id, "sel-1");
  assert.equal(population.standard_deviation, "population_n");
  assert.deepEqual(
    plotStatisticsInterval(100, null),
    { low: 100, high: null, closure: "[lo,+inf)" },
    "a last-top-to-TD population must not be mislabeled as all depth",
  );
  assert.deepEqual(
    plotStatisticsInterval(null, 101),
    { low: null, high: 101, closure: "(-inf,hi)" },
    "an upper-bounded population must retain its only finite limit",
  );

  const liveRecords = [
    ["Histogram", [buildHistogramStatisticsRecord(
      input, DEFAULT_HISTOGRAM_OPTIONS, "VALUE", "scope-a", 100, 101,
    )]],
    ["Crossplot", buildCrossplotStatisticsRecords(
      input,
      input,
      DEFAULT_CROSSPLOT_OPTIONS,
      "X",
      "Y",
      "scope-a",
      100,
      101,
      null,
      null,
    )],
    ["Vega", buildVegaStatisticsRecords(
      Array.from(input, (x, index) => ({ x, depth: index })),
      "histogram",
      { apply: false, x: null, y: null },
      "scope-a",
      100,
      101,
      "VALUE",
      "",
      null,
      null,
    )],
    ["Pickett", buildPickettStatisticsRecords(
      input,
      input,
      undefined,
      "RT",
      "PHIE",
      "scope-a",
      100,
      101,
      null,
      null,
    )],
    ["Correlation", [buildCorrelationStatisticsRecord(
      input,
      Float32Array.of(10, 11, 12, 13, 14),
      DEFAULT_CORRELATION_OPTIONS,
      ["scope-a", "scope-b"],
      null,
      null,
    )]],
  ];
  for (const [surface, records] of liveRecords) {
    assert.ok(records.length > 0 && records.every(Boolean), `${surface}: the live adapter returns its governed record`);
    for (const record of records) {
      assert.equal(record.finite_pair_count, 3, `${surface}: finite-pair count is T12`);
      assert.equal(record.exclusions.non_finite, 2, `${surface}: non-finite exclusions are T12`);
      assert.equal(record.values.mean, 2, `${surface}: mean is T12`);
      assert.equal(record.values.p50, 2, `${surface}: P50 is T12`);
      assert.equal(record.standard_deviation, "sample_n_minus_one", `${surface}: estimator choice is explicit`);
      if (surface === "Correlation") {
        assert.equal(record.population, "pooled", "Correlation discloses its real pooled-well population");
        assert.deepEqual(record.well_ids, ["scope-a", "scope-b"]);
      }
    }
  }
  const clippedHistogram = buildHistogramStatisticsRecord(
    Float32Array.of(1, 2, 3),
    DEFAULT_HISTOGRAM_OPTIONS,
    "VALUE",
    "scope-a",
    100,
    101,
    "all eligible",
    { min: 2, max: 3 },
  );
  assert.equal(clippedHistogram.finite_pair_count, 3, "Histogram display clipping cannot change statistics n");
  assert.equal(clippedHistogram.exclusions.display_hidden, 1, "Histogram records the finite value hidden by its display range");

  const ordered = Array.from({ length: 101 }, (_, value) => value);
  const box = buildVegaBoxStatistics(ordered);
  assert.deepEqual(
    { lo: box.lo, q1: box.q1, med: box.med, q3: box.q3, hi: box.hi, n: box.n },
    { lo: 5, q1: 25, med: 50, q3: 75, hi: 95, n: 101 },
    "the Vega box uses the same governed P5/P25/P50/P75/P95 contract as Histogram",
  );

  const groupedRows = [
    ...ordered.map((x, depth) => ({ x, depth, group: "1" })),
    { x: Number.NaN, depth: 101, group: "1" },
    { x: 200, depth: 102, group: "2" },
    { x: 300, depth: 103, group: "2" },
    { x: 400, depth: 104 },
  ];
  const raincloudRecords = buildVegaStatisticsRecords(
    groupedRows,
    "raincloud",
    { apply: false, x: null, y: null },
    "scope-a",
    100,
    101,
    "VALUE",
    "",
    { min: 25, max: 250 },
    null,
  );
  assert.equal(raincloudRecords.length, 2, "each displayed raincloud group owns one statistics record");
  assert.deepEqual(
    raincloudRecords.map((record) => ({
      label: record.selection.label,
      applied: record.selection.applied,
      n: record.finite_pair_count,
      nonFinite: record.exclusions.non_finite,
      selection: record.exclusions.selection,
      unclassified: record.exclusions.unpaired_or_unclassified,
      hidden: record.exclusions.display_hidden,
      p50: record.values.p50,
    })),
    [
      { label: "group 1; all eligible", applied: true, n: 101, nonFinite: 1, selection: 2, unclassified: 1, hidden: 25, p50: 50 },
      { label: "group 2; all eligible", applied: true, n: 2, nonFinite: 1, selection: 101, unclassified: 1, hidden: 1, p50: 250 },
    ],
    "group-specific statistics cannot be replaced by one plausible whole-population summary",
  );

  // Arithmetic above is necessary but insufficient: inventory every live plotting consumer and
  // the common screen/export record so a dead helper or value-only chip cannot satisfy the test.
  for (const panel of ["histogramPanel.ts", "crossplotPanel.ts", "pickettPanel.ts", "correlationPanel.ts", "vegaPanel.ts"]) {
    const source = await readFile(new URL(`../src/ui/${panel}`, import.meta.url), "utf8");
    assert.match(source, /buildPlotStatisticsRecord\(/, `${panel} must build the governed record`);
    assert.match(source, /formatPlotStatisticsRecord\(/, `${panel} must disclose the record on screen`);
    assert.match(source, /statisticsRecords:/, `${panel} must carry the record into plot export`);
  }
  const crossplotSource = await readFile(new URL("../src/ui/crossplotPanel.ts", import.meta.url), "utf8");
  assert.match(
    crossplotSource,
    /if \(!plot\) \{[\s\S]*?return;\s*\}\s*refreshStatisticsRecords\(\);/u,
    "the successful live crossplot draw must refresh the records, not only its no-data refusal",
  );
  const ipcSource = await readFile(new URL("../src/ipc.ts", import.meta.url), "utf8");
  const rustSource = await readFile(new URL("../src-tauri/src/plotting.rs", import.meta.url), "utf8");
  assert.match(ipcSource, /statisticsRecords\?: PlotStatisticsRecord\[\]/, "the export scope carries typed statistics records");
  assert.match(rustSource, /statistics_records: Vec<PlotStatisticsRecord>/, "Rust validates and serializes the same records");
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

test("a_long_axis_label_and_outside_legend_stay_uncropped_vectors_while_the_same_print_is_labelled_raster_and_keeps_its_provenance_footer", async () => {
  // CORRECTNESS — SB-PLT-026 / SB-PLT-T37/T38. The chapter requires the same
  // scientific draw at paper scale, uncropped vector labels/legend, and an explicitly
  // labelled raster print retaining its provenance footer. The deliberately small
  // source canvas and out-of-frame text pin automatic bound expansion from both sides;
  // the strings and coordinates are discriminator fixtures, not scientific values.
  const { imageExportMenuEntries, paperProvenanceFooter, rasterPrintRecord } = await load("/src/ui/plotExport.ts");
  const { validatePaperExportRecord } = await load("/src/ui/paperExport.ts");
  const { renderPlotToPaperSvg } = await load("/src/ui/svgExport.ts");
  const { renderPlotToPaperPdf } = await load("/src/ui/pdfExport.ts");
  const scope = {
    wellIds: ["scope-a"],
    curves: ["bulk-density"],
    plotBindings: [{ intent: { channel: "x", semantic_request: "bulk density", required: true }, resolved: [] }],
    axisRanges: [{ axis: "x", min: 1, max: 2, tier: "user" }],
    statisticsRecords: [{ exclusions: { input_count: 11, non_finite: 2, log_domain: 0, validity: 1, selection: 0, unpaired_or_unclassified: 0, display_hidden: 0 } }],
  };
  const axis = "A deliberately long quantitative axis label";
  const legend = "Outside legend";
  const annotation = "Excluded: 3";
  const draws = [];
  const measuredLegendWidths = [];
  const draw = (canvas) => {
    draws.push([canvas.width, canvas.height]);
    const ctx = canvas.__recordingCtx2d;
    ctx.font = "10px sans-serif";
    measuredLegendWidths.push(ctx.measureText(legend).width);
    ctx.textAlign = "right";
    ctx.fillText(axis, -4, canvas.height / 2);
    ctx.textAlign = "left";
    ctx.fillText(legend, canvas.width + 8, 20);
    ctx.fillText(annotation, 20, canvas.height - 8);
    return { theme: { bg: "#ffffff" } };
  };

  const svg = renderPlotToPaperSvg(100, 80, draw, scope);
  const pdf = renderPlotToPaperPdf(100, 80, draw, scope);
  assert.ok(svg);
  assert.ok(pdf);
  for (const text of [axis, legend, annotation, paperProvenanceFooter(scope)]) {
    assert.match(svg, new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")), `${text} remains SVG vector text`);
    assert.match(pdf.content, new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")), `${text} remains PDF vector text`);
  }
  assert.match(svg, /<metadata id="sandibumi-paper-export-v1">/);
  assert.equal(pdf.paperRecord.crop_proof, "all_recorded_bounds_inside_page");
  assert.ok(
    pdf.paperRecord.content_bounds.min_x <= -4 - axis.length * 9,
    "the page uses the rendered label's measured left bound rather than a character-count estimate",
  );
  assert.ok(
    pdf.paperRecord.content_bounds.max_x >= 108 + legend.length * 9,
    "the page uses the rendered legend's measured right bound rather than a character-count estimate",
  );
  assert.ok(pdf.paperRecord.page_bounds.min_x <= pdf.paperRecord.content_bounds.min_x);
  assert.ok(pdf.paperRecord.page_bounds.max_x >= pdf.paperRecord.content_bounds.max_x);
  assert.ok(pdf.paperRecord.page_bounds.min_y <= pdf.paperRecord.content_bounds.min_y);
  assert.ok(pdf.paperRecord.page_bounds.max_y >= pdf.paperRecord.content_bounds.max_y);
  assert.deepEqual(
    measuredLegendWidths,
    [legend.length * 9, legend.length * 9, legend.length * 9],
    "SVG measurement, PDF preflight, and PDF draw consume the same rendered text width",
  );
  assert.ok(draws.length >= 3, "SVG and PDF each rerun the supplied scientific draw; PDF also preflights bounds");
  const sourceCroppingLie = structuredClone(pdf.paperRecord);
  sourceCroppingLie.content_bounds.max_x = 99;
  assert.throws(
    () => validatePaperExportRecord(sourceCroppingLie),
    /source canvas is cropped/,
    "a page cannot prove no crop by declaring content smaller than its source canvas",
  );

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
  assert.ok(labels.includes("Print raster…"));
  const raster = rasterPrintRecord(100, 80, scope);
  assert.equal(raster.medium, "print-raster");
  assert.equal(raster.unit, "px", "raster backing-store pixels are never mislabelled as physical points");
  assert.equal(raster.provenance_footer, paperProvenanceFooter(scope));
  assert.equal(raster.crop_proof, "raster_pixels_preserved_before_browser_print_layout");
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

test("every_async_plot_build_and_refetch_is_registered_and_reverse_order_or_data_revision_completion_never_replaces_the_active_panel", async () => {
  // CORRECTNESS — SB-PLT-029 / SB-PLT-T28/T33. The expected inventory comes from the
  // five plot surfaces in docs/PRD_v2/23_plotting-interactivity.md §3.1 plus T27's
  // viewport refetch; the newest-generation-only result and stale disposal come from §6.
  const {
    PLOT_ASYNC_LOAD_REGISTRY,
    beginPlotAsyncGeneration,
    commitPlotAsyncGeneration,
  } = await load("/src/ui/plotAsync.ts");
  const expectedInventory = [
    ["workspace-plot-build", "src/ui/workspace.ts"],
    ["histogram-data-refetch", "src/ui/histogramPanel.ts"],
    ["histogram-context-refetch", "src/ui/histogramPanel.ts"],
    ["crossplot-data-refetch", "src/ui/crossplotPanel.ts"],
    ["crossplot-core-refetch", "src/ui/crossplotPanel.ts"],
    ["crossplot-context-refetch", "src/ui/crossplotPanel.ts"],
    ["pickett-data-refetch", "src/ui/pickettPanel.ts"],
    ["pickett-context-refetch", "src/ui/pickettPanel.ts"],
    ["correlation-data-refetch", "src/ui/correlationPanel.ts"],
    ["correlation-well-refetch", "src/ui/correlationPanel.ts"],
    ["vega-data-refetch", "src/ui/vegaPanel.ts"],
    ["vega-selector-refetch", "src/ui/vegaPanel.ts"],
    ["vega-editor-load", "src/ui/vegaPanel.ts"],
    ["vega-resize", "src/ui/vegaPanel.ts"],
    ["logview-viewport-refetch", "src/ui/viewportRefetch.ts"],
  ];
  assert.deepEqual(
    PLOT_ASYNC_LOAD_REGISTRY.map(({ id, owner }) => [id, owner]),
    expectedInventory,
  );
  for (const [id, owner] of expectedInventory) {
    const source = await readFile(new URL(`../${owner}`, import.meta.url), "utf8");
    assert.match(
      source,
      new RegExp(`beginPlotAsyncGeneration\\(\\s*[\"']${id}[\"']`),
      `${id} must create its registered token in ${owner}`,
    );
  }
  const workspaceSource = await readFile(new URL("../src/ui/workspace.ts", import.meta.url), "utf8");
  const workspaceCommit = workspaceSource.indexOf("commitPlotAsyncGeneration(token");
  const workspaceAppend = workspaceSource.indexOf("host.appendChild(content.el)", workspaceCommit);
  assert.ok(workspaceCommit >= 0 && workspaceAppend > workspaceCommit, "the workspace checks and disposes before panel replacement");

  const correlationSource = await readFile(new URL("../src/ui/correlationPanel.ts", import.meta.url), "utf8");
  assert.match(correlationSource, /async function reload\(\): Promise<boolean>/);
  assert.match(correlationSource, /\.then\(\(applied\) => \{\s*if \(!applied\) return;\s*draw\(\)/);

  const vegaSource = await readFile(new URL("../src/ui/vegaPanel.ts", import.meta.url), "utf8");
  const detachedHost = vegaSource.indexOf('const stagingHost = document.createElement("div")');
  const detachedEmbed = vegaSource.indexOf("vegaEmbed(stagingHost", detachedHost);
  const vegaGuard = vegaSource.indexOf("isPlotAsyncGenerationCurrent(token", detachedEmbed);
  const activeVegaCommit = vegaSource.indexOf("chartHost.replaceChildren", vegaGuard);
  assert.ok(detachedHost >= 0 && detachedEmbed > detachedHost, "Vega embeds only into a detached host while pending");
  assert.ok(vegaGuard > detachedEmbed && activeVegaCommit > vegaGuard, "Vega checks freshness before touching the live host");
  assert.doesNotMatch(vegaSource, /vegaEmbed\(chartHost/);

  const deferred = () => {
    let resolve;
    const promise = new Promise((done) => { resolve = done; });
    return { promise, resolve };
  };
  let generation = 0;
  let activePanel = "initial";
  const disposed = [];
  const oldLoad = deferred();
  const newestLoad = deferred();
  const oldToken = beginPlotAsyncGeneration("workspace-plot-build", ++generation);
  const oldSettlement = oldLoad.promise.then((content) =>
    commitPlotAsyncGeneration(oldToken, generation, false, content, {
      apply: (value) => { activePanel = value.id; },
      disposeStale: (value) => disposed.push(value.id),
    }));
  const newestToken = beginPlotAsyncGeneration("workspace-plot-build", ++generation);
  const newestSettlement = newestLoad.promise.then((content) =>
    commitPlotAsyncGeneration(newestToken, generation, false, content, {
      apply: (value) => { activePanel = value.id; },
      disposeStale: (value) => disposed.push(value.id),
    }));

  newestLoad.resolve({ id: "newest" });
  assert.equal(await newestSettlement, "applied");
  oldLoad.resolve({ id: "old" });
  assert.equal(await oldSettlement, "stale");
  assert.equal(activePanel, "newest", "reverse-order T28 keeps the newest panel");
  assert.deepEqual(disposed, ["old"], "the superseded build is disposed before any active mutation");

  const priorRevision = deferred();
  const revisionToken = beginPlotAsyncGeneration("histogram-data-refetch", ++generation);
  const priorSettlement = priorRevision.promise.then((content) =>
    commitPlotAsyncGeneration(revisionToken, generation, false, content, {
      apply: (value) => { activePanel = value.id; },
      disposeStale: (value) => disposed.push(value.id),
    }));
  beginPlotAsyncGeneration("histogram-data-refetch", ++generation); // the new data revision
  priorRevision.resolve({ id: "prior-revision" });
  assert.equal(await priorSettlement, "stale");
  assert.equal(activePanel, "newest", "T33's stale data revision never replaces current content");
  assert.deepEqual(disposed, ["old", "prior-revision"]);
});

test("a_superseded_async_plot_build_is_disposed_before_it_can_replace_the_active_panel", async () => {
  // CORRECTNESS — supporting workspace seam for SB-PLT-029 / T28-T33. The whole
  // inventory and executed reverse-order race live in the adjacent requirement test.
  const source = await readFile(new URL("../src/ui/workspace.ts", import.meta.url), "utf8");
  const tokenStart = source.indexOf('beginPlotAsyncGeneration("workspace-plot-build"');
  const generationCommit = source.indexOf("commitPlotAsyncGeneration(token", tokenStart);
  const staleDispose = source.indexOf("disposeStale:", generationCommit);
  const activeAppend = source.indexOf("host.appendChild(content.el)", staleDispose);

  assert.ok(tokenStart >= 0, "the actual workspace build creates its registered generation token");
  assert.ok(generationCommit > tokenStart, "resolved content crosses the shared currentness boundary");
  assert.ok(staleDispose > generationCommit, "stale content is disposed inside that boundary");
  assert.ok(activeAppend > staleDispose, "active-panel mutation remains after stale disposal");
});

test("a_viewport_crossing_its_loaded_high_bound_issues_one_generation_tagged_half_open_refetch_and_only_the_newest_reverse_order_response_renders", async () => {
  // CORRECTNESS — SB-PLT-017 / SB-PLT-T27/T28. The half-open interval, one tagged
  // refetch on a crossed bound, and newest-generation-only render are specified by
  // docs/PRD_v2/23_plotting-interactivity.md §4.3 and §6, citing dossier §§2.8/5.4.
  const { ViewportRefetchCoordinator } = await load("/src/ui/viewportRefetch.ts");
  const coordinator = new ViewportRefetchCoordinator();
  const pending = [];
  const requests = [];
  const rendered = [];
  const pendingNotices = [];
  const failures = [];
  const deferredLoad = (request) => {
    requests.push(request);
    return new Promise((resolve, reject) => pending.push({ resolve, reject }));
  };
  const apply = (value) => rendered.push(value);
  const reportPending = (message) => pendingNotices.push(message);
  const reportFailure = (message) => failures.push(message);

  coordinator.seedLoaded({ sourceKey: "well-a|GR", low: 100, high: 101, targetPixelHeight: 100 });
  const contained = await coordinator.refetch(
    { sourceKey: "well-a|GR", low: 100.1, high: 100.9, targetPixelHeight: 80 },
    deferredLoad,
    apply,
    reportPending,
    reportFailure,
  );
  assert.equal(contained, "loaded", "an equally dense interval already inside the loaded range does not refetch");
  assert.equal(requests.length, 0);

  const first = coordinator.refetch(
    { sourceKey: "well-a|GR", low: 100.5, high: 101.5, targetPixelHeight: 100 },
    deferredLoad,
    apply,
    reportPending,
    reportFailure,
  );
  const duplicate = await coordinator.refetch(
    { sourceKey: "well-a|GR", low: 100.5, high: 101.5, targetPixelHeight: 100 },
    deferredLoad,
    apply,
    reportPending,
    reportFailure,
  );
  assert.equal(duplicate, "pending", "the same crossed-bound request is issued once");
  assert.equal(requests.length, 1);
  assert.equal(pendingNotices.length, 1, "the one in-flight request has one visible provisional-data notice");
  assert.match(pendingNotices[0], /\[100\.5, 101\.5\).*existing samples remain visible/i);
  assert.deepEqual(requests[0], {
    sourceKey: "well-a|GR",
    low: 100.5,
    high: 101.5,
    targetPixelHeight: 100,
    operation: "logview-viewport-refetch",
    generation: 1,
  });

  const second = coordinator.refetch(
    { sourceKey: "well-a|GR", low: 101, high: 102, targetPixelHeight: 100 },
    deferredLoad,
    apply,
    reportPending,
    reportFailure,
  );
  assert.equal(requests[1].generation, 2, "each distinct async request carries a newer generation");
  pending[1].resolve("newest");
  assert.equal(await second, "applied");
  pending[0].resolve("stale");
  assert.equal(await first, "stale");
  assert.deepEqual(rendered, ["newest"], "reverse-order completion cannot repaint with stale samples");

  const failed = coordinator.refetch(
    { sourceKey: "well-a|GR", low: 101.5, high: 102.5, targetPixelHeight: 100 },
    deferredLoad,
    apply,
    reportPending,
    reportFailure,
  );
  pending[2].reject(new Error("offline"));
  assert.equal(await failed, "failed");
  assert.match(failures[0], /101\.5.*102\.5.*existing samples remain/i);

  // The executed coordinator proves the race and interval rules. This second side inventories
  // the live panel route so a dead helper cannot pass while the viewer still stretches old data.
  const panelSource = await readFile(new URL("../src/ui/logViewPanel.ts", import.meta.url), "utf8");
  assert.match(panelSource, /viewportRefetch\.refetch\(/);
  assert.match(panelSource, /getTrackData\([\s\S]*tagged\.low,[\s\S]*tagged\.high/);
  assert.match(panelSource, /this\.message\(pending\)/, "the old trace is labelled while detail is in flight");
  assert.match(panelSource, /this\.message\(failure\)/, "the coordinator's failure is visible in the panel");
});

test("every_plot_uses_one_change_only_invalidation_contract_and_a_theme_change_redraws_each_once_without_replacing_data_or_viewport_while_dispose_cancels_all_work", async () => {
  // CORRECTNESS — SB-PLT-019 / SB-PLT-T32. docs/PRD_v2/23_plotting-interactivity.md
  // §4.4 and §6 require the shared theme/data-revision/interval/selection/size contract,
  // one theme redraw with data and viewport retained, and complete subscription/work cleanup.
  // The counts below are event arithmetic, not scientific values or product defaults.
  const {
    PLOT_INVALIDATION_KINDS,
    subscribePlotInvalidationContract,
  } = await load("/src/ui/plotInvalidation.ts");

  const currentSource = (initial) => {
    let value = initial;
    const listeners = new Set();
    return {
      subscribe(listener) {
        listeners.add(listener);
        listener(value);
        return () => listeners.delete(listener);
      },
      set(next) {
        value = next;
        for (const listener of listeners) listener(next);
      },
      listenerCount: () => listeners.size,
    };
  };
  const sources = {
    theme: currentSource(0),
    dataRevision: currentSource(0),
    interval: currentSource(null),
    selection: currentSource(null),
    size: currentSource({ width: 640, height: 480 }),
  };
  const panels = ["crossplot", "histogram", "pickett", "vega", "correlation"].map((kind) => {
    const data = { identity: `${kind}-data` };
    const viewport = { identity: `${kind}-viewport` };
    const counts = Object.fromEntries(PLOT_INVALIDATION_KINDS.map((event) => [event, 0]));
    let cancelled = 0;
    const subscription = subscribePlotInvalidationContract(sources, {
      theme: () => counts.theme++,
      dataRevision: () => counts.dataRevision++,
      interval: () => counts.interval++,
      selection: () => counts.selection++,
      size: () => counts.size++,
      cancelPending: () => cancelled++,
    });
    return { kind, data, viewport, counts, subscription, cancelled: () => cancelled };
  });

  assert.deepEqual(PLOT_INVALIDATION_KINDS, ["theme", "dataRevision", "interval", "selection", "size"]);
  for (const panel of panels) {
    assert.deepEqual(panel.counts, {
      theme: 0,
      dataRevision: 0,
      interval: 0,
      selection: 0,
      size: 0,
    }, `${panel.kind} initializes from the current snapshot without treating it as a change`);
  }

  const retained = panels.map(({ data, viewport }) => ({ data, viewport }));
  sources.theme.set(1);
  for (let index = 0; index < panels.length; index++) {
    assert.equal(panels[index].counts.theme, 1, `${panels[index].kind} redraws exactly once for one theme change`);
    assert.strictEqual(panels[index].data, retained[index].data, `${panels[index].kind} retains its data`);
    assert.strictEqual(panels[index].viewport, retained[index].viewport, `${panels[index].kind} retains its viewport`);
  }

  sources.dataRevision.set(1);
  sources.interval.set({ wellId: "well-a", topName: "interval", depthMin: 100, depthMax: 101 });
  sources.selection.set({ wellId: "well-a", depths: new Set([100]) });
  sources.size.set({ width: 800, height: 600 });
  for (const panel of panels) {
    assert.deepEqual(panel.counts, {
      theme: 1,
      dataRevision: 1,
      interval: 1,
      selection: 1,
      size: 1,
    }, `${panel.kind} receives every governed invalidation once`);
    panel.subscription.dispose();
    panel.subscription.dispose();
    assert.equal(panel.cancelled(), 1, `${panel.kind} cancels pending work exactly once`);
  }
  for (const source of Object.values(sources)) assert.equal(source.listenerCount(), 0, "disposal removes every governed subscription");

  sources.theme.set(2);
  sources.dataRevision.set(2);
  sources.interval.set(null);
  sources.selection.set(null);
  sources.size.set({ width: 900, height: 700 });
  for (const panel of panels) {
    assert.deepEqual(panel.counts, {
      theme: 1,
      dataRevision: 1,
      interval: 1,
      selection: 1,
      size: 1,
    }, `${panel.kind} performs no work after disposal`);
  }

  // The executable contract alone would let a dead helper pass. Inventory the five live
  // builders and exclude their old independent invalidation lists as the opposite side.
  const liveRegistrations = new Map();
  for (const panel of [
    "crossplotPanel.ts",
    "histogramPanel.ts",
    "pickettPanel.ts",
    "vegaPanel.ts",
    "correlationPanel.ts",
  ]) {
    const source = await readFile(new URL(`../src/ui/${panel}`, import.meta.url), "utf8");
    assert.equal((source.match(/registerPlotInvalidationContract\(/g) ?? []).length, 1, `${panel} registers the shared contract exactly once`);
    assert.doesNotMatch(source, /appState\.(?:themeVersion|dataVersion|selectedInterval|brushedDepths)\.subscribe/, `${panel} has no private governed subscription list`);
    const start = source.indexOf("registerPlotInvalidationContract(");
    const end = source.indexOf("\n  });", start);
    assert.ok(end > start, `${panel} has one inspectable complete registration`);
    const registration = source.slice(start, end + 6);
    for (const event of ["theme", "dataRevision", "interval", "selection", "size", "cancelPending"]) {
      assert.match(registration, new RegExp(`\\b${event}\\s*:`), `${panel} declares a live ${event} handler`);
    }
    liveRegistrations.set(panel, registration);
  }

  for (const pattern of [/theme:\s*\(\)\s*=>\s*redraw\(\)/, /reload\(true\)/, /reloadContext\(\)/, /applySelectedInterval/, /recomputeBrush\(selection\)/, /size:\s*\(\)\s*=>\s*redraw\(\)/, /reloadGen\+\+/, /coreGen\+\+/, /ctxGen\+\+/, /cancelAnimationFrame/]) {
    assert.match(liveRegistrations.get("crossplotPanel.ts"), pattern, `crossplot's handler is not a no-op: ${pattern}`);
  }
  for (const pattern of [/theme:\s*\(\)\s*=>\s*redraw\(\)/, /reload\(true\)/, /reloadContext\(\)/, /applySelectedInterval/, /recomputeBrushValues\(selection\)/, /refreshStatistics\(\)/, /size:\s*\(\)\s*=>\s*redraw\(\)/, /reloadGen\+\+/, /ctxGen\+\+/, /cancelAnimationFrame/]) {
    assert.match(liveRegistrations.get("histogramPanel.ts"), pattern, `histogram's handler is not a no-op: ${pattern}`);
  }
  for (const pattern of [/theme:\s*\(\)\s*=>\s*redraw\(\)/, /reload\(true\)/, /reloadContext\(\)/, /applySelectedInterval/, /brushSet\s*=\s*next/, /size:\s*\(\)\s*=>\s*redraw\(\)/, /reloadGen\+\+/, /ctxGen\+\+/, /cancelAnimationFrame/]) {
    assert.match(liveRegistrations.get("pickettPanel.ts"), pattern, `Pickett's handler is not a no-op: ${pattern}`);
  }
  for (const pattern of [/applyBrush\(selection\)/, /repaint\(\)/, /loadCurveNames\(\)/, /render\(\)/, /applySelectedInterval/, /const resized = current/, /beginPlotAsyncGeneration\("vega-resize"/, /resized\.view\.resize\(\)/, /current === resized/, /gen\+\+/, /cancelAnimationFrame/]) {
    assert.match(liveRegistrations.get("vegaPanel.ts"), pattern, `Vega's handler is not a no-op: ${pattern}`);
  }
  for (const pattern of [/theme:\s*\(\)\s*=>\s*draw\(\)/, /refreshWells\(\)/, /reload\(\)/, /selectedInterval\s*=\s*interval/, /brushSelection\s*=\s*selection/, /size:\s*\(\)\s*=>\s*draw\(\)/, /reloadGen\+\+/, /clearTimeout\(fitTimer\)/, /removeWellsMenu\(\)/]) {
    assert.match(liveRegistrations.get("correlationPanel.ts"), pattern, `Correlation's handler is not a no-op: ${pattern}`);
  }

  const { captureVegaViewportDomains } = await load("/src/ui/vegaPanel.ts");
  assert.deepEqual(
    captureVegaViewportDomains([
      { axis: "x", min: 10, max: 20, tier: "user" },
      { axis: "y", min: 30, max: 40, tier: "user" },
      { axis: "colour", min: 0, max: 1, tier: "finite_data" },
    ]),
    { x: [10, 20], y: [30, 40] },
    "a theme re-embed restores the current pan/zoom domains rather than the original data domains",
  );
});

test("a_chart_missing_its_source_revision_is_blocked_on_screen_save_template_svg_and_pdf_while_one_complete_public_primary_record_survives_all_five", async () => {
  // CORRECTNESS — SB-PLT-023 / SB-PLT-T35. docs/PRD_v2/23_plotting-interactivity.md
  // §§4.5 and 6 require the complete record and the missing-revision refusal on every
  // deliverable route. Pittman, E.D. (1992), AAPG Bulletin 76(2), 191-198, Table 1 is
  // the public-primary citation already classified in chapter 15 §5; this test transports
  // metadata for a non-shipped contract fixture and does not transcribe chart values.
  const { chartRecordForSurface } = await load("/src/ui/chartProvenance.ts");
  const { normalizeCrossplotOptions } = await load("/src/ui/crossplotPanel.ts");
  const payload = JSON.stringify({ fixture: "metadata-only", values: [] });
  const complete = {
    chart_id: "pittman-1992-metadata-fixture",
    title: "Pittman 1992 metadata fixture",
    chart_type: "published-primary-metadata-fixture",
    x_quantity: "porosity",
    x_unit: "%",
    y_quantity: "permeability",
    y_unit: "mD",
    citation: "Pittman, E.D. (1992), AAPG Bulletin 76(2), 191-198, Table 1",
    publisher: "AAPG",
    revision_date: "1992",
    digitizer: "SandiBumi acceptance fixture",
    approved_derivation_path: "independently_digitized_public_primary_source",
    payload_checksum: createHash("sha256").update(payload).digest("hex"),
    transform_applied: "metadata-only; no chart values transcribed",
  };
  const surfaces = ["screen", "save", "template", "svg", "pdf"];

  for (const surface of surfaces) {
    assert.deepEqual(
      chartRecordForSurface(complete.chart_id, complete, surface),
      complete,
      `${surface} must carry the same complete stable record`,
    );
  }

  const saved = normalizeCrossplotOptions({
    chartOverlay: complete.chart_id,
    chartProvenance: complete,
  });
  assert.deepEqual(saved.chartProvenance, complete, "save/template normalization must not discard provenance");

  const missingRevision = { ...complete, revision_date: "" };
  for (const surface of surfaces) {
    assert.throws(
      () => chartRecordForSurface(complete.chart_id, missingRevision, surface),
      new RegExp(`${surface}.*revision`, "iu"),
      `${surface} must refuse the same incomplete record`,
    );
  }
  assert.throws(
    () => chartRecordForSurface("different-chart", complete, "screen"),
    /screen.*identity/iu,
    "a complete record for a different chart must not authorize the selected payload",
  );

  const [crossplot, ipc, backend] = await Promise.all([
    readFile(new URL("../src/ui/crossplotPanel.ts", import.meta.url), "utf8"),
    readFile(new URL("../src/ipc.ts", import.meta.url), "utf8"),
    readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8"),
  ]);
  assert.match(crossplot, /authorizeProvenancedChart\([\s\S]*?"screen"/u);
  assert.match(crossplot, /savePlotProps\("crossplot",\s*persistedChartState\("save"\)\)/u);
  assert.match(crossplot, /buildPlotTemplateBar<[\s\S]*?persistedChartState\("template"\)/u);
  assert.match(crossplot, /surface === "svg" \|\| surface === "pdf" \? surface : "save"/u);
  assert.match(crossplot, /chartRecordForSelectedSurface\(chartSurface\)/u);
  assert.match(crossplot, /chartRenderRecord:\s*chartRenderRecord\s*\?\?/u);
  assert.match(ipc, /chartRenderRecord:\s*scope\?\.chartRenderRecord\s*\?\?/u);
  assert.match(backend, /validate_chart_render_record\(chart_render_record\.as_ref\(\)\)/u);
  assert.match(backend, /embed_chart_render_record_json_in_svg/u);
  assert.match(backend, /embed_chart_render_record_json_in_pdf/u);
});

test("a_focused_accessible_canvas_changes_view_by_keyboard_and_removes_the_handler_on_dispose", async () => {
  // CORRECTNESS — SB-PLT-030 / SB-PLT-T39 cites plotCanvas.ts:527-618 and
  // docs/PRD_v2/23_plotting-interactivity.md §4.6: every interactive plot canvas
  // keeps its current label, focus, pan/zoom, Properties and export routes.
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
  makeCanvasAccessible(canvas, "Earlier finite-pair crossplot");
  makeCanvasAccessible(canvas, "Current finite-pair crossplot");
  assert.equal(attributes.get("role"), "img");
  assert.equal(attributes.get("aria-label"), "Current finite-pair crossplot");
  assert.equal(canvas.tabIndex, 0);

  const view = { current: null };
  let redraws = 0;
  let propertiesRoutes = 0;
  let exportRoutes = 0;
  const accessibility = attachKeyboardPanZoom({
    canvas,
    getPlot: () => ({
      x: { min: 0, max: 10, log: false },
      y: { min: 100, max: 200, log: false },
    }),
    view,
    redraw: () => {
      redraws += 1;
    },
    getLabel: () => "Current finite-pair crossplot",
    openProperties: () => {
      propertiesRoutes += 1;
    },
    focusExport: () => {
      exportRoutes += 1;
    },
  });
  accessibility.refresh();
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
  assert.match(attributes.get("aria-keyshortcuts"), /ArrowLeft.*Home.*P.*E/u);
  listeners.get("keydown")({ key: "p", shiftKey: false, preventDefault() {} });
  listeners.get("keydown")({ key: "E", shiftKey: false, preventDefault() {} });
  assert.equal(propertiesRoutes, 1, "P reaches Properties without a pointer");
  assert.equal(exportRoutes, 1, "E reaches export without a pointer");
  accessibility.dispose();
  assert.equal(listeners.has("keydown"), false);

  const panelContracts = new Map([
    ["histogramPanel.ts", "attachKeyboardPanZoom"],
    ["crossplotPanel.ts", "attachKeyboardPanZoom"],
    ["pickettPanel.ts", "attachKeyboardPanZoom"],
    ["correlationPanel.ts", "attachAccessiblePlotKeyboard"],
    ["vegaPanel.ts", "attachAccessiblePlotKeyboard"],
  ]);
  for (const [file, helper] of panelContracts) {
    const source = await readFile(new URL(`../src/ui/${file}`, import.meta.url), "utf8");
    assert.match(source, new RegExp(`${helper}\\(`), `${file} must use the governed keyboard contract`);
    assert.match(source, /getLabel:/u, `${file} must refresh a current accessible label`);
    if (helper === "attachAccessiblePlotKeyboard") {
      assert.match(source, /changeView:/u, `${file} must route the keyboard command into its real viewport`);
    }
    assert.match(source, /openProperties:/u, `${file} must expose a non-pointer Properties route`);
    assert.match(source, /focusExport:/u, `${file} must expose a non-pointer export route`);
    assert.match(source, /\.dispose\(\)/u, `${file} must remove the accessibility handler on close or replacement`);
  }
  const correlation = await readFile(new URL("../src/ui/correlationPanel.ts", import.meta.url), "utf8");
  assert.match(correlation, /viewTop \+= command\.direction \* span/u);
  assert.match(correlation, /zoomAtCenter\(command\.direction === "in"/u);
  const vega = await readFile(new URL("../src/ui/vegaPanel.ts", import.meta.url), "utf8");
  assert.match(vega, /domains\[command\.axis\] = \[domain\[0\] \+ delta, domain\[1\] \+ delta\]/u);
  assert.match(vega, /void repaint\(domains\)/u);
  const styles = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  assert.match(styles, /\.plot-canvas:focus-visible,\s*\.vega-chart-host canvas:focus-visible/u);
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

test("the_pay_summary_table_heads_and_converts_its_thicknesses_together_and_leaves_the_ratios_alone", async () => {
  // CORRECTNESS — SB-ENV-057. The other place these numbers are read, and the one where the
  // stale label was quietest: only HPV carried a unit at all, and it said metres over whatever
  // the project stored, while Top through Net said nothing. Heading and value are asserted
  // together because either alone still misreports — the same rule the Field Dashboard's CSV
  // follows. HPV converts with the thicknesses because it is one; N/G and the volume-fraction
  // averages are dimensionless and must come through untouched.
  const { appState } = await load("/src/state.ts");
  const { M_PER_FT } = await load("/src/units.ts");
  const { renderPaySummaryTable } = await load("/src/ui/summaryDialog.ts");
  const project = appState.projectDepthUnit.get();
  const display = appState.displayDepthUnit.get();
  const row = {
    well_id: "w1",
    well_name: "SANDI-1",
    zone: "A",
    flag: "PAY",
    top: 1000,
    bottom: 1050,
    gross: 50,
    net: 25,
    ntg: 0.5,
    avg_vsh: 0.3,
    avg_phie: 0.18,
    avg_swe: 0.4,
    hpv: 12.5,
    n_classified: 100,
    perm_cutoff_no_data: false,
  };

  try {
    appState.projectDepthUnit.set("FT");
    appState.displayDepthUnit.set("FT");
    const asStored = document.createElement("div");
    renderPaySummaryTable(asStored, [row]);
    const storedHead = asStored.children[0].children[0].innerHTML;
    assert.match(storedHead, /<th>Net \(ft\)<\/th>/);
    assert.match(storedHead, /<th>HPV \(ft\)<\/th>/);
    assert.doesNotMatch(storedHead, /\(m\)/);
    const storedBody = asStored.children[0].children[0].children[0].children[0].innerHTML;
    assert.match(storedBody, /<td>25\.0<\/td>/, "a foot project read in feet converts nothing");
    assert.match(storedBody, /<td>12\.50<\/td>/);

    appState.displayDepthUnit.set("M");
    const asShown = document.createElement("div");
    renderPaySummaryTable(asShown, [row]);
    const shownHead = asShown.children[0].children[0].innerHTML;
    assert.match(shownHead, /<th>HPV \(m\)<\/th>/, "the heading moves with the values");
    assert.doesNotMatch(shownHead, /\(ft\)/);
    const shownBody = asShown.children[0].children[0].children[0].children[0].innerHTML;
    assert.match(shownBody, new RegExp(`<td>${(25 * M_PER_FT).toFixed(1)}</td>`));
    assert.match(shownBody, new RegExp(`<td>${(12.5 * M_PER_FT).toFixed(2)}</td>`));
    assert.match(shownBody, /<td>0\.50<\/td>/, "N/G is a ratio and does not convert");
    assert.match(shownBody, /<td>0\.180<\/td>/, "a volume fraction does not convert");
  } finally {
    appState.projectDepthUnit.set(project);
    appState.displayDepthUnit.set(display);
  }
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

/** Every hard-coded depth-unit label left in the frontend, each classified. A row is a claim
 *  that the literal is CORRECT as written; the sweep below refuses any occurrence that is not
 *  on this list, and any row here that no longer exists.
 *
 *  - `unit-picker` — the control whose whole job is choosing between metres and feet.
 *  - `map-coordinate` — a UTM easting or northing. Metres because the PROJECTION is, which has
 *    nothing to do with which unit this project's depths are logged in. These must never be
 *    swept along with the depth family.
 *
 *  There is deliberately no third category. Every label over a depth, a thickness or a
 *  hydrocarbon pore thickness now resolves from the project, and each was converted in the same
 *  change that relabelled it — a converted value under a stale heading and a stale value under a
 *  converted heading are the same lie.
 */
const HARD_CODED_DEPTH_UNIT_LABELS = [
  // `buildDepthUnitSelect` is the ONE depth-unit picker (core wizard, Import Deviation, Import
  // SCAL). Three copies of these labels would be three places for this classification to drift,
  // so the helper is where they live and this list stays one file long.
  ["src/ui/followCore.ts", "Metres (m)", "unit-picker"],
  ["src/ui/followCore.ts", "Feet (ft)", "unit-picker"],
  ["src/ui/ribbon.ts", "easting (m)", "map-coordinate"],
  ["src/ui/ribbon.ts", "northing (m)", "map-coordinate"],
];

test("every_hard_coded_depth_unit_label_left_in_the_frontend_is_one_somebody_classified", async () => {
  // CORRECTNESS — SB-ENV-057. A depth a user TYPES is labelled in the project's stored unit
  // (it reaches the backend unconverted); a depth a user READS is labelled in the display unit.
  // Neither can be a literal, because the project decides. This sweep is the standing guard: a
  // new dialog that hard-codes a metre label fails here instead of shipping a foot project a
  // form that asks for metres. Rows are matched as (file, literal) pairs, so moving a line is
  // free and changing what it claims is not.
  const files = [];
  const walk = async (dir) => {
    const { readdir } = await import("node:fs/promises");
    for (const entry of await readdir(dir, { withFileTypes: true })) {
      const full = `${dir}/${entry.name}`;
      if (entry.isDirectory()) await walk(full);
      else if (entry.name.endsWith(".ts")) files.push(full);
    }
  };
  await walk("src");

  const found = [];
  for (const file of files) {
    const text = await readFile(file, "utf8");
    for (const raw of text.split(/\r?\n/)) {
      const trimmed = raw.trim();
      // Prose about the rule is not the rule being broken.
      if (trimmed.startsWith("//") || trimmed.startsWith("*") || trimmed.startsWith("/*")) continue;
      for (const pattern of [
        /"([^"\n]*(?:^|\s)\((?:m|ft)\)[^"\n]*)"/g,
        /`([^`\n]*(?:^|\s)\((?:m|ft)\)[^`\n]*)`/g,
      ]) {
        let match;
        while ((match = pattern.exec(raw))) found.push(JSON.stringify([file, match[1]]));
      }
    }
  }

  const classified = new Set(HARD_CODED_DEPTH_UNIT_LABELS.map(([f, l]) => JSON.stringify([f, l])));
  const unclassified = [...new Set(found)].filter((row) => !classified.has(row));
  assert.deepEqual(
    unclassified.map((row) => JSON.parse(row)),
    [],
    "a hard-coded depth-unit label must be resolved from the project, or classified as a unit picker, a map coordinate, or a display label still awaiting its conversion",
  );

  const present = new Set(found);
  const stale = HARD_CODED_DEPTH_UNIT_LABELS.filter(([f, l]) => !present.has(JSON.stringify([f, l])));
  assert.deepEqual(
    stale.map(([f, l]) => [f, l]),
    [],
    "a classified row that no longer exists must be deleted, so the list keeps meaning what it says",
  );
});

test("a_project_native_parameter_is_labelled_in_the_stored_unit_and_never_follows_the_view_preference", async () => {
  // CORRECTNESS — SB-ENV-057. A module argument declared with the project-native depth token
  // has no fixed unit: the number the user types goes to the backend unconverted and is
  // differenced against the stored depth grid. So its label must name the STORED unit — and
  // deliberately NOT the display unit, which is the opposite choice from a read-only panel
  // like the Field Dashboard, and for the opposite reason: there the number is leaving, here
  // it is arriving. Labelling a free-water level with a view preference would invite exactly
  // the mis-entry the token exists to prevent. Every genuinely fixed unit — including the
  // metres a module converts for itself — must pass through untouched.
  const { appState } = await load("/src/state.ts");
  const { argumentUnitLabel, PROJECT_DEPTH_UNIT_TOKEN } = await load("/src/depthUnitPref.ts");
  const project = appState.projectDepthUnit.get();
  const display = appState.displayDepthUnit.get();

  try {
    appState.projectDepthUnit.set("FT");
    appState.displayDepthUnit.set("FT");
    assert.equal(argumentUnitLabel(PROJECT_DEPTH_UNIT_TOKEN), "ft");

    // The user switches the VIEW to metres. The stored grid has not moved, so neither does
    // the label on an input whose value is compared against that grid.
    appState.displayDepthUnit.set("M");
    assert.equal(
      argumentUnitLabel(PROJECT_DEPTH_UNIT_TOKEN),
      "ft",
      "an input label follows the stored unit, never the view preference",
    );

    appState.projectDepthUnit.set("M");
    assert.equal(argumentUnitLabel(PROJECT_DEPTH_UNIT_TOKEN), "m");

    // Fixed units are untouched, the converted metres among them.
    for (const fixed of ["m", "ft", "g/cc", "v/v", "mD", "dyn/cm", "degC"]) {
      assert.equal(argumentUnitLabel(fixed), fixed);
    }
    assert.equal(argumentUnitLabel(""), "");
    assert.equal(argumentUnitLabel(null), "");
  } finally {
    appState.projectDepthUnit.set(project);
    appState.displayDepthUnit.set(display);
  }
});

test("a_dashboard_csv_carries_lengths_and_a_heading_in_one_resolved_unit_and_leaves_the_dimensionless_columns_alone", async () => {
  // CORRECTNESS — the Field Dashboard's CSV is a client deliverable. It printed a
  // hard-coded "(m)" over values the backend returns in the PROJECT's stored unit, so a
  // foot-declared project exported a header claiming metres above columns of feet: wrong by
  // 3.28084x with every number plausible. The two halves are pinned separately because
  // either alone still ships a wrong file — a converted value under a stale heading is the
  // same lie as a stale value under a converted heading. HPV rides with the lengths because
  // it is a hydrocarbon pore THICKNESS; N/G and the volume-fraction averages are
  // dimensionless and must survive the conversion untouched.
  const { appState } = await load("/src/state.ts");
  const { M_PER_FT } = await load("/src/units.ts");
  const { buildDashboardCsv } = await load("/src/ui/dashboardPanel.ts");
  const project = appState.projectDepthUnit.get();
  const display = appState.displayDepthUnit.get();

  const row = {
    well_id: "w1",
    well_name: "SANDI-1",
    zone: "A",
    flag: "PAY",
    top: 1000,
    bottom: 1050,
    gross: 50,
    net: 25,
    not_net: 20,
    unknown: 5,
    ntg_known: 0.55,
    residual_absorbed: 0,
    frame: "MD",
    weights_source: "MD",
    unfiltered: [],
    ntg: 0.5,
    avg_vsh: 0.3,
    avg_phie: 0.18,
    avg_swe: 0.4,
    hpv: 12.5,
    n_classified: 100,
    perm_cutoff_no_data: false,
    quicklook_phie_excluded: false,
  };
  const cell = (csv, label) => {
    const [head, data] = csv.split("\r\n");
    const at = head.split(",").indexOf(label);
    assert.ok(at >= 0, `no column headed ${label} in: ${head}`);
    return data.split(",")[at];
  };

  try {
    // A foot project read in feet: nothing to convert, and the heading says so.
    appState.projectDepthUnit.set("FT");
    appState.displayDepthUnit.set("FT");
    const asStored = buildDashboardCsv([row]);
    assert.match(asStored.split("\r\n")[0], /HPV \(ft\)/, "the heading follows the displayed unit");
    assert.equal(cell(asStored, "Net (ft)"), "25");
    assert.equal(cell(asStored, "HPV (ft)"), "12.5");

    // The same foot project read in metres: the lengths convert, the heading moves with them,
    // and the dimensionless columns do not move at all.
    appState.displayDepthUnit.set("M");
    const asDisplayed = buildDashboardCsv([row]);
    assert.match(asDisplayed.split("\r\n")[0], /HPV \(m\)/);
    assert.doesNotMatch(asDisplayed.split("\r\n")[0], /\(ft\)/);
    assert.equal(Number(cell(asDisplayed, "HPV (m)")), 12.5 * M_PER_FT);
    assert.equal(Number(cell(asDisplayed, "Net (m)")), 25 * M_PER_FT);
    assert.equal(cell(asDisplayed, "N/G"), "0.5", "a ratio is not a length");
    assert.equal(cell(asDisplayed, "Avg PHIE"), "0.18", "a volume fraction is not a length");

    // A metre project stays byte-identical to what it always exported.
    appState.projectDepthUnit.set("M");
    const metric = buildDashboardCsv([row]);
    assert.match(metric.split("\r\n")[0], /HPV \(m\)/);
    assert.equal(cell(metric, "HPV (m)"), "12.5");
    assert.equal(cell(metric, "Top (m)"), "1000");
  } finally {
    appState.projectDepthUnit.set(project);
    appState.displayDepthUnit.set(display);
  }
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

test("the_live_despike_ceiling_names_every_estimator_branch_and_what_the_percentage_means", async () => {
  // CORRECTNESS — SB-ENV-031 presentation half. docs/PRD_v2/20_envcorr-qc.md §4.4 and
  // §6 T40/T69 require 33.33 % for the zero-MAD fallback at k=3, 50.00 % for true MAD,
  // and the explicit masking meaning. The percentages arrive from the Rust estimator preview;
  // this test pins that the dialog does not collapse two actual branches into one number.
  const { renderDespikeContaminationPreview } = await load("/src/ui/moduleDialog.ts");
  const host = document.createElement("div");
  renderDespikeContaminationPreview(
    host,
    {
      branches: [
        { estimator: "TRUE_MAD", ceiling_pct: 50, sample_count: 11 },
        { estimator: "MEAN_DEVIATION_FALLBACK", ceiling_pct: 100 / 3, sample_count: 7 },
      ],
      evaluated_wells: 2,
      unavailable_well_ids: ["missing"],
      issues: [],
    },
    (wellId) => (wellId === "missing" ? "Unavailable curve" : wellId),
  );

  assert.match(host.textContent, /True MAD: 50\.00%/);
  assert.match(host.textContent, /Mean-deviation fallback \(zero MAD\): 33\.33%/);
  assert.match(
    host.textContent,
    /Above this fraction of contaminated samples in a window, spikes mask each other and are not detected\./,
  );
  assert.match(host.textContent, /Computed from the selected curve and current window in 2 wells\./);
  assert.match(host.textContent, /Curve unavailable after the current mask: Unavailable curve\./);
});

test("a_missing_required_well_input_is_marked_beside_its_sourced_condition_before_the_run", async () => {
  // CORRECTNESS — SB-ENV-008 / SB-ENV-T14. docs/PRD_v2/20_envcorr-qc.md section 4.1
  // and section 6.1 T14 require the dialog to show the declared condition and source beside
  // its field, and to mark an absent required well input before launch. The two physical-state
  // fixtures below pin both sides; neither number is a petrophysical value or product default.
  const {
    renderValidityConditions,
    validityConditionViews,
  } = await load("/src/ui/moduleDialog.ts");
  const condition = {
    kind: "required_where_finite",
    id: "gr_hole_corr.caliper_coverage",
    statement: "Caliper is required at every finite GR sample.",
    source: "docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and section 6.2 T11/T12",
    input: "GR",
  };
  const logArg = (name, selected, validityConditions = []) => ({
    name,
    desc: `${name} input`,
    unit: "",
    kind: "log_in",
    default: selected,
    default_source: "",
    choices: [],
    validity_conditions: validityConditions,
    min: null,
    max: null,
    required: true,
  });
  const spec = {
    name: "gr_hole_corr",
    title: "GR environmental correction",
    category: "Condition",
    doc: "",
    args: [logArg("GR", "GR"), logArg("CALI", "CALI", [condition])],
  };
  const selected = { GR: "GR", CALI: "CALI" };
  const missing = validityConditionViews(
    spec.args[1],
    spec,
    selected,
    [
      { well_id: "CALIPER_PRESENT", available_arguments: ["GR", "CALI"], error: null },
      { well_id: "CALIPER_ABSENT", available_arguments: ["GR"], error: null },
    ],
    (id) => id,
  );
  const missingHost = document.createElement("div");
  renderValidityConditions(missingHost, missing);
  assert.match(missingHost.textContent, /gr_hole_corr\.caliper_coverage/);
  assert.match(missingHost.textContent, /Caliper is required at every finite GR sample/);
  assert.match(missingHost.textContent, /Source: docs\/PRD_v2\/20_envcorr-qc\.md/);
  assert.match(missingHost.textContent, /Cannot evaluate before run/);
  assert.match(missingHost.textContent, /CALI/);
  assert.match(missingHost.textContent, /CALIPER_ABSENT/);

  const available = validityConditionViews(
    spec.args[1],
    spec,
    selected,
    [{ well_id: "CALIPER_PRESENT", available_arguments: ["GR", "CALI"], error: null }],
    (id) => id,
  );
  const availableHost = document.createElement("div");
  renderValidityConditions(availableHost, available);
  assert.match(availableHost.textContent, /Inputs available before run/);
  assert.doesNotMatch(availableHost.textContent, /Cannot evaluate before run/);

  const source = await readFile(new URL("../src/ui/moduleDialog.ts", import.meta.url), "utf8");
  assert.match(source, /moduleInputAvailability\(/u, "the pane must query scoped runner input availability");
  assert.match(source, /renderValidityConditions\(/u, "the query result must reach the visible field-adjacent surface");
});

test("source_bearing_picking_guidance_is_rendered_beside_the_parameter_without_becoming_its_value", async () => {
  // CORRECTNESS — SB-CLY-042 and docs/PRD_v2/10_clay-volume.md sections 3.5 F17,
  // 4.3 and 5. The literal advice/source are chapter fixtures; the empty default is the
  // independently specified distinction between a picking convention and a numeric endpoint.
  const { argumentHint } = await load("/src/ui/moduleDialog.ts");
  const arg = {
    name: "GR_MA",
    desc: "Gamma ray matrix (clean)",
    unit: "gapi",
    kind: "param",
    default: "",
    default_source: "ABSENT",
    choices: [],
    guidance: [{
      text: "Pool comparable rock, pre-clip the distribution, then select the documented percentile convention.",
      source: "docs/PRD_v2/10_clay-volume.md section 3.5 F17",
    }],
    min: 0,
    max: 200,
    required: true,
  };

  const hint = argumentHint(arg);
  assert.match(hint, /Guidance: Pool comparable rock, pre-clip the distribution/);
  assert.match(hint, /Source: docs\/PRD_v2\/10_clay-volume\.md section 3\.5 F17/);
  assert.match(hint, /Default: ABSENT/);
  assert.equal(arg.default, "", "rendering advice must never populate the numeric value slot");

  const withoutGuidance = argumentHint({ ...arg, guidance: [] });
  assert.doesNotMatch(withoutGuidance, /Pool comparable rock/);
  assert.match(withoutGuidance, /Default: ABSENT/, "absence stays explicit even with no advice");
});

test("a_converted_clay_default_shows_its_artefact_unit_named_conversion_and_canonical_value", async () => {
  // CORRECTNESS — SB-CLY-054 / SB-CLY-T42. Chapter 10 section 6 gives the exact
  // 1000 k/m3 -> 1 g/cc instance through the cited Geolog RHO_FL default and requires the
  // artefact unit plus named conversion to remain visible; the fixture is not current UI text.
  const { argumentHint } = await load("/src/ui/moduleDialog.ts");
  const arg = {
    name: "RHO_FL",
    desc: "Fluid density",
    unit: "g/cc",
    kind: "param",
    default: "1",
    default_source: "Geolog vsh_dn.info RHO_FL DEFAULT 1000 k/m3",
    default_unit_custody: {
      artefact_value: 1000,
      artefact_unit: "k/m3",
      canonical_value: 1,
      canonical_unit: "g/cc",
      conversion: {
        identity: "curve-units-v2:kg/m3->g/cc",
        from_unit: "kg/m3",
        to_unit: "g/cc",
        factor: 0.001,
        offset: 0,
        derivation: "1 g/cc = 1000 kg/m3; kg/m3 -> g/cc divides by 1000",
      },
    },
    choices: [],
    min: 0.5,
    max: 1.5,
    required: true,
  };

  const hint = argumentHint(arg);
  assert.match(hint, /Artefact value: 1000 k\/m3/);
  assert.match(hint, /Canonical value: 1 g\/cc/);
  assert.match(hint, /Named conversion: curve-units-v2:kg\/m3->g\/cc/);
  assert.match(hint, /1 g\/cc = 1000 kg\/m3/);
  assert.match(hint, /Default source: Geolog vsh_dn\.info RHO_FL DEFAULT 1000 k\/m3/);
  assert.equal(arg.default, "1", "rendering custody must never rewrite the effective value");
});

test("a_disputed_parameter_stays_empty_beside_every_source_and_failed_evidence_loading_stays_visible", async () => {
  // CORRECTNESS — SB-CLY-050 requires every held position and source at the point of entry,
  // with no selected value. These unequal source rows are copied from SB-CLY-T18 in
  // docs/PRD_v2/10_clay-volume.md; the failure branch is the requirement's no-silent-selection
  // rule applied when the evidence cannot be loaded.
  const { withParamSources, buildParamSources } = await load("/src/ui/paramSources.ts");
  const input = document.createElement("input");
  input.value = "";
  const rows = [
    {
      product: "Techlog documentation",
      value: "2.40",
      note: "documented shale density",
      source: "Techlog petrophysics-vsh-from-neutrondensity.html",
      tier: "T1-prime",
    },
    {
      product: "Techlog template",
      value: "2.45",
      note: "shipped template shale density",
      source: "Techlog C2_method_defaults.json RHOB_shale",
      tier: "T3",
    },
  ];

  const stack = withParamSources(input, "shale_density", async () => rows);
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(input.value, "", "rendering evidence must not choose or average a value");
  const panel = stack.children[1];
  assert.equal(panel.hidden, false);
  assert.match(panel.children[0].textContent, /Shipped values elsewhere \(2\)/);
  assert.equal(panel.children[1].hidden, true, "the evidence starts collapsed beside the input");
  panel.children[0].click();
  assert.equal(panel.children[1].hidden, false);
  assert.match(panel.children[1].textContent, /2\.40Techlog documentation/);
  assert.match(panel.children[1].textContent, /T1-prime · Techlog petrophysics-vsh-from-neutrondensity\.html/);
  assert.match(panel.children[1].textContent, /2\.45Techlog template/);
  assert.match(panel.children[1].textContent, /T3 · Techlog C2_method_defaults\.json RHOB_shale/);

  const unavailable = buildParamSources("shale_density", async () => {
    throw new Error("synthetic source registry outage");
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(unavailable.hidden, false);
  assert.match(unavailable.textContent, /Source comparison unavailable/);
  assert.match(unavailable.textContent, /not thereby adjudicated/);
});

test("the_lorenz_capacity_totals_follow_the_projects_depth_unit_while_the_coefficient_stays_unit_free", async () => {
  // CORRECTNESS - Codex whole-repo P1. Two different kinds of number share one headline and only
  // the capacity totals are lengths. k is in mD and phi is v/v, so each of Sigma-k-h and
  // Sigma-phi-h carries exactly one factor of depth; the coefficient is built from cumulative
  // FRACTIONS of those same sums, so a uniform depth factor cancels out of it exactly. The
  // fixture is the finding's own scenario: a 100 ft interval at 1 mD and phi 0.20, which the
  // backend returns as total_kh 100 and total_phih 20 in the project's own unit.
  const { lorenzHeadline } = await load("/src/ui/lorenzDialog.ts");
  const res = { lorenz_coefficient: 0.42, total_kh: 100, total_phih: 20 };

  // A metre project: nothing converts, and this is the control that says the fix did not move
  // the reading anyone already had.
  const metres = lorenzHeadline(res, "m", (v) => v);
  assert.match(metres, /100\.0 mD·m/);
  assert.match(metres, /20\.00 m/);

  // A foot project READ IN FEET: the numbers are unchanged and the LABEL is what must follow,
  // because these totals arrive in the project's own unit and are not being converted at all.
  const feet = lorenzHeadline(res, "ft", (v) => v);
  assert.match(feet, /100\.0 mD·ft/, "the totals are project-native, so feet must say ft");
  assert.match(feet, /20\.00 ft/);
  assert.doesNotMatch(feet, /mD·m/, "the hard-coded metre label is what this fixes");

  // A foot project read in METRES: now the numbers convert too, and both must move together -
  // 100 ft is 30.48 mD·m and 20 ft is 6.096 m, the finding's own arithmetic.
  const shown = (v) => v * 0.3048;
  const converted = lorenzHeadline(res, "m", shown);
  assert.match(converted, /30\.48 mD·m/);
  assert.match(converted, /6\.096 m/);

  // THE OTHER SIDE, and the one that matters most: the coefficient is unit-free and must be
  // byte-identical in all three. A fix that converted everything on the line would be a wrong
  // heterogeneity answer, which is the whole point of the plot.
  for (const line of [metres, feet, converted]) {
    assert.match(line, /Lorenz coefficient 0\.420 /, `coefficient must not convert: ${line}`);
  }
});

test("the_packed_ipc_envelope_decodes_to_the_columns_the_header_names_and_keeps_a_missing_sample_missing", async () => {
  // Codex whole-repository review, P2. `equations::pack_frame` writes a JSON header plus anonymous
  // f32 columns; `decodeFrame` is the reading half. The Rust side pins what is WRITTEN — this pins
  // what is READ, because a mistake in this offset arithmetic is silent: misaligned floats still
  // look like saturations, and nothing downstream can tell.
  const { decodeFrame } = await load("/src/ipc.ts");

  const header = JSON.stringify({ columns: ["depth", "ARCHIE"], n: 3 });
  const headerBytes = new TextEncoder().encode(header);
  const depth = [2000, 2000.5, 2001];
  // NaN is how rule 2 spells "missing"; an f32 column has no null, so it has to survive as NaN.
  // Values chosen exact in f32 — 0.4 is not, and would fail on the round trip for a reason that
  // has nothing to do with the envelope.
  const archie = [0.5, NaN, 0.75];
  const size = 4 + headerBytes.length + 4 + (4 + depth.length * 4) + (4 + archie.length * 4);
  const buf = new ArrayBuffer(size);
  const view = new DataView(buf);
  let off = 0;
  view.setUint32(off, headerBytes.length, true);
  off += 4;
  new Uint8Array(buf).set(headerBytes, off);
  off += headerBytes.length;
  view.setUint32(off, 2, true);
  off += 4;
  for (const column of [depth, archie]) {
    view.setUint32(off, column.length, true);
    off += 4;
    for (const v of column) {
      view.setFloat32(off, v, true);
      off += 4;
    }
  }
  assert.equal(off, size, "the fixture writes exactly the envelope it describes");

  const decoded = decodeFrame(buf);
  assert.deepEqual(decoded.header.columns, ["depth", "ARCHIE"], "the header round-trips");
  assert.equal(decoded.columns.length, 2, "one array per packed column");
  assert.ok(decoded.columns[0] instanceof Float32Array, "columns arrive as Float32Array, not JSON numbers");
  assert.deepEqual(Array.from(decoded.columns[0]), depth, "depth survives byte for byte");

  // The second column is the one that proves the offsets: it is only correct if the first
  // column's length prefix and payload were both consumed exactly.
  const values = decoded.columns[1];
  assert.equal(values[0], 0.5);
  assert.ok(Number.isNaN(values[1]), "a non-physical sample stays MISSING across the bridge");
  assert.equal(values[2], 0.75);

  // And pairing is BY NAME, the rule the callers follow.
  const byName = (name) => decoded.columns[decoded.header.columns.indexOf(name)];
  assert.equal(byName("ARCHIE")[0], 0.5, "a column is found by its name, never by its position");
});

test("a value with no place on a log axis is drawn by neither the screen nor the print", async () => {
  const { valueFrac } = await load("/src/ui/plotCanvas.ts");

  // The rule is `composite.rs::value_frac`, restated once so the two renderers cannot disagree.
  // ZERO IS NOT A SMALL PERMEABILITY. On a log axis it has no position at all, and the log view
  // used to substitute Math.max(v, 1e-6): a PERM written as exactly 0.0 over a tight streak drew
  // a continuous dip to the track edge, and the crossover shading filled the interval, while the
  // print showed an honest gap. Both statements cannot be right about the same rock.
  assert.equal(valueFrac(0, 0.1, 1000, true), null, "PERM = 0 has no position on a log axis");
  assert.equal(valueFrac(-2, 0.1, 1000, true), null, "and neither does a negative sample");
  assert.equal(valueFrac(NaN, 0.1, 1000, true), null, "missing stays missing");

  // A track whose own min is non-positive is refused rather than rendered against a substituted
  // decade - that was the version where screen and print disagreed about a WHOLE track, not one
  // sample: the screen drew something and the print drew nothing.
  assert.equal(valueFrac(5, 0, 1000, true), null, "a log track cannot start at zero");

  // What it DOES answer, and the decade arithmetic that has to stay right.
  assert.ok(Math.abs(valueFrac(10, 0.1, 1000, true) - 0.5) < 1e-12, "0.1..1000 puts 10 mid-track");
  assert.ok(Math.abs(valueFrac(0.1, 0.1, 1000, true) - 0) < 1e-12, "min sits at 0");
  assert.ok(Math.abs(valueFrac(1000, 0.1, 1000, true) - 1) < 1e-12, "max sits at 1");

  // Linear keeps zero and negatives, because there they are ordinary values - an SP of -80 mV
  // is a measurement, and refusing it would be the same defect wearing the opposite sign.
  assert.ok(Math.abs(valueFrac(0, -100, 100, false) - 0.5) < 1e-12, "0 is mid-track on linear");
  assert.ok(Math.abs(valueFrac(-80, -100, 100, false) - 0.1) < 1e-12, "a negative SP is a value");
  assert.equal(valueFrac(5, 3, 3, false), null, "a zero-width track has no position for anything");

  // Deliberately NOT clamped: off-scale is the caller's decision - a continuous curve clamps at
  // the track edge, a point sample is skipped. Folding that in here would take the choice away.
  assert.ok(valueFrac(2000, 0.1, 1000, true) > 1, "off-scale is reported, not clamped");
});
