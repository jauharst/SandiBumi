import { basicSetup, EditorView } from "codemirror";
import { python } from "@codemirror/lang-python";
import { bumpDataVersion, filterByActiveGroup, setStatus as globalStatus } from "../state";
import { recordProcess } from "../processLog";
import {
  saveEquation,
  listEquations,
  runEquation,
  listCurveCatalog,
  listComputedCatalog,
  listGenericCurveCatalog,
  listLogSets,
  listWells,
  deleteLogSet,
  pythonStatus,
  restoreLogSet,
  type ComputedCatalogEntry,
  type EquationDef,
  type EquationRunResult,
  type GenericCurveCatalogEntry,
  type LogSetEntry,
} from "../ipc";

const BLANK_EQUATION: EquationDef = {
  equation_id: "",
  name: "",
  description: "",
  script: "",
  input_curves: [],
  output_curve: "",
  output_units: "",
  language: "python",
};

const LANGUAGE_NOTES: Record<string, string> = {
  python:
    "Python (numpy): input curves are float32 arrays (NaN = missing) plus `depth`; " +
    "assign the output curve name, e.g.  vsh = np.clip((gr - 20) / 120, 0, 1)",
  rhai: "Rhai (legacy): evaluated once per depth sample; the expression's value is the output. Any NaN input yields NaN.",
};

/**
 * Right-hand side panel: Equation Editor (Rhai script editor + run, backed by the
 * `equations` / `computed_curves` tables) and Curve Catalog (auto-derived from the
 * database, in the spirit of IP's CPARMDEF).
 */
export class InspectorPanel {
  private equationTab: HTMLElement;
  private catalogTab: HTMLElement;
  public getSelectedWellId: (() => string | null) | null = null;

  private equations: EquationDef[] = [];
  private current: EquationDef = { ...BLANK_EQUATION };
  private editor: EditorView | null = null;
  /** Path of the Python the backend found; null = none; undefined = not asked yet. */
  private pythonPath: string | null | undefined = undefined;
  /** Last-loaded generic-store catalog for the selected well, kept so the filter box can
   *  re-render without refetching. */
  private genericEntries: GenericCurveCatalogEntry[] = [];
  /** P1-c: current computed curves with provenance/stats + the well's set versions. */
  private computedEntries: ComputedCatalogEntry[] = [];
  private logSets: LogSetEntry[] = [];
  private catalogFilter = "";
  private catalogSortKey = "name";
  private catalogSortAsc = true;

  constructor(root: HTMLElement) {
    const tabButtons = Array.from(root.querySelectorAll<HTMLButtonElement>(".tab-btn"));
    const tabContents = new Map<string, HTMLElement>(
      Array.from(root.querySelectorAll<HTMLElement>(".tab-content")).map((el) => [
        el.id.replace("tab-", ""),
        el,
      ]),
    );

    for (const btn of tabButtons) {
      btn.addEventListener("click", () => {
        const target = btn.dataset.tab!;
        for (const b of tabButtons) b.classList.toggle("active", b === btn);
        for (const [key, el] of tabContents) el.hidden = key !== target;
        if (target === "catalog") this.refreshCatalog();
      });
    }

    this.equationTab = tabContents.get("equation")!;
    this.catalogTab = tabContents.get("catalog")!;

    this.renderEquationEditor();
    this.renderLegacyCatalog([]);
    this.refreshEquationList();
    this.refreshCatalog();
  }

  private async refreshEquationList(): Promise<void> {
    try {
      this.equations = await listEquations();
    } catch (err) {
      console.error("Failed to load equations:", err);
      this.equations = [];
    }
    this.renderEquationEditor();
  }

  public async refreshCatalog(): Promise<void> {
    // Per selected well: generic store (imported curves), computed curves with
    // provenance/statistics, and the log-set version history (P1-c) render as one
    // searchable, sortable catalog. Fall back to the legacy standard+computed reference
    // when no well is selected or nothing well-scoped exists yet.
    const wellId = this.getSelectedWellId?.() ?? null;
    if (wellId) {
      const [generic, computed, sets] = await Promise.all([
        listGenericCurveCatalog(wellId).catch(() => [] as GenericCurveCatalogEntry[]),
        listComputedCatalog(wellId).catch(() => [] as ComputedCatalogEntry[]),
        listLogSets(wellId).catch(() => [] as LogSetEntry[]),
      ]);
      this.genericEntries = generic;
      this.computedEntries = computed;
      this.logSets = sets;
      if (generic.length > 0 || computed.length > 0 || sets.length > 0) {
        this.renderGenericCatalog();
        return;
      }
    }
    this.genericEntries = [];
    this.computedEntries = [];
    this.logSets = [];
    try {
      const entries = await listCurveCatalog();
      this.renderLegacyCatalog(entries);
    } catch (err) {
      console.error("Failed to load curve catalog:", err);
      this.renderLegacyCatalog([]);
    }
  }

