import { getCoreData, getCurveData, setZoneParam, type TrackCurveSeries, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow, openModal } from "./modal";
import {
  attachResizeRedraw,
  attachZoomPan,
  categoricalColors,
  colormapColor,
  colorRampEx,
  distinctValues,
  faciesColor,
  fitCanvasBackingStore,
  looksDiscrete,
  percentile,
  PlotCanvas,
  canvasFont,
  readTheme,
  type ColormapName,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import { parsePercentiles } from "./histogramPanel";
import { AXIS_ALIASES, CHART_OVERLAYS, findChartOverlay, type ChartOverlayDef } from "./chartOverlays";
import {
  buildPlotTemplateBar,
  buildZoneSelect,
  CORE_OVERLAY_MAP,
  curveSelect,
  loadCurveNames,
  loadPlotProps,
  nearestDepthIndex,
  pickRow,
  savePlotProps,
  trySelect,
  type PlotContent,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

export type RegModel = "linear" | "power" | "logx" | "exp";
export type RegMethod = "yx" | "xy" | "rma";
export type SizeMode = "fill" | "fixed";

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
  /** Manual axis ranges; null = auto (mnemonic defaults, else P2–P98). */
  xMin: number | null;
  xMax: number | null;
  yMin: number | null;
  yMax: number | null;
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
}

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
  showCore: false,
  tsOverlay: false,
  tsPhiSd: 0.3,
  tsPhiSh: 0.15,
  matrixPoints: false,
  rockOverlay: "",
  chartOverlay: "",
  sizeMode: "fill",
  plotW: 640,
  plotH: 480,
  marginals: false,
  bins: 40,
  color: "",
  percentiles: [],
  zLog: false,
  colormap: "rainbow",
  showPicks: true,
};

/** Fills defaults and sanitizes saved/template options. v1 props carried no regModel —
 *  the old fit ran in the axes' own lin/log space, so derive the equivalent model from
 *  the axis-log flags to keep saved por-perm regressions meaning the same thing. */
export function normalizeCrossplotOptions(raw: Partial<CrossplotOptions>): CrossplotOptions {
  const opts: CrossplotOptions = { ...DEFAULT_CROSSPLOT_OPTIONS, ...raw };
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
  if (typeof opts.chartOverlay !== "string" || (opts.chartOverlay !== "" && !findChartOverlay(opts.chartOverlay))) {
    opts.chartOverlay = "";
  }
  if (!["", "winland", "pittman_r25", "pittman_r35", "pittman_r50"].includes(opts.rockOverlay)) {
    opts.rockOverlay = "";
  }
  opts.plotW = Math.max(200, Math.min(2000, Math.round(opts.plotW) || DEFAULT_CROSSPLOT_OPTIONS.plotW));
  opts.plotH = Math.max(200, Math.min(2000, Math.round(opts.plotH) || DEFAULT_CROSSPLOT_OPTIONS.plotH));
  opts.bins = Math.max(5, Math.min(200, Math.round(opts.bins) || DEFAULT_CROSSPLOT_OPTIONS.bins));
  opts.color = typeof opts.color === "string" ? opts.color : "";
  opts.percentiles = Array.isArray(opts.percentiles) ? parsePercentiles(opts.percentiles.join(",")) : [];
  if (opts.colormap !== "viridis") opts.colormap = "rainbow";
  opts.pointSize = Math.max(0.5, Math.min(8, opts.pointSize || DEFAULT_CROSSPLOT_OPTIONS.pointSize));
  return opts;
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

/** Default axis ranges by mnemonic; anything else auto-ranges from P2–P98. */
function axisDefaults(curve: string): { min: number; max: number; invert: boolean } | null {
  switch (curve.toUpperCase()) {
    case "NPHI":
      return { min: -0.05, max: 0.6, invert: false };
    case "RHOB":
      return { min: 1.9, max: 3.0, invert: true }; // density increases downward (D-N convention)
    case "GR":
      return { min: 0, max: 200, invert: false };
    case "DT":
      return { min: 40, max: 190, invert: false };
    case "VSH":
    case "PHIE":
    case "PHIT":
    case "SWE":
    case "SWT":
      return { min: 0, max: 1, invert: false };
    default:
      return null;
  }
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

/** Generic chartbook overlay renderer: matrix curves with graduation dots (every
 *  5) + numeric labels (every labelEvery) + along-slope names, dashed
 *  iso-graduation connectors, reference lines, mineral-region polygons, and
 *  labeled reference points. Everything is drawn in data space, so it stays
 *  registered under zoom/pan and on either axis orientation. */
function drawChartOverlay(plot: PlotCanvas, def: ChartOverlayDef, flipped: boolean): void {
  const { ctx } = plot;
  const r = plot.plotRect;
  const XY = (x: number, y: number): [number, number] => (flipped ? [y, x] : [x, y]);

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

/** Bin counts over the axis's own space (log axes bin log-uniformly so bars align with
 *  the display); returns null when nothing falls in range. */
function marginalCounts(
  values: ArrayLike<number>,
  lo: number,
  hi: number,
  bins: number,
  log: boolean,
): number[] | null {
  const t = (v: number) => (log ? Math.log10(v) : v);
  const tLo = t(lo);
  const tHi = t(hi);
  if (!Number.isFinite(tLo) || !Number.isFinite(tHi) || tLo === tHi) return null;
  const counts = new Array(bins).fill(0);
  let any = false;
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v) || (log && v <= 0)) continue;
    const f = (t(v) - tLo) / (tHi - tLo);
    if (f < 0 || f > 1) continue;
    counts[Math.min(bins - 1, Math.floor(f * bins))]++;
    any = true;
  }
  return any ? counts : null;
}

