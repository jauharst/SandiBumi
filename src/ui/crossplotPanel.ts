import { getCoreData, getCurveData, setZoneParam, type TrackCurveSeries, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";
import {
  attachResizeRedraw,
  attachZoomPan,
  categoricalColors,
  colorRamp,
  distinctValues,
  faciesColor,
  fitCanvasBackingStore,
  linearFit,
  looksDiscrete,
  percentile,
  PlotCanvas,
  type LinearFit,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildPlotTemplateBar,
  buildZoneSelect,
  checkboxField,
  CORE_OVERLAY_MAP,
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

export interface CrossplotOptions {
  pointSize: number;
  xLog: boolean;
  yLog: boolean;
  /** Least-squares fit (in the axes' own linear/log space) with R² readout. */
  regression: boolean;
  /** Manual axis ranges; null = auto (mnemonic defaults, else P2–P98). */
  xMin: number | null;
  xMax: number | null;
  yMin: number | null;
  yMax: number | null;
  /** Overlay core plug data as diamond markers when the X/Y axes are recognized
   *  log-curve counterparts of a core measurement (see CORE_OVERLAY_MAP). */
  showCore: boolean;
  /** Thomas-Stieber triangle overlay (meant for a VSH-PHIT plot): laminated and
   *  dispersed shale lines between draggable clean-sand / shale endpoints. */
  tsOverlay: boolean;
  /** T-S endpoints: clean-sand porosity (at VSH=0) and shale porosity (at VSH=1). */
  tsPhiSd: number;
  tsPhiSh: number;
}

export const DEFAULT_CROSSPLOT_OPTIONS: CrossplotOptions = {
  pointSize: 1.6,
  xLog: false,
  yLog: false,
  regression: false,
  xMin: null,
  xMax: null,
  yMin: null,
  yMax: null,
  showCore: false,
  tsOverlay: false,
  tsPhiSd: 0.3,
  tsPhiSh: 0.15,
};

/** Draws the Thomas-Stieber construction on an (assumed) VSH-PHIT plot: the laminated
 *  line sand→shale, the dispersed-shale line descending to its porosity minimum at
 *  VSH = PHI_SD (pores full of shale), and circle handles on the two endpoints. */
export function drawTsOverlay(plot: PlotCanvas, phiSd: number, phiSh: number): void {
  const lamColor = plot.theme.accent2;
  plot.drawLine(
    [
      [0, phiSd],
      [1, phiSh],
    ],
    lamColor,
    1.6,
  );
  // Dispersed trend: PHIT = PHI_SD − VSH·(1−PHI_SH) down to the minimum, then back up
  // to the shale point (shale beyond the pore-filling limit displaces matrix).
  const vMin = Math.min(1, phiSd); // VSH where dispersed shale exactly fills the pores
  plot.drawLine(
    [
      [0, phiSd],
      [vMin, phiSd * phiSh],
    ],
    lamColor,
    1.6,
    [5, 4],
  );
  if (vMin < 1) {
    plot.drawLine(
      [
        [vMin, phiSd * phiSh],
        [1, phiSh],
      ],
      lamColor,
      1.0,
      [2, 4],
    );
  }
  const { ctx } = plot;
  ctx.save();
  for (const [vx, vy, label] of [
    [0, phiSd, `PHI_SD_MAX ${phiSd.toFixed(3)}`] as const,
    [1, phiSh, `PHI_SH ${phiSh.toFixed(3)}`] as const,
  ]) {
    const [px, py] = plot.toPx(vx, vy);
    ctx.fillStyle = lamColor;
    ctx.strokeStyle = plot.theme.text;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(px, py, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.font = "500 10px system-ui, sans-serif";
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = vx === 0 ? "left" : "right";
    ctx.fillText(label, px + (vx === 0 ? 8 : -8), py - 8);
  }
  ctx.restore();
}

/** Default axis ranges by mnemonic; anything else auto-ranges from P2–P98. */
function axisDefaults(curve: string): { min: number; max: number; invert: boolean } | null {
  switch (curve.toUpperCase()) {
    case "NPHI":
      return { min: -0.05, max: 0.6, invert: false };
    case "RHOB":
      return { min: 1.9, max: 3.0, invert: true }; // density increases downward (D-N convention)
    case "GR":
      return { min: 0, max: 200, invert: false };
    case "DT":
      return { min: 40, max: 190, invert: false };
    case "VSH":
    case "PHIE":
    case "PHIT":
    case "SWE":
    case "SWT":
      return { min: 0, max: 1, invert: false };
    default:
      return null;
  }
}

/** Quartz / calcite / dolomite matrix reference points in (NPHI, RHOB) space — drawn
 *  only when the plot actually is an NPHI-RHOB crossplot. */
const MATRIX_POINTS: { nphi: number; rhob: number; label: string }[] = [
  { nphi: -0.02, rhob: 2.65, label: "Qtz" },
  { nphi: 0.0, rhob: 2.71, label: "Cal" },
  { nphi: 0.02, rhob: 2.87, label: "Dol" },
];

/** Pairs two independently-filtered core series (each keeps only its own non-NaN
 *  samples, so their indices don't line up) by matching on exact depth — only plugs
 *  with both measurements present are plotted. */
function alignCoreSeriesByDepth(a: TrackCurveSeries, b: TrackCurveSeries): { xs: Float32Array; ys: Float32Array } {
  const bByDepth = new Map<number, number>();
  for (let i = 0; i < b.depth.length; i++) bByDepth.set(b.depth[i], b.value[i]);
  const xs: number[] = [];
  const ys: number[] = [];
  for (let i = 0; i < a.depth.length; i++) {
    const bv = bByDepth.get(a.depth[i]);
    if (bv !== undefined) {
      xs.push(a.value[i]);
      ys.push(bv);
    }
  }
  return { xs: Float32Array.from(xs), ys: Float32Array.from(ys) };
}

/** Human-readable fit equation in the space the fit was made in. */
export function fitEquation(fit: LinearFit, xName: string, yName: string, xLog: boolean, yLog: boolean): string {
  const a = fit.a;
  const b = fit.b;
  const num = (v: number) => v.toPrecision(4);
  if (!xLog && !yLog) return `${yName} = ${num(a)} + ${num(b)}·${xName}`;
  if (xLog && yLog) return `${yName} = ${num(Math.pow(10, a))} · ${xName}^${num(b)}`;
  if (xLog) return `${yName} = ${num(a)} + ${num(b)}·log10(${xName})`;
  return `${yName} = 10^(${num(a)} + ${num(b)}·${xName})`;
}

export function drawCrossplot(
  canvas: HTMLCanvasElement,
  xName: string,
  yName: string,
  zName: string,
  xs: Float32Array,
  ys: Float32Array,
  zs: Float32Array,
  opts: CrossplotOptions = DEFAULT_CROSSPLOT_OPTIONS,
  hoverIdx = -1,
  view: Viewport | null = null,
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const auto = (values: Float32Array): { min: number; max: number } | null => {
    const lo = percentile(values, 2);
    const hi = percentile(values, 98);
    if (Number.isNaN(lo) || Number.isNaN(hi) || lo === hi) return null;
    const pad = (hi - lo) * 0.08;
    return { min: lo - pad, max: hi + pad };
  };
  /** Manual range > mnemonic default > P2–P98 auto; log axes get a positive floor. */
  const resolve = (
    name: string,
    values: Float32Array,
    log: boolean,
    manMin: number | null,
    manMax: number | null,
  ): { min: number; max: number; invert: boolean } | null => {
    const base = axisDefaults(name) ?? (auto(values) ? { ...auto(values)!, invert: false } : null);
    if (!base && (manMin === null || manMax === null)) return null;
    let min = manMin ?? base!.min;
    let max = manMax ?? base!.max;
    const invert = base?.invert ?? false;
    if (log) {
      if (max <= 0) return null;
      if (min <= 0) {
        let smallest = Infinity;
        for (let i = 0; i < values.length; i++) {
          const v = values[i];
          if (!Number.isNaN(v) && v > 0 && v < smallest) smallest = v;
        }
        min = Number.isFinite(smallest) ? smallest * 0.8 : max / 1000;
      }
    }
    if (min === max) return null;
    return { min, max, invert };
  };

  const xr = resolve(xName, xs, opts.xLog, opts.xMin, opts.xMax);
  const yr = resolve(yName, ys, opts.yLog, opts.yMin, opts.yMax);
  if (!xr || !yr) return null;
  // A zoom/pan viewport (if any) overrides the computed window, keeping axis inversion.
  if (view) {
    xr.min = view.xMin;
    xr.max = view.xMax;
    yr.min = view.yMin;
    yr.max = view.yMax;
  }

  const plot = new PlotCanvas(
    canvas,
    { label: xName, min: xr.min, max: xr.max, log: opts.xLog, invert: xr.invert },
    { label: yName, min: yr.min, max: yr.max, log: opts.yLog, invert: yr.invert },
  );
  plot.drawFrame();

  // Discrete class curves (electrofacies, clusters) get categorical coloring + a swatch
  // legend; continuous curves keep the blue→red ramp + color bar.
  const categorical = /FACIES|CLUSTER|LITHO|CLASS/i.test(zName) || looksDiscrete(zs);
  const zLo = percentile(zs, 5);
  const zHi = percentile(zs, 95);
  let colors: string[] | undefined;
  if (categorical) {
    colors = categoricalColors(zs);
  } else {
    colors = !Number.isNaN(zLo) && zLo !== zHi ? colorRamp(zs, zLo, zHi) : undefined;
  }
  plot.drawScatter(xs, ys, colors, Math.max(0.5, opts.pointSize));

  if (categorical) {
    // Discrete facies legend: one swatch per class actually present.
    const { ctx } = plot;
    const r = plot.plotRect;
    const classes = distinctValues(zs);
    ctx.save();
    ctx.font = "500 10px system-ui, sans-serif";
    const rowH = 15;
    const boxW = 78;
    const boxX = r.x0 + r.w - boxW - 8;
    let boxY = r.y0 + 8;
    ctx.fillStyle = plot.theme.text;
    ctx.textAlign = "left";
    ctx.fillText(zName, boxX, boxY + 9);
    boxY += rowH;
    for (const c of classes) {
      ctx.fillStyle = faciesColor(c);
      ctx.fillRect(boxX, boxY, 11, 11);
      ctx.strokeStyle = plot.theme.text;
      ctx.lineWidth = 0.5;
      ctx.strokeRect(boxX, boxY, 11, 11);
      ctx.fillStyle = plot.theme.text;
      ctx.fillText(`F${c}`, boxX + 16, boxY + 9);
      boxY += rowH;
    }
    ctx.restore();
  } else if (colors) {
    // Z color-bar legend.
    const { ctx } = plot;
    const r = plot.plotRect;
    const barW = 90;
    const barX = r.x0 + r.w - barW - 8;
    const barY = r.y0 + 8;
    for (let i = 0; i < barW; i++) {
      const t = i / (barW - 1);
      ctx.fillStyle = `hsl(${(220 * (1 - t)).toFixed(0)}, 75%, 45%)`;
      ctx.fillRect(barX + i, barY, 1, 8);
    }
    ctx.fillStyle = plot.theme.text;
    ctx.font = "500 9px system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(zLo.toPrecision(3), barX, barY + 18);
    ctx.fillText(zName, barX + barW / 2, barY + 18);
    ctx.fillText(zHi.toPrecision(3), barX + barW, barY + 18);
  }

  // Least-squares regression in the axes' own space (straight on the plot).
  if (opts.regression) {
    const fit = linearFit(xs, ys, opts.xLog, opts.yLog);
    if (fit) {
      const tx = (v: number) => (opts.xLog ? Math.log10(v) : v);
      const invY = (v: number) => (opts.yLog ? Math.pow(10, v) : v);
      const yAt = (x: number) => invY(fit.a + fit.b * tx(x));
      plot.drawLine(
        [
          [xr.min, yAt(xr.min)],
          [xr.max, yAt(xr.max)],
        ],
        plot.theme.accent2,
        1.8,
        [7, 4],
      );
      const { ctx } = plot;
      const r = plot.plotRect;
      ctx.save();
      ctx.font = "500 10px system-ui, sans-serif";
      ctx.fillStyle = plot.theme.text;
      ctx.textAlign = "left";
      ctx.fillText(
        `${fitEquation(fit, xName, yName, opts.xLog, opts.yLog)}   (R² = ${fit.r2.toFixed(3)}, n = ${fit.n})`,
        r.x0 + 8,
        r.y0 + 14,
      );
      ctx.restore();
    }
  }

  // Matrix reference points on a genuine NPHI-RHOB plot (either orientation).
  const isND = xName.toUpperCase() === "NPHI" && yName.toUpperCase() === "RHOB";
  const isDN = xName.toUpperCase() === "RHOB" && yName.toUpperCase() === "NPHI";
  if (isND || isDN) {
    for (const m of MATRIX_POINTS) {
      if (isND) plot.drawRefPoint(m.nphi, m.rhob, m.label);
      else plot.drawRefPoint(m.rhob, m.nphi, m.label);
    }
  }

  // Synchronized hover: ring the sample at the depth under another view's cursor.
  if (hoverIdx >= 0 && hoverIdx < xs.length) {
    const hx = xs[hoverIdx];
    const hy = ys[hoverIdx];
    if (!Number.isNaN(hx) && !Number.isNaN(hy) && (!opts.xLog || hx > 0) && (!opts.yLog || hy > 0)) {
      const [px, py] = plot.toPx(hx, hy);
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

/** Crossplot panel: any two catalog curves with a third as color (default NPHI-RHOB
 *  colored by GR). Click a point to push its (x, y) into zone parameters; optional
 *  least-squares regression with R² (por-perm transform calibration); properties row
 *  for point size, log axes, and manual ranges. Follows the synchronized depth cursor. */
export async function buildCrossplotContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);
  const opts: CrossplotOptions = { ...DEFAULT_CROSSPLOT_OPTIONS, ...(await loadPlotProps<CrossplotOptions>("crossplot")) };

  const content = document.createElement("div");
  content.className = "plot-content";
  const xSel = curveSelect(curveNames, initial?.x ?? "NPHI");
  const ySel = curveSelect(curveNames, initial?.y ?? "RHOB");
  const zSel = curveSelect(curveNames, initial?.z ?? "GR");

  const persist = () => savePlotProps("crossplot", opts);

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("X", xSel));
  selRow.appendChild(formRow("Y", ySel));
  selRow.appendChild(formRow("Color", zSel));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(
    buildPlotTemplateBar<CrossplotOptions>(
      "crossplot",
      "Crossplot",
      () => ({ ...opts }),
      (t) => {
        Object.assign(opts, t);
        renderProps();
        persist();
        void reloadCore();
        redraw();
      },
      setStatus,
    ),
  );
  selRow.appendChild(buildImageExportButtons(() => canvas, "Crossplot", setStatus));
  content.appendChild(selRow);

  // Properties row: regression, log axes, point size, manual ranges (blank = auto).
  // Built by a function so recalling a template can rebuild it to reflect the new opts.
  const propsRow = document.createElement("div");
  propsRow.className = "plot-props";

  const numField = (label: string, value: number | null, apply: (v: number | null) => void, width = 58): HTMLElement => {
    const wrap = document.createElement("label");
    wrap.className = "chk-field";
    wrap.appendChild(document.createTextNode(label));
    const input = document.createElement("input");
    input.className = "form-control";
    input.type = "number";
    input.step = "any";
    input.style.width = `${width}px`;
    input.placeholder = "auto";
    if (value !== null) input.value = String(value);
    input.addEventListener("change", () => {
      const v = input.value.trim() === "" ? null : parseFloat(input.value);
      apply(v === null || Number.isNaN(v) ? null : v);
      persist();
      redraw();
    });
    wrap.appendChild(input);
    return wrap;
  };

  const renderProps = (): void => {
    propsRow.innerHTML = "";
    propsRow.appendChild(
      checkboxField("Regression", opts.regression, (v) => {
        opts.regression = v;
        persist();
        redraw();
      }),
    );
    propsRow.appendChild(
      checkboxField("X log", opts.xLog, (v) => {
        opts.xLog = v;
        persist();
        redraw();
      }),
    );
    propsRow.appendChild(
      checkboxField("Y log", opts.yLog, (v) => {
        opts.yLog = v;
        persist();
        redraw();
      }),
    );
    propsRow.appendChild(
      checkboxField("Core data", opts.showCore, (v) => {
        opts.showCore = v;
        persist();
        void reloadCore();
      }),
    );
    propsRow.appendChild(
      checkboxField("T-S triangle", opts.tsOverlay, (v) => {
        opts.tsOverlay = v;
        persist();
        redraw();
      }),
    );
    propsRow.appendChild(numField("Point px", opts.pointSize, (v) => (opts.pointSize = v ?? DEFAULT_CROSSPLOT_OPTIONS.pointSize), 48));
    propsRow.appendChild(numField("X min", opts.xMin, (v) => (opts.xMin = v)));
    propsRow.appendChild(numField("X max", opts.xMax, (v) => (opts.xMax = v)));
    propsRow.appendChild(numField("Y min", opts.yMin, (v) => (opts.yMin = v)));
    propsRow.appendChild(numField("Y max", opts.yMax, (v) => (opts.yMax = v)));
  };
  renderProps();
  content.appendChild(propsRow);

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 460;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  hint.textContent =
    "Drag the ringed handle to set the X/Y parameters (release writes them to the zone, e.g. shale point → NPHI_SH + RHO_SH). Click empty space to reposition it. Ctrl+wheel = zoom, drag background = pan, double-click = reset. Qtz/Cal/Dol matrix points marked.";
  content.appendChild(hint);

  const pickX = pickRow("X pick", "#b5651d", "NPHI_SH", well, zoneSel.current, setStatus);
  const pickY = pickRow("Y pick", "#5f7350", "RHO_SH", well, zoneSel.current, setStatus);
  content.appendChild(pickX.row);
  content.appendChild(pickY.row);

  let xs = new Float32Array(0);
  let ys = new Float32Array(0);
  let zs = new Float32Array(0);
  let depths = new Float32Array(0);
  let coreByName = new Map<string, TrackCurveSeries>();
  let plot: PlotCanvas | null = null;
  let marker: [number, number] | null = null;
  let hoverIdx = -1;
  const viewRef: ViewportRef = { current: null };

  const redraw = () => {
    plot = drawCrossplot(canvas, xSel.value, ySel.value, zSel.value, xs, ys, zs, opts, hoverIdx, viewRef.current);
    if (!plot) {
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.font = "500 12px system-ui, sans-serif";
      ctx.fillStyle = "#888";
      ctx.textAlign = "center";
      ctx.fillText("No valid data for these curves/zone.", canvas.width / 2, canvas.height / 2);
      return;
    }
    if (opts.showCore) {
      const coreX = coreByName.get(CORE_OVERLAY_MAP[xSel.value.toUpperCase()] ?? "");
      const coreY = coreByName.get(CORE_OVERLAY_MAP[ySel.value.toUpperCase()] ?? "");
      if (coreX && coreY) {
        const { xs: cxs, ys: cys } = alignCoreSeriesByDepth(coreX, coreY);
        plot.drawDiamonds(cxs, cys, plot.theme.accent2);
      }
    }
    if (opts.tsOverlay) {
      drawTsOverlay(plot, opts.tsPhiSd, opts.tsPhiSh);
    }
    if (marker) {
      const [px, py] = plot.toPx(marker[0], marker[1]);
      const ctx = plot.ctx;
      ctx.save();
      ctx.strokeStyle = plot.theme.warn;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(px - 7, py);
      ctx.lineTo(px + 7, py);
      ctx.moveTo(px, py - 7);
      ctx.lineTo(px, py + 7);
      ctx.stroke();
      ctx.restore();
    }
    drawParamHandle();
  };

  /** Draws the draggable parameter point at (X pick, Y pick) as a grabbable ring, so the
   *  user can drag it around the cloud to set the two zone parameters interactively. */
  const drawParamHandle = () => {
    if (!plot) return;
    const vx = pickX.getValue();
    const vy = pickY.getValue();
    if (Number.isNaN(vx) || Number.isNaN(vy)) return;
    if ((opts.xLog && vx <= 0) || (opts.yLog && vy <= 0)) return;
    const [px, py] = plot.toPx(vx, vy);
    if (!plot.inPlot(px, py)) return;
    const ctx = plot.ctx;
    ctx.save();
    ctx.fillStyle = plot.theme.accent;
    ctx.strokeStyle = plot.theme.bg;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(px, py, 6, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.strokeStyle = plot.theme.accent;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(px, py, 9, 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  };

  const reload = async () => {
    const zone = zoneSel.current();
    try {
      const series = await getCurveData(well.well_id, [xSel.value, ySel.value, zSel.value], zone.depthMin, zone.depthMax);
      const byName = new Map(series.map((s) => [s.curve_name, s]));
      xs = byName.get(xSel.value.toUpperCase())?.value ?? new Float32Array(0);
      ys = byName.get(ySel.value.toUpperCase())?.value ?? new Float32Array(0);
      zs = byName.get(zSel.value.toUpperCase())?.value ?? new Float32Array(0);
      depths = byName.get(xSel.value.toUpperCase())?.depth ?? new Float32Array(0);
    } catch (err) {
      setStatus(`Crossplot data load failed: ${err}`);
      xs = ys = zs = depths = new Float32Array(0);
    }
    marker = null;
    hoverIdx = -1;
    viewRef.current = null; // new data → reset any zoom/pan
    // Seed the draggable parameter point at the cloud's median so it's always visible and
    // grabbable (the user drags it to set the shale/matrix point); only when still unset.
    if (Number.isNaN(pickX.getValue()) || Number.isNaN(pickY.getValue())) {
      const mx = percentile(xs, 50);
      const my = percentile(ys, 50);
      if (!Number.isNaN(mx) && !Number.isNaN(my)) {
        pickX.setValue(mx);
        pickY.setValue(my);
        marker = [mx, my];
      }
    }
    redraw();
  };

  /** Loads the well's core data once (all four series; cheap — core datasets are
   *  small) whenever the "Core data" toggle is switched on. */
  const reloadCore = async () => {
    if (!opts.showCore) {
      coreByName = new Map();
      redraw();
      return;
    }
    try {
      const series = await getCoreData(well.well_id);
      coreByName = new Map(series.map((s) => [s.curve_name, s]));
    } catch (err) {
      setStatus(`Core data load failed: ${err}`);
      coreByName = new Map();
    }
    redraw();
  };

  for (const sel of [xSel, ySel, zSel, zoneSel.select]) {
    sel.addEventListener("change", () => void reload());
  }

  // --- Interactive handles: parameter point + Thomas-Stieber endpoints -----
  // The parameter point is a draggable 2-D handle at (X pick, Y pick): dragging it moves
  // both picks live, and releasing writes them to the selected zone's parameters (e.g. a
  // shale point → NPHI_SH + RHO_SH). The T-S endpoints (when that overlay is on) drag
  // vertically as before. Empty-space drag pans and the wheel zooms (attachZoomPan); a
  // press landing on a handle vetoes the pan so the drag stays clean. Coordinates are in
  // logical (CSS) pixels — the same space PlotCanvas.toPx/toData use post-HiDPI.
  type DragMode = "param" | "ts-sand" | "ts-shale" | null;
  let drag: DragMode = null;
  let downXY: [number, number] | null = null;
  let movedSinceDown = false;

  const canvasPx = (e: MouseEvent): [number, number] => {
    const rect = canvas.getBoundingClientRect();
    return [e.clientX - rect.left, e.clientY - rect.top];
  };
  const paramName = (row: HTMLElement): string =>
    (row.querySelector(".pick-param") as HTMLInputElement | null)?.value.trim().toUpperCase() ?? "";

  /** Which draggable handle (if any) is under a screen point. */
  const handleAt = (px: number, py: number): DragMode => {
    if (!plot) return null;
    if (opts.tsOverlay) {
      for (const [which, vx, vy] of [
        ["ts-sand", 0, opts.tsPhiSd] as const,
        ["ts-shale", 1, opts.tsPhiSh] as const,
      ]) {
        const [hx, hy] = plot.toPx(vx, vy);
        if (Math.hypot(px - hx, py - hy) <= 9) return which;
      }
    }
    const vx = pickX.getValue();
    const vy = pickY.getValue();
    if (!Number.isNaN(vx) && !Number.isNaN(vy) && (!opts.xLog || vx > 0) && (!opts.yLog || vy > 0)) {
      const [hx, hy] = plot.toPx(vx, vy);
      if (Math.hypot(px - hx, py - hy) <= 10) return "param";
    }
    return null;
  };

  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0 || !plot) return;
    const [px, py] = canvasPx(e);
    downXY = [px, py];
    movedSinceDown = false;
    drag = handleAt(px, py);
    if (drag) e.preventDefault();
  });

  canvas.addEventListener("mousemove", (e) => {
    if (!plot) return;
    const [px, py] = canvasPx(e);
    if (downXY && Math.hypot(px - downXY[0], py - downXY[1]) > 4) movedSinceDown = true;
    if (drag) {
      const [vx, vy] = plot.toData(px, py);
      if (drag === "param") {
        pickX.setValue(vx);
        pickY.setValue(vy);
        marker = [vx, vy];
      } else {
        const clamped = Math.min(0.5, Math.max(0, vy));
        if (drag === "ts-sand") opts.tsPhiSd = clamped;
        else opts.tsPhiSh = clamped;
      }
      redraw();
      return;
    }
    if (!downXY) canvas.style.cursor = handleAt(px, py) ? "grab" : "";
  });

  const endDrag = () => {
    const mode = drag;
    drag = null;
    downXY = null;
    if (!mode || !movedSinceDown) return;
    const zone = zoneSel.current();
    const write = (param: string, value: number, dp: (v: number) => string) => {
      if (!param || Number.isNaN(value)) return;
      void setZoneParam(well.well_id, zone.zoneName, param, value, null)
        .then(() => setStatus(`${param} = ${dp(value)} set on zone '${zone.zoneName}' of ${well.well_name}`))
        .catch((err) => setStatus(`Failed to set ${param}: ${err}`));
    };
    if (mode === "param") {
      write(paramName(pickX.row), pickX.getValue(), (v) => v.toPrecision(4));
      write(paramName(pickY.row), pickY.getValue(), (v) => v.toPrecision(4));
    } else {
      persist();
      if (mode === "ts-sand") write("PHI_SD_MAX", opts.tsPhiSd, (v) => v.toFixed(3));
      else write("PHI_SH", opts.tsPhiSh, (v) => v.toFixed(3));
    }
  };
  canvas.addEventListener("mouseup", endDrag);

  canvas.addEventListener("click", (e) => {
    if (!plot || movedSinceDown) return; // tail of a drag or pan, not a point pick
    const [px, py] = canvasPx(e);
    if (!plot.inPlot(px, py)) return;
    const [vx, vy] = plot.toData(px, py);
    marker = [vx, vy];
    pickX.setValue(vx);
    pickY.setValue(vy);
    redraw();
  });

  const detachZoomPan = attachZoomPan({
    canvas,
    getPlot: () => plot,
    view: viewRef,
    redraw,
    onPanStart: (px, py) => handleAt(px, py) === null, // a handle grab vetoes panning
  });
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
  await reloadCore();
  return {
    el: content,
    dispose: () => {
      unsubHover();
      unsubTheme();
      detachZoomPan();
      detachResize();
      zoneSel.dispose();
    },
    getState: () => ({ x: xSel.value, y: ySel.value, z: zSel.value, zone: zoneSel.select.value }),
  };
}
