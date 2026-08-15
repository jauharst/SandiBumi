// Interactive Vega-Lite chart panel (features V1–V3). A well-bound plot rendered by the
// vega-embed engine — grammar-of-graphics interactivity (hover tooltips, drag-brush, scroll-zoom)
// the hand-rolled Canvas-2D plots don't give for free. Lazy-loaded (vega is large) so it stays out
// of the main bundle until the user opens a Vega chart; see workspace.createPlot.
//
// V2 scope: a control bar — chart type (scatter / line / histogram), X / Y curves, a colour curve
// (scatter only, viridis), and a zone filter (restrict to the selected zone's depth range).
//
// V3 scope: workspace integration.
//   - Live theme repaint: re-embed from cached rows with the new theme's CSS vars on themeVersion.
//   - Linked brushing (emit): plain-drag draws a Vega interval brush; the samples inside are
//     published to appState.brushedDepths so the crossplot / histogram / log view highlight the
//     same depths. Pan moves to Shift-drag, zoom stays on the wheel so plain-drag is free to brush.
//   - Linked brushing (consume): a brush arriving from any panel (of this well) dims the
//     non-selected scatter points via an opacity condition driven by two runtime signals. Depths
//     ride the same shared grid across panels, so membership is a direct datum.depth lookup.
//   Line charts emit but don't dim (per-vertex opacity is meaningless on a path); histograms do
//   neither (bars are aggregates).
//
// V4 scope: the panel earns its keep as a report/export surface.
//   - Export: PNG (copy / save / print) reuses the shared canvas export buttons — vega renders to a
//     <canvas> — plus a true-vector SVG from vega's own renderer.
//   - Spec editor: a CodeMirror JSON view of the effective Vega-Lite spec (data elided) that the
//     user can edit and Apply as an override; the control bar still drives which curves/zone fill it.
//     CodeMirror is dynamic-imported so it stays out of the chunk until the editor is opened.
//   - Last-used persistence: the control selections are saved via savePlotProps so a new panel opens
//     where the previous one left off (getState still carries settings across a well switch).
//
// V5 scope: analytical modes.
//   - Density: a 2D binned heatmap (chart type) for clouds too dense to read as a scatter.
//   - Trend: a regression overlay on the scatter (fit line + R²), method linear/log/exp/pow/quad.
//     Layered spec — the selection params sit on the points layer, the variable signals top-level.
import vegaEmbed, { type VisualizationSpec, type Result as VegaResult } from "vega-embed";
import type { EditorView } from "codemirror";
import {
  HISTOGRAM_BINS_DEFAULT,
  canonicalHistogram,
  type HistogramContract,
} from "../distribution";
import { getCurveData, listZones, plotBindingSnapshotForChannels, type PlotChannelBinding, type TrackCurveSeries, type WellSummary, type ZoneEntry } from "../ipc";
import { recordProcess } from "../processLog";
import { appState, clearBrush, setBrushedDepths, type BrushSelection } from "../state";
import { buildPersistedPlotState, buildZoneSelect, curveSelect, loadCurveNames, loadPlotProps, savePlotProps, trySelect, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import { messageNode } from "./safeDom";
import { paperExportRecordFromSvg, paperizeMeasuredSvg, saveSvg } from "./svgExport";
import {
  axisRangeExportRecord,
  formatAxisRangeSummary,
  resolveBoundAxisRange,
  type AxisDisplayRange,
  type PlotAxisRangeExport,
} from "./axisRange";
import { applyPlotRangePolicy, formatPlotRangePolicySummary, type PlotRangePolicyReport } from "./plotRangePolicy";
import { registerPlotInvalidationContract } from "./plotInvalidation";
import { applyPlotChannelPolicy, type PlotRangeEdge } from "./plotTypes";
import {
  basicStats,
  buildPlotStatisticsRecord,
  formatPlotStatisticsRecord,
  plotStatisticsInterval,
  type PlotStatisticsRecord,
} from "./plotCanvas";

export type ChartType = "scatter" | "line" | "histogram" | "density" | "raincloud";

/** Preserve only interactive X/Y domains across a theme re-embed; colour is not a viewport. */
export function captureVegaViewportDomains(
  ranges: PlotAxisRangeExport[],
): Partial<Record<"x" | "y", [number, number]>> {
  const domains: Partial<Record<"x" | "y", [number, number]>> = {};
  for (const range of ranges) {
    if (range.axis === "x" || range.axis === "y") domains[range.axis] = [range.min, range.max];
  }
  return domains;
}

export interface Row {
  x: number;
  y?: number;
  z?: number;
  /** Display-only Z after SB-PLT-013 endpoint clamping. Raw `z` remains in the tooltip. */
  zDisplay?: number;
  /** Endpoint marker for a clamped Z value. */
  zEdge?: PlotRangeEdge;
  depth: number;
  /** Raincloud only: the categorical lane this sample belongs to (a zone or a class value). */
  group?: string;
}

export interface VegaHistogramData extends HistogramContract {
  rows: { binStart: number; binEnd: number; count: number }[];
}

/** Pre-bin Vega data through the same contract as Canvas plots; Vega renders these rows verbatim. */
export function buildVegaHistogramData(
  values: ArrayLike<number>,
  min: number,
  max: number,
  bins = HISTOGRAM_BINS_DEFAULT,
): VegaHistogramData {
  const contract = canonicalHistogram(values, min, max, bins);
  return {
    ...contract,
    rows: contract.counts.map((count, index) => ({
      binStart: contract.edges[index],
      binEnd: contract.edges[index + 1],
      count,
    })),
  };
}

/** Options threaded through buildSpec: the scatter trend overlay, and the raincloud group ordering. */
interface SpecOpts {
  trend?: boolean;
  method?: string;
  /** Raincloud lane order (zones run structurally top→bottom; class values come pre-sorted). */
  groupOrder?: string[];
  /** Raincloud: what the grouping represents ("zone" or a curve mnemonic) — for tooltips. */
  groupLabel?: string;
  /** Governed source-curve domains. Aggregate/synthetic axes are read back from Vega after render. */
  axisDomains?: Partial<Record<"x" | "y" | "colour", [number, number]>>;
}

/** Types with per-sample point marks that take part in linked brushing (emit + consume). Histogram,
 *  density and raincloud are aggregates/distributions, so they neither publish nor reflect a shared
 *  selection. */
const brushable = (t: ChartType): boolean => t === "scatter" || t === "line";

/** One CSS custom property off :root, with a fallback so the spec never carries an empty string. */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

const dKey = (d: number): number => Math.round(d * 1000); // mm resolution — depths are in metres

/** Join X/Y (and optional Z) curve series by depth while retaining incomplete pairs until the
 * governed population screen counts and excludes them. */
function joinXYZ(series: TrackCurveSeries[], xName: string, yName: string, zName: string | null): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  const ys = series.find((s) => s.curve_name === yName);
  if (!xs || !ys) return [];
  const zs = zName ? (series.find((s) => s.curve_name === zName) ?? null) : null;
  const xByD = new Map<number, { value: number; depth: number }>();
  for (let i = 0; i < xs.depth.length; i++) xByD.set(dKey(xs.depth[i]), { value: xs.value[i], depth: xs.depth[i] });
  const yByD = new Map<number, { value: number; depth: number }>();
  for (let i = 0; i < ys.depth.length; i++) yByD.set(dKey(ys.depth[i]), { value: ys.value[i], depth: ys.depth[i] });
  const zByD = zs ? new Map<number, number>() : null;
  if (zs) for (let i = 0; i < zs.depth.length; i++) if (Number.isFinite(zs.value[i])) zByD!.set(dKey(zs.depth[i]), zs.value[i]);
  const out: Row[] = [];
  const depthKeys = [...new Set([...xByD.keys(), ...yByD.keys()])].sort((a, b) => a - b);
  for (const key of depthKeys) {
    const x = xByD.get(key);
    const y = yByD.get(key);
    const depth = x?.depth ?? y!.depth;
    const row: Row = { x: x?.value ?? Number.NaN, y: y?.value ?? Number.NaN, depth };
    if (zByD) {
      const zv = zByD.get(key);
      if (zv !== undefined) row.z = zv;
    }
    out.push(row);
  }
  return out;
}

/** Raw X samples for a distribution view; the governed screen owns exclusion accounting. */
function xValues(series: TrackCurveSeries[], xName: string): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  if (!xs) return [];
  const out: Row[] = [];
  // Preserve raw non-finite samples until screenVegaPopulation counts and excludes them.
  for (let i = 0; i < xs.depth.length; i++) out.push({ x: xs.value[i], depth: xs.depth[i] });
  return out;
}

interface VegaValidityPolicy {
  apply: boolean;
  x: AxisDisplayRange | null;
  y: AxisDisplayRange | null;
}

export function screenVegaPopulation(
  rows: readonly Row[],
  type: ChartType,
  policy: VegaValidityPolicy,
  xDisplay: AxisDisplayRange | null,
  yDisplay: AxisDisplayRange | null,
): PlotRangePolicyReport {
  const usesY = type === "scatter" || type === "line" || type === "density";
  return applyPlotRangePolicy([
    { values: rows.map((row) => row.x), display: xDisplay, validity: policy.x },
    ...(usesY ? [{
      values: rows.map((row) => row.y ?? Number.NaN),
      display: yDisplay,
      validity: policy.y,
    }] : []),
  ], policy.apply);
}

export interface VegaColourPolicy {
  rows: Row[];
  included: Uint8Array;
  nonFiniteExcluded: number;
  logDomainExcluded: number;
  excluded: number;
  clamped: number;
}

/** SB-PLT-013's generated-Vega Z adapter. It carries both the raw tooltip value and a
 * derived display value, so endpoint colour/edge treatment can never rewrite source data. */
