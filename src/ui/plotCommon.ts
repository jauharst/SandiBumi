import { deleteDocument, getCurveData, listCurveCatalog, listDocuments, listTops, listZones, saveDocument, setZoneParam, type TrackCurveSeries, type WellSummary, type ZoneEntry } from "../ipc";
import { appState, type TopInterval } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";
import { FACIES_PALETTE } from "./plotCanvas";
import { allocateFinitePairBudget, type ReductionManifest } from "./plotTypes";

/** Shared pieces for the parameter-selection dialogs: curve/zone selectors and the
 *  "apply picked value to a zone parameter" row. */

/** What a plot panel builder hands to the dock: its DOM plus an optional cleanup for
 *  subscriptions (synchronized hover etc.), called when the panel closes. */
export interface PlotContent {
  el: HTMLElement;
  dispose?: () => void;
  /** Current user selections (curves/zone) so the workspace can rebuild the plot for a
   *  newly selected well without losing them. */
  getState?: () => Record<string, string>;
  /** Opens this plot's Properties dialog. The workspace puts it at the top of the pane's
   *  right-click menu — the canvas no longer swallows right-click to open it directly,
   *  which had hidden the window actions (split/float/export) on every plot. */
  openProperties?: () => void;
}

/** Maps a log curve mnemonic to the core measurement it's calibrated against, so
 *  crossplots and log tracks can overlay CPOR/CPERM/CGD/CSW plug points onto the
 *  matching curve (e.g. PHIE vs a permeability curve → the classic por-perm
 *  calibration plot; a PHIE track → CPOR dots). */
export const CORE_OVERLAY_MAP: Record<string, string> = {
  PHIE: "CPOR",
  PHIT: "CPOR",
  PHIE_LAM: "CPOR",
  NPHI: "CPOR",
  PERM: "CPERM",
  PERM_WR: "CPERM",
  PERM_COATES: "CPERM",
  PERM_XFM: "CPERM",
  RHOB: "CGD",
  SWE: "CSW",
  SWT: "CSW",
};

/** Index of the sample whose depth is nearest `depth` (depths ascending); -1 if empty. */
export function nearestDepthIndex(depths: Float32Array, depth: number): number {
  if (depths.length === 0) return -1;
  let lo = 0;
  let hi = depths.length - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (depths[mid] < depth) lo = mid;
    else hi = mid;
  }
  return Math.abs(depths[lo] - depth) <= Math.abs(depths[hi] - depth) ? lo : hi;
}

/** Last-used properties for a plot kind ("histogram", "crossplot", ...) from the
 *  project's `documents` table; {} when unset or when there is no backend. */
export async function loadPlotProps<T>(kind: string): Promise<Partial<T>> {
  try {
    const docs = await listDocuments("plotprops");
    const doc = docs.find((d) => d.name === kind);
    return doc ? (JSON.parse(doc.json) as Partial<T>) : {};
  } catch {
    return {};
  }
}

/** Fire-and-forget save of a plot kind's properties — new panels of that kind open
 *  with them. */
export function savePlotProps(kind: string, props: unknown): void {
  void saveDocument("plotprops", kind, JSON.stringify(props)).catch(() => {});
}

// --- Named plot templates ---------------------------------------------------
// Beyond the single "last used" props above, users can save the current display
// settings of a plot under a name and recall them later — the visualization equivalent
// of a named log layout. Stored per kind so a histogram template never shows up for a
// crossplot.

function plotTemplateDocType(kind: string): string {
  return `plottmpl:${kind}`;
}

export async function listPlotTemplates(kind: string): Promise<{ name: string; json: string }[]> {
  try {
    return await listDocuments(plotTemplateDocType(kind));
  } catch {
    return [];
  }
}

export async function savePlotTemplate(kind: string, name: string, opts: unknown): Promise<void> {
  await saveDocument(plotTemplateDocType(kind), name, JSON.stringify(opts));
}

export async function deletePlotTemplate(kind: string, name: string): Promise<void> {
  await deleteDocument(plotTemplateDocType(kind), name);
}

/** A toolbar control for a plot panel: a template picker plus Save/Delete. `getOpts`
 *  returns the current settings to store; `applyOpts` receives a recalled template's
 *  settings and should merge them in and re-render. Kept generic so every plot kind reuses
 *  the same UI. */
