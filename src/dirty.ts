/** Unsaved-state registry (Jauhar field review 2026-07-19, P1): panels register here
 *  when the user changes state that isn't captured in a named save yet (log-view layout
 *  edits, workspace arrangement). The workspace paints a ● on dirty panel tabs and the
 *  QAT "Save Session" button shows a dot while anything is unsaved. Cleared by Save
 *  Layout (that panel), Save/Open Session (everything), and workspace resets.
 *
 *  This tracks *named-save* freshness only — the rolling crash autosave (autosave.ts)
 *  runs regardless, so "dirty" never means "at risk", just "not yet in a named save". */

/** Pseudo-panel id for workspace-level state (pane arrangement, splits, floats). */
export const WORKSPACE_DIRTY = "__workspace__";

const dirty = new Set<string>();
const listeners = new Set<() => void>();

function notify(): void {
  for (const fn of listeners) fn();
}

export function markDirty(id: string): void {
  if (dirty.has(id)) return;
  dirty.add(id);
  notify();
}

/** Without an id, clears everything (a full session save/restore). */
export function clearDirty(id?: string): void {
  if (id === undefined) {
    if (dirty.size === 0) return;
    dirty.clear();
  } else if (!dirty.delete(id)) {
    return;
  }
  notify();
}

export function isDirty(id: string): boolean {
  return dirty.has(id);
}

export function anyDirty(): boolean {
  return dirty.size > 0;
}

/** Subscribes and fires immediately. Returns unsubscribe. */
export function subscribeDirty(fn: () => void): () => void {
  listeners.add(fn);
  fn();
  return () => listeners.delete(fn);
}
