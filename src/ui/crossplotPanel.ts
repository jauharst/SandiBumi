import { getCoreData, getCurveData, plotBindingSnapshot, plotBindingSnapshotForChannels, resolveWellScope, runNetFlag, type NetFlagSpec, type PlotAncestryScope, type PlotChannelBinding, type ResolvedPlotCurve, type TrackCurveSeries, type WellSummary } from "../ipc";
import {
  HISTOGRAM_BINS_DEFAULT,
  HISTOGRAM_BINS_MAX,
  HISTOGRAM_BINS_MIN,
  canonicalHistogram,
  normalizeHistogramBinCount,
  type HistogramContract,
} from "../distribution";
import { appState, bumpDataVersion, clearBrush, setBrushedDepths, type BrushSelection } from "../state";
import { formRow, openModal } from "./modal";
import { requestRunCustody } from "./runCustody";
import {
  attachKeyboardPanZoom,
  attachScatterTooltip,
  attachZoomPan,
  buildPlotStatisticsRecord,
  categoricalColors,
  colorRampEx,
  distinctValues,
  drawColorbar,
  faciesColor,
  faciesLabel,
  fitCanvasBackingStore,
  formatPlotStatisticsRecord,
  fmtValue,
  looksDiscrete,
  makeCanvasAccessible,
  percentile,
  plotStatisticsInterval,
  PlotCanvas,
  canvasFont,
  readTheme,
  type ColormapName,
  type PlotStatisticsRecord,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import { parsePercentiles } from "./histogramPanel";
import { AXIS_ALIASES, type AxisKind, type ChartOverlayDef } from "./chartOverlays";
// Resolution goes through the policy module: a route-2 derived overlay (DEC-078) shadows
// its digitized coordinates the moment its derivation lands.
import { allChartOverlays, resolveChartOverlay } from "./chartOverlayPolicy";
import {
  applyPlotChannelPolicy,
  reconcileDepthChannels,
  type PlotRangeEdge,
  type PlotReductionExport,
} from "./plotTypes";
import {
  buildPlotTemplateBar,
  buildPersistedPlotState,
  buildDepthReframeHandoff,
  buildZoneSelect,
  concatValues,
  contextZoneWindow,
  CORE_OVERLAY_MAP,
  contextReductionExport,
  curveSelect,
  depthReframeHandoff,
  describeContextOutcome,
  fetchContextLayers,
  loadCurveNames,
  loadPlotProps,
  mergeDepthReframeHandoffs,
  nearestDepthIndex,
  pickRow,
  plotWriteAxis,
  plotWriteSelection,
  savePlotProps,
  trySelect,
  writePlotParameter,
  type PlotContent,
  type PlotWriteSource,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import { applyPlotRecordLimit, plotRecordLimit, reducePlotLabel } from "./plotLimits";
import { buildWellScope } from "./wellScope";
import { renderPlotToPaperSvg } from "./svgExport";
import { renderPlotToPaperPdf, type PlotPdf } from "./pdfExport";
import {
  axisRangeExportRecord,
  formatAxisRangeSummary,
  resolveBoundAxisRange,
  type AxisDisplayRange,
  type AxisRangeResolution,
  type PlotAxisRangeExport,
} from "./axisRange";
import { registerPlotInvalidationContract } from "./plotInvalidation";
import { beginPlotAsyncGeneration, isPlotAsyncGenerationCurrent } from "./plotAsync";
import {
  chartRecordForSurface,
  type ChartRenderRecord,
  type ChartRenderSurface,
} from "./chartProvenance";
import {
  applyPlotRangePolicy,
  formatPlotRangePolicySummary,
  type PlotRangePolicyReport,
} from "./plotRangePolicy";

export type RegModel = "linear" | "power" | "logx" | "exp";
export type RegMethod = "yx" | "xy" | "rma";
export type SizeMode = "fill" | "fixed";

interface ChartSourceProvenance {
  chartType: string;
  citation: string;
  publisher: string;
  revisionDate: string;
  digitizer?: string;
  approvedDerivationPath: "licensed_source" | "independently_digitized_public_primary_source";
  payloadChecksum: string;
}

export interface CrossplotOptions {
  pointSize: number;
  xLog: boolean;
  yLog: boolean;
  /** Regression line with equation + R² readout. */
  regression: boolean;
  /** Fit model: linear, power (log-log), logx (semilog X), exp (semilog Y). */
  regModel: RegModel;
  /** Fit method: ordinary Y-on-X, inverse X-on-Y, or reduced major axis. */
  regMethod: RegMethod;
  /** Manual axis ranges; null = header display, audited family display, then finite data. */
  xMin: number | null;
  xMax: number | null;
  yMin: number | null;
  yMax: number | null;
  /** Scientific validity limits are opt-in and never become display ranges. */
  validityFilter: boolean;
  xValidMin: number | null;
  xValidMax: number | null;
  yValidMin: number | null;
  yValidMax: number | null;
  /** Overlay core plug data as diamond markers when the X/Y axes are recognized
   *  log-curve counterparts of a core measurement (see CORE_OVERLAY_MAP). */
  showCore: boolean;
  /** Thomas-Stieber triangle overlay (meant for a VSH-PHIT plot): laminated and
   *  dispersed shale lines between draggable clean-sand / shale endpoints. */
  tsOverlay: boolean;
  /** T-S endpoints: clean-sand porosity (at VSH=0) and shale porosity (at VSH=1). */
  tsPhiSd: number;
  tsPhiSh: number;
  /** Qtz/Cal/Dol matrix reference points on an NPHI-RHOB plot — opt-in (universal:
   *  overlays only when requested). */
  matrixPoints: boolean;
  /** Rock-typing pore-throat iso-radius grid on a φ-k crossplot: "" (none), "winland"
   *  (Kolodzie 1980 R35) or "pittman_r25"/"pittman_r35"/"pittman_r50" (Pittman 1992).
   *  Drawn when one axis is a porosity curve and the other a permeability curve. */
  rockOverlay: string;
  /** Chartbook overlay by id from CHART_OVERLAYS ("" = none): digitized matrix
   *  curves / mineral regions / reference lines, drawn when the plot axes match
   *  the chart's axes (either orientation). */
  chartOverlay: string;
  /** Complete record for the chart payload actually authorized for this plot. A selected
   *  chart with absent source metadata remains null and is not rendered. */
  chartProvenance: ChartRenderRecord | null;
  /** Plot size: fill the panel (default) or a fixed pixel size (consistent exports). */
  sizeMode: SizeMode;
  plotW: number;
  plotH: number;
  /** Marginal histograms of X (top strip) and Y (right strip). */
  marginals: boolean;
  /** Bins for the marginal histograms. */
  bins: number;
  /** Point color when no Z curve is selected; empty = theme accent. */
  color: string;
  /** User percentiles: dashed X (vertical) and Y (horizontal) reference lines. */
  percentiles: number[];
  /** Z color scale: log10 spacing (permeability-style Z). */
  zLog: boolean;
  /** Continuous-Z colormap; viridis stays ordered/readable where rainbow fails. */
  colormap: ColormapName;
  /** Show the pick rows + draggable parameter handle. */
  showPicks: boolean;
  /** Cutoff-region overlay anchored at the parameter handle: which quadrant is "net"
   *  (X≥/≤ pick, Y≥/≤ pick), or "off" for just the point handle (default). */
  netSense: NetSense;
}

/** Which quadrant relative to the parameter handle counts as net reservoir. The user picks
 *  the sense explicitly (no cutoff direction is assumed from the axes). */
export type NetSense = "off" | "xge_yle" | "xle_yge" | "xge_yge" | "xle_yle";

export const DEFAULT_CROSSPLOT_OPTIONS: CrossplotOptions = {
  pointSize: 1.6,
  xLog: false,
  yLog: false,
  regression: false,
  regModel: "linear",
  regMethod: "yx",
  xMin: null,
  xMax: null,
  yMin: null,
  yMax: null,
  validityFilter: false,
  xValidMin: null,
  xValidMax: null,
  yValidMin: null,
  yValidMax: null,
  showCore: false,
  tsOverlay: false,
  tsPhiSd: 0.3,
  tsPhiSh: 0.15,
  matrixPoints: false,
  rockOverlay: "",
  chartOverlay: "",
  chartProvenance: null,
  sizeMode: "fill",
  plotW: 640,
  plotH: 480,
  marginals: false,
  bins: HISTOGRAM_BINS_DEFAULT,
  color: "",
  percentiles: [],
  zLog: false,
  colormap: "rainbow",
  showPicks: true,
  netSense: "off",
};

/** Fills defaults and sanitizes saved/template options. v1 props carried no regModel —
 *  the old fit ran in the axes' own lin/log space, so derive the equivalent model from
 *  the axis-log flags to keep saved por-perm regressions meaning the same thing. */
export function normalizeCrossplotOptions(raw: Partial<CrossplotOptions>): CrossplotOptions {
  const opts: CrossplotOptions = { ...DEFAULT_CROSSPLOT_OPTIONS, ...raw };
  // Preserve the record as portable evidence. It never authorizes itself: every live draw,
  // state write and export rebuilds and validates the record from the current chart definition.
  if (raw.regModel === undefined) {
    opts.regModel = opts.xLog && opts.yLog ? "power" : opts.xLog ? "logx" : opts.yLog ? "exp" : "linear";
  }
  if (!["linear", "power", "logx", "exp"].includes(opts.regModel)) opts.regModel = "linear";
  if (!["yx", "xy", "rma"].includes(opts.regMethod)) opts.regMethod = "yx";
  if (opts.sizeMode !== "fixed") opts.sizeMode = "fill";
  // migrate the short-lived dnOverlay option (P2-f+) to the chart registry
  const legacyDn = (raw as { dnOverlay?: string }).dnOverlay;
  if (raw.chartOverlay === undefined && legacyDn) {
    opts.chartOverlay = legacyDn === "fresh" ? "por11" : legacyDn === "salt" ? "por12" : "";
  }
  if (typeof opts.chartOverlay !== "string" || (opts.chartOverlay !== "" && !resolveChartOverlay(opts.chartOverlay))) {
    opts.chartOverlay = "";
  }
  if (!["", "winland", "pittman_r25", "pittman_r35", "pittman_r50"].includes(opts.rockOverlay)) {
    opts.rockOverlay = "";
  }
  opts.plotW = Math.max(200, Math.min(2000, Math.round(opts.plotW) || DEFAULT_CROSSPLOT_OPTIONS.plotW));
  opts.plotH = Math.max(200, Math.min(2000, Math.round(opts.plotH) || DEFAULT_CROSSPLOT_OPTIONS.plotH));
  opts.bins = normalizeHistogramBinCount(opts.bins);
  opts.color = typeof opts.color === "string" ? opts.color : "";
  opts.percentiles = Array.isArray(opts.percentiles) ? parsePercentiles(opts.percentiles.join(",")) : [];
  if (opts.colormap !== "viridis") opts.colormap = "rainbow";
  opts.pointSize = Math.max(0.5, Math.min(8, opts.pointSize || DEFAULT_CROSSPLOT_OPTIONS.pointSize));
  if (!["off", "xge_yle", "xle_yge", "xge_yge", "xle_yle"].includes(opts.netSense)) opts.netSense = "off";
  return opts;
}

/** Even-odd point-in-polygon in the axes' *drawing* plane (log10 on a log axis) — the frontend twin
 *  of `netflag.rs::point_in_polygon`, so the crossplot's live net-polygon count matches the curve the
 *  backend writes. The point and the polygon vertices are in DATA space; a value that can't be placed
 *  (NaN, or ≤ 0 on a log axis) is treated as outside. The ring is implicitly closed. */
export function netPolygonContains(
  dx: number,
  dy: number,
  poly: [number, number][],
  xLog: boolean,
  yLog: boolean,
): boolean {
  const tf = (v: number, log: boolean): number => (log ? (v > 0 ? Math.log10(v) : NaN) : v);
  const px = tf(dx, xLog);
  const py = tf(dy, yLog);
  if (Number.isNaN(px) || Number.isNaN(py) || poly.length < 3) return false;
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const xi = tf(poly[i][0], xLog);
    const yi = tf(poly[i][1], yLog);
    const xj = tf(poly[j][0], xLog);
    const yj = tf(poly[j][1], yLog);
    if (yi > py !== yj > py) {
      const xc = xi + ((py - yi) / (yj - yi)) * (xj - xi);
      if (px < xc) inside = !inside;
    }
  }
  return inside;
}

export interface RegFit {
  a: number;
  b: number;
  r2: number;
  n: number;
}

/** Regression over valid pairs in the MODEL's transformed space (log10 where the model
 *  says so — independent of how the axes are displayed). Method: "yx" ordinary least
 *  squares, "xy" inverse regression (fit x on y, invert the line), "rma" reduced major
 *  axis (symmetric — slope = sign(r)·σy/σx). R² is the squared correlation, identical
 *  for all three methods. Needs ≥3 points and non-degenerate variance. */
export function fitRegression(
  xs: ArrayLike<number>,
  ys: ArrayLike<number>,
  model: RegModel,
  method: RegMethod,
): RegFit | null {
  const xNeedsLog = model === "power" || model === "logx";
  const yNeedsLog = model === "power" || model === "exp";
  let n = 0;
  let sx = 0;
  let sy = 0;
  let sxx = 0;
  let syy = 0;
  let sxy = 0;
  const len = Math.min(xs.length, ys.length);
  for (let i = 0; i < len; i++) {
    let x = xs[i];
    let y = ys[i];
    if (Number.isNaN(x) || Number.isNaN(y)) continue;
    if (xNeedsLog) {
      if (x <= 0) continue;
      x = Math.log10(x);
    }
    if (yNeedsLog) {
      if (y <= 0) continue;
      y = Math.log10(y);
    }
    n++;
    sx += x;
    sy += y;
    sxx += x * x;
    syy += y * y;
    sxy += x * y;
  }
  if (n < 3) return null;
  const varX = sxx - (sx * sx) / n;
  const varY = syy - (sy * sy) / n;
  const cov = sxy - (sx * sy) / n;
  if (varX <= 1e-12 || varY <= 1e-12) return null;
  let b: number;
  if (method === "yx") b = cov / varX;
  else if (method === "xy") {
    if (Math.abs(cov) < 1e-12) return null;
    b = varY / cov;
  } else b = (cov < 0 ? -1 : 1) * Math.sqrt(varY / varX);
  const a = (sy - b * sx) / n;
  const r2 = (cov * cov) / (varX * varY);
  return { a, b, r2, n };
}

/** Human-readable fit equation in real units for the chosen model. */
export function fitEquation(fit: RegFit, xName: string, yName: string, model: RegModel): string {
  const num = (v: number) => v.toPrecision(4);
  switch (model) {
    case "power":
      return `${yName} = ${num(Math.pow(10, fit.a))} · ${xName}^${num(fit.b)}`;
    case "logx":
      return `${yName} = ${num(fit.a)} + ${num(fit.b)}·log10(${xName})`;
    case "exp":
      return `${yName} = 10^(${num(fit.a)} + ${num(fit.b)}·${xName})`;
    default:
      return `${yName} = ${num(fit.a)} + ${num(fit.b)}·${xName}`;
  }
}

const METHOD_LABEL: Record<RegMethod, string> = { yx: "", xy: " · X on Y", rma: " · RMA" };

/** Draws the Thomas-Stieber construction on an (assumed) VSH-PHIT plot: the laminated
 *  line sand→shale, the dispersed-shale line descending to its porosity minimum at
 *  VSH = PHI_SD (pores full of shale), and circle handles on the two endpoints. */
export function drawTsOverlay(plot: PlotCanvas, phiSd: number, phiSh: number): void {
  const lamColor = plot.theme.accent2;
  plot.drawLine(
    [
      [0, phiSd],
      [1, phiSh],
    ],
    lamColor,
    1.6,
  );
  // Dispersed trend: PHIT = PHI_SD − VSH·(1−PHI_SH) down to the minimum, then back up
  // to the shale point (shale beyond the pore-filling limit displaces matrix).
  const vMin = Math.min(1, phiSd); // VSH where dispersed shale exactly fills the pores
  plot.drawLine(
    [
      [0, phiSd],
      [vMin, phiSd * phiSh],
    ],
    lamColor,
    1.6,
    [5, 4],
  );
  if (vMin < 1) {
    plot.drawLine(
      [
        [vMin, phiSd * phiSh],
        [1, phiSh],
      ],
      lamColor,
      1.0,
      [2, 4],
    );
  }
  const { ctx } = plot;
  ctx.save();
  for (const [vx, vy, label] of [
    [0, phiSd, `PHI_SD_MAX ${phiSd.toFixed(3)}`] as const,
    [1, phiSh, `PHI_SH ${phiSh.toFixed(3)}`] as const,
  ]) {
    const [px, py] = plot.toPx(vx, vy);
    ctx.fillStyle = lamColor;
    ctx.strokeStyle = plot.theme.text;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.font = canvasFont(plot.theme, 10);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = vx === 0 ? "left" : "right";
    ctx.fillText(label, px + (vx === 0 ? 8 : -8), py - 8);
  }
  ctx.restore();
}

/** Curve mnemonics recognized as porosity / permeability axes for the rock-typing grid. */
const PORO_AXIS = ["PHIE", "PHIT", "PHI", "POR", "CPOR", "MM_PHIE", "MM_PHIT", "PHIX", "PHIE_ND", "PHIT_ND"];
const PERM_AXIS = ["PERM", "KLOGH", "CPERM", "KH", "K", "PERM_RT", "KINT", "KAIR", "KTIM", "PERM_COATES"];

/** Pittman (1992) log10 rX = C0 + C1·log10 k + C2·log10 φ% coefficient rows offered as grids
 *  (mirrors PITTMAN_RX in rocktyping.rs; r35 anchors the transcription). */
const ROCK_OVERLAY_COEF: Record<string, { label: string; c0: number; c1: number; c2: number }> = {
  // Winland written in the same form: log10 R35 = 0.732 + 0.588·log10 k − 0.864·log10 φ%.
  winland: { label: "Winland R35", c0: 0.732, c1: 0.588, c2: -0.864 },
  pittman_r25: { label: "Pittman r25", c0: 0.204, c1: 0.531, c2: -0.35 },
  pittman_r35: { label: "Pittman r35", c0: 0.255, c1: 0.565, c2: -0.523 },
  pittman_r50: { label: "Pittman r50", c0: 0.609, c1: 0.608, c2: -0.974 },
};

/** Hartmann-Beaumont port-class boundaries (µm) drawn as iso-radius lines, port names between. */
const PORT_ISO_R = [0.1, 0.5, 2.5, 10];

/** Which orientation (if any) makes (xName, yName) a φ-k crossplot for the rock-typing grid. */
export function matchRockOverlayAxes(xName: string, yName: string): "normal" | "flipped" | null {
  const X = xName.toUpperCase();
  const Y = yName.toUpperCase();
  if (PORO_AXIS.includes(X) && PERM_AXIS.includes(Y)) return "normal";
  if (PORO_AXIS.includes(Y) && PERM_AXIS.includes(X)) return "flipped";
  return null;
}

/** Draws pore-throat iso-radius lines (Winland R35 or a Pittman rX) on a φ-k crossplot: for each
 *  port-class boundary radius, k(φ) from log10 k = (log10 r − C0 − C2·log10 φ%)/C1 (φ v/v on the
 *  porosity axis, k mD on the permeability axis; "flipped" = φ on Y). Drawn in data space so the
 *  grid stays registered under zoom/pan and honors log axes. */
export function drawRockOverlay(plot: PlotCanvas, kind: string, flipped: boolean): void {
  const coef = ROCK_OVERLAY_COEF[kind];
  if (!coef) return;
  const color = plot.theme.accent2;
  const { ctx } = plot;
  // Sample φ across the practical band, log-spaced so log-φ axes stay smooth.
  const PHI_LO = 0.01;
  const PHI_HI = 0.47;
  const N = 40;
  for (const r of PORT_ISO_R) {
    const pts: [number, number][] = [];
    for (let i = 0; i <= N; i++) {
      const phi = PHI_LO * Math.pow(PHI_HI / PHI_LO, i / N);
      const logk = (Math.log10(r) - coef.c0 - coef.c2 * Math.log10(phi * 100)) / coef.c1;
      const k = Math.pow(10, logk);
      pts.push(flipped ? [k, phi] : [phi, k]);
    }
    plot.drawLine(pts, color, 1.2, [6, 4]);
    // Label at the line's high-φ end.
    const [lx, ly] = pts[pts.length - 1];
    const [px, py] = plot.toPx(lx, ly);
    ctx.save();
    ctx.font = canvasFont(plot.theme, 10);
    ctx.fillStyle = color;
    ctx.textAlign = "left";
    ctx.fillText(`${r} µm`, px + 4, py - 3);
    ctx.restore();
  }
  // Caption with the grid's identity, top-left of the data region.
  ctx.save();
  ctx.font = canvasFont(plot.theme, 10, 600);
  ctx.fillStyle = color;
  ctx.textAlign = "left";
  ctx.fillText(`${coef.label} port classes (nano < 0.1 < micro < 0.5 < meso < 2.5 < macro < 10 < mega µm)`, plot.margin.left + 6, plot.margin.top + 12);
  ctx.restore();
}

/** Quartz / calcite / dolomite matrix reference points in (NPHI, RHOB) space — drawn
 *  only when requested AND the plot actually is an NPHI-RHOB crossplot. */
const MATRIX_POINTS: { nphi: number; rhob: number; label: string }[] = [
  { nphi: -0.02, rhob: 2.65, label: "Qtz" },
  { nphi: 0.0, rhob: 2.71, label: "Cal" },
  { nphi: 0.02, rhob: 2.87, label: "Dol" },
];

/** Which orientation (if any) lets this chart overlay draw on the given axes. */
export function matchOverlayAxes(def: ChartOverlayDef, xName: string, yName: string): "normal" | "flipped" | null {
  const xs = AXIS_ALIASES[def.xAxis];
  const ys = AXIS_ALIASES[def.yAxis];
  const X = xName.toUpperCase();
  const Y = yName.toUpperCase();
  if (xs.includes(X) && ys.includes(Y)) return "normal";
  if (xs.includes(Y) && ys.includes(X)) return "flipped";
  return null;
}

interface OverlayAxisDeclaration {
  quantity: string;
  canonicalUnit: string;
  admissibleTransform: "identity" | "affine";
}

const OVERLAY_AXIS_DECLARATIONS: Record<AxisKind, OverlayAxisDeclaration> = {
  neutron: { quantity: "fraction", canonicalUnit: "v/v", admissibleTransform: "affine" },
  density: { quantity: "bulk_density", canonicalUnit: "g/cc", admissibleTransform: "affine" },
  dt: { quantity: "slowness", canonicalUnit: "us/ft", admissibleTransform: "affine" },
  pef: { quantity: "photoelectric_factor", canonicalUnit: "b/e", admissibleTransform: "identity" },
  k: { quantity: "fraction", canonicalUnit: "%", admissibleTransform: "identity" },
  th: { quantity: "thorium_concentration", canonicalUnit: "ppm", admissibleTransform: "identity" },
  thk: { quantity: "thorium_potassium_ratio", canonicalUnit: "ratio", admissibleTransform: "identity" },
  umaa: { quantity: "umaa", canonicalUnit: "chart_unit", admissibleTransform: "identity" },
  rhomaa: { quantity: "bulk_density", canonicalUnit: "g/cc", admissibleTransform: "affine" },
};

interface OverlayUnitTransform {
  sourceUnit: string;
  displayUnit: string;
  factor: number;
  offset: number;
  toSource: (displayValue: number) => number;
}

const normalizedUnit = (unit: string): string => unit.trim().toLowerCase().replace(/[._\s]/g, "");

function overlayUnitTransform(sourceUnit: string, targetUnit: string): OverlayUnitTransform | null {
  if (normalizedUnit(sourceUnit) === normalizedUnit(targetUnit)) {
    return { sourceUnit, displayUnit: targetUnit, factor: 1, offset: 0, toSource: (value) => value };
  }
  // Exact reviewed transforms from curves.rs. The renderer uses their inverse to place a
  // canonical chart coordinate on an axis that still displays the source unit.
  const rules: Array<[string, string, number, number]> = [
    ["us/m", "us/ft", 0.3048, 0],
    ["kg/m3", "g/cc", 0.001, 0],
    ["pu", "v/v", 0.01, 0],
    ["%", "v/v", 0.01, 0],
  ];
  const rule = rules.find(([from, to]) => normalizedUnit(from) === normalizedUnit(sourceUnit)
    && normalizedUnit(to) === normalizedUnit(targetUnit));
  if (!rule) return null;
  const [, , factor, offset] = rule;
  return {
    sourceUnit,
    displayUnit: targetUnit,
    factor,
    offset,
    toSource: (displayValue) => displayValue / factor - offset,
  };
}

interface TypedOverlayAuthorization {
  orientation: "normal" | "flipped";
  x: OverlayUnitTransform;
  y: OverlayUnitTransform;
}

interface ChartRenderDecision {
  authorization: TypedOverlayAuthorization | null;
  record: ChartRenderRecord | null;
  refusal: string | null;
}

function authorizeTypedOverlay(
  def: ChartOverlayDef,
  xName: string,
  yName: string,
  xSource: ResolvedPlotCurve | null,
  ySource: ResolvedPlotCurve | null,
): TypedOverlayAuthorization | null {
  const orientation = matchOverlayAxes(def, xName, yName);
  if (!orientation || !xSource || !ySource) return null;
  const xKind = orientation === "normal" ? def.xAxis : def.yAxis;
  const yKind = orientation === "normal" ? def.yAxis : def.xAxis;
  const xDecl = OVERLAY_AXIS_DECLARATIONS[xKind];
  const yDecl = OVERLAY_AXIS_DECLARATIONS[yKind];
  if (xSource.quantity !== xDecl.quantity || ySource.quantity !== yDecl.quantity) return null;
  const x = overlayUnitTransform(xSource.source_unit, xDecl.canonicalUnit);
  const y = overlayUnitTransform(ySource.source_unit, yDecl.canonicalUnit);
  if (!x || !y) return null;
  if ((x.factor !== 1 && xDecl.admissibleTransform !== "affine")
    || (y.factor !== 1 && yDecl.admissibleTransform !== "affine")) return null;
  return { orientation, x, y };
}

function authorizeProvenancedChart(
  def: ChartOverlayDef,
  xName: string,
  yName: string,
  xSource: ResolvedPlotCurve | null,
  ySource: ResolvedPlotCurve | null,
  surface: ChartRenderSurface = "screen",
): ChartRenderDecision {
  const authorization = authorizeTypedOverlay(def, xName, yName, xSource, ySource);
  if (!authorization) return { authorization: null, record: null, refusal: null };
  const source = (def as ChartOverlayDef & { provenance?: ChartSourceProvenance }).provenance;
  if (!source) {
    return {
      authorization,
      record: null,
      refusal: `chart ${def.id} is blocked: source provenance is absent`,
    };
  }
  const required: Array<[string, string]> = [
    [def.id, "chart id"],
    [def.label, "chart title"],
    [source.chartType, "chart type"],
    [source.citation, "citation"],
    [source.publisher, "publisher"],
    [source.revisionDate, "source revision/date"],
    [source.approvedDerivationPath, "approved derivation path"],
  ];
  const missing = required.find(([value]) => !value?.trim());
  if (missing) {
    return { authorization, record: null, refusal: `chart ${def.id} is blocked: missing ${missing[1]}` };
  }
  if (!["licensed_source", "independently_digitized_public_primary_source"].includes(source.approvedDerivationPath)) {
    return { authorization, record: null, refusal: `chart ${def.id} is blocked: derivation path is not approved` };
  }
  if (source.approvedDerivationPath === "independently_digitized_public_primary_source" && !source.digitizer?.trim()) {
    return { authorization, record: null, refusal: `chart ${def.id} is blocked: digitizer is absent` };
  }
  if (!/^[0-9a-f]{64}$/i.test(source.payloadChecksum)) {
    return { authorization, record: null, refusal: `chart ${def.id} is blocked: payload checksum is absent or invalid` };
  }
  const xKind = authorization.orientation === "normal" ? def.xAxis : def.yAxis;
  const yKind = authorization.orientation === "normal" ? def.yAxis : def.xAxis;
  const transform = JSON.stringify({
    orientation: authorization.orientation,
    x: {
      source_unit: authorization.x.sourceUnit,
      display_unit: authorization.x.displayUnit,
      factor: authorization.x.factor,
      offset: authorization.x.offset,
    },
    y: {
      source_unit: authorization.y.sourceUnit,
      display_unit: authorization.y.displayUnit,
      factor: authorization.y.factor,
      offset: authorization.y.offset,
    },
  });
  const record: ChartRenderRecord = {
      chart_id: def.id,
      title: def.label,
      chart_type: source.chartType,
      x_quantity: OVERLAY_AXIS_DECLARATIONS[xKind].quantity,
      x_unit: OVERLAY_AXIS_DECLARATIONS[xKind].canonicalUnit,
      y_quantity: OVERLAY_AXIS_DECLARATIONS[yKind].quantity,
      y_unit: OVERLAY_AXIS_DECLARATIONS[yKind].canonicalUnit,
      citation: source.citation,
      publisher: source.publisher,
      revision_date: source.revisionDate,
      digitizer: source.digitizer?.trim() || null,
      approved_derivation_path: source.approvedDerivationPath,
      payload_checksum: source.payloadChecksum.toLowerCase(),
      transform_applied: transform,
  };
  try {
    return { authorization, refusal: null, record: chartRecordForSurface(def.id, record, surface) };
  } catch (error) {
    return { authorization, record: null, refusal: String(error) };
  }
}

function drawChartProvenanceRefusal(plot: PlotCanvas, refusal: string): void {
  const { ctx } = plot;
  const r = plot.plotRect;
  ctx.save();
  ctx.fillStyle = plot.theme.warn;
  ctx.font = canvasFont(plot.theme, 10, 600);
  ctx.textAlign = "left";
  ctx.fillText(refusal, r.x0 + 8, r.y0 + r.h - 9, Math.max(40, r.w - 16));
  ctx.restore();
}

/** Generic chartbook overlay renderer: matrix curves with graduation dots (every
 *  5) + numeric labels (every labelEvery) + along-slope names, dashed
 *  iso-graduation connectors, reference lines, mineral-region polygons, and
 *  labeled reference points. Everything is drawn in data space, so it stays
 *  registered under zoom/pan and on either axis orientation. */
function drawChartOverlay(
  plot: PlotCanvas,
  def: ChartOverlayDef,
  authorization: TypedOverlayAuthorization,
): void {
  const { ctx } = plot;
  const r = plot.plotRect;
  const XY = (x: number, y: number): [number, number] => {
    const oriented: [number, number] = authorization.orientation === "flipped" ? [y, x] : [x, y];
    return [authorization.x.toSource(oriented[0]), authorization.y.toSource(oriented[1])];
  };

  if (def.isoConnect && def.curves) {
    const maxT = Math.max(...def.curves.flatMap((c) => c.grads.map((g) => g[0])));
    for (let t = 0; t <= maxT; t += 5) {
      const line: [number, number][] = [];
      for (const c of def.curves) {
        const g = c.grads.find((q) => q[0] === t);
        if (g) line.push(XY(g[1], g[2]));
      }
      if (line.length >= 2) plot.drawLine(line, plot.theme.axis, 0.7, [2, 3]);
    }
  }
  for (const c of def.curves ?? []) {
    plot.drawLine(c.grads.map((g) => XY(g[1], g[2])), plot.theme.axis, 1.3);
  }
  for (const l of def.lines ?? []) {
    plot.drawLine(l.pts.map((p) => XY(p[0], p[1])), plot.theme.axis, 1.1, l.dash ? [5, 4] : []);
  }
  for (const g of def.regions ?? []) {
    plot.drawLine([...g.poly, g.poly[0]].map((p) => XY(p[0], p[1])), plot.theme.axis, 1, [3, 3]);
  }

  ctx.save();
  ctx.beginPath();
  ctx.rect(r.x0, r.y0, r.w, r.h);
  ctx.clip();
  for (const c of def.curves ?? []) {
    ctx.font = canvasFont(plot.theme, 8);
    for (const [t, gx, gy] of c.grads) {
      if (t % 5 !== 0) continue;
      const [px, py] = plot.toPx(...XY(gx, gy));
      ctx.fillStyle = plot.theme.axis;
      ctx.beginPath();
      ctx.arc(px, py, 2, 0, Math.PI * 2);
      ctx.fill();
      if (t % c.labelEvery === 0) {
        ctx.fillStyle = plot.theme.text;
        ctx.textAlign = "right";
        ctx.fillText(String(t), px - 3, py - 3);
      }
    }
    // Curve name written along the local slope, ~60% of the way along.
    const i = Math.max(1, Math.floor(c.grads.length * 0.6));
    const [ax, ay] = plot.toPx(...XY(c.grads[i - 1][1], c.grads[i - 1][2]));
    const [bx, by] = plot.toPx(...XY(c.grads[i][1], c.grads[i][2]));
    let angle = Math.atan2(by - ay, bx - ax);
    if (angle > Math.PI / 2 || angle < -Math.PI / 2) angle += Math.PI;
    ctx.save();
    ctx.translate((ax + bx) / 2, (ay + by) / 2);
    ctx.rotate(angle);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "center";
    ctx.font = canvasFont(plot.theme, 9, 600);
    ctx.fillText(c.name, 0, -7);
    ctx.restore();
  }
  ctx.font = canvasFont(plot.theme, 8);
  for (const l of def.lines ?? []) {
    if (!l.label) continue;
    const end = l.pts[l.pts.length - 1];
    const [px, py] = plot.toPx(...XY(end[0], end[1]));
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "right";
    ctx.fillText(l.label, px - 3, py - 4);
  }
  for (const g of def.regions ?? []) {
    const cx = g.poly.reduce((a, p) => a + p[0] / g.poly.length, 0);
    const cy = g.poly.reduce((a, p) => a + p[1] / g.poly.length, 0);
    const [px, py] = plot.toPx(...XY(cx, cy));
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "center";
    ctx.font = canvasFont(plot.theme, 9);
    ctx.fillText(g.label, px, py - 8);
  }
  ctx.restore();

  for (const p of def.points ?? []) {
    const [vx, vy] = XY(p.x, p.y);
    plot.drawRefPoint(vx, vy, p.label);
  }
}

/** Pairs two independently-filtered core series (each keeps only its own non-NaN
 *  samples, so their indices don't line up) by matching on exact depth — only plugs
 *  with both measurements present are plotted. */
function alignCoreSeriesByDepth(a: TrackCurveSeries, b: TrackCurveSeries): { xs: Float32Array; ys: Float32Array } {
  const bByDepth = new Map<number, number>();
  for (let i = 0; i < b.depth.length; i++) bByDepth.set(b.depth[i], b.value[i]);
  const xs: number[] = [];
  const ys: number[] = [];
  for (let i = 0; i < a.depth.length; i++) {
    const bv = bByDepth.get(a.depth[i]);
    if (bv !== undefined) {
      xs.push(a.value[i]);
      ys.push(bv);
    }
  }
  return { xs: Float32Array.from(xs), ys: Float32Array.from(ys) };
}

export interface MarginalHistogramContract extends HistogramContract {
  /** Finite non-positive values excluded before a logarithmic transform. */
  logDomainExcluded: number;
}

/** Canonical bins over the axis's own space. Log axes transform only eligible finite values,
 *  preserving source non-finite and log-domain exclusion counts as different facts. */
export function computeMarginalHistogram(
  values: ArrayLike<number>,
  lo: number,
  hi: number,
  bins: number,
  log: boolean,
): MarginalHistogramContract | null {
  const t = (v: number) => (log ? Math.log10(v) : v);
  const tLo = t(lo);
  const tHi = t(hi);
  if (!Number.isFinite(tLo) || !Number.isFinite(tHi) || tLo === tHi) return null;
  const transformed: number[] = [];
  let logDomainExcluded = 0;
  for (let index = 0; index < values.length; index++) {
    const value = values[index];
    if (!Number.isFinite(value)) {
      transformed.push(value);
    } else if (log && value <= 0) {
      logDomainExcluded++;
    } else {
      transformed.push(t(value));
    }
  }
  return { ...canonicalHistogram(transformed, tLo, tHi, bins), logDomainExcluded };
}

/** Marginal histograms in the widened top (X) and right (Y) margins, aligned with the
 *  plot's axes (fraction-based so log scales and inverted axes stay in register). */
function drawMarginals(
  plot: PlotCanvas,
  xs: Float32Array,
  ys: Float32Array,
  bins: number,
  color: string,
): { x: MarginalHistogramContract | null; y: MarginalHistogramContract | null } {
  const r = plot.plotRect;
  const { ctx } = plot;
  const stripH = plot.margin.top - 12;
  const stripW = plot.margin.right - 16;
  ctx.save();
  ctx.fillStyle = color;
  ctx.globalAlpha = 0.55;
  const xHistogram = computeMarginalHistogram(xs, Math.min(plot.x.min, plot.x.max), Math.max(plot.x.min, plot.x.max), bins, plot.x.log);
  if (xHistogram && xHistogram.displayedTotal > 0 && stripH > 6) {
    const peak = Math.max(...xHistogram.counts);
    const bw = r.w / xHistogram.counts.length;
    for (let i = 0; i < xHistogram.counts.length; i++) {
      if (xHistogram.counts[i] === 0) continue;
      const f = (i + 0.5) / xHistogram.counts.length;
      const px = r.x0 + (plot.x.invert ? 1 - f : f) * r.w;
      const h = (xHistogram.counts[i] / peak) * stripH;
      ctx.fillRect(px - bw / 2 + 0.5, r.y0 - 4 - h, Math.max(1, bw - 1), h);
    }
  }
  const yHistogram = computeMarginalHistogram(ys, Math.min(plot.y.min, plot.y.max), Math.max(plot.y.min, plot.y.max), bins, plot.y.log);
  if (yHistogram && yHistogram.displayedTotal > 0 && stripW > 6) {
    const peak = Math.max(...yHistogram.counts);
    const bh = r.h / yHistogram.counts.length;
    for (let i = 0; i < yHistogram.counts.length; i++) {
      if (yHistogram.counts[i] === 0) continue;
      const f = (i + 0.5) / yHistogram.counts.length;
      const py = r.y0 + (plot.y.invert ? f : 1 - f) * r.h;
      const w = (yHistogram.counts[i] / peak) * stripW;
      ctx.fillRect(r.x0 + r.w + 4, py - bh / 2 + 0.5, w, Math.max(1, bh - 1));
    }
  }
  ctx.restore();
  return { x: xHistogram, y: yHistogram };
}

/** Precomputed per-point Z coloring for the scatter — the redraw's heaviest step (two
 *  percentile sorts + an N-length color array, or the categorical palette map). It depends
 *  only on the Z data + colormap, never the viewport, so the panel caches it and reuses it
 *  across pan/zoom/hover frames (see the `zColors` memo in buildCrossplotContent). */
export interface CrossplotColors {
  /** Per-point colors, or undefined to fall back to the theme accent in drawScatter. */
  colors: string[] | undefined;
  /** True when Z is a discrete class curve (facies/cluster) → categorical legend. */
  categorical: boolean;
  /** Continuous color-bar range; NaN when categorical, no Z, or degenerate. */
  zLo: number;
  zHi: number;
  /** Per-point endpoint marker for a continuous Z value clamped to its display range. */
  edgeMarks: PlotRangeEdge[];
  zClamped: number;
  zExcluded: number;
  zIncluded: Uint8Array;
}

/** Builds the Z coloring + legend range. `fillColor` is the solid point color used when
 *  there's no Z but an explicit color is set (opts.color); pass "" for the theme-accent
 *  default. Pure — same inputs give the same output, which is what lets the panel memoize
 *  it (the two percentile sorts + N-length allocations here dominated the pan/zoom redraw). */
export function computeCrossplotColors(
  zName: string,
  zs: Float32Array,
  pointCount: number,
  colormap: ColormapName,
  zLog: boolean,
  fillColor: string,
): CrossplotColors {
  const hasZ = zName !== "" && zs.length > 0;
  const categorical = hasZ && (/FACIES|CLUSTER|LITHO|CLASS/i.test(zName) || looksDiscrete(zs));
  let zLo = NaN;
  let zHi = NaN;
  let colors: string[] | undefined;
  let edgeMarks: PlotRangeEdge[] = new Array(pointCount).fill("none");
  let zClamped = 0;
  let zExcluded = 0;
  let zIncluded = new Uint8Array(pointCount);
  if (categorical) {
    colors = categoricalColors(zs);
    for (let index = 0; index < zs.length; index++) {
      if (Number.isFinite(zs[index])) zIncluded[index] = 1;
      else {
        colors[index] = "rgba(0,0,0,0)";
        zExcluded++;
      }
    }
  } else if (hasZ) {
    // Log Z: percentile over the positive values only, else ≤0 junk wrecks the range.
    const zForRange = zLog ? Float32Array.from([...zs].filter((v) => !Number.isNaN(v) && v > 0)) : zs;
    zLo = percentile(zForRange, 5);
    zHi = percentile(zForRange, 95);
    if (!Number.isNaN(zLo) && zLo !== zHi) {
      const policy = applyPlotChannelPolicy(zs, "colour", { min: zLo, max: zHi }, zLog);
      colors = colorRampEx(policy.values, zLo, zHi, colormap, zLog);
      for (let index = 0; index < colors.length; index++) {
        if (policy.included[index] === 0) colors[index] = "rgba(0,0,0,0)";
      }
      edgeMarks = policy.edgeMarks;
      zClamped = policy.clamped;
      zExcluded = policy.nonFiniteExcluded + policy.logDomainExcluded;
      zIncluded = policy.included;
    } else {
      for (let index = 0; index < zs.length; index++) {
        if (Number.isFinite(zs[index]) && (!zLog || zs[index] > 0)) zIncluded[index] = 1;
        else zExcluded++;
      }
    }
  }
  if (!colors && fillColor) colors = new Array(pointCount).fill(fillColor);
  return { colors, categorical, zLo, zHi, edgeMarks, zClamped, zExcluded, zIncluded };
}

/** One extra well drawn behind the active well's cloud — display-only: no brushing,
 *  picks, tooltips or regression ever read these points. Arrays are pre-decimated to
 *  the panel's point budget before they get here. */
export interface ContextWellLayer {
  name: string;
  xs: Float32Array;
  ys: Float32Array;
  color: string;
}

export interface CrossplotContext {
  /** The active well's name, for the legend's first row. */
  activeName: string;
  layers: ContextWellLayer[];
}

export interface PairValidityReport {
  indices: number[];
  nonFiniteExcluded: number;
  logDomainExcluded: number;
  validityExcluded: number;
  statisticsCount: number;
}

export function screenPlotPairs(
  xs: Float32Array,
  ys: Float32Array,
  enabled: boolean,
  xValidity: AxisDisplayRange | null,
  yValidity: AxisDisplayRange | null,
  xLog = false,
  yLog = false,
): PairValidityReport {
  const report = screenCrossplotPopulation(
    xs,
    ys,
    enabled,
    xValidity,
    yValidity,
    null,
    null,
    xLog,
    yLog,
  );
  return {
    indices: report.indices,
    nonFiniteExcluded: report.nonFiniteExcluded,
    logDomainExcluded: report.logDomainExcluded,
    validityExcluded: report.validityExcluded,
    statisticsCount: report.analysisCount,
  };
}

export function screenCrossplotPopulation(
  xs: Float32Array,
  ys: Float32Array,
  enabled: boolean,
  xValidity: AxisDisplayRange | null,
  yValidity: AxisDisplayRange | null,
  xDisplay: AxisDisplayRange | null,
  yDisplay: AxisDisplayRange | null,
  xLog = false,
  yLog = false,
): PlotRangePolicyReport {
  return applyPlotRangePolicy([
    { values: xs, display: xDisplay, validity: xValidity, log: xLog },
    { values: ys, display: yDisplay, validity: yValidity, log: yLog },
  ], enabled);
}

/** Live Crossplot adapter: both channel summaries share one reconciled finite-pair population. */
export function buildCrossplotStatisticsRecords(
  xs: Float32Array,
  ys: Float32Array,
  opts: CrossplotOptions,
  xName: string,
  yName: string,
  wellId: string,
  intervalLow: number | null,
  intervalHigh: number | null,
  xDisplay: AxisDisplayRange | null,
  yDisplay: AxisDisplayRange | null,
  selectionLabel = "all eligible",
): PlotStatisticsRecord[] {
  const xValidity = opts.xValidMin !== null && opts.xValidMax !== null
    ? { min: opts.xValidMin, max: opts.xValidMax }
    : null;
  const yValidity = opts.yValidMin !== null && opts.yValidMax !== null
    ? { min: opts.yValidMin, max: opts.yValidMax }
    : null;
  const policy = screenCrossplotPopulation(
    xs,
    ys,
    opts.validityFilter,
    xValidity,
    yValidity,
    xDisplay,
    yDisplay,
    opts.xLog,
    opts.yLog,
  );
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
      Float32Array.from(policy.indices.map((index) => xs[index])),
      { ...context, binding_channel: "x", channel: `x:${xName}` },
    ),
    buildPlotStatisticsRecord(
      Float32Array.from(policy.indices.map((index) => ys[index])),
      { ...context, binding_channel: "y", channel: `y:${yName}` },
    ),
  ];
}

