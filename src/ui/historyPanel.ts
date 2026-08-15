import { save } from "@tauri-apps/plugin-dialog";
import { listCurveEditRecords, savePng, type CurveEditRecord } from "../ipc";
import { appState, setStatus } from "../state";
import {
  clearProcessLog,
  subscribeProcessLog,
  type ProcessEntry,
} from "../processLog";

interface DisplayEntry {
  ts: number;
  kind: string;
  detail: string;
  well: string | null;
  durable: boolean;
}

/** The Processing History panel: a live, timestamped audit of everything done in the
 *  project (imports, module runs, equations, edits, exports). It persists with the
 *  project and can be exported to a text file so a study's processing is documented. */
export class HistoryPanel {
  private list: HTMLElement;
  private count: HTMLElement;
  private activity: ProcessEntry[] = [];
  private curveEdits: CurveEditRecord[] = [];
  private curveEditError: string | null = null;
  private loadToken = 0;
  private unsubProcess: () => void;
  private unsubData: () => void;

  constructor(host: HTMLElement) {
    host.classList.add("history-panel");
    host.innerHTML = `
      <div class="history-toolbar">
        <span class="history-count"></span>
        <span class="history-spacer"></span>
        <button class="plot-export-btn history-export">⭳ Export…</button>
        <button class="plot-export-btn history-clear">🗑 Clear activity</button>
      </div>
      <div class="history-list"></div>`;
    this.list = host.querySelector<HTMLElement>(".history-list")!;
    this.count = host.querySelector<HTMLElement>(".history-count")!;

    host.querySelector<HTMLButtonElement>(".history-export")!.addEventListener("click", () => void this.exportLog());
    host.querySelector<HTMLButtonElement>(".history-clear")!.addEventListener("click", () => {
      if (confirm("Clear the general activity log? Durable curve-edit provenance remains in the project.")) {
        clearProcessLog();
      }
    });

    this.unsubProcess = subscribeProcessLog((entries) => {
      this.activity = entries;
      this.render();
    });
    this.unsubData = appState.dataVersion.subscribe(() => void this.reloadCurveEdits());
  }

  private async reloadCurveEdits(): Promise<void> {
    const token = ++this.loadToken;
    try {
      const records = await listCurveEditRecords();
      if (token !== this.loadToken) return;
      this.curveEdits = records;
      this.curveEditError = null;
    } catch (err) {
      if (token !== this.loadToken) return;
      this.curveEdits = [];
      this.curveEditError = String(err);
    }
    this.render();
  }

  private entries(): DisplayEntry[] {
    const activity = this.activity.map<DisplayEntry>((entry) => ({
      ...entry,
      durable: false,
    }));
    const edits = this.curveEdits.map<DisplayEntry>((record) => {
      const interval = record.interval.kind === "WHOLE_CURVE"
        ? "whole curve"
        : `${record.interval.top}–${record.interval.bottom} inclusive`;
      const parameters = Object.entries(record.parameters)
        .map(([name, value]) => `${name}=${String(value)}`)
        .join(", ");
      const custody = record.actor ? `; actor ${record.actor}` : "";
      const source = record.source_note ? `; source ${record.source_note}` : "";
      return {
        ts: record.timestamp_utc_ms,
        kind: "Edit",
        detail: `${record.curve}: ${record.operation}; ${interval}${parameters ? `; ${parameters}` : ""}; ${record.store} store${custody}${source}`,
        well: record.well_name,
        durable: true,
      };
    });
    return [...activity, ...edits].sort((left, right) => left.ts - right.ts);
  }

  private render(): void {
    const entries = this.entries();
    this.count.textContent = `${entries.length} record${entries.length === 1 ? "" : "s"} · ${this.curveEdits.length} durable curve edit${this.curveEdits.length === 1 ? "" : "s"}`;
    this.list.innerHTML = "";
    if (this.curveEditError) {
      const error = document.createElement("div");
      error.className = "history-empty";
      error.textContent = `Durable curve-edit provenance could not be read: ${this.curveEditError}`;
      this.list.appendChild(error);
    }
    if (entries.length === 0 && !this.curveEditError) {
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
      if (e.durable) {
        row.classList.add("history-row-durable");
        row.title = "Persistent curve-owned provenance; clearing the activity log does not remove it.";
      }
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
      const entries = this.entries();
      const lines = entries.map((entry) => {
        const time = new Date(entry.ts).toISOString();
        const durable = entry.durable ? " [DURABLE CURVE EDIT]" : "";
        return `${time}  [${entry.kind}]${durable}${entry.well ? ` ${entry.well}:` : ""} ${entry.detail}`;
      });
      const text = `SandiBumi processing history (${entries.length} records)\n${lines.join("\n")}\n`;
      const base64 = btoa(unescape(encodeURIComponent(text)));
      const path = await savePng(dest, base64);
      setStatus(`Processing history exported to ${path}`);
    } catch (err) {
      setStatus(`Export failed: ${err}`);
    }
  }

  dispose(): void {
    this.unsubProcess();
    this.unsubData();
  }
}