export function buildPlotTemplateBar<T>(
  kind: string,
  niceName: string,
  getOpts: () => T,
  applyOpts: (opts: Partial<T>) => void,
  setStatus: (text: string) => void,
): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "plot-template-bar";

  const select = document.createElement("select");
  select.className = "form-control plot-template-select";
  select.title = "Recall a saved template of settings for this plot";

  const refill = async () => {
    const templates = await listPlotTemplates(kind);
    select.innerHTML = "";
    const head = document.createElement("option");
    head.value = "";
    head.textContent = templates.length ? "— Template —" : "— No templates —";
    select.appendChild(head);
    for (const t of templates) {
      const opt = document.createElement("option");
      opt.value = t.name;
      opt.textContent = t.name;
      select.appendChild(opt);
    }
  };

  select.addEventListener("change", async () => {
    const name = select.value;
    if (!name) return;
    const templates = await listPlotTemplates(kind);
    const doc = templates.find((t) => t.name === name);
    if (!doc) return;
    try {
      applyOpts(JSON.parse(doc.json) as Partial<T>);
      setStatus(`Applied ${niceName} template "${name}"`);
    } catch {
      setStatus(`Template "${name}" is unreadable`);
    }
    select.value = "";
  });

  const saveBtn = document.createElement("button");
  saveBtn.className = "plot-export-btn";
  saveBtn.textContent = "★ Save template";
  saveBtn.title = "Save the current settings as a named template";
  saveBtn.addEventListener("click", () => {
    const body = document.createElement("div");
    const nameInput = document.createElement("input");
    nameInput.className = "form-control";
    nameInput.placeholder = `My ${niceName} template`;
    body.appendChild(formRow("Template name", nameInput));
    const doSaveBtn = document.createElement("button");
    doSaveBtn.className = "lp-btn primary";
    doSaveBtn.textContent = "Save";
    doSaveBtn.style.marginTop = "10px";
    body.appendChild(doSaveBtn);
    const close = openModal(`Save ${niceName} Template`, body, 360);
    nameInput.focus();
    const commit = async () => {
      const name = nameInput.value.trim();
      if (!name) return;
      try {
        await savePlotTemplate(kind, name, getOpts());
        recordProcess("Template", `Saved ${niceName} template "${name}"`);
        setStatus(`${niceName} template "${name}" saved`);
        close();
        await refill();
      } catch (err) {
        setStatus(`Save failed: ${err}`);
      }
    };
    doSaveBtn.addEventListener("click", () => void commit());
    nameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void commit();
    });
  });

  const delBtn = document.createElement("button");
  delBtn.className = "plot-export-btn";
  delBtn.textContent = "🗑";
  delBtn.title = "Delete the selected template";
  delBtn.addEventListener("click", async () => {
    const name = select.value;
    if (!name) {
      setStatus("Pick a template in the list to delete");
      return;
    }
    try {
      await deletePlotTemplate(kind, name);
      setStatus(`Deleted template "${name}"`);
      await refill();
    } catch (err) {
      setStatus(`Delete failed: ${err}`);
    }
  });

  wrap.append(select, saveBtn, delBtn);
  void refill();
  return wrap;
}

/** Small labeled checkbox for plot property rows. */
export function checkboxField(label: string, checked: boolean, onChange: (v: boolean) => void): HTMLElement {
  const wrap = document.createElement("label");
  wrap.className = "chk-field";
  const box = document.createElement("input");
  box.type = "checkbox";
  box.checked = checked;
  box.addEventListener("change", () => onChange(box.checked));
  wrap.appendChild(box);
  wrap.appendChild(document.createTextNode(label));
  return wrap;
}

export interface ZoneChoice {
  zoneName: string; // '*' = whole well
  depthMin: number | null;
  depthMax: number | null;
}

/** Sets a select's value only when that option actually exists (zone names differ
 *  across wells; curve lists change as modules run). */
export function trySelect(select: HTMLSelectElement, value: string | undefined): void {
  if (!value) return;
  if (Array.from(select.options).some((o) => o.value === value)) select.value = value;
}

