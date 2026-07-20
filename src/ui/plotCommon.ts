import { deleteDocument, listCurveCatalog, listDocuments, listZones, saveDocument, setZoneParam, type WellSummary, type ZoneEntry } from "../ipc";
import { appState, type TopInterval } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";

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

/** The zone select's option value for the Wells & Tops pane's selected top interval. */
const TOP_OPTION = "@top";

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
    await setZoneParam(well.well_id, zone.zoneName, param, value, null);
    setStatus(`${param} = ${value.toPrecision(4)} set on zone '${zone.zoneName}' of ${well.well_name}`);
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
