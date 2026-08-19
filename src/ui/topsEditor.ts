import { checkTopOrder, deleteTop, listTops, upsertTop, type TopEntry } from "../ipc";
import { recordProcess } from "../processLog";
import { bumpDataVersion, setStatus } from "../state";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";

const GRAB_PX = 6; // vertical pick tolerance around a top line
const DEFAULT_COLOR = "#e2b93d";

/** Petrel-style tops overlay + editor for one log view: always draws the well's tops
 *  as labeled lines across the tracks; with edit mode ON (🏷 in the view toolbar),
 *  click an empty depth to add a top, drag a line to move it, double-click a line to
 *  rename/recolor/delete. Every change is undoable and immediately re-checked for
 *  stratigraphic crossings against the other wells. */
export class TopsEditor {
  private overlay: HTMLCanvasElement;
  private wellId: string | null = null;
  private tops: TopEntry[] = [];
  private editMode = false;

  /** Index into `tops` currently being dragged, with its live preview depth. */
  private dragIdx: number | null = null;
  private dragDepth = 0;
  private dragMoved = false;

  constructor(
    body: HTMLElement,
    private logCanvas: HTMLCanvasElement,
    /** Current [top, bottom] visible depth range of the log renderer. */
    private getRange: () => [number, number],
  ) {
    this.overlay = document.createElement("canvas");
    this.overlay.className = "tops-overlay";
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
    if (!on) this.dragIdx = null;
    this.draw();
  }

  /** (Re)loads tops for the well; call from loadWell and on dataVersion bumps. */
  async setWell(wellId: string | null): Promise<void> {
    this.wellId = wellId;
    if (!wellId) {
      this.tops = [];
      this.draw();
      return;
    }
    try {
      this.tops = (await listTops(wellId)).slice().sort((a, b) => a.depth - b.depth);
    } catch (err) {
      this.tops = [];
      setStatus(`Tops unavailable: ${String(err)}`);
    }
    this.draw();
  }

  /** Redraws lines + labels; call after every rendered frame so tops track pan/zoom. */
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
    if (this.tops.length === 0) return;

    const [top, bottom] = this.getRange();
    if (bottom <= top) return;
    const textColor = getComputedStyle(document.documentElement).getPropertyValue("--text").trim() || "#ddd";
    ctx.font = canvasFont(readTheme(this.overlay), 10.5, 400);
    ctx.textBaseline = "bottom";

