import {
  deleteDocument,
  finalizePlotWriteProvenance,
  getCurveData,
  plotBindingSnapshotForChannels,
  listCurveCatalog,
  listDocuments,
  listTops,
  listZoneParams,
  listZones,
  resolveWellScope,
  savePlotState,
  setZoneParam,
  type PlotWriteProvenanceInput,
  type PersistedPlotState,
  type PlotChannelIntent,
  type ResolvedPlotCurve,
  type TrackCurveSeries,
  type WellSummary,
  type ZoneEntry,
} from "../ipc";
import { appState, type TopInterval } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { FACIES_PALETTE } from "./plotCanvas";
import {
  allocateFinitePairBudget,
  DepthGridReconciliationError,
  halfOpenDepthIndices,
  reconcileDepthChannels,
  type DepthGridReconciliation,
  type DepthStepManifest,
  type PlotReductionExport,
  type ReductionManifest,
} from "./plotTypes";
import type { PlotAxisRangeExport } from "./axisRange";
import {
  applyPlotRecordLimit,
  plotRecordCountReduction,
  plotRecordLimit,
  reducePlotLabel,
} from "./plotLimits";
import {
  beginPlotAsyncGeneration,
  isPlotAsyncGenerationCurrent,
  type PlotAsyncOperationId,
} from "./plotAsync";
import type { WellScope } from "./wellScope";

import { ensureSessionOperator } from "./runCustody";
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
  /** Complete typed state used by named sessions and exports. It throws while a
   * required represented-well binding is absent, so callers refuse instead of guess. */
  getPersistedState?: () => PersistedPlotState;
  /** Resolves after every channel represented by the initial panel state has completed
   * its first binding pass. Session restore waits for this before comparing custody. */
  bindingReady?: Promise<void>;
  /** Opens this plot's Properties dialog. The workspace puts it at the top of the pane's
   *  right-click menu — the canvas no longer swallows right-click to open it directly,
   *  which had hidden the window actions (split/float/export) on every plot. */
  openProperties?: () => void;
}

export interface DepthReframeHandoff {
  event: "sandibumi:open-reframe";
  actionLabel: "Open Reframe";
  reason: string;
  wellIds: string[];
  curves: string[];
  automaticResampling: false;
}

export interface DepthReframeHandoffControl {
  el: HTMLDivElement;
  show: (handoff: DepthReframeHandoff | null) => void;
  clear: () => void;
}

export function depthReframeHandoff(
  error: unknown,
  wellIds: string[],
  curves: string[],
): DepthReframeHandoff | null {
  if (!(error instanceof DepthGridReconciliationError)) return null;
  return {
    event: "sandibumi:open-reframe",
    actionLabel: error.actionLabel,
    reason: error.message,
    wellIds: [...new Set(wellIds.map((wellId) => wellId.trim()).filter(Boolean))],
    curves: [...new Set(curves.map((curve) => curve.trim()).filter(Boolean))],
    automaticResampling: error.automaticResampling,
  };
}

export function mergeDepthReframeHandoffs(
  handoffs: DepthReframeHandoff[],
): DepthReframeHandoff | null {
  if (handoffs.length === 0) return null;
  if (handoffs.length === 1) return handoffs[0];
  return {
    event: "sandibumi:open-reframe",
    actionLabel: "Open Reframe",
    reason:
      `${handoffs.length} selected wells have irregular depth grids or steps that are not exact integer multiples; ` +
      "use Reframe to create explicit shared depth frames",
    wellIds: [...new Set(handoffs.flatMap((handoff) => handoff.wellIds))],
    curves: [...new Set(handoffs.flatMap((handoff) => handoff.curves))],
    automaticResampling: false,
  };
}

