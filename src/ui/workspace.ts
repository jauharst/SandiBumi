import {
  DockviewComponent,
  type CreateComponentOptions,
  type DockviewGroupPanel,
  type GroupPanelPartInitParameters,
  type IContentRenderer,
  type IHeaderActionsRenderer,
} from "dockview-core";
import "dockview-core/dist/styles/dockview.css";
import { appState, bumpDataVersion, setStatus } from "../state";
import { WORKSPACE_DIRTY, clearDirty, isDirty, markDirty, subscribeDirty } from "../dirty";
import { listModules, type Layout, type ModuleSpec, type WellSummary } from "../ipc";
import { recordProcess } from "../processLog";
import { LogViewPanel } from "./logViewPanel";
import { ObjectTree } from "./objectTree";
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

const LAYOUT_STORAGE_KEY = "sandibumi.workspace";

type PlotKind = "histogram" | "crossplot" | "pickett" | "correlation";

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
  private logViews = new Map<string, LogViewPanel>();
  private counter = 0;
  /** Layout-change events before this time don't mark the workspace dirty — set around
   *  programmatic rebuilds (applySession/reset) and named saves, whose tab-title updates
   *  also fire onDidLayoutChange and would otherwise re-dirty a just-saved workspace. */
  private dirtyMuteUntil = 0;

  constructor(container: HTMLElement) {
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
    const resizeObserver = new ResizeObserver(() => {
      this.relayoutKeepingPaneSizes(container.clientWidth, container.clientHeight);
    });
    resizeObserver.observe(container);

    if (!this.restore()) this.defaultWorkspace();

    // Constructor's own restore/default build happens above this subscription, so it
    // never marks the workspace dirty; only later (user) arrangement changes do.
    this.muteDirty();
    let saveHandle: number | undefined;
    this.dock.onDidLayoutChange(() => {
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

  /** Window resize should NOT proportionally squeeze every pane (dockview's hardcoded
   *  behavior): side panes keep their user-set size and only the largest pane — the main
   *  content area — absorbs the delta. Snapshot sizes, layout, restore all but the largest.
   *  Sash drags are untouched (they don't go through this path), so manual pane resizing
   *  still works exactly as before. */
  private relayoutKeepingPaneSizes(width: number, height: number): void {
    const groups = this.dock.groups.filter((g) => g.api.location.type === "grid");
    const before = groups.map((g) => ({ g, w: g.width, h: g.height }));
    let largest = before[0];
    for (const entry of before) {
      if (entry.w * entry.h > (largest?.w ?? 0) * (largest?.h ?? 0)) largest = entry;
    }
    this.dock.layout(width, height);
    for (const { g, w, h } of before) {
      if (g === largest?.g) continue;
      if (g.width !== w || g.height !== h) g.api.setSize({ width: w, height: h });
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
    btn("✕", "Close this window and every panel in it", () => group.api.close());

    // Grid windows can split/float/maximize; floating windows dock back instead.
    const sync = () => {
      const floating = group.api.location.type !== "grid";
      floatBtn.style.display = floating ? "none" : "";
      maxBtn.style.display = floating ? "none" : "";
      splitVBtn.style.display = floating ? "none" : "";
      splitHBtn.style.display = floating ? "none" : "";
      dockBtn.style.display = floating ? "" : "none";
    };
    sync();
    const sub = group.api.onDidLocationChange(() => sync());

    return { element, init: () => {}, dispose: () => sub.dispose() };
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
      ["SandiMin Solver", () => this.openMultimin(group)],
      "sep",
      ["Zones", () => this.openZones(group)],
      ["Autocorrelate Tops", () => this.openAutoCorr(group)],
      ["Composite Log", () => this.openComposite(group)],
      ["Report", () => this.openReport(group)],
      "sep",
      ["Wells & Tops", () => this.openWellsTops(group)],
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
      case "wellsTops":
        return this.createWellsTops();
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
              onRunComplete: () => {
                recordProcess("Module", `Ran ${spec.title}`, appState.selectedWell.get()?.well_name ?? null);
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
      case "histogram":
      case "crossplot":
      case "pickett":
      case "correlation":
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
          host.innerHTML = `<div class="logview-message">Failed to open ${label}: ${err}</div>`;
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
          host.innerHTML = `<div class="logview-message">Select a well (Wells &amp; Tops) — ${label} will follow.</div>`;
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
            host.innerHTML = `<div class="logview-message">Failed to open ${label}: ${err}</div>`;
          });
      };

      const unsubWell = appState.selectedWell.subscribe((well) => {
        const nextId = well?.well_id ?? null;
        if (generation > 0 && nextId === currentWell?.well_id) return;
        // Pin OFF = working-pane mode: an already-built pane holding a well only follows
        // the selection to ANOTHER real well while it is the active panel. Always rebuild
        // when the pane has no well yet (catch up to a selection) or the selection was
        // cleared (project switch), so a stale well can never linger in the pane.
        if (
          generation > 0 &&
          currentWell !== null &&
          nextId !== null &&
          !appState.wellPinned.get() &&
          !params.api.isActive
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

      return () => {
        closed = true;
        unsubWell();
        unsubData();
        disposer?.();
      };
    });
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
      items.push(
        { heading: nice },
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

    // --- Window block (every panel) ---
    if (items.length) items.push("sep");
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
    items.push(
      "sep",
      { label: "Close panel", danger: true, onClick: () => this.dock.panels.find((p) => p.id === panelId)?.api.close() },
      { label: "Close window", danger: true, onClick: () => group?.api.close() },
    );
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
      view.isActivePanel = () => params.api.isActive;
      const unsubDirty = subscribeDirty(applyTitle);
      this.logViews.set(panelId, view);
      return () => {
        unsubDirty();
        clearDirty(panelId);
        this.logViews.delete(panelId);
        view.dispose();
      };
    });
  }

  private createWellsTops(): IContentRenderer {
    return new DomPanel("dock-wells", (host) => {
      host.innerHTML = `
        <div class="sidebar-section">
          <div class="sidebar-title">Wells</div>
          <div class="sidebar-body dock-object-tree"></div>
        </div>
        <div class="sidebar-section">
          <div class="sidebar-title">Tops</div>
          <div class="sidebar-body dock-tops-panel"></div>
        </div>`;
      const tree = new ObjectTree(host.querySelector<HTMLElement>(".dock-object-tree")!);
      const tops = new TopsPanel(host.querySelector<HTMLElement>(".dock-tops-panel")!);
      tree.onSelectWell = (well) => {
        // A different well invalidates the old well's top interval BEFORE the well
        // broadcast, so followers never see a foreign interval.
        if (appState.selectedWell.get()?.well_id !== well.well_id) {
          appState.selectedInterval.set(null);
        }
        appState.selectedWell.set(well);
        setStatus(`Selected well ${well.well_name}`);
        void tops.refresh(well.well_id);
      };
      tops.onSelectInterval = (interval) => {
        appState.selectedInterval.set(interval);
        setStatus(
          interval
            ? `Windowed to top ${interval.topName} (${interval.depthMin.toFixed(1)}–${interval.depthMax?.toFixed(1) ?? "TD"}) — plots and log views follow`
            : "Top interval cleared — plots back to full depth",
        );
      };
      tree.selectedWellId = appState.selectedWell.get()?.well_id ?? null;
      void tree.refresh();
      const unsub = appState.dataVersion.subscribe(() => {
        tree.selectedWellId = appState.selectedWell.get()?.well_id ?? null;
        void tree.refresh();
        void tops.refresh(appState.selectedWell.get()?.well_id ?? null);
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
      return () => {
        unsubData();
        unsubWell();
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
              : buildCorrelationContent;

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
        currentWellId = well?.well_id ?? null;
        // Correlation is inherently multi-well; every other plot needs the selected well.
        if (!well && kind !== "correlation") {
          host.innerHTML = `<div class="logview-message">Select a well (Wells &amp; Tops) — this ${kind} will follow.</div>`;
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
          })
          .catch((err) => {
            host.innerHTML = `<div class="logview-message">Failed to open ${kind}: ${err}</div>`;
          });
      };

      const unsubWell = appState.selectedWell.subscribe((well) => {
        if (kind === "correlation") {
          if (generation === 0) rebuild(null);
          return;
        }
        if (generation > 0 && (well?.well_id ?? null) === currentWellId) return;
        // Pin OFF = working-pane mode: an already-built plot only follows the
        // selection while it is the active panel (fresh panels always build).
        if (generation > 0 && !appState.wellPinned.get() && !params.api.isActive) return;
        rebuild(well);
      });

      return () => {
        closed = true;
        unsubWell();
        disposer?.();
      };
    });
  }

  private defaultWorkspace(): void {
    const wells = this.dock.addPanel({ id: "wellsTops", component: "wellsTops", title: "Wells & Tops" });
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
      return;
    }
    this.dock.addPanel({
      id,
      component,
      title,
      position: group ? { referenceGroup: group } : { direction: component === "wellsTops" ? "left" : "right" },
    });
  }

  openWellsTops(group?: DockviewGroupPanel): void {
    this.openSingleton("wellsTops", "wellsTops", "Wells & Tops", group);
  }

  openInspector(group?: DockviewGroupPanel): void {
    this.openSingleton("inspector", "inspector", "Inspector", group);
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