export function drawCrossplot(
  canvas: HTMLCanvasElement,
  xName: string,
  yName: string,
  zName: string,
  xs: Float32Array,
  ys: Float32Array,
  zs: Float32Array,
  opts: CrossplotOptions = DEFAULT_CROSSPLOT_OPTIONS,
  hoverIdx = -1,
  view: Viewport | null = null,
  precolors: CrossplotColors | null = null,
  context: CrossplotContext | null = null,
  typedAxes: { x: ResolvedPlotCurve | null; y: ResolvedPlotCurve | null } | null = null,
  axisBindings: { x: PlotChannelBinding | null; y: PlotChannelBinding | null } | null = null,
  onAxisRanges?: (ranges: PlotAxisRangeExport[]) => void,
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const xValidity = opts.xValidMin !== null && opts.xValidMax !== null
    ? { min: opts.xValidMin, max: opts.xValidMax }
    : null;
  const yValidity = opts.yValidMin !== null && opts.yValidMax !== null
    ? { min: opts.yValidMin, max: opts.yValidMax }
    : null;
  const validity = screenPlotPairs(xs, ys, opts.validityFilter, xValidity, yValidity, opts.xLog, opts.yLog);
  const plotXs = Float32Array.from(validity.indices.map((index) => xs[index]));
  const plotYs = Float32Array.from(validity.indices.map((index) => ys[index]));
  const plotZs = zs.length > 0 ? Float32Array.from(validity.indices.map((index) => zs[index])) : zs;
  const auto = (values: Float32Array, log: boolean): { min: number; max: number } | null => {
    const eligible = log
      ? Float32Array.from(values.filter((value) => Number.isFinite(value) && value > 0))
      : values;
    const lo = percentile(eligible, 2);
    const hi = percentile(eligible, 98);
    if (Number.isNaN(lo) || Number.isNaN(hi)) return null;
    const pad = (hi - lo) * 0.08 || Math.max(Math.abs(lo) * 0.01, 1e-6);
    const min = log ? Math.max(lo - pad, lo * 0.8) : lo - pad;
    return { min, max: hi + pad };
  };
  /** User > header display > audited family display > finite-data range. */
  const resolve = (
    binding: PlotChannelBinding | null,
    values: Float32Array,
    log: boolean,
    manMin: number | null,
    manMax: number | null,
    viewport: AxisDisplayRange | null,
    validityRange: AxisDisplayRange | null,
  ): { min: number; max: number; invert: boolean; resolution: AxisRangeResolution } | null => {
    const base = resolveBoundAxisRange({
      binding,
      user: manMin !== null && manMax !== null ? { min: manMin, max: manMax } : null,
      finiteData: auto(values, log),
      validity: validityRange,
      log,
    });
    if (!base) return null;
    const invert = base.min > base.max;
    const resolution = viewport
      ? {
          min: invert ? viewport.max : viewport.min,
          max: invert ? viewport.min : viewport.max,
          tier: "user" as const,
        }
      : base;
    return {
      min: Math.min(resolution.min, resolution.max),
      max: Math.max(resolution.min, resolution.max),
      invert,
      resolution,
    };
  };

  // With context wells the auto range (and the log-axis positive floor) must cover the
  // whole field's spread, not just the active well — otherwise the overlay draws clipped.
  const hasCtx = !!context && context.layers.length > 0;
  const rangeXs = hasCtx ? concatValues(plotXs, context!.layers.map((l) => l.xs)) : plotXs;
  const rangeYs = hasCtx ? concatValues(plotYs, context!.layers.map((l) => l.ys)) : plotYs;
  const xr = resolve(
    axisBindings?.x ?? null,
    rangeXs,
    opts.xLog,
    opts.xMin,
    opts.xMax,
    view ? { min: view.xMin, max: view.xMax } : null,
    xValidity,
  );
  const yr = resolve(
    axisBindings?.y ?? null,
    rangeYs,
    opts.yLog,
    opts.yMin,
    opts.yMax,
    view ? { min: view.yMin, max: view.yMax } : null,
    yValidity,
  );
  if (!xr || !yr) return null;
  const resolvedRanges = [
    axisRangeExportRecord("x", xr.resolution),
    axisRangeExportRecord("y", yr.resolution),
  ];
  onAxisRanges?.(resolvedRanges);

  const plot = new PlotCanvas(
    canvas,
    { label: xName, min: xr.min, max: xr.max, log: opts.xLog, invert: xr.invert },
    { label: yName, min: yr.min, max: yr.max, log: opts.yLog, invert: yr.invert },
    opts.marginals ? { top: 56, right: 64 } : undefined,
  );
  plot.drawFrame();
  plot.ctx.save();
  plot.ctx.font = canvasFont(plot.theme, 9);
  plot.ctx.fillStyle = plot.theme.axis;
  plot.ctx.textAlign = "left";
  plot.ctx.fillText(formatAxisRangeSummary(resolvedRanges), plot.plotRect.x0 + 4, plot.margin.top - 7);
  plot.ctx.restore();
  const pointColor = opts.color || plot.theme.accent;

  // Z coloring only when a Z curve is selected. Discrete class curves (electrofacies,
  // clusters) get categorical coloring + a swatch legend; continuous curves get the chosen
  // colormap (optionally log10-scaled) + a color bar. This is the redraw's heaviest step and
  // is viewport-independent, so the panel memoizes it and passes it in; we only compute it
  // here when a caller (or a test) doesn't supply one.
  const hasZ = zName !== "" && plotZs.length > 0;
  const indicesAreIdentity = validity.indices.length === xs.length
    && validity.indices.every((sourceIndex, displayIndex) => sourceIndex === displayIndex);
  const { colors, categorical, zLo, zHi, edgeMarks, zClamped, zExcluded, zIncluded } =
    (!opts.validityFilter && indicesAreIdentity ? precolors : null)
      ?? computeCrossplotColors(zName, plotZs, plotXs.length, opts.colormap, opts.zLog, opts.color);
  const displayColors = hasZ && !colors
    ? Array.from(zIncluded, (included) => included ? pointColor : "rgba(0,0,0,0)")
    : colors;

  // Context wells first, faded, so the active well's cloud always reads on top of them.
  if (hasCtx) {
    const { ctx } = plot;
    ctx.save();
    ctx.globalAlpha = 0.4;
    for (const layer of context!.layers) {
      plot.drawScatter(layer.xs, layer.ys, layer.color, Math.max(0.5, opts.pointSize));
    }
    ctx.restore();
  }
  plot.drawScatter(plotXs, plotYs, displayColors, Math.max(0.5, opts.pointSize));
  if (hasZ && zClamped > 0) {
    const edgePoints = (edge: PlotRangeEdge): [Float32Array, Float32Array] => {
      const edgeXs: number[] = [];
      const edgeYs: number[] = [];
      for (let index = 0; index < edgeMarks.length; index++) {
        if (edgeMarks[index] !== edge) continue;
        edgeXs.push(plotXs[index]);
        edgeYs.push(plotYs[index]);
      }
      return [Float32Array.from(edgeXs), Float32Array.from(edgeYs)];
    };
    const [lowXs, lowYs] = edgePoints("low");
    const [highXs, highYs] = edgePoints("high");
    plot.drawDiamonds(lowXs, lowYs, plot.theme.accent2, Math.max(2.5, opts.pointSize + 1));
    plot.drawDiamonds(highXs, highYs, plot.theme.warn, Math.max(2.5, opts.pointSize + 1));
  }

  const marginalHistograms = opts.marginals
    ? drawMarginals(plot, plotXs, plotYs, opts.bins, pointColor)
    : null;

  const population = screenCrossplotPopulation(
    xs,
    ys,
    opts.validityFilter,
    xValidity,
    yValidity,
    { min: xr.min, max: xr.max },
    { min: yr.min, max: yr.max },
    opts.xLog,
    opts.yLog,
  );
  plot.ctx.save();
  plot.ctx.font = canvasFont(plot.theme, 9);
  plot.ctx.fillStyle = plot.theme.axis;
  plot.ctx.textAlign = "right";
  const marginalSummary = marginalHistograms
    ? ` · marginal displayed n X=${marginalHistograms.x?.displayedTotal ?? 0}, Y=${marginalHistograms.y?.displayedTotal ?? 0}`
    : "";
  plot.ctx.fillText(
    `${formatPlotRangePolicySummary(population, {
      statistics: true,
      fitInputs: opts.regression ? population.analysisCount : null,
    })}${marginalSummary}${zExcluded ? ` · Z excluded=${zExcluded}` : ""}${zClamped ? ` · Z clamped/edge-marked=${zClamped}` : ""}`,
    plot.plotRect.x0 + plot.plotRect.w,
    plot.plotRect.y0 + plot.plotRect.h + 31,
  );
  plot.ctx.restore();

  // Where the next top-right legend block may start (the well legend stacks under the
  // facies legend when both are shown).
  let legendBottom = plot.plotRect.y0 + 8;
  if (categorical) {
    // Discrete facies legend: one swatch per class actually present.
    const { ctx } = plot;
    const r = plot.plotRect;
    const classes = distinctValues(plotZs);
    ctx.save();
    ctx.font = canvasFont(plot.theme, 10);
    const rowH = 15;
    const boxW = 78;
    const boxX = r.x0 + r.w - boxW - 8;
    let boxY = r.y0 + 8;
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(zName, boxX, boxY + 9);
    boxY += rowH;
    for (const c of classes) {
      ctx.fillStyle = faciesColor(c);
      ctx.fillRect(boxX, boxY, 11, 11);
      ctx.strokeStyle = plot.theme.text;
      ctx.lineWidth = 0.5;
      ctx.strokeRect(boxX, boxY, 11, 11);
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(faciesLabel(c), boxX + 16, boxY + 9);
      boxY += rowH;
    }
    ctx.restore();
    legendBottom = boxY + 6;
  } else if (hasZ && colors && !Number.isNaN(zLo)) {
    // Z color-bar legend in the chosen colormap ("log" tag when log-scaled).
    drawColorbar(plot, { map: opts.colormap, lo: zLo, hi: zHi, label: zName, log: opts.zLog });
  }

  // Well legend for the multi-well overlay: the active well first, then the context
  // wells in their layer colors. The footer states the interaction contract right on the
  // plot — context wells are display-only, every gesture acts on the active well.
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
    let boxY = legendBottom;
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
    // The active row's swatch only means something when the cloud is one color; a
    // Z-colored cloud gets a label instead of a misleading single swatch.
    const activeName = reducePlotLabel("context_well_name_characters", context!.activeName, "active").displayed;
    row(hasZ ? null : opts.color || plot.theme.accent, `${activeName} (active${hasZ ? `, by ${zName}` : ""})`);
    const layers = context!.layers;
    const visibleLegend = applyPlotRecordLimit("context_well_legend_rows", layers, "well_legend");
    for (const layer of visibleLegend.displayed) {
      row(layer.color, reducePlotLabel("context_well_name_characters", layer.name, layer.name).displayed);
    }
    if (visibleLegend.item) {
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(
        `context legend: ${visibleLegend.item.displayed_count} of ${visibleLegend.item.original_count} wells`,
        boxX + 16,
        boxY + 10,
      );
      boxY += rowH;
    }
    ctx.font = canvasFont(plot.theme, 9);
    ctx.fillText("context is display-only", boxX, boxY + 9);
    ctx.restore();
  }

  // User percentiles: dashed X (vertical) and Y (horizontal) reference lines.
  if (opts.percentiles.length > 0) {
    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.strokeStyle = plot.theme.text;
    ctx.fillStyle = plot.theme.text;
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 4]);
    ctx.font = canvasFont(plot.theme, 9);
    for (const p of opts.percentiles) {
      const vx = percentile(plotXs, p);
      if (!Number.isNaN(vx) && (!opts.xLog || vx > 0)) {
        const [px] = plot.toPx(vx, plot.y.min);
        if (px >= r.x0 && px <= r.x0 + r.w) {
          ctx.beginPath();
          ctx.moveTo(px, r.y0);
          ctx.lineTo(px, r.y0 + r.h);
          ctx.stroke();
          ctx.textAlign = "left";
          ctx.fillText(`P${p}`, px + 3, r.y0 + r.h - 4);
        }
      }
      const vy = percentile(plotYs, p);
      if (!Number.isNaN(vy) && (!opts.yLog || vy > 0)) {
        const [, py] = plot.toPx(plot.x.min, vy);
        if (py >= r.y0 && py <= r.y0 + r.h) {
          ctx.beginPath();
          ctx.moveTo(r.x0, py);
          ctx.lineTo(r.x0 + r.w, py);
          ctx.stroke();
          ctx.textAlign = "left";
          ctx.fillText(`P${p}`, r.x0 + 3, py - 3);
        }
      }
    }
    ctx.restore();
  }

  // Regression in the MODEL's space (drawn as a sampled polyline, so it's correct on
  // any axis scaling — a power fit is straight on log-log axes, curved on linear).
  if (opts.regression) {
    const fit = fitRegression(plotXs, plotYs, opts.regModel, opts.regMethod);
    if (fit) {
      const xNeedsLog = opts.regModel === "power" || opts.regModel === "logx";
      const yFromT = (t: number) => (opts.regModel === "power" || opts.regModel === "exp" ? Math.pow(10, t) : t);
      const points: [number, number][] = [];
      const N = 64;
      const lo = Math.min(xr.min, xr.max);
      const hi = Math.max(xr.min, xr.max);
      for (let i = 0; i <= N; i++) {
        // Sample uniformly in the DISPLAY axis's space so the polyline looks smooth.
        const x = opts.xLog
          ? Math.pow(10, Math.log10(Math.max(lo, 1e-30)) + (i / N) * (Math.log10(hi) - Math.log10(Math.max(lo, 1e-30))))
          : lo + (i / N) * (hi - lo);
        if (xNeedsLog && x <= 0) continue;
        const t = fit.a + fit.b * (xNeedsLog ? Math.log10(x) : x);
        points.push([x, yFromT(t)]);
      }
      plot.drawLine(points, plot.theme.accent2, 1.8, [7, 4]);
      const { ctx } = plot;
      const r = plot.plotRect;
      ctx.save();
      ctx.font = canvasFont(plot.theme, 10);
      ctx.fillStyle = plot.theme.text;
      ctx.textAlign = "left";
      ctx.fillText(
        `${fitEquation(fit, xName, yName, opts.regModel)}   (R² = ${fit.r2.toFixed(3)}, n = ${fit.n}${METHOD_LABEL[opts.regMethod]})`,
        r.x0 + 8,
        r.y0 + 14,
      );
      ctx.restore();
    }
  }

  // Matrix reference points on a genuine NPHI-RHOB plot (either orientation), opt-in.
  const isND = xName.toUpperCase() === "NPHI" && yName.toUpperCase() === "RHOB";
  const isDN = xName.toUpperCase() === "RHOB" && yName.toUpperCase() === "NPHI";
  if (opts.matrixPoints && (isND || isDN)) {
    for (const m of MATRIX_POINTS) {
      if (isND) plot.drawRefPoint(m.nphi, m.rhob, m.label);
      else plot.drawRefPoint(m.rhob, m.nphi, m.label);
    }
  }

  // Chartbook overlay — drawn when the plot axes match the chart's axes. Linear
  // axes only, except charts that themselves need a log axis (e.g. Th/K ratio).
  const overlayDef = opts.chartOverlay ? resolveChartOverlay(opts.chartOverlay) : undefined;
  if (overlayDef) {
    const decision = authorizeProvenancedChart(
      overlayDef,
      xName,
      yName,
      typedAxes?.x ?? null,
      typedAxes?.y ?? null,
      "screen",
    );
    if (decision.authorization && decision.record) {
      const flipped = decision.authorization.orientation === "flipped";
      const logOk = overlayDef.xLogNeeded
        ? (flipped ? opts.yLog && !opts.xLog : opts.xLog && !opts.yLog)
        : !opts.xLog && !opts.yLog;
      if (logOk) drawChartOverlay(plot, overlayDef, decision.authorization);
    } else if (decision.refusal) {
      drawChartProvenanceRefusal(plot, decision.refusal);
    }
  }

  // Synchronized hover: ring the sample at the depth under another view's cursor.
  if (hoverIdx >= 0 && hoverIdx < xs.length && validity.indices.includes(hoverIdx)) {
    const hx = xs[hoverIdx];
    const hy = ys[hoverIdx];
    if (!Number.isNaN(hx) && !Number.isNaN(hy) && (!opts.xLog || hx > 0) && (!opts.yLog || hy > 0)) {
      const [px, py] = plot.toPx(hx, hy);
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

/** Crossplot panel: any two catalog curves with an optional third as color (default
 *  NPHI-RHOB colored by GR). All display settings live in a Properties dialog opened
 *  by double-click, right-click, or the ⚙ button — plot size, marginal histograms,
 *  point color/size, percentile lines, log axes and ranges, Z colormap (log-safe
 *  viridis), regression model/method, and the overlays (core, T-S, matrix points).
 *  Draggable parameter handle writes picks to zone parameters (toggleable). Follows
 *  the synchronized depth cursor. */
export async function buildCrossplotContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well, { followSelectedInterval: false });
  trySelect(zoneSel.select, initial?.zone);
  const opts = normalizeCrossplotOptions(await loadPlotProps<CrossplotOptions>("crossplot"));
  const plotId = initial?.plotId ?? crypto.randomUUID();

  const content = document.createElement("div");
  content.className = "plot-content";
  const activeDepthHandoff = buildDepthReframeHandoff(setStatus);
  const contextDepthHandoff = buildDepthReframeHandoff(setStatus);
  const xSel = curveSelect(curveNames, initial?.x ?? "NPHI");
  const ySel = curveSelect(curveNames, initial?.y ?? "RHOB");
  // Z select with a "— None —" head option (universal: color only when wanted).
  const zSel = document.createElement("select");
  zSel.className = "form-control";
  {
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "— None —";
    zSel.appendChild(none);
    const wanted = initial?.z ?? "GR";
    for (const name of curveNames.includes(wanted) || wanted === "" ? curveNames : [wanted, ...curveNames]) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      zSel.appendChild(option);
    }
    zSel.value = wanted;
  }

  // --- Multi-well scope: extra wells drawn as a faded context layer BEHIND the active
  // well's cloud. Default "Active" keeps today's single-well behaviour exactly; the
  // active well's path (fetch, brushing, picks, zones, core, T-S, regression, tooltips)
  // is untouched — context wells are display-only by design, because a brushed depth
  // only means something relative to ONE well.
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
  scopeBtn.title = "Overlay more wells as a context layer behind the active well's points";
  const scopeRow = document.createElement("div");
  scopeRow.style.display = "none";
  const scopeStaticHint = document.createElement("p");
  scopeStaticHint.className = "modal-hint";
  scopeStaticHint.textContent =
    "Context wells are display-only: brushing, parameter picks, zone writes, core and T-S overlays act on the active well only. " +
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
    { channel: "x", semantic_request: xSel.value, required: true },
    { channel: "y", semantic_request: ySel.value, required: true },
    ...(zSel.value ? [{ channel: "colour", semantic_request: zSel.value, required: false }] : []),
  ];
  const representedWellIds = () => [well.well_id, ...ctxWellIds];
  let axisRanges: PlotAxisRangeExport[] = [];
  const currentAxisBindings = (): { x: PlotChannelBinding | null; y: PlotChannelBinding | null } => {
    const bindings = plotBindingSnapshotForChannels(representedWellIds(), plotIntents());
    return {
      x: bindings.find((binding) => binding.intent.channel === "x") ?? null,
      y: bindings.find((binding) => binding.intent.channel === "y") ?? null,
    };
  };
  const selectionState = (surface: ChartRenderSurface = "save"): Record<string, string> => {
    const chartProvenance = chartRecordForSelectedSurface(surface);
    return {
      plotId,
      x: xSel.value,
      y: ySel.value,
      z: zSel.value,
      zone: zoneSel.select.value,
      wells: scope.serialize(),
      chartProvenance: chartProvenance ? JSON.stringify(chartProvenance) : "",
    };
  };
  const persistedState = (options: Record<string, unknown>) =>
    buildPersistedPlotState("crossplot", options, representedWellIds(), plotIntents(), axisRanges);
  const persist = () => {
    try {
      void savePlotProps("crossplot", persistedChartState("save"))
        .catch((error) => setStatus(`Crossplot state not saved: ${error}`));
    } catch (error) {
      setStatus(`Crossplot state not saved: ${error}`);
    }
  };

  const propsBtn = document.createElement("button");
  propsBtn.className = "plot-export-btn";
  propsBtn.textContent = "⚙ Properties";
  propsBtn.title = "Crossplot properties (or double-click / right-click the plot)";
  propsBtn.addEventListener("click", () => openProps());

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("X", xSel));
  selRow.appendChild(formRow("Y", ySel));
  selRow.appendChild(formRow("Color", zSel));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(scopeBtn);
  selRow.appendChild(propsBtn);
  selRow.appendChild(
    buildPlotTemplateBar<CrossplotOptions>(
      "crossplot",
      "Crossplot",
      () => persistedChartState("template"),
      (t) => {
        Object.assign(opts, normalizeCrossplotOptions({ ...opts, ...t }));
        applySize();
        applyPicksVisibility();
        senseSel.value = opts.netSense; // a template can carry netSense — keep the dropdown in sync
        void reloadCore();
        redraw();
        persist();
      },
      setStatus,
    ),
  );
  const exportGroup = buildImageExportButtons(
    () => canvas,
    "Crossplot",
    setStatus,
    (scope) => getSvg(scope),
    (scope) => getPdf(scope),
    () => ctxReductionManifest,
    (surface) => {
      const chartSurface: ChartRenderSurface = surface === "svg" || surface === "pdf" ? surface : "save";
      const state = persistedState(selectionState(chartSurface));
      const chartRenderRecord = chartRecordForSelectedSurface(chartSurface);
      return {
        wellIds: state.well_ids,
        curves: plotIntents().map((intent) => intent.semantic_request),
        plotBindings: state.bindings,
        axisRanges: state.axis_ranges,
        statisticsRecords,
        chartRenderRecord: chartRenderRecord ?? undefined,
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
  content.appendChild(hint);
  const statisticsInfo = document.createElement("p");
  statisticsInfo.className = "modal-hint";
  statisticsInfo.style.whiteSpace = "pre-wrap";
  content.appendChild(statisticsInfo);
  const updateHint = () => {
    hint.textContent =
      (opts.showPicks
        ? "Drag the ringed handle to set the X/Y parameters (release writes them to the zone). Click empty space to reposition it. "
        : "") +
      "Double-click or right-click the plot for properties (when zoomed, double-click resets the zoom first). " +
      "Ctrl+wheel = zoom, drag background = pan.";
  };

  const tc = readTheme(document.documentElement);
  const pickX = pickRow("X pick", tc.accent, "NPHI_SH", well, zoneSel.current, setStatus, () => crossplotWriteSource("manual_pick"));
  const pickY = pickRow("Y pick", tc.accent2, "RHO_SH", well, zoneSel.current, setStatus, () => crossplotWriteSource("manual_pick"));
  const picksWrap = document.createElement("div");
  // Cutoff-region selector: shade a net quadrant anchored at the handle. The sense is chosen
  // explicitly here (no cutoff direction is inferred from the axes).
  const senseSel = document.createElement("select");
  senseSel.className = "form-control";
  for (const [val, label] of [
    ["off", "Net cutoff: off"],
    ["xge_yle", "Net: X ≥ pick, Y ≤ pick"],
    ["xle_yge", "Net: X ≤ pick, Y ≥ pick"],
    ["xge_yge", "Net: X ≥ pick, Y ≥ pick"],
    ["xle_yle", "Net: X ≤ pick, Y ≤ pick"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = label;
    senseSel.appendChild(o);
  }
  senseSel.value = opts.netSense;
  senseSel.title =
    "Shade the net-reservoir quadrant relative to the parameter handle and count the points inside it";
  senseSel.addEventListener("change", () => {
    opts.netSense = senseSel.value as NetSense;
    redraw();
    persist();
  });
  const senseWrap = document.createElement("div");
  senseWrap.style.margin = "2px 0 6px";
  senseWrap.appendChild(senseSel);
  picksWrap.append(senseWrap, pickX.row, pickY.row);
  content.appendChild(picksWrap);
  const applyPicksVisibility = () => {
    picksWrap.style.display = opts.showPicks ? "" : "none";
    updateHint();
  };
  applyPicksVisibility();

  /** Fixed size = explicit CSS box (consistent exported figures); fill = flex default. */
  const applySize = () => {
    if (opts.sizeMode === "fixed") {
      canvas.style.flex = "0 0 auto";
      canvas.style.width = `${opts.plotW}px`;
      canvas.style.height = `${opts.plotH}px`;
      canvas.style.maxWidth = "100%";
      canvas.style.alignSelf = "flex-start";
    } else {
      canvas.style.flex = "";
      canvas.style.width = "";
      canvas.style.height = "";
      canvas.style.maxWidth = "";
      canvas.style.alignSelf = "";
    }
  };
  applySize();

  // Accessibility: describe the chart for screen readers and make it keyboard-focusable.
  const ariaLabel = () =>
    `Crossplot: ${ySel.value} versus ${xSel.value}${zSel.value ? `, coloured by ${zSel.value}` : ""}`;
  makeCanvasAccessible(canvas, ariaLabel());

  let xs = new Float32Array(0);
  let ys = new Float32Array(0);
  let zs = new Float32Array(0);
  let depths = new Float32Array(0);
  let typedAxes: { x: ResolvedPlotCurve | null; y: ResolvedPlotCurve | null } = { x: null, y: null };
  let coreByName = new Map<string, TrackCurveSeries>();
  let plot: PlotCanvas | null = null;
  let marker: [number, number] | null = null;
  let hoverIdx = -1;
  let statisticsRecords: PlotStatisticsRecord[] = [];
  let statisticsSignature = "";
  // Shared-brush state: indices of the samples in the current brush (this well), plus the live
  // drag rectangle (CSS px) while a Shift+drag is in progress.
  let brushIdx: number[] = [];
  let brushRect: { x0: number; y0: number; x1: number; y1: number } | null = null;
  let brushing = false;
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
      dataGen,
      xSel.value,
      ySel.value,
      zone.depthMin,
      zone.depthMax,
      opts.validityFilter,
      opts.xValidMin,
      opts.xValidMax,
      opts.yValidMin,
      opts.yValidMax,
      opts.xLog,
      opts.yLog,
      plot.x.min,
      plot.x.max,
      plot.y.min,
      plot.y.max,
      selectionLabel,
    ]);
    if (signature === statisticsSignature) return;
    statisticsRecords = buildCrossplotStatisticsRecords(
      xs,
      ys,
      opts,
      xSel.value,
      ySel.value,
      well.well_id,
      zone.depthMin,
      zone.depthMax,
      { min: plot.x.min, max: plot.x.max },
      { min: plot.y.min, max: plot.y.max },
      selectionLabel,
    );
    statisticsSignature = signature;
    if (statisticsRecords.length === 0) {
      statisticsInfo.textContent = "No governed statistics population.";
      return;
    }
    statisticsInfo.textContent = statisticsRecords
      .map((record) => formatPlotStatisticsRecord(record))
      .join("\n");
  };
  // Free-form net-flag polygon: click to drop vertices (captured in DATA space so they track
  // zoom/pan), then write a discrete 0/1 net-reservoir flag curve from the polygon's interior.
  const lasso: { active: boolean; pts: [number, number][]; cursor: [number, number] | null } = {
    active: false,
    pts: [],
    cursor: null,
  };

  const resolvedBinding = (curveName: string): ResolvedPlotCurve | null =>
    plotBindingSnapshot([well.well_id], [curveName])
      .find((binding) => binding.intent.semantic_request.toUpperCase() === curveName.toUpperCase())
      ?.resolved[0] ?? null;

  function currentChartDecision(surface: ChartRenderSurface = "screen"): ChartRenderDecision {
    const def = opts.chartOverlay ? resolveChartOverlay(opts.chartOverlay) : undefined;
    return def
      ? authorizeProvenancedChart(def, xSel.value, ySel.value, typedAxes.x, typedAxes.y, surface)
      : { authorization: null, record: null, refusal: null };
  }

  function chartRecordForSelectedSurface(surface: ChartRenderSurface): ChartRenderRecord | null {
    if (!opts.chartOverlay) return null;
    const decision = currentChartDecision(surface);
    if (!decision.record) throw new Error(decision.refusal ?? `${surface} chart rendering is blocked`);
    return decision.record;
  }

  function persistedChartState(surface: "save" | "template") {
    const chartProvenance = chartRecordForSelectedSurface(surface);
    opts.chartProvenance = chartProvenance;
    return persistedState({ ...opts, chartProvenance });
  }

  function persistChartProvenanceIfChanged(): boolean {
    const next = currentChartDecision().record;
    if (JSON.stringify(opts.chartProvenance) === JSON.stringify(next)) return false;
    opts.chartProvenance = next;
    return true;
  }

  const crossplotWriteSource = async (method: string): Promise<PlotWriteSource> => {
    if (!plot) throw new Error("plot-derived write requires a rendered viewport");
    const zone = zoneSel.current();
    return {
      plot_id: plotId,
      plot_type: "crossplot",
      x_axis: plotWriteAxis("x", resolvedBinding(xSel.value)),
      y_axis: plotWriteAxis("y", resolvedBinding(ySel.value)),
      z_axis: zSel.value ? plotWriteAxis("z", resolvedBinding(zSel.value)) : null,
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
      method,
      fit_record: null,
    };
  };

  /** Recomputes which sample indices the shared brush covers (only when it targets THIS well;
   *  membership is exact on the shared depth grid). */
  const recomputeBrush = (sel: BrushSelection | null): void => {
    brushIdx = [];
    if (!sel || sel.wellId !== well.well_id) return;
    for (let i = 0; i < depths.length; i++) {
      if (sel.depths.has(depths[i])) brushIdx.push(i);
    }
  };

  // Memoized Z coloring — the redraw's heaviest step (two percentile sorts + an N-length
  // color array). It's viewport-independent, so recompute it only when the Z data or the
  // color settings actually change, not on every pan/zoom/hover frame. `dataGen` bumps
  // whenever reload() swaps in new arrays; colormap/zLog/color come from opts (a Properties
  // apply triggers a plain redraw, not a reload, so they must be part of the key).
  let dataGen = 0;
  let colorMemo: { key: string; value: CrossplotColors } | null = null;
  const zColors = (): CrossplotColors => {
    const key = `${zSel.value}\0${opts.colormap}\0${opts.zLog}\0${opts.color}\0${dataGen}`;
    if (!colorMemo || colorMemo.key !== key) {
      colorMemo = {
        key,
        value: computeCrossplotColors(zSel.value, zs, xs.length, opts.colormap, opts.zLog, opts.color),
      };
    }
    return colorMemo.value;
  };

  // The clean, publishable chart (frame, cloud, colorbar, and the static overlays) — the part
  // shared by the on-screen redraw and the vector-SVG export. Transient decorations (hover
  // marker, brush highlight, cutoff shading, parameter handle) are drawn only in redraw, so the
  // SVG export omits them. `hi` = hover index (-1 for a still export).
  const drawStatic = (target: HTMLCanvasElement, hi: number): PlotCanvas | null => {
    const p = drawCrossplot(
      target,
      xSel.value,
      ySel.value,
      zSel.value,
      xs,
      ys,
      zs,
      opts,
      hi,
      viewRef.current,
      zColors(),
      ctxLayers.length ? { activeName: well.well_name, layers: ctxLayers } : null,
      typedAxes,
      currentAxisBindings(),
      (ranges) => {
        axisRanges = ranges;
      },
    );
    if (!p) return null;
    if (opts.showCore) {
      const coreX = coreByName.get(CORE_OVERLAY_MAP[xSel.value.toUpperCase()] ?? "");
      const coreY = coreByName.get(CORE_OVERLAY_MAP[ySel.value.toUpperCase()] ?? "");
      if (coreX && coreY) {
        const { xs: cxs, ys: cys } = alignCoreSeriesByDepth(coreX, coreY);
        p.drawDiamonds(cxs, cys, p.theme.accent2);
      }
    }
    if (opts.tsOverlay) drawTsOverlay(p, opts.tsPhiSd, opts.tsPhiSh);
    if (opts.rockOverlay) {
      const orient = matchRockOverlayAxes(xSel.value, ySel.value);
      if (orient) drawRockOverlay(p, opts.rockOverlay, orient === "flipped");
    }
    return p;
  };

  // Vector export: the static chart re-run into a recording context sized to the live plot.
  const getSvg = (scope: PlotAncestryScope): string | null =>
    plot ? renderPlotToPaperSvg(plot.width, plot.height, (c) => drawStatic(c, -1), scope) : null;
  const getPdf = (scope: PlotAncestryScope): PlotPdf | null =>
    plot ? renderPlotToPaperPdf(plot.width, plot.height, (c) => drawStatic(c, -1), scope) : null;

  const redraw = () => {
    canvas.setAttribute("aria-label", ariaLabel()); // keep the a11y description in sync with the axes
    axisRanges = [];
    plot = drawStatic(canvas, hoverIdx);
    if (!plot) {
      refreshStatisticsRecords();
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const th = readTheme(canvas);
      ctx.font = canvasFont(th, 12);
      ctx.fillStyle = th.text;
      ctx.textAlign = "center";
      ctx.fillText("No valid data for these curves/zone.", canvas.width / 2, canvas.height / 2);
      return;
    }
    refreshStatisticsRecords();
    if (marker && opts.showPicks) {
      const [px, py] = plot.toPx(marker[0], marker[1]);
      const ctx = plot.ctx;
      ctx.save();
      ctx.strokeStyle = plot.theme.warn;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(px - 7, py);
      ctx.lineTo(px + 7, py);
      ctx.moveTo(px, py - 7);
      ctx.lineTo(px, py + 7);
      ctx.stroke();
      ctx.restore();
    }
    // Shared-brush highlight: emphasise the brushed samples; draw the live drag rectangle on top.
    if (plot && brushIdx.length) {
      const ctx = plot.ctx;
      const rp = plot.plotRect;
      ctx.save();
      ctx.beginPath();
      ctx.rect(rp.x0, rp.y0, rp.w, rp.h);
      ctx.clip();
      ctx.fillStyle = plot.theme.accent2;
      const rad = Math.max(1.8, opts.pointSize + 0.8);
      for (const i of brushIdx) {
        const vx = xs[i];
        const vy = ys[i];
        if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
        if (opts.xLog && vx <= 0) continue;
        if (opts.yLog && vy <= 0) continue;
        const [px, py] = plot.toPx(vx, vy);
        ctx.beginPath();
        ctx.arc(px, py, rad, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    }
    if (plot && brushRect) {
      const ctx = plot.ctx;
      const x = Math.min(brushRect.x0, brushRect.x1);
      const y = Math.min(brushRect.y0, brushRect.y1);
      const w = Math.abs(brushRect.x1 - brushRect.x0);
      const h = Math.abs(brushRect.y1 - brushRect.y0);
      ctx.save();
      ctx.fillStyle = plot.theme.accent2;
      ctx.globalAlpha = 0.12;
      ctx.fillRect(x, y, w, h);
      ctx.globalAlpha = 0.9;
      ctx.strokeStyle = plot.theme.accent2;
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 3]);
      ctx.strokeRect(x, y, w, h);
      ctx.restore();
    }
    drawCutoffRegion();
    drawParamHandle();
    drawLasso();
  };

  /** Cutoff-region overlay: turns the parameter handle into a pair of cutoff thresholds.
   *  Draws the vertical/horizontal cutoff lines through the handle, shades the user-chosen
   *  "net" quadrant, and reads out how many plotted points fall inside it. The quadrant is
   *  mapped from data to pixels via the axis extents, so it is correct under log / inverted
   *  axes. Off by default (opts.netSense === "off"). */
  const drawCutoffRegion = () => {
    if (!plot || !opts.showPicks || opts.netSense === "off") return;
    const cx = pickX.getValue();
    const cy = pickY.getValue();
    if (Number.isNaN(cx) || Number.isNaN(cy)) return;
    if ((opts.xLog && cx <= 0) || (opts.yLog && cy <= 0)) return;
    const xge = opts.netSense === "xge_yle" || opts.netSense === "xge_yge";
    const yge = opts.netSense === "xle_yge" || opts.netSense === "xge_yge";
    const ctx = plot.ctx;
    const rp = plot.plotRect;
    const [hx, hy] = plot.toPx(cx, cy);
    // The net side runs from the cutoff to whichever axis extent that sense points at; letting
    // toPx place both ends keeps the shaded box correct for inverted / log axes.
    const [ex, ey] = plot.toPx(xge ? plot.x.max : plot.x.min, yge ? plot.y.max : plot.y.min);
    const rx0 = Math.max(rp.x0, Math.min(hx, ex));
    const rx1 = Math.min(rp.x0 + rp.w, Math.max(hx, ex));
    const ry0 = Math.max(rp.y0, Math.min(hy, ey));
    const ry1 = Math.min(rp.y0 + rp.h, Math.max(hy, ey));
    ctx.save();
    ctx.beginPath();
    ctx.rect(rp.x0, rp.y0, rp.w, rp.h);
    ctx.clip();
    if (rx1 > rx0 && ry1 > ry0) {
      ctx.fillStyle = plot.theme.accent;
      ctx.globalAlpha = 0.1;
      ctx.fillRect(rx0, ry0, rx1 - rx0, ry1 - ry0);
      ctx.globalAlpha = 1;
    }
    ctx.strokeStyle = plot.theme.accent;
    ctx.lineWidth = 1;
    ctx.setLineDash([5, 4]);
    ctx.beginPath();
    ctx.moveTo(hx, rp.y0);
    ctx.lineTo(hx, rp.y0 + rp.h);
    ctx.moveTo(rp.x0, hy);
    ctx.lineTo(rp.x0 + rp.w, hy);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.restore();
    // Count plotted (finite, log-valid) points inside the net quadrant.
    let inN = 0;
    let tot = 0;
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (opts.xLog && vx <= 0) continue;
      if (opts.yLog && vy <= 0) continue;
      tot++;
      if ((xge ? vx >= cx : vx <= cx) && (yge ? vy >= cy : vy <= cy)) inN++;
    }
    const pct = tot ? (100 * inN) / tot : 0;
    ctx.save();
    ctx.font = canvasFont(plot.theme, 10);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(`net cutoff: ${inN} / ${tot} pts (${pct.toFixed(1)}%)`, rp.x0 + 6, rp.y0 + rp.h - 6);
    ctx.restore();
  };

  /** Draws the draggable parameter point at (X pick, Y pick) as a grabbable ring, so the
   *  user can drag it around the cloud to set the two zone parameters interactively. */
  const drawParamHandle = () => {
    if (!plot || !opts.showPicks) return;
    const vx = pickX.getValue();
    const vy = pickY.getValue();
    if (Number.isNaN(vx) || Number.isNaN(vy)) return;
    if ((opts.xLog && vx <= 0) || (opts.yLog && vy <= 0)) return;
    const [px, py] = plot.toPx(vx, vy);
    if (!plot.inPlot(px, py)) return;
    const ctx = plot.ctx;
    ctx.save();
    ctx.fillStyle = plot.theme.accent;
    ctx.strokeStyle = plot.theme.bg;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(px, py, 6, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.strokeStyle = plot.theme.accent;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(px, py, 9, 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  };

  // --- Free-form net-flag polygon ------------------------------------------
  const inLassoPoly = (dx: number, dy: number): boolean => netPolygonContains(dx, dy, lasso.pts, opts.xLog, opts.yLog);
  /** Live inside/total counts over the plotted (finite, log-valid) samples. */
  const lassoCount = (): { inside: number; total: number } => {
    let inside = 0;
    let total = 0;
    if (lasso.pts.length < 3) return { inside, total };
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (opts.xLog && vx <= 0) continue;
      if (opts.yLog && vy <= 0) continue;
      total++;
      if (inLassoPoly(vx, vy)) inside++;
    }
    return { inside, total };
  };

  const lassoBtn = document.createElement("button");
  lassoBtn.className = "plot-export-btn";
  lassoBtn.textContent = "⬡ Net polygon";
  lassoBtn.title = "Draw a free-form polygon on the cloud, then write its interior as a 0/1 net-flag curve";
  const lassoBar = document.createElement("div");
  lassoBar.style.display = "none";
  lassoBar.style.gap = "8px";
  lassoBar.style.alignItems = "center";
  lassoBar.style.margin = "2px 0 6px";
  const lassoInfo = document.createElement("span");
  lassoInfo.className = "modal-hint";
  const mkBar = (label: string, title: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.className = "plot-export-btn";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };
  const undoPt = mkBar("Undo point", "Remove the last polygon vertex", () => {
    lasso.pts.pop();
    redraw();
  });
  const clearPts = mkBar("Clear", "Discard the polygon", () => {
    lasso.pts = [];
    redraw();
  });
  const writePt = mkBar("Write net flag…", "Write the polygon interior as a 0/1 net-flag curve", () => openNetFlagDialog());
  lassoBar.append(lassoInfo, undoPt, clearPts, writePt);
  selRow.appendChild(lassoBtn);
  selRow.insertAdjacentElement("afterend", lassoBar);

  const setLassoActive = (on: boolean): void => {
    lasso.active = on;
    lassoBtn.style.fontWeight = on ? "700" : "";
    lassoBar.style.display = on ? "flex" : "none";
    canvas.style.cursor = on ? "crosshair" : "";
    if (!on) {
      lasso.pts = [];
      lasso.cursor = null;
    }
    redraw();
  };
  lassoBtn.addEventListener("click", () => setLassoActive(!lasso.active));

  /** Draws the in-progress polygon: faint interior fill, solid edges, dashed closing edge +
   *  rubber-band to the cursor, and vertex dots. Vertices are in data space, so it tracks
   *  zoom/pan. Updates the toolbar's live inside-count. */
  const drawLasso = (): void => {
    if (!plot || !lasso.active) return;
    const p = plot;
    if (lasso.pts.length) {
      const ctx = p.ctx;
      const rp = p.plotRect;
      const px = lasso.pts.map(([dx, dy]) => p.toPx(dx, dy));
      ctx.save();
      ctx.beginPath();
      ctx.rect(rp.x0, rp.y0, rp.w, rp.h);
      ctx.clip();
      if (px.length >= 3) {
        ctx.beginPath();
        ctx.moveTo(px[0][0], px[0][1]);
        for (let i = 1; i < px.length; i++) ctx.lineTo(px[i][0], px[i][1]);
        ctx.closePath();
        ctx.fillStyle = p.theme.accent2;
        ctx.globalAlpha = 0.12;
        ctx.fill();
        ctx.globalAlpha = 1;
      }
      ctx.strokeStyle = p.theme.accent2;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(px[0][0], px[0][1]);
      for (let i = 1; i < px.length; i++) ctx.lineTo(px[i][0], px[i][1]);
      ctx.stroke();
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      const last = px[px.length - 1];
      if (lasso.cursor) {
        ctx.moveTo(last[0], last[1]);
        ctx.lineTo(lasso.cursor[0], lasso.cursor[1]);
      }
      if (px.length >= 2) {
        ctx.moveTo(last[0], last[1]);
        ctx.lineTo(px[0][0], px[0][1]);
      }
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = p.theme.accent2;
      for (const [x, y] of px) {
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    }
    const { inside, total } = lassoCount();
    lassoInfo.textContent =
      lasso.pts.length < 3
        ? `Click to add points (${lasso.pts.length}/3 min)`
        : `${inside} / ${total} points inside`;
  };

  /** Names + writes the polygon interior as a 0/1 net-flag curve over the crossplot's depth window. */
  const openNetFlagDialog = (): void => {
    if (lasso.pts.length < 3) {
      setStatus("Draw at least 3 polygon points first");
      return;
    }
    const body = document.createElement("div");
    const nameIn = document.createElement("input");
    nameIn.className = "form-control";
    nameIn.value = "NET_FLAG";
    body.appendChild(
      formRow("Curve name", nameIn, "Writes/overwrites a 0/1 net-reservoir flag curve (NaN where a sample can't be evaluated)"),
    );
    const zone = zoneSel.current();
    const { inside, total } = lassoCount();
    const note = document.createElement("p");
    note.className = "modal-hint";
    note.textContent =
      `${inside} / ${total} plotted points inside, over ${zone.zoneName === "*" ? "the whole well" : `zone ${zone.zoneName}`}. ` +
      `X = ${xSel.value}, Y = ${ySel.value}.`;
    body.appendChild(note);
    const go = document.createElement("button");
    go.className = "lp-btn primary";
    go.textContent = "Write net flag";
    go.style.marginTop = "10px";
    body.appendChild(go);
    const close = openModal("Write Net Flag", body, 380);
    go.addEventListener("click", async () => {
      const name = nameIn.value.trim();
      if (!name) {
        setStatus("Net-flag curve needs a name");
        return;
      }
      const z = zoneSel.current();
      const custody = await requestRunCustody("Write net flag");
      if (!custody) return;
      const spec: NetFlagSpec = {
        well_id: well.well_id,
        x_curve: xSel.value,
        y_curve: ySel.value,
        x_log: opts.xLog,
        y_log: opts.yLog,
        polygon: lasso.pts.map(([x, y]) => [x, y] as [number, number]),
        output_curve: name,
        depth_top: z.depthMin,
        depth_bottom: z.depthMax,
        custody,
      };
      go.disabled = true;
      void runNetFlag(spec)
        .then((res) => {
          setStatus(`Net flag ${res.output_curve}: ${res.inside} / ${res.evaluated} samples net (${res.written} written)`);
          bumpDataVersion(); // refresh selectors / log views / other plots so the new curve shows up
          setLassoActive(false);
          close();
        })
        .catch((err) => {
          setStatus(`Net flag failed: ${err}`);
          go.disabled = false;
        });
    });
  };

  // Monotonic token so a slow curve/zone load that resolves after a newer one (fast
  // switching) can't overwrite the newer axes' data. `preserveView` keeps the zoom/pan
  // AND the placed parameter handle on a data refresh (module run); a user-initiated
  // axis/zone change still re-fits and re-seeds the handle.
  let reloadGen = 0;
  // A reset-intent reload (axis/zone change) can be superseded by a preserveView data
  // refresh; this sticky flag carries the "reset viewport + re-seed handle" intent to
  // whichever reload commits, so a background bump can't strand new axes at the old zoom.
  let resetPending = false;
  const reload = async (preserveView = false) => {
    const token = beginPlotAsyncGeneration("crossplot-data-refetch", ++reloadGen);
    if (!preserveView) resetPending = true;
    const zone = zoneSel.current();
    const wanted = [xSel.value, ySel.value, ...(zSel.value ? [zSel.value] : [])];
    activeDepthHandoff.clear();
    try {
      const series = await getCurveData(well.well_id, wanted, zone.depthMin, zone.depthMax);
      if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return;
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      const required = wanted.map((name) => byName.get(name.toUpperCase()));
      if (required.some((item) => !item)) throw new Error("one or more required plot curves are absent");
      const reconciled = reconcileDepthChannels(required.map((item) => ({
        depth: item!.depth,
        values: item!.value,
      })));
      xs = reconciled.channels[0];
      ys = reconciled.channels[1];
      zs = zSel.value ? reconciled.channels[2] : new Float32Array(0);
      depths = reconciled.depth;
      if (reconciled.mode === "decimated_to_coarsest") {
        setStatus(`Crossplot depth inputs decimated to the coarsest exact step; factors ${reconciled.decimationFactors.join("/")} · interval ${reconciled.intervalClosure}`);
      }
      const bindings = plotBindingSnapshot([well.well_id], [xSel.value, ySel.value]);
      typedAxes = {
        x: bindings.find((binding) => binding.intent.semantic_request === xSel.value)?.resolved[0] ?? null,
        y: bindings.find((binding) => binding.intent.semantic_request === ySel.value)?.resolved[0] ?? null,
      };
    } catch (err) {
      if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return;
      const handoff = depthReframeHandoff(err, [well.well_id], wanted);
      activeDepthHandoff.show(handoff);
      setStatus(handoff ? `Crossplot refused: ${handoff.reason}` : `Crossplot data load failed: ${err}`);
      xs = ys = zs = depths = new Float32Array(0);
      typedAxes = { x: null, y: null };
    }
    const chartProvenanceChanged = persistChartProvenanceIfChanged();
    hoverIdx = -1; // the old hover index may point at a different sample now
    if (resetPending) {
      resetPending = false;
      marker = null;
      viewRef.current = null; // new data → reset any zoom/pan
      // Seed the draggable parameter point at the cloud's median so it's always visible and
      // grabbable (the user drags it to set the shale/matrix point); only when still unset.
      if (Number.isNaN(pickX.getValue()) || Number.isNaN(pickY.getValue())) {
        const mx = percentile(xs, 50);
        const my = percentile(ys, 50);
        if (!Number.isNaN(mx) && !Number.isNaN(my)) {
          pickX.setValue(mx);
          pickY.setValue(my);
          marker = [mx, my];
        }
      }
    }
    recomputeBrush(appState.brushedDepths.get()); // depths grid changed — re-map the brush
    dataGen++; // new arrays are in place — invalidate the memoized Z coloring
    redraw();
    if (chartProvenanceChanged) persist();
  };

  /** Loads the well's core data once (all four series; cheap — core datasets are
   *  small) whenever the "Core data" toggle is switched on. Its own token (separate
   *  from reload's — it writes only coreByName, disjoint from the curve arrays) so a
   *  rapid core toggle supersedes an in-flight core fetch without dropping a data reload. */
  let coreGen = 0;
  const reloadCore = async () => {
    const token = beginPlotAsyncGeneration("crossplot-core-refetch", ++coreGen);
    if (!opts.showCore) {
      coreByName = new Map();
      redraw();
      return;
    }
    try {
      const series = await getCoreData(well.well_id);
      if (!isPlotAsyncGenerationCurrent(token, coreGen)) return;
      coreByName = new Map(series.map((s) => [s.curve_name, s]));
    } catch (err) {
      if (!isPlotAsyncGenerationCurrent(token, coreGen)) return;
      setStatus(`Core data load failed: ${err}`);
      coreByName = new Map();
    }
    redraw();
  };

  // --- Context-well data (multi-well overlay) -------------------------------
  // Total point budget across ALL context wells: 2,000 wells × ~5,000 samples is 10M
  // points — far past what canvas 2D (or the eye) can use. Each well gets an equal
  // share and is stride-decimated down to it; the scope row reports the decimation.
  let ctxLayers: ContextWellLayer[] = [];
  let ctxWellIds: string[] = [];
  let ctxReductionManifest: PlotReductionExport | null = null;
  let ctxInfo = "";
  let ctxGen = 0;

  /** Fetches the scoped context wells' X/Y data through the shared plotCommon machinery
   *  (per-well zone/top-by-name windows, point budget, concurrency, cancellation via the
   *  generation token). Scope = just the active well → clears the overlay: byte-identical
   *  single-well behaviour. */
  const reloadContext = async () => {
    const token = beginPlotAsyncGeneration("crossplot-context-refetch", ++ctxGen);
    contextDepthHandoff.clear();
    let resolvedIds: string[];
    try {
      resolvedIds = await resolveWellScope(scope.backend());
    } catch (error) {
      if (isPlotAsyncGenerationCurrent(token, ctxGen)) setStatus(`Crossplot scope refused: ${error}`);
      return;
    }
    refreshStatisticsRecords();
    if (!isPlotAsyncGenerationCurrent(token, ctxGen)) return;
    const ids = resolvedIds.filter((id) => id !== well.well_id);
    if (ids.length === 0) {
      const had = ctxLayers.length > 0;
      ctxLayers = [];
      ctxWellIds = [];
      ctxReductionManifest = null;
      ctxInfo = "";
      contextDepthHandoff.clear();
      updateScopeUi();
      if (had) redraw();
      return;
    }
    ctxReductionManifest = contextReductionExport(
      "crossplot",
      null,
      resolvedIds.length,
    );
    setStatus(`Crossplot: loading ${ids.length} context well${ids.length === 1 ? "" : "s"}…`);
    const outcome = await fetchContextLayers({
      ids,
      names: scope.namesFor(ids),
      curves: [xSel.value, ySel.value],
      windowFor: (id) => contextZoneWindow(zoneSel, id),
      budget: plotRecordLimit("context_point_budget").maximum,
      isStale: () => !isPlotAsyncGenerationCurrent(token, ctxGen),
    });
    if (!outcome) return; // superseded by a newer call (or dispose)
    ctxReductionManifest = contextReductionExport(
      "crossplot",
      outcome,
      resolvedIds.length,
      { wellId: well.well_id, name: well.well_name },
    );
    ctxLayers = outcome.layers.map((l) => ({
      name: l.name,
      color: l.color,
      xs: l.series.get(xSel.value.toUpperCase())!,
      ys: l.series.get(ySel.value.toUpperCase())!,
    }));
    ctxWellIds = outcome.layers.map((layer) => layer.wellId);
    ctxInfo = describeContextOutcome(outcome);
    contextDepthHandoff.show(mergeDepthReframeHandoffs(outcome.depthReframeHandoffs));
    updateScopeUi();
    setStatus(`Crossplot ${ctxInfo.toLowerCase()}`);
    redraw();
  };
  updateScopeUi();

  /** T-S triangle lives on VSH (0–1) vs PHIT axes; on any other pair (e.g. the default
   *  NPHI-RHOB) it lands entirely off-scale and looks like nothing happened. When it's
   *  switched on, auto-switch the axes to a VSH/porosity pair when the well has one.
   *  Returns true when a reload was triggered (axes changed). */
  const tsAutoAxes = (): boolean => {
    if (!/VSH|VCL/i.test(xSel.value) || !/PHI/i.test(ySel.value)) {
      const names = (sel: HTMLSelectElement) => [...sel.options].map((o) => o.value);
      const pick = (cands: string[], pats: RegExp[]) => {
        for (const p of pats) {
          const hit = cands.find((n) => p.test(n));
          if (hit) return hit;
        }
        return null;
      };
      const vsh = pick(names(xSel), [/^VSH$/i, /^VSH/i, /^VCL/i]);
      const phi = pick(names(ySel), [/^PHIT$/i, /^PHIT/i, /^PHIE/i, /^PHI/i]);
      if (vsh && phi) {
        xSel.value = vsh;
        ySel.value = phi;
        setStatus(`T-S triangle: axes switched to ${vsh} vs ${phi}`);
        xSel.dispatchEvent(new Event("change")); // one reload with both new axes
        return true;
      }
      setStatus("T-S triangle needs X = VSH and Y = PHIT — run VSH + Porosity modules first");
    }
    return false;
  };

  // Properties dialog — everything about the display in one place (matches Histogram v2).
  const openProps = () => {
    const body = document.createElement("div");

    const chk = (label: string, checked: boolean): { el: HTMLElement; input: HTMLInputElement } => {
      const wrap = document.createElement("label");
      wrap.className = "chk-field";
      const input = document.createElement("input");
      input.type = "checkbox";
      input.checked = checked;
      wrap.append(input, document.createTextNode(label));
      return { el: wrap, input };
    };
    const num = (value: number | null, width = 62, placeholder = "auto"): HTMLInputElement => {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.style.width = `${width}px`;
      input.placeholder = placeholder;
      if (value !== null) input.value = String(value);
      return input;
    };
    const sel = (pairs: [string, string][], value: string): HTMLSelectElement => {
      const select = document.createElement("select");
      select.className = "form-control";
      for (const [v, label] of pairs) {
        const option = document.createElement("option");
        option.value = v;
        option.textContent = label;
        select.appendChild(option);
      }
      select.value = value;
      return select;
    };
    const section = (label: string): HTMLElement => {
      const el = document.createElement("div");
      el.className = "props-section";
      el.textContent = label;
      return el;
    };
    const inline = (...els: (HTMLElement | string)[]): HTMLElement => {
      const wrap = document.createElement("div");
      wrap.style.display = "flex";
      wrap.style.gap = "10px";
      wrap.style.alignItems = "center";
      wrap.style.flexWrap = "wrap";
      wrap.append(...els);
      return wrap;
    };

    // --- Plot ---
    const sizeSel = sel(
      [
        ["fill", "Fill panel"],
        ["fixed", "Fixed size"],
      ],
      opts.sizeMode,
    );
    const wIn = num(opts.plotW, 62, "");
    const hIn = num(opts.plotH, 62, "");
    const sizeDims = inline(wIn, "×", hIn);
    const syncSizeDims = () => {
      wIn.disabled = hIn.disabled = sizeSel.value !== "fixed";
    };
    sizeSel.addEventListener("change", syncSizeDims);
    syncSizeDims();
    const pointIn = num(opts.pointSize, 48, "");
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
    const marginalsChk = chk("Marginal histograms (X top, Y right)", opts.marginals);
    const binsIn = num(opts.bins, 52, "");
    binsIn.min = String(HISTOGRAM_BINS_MIN);
    binsIn.max = String(HISTOGRAM_BINS_MAX);
    const pctIn = document.createElement("input");
    pctIn.className = "form-control";
    pctIn.placeholder = "e.g. 10, 50, 90";
    pctIn.value = opts.percentiles.join(", ");

    // --- Axes ---
    const xLogChk = chk("X log", opts.xLog);
    const yLogChk = chk("Y log", opts.yLog);
    const xMinIn = num(opts.xMin);
    const xMaxIn = num(opts.xMax);
    const yMinIn = num(opts.yMin);
    const yMaxIn = num(opts.yMax);
    const validityChk = chk("Apply validity filters", opts.validityFilter);
    const xValidMinIn = num(opts.xValidMin);
    const xValidMaxIn = num(opts.xValidMax);
    const yValidMinIn = num(opts.yValidMin);
    const yValidMaxIn = num(opts.yValidMax);

    // --- Z color ---
    const zLogChk = chk("Log Z scale", opts.zLog);
    const cmapSel = sel(
      [
        ["rainbow", "Rainbow (blue→red)"],
        ["viridis", "Viridis (log-safe)"],
      ],
      opts.colormap,
    );

    // --- Regression ---
    const regChk = chk("Regression line", opts.regression);
    const modelSel = sel(
      [
        ["linear", "Linear  Y = a + b·X"],
        ["power", "Power  Y = A·X^b"],
        ["logx", "Log  Y = a + b·log10(X)"],
        ["exp", "Exponential  Y = 10^(a + b·X)"],
      ],
      opts.regModel,
    );
    const methodSel = sel(
      [
        ["yx", "Y on X (ordinary)"],
        ["xy", "X on Y (inverse)"],
        ["rma", "RMA (symmetric)"],
      ],
      opts.regMethod,
    );

    // --- Overlays ---
    const coreChk = chk("Core data (diamonds)", opts.showCore);
    const tsChk = chk("T-S triangle", opts.tsOverlay);
    const matrixChk = chk("Matrix points (Qtz/Cal/Dol on NPHI-RHOB)", opts.matrixPoints);
    // Chart-overlay select, grouped: charts matching the CURRENT axes first.
    const chartSel = document.createElement("select");
    chartSel.className = "form-control";
    const noneOpt = document.createElement("option");
    noneOpt.value = "";
    noneOpt.textContent = "— None —";
    chartSel.appendChild(noneOpt);
    const applicable = allChartOverlays().filter((d) => matchOverlayAxes(d, xSel.value, ySel.value));
    const other = allChartOverlays().filter((d) => !matchOverlayAxes(d, xSel.value, ySel.value));
    for (const [groupLabel, list] of [
      ["For these axes", applicable],
      ["Other axes (drawn only when axes match)", other],
    ] as [string, ChartOverlayDef[]][]) {
      if (!list.length) continue;
      const group = document.createElement("optgroup");
      group.label = groupLabel;
      for (const d of list) {
        const option = document.createElement("option");
        option.value = d.id;
        const source = (d as ChartOverlayDef & { provenance?: ChartSourceProvenance }).provenance;
        option.textContent = `${d.label}${source ? "" : " — BLOCKED: source provenance absent"}`;
        group.appendChild(option);
      }
      chartSel.appendChild(group);
    }
    chartSel.value = opts.chartOverlay && resolveChartOverlay(opts.chartOverlay) ? opts.chartOverlay : "";
    // Rock-typing pore-throat grid (Winland/Pittman) — drawn when the axes are a φ-k pair.
    const rockSel = document.createElement("select");
    rockSel.className = "form-control";
    for (const [val, lbl] of [
      ["", "— None —"],
      ["winland", "Winland R35 (Kolodzie 1980)"],
      ["pittman_r25", "Pittman r25 (1992)"],
      ["pittman_r35", "Pittman r35 (1992)"],
      ["pittman_r50", "Pittman r50 (1992)"],
    ] as const) {
      const option = document.createElement("option");
      option.value = val;
      option.textContent = lbl;
      rockSel.appendChild(option);
    }
    rockSel.value = opts.rockOverlay;
    const picksChk = chk("Show parameter pickers (drag handle → zone parameters)", opts.showPicks);

    body.appendChild(section("Plot"));
    body.appendChild(formRow("Size", inline(sizeSel, sizeDims)));
    body.appendChild(formRow("Point px", pointIn));
    body.appendChild(formRow("Point color", inline(themeChk.el, colorIn), "Used when Color is — None — (Z coloring wins otherwise)"));
    body.appendChild(inline(marginalsChk.el, formRow("Bins", binsIn)));
    body.appendChild(formRow("Percentiles", pctIn, "Dashed X/Y reference lines, comma-separated (0–100)"));
    body.appendChild(section("Axes"));
    body.appendChild(inline(xLogChk.el, yLogChk.el));
    body.appendChild(formRow("X range", inline(xMinIn, xMaxIn), "Blank = header display, then audited unit-family display, then finite data"));
    body.appendChild(formRow("Y range", inline(yMinIn, yMaxIn)));
    body.appendChild(validityChk.el);
    body.appendChild(formRow("X valid", inline(xValidMinIn, xValidMaxIn), "Blank = no X validity exclusion"));
    body.appendChild(formRow("Y valid", inline(yValidMinIn, yValidMaxIn), "Filtering changes n, statistics and fits; display clipping does not"));
    body.appendChild(section("Z color"));
    body.appendChild(inline(formRow("Colormap", cmapSel), zLogChk.el));
    body.appendChild(section("Regression"));
    body.appendChild(inline(regChk.el, modelSel, methodSel));
    body.appendChild(section("Overlays"));
    body.appendChild(inline(coreChk.el, tsChk.el));
    body.appendChild(matrixChk.el);
    body.appendChild(formRow("Chart overlay", chartSel, "Chartbook curves/regions digitized at vector precision — drawn when the plot axes match the chart"));
    body.appendChild(formRow("Rock-type grid", rockSel, "Pore-throat iso-radius lines at the port-class bounds (0.1/0.5/2.5/10 µm) — drawn when one axis is porosity and the other permeability (use log k)"));
    body.appendChild(picksChk.el);

    const applyBtn = document.createElement("button");
    applyBtn.className = "lp-btn primary";
    applyBtn.textContent = "Apply";
    applyBtn.style.marginTop = "12px";
    body.appendChild(applyBtn);

    const close = openModal("Crossplot Properties", body, 470);
    applyBtn.addEventListener("click", () => {
      const parseOrNull = (input: HTMLInputElement): number | null => {
        const v = input.value.trim() === "" ? NaN : parseFloat(input.value);
        return Number.isNaN(v) ? null : v;
      };
      const coreWas = opts.showCore;
      const tsWas = opts.tsOverlay;
      opts.sizeMode = sizeSel.value as SizeMode;
      opts.plotW = parseOrNull(wIn) ?? opts.plotW;
      opts.plotH = parseOrNull(hIn) ?? opts.plotH;
      opts.pointSize = parseOrNull(pointIn) ?? DEFAULT_CROSSPLOT_OPTIONS.pointSize;
      opts.color = themeChk.input.checked ? "" : colorIn.value;
      opts.marginals = marginalsChk.input.checked;
      opts.bins = parseOrNull(binsIn) ?? DEFAULT_CROSSPLOT_OPTIONS.bins;
      opts.percentiles = parsePercentiles(pctIn.value);
      opts.xLog = xLogChk.input.checked;
      opts.yLog = yLogChk.input.checked;
      opts.xMin = parseOrNull(xMinIn);
      opts.xMax = parseOrNull(xMaxIn);
      opts.yMin = parseOrNull(yMinIn);
      opts.yMax = parseOrNull(yMaxIn);
      opts.validityFilter = validityChk.input.checked;
      opts.xValidMin = parseOrNull(xValidMinIn);
      opts.xValidMax = parseOrNull(xValidMaxIn);
      opts.yValidMin = parseOrNull(yValidMinIn);
      opts.yValidMax = parseOrNull(yValidMaxIn);
      opts.zLog = zLogChk.input.checked;
      opts.colormap = cmapSel.value as ColormapName;
      opts.regression = regChk.input.checked;
      opts.regModel = modelSel.value as RegModel;
      opts.regMethod = methodSel.value as RegMethod;
      opts.showCore = coreChk.input.checked;
      opts.tsOverlay = tsChk.input.checked;
      opts.matrixPoints = matrixChk.input.checked;
      opts.chartOverlay = chartSel.value;
      opts.rockOverlay = rockSel.value;
      opts.showPicks = picksChk.input.checked;
      Object.assign(opts, normalizeCrossplotOptions({ ...opts }));
      applySize();
      applyPicksVisibility();
      let reloading = false;
      if (opts.tsOverlay && !tsWas) reloading = tsAutoAxes();
      if (opts.showCore !== coreWas) void reloadCore();
      if (!reloading) {
        redraw();
        persist();
      }
      const chartDecision = currentChartDecision();
      setStatus(chartDecision.refusal ?? "Crossplot properties applied");
      close();
    });
  };

  for (const sel of [xSel, ySel, zSel, zoneSel.select]) {
    sel.addEventListener("change", () => {
      void reload();
      // Context wells share the X/Y axes and the zone window; the Z curve is active-well only.
      if (sel !== zSel) void reloadContext();
    });
  }

  // --- Interactive handles: parameter point + Thomas-Stieber endpoints -----
  // The parameter point is a draggable 2-D handle at (X pick, Y pick): dragging it moves
  // both picks live, and releasing writes them to the selected zone's parameters (e.g. a
  // shale point → NPHI_SH + RHO_SH). The T-S endpoints (when that overlay is on) drag
  // vertically as before. Empty-space drag pans and the wheel zooms (attachZoomPan); a
  // press landing on a handle vetoes the pan so the drag stays clean. Coordinates are in
  // logical (CSS) pixels — the same space PlotCanvas.toPx/toData use post-HiDPI.
  type DragMode = "param" | "ts-sand" | "ts-shale" | null;
  let drag: DragMode = null;
  let downXY: [number, number] | null = null;
  let movedSinceDown = false;

  const canvasPx = (e: MouseEvent): [number, number] => {
    const rect = canvas.getBoundingClientRect();
    return [e.clientX - rect.left, e.clientY - rect.top];
  };
  const paramName = (row: HTMLElement): string =>
    (row.querySelector(".pick-param") as HTMLInputElement | null)?.value.trim().toUpperCase() ?? "";

  /** Which draggable handle (if any) is under a screen point. */
  const handleAt = (px: number, py: number): DragMode => {
    if (!plot) return null;
    if (opts.tsOverlay) {
      for (const [which, vx, vy] of [
        ["ts-sand", 0, opts.tsPhiSd] as const,
        ["ts-shale", 1, opts.tsPhiSh] as const,
      ]) {
        const [hx, hy] = plot.toPx(vx, vy);
        if (Math.hypot(px - hx, py - hy) <= 9) return which;
      }
    }
    if (!opts.showPicks) return null;
    const vx = pickX.getValue();
    const vy = pickY.getValue();
    if (!Number.isNaN(vx) && !Number.isNaN(vy) && (!opts.xLog || vx > 0) && (!opts.yLog || vy > 0)) {
      const [hx, hy] = plot.toPx(vx, vy);
      if (Math.hypot(px - hx, py - hy) <= 10) return "param";
    }
    return null;
  };

  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0 || !plot) return;
    const [px, py] = canvasPx(e);
    // Net-polygon mode owns every left-click: drop a vertex (data space) and swallow the event so
    // attachZoomPan can't pan and the trailing click can't drop a parameter pick.
    if (lasso.active) {
      if (plot.inPlot(px, py)) {
        lasso.pts.push(plot.toData(px, py));
        movedSinceDown = true;
        e.preventDefault();
        e.stopImmediatePropagation();
        redraw();
      }
      return;
    }
    // Shift+drag inside the plot = brush-select. Takes precedence over pan/handle: stop the event
    // reaching attachZoomPan's mousedown (registered later on this canvas), and mark movedSinceDown
    // so the trailing click doesn't drop a parameter pick.
    if (e.shiftKey && plot.inPlot(px, py)) {
      brushing = true;
      brushRect = { x0: px, y0: py, x1: px, y1: py };
      movedSinceDown = true;
      e.preventDefault();
      e.stopImmediatePropagation();
      return;
    }
    downXY = [px, py];
    movedSinceDown = false;
    drag = handleAt(px, py);
    if (drag) e.preventDefault();
  });

  canvas.addEventListener("mousemove", (e) => {
    if (!plot) return;
    const [px, py] = canvasPx(e);
    if (lasso.active) {
      // Track the cursor for the rubber-band edge (only redraw once there's a segment to rubber-band).
      lasso.cursor = plot.inPlot(px, py) ? [px, py] : null;
      if (lasso.pts.length) redraw();
      return;
    }
    if (brushing && brushRect) {
      brushRect.x1 = px;
      brushRect.y1 = py;
      redraw();
      return;
    }
    if (downXY && Math.hypot(px - downXY[0], py - downXY[1]) > 4) movedSinceDown = true;
    if (drag) {
      const [vx, vy] = plot.toData(px, py);
      if (drag === "param") {
        pickX.setValue(vx);
        pickY.setValue(vy);
        marker = [vx, vy];
      } else {
        const clamped = Math.min(0.5, Math.max(0, vy));
        if (drag === "ts-sand") opts.tsPhiSd = clamped;
        else opts.tsPhiSh = clamped;
      }
      redraw();
      return;
    }
    if (!downXY) canvas.style.cursor = handleAt(px, py) ? "grab" : "";
  });

  const endDrag = () => {
    const mode = drag;
    drag = null;
    downXY = null;
    if (!mode || !movedSinceDown) return;
    const zone = zoneSel.current();
    const write = (param: string, value: number, dp: (v: number) => string, method: string) => {
      if (!param || Number.isNaN(value)) return;
      void crossplotWriteSource(method)
        .then((source) => writePlotParameter({ well, zone, parameter: param, value, source }))
        .then(() => setStatus(`${param} = ${dp(value)} set with plot provenance (Ctrl+Z undoes)`))
        .catch((err) => setStatus(`Failed to set ${param}: ${err}`));
    };
    if (mode === "param") {
      write(paramName(pickX.row), pickX.getValue(), (v) => v.toPrecision(4), "manual_handle_drag");
      write(paramName(pickY.row), pickY.getValue(), (v) => v.toPrecision(4), "manual_handle_drag");
    } else {
      persist();
      if (mode === "ts-sand") write("PHI_SD_MAX", opts.tsPhiSd, (v) => v.toFixed(3), "thomas_stieber_handle_drag");
      else write("PHI_SH", opts.tsPhiSh, (v) => v.toFixed(3), "thomas_stieber_handle_drag");
    }
  };
  canvas.addEventListener("mouseup", endDrag);

  canvas.addEventListener("click", (e) => {
    if (lasso.active) return; // clicks build the polygon, not parameter picks
    if (!plot || movedSinceDown || !opts.showPicks) return; // drag/pan tail, or pickers off
    const [px, py] = canvasPx(e);
    if (!plot.inPlot(px, py)) return;
    const [vx, vy] = plot.toData(px, py);
    marker = [vx, vy];
    pickX.setValue(vx);
    pickY.setValue(vy);
    redraw();
  });

  // Double-click = properties, unless a zoom is active (then attachZoomPan — registered
  // after this listener — resets it on the same event). Right-click is deliberately NOT
  // bound here: it belongs to the pane context menu (which lists Properties… first), so
  // the plots keep the same split/float/export actions every other pane has.
  canvas.addEventListener("dblclick", () => {
    if (lasso.active) return; // a double-click while lassoing just drops two vertices, no dialog
    if (!viewRef.current) openProps();
  });

  const detachZoomPan = attachZoomPan({
    canvas,
    getPlot: () => plot,
    view: viewRef,
    redraw,
    onPanStart: (px, py) => handleAt(px, py) === null, // a handle grab vetoes panning
  });
  const detachKeys = attachKeyboardPanZoom({
    canvas,
    getPlot: () => plot,
    view: viewRef,
    redraw,
    getLabel: ariaLabel,
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

  // Finish a brush on mouseup anywhere (the release may land outside the canvas): collect the
  // samples inside the rectangle and publish their depths; a tiny rectangle clears the selection.
  const onBrushEnd = () => {
    if (!brushing) return;
    brushing = false;
    const rect = brushRect;
    brushRect = null;
    if (!rect || !plot) {
      redraw();
      return;
    }
    const x = Math.min(rect.x0, rect.x1);
    const y = Math.min(rect.y0, rect.y1);
    const w = Math.abs(rect.x1 - rect.x0);
    const h = Math.abs(rect.y1 - rect.y0);
    if (w < 3 || h < 3) {
      clearBrush();
      redraw();
      return;
    }
    const sel = new Set<number>();
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (opts.xLog && vx <= 0) continue;
      if (opts.yLog && vy <= 0) continue;
      const [px, py] = plot.toPx(vx, vy);
      if (px >= x && px <= x + w && py >= y && py <= y + h) {
        const d = depths[i];
        if (Number.isFinite(d)) sel.add(d);
      }
    }
    setBrushedDepths(well.well_id, sel); // fires our own subscribe → recompute + redraw
  };
  window.addEventListener("mouseup", onBrushEnd);

  const invalidation = registerPlotInvalidationContract(canvas, {
    theme: () => redraw(),
    dataRevision: () => {
      // Re-fetch after module/equation runs, imports and undo while preserving the viewport
      // and placed parameter handle. Context wells can have changed in the same revision.
      void reload(true);
      void reloadContext();
    },
    interval: (interval) => zoneSel.applySelectedInterval(interval, true),
    selection: (selection) => {
      recomputeBrush(selection);
      if (!rafId) {
        rafId = requestAnimationFrame(() => {
          rafId = 0;
          redraw();
        });
      }
    },
    size: () => redraw(),
    cancelPending: () => {
      reloadGen++;
      coreGen++;
      ctxGen++;
      if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = 0;
      }
    },
  });

  // Local hover tooltip: the sample values under the cursor (X/Y/Z + depth), independent of the
  // depth-synced ring. Suppressed while dragging a handle so it doesn't fight the pick gesture.
  const detachTip = attachScatterTooltip(canvas, (px, py) => {
    if (drag || !plot || !plot.inPlot(px, py)) return null;
    let best = -1;
    let bestD = 12 * 12; // within a 12 px radius
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (opts.xLog && vx <= 0) continue;
      if (opts.yLog && vy <= 0) continue;
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
    lines.push(`${xSel.value}  ${fmtValue(xs[best])}`);
    lines.push(`${ySel.value}  ${fmtValue(ys[best])}`);
    if (zSel.value && best < zs.length && Number.isFinite(zs[best])) lines.push(`${zSel.value}  ${fmtValue(zs[best])}`);
    return lines;
  });

  await reload();
  await reloadCore();
  // Not awaited: a big scope (hundreds of wells) must not block the panel build — the
  // active well's plot appears immediately and the context layer fades in when ready.
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
      window.removeEventListener("mouseup", onBrushEnd);
      zoneSel.dispose();
    },
    getState: selectionState,
    getPersistedState: () => persistedState(selectionState()),
    bindingReady,
    openProperties: openProps,
  };
}