  private renderEquationEditor(): void {
    const eq = this.current;
    this.editor?.destroy();
    this.equationTab.innerHTML = `
      <div class="eq-form">
        <div class="eq-note" id="eq-lang-note">${escapeHtml(LANGUAGE_NOTES[eq.language] ?? LANGUAGE_NOTES.rhai)}</div>

        <label class="field-label">Equation
          <select id="eq-picker">
            <option value="">— New equation —</option>
            ${this.equations
              .map((e) => `<option value="${e.equation_id}" ${e.equation_id === eq.equation_id ? "selected" : ""}>${escapeHtml(e.name)}</option>`)
              .join("")}
          </select>
        </label>

        <div class="field-grid two">
          <label class="field-label">Name
            <input id="eq-name" type="text" value="${escapeAttr(eq.name)}" placeholder="e.g. VSH_LINEAR" />
          </label>
          <label class="field-label">Description <span class="field-hint">optional</span>
            <input id="eq-description" type="text" value="${escapeAttr(eq.description ?? "")}" placeholder="what it computes" />
          </label>
        </div>

        <label class="field-label">Input curves <span class="field-hint">comma-separated</span>
          <input id="eq-inputs" type="text" value="${escapeAttr(eq.input_curves.join(", "))}" placeholder="GR, RHOB" />
        </label>

        <div class="field-grid three">
          <label class="field-label">Output curve
            <input id="eq-output" type="text" value="${escapeAttr(eq.output_curve)}" placeholder="VSH" />
          </label>
          <label class="field-label">Units
            <input id="eq-units" type="text" value="${escapeAttr(eq.output_units ?? "")}" placeholder="V/V" />
          </label>
          <label class="field-label">Language
            <select id="eq-language">
              <option value="python" ${eq.language === "python" ? "selected" : ""}>Python (numpy)</option>
              <option value="rhai" ${eq.language !== "python" ? "selected" : ""}>Rhai (legacy)</option>
            </select>
          </label>
        </div>

        <label class="field-label">Script</label>
        <div id="eq-script-host" class="eq-editor"></div>

        <div class="eq-footer">
          <label class="field-checkbox">
            <input id="eq-all-wells" type="checkbox" />
            Apply to all wells
          </label>
          <div class="eq-actions">
            <button id="eq-save" class="btn">Save</button>
            <button id="eq-run" class="btn btn-accent">Run</button>
          </div>
        </div>

        <div id="eq-status" class="eq-status" hidden></div>
      </div>
    `;

    // CodeMirror replaces the old plain textarea (python syntax highlighting when apt).
    const scriptHost = this.equationTab.querySelector<HTMLElement>("#eq-script-host")!;
    this.editor = new EditorView({
      doc: eq.script,
      parent: scriptHost,
      extensions: [basicSetup, ...(eq.language === "python" ? [python()] : []), EditorView.lineWrapping],
    });

    const picker = this.equationTab.querySelector<HTMLSelectElement>("#eq-picker")!;
    picker.addEventListener("change", () => {
      const found = this.equations.find((e) => e.equation_id === picker.value);
      this.current = found ? { ...found } : { ...BLANK_EQUATION };
      this.renderEquationEditor();
    });

    const langSel = this.equationTab.querySelector<HTMLSelectElement>("#eq-language")!;
    langSel.addEventListener("change", () => {
      this.readFormIntoCurrent();
      this.current.language = langSel.value;
      this.renderEquationEditor();
    });

    this.equationTab.querySelector<HTMLButtonElement>("#eq-save")!.addEventListener("click", () => this.handleSave());
    this.equationTab.querySelector<HTMLButtonElement>("#eq-run")!.addEventListener("click", () => this.handleRun());

    // Surface which Python the engine found (or that none was) next to the language note.
    if (eq.language === "python") void this.showPythonStatus();
  }

