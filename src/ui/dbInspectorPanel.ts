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
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { messageNode } from "./safeDom";
import { pushUndo } from "../undo";
import { requestRunCustody } from "./runCustody";

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
  // Tops-style auxiliary datasets (petrography / XRD / perforations) — read-only view;
  // re-import the file to change values.
  { key: "aux_data", label: "Aux Data", wellScoped: true, editable: [] },
];

/** A rendered page together with the exact scope it was fetched under. Threading this into the grid
 *  and its edit closures — instead of re-reading `tableSel`/`selectedWell` live at paint/commit time
 *  — is what keeps a cell edit bound to the rows actually on screen when a reload is in flight. */
interface GridView {
  def: TableDef;
  well: WellSummary | null;
  offset: number;
  page: TablePage;
}

/** Spreadsheet-style editable grid over the project database: pick a table, page
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
  /** The currently displayed page bundled with the (def, well, offset) it was fetched for, so a
   *  cell edit always writes against the data actually on screen. `null` while no editable page
   *  is shown (no well selected, load error). */
  private view: GridView | null = null;
  private unsub: (() => void)[] = [];
  /** Bumped at the start of every reload; a reload whose token is stale by the time its page
   *  resolves drops the result, so a slow response can never paint under a newer table/well. */
  private reloadGen = 0;
  private disposed = false;

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
      if (this.view && this.offset + PAGE_SIZE < this.view.page.total_rows) {
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
    this.disposed = true;
    for (const u of this.unsub) u();
  }

  private tableDef(): TableDef {
    return TABLES.find((t) => t.key === this.tableSel.value)!;
  }

  private async reload(): Promise<void> {
    // Snapshot everything this load depends on up front and tag it with a token. Two subscriptions
    // (selectedWell + dataVersion) plus the toolbar can start reloads back-to-back; without the
    // token the slower getTablePage could resolve last and paint its rows under whatever table/well
    // is live by then. With it, only the newest reload renders — and because the (def, well, offset)
    // travel with the page into renderGrid/commitEdit, an edit can never be committed under a
    // different table's rules or against a different well than the rows currently on screen.
    const gen = ++this.reloadGen;
    const def = this.tableDef();
    const well = appState.selectedWell.get();
    const offset = this.offset;
    this.scopeEl.textContent = def.wellScoped ? (well ? `Well: ${well.well_name}` : "— select a well —") : "(whole project)";
    if (def.wellScoped && !well) {
      this.view = null;
      // `def.label` is a static internal table label, not untrusted — but keeping the whole
      // panel on the textContent path means no future edit here reintroduces an innerHTML sink.
      this.gridHost.replaceChildren(
        messageNode("placeholder-note", `Select a well in Wells & Tops to browse ${def.label}.`),
      );
      this.pageInfo.textContent = "";
      return;
    }
    let page: TablePage;
    try {
      page = await getTablePage(def.key, def.wellScoped ? well!.well_id : null, offset, PAGE_SIZE);
    } catch (err) {
      if (this.disposed || gen !== this.reloadGen) return;
      this.gridHost.replaceChildren(messageNode("placeholder-note", `Load failed: ${err}`));
      this.pageInfo.textContent = "";
      return;
    }
    if (this.disposed || gen !== this.reloadGen) return; // a newer reload owns the panel now
    const view: GridView = { def, well, offset, page };
    this.view = view;
    this.renderGrid(view);
  }

  private renderGrid(view: GridView): void {
    const { def, page, offset } = view;
    const from = page.total_rows === 0 ? 0 : offset + 1;
    const to = Math.min(offset + page.rows.length, page.total_rows);
    this.pageInfo.textContent = `${from}–${to} of ${page.total_rows}`;
    this.prevBtn.disabled = offset === 0;
    this.nextBtn.disabled = offset + PAGE_SIZE >= page.total_rows;

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
          td.addEventListener("dblclick", () => this.beginEdit(view, td, row, col));
        }
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);

    this.gridHost.innerHTML = "";
    this.gridHost.appendChild(table);
  }

  private beginEdit(view: GridView, td: HTMLElement, row: (string | null)[], column: string): void {
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
        void this.commitEdit(view, row, column, oldValue.trim(), newValue).catch((err) => {
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
  private async commitEdit(view: GridView, row: (string | null)[], column: string, oldValue: string, newValue: string): Promise<void> {
    // def / well / page come from the view the edited row was rendered from — never a live re-read,
    // so a well or table switch that landed mid-edit cannot redirect this write to another well or
    // interpret the row under another table's schema.
    const { def, well, page } = view;
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
    const wellId = def.key === "wells" ? cell("well_id") : well?.well_id;
    if (!wellId) throw new Error("no well in scope");

    // apply(value) writes one value; called with newValue now and oldValue on undo.
    let apply: (value: string) => Promise<void>;
    let undoApply: ((value: string) => Promise<void>) | null = null;
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
        const custody = await requestRunCustody(`Edit ${curve} sample`);
        if (!custody) throw new Error("computed curve edit cancelled — nothing was written");
        const undoCustody = {
          actor: custody.actor,
          source_note: `Undo of prior computed-sample edit; original source/reference: ${custody.source_note}`,
        };
        apply = (v) => updateComputedSample(wellId, depth, curve, v === "" ? NaN : num(v), custody);
        undoApply = (v) => updateComputedSample(wellId, depth, curve, v === "" ? NaN : num(v), undoCustody);
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
        await (undoApply ?? apply)(oldValue);
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