/** Marginal histograms in the widened top (X) and right (Y) margins, aligned with the
 *  plot's axes (fraction-based so log scales and inverted axes stay in register). */
function drawMarginals(plot: PlotCanvas, xs: Float32Array, ys: Float32Array, bins: number, color: string): void {
  const r = plot.plotRect;
  const { ctx } = plot;
  const stripH = plot.margin.top - 12;
  const stripW = plot.margin.right - 16;
  ctx.save();
  ctx.fillStyle = color;
  ctx.globalAlpha = 0.55;
  const cx = marginalCounts(xs, Math.min(plot.x.min, plot.x.max), Math.max(plot.x.min, plot.x.max), bins, plot.x.log);
  if (cx && stripH > 6) {
    const peak = Math.max(...cx);
    const bw = r.w / bins;
    for (let i = 0; i < bins; i++) {
      if (cx[i] === 0) continue;
      const f = (i + 0.5) / bins;
      const px = r.x0 + (plot.x.invert ? 1 - f : f) * r.w;
      const h = (cx[i] / peak) * stripH;
      ctx.fillRect(px - bw / 2 + 0.5, r.y0 - 4 - h, Math.max(1, bw - 1), h);
    }
  }
  const cy = marginalCounts(ys, Math.min(plot.y.min, plot.y.max), Math.max(plot.y.min, plot.y.max), bins, plot.y.log);
  if (cy && stripW > 6) {
    const peak = Math.max(...cy);
    const bh = r.h / bins;
    for (let i = 0; i < bins; i++) {
      if (cy[i] === 0) continue;
      const f = (i + 0.5) / bins;
      const py = r.y0 + (plot.y.invert ? f : 1 - f) * r.h;
      const w = (cy[i] / peak) * stripW;
      ctx.fillRect(r.x0 + r.w + 4, py - bh / 2 + 0.5, w, Math.max(1, bh - 1));
    }
  }
  ctx.restore();
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
  if (categorical) {
    colors = categoricalColors(zs);
  } else if (hasZ) {
    // Log Z: percentile over the positive values only, else ≤0 junk wrecks the range.
    const zForRange = zLog ? Float32Array.from([...zs].filter((v) => !Number.isNaN(v) && v > 0)) : zs;
    zLo = percentile(zForRange, 5);
    zHi = percentile(zForRange, 95);
    if (!Number.isNaN(zLo) && zLo !== zHi) colors = colorRampEx(zs, zLo, zHi, colormap, zLog);
  }
  if (!colors && fillColor) colors = new Array(pointCount).fill(fillColor);
  return { colors, categorical, zLo, zHi };
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
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const auto = (values: Float32Array): { min: number; max: number } | null => {
    const lo = percentile(values, 2);
    const hi = percentile(values, 98);
    if (Number.isNaN(lo) || Number.isNaN(hi) || lo === hi) return null;
    const pad = (hi - lo) * 0.08;
    return { min: lo - pad, max: hi + pad };
  };
  /** Manual range > mnemonic default > P2–P98 auto; log axes get a positive floor. */
  const resolve = (
    name: string,
    values: Float32Array,
    log: boolean,
    manMin: number | null,
    manMax: number | null,
  ): { min: number; max: number; invert: boolean } | null => {
    const base = axisDefaults(name) ?? (auto(values) ? { ...auto(values)!, invert: false } : null);
    if (!base && (manMin === null || manMax === null)) return null;
    let min = manMin ?? base!.min;
    let max = manMax ?? base!.max;
    const invert = base?.invert ?? false;
    if (log) {
      if (max <= 0) return null;
      if (min <= 0) {
        let smallest = Infinity;
        for (let i = 0; i < values.length; i++) {
          const v = values[i];
          if (!Number.isNaN(v) && v > 0 && v < smallest) smallest = v;
        }
        min = Number.isFinite(smallest) ? smallest * 0.8 : max / 1000;
      }
    }
    if (min === max) return null;
    return { min, max, invert };
  };

  const xr = resolve(xName, xs, opts.xLog, opts.xMin, opts.xMax);
  const yr = resolve(yName, ys, opts.yLog, opts.yMin, opts.yMax);
  if (!xr || !yr) return null;
  // A zoom/pan viewport (if any) overrides the computed window, keeping axis inversion.
  if (view) {
    xr.min = view.xMin;
    xr.max = view.xMax;
    yr.min = view.yMin;
    yr.max = view.yMax;
  }

  const plot = new PlotCanvas(
    canvas,
    { label: xName, min: xr.min, max: xr.max, log: opts.xLog, invert: xr.invert },
    { label: yName, min: yr.min, max: yr.max, log: opts.yLog, invert: yr.invert },
    opts.marginals ? { top: 56, right: 64 } : undefined,
  );
  plot.drawFrame();
  const pointColor = opts.color || plot.theme.accent;

  // Z coloring only when a Z curve is selected. Discrete class curves (electrofacies,
  // clusters) get categorical coloring + a swatch legend; continuous curves get the chosen
  // colormap (optionally log10-scaled) + a color bar. This is the redraw's heaviest step and
  // is viewport-independent, so the panel memoizes it and passes it in; we only compute it
  // here when a caller (or a test) doesn't supply one.
  const hasZ = zName !== "" && zs.length > 0;
  const { colors, categorical, zLo, zHi } =
    precolors ?? computeCrossplotColors(zName, zs, xs.length, opts.colormap, opts.zLog, opts.color);
  plot.drawScatter(xs, ys, colors, Math.max(0.5, opts.pointSize));

  if (opts.marginals) drawMarginals(plot, xs, ys, opts.bins, pointColor);

  if (categorical) {
    // Discrete facies legend: one swatch per class actually present.
    const { ctx } = plot;
    const r = plot.plotRect;
    const classes = distinctValues(zs);
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
      ctx.fillText(`F${c}`, boxX + 16, boxY + 9);
      boxY += rowH;
    }
    ctx.restore();
  } else if (hasZ && colors && !Number.isNaN(zLo)) {
    // Z color-bar legend in the chosen colormap ("log" tag when log-scaled).
    const { ctx } = plot;
    const r = plot.plotRect;
    const barW = 90;
    const barX = r.x0 + r.w - barW - 8;
    const barY = r.y0 + 8;
    for (let i = 0; i < barW; i++) {
      ctx.fillStyle = colormapColor(opts.colormap, i / (barW - 1));
      ctx.fillRect(barX + i, barY, 1, 8);
    }
    ctx.fillStyle = plot.theme.text;
    ctx.font = canvasFont(plot.theme, 9);
    ctx.textAlign = "center";
    ctx.fillText(zLo.toPrecision(3), barX, barY + 18);
    ctx.fillText(opts.zLog ? `${zName} (log)` : zName, barX + barW / 2, barY + 18);
    ctx.fillText(zHi.toPrecision(3), barX + barW, barY + 18);
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
      const vx = percentile(xs, p);
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
      const vy = percentile(ys, p);
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
    const fit = fitRegression(xs, ys, opts.regModel, opts.regMethod);
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
  const overlayDef = opts.chartOverlay ? findChartOverlay(opts.chartOverlay) : undefined;
  if (overlayDef) {
    const orient = matchOverlayAxes(overlayDef, xName, yName);
    if (orient) {
      const flipped = orient === "flipped";
      const logOk = overlayDef.xLogNeeded
        ? (flipped ? opts.yLog && !opts.xLog : opts.xLog && !opts.yLog)
        : !opts.xLog && !opts.yLog;
      if (logOk) drawChartOverlay(plot, overlayDef, flipped);
    }
  }

  // Synchronized hover: ring the sample at the depth under another view's cursor.
  if (hoverIdx >= 0 && hoverIdx < xs.length) {
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
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);
  const opts = normalizeCrossplotOptions(await loadPlotProps<CrossplotOptions>("crossplot"));

  const content = document.createElement("div");
  content.className = "plot-content";
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

  const persist = () => savePlotProps("crossplot", opts);

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
  selRow.appendChild(propsBtn);
  selRow.appendChild(
    buildPlotTemplateBar<CrossplotOptions>(
      "crossplot",
      "Crossplot",
      () => ({ ...opts }),
      (t) => {
        Object.assign(opts, normalizeCrossplotOptions({ ...opts, ...t }));
        persist();
        applySize();
        applyPicksVisibility();
        void reloadCore();
        redraw();
      },
      setStatus,
    ),
  );
  selRow.appendChild(buildImageExportButtons(() => canvas, "Crossplot", setStatus));
  content.appendChild(selRow);

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 460;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  content.appendChild(hint);
  const updateHint = () => {
    hint.textContent =
      (opts.showPicks
        ? "Drag the ringed handle to set the X/Y parameters (release writes them to the zone). Click empty space to reposition it. "
        : "") +
      "Double-click or right-click the plot for properties (when zoomed, double-click resets the zoom first). " +
      "Ctrl+wheel = zoom, drag background = pan.";
  };

  const tc = readTheme(document.documentElement);
  const pickX = pickRow("X pick", tc.accent, "NPHI_SH", well, zoneSel.current, setStatus);
  const pickY = pickRow("Y pick", tc.accent2, "RHO_SH", well, zoneSel.current, setStatus);
  const picksWrap = document.createElement("div");
  picksWrap.append(pickX.row, pickY.row);
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

  let xs = new Float32Array(0);
  let ys = new Float32Array(0);
  let zs = new Float32Array(0);
  let depths = new Float32Array(0);
  let coreByName = new Map<string, TrackCurveSeries>();
  let plot: PlotCanvas | null = null;
  let marker: [number, number] | null = null;
  let hoverIdx = -1;
  const viewRef: ViewportRef = { current: null };

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

  const redraw = () => {
    plot = drawCrossplot(canvas, xSel.value, ySel.value, zSel.value, xs, ys, zs, opts, hoverIdx, viewRef.current, zColors());
    if (!plot) {
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const th = readTheme(canvas);
      ctx.font = canvasFont(th, 12);
      ctx.fillStyle = th.text;
      ctx.textAlign = "center";
      ctx.fillText("No valid data for these curves/zone.", canvas.width / 2, canvas.height / 2);
      return;
    }
    if (opts.showCore) {
      const coreX = coreByName.get(CORE_OVERLAY_MAP[xSel.value.toUpperCase()] ?? "");
      const coreY = coreByName.get(CORE_OVERLAY_MAP[ySel.value.toUpperCase()] ?? "");
      if (coreX && coreY) {
        const { xs: cxs, ys: cys } = alignCoreSeriesByDepth(coreX, coreY);
        plot.drawDiamonds(cxs, cys, plot.theme.accent2);
      }
    }
    if (opts.tsOverlay) {
      drawTsOverlay(plot, opts.tsPhiSd, opts.tsPhiSh);
    }
    if (opts.rockOverlay) {
      const orient = matchRockOverlayAxes(xSel.value, ySel.value);
      if (orient) drawRockOverlay(plot, opts.rockOverlay, orient === "flipped");
    }
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
    drawParamHandle();
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
    const gen = ++reloadGen;
    if (!preserveView) resetPending = true;
    const zone = zoneSel.current();
    try {
      const wanted = [xSel.value, ySel.value, ...(zSel.value ? [zSel.value] : [])];
      const series = await getCurveData(well.well_id, wanted, zone.depthMin, zone.depthMax);
      if (gen !== reloadGen) return; // a newer reload started while we awaited
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      xs = byName.get(xSel.value.toUpperCase())?.value ?? new Float32Array(0);
      ys = byName.get(ySel.value.toUpperCase())?.value ?? new Float32Array(0);
      zs = zSel.value ? (byName.get(zSel.value.toUpperCase())?.value ?? new Float32Array(0)) : new Float32Array(0);
      depths = byName.get(xSel.value.toUpperCase())?.depth ?? new Float32Array(0);
    } catch (err) {
      if (gen !== reloadGen) return; // superseded — don't clobber newer data with this error
      setStatus(`Crossplot data load failed: ${err}`);
      xs = ys = zs = depths = new Float32Array(0);
    }
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
    dataGen++; // new arrays are in place — invalidate the memoized Z coloring
    redraw();
  };

  /** Loads the well's core data once (all four series; cheap — core datasets are
   *  small) whenever the "Core data" toggle is switched on. Its own token (separate
   *  from reload's — it writes only coreByName, disjoint from the curve arrays) so a
   *  rapid core toggle supersedes an in-flight core fetch without dropping a data reload. */
  let coreGen = 0;
  const reloadCore = async () => {
    const gen = ++coreGen;
    if (!opts.showCore) {
      coreByName = new Map();
      redraw();
      return;
    }
    try {
      const series = await getCoreData(well.well_id);
      if (gen !== coreGen) return; // a newer core load started while we awaited
      coreByName = new Map(series.map((s) => [s.curve_name, s]));
    } catch (err) {
      if (gen !== coreGen) return;
      setStatus(`Core data load failed: ${err}`);
      coreByName = new Map();
    }
    redraw();
  };

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
    const applicable = CHART_OVERLAYS.filter((d) => matchOverlayAxes(d, xSel.value, ySel.value));
    const other = CHART_OVERLAYS.filter((d) => !matchOverlayAxes(d, xSel.value, ySel.value));
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
        option.textContent = d.label;
        group.appendChild(option);
      }
      chartSel.appendChild(group);
    }
    chartSel.value = opts.chartOverlay && findChartOverlay(opts.chartOverlay) ? opts.chartOverlay : "";
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
    body.appendChild(formRow("X range", inline(xMinIn, xMaxIn), "Blank = auto (mnemonic default, else P2–P98)"));
    body.appendChild(formRow("Y range", inline(yMinIn, yMaxIn)));
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
      persist();
      applySize();
      applyPicksVisibility();
      let reloading = false;
      if (opts.tsOverlay && !tsWas) reloading = tsAutoAxes();
      if (opts.showCore !== coreWas) void reloadCore();
      if (!reloading) redraw();
      setStatus("Crossplot properties applied");
      close();
    });
  };

  for (const sel of [xSel, ySel, zSel, zoneSel.select]) {
    sel.addEventListener("change", () => void reload());
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
    downXY = [px, py];
    movedSinceDown = false;
    drag = handleAt(px, py);
    if (drag) e.preventDefault();
  });

  canvas.addEventListener("mousemove", (e) => {
    if (!plot) return;
    const [px, py] = canvasPx(e);
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
    const write = (param: string, value: number, dp: (v: number) => string) => {
      if (!param || Number.isNaN(value)) return;
      void setZoneParam(well.well_id, zone.zoneName, param, value, null)
        .then(() => setStatus(`${param} = ${dp(value)} set on zone '${zone.zoneName}' of ${well.well_name}`))
        .catch((err) => setStatus(`Failed to set ${param}: ${err}`));
    };
    if (mode === "param") {
      write(paramName(pickX.row), pickX.getValue(), (v) => v.toPrecision(4));
      write(paramName(pickY.row), pickY.getValue(), (v) => v.toPrecision(4));
    } else {
      persist();
      if (mode === "ts-sand") write("PHI_SD_MAX", opts.tsPhiSd, (v) => v.toFixed(3));
      else write("PHI_SH", opts.tsPhiSh, (v) => v.toFixed(3));
    }
  };
  canvas.addEventListener("mouseup", endDrag);

  canvas.addEventListener("click", (e) => {
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
  // after this listener — resets it on the same event). Right-click = properties.
  canvas.addEventListener("dblclick", () => {
    if (!viewRef.current) openProps();
  });
  canvas.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
    openProps();
  });

  const detachZoomPan = attachZoomPan({
    canvas,
    getPlot: () => plot,
    view: viewRef,
    redraw,
    onPanStart: (px, py) => handleAt(px, py) === null, // a handle grab vetoes panning
  });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

  // Re-fetch when computed curves change (module/equation run, import, undo) so the
  // crossplot never shows stale data; keep the zoom/pan and the placed parameter handle.
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    void reload(true);
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

  await reload();
  await reloadCore();
  return {
    el: content,
    dispose: () => {
      unsubHover();
      unsubTheme();
      unsubData();
      detachZoomPan();
      detachResize();
      zoneSel.dispose();
    },
    getState: () => ({ x: xSel.value, y: ySel.value, z: zSel.value, zone: zoneSel.select.value }),
  };
}