export function curveSelect(names: string[], selected: string): HTMLSelectElement {
  const select = document.createElement("select");
  select.className = "form-control";
  const all = names.includes(selected) ? names : [selected, ...names];
  for (const name of all) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    if (name === selected) option.selected = true;
    select.appendChild(option);
  }
  return select;
}

/** Curve select seeded from a preference list (e.g. ["PERM", "KLOGH", "K"]): the first
 *  preferred name present in the catalog wins. When NONE is present, the first preferred
 *  name stays selected anyway — curveSelect prepends it as a visible option — so the run
 *  fails loudly with the backend's own "curve has no data in this well" instead of
 *  silently defaulting to option 0 of the catalog (deterministically GR, whose gAPI range
 *  is numerically indistinguishable from mD and computes a fully plausible wrong answer). */
export function preferredCurveSelect(names: string[], preferred: string[]): HTMLSelectElement {
  return curveSelect(names, preferred.find((p) => names.includes(p)) ?? preferred[0] ?? names[0] ?? "");
}

export async function loadCurveNames(): Promise<string[]> {
  const catalog = await listCurveCatalog();
  return catalog.map((c) => c.name);
}

/** Adaptive formatter for CURVE VALUES (not depths or counts). Picks decimals from the
 *  value's magnitude so small readings keep resolution — perm 0.003 stays "0.003" instead
 *  of the "0.00" a blanket toFixed(2) produces — and large ones aren't buried in decimals
 *  (RT → "2151", not "2150.73"). Extreme magnitudes fall back to scientific notation.
 *  Optionally appends a unit. Default 4 significant figures. */
export function formatValue(v: number, opts?: { unit?: string | null; sig?: number }): string {
  const sig = opts?.sig ?? 4;
  let text: string;
  if (!Number.isFinite(v)) {
    return "—";
  } else if (v === 0) {
    text = "0";
  } else {
    const a = Math.abs(v);
    if (a >= 1e6 || a < 1e-4) {
      text = v.toExponential(Math.max(0, sig - 1));
    } else {
      const decimals = Math.min(6, Math.max(0, sig - 1 - Math.floor(Math.log10(a))));
      text = v.toFixed(decimals);
      if (text.includes(".")) text = text.replace(/0+$/, "").replace(/\.$/, "");
    }
  }
  const unit = opts?.unit?.trim();
  return unit ? `${text} ${unit}` : text;
}

/** Curve name → unit (non-empty) from the catalog, for readouts and axis labels that have
 *  a curve name but not its unit. Units the catalog reports as null/blank are omitted so a
 *  lookup miss (no unit) is indistinguishable from an absent curve. */
export async function loadCurveUnits(): Promise<Map<string, string>> {
  const catalog = await listCurveCatalog();
  const units = new Map<string, string>();
  for (const c of catalog) {
    const u = c.units?.trim();
    if (u) units.set(c.name, u);
  }
  return units;
}

export interface ZoneSelect {
  select: HTMLSelectElement;
  current: () => ZoneChoice;
  /** Unsubscribes the top-interval follower — call from the panel's dispose. */
  dispose: () => void;
}

/** The zone select's option value for the Wells & Tops pane's selected top interval.
 *  Exported so panels can tell "windowed to a top interval" apart from a named zone
 *  (the ZoneChoice's zoneName is the top's name in that case, which could collide). */
export const TOP_OPTION = "@top";

/** Zone dropdown: "All depth" plus the well's zones. Selecting a zone windows the data
 *  and targets that zone for parameter writes. When a top is selected in the Wells &
 *  Tops pane, a "Top X (min–max)" option appears, is auto-selected, and fires `change`
 *  so the plot reloads windowed to it. */
