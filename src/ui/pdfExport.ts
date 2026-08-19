import { save } from "@tauri-apps/plugin-dialog";
import { savePlotPdf, type PaperExportRecord, type PlotAncestryScope } from "../ipc";
import { readTheme, type PlotCanvas } from "./plotCanvas";
import {
  buildPaperExportRecord,
  measurePaperText,
  PAPER_PROVENANCE_FONT_PT,
  paperPageHeight,
  paperPageWidth,
} from "./paperExport";
import { measurePlotForPaper } from "./svgExport";

/** True-vector PDF export for the Canvas-2D plots, a sibling of svgExport.ts. It drives the
 *  *same* draw functions through a recording 2D context (the SvgRecorder pattern), but serialises
 *  every call into a PDF *content stream* — PDF operators in user space (points, bottom-left
 *  origin). The frontend owns only the drawing operators; the backend (`composite::assemble_pdf`
 *  via `save_plot_pdf`) wraps them in the PDF document scaffolding (catalog, xref, Helvetica
 *  fonts), so the fiddly, tested document structure lives in one place and is shared with the
 *  composite-log exporter.
 *
 *  Only the subset of the 2D API the plot code uses is implemented (rectangular clips via q/W n,
 *  full-circle arcs as béziers, textAlign/textBaseline, dashes); a missing *method* throws rather
 *  than mis-rendering. Text renders in base-14 Helvetica (no font embedding, same as the composite
 *  PDF), and transparency is flattened against the plot background — exact for the plots, which
 *  only ever use alpha for gridlines/marginals drawn straight over that background. The SVG export
 *  is the fully device-independent option; the PDF is the portable single-file one. */

type Mat = [number, number, number, number, number, number];
const IDENTITY: Mat = [1, 0, 0, 1, 0, 0];

/** Compose canvas transforms: A ∘ B (apply B, then A) — matches ctx.transform semantics. */
function mul(a: Mat, b: Mat): Mat {
  return [
    a[0] * b[0] + a[2] * b[1],
    a[1] * b[0] + a[3] * b[1],
    a[0] * b[2] + a[2] * b[3],
    a[1] * b[2] + a[3] * b[3],
    a[0] * b[4] + a[2] * b[5] + a[4],
    a[1] * b[4] + a[3] * b[5] + a[5],
  ];
}

/** Round to 2 dp; non-finite → 0 so the stream never contains NaN/Infinity tokens. */
function r2(v: number): number {
  return Number.isFinite(v) ? Math.round(v * 100) / 100 : 0;
}
/** Colour component to a 3-dp string. */
function c3(v: number): string {
  return (Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : 0).toFixed(3);
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  h = ((h % 360) + 360) % 360;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let rp = 0;
  let gp = 0;
  let bp = 0;
  if (h < 60) [rp, gp, bp] = [c, x, 0];
  else if (h < 120) [rp, gp, bp] = [x, c, 0];
  else if (h < 180) [rp, gp, bp] = [0, c, x];
  else if (h < 240) [rp, gp, bp] = [0, x, c];
  else if (h < 300) [rp, gp, bp] = [x, 0, c];
  else [rp, gp, bp] = [c, 0, x];
  return [rp + m, gp + m, bp + m];
}

/** Parse the CSS colour strings the plots actually emit (#rgb / #rrggbb, rgb()/rgba(),
 *  hsl()/hsla()) into [r, g, b, a] in 0..1. Unknown input → opaque black. */
