import {
  listCurveCatalog,
  listLogSetNames,
  runWorkflowModule,
  type ModuleSpec,
  type RunModuleRequest,
} from "../ipc";
import { appState } from "../state";
import { formRow, openModal } from "./modal";
import { buildWellScope } from "./wellScope";

export interface ModulePaneCallbacks {
  /** Called after a successful run so the host can refresh the catalog/layout and record which
   *  wells were actually run (for the History panel) rather than the globally-selected well. */
  onRunComplete: (outputCurves: string[], wellNames: string[]) => void;
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
  let catalog = await listCurveCatalog();
  let curveNames = catalog.map((c) => c.name);
  let disposed = false;
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "module-pane";

  // --- Header (design 1d): icon chip + display-face title + Help on the right.
  // The chip shows the module's initial — the manifest carries no icon, and a
  // hand-kept icon registry would break rule 9 (new modules need no UI work).
  // The Help button surfaces spec.doc in the same modal the ribbon ? uses; the
  // pane already holds the spec, so no workspace wiring is needed.
  const shortName = (spec.title.split("—")[0] ?? "").trim() || spec.name.toUpperCase();
  const head = document.createElement("div");
  head.className = "module-head";
  const chip = document.createElement("span");
  chip.className = "module-chip";
  chip.textContent = (shortName[0] ?? "?").toUpperCase();
  const titleEl = document.createElement("span");
  titleEl.className = "module-title";
  titleEl.textContent = spec.title;
  const helpBtn = document.createElement("button");
  helpBtn.type = "button";
  helpBtn.className = "btn module-help";
  helpBtn.textContent = "? Help";
  helpBtn.addEventListener("click", () => {
    const body = document.createElement("p");
    body.className = "help-body";
    body.textContent = spec.doc || "Documentation for this module is unavailable.";
    openModal(`Help — ${spec.title}`, body, 480);
  });
  head.append(chip, titleEl, helpBtn);
  content.appendChild(head);

  // --- Well selection (scope, not a checklist) ---
  // The scope resolves against live state at run time (group / ★ pinned / selection / all), so a
  // 2000-well field never needs a well-by-well checklist here.
  content.appendChild(scope.el);

  // Labeled fields live in a responsive two-column grid (design 1d); at narrow
  // pane widths it collapses to one column on its own.
  const argsGrid = document.createElement("div");
  argsGrid.className = "module-args";

  // --- Args from manifest ---
  const logSelects = new Map<string, HTMLSelectElement>();
  const optSelects = new Map<string, HTMLSelectElement>();
  const paramInputs = new Map<string, HTMLInputElement>();

