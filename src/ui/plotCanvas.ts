import type { PlotRangePolicyReport } from "./plotRangePolicy";

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
    // Organic 1c: the plot area sits on its own surface tokens (warm neutral
    // ground, white gridlines on light themes); older fallbacks keep a canvas
    // readable if a stripped-down page ever lacks the token.
    bg: v("--plot-bg", "") || v("--bg-app", "#f2ebdc"),
    grid: v("--plot-grid", "") || v("--border", "#ddd0af"),
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
    // Vector export binds a recording 2D context on the canvas (see svgExport.ts); when it is
    // present the same draw code paints into the recorder instead of a raster context. Private
    // contract — kept off the public constructor signature.
    const override = (canvas as unknown as { __recordingCtx2d?: CanvasRenderingContext2D }).__recordingCtx2d;
    const ctx = override ?? canvas.getContext("2d");
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

  /** Scatter with optional colors: per-point array, one uniform color, or the theme
   *  accent when omitted. Clips to the plot area. */
  drawScatter(xs: ArrayLike<number>, ys: ArrayLike<number>, colors?: string[] | string, radius = 1.6): void {
    const { ctx } = this;
    const r = this.plotRect;
    const uniform = typeof colors === "string" ? colors : colors ? null : this.theme.accent;
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
      ctx.fillStyle = uniform ?? (colors as string[])[i];
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

/** Marks a plot canvas up for assistive tech: `role="img"` with a text `label` describing the chart,
 *  and `tabindex=0` so it can take keyboard focus (for {@link attachKeyboardPanZoom}). Re-set the
 *  `aria-label` when the plotted curves change so the description stays accurate. */
export function makeCanvasAccessible(surface: HTMLElement, label: string): void {
  surface.setAttribute("role", "img");
  surface.setAttribute("aria-label", label);
  if (!surface.hasAttribute("tabindex")) surface.tabIndex = 0;
}

export type PlotViewKeyboardCommand =
  | { kind: "pan"; axis: "x" | "y"; direction: -1 | 1; large: boolean }
  | { kind: "zoom"; direction: "in" | "out" }
  | { kind: "reset" };

export interface PlotAccessibilityBinding {
  /** Re-reads the current chart identity after a curve, zone or chart-type change. */
  refresh(): void;
  /** Removes the keyboard handler; required when the panel or generated canvas is replaced. */
  dispose(): void;
}

/**
 * One non-pointer contract shared by every interactive plot surface. Arrow/+/-/Home change the
 * view, P reaches Properties and E reaches export. The callbacks keep canvas-specific rendering
 * outside this accessibility shell while one handler owns focus, labels, shortcuts and teardown.
 */
export function attachAccessiblePlotKeyboard(opts: {
  surface: HTMLElement;
  getLabel: () => string;
  changeView: (command: PlotViewKeyboardCommand) => boolean;
  openProperties: () => void;
  focusExport: () => void;
}): PlotAccessibilityBinding {
  const { surface } = opts;
  const refresh = (): void => {
    makeCanvasAccessible(surface, opts.getLabel());
    surface.setAttribute(
      "aria-keyshortcuts",
      "ArrowLeft ArrowRight ArrowUp ArrowDown + - Home P E",
    );
    surface.setAttribute(
      "aria-description",
      "Arrow keys pan, plus and minus zoom, Home resets, P opens Properties, and E moves to export controls.",
    );
  };
  const onKey = (event: KeyboardEvent): void => {
    const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
    if (key === "p") {
      opts.openProperties();
      event.preventDefault();
      return;
    }
    if (key === "e") {
      opts.focusExport();
      event.preventDefault();
      return;
    }
    let command: PlotViewKeyboardCommand | null = null;
    switch (event.key) {
      case "ArrowLeft":
        command = { kind: "pan", axis: "x", direction: -1, large: event.shiftKey };
        break;
      case "ArrowRight":
        command = { kind: "pan", axis: "x", direction: 1, large: event.shiftKey };
        break;
      case "ArrowUp":
        command = { kind: "pan", axis: "y", direction: 1, large: event.shiftKey };
        break;
      case "ArrowDown":
        command = { kind: "pan", axis: "y", direction: -1, large: event.shiftKey };
        break;
      case "+":
      case "=":
        command = { kind: "zoom", direction: "in" };
        break;
      case "-":
      case "_":
        command = { kind: "zoom", direction: "out" };
        break;
      case "0":
      case "Home":
        command = { kind: "reset" };
        break;
      default:
        break;
    }
    if (!command || !opts.changeView(command)) return;
    refresh();
    event.preventDefault();
  };
  refresh();
  surface.addEventListener("keydown", onKey);
  return {
    refresh,
    dispose: () => surface.removeEventListener("keydown", onKey),
  };
}

/** Keyboard pan/zoom for a focused plot canvas, mirroring {@link attachZoomPan}: arrow keys pan
 *  (Shift = larger step), +/- zoom around centre, 0 or Home resets. Drives the same
 *  {@link ViewportRef} in each axis's transformed space (log-safe). `axes:"x"` locks Y (histograms).
 *  Only acts on keys it handles, so Tab/Enter/etc. still work. Returns a disposer. */
export function attachKeyboardPanZoom(opts: {
  canvas: HTMLCanvasElement;
  getPlot: () => PlotCanvas | null;
  view: ViewportRef;
  redraw: () => void;
  axes?: "both" | "x";
  getLabel: () => string;
  openProperties: () => void;
  focusExport: () => void;
}): PlotAccessibilityBinding {
  const { canvas, getPlot, view, redraw } = opts;
  const axes = opts.axes ?? "both";
  const seed = (plot: PlotCanvas): Viewport => ({
    xMin: Math.min(plot.x.min, plot.x.max),
    xMax: Math.max(plot.x.min, plot.x.max),
    yMin: Math.min(plot.y.min, plot.y.max),
    yMax: Math.max(plot.y.min, plot.y.max),
  });
  const pan = (lo: number, hi: number, log: boolean, dir: number, step: number): [number, number] => {
    const a = tf(log, lo);
    const b = tf(log, hi);
    const d = (b - a) * step * dir;
    return [itf(log, a + d), itf(log, b + d)];
  };
  const zoom = (lo: number, hi: number, log: boolean, factor: number): [number, number] => {
    const a = tf(log, lo);
    const b = tf(log, hi);
    const c = (a + b) / 2;
    const half = ((b - a) / 2) * factor;
    return [itf(log, c - half), itf(log, c + half)];
  };
  return attachAccessiblePlotKeyboard({
    surface: canvas,
    getLabel: opts.getLabel,
    openProperties: opts.openProperties,
    focusExport: opts.focusExport,
    changeView: (command) => {
    const plot = getPlot();
    if (!plot) return false;
    if (command.kind === "reset") {
      if (view.current) {
        view.current = null;
        redraw();
        return true;
      }
      return false;
    }
    const v = view.current ?? seed(plot);
    if (command.kind === "pan") {
      const step = command.large ? 0.2 : 0.08;
      if (command.axis === "x") {
        [v.xMin, v.xMax] = pan(v.xMin, v.xMax, plot.x.log, command.direction, step);
      } else if (axes === "both") {
        [v.yMin, v.yMax] = pan(v.yMin, v.yMax, plot.y.log, command.direction, step);
      } else {
        return false;
      }
    } else {
      const factor = command.direction === "in" ? 0.83 : 1.2;
      [v.xMin, v.xMax] = zoom(v.xMin, v.xMax, plot.x.log, factor);
      if (axes === "both") [v.yMin, v.yMax] = zoom(v.yMin, v.yMax, plot.y.log, factor);
    }
    view.current = { ...v };
    redraw();
    return true;
    },
  });
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

/** Compact numeric label for tooltips/legends: 4 significant figures, trailing zeros trimmed,
 *  non-finite → "—". Keeps "0.182", "12.4", "1.05e+4" readable without axis-tick machinery. */
export function fmtValue(v: number): string {
  if (!Number.isFinite(v)) return "—";
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (a >= 1e5 || a < 1e-3) return v.toExponential(2);
  const s = v.toPrecision(4);
  return s.includes(".") ? s.replace(/\.?0+$/, "") : s;
}

/** Continuous colour-bar legend for a Z-coloured scatter: a horizontal ramp in `map` from
 *  `lo`→`hi` with min / label / max captions, drawn top-right inside the plot rect. Extracted so
 *  every ramp chart (crossplot, Pickett, HFU, …) shows the same bar instead of a bespoke copy. */
export function drawColorbar(
  plot: PlotCanvas,
  opts: { map: ColormapName; lo: number; hi: number; label: string; log?: boolean },
): void {
  const { ctx } = plot;
  const r = plot.plotRect;
  const barW = 90;
  const barX = r.x0 + r.w - barW - 8;
  const barY = r.y0 + 8;
  for (let i = 0; i < barW; i++) {
    ctx.fillStyle = colormapColor(opts.map, i / (barW - 1));
    ctx.fillRect(barX + i, barY, 1, 8);
  }
  ctx.save();
  ctx.fillStyle = plot.theme.text;
  ctx.font = canvasFont(plot.theme, 9);
  ctx.textAlign = "center";
  ctx.fillText(opts.lo.toPrecision(3), barX, barY + 18);
  ctx.fillText(opts.log ? `${opts.label} (log)` : opts.label, barX + barW / 2, barY + 18);
  ctx.fillText(opts.hi.toPrecision(3), barX + barW, barY + 18);
  ctx.restore();
}

/** Wires a hover tooltip bubble onto a plot canvas. On every mouse move, `hit(px, py)` (canvas
 *  CSS-pixel coords) returns the lines to show for the sample under the cursor, or null to hide.
 *  The bubble is a `pointer-events:none` DOM node appended to `<body>` and positioned near the
 *  cursor (clamped to the viewport), so it never steals the canvas's own mouse events. Returns a
 *  disposer that removes the listeners and the node. Colours come from the `.plot-tooltip` CSS. */
export function attachScatterTooltip(
  canvas: HTMLCanvasElement,
  hit: (px: number, py: number) => string[] | null,
): () => void {
  const tip = document.createElement("div");
  tip.className = "plot-tooltip";
  tip.style.position = "fixed";
  tip.style.pointerEvents = "none";
  tip.style.zIndex = "9000";
  tip.style.display = "none";
  document.body.appendChild(tip);

  const onMove = (e: MouseEvent) => {
    const rect = canvas.getBoundingClientRect();
    const lines = hit(e.clientX - rect.left, e.clientY - rect.top);
    if (!lines || !lines.length) {
      tip.style.display = "none";
      return;
    }
    tip.textContent = "";
    for (const ln of lines) {
      const row = document.createElement("div");
      row.textContent = ln;
      tip.appendChild(row);
    }
    tip.style.display = "block";
    const pad = 14;
    let left = e.clientX + pad;
    let top = e.clientY + pad;
    if (left + tip.offsetWidth > window.innerWidth - 4) left = e.clientX - pad - tip.offsetWidth;
    if (top + tip.offsetHeight > window.innerHeight - 4) top = e.clientY - pad - tip.offsetHeight;
    tip.style.left = `${Math.max(4, left)}px`;
    tip.style.top = `${Math.max(4, top)}px`;
  };
  const onLeave = () => {
    tip.style.display = "none";
  };
  canvas.addEventListener("mousemove", onMove);
  canvas.addEventListener("mouseleave", onLeave);
  return () => {
    canvas.removeEventListener("mousemove", onMove);
    canvas.removeEventListener("mouseleave", onLeave);
    tip.remove();
  };
}

/** Blue→green→yellow→red color ramp for Z-colored crossplots; NaN → dim gray. */
export function colorRamp(values: ArrayLike<number>, min: number, max: number): string[] {
  const out: string[] = new Array(values.length);
  const nan = dimRgba(0.35);
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (!Number.isFinite(v)) {
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
    if (!Number.isFinite(v) || (log && v <= 0)) {
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

/** Neutral grey for a REJECTED sample (SB-MLA-021), deliberately outside the qualitative palette
 *  so it cannot be read as one of the clusters.
 *  Keep in sync with FACIES_REJECT_COLOR in src-tauri/src/composite.rs. */
export const REJECT_COLOR = "#9aa0a6";

/** Color for a single facies/cluster index (rounded, wraps). A NEGATIVE class is one the algorithm
 *  REJECTED — DBSCAN noise — not one of the clusters, and the wrap below would otherwise fold it
 *  back onto a real cluster's colour and draw an outlier as a legitimate facies. Any negative, not
 *  just CLUSTER_REJECT: a code this renderer does not recognise must not be painted as rock it
 *  is not. */
export function faciesColor(index: number): string {
  const i = Math.round(index);
  if (i < 0) return REJECT_COLOR;
  const n = FACIES_PALETTE.length;
  return FACIES_PALETTE[((i % n) + n) % n];
}

/** Legend label for a facies/cluster class. A negative is the reject code (SB-MLA-021) and is
 *  named rather than numbered: "F-1" reads as a facies with a strange id, which is exactly the
 *  reading this class exists to prevent. Beside `faciesColor` so a legend cannot draw the grey
 *  swatch and label it as rock. */
export function faciesLabel(index: number): string {
  const i = Math.round(index);
  return i < 0 ? "Rejected" : `F${i}`;
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

/** Heuristic: does this curve look like discrete class labels (small integers, 2–16 distinct)?
 *  Used to auto-switch crossplot coloring to categorical.
 *
 *  −1 is admitted because it is the reject code (SB-MLA-021): a DBSCAN facies curve carrying even
 *  one rejected sample is still a class curve, and excluding it here would silently drop the whole
 *  curve back to a continuous colour ramp — the one presentation that makes class codes meaningless.
 *  Only −1, not any negative: the floor is what keeps this a heuristic about class labels rather
 *  than one that accepts a signed measurement. */
export function looksDiscrete(values: ArrayLike<number>): boolean {
  const seen = new Set<number>();
  let any = false;
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v)) continue;
    any = true;
    if (!Number.isInteger(v) || v < -1 || v > 50) return false;
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
  p25: number;
  p50: number;
  p75: number;
  p95: number;
}

export type StatisticsPopulation = "active_well" | "pooled";
export type StatisticsSelectionKind = "all_eligible" | "named";
export type StandardDeviationChoice = "sample_n_minus_one" | "population_n";

export interface PlotStatisticsInterval {
  low: number | null;
  high: number | null;
  closure: "[lo,hi)" | "[lo,+inf)" | "(-inf,hi)" | "all";
}

export interface PlotStatisticsSelection {
  kind: StatisticsSelectionKind;
  selection_id: string | null;
  label: string;
  applied: boolean;
}

export interface PlotStatisticsExclusions {
  input_count: number;
  non_finite: number;
  log_domain: number;
  validity: number;
  selection: number;
  unpaired_or_unclassified: number;
  display_hidden: number;
}

/** Complete statistics custody used by the screen and every plot-export route. */
export interface PlotStatisticsRecord {
  schema_version: 1;
  binding_channel: string;
  channel: string;
  population: StatisticsPopulation;
  well_ids: string[];
  interval: PlotStatisticsInterval;
  selection: PlotStatisticsSelection;
  finite_pair_count: number;
  exclusions: PlotStatisticsExclusions;
  percentile_interpolation: "linear_index_n_minus_one";
  standard_deviation: StandardDeviationChoice;
  values: BasicStats;
}

export interface PlotStatisticsContext {
  binding_channel: string;
  channel: string;
  population: StatisticsPopulation;
  well_ids: string[];
  interval: PlotStatisticsInterval;
  selection: PlotStatisticsSelection;
  policy: PlotRangePolicyReport;
  selection_excluded: number;
  unpaired_or_unclassified_excluded: number;
  standard_deviation: StandardDeviationChoice;
}

/** NaN-skipping summary statistics with an explicit standard-deviation estimator. */
export function basicStats(
  values: ArrayLike<number>,
  standardDeviation: StandardDeviationChoice = "sample_n_minus_one",
): BasicStats {
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
  const divisor = standardDeviation === "sample_n_minus_one" ? n - 1 : n;
  const std = divisor > 0 ? Math.sqrt(Math.max(0, (sumSq - n * mean * mean) / divisor)) : NaN;
  return {
    count: n,
    mean,
    std,
    min: n > 0 ? min : NaN,
    max: n > 0 ? max : NaN,
    p5: percentile(values, 5),
    p25: percentile(values, 25),
    p50: percentile(values, 50),
    p75: percentile(values, 75),
    p95: percentile(values, 95),
  };
}

/** Build one internally consistent record from the already-screened analysis population. */
export function buildPlotStatisticsRecord(
  eligibleValues: ArrayLike<number>,
  context: PlotStatisticsContext,
): PlotStatisticsRecord {
  if (context.well_ids.length === 0 || context.well_ids.some((wellId) => wellId.trim() === "")) {
    throw new Error("plot statistics require every represented well identity");
  }
  if (new Set(context.well_ids).size !== context.well_ids.length) {
    throw new Error("plot statistics cannot repeat a represented well identity");
  }
  if (context.binding_channel.trim() === "") throw new Error("plot statistics require a binding channel");
  if (context.channel.trim() === "") throw new Error("plot statistics require a channel identity");
  if (context.population === "active_well" && context.well_ids.length !== 1) {
    throw new Error("active-well statistics require exactly one represented well");
  }
  if (context.population === "pooled" && context.well_ids.length < 2) {
    throw new Error("pooled statistics require at least two represented wells");
  }
  const { low, high, closure } = context.interval;
  if (closure === "[lo,hi)") {
    if (low === null || high === null || !Number.isFinite(low) || !Number.isFinite(high) || low >= high) {
      throw new Error("plot statistics interval requires increasing finite limits");
    }
  } else if (closure === "[lo,+inf)") {
    if (low === null || !Number.isFinite(low) || high !== null) {
      throw new Error("lower-bounded plot statistics require one finite low limit");
    }
  } else if (closure === "(-inf,hi)") {
    if (low !== null || high === null || !Number.isFinite(high)) {
      throw new Error("upper-bounded plot statistics require one finite high limit");
    }
  } else if (low !== null || high !== null) {
    throw new Error("all-depth statistics cannot carry numeric interval limits");
  }
  if (context.selection.label.trim() === "") throw new Error("plot statistics selection requires a label");
  if (context.selection.kind === "named" && !context.selection.selection_id?.trim()) {
    throw new Error("named plot statistics selection requires an identity");
  }
  if (context.selection.kind === "all_eligible" && context.selection.selection_id !== null) {
    throw new Error("all-eligible plot statistics selection cannot carry an identity");
  }
  if (context.selection.kind === "named" && !context.selection.applied) {
    throw new Error("a named statistics selection must be applied to its population");
  }
  if (!Number.isInteger(context.selection_excluded) || context.selection_excluded < 0) {
    throw new Error("plot statistics selection exclusion count must be a non-negative integer");
  }
  if (!Number.isInteger(context.unpaired_or_unclassified_excluded)
    || context.unpaired_or_unclassified_excluded < 0) {
    throw new Error("plot statistics unpaired/unclassified exclusion count must be a non-negative integer");
  }
  const values = basicStats(eligibleValues, context.standard_deviation);
  if (values.count + context.selection_excluded !== context.policy.analysisCount
    || values.count !== eligibleValues.length) {
    throw new Error("plot statistics values do not match the governed analysis population");
  }
  return {
    schema_version: 1,
    binding_channel: context.binding_channel,
    channel: context.channel,
    population: context.population,
    well_ids: [...context.well_ids],
    interval: { ...context.interval },
    selection: { ...context.selection },
    finite_pair_count: values.count,
    exclusions: {
      input_count: context.policy.inputCount + context.unpaired_or_unclassified_excluded,
      non_finite: context.policy.nonFiniteExcluded,
      log_domain: context.policy.logDomainExcluded,
      validity: context.policy.validityExcluded,
      selection: context.selection_excluded,
      unpaired_or_unclassified: context.unpaired_or_unclassified_excluded,
      display_hidden: context.policy.displayHidden,
    },
    percentile_interpolation: "linear_index_n_minus_one",
    standard_deviation: context.standard_deviation,
    values,
  };
}

export function plotStatisticsInterval(low: number | null, high: number | null): PlotStatisticsInterval {
  if (low !== null && !Number.isFinite(low)) throw new Error("plot statistics low interval limit must be finite");
  if (high !== null && !Number.isFinite(high)) throw new Error("plot statistics high interval limit must be finite");
  if (low !== null && high !== null) {
    if (low >= high) throw new Error("plot statistics interval limits must increase");
    return { low, high, closure: "[lo,hi)" };
  }
  if (low !== null) return { low, high: null, closure: "[lo,+inf)" };
  if (high !== null) return { low: null, high, closure: "(-inf,hi)" };
  return { low: null, high: null, closure: "all" };
}

/** Compact human-readable form for the live panel; exports retain the full structured record. */
export function formatPlotStatisticsRecord(record: PlotStatisticsRecord): string {
  const population = record.population === "active_well" ? "active well" : "pooled";
  const interval = record.interval.closure === "[lo,hi)"
    ? `[${record.interval.low},${record.interval.high})`
    : record.interval.closure === "[lo,+inf)"
      ? `[${record.interval.low},+inf)`
      : record.interval.closure === "(-inf,hi)"
        ? `(-inf,${record.interval.high})`
        : "all";
  const std = record.standard_deviation === "sample_n_minus_one" ? "sample (n-1)" : "population (n)";
  const excluded = record.exclusions;
  return `statistics[${record.channel}]: ${population} · interval=${interval} · selection=${record.selection.label}`
    + ` (applied=${record.selection.applied})`
    + ` · finite pairs=${record.finite_pair_count}`
    + ` · exclusions input=${excluded.input_count}, non-finite=${excluded.non_finite}, log-domain=${excluded.log_domain},`
    + ` validity=${excluded.validity}, selection=${excluded.selection},`
    + ` unpaired/unclassified=${excluded.unpaired_or_unclassified}, display-hidden=${excluded.display_hidden}`
    + ` · percentile=linear index (n-1) · std=${std}`;
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