export async function buildZoneSelect(well: WellSummary): Promise<ZoneSelect> {
  let zones: ZoneEntry[] = [];
  try {
    zones = await listZones(well.well_id);
  } catch {
    zones = [];
  }
  const select = document.createElement("select");
  select.className = "form-control";
  const allOption = document.createElement("option");
  allOption.value = "*";
  allOption.textContent = "All depth (zone *)";
  select.appendChild(allOption);
  for (const zone of zones) {
    const option = document.createElement("option");
    option.value = zone.zone_name;
    option.textContent = `${zone.zone_name} (${zone.top_depth.toFixed(0)}–${zone.bottom_depth.toFixed(0)})`;
    select.appendChild(option);
  }

  let interval: TopInterval | null = null;
  const applyInterval = (iv: TopInterval | null, fireChange: boolean) => {
    interval = iv && iv.wellId === well.well_id ? iv : null;
    const existing = Array.from(select.options).find((o) => o.value === TOP_OPTION);
    const wasOnTop = select.value === TOP_OPTION;
    if (!interval) {
      existing?.remove();
      if (wasOnTop) {
        select.value = "*";
        if (fireChange) select.dispatchEvent(new Event("change"));
      }
      return;
    }
    const label = `Top ${interval.topName} (${interval.depthMin.toFixed(0)}–${interval.depthMax?.toFixed(0) ?? "TD"})`;
    if (existing) {
      existing.textContent = label;
    } else {
      const option = document.createElement("option");
      option.value = TOP_OPTION;
      option.textContent = label;
      select.insertBefore(option, select.firstChild);
    }
    select.value = TOP_OPTION;
    if (fireChange) select.dispatchEvent(new Event("change"));
  };

  applyInterval(appState.selectedInterval.get(), false);
  // subscribe() fires immediately with the current value, which applyInterval above
  // already handled — skip that first call so building a panel never fires `change`.
  let first = true;
  const unsub = appState.selectedInterval.subscribe((iv) => {
    if (first) {
      first = false;
      return;
    }
    applyInterval(iv, true);
  });

  const current = (): ZoneChoice => {
    if (select.value === TOP_OPTION && interval) {
      return { zoneName: interval.topName, depthMin: interval.depthMin, depthMax: interval.depthMax };
    }
    const zone = zones.find((z) => z.zone_name === select.value);
    return zone
      ? { zoneName: zone.zone_name, depthMin: zone.top_depth, depthMax: zone.bottom_depth }
      : { zoneName: "*", depthMin: null, depthMax: null };
  };
  return { select, current, dispose: unsub };
}

// --- Multi-well plot context (shared by crossplot + histogram) ---------------
// Extra wells drawn as a display-only layer behind the active well. The two
// correctness-critical rules live HERE, once: (1) zone/top windows are resolved
// per well BY NAME in that well's own depth frame — a zone sits at different
// measured depths in every well, and reusing the active well's window would
// slice arbitrary rock; (2) a total point budget with per-well stride decimation
// keeps a 2,000-well scope drawable.

/** Concatenates value arrays for pooled auto-ranging, so context wells aren't
 *  clipped by a window fitted to the active well alone. */
export function concatValues(head: Float32Array, rest: Float32Array[]): Float32Array {
  let n = head.length;
  for (const a of rest) n += a.length;
  const out = new Float32Array(n);
  out.set(head, 0);
  let off = head.length;
  for (const a of rest) {
    out.set(a, off);
    off += a.length;
  }
  return out;
}

/** The active zone choice resolved in ANOTHER well's own depth frame: the same-NAMED
 *  zone (or top interval, Wells & Tops convention: this top down to the next, last →
 *  TD) from that well's tables. null = this well doesn't carry the zone/top → skip. */
export async function contextZoneWindow(
  zoneSel: ZoneSelect,
  wellId: string,
): Promise<[number | null, number | null] | null> {
  if (zoneSel.select.value === "*") return [null, null];
  if (zoneSel.select.value === TOP_OPTION) {
    const iv = appState.selectedInterval.get();
    if (!iv) return null;
    const tops = await listTops(wellId).catch(() => []);
    const sorted = [...tops].sort((a, b) => a.depth - b.depth);
    const i = sorted.findIndex((t) => t.top_name === iv.topName);
    if (i < 0) return null;
    return [sorted[i].depth, sorted[i + 1]?.depth ?? null];
  }
  const zones = await listZones(wellId).catch(() => []);
  const z = zones.find((q) => q.zone_name === zoneSel.current().zoneName);
  return z ? [z.top_depth, z.bottom_depth] : null;
}

/** One fetched context well: its requested curves screened and reduced by one shared
 * source-index vector, keyed by upper-cased curve name. */
export interface ContextLayerData {
  wellId: string;
  name: string;
  color: string;
  depth: Float32Array;
  series: Map<string, Float32Array>;
  reduction: ReductionManifest;
}

