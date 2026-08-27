import { LogCanvasRenderer } from "../LogCanvasRenderer";
import {
  getArrayLog,
  getCoreData,
  getTrackData,
  getWellImage,
  listArrayCurves,
  listAuxData,
  listCurveCatalog,
  listGenericCurveInventory,
  listWellImages,
  type ArrayLog,
  type ArrayStyle,
  type AuxRow,
  type ImageInfo,
  type ImageStyle,
  type GenericCurveInventoryEntry,
  type Layout,
  type PointStyle,
  type TrackCurveSeries,
  type WellSummary,
} from "../ipc";
import { band, binByDepth, boxStats, canonicalHistogram, defaultBinHeight, evenIndices, type WhiskerRule } from "../distribution";
import { appState, setStatus } from "../state";
import { setDisplayDepthUnit } from "../depthUnitPref";
import { unitLabel } from "../units";
import { pushUndo } from "../undo";
import type { ContextMenuEntry } from "./contextMenu";
import { openCurveEditDialog } from "./curveEditDialog";
import { openLayoutPropsDialog, type PointSuggestion } from "./layoutPropsDialog";
import type { CurveSuggestion } from "./layoutPropsDialog";
import { trackCurveKey, type TrackCurveRequest } from "../trackCurveRequest";
import { formRow, openModal } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";
import { coreOverlayItem, loadCurveUnits } from "./plotCommon";
import { HighlightsOverlay } from "./highlightsOverlay";
import { TopsEditor } from "./topsEditor";
import { renderDepthAxis, renderReadout, renderReportHeader, renderTrackHeaders } from "./viewerChrome";
import {
  applyPlotChannelPolicy,
  type PlotChannelPolicyReport,
  type PlotDisplayRange,
} from "./plotTypes";
import { ViewportRefetchCoordinator } from "./viewportRefetch";

type HeaderMode = "full" | "compact" | "collapsed";
type BorderStyle = { style: "solid" | "dashed" | "none"; width: number; color: string };

/** SB-PLT-013's screen/composite waveform contract: derive the clamped display values,
 * preserve the source samples, and retain separate exclusion/clamp counts. */
export function applyArrayWaveformPolicy(
  source: Float32Array,
  display: PlotDisplayRange,
  logAxis: boolean,
): PlotChannelPolicyReport {
  return applyPlotChannelPolicy(source, "array_waveform", display, logAxis);
}

/** How many decoded plates one viewer keeps. Small on purpose: this is a VIEW cache, and a
 *  core photo run of 300 frames must not end up mirrored in memory. */
const IMAGE_CACHE_MAX = 24;

/** The box one picture occupies in a track, in canvas pixels.
 *
 *  Mirrors `image_box` in `src-tauri/src/composite.rs` — the screen and the print must place a
 *  plate identically, or a composite would not be what the user checked on screen. `cover`
 *  means the box is the requested frame and the picture overfills it; otherwise the box has
 *  already been fitted to the picture's own aspect ratio, so nothing is ever distorted.
 *
 *  EXPORTED so that agreement can actually be checked: the Rust side's numbers are pinned by
 *  its own unit tests, and this can be driven with the same inputs against the dev server
 *  (see the browser-verification note in CLAUDE.md). Nothing else imports it. */
