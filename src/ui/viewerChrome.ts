import type { CurveStyle, Layout, Track, WellSummary } from "../ipc";
import { trackCurveKey } from "../trackCurveRequest";
import { FACIES_PALETTE } from "./plotCanvas";
import { formatValue } from "./plotCommon";
import { appState } from "../state";
import { formatDepth, unitLabel } from "../units";

export interface TrackChromeCallbacks {
  /** Fired after resize, reorder, or scale edit — needs a full geometry rebuild. */
  onLayoutMutated: () => void;
  /** Fired on curve visibility toggle — cheap path, no geometry rebuild needed. */
  onCurveToggle: (curveName: string, hidden: boolean) => void;
  /** Fired after a curve was dragged to another track (move, or copy with Ctrl held),
   *  BEFORE onLayoutMutated. `before` is a snapshot of the layout pre-mutation so the
   *  panel can push an undo action. */
  onCurveMoved?: (before: Layout, label: string) => void;
}

const CURVE_DRAG_TYPE = "application/x-sandibumi-curve";

/** Renders track headers (title, curve legend, editable scale) and wires up drag-resize,
 * drag-reorder, curve visibility toggle, and inline scale editing. `trackWeights` is
 * mutated in place by resize interactions. `hiddenCurves` reflects (and is mutated by)
 * which curves are currently toggled off, so re-rendering headers doesn't lose that state. */
export function renderTrackHeaders(
  container: HTMLElement,
  layout: Layout,
  trackWeights: Map<string, number>,
  hiddenCurves: Set<string>,
  callbacks: TrackChromeCallbacks,
): void {
  container.innerHTML = "";

  const spacer = document.createElement("div");
  spacer.className = "track-header-spacer";
  container.appendChild(spacer);

  layout.tracks.forEach((track) => {
    const header = document.createElement("div");
    header.className = "track-header";
    header.dataset.track = track.title; // hover highlight targets headers by title
    header.style.flexGrow = String(trackWeights.get(track.title) ?? 150);
    header.style.flexBasis = "0";

    const title = document.createElement("div");
    title.className = "track-header-title";
    title.textContent = `${track.title}${track.scale_type === "log" ? " (log)" : ""}`;
    title.title = "Drag to reorder";
    attachHeaderDragReorder(title, header, layout, track, callbacks);
    header.appendChild(title);

    const legend = document.createElement("div");
    legend.className = "track-header-legend";
    for (const c of track.curves) {
      legend.appendChild(buildCurveRow(c, track, hiddenCurves, callbacks));
    }
    header.appendChild(legend);

    const resizer = document.createElement("div");
    resizer.className = "track-resizer";
    attachTrackResizer(resizer, track, trackWeights, header, callbacks);
    header.appendChild(resizer);

    container.appendChild(header);
  });
}

function buildCurveRow(
  curve: CurveStyle,
  track: Track,
  hiddenCurves: Set<string>,
  callbacks: TrackChromeCallbacks,
): HTMLElement {
  const wrapper = document.createElement("div");
  wrapper.className = "curve-entry";
  const isBlocks = curve.fill === "blocks";
  const curveKey = trackCurveKey(curve);

  const row = document.createElement("div");
  row.className = "curve-row" + (hiddenCurves.has(curveKey) ? " hidden" : "");
  // Drag a curve onto another track's header to MOVE it there (hold Ctrl to COPY).
  row.draggable = true;
  row.addEventListener("dragstart", (e) => {
    e.dataTransfer?.setData(CURVE_DRAG_TYPE, JSON.stringify({ track: track.title, curveKey }));
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "copyMove";
  });

  const swatch = document.createElement("span");
  swatch.className = "legend-dot" + (isBlocks ? " blocks" : "");
  if (isBlocks) {
    // Discrete block curve: striped swatch from the facies palette instead of one color.
    const stops = FACIES_PALETTE.slice(0, 6)
      .map((c, i) => `${c} ${(i / 6) * 100}%, ${c} ${((i + 1) / 6) * 100}%`)
      .join(", ");
    swatch.style.background = `linear-gradient(90deg, ${stops})`;
  } else {
    swatch.style.borderTopColor = curve.color;
  }
  swatch.title = "Toggle visibility";

  const name = document.createElement("span");
  name.className = "legend-name";
  name.textContent = `${curve.curve_name}${curve.set_name ? ` · ${curve.set_name}` : ""}`;

  const toggle = () => {
    row.classList.toggle("hidden");
    const nowHidden = row.classList.contains("hidden");
    if (nowHidden) hiddenCurves.add(curveKey);
    else hiddenCurves.delete(curveKey);
    callbacks.onCurveToggle(curveKey, nowHidden);
  };
  swatch.addEventListener("click", toggle);
  name.addEventListener("click", toggle);
  row.appendChild(swatch);
  row.appendChild(name);
  wrapper.appendChild(row);

  const scaleLine = document.createElement("div");
  scaleLine.className = "scale-line";
  if (isBlocks) {
    // Class index has no meaningful min/max scale — blocks always span the track.
    const label = document.createElement("span");
    label.textContent = "class blocks";
    label.style.cursor = "default";
    scaleLine.appendChild(label);
  } else {
    const minSpan = document.createElement("span");
    minSpan.textContent = fmtScale(curve.min);
    const maxSpan = document.createElement("span");
    maxSpan.textContent = fmtScale(curve.max);
    minSpan.addEventListener("click", () => editScale(minSpan, curve, "min", callbacks));
    maxSpan.addEventListener("click", () => editScale(maxSpan, curve, "max", callbacks));
    scaleLine.appendChild(minSpan);
    scaleLine.appendChild(maxSpan);
  }
  wrapper.appendChild(scaleLine);

  return wrapper;
}