export interface ContextFetchOutcome {
  layers: ContextLayerData[];
  /** Decimated sample rows actually held, across all layers. */
  shown: number;
  decimated: boolean;
  /** Wells dropped: no matching zone/top, no data for the curves, or a fetch error. */
  skipped: number;
  /** Per-well absence is explicit; an all-NaN required curve is not represented. */
  absent: { wellId: string; reason: string; quota: 0 }[];
  /** Present only when the total budget cannot retain required endpoints. */
  refusal: string | null;
}

/** Fetches the context wells' curves, concurrency-limited and cancellable: `isStale()`
 *  is checked after every await, and a stale call returns null without touching anything.
 *  Every requested curve must be present in a well or that well is skipped — a layer with
 *  X but not Y would draw a broken cloud. */
export async function fetchContextLayers(args: {
  ids: string[];
  names: string[];
  curves: string[];
  windowFor: (wellId: string) => Promise<[number | null, number | null] | null>;
  budget: number;
  concurrency?: number;
  isStale: () => boolean;
}): Promise<ContextFetchOutcome | null> {
  const { ids, names, curves, windowFor, budget, isStale } = args;
  interface Candidate {
    wellId: string;
    name: string;
    color: string;
    depth: Float32Array;
    arrays: Float32Array[];
  }
  const candidates: (Candidate | null)[] = new Array(ids.length).fill(null);
  const absent: { wellId: string; reason: string; quota: 0 }[] = [];
  let next = 0;
  const worker = async (): Promise<void> => {
    while (next < ids.length && !isStale()) {
      const i = next++;
      try {
        const win = await windowFor(ids[i]);
        if (isStale()) return;
        if (!win) {
          absent.push({ wellId: ids[i], reason: "no matching zone or top interval", quota: 0 });
          continue;
        }
        const series = await getCurveData(ids[i], curves, win[0], win[1]);
        if (isStale()) return;
        const byName = new Map(series.map((item) => [item.curve_name, item]));
        const required = curves.map((curve) => byName.get(curve.toUpperCase()));
        if (required.some((item) => !item)) {
          absent.push({ wellId: ids[i], reason: "one or more required curves are absent", quota: 0 });
          continue;
        }
        const present = required as TrackCurveSeries[];
        candidates[i] = {
          wellId: ids[i],
          name: names[i],
          color: FACIES_PALETTE[i % FACIES_PALETTE.length],
          depth: present[0].depth,
          arrays: present.map((item) => item.value),
        };
      } catch (error) {
        absent.push({ wellId: ids[i], reason: `context fetch failed: ${String(error)}`, quota: 0 });
      }
    }
  };
  await Promise.all(Array.from({ length: Math.min(args.concurrency ?? 8, ids.length) }, () => worker()));
  if (isStale()) return null;
  const fetched = candidates.filter((candidate): candidate is Candidate => candidate !== null);
  const allocation = allocateFinitePairBudget(
    fetched.map((candidate) => ({ wellId: candidate.wellId, channels: [candidate.depth, ...candidate.arrays] })),
    budget,
  );
  absent.push(...allocation.absent);
  if (allocation.refusal) {
    for (const candidate of fetched) {
      if (allocation.absent.some((item) => item.wellId === candidate.wellId)) continue;
      absent.push({ wellId: candidate.wellId, reason: allocation.refusal, quota: 0 });
    }
    return {
      layers: [],
      shown: 0,
      decimated: false,
      skipped: absent.length,
      absent,
      refusal: allocation.refusal,
    };
  }
  const byWell = new Map(allocation.wells.map((item) => [item.wellId, item]));
  const layers: ContextLayerData[] = [];
  let shown = 0;
  let decimated = false;
  for (const candidate of fetched) {
    const assigned = byWell.get(candidate.wellId);
    if (!assigned) continue;
    const out = new Map<string, Float32Array>();
    for (let channel = 0; channel < curves.length; channel++) {
      out.set(
        curves[channel].toUpperCase(),
        Float32Array.from(assigned.sourceIndices.map((sourceIndex) => candidate.arrays[channel][sourceIndex])),
      );
    }
    layers.push({
      wellId: candidate.wellId,
      name: candidate.name,
      color: candidate.color,
      depth: Float32Array.from(assigned.sourceIndices.map((sourceIndex) => candidate.depth[sourceIndex])),
      series: out,
      reduction: assigned.manifest,
    });
    shown += assigned.manifest.displayedCount;
    if (assigned.manifest.displayedCount < assigned.manifest.originalCount) decimated = true;
  }
  return {
    layers,
    shown,
    decimated,
    skipped: absent.length,
    absent,
    refusal: null,
  };
}

