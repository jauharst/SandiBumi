import { open, save } from "@tauri-apps/plugin-dialog";
import {
  exportLas,
  importCoreCsv,
  importDeviationCsv,
  importScalCsv,
  importDlisFile,
  importLasFiles,
  deleteDocument,
  listDocuments,
  listLayouts,
  listModules,
  saveDocument,
  saveProjectAs,
  shiftCoreData,
  updateWellField,
  type Layout,
  type ModuleSpec,
} from "../ipc";
import { appState, bumpThemeVersion, setStatus } from "../state";
import { nextRedoLabel, nextUndoLabel, onUndoChange, pushUndo, redo, redoDepth, undo, undoDepth } from "../undo";
import { recordProcess } from "../processLog";
import { getTheme, setTheme, type ThemeChoice } from "../theme";
import { getLocale, setLocale, type Locale } from "../i18n";
import type { SessionSnapshot, Workspace } from "./workspace";
import { formRow, openModal } from "./modal";
import { openModuleDialog } from "./moduleDialog";
import { openCompositeDialog } from "./compositeDialog";
import { openReportDialog } from "./reportDialog";
import { openZonesDialog } from "./zonesDialog";
import { openSummaryDialog } from "./summaryDialog";
import { openWorkflowDialog } from "./workflowDialog";
import { openMonteCarloDialog } from "./monteCarloDialog";
import { openMlDialog } from "./mlDialog";
import { openMultiminDialog } from "./multiminDialog";

interface RibbonMenuItem {
  label: string;
  doc: string;
  onPick: () => void;
}

/** An Office-style dropdown ribbon button: large icon + label + ▾, opening a menu of
 *  method items below it. */
function buildRibbonDropdown(label: string, iconPath: string, items: RibbonMenuItem[]): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "ribbon-dropdown";

  const button = document.createElement("button");
  button.className = "ribbon-btn ribbon-dropdown-btn";
  button.innerHTML = `
    <svg class="ribbon-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5"
         stroke-linecap="round" stroke-linejoin="round"><path d="${iconPath}"/></svg>
    <span class="ribbon-label">${label} <span class="ribbon-caret">▾</span></span>`;

  const menu = document.createElement("div");
  menu.className = "ribbon-menu";
  menu.hidden = true;
  for (const item of items) {
    const entry = document.createElement("button");
    entry.className = "ribbon-menu-item";
    entry.textContent = item.label;
    entry.title = item.doc;
    entry.addEventListener("click", () => {
      menu.hidden = true;
      item.onPick();
    });
    menu.appendChild(entry);
  }

  button.addEventListener("click", () => {
    const wasOpen = !menu.hidden;
    for (const m of document.querySelectorAll<HTMLElement>(".ribbon-menu:not([hidden])")) m.hidden = true;
    menu.hidden = wasOpen;
  });

  wrap.appendChild(button);
  wrap.appendChild(menu);
  return wrap;
}

/** The main ribbon (Project | Data | Petrophysics | Plot | View). Talks to the docking
 *  workspace directly: panel-opening actions create dock panels, view actions target the
 *  active log view. */
export class Ribbon {
  private layouts: Layout[] = [];

