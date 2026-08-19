import { getCurveData, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";
import { attachResizeRedraw, canvasFont, readTheme, type PlotTheme } from "./plotCanvas";
import {
  buildZoneSelect,
  curveSelect,
  loadCurveNames,
  trySelect,
  type PlotContent,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

/** Unconventional visual companion (playbook #7, increment 5 — the picture behind the
 *  toc_passey / gip compute modules). Two panes that share one well:
 *
 *   • Passey (1990) ΔlogR overlay — deep resistivity (log) and a porosity curve baselined and
 *     scaled so the two overlie in non-source, clay-rich rock and separate over organic-rich
 *     intervals; the separation (shaded) IS ΔlogR, the input to the TOC module. Uses the exact
 *     same scaling as `toc_passey`: ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base) [sonic] or
 *     −2.5·(RHOB−RHOB_base) [density] (docs/ref_unconventional.md §1).
 *
 *   • Langmuir adsorption isotherm — Gs(P) = VL·P/(PL+P) (scf/ton) with the VL asymptote, the
 *     reservoir-pressure operating point, and (for undersaturated coal/shale) the critical
 *     desorption pressure PCD = PL·GC/(VL−GC), exactly the adsorbed term of the `gip` module
 *     (docs/ref_unconventional.md §3).
 *
 *  Both are display-only — no new physics. The overlay reads curves for the active well; the
 *  isotherm is parametric, so it renders even before a well or curves exist. */

/** Sonic decades-per-µs/ft and density decades-per-(g/cc): the standard Passey overlay ties
 *  one resistivity cycle to 50 µs/ft (1/50 = 0.02) and to 0.4 g/cc (1/0.4 = 2.5). Kept in sync
 *  with `toc_passey` in unconventional.rs. */
const DT_PER_DECADE = 0.02;
const RHOB_PER_DECADE = 2.5;

/** DPR-scaled 2D context painted with the theme background (mirrors lorenzDialog's setup so
 *  both custom panels antialias the same on hi-DPI screens). Returns null if unavailable. */
function setupCanvas(
  canvas: HTMLCanvasElement,
  bg: string,
): { ctx: CanvasRenderingContext2D; w: number; h: number } | null {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 420;
  const h = canvas.clientHeight || 480;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, w, h);
  return { ctx, w, h };
}

function centeredMessage(canvas: HTMLCanvasElement, theme: PlotTheme, text: string): void {
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 12, 400);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(text, w / 2, h / 2);
}

interface OverlayData {
  depth: Float32Array;
  res: Float32Array;
  por: Float32Array;
  mode: string; // "sonic" | "density"
  rBase: number;
  dtBase: number;
  rhobBase: number;
  resName: string;
  porName: string;
}

/** Passey ΔlogR overlay in a depth track. Resistivity sits at xR = log10(R/R_base) decades;
 *  the porosity curve is drawn at xPor = xR − ΔlogR = −(porosity term), so it overlies R at the
 *  baseline and swings the OPPOSITE way over source rock — the shaded gap between them is ΔlogR. */
