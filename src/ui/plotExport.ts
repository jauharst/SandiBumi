import { save } from "@tauri-apps/plugin-dialog";
import { savePlotReductionManifest, savePng } from "../ipc";
import { recordProcess } from "../processLog";
import type { ContextMenuEntry } from "./contextMenu";
import { saveSvg } from "./svgExport";
import { savePdf, type PlotPdf } from "./pdfExport";
import type { PlotReductionExport } from "./plotTypes";

/** Print / copy / export-image actions shared by every canvas-based visualization
 *  (histogram, crossplot, Pickett, correlation) so a chart can leave the app as a picture
 *  for a report or slide. The log view exports through its Composite dialog instead — a
 *  WebGPU canvas does not reliably read back via toDataURL. */

/** Copies the canvas as a PNG onto the system clipboard. */
export async function copyCanvasToClipboard(canvas: HTMLCanvasElement): Promise<void> {
  const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob((b) => resolve(b), "image/png"));
  if (!blob) throw new Error("could not render the plot to an image");
  // ClipboardItem is available in the WebView2 webview; typed loosely for older TS libs.
  const item = new (window as unknown as { ClipboardItem: typeof ClipboardItem }).ClipboardItem({
    "image/png": blob,
  });
  await navigator.clipboard.write([item]);
}

/** Writes the canvas as a PNG to a user-picked path (via the Tauri save dialog + backend).
 *  Returns the written path, or null if the dialog was cancelled. */
export async function saveCanvasAsPng(canvas: HTMLCanvasElement, defaultName: string): Promise<string | null> {
  const dest = await save({
    title: "Export plot as image",
    defaultPath: `${defaultName.replace(/[^\w.-]+/g, "_")}.png`,
    filters: [{ name: "PNG image", extensions: ["png"] }],
  });
  if (!dest) return null;
  const base64 = canvas.toDataURL("image/png").split(",")[1];
  return savePng(dest, base64);
}

/** Prints the canvas image via a hidden iframe (window.print() would print the whole app
 *  chrome). The iframe is removed after the print dialog closes. */
export function printCanvas(canvas: HTMLCanvasElement, title: string): void {
  const dataUrl = canvas.toDataURL("image/png");
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
    frame.remove();
    return;
  }
  doc.open();
  doc.write(
    `<html><head><title>${title}</title>` +
      `<style>@page{margin:12mm}html,body{margin:0}img{max-width:100%}</style></head>` +
      `<body><img src="${dataUrl}"></body></html>`,
  );
  doc.close();
  const cleanup = () => window.setTimeout(() => frame.remove(), 500);
  const win = frame.contentWindow!;
  win.onafterprint = cleanup;
  const img = doc.querySelector("img");
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
): void {
  if (!canvas) {
    setStatus("No plot to export yet");
    return;
  }
  if (action === "copy") {
    void copyCanvasToClipboard(canvas)
      .then(() => {
        setStatus(`${name} copied to clipboard`);
        recordProcess("Export", `${name} copied to clipboard`);
      })
      .catch((err) => setStatus(`Copy failed: ${err}`));
  } else if (action === "save") {
    void saveCanvasAsPng(canvas, name)
      .then((path) => {
        if (path) {
          setStatus(`${name} image saved to ${path}`);
          recordProcess("Export", `${name} image → ${path}`);
        }
      })
      .catch((err) => setStatus(`Save failed: ${err}`));
  } else {
    printCanvas(canvas, name);
    setStatus(`Printing ${name}…`);
  }
}

/** Saves the plot as a true-vector SVG (via the panel's `getSvg`, which re-runs the chart's
 *  static draw through a recording context). No-ops with a status note when there's no plot. */
export function svgAction(getSvg: () => string | null, name: string, setStatus: (text: string) => void): void {
  const svg = getSvg();
  if (!svg) {
    setStatus("No plot to export yet");
    return;
  }
  void saveSvg(svg, name)
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
export function pdfAction(getPdf: () => PlotPdf | null, name: string, setStatus: (text: string) => void): void {
  const pdf = getPdf();
  if (!pdf) {
    setStatus("No plot to export yet");
    return;
  }
  void savePdf(pdf, name)
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
): ContextMenuEntry[] {
  const entries: ContextMenuEntry[] = [
    { label: "Copy image", onClick: () => imageAction("copy", getCanvas(), name, setStatus) },
    { label: "Save image…", onClick: () => imageAction("save", getCanvas(), name, setStatus) },
  ];
  if (getSvg) entries.push({ label: "Export SVG (vector)…", onClick: () => svgAction(getSvg, name, setStatus) });
  if (getPdf) entries.push({ label: "Export PDF (vector)…", onClick: () => pdfAction(getPdf, name, setStatus) });
  if (getReductionManifest) {
    entries.push({
      label: "Export reduction manifest…",
      onClick: () => reductionManifestAction(getReductionManifest, name, setStatus),
    });
  }
  entries.push({ label: "Print…", onClick: () => imageAction("print", getCanvas(), name, setStatus) });
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
): HTMLElement {
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
  mk("⧉ Copy", "Copy this plot as an image to the clipboard", () => imageAction("copy", getCanvas(), name, setStatus));
  mk("⭳ Image", "Export this plot as a PNG image", () => imageAction("save", getCanvas(), name, setStatus));
  if (getSvg) mk("⭳ SVG", "Export this plot as a true-vector SVG", () => svgAction(getSvg, name, setStatus));
  if (getPdf) mk("⭳ PDF", "Export this plot as a true-vector PDF", () => pdfAction(getPdf, name, setStatus));
  if (getReductionManifest) {
    mk("⭳ Manifest", "Export original/displayed counts and reduction algorithms", () =>
      reductionManifestAction(getReductionManifest, name, setStatus));
  }
  mk("⎙ Print", "Print this plot", () => imageAction("print", getCanvas(), name, setStatus));
  return wrap;
}