  constructor(root: HTMLElement, private workspace: Workspace) {
    this.attachTabs(root);

    const q = <T extends HTMLElement>(sel: string) => root.querySelector<T>(sel);

    // --- Quick access toolbar (top-left, outside the ribbon tabs) ---
    const undoBtn = q<HTMLButtonElement>("#qat-undo");
    const redoBtn = q<HTMLButtonElement>("#qat-redo");
    undoBtn?.addEventListener("click", () => {
      void undo().then((label) => setStatus(label ? `Undo: ${label}` : "Nothing to undo"));
    });
    redoBtn?.addEventListener("click", () => {
      void redo().then((label) => setStatus(label ? `Redo: ${label}` : "Nothing to redo"));
    });
    // Enable/disable + tooltips track the stacks live.
    onUndoChange(() => {
      if (undoBtn) {
        undoBtn.disabled = undoDepth() === 0;
        const l = nextUndoLabel();
        undoBtn.title = l ? `Undo ${l} (Ctrl+Z)` : "Undo (Ctrl+Z)";
      }
      if (redoBtn) {
        redoBtn.disabled = redoDepth() === 0;
        const l = nextRedoLabel();
        redoBtn.title = l ? `Redo ${l} (Ctrl+Y)` : "Redo (Ctrl+Y)";
      }
    });
    q<HTMLButtonElement>("#qat-save")?.addEventListener("click", () => void this.handleSaveProject());
    q<HTMLButtonElement>("#qat-save-session")?.addEventListener("click", () => this.handleSaveSession());
    q<HTMLButtonElement>("#qat-open-session")?.addEventListener("click", () => void this.handleOpenSession());
    q<HTMLButtonElement>("#qat-history")?.addEventListener("click", () => workspace.openHistory());

    // --- Project ---
    q<HTMLButtonElement>("#save-project-btn")?.addEventListener("click", () => void this.handleSaveProject());
    const themeSelect = q<HTMLSelectElement>("#theme-select");
    if (themeSelect) {
      themeSelect.value = getTheme();
      themeSelect.addEventListener("change", () => {
        setTheme(themeSelect.value as ThemeChoice);
        bumpThemeVersion(); // repaint canvas panels (log views, correlation) with new colours
        setStatus(`Theme: ${themeSelect.value}`);
      });
    }
    const langSelect = q<HTMLSelectElement>("#language-select");
    if (langSelect) {
      langSelect.value = getLocale();
      langSelect.addEventListener("change", () => {
        setLocale(langSelect.value as Locale);
        setStatus(`Language: ${langSelect.selectedOptions[0]?.textContent ?? langSelect.value}`);
      });
    }

    // --- Data ---
    q<HTMLButtonElement>("#import-las-btn")?.addEventListener("click", () => void this.handleImport());
    q<HTMLButtonElement>("#export-las-btn")?.addEventListener("click", () => void this.handleExport());
    q<HTMLButtonElement>("#import-core-btn")?.addEventListener("click", () => void this.handleImportCore());
    q<HTMLButtonElement>("#shift-core-btn")?.addEventListener("click", () => this.handleShiftCore());
    q<HTMLButtonElement>("#import-dlis-btn")?.addEventListener("click", () => void this.handleImportDlis());
    q<HTMLButtonElement>("#import-scal-btn")?.addEventListener("click", () => void this.handleImportScal());
    q<HTMLButtonElement>("#import-deviation-btn")?.addEventListener("click", () => void this.handleImportDeviation());
    q<HTMLButtonElement>("#well-header-btn")?.addEventListener("click", () => this.handleWellHeader());
    q<HTMLButtonElement>("#open-wells-btn")?.addEventListener("click", () => workspace.openWellsTops());
    q<HTMLButtonElement>("#open-inspector-btn")?.addEventListener("click", () => workspace.openInspector());
    q<HTMLButtonElement>("#db-inspector-btn")?.addEventListener("click", () => workspace.openDbInspector());
    q<HTMLButtonElement>("#sql-query-btn")?.addEventListener("click", () => workspace.openSqlQuery());

    // --- Petrophysics ---
    q<HTMLButtonElement>("#zones-btn")?.addEventListener("click", () => {
      const well = appState.selectedWell.get();
      if (!well) {
        setStatus("Select a well first (Wells & Tops panel)");
        return;
      }
      void openZonesDialog(well, setStatus);
    });
    q<HTMLButtonElement>("#paysum-btn")?.addEventListener("click", () => {
      void openSummaryDialog(appState.selectedWell.get(), {
        setStatus,
        onRunComplete: () => workspace.notifyDataChanged(),
      });
    });
    q<HTMLButtonElement>("#workflow-btn")?.addEventListener("click", () => {
      void openWorkflowDialog(setStatus);
    });
    q<HTMLButtonElement>("#montecarlo-btn")?.addEventListener("click", () => {
      void openMonteCarloDialog(setStatus);
    });
    q<HTMLButtonElement>("#ml-btn")?.addEventListener("click", () => {
      void openMlDialog(setStatus);
    });
    q<HTMLButtonElement>("#multimin-btn")?.addEventListener("click", () => {
      void openMultiminDialog(setStatus);
    });
    q<HTMLButtonElement>("#dashboard-btn")?.addEventListener("click", () => workspace.openDashboard());
    void this.loadAllModules(root);

    // --- Plot ---
    q<HTMLButtonElement>("#new-logview-btn")?.addEventListener("click", () => workspace.openLogView());
    q<HTMLButtonElement>("#layout-props-btn")?.addEventListener("click", () => {
      const view = this.workspace.activeLogView();
      if (!view) {
        setStatus("Open a Log View first (Plot → New Log View)");
        return;
      }
      void view.openProperties();
    });
    q<HTMLButtonElement>("#save-layout-btn")?.addEventListener("click", () => this.handleSaveLayout());
    q<HTMLButtonElement>("#histogram-btn")?.addEventListener("click", () => workspace.openPlot("histogram"));
    q<HTMLButtonElement>("#crossplot-btn")?.addEventListener("click", () => workspace.openPlot("crossplot"));
    q<HTMLButtonElement>("#pickett-btn")?.addEventListener("click", () => workspace.openPlot("pickett"));
    q<HTMLButtonElement>("#correlation-btn")?.addEventListener("click", () => workspace.openPlot("correlation"));
    q<HTMLButtonElement>("#composite-btn")?.addEventListener("click", () => {
      const well = appState.selectedWell.get();
      if (!well) {
        setStatus("Select a well first (Wells & Tops panel)");
        return;
      }
      void openCompositeDialog(well, setStatus);
    });
    q<HTMLButtonElement>("#report-btn")?.addEventListener("click", () => {
      const well = appState.selectedWell.get();
      if (!well) {
        setStatus("Select a well first (Wells & Tops panel)");
        return;
      }
      void openReportDialog(well, setStatus);
    });
    const layoutSelect = q<HTMLSelectElement>("#layout-select");
    if (layoutSelect) {
      layoutSelect.addEventListener("change", () => {
        const layout = this.layouts.find((l) => l.name === layoutSelect.value);
        if (!layout) return;
        appState.activeLayout.set(layout);
        this.workspace.activeLogView()?.setLayout(layout);
        setStatus(`Layout: ${layout.name}`);
      });
      void this.loadLayouts(layoutSelect);
    }

    // --- View --- (depth scale / zoom / track width live in each log view's own toolbar)
    q<HTMLButtonElement>("#new-window-btn")?.addEventListener("click", () => {
      workspace.newWindow();
      setStatus("New window added — fill it with its ＋ button or drag panel tabs into it");
    });
    q<HTMLButtonElement>("#reset-workspace-btn")?.addEventListener("click", () => {
      workspace.resetWorkspace();
      setStatus("Workspace reset to default");
    });
  }

