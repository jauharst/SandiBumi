import { getCurveData, type WellSummary } from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";
import {
  attachResizeRedraw,
  attachZoomPan,
  basicStats,
  fitCanvasBackingStore,
  percentile,
  PlotCanvas,
  type BasicStats,
  type Viewport,
  type ViewportRef,
} from "./plotCanvas";
import {
  buildPlotTemplateBar,
  buildZoneSelect,
  checkboxField,
  curveSelect,
  defaultPickParams,
  loadCurveNames,
  loadPlotProps,
  nearestDepthIndex,
  pickRow,
  savePlotProps,
  trySelect,
  type PlotContent,
} from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

export type HistogramMode = "bars" | "line" | "cumulative";

export type StatKey = "mean" | "p50" | "p5" | "p95" | "std" | "count";

export interface HistogramOptions {
  mode: HistogramMode;
  bins: number;
  /** Y axis as % of samples instead of raw count (ignored in cumulative mode). */
  normalize: boolean;
  /** Which statistics chips are active; mean/p5/p50/p95 also draw marker lines. */
  stats: StatKey[];
}

export const DEFAULT_HISTOGRAM_OPTIONS: HistogramOptions = {
  mode: "bars",
  bins: 60,
  normalize: false,
  stats: ["p5", "p50", "p95", "count"],
};

const STAT_DEFS: { key: StatKey; label: string; fmt: (s: BasicStats) => string; marker: boolean }[] = [
  { key: "mean", label: "Mean", fmt: (s) => s.mean.toPrecision(4), marker: true },
  { key: "p50", label: "P50", fmt: (s) => s.p50.toPrecision(4), marker: true },
  { key: "p5", label: "P5", fmt: (s) => s.p5.toPrecision(4), marker: true },
  { key: "p95", label: "P95", fmt: (s) => s.p95.toPrecision(4), marker: true },
  { key: "std", label: "Std", fmt: (s) => s.std.toPrecision(4), marker: false },
  { key: "count", label: "n", fmt: (s) => String(s.count), marker: false },
];

/** Computes histogram bin counts over [min, max]; NaN values are skipped. */
export function computeHistogram(
  values: ArrayLike<number>,
  min: number,
  max: number,
  bins = DEFAULT_HISTOGRAM_OPTIONS.bins,
): { counts: number[]; edges: number[]; n: number } {
  const counts = new Array(bins).fill(0);
  const width = (max - min) / bins;
  let n = 0;
  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (Number.isNaN(v) || v < min || v > max) continue;
    const bin = Math.min(bins - 1, Math.floor((v - min) / width));
    counts[bin]++;
    n++;
  }
  const edges = Array.from({ length: bins + 1 }, (_, i) => min + i * width);
  return { counts, edges, n };
}

/** Draws the histogram in the requested mode (bars / frequency polyline / cumulative %)
 *  with the selected statistic markers, pick markers, and an optional synchronized-hover
 *  marker at the curve's value under another view's cursor. Exported so it can be
 *  exercised with synthetic data in tests. */
