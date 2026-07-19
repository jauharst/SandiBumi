import { LogCanvasRenderer } from "../LogCanvasRenderer";
import { getCoreData, getTrackData, listCurveCatalog, type Layout, type TrackCurveSeries, type WellSummary } from "../ipc";
import { appState, setStatus } from "../state";
import { pushUndo } from "../undo";
import { openLayoutPropsDialog } from "./layoutPropsDialog";
import { CORE_OVERLAY_MAP } from "./plotCommon";
import { TopsEditor } from "./topsEditor";
import { renderDepthAxis, renderReadout, renderReportHeader, renderTrackHeaders } from "./viewerChrome";

/** One dockable log-layout viewer: its own WebGPU canvas + renderer, mini view toolbar
 *  (depth scale/zoom/track width/pin), track headers, depth axis, cursor readout,
 *  compact report header, and per-panel view state (layout copy, track weights, hidden
 *  curves, well). Multiple instances coexist — every panel follows global well selection
 *  unless the user PINS it to its well (📌 in the toolbar), which is how side-by-side
 *  multi-well viewing works. */
export class LogViewPanel {
  readonly root: HTMLElement;
  private headersEl: HTMLElement;
  private depthAxisEl: HTMLElement;
  private readoutEl: HTMLElement;
  private reportEl: HTMLElement;
  private canvas: HTMLCanvasElement;
  private coreOverlay: HTMLCanvasElement;
  private messageEl: HTMLElement;
  private crosshairEl: HTMLElement;
  private lastHoverDepth: number | null = null;
  /** Core plug series (CPOR/CPERM/CGD/CSW) for the loaded well; empty = no overlay. */
  private coreByName = new Map<string, TrackCurveSeries>();
  /** Tops overlay + Petrel-style interactive editor (🏷 in the toolbar toggles editing). */
  private topsEditor!: TopsEditor;

  private renderer: LogCanvasRenderer | null = null;
  private layout: Layout | null = null;
  private series: TrackCurveSeries[] = [];
  private trackWeights = new Map<string, number>();
  private hiddenCurves = new Set<string>();
  private widthScalePct = 100;
  private well: WellSummary | null = null;

  private unsubscribers: (() => void)[] = [];
  private resizeObserver: ResizeObserver | null = null;
  private setTitle: (title: string) => void;

  /** Fired on user edits to this panel's view state (layout properties, track widths,
   *  curve visibility) — the workspace uses it to mark the panel tab unsaved (●). */
  onUserEdit?: () => void;
  /** Set by the workspace: whether this panel is the active dock panel. With the well
   *  pin OFF, only the active panel follows well selection (working-pane mode). */
  isActivePanel?: () => boolean;

  constructor(container: HTMLElement, setTitle: (title: string) => void) {
    this.setTitle = setTitle;

    this.root = document.createElement("div");
    this.root.className = "logview-panel";
    this.reportEl = document.createElement("div");
    this.reportEl.className = "logview-report";
    this.reportEl.hidden = true;
    this.headersEl = document.createElement("div");
    this.headersEl.className = "track-headers";
    const body = document.createElement("div");
    body.className = "logview-body";
    this.depthAxisEl = document.createElement("div");
    this.depthAxisEl.className = "depth-axis";
    this.canvas = document.createElement("canvas");
    this.canvas.className = "log-canvas";
    this.coreOverlay = document.createElement("canvas");
    this.coreOverlay.className = "core-overlay";
    this.readoutEl = document.createElement("div");
    this.readoutEl.className = "cursor-readout";
    this.readoutEl.hidden = true;
    this.crosshairEl = document.createElement("div");
    this.crosshairEl.className = "depth-crosshair";
    this.crosshairEl.hidden = true;
    this.messageEl = document.createElement("div");
    this.messageEl.className = "logview-message";
    body.appendChild(this.depthAxisEl);
    body.appendChild(this.canvas);
    body.appendChild(this.coreOverlay);
    this.topsEditor = new TopsEditor(body, this.canvas, () => this.renderer?.getVisibleDepthRange() ?? [0, 0]);
    body.appendChild(this.crosshairEl);
    body.appendChild(this.readoutEl);
    body.appendChild(this.messageEl);
    this.root.appendChild(this.reportEl);
    this.root.appendChild(this.buildTools());
    this.root.appendChild(this.headersEl);
    this.root.appendChild(body);
    container.appendChild(this.root);

    // Take a private copy of the active layout so per-panel edits don't leak.
    const active = appState.activeLayout.get();
    if (active) this.adoptLayout(active);

    void this.initRenderer(body);
  }