function drawOverlay(canvas: HTMLCanvasElement, theme: PlotTheme, d: OverlayData): void {
  const n = d.depth.length;
  if (n === 0) {
    centeredMessage(canvas, theme, `No ${d.resName} / ${d.porName} data in this interval.`);
    return;
  }
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;

  // Per-sample plot positions (decades) + the separation. NaN where a curve/baseline is missing.
  const xR = new Float64Array(n);
  const xPor = new Float64Array(n);
  const dlogr = new Float64Array(n);
  let xLo = Infinity;
  let xHi = -Infinity;
  let dLo = Infinity;
  let dHi = -Infinity;
  for (let i = 0; i < n; i++) {
    const r = d.res[i];
    const p = d.por[i];
    const xr = r > 0 && d.rBase > 0 ? Math.log10(r / d.rBase) : NaN;
    const poroTerm =
      d.mode === "density" ? -RHOB_PER_DECADE * (p - d.rhobBase) : DT_PER_DECADE * (p - d.dtBase);
    // The porosity curve sits at an ABSOLUTE decade position −poroTerm (0 at its baseline, like
    // resistivity at its own), so the two overlie in non-source rock and fan the OPPOSITE way over
    // source rock. Their gap xR−xPor = log10(R/Rbase)+poroTerm is exactly the module's ΔlogR.
    const xp = Number.isFinite(poroTerm) ? -poroTerm : NaN;
    xR[i] = xr;
    xPor[i] = xp;
    dlogr[i] = xr - xp; // = log10(R/Rbase) + poroTerm  (the toc_passey ΔlogR)
    if (Number.isFinite(xr)) {
      if (xr < xLo) xLo = xr;
      if (xr > xHi) xHi = xr;
    }
    if (Number.isFinite(xp)) {
      if (xp < xLo) xLo = xp;
      if (xp > xHi) xHi = xp;
    }
    const dep = d.depth[i];
    if (Number.isFinite(dep)) {
      if (dep < dLo) dLo = dep;
      if (dep > dHi) dHi = dep;
    }
  }
  if (!Number.isFinite(xLo) || !Number.isFinite(dLo) || dHi <= dLo) {
    centeredMessage(canvas, theme, `No overlapping ${d.resName} / ${d.porName} samples.`);
    return;
  }
  // Snap the decade axis to whole cycles with a little breathing room.
  let axLo = Math.floor(xLo - 0.15);
  let axHi = Math.ceil(xHi + 0.15);
  if (axHi - axLo < 2) axHi = axLo + 2; // never fewer than two visible cycles

  const padL = 30;
  const padR = 12;
  const padT = 34;
  const padB = 24;
  const X = (dec: number) => padL + ((dec - axLo) / (axHi - axLo)) * (w - padL - padR);
  const Y = (dep: number) => padT + ((dep - dLo) / (dHi - dLo)) * (h - padT - padB);

  // Decade gridlines + frame.
  ctx.strokeStyle = theme.grid;
  ctx.lineWidth = 1;
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 9, 400);
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (let dec = axLo; dec <= axHi; dec++) {
    ctx.beginPath();
    ctx.moveTo(X(dec), Y(dLo));
    ctx.lineTo(X(dec), Y(dHi));
    ctx.stroke();
    ctx.fillText(dec === 0 ? "base" : `${dec > 0 ? "+" : ""}${dec}`, X(dec), padT - 11);
  }
  ctx.strokeStyle = theme.axis;
  ctx.strokeRect(padL, padT, w - padL - padR, h - padT - padB);

  // Depth ticks (~6 rounded labels down the left edge).
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  const span = dHi - dLo;
  const step = niceStep(span / 6);
  const first = Math.ceil(dLo / step) * step;
  for (let dep = first; dep <= dHi; dep += step) {
    ctx.strokeStyle = theme.grid;
    ctx.beginPath();
    ctx.moveTo(padL, Y(dep));
    ctx.lineTo(padL - 3, Y(dep));
    ctx.stroke();
    ctx.fillStyle = theme.text;
    ctx.fillText(String(Math.round(dep)), padL - 4, Y(dep));
  }

  // Shade the separation (ΔlogR > 0 → organic-rich) as quads between the two curves.
  ctx.fillStyle = hexAlpha(theme.accent2, 0.28);
  for (let i = 0; i + 1 < n; i++) {
    if (
      dlogr[i] > 0 &&
      dlogr[i + 1] > 0 &&
      Number.isFinite(xR[i]) &&
      Number.isFinite(xPor[i]) &&
      Number.isFinite(xR[i + 1]) &&
      Number.isFinite(xPor[i + 1])
    ) {
      ctx.beginPath();
      ctx.moveTo(X(xPor[i]), Y(d.depth[i]));
      ctx.lineTo(X(xR[i]), Y(d.depth[i]));
      ctx.lineTo(X(xR[i + 1]), Y(d.depth[i + 1]));
      ctx.lineTo(X(xPor[i + 1]), Y(d.depth[i + 1]));
      ctx.closePath();
      ctx.fill();
    }
  }

  drawTrackCurve(ctx, xPor, d.depth, X, Y, theme.accent2, 1.2); // porosity (scaled)
  drawTrackCurve(ctx, xR, d.depth, X, Y, theme.accent, 1.6); // resistivity

  // Header: which curves, the overlay mode, and the baselines in play.
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
  ctx.fillStyle = theme.accent;
  ctx.font = canvasFont(theme, 10, 600);
  ctx.fillText(`▬ ${d.resName}`, padL, 12);
  const baseTxt =
    d.mode === "density"
      ? `RHOB_base ${d.rhobBase} g/cc`
      : `DT_base ${d.dtBase} µs/ft`;
  ctx.fillStyle = theme.accent2;
  ctx.fillText(`▬ ${d.porName} (${d.mode})`, padL + 84, 12);
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 9, 400);
  ctx.fillText(`R_base ${d.rBase} ohm·m · ${baseTxt} · x = resistivity cycles`, padL, h - 8);
}