export function drawHistogram(
  canvas: HTMLCanvasElement,
  values: ArrayLike<number>,
  curveName: string,
  picks: { value: number; color: string; label: string }[],
  opts: HistogramOptions = DEFAULT_HISTOGRAM_OPTIONS,
  hoverValue: number | null = null,
  view: Viewport | null = null,
): PlotCanvas | null {
  fitCanvasBackingStore(canvas);
  const p2 = percentile(values, 2);
  const p98 = percentile(values, 98);
  if (Number.isNaN(p2) || Number.isNaN(p98) || p2 === p98) return null;
  const pad = (p98 - p2) * 0.08;
  // Bin edges always span the full data range; a zoom/pan viewport only changes the
  // visible X window (the axis), so bars keep their identity as you zoom in.
  const min = p2 - pad;
  const max = p98 + pad;
  const xMin = view ? view.xMin : min;
  const xMax = view ? view.xMax : max;

  const bins = Math.max(5, Math.min(400, Math.round(opts.bins) || DEFAULT_HISTOGRAM_OPTIONS.bins));
  const { counts, edges, n } = computeHistogram(values, min, max, bins);
  if (n === 0) return null;

  const cumulative = opts.mode === "cumulative";
  const normalize = opts.normalize && !cumulative;
  const yScale = cumulative ? 100 / n : normalize ? 100 / n : 1;
  const peak = Math.max(...counts, 1) * yScale;
  const yMax = cumulative ? 105 : peak * 1.06;
  const yLabel = cumulative ? "Cumulative %" : normalize ? `% of samples (n=${n})` : `Count (n=${n})`;

  const plot = new PlotCanvas(
    canvas,
    { label: curveName, min: xMin, max: xMax, log: false, invert: false },
    { label: yLabel, min: 0, max: yMax, log: false, invert: false },
  );
  plot.drawFrame();

  if (opts.mode === "bars") {
    const { ctx } = plot;
    ctx.save();
    ctx.fillStyle = plot.theme.accent;
    ctx.globalAlpha = 0.75;
    for (let i = 0; i < counts.length; i++) {
      if (counts[i] === 0) continue;
      const [x0, yTop] = plot.toPx(edges[i], counts[i] * yScale);
      const [x1, yBase] = plot.toPx(edges[i + 1], 0);
      ctx.fillRect(x0 + 0.5, yTop, Math.max(1, x1 - x0 - 1), yBase - yTop);
    }
    ctx.restore();
  } else if (opts.mode === "line") {
    const points: [number, number][] = counts.map((c, i) => [(edges[i] + edges[i + 1]) / 2, c * yScale]);
    plot.drawLine(points, plot.theme.accent, 1.8);
  } else {
    // Cumulative % at each bin's right edge, anchored at (min, 0).
    const points: [number, number][] = [[edges[0], 0]];
    let running = 0;
    for (let i = 0; i < counts.length; i++) {
      running += counts[i];
      points.push([edges[i + 1], (running / n) * 100]);
    }
    plot.drawLine(points, plot.theme.accent, 1.8);
  }

  const stats = basicStats(values);
  for (const def of STAT_DEFS) {
    if (!def.marker || !opts.stats.includes(def.key)) continue;
    const v = { mean: stats.mean, p50: stats.p50, p5: stats.p5, p95: stats.p95 }[def.key as "mean" | "p50" | "p5" | "p95"];
    if (!Number.isNaN(v)) plot.drawVMarker(v, plot.theme.text, `${def.label} ${v.toPrecision(4)}`);
  }
  for (const pick of picks) {
    if (!Number.isNaN(pick.value)) plot.drawVMarker(pick.value, pick.color, pick.label);
  }
  if (hoverValue !== null && !Number.isNaN(hoverValue)) {
    plot.drawVMarker(hoverValue, plot.theme.warn);
  }
  return plot;
}

/** Histogram panel: pick a curve + zone window, choose bars/line/cumulative display,
 *  toggle statistics chips, click the plot to place two picks, and write each straight
 *  into a zone parameter (e.g. GR_MA / GR_SH). Follows the synchronized depth cursor of
 *  the log views (red marker = value at the hovered depth). */