export function buildDepthReframeHandoff(
  setStatus: (text: string) => void,
): DepthReframeHandoffControl {
  const el = document.createElement("div");
  el.className = "depth-reframe-handoff";
  el.style.display = "none";
  const message = document.createElement("span");
  message.className = "depth-reframe-handoff-message";
  const button = document.createElement("button");
  button.type = "button";
  button.className = "plot-export-btn";
  let current: DepthReframeHandoff | null = null;
  const clear = () => {
    current = null;
    message.textContent = "";
    button.textContent = "";
    el.style.display = "none";
  };
  button.addEventListener("click", () => {
    if (!current) return;
    window.dispatchEvent(new CustomEvent(current.event, { detail: current }));
    setStatus("Opening Reframe for an explicit depth-grid decision; no plot data were resampled.");
  });
  el.append(message, button);
  return {
    el,
    show: (handoff) => {
      if (!handoff) {
        clear();
        return;
      }
      current = handoff;
      message.textContent = `Plot refused: ${handoff.reason} No data were resampled.`;
      button.textContent = handoff.actionLabel;
      el.style.display = "";
    },
    clear,
  };
}

export function registerDepthReframeRoute(openReframe: () => void): () => void {
  const listener = () => openReframe();
  window.addEventListener("sandibumi:open-reframe", listener);
  return () => window.removeEventListener("sandibumi:open-reframe", listener);
}

function bindingKey(binding: PersistedPlotState["bindings"][number]): string {
  return binding.intent.channel.trim().toUpperCase();
}

function canonicalBindings(state: PersistedPlotState): string {
  return JSON.stringify(
    [...state.bindings]
      .sort((left, right) => bindingKey(left).localeCompare(bindingKey(right)))
      .map((binding) => ({
        intent: binding.intent,
        resolved: [...binding.resolved].sort((left, right) => left.well_id.localeCompare(right.well_id)),
      })),
  );
}

function canonicalAxisRanges(state: PersistedPlotState): string {
  return JSON.stringify(
    [...(state.axis_ranges ?? [])].sort((left, right) => left.axis.localeCompare(right.axis)),
  );
}

export function buildPersistedPlotState(
  plotType: string,
  options: Record<string, unknown>,
  wellIds: string[],
  intents: PlotChannelIntent[],
  axisRanges: PlotAxisRangeExport[],
): PersistedPlotState {
  const represented = [...new Set(wellIds.map((wellId) => wellId.trim()).filter(Boolean))];
  if (represented.length === 0) throw new Error("plot state has no represented wells");
  const bindings = plotBindingSnapshotForChannels(represented, intents);
  for (const binding of bindings) {
    if (!binding.intent.required) continue;
    const resolved = new Set(binding.resolved.map((curve) => curve.well_id));
    const missing = represented.filter((wellId) => !resolved.has(wellId));
    if (missing.length > 0) {
      throw new Error(
        `required channel '${binding.intent.semantic_request}' is unresolved for represented well(s): ${missing.join(", ")}`,
      );
    }
  }
  if (axisRanges.length === 0) throw new Error("plot state has no resolved axis ranges");
  const axes = new Set<string>();
  for (const range of axisRanges) {
    const axis = range.axis.trim().toLowerCase();
    if (!axis) throw new Error("plot state has an unnamed axis range");
    if (axes.has(axis)) throw new Error(`plot state repeats axis '${range.axis}'`);
    axes.add(axis);
    if (!Number.isFinite(range.min) || !Number.isFinite(range.max) || range.min === range.max) {
      throw new Error(`plot axis '${range.axis}' requires two distinct finite display limits`);
    }
  }
  return {
    schema_version: 1,
    plot_type: plotType,
    well_ids: represented,
    options,
    bindings,
    axis_ranges: axisRanges,
  };
}

/** A named session may reopen only onto the same concrete curve answers. Templates
 * intentionally skip this comparison and re-resolve their semantic requests. */
export function assertPlotStateRestored(
  expected: PersistedPlotState,
  actual: PersistedPlotState,
): void {
  if (
    expected.plot_type !== actual.plot_type ||
    canonicalBindings(expected) !== canonicalBindings(actual) ||
    ((expected.axis_ranges?.length ?? 0) > 0 && canonicalAxisRanges(expected) !== canonicalAxisRanges(actual))
  ) {
    throw new Error(
      `saved ${expected.plot_type} refused: a concrete curve binding, source revision, or resolved axis range changed`,
    );
  }
}

