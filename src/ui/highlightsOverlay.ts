import {
  deleteHighlight,
  deleteZone,
  listHighlights,
  listZones,
  upsertHighlight,
  upsertZone,
  type HighlightEntry,
} from "../ipc";
import { recordProcess } from "../processLog";
import { bumpDataVersion, setStatus } from "../state";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";

/** Minimum drag span (in device-independent px) below which a pointerup is treated as a
 *  click, not a band creation — keeps double-clicks and stray taps from making zero-height
 *  highlights. */
const MIN_DRAG_PX = 4;
const DEFAULT_ALPHA = 0.16;

/** Rotating default palette for freshly created highlights (all editable afterwards) —
 *  derived from the ACTIVE theme's vars at call time, so new bands land in the current
 *  palette (light/dark/client-branded) instead of a fixed light-theme set. Values are
 *  hex-validated because they seed <input type=color> and are persisted per highlight. */
function palette(): string[] {
  const s = getComputedStyle(document.documentElement);
  const v = (name: string, fallback: string) => {
    const raw = s.getPropertyValue(name).trim();
    return /^#[0-9a-fA-F]{3}(?:[0-9a-fA-F]{3})?$/.test(raw) ? raw : fallback;
  };
  return [
    v("--accent", "#e0b64a"),
    v("--accent2", "#5fae5f"),
    v("--warn", "#d9694a"),
    v("--accent-dim", "#4a90d9"),
    v("--border-strong", "#9b6fd0"),
    v("--text-dim", "#3fb0b0"),
  ];
}

/** Colored highlight overlay for one log view: draws the well's informal colored
 *  depth bands across every track (mark pay, bad hole, intervals of interest), independent
 *  of formal tops/zones. With edit mode ON (🖍 in the view toolbar), drag a depth span to
 *  create a band; double-click a band to recolor/label/resize it, convert it to a zone, or
 *  delete it. Bands are translucent so curves read through them, and sit just below the tops
 *  layer so top lines/labels stay legible. Every change is undoable. */
export class HighlightsOverlay {
  private overlay: HTMLCanvasElement;
  private wellId: string | null = null;
  private highlights: HighlightEntry[] = [];
  private editMode = false;

  /** Live drag-create preview: anchor depth + current depth, or null when idle. */
  private dragFrom: number | null = null;
  private dragTo = 0;

  constructor(
    body: HTMLElement,
    private logCanvas: HTMLCanvasElement,
    /** Current [top, bottom] visible depth range of the log renderer. */
    private getRange: () => [number, number],
  ) {
    this.overlay = document.createElement("canvas");
    this.overlay.className = "highlights-overlay";
    body.appendChild(this.overlay);
    this.bindPointerHandlers();
  }

  dispose(): void {
    this.overlay.remove();
  }

  get editing(): boolean {
    return this.editMode;
  }

  setEditMode(on: boolean): void {
    this.editMode = on;
    this.overlay.classList.toggle("editing", on);
    if (!on) this.dragFrom = null;
    this.draw();
  }

  /** (Re)loads highlights for the well; call from loadWell and on dataVersion bumps. */
  async setWell(wellId: string | null): Promise<void> {
    this.wellId = wellId;
    if (!wellId) {
      this.highlights = [];
      this.draw();
      return;
    }
    try {
      this.highlights = (await listHighlights(wellId)).slice().sort((a, b) => a.top_depth - b.top_depth);
    } catch {
      this.highlights = []; // no backend — overlay stays empty
    }
    this.draw();
  }

  /** Redraws all bands (+ any live drag preview); call after every rendered frame so
   *  highlights track pan/zoom. */
  draw(): void {
    const w = this.logCanvas.clientWidth;
    const h = this.logCanvas.clientHeight;
    if (w === 0 || h === 0) return;
    this.overlay.style.left = `${this.logCanvas.offsetLeft}px`;
    this.overlay.style.top = `${this.logCanvas.offsetTop}px`;
    if (this.overlay.width !== w || this.overlay.height !== h) {
      this.overlay.width = w;
      this.overlay.height = h;
    }
    const ctx = this.overlay.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);