  private attachTabs(root: HTMLElement): void {
    const tabs = Array.from(root.querySelectorAll<HTMLButtonElement>(".ribbon-tab"));
    const panels = new Map<string, HTMLElement>(
      Array.from(root.querySelectorAll<HTMLElement>(".ribbon-panel")).map((el) => [el.dataset.panel!, el]),
    );
    for (const tab of tabs) {
      tab.addEventListener("click", () => {
        const target = tab.dataset.tab!;
        for (const t of tabs) t.classList.toggle("active", t === tab);
        for (const [key, el] of panels) el.hidden = key !== target;
      });
    }
  }

  /** Modules promoted out of the auto-generated category dropdowns into the dedicated
   *  "Advance" tab — Jauhar's flagship in-house methods. Skipped by the category render
   *  so they appear only once, as their own buttons. */
  /** "multimin" (the legacy fixed 4-component inversion) is filtered out of the Saturation
   *  dropdown but given no Advance button: it is superseded by SandiMin (the generalized
   *  solver) and Jauhar asked for mineral inversion to be independent of Sw. It still runs
   *  in saved workflow chains. */
  private static readonly ADVANCED_MODULE_IDS = ["ssc", "sspw", "sw_rtc", "sw_imts", "thin_bed_ts", "multimin"] as const;

  /** Fetches the backend manifests once and fills both module areas: the Petrophysics
   *  tab (category dropdowns) and the Advance tab (the promoted flagship methods). */
  private async loadAllModules(root: HTMLElement): Promise<void> {
    let modules: ModuleSpec[] = [];
    try {
      modules = await listModules();
    } catch (err) {
      console.error("Failed to load module manifests:", err);
      return;
    }
    const petroEl = root.querySelector<HTMLElement>("#petro-modules");
    if (petroEl) this.renderCategoryModules(petroEl, modules);
    const advanceEl = root.querySelector<HTMLElement>("#advance-modules");
    if (advanceEl) this.renderAdvancedModules(advanceEl, modules);
  }