export interface PlotRestoreGate {
  /** Saved curve/zone selections, seeded into every rebuild. They outlive a refusal on
   * purpose: only the concrete-binding comparison is given up, never the user's picks. */
  readonly initialOptions: Record<string, string> | undefined;
  /** True while the strict saved-vs-actual comparison still stands. */
  readonly pending: boolean;
  /** True after a failed build attempt, until the next one begins. */
  readonly failed: boolean;
  beginAttempt(): void;
  /** Runs the strict comparison while the expectation stands. `actual()` itself throws
   * while a required binding is unresolved; either throw is a refusal, and a throw
   * leaves the expectation standing — only {@link refuse} or success consumes it. */
  validate(actual: () => PersistedPlotState): void;
  /** Records a failed build attempt and drops the expectation: refused once, not latched. */
  refuse(): void;
  /** Whether a well-selection broadcast rebuilds the pane. `follows` = the global well
   * lock is on OR this is the working pane; a healthy background pane stays put. */
  shouldRebuild(input: { built: boolean; sameWell: boolean; follows: boolean }): boolean;
}

/** Per-pane policy for reopening a saved plot (session open, app-start autosave restore).
 * While the expectation stands the pane may only reopen onto the same concrete curve
 * answers ({@link assertPlotStateRestored}). A refusal is shown ONCE and then drops the
 * expectation — a pane latched on a dead error has no recovery path — so the next
 * well-selection broadcast rebuilds it fresh, re-resolving its semantic requests the way
 * a template does, still seeded with the saved selections. */
export function createPlotRestoreGate(expected: PersistedPlotState | undefined): PlotRestoreGate {
  let expectation = expected;
  let failed = false;
  return {
    initialOptions: expected?.options as Record<string, string> | undefined,
    get pending() {
      return expectation !== undefined;
    },
    get failed() {
      return failed;
    },
    beginAttempt() {
      failed = false;
    },
    validate(actual) {
      if (!expectation) return;
      assertPlotStateRestored(expectation, actual());
      expectation = undefined;
    },
    refuse() {
      failed = true;
      expectation = undefined;
    },
    shouldRebuild({ built, sameWell, follows }) {
      if (failed) return true;
      if (!built) return true;
      if (sameWell) return false;
      return follows;
    },
  };
}

function persistedOptions<T>(raw: unknown): Partial<T> {
  if (
    raw &&
    typeof raw === "object" &&
    "schema_version" in raw &&
    "options" in raw &&
    (raw as { schema_version?: unknown }).schema_version === 1
  ) {
    return (raw as PersistedPlotState).options as Partial<T>;
  }
  return (raw ?? {}) as Partial<T>;
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

/** The core measurement a log curve is calibrated against, or "" for none. Exact map
 *  entries first (NPHI→CPOR is deliberate and not a family), then the mnemonic FAMILY:
 *  the exact list froze while modules kept naming their outputs (PHIE_DEN, PHIT_DEN,
 *  PERM_RT …), so the classic por-perm calibration plot silently lost its plugs on any
 *  porosity the density module wrote. A family prefix is the calibration statement —
 *  every PHIE_ or PHIT_ curve is a porosity a plug porosity checks, and every PERM_
 *  curve a permeability a plug permeability checks. */
export function coreOverlayItem(mnemonic: string): string {
  const upper = mnemonic.trim().toUpperCase();
  const exact = CORE_OVERLAY_MAP[upper];
  if (exact) return exact;
  if (/^(PHIE|PHIT|PHID)(_|$)/.test(upper)) return "CPOR";
  if (/^PERM(_|$)/.test(upper)) return "CPERM";
  if (/^(SWE|SWT)(_|$)/.test(upper)) return "CSW";
  return "";
}

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
    return doc ? persistedOptions<T>(JSON.parse(doc.json)) : {};
  } catch {
    return {};
  }
}