    this.tops.forEach((t, i) => {
      const depth = i === this.dragIdx ? this.dragDepth : t.depth;
      if (depth < top || depth > bottom) return;
      const y = ((depth - top) / (bottom - top)) * h;
      const color = t.color ?? DEFAULT_COLOR;
      ctx.strokeStyle = color;
      ctx.lineWidth = i === this.dragIdx ? 2 : 1.25;
      ctx.setLineDash(i === this.dragIdx ? [6, 3] : []);
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
      ctx.setLineDash([]);

      const sourceLabel =
        t.source_depth_datum && t.source_depth_datum !== "MD"
          ? ` MD ← ${t.source_depth.toFixed(1)} ${t.source_depth_datum}`
          : " MD";
      const label = `${t.top_name}  ${depth.toFixed(1)}${sourceLabel}`;
      const tw = ctx.measureText(label).width;
      ctx.fillStyle = color;
      ctx.globalAlpha = 0.9;
      ctx.fillRect(2, y - 14, tw + 8, 13);
      ctx.globalAlpha = 1;
      ctx.fillStyle = contrastText(color) ?? textColor;
      ctx.fillText(label, 6, y - 2.5);
    });
  }

  // --- Interaction (active only while .editing gives the overlay pointer events) ---

  private bindPointerHandlers(): void {
    this.overlay.addEventListener("pointerdown", (e) => {
      if (!this.editMode || e.button !== 0) return;
      const idx = this.hitTest(e.offsetY);
      if (idx !== null) {
        this.dragIdx = idx;
        this.dragDepth = this.tops[idx].depth;
        this.dragMoved = false;
        this.overlay.setPointerCapture(e.pointerId);
      }
    });
    this.overlay.addEventListener("pointermove", (e) => {
      if (!this.editMode) return;
      if (this.dragIdx !== null) {
        this.dragDepth = this.depthAt(e.offsetY);
        this.dragMoved = true;
        this.draw();
        return;
      }
      this.overlay.style.cursor = this.hitTest(e.offsetY) !== null ? "ns-resize" : "crosshair";
    });
    this.overlay.addEventListener("pointerup", (e) => {
      if (!this.editMode) return;
      if (this.dragIdx !== null) {
        const idx = this.dragIdx;
        this.dragIdx = null;
        if (this.dragMoved && Math.abs(this.dragDepth - this.tops[idx].depth) > 1e-6) {
          void this.commitMove(this.tops[idx], this.dragDepth);
        } else {
          this.draw();
        }
        return;
      }
      // Plain click on empty depth → add a new top there.
      this.openAddDialog(this.depthAt(e.offsetY));
    });
    this.overlay.addEventListener("dblclick", (e) => {
      if (!this.editMode) return;
      const idx = this.hitTest(e.offsetY);
      if (idx !== null) this.openEditDialog(this.tops[idx]);
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

  private hitTest(offsetY: number): number | null {
    const [top, bottom] = this.getRange();
    if (bottom <= top) return null;
    const h = this.overlay.clientHeight || 1;
    let best: number | null = null;
    let bestDist = GRAB_PX + 1;
    this.tops.forEach((t, i) => {
      const y = ((t.depth - top) / (bottom - top)) * h;
      const dist = Math.abs(y - offsetY);
      if (dist <= GRAB_PX && dist < bestDist) {
        best = i;
        bestDist = dist;
      }
    });
    return best;
  }

  // --- Commits (all undoable, all followed by a crossing check) ---

  private async commitMove(top: TopEntry, newDepth: number): Promise<void> {
    const wellId = this.wellId;
    if (!wellId) return;
    const oldDepth = top.depth;
    const apply = async (depth: number) => {
      await upsertTop(wellId, top.top_name, depth, null);
      bumpDataVersion();
    };
    try {
      await apply(newDepth);
    } catch (err) {
      setStatus(`Move failed: ${err}`);
      this.draw();
      return;
    }
    pushUndo({
      label: `move top ${top.top_name}`,
      undo: () => apply(oldDepth),
      redo: () => apply(newDepth),
    });
    setStatus(`${top.top_name}: ${oldDepth.toFixed(1)} → ${newDepth.toFixed(1)}`);
    void this.warnCrossings();
  }

  private openAddDialog(depth: number): void {
    const wellId = this.wellId;
    if (!wellId) return;
    const content = document.createElement("div");
    const nameInput = document.createElement("input");
    nameInput.className = "form-control";
    nameInput.placeholder = "e.g. TOP_BEKASAP";
    const depthInput = document.createElement("input");
    depthInput.className = "form-control";
    depthInput.type = "number";
    depthInput.step = "0.1";
    depthInput.value = depth.toFixed(1);
    const colorInput = document.createElement("input");
    colorInput.type = "color";
    colorInput.value = DEFAULT_COLOR;
    const addBtn = document.createElement("button");
    addBtn.className = "lp-btn";
    addBtn.textContent = "Add top";
    content.appendChild(formRow("Name", nameInput));
    content.appendChild(formRow("Depth", depthInput));
    content.appendChild(formRow("Color", colorInput));
    content.appendChild(addBtn);
    const close = openModal("New top", content, 360);
    nameInput.focus();

    const submit = async () => {
      const name = nameInput.value.trim().toUpperCase();
      const d = parseFloat(depthInput.value);
      if (!name || !Number.isFinite(d)) {
        setStatus("Top needs a name and a numeric depth");
        return;
      }
      const color = colorInput.value;
      // A same-name top makes this an overwrite — undo must RESTORE the previous
      // depth, not delete the top outright.
      const previous = this.tops.find((t) => t.top_name === name);
      const prevDepth = previous?.depth;
      try {
        await upsertTop(wellId, name, d, color);
      } catch (err) {
        setStatus(`Add top failed: ${err}`);
        return;
      }
      bumpDataVersion();
      pushUndo({
        label: prevDepth === undefined ? `add top ${name}` : `overwrite top ${name}`,
        undo: async () => {
          if (prevDepth === undefined) await deleteTop(wellId, name);
          else await upsertTop(wellId, name, prevDepth, null);
          bumpDataVersion();
        },
        redo: async () => {
          await upsertTop(wellId, name, d, color);
          bumpDataVersion();
        },
      });
      setStatus(`Added top ${name} at ${d.toFixed(1)}`);
      recordProcess("Tops", `Added top ${name} at ${d.toFixed(1)}`);
      close();
      void this.warnCrossings();
    };
    addBtn.addEventListener("click", () => void submit());
    nameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void submit();
    });
    depthInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") void submit();
    });
  }

  private openEditDialog(top: TopEntry): void {
    const wellId = this.wellId;
    if (!wellId) return;
    if (top.source_depth_datum && top.source_depth_datum !== "MD") {
      setStatus(
        `${top.source_depth_datum}-referenced top ${top.top_name} is read-only in the MD tops editor; use a source-reference-aware workflow`,
      );
      return;
    }
    const before = { ...top };
    const content = document.createElement("div");
    const nameInput = document.createElement("input");
    nameInput.className = "form-control";
    nameInput.value = top.top_name;
    const depthInput = document.createElement("input");
    depthInput.className = "form-control";
    depthInput.type = "number";
    depthInput.step = "0.1";
    depthInput.value = top.depth.toFixed(1);
    const colorInput = document.createElement("input");
    colorInput.type = "color";
    colorInput.value = top.color ?? DEFAULT_COLOR;
    const row = document.createElement("div");
    row.className = "form-row";
    const saveBtn = document.createElement("button");
    saveBtn.className = "lp-btn";
    saveBtn.textContent = "Save";
    const delBtn = document.createElement("button");
    delBtn.className = "lp-btn";
    delBtn.textContent = "Delete";
    row.appendChild(saveBtn);
    row.appendChild(delBtn);
    content.appendChild(formRow("Name", nameInput));
    content.appendChild(formRow("Depth", depthInput));
    content.appendChild(formRow("Color", colorInput));
    content.appendChild(row);
    const close = openModal(`Edit top — ${top.top_name}`, content, 360);

    saveBtn.addEventListener("click", () => {
      void (async () => {
        const name = nameInput.value.trim().toUpperCase();
        const d = parseFloat(depthInput.value);
        if (!name || !Number.isFinite(d)) {
          setStatus("Top needs a name and a numeric depth");
          return;
        }
        const color = colorInput.value;
        const applyNew = async () => {
          if (name !== before.top_name) await deleteTop(wellId, before.top_name);
          await upsertTop(wellId, name, d, color);
          bumpDataVersion();
        };
        const applyOld = async () => {
          if (name !== before.top_name) await deleteTop(wellId, name);
          await upsertTop(wellId, before.top_name, before.depth, before.color ?? DEFAULT_COLOR);
          bumpDataVersion();
        };
        try {
          await applyNew();
        } catch (err) {
          setStatus(`Edit top failed: ${err}`);
          return;
        }
        pushUndo({ label: `edit top ${before.top_name}`, undo: applyOld, redo: applyNew });
        setStatus(`Top ${name} saved`);
        recordProcess("Tops", `Edited top ${before.top_name}${name !== before.top_name ? ` → ${name}` : ""} @ ${d.toFixed(1)}`);
        close();
        void this.warnCrossings();
      })();
    });
    delBtn.addEventListener("click", () => {
      void (async () => {
        try {
          await deleteTop(wellId, before.top_name);
        } catch (err) {
          setStatus(`Delete top failed: ${err}`);
          return;
        }
        bumpDataVersion();
        pushUndo({
          label: `delete top ${before.top_name}`,
          undo: async () => {
            await upsertTop(wellId, before.top_name, before.depth, before.color ?? DEFAULT_COLOR);
            bumpDataVersion();
          },
          redo: async () => {
            await deleteTop(wellId, before.top_name);
            bumpDataVersion();
          },
        });
        setStatus(`Deleted top ${before.top_name}`);
        recordProcess("Tops", `Deleted top ${before.top_name}`);
        close();
      })();
    });
  }

  /** Stratigraphic sanity check after every pick: surfaces crossings in the status bar. */
  private async warnCrossings(): Promise<void> {
    if (!this.wellId) return;
    try {
      const warnings = await checkTopOrder(this.wellId);
      if (warnings.length > 0) {
        const extra = warnings.length > 1 ? ` (+${warnings.length - 1} more crossing${warnings.length > 2 ? "s" : ""})` : "";
        setStatus(`⚠ Crossing: ${warnings[0]}${extra}`);
      }
    } catch {
      // no backend — skip silently
    }
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
