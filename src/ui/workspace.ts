import {
  DockviewComponent,
  type CreateComponentOptions,
  type DockviewGroupPanel,
  type GroupPanelPartInitParameters,
  type IContentRenderer,
  type IDockviewPanel,
  type IHeaderActionsRenderer,
} from "dockview-core";
import { openModal } from "./modal";
import { escapeHtml } from "./safeDom";
import "dockview-core/dist/styles/dockview.css";
import { appState, bumpDataVersion, setStatus } from "../state";
import { WORKSPACE_DIRTY, clearDirty, isDirty, markDirty, subscribeDirty } from "../dirty";
import { listModules, type Layout, type ModuleSpec, type WellSummary } from "../ipc";
import { recordProcess } from "../processLog";
import { LogViewPanel } from "./logViewPanel";
import { ObjectTree } from "./objectTree";
import { openDataSetsDialog } from "./dataSetsDialog";
import { TopsPanel } from "./topsPanel";
import { InspectorPanel } from "./inspectorPanel";
import { buildHistogramContent } from "./histogramPanel";
import { buildCrossplotContent } from "./crossplotPanel";
import { buildPickettContent } from "./pickettPanel";
import { buildCorrelationContent } from "./correlationPanel";
import { buildDashboardContent } from "./dashboardPanel";
import { DbInspectorPanel } from "./dbInspectorPanel";
import type { PlotContent } from "./plotCommon";
import { SqlQueryPanel } from "./sqlQueryPanel";
import { HistoryPanel } from "./historyPanel";
import { showContextMenu, type ContextMenuEntry } from "./contextMenu";
import { imageExportMenuEntries } from "./plotExport";
import { forgetViewer, isWorkingPane, markActiveViewer } from "./activeViewer";

const LAYOUT_STORAGE_KEY = "sandibumi.workspace";
/** Id of the blank content-area placeholder shown only when every real content pane is closed
 *  (so the fixed-width sidebar never stretches to fill the window). */
const CANVAS_ID = "canvas";

type PlotKind = "histogram" | "crossplot" | "pickett" | "correlation" | "vega";

/** A named, reopenable workspace snapshot: the dock layout (which panes/plots/log views
 *  are open and how they're arranged) plus the active well so the visualizations come
 *  back pointed at the same data. Stored in the `documents` table (doc_type "session"). */
export interface SessionSnapshot {
  version: 1;
  layout: ReturnType<DockviewComponent["toJSON"]>;
  well: WellSummary | null;
  /** Each log view's chosen Layout, keyed by dock panel id. Dockview's toJSON doesn't
   *  serialize panel-internal state, so we snapshot it alongside and reapply on restore.
   *  Optional — sessions saved before this field simply restore the default layout. */
  logViewLayouts?: Record<string, Layout>;
}

/** A dock panel whose content is a plain DOM subtree with optional async fill + cleanup. */
class DomPanel implements IContentRenderer {
  readonly element = document.createElement("div");
  private cleanup: (() => void) | null = null;

  constructor(
    className: string,
    private fill: (host: HTMLElement, params: GroupPanelPartInitParameters) => (() => void) | void,
  ) {
    this.element.className = `dock-panel ${className}`;
  }

  init(params: GroupPanelPartInitParameters): void {
    this.cleanup = this.fill(this.element, params) ?? null;
  }

  dispose(): void {
    this.cleanup?.();
  }
}

/** The MDI docking workspace: hosts log views, plots, Wells & Tops, and the Inspector as
 *  dockable/floatable/tabbed panels (dockview-core). Singleton panels (wellsTops,
 *  inspector) are focused if already open; log views and plots always open fresh. */
export class Workspace {
  private dock: DockviewComponent;
  private rootContainer!: HTMLElement;
  private logViews = new Map<string, LogViewPanel>();
  /** Each open plot pane's Properties dialog opener, so the pane's right-click menu can
   *  offer it (the canvas no longer hijacks right-click for the dialog). */
  private plotProps = new Map<string, () => void>();
  private counter = 0;
  /** Layout-change events before this time don't mark the workspace dirty — set around
   *  programmatic rebuilds (applySession/reset) and named saves, whose tab-title updates
   *  also fire onDidLayoutChange and would otherwise re-dirty a just-saved workspace. */
  private dirtyMuteUntil = 0;
  /** True only while the user is mid-drag on a splitter: the anchor panes are momentarily released
   *  from their fixed (min == max) width so they can be resized, then re-pinned on release. */
  private anchorsUnlocked = false;