/** Fire-and-forget save of a plot kind's properties — new panels of that kind open
 *  with them. */
export function savePlotProps(kind: string, state: PersistedPlotState): Promise<void> {
  return savePlotState("plotprops", kind, state);
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

export async function savePlotTemplate(kind: string, name: string, state: PersistedPlotState): Promise<void> {
  await savePlotState(plotTemplateDocType(kind), name, state);
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
  getState: () => PersistedPlotState,
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
      applyOpts(persistedOptions<T>(JSON.parse(doc.json)));
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
        await savePlotTemplate(kind, name, getState());
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
  /** Apply the application's selected top interval; governed plot invalidation owns later changes. */
  applySelectedInterval: (interval: TopInterval | null, fireChange: boolean) => void;
  /** Unsubscribes the top-interval follower — call from the panel's dispose. */
  dispose: () => void;
}

export interface ZoneSelectOptions {
  /** Defaults true for legacy/non-governed consumers. Governed plots subscribe through SB-PLT-019. */
  followSelectedInterval?: boolean;
}

/** The zone select's option value for the Wells & Tops pane's selected top interval.
 *  Exported so panels can tell "windowed to a top interval" apart from a named zone
 *  (the ZoneChoice's zoneName is the top's name in that case, which could collide). */
export const TOP_OPTION = "@top";

/** Zone dropdown: "All depth" plus the well's zones. Selecting a zone windows the data
 *  and targets that zone for parameter writes. When a top is selected in the Wells &
 *  Tops pane, a "Top X (min–max)" option appears, is auto-selected, and fires `change`
 *  so the plot reloads windowed to it. */
export async function buildZoneSelect(
  well: WellSummary,
  options: ZoneSelectOptions = {},
): Promise<ZoneSelect> {
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
  let unsub = (): void => {};
  if (options.followSelectedInterval !== false) {
    let first = true;
    unsub = appState.selectedInterval.subscribe((iv) => {
      if (first) {
        first = false;
        return;
      }
      applyInterval(iv, true);
    });
  }

  const current = (): ZoneChoice => {
    if (select.value === TOP_OPTION && interval) {
      return { zoneName: interval.topName, depthMin: interval.depthMin, depthMax: interval.depthMax };
    }
    const zone = zones.find((z) => z.zone_name === select.value);
    return zone
      ? { zoneName: zone.zone_name, depthMin: zone.top_depth, depthMax: zone.bottom_depth }
      : { zoneName: "*", depthMin: null, depthMax: null };
  };
  return { select, current, applySelectedInterval: applyInterval, dispose: unsub };
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
    const tops = await listTops(wellId);
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
  depthStep: DepthStepManifest;
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
  /** Incompatible depth grids remain absent until the user explicitly opens Reframe. */
  depthReframeHandoffs: DepthReframeHandoff[];
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
  isStale: () => boolean;
}): Promise<ContextFetchOutcome | null> {
  const { ids, names, curves, windowFor, budget, isStale } = args;
  interface Candidate {
    wellId: string;
    name: string;
    color: string;
    depth: Float32Array;
    arrays: Float32Array[];
    depthStep: DepthStepManifest;
  }
  const candidates: (Candidate | null)[] = new Array(ids.length).fill(null);
  const depthReframeByIndex: (DepthReframeHandoff | null)[] = new Array(ids.length).fill(null);
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
        let reconciled: DepthGridReconciliation;
        try {
          reconciled = reconcileDepthChannels(
            present.map((item) => ({ depth: item.depth, values: item.value })),
          );
        } catch (error) {
          depthReframeByIndex[i] = depthReframeHandoff(error, [ids[i]], curves);
          absent.push({ wellId: ids[i], reason: String(error), quota: 0 });
          continue;
        }
        const intervalIndices = halfOpenDepthIndices(reconciled.depth, win[0], win[1]);
        const depth = Float32Array.from(intervalIndices.map((index) => reconciled.depth[index]));
        const arrays = reconciled.channels.map((channel) =>
          Float32Array.from(intervalIndices.map((index) => channel[index])));
        candidates[i] = {
          wellId: ids[i],
          name: names[i],
          color: FACIES_PALETTE[i % FACIES_PALETTE.length],
          depth,
          arrays,
          depthStep: {
            coarsestStep: reconciled.coarsestStep,
            decimationFactors: reconciled.decimationFactors,
            mode: reconciled.mode,
            intervalClosure: reconciled.intervalClosure,
          },
        };
      } catch (error) {
        absent.push({ wellId: ids[i], reason: `context fetch failed: ${String(error)}`, quota: 0 });
      }
    }
  };
  const workerCount = Math.min(plotRecordLimit("context_fetch_concurrency").maximum, ids.length);
  await Promise.all(Array.from({ length: workerCount }, () => worker()));
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
      depthReframeHandoffs: depthReframeByIndex.filter(
        (handoff): handoff is DepthReframeHandoff => handoff !== null,
      ),
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
      depthStep: candidate.depthStep,
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
    depthReframeHandoffs: depthReframeByIndex.filter(
      (handoff): handoff is DepthReframeHandoff => handoff !== null,
    ),
    refusal: null,
  };
}

