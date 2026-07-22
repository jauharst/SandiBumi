import { getCurveData, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow, openModal } from "./modal";
import {
  attachResizeRedraw,
  attachZoomPan,
  colorRampEx,
  fitCanvasBackingStore,
  PlotCanvas,
  canvasFont,
  readTheme,
  type ColormapName,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildZoneSelect,
  curveSelect,
  loadCurveNames,
  loadPlotProps,
  nearestDepthIndex,
  pickRow,
  savePlotProps,
  trySelect,
  type PlotContent,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

/** Persisted Pickett v2 display settings (plotprops doc "pickett"). Axis ranges replace the
 *  old hard-coded 0.1–1000 / 0.01–1 defaults; Z-color paints each sample by a chosen log. */
interface PickettProps {
  rtMin: number;
  rtMax: number;
  phiMin: number;
  phiMax: number;
  pointSize: number;
  /** Curve to color points by; "" = single theme color. */
  zCurve: string;
  colormap: ColormapName;
  zLog: boolean;
}

const PICKETT_DEFAULTS: PickettProps = {
  rtMin: 0.1,
  rtMax: 1000,
  phiMin: 0.01,
  phiMax: 1,
  pointSize: 1.8,
  zCurve: "",
  colormap: "rainbow",
  zLog: false,
};

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
  style?: { rtMin?: number; rtMax?: number; phiMin?: number; phiMax?: number; pointSize?: number; colors?: string[] },
): PlotCanvas {
  fitCanvasBackingStore(canvas);
  const plot = new PlotCanvas(
    canvas,
    { label: "RT (ohmm)", min: view ? view.xMin : style?.rtMin ?? 0.1, max: view ? view.xMax : style?.rtMax ?? 1000, log: true, invert: false },
    { label: "PHIE (v/v)", min: view ? view.yMin : style?.phiMin ?? 0.01, max: view ? view.yMax : style?.phiMax ?? 1, log: true, invert: false },
  );
  plot.drawFrame();
  plot.drawScatter(rt, phi, style?.colors, style?.pointSize ?? 1.8);

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
    ctx.font = canvasFont(plot.theme, 10);
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

/** Pickett plot dialog (v2): log-log RT vs PHIE. Click two points along the water-bearing
 *  trend to fit the Sw=1 line (M, Rw), OR type M / Rw directly and the lines follow. N, M and
 *  Rw are all editable in the toolbar. A Properties dialog (⚙ / right-click) sets the axis
 *  ranges, point size, and optional Z-coloring by a chosen log, persisted via plotprops. */
export async function buildPickettContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);
  const props: PickettProps = { ...PICKETT_DEFAULTS, ...(await loadPlotProps<PickettProps>("pickett")) };

  const numField = (value: string, placeholder = ""): HTMLInputElement => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = "any";
    i.style.width = "72px";
    i.placeholder = placeholder;
    i.value = value;
    return i;
  };

  const content = document.createElement("div");
  content.className = "plot-content";
  const rtSel = curveSelect(curveNames, initial?.rt ?? "RES_DEEP");
  const phiSel = curveSelect(curveNames, initial?.phi ?? "PHIE");
  const nIn = numField(initial?.n ?? "2");
  // Manual M / Rw: blank = derive from the two picks; typed = the water line follows them.
  const mIn = numField(initial?.m ?? "", "pick");
  const rwIn = numField(initial?.rw ?? "", "pick");

  const propsBtn = document.createElement("button");
  propsBtn.className = "form-control";
  propsBtn.textContent = "⚙";
  propsBtn.title = "Pickett properties — axes, point size, Z-color (or right-click the plot)";
  propsBtn.addEventListener("click", () => openProps());

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("RT", rtSel));
  selRow.appendChild(formRow("Porosity", phiSel));
  selRow.appendChild(formRow("N", nIn));
  selRow.appendChild(formRow("M", mIn));
  selRow.appendChild(formRow("Rw", rwIn));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(propsBtn);
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
    "Click TWO points along the water-bearing (lowest-RT) trend to fit the Sw=1 line — or type M and Rw " +
    "directly. Ctrl+wheel = zoom, drag = pan, double-click = reset zoom, right-click = properties. " +
    "Needs a computed porosity curve (run a Porosity module first).";
  content.appendChild(hint);

  const tc = readTheme(document.documentElement);
  const pickM = pickRow("M (slope)", tc.accent, "M", well, zoneSel.current, setStatus);
  const pickRw = pickRow("Rw @ FT", tc.accent2, "RW", well, zoneSel.current, setStatus);
  content.appendChild(pickM.row);
  content.appendChild(pickRw.row);

  let rt = new Float32Array(0);
  let phi = new Float32Array(0);
  let depths = new Float32Array(0);
  let picks: [number, number][] = [];
  let colors: string[] | undefined;
  let plot: PlotCanvas | null = null;
  let hoverIdx = -1;
  const viewRef: ViewportRef = { current: null };

  /** The effective water line: typed M+Rw win; otherwise none until two points are picked
   *  (a completed pick writes its fit into the M/Rw fields, so both paths share one source). */
  const currentLine = (): { m: number; rw: number } | null => {
    const m = parseFloat(mIn.value);
    const rw = parseFloat(rwIn.value);
    if (Number.isFinite(m) && Number.isFinite(rw) && m > 0 && rw > 0) return { m, rw };
    return null;
  };

  const computeColors = (z: Float32Array | undefined): string[] | undefined => {
    if (!z || z.length === 0) return undefined;
    let lo = Infinity;
    let hi = -Infinity;
    for (const v of z) {
      if (!Number.isFinite(v) || (props.zLog && v <= 0)) continue;
      if (v < lo) lo = v;
      if (v > hi) hi = v;
    }
    if (!Number.isFinite(lo) || lo === hi) return undefined;
    return colorRampEx(z, lo, hi, props.colormap, props.zLog);
  };

  const redraw = () => {
    const n = parseFloat(nIn.value) || 2;
    plot = drawPickett(canvas, rt, phi, currentLine(), n, picks, hoverIdx, viewRef.current, {
      rtMin: props.rtMin,
      rtMax: props.rtMax,
      phiMin: props.phiMin,
      phiMax: props.phiMax,
      pointSize: props.pointSize,
      colors,
    });
  };

  // Monotonic token so a slow curve/zone load that resolves after a newer one (fast
  // switching) can't overwrite the newer data. `preserveView` keeps the zoom/pan AND the
  // user's M/Rw line on a data refresh (module run); a user-initiated curve/zone change
  // re-fits from scratch and clears the picks + typed line.
  let reloadGen = 0;
  let resetPending = false;
  const reload = async (preserveView = false) => {
    const gen = ++reloadGen;
    if (!preserveView) resetPending = true;
    const zone = zoneSel.current();
    const names = props.zCurve ? [rtSel.value, phiSel.value, props.zCurve] : [rtSel.value, phiSel.value];
    try {
      const series = await getCurveData(well.well_id, names, zone.depthMin, zone.depthMax);
      if (gen !== reloadGen) return; // a newer reload started while we awaited
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      rt = byName.get(rtSel.value.toUpperCase())?.value ?? new Float32Array(0);
      phi = byName.get(phiSel.value.toUpperCase())?.value ?? new Float32Array(0);
      depths = byName.get(rtSel.value.toUpperCase())?.depth ?? new Float32Array(0);
      colors = props.zCurve ? computeColors(byName.get(props.zCurve.toUpperCase())?.value) : undefined;
    } catch (err) {
      if (gen !== reloadGen) return; // superseded — don't clobber newer data with this error
      setStatus(`Pickett data load failed: ${err}`);
      rt = phi = depths = new Float32Array(0);
      colors = undefined;
    }
    hoverIdx = -1; // the old hover index may point at a different sample now
    if (resetPending) {
      resetPending = false;
      picks = [];
      mIn.value = "";
      rwIn.value = "";
      viewRef.current = null; // new data → reset any zoom/pan
    }
    redraw();
  };

  for (const sel of [rtSel, phiSel, zoneSel.select]) {
    sel.addEventListener("change", () => void reload());
  }
  nIn.addEventListener("change", redraw);
  // Typing M or Rw makes the line follow immediately (free user input of line parameters).
  mIn.addEventListener("input", redraw);
  rwIn.addEventListener("input", redraw);

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
      const fit = fitWaterLine(picks[0], picks[1]);
      if (fit) {
        // Feed the fit into the M/Rw fields so picked and typed lines share one source.
        mIn.value = fit.m.toPrecision(4);
        rwIn.value = fit.rw.toPrecision(4);
        pickM.setValue(fit.m);
        pickRw.setValue(fit.rw);
        setStatus(`Water line: M = ${fit.m.toFixed(2)}, Rw = ${fit.rw.toPrecision(3)} ohmm`);
      } else {
        setStatus("Could not fit a water line from those two points — pick points at different porosities.");
      }
    }
    redraw();
  });

  // Right-click opens Properties (matches Histogram/Crossplot v2); left-click stays reserved
  // for water-line picks, so — unlike the other plots — double-click is not overloaded here.
  canvas.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
    openProps();
  });

  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

  // Re-fetch when computed curves change (module/equation run, import, undo) so the
  // Pickett plot never shows stale data; keep the zoom/pan and the M/Rw line.
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    void reload(true);
  });

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

  /** Properties dialog: axis ranges, point size, and Z-color-by-curve. Persisted per plot
   *  kind (plotprops "pickett") like Histogram/Crossplot v2. */
  const openProps = () => {
    const body = document.createElement("div");
    const num = (value: number, w = 72): HTMLInputElement => {
      const i = document.createElement("input");
      i.className = "form-control";
      i.type = "number";
      i.step = "any";
      i.style.width = `${w}px`;
      i.value = String(value);
      return i;
    };
    const inline = (...els: (HTMLElement | string)[]): HTMLElement => {
      const wrap = document.createElement("div");
      wrap.style.display = "flex";
      wrap.style.gap = "8px";
      wrap.style.alignItems = "center";
      wrap.style.flexWrap = "wrap";
      wrap.append(...els);
      return wrap;
    };

    const rtMinI = num(props.rtMin);
    const rtMaxI = num(props.rtMax);
    const phiMinI = num(props.phiMin);
    const phiMaxI = num(props.phiMax);
    const psI = num(props.pointSize, 56);

    const zSel = document.createElement("select");
    zSel.className = "form-control";
    for (const [v, label] of [["", "— None —"] as [string, string], ...curveNames.map((c) => [c, c] as [string, string])]) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      zSel.appendChild(o);
    }
    zSel.value = props.zCurve;

    const cmSel = document.createElement("select");
    cmSel.className = "form-control";
    for (const [v, label] of [["rainbow", "Rainbow"], ["viridis", "Viridis"]] as [string, string][]) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      cmSel.appendChild(o);
    }
    cmSel.value = props.colormap;

    const zLogWrap = document.createElement("label");
    zLogWrap.className = "chk-field";
    const zLogChk = document.createElement("input");
    zLogChk.type = "checkbox";
    zLogChk.checked = props.zLog;
    zLogWrap.append(zLogChk, document.createTextNode("log Z"));

    body.appendChild(formRow("RT axis", inline(rtMinI, "→", rtMaxI)));
    body.appendChild(formRow("PHIE axis", inline(phiMinI, "→", phiMaxI)));
    body.appendChild(formRow("Point size", psI));
    body.appendChild(formRow("Color by", zSel));
    body.appendChild(formRow("Colormap", inline(cmSel, zLogWrap)));

    const applyBtn = document.createElement("button");
    applyBtn.className = "form-run-btn";
    applyBtn.textContent = "Apply";
    const btnRow = document.createElement("div");
    btnRow.className = "form-row";
    btnRow.appendChild(applyBtn);
    body.appendChild(btnRow);

    const close = openModal("Pickett properties", body, 340);
    applyBtn.addEventListener("click", () => {
      const posOr = (v: string, fallback: number) => {
        const p = parseFloat(v);
        return Number.isFinite(p) && p > 0 ? p : fallback;
      };
      props.rtMin = posOr(rtMinI.value, PICKETT_DEFAULTS.rtMin);
      props.rtMax = posOr(rtMaxI.value, PICKETT_DEFAULTS.rtMax);
      props.phiMin = posOr(phiMinI.value, PICKETT_DEFAULTS.phiMin);
      props.phiMax = posOr(phiMaxI.value, PICKETT_DEFAULTS.phiMax);
      props.pointSize = Math.max(0.5, parseFloat(psI.value) || PICKETT_DEFAULTS.pointSize);
      props.zCurve = zSel.value;
      props.colormap = cmSel.value as ColormapName;
      props.zLog = zLogChk.checked;
      savePlotProps("pickett", props);
      viewRef.current = null; // show the new axis ranges (a live zoom would otherwise mask them)
      void reload(true); // refetch (the Z curve may have changed); keeps picks + M/Rw line
      close();
    });
  };

  await reload();
  return {
    el: content,
    dispose: () => {
      unsubHover();
      unsubTheme();
      unsubData();
      detachZoomPan();
      detachResize();
      zoneSel.dispose();
    },
    getState: () => ({
      rt: rtSel.value,
      phi: phiSel.value,
      n: nIn.value,
      m: mIn.value,
      rw: rwIn.value,
      zone: zoneSel.select.value,
    }),
  };
}