  /** Per-panel view controls (moved here from the View ribbon so every log view carries
   *  its own tools): depth scale presets, zoom, track width, reset, properties, pin. */
  private buildTools(): HTMLElement {
    const bar = document.createElement("div");
    bar.className = "logview-tools";

    const btn = (text: string, title: string, onClick: () => void): HTMLButtonElement => {
      const b = document.createElement("button");
      b.className = "lv-btn";
      b.title = title;
      b.textContent = text;
      b.addEventListener("click", onClick);
      bar.appendChild(b);
      return b;
    };
    const sep = () => {
      const s = document.createElement("span");
      s.className = "lv-sep";
      bar.appendChild(s);
    };

    const scaleSel = document.createElement("select");
    scaleSel.className = "lv-scale";
    scaleSel.title = "Vertical depth scale";
    for (const ratio of [20, 50, 100, 200, 240, 500, 1000, 2000]) {
      const opt = document.createElement("option");
      opt.value = String(ratio);
      opt.textContent = `1:${ratio}`;
      scaleSel.appendChild(opt);
    }
    scaleSel.value = "100";
    scaleSel.addEventListener("change", () => {
      const ratio = parseFloat(scaleSel.value);
      this.setScale(96 / ratio);
      setStatus(`Vertical scale set to 1:${ratio}`);
    });
    bar.appendChild(scaleSel);
    btn("−", "Zoom out", () => this.stepZoom(1 / 1.25));
    btn("＋", "Zoom in", () => this.stepZoom(1.25));
    sep();

    const widthLabel = document.createElement("span");
    widthLabel.className = "lv-label";
    widthLabel.textContent = "Tracks";
    bar.appendChild(widthLabel);
    const pct = document.createElement("span");
    btn("−", "Narrow all tracks", () => (pct.textContent = `${this.scaleAllTracks(0.9).pct}%`));
    btn("＋", "Widen all tracks", () => (pct.textContent = `${this.scaleAllTracks(1 / 0.9).pct}%`));
    pct.className = "lv-label";
    pct.textContent = "100%";
    bar.appendChild(pct);
    sep();

    btn("⟳", "Reset view (top of well, default scale)", () => this.resetView());
    btn("⚙", "Layout properties…", () => void this.openProperties());
    sep();

    const topsBtn = btn("🏷", "Edit tops: click to add, drag to move, double-click to rename/delete", () => {
      const on = !this.topsEditor.editing;
      this.topsEditor.setEditMode(on);
      topsBtn.classList.toggle("active", on);
      setStatus(on ? "Tops editing ON — click adds, drag moves, double-click edits" : "Tops editing off");
    });

    return bar;
  }