/** What one panel holds after a context load finishes. The panel keeps its own `ctx*`
 *  variables — this is only the set of values a completed load replaces. */
export interface ContextReloadState<L> {
  layers: L[];
  wellIds: string[];
  reductionManifest: PlotReductionExport | null;
  info: string;
}

export interface ContextReload {
  /** Re-resolve the scope and refetch. Safe to call while an earlier call is in flight. */
  reload: () => Promise<void>;
  /** Abandon whatever is in flight (panel dispose, or a newer load taking over). */
  cancel: () => void;
}

/**
 * AUDIT-2026-08-20 finding 57. The multi-well context reload existed THREE times — histogram,
 * Pickett and crossplot — about sixty near-verbatim lines each, the doc comment included. Four
 * separate concerns had been added to all three copies rather than to one: scope resolution, a
 * generation token with three staleness checks, depth-handoff merging, and `ctxWellIds`.
 *
 * Only two things genuinely differ between the panels: which CURVES to fetch, and how a fetched
 * layer PROJECTS onto that panel's own layer type. Everything else is identical — and in
 * particular every staleness check is, which is exactly the code three copies must not diverge
 * in. A dropped check does not throw: it draws an overlay from the well the user already moved
 * off, with the current well's name on it.
 *
 * The panel keeps its own `ctx*` variables and receives new values through `apply`, so the logic
 * is shared without moving sixty read sites across three files. Note `apply` is deliberately not
 * called before the fetch: a reload REPLACES the overlay on success and leaves the previous one
 * on screen while it loads, so only the reduction manifest is reset up front.
 */
