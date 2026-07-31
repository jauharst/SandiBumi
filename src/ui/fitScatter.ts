import { attachResizeRedraw, attachScatterTooltip, canvasFont, faciesColor, fitCanvasBackingStore, readTheme } from "./plotCanvas";
import { buildImageExportButtons } from "./plotExport";
import { setStatus } from "../state";

/** QC scatter for a calibration fit — shared by the RtC and S-factor dialogs.
 *
 *  A calibration reduces a core or a water leg to two or three numbers, and R² alone cannot tell
 *  you *how* it failed. Curvature, a single well sitting off the trend, a cluster of plugs the
 *  fit is being dragged by: all of those read instantly from the scatter and not at all from the
 *  coefficient table above it. That is the whole reason both backends return their points.
 *
 *  Two rules that are specific to a FIT scatter and are the point of sharing this code:
 *
 *  **The reference line's slope must be honest on screen.** For a measured-vs-fitted plot the
 *  axes are forced to the SAME range so the 1:1 line lands at 45° — scale them independently and
 *  a perfect fit looks biased, or a biased one looks perfect, purely from the aspect ratio.
 *
 *  **A through-the-origin fit must show the origin.** Proportionality is the model's claim, so
 *  cropping to the data hides whether the cloud actually heads for zero, which is the one thing
 *  that would falsify it.
 */
export interface FitScatterPoint {
  x: number;
  y: number;
  /** Colour key — the well, so a single well pulling the calibration is visible. */
  group: string;
  /** Tooltip lines for this point. */
  detail: string[];
}

export type FitScatterLine =
  /** y = x. Measured against fitted; axes are forced square. */
  | { kind: "identity" }
  /** y = slope·x. Axes are forced to include the origin. */
  | { kind: "origin"; slope: number };

export interface FitScatterSpec {
  points: FitScatterPoint[];
  xLabel: string;
  yLabel: string;
  line: FitScatterLine;
  caption: string;
  /** Base name for the exported image file. */
  exportName: string;
}

interface Frame {
  padL: number;
  padR: number;
  padT: number;
  padB: number;
  xmin: number;
  xmax: number;
  ymin: number;
  ymax: number;
  w: number;
  h: number;
}

function nice(v: number): string {
  if (!Number.isFinite(v)) return "—";
  const a = Math.abs(v);
  if (a === 0) return "0";
  if (a < 0.001 || a >= 100000) return v.toExponential(1);
  if (a < 1) return v.toFixed(4);
  if (a < 100) return v.toFixed(2);
  return v.toFixed(0);
}

/** Rounded tick step for a span, so labels land on readable numbers. */
function tickStep(span: number): number {
  if (!(span > 0) || !Number.isFinite(span)) return 1;
  const raw = span / 4;
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const norm = raw / mag;
  const step = norm >= 5 ? 5 : norm >= 2 ? 2 : 1;
  return step * mag;
}

function frameFor(spec: FitScatterSpec, w: number, h: number): Frame | null {
  const pts = spec.points.filter((p) => Number.isFinite(p.x) && Number.isFinite(p.y));
  if (!pts.length) return null;
  let xmin = Math.min(...pts.map((p) => p.x));
  let xmax = Math.max(...pts.map((p) => p.x));
  let ymin = Math.min(...pts.map((p) => p.y));
  let ymax = Math.max(...pts.map((p) => p.y));

  if (spec.line.kind === "identity") {
    // Same window on both axes, or the 1:1 line is not at 45 degrees and the eye reads a bias
    // that is an artefact of the aspect ratio.
    const lo = Math.min(xmin, ymin);
    const hi = Math.max(xmax, ymax);
    xmin = ymin = lo;
    xmax = ymax = hi;
  } else {
    // Proportionality is the claim being made, so the origin has to be on the page.
    xmin = Math.min(0, xmin);
    ymin = Math.min(0, ymin);
    xmax = Math.max(0, xmax);
    ymax = Math.max(0, ymax);
  }
  const padX = (xmax - xmin) * 0.04 || 1;
  const padY = (ymax - ymin) * 0.04 || 1;
  return {
    padL: 52,
    padR: 10,
    padT: 10,
    padB: 30,
    xmin: xmin - padX,
    xmax: xmax + padX,
    ymin: ymin - padY,
    ymax: ymax + padY,
    w,
    h,
  };
}

