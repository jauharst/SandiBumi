import {
  listCurveCatalog,
  listLogSets,
  listWells,
  runWorkflowModule,
  type ModuleSpec,
  type RunModuleRequest,
  type WellSummary,
} from "../ipc";
import { filterByActiveGroup } from "../state";
import { formRow, openModal } from "./modal";

export interface ModuleDialogCallbacks {
  /** Called after a successful run so the host can refresh the catalog/layout. */
  onRunComplete: (outputCurves: string[]) => void;
  setStatus: (text: string) => void;
}

/** Opens the auto-generated parameter dialog for one module: input-curve selectors,
 *  option dropdowns, and validated numeric parameters — all straight from the manifest
 *  (Geolog .info model). Zone-level overrides come from the Zones dialog; values here
 *  are the whole-well defaults. */
export async function openModuleDialog(
  spec: ModuleSpec,
  selectedWell: WellSummary | null,
  callbacks: ModuleDialogCallbacks,
): Promise<void> {
  const [wells, catalog] = await Promise.all([listWells().then(filterByActiveGroup), listCurveCatalog()]);
  const curveNames = catalog.map((c) => c.name);

  const content = document.createElement("div");

  const doc = document.createElement("p");
  doc.className = "modal-doc";
  doc.textContent = spec.doc;
  content.appendChild(doc);

  // --- Well selection (multi) ---
  const wellBox = document.createElement("div");
  wellBox.className = "well-checklist";
  const wellChecks: { well: WellSummary; input: HTMLInputElement }[] = [];
  for (const well of wells) {
    const label = document.createElement("label");
    label.className = "well-check";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.checked = selectedWell ? well.well_id === selectedWell.well_id : false;
    label.appendChild(input);
    label.appendChild(document.createTextNode(well.well_name));
    wellBox.appendChild(label);
    wellChecks.push({ well, input });
  }
  content.appendChild(formRow("Wells", wellBox));

  // --- Args from manifest ---
  const logSelects = new Map<string, HTMLSelectElement>();
  const optSelects = new Map<string, HTMLSelectElement>();
  const paramInputs = new Map<string, HTMLInputElement>();

  for (const arg of spec.args) {
    if (arg.kind === "log_in") {
      const select = document.createElement("select");
      select.className = "form-control";
      const names = curveNames.includes(arg.default) ? curveNames : [arg.default, ...curveNames];
      for (const name of names) {
        const option = document.createElement("option");
        option.value = name;
        option.textContent = name;
        if (name === arg.default) option.selected = true;
        select.appendChild(option);
      }
      logSelects.set(arg.name, select);
      content.appendChild(formRow(`${arg.name} ${arg.required ? "" : "(optional)"}`, select, arg.desc));
    } else if (arg.kind === "option") {
      const select = document.createElement("select");
      select.className = "form-control";
      for (const choice of arg.choices) {
        const option = document.createElement("option");
        option.value = choice;
        option.textContent = choice;
        if (choice === arg.default) option.selected = true;
        select.appendChild(option);
      }
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
  {
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    maskSelect.appendChild(none);
    for (const name of curveNames) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      maskSelect.appendChild(option);
    }
  }
  content.appendChild(
    formRow("Mask (optional)", maskSelect, "Flag curve (=1 bad) to blank out of every output — e.g. BADHOLE."),
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
  {
    const names = new Set(["INTERP", "FINAL", "TEST"]);
    if (selectedWell) {
      // Existing set names of the selected well join the suggestions (best-effort).
      listLogSets(selectedWell.well_id)
        .then((sets) => {
          for (const s of sets) names.add(s.set_name);
          setList!.innerHTML = "";
          for (const n of names) {
            const o = document.createElement("option");
            o.value = n;
            setList!.appendChild(o);
          }
        })
        .catch(() => {});
    }
    setList.innerHTML = "";
    for (const n of names) {
      const o = document.createElement("option");
      o.value = n;
      setList.appendChild(o);
    }
  }
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

  const close = openModal(spec.title, content, 560);

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
    };
    runBtn.disabled = true;
    resultBox.textContent = `Running ${spec.name} on ${wellIds.length} well(s)…`;
    try {
      const results = await runWorkflowModule(req);
      const ok = results.filter((r) => !r.error);
      const failed = results.filter((r) => r.error);
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
      if (ok.length > 0) {
        callbacks.onRunComplete(outputs);
        if (failed.length === 0) setTimeout(close, 900);
      }
    } catch (err) {
      resultBox.textContent = `Run failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  });
}
