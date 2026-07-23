// Interactive Vega-Lite chart panel (feature V1). A well-bound crossplot rendered by the
// vega-embed engine — grammar-of-graphics interactivity (hover tooltips, drag-pan, scroll-zoom)
// that the hand-rolled Canvas-2D plots don't give for free. Lazy-loaded (vega is large) so it
// stays out of the main bundle until the user opens a Vega chart; see workspace.createPlot.
//
// V1 scope: pick X/Y curves, plot the selected well, theme from the CSS vars. Zone filtering,
// more chart types, a spec editor, live theme repaint and brush-linking are later increments.
import vegaEmbed, { type VisualizationSpec, type Result as VegaResult } from "vega-embed";
import { getCurveData, type TrackCurveSeries, type WellSummary } from "../ipc";
import { curveSelect, loadCurveNames, type PlotContent } from "./plotCommon";

/** One CSS custom property off :root, with a fallback so the spec never carries an empty string. */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

interface XYPoint {
  x: number;
  y: number;
  depth: number;
}

/** Join two curve series on shared depth into finite {x, y, depth} rows. Curves ride the same
 *  standard grid, but they are joined by depth (not index) so a curve with its own sampling still
 *  lines up; non-finite pairs are dropped so vega never sees NaN. */
function joinXY(series: TrackCurveSeries[], xName: string, yName: string): XYPoint[] {
  const xs = series.find((s) => s.curve_name === xName);
  const ys = series.find((s) => s.curve_name === yName);
  if (!xs || !ys) return [];
  const key = (d: number): number => Math.round(d * 1000); // mm resolution — depths are in metres
  const yByDepth = new Map<number, number>();
  for (let i = 0; i < ys.depth.length; i++) yByDepth.set(key(ys.depth[i]), ys.value[i]);
  const out: XYPoint[] = [];
  for (let i = 0; i < xs.depth.length; i++) {
    const yv = yByDepth.get(key(xs.depth[i]));
    const xv = xs.value[i];
    if (yv === undefined || !Number.isFinite(xv) || !Number.isFinite(yv)) continue;
    out.push({ x: xv, y: yv, depth: xs.depth[i] });
  }
  return out;
}

/** A themed Vega-Lite scatter spec. Colours are pulled from the active theme's CSS vars at build
 *  time (live repaint on theme change lands in V3). `params: interval bind:scales` gives the
 *  drag-pan / scroll-zoom; `width/height: container` makes vega track the panel size. */
function buildSpec(points: XYPoint[], xName: string, yName: string): VisualizationSpec {
  const text = cssVar("--text", "#333333");
  const dim = cssVar("--text-dim", "#888888");
  const border = cssVar("--border", "#cccccc");
  const accent = cssVar("--accent", "#b5651d");
  const axis = {
    labelColor: dim,
    titleColor: text,
    gridColor: border,
    domainColor: border,
    tickColor: border,
  };
  return {
    $schema: "https://vega.github.io/schema/vega-lite/v5.json",
    background: "transparent",
    width: "container",
    height: "container",
    autosize: { type: "fit", contains: "padding", resize: true },
    data: { values: points },
    params: [{ name: "grid", select: "interval", bind: "scales" }],
    mark: { type: "point", filled: true, size: 20, opacity: 0.55, color: accent },
    encoding: {
      x: { field: "x", type: "quantitative", title: xName, scale: { zero: false }, axis },
      y: { field: "y", type: "quantitative", title: yName, scale: { zero: false }, axis },
      tooltip: [
        { field: "x", type: "quantitative", title: xName, format: ".3f" },
        { field: "y", type: "quantitative", title: yName, format: ".3f" },
        { field: "depth", type: "quantitative", title: "Depth", format: ".2f" },
      ],
    },
    config: {
      background: "transparent",
      view: { stroke: border },
      axis,
    },
  } as VisualizationSpec;
}

/** Build the well-bound Vega chart panel. Signature matches the other plot builders
 *  (`workspace.createPlot`), returning `{ el, dispose, getState }`. */
export async function buildVegaContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();

  const container = document.createElement("div");
  container.className = "plot-content vega-panel";

  const toolbar = document.createElement("div");
  toolbar.className = "vega-toolbar";
  const xSel = curveSelect(curveNames, initial?.x ?? "NPHI");
  const ySel = curveSelect(curveNames, initial?.y ?? "RHOB");
  const wrapSel = (label: string, sel: HTMLSelectElement): HTMLElement => {
    const l = document.createElement("label");
    l.className = "vega-field";
    const t = document.createElement("span");
    t.textContent = label;
    l.append(t, sel);
    return l;
  };
  toolbar.append(wrapSel("X", xSel), wrapSel("Y", ySel));

  const chartHost = document.createElement("div");
  chartHost.className = "vega-chart-host";
  container.append(toolbar, chartHost);

  let current: VegaResult | null = null;
  let disposed = false;
  let gen = 0;

  async function render(): Promise<void> {
    const myGen = ++gen;
    const xName = xSel.value;
    const yName = ySel.value;
    setStatus(`Vega — loading ${xName} vs ${yName}…`);
    let series: TrackCurveSeries[];
    try {
      series = await getCurveData(well.well_id, [xName, yName], null, null);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      chartHost.innerHTML = `<div class="logview-message">Failed to load curves: ${err}</div>`;
      setStatus("Vega — load failed");
      return;
    }
    if (disposed || myGen !== gen) return; // a newer render (or close) already won
    const points = joinXY(series, xName, yName);
    current?.finalize();
    current = null;
    chartHost.innerHTML = "";
    if (points.length === 0) {
      chartHost.innerHTML = `<div class="logview-message">No overlapping finite ${xName} / ${yName} samples in ${well.well_name}.</div>`;
      setStatus(`Vega — no ${xName}/${yName} data`);
      return;
    }
    try {
      const result = await vegaEmbed(chartHost, buildSpec(points, xName, yName), {
        actions: false,
        renderer: "canvas",
        tooltip: true,
      });
      if (disposed || myGen !== gen) {
        result.finalize();
        return;
      }
      current = result;
      setStatus(`Vega — ${points.length.toLocaleString()} points (${xName} vs ${yName})`);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      chartHost.innerHTML = `<div class="logview-message">Vega render failed: ${err}</div>`;
      setStatus("Vega — render failed");
    }
  }

  xSel.addEventListener("change", () => void render());
  ySel.addEventListener("change", () => void render());

  // vega's container sizing needs the host attached with a non-zero size, which only happens once
  // the dock appends this panel. Embed on the first non-zero measurement; vega tracks resizes after.
  let embedded = false;
  const ro = new ResizeObserver(() => {
    if (embedded || disposed) return;
    if (chartHost.clientWidth > 0 && chartHost.clientHeight > 0) {
      embedded = true;
      void render();
    }
  });
  ro.observe(chartHost);

  return {
    el: container,
    dispose: () => {
      disposed = true;
      ro.disconnect();
      current?.finalize();
      current = null;
    },
    getState: () => ({ x: xSel.value, y: ySel.value }),
  };
}
