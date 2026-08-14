import { save } from "@tauri-apps/plugin-dialog";
import {
  getCurveAncestryDisclosures,
  savePlotReductionManifest,
  savePng,
  serializePlotBindingExport,
  type PlotAncestryScope,
} from "../ipc";
import { recordProcess } from "../processLog";
import type { ContextMenuEntry } from "./contextMenu";
import { saveSvg } from "./svgExport";
import { savePdf, type PlotPdf } from "./pdfExport";
import type { PlotReductionExport } from "./plotTypes";

const canvasScopes = new WeakMap<HTMLCanvasElement, () => PlotAncestryScope>();

/** Returns the live scope registered by a plot's own toolbar. The generic dock context menu uses
 *  this instead of guessing a well or exporting all-project ancestry. */
export function plotAncestryScope(canvas: HTMLCanvasElement | null): PlotAncestryScope {
  if (!canvas) throw new Error("No plot to export yet");
  const getScope = canvasScopes.get(canvas);
  if (!getScope) throw new Error("Plot export refused: this plot has no declared ancestry scope");
  return getScope();
}

/** Print / copy / export-image actions shared by every canvas-based visualization
 *  (histogram, crossplot, Pickett, correlation) so a chart can leave the app as a picture
 *  for a report or slide. The log view exports through its Composite dialog instead — a
 *  WebGPU canvas does not reliably read back via toDataURL. */

function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc & 1) !== 0 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function withPngText(png: Uint8Array, keywordText: string, jsonText: string): Uint8Array {
  const signature = [137, 80, 78, 71, 13, 10, 26, 10];
  if (!signature.every((byte, index) => png[index] === byte)) throw new Error("canvas did not produce a PNG");
  let offset = 8;
  let iend = -1;
  while (offset + 12 <= png.length) {
    const length = new DataView(png.buffer, png.byteOffset + offset, 4).getUint32(0, false);
    const type = String.fromCharCode(...png.slice(offset + 4, offset + 8));
    if (type === "IEND") {
      iend = offset;
      break;
    }
    offset += 12 + length;
  }
  if (iend < 0) throw new Error("canvas PNG has no IEND chunk");
  const keyword = new TextEncoder().encode(`${keywordText}\0`);
  const json = new TextEncoder().encode(jsonText);
  const data = new Uint8Array(keyword.length + json.length);
  data.set(keyword);
  data.set(json, keyword.length);
  const type = new TextEncoder().encode("tEXt");
  const chunk = new Uint8Array(12 + data.length);
  new DataView(chunk.buffer).setUint32(0, data.length, false);
  chunk.set(type, 4);
  chunk.set(data, 8);
  const crcInput = new Uint8Array(type.length + data.length);
  crcInput.set(type);
  crcInput.set(data, type.length);
  new DataView(chunk.buffer).setUint32(8 + data.length, crc32(crcInput), false);
  const out = new Uint8Array(png.length + chunk.length);
  out.set(png.slice(0, iend));
  out.set(chunk, iend);
  out.set(png.slice(iend), iend + chunk.length);
  return out;
}

async function canvasAncestry(
  canvas: HTMLCanvasElement,
  scope: PlotAncestryScope,
): Promise<{ png: Blob; ancestryJson: string; bindingJson: string }> {
  const [blob, ancestry, bindingJson] = await Promise.all([
    new Promise<Blob | null>((resolve) => canvas.toBlob((value) => resolve(value), "image/png")),
    getCurveAncestryDisclosures(scope),
    scope.plotBindings
      ? serializePlotBindingExport(scope.wellIds, scope.plotBindings, scope.axisRanges ?? [])
      : Promise.resolve(""),
  ]);
  if (!blob) throw new Error("could not render the plot to an image");
  const ancestryJson = JSON.stringify(ancestry);
  let png = withPngText(
    new Uint8Array(await blob.arrayBuffer()),
    "SandiBumiCurveAncestry",
    ancestryJson,
  );
  if (bindingJson) png = withPngText(png, "SandiBumiPlotBindings", bindingJson);
  return { png: new Blob([png], { type: "image/png" }), ancestryJson, bindingJson };
}

/** Copies the canvas as an ancestry-bearing PNG onto the system clipboard. */
export async function copyCanvasToClipboard(canvas: HTMLCanvasElement, scope: PlotAncestryScope): Promise<void> {
  const { png } = await canvasAncestry(canvas, scope);
  // ClipboardItem is available in the WebView2 webview; typed loosely for older TS libs.
  const item = new (window as unknown as { ClipboardItem: typeof ClipboardItem }).ClipboardItem({
    "image/png": png,
  });
  await navigator.clipboard.write([item]);
}

/** Writes the canvas as a PNG to a user-picked path (via the Tauri save dialog + backend).
 *  Returns the written path, or null if the dialog was cancelled. */