  private async initRenderer(body: HTMLElement): Promise<void> {
    this.canvas.width = this.canvas.clientWidth || 400;
    this.canvas.height = this.canvas.clientHeight || 400;
    this.renderer = new LogCanvasRenderer(this.canvas);
    this.renderer.onViewSettled = () => {
      this.refreshDepthAxis();
      this.positionCrosshair(this.lastHoverDepth);
    };
    // Redraw core points + tops lines after every rendered frame so they track pan/zoom.
    this.renderer.onFrameRendered = () => {
      this.drawCoreOverlay();
      this.topsEditor.draw();
    };
    this.renderer.onCursorMove = (depth, samples, trackTitle) => {
      // Emphasize the hovered track's curves in the readout and tint its header.
      const track = trackTitle ? this.layout?.tracks.find((t) => t.title === trackTitle) : undefined;
      const emphasize = track ? new Set(track.curves.map((c) => c.curve_name)) : undefined;
      renderReadout(this.readoutEl, depth, samples, emphasize);
      this.highlightTrack(trackTitle);
      // Broadcast so every other open log view draws a synchronized crosshair.
      appState.hoverDepth.set(depth);
    };

    try {
      await this.renderer.init();
      this.message("");
    } catch (err) {
      console.error("WebGPU init failed:", err);
      this.message("WebGPU unavailable — viewer disabled");
      this.renderer = null;
    }

    this.resizeObserver = new ResizeObserver(() => {
      this.renderer?.resize();
      this.refreshDepthAxis();
      this.drawCoreOverlay();
      this.topsEditor.draw();
    });
    this.resizeObserver.observe(body);

    // Pin ON (default): every log view follows the well selection. Pin OFF: viewers
    // keep their wells and only the active panel follows — side-by-side multi-well.
    this.unsubscribers.push(
      appState.selectedWell.subscribe((well) => {
        if (!well || well.well_id === this.well?.well_id) return;
        if (!appState.wellPinned.get() && this.well && !(this.isActivePanel?.() ?? true)) return;
        void this.loadWell(well);
      }),
      appState.dataVersion.subscribe(() => {
        if (this.well) void this.loadWell(this.well, true);
      }),
      // Repaint on theme change so the WebGPU clear colour + curve colours (read from CSS
      // vars at draw time) follow the new palette without waiting for an interaction.
      // Just mark the frame dirty — reloading well data here was async and well-gated,
      // which left empty or slow panels showing the old palette until a mouse move.
      appState.themeVersion.subscribe(() => {
        this.renderer?.repaint();
        this.refreshDepthAxis();
        this.drawCoreOverlay();
        this.topsEditor.draw();
      }),
      appState.hoverDepth.subscribe((depth) => this.positionCrosshair(depth)),
      // A top clicked in the Wells & Tops pane scrolls every view of that well to it.
      appState.selectedInterval.subscribe((interval) => {
        if (!interval || !this.renderer || this.well?.well_id !== interval.wellId) return;
        this.renderer.scrollToDepth(interval.depthMin);
      }),
    );
  }

  /** Tints the header of the track under the cursor (null clears the highlight). */
  private highlightTrack(title: string | null): void {
    for (const el of this.headersEl.querySelectorAll<HTMLElement>(".track-header")) {
      el.classList.toggle("hover", title !== null && el.dataset.track === title);
    }
  }

  /** Places the synchronized crosshair line at `depth` (hidden when null/off-screen). */
  private positionCrosshair(depth: number | null): void {
    this.lastHoverDepth = depth;
    if (depth === null || !this.renderer) {
      this.crosshairEl.hidden = true;
      return;
    }
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (depth < top || depth > bottom || bottom === top) {
      this.crosshairEl.hidden = true;
      return;
    }
    this.crosshairEl.hidden = false;
    this.crosshairEl.style.top = `${((depth - top) / (bottom - top)) * 100}%`;
  }

  private message(text: string): void {
    this.messageEl.textContent = text;
    this.messageEl.hidden = !text;
  }

  private adoptLayout(layout: Layout): void {
    this.layout = structuredClone(layout);
    this.trackWeights.clear();
    for (const t of this.layout.tracks) this.trackWeights.set(t.title, t.width_weight * 150);
    this.widthScalePct = 100;
    this.hiddenCurves.clear();
  }

  /** Ribbon layout picker targets the active panel through this. */
  setLayout(layout: Layout): void {
    this.adoptLayout(layout);
    this.updateTitle();
    if (this.well) void this.loadWell(this.well, true);
  }