function parseColor(css: string): [number, number, number, number] {
  const s = String(css).trim().toLowerCase();
  let m = /^#([0-9a-f]{3})$/.exec(s);
  if (m) {
    const h = m[1];
    return [parseInt(h[0] + h[0], 16) / 255, parseInt(h[1] + h[1], 16) / 255, parseInt(h[2] + h[2], 16) / 255, 1];
  }
  m = /^#([0-9a-f]{6})$/.exec(s);
  if (m) {
    const h = m[1];
    return [parseInt(h.slice(0, 2), 16) / 255, parseInt(h.slice(2, 4), 16) / 255, parseInt(h.slice(4, 6), 16) / 255, 1];
  }
  m = /^rgba?\(([^)]+)\)$/.exec(s);
  if (m) {
    const p = m[1].split(/[,/]/).map((v) => v.trim());
    const comp = (v: string) => (v.endsWith("%") ? parseFloat(v) / 100 : parseFloat(v) / 255);
    const a = p[3] !== undefined ? parseFloat(p[3]) : 1;
    return [comp(p[0] ?? "0"), comp(p[1] ?? "0"), comp(p[2] ?? "0"), Number.isFinite(a) ? a : 1];
  }
  m = /^hsla?\(([^)]+)\)$/.exec(s);
  if (m) {
    const p = m[1].split(/[,/]/).map((v) => v.trim());
    const [rr, gg, bb] = hslToRgb(parseFloat(p[0]), parseFloat(p[1]) / 100, parseFloat(p[2]) / 100);
    const a = p[3] !== undefined ? parseFloat(p[3]) : 1;
    return [rr, gg, bb, Number.isFinite(a) ? a : 1];
  }
  return [0, 0, 0, 1];
}

/** Transliterate the few non-ASCII glyphs the plots use into WinAnsi-safe ASCII (base-14
 *  Helvetica can't show them), then escape PDF string metacharacters. Mirrors the composite
 *  PDF's ASCII-only text discipline. */
const TRANSLIT: Record<string, string> = {
  "²": "2", "³": "3", "·": ".", "–": "-", "—": "-", "µ": "u", "μ": "u", "≥": ">=", "≤": "<=", "×": "x", "°": "deg", "−": "-",
};
function pdfText(s: string): string {
  let out = "";
  for (const ch of String(s)) {
    const t = TRANSLIT[ch] ?? ch;
    for (const c of t) {
      if (c === "(" || c === ")" || c === "\\") out += "\\" + c;
      else if (c.charCodeAt(0) < 128) out += c;
      else out += "-";
    }
  }
  return out;
}

interface Frame {
  m: Mat;
  fillStyle: string;
  strokeStyle: string;
  lineWidth: number;
  font: string;
  textAlign: string;
  textBaseline: string;
  globalAlpha: number;
  dash: number[];
}

/** A minimal CanvasRenderingContext2D-compatible recorder that emits a PDF content stream. */
export class PdfRecorder {
  private body: string[] = [];
  private stack: Frame[] = [];
  private open = 0; // balance of emitted q/Q (each save = q, restore = Q)
  private d = ""; // current path, in PDF space
  private circles: { cx: number; cy: number; r: number }[] = [];
  private pendingRect: { x: number; y: number; w: number; h: number } | null = null;
  private bgRgb: [number, number, number] = [1, 1, 1];

  m: Mat = [...IDENTITY] as Mat;
  fillStyle = "#000000";
  strokeStyle = "#000000";
  lineWidth = 1;
  font = "10px sans-serif";
  textAlign = "start";
  textBaseline = "alphabetic";
  globalAlpha = 1;
  private dash: number[] = [];
  bg = "";

  constructor(
    readonly width: number,
    readonly height: number,
  ) {}

  /** Sets the background colour used both for the page fill and for flattening any alpha < 1. */
  setBackground(css: string): void {
    this.bg = css;
    const [r, g, b] = parseColor(css);
    this.bgRgb = [r, g, b];
  }

