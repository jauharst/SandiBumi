import { save } from "@tauri-apps/plugin-dialog";
import { savePng } from "../ipc";
import { setStatus } from "../state";
import {
  clearProcessLog,
  processLogToText,
  subscribeProcessLog,
  type ProcessEntry,
} from "../processLog";

/** The Processing History panel: a live, timestamped audit of everything done in the
 *  project (imports, module runs, equations, edits, exports). It persists with the
 *  project and can be exported to a text file so a study's processing is documented. */
export class HistoryPanel {
  private list: HTMLElement;
  private unsub: () => void;

  constructor(host: HTMLElement) {
    host.classList.add("history-panel");
    host.innerHTML = `
      <div class="history-toolbar">
        <span class="history-count"></span>
        <span class="history-spacer"></span>
        <button class="plot-export-btn history-export">⭳ Export…</button>
        <button class="plot-export-btn history-clear">🗑 Clear</button>
      </div>
      <div class="history-list"></div>`;
    this.list = host.querySelector<HTMLElement>(".history-list")!;
    const count = host.querySelector<HTMLElement>(".history-count")!;

    host.querySelector<HTMLButtonElement>(".history-export")!.addEventListener("click", () => void this.exportLog());
    host.querySelector<HTMLButtonElement>(".history-clear")!.addEventListener("click", () => {
      if (confirm("Clear the processing history for this project? This cannot be undone.")) clearProcessLog();
    });

    this.unsub = subscribeProcessLog((entries) => {
      count.textContent = `${entries.length} operation${entries.length === 1 ? "" : "s"}`;
      this.render(entries);
    });
  }

  private render(entries: ProcessEntry[]): void {
    this.list.innerHTML = "";
    if (entries.length === 0) {
      const empty = document.createElement("div");
      empty.className = "history-empty";
      empty.textContent = "No operations recorded yet. Imports, module runs, equations and edits appear here.";
      this.list.appendChild(empty);
      return;
    }
    // Newest first.
    for (let i = entries.length - 1; i >= 0; i--) {
      const e = entries[i];
      const row = document.createElement("div");
      row.className = "history-row";
      const time = new Date(e.ts);
      const hhmm = time.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
      const day = time.toLocaleDateString();
      row.innerHTML =
        `<span class="history-time" title="${day} ${hhmm}">${hhmm}</span>` +
        `<span class="history-kind history-kind-${e.kind.toLowerCase()}">${e.kind}</span>` +
        `<span class="history-detail"></span>`;
      const detail = row.querySelector<HTMLElement>(".history-detail")!;
      detail.textContent = (e.well ? `${e.well}: ` : "") + e.detail;
      this.list.appendChild(row);
    }
  }

  /** Writes the whole log to a user-picked .txt file (reusing the PNG-writer command,
   *  which just base64-decodes bytes to a path). */
  private async exportLog(): Promise<void> {
    let dest: string | null;
    try {
      dest = await save({
        title: "Export processing history",
        defaultPath: "processing-history.txt",
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
    } catch (err) {
      setStatus(`Save dialog unavailable: ${err}`);
      return;
    }
    if (!dest) return;
    try {
      const base64 = btoa(unescape(encodeURIComponent(processLogToText())));
      const path = await savePng(dest, base64);
      setStatus(`Processing history exported to ${path}`);
    } catch (err) {
      setStatus(`Export failed: ${err}`);
    }
  }

  dispose(): void {
    this.unsub();
  }
}