  async loadWell(well: WellSummary, keepView = false): Promise<void> {
    if (!this.layout || !this.renderer) return;
    this.well = well;
    this.updateTitle();
    const curveNames = Array.from(new Set(this.layout.tracks.flatMap((t) => t.curves.map((c) => c.curve_name))));
    try {
      this.series = await getTrackData(well.well_id, curveNames, this.canvas.clientHeight || 400);
      if (!keepView) this.renderer.resetView();
      this.refresh();
      setStatus(`Loaded well ${well.well_name}`);
    } catch (err) {
      console.error("Failed to load track data:", err);
      setStatus(`Failed to load curve data: ${err}`);
    }
    try {
      const core = await getCoreData(well.well_id);
      this.coreByName = new Map(core.map((s) => [s.curve_name, s]));
    } catch {
      this.coreByName = new Map(); // no backend or no core data — overlay simply stays empty
    }
    this.drawCoreOverlay();
    await this.topsEditor.setWell(well.well_id);
  }

  /** Draws core plug points over any track showing a curve with a core counterpart
   *  (PHIE/PHIT→CPOR, PERM*→CPERM, RHOB→CGD, SWE/SWT→CSW): diamonds in the curve's
   *  color at the plug's depth, positioned on that curve's own scale. Values outside
   *  the track's scale range are skipped, never clamped to a false position. */
  private drawCoreOverlay(): void {
    const w = this.canvas.clientWidth;
    const h = this.canvas.clientHeight;
    if (w === 0 || h === 0) return;
    this.coreOverlay.style.left = `${this.canvas.offsetLeft}px`;
    this.coreOverlay.style.top = `${this.canvas.offsetTop}px`;
    if (this.coreOverlay.width !== w || this.coreOverlay.height !== h) {
      this.coreOverlay.width = w;
      this.coreOverlay.height = h;
    }
    const ctx = this.coreOverlay.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);
    if (!this.renderer || !this.layout || this.coreByName.size === 0) return;

    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;
    ctx.lineWidth = 1;
    ctx.strokeStyle = "rgba(0, 0, 0, 0.65)";

