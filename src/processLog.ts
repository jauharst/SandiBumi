import { listDocuments, saveDocument } from "./ipc";

/** Processing history — a timestamped audit trail of every meaningful operation the user
 *  performs (imports, module runs, equation runs, edits, shifts). Unlike undo (which
 *  reverses UI/data edits), this is a permanent record: it is persisted with the project
 *  and can be reviewed in the History panel or exported as text. A 2000-well study needs
 *  to know what was done, when, and to which well. */

export interface ProcessEntry {
  /** Epoch milliseconds. */
  ts: number;
  /** Short category ("Import", "Module", "Equation", "Edit", "Summary", "Export"…). */
  kind: string;
  /** What happened, human readable. */
  detail: string;
  /** Well it applied to, when it is well-scoped (null for field-wide/batch actions). */
  well: string | null;
}

const DOC_TYPE = "history";
const DOC_NAME = "log";
const LIMIT = 5000; // keep the audit bounded; oldest entries roll off

let entries: ProcessEntry[] = [];
const listeners = new Set<(entries: ProcessEntry[]) => void>();
let saveHandle: number | undefined;

function notify(): void {
  for (const fn of listeners) fn(entries);
}

/** Debounced persistence to the project's `documents` table so the log survives restarts
 *  and travels with "Save Project As". */
function scheduleSave(): void {
  if (saveHandle !== undefined) window.clearTimeout(saveHandle);
  saveHandle = window.setTimeout(() => {
    void saveDocument(DOC_TYPE, DOC_NAME, JSON.stringify(entries)).catch(() => {});
  }, 600);
}

/** Records one operation. Call it right after the operation succeeds. */
export function recordProcess(kind: string, detail: string, well: string | null = null): void {
  entries.push({ ts: Date.now(), kind, detail, well });
  if (entries.length > LIMIT) entries = entries.slice(entries.length - LIMIT);
  notify();
  scheduleSave();
}

export function getProcessLog(): ProcessEntry[] {
  return entries;
}

/** Subscribe to log changes; fires once immediately with the current entries. */
export function subscribeProcessLog(fn: (entries: ProcessEntry[]) => void): () => void {
  listeners.add(fn);
  fn(entries);
  return () => listeners.delete(fn);
}

export function clearProcessLog(): void {
  entries = [];
  notify();
  scheduleSave();
}

/** Loads the persisted history once at startup (no-op if there is no backend/document). */
export async function loadProcessLog(): Promise<void> {
  try {
    const docs = await listDocuments(DOC_TYPE);
    const doc = docs.find((d) => d.name === DOC_NAME);
    if (doc) {
      const parsed = JSON.parse(doc.json) as ProcessEntry[];
      if (Array.isArray(parsed)) {
        entries = parsed;
        notify();
      }
    }
  } catch {
    /* first run / no backend — start empty */
  }
}

/** Plain-text rendering of the whole log for export/copy. */
export function processLogToText(): string {
  // SB-DBM-009 / DEC-022: the exported record keeps UTC but SAYS so - a zone-less
  // timestamp reads as local on whatever machine opens the file.
  const iso = (ts: number) => new Date(ts).toISOString().replace("T", " ").slice(0, 19) + " UTC";
  const lines = entries.map(
    (e) => `${iso(e.ts)}  [${e.kind}]${e.well ? ` ${e.well}:` : ""} ${e.detail}`,
  );
  return `SandiBumi processing history (${entries.length} entries)\n` + lines.join("\n") + "\n";
}
