import { listTops, type TopEntry } from "../ipc";
import type { TopInterval } from "../state";
import { escapeHtml } from "./safeDom";

/** The rows on screen together with the well they were fetched for. The two are ONE unit so a
 *  click can never pair a freshly-selected well id with the previous well's rows — the mismatch
 *  that used to publish `{wellId: B, topName: <A's>, depthMin: <A's depth>}`. Same snapshot shape
 *  as dbInspectorPanel's GridView, and for the same reason. */
interface TopsView {
  wellId: string;
  tops: TopEntry[];
}

/** Formation tops for the currently selected well — SandiBumi's own formation-tops
 * interval panel. Clicking a top selects the interval from it down to
 * the next top (or TD); clicking it again deselects. */
export class TopsPanel {
  private container: HTMLElement;
  /** What is actually painted, well id and rows together. Never assign one without the other. */
  private view: TopsView | null = null;
  private selectedTop: string | null = null;
  /** Bumped on every refresh; a superseded in-flight fetch bails instead of repainting. */
  private refreshGen = 0;
  /** Fired with the clicked top's interval, or null when the selection is cleared. */
  public onSelectInterval: ((interval: TopInterval | null) => void) | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.renderEmpty("Select a well");
  }

  async refresh(wellId: string | null): Promise<void> {
    const gen = ++this.refreshGen;
    const wellChanged = wellId !== this.view?.wellId;
    if (wellChanged) this.selectedTop = null;
    if (!wellId) {
      this.view = null;
      this.renderEmpty("Select a well");
      return;
    }
    // A different well: drop the old rows NOW, before the await. This is not a lost-race guard —
    // list_tops is a synchronous #[tauri::command] (lib.rs:694), so responses already resolve FIFO.
    // The bug was simpler and deterministic: the previous well's rows stayed on screen, and stayed
    // clickable, for the whole width of the DuckDB query while the id had already moved on. Clicking
    // one minted the OLD well's top name and depths under the NEW well's id, and logViewPanel.ts:341
    // and plotCommon.ts:322 accept an interval on the id match alone — so well B's log view scrolled
    // to well A's depth and every plot of B silently re-windowed to a foreign interval, which is a
    // parameter pick (Rw, m/n, cutoffs) read off the wrong zone. Clearing first makes that click
    // unreachable rather than merely unlikely. A same-well refresh (dataVersion after a run) keeps
    // its rows, so a recompute does not flicker the pane.
    if (wellChanged) this.renderEmpty("Loading tops…");
    try {
      const tops = (await listTops(wellId)).slice().sort((a, b) => a.depth - b.depth);
      if (gen !== this.refreshGen) return; // a newer refresh owns the pane now
      this.view = { wellId, tops };
      if (tops.length === 0) {
        this.renderEmpty("No tops for this well");
        return;
      }
      this.render();
    } catch (err) {
      if (gen !== this.refreshGen) return;
      console.error("Failed to load tops:", err);
      this.view = null;
      this.renderEmpty(`Unable to load tops: ${String(err)}`);
    }
  }

  private render(): void {
    const view = this.view;
    if (!view) return;
    this.container.innerHTML = "";
    view.tops.forEach((t, i) => {
      const node = document.createElement("div");
      node.className = "tree-node top-node" + (t.top_name === this.selectedTop ? " top-selected" : "");
      const sourceReference =
        t.source_depth_datum && t.source_depth_datum !== "MD"
          ? `; source ${t.source_depth.toFixed(1)} ${t.source_depth_datum}, survey-resolved to ${t.depth.toFixed(1)} MD`
          : "";
      node.title = `Click to window plots and log views to this MD interval${sourceReference}`;
      node.innerHTML = `
        <span class="top-color" style="background:${t.color ?? "#8b8f96"}"></span>
        <span class="top-name">${escapeHtml(t.top_name)}</span>
        <span class="top-depth">${t.depth.toFixed(1)} MD${
          t.source_depth_datum && t.source_depth_datum !== "MD"
            ? ` ← ${t.source_depth.toFixed(1)} ${t.source_depth_datum}`
            : ""
        }</span>`;
      // `view` is captured, not re-read: a row emits the interval for the well it was PAINTED for,
      // so even a node that outlives its refresh can only publish a self-consistent interval.
      node.addEventListener("click", () => this.toggle(view, i));
      this.container.appendChild(node);
    });
  }

  private toggle(view: TopsView, index: number): void {
    const top = view.tops[index];
    if (!top) return;
    if (this.selectedTop === top.top_name) {
      this.selectedTop = null;
      this.onSelectInterval?.(null);
    } else {
      this.selectedTop = top.top_name;
      this.onSelectInterval?.({
        wellId: view.wellId,
        topName: top.top_name,
        depthMin: top.depth,
        depthMax: view.tops[index + 1]?.depth ?? null,
      });
    }
    this.render();
  }

  private renderEmpty(text: string): void {
    this.container.innerHTML = `<div class="tree-empty">${escapeHtml(text)}</div>`;
  }
}
