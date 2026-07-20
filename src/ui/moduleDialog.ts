import {
  listCurveCatalog,
  listLogSets,
  listWells,
  runWorkflowModule,
  type ModuleSpec,
  type RunModuleRequest,
  type WellSummary,
} from "../ipc";
import { appState, defaultRunWellIds, filterByActiveGroup } from "../state";
import { formRow } from "./modal";

export interface ModulePaneCallbacks {
  /** Called after a successful run so the host can refresh the catalog/layout. */
  onRunComplete: (outputCurves: string[]) => void;
  setStatus: (text: string) => void;
}

/** Flag curves offered as Mask choices even before they first exist in the catalog,
 *  so the canonical chain (badhole → condflag → solvers with Mask) is composable in a
 *  fresh project. The catalog only lists curves that have actually been computed. */
export const MASK_CURVE_SUGGESTIONS = ["BADHOLE", "COND_FLAG"];

/** Catalog names with the well-known flag curves prepended when absent. */
export function maskCurveNames(curveNames: string[]): string[] {
  return [...MASK_CURVE_SUGGESTIONS.filter((s) => !curveNames.includes(s)), ...curveNames];
}

/** Builds the auto-generated parameter form for one module: input-curve selectors,
 *  option dropdowns, and validated numeric parameters — all straight from the manifest
 *  (Geolog .info model). Hosted as a dock pane (workspace component "module", panel id
 *  "module:<name>"), not a popup — one singleton pane per module, so every module the
 *  backend registers gets its pane with no frontend work. The pane is persistent: the
 *  well list and curve dropdowns refresh on data changes (keeping the user's choices),
 *  so curves produced by one run are immediately selectable in the next. Zone-level
 *  overrides come from the Zones pane; values here are the whole-well defaults. */