    const [top, bottom] = this.getRange();
    if (bottom <= top) return;
    const span = bottom - top;
    ctx.font = canvasFont(readTheme(this.overlay), 10.5, 400);
    ctx.textBaseline = "top";

    const pal = palette();
    this.highlights.forEach((hl, i) => {
      const color = hl.color ?? pal[i % pal.length];
      this.paintBand(ctx, w, h, top, span, hl.top_depth, hl.bottom_depth, color, hl.label, false);
    });

    // Live drag preview (dashed) on top of committed bands.
    if (this.dragFrom !== null) {
      const a = Math.min(this.dragFrom, this.dragTo);
      const b = Math.max(this.dragFrom, this.dragTo);
      this.paintBand(ctx, w, h, top, span, a, b, pal[this.highlights.length % pal.length], null, true);
    }
  }

  /** Paints one translucent band with top/bottom edge lines and an optional label chip. */
  private paintBand(
    ctx: CanvasRenderingContext2D,
    w: number,
    h: number,
    top: number,
    span: number,
    d0: number,
    d1: number,
    color: string,
    label: string | null,
    preview: boolean,
  ): void {
    const yTopRaw = ((d0 - top) / span) * h;
    const yBotRaw = ((d1 - top) / span) * h;
    if (yBotRaw < 0 || yTopRaw > h) return; // fully off-screen
    const yTop = Math.max(0, yTopRaw);
    const yBot = Math.min(h, yBotRaw);
    const bandH = Math.max(0, yBot - yTop);

    ctx.save();
    ctx.globalAlpha = preview ? DEFAULT_ALPHA + 0.06 : DEFAULT_ALPHA;
    ctx.fillStyle = color;
    ctx.fillRect(0, yTop, w, bandH);
    ctx.globalAlpha = preview ? 0.9 : 0.7;
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.setLineDash(preview ? [5, 3] : []);
    if (yTopRaw >= 0) {
      ctx.beginPath();
      ctx.moveTo(0, yTop + 0.5);
      ctx.lineTo(w, yTop + 0.5);
      ctx.stroke();
    }
    if (yBotRaw <= h) {
      ctx.beginPath();
      ctx.moveTo(0, yBot - 0.5);
      ctx.lineTo(w, yBot - 0.5);
      ctx.stroke();
    }
    ctx.setLineDash([]);

    const text = label && label.trim() ? label.trim() : preview ? `${d0.toFixed(1)}–${d1.toFixed(1)}` : "";
    if (text && bandH >= 14 && yTopRaw >= -2) {
      ctx.globalAlpha = 0.92;
      const tw = ctx.measureText(text).width;
      ctx.fillStyle = color;
      ctx.fillRect(2, yTop + 2, tw + 8, 13);
      ctx.globalAlpha = 1;
      ctx.fillStyle = contrastText(color) ?? "#f5f5f5";
      ctx.fillText(text, 6, yTop + 3.5);
    }
    ctx.restore();
  }

  // --- Interaction (active only while .editing gives the overlay pointer events) ---

  private bindPointerHandlers(): void {
    this.overlay.addEventListener("pointerdown", (e) => {
      if (!this.editMode || e.button !== 0) return;
      this.dragFrom = this.depthAt(e.offsetY);
      this.dragTo = this.dragFrom;
      this.overlay.setPointerCapture(e.pointerId);
    });
    this.overlay.addEventListener("pointermove", (e) => {
      if (!this.editMode) return;
      if (this.dragFrom !== null) {
        this.dragTo = this.depthAt(e.offsetY);
        this.draw();
        return;
      }
      this.overlay.style.cursor = this.hitTest(e.offsetY) !== null ? "pointer" : "crosshair";
    });
    this.overlay.addEventListener("pointerup", () => {
      if (!this.editMode || this.dragFrom === null) return;
      const from = this.dragFrom;
      this.dragFrom = null;
      const dragPx = Math.abs(this.depthToY(this.dragTo) - this.depthToY(from));
      if (dragPx < MIN_DRAG_PX) {
        this.draw(); // treat as a click — clear the preview
        return;
      }
      const a = Math.min(from, this.dragTo);
      const b = Math.max(from, this.dragTo);
      void this.createHighlight(a, b);
    });
    this.overlay.addEventListener("dblclick", (e) => {
      if (!this.editMode) return;
      const idx = this.hitTest(e.offsetY);
      if (idx !== null) this.openEditDialog(this.highlights[idx]);
    });
    // Keep wheel zoom/pan alive in edit mode by forwarding to the log canvas.
    this.overlay.addEventListener(
      "wheel",
      (e) => {
        if (!this.editMode) return;
        e.preventDefault();
        this.logCanvas.dispatchEvent(new WheelEvent("wheel", e));
      },
      { passive: false },
    );
  }

  private depthAt(offsetY: number): number {
    const [top, bottom] = this.getRange();
    const h = this.overlay.clientHeight || 1;
    return top + (offsetY / h) * (bottom - top);
  }

  private depthToY(depth: number): number {
    const [top, bottom] = this.getRange();
    if (bottom <= top) return 0;
    const h = this.overlay.clientHeight || 1;
    return ((depth - top) / (bottom - top)) * h;
  }

  /** Topmost band whose span contains offsetY (null if none). */
  private hitTest(offsetY: number): number | null {
    const depth = this.depthAt(offsetY);
    for (let i = this.highlights.length - 1; i >= 0; i--) {
      const hl = this.highlights[i];
      if (depth >= hl.top_depth && depth <= hl.bottom_depth) return i;
    }
    return null;
  }

  // --- Commits (all undoable) ---

  private async createHighlight(topDepth: number, bottomDepth: number): Promise<void> {
    const wellId = this.wellId;
    if (!wellId) return;
    const id = newId();
    const pal = palette();
    const color = pal[this.highlights.length % pal.length];
    const apply = async () => {
      await upsertHighlight(wellId, id, topDepth, bottomDepth, color, null);
      bumpDataVersion();
    };
    try {
      await apply();
    } catch (err) {
      setStatus(`Add highlight failed: ${err}`);
      this.draw();
      return;
    }
    pushUndo({
      label: "add highlight",
      undo: async () => {
        await deleteHighlight(wellId, id);
        bumpDataVersion();
      },
      redo: apply,
    });
    setStatus(`Highlight ${topDepth.toFixed(1)}–${bottomDepth.toFixed(1)} added`);
    recordProcess("Highlights", `Added highlight ${topDepth.toFixed(1)}–${bottomDepth.toFixed(1)}`);
    // Open the editor immediately so a color/label can be set on the fresh band. The reload
    // triggered by bumpDataVersion() is async, so build the entry locally rather than search
    // the (not-yet-refreshed) list.
    this.openEditDialog({ highlight_id: id, top_depth: topDepth, bottom_depth: bottomDepth, color, label: null });
  }

  private openEditDialog(hl: HighlightEntry): void {
    const wellId = this.wellId;
    if (!wellId) return;
    const before: HighlightEntry = { ...hl };

    const content = document.createElement("div");
    const labelInput = document.createElement("input");
    labelInput.className = "form-control";
    labelInput.placeholder = "e.g. Pay / Bad hole / Zone of interest";
    labelInput.value = hl.label ?? "";
    const topInput = numberInput(hl.top_depth);
    const botInput = numberInput(hl.bottom_depth);
    const colorInput = document.createElement("input");
    colorInput.type = "color";
    colorInput.value = hl.color ?? palette()[0];

    content.appendChild(formRow("Label", labelInput));
    content.appendChild(formRow("Top", topInput));
    content.appendChild(formRow("Bottom", botInput));
    content.appendChild(formRow("Color", colorInput));

    const row = document.createElement("div");
    row.className = "form-row hl-edit-actions";
    const saveBtn = actionBtn("Save");
    const zoneBtn = actionBtn("Convert to zone");
    const delBtn = actionBtn("Delete");
    row.append(saveBtn, zoneBtn, delBtn);
    content.appendChild(row);
    const close = openModal("Edit highlight", content, 380);
    labelInput.focus();

    const readFields = (): { top: number; bottom: number; color: string; label: string | null } | null => {
      let top = parseFloat(topInput.value);
      let bottom = parseFloat(botInput.value);
      if (!Number.isFinite(top) || !Number.isFinite(bottom) || top === bottom) {
        setStatus("Highlight needs two different numeric depths");
        return null;
      }
      if (top > bottom) [top, bottom] = [bottom, top];
      const label = labelInput.value.trim() || null;
      return { top, bottom, color: colorInput.value, label };
    };

    saveBtn.addEventListener("click", () => {
      void (async () => {
        const f = readFields();
        if (!f) return;
        const applyNew = async () => {
          await upsertHighlight(wellId, before.highlight_id, f.top, f.bottom, f.color, f.label);
          bumpDataVersion();
        };
        const applyOld = async () => {
          await upsertHighlight(wellId, before.highlight_id, before.top_depth, before.bottom_depth, before.color, before.label);
          bumpDataVersion();
        };
        try {
          await applyNew();
        } catch (err) {
          setStatus(`Edit highlight failed: ${err}`);
          return;
        }
        pushUndo({ label: "edit highlight", undo: applyOld, redo: applyNew });
        setStatus(`Highlight ${f.top.toFixed(1)}–${f.bottom.toFixed(1)} saved`);
        recordProcess("Highlights", `Edited highlight ${f.top.toFixed(1)}–${f.bottom.toFixed(1)}${f.label ? ` (${f.label})` : ""}`);
        close();
      })();
    });

    zoneBtn.addEventListener("click", () => {
      void (async () => {
        const f = readFields();
        if (!f) return;
        const name = (f.label || `HL_${f.top.toFixed(0)}_${f.bottom.toFixed(0)}`).toUpperCase().replace(/\s+/g, "_");
        // If a zone with this name exists, capture its geometry so undo can restore it.
        let prev: { top: number; bottom: number } | null = null;
        try {
          const existing = (await listZones(wellId)).find((z) => z.zone_name === name);
          if (existing) prev = { top: existing.top_depth, bottom: existing.bottom_depth };
        } catch {
          /* no backend — treat as new */
        }
        const applyNew = async () => {
          await upsertZone(wellId, name, f.top, f.bottom);
          bumpDataVersion();
        };
        try {
          await applyNew();
        } catch (err) {
          setStatus(`Convert to zone failed: ${err}`);
          return;
        }
        pushUndo({
          label: `zone from highlight ${name}`,
          undo: async () => {
            if (prev) await upsertZone(wellId, name, prev.top, prev.bottom);
            else await deleteZone(wellId, name);
            bumpDataVersion();
          },
          redo: applyNew,
        });
        setStatus(`Zone ${name} created (${f.top.toFixed(1)}–${f.bottom.toFixed(1)})`);
        recordProcess("Highlights", `Converted highlight to zone ${name}`);
        close();
      })();
    });

    delBtn.addEventListener("click", () => {
      void (async () => {
        try {
          await deleteHighlight(wellId, before.highlight_id);
        } catch (err) {
          setStatus(`Delete highlight failed: ${err}`);
          return;
        }
        bumpDataVersion();
        pushUndo({
          label: "delete highlight",
          undo: async () => {
            await upsertHighlight(wellId, before.highlight_id, before.top_depth, before.bottom_depth, before.color, before.label);
            bumpDataVersion();
          },
          redo: async () => {
            await deleteHighlight(wellId, before.highlight_id);
            bumpDataVersion();
          },
        });
        setStatus("Highlight deleted");
        recordProcess("Highlights", `Deleted highlight ${before.top_depth.toFixed(1)}–${before.bottom_depth.toFixed(1)}`);
        close();
      })();
    });
  }
}

function numberInput(value: number): HTMLInputElement {
  const el = document.createElement("input");
  el.className = "form-control";
  el.type = "number";
  el.step = "0.1";
  el.value = value.toFixed(1);
  return el;
}

function actionBtn(text: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "lp-btn";
  b.textContent = text;
  return b;
}

/** Stable unique id; falls back if crypto.randomUUID is unavailable. */
function newId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return `hl-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e9).toString(36)}`;
  }
}

/** Black or white, whichever reads better on `hex` (returns null on parse failure). */
function contrastText(hex: string): string | null {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return null;
  const v = parseInt(m[1], 16);
  const lum = 0.299 * ((v >> 16) & 255) + 0.587 * ((v >> 8) & 255) + 0.114 * (v & 255);
  return lum > 140 ? "#1a1a1a" : "#f5f5f5";
}
