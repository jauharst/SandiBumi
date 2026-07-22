import {
  cancelWorkflowChain,
  deleteDocument,
  getChainStatus,
  listCurveCatalog,
  listDocuments,
  listLogSetNames,
  listModules,
  runWorkflowChain,
  saveDocument,
  type ArgSpec,
  type ChainStatus,
  type ChainStep,
  type CurveCatalogEntry,
  type ModuleSpec,
} from "../ipc";
import { bumpDataVersion } from "../state";
import { formRow } from "./modal";
import { maskCurveNames } from "./moduleDialog";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

const WORKFLOW_DOC_TYPE = "workflow";
const VIEW_KEY = "sandibumi.workflowView";

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
 *  Numeric params follow the app-wide double-click-to-edit rule via interactionGuard.
 *
 *  Steps can be edited two ways (List/Grid toggle): per-step accordion editors, or the
 *  multi-line grid inspector — rows = steps, columns = the union of every step's args,
 *  so a shared parameter (RW across the sw_* steps) lines up in one column and the
 *  Set-all row writes it to every step that takes it in a single edit. */
export async function buildWorkflowContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const modules = await listModules().catch(() => [] as ModuleSpec[]);
  const scope = await buildWellScope();
  const catalog = await listCurveCatalog().catch(() => [] as CurveCatalogEntry[]);
  const curveNames = catalog.map((c) => c.name);
  const moduleByName = new Map(modules.map((m) => [m.name, m]));
  // Every module's declared outputs stay selectable as INPUTS even before any run has
  // written them: chain steps execute in order, so "nphimat then phi_dn(NPHI = NPHI_SS)"
  // must be expressible in a fresh project whose catalog has no computed curves yet —
  // the same composability rule MASK_CURVE_SUGGESTIONS applies to the Mask dropdowns.
  const moduleOutputs = [
    ...new Set(modules.flatMap((m) => m.args.filter((a) => a.kind === "log_out").map((a) => a.name))),
  ].filter((n) => !curveNames.includes(n));
  const inputCurveNames = [...curveNames, ...moduleOutputs];

  let steps: ChainStep[] = [];
  // Steps whose parameter editor is expanded — tracked by object reference so it survives
  // reordering (move swaps references, not indices).
  const expanded = new Set<ChainStep>();
  // "list" = one accordion editor per step; "grid" = the multi-line inspector.
  let view: "list" | "grid" = localStorage.getItem(VIEW_KEY) === "grid" ? "grid" : "list";
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
  // The legacy 4-component `multimin` module is superseded by SandiMin (its own Advance-tab
  // pane) — keep it out of the chain step picker so new chains don't wire up the deprecated
  // solver. Saved chains that already reference it still resolve via moduleByName.
  const DEPRECATED_STEP_MODULES = new Set(["multimin"]);
  const byCategory = new Map<string, ModuleSpec[]>();
  for (const m of modules) {
    if (DEPRECATED_STEP_MODULES.has(m.name)) continue;
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

  const listBtn = miniButton("List", () => setView("list"));
  listBtn.title = "One collapsible parameter editor per step";
  const gridBtn = miniButton("Grid", () => setView("grid"));
  gridBtn.title = "All steps' parameters in one editable grid — the Set-all row edits a shared parameter across every step";
  const viewToggle = document.createElement("div");
  viewToggle.className = "workflow-view-toggle";
  viewToggle.append(listBtn, gridBtn);
  content.appendChild(viewToggle);

  const stepsList = document.createElement("ol");
  stepsList.className = "workflow-steps";
  const gridWrap = document.createElement("div");
  gridWrap.className = "workflow-grid-wrap";
  content.append(stepsList, gridWrap);

  function setView(v: "list" | "grid"): void {
    view = v;
    localStorage.setItem(VIEW_KEY, v);
    renderSteps();
  }

  function renderSteps(): void {
    listBtn.classList.toggle("workflow-mini-active", view === "list");
    gridBtn.classList.toggle("workflow-mini-active", view === "grid");
    // The grid needs the whole pane width; the form column keeps its 640px cap.
    content.classList.toggle("workflow-grid-active", view === "grid");
    stepsList.style.display = view === "list" ? "" : "none";
    gridWrap.style.display = view === "grid" ? "" : "none";
    if (view === "list") renderListView();
    else renderGridView();
  }

  function renderListView(): void {
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

  // --- Shared per-arg controls -------------------------------------------------------
  // Only values that differ from the manifest default are stored on the step, so an
  // untouched control keeps the step's maps empty (pure manifest + zone_params
  // behaviour). Zone parameters still override these whole-well values per zone at run
  // time. The same builders back both the accordion and the grid so semantics match.

  function logInControl(step: ChainStep, arg: ArgSpec, onChanged: () => void): HTMLSelectElement {
    const select = document.createElement("select");
    const names = inputCurveNames.includes(arg.default) ? inputCurveNames : [arg.default, ...inputCurveNames];
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
      onChanged();
    });
    return select;
  }

  function optionControl(step: ChainStep, arg: ArgSpec, onChanged: () => void): HTMLSelectElement {
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
      onChanged();
    });
    return select;
  }

  function paramControl(step: ChainStep, arg: ArgSpec, onChanged: () => void): HTMLInputElement {
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
        const lim = [arg.min !== null ? `min ${arg.min}` : "", arg.max !== null ? `max ${arg.max}` : ""]
          .filter(Boolean)
          .join(", ");
        setStatus(`${arg.name}: "${input.value}" not applied${lim ? ` (${lim})` : ""} — the stored value still runs`);
        return;
      }
      input.classList.remove("workflow-invalid");
      if (v === defaultNum) delete step.params[arg.name];
      else step.params[arg.name] = v;
      onChanged();
    });
    return input;
  }

  /** Universal bad-hole mask (opts.MASK) — same capability the module dialog exposes. */
  function maskControl(step: ChainStep, onChanged: () => void): HTMLSelectElement {
    const select = document.createElement("select");
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    select.appendChild(none);
    for (const name of maskCurveNames(curveNames)) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      select.appendChild(o);
    }
    select.value = step.opts.MASK ?? "";
    select.addEventListener("change", () => {
      if (select.value) step.opts.MASK = select.value;
      else delete step.opts.MASK;
      onChanged();
    });
    return select;
  }

  const MASK_DESC = "Flag curve (=1 bad) blanked from every output — e.g. BADHOLE.";

  /** Inline per-step editor: input curves, options, and numeric params straight from the
   *  module manifest. */
  function buildStepEditor(step: ChainStep, spec: ModuleSpec, badge: HTMLElement): HTMLElement {
    const box = document.createElement("div");
    box.className = "workflow-step-editor";
    const onChanged = (): void => updateBadge(step, badge);

    for (const arg of spec.args) {
      if (arg.kind === "log_in") {
        box.appendChild(editorRow(`${arg.name}${arg.required ? "" : " (opt)"}`, logInControl(step, arg, onChanged), arg.desc));
      } else if (arg.kind === "option") {
        box.appendChild(editorRow(arg.name, optionControl(step, arg, onChanged), arg.desc));
      } else if (arg.kind === "param") {
        const unit = arg.unit ? ` [${arg.unit}]` : "";
        box.appendChild(editorRow(`${arg.name}${unit}`, paramControl(step, arg, onChanged), arg.desc));
      }
    }
    box.appendChild(editorRow("Mask (opt)", maskControl(step, onChanged), MASK_DESC));

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

  // --- Grid view (multi-line inspector) ------------------------------------
  // Rows = steps, columns = the union of every step's manifest args (curves first, then
  // numeric params, then options, MASK last). A parameter shared by several modules
  // lines up in one column; a step that doesn't take a column's arg shows "—".

  type GridKind = "log_in" | "param" | "option" | "mask";
  interface GridCol {
    kind: GridKind;
    name: string;
    unit: string;
    desc: string;
    /** Per-step manifest arg — defaults and limits can differ per module. */
    args: Map<ChainStep, ArgSpec>;
  }

  function gridColumns(): GridCol[] {
    // Column order follows the module MANIFEST order (not chain order), so moving a
    // step up/down never shuffles the columns; kinds still group curves → params →
    // options, with the universal Mask last.
    const stepsByModule = new Map<string, ChainStep[]>();
    for (const step of steps) {
      if (!stepsByModule.has(step.module)) stepsByModule.set(step.module, []);
      stepsByModule.get(step.module)!.push(step);
    }
    const by = new Map<string, GridCol>();
    for (const spec of modules) {
      const own = stepsByModule.get(spec.name);
      if (!own) continue;
      for (const arg of spec.args) {
        if (arg.kind === "log_out") continue;
        const key = `${arg.kind}:${arg.name}`;
        let col = by.get(key);
        if (!col) {
          col = { kind: arg.kind, name: arg.name, unit: arg.unit, desc: arg.desc, args: new Map() };
          by.set(key, col);
        }
        for (const step of own) {
          if (!col.args.has(step)) col.args.set(step, arg);
        }
      }
    }
    const rank: GridKind[] = ["log_in", "param", "option"];
    const cols = [...by.values()].sort((a, b) => rank.indexOf(a.kind) - rank.indexOf(b.kind));
    cols.push({ kind: "mask", name: "MASK", unit: "", desc: MASK_DESC, args: new Map() });
    return cols;
  }

  function hasOverride(step: ChainStep, col: GridCol): boolean {
    if (col.kind === "param") return step.params[col.name] !== undefined;
    if (col.kind === "log_in") return step.log_inputs[col.name] !== undefined;
    return step.opts[col.kind === "mask" ? "MASK" : col.name] !== undefined;
  }

  // Live grid controls, keyed by column then step, so a Set-all edit refreshes the
  // affected column IN PLACE — no rebuild, so scroll position, focus and any
  // .workflow-invalid state elsewhere in the grid are untouched.
  const colKey = (col: GridCol): string => `${col.kind}:${col.name}`;
  const gridCells = new Map<string, Map<ChainStep, { control: HTMLInputElement | HTMLSelectElement; td: HTMLTableCellElement }>>();
  const gridBadges = new Map<ChainStep, HTMLElement>();

  function refreshColumn(col: GridCol): void {
    const cells = gridCells.get(colKey(col));
    if (!cells) return;
    for (const [step, { control, td }] of cells) {
      const arg = col.args.get(step);
      if (col.kind === "param" && arg) {
        control.value = step.params[arg.name] !== undefined ? String(step.params[arg.name]) : arg.default;
        control.classList.remove("workflow-invalid");
      } else if (col.kind === "log_in" && arg) {
        control.value = step.log_inputs[arg.name] ?? arg.default;
      } else if (col.kind === "option" && arg) {
        control.value = step.opts[arg.name] ?? arg.default;
      } else if (col.kind === "mask") {
        control.value = step.opts.MASK ?? "";
      }
      td.classList.toggle("workflow-grid-mod", hasOverride(step, col));
      const badge = gridBadges.get(step);
      if (badge) updateBadge(step, badge);
    }
  }

  function gridCell(step: ChainStep, col: GridCol, badge: HTMLElement, known: boolean): HTMLTableCellElement {
    const td = document.createElement("td");
    const arg = col.args.get(step);
    if (!known || (col.kind !== "mask" && !arg)) {
      td.className = "workflow-grid-na";
      td.textContent = "—";
      return td;
    }
    const onChanged = (): void => {
      updateBadge(step, badge);
      td.classList.toggle("workflow-grid-mod", hasOverride(step, col));
    };
    let control: HTMLInputElement | HTMLSelectElement;
    if (col.kind === "mask") control = maskControl(step, onChanged);
    else if (col.kind === "log_in") control = logInControl(step, arg!, onChanged);
    else if (col.kind === "option") control = optionControl(step, arg!, onChanged);
    else control = paramControl(step, arg!, onChanged);
    td.title = arg?.desc || col.desc;
    td.classList.toggle("workflow-grid-mod", hasOverride(step, col));
    td.appendChild(control);
    let cells = gridCells.get(colKey(col));
    if (!cells) {
      cells = new Map();
      gridCells.set(colKey(col), cells);
    }
    cells.set(step, { control, td });
    return td;
  }

  /** Set-all row cell: one edit fans out to every step that takes this arg. Values are
   *  validated against each step's own manifest limits; per-step delete-if-default keeps
   *  the override semantics identical to editing the cells one by one. */
  function setAllCell(col: GridCol): HTMLTableCellElement {
    const td = document.createElement("td");
    if (col.kind === "param") {
      const input = document.createElement("input");
      input.type = "number";
      input.step = "any";
      input.placeholder = "all";
      input.addEventListener("change", () => {
        const v = parseFloat(input.value);
        if (Number.isNaN(v)) return;
        let applied = 0;
        let skipped = 0;
        for (const [step, arg] of col.args) {
          if ((arg.min !== null && v < arg.min) || (arg.max !== null && v > arg.max)) {
            skipped++;
            continue;
          }
          if (v === parseFloat(arg.default)) delete step.params[arg.name];
          else step.params[arg.name] = v;
          applied++;
        }
        refreshColumn(col);
        setStatus(
          `${col.name} = ${v} set on ${applied} step${applied === 1 ? "" : "s"}` +
            (skipped ? ` (${skipped} skipped: out of range)` : ""),
        );
      });
      td.appendChild(input);
      return td;
    }
    const select = document.createElement("select");
    const ph = document.createElement("option");
    ph.value = "";
    ph.textContent = "(set all)";
    select.appendChild(ph);
    let values: string[];
    if (col.kind === "log_in") {
      // Off-catalog manifest defaults must stay pickable so Set-all can revert every
      // step to its default (which deletes the overrides), same as the per-cell select.
      const extras = [...new Set([...col.args.values()].map((a) => a.default))].filter(
        (d) => !inputCurveNames.includes(d),
      );
      values = [...extras, ...inputCurveNames];
    } else if (col.kind === "mask") values = ["(none)", ...maskCurveNames(curveNames)];
    else {
      // Options only get a set-all control when every step offers the same choices.
      const lists = [...col.args.values()].map((a) => a.choices.join("\n"));
      if (lists.length === 0 || !lists.every((l) => l === lists[0])) {
        td.className = "workflow-grid-na";
        td.textContent = "—";
        return td;
      }
      values = [...col.args.values()][0].choices;
    }
    for (const v of values) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = v;
      select.appendChild(o);
    }
    select.addEventListener("change", () => {
      const v = select.value;
      if (!v) return;
      let applied = 0;
      if (col.kind === "mask") {
        for (const step of steps) {
          if (!moduleByName.has(step.module)) continue;
          if (v === "(none)") delete step.opts.MASK;
          else step.opts.MASK = v;
          applied++;
        }
      } else if (col.kind === "log_in") {
        for (const [step, arg] of col.args) {
          if (v === arg.default) delete step.log_inputs[arg.name];
          else step.log_inputs[arg.name] = v;
          applied++;
        }
      } else {
        for (const [step, arg] of col.args) {
          if (v === arg.default) delete step.opts[arg.name];
          else step.opts[arg.name] = v;
          applied++;
        }
      }
      refreshColumn(col);
      setStatus(`${col.kind === "mask" ? "Mask" : col.name} set on ${applied} step${applied === 1 ? "" : "s"}`);
    });
    td.appendChild(select);
    return td;
  }

  function renderGridView(): void {
    // Structural rebuilds (add/move/remove/load/view toggle) must not lose the scroll
    // position — the grid is wider than the pane for any realistic chain.
    const scrollX = gridWrap.scrollLeft;
    const scrollY = gridWrap.scrollTop;
    gridWrap.innerHTML = "";
    gridCells.clear();
    gridBadges.clear();
    if (steps.length === 0) {
      const empty = document.createElement("div");
      empty.className = "workflow-empty";
      empty.textContent = "No steps yet — add modules above.";
      gridWrap.appendChild(empty);
      return;
    }
    const cols = gridColumns();
    const table = document.createElement("table");
    table.className = "workflow-grid";

    const thead = document.createElement("thead");
    const headRow = document.createElement("tr");
    const corner = document.createElement("th");
    corner.textContent = "Step";
    headRow.appendChild(corner);
    for (const col of cols) {
      const th = document.createElement("th");
      th.textContent = col.kind === "mask" ? "Mask" : col.name;
      if (col.unit) {
        const u = document.createElement("span");
        u.className = "workflow-grid-unit";
        u.textContent = col.unit;
        th.appendChild(u);
      }
      th.title = col.desc;
      headRow.appendChild(th);
    }
    thead.appendChild(headRow);
    table.appendChild(thead);

    const tbody = document.createElement("tbody");
    const allRow = document.createElement("tr");
    allRow.className = "workflow-grid-allrow";
    const allTh = document.createElement("th");
    allTh.textContent = "Set all";
    allTh.title = "Edits in this row apply to every step that takes the parameter";
    allRow.appendChild(allTh);
    for (const col of cols) {
      allRow.appendChild(setAllCell(col));
    }
    tbody.appendChild(allRow);

    steps.forEach((step, i) => {
      const spec = moduleByName.get(step.module);
      const tr = document.createElement("tr");
      const th = document.createElement("th");
      const title = document.createElement("span");
      title.textContent = `${i + 1}. ${spec?.title ?? step.module}`;
      const badge = document.createElement("span");
      badge.className = "workflow-step-badge";
      updateBadge(step, badge);
      gridBadges.set(step, badge);
      th.append(title, badge);
      tr.appendChild(th);
      for (const col of cols) {
        tr.appendChild(gridCell(step, col, badge, !!spec));
      }
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);
    gridWrap.appendChild(table);
    gridWrap.scrollLeft = scrollX;
    gridWrap.scrollTop = scrollY;
  }

  // --- Wells (scope, not a checklist) --------------------------------------
  content.appendChild(scope.el);

  // --- Input cons: strict dropdown of existing constellations (you can only read from
  // one that exists); blank = current values (chained outputs always resolve) ----------
  const inSetSelect = document.createElement("select");
  const inSetLatest = document.createElement("option");
  inSetLatest.value = "";
  inSetLatest.textContent = "(latest values)";
  inSetSelect.appendChild(inSetLatest);
  content.appendChild(
    formRow(
      "Input cons",
      inSetSelect,
      "Every step reads its inputs from this constellation where available (latest version per well). Blank = current values.",
    ),
  );

  // --- Output cons (P1-c): one version per chain run, never overwriting. Editable
  // combobox — pick an existing constellation or type a brand-new name. -----------------
  const setInput = document.createElement("input");
  setInput.type = "text";
  setInput.value = "INTERP";
  setInput.setAttribute("list", "log-cons-names");
  let consList = document.querySelector<HTMLDataListElement>("#log-cons-names");
  if (!consList) {
    consList = document.createElement("datalist");
    consList.id = "log-cons-names";
    document.body.appendChild(consList);
  }
  content.appendChild(
    formRow(
      "Output cons",
      setInput,
      "The whole chain run is versioned into this constellation (re-run = version N+1). Pick an existing one or type a new name. Manage versions in the Curve Catalog.",
    ),
  );

  // Fill both pickers from the project's existing constellation names. Input is strict
  // (dropdown only); output offers them as datalist suggestions plus the common defaults.
  void listLogSetNames()
    .then((names) => {
      for (const n of names) {
        const o = document.createElement("option");
        o.value = n;
        o.textContent = n;
        inSetSelect.appendChild(o);
      }
      const seeds = [...new Set(["INTERP", "FINAL", "TEST", ...names])];
      consList!.innerHTML = "";
      for (const n of seeds) {
        const o = document.createElement("option");
        o.value = n;
        consList!.appendChild(o);
      }
    })
    .catch(() => {});

  // --- Run bar -------------------------------------------------------------
  // No inline progress bar here — the universal Processing panel owns the live bar, per-well
  // status and Cancel. This dialog keeps only a one-line status for quick confirmation.
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
      statusLine.textContent = `Step ${s.step + 1}/${s.total_steps}: ${moduleByName.get(s.module)?.title ?? s.module} — see Processing panel`;
    } else if (s.state === "completed") {
      const errNote = s.errors.length ? ` — ${s.errors.length} well/step warnings` : "";
      statusLine.textContent = `Done: ${s.steps_run} steps, ${s.curves_written} curves across ${s.wells} wells${errNote}`;
      if (s.errors.length) console.warn("chain warnings:", s.errors);
      finishRun();
      bumpDataVersion();
      setStatus(`Workflow finished (${s.steps_run} steps, ${s.wells} wells)`);
    } else if (s.state === "cancelled") {
      statusLine.textContent = `Cancelled at step ${s.at_step + 1}`;
      finishRun();
      // A cancelled chain routinely committed the earlier wells/steps before draining, so
      // refresh open plots/log views rather than leaving them on pre-run curves.
      bumpDataVersion();
      setStatus("Workflow cancelled");
    } else if (s.state === "failed") {
      statusLine.textContent = `Failed: ${s.error}`;
      finishRun();
      // A mid-chain failure can leave earlier steps/wells committed — refresh open views.
      bumpDataVersion();
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
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    const jobId = crypto.randomUUID();
    currentJob = jobId;
    running = true;
    runBtn.disabled = true;
    cancelBtn.disabled = false;
    statusLine.textContent = "Starting… (progress in the Processing panel)";
    recordProcess("Workflow", `Ran chain (${steps.length} step(s) × ${wellIds.length} well(s))`);
    // Pop open the universal Processing panel so live per-well progress + Cancel are visible
    // without the user hunting for it (the ribbon listens for this).
    window.dispatchEvent(new CustomEvent("sandibumi:open-processing"));

    // Fire the (blocking) run without awaiting so we can poll progress meanwhile.
    void runWorkflowChain(
      jobId,
      steps,
      wellIds,
      setInput.value.trim() || undefined,
      inSetSelect.value.trim() || undefined,
    ).catch((e) => {
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
    if (!currentJob) return;
    // The run drains in a well or two; the poll confirms the Cancelled state shortly after.
    cancelBtn.disabled = true;
    statusLine.textContent = "Cancelling…";
    await cancelWorkflowChain(currentJob).catch(() => {});
  });

  content.append(runRow, statusLine);

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
      scope.dispose();
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
