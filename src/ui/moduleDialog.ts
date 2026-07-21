import {
  listCurveCatalog,
  listLogSetNames,
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
 *  (module-manifest model). Hosted as a dock pane (workspace component "module", panel id
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

  // The method narration/formula used to sit here as a paragraph. It moved to the on-demand
  // Help tool (the ? in the quick-access bar → "Help — <this module>"), which reads spec.doc,
  // so the form stays uncluttered and the same text is the seed for the future HTML help library.

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

  // --- Input cons (read half of "cons in/out"): strict dropdown, blank = current values ---
  const inSetSelect = document.createElement("select");
  inSetSelect.className = "form-control";
  const inSetLatest = document.createElement("option");
  inSetLatest.value = "";
  inSetLatest.textContent = "(latest values)";
  inSetSelect.appendChild(inSetLatest);
  content.appendChild(
    formRow(
      "Input cons",
      inSetSelect,
      "Read inputs from this constellation's values (latest version per well). Curves it never wrote fall back to the usual sources. Blank = current values.",
    ),
  );

  // --- Output cons (P1-c versioning: re-run = version N+1, never overwrites). Editable
  // combobox — pick an existing constellation or type a brand-new name. ---
  const setInput = document.createElement("input");
  setInput.className = "form-control";
  setInput.type = "text";
  setInput.value = "INTERP";
  setInput.setAttribute("list", "log-cons-names");
  let consList = document.querySelector<HTMLDataListElement>("#log-cons-names");
  if (!consList) {
    consList = document.createElement("datalist");
    consList.id = "log-cons-names";
    document.body.appendChild(consList);
  }
  // Fill both pickers from the project's existing constellation names. Input is a strict
  // dropdown (you can only read from one that exists); output offers them as suggestions
  // plus the common defaults. The input select keeps the user's current choice across
  // refreshes (a new run, or a well switch, can add names).
  const refreshConsPickers = () => {
    void listLogSetNames()
      .then((names) => {
        const keep = inSetSelect.value;
        while (inSetSelect.options.length > 1) inSetSelect.remove(1);
        for (const n of names) {
          const o = document.createElement("option");
          o.value = n;
          o.textContent = n;
          inSetSelect.appendChild(o);
        }
        if ([...inSetSelect.options].some((o) => o.value === keep)) inSetSelect.value = keep;
        const seeds = [...new Set(["INTERP", "FINAL", "TEST", ...names])];
        consList!.innerHTML = "";
        for (const n of seeds) {
          const o = document.createElement("option");
          o.value = n;
          consList!.appendChild(o);
        }
      })
      .catch(() => {});
  };
  refreshConsPickers();
  content.appendChild(
    formRow(
      "Output cons",
      setInput,
      "Outputs are versioned into this constellation — a re-run becomes version N+1, never overwriting. Pick an existing one or type a new name. Manage versions in the Curve Catalog.",
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
    refreshConsPickers();
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
      input_set: inSetSelect.value.trim() || undefined,
    };
    runBtn.disabled = true;
    // Live progress and the per-well ✓/⚠/✗ breakdown now live in the Processing panel (this
    // run reports into the shared job registry). Surface that panel and keep only a one-line
    // outcome here, so the form isn't a second, redundant results log.
    resultBox.className = "modal-result";
    resultBox.textContent = `Running ${spec.name} on ${wellIds.length} well(s)… see the Processing panel for progress.`;
    window.dispatchEvent(new Event("sandibumi:open-processing"));
    try {
      const results = await runWorkflowModule(req);
      const ok = results.filter((r) => !r.error).length;
      const failed = results.length - ok;
      resultBox.textContent = failed
        ? `${ok}/${results.length} well(s) computed — ${failed} need attention. Open Processing → details for the report.`
        : `All ${ok} well(s) computed. Per-well details are in the Processing panel.`;
      callbacks.setStatus(`${spec.name}: ${ok}/${results.length} well(s) computed`);
      if (ok > 0) callbacks.onRunComplete(outputs);
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
