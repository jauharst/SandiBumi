// Interactive Vega-Lite chart panel (feature V1 + V2). A well-bound plot rendered by the
// vega-embed engine — grammar-of-graphics interactivity (hover tooltips, drag-pan, scroll-zoom)
// the hand-rolled Canvas-2D plots don't give for free. Lazy-loaded (vega is large) so it stays out
// of the main bundle until the user opens a Vega chart; see workspace.createPlot.
//
// V2 scope: a control bar — chart type (scatter / line / histogram), X / Y curves, a colour curve
// (scatter only, viridis), and a zone filter (restrict to the selected zone's depth range). Live
// theme repaint, brush-linking and a spec editor are later increments.
import vegaEmbed, { type VisualizationSpec, type Result as VegaResult } from "vega-embed";
import { getCurveData, type TrackCurveSeries, type WellSummary } from "../ipc";
import { buildZoneSelect, curveSelect, loadCurveNames, trySelect, type PlotContent } from "./plotCommon";

type ChartType = "scatter" | "line" | "histogram";

interface Row {
  x: number;
  y?: number;
  z?: number;
  depth: number;
}

/** One CSS custom property off :root, with a fallback so the spec never carries an empty string. */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

const dKey = (d: number): number => Math.round(d * 1000); // mm resolution — depths are in metres

/** Join X/Y (and optional Z) curve series on shared depth into finite rows. Curves ride the same
 *  standard grid, but they are joined by depth (not index) so a curve with its own sampling still
 *  lines up; non-finite values are dropped so vega never sees NaN. */
function joinXYZ(series: TrackCurveSeries[], xName: string, yName: string, zName: string | null): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  const ys = series.find((s) => s.curve_name === yName);
  if (!xs || !ys) return [];
  const zs = zName ? (series.find((s) => s.curve_name === zName) ?? null) : null;
  const yByD = new Map<number, number>();
  for (let i = 0; i < ys.depth.length; i++) if (Number.isFinite(ys.value[i])) yByD.set(dKey(ys.depth[i]), ys.value[i]);
  const zByD = zs ? new Map<number, number>() : null;
  if (zs) for (let i = 0; i < zs.depth.length; i++) if (Number.isFinite(zs.value[i])) zByD!.set(dKey(zs.depth[i]), zs.value[i]);
  const out: Row[] = [];
  for (let i = 0; i < xs.depth.length; i++) {
    const xv = xs.value[i];
    if (!Number.isFinite(xv)) continue;
    const yv = yByD.get(dKey(xs.depth[i]));
    if (yv === undefined) continue;
    const row: Row = { x: xv, y: yv, depth: xs.depth[i] };
    if (zByD) {
      const zv = zByD.get(dKey(xs.depth[i]));
      if (zv !== undefined) row.z = zv;
    }
    out.push(row);
  }
  return out;
}

/** Finite X samples only — the data for a distribution (histogram) view. */
function xValues(series: TrackCurveSeries[], xName: string): Row[] {
  const xs = series.find((s) => s.curve_name === xName);
  if (!xs) return [];
  const out: Row[] = [];
  for (let i = 0; i < xs.depth.length; i++) if (Number.isFinite(xs.value[i])) out.push({ x: xs.value[i], depth: xs.depth[i] });
  return out;
}

/** A themed Vega-Lite spec for one chart type. Colours are pulled from the active theme's CSS vars
 *  at build time (live repaint on theme change lands in V3). Scatter/line get `params: interval
 *  bind:scales` for drag-pan / scroll-zoom; `width/height: container` makes vega track the panel. */
function buildSpec(type: ChartType, rows: Row[], xName: string, yName: string, zName: string | null): VisualizationSpec {
  const text = cssVar("--text", "#333333");
  const dim = cssVar("--text-dim", "#888888");
  const border = cssVar("--border", "#cccccc");
  const accent = cssVar("--accent", "#b5651d");
  const axis = { labelColor: dim, titleColor: text, gridColor: border, domainColor: border, tickColor: border };
  const base = {
    $schema: "https://vega.github.io/schema/vega-lite/v5.json",
    background: "transparent",
    width: "container",
    height: "container",
    autosize: { type: "fit", contains: "padding", resize: true },
    data: { values: rows },
    config: { background: "transparent", view: { stroke: border }, axis },
  };

  if (type === "histogram") {
    return {
      ...base,
      mark: { type: "bar", color: accent, opacity: 0.85 },
      encoding: {
        x: { field: "x", bin: true, type: "quantitative", title: xName, axis },
        y: { aggregate: "count", type: "quantitative", title: "count", axis },
        tooltip: [
          { field: "x", bin: true, type: "quantitative", title: xName },
          { aggregate: "count", type: "quantitative", title: "count" },
        ],
      },
    } as VisualizationSpec;
  }

  const encoding: Record<string, unknown> = {
    x: { field: "x", type: "quantitative", title: xName, scale: { zero: false }, axis },
    y: { field: "y", type: "quantitative", title: yName, scale: { zero: false }, axis },
    tooltip: [
      { field: "x", type: "quantitative", title: xName, format: ".3f" },
      { field: "y", type: "quantitative", title: yName, format: ".3f" },
      ...(zName ? [{ field: "z", type: "quantitative", title: zName, format: ".3f" }] : []),
      { field: "depth", type: "quantitative", title: "Depth", format: ".2f" },
    ],
  };
  if (zName) {
    encoding.color = {
      field: "z",
      type: "quantitative",
      title: zName,
      scale: { scheme: "viridis" },
      legend: { labelColor: dim, titleColor: text },
    };
  }
  if (type === "line") encoding.order = { field: "depth", type: "quantitative" };
  const mark =
    type === "line"
      ? { type: "line", strokeWidth: 1.3, opacity: 0.85, point: false, ...(zName ? {} : { color: accent }) }
      : { type: "point", filled: true, size: 20, opacity: 0.55, ...(zName ? {} : { color: accent }) };

  return { ...base, params: [{ name: "grid", select: "interval", bind: "scales" }], mark, encoding } as VisualizationSpec;
}