  /** Builds the Petrophysics tab from the backend manifests: one Office-style dropdown
   *  button per category (the methods are the menu items) — new modules appear
   *  automatically. "Prep" modules (formation temperature etc.) live in their own
   *  "Data Cond & Prep" group. Advance-tab methods are excluded here. */
  private renderCategoryModules(container: HTMLElement, modules: ModuleSpec[]): void {
    const advanced = new Set<string>(Ribbon.ADVANCED_MODULE_IDS);
    modules = modules.filter((spec) => !advanced.has(spec.name));
    container.innerHTML = "";

    // category id -> [dropdown label, group caption, icon path data]
    const CATEGORIES: Record<string, [string, string, string]> = {
      Prep: [
        "Data Prep",
        "Data Cond & Prep",
        "M5 15c1.5-3 2-8 5-8s3.5 5 5 8M4 11h3M13 11h3",
      ],
      VSH: [
        "VSH",
        "Shale Volume",
        "M3 4h14M3 8h10M3 12h14M3 16h8",
      ],
      Porosity: [
        "Porosity",
        "Porosity",
        "M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14ZM7.5 8.5h.01M12 7h.01M9 12.5h.01M12.5 11.5h.01M7 11h.01",
      ],
      Saturation: [
        "Saturation",
        "Water Saturation",
        "M10 3s-5 6-5 9.5a5 5 0 0 0 10 0C15 9 10 3 10 3Z",
      ],
      Permeability: [
        "Permeability",
        "Permeability",
        "M3 6h10M15 6h2M3 10h4M9 10h8M3 14h12M17 14h0M13 4l2 2-2 2M7 8l2 2-2 2M15 12l2 2-2 2",
      ],
      ThinBeds: [
        "Thin Beds",
        "Thin Beds",
        "M3 5h14M3 8h14M3 11.5h14M3 15h14",
      ],
      Facies: [
        "Facies",
        "Facies",
        "M5 6a1.5 1.5 0 1 0 0-.01M11 5a1.5 1.5 0 1 0 0-.01M14.5 9a1.5 1.5 0 1 0 0-.01M6 11a1.5 1.5 0 1 0 0-.01M11.5 13.5a1.5 1.5 0 1 0 0-.01",
      ],
    };
    const order = Object.keys(CATEGORIES);

    const byCategory = new Map<string, ModuleSpec[]>();
    for (const spec of modules) {
      const list = byCategory.get(spec.category) ?? [];
      list.push(spec);
      byCategory.set(spec.category, list);
    }

    for (const category of order) {
      const specs = byCategory.get(category);
      if (!specs) continue;
      const [label, caption, iconPath] = CATEGORIES[category];
      const group = document.createElement("div");
      group.className = "ribbon-group";
      group.appendChild(
        buildRibbonDropdown(label, iconPath, specs.map((spec) => ({
          label: spec.title,
          doc: spec.doc,
          onPick: () => this.openModule(spec),
        }))),
      );
      const captionEl = document.createElement("span");
      captionEl.className = "ribbon-group-caption";
      captionEl.textContent = caption;
      group.appendChild(captionEl);
      container.appendChild(group);
    }
  }

  /** Fills the Advance tab with the promoted flagship methods as their own icon buttons
   *  in one "Advance Methods" group. Short labels (SSC/SSPW/RtC/IMTS/Thin Beds)
   *  keep the tab compact; the full title + description live in the button tooltip. */
  private renderAdvancedModules(container: HTMLElement, modules: ModuleSpec[]): void {
    // module id -> [short label, group caption, icon path]
    const META: Record<string, [string, string, string]> = {
      ssc: ["SSC", "Advance Methods", "M10 3 3 9h4v6h6V9h4L10 3Z"],
      sspw: ["SSPW", "Advance Methods", "M3 6h14M3 10h14M3 14h14M6 4v12"],
      sw_rtc: ["RtC", "Advance Methods", "M10 3s-5 6-5 9.5a5 5 0 0 0 10 0C15 9 10 3 10 3ZM8 11.5l1.5 1.5L13 9.5"],
      sw_imts: ["IMTS", "Advance Methods", "M10 3s-5 6-5 9.5a5 5 0 0 0 10 0C15 9 10 3 10 3ZM7.5 12h5M10 9.5v5"],
      thin_bed_ts: ["Thin Beds", "Advance Methods", "M3 5h14M3 8h14M3 11.5h14M3 15h14"],
      // Legacy fixed inversion: filtered from Saturation but not shown here (see ADVANCED_MODULE_IDS).
      multimin: ["Mineral Inv", "(hidden)", "M10 2.5 3 6.5v7L10 17.5 17 13.5v-7L10 2.5Z"],
    };
    const groupOrder = ["Advance Methods"];
    const byId = new Map(modules.map((spec) => [spec.name, spec]));
    container.innerHTML = "";

    for (const caption of groupOrder) {
      const ids = Ribbon.ADVANCED_MODULE_IDS.filter((id) => META[id][1] === caption && byId.has(id));
      if (ids.length === 0) continue;
      const group = document.createElement("div");
      group.className = "ribbon-group";
      const row = document.createElement("div");
      row.className = "ribbon-btn-row";
      for (const id of ids) {
        const spec = byId.get(id)!;
        const [short, , iconPath] = META[id];
        const btn = document.createElement("button");
        btn.className = "ribbon-btn";
        btn.title = `${spec.title} — ${spec.doc}`;
        btn.innerHTML =
          `<svg class="ribbon-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" ` +
          `stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="${iconPath}"/></svg>` +
          `<span class="ribbon-label">${short}</span>`;
        btn.addEventListener("click", () => this.openModule(spec));
        row.appendChild(btn);
      }
      group.appendChild(row);
      const captionEl = document.createElement("span");
      captionEl.className = "ribbon-group-caption";
      captionEl.textContent = caption;
      group.appendChild(captionEl);
      container.appendChild(group);
    }
  }

  private openModule(spec: ModuleSpec): void {
    void openModuleDialog(spec, appState.selectedWell.get(), {
      setStatus,
      onRunComplete: () => {
        recordProcess("Module", `Ran ${spec.title}`, appState.selectedWell.get()?.well_name ?? null);
        this.workspace.notifyDataChanged();
      },
    });
  }