  /** `labels` is parallel to `names` and optional — a missing or short entry shows the id, which
   *  is what every un-labelled module still does. The VALUE is always the id, because it is what
   *  `params_json` stores on every saved run (`docs/review_triage.md` finding 21). */
  const fillSelect = (
    select: HTMLSelectElement,
    names: string[],
    selected: string,
    labels?: string[]
  ) => {
    select.innerHTML = "";
    for (const [i, name] of names.entries()) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = labels?.[i] || name;
      if (name === selected) option.selected = true;
      select.appendChild(option);
    }
  };
  /** Fills a log-input <select>: catalog names, plus the current/default choice prepended when
   *  absent (so a selection never disappears from its own dropdown), plus a leading "(none)"
   *  for OPTIONAL inputs so the user can deliberately drop a slot even when a curve of that
   *  name exists — the module doc's advertised "any unwanted curve slot is dropped" behaviour.
   *  An empty value ("") is sent to the backend, which resolves it as an absent (all-NaN) input. */
  const fillLogSelect = (select: HTMLSelectElement, arg: (typeof spec.args)[number], current: string) => {
    select.innerHTML = "";
    if (!arg.required) {
      const none = document.createElement("option");
      none.value = "";
      none.textContent = "(none)";
      select.appendChild(none);
    }
    const names = current === "" || curveNames.includes(current) ? curveNames : [current, ...curveNames];
    for (const name of names) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      select.appendChild(o);
    }
    select.value = current; // "" → (none) for optional; else the chosen/default mnemonic
  };

  for (const arg of spec.args) {
    if (arg.kind === "log_in") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillLogSelect(select, arg, arg.default);
      logSelects.set(arg.name, select);
      argsGrid.appendChild(formRow(`${arg.name} ${arg.required ? "" : "(optional)"}`, select, arg.desc));
    } else if (arg.kind === "option") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillSelect(select, arg.choices, arg.default, arg.choice_labels);
      optSelects.set(arg.name, select);
      argsGrid.appendChild(formRow(arg.name, select, arg.desc));
    } else if (arg.kind === "param") {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.value = arg.default;
      if (arg.min !== null) input.min = String(arg.min);
      if (arg.max !== null) input.max = String(arg.max);
      paramInputs.set(arg.name, input);
      // The unit sits to the RIGHT of the input (design 1d), no longer inside
      // the label — the value and its unit read as one fact.
      let control: HTMLElement = input;
      if (arg.unit) {
        const wrap = document.createElement("div");
        wrap.className = "input-unit";
        const u = document.createElement("span");
        u.className = "unit-suffix";
        u.textContent = arg.unit;
        wrap.append(input, u);
        control = wrap;
      }
      argsGrid.appendChild(formRow(arg.name, control, arg.desc));
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
  argsGrid.appendChild(
    formRow("Mask (optional)", maskSelect, "Flag curve (=1 bad) to blank out of every output — e.g. BADHOLE."),
  );

  // --- Input cons (read half of "cons in/out"): strict dropdown, blank = current values ---
  const inSetSelect = document.createElement("select");
  inSetSelect.className = "form-control";
  const inSetLatest = document.createElement("option");
  inSetLatest.value = "";
  inSetLatest.textContent = "(latest values)";
  inSetSelect.appendChild(inSetLatest);
  argsGrid.appendChild(
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
  argsGrid.appendChild(
    formRow(
      "Output cons",
      setInput,
      "Outputs are versioned into this constellation — a re-run becomes version N+1, never overwriting. Pick an existing one or type a new name. Manage versions in the Curve Catalog.",
    ),
  );
  content.appendChild(argsGrid);

  // --- Zone-override callout (design 1d): the precedence rule, stated where
  // the whole-well defaults are being typed rather than in a help page.
  const callout = document.createElement("div");
  callout.className = "module-callout";
  callout.textContent =
    "Values here are the whole-well defaults — per-zone parameters from the Zones pane take precedence inside their zones.";
  content.appendChild(callout);

  // --- Outputs note ---
  const outputs = spec.args.filter((a) => a.kind === "log_out").map((a) => a.name);
  const outNote = document.createElement("p");
  outNote.className = "modal-hint";
  outNote.textContent = `Outputs: ${outputs.join(", ")}`;
  content.appendChild(outNote);

  // --- Footer (design 1d): primary pill naming the module, last-run status
  // right-aligned beside it.
  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = `Run ${shortName}`;
  const resultBox = document.createElement("div");
  resultBox.className = "modal-result module-status";
  const footer = document.createElement("div");
  footer.className = "module-footer";
  footer.append(runBtn, resultBox);
  content.appendChild(footer);

  // --- Persistent-pane refresh: keep the pickers current without touching user choices.
  // Data changes (imports, module runs — including this pane's own) refresh the well list
  // and curve dropdowns in place; selecting another well only updates the pre-tick when
  // the checklist is empty and the set-name suggestions.
  let refreshGen = 0;
  const refreshData = async () => {
    // Race guard: a slower refresh resolving after a fresher one must not overwrite it with a
    // stale catalog (dataVersion can bump several times in quick succession).
    const gen = ++refreshGen;
    try {
      const freshCatalog = await listCurveCatalog();
      if (disposed || gen !== refreshGen) return;
      catalog = freshCatalog;
      curveNames = catalog.map((c) => c.name);
      for (const [name, select] of logSelects) {
        const arg = spec.args.find((a) => a.name === name)!;
        // Preserve the current selection, including a deliberate "(none)" ("") on an optional arg.
        fillLogSelect(select, arg, select.value);
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
  // The cons pickers can gain names as the active well changes; the well scope tracks live state
  // on its own, so nothing well-related to re-tick here.
  const unsubWell = appState.selectedWell.subscribe(() => refreshConsPickers());

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      resultBox.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
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
    resultBox.className = "modal-result module-status";
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
      if (ok > 0) callbacks.onRunComplete(outputs, scope.namesFor(wellIds));
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
      scope.dispose();
    },
  };
}
