import { save } from "@tauri-apps/plugin-dialog";
import { savePng, type PaperExportBounds, type PaperExportRecord, type PlotAncestryScope } from "../ipc";
import type { PlotCanvas } from "./plotCanvas";
import {
  buildPaperExportRecord,
  PAPER_MARGIN_PT,
  PAPER_PROVENANCE_FONT_PT,
  paperPageHeight,
  paperPageWidth,
  paperProvenanceFooter,
  measurePaperText,
  validatePaperExportRecord,
} from "./paperExport";

/** True-vector export for the Canvas-2D plots (crossplot, histogram, Pickett). Rather than
 *  re-implementing each chart in SVG (which would drift from the on-screen version), we drive
 *  the *same* draw functions through a recording 2D context that duck-types
 *  CanvasRenderingContext2D and serialises every call to SVG. A detached canvas carries the
 *  recorder via a private property that PlotCanvas reads, so `drawCrossplot(canvas, …)` etc.
 *  paint into the recorder unchanged. Only the subset of the 2D API the plot code uses is
 *  implemented — rectangular clips, full-circle arcs, and textAlign/textBaseline; a missing
 *  *method* throws rather than mis-rendering. A couple of degenerate cases the plots never hit
 *  (path-based clips, non-uniformly-scaled arcs) are not modelled. */

type Mat = [number, number, number, number, number, number];

const IDENTITY: Mat = [1, 0, 0, 1, 0, 0];

/** Compose canvas transforms: apply B, then A (A ∘ B), matching ctx.transform semantics. */
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

/** Round to 2 dp and neutralise non-finite values so the SVG never contains NaN/Infinity. */
function n(v: number): number {
  return Number.isFinite(v) ? Math.round(v * 100) / 100 : 0;
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
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
  lineDash: number[];
  /** How many clip groups were open when this frame was pushed (closed on restore). */
  groups: number;
}

/** A minimal CanvasRenderingContext2D-compatible recorder that emits an SVG document. */
export class SvgRecorder {
  private body: string[] = [];
  private defs: string[] = [];
  private clipId = 0;
  private openGroups = 0;
  private stack: Frame[] = [];
  // Current path accumulation (user space transformed to device on the fly).
  private d = "";
  private circles: { cx: number; cy: number; r: number }[] = [];
  private pendingRect: { x: number; y: number; w: number; h: number } | null = null;
  private bounds: PaperExportBounds;

  m: Mat = [...IDENTITY] as Mat;
  fillStyle = "#000000";
  strokeStyle = "#000000";
  lineWidth = 1;
  font = "10px sans-serif";
  textAlign = "start";
  textBaseline = "alphabetic";
  globalAlpha = 1;
  private dash: number[] = [];

  constructor(
    readonly width: number,
    readonly height: number,
  ) {
    this.bounds = { min_x: 0, min_y: 0, max_x: width, max_y: height };
  }

  private includePoint(x: number, y: number, pad = 0): void {
    this.bounds.min_x = Math.min(this.bounds.min_x, x - pad);
    this.bounds.min_y = Math.min(this.bounds.min_y, y - pad);
    this.bounds.max_x = Math.max(this.bounds.max_x, x + pad);
    this.bounds.max_y = Math.max(this.bounds.max_y, y + pad);
  }

  private includeRect(x: number, y: number, w: number, h: number, pad = 0): void {
    this.includePoint(Math.min(x, x + w), Math.min(y, y + h), pad);
    this.includePoint(Math.max(x, x + w), Math.max(y, y + h), pad);
  }