  // --- transforms -----------------------------------------------------------
  save(): void {
    this.stack.push({
      m: [...this.m] as Mat,
      fillStyle: this.fillStyle,
      strokeStyle: this.strokeStyle,
      lineWidth: this.lineWidth,
      font: this.font,
      textAlign: this.textAlign,
      textBaseline: this.textBaseline,
      globalAlpha: this.globalAlpha,
      dash: [...this.dash],
    });
    this.body.push("q");
    this.open++;
  }
  restore(): void {
    const f = this.stack.pop();
    if (!f) return;
    this.m = f.m;
    this.fillStyle = f.fillStyle;
    this.strokeStyle = f.strokeStyle;
    this.lineWidth = f.lineWidth;
    this.font = f.font;
    this.textAlign = f.textAlign;
    this.textBaseline = f.textBaseline;
    this.globalAlpha = f.globalAlpha;
    this.dash = f.dash;
    if (this.open > 0) {
      this.body.push("Q");
      this.open--;
    }
  }
  translate(tx: number, ty: number): void {
    this.m = mul(this.m, [1, 0, 0, 1, tx, ty]);
  }
  rotate(rad: number): void {
    const c = Math.cos(rad);
    const s = Math.sin(rad);
    this.m = mul(this.m, [c, s, -s, c, 0, 0]);
  }
  scale(sx: number, sy: number): void {
    this.m = mul(this.m, [sx, 0, 0, sy, 0, 0]);
  }
  setTransform(a: number, b: number, c: number, d: number, e: number, f: number): void {
    this.m = [a, b, c, d, e, f];
  }
  setLineDash(d: number[]): void {
    this.dash = Array.isArray(d) ? d.slice() : [];
  }

  /** Apply the current matrix, then flip device (top-left, y-down) → PDF (bottom-left, y-up). */
  private toPdf(x: number, y: number): [number, number] {
    const m = this.m;
    const dx = m[0] * x + m[2] * y + m[4];
    const dy = m[1] * x + m[3] * y + m[5];
    return [dx, this.height - dy];
  }
  private scaleFactor(): number {
    return Math.hypot(this.m[0], this.m[1]) || 1;
  }
  private fillRgb(): [number, number, number] {
    return this.blend(this.fillStyle);
  }
  private strokeRgb(): [number, number, number] {
    return this.blend(this.strokeStyle);
  }
  /** Effective colour after flattening (colour-alpha × globalAlpha) against the background. */
  private blend(style: string): [number, number, number] {
    const [r, g, b, a] = parseColor(style);
    const eff = a * this.globalAlpha;
    if (eff >= 1) return [r, g, b];
    const [br, bg, bb] = this.bgRgb;
    return [r * eff + br * (1 - eff), g * eff + bg * (1 - eff), b * eff + bb * (1 - eff)];
  }

  // --- path ---------------------------------------------------------------
  beginPath(): void {
    this.d = "";
    this.circles = [];
    this.pendingRect = null;
  }
  moveTo(x: number, y: number): void {
    const [px, py] = this.toPdf(x, y);
    this.d += `${r2(px)} ${r2(py)} m\n`;
  }
  lineTo(x: number, y: number): void {
    const [px, py] = this.toPdf(x, y);
    this.d += `${r2(px)} ${r2(py)} l\n`;
  }
  closePath(): void {
    this.d += "h\n";
  }
  arc(cx: number, cy: number, rad: number, a0: number, a1: number): void {
    const rr = rad * this.scaleFactor();
    if (Math.abs(a1 - a0) >= Math.PI * 2 - 1e-6) {
      const [px, py] = this.toPdf(cx, cy);
      this.circles.push({ cx: px, cy: py, r: rr });
      return;
    }
    // Partial arc (rare): sample into line segments so the path stays faithful.
    const steps = Math.max(2, Math.ceil((Math.abs(a1 - a0) / (Math.PI * 2)) * 32));
    for (let i = 0; i <= steps; i++) {
      const a = a0 + ((a1 - a0) * i) / steps;
      const [px, py] = this.toPdf(cx + Math.cos(a) * rad, cy + Math.sin(a) * rad);
      this.d += `${r2(px)} ${r2(py)} ${i === 0 ? "m" : "l"}\n`;
    }
  }
  rect(x: number, y: number, w: number, h: number): void {
    // Used only to build a rectangular clip region (drawScatter/drawLine/…).
    const [x0, y0] = this.toPdf(x, y);
    const [x1, y1] = this.toPdf(x + w, y + h);
    this.pendingRect = { x: Math.min(x0, x1), y: Math.min(y0, y1), w: Math.abs(x1 - x0), h: Math.abs(y1 - y0) };
  }
  clip(): void {
    if (!this.pendingRect) return;
    const { x, y, w, h } = this.pendingRect;
    // Intersect the current clip with this rect (nonzero winding), then end the path unpainted.
    // Scoped by the enclosing q/Q the plot always wraps a clip in.
    this.body.push(`${r2(x)} ${r2(y)} ${r2(w)} ${r2(h)} re W n`);
    this.pendingRect = null;
  }