export function createContextReload<L>(spec: {
  /** Reduction-manifest key: "histogram" | "pickett" | "crossplot". */
  kind: string;
  /** Status-line prefix: "Histogram" | "Pickett" | "Crossplot". */
  label: string;
  /** This panel's registered token operation — see PLOT_ASYNC_LOAD_REGISTRY. */
  operation: PlotAsyncOperationId;
  well: WellSummary;
  scope: WellScope;
  zoneSel: ZoneSelect;
  handoff: DepthReframeHandoffControl;
  /** Read fresh on every call — the panel's curve selects may have changed since the last. */
  curves: () => string[];
  project: (layer: ContextLayerData) => L;
  /** Whether the panel currently holds layers, so a clear knows whether it must redraw. */
  hadLayers: () => boolean;
  apply: (next: ContextReloadState<L>) => void;
  setPendingManifest: (manifest: PlotReductionExport | null) => void;
  setStatus: (text: string) => void;
  updateScopeUi: () => void;
  redraw: () => void;
  /** Crossplot alone refreshes its statistics records as soon as the scope is known. */
  afterScope?: () => void;
}): ContextReload {
  let generation = 0;
  const reload = async (): Promise<void> => {
    const token = beginPlotAsyncGeneration(spec.operation, ++generation);
    spec.handoff.clear();
    let resolvedIds: string[];
    try {
      resolvedIds = await resolveWellScope(spec.scope.backend());
    } catch (error) {
      if (isPlotAsyncGenerationCurrent(token, generation)) {
        spec.setStatus(`${spec.label} scope refused: ${error}`);
      }
      return;
    }
    spec.afterScope?.();
    if (!isPlotAsyncGenerationCurrent(token, generation)) return;
    const ids = resolvedIds.filter((id) => id !== spec.well.well_id);
    if (ids.length === 0) {
      // Scope narrowed back to the active well: clear the overlay so the panel behaves
      // byte-identically to one that never had context wells.
      const had = spec.hadLayers();
      spec.apply({ layers: [], wellIds: [], reductionManifest: null, info: "" });
      spec.handoff.clear();
      spec.updateScopeUi();
      if (had) spec.redraw();
      return;
    }
    const curves = spec.curves();
    spec.setPendingManifest(contextReductionExport(spec.kind, null, resolvedIds.length));
    spec.setStatus(
      `${spec.label}: loading ${ids.length} context well${ids.length === 1 ? "" : "s"}…`,
    );
    const outcome = await fetchContextLayers({
      ids,
      names: spec.scope.namesFor(ids),
      curves,
      windowFor: (id) => contextZoneWindow(spec.zoneSel, id),
      budget: plotRecordLimit("context_point_budget").maximum,
      isStale: () => !isPlotAsyncGenerationCurrent(token, generation),
    });
    if (!outcome) return; // superseded by a newer call (or dispose)
    const info = describeContextOutcome(outcome);
    spec.apply({
      layers: outcome.layers.map(spec.project),
      wellIds: outcome.layers.map((layer) => layer.wellId),
      reductionManifest: contextReductionExport(spec.kind, outcome, resolvedIds.length, {
        wellId: spec.well.well_id,
        name: spec.well.well_name,
      }),
      info,
    });
    spec.handoff.show(mergeDepthReframeHandoffs(outcome.depthReframeHandoffs));
    spec.updateScopeUi();
    spec.setStatus(`${spec.label} ${info.toLowerCase()}`);
    spec.redraw();
  };
  return { reload, cancel: () => void generation++ };
}

/** Human line for the scope row: "Context: 41 wells · ~58,200 pts (decimated) · 3 skipped …". */
export function describeContextOutcome(o: ContextFetchOutcome): string {
  if (o.refusal) return `Context refused: ${o.refusal}`;
  const original = o.layers.reduce((sum, layer) => sum + layer.reduction.originalCount, 0);
  const strides = [...new Set(o.layers.map((layer) => layer.reduction.stride))].sort((a, b) => a - b);
  const strideText = strides.length === 1 ? `${strides[0]}` : `${strides[0]}–${strides[strides.length - 1]}`;
  const forced = o.layers.filter((layer) => layer.reduction.endpointsForced).length;
  const depthFactors = [...new Set(o.layers.flatMap((layer) => layer.depthStep.decimationFactors))]
    .sort((a, b) => a - b);
  const depthText = depthFactors.some((factor) => factor > 1)
    ? ` · depth decimation factor${depthFactors.length === 1 ? "" : "s"} ${depthFactors.join("/")} to coarsest exact step`
    : "";
  const reframeCount = o.depthReframeHandoffs?.length ?? 0;
  return (
    `Context: ${o.layers.length} well${o.layers.length === 1 ? "" : "s"} · ~${o.shown.toLocaleString()} pts` +
    (o.decimated
      ? ` (reduced ${original.toLocaleString()}→${o.shown.toLocaleString()}; stride ${strideText}; final endpoint forced in ${forced} well${forced === 1 ? "" : "s"})`
      : "") +
    depthText +
    (o.layers.length ? " · interval [lo,hi)" : "") +
    (o.skipped ? ` · ${o.skipped} absent (reasons retained per well)` : "") +
    (reframeCount ? ` · ${reframeCount} require explicit Reframe` : "")
  );
}