  constructor(container: HTMLElement) {
    this.rootContainer = container;
    this.dock = new DockviewComponent(container, {
      // A DockviewTheme object (not `className`) is what actually applies the theme class;
      // otherwise dockview falls back to its default dark "abyss" theme.
      theme: {
        name: "sandibumi",
        className: "dockview-theme-sandibumi",
        gap: 2,
        dndTabIndicator: "line",
      },
      createComponent: (options) => this.createComponent(options),
      createRightHeaderActionComponent: (group) => this.createGroupActions(group),
    });

    // Drive the dock's layout from the container size explicitly — the built-in
    // auto-resize does not reliably fire an initial layout, leaving groups at 100px.
    this.dock.layout(container.clientWidth, container.clientHeight);
    // Reflow on resize. `forceResize=true` is essential: without it dockview no-ops when it
    // thinks its cached size is unchanged, so the inner grid never redistributes and the panes
    // "stay" as the window grows. We drive it from BOTH a ResizeObserver (catches container
    // changes: ribbon height, devtools) AND the window resize event (the reliable signal when
    // the whole application window is resized — the ResizeObserver alone was not firing for it).
    const relayout = () => this.dock.layout(container.clientWidth, container.clientHeight, true);
    const resizeObserver = new ResizeObserver(() => relayout());
    resizeObserver.observe(container);
    window.addEventListener("resize", relayout);

    // The anchors are pinned to a fixed width so dockview never reflows them. To still allow the
    // user to resize them, release the pin the instant a splitter (`.dv-sash`) is grabbed — CAPTURE
    // phase, so it runs before dockview's own drag handler and the drag goes live — then re-pin them
    // at their new width on release. (Grabbing a content-content sash is harmless: the sidebar isn't
    // adjacent to it, so it doesn't move, and it's re-pinned at the same width.)
    container.addEventListener(
      "pointerdown",
      (e) => {
        const t = e.target as HTMLElement | null;
        if (t?.closest?.(".dv-sash") && !this.anchorsUnlocked) {
          this.anchorsUnlocked = true;
          this.setAnchorsFixed(false);
        }
      },
      true,
    );
    const endSashDrag = () => {
      if (!this.anchorsUnlocked) return;
      this.anchorsUnlocked = false;
      // Re-pin fixed at whatever width the anchors now have (deferred a frame so the drag's final
      // geometry has landed before we read it).
      requestAnimationFrame(() => this.setAnchorsFixed(true));
    };
    window.addEventListener("pointerup", endSashDrag, true);
    window.addEventListener("pointercancel", endSashDrag, true);

    if (!this.restore()) this.defaultWorkspace();
    this.ensureWellsPane();
    this.ensureTopsPane();
    this.ensureMonitorsBelowWells();
    this.ensureContentPlaceholder();
    this.lockAnchorGroups();
    // Settle all sizes after the build (locks + any placeholder) so a layout restored in a
    // stretched/zero-width state comes up correctly.
    this.dock.layout(container.clientWidth, container.clientHeight, true);

    // Constructor's own restore/default build happens above this subscription, so it
    // never marks the workspace dirty; only later (user) arrangement changes do.
    this.muteDirty();
    let saveHandle: number | undefined;
    this.dock.onDidLayoutChange(() => {
      // A closed last content pane brings the blank Workspace placeholder in (and any newly
      // opened content takes it back out) — keeps the fixed sidebar from stretching to fill.
      this.ensureContentPlaceholder();
      if (Date.now() > this.dirtyMuteUntil) markDirty(WORKSPACE_DIRTY);
      if (saveHandle !== undefined) window.clearTimeout(saveHandle);
      saveHandle = window.setTimeout(() => {
        try {
          localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify(this.dock.toJSON()));
        } catch {
          /* quota/serialization issues are non-fatal */
        }
      }, 500);
    });
  }

  /** The panels that anchor the workspace — Wells, Tops, Processing, Performance. Each anchor group
   *  is pinned to a FIXED width (min == max), which dockview excludes from its proportional
   *  redistribution — so opening/closing panes and windows (and resizing the app window) can never
   *  reflow the sidebar (Jauhar 2026-07-21: "when i add new panes or windows, it still change size").
   *  The user can still resize them: grabbing a splitter unlocks the anchors for the drag, and they
   *  are re-pinned at the new width on release (see the pointerdown/up handlers in the constructor).
   *  The initial pin is applied once per group instance (WeakSet) so a reopened/moved anchor is
   *  re-seeded, and a user's manual resize is never stomped on a refocus. */
  private static readonly ANCHOR_PANEL_IDS = ["wellsTops", "tops", "processing", "health"];
  private readonly lockedGroups = new WeakSet<DockviewGroupPanel>();

  private lockAnchorGroups(): void {
    for (const id of Workspace.ANCHOR_PANEL_IDS) {
      const group = this.dock.panels.find((p) => p.id === id)?.api.group;
      if (!group || group.api.location.type !== "grid" || this.lockedGroups.has(group)) continue;
      // Seed a sane sidebar width. A width over ~450 means the layout was restored in a stretched
      // state (e.g. saved while the sidebar was the only pane) — reset to the default. Otherwise
      // respect the user's width within reason.
      const raw = group.width || 260;
      const width = raw > 450 ? 260 : Math.max(raw, 180);
      group.api.setSize({ width });
      // Fixed width (min == max) → dockview keeps it out of proportional redistribution, so it STAYS
      // put when other panes/windows open or close. A splitter drag unlocks it briefly (constructor
      // handlers) and re-pins it at the new width on release.
      group.api.setConstraints({ minimumWidth: width, maximumWidth: width });
      this.lockedGroups.add(group);
    }
  }

  /** Pin every anchor group to a fixed width (min == max at its current width) so dockview won't
   *  reflow it, or — while the user is dragging a splitter — release it to a floor-only constraint
   *  so it can be resized. `fixed=true` reads each group's current width and pins there. */
  private setAnchorsFixed(fixed: boolean): void {
    for (const id of Workspace.ANCHOR_PANEL_IDS) {
      const group = this.dock.panels.find((p) => p.id === id)?.api.group;
      if (!group || group.api.location.type !== "grid") continue;
      if (fixed) {
        const w = Math.min(700, Math.max(150, Math.round(group.width)));
        group.api.setConstraints({ minimumWidth: w, maximumWidth: w });
      } else {
        group.api.setConstraints({ minimumWidth: 150, maximumWidth: Number.MAX_SAFE_INTEGER });
      }
    }
  }

  /** Keeps a blank "Workspace" pane in the CONTENT area whenever every real content pane (log
   *  views, plots, tools) has been closed, so the fixed-width sidebar (Wells & Tops / Processing /
   *  Performance) never has to stretch to fill the window — Jauhar asked for blank space there
   *  rather than the main panes resizing. Added when content hits zero, removed the moment any
   *  content opens, so it is invisible during normal work. The two conditions are stable fixpoints
   *  (after adding, the pane exists so the add branch stops; after removing, content exists so the
   *  remove branch stops), hence calling this on every layout change cannot loop. */
  private ensureContentPlaceholder(): void {
    const anchors = new Set<string>(Workspace.ANCHOR_PANEL_IDS);
    const hasCanvas = this.dock.panels.some((p) => p.id === CANVAS_ID);
    const hasContent = this.dock.panels.some((p) => p.id !== CANVAS_ID && !anchors.has(p.id));
    if (!hasContent && !hasCanvas) {
      this.muteDirty();
      this.dock.addPanel({ id: CANVAS_ID, component: "canvas", title: "Workspace", position: { direction: "right" } });
      // Next to the width-locked sidebar dockview can create the new pane at width 0; force a
      // relayout so it fills the free space instead of leaving the window mostly blank.
      this.dock.layout(this.rootContainer.clientWidth, this.rootContainer.clientHeight, true);
    } else if (hasContent && hasCanvas) {
      this.muteDirty();
      this.dock.panels.find((p) => p.id === CANVAS_ID)?.api.close();
    }
  }

  /** Per-group ("window") tab-bar actions: add a panel, maximize, float, dock back,
   *  and close. Splitting/tabbing/moving panels between windows is dockview's
   *  built-in tab drag-and-drop. */
  private createGroupActions(group: DockviewGroupPanel): IHeaderActionsRenderer {
    const element = document.createElement("div");
    element.className = "dock-group-actions";

    const btn = (text: string, title: string, onClick: () => void) => {
      const b = document.createElement("button");
      b.className = "dock-action-btn";
      b.title = title;
      b.textContent = text;
      b.addEventListener("click", onClick);
      element.appendChild(b);
      return b;
    };

    const addBtn = btn("＋", "Add a panel to this window", () => this.showAddPanelMenu(addBtn, group));
    const splitVBtn = btn("⬌", "Split this window vertically (new window on the right)", () =>
      this.splitGroup(group, "right"),
    );
    const splitHBtn = btn("⬍", "Split this window horizontally (new window below)", () =>
      this.splitGroup(group, "below"),
    );
    const maxBtn = btn("▢", "Maximize / restore this window", () => {
      if (group.api.isMaximized()) group.api.exitMaximized();
      else group.api.maximize();
    });
    const floatBtn = btn("⧉", "Float this window (drag its title bar to move it)", () => {
      if (group.api.location.type === "grid") {
        this.dock.addFloatingGroup(group, {
          position: { left: 80 + (this.counter % 5) * 30, top: 60 + (this.counter % 5) * 30 },
          width: 560,
          height: 420,
        });
      }
    });
    const dockBtn = btn("⇱", "Dock this window back into the workspace", () => {
      if (group.api.location.type !== "grid") group.api.moveTo({ position: "right" });
    });
    const closeBtn = btn("✕", "Close this window and every panel in it", () => group.api.close());

    // An anchor group holds only sidebar anchor panes (Wells/Tops/Processing/Performance).
    const isAnchorGroup = () =>
      this.dock.panels.some((p) => p.api.group === group) &&
      this.dock.panels.filter((p) => p.api.group === group).every((p) => Workspace.ANCHOR_PANEL_IDS.includes(p.id));

    // Grid windows can split/float/maximize; floating windows dock back instead.
    const sync = () => {
      const floating = group.api.location.type !== "grid";
      floatBtn.style.display = floating ? "none" : "";
      maxBtn.style.display = floating ? "none" : "";
      splitVBtn.style.display = floating ? "none" : "";
      splitHBtn.style.display = floating ? "none" : "";
      dockBtn.style.display = floating ? "" : "none";
      // The 4 anchor sidebar panes must always STAY (Jauhar 2026-07-21): no close and no float,
      // so opening/closing other windows can never make them vanish. They remain freely
      // resizable via the splitter (the min-width floor set in lockAnchorGroups).
      if (isAnchorGroup()) {
        closeBtn.style.display = "none";
        floatBtn.style.display = "none";
      }
    };
    sync();
    // A newly created anchor group may not have its panel attached at the first sync; re-run it on
    // the next microtask, and whenever the group's location or panel membership changes.
    queueMicrotask(sync);
    const subs = [group.api.onDidLocationChange(() => sync()), group.api.onDidActivePanelChange(() => sync())];

    return { element, init: () => {}, dispose: () => subs.forEach((s) => s.dispose()) };
  }

  /** The ＋ menu on a window's tab bar: opens any panel type as a new tab inside
   *  THAT window (singletons move there instead of duplicating). */
  private showAddPanelMenu(anchor: HTMLElement, group: DockviewGroupPanel): void {
    document.querySelector(".dock-add-menu")?.remove();
    const menu = document.createElement("div");
    menu.className = "dock-add-menu";

    const entries: ([string, () => void] | "sep")[] = [
      ["New Log View", () => this.openLogView(group)],
      ["New Histogram", () => this.openPlot("histogram", group)],
      ["New Crossplot", () => this.openPlot("crossplot", group)],
      ["New Vega Chart", () => this.openPlot("vega", group)],
      ["New Pickett", () => this.openPlot("pickett", group)],
      ["New Correlation", () => this.openPlot("correlation", group)],
      "sep",
      ["Field Dashboard", () => this.openDashboard(group)],
      ["Field Map", () => this.openMap(group)],
      ["Workflow Builder", () => this.openWorkflow(group)],
      ["Cutoffs & Pay Summary", () => this.openPaySummary(group)],
      ["Cutoff Sensitivity", () => this.openCutoff(group)],
      ["Machine Learning", () => this.openMl(group)],
      ["Monte Carlo", () => this.openMonteCarlo(group)],
      ["SHF Fit (Cuddy FOIL)", () => this.openShf(group)],
      ["Pc Fit (Thomeer)", () => this.openThomeer(group)],
      ["HFU Clustering (FZI)", () => this.openHfu(group)],
      ["Lorenz Plot (flow units)", () => this.openLorenz(group)],
      ["Facies Tie-in (RT confusion)", () => this.openFaciesTie(group)],
      ["Unconventional (ΔlogR + Langmuir)", () => this.openUnconventional(group)],
      ["Results QC (Sw spread)", () => this.openResultsQc(group)],
      ["SandiMin Solver", () => this.openMultimin(group)],
      "sep",
      ["Zones", () => this.openZones(group)],
      ["Autocorrelate Tops", () => this.openAutoCorr(group)],
      ["Composite Log", () => this.openComposite(group)],
      ["Report", () => this.openReport(group)],
      "sep",
      ["Wells", () => this.openWellsTops(group)],
      ["Tops", () => this.openTops(group)],
      ["Inspector", () => this.openInspector(group)],
      ["Database Inspector", () => this.openDbInspector(group)],
      ["SQL Query", () => this.openSqlQuery(group)],
      ["Processing History", () => this.openHistory(group)],
    ];
    for (const entry of entries) {
      if (entry === "sep") {
        const sep = document.createElement("div");
        sep.className = "dock-add-menu-sep";
        menu.appendChild(sep);
        continue;
      }
      const [label, onPick] = entry;
      const item = document.createElement("button");
      item.className = "ribbon-menu-item";
      item.textContent = label;
      item.addEventListener("click", () => {
        menu.remove();
        onPick();
      });
      menu.appendChild(item);
    }

    const rect = anchor.getBoundingClientRect();
    menu.style.left = `${Math.max(4, Math.min(rect.left, window.innerWidth - 200))}px`;
    menu.style.top = `${rect.bottom + 4}px`;
    document.body.appendChild(menu);

    // Defer the outside-click closer so the opening click doesn't fire it.
    const closeOnOutside = (e: MouseEvent) => {
      if (!menu.contains(e.target as Node)) {
        menu.remove();
        document.removeEventListener("mousedown", closeOnOutside);
      }
    };
    window.setTimeout(() => document.addEventListener("mousedown", closeOnOutside), 0);
  }

  private createComponent(options: CreateComponentOptions): IContentRenderer {
    const renderer = this.buildRenderer(options);
    // A right-click anywhere in the panel opens a menu whose items are specific to this
    // panel's type ("personalized by active window").
    this.attachContextMenu(renderer.element, options.id, options.name);
    return renderer;
  }

  private buildRenderer(options: CreateComponentOptions): IContentRenderer {
    switch (options.name) {
      case "logview":
        return this.createLogView(options.id);
      case "canvas":
        return new DomPanel("dock-canvas", (host) => {
          host.innerHTML = `
            <div class="canvas-empty">
              <img class="canvas-logo" src="/logo-mark.svg" alt="" width="52" height="52" />
              <div class="canvas-hint">Open a Log View or a plot from the ribbon to get started.</div>
            </div>`;
        });
      case "wellsTops":
        return this.createWellsTops();
      case "tops":
        return this.createTops();
      case "inspector":
        return this.createInspector();
      case "dbInspector":
        return new DomPanel("dock-dbinspector", (host) => {
          const panel = new DbInspectorPanel(host);
          return () => panel.dispose();
        });
      case "sqlQuery":
        return new DomPanel("dock-sqlquery", (host) => {
          new SqlQueryPanel(host);
        });
      case "history":
        return new DomPanel("dock-history", (host) => {
          const panel = new HistoryPanel(host);
          return () => panel.dispose();
        });
      case "dashboard":
        return this.asyncPane("dock-dashboard", () => buildDashboardContent(setStatus), "dashboard");
      case "map":
        return this.asyncPane(
          "dock-map",
          () => import("./mapPanel").then((m) => m.buildMapContent(setStatus)),
          "field map",
        );
      case "workflow":
        return this.asyncPane(
          "dock-workflow",
          () => import("./workflowDialog").then((m) => m.buildWorkflowContent(setStatus)),
          "workflow builder",
        );
      case "processing":
        return this.asyncPane(
          "dock-processing",
          () => import("./processingPanel").then((m) => m.buildProcessingContent(setStatus)),
          "processing",
        );
      case "health":
        return this.asyncPane(
          "dock-health",
          () => import("./healthPanel").then((m) => m.buildHealthContent(setStatus)),
          "performance monitor",
        );
      // Tool panes ported from popups (ROADMAP §4c item 14): dynamic imports keep
      // workspace.ts free of ribbon↔dialog cycles, same as the workflow builder.
      case "paysummary":
        return this.asyncPane(
          "dock-paysummary",
          () => import("./summaryDialog").then((m) => m.buildSummaryContent(setStatus)),
          "pay summary",
        );
      case "cutoff":
        return this.asyncPane(
          "dock-cutoff",
          () => import("./cutoffDialog").then((m) => m.buildCutoffContent(setStatus)),
          "cutoff sensitivity",
        );
      case "ml":
        return this.asyncPane(
          "dock-ml",
          () => import("./mlDialog").then((m) => m.buildMlContent(setStatus)),
          "machine learning",
        );
      case "montecarlo":
        return this.asyncPane(
          "dock-montecarlo",
          () => import("./monteCarloDialog").then((m) => m.buildMonteCarloContent(setStatus)),
          "Monte Carlo",
        );
      case "shf":
        return this.asyncPane(
          "dock-shf",
          () => import("./shfDialog").then((m) => m.buildShfContent(setStatus)),
          "SHF fit",
        );
      case "thomeer":
        return this.asyncPane(
          "dock-thomeer",
          () => import("./thomeerDialog").then((m) => m.buildThomeerContent(setStatus)),
          "Thomeer Pc fit",
        );
      case "hfu":
        return this.asyncPane(
          "dock-hfu",
          () => import("./hfuDialog").then((m) => m.buildHfuContent(setStatus)),
          "HFU clustering",
        );
      case "lorenz":
        return this.asyncPane(
          "dock-lorenz",
          () => import("./lorenzDialog").then((m) => m.buildLorenzContent(setStatus)),
          "Lorenz plot",
        );
      case "faciesTie":
        return this.asyncPane(
          "dock-facies-tie",
          () => import("./faciesTieDialog").then((m) => m.buildFaciesTieContent(setStatus)),
          "facies tie-in",
        );
      case "multimin":
        return this.asyncPane(
          "dock-multimin",
          () => import("./multiminDialog").then((m) => m.buildMultiminContent(setStatus)),
          "SandiMin",
        );
      // Auto-generated module form (panel id "module:<name>"): the spec is looked up in
      // the backend manifest, so layout restore rebuilds the pane from its id alone and
      // every module the backend registers gets a pane with no frontend work.
      case "module":
        return this.asyncPane(
          "dock-module",
          async () => {
            const name = options.id.replace(/^module:/, "");
            const spec = (await listModules()).find((s) => s.name === name);
            if (!spec) throw new Error(`unknown module "${name}"`);
            const m = await import("./moduleDialog");
            return m.buildModuleContent(spec, {
              setStatus,
              onRunComplete: (_outputs, wellNames) => {
                // Attribute History to the wells actually run — a single well by name, a genuine
                // multi-well batch as null (the processLog contract) — never the globally
                // "selected" well, which a scoped run may not have touched at all.
                recordProcess(
                  "Module",
                  `Ran ${spec.title}${wellNames.length > 1 ? ` on ${wellNames.length} wells` : ""}`,
                  wellNames.length === 1 ? wellNames[0] : null,
                );
                this.notifyDataChanged();
              },
            });
          },
          "module",
        );
      // Well-following tool panes (converted popups): wellPane rebuilds the content for
      // each newly selected well, so the builders stay well-bound like the plot panes.
      case "zones":
        return this.wellPane(
          "dock-zones",
          "Zones",
          "the zone manager",
          () => import("./zonesDialog").then((m) => m.buildZonesContent),
          true,
        );
      case "autocorr":
        return this.wellPane(
          "dock-autocorr",
          "Autocorrelate Tops",
          "top autocorrelation",
          () => import("./autoCorrDialog").then((m) => m.buildAutoCorrContent),
          true,
        );
      case "composite":
        return this.wellPane(
          "dock-composite",
          "Composite Log",
          "the composite log",
          () => import("./compositeDialog").then((m) => m.buildCompositeContent),
          true,
        );
      case "report":
        return this.wellPane(
          "dock-report",
          "Report",
          "the report generator",
          () => import("./reportDialog").then((m) => m.buildReportContent),
          true,
        );
      case "unconventional":
        return this.wellPane(
          "dock-unconventional",
          "Unconventional",
          "the unconventional visuals",
          () => import("./unconventionalPanel").then((m) => m.buildUnconventionalContent),
          true,
        );
      case "resultsQc":
        return this.wellPane(
          "dock-results-qc",
          "Results QC",
          "the results-QC dashboard",
          () => import("./resultsQcPanel").then((m) => m.buildResultsQcContent),
          true,
        );
      case "histogram":
      case "crossplot":
      case "pickett":
      case "correlation":
      case "vega":
        return this.createPlot(options.name);
      default:
        return new DomPanel("dock-unknown", (host) => {
          host.textContent = `Unknown panel: ${options.name}`;
        });
    }
  }

  /** A DomPanel whose content resolves asynchronously (dynamic import + async builder):
   *  drops the build result if the panel closed meanwhile, always runs the dispose. */
  private asyncPane(
    className: string,
    load: () => Promise<{ el: HTMLElement; dispose?: () => void }>,
    label: string,
  ): IContentRenderer {
    return new DomPanel(className, (host) => {
      let disposer: (() => void) | undefined;
      let closed = false;
      load()
        .then((content) => {
          if (closed) return void content.dispose?.();
          host.appendChild(content.el);
          disposer = content.dispose;
        })
        .catch((err) => {
          host.innerHTML = `<div class="logview-message">Failed to open ${escapeHtml(label)}: ${escapeHtml(String(err))}</div>`;
        });
      return () => {
        closed = true;
        disposer?.();
      };
    });
  }

  /** A singleton tool pane bound to the selected well: the content is rebuilt for each
   *  newly selected well (same follow rules as the plots — with the pin off it only
   *  follows to another well while active) and shows a hint until a well exists. The
   *  builders receive the well as an argument and never track the selection themselves;
   *  with `followData` a data-version bump rebuilds the pane's own well so its lists
   *  (tops, wells, layouts) stay current, the way the modal era re-fetched on every open. */
  private wellPane(
    className: string,
    titleBase: string,
    label: string,
    loadBuilder: () => Promise<
      (well: WellSummary, setStatus: (t: string) => void) => Promise<{ el: HTMLElement; dispose?: () => void }>
    >,
    followData = false,
  ): IContentRenderer {
    return new DomPanel(className, (host, params) => {
      let disposer: (() => void) | undefined;
      let generation = 0;
      let closed = false;
      let currentWell: WellSummary | null = null;

      const rebuild = (well: WellSummary | null) => {
        const gen = ++generation;
        disposer?.();
        disposer = undefined;
        host.innerHTML = "";
        currentWell = well;
        if (!well) {
          // No well: reset the tab title too, or a closed project's well lingers there.
          params.api.setTitle(titleBase);
          host.innerHTML = `<div class="logview-message">Select a well (Wells &amp; Tops) — ${escapeHtml(label)} will follow.</div>`;
          return;
        }
        params.api.setTitle(`${titleBase} — ${well.well_name}`);
        loadBuilder()
          .then((build) => build(well, setStatus))
          .then((content) => {
            if (closed || gen !== generation) {
              content.dispose?.();
              return;
            }
            host.appendChild(content.el);
            disposer = content.dispose;
          })
          .catch((err) => {
            if (closed || gen !== generation) return; // a newer build/close already won
            host.innerHTML = `<div class="logview-message">Failed to open ${escapeHtml(label)}: ${escapeHtml(String(err))}</div>`;
          });
      };

      const unsubWell = appState.selectedWell.subscribe((well) => {
        const nextId = well?.well_id ?? null;
        if (generation > 0 && nextId === currentWell?.well_id) return;
        // Pin OFF = working-pane mode: an already-built pane holding a well only follows
        // the selection to ANOTHER real well while it is the WORKING pane. Always rebuild
        // when the pane has no well yet (catch up to a selection) or the selection was
        // cleared (project switch), so a stale well can never linger in the pane.
        // The gate is isWorkingPane, not api.isActive — selecting a well activates the
        // Wells tree, so no viewer is ever "active" at that instant (see activeViewer.ts).
        if (
          generation > 0 &&
          currentWell !== null &&
          nextId !== null &&
          !appState.wellPinned.get() &&
          !isWorkingPane(params.api.id)
        )
          return;
        rebuild(well);
      });

      // Data changes (a top picked, an import, an undo, a newly saved layout) invalidate
      // the built form's cached lists. Rebuild the pane's OWN well — not the global
      // selection, so pin-off working panes keep their well — deferred a microtask so a
      // bump fired from inside this pane's own action unwinds first.
      let dataPrimed = false;
      const unsubData = appState.dataVersion.subscribe(() => {
        if (!followData) return;
        if (!dataPrimed) {
          dataPrimed = true; // the subscribe fires immediately; the first build already ran
          return;
        }
        if (currentWell === null) return;
        const at = generation;
        queueMicrotask(() => {
          if (!closed && at === generation) rebuild(currentWell);
        });
      });

      const untrack = this.trackViewer(params);
      return () => {
        closed = true;
        untrack();
        unsubWell();
        unsubData();
        disposer?.();
      };
    });
  }

  /** Registers a viewer pane (log view, plot, well-bound tool pane) with the working-pane
   *  tracker that drives Pin-OFF following. Returns the teardown to run on close. */
  private trackViewer(params: GroupPanelPartInitParameters): () => void {
    const id = params.api.id;
    if (params.api.isActive) markActiveViewer(id);
    const sub = params.api.onDidActiveChange((e) => {
      if (e.isActive) markActiveViewer(id);
    });
    return () => {
      sub.dispose();
      forgetViewer(id);
    };
  }

  /** Adds a new (empty) window beside `group` — the user's Split Right / Split Down. */
  private splitGroup(group: DockviewGroupPanel, direction: "right" | "below"): void {
    if (group.api.location.type !== "grid") return;
    this.dock.addGroup({ referenceGroup: group, direction });
  }

  /** Attaches a panel-type-aware context menu to a panel's host element. The custom menu
   *  is deliberately limited to "empty" areas of a pane — the background and the plot
   *  canvas. Over anything that already offers its own actions (buttons, form controls,
   *  toolbars, tree/table rows, editors, links) or selectable text, the right-click is
   *  left alone so those controls keep their native behaviour. */
  private attachContextMenu(host: HTMLElement, panelId: string, kind: string): void {
    host.addEventListener("contextmenu", (e) => {
      const target = e.target as HTMLElement;
      if (!target.closest) return;
      // Controls / editors / selectable data grids that own their own right-click.
      if (
        target.closest(
          "input, textarea, select, button, a, [contenteditable], .cm-editor, " +
            ".catalog-table, .db-grid, table, .plot-toolbar, .plot-template-bar, " +
            ".plot-export-group, .lv-tools, .tree-group-bar, .tree-node, .pick-row, " +
            ".stat-chips, .history-toolbar, .sidebar-title",
        )
      ) {
        return;
      }
      e.preventDefault();
      const group = this.dock.panels.find((p) => p.id === panelId)?.api.group ?? undefined;
      showContextMenu(e.clientX, e.clientY, this.contextItemsFor(kind, panelId, host, group, e));
    });
  }

  /** Builds the context-menu entries: panel-specific actions on top, then the window
   *  (split/float/maximize/close) block shared by every panel. */
  private contextItemsFor(
    kind: string,
    panelId: string,
    host: HTMLElement,
    group: DockviewGroupPanel | undefined,
    event?: MouseEvent,
  ): ContextMenuEntry[] {
    const items: ContextMenuEntry[] = [];

    if (kind === "logview") {
      const view = this.logViews.get(panelId);
      if (view) {
        // Right-click over a track: per-curve edit entries (wireline shift, interval
        // set/blank/interpolate/scale) come first, then the generic view actions.
        if (event) items.push(...view.curveMenuEntries(event));
        items.push(
          { heading: "Log View" },
          { label: "Reset view", onClick: () => view.resetView() },
          { label: "Zoom in", onClick: () => view.stepZoom(1.25) },
          { label: "Zoom out", onClick: () => view.stepZoom(1 / 1.25) },
          { label: "Widen tracks", onClick: () => void view.scaleAllTracks(1.15) },
          { label: "Narrow tracks", onClick: () => void view.scaleAllTracks(1 / 1.15) },
          "sep",
          { label: "Layout properties…", onClick: () => void view.openProperties() },
          { label: "Print / export layout…", onClick: () => this.openComposite(group) },
        );
      }
    } else if (kind === "histogram" || kind === "crossplot" || kind === "pickett" || kind === "correlation") {
      const nice = kind[0].toUpperCase() + kind.slice(1);
      items.push({ heading: nice });
      // Properties first — right-click used to open this dialog directly, which cost the
      // plots their pane menu (split/float/export). It is now the menu's top entry, so
      // both are reachable (Jauhar field review 2026-07-29, T-SHELL-17).
      const openProps = this.plotProps.get(panelId);
      if (openProps) items.push({ label: "Properties…", onClick: () => openProps() }, "sep");
      items.push(
        ...imageExportMenuEntries(() => host.querySelector<HTMLCanvasElement>("canvas.plot-canvas"), nice, setStatus),
        "sep",
        { label: `New ${kind} window`, onClick: () => this.openPlot(kind as PlotKind, group) },
      );
    } else if (kind === "history") {
      items.push({ heading: "Processing History" });
    } else if (kind === "wellsTops") {
      items.push({ heading: "Wells & Tops" }, { label: "Refresh", onClick: () => bumpDataVersion() });
    } else if (kind === "inspector") {
      items.push({ heading: "Inspector" }, { label: "Open Curve Catalog", onClick: () => this.openInspector(group) });
    } else if (kind === "dbInspector") {
      items.push({ heading: "Database Inspector" }, { label: "Refresh", onClick: () => bumpDataVersion() });
    } else if (kind === "dashboard") {
      items.push({ heading: "Field Dashboard" });
    } else if (kind === "map") {
      items.push({ heading: "Field Map" }, { label: "Refresh", onClick: () => bumpDataVersion() });
    } else if (kind === "workflow") {
      items.push({ heading: "Workflow Builder" });
    } else if (kind === "processing") {
      items.push({ heading: "Processing" });
    } else if (kind === "health") {
      items.push({ heading: "Performance" });
    } else if (kind === "paysummary") {
      items.push({ heading: "Cutoffs & Pay Summary" });
    } else if (kind === "cutoff") {
      items.push({ heading: "Cutoff Sensitivity" });
    } else if (kind === "ml") {
      items.push({ heading: "Machine Learning" });
    } else if (kind === "montecarlo") {
      items.push({ heading: "Monte Carlo" });
    } else if (kind === "multimin") {
      items.push({ heading: "SandiMin Solver" });
    } else if (kind === "module") {
      items.push({ heading: "Module" });
    } else if (kind === "zones") {
      items.push({ heading: "Zones" });
    } else if (kind === "autocorr") {
      items.push({ heading: "Autocorrelate Tops" });
    } else if (kind === "composite") {
      items.push({ heading: "Composite Log" });
    } else if (kind === "report") {
      items.push({ heading: "Report" });
    }

    // --- Help (every panel): opens the same contextual guide as the quick-access ? button. ---
    if (items.length) items.push("sep");
    items.push({ label: "Help for this panel…", onClick: () => void this.openHelpForPanelId(panelId) });

    // --- Window block (every panel) ---
    items.push("sep");
    items.push({ heading: "Window" });
    const inGrid = group?.api.location.type === "grid";
    if (group && inGrid) {
      items.push(
        { label: "Split right (vertical)", onClick: () => this.splitGroup(group, "right") },
        { label: "Split down (horizontal)", onClick: () => this.splitGroup(group, "below") },
        { label: group.api.isMaximized() ? "Restore window" : "Maximize window", onClick: () =>
            group.api.isMaximized() ? group.api.exitMaximized() : group.api.maximize() },
        {
          label: "Float window",
          onClick: () =>
            this.dock.addFloatingGroup(group, {
              position: { left: 100, top: 80 },
              width: 560,
              height: 420,
            }),
        },
      );
    } else if (group) {
      items.push({ label: "Dock into workspace", onClick: () => group.api.moveTo({ position: "right" }) });
    }
    // Anchor sidebar panes can't be closed (they must always stay), so their menu omits the
    // Close entries — matching the hidden ✕ on their window header.
    if (!Workspace.ANCHOR_PANEL_IDS.includes(panelId)) {
      items.push(
        "sep",
        { label: "Close panel", danger: true, onClick: () => this.dock.panels.find((p) => p.id === panelId)?.api.close() },
        { label: "Close window", danger: true, onClick: () => group?.api.close() },
      );
    }
    return items;
  }

  private createLogView(panelId: string): IContentRenderer {
    return new DomPanel("dock-logview", (host, params) => {
      // The tab shows ● while this panel has view-state edits not yet in a named save
      // (Save Layout / Save Session clear it). Only re-set the title when it actually
      // changes — setTitle fires onDidLayoutChange, which must not loop the dirty flag.
      let baseTitle = "Log View";
      let applied = "";
      const applyTitle = () => {
        const decorated = (isDirty(panelId) ? "● " : "") + baseTitle;
        if (decorated === applied) return;
        applied = decorated;
        params.api.setTitle(decorated);
      };
      const view = new LogViewPanel(host, (title) => {
        baseTitle = title;
        applyTitle();
      });
      view.onUserEdit = () => markDirty(panelId);
      // Pin-OFF follow gate: the WORKING pane, not dockview's instantaneous active panel
      // (selecting a well activates the Wells tree — see activeViewer.ts).
      view.isActivePanel = () => isWorkingPane(panelId);
      const unsubDirty = subscribeDirty(applyTitle);
      const untrack = this.trackViewer(params);
      this.logViews.set(panelId, view);
      return () => {
        untrack();
        unsubDirty();
        clearDirty(panelId);
        this.logViews.delete(panelId);
        view.dispose();
      };
    });
  }

  /** The Wells pane (component id "wellsTops", kept for layout-restore compatibility). Tops now
   *  live in their OWN pane (see createTops) — Jauhar asked for them separated. The ObjectTree
   *  renders its own "Wells (N)" group header, so there is no static section title here (that was
   *  the duplicate "Wells" label). Tops follow the selection through appState, not a shared
   *  closure, so the two panes are fully independent. */
  private createWellsTops(): IContentRenderer {
    return new DomPanel("dock-wells", (host) => {
      host.innerHTML = `<div class="sidebar-section"><div class="sidebar-body dock-object-tree"></div></div>`;
      const tree = new ObjectTree(host.querySelector<HTMLElement>(".dock-object-tree")!);
      tree.onSelectWell = (well) => {
        // A different well invalidates the old well's top interval BEFORE the well
        // broadcast, so followers never see a foreign interval.
        if (appState.selectedWell.get()?.well_id !== well.well_id) {
          appState.selectedInterval.set(null);
        }
        appState.selectedWell.set(well);
        setStatus(`Selected well ${well.well_name}`);
      };
      // Switching a core set / survey / point-data set from the tree changes what every
      // panel reads, so it has to repaint like any other data change.
      tree.onDataChanged = () => this.notifyDataChanged();
      // Right-click routes from the tree into the panels that own the values.
      tree.onOpenCurveCatalog = (mnemonic) => this.openCurveCatalog(mnemonic);
      tree.onOpenDbInspector = () => this.openDbInspector();
      tree.onManageDataSets = (well) => void openDataSetsDialog(well, () => this.notifyDataChanged());
      tree.selectedWellId = appState.selectedWell.get()?.well_id ?? null;
      void tree.refresh();
      const unsub = appState.dataVersion.subscribe(() => {
        tree.selectedWellId = appState.selectedWell.get()?.well_id ?? null;
        void tree.refresh();
      });
      // Group changes from the manager (create/rename/delete/membership/active) refresh
      // the list independently of data changes. subscribe() fires once immediately; the
      // refresh above already rendered, so skip that first synchronous call.
      let firstGroups = true;
      const unsubGroups = appState.wellGroupsVersion.subscribe(() => {
        if (firstGroups) {
          firstGroups = false;
          return;
        }
        tree.selectedWellId = appState.selectedWell.get()?.well_id ?? null;
        void tree.refresh();
      });
      return () => {
        unsub();
        unsubGroups();
      };
    });
  }

  /** The Tops pane — its own dock panel now (split out of Wells at Jauhar's request). It follows
   *  the globally selected well via appState and windows the plots/log views by publishing the
   *  clicked top interval to appState.selectedInterval. */
  private createTops(): IContentRenderer {
    return new DomPanel("dock-tops", (host) => {
      host.innerHTML = `<div class="sidebar-section"><div class="sidebar-body dock-tops-panel"></div></div>`;
      const tops = new TopsPanel(host.querySelector<HTMLElement>(".dock-tops-panel")!);
      tops.onSelectInterval = (interval) => {
        appState.selectedInterval.set(interval);
        setStatus(
          interval
            ? `Windowed to top ${interval.topName} (${interval.depthMin.toFixed(1)}–${interval.depthMax?.toFixed(1) ?? "TD"}) — plots and log views follow`
            : "Top interval cleared — plots back to full depth",
        );
      };
      // Follow the selected well; refresh on data changes. BOTH subscriptions fire immediately, so
      // the selectedWell one already populates the pane for the current well — skip dataVersion's
      // first synchronous call rather than issuing a second identical list_tops and a second full
      // DOM rebuild on every pane open. Same primed-flag shape as the wellGroupsVersion guard in
      // createWellsTops above.
      const unsubWell = appState.selectedWell.subscribe((well) => void tops.refresh(well?.well_id ?? null));
      let firstData = true;
      const unsubData = appState.dataVersion.subscribe(() => {
        if (firstData) {
          firstData = false;
          return;
        }
        void tops.refresh(appState.selectedWell.get()?.well_id ?? null);
      });
      return () => {
        unsubWell();
        unsubData();
      };
    });
  }

  private createInspector(): IContentRenderer {
    return new DomPanel("dock-inspector", (host) => {
      host.innerHTML = `
        <div class="tabs">
          <button class="tab-btn active" data-tab="equation">Equation Editor</button>
          <button class="tab-btn" data-tab="catalog">Curve Catalog</button>
        </div>
        <div class="tab-content" id="tab-equation"></div>
        <div class="tab-content" id="tab-catalog" hidden></div>`;
      const inspector = new InspectorPanel(host);
      inspector.getSelectedWellId = () => appState.selectedWell.get()?.well_id ?? null;
      const unsubData = appState.dataVersion.subscribe(() => void inspector.refreshCatalog());
      const unsubWell = appState.selectedWell.subscribe(() => void inspector.refreshCatalog());
      // Published so "Open in Curve Catalog" (Wells-pane right-click) can land on the row it
      // means, rather than dumping the user in an unfiltered catalog to hunt for it.
      this.inspectorPanel = inspector;
      return () => {
        if (this.inspectorPanel === inspector) this.inspectorPanel = null;
        unsubData();
        unsubWell();
        // The panel owns a CodeMirror EditorView whose window/document listeners only come off in
        // destroy(); unsubscribing alone would strand it. Same as dbInspector/history above.
        inspector.dispose();
      };
    });
  }

  private createPlot(kind: PlotKind): IContentRenderer {
    return new DomPanel(`dock-plot dock-${kind}`, (host, params) => {
      const build: (well: WellSummary, setStatus: (t: string) => void, initial?: Record<string, string>) => Promise<PlotContent> =
        kind === "histogram"
          ? buildHistogramContent
          : kind === "crossplot"
            ? buildCrossplotContent
            : kind === "pickett"
              ? buildPickettContent
              : kind === "correlation"
                ? buildCorrelationContent
                : (w, s, initial) => import("./vegaPanel").then((m) => m.buildVegaContent(w, s, initial));

      // The panel follows the Wells & Tops pane: selecting a different well rebuilds
      // the plot for it, carrying the curve/zone selections over (getState → initial).
      // The builders resolve async, so a generation counter drops stale builds and the
      // dispose (hover/interval subscriptions) always runs.
      let disposer: (() => void) | undefined;
      let getState: (() => Record<string, string>) | undefined;
      let generation = 0;
      let closed = false;
      let currentWellId: string | null = null;

      const rebuild = (well: WellSummary | null) => {
        const gen = ++generation;
        const initial = getState?.();
        disposer?.();
        disposer = undefined;
        getState = undefined;
        host.innerHTML = "";
        this.plotProps.delete(params.api.id);
        currentWellId = well?.well_id ?? null;
        // Correlation is inherently multi-well; every other plot needs the selected well.
        if (!well && kind !== "correlation") {
          host.innerHTML = `<div class="logview-message">Select a well (Wells &amp; Tops) — this ${escapeHtml(kind)} will follow.</div>`;
          return;
        }
        if (well && kind !== "correlation") {
          params.api.setTitle(`${kind[0].toUpperCase()}${kind.slice(1)} — ${well.well_name}`);
        }
        // well is only null for correlation, whose builder tolerates it.
        build(well as WellSummary, setStatus, initial)
          .then((content) => {
            if (closed || gen !== generation) {
              content.dispose?.();
              return;
            }
            host.appendChild(content.el);
            disposer = content.dispose;
            getState = content.getState;
            // Expose this plot's Properties dialog to the pane's right-click menu.
            if (content.openProperties) this.plotProps.set(params.api.id, content.openProperties);
          })
          .catch((err) => {
            if (closed || gen !== generation) return; // a newer build/close already won
            host.innerHTML = `<div class="logview-message">Failed to open ${escapeHtml(kind)}: ${escapeHtml(String(err))}</div>`;
          });
      };

      const unsubWell = appState.selectedWell.subscribe((well) => {
        if (kind === "correlation") {
          if (generation === 0) rebuild(null);
          return;
        }
        if (generation > 0 && (well?.well_id ?? null) === currentWellId) return;
        // Pin OFF = working-pane mode: an already-built plot only follows the selection
        // while it is the WORKING pane (fresh panels always build). Not api.isActive —
        // clicking a well activates the Wells tree, never a plot (see activeViewer.ts).
        if (generation > 0 && !appState.wellPinned.get() && !isWorkingPane(params.api.id)) return;
        rebuild(well);
      });

      const untrack = this.trackViewer(params);
      return () => {
        closed = true;
        untrack();
        this.plotProps.delete(params.api.id);
        unsubWell();
        disposer?.();
      };
    });
  }

  private defaultWorkspace(): void {
    const wells = this.dock.addPanel({ id: "wellsTops", component: "wellsTops", title: "Wells" });
    const log = this.dock.addPanel({
      id: this.freshId("logview"),
      component: "logview",
      title: "Log View",
      position: { referencePanel: wells, direction: "right" },
    });
    const inspector = this.dock.addPanel({
      id: "inspector",
      component: "inspector",
      title: "Inspector",
      position: { referencePanel: log, direction: "right" },
    });
    // Size the side panels explicitly (initialWidth is unreliable pre-layout);
    // the log view keeps the remaining center space.
    wells.api.setSize({ width: 260 });
    inspector.api.setSize({ width: 320 });
    log.api.setActive();
  }

  /** The Tops pane docks directly below the Wells pane (its own resizable panel, split out of the
   *  old combined Wells & Tops). Added when absent — after both the default build and a layout
   *  restore, so older saved layouts (which had Tops embedded in the wells pane) pick up the
   *  standalone pane too. An instance the user already moved elsewhere is left in place. */
  /** The Wells pane anchors the sidebar and can no longer be closed — but an OLD saved layout may
   *  predate that (the user closed it back when close was still allowed). Re-add it on restore so
   *  the sidebar always has its Wells pane; ensureTopsPane/monitors then dock beneath it. */
  private ensureWellsPane(): void {
    if (this.dock.panels.some((p) => p.id === "wellsTops")) return;
    this.dock.addPanel({ id: "wellsTops", component: "wellsTops", title: "Wells", position: { direction: "left" } });
  }

  private ensureTopsPane(): void {
    if (this.dock.panels.some((p) => p.id === "tops")) return;
    const wellsGroup = this.dock.panels.find((p) => p.id === "wellsTops")?.api.group;
    const panel = this.dock.addPanel({
      id: "tops",
      component: "tops",
      title: "Tops",
      position: wellsGroup ? { referenceGroup: wellsGroup, direction: "below" as const } : undefined,
    });
    // Tops are usually short — give the wells list the bulk of the sidebar height.
    panel.api.setSize({ height: 220 });
  }

  /** Processing + Performance monitors dock below the sidebar column (under Tops when it exists,
   *  else Wells) and default to visible (the user asked for them always showing). Adds whichever
   *  is missing — tabbed together in one group; an instance the user already moved elsewhere is
   *  left in place. Runs after both the default build AND a layout restore, so older saved layouts
   *  (which predate these panels) pick them up too. */
  private ensureMonitorsBelowWells(): void {
    const anchorGroup =
      this.dock.panels.find((p) => p.id === "tops")?.api.group ??
      this.dock.panels.find((p) => p.id === "wellsTops")?.api.group;
    let monitorGroup = this.dock.panels.find((p) => p.id === "processing")?.api.group;
    const add = (id: string, component: string, title: string): void => {
      if (this.dock.panels.some((p) => p.id === id)) return;
      const position = monitorGroup
        ? { referenceGroup: monitorGroup }
        : anchorGroup
          ? { referenceGroup: anchorGroup, direction: "below" as const }
          : undefined;
      const panel = this.dock.addPanel({ id, component, title, position });
      monitorGroup = panel.api.group; // the second monitor tabs into the first's group
    };
    add("processing", "processing", "Processing");
    add("health", "health", "Performance");
  }

  private restore(): boolean {
    const stored = localStorage.getItem(LAYOUT_STORAGE_KEY);
    if (!stored) return false;
    try {
      this.dock.fromJSON(JSON.parse(stored));
      // Track ids of restored log views: recreate the map keys is handled by createComponent.
      return this.dock.panels.length > 0;
    } catch (err) {
      console.warn("Workspace restore failed, using default:", err);
      localStorage.removeItem(LAYOUT_STORAGE_KEY);
      this.dock.clear();
      return false;
    }
  }

  resetWorkspace(): void {
    this.muteDirty();
    localStorage.removeItem(LAYOUT_STORAGE_KEY);
    this.dock.clear();
    this.logViews.clear();
    this.defaultWorkspace();
    this.ensureWellsPane();
    this.ensureTopsPane();
    this.ensureMonitorsBelowWells();
    this.ensureContentPlaceholder();
    this.lockAnchorGroups();
    clearDirty();
  }

  /** Suppresses workspace-dirty marking for a short window around programmatic layout
   *  changes and named saves (tab-title updates fire onDidLayoutChange too). */
  muteDirty(ms = 1500): void {
    this.dirtyMuteUntil = Math.max(this.dirtyMuteUntil, Date.now() + ms);
  }

  private freshId(prefix: string): string {
    this.counter += 1;
    return `${prefix}-${Date.now().toString(36)}-${this.counter}`;
  }

  /** Focus-or-open for the singleton panels; with `group` the panel opens in (or
   *  moves to) that window. */
  private openSingleton(id: string, component: string, title: string, group?: DockviewGroupPanel): void {
    const existing = this.dock.panels.find((p) => p.id === id);
    if (existing) {
      if (group && existing.api.group !== group) existing.api.moveTo({ group, position: "center" });
      existing.api.setActive();
    } else {
      this.dock.addPanel({
        id,
        component,
        title,
        position: group ? { referenceGroup: group } : { direction: component === "wellsTops" ? "left" : "right" },
      });
    }
    // A reopened/moved anchor (Wells & Tops / Processing / Performance) is a new group instance —
    // re-lock its width so it stays fixed like the rest of the sidebar.
    this.lockAnchorGroups();
  }

  openWellsTops(group?: DockviewGroupPanel): void {
    this.openSingleton("wellsTops", "wellsTops", "Wells", group);
  }

  openTops(group?: DockviewGroupPanel): void {
    this.openSingleton("tops", "tops", "Tops", group);
  }

  openInspector(group?: DockviewGroupPanel): void {
    this.openSingleton("inspector", "inspector", "Inspector", group);
  }

  /** The live Inspector, when one is open — set by `createInspector`. */
  private inspectorPanel: InspectorPanel | null = null;

  /** Opens the Inspector on its Curve Catalog tab, filtered to `mnemonic`. Used by the
   *  Wells-pane right-click so a curve can be inspected/edited where its values live.
   *  The panel may have just been created, so the focus is applied on the next frame. */
  openCurveCatalog(mnemonic?: string): void {
    this.openInspector();
    requestAnimationFrame(() => this.inspectorPanel?.focusCatalog(mnemonic ?? ""));
  }

  openDbInspector(group?: DockviewGroupPanel): void {
    this.openSingleton("dbInspector", "dbInspector", "Database Inspector", group);
  }

  openSqlQuery(group?: DockviewGroupPanel): void {
    this.openSingleton("sqlQuery", "sqlQuery", "SQL Query", group);
  }

  openDashboard(group?: DockviewGroupPanel): void {
    this.openSingleton("dashboard", "dashboard", "Field Dashboard", group);
  }

  openMap(group?: DockviewGroupPanel): void {
    this.openSingleton("map", "map", "Field Map", group);
  }

  openWorkflow(group?: DockviewGroupPanel): void {
    this.openSingleton("workflow", "workflow", "Workflow Builder", group);
  }

  /** Universal Processing panel — live progress + Cancel for every long op. Safe to call
   *  repeatedly (singleton); the Workflow Builder auto-opens it when a chain starts. */
  openProcessing(group?: DockviewGroupPanel): void {
    this.openSingleton("processing", "processing", "Processing", group);
  }

  /** Performance Monitor — CPU / system memory / USER + GDI object gauges. */
  openHealth(group?: DockviewGroupPanel): void {
    this.openSingleton("health", "health", "Performance", group);
  }

  /** Placeholder help copy keyed by panel kind — the seed for the future HTML help library.
   *  The Help tool (the ? in the quick-access bar) resolves the ACTIVE panel to its kind and
   *  shows this; module panes instead surface their manifest doc (spec.doc, the narration that
   *  used to sit in the form). When the illustrated HTML help set lands, swap an entry here for
   *  a link into it — this map is the single wiring point. */
  private static readonly PANEL_HELP: Record<string, string> = {
    logview:
      "Log View — the depth-track viewer. Add/arrange tracks and curves from the mini toolbar, right-click a curve to edit it, and pick a saved Layout from the Plot tab. It follows the well selected in Wells & Tops.",
    wellsTops:
      "Wells & Tops — the project's control panel. Pick a well to drive every plot, view and tool; window to a top to focus plots on that interval; manage well groups and pick/edit tops.",
    inspector:
      "Inspector — the Equation Editor (write custom curves in Python or the expression language) and the Curve Catalog (every stored curve, its versions and provenance).",
    dbInspector: "Database Inspector — browse and edit any project table directly, spreadsheet-style.",
    sqlQuery: "SQL Query — a read-only DuckDB SQL console over the project database.",
    history: "Processing History — the audit trail of everything done in this project, saveable to file.",
    dashboard: "Field Dashboard — field-wide pay and quality tiles across all wells (or the active group).",
    map: "Field Map — wells plotted by surface location; draw a polygon to capture them into a well group.",
    workflow: "Workflow Builder — chain modules into a repeatable pipeline and run it across many wells; progress shows in the Processing panel.",
    processing:
      "Processing — live progress, the well being worked, per-reason ✓/⚠/✗ outcomes and a Cancel button for every long operation. Expand 'details' for the grouped failure report.",
    health: "Performance — CPU, system memory and USER/GDI object-handle gauges for this session.",
    paysummary: "Cutoffs & Pay Summary — apply VSH/PHIE/SW cutoffs and summarise net/gross pay per zone and well.",
    cutoff: "Cutoff Sensitivity — sweep net pay against VSH/PHIE/SW cutoffs and read them off DST-highlighted crossplots.",
    ml: "Machine Learning — supervised prediction (regression/classification) and unsupervised clustering/reduction via scikit-learn.",
    montecarlo: "Monte Carlo — propagate input uncertainty through the interpretation to P10/P50/P90 volumes.",
    multimin: "SandiMin — the simultaneous probabilistic multi-mineral solver. (Full guide is being written.)",
    zones: "Zones — define depth zones and per-zone parameters that override the whole-well defaults used by modules.",
    autocorr: "Autocorrelate Tops — propagate a top from one well to others by matching a log's shape.",
    composite: "Composite Log — lay out a print-scale (1:200/500/1000) log plot and export it to SVG or PDF.",
    report: "Report — build a full PDF (cover, methodology, zone parameters, pay summary, composite pages); batch per well.",
    histogram: "Histogram — distribution and percentiles for any curve; pick matrix/shale parameters straight off it.",
    crossplot: "Crossplot — any curve pair, Z-coloured, with matrix/shale points you can drag to write zone parameters live.",
    pickett: "Pickett — log-log RT vs PHIE; drag the Sw=1 water line to read Rw and the cementation exponent m.",
    correlation: "Correlation — side-by-side well strips with tops connected; flatten on a datum top.",
  };

  /** Open contextual help for whichever panel is active — the quick-access ? button. */
  async openHelpForActivePanel(): Promise<void> {
    await this.openHelpForPanel(this.dock.activePanel ?? undefined);
  }

  /** Open help for a specific panel by id — used by the panel's right-click menu, where the
   *  right-clicked panel isn't necessarily the active one. */
  async openHelpForPanelId(panelId: string): Promise<void> {
    await this.openHelpForPanel(this.dock.panels.find((p) => p.id === panelId));
  }

  private async openHelpForPanel(panel?: IDockviewPanel): Promise<void> {
    if (!panel) {
      setStatus("Click a panel first, then press Help (?) for its guide.");
      return;
    }
    const title = panel.title ?? "Panel";
    let body: string;
    if (panel.id.startsWith("module:")) {
      const name = panel.id.slice("module:".length);
      const spec = await listModules()
        .then((all) => all.find((s) => s.name === name))
        .catch(() => undefined);
      body = spec?.doc ?? "Documentation for this module is unavailable.";
    } else {
      const kind = panel.id.split("-")[0];
      body = Workspace.PANEL_HELP[kind] ?? "Documentation for this panel is coming soon.";
    }
    const content = document.createElement("div");
    const bodyEl = document.createElement("p");
    bodyEl.className = "help-body";
    bodyEl.textContent = body;
    content.appendChild(bodyEl);
    const note = document.createElement("p");
    note.className = "help-note";
    note.textContent = "Illustrated help for each panel will open here in a later release.";
    content.appendChild(note);
    openModal(`Help — ${title}`, content, 480);
  }

  openHistory(group?: DockviewGroupPanel): void {
    this.openSingleton("history", "history", "Processing History", group);
  }

  openPaySummary(group?: DockviewGroupPanel): void {
    this.openSingleton("paysummary", "paysummary", "Cutoffs & Pay Summary", group);
  }

  openCutoff(group?: DockviewGroupPanel): void {
    this.openSingleton("cutoff", "cutoff", "Cutoff Sensitivity", group);
  }

  openMl(group?: DockviewGroupPanel): void {
    this.openSingleton("ml", "ml", "Machine Learning", group);
  }

  openMonteCarlo(group?: DockviewGroupPanel): void {
    this.openSingleton("montecarlo", "montecarlo", "Monte Carlo", group);
  }

  openShf(group?: DockviewGroupPanel): void {
    this.openSingleton("shf", "shf", "SHF Fit (Cuddy FOIL)", group);
  }

  openThomeer(group?: DockviewGroupPanel): void {
    this.openSingleton("thomeer", "thomeer", "Pc Fit (Thomeer)", group);
  }

  openHfu(group?: DockviewGroupPanel): void {
    this.openSingleton("hfu", "hfu", "HFU Clustering (FZI)", group);
  }

  openLorenz(group?: DockviewGroupPanel): void {
    this.openSingleton("lorenz", "lorenz", "Lorenz Plot (flow units)", group);
  }

  openFaciesTie(group?: DockviewGroupPanel): void {
    this.openSingleton("faciesTie", "faciesTie", "Facies Tie-in", group);
  }

  openUnconventional(group?: DockviewGroupPanel): void {
    this.openSingleton("unconventional", "unconventional", "Unconventional (ΔlogR + Langmuir)", group);
  }

  openResultsQc(group?: DockviewGroupPanel): void {
    this.openSingleton("resultsQc", "resultsQc", "Results QC", group);
  }

  openMultimin(group?: DockviewGroupPanel): void {
    this.openSingleton("multimin", "multimin", "SandiMin — Mineral Solver", group);
  }

  /** Every manifest module opens as its own singleton pane (id "module:<name>"), so new
   *  backend modules get a dockable pane automatically. */
  openModulePane(spec: ModuleSpec, group?: DockviewGroupPanel): void {
    this.openSingleton(`module:${spec.name}`, "module", spec.title, group);
  }

  openZones(group?: DockviewGroupPanel): void {
    this.openSingleton("zones", "zones", "Zones", group);
  }

  openAutoCorr(group?: DockviewGroupPanel): void {
    this.openSingleton("autocorr", "autocorr", "Autocorrelate Tops", group);
  }

  openComposite(group?: DockviewGroupPanel): void {
    this.openSingleton("composite", "composite", "Composite Log", group);
  }

  openReport(group?: DockviewGroupPanel): void {
    this.openSingleton("report", "report", "Report", group);
  }

  openLogView(group?: DockviewGroupPanel): void {
    const panel = this.dock.addPanel({
      id: this.freshId("logview"),
      component: "logview",
      title: "Log View",
      ...(group ? { position: { referenceGroup: group } } : {}),
    });
    panel.api.setActive();
  }

  openPlot(kind: PlotKind, group?: DockviewGroupPanel): void {
    const well = appState.selectedWell.get();
    const title =
      kind === "correlation"
        ? "Correlation"
        : `${kind[0].toUpperCase()}${kind.slice(1)}${well ? ` — ${well.well_name}` : ""}`;
    const panel = this.dock.addPanel({
      id: this.freshId(kind),
      component: kind,
      title,
      ...(group ? { position: { referenceGroup: group } } : {}),
    });
    panel.api.setActive();
  }

  /** Adds an empty window to the right edge of the workspace — fill it via its ＋
   *  button or by dragging panel tabs into it. */
  newWindow(): void {
    this.dock.addGroup({ direction: "right" });
  }

  /** The most recently active log view — target for ribbon scale/zoom/layout actions. */
  activeLogView(): LogViewPanel | null {
    return this.activeLogViewEntry()?.view ?? null;
  }

  /** Same resolution as {@link activeLogView} but with the dock panel id, so a named
   *  save can clear exactly that panel's unsaved (●) marker. */
  activeLogViewEntry(): { id: string; view: LogViewPanel } | null {
    const active = this.dock.activePanel;
    if (active && this.logViews.has(active.id)) return { id: active.id, view: this.logViews.get(active.id)! };
    // Fall back to any open log view (e.g. a plot panel is focused).
    const first = this.logViews.entries().next();
    return first.done ? null : { id: first.value[0], view: first.value[1] };
  }

  /** Called after LAS import or module/equation runs so every panel refreshes. */
  notifyDataChanged(): void {
    bumpDataVersion();
  }

  /** Capture the current workspace as a named session (dock layout + active well +
   *  each log view's chosen layout). */
  snapshotSession(): SessionSnapshot {
    const logViewLayouts: Record<string, Layout> = {};
    for (const [panelId, view] of this.logViews) {
      const layout = view.getLayout();
      if (layout) logViewLayouts[panelId] = layout;
    }
    return {
      version: 1,
      layout: this.dock.toJSON(),
      well: appState.selectedWell.get(),
      logViewLayouts,
    };
  }

  /** Restore a session saved with {@link snapshotSession}: rebuild the dock layout, point
   *  it at the session's well so every recreated plot/log view loads that data, then
   *  reapply each log view's saved layout (which dockview's own restore doesn't carry). */
  applySession(snap: SessionSnapshot): void {
    this.muteDirty(3000);
    this.logViews.clear();
    this.dock.clear();
    // Point app state at the session's well BEFORE fromJSON, so panels recreated by the
    // restore read the right well at init (tree highlight, plots, log views all follow).
    appState.selectedInterval.set(null);
    if (snap.well) appState.selectedWell.set(snap.well);
    this.dock.fromJSON(snap.layout);
    // fromJSON has synchronously recreated the log-view panels (repopulating logViews via
    // createLogView), so their ids now resolve; reapply each saved layout.
    if (snap.logViewLayouts) {
      for (const [panelId, layout] of Object.entries(snap.logViewLayouts)) {
        this.logViews.get(panelId)?.setLayout(layout);
      }
    }
    this.ensureTopsPane(); // pre-split snapshots had Tops embedded in the wells pane
    this.ensureContentPlaceholder();
    this.lockAnchorGroups();
    // Everything now matches the applied session — nothing is "unsaved".
    clearDirty();
  }

  /** After the constructor's own dock restore on a NORMAL launch: re-applies the parts
   *  of the autosave that dockview's layout JSON doesn't carry — the active well and
   *  each log view's layout. Panel ids that no longer exist are skipped. */
  applyAutosaveExtras(snap: SessionSnapshot): void {
    this.muteDirty(5000); // async well/title loads must not mark a fresh boot dirty
    if (snap.well) {
      appState.selectedInterval.set(null);
      appState.selectedWell.set(snap.well);
    }
    if (snap.logViewLayouts) {
      for (const [panelId, layout] of Object.entries(snap.logViewLayouts)) {
        this.logViews.get(panelId)?.setLayout(layout);
      }
    }
  }
}
