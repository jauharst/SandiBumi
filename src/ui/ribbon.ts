import { open, save } from "@tauri-apps/plugin-dialog";
import {
  exportLas,
  importAuxData,
  importCoreCsv,
  importDeviationCsv,
  importScalFiles,
  importTopsCsv,
  importWellLocations,
  importDlisFile,
  importLasFiles,
  currentProject,
  listRecentProjects,
  newProject,
  openProject,
  deleteDocument,
  listDocuments,
  listLayouts,
  listModules,
  listWells,
  saveDocument,
  saveProjectAs,
  shiftCoreData,
  updateWellField,
  type Layout,
  type ModuleSpec,
  type RecentProject,
} from "../ipc";
import { appState, bumpThemeVersion, setStatus } from "../state";
import { anyDirty, clearDirty, subscribeDirty } from "../dirty";
import { syncWellGroups } from "./wellGroups";
import { clearUndoStacks, nextRedoLabel, nextUndoLabel, onUndoChange, pushUndo, redo, redoDepth, undo, undoDepth } from "../undo";
import { recordProcess } from "../processLog";
import { getTheme, setTheme, type ThemeChoice } from "../theme";
import { getLocale, setLocale, type Locale } from "../i18n";
import type { SessionSnapshot, Workspace } from "./workspace";
import { formRow, openModal } from "./modal";

interface RibbonMenuItem {
  label: string;
  doc: string;
  onPick: () => void;
}

/** Places a `.ribbon-menu` (position:fixed) just below its button, clamped to the viewport.
 *  Fixed positioning is what lets the menu escape the ribbon panel's horizontal-scroll clip,
 *  so the coordinates must be set here on every open rather than by CSS `top:100%`. */
