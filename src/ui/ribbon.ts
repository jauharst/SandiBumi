import { open, save } from "@tauri-apps/plugin-dialog";
import {
  exportLas,
  importDeviationCsv,
  materializeTvd,
  importScalFiles,
  importTopsCsv,
  importWellLocations,
  importDlisFile,
  importLasFiles,
  currentProject,
  bootReport,
  compactProject,
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
import { syncDepthUnits } from "../depthUnitPref";
import { clearUndoStacks, nextRedoLabel, nextUndoLabel, onUndoChange, pushUndo, redo, redoDepth, undo, undoDepth } from "../undo";
import { recordProcess } from "../processLog";
import { getTheme, setTheme, type ThemeChoice } from "../theme";
import { getLocale, setLocale, type Locale } from "../i18n";
import type { SessionSnapshot, Workspace } from "./workspace";
import { buildFollowCoreRow } from "./followCore";
import { formRow, openModal } from "./modal";
import { openImportSetDialog, suggestSetName } from "./importSetDialog";
import { openCoreImportWizard } from "./coreImportDialog";
import { openImageImportDialog } from "./imageImportDialog";
import { openDataSetsDialog } from "./dataSetsDialog";
import { openWorkbookDialog } from "./workbookDialog";
import { openDeckDialog } from "./deckDialog";
import { requireWell } from "./needWell";

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

    // --- Project ▸ Edit (Undo / Redo) ---
    // These were the icon-only quick-access strip until 2026-07-30; they are labelled
    // ribbon tools in the Project tab now. The keyboard shortcuts are unchanged and are
    // still the fast path — the buttons exist so the action is discoverable and readable.
    const undoBtn = q<HTMLButtonElement>("#undo-btn");
    const redoBtn = q<HTMLButtonElement>("#redo-btn");
    undoBtn?.addEventListener("click", () => {
      void undo().then(
        (label) => setStatus(label ? `Undo: ${label}` : "Nothing to undo"),
        (err) => setStatus(`Undo failed — the change was not undone: ${err}`),
      );
    });
    redoBtn?.addEventListener("click", () => {
      void redo().then(
        (label) => setStatus(label ? `Redo: ${label}` : "Nothing to redo"),
        (err) => setStatus(`Redo failed — the change was not reapplied: ${err}`),
      );
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
    // --- Project ▸ Session ---
    const saveSessionBtn = q<HTMLButtonElement>("#save-session-btn");
    saveSessionBtn?.addEventListener("click", () => this.handleSaveSession());
    q<HTMLButtonElement>("#open-session-btn")?.addEventListener("click", () => void this.handleOpenSession());

    // --- Project ▸ Monitor --- (History + Processing + Performance all watch the whole
    // application rather than a petrophysics run, so they share one group.)
    q<HTMLButtonElement>("#history-btn")?.addEventListener("click", () => workspace.openHistory());
    q<HTMLButtonElement>("#processing-btn")?.addEventListener("click", () => workspace.openProcessing());
    q<HTMLButtonElement>("#health-btn")?.addEventListener("click", () => workspace.openHealth());
    // Contextual Help (?): opens a guide for whichever panel is active — the future hook for
    // the illustrated HTML help library, keyed to the "current active panel".
    q<HTMLButtonElement>("#help-btn")?.addEventListener("click", () => void workspace.openHelpForActivePanel());
    // Unsaved-state dot: lights while any panel/workspace state isn't in a named save yet.
    // It is mirrored onto the PROJECT TAB as well, because Save Session… now lives inside
    // that tab: a warning you only see after opening the tab that holds the fix is no
    // warning at all. The tab dot is what keeps the signal visible from anywhere.
    const projectTab = root.querySelector<HTMLElement>('.ribbon-tab[data-tab="project"]');
    if (saveSessionBtn) {
      const baseTitle = saveSessionBtn.title;
      subscribeDirty(() => {
        const dirty = anyDirty();
        saveSessionBtn.classList.toggle("ribbon-btn-dirty", dirty);
        saveSessionBtn.title = dirty ? `${baseTitle} — unsaved changes` : baseTitle;
        projectTab?.classList.toggle("ribbon-tab-dirty", dirty);
        if (projectTab) {
          projectTab.title = dirty ? "Unsaved changes — Project ▸ Session ▸ Save Session…" : "";
        }
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
    // The Workflow Builder fires this when a chain starts so the universal Processing panel
    // pops open on its own — the user shouldn't have to hunt for progress. Ribbon is a
    // singleton created once in main.ts, so this window listener is registered exactly once.
    window.addEventListener("sandibumi:open-processing", () => workspace.openProcessing());
    // The start sheet's recent-project rows route through the SAME switchProject guard the
    // Recent ▾ menu uses — a busy chain blocks a switch there exactly as it does here.
    window.addEventListener("sandibumi:open-recent-project", (e) => {
      const path = (e as CustomEvent<string>).detail;
      if (typeof path === "string" && path) void this.switchProject(() => openProject(path));
    });
    q<HTMLButtonElement>("#montecarlo-btn")?.addEventListener("click", () => workspace.openMonteCarlo());
    q<HTMLButtonElement>("#ml-btn")?.addEventListener("click", () => workspace.openMl());
    // Every tool below opens as a WORKING PANE (Jauhar, 2026-08-01: "dont use pop up for future
    // except my request"). These are all worked through iteratively — plate by plate, barrel by
    // barrel, fit after fit — against the Wells pane and the log views a modal would cover.
    // Same workspace the core photographs use — a thin section arrives with the same problems, and
    // two dialogs would be two places for the wording and the white-balance rule to drift.
    q<HTMLButtonElement>("#plate-condition-btn")?.addEventListener("click", () =>
      workspace.openCoreCondition("plate"),
    );
    q<HTMLButtonElement>("#pore-area-btn")?.addEventListener("click", () => workspace.openPoreArea());
    q<HTMLButtonElement>("#mineral-class-btn")?.addEventListener("click", () => workspace.openMineralClass());
    // Moved out of the Data ▸ Tools ▾ dropdown onto their own ribbon groups (Jauhar,
    // 2026-08-01): core depth work is one job done in sequence, core photographs are an
    // interpretation method, and plate details belong with the rest of petrography.
    q<HTMLButtonElement>("#plate-details-btn")?.addEventListener("click", () => workspace.openPlateDetails());
    q<HTMLButtonElement>("#register-depth-btn")?.addEventListener("click", () => workspace.openDepthReg());
    q<HTMLButtonElement>("#condition-core-btn")?.addEventListener("click", () =>
      workspace.openCoreCondition("core"),
    );
    q<HTMLButtonElement>("#core-trace-btn")?.addEventListener("click", () => workspace.openCoreTrace());
    q<HTMLButtonElement>("#fluid-contacts-btn")?.addEventListener("click", () =>
      workspace.openFluidContacts(),
    );
    q<HTMLButtonElement>("#shift-core-btn")?.addEventListener("click", () => this.handleShiftCore());
    q<HTMLButtonElement>("#data-sets-btn")?.addEventListener("click", () => this.handleDataSets());
    q<HTMLButtonElement>("#plug-qc-btn")?.addEventListener("click", () => workspace.openPlugQc());
    q<HTMLButtonElement>("#multimin-btn")?.addEventListener("click", () => workspace.openMultimin());
    q<HTMLButtonElement>("#rtc-fit-btn")?.addEventListener("click", () => workspace.openRtcFit());
    q<HTMLButtonElement>("#sfactor-fit-btn")?.addEventListener("click", () => workspace.openSFactorFit());
    q<HTMLButtonElement>("#intake-btn")?.addEventListener("click", () => workspace.openIntake());
    q<HTMLButtonElement>("#statistics-btn")?.addEventListener("click", () => workspace.openStatistics());
    q<HTMLButtonElement>("#reframe-btn")?.addEventListener("click", () => workspace.openReframe());
    q<HTMLButtonElement>("#dashboard-btn")?.addEventListener("click", () => workspace.openDashboard());
    q<HTMLButtonElement>("#results-qc-btn")?.addEventListener("click", () => workspace.openResultsQc());
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
    q<HTMLButtonElement>("#vega-btn")?.addEventListener("click", () => workspace.openPlot("vega"));
    q<HTMLButtonElement>("#pickett-btn")?.addEventListener("click", () => workspace.openPlot("pickett"));
    q<HTMLButtonElement>("#correlation-btn")?.addEventListener("click", () => workspace.openPlot("correlation"));
    q<HTMLButtonElement>("#composite-btn")?.addEventListener("click", () => workspace.openComposite());
    q<HTMLButtonElement>("#report-btn")?.addEventListener("click", () => workspace.openReport());
    q<HTMLButtonElement>("#workbook-btn")?.addEventListener("click", () => void openWorkbookDialog());
    q<HTMLButtonElement>("#deck-btn")?.addEventListener("click", () => void openDeckDialog());
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
          label: "Import Images…",
          doc: "Import thin-section, core-photo or SEM pictures for the selected well — the depth in each file name is guessed and shown for confirmation before anything is stored",
          onPick: () => void this.handleImportImages(),
        },
        {
          label: "Import Deviation…",
          doc: "Import a deviation survey (MD/INC/AZI CSV) and compute TVD/TVDSS for the selected well",
          onPick: () => void this.handleImportDeviation(),
        },
        {
          label: "Recompute TVD/TVDSS Curves",
          doc: "Rebuild TVD/TVDSS curves from every well's deviation survey onto its log grid — run after importing logs later or editing a well's KB (deviation import does this automatically)",
          onPick: () => void this.handleMaterializeTvd(),
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
          label: "Well Header…",
          doc: "Edit the selected well's header (field, TD, KB datum)",
          onPick: () => void this.handleWellHeader(),
        },
        {
          label: "Compact Project…",
          doc: "Rewrite the project file keeping only live data — module re-runs leave dead space the file never returns (a field project can carry 4× its true size). The original file is kept beside it until you delete it",
          onPick: () => void this.handleCompactProject(),
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
   *  dropdown and given no Advance button: it is superseded by SandiMin (the generalized
   *  solver) and Jauhar asked for mineral inversion to be independent of Sw. It is now RETIRED —
   *  a saved workflow chain that still references it resolves by name but fails on run with a
   *  message directing to SandiMin (backend `modules::retired_module`), rather than running. */
  private static readonly ADVANCED_MODULE_IDS = ["ssc", "sspw", "sw_rtc", "sw_imts", "thin_bed_ts", "multimin"] as const;

  /** Modules superseded by a more general one: still in the catalog and still RUNNABLE, so every
   *  saved chain and every stored run resolves, but kept out of the pickers so the user is not
   *  offered the same operation twice (Jauhar, 2026-08-05: *"dont dupilcates, normalize tools here
   *  should be universal for all logs"*). `gr_normalize` is now a preset of Condition ▸ Normalize
   *  and delegates to the same code. Distinct from ADVANCED_MODULE_IDS, which MOVES a module to
   *  the Advance tab, and from the retirement list in `modules.rs`, which stops one running. */
  private static readonly SUPERSEDED_MODULE_IDS = ["gr_normalize"] as const;

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
      // Assign scrollLeft directly rather than scrollBy({behavior:"smooth"}): in the WebView
      // ANY smooth scroll on this element is silently a no-op (measured — scrollBy smooth and
      // a scrollLeft assignment under CSS `scroll-behavior: smooth` both leave scrollLeft at
      // 0; a plain assignment moves it correctly). The chevrons used to appear and do nothing.
      // Keep this unanimated — do not "restore" smooth scrolling here or in .ribbon-panel.
      const max = p.scrollWidth - p.clientWidth;
      const step = dir * Math.max(120, p.clientWidth * 0.7);
      p.scrollLeft = Math.min(max, Math.max(0, p.scrollLeft + step));
      update(); // scroll events are async; refresh the chevrons now so neither sticks
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
    const hidden = new Set<string>([...Ribbon.ADVANCED_MODULE_IDS, ...Ribbon.SUPERSEDED_MODULE_IDS]);
    modules = modules.filter((spec) => !hidden.has(spec.name));
    container.innerHTML = "";

    // category id -> [dropdown label, group caption, icon path data]
    const CATEGORIES: Record<string, [string, string, string]> = {
      Prep: [
        "Data Prep",
        "Data Cond & Prep",
        "M5 15c1.5-3 2-8 5-8s3.5 5 5 8M4 11h3M13 11h3",
      ],
      // Condition — the curve-conditioning family (despike, smooth, clip, fill gaps, flip). Its
      // own group rather than more entries in Data Prep: these act on ANY curve and are run
      // before the interpretation starts, where Prep's members already assume a petrophysical
      // role for their inputs. `Frame` (block, resample, regularize) is the companion category.
      Condition: [
        "Condition",
        "Curve Conditioning",
        "M3 13c2-6 3.2 3 5-4s2.6 6 4.4 1S16 12 17 10",
      ],
      // Frame — depth sampling: upscaling a curve to beds, and finding the beds. Separate from
      // Condition because conditioning changes a curve's VALUES while leaving every sample where
      // it was, and blocking changes which samples say anything at all.
      Frame: [
        "Frame",
        "Depth Sampling",
        "M3 5h14M3 9h14M3 13h14M3 17h14M7 3v16M13 3v16",
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
      Lithology: [
        "Lithology",
        "Lithology",
        "M3 15l4-7 3 4 2.5-5L17 15ZM3 15h14",
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
      Unconventional: [
        "Unconventional",
        "Unconventional",
        "M5 16a1.4 1.4 0 1 0 0-.01M9 12a1.4 1.4 0 1 0 0-.01M13 8a1.4 1.4 0 1 0 0-.01M3 18h14",
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
      const items = specs.map((spec) => ({
        label: spec.title,
        doc: spec.doc,
        onPick: () => this.openModule(spec),
      }));
      // The Unconventional group also carries the visual companion to its compute methods —
      // the ΔlogR overlay + Langmuir isotherm panel (a workspace pane, not a module form).
      if (category === "Unconventional") {
        items.push({
          label: "ΔlogR + Langmuir Visuals…",
          doc: "Passey ΔlogR resistivity/porosity overlay and the Langmuir adsorption isotherm — the pictures behind toc_passey and gip.",
          onPick: () => this.workspace.openUnconventional(),
        });
      }
      const group = document.createElement("div");
      group.className = "ribbon-group";
      group.appendChild(buildRibbonDropdown(label, iconPath, items));
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
    setStatus("Opening project… (a first open after an update can run one-time storage upgrades — a large project may take minutes)");
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
    // A different project can be in a different depth unit. Re-read it BEFORE panels
    // reload — a stale unit would mislabel every depth and skew the 1:N print scale.
    await syncDepthUnits().catch(() => {});
    await syncWellGroups().catch(() => {});
    this.reflectProject(info, true);
    this.workspace.notifyDataChanged();
    recordProcess("Project", `Opened project ${info.name} (${info.path})`);
    setStatus(`Project: ${info.name}`);
    // Anything noteworthy the open did (one-time migration backups, a slow open
    // explained) goes into the history — and the status line, so it isn't silent.
    void bootReport()
      .then((notes) => {
        for (const n of notes) recordProcess("Project", n);
        const visible = notes.filter((n) => !n.startsWith("DuckDB memory"));
        if (visible.length > 0) setStatus(visible[visible.length - 1]);
      })
      .catch(() => {});
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
    const well = requireWell("Export LAS");
    if (!well) return;
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
      const result = await exportLas(well.well_id, dest);
      const omission = result.omitted.length
        ? ` Omitted ${result.omitted.map((item) => `${item.curve}: ${item.reason}`).join("; ")}.`
        : "";
      const summary = `${result.rows} rows; ${result.curves_written} of ${result.curves_held} held curves written.`;
      const precision = result.precision.reduced
        ? ` Precision: ${result.precision.values_reduced} value(s) reduced, ${result.precision.source_precision} → ${result.precision.destination_precision}.`
        : ` Precision: no values reduced, ${result.precision.source_precision} → ${result.precision.destination_precision}.`;
      const selfCheck = result.self_checked ? " SandiBumi reader self-check passed." : "";
      const finalCount = result.curve_states.filter((curve) => curve.state === "final").length;
      const workingCount = result.curve_states.filter((curve) => curve.state === "working").length;
      const states = result.curve_states.length
        ? ` Curve states: ${finalCount} final, ${workingCount} working.`
        : "";
      setStatus(`Exported ${well.well_name} (${summary}) to ${dest}.${precision}${selfCheck}${states}${omission}`);
      recordProcess("Export", `Exported LAS (${summary})${precision}${selfCheck}${states}${omission} → ${dest}`, well.well_name);
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

    // Which curve set does this delivery land under, and should same-named files attach to
    // the wells already in the project? (T-IMP-02 — the Geolog/IP set model.)
    const choice = await openImportSetDialog(paths);
    if (!choice) {
      setStatus("Import cancelled");
      return;
    }

    const setLabel = choice.setName ? choice.setName.toUpperCase().replace(/\s+/g, "_") : "RAW";
    setStatus(`Importing ${paths.length} LAS file(s) as set ${setLabel}...`);
    try {
      const results = await importLasFiles(paths, choice);
      // Partition on `well_id`, which is set only when a well row was actually committed.
      // `!r.error` used to stand in for "imported", but cancelling an import now returns an entry
      // with neither a well nor an error — so every cancelled file counted as imported, and
      // "Imported 120/120" was written to the permanent History for a run that stopped at 75.
      const imported = results.filter((r) => r.well_id);
      const failed = results.filter((r) => r.error);
      const cancelled = results.length - imported.length - failed.length;
      // Files that landed on an EXISTING well as a new set rather than creating a record.
      // Reported separately because "Imported 544 wells" would be a lie when 544 files
      // attached to 544 wells that were already there — the count of NEW wells is what
      // changed, and the attach count is what the user asked the dialog to do.
      const attached = imported.filter((r) => r.attached_set);
      const created = imported.length - attached.length;
      const attachNote = attached.length
        ? ` ${attached.length} attached to existing well(s) as set ${
            [...new Set(attached.map((r) => r.attached_set))].join(", ")
          }.`
        : "";
      // Warnings belong to wells that DID import: "depth issues" was accurate when depth
      // sanitising was the only note, but it now also carries duplicate-name and failed
      // full-curve-load warnings, so name the count and let the per-well notes say what happened.
      // A cancelled file carries a note too; it is reported in the aggregate instead.
      const warned = imported.filter((r) => r.warning);
      const warnNote = warned.length ? ` ${warned.length} well(s) imported with warnings.` : "";
      const cancelNote = cancelled > 0 ? ` ${cancelled} cancelled before import.` : "";
      const encodingCounts = new Map<string, number>();
      for (const result of imported) {
        if (result.text_encoding) {
          encodingCounts.set(result.text_encoding, (encodingCounts.get(result.text_encoding) ?? 0) + 1);
        }
      }
      const encodingSummary = [...encodingCounts]
        .map(([encoding, count]) => `${encoding} (${count})`)
        .join(", ");
      setStatus(
        `Imported ${imported.length}/${results.length} file(s) as set ${setLabel}` +
          ` — ${created} new well(s).${attachNote}${warnNote}${cancelNote}`,
      );
      recordProcess(
        "Import",
        `Imported ${imported.length}/${results.length} LAS file(s) as set ${setLabel}: ` +
          `${created} new well(s), ${attached.length} attached` +
          (cancelled > 0 ? ` — ${cancelled} cancelled` : ""),
      );
      if (encodingSummary) {
        recordProcess("Import", `Text encodings detected: ${encodingSummary}`);
      }
      for (const w of warned) {
        recordProcess("Import", `${w.well_name ?? w.path}: ${w.warning}`, w.well_name ?? undefined);
      }
      this.workspace.notifyDataChanged();
    } catch (err) {
      setStatus(`Import failed: ${err}`);
    }
  }

  /** "Import Core…" — core import v2 (T-IMP-07): probe → confirm-mapping wizard →
   *  commit. Multi-file and multi-well: files with a WELL/WN column route rows by name
   *  (no well needs to be selected); files without one land on the selected well. CSV
   *  and TXT/tab-delimited both accepted. */
  private async handleImportCore(): Promise<void> {
    const well = appState.selectedWell.get();
    let paths: string[] | null;
    try {
      const selection = await open({
        multiple: true,
        filters: [{ name: "Core data (CSV/TXT)", extensions: ["csv", "txt", "dat"] }],
      });
      paths = Array.isArray(selection) ? selection : selection ? [selection] : null;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!paths || paths.length === 0) return;

    await openCoreImportWizard(paths, well, () => this.workspace.notifyDataChanged());
  }

  /** "Import Images…" — depth-registered pictures (thin sections, core photographs, SEM
   *  plates) for the selected well. The wizard shows the depth guessed from each file name
   *  and only writes once it is confirmed. */
  private async handleImportImages(): Promise<void> {
    const well = appState.selectedWell.get();
    let paths: string[] | null;
    try {
      const selection = await open({
        multiple: true,
        filters: [
          // A petrography delivery usually arrives as a WORKBOOK, one worksheet per plate, rather
          // than as loose files — so it is offered first. .xls is listed on purpose: selecting one
          // gets a named refusal with the fix rather than a picker that ignores it.
          {
            name: "Plates and petrography workbooks",
            extensions: ["xlsx", "xlsm", "xls", "jpg", "jpeg", "png", "tif", "tiff", "bmp", "gif", "webp"],
          },
          { name: "Petrography workbook", extensions: ["xlsx", "xlsm", "xls"] },
          { name: "Images", extensions: ["jpg", "jpeg", "png", "tif", "tiff", "bmp", "gif", "webp"] },
        ],
      });
      paths = Array.isArray(selection) ? selection : selection ? [selection] : null;
    } catch (err) {
      setStatus(`Import dialog unavailable: ${err}`);
      return;
    }
    if (!paths || paths.length === 0) return;

    await openImageImportDialog(paths, well, () => this.workspace.notifyDataChanged());
  }

  /** "Data Sets…" — every delivery on the selected well (core, SCAL, surveys, point data):
   *  which one is live, switch, or delete (T-IMP-08 / T-IMP-12). */
  private handleDataSets(): void {
    const well = requireWell("Data Sets");
    if (!well) return;
    openDataSetsDialog(well, () => this.workspace.notifyDataChanged());
  }

  /** "Compact Project…" — rewrite the project file with only live rows. Asks first (the
   *  app is briefly unresponsive while gigabytes rewrite), reports old → new size, and
   *  names the parked original so the user can reclaim the disk once satisfied. */
  private async handleCompactProject(): Promise<void> {
    const ok = window.confirm(
      "Compact this project?\n\n" +
        "The project file is rewritten keeping only live data — re-running modules leaves dead space " +
        "behind, and a long-lived field project can carry several times its true size. " +
        "The app will be busy for a while on a large project.\n\n" +
        "The original file is kept beside the project until you delete it yourself.",
    );
    if (!ok) return;
    setStatus("Compacting project — this can take a few minutes on a large project…");
    try {
      const rep = await compactProject();
      const mb = (b: number) => `${Math.round(b / 1048576).toLocaleString()} MB`;
      const line = `Compacted: ${mb(rep.bytes_before)} → ${mb(rep.bytes_after)}. Original kept as ${rep.old_file} — delete it once you are happy.`;
      setStatus(line);
      recordProcess("Project", line);
    } catch (err) {
      setStatus(`Compact failed: ${err}`);
    }
  }

  /** "Shift Core…" — constant core-to-log depth shift for the ACTIVE core set's plugs
   *  (other deliveries of the well keep their own depths).
   *  Exactly reversible, so it lands on the undo stack (Ctrl+Z shifts back). */
  private handleShiftCore(): void {
    const well = requireWell("Shift Core");
    if (!well) return;
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
    const doShift = async (delta: number, kind = "manual"): Promise<void> => {
      // No dataset list = the point data delivered with this core rides along, so the plugs and
      // the measurements made on them never part company.
      //
      // A typed shift goes into the depth record too, marked "manual": next year the question is
      // "why is this core here?", and "somebody typed it" is a real answer — a blank is not.
      const n = await shiftCoreData(well.well_id, delta, undefined, {
        kind,
        note: "typed in Shift Core",
      });
      const sign = delta > 0 ? "+" : "";
      setStatus(
        `Shifted ${n.plugs} core plug(s) and ${n.extras} point sample(s) of ${well.well_name} by ${sign}${delta} m`
      );
      recordProcess("Edit", `Core shift ${sign}${delta} m (${n.plugs} plugs, ${n.extras} point samples)`, well.well_name);
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
            undo: () => void doShift(-delta, "undo"),
            redo: () => void doShift(delta),
          });
          close();
        })
        .catch((err) => setStatus(`Core shift failed: ${err}`));
    });
    input.focus();
  }

  /** "Import DLIS…" — loads scalar channels from a DLIS file through the dlisio subprocess.
   *  A single-well file targets the selected project well; a multi-well container proposes
   *  separate project wells and requires its mapping to be confirmed before any write. The set-name
   *  prompt (T-IMP-06) means a second DLIS never silently replaces the first: you rarely
   *  know what a vendor tape holds until it is in, so duplicates are KEPT under their own
   *  set and compared afterwards. */
  private async handleImportDlis(): Promise<void> {
    const well = appState.selectedWell.get();
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

    const choice = await this.askDlisSetName(path);
    if (choice === null) {
      setStatus("Import cancelled");
      return;
    }
    const setName = choice.setName;
    const setLabel = setName ? setName.toUpperCase().replace(/\s+/g, "_") : "RAW";

    setStatus(
      well
        ? `Importing DLIS into ${well.well_name} as set ${setLabel}… (dlisio may take a moment)`
        : `Inspecting DLIS well identities before import… (dlisio may take a moment)`,
    );
    try {
      let intervalDecision: "accept_outside_declared_interval" | null = null;
      let duplicateDecisions: Array<{
        mnemonic: string;
        run: number;
        action: "keep_separate" | "skip_incoming";
      }> | null = null;
      let confirmedWellMappings: Awaited<ReturnType<typeof importDlisFile>>["well_mappings"] | null = null;
      let result: Awaited<ReturnType<typeof importDlisFile>>;
      for (;;) {
        result = await importDlisFile(
          well?.well_id ?? null,
          path,
          setName,
          choice.fileDepthUnit,
          null,
          intervalDecision,
          duplicateDecisions,
          choice.lasSentinelExceptions,
          confirmedWellMappings,
        );
        if (result.error && result.mapping_confirmation_required) {
          const detail = result.well_mappings
            .map(
              (mapping) =>
                `${mapping.source_well} (logical files ${mapping.logical_files.join(", ")}) ` +
                `→ ${mapping.will_create ? "new project well" : "project well"} ${mapping.target_well_name}`,
            )
            .join("\n");
          const accepted = window.confirm(
            `This DLIS contains more than one source well. Nothing has been written.\n\n${detail}\n\n` +
              "Create these separate project wells and import each logical file only into its mapped well?",
          );
          if (!accepted) {
            setStatus("Multi-well DLIS import cancelled before any well or curve was written.");
            return;
          }
          confirmedWellMappings = result.well_mappings;
          setStatus(`Well mapping confirmed; importing ${confirmedWellMappings.length} separate DLIS wells…`);
          continue;
        }
        if (result.error && result.duplicate_conflicts.length > 0 && duplicateDecisions === null) {
          const detail = result.duplicate_conflicts
            .map(
              (conflict) =>
                `${conflict.mnemonic} frame ${conflict.run}: ${conflict.existing.join(", ")}`,
            )
            .join("\n");
          const keep = window.confirm(
            `This well already holds the following DLIS mnemonic(s):\n\n${detail}\n\n` +
              "Keep every incoming curve separate in this delivery? Cancel writes nothing. " +
              "No merge-into-existing action is available.",
          );
          if (!keep) {
            setStatus("DLIS import stopped at duplicate mnemonics; no existing curve was changed.");
            return;
          }
          duplicateDecisions = result.duplicate_conflicts.map((conflict) => ({
            mnemonic: conflict.mnemonic,
            run: conflict.run,
            action: "keep_separate",
          }));
          setStatus(`Duplicate choices recorded; checking DLIS intervals for ${well?.well_name ?? "the selected well"}…`);
          continue;
        }
        if (result.error && result.interval_conflicts.length > 0 && intervalDecision === null) {
          const detail = result.interval_conflicts
            .map(
              (conflict) =>
                `${conflict.scope} ${conflict.name}: ${conflict.declared_top}-${conflict.declared_base}; ` +
                `incoming ${conflict.incoming_top}-${conflict.incoming_base}`,
            )
            .join("\n");
          const accepted = window.confirm(
            `This DLIS falls outside an existing declared interval:\n\n${detail}\n\n` +
              "Import those outside samples anyway? Nothing has been written yet.",
          );
          if (!accepted) {
            setStatus("DLIS import stopped at the interval conflict; the existing range is unchanged.");
            return;
          }
          intervalDecision = "accept_outside_declared_interval";
          setStatus(`Interval conflict accepted; importing DLIS into ${well?.well_name ?? "the selected well"}…`);
          continue;
        }
        break;
      }
      const skippedNote = result.skipped.length
        ? ` Skipped ${result.skipped.map((item) => `${item.kind} ${item.name} ×${item.count}: ${item.rule}`).join("; ")}.`
        : "";
      if (result.error) {
        setStatus(`DLIS import failed: ${result.error}.${skippedNote}`);
      } else {
        const resultNotes = [...result.notes];
        if (result.sentinel_exceptions.length > 0) {
          resultNotes.push(
            `LAS-sentinel fallback disabled for ${result.sentinel_exceptions.join(", ")}`,
          );
        }
        const unitNote = resultNotes.length ? ` ${resultNotes.join("; ")}` : "";
        const outcome = result.status === "partial" ? "Partially imported" : "Imported";
        const channelCount = result.channels_declared > 0
          ? ` of ${result.channels_declared} declared channel(s)`
          : "";
        const destination = result.well_mappings.length > 0
          ? `${result.well_mappings.length} separately mapped project wells`
          : well?.well_name ?? "the selected project well";
        setStatus(
          `${outcome} ${result.curves_imported}${channelCount}, ${result.rows} samples into ${destination} as set ${setLabel}.${unitNote}${skippedNote}`,
        );
        recordProcess(
          "Import",
          `${outcome} DLIS as set ${setLabel} (${result.curves_imported}${channelCount}, ${result.rows} samples)${unitNote}${skippedNote} ← ${path}`,
          result.well_mappings.length > 0 ? null : well?.well_name ?? null,
        );
        this.workspace.notifyDataChanged();
      }
    } catch (err) {
      setStatus(`DLIS import failed: ${err}`);
    }
  }

  /** Set-name and file-level choices collected before the DLIS scan. A single-well file still
   *  requires a selected target; a multi-well file obtains its targets from the confirmed map. */
  private askDlisSetName(path: string): Promise<{
    setName: string;
    fileDepthUnit: "M" | "FT" | null;
    lasSentinelExceptions: string[];
  } | null> {
    return new Promise((resolve) => {
      const wrap = document.createElement("div");
      const input = document.createElement("input");
      input.type = "text";
      input.className = "form-control";
      input.value = suggestSetName([path]);
      input.placeholder = "RAW";
      input.spellcheck = false;
      wrap.appendChild(
        formRow("Set name", input, "Curves land under this name. Blank = RAW."),
      );
      const hint = document.createElement("p");
      hint.className = "form-hint";
      hint.textContent =
        "A name already used on this well is auto-suffixed (WIRE → WIRE_1), so a second tape " +
        "never overwrites the first — import it, then compare. RAW duplicates also stop for a " +
        "per-curve keep-separate or skip decision; merge-into-existing is never the default.";
      wrap.appendChild(hint);

      const undeclaredUnit = document.createElement("select");
      undeclaredUnit.className = "form-control";
      for (const [value, label] of [
        ["", "Require the DLIS index to declare it"],
        ["M", "Metres (explicit confirmation)"],
        ["FT", "Feet (explicit confirmation)"],
      ] as const) {
        const option = document.createElement("option");
        option.value = value;
        option.textContent = label;
        undeclaredUnit.appendChild(option);
      }
      wrap.appendChild(
        formRow(
          "File depth unit when undeclared",
          undeclaredUnit,
          "The index channel's UNITS attribute is used first. This choice is only for a tape that omits it.",
        ),
      );

      const sentinelExceptions = document.createElement("input");
      sentinelExceptions.type = "text";
      sentinelExceptions.className = "form-control";
      sentinelExceptions.placeholder = "e.g. TENSION, AMPLITUDE";
      sentinelExceptions.spellcheck = false;
      wrap.appendChild(
        formRow(
          "Keep LAS sentinel values in",
          sentinelExceptions,
          "Optional exact DLIS channel mnemonics, comma-separated. In these channels, finite −999.25/−9999 samples remain data; non-finite and >1e30 values are still missing.",
        ),
      );

      const actions = document.createElement("div");
      actions.className = "form-actions";
      const cancelBtn = document.createElement("button");
      cancelBtn.className = "btn";
      cancelBtn.textContent = "Cancel";
      const okBtn = document.createElement("button");
      okBtn.className = "btn btn-accent";
      okBtn.textContent = "Import";
      actions.append(cancelBtn, okBtn);
      wrap.appendChild(actions);

      let settled = false;
      const finish = (v: {
        setName: string;
        fileDepthUnit: "M" | "FT" | null;
        lasSentinelExceptions: string[];
      } | null) => {
        if (settled) return;
        settled = true;
        close();
        resolve(v);
      };
      const close = openModal("Import DLIS — curve set", wrap, 520);
      cancelBtn.addEventListener("click", () => finish(null));
      okBtn.addEventListener("click", () => finish({
        setName: input.value.trim(),
        fileDepthUnit: undeclaredUnit.value === "M" || undeclaredUnit.value === "FT"
          ? undeclaredUnit.value
          : null,
        lasSentinelExceptions: sentinelExceptions.value
          .split(",")
          .map((name) => name.trim())
          .filter((name) => name.length > 0),
      }));
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") okBtn.click();
      });
      const root = document.querySelector<HTMLElement>("#modal-root");
      if (root) {
        const observer = new MutationObserver(() => {
          if (!wrap.isConnected) {
            observer.disconnect();
            finish(null);
          }
        });
        observer.observe(root, { childList: true });
      }
      input.focus();
      input.select();
    });
  }

  /** "Import SCAL…" — replaces the well's capillary-pressure (Pc/Sw) points from one or
   *  more files (flat CSV, porous-plate wide table, or per-plug centrifuge blocks) and
   *  fits the Leverett J-function, reporting SWH_A/SWH_B for the sw_height module. */
  private async handleImportScal(): Promise<void> {
    const well = requireWell("Import SCAL");
    if (!well) return;
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
    // The delivery these files form (T-IMP-08): a later report never overwrites an
    // earlier one — it becomes a second SCAL set and goes live.
    const setInput = document.createElement("input");
    setInput.type = "text";
    setInput.className = "form-control";
    setInput.value = "SCAL";
    content.appendChild(
      formRow(
        "SCAL set",
        setInput,
        "Names this delivery (the files selected together). A name already on the well is suffixed (SCAL → SCAL_1); the new set becomes live and drives Pc QC, J-fits and Thomeer.",
      ),
    );
    // SCAL plugs ARE core plugs, so their depths are the core report's depths.
    const scalFollowCore = buildFollowCoreRow("the plug depths", "scal");
    content.appendChild(scalFollowCore.el);

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
      void importScalFiles(
        well.well_id,
        paths,
        fmt,
        sysSel.value,
        ift,
        setInput.value.trim() || "SCAL",
        scalFollowCore.checked(),
      )
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
          const setNote = result.set_name ? ` Set ${result.set_name}.` : "";
          // The core-following note is the one thing here the user cannot check by eye.
          const coreNote = result.note ? ` ${result.note}.` : "";
          resultBox.textContent = `Imported ${result.rows} Pc point(s).${setNote}${coreNote} ${fitText}`;
          setStatus(`SCAL: ${result.rows} points imported for ${well.well_name}.${setNote}`);
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

  /** "Import Deviation…" — loads an MD/INC/AZI survey CSV and computes minimum-curvature
   *  TVD/TVDSS for the selected well. Prompts for the datum (KB) elevation. */
  private async handleImportDeviation(): Promise<void> {
    const well = requireWell("Import deviation survey");
    if (!well) return;
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
    // T-IMP-12: the survey is VERSIONED. A definitive survey imported over a preliminary
    // one lands beside it and takes over TVD/TVDSS; the old one stays switchable.
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "form-control";
    nameInput.value = "SURVEY";
    content.appendChild(
      formRow(
        "Survey name",
        nameInput,
        "Names this survey. A name already used on the well is suffixed (SURVEY → SURVEY_1) — an import never overwrites an earlier survey. The new survey becomes active and drives TVD/TVDSS.",
      ),
    );
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
      void importDeviationCsv(well.well_id, path, datum, nameInput.value.trim() || "SURVEY")
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

  /** "Recompute TVD/TVDSS Curves" — rebuilds the TVD/TVDSS curves from every well's deviation
   *  survey onto its current log grid. Deviation import already does this; this covers logs
   *  imported after the survey, or a KB edit. A no-op (0 samples) for wells without a survey. */
  private async handleMaterializeTvd(): Promise<void> {
    setStatus("Computing TVD/TVDSS curves…");
    try {
      const wells = await listWells();
      if (wells.length === 0) {
        setStatus("No wells in the project");
        return;
      }
      const results = await materializeTvd(wells.map((w) => w.well_id));
      const surveyed = results.filter((r) => r.has_survey);
      if (surveyed.length === 0) {
        setStatus("No deviation surveys found — import one first (Import Deviation…)");
        return;
      }
      const written = surveyed.filter((r) => r.samples > 0);
      const total = written.reduce((sum, r) => sum + r.samples, 0);
      const pending = surveyed.length - written.length;
      const note = pending > 0 ? ` (${pending} surveyed well(s) have no logs yet)` : "";
      setStatus(`TVD/TVDSS computed for ${written.length} of ${surveyed.length} surveyed well(s), ${total} samples${note}.`);
      recordProcess("Edit", `Recomputed TVD/TVDSS curves for ${written.length} well(s)`);
      this.workspace.notifyDataChanged();
    } catch (err) {
      setStatus(`TVD/TVDSS compute failed: ${err}`);
    }
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
    // (Mahakam 50S; Java Sea 48S/49S) with the north straddling the equator.
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
    const selected = requireWell("Well header");
    if (!selected) return;
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
