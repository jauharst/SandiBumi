import { listTops, type TopEntry } from "../ipc";
import type { TopInterval } from "../state";

/** Formation tops for the currently selected well — SandiBumi's equivalent of Geolog's
 * TOPS_GEO.TOPS interval log panel. Clicking a top selects the interval from it down to
 * the next top (or TD); clicking it again deselects. */
export class TopsPanel {
  private container: HTMLElement;
  private wellId: string | null = null;
  private tops: TopEntry[] = [];
  private selectedTop: string | null = null;
  /** Fired with the clicked top's interval, or null when the selection is cleared. */
  public onSelectInterval: ((interval: TopInterval | null) => void) | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.renderEmpty("Select a well");
  }

  async refresh(wellId: string | null): Promise<void> {
    if (wellId !== this.wellId) this.selectedTop = null;
    this.wellId = wellId;
    if (!wellId) {
      this.tops = [];
      this.renderEmpty("Select a well");
      return;
    }
    try {
      this.tops = (await listTops(wellId)).slice().sort((a, b) => a.depth - b.depth);
      if (this.tops.length === 0) {
        this.renderEmpty("No tops for this well");
        return;
      }
      this.render();
    } catch (err) {
      console.error("Failed to load tops:", err);
      this.tops = [];
      this.renderEmpty("Unable to load tops");
    }
  }

  private render(): void {
    this.container.innerHTML = "";
    this.tops.forEach((t, i) => {
      const node = document.createElement("div");
      node.className = "tree-node top-node" + (t.top_name === this.selectedTop ? " top-selected" : "");
      node.title = "Click to window plots and log views to this interval";
      node.innerHTML = `
        <span class="top-color" style="background:${t.color ?? "#8b8f96"}"></span>
        <span class="top-name">${escapeHtml(t.top_name)}</span>
        <span class="top-depth">${t.depth.toFixed(1)}</span>`;
      node.addEventListener("click", () => this.toggle(i));
      this.container.appendChild(node);
    });
  }

  private toggle(index: number): void {
    const top = this.tops[index];
    if (!top || !this.wellId) return;
    if (this.selectedTop === top.top_name) {
      this.selectedTop = null;
      this.onSelectInterval?.(null);
    } else {
      this.selectedTop = top.top_name;
      this.onSelectInterval?.({
        wellId: this.wellId,
        topName: top.top_name,
        depthMin: top.depth,
        depthMax: this.tops[index + 1]?.depth ?? null,
      });
    }
    this.render();
  }

  private renderEmpty(text: string): void {
    this.container.innerHTML = `<div class="tree-empty">${text}</div>`;
  }
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}
