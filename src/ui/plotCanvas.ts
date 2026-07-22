/** Shared Canvas-2D plot scaffolding for the parameter-selection tools (histogram,
 *  crossplot, Pickett). Handles margins, linear/log axes, inverted axes, ticks/grid,
 *  and data↔pixel transforms. Colors come from the CSS theme variables so plots follow
 *  light/dark mode. */

export interface AxisSpec {
  label: string;
  min: number;
  max: number;
  log: boolean;
  invert: boolean;
}

export interface PlotTheme {
  bg: string;
  grid: string;
  axis: string;
  text: string;
  accent: string;
  accent2: string;
  warn: string;
  /** Canvas font family from --font-canvas — use via [`canvasFont`], never a literal. */
  fontFamily: string;
}

/** The one canvas typography token: weight/size steps over the theme's --font-canvas
 *  family. Every ctx.font in the app goes through this so plots, dialogs, and overlays
 *  share a face (and a branded skin can restyle text by overriding one variable). */
export function canvasFont(theme: PlotTheme, sizePx: number, weight = 500): string {
  return `${weight} ${sizePx}px ${theme.fontFamily}`;
}

export function readTheme(el: HTMLElement): PlotTheme {
  // Inactive dockview tabs are detached from the DOM; computed styles there come back
  // empty and every color would fall through to the light-theme literals. Read from the
  // root element instead so a redraw while hidden still picks up the active palette.
  let s = getComputedStyle(el);
  if (!s.getPropertyValue("--bg-app").trim()) s = getComputedStyle(document.documentElement);
  const v = (name: string, fallback: string) => s.getPropertyValue(name).trim() || fallback;
  return {
    bg: v("--bg-app", "#f2ebdc"),
    grid: v("--border", "#ddd0af"),
    axis: v("--border-strong", "#c2ac81"),
    text: v("--text-dim", "#7c6b52"),
    accent: v("--accent", "#b5651d"),
    accent2: v("--accent2", "#5f7350"),
    warn: v("--warn", "#a83e2c"),
    fontFamily: v("--font-canvas", "system-ui, sans-serif"),
  };
}

/** The theme's "no data" marker: --text-dim at the given alpha, so NaN points dim into the
 *  palette instead of a fixed mid-gray that vanishes on dark/brand themes. Read ONCE per
 *  ramp call (not per point) — pass the returned string into the loop. */
