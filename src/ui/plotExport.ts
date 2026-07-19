import { save } from "@tauri-apps/plugin-dialog";
import { savePng } from "../ipc";
import { recordProcess } from "../processLog";
import type { ContextMenuEntry } from "./contextMenu";

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

/** The three image entries for a canvas panel's right-click menu. */
export function imageExportMenuEntries(
  getCanvas: () => HTMLCanvasElement | null,
  name: string,
  setStatus: (text: string) => void,
): ContextMenuEntry[] {
  return [
    { label: "Copy image", onClick: () => imageAction("copy", getCanvas(), name, setStatus) },
    { label: "Save image…", onClick: () => imageAction("save", getCanvas(), name, setStatus) },
    { label: "Print…", onClick: () => imageAction("print", getCanvas(), name, setStatus) },
  ];
}

/** A compact toolbar group (Copy / Save / Print) for a plot panel's toolbar. `getCanvas`
 *  is called lazily so it always targets the panel's current canvas. */
export function buildImageExportButtons(
  getCanvas: () => HTMLCanvasElement | null,
  name: string,
  setStatus: (text: string) => void,
): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "plot-export-group";
  const mk = (label: string, title: string, action: "copy" | "save" | "print") => {
    const b = document.createElement("button");
    b.className = "plot-export-btn";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", () => imageAction(action, getCanvas(), name, setStatus));
    wrap.appendChild(b);
  };
  mk("⧉ Copy", "Copy this plot as an image to the clipboard", "copy");
  mk("⭳ Image", "Export this plot as a PNG image", "save");
  mk("⎙ Print", "Print this plot", "print");
  return wrap;
}