/** Human line for the scope row: "Context: 41 wells · ~58,200 pts (decimated) · 3 skipped …". */
export function describeContextOutcome(o: ContextFetchOutcome): string {
  if (o.refusal) return `Context refused: ${o.refusal}`;
  const original = o.layers.reduce((sum, layer) => sum + layer.reduction.originalCount, 0);
  const strides = [...new Set(o.layers.map((layer) => layer.reduction.stride))].sort((a, b) => a - b);
  const strideText = strides.length === 1 ? `${strides[0]}` : `${strides[0]}–${strides[strides.length - 1]}`;
  const forced = o.layers.filter((layer) => layer.reduction.endpointsForced).length;
  return (
    `Context: ${o.layers.length} well${o.layers.length === 1 ? "" : "s"} · ~${o.shown.toLocaleString()} pts` +
    (o.decimated
      ? ` (reduced ${original.toLocaleString()}→${o.shown.toLocaleString()}; stride ${strideText}; final endpoint forced in ${forced} well${forced === 1 ? "" : "s"})`
      : "") +
    (o.skipped ? ` · ${o.skipped} absent (reasons retained per well)` : "")
  );
}

/** One "pick → parameter" row: colored swatch, picked-value readout, editable target
 *  parameter name, and a Set button that writes the value to the chosen zone. */
export function pickRow(
  label: string,
  color: string,
  defaultParam: string,
  well: WellSummary,
  getZone: () => ZoneChoice,
  setStatus: (text: string) => void,
): { row: HTMLElement; setValue: (v: number) => void; getValue: () => number } {
  let value = NaN;

  const row = document.createElement("div");
  row.className = "pick-row";

  const swatch = document.createElement("span");
  swatch.className = "pick-swatch";
  swatch.style.background = color;

  const name = document.createElement("span");
  name.className = "pick-label";
  name.textContent = label;

  const readout = document.createElement("span");
  readout.className = "pick-value";
  readout.textContent = "—";

  const paramIn = document.createElement("input");
  paramIn.className = "form-control pick-param";
  paramIn.value = defaultParam;
  paramIn.placeholder = "Parameter";

  const setBtn = document.createElement("button");
  setBtn.className = "form-run-btn pick-set";
  setBtn.textContent = "Set";
  setBtn.addEventListener("click", async () => {
    const param = paramIn.value.trim().toUpperCase();
    if (!param || Number.isNaN(value)) return;
    const zone = getZone();
    try {
      await setZoneParam(well.well_id, zone.zoneName, param, value, null);
      setStatus(`${param} = ${value.toPrecision(4)} set on zone '${zone.zoneName}' of ${well.well_name}`);
    } catch (err) {
      // Never report success on a rejected write — a swallowed failure makes the user
      // believe GR_MA/RW landed on the zone when it never reached the module runs.
      setStatus(`Failed to set ${param}: ${err}`);
    }
  });

  row.appendChild(swatch);
  row.appendChild(name);
  row.appendChild(readout);
  row.appendChild(paramIn);
  row.appendChild(setBtn);

  return {
    row,
    setValue: (v: number) => {
      value = v;
      readout.textContent = v.toPrecision(4);
    },
    getValue: () => value,
  };
}

/** Default target parameter names per picked curve, so the histogram opens ready to
 *  write GR_MA/GR_SH when GR is selected, etc. */
export function defaultPickParams(curve: string): [string, string] {
  switch (curve.toUpperCase()) {
    case "GR":
      return ["GR_MA", "GR_SH"];
    case "NPHI":
      return ["NPHI_MA", "NPHI_SH"];
    case "RHOB":
      return ["RHO_MA", "RHO_SH"];
    case "DT":
      return ["DT_MA", "DT_SH"];
    case "RES_DEEP":
      return ["RW", "RT_SH"];
    default:
      return ["", ""];
  }
}