  getContentBounds(): PaperExportBounds {
    return { ...this.bounds };
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
      lineDash: [...this.dash],
      groups: this.openGroups,
    });
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
    this.dash = f.lineDash;
    while (this.openGroups > f.groups) {
      this.body.push("</g>");
      this.openGroups--;
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

  private apply(x: number, y: number): [number, number] {
    const m = this.m;
    return [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
  }
  /** Uniform-ish scale factor of the current matrix, for circle radii. */
  private scaleFactor(): number {
    return Math.hypot(this.m[0], this.m[1]) || 1;
  }

  // --- path ---------------------------------------------------------------
  beginPath(): void {
    this.d = "";
    this.circles = [];
    this.pendingRect = null;
  }
  moveTo(x: number, y: number): void {
    const [px, py] = this.apply(x, y);
    this.includePoint(px, py, (this.lineWidth * this.scaleFactor()) / 2);
    this.d += `M${n(px)} ${n(py)} `;
  }
  lineTo(x: number, y: number): void {
    const [px, py] = this.apply(x, y);
    this.includePoint(px, py, (this.lineWidth * this.scaleFactor()) / 2);
    this.d += `L${n(px)} ${n(py)} `;
  }
  closePath(): void {
    this.d += "Z ";
  }
  arc(cx: number, cy: number, r: number, a0: number, a1: number): void {
    const rr = r * this.scaleFactor();
    if (Math.abs(a1 - a0) >= Math.PI * 2 - 1e-6) {
      const [px, py] = this.apply(cx, cy);
      this.includeRect(px - rr, py - rr, rr * 2, rr * 2, (this.lineWidth * this.scaleFactor()) / 2);
      this.circles.push({ cx: px, cy: py, r: rr });
      return;
    }
    // Partial arc (rare): sample into line segments so the path stays faithful.
    const steps = Math.max(2, Math.ceil((Math.abs(a1 - a0) / (Math.PI * 2)) * 32));
    for (let i = 0; i <= steps; i++) {
      const a = a0 + ((a1 - a0) * i) / steps;
      const [px, py] = this.apply(cx + Math.cos(a) * r, cy + Math.sin(a) * r);
      this.includePoint(px, py, (this.lineWidth * this.scaleFactor()) / 2);
      this.d += `${i === 0 ? "M" : "L"}${n(px)} ${n(py)} `;
    }
  }
  rect(x: number, y: number, w: number, h: number): void {
    // In this codebase rect() is only ever used to build a clip region.
    const [x0, y0] = this.apply(x, y);
    const [x1, y1] = this.apply(x + w, y + h);
    this.includeRect(x0, y0, x1 - x0, y1 - y0);
    this.pendingRect = { x: Math.min(x0, x1), y: Math.min(y0, y1), w: Math.abs(x1 - x0), h: Math.abs(y1 - y0) };
  }
  clip(): void {
    if (!this.pendingRect) return;
    const id = `clip${++this.clipId}`;
    const { x, y, w, h } = this.pendingRect;
    this.defs.push(`<clipPath id="${id}"><rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}"/></clipPath>`);
    this.body.push(`<g clip-path="url(#${id})">`);
    this.openGroups++;
    this.pendingRect = null;
  }

  private alphaAttr(): string {
    return this.globalAlpha < 1 ? ` opacity="${n(this.globalAlpha)}"` : "";
  }
  private dashAttr(): string {
    return this.dash.length ? ` stroke-dasharray="${this.dash.map((v) => n(v)).join(",")}"` : "";
  }

  fill(): void {
    for (const c of this.circles) {
      this.body.push(`<circle cx="${n(c.cx)}" cy="${n(c.cy)}" r="${n(c.r)}" fill="${esc(this.fillStyle)}"${this.alphaAttr()}/>`);
    }
    if (this.d.trim()) {
      this.body.push(`<path d="${this.d.trim()}" fill="${esc(this.fillStyle)}"${this.alphaAttr()}/>`);
    }
  }
  stroke(): void {
    for (const c of this.circles) {
      this.body.push(
        `<circle cx="${n(c.cx)}" cy="${n(c.cy)}" r="${n(c.r)}" fill="none" stroke="${esc(this.strokeStyle)}" stroke-width="${n(this.lineWidth)}"${this.dashAttr()}${this.alphaAttr()}/>`,
      );
    }
    if (this.d.trim()) {
      this.body.push(
        `<path d="${this.d.trim()}" fill="none" stroke="${esc(this.strokeStyle)}" stroke-width="${n(this.lineWidth)}"${this.dashAttr()}${this.alphaAttr()}/>`,
      );
    }
  }
  fillRect(x: number, y: number, w: number, h: number): void {
    const [x0, y0] = this.apply(x, y);
    const [x1, y1] = this.apply(x + w, y + h);
    this.includeRect(x0, y0, x1 - x0, y1 - y0);
    this.body.push(
      `<rect x="${n(Math.min(x0, x1))}" y="${n(Math.min(y0, y1))}" width="${n(Math.abs(x1 - x0))}" height="${n(Math.abs(y1 - y0))}" fill="${esc(this.fillStyle)}"${this.alphaAttr()}/>`,
    );
  }
  strokeRect(x: number, y: number, w: number, h: number): void {
    const [x0, y0] = this.apply(x, y);
    const [x1, y1] = this.apply(x + w, y + h);
    this.includeRect(x0, y0, x1 - x0, y1 - y0, (this.lineWidth * this.scaleFactor()) / 2);
    this.body.push(
      `<rect x="${n(Math.min(x0, x1))}" y="${n(Math.min(y0, y1))}" width="${n(Math.abs(x1 - x0))}" height="${n(Math.abs(y1 - y0))}" fill="none" stroke="${esc(this.strokeStyle)}" stroke-width="${n(this.lineWidth)}"${this.dashAttr()}${this.alphaAttr()}/>`,
    );
  }
  clearRect(): void {
    // SVG starts transparent; the plot background is painted explicitly via fillRect.
  }

  private fontAttrs(): string {
    // canvasFont() emits "[weight ]<px>px <family>"; split it into portable SVG attributes.
    const mm = /^(?:(\d+)\s+)?(\d+(?:\.\d+)?)px\s+(.+)$/.exec(this.font.trim());
    if (!mm) return ` style="font:${esc(this.font)}"`;
    const weight = mm[1] ? ` font-weight="${mm[1]}"` : "";
    return ` font-size="${mm[2]}" font-family="${esc(mm[3])}"${weight}`;
  }
  fillText(text: string, x: number, y: number): void {
    const anchor = this.textAlign === "center" ? "middle" : this.textAlign === "right" || this.textAlign === "end" ? "end" : "start";
    // Canvas textBaseline → SVG dominant-baseline; "alphabetic" (canvas default) is SVG's default
    // so it is omitted, keeping the common tick-label path byte-identical.
    const baseline =
      this.textBaseline === "middle"
        ? ' dominant-baseline="central"'
        : this.textBaseline === "top"
          ? ' dominant-baseline="text-before-edge"'
          : this.textBaseline === "bottom"
            ? ' dominant-baseline="text-after-edge"'
            : this.textBaseline === "hanging"
              ? ' dominant-baseline="hanging"'
              : this.textBaseline === "ideographic"
                ? ' dominant-baseline="ideographic"'
                : "";
    const t = `matrix(${this.m.map((v) => n(v)).join(",")})`;
    const metrics = measurePaperText(
      this.font,
      String(text),
      this.textAlign as CanvasTextAlign,
      this.textBaseline as CanvasTextBaseline,
    );
    for (const [tx, ty] of [
      [x - metrics.left, y - metrics.ascent],
      [x + metrics.right, y - metrics.ascent],
      [x - metrics.left, y + metrics.descent],
      [x + metrics.right, y + metrics.descent],
    ]) {
      const [px, py] = this.apply(tx, ty);
      this.includePoint(px, py);
    }
    this.body.push(
      `<text transform="${t}" x="${n(x)}" y="${n(y)}" text-anchor="${anchor}"${baseline}${this.fontAttrs()} fill="${esc(this.fillStyle)}"${this.alphaAttr()}>${esc(String(text))}</text>`,
    );
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

  /** Serialise everything drawn so far into a standalone SVG document. `bg`, when given,
   *  paints a full-viewport background rectangle first (to match the panel's themed backdrop). */
  toSvg(bg?: string, paperRecord?: PaperExportRecord): string {
    if (paperRecord) validatePaperExportRecord(paperRecord);
    let tail = "";
    for (let i = 0; i < this.openGroups; i++) tail += "</g>";
    const defs = this.defs.length ? `<defs>${this.defs.join("")}</defs>` : "";
    const bgRect = bg ? `<rect x="0" y="0" width="${n(this.width)}" height="${n(this.height)}" fill="${esc(bg)}"/>` : "";
    const metadata = paperRecord
      ? `<metadata id="sandibumi-paper-export-v1">${esc(JSON.stringify(paperRecord))}</metadata>`
      : "";
    const minX = paperRecord?.page_bounds.min_x ?? 0;
    const minY = paperRecord?.page_bounds.min_y ?? 0;
    const rootWidth = paperRecord ? paperPageWidth(paperRecord) : this.width;
    const rootHeight = paperRecord ? paperPageHeight(paperRecord) : this.height;
    const width = paperRecord ? `${n(rootWidth)}pt` : String(n(rootWidth));
    const height = paperRecord ? `${n(rootHeight)}pt` : String(n(rootHeight));
    return (
      `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" ` +
      `viewBox="${n(minX)} ${n(minY)} ${n(rootWidth)} ${n(rootHeight)}">${metadata}${defs}${bgRect}${this.body.join("")}${tail}</svg>`
    );
  }
}

export interface PaperDrawMeasurement {
  recorder: SvgRecorder;
  bounds: PaperExportBounds;
  footer: string;
  footerX: number;
  footerY: number;
  plot: PlotCanvas;
}

/** Runs the exact scientific draw in a recording context, measures every recorded vector mark,
 *  and adds the visible provenance footer below those marks. */
export function measurePlotForPaper(
  width: number,
  height: number,
  draw: (canvas: HTMLCanvasElement) => PlotCanvas | null,
  scope: PlotAncestryScope,
): PaperDrawMeasurement | null {
  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(width));
  c.height = Math.max(1, Math.round(height));
  const recorder = new SvgRecorder(c.width, c.height);
  const holder = c as unknown as { __recordingCtx2d?: CanvasRenderingContext2D };
  holder.__recordingCtx2d = recorder as unknown as CanvasRenderingContext2D;
  let plot: PlotCanvas | null;
  try {
    plot = draw(c);
  } finally {
    delete holder.__recordingCtx2d;
  }
  if (!plot) return null;
  const beforeFooter = recorder.getContentBounds();
  const footer = paperProvenanceFooter(scope);
  const footerX = beforeFooter.min_x;
  const footerY = beforeFooter.max_y + PAPER_MARGIN_PT;
  recorder.font = `${PAPER_PROVENANCE_FONT_PT}px monospace`;
  recorder.fillStyle = plot.theme?.text ?? "#000000";
  recorder.textAlign = "left";
  recorder.textBaseline = "top";
  recorder.fillText(footer, footerX, footerY);
  return { recorder, bounds: recorder.getContentBounds(), footer, footerX, footerY, plot };
}

/** Paper-space SVG: same draw callback, point-sized root, visible footer and measured no-crop page. */
export function renderPlotToPaperSvg(
  width: number,
  height: number,
  draw: (canvas: HTMLCanvasElement) => PlotCanvas | null,
  scope: PlotAncestryScope,
): string | null {
  const measured = measurePlotForPaper(width, height, draw, scope);
  if (!measured) return null;
  const record = buildPaperExportRecord("svg-vector", width, height, measured.bounds, measured.footer);
  return measured.recorder.toSvg(measured.plot.theme?.bg, record);
}

/** Recovers and validates the custody record generated by renderPlotToPaperSvg before the SVG
 *  crosses IPC. The backend receives the same typed record rather than trusting an opaque string. */
export function paperExportRecordFromSvg(svg: string): PaperExportRecord {
  const match = /<metadata id="sandibumi-paper-export-v1">([\s\S]*?)<\/metadata>/.exec(svg);
  if (!match) throw new Error("paper SVG export lacks its crop and provenance record");
  const json = match[1]
    .replace(/&quot;/g, '"')
    .replace(/&gt;/g, ">")
    .replace(/&lt;/g, "<")
    .replace(/&amp;/g, "&");
  const record = JSON.parse(json) as PaperExportRecord;
  validatePaperExportRecord(record);
  if (record.medium !== "svg-vector") throw new Error("paper SVG record names the wrong medium");
  return record;
}

/** Wraps a third-party vector renderer's measured scenegraph in the same paper record used by
 *  Canvas plots. Vega supplies the actual scenegraph bounds, including outside legends and labels;
 *  the nested SVG is marked overflow-visible and the outer point-sized page encloses those bounds. */
export function paperizeMeasuredSvg(
  svg: string,
  sourceWidth: number,
  sourceHeight: number,
  measuredBounds: PaperExportBounds,
  scope: PlotAncestryScope,
): string {
  const baseBounds: PaperExportBounds = {
    min_x: Math.min(0, measuredBounds.min_x),
    min_y: Math.min(0, measuredBounds.min_y),
    max_x: Math.max(sourceWidth, measuredBounds.max_x),
    max_y: Math.max(sourceHeight, measuredBounds.max_y),
  };
  const footer = paperProvenanceFooter(scope);
  const footerX = baseBounds.min_x;
  const footerY = baseBounds.max_y + PAPER_MARGIN_PT;
  const footerMetrics = measurePaperText(
    `${PAPER_PROVENANCE_FONT_PT}px monospace`,
    footer,
    "left",
    "top",
  );
  const contentBounds: PaperExportBounds = {
    min_x: Math.min(baseBounds.min_x, footerX - footerMetrics.left),
    min_y: Math.min(baseBounds.min_y, footerY - footerMetrics.ascent),
    max_x: Math.max(baseBounds.max_x, footerX + footerMetrics.right),
    max_y: Math.max(baseBounds.max_y, footerY + footerMetrics.descent),
  };
  const record = buildPaperExportRecord("svg-vector", sourceWidth, sourceHeight, contentBounds, footer);
  validatePaperExportRecord(record);
  const inner = svg
    .replace(/^\s*<\?xml[^>]*>\s*/i, "")
    .replace(/<svg\b/, '<svg overflow="visible"');
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" width="${n(paperPageWidth(record))}pt" height="${n(paperPageHeight(record))}pt" ` +
    `viewBox="${n(record.page_bounds.min_x)} ${n(record.page_bounds.min_y)} ${n(paperPageWidth(record))} ${n(paperPageHeight(record))}">` +
    `<metadata id="sandibumi-paper-export-v1">${esc(JSON.stringify(record))}</metadata>${inner}` +
    `<text x="${n(footerX)}" y="${n(footerY)}" dominant-baseline="text-before-edge" font-size="${PAPER_PROVENANCE_FONT_PT}" font-family="monospace">${esc(footer)}</text></svg>`
  );
}