/** One context-well legend, as the three quantitative plots draw it. */
export interface ContextLegend {
  /** The active well's label, already inside the declared character budget. */
  activeName: string;
  /** The context wells this legend may list, each label inside the same budget. */
  rows: { color: string; name: string }[];
  /** The disclosure line when the list was cut, or null when every well is on it. */
  remainder: string | null;
}

/**
 * The context-well legend under `context_well_legend_rows` and `context_well_name_characters`.
 *
 * `plotLimits.ts` names crossplot, histogram and Pickett as this budget's consumers, and each of
 * the three carried its own copy of the limit call, the per-row label reduction and the "N of M
 * wells" line. Three copies is three answers to "how many wells does a legend show", and the
 * disclosure is the half that must not vary: a legend that silently stops at ten reads as all of
 * them. Only the drawing stays per panel, because the three genuinely draw different swatches.
 */
export function contextLegend(
  activeName: string,
  layers: readonly { color: string; name: string }[],
): ContextLegend {
  const visible = applyPlotRecordLimit("context_well_legend_rows", layers, "well_legend");
  return {
    activeName: reducePlotLabel("context_well_name_characters", activeName, "active").displayed,
    rows: visible.displayed.map((layer) => ({
      color: layer.color,
      name: reducePlotLabel("context_well_name_characters", layer.name, layer.name).displayed,
    })),
    remainder: visible.item
      ? `context legend: ${visible.item.displayed_count} of ${visible.item.original_count} wells`
      : null,
  };
}

/** Builds the portable disclosure required whenever context points, well-name preview,
 * or on-plot legend are reduced. All represented wells stay in the point inventory,
 * including wells whose point count did not need reducing; absent wells retain their reason. */
export function contextReductionExport(
  plotType: string,
  outcome: ContextFetchOutcome | null,
  scopedWellCount: number,
  activeWell?: { wellId: string; name: string },
): PlotReductionExport | null {
  const layers = outcome?.layers ?? [];
  const pointReduced = outcome?.layers.some(
    (layer) => layer.reduction.displayedCount < layer.reduction.originalCount,
  ) ?? false;
  const legend = applyPlotRecordLimit("context_well_legend_rows", layers, "well_legend");
  const scopePreview = plotRecordCountReduction(
    "well_scope_name_preview_rows",
    scopedWellCount,
    "well_scope_name_preview",
  );
  const labelItems = legend.displayed.flatMap((layer) => {
    const item = reducePlotLabel("context_well_name_characters", layer.name, layer.wellId).item;
    return item ? [item] : [];
  });
  if (activeWell && layers.length > 0) {
    const activeItem = reducePlotLabel(
      "context_well_name_characters",
      activeWell.name,
      activeWell.wellId,
    ).item;
    if (activeItem) labelItems.unshift(activeItem);
  }
  if (!pointReduced && !legend.item && !scopePreview && labelItems.length === 0 && !outcome?.refusal) return null;

  const items: PlotReductionExport["items"] = layers.map((layer) => ({
    subject_kind: "points",
    subject_id: layer.wellId,
    original_count: layer.reduction.originalCount,
    displayed_count: layer.reduction.displayedCount,
    algorithm: layer.reduction.algorithm,
    stride: layer.reduction.stride,
    endpoints_forced: layer.reduction.endpointsForced,
  }));
  if (scopePreview) items.push(scopePreview);
  if (legend.item) items.push(legend.item);
  items.push(...labelItems);
  return {
    schema_version: 1,
    plot_type: plotType,
    items,
    absent: (outcome?.absent ?? []).map((item) => ({ subject_id: item.wellId, reason: item.reason })),
    refusal: outcome?.refusal ?? null,
  };
}