export async function saveCanvasAsPng(
  canvas: HTMLCanvasElement,
  defaultName: string,
  scope: PlotAncestryScope,
): Promise<string | null> {
  const dest = await save({
    title: "Export plot as image",
    defaultPath: `${defaultName.replace(/[^\w.-]+/g, "_")}.png`,
    filters: [{ name: "PNG image", extensions: ["png"] }],
  });
  if (!dest) return null;
  const base64 = canvas.toDataURL("image/png").split(",")[1];
  return savePng(dest, base64, scope);
}

/** Prints the canvas image via a hidden iframe (window.print() would print the whole app
 *  chrome). The iframe is removed after the print dialog closes. */
export async function printCanvas(canvas: HTMLCanvasElement, title: string, scope: PlotAncestryScope): Promise<void> {
  const { png, ancestryJson, bindingJson } = await canvasAncestry(canvas, scope);
  if (png.type !== "image/png") throw new Error("print export did not produce labelled PNG raster bytes");
  const dataUrl = URL.createObjectURL(png);
  const frame = document.createElement("iframe");
  frame.setAttribute("aria-hidden", "true");
  Object.assign(frame.style, {
    position: "fixed",
    right: "0",
    bottom: "0",
    width: "0",
    height: "0",
    border: "0",
  });
  document.body.appendChild(frame);
  const doc = frame.contentWindow?.document;
  if (!doc) {
    URL.revokeObjectURL(dataUrl);
    frame.remove();
    return;
  }
  doc.open();
  doc.write("<html><head><style>@page{margin:12mm}html,body{margin:0}img{max-width:100%}pre{white-space:pre-wrap;font:8pt monospace;page-break-before:always}</style></head><body></body></html>");
  doc.close();
  doc.title = title;
  const img = doc.createElement("img");
  img.src = dataUrl;
  const ancestry = doc.createElement("pre");
  ancestry.textContent = `SANDIBUMI_CURVE_ANCESTRY_V1\n${ancestryJson}`;
  if (bindingJson) {
    const bindings = doc.createElement("pre");
    bindings.textContent = `SANDIBUMI_PLOT_BINDINGS_V1\n${bindingJson}`;
    doc.body.append(img, ancestry, bindings);
  } else {
    doc.body.append(img, ancestry);
  }
  const cleanup = () => window.setTimeout(() => {
    URL.revokeObjectURL(dataUrl);
    frame.remove();
  }, 500);
  const win = frame.contentWindow!;
  win.onafterprint = cleanup;
  const go = () => {
    win.focus();
    win.print();
  };
  if (img && !img.complete) img.onload = go;
  else go();
}

/** Runs an image action against the panel's canvas, reporting via the status bar. */
export function imageAction(
  action: "copy" | "save" | "print",
  canvas: HTMLCanvasElement | null,
  name: string,
  setStatus: (text: string) => void,
  getScope: () => PlotAncestryScope,
): void {
  if (!canvas) {
    setStatus("No plot to export yet");
    return;
  }
  let scope: PlotAncestryScope;
  try {
    scope = getScope();
  } catch (error) {
    setStatus(`${name} export refused: ${error}`);
    return;
  }
  if (action === "copy") {
    void copyCanvasToClipboard(canvas, scope)
      .then(() => {
        setStatus(`${name} copied to clipboard`);
        recordProcess("Export", `${name} copied to clipboard`);
      })
      .catch((err) => setStatus(`Copy failed: ${err}`));
  } else if (action === "save") {
    void saveCanvasAsPng(canvas, name, scope)
      .then((path) => {
        if (path) {
          setStatus(`${name} image saved to ${path}`);
          recordProcess("Export", `${name} image → ${path}`);
        }
      })
      .catch((err) => setStatus(`Save failed: ${err}`));
  } else {
    void printCanvas(canvas, name, scope).catch((err) => setStatus(`Print failed: ${err}`));
    setStatus(`Printing ${name}…`);
  }
}

/** Saves the plot as a true-vector SVG (via the panel's `getSvg`, which re-runs the chart's
 *  static draw through a recording context). No-ops with a status note when there's no plot. */
export function svgAction(
  getSvg: () => string | null,
  name: string,
  setStatus: (text: string) => void,
  getScope: () => PlotAncestryScope,
): void {
  const svg = getSvg();
  if (!svg) {
    setStatus("No plot to export yet");
    return;
  }
  let scope: PlotAncestryScope;
  try {
    scope = getScope();
  } catch (error) {
    setStatus(`${name} SVG export refused: ${error}`);
    return;
  }
  void saveSvg(svg, name, scope)
    .then((path) => {
      if (path) {
        setStatus(`${name} SVG saved to ${path}`);
        recordProcess("Export", `${name} SVG (vector) → ${path}`);
      }
    })
    .catch((err) => setStatus(`SVG export failed: ${err}`));
}

/** Saves the plot as a true-vector PDF (via the panel's `getPdf`, which re-runs the chart's static
 *  draw through a recording context into a PDF content stream). No-ops with a status note when
 *  there's no plot. */