export async function buildModuleContent(
  spec: ModuleSpec,
  callbacks: ModulePaneCallbacks,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  let [wells, catalog] = await Promise.all([listWells().then(filterByActiveGroup), listCurveCatalog()]);
  let curveNames = catalog.map((c) => c.name);
  let disposed = false;

  const content = document.createElement("div");
  content.className = "module-pane";

  const doc = document.createElement("p");
  doc.className = "modal-doc";
  doc.textContent = spec.doc;
  content.appendChild(doc);

  // --- Well selection (multi) ---
  const wellBox = document.createElement("div");
  wellBox.className = "well-checklist";
  let wellChecks: { well: WellSummary; input: HTMLInputElement }[] = [];
  const rebuildWellChecklist = (checkedIds: Set<string>) => {
    wellBox.innerHTML = "";
    wellChecks = [];
    for (const well of wells) {
      const label = document.createElement("label");
      label.className = "well-check";
      const input = document.createElement("input");
      input.type = "checkbox";
      input.checked = checkedIds.has(well.well_id);
      label.appendChild(input);
      label.appendChild(document.createTextNode(well.well_name));
      wellBox.appendChild(label);
      wellChecks.push({ well, input });
    }
  };
  // Pre-tick the Wells & Tops multi-selection when one exists, else the active well.
  const runDefaults = defaultRunWellIds(wells);
  const initialWell = appState.selectedWell.get();
  if (runDefaults.size === 0 && initialWell) runDefaults.add(initialWell.well_id);
  rebuildWellChecklist(runDefaults);
  content.appendChild(formRow("Wells", wellBox));

  // --- Args from manifest ---
  const logSelects = new Map<string, HTMLSelectElement>();
  const optSelects = new Map<string, HTMLSelectElement>();
  const paramInputs = new Map<string, HTMLInputElement>();

  const fillSelect = (select: HTMLSelectElement, names: string[], selected: string) => {
    select.innerHTML = "";
    for (const name of names) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      if (name === selected) option.selected = true;
      select.appendChild(option);
    }
  };
  /** Catalog names with `keep` (the current/default choice) prepended when absent, so a
   *  selection never disappears from its own dropdown. */
  const logChoiceNames = (keep: string) => (curveNames.includes(keep) ? curveNames : [keep, ...curveNames]);

  for (const arg of spec.args) {
    if (arg.kind === "log_in") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillSelect(select, logChoiceNames(arg.default), arg.default);
      logSelects.set(arg.name, select);
      content.appendChild(formRow(`${arg.name} ${arg.required ? "" : "(optional)"}`, select, arg.desc));
    } else if (arg.kind === "option") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillSelect(select, arg.choices, arg.default);
      optSelects.set(arg.name, select);
      content.appendChild(formRow(arg.name, select, arg.desc));
    } else if (arg.kind === "param") {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.value = arg.default;
      if (arg.min !== null) input.min = String(arg.min);
      if (arg.max !== null) input.max = String(arg.max);
      paramInputs.set(arg.name, input);
      const unit = arg.unit ? ` [${arg.unit}]` : "";
      content.appendChild(formRow(`${arg.name}${unit}`, input, arg.desc));
    }
  }

  // --- Universal bad-hole mask (optional) ---
  // Applies to every module via the runner: samples where the chosen flag curve == 1 are
  // set missing in all outputs, so flagged intervals never pollute results. Typically the
  // BADHOLE curve produced by the Bad-Hole QC module.
  const maskSelect = document.createElement("select");
  maskSelect.className = "form-control";
  const rebuildMaskOptions = (selected: string) => {
    maskSelect.innerHTML = "";
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    maskSelect.appendChild(none);
    const names = maskCurveNames(curveNames);
    if (selected && !names.includes(selected)) names.unshift(selected);
    for (const name of names) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      if (name === selected) option.selected = true;
      maskSelect.appendChild(option);
    }
  };
  rebuildMaskOptions("");
  content.appendChild(
    formRow("Mask (optional)", maskSelect, "Flag curve (=1 bad) to blank out of every output — e.g. BADHOLE."),
  );

  // --- Input set (read half of "set in/out"): blank = current values ---
  const inSetInput = document.createElement("input");
  inSetInput.className = "form-control";
  inSetInput.type = "text";
  inSetInput.value = "";
  inSetInput.placeholder = "(latest values)";
  inSetInput.setAttribute("list", "log-set-names");
  content.appendChild(
    formRow(
      "Input set",
      inSetInput,
      "Read inputs from this log set's values (latest version per well). Curves the set never wrote fall back to the usual sources. Blank = current values.",
    ),
  );

  // --- Output set (P1-c versioning: re-run = version N+1, never overwrites) ---
  const setInput = document.createElement("input");
  setInput.className = "form-control";
  setInput.type = "text";
  setInput.value = "INTERP";
  setInput.setAttribute("list", "log-set-names");
  let setList = document.querySelector<HTMLDataListElement>("#log-set-names");
  if (!setList) {
    setList = document.createElement("datalist");
    setList.id = "log-set-names";
    document.body.appendChild(setList);
  }
  // The datalist is shared/global (on document.body); refresh its suggestions from the
  // selected well's existing set names (best-effort — fine without a backend). The epoch
  // guard drops a slow listLogSets from a previously selected well so it can't overwrite
  // the current well's suggestions.
  let setSuggestEpoch = 0;
  const refreshSetSuggestions = (well: WellSummary | null) => {
    const epoch = ++setSuggestEpoch;
    const names = new Set(["INTERP", "FINAL", "TEST"]);
    const apply = () => {
      setList!.innerHTML = "";
      for (const n of names) {
        const o = document.createElement("option");
        o.value = n;
        setList!.appendChild(o);
      }
    };
    apply();
    if (well) {
      listLogSets(well.well_id)
        .then((sets) => {
          if (epoch !== setSuggestEpoch) return; // a newer well's refresh already ran
          for (const s of sets) names.add(s.set_name);
          apply();
        })
        .catch(() => {});
    }
  };
  refreshSetSuggestions(initialWell);
  content.appendChild(
    formRow(
      "Output set",
      setInput,
      "Outputs are versioned into this log set — a re-run becomes version N+1, never overwriting. Manage versions in the Curve Catalog.",
    ),
  );

  // --- Outputs note ---
  const outputs = spec.args.filter((a) => a.kind === "log_out").map((a) => a.name);
  const outNote = document.createElement("p");
  outNote.className = "modal-hint";
  outNote.textContent = `Outputs: ${outputs.join(", ")}`;
  content.appendChild(outNote);

  // --- Run ---
  const runBtn = document.createElement("button");
  runBtn.className = "form-run-btn";
  runBtn.textContent = "Run";
  const resultBox = document.createElement("div");
  resultBox.className = "modal-result";
  content.appendChild(runBtn);
  content.appendChild(resultBox);

  // --- Persistent-pane refresh: keep the pickers current without touching user choices.
  // Data changes (imports, module runs — including this pane's own) refresh the well list
  // and curve dropdowns in place; selecting another well only updates the pre-tick when
  // the checklist is empty and the set-name suggestions.
  const refreshData = async () => {
    try {
      const [freshWells, freshCatalog] = await Promise.all([
        listWells().then(filterByActiveGroup),
        listCurveCatalog(),
      ]);
      if (disposed) return;
      const checkedIds = new Set(wellChecks.filter((w) => w.input.checked).map((w) => w.well.well_id));
      wells = freshWells;
      catalog = freshCatalog;
      curveNames = catalog.map((c) => c.name);
      rebuildWellChecklist(checkedIds);
      for (const [name, select] of logSelects) {
        const arg = spec.args.find((a) => a.name === name)!;
        const current = select.value || arg.default;
        fillSelect(select, logChoiceNames(current), current);
      }
      rebuildMaskOptions(maskSelect.value);
    } catch {
      // No backend / transient failure: keep the current form as-is.
    }
  };
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true; // subscribe fires immediately; the initial fetch already ran
      return;
    }
    void refreshData();
  });
  const unsubWell = appState.selectedWell.subscribe((well) => {
    refreshSetSuggestions(well);
    // Only when nothing is ticked yet (non-destructive): re-apply the Wells & Tops
    // multi-selection — the batch pre-tick — so a pane opened or restored before the
    // selection existed still gets all selected wells, not just the active one.
    if (wellChecks.some((w) => w.input.checked)) return;
    const defaults = defaultRunWellIds(wells);
    if (defaults.size === 0 && well) defaults.add(well.well_id);
    if (defaults.size > 0) rebuildWellChecklist(defaults);
  });

  runBtn.addEventListener("click", async () => {
    const wellIds = wellChecks.filter((w) => w.input.checked).map((w) => w.well.well_id);
    if (wellIds.length === 0) {
      resultBox.textContent = "Select at least one well.";
      return;
    }

    // Validate numeric params against manifest ranges.
    const params: Record<string, number> = {};
    for (const [name, input] of paramInputs) {
      const v = parseFloat(input.value);
      const arg = spec.args.find((a) => a.name === name)!;
      if (Number.isNaN(v) || (arg.min !== null && v < arg.min) || (arg.max !== null && v > arg.max)) {
        resultBox.textContent = `${name}: value must be between ${arg.min} and ${arg.max}.`;
        input.focus();
        return;
      }
      params[name] = v;
    }
    const opts: Record<string, string> = {};
    for (const [name, select] of optSelects) opts[name] = select.value;
    if (maskSelect.value) opts.MASK = maskSelect.value;
    const logInputs: Record<string, string> = {};
    for (const [name, select] of logSelects) logInputs[name] = select.value;

    const req: RunModuleRequest = {
      module: spec.name,
      well_ids: wellIds,
      log_inputs: logInputs,
      params,
      opts,
      output_set: setInput.value.trim() || undefined,
      input_set: inSetInput.value.trim() || undefined,
    };
    runBtn.disabled = true;
    resultBox.textContent = `Running ${spec.name} on ${wellIds.length} well(s)…`;
    try {
      const results = await runWorkflowModule(req);
      const ok = results.filter((r) => !r.error);
      resultBox.innerHTML = "";
      for (const r of results) {
        const line = document.createElement("div");
        const well = wells.find((w) => w.well_id === r.well_id);
        line.textContent = r.error
          ? `✗ ${well?.well_name ?? r.well_id}: ${r.error}`
          : `✓ ${well?.well_name ?? r.well_id}: ${r.rows_written} samples → ${r.output_curves.join(", ")}`;
        line.className = r.error ? "result-fail" : "result-ok";
        resultBox.appendChild(line);
      }
      callbacks.setStatus(`${spec.name}: ${ok.length}/${results.length} well(s) computed`);
      if (ok.length > 0) callbacks.onRunComplete(outputs);
    } catch (err) {
      resultBox.textContent = `Run failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return {
    el: content,
    dispose: () => {
      disposed = true;
      unsubData();
      unsubWell();
    },
  };
}