  /** Built-ins from Rust plus user-saved layouts from the `documents` table. */
  private async loadLayouts(select: HTMLSelectElement, keepSelection = false): Promise<void> {
    const previous = select.value;
    let builtins: Layout[] = [];
    let saved: Layout[] = [];
    try {
      builtins = await listLayouts();
    } catch (err) {
      console.error("Failed to load layouts:", err);
    }
    try {
      saved = (await listDocuments("layout")).flatMap((doc) => {
        try {
          const layout = JSON.parse(doc.json) as Layout;
          layout.name = doc.name;
          return [layout];
        } catch {
          return [];
        }
      });
    } catch (err) {
      console.error("Failed to load saved layouts:", err);
    }
    // A saved layout shadows a built-in of the same name.
    const byName = new Map<string, Layout>();
    for (const l of [...builtins, ...saved]) byName.set(l.name, l);
    this.layouts = Array.from(byName.values());

    select.innerHTML = "";
    for (const layout of this.layouts) {
      const option = document.createElement("option");
      option.value = layout.name;
      option.textContent = layout.name;
      select.appendChild(option);
    }
    if (this.layouts.length === 0) return;
    if (keepSelection && this.layouts.some((l) => l.name === previous)) {
      select.value = previous;
      return;
    }
    appState.activeLayout.set(this.layouts[0]);
    this.workspace.activeLogView()?.setLayout(this.layouts[0]);
  }