export function applyVegaColourPolicy(
  rows: readonly Row[],
  display: AxisDisplayRange,
  logAxis = false,
): VegaColourPolicy {
  const policy = applyPlotChannelPolicy(
    Float32Array.from(rows, (row) => row.z ?? Number.NaN),
    "colour",
    display,
    logAxis,
  );
  return {
    rows: rows.map((row, index) => ({
      ...row,
      zDisplay: policy.included[index] ? policy.values[index] : undefined,
      zEdge: policy.included[index] ? policy.edgeMarks[index] : "none",
    })),
    included: policy.included,
    nonFiniteExcluded: policy.nonFiniteExcluded,
    logDomainExcluded: policy.logDomainExcluded,
    excluded: policy.nonFiniteExcluded + policy.logDomainExcluded,
    clamped: policy.clamped,
  };
}

/** Live Vega adapter: the generated grammar and every export use the same complete records. */
export function buildVegaStatisticsRecords(
  rows: readonly Row[],
  type: ChartType,
  policy: VegaValidityPolicy,
  wellId: string,
  intervalLow: number | null,
  intervalHigh: number | null,
  xName: string,
  yName: string,
  xDisplay: AxisDisplayRange | null,
  yDisplay: AxisDisplayRange | null,
  selectionLabel = "all eligible",
  unpairedOrUnclassifiedExcluded = 0,
): PlotStatisticsRecord[] {
  const classifiedRows = type === "raincloud"
    ? rows.filter((row) => typeof row.group === "string" && row.group.trim() !== "")
    : [...rows];
  const totalUnpairedOrUnclassifiedExcluded = unpairedOrUnclassifiedExcluded
    + (rows.length - classifiedRows.length);
  const screened = screenVegaPopulation(classifiedRows, type, policy, xDisplay, yDisplay);
  if (screened.analysisCount === 0) return [];
  const context = {
    binding_channel: "x",
    population: "active_well" as const,
    well_ids: [wellId],
    interval: plotStatisticsInterval(intervalLow, intervalHigh),
    selection: { kind: "all_eligible" as const, selection_id: null, label: selectionLabel, applied: false },
    policy: screened,
    selection_excluded: 0,
    unpaired_or_unclassified_excluded: totalUnpairedOrUnclassifiedExcluded,
    standard_deviation: "sample_n_minus_one" as const,
  };
  if (type === "raincloud") {
    const byGroup = new Map<string, number[]>();
    for (const index of screened.indices) {
      const row = classifiedRows[index];
      const group = row.group!;
      const values = byGroup.get(group) ?? [];
      values.push(row.x);
      byGroup.set(group, values);
    }
    return [...byGroup.entries()].map(([group, values], groupIndex) => {
      const groupDisplayHidden = applyPlotRangePolicy([
        { values, display: xDisplay, validity: null },
      ], false).displayHidden;
      return buildPlotStatisticsRecord(values, {
        ...context,
        channel: `x:${xName}:group:${groupIndex}`,
        selection: {
          kind: "named",
          selection_id: `raincloud-group:${group}`,
          label: `group ${group}; ${selectionLabel}`,
          applied: true,
        },
        policy: { ...screened, displayHidden: groupDisplayHidden },
        selection_excluded: screened.analysisCount - values.length,
      });
    });
  }
  const channels: { bindingChannel: string; channel: string; values: number[] }[] = [
    { bindingChannel: "x", channel: `x:${xName}`, values: screened.indices.map((index) => classifiedRows[index].x) },
  ];
  if (type === "scatter" || type === "line" || type === "density") {
    channels.push({
      bindingChannel: "y",
      channel: `y:${yName}`,
      values: screened.indices.map((index) => classifiedRows[index].y as number),
    });
  }
  return channels.map(({ bindingChannel, channel, values }) => buildPlotStatisticsRecord(values, {
    ...context,
    binding_channel: bindingChannel,
    channel,
  }));
}

// ---- Raincloud (PtitPrince-style) geometry -------------------------------------------------
// A raincloud stacks, per group: a jittered strip of raw points (rain, bottom), a boxplot
// (middle), and a half-violin KDE (cloud, top). Vega-Lite has no native violin, and its density /
// boxplot / facet paths fight the panel's `container` sizing, so the geometry is computed here and
// drawn with trivial single-view marks (area / bar / rule / point). Everything shares one synthetic
// quantitative y where group `gi` sits on a lane at gy = gi * LANE:
//   cloud [gy, gy+CLOUD_H]   box [gy+BOX_LO, gy+BOX_HI]   rain [gy+RAIN_LO, gy+RAIN_HI]
const LANE = 2,
  CLOUD_H = 0.9,
  BOX_HI = -0.1,
  BOX_LO = -0.3,
  RAIN_HI = -0.38,
  RAIN_LO = -0.95;
const MAX_GROUPS = 24;

function quantileSorted(s: number[], p: number): number {
  const n = s.length;
  if (n === 0) return NaN;
  if (n === 1) return s[0];
  const idx = p * (n - 1),
    lo = Math.floor(idx),
    hi = Math.ceil(idx);
  return s[lo] + (s[hi] - s[lo]) * (idx - lo);
}
function stddev(values: number[]): number {
  const n = values.length;
  if (n < 2) return 0;
  const m = values.reduce((a, b) => a + b, 0) / n;
  return Math.sqrt(values.reduce((a, b) => a + (b - m) * (b - m), 0) / (n - 1));
}
/** Gaussian KDE on a shared grid; Silverman bandwidth made robust via min(sd, IQR/1.349). */
function kde(values: number[], gridMin: number, gridMax: number, steps: number): { v: number; d: number }[] {
  const n = values.length;
  const s = [...values].sort((a, b) => a - b);
  const iqr = quantileSorted(s, 0.75) - quantileSorted(s, 0.25);
  const spread = Math.min(stddev(values), iqr / 1.349) || stddev(values) || 1e-6;
  const bw = 1.06 * spread * Math.pow(n, -0.2) || (gridMax - gridMin) / 20 || 1e-6;
  const out: { v: number; d: number }[] = [];
  for (let i = 0; i < steps; i++) {
    const v = gridMin + ((gridMax - gridMin) * i) / (steps - 1);
    let d = 0;
    for (const x of values) {
      const u = (v - x) / bw;
      d += Math.exp(-0.5 * u * u);
    }
    out.push({ v, d: d / (n * bw * Math.sqrt(2 * Math.PI)) });
  }
  return out;
}
interface BoxStat {
  group: string;
  q1: number;
  med: number;
  q3: number;
  lo: number;
  hi: number;
  n: number;
  yb0: number;
  yb1: number;
  ymid: number;
}
export function buildVegaBoxStatistics(
  values: ArrayLike<number>,
): { q1: number; med: number; q3: number; lo: number; hi: number; n: number } {
  const stats = basicStats(values, "sample_n_minus_one");
  return {
    q1: stats.p25,
    med: stats.p50,
    q3: stats.p75,
    lo: stats.p5,
    hi: stats.p95,
    n: stats.count,
  };
}
interface Raincloud {
  cloud: { group: string; x: number; yTop: number; yBase: number }[];
  box: BoxStat[];
  rain: { group: string; x: number; y: number; depth: number }[];
  labels: { group: string; x: number; y: number }[];
  yMin: number;
  yMax: number;
  xMin: number;
  xMax: number;
}
/** Turn grouped samples into the six mark datasets. `groupOrder` fixes the lane order; groups with
 *  no rows are dropped. Cloud density is normalised per group (each peaks at CLOUD_H), so shapes are
 *  comparable even when group counts differ — the standard raincloud/violin convention. */
function buildRaincloud(rows: Row[], groupOrder: string[]): Raincloud {
  const byG = new Map<string, Row[]>();
  for (const r of rows) {
    const g = r.group ?? "";
    if (!byG.has(g)) byG.set(g, []);
    byG.get(g)!.push(r);
  }
  const groups = groupOrder.filter((g) => byG.has(g));
  const allX = rows.map((r) => r.x);
  const xMin = allX.length ? Math.min(...allX) : 0,
    xMax = allX.length ? Math.max(...allX) : 1;
  const pad = (xMax - xMin) * 0.02 || 1;
  const cloud: Raincloud["cloud"] = [],
    box: BoxStat[] = [],
    rain: Raincloud["rain"] = [],
    labels: Raincloud["labels"] = [];
  groups.forEach((g, gi) => {
    const gy = gi * LANE;
    const rs = byG.get(g)!;
    const vals = rs.map((r) => r.x);
    const dens = kde(vals, xMin - pad, xMax + pad, 64);
    const maxD = Math.max(...dens.map((p) => p.d)) || 1;
    for (const p of dens) cloud.push({ group: g, x: p.v, yTop: gy + (p.d / maxD) * CLOUD_H, yBase: gy });
    box.push({ group: g, ...buildVegaBoxStatistics(vals), yb0: gy + BOX_LO, yb1: gy + BOX_HI, ymid: gy + (BOX_LO + BOX_HI) / 2 });
    for (const r of rs) rain.push({ group: g, x: r.x, y: gy + RAIN_HI - Math.random() * (RAIN_HI - RAIN_LO), depth: r.depth });
    labels.push({ group: g, x: xMin - pad, y: gy + CLOUD_H * 0.45 });
  });
  return { cloud, box, rain, labels, yMin: -1.0, yMax: (groups.length - 1) * LANE + CLOUD_H + 0.15, xMin: xMin - pad, xMax: xMax + pad };
}

/** Assign each X sample to the zone whose [top, bottom) contains its depth. Samples outside
 *  every zone form an honest "(outside zones)" lane rather than being dropped; with no zones defined
 *  the whole well is one "(all)" lane. Non-finite X stays until the population screen counts it. */
