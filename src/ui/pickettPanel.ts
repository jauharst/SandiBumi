import { getCurveData, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";
import {
  attachResizeRedraw,
  attachZoomPan,
  fitCanvasBackingStore,
  PlotCanvas,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import { buildZoneSelect, curveSelect, loadCurveNames, nearestDepthIndex, pickRow, trySelect, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

/** Water line fit from two picked points on the log-log plot:
 *  Sw=1 (Archie): RT = Rw / PHI^m  →  m = -Δlog(RT)/Δlog(PHI),  Rw = RT·PHI^m. */
export function fitWaterLine(
  p1: [number, number],
  p2: [number, number],
): { m: number; rw: number } | null {
  const [rt1, phi1] = p1;
  const [rt2, phi2] = p2;
  if (rt1 <= 0 || rt2 <= 0 || phi1 <= 0 || phi2 <= 0) return null;
  const dLogPhi = Math.log10(phi2) - Math.log10(phi1);
  if (Math.abs(dLogPhi) < 1e-6) return null;
  const m = -(Math.log10(rt2) - Math.log10(rt1)) / dLogPhi;
  const rw = rt1 * Math.pow(phi1, m);
  if (!Number.isFinite(m) || !Number.isFinite(rw) || m <= 0 || rw <= 0) return null;
  return { m, rw };
}

export function drawPickett(
  canvas: HTMLCanvasElement,
  rt: Float32Array,
  phi: Float32Array,
  line: { m: number; rw: number } | null,
  n: number,
  picks: [number, number][],
  hoverIdx = -1,
  view: Viewport | null = null,
): PlotCanvas {
  fitCanvasBackingStore(canvas);
  const plot = new PlotCanvas(
    canvas,
    { label: "RT (ohmm)", min: view ? view.xMin : 0.1, max: view ? view.xMax : 1000, log: true, invert: false },
    { label: "PHIE (v/v)", min: view ? view.yMin : 0.01, max: view ? view.yMax : 1, log: true, invert: false },
  );
  plot.drawFrame();
  plot.drawScatter(rt, phi, undefined, 1.8);

  if (line) {
    // rt(phi) = Rw / (phi^m · sw^n); straight lines in log-log space.
    const lineFor = (sw: number): [number, number][] => {
      const points: [number, number][] = [];
      for (const phiV of [0.01, 1.0]) {
        points.push([line.rw / (Math.pow(phiV, line.m) * Math.pow(sw, n)), phiV]);
      }
      return points;
    };
    plot.drawLine(lineFor(1.0), plot.theme.accent, 2);
    plot.drawLine(lineFor(0.5), plot.theme.accent2, 1.2, [6, 4]);
    plot.drawLine(lineFor(0.25), plot.theme.warn, 1.2, [6, 4]);

    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.font = "500 10px system-ui, sans-serif";
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(`Sw=1 line:  M = ${line.m.toFixed(2)},  Rw = ${line.rw.toPrecision(3)} ohmm  (N = ${n})`, r.x0 + 8, r.y0 + 14);
    ctx.fillText("dashed: Sw = 0.5, 0.25", r.x0 + 8, r.y0 + 27);
    ctx.restore();
  }

  // Picked anchor points.
  for (const [rtV, phiV] of picks) {
    const [px, py] = plot.toPx(rtV, phiV);
    const { ctx } = plot;
    ctx.save();
    ctx.strokeStyle = plot.theme.accent;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  }

  // Synchronized hover: ring the sample at the depth under another view's cursor.
  if (hoverIdx >= 0 && hoverIdx < rt.length) {
    const hr = rt[hoverIdx];
    const hp = phi[hoverIdx];
    if (!Number.isNaN(hr) && !Number.isNaN(hp) && hr > 0 && hp > 0) {
      const [px, py] = plot.toPx(hr, hp);
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

/** Pickett plot dialog: log-log RT vs PHIE. Click two points along the water-bearing
 *  trend to define the Sw=1 line — M (slope) and Rw (intercept at PHI=1) are derived
 *  instantly, iso-Sw lines drawn, and both values can be written to zone parameters. */
export async function buildPickettContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);

  const content = document.createElement("div");
  content.className = "plot-content";
  const rtSel = curveSelect(curveNames, initial?.rt ?? "RES_DEEP");
  const phiSel = curveSelect(curveNames, initial?.phi ?? "PHIE");
  const nIn = document.createElement("input");
  nIn.className = "form-control";
  nIn.type = "number";
  nIn.step = "any";
  nIn.value = initial?.n ?? "2";

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("RT", rtSel));
  selRow.appendChild(formRow("Porosity", phiSel));
  selRow.appendChild(formRow("N", nIn));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(buildImageExportButtons(() => canvas, "Pickett", setStatus));
  content.appendChild(selRow);

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 460;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  hint.textContent =
    "Click TWO points along the water-bearing (lowest-RT) trend to set the Sw=1 water line. " +
    "M and Rw update instantly. Ctrl+wheel = zoom, drag = pan, double-click = reset. " +
    "Needs a computed porosity curve (run a Porosity module first).";
  content.appendChild(hint);

  const pickM = pickRow("M (slope)", "#b5651d", "M", well, zoneSel.current, setStatus);
  const pickRw = pickRow("Rw @ FT", "#5f7350", "RW", well, zoneSel.current, setStatus);
  content.appendChild(pickM.row);
  content.appendChild(pickRw.row);

  let rt = new Float32Array(0);
  let phi = new Float32Array(0);
  let depths = new Float32Array(0);
  let picks: [number, number][] = [];
  let line: { m: number; rw: number } | null = null;
  let plot: PlotCanvas | null = null;
  let hoverIdx = -1;
  const viewRef: ViewportRef = { current: null };

  const redraw = () => {
    const n = parseFloat(nIn.value) || 2;
    plot = drawPickett(canvas, rt, phi, line, n, picks, hoverIdx, viewRef.current);
  };

  const reload = async () => {
    const zone = zoneSel.current();
    try {
      const series = await getCurveData(well.well_id, [rtSel.value, phiSel.value], zone.depthMin, zone.depthMax);
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      rt = byName.get(rtSel.value.toUpperCase())?.value ?? new Float32Array(0);
      phi = byName.get(phiSel.value.toUpperCase())?.value ?? new Float32Array(0);
      depths = byName.get(rtSel.value.toUpperCase())?.depth ?? new Float32Array(0);
    } catch (err) {
      setStatus(`Pickett data load failed: ${err}`);
      rt = phi = depths = new Float32Array(0);
    }
    picks = [];
    line = null;
    hoverIdx = -1;
    viewRef.current = null; // new data → reset any zoom/pan
    redraw();
  };

  for (const sel of [rtSel, phiSel, zoneSel.select]) {
    sel.addEventListener("change", () => void reload());
  }
  nIn.addEventListener("change", redraw);

  // Track drag so a pan doesn't also drop a water-line anchor. Logical (CSS) pixels.
  let downXY: [number, number] | null = null;
  let movedSinceDown = false;
  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    const rect = canvas.getBoundingClientRect();
    downXY = [e.clientX - rect.left, e.clientY - rect.top];
    movedSinceDown = false;
  });
  canvas.addEventListener("mousemove", (e) => {
    if (!downXY) return;
    const rect = canvas.getBoundingClientRect();
    if (Math.hypot(e.clientX - rect.left - downXY[0], e.clientY - rect.top - downXY[1]) > 4) movedSinceDown = true;
  });

  canvas.addEventListener("click", (e) => {
    downXY = null;
    if (!plot || movedSinceDown) return; // tail of a pan, not an anchor pick
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    if (!plot.inPlot(px, py)) return;
    const [rtV, phiV] = plot.toData(px, py);

    if (picks.length >= 2) picks = [];
    picks.push([rtV, phiV]);
    if (picks.length === 2) {
      line = fitWaterLine(picks[0], picks[1]);
      if (line) {
        pickM.setValue(line.m);
        pickRw.setValue(line.rw);
        setStatus(`Water line: M = ${line.m.toFixed(2)}, Rw = ${line.rw.toPrecision(3)} ohmm`);
      } else {
        setStatus("Could not fit a water line from those two points — pick points at different porosities.");
      }
    }
    redraw();
  });

  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

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
  return {
    el: content,
    dispose: () => {
      unsubHover();
      unsubTheme();
      detachZoomPan();
      detachResize();
      zoneSel.dispose();
    },
    getState: () => ({ rt: rtSel.value, phi: phiSel.value, n: nIn.value, zone: zoneSel.select.value }),
  };
}