function positionRibbonMenu(menu: HTMLElement, anchor: HTMLElement): void {
  const rect = anchor.getBoundingClientRect();
  menu.style.left = `${Math.max(4, Math.min(rect.left, window.innerWidth - 228))}px`;
  menu.style.top = `${rect.bottom}px`;
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
    if (wasOpen) return;
    positionRibbonMenu(menu, button);
    menu.hidden = false;
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
  /** Name of the session last saved or opened, so Ctrl+S can re-save it in place without a
   *  dialog. Null until a session has a name → Ctrl+S then falls back to Save Session As. */
  private lastSessionName: string | null = null;

  /** Set by initRibbonOverflow — lets tab switches and async module loads re-check whether
   *  the overflow chevrons should show. */
  private updateRibbonOverflow?: () => void;

  constructor(root: HTMLElement, private workspace: Workspace) {
    this.attachTabs(root);
    this.initRibbonOverflow(root);

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
    // Ctrl/Cmd+S quietly re-saves the current session (no dialog once it has a name); Escape
    // closes any open ribbon menu. Only intercept Ctrl+S when the target isn't a text field —
    // an editor/CodeMirror inside a pane keeps its own Save. (Ribbon is a singleton created
    // once in main.ts, so this window listener is registered exactly once.)
    window.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && !e.altKey && (e.key === "s" || e.key === "S")) {
        const el = e.target as HTMLElement | null;
        const editable =
          el?.isContentEditable ||
          el?.tagName === "INPUT" ||
          el?.tagName === "TEXTAREA" ||
          el?.closest(".cm-editor") != null;
        if (editable) return;
        e.preventDefault();
        void this.quickSaveSession();
      } else if (e.key === "Escape") {
        // Soft close: only touch open ribbon menus, and don't stop propagation so a modal's
        // own Escape handling (scoped) is unaffected.
        for (const menu of document.querySelectorAll<HTMLElement>(".ribbon-menu:not([hidden])")) {
          menu.hidden = true;
        }
      }
    });
    const saveSessionBtn = q<HTMLButtonElement>("#qat-save-session");
    saveSessionBtn?.addEventListener("click", () => this.handleSaveSession());
    q<HTMLButtonElement>("#qat-open-session")?.addEventListener("click", () => void this.handleOpenSession());
    q<HTMLButtonElement>("#qat-history")?.addEventListener("click", () => workspace.openHistory());
    // Contextual Help (?): opens a guide for whichever panel is active — the future hook for
    // the illustrated HTML help library, keyed to the "current active panel".
    q<HTMLButtonElement>("#qat-help")?.addEventListener("click", () => void workspace.openHelpForActivePanel());
    // Unsaved-state dot: lights while any panel/workspace state isn't in a named save yet.
    if (saveSessionBtn) {
      const baseTitle = saveSessionBtn.title;
      subscribeDirty(() => {
        const dirty = anyDirty();
        saveSessionBtn.classList.toggle("qat-dirty", dirty);
        saveSessionBtn.title = dirty ? `${baseTitle} — unsaved changes` : baseTitle;
      });
    }

    // --- Project ---
    q<HTMLButtonElement>("#save-project-btn")?.addEventListener("click", () => void this.handleSaveProject());
    q<HTMLButtonElement>("#open-project-btn")?.addEventListener("click", () => void this.handleOpenProject());
    q<HTMLButtonElement>("#new-project-btn")?.addEventListener("click", () => void this.handleNewProject());
    this.buildRecentProjectsDropdown(root);
    // Reflect the startup project in the window title + group caption (fails
    // benignly in the vite-only preview where there is no backend).
    void currentProject()
      .then((info) => this.reflectProject(info, false))
      .catch(() => {});
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
    q<HTMLButtonElement>("#export-las-btn")?.addEventListener("click", () => void this.handleExport());
    this.buildDataDropdowns(root);
    q<HTMLButtonElement>("#open-wells-btn")?.addEventListener("click", () => workspace.openWellsTops());
    q<HTMLButtonElement>("#open-inspector-btn")?.addEventListener("click", () => workspace.openInspector());
    q<HTMLButtonElement>("#db-inspector-btn")?.addEventListener("click", () => workspace.openDbInspector());
    q<HTMLButtonElement>("#sql-query-btn")?.addEventListener("click", () => workspace.openSqlQuery());

    // --- Petrophysics ---
    q<HTMLButtonElement>("#zones-btn")?.addEventListener("click", () => workspace.openZones());
    q<HTMLButtonElement>("#paysum-btn")?.addEventListener("click", () => workspace.openPaySummary());
    q<HTMLButtonElement>("#cutoff-sens-btn")?.addEventListener("click", () => workspace.openCutoff());
    q<HTMLButtonElement>("#workflow-btn")?.addEventListener("click", () => workspace.openWorkflow());
    q<HTMLButtonElement>("#processing-btn")?.addEventListener("click", () => workspace.openProcessing());
    q<HTMLButtonElement>("#health-btn")?.addEventListener("click", () => workspace.openHealth());
    // The Workflow Builder fires this when a chain starts so the universal Processing panel
    // pops open on its own — the user shouldn't have to hunt for progress. Ribbon is a
    // singleton created once in main.ts, so this window listener is registered exactly once.
    window.addEventListener("sandibumi:open-processing", () => workspace.openProcessing());
    q<HTMLButtonElement>("#montecarlo-btn")?.addEventListener("click", () => workspace.openMonteCarlo());
    q<HTMLButtonElement>("#ml-btn")?.addEventListener("click", () => workspace.openMl());
    q<HTMLButtonElement>("#multimin-btn")?.addEventListener("click", () => workspace.openMultimin());
    q<HTMLButtonElement>("#dashboard-btn")?.addEventListener("click", () => workspace.openDashboard());
    q<HTMLButtonElement>("#map-btn")?.addEventListener("click", () => workspace.openMap());
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
    q<HTMLButtonElement>("#composite-btn")?.addEventListener("click", () => workspace.openComposite());
    q<HTMLButtonElement>("#report-btn")?.addEventListener("click", () => workspace.openReport());
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

  /** Compact Data-tab import/export (ROADMAP §4c item 4): the eleven flat buttons are
   *  now three Office-style dropdowns around the static Export LAS button. Tooltips
   *  carry the per-format guidance the old buttons' title attributes held. */
  private buildDataDropdowns(root: HTMLElement): void {
    const row = root.querySelector<HTMLElement>("#data-io-row");
    const exportBtn = root.querySelector<HTMLButtonElement>("#export-las-btn");
    if (!row || !exportBtn) return;

    const importLogs = buildRibbonDropdown(
      "Import Logs",
      "M4 13v3a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-3M10 3v9M6.5 8.5 10 12l3.5-3.5",
      [
        {
          label: "Import LAS…",
          doc: "Import an LAS 2.0 file as a new well (every curve, family-aliased)",
          onPick: () => void this.handleImport(),
        },
        {
          label: "Import DLIS…",
          doc: "Import every curve from a DLIS file into the selected well (via dlisio)",
          onPick: () => void this.handleImportDlis(),
        },
      ],
    );

    const importData = buildRibbonDropdown(
      "Import Data",
      "M6 3h6l3 3v11a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z M7 11l2.5 2.5L13 9",
      [
        {
          label: "Import Core…",
          doc: "Import routine core analysis (CPOR/CPERM/CGD/CSW) CSV for the selected well",
          onPick: () => void this.handleImportCore(),
        },
        {
          label: "Import SCAL…",
          doc: "Import SCAL capillary pressure (flat CSV, porous-plate wide table, or per-plug centrifuge files) for the selected well and fit the Leverett J-function",
          onPick: () => void this.handleImportScal(),
        },
        {
          label: "Import Tops…",
          doc: "Import formation tops from CSV/TXT — a WELL column updates every matching well; without one, tops land in the selected well",
          onPick: () => void this.handleImportTops(),
        },
        {
          label: "Import Aux…",
          doc: "Import petrography, XRD or perforation data (tops-style CSV/TXT) for the selected well",
          onPick: () => this.handleImportAux(),
        },
        {
          label: "Import Deviation…",
          doc: "Import a deviation survey (MD/INC/AZI CSV) and compute TVD/TVDSS for the selected well",
          onPick: () => void this.handleImportDeviation(),
        },
        {
          label: "Import Well Locations…",
          doc: "Import well surface coordinates (WELL/EASTING/NORTHING CSV) for the Field Map — a WELL column locates every matching well",
          onPick: () => void this.handleImportWellLocations(),
        },
      ],
    );

    const tools = buildRibbonDropdown(
      "Tools",
      "M3 6c2-2 3 2 5 0s3 2 5 0 3 2 4 1M3 13c2-2 3 2 5 0s3 2 5 0 3 2 4 1M10 8v3",
      [
        {
          label: "Autocorrelate Tops…",
          doc: "Propagate a top from the selected well to other wells by matching a log's shape (GR by default)",
          onPick: () => this.workspace.openAutoCorr(),
        },
        {
          label: "Shift Core…",
          doc: "Shift the selected well's core plugs by a constant depth (core-to-log alignment; undoable)",
          onPick: () => this.handleShiftCore(),
        },
        {
          label: "Well Header…",
          doc: "Edit the selected well's header (field, TD, KB datum)",
          onPick: () => void this.handleWellHeader(),
        },
      ],
    );

    row.insertBefore(importLogs, exportBtn);
    row.insertBefore(importData, exportBtn);
    row.appendChild(tools);
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
        this.updateRibbonOverflow?.(); // the newly shown panel may over/under-flow differently
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
    // The Petrophysics/Advance panels just gained their group content, so their scroll width
    // changed — re-check whether the overflow chevrons are needed.
    this.updateRibbonOverflow?.();
  }

  /** PowerPoint-style ribbon overflow: when the active tab's group row is wider than the
   *  window, show a chevron box at each overflowing edge (scroll ‹ / more ›) instead of a raw
   *  scrollbar, so the user can always reach every tool. Chevrons appear only in the direction
   *  that has hidden content and scroll the active panel a page at a time. */
  private initRibbonOverflow(root: HTMLElement): void {
    const body = root.querySelector<HTMLElement>(".ribbon-body");
    if (!body) return;
    const panels = Array.from(root.querySelectorAll<HTMLElement>(".ribbon-panel"));

    const mkChevron = (side: "left" | "right", glyph: string, label: string): HTMLButtonElement => {
      const b = document.createElement("button");
      b.type = "button";
      b.className = `ribbon-overflow ribbon-overflow-${side}`;
      b.setAttribute("aria-label", label);
      b.title = label;
      b.textContent = glyph;
      b.hidden = true;
      return b;
    };
    const left = mkChevron("left", "‹", "Scroll ribbon left");
    const right = mkChevron("right", "›", "Show more ribbon tools");
    body.append(left, right);

    const activePanel = (): HTMLElement | null => panels.find((p) => !p.hidden) ?? null;
    const update = (): void => {
      const p = activePanel();
      if (!p) {
        left.hidden = true;
        right.hidden = true;
        return;
      }
      const max = p.scrollWidth - p.clientWidth;
      const overflowing = max > 1;
      left.hidden = !overflowing || p.scrollLeft <= 1;
      right.hidden = !overflowing || p.scrollLeft >= max - 1;
    };
    const scrollActive = (dir: number): void => {
      const p = activePanel();
      if (!p) return;
      p.scrollBy({ left: dir * Math.max(120, p.clientWidth * 0.7), behavior: "smooth" });
    };
    left.addEventListener("click", () => scrollActive(-1));
    right.addEventListener("click", () => scrollActive(1));

    for (const p of panels) p.addEventListener("scroll", update, { passive: true });
    window.addEventListener("resize", update);
    new ResizeObserver(update).observe(body);
    this.updateRibbonOverflow = update;
    requestAnimationFrame(update); // after first layout settles
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
      "Rock Typing": [
        "Rock Typing",
        "Rock Typing",
        "M4 14l3-5 3 3 3-6 3 8M3 16h14",
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
    // Each module is a singleton dock pane; run bookkeeping (process log + data-version
    // bump) lives with the pane host in workspace.ts, so restored panes get it too.
    this.workspace.openModulePane(spec);
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
    const entry = this.workspace.activeLogViewEntry();
    const view = entry?.view;
    const layout = view?.getLayout();
    if (!entry || !view || !layout) {
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
        // This panel's layout is now in a named save — drop its ● (the title update
        // fires a layout event, so mute workspace-dirty around it).
        this.workspace.muteDirty();
        clearDirty(entry.id);
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

  /** "IP style" project switching (ROADMAP §4c item 2): open an existing .duckdb. */
  private async handleOpenProject(): Promise<void> {
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        title: "Open Project",
        filters: [{ name: "SandiBumi / DuckDB project", extensions: ["duckdb"] }],
      });
      path = typeof selection === "string" ? selection : null;
    } catch (err) {
      setStatus(`Open dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;
    await this.switchProject(() => openProject(path));
  }

  /** Creates a fresh, empty project database and switches to it. */
  private async handleNewProject(): Promise<void> {
    let path: string | null;
    try {
      path = await save({
        title: "New Project",
        defaultPath: "new-project.duckdb",
        filters: [{ name: "SandiBumi / DuckDB project", extensions: ["duckdb"] }],
      });
    } catch (err) {
      setStatus(`Save dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;
    await this.switchProject(() => newProject(path));
  }

  /** Runs one of the project-switch commands, then resets everything that referenced
   *  the old database: selection, undo stacks, well groups, and every data-driven pane. */
  private async switchProject(action: () => Promise<RecentProject>): Promise<void> {
    setStatus("Switching project…");
    let info: RecentProject;
    try {
      info = await action();
    } catch (err) {
      setStatus(`Project switch failed: ${err}`);
      return;
    }
    // Old-project references are meaningless now — clear BEFORE panels reload.
    appState.selectedInterval.set(null);
    appState.selectedWell.set(null);
    clearUndoStacks();
    await syncWellGroups().catch(() => {});
    this.reflectProject(info, true);
    this.workspace.notifyDataChanged();
    recordProcess("Project", `Opened project ${info.name} (${info.path})`);
    setStatus(`Project: ${info.name}`);
  }

  /** Window title + Project group caption show which project is open. */
  private reflectProject(info: RecentProject, announce: boolean): void {
    document.title = `SandiBumi — ${info.name}`;
    const caption = document.querySelector<HTMLElement>("#project-caption");
    if (caption) {
      caption.textContent = info.name;
      caption.title = info.path;
    }
    if (announce) void this.refreshRecentMenu();
  }

  /** The Recent ▾ dropdown: a menu rebuilt from the recents list every time it opens
   *  (unlike buildRibbonDropdown's fixed items). Missing files are disabled entries. */
  private recentMenu: HTMLElement | null = null;

  private buildRecentProjectsDropdown(root: HTMLElement): void {
    const row = root.querySelector<HTMLElement>("#project-row");
    if (!row) return;

    const wrap = document.createElement("div");
    wrap.className = "ribbon-dropdown";
    const button = document.createElement("button");
    button.className = "ribbon-btn ribbon-dropdown-btn";
    button.innerHTML = `
      <svg class="ribbon-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5"
           stroke-linecap="round" stroke-linejoin="round"><path d="M10 5v5l3.5 2M17 10a7 7 0 1 1-7-7 7 7 0 0 1 7 7Z"/></svg>
      <span class="ribbon-label">Recent <span class="ribbon-caret">▾</span></span>`;
    const menu = document.createElement("div");
    menu.className = "ribbon-menu";
    menu.hidden = true;
    this.recentMenu = menu;

    button.addEventListener("click", () => {
      const wasOpen = !menu.hidden;
      for (const m of document.querySelectorAll<HTMLElement>(".ribbon-menu:not([hidden])")) m.hidden = true;
      if (wasOpen) return;
      void this.refreshRecentMenu().then(() => {
        positionRibbonMenu(menu, button);
        menu.hidden = false;
      });
    });

    wrap.appendChild(button);
    wrap.appendChild(menu);
    row.appendChild(wrap);
  }

  private async refreshRecentMenu(): Promise<void> {
    const menu = this.recentMenu;
    if (!menu) return;
    menu.innerHTML = "";
    const [recents, current] = await Promise.all([
      listRecentProjects().catch(() => [] as RecentProject[]),
      currentProject().catch(() => null),
    ]);
    if (recents.length === 0) {
      const empty = document.createElement("div");
      empty.className = "ribbon-menu-empty";
      empty.textContent = "No recent projects";
      menu.appendChild(empty);
      return;
    }
    for (const r of recents) {
      const entry = document.createElement("button");
      entry.className = "ribbon-menu-item";
      entry.setAttribute("data-no-i18n", ""); // project names are user data
      const isCurrent = current !== null && r.path === current.path;
      entry.textContent = (isCurrent ? "● " : "") + r.name + (r.exists ? "" : "  (missing)");
      entry.title = r.path;
      entry.disabled = !r.exists || isCurrent;
      entry.addEventListener("click", () => {
        menu.hidden = true;
        void this.switchProject(() => openProject(r.path));
      });
      menu.appendChild(entry);
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
        await this.writeSession(name);
        close();
      } catch (err) {
        setStatus(`Save failed: ${err}`);
      }
    };
    saveBtn.addEventListener("click", () => void doSave());
    nameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void doSave();
    });
  }

  /** Persists the current workspace under `name` and clears the unsaved markers. Shared by
   *  the Save Session dialog and the Ctrl+S quiet re-save. */
  private async writeSession(name: string): Promise<void> {
    await saveDocument("session", name, JSON.stringify(this.workspace.snapshotSession()));
    this.lastSessionName = name;
    // Everything is captured in the named session — clear all unsaved markers.
    this.workspace.muteDirty();
    clearDirty();
    setStatus(`Session "${name}" saved`);
    recordProcess("Session", `Saved session "${name}"`);
  }

  /** Ctrl+S: re-save the current session in place if it already has a name, otherwise open
   *  the Save Session dialog to name it first. */
  private async quickSaveSession(): Promise<void> {
    if (this.lastSessionName) {
      try {
        await this.writeSession(this.lastSessionName);
      } catch (err) {
        setStatus(`Save failed: ${err}`);
      }
    } else {
      this.handleSaveSession();
    }
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
          this.lastSessionName = session.name; // Ctrl+S now re-saves this session in place
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
      const warned = results.filter((r) => r.warning);
      const warnNote = warned.length ? ` ${warned.length} well(s) had depth issues.` : "";
      setStatus(`Imported ${ok}/${results.length} well(s).${warnNote}`);
      recordProcess("Import", `Imported ${ok}/${results.length} LAS well(s)`);
      for (const w of warned) {
        recordProcess("Import", `${w.well_name ?? w.path}: ${w.warning}`, w.well_name ?? undefined);
      }
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
        recordProcess("Import", `Imported ${result.rows} core sample(s) ← ${path}`, well.well_name);
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
        const replacedNote = result.replaced > 0 ? ` (replaced ${result.replaced} existing curve(s))` : "";
        setStatus(`Imported ${result.curves_imported} curve(s), ${result.rows} samples into ${well.well_name}.${replacedNote}`);
        recordProcess(
          "Import",
          `Imported DLIS (${result.curves_imported} curves, ${result.rows} samples)${replacedNote} ← ${path}`,
          well.well_name,
        );
        this.workspace.notifyDataChanged();
      }
    } catch (err) {
      setStatus(`DLIS import failed: ${err}`);
    }
  }

  /** "Import SCAL…" — replaces the well's capillary-pressure (Pc/Sw) points from one or
   *  more files (flat CSV, porous-plate wide table, or per-plug centrifuge blocks) and
   *  fits the Leverett J-function, reporting SWH_A/SWH_B for the sw_height module. */
  private async handleImportScal(): Promise<void> {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    let paths: string[];
    try {
      const selection = await open({
        multiple: true,
        filters: [{ name: "SCAL Pc files", extensions: ["csv", "txt"] }],
      });
      paths = Array.isArray(selection) ? selection : selection ? [selection] : [];
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (paths.length === 0) return;

    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      `Imports capillary-pressure points from ${paths.length} file(s) — flat Pc/Sw tables, ` +
      "porous-plate wide tables (pressure columns × plug rows, cells = Sw %PV), or per-plug " +
      "centrifuge blocks (multi-select one file per plug) — then fits the Leverett J-function " +
      "Sw = A·J^B. The lab sigma·cosθ converts Pc to J: 72 air-brine, 367 air-mercury, " +
      "26 oil-brine. Carry the fitted A/B into SW — Saturation-Height (SWH_A/SWH_B). " +
      "One import = ONE lab fluid system: don't mix air-brine and mercury deliveries in a " +
      "single multi-select — their Pc scales differ and the pooled J-fit would be biased.";
    content.appendChild(doc);
    const fmtSel = document.createElement("select");
    fmtSel.className = "form-control";
    for (const [value, label] of [
      ["auto", "Auto-detect per file"],
      ["long", "Flat table (PC/SW columns)"],
      ["porous_plate", "Porous plate (wide, pressure columns)"],
      ["centrifuge", "Centrifuge (per-plug blocks)"],
    ] as const) {
      const o = document.createElement("option");
      o.value = value;
      o.textContent = label;
      fmtSel.appendChild(o);
    }
    content.appendChild(formRow("File format", fmtSel, "How each file's Pc/Sw points are laid out"));
    const sysSel = document.createElement("select");
    sysSel.className = "form-control";
    for (const [value, label] of [
      ["air_brine", "Air-brine (72)"],
      ["hg_air", "Air-mercury (367)"],
      ["oil_brine", "Oil-brine (26)"],
      ["other", "Other / custom"],
    ] as const) {
      const o = document.createElement("option");
      o.value = value;
      o.textContent = label;
      sysSel.appendChild(o);
    }
    content.appendChild(formRow("Fluid system", sysSel, "Stored on every imported point; ONE system per import"));
    const iftInput = document.createElement("input");
    iftInput.type = "number";
    iftInput.step = "0.1";
    iftInput.className = "form-control";
    iftInput.value = "72";
    sysSel.addEventListener("change", () => {
      const preset: Record<string, string> = { air_brine: "72", hg_air: "367", oil_brine: "26" };
      const v = preset[sysSel.value];
      // "Other" clears the field: a stale preset σcosθ silently stored on every point
      // would bias the J-fit — force an explicit entry instead.
      iftInput.value = v ?? "";
      if (!v) iftInput.focus();
    });
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
      const fmt = fmtSel.value as "auto" | "long" | "porous_plate" | "centrifuge";
      void importScalFiles(well.well_id, paths, fmt, sysSel.value, ift)
        .then((result) => {
          if (result.error) {
            resultBox.textContent = `SCAL import failed: ${result.error}`;
            return;
          }
          recordProcess("Import", `Imported SCAL Pc data (${fmt}) ← ${result.path}`, well.well_name);
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

  /** "Import Tops…" — formation tops from CSV/TXT. Multi-well files (a WELL column)
   *  match project wells by name; single-well files land in the selected well. */
  private async handleImportTops(): Promise<void> {
    const well = appState.selectedWell.get();
    let path: string | null;
    try {
      const selection = await open({
        multiple: false,
        filters: [{ name: "Tops CSV/TXT", extensions: ["csv", "txt", "asc", "dat"] }],
      });
      path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!path) return;
    try {
      const result = await importTopsCsv(well?.well_id ?? null, path);
      if (result.error) {
        setStatus(`Tops import failed: ${result.error}`);
        return;
      }
      const unmatched = result.unmatched_wells.length
        ? ` — unmatched well name(s): ${result.unmatched_wells.slice(0, 5).join(", ")}${result.unmatched_wells.length > 5 ? "…" : ""}`
        : "";
      setStatus(`Tops: ${result.tops_written} marker(s) across ${result.wells_matched} well(s)${unmatched}`);
      recordProcess(
        "Import",
        `Imported tops (${result.tops_written} markers, ${result.wells_matched} wells) ← ${path}`,
        well?.well_name,
      );
      this.workspace.notifyDataChanged();
    } catch (err) {
      setStatus(`Tops import failed: ${err}`);
    }
  }

  /** "Import Aux…" — petrography / XRD / perforation (tops-style CSV/TXT) for the
   *  selected well. Each import replaces the well's previous rows of that dataset. */
  private handleImportAux(): void {
    const well = appState.selectedWell.get();
    if (!well) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "Tops-style data: a TOP/DEPTH column (plus optional BASE/TO for intervals); every other " +
      "column becomes an item — mineral percentages, textural values, perforation status. " +
      "Re-importing a dataset replaces this well's previous rows of that dataset only.";
    content.appendChild(doc);

    const dsSelect = document.createElement("select");
    dsSelect.className = "form-control";
    for (const name of ["PETROGRAPHY", "XRD", "PERFORATION", "Custom…"]) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      dsSelect.appendChild(o);
    }
    content.appendChild(formRow("Dataset", dsSelect));
    const customInput = document.createElement("input");
    customInput.className = "form-control";
    customInput.type = "text";
    customInput.placeholder = "dataset name";
    const customRow = formRow("Custom name", customInput);
    customRow.style.display = "none";
    content.appendChild(customRow);
    dsSelect.addEventListener("change", () => {
      customRow.style.display = dsSelect.value === "Custom…" ? "" : "none";
    });

    const pick = document.createElement("button");
    pick.className = "form-run-btn";
    pick.textContent = "Choose file & import…";
    const resultBox = document.createElement("div");
    resultBox.className = "modal-result";
    content.appendChild(pick);
    content.appendChild(resultBox);
    openModal(`Import Aux Data — ${well.well_name}`, content, 460);

    pick.addEventListener("click", async () => {
      const dataset = dsSelect.value === "Custom…" ? customInput.value.trim() : dsSelect.value;
      if (!dataset) {
        resultBox.textContent = "Enter a dataset name.";
        return;
      }
      let path: string | null;
      try {
        const selection = await open({
          multiple: false,
          filters: [{ name: "Tops-style CSV/TXT", extensions: ["csv", "txt", "asc", "dat"] }],
        });
        path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
      } catch (err) {
        resultBox.textContent = `Import dialog unavailable: ${err}`;
        return;
      }
      if (!path) return;
      pick.disabled = true;
      resultBox.textContent = `Importing ${dataset} for ${well.well_name}…`;
      try {
        const result = await importAuxData(well.well_id, dataset, path);
        if (result.error) {
          resultBox.textContent = `Import failed: ${result.error}`;
          return;
        }
        resultBox.textContent = `Imported ${result.rows} value(s) across ${result.items.length} column(s): ${result.items.join(", ")}`;
        setStatus(`${result.dataset}: ${result.rows} values imported for ${well.well_name}`);
        recordProcess("Import", `Imported ${result.dataset} (${result.rows} values) ← ${path}`, well.well_name);
        this.workspace.notifyDataChanged();
      } catch (err) {
        resultBox.textContent = `Import failed: ${err}`;
      } finally {
        pick.disabled = false;
      }
    });
  }

  /** "Import Deviation…" — loads an MD/INC/AZI survey CSV and computes minimum-curvature
   *  TVD/TVDSS for the selected well. Prompts for the datum (KB) elevation. */
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
            recordProcess("Import", `Imported deviation survey (${result.rows} stations) ← ${path}`, well.well_name);
            this.workspace.notifyDataChanged();
            close();
          }
        })
        .catch((err) => setStatus(`Deviation import failed: ${err}`));
    });
    datumInput.focus();
  }

  /** "Import Well Locations…" — surface easting/northing (+optional per-row zone) from a
   *  CSV/TXT for the Field Map. A WELL column locates every matching well; without one the
   *  file locates the selected well. The chosen UTM zone (Indonesia spans 46–54, N/S) fills
   *  rows that carry no ZONE column. */
  private handleImportWellLocations(): void {
    const content = document.createElement("div");
    const doc = document.createElement("p");
    doc.className = "modal-doc";
    doc.textContent =
      "CSV/TXT with EASTING/NORTHING (aliases X/Y, UTM_X/UTM_Y) columns; a WELL column locates " +
      "every matching well, otherwise the file locates the selected well. A per-row ZONE column " +
      "overrides the default zone below.";
    content.appendChild(doc);

    const zoneSel = document.createElement("select");
    zoneSel.className = "form-control";
    // Indonesian acreage runs across UTM zones 46–54, mostly southern hemisphere
    // (Mahakam 50S; ONWJ 48S/49S) with the north straddling the equator.
    for (const hemi of ["S", "N"]) {
      for (let z = 46; z <= 54; z++) {
        const o = document.createElement("option");
        o.value = `${z}${hemi}`;
        o.textContent = `UTM ${z}${hemi}`;
        zoneSel.appendChild(o);
      }
    }
    zoneSel.value = "50S"; // Mahakam Delta default
    content.appendChild(formRow("Default UTM zone", zoneSel, "Applied to rows without a ZONE column"));

    const pick = document.createElement("button");
    pick.className = "form-run-btn";
    pick.textContent = "Choose file & import…";
    const resultBox = document.createElement("div");
    resultBox.className = "modal-result";
    content.appendChild(pick);
    content.appendChild(resultBox);
    openModal("Import Well Locations", content, 480);

    pick.addEventListener("click", async () => {
      const well = appState.selectedWell.get();
      const zone = zoneSel.value;
      let path: string | null;
      try {
        const selection = await open({
          multiple: false,
          filters: [{ name: "Locations CSV/TXT", extensions: ["csv", "txt", "asc", "dat"] }],
        });
        path = Array.isArray(selection) ? (selection[0] ?? null) : selection;
      } catch (err) {
        resultBox.textContent = `Import dialog unavailable: ${err}`;
        return;
      }
      if (!path) return;
      pick.disabled = true;
      resultBox.textContent = "Importing locations…";
      try {
        const result = await importWellLocations(well?.well_id ?? null, zone, path);
        if (result.error) {
          resultBox.textContent = `Locations import failed: ${result.error}`;
          return;
        }
        const unmatched = result.unmatched_wells.length
          ? ` — unmatched: ${result.unmatched_wells.slice(0, 5).join(", ")}${result.unmatched_wells.length > 5 ? "…" : ""}`
          : "";
        resultBox.textContent = `Located ${result.wells_located} well(s)${unmatched}. Open Field Map to view.`;
        setStatus(`Locations: ${result.wells_located} well(s) placed${unmatched}`);
        recordProcess("Import", `Imported well locations (${result.wells_located} wells) ← ${path}`, well?.well_name);
        this.workspace.notifyDataChanged();
      } catch (err) {
        resultBox.textContent = `Locations import failed: ${err}`;
      } finally {
        pick.disabled = false;
      }
    });
  }

  /** "Well Header…" — edits the selected well's field / TD / KB datum (Phase 6c). */
  private async handleWellHeader(): Promise<void> {
    const selected = appState.selectedWell.get();
    if (!selected) {
      setStatus("Select a well first (Wells & Tops panel)");
      return;
    }
    // `selectedWell` is a snapshot captured on tree-click and is NOT re-broadcast on a
    // dataVersion bump, so after an import (or a prior header save) it can carry stale
    // (often null) coordinates. Re-read the well from the DB so the X/Y/zone fields show
    // current values — otherwise the unconditional coordinate writes below would clobber
    // a just-imported/entered location with the stale snapshot.
    const fresh = await listWells()
      .then((ws) => ws.find((w) => w.well_id === selected.well_id))
      .catch(() => undefined);
    const well = fresh ?? selected;
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
    if (well.td != null) tdInput.value = String(well.td);
    content.appendChild(formRow("TD (m)", tdInput));

    const kbInput = document.createElement("input");
    kbInput.type = "number";
    kbInput.step = "0.1";
    kbInput.className = "form-control";
    kbInput.placeholder = "KB elevation (m)";
    // Show the CURRENT KB — it silently drives TVDSS in deviation import, so editing it blind
    // (the old behaviour: always-empty field) risked poisoning every TVDSS.
    if (well.kb != null) kbInput.value = String(well.kb);
    content.appendChild(formRow("KB (m)", kbInput, "datum for TVDSS"));

    // Surface location for the Field Map — the manual-entry counterpart to the CSV importer.
    const xInput = document.createElement("input");
    xInput.type = "number";
    xInput.step = "0.01";
    xInput.className = "form-control";
    xInput.placeholder = "easting (m)";
    if (well.surface_x != null) xInput.value = String(well.surface_x);
    content.appendChild(formRow("Surface X", xInput, "UTM easting"));

    const yInput = document.createElement("input");
    yInput.type = "number";
    yInput.step = "0.01";
    yInput.className = "form-control";
    yInput.placeholder = "northing (m)";
    if (well.surface_y != null) yInput.value = String(well.surface_y);
    content.appendChild(formRow("Surface Y", yInput, "UTM northing"));

    const zoneInput = document.createElement("input");
    zoneInput.type = "text";
    zoneInput.className = "form-control";
    zoneInput.placeholder = "e.g. 50S";
    if (well.utm_zone != null) zoneInput.value = well.utm_zone;
    content.appendChild(formRow("UTM zone", zoneInput));

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
        updateWellField(well.well_id, "surface_x", xInput.value.trim() || null),
        updateWellField(well.well_id, "surface_y", yInput.value.trim() || null),
        updateWellField(well.well_id, "utm_zone", zoneInput.value.trim() || null),
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