  /** "Save Layout…" — names the active log view's current layout (tracks, styles,
   *  fills, widths are all part of it) and stores it in the project database. */
  private handleSaveLayout(): void {
    const view = this.workspace.activeLogView();
    const layout = view?.getLayout();
    if (!view || !layout) {
      setStatus("Open a Log View first (Plot → New Log View)");
      return;
    }
    const content = document.createElement("div");
    const nameInput = document.createElement("input");
    nameInput.className = "form-control";
    nameInput.value = layout.name === "Standard Layout" ? "My Layout" : layout.name;
    content.appendChild(formRow("Layout name", nameInput));
    const saveBtn = document.createElement("button");
    saveBtn.className = "lp-btn primary";
    saveBtn.textContent = "Save";
    saveBtn.style.marginTop = "10px";
    content.appendChild(saveBtn);
    const close = openModal("Save Layout As", content, 380);
    nameInput.focus();
    nameInput.select();

    const doSave = async () => {
      const name = nameInput.value.trim();
      if (!name) return;
      try {
        const toSave = structuredClone(layout);
        toSave.name = name;
        await saveDocument("layout", name, JSON.stringify(toSave));
        close();
        setStatus(`Layout "${name}" saved`);
        const select = document.querySelector<HTMLSelectElement>("#layout-select");
        if (select) {
          await this.loadLayouts(select, true);
          select.value = name;
        }
      } catch (err) {
        setStatus(`Save failed: ${err}`);
      }
    };
    saveBtn.addEventListener("click", () => void doSave());
    nameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void doSave();
    });
  }

  static {
    // Any click outside a dropdown closes every open ribbon menu.
    document.addEventListener("click", (e) => {
      if ((e.target as HTMLElement).closest?.(".ribbon-dropdown")) return;
      for (const menu of document.querySelectorAll<HTMLElement>(".ribbon-menu:not([hidden])")) menu.hidden = true;
    });
  }

  private async handleSaveProject(): Promise<void> {
    let dest: string | null;
    try {
      dest = await save({
        title: "Save Project As",
        defaultPath: "sandibumi-project.duckdb",
        filters: [{ name: "SandiBumi / DuckDB project", extensions: ["duckdb"] }],
      });
    } catch (err) {
      setStatus(`Save dialog unavailable: ${err}`);
      return;
    }
    if (!dest) return;
    try {
      await saveProjectAs(dest);
      setStatus(`Project saved to ${dest}`);
      recordProcess("Project", `Saved project to ${dest}`);
    } catch (err) {
      setStatus(`Save failed: ${err}`);
    }
  }

  /** "Save Session As…" — names the current workspace (open panes, wells, visualizations)
   *  and stores it in the project database so it can be reopened later. Unlike Save
   *  Project As (which copies the whole database file), a session is just the arrangement. */
  private handleSaveSession(): void {
    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Saves the current workspace — which panes, plots and log views are open, their " +
      "arrangement, and the active well — under a name. Reopen it any time from Open Session.";
    content.appendChild(doc);
    const nameInput = document.createElement("input");
    nameInput.className = "form-control";
    nameInput.value = "My Session";
    content.appendChild(formRow("Session name", nameInput));
    const saveBtn = document.createElement("button");
    saveBtn.className = "lp-btn primary";
    saveBtn.textContent = "Save";
    saveBtn.style.marginTop = "10px";
    content.appendChild(saveBtn);
    const close = openModal("Save Session As", content, 420);
    nameInput.focus();
    nameInput.select();

    const doSave = async () => {
      const name = nameInput.value.trim();
      if (!name) return;
      try {
        await saveDocument("session", name, JSON.stringify(this.workspace.snapshotSession()));
        close();
        setStatus(`Session "${name}" saved`);
        recordProcess("Session", `Saved session "${name}"`);
      } catch (err) {
        setStatus(`Save failed: ${err}`);
      }
    };
    saveBtn.addEventListener("click", () => void doSave());
    nameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void doSave();
    });
  }

  /** "Open Session…" — lists saved sessions; picking one rebuilds the workspace from it,
   *  and each row can be deleted. */
  private async handleOpenSession(): Promise<void> {
    let sessions: { name: string; json: string }[] = [];
    try {
      sessions = await listDocuments("session");
    } catch (err) {
      setStatus(`Could not load sessions: ${err}`);
      return;
    }

    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Reopen a saved workspace. This replaces the current panes and visualizations with " +
      "the session's arrangement and switches to its well.";
    content.appendChild(doc);

    const list = document.createElement("div");
    list.className = "session-list";
    content.appendChild(list);
    const close = openModal("Open Session", content, 460);

    const renderList = () => {
      list.innerHTML = "";
      if (sessions.length === 0) {
        const empty = document.createElement("div");
        empty.className = "session-empty";
        empty.textContent = "No saved sessions yet. Use Save Session to create one.";
        list.appendChild(empty);
        return;
      }
      for (const session of sessions) {
        const row = document.createElement("div");
        row.className = "session-row";
        const openBtn = document.createElement("button");
        openBtn.className = "session-open-btn";
        openBtn.textContent = session.name;
        openBtn.title = `Open session "${session.name}"`;
        openBtn.addEventListener("click", () => {
          let snap: SessionSnapshot;
          try {
            snap = JSON.parse(session.json) as SessionSnapshot;
          } catch {
            setStatus(`Session "${session.name}" is corrupt and can't be opened`);
            return;
          }
          this.workspace.applySession(snap);
          close();
          setStatus(`Opened session "${session.name}"`);
          recordProcess("Session", `Opened session "${session.name}"`);
        });
        const delBtn = document.createElement("button");
        delBtn.className = "session-del-btn";
        delBtn.textContent = "🗑";
        delBtn.title = `Delete session "${session.name}"`;
        delBtn.addEventListener("click", () => {
          void deleteDocument("session", session.name)
            .then(() => {
              sessions = sessions.filter((s) => s.name !== session.name);
              renderList();
              setStatus(`Deleted session "${session.name}"`);
            })
            .catch((err) => setStatus(`Delete failed: ${err}`));
        });
        row.append(openBtn, delBtn);
        list.appendChild(row);
      }
    };
    renderList();
  }

  /** "Export LAS…" — writes the selected well (standard + computed curves) as LAS 2.0. */
  private async handleExport(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let dest: string | null;
    try {
      dest = await save({
        title: `Export ${well.well_name} as LAS 2.0`,
        defaultPath: `${well.well_name.replace(/[^\w.-]+/g, "_")}.las`,
        filters: [{ name: "LAS 2.0", extensions: ["las"] }],
      });
    } catch (err) {
      setStatus(`Export dialog unavailable: ${err}`);
      return;
    }
    if (!dest) return;
    try {
      const rows = await exportLas(well.well_id, dest);
      setStatus(`Exported ${well.well_name} (${rows} rows) to ${dest}`);
      recordProcess("Export", `Exported LAS (${rows} rows) → ${dest}`, well.well_name);
    } catch (err) {
      setStatus(`Export failed: ${err}`);
    }
  }

  private async handleImport(): Promise<void> {
    let paths: string[] | null;
    try {
      const selection = await open({
        multiple: true,
        filters: [{ name: "LAS 2.0", extensions: ["las"] }],
      });
      paths = Array.isArray(selection) ? selection : selection ? [selection] : null;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }

    if (!paths || paths.length === 0) return;

    setStatus(`Importing ${paths.length} LAS file(s)...`);
    try {
      const results = await importLasFiles(paths);
      const ok = results.filter((r) => !r.error).length;
      setStatus(`Imported ${ok}/${results.length} well(s).`);
      recordProcess("Import", `Imported ${ok}/${results.length} LAS well(s)`);
      this.workspace.notifyDataChanged();
    } catch (err) {
      setStatus(`Import failed: ${err}`);
    }
  }

  /** "Import Core…" — replaces the selected well's routine core analysis data
   *  (CPOR/CPERM/CGD/CSW) from a CSV; overlaid onto the crossplot panel. */
  private async handleImportCore(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        filters: [{ name: "Core Data CSV", extensions: ["csv"] }],
      });
      path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;

    setStatus(`Importing core data for ${well.well_name}...`);
    try {
      const result = await importCoreCsv(well.well_id, path);
      if (result.error) {
        setStatus(`Core import failed: ${result.error}`);
      } else {
        setStatus(`Imported ${result.rows} core sample(s) for ${well.well_name}.`);
        this.workspace.notifyDataChanged();
      }
    } catch (err) {
      setStatus(`Core import failed: ${err}`);
    }
  }

  /** "Shift Core…" — constant core-to-log depth shift for the selected well's plugs.
   *  Exactly reversible, so it lands on the undo stack (Ctrl+Z shifts back). */
  private handleShiftCore(): void {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Moves every core plug of the selected well by a constant depth (+ = deeper). " +
      "Use it to align core porosity/permeability points with the log response, then Ctrl+Z to revert if needed.";
    content.appendChild(doc);
    const input = document.createElement("input");
    input.type = "number";
    input.step = "0.1";
    input.className = "form-control";
    input.placeholder = "e.g. 2.5";
    content.appendChild(formRow("Shift (m)", input, "+ = plugs move deeper"));
    const apply = document.createElement("button");
    apply.className = "form-run-btn";
    apply.textContent = "Apply Shift";
    content.appendChild(apply);

    const close = openModal(`Shift Core — ${well.well_name}`, content, 420);
    const doShift = async (delta: number): Promise<void> => {
      const n = await shiftCoreData(well.well_id, delta);
      setStatus(`Shifted ${n} core plug(s) of ${well.well_name} by ${delta > 0 ? "+" : ""}${delta} m`);
      recordProcess("Edit", `Core shift ${delta > 0 ? "+" : ""}${delta} m (${n} plugs)`, well.well_name);
      this.workspace.notifyDataChanged();
    };
    apply.addEventListener("click", () => {
      const delta = Number(input.value);
      if (!Number.isFinite(delta) || delta === 0) {
        setStatus("Enter a non-zero shift in metres");
        return;
      }
      void doShift(delta)
        .then(() => {
          pushUndo({
            label: `core shift ${delta} m (${well.well_name})`,
            undo: () => void doShift(-delta),
            redo: () => void doShift(delta),
          });
          close();
        })
        .catch((err) => setStatus(`Core shift failed: ${err}`));
    });
    input.focus();
  }

  /** "Import DLIS…" — loads every scalar channel from a DLIS file into the selected
   *  well's generic curve store (RAW set), via dlisio through the Python subprocess. */
  private async handleImportDlis(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        filters: [{ name: "DLIS", extensions: ["dlis", "DLIS"] }],
      });
      path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;

    setStatus(`Importing DLIS into ${well.well_name}… (dlisio may take a moment)`);
    try {
      const result = await importDlisFile(well.well_id, path);
      if (result.error) {
        setStatus(`DLIS import failed: ${result.error}`);
      } else {
        setStatus(`Imported ${result.curves_imported} curve(s), ${result.rows} samples into ${well.well_name}.`);
        this.workspace.notifyDataChanged();
      }
    } catch (err) {
      setStatus(`DLIS import failed: ${err}`);
    }
  }

  /** "Import Deviation…" — loads an MD/INC/AZI survey CSV and computes minimum-curvature
   *  TVD/TVDSS for the selected well. Prompts for the datum (KB) elevation. */
  /** "Import SCAL…" — replaces the well's capillary-pressure (Pc/Sw) points from a CSV
   *  and fits the Leverett J-function, reporting SWH_A/SWH_B for the sw_height module. */
  private async handleImportScal(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        filters: [{ name: "SCAL Pc CSV", extensions: ["csv"] }],
      });
      path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;

    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Imports capillary-pressure points (PC/SW columns; SAMPLE/DEPTH/PERM/PORO optional) and fits " +
      "the Leverett J-function Sw = A·J^B. The lab sigma·cosθ converts Pc to J: 72 air-brine, " +
      "367 air-mercury, 26 oil-brine. Carry the fitted A/B into SW — Saturation-Height (SWH_A/SWH_B).";
    content.appendChild(doc);
    const iftInput = document.createElement("input");
    iftInput.type = "number";
    iftInput.step = "0.1";
    iftInput.className = "form-control";
    iftInput.value = "72";
    content.appendChild(formRow("Lab sigma·cosθ (dyn/cm)", iftInput, "Fluid system of the lab measurement"));
    const apply = document.createElement("button");
    apply.className = "form-run-btn";
    apply.textContent = "Import & Fit";
    const resultBox = document.createElement("div");
    resultBox.className = "modal-result";
    content.appendChild(apply);
    content.appendChild(resultBox);

    openModal(`Import SCAL — ${well.well_name}`, content, 480);
    apply.addEventListener("click", () => {
      const ift = Number(iftInput.value);
      if (!Number.isFinite(ift) || ift <= 0) {
        resultBox.textContent = "Lab sigma·cosθ must be a positive number.";
        return;
      }
      apply.disabled = true;
      resultBox.textContent = `Importing SCAL data for ${well.well_name}…`;
      void importScalCsv(well.well_id, path, ift)
        .then((result) => {
          if (result.error) {
            resultBox.textContent = `SCAL import failed: ${result.error}`;
            return;
          }
          const fitText = result.fit
            ? `J-fit: A = ${result.fit.a.toFixed(4)}, B = ${result.fit.b.toFixed(4)}, ` +
              `R² = ${result.fit.r2.toFixed(3)} (${result.fit.n_points} points). ` +
              `Enter these as SWH_A/SWH_B in SW — Saturation-Height.`
            : "Too few valid points to fit the J-function (need Pc, Sw, perm and poro on ≥ 3 rows).";
          resultBox.textContent = `Imported ${result.rows} Pc point(s). ${fitText}`;
          setStatus(`SCAL: ${result.rows} points imported for ${well.well_name}.`);
          this.workspace.notifyDataChanged();
        })
        .catch((err) => {
          resultBox.textContent = `SCAL import failed: ${err}`;
        })
        .finally(() => {
          apply.disabled = false;
        });
    });
  }

  private async handleImportDeviation(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        filters: [{ name: "Deviation Survey CSV", extensions: ["csv"] }],
      });
      path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;

    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Computes TVD/TVDSS by the minimum-curvature method from the MD/INC/AZI survey. " +
      "Datum elevation (KB above mean sea level) sets TVDSS = datum − TVD; leave blank to use the well's KB.";
    content.appendChild(doc);
    const datumInput = document.createElement("input");
    datumInput.type = "number";
    datumInput.step = "0.1";
    datumInput.className = "form-control";
    datumInput.placeholder = "e.g. 25 (optional)";
    content.appendChild(formRow("Datum / KB (m)", datumInput, "TVDSS reference; blank = well KB"));
    const apply = document.createElement("button");
    apply.className = "form-run-btn";
    apply.textContent = "Import Survey";
    content.appendChild(apply);

    const close = openModal(`Import Deviation — ${well.well_name}`, content, 460);
    apply.addEventListener("click", () => {
      const raw = datumInput.value.trim();
      const datum = raw === "" ? null : Number(raw);
      if (datum !== null && !Number.isFinite(datum)) {
        setStatus("Datum must be a number, or blank");
        return;
      }
      setStatus(`Importing deviation survey for ${well.well_name}…`);
      void importDeviationCsv(well.well_id, path, datum)
        .then((result) => {
          if (result.error) {
            setStatus(`Deviation import failed: ${result.error}`);
          } else {
            setStatus(`Imported ${result.rows} survey station(s); TVD/TVDSS computed for ${well.well_name}.`);
            this.workspace.notifyDataChanged();
            close();
          }
        })
        .catch((err) => setStatus(`Deviation import failed: ${err}`));
    });
    datumInput.focus();
  }

  /** "Well Header…" — edits the selected well's field / TD / KB datum (Phase 6c). */
  private handleWellHeader(): void {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent = "Edit this well's header. KB is the datum elevation used for TVDSS.";
    content.appendChild(doc);

    const fieldInput = document.createElement("input");
    fieldInput.type = "text";
    fieldInput.className = "form-control";
    fieldInput.value = well.field_name ?? "";
    content.appendChild(formRow("Field", fieldInput));

    const tdInput = document.createElement("input");
    tdInput.type = "number";
    tdInput.step = "0.1";
    tdInput.className = "form-control";
    tdInput.placeholder = "total depth (m)";
    content.appendChild(formRow("TD (m)", tdInput));

    const kbInput = document.createElement("input");
    kbInput.type = "number";
    kbInput.step = "0.1";
    kbInput.className = "form-control";
    kbInput.placeholder = "KB elevation (m)";
    content.appendChild(formRow("KB (m)", kbInput, "datum for TVDSS"));

    const applyBtn = document.createElement("button");
    applyBtn.className = "form-run-btn";
    applyBtn.textContent = "Save Header";
    content.appendChild(applyBtn);

    const close = openModal(`Well Header — ${well.well_name}`, content, 440);
    applyBtn.addEventListener("click", () => {
      const field = fieldInput.value.trim();
      const td = tdInput.value.trim();
      const kb = kbInput.value.trim();
      const writes: Promise<void>[] = [
        updateWellField(well.well_id, "field_name", field === "" ? null : field),
      ];
      if (td !== "") writes.push(updateWellField(well.well_id, "td", td));
      if (kb !== "") writes.push(updateWellField(well.well_id, "kb", kb));
      void Promise.all(writes)
        .then(() => {
          setStatus(`Updated header for ${well.well_name}.`);
          recordProcess("Edit", "Updated well header", well.well_name);
          this.workspace.notifyDataChanged();
          close();
        })
        .catch((err) => setStatus(`Header update failed: ${err}`));
    });
    fieldInput.focus();
  }
}
