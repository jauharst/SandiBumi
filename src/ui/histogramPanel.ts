import { getCurveData, plotBindingSnapshot, plotBindingSnapshotForChannels, resolveWellScope, type PlotChannelBinding, type ResolvedPlotCurve, type WellSummary } from "../ipc";
import {
  HISTOGRAM_BINS_DEFAULT,
  HISTOGRAM_BINS_MAX,
  HISTOGRAM_BINS_MIN,
  canonicalHistogram,
  normalizeHistogramBinCount,
  type HistogramContract,
} from "../distribution";
import { appState, type BrushSelection } from "../state";
import { formRow, openModal } from "./modal";
import {
  attachKeyboardPanZoom,
  attachResizeRedraw,
  attachZoomPan,
  basicStats,
  buildPlotStatisticsRecord,
  fitCanvasBackingStore,
  formatPlotStatisticsRecord,
  makeCanvasAccessible,
  percentile,
  plotStatisticsInterval,
  PlotCanvas,
  canvasFont,
  readTheme,
  type BasicStats,
  type PlotStatisticsRecord,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildPlotTemplateBar,
  buildPersistedPlotState,
  buildZoneSelect,
  concatValues,
  contextReductionExport,
  CONTEXT_LEGEND_ROWS,
  contextZoneWindow,
  curveSelect,
  defaultPickParams,
  describeContextOutcome,
  fetchContextLayers,
  loadCurveNames,
  loadPlotProps,
  nearestDepthIndex,
  pickRow,
  plotWriteAxis,
  plotWriteSelection,
  savePlotProps,
  trySelect,
  type PlotContent,
  type PlotWriteSource,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import { buildWellScope, WELL_SCOPE_NAME_PREVIEW_ROWS } from "./wellScope";
import { renderPlotToSvg } from "./svgExport";
import { renderPlotToPdf, type PlotPdf } from "./pdfExport";
import { parsePercentileP, type PlotReductionExport } from "./plotTypes";
import {
  axisRangeExportRecord,
  formatAxisRangeSummary,
  resolveBoundAxisRange,
  type AxisDisplayRange,
  type PlotAxisRangeExport,
} from "./axisRange";
import { applyPlotRangePolicy, formatPlotRangePolicySummary, type PlotRangePolicyReport } from "./plotRangePolicy";

export type HistogramMode = "bars" | "line";

export type StatKey = "min" | "max" | "mean" | "p50" | "p5" | "p95" | "std" | "count";

export type StatsPlacement = "outside" | "inside" | "both";

export interface HistogramOptions {
  mode: HistogramMode;
  bins: number;
  /** Y axis as % of samples instead of raw count. */
  normalize: boolean;
  /** Which statistics chips are active; mean/p5/p50/p95 also draw marker lines. */
  stats: StatKey[];
  /** Cumulative % overlay curve (0–100% labels along the right edge). */
  cumulative: boolean;
  /** Box-and-whisker strip above the bars: P25–P75 box, P50 line, P5/P95 whiskers. */
  boxPlot: boolean;
  /** Bar/line color; empty = follow the theme accent. */
  color: string;
  /** Extra user percentiles (0 < p < 100): marker lines + statistic entries. */
  percentiles: number[];
  /** Where statistics are shown: chips above the plot, a block inside it, or both. */
  statsPlacement: StatsPlacement;
  /** Show the pick → zone-parameter rows. Off by default — the histogram is a
   *  general-purpose tool; parameter picking is opted into via Properties. */
  showPicks: boolean;
  /** Scientific validity exclusion is opt-in and never supplies the display axis. */
  validityFilter: boolean;
  validMin: number | null;
  validMax: number | null;
}

export const DEFAULT_HISTOGRAM_OPTIONS: HistogramOptions = {
  mode: "bars",
  bins: HISTOGRAM_BINS_DEFAULT,
  normalize: false,
  stats: ["p5", "p50", "p95", "count"],
  cumulative: false,
  boxPlot: false,
  color: "",
  percentiles: [],
  statsPlacement: "outside",
  showPicks: false,
  validityFilter: false,
  validMin: null,
  validMax: null,
};

const STAT_DEFS: { key: StatKey; label: string; fmt: (s: BasicStats) => string; marker: boolean }[] = [
  { key: "min", label: "Min", fmt: (s) => s.min.toPrecision(4), marker: false },
  { key: "max", label: "Max", fmt: (s) => s.max.toPrecision(4), marker: false },
  { key: "mean", label: "Mean", fmt: (s) => s.mean.toPrecision(4), marker: true },
  { key: "p50", label: "P50", fmt: (s) => s.p50.toPrecision(4), marker: true },
  { key: "p5", label: "P5", fmt: (s) => s.p5.toPrecision(4), marker: true },
  { key: "p95", label: "P95", fmt: (s) => s.p95.toPrecision(4), marker: true },
  { key: "std", label: "Std", fmt: (s) => s.std.toPrecision(4), marker: false },
  { key: "count", label: "n", fmt: (s) => String(s.count), marker: false },
];

const clampBins = normalizeHistogramBinCount;

/** Parses "10, 50, 90"-style user input into clean percentiles (bounded, deduped, sorted). */
export function parsePercentiles(text: string): number[] {
  const out = new Set<number>();
  for (const part of text.split(/[,;\s]+/)) {
    if (!part) continue;
    const p = Number(part);
    if (Number.isFinite(p) && p > 0 && p < 100) {
      out.add(parsePercentileP(Math.round(p * 100) / 100).value);
    }
  }
  return [...out].sort((a, b) => a - b);
}

/** Fills defaults, migrates v1 settings (cumulative used to be a display mode, now an
 *  overlay), and sanitizes everything user- or template-supplied. */
export function normalizeHistogramOptions(raw: Partial<HistogramOptions>): HistogramOptions {
  const opts: HistogramOptions = { ...DEFAULT_HISTOGRAM_OPTIONS, ...raw };
  if ((raw.mode as string) === "cumulative") {
    opts.mode = "bars";
    opts.cumulative = true;
  }
  if (opts.mode !== "bars" && opts.mode !== "line") opts.mode = DEFAULT_HISTOGRAM_OPTIONS.mode;
  opts.bins = clampBins(opts.bins);
  opts.color = typeof opts.color === "string" ? opts.color : "";
  opts.percentiles = Array.isArray(opts.percentiles)
    ? parsePercentiles(opts.percentiles.join(","))
    : [];
  opts.stats = Array.isArray(opts.stats)
    ? opts.stats.filter((k) => STAT_DEFS.some((d) => d.key === k))
    : [...DEFAULT_HISTOGRAM_OPTIONS.stats];
  if (!["outside", "inside", "both"].includes(opts.statsPlacement)) {
    opts.statsPlacement = DEFAULT_HISTOGRAM_OPTIONS.statsPlacement;
  }
  const validPair = typeof opts.validMin === "number" && Number.isFinite(opts.validMin)
    && typeof opts.validMax === "number" && Number.isFinite(opts.validMax)
    && opts.validMin !== opts.validMax;
  if (!validPair) {
    opts.validMin = null;
    opts.validMax = null;
    opts.validityFilter = false;
  } else {
    opts.validityFilter = !!opts.validityFilter;
  }
  return opts;
}

function histogramValidityRange(opts: HistogramOptions): AxisDisplayRange | null {
  return opts.validMin !== null && opts.validMax !== null
    ? { min: opts.validMin, max: opts.validMax }
    : null;
}

export function screenHistogramPopulation(
  values: ArrayLike<number>,
  opts: HistogramOptions,
  display: AxisDisplayRange | null,
): PlotRangePolicyReport {
  return applyPlotRangePolicy([{
    values,
    display,
    validity: histogramValidityRange(opts),
  }], opts.validityFilter);
}

function valuesAtIndices(values: ArrayLike<number>, indices: readonly number[]): Float32Array {
  return Float32Array.from(indices.map((index) => values[index]));
}

/** Reporting-surface adapter for the canonical histogram contract. */
export function computeHistogram(
  values: ArrayLike<number>,
  min: number,
  max: number,
  bins = DEFAULT_HISTOGRAM_OPTIONS.bins,
): HistogramContract {
  return canonicalHistogram(values, min, max, bins);
}

/** Live Histogram adapter: the values and every population decision become one record. */
export function buildHistogramStatisticsRecord(
  values: ArrayLike<number>,
  opts: HistogramOptions,
  curveName: string,
  wellId: string,
  intervalLow: number | null,
  intervalHigh: number | null,
  selectionLabel = "all eligible",
  display: AxisDisplayRange | null = null,
): PlotStatisticsRecord | null {
  const policy = screenHistogramPopulation(values, opts, display);
  if (policy.analysisCount === 0) return null;
  return buildPlotStatisticsRecord(
    valuesAtIndices(values, policy.indices),
    {
      binding_channel: "value",
      channel: `value:${curveName}`,
      population: "active_well",
      well_ids: [wellId],
      interval: plotStatisticsInterval(intervalLow, intervalHigh),
      selection: { kind: "all_eligible", selection_id: null, label: selectionLabel, applied: false },
      policy,
      selection_excluded: 0,
      unpaired_or_unclassified_excluded: 0,
      standard_deviation: "sample_n_minus_one",
    },
  );
}

/** Compact percentile label ("P10", "P97.5"). */
const pLabel = (p: number): string => `P${String(p)}`;

/** One extra well drawn as a stepped outline behind the active well's bars —
 *  display-only: stats, chips, picks, brushing and markers never read these values. */
export interface HistogramContextLayer {
  name: string;
  values: Float32Array;
  color: string;
}

export interface HistogramContext {
  /** The active well's name, for the legend's first row. */
  activeName: string;
  layers: HistogramContextLayer[];
}

/** Draws the histogram (bars or frequency polyline) with optional cumulative-% overlay,
 *  box-plot strip, statistic/percentile markers, an in-plot statistics block, pick
 *  markers, and an optional synchronized-hover marker at the curve's value under another
 *  view's cursor. Exported so it can be exercised with synthetic data in tests. */
export function drawHistogram(
  canvas: HTMLCanvasElement,
  values: ArrayLike<number>,
  curveName: string,
  picks: { value: number; color: string; label: string }[],
  opts: HistogramOptions = DEFAULT_HISTOGRAM_OPTIONS,
  hoverValue: number | null = null,
  view: Viewport | null = null,
  brushValues: ArrayLike<number> | null = null,
  context: HistogramContext | null = null,
  axisBinding: PlotChannelBinding | null = null,
  onAxisRanges?: (ranges: PlotAxisRangeExport[]) => void,
  statisticsRecord: PlotStatisticsRecord | null = null,
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const preliminary = screenHistogramPopulation(values, opts, null);
  const analysisValues = valuesAtIndices(values, preliminary.indices);
  if (analysisValues.length === 0) return null;
  // With context wells the auto range covers the pooled field spread, so a shifted
  // neighbour (the GR-normalization case) isn't clipped by the active well's window.
  const hasCtx = !!context && context.layers.length > 0;
  const contextValues = context?.layers.map((layer) =>
    valuesAtIndices(layer.values, screenHistogramPopulation(layer.values, opts, null).indices)) ?? [];
  const rangeValues = hasCtx
    ? concatValues(analysisValues, contextValues)
    : analysisValues;
  const p2 = percentile(rangeValues, 2);
  const p98 = percentile(rangeValues, 98);
  if (Number.isNaN(p2) || Number.isNaN(p98)) return null;
  // Floor the pad so a legitimately constant curve (p2 === p98, e.g. a flag/class curve or a
  // single-sample zone) still gets a positive-width window and renders one central bar,
  // instead of the old p2===p98 guard bailing out to a false "No valid data".
  const pad = (p98 - p2) * 0.08 || Math.max(Math.abs(p2) * 0.01, 1e-6);
  // Bin edges always span the full data range; a zoom/pan viewport only changes the
  // visible X window (the axis), so bars keep their identity as you zoom in.
  const min = p2 - pad;
  const max = p98 + pad;

  const xResolution = resolveBoundAxisRange({
    binding: axisBinding,
    user: view ? { min: view.xMin, max: view.xMax } : null,
    finiteData: { min, max },
    validity: histogramValidityRange(opts),
  });
  if (!xResolution) return null;
  const population = screenHistogramPopulation(values, opts, {
    min: xResolution.min,
    max: xResolution.max,
  });

  const bins = clampBins(opts.bins);
  const { counts, edges, displayedTotal } = computeHistogram(analysisValues, min, max, bins);
  if (displayedTotal === 0) return null;

  const stats = statisticsRecord?.values ?? basicStats(analysisValues);
  const yScale = opts.normalize ? 100 / displayedTotal : 1;

  // Context wells: each is binned over the SAME edges and normalized to its OWN sample
  // count, then scaled to the active axis — a neighbour with 3× the samples must not
  // dwarf the active well; what's compared across wells is distribution SHAPE. In count
  // mode a same-shaped well peaks at the active well's height; in % mode it's that
  // well's true per-well percentage. Computed before the axis so yMax covers them.
  const ctxCurves: { color: string; pts: [number, number][]; peak: number }[] = [];
  if (hasCtx) {
    for (let layerIndex = 0; layerIndex < context!.layers.length; layerIndex++) {
      const layer = context!.layers[layerIndex];
      const { counts: cc, displayedTotal: contextDisplayedTotal } = computeHistogram(contextValues[layerIndex], min, max, bins);
      if (contextDisplayedTotal === 0) continue;
      const scale = (displayedTotal * yScale) / contextDisplayedTotal;
      let layerPeak = 0;
      const pts: [number, number][] = [];
      for (let i = 0; i < cc.length; i++) {
        const y = cc[i] * scale;
        if (y > layerPeak) layerPeak = y;
        pts.push([edges[i], y], [edges[i + 1], y]);
      }
      ctxCurves.push({ color: layer.color, pts, peak: layerPeak });
    }
  }

  const peak = Math.max(Math.max(...counts, 1) * yScale, ...ctxCurves.map((c) => c.peak));
  const yMax = peak * 1.06;
  // The P2–P98 axis window can clip tail samples, so the in-window n is below the total valid
  // count that the stats chips show. Surface both ("n = X of Y") so the two never silently
  // disagree — a real QC trap for anyone standardizing GR on P3/P97 tails.
  const yLabel = opts.normalize
    ? `% of displayed samples (displayed n=${displayedTotal} of analysis n=${stats.count})`
    : `Count (displayed n=${displayedTotal} of analysis n=${stats.count})`;
  const yResolution = resolveBoundAxisRange({
    binding: null,
    user: null,
    finiteData: { min: 0, max: yMax },
  });
  if (!yResolution) return null;
  const xInvert = xResolution.min > xResolution.max;
  const resolvedRanges = [
    axisRangeExportRecord("x", xResolution),
    axisRangeExportRecord("y", yResolution),
  ];
  onAxisRanges?.(resolvedRanges);

  const plot = new PlotCanvas(
    canvas,
    { label: curveName, min: Math.min(xResolution.min, xResolution.max), max: Math.max(xResolution.min, xResolution.max), log: false, invert: xInvert },
    { label: yLabel, min: Math.min(yResolution.min, yResolution.max), max: Math.max(yResolution.min, yResolution.max), log: false, invert: yResolution.min > yResolution.max },
  );
  plot.drawFrame();
  plot.ctx.save();
  plot.ctx.font = canvasFont(plot.theme, 9);
  plot.ctx.fillStyle = plot.theme.axis;
  plot.ctx.textAlign = "left";
  plot.ctx.fillText(formatAxisRangeSummary(resolvedRanges), plot.plotRect.x0 + 4, plot.margin.top - 7);
  plot.ctx.textAlign = "right";
  plot.ctx.fillText(
    formatPlotRangePolicySummary(population, { statistics: true }),
    plot.plotRect.x0 + plot.plotRect.w,
    plot.plotRect.y0 + plot.plotRect.h + 31,
  );
  plot.ctx.restore();
  const barColor = opts.color || plot.theme.accent;

  // Context wells first (stepped outlines), so the active well's bars read on top.
  if (ctxCurves.length) {
    const { ctx } = plot;
    ctx.save();
    ctx.globalAlpha = 0.75;
    for (const c of ctxCurves) plot.drawLine(c.pts, c.color, 1.4);
    ctx.restore();
  }

  if (opts.mode === "bars") {
    const { ctx } = plot;
    ctx.save();
    ctx.fillStyle = barColor;
    ctx.globalAlpha = 0.75;
    for (let i = 0; i < counts.length; i++) {
      if (counts[i] === 0) continue;
      const [x0, yTop] = plot.toPx(edges[i], counts[i] * yScale);
      const [x1, yBase] = plot.toPx(edges[i + 1], 0);
      ctx.fillRect(x0 + 0.5, yTop, Math.max(1, x1 - x0 - 1), yBase - yTop);
    }
    ctx.restore();
  } else {
    const points: [number, number][] = counts.map((c, i) => [(edges[i] + edges[i + 1]) / 2, c * yScale]);
    plot.drawLine(points, barColor, 1.8);
  }

  // Brushed sub-distribution (linked selection): the selected samples' counts in the SAME bins,
  // over-painted in accent2 so you see where a brushed crossplot cloud falls in this property.
  if (brushValues && brushValues.length) {
    const eligibleBrush = valuesAtIndices(
      brushValues,
      screenHistogramPopulation(brushValues, opts, null).indices,
    );
    const bc = computeHistogram(eligibleBrush, min, max, bins).counts;
    const { ctx } = plot;
    ctx.save();
    ctx.fillStyle = plot.theme.accent2;
    ctx.globalAlpha = 0.85;
    for (let i = 0; i < bc.length; i++) {
      if (bc[i] === 0) continue;
      const [x0, yTop] = plot.toPx(edges[i], bc[i] * yScale);
      const [x1, yBase] = plot.toPx(edges[i + 1], 0);
      ctx.fillRect(x0 + 0.5, yTop, Math.max(1, x1 - x0 - 1), yBase - yTop);
    }
    ctx.restore();
  }

  // Cumulative % overlay: 0–100% mapped to the full plot height, labeled on the right.
  if (opts.cumulative) {
    const points: [number, number][] = [[edges[0], 0]];
    let running = 0;
    for (let i = 0; i < counts.length; i++) {
      running += counts[i];
      points.push([edges[i + 1], (running / displayedTotal) * yMax]);
    }
    plot.drawLine(points, plot.theme.accent2, 1.8);
    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.fillStyle = plot.theme.accent2;
    ctx.strokeStyle = plot.theme.accent2;
    ctx.font = canvasFont(plot.theme, 9);
    ctx.textAlign = "right";
    for (const c of [25, 50, 75, 100]) {
      const [, py] = plot.toPx(plot.x.min, (c / 100) * yMax);
      ctx.beginPath();
      ctx.moveTo(r.x0 + r.w - 4, py);
      ctx.lineTo(r.x0 + r.w, py);
      ctx.stroke();
      ctx.fillText(`${c}%`, r.x0 + r.w - 6, py + (c === 100 ? 9 : 3));
    }
    ctx.restore();
  }

  // Box-and-whisker strip across the top of the plot area.
  if (opts.boxPlot) {
    const q1 = stats.p25;
    const q3 = stats.p75;
    if (![q1, q3, stats.p5, stats.p50, stats.p95].some(Number.isNaN)) {
      const { ctx } = plot;
      const r = plot.plotRect;
      // Strip sits just below the marker-label line (drawVMarker text at y0+12).
      const boxTop = r.y0 + 20;
      const boxH = 14;
      const yC = boxTop + boxH / 2;
      const xAt = (v: number): number => plot.toPx(v, 0)[0];
      ctx.save();
      ctx.beginPath();
      ctx.rect(r.x0, r.y0, r.w, r.h);
      ctx.clip();
      ctx.strokeStyle = barColor;
      ctx.lineWidth = 1.2;
      // Whiskers P5→P25 and P75→P95 with end caps.
      for (const [a, b] of [
        [stats.p5, q1],
        [q3, stats.p95],
      ]) {
        ctx.beginPath();
        ctx.moveTo(xAt(a), yC);
        ctx.lineTo(xAt(b), yC);
        ctx.stroke();
      }
      for (const v of [stats.p5, stats.p95]) {
        ctx.beginPath();
        ctx.moveTo(xAt(v), yC - 5);
        ctx.lineTo(xAt(v), yC + 5);
        ctx.stroke();
      }
      ctx.globalAlpha = 0.3;
      ctx.fillStyle = barColor;
      ctx.fillRect(xAt(q1), boxTop, xAt(q3) - xAt(q1), boxH);
      ctx.globalAlpha = 1;
      ctx.strokeRect(xAt(q1), boxTop, xAt(q3) - xAt(q1), boxH);
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(xAt(stats.p50), boxTop);
      ctx.lineTo(xAt(stats.p50), boxTop + boxH);
      ctx.stroke();
      ctx.restore();
    }
  }

  for (const def of STAT_DEFS) {
    if (!def.marker || !opts.stats.includes(def.key)) continue;
    const v = { mean: stats.mean, p50: stats.p50, p5: stats.p5, p95: stats.p95 }[def.key as "mean" | "p50" | "p5" | "p95"];
    if (!Number.isNaN(v)) plot.drawVMarker(v, plot.theme.text, `${def.label} ${v.toPrecision(4)}`);
  }
  for (const p of opts.percentiles) {
    const v = percentile(analysisValues, p);
    if (!Number.isNaN(v)) plot.drawVMarker(v, plot.theme.text, `${pLabel(p)} ${v.toPrecision(4)}`);
  }
  for (const pick of picks) {
    if (!Number.isNaN(pick.value)) plot.drawVMarker(pick.value, pick.color, pick.label);
  }
  if (hoverValue !== null && !Number.isNaN(hoverValue)) {
    plot.drawVMarker(hoverValue, plot.theme.warn);
  }

  // In-plot statistics block (top-right, below the box-plot strip when present).
  if (opts.statsPlacement !== "outside") {
    const lines: string[] = [];
    for (const def of STAT_DEFS) {
      if (opts.stats.includes(def.key)) lines.push(`${def.label}  ${def.fmt(stats)}`);
    }
    for (const p of opts.percentiles) {
      const v = percentile(analysisValues, p);
      if (!Number.isNaN(v)) lines.push(`${pLabel(p)}  ${v.toPrecision(4)}`);
    }
    if (lines.length > 0) {
      const { ctx } = plot;
      const r = plot.plotRect;
      ctx.save();
      ctx.font = canvasFont(plot.theme, 10);
      const boxW = Math.max(...lines.map((l) => ctx.measureText(l).width)) + 16;
      const boxH = lines.length * 13 + 10;
      const bx = r.x0 + r.w - boxW - 8;
      const by = r.y0 + (opts.boxPlot ? 40 : 8);
      ctx.globalAlpha = 0.88;
      ctx.fillStyle = plot.theme.bg;
      ctx.fillRect(bx, by, boxW, boxH);
      ctx.globalAlpha = 1;
      ctx.strokeStyle = plot.theme.axis;
      ctx.strokeRect(bx, by, boxW, boxH);
      ctx.fillStyle = plot.theme.text;
      ctx.textAlign = "left";
      lines.forEach((l, i) => ctx.fillText(l, bx + 8, by + 15 + i * 13));
      ctx.restore();
    }
  }

  // Well legend for the multi-well overlay (top-left; the stats block owns top-right).
  // The footer states the contract on the plot itself: context wells are scaled to a
  // per-well shape comparison and are display-only.
  if (hasCtx) {
    const { ctx } = plot;
    const r = plot.plotRect;
    const trunc = (s: string) => (s.length > 18 ? `${s.slice(0, 17)}…` : s);
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    const rowH = 15;
    const boxX = r.x0 + 8;
    let boxY = r.y0 + (opts.boxPlot ? 42 : 20);
    ctx.font = canvasFont(plot.theme, 10, 600);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText("Wells", boxX, boxY + 9);
    boxY += rowH;
    ctx.font = canvasFont(plot.theme, 10);
    const row = (color: string, filled: boolean, label: string) => {
      if (filled) {
        ctx.fillStyle = color;
        ctx.fillRect(boxX, boxY + 1, 11, 11);
        ctx.strokeStyle = plot.theme.text;
        ctx.lineWidth = 0.5;
        ctx.strokeRect(boxX, boxY + 1, 11, 11);
      } else {
        // Outline swatch — matches how the context wells render (a stepped line, not bars).
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(boxX, boxY + 7);
        ctx.lineTo(boxX + 11, boxY + 7);
        ctx.stroke();
      }
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(label, boxX + 16, boxY + 10);
      boxY += rowH;
    };
    row(barColor, true, `${trunc(context!.activeName)} (active)`);
    const layers = context!.layers;
    for (const layer of layers.slice(0, CONTEXT_LEGEND_ROWS)) row(layer.color, false, trunc(layer.name));
    if (layers.length > CONTEXT_LEGEND_ROWS) {
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(`context legend: ${CONTEXT_LEGEND_ROWS} of ${layers.length} wells`, boxX + 16, boxY + 10);
      boxY += rowH;
    }
    ctx.font = canvasFont(plot.theme, 9);
    ctx.fillText("context: per-well shape · display-only", boxX, boxY + 9);
    ctx.restore();
  }
  return plot;
}

/** Histogram panel: pick a curve + zone window; everything about the display (bars/line,
 *  bins, normalize, cumulative overlay, box plot, color, user percentiles, statistics
 *  placement, parameter pickers) lives in a Properties dialog opened by double-click,
 *  right-click, or the ⚙ button. Follows the synchronized depth cursor
 *  of the log views (red marker = value at the hovered depth). */
export async function buildHistogramContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);
  const opts = normalizeHistogramOptions(await loadPlotProps<HistogramOptions>("histogram"));
  const plotId = initial?.plotId ?? crypto.randomUUID();

  const content = document.createElement("div");
  content.className = "plot-content";
  const curveSel = curveSelect(curveNames, initial?.curve ?? "GR");

  const propsBtn = document.createElement("button");
  propsBtn.className = "plot-export-btn";
  propsBtn.textContent = "⚙ Properties";
  propsBtn.title = "Histogram properties (or double-click / right-click the plot)";
  propsBtn.addEventListener("click", () => openProps());

  // --- Multi-well scope: extra wells drawn as stepped outlines BEHIND the active
  // well's bars, per-well normalized (shape comparison). Default "Active" keeps
  // today's single-well behaviour exactly; stats, chips, picks, brushing and the
  // hover marker stay bound to the active well — context wells are display-only.
  const scope = await buildWellScope({
    includeActive: true,
    defaultMode: "active",
    initial: initial?.wells,
    onChange: () => {
      updateScopeUi();
      void reloadContext();
    },
  });
  const scopeBtn = document.createElement("button");
  scopeBtn.className = "plot-export-btn";
  scopeBtn.title = "Overlay more wells' distributions as outlines behind the active well's bars";
  const scopeRow = document.createElement("div");
  scopeRow.style.display = "none";
  const scopeStaticHint = document.createElement("p");
  scopeStaticHint.className = "modal-hint";
  scopeStaticHint.textContent =
    "Context wells are display-only and normalized per well (shape comparison — a bigger well never dwarfs the active one): " +
    "statistics, chips, picks and brushing act on the active well only. " +
    "Zone/top windows are resolved per well by NAME (a well without that zone or top is skipped).";
  const scopeInfo = document.createElement("p");
  scopeInfo.className = "modal-hint";
  scopeRow.append(scope.el, scopeStaticHint, scopeInfo);
  scopeBtn.addEventListener("click", () => {
    scopeRow.style.display = scopeRow.style.display === "none" ? "" : "none";
  });
  const updateScopeUi = () => {
    scopeBtn.textContent = `Wells: ${scope.describe()}`;
    scopeInfo.textContent = ctxInfo;
    scopeInfo.style.display = ctxInfo ? "" : "none";
  };
  const plotIntents = () => [{ channel: "value", semantic_request: curveSel.value, required: true }];
  const representedWellIds = () => [well.well_id, ...ctxWellIds];
  let axisRanges: PlotAxisRangeExport[] = [];
  const currentAxisBinding = (): PlotChannelBinding | null =>
    plotBindingSnapshotForChannels(representedWellIds(), plotIntents())[0] ?? null;
  const selectionState = (): Record<string, string> => ({
    plotId,
    curve: curveSel.value,
    zone: zoneSel.select.value,
    wells: scope.serialize(),
  });
  const persistedState = (options: Record<string, unknown>) =>
    buildPersistedPlotState("histogram", options, representedWellIds(), plotIntents(), axisRanges);
  const persist = () => {
    try {
      void savePlotProps("histogram", persistedState({ ...opts }))
        .catch((error) => setStatus(`Histogram state not saved: ${error}`));
    } catch (error) {
      setStatus(`Histogram state not saved: ${error}`);
    }
  };

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("Curve", curveSel));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(scopeBtn);
  selRow.appendChild(propsBtn);

  // Named templates for the display style, and image export (copy / save / print).
  selRow.appendChild(
    buildPlotTemplateBar<HistogramOptions>(
      "histogram",
      "Histogram",
      () => persistedState({ ...opts }),
      (t) => {
        Object.assign(opts, normalizeHistogramOptions({ ...opts, ...t }));
        persist();
        renderChips();
        applyPicksVisibility();
        redraw();
      },
      setStatus,
    ),
  );
  selRow.appendChild(buildImageExportButtons(
    () => canvas,
    "Histogram",
    setStatus,
    () => getSvg(),
    () => getPdf(),
    () => ctxReductionManifest,
    () => {
      const state = persistedState(selectionState());
      return {
        wellIds: state.well_ids,
        curves: [curveSel.value],
        plotBindings: state.bindings,
        axisRanges: state.axis_ranges,
        statisticsRecords: currentStatisticsRecords(),
      };
    },
  ));
  content.appendChild(selRow);
  content.appendChild(scopeRow);

  // Statistics chips — click to toggle; active chips show the value and (for the
  // percentile/mean ones) draw a marker line on the plot. User percentiles from the
  // Properties dialog appear as removable chips after the fixed set.
  const chipsRow = document.createElement("div");
  chipsRow.className = "stat-chips";
  content.appendChild(chipsRow);
  let stats: BasicStats | null = null;
  let statisticsRecord: PlotStatisticsRecord | null = null;
  const currentStatisticsRecords = (): PlotStatisticsRecord[] => statisticsRecord ? [statisticsRecord] : [];

  const renderChips = () => {
    chipsRow.style.display = opts.statsPlacement === "inside" ? "none" : "";
    chipsRow.innerHTML = "";
    for (const def of STAT_DEFS) {
      const chip = document.createElement("button");
      const active = opts.stats.includes(def.key);
      chip.className = "stat-chip" + (active ? " active" : "");
      chip.textContent = active && stats && stats.count > 0 ? `${def.label} ${def.fmt(stats)}` : def.label;
      chip.title = active ? "Click to hide" : "Click to show";
      chip.addEventListener("click", () => {
        opts.stats = active ? opts.stats.filter((k) => k !== def.key) : [...opts.stats, def.key];
        persist();
        renderChips();
        redraw();
      });
      chipsRow.appendChild(chip);
    }
    for (const p of opts.percentiles) {
      const chip = document.createElement("button");
      chip.className = "stat-chip active";
      const v = percentile(currentAnalysisValues(), p);
      chip.textContent = Number.isNaN(v) ? pLabel(p) : `${pLabel(p)} ${v.toPrecision(4)}`;
      chip.title = "User percentile — click to remove (add more via Properties)";
      chip.addEventListener("click", () => {
        opts.percentiles = opts.percentiles.filter((x) => x !== p);
        persist();
        renderChips();
        redraw();
      });
      chipsRow.appendChild(chip);
    }
  };

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 380;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const statisticsInfo = document.createElement("p");
  statisticsInfo.className = "modal-hint";
  content.appendChild(statisticsInfo);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  content.appendChild(hint);
  const updateHint = () => {
    hint.textContent =
      (opts.showPicks ? "Click the plot to place the active pick. " : "") +
      "Double-click or right-click the plot for properties (when zoomed, double-click resets the zoom first). " +
      "Ctrl+wheel = zoom X, drag = pan.";
  };

  // Two picks with radio-style activation; hidden until enabled in Properties.
  const [p1Default, p2Default] = defaultPickParams(curveSel.value);
  const tc = readTheme(document.documentElement);
  const theme = { a: tc.accent, b: tc.accent2 };
  const pickA = pickRow("Pick A", theme.a, p1Default, well, zoneSel.current, setStatus, () => histogramWriteSource());
  const pickB = pickRow("Pick B", theme.b, p2Default, well, zoneSel.current, setStatus, () => histogramWriteSource());
  let active: "A" | "B" = "A";

  const activator = (which: "A" | "B", row: HTMLElement) => {
    row.classList.toggle("pick-active", active === which);
    row.addEventListener("click", () => {
      active = which;
      pickA.row.classList.toggle("pick-active", active === "A");
      pickB.row.classList.toggle("pick-active", active === "B");
    });
  };
  activator("A", pickA.row);
  activator("B", pickB.row);
  const picksWrap = document.createElement("div");
  picksWrap.append(pickA.row, pickB.row);
  content.appendChild(picksWrap);
  const applyPicksVisibility = () => {
    picksWrap.style.display = opts.showPicks ? "" : "none";
    updateHint();
  };
  applyPicksVisibility();

  let values: Float32Array = new Float32Array(0);
  let depths: Float32Array = new Float32Array(0);
  let plot: PlotCanvas | null = null;
  let hoverValue: number | null = null;
  let brushValues: number[] = []; // this curve's values at the shared-brush depths (this well)
  const viewRef: ViewportRef = { current: null };

  function currentAnalysisValues(): Float32Array {
    return valuesAtIndices(values, screenHistogramPopulation(values, opts, null).indices);
  }

  function refreshStatistics(display: AxisDisplayRange | null = null): void {
    const brush = appState.brushedDepths.get();
    const selectionLabel = brush && brush.wellId === well.well_id
      ? "all eligible; current brush not applied"
      : "all eligible";
    const zone = zoneSel.current();
    statisticsRecord = buildHistogramStatisticsRecord(
      values,
      opts,
      curveSel.value,
      well.well_id,
      zone.depthMin,
      zone.depthMax,
      selectionLabel,
      display,
    );
    if (!statisticsRecord) {
      statisticsRecord = null;
      stats = basicStats([]);
      statisticsInfo.textContent = "No governed statistics population.";
      renderChips();
      return;
    }
    stats = statisticsRecord.values;
    statisticsInfo.textContent = formatPlotStatisticsRecord(statisticsRecord);
    renderChips();
  }

  const resolvedBinding = (curveName: string): ResolvedPlotCurve | null =>
    plotBindingSnapshot([well.well_id], [curveName])
      .find((binding) => binding.intent.semantic_request.toUpperCase() === curveName.toUpperCase())
      ?.resolved[0] ?? null;

  async function histogramWriteSource(): Promise<PlotWriteSource> {
    if (!plot) throw new Error("plot-derived write requires a rendered viewport");
    const sourceCurve = resolvedBinding(curveSel.value);
    const xAxis = plotWriteAxis("value", sourceCurve);
    const zone = zoneSel.current();
    return {
      plot_id: plotId,
      plot_type: "histogram",
      x_axis: xAxis,
      y_axis: {
        ...xAxis,
        channel: opts.normalize ? "relative_frequency" : "sample_count",
        mnemonic: opts.normalize ? "RELATIVE_FREQUENCY" : "SAMPLE_COUNT",
        quantity: opts.normalize ? "relative_frequency" : "sample_count",
        source_unit: opts.normalize ? "%" : "count",
        display_unit: opts.normalize ? "%" : "count",
        conversion: `canonical_histogram_from:${xAxis.mnemonic}`,
      },
      z_axis: null,
      viewport: {
        x_min: plot.x.min,
        x_max: plot.x.max,
        y_min: plot.y.min,
        y_max: plot.y.max,
        x_log: plot.x.log,
        y_log: plot.y.log,
      },
      selection: await plotWriteSelection(well.well_id),
      interval: { low: zone.depthMin, high: zone.depthMax, closure: "[lo,hi)" },
      method: "manual_histogram_marker",
      fit_record: null,
    };
  }

  /** Recomputes the brushed subset's values for this curve (only when the brush targets THIS well). */
  const recomputeBrushValues = (sel: BrushSelection | null): void => {
    brushValues = [];
    if (!sel || sel.wellId !== well.well_id) return;
    for (let i = 0; i < depths.length; i++) {
      if (sel.depths.has(depths[i])) {
        const v = values[i];
        if (Number.isFinite(v)) brushValues.push(v);
      }
    }
  };

  // --- Context-well data (multi-well overlay) — same budget rule as the crossplot.
  const MAX_CONTEXT_POINTS = 60_000;
  let ctxLayers: HistogramContextLayer[] = [];
  let ctxReductionManifest: PlotReductionExport | null = null;
  let ctxWellIds: string[] = [];
  let ctxInfo = "";
  let ctxGen = 0;
  const histContext = (): HistogramContext | null =>
    ctxLayers.length ? { activeName: well.well_name, layers: ctxLayers } : null;

  /** Fetches the scoped context wells' curve through the shared plotCommon machinery
   *  (per-well zone/top-by-name windows, point budget, cancellation). Scope = just the
   *  active well → clears the overlay: byte-identical single-well behaviour. */
  const reloadContext = async () => {
    const gen = ++ctxGen;
    let resolvedIds: string[];
    try {
      resolvedIds = await resolveWellScope(scope.backend());
    } catch (error) {
      if (gen === ctxGen) setStatus(`Histogram scope refused: ${error}`);
      return;
    }
    if (gen !== ctxGen) return;
    const ids = resolvedIds.filter((id) => id !== well.well_id);
    if (ids.length === 0) {
      const had = ctxLayers.length > 0;
      ctxLayers = [];
      ctxWellIds = [];
      ctxReductionManifest = null;
      ctxInfo = "";
      updateScopeUi();
      if (had) redraw();
      return;
    }
    ctxReductionManifest = contextReductionExport(
      "histogram",
      null,
      resolvedIds.length,
      WELL_SCOPE_NAME_PREVIEW_ROWS,
    );
    setStatus(`Histogram: loading ${ids.length} context well${ids.length === 1 ? "" : "s"}…`);
    const outcome = await fetchContextLayers({
      ids,
      names: scope.namesFor(ids),
      curves: [curveSel.value],
      windowFor: (id) => contextZoneWindow(zoneSel, id),
      budget: MAX_CONTEXT_POINTS,
      isStale: () => gen !== ctxGen,
    });
    if (!outcome) return; // superseded by a newer call (or dispose)
    ctxReductionManifest = contextReductionExport(
      "histogram",
      outcome,
      resolvedIds.length,
      WELL_SCOPE_NAME_PREVIEW_ROWS,
    );
    ctxLayers = outcome.layers.map((l) => ({
      name: l.name,
      color: l.color,
      values: l.series.get(curveSel.value.toUpperCase())!,
    }));
    ctxWellIds = outcome.layers.map((layer) => layer.wellId);
    ctxInfo = describeContextOutcome(outcome);
    updateScopeUi();
    setStatus(`Histogram ${ctxInfo.toLowerCase()}`);
    redraw();
  };
  updateScopeUi();

  const redraw = () => {
    canvas.setAttribute("aria-label", `Histogram of ${curveSel.value}`); // a11y label follows the curve
    axisRanges = [];
    plot = drawHistogram(
      canvas,
      values,
      curveSel.value,
      opts.showPicks
        ? [
            { value: pickA.getValue(), color: theme.a, label: "A" },
            { value: pickB.getValue(), color: theme.b, label: "B" },
          ]
        : [],
      opts,
      hoverValue,
      viewRef.current,
      brushValues,
      histContext(),
      currentAxisBinding(),
      (ranges) => {
        axisRanges = ranges;
      },
      statisticsRecord,
    );
    if (!plot) {
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const th = readTheme(canvas);
      ctx.font = canvasFont(th, 12);
      ctx.fillStyle = th.text;
      ctx.textAlign = "center";
      ctx.fillText("No valid data for this curve/zone.", canvas.width / 2, canvas.height / 2);
      return;
    }
    refreshStatistics({ min: plot.x.min, max: plot.x.max });
  };

  // Vector export: re-run the same static draw (no hover marker, no brush overlay) into a
  // recording context sized to the live plot.
  // The static draw shared by the two vector-export paths (no hover marker, no brush overlay).
  const drawStatic = (c: HTMLCanvasElement) =>
    drawHistogram(
      c,
      values,
      curveSel.value,
      opts.showPicks
        ? [
            { value: pickA.getValue(), color: theme.a, label: "A" },
            { value: pickB.getValue(), color: theme.b, label: "B" },
          ]
        : [],
      opts,
      null,
      viewRef.current,
      null,
      histContext(),
      currentAxisBinding(),
      (ranges) => {
        axisRanges = ranges;
      },
      statisticsRecord,
    );
  const getSvg = (): string | null => (plot ? renderPlotToSvg(plot.width, plot.height, drawStatic) : null);
  const getPdf = (): PlotPdf | null => (plot ? renderPlotToPdf(plot.width, plot.height, drawStatic) : null);

  // Monotonic token so a slow curve/zone load that resolves after a newer one (fast
  // switching) can't overwrite the newer data. `preserveView` keeps the zoom/pan on a
  // data refresh (module run) while a user-initiated curve/zone change still re-fits.
  let reloadGen = 0;
  // A reset-intent reload (curve/zone change) can be superseded by a preserveView data
  // refresh; this sticky flag carries the "reset the viewport" intent to whichever reload
  // actually commits, so a background bump can't strand the new curve at the old zoom.
  let resetPending = false;
  const reload = async (preserveView = false) => {
    const gen = ++reloadGen;
    if (!preserveView) resetPending = true;
    const zone = zoneSel.current();
    try {
      const series = await getCurveData(well.well_id, [curveSel.value], zone.depthMin, zone.depthMax);
      if (gen !== reloadGen) return; // a newer reload started while we awaited
      values = series[0]?.value ?? new Float32Array(0);
      depths = series[0]?.depth ?? new Float32Array(0);
    } catch (err) {
      if (gen !== reloadGen) return; // superseded — don't clobber newer data with this error
      setStatus(`Histogram data load failed: ${err}`);
      values = new Float32Array(0);
      depths = new Float32Array(0);
    }
    hoverValue = null; // the old hover marker points at a stale value after new data
    recomputeBrushValues(appState.brushedDepths.get()); // depths grid changed — re-map the brush
    if (resetPending) {
      viewRef.current = null; // new data → reset any zoom/pan
      resetPending = false;
    }
    refreshStatistics();
    redraw();
  };

  // Properties dialog — everything about the display in one place.
  const openProps = () => {
    const body = document.createElement("div");

    const modeSel = document.createElement("select");
    modeSel.className = "form-control";
    for (const [value, label] of [
      ["bars", "Bars"],
      ["line", "Line"],
    ] as [HistogramMode, string][]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      modeSel.appendChild(option);
    }
    modeSel.value = opts.mode;

    const binsIn = document.createElement("input");
    binsIn.className = "form-control";
    binsIn.type = "number";
    binsIn.min = String(HISTOGRAM_BINS_MIN);
    binsIn.max = String(HISTOGRAM_BINS_MAX);
    binsIn.value = String(opts.bins);

    const chk = (label: string, checked: boolean): { el: HTMLElement; input: HTMLInputElement } => {
      const wrap = document.createElement("label");
      wrap.className = "chk-field";
      const input = document.createElement("input");
      input.type = "checkbox";
      input.checked = checked;
      wrap.append(input, document.createTextNode(label));
      return { el: wrap, input };
    };

    const normChk = chk("Normalize (%)", opts.normalize);
    const cumChk = chk("Cumulative % overlay", opts.cumulative);
    const boxChk = chk("Box plot (P5–P25–P50–P75–P95)", opts.boxPlot);
    const overlayRow = document.createElement("div");
    overlayRow.style.margin = "4px 0 8px";
    overlayRow.append(normChk.el, cumChk.el, boxChk.el);

    const themeChk = chk("Theme color", !opts.color);
    const colorIn = document.createElement("input");
    colorIn.type = "color";
    colorIn.className = "form-control";
    colorIn.style.width = "48px";
    colorIn.style.padding = "1px";
    colorIn.value = /^#[0-9a-fA-F]{6}$/.test(opts.color) ? opts.color : "#b5651d";
    colorIn.disabled = !opts.color;
    themeChk.input.addEventListener("change", () => {
      colorIn.disabled = themeChk.input.checked;
    });
    const colorWrap = document.createElement("div");
    colorWrap.style.display = "flex";
    colorWrap.style.gap = "8px";
    colorWrap.style.alignItems = "center";
    colorWrap.append(themeChk.el, colorIn);

    const pctIn = document.createElement("input");
    pctIn.className = "form-control";
    pctIn.placeholder = "e.g. 10, 50, 90";
    pctIn.value = opts.percentiles.join(", ");

    const placeSel = document.createElement("select");
    placeSel.className = "form-control";
    for (const [value, label] of [
      ["outside", "Chips above the plot"],
      ["inside", "Inside the plot"],
      ["both", "Both"],
    ] as [StatsPlacement, string][]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      placeSel.appendChild(option);
    }
    placeSel.value = opts.statsPlacement;

    const picksChk = chk("Show parameter pickers (Pick A/B → zone parameter)", opts.showPicks);
    picksChk.el.style.margin = "2px 0 8px";
    const validityChk = chk("Apply validity range to n and statistics", opts.validityFilter);
    const validityNumber = (value: number | null, placeholder: string): HTMLInputElement => {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.placeholder = placeholder;
      input.value = value === null ? "" : String(value);
      return input;
    };
    const validMinIn = validityNumber(opts.validMin, "min");
    const validMaxIn = validityNumber(opts.validMax, "max");
    const validityRange = document.createElement("div");
    validityRange.style.display = "flex";
    validityRange.style.gap = "8px";
    validityRange.append(validMinIn, validMaxIn);

    body.appendChild(formRow("Display", modeSel));
    body.appendChild(formRow("Bins", binsIn));
    body.appendChild(overlayRow);
    body.appendChild(formRow("Color", colorWrap));
    body.appendChild(formRow("Percentiles", pctIn, "Extra user percentiles, comma-separated (0–100)"));
    body.appendChild(formRow("Statistics", placeSel));
    body.appendChild(validityChk.el);
    body.appendChild(formRow("Valid range", validityRange, "Explicit opt-in: changes n/statistics; display clipping does not"));
    body.appendChild(picksChk.el);

    const applyBtn = document.createElement("button");
    applyBtn.className = "lp-btn primary";
    applyBtn.textContent = "Apply";
    applyBtn.style.marginTop = "10px";
    body.appendChild(applyBtn);

    const close = openModal("Histogram Properties", body, 420);
    applyBtn.addEventListener("click", () => {
      const validMinText = validMinIn.value.trim();
      const validMaxText = validMaxIn.value.trim();
      const validMin = validMinText === "" ? null : Number(validMinText);
      const validMax = validMaxText === "" ? null : Number(validMaxText);
      const validPair = validMin !== null && validMax !== null
        && Number.isFinite(validMin) && Number.isFinite(validMax) && validMin !== validMax;
      if ((validMinText !== "" || validMaxText !== "" || validityChk.input.checked) && !validPair) {
        setStatus("Histogram validity requires two distinct finite limits, or two blanks while disabled");
        return;
      }
      opts.mode = modeSel.value as HistogramMode;
      opts.bins = clampBins(parseInt(binsIn.value, 10));
      opts.normalize = normChk.input.checked;
      opts.cumulative = cumChk.input.checked;
      opts.boxPlot = boxChk.input.checked;
      opts.color = themeChk.input.checked ? "" : colorIn.value;
      opts.percentiles = parsePercentiles(pctIn.value);
      opts.statsPlacement = placeSel.value as StatsPlacement;
      opts.showPicks = picksChk.input.checked;
      opts.validityFilter = validityChk.input.checked;
      opts.validMin = validPair ? validMin : null;
      opts.validMax = validPair ? validMax : null;
      persist();
      refreshStatistics();
      applyPicksVisibility();
      redraw();
      setStatus("Histogram properties applied");
      close();
    });
  };

  curveSel.addEventListener("change", () => {
    const [pa, pb] = defaultPickParams(curveSel.value);
    (pickA.row.querySelector(".pick-param") as HTMLInputElement).value = pa;
    (pickB.row.querySelector(".pick-param") as HTMLInputElement).value = pb;
    void reload();
    void reloadContext(); // context wells show the same curve
  });
  zoneSel.select.addEventListener("change", () => {
    void reload();
    void reloadContext(); // per-well zone windows follow the zone choice
  });

  // Track drag so a pan (below) doesn't also fire a pick. Coordinates are logical (CSS)
  // pixels — the space PlotCanvas.toData works in after HiDPI scaling.
  let downX: number | null = null;
  let movedSinceDown = false;
  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    const rect = canvas.getBoundingClientRect();
    downX = e.clientX - rect.left;
    movedSinceDown = false;
  });
  canvas.addEventListener("mousemove", (e) => {
    if (downX === null) return;
    if (Math.abs(e.clientX - canvas.getBoundingClientRect().left - downX) > 4) movedSinceDown = true;
  });
  canvas.addEventListener("click", (e) => {
    downX = null;
    if (!plot || movedSinceDown || !opts.showPicks) return; // pan tail / pickers hidden
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    if (!plot.inPlot(px, py)) return;
    const [vx] = plot.toData(px, py);
    (active === "A" ? pickA : pickB).setValue(vx);
    redraw();
  });

  // Double-click = properties, unless a zoom is active (then attachZoomPan — registered
  // after this listener — resets it on the same event; the next double-click opens
  // properties). Right-click is deliberately NOT bound here: it belongs to the pane
  // context menu, which lists Properties… first alongside the window actions.
  canvas.addEventListener("dblclick", () => {
    if (!viewRef.current) openProps();
  });

  // Wheel-zoom + drag-pan on the X axis only (Y is the count axis); double-click resets.
  makeCanvasAccessible(canvas, `Histogram of ${curveSel.value}`);
  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw, axes: "x" });
  const detachKeys = attachKeyboardPanZoom({ canvas, getPlot: () => plot, view: viewRef, redraw, axes: "x" });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

  // Re-fetch when computed curves change (module/equation run, import, undo) so the
  // histogram never shows stale data; keep the current zoom/pan. The primed flag drops
  // subscribe's immediate fire so the trailing `await reload()` stays the only build load.
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    void reload(true);
    void reloadContext(); // a module run may have rewritten the context wells' curves too
  });

  // Synchronized hover: mark this curve's value at the depth under any log view's cursor.
  let rafId = 0;
  const unsubHover = appState.hoverDepth.subscribe((depth) => {
    const idx = depth === null ? -1 : nearestDepthIndex(depths, depth);
    const next = idx < 0 ? null : values[idx];
    const normalized = next === undefined || Number.isNaN(next!) ? null : next;
    if (normalized === hoverValue) return;
    hoverValue = normalized;
    if (!rafId) {
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        redraw();
      });
    }
  });

  // Linked brushing: highlight this curve's sub-distribution for the shared brush's samples.
  const unsubBrush = appState.brushedDepths.subscribe((sel) => {
    recomputeBrushValues(sel);
    refreshStatistics();
    if (!rafId) {
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        redraw();
      });
    }
  });

  await reload();
  // Not awaited: a big scope must not block the panel build — the active well's plot
  // appears immediately and the context outlines fade in when ready.
  const bindingReady = reloadContext();
  return {
    el: content,
    dispose: () => {
      ctxGen++; // cancel any in-flight context fetch
      scope.dispose();
      unsubHover();
      unsubTheme();
      unsubData();
      unsubBrush();
      detachZoomPan();
      detachKeys();
      detachResize();
      if (rafId) cancelAnimationFrame(rafId);
      zoneSel.dispose();
    },
    getState: selectionState,
    getPersistedState: () => persistedState(selectionState()),
    bindingReady,
    openProperties: openProps,
  };
}
