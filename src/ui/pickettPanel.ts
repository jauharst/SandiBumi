import { getCurveData, plotBindingSnapshot, type ResolvedPlotCurve, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow, openModal } from "./modal";
import {
  attachKeyboardPanZoom,
  attachResizeRedraw,
  attachScatterTooltip,
  attachZoomPan,
  colorRampEx,
  fitCanvasBackingStore,
  fmtValue,
  makeCanvasAccessible,
  PlotCanvas,
  canvasFont,
  readTheme,
  type ColormapName,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildPlotTemplateBar,
  buildZoneSelect,
  contextZoneWindow,
  curveSelect,
  describeContextOutcome,
  fetchContextLayers,
  loadCurveNames,
  loadPlotProps,
  nearestDepthIndex,
  pickRow,
  plotWriteAxis,
  plotWriteSelection,
  savePlotProps,
  trySelect,
  type PlotContent,
  type PlotWriteSource,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import { buildWellScope } from "./wellScope";
import { renderPlotToSvg } from "./svgExport";
import { renderPlotToPdf, type PlotPdf } from "./pdfExport";
import { reconcileDepthChannels } from "./plotTypes";

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
  // RT 0.2–2000 per the senior audit (AUDIT-2026-07-20, "Pickett plot defaults"): 0.1–1000
  // clipped high-resistivity pay in the field data it was checked against. Saved props win.
  rtMin: 0.2,
  rtMax: 2000,
  phiMin: 0.01,
  phiMax: 1,
  pointSize: 1.8,
  zCurve: "",
  colormap: "rainbow",
  zLog: false,
};

/** Fills defaults and sanitizes saved/template-supplied props (a template can carry
 *  anything, so every field is guarded — same policy as normalizeCrossplotOptions). */
export function sanitizePickettProps(raw: Partial<PickettProps>): PickettProps {
  const p: PickettProps = { ...PICKETT_DEFAULTS, ...raw };
  const pos = (v: unknown, fb: number): number =>
    typeof v === "number" && Number.isFinite(v) && v > 0 ? v : fb;
  p.rtMin = pos(p.rtMin, PICKETT_DEFAULTS.rtMin);
  p.rtMax = pos(p.rtMax, PICKETT_DEFAULTS.rtMax);
  p.phiMin = pos(p.phiMin, PICKETT_DEFAULTS.phiMin);
  p.phiMax = pos(p.phiMax, PICKETT_DEFAULTS.phiMax);
  p.pointSize = Math.max(0.5, Math.min(8, pos(p.pointSize, PICKETT_DEFAULTS.pointSize)));
  p.zCurve = typeof p.zCurve === "string" ? p.zCurve : "";
  if (p.colormap !== "viridis") p.colormap = "rainbow";
  p.zLog = !!p.zLog;
  return p;
}

/** One extra well's cloud drawn faded behind the active well — display-only: the fitted
 *  product line, picks, brushing and tooltips stay the ACTIVE well's. */
export interface PickettContextLayer {
  name: string;
  rt: Float32Array;
  phi: Float32Array;
  color: string;
}

export interface PickettContext {
  /** The active well's name, for the legend's first row. */
  activeName: string;
  layers: PickettContextLayer[];
}

/** Water-line fit from two picked points on the log-log plot.
 *  At Sw=1 the intercept is a·Rw: RT = (a·Rw)/PHI^m. The plot cannot identify
 *  a and Rw separately without one independently supplied, provenance-bearing value. */
export function fitWaterLine(
  p1: [number, number],
  p2: [number, number],
): { m: number; aRw: number } | null {
  const [rt1, phi1] = p1;
  const [rt2, phi2] = p2;
  if (rt1 <= 0 || rt2 <= 0 || phi1 <= 0 || phi2 <= 0) return null;
  const dLogPhi = Math.log10(phi2) - Math.log10(phi1);
  if (Math.abs(dLogPhi) < 1e-6) return null;
  const m = -(Math.log10(rt2) - Math.log10(rt1)) / dLogPhi;
  const aRw = rt1 * Math.pow(phi1, m);
  if (!Number.isFinite(m) || !Number.isFinite(aRw) || m <= 0 || aRw <= 0) return null;
  return { m, aRw };
}