function dimRgba(alpha: number): string {
  const raw = getComputedStyle(document.documentElement).getPropertyValue("--text-dim").trim() || "#7c6b52";
  const hex = /^#([0-9a-fA-F]{6})$/.exec(raw)?.[1] ?? (/^#([0-9a-fA-F]{3})$/.exec(raw)?.[1] ?? "")
    .split("")
    .map((c) => c + c)
    .join("");
  if (hex.length !== 6) return `rgba(128,128,128,${alpha})`;
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

const MARGIN = { left: 52, right: 14, top: 10, bottom: 34 };

export type PlotMargin = typeof MARGIN;

/** Sizes a canvas's backing store to its on-screen (CSS) size × devicePixelRatio so it
 *  renders crisply at the panel's real dimensions instead of being a fixed bitmap that CSS
 *  up-scales (the "looks like a screenshot" problem). Returns the dpr actually used. An
 *  off-screen canvas (clientWidth 0 — e.g. unit tests) is left at its attribute size with
 *  dpr 1. Call this immediately before constructing a PlotCanvas for the redraw. */
export function fitCanvasBackingStore(canvas: HTMLCanvasElement): number {
  const cssW = canvas.clientWidth;
  const cssH = canvas.clientHeight;
  if (cssW <= 0 || cssH <= 0) return 1;
  const dpr = Math.min(window.devicePixelRatio || 1, 2.5);
  const w = Math.round(cssW * dpr);
  const h = Math.round(cssH * dpr);
  if (canvas.width !== w) canvas.width = w;
  if (canvas.height !== h) canvas.height = h;
  return dpr;
}

export class PlotCanvas {
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  x: AxisSpec;
  y: AxisSpec;
  theme: PlotTheme;
  /** Logical (CSS-pixel) drawing size; all draw coordinates are in these units. */
  readonly width: number;
  readonly height: number;
  readonly dpr: number;
  /** Effective margins; callers can widen them (e.g. marginal-histogram strips). */
  readonly margin: PlotMargin;

  constructor(canvas: HTMLCanvasElement, x: AxisSpec, y: AxisSpec, margin?: Partial<PlotMargin>) {
    this.canvas = canvas;
    this.margin = { ...MARGIN, ...margin };
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("2D canvas unavailable");
    this.ctx = ctx;
    this.x = x;
    this.y = y;
    this.theme = readTheme(canvas);
    // Logical size = CSS size when on-screen, else the attribute size (off-screen/tests).
    // The backing store is assumed already sized (fitCanvasBackingStore); scale the context
    // so every draw call works in logical pixels regardless of dpr.
    const cssW = canvas.clientWidth;
    this.dpr = cssW > 0 ? canvas.width / cssW : 1;
    this.width = cssW > 0 ? cssW : canvas.width;
    this.height = canvas.clientHeight > 0 ? canvas.clientHeight : canvas.height;
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
  }

  get plotRect(): { x0: number; y0: number; w: number; h: number } {
    return {
      x0: this.margin.left,
      y0: this.margin.top,
      w: this.width - this.margin.left - this.margin.right,
      h: this.height - this.margin.top - this.margin.bottom,
    };
  }

  private frac(spec: AxisSpec, v: number): number {
    let f: number;
    if (spec.log) {
      const lmin = Math.log10(spec.min);
      const lmax = Math.log10(spec.max);
      f = (Math.log10(v) - lmin) / (lmax - lmin);
    } else {
      f = (v - spec.min) / (spec.max - spec.min);
    }
    return spec.invert ? 1 - f : f;
  }

  private unfrac(spec: AxisSpec, f: number): number {
    if (spec.invert) f = 1 - f;
    if (spec.log) {
      const lmin = Math.log10(spec.min);
      const lmax = Math.log10(spec.max);
      return Math.pow(10, lmin + f * (lmax - lmin));
    }
    return spec.min + f * (spec.max - spec.min);
  }

  toPx(vx: number, vy: number): [number, number] {
    const r = this.plotRect;
    return [r.x0 + this.frac(this.x, vx) * r.w, r.y0 + (1 - this.frac(this.y, vy)) * r.h];
  }

  toData(px: number, py: number): [number, number] {
    const r = this.plotRect;
    return [this.unfrac(this.x, (px - r.x0) / r.w), this.unfrac(this.y, 1 - (py - r.y0) / r.h)];
  }

  inPlot(px: number, py: number): boolean {
    const r = this.plotRect;
    return px >= r.x0 && px <= r.x0 + r.w && py >= r.y0 && py <= r.y0 + r.h;
  }

  private ticks(spec: AxisSpec): number[] {
    if (spec.log) {
      const out: number[] = [];
      const lo = Math.floor(Math.log10(spec.min));
      const hi = Math.ceil(Math.log10(spec.max));
      for (let e = lo; e <= hi; e++) {
        const v = Math.pow(10, e);
        if (v >= spec.min && v <= spec.max) out.push(v);
      }
      return out;
    }
    const span = spec.max - spec.min;
    const rawStep = span / 6;
    const mag = Math.pow(10, Math.floor(Math.log10(Math.abs(rawStep))));
    const norm = rawStep / mag;
    const step = (norm < 1.5 ? 1 : norm < 3.5 ? 2 : norm < 7.5 ? 5 : 10) * mag;
    const out: number[] = [];
    for (let v = Math.ceil(spec.min / step) * step; v <= spec.max + step / 1e6; v += step) {
      out.push(Math.abs(v) < step / 1e6 ? 0 : v);
    }
    return out;
  }

  private fmtTick(v: number): string {
    if (v === 0) return "0";
    const a = Math.abs(v);
    if (a >= 10000 || a < 0.01) return v.toExponential(0);
    return String(Math.round(v * 1000) / 1000);
  }

  /** Clears everything and draws background, grid, axes, tick labels, and axis labels. */
  drawFrame(): void {
    this.theme = readTheme(this.canvas);
    const { ctx } = this;
    const r = this.plotRect;
    ctx.clearRect(0, 0, this.width, this.height);
    ctx.fillStyle = this.theme.bg;
    ctx.fillRect(r.x0, r.y0, r.w, r.h);

    ctx.font = canvasFont(this.theme, 10);
    ctx.strokeStyle = this.theme.grid;
    ctx.fillStyle = this.theme.text;
    ctx.lineWidth = 1;

    for (const v of this.ticks(this.x)) {
      const [px] = this.toPx(v, this.y.min);
      ctx.beginPath();
      ctx.moveTo(px, r.y0);
      ctx.lineTo(px, r.y0 + r.h);
      ctx.stroke();
      ctx.textAlign = "center";
      ctx.fillText(this.fmtTick(v), px, r.y0 + r.h + 14);
    }
    for (const v of this.ticks(this.y)) {
      const [, py] = this.toPx(this.x.min, v);
      ctx.beginPath();
      ctx.moveTo(r.x0, py);
      ctx.lineTo(r.x0 + r.w, py);
      ctx.stroke();
      ctx.textAlign = "right";
      ctx.fillText(this.fmtTick(v), r.x0 - 6, py + 3);
    }

    // Minor log decade gridlines (2–9).
    for (const spec of [this.x, this.y]) {
      if (!spec.log) continue;
      ctx.save();
      ctx.strokeStyle = this.theme.grid;
      ctx.globalAlpha = 0.4;
      const lo = Math.floor(Math.log10(spec.min));
      const hi = Math.ceil(Math.log10(spec.max));
      for (let e = lo; e < hi; e++) {
        for (let mult = 2; mult <= 9; mult++) {
          const v = mult * Math.pow(10, e);
          if (v <= spec.min || v >= spec.max) continue;
          ctx.beginPath();
          if (spec === this.x) {
            const [px] = this.toPx(v, this.y.min);
            ctx.moveTo(px, r.y0);
            ctx.lineTo(px, r.y0 + r.h);
          } else {
            const [, py] = this.toPx(this.x.min, v);
            ctx.moveTo(r.x0, py);
            ctx.lineTo(r.x0 + r.w, py);
          }
          ctx.stroke();
        }
      }
      ctx.restore();
    }

    ctx.strokeStyle = this.theme.axis;
    ctx.strokeRect(r.x0, r.y0, r.w, r.h);

    ctx.fillStyle = this.theme.text;
    ctx.textAlign = "center";
    ctx.fillText(this.x.label, r.x0 + r.w / 2, this.height - 6);
    ctx.save();
    ctx.translate(12, r.y0 + r.h / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText(this.y.label, 0, 0);
    ctx.restore();
  }

  /** Scatter with optional per-point colors (else theme accent). Clips to the plot area. */
  drawScatter(xs: ArrayLike<number>, ys: ArrayLike<number>, colors?: string[], radius = 1.6): void {
    const { ctx } = this;
    const r = this.plotRect;
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (this.x.log && vx <= 0) continue;
      if (this.y.log && vy <= 0) continue;
      const [px, py] = this.toPx(vx, vy);
      ctx.fillStyle = colors ? colors[i] : this.theme.accent;
      ctx.beginPath();
      ctx.arc(px, py, radius, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  /** Polyline through data points (clipped). */
  drawLine(points: [number, number][], color: string, width = 1.5, dash: number[] = []): void {
    if (points.length < 2) return;
    const { ctx } = this;
    const r = this.plotRect;
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.setLineDash(dash);
    ctx.beginPath();
    points.forEach(([vx, vy], i) => {
      const [px, py] = this.toPx(vx, vy);
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    });
    ctx.stroke();
    ctx.restore();
  }

  /** Vertical marker line at data-x with a small label. */
  drawVMarker(vx: number, color: string, label?: string): void {
    const r = this.plotRect;
    const [px] = this.toPx(vx, this.y.min);
    if (px < r.x0 || px > r.x0 + r.w) return;
    const { ctx } = this;
    ctx.save();
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.setLineDash([5, 3]);
    ctx.beginPath();
    ctx.moveTo(px, r.y0);
    ctx.lineTo(px, r.y0 + r.h);
    ctx.stroke();
    if (label) {
      ctx.setLineDash([]);
      ctx.fillStyle = color;
      ctx.font = canvasFont(this.theme, 10);
      ctx.textAlign = "left";
      ctx.fillText(label, px + 4, r.y0 + 12);
    }
    ctx.restore();
  }

  /** Diamond-marker scatter (clipped), visually distinct from the round `drawScatter`
   *  dots — used for overlaying a second, independently-sourced data set (e.g. core
   *  plug measurements over log-derived crossplot points). */
  drawDiamonds(xs: ArrayLike<number>, ys: ArrayLike<number>, color: string, radius = 4): void {
    const { ctx } = this;
    const r = this.plotRect;
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    ctx.fillStyle = color;
    ctx.strokeStyle = this.theme.bg;
    ctx.lineWidth = 1;
    for (let i = 0; i < xs.length; i++) {
      const vx = xs[i];
      const vy = ys[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (this.x.log && vx <= 0) continue;
      if (this.y.log && vy <= 0) continue;
      const [px, py] = this.toPx(vx, vy);
      ctx.beginPath();
      ctx.moveTo(px, py - radius);
      ctx.lineTo(px + radius, py);
      ctx.lineTo(px, py + radius);
      ctx.lineTo(px - radius, py);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }
    ctx.restore();
  }

  /** Labeled reference point (crossplot matrix points). */
  drawRefPoint(vx: number, vy: number, label: string): void {
    const [px, py] = this.toPx(vx, vy);
    const { ctx } = this;
    ctx.save();
    ctx.fillStyle = this.theme.warn;
    ctx.strokeStyle = this.theme.bg;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(px, py, 4, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.font = canvasFont(this.theme, 10, 700);
    ctx.textAlign = "left";
    ctx.fillText(label, px + 6, py - 5);
    ctx.restore();
  }
}

/** A zoom/pan window in data space (min < max even for inverted axes). Null = auto range. */
export interface Viewport {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

/** Mutable holder so a panel and its event handlers share one viewport reference. */
export interface ViewportRef {
  current: Viewport | null;
}

const tf = (log: boolean, v: number): number => (log ? Math.log10(v) : v);
const itf = (log: boolean, t: number): number => (log ? Math.pow(10, t) : t);

/** Wires wheel-zoom (around the cursor), drag-to-pan, and double-click-reset onto a plot
 *  canvas, driving a shared {@link ViewportRef}. Zooming/panning happen in each axis's
 *  transformed space, so log axes (por–perm, Pickett) behave correctly. `axes:"x"` locks
 *  the Y range (histograms). `onPanStart` lets a caller veto a pan (e.g. when a drag began
 *  on a draggable handle). Returns a disposer. */
export function attachZoomPan(opts: {
  canvas: HTMLCanvasElement;
  getPlot: () => PlotCanvas | null;
  view: ViewportRef;
  redraw: () => void;
  axes?: "both" | "x";
  onPanStart?: (px: number, py: number) => boolean;
}): () => void {
  const { canvas, getPlot, view, redraw } = opts;
  const axes = opts.axes ?? "both";

  const pxOf = (e: MouseEvent | WheelEvent): [number, number] => {
    const rect = canvas.getBoundingClientRect();
    return [e.clientX - rect.left, e.clientY - rect.top];
  };
  const seed = (plot: PlotCanvas): Viewport => ({
    xMin: Math.min(plot.x.min, plot.x.max),
    xMax: Math.max(plot.x.min, plot.x.max),
    yMin: Math.min(plot.y.min, plot.y.max),
    yMax: Math.max(plot.y.min, plot.y.max),
  });

  const onWheel = (e: WheelEvent) => {
    // Zoom only on Ctrl+wheel — a plain wheel must keep scrolling the page/panel,
    // otherwise a plot that fills the pane traps the user (Jauhar 2026-07-19).
    if (!e.ctrlKey) return;
    const plot = getPlot();
    if (!plot) return;
    const [px, py] = pxOf(e);
    if (!plot.inPlot(px, py)) return;
    e.preventDefault();
    const v = view.current ?? seed(plot);
    const [dx, dy] = plot.toData(px, py);
    const f = e.deltaY < 0 ? 0.83 : 1.2; // in = shrink window, out = grow
    const zoom = (lo: number, hi: number, cur: number, log: boolean): [number, number] => {
      const tLo = tf(log, lo);
      const tHi = tf(log, hi);
      const tCur = tf(log, log ? Math.max(cur, 1e-30) : cur);
      return [itf(log, tCur - (tCur - tLo) * f), itf(log, tCur + (tHi - tCur) * f)];
    };
    [v.xMin, v.xMax] = zoom(v.xMin, v.xMax, dx, plot.x.log);
    if (axes === "both") [v.yMin, v.yMax] = zoom(v.yMin, v.yMax, dy, plot.y.log);
    view.current = { ...v };
    redraw();
  };

  let panning = false;
  let last: [number, number] | null = null;
  const onDown = (e: MouseEvent) => {
    if (e.button !== 0) return;
    const plot = getPlot();
    if (!plot) return;
    const [px, py] = pxOf(e);
    if (!plot.inPlot(px, py)) return;
    if (opts.onPanStart && !opts.onPanStart(px, py)) return; // handle grabbed the press
    panning = true;
    last = [px, py];
    // Note: the viewport is only seeded on the first actual move, so a pure click (no
    // drag) never freezes auto-ranging — it stays a click for the panel's pick handler.
  };
  const onMove = (e: MouseEvent) => {
    if (!panning || !last) return;
    const plot = getPlot();
    if (!plot) return;
    if (!view.current) view.current = seed(plot);
    canvas.style.cursor = "grabbing";
    const [px, py] = pxOf(e);
    const [d0x, d0y] = plot.toData(last[0], last[1]);
    const [d1x, d1y] = plot.toData(px, py);
    const v = view.current;
    const shift = (lo: number, hi: number, a: number, b: number, log: boolean): [number, number] => {
      const d = tf(log, log ? Math.max(a, 1e-30) : a) - tf(log, log ? Math.max(b, 1e-30) : b);
      return [itf(log, tf(log, lo) + d), itf(log, tf(log, hi) + d)];
    };
    [v.xMin, v.xMax] = shift(v.xMin, v.xMax, d0x, d1x, plot.x.log);
    if (axes === "both") [v.yMin, v.yMax] = shift(v.yMin, v.yMax, d0y, d1y, plot.y.log);
    view.current = { ...v };
    last = [px, py];
    redraw();
  };
  const endPan = () => {
    panning = false;
    last = null;
    canvas.style.cursor = "";
  };
  const onDbl = () => {
    if (!view.current) return;
    view.current = null;
    redraw();
  };

  canvas.addEventListener("wheel", onWheel, { passive: false });
  canvas.addEventListener("mousedown", onDown);
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", endPan);
  canvas.addEventListener("dblclick", onDbl);
  return () => {
    canvas.removeEventListener("wheel", onWheel);
    canvas.removeEventListener("mousedown", onDown);
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", endPan);
    canvas.removeEventListener("dblclick", onDbl);
  };
}

/** ResizeObserver → rAF-debounced redraw, so a plot re-renders at the panel's real size.
 *  Returns a disposer. */
export function attachResizeRedraw(canvas: HTMLCanvasElement, redraw: () => void): () => void {
  let raf = 0;
  let lastW = canvas.clientWidth;
  let lastH = canvas.clientHeight;
  const ro = new ResizeObserver(() => {
    if (canvas.clientWidth === lastW && canvas.clientHeight === lastH) return;
    lastW = canvas.clientWidth;
    lastH = canvas.clientHeight;
    if (raf) return;
    raf = requestAnimationFrame(() => {
      raf = 0;
      redraw();
    });
  });
  ro.observe(canvas);
  return () => {
    ro.disconnect();
    if (raf) cancelAnimationFrame(raf);
  };
}

/** Blue→green→yellow→red color ramp for Z-colored crossplots; NaN → dim gray. */
export function colorRamp(values: ArrayLike<number>, min: number, max: number): string[] {
  const out: string[] = new Array(values.length);
  const nan = dimRgba(0.35);
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v)) {
      out[i] = nan;
      continue;
    }
    const t = Math.max(0, Math.min(1, (v - min) / (max - min)));
    // Piecewise hue: 220° (blue) → 0° (red).
    const hue = 220 * (1 - t);
    out[i] = `hsl(${hue.toFixed(0)}, 75%, 45%)`;
  }
  return out;
}

export type ColormapName = "rainbow" | "viridis";

/** Viridis anchor colors (perceptually uniform — readable where the rainbow's hue
 *  wheel loses order, and safe for log-scaled Z). Linear interpolation between stops. */
const VIRIDIS: [number, number, number][] = [
  [68, 1, 84], [72, 40, 120], [62, 74, 137], [49, 104, 142], [38, 130, 142],
  [31, 158, 137], [53, 183, 121], [109, 205, 89], [180, 222, 44], [253, 231, 37],
];

/** Color at t ∈ [0,1] for a named colormap. */
export function colormapColor(map: ColormapName, t: number): string {
  const tc = Math.max(0, Math.min(1, t));
  if (map === "rainbow") {
    return `hsl(${(220 * (1 - tc)).toFixed(0)}, 75%, 45%)`;
  }
  const f = tc * (VIRIDIS.length - 1);
  const i = Math.min(VIRIDIS.length - 2, Math.floor(f));
  const u = f - i;
  const c = VIRIDIS[i].map((v, k) => Math.round(v + (VIRIDIS[i + 1][k] - v) * u));
  return `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
}

/** Per-point colors for a continuous Z with colormap choice and optional log scaling
 *  (t computed in log10 space — a linear ramp on a log-distributed Z like permeability
 *  crams everything into one end). NaN → dim gray; log-illegal (≤0) too. */
export function colorRampEx(
  values: ArrayLike<number>,
  min: number,
  max: number,
  map: ColormapName,
  log: boolean,
): string[] {
  const lo = log ? Math.log10(min) : min;
  const hi = log ? Math.log10(max) : max;
  const out: string[] = new Array(values.length);
  const nan = dimRgba(0.35);
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v) || (log && v <= 0)) {
      out[i] = nan;
      continue;
    }
    const tv = log ? Math.log10(v) : v;
    out[i] = colormapColor(map, (tv - lo) / (hi - lo));
  }
  return out;
}

/** Fixed qualitative palette for discrete facies / cluster coloring (Tableau-10 + spares,
 *  wraps beyond 12). Distinct hues so adjacent facies are easy to tell apart. */
export const FACIES_PALETTE: string[] = [
  "#4e79a7", "#f28e2b", "#59a14f", "#e15759", "#b07aa1", "#76b7b2",
  "#edc948", "#ff9da7", "#9c755f", "#8cd17d", "#86bcb6", "#d37295",
];

/** Color for a single facies/cluster index (rounded, wraps, never negative). */
export function faciesColor(index: number): string {
  const i = Math.round(index);
  const n = FACIES_PALETTE.length;
  return FACIES_PALETTE[((i % n) + n) % n];
}

/** Per-point categorical colors for a discrete curve (NaN → faint gray). */
export function categoricalColors(values: ArrayLike<number>): string[] {
  const out: string[] = new Array(values.length);
  const nan = dimRgba(0.35);
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    out[i] = Number.isNaN(v) ? nan : faciesColor(v);
  }
  return out;
}

/** Heuristic: does this curve look like discrete class labels (small non-negative
 *  integers, 2–16 distinct)? Used to auto-switch crossplot coloring to categorical. */
export function looksDiscrete(values: ArrayLike<number>): boolean {
  const seen = new Set<number>();
  let any = false;
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v)) continue;
    any = true;
    if (!Number.isInteger(v) || v < 0 || v > 50) return false;
    seen.add(v);
    if (seen.size > 16) return false;
  }
  return any && seen.size >= 2 && seen.size <= 16;
}

/** Sorted distinct non-NaN integer values (facies indices actually present). */
export function distinctValues(values: ArrayLike<number>): number[] {
  const seen = new Set<number>();
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (!Number.isNaN(v)) seen.add(Math.round(v));
  }
  return [...seen].sort((a, b) => a - b);
}

export interface BasicStats {
  count: number;
  mean: number;
  std: number;
  min: number;
  max: number;
  p5: number;
  p50: number;
  p95: number;
}

/** NaN-skipping summary statistics for the histogram chips (sample std dev). */
export function basicStats(values: ArrayLike<number>): BasicStats {
  let n = 0;
  let sum = 0;
  let sumSq = 0;
  let min = Infinity;
  let max = -Infinity;
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    // Reject ±Infinity too (not just NaN): a Python equation like 1/phi at phi=0 emits
    // inf into computed_curves, and one inf makes mean/max read "Infinity" and std NaN.
    if (!Number.isFinite(v)) continue;
    n++;
    sum += v;
    sumSq += v * v;
    if (v < min) min = v;
    if (v > max) max = v;
  }
  const mean = n > 0 ? sum / n : NaN;
  const std = n > 1 ? Math.sqrt(Math.max(0, (sumSq - n * mean * mean) / (n - 1))) : NaN;
  return {
    count: n,
    mean,
    std,
    min: n > 0 ? min : NaN,
    max: n > 0 ? max : NaN,
    p5: percentile(values, 5),
    p50: percentile(values, 50),
    p95: percentile(values, 95),
  };
}

export interface LinearFit {
  a: number;
  b: number;
  r2: number;
  n: number;
}

/** Least-squares fit y' = a + b·x' over valid pairs, where x'/y' are log10-transformed
 *  when the matching axis is log (so the fitted line is straight on the plot). NaN pairs
 *  and log-illegal (≤0) values are skipped. Needs ≥3 points and non-degenerate x. */
export function linearFit(
  xs: ArrayLike<number>,
  ys: ArrayLike<number>,
  xLog: boolean,
  yLog: boolean,
): LinearFit | null {
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
    // Reject ±Infinity as well as NaN — one inf pair corrupts the regression sums
    // (den = n·sxx − sx² becomes NaN, so the fit silently vanishes).
    if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
    if (xLog) {
      if (x <= 0) continue;
      x = Math.log10(x);
    }
    if (yLog) {
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
  const den = n * sxx - sx * sx;
  if (Math.abs(den) < 1e-12) return null;
  const b = (n * sxy - sx * sy) / den;
  const a = (sy - b * sx) / n;
  const r2Den = den * (n * syy - sy * sy);
  const r2 = r2Den > 0 ? Math.pow(n * sxy - sx * sy, 2) / r2Den : 0;
  return { a, b, r2, n };
}

/** Percentile (p in 0–100) over a Float32Array, skipping non-finite (NaN/±Infinity) values. */
export function percentile(values: ArrayLike<number>, p: number): number {
  const clean: number[] = [];
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isFinite(v)) clean.push(v);
  }
  if (clean.length === 0) return NaN;
  clean.sort((a, b) => a - b);
  const idx = ((clean.length - 1) * p) / 100;
  const lo = Math.floor(idx);
  const hi = Math.ceil(idx);
  return clean[lo] + (clean[hi] - clean[lo]) * (idx - lo);
}