  /** Emits the béziers for a filled/stroked circle path (4-arc approximation). */
  private circlePath(cx: number, cy: number, rad: number): string {
    const k = 0.5522847498307936 * rad;
    let s = `${r2(cx + rad)} ${r2(cy)} m\n`;
    s += `${r2(cx + rad)} ${r2(cy + k)} ${r2(cx + k)} ${r2(cy + rad)} ${r2(cx)} ${r2(cy + rad)} c\n`;
    s += `${r2(cx - k)} ${r2(cy + rad)} ${r2(cx - rad)} ${r2(cy + k)} ${r2(cx - rad)} ${r2(cy)} c\n`;
    s += `${r2(cx - rad)} ${r2(cy - k)} ${r2(cx - k)} ${r2(cy - rad)} ${r2(cx)} ${r2(cy - rad)} c\n`;
    s += `${r2(cx + k)} ${r2(cy - rad)} ${r2(cx + rad)} ${r2(cy - k)} ${r2(cx + rad)} ${r2(cy)} c\n`;
    return s;
  }

  fill(): void {
    const [r, g, b] = this.fillRgb();
    let path = "";
    for (const c of this.circles) path += this.circlePath(c.cx, c.cy, c.r);
    path += this.d;
    if (!path.trim()) return;
    this.body.push(`${c3(r)} ${c3(g)} ${c3(b)} rg\n${path}f`);
  }
  stroke(): void {
    const [r, g, b] = this.strokeRgb();
    let path = "";
    for (const c of this.circles) path += this.circlePath(c.cx, c.cy, c.r);
    path += this.d;
    if (!path.trim()) return;
    const dash = this.dash.length ? `[${this.dash.map(r2).join(" ")}] 0 d\n` : "[] 0 d\n";
    // No line-cap/join override: PDF defaults (butt cap, miter join) match canvas — and therefore
    // the on-screen render and the SVG export — rather than silently prettifying to round.
    this.body.push(`${c3(r)} ${c3(g)} ${c3(b)} RG\n${r2(this.lineWidth)} w\n${dash}${path}S`);
  }
  fillRect(x: number, y: number, w: number, h: number): void {
    const [x0, y0] = this.toPdf(x, y);
    const [x1, y1] = this.toPdf(x + w, y + h);
    const [r, g, b] = this.fillRgb();
    this.body.push(
      `${c3(r)} ${c3(g)} ${c3(b)} rg\n${r2(Math.min(x0, x1))} ${r2(Math.min(y0, y1))} ${r2(Math.abs(x1 - x0))} ${r2(Math.abs(y1 - y0))} re f`,
    );
  }
  strokeRect(x: number, y: number, w: number, h: number): void {
    const [x0, y0] = this.toPdf(x, y);
    const [x1, y1] = this.toPdf(x + w, y + h);
    const [r, g, b] = this.strokeRgb();
    const dash = this.dash.length ? `[${this.dash.map(r2).join(" ")}] 0 d\n` : "[] 0 d\n";
    this.body.push(
      `${c3(r)} ${c3(g)} ${c3(b)} RG\n${r2(this.lineWidth)} w\n${dash}${r2(Math.min(x0, x1))} ${r2(Math.min(y0, y1))} ${r2(Math.abs(x1 - x0))} ${r2(Math.abs(y1 - y0))} re S`,
    );
  }
  clearRect(): void {
    // The page starts with an explicit background fill; nothing to erase.
  }