function fmtScale(v: number): string {
  if (Math.abs(v) >= 1000 || (Math.abs(v) < 0.001 && v !== 0)) return v.toExponential(0);
  return (Math.round(v * 1000) / 1000).toString();
}

function editScale(
  span: HTMLElement,
  curve: { min: number; max: number },
  key: "min" | "max",
  callbacks: TrackChromeCallbacks,
): void {
  const rect = span.getBoundingClientRect();
  const input = document.createElement("input");
  input.className = "edit-input";
  input.value = String(curve[key]);
  input.style.left = `${rect.left}px`;
  input.style.top = `${rect.top - 2}px`;
  document.body.appendChild(input);
  input.focus();
  input.select();

  const commit = () => {
    const v = parseFloat(input.value);
    if (!Number.isNaN(v)) curve[key] = v;
    input.remove();
    callbacks.onLayoutMutated();
  };
  input.addEventListener("blur", commit);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") input.blur();
    if (e.key === "Escape") input.remove();
  });
}

function attachTrackResizer(
  resizer: HTMLElement,
  track: Track,
  trackWeights: Map<string, number>,
  header: HTMLElement,
  callbacks: TrackChromeCallbacks,
): void {
  resizer.addEventListener("mousedown", (e) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWeight = trackWeights.get(track.title) ?? 150;
    resizer.classList.add("active");

    const onMove = (ev: MouseEvent) => {
      const w = Math.max(40, startWeight + (ev.clientX - startX));
      trackWeights.set(track.title, Math.round(w));
      header.style.flexGrow = String(w);
    };
    const onUp = () => {
      resizer.classList.remove("active");
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      callbacks.onLayoutMutated();
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  });
}

