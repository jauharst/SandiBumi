import {
  listModules,
  listWells,
  listWellParamOverrides,
  setWellParamOverrides,
  type ArgSpec,
  type BackendWellScope,
  type ChainStep,
  type ModuleSpec,
  type WellSummary,
} from "../ipc";
import { setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { buildWellScope } from "./wellScope";

/** Per-well parameter override grid (Phase 9-2, the last open Phase 9 item).
 *
 *  A workflow step carries ONE parameter set for every well it runs on, which breaks as soon
 *  as a field needs a different Rw per fault block or a different RHO_MA per facies belt.
 *  The data model already allowed the fix — a `zone_params` row with zone `*` is a whole-well
 *  override, and `workflow::resolve_param_arrays` already applies it — but reaching it meant
 *  selecting one well at a time in the Zones panel. At 2,000 wells that is not a workflow.
 *
 *  So this is a grid, not new machinery: rows are wells, columns are the numeric parameters
 *  the chain's steps actually take, and a cell writes that one `zone_params` row.
 *
 *  Three rules it inherits rather than invents:
 *
 *   - **Resolution order stays** step value → whole-well override (here) → named zone. A cell
 *     shows the inherited value dimmed until you override it, so the grid never implies a well
 *     has been given a value it hasn't.
 *   - **Only differences are stored** (same as the per-step editors): typing the inherited
 *     value back clears the override rather than storing a duplicate.
 *   - **Out of range is refused, not clamped.** `resolve_param_arrays` REJECTS an out-of-spec
 *     `zone_params` value and fails the whole run — a v/v fraction typed as a percentage is the
 *     documented cause. Catching it in the cell turns a failed 2,000-well run into a red cell.
 */

/** A grid column: one distinct parameter name, plus which modules asked for it. */
interface ParamColumn {
  name: string;
  arg: ArgSpec;
  /** Module titles taking this parameter — one parameter can be shared by several steps. */
  modules: string[];
  /** The step-level value every well inherits when it has no override of its own. */
  inherited: number;
  /** Two steps take this parameter with DIFFERENT step-level values. Then "inherited" is
   *  only the first step's, and clearing an override does not give every step the displayed
   *  number — it restores the disagreement. The only-store-differences shortcut is therefore
   *  switched off for such a column (see `overrideValueFor`). */
  conflict: boolean;
}

/** What to store for a typed value: `null` clears the override, a number stores it.
 *
 *  Normally typing the inherited value back means "no override" — the app-wide
 *  only-store-differences rule. But when the chain's steps disagree on this parameter,
 *  clearing is NOT equivalent to typing the displayed number: without an override each step
 *  keeps its own value, so a user who types the displayed 0.05 to mean "0.05 everywhere"
 *  would silently leave the other step on 0.07. In that case the value is always stored,
 *  because storing it is the only way to make the steps agree. */
export function overrideValueFor(col: { inherited: number; conflict: boolean }, typed: number): number | null {
  return !col.conflict && typed === col.inherited ? null : typed;
}

/** Builds the column set from the chain's steps. Parameters are keyed by NAME, not by step,
 *  because `zone_params` is keyed that way too: one RW override applies to every step that
 *  takes RW. Collapsing them into one column is therefore the truthful presentation — a
 *  column per step would imply an independence the storage does not have.
 *
 *  When two steps disagree about a shared parameter's step-level value, the inherited number
 *  shown is the first step's, and the header says the value is not shared. */
export function buildParamColumns(steps: ChainStep[], specs: Map<string, ModuleSpec>): ParamColumn[] {
  const cols = new Map<string, ParamColumn & { conflict: boolean }>();
  for (const step of steps) {
    const spec = specs.get(step.module);
    if (!spec) continue;
    for (const arg of spec.args) {
      if (arg.kind !== "param") continue;
      const stepValue = step.params[arg.name] ?? parseFloat(arg.default);
      if (!Number.isFinite(stepValue)) continue;
      const existing = cols.get(arg.name);
      if (existing) {
        if (!existing.modules.includes(spec.title)) existing.modules.push(spec.title);
        if (existing.inherited !== stepValue) existing.conflict = true;
      } else {
        cols.set(arg.name, { name: arg.name, arg, modules: [spec.title], inherited: stepValue, conflict: false });
      }
    }
  }
  return [...cols.values()].map((c) => ({
    name: c.name,
    arg: c.arg,
    modules: c.modules,
    inherited: c.inherited,
    conflict: c.conflict,
  }));
}

/** `well_id -> param_name -> value`, from the flat backend list. */
function indexOverrides(rows: { well_id: string; param_name: string; value_num: number }[]): Map<string, Map<string, number>> {
  const out = new Map<string, Map<string, number>>();
  for (const r of rows) {
    let byParam = out.get(r.well_id);
    if (!byParam) {
      byParam = new Map();
      out.set(r.well_id, byParam);
    }
    byParam.set(r.param_name, r.value_num);
  }
  return out;
}

/** Is `v` inside the parameter's declared range? Mirrors `resolve_param_arrays`'s test
 *  exactly, including that a non-finite value is out of range by definition. */
export function paramInRange(arg: ArgSpec, v: number): boolean {
  if (!Number.isFinite(v)) return false;
  return (arg.min === null || v >= arg.min) && (arg.max === null || v <= arg.max);
}

export function rangeLabel(arg: ArgSpec): string {
  if (arg.min !== null && arg.max !== null) return `valid ${arg.min} to ${arg.max}`;
  if (arg.min !== null) return `valid >= ${arg.min}`;
  if (arg.max !== null) return `valid <= ${arg.max}`;
  return "no declared range";
}

/** Opens the grid over a chain's steps. Returns a dispose for the caller's cleanup. */
export async function openWellParamsDialog(steps: ChainStep[]): Promise<void> {
  const specList = await listModules().catch(() => [] as ModuleSpec[]);
  const specs = new Map(specList.map((s) => [s.name, s] as const));
  const columns = buildParamColumns(steps, specs);
  const allWells: WellSummary[] = await listWells().catch(() => []);
  const wellName = new Map(allWells.map((w) => [w.well_id, w.well_name] as const));
  let overrides = indexOverrides(await listWellParamOverrides().catch(() => []));

  const body = document.createElement("div");

  if (columns.length === 0) {
    const empty = document.createElement("p");
    empty.className = "modal-hint";
    empty.textContent =
      "This workflow has no numeric parameters to override yet — add a step first, then reopen this grid.";
    body.appendChild(empty);
    openModal("Per-Well Parameters", body, 460);
    return;
  }

  // --- Which wells are rows -------------------------------------------------
  let scopedIds: string[] = [];
  const scope = await buildWellScope({
    onChange: (ids) => {
      scopedIds = ids;
      renderRows();
    },
  });
  scopedIds = scope.getWellIds();

  const filterIn = document.createElement("input");
  filterIn.type = "search";
  filterIn.className = "form-control";
  filterIn.placeholder = "filter wells…";
  filterIn.addEventListener("input", renderRows);

  const summary = document.createElement("p");
  summary.className = "modal-hint";

  // --- Table ----------------------------------------------------------------
  const wrap = document.createElement("div");
  wrap.className = "wellparams-scroll";
  const table = document.createElement("table");
  table.className = "wellparams-grid";
  const thead = document.createElement("thead");
  const headRow = document.createElement("tr");
  const wellTh = document.createElement("th");
  wellTh.textContent = "Well";
  wellTh.className = "wellparams-wellcol";
  headRow.appendChild(wellTh);
  for (const col of columns) {
    const th = document.createElement("th");
    const unit = col.arg.unit ? ` (${col.arg.unit})` : "";
    // A conflicted column is marked in the header, not just in a tooltip: the number below
    // it is only one of the steps' values, which changes what an edit there means.
    th.textContent = col.name + unit + (col.conflict ? " ⚠" : "");
    th.title =
      `${col.arg.desc}\n${rangeLabel(col.arg)}\n` +
      (col.conflict
        ? `Steps DISAGREE on this parameter (showing ${col.inherited}, from ${col.modules[0]}). ` +
          `Used by: ${col.modules.join(", ")}. Setting a value here makes every step use it for that well.`
        : `Step value ${col.inherited} — used by: ${col.modules.join(", ")}`);
    headRow.appendChild(th);
  }
  thead.appendChild(headRow);
  const tbody = document.createElement("tbody");
  table.append(thead, tbody);
  wrap.appendChild(table);

  /** Wells currently shown, after scope + name filter. */
  function shownWells(): string[] {
    const q = filterIn.value.trim().toLowerCase();
    return scopedIds.filter((id) => {
      if (!wellName.has(id)) return false;
      return !q || (wellName.get(id) ?? "").toLowerCase().includes(q);
    });
  }

  /** Writes a batch and records it as ONE undoable action, so a fill-column sweep reverses
   *  in a single Ctrl+Z rather than well by well. The previous values are captured before
   *  the write, and undo/redo replay through the same atomic backend call. */
  async function applyBatch(entries: [string, string, number | null][], label: string): Promise<void> {
    if (entries.length === 0) return;
    const before: [string, string, number | null][] = entries.map(([wellId, param]) => [
      wellId,
      param,
      overrides.get(wellId)?.get(param) ?? null,
    ]);
    const explicitScope = (batch: [string, string, number | null][]): BackendWellScope => ({
      kind: "explicit",
      well_ids: [...new Set(batch.map(([wellId]) => wellId))],
    });
    const write = async (batch: [string, string, number | null][], backendScope: BackendWellScope) => {
      await setWellParamOverrides(batch, backendScope);
      for (const [wellId, param, value] of batch) {
        let byParam = overrides.get(wellId);
        if (!byParam) {
          byParam = new Map();
          overrides.set(wellId, byParam);
        }
        if (value === null) byParam.delete(param);
        else byParam.set(param, value);
      }
      renderRows();
    };
    try {
      await write(entries, scope.backend());
    } catch (err) {
      setStatus(`Per-well parameters: write failed — ${err}`);
      return;
    }
    recordProcess("Parameters", `${label} (${entries.length} well${entries.length === 1 ? "" : "s"})`);
    setStatus(`${label} — ${entries.length} well${entries.length === 1 ? "" : "s"}`);
    pushUndo({
      label,
      // Undo/redo reverses the exact historical rows, not whatever a saved group contains later.
      undo: () => write(before, explicitScope(before)),
      redo: () => write(entries, explicitScope(entries)),
    });
  }

  /** Turns a cell into an editor on double-click. Cells are plain text until then: the grid
   *  can hold thousands of rows, and the app-wide click-to-arm rule exists precisely so a
   *  stray click near a petrophysical parameter cannot change it. */
  function makeCell(wellId: string, col: ParamColumn): HTMLTableCellElement {
    const td = document.createElement("td");
    const own = overrides.get(wellId)?.get(col.name);
    const isOverride = own !== undefined;
    td.textContent = String(isOverride ? own : col.inherited);
    td.className = isOverride ? "wellparams-cell wellparams-override" : "wellparams-cell wellparams-inherited";
    td.title = isOverride
      ? `Overridden for this well. Double-click to edit${col.conflict ? "" : `; type ${col.inherited} to clear it`}.`
      : col.conflict
        ? `The steps disagree on ${col.name}; showing ${col.inherited}. Double-click to give this well one value for every step.`
        : `Inherited from the step (${col.inherited}). Double-click to override for this well.`;

    td.addEventListener("dblclick", () => {
      if (td.querySelector("input")) return;
      const input = document.createElement("input");
      input.type = "number";
      input.step = "any";
      input.className = "form-control wellparams-input";
      input.value = String(isOverride ? own : col.inherited);
      // The guard's click-to-arm is for stray clicks on a resting field; this input exists
      // only because the user already double-clicked, so it opens ready to type.
      input.setAttribute("data-free-edit", "");
      td.textContent = "";
      td.appendChild(input);
      input.focus();
      input.select();

      let done = false;
      const commit = (save: boolean) => {
        if (done) return;
        done = true;
        const raw = input.value.trim();
        if (!save) {
          renderRows();
          return;
        }
        // Blank clears the override back to the inherited value.
        if (raw === "") {
          void applyBatch([[wellId, col.name, null]], `Clear ${col.name} on ${wellName.get(wellId) ?? wellId}`);
          return;
        }
        const v = parseFloat(raw);
        if (!paramInRange(col.arg, v)) {
          setStatus(`${col.name} = ${raw} is outside the module's range (${rangeLabel(col.arg)}) — not saved`);
          renderRows();
          return;
        }
        const next = overrideValueFor(col, v);
        void applyBatch(
          [[wellId, col.name, next]],
          `${next === null ? "Clear" : "Set"} ${col.name} on ${wellName.get(wellId) ?? wellId}`,
        );
      };
      input.addEventListener("blur", () => commit(true));
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          commit(true);
        } else if (e.key === "Escape") {
          e.preventDefault();
          commit(false);
        }
      });
    });
    return td;
  }

  function renderRows(): void {
    const wells = shownWells();
    tbody.innerHTML = "";
    for (const id of wells) {
      const tr = document.createElement("tr");
      const nameTd = document.createElement("td");
      nameTd.textContent = wellName.get(id) ?? id;
      nameTd.className = "wellparams-wellcol";
      tr.appendChild(nameTd);
      for (const col of columns) tr.appendChild(makeCell(id, col));
      tbody.appendChild(tr);
    }
    let overridden = 0;
    for (const id of wells) {
      const byParam = overrides.get(id);
      if (byParam && columns.some((c) => byParam.has(c.name))) overridden++;
    }
    summary.textContent =
      `${wells.length} well${wells.length === 1 ? "" : "s"} shown · ${overridden} with an override · ` +
      `${columns.length} parameter${columns.length === 1 ? "" : "s"} from this workflow. ` +
      "Amber = overridden for that well; grey = inherited from the step. Zone parameters still win per zone at run time.";
  }

  // --- Column-wide actions (the reason this exists at field scale) ----------
  const colSel = document.createElement("select");
  colSel.className = "form-control";
  for (const col of columns) {
    const o = document.createElement("option");
    o.value = col.name;
    o.textContent = col.name;
    colSel.appendChild(o);
  }
  const fillIn = document.createElement("input");
  fillIn.type = "number";
  fillIn.step = "any";
  fillIn.className = "form-control";
  fillIn.style.width = "96px";
  fillIn.placeholder = "value";

  const currentCol = (): ParamColumn => columns.find((c) => c.name === colSel.value) ?? columns[0];

  const fillBtn = document.createElement("button");
  fillBtn.className = "lp-btn";
  fillBtn.textContent = "Set for all shown";
  fillBtn.addEventListener("click", () => {
    const col = currentCol();
    const raw = fillIn.value.trim();
    if (raw === "") {
      setStatus("Type a value to set across the shown wells");
      return;
    }
    const v = parseFloat(raw);
    if (!paramInRange(col.arg, v)) {
      setStatus(`${col.name} = ${raw} is outside the module's range (${rangeLabel(col.arg)}) — nothing written`);
      return;
    }
    const next = overrideValueFor(col, v);
    const wells = shownWells();
    void applyBatch(
      wells.map((id) => [id, col.name, next] as [string, string, number | null]),
      `${next === null ? "Clear" : `Set ${col.name} = ${v}`} across shown wells`,
    );
  });

  const clearBtn = document.createElement("button");
  clearBtn.className = "lp-btn";
  clearBtn.textContent = "Clear for all shown";
  clearBtn.title = "Remove this parameter's override from every shown well — they fall back to the step value";
  clearBtn.addEventListener("click", () => {
    const col = currentCol();
    const wells = shownWells().filter((id) => overrides.get(id)?.has(col.name));
    if (wells.length === 0) {
      setStatus(`No ${col.name} overrides among the shown wells`);
      return;
    }
    void applyBatch(
      wells.map((id) => [id, col.name, null] as [string, string, number | null]),
      `Clear ${col.name} across shown wells`,
    );
  });

  const csvBtn = document.createElement("button");
  csvBtn.className = "lp-btn";
  csvBtn.textContent = "Copy as CSV";
  csvBtn.title = "Copy the shown grid to the clipboard — edit it in a spreadsheet alongside your own well table";
  csvBtn.addEventListener("click", () => {
    const header = ["Well", ...columns.map((c) => c.name)].join(",");
    const lines = shownWells().map((id) => {
      const byParam = overrides.get(id);
      const cells = columns.map((c) => String(byParam?.get(c.name) ?? c.inherited));
      return [`"${(wellName.get(id) ?? id).replace(/"/g, '""')}"`, ...cells].join(",");
    });
    void navigator.clipboard
      .writeText([header, ...lines].join("\n"))
      .then(() => setStatus(`Copied ${lines.length} well row(s) as CSV`))
      .catch((err) => setStatus(`Clipboard copy failed: ${err}`));
  });

  const actions = document.createElement("div");
  actions.className = "wellparams-actions";
  actions.append(colSel, fillIn, fillBtn, clearBtn, csvBtn);

  body.appendChild(formRow("Wells", scope.el));
  body.appendChild(formRow("Filter", filterIn));
  body.appendChild(actions);
  body.appendChild(wrap);
  body.appendChild(summary);

  renderRows();
  openModal("Per-Well Parameters", body, 820);

  // The well-scope control subscribes to live pin/selection state, so it has to be disposed
  // when this dialog goes away. `openModal` exposes no close hook, and the dialog can close
  // three ways (✕, Escape, or another dialog superseding it) — all of which empty
  // `#modal-root`. Watching that one node's children catches every path; watching the
  // document would fire on every canvas repaint in the app.
  const modalRoot = document.querySelector("#modal-root");
  if (modalRoot) {
    const observer = new MutationObserver(() => {
      if (!body.isConnected) {
        scope.dispose();
        observer.disconnect();
      }
    });
    observer.observe(modalRoot, { childList: true });
  }
}