export function drawPickett(
  canvas: HTMLCanvasElement,
  rt: Float32Array,
  phi: Float32Array,
  line: { m: number; aRw: number } | null,
  picks: [number, number][],
  hoverIdx = -1,
  view: Viewport | null = null,
  style?: { rtMin?: number; rtMax?: number; phiMin?: number; phiMax?: number; pointSize?: number; colors?: string[] },
  context: PickettContext | null = null,
): PlotCanvas {
  fitCanvasBackingStore(canvas);
  const plot = new PlotCanvas(
    canvas,
    { label: "RT (ohmm)", min: view ? view.xMin : style?.rtMin ?? PICKETT_DEFAULTS.rtMin, max: view ? view.xMax : style?.rtMax ?? PICKETT_DEFAULTS.rtMax, log: true, invert: false },
    { label: "PHIE (v/v)", min: view ? view.yMin : style?.phiMin ?? PICKETT_DEFAULTS.phiMin, max: view ? view.yMax : style?.phiMax ?? PICKETT_DEFAULTS.phiMax, log: true, invert: false },
  );
  plot.drawFrame();

  // Context wells first, faded, so the active well's cloud reads on top of them.
  const hasCtx = !!context && context.layers.length > 0;
  if (hasCtx) {
    const { ctx } = plot;
    ctx.save();
    ctx.globalAlpha = 0.4;
    for (const layer of context!.layers) {
      plot.drawScatter(layer.rt, layer.phi, layer.color, style?.pointSize ?? 1.8);
    }
    ctx.restore();
  }
  plot.drawScatter(rt, phi, style?.colors, style?.pointSize ?? 1.8);

  if (line) {
    // The fitted Sw=1 trend needs only m and the identifiable product a·Rw. Saturation
    // guides remain absent until a, m, n and Rw are all supplied with provenance.
    const phiLo = Math.min(plot.y.min, plot.y.max);
    const phiHi = Math.max(plot.y.min, plot.y.max);
    const fittedLine: [number, number][] = [phiLo, phiHi].map((phiV) => [
      line.aRw / Math.pow(phiV, line.m),
      phiV,
    ]);
    plot.drawLine(fittedLine, plot.theme.accent, 2);

    const { ctx } = plot;
    const r = plot.plotRect;
    ctx.save();
    ctx.font = canvasFont(plot.theme, 10);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(`Fitted line: M = ${line.m.toFixed(2)}, a·Rw = ${line.aRw.toPrecision(3)} ohmm`, r.x0 + 8, r.y0 + 14);
    ctx.fillText(`a and Rw are not separately identified${hasCtx ? " — line = ACTIVE well's fit" : ""}`, r.x0 + 8, r.y0 + 27);
    ctx.restore();
  }

  // Well legend for the multi-well overlay (top-right; the water-line readout owns
  // top-left). The footer states the contract on the plot itself.
  if (hasCtx) {
    const { ctx } = plot;
    const r = plot.plotRect;
    const MAX_ROWS = 10;
    const trunc = (s: string) => (s.length > 18 ? `${s.slice(0, 17)}…` : s);
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    const rowH = 15;
    const boxW = 150;
    const boxX = r.x0 + r.w - boxW - 8;
    let boxY = r.y0 + 8;
    ctx.font = canvasFont(plot.theme, 10, 600);
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText("Wells", boxX, boxY + 9);
    boxY += rowH;
    ctx.font = canvasFont(plot.theme, 10);
    const row = (color: string | null, label: string) => {
      if (color) {
        ctx.fillStyle = color;
        ctx.fillRect(boxX, boxY + 1, 11, 11);
        ctx.strokeStyle = plot.theme.text;
        ctx.lineWidth = 0.5;
        ctx.strokeRect(boxX, boxY + 1, 11, 11);
      }
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(label, boxX + 16, boxY + 10);
      boxY += rowH;
    };
    // The active swatch only means something when the cloud is one color (no Z coloring).
    row(style?.colors ? null : plot.theme.accent, `${trunc(context!.activeName)} (active${style?.colors ? ", by Z" : ""})`);
    const layers = context!.layers;
    for (const layer of layers.slice(0, MAX_ROWS)) row(layer.color, trunc(layer.name));
    if (layers.length > MAX_ROWS) {
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(`+${layers.length - MAX_ROWS} more`, boxX + 16, boxY + 10);
      boxY += rowH;
    }
    ctx.font = canvasFont(plot.theme, 9);
    ctx.fillText("context is display-only", boxX, boxY + 9);
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
  const plotId = initial?.plotId ?? crypto.randomUUID();
  const props: PickettProps = sanitizePickettProps(await loadPlotProps<PickettProps>("pickett"));

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
  // Manual M / a·Rw: blank = derive from the two picks; typed = the fitted line follows them.
  const mIn = numField(initial?.m ?? "", "pick");
  const aRwIn = numField(initial?.aRw ?? "", "pick");

  const propsBtn = document.createElement("button");
  propsBtn.className = "form-control";
  propsBtn.textContent = "⚙";
  propsBtn.title = "Pickett properties — axes, point size, Z-color (or right-click the plot)";
  propsBtn.addEventListener("click", () => openProps());

  // --- Multi-well scope: extra wells drawn as faded clouds BEHIND the active well's.
  // The fitted product line, picks, brushing and tooltips stay bound to the ACTIVE well.
  const scope = await buildWellScope({
    includeActive: true,
    defaultMode: "active",
    initial: initial?.wells,
    onChange: () => {
      updateScopeUi();
      void reloadContext();
    },
  });
  const scopeBtn = document.createElement("button");
  scopeBtn.className = "plot-export-btn";
  scopeBtn.title = "Overlay more wells' clouds behind the active well — the fitted line stays the active well's";
  const scopeRow = document.createElement("div");
  scopeRow.style.display = "none";
  const scopeStaticHint = document.createElement("p");
  scopeStaticHint.className = "modal-hint";
  scopeStaticHint.textContent =
    "Context wells are display-only: the fitted M and a·Rw line, water-line picks, brushing and tooltips all belong to the " +
    "active well. a and Rw are not separately identified, and saturation guides need independently sourced parameters. " +
    "Zone/top windows are resolved per well by NAME (a well without that zone or top is skipped).";
  const scopeInfo = document.createElement("p");
  scopeInfo.className = "modal-hint";
  scopeRow.append(scope.el, scopeStaticHint, scopeInfo);
  scopeBtn.addEventListener("click", () => {
    scopeRow.style.display = scopeRow.style.display === "none" ? "" : "none";
  });
  const updateScopeUi = () => {
    scopeBtn.textContent = `Wells: ${scope.describe()}`;
    scopeInfo.textContent = ctxInfo;
    scopeInfo.style.display = ctxInfo ? "" : "none";
  };

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("RT", rtSel));
  selRow.appendChild(formRow("Porosity", phiSel));
  selRow.appendChild(formRow("M", mIn));
  selRow.appendChild(formRow("a·Rw", aRwIn));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(scopeBtn);
  selRow.appendChild(propsBtn);
  selRow.appendChild(
    buildPlotTemplateBar<PickettProps>(
      "pickett",
      "Pickett",
      () => ({ ...props }),
      (t) => {
        Object.assign(props, sanitizePickettProps({ ...props, ...t }));
        savePlotProps("pickett", props);
        viewRef.current = null; // show the template's axis ranges (a live zoom would mask them)
        void reload(true); // refetch (the Z curve may have changed); keeps picks + fitted product line
      },
      setStatus,
    ),
  );
  selRow.appendChild(buildImageExportButtons(() => canvas, "Pickett", setStatus, () => getSvg(), () => getPdf()));
  content.appendChild(selRow);
  content.appendChild(scopeRow);

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 460;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  hint.textContent =
    "Click TWO points along the water-bearing (lowest-RT) trend to fit M and the identifiable product a·Rw — or type those values " +
    "directly. The fit does not identify a or Rw separately. Ctrl+wheel = zoom, drag = pan, double-click = reset zoom, right-click = properties. " +
    "Needs a computed porosity curve (run a Porosity module first).";
  content.appendChild(hint);

  const tc = readTheme(document.documentElement);
  const pickM = pickRow("M (slope)", tc.accent, "M", well, zoneSel.current, setStatus, () => pickettWriteSource());
  const pickARw = pickRow("a·Rw (intercept)", tc.accent2, "A_RW", well, zoneSel.current, setStatus, () => pickettWriteSource());
  content.appendChild(pickM.row);
  content.appendChild(pickARw.row);

  let rt = new Float32Array(0);
  let phi = new Float32Array(0);
  let depths = new Float32Array(0);
  let picks: [number, number][] = [];
  let colors: string[] | undefined;
  let plot: PlotCanvas | null = null;
  let lastFit: { m: number; aRw: number } | null = null;
  let hoverIdx = -1;
  // Linked-brush consumer: samples brushed in the crossplot (same well, same backend depth
  // grid) are ringed here so a selection made in one plot is visible in the other.
  let brushSet: Set<number> | null = null;
  const viewRef: ViewportRef = { current: null };

  const resolvedBinding = (curveName: string): ResolvedPlotCurve | null =>
    plotBindingSnapshot([well.well_id], [curveName])
      .find((binding) => binding.intent.semantic_request.toUpperCase() === curveName.toUpperCase())
      ?.resolved[0] ?? null;

  async function pickettWriteSource(): Promise<PlotWriteSource> {
    if (!plot) throw new Error("plot-derived write requires a rendered viewport");
    const zone = zoneSel.current();
    return {
      plot_id: plotId,
      plot_type: "pickett",
      x_axis: plotWriteAxis("resistivity", resolvedBinding(rtSel.value)),
      y_axis: plotWriteAxis("porosity", resolvedBinding(phiSel.value)),
      z_axis: props.zCurve ? plotWriteAxis("colour", resolvedBinding(props.zCurve)) : null,
      viewport: {
        x_min: plot.x.min,
        x_max: plot.x.max,
        y_min: plot.y.min,
        y_max: plot.y.max,
        x_log: plot.x.log,
        y_log: plot.y.log,
      },
      selection: await plotWriteSelection(well.well_id),
      interval: { low: zone.depthMin, high: zone.depthMax, closure: "[lo,hi)" },
      method: lastFit ? "two_point_pickett_fit" : "manual_pickett_value",
      fit_record: lastFit
        ? { model: "two_point_pickett_water_line", m: lastFit.m, a_rw: lastFit.aRw }
        : null,
    };
  }

  /** The effective fitted line: typed M+a·Rw win; otherwise none until two points are picked. */
  const currentLine = (): { m: number; aRw: number } | null => {
    const m = parseFloat(mIn.value);
    const aRw = parseFloat(aRwIn.value);
    if (Number.isFinite(m) && Number.isFinite(aRw) && m > 0 && aRw > 0) return { m, aRw };
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

  // --- Context-well data (multi-well overlay) — same budget rule as crossplot/histogram.
  const MAX_CONTEXT_POINTS = 60_000;
  let ctxLayers: PickettContextLayer[] = [];
  let ctxInfo = "";
  let ctxGen = 0;
  const pickettContext = (): PickettContext | null =>
    ctxLayers.length ? { activeName: well.well_name, layers: ctxLayers } : null;

  /** Fetches the scoped context wells' RT/porosity through the shared plotCommon machinery
   *  (per-well zone/top-by-name windows, point budget, cancellation). Scope = just the
   *  active well → clears the overlay: byte-identical single-well behaviour. */
  const reloadContext = async () => {
    const gen = ++ctxGen;
    const ids = scope.getWellIds().filter((id) => id !== well.well_id);
    if (ids.length === 0) {
      const had = ctxLayers.length > 0;
      ctxLayers = [];
      ctxInfo = "";
      updateScopeUi();
      if (had) redraw();
      return;
    }
    setStatus(`Pickett: loading ${ids.length} context well${ids.length === 1 ? "" : "s"}…`);
    const outcome = await fetchContextLayers({
      ids,
      names: scope.namesFor(ids),
      curves: [rtSel.value, phiSel.value],
      windowFor: (id) => contextZoneWindow(zoneSel, id),
      budget: MAX_CONTEXT_POINTS,
      isStale: () => gen !== ctxGen,
    });
    if (!outcome) return; // superseded by a newer call (or dispose)
    ctxLayers = outcome.layers.map((l) => ({
      name: l.name,
      color: l.color,
      rt: l.series.get(rtSel.value.toUpperCase())!,
      phi: l.series.get(phiSel.value.toUpperCase())!,
    }));
    ctxInfo = describeContextOutcome(outcome);
    updateScopeUi();
    setStatus(`Pickett ${ctxInfo.toLowerCase()}`);
    redraw();
  };
  updateScopeUi();

  const redraw = () => {
    canvas.setAttribute("aria-label", `Pickett plot: ${rtSel.value} versus ${phiSel.value}`); // a11y label
    plot = drawPickett(canvas, rt, phi, currentLine(), picks, hoverIdx, viewRef.current, {
      rtMin: props.rtMin,
      rtMax: props.rtMax,
      phiMin: props.phiMin,
      phiMax: props.phiMax,
      pointSize: props.pointSize,
      colors,
    }, pickettContext());
    // Ring the samples brushed in the crossplot. Depths come off the same backend grid, so an
    // exact Set membership test aligns them; clipped to the plot and skipping log-invalid points.
    if (plot && brushSet && brushSet.size && depths.length === rt.length) {
      const { ctx } = plot;
      const rp = plot.plotRect;
      ctx.save();
      ctx.beginPath();
      ctx.rect(rp.x0, rp.y0, rp.w, rp.h);
      ctx.clip();
      ctx.strokeStyle = plot.theme.accent2;
      ctx.lineWidth = 1.5;
      const rad = Math.max(3, props.pointSize + 1.6);
      for (let i = 0; i < depths.length; i++) {
        if (!brushSet.has(depths[i])) continue;
        const rv = rt[i];
        const pv = phi[i];
        if (!(rv > 0) || !(pv > 0)) continue; // both axes are log
        const [px, py] = plot.toPx(rv, pv);
        ctx.beginPath();
        ctx.arc(px, py, rad, 0, Math.PI * 2);
        ctx.stroke();
      }
      ctx.restore();
    }
  };

  // Vector export: re-run the same static draw (no hover ring, no brush) into a recording
  // context sized to the live plot, so the SVG matches what's on screen.
  // The static draw shared by the two vector-export paths (no hover ring, no brush).
  const drawStatic = (c: HTMLCanvasElement) =>
    drawPickett(c, rt, phi, currentLine(), picks, -1, viewRef.current, {
      rtMin: props.rtMin,
      rtMax: props.rtMax,
      phiMin: props.phiMin,
      phiMax: props.phiMax,
      pointSize: props.pointSize,
      colors,
    }, pickettContext());
  const getSvg = (): string | null => (plot ? renderPlotToSvg(plot.width, plot.height, drawStatic) : null);
  const getPdf = (): PlotPdf | null => (plot ? renderPlotToPdf(plot.width, plot.height, drawStatic) : null);

  // Monotonic token so a slow curve/zone load that resolves after a newer one (fast
  // switching) can't overwrite the newer data. `preserveView` keeps the zoom/pan AND the
  // user's M/a·Rw line on a data refresh (module run); a user-initiated curve/zone change
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
      const required = names.map((name) => byName.get(name.toUpperCase()));
      if (required.some((item) => !item)) throw new Error("one or more required plot curves are absent");
      const reconciled = reconcileDepthChannels(required.map((item) => ({
        depth: item!.depth,
        values: item!.value,
      })));
      rt = reconciled.channels[0];
      phi = reconciled.channels[1];
      depths = reconciled.depth;
      colors = props.zCurve ? computeColors(reconciled.channels[2]) : undefined;
      if (reconciled.mode === "decimated_to_coarsest") {
        setStatus(`Pickett depth inputs decimated to the coarsest exact step; factors ${reconciled.decimationFactors.join("/")} · interval ${reconciled.intervalClosure}`);
      }
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
      lastFit = null;
      mIn.value = "";
      aRwIn.value = "";
      viewRef.current = null; // new data → reset any zoom/pan
    }
    redraw();
  };

  for (const sel of [rtSel, phiSel, zoneSel.select]) {
    sel.addEventListener("change", () => {
      void reload();
      void reloadContext(); // context wells share the RT/porosity curves and the zone window
    });
  }
  // Typing M or a·Rw makes the fitted line follow immediately.
  mIn.addEventListener("input", () => {
    lastFit = null;
    redraw();
  });
  aRwIn.addEventListener("input", () => {
    lastFit = null;
    redraw();
  });

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
        lastFit = fit;
        // Feed the identifiable fit into the M/a·Rw fields so both paths share one source.
        mIn.value = fit.m.toPrecision(4);
        aRwIn.value = fit.aRw.toPrecision(4);
        pickM.setValue(fit.m);
        pickARw.setValue(fit.aRw);
        setStatus(`Water line: M = ${fit.m.toFixed(2)}, a·Rw = ${fit.aRw.toPrecision(3)} ohmm; a and Rw are not separately identified`);
      } else {
        setStatus("Could not fit a water line from those two points — pick points at different porosities.");
      }
    }
    redraw();
  });

  // Right-click belongs to the pane context menu (Properties… is its first entry), not to
  // the canvas. Left-click stays reserved for water-line picks, so — unlike the other
  // plots — double-click is not overloaded here either; the ⚙ toolbar button is the
  // direct route to Properties.

  makeCanvasAccessible(canvas, `Pickett plot: ${rtSel.value} versus ${phiSel.value}`);
  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw });
  const detachKeys = attachKeyboardPanZoom({ canvas, getPlot: () => plot, view: viewRef, redraw });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

  // Linked brushing: mirror the crossplot's selection (this well only) as rings.
  const unsubBrush = appState.brushedDepths.subscribe((b) => {
    const next = b && b.wellId === well.well_id ? b.depths : null;
    if (next === brushSet) return;
    brushSet = next;
    redraw();
  });

  // Re-fetch when computed curves change (module/equation run, import, undo) so the
  // Pickett plot never shows stale data; keep the zoom/pan and the M/a·Rw line.
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    void reload(true);
    void reloadContext(); // a module run may have rewritten the context wells' curves too
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
      void reload(true); // refetch (the Z curve may have changed); keeps picks + fitted product line
      close();
    });
  };

  // Local hover tooltip: the Rt / porosity / depth of the sample under the cursor. Suppressed
  // while a button is down (pan or the tail of a water-line pick).
  const detachTip = attachScatterTooltip(canvas, (px, py) => {
    if (downXY || !plot || !plot.inPlot(px, py)) return null;
    let best = -1;
    let bestD = 12 * 12; // within a 12 px radius
    for (let i = 0; i < rt.length; i++) {
      const vx = rt[i];
      const vy = phi[i];
      if (!Number.isFinite(vx) || !Number.isFinite(vy)) continue;
      if (plot.x.log && vx <= 0) continue;
      if (plot.y.log && vy <= 0) continue;
      const [sx, sy] = plot.toPx(vx, vy);
      const d = (sx - px) * (sx - px) + (sy - py) * (sy - py);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    if (best < 0) return null;
    const lines: string[] = [];
    if (best < depths.length && Number.isFinite(depths[best])) lines.push(`${depths[best].toFixed(1)} m`);
    lines.push(`${rtSel.value}  ${fmtValue(rt[best])}`);
    lines.push(`${phiSel.value}  ${fmtValue(phi[best])}`);
    return lines;
  });

  await reload();
  // Not awaited: a big scope must not block the panel build — the active well's plot
  // appears immediately and the context clouds fade in when ready.
  void reloadContext();
  return {
    el: content,
    dispose: () => {
      ctxGen++; // cancel any in-flight context fetch
      scope.dispose();
      unsubHover();
      unsubTheme();
      unsubData();
      unsubBrush();
      detachZoomPan();
      detachKeys();
      detachResize();
      detachTip();
      if (rafId) cancelAnimationFrame(rafId); // drop any queued hover redraw so it can't fire post-dispose
      zoneSel.dispose();
    },
    getState: () => ({
      plotId,
      rt: rtSel.value,
      phi: phiSel.value,
      m: mIn.value,
      aRw: aRwIn.value,
      zone: zoneSel.select.value,
      wells: scope.serialize(),
    }),
    openProperties: openProps,
  };
}
