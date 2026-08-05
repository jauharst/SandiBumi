// Type-only: the CodeMirror runtime is dynamic-imported in `renderEquationEditor` so the whole
// CM6 stack stays out of the eager startup bundle. A static import here pulled 461.3 kB — 41% of
// the main chunk — into every launch, for a panel most sessions never open, and it also defeated
// vegaPanel's own dynamic import (once CM is in the eager chunk, deferring it there buys nothing).
import type { EditorView } from "codemirror";
import { bumpDataVersion, filterByActiveGroup, setStatus as globalStatus } from "../state";
import { recordProcess } from "../processLog";
import {
  saveEquation,
  listEquations,
  runEquation,
  listCurveCatalog,
  listComputedCatalog,
  listGenericCurveCatalog,
  deleteGenericCurve,
  promoteGenericCurve,
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
  type PythonStatus,
} from "../ipc";
import { escapeAttr, escapeHtml } from "./safeDom";

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
    "assign the output curve name, e.g.  vsh = np.clip((gr - 20) / 120, 0, 1). " +
    "If scipy is installed, `signal`, `interpolate`, `optimize`, `stats` and `ndimage` are " +
    "also bound — e.g.  grs = signal.savgol_filter(gr, 11, 2)",
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
  /** Bumped on every `renderEquationEditor`, so an in-flight async CodeMirror mount that is
   *  superseded by a newer render drops itself instead of attaching to a replaced host. */
  private editorGen = 0;
  /** Set by `dispose()`. The CodeMirror mount is async, so a panel closed inside that window would
   *  otherwise create a brand-new EditorView — and its window/document listeners — AFTER the panel
   *  is gone, which is the very leak dispose() exists to prevent. */
  private disposed = false;
  /** Interpreter + optional-package status from the backend; undefined = not asked yet. */
  private pythonInfo: PythonStatus | undefined = undefined;
  /** Last-loaded generic-store catalog for the selected well, kept so the filter box can
   *  re-render without refetching. */
  private genericEntries: GenericCurveCatalogEntry[] = [];
  /** P1-c: current computed curves with provenance/stats + the well's set versions. */
  private computedEntries: ComputedCatalogEntry[] = [];
  private logSets: LogSetEntry[] = [];
  private catalogFilter = "";
  private catalogSortKey = "name";
  private catalogSortAsc = true;
  /** Kept so `focusCatalog` can drive the tab buttons from outside. */
  private root: HTMLElement;

  constructor(root: HTMLElement) {
    this.root = root;
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

  /** Releases the CodeMirror view when the panel closes. Not optional bookkeeping: an EditorView
   *  registers four listeners rooted at `window`/`document` — `resize`, `scroll`, `beforeprint` (or
   *  a matchMedia `change`) and `selectionchange` (@codemirror/view DOMObserver.addWindowListeners,
   *  index.js:7480-7492) — and the ONLY path that removes them is `EditorView.destroy()`
   *  (7513→7521). Because `window` and `document` are GC roots, an undestroyed view keeps itself,
   *  its history/autocomplete state, the python parse tree and the detached editor DOM reachable for
   *  the app's life, and every caret move anywhere in the app still dispatches into it. The Inspector
   *  is a CLOSABLE panel and `dock.clear()` runs on every session switch and workspace reset, so this
   *  is per-cycle growth, not a bounded one-off. `renderEquationEditor` already recycles the view on
   *  internal re-renders — this covers the last one, which nothing else does. Same shape as
   *  vegaPanel.ts's `editor?.destroy()` and the dbInspector/history wiring at workspace.ts:419/428. */
  public dispose(): void {
    this.disposed = true;
    this.editor?.destroy();
    this.editor = null;
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

  /** Switches to the Curve Catalog tab and filters it to `filter` (a mnemonic, usually).
   *  Entry point for the Wells-pane right-click — landing on the row the user meant beats
   *  opening an unfiltered catalog of every curve in the well. */
  public focusCatalog(filter: string): void {
    this.catalogFilter = filter;
    for (const b of this.root.querySelectorAll<HTMLButtonElement>(".tab-btn")) {
      b.classList.toggle("active", b.dataset.tab === "catalog");
    }
    this.equationTab.hidden = true;
    this.catalogTab.hidden = false;
    void this.refreshCatalog();
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
    // Null it, don't just destroy it. `destroy()` tears down the DOM but leaves `view.state`
    // readable, so a destroyed view still answers `readFormIntoCurrent` with the PREVIOUS
    // equation's text. That was harmless while the mount below was synchronous — the field was
    // reassigned on the next line — but it is now awaited, and saving in that window would write
    // the old equation's script into the newly-selected one.
    this.editor?.destroy();
    this.editor = null;
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

    // CodeMirror replaces the old plain textarea (python syntax highlighting when apt). It mounts
    // asynchronously because the module is dynamic-imported, so between here and the mount
    // `this.editor` is null (cleared above) and `readFormIntoCurrent` falls back to
    // `this.current.script` — which is this equation's own text, so a Save in that window keeps
    // the script rather than blanking it or writing the previously-open equation's. The language
    // mode is fetched only for python, so a Rhai-only session never pays for the lezer parser.
    const scriptHost = this.equationTab.querySelector<HTMLElement>("#eq-script-host")!;
    const gen = ++this.editorGen;
    void (async () => {
      const [cm, lang] = await Promise.all([
        import("codemirror"),
        eq.language === "python" ? import("@codemirror/lang-python") : Promise.resolve(null),
      ]);
      // A re-render (equation picked, language switched) may have landed while we awaited — that
      // render owns the host now, so this one must not mount into a detached or replaced node.
      if (this.disposed || gen !== this.editorGen || !scriptHost.isConnected) return;
      this.editor = new cm.EditorView({
        doc: eq.script,
        parent: scriptHost,
        extensions: [cm.basicSetup, ...(lang ? [lang.python()] : []), cm.EditorView.lineWrapping],
      });
    })();

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
    if (this.pythonInfo === undefined) {
      try {
        this.pythonInfo = await pythonStatus();
      } catch {
        return; // no backend (browser preview) — say nothing
      }
    }
    const note = this.equationTab.querySelector<HTMLElement>("#eq-lang-note");
    if (!note) return;
    if (this.pythonInfo.path === null) {
      note.textContent += "  ⚠ No Python with numpy found — install Python 3.10+ & numpy, or set SANDIBUMI_PYTHON.";
      return;
    }
    // scipy is optional, so its absence is a NOTE, not a warning — the engine is fully usable
    // without it. Say so while the script is being written rather than after it is queued.
    note.textContent +=
      this.pythonInfo.scipy === null
        ? `  (engine: ${this.pythonInfo.path} · no scipy — install it for signal/interpolate/optimize/stats)`
        : `  (engine: ${this.pythonInfo.path} · scipy ${this.pythonInfo.scipy})`;
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
      curveId: string | null; // generic-store rows only (for promote/delete)
      pinned: boolean;
      collision: boolean; // >1 generic curve shares this (set, mnemonic)
      winner: boolean; // this row is the current resolver winner in its collision group
      // Resolution actually comes from a HIGHER-priority store than the generic RAW one, so
      // promote/pin has no effect on what modules/plots read (fetch_curve_frame resolves
      // standard column → computed → generic). Neutralises the promote lie for those cases.
      overriddenBy: "log" | "computed" | null;
    };

    // Detect same-mnemonic shadowing within the generic store (a DLIS import can collide with a
    // LAS curve of the same mnemonic) and mark the resolver's current winner (pinned, else the
    // NULL/lowest run_no — mirroring the backend `pinned DESC, run_no NULLS FIRST`).
    const groups = new Map<string, GenericCurveCatalogEntry[]>();
    for (const e of this.genericEntries) {
      const k = `${e.set_name} ${e.mnemonic.toUpperCase()}`;
      const arr = groups.get(k);
      if (arr) arr.push(e);
      else groups.set(k, [e]);
    }
    const winnerId = new Map<string, string>();
    for (const [k, es] of groups) {
      if (es.length < 2) continue;
      const win =
        es.find((e) => e.pinned) ??
        [...es].sort((a, b) => {
          if (a.run_no == null && b.run_no != null) return -1;
          if (a.run_no != null && b.run_no == null) return 1;
          if (a.run_no !== b.run_no) return (a.run_no ?? 0) - (b.run_no ?? 0);
          return a.curve_id.localeCompare(b.curve_id); // final key mirrors the resolver's curve_id tiebreak
        })[0];
      winnerId.set(k, win.curve_id);
    }

    // A generic RAW curve only governs resolution when NO higher-priority store holds its
    // mnemonic: fetch_curve_frame resolves standard column → computed → generic. When a standard
    // column is populated (its migration mirror row is present) or a computed curve of the same
    // name exists, promote/pin on the RAW row is inert — so the badge must not claim it "resolves"
    // and Promote must be disabled, or the UI would assert a win the resolver never honours.
    const STANDARD_MNEMONICS = new Set(["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"]);
    const computedNames = new Set(this.computedEntries.map((e) => e.curve_name.toUpperCase()));
    const overrideFor = (setName: string, mnemUpper: string): "log" | "computed" | null => {
      if (setName !== "RAW") return null; // the generic resolver only reads the RAW set
      const es = groups.get(`${setName} ${mnemUpper}`) ?? [];
      // The boot migration inserts a 'standard_curves migration' RAW row iff the standard column
      // has data — a precise per-well signal that the standard column governs this mnemonic.
      if (STANDARD_MNEMONICS.has(mnemUpper) && es.some((e) => (e.source ?? "").includes("standard_curves migration"))) {
        return "log";
      }
      if (computedNames.has(mnemUpper)) return "computed";
      return null;
    };

    const rows: Row[] = [
      ...this.genericEntries.map((e) => {
        const mnemUpper = e.mnemonic.toUpperCase();
        const gk = `${e.set_name} ${mnemUpper}`;
        const collision = (groups.get(gk)?.length ?? 0) > 1;
        return {
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
          curveId: e.curve_id,
          pinned: e.pinned,
          collision,
          winner: collision && winnerId.get(gk) === e.curve_id,
          overriddenBy: overrideFor(e.set_name, mnemUpper),
        };
      }),
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
        curveId: null,
        pinned: false,
        collision: false,
        winner: false,
        overriddenBy: null,
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
    // When resolution is served by a higher-priority store, the generic-store "resolves/shadowed"
    // badges would lie — show a neutral "served by …" note instead, and suppress pinned (inert here).
    const overrideNote = (r: Row) =>
      r.overriddenBy === "log"
        ? ` <span class="catalog-badge muted" title="fetch_curve_frame reads the standard log column for this mnemonic; the RAW copy does not resolve">served by log</span>`
        : r.overriddenBy === "computed"
          ? ` <span class="catalog-badge muted" title="a computed curve of this name resolves before the RAW store">served by computed</span>`
          : "";
    const badges = (r: Row) =>
      r.overriddenBy != null
        ? overrideNote(r)
        : (r.collision
            ? r.winner
              ? ` <span class="catalog-badge win">resolves</span>`
              : ` <span class="catalog-badge shadow">shadowed</span>`
            : "") + (r.pinned ? ` <span class="catalog-badge pin">pinned</span>` : "");
    // Promote is inert (and would falsely claim victory) when a higher-priority store already
    // resolves the mnemonic, or when this row is already the generic-store winner.
    const promoteBlock = (r: Row): string =>
      r.overriddenBy === "log"
        ? " disabled title=\"resolution comes from the standard log column — promoting has no effect\""
        : r.overriddenBy === "computed"
          ? " disabled title=\"a computed curve of this name resolves first — promoting has no effect\""
          : r.winner
            ? " disabled title=\"already the resolved curve\""
            : "";
    const actions = (r: Row) =>
      r.curveId == null
        ? ""
        : `<button class="catalog-set-btn" data-promote="${escapeAttr(r.curveId)}"${promoteBlock(r)}>Promote</button>` +
          `<button class="catalog-set-btn danger" data-del-curve="${escapeAttr(r.curveId)}">Delete</button>`;
    const bodyRows = shown
      .map(
        (r) =>
          `<tr><td>${escapeHtml(r.name)}${r.runNo != null ? `<span class="catalog-run"> · run ${r.runNo}</span>` : ""}${badges(r)}</td>` +
          `<td>${escapeHtml(r.unit)}</td>` +
          `<td>${escapeHtml(r.family || "—")}</td>` +
          `<td>${escapeHtml(r.set)}${r.ver != null ? `<span class="catalog-run"> v${r.ver}</span>` : ""}</td>` +
          `<td>${escapeHtml(r.source)}</td>` +
          `<td>${escapeHtml(r.when || "—")}</td>` +
          `<td>${r.samples}</td>` +
          `<td>${fmt(r.min)}</td><td>${fmt(r.max)}</td><td>${fmt(r.mean)}</td>` +
          `<td class="catalog-actions">${actions(r)}</td></tr>`,
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
    const header =
      cols
        .map(
          ([k, label]) =>
            `<th class="catalog-sortable" data-sort="${k}">${label}${
              this.catalogSortKey === k ? (this.catalogSortAsc ? " ▲" : " ▼") : ""
            }</th>`,
        )
        .join("") + `<th>Actions</th>`;

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
      <input id="catalog-filter" class="catalog-filter" type="search" placeholder="Search mnemonic, log set, module, unit, date…" value="${escapeAttr(this.catalogFilter)}" />
      <table class="catalog-table">
        <thead><tr>${header}</tr></thead>
        <tbody>${bodyRows || `<tr><td colspan="11" class="placeholder-note">No curves match "${escapeHtml(this.catalogFilter)}"</td></tr>`}</tbody>
      </table>
      <div class="catalog-sets">
        <div class="catalog-sets-title">Log sets — every run is kept as a version (nothing is overwritten)</div>
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
          recordProcess("Log set", `Restored a curve version (${n} samples)`);
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
          globalStatus("Log-set version deleted (current curve values kept)");
          recordProcess("Log set", "Deleted a log-set version");
          void this.refreshCatalog();
        } catch (err) {
          globalStatus(`Delete failed: ${err}`);
          btn.disabled = false;
        }
      });
    }
    // Promote a generic curve so it wins its mnemonic (resolve DLIS/LAS shadowing).
    for (const btn of this.catalogTab.querySelectorAll<HTMLButtonElement>("[data-promote]")) {
      btn.addEventListener("click", async () => {
        btn.disabled = true;
        try {
          await promoteGenericCurve(btn.dataset.promote!);
          globalStatus("Curve promoted — it now wins its mnemonic");
          recordProcess("Curve", "Promoted a generic curve (resolved a mnemonic shadow)");
          bumpDataVersion(); // log views / plots / modules re-resolve immediately
          void this.refreshCatalog();
        } catch (err) {
          globalStatus(`Promote failed: ${err}`);
          btn.disabled = false;
        }
      });
    }
    // Delete an imported (generic-store) curve outright — two-click confirm, irreversible.
    for (const btn of this.catalogTab.querySelectorAll<HTMLButtonElement>("[data-del-curve]")) {
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
          await deleteGenericCurve(btn.dataset.delCurve!);
          globalStatus("Imported curve deleted");
          recordProcess("Curve", "Deleted a generic curve");
          bumpDataVersion();
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
  // A well that succeeded with holes in its curve. Reported separately from the errors because it
  // is a different thing to know: the curve is there and usable, and the script threw on part of
  // it. Silence here is what made a half-failed script indistinguishable from absent inputs.
  const warned = ok.filter((r) => r.note);
  if (warned.length > 0) {
    text += ` Warnings: ${warned.map((r) => `${r.well_id}: ${r.note}`).join("; ")}`;
  }
  return text;
}