export function pdfAction(
  getPdf: () => PlotPdf | null,
  name: string,
  setStatus: (text: string) => void,
  getScope: () => PlotAncestryScope,
): void {
  const pdf = getPdf();
  if (!pdf) {
    setStatus("No plot to export yet");
    return;
  }
  let scope: PlotAncestryScope;
  try {
    scope = getScope();
  } catch (error) {
    setStatus(`${name} PDF export refused: ${error}`);
    return;
  }
  void savePdf(pdf, name, scope)
    .then((path) => {
      if (path) {
        setStatus(`${name} PDF saved to ${path}`);
        recordProcess("Export", `${name} PDF (vector) → ${path}`);
      }
    })
    .catch((err) => setStatus(`PDF export failed: ${err}`));
}

/** Saves a validated, machine-readable disclosure of every applied plot reduction. */
export function reductionManifestAction(
  getManifest: () => PlotReductionExport | null,
  name: string,
  setStatus: (text: string) => void,
): void {
  const manifest = getManifest();
  if (!manifest) {
    setStatus("No plot reduction or refusal to export");
    return;
  }
  void save({
    title: "Export plot reduction manifest",
    defaultPath: `${name.replace(/[^\w.-]+/g, "_")}_reduction.json`,
    filters: [{ name: "Plot reduction manifest", extensions: ["json"] }],
  })
    .then((dest) => {
      if (!dest) return null;
      return savePlotReductionManifest(dest, JSON.stringify(manifest));
    })
    .then((path) => {
      if (path) {
        setStatus(`${name} reduction manifest saved to ${path}`);
        recordProcess("Export", `${name} reduction manifest → ${path}`);
      }
    })
    .catch((err) => setStatus(`Reduction manifest export failed: ${err}`));
}

/** The image entries for a canvas panel's right-click menu. When `getSvg` / `getPdf` are given,
 *  the matching vector-export entries are included. */
export function imageExportMenuEntries(
  getCanvas: () => HTMLCanvasElement | null,
  name: string,
  setStatus: (text: string) => void,
  getSvg?: () => string | null,
  getPdf?: () => PlotPdf | null,
  getReductionManifest?: () => PlotReductionExport | null,
  getScope?: () => PlotAncestryScope,
): ContextMenuEntry[] {
  if (!getScope) throw new Error("Plot export controls require an ancestry scope");
  const scope = getScope;
  const entries: ContextMenuEntry[] = [
    { label: "Copy image", onClick: () => imageAction("copy", getCanvas(), name, setStatus, scope) },
    { label: "Save image…", onClick: () => imageAction("save", getCanvas(), name, setStatus, scope) },
  ];
  if (getSvg) entries.push({ label: "Export SVG (vector)…", onClick: () => svgAction(getSvg, name, setStatus, scope) });
  if (getPdf) entries.push({ label: "Export PDF (vector)…", onClick: () => pdfAction(getPdf, name, setStatus, scope) });
  if (getReductionManifest) {
    entries.push({
      label: "Export reduction manifest…",
      onClick: () => reductionManifestAction(getReductionManifest, name, setStatus),
    });
  }
  entries.push({ label: "Print…", onClick: () => imageAction("print", getCanvas(), name, setStatus, scope) });
  return entries;
}

/** A compact toolbar group (Copy / Image / [SVG] / [PDF] / Print) for a plot panel's toolbar.
 *  `getCanvas` is called lazily so it always targets the panel's current canvas; when `getSvg` /
 *  `getPdf` are supplied the matching vector-export buttons are added. */
export function buildImageExportButtons(
  getCanvas: () => HTMLCanvasElement | null,
  name: string,
  setStatus: (text: string) => void,
  getSvg?: () => string | null,
  getPdf?: () => PlotPdf | null,
  getReductionManifest?: () => PlotReductionExport | null,
  getScope?: () => PlotAncestryScope,
): HTMLElement {
  if (!getScope) throw new Error("Plot export controls require an ancestry scope");
  const scope = getScope;
  const wrap = document.createElement("div");
  wrap.className = "plot-export-group";
  const mk = (label: string, title: string, onClick: () => void) => {
    const b = document.createElement("button");
    b.className = "plot-export-btn";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    wrap.appendChild(b);
  };
  mk("⧉ Copy", "Copy this plot as an image to the clipboard", () => imageAction("copy", getCanvas(), name, setStatus, scope));
  mk("⭳ Image", "Export this plot as a PNG image", () => imageAction("save", getCanvas(), name, setStatus, scope));
  if (getSvg) mk("⭳ SVG", "Export this plot as a true-vector SVG", () => svgAction(getSvg, name, setStatus, scope));
  if (getPdf) mk("⭳ PDF", "Export this plot as a true-vector PDF", () => pdfAction(getPdf, name, setStatus, scope));
  if (getReductionManifest) {
    mk("⭳ Manifest", "Export original/displayed counts and reduction algorithms", () =>
      reductionManifestAction(getReductionManifest, name, setStatus));
  }
  mk("⎙ Print", "Print this plot", () => imageAction("print", getCanvas(), name, setStatus, scope));
  queueMicrotask(() => {
    const canvas = getCanvas();
    if (canvas) canvasScopes.set(canvas, scope);
  });
  return wrap;
}
