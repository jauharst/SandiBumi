import {
  cancelWorkflowChain,
  deleteDocument,
  getChainStatus,
  listCurveCatalog,
  listDocuments,
  listModules,
  listWells,
  runWorkflowChain,
  saveDocument,
  type ChainStatus,
  type ChainStep,
  type CurveCatalogEntry,
  type ModuleSpec,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, filterByActiveGroup } from "../state";
import { formRow } from "./modal";

const WORKFLOW_DOC_TYPE = "workflow";

interface WorkflowDoc {
  steps: ChainStep[];
}

function emptyStep(module: string): ChainStep {
  return { module, log_inputs: {}, params: {}, opts: {} };
}

/** Workflow Builder (Phase 9): compose an ordered list of modules and run the whole chain
 *  across many wells in one click, with live progress. Steps use each module's default
 *  parameters, which zone_params (from the Zones panel) then override per zone — so a saved
 *  chain reproduces a full interpretation. Chains persist as `workflow` documents.
 *
 *  Hosted as a rigid dock pane (workspace component "workflow"), not a popup — a popup
 *  was too easy to dismiss mid-build and its inputs too easy to nudge (Jauhar 2026-07-19).
 *  Numeric params follow the app-wide double-click-to-edit rule via interactionGuard. */
export async function buildWorkflowContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const modules = await listModules().catch(() => [] as ModuleSpec[]);
  const wells = filterByActiveGroup(await listWells().catch(() => [] as WellSummary[]));
  const catalog = await listCurveCatalog().catch(() => [] as CurveCatalogEntry[]);
  const curveNames = catalog.map((c) => c.name);
  const moduleByName = new Map(modules.map((m) => [m.name, m]));

  let steps: ChainStep[] = [];
  // Steps whose parameter editor is expanded — tracked by object reference so it survives
  // reordering (move swaps references, not indices).
  const expanded = new Set<ChainStep>();
  let polling: number | null = null;
  let running = false;

  const content = document.createElement("div");
  content.className = "workflow-dialog";

  // --- Saved workflows -----------------------------------------------------
  const savedSelect = document.createElement("select");
  const nameInput = document.createElement("input");
  nameInput.type = "text";
  nameInput.placeholder = "workflow name";
  const loadBtn = button("Load");
  const saveBtn = button("Save");
  const delBtn = button("Delete");

  async function refreshSaved(): Promise<void> {
    const docs = await listDocuments(WORKFLOW_DOC_TYPE).catch(() => []);
    savedSelect.innerHTML = "";
    const ph = document.createElement("option");
    ph.value = "";
    ph.textContent = docs.length ? "— saved workflows —" : "(none saved)";
    savedSelect.appendChild(ph);
    for (const d of docs) {
      const o = document.createElement("option");
      o.value = d.name;
      o.textContent = d.name;
      savedSelect.appendChild(o);
    }
  }

  loadBtn.addEventListener("click", async () => {
    const name = savedSelect.value;
    if (!name) return;
    const docs = await listDocuments(WORKFLOW_DOC_TYPE).catch(() => []);
    const doc = docs.find((d) => d.name === name);
    if (!doc) return;
    try {
      const parsed = JSON.parse(doc.json) as WorkflowDoc;
      steps = (parsed.steps ?? []).map((s) => ({
        module: s.module,
        log_inputs: s.log_inputs ?? {},
        params: s.params ?? {},
        opts: s.opts ?? {},
      }));
      expanded.clear();
      nameInput.value = name;
      renderSteps();
      setStatus(`Loaded workflow "${name}" (${steps.length} steps)`);
    } catch (e) {
      setStatus(`Failed to parse workflow: ${e}`);
    }
  });

  saveBtn.addEventListener("click", async () => {
    const name = nameInput.value.trim();
    if (!name) {
      setStatus("Enter a workflow name first");
      return;
    }
    if (steps.length === 0) {
      setStatus("Add at least one step before saving");
      return;
    }
    const doc: WorkflowDoc = { steps };
    await saveDocument(WORKFLOW_DOC_TYPE, name, JSON.stringify(doc));
    await refreshSaved();
    savedSelect.value = name;
    setStatus(`Saved workflow "${name}"`);
  });

  delBtn.addEventListener("click", async () => {
    const name = savedSelect.value;
    if (!name) return;
    await deleteDocument(WORKFLOW_DOC_TYPE, name);
    await refreshSaved();
    setStatus(`Deleted workflow "${name}"`);
  });

  const savedRow = document.createElement("div");
  savedRow.className = "workflow-saved-row";
  savedRow.append(savedSelect, loadBtn, delBtn);
  content.appendChild(formRow("Saved", savedRow));

  // --- Step builder --------------------------------------------------------
  const moduleSelect = document.createElement("select");
  const byCategory = new Map<string, ModuleSpec[]>();
  for (const m of modules) {
    if (!byCategory.has(m.category)) byCategory.set(m.category, []);
    byCategory.get(m.category)!.push(m);
  }
  for (const [cat, mods] of byCategory) {
    const group = document.createElement("optgroup");
    group.label = cat;
    for (const m of mods) {
      const o = document.createElement("option");
      o.value = m.name;
      o.textContent = m.title;
      group.appendChild(o);
    }
    moduleSelect.appendChild(group);
  }
  const addBtn = button("+ Add step");
  addBtn.addEventListener("click", () => {
    if (!moduleSelect.value) return;
    steps.push(emptyStep(moduleSelect.value));
    renderSteps();
  });
  const addRow = document.createElement("div");
  addRow.className = "workflow-add-row";
  addRow.append(moduleSelect, addBtn);
  content.appendChild(formRow("Add module", addRow, "Steps run top-to-bottom; later steps use earlier outputs (VSH → porosity → saturation)."));

  const stepsList = document.createElement("ol");
  stepsList.className = "workflow-steps";
  content.appendChild(stepsList);

  function renderSteps(): void {
    stepsList.innerHTML = "";
    if (steps.length === 0) {
      const empty = document.createElement("li");
      empty.className = "workflow-empty";
      empty.textContent = "No steps yet — add modules above.";
      stepsList.appendChild(empty);
      return;
    }
    steps.forEach((step, i) => {
      const li = document.createElement("li");
      li.className = "workflow-step";
      const head = document.createElement("div");
      head.className = "workflow-step-head";
      const title = document.createElement("span");
      title.className = "workflow-step-title";
      title.textContent = moduleByName.get(step.module)?.title ?? step.module;
      const badge = document.createElement("span");
      badge.className = "workflow-step-badge";
      updateBadge(step, badge);

      const spec = moduleByName.get(step.module);
      const gear = miniButton(expanded.has(step) ? "⚙▾" : "⚙", () => {
        if (expanded.has(step)) expanded.delete(step);
        else expanded.add(step);
        renderSteps();
      });
      gear.title = "Edit parameters for this step";
      gear.disabled = !spec;
      const up = miniButton("↑", () => move(i, -1));
      const down = miniButton("↓", () => move(i, +1));
      const rm = miniButton("✕", () => {
        expanded.delete(step);
        steps.splice(i, 1);
        renderSteps();
      });
      up.disabled = i === 0;
      down.disabled = i === steps.length - 1;
      const ctrls = document.createElement("span");
      ctrls.className = "workflow-step-ctrls";
      ctrls.append(gear, up, down, rm);
      head.append(title, badge, ctrls);
      li.appendChild(head);

      if (spec && expanded.has(step)) {
        li.appendChild(buildStepEditor(step, spec, badge));
      }
      stepsList.appendChild(li);
    });
  }

  /** Count of overrides set on a step (params + non-default log inputs + options/mask). */
  function overrideCount(step: ChainStep): number {
    return (
      Object.keys(step.params).length +
      Object.keys(step.log_inputs).length +
      Object.keys(step.opts).length
    );
  }

  function updateBadge(step: ChainStep, badge: HTMLElement): void {
    const n = overrideCount(step);
    badge.textContent = n ? `· ${n} override${n === 1 ? "" : "s"}` : "· defaults";
    badge.classList.toggle("workflow-step-badge-set", n > 0);
  }

  /** Inline per-step editor: input curves, options, and numeric params straight from the
   *  module manifest. Only values that differ from the manifest default are stored on the
   *  step, so an untouched step keeps empty maps (pure manifest + zone_params behaviour).
   *  Zone parameters still override these whole-well values per zone at run time. */
  function buildStepEditor(step: ChainStep, spec: ModuleSpec, badge: HTMLElement): HTMLElement {
    const box = document.createElement("div");
    box.className = "workflow-step-editor";

    for (const arg of spec.args) {
      if (arg.kind === "log_in") {
        const select = document.createElement("select");
        const names = curveNames.includes(arg.default) ? curveNames : [arg.default, ...curveNames];
        for (const name of names) {
          const o = document.createElement("option");
          o.value = name;
          o.textContent = name;
          select.appendChild(o);
        }
        select.value = step.log_inputs[arg.name] ?? arg.default;
        select.addEventListener("change", () => {
          if (select.value === arg.default) delete step.log_inputs[arg.name];
          else step.log_inputs[arg.name] = select.value;
          updateBadge(step, badge);
        });
        box.appendChild(editorRow(`${arg.name}${arg.required ? "" : " (opt)"}`, select, arg.desc));
      } else if (arg.kind === "option") {
        const select = document.createElement("select");
        for (const choice of arg.choices) {
          const o = document.createElement("option");
          o.value = choice;
          o.textContent = choice;
          select.appendChild(o);
        }
        select.value = step.opts[arg.name] ?? arg.default;
        select.addEventListener("change", () => {
          if (select.value === arg.default) delete step.opts[arg.name];
          else step.opts[arg.name] = select.value;
          updateBadge(step, badge);
        });
        box.appendChild(editorRow(arg.name, select, arg.desc));
      } else if (arg.kind === "param") {
        const input = document.createElement("input");
        input.type = "number";
        input.step = "any";
        input.value = step.params[arg.name] !== undefined ? String(step.params[arg.name]) : arg.default;
        if (arg.min !== null) input.min = String(arg.min);
        if (arg.max !== null) input.max = String(arg.max);
        const defaultNum = parseFloat(arg.default);
        input.addEventListener("change", () => {
          const v = parseFloat(input.value);
          if (Number.isNaN(v) || (arg.min !== null && v < arg.min) || (arg.max !== null && v > arg.max)) {
            input.classList.add("workflow-invalid");
            return;
          }
          input.classList.remove("workflow-invalid");
          if (v === defaultNum) delete step.params[arg.name];
          else step.params[arg.name] = v;
          updateBadge(step, badge);
        });
        const unit = arg.unit ? ` [${arg.unit}]` : "";
        box.appendChild(editorRow(`${arg.name}${unit}`, input, arg.desc));
      }
    }

    // Universal bad-hole mask (opts.MASK) — same capability the module dialog exposes.
    const maskSelect = document.createElement("select");
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    maskSelect.appendChild(none);
    for (const name of curveNames) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      maskSelect.appendChild(o);
    }
    maskSelect.value = step.opts.MASK ?? "";
    maskSelect.addEventListener("change", () => {
      if (maskSelect.value) step.opts.MASK = maskSelect.value;
      else delete step.opts.MASK;
      updateBadge(step, badge);
    });
    box.appendChild(editorRow("Mask (opt)", maskSelect, "Flag curve (=1 bad) blanked from every output — e.g. BADHOLE."));

    const outputs = spec.args.filter((a) => a.kind === "log_out").map((a) => a.name);
    const footer = document.createElement("div");
    footer.className = "workflow-editor-footer";
    const outNote = document.createElement("span");
    outNote.className = "workflow-editor-outputs";
    outNote.textContent = outputs.length ? `Outputs: ${outputs.join(", ")}` : "";
    const reset = miniButton("Reset", () => {
      step.params = {};
      step.log_inputs = {};
      step.opts = {};
      renderSteps();
    });
    reset.title = "Clear all overrides (back to manifest defaults)";
    footer.append(outNote, reset);
    box.appendChild(footer);
    return box;
  }

  function move(i: number, delta: number): void {
    const j = i + delta;
    if (j < 0 || j >= steps.length) return;
    [steps[i], steps[j]] = [steps[j], steps[i]];
    renderSteps();
  }

  // --- Wells ---------------------------------------------------------------
  const wellsBox = document.createElement("div");
  wellsBox.className = "workflow-wells";
  const wellChecks = new Map<string, HTMLInputElement>();
  const selected = appState.selectedWell.get();
  for (const w of wells) {
    const label = document.createElement("label");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.value = w.well_id;
    cb.checked = selected ? w.well_id === selected.well_id : true;
    wellChecks.set(w.well_id, cb);
    label.append(cb, document.createTextNode(` ${w.well_name}`));
    wellsBox.appendChild(label);
  }
  const allBtn = miniButton("All", () => wellChecks.forEach((cb) => (cb.checked = true)));
  const noneBtn = miniButton("None", () => wellChecks.forEach((cb) => (cb.checked = false)));
  const wellsHead = document.createElement("div");
  wellsHead.className = "workflow-wells-head";
  wellsHead.append(allBtn, noneBtn);
  content.appendChild(formRow("Wells", wellsHead));
  content.appendChild(wellsBox);

  // --- Run bar -------------------------------------------------------------
  const progress = document.createElement("progress");
  progress.max = 1;
  progress.value = 0;
  progress.style.display = "none";
  const statusLine = document.createElement("div");
  statusLine.className = "workflow-status";
  const runBtn = button("Run chain");
  runBtn.classList.add("primary");
  const cancelBtn = button("Cancel");
  cancelBtn.disabled = true;

  const runRow = document.createElement("div");
  runRow.className = "workflow-run-row";
  runRow.append(runBtn, cancelBtn);

  let currentJob: string | null = null;

  function stopPolling(): void {
    if (polling !== null) {
      clearInterval(polling);
      polling = null;
    }
  }

  function applyStatus(s: ChainStatus | null): void {
    if (!s) return;
    if (s.state === "running") {
      progress.style.display = "";
      progress.max = s.total_steps;
      progress.value = s.step + (s.wells_done >= s.wells_total ? 1 : 0);
      statusLine.textContent = `Step ${s.step + 1}/${s.total_steps}: ${moduleByName.get(s.module)?.title ?? s.module}`;
    } else if (s.state === "completed") {
      progress.value = progress.max;
      const errNote = s.errors.length ? ` — ${s.errors.length} well/step warnings` : "";
      statusLine.textContent = `Done: ${s.steps_run} steps, ${s.curves_written} curves across ${s.wells} wells${errNote}`;
      if (s.errors.length) console.warn("chain warnings:", s.errors);
      finishRun();
      bumpDataVersion();
      setStatus(`Workflow finished (${s.steps_run} steps, ${s.wells} wells)`);
    } else if (s.state === "cancelled") {
      statusLine.textContent = `Cancelled at step ${s.at_step + 1}`;
      finishRun();
      setStatus("Workflow cancelled");
    } else if (s.state === "failed") {
      statusLine.textContent = `Failed: ${s.error}`;
      finishRun();
      setStatus(`Workflow failed: ${s.error}`);
    }
  }

  function finishRun(): void {
    running = false;
    stopPolling();
    runBtn.disabled = false;
    cancelBtn.disabled = true;
    currentJob = null;
  }

  runBtn.addEventListener("click", async () => {
    if (running) return;
    if (steps.length === 0) {
      setStatus("Add at least one step");
      return;
    }
    const wellIds = [...wellChecks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id);
    if (wellIds.length === 0) {
      setStatus("Select at least one well");
      return;
    }
    const jobId = crypto.randomUUID();
    currentJob = jobId;
    running = true;
    runBtn.disabled = true;
    cancelBtn.disabled = false;
    progress.style.display = "";
    progress.max = steps.length;
    progress.value = 0;
    statusLine.textContent = "Starting…";

    // Fire the (blocking) run without awaiting so we can poll progress meanwhile.
    void runWorkflowChain(jobId, steps, wellIds).catch((e) => {
      statusLine.textContent = `Error: ${e}`;
      finishRun();
    });
    polling = window.setInterval(async () => {
      if (!currentJob) return;
      // If the dialog was closed mid-run, cancel the job and stop polling.
      if (!content.isConnected) {
        void cancelWorkflowChain(currentJob).catch(() => {});
        finishRun();
        return;
      }
      const s = await getChainStatus(currentJob).catch(() => null);
      applyStatus(s);
    }, 250);
  });

  cancelBtn.addEventListener("click", async () => {
    if (currentJob) await cancelWorkflowChain(currentJob).catch(() => {});
  });

  content.append(runRow, progress, statusLine);

  // Save controls live at the bottom with the name field.
  const saveRow = document.createElement("div");
  saveRow.className = "workflow-save-row";
  saveRow.append(nameInput, saveBtn);
  content.appendChild(formRow("Save as", saveRow));

  await refreshSaved();
  renderSteps();

  return {
    el: content,
    dispose: () => {
      // Pane closed mid-run: cancel the chain and stop polling.
      if (currentJob) void cancelWorkflowChain(currentJob).catch(() => {});
      finishRun();
    },
  };
}

function button(text: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.textContent = text;
  return b;
}

function miniButton(text: string, onClick: () => void): HTMLButtonElement {
  const b = button(text);
  b.className = "workflow-mini";
  b.addEventListener("click", onClick);
  return b;
}

/** Compact label + control + hint row used inside the per-step parameter editor. */
function editorRow(label: string, control: HTMLElement, hint?: string): HTMLElement {
  const row = document.createElement("div");
  row.className = "workflow-editor-row";
  const lab = document.createElement("label");
  lab.textContent = label;
  if (hint) lab.title = hint;
  row.append(lab, control);
  return row;
}
