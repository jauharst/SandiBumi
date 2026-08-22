import {
  checkContactConsistency,
  deleteFluidContact,
  getTrackData,
  listFluidContacts,
  listTops,
  listWells,
  plotBindingSnapshotForChannels,
  resolvePlotBindings,
  resolveWellScope,
  suggestContacts,
  upsertFluidContact,
  type ContactCandidate,
  type ContactConsistency,
  type ContactSuggestResult,
  type FluidContact,
  type PlotChannelBinding,
  type TopEntry,
  type TrackCurveSeries,
  type WellSummary,
} from "../ipc";
import { appState, type BrushSelection, type TopInterval } from "../state";
import { openModal } from "./modal";
import { shownDepthLabel, toShownDepth } from "../depthUnitPref";
import {
  attachAccessiblePlotKeyboard,
  buildPlotStatisticsRecord,
  canvasFont,
  formatPlotStatisticsRecord,
  makeCanvasAccessible,
  percentile,
  plotStatisticsInterval,
  readTheme,
  type PlotStatisticsRecord,
} from "./plotCanvas";
import { buildPersistedPlotState, curveSelect, loadCurveNames, loadPlotProps, savePlotProps, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";
import {
  axisRangeExportRecord,
  formatAxisRangeSummary,
  resolveBoundAxisRange,
  type AxisDisplayRange,
  type AxisRangeResolution,
  type PlotAxisRangeExport,
} from "./axisRange";
import { applyPlotRangePolicy, formatPlotRangePolicySummary, type PlotRangePolicyReport } from "./plotRangePolicy";
import { registerPlotInvalidationContract } from "./plotInvalidation";
import { beginPlotAsyncGeneration, isPlotAsyncGenerationCurrent } from "./plotAsync";

/** Default marker colors per contact type (a stored color overrides these). */
const CONTACT_COLORS: Record<string, string> = {
  OWC: "#2f6fed",
  GWC: "#e0483d",
  GOC: "#e08a1e",
  GDT: "#b58a2b",
  ODT: "#3b7a57",
  FWL: "#8e44ad",
};
const CONTACT_TYPES = ["OWC", "GWC", "GOC", "GDT", "ODT", "FWL"];

/** Multi-well correlation view (well-correlation view): the included
 *  wells drawn as side-by-side strips of one shared curve, formation tops connected
 *  between adjacent wells, optionally flattened on a datum top. */

export interface CorrelationOptions {
  curve: string;
  /** Shared strip scale; null = auto (global P2–P98 across the included wells). */
  min: number | null;
  max: number | null;
  /** Top name to flatten on; "" = measured depth. */
  datum: string;
  /** Depth axis: measured depth, or true vertical depth subsea (contacts are flat in TVDSS). */
  depthMode: "md" | "tvdss";
  /** Draw fluid contacts (OWC/GWC/…) as horizontal lines across the strips. */
  showContacts: boolean;
  /** Scientific validity is opt-in and applies to curve values, never the depth display. */
  validityFilter: boolean;
  validMin: number | null;
  validMax: number | null;
}

async function listActiveScopedWells(): Promise<WellSummary[]> {
  const [all, ids] = await Promise.all([listWells(), resolveWellScope({ kind: "active_group" })]);
  const allowed = new Set(ids);
  return all.filter((well) => allowed.has(well.well_id));
}

export const DEFAULT_CORRELATION_OPTIONS: CorrelationOptions = {
  curve: "GR",
  min: null,
  max: null,
  datum: "",
  depthMode: "md",
  showContacts: true,
  validityFilter: false,
  validMin: null,
  validMax: null,
};

function correlationValidityRange(opts: CorrelationOptions): AxisDisplayRange | null {
  return opts.validMin !== null && opts.validMax !== null && opts.validMin !== opts.validMax
    ? { min: opts.validMin, max: opts.validMax }
    : null;
}

export function screenCorrelationPopulation(
  values: ArrayLike<number>,
  opts: CorrelationOptions,
  valueDisplay: AxisDisplayRange | null,
  displayDepths: ArrayLike<number> | null = null,
  depthDisplay: AxisDisplayRange | null = null,
): PlotRangePolicyReport {
  return applyPlotRangePolicy([
    { values, display: valueDisplay, validity: correlationValidityRange(opts) },
    ...(displayDepths ? [{ values: displayDepths, display: depthDisplay, validity: null }] : []),
  ], opts.validityFilter);
}

/** Live Correlation adapter: one active-well or pooled record over aligned value/depth rows. */
export function buildCorrelationStatisticsRecord(
  values: ArrayLike<number>,
  displayDepths: ArrayLike<number>,
  opts: CorrelationOptions,
  wellIds: string[],
  valueDisplay: AxisDisplayRange | null,
  depthDisplay: AxisDisplayRange | null,
): PlotStatisticsRecord | null {
  if (wellIds.length === 0) return null;
  const policy = screenCorrelationPopulation(values, opts, valueDisplay, displayDepths, depthDisplay);
  if (policy.analysisCount === 0) return null;
  return buildPlotStatisticsRecord(
    Float32Array.from(policy.indices.map((index) => values[index])),
    {
      binding_channel: "value",
      channel: `value:${opts.curve}`,
      population: wellIds.length === 1 ? "active_well" : "pooled",
      well_ids: wellIds,
      interval: plotStatisticsInterval(null, null),
      selection: {
        kind: "all_eligible",
        selection_id: null,
        label: wellIds.length === 1 ? "all eligible" : "all eligible in included wells",
        applied: false,
      },
      policy,
      selection_excluded: 0,
      unpaired_or_unclassified_excluded: 0,
      standard_deviation: "sample_n_minus_one",
    },
  );
}

/** Finite (MD, TVDSS) pairs for one well, ascending in MD (hence in TVDSS). */
interface TvdssMap {
  md: Float64Array;
  ss: Float64Array;
}

interface WellStrip {
  well: WellSummary;
  series: TrackCurveSeries | null;
  tops: TopEntry[];
  /** MD→TVDSS lookup built from the well's TVDSS curve; null means no declared frame. */
  tv: TvdssMap | null;
  /** Display depth = displayOf(MD) - shift (flattening); 0 when the well lacks the datum top. */
  shift: number;
  hasDatum: boolean;
}

/** Linear interpolation on an ascending x-grid, clamped flat beyond the ends. */
function interpAsc(xs: Float64Array, ys: Float64Array, x: number): number {
  const n = xs.length;
  if (n === 0) return x;
  if (x <= xs[0]) return ys[0];
  if (x >= xs[n - 1]) return ys[n - 1];
  let lo = 0;
  let hi = n - 1;
  while (hi - lo > 1) {
    const m = (lo + hi) >> 1;
    if (xs[m] <= x) lo = m;
    else hi = m;
  }
  const t = (x - xs[lo]) / (xs[hi] - xs[lo]);
  return ys[lo] + t * (ys[hi] - ys[lo]);
}

/** Builds the finite (MD, TVDSS) lookup for a well from its TVDSS curve, or null if unusable. */
function buildTvdssMap(series: TrackCurveSeries | null): TvdssMap | null {
  if (!series) return null;
  const md: number[] = [];
  const ss: number[] = [];
  for (let i = 0; i < series.depth.length; i++) {
    const d = series.depth[i];
    const v = series.value[i];
    if (Number.isFinite(d) && Number.isFinite(v)) {
      md.push(d);
      ss.push(v);
    }
  }
  return md.length >= 2 ? { md: Float64Array.from(md), ss: Float64Array.from(ss) } : null;
}

const AXIS_W = 52;
const HEADER_H = 30;

export async function buildCorrelationContent(
  _well: WellSummary | null,
  setStatus: (text: string) => void,
  initial?: Record<string, string>,
): Promise<PlotContent> {
  const opts: CorrelationOptions = { ...DEFAULT_CORRELATION_OPTIONS, ...(await loadPlotProps<CorrelationOptions>("correlation")) };
  if (!correlationValidityRange(opts)) {
    opts.validityFilter = false;
    opts.validMin = null;
    opts.validMax = null;
  } else {
    opts.validityFilter = !!opts.validityFilter;
  }
  if (initial) {
    if (initial.curve) opts.curve = initial.curve;
    if (initial.datum !== undefined) opts.datum = initial.datum;
    if (initial.depthMode === "md" || initial.depthMode === "tvdss") opts.depthMode = initial.depthMode;
    if (initial.min !== undefined) opts.min = initial.min === "" ? null : Number(initial.min);
    if (initial.max !== undefined) opts.max = initial.max === "" ? null : Number(initial.max);
    if (initial.showContacts !== undefined) opts.showContacts = initial.showContacts === "1";
  }

  let wells: WellSummary[] = [];
  try {
    wells = await listActiveScopedWells();
  } catch {
    wells = [];
  }
  const included = new Set(wells.map((w) => w.well_id));
  let strips: WellStrip[] = [];
  const plotIntents = () => [
    { channel: "value", semantic_request: opts.curve, required: true },
    ...(opts.depthMode === "tvdss"
      ? [{ channel: "depth", semantic_request: "TVDSS", required: true }]
      : []),
  ];
  const selectionState = (): Record<string, string> => ({
    curve: opts.curve,
    min: opts.min?.toString() ?? "",
    max: opts.max?.toString() ?? "",
    datum: opts.datum,
    depthMode: opts.depthMode,
    showContacts: opts.showContacts ? "1" : "",
    depthMin: depthViewIsUser ? String(viewTop) : "",
    depthMax: depthViewIsUser ? String(viewTop + Math.max(50, canvas.clientHeight - HEADER_H) / pxPerUnit) : "",
  });
  let axisRanges: PlotAxisRangeExport[] = [];
  let statisticsRecords: PlotStatisticsRecord[] = [];
  let statisticsSignature = "";
  let statisticsDataVersion = 0;
  const currentValueBinding = (): PlotChannelBinding | null =>
    plotBindingSnapshotForChannels(
      strips.filter((strip) => strip.series !== null).map((strip) => strip.well.well_id),
      plotIntents(),
    ).find((binding) => binding.intent.channel === "value") ?? null;
  const persistedState = (options: Record<string, unknown>) =>
    buildPersistedPlotState(
      "correlation",
      options,
      strips.filter((strip) => strip.series !== null).map((strip) => strip.well.well_id),
      plotIntents(),
      axisRanges,
    );
  const persist = () => {
    try {
      void savePlotProps("correlation", persistedState({ ...opts }))
        .catch((error) => setStatus(`Correlation state not saved: ${error}`));
    } catch (error) {
      setStatus(`Correlation state not saved: ${error}`);
    }
  };
  // A physical wheel gesture emits a burst of events. Coalesce that burst into one durable
  // plot-state write; the rendered viewport and exported custody still update on every frame.
  // This delay is UI event coalescing only, not a scientific or petrophysical parameter.
  let wheelPersistTimer: ReturnType<typeof setTimeout> | null = null;
  const scheduleWheelPersist = (): void => {
    if (wheelPersistTimer !== null) clearTimeout(wheelPersistTimer);
    wheelPersistTimer = setTimeout(() => {
      wheelPersistTimer = null;
      persist();
    }, 180);
  };
  let curveNames: string[] = [];
  try {
    curveNames = await loadCurveNames();
  } catch {
    curveNames = [opts.curve];
  }

  // --- DOM scaffold ---
  const el = document.createElement("div");
  el.className = "correlation-panel";
  const props = document.createElement("div");
  props.className = "plot-props";
  const rangeInfo = document.createElement("p");
  rangeInfo.className = "modal-hint";
  rangeInfo.style.whiteSpace = "pre-wrap";
  const canvasHost = document.createElement("div");
  canvasHost.className = "correlation-canvas-host";
  const canvas = document.createElement("canvas");
  canvas.className = "plot-canvas";
  canvasHost.appendChild(canvas);
  el.appendChild(props);
  el.appendChild(rangeInfo);
  el.appendChild(canvasHost);

  // --- View state (display-depth space) ---
  let viewTop = 0;
  let pxPerUnit = 1;
  let depthViewIsUser = false;
  const restoredDepthView = (() => {
    if (!initial?.depthMin || !initial?.depthMax) return null;
    const min = Number(initial.depthMin);
    const max = Number(initial.depthMax);
    return Number.isFinite(min) && Number.isFinite(max) && min !== max ? { min, max } : null;
  })();
  let hoverY: number | null = null;
  /** All fluid contacts in the project; each strip renders the ones that apply to it. */
  let contacts: FluidContact[] = [];
  let selectedInterval: TopInterval | null = appState.selectedInterval.get();
  let brushSelection: BrushSelection | null = appState.brushedDepths.get();

  // --- Depth-mode helpers: measured depth vs TVDSS -----------------------------------------
  /** MD → TVDSS via the well's declared TVDSS curve. Absence is never a vertical-well claim. */
  const mdToTvdss = (s: WellStrip, md: number): number | null =>
    s.tv ? interpAsc(s.tv.md, s.tv.ss, md) : null;
  /** TVDSS → MD (inverse of the above; TVDSS rises monotonically with MD). */
  const tvdssToMd = (s: WellStrip, ss: number): number | null =>
    s.tv ? interpAsc(s.tv.ss, s.tv.md, ss) : null;
  /** Raw display depth (before flattening) for a measured depth, in the active depth mode. */
  const displayOf = (s: WellStrip, md: number): number | null =>
    opts.depthMode === "tvdss" ? mdToTvdss(s, md) : md;
  /** A contact's display depth (after flattening) inside one strip. A TVDSS contact in TVDSS
   *  mode round-trips back to its own depth for every well → the line is perfectly flat. */
  const contactDisplay = (s: WellStrip, c: FluidContact): number | null => {
    const datum = c.depth_datum ?? (c.is_tvdss ? "TVDSS" : "MD");
    if (datum !== "MD" && datum !== "TVDSS") return null;
    const md = datum === "TVDSS" ? tvdssToMd(s, c.depth) : c.depth;
    if (md === null) return null;
    const display = displayOf(s, md);
    return display === null ? null : display - s.shift;
  };
  /** Whether a contact applies to a well: explicit well, else field, else global. */
  const contactApplies = (c: FluidContact, well: WellSummary): boolean => {
    if (c.well_id) return c.well_id === well.well_id;
    if (c.field_name) return c.field_name === well.field_name;
    return true;
  };
  const contactColor = (c: FluidContact): string => c.color || CONTACT_COLORS[c.contact_type] || "#888";

  const displayExtent = (): [number, number] | null => {
    let lo = Infinity;
    let hi = -Infinity;
    for (const s of strips) {
      if (!s.series || s.series.depth.length === 0) continue;
      const aRaw = displayOf(s, s.series.depth[0]);
      const bRaw = displayOf(s, s.series.depth[s.series.depth.length - 1]);
      if (aRaw === null || bRaw === null) continue;
      const a = aRaw - s.shift;
      const b = bRaw - s.shift;
      lo = Math.min(lo, a, b);
      hi = Math.max(hi, a, b);
    }
    return lo < hi ? [lo, hi] : null;
  };

  const fit = () => {
    const extent = displayExtent();
    if (!extent) return;
    const [lo, hi] = extent;
    const h = Math.max(50, canvas.clientHeight - HEADER_H);
    pxPerUnit = h / (hi - lo);
    viewTop = lo;
    depthViewIsUser = false;
    draw();
    persist();
  };

  /** One global strip scale through the SB-PLT-002 precedence chain. */
  const stripScale = (): AxisRangeResolution | null => {
    const pool: number[] = [];
    for (const s of strips) {
      if (!included.has(s.well.well_id) || !s.series) continue;
      const screened = screenCorrelationPopulation(s.series.value, opts, null);
      for (const index of screened.indices) pool.push(s.series.value[index]);
    }
    const finiteData = pool.length >= 2
      ? { min: percentile(pool, 2), max: percentile(pool, 98) }
      : null;
    return resolveBoundAxisRange({
      binding: currentValueBinding(),
      user: opts.min !== null && opts.max !== null ? { min: opts.min, max: opts.max } : null,
      finiteData,
      validity: correlationValidityRange(opts),
    });
  };

  /** Nice tick step so depth labels sit ≥ 45px apart. */
  const tickStep = (): number => {
    const target = 45 / pxPerUnit;
    const pow = Math.pow(10, Math.floor(Math.log10(target)));
    for (const m of [1, 2, 5, 10]) {
      if (m * pow >= target) return m * pow;
    }
    return 10 * pow;
  };

  function draw(): void {
    makeCanvasAccessible(
      canvas,
      `Correlation: ${opts.curve} across ${included.size} included well${included.size === 1 ? "" : "s"}, ${opts.depthMode.toUpperCase()} depth${opts.datum ? ` flattened on ${opts.datum}` : ""}`,
    );
    const dpr = window.devicePixelRatio || 1;
    const w = canvasHost.clientWidth;
    const h = canvasHost.clientHeight;
    if (w === 0 || h === 0) return;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const theme = readTheme(el);
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);

    const active = strips.filter((s) => included.has(s.well.well_id));
    if (active.length === 0) {
      axisRanges = [];
      statisticsRecords = [];
      statisticsSignature = "";
      rangeInfo.textContent = "";
      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 13);
      ctx.fillText("No wells included — pick some under Wells…", AXIS_W + 10, 40);
      return;
    }

    const plotH = h - HEADER_H;
    const yOf = (disp: number) => HEADER_H + (disp - viewTop) * pxPerUnit;
    const valueRange = stripScale();
    const finiteDepth = displayExtent();
    const depthRange = resolveBoundAxisRange({
      binding: null,
      user: depthViewIsUser ? { min: viewTop, max: viewTop + plotH / pxPerUnit } : null,
      finiteData: finiteDepth ? { min: finiteDepth[0], max: finiteDepth[1] } : null,
    });
    if (!valueRange || !depthRange) {
      axisRanges = [];
      statisticsRecords = [];
      statisticsSignature = "";
      rangeInfo.textContent = "Axis range unavailable: this view has no complete user, header, audited-family, or finite-data range.";
      return;
    }
    const vMin = valueRange.min;
    const vMax = valueRange.max;
    axisRanges = [
      axisRangeExportRecord("value", valueRange),
      axisRangeExportRecord("depth", depthRange),
    ];
    const populationValues: number[] = [];
    const populationDepths: number[] = [];
    const populationWellIds: string[] = [];
    for (const strip of active) {
      if (!strip.series || (opts.depthMode === "tvdss" && !strip.tv)) continue;
      populationWellIds.push(strip.well.well_id);
      for (let sample = 0; sample < strip.series.value.length; sample++) {
        populationValues.push(strip.series.value[sample]);
        const displayed = displayOf(strip, strip.series.depth[sample]);
        populationDepths.push(displayed === null ? Number.NaN : displayed - strip.shift);
      }
    }
    const population = screenCorrelationPopulation(
      populationValues,
      opts,
      { min: valueRange.min, max: valueRange.max },
      populationDepths,
      { min: depthRange.min, max: depthRange.max },
    );
    const statisticsKey = JSON.stringify([
      statisticsDataVersion,
      opts.curve,
      opts.validityFilter,
      opts.validMin,
      opts.validMax,
      valueRange.min,
      valueRange.max,
      depthRange.min,
      depthRange.max,
      populationWellIds,
      active.map((strip) => strip.shift),
    ]);
    if (statisticsKey !== statisticsSignature) {
      const record = buildCorrelationStatisticsRecord(
        populationValues,
        populationDepths,
        opts,
        populationWellIds,
        { min: valueRange.min, max: valueRange.max },
        { min: depthRange.min, max: depthRange.max },
      );
      statisticsRecords = record ? [record] : [];
      statisticsSignature = statisticsKey;
    }
    const statisticsText = statisticsRecords.length > 0
      ? `\n${statisticsRecords.map((record) => formatPlotStatisticsRecord(record)).join("\n")}`
      : "\nNo governed statistics population.";
    rangeInfo.textContent = `${formatAxisRangeSummary(axisRanges)} · ${formatPlotRangePolicySummary(population, { statistics: true })}${statisticsText}`;
    const slot = (w - AXIS_W) / active.length;
    const gap = Math.min(46, slot * 0.28);
    const stripW = slot - gap;
    const stripLeft = (i: number) => AXIS_W + i * slot + gap / 2;

    // Depth axis (display depth: MD, or relative to the datum when flattened).
    ctx.strokeStyle = theme.grid;
    ctx.fillStyle = theme.text;
    ctx.font = canvasFont(theme, 10);
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    const step = tickStep();
    const first = Math.ceil(viewTop / step) * step;
    for (let d = first; yOf(d) < h; d += step) {
      const y = yOf(d);
      if (y < HEADER_H) continue;
      ctx.beginPath();
      ctx.moveTo(AXIS_W - 4, y);
      ctx.lineTo(w, y);
      ctx.globalAlpha = 0.35;
      ctx.stroke();
      ctx.globalAlpha = 1;
      ctx.fillText(String(Math.round(d)), AXIS_W - 7, y);
    }

    // Flattening datum line at display depth 0.
    if (opts.datum) {
      const y = yOf(0);
      if (y >= HEADER_H && y <= h) {
        ctx.strokeStyle = theme.accent;
        ctx.setLineDash([6, 4]);
        ctx.beginPath();
        ctx.moveTo(AXIS_W, y);
        ctx.lineTo(w, y);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }

    // Strips: frame, header, curve.
    active.forEach((s, i) => {
      const left = stripLeft(i);
      ctx.strokeStyle = theme.axis;
      ctx.strokeRect(left, HEADER_H, stripW, plotH);

      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 11, 600);
      ctx.textAlign = "center";
      ctx.textBaseline = "alphabetic";
      const label = opts.depthMode === "tvdss" && !s.tv
        ? `${s.well.well_name} (no TVDSS frame)`
        : opts.datum && !s.hasDatum
          ? `${s.well.well_name} (no datum)`
          : s.well.well_name;
      ctx.fillText(label, left + stripW / 2, 12, stripW + gap - 6);
      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 10);
      ctx.fillText(opts.curve, left + stripW / 2, 24, stripW - 4);

      if (!s.series || s.series.depth.length === 0 || (opts.depthMode === "tvdss" && !s.tv)) return;
      const sampleDepths = Float32Array.from(s.series.depth, (depth) => {
        const displayed = displayOf(s, depth);
        return displayed === null ? Number.NaN : displayed - s.shift;
      });
      const screened = screenCorrelationPopulation(
        s.series.value,
        opts,
        { min: valueRange.min, max: valueRange.max },
        sampleDepths,
        { min: depthRange.min, max: depthRange.max },
      );
      const eligible = new Set(screened.indices);
      ctx.save();
      ctx.beginPath();
      ctx.rect(left, HEADER_H, stripW, plotH);
      ctx.clip();

      // A selected top interval is an application-level plot invalidation. Correlation keeps its
      // multi-well viewport and marks that exact well's MD interval instead of silently ignoring it.
      if (selectedInterval?.wellId === s.well.well_id) {
        const intervalBottom = selectedInterval.depthMax
          ?? s.series.depth[s.series.depth.length - 1];
        const topDisplay = displayOf(s, selectedInterval.depthMin);
        const bottomDisplay = displayOf(s, intervalBottom);
        if (topDisplay !== null && bottomDisplay !== null) {
          const y0 = yOf(topDisplay - s.shift);
          const y1 = yOf(bottomDisplay - s.shift);
          ctx.fillStyle = theme.series1;
          ctx.globalAlpha = 0.09;
          ctx.fillRect(left, Math.min(y0, y1), stripW, Math.abs(y1 - y0));
          ctx.globalAlpha = 1;
        }
      }

      ctx.strokeStyle = theme.series2;
      ctx.lineWidth = 1;
      ctx.beginPath();
      let pen = false;
      for (let k = 0; k < s.series.depth.length; k++) {
        const v = s.series.value[k];
        const displayDepth = sampleDepths[k];
        const displayHidden = v < Math.min(vMin, vMax) || v > Math.max(vMin, vMax)
          || displayDepth < Math.min(depthRange.min, depthRange.max)
          || displayDepth > Math.max(depthRange.min, depthRange.max);
        if (!eligible.has(k) || displayHidden) {
          pen = false;
          continue;
        }
        const frac = (v - vMin) / (vMax - vMin);
        const x = left + frac * stripW;
        const y = yOf(displayDepth);
        if (pen) ctx.lineTo(x, y);
        else ctx.moveTo(x, y);
        pen = true;
      }
      ctx.stroke();

      // Linked selection is exact depth membership on the selected well. Rings make the redraw
      // observable without changing the curve, source values, viewport or multi-well population.
      if (brushSelection?.wellId === s.well.well_id && brushSelection.depths.size > 0) {
        ctx.strokeStyle = theme.accent;
        ctx.lineWidth = 1.5;
        for (let k = 0; k < s.series.depth.length; k++) {
          if (!brushSelection.depths.has(s.series.depth[k]) || !eligible.has(k)) continue;
          const v = s.series.value[k];
          const displayDepth = sampleDepths[k];
          const displayHidden = v < Math.min(vMin, vMax) || v > Math.max(vMin, vMax)
            || displayDepth < Math.min(depthRange.min, depthRange.max)
            || displayDepth > Math.max(depthRange.min, depthRange.max);
          if (displayHidden) continue;
          const x = left + ((v - vMin) / (vMax - vMin)) * stripW;
          const y = yOf(displayDepth);
          ctx.beginPath();
          ctx.arc(x, y, 2.5, 0, Math.PI * 2);
          ctx.stroke();
        }
      }
      ctx.restore();
    });

    // Tops: marker line inside each strip, then connectors between adjacent wells.
    const topY = (s: WellStrip, name: string): number | null => {
      const top = s.tops.find((t) => t.top_name === name);
      if (!top) return null;
      const display = displayOf(s, top.depth);
      if (display === null) return null;
      const y = yOf(display - s.shift);
      return y >= HEADER_H && y <= h ? y : null;
    };
    const allTopNames = Array.from(new Set(active.flatMap((s) => s.tops.map((t) => t.top_name))));
    ctx.lineWidth = 1.5;
    ctx.font = canvasFont(theme, 10);
    ctx.textBaseline = "bottom";
    for (const name of allTopNames) {
      const color = active.flatMap((s) => s.tops).find((t) => t.top_name === name)?.color || theme.warn;
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      let labeled = false;
      active.forEach((s, i) => {
        const y = topY(s, name);
        if (y === null) return;
        const left = stripLeft(i);
        ctx.beginPath();
        ctx.moveTo(left, y);
        ctx.lineTo(left + stripW, y);
        ctx.stroke();
        if (!labeled) {
          ctx.textAlign = "left";
          ctx.fillText(name, left + 2, y - 1);
          labeled = true;
        }
      });
      // Dashed connectors bridge the gaps between adjacent strips that both have the top.
      ctx.setLineDash([4, 3]);
      for (let i = 0; i + 1 < active.length; i++) {
        const y1 = topY(active[i], name);
        const y2 = topY(active[i + 1], name);
        if (y1 === null || y2 === null) continue;
        ctx.beginPath();
        ctx.moveTo(stripLeft(i) + stripW, y1);
        ctx.lineTo(stripLeft(i + 1), y2);
        ctx.stroke();
      }
      ctx.setLineDash([]);
    }

    // Fluid contacts: solid horizontal markers across each applicable strip, with a small
    // triangle at the left edge and dashed cross-well connectors. A TVDSS contact drawn in
    // TVDSS mode round-trips to its own depth in every well, so its line is perfectly flat.
    if (opts.showContacts && contacts.length) {
      ctx.font = canvasFont(theme, 10, 600);
      ctx.textBaseline = "bottom";
      for (const c of contacts) {
        if (!active.some((s) => contactApplies(c, s.well))) continue;
        const color = contactColor(c);
        ctx.strokeStyle = color;
        ctx.fillStyle = color;
        const ys = active.map((s) => {
          if (!contactApplies(c, s.well)) return null;
          const display = contactDisplay(s, c);
          if (display === null) return null;
          const y = yOf(display);
          return y >= HEADER_H && y <= h ? y : null;
        });
        let labeled = false;
        active.forEach((_s, i) => {
          const y = ys[i];
          if (y === null) return;
          const left = stripLeft(i);
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(left, y);
          ctx.lineTo(left + stripW, y);
          ctx.stroke();
          // Left-edge triangle marker distinguishes contacts from tops.
          ctx.beginPath();
          ctx.moveTo(left, y - 4);
          ctx.lineTo(left, y + 4);
          ctx.lineTo(left + 7, y);
          ctx.closePath();
          ctx.fill();
          if (!labeled) {
            ctx.textAlign = "left";
            const lbl = c.label || `${c.contact_type} ${Math.round(c.depth)}${c.is_tvdss ? "ss" : ""}`;
            ctx.fillText(lbl, left + 9, y - 1);
            labeled = true;
          }
        });
        ctx.setLineDash([5, 3]);
        ctx.lineWidth = 1.2;
        for (let i = 0; i + 1 < active.length; i++) {
          const y1 = ys[i];
          const y2 = ys[i + 1];
          if (y1 === null || y2 === null) continue;
          ctx.beginPath();
          ctx.moveTo(stripLeft(i) + stripW, y1);
          ctx.lineTo(stripLeft(i + 1), y2);
          ctx.stroke();
        }
        ctx.setLineDash([]);
      }
    }

    // Hover crosshair.
    if (hoverY !== null && hoverY > HEADER_H) {
      ctx.strokeStyle = theme.accent;
      ctx.globalAlpha = 0.75;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(AXIS_W, hoverY);
      ctx.lineTo(w, hoverY);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
    }
  }

  // --- Data loading ---
  // Monotonic token: a rapid well-toggle / curve change / dataVersion bump can leave an
  // older Promise.all in flight; whichever reload started last wins, so a stale set of
  // strips can't replace the current one. reload() preserves the pan/zoom viewport.
  let reloadGen = 0;
  async function reload(): Promise<boolean> {
    const token = beginPlotAsyncGeneration("correlation-data-refetch", ++reloadGen);
    const chosen = wells.filter((w) => included.has(w.well_id));
    if (chosen.length === 0) {
      strips = [];
      return true;
    }
    try {
      await resolvePlotBindings(plotIntents(), {
        kind: "explicit",
        well_ids: chosen.map((well) => well.well_id),
      });
    } catch (error) {
      if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return false;
      throw error;
    }
    if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return false;
    // TVDSS rides along in the same batch read so a TVDSS-mode switch needs no refetch.
    const names = Array.from(new Set([opts.curve, "TVDSS"]));
    const [loaded, loadedContacts] = await Promise.all([
      Promise.all(
        chosen.map(async (well): Promise<{ strip: WellStrip; topsError: string | null }> => {
          let series: TrackCurveSeries | null = null;
          let tv: TvdssMap | null = null;
          let tops: TopEntry[] = [];
          let topsError: string | null = null;
          try {
            const data = await getTrackData(well.well_id, names, 1400);
            series = data.find((s) => s.curve_name === opts.curve) ?? null;
            tv = buildTvdssMap(data.find((s) => s.curve_name === "TVDSS") ?? null);
          } catch {
            series = null;
          }
          try {
            tops = await listTops(well.well_id);
          } catch (err) {
            tops = [];
            topsError = String(err);
          }
          return { strip: { well, series, tops, tv, shift: 0, hasDatum: false }, topsError };
        }),
      ),
      listFluidContacts().catch(() => [] as FluidContact[]),
    ]);
    if (!isPlotAsyncGenerationCurrent(token, reloadGen)) return false;
    const topsError = loaded.find((item) => item.topsError)?.topsError;
    if (topsError) setStatus(`Correlation tops unavailable: ${topsError}`);
    strips = loaded.map((item) => item.strip);
    statisticsDataVersion++;
    contacts = loadedContacts;
    applyDatum();
    refreshDatumChoices();
    return true;
  }
  const reloadAndDraw = (): void => {
    void reload()
      .then((applied) => {
        if (!applied) return;
        draw();
        persist();
      })
      .catch((error) => {
        strips = [];
        setStatus(`Correlation refused: ${error}`);
        draw();
      });
  };

  /** Re-fetches the well list so the Wells menu and strips track the current project after
   *  an import, delete, or active-group change — reload() alone only re-reads curves for the
   *  wells already included, so a freshly imported well never appeared. New wells join the
   *  included set (they show as strips immediately); wells that no longer exist drop out. */
  let wellsGen = 0;
  async function refreshWells(): Promise<boolean> {
    const token = beginPlotAsyncGeneration("correlation-well-refetch", ++wellsGen);
    let latest: WellSummary[];
    try {
      latest = await listActiveScopedWells();
    } catch {
      // Keep the current list if the fetch fails, but let the winning data-revision path
      // continue to reload curves for that current inventory.
      return isPlotAsyncGenerationCurrent(token, wellsGen);
    }
    if (!isPlotAsyncGenerationCurrent(token, wellsGen)) return false;
    const known = new Set(wells.map((w) => w.well_id));
    const live = new Set(latest.map((w) => w.well_id));
    for (const w of latest) if (!known.has(w.well_id)) included.add(w.well_id);
    for (const id of Array.from(included)) if (!live.has(id)) included.delete(id);
    wells = latest;
    refreshWellsBtn();
    return true;
  }

  /** Recomputes per-well flattening shifts from the chosen datum top. */
  function applyDatum(): void {
    for (const s of strips) {
      const top = opts.datum ? s.tops.find((t) => t.top_name === opts.datum) : undefined;
      const display = top ? displayOf(s, top.depth) : null;
      s.hasDatum = !!top && display !== null;
      // Shift is in display space, so re-derive it whenever the depth mode changes too.
      s.shift = display ?? 0;
    }
  }

  // --- Property row ---
  const wellsBtn = document.createElement("button");
  wellsBtn.className = "form-control";
  const refreshWellsBtn = () => {
    wellsBtn.textContent = `Wells (${included.size}/${wells.length})…`;
  };
  refreshWellsBtn();
  let wellsMenu: HTMLElement | null = null;
  let wellsMenuClose: ((event: MouseEvent) => void) | null = null;
  let wellsMenuAttachTimer: ReturnType<typeof setTimeout> | null = null;
  const removeWellsMenu = (): void => {
    if (wellsMenuAttachTimer !== null) {
      clearTimeout(wellsMenuAttachTimer);
      wellsMenuAttachTimer = null;
    }
    if (wellsMenuClose) {
      document.removeEventListener("mousedown", wellsMenuClose);
      wellsMenuClose = null;
    }
    wellsMenu?.remove();
    wellsMenu = null;
  };
  wellsBtn.addEventListener("click", () => {
    removeWellsMenu();
    const menu = document.createElement("div");
    wellsMenu = menu;
    menu.className = "dock-add-menu";
    const rect = wellsBtn.getBoundingClientRect();
    menu.style.left = `${rect.left}px`;
    menu.style.top = `${rect.bottom + 2}px`;
    for (const well of wells) {
      const row = document.createElement("label");
      row.className = "well-check";
      const box = document.createElement("input");
      box.type = "checkbox";
      box.checked = included.has(well.well_id);
      box.addEventListener("change", () => {
        if (box.checked) included.add(well.well_id);
        else included.delete(well.well_id);
        refreshWellsBtn();
        reloadAndDraw();
      });
      row.appendChild(box);
      row.appendChild(document.createTextNode(well.well_name));
      menu.appendChild(row);
    }
    document.body.appendChild(menu);
    const close = (e: MouseEvent) => {
      if (!menu.contains(e.target as Node) && e.target !== wellsBtn) {
        removeWellsMenu();
      }
    };
    wellsMenuClose = close;
    wellsMenuAttachTimer = setTimeout(() => {
      wellsMenuAttachTimer = null;
      if (wellsMenu === menu) document.addEventListener("mousedown", close);
    }, 0);
  });

  const curveSel = curveSelect(curveNames, opts.curve);
  curveSel.addEventListener("change", () => {
    opts.curve = curveSel.value;
    reloadAndDraw();
  });

  const numField = (placeholder: string, value: number | null, onChange: (v: number | null) => void): HTMLInputElement => {
    const input = document.createElement("input");
    input.type = "number";
    input.className = "form-control num-field";
    input.placeholder = placeholder;
    if (value !== null) input.value = String(value);
    input.addEventListener("change", () => {
      const v = input.value.trim() === "" ? null : Number(input.value);
      onChange(v !== null && Number.isFinite(v) ? v : null);
      draw();
      persist();
    });
    return input;
  };

  const validityWrap = document.createElement("label");
  validityWrap.className = "chk-field";
  const validityChk = document.createElement("input");
  validityChk.type = "checkbox";
  validityChk.checked = opts.validityFilter;
  validityWrap.append(validityChk, document.createTextNode(" Validity"));
  const validityChanged = (): void => {
    if (validityChk.checked && !correlationValidityRange(opts)) {
      validityChk.checked = false;
      opts.validityFilter = false;
      setStatus("Correlation validity requires two distinct finite limits before it can be enabled");
      return;
    }
    opts.validityFilter = validityChk.checked;
    draw();
    persist();
  };
  validityChk.addEventListener("change", validityChanged);
  const validMinField = numField("valid min", opts.validMin, (value) => {
    opts.validMin = value;
    if (opts.validityFilter && !correlationValidityRange(opts)) {
      opts.validityFilter = false;
      validityChk.checked = false;
      setStatus("Correlation validity disabled until both distinct finite limits are supplied");
    }
  });
  const validMaxField = numField("valid max", opts.validMax, (value) => {
    opts.validMax = value;
    if (opts.validityFilter && !correlationValidityRange(opts)) {
      opts.validityFilter = false;
      validityChk.checked = false;
      setStatus("Correlation validity disabled until both distinct finite limits are supplied");
    }
  });

  const datumSel = document.createElement("select");
  datumSel.className = "form-control";
  function refreshDatumChoices(): void {
    const names = Array.from(new Set(strips.flatMap((s) => s.tops.map((t) => t.top_name)))).sort();
    datumSel.innerHTML = "";
    const md = document.createElement("option");
    md.value = "";
    md.textContent = "Measured depth";
    datumSel.appendChild(md);
    for (const name of names) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = `Flatten on ${name}`;
      datumSel.appendChild(option);
    }
    datumSel.value = names.includes(opts.datum) ? opts.datum : "";
  }
  datumSel.addEventListener("change", () => {
    opts.datum = datumSel.value;
    applyDatum();
    fit();
  });

  const mkBtn = (label: string, title: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.className = "form-control";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };

  const depthModeSel = document.createElement("select");
  depthModeSel.className = "form-control";
  depthModeSel.title = "Depth axis — measured depth, or TVDSS (fluid contacts are flat in TVDSS)";
  for (const [val, lbl] of [
    ["md", "MD"],
    ["tvdss", "TVDSS"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = lbl;
    depthModeSel.appendChild(o);
  }
  depthModeSel.value = opts.depthMode;
  depthModeSel.addEventListener("change", () => {
    opts.depthMode = depthModeSel.value === "tvdss" ? "tvdss" : "md";
    applyDatum(); // shift is in display space → re-derive for the new mode
    fit();
    if (opts.depthMode === "tvdss") {
      const missing = strips.filter((s) => included.has(s.well.well_id) && !s.tv).length;
      if (missing > 0) {
        setStatus(`${missing} well(s) have no TVDSS reference frame; MD was not substituted.`);
      }
    }
  });

  // --- Fluid-contacts editor ---
  const scopeValue = (c: FluidContact): string =>
    c.well_id ? `well:${c.well_id}` : c.field_name ? `field:${c.field_name}` : "";
  const applyScope = (c: FluidContact, value: string): void => {
    if (value.startsWith("well:")) {
      c.well_id = value.slice(5);
      c.field_name = null;
    } else if (value.startsWith("field:")) {
      c.field_name = value.slice(6);
      c.well_id = null;
    } else {
      c.well_id = null;
      c.field_name = null;
    }
  };

  function openContactsEditor(): void {
    const body = document.createElement("div");
    body.className = "contacts-editor";

    const showRow = document.createElement("label");
    showRow.className = "contacts-show";
    const showBox = document.createElement("input");
    showBox.type = "checkbox";
    showBox.checked = opts.showContacts;
    showBox.addEventListener("change", () => {
      opts.showContacts = showBox.checked;
      persist();
      draw();
    });
    showRow.append(showBox, document.createTextNode(" Show contacts in the view"));
    body.appendChild(showRow);

    const table = document.createElement("div");
    table.className = "contacts-table";
    body.appendChild(table);

    const fields = Array.from(new Set(wells.map((w) => w.field_name).filter((f): f is string => !!f))).sort();

    const save = async (c: FluidContact): Promise<void> => {
      await upsertFluidContact(c).catch((e) => setStatus(`Contact save failed: ${e}`));
      draw();
    };

    const renderRows = (): void => {
      table.innerHTML = "";
      if (!contacts.length) {
        const empty = document.createElement("div");
        empty.className = "contacts-empty";
        empty.textContent = "No fluid contacts yet — add one below.";
        table.appendChild(empty);
      }
      for (const c of contacts) {
        const row = document.createElement("div");
        row.className = "contacts-row";

        const typeSel = document.createElement("select");
        typeSel.className = "form-control";
        for (const t of CONTACT_TYPES) {
          const o = document.createElement("option");
          o.value = t;
          o.textContent = t;
          typeSel.appendChild(o);
        }
        if (!CONTACT_TYPES.includes(c.contact_type)) {
          const o = document.createElement("option");
          o.value = c.contact_type;
          o.textContent = c.contact_type;
          typeSel.appendChild(o);
        }
        typeSel.value = c.contact_type;
        typeSel.addEventListener("change", () => {
          c.contact_type = typeSel.value;
          void save(c);
        });

        const depthInput = document.createElement("input");
        depthInput.type = "number";
        depthInput.className = "form-control num-field";
        depthInput.value = String(c.depth);
        depthInput.addEventListener("change", () => {
          const v = Number(depthInput.value);
          if (Number.isFinite(v)) {
            c.depth = v;
            void save(c);
          }
        });

        const ssLabel = document.createElement("label");
        ssLabel.className = "contacts-ss";
        const ssBox = document.createElement("input");
        ssBox.type = "checkbox";
        ssBox.checked = c.is_tvdss;
        ssBox.addEventListener("change", () => {
          c.is_tvdss = ssBox.checked;
          c.depth_datum = c.is_tvdss ? "TVDSS" : "MD";
          void save(c);
        });
        ssLabel.append(ssBox, document.createTextNode(" TVDSS"));

        const scopeSel = document.createElement("select");
        scopeSel.className = "form-control";
        const gOpt = document.createElement("option");
        gOpt.value = "";
        gOpt.textContent = "All wells";
        scopeSel.appendChild(gOpt);
        for (const f of fields) {
          const o = document.createElement("option");
          o.value = `field:${f}`;
          o.textContent = `Field: ${f}`;
          scopeSel.appendChild(o);
        }
        for (const w of wells) {
          const o = document.createElement("option");
          o.value = `well:${w.well_id}`;
          o.textContent = `Well: ${w.well_name}`;
          scopeSel.appendChild(o);
        }
        scopeSel.value = scopeValue(c);
        scopeSel.addEventListener("change", () => {
          applyScope(c, scopeSel.value);
          void save(c);
        });

        const colorInput = document.createElement("input");
        colorInput.type = "color";
        colorInput.className = "contacts-color";
        colorInput.value = contactColor(c);
        colorInput.title = "Marker color";
        colorInput.addEventListener("change", () => {
          c.color = colorInput.value;
          void save(c);
        });

        const del = document.createElement("button");
        del.className = "form-control contacts-del";
        del.textContent = "✕";
        del.title = "Delete contact";
        del.addEventListener("click", () => {
          void deleteFluidContact(c.contact_id).then(() => {
            contacts = contacts.filter((x) => x.contact_id !== c.contact_id);
            renderRows();
            draw();
          });
        });

        row.append(typeSel, depthInput, ssLabel, scopeSel, colorInput, del);
        table.appendChild(row);
      }
    };

    const addBtn = document.createElement("button");
    addBtn.className = "form-control contacts-add";
    addBtn.textContent = "＋ Add contact";
    addBtn.addEventListener("click", () => {
      const c: FluidContact = {
        contact_id: crypto.randomUUID(),
        field_name: null,
        well_id: null,
        contact_type: "OWC",
        depth: Math.round(viewTop + 50),
        depth_datum: opts.depthMode === "tvdss" ? "TVDSS" : "MD",
        is_tvdss: opts.depthMode === "tvdss",
        color: null,
        label: null,
      };
      contacts.push(c);
      void upsertFluidContact(c).then(() => {
        renderRows();
        draw();
      });
    });
    body.appendChild(addBtn);

    const labeled = (text: string, el: HTMLElement): HTMLLabelElement => {
      const l = document.createElement("label");
      l.className = "contacts-inline";
      l.append(document.createTextNode(`${text} `), el);
      return l;
    };
    const sectionTitle = (text: string): HTMLDivElement => {
      const d = document.createElement("div");
      d.className = "contacts-section-title";
      d.textContent = text;
      return d;
    };

    // --- Suggest a contact from logs (Sw crossover / resistivity drop / density-neutron) ---
    const sug = document.createElement("div");
    sug.className = "contacts-suggest";
    sug.appendChild(sectionTitle("Suggest from logs"));
    const sugWell = document.createElement("select");
    sugWell.className = "form-control";
    for (const w of wells) {
      const o = document.createElement("option");
      o.value = w.well_id;
      o.textContent = w.well_name;
      sugWell.appendChild(o);
    }
    const plotH = Math.max(50, canvas.clientHeight - HEADER_H);
    const zTop = document.createElement("input");
    zTop.type = "number";
    zTop.className = "form-control num-field";
    zTop.value = String(Math.round(viewTop));
    const zBase = document.createElement("input");
    zBase.type = "number";
    zBase.className = "form-control num-field";
    zBase.value = String(Math.round(viewTop + plotH / pxPerUnit));
    const sugBtn = document.createElement("button");
    sugBtn.className = "form-control";
    sugBtn.textContent = "Suggest";
    const sugControls = document.createElement("div");
    sugControls.className = "contacts-section-controls";
    sugControls.append(sugWell, labeled("top", zTop), labeled("base", zBase), sugBtn);
    sug.appendChild(sugControls);
    const sugResults = document.createElement("div");
    sugResults.className = "contacts-section-results";
    sug.appendChild(sugResults);

    const renderCandidates = (res: ContactSuggestResult): void => {
      sugResults.innerHTML = "";
      if (res.error) {
        sugResults.textContent = res.error;
        return;
      }
      if (!res.candidates.length) {
        sugResults.textContent = "No Sw / resistivity / density-neutron indicators in that zone.";
        return;
      }
      const wellId = sugWell.value;
      for (const cand of res.candidates as ContactCandidate[]) {
        const row = document.createElement("div");
        row.className = "contacts-cand";
        if (cand.confidence < 0.4) row.classList.add("weak-match");
        const info = document.createElement("span");
        info.textContent = `${cand.contact_type} @ ${cand.depth.toFixed(1)} — ${cand.method} (${Math.round(cand.confidence * 100)}%)`;
        info.title = cand.detail;
        const accept = document.createElement("button");
        accept.className = "form-control";
        accept.textContent = "Accept";
        accept.addEventListener("click", () => {
          const c: FluidContact = {
            contact_id: crypto.randomUUID(),
            field_name: null,
            well_id: wellId,
            contact_type: cand.contact_type,
            depth: Number(cand.depth.toFixed(1)),
            depth_datum: "MD",
            is_tvdss: false, // suggestions are in measured depth
            color: null,
            label: cand.method,
          };
          contacts.push(c);
          void upsertFluidContact(c).then(() => {
            renderRows();
            draw();
            setStatus(`Added ${cand.contact_type} at ${c.depth} m (${cand.method})`);
          });
          accept.disabled = true;
          accept.textContent = "Added";
        });
        row.append(info, accept);
        sugResults.appendChild(row);
      }
    };
    sugBtn.addEventListener("click", () => {
      void (async () => {
        sugBtn.disabled = true;
        sugResults.textContent = "Scanning…";
        try {
          const res = await suggestContacts({
            well_id: sugWell.value,
            zone_top: Number(zTop.value),
            zone_base: Number(zBase.value),
          });
          renderCandidates(res);
        } catch (e) {
          sugResults.textContent = `Suggest failed: ${e}`;
        } finally {
          sugBtn.disabled = false;
        }
      })();
    });
    body.appendChild(sug);

    // --- Cross-well consistency: a contact is flat in TVDSS ---
    const con = document.createElement("div");
    con.className = "contacts-consistency";
    con.appendChild(sectionTitle("Cross-well consistency"));
    const conType = document.createElement("select");
    conType.className = "form-control";
    for (const t of CONTACT_TYPES) {
      const o = document.createElement("option");
      o.value = t;
      o.textContent = t;
      conType.appendChild(o);
    }
    const conBtn = document.createElement("button");
    conBtn.className = "form-control";
    conBtn.textContent = "Check";
    const conControls = document.createElement("div");
    conControls.className = "contacts-section-controls";
    conControls.append(conType, conBtn);
    con.appendChild(conControls);
    const conResults = document.createElement("div");
    conResults.className = "contacts-section-results";
    con.appendChild(conResults);

    const renderConsistency = (r: ContactConsistency): void => {
      conResults.innerHTML = "";
      if (r.error) {
        conResults.textContent = r.error;
        return;
      }
      const summary = document.createElement("div");
      summary.className = "contacts-consistency-summary";
      // Every number below is a length in the depth dimension and arrives in the project's
      // STORED unit, so it is converted for reading and labelled with the display unit. The
      // `m` here used to be hard-coded, which made a foot project's residuals read 3.28x too
      // large under a metre heading - and a residual is exactly what the flag threshold above
      // is judged against, so the two disagreed on screen.
      const su = shownDepthLabel();
      const rms = Number.isFinite(r.rms) ? `${toShownDepth(r.rms).toFixed(1)} ${su}` : "—";
      summary.textContent = `${r.n} wells · ${r.plane ? "dip plane" : "flat mean"} · mean ${toShownDepth(r.mean_tvdss).toFixed(1)} TVDSS · rms ${rms}`;
      conResults.appendChild(summary);
      const table = document.createElement("table");
      table.className = "dbgrid";
      table.innerHTML = `<thead><tr><th>Well</th><th>TVDSS (${su})</th><th>Predicted (${su})</th><th>Resid (${su})</th><th></th></tr></thead>`;
      const tb = document.createElement("tbody");
      for (const w of r.wells) {
        const tr = document.createElement("tr");
        if (w.flagged) tr.classList.add("weak-match");
        const cells = [
          w.well_name,
          toShownDepth(w.tvdss).toFixed(1),
          Number.isFinite(w.predicted) ? toShownDepth(w.predicted).toFixed(1) : "—",
          Number.isFinite(w.residual) ? toShownDepth(w.residual).toFixed(1) : "—",
          w.flagged ? "⚠" : "",
        ];
        for (const t of cells) {
          const td = document.createElement("td");
          td.textContent = t;
          tr.appendChild(td);
        }
        tb.appendChild(tr);
      }
      table.appendChild(tb);
      conResults.appendChild(table);
    };
    conBtn.addEventListener("click", () => {
      void (async () => {
        conBtn.disabled = true;
        conResults.textContent = "Checking…";
        try {
          renderConsistency(await checkContactConsistency(conType.value));
        } catch (e) {
          conResults.textContent = `Check failed: ${e}`;
        } finally {
          conBtn.disabled = false;
        }
      })();
    });
    body.appendChild(con);

    renderRows();
    openModal("Fluid contacts", body, 640);
  }

  props.appendChild(wellsBtn);
  props.appendChild(curveSel);
  props.appendChild(numField("min", opts.min, (v) => (opts.min = v)));
  props.appendChild(numField("max", opts.max, (v) => (opts.max = v)));
  props.appendChild(validityWrap);
  props.appendChild(validMinField);
  props.appendChild(validMaxField);
  props.appendChild(datumSel);
  props.appendChild(depthModeSel);
  props.appendChild(mkBtn("Contacts…", "Add / edit fluid contacts (OWC, GWC, …)", openContactsEditor));
  props.appendChild(mkBtn("Fit", "Fit all wells vertically", fit));
  props.appendChild(mkBtn("＋", "Zoom in", () => {
    zoomAtCenter(1.25);
  }));
  props.appendChild(mkBtn("−", "Zoom out", () => {
    zoomAtCenter(1 / 1.25);
  }));
  const exportGroup = buildImageExportButtons(
    () => canvas,
    "Correlation",
    setStatus,
    undefined,
    undefined,
    undefined,
    () => {
      const state = persistedState(selectionState());
      return {
        wellIds: state.well_ids,
        curves: plotIntents().map((intent) => intent.semantic_request),
        plotBindings: state.bindings,
        axisRanges: state.axis_ranges,
        statisticsRecords,
      };
    },
  );
  props.appendChild(exportGroup);

  function zoomAtCenter(factor: number): void {
    const plotH = Math.max(50, canvas.clientHeight - HEADER_H);
    const mid = viewTop + plotH / 2 / pxPerUnit;
    pxPerUnit *= factor;
    viewTop = mid - plotH / 2 / pxPerUnit;
    depthViewIsUser = true;
    draw();
    persist();
  }

  const accessibility = attachAccessiblePlotKeyboard({
    surface: canvas,
    getLabel: () =>
      `Correlation: ${opts.curve} across ${included.size} included well${included.size === 1 ? "" : "s"}, ${opts.depthMode.toUpperCase()} depth${opts.datum ? ` flattened on ${opts.datum}` : ""}`,
    openProperties: () => curveSel.focus(),
    focusExport: () => exportGroup.querySelector<HTMLButtonElement>("button")?.focus(),
    changeView: (command) => {
      if (command.kind === "reset") {
        if (!displayExtent()) return false;
        fit();
        return true;
      }
      if (command.kind === "zoom") {
        if (!displayExtent()) return false;
        zoomAtCenter(command.direction === "in" ? 1.25 : 1 / 1.25);
        return true;
      }
      if (command.axis !== "y") return false;
      const plotH = Math.max(50, canvas.clientHeight - HEADER_H);
      const span = plotH / pxPerUnit;
      if (!Number.isFinite(span) || span <= 0) return false;
      viewTop += command.direction * span * (command.large ? 0.2 : 0.08);
      depthViewIsUser = true;
      draw();
      persist();
      return true;
    },
  });

  // --- Interactions: wheel/drag pan, hover broadcast ---
  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      if (e.ctrlKey || e.metaKey) {
        // Ctrl/Cmd+wheel zooms about the cursor depth — same convention (and factors) as
        // attachZoomPan on the other plots (in = shrink the depth window). Plain wheel keeps
        // panning through depth (there's no competing page scroll inside a dock pane).
        const rect = canvas.getBoundingClientRect();
        const y = Math.max(HEADER_H, e.clientY - rect.top);
        const anchor = viewTop + (y - HEADER_H) / pxPerUnit;
        const f = e.deltaY < 0 ? 0.83 : 1.2;
        pxPerUnit /= f;
        viewTop = anchor - (y - HEADER_H) / pxPerUnit;
      } else {
        viewTop += e.deltaY / pxPerUnit;
      }
      depthViewIsUser = true;
      draw();
      scheduleWheelPersist();
    },
    { passive: false },
  );
  let dragging = false;
  let lastY = 0;
  canvas.addEventListener("pointerdown", (e) => {
    dragging = true;
    lastY = e.clientY;
  });
  // A drag started on the canvas must end even when the pointer is released outside it, so this
  // lives on `window` — which means it survives the panel being removed on close. Keep a named
  // reference and remove it in dispose(), or every closed correlation panel leaks one pointerup
  // that pins this whole builder closure (every strip's decimated curve arrays + the detached
  // canvas) for the app's life — the exact trap LogCanvasRenderer.ts:540-561 documents and the
  // only window-level listener in a panel builder that was missing its removal.
  const onWindowPointerUp = () => {
    const wasDragging = dragging;
    dragging = false;
    if (wasDragging) persist();
  };
  window.addEventListener("pointerup", onWindowPointerUp);
  canvas.addEventListener("pointermove", (e) => {
    const rect = canvas.getBoundingClientRect();
    const y = e.clientY - rect.top;
    if (dragging) {
      viewTop -= (e.clientY - lastY) / pxPerUnit;
      lastY = e.clientY;
      depthViewIsUser = true;
    }
    hoverY = y;
    // Broadcast the hovered STRIP's measured depth so the well's other views sync.
    const active = strips.filter((s) => included.has(s.well.well_id));
    const x = e.clientX - rect.left;
    const slot = (rect.width - AXIS_W) / Math.max(1, active.length);
    const idx = Math.floor((x - AXIS_W) / slot);
    const disp = viewTop + (y - HEADER_H) / pxPerUnit;
    if (idx >= 0 && idx < active.length && y > HEADER_H) {
      const s = active[idx];
      const unflattened = disp + s.shift; // display depth without flattening
      // Other views expect measured depth, so undo the TVDSS mapping before broadcasting.
      const measured = opts.depthMode === "tvdss" ? tvdssToMd(s, unflattened) : unflattened;
      appState.hoverDepth.set(measured);
    } else {
      appState.hoverDepth.set(null);
    }
    draw();
  });
  canvas.addEventListener("pointerleave", () => {
    hoverY = null;
    appState.hoverDepth.set(null);
    draw();
  });

  try {
    await reload();
  } catch (err) {
    setStatus(`Correlation load failed: ${err}`);
  }
  // Initial fit once the panel has a size (dock lays out after mount). Captured so a panel closed
  // inside 50 ms doesn't run fit()→draw() against a canvas that has already been detached.
  const fitTimer = setTimeout(() => {
    if (restoredDepthView) {
      const h = Math.max(50, canvas.clientHeight - HEADER_H);
      pxPerUnit = h / (restoredDepthView.max - restoredDepthView.min);
      viewTop = restoredDepthView.min;
      depthViewIsUser = true;
      draw();
    } else {
      fit();
    }
  }, 50);

  const invalidation = registerPlotInvalidationContract(canvasHost, {
    theme: () => draw(),
    dataRevision: () => {
      // Refresh the well inventory before its curves so an import/delete/group change cannot
      // leave the Wells menu and the rendered strips on different data revisions.
      void refreshWells()
        .then((current) => current ? reload() : false)
        .then((applied) => {
          if (applied) draw();
        })
        .catch((error) => {
          strips = [];
          setStatus(`Correlation refused: ${error}`);
          draw();
        });
    },
    interval: (interval) => {
      selectedInterval = interval;
      draw();
    },
    selection: (selection) => {
      brushSelection = selection;
      draw();
    },
    size: () => draw(),
    cancelPending: () => {
      reloadGen++;
      wellsGen++;
      clearTimeout(fitTimer);
      if (wheelPersistTimer !== null) {
        clearTimeout(wheelPersistTimer);
        wheelPersistTimer = null;
      }
      removeWellsMenu();
    },
  });

  return {
    el,
    dispose: () => {
      if (wheelPersistTimer !== null) {
        clearTimeout(wheelPersistTimer);
        wheelPersistTimer = null;
        persist();
      }
      invalidation.dispose();
      accessibility.dispose();
      window.removeEventListener("pointerup", onWindowPointerUp);
    },
    getState: selectionState,
    getPersistedState: () => persistedState(selectionState()),
  };
}
