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