export type PlotWriteSource = Omit<PlotWriteProvenanceInput, "target">;

export function plotWriteAxis(
  channel: string,
  curve: ResolvedPlotCurve | null | undefined,
): PlotWriteProvenanceInput["x_axis"] {
  if (!curve) throw new Error(`plot-derived write is missing the concrete ${channel} axis binding`);
  return {
    channel,
    curve_id: curve.curve_id,
    mnemonic: curve.mnemonic,
    quantity: curve.quantity,
    source_unit: curve.source_unit,
    display_unit: curve.display_unit,
    conversion: curve.conversion,
    source_revision: curve.source_revision,
  };
}

function bytesToHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** Selection metadata is hashed over sorted f32 depths; sample arrays never cross IPC
 * as JSON. The provenance carries only an ID, count and SHA-256 revision. */
export async function plotWriteSelection(wellId: string): Promise<PlotWriteProvenanceInput["selection"]> {
  const selection = appState.brushedDepths.get();
  if (!selection || selection.wellId !== wellId || selection.depths.size === 0) {
    return { kind: "none", selection_id: null, member_count: 0, revision: null };
  }
  const depths = Float32Array.from([...selection.depths].sort((a, b) => a - b));
  const digest = await crypto.subtle.digest("SHA-256", depths.buffer);
  return {
    kind: "ephemeral_brushed_depths",
    selection_id: `brushed:${wellId}`,
    member_count: depths.length,
    revision: bytesToHex(new Uint8Array(digest)),
  };
}

export async function writePlotParameter(args: {
  well: WellSummary;
  zone: ZoneChoice;
  parameter: string;
  value: number;
  source: PlotWriteSource;
}): Promise<void> {
  const parameter = args.parameter.trim().toUpperCase();
  if (!parameter || !Number.isFinite(args.value)) {
    throw new Error("plot-derived parameter and value must be present and finite");
  }
  const completeSource: PlotWriteProvenanceInput = {
    ...args.source,
    target: {
      well_id: args.well.well_id,
      zone_name: args.zone.zoneName,
      parameter_name: parameter,
      value: args.value,
    },
  };
  const [sourceNote, current] = await Promise.all([
    finalizePlotWriteProvenance(completeSource),
    listZoneParams(args.well.well_id),
  ]);
  const before = current.find((entry) =>
    entry.zone_name === args.zone.zoneName && entry.param_name === parameter);
  // SB-DBM-011: an audited surface. Uninterrupted drags of the same handle collapse into
  // ONE audit entry backend-side, so this can fire per gesture without flooding the audit.
  const applyNew = async () => {
    const op = await ensureSessionOperator("Crossplot parameter pick");
    if (!op) throw new Error("edit cancelled: no session operator entered");
    return setZoneParam(
      args.well.well_id,
      args.zone.zoneName,
      parameter,
      args.value,
      sourceNote,
      op,
      "Crossplot",
    );
  };
  const applyOld = async () => {
    const op = await ensureSessionOperator("Crossplot parameter undo");
    if (!op) throw new Error("edit cancelled: no session operator entered");
    return setZoneParam(
      args.well.well_id,
      args.zone.zoneName,
      parameter,
      before?.value_num ?? null,
      before?.value_text ?? null,
      op,
      "Crossplot",
    );
  };
  await applyNew();
  pushUndo({
    label: `set ${parameter} from ${args.source.plot_type}`,
    undo: applyOld,
    redo: applyNew,
  });
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
  getSource: (target: { parameter: string; value: number }) => Promise<PlotWriteSource>,
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
      const source = await getSource({ parameter: param, value });
      await writePlotParameter({ well, zone, parameter: param, value, source });
      setStatus(`${param} = ${value.toPrecision(4)} set with plot provenance (Ctrl+Z undoes)`);
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