function drawTrackCurve(
  ctx: CanvasRenderingContext2D,
  x: Float64Array,
  depth: Float32Array,
  X: (v: number) => number,
  Y: (v: number) => number,
  color: string,
  width: number,
): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.beginPath();
  let pen = false;
  for (let i = 0; i < x.length; i++) {
    if (!Number.isFinite(x[i]) || !Number.isFinite(depth[i])) {
      pen = false; // break the line across gaps rather than spanning them
      continue;
    }
    const px = X(x[i]);
    const py = Y(depth[i]);
    if (pen) ctx.lineTo(px, py);
    else ctx.moveTo(px, py);
    pen = true;
  }
  ctx.stroke();
}

interface LangmuirData {
  vl: number; // Langmuir volume, scf/ton
  pl: number; // Langmuir pressure, psia
  pres: number; // reservoir pressure, psia
  gc: number; // in-situ gas content (0 = saturated), scf/ton
}

/** Langmuir isotherm Gs(P) = VL·P/(PL+P): the storage curve, its VL asymptote, the
 *  reservoir-pressure operating point, and (undersaturated) the critical desorption pressure
 *  PCD = PL·GC/(VL−GC) at which free gas first desorbs. */
function drawLangmuir(canvas: HTMLCanvasElement, theme: PlotTheme, d: LangmuirData): void {
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;
  if (!(d.vl > 0) || !(d.pl > 0)) {
    centeredMessage(canvas, theme, "Enter a positive VL and PL.");
    return;
  }

  const undersat = d.gc > 0 && d.gc < d.vl;
  const pcd = undersat ? (d.pl * d.gc) / (d.vl - d.gc) : NaN;
  const pMax = Math.max(d.pres * 1.25, d.pl * 2, Number.isFinite(pcd) ? pcd * 1.2 : 0, 500);
  const gMax = d.vl * 1.08;

  const padL = 52;
  const padR = 14;
  const padT = 26;
  const padB = 34;
  const X = (p: number) => padL + (p / pMax) * (w - padL - padR);
  const Y = (g: number) => padT + (1 - g / gMax) * (h - padT - padB);

  // Gridlines + frame.
  ctx.strokeStyle = theme.grid;
  ctx.lineWidth = 1;
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 9, 400);
  const pStep = niceStep(pMax / 5);
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (let p = 0; p <= pMax + 1e-6; p += pStep) {
    ctx.strokeStyle = theme.grid;
    ctx.beginPath();
    ctx.moveTo(X(p), Y(0));
    ctx.lineTo(X(p), Y(gMax));
    ctx.stroke();
    ctx.fillStyle = theme.text;
    ctx.fillText(String(Math.round(p)), X(p), h - padB + 4);
  }
  const gStep = niceStep(gMax / 5);
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  for (let g = 0; g <= gMax + 1e-6; g += gStep) {
    ctx.strokeStyle = theme.grid;
    ctx.beginPath();
    ctx.moveTo(X(0), Y(g));
    ctx.lineTo(X(pMax), Y(g));
    ctx.stroke();
    ctx.fillStyle = theme.text;
    ctx.fillText(String(Math.round(g)), padL - 5, Y(g));
  }
  ctx.strokeStyle = theme.axis;
  ctx.strokeRect(padL, padT, w - padL - padR, h - padT - padB);

  // VL asymptote (max sorption).
  ctx.strokeStyle = hexAlpha(theme.warn, 0.8);
  ctx.setLineDash([5, 4]);
  ctx.beginPath();
  ctx.moveTo(X(0), Y(d.vl));
  ctx.lineTo(X(pMax), Y(d.vl));
  ctx.stroke();
  ctx.setLineDash([]);

  // The isotherm.
  ctx.strokeStyle = theme.accent;
  ctx.lineWidth = 2;
  ctx.beginPath();
  const steps = 140;
  for (let k = 0; k <= steps; k++) {
    const p = (k / steps) * pMax;
    const g = (d.vl * p) / (d.pl + p);
    if (k === 0) ctx.moveTo(X(p), Y(g));
    else ctx.lineTo(X(p), Y(g));
  }
  ctx.stroke();
  ctx.lineWidth = 1;

  // PL marker: Gs = VL/2 at P = PL.
  marker(ctx, theme, X(d.pl), Y(d.vl / 2), theme.axis);
  label(ctx, theme, X(d.pl), Y(d.vl / 2), `PL ${round(d.pl)}`, "below");

  // Reservoir-pressure operating point.
  if (d.pres > 0 && d.pres <= pMax) {
    const gRes = (d.vl * d.pres) / (d.pl + d.pres);
    ctx.strokeStyle = hexAlpha(theme.accent, 0.5);
    ctx.setLineDash([3, 3]);
    ctx.beginPath();
    ctx.moveTo(X(d.pres), Y(0));
    ctx.lineTo(X(d.pres), Y(gRes));
    ctx.stroke();
    ctx.setLineDash([]);
    marker(ctx, theme, X(d.pres), Y(gRes), theme.accent);
    label(ctx, theme, X(d.pres), Y(gRes), `Pres ${round(d.pres)} → ${round(gRes)}`, "above");
  }

  // Critical desorption pressure (undersaturated coal/shale).
  if (Number.isFinite(pcd) && pcd <= pMax) {
    ctx.strokeStyle = hexAlpha(theme.accent2, 0.7);
    ctx.setLineDash([2, 3]);
    ctx.beginPath(); // horizontal GC line
    ctx.moveTo(X(0), Y(d.gc));
    ctx.lineTo(X(pcd), Y(d.gc));
    ctx.stroke();
    ctx.beginPath(); // vertical to Pcd
    ctx.moveTo(X(pcd), Y(d.gc));
    ctx.lineTo(X(pcd), Y(0));
    ctx.stroke();
    ctx.setLineDash([]);
    marker(ctx, theme, X(pcd), Y(d.gc), theme.accent2);
    label(ctx, theme, X(pcd), Y(d.gc), `Pcd ${round(pcd)} (GC ${round(d.gc)})`, "above");
  }

  // Axis titles + headline.
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.textBaseline = "alphabetic";
  ctx.fillText("pressure (psia)", (padL + w - padR) / 2, h - 4);
  ctx.save();
  ctx.translate(12, (padT + h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("adsorbed gas Gs (scf/ton)", 0, 0);
  ctx.restore();
  ctx.textAlign = "left";
  ctx.font = canvasFont(theme, 9, 600);
  ctx.fillStyle = theme.accent;
  const sat = undersat ? "undersaturated" : "saturated";
  ctx.fillText(`VL ${round(d.vl)} scf/ton · PL ${round(d.pl)} psia · ${sat}`, padL, 14);
}

function marker(
  ctx: CanvasRenderingContext2D,
  _theme: PlotTheme,
  px: number,
  py: number,
  color: string,
): void {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(px, py, 3.5, 0, Math.PI * 2);
  ctx.fill();
}

function label(
  ctx: CanvasRenderingContext2D,
  theme: PlotTheme,
  px: number,
  py: number,
  text: string,
  where: "above" | "below",
): void {
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 9, 500);
  ctx.textBaseline = "alphabetic";
  // Keep the label inside the canvas horizontally.
  const wpx = ctx.measureText(text).width;
  ctx.textAlign = px + wpx + 6 > (ctx.canvas.clientWidth || 460) ? "right" : "left";
  const dx = ctx.textAlign === "right" ? -6 : 6;
  ctx.fillText(text, px + dx, where === "above" ? py - 6 : py + 13);
}