function draw(canvas: HTMLCanvasElement, spec: FitScatterSpec): void {
  const theme = readTheme(canvas);
  const dpr = fitCanvasBackingStore(canvas);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  // The backing store is sized in DEVICE pixels; everything below is written in CSS pixels, so
  // the transform has to carry the ratio or a HiDPI screen draws the whole plot at half scale in
  // the top-left corner. Same idiom as cutoffDialog/histogramPanel.
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const w = canvas.clientWidth || 380;
  const h = canvas.clientHeight || 220;
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);

  const fr = frameFor(spec, w, h);
  if (!fr) {
    ctx.fillStyle = theme.text;
    ctx.font = canvasFont(theme, 11, 400);
    ctx.textAlign = "center";
    ctx.fillText("no points returned", w / 2, h / 2);
    return;
  }
  const X = (v: number) => fr.padL + ((v - fr.xmin) / (fr.xmax - fr.xmin)) * (w - fr.padL - fr.padR);
  const Y = (v: number) => fr.padT + (1 - (v - fr.ymin) / (fr.ymax - fr.ymin)) * (h - fr.padT - fr.padB);

  // Ticks and gridlines.
  ctx.font = canvasFont(theme, 9, 400);
  ctx.strokeStyle = theme.grid;
  ctx.lineWidth = 1;
  const sx = tickStep(fr.xmax - fr.xmin);
  const sy = tickStep(fr.ymax - fr.ymin);
  ctx.fillStyle = theme.text;
  ctx.textAlign = "center";
  for (let v = Math.ceil(fr.xmin / sx) * sx; v <= fr.xmax; v += sx) {
    const px = X(v);
    ctx.beginPath();
    ctx.moveTo(px, fr.padT);
    ctx.lineTo(px, h - fr.padB);
    ctx.stroke();
    ctx.fillText(nice(v), px, h - fr.padB + 12);
  }
  ctx.textAlign = "right";
  for (let v = Math.ceil(fr.ymin / sy) * sy; v <= fr.ymax; v += sy) {
    const py = Y(v);
    ctx.beginPath();
    ctx.moveTo(fr.padL, py);
    ctx.lineTo(w - fr.padR, py);
    ctx.stroke();
    ctx.fillText(nice(v), fr.padL - 4, py + 3);
  }

  // Axes.
  ctx.strokeStyle = theme.axis;
  ctx.beginPath();
  ctx.moveTo(fr.padL, fr.padT);
  ctx.lineTo(fr.padL, h - fr.padB);
  ctx.lineTo(w - fr.padR, h - fr.padB);
  ctx.stroke();

  // The reference line, drawn UNDER the points so a dense cloud is not hidden by it.
  const slope = spec.line.kind === "identity" ? 1 : spec.line.slope;
  if (Number.isFinite(slope)) {
    ctx.strokeStyle = theme.accent;
    ctx.lineWidth = 1.6;
    ctx.setLineDash(spec.line.kind === "identity" ? [5, 4] : []);
    ctx.beginPath();
    ctx.moveTo(X(fr.xmin), Y(slope * fr.xmin));
    ctx.lineTo(X(fr.xmax), Y(slope * fr.xmax));
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // Points, coloured by well. Anything outside the window is SKIPPED, not clamped to an edge —
  // a point pinned to the frame states a value it does not have (same rule as the point tracks).
  const groups = [...new Set(spec.points.map((p) => p.group))];
  const colorOf = new Map(groups.map((g, i) => [g, faciesColor(i)]));
  for (const p of spec.points) {
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
    if (p.x < fr.xmin || p.x > fr.xmax || p.y < fr.ymin || p.y > fr.ymax) continue;
    ctx.fillStyle = colorOf.get(p.group) ?? theme.accent2;
    ctx.beginPath();
    ctx.arc(X(p.x), Y(p.y), 2.1, 0, Math.PI * 2);
    ctx.fill();
  }

  // Axis titles.
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 500);
  ctx.textAlign = "center";
  ctx.fillText(spec.xLabel, (fr.padL + w - fr.padR) / 2, h - 4);
  ctx.save();
  ctx.translate(11, (fr.padT + h - fr.padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText(spec.yLabel, 0, 0);
  ctx.restore();
}

/** Legend of the wells contributing points, in the same colour order the canvas uses. */
function buildLegend(points: FitScatterPoint[]): HTMLElement {
  const el = document.createElement("div");
  el.className = "fit-scatter-legend";
  const groups = [...new Set(points.map((p) => p.group))];
  // A legend of ninety wells is noise, not information — the colours still separate them on the
  // canvas, so name what fits and say how many are left.
  const shown = groups.slice(0, 12);
  for (let i = 0; i < shown.length; i++) {
    const item = document.createElement("span");
    item.className = "fit-scatter-legend-item";
    const sw = document.createElement("span");
    sw.className = "fit-scatter-swatch";
    sw.style.background = faciesColor(i);
    item.appendChild(sw);
    item.appendChild(document.createTextNode(shown[i]));
    el.appendChild(item);
  }
  if (groups.length > shown.length) {
    const more = document.createElement("span");
    more.className = "fit-scatter-legend-item";
    more.textContent = `+${groups.length - shown.length} more`;
    el.appendChild(more);
  }
  return el;
}

export interface FitScatter {
  el: HTMLElement;
  /** Draw now. Call it once after inserting `el` — see the note in `buildFitScatter`. */
  redraw: () => void;
  dispose: () => void;
}

/** Builds the QC scatter, its caption, export buttons, hover readout and well legend. */
export function buildFitScatter(spec: FitScatterSpec): FitScatter {
  const wrap = document.createElement("div");
  wrap.className = "mc-hist-wrap";

  const head = document.createElement("div");
  head.className = "mc-plot-head";
  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = spec.caption;
  head.appendChild(cap);

  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist plot-canvas";
  head.appendChild(buildImageExportButtons(() => canvas, spec.exportName, setStatus));
  wrap.appendChild(head);
  wrap.appendChild(canvas);
  wrap.appendChild(buildLegend(spec.points));

  const redraw = () => draw(canvas, spec);
  // The FIRST paint is the caller's, synchronously after inserting `el`, and deliberately NOT
  // deferred to requestAnimationFrame: rAF only fires while the tab is compositing, so in an
  // occluded or background window the plot would stay blank until something resized it — and
  // `attachResizeRedraw` schedules through rAF too, so there is no fallback. Reading clientWidth
  // after insertion forces layout, which is all the first draw needs.
  const stopResize = attachResizeRedraw(canvas, redraw);

  // Hover readout: which well and depth a point came from. On a calibration this is the question
  // that follows "one of these is off the trend" — and the answer is not in the table.
  const stopTip = attachScatterTooltip(canvas, (px, py) => {
    const fr = frameFor(spec, canvas.clientWidth || 380, canvas.clientHeight || 220);
    if (!fr) return null;
    const w = fr.w;
    const h = fr.h;
    const X = (v: number) => fr.padL + ((v - fr.xmin) / (fr.xmax - fr.xmin)) * (w - fr.padL - fr.padR);
    const Y = (v: number) => fr.padT + (1 - (v - fr.ymin) / (fr.ymax - fr.ymin)) * (h - fr.padT - fr.padB);
    let best: FitScatterPoint | null = null;
    let bestD = 81; // 9 px, squared — beyond that the pointer is not on a point
    for (const p of spec.points) {
      if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
      const dx = X(p.x) - px;
      const dy = Y(p.y) - py;
      const d = dx * dx + dy * dy;
      if (d < bestD) {
        bestD = d;
        best = p;
      }
    }
    return best ? best.detail : null;
  });

  return {
    el: wrap,
    redraw,
    dispose: () => {
      stopResize();
      stopTip();
    },
  };
}