/** A labelled select for the control bar. */
function field(label: string, sel: HTMLSelectElement): HTMLElement {
  const l = document.createElement("label");
  l.className = "vega-field";
  const t = document.createElement("span");
  t.textContent = label;
  l.append(t, sel);
  return l;
}

/** Curve select with a leading "— None —" (empty value) for the optional colour channel. */
function colorSelect(names: string[], initial: string): HTMLSelectElement {
  const sel = document.createElement("select");
  sel.className = "form-control";
  const none = document.createElement("option");
  none.value = "";
  none.textContent = "— None —";
  sel.appendChild(none);
  for (const n of names) {
    const o = document.createElement("option");
    o.value = n;
    o.textContent = n;
    sel.appendChild(o);
  }
  sel.value = names.includes(initial) ? initial : "";
  return sel;
}

/** Build the well-bound Vega chart panel. Signature matches the other plot builders
 *  (`workspace.createPlot`), returning `{ el, dispose, getState }`. */
export async function buildVegaContent(
  well: WellSummary,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const curveNames = await loadCurveNames();
  const zoneSel = await buildZoneSelect(well);
  trySelect(zoneSel.select, initial?.zone);

  const container = document.createElement("div");
  container.className = "plot-content vega-panel";

  const typeSel = document.createElement("select");
  typeSel.className = "form-control";
  for (const [v, label] of [
    ["scatter", "Scatter"],
    ["line", "Line"],
    ["histogram", "Histogram"],
  ] as [ChartType, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    typeSel.appendChild(o);
  }
  typeSel.value = ["scatter", "line", "histogram"].includes(initial?.type ?? "") ? (initial!.type as ChartType) : "scatter";

  const xSel = curveSelect(curveNames, initial?.x ?? "NPHI");
  const ySel = curveSelect(curveNames, initial?.y ?? "RHOB");
  const zSel = colorSelect(curveNames, initial?.z ?? "");

  const toolbar = document.createElement("div");
  toolbar.className = "vega-toolbar";
  const yField = field("Y", ySel);
  const zField = field("Color", zSel);
  toolbar.append(field("Type", typeSel), field("X", xSel), yField, zField, field("Zone", zoneSel.select));

  const chartHost = document.createElement("div");
  chartHost.className = "vega-chart-host";
  container.append(toolbar, chartHost);

  // Y is irrelevant to a histogram; colour is meaningful only on a scatter. Dim the controls that
  // don't apply so the toolbar reads honestly for the active chart type.
  const syncControls = (): void => {
    const t = typeSel.value as ChartType;
    ySel.disabled = t === "histogram";
    zSel.disabled = t !== "scatter";
    yField.classList.toggle("vega-field-off", ySel.disabled);
    zField.classList.toggle("vega-field-off", zSel.disabled);
  };
  syncControls();

  let current: VegaResult | null = null;
  let disposed = false;
  let gen = 0;

  async function render(): Promise<void> {
    const myGen = ++gen;
    const type = typeSel.value as ChartType;
    const xName = xSel.value;
    const yName = ySel.value;
    const useZ = type === "scatter" && zSel.value ? zSel.value : null;
    const zc = zoneSel.current();
    const needed = type === "histogram" ? [xName] : useZ ? [xName, yName, useZ] : [xName, yName];
    setStatus(`Vega — loading ${needed.join(", ")}…`);
    let series: TrackCurveSeries[];
    try {
      series = await getCurveData(well.well_id, needed, zc.depthMin, zc.depthMax);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      chartHost.innerHTML = `<div class="logview-message">Failed to load curves: ${err}</div>`;
      setStatus("Vega — load failed");
      return;
    }
    if (disposed || myGen !== gen) return; // a newer render (or close) already won
    const rows = type === "histogram" ? xValues(series, xName) : joinXYZ(series, xName, yName, useZ);
    current?.finalize();
    current = null;
    chartHost.innerHTML = "";
    if (rows.length === 0) {
      const what = type === "histogram" ? xName : `${xName} / ${yName}`;
      chartHost.innerHTML = `<div class="logview-message">No finite ${what} samples in ${well.well_name}${zc.zoneName !== "*" ? ` · ${zc.zoneName}` : ""}.</div>`;
      setStatus("Vega — no data");
      return;
    }
    try {
      const result = await vegaEmbed(chartHost, buildSpec(type, rows, xName, yName, useZ), {
        actions: false,
        renderer: "canvas",
        tooltip: true,
      });
      if (disposed || myGen !== gen) {
        result.finalize();
        return;
      }
      current = result;
      const scope = zc.zoneName !== "*" ? ` · ${zc.zoneName}` : "";
      setStatus(`Vega — ${type}, ${rows.length.toLocaleString()} points${scope}`);
    } catch (err) {
      if (disposed || myGen !== gen) return;
      chartHost.innerHTML = `<div class="logview-message">Vega render failed: ${err}</div>`;
      setStatus("Vega — render failed");
    }
  }

  typeSel.addEventListener("change", () => {
    syncControls();
    void render();
  });
  for (const sel of [xSel, ySel, zSel, zoneSel.select]) sel.addEventListener("change", () => void render());

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
      zoneSel.dispose();
      current?.finalize();
      current = null;
    },
    getState: () => ({
      type: typeSel.value,
      x: xSel.value,
      y: ySel.value,
      z: zSel.value,
      zone: zoneSel.select.value,
    }),
  };
}
