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
import { getCurveData, listZones, type TrackCurveSeries, type WellSummary, type ZoneEntry } from "../ipc";
import { recordProcess } from "../processLog";
import { appState, clearBrush, setBrushedDepths, type BrushSelection } from "../state";
import { buildZoneSelect, curveSelect, loadCurveNames, loadPlotProps, savePlotProps, trySelect, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import { messageNode } from "./safeDom";
import { saveSvg } from "./svgExport";

type ChartType = "scatter" | "line" | "histogram" | "density" | "raincloud";

interface Row {
  x: number;
  y?: number;
  z?: number;
  depth: number;
  /** Raincloud only: the categorical lane this sample belongs to (a zone or a class value). */
  group?: string;
}

/** Options threaded through buildSpec: the scatter trend overlay, and the raincloud group ordering. */
interface SpecOpts {
  trend?: boolean;
  method?: string;
  /** Raincloud lane order (zones run structurally top→bottom; class values come pre-sorted). */
  groupOrder?: string[];
  /** Raincloud: what the grouping represents ("zone" or a curve mnemonic) — for tooltips. */
  groupLabel?: string;
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

/** Join X/Y (and optional Z) curve series on shared depth into finite rows. Curves ride the same
 *  standard grid, but they are joined by depth (not index) so a curve with its own sampling still
 *  lines up; non-finite values are dropped so vega never sees NaN. */
function joinXYZ(series: TrackCurveSeries[], xName: string, yName: string, zName: string | null): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  const ys = series.find((s) => s.curve_name === yName);
  if (!xs || !ys) return [];
  const zs = zName ? (series.find((s) => s.curve_name === zName) ?? null) : null;
  const yByD = new Map<number, number>();
  for (let i = 0; i < ys.depth.length; i++) if (Number.isFinite(ys.value[i])) yByD.set(dKey(ys.depth[i]), ys.value[i]);
  const zByD = zs ? new Map<number, number>() : null;
  if (zs) for (let i = 0; i < zs.depth.length; i++) if (Number.isFinite(zs.value[i])) zByD!.set(dKey(zs.depth[i]), zs.value[i]);
  const out: Row[] = [];
  for (let i = 0; i < xs.depth.length; i++) {
    const xv = xs.value[i];
    if (!Number.isFinite(xv)) continue;
    const yv = yByD.get(dKey(xs.depth[i]));
    if (yv === undefined) continue;
    const row: Row = { x: xv, y: yv, depth: xs.depth[i] };
    if (zByD) {
      const zv = zByD.get(dKey(xs.depth[i]));
      if (zv !== undefined) row.z = zv;
    }
    out.push(row);
  }
  return out;
}

/** Finite X samples only — the data for a distribution (histogram) view. */
function xValues(series: TrackCurveSeries[], xName: string): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  if (!xs) return [];
  const out: Row[] = [];
  for (let i = 0; i < xs.depth.length; i++) if (Number.isFinite(xs.value[i])) out.push({ x: xs.value[i], depth: xs.depth[i] });
  return out;
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
function boxStats(values: number[]): { q1: number; med: number; q3: number; lo: number; hi: number; n: number } {
  const s = [...values].sort((a, b) => a - b);
  const q1 = quantileSorted(s, 0.25),
    med = quantileSorted(s, 0.5),
    q3 = quantileSorted(s, 0.75);
  const iqr = q3 - q1,
    loF = q1 - 1.5 * iqr,
    hiF = q3 + 1.5 * iqr;
  return { q1, med, q3, lo: s.find((v) => v >= loF) ?? s[0], hi: [...s].reverse().find((v) => v <= hiF) ?? s[s.length - 1], n: s.length };
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
    box.push({ group: g, ...boxStats(vals), yb0: gy + BOX_LO, yb1: gy + BOX_HI, ymid: gy + (BOX_LO + BOX_HI) / 2 });
    for (const r of rs) rain.push({ group: g, x: r.x, y: gy + RAIN_HI - Math.random() * (RAIN_HI - RAIN_LO), depth: r.depth });
    labels.push({ group: g, x: xMin - pad, y: gy + CLOUD_H * 0.45 });
  });
  return { cloud, box, rain, labels, yMin: -1.0, yMax: (groups.length - 1) * LANE + CLOUD_H + 0.15, xMin: xMin - pad, xMax: xMax + pad };
}