function groupByZone(xs: TrackCurveSeries, zones: ZoneEntry[]): { rows: Row[]; order: string[] } {
  const rows: Row[] = [];
  if (zones.length === 0) {
    for (let i = 0; i < xs.depth.length; i++) rows.push({ x: xs.value[i], group: "(all)", depth: xs.depth[i] });
    return { rows, order: ["(all)"] };
  }
  const sorted = [...zones].sort((a, b) => a.top_depth - b.top_depth);
  const order = sorted.map((z) => z.zone_name);
  let outside = false;
  for (let i = 0; i < xs.depth.length; i++) {
    const v = xs.value[i];
    const d = xs.depth[i];
    const z = sorted.find((zz) => d >= zz.top_depth && d < zz.bottom_depth);
    if (z) rows.push({ x: v, group: z.zone_name, depth: d });
    else {
      rows.push({ x: v, group: "(outside zones)", depth: d });
      outside = true;
    }
  }
  if (outside) order.push("(outside zones)");
  return { rows, order };
}

/** Group each finite X sample by the (rounded) value of a second, categorical curve — rock-type /
 *  facies / RT. Samples with no group value at their depth are counted and reported, not silently
 *  kept. Refuses (returns an error) when the curve resolves to too many classes to be categorical. */
function groupByCurve(
  xs: TrackCurveSeries,
  gs: TrackCurveSeries,
  label: string,
): { rows: Row[]; order: string[]; note: string; excluded: number; error?: string } {
  const gByD = new Map<number, number>();
  for (let i = 0; i < gs.depth.length; i++) if (Number.isFinite(gs.value[i])) gByD.set(dKey(gs.depth[i]), gs.value[i]);
  const rows: Row[] = [];
  const seen = new Set<string>();
  let missing = 0;
  for (let i = 0; i < xs.depth.length; i++) {
    const v = xs.value[i];
    const g = gByD.get(dKey(xs.depth[i]));
    if (g === undefined) {
      missing++;
      continue;
    }
    const key = Number.isInteger(g) ? String(g) : g.toFixed(2);
    rows.push({ x: v, group: key, depth: xs.depth[i] });
    seen.add(key);
  }
  const order = [...seen].sort((a, b) => Number(a) - Number(b) || a.localeCompare(b));
  if (order.length > MAX_GROUPS) {
    return { rows: [], order: [], note: "", excluded: missing, error: `'${label}' has ${order.length} distinct values — pick a categorical curve (rock-type / facies / RT), not a continuous one.` };
  }
  return { rows, order, note: missing > 0 ? ` · ${missing.toLocaleString()} with no ${label}` : "", excluded: missing };
}

/** A themed Vega-Lite spec for one chart type. Colours are pulled from the active theme's CSS vars
 *  at build time; a theme switch re-embeds from the cached rows (see repaint). Scatter/line carry
 *  three params: `grid` (scales-bound, Shift-drag pan + wheel zoom), `brush` (plain-drag interval
 *  whose extent we publish as the shared selection), and the `brushedActive`/`brushedObj` runtime
 *  signals the opacity condition reads to dim non-selected points. `width/height: container` makes
 *  vega track the panel. */
function buildSpec(
  type: ChartType,
  rows: Row[],
  xName: string,
  yName: string,
  zName: string | null,
  opts?: SpecOpts,
): VisualizationSpec {
  const text = cssVar("--text", "#333333");
  const dim = cssVar("--text-dim", "#888888");
  const border = cssVar("--border", "#cccccc");
  const accent = cssVar("--accent", "#b5651d");
  const accent2 = cssVar("--accent-2", "#247a78");
  const warn = cssVar("--warn", "#c0392b");
  const axis = { labelColor: dim, titleColor: text, gridColor: border, domainColor: border, tickColor: border };
  const scale = (channel: "x" | "y", extra: Record<string, unknown> = {}): Record<string, unknown> => ({
    ...extra,
    ...(opts?.axisDomains?.[channel]
      ? { domain: opts.axisDomains[channel], nice: false }
      : {}),
  });
  const base = {
    $schema: "https://vega.github.io/schema/vega-lite/v5.json",
    background: "transparent",
    width: "container",
    height: "container",
    autosize: { type: "fit", contains: "padding", resize: true },
    data: { values: rows },
    config: { background: "transparent", view: { stroke: border }, axis },
  };

  if (type === "histogram") {
    const governedDomain = opts?.axisDomains?.x ?? null;
    const finite = rows.map((row) => row.x).filter(Number.isFinite);
    const derivedDomain = finite.length > 0
      ? [Math.min(...finite), Math.max(...finite)] as [number, number]
      : null;
    const domain = governedDomain ?? derivedDomain;
    const histogram = domain && domain[0] !== domain[1]
      ? buildVegaHistogramData(rows.map((row) => row.x), domain[0], domain[1])
      : { counts: [], edges: [], displayedTotal: 0, nonFiniteExcluded: 0, rows: [] };
    return {
      ...base,
      data: { values: histogram.rows },
      mark: { type: "bar", color: accent, opacity: 0.85 },
      encoding: {
        x: { field: "binStart", type: "quantitative", title: xName, scale: scale("x"), axis },
        x2: { field: "binEnd" },
        y: { field: "count", type: "quantitative", title: `count (displayed n=${histogram.displayedTotal})`, axis },
        tooltip: [
          { field: "binStart", type: "quantitative", title: `${xName} start` },
          { field: "binEnd", type: "quantitative", title: `${xName} end` },
          { field: "count", type: "quantitative", title: "count" },
        ],
      },
    } as VisualizationSpec;
  }

  if (type === "density") {
    // 2D binned heatmap — the density view for clouds too dense to read as a scatter (a Mahakam
    // NPHI–RHOB cloud overplots into a blob; binned counts reveal where the mass actually is).
    const bin = { maxbins: 40 };
    return {
      ...base,
      mark: { type: "rect" },
      encoding: {
        x: { field: "x", bin, type: "quantitative", title: xName, scale: scale("x"), axis },
        y: { field: "y", bin, type: "quantitative", title: yName, scale: scale("y"), axis },
        color: {
          aggregate: "count",
          type: "quantitative",
          title: "count",
          scale: { scheme: "viridis" },
          legend: { labelColor: dim, titleColor: text },
        },
        tooltip: [
          { field: "x", bin, type: "quantitative", title: xName },
          { field: "y", bin, type: "quantitative", title: yName },
          { aggregate: "count", type: "quantitative", title: "count" },
        ],
      },
    } as VisualizationSpec;
  }

  if (type === "raincloud") {
    // Six trivial layers over one synthetic quantitative y (group lanes). x is the value axis,
    // shared across layers; y is hidden (the group labels name each lane). Not brushable — it is a
    // distribution summary, not a per-sample scatter.
    const geo = buildRaincloud(rows, opts?.groupOrder ?? []);
    const yScale = { domain: [geo.yMin, geo.yMax], nice: false };
    const y = (f: string): Record<string, unknown> => ({ field: f, type: "quantitative", scale: yScale, axis: null });
    const gTitle = opts?.groupLabel ?? "group";
    return {
      ...base,
      data: { values: [] }, // each layer carries its own precomputed data; no shared top-level rows
      encoding: {
        x: { type: "quantitative", scale: scale("x", { domain: [geo.xMin, geo.xMax], nice: false }), axis: { title: xName, ...axis } },
      },
      layer: [
        // cloud — half-violin KDE band, filled from the lane baseline up to the density
        {
          data: { values: geo.cloud },
          mark: { type: "area", opacity: 0.4, color: accent },
          encoding: {
            x: { field: "x" },
            y: y("yTop"),
            y2: { field: "yBase" },
            detail: { field: "group" },
            order: { field: "x" },
            tooltip: [{ field: "group", type: "nominal", title: gTitle }],
          },
        },
        // box — inter-quartile range
        {
          data: { values: geo.box },
          mark: { type: "bar", color: dim, opacity: 0.5 },
          encoding: {
            x: { field: "q1", type: "quantitative" },
            x2: { field: "q3" },
            y: y("yb0"),
            y2: { field: "yb1" },
            tooltip: [
              { field: "group", type: "nominal", title: gTitle },
              { field: "med", type: "quantitative", title: "median", format: ".3f" },
              { field: "q1", type: "quantitative", format: ".3f" },
              { field: "q3", type: "quantitative", format: ".3f" },
              { field: "n", type: "quantitative", title: "count" },
            ],
          },
        },
        // box — median rule
        {
          data: { values: geo.box },
          mark: { type: "rule", color: text, strokeWidth: 2 },
          encoding: { x: { field: "med", type: "quantitative" }, y: y("yb0"), y2: { field: "yb1" } },
        },
        // box — governed P5/P95 whiskers, matching the Histogram and SB-PLT-T13
        {
          data: { values: geo.box },
          mark: { type: "rule", color: dim },
          encoding: { x: { field: "lo", type: "quantitative" }, x2: { field: "hi" }, y: y("ymid") },
        },
        // rain — jittered raw samples
        {
          data: { values: geo.rain },
          mark: { type: "point", filled: true, size: 7, opacity: 0.3, color: accent },
          encoding: {
            x: { field: "x", type: "quantitative" },
            y: y("y"),
            tooltip: [
              { field: "group", type: "nominal", title: gTitle },
              { field: "x", type: "quantitative", title: xName, format: ".3f" },
              { field: "depth", type: "quantitative", title: "Depth", format: ".2f" },
            ],
          },
        },
        // group labels, one per lane
        {
          data: { values: geo.labels },
          mark: { type: "text", align: "left", baseline: "middle", dx: 2, color: text, fontSize: 11 },
          encoding: { x: { field: "x", type: "quantitative" }, y: y("y"), text: { field: "group", type: "nominal" } },
        },
      ],
    } as VisualizationSpec;
  }

  const encoding: Record<string, unknown> = {
    x: { field: "x", type: "quantitative", title: xName, scale: scale("x", { zero: false }), axis },
    y: { field: "y", type: "quantitative", title: yName, scale: scale("y", { zero: false }), axis },
    tooltip: [
      { field: "x", type: "quantitative", title: xName, format: ".3f" },
      { field: "y", type: "quantitative", title: yName, format: ".3f" },
      ...(zName ? [{ field: "z", type: "quantitative", title: zName, format: ".3f" }] : []),
      { field: "depth", type: "quantitative", title: "Depth", format: ".2f" },
    ],
  };
  if (zName) {
    encoding.color = {
      field: "zDisplay",
      type: "quantitative",
      title: zName,
      scale: {
        scheme: "viridis",
        ...(opts?.axisDomains?.colour ? { domain: opts.axisDomains.colour } : {}),
      },
      legend: { labelColor: dim, titleColor: text },
    };
    encoding.shape = {
      condition: [{ test: "datum.zEdge !== 'none'", value: "diamond" }],
      value: "circle",
    };
    encoding.stroke = {
      condition: [
        { test: "datum.zEdge === 'low'", value: accent2 },
        { test: "datum.zEdge === 'high'", value: warn },
      ],
      value: null,
    };
    encoding.strokeWidth = {
      condition: [{ test: "datum.zEdge !== 'none'", value: 2 }],
      value: 0,
    };
  }
  if (type === "line") encoding.order = { field: "depth", type: "quantitative" };
  // Scatter dims the un-brushed points to make the shared selection legible: full opacity when
  // nothing is selected, bright inside the brush, faint outside. A line is one path, so per-vertex
  // opacity would just fade the whole stroke — keep it constant and let line brushing only emit.
  const mark =
    type === "line"
      ? { type: "line", strokeWidth: 1.3, opacity: 0.85, point: false, ...(zName ? {} : { color: accent }) }
      : { type: "point", filled: true, size: 20, ...(zName ? {} : { color: accent }) };
  if (type === "scatter") {
    encoding.opacity = {
      condition: [
        { test: "!brushedActive", value: 0.55 },
        { test: "brushedObj[datum.depth] === 1", value: 0.95 },
      ],
      value: 0.08,
    };
  }

  const params = [
    // Scales-bound zoom/pan. Restricted to Shift-drag + wheel so a plain drag is free for the brush.
    {
      name: "grid",
      select: {
        type: "interval",
        on: "[pointerdown[event.shiftKey], pointerup] > pointermove",
        translate: "[pointerdown[event.shiftKey], pointerup] > pointermove!",
        zoom: "wheel!",
      },
      bind: "scales",
    },
    // Plain-drag selection whose extent drives appState.brushedDepths. Fixed once drawn (translate/
    // zoom off) so the wheel zooms the view, not the box; a subtle themed rectangle marks it.
    {
      name: "brush",
      select: {
        type: "interval",
        encodings: ["x", "y"],
        translate: false,
        zoom: false,
        mark: { fill: accent, fillOpacity: 0.08, stroke: accent, strokeOpacity: 0.55, strokeDash: [4, 3] },
      },
    },
    // Runtime signals the opacity condition reads; the panel sets them when a shared brush arrives.
    { name: "brushedActive", value: false },
    { name: "brushedObj", value: {} },
  ];

  if (type === "scatter" && opts?.trend) {
    // Overlay a regression fit (+ its R²) as extra layers over the point cloud. The interactive
    // params stay top-level so brushing / zoom still target the points; the fit line and label share
    // the points' x/y scales. Methods map to Vega-Lite regression: log/exp/pow assume positive data.
    const trendColor = cssVar("--warn", "#c0392b");
    const method = opts.method ?? "linear";
    // Split the params across the layered spec. The selection params (grid/brush) must sit on the
    // POINTS layer so they project onto its x/y encoding — a top-level interval selection has no
    // fields and its scale signals collide ("Duplicate signal name: grid_x"). The variable signals
    // the opacity condition reads (brushedActive/brushedObj) must stay top-level, or that condition
    // can't resolve them ("Unrecognized signal name: brushedActive").
    const selParams = params.filter((p) => "select" in p);
    const valParams = params.filter((p) => !("select" in p));
    return {
      ...base,
      params: valParams,
      layer: [
        { params: selParams, mark, encoding },
        {
          transform: [{ regression: "y", on: "x", method }],
          mark: { type: "line", color: trendColor, strokeWidth: 2 },
          encoding: {
            x: { field: "x", type: "quantitative" },
            y: { field: "y", type: "quantitative" },
          },
        },
        {
          transform: [
            { regression: "y", on: "x", method, params: true },
            { calculate: "'R² = ' + format(datum.rSquared, '.3f')", as: "label" },
          ],
          mark: { type: "text", align: "left", baseline: "top", dx: 6, dy: 6, color: trendColor, fontSize: 11 },
          encoding: {
            x: { value: 6 },
            y: { value: 6 },
            text: { field: "label", type: "nominal" },
          },
        },
      ],
    } as VisualizationSpec;
  }

  return { ...base, params, mark, encoding } as VisualizationSpec;
}

