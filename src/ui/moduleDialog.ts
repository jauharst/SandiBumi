import {
  listCurveCatalog,
  moduleOutputNames,
  runWorkflowModule,
  type ModuleSpec,
  type OutputName,
  type RunModuleRequest,
} from "../ipc";
import { appState } from "../state";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow, openModal } from "./modal";
import { buildParamSources } from "./paramSources";
import { buildRunCustodyControls } from "./runCustody";
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

/** The module manifest is the user-facing custody surface for a method's preconditions. Keep the
 * statement, activation branch and source together so the dialog never shows a bare range whose
 * authority has to be guessed. */
export function argumentHint(arg: ModuleSpec["args"][number]): string {
  const defaultSource = arg.kind !== "param"
    ? ""
    : arg.default_source === "ABSENT"
      ? "Default: ABSENT — no numeric value ships; supply an interpreter value when the selected method requires it."
      : `Default source: ${arg.default_source}.`;
  const conditions = (arg.validity_conditions ?? []).map((condition) => {
    const branch = "when" in condition && condition.when
      ? ` When ${condition.when.argument} = ${condition.when.equals}.`
      : "";
    return `Condition ${condition.id}: ${condition.statement}${branch} Source: ${condition.source}.`;
  });
  return [arg.desc, defaultSource, ...conditions].filter(Boolean).join(" ");
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
  const custodyControls = buildRunCustodyControls();

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
  const textInputs = new Map<string, HTMLInputElement>();
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
      const requirement = arg.required_any_of?.length
        ? `(one of ${[arg.name, ...arg.required_any_of].join(" / ")})`
        : arg.required
          ? ""
          : "(optional)";
      argsGrid.appendChild(formRow(`${arg.name} ${requirement}`, select, argumentHint(arg)));
    } else if (arg.kind === "option") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillSelect(select, arg.choices, arg.default, arg.choice_labels);
      optSelects.set(arg.name, select);
      argsGrid.appendChild(formRow(arg.name, select, argumentHint(arg)));
    } else if (arg.kind === "text") {
      // Free-typed run option (ArgKind::Text) — the Condition family's user-named output curve.
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "text";
      input.value = arg.default;
      // The description carries what a blank means, so it belongs in the field too — a
      // placeholder is where a user looks before reading a hint.
      input.placeholder = arg.desc.includes("blank") ? arg.desc.split("—").pop()!.trim() : "";
      textInputs.set(arg.name, input);
      argsGrid.appendChild(formRow(arg.name, input, argumentHint(arg)));
    } else if (arg.kind === "param") {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.value = arg.default;
      // A manifest with NO default opens EMPTY on purpose (`modules::param_open`) — a despike
      // window and a gap limit have no value that is right in two basins, so the user states one.
      if (!arg.default) input.placeholder = arg.required ? "set a value" : "no bound";
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
      // SB-CORE-013. Where the corpus records competing shipped values for this parameter, they
      // are shown WITH the field rather than in a manual — the point of choice is the only place
      // the disagreement can change a decision. Renders nothing when the topic has no entries, so
      // this is safe on every numeric arg.
      if (arg.sources_topic) {
        const stack = document.createElement("div");
        stack.className = "param-with-sources";
        stack.append(control, buildParamSources(arg.sources_topic));
        control = stack;
      }
      argsGrid.appendChild(formRow(arg.name, control, argumentHint(arg)));
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

  // --- Input / output log set: the ONE shared control (`logSetPicker.ts`). Was two bespoke
  // blocks here labelled "Input cons" / "Output cons" — the store, the backend and the docs all
  // say LOG SET, and only this UI said constellation, which is why the word did not connect.
  const setPicker = buildLogSetPicker({ write: "INTERP" });
  for (const row of setPicker.rows) argsGrid.appendChild(row);
  for (const row of custodyControls.rows) argsGrid.appendChild(row);

  content.appendChild(argsGrid);

  // --- Zone-override callout (design 1d): the precedence rule, stated where
  // the whole-well defaults are being typed rather than in a help page.
  const callout = document.createElement("div");
  callout.className = "module-callout";
  callout.textContent =
    "Values here are the whole-well defaults — per-zone parameters from the Zones pane take precedence inside their zones.";
  content.appendChild(callout);

  // --- Output curves: a row per declared output with the name it will be written under.
  //
  // Jauhar, 2026-08-05: *"naming each output curve in bulk when modules gonna run"*. Every name a
  // run is about to write is on screen at once, editable, before the run — rather than discovered
  // afterwards in the Curve Catalog. The Condition and Frame families used to carry their own
  // "Output curve name" text field for this; that field is gone, because the grid IS it and one
  // control for forty modules cannot drift the way two could.
  //
  // The DEFAULT names come from the backend (`moduleOutputNames`), never from expanding the
  // manifest's patterns here — a `{CURVE}_C` expanded in TypeScript would be a second copy of a
  // naming rule, agreeing with the run right up until one of them changed.
  const outArgs = spec.args.filter((a) => a.kind === "log_out");
  const outSection = document.createElement("div");
  outSection.className = "module-outputs";
  const outHead = document.createElement("div");
  outHead.className = "module-outputs-head";
  const outTitle = document.createElement("span");
  outTitle.className = "module-outputs-title";
  outTitle.textContent = outArgs.length === 1 ? "Output curve" : "Output curves";
  // The bulk fill: one prefix over every name below, for a trial run that must not land on the
  // interpretation the field is already using. Applied by the backend on top of the names in the
  // grid, so the two compose rather than competing.
  const prefixInput = document.createElement("input");
  prefixInput.className = "form-control module-outputs-prefix";
  prefixInput.type = "text";
  prefixInput.placeholder = "prefix all, e.g. TEST_";
  prefixInput.title =
    "Prefixes every curve this run writes, on top of the names below — TEST_ gives TEST_VSH. " +
    "A Monte Carlo study refuses a renamed step, because its cutoffs are resolved from the " +
    "module's declared output names.";
  outHead.append(outTitle, prefixInput);
  outSection.appendChild(outHead);

  const outNameInputs = new Map<string, HTMLInputElement>();
  const outGrid = document.createElement("div");
  outGrid.className = "module-outputs-grid";
  for (const arg of outArgs) {
    const label = document.createElement("label");
    label.className = "module-output-label";
    label.textContent = arg.desc || arg.name;
    if (arg.unit) label.textContent += ` (${arg.unit})`;
    label.title = `Declared as ${arg.name}`;
    const input = document.createElement("input");
    input.className = "form-control module-output-name";
    input.type = "text";
    input.spellcheck = false;
    label.appendChild(input);
    outNameInputs.set(arg.name, input);
    outGrid.appendChild(label);
  }
  outSection.appendChild(outGrid);
  const outWarn = document.createElement("div");
  outWarn.className = "module-outputs-warn";
  outWarn.hidden = true;
  outSection.appendChild(outWarn);
  content.appendChild(outSection);

  /** The run options as the form currently stands — the same map the preview and the run send, so
   *  what the grid shows can never describe a different run from the one the button starts. */
  const collectOpts = (): Record<string, string> => {
    const opts: Record<string, string> = {};
    for (const [name, select] of optSelects) opts[name] = select.value;
    // A blank Text arg is sent as a blank, not dropped: the module's own default is what an empty
    // box means, and omitting the key would make a cleared field indistinguishable from one that
    // was never on the dialog.
    for (const [name, input] of textInputs) opts[name] = input.value.trim();
    for (const [arg, input] of outNameInputs) {
      const typed = input.value.trim();
      if (typed) opts[`__OUT_${arg}`] = typed;
    }
    if (maskSelect.value) opts.MASK = maskSelect.value;
    if (prefixInput.value.trim()) opts.OUT_PREFIX = prefixInput.value.trim();
    return opts;
  };
  const collectLogInputs = (): Record<string, string> => {
    const logInputs: Record<string, string> = {};
    for (const [name, select] of logSelects) logInputs[name] = select.value;
    return logInputs;
  };

  // The placeholder in each name box is the name the run would write if the box is left alone —
  // so an untouched form still SHOWS its answer rather than an empty field. Re-asked whenever an
  // input curve or a rename changes, because a pattern is built from those: renaming the despiked
  // curve to GR_ED moves its flag to GR_ED_SPK, and the grid has to say so.
  let outGen = 0;
  const refreshOutNames = async () => {
    const gen = ++outGen;
    try {
      const names: OutputName[] = await moduleOutputNames(spec.name, collectLogInputs(), collectOpts());
      if (disposed || gen !== outGen) return;
      for (const n of names) outNameInputs.get(n.arg)?.setAttribute("placeholder", n.name);
      outWarn.hidden = true;
    } catch (e) {
      if (disposed || gen !== outGen) return;
      // A refusal (a shadowed name, a collision) is shown HERE, beside the box that caused it,
      // rather than saved up for the run — the user is looking at this grid.
      // The backend's message is the whole message; a leading "Error:" from the Error wrapper
      // just pushes the part that names the curve further from the eye.
      outWarn.textContent = String(e).replace(/^Error:\s*/, "");
      outWarn.hidden = false;
    }
  };
  for (const [, select] of logSelects) select.addEventListener("change", () => void refreshOutNames());
  for (const [, select] of optSelects) select.addEventListener("change", () => void refreshOutNames());
  for (const [, input] of outNameInputs) input.addEventListener("input", () => void refreshOutNames());
  void refreshOutNames();

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
  // The log-set pickers can gain names as the active well changes; the well scope tracks live
  // state on its own, so nothing well-related to re-tick here.
  const unsubWell = appState.selectedWell.subscribe(() => setPicker.refresh());

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      resultBox.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return;
    }

    // Validate numeric params against manifest ranges.
    const params: Record<string, number> = {};
    for (const [name, input] of paramInputs) {
      const arg = spec.args.find((a) => a.name === name)!;
      // A blank field on a no-default param is a real state, not a parse failure. Required →
      // refused BY NAME here, where the user is looking (the needWell.ts rule), rather than as N
      // per-well failures in the Processing panel. Optional → omitted, and the module reads the
      // absence as "no bound on this side" (Clip's MIN/MAX).
      if (!input.value.trim()) {
        if (arg.required) {
          resultBox.textContent = `${name} must be set — ${arg.desc}`;
          input.focus();
          return;
        }
        continue;
      }
      const v = parseFloat(input.value);
      if (Number.isNaN(v) || (arg.min !== null && v < arg.min) || (arg.max !== null && v > arg.max)) {
        resultBox.textContent = `${name}: value must be between ${arg.min} and ${arg.max}.`;
        input.focus();
        return;
      }
      params[name] = v;
    }
    let custody;
    try {
      custody = custodyControls.collect();
    } catch (error) {
      resultBox.textContent = String(error);
      return;
    }
    const req: RunModuleRequest = {
      module: spec.name,
      well_ids: wellIds,
      log_inputs: collectLogInputs(),
      params,
      opts: collectOpts(),
      output_set: setPicker.outputSet(),
      input_set: setPicker.inputSet(),
      custody,
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
      // The names the run actually WROTE, not the manifest's declared ones. Those two used to
      // differ silently for any module that built its own name, so the History panel recorded
      // curves the project did not hold — and now that a name can be renamed they differ by
      // design. The results carry the truth; a well that failed carries none, hence the union.
      const written = [...new Set(results.flatMap((r) => r.output_curves))];
      if (ok > 0) callbacks.onRunComplete(written, scope.namesFor(wellIds));
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
