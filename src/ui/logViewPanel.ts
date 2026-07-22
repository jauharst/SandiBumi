import { LogCanvasRenderer } from "../LogCanvasRenderer";
import {
  getCoreData,
  getTrackData,
  listAuxData,
  listCurveCatalog,
  type AuxRow,
  type Layout,
  type TrackCurveSeries,
  type WellSummary,
} from "../ipc";
import { appState, setStatus } from "../state";
import { pushUndo } from "../undo";
import type { ContextMenuEntry } from "./contextMenu";
import { openCurveEditDialog } from "./curveEditDialog";
import { openLayoutPropsDialog } from "./layoutPropsDialog";
import { formRow, openModal } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";
import { CORE_OVERLAY_MAP, loadCurveUnits } from "./plotCommon";
import { HighlightsOverlay } from "./highlightsOverlay";
import { TopsEditor } from "./topsEditor";
import { renderDepthAxis, renderReadout, renderReportHeader, renderTrackHeaders } from "./viewerChrome";

type HeaderMode = "full" | "compact" | "collapsed";
type BorderStyle = { style: "solid" | "dashed" | "none"; width: number; color: string };

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
  /** Completion (casing/tubing/liner) + perforation rows for the loaded well, drawn in any
   *  track whose kind is "well_diagram". Both come from aux_data (COMPLETION / PERFORATION). */
  private wellDiagram: { casing: AuxRow[]; perfs: AuxRow[] } = { casing: [], perfs: [] };
  /** Tops overlay + Petrel-style interactive editor (🏷 in the toolbar toggles editing). */
  private topsEditor!: TopsEditor;
  /** Colored highlight bands + interactive editor (🖍 in the toolbar toggles editing). */
  private highlightsOverlay!: HighlightsOverlay;

  private renderer: LogCanvasRenderer | null = null;
  private layout: Layout | null = null;
  /** Curve name → unit, refreshed with each load (also on dataVersion). Feeds the cursor
   *  readout so RT shows "ohm.m", PHI "v/v", etc. Empty until the first successful load. */
  private curveUnits = new Map<string, string>();
  private series: TrackCurveSeries[] = [];
  private trackWeights = new Map<string, number>();
  private hiddenCurves = new Set<string>();
  private widthScalePct = 100;
  private well: WellSummary | null = null;
  /** Click a track (on the canvas or its header) to scope the hover readout to just
   *  that track's curves; null = follow whichever track the cursor is over. */
  private selectedTrack: string | null = null;
  /** ▤ cycles full → compact (inline chips, no scales) → collapsed (titles only), so a
   *  15-curve layout doesn't eat the screen. */
  private headerMode: HeaderMode = "full";
  /** Vertical separators between tracks, drawn on the overlay canvas. Empty color =
   *  the theme's --border. */
  private borders: BorderStyle = { style: "solid", width: 1, color: "" };

  /** The "1:N" depth-scale selector, kept as a field so the view-settled callback can
   *  re-sync it to the true live scale after a zoom/pan. */
  private scaleSel: HTMLSelectElement | null = null;
  private unsubscribers: (() => void)[] = [];
  private resizeObserver: ResizeObserver | null = null;
  /** Monotonic token so an out-of-order loadWell (fast well switching, or a dataVersion
   *  bump mid-load) can't render a stale well's series over a newer one. */
  private loadGen = 0;
  /** Set in dispose(); the un-awaited initRenderer checks it so subscriptions/observers
   *  aren't registered after the panel already closed (they would leak forever). */
  private disposed = false;
  /** A well switch (keepView=false) asks to reset the scroll; a coincident dataVersion
   *  refresh (keepView=true) can win the loadGen race, so carry the reset intent here
   *  and honour it on whichever load commits — else the new well inherits the old scroll. */
  private viewResetPending = false;
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
    // Highlights sit just below the tops layer (lower z-index) so top lines stay legible.
    this.highlightsOverlay = new HighlightsOverlay(body, this.canvas, () => this.renderer?.getVisibleDepthRange() ?? [0, 0]);
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
    scaleSel.title = "Vertical depth scale (true 1:N; shows the live scale after zooming)";
    for (const ratio of [20, 50, 100, 200, 240, 500, 1000, 2000, 5000]) {
      const opt = document.createElement("option");
      opt.value = String(ratio);
      opt.textContent = `1:${ratio}`;
      scaleSel.appendChild(opt);
    }
    scaleSel.value = "2000"; // matches DEFAULT_PX_PER_UNIT (a true 1:2000 opening overview)
    scaleSel.addEventListener("change", () => {
      const ratio = parseFloat(scaleSel.value);
      if (!Number.isFinite(ratio) || ratio <= 0) return;
      // A true print-style ratio: 1 depth unit occupies (1/ratio) unit on screen. The
      // renderer owns the px-per-unit conversion (PX_PER_UNIT_1_1) so it stays single-sourced.
      this.renderer?.setScaleRatio(ratio);
      setStatus(`Vertical scale set to 1:${ratio}`);
    });
    this.scaleSel = scaleSel;
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

    const headerBtn = btn("▤", "Track headers: full → compact → titles only", () => {
      this.headerMode = this.headerMode === "full" ? "compact" : this.headerMode === "compact" ? "collapsed" : "full";
      this.applyHeaderMode();
      headerBtn.classList.toggle("active", this.headerMode !== "full");
      this.onUserEdit?.();
      setStatus(
        this.headerMode === "full"
          ? "Track headers: full (curves + scales)"
          : this.headerMode === "compact"
            ? "Track headers: compact (curve chips, no scales)"
            : "Track headers: titles only",
      );
    });
    btn("▦", "Track borders…", () => this.openBordersDialog());
    sep();

    const topsBtn = btn("🏷", "Edit tops: click to add, drag to move, double-click to rename/delete", () => {
      const on = !this.topsEditor.editing;
      this.topsEditor.setEditMode(on);
      topsBtn.classList.toggle("active", on);
      if (on) {
        // Only one on-plot editor captures the overlay at a time.
        this.highlightsOverlay.setEditMode(false);
        highlightsBtn.classList.remove("active");
      }
      setStatus(on ? "Tops editing ON — click adds, drag moves, double-click edits" : "Tops editing off");
    });
    const highlightsBtn = btn(
      "🖍",
      "Highlight intervals: drag to paint a colored band; double-click a band to recolor / label / convert to zone / delete",
      () => {
        const on = !this.highlightsOverlay.editing;
        this.highlightsOverlay.setEditMode(on);
        highlightsBtn.classList.toggle("active", on);
        if (on) {
          this.topsEditor.setEditMode(false);
          topsBtn.classList.remove("active");
        }
        setStatus(on ? "Highlight editing ON — drag paints a band, double-click edits/converts" : "Highlight editing off");
      },
    );

    return bar;
  }

  private async initRenderer(body: HTMLElement): Promise<void> {
    this.canvas.width = this.canvas.clientWidth || 400;
    this.canvas.height = this.canvas.clientHeight || 400;
    // Keep a LOCAL reference: LogCanvasRenderer.init() attaches its window listeners and
    // starts the rAF loop only AFTER its WebGPU awaits, so if dispose() lands during init
    // it nulls this.renderer before those exist. The disposed-guard below must dispose
    // this captured instance (which finished init) — not the nulled field — to tear them down.
    const renderer = (this.renderer = new LogCanvasRenderer(this.canvas));
    this.renderer.onViewSettled = () => {
      this.refreshDepthAxis();
      this.positionCrosshair(this.lastHoverDepth);
      this.syncScaleReadout();
    };
    // Redraw core points + tops lines after every rendered frame so they track pan/zoom.
    this.renderer.onFrameRendered = () => {
      this.drawCoreOverlay();
      this.highlightsOverlay.draw();
      this.topsEditor.draw();
    };
    this.renderer.onCursorMove = (depth, samples, trackTitle) => {
      // The readout is scoped to ONE track: the clicked/selected one, else the track
      // under the cursor — not every curve in the layout (Jauhar: 15 curves must not
      // eat the screen). With no track resolved, all curves show as a fallback.
      const focusTitle = this.selectedTrack ?? trackTitle;
      const track = focusTitle ? this.layout?.tracks.find((t) => t.title === focusTitle) : undefined;
      let shown = samples;
      if (track) {
        const names = new Set(track.curves.map((c) => c.curve_name));
        shown = samples.filter((s) => names.has(s.curveName));
      }
      renderReadout(this.readoutEl, depth, shown, undefined, this.curveUnits);
      this.highlightTrack(trackTitle);
      // Broadcast so every other open log view draws a synchronized crosshair.
      appState.hoverDepth.set(depth);
    };
    this.attachTrackSelect();

    try {
      await renderer.init();
      this.message("");
    } catch (err) {
      console.error("WebGPU init failed:", err);
      this.message("WebGPU unavailable — viewer disabled");
      this.renderer = null;
    }

    // The panel may have been closed while WebGPU was initializing — dispose() already ran
    // (with empty unsubscribers and no ResizeObserver), so registering them below would
    // leak them forever. Dispose the LOCAL instance (init() just attached its window
    // listeners + rAF loop, and dispose() already nulled this.renderer so a field deref
    // would be a no-op) and stop. dispose() is idempotent, so the earlier call is harmless.
    if (this.disposed) {
      renderer.dispose();
      this.renderer = null;
      return;
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

  /** The track whose horizontal extent contains the given client-X (null when off). */
  private trackTitleAtX(clientX: number): string | null {
    const rect = this.canvas.getBoundingClientRect();
    if (rect.width <= 0) return null;
    const frac = (clientX - rect.left) / rect.width;
    for (const r of this.renderer?.getTrackRanges() ?? []) {
      if (frac >= r.leftFrac && frac < r.rightFrac) return r.title;
    }
    return null;
  }

  /** A plain click (no drag) on the canvas selects/deselects the track under the
   *  cursor — the readout then sticks to that track's curves until deselected. */
  private attachTrackSelect(): void {
    let down: [number, number] | null = null;
    this.canvas.addEventListener("pointerdown", (e) => {
      down = [e.clientX, e.clientY];
    });
    this.canvas.addEventListener("pointerup", (e) => {
      if (!down) return;
      const moved = Math.hypot(e.clientX - down[0], e.clientY - down[1]);
      down = null;
      if (moved > 4) return; // it was a pan, not a click
      const title = this.trackTitleAtX(e.clientX);
      if (!title) return;
      this.selectedTrack = this.selectedTrack === title ? null : title;
      this.applyTrackSelection();
      setStatus(
        this.selectedTrack
          ? `Track "${this.selectedTrack}" selected — readout follows it (click it again to release)`
          : "Track selection cleared — readout follows the cursor",
      );
    });
  }

  private applyTrackSelection(): void {
    for (const el of this.headersEl.querySelectorAll<HTMLElement>(".track-header")) {
      el.classList.toggle("selected", this.selectedTrack !== null && el.dataset.track === this.selectedTrack);
    }
  }

  private applyHeaderMode(): void {
    this.headersEl.classList.toggle("compact", this.headerMode === "compact");
    this.headersEl.classList.toggle("collapsed", this.headerMode === "collapsed");
  }

  /** Small dialog for the vertical separators between tracks (style/width/color). */
  private openBordersDialog(): void {
    const content = document.createElement("div");
    const styleSel = document.createElement("select");
    styleSel.className = "form-control";
    for (const [value, label] of [["solid", "Solid"], ["dashed", "Dashed"], ["none", "None"]]) {
      const opt = document.createElement("option");
      opt.value = value;
      opt.textContent = label;
      styleSel.appendChild(opt);
    }
    styleSel.value = this.borders.style;
    const widthInput = document.createElement("input");
    widthInput.className = "form-control";
    widthInput.type = "number";
    widthInput.min = "1";
    widthInput.max = "4";
    widthInput.step = "1";
    widthInput.value = String(this.borders.width);
    const themeColor = document.createElement("input");
    themeColor.type = "checkbox";
    themeColor.checked = this.borders.color === "";
    const colorInput = document.createElement("input");
    colorInput.type = "color";
    colorInput.className = "form-control";
    colorInput.value = this.borders.color || "#888888";
    colorInput.disabled = themeColor.checked;
    themeColor.addEventListener("change", () => (colorInput.disabled = themeColor.checked));

    content.appendChild(formRow("Style", styleSel));
    content.appendChild(formRow("Width (px)", widthInput));
    content.appendChild(formRow("Theme color", themeColor, "Use the theme's border color (follows light/dark)"));
    content.appendChild(formRow("Custom color", colorInput));

    const applyBtn = document.createElement("button");
    applyBtn.className = "lp-btn";
    applyBtn.textContent = "Apply";
    content.appendChild(applyBtn);
    const close = openModal("Track borders", content, 340);
    applyBtn.addEventListener("click", () => {
      this.borders = {
        style: styleSel.value as BorderStyle["style"],
        width: Math.max(1, Math.min(4, parseInt(widthInput.value, 10) || 1)),
        color: themeColor.checked ? "" : colorInput.value,
      };
      this.onUserEdit?.();
      this.drawCoreOverlay();
      setStatus(`Track borders: ${this.borders.style}`);
      close();
    });
  }

  /** Right-click entries for the workspace context menu: "Edit CURVE…" for every curve
   *  in the track under the pointer (wireline shift, set/blank/interpolate/scale). */
  curveMenuEntries(e: MouseEvent): ContextMenuEntry[] {
    if (!this.well || !this.renderer || !this.layout) return [];
    const rect = this.canvas.getBoundingClientRect();
    if (e.clientX < rect.left || e.clientX > rect.right || e.clientY < rect.top || e.clientY > rect.bottom) return [];
    const title = this.trackTitleAtX(e.clientX);
    const track = title ? this.layout.tracks.find((t) => t.title === title) : undefined;
    if (!track) return [];
    const editable = track.curves.filter((c) => c.fill !== "blocks");
    if (editable.length === 0) return [];
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    const depth = top + ((e.clientY - rect.top) / (rect.height || 1)) * (bottom - top);
    const well = this.well;
    const entries: ContextMenuEntry[] = [{ heading: `Track ${track.title}` }];
    for (const c of editable) {
      entries.push({
        label: `Edit ${c.curve_name}…`,
        onClick: () => openCurveEditDialog(well.well_id, well.well_name, c.curve_name, depth),
      });
    }
    return entries;
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
    // Capture a token before the first await; a newer loadWell (fast switch) or dispose
    // supersedes this one and we bail before writing this.series/coreByName. this.well is
    // set synchronously and NOT rolled back — a newer load already advanced it.
    const gen = ++this.loadGen;
    if (!keepView) this.viewResetPending = true;
    this.well = well;
    this.updateTitle();
    const curveNames = Array.from(new Set(this.layout.tracks.flatMap((t) => t.curves.map((c) => c.curve_name))));
    try {
      const [series, units] = await Promise.all([
        getTrackData(well.well_id, curveNames, this.canvas.clientHeight || 400),
        loadCurveUnits().catch(() => new Map<string, string>()),
      ]);
      if (gen !== this.loadGen || !this.renderer) return; // superseded or disposed
      this.series = series;
      this.curveUnits = units;
      if (this.viewResetPending) {
        this.renderer.resetView();
        this.viewResetPending = false;
      }
      this.refresh();
      setStatus(`Loaded well ${well.well_name}`);
    } catch (err) {
      if (gen !== this.loadGen) return;
      console.error("Failed to load track data:", err);
      setStatus(`Failed to load curve data: ${err}`);
      // The winning load failed: drop the previous well's series so the (already updated)
      // title and the rendered curves can't diverge — show the well as empty instead.
      this.series = [];
      if (this.viewResetPending) {
        this.renderer?.resetView();
        this.viewResetPending = false;
      }
      this.refresh();
    }
    try {
      const core = await getCoreData(well.well_id);
      if (gen !== this.loadGen) return;
      this.coreByName = new Map(core.map((s) => [s.curve_name, s]));
    } catch {
      if (gen !== this.loadGen) return;
      this.coreByName = new Map(); // no backend or no core data — overlay simply stays empty
    }
    try {
      const [casing, perfs] = await Promise.all([
        listAuxData(well.well_id, "COMPLETION").catch(() => [] as AuxRow[]),
        listAuxData(well.well_id, "PERFORATION").catch(() => [] as AuxRow[]),
      ]);
      if (gen !== this.loadGen) return;
      this.wellDiagram = { casing, perfs };
    } catch {
      if (gen !== this.loadGen) return;
      this.wellDiagram = { casing: [], perfs: [] };
    }
    this.drawCoreOverlay();
    if (gen !== this.loadGen || !this.renderer) return;
    await this.topsEditor.setWell(well.well_id);
    await this.highlightsOverlay.setWell(well.well_id);
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
    if (!this.renderer || !this.layout) return;
    this.drawTrackBorders(ctx, w, h);
    this.drawWellDiagram(ctx, w, h);
    if (this.coreByName.size === 0) return;

    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;
    ctx.lineWidth = 1;
    // Diamond outlines follow the theme's text color (65% alpha) so they stay
    // visible on dark themes — near-black was invisible there.
    const outline =
      getComputedStyle(this.root).getPropertyValue("--text").trim() ||
      getComputedStyle(document.documentElement).getPropertyValue("--text").trim() ||
      "#332a1f";
    ctx.strokeStyle = /^#[0-9a-fA-F]{6}$/.test(outline) ? `${outline}a6` : outline;

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

  /** Well-diagram tracks (Track.kind === "well_diagram"): schematic casing/tubing/liner strings
   *  with shoe symbols and perforation ticks, from the well's COMPLETION + PERFORATION aux
   *  datasets (value_num = OD in inches; depth_top..depth_base = the run). */
  private drawWellDiagram(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    if (!this.renderer || !this.layout) return;
    const diagramTracks = new Set(
      this.layout.tracks.filter((t) => (t.kind ?? "curves") === "well_diagram").map((t) => t.title),
    );
    if (diagramTracks.size === 0) return;
    const { casing, perfs } = this.wellDiagram;

    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;
    const yOf = (d: number) => ((d - top) / (bottom - top)) * h;
    const clampY = (y: number) => Math.max(0, Math.min(h, y));
    const text = getComputedStyle(this.root).getPropertyValue("--text").trim() || "#332a1f";
    // Theme bundle for the diagram's structural colors: casing strings in the dim text
    // color, perforations in the theme's warn red — mid-gray/fixed-red literals vanish on
    // dark and client-branded palettes.
    const th = readTheme(this.root);
    const maxOd = Math.max(1, ...casing.map((c) => c.value_num ?? 0));

    for (const range of this.renderer.getTrackRanges()) {
      if (!diagramTracks.has(range.title)) continue;
      const left = range.leftFrac * w;
      const span = (range.rightFrac - range.leftFrac) * w;
      const cx = left + span / 2;
      const maxHalf = span * 0.36;

      // Casing / tubing / liner strings: two vertical lines with a shoe at the base.
      for (const c of casing) {
        const od = c.value_num ?? maxOd;
        const half = Math.max(3, (od / maxOd) * maxHalf);
        const yTop = clampY(yOf(c.depth_top));
        const yBot = clampY(yOf(c.depth_base ?? bottom));
        if (yBot <= 0 || yTop >= h) continue;
        ctx.strokeStyle = th.text;
        ctx.lineWidth = 1.5;
        for (const sx of [cx - half, cx + half]) {
          ctx.beginPath();
          ctx.moveTo(sx, yTop);
          ctx.lineTo(sx, yBot);
          ctx.stroke();
        }
        // Shoe: small filled triangles at the casing base, pointing inward.
        if (c.depth_base != null && yBot > 0 && yBot < h) {
          ctx.fillStyle = text;
          for (const dir of [-1, 1]) {
            const sx = cx + dir * half;
            ctx.beginPath();
            ctx.moveTo(sx, yBot);
            ctx.lineTo(sx - dir * 5, yBot);
            ctx.lineTo(sx, yBot - 6);
            ctx.closePath();
            ctx.fill();
          }
        }
        // OD label at the top of the string.
        const label = c.value_text || (c.value_num != null ? `${c.value_num}"` : c.item);
        if (label && yTop > 8 && yTop < h) {
          ctx.fillStyle = text;
          ctx.font = canvasFont(th, 9, 400);
          ctx.textAlign = "center";
          ctx.textBaseline = "bottom";
          ctx.fillText(label, cx, yTop - 1, Math.max(20, span - 4));
        }
      }

      // Perforations: theme-warn ticks radiating from the well centre over the perf interval.
      ctx.strokeStyle = th.warn;
      ctx.lineWidth = 1.5;
      const tickHalf = Math.min(maxHalf, span * 0.28);
      for (const p of perfs) {
        const y0 = yOf(p.depth_top);
        const y1 = yOf(p.depth_base ?? p.depth_top);
        const lo = clampY(Math.min(y0, y1));
        const hi = clampY(Math.max(y0, y1));
        for (let y = lo; y <= Math.max(lo, hi); y += 5) {
          if (y < 0 || y > h) continue;
          ctx.beginPath();
          ctx.moveTo(cx - tickHalf, y);
          ctx.lineTo(cx - tickHalf * 0.4, y);
          ctx.moveTo(cx + tickHalf * 0.4, y);
          ctx.lineTo(cx + tickHalf, y);
          ctx.stroke();
        }
      }
    }
  }

  /** Vertical separators at the interior track boundaries, in the user's chosen style
   *  (▦ in the toolbar). Empty color follows the theme's --border on every repaint. */
  private drawTrackBorders(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    if (this.borders.style === "none" || !this.renderer) return;
    const color =
      this.borders.color ||
      getComputedStyle(this.root).getPropertyValue("--border").trim() ||
      getComputedStyle(document.documentElement).getPropertyValue("--border").trim() ||
      "#999";
    ctx.save();
    ctx.strokeStyle = color;
    ctx.lineWidth = this.borders.width;
    ctx.setLineDash(this.borders.style === "dashed" ? [6, 4] : []);
    const ranges = this.renderer.getTrackRanges();
    for (let i = 0; i < ranges.length - 1; i++) {
      const x = Math.round(ranges[i].rightFrac * w) + (this.borders.width % 2 === 1 ? 0.5 : 0);
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();
    }
    ctx.restore();
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
    // A selected track that no longer exists (renamed/removed in properties) unsticks.
    if (this.selectedTrack && !this.layout.tracks.some((t) => t.title === this.selectedTrack)) {
      this.selectedTrack = null;
    }
    renderTrackHeaders(this.headersEl, this.layout, this.trackWeights, this.hiddenCurves, {
      onLayoutMutated: () => {
        this.onUserEdit?.();
        this.refresh();
      },
      onCurveToggle: (curveName, hidden) => {
        this.onUserEdit?.();
        this.renderer?.setCurveHidden(curveName, hidden);
      },
      onCurveMoved: (before, label) => {
        // Snapshot AFTER the drop mutation so redo replays the exact result.
        const after = structuredClone(this.layout!);
        pushUndo({
          label,
          undo: () => this.applyLayoutEdit(structuredClone(before)),
          redo: () => this.applyLayoutEdit(structuredClone(after)),
        });
      },
    });
    this.applyTrackSelection();
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

  /** Re-point the "1:N" selector at the TRUE current scale after any zoom/pan/reset. When the
   *  live ratio lands on a preset it selects it; otherwise it shows a transient "1:N ⟳" entry
   *  so the box never lies about the scale (the old code left it stuck on the last preset). */
  private syncScaleReadout(): void {
    const sel = this.scaleSel;
    if (!sel || !this.renderer) return;
    const ratio = Math.round(this.renderer.getScaleRatio());
    if (!Number.isFinite(ratio) || ratio <= 0) return;
    const dyn = Array.from(sel.options).find((o) => o.dataset.dynamic === "1");
    // A preset counts as "current" only within 0.5 of the rounded live ratio.
    const preset = Array.from(sel.options).find(
      (o) => o.dataset.dynamic !== "1" && Math.abs(parseFloat(o.value) - ratio) < 0.5,
    );
    if (preset) {
      dyn?.remove();
      sel.value = preset.value;
    } else {
      if (dyn) {
        dyn.value = String(ratio);
        dyn.textContent = `1:${ratio} ⟳`;
      } else {
        const o = document.createElement("option");
        o.dataset.dynamic = "1";
        o.value = String(ratio);
        o.textContent = `1:${ratio} ⟳`;
        sel.insertBefore(o, sel.firstChild);
      }
      sel.value = String(ratio);
    }
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
    this.disposed = true;
    for (const unsub of this.unsubscribers) unsub();
    this.resizeObserver?.disconnect();
    this.topsEditor.dispose();
    this.highlightsOverlay.dispose();
    this.renderer?.dispose();
    this.renderer = null;
  }
}