/** A labelled control for the control bar. */
function field(label: string, sel: HTMLElement): HTMLElement {
  const l = document.createElement("label");
  l.className = "vega-field";
  const t = document.createElement("span");
  t.textContent = label;
  l.append(t, sel);
  return l;
}

/** Curve select with a leading "— None —" (empty value) for the optional colour channel. */
function colorSelect(names: string[], initial: string): HTMLSelectElement {
  const sel = document.createElement("select");
  sel.className = "form-control";
  const none = document.createElement("option");
  none.value = "";
  none.textContent = "— None —";
  sel.appendChild(none);
  for (const n of names) {
    const o = document.createElement("option");
    o.value = n;
    o.textContent = n;
    sel.appendChild(o);
  }
  sel.value = names.includes(initial) ? initial : "";
  return sel;
}

/** Rebuild a curve <select>'s options from a fresh catalog after a data-version bump, keeping the
 *  current selection. `lead` are the fixed non-curve options that must survive the rebuild (the
 *  colour channel's "— None —", the raincloud group's "By zone"). A selected curve that has since
 *  vanished from the catalog is re-added as a leading option so the axis never silently jumps to a
 *  different curve. */
function refillCurveSelect(
  sel: HTMLSelectElement,
  names: string[],
  lead: { value: string; label: string }[] = [],
): void {
  const current = sel.value;
  const opt = (value: string, label: string): void => {
    const o = document.createElement("option");
    o.value = value;
    o.textContent = label;
    sel.appendChild(o);
  };
  sel.replaceChildren();
  for (const l of lead) opt(l.value, l.label);
  const leadValues = new Set(lead.map((l) => l.value));
  if (current && !leadValues.has(current) && !names.includes(current)) opt(current, current);
  for (const n of names) opt(n, n);
  sel.value = current;
}

/** Build the well-bound Vega chart panel. Signature matches the other plot builders
 *  (`workspace.createPlot`), returning `{ el, dispose, getState }`. */