    for (const range of this.renderer.getTrackRanges()) {
      const track = this.layout.tracks.find((t) => t.title === range.title);
      if (!track) continue;
      for (const curve of track.curves) {
        if (this.hiddenCurves.has(curve.curve_name)) continue;
        const coreName = CORE_OVERLAY_MAP[curve.curve_name.toUpperCase()];
        const series = coreName ? this.coreByName.get(coreName) : undefined;
        if (!series || series.depth.length === 0) continue;

        const log = track.scale_type === "log";
        const lo = log ? Math.log10(Math.max(curve.min, 1e-6)) : curve.min;
        const hi = log ? Math.log10(Math.max(curve.max, 1e-6)) : curve.max;
        if (hi === lo) continue;
        const left = range.leftFrac * w;
        const span = (range.rightFrac - range.leftFrac) * w;
        ctx.fillStyle = curve.color;

        for (let i = 0; i < series.depth.length; i++) {
          const d = series.depth[i];
          if (d < top || d > bottom) continue;
          const v = series.value[i];
          if (!Number.isFinite(v)) continue;
          const tv = log ? Math.log10(Math.max(v, 1e-6)) : v;
          const frac = (tv - lo) / (hi - lo);
          if (frac < 0 || frac > 1) continue;
          const x = left + frac * span;
          const y = ((d - top) / (bottom - top)) * h;
          ctx.beginPath();
          ctx.moveTo(x, y - 4);
          ctx.lineTo(x + 4, y);
          ctx.lineTo(x, y + 4);
          ctx.lineTo(x - 4, y);
          ctx.closePath();
          ctx.fill();
          ctx.stroke();
        }
        break; // one core series per track — the first curve with a counterpart wins
      }
    }
  }

  private updateTitle(): void {
    const parts = [this.well?.well_name ?? "Log View", this.layout?.name].filter(Boolean);
    this.setTitle(parts.join(" — "));
  }

  private refreshDepthAxis(): void {
    if (!this.renderer) return;
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    renderDepthAxis(this.depthAxisEl, top, bottom);
  }

  refresh(): void {
    if (!this.layout || !this.renderer) return;
    renderTrackHeaders(this.headersEl, this.layout, this.trackWeights, this.hiddenCurves, {
      onLayoutMutated: () => {
        this.onUserEdit?.();
        this.refresh();
      },
      onCurveToggle: (curveName, hidden) => {
        this.onUserEdit?.();
        this.renderer?.setCurveHidden(curveName, hidden);
      },
    });
    this.renderer.loadLayout(this.layout, this.series, this.trackWeights);
    for (const curveName of this.hiddenCurves) this.renderer.setCurveHidden(curveName, true);
    renderReportHeader(this.reportEl, this.well, this.renderer.getDataDepthRange());
    this.refreshDepthAxis();
    this.drawCoreOverlay();
  }

  // --- Ribbon actions (routed to the active panel by the workspace) ---

  /** A copy of this panel's current layout (for "Save Layout…"). */
  getLayout(): Layout | null {
    return this.layout ? structuredClone(this.layout) : null;
  }

  /** Opens the Layout Properties dialog for this panel's private layout copy. */
  async openProperties(): Promise<void> {
    if (!this.layout) return;
    let available: string[] = [];
    try {
      available = (await listCurveCatalog()).map((e) => e.name);
    } catch {
      // No backend (or empty DB) — fall back to the curves the layout already references.
      available = Array.from(new Set(this.layout.tracks.flatMap((t) => t.curves.map((c) => c.curve_name))));
    }
    const before = structuredClone(this.layout);
    openLayoutPropsDialog(this.layout, available, (edited) => {
      this.applyLayoutEdit(edited);
      // Layout property changes are undoable (Ctrl+Z restores the previous tracks/styles).
      pushUndo({
        label: `layout properties (${edited.name})`,
        undo: () => this.applyLayoutEdit(structuredClone(before)),
        redo: () => this.applyLayoutEdit(structuredClone(edited)),
      });
    });
  }

  /** Swaps in an edited layout, keeping user-dragged widths for surviving tracks. */
  private applyLayoutEdit(edited: Layout): void {
    this.onUserEdit?.();
    const oldWeights = new Map(this.trackWeights);
    this.layout = edited;
    this.trackWeights.clear();
    for (const t of edited.tracks) {
      this.trackWeights.set(t.title, oldWeights.get(t.title) ?? t.width_weight * 150);
    }
    if (this.well) void this.loadWell(this.well, true);
    else this.refresh();
  }

  setScale(pxPerUnit: number): void {
    this.renderer?.setScale(pxPerUnit);
    this.refreshDepthAxis();
  }

  /** Resets pan/zoom to the top of the well at the default depth scale. */
  resetView(): void {
    this.renderer?.resetView();
    this.refreshDepthAxis();
  }

  stepZoom(factor: number): void {
    this.renderer?.stepZoom(factor);
    this.refreshDepthAxis();
  }

  scaleAllTracks(factor: number): { pct: number } {
    this.onUserEdit?.();
    this.widthScalePct = Math.max(30, Math.min(300, Math.round(this.widthScalePct * factor)));
    if (this.layout) {
      for (const t of this.layout.tracks) {
        const base = t.width_weight * 150;
        this.trackWeights.set(t.title, Math.max(36, Math.round((base * this.widthScalePct) / 100)));
      }
    }
    this.refresh();
    return { pct: this.widthScalePct };
  }

  dispose(): void {
    for (const unsub of this.unsubscribers) unsub();
    this.resizeObserver?.disconnect();
    this.topsEditor.dispose();
    this.renderer?.dispose();
    this.renderer = null;
  }
}