function attachHeaderDragReorder(
  title: HTMLElement,
  header: HTMLElement,
  layout: Layout,
  track: Track,
  callbacks: TrackChromeCallbacks,
): void {
  title.draggable = true;
  title.addEventListener("dragstart", (e) => {
    e.dataTransfer?.setData("text/plain", track.title);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  });
  header.addEventListener("dragover", (e) => {
    e.preventDefault();
    if (e.dataTransfer) {
      // Curve drags copy with Ctrl held; track reorders are always a move.
      const isCurve = e.dataTransfer.types.includes(CURVE_DRAG_TYPE);
      e.dataTransfer.dropEffect = isCurve && e.ctrlKey ? "copy" : "move";
    }
    header.classList.add("drag-over");
  });
  header.addEventListener("dragleave", () => header.classList.remove("drag-over"));
  header.addEventListener("drop", (e) => {
    e.preventDefault();
    header.classList.remove("drag-over");

    // A curve row dropped on this track: move it here (Ctrl at drop = copy, keeping
    // the original). The panel gets a pre-mutation snapshot for undo.
    const curvePayload = e.dataTransfer?.getData(CURVE_DRAG_TYPE);
    if (curvePayload) {
      const { track: srcTitle, curveKey } = JSON.parse(curvePayload) as { track: string; curveKey: string };
      if (srcTitle === track.title) return;
      const src = layout.tracks.find((t) => t.title === srcTitle);
      const style = src?.curves.find((curve) => trackCurveKey(curve) === curveKey);
      if (!src || !style) return;
      if (track.curves.some((curve) => trackCurveKey(curve) === curveKey)) return; // already shown here
      const before = structuredClone(layout);
      const copy = e.ctrlKey;
      if (!copy) src.curves.splice(src.curves.indexOf(style), 1);
      track.curves.push(copy ? structuredClone(style) : style);
      callbacks.onCurveMoved?.(before, `${copy ? "copy" : "move"} ${style.curve_name} → ${track.title}`);
      callbacks.onLayoutMutated();
      return;
    }

    const srcTitle = e.dataTransfer?.getData("text/plain");
    if (!srcTitle || srcTitle === track.title) return;
    const from = layout.tracks.findIndex((t) => t.title === srcTitle);
    const to = layout.tracks.findIndex((t) => t.title === track.title);
    if (from === -1 || to === -1) return;
    const [moved] = layout.tracks.splice(from, 1);
    layout.tracks.splice(to, 0, moved);
    callbacks.onLayoutMutated();
  });
}

export function renderDepthAxis(container: HTMLElement, top: number, bottom: number, tickCount = 6): void {
  container.innerHTML = "";
  for (let i = 0; i < tickCount; i++) {
    const frac = i / (tickCount - 1);
    const depth = top + frac * (bottom - top);
    const tick = document.createElement("div");
    tick.className = "depth-tick";
    tick.style.top = `${frac * 100}%`;
    // Depths arrive in the project's STORED unit; show them in the unit the user chose.
    tick.textContent = formatDepth(depth, appState.projectDepthUnit.get(), appState.displayDepthUnit.get(), 0);
    container.appendChild(tick);
  }
}

export function renderReadout(
  container: HTMLElement,
  depth: number | null,
  samples: { curveName: string; value: number }[],
  emphasize?: Set<string>,
  units?: Map<string, string>,
  labels?: Map<string, string>,
): void {
  if (depth === null) {
    container.hidden = true;
    return;
  }
  container.hidden = false;
  container.innerHTML = "";
  const depthSpan = document.createElement("span");
  const display = appState.displayDepthUnit.get();
  depthSpan.textContent = `Depth: ${formatDepth(depth, appState.projectDepthUnit.get(), display, 1)} ${unitLabel(display)}`;
  container.appendChild(depthSpan);
  for (const s of samples) {
    const item = document.createElement("span");
    item.className = "readout-item" + (emphasize?.has(s.curveName) ? " em" : "");
    item.textContent = `${labels?.get(s.curveName) ?? s.curveName}: ${formatValue(s.value, { unit: units?.get(s.curveName) })}`;
    container.appendChild(item);
  }
}

const REPORT_FIELDS: { key: "well" | "field" | "depth"; label: string }[] = [
  { key: "well", label: "Well" },
  { key: "field", label: "Field" },
  { key: "depth", label: "Depth Coverage" },
];

/** "Printed sheet" style info strip: Well / Field / Depth Coverage, from what our schema
 * actually captures (no UWI/Company/Location fields yet — those show as empty). */
export function renderReportHeader(container: HTMLElement, well: WellSummary | null, depthRange: [number, number] | null): void {
  if (!well) {
    container.hidden = true;
    return;
  }
  container.hidden = false;
  container.innerHTML = "";

  const values: Record<string, string> = {
    well: well.well_name,
    field: well.field_name ?? "",
    depth: depthRange ? `${depthRange[0].toFixed(1)} – ${depthRange[1].toFixed(1)}` : "",
  };

  for (const field of REPORT_FIELDS) {
    const item = document.createElement("div");
    item.className = "rh-item";
    const label = document.createElement("div");
    label.className = "rh-label";
    label.textContent = field.label;
    const value = document.createElement("div");
    value.className = "rh-value";
    const text = values[field.key];
    if (!text) {
      value.classList.add("empty");
      value.textContent = "—";
    } else {
      value.textContent = text;
    }
    item.appendChild(label);
    item.appendChild(value);
    container.appendChild(item);
  }
}