export async function buildVegaContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  // Seed defaults from the last-used Vega props, overridden by an explicit `initial` (a well-switch
  // rebuild via getState) so a re-selected well keeps its exact settings.
  const saved = await loadPlotProps<Record<string, string>>("vega");
  const seed = { ...saved, ...(initial ?? {}) };
  const zoneSel = await buildZoneSelect(well, { followSelectedInterval: false });
  trySelect(zoneSel.select, seed.zone);

  const container = document.createElement("div");
  container.className = "plot-content vega-panel";

  const typeSel = document.createElement("select");
  typeSel.className = "form-control";
  for (const [v, label] of [
    ["scatter", "Scatter"],
    ["line", "Line"],
    ["histogram", "Histogram"],
    ["density", "Density"],
    ["raincloud", "Raincloud"],
  ] as [ChartType, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    typeSel.appendChild(o);
  }
  typeSel.value = ["scatter", "line", "histogram", "density", "raincloud"].includes(seed.type ?? "") ? (seed.type as ChartType) : "scatter";

  const xSel = curveSelect(curveNames, seed.x ?? "NPHI");
  const ySel = curveSelect(curveNames, seed.y ?? "RHOB");
  const zSel = colorSelect(curveNames, seed.z ?? "");

  // Raincloud group-by: "By zone" (sentinel) or a categorical curve (rock-type / facies / RT).
  const GROUP_ZONE = "__zone__";
  const groupSel = document.createElement("select");
  groupSel.className = "form-control";
  const zoneGroupOpt = document.createElement("option");
  zoneGroupOpt.value = GROUP_ZONE;
  zoneGroupOpt.textContent = "By zone";
  groupSel.appendChild(zoneGroupOpt);
  for (const name of curveNames) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = name;
    groupSel.appendChild(o);
  }
  groupSel.value = seed.group === GROUP_ZONE || curveNames.includes(seed.group ?? "") ? (seed.group as string) : GROUP_ZONE;

  // V5: a regression trend overlay (scatter only) — a fit line + R² on top of the cloud.
  const trendMethods = ["linear", "log", "exp", "pow", "quad"];
  const trendChk = document.createElement("input");
  trendChk.type = "checkbox";
  trendChk.checked = seed.trend === "1";
  const trendMethodSel = document.createElement("select");
  trendMethodSel.className = "form-control";
  for (const m of trendMethods) {
    const o = document.createElement("option");
    o.value = m;
    o.textContent = m;
    trendMethodSel.appendChild(o);
  }
  trendMethodSel.value = trendMethods.includes(seed.trendMethod ?? "") ? (seed.trendMethod as string) : "linear";
  const trendField = document.createElement("label");
  trendField.className = "vega-field";
  const trendLabelText = document.createElement("span");
  trendLabelText.textContent = "Trend";
  trendField.append(trendLabelText, trendChk, trendMethodSel);

  const validityNumber = (value: string | undefined, placeholder: string): HTMLInputElement => {
    const input = document.createElement("input");
    input.className = "form-control";
    input.type = "number";
    input.step = "any";
    input.placeholder = placeholder;
    const parsed = value === undefined || value.trim() === "" ? null : Number(value);
    input.value = parsed !== null && Number.isFinite(parsed) ? String(parsed) : "";
    input.style.width = "68px";
    return input;
  };
  const xValidMin = validityNumber(seed.xValidMin, "min");
  const xValidMax = validityNumber(seed.xValidMax, "max");
  const yValidMin = validityNumber(seed.yValidMin, "min");
  const yValidMax = validityNumber(seed.yValidMax, "max");
  const rangeControl = (low: HTMLInputElement, high: HTMLInputElement): HTMLElement => {
    const wrap = document.createElement("span");
    wrap.style.display = "flex";
    wrap.style.gap = "4px";
    wrap.append(low, high);
    return wrap;
  };
  const xValidityField = field("X valid", rangeControl(xValidMin, xValidMax));
  const yValidityField = field("Y valid", rangeControl(yValidMin, yValidMax));
  const validityChk = document.createElement("input");
  validityChk.type = "checkbox";
  validityChk.checked = seed.validity === "1";
  const validityField = document.createElement("label");
  validityField.className = "vega-field";
  const validityLabel = document.createElement("span");
  validityLabel.textContent = "Validity";
  validityField.append(validityLabel, validityChk);
  const parsedRange = (low: HTMLInputElement, high: HTMLInputElement): AxisDisplayRange | null => {
    const minimum = low.value.trim() === "" ? null : Number(low.value);
    const maximum = high.value.trim() === "" ? null : Number(high.value);
    return minimum !== null && maximum !== null
      && Number.isFinite(minimum) && Number.isFinite(maximum) && minimum !== maximum
      ? { min: minimum, max: maximum }
      : null;
  };
  const currentValidityPolicy = (): VegaValidityPolicy => ({
    apply: validityChk.checked,
    x: parsedRange(xValidMin, xValidMax),
    y: parsedRange(yValidMin, yValidMax),
  });

  const toolbar = document.createElement("div");
  toolbar.className = "vega-toolbar";
  const yField = field("Y", ySel);
  const zField = field("Color", zSel);
  const groupField = field("Group", groupSel);
  const plotIntents = () => {
    const type = typeSel.value as ChartType;
    if (type === "histogram") {
      return [{ channel: "x", semantic_request: xSel.value, required: true }];
    }
    if (type === "raincloud") {
      return [
        { channel: "x", semantic_request: xSel.value, required: true },
        ...(groupSel.value !== GROUP_ZONE && groupSel.value !== xSel.value
          ? [{ channel: "group", semantic_request: groupSel.value, required: true }]
          : []),
      ];
    }
    return [
      { channel: "x", semantic_request: xSel.value, required: true },
      { channel: "y", semantic_request: ySel.value, required: true },
      ...(type === "scatter" && zSel.value
        ? [{ channel: "colour", semantic_request: zSel.value, required: true }]
        : []),
    ];
  };
  const selectionState = (): Record<string, string> => ({
    type: typeSel.value,
    x: xSel.value,
    y: ySel.value,
    z: zSel.value,
    zone: zoneSel.select.value,
    trend: trendChk.checked ? "1" : "",
    trendMethod: trendMethodSel.value,
    group: groupSel.value,
    validity: validityChk.checked ? "1" : "",
    xValidMin: xValidMin.value,
    xValidMax: xValidMax.value,
    yValidMin: yValidMin.value,
    yValidMax: yValidMax.value,
    histogramBins: typeSel.value === "histogram" ? String(HISTOGRAM_BINS_DEFAULT) : "",
  });
  let baseAxisRanges: PlotAxisRangeExport[] = [];
  let axisRanges: PlotAxisRangeExport[] = [];
  const currentAxisBindings = (): PlotChannelBinding[] =>
    plotBindingSnapshotForChannels([well.well_id], plotIntents());
  const finiteRange = (rows: Row[], field: "x" | "y" | "z"): { min: number; max: number } | null => {
    let min = Infinity;
    let max = -Infinity;
    for (const row of rows) {
      const value = row[field];
      if (value === undefined || !Number.isFinite(value)) continue;
      min = Math.min(min, value);
      max = Math.max(max, value);
    }
    return Number.isFinite(min) && Number.isFinite(max) && min !== max ? { min, max } : null;
  };
  const persistedState = (options: Record<string, unknown>) =>
    (syncRuntimeAxisRanges(), buildPersistedPlotState("vega", options, [well.well_id], plotIntents(), axisRanges));
  toolbar.append(
    field("Type", typeSel),
    field("X", xSel),
    yField,
    zField,
    trendField,
    groupField,
    field("Zone", zoneSel.select),
    validityField,
    xValidityField,
    yValidityField,
  );

  const chartHost = document.createElement("div");
  chartHost.className = "vega-chart-host";
  const rangeInfo = document.createElement("p");
  rangeInfo.className = "modal-hint";
  const statisticsInfo = document.createElement("details");
  statisticsInfo.className = "modal-hint";
  statisticsInfo.hidden = true;
  const statisticsHeading = document.createElement("summary");
  const statisticsBody = document.createElement("pre");
  statisticsBody.style.whiteSpace = "pre-wrap";
  statisticsBody.style.margin = "6px 0 0";
  statisticsInfo.append(statisticsHeading, statisticsBody);
  container.append(toolbar, rangeInfo, statisticsInfo, chartHost);

  // Dim the controls that don't apply to the active type so the toolbar reads honestly: Y is
  // irrelevant to a histogram; colour and the trend overlay are meaningful only on a scatter.
  const syncControls = (): void => {
    const t = typeSel.value as ChartType;
    const isRC = t === "raincloud";
    const usesY = t !== "histogram" && !isRC;
    // Raincloud uses X as the distribution variable and the Group picker; Y / Colour / Trend don't
    // apply. Histogram also has no Y. Group applies only to raincloud.
    ySel.disabled = t === "histogram" || isRC;
    zSel.disabled = t !== "scatter";
    const trendable = t === "scatter";
    trendChk.disabled = !trendable;
    trendMethodSel.disabled = !trendable || !trendChk.checked;
    groupSel.disabled = !isRC;
    yValidMin.disabled = !usesY;
    yValidMax.disabled = !usesY;
    yField.classList.toggle("vega-field-off", ySel.disabled);
    zField.classList.toggle("vega-field-off", zSel.disabled);
    trendField.classList.toggle("vega-field-off", !trendable);
    groupField.classList.toggle("vega-field-off", !isRC);
    yValidityField.classList.toggle("vega-field-off", !usesY);
  };
  syncControls();
  {
    const policy = currentValidityPolicy();
    const type = typeSel.value as ChartType;
    const usesY = type === "scatter" || type === "line" || type === "density";
    if (policy.apply && !policy.x && !(usesY && policy.y)) validityChk.checked = false;
  }

  let current: VegaResult | null = null;
  let disposed = false;
  let gen = 0;
  let settleBindingReady!: () => void;
  let refuseBindingReady!: (error: unknown) => void;
  let bindingReadySettled = false;
  const bindingReady = new Promise<void>((resolve, reject) => {
    settleBindingReady = resolve;
    refuseBindingReady = reject;
  });
  void bindingReady.catch(() => {});
  const bindingReadyOk = (): void => {
    if (bindingReadySettled) return;
    bindingReadySettled = true;
    settleBindingReady();
  };
  const bindingReadyError = (error: unknown): void => {
    if (bindingReadySettled) return;
    bindingReadySettled = true;
    refuseBindingReady(error);
  };
  // True once the panel has been measured non-zero and the first embed has fired. Declared up here
  // (not by the ResizeObserver) because themeVersion.subscribe below fires synchronously on
  // subscribe and reads it — a `let` declared later would be in the temporal dead zone.
  let embedded = false;
  // Cache of the last-rendered view so a theme switch can re-embed without re-fetching.
  let lastRows: Row[] | null = null;
  let lastSourceRows: Row[] | null = null;
  let lastType: ChartType = "scatter";
  let lastX = "";
  let lastY = "";
  let lastZ: string | null = null;
  let lastTrend = false;
  let lastMethod = "linear";
  let lastGroupOrder: string[] = [];
  let lastGroupLabel = "";
  let lastUnpairedOrUnclassifiedExcluded = 0;
  let lastColourPolicy: Pick<VegaColourPolicy, "excluded" | "clamped"> | null = null;
  let statisticsRecords: PlotStatisticsRecord[] = [];
  // V4: an optional hand-edited spec. When set it replaces the generated grammar (the current rows
  // are injected as its data); a chart-type change clears it since the grammar is type-specific.
  let specOverride: VisualizationSpec | null = null;
  const updateRangeInfo = (): void => {
    if (specOverride) {
      statisticsRecords = [];
      statisticsInfo.hidden = true;
      statisticsBody.textContent = "";
      rangeInfo.textContent = "Custom spec active: governed display clipping is unavailable, so persistence and export are refused.";
      return;
    }
    if (axisRanges.length === 0 || !lastSourceRows) {
      statisticsRecords = [];
      statisticsInfo.hidden = true;
      statisticsBody.textContent = "";
      return;
    }
    const x = axisRanges.find((range) => range.axis === "x") ?? null;
    const y = axisRanges.find((range) => range.axis === "y") ?? null;
    const population = screenVegaPopulation(
      lastSourceRows,
      lastType,
      currentValidityPolicy(),
      x ? { min: x.min, max: x.max } : null,
      y ? { min: y.min, max: y.max } : null,
    );
    const zone = zoneSel.current();
    const brush = appState.brushedDepths.get();
    const selectionLabel = brush && brush.wellId === well.well_id
      ? "all eligible; current brush not applied"
      : "all eligible";
    const previousStatisticsCount = statisticsRecords.length;
    statisticsRecords = buildVegaStatisticsRecords(
      lastSourceRows,
      lastType,
      currentValidityPolicy(),
      well.well_id,
      zone.depthMin,
      zone.depthMax,
      lastX,
      lastY,
      x ? { min: x.min, max: x.max } : null,
      y ? { min: y.min, max: y.max } : null,
      selectionLabel,
      lastUnpairedOrUnclassifiedExcluded,
    );
    statisticsInfo.hidden = statisticsRecords.length === 0;
    statisticsHeading.textContent = `Statistics custody — ${statisticsRecords.length} record${statisticsRecords.length === 1 ? "" : "s"}`;
    statisticsBody.textContent = statisticsRecords
      .map((record) => formatPlotStatisticsRecord(record))
      .join("\n");
    if (statisticsRecords.length !== previousStatisticsCount) {
      statisticsInfo.open = statisticsRecords.length <= 2;
    }
    const histogramDomain = lastType === "histogram"
      ? baseAxisRanges.find((range) => range.axis === "x") ?? null
      : null;
    const histogram = histogramDomain && lastRows
      ? buildVegaHistogramData(
          lastRows.map((row) => row.x),
          histogramDomain.min,
          histogramDomain.max,
          HISTOGRAM_BINS_DEFAULT,
        )
      : null;
    const histogramSummary = histogram
      ? ` · histogram bins=${histogram.counts.length} · displayed total=${histogram.displayedTotal}`
      : "";
    const colourSummary = lastZ && lastColourPolicy
      ? `${lastColourPolicy.excluded ? ` · Z excluded=${lastColourPolicy.excluded}` : ""}${lastColourPolicy.clamped ? ` · Z clamped/edge-marked=${lastColourPolicy.clamped}` : ""}`
      : "";
    rangeInfo.textContent = `${formatAxisRangeSummary(axisRanges)} · ${formatPlotRangePolicySummary(population, {
      statistics: true,
      fitInputs: lastType === "scatter" && lastTrend ? population.analysisCount : null,
    })}${histogramSummary}${colourSummary}`;
  };
  const prepareAxisRanges = (type: ChartType, rows: Row[], zName: string | null): boolean => {
    if (specOverride) {
      baseAxisRanges = [];
      axisRanges = [];
      rangeInfo.textContent = "Custom spec active: governed axis custody is unavailable, so persistence and export are refused.";
      return true;
    }
    const bindings = currentAxisBindings();
    const xBinding = bindings.find((binding) => binding.intent.channel === "x") ?? null;
    const yBinding = bindings.find((binding) => binding.intent.channel === "y") ?? null;
    const colourBinding = bindings.find((binding) => binding.intent.channel === "colour") ?? null;
    const xRange = resolveBoundAxisRange({
      binding: xBinding,
      user: null,
      finiteData: finiteRange(rows, "x"),
      validity: currentValidityPolicy().x,
    });
    const needsY = type === "scatter" || type === "line" || type === "density";
    const yRange = needsY
      ? resolveBoundAxisRange({
          binding: yBinding,
          user: null,
          finiteData: finiteRange(rows, "y"),
          validity: currentValidityPolicy().y,
        })
      : null;
    const colourRange = type === "scatter" && zName
      ? resolveBoundAxisRange({
          binding: colourBinding,
          user: null,
          finiteData: finiteRange(rows, "z"),
        })
      : null;
    if (!xRange || (needsY && !yRange) || (type === "scatter" && zName && !colourRange)) {
      baseAxisRanges = [];
      axisRanges = [];
      statisticsRecords = [];
      statisticsInfo.hidden = true;
      rangeInfo.textContent = "Axis range unavailable: this chart has no complete header, audited-family, or finite-data range.";
      return false;
    }
    baseAxisRanges = [
      axisRangeExportRecord("x", xRange),
      ...(yRange ? [axisRangeExportRecord("y", yRange)] : []),
      ...(colourRange ? [axisRangeExportRecord("colour", colourRange)] : []),
    ];
    axisRanges = [...baseAxisRanges];
    updateRangeInfo();
    return true;
  };
  function syncRuntimeAxisRanges(): void {
    if (!current || specOverride || baseAxisRanges.length === 0) return;
    const channels: Array<"x" | "y"> = lastType === "histogram" ? ["x", "y"] : ["x", ...(baseAxisRanges.some((range) => range.axis === "y") ? ["y" as const] : [])];
    const resolved: PlotAxisRangeExport[] = [];
    for (const channel of channels) {
      const base = baseAxisRanges.find((range) => range.axis === channel) ?? null;
      try {
        const runtimeScale = (current.view as unknown as { scale: (name: string) => { domain: () => unknown[] } }).scale(channel);
        const domain = runtimeScale?.domain();
        const min = Number(domain?.[0]);
        const max = Number(domain?.[domain.length - 1]);
        if (!Number.isFinite(min) || !Number.isFinite(max) || min === max) {
          if (base) resolved.push(base);
          continue;
        }
        const unchanged = !!base && base.min === min && base.max === max;
        resolved.push({
          axis: channel,
          min,
          max,
          tier: base ? (unchanged ? base.tier : "user") : "finite_data",
        });
      } catch {
        if (base) resolved.push(base);
      }
    }
    if (resolved.length > 0) {
      const colour = baseAxisRanges.find((range) => range.axis === "colour");
      if (colour) resolved.push(colour);
      axisRanges = resolved;
      updateRangeInfo();
    }
  }
  let axisSyncRaf = 0;
  const syncAxesAfterInteraction = (): void => {
    if (axisSyncRaf) return;
    axisSyncRaf = requestAnimationFrame(() => {
      axisSyncRaf = 0;
      syncRuntimeAxisRanges();
    });
  };
  chartHost.addEventListener("wheel", syncAxesAfterInteraction, { passive: true });
  chartHost.addEventListener("pointerup", syncAxesAfterInteraction);
  const specFor = (
    type: ChartType,
    rows: Row[],
    xName: string,
    yName: string,
    zName: string | null,
    opts: SpecOpts,
  ): VisualizationSpec => {
    if (specOverride) {
      return { ...(specOverride as Record<string, unknown>), data: { values: rows } } as VisualizationSpec;
    }
    return buildSpec(type, rows, xName, yName, zName, {
      ...opts,
      axisDomains: opts.axisDomains
        ?? Object.fromEntries(baseAxisRanges.map((range) => [range.axis, [range.min, range.max]])),
    });
  };

  // --- Linked brushing -------------------------------------------------------
  // Emit: publish the depths inside the Vega brush rectangle. rAF-coalesced during a drag, with a
  // final read on pointer-up so the release position lands even if a frame was pending.
  let localDragging = false;
  let emitRaf = 0;
  const emitBrush = (): void => {
    if (!current || !brushable(lastType) || !lastRows) return;
    let v: { x?: number[]; y?: number[] } | null = null;
    try {
      v = current.view.signal("brush") as { x?: number[]; y?: number[] } | null;
    } catch {
      return;
    }
    const bx = v?.x;
    const by = v?.y;
    if (!Array.isArray(bx) || bx.length < 2 || !Array.isArray(by) || by.length < 2) {
      clearBrush(); // an empty box (a click, or a drag off the data) clears the shared selection
      return;
    }
    const x0 = Math.min(bx[0], bx[1]);
    const x1 = Math.max(bx[0], bx[1]);
    const y0 = Math.min(by[0], by[1]);
    const y1 = Math.max(by[0], by[1]);
    const sel = new Set<number>();
    for (const r of lastRows) {
      if (r.y === undefined) continue;
      if (r.x >= x0 && r.x <= x1 && r.y >= y0 && r.y <= y1) sel.add(r.depth);
    }
    setBrushedDepths(well.well_id, sel);
  };
  const scheduleEmit = (): void => {
    if (emitRaf) return;
    emitRaf = requestAnimationFrame(() => {
      emitRaf = 0;
      emitBrush();
    });
  };

  // Consume: reflect any shared brush (of this well) into the two runtime signals the opacity
  // condition reads. rAF-coalesced so a neighbour panel dragging at frame rate costs one push/frame.
  let applyRaf = 0;
  let pendingSel: BrushSelection | null = null;
  const pushBrush = (sel: BrushSelection | null): void => {
    if (!current || !brushable(lastType)) return;
    const obj: Record<string, number> = {};
    let active = false;
    if (sel && sel.wellId === well.well_id && sel.depths.size) {
      active = true;
      for (const d of sel.depths) obj[String(d)] = 1;
    }
    try {
      current.view.signal("brushedActive", active);
      current.view.signal("brushedObj", obj);
      void current.view.runAsync();
    } catch {
      /* histogram / torn-down view has no such signals — ignore */
    }
  };
  const applyBrush = (sel: BrushSelection | null): void => {
    pendingSel = sel;
    if (applyRaf) return;
    applyRaf = requestAnimationFrame(() => {
      applyRaf = 0;
      pushBrush(pendingSel);
    });
  };

  /** Embed `rows` for `type`, wiring the brush emit listener and syncing the current shared brush.
   *  `myGen` guards against a newer render/repaint having superseded this one mid-await. */
  async function embedRows(
    type: ChartType,
    rows: Row[],
    xName: string,
    yName: string,
    zName: string | null,
    opts: SpecOpts,
    myGen: number,
  ): Promise<void> {
    current?.finalize();
    current = null;
    chartHost.innerHTML = "";
    if (rows.length === 0) {
      statisticsRecords = [];
      statisticsInfo.hidden = true;
      baseAxisRanges = [];
      axisRanges = [];
      const population = screenVegaPopulation(lastSourceRows ?? [], type, currentValidityPolicy(), null, null);
      rangeInfo.textContent = formatPlotRangePolicySummary(population, {
        statistics: true,
        fitInputs: type === "scatter" && !!opts.trend ? population.analysisCount : null,
      });
      const what = type === "histogram" || type === "raincloud" ? xName : `${xName} / ${yName}`;
      const zc = zoneSel.current();
      // `well.well_name` and the curve mnemonics in `what` are LAS-supplied and stored verbatim;
      // building this line as textContent (not innerHTML) keeps a hostile `~W WELL` value inert.
      const scope = zc.zoneName !== "*" ? ` · ${zc.zoneName}` : "";
      chartHost.replaceChildren(
        messageNode("logview-message", `No eligible ${what} samples in ${well.well_name}${scope}; review validity and finite-data exclusions.`),
      );
      setStatus("Vega — no data");
      return;
    }
    lastColourPolicy = null;
    if (!prepareAxisRanges(type, rows, zName)) {
      chartHost.replaceChildren(messageNode("logview-message", "Vega refused: no governed display range is available for every source-curve axis."));
      setStatus("Vega — axis range refused");
      return;
    }
    let renderRows = rows;
    if (type === "scatter" && zName) {
      const colourRange = baseAxisRanges.find((range) => range.axis === "colour");
      if (!colourRange) {
        chartHost.replaceChildren(messageNode("logview-message", "Vega refused: no governed colour range is available for the selected Z curve."));
        setStatus("Vega — colour range refused");
        return;
      }
      const colourPolicy = applyVegaColourPolicy(rows, { min: colourRange.min, max: colourRange.max });
      lastColourPolicy = { excluded: colourPolicy.excluded, clamped: colourPolicy.clamped };
      renderRows = colourPolicy.rows.filter((_row, index) => colourPolicy.included[index] === 1);
    }
    lastRows = renderRows;
    updateRangeInfo();
    try {
      const result = await vegaEmbed(chartHost, specFor(type, renderRows, xName, yName, zName, opts), {
        actions: false,
        renderer: "canvas",
        tooltip: true,
      });
      if (disposed || myGen !== gen) {
        result.finalize();
        return;
      }
      current = result;
      syncRuntimeAxisRanges();
      if (brushable(type)) {
        result.view.addSignalListener("brush", () => scheduleEmit());
        pushBrush(appState.brushedDepths.get()); // reflect any selection already active elsewhere
      }
      const zc = zoneSel.current();
      const scope = zc.zoneName !== "*" ? ` · ${zc.zoneName}` : "";
      setStatus(`Vega — ${type}, ${renderRows.length.toLocaleString()} points${scope}`);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      chartHost.replaceChildren(messageNode("logview-message", `Vega render failed: ${err}`));
      setStatus("Vega — render failed");
    }
  }

  async function render(): Promise<void> {
    const myGen = ++gen;
    const type = typeSel.value as ChartType;
    const xName = xSel.value;
    const yName = ySel.value;
    const useZ = type === "scatter" && zSel.value ? zSel.value : null;
    const useTrend = type === "scatter" && trendChk.checked;
    const method = trendMethodSel.value;
    const zc = zoneSel.current();

    if (type === "raincloud") {
      const groupBy = groupSel.value;
      const byZone = groupBy === GROUP_ZONE;
      const needed = byZone || groupBy === xName ? [xName] : [xName, groupBy];
      setStatus(`Vega — loading ${needed.join(", ")}…`);
      let series: TrackCurveSeries[];
      try {
        series = await getCurveData(well.well_id, needed, zc.depthMin, zc.depthMax);
      } catch (err) {
        if (disposed || myGen !== gen) return;
        bindingReadyError(err);
        chartHost.replaceChildren(messageNode("logview-message", `Failed to load curves: ${err}`));
        setStatus("Vega — load failed");
        return;
      }
      if (disposed || myGen !== gen) return;
      const xs = series.find((s) => s.curve_name === xName);
      if (!xs) {
        bindingReadyError(new Error(`required channel '${xName}' is unresolved`));
        chartHost.replaceChildren(messageNode("logview-message", `No ${xName} data in ${well.well_name}.`));
        setStatus("Vega — no data");
        return;
      }
      let rows: Row[], order: string[], note = "", groupLabel: string, unpairedOrUnclassifiedExcluded = 0;
      if (byZone) {
        let zones: ZoneEntry[] = [];
        try {
          zones = await listZones(well.well_id);
        } catch {
          zones = [];
        }
        if (disposed || myGen !== gen) return;
        ({ rows, order } = groupByZone(xs, zones));
        groupLabel = "zone";
      } else {
        const gs = series.find((s) => s.curve_name === groupBy);
        if (!gs) {
          bindingReadyError(new Error(`required channel '${groupBy}' is unresolved`));
          chartHost.replaceChildren(messageNode("logview-message", `No ${groupBy} data in ${well.well_name}.`));
          setStatus("Vega — no data");
          return;
        }
        const res = groupByCurve(xs, gs, groupBy);
        if (res.error) {
          bindingReadyError(new Error(res.error));
          chartHost.replaceChildren(messageNode("logview-message", res.error));
          setStatus("Vega — too many groups");
          return;
        }
        ({ rows, order, note, excluded: unpairedOrUnclassifiedExcluded } = res);
        groupLabel = groupBy;
      }
      const sourceRows = rows;
      const screened = screenVegaPopulation(sourceRows, type, currentValidityPolicy(), null, null);
      rows = screened.indices.map((index) => sourceRows[index]);
      const presentGroups = new Set(rows.map((row) => row.group));
      order = order.filter((group) => presentGroups.has(group));
      lastSourceRows = sourceRows;
      lastRows = rows;
      lastType = type;
      lastX = xName;
      lastY = "";
      lastZ = null;
      lastTrend = false;
      lastMethod = method;
      lastGroupOrder = order;
      lastGroupLabel = groupLabel;
      lastUnpairedOrUnclassifiedExcluded = unpairedOrUnclassifiedExcluded;
      await embedRows(type, rows, xName, "", null, { groupOrder: order, groupLabel }, myGen);
      if (!current) {
        bindingReadyError(new Error("Vega refused: the initial chart did not render"));
        return;
      }
      bindingReadyOk();
      persist();
      // embedRows sets a generic status; refine it with the group count and any dropped-sample note.
      if (current && !disposed && myGen === gen) {
        setStatus(`Vega — raincloud · ${order.length} group(s) · ${rows.length.toLocaleString()} pts${note}`);
      }
      return;
    }

    const needed = type === "histogram" ? [xName] : useZ ? [xName, yName, useZ] : [xName, yName];
    setStatus(`Vega — loading ${needed.join(", ")}…`);
    let series: TrackCurveSeries[];
    try {
      series = await getCurveData(well.well_id, needed, zc.depthMin, zc.depthMax);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      bindingReadyError(err);
      chartHost.replaceChildren(messageNode("logview-message", `Failed to load curves: ${err}`));
      setStatus("Vega — load failed");
      return;
    }
    if (disposed || myGen !== gen) return; // a newer render (or close) already won
    const sourceRows = type === "histogram" ? xValues(series, xName) : joinXYZ(series, xName, yName, useZ);
    const screened = screenVegaPopulation(sourceRows, type, currentValidityPolicy(), null, null);
    const rows = screened.indices.map((index) => sourceRows[index]);
    lastSourceRows = sourceRows;
    lastRows = rows;
    lastType = type;
    lastX = xName;
    lastY = yName;
    lastZ = useZ;
    lastTrend = useTrend;
    lastMethod = method;
    lastUnpairedOrUnclassifiedExcluded = 0;
    await embedRows(type, rows, xName, yName, useZ, { trend: useTrend, method }, myGen);
    if (!current) {
      bindingReadyError(new Error("Vega refused: the initial chart did not render"));
      return;
    }
    bindingReadyOk();
    persist();
  }

  /** Re-embed cached rows with new theme colours, restoring current runtime X/Y domains. */
  async function repaint(): Promise<void> {
    if (!lastSourceRows) return;
    syncRuntimeAxisRanges();
    const axisDomains = captureVegaViewportDomains(axisRanges);
    const myGen = ++gen;
    const screened = screenVegaPopulation(lastSourceRows, lastType, currentValidityPolicy(), null, null);
    const rows = screened.indices.map((index) => lastSourceRows![index]);
    await embedRows(lastType, rows, lastX, lastY, lastZ, {
      trend: lastTrend,
      method: lastMethod,
      groupOrder: lastGroupOrder,
      groupLabel: lastGroupLabel,
      axisDomains,
    }, myGen);
  }

  // --- V4: last-used persistence, export, spec editor ------------------------
  const persist = (): void => {
    try {
      void savePlotProps("vega", persistedState(selectionState()))
        .catch((error) => setStatus(`Vega state not saved: ${error}`));
    } catch (error) {
      setStatus(`Vega state not saved: ${error}`);
    }
  };

  // Export. Vega renders to a <canvas>, so the shared PNG copy/save/print buttons work against it;
  // SVG comes from vega's own vector renderer.
  const getCanvas = (): HTMLCanvasElement | null => chartHost.querySelector<HTMLCanvasElement>("canvas");
  const exportSvg = async (): Promise<void> => {
    if (!current) {
      setStatus("No Vega chart to export yet");
      return;
    }
    try {
      const state = persistedState(selectionState());
      const scope = {
        wellIds: state.well_ids,
        curves: plotIntents().map((intent) => intent.semantic_request),
        plotBindings: state.bindings,
        axisRanges: state.axis_ranges,
        statisticsRecords,
      };
      const measured = current.view.scenegraph().bounds;
      const svg = paperizeMeasuredSvg(
        await current.view.toSVG(),
        current.view.width(),
        current.view.height(),
        { min_x: measured.x1, min_y: measured.y1, max_x: measured.x2, max_y: measured.y2 },
        scope,
      );
      const path = await saveSvg(svg, "Vega chart", {
        ...scope,
        paperExportRecord: paperExportRecordFromSvg(svg),
      });
      if (path) {
        setStatus(`Vega chart SVG saved to ${path}`);
        recordProcess("Export", `Vega chart SVG (vector) → ${path}`);
      }
    } catch (err) {
      setStatus(`SVG export failed: ${err}`);
    }
  };
  const exportGroup = buildImageExportButtons(
    getCanvas,
    "Vega chart",
    setStatus,
    undefined,
    undefined,
    undefined,
    () => {
      const state = persistedState(selectionState());
      return {
        wellIds: state.well_ids,
        curves: plotIntents().map((intent) => intent.semantic_request),
        plotBindings: state.bindings,
        axisRanges: state.axis_ranges,
        statisticsRecords,
      };
    },
  );
  const svgBtn = document.createElement("button");
  svgBtn.className = "plot-export-btn";
  svgBtn.textContent = "⭳ SVG";
  svgBtn.title = "Export this chart as a true-vector SVG (vega renderer)";
  svgBtn.addEventListener("click", () => void exportSvg());
  exportGroup.appendChild(svgBtn);

  // Spec editor: reveal the effective Vega-Lite spec as JSON and let the user override the grammar.
  const specToggle = document.createElement("button");
  specToggle.className = "plot-export-btn";
  specToggle.textContent = "⧉ Spec";
  specToggle.title = "View / edit the Vega-Lite spec (JSON)";
  specToggle.style.marginLeft = "auto"; // push the action cluster to the right end of the toolbar
  toolbar.append(specToggle, exportGroup);

  const specWrap = document.createElement("div");
  specWrap.className = "vega-spec";
  specWrap.style.display = "none";
  const editorHost = document.createElement("div");
  editorHost.className = "vega-spec-editor";
  const specBar = document.createElement("div");
  specBar.className = "vega-spec-bar";
  const applyBtn = document.createElement("button");
  applyBtn.className = "lp-btn primary";
  applyBtn.textContent = "Apply";
  const resetBtn = document.createElement("button");
  resetBtn.className = "lp-btn";
  resetBtn.textContent = "Reset";
  const specErr = document.createElement("span");
  specErr.className = "vega-spec-err";
  specBar.append(applyBtn, resetBtn, specErr);
  specWrap.append(editorHost, specBar);
  container.append(specWrap);

  // The generated spec as pretty JSON with the (potentially huge) data values elided — the editor
  // shows grammar only; the current rows are re-injected on Apply (see specFor).
  const templateJson = (): string => {
    const type = typeSel.value as ChartType;
    const useZ = type === "scatter" && zSel.value ? zSel.value : null;
    // Raincloud bakes its geometry into per-layer data (there is no shared top-level dataset to
    // elide), so show it built from the live rows; the other types show grammar with data elided.
    const rows = type === "raincloud" ? (lastRows ?? []) : [];
    const spec = buildSpec(type, rows, xSel.value, ySel.value, useZ, {
      trend: type === "scatter" && trendChk.checked,
      method: trendMethodSel.value,
      groupOrder: type === "raincloud" ? lastGroupOrder : undefined,
      groupLabel: type === "raincloud" ? lastGroupLabel : undefined,
    }) as Record<string, unknown>;
    delete spec.data;
    return JSON.stringify(spec, null, 2);
  };
  let editor: EditorView | null = null;
  const refreshTemplate = (): void => {
    if (editor) editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: templateJson() } });
  };
  const ensureEditor = async (): Promise<void> => {
    if (editor || disposed) return;
    const { EditorView: CM, basicSetup } = await import("codemirror");
    if (disposed) return; // the panel closed while the (lazy) editor module loaded
    editor = new CM({ parent: editorHost, doc: templateJson(), extensions: [basicSetup, CM.lineWrapping] });
  };
  let specOpen = false;
  specToggle.addEventListener("click", () => {
    specOpen = !specOpen;
    specWrap.style.display = specOpen ? "" : "none";
    specToggle.classList.toggle("active", specOpen);
    if (specOpen) void ensureEditor().then(() => !specOverride && refreshTemplate());
  });
  applyBtn.addEventListener("click", () => {
    if (!editor) return;
    let parsed: unknown;
    try {
      parsed = JSON.parse(editor.state.doc.toString());
    } catch (e) {
      specErr.textContent = `Invalid JSON: ${e instanceof Error ? e.message : e}`;
      return;
    }
    specErr.textContent = "";
    specOverride = parsed as VisualizationSpec;
    setStatus("Vega — applied spec override");
    void render();
  });
  resetBtn.addEventListener("click", () => {
    specOverride = null;
    specErr.textContent = "";
    refreshTemplate();
    setStatus("Vega — spec override reset");
    void render();
  });

  // Control-bar changes. A chart-type change is structural, so it drops any spec override; the other
  // controls only change which curves/zone fill the plot and keep an override in place.
  const ensureValidityApplicable = (): boolean => {
    if (!validityChk.checked) return true;
    const policy = currentValidityPolicy();
    const type = typeSel.value as ChartType;
    const usesY = type === "scatter" || type === "line" || type === "density";
    if (policy.x || (usesY && policy.y)) return true;
    validityChk.checked = false;
    setStatus("Vega validity disabled: supply at least one complete applicable X or Y range before enabling it");
    return false;
  };
  typeSel.addEventListener("change", () => {
    syncControls();
    ensureValidityApplicable();
    if (specOverride) {
      specOverride = null;
      setStatus("Vega — spec override reset (chart type changed)");
    }
    persist();
    if (specOpen) refreshTemplate();
    void render();
  });
  for (const sel of [xSel, ySel, zSel, groupSel, zoneSel.select]) {
    sel.addEventListener("change", () => {
      persist();
      if (specOpen && !specOverride) refreshTemplate();
      void render();
    });
  }
  // The trend toggle also re-syncs the method select's enabled state.
  const onTrendChange = (): void => {
    syncControls();
    persist();
    if (specOpen && !specOverride) refreshTemplate();
    void render();
  };
  trendChk.addEventListener("change", onTrendChange);
  trendMethodSel.addEventListener("change", onTrendChange);
  const onValidityChange = (): void => {
    ensureValidityApplicable();
    persist();
    void render();
  };
  validityChk.addEventListener("change", onValidityChange);
  for (const input of [xValidMin, xValidMax, yValidMin, yValidMax]) {
    input.addEventListener("change", onValidityChange);
  }

  // A plain (non-Shift) drag on the chart is a brush; remember it so pointer-up can flush the final
  // extent. Shift-drag is a pan (grid param) and must not publish a selection.
  chartHost.addEventListener("pointerdown", (e) => {
    if (!e.shiftKey && brushable(lastType)) localDragging = true;
  });
  const onPointerUp = (): void => {
    if (!localDragging) return;
    localDragging = false;
    if (emitRaf) {
      cancelAnimationFrame(emitRaf);
      emitRaf = 0;
    }
    emitBrush();
  };
  window.addEventListener("pointerup", onPointerUp);

  const invalidation = registerPlotInvalidationContract(chartHost, {
    selection: (selection) => {
      applyBrush(selection);
      updateRangeInfo();
    },
    theme: () => {
      if (embedded && lastRows) void repaint();
    },
    dataRevision: () => {
      // Refill selectors and re-fetch after module/equation runs, imports and undo, so the
      // grammar cannot keep stale rows while sibling plots already show the new revision.
      void (async () => {
        try {
          const names = await loadCurveNames();
          if (disposed) return;
          refillCurveSelect(xSel, names);
          refillCurveSelect(ySel, names);
          refillCurveSelect(zSel, names, [{ value: "", label: "— None —" }]);
          refillCurveSelect(groupSel, names, [{ value: GROUP_ZONE, label: "By zone" }]);
        } catch {
          // Catalog refill failed; still re-render below so the plot reflects the new revision.
        }
        if (!disposed && embedded) await render();
      })();
    },
    interval: (interval) => zoneSel.applySelectedInterval(interval, true),
    size: ({ width, height }) => {
      if (disposed || width <= 0 || height <= 0) return;
      if (!embedded) {
        embedded = true;
        void render();
        return;
      }
      if (current) {
        void current.view.resize().runAsync().then(() => {
          if (!disposed) syncRuntimeAxisRanges();
        }).catch(() => {});
      }
    },
    cancelPending: () => {
      disposed = true;
      gen++;
      if (emitRaf) {
        cancelAnimationFrame(emitRaf);
        emitRaf = 0;
      }
      if (applyRaf) {
        cancelAnimationFrame(applyRaf);
        applyRaf = 0;
      }
      if (axisSyncRaf) {
        cancelAnimationFrame(axisSyncRaf);
        axisSyncRaf = 0;
      }
    },
  });

  // The dock normally attaches after this builder returns, so the shared size source performs
  // the initial render on the first 0→non-zero change. Handle an already-attached host too.
  if (chartHost.clientWidth > 0 && chartHost.clientHeight > 0) {
    embedded = true;
    void render();
  }

  return {
    el: container,
    dispose: () => {
      invalidation.dispose();
      window.removeEventListener("pointerup", onPointerUp);
      chartHost.removeEventListener("wheel", syncAxesAfterInteraction);
      chartHost.removeEventListener("pointerup", syncAxesAfterInteraction);
      zoneSel.dispose();
      editor?.destroy();
      current?.finalize();
      current = null;
    },
    getState: selectionState,
    getPersistedState: () => persistedState(selectionState()),
    bindingReady,
  };
}