/** Drives a panel's static-draw callback through an SvgRecorder and returns the SVG string,
 *  or null if the draw produced no plot (e.g. no valid data). `width`/`height` are logical
 *  (CSS) pixels — pass the live plot's `width`/`height` so the export matches the screen. */
export function renderPlotToSvg(
  width: number,
  height: number,
  draw: (canvas: HTMLCanvasElement) => PlotCanvas | null,
): string | null {
  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(width));
  c.height = Math.max(1, Math.round(height));
  const rec = new SvgRecorder(c.width, c.height);
  const holder = c as unknown as { __recordingCtx2d?: CanvasRenderingContext2D };
  holder.__recordingCtx2d = rec as unknown as CanvasRenderingContext2D;
  let plot: PlotCanvas | null;
  try {
    plot = draw(c);
  } finally {
    delete holder.__recordingCtx2d;
  }
  if (!plot) return null;
  return rec.toSvg(plot.theme?.bg);
}

/** Writes an SVG string to a user-picked path (Tauri save dialog + backend). savePng writes
 *  the decoded bytes verbatim, so it serves for any text payload. Returns the path or null if
 *  the dialog was cancelled. */
export async function saveSvg(svg: string, defaultName: string, scope?: PlotAncestryScope): Promise<string | null> {
  const dest = await save({
    title: "Export plot as SVG (vector)",
    defaultPath: `${defaultName.replace(/[^\w.-]+/g, "_")}.svg`,
    filters: [{ name: "SVG image", extensions: ["svg"] }],
  });
  if (!dest) return null;
  // Base64-encode the UTF-8 bytes for the same transport savePng uses.
  const bytes = new TextEncoder().encode(svg);
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return savePng(dest, btoa(bin), scope);
}