/** Assign each finite X sample to the zone whose [top, bottom) contains its depth. Samples outside
 *  every zone form an honest "(outside zones)" lane rather than being dropped; with no zones defined
 *  the whole well is one "(all)" lane. */
function groupByZone(xs: TrackCurveSeries, zones: ZoneEntry[]): { rows: Row[]; order: string[] } {
  const rows: Row[] = [];
  if (zones.length === 0) {
    for (let i = 0; i < xs.depth.length; i++) if (Number.isFinite(xs.value[i])) rows.push({ x: xs.value[i], group: "(all)", depth: xs.depth[i] });
    return { rows, order: ["(all)"] };
  }
  const sorted = [...zones].sort((a, b) => a.top_depth - b.top_depth);
  const order = sorted.map((z) => z.zone_name);
  let outside = false;
  for (let i = 0; i < xs.depth.length; i++) {
    const v = xs.value[i];
    if (!Number.isFinite(v)) continue;
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
): { rows: Row[]; order: string[]; note: string; error?: string } {
  const gByD = new Map<number, number>();
  for (let i = 0; i < gs.depth.length; i++) if (Number.isFinite(gs.value[i])) gByD.set(dKey(gs.depth[i]), gs.value[i]);
  const rows: Row[] = [];
  const seen = new Set<string>();
  let missing = 0;
  for (let i = 0; i < xs.depth.length; i++) {
    const v = xs.value[i];
    if (!Number.isFinite(v)) continue;
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
    return { rows: [], order: [], note: "", error: `'${label}' has ${order.length} distinct values — pick a categorical curve (rock-type / facies / RT), not a continuous one.` };
  }
  return { rows, order, note: missing > 0 ? ` · ${missing.toLocaleString()} with no ${label}` : "" };
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
  const axis = { labelColor: dim, titleColor: text, gridColor: border, domainColor: border, tickColor: border };
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
    return {
      ...base,
      mark: { type: "bar", color: accent, opacity: 0.85 },
      encoding: {
        x: { field: "x", bin: true, type: "quantitative", title: xName, axis },
        y: { aggregate: "count", type: "quantitative", title: "count", axis },
        tooltip: [
          { field: "x", bin: true, type: "quantitative", title: xName },
          { aggregate: "count", type: "quantitative", title: "count" },
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
        x: { field: "x", bin, type: "quantitative", title: xName, axis },
        y: { field: "y", bin, type: "quantitative", title: yName, axis },
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
        x: { type: "quantitative", scale: { domain: [geo.xMin, geo.xMax], nice: false }, axis: { title: xName, ...axis } },
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
        // box — whiskers (Tukey 1.5·IQR fences)
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
    x: { field: "x", type: "quantitative", title: xName, scale: { zero: false }, axis },
    y: { field: "y", type: "quantitative", title: yName, scale: { zero: false }, axis },
    tooltip: [
      { field: "x", type: "quantitative", title: xName, format: ".3f" },
      { field: "y", type: "quantitative", title: yName, format: ".3f" },
      ...(zName ? [{ field: "z", type: "quantitative", title: zName, format: ".3f" }] : []),
      { field: "depth", type: "quantitative", title: "Depth", format: ".2f" },
    ],
  };
  if (zName) {
    encoding.color = {
      field: "z",
      type: "quantitative",
      title: zName,
      scale: { scheme: "viridis" },
      legend: { labelColor: dim, titleColor: text },
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

/** A labelled select for the control bar. */
function field(label: string, sel: HTMLSelectElement): HTMLElement {
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
  const zoneSel = await buildZoneSelect(well);
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

  const toolbar = document.createElement("div");
  toolbar.className = "vega-toolbar";
  const yField = field("Y", ySel);
  const zField = field("Color", zSel);
  const groupField = field("Group", groupSel);
  toolbar.append(field("Type", typeSel), field("X", xSel), yField, zField, trendField, groupField, field("Zone", zoneSel.select));

  const chartHost = document.createElement("div");
  chartHost.className = "vega-chart-host";
  container.append(toolbar, chartHost);

  // Dim the controls that don't apply to the active type so the toolbar reads honestly: Y is
  // irrelevant to a histogram; colour and the trend overlay are meaningful only on a scatter.
  const syncControls = (): void => {
    const t = typeSel.value as ChartType;
    const isRC = t === "raincloud";
    // Raincloud uses X as the distribution variable and the Group picker; Y / Colour / Trend don't
    // apply. Histogram also has no Y. Group applies only to raincloud.
    ySel.disabled = t === "histogram" || isRC;
    zSel.disabled = t !== "scatter";
    const trendable = t === "scatter";
    trendChk.disabled = !trendable;
    trendMethodSel.disabled = !trendable || !trendChk.checked;
    groupSel.disabled = !isRC;
    yField.classList.toggle("vega-field-off", ySel.disabled);
    zField.classList.toggle("vega-field-off", zSel.disabled);
    trendField.classList.toggle("vega-field-off", !trendable);
    groupField.classList.toggle("vega-field-off", !isRC);
  };
  syncControls();

  let current: VegaResult | null = null;
  let disposed = false;
  let gen = 0;
  // True once the panel has been measured non-zero and the first embed has fired. Declared up here
  // (not by the ResizeObserver) because themeVersion.subscribe below fires synchronously on
  // subscribe and reads it — a `let` declared later would be in the temporal dead zone.
  let embedded = false;
  // Cache of the last-rendered view so a theme switch can re-embed without re-fetching.
  let lastRows: Row[] | null = null;
  let lastType: ChartType = "scatter";
  let lastX = "";
  let lastY = "";
  let lastZ: string | null = null;
  let lastTrend = false;
  let lastMethod = "linear";
  let lastGroupOrder: string[] = [];
  let lastGroupLabel = "";
  // V4: an optional hand-edited spec. When set it replaces the generated grammar (the current rows
  // are injected as its data); a chart-type change clears it since the grammar is type-specific.
  let specOverride: VisualizationSpec | null = null;
  const specFor = (
    type: ChartType,
    rows: Row[],
    xName: string,
    yName: string,
    zName: string | null,
    opts: SpecOpts,
  ): VisualizationSpec =>
    specOverride
      ? ({ ...(specOverride as Record<string, unknown>), data: { values: rows } } as VisualizationSpec)
      : buildSpec(type, rows, xName, yName, zName, opts);

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
      const what = type === "histogram" || type === "raincloud" ? xName : `${xName} / ${yName}`;
      const zc = zoneSel.current();
      // `well.well_name` and the curve mnemonics in `what` are LAS-supplied and stored verbatim;
      // building this line as textContent (not innerHTML) keeps a hostile `~W WELL` value inert.
      const scope = zc.zoneName !== "*" ? ` · ${zc.zoneName}` : "";
      chartHost.replaceChildren(
        messageNode("logview-message", `No finite ${what} samples in ${well.well_name}${scope}.`),
      );
      setStatus("Vega — no data");
      return;
    }
    try {
      const result = await vegaEmbed(chartHost, specFor(type, rows, xName, yName, zName, opts), {
        actions: false,
        renderer: "canvas",
        tooltip: true,
      });
      if (disposed || myGen !== gen) {
        result.finalize();
        return;
      }
      current = result;
      if (brushable(type)) {
        result.view.addSignalListener("brush", () => scheduleEmit());
        pushBrush(appState.brushedDepths.get()); // reflect any selection already active elsewhere
      }
      const zc = zoneSel.current();
      const scope = zc.zoneName !== "*" ? ` · ${zc.zoneName}` : "";
      setStatus(`Vega — ${type}, ${rows.length.toLocaleString()} points${scope}`);
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
        chartHost.replaceChildren(messageNode("logview-message", `Failed to load curves: ${err}`));
        setStatus("Vega — load failed");
        return;
      }
      if (disposed || myGen !== gen) return;
      const xs = series.find((s) => s.curve_name === xName);
      if (!xs) {
        chartHost.replaceChildren(messageNode("logview-message", `No ${xName} data in ${well.well_name}.`));
        setStatus("Vega — no data");
        return;
      }
      let rows: Row[], order: string[], note = "", groupLabel: string;
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
          chartHost.replaceChildren(messageNode("logview-message", `No ${groupBy} data in ${well.well_name}.`));
          setStatus("Vega — no data");
          return;
        }
        const res = groupByCurve(xs, gs, groupBy);
        if (res.error) {
          chartHost.replaceChildren(messageNode("logview-message", res.error));
          setStatus("Vega — too many groups");
          return;
        }
        ({ rows, order, note } = res);
        groupLabel = groupBy;
      }
      lastRows = rows;
      lastType = type;
      lastX = xName;
      lastY = "";
      lastZ = null;
      lastTrend = false;
      lastMethod = method;
      lastGroupOrder = order;
      lastGroupLabel = groupLabel;
      await embedRows(type, rows, xName, "", null, { groupOrder: order, groupLabel }, myGen);
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
      chartHost.replaceChildren(messageNode("logview-message", `Failed to load curves: ${err}`));
      setStatus("Vega — load failed");
      return;
    }
    if (disposed || myGen !== gen) return; // a newer render (or close) already won
    const rows = type === "histogram" ? xValues(series, xName) : joinXYZ(series, xName, yName, useZ);
    lastRows = rows;
    lastType = type;
    lastX = xName;
    lastY = yName;
    lastZ = useZ;
    lastTrend = useTrend;
    lastMethod = method;
    await embedRows(type, rows, xName, yName, useZ, { trend: useTrend, method }, myGen);
  }

  /** Re-embed the cached rows with the new theme's colours (a theme switch resets zoom/pan — a rare,
   *  deliberate trade for repainting without a re-fetch). No-op until the first render has cached. */
  async function repaint(): Promise<void> {
    if (!lastRows) return;
    const myGen = ++gen;
    await embedRows(lastType, lastRows, lastX, lastY, lastZ, { trend: lastTrend, method: lastMethod, groupOrder: lastGroupOrder, groupLabel: lastGroupLabel }, myGen);
  }

  // --- V4: last-used persistence, export, spec editor ------------------------
  const persist = (): void =>
    savePlotProps("vega", {
      type: typeSel.value,
      x: xSel.value,
      y: ySel.value,
      z: zSel.value,
      zone: zoneSel.select.value,
      trend: trendChk.checked ? "1" : "",
      trendMethod: trendMethodSel.value,
      group: groupSel.value,
    });

  // Export. Vega renders to a <canvas>, so the shared PNG copy/save/print buttons work against it;
  // SVG comes from vega's own vector renderer.
  const getCanvas = (): HTMLCanvasElement | null => chartHost.querySelector<HTMLCanvasElement>("canvas");
  const exportSvg = async (): Promise<void> => {
    if (!current) {
      setStatus("No Vega chart to export yet");
      return;
    }
    try {
      const svg = await current.view.toSVG();
      const path = await saveSvg(svg, "Vega chart");
      if (path) {
        setStatus(`Vega chart SVG saved to ${path}`);
        recordProcess("Export", `Vega chart SVG (vector) → ${path}`);
      }
    } catch (err) {
      setStatus(`SVG export failed: ${err}`);
    }
  };
  const exportGroup = buildImageExportButtons(getCanvas, "Vega chart", setStatus);
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
  typeSel.addEventListener("change", () => {
    syncControls();
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

  const unsubBrush = appState.brushedDepths.subscribe((sel) => applyBrush(sel));
  const unsubTheme = appState.themeVersion.subscribe(() => {
    if (embedded && lastRows) void repaint();
  });

  // vega's container sizing needs the host attached with a non-zero size, which only happens once
  // the dock appends this panel. Embed on the first non-zero measurement; vega tracks resizes after.
  const ro = new ResizeObserver(() => {
    if (embedded || disposed) return;
    if (chartHost.clientWidth > 0 && chartHost.clientHeight > 0) {
      embedded = true;
      void render();
    }
  });
  ro.observe(chartHost);

  return {
    el: container,
    dispose: () => {
      disposed = true;
      if (emitRaf) cancelAnimationFrame(emitRaf);
      if (applyRaf) cancelAnimationFrame(applyRaf);
      window.removeEventListener("pointerup", onPointerUp);
      unsubBrush();
      unsubTheme();
      ro.disconnect();
      zoneSel.dispose();
      editor?.destroy();
      current?.finalize();
      current = null;
    },
    getState: () => ({
      type: typeSel.value,
      x: xSel.value,
      y: ySel.value,
      z: zSel.value,
      zone: zoneSel.select.value,
      trend: trendChk.checked ? "1" : "",
      trendMethod: trendMethodSel.value,
    }),
  };
}
