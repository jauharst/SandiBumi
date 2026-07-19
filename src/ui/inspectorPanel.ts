import { basicSetup, EditorView } from "codemirror";
import { python } from "@codemirror/lang-python";
import { filterByActiveGroup } from "../state";
import {
  saveEquation,
  listEquations,
  runEquation,
  listCurveCatalog,
  listGenericCurveCatalog,
  listWells,
  pythonStatus,
  type EquationDef,
  type EquationRunResult,
  type GenericCurveCatalogEntry,
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
  private catalogFilter = "";

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
    // Phase 6c: prefer the generic curve store (family/set/unit, per selected well). It
    // holds every imported curve — PEF, CALI, multiple runs, DLIS channels — not just the
    // fixed 6. Fall back to the legacy standard+computed catalog when no well is selected
    // or the generic store is empty for it (e.g. a fresh project mid-migration).
    const wellId = this.getSelectedWellId?.() ?? null;
    if (wellId) {
      try {
        this.genericEntries = await listGenericCurveCatalog(wellId);
        if (this.genericEntries.length > 0) {
          this.renderGenericCatalog();
          return;
        }
      } catch (err) {
        console.error("Failed to load generic curve catalog:", err);
      }
    }
    this.genericEntries = [];
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
      await this.refreshCatalog();
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

  /** Phase 6c generic-store catalog for the selected well: every curve with its family,
   *  set (RAW/EDIT/FINAL), unit, source and sample count, with a live text filter. */
  private renderGenericCatalog(): void {
    const filter = this.catalogFilter.trim().toLowerCase();
    const matches = (e: GenericCurveCatalogEntry) =>
      filter === "" ||
      [e.mnemonic, e.family ?? "", e.unit ?? "", e.set_name, e.source ?? ""]
        .some((f) => f.toLowerCase().includes(filter));

    const shown = this.genericEntries.filter(matches);
    const rows = shown
      .map(
        (e) =>
          `<tr><td>${escapeHtml(e.mnemonic)}${e.run_no != null ? `<span class="catalog-run"> · run ${e.run_no}</span>` : ""}</td>` +
          `<td>${escapeHtml(e.unit ?? "")}</td>` +
          `<td>${escapeHtml(e.family ?? "—")}</td>` +
          `<td>${escapeHtml(e.set_name)}</td>` +
          `<td>${escapeHtml(e.source ?? "")}</td>` +
          `<td>${e.n_samples}</td></tr>`,
      )
      .join("");

    this.catalogTab.innerHTML = `
      <p class="placeholder-note">Generic curve store — ${this.genericEntries.length} curve(s) across all sets. Every imported curve (LAS, DLIS, computed) appears here.</p>
      <input id="catalog-filter" class="catalog-filter" type="search" placeholder="Filter by mnemonic, family, unit, set…" value="${escapeAttr(this.catalogFilter)}" />
      <table class="catalog-table">
        <thead><tr><th>Mnemonic</th><th>Unit</th><th>Family</th><th>Set</th><th>Source</th><th>Samples</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="6" class="placeholder-note">No curves match "${escapeHtml(this.catalogFilter)}"</td></tr>`}</tbody>
      </table>
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