export async function buildHistogramContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);
  const opts: HistogramOptions = { ...DEFAULT_HISTOGRAM_OPTIONS, ...(await loadPlotProps<HistogramOptions>("histogram")) };

  const content = document.createElement("div");
  content.className = "plot-content";
  const curveSel = curveSelect(curveNames, initial?.curve ?? "GR");

  const modeSel = document.createElement("select");
  modeSel.className = "form-control";
  for (const [value, label] of [
    ["bars", "Bars"],
    ["line", "Line"],
    ["cumulative", "Cumulative"],
  ] as [HistogramMode, string][]) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    modeSel.appendChild(option);
  }
  modeSel.value = opts.mode;

  const binsIn = document.createElement("input");
  binsIn.className = "form-control";
  binsIn.type = "number";
  binsIn.min = "5";
  binsIn.max = "400";
  binsIn.value = String(opts.bins);

  const selRow = document.createElement("div");
  selRow.className = "plot-toolbar";
  selRow.appendChild(formRow("Curve", curveSel));
  selRow.appendChild(formRow("Zone", zoneSel.select));
  selRow.appendChild(formRow("Display", modeSel));
  selRow.appendChild(formRow("Bins", binsIn));
  const normChk = checkboxField("Normalize (%)", opts.normalize, (v) => {
    opts.normalize = v;
    persist();
    redraw();
  });
  selRow.appendChild(normChk);
  const normInput = normChk.querySelector<HTMLInputElement>("input")!;

  // Named templates for the display style, and image export (copy / save / print).
  selRow.appendChild(
    buildPlotTemplateBar<HistogramOptions>(
      "histogram",
      "Histogram",
      () => ({ ...opts }),
      (t) => {
        if (t.mode) modeSel.value = opts.mode = t.mode;
        if (typeof t.bins === "number") binsIn.value = String((opts.bins = t.bins));
        if (typeof t.normalize === "boolean") normInput.checked = opts.normalize = t.normalize;
        if (t.stats) opts.stats = t.stats;
        persist();
        renderChips();
        redraw();
      },
      setStatus,
    ),
  );
  selRow.appendChild(buildImageExportButtons(() => canvas, "Histogram", setStatus));
  content.appendChild(selRow);

  // Statistics chips — click to toggle; active chips show the value and (for the
  // percentile/mean ones) draw a marker line on the plot.
  const chipsRow = document.createElement("div");
  chipsRow.className = "stat-chips";
  content.appendChild(chipsRow);
  let stats: BasicStats | null = null;

  const renderChips = () => {
    chipsRow.innerHTML = "";
    for (const def of STAT_DEFS) {
      const chip = document.createElement("button");
      const active = opts.stats.includes(def.key);
      chip.className = "stat-chip" + (active ? " active" : "");
      chip.textContent = active && stats && stats.count > 0 ? `${def.label} ${def.fmt(stats)}` : def.label;
      chip.title = active ? "Click to hide" : "Click to show";
      chip.addEventListener("click", () => {
        opts.stats = active ? opts.stats.filter((k) => k !== def.key) : [...opts.stats, def.key];
        persist();
        renderChips();
        redraw();
      });
      chipsRow.appendChild(chip);
    }
  };

  const canvas = document.createElement("canvas");
  canvas.width = 720;
  canvas.height = 380;
  canvas.className = "plot-canvas";
  content.appendChild(canvas);

  const hint = document.createElement("p");
  hint.className = "modal-hint";
  hint.textContent =
    "Click the plot to place the active pick. Ctrl+wheel = zoom X, drag = pan, double-click = reset. Toggle the chips above to show/hide statistics.";
  content.appendChild(hint);

  // Two picks with radio-style activation.
  const [p1Default, p2Default] = defaultPickParams(curveSel.value);
  const theme = { a: "#b5651d", b: "#5f7350" };
  const pickA = pickRow("Pick A", theme.a, p1Default, well, zoneSel.current, setStatus);
  const pickB = pickRow("Pick B", theme.b, p2Default, well, zoneSel.current, setStatus);
  let active: "A" | "B" = "A";

  const activator = (which: "A" | "B", row: HTMLElement) => {
    row.classList.toggle("pick-active", active === which);
    row.addEventListener("click", () => {
      active = which;
      pickA.row.classList.toggle("pick-active", active === "A");
      pickB.row.classList.toggle("pick-active", active === "B");
    });
  };
  activator("A", pickA.row);
  activator("B", pickB.row);
  content.appendChild(pickA.row);
  content.appendChild(pickB.row);

  let values: Float32Array = new Float32Array(0);
  let depths: Float32Array = new Float32Array(0);
  let plot: PlotCanvas | null = null;
  let hoverValue: number | null = null;
  const viewRef: ViewportRef = { current: null };

  const persist = () => savePlotProps("histogram", opts);

  const redraw = () => {
    plot = drawHistogram(
      canvas,
      values,
      curveSel.value,
      [
        { value: pickA.getValue(), color: theme.a, label: "A" },
        { value: pickB.getValue(), color: theme.b, label: "B" },
      ],
      opts,
      hoverValue,
      viewRef.current,
    );
    if (!plot) {
      const ctx = canvas.getContext("2d")!;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.font = "500 12px system-ui, sans-serif";
      ctx.fillStyle = "#888";
      ctx.textAlign = "center";
      ctx.fillText("No valid data for this curve/zone.", canvas.width / 2, canvas.height / 2);
    }
  };

  const reload = async () => {
    const zone = zoneSel.current();
    try {
      const series = await getCurveData(well.well_id, [curveSel.value], zone.depthMin, zone.depthMax);
      values = series[0]?.value ?? new Float32Array(0);
      depths = series[0]?.depth ?? new Float32Array(0);
    } catch (err) {
      setStatus(`Histogram data load failed: ${err}`);
      values = new Float32Array(0);
      depths = new Float32Array(0);
    }
    stats = basicStats(values);
    viewRef.current = null; // new data → reset any zoom/pan
    renderChips();
    redraw();
  };

  curveSel.addEventListener("change", () => {
    const [pa, pb] = defaultPickParams(curveSel.value);
    (pickA.row.querySelector(".pick-param") as HTMLInputElement).value = pa;
    (pickB.row.querySelector(".pick-param") as HTMLInputElement).value = pb;
    void reload();
  });
  zoneSel.select.addEventListener("change", () => void reload());
  modeSel.addEventListener("change", () => {
    opts.mode = modeSel.value as HistogramMode;
    persist();
    redraw();
  });
  binsIn.addEventListener("change", () => {
    opts.bins = Math.max(5, Math.min(400, parseInt(binsIn.value, 10) || DEFAULT_HISTOGRAM_OPTIONS.bins));
    binsIn.value = String(opts.bins);
    persist();
    redraw();
  });

  // Track drag so a pan (below) doesn't also fire a pick. Coordinates are logical (CSS)
  // pixels — the space PlotCanvas.toData works in after HiDPI scaling.
  let downX: number | null = null;
  let movedSinceDown = false;
  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    const rect = canvas.getBoundingClientRect();
    downX = e.clientX - rect.left;
    movedSinceDown = false;
  });
  canvas.addEventListener("mousemove", (e) => {
    if (downX === null) return;
    if (Math.abs(e.clientX - canvas.getBoundingClientRect().left - downX) > 4) movedSinceDown = true;
  });
  canvas.addEventListener("click", (e) => {
    downX = null;
    if (!plot || movedSinceDown) return; // tail of a pan, not a pick
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    if (!plot.inPlot(px, py)) return;
    const [vx] = plot.toData(px, py);
    (active === "A" ? pickA : pickB).setValue(vx);
    redraw();
  });

  // Wheel-zoom + drag-pan on the X axis only (Y is the count axis); double-click resets.
  const detachZoomPan = attachZoomPan({ canvas, getPlot: () => plot, view: viewRef, redraw, axes: "x" });
  const detachResize = attachResizeRedraw(canvas, redraw);
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());

  // Synchronized hover: mark this curve's value at the depth under any log view's cursor.
  let rafId = 0;
  const unsubHover = appState.hoverDepth.subscribe((depth) => {
    const idx = depth === null ? -1 : nearestDepthIndex(depths, depth);
    const next = idx < 0 ? null : values[idx];
    const normalized = next === undefined || Number.isNaN(next!) ? null : next;
    if (normalized === hoverValue) return;
    hoverValue = normalized;
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
    getState: () => ({ curve: curveSel.value, zone: zoneSel.select.value }),
  };
}