  private async showPythonStatus(): Promise<void> {
    if (this.pythonPath === undefined) {
      try {
        this.pythonPath = await pythonStatus();
      } catch {
        return; // no backend (browser preview) — say nothing
      }
    }
    const note = this.equationTab.querySelector<HTMLElement>("#eq-lang-note");
    if (!note) return;
    note.textContent +=
      this.pythonPath === null
        ? "  ⚠ No Python with numpy found — install Python 3.10+ & numpy, or set ARSHILLA_PYTHON."
        : `  (engine: ${this.pythonPath})`;
  }

  private readFormIntoCurrent(): void {
    const val = (id: string) => this.equationTab.querySelector<HTMLInputElement | HTMLSelectElement>(id)!.value;
    this.current = {
      ...this.current,
      name: val("#eq-name").trim(),
      description: val("#eq-description").trim() || null,
      input_curves: val("#eq-inputs")
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean),
      output_curve: val("#eq-output").trim(),
      output_units: val("#eq-units").trim() || null,
      language: val("#eq-language"),
      script: this.editor?.state.doc.toString() ?? this.current.script,
    };
  }

  private setStatus(text: string): void {
    const el = this.equationTab.querySelector<HTMLElement>("#eq-status");
    if (el) {
      el.textContent = text;
      el.hidden = text === "";
      el.classList.toggle("error", /fail|error|required|select/i.test(text));
    }
  }

  private async handleSave(): Promise<void> {
    this.readFormIntoCurrent();
    if (!this.current.name || !this.current.output_curve || !this.current.script) {
      this.setStatus("Name, output curve, and script are required.");
      return;
    }
    try {
      const id = await saveEquation(this.current);
      this.current.equation_id = id;
      this.setStatus(`Saved "${this.current.name}".`);
      await this.refreshEquationList();
    } catch (err) {
      console.error("Failed to save equation:", err);
      this.setStatus(`Save failed: ${err}`);
    }
  }

  private async handleRun(): Promise<void> {
    this.readFormIntoCurrent();
    if (!this.current.equation_id) {
      this.setStatus("Save the equation before running it.");
      return;
    }

    const applyAll = this.equationTab.querySelector<HTMLInputElement>("#eq-all-wells")!.checked;
    let wellIds: string[];
    if (applyAll) {
      try {
        wellIds = filterByActiveGroup(await listWells()).map((w) => w.well_id);
      } catch (err) {
        this.setStatus(`Failed to list wells: ${err}`);
        return;
      }
    } else {
      const selected = this.getSelectedWellId?.();
      if (!selected) {
        this.setStatus("Select a well in the object tree first, or check 'Apply to all wells'.");
        return;
      }
      wellIds = [selected];
    }

    if (wellIds.length === 0) {
      this.setStatus("No wells to run against.");
      return;
    }

    this.setStatus(`Running on ${wellIds.length} well(s)...`);
    try {
      const results = await runEquation(this.current.equation_id, wellIds);
      this.setStatus(summarizeRun(results));
      recordProcess("Equation", `Ran "${this.current.name}" on ${wellIds.length} well(s)`);
      await this.refreshCatalog();
      bumpDataVersion(); // refresh other open panels (log views, plots) — not just this catalog
    } catch (err) {
      console.error("Equation run failed:", err);
      this.setStatus(`Run failed: ${err}`);
    }
  }

  /** Legacy catalog (standard + computed, no well context) — shown when no well is
   *  selected or the generic store hasn't been populated for the selected well yet. */
  private renderLegacyCatalog(entries: { name: string; units: string | null; source: string }[]): void {
    const rows = entries
      .map(
        (e) =>
          `<tr><td>${escapeHtml(e.name)}</td><td>${escapeHtml(e.units ?? "")}</td><td>—</td><td>—</td><td>${escapeHtml(e.source)}</td><td>—</td></tr>`,
      )
      .join("");

    this.catalogTab.innerHTML = `
      <p class="placeholder-note">Select a well to see its full curve catalog (all sets, families and units). Showing the standard + computed reference.</p>
      <table class="catalog-table">
        <thead><tr><th>Mnemonic</th><th>Unit</th><th>Family</th><th>Set</th><th>Source</th><th>Samples</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="6" class="placeholder-note">No curves yet</td></tr>`}</tbody>
      </table>
    `;
  }

  /** The selected well's full catalog: imported curves (generic store) merged with
   *  computed curves carrying per-curve provenance (set/version/module/when) and basic
   *  statistics; live text search + click-to-sort headers; plus the log-set version
   *  history with Restore / Delete (P1-c "never overwrite"). */
  private renderGenericCatalog(): void {
    type Row = {
      name: string;
      runNo: number | null;
      unit: string;
      family: string;
      set: string;
      ver: number | null;
      source: string;
      when: string;
      samples: number;
      min: number | null;
      max: number | null;
      mean: number | null;
    };
    const rows: Row[] = [
      ...this.genericEntries.map((e) => ({
        name: e.mnemonic,
        runNo: e.run_no,
        unit: e.unit ?? "",
        family: e.family ?? "",
        set: e.set_name,
        ver: null,
        source: e.source ?? "",
        when: "",
        samples: e.n_samples,
        min: null,
        max: null,
        mean: null,
      })),
      ...this.computedEntries.map((e) => ({
        name: e.curve_name,
        runNo: null,
        unit: "",
        family: "",
        set: e.set_name ?? "—",
        ver: e.version,
        source: e.module ?? "",
        when: e.created_at ?? "",
        samples: e.n_samples,
        min: e.min,
        max: e.max,
        mean: e.mean,
      })),
    ];

    const filter = this.catalogFilter.trim().toLowerCase();
    const shown = rows.filter(
      (r) =>
        filter === "" ||
        [r.name, r.family, r.unit, r.set, r.source, r.when, r.ver != null ? `v${r.ver}` : ""].some((f) =>
          f.toLowerCase().includes(filter),
        ),
    );

    const key = this.catalogSortKey as keyof Row;
    const dir = this.catalogSortAsc ? 1 : -1;
    shown.sort((a, b) => {
      const av = a[key];
      const bv = b[key];
      if (typeof av === "number" || typeof bv === "number") {
        return (Number(av ?? Number.NEGATIVE_INFINITY) - Number(bv ?? Number.NEGATIVE_INFINITY)) * dir;
      }
      return String(av ?? "").localeCompare(String(bv ?? "")) * dir;
    });

    const fmt = (v: number | null) => (v == null ? "—" : Math.abs(v) >= 100 ? v.toFixed(1) : v.toFixed(3));
    const bodyRows = shown
      .map(
        (r) =>
          `<tr><td>${escapeHtml(r.name)}${r.runNo != null ? `<span class="catalog-run"> · run ${r.runNo}</span>` : ""}</td>` +
          `<td>${escapeHtml(r.unit)}</td>` +
          `<td>${escapeHtml(r.family || "—")}</td>` +
          `<td>${escapeHtml(r.set)}${r.ver != null ? `<span class="catalog-run"> v${r.ver}</span>` : ""}</td>` +
          `<td>${escapeHtml(r.source)}</td>` +
          `<td>${escapeHtml(r.when || "—")}</td>` +
          `<td>${r.samples}</td>` +
          `<td>${fmt(r.min)}</td><td>${fmt(r.max)}</td><td>${fmt(r.mean)}</td></tr>`,
      )
      .join("");

    const cols: [string, string][] = [
      ["name", "Mnemonic"],
      ["unit", "Unit"],
      ["family", "Family"],
      ["set", "Set"],
      ["source", "Module / Source"],
      ["when", "When"],
      ["samples", "n"],
      ["min", "Min"],
      ["max", "Max"],
      ["mean", "Mean"],
    ];
    const header = cols
      .map(
        ([k, label]) =>
          `<th class="catalog-sortable" data-sort="${k}">${label}${
            this.catalogSortKey === k ? (this.catalogSortAsc ? " ▲" : " ▼") : ""
          }</th>`,
      )
      .join("");

    // Log-set version history (newest first per set); restore any version, prune old ones.
    const setRows = this.logSets
      .map((s) => {
        const tip = escapeAttr(
          `params: ${s.params_json ?? "—"}\ninputs: ${s.inputs_json ?? "—"}\ncurves: ${s.curve_names.join(", ") || "—"}`,
        );
        return (
          `<div class="catalog-set-row" title="${tip}">` +
          `<span class="catalog-set-badge${s.is_current ? " current" : ""}">${escapeHtml(s.set_name)} v${s.version}</span>` +
          `<span class="catalog-set-info">${escapeHtml(s.module)} · ${escapeHtml(s.created_at)} · ${escapeHtml(
            s.curve_names.join(", ") || "(no curves)",
          )}${s.is_current ? " · current" : ""}</span>` +
          `<button class="catalog-set-btn" data-restore="${escapeAttr(s.set_id)}">Restore</button>` +
          `<button class="catalog-set-btn danger" data-del="${escapeAttr(s.set_id)}">Delete</button>` +
          `</div>`
        );
      })
      .join("");

    this.catalogTab.innerHTML = `
      <input id="catalog-filter" class="catalog-filter" type="search" placeholder="Search mnemonic, cons, module, unit, date…" value="${escapeAttr(this.catalogFilter)}" />
      <table class="catalog-table">
        <thead><tr>${header}</tr></thead>
        <tbody>${bodyRows || `<tr><td colspan="10" class="placeholder-note">No curves match "${escapeHtml(this.catalogFilter)}"</td></tr>`}</tbody>
      </table>
      <div class="catalog-sets">
        <div class="catalog-sets-title">Constellations — every run is kept as a version (nothing is overwritten)</div>
        ${setRows || `<div class="placeholder-note">No versioned runs yet — run any module and its outputs appear here as version 1.</div>`}
      </div>
    `;

    const filterInput = this.catalogTab.querySelector<HTMLInputElement>("#catalog-filter");
    if (filterInput) {
      filterInput.addEventListener("input", () => {
        this.catalogFilter = filterInput.value;
        this.renderGenericCatalog();
        // Keep focus + caret at end after the re-render.
        const again = this.catalogTab.querySelector<HTMLInputElement>("#catalog-filter");
        again?.focus();
        again?.setSelectionRange(again.value.length, again.value.length);
      });
    }
    for (const th of this.catalogTab.querySelectorAll<HTMLElement>("th.catalog-sortable")) {
      th.addEventListener("click", () => {
        const k = th.dataset.sort!;
        if (this.catalogSortKey === k) this.catalogSortAsc = !this.catalogSortAsc;
        else {
          this.catalogSortKey = k;
          this.catalogSortAsc = true;
        }
        this.renderGenericCatalog();
      });
    }
    for (const btn of this.catalogTab.querySelectorAll<HTMLButtonElement>("[data-restore]")) {
      btn.addEventListener("click", async () => {
        btn.disabled = true;
        try {
          const n = await restoreLogSet(btn.dataset.restore!);
          globalStatus(`Version restored (${n} samples back in the current curves)`);
          recordProcess("Constellation", `Restored a curve version (${n} samples)`);
          bumpDataVersion(); // every open panel (log views, plots, this catalog) refreshes
        } catch (err) {
          globalStatus(`Restore failed: ${err}`);
          btn.disabled = false;
        }
      });
    }
    for (const btn of this.catalogTab.querySelectorAll<HTMLButtonElement>("[data-del]")) {
      // Two-click confirm: deleting history is allowed but must be deliberate.
      btn.addEventListener("click", async () => {
        if (!btn.dataset.armed) {
          btn.dataset.armed = "1";
          btn.textContent = "Confirm delete";
          window.setTimeout(() => {
            btn.textContent = "Delete";
            delete btn.dataset.armed;
          }, 2500);
          return;
        }
        btn.disabled = true;
        try {
          await deleteLogSet(btn.dataset.del!);
          globalStatus("Constellation version deleted (current curve values kept)");
          recordProcess("Constellation", "Deleted a constellation version");
          void this.refreshCatalog();
        } catch (err) {
          globalStatus(`Delete failed: ${err}`);
          btn.disabled = false;
        }
      });
    }
  }
}

function summarizeRun(results: EquationRunResult[]): string {
  const ok = results.filter((r) => !r.error);
  const failed = results.filter((r) => r.error);
  const totalRows = ok.reduce((sum, r) => sum + r.rows_written, 0);
  let text = `${ok.length}/${results.length} well(s) succeeded, ${totalRows} rows written.`;
  if (failed.length > 0) {
    text += ` Errors: ${failed.map((r) => `${r.well_id}: ${r.error}`).join("; ")}`;
  }
  return text;
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function escapeAttr(text: string): string {
  return escapeHtml(text).replace(/"/g, "&quot;");
}
