import { getCurveData, plotBindingSnapshot, plotBindingSnapshotForChannels, type PlotAncestryScope, type PlotChannelBinding, type ResolvedPlotCurve, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow, openModal } from "./modal";
import {
  attachKeyboardPanZoom,
  attachScatterTooltip,
  attachZoomPan,
  buildPlotStatisticsRecord,
  colormapColor,
  fitCanvasBackingStore,
  formatPlotStatisticsRecord,
  fmtValue,
  makeCanvasAccessible,
  PlotCanvas,
  canvasFont,
  readTheme,
  percentile,
  plotStatisticsInterval,
  type ColormapName,
  type PlotStatisticsRecord,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildPlotTemplateBar,
  buildPersistedPlotState,
  buildDepthReframeHandoff,
  buildZoneSelect,
  curveSelect,
  depthReframeHandoff,
  contextLegend,
  createContextReload,
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
import { buildWellScope } from "./wellScope";
import { renderPlotToPaperSvg } from "./svgExport";
import { renderPlotToPaperPdf, type PlotPdf } from "./pdfExport";
import {
  applyPlotChannelPolicy,
  reconcileDepthChannels,
  type PlotRangeEdge,
  type PlotReductionExport,
} from "./plotTypes";
import {
  axisRangeExportRecord,
  formatAxisRangeSummary,
  resolveBoundAxisRange,
  type AxisDisplayRange,
  type PlotAxisRangeExport,
} from "./axisRange";
import { applyPlotRangePolicy, formatPlotRangePolicySummary, type PlotRangePolicyReport } from "./plotRangePolicy";
import { registerPlotInvalidationContract } from "./plotInvalidation";
import { beginPlotAsyncGeneration, isPlotAsyncGenerationCurrent } from "./plotAsync";

/** Persisted Pickett v2 display settings (plotprops doc "pickett"). Complete axis pairs are
 *  user overrides; absent pairs continue to header/family/finite data. */
interface PickettProps {
  /** A complete positive pair is a user override; null/null continues down SB-PLT-002's chain. */
  rtMin: number | null;
  rtMax: number | null;
  phiMin: number | null;
  phiMax: number | null;
  pointSize: number;
  /** Curve to color points by; "" = single theme color. */
  zCurve: string;
  colormap: ColormapName;
  zLog: boolean;
  validityFilter: boolean;
  rtValidMin: number | null;
  rtValidMax: number | null;
  phiValidMin: number | null;
  phiValidMax: number | null;
}

const PICKETT_DEFAULTS: PickettProps = {
  // Axis defaults are deliberately absent. Concrete header/family metadata or finite data
  // supplies the range until a user records an explicit override.
  rtMin: null,
  rtMax: null,
  phiMin: null,
  phiMax: null,
  pointSize: 1.8,
  zCurve: "",
  colormap: "rainbow",
  zLog: false,
  validityFilter: false,
  rtValidMin: null,
  rtValidMax: null,
  phiValidMin: null,
  phiValidMax: null,
};

/** Fills defaults and sanitizes saved/template-supplied props (a template can carry
 *  anything, so every field is guarded — same policy as normalizeCrossplotOptions). */
export function sanitizePickettProps(raw: Partial<PickettProps>): PickettProps {
  const p: PickettProps = { ...PICKETT_DEFAULTS, ...raw };
  const pos = (v: unknown, fb: number): number =>
    typeof v === "number" && Number.isFinite(v) && v > 0 ? v : fb;
  const pair = (low: unknown, high: unknown): [number | null, number | null] => {
    const valid = (value: unknown): value is number =>
      typeof value === "number" && Number.isFinite(value) && value > 0;
    return valid(low) && valid(high) && low !== high ? [low, high] : [null, null];
  };
  [p.rtMin, p.rtMax] = pair(p.rtMin, p.rtMax);
  [p.phiMin, p.phiMax] = pair(p.phiMin, p.phiMax);
  [p.rtValidMin, p.rtValidMax] = pair(p.rtValidMin, p.rtValidMax);
  [p.phiValidMin, p.phiValidMax] = pair(p.phiValidMin, p.phiValidMax);
  p.validityFilter = !!p.validityFilter
    && (p.rtValidMin !== null || p.phiValidMin !== null);
  p.pointSize = Math.max(0.5, Math.min(8, pos(p.pointSize, PICKETT_DEFAULTS.pointSize)));
  p.zCurve = typeof p.zCurve === "string" ? p.zCurve : "";
  if (p.colormap !== "viridis") p.colormap = "rainbow";
  p.zLog = !!p.zLog;
  return p;
}

interface PickettRenderStyle {
  rtMin?: number | null;
  rtMax?: number | null;
  phiMin?: number | null;
  phiMax?: number | null;
  pointSize?: number;
  zValues?: Float32Array;
  zRange?: PlotAxisRangeExport | null;
  zLog?: boolean;
  colormap?: ColormapName;
  validityFilter?: boolean;
  rtValidMin?: number | null;
  rtValidMax?: number | null;
  phiValidMin?: number | null;
  phiValidMax?: number | null;
}

export interface PickettColourPolicy {
  colors: string[];
  included: Uint8Array;
  edgeMarks: PlotRangeEdge[];
  nonFiniteExcluded: number;
  logDomainExcluded: number;
  excluded: number;
  clamped: number;
}

/** SB-PLT-013's Pickett Z adapter. It colours a derived, display-clamped copy and
 * leaves every source sample bit-for-bit unchanged. */
export function buildPickettColourPolicy(
  source: Float32Array,
  display: AxisDisplayRange,
  logAxis: boolean,
  colormap: ColormapName,
): PickettColourPolicy {
  const policy = applyPlotChannelPolicy(source, "colour", display, logAxis);
  const low = Math.min(display.min, display.max);
  const high = Math.max(display.min, display.max);
  const transformedLow = logAxis ? Math.log10(low) : low;
  const transformedHigh = logAxis ? Math.log10(high) : high;
  const colors = new Array<string>(policy.values.length).fill("rgba(0,0,0,0)");
  for (let index = 0; index < colors.length; index++) {
    if (policy.included[index] === 0) continue;
    const transformed = logAxis ? Math.log10(policy.values[index]) : policy.values[index];
    colors[index] = colormapColor(
      colormap,
      (transformed - transformedLow) / (transformedHigh - transformedLow),
    );
  }
  return {
    colors,
    included: policy.included,
    edgeMarks: policy.edgeMarks,
    nonFiniteExcluded: policy.nonFiniteExcluded,
    logDomainExcluded: policy.logDomainExcluded,
    excluded: policy.nonFiniteExcluded + policy.logDomainExcluded,
    clamped: policy.clamped,
  };
}

function pickettRange(low: number | null | undefined, high: number | null | undefined): AxisDisplayRange | null {
  return low !== null && low !== undefined && high !== null && high !== undefined
    ? { min: low, max: high }
    : null;
}

export function screenPickettPopulation(
  rt: Float32Array,
  phi: Float32Array,
  style: PickettRenderStyle | undefined,
  rtDisplay: AxisDisplayRange | null,
  phiDisplay: AxisDisplayRange | null,
): PlotRangePolicyReport {
  return applyPlotRangePolicy([
    {
      values: rt,
      display: rtDisplay,
      validity: pickettRange(style?.rtValidMin, style?.rtValidMax),
      log: true,
    },
    {
      values: phi,
      display: phiDisplay,
      validity: pickettRange(style?.phiValidMin, style?.phiValidMax),
      log: true,
    },
  ], !!style?.validityFilter);
}

/** Live Pickett adapter: RT and porosity share one finite-positive pair population. */
export function buildPickettStatisticsRecords(
  rt: Float32Array,
  phi: Float32Array,
  style: PickettRenderStyle | undefined,
  rtName: string,
  phiName: string,
  wellId: string,
  intervalLow: number | null,
  intervalHigh: number | null,
  rtDisplay: AxisDisplayRange | null,
  phiDisplay: AxisDisplayRange | null,
  selectionLabel = "all eligible",
): PlotStatisticsRecord[] {
  const policy = screenPickettPopulation(rt, phi, style, rtDisplay, phiDisplay);
  if (policy.analysisCount === 0) return [];
  const context = {
    population: "active_well" as const,
    well_ids: [wellId],
    interval: plotStatisticsInterval(intervalLow, intervalHigh),
    selection: { kind: "all_eligible" as const, selection_id: null, label: selectionLabel, applied: false },
    policy,
    selection_excluded: 0,
    unpaired_or_unclassified_excluded: 0,
    standard_deviation: "sample_n_minus_one" as const,
  };
  return [
    buildPlotStatisticsRecord(
      Float32Array.from(policy.indices.map((index) => rt[index])),
      { ...context, binding_channel: "resistivity", channel: `resistivity:${rtName}` },
    ),
    buildPlotStatisticsRecord(
      Float32Array.from(policy.indices.map((index) => phi[index])),
      { ...context, binding_channel: "porosity", channel: `porosity:${phiName}` },
    ),
  ];
}

/** One extra well's cloud drawn faded behind the active well — display-only: the fitted
 *  product line, picks, brushing and tooltips stay the ACTIVE well's. */
export interface PickettContextLayer {
  name: string;
  rt: Float32Array;
  phi: Float32Array;
  color: string;
}

export interface PickettContext {
  /** The active well's name, for the legend's first row. */
  activeName: string;
  layers: PickettContextLayer[];
}

/** Water-line fit from two picked points on the log-log plot.
 *  At Sw=1 the intercept is a·Rw: RT = (a·Rw)/PHI^m. The plot cannot identify
 *  a and Rw separately without one independently supplied, provenance-bearing value. */
export function fitWaterLine(
  p1: [number, number],
  p2: [number, number],
): { m: number; aRw: number } | null {
  const [rt1, phi1] = p1;
  const [rt2, phi2] = p2;
  if (rt1 <= 0 || rt2 <= 0 || phi1 <= 0 || phi2 <= 0) return null;
  const dLogPhi = Math.log10(phi2) - Math.log10(phi1);
  if (Math.abs(dLogPhi) < 1e-6) return null;
  const m = -(Math.log10(rt2) - Math.log10(rt1)) / dLogPhi;
  const aRw = rt1 * Math.pow(phi1, m);
  if (!Number.isFinite(m) || !Number.isFinite(aRw) || m <= 0 || aRw <= 0) return null;
  return { m, aRw };
}

export function drawPickett(
  canvas: HTMLCanvasElement,
  rt: Float32Array,
  phi: Float32Array,
  line: { m: number; aRw: number } | null,
  picks: [number, number][],
  hoverIdx = -1,
  view: Viewport | null = null,
  style?: PickettRenderStyle,
  context: PickettContext | null = null,
  axisBindings: { resistivity: PlotChannelBinding | null; porosity: PlotChannelBinding | null } | null = null,
  onAxisRanges?: (ranges: PlotAxisRangeExport[]) => void,
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const preliminary = screenPickettPopulation(rt, phi, style, null, null);
  const plotRt = Float32Array.from(preliminary.indices.map((index) => rt[index]));
  const plotPhi = Float32Array.from(preliminary.indices.map((index) => phi[index]));
  const plotZ = style?.zValues
    ? Float32Array.from(preliminary.indices.map((index) => style.zValues![index]))
    : null;
  const colourPolicy = plotZ && style?.zRange
    ? buildPickettColourPolicy(
        plotZ,
        { min: style.zRange.min, max: style.zRange.max },
        !!style.zLog,
        style.colormap ?? "rainbow",
      )
    : null;
  const plotColors = colourPolicy?.colors;
  const contextLayers = context?.layers.map((layer) => {
    const screened = screenPickettPopulation(layer.rt, layer.phi, style, null, null);
    return {
      ...layer,
      rt: Float32Array.from(screened.indices.map((index) => layer.rt[index])),
      phi: Float32Array.from(screened.indices.map((index) => layer.phi[index])),
    };
  }) ?? [];
  const finitePositiveRange = (active: Float32Array, layers: Float32Array[]): { min: number; max: number } | null => {
    const values: number[] = [];
    for (const source of [active, ...layers]) {
      for (const value of source) if (Number.isFinite(value) && value > 0) values.push(value);
    }
    if (values.length < 2) return null;
    const min = percentile(values, 2);
    const max = percentile(values, 98);
    return Number.isFinite(min) && Number.isFinite(max) && min !== max ? { min, max } : null;
  };
  const rtRange = resolveBoundAxisRange({
    binding: axisBindings?.resistivity ?? null,
    user: view
      ? { min: view.xMin, max: view.xMax }
      : style?.rtMin !== null && style?.rtMin !== undefined && style?.rtMax !== null && style?.rtMax !== undefined
        ? { min: style.rtMin, max: style.rtMax }
        : null,
    finiteData: finitePositiveRange(plotRt, contextLayers.map((layer) => layer.rt)),
    validity: pickettRange(style?.rtValidMin, style?.rtValidMax),
    log: true,
  });
  const phiRange = resolveBoundAxisRange({
    binding: axisBindings?.porosity ?? null,
    user: view
      ? { min: view.yMin, max: view.yMax }
      : style?.phiMin !== null && style?.phiMin !== undefined && style?.phiMax !== null && style?.phiMax !== undefined
        ? { min: style.phiMin, max: style.phiMax }
        : null,
    finiteData: finitePositiveRange(plotPhi, contextLayers.map((layer) => layer.phi)),
    validity: pickettRange(style?.phiValidMin, style?.phiValidMax),
    log: true,
  });
  if (!rtRange || !phiRange) return null;
  const population = screenPickettPopulation(
    rt,
    phi,
    style,
    { min: rtRange.min, max: rtRange.max },
    { min: phiRange.min, max: phiRange.max },
  );
  const pickPopulation = picks.length > 0
    ? screenPickettPopulation(
        Float32Array.from(picks.map((pick) => pick[0])),
        Float32Array.from(picks.map((pick) => pick[1])),
        style,
        null,
        null,
      )
    : null;
  const resolvedRanges = [
    axisRangeExportRecord("x", rtRange),
    axisRangeExportRecord("y", phiRange),
    ...(style?.zRange ? [style.zRange] : []),
  ];
  onAxisRanges?.(resolvedRanges);
  const plot = new PlotCanvas(
    canvas,
    {
      label: "RT (ohmm)",
      min: Math.min(rtRange.min, rtRange.max),
      max: Math.max(rtRange.min, rtRange.max),
      log: true,
      invert: rtRange.min > rtRange.max,
    },
    {
      label: "PHIE (v/v)",
      min: Math.min(phiRange.min, phiRange.max),
      max: Math.max(phiRange.min, phiRange.max),
      log: true,
      invert: phiRange.min > phiRange.max,
    },
  );
  plot.drawFrame();
  plot.ctx.save();
  plot.ctx.font = canvasFont(plot.theme, 9);
  plot.ctx.fillStyle = plot.theme.axis;
  plot.ctx.textAlign = "left";
  plot.ctx.fillText(formatAxisRangeSummary(resolvedRanges), plot.plotRect.x0 + 4, plot.margin.top - 7);
  plot.ctx.textAlign = "right";
  plot.ctx.fillText(
    `${formatPlotRangePolicySummary(population, {
      statistics: true,
      fitInputs: line && pickPopulation ? pickPopulation.analysisCount : null,
    })}${plotZ && !style?.zRange ? " · Z colour refused: no governed range" : ""}${colourPolicy?.excluded ? ` · Z excluded=${colourPolicy.excluded}` : ""}${colourPolicy?.clamped ? ` · Z clamped/edge-marked=${colourPolicy.clamped}` : ""}`,
    plot.plotRect.x0 + plot.plotRect.w,
    plot.plotRect.y0 + plot.plotRect.h + 31,
  );
  plot.ctx.restore();

  // Context wells first, faded, so the active well's cloud reads on top of them.
  const hasCtx = !!context && contextLayers.length > 0;
  if (hasCtx) {
    const { ctx } = plot;
    ctx.save();
    ctx.globalAlpha = 0.4;
    for (const layer of contextLayers) {
      plot.drawScatter(layer.rt, layer.phi, layer.color, style?.pointSize ?? 1.8);
    }
    ctx.restore();
  }
  plot.drawScatter(plotRt, plotPhi, plotColors, style?.pointSize ?? 1.8);
  if (colourPolicy?.clamped) {
    const edgePoints = (edge: PlotRangeEdge): [Float32Array, Float32Array] => {
      const xs: number[] = [];
      const ys: number[] = [];
      for (let index = 0; index < colourPolicy.edgeMarks.length; index++) {
        if (colourPolicy.edgeMarks[index] !== edge) continue;
        xs.push(plotRt[index]);
        ys.push(plotPhi[index]);
      }
      return [Float32Array.from(xs), Float32Array.from(ys)];
    };
    const [lowXs, lowYs] = edgePoints("low");
    const [highXs, highYs] = edgePoints("high");
    plot.drawDiamonds(lowXs, lowYs, plot.theme.accent2, Math.max(2.5, (style?.pointSize ?? 1.8) + 1));
    plot.drawDiamonds(highXs, highYs, plot.theme.warn, Math.max(2.5, (style?.pointSize ?? 1.8) + 1));
  }

  if (line) {
    // The fitted Sw=1 trend needs only m and the identifiable product a·Rw. Saturation
    // guides remain absent until a, m, n and Rw are all supplied with provenance.
    const phiLo = Math.min(plot.y.min, plot.y.max);
    const phiHi = Math.max(plot.y.min, plot.y.max);
    const fittedLine: [number, number][] = [phiLo, phiHi].map((phiV) => [
      line.aRw / Math.pow(phiV, line.m),
      phiV,
    ]);
    plot.drawLine(fittedLine, plot.theme.accent, 2);

    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.font = canvasFont(plot.theme, 10);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(`Fitted line: M = ${line.m.toFixed(2)}, a·Rw = ${line.aRw.toPrecision(3)} ohmm`, r.x0 + 8, r.y0 + 14);
    ctx.fillText(`a and Rw are not separately identified${hasCtx ? " — line = ACTIVE well's fit" : ""}`, r.x0 + 8, r.y0 + 27);
    ctx.restore();
  }

  // Well legend for the multi-well overlay (top-right; the water-line readout owns
  // top-left). The footer states the contract on the plot itself.
  if (hasCtx) {
    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    const rowH = 15;
    const boxW = 150;
    const boxX = r.x0 + r.w - boxW - 8;
    let boxY = r.y0 + 8;
    ctx.font = canvasFont(plot.theme, 10, 600);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText("Wells", boxX, boxY + 9);
    boxY += rowH;
    ctx.font = canvasFont(plot.theme, 10);
    const row = (color: string | null, label: string) => {
      if (color) {
        ctx.fillStyle = color;
        ctx.fillRect(boxX, boxY + 1, 11, 11);
        ctx.strokeStyle = plot.theme.text;
        ctx.lineWidth = 0.5;
        ctx.strokeRect(boxX, boxY + 1, 11, 11);
      }
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(label, boxX + 16, boxY + 10);
      boxY += rowH;
    };
    // The active swatch only means something when the cloud is one color (no Z coloring).
    const legend = contextLegend(context!.activeName, context!.layers);
    row(plotColors ? null : plot.theme.accent, `${legend.activeName} (active${plotColors ? ", by Z" : ""})`);
    for (const entry of legend.rows) row(entry.color, entry.name);
    if (legend.remainder) {
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(legend.remainder, boxX + 16, boxY + 10);
      boxY += rowH;
    }
    ctx.font = canvasFont(plot.theme, 9);
    ctx.fillText("context is display-only", boxX, boxY + 9);
    ctx.restore();
  }

  // Picked anchor points.
  for (const [rtV, phiV] of picks) {
    const [px, py] = plot.toPx(rtV, phiV);
    const { ctx } = plot;
    ctx.save();
    ctx.strokeStyle = plot.theme.accent;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  }

  // Synchronized hover: ring the sample at the depth under another view's cursor.
  const eligibleHoverIndex = preliminary.indices.indexOf(hoverIdx);
  if (eligibleHoverIndex >= 0) {
    const hr = plotRt[eligibleHoverIndex];
    const hp = plotPhi[eligibleHoverIndex];
    if (!Number.isNaN(hr) && !Number.isNaN(hp) && hr > 0 && hp > 0) {
      const [px, py] = plot.toPx(hr, hp);
      const { ctx } = plot;
      ctx.save();
      ctx.strokeStyle = plot.theme.warn;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(px, py, 5.5, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    }
  }
  return plot;
}

/** Pickett plot dialog (v2): log-log RT vs PHIE. Click two points along the water-bearing
 *  trend to fit the Sw=1 line (M, Rw), OR type M / Rw directly and the lines follow. N, M and
 *  Rw are all editable in the toolbar. A Properties dialog (⚙ / right-click) sets the axis
 *  ranges, point size, and optional Z-coloring by a chosen log, persisted via plotprops. */
export async function buildPickettContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well, { followSelectedInterval: false });
  trySelect(zoneSel.select, initial?.zone);
  const plotId = initial?.plotId ?? crypto.randomUUID();
  const props: PickettProps = sanitizePickettProps(await loadPlotProps<PickettProps>("pickett"));

  const numField = (value: string, placeholder = ""): HTMLInputElement => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = "any";
    i.style.width = "72px";
    i.placeholder = placeholder;
    i.value = value;
    return i;
  };

  const content = document.createElement("div");
  content.className = "plot-content";
  const activeDepthHandoff = buildDepthReframeHandoff(setStatus);
  const contextDepthHandoff = buildDepthReframeHandoff(setStatus);
  const rtSel = curveSelect(curveNames, initial?.rt ?? "RES_DEEP");
  const phiSel = curveSelect(curveNames, initial?.phi ?? "PHIE");
  // Manual M / a·Rw: blank = derive from the two picks; typed = the fitted line follows them.
  const mIn = numField(initial?.m ?? "", "pick");
  const aRwIn = numField(initial?.aRw ?? "", "pick");

  const propsBtn = document.createElement("button");
  propsBtn.className = "form-control";
  propsBtn.textContent = "⚙";
  propsBtn.title = "Pickett properties — axes, point size, Z-color (or right-click the plot)";
  propsBtn.addEventListener("click", () => openProps());

  // --- Multi-well scope: extra wells drawn as faded clouds BEHIND the active well's.
  // The fitted product line, picks, brushing and tooltips stay bound to the ACTIVE well.
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
  scopeBtn.title = "Overlay more wells' clouds behind the active well — the fitted line stays the active well's";
  const scopeRow = document.createElement("div");
  scopeRow.style.display = "none";
  const scopeStaticHint = document.createElement("p");
  scopeStaticHint.className = "modal-hint";
  scopeStaticHint.textContent =
    "Context wells are display-only: the fitted M and a·Rw line, water-line picks, brushing and tooltips all belong to the " +
    "active well. a and Rw are not separately identified, and saturation guides need independently sourced parameters. " +
    "Zone/top windows are resolved per well by NAME (a well without that zone or top is skipped).";
  const scopeInfo = document.createElement("p");
  scopeInfo.className = "modal-hint";
  scopeRow.append(scope.el, scopeStaticHint, scopeInfo, contextDepthHandoff.el);
  scopeBtn.addEventListener("click", () => {
    scopeRow.style.display = scopeRow.style.display === "none" ? "" : "none";
  });
  const updateScopeUi = () => {
    scopeBtn.textContent = `Wells: ${scope.describe()}`;
    scopeInfo.textContent = ctxInfo;
    scopeInfo.style.display = ctxInfo ? "" : "none";
  };
  const plotIntents = () => [
    { channel: "resistivity", semantic_request: rtSel.value, required: true },
    { channel: "porosity", semantic_request: phiSel.value, required: true },
    ...(props.zCurve ? [{ channel: "colour", semantic_request: props.zCurve, required: false }] : []),
  ];
  const representedWellIds = () => [well.well_id, ...ctxWellIds];
  let axisRanges: PlotAxisRangeExport[] = [];
  const currentAxisBindings = (): { resistivity: PlotChannelBinding | null; porosity: PlotChannelBinding | null; colour: PlotChannelBinding | null } => {
    const bindings = plotBindingSnapshotForChannels(representedWellIds(), plotIntents());
    return {
      resistivity: bindings.find((binding) => binding.intent.channel === "resistivity") ?? null,
      porosity: bindings.find((binding) => binding.intent.channel === "porosity") ?? null,
      colour: bindings.find((binding) => binding.intent.channel === "colour") ?? null,
    };
  };
  const selectionState = (): Record<string, string> => ({
    plotId,
    rt: rtSel.value,
    phi: phiSel.value,
    m: mIn.value,
    aRw: aRwIn.value,
    zone: zoneSel.select.value,
    wells: scope.serialize(),
  });
  const persistedState = (options: Record<string, unknown>) =>
    buildPersistedPlotState("pickett", options, representedWellIds(), plotIntents(), axisRanges);
  const persist = () => {
    try {
      void savePlotProps("pickett", persistedState({ ...props }))
        .catch((error) => setStatus(`Pickett state not saved: ${error}`));
    } catch (error) {
      setStatus(`Pickett state not saved: ${error}`);
    }
  };

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("RT", rtSel));
  selRow.appendChild(formRow("Porosity", phiSel));
  selRow.appendChild(formRow("M", mIn));
  selRow.appendChild(formRow("a·Rw", aRwIn));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(scopeBtn);
  selRow.appendChild(propsBtn);
  selRow.appendChild(
    buildPlotTemplateBar<PickettProps>(
      "pickett",
      "Pickett",
      () => persistedState({ ...props }),
      (t) => {
        Object.assign(props, sanitizePickettProps({ ...props, ...t }));
        viewRef.current = null; // show the template's axis ranges (a live zoom would mask them)
        void reload(true).then(persist); // refetch first so the saved tier/range matches the rendered plot
      },
      setStatus,
    ),
  );
  const exportGroup = buildImageExportButtons(
    () => canvas,
    "Pickett",
    setStatus,
    (scope) => getSvg(scope),
    (scope) => getPdf(scope),
    () => ctxReductionManifest,
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
  selRow.appendChild(exportGroup);
  content.appendChild(selRow);
  content.appendChild(scopeRow);
  content.appendChild(activeDepthHandoff.el);

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 460;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  hint.textContent =
    "Click TWO points along the water-bearing (lowest-RT) trend to fit M and the identifiable product a·Rw — or type those values " +
    "directly. The fit does not identify a or Rw separately. Ctrl+wheel = zoom, drag = pan, double-click = reset zoom, right-click = properties. " +
    "Needs a computed porosity curve (run a Porosity module first).";
  content.appendChild(hint);
  const statisticsInfo = document.createElement("p");
  statisticsInfo.className = "modal-hint";
  statisticsInfo.style.whiteSpace = "pre-wrap";
  content.appendChild(statisticsInfo);

  const tc = readTheme(document.documentElement);
  const pickM = pickRow("M (slope)", tc.accent, "M", well, zoneSel.current, setStatus, () => pickettWriteSource());
  const pickARw = pickRow("a·Rw (intercept)", tc.accent2, "A_RW", well, zoneSel.current, setStatus, () => pickettWriteSource());
  content.appendChild(pickM.row);
  content.appendChild(pickARw.row);

  let rt = new Float32Array(0);
  let phi = new Float32Array(0);
  let depths = new Float32Array(0);
  let zValues = new Float32Array(0);
  let zRange: PlotAxisRangeExport | null = null;
  let picks: [number, number][] = [];
  let plot: PlotCanvas | null = null;
  let lastFit: { m: number; aRw: number } | null = null;
  let hoverIdx = -1;
  let statisticsRecords: PlotStatisticsRecord[] = [];
  let statisticsSignature = "";
  let statisticsDataVersion = 0;
  // Linked-brush consumer: samples brushed in the crossplot (same well, same backend depth
  // grid) are ringed here so a selection made in one plot is visible in the other.
  const initialBrush = appState.brushedDepths.get();
  let brushSet: Set<number> | null = initialBrush?.wellId === well.well_id ? initialBrush.depths : null;
  const viewRef: ViewportRef = { current: null };

  const refreshStatisticsRecords = (): void => {
    if (!plot) {
      statisticsSignature = "";
      statisticsRecords = [];
      statisticsInfo.textContent = "No governed statistics population.";
      return;
    }
    const zone = zoneSel.current();
    const brush = appState.brushedDepths.get();
    const selectionLabel = brush && brush.wellId === well.well_id
      ? "all eligible; current brush not applied"
      : "all eligible";
    const signature = JSON.stringify([
      statisticsDataVersion,
      rtSel.value,
      phiSel.value,
      zone.depthMin,
      zone.depthMax,
      props.validityFilter,
      props.rtValidMin,
      props.rtValidMax,
      props.phiValidMin,
      props.phiValidMax,
      plot.x.min,
      plot.x.max,
      plot.y.min,
      plot.y.max,
      selectionLabel,
    ]);
    if (signature === statisticsSignature) return;
    statisticsRecords = buildPickettStatisticsRecords(
      rt,
      phi,
      {
        validityFilter: props.validityFilter,
        rtValidMin: props.rtValidMin,
        rtValidMax: props.rtValidMax,
        phiValidMin: props.phiValidMin,
        phiValidMax: props.phiValidMax,
      },
      rtSel.value,
      phiSel.value,
      well.well_id,
      zone.depthMin,
      zone.depthMax,
      { min: plot.x.min, max: plot.x.max },
      { min: plot.y.min, max: plot.y.max },
      selectionLabel,
    );
    statisticsSignature = signature;
    statisticsInfo.textContent = statisticsRecords.length > 0
      ? statisticsRecords.map((record) => formatPlotStatisticsRecord(record)).join("\n")
      : "No governed statistics population.";
  };

  const resolvedBinding = (curveName: string): ResolvedPlotCurve | null =>
    plotBindingSnapshot([well.well_id], [curveName])
      .find((binding) => binding.intent.semantic_request.toUpperCase() === curveName.toUpperCase())
      ?.resolved[0] ?? null;

  async function pickettWriteSource(): Promise<PlotWriteSource> {
    if (!plot) throw new Error("plot-derived write requires a rendered viewport");
    const zone = zoneSel.current();
    return {
      plot_id: plotId,
      plot_type: "pickett",
      x_axis: plotWriteAxis("resistivity", resolvedBinding(rtSel.value)),
      y_axis: plotWriteAxis("porosity", resolvedBinding(phiSel.value)),
      z_axis: props.zCurve ? plotWriteAxis("colour", resolvedBinding(props.zCurve)) : null,
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
      method: lastFit ? "two_point_pickett_fit" : "manual_pickett_value",
      fit_record: lastFit
        ? { model: "two_point_pickett_water_line", m: lastFit.m, a_rw: lastFit.aRw }
        : null,
    };
  }

  /** The effective fitted line: typed M+a·Rw win; otherwise none until two points are picked. */
  const currentLine = (): { m: number; aRw: number } | null => {
    const m = parseFloat(mIn.value);
    const aRw = parseFloat(aRwIn.value);
    if (Number.isFinite(m) && Number.isFinite(aRw) && m > 0 && aRw > 0) return { m, aRw };
    return null;
  };

  const eligibleSampleIndices = (): Set<number> => new Set(
    screenPickettPopulation(rt, phi, {
      validityFilter: props.validityFilter,
      rtValidMin: props.rtValidMin,
      rtValidMax: props.rtValidMax,
      phiValidMin: props.phiValidMin,
      phiValidMax: props.phiValidMax,
    }, null, null).indices,
  );

  const resolveColourRange = (z: Float32Array): PlotAxisRangeExport | null => {
    let min = Infinity;
    let max = -Infinity;
    for (const value of z) {
      if (!Number.isFinite(value) || (props.zLog && value <= 0)) continue;
      min = Math.min(min, value);
      max = Math.max(max, value);
    }
    const finiteData = Number.isFinite(min) && Number.isFinite(max) && min !== max
      ? { min, max }
      : null;
    const resolved = resolveBoundAxisRange({
      binding: currentAxisBindings().colour,
      user: null,
      finiteData,
      log: props.zLog,
    });
    return resolved ? axisRangeExportRecord("colour", resolved) : null;
  };

  // --- Context-well data (multi-well overlay) — same budget rule as crossplot/histogram.
  let ctxLayers: PickettContextLayer[] = [];
  let ctxReductionManifest: PlotReductionExport | null = null;
  let ctxWellIds: string[] = [];
  let ctxInfo = "";
  const pickettContext = (): PickettContext | null =>
    ctxLayers.length ? { activeName: well.well_name, layers: ctxLayers } : null;

  /** Fetches the scoped context wells' RT/porosity through the shared plotCommon machinery
   *  (per-well zone/top-by-name windows, point budget, cancellation). Scope = just the
   *  active well → clears the overlay: byte-identical single-well behaviour. */
  const { reload: reloadContext, cancel: cancelContextReload } =
    createContextReload<PickettContextLayer>({
      kind: "pickett",
      label: "Pickett",
      operation: "pickett-context-refetch",
      well,
      scope,
      zoneSel,
      handoff: contextDepthHandoff,
      curves: () => [rtSel.value, phiSel.value],
      project: (layer) => ({
        name: layer.name,
        color: layer.color,
        rt: layer.series.get(rtSel.value.toUpperCase())!,
        phi: layer.series.get(phiSel.value.toUpperCase())!,
      }),
      hadLayers: () => ctxLayers.length > 0,
      apply: (next) => {
        ctxLayers = next.layers;
        ctxWellIds = next.wellIds;
        ctxReductionManifest = next.reductionManifest;
        ctxInfo = next.info;
      },
      setPendingManifest: (manifest) => {
        ctxReductionManifest = manifest;
      },
      setStatus: (text) => setStatus(text),
      updateScopeUi: () => updateScopeUi(),
      redraw: () => redraw(),
    });
  updateScopeUi();

  const redraw = () => {
    canvas.setAttribute("aria-label", `Pickett plot: ${rtSel.value} versus ${phiSel.value}`); // a11y label
    axisRanges = [];
    plot = drawPickett(canvas, rt, phi, currentLine(), picks, hoverIdx, viewRef.current, {
      rtMin: props.rtMin,
      rtMax: props.rtMax,
      phiMin: props.phiMin,
      phiMax: props.phiMax,
      pointSize: props.pointSize,
      zValues: props.zCurve ? zValues : undefined,
      zRange,
      zLog: props.zLog,
      colormap: props.colormap,
      validityFilter: props.validityFilter,
      rtValidMin: props.rtValidMin,
      rtValidMax: props.rtValidMax,
      phiValidMin: props.phiValidMin,
      phiValidMax: props.phiValidMax,
    }, pickettContext(), currentAxisBindings(), (ranges) => {
      axisRanges = ranges;
    });
    if (!plot) {
      refreshStatisticsRecords();
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const theme = readTheme(canvas);
      ctx.font = canvasFont(theme, 12);
      ctx.fillStyle = theme.text;
      ctx.textAlign = "center";
      ctx.fillText("No finite positive data or governed display range for both Pickett axes.", canvas.width / 2, canvas.height / 2);
      return;
    }
    refreshStatisticsRecords();
    // Ring the samples brushed in the crossplot. Depths come off the same backend grid, so an
    // exact Set membership test aligns them; clipped to the plot and skipping log-invalid points.
    if (plot && brushSet && brushSet.size && depths.length === rt.length) {
      const { ctx } = plot;
      const eligible = eligibleSampleIndices();
      const rp = plot.plotRect;
      ctx.save();
      ctx.beginPath();
      ctx.rect(rp.x0, rp.y0, rp.w, rp.h);
      ctx.clip();
      ctx.strokeStyle = plot.theme.accent2;
      ctx.lineWidth = 1.5;
      const rad = Math.max(3, props.pointSize + 1.6);
      for (let i = 0; i < depths.length; i++) {
        if (!eligible.has(i)) continue;
        if (!brushSet.has(depths[i])) continue;
        const rv = rt[i];
        const pv = phi[i];
        if (!(rv > 0) || !(pv > 0)) continue; // both axes are log
        const [px, py] = plot.toPx(rv, pv);
        ctx.beginPath();
        ctx.arc(px, py, rad, 0, Math.PI * 2);
        ctx.stroke();
      }
      ctx.restore();
    }
  };

  // Vector export: re-run the same static draw (no hover ring, no brush) into a recording
  // context sized to the live plot, so the SVG matches what's on screen.
  // The static draw shared by the two vector-export paths (no hover ring, no brush).
  const drawStatic = (c: HTMLCanvasElement) =>
    drawPickett(c, rt, phi, currentLine(), picks, -1, viewRef.current, {
      rtMin: props.rtMin,
      rtMax: props.rtMax,
      phiMin: props.phiMin,
      phiMax: props.phiMax,
      pointSize: props.pointSize,
      zValues: props.zCurve ? zValues : undefined,
      zRange,
      zLog: props.zLog,
      colormap: props.colormap,
      validityFilter: props.validityFilter,
      rtValidMin: props.rtValidMin,
      rtValidMax: props.rtValidMax,
      phiValidMin: props.phiValidMin,
      phiValidMax: props.phiValidMax,
    }, pickettContext(), currentAxisBindings(), (ranges) => {
      axisRanges = ranges;
    });
  const getSvg = (scope: PlotAncestryScope): string | null =>
    plot ? renderPlotToPaperSvg(plot.width, plot.height, drawStatic, scope) : null;
  const getPdf = (scope: PlotAncestryScope): PlotPdf | null =>
    plot ? renderPlotToPaperPdf(plot.width, plot.height, drawStatic, scope) : null;

  // Monotonic token so a slow curve/zone load that resolves after a newer one (fast
  // switching) can't overwrite the newer data. `preserveView` keeps the zoom/pan AND the
  // user's M/a·Rw line on a data refresh (module run); a user-initiated curve/zone change
  // re-fits from scratch and clears the picks + typed line.
  let reloadGen = 0;
  let resetPending = false;
  const reload = async (preserveView = false) => {
    const token = beginPlotAsyncGeneration("pickett-data-refetch", ++reloadGen);
    if (!preserveView) resetPending = true;
    const zone = zoneSel.current();
    const names = props.zCurve ? [rtSel.value, phiSel.value, props.zCurve] : [rtSel.value, phiSel.value];
    activeDepthHandoff.clear();
    try {
      const series = await getCurveData(well.well_id, names, zone.depthMin, zone.depthMax);
      if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return;
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      const required = names.map((name) => byName.get(name.toUpperCase()));
      if (required.some((item) => !item)) throw new Error("one or more required plot curves are absent");
      const reconciled = reconcileDepthChannels(required.map((item) => ({
        depth: item!.depth,
        values: item!.value,
      })));
      rt = reconciled.channels[0];
      phi = reconciled.channels[1];
      depths = reconciled.depth;
      zValues = props.zCurve ? reconciled.channels[2] : new Float32Array(0);
      const zPopulation = props.zCurve
        ? screenPickettPopulation(rt, phi, {
            validityFilter: props.validityFilter,
            rtValidMin: props.rtValidMin,
            rtValidMax: props.rtValidMax,
            phiValidMin: props.phiValidMin,
            phiValidMax: props.phiValidMax,
          }, null, null).indices
        : [];
      zRange = props.zCurve
        ? resolveColourRange(Float32Array.from(zPopulation.map((index) => zValues[index])))
        : null;
      if (reconciled.mode === "decimated_to_coarsest") {
        setStatus(`Pickett depth inputs decimated to the coarsest exact step; factors ${reconciled.decimationFactors.join("/")} · interval ${reconciled.intervalClosure}`);
      }
    } catch (err) {
      if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return;
      const handoff = depthReframeHandoff(err, [well.well_id], names);
      activeDepthHandoff.show(handoff);
      setStatus(handoff ? `Pickett refused: ${handoff.reason}` : `Pickett data load failed: ${err}`);
      rt = phi = depths = new Float32Array(0);
      zValues = new Float32Array(0);
      zRange = null;
    }
    statisticsDataVersion++;
    hoverIdx = -1; // the old hover index may point at a different sample now
    if (resetPending) {
      resetPending = false;
      picks = [];
      lastFit = null;
      mIn.value = "";
      aRwIn.value = "";
      viewRef.current = null; // new data → reset any zoom/pan
    }
    redraw();
  };

  for (const sel of [rtSel, phiSel, zoneSel.select]) {
    sel.addEventListener("change", () => {
      void reload();
      void reloadContext(); // context wells share the RT/porosity curves and the zone window
    });
  }
  // Typing M or a·Rw makes the fitted line follow immediately.
  mIn.addEventListener("input", () => {
    lastFit = null;
    redraw();
  });
  aRwIn.addEventListener("input", () => {
    lastFit = null;
    redraw();
  });

  // Track drag so a pan doesn't also drop a water-line anchor. Logical (CSS) pixels.
  let downXY: [number, number] | null = null;
  let movedSinceDown = false;
  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    const rect = canvas.getBoundingClientRect();
    downXY = [e.clientX - rect.left, e.clientY - rect.top];
    movedSinceDown = false;
  });
  canvas.addEventListener("mousemove", (e) => {
    if (!downXY) return;
    const rect = canvas.getBoundingClientRect();
    if (Math.hypot(e.clientX - rect.left - downXY[0], e.clientY - rect.top - downXY[1]) > 4) movedSinceDown = true;
  });

  canvas.addEventListener("click", (e) => {
    downXY = null;
    if (!plot || movedSinceDown) return; // tail of a pan, not an anchor pick
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    if (!plot.inPlot(px, py)) return;
    const [rtV, phiV] = plot.toData(px, py);
    const pickedPopulation = screenPickettPopulation(
      Float32Array.of(rtV),
      Float32Array.of(phiV),
      {
        validityFilter: props.validityFilter,
        rtValidMin: props.rtValidMin,
        rtValidMax: props.rtValidMax,
        phiValidMin: props.phiValidMin,
        phiValidMax: props.phiValidMax,
      },
      null,
      null,
    );
    if (pickedPopulation.analysisCount === 0) {
      setStatus("Pickett fit input refused: the picked point is outside the active validity range");
      return;
    }

    if (picks.length >= 2) picks = [];
    picks.push([rtV, phiV]);
    if (picks.length === 2) {
      const fit = fitWaterLine(picks[0], picks[1]);
      if (fit) {
        lastFit = fit;
        // Feed the identifiable fit into the M/a·Rw fields so both paths share one source.
        mIn.value = fit.m.toPrecision(4);
        aRwIn.value = fit.aRw.toPrecision(4);
        pickM.setValue(fit.m);
        pickARw.setValue(fit.aRw);
        setStatus(`Water line: M = ${fit.m.toFixed(2)}, a·Rw = ${fit.aRw.toPrecision(3)} ohmm; a and Rw are not separately identified`);
      } else {
        setStatus("Could not fit a water line from those two points — pick points at different porosities.");
      }
    }
    redraw();
  });

  // Right-click belongs to the pane context menu (Properties… is its first entry), not to
  // the canvas. Left-click stays reserved for water-line picks, so — unlike the other
  // plots — double-click is not overloaded here either; the ⚙ toolbar button is the
  // direct route to Properties.

  makeCanvasAccessible(canvas, `Pickett plot: ${rtSel.value} versus ${phiSel.value}`);
  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw });
  const detachKeys = attachKeyboardPanZoom({
    canvas,
    getPlot: () => plot,
    view: viewRef,
    redraw,
    getLabel: () => `Pickett plot: ${rtSel.value} versus ${phiSel.value}`,
    openProperties: () => openProps(),
    focusExport: () => exportGroup.querySelector<HTMLButtonElement>("button")?.focus(),
  });

  // Synchronized hover: ring the sample nearest the depth under any log view's cursor.
  let rafId = 0;
  const unsubHover = appState.hoverDepth.subscribe((depth) => {
    const idx = depth === null ? -1 : nearestDepthIndex(depths, depth);
    if (idx === hoverIdx) return;
    hoverIdx = idx;
    if (!rafId) {
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        redraw();
      });
    }
  });

  /** Properties dialog: axis ranges, point size, and Z-color-by-curve. Persisted per plot
   *  kind (plotprops "pickett") like Histogram/Crossplot v2. */
  const openProps = () => {
    const body = document.createElement("div");
    const num = (value: number | null, w = 72): HTMLInputElement => {
      const i = document.createElement("input");
      i.className = "form-control";
      i.type = "number";
      i.step = "any";
      i.style.width = `${w}px`;
      i.value = value === null ? "" : String(value);
      return i;
    };
    const inline = (...els: (HTMLElement | string)[]): HTMLElement => {
      const wrap = document.createElement("div");
      wrap.style.display = "flex";
      wrap.style.gap = "8px";
      wrap.style.alignItems = "center";
      wrap.style.flexWrap = "wrap";
      wrap.append(...els);
      return wrap;
    };

    const rtMinI = num(props.rtMin);
    const rtMaxI = num(props.rtMax);
    const phiMinI = num(props.phiMin);
    const phiMaxI = num(props.phiMax);
    const psI = num(props.pointSize, 56);
    const rtValidMinI = num(props.rtValidMin);
    const rtValidMaxI = num(props.rtValidMax);
    const phiValidMinI = num(props.phiValidMin);
    const phiValidMaxI = num(props.phiValidMax);
    const validityWrap = document.createElement("label");
    validityWrap.className = "chk-field";
    const validityChk = document.createElement("input");
    validityChk.type = "checkbox";
    validityChk.checked = props.validityFilter;
    validityWrap.append(validityChk, document.createTextNode("Apply validity ranges to n and fit inputs"));

    const zSel = document.createElement("select");
    zSel.className = "form-control";
    for (const [v, label] of [["", "— None —"] as [string, string], ...curveNames.map((c) => [c, c] as [string, string])]) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      zSel.appendChild(o);
    }
    zSel.value = props.zCurve;

    const cmSel = document.createElement("select");
    cmSel.className = "form-control";
    for (const [v, label] of [["rainbow", "Rainbow"], ["viridis", "Viridis"]] as [string, string][]) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      cmSel.appendChild(o);
    }
    cmSel.value = props.colormap;

    const zLogWrap = document.createElement("label");
    zLogWrap.className = "chk-field";
    const zLogChk = document.createElement("input");
    zLogChk.type = "checkbox";
    zLogChk.checked = props.zLog;
    zLogWrap.append(zLogChk, document.createTextNode("log Z"));

    body.appendChild(formRow("RT axis", inline(rtMinI, "→", rtMaxI), "Blank = header display, then audited unit-family display, then finite positive data"));
    body.appendChild(formRow("PHIE axis", inline(phiMinI, "→", phiMaxI), "Both limits are required for a user override"));
    body.appendChild(validityWrap);
    body.appendChild(formRow("RT valid", inline(rtValidMinI, "to", rtValidMaxI), "Optional; both limits required when supplied"));
    body.appendChild(formRow("PHIE valid", inline(phiValidMinI, "to", phiValidMaxI), "Validity changes n and fit inputs; display clipping does not"));
    body.appendChild(formRow("Point size", psI));
    body.appendChild(formRow("Color by", zSel));
    body.appendChild(formRow("Colormap", inline(cmSel, zLogWrap)));

    const applyBtn = document.createElement("button");
    applyBtn.className = "form-run-btn";
    applyBtn.textContent = "Apply";
    const btnRow = document.createElement("div");
    btnRow.className = "form-row";
    btnRow.appendChild(applyBtn);
    body.appendChild(btnRow);

    const close = openModal("Pickett properties", body, 340);
    applyBtn.addEventListener("click", () => {
      const rangePair = (low: HTMLInputElement, high: HTMLInputElement, label: string): [number | null, number | null] | null => {
        const lowText = low.value.trim();
        const highText = high.value.trim();
        if (!lowText && !highText) return [null, null];
        const parsedLow = Number(lowText);
        const parsedHigh = Number(highText);
        if (!lowText || !highText || !Number.isFinite(parsedLow) || !Number.isFinite(parsedHigh) || parsedLow <= 0 || parsedHigh <= 0 || parsedLow === parsedHigh) {
          setStatus(`${label} requires two distinct positive limits, or two blanks`);
          return null;
        }
        return [parsedLow, parsedHigh];
      };
      const rtPair = rangePair(rtMinI, rtMaxI, "RT axis");
      const phiPair = rangePair(phiMinI, phiMaxI, "PHIE axis");
      const rtValidPair = rangePair(rtValidMinI, rtValidMaxI, "RT validity");
      const phiValidPair = rangePair(phiValidMinI, phiValidMaxI, "PHIE validity");
      if (!rtPair || !phiPair || !rtValidPair || !phiValidPair) return;
      if (validityChk.checked && rtValidPair[0] === null && phiValidPair[0] === null) {
        setStatus("Pickett validity requires at least one complete RT or PHIE range before it can be enabled");
        return;
      }
      [props.rtMin, props.rtMax] = rtPair;
      [props.phiMin, props.phiMax] = phiPair;
      [props.rtValidMin, props.rtValidMax] = rtValidPair;
      [props.phiValidMin, props.phiValidMax] = phiValidPair;
      props.validityFilter = validityChk.checked;
      props.pointSize = Math.max(0.5, parseFloat(psI.value) || PICKETT_DEFAULTS.pointSize);
      props.zCurve = zSel.value;
      props.colormap = cmSel.value as ColormapName;
      props.zLog = zLogChk.checked;
      if (lastFit && picks.length === 2) {
        const retainedPicks = screenPickettPopulation(
          Float32Array.from(picks.map((pick) => pick[0])),
          Float32Array.from(picks.map((pick) => pick[1])),
          {
            validityFilter: props.validityFilter,
            rtValidMin: props.rtValidMin,
            rtValidMax: props.rtValidMax,
            phiValidMin: props.phiValidMin,
            phiValidMax: props.phiValidMax,
          },
          null,
          null,
        );
        if (retainedPicks.analysisCount !== 2) {
          picks = [];
          lastFit = null;
          mIn.value = "";
          aRwIn.value = "";
          setStatus("Pickett fit cleared because an anchor is outside the active validity range");
        }
      }
      viewRef.current = null; // show the new axis ranges (a live zoom would otherwise mask them)
      void reload(true).then(persist); // refetch first so the saved tier/range matches the rendered plot
      close();
    });
  };

  // Local hover tooltip: the Rt / porosity / depth of the sample under the cursor. Suppressed
  // while a button is down (pan or the tail of a water-line pick).
  const detachTip = attachScatterTooltip(canvas, (px, py) => {
    if (downXY || !plot || !plot.inPlot(px, py)) return null;
    const eligible = eligibleSampleIndices();
    let best = -1;
    let bestD = 12 * 12; // within a 12 px radius
    for (let i = 0; i < rt.length; i++) {
      if (!eligible.has(i)) continue;
      const vx = rt[i];
      const vy = phi[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (plot.x.log && vx <= 0) continue;
      if (plot.y.log && vy <= 0) continue;
      const [sx, sy] = plot.toPx(vx, vy);
      const d = (sx - px) * (sx - px) + (sy - py) * (sy - py);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    if (best < 0) return null;
    const lines: string[] = [];
    if (best < depths.length && Number.isFinite(depths[best])) lines.push(`${depths[best].toFixed(1)} m`);
    lines.push(`${rtSel.value}  ${fmtValue(rt[best])}`);
    lines.push(`${phiSel.value}  ${fmtValue(phi[best])}`);
    return lines;
  });

  const invalidation = registerPlotInvalidationContract(canvas, {
    theme: () => redraw(),
    dataRevision: () => {
      // Keep the pan/zoom and M/a·Rw line while active and context data refresh together.
      void reload(true);
      void reloadContext();
    },
    interval: (interval) => zoneSel.applySelectedInterval(interval, true),
    selection: (selection) => {
      const next = selection?.wellId === well.well_id ? selection.depths : null;
      if (next === brushSet) return;
      brushSet = next;
      redraw();
    },
    size: () => redraw(),
    cancelPending: () => {
      reloadGen++;
      cancelContextReload();
      if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = 0;
      }
    },
  });

  await reload();
  // Not awaited: a big scope must not block the panel build — the active well's plot
  // appears immediately and the context clouds fade in when ready.
  const bindingReady = reloadContext();
  return {
    el: content,
    dispose: () => {
      invalidation.dispose();
      scope.dispose();
      unsubHover();
      detachZoomPan();
      detachKeys.dispose();
      detachTip();
      zoneSel.dispose();
    },
    getState: selectionState,
    getPersistedState: () => persistedState(selectionState()),
    bindingReady,
    openProperties: openProps,
  };
}