/** A "nice" step (1/2/5 × 10ⁿ) near the requested magnitude for readable axis ticks. */
function niceStep(raw: number): number {
  if (!(raw > 0) || !Number.isFinite(raw)) return 1;
  const pow = Math.pow(10, Math.floor(Math.log10(raw)));
  const f = raw / pow;
  const nice = f < 1.5 ? 1 : f < 3 ? 2 : f < 7 ? 5 : 10;
  return nice * pow;
}

function round(v: number): number {
  return Math.round(v);
}

/** #rrggbb + alpha → rgba(); passes non-hex colors through unchanged (rare theme overrides). */
function hexAlpha(hex: string, alpha: number): string {
  const m = /^#([0-9a-fA-F]{6})$/.exec(hex.trim());
  if (!m) return hex;
  const r = parseInt(m[1].slice(0, 2), 16);
  const g = parseInt(m[1].slice(2, 4), 16);
  const b = parseInt(m[1].slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

function numField(value: string, width = "68px"): HTMLInputElement {
  const i = document.createElement("input");
  i.className = "form-control";
  i.type = "number";
  i.step = "any";
  i.style.width = width;
  i.value = value;
  return i;
}

/** Default porosity curve for an overlay mode: prefer a real sonic / density mnemonic that the
 *  well actually has, else fall back to the canonical name so the selector still shows intent. */
function defaultPorCurve(mode: string, names: string[]): string {
  const wants = mode === "density" ? ["RHOB", "RHOZ", "RHOD"] : ["DT", "DTCO", "DTC", "AC"];
  return wants.find((c) => names.includes(c)) ?? wants[0];
}

export async function buildUnconventionalContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);

  const content = document.createElement("div");
  content.className = "plot-content";

  const split = document.createElement("div");
  split.style.display = "flex";
  split.style.gap = "14px";
  split.style.flexWrap = "wrap";
  split.style.alignItems = "flex-start";
  content.appendChild(split);

  // ---- Left: ΔlogR overlay track ----
  const leftCol = document.createElement("div");
  leftCol.style.flex = "1 1 380px";
  leftCol.style.minWidth = "320px";

  const modeSel = document.createElement("select");
  modeSel.className = "form-control";
  for (const [v, t] of [["sonic", "Sonic"], ["density", "Density"]] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = t;
    modeSel.appendChild(o);
  }
  modeSel.value = initial?.mode === "density" ? "density" : "sonic";

  const resSel = curveSelect(curveNames, initial?.res ?? "RES_DEEP");
  const porSel = curveSelect(curveNames, initial?.por ?? defaultPorCurve(modeSel.value, curveNames));
  const rBaseIn = numField(initial?.rBase ?? "2");
  const dtBaseIn = numField(initial?.dtBase ?? "70");
  const rhobBaseIn = numField(initial?.rhobBase ?? "2.65");

  const overlayBar = document.createElement("div");
  overlayBar.className = "plot-toolbar";
  overlayBar.appendChild(formRow("Overlay", modeSel));
  overlayBar.appendChild(formRow("Resistivity", resSel));
  overlayBar.appendChild(formRow("Porosity", porSel));
  overlayBar.appendChild(formRow("R_base", rBaseIn));
  const dtBaseRow = formRow("DT_base", dtBaseIn);
  const rhobBaseRow = formRow("RHOB_base", rhobBaseIn);
  overlayBar.appendChild(dtBaseRow);
  overlayBar.appendChild(rhobBaseRow);
  overlayBar.appendChild(formRow("Zone", zoneSel.select));
  const overlayCanvas = document.createElement("canvas");
  overlayCanvas.className = "plot-canvas";
  overlayCanvas.style.width = "100%";
  overlayCanvas.style.height = "520px";
  overlayBar.appendChild(buildImageExportButtons(
    () => overlayCanvas,
    "DeltaLogR",
    setStatus,
    undefined,
    undefined,
    undefined,
    () => ({ wellIds: [well.well_id], curves: [resSel.value, porSel.value] }),
  ));
  leftCol.appendChild(overlayBar);
  leftCol.appendChild(overlayCanvas);

  const leftHint = document.createElement("p");
  leftHint.className = "modal-hint";
  leftHint.textContent =
    "Passey ΔlogR: resistivity (log) and the baselined porosity curve overlie in non-source rock and " +
    "separate over organic-rich intervals — the shaded gap is ΔlogR, the input to the TOC (toc_passey) " +
    "module. Pick baselines on a clay-rich, non-source shale. Needs a deep resistivity and a sonic (DT) " +
    "or density (RHOB) curve.";
  leftCol.appendChild(leftHint);

  // ---- Right: Langmuir isotherm ----
  const rightCol = document.createElement("div");
  rightCol.style.flex = "1 1 380px";
  rightCol.style.minWidth = "320px";

  const vlIn = numField(initial?.vl ?? "100");
  const plIn = numField(initial?.pl ?? "1000");
  const presIn = numField(initial?.pres ?? "3000");
  const gcIn = numField(initial?.gc ?? "0");

  const langBar = document.createElement("div");
  langBar.className = "plot-toolbar";
  langBar.appendChild(formRow("VL", vlIn));
  langBar.appendChild(formRow("PL", plIn));
  langBar.appendChild(formRow("Pres", presIn));
  langBar.appendChild(formRow("GC", gcIn));
  const langCanvas = document.createElement("canvas");
  langCanvas.className = "plot-canvas";
  langCanvas.style.width = "100%";
  langCanvas.style.height = "520px";
  langBar.appendChild(buildImageExportButtons(
    () => langCanvas,
    "Langmuir",
    setStatus,
    undefined,
    undefined,
    undefined,
    () => ({ wellIds: [], curves: [] }),
  ));
  rightCol.appendChild(langBar);
  rightCol.appendChild(langCanvas);

  const rightHint = document.createElement("p");
  rightHint.className = "modal-hint";
  rightHint.textContent =
    "Langmuir isotherm Gs = VL·P/(PL+P) (scf/ton): storage capacity vs pressure with the VL ceiling and " +
    "the reservoir-pressure point. Enter an in-situ gas content GC (< VL) for undersaturated coal/shale to " +
    "mark the critical desorption pressure Pcd = PL·GC/(VL−GC) — below Pcd gas begins to desorb. Matches the " +
    "adsorbed term of the gip module.";
  rightCol.appendChild(rightHint);

  split.appendChild(leftCol);
  split.appendChild(rightCol);

  // ---- State + rendering ----
  let overlayRes = new Float32Array(0);
  let overlayPor = new Float32Array(0);
  let overlayDepth = new Float32Array(0);

  const syncBaseVisibility = () => {
    // Show only the baseline the active overlay uses: DT_base for sonic, RHOB_base for density.
    const density = modeSel.value === "density";
    dtBaseRow.style.display = density ? "none" : "";
    rhobBaseRow.style.display = density ? "" : "none";
  };
  syncBaseVisibility();

  const redrawOverlay = () => {
    const theme = readTheme(document.documentElement);
    drawOverlay(overlayCanvas, theme, {
      depth: overlayDepth,
      res: overlayRes,
      por: overlayPor,
      mode: modeSel.value,
      rBase: parseFloat(rBaseIn.value) || 2,
      dtBase: parseFloat(dtBaseIn.value) || 70,
      rhobBase: parseFloat(rhobBaseIn.value) || 2.65,
      resName: resSel.value,
      porName: porSel.value,
    });
  };

  const redrawLangmuir = () => {
    const theme = readTheme(document.documentElement);
    drawLangmuir(langCanvas, theme, {
      vl: parseFloat(vlIn.value),
      pl: parseFloat(plIn.value),
      pres: parseFloat(presIn.value),
      gc: parseFloat(gcIn.value) || 0,
    });
  };

  // Monotonic token so a slow curve load can't overwrite a newer one (fast well/zone switching).
  let reloadGen = 0;
  const reloadOverlay = async () => {
    const gen = ++reloadGen;
    const zone = zoneSel.current();
    try {
      const series = await getCurveData(
        well.well_id,
        [resSel.value, porSel.value],
        zone.depthMin,
        zone.depthMax,
      );
      if (gen !== reloadGen) return;
      const byName = new Map(series.map((sd) => [sd.curve_name.toUpperCase(), sd]));
      const resS = byName.get(resSel.value.toUpperCase());
      const porS = byName.get(porSel.value.toUpperCase());
      overlayRes = resS?.value ?? new Float32Array(0);
      overlayPor = porS?.value ?? new Float32Array(0);
      overlayDepth = resS?.depth ?? porS?.depth ?? new Float32Array(0);
    } catch (err) {
      if (gen !== reloadGen) return;
      setStatus(`ΔlogR overlay load failed: ${err}`);
      overlayRes = overlayPor = overlayDepth = new Float32Array(0);
    }
    redrawOverlay();
  };

  modeSel.addEventListener("change", () => {
    syncBaseVisibility();
    porSel.value = defaultPorCurve(modeSel.value, curveNames);
    void reloadOverlay();
  });
  for (const sel of [resSel, porSel, zoneSel.select]) {
    sel.addEventListener("change", () => void reloadOverlay());
  }
  for (const inp of [rBaseIn, dtBaseIn, rhobBaseIn]) {
    inp.addEventListener("input", redrawOverlay); // baselines are client-side — no refetch
  }
  for (const inp of [vlIn, plIn, presIn, gcIn]) {
    inp.addEventListener("input", redrawLangmuir);
  }

  const detachOverlayResize = attachResizeRedraw(overlayCanvas, redrawOverlay);
  const detachLangResize = attachResizeRedraw(langCanvas, redrawLangmuir);
  const unsubTheme = appState.themeVersion.subscribe(() => {
    redrawOverlay();
    redrawLangmuir();
  });

  // Refetch the overlay when computed curves change (a module run, import, undo) so it never
  // shows stale data; the isotherm is parametric and needs no refetch.
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    void reloadOverlay();
  });

  await reloadOverlay();
  redrawLangmuir();

  return {
    el: content,
    dispose: () => {
      unsubTheme();
      unsubData();
      detachOverlayResize();
      detachLangResize();
      zoneSel.dispose();
    },
    getState: () => ({
      mode: modeSel.value,
      res: resSel.value,
      por: porSel.value,
      rBase: rBaseIn.value,
      dtBase: dtBaseIn.value,
      rhobBase: rhobBaseIn.value,
      vl: vlIn.value,
      pl: plIn.value,
      pres: presIn.value,
      gc: gcIn.value,
      zone: zoneSel.select.value,
    }),
  };
}