  private fontSizePx(): number {
    const mm = /(\d+(?:\.\d+)?)px/.exec(this.font);
    return mm ? parseFloat(mm[1]) : 10;
  }
  private isBold(): boolean {
    const mm = /^(\d+)\s+/.exec(this.font.trim());
    return mm ? parseInt(mm[1], 10) >= 600 : /bold/i.test(this.font);
  }
  /** textBaseline → fraction of the em to shift the origin toward the descender (device +y),
   *  converting canvas baselines to an alphabetic PDF text origin. Approximate (glyph metrics
   *  aren't available), matching the SVG export's dominant-baseline approximation. */
  private baselineShift(): number {
    switch (this.textBaseline) {
      case "middle":
        return 0.35;
      case "top":
        return 0.8;
      case "hanging":
        return 0.75;
      case "bottom":
        return -0.2;
      case "ideographic":
        return -0.12;
      default:
        return 0; // alphabetic
    }
  }
  fillText(text: string, x: number, y: number): void {
    const s = String(text);
    if (!s) return;
    const size = this.fontSizePx();
    const m = this.m;
    // Device origin, then align/baseline shifts in device space (the plots never scale text, so
    // the matrix's x-basis / y-basis are unit directions for the advance / descender).
    let dx = m[0] * x + m[2] * y + m[4];
    let dy = m[1] * x + m[3] * y + m[5];
    const width = measurePaperText(
      this.font,
      s,
      this.textAlign as CanvasTextAlign,
      this.textBaseline as CanvasTextBaseline,
    ).width;
    const alignFrac = this.textAlign === "center" ? 0.5 : this.textAlign === "right" || this.textAlign === "end" ? 1 : 0;
    dx -= alignFrac * width * m[0];
    dy -= alignFrac * width * m[1];
    const bl = this.baselineShift() * size;
    dx += bl * m[2];
    dy += bl * m[3];
    // Text matrix: advance = (m0, -m1); ascender = (-m2, m3); origin = device flipped to PDF.
    const tm = `${r2(m[0])} ${r2(-m[1])} ${r2(-m[2])} ${r2(m[3])} ${r2(dx)} ${r2(this.height - dy)}`;
    const [r, g, b] = this.fillRgb();
    const font = this.isBold() ? "/F2" : "/F1";
    this.body.push(`q\nBT ${font} ${r2(size)} Tf\n${c3(r)} ${c3(g)} ${c3(b)} rg\n${tm} Tm\n(${pdfText(s)}) Tj\nET\nQ`);
  }
  measureText(text: string): { width: number } {
    const metrics = measurePaperText(
      this.font,
      String(text),
      this.textAlign as CanvasTextAlign,
      this.textBaseline as CanvasTextBaseline,
    );
    return { width: metrics.width };
  }

  /** The assembled content stream: a full-page background fill, then every recorded operator,
   *  with any unbalanced q closed. */
  toContentStream(): string {
    const [br, bg, bb] = this.bgRgb;
    const head = this.bg ? `${c3(br)} ${c3(bg)} ${c3(bb)} rg\n0 0 ${r2(this.width)} ${r2(this.height)} re f\n` : "";
    let tail = "";
    for (let i = 0; i < this.open; i++) tail += "\nQ";
    return head + this.body.join("\n") + tail + "\n";
  }
}

/** The single-chart PDF payload: a content stream plus the page size in points (1 logical px = 1
 *  pt), matched to the live plot so the figure crops tight to the chart. */
export interface PlotPdf {
  content: string;
  widthPt: number;
  heightPt: number;
  paperRecord?: PaperExportRecord;
}

/** Drives a panel's static-draw callback through a PdfRecorder and returns the PDF content stream
 *  + page size, or null if the draw produced no plot. `width`/`height` are logical (CSS) pixels —
 *  pass the live plot's `width`/`height` so the export matches the screen. */
