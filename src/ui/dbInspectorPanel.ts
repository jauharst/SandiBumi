import {
  getTablePage,
  setZoneParam,
  updateComputedSample,
  updateCoreSample,
  updateStandardSample,
  updateWellField,
  upsertTop,
  upsertZone,
  type TablePage,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { pushUndo } from "../undo";

const PAGE_SIZE = 200;

interface TableDef {
  key: string;
  label: string;
  wellScoped: boolean;
  /** Column names editable via double-click. */
  editable: string[];
}

const TABLES: TableDef[] = [
  { key: "wells", label: "Wells", wellScoped: false, editable: ["well_name", "field_name", "td", "kb"] },
  { key: "standard_curves", label: "Standard Curves", wellScoped: true, editable: ["gr", "res_deep", "nphi", "rhob", "dt", "sp"] },
  { key: "computed_curves", label: "Computed Curves", wellScoped: true, editable: ["value"] },
  { key: "tops", label: "Tops", wellScoped: true, editable: ["depth", "color"] },
  { key: "zones", label: "Zones", wellScoped: true, editable: ["top_depth", "bottom_depth"] },
  { key: "zone_params", label: "Zone Parameters", wellScoped: true, editable: ["value_num", "value_text"] },
  { key: "core_data", label: "Core Data", wellScoped: true, editable: ["cpor", "cperm", "cgd", "csw"] },
];

/** Geolog "Text"-style editable grid over the project database: pick a table, page
 *  through rows, double-click a cell to edit. Every edit goes through a whitelisted
 *  IPC command, refreshes open views, and lands on the undo stack (Ctrl+Z). */
export class DbInspectorPanel {
  private root: HTMLElement;
  private tableSel!: HTMLSelectElement;
  private scopeEl!: HTMLElement;
  private gridHost!: HTMLElement;
  private pageInfo!: HTMLElement;
  private prevBtn!: HTMLButtonElement;
  private nextBtn!: HTMLButtonElement;

  private offset = 0;
  private page: TablePage | null = null;
  private unsub: (() => void)[] = [];

  constructor(host: HTMLElement) {
    this.root = document.createElement("div");
    this.root.className = "dbinspector";
    this.root.innerHTML = `
      <div class="dbi-toolbar">
        <label class="chk-field">Table <select class="form-control dbi-table"></select></label>
        <span class="dbi-scope"></span>
        <span class="dbi-spacer"></span>
        <button class="lp-btn dbi-prev">◀</button>
        <span class="dbi-pageinfo"></span>
        <button class="lp-btn dbi-next">▶</button>
      </div>
      <div class="dbi-grid"></div>
      <p class="modal-hint">Double-click a cell to edit; Enter commits, Esc cancels. Edits are undoable (Ctrl+Z).</p>`;
    host.appendChild(this.root);

    this.tableSel = this.root.querySelector<HTMLSelectElement>(".dbi-table")!;
    this.scopeEl = this.root.querySelector<HTMLElement>(".dbi-scope")!;
    this.gridHost = this.root.querySelector<HTMLElement>(".dbi-grid")!;
    this.pageInfo = this.root.querySelector<HTMLElement>(".dbi-pageinfo")!;
    this.prevBtn = this.root.querySelector<HTMLButtonElement>(".dbi-prev")!;
    this.nextBtn = this.root.querySelector<HTMLButtonElement>(".dbi-next")!;

    for (const t of TABLES) {
      const option = document.createElement("option");
      option.value = t.key;
      option.textContent = t.label;
      this.tableSel.appendChild(option);
    }
    this.tableSel.value = "standard_curves";

    this.tableSel.addEventListener("change", () => {
      this.offset = 0;
      void this.reload();
    });
    this.prevBtn.addEventListener("click", () => {
      this.offset = Math.max(0, this.offset - PAGE_SIZE);
      void this.reload();
    });
    this.nextBtn.addEventListener("click", () => {
      if (this.page && this.offset + PAGE_SIZE < this.page.total_rows) {
        this.offset += PAGE_SIZE;
        void this.reload();
      }
    });

    this.unsub.push(
      appState.selectedWell.subscribe(() => {
        this.offset = 0;
        void this.reload();
      }),
      appState.dataVersion.subscribe(() => void this.reload()),
    );
  }

  dispose(): void {
    for (const u of this.unsub) u();
  }

  private tableDef(): TableDef {
    return TABLES.find((t) => t.key === this.tableSel.value)!;
  }

  private async reload(): Promise<void> {
    const def = this.tableDef();
    const well = appState.selectedWell.get();
    this.scopeEl.textContent = def.wellScoped ? (well ? `Well: ${well.well_name}` : "— select a well —") : "(whole project)";
    if (def.wellScoped && !well) {
      this.gridHost.innerHTML = `<div class="placeholder-note">Select a well in Wells &amp; Tops to browse ${def.label}.</div>`;
      this.pageInfo.textContent = "";
      return;
    }
    try {
      this.page = await getTablePage(def.key, def.wellScoped ? well!.well_id : null, this.offset, PAGE_SIZE);
    } catch (err) {
      this.gridHost.innerHTML = `<div class="placeholder-note">Load failed: ${err}</div>`;
      this.pageInfo.textContent = "";
      return;
    }
    this.renderGrid();
  }

  private renderGrid(): void {
    const page = this.page!;
    const def = this.tableDef();
    const from = page.total_rows === 0 ? 0 : this.offset + 1;
    const to = Math.min(this.offset + page.rows.length, page.total_rows);
    this.pageInfo.textContent = `${from}–${to} of ${page.total_rows}`;
    this.prevBtn.disabled = this.offset === 0;
    this.nextBtn.disabled = this.offset + PAGE_SIZE >= page.total_rows;

    const table = document.createElement("table");
    table.className = "dbgrid";
    const thead = document.createElement("thead");
    const headRow = document.createElement("tr");
    for (const col of page.columns) {
      const th = document.createElement("th");
      th.textContent = col;
      if (def.editable.includes(col)) th.classList.add("editable");
      headRow.appendChild(th);
    }
    thead.appendChild(headRow);
    table.appendChild(thead);

    const tbody = document.createElement("tbody");
    page.rows.forEach((row) => {
      const tr = document.createElement("tr");
      row.forEach((cell, colIdx) => {
        const td = document.createElement("td");
        td.textContent = cell ?? "";
        if (cell === null) td.classList.add("null");
        const col = page.columns[colIdx];
        if (def.editable.includes(col)) {
          td.classList.add("editable");
          td.title = "Double-click to edit";
          td.addEventListener("dblclick", () => this.beginEdit(td, row, col));
        }
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);

    this.gridHost.innerHTML = "";
    this.gridHost.appendChild(table);
  }

  private beginEdit(td: HTMLElement, row: (string | null)[], column: string): void {
    const oldValue = td.textContent ?? "";
    const input = document.createElement("input");
    input.className = "dbgrid-edit";
    input.value = oldValue;
    td.textContent = "";
    td.appendChild(input);
    input.focus();
    input.select();

    let done = false;
    const finish = (commit: boolean) => {
      if (done) return;
      done = true;
      const newValue = input.value.trim();
      input.remove();
      td.textContent = commit ? newValue : oldValue;
      if (commit && newValue !== oldValue.trim()) {
        void this.commitEdit(row, column, oldValue.trim(), newValue).catch((err) => {
          td.textContent = oldValue;
          setStatus(`Edit failed: ${err}`);
        });
      }
    };
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") finish(true);
      if (e.key === "Escape") finish(false);
      e.stopPropagation();
    });
    input.addEventListener("blur", () => finish(true));
  }

  /** Applies one cell edit via the matching whitelisted command and records its undo. */
  private async commitEdit(row: (string | null)[], column: string, oldValue: string, newValue: string): Promise<void> {
    const def = this.tableDef();
    const page = this.page!;
    const cell = (name: string): string => {
      const idx = page.columns.indexOf(name);
      return (idx >= 0 ? row[idx] : null) ?? "";
    };
    const num = (s: string): number => {
      if (s === "") return NaN;
      const v = parseFloat(s);
      if (Number.isNaN(v)) throw new Error(`'${s}' is not a number`);
      return v;
    };
    const well = appState.selectedWell.get();
    const wellId = def.key === "wells" ? cell("well_id") : well?.well_id;
    if (!wellId) throw new Error("no well in scope");

    // apply(value) writes one value; called with newValue now and oldValue on undo.
    let apply: (value: string) => Promise<void>;
    switch (def.key) {
      case "wells":
        apply = (v) => updateWellField(wellId, column, v === "" ? null : v);
        break;
      case "standard_curves": {
        const depth = num(cell("depth"));
        apply = (v) => updateStandardSample(wellId, depth, column, v === "" ? NaN : num(v));
        break;
      }
      case "computed_curves": {
        const depth = num(cell("depth"));
        const curve = cell("curve_name");
        apply = (v) => updateComputedSample(wellId, depth, curve, v === "" ? NaN : num(v));
        break;
      }
      case "tops": {
        const topName = cell("top_name");
        apply = (v) => {
          const depth = column === "depth" ? num(v) : num(cell("depth"));
          const color = column === "color" ? (v === "" ? null : v) : cell("color") || null;
          return upsertTop(wellId, topName, depth, color);
        };
        break;
      }
      case "zones": {
        const zoneName = cell("zone_name");
        apply = (v) => {
          const top = column === "top_depth" ? num(v) : num(cell("top_depth"));
          const bottom = column === "bottom_depth" ? num(v) : num(cell("bottom_depth"));
          return upsertZone(wellId, zoneName, top, bottom);
        };
        break;
      }
      case "zone_params": {
        const zoneName = cell("zone_name");
        const paramName = cell("param_name");
        apply = (v) => {
          const valueNum = column === "value_num" ? (v === "" ? null : num(v)) : cell("value_num") === "" ? null : num(cell("value_num"));
          const valueText = column === "value_text" ? (v === "" ? null : v) : cell("value_text") || null;
          return setZoneParam(wellId, zoneName, paramName, valueNum, valueText);
        };
        break;
      }
      case "core_data": {
        const depth = num(cell("depth"));
        apply = (v) => updateCoreSample(wellId, depth, column, v === "" ? NaN : num(v));
        break;
      }
      default:
        throw new Error(`table '${def.key}' is not editable`);
    }

    await apply(newValue);
    const label = `edit ${def.label}.${column}`;
    pushUndo({
      label,
      undo: async () => {
        await apply(oldValue);
        bumpDataVersion();
      },
      redo: async () => {
        await apply(newValue);
        bumpDataVersion();
      },
    });
    setStatus(`${label}: '${oldValue}' → '${newValue}'`);
    bumpDataVersion();
  }
}