export function imageBox(
  style: ImageStyle,
  info: ImageInfo,
  left: number,
  span: number,
  yOf: (d: number) => number,
): { x: number; y: number; w: number; h: number; cover: boolean } {
  const boxW = span * Math.min(1, Math.max(0.05, style.size ?? 0.9));
  const align = style.align ?? "center";
  const x = align === "left" ? left : align === "right" ? left + span - boxW : left + (span - boxW) / 2;
  const aspect = info.height / Math.max(1, info.width);
  const interval = info.depth_base != null && info.depth_base > info.depth_top ? info.depth_base : null;

  if ((style.mode ?? "anchor") === "depth" && interval != null) {
    const y0 = yOf(info.depth_top);
    const boxH = Math.max(2, yOf(interval) - y0);
    if ((style.fit ?? "contain") === "cover") return { x, y: y0, w: boxW, h: boxH, cover: true };
    // "stretch": the box IS the picture. Only honest for a depth strip, whose vertical axis is
    // depth and whose width is the track — neither of them the picture's own. See ImageStyle.fit.
    if (style.fit === "stretch") return { x, y: y0, w: boxW, h: boxH, cover: false };
    let w = boxW;
    let hh = boxW * aspect;
    if (hh > boxH) {
      hh = boxH;
      w = boxH / Math.max(1e-6, aspect);
    }
    return { x: x + (boxW - w) / 2, y: y0 + (boxH - hh) / 2, w, h: hh, cover: false };
  }
  const h = boxW * aspect;
  const yc = yOf(info.depth_base == null ? info.depth_top : (info.depth_top + info.depth_base) / 2);
  return { x, y: yc - h / 2, w: boxW, h, cover: false };
}

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
  /** Every point dataset for the loaded well (XRD, CEC, oil show, core extras …), one
   *  active delivery of each — the source for `point_data` tracks. */
  private auxRows: AuxRow[] = [];
  /** Array logs referenced by `array_log` tracks, keyed by upper-cased curve name. Loaded
   *  lazily — only curves some track actually asks for, since one of these is a whole
   *  realization matrix and a layout with none must cost nothing. */
  private arrayLogs = new Map<string, ArrayLog>();
  /** Array curves available on the loaded well, for the properties dialog's picker. */
  private arrayCatalog: { set_name: string; curve_name: string }[] = [];
  /** Picture METADATA for the loaded well — depth registration and pixel size, never the
   *  pixels. A well can carry hundreds of core photographs, so the bytes are fetched one at
   *  a time as plates scroll into view (see `imageBitmaps`). */
  private imageMeta: ImageInfo[] = [];
  /** Decoded plates, keyed by image_id, filled on demand and capped — the cache is a view
   *  cache, not a copy of the delivery. */
  private imageBitmaps = new Map<string, ImageBitmap>();
  /** Ids currently being fetched, so a redraw storm cannot ask for the same plate twice. */
  private imagePending = new Set<string>();
  /** Image datasets on the loaded well, for the properties dialog's picker. */
  private imageCatalog: { dataset: string; count: number }[] = [];
  /** Tops overlay + Petrel-style interactive editor (🏷 in the toolbar toggles editing). */
  private topsEditor!: TopsEditor;
  /** Colored highlight bands + interactive editor (🖍 in the toolbar toggles editing). */
  private highlightsOverlay!: HighlightsOverlay;

  private renderer: LogCanvasRenderer | null = null;
  private layout: Layout | null = null;
  /** Curve name → unit, refreshed with each load (also on dataVersion). Feeds the cursor
   *  readout so RT shows "ohm.m", PHI "v/v", etc. Empty until the first successful load. */
  private curveUnits = new Map<string, string>();
  private curveLabels = new Map<string, string>();
  private curveInventory: GenericCurveInventoryEntry[] = [];
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
  /** Disposable loaded-interval/density identity plus the generation guard for viewport loads. */
  private readonly viewportRefetch = new ViewportRefetchCoordinator<TrackCurveSeries[]>();
  private viewportTimer: number | undefined;
  private depthRangeInitialized = false;
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

    // Display-unit toggle. Purely a view setting — it converts what is SHOWN and never
    // touches stored depths, which stay in the project's declared unit. The 1:N scale
    // above is deliberately unaffected: it is a physical ratio of rock to paper.
    const unitBtn = btn("", "", () => {
      setDisplayDepthUnit(appState.displayDepthUnit.get() === "M" ? "FT" : "M");
    });
    unitBtn.classList.add("lv-unit");
    const paintUnit = () => {
      const display = appState.displayDepthUnit.get();
      const stored = appState.projectDepthUnit.get();
      unitBtn.textContent = unitLabel(display);
      unitBtn.title =
        display === stored
          ? `Depths shown in ${unitLabel(display)} (the unit they are stored in). Click to show ${unitLabel(display === "M" ? "FT" : "M")}.`
          : `Depths shown in ${unitLabel(display)}, converted from the stored ${unitLabel(stored)} — stored data is unchanged. Click to switch back.`;
      // Flag the converted state so it is never mistaken for the stored numbers.
      unitBtn.classList.toggle("active", display !== stored);
    };
    paintUnit();
    this.unsubscribers.push(
      appState.displayDepthUnit.subscribe(() => {
        paintUnit();
        this.refreshDepthAxis();
        this.renderer?.repaint();
      }),
      appState.projectDepthUnit.subscribe(paintUnit),
    );
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
      this.scheduleViewportReload();
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
        const names = new Set(track.curves.map(trackCurveKey));
        shown = samples.filter((s) => names.has(s.curveName));
      }
      renderReadout(this.readoutEl, depth, shown, undefined, this.curveUnits, this.curveLabels);
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
      this.showGpuRefusal(err instanceof Error ? err.message : String(err));
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
      this.scheduleViewportReload();
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
      // Linked brushing: paint the crossplot-brushed sample depths as ticks across this well's tracks.
      appState.brushedDepths.subscribe((sel) => {
        const depths = sel && this.well && sel.wellId === this.well.well_id ? [...sel.depths] : [];
        this.highlightsOverlay.setBrush(depths);
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
    // The curve editor writes current/computed values by mnemonic. An explicit imported set
    // has a different identity, so its value-edit route remains the Curve Catalog.
    const editable = track.curves.filter((c) => c.fill !== "blocks" && !c.set_name);
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

  /** The named refusal for a machine whose graphics stack has no WebGPU (needWell.ts's rule:
   *  refuse BY NAME with the fix stated, where the user is looking — a near-blank track area
   *  reads as the whole app being broken). Only log views draw through the GPU; every other
   *  surface is 2D canvas and keeps working, and the card says so. The status line gets the
   *  message too — it belongs in the record of what was attempted — but cannot be the only
   *  place it appears. */
  private showGpuRefusal(detail: string): void {
    setStatus("Log view disabled — this machine's graphics stack has no WebGPU");
    this.messageEl.textContent = "";
    const card = document.createElement("div");
    card.className = "logview-gpu-note";
    const title = document.createElement("div");
    title.className = "logview-gpu-title";
    title.textContent = "Log views need WebGPU";
    const why = document.createElement("div");
    why.textContent =
      "This machine's graphics driver or WebView2 runtime did not provide WebGPU, which log tracks " +
      "are drawn with. The rest of the application — plots, dialogs, imports, exports — still works; " +
      "only log views are affected.";
    const fix = document.createElement("div");
    fix.textContent =
      "Update the graphics (GPU) driver and the Microsoft WebView2 Runtime, then reopen this log view.";
    const det = document.createElement("div");
    det.className = "logview-gpu-detail";
    det.textContent = detail;
    card.append(title, why, fix, det);
    this.messageEl.appendChild(card);
    this.messageEl.hidden = false;
  }

  private adoptLayout(layout: Layout): void {
    this.layout = structuredClone(layout);
    this.syncCurveLabels();
    this.trackWeights.clear();
    for (const t of this.layout.tracks) this.trackWeights.set(t.title, t.width_weight * 150);
    this.widthScalePct = 100;
    this.hiddenCurves.clear();
  }

  private syncCurveLabels(): void {
    const labels = new Map<string, string>();
    for (const style of this.layout?.tracks.flatMap((track) => track.curves) ?? []) {
      const mnemonic = style.curve_name.trim().toUpperCase();
      labels.set(trackCurveKey(style), style.set_name ? `${mnemonic} [${style.set_name}]` : mnemonic);
    }
    this.curveLabels = labels;
  }

  private trackCurveRequests(): TrackCurveRequest[] {
    const requests = new Map<string, TrackCurveRequest>();
    for (const style of this.layout?.tracks.flatMap((track) => track.curves) ?? []) {
      const request: TrackCurveRequest = {
        curve_name: style.curve_name,
        set_name: style.set_name,
        class_curve: style.fill === "blocks",
      };
      const key = trackCurveKey(request);
      // The key is the curve's IDENTITY, so one curve drawn both as blocks and as a line in the
      // same layout arrives here twice under one key. Class wins: it means "send this one
      // undecimated", which is strictly more data, so the line rendering is if anything more
      // faithful - never less. The reverse would silently shred the block track.
      const seen = requests.get(key);
      if (seen) seen.class_curve = seen.class_curve || request.class_curve;
      else requests.set(key, request);
    }
    return [...requests.values()];
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
    this.viewportRefetch.reset();
    if (this.viewportTimer !== undefined) window.clearTimeout(this.viewportTimer);
    this.viewportTimer = undefined;
    this.depthRangeInitialized = false;
    if (!keepView) this.viewResetPending = true;
    this.well = well;
    this.updateTitle();
    const curveRequests = this.trackCurveRequests();
    try {
      const [series, units, inventory] = await Promise.all([
        getTrackData(well.well_id, curveRequests, this.canvas.clientHeight || 400),
        loadCurveUnits().catch(() => new Map<string, string>()),
        listGenericCurveInventory(well.well_id).catch(() => [] as GenericCurveInventoryEntry[]),
      ]);
      if (gen !== this.loadGen || !this.renderer) return; // superseded or disposed
      this.series = series;
      this.curveInventory = inventory;
      for (const request of curveRequests) {
        const setName = request.set_name?.trim();
        if (!setName) continue;
        const candidates = inventory
          .filter(
            (entry) =>
              entry.set_name === setName &&
              entry.mnemonic.trim().toUpperCase() === request.curve_name.trim().toUpperCase(),
          )
          .sort((a, b) => {
            if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
            if (a.modified_seq == null && b.modified_seq != null) return 1;
            if (a.modified_seq != null && b.modified_seq == null) return -1;
            if (a.modified_seq !== b.modified_seq) return (b.modified_seq ?? 0) - (a.modified_seq ?? 0);
            if (a.run_no == null && b.run_no != null) return 1;
            if (a.run_no != null && b.run_no == null) return -1;
            if (a.run_no !== b.run_no) return (b.run_no ?? 0) - (a.run_no ?? 0);
            return a.curve_id.localeCompare(b.curve_id);
          });
        if (candidates[0]?.unit) units.set(trackCurveKey(request), candidates[0].unit);
      }
      this.curveUnits = units;
      this.refresh(false);
      this.depthRangeInitialized = true;
      const [loadedLow, loadedHigh] = this.renderer.getDataDepthRange();
      if (Number.isFinite(loadedLow) && Number.isFinite(loadedHigh) && loadedHigh > loadedLow) {
        this.viewportRefetch.seedLoaded({
          sourceKey: this.viewportSourceKey(well.well_id, curveRequests),
          low: loadedLow,
          high: loadedHigh,
          targetPixelHeight: this.canvas.clientHeight || 400,
        });
      }
      if (this.viewResetPending) {
        this.renderer.resetView();
        this.viewResetPending = false;
      }
      this.scheduleViewportReload();
      setStatus(`Loaded well ${well.well_name}`);
    } catch (err) {
      if (gen !== this.loadGen) return;
      console.error("Failed to load track data:", err);
      setStatus(`Failed to load curve data: ${err}`);
      // The winning load failed: drop the previous well's series so the (already updated)
      // title and the rendered curves can't diverge — show the well as empty instead.
      this.series = [];
      this.curveInventory = [];
      this.refresh(false);
      this.depthRangeInitialized = true;
      if (this.viewResetPending) {
        this.renderer?.resetView();
        this.viewResetPending = false;
      }
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
    try {
      // Every point dataset in one read — the backend's active-set filter is correlated on
      // dataset, so this returns exactly one delivery of each (XRD, CEC, oil show, core
      // extras) rather than unioning two.
      const aux = await listAuxData(well.well_id, null).catch(() => [] as AuxRow[]);
      if (gen !== this.loadGen) return;
      this.auxRows = aux;
    } catch {
      if (gen !== this.loadGen) return;
      this.auxRows = [];
    }
    await this.loadArrayLogs(well.well_id, gen);
    await this.loadImageMeta(well.well_id, gen);
    this.drawCoreOverlay();
    if (gen !== this.loadGen || !this.renderer) return;
    await this.topsEditor.setWell(well.well_id);
    await this.highlightsOverlay.setWell(well.well_id);
    // Re-apply the shared brush for the newly loaded well (the subscription only fires on brush
    // changes, so a well switch would otherwise leave the previous well's ticks painted). Re-check
    // the load token first: a superseded fast-switch must not wipe the winning load's brush.
    if (gen !== this.loadGen || !this.renderer) return;
    const brush = appState.brushedDepths.get();
    this.highlightsOverlay.setBrush(brush && brush.wellId === well.well_id ? [...brush.depths] : []);
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
    this.drawArrayTracks(ctx, w, h);
    this.drawPointTracks(ctx, w, h);
    this.drawImageTracks(ctx, w, h);
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
        if (this.hiddenCurves.has(trackCurveKey(curve))) continue;
        const coreName = coreOverlayItem(curve.curve_name);
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

  /** Gathers one point series' samples for the loaded well. Core reads the ACTIVE core set's
   *  plug property; aux reads one item of one point dataset. Text samples come back with a
   *  NaN value and their string, so a "text" display can draw them and a numeric display
   *  simply skips them rather than coercing a lithology description to zero. */
  private pointSamples(style: PointStyle): { depth: number[]; value: number[]; text: (string | null)[] } {
    const out = { depth: [] as number[], value: [] as number[], text: [] as (string | null)[] };
    const item = style.item.trim().toUpperCase();
    if (style.source === "core") {
      const s = this.coreByName.get(item);
      if (!s) return out;
      for (let i = 0; i < s.depth.length; i++) {
        out.depth.push(s.depth[i]);
        out.value.push(s.value[i]);
        out.text.push(null);
      }
      return out;
    }
    const dataset = style.dataset?.trim().toUpperCase();
    for (const r of this.auxRows) {
      if (dataset && r.dataset.toUpperCase() !== dataset) continue;
      if (r.item.toUpperCase() !== item) continue;
      // depth_base marks an interval sample (a described core run); anchor it at its middle
      // so the glyph sits where the measurement actually applies.
      const d = r.depth_base != null ? (r.depth_top + r.depth_base) / 2 : r.depth_top;
      out.depth.push(d);
      out.value.push(r.value_num ?? NaN);
      out.text.push(r.value_text ?? null);
    }
    return out;
  }

  /** Loads the array logs this layout actually references, plus the well's array catalog for
   *  the properties dialog. Only referenced curves are fetched: one array log is a whole
   *  realization matrix, so a layout with no array track must not pay for any of them. */
  private async loadArrayLogs(wellId: string, gen: number): Promise<void> {
    this.arrayLogs.clear();
    this.arrayCatalog = [];
    try {
      const cat = await listArrayCurves(wellId).catch(() => []);
      if (gen !== this.loadGen) return;
      this.arrayCatalog = cat.map((c) => ({ set_name: c.set_name, curve_name: c.curve_name }));
    } catch {
      /* no backend, or none stored — array tracks simply draw nothing */
    }
    const wanted = new Map<string, string | null>();
    for (const t of this.layout?.tracks ?? []) {
      if ((t.kind ?? "curves") !== "array_log") continue;
      for (const a of t.arrays ?? []) {
        const key = a.curve_name.trim().toUpperCase();
        if (key && !wanted.has(key)) wanted.set(key, a.set_name ?? null);
      }
    }
    for (const [name, set] of wanted) {
      try {
        const log = await getArrayLog(wellId, set, name);
        if (gen !== this.loadGen) return;
        if (log.depth.length > 0) this.arrayLogs.set(name, log);
      } catch {
        /* a missing array log leaves its track empty rather than failing the whole view */
      }
    }
  }

  /** Loads picture METADATA for the loaded well — every dataset, so the properties dialog can
   *  offer a picker, but never the pixels. A well can carry hundreds of core photographs;
   *  each one's bytes arrive only when a plate is actually on screen. */
  private async loadImageMeta(wellId: string, gen: number): Promise<void> {
    this.imageMeta = [];
    this.imageCatalog = [];
    for (const b of this.imageBitmaps.values()) b.close();
    this.imageBitmaps.clear();
    this.imagePending.clear();
    try {
      const meta = await listWellImages(wellId, null).catch(() => [] as ImageInfo[]);
      if (gen !== this.loadGen) return;
      this.imageMeta = meta;
      const counts = new Map<string, number>();
      for (const m of meta) counts.set(m.dataset, (counts.get(m.dataset) ?? 0) + 1);
      this.imageCatalog = [...counts].map(([dataset, count]) => ({ dataset, count }));
    } catch {
      /* no backend, or no pictures — image tracks simply draw nothing */
    }
  }

  /** Fetches and decodes one plate, then repaints. Guarded by `imagePending` so scrolling
   *  cannot queue the same picture a hundred times, and capped so a long core photo run is
   *  a view cache rather than a second copy of the delivery in memory. */
  private requestImageBitmap(info: ImageInfo): void {
    if (this.imageBitmaps.has(info.image_id) || this.imagePending.has(info.image_id)) return;
    this.imagePending.add(info.image_id);
    const gen = this.loadGen;
    void (async () => {
      try {
        const buf = await getWellImage(info.image_id);
        if (gen !== this.loadGen) return;
        const bmp = await createImageBitmap(new Blob([buf], { type: info.mime }));
        if (gen !== this.loadGen) {
          bmp.close();
          return;
        }
        if (this.imageBitmaps.size >= IMAGE_CACHE_MAX) {
          // Oldest insertion first — Map preserves it, and a plate scrolled past is the
          // least likely to be needed next.
          const oldest = this.imageBitmaps.keys().next();
          if (!oldest.done) {
            this.imageBitmaps.get(oldest.value)?.close();
            this.imageBitmaps.delete(oldest.value);
          }
        }
        this.imageBitmaps.set(info.image_id, bmp);
        this.drawCoreOverlay();
      } catch {
        /* a plate that will not decode draws as its labelled frame, never as a blank gap */
      } finally {
        this.imagePending.delete(info.image_id);
      }
    })();
  }

  /** Image tracks: depth-registered pictures — thin sections, core photographs, SEM plates.
   *
   *  Mirrors `draw_image_series` in `src-tauri/src/composite.rs`; the two must agree, and both
   *  take their geometry from the same two rules. **anchor** centres a fixed-size plate on its
   *  sample depth, because a thin section is cut from one plug and has no thickness; **depth**
   *  stretches the picture over its own depth_top..depth_base, which a core photograph of a
   *  measured run genuinely occupies. Aspect ratio is never distorted — a squashed thin
   *  section misstates grain shape, which is the one thing the plate is there to show. */
  private drawImageTracks(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    if (!this.renderer || !this.layout || this.imageMeta.length === 0) return;
    const imageTracks = this.layout.tracks.filter((t) => (t.kind ?? "curves") === "image");
    if (imageTracks.length === 0) return;
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;
    const yOf = (d: number): number => ((d - top) / (bottom - top)) * h;
    const theme = readTheme(this.root);

    for (const range of this.renderer.getTrackRanges()) {
      const track = imageTracks.find((t) => t.title === range.title);
      if (!track?.images?.length) continue;
      const left = range.leftFrac * w;
      const span = (range.rightFrac - range.leftFrac) * w;

      for (const style of track.images) {
        const ds = style.dataset.trim().toUpperCase();
        const entries = this.imageMeta.filter((m) => m.dataset.toUpperCase() === ds);
        if (entries.length === 0) continue;
        const label = style.label ?? true;
        const border = style.border ?? true;
        // Boxes are computed over EVERY plate in depth order, not just the visible ones, so
        // which plate loses an overlap does not change as you scroll.
        let lastBottom = -Infinity;
        ctx.save();
        ctx.beginPath();
        ctx.rect(left, 0, span, h);
        ctx.clip();
        for (const info of entries) {
          const sampleDepth = info.depth_base == null ? info.depth_top : (info.depth_top + info.depth_base) / 2;
          const box = imageBox(style, info, left, span, yOf);
          // Touching is not overlap: build_core_strips delivers adjacent barrels whose depth
          // boxes share an edge EXACTLY, at every scale, so a guard firing at equality would
          // condemn the middle barrel of every core delivery — and zooming could never reveal
          // it. Only a box that genuinely starts above the previous one's base (beyond half a
          // pixel of float slack) collides. Mirrors draw_image_series in composite.rs.
          if (box.y < lastBottom - 0.5) {
            // Skipped, never nudged: a thin section moved to make room is a thin section
            // attributed to the wrong sand. A tick keeps its true depth visible.
            if (sampleDepth >= top && sampleDepth <= bottom) {
              ctx.strokeStyle = theme.axis;
              ctx.lineWidth = 1;
              ctx.beginPath();
              ctx.moveTo(left, yOf(sampleDepth));
              ctx.lineTo(left + span * 0.15, yOf(sampleDepth));
              ctx.stroke();
            }
            continue;
          }
          lastBottom = box.y + box.h;
          if (box.y + box.h < 0 || box.y > h) continue; // off screen: nothing to draw

          const bmp = this.imageBitmaps.get(info.image_id);
          if (!bmp) {
            this.requestImageBitmap(info);
            ctx.strokeStyle = theme.axis;
            ctx.lineWidth = 1;
            ctx.setLineDash([3, 3]);
            ctx.strokeRect(box.x, box.y, box.w, box.h);
            ctx.setLineDash([]);
          } else if (box.cover) {
            // Fill the box and crop the overhang, centred — the same crop the SVG export's
            // `slice` and the PDF export's clip produce.
            const aspect = bmp.height / Math.max(1, bmp.width);
            let dw = box.w;
            let dh = box.w * aspect;
            if (dh < box.h) {
              dh = box.h;
              dw = box.h / Math.max(1e-6, aspect);
            }
            ctx.save();
            ctx.beginPath();
            ctx.rect(box.x, box.y, box.w, box.h);
            ctx.clip();
            ctx.drawImage(bmp, box.x + (box.w - dw) / 2, box.y + (box.h - dh) / 2, dw, dh);
            ctx.restore();
          } else {
            ctx.drawImage(bmp, box.x, box.y, box.w, box.h);
          }
          if (border) {
            ctx.strokeStyle = theme.axis;
            ctx.lineWidth = 1;
            ctx.strokeRect(box.x, box.y, box.w, box.h);
          }
          // Depth leader: the plate sits somewhere in the track, its depth is on the edge.
          ctx.strokeStyle = theme.axis;
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(left, yOf(sampleDepth));
          ctx.lineTo(box.x, yOf(sampleDepth));
          ctx.stroke();
          if (label && box.y > 12) {
            ctx.fillStyle = theme.text;
            ctx.font = canvasFont(theme, 10);
            ctx.textBaseline = "alphabetic";
            ctx.fillText(info.name, box.x, box.y - 3, box.w);
          }
        }
        ctx.restore();
      }
    }
  }

  /** Array-log tracks: a band, a spaghetti overlay or a density heat map per series.
   *
   *  Mirrors `draw_array_series` in `src-tauri/src/composite.rs` — the two must agree, and both
   *  take their statistics from the shared distribution module so a band here and a box plot on
   *  a point track answer the same question the same way. */
  private drawArrayTracks(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    if (!this.renderer || !this.layout || this.arrayLogs.size === 0) return;
    const arrayTracks = this.layout.tracks.filter((t) => (t.kind ?? "curves") === "array_log");
    if (arrayTracks.length === 0) return;
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;
    const yOf = (d: number): number => ((d - top) / (bottom - top)) * h;
    const theme = readTheme(this.root);

    for (const range of this.renderer.getTrackRanges()) {
      const track = arrayTracks.find((t) => t.title === range.title);
      if (!track?.arrays?.length) continue;
      const left = range.leftFrac * w;
      const span = (range.rightFrac - range.leftFrac) * w;
      const log = track.scale_type === "log";

      for (const [styleIndex, style] of track.arrays.entries()) {
        const series = this.arrayLogs.get(style.curve_name.trim().toUpperCase());
        if (!series) continue;
        if (!Number.isFinite(style.min) || !Number.isFinite(style.max) || (log && (style.min <= 0 || style.max <= 0))) continue;
        const lo = log ? Math.log10(style.min) : style.min;
        const hi = log ? Math.log10(style.max) : style.max;
        if (hi === lo) continue;
        // CLAMPED at the track edge, unlike a point sample. The rule follows what the data is:
        // a discrete plug drawn at a value it never had is a lie, while a continuous reading
        // running past the scale is the ordinary log-display convention.
        const xOf = (v: number): number | null => {
          if (!Number.isFinite(v) || (log && v <= 0)) return null;
          const tv = log ? Math.log10(v) : v;
          return left + Math.min(1, Math.max(0, (tv - lo) / (hi - lo))) * span;
        };
        // Only the depths on screen — a 2000-sample matrix zoomed to 10 m must cost 10 m of work.
        const rows: number[] = [];
        for (let i = 0; i < series.depth.length; i++) {
          if (series.depth[i] >= top && series.depth[i] <= bottom) rows.push(i);
        }
        if (rows.length === 0) continue;

        ctx.save();
        ctx.beginPath();
        ctx.rect(left, 0, span, h);
        ctx.clip();
        ctx.fillStyle = style.color;
        ctx.strokeStyle = style.color;
        const display = style.display ?? "band";
        if (display === "spaghetti") {
          const policy = this.drawSpaghetti(
            ctx,
            style,
            series,
            rows,
            yOf,
            xOf,
            { min: style.min, max: style.max },
            log,
          );
          ctx.globalAlpha = 1;
          ctx.fillStyle = theme.text;
          ctx.font = canvasFont(theme, 9);
          ctx.textAlign = "left";
          ctx.fillText(
            `waveform clamped=${policy.clamped} · non-finite excluded=${policy.nonFiniteExcluded} · log-domain excluded=${policy.logDomainExcluded}`,
            left + 3,
            12 + styleIndex * 12,
            Math.max(0, span - 6),
          );
        } else if (display === "heatmap") this.drawArrayHeatmap(ctx, style, series, rows, yOf, left, span);
        else this.drawArrayBand(ctx, style, series, rows, yOf, xOf);
        ctx.restore();
      }
    }
  }

  /** One realization's path down the well, for an evenly-spread subset of them. */
  private drawSpaghetti(
    ctx: CanvasRenderingContext2D,
    style: ArrayStyle,
    s: ArrayLog,
    rows: number[],
    yOf: (d: number) => number,
    xOf: (v: number) => number | null,
    display: PlotDisplayRange,
    logAxis: boolean,
  ): PlotChannelPolicyReport {
    ctx.lineWidth = 0.5;
    ctx.globalAlpha = Math.min(1, Math.max(0.08, 8 / Math.max(1, style.traces ?? 40)));
    const traces = evenIndices(s.width, style.traces ?? 40);
    const source = new Float32Array(traces.length * rows.length);
    let sourceIndex = 0;
    for (const trace of traces) {
      for (const row of rows) {
        source[sourceIndex++] = s.values[row * s.width + trace] ?? Number.NaN;
      }
    }
    const policy = applyArrayWaveformPolicy(source, display, logAxis);
    let displayIndex = 0;
    for (let traceIndex = 0; traceIndex < traces.length; traceIndex++) {
      ctx.beginPath();
      let drawing = false;
      for (const i of rows) {
        const included = policy.included[displayIndex] === 1;
        const x = included ? xOf(policy.values[displayIndex]) : null;
        displayIndex++;
        // A realization that produced nothing here BREAKS its own trace rather than being
        // bridged — joining across the gap would draw a path this realization never took.
        if (x === null) {
          drawing = false;
          continue;
        }
        const y = yOf(s.depth[i]);
        if (drawing) ctx.lineTo(x, y);
        else {
          ctx.moveTo(x, y);
          drawing = true;
        }
      }
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
    return policy;
  }

  /** P-low to P-high shaded, with the median line through it. */
  private drawArrayBand(
    ctx: CanvasRenderingContext2D,
    style: ArrayStyle,
    s: ArrayLog,
    rows: number[],
    yOf: (d: number) => number,
    xOf: (v: number) => number | null,
  ): void {
    const loP = style.band_lo ?? 10;
    const hiP = style.band_hi ?? 90;
    // Runs of consecutive summarisable depths: a depth where nothing converged is a GAP, so the
    // shading stops rather than spanning an interval the study gave no answer for.
    let run: { y: number; xl: number; xm: number; xh: number }[] = [];
    const flush = (): void => {
      if (run.length > 1) {
        ctx.globalAlpha = style.fill_opacity ?? 0.3;
        ctx.beginPath();
        ctx.moveTo(run[0].xh, run[0].y);
        for (let i = 1; i < run.length; i++) ctx.lineTo(run[i].xh, run[i].y);
        for (let i = run.length - 1; i >= 0; i--) ctx.lineTo(run[i].xl, run[i].y);
        ctx.closePath();
        ctx.fill();
        ctx.globalAlpha = 1;
        if (style.show_median !== false) {
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.moveTo(run[0].xm, run[0].y);
          for (let i = 1; i < run.length; i++) ctx.lineTo(run[i].xm, run[i].y);
          ctx.stroke();
        }
      }
      run = [];
    };
    for (const i of rows) {
      const st = band(s.values.subarray(i * s.width, (i + 1) * s.width), loP, hiP);
      const xl = st === null ? null : xOf(st.lo);
      const xm = st === null ? null : xOf(st.med);
      const xh = st === null ? null : xOf(st.hi);
      if (xl === null || xm === null || xh === null) flush();
      else run.push({ y: yOf(s.depth[i]), xl, xm, xh });
    }
    flush();
  }

  /** Per-depth value histogram, drawn as opacity of the series colour. */
  private drawArrayHeatmap(
    ctx: CanvasRenderingContext2D,
    style: ArrayStyle,
    s: ArrayLog,
    rows: number[],
    yOf: (d: number) => number,
    left: number,
    span: number,
  ): void {
    const bins = Math.max(1, style.hist_bins ?? 32);
    const bw = span / bins;
    for (let k = 0; k < rows.length; k++) {
      const i = rows[k];
      // `histogram` DROPS out-of-range values rather than clamping: a heat-map cell is a count
      // AT a value, so a clamped sample would invent density the data never had.
      const counts = canonicalHistogram(
        s.values.subarray(i * s.width, (i + 1) * s.width),
        style.min,
        style.max,
        bins,
      ).counts;
      let peak = 0;
      for (const c of counts) if (c > peak) peak = c;
      if (peak === 0) continue;
      // Cell extent = half-way to each neighbour, so the column tiles seamlessly at whatever
      // depth sampling the array happens to have.
      const yc = yOf(s.depth[i]);
      const yPrev = k > 0 ? yOf(s.depth[rows[k - 1]]) : null;
      const yNext = k + 1 < rows.length ? yOf(s.depth[rows[k + 1]]) : null;
      const t = yPrev !== null ? (yPrev + yc) / 2 : yc - (yNext !== null ? (yNext - yc) / 2 : 1);
      const b = yNext !== null ? (yNext + yc) / 2 : yc + (yPrev !== null ? (yc - yPrev) / 2 : 1);
      if (b <= t) continue;
      for (let j = 0; j < bins; j++) {
        if (counts[j] === 0) continue;
        // Normalised to THIS depth's peak, matching the point track's per-bin histogram: it
        // reads the shape of the distribution at each depth rather than letting one dense
        // interval flatten every other.
        ctx.globalAlpha = counts[j] / peak;
        ctx.fillRect(left + j * bw, t, bw, b - t);
      }
    }
    ctx.globalAlpha = 1;
  }

  /** Point-data tracks (Track.kind === "point_data"): measured samples rather than a
   *  continuous log. Four displays, all drawn on the 2D overlay because none of them is a
   *  polyline the GPU pipeline can express:
   *
   *  - "points"    one glyph per sample at its own depth and value
   *  - "box"       a box plot per depth bin — box edges, median, whiskers, outliers
   *  - "histogram" a value-axis histogram per depth bin, bars scaled to the bin's peak count
   *  - "text"      the sample's text value as a label (lithology descriptions, oil show)
   *
   *  The statistics come from the shared `distribution` module, which is source-agnostic on
   *  purpose so array logs can reuse it unchanged. Values outside the track's scale are
   *  skipped, never clamped to a false position — the same rule the core overlay follows. */
  private drawPointTracks(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    if (!this.renderer || !this.layout) return;
    const pointTracks = this.layout.tracks.filter((t) => (t.kind ?? "curves") === "point_data");
    if (pointTracks.length === 0) return;
    const [top, bottom] = this.renderer.getVisibleDepthRange();
    if (bottom <= top) return;

    const themed = (name: string, fallback: string): string =>
      getComputedStyle(this.root).getPropertyValue(name).trim() || fallback;
    const textColor = themed("--text", "#332a1f");
    const dim = themed("--text-dim", "#7a6f63");
    const yOf = (d: number): number => ((d - top) / (bottom - top)) * h;

    for (const range of this.renderer.getTrackRanges()) {
      const track = pointTracks.find((t) => t.title === range.title);
      if (!track?.points?.length) continue;
      const left = range.leftFrac * w;
      const span = (range.rightFrac - range.leftFrac) * w;
      const log = track.scale_type === "log";

      for (const style of track.points) {
        const lo = log ? Math.log10(Math.max(style.min, 1e-6)) : style.min;
        const hi = log ? Math.log10(Math.max(style.max, 1e-6)) : style.max;
        if (hi === lo) continue;
        // Returns null (not a clamped edge position) for anything off-scale.
        const xOf = (v: number): number | null => {
          const tv = log ? Math.log10(Math.max(v, 1e-6)) : v;
          const frac = (tv - lo) / (hi - lo);
          return frac < 0 || frac > 1 ? null : left + frac * span;
        };
        const samples = this.pointSamples(style);
        if (samples.depth.length === 0) continue;
        ctx.save();
        ctx.beginPath();
        ctx.rect(left, 0, span, h);
        ctx.clip();
        ctx.fillStyle = style.color;
        ctx.strokeStyle = style.color;
        ctx.lineWidth = 1;

        const display = style.display ?? "points";
        if (display === "text") {
          this.drawPointText(ctx, samples, top, bottom, left, span, yOf, textColor);
        } else if (display === "box" || display === "histogram") {
          this.drawBinnedPoints(ctx, style, samples, display, top, bottom, yOf, xOf, left, span, dim);
        } else {
          this.drawPointGlyphs(ctx, samples, top, bottom, yOf, xOf);
        }
        ctx.restore();
      }
    }
  }

  /** One diamond per sample, at its own depth and value. */
  private drawPointGlyphs(
    ctx: CanvasRenderingContext2D,
    s: { depth: number[]; value: number[] },
    top: number,
    bottom: number,
    yOf: (d: number) => number,
    xOf: (v: number) => number | null,
    size = 3.5,
  ): void {
    for (let i = 0; i < s.depth.length; i++) {
      const d = s.depth[i];
      if (d < top || d > bottom) continue;
      const v = s.value[i];
      if (!Number.isFinite(v)) continue;
      const x = xOf(v);
      if (x == null) continue;
      const y = yOf(d);
      ctx.beginPath();
      ctx.moveTo(x, y - size);
      ctx.lineTo(x + size, y);
      ctx.lineTo(x, y + size);
      ctx.lineTo(x - size, y);
      ctx.closePath();
      ctx.fill();
    }
  }

  /** Text samples as labels at their depth — lithology descriptions, oil show, any aux item
   *  carrying a string. Truncated to the track width rather than spilling into the neighbour. */
  private drawPointText(
    ctx: CanvasRenderingContext2D,
    s: { depth: number[]; text: (string | null)[] },
    top: number,
    bottom: number,
    left: number,
    span: number,
    yOf: (d: number) => number,
    color: string,
  ): void {
    ctx.fillStyle = color;
    ctx.font = "10px system-ui, sans-serif";
    ctx.textBaseline = "middle";
    let lastY = -Infinity;
    for (let i = 0; i < s.depth.length; i++) {
      const d = s.depth[i];
      const label = s.text[i];
      if (d < top || d > bottom || !label) continue;
      const y = yOf(d);
      // At a coarse depth scale hundreds of descriptions land on the same few pixels; keep
      // one label per 11 px so the track stays readable instead of a black smear.
      if (y - lastY < 11) continue;
      lastY = y;
      let text = label;
      while (text.length > 1 && ctx.measureText(text).width > span - 6) {
        text = text.slice(0, -2);
      }
      ctx.fillText(text === label ? text : `${text}…`, left + 3, y);
    }
  }

  /** Box plot or histogram per depth bin. Both share the binning and the distribution stats;
   *  they differ only in the glyph drawn inside the bin's vertical extent. */
  private drawBinnedPoints(
    ctx: CanvasRenderingContext2D,
    style: PointStyle,
    s: { depth: number[]; value: number[] },
    display: "box" | "histogram",
    top: number,
    bottom: number,
    yOf: (d: number) => number,
    xOf: (v: number) => number | null,
    left: number,
    span: number,
    dimColor: string,
  ): void {
    // The default is a property of the SERIES, never of the zoom — see defaultBinHeight. An
    // explicit bin height is a fixed depth interval and likewise must not follow the zoom.
    const bin = style.bin && style.bin > 0 ? style.bin : defaultBinHeight(s.depth);
    const bins = binByDepth(s.depth, s.value, bin);
    const whisker: WhiskerRule =
      style.whisker === "minmax"
        ? { kind: "minmax" }
        : style.whisker === "percentile"
          ? { kind: "percentile", lo: style.whisker_lo ?? 10, hi: style.whisker_hi ?? 90 }
          : { kind: "tukey", k: style.whisker_k ?? 1.5 };

    for (const b of bins) {
      if (b.base < top || b.top > bottom) continue;
      const yTop = yOf(b.top);
      const yBase = yOf(b.base);
      const height = yBase - yTop;
      if (height < 2) continue; // too compressed to read — drawing it would be noise
      const mid = (yTop + yBase) / 2;

      if (display === "histogram") {
        const counts = canonicalHistogram(b.values, style.min, style.max, style.hist_bins ?? 12).counts;
        const peak = Math.max(...counts);
        if (peak === 0) continue;
        const barW = span / counts.length;
        for (let i = 0; i < counts.length; i++) {
          if (counts[i] === 0) continue;
          // Bars grow UP from the bin's base, scaled to the bin's own peak count, so a
          // sparse interval is still readable next to a densely sampled one.
          const barH = (counts[i] / peak) * (height - 1);
          ctx.globalAlpha = 0.75;
          ctx.fillRect(left + i * barW, yBase - barH, Math.max(1, barW - 0.5), barH);
          ctx.globalAlpha = 1;
        }
        continue;
      }

      const st = boxStats(b.values, style.box_lo ?? 25, style.box_hi ?? 75, whisker);
      if (!st) continue;
      const xLo = xOf(st.lo);
      const xHi = xOf(st.hi);
      const boxH = Math.max(3, Math.min(height * 0.6, 14));
      // Whiskers first, so the box paints over their inner ends.
      const wLo = xOf(st.whiskerLo);
      const wHi = xOf(st.whiskerHi);
      ctx.strokeStyle = dimColor;
      if (wLo != null && wHi != null) {
        ctx.beginPath();
        ctx.moveTo(wLo, mid);
        ctx.lineTo(wHi, mid);
        ctx.moveTo(wLo, mid - boxH / 3);
        ctx.lineTo(wLo, mid + boxH / 3);
        ctx.moveTo(wHi, mid - boxH / 3);
        ctx.lineTo(wHi, mid + boxH / 3);
        ctx.stroke();
      }
      if (xLo != null && xHi != null) {
        ctx.globalAlpha = 0.5;
        ctx.fillStyle = style.color;
        ctx.fillRect(Math.min(xLo, xHi), mid - boxH / 2, Math.abs(xHi - xLo), boxH);
        ctx.globalAlpha = 1;
        ctx.strokeStyle = style.color;
        ctx.strokeRect(Math.min(xLo, xHi), mid - boxH / 2, Math.abs(xHi - xLo), boxH);
      }
      const xMed = xOf(st.med);
      if (xMed != null) {
        ctx.strokeStyle = style.color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(xMed, mid - boxH / 2);
        ctx.lineTo(xMed, mid + boxH / 2);
        ctx.stroke();
        ctx.lineWidth = 1;
      }
      // Outliers are the whole reason to prefer Tukey — draw every one, individually.
      ctx.fillStyle = style.color;
      for (const o of st.outliers) {
        const x = xOf(o);
        if (x == null) continue;
        ctx.beginPath();
        ctx.arc(x, mid, 1.6, 0, Math.PI * 2);
        ctx.fill();
      }
      if (style.show_samples) {
        ctx.globalAlpha = 0.55;
        for (const v of b.values) {
          const x = xOf(v);
          if (x == null) continue;
          ctx.fillRect(x - 0.75, mid - boxH / 2 - 3, 1.5, 3);
        }
        ctx.globalAlpha = 1;
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

  private scheduleViewportReload(): void {
    if (this.disposed || !this.well || !this.layout || !this.renderer) return;
    if (this.viewportTimer !== undefined) window.clearTimeout(this.viewportTimer);
    this.viewportTimer = window.setTimeout(() => {
      this.viewportTimer = undefined;
      void this.reloadViewport();
    }, 100);
  }

  private viewportSourceKey(wellId: string, requests: TrackCurveRequest[]): string {
    return JSON.stringify([wellId, requests]);
  }

  private async reloadViewport(): Promise<void> {
    const well = this.well;
    const renderer = this.renderer;
    if (this.disposed || !well || !renderer || !this.layout) return;
    const [depthMin, depthMax] = renderer.getVisibleDepthRange();
    if (!Number.isFinite(depthMin) || !Number.isFinite(depthMax) || depthMax <= depthMin) return;
    const targetPixelHeight = this.canvas.clientHeight || 400;
    const requests = this.trackCurveRequests();
    const loadGen = this.loadGen;
    await this.viewportRefetch.refetch(
      {
        sourceKey: this.viewportSourceKey(well.well_id, requests),
        low: depthMin,
        high: depthMax,
        targetPixelHeight,
      },
      (tagged) =>
        getTrackData(
          well.well_id,
          requests,
          tagged.targetPixelHeight,
          tagged.low,
          tagged.high,
        ),
      (series) => {
        if (
          this.disposed ||
          loadGen !== this.loadGen ||
          this.well?.well_id !== well.well_id ||
          !this.renderer
        ) {
          return;
        }
        this.series = series;
        this.applySeriesToRenderer(true);
        this.message("");
      },
      (pending) => {
        if (this.disposed || loadGen !== this.loadGen || this.well?.well_id !== well.well_id) return;
        this.message(pending);
      },
      (failure, error) => {
        if (this.disposed || loadGen !== this.loadGen || this.well?.well_id !== well.well_id) return;
        console.error("Failed to refresh visible curve interval:", error);
        this.message(failure);
        setStatus(failure);
      },
    );
  }

  private applySeriesToRenderer(preserveDepthRange: boolean): void {
    if (!this.layout || !this.renderer) return;
    this.renderer.loadLayout(this.layout, this.series, this.trackWeights, preserveDepthRange);
    for (const curveKey of this.hiddenCurves) this.renderer.setCurveHidden(curveKey, true);
    renderReportHeader(this.reportEl, this.well, this.renderer.getDataDepthRange());
    this.refreshDepthAxis();
    this.drawCoreOverlay();
  }

  refresh(preserveDepthRange = this.depthRangeInitialized): void {
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
    this.applySeriesToRenderer(preserveDepthRange);
  }

  // --- Ribbon actions (routed to the active panel by the workspace) ---

  /** A copy of this panel's current layout (for "Save Layout…"). */
  getLayout(): Layout | null {
    return this.layout ? structuredClone(this.layout) : null;
  }

  /** Opens the Layout Properties dialog for this panel's private layout copy. */
  async openProperties(): Promise<void> {
    if (!this.layout) return;
    let available: CurveSuggestion[] = [];
    try {
      available = (await listCurveCatalog()).map((e) => ({ curve_name: e.name }));
    } catch {
      // No backend (or empty DB) — fall back to the curves the layout already references.
      available = this.layout.tracks.flatMap((t) =>
        t.curves.map((c) => ({ curve_name: c.curve_name, set_name: c.set_name })),
      );
    }
    // Imported curves remain addressable by their exact source set. A WIRE/GR and a
    // WIRE_1/GR are two distinct display requests even though the mnemonic matches.
    available.push(
      ...this.curveInventory.map((curve) => ({
        curve_name: curve.mnemonic,
        set_name: curve.set_name,
      })),
    );
    available = [...new Map(available.map((curve) => [trackCurveKey(curve), curve])).values()];
    // What this well actually carries as measured samples, so the point-track editor
    // suggests real properties and datasets instead of asking you to remember mnemonics.
    const points: PointSuggestion[] = [
      ...[...this.coreByName.keys()].map((item) => ({ source: "core" as const, item })),
      ...[
        ...new Map(
          this.auxRows.map((r) => [`${r.dataset}\u0000${r.item}`, { source: "aux" as const, dataset: r.dataset, item: r.item }]),
        ).values(),
      ],
    ];
    const before = structuredClone(this.layout);
    openLayoutPropsDialog(
      this.layout,
      available,
      (edited) => {
        this.applyLayoutEdit(edited);
        // Layout property changes are undoable (Ctrl+Z restores the previous tracks/styles).
        pushUndo({
          label: `layout properties (${edited.name})`,
          undo: () => this.applyLayoutEdit(structuredClone(before)),
          redo: () => this.applyLayoutEdit(structuredClone(edited)),
        });
      },
      points,
      this.arrayCatalog.map((c) => c.curve_name),
      this.imageCatalog.map((c) => c.dataset),
    );
  }

  /** Swaps in an edited layout, keeping user-dragged widths for surviving tracks. */
  private applyLayoutEdit(edited: Layout): void {
    this.onUserEdit?.();
    const oldWeights = new Map(this.trackWeights);
    this.layout = edited;
    this.syncCurveLabels();
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
    this.viewportRefetch.reset();
    if (this.viewportTimer !== undefined) {
      window.clearTimeout(this.viewportTimer);
      this.viewportTimer = undefined;
    }
    for (const unsub of this.unsubscribers) unsub();
    this.resizeObserver?.disconnect();
    this.topsEditor.dispose();
    this.highlightsOverlay.dispose();
    // Decoded plates hold GPU-backed memory that garbage collection does not reclaim on its
    // own; a closed panel must give them back.
    for (const b of this.imageBitmaps.values()) b.close();
    this.imageBitmaps.clear();
    this.renderer?.dispose();
    this.renderer = null;
  }
}