export function renderPlotToPdf(
  width: number,
  height: number,
  draw: (canvas: HTMLCanvasElement) => PlotCanvas | null,
): PlotPdf | null {
  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(width));
  c.height = Math.max(1, Math.round(height));
  const rec = new PdfRecorder(c.width, c.height);
  // A detached canvas inherits the root theme (readTheme falls back to documentElement); use the
  // same background the draw code will, so both the page fill and alpha flattening match.
  rec.setBackground(readTheme(document.documentElement).bg);
  const holder = c as unknown as { __recordingCtx2d?: CanvasRenderingContext2D };
  holder.__recordingCtx2d = rec as unknown as CanvasRenderingContext2D;
  let plot: PlotCanvas | null;
  try {
    plot = draw(c);
  } finally {
    delete holder.__recordingCtx2d;
  }
  if (!plot) return null;
  return { content: rec.toContentStream(), widthPt: c.width, heightPt: c.height };
}

/** Paper-space PDF sibling of renderPlotToPaperSvg. SVG preflight measures the exact same draw
 *  primitives, then the PDF recorder reruns that callback and translates every operator into the
 *  expanded physical page. No label, legend or annotation can sit outside the recorded page. */
export function renderPlotToPaperPdf(
  width: number,
  height: number,
  draw: (canvas: HTMLCanvasElement) => PlotCanvas | null,
  scope: PlotAncestryScope,
): PlotPdf | null {
  const measured = measurePlotForPaper(width, height, draw, scope);
  if (!measured) return null;
  const record = buildPaperExportRecord("pdf-vector", width, height, measured.bounds, measured.footer);

  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(width));
  c.height = Math.max(1, Math.round(height));
  const rec = new PdfRecorder(c.width, c.height);
  rec.setBackground(measured.plot.theme?.bg ?? "#ffffff");
  const holder = c as unknown as { __recordingCtx2d?: CanvasRenderingContext2D };
  holder.__recordingCtx2d = rec as unknown as CanvasRenderingContext2D;
  let plot: PlotCanvas | null;
  try {
    plot = draw(c);
    if (plot) {
      rec.font = `${PAPER_PROVENANCE_FONT_PT}px monospace`;
      rec.fillStyle = plot.theme?.text ?? "#000000";
      rec.textAlign = "left";
      rec.textBaseline = "top";
      rec.fillText(measured.footer, measured.footerX, measured.footerY);
    }
  } finally {
    delete holder.__recordingCtx2d;
  }
  if (!plot) return null;

  const translateX = -record.page_bounds.min_x;
  const translateY = record.page_bounds.max_y - c.height;
  const content = `q\n1 0 0 1 ${r2(translateX)} ${r2(translateY)} cm\n${rec.toContentStream()}Q\n`;
  return {
    content,
    widthPt: paperPageWidth(record),
    heightPt: paperPageHeight(record),
    paperRecord: record,
  };
}

/** Writes a chart PDF to a user-picked path: the content stream is assembled into a one-page PDF
 *  document by the backend (`save_plot_pdf` → `composite::assemble_single_page_pdf`). Returns the
 *  path, or null if the dialog was cancelled. */
export async function savePdf(pdf: PlotPdf, defaultName: string, scope?: PlotAncestryScope): Promise<string | null> {
  const dest = await save({
    title: "Export plot as PDF (vector)",
    defaultPath: `${defaultName.replace(/[^\w.-]+/g, "_")}.pdf`,
    filters: [{ name: "PDF document", extensions: ["pdf"] }],
  });
  if (!dest) return null;
  if (pdf.paperRecord && !scope) throw new Error("paper PDF export requires its plot ancestry scope");
  const exportScope = pdf.paperRecord && scope ? { ...scope, paperExportRecord: pdf.paperRecord } : scope;
  return savePlotPdf(dest, pdf.content, pdf.widthPt, pdf.heightPt, exportScope);
}
