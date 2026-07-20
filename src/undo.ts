/** Global undo/redo stack (Ctrl+Z / Ctrl+Y) for UI and data edits: database-inspector
 *  cell edits, layout property changes, and similar reversible actions. Module runs are
 *  intentionally NOT undoable — they're deterministic and re-runnable. */

export interface UndoableAction {
  /** Short human label for the status bar ("edit GR @ 1502.5m"). */
  label: string;
  undo: () => void | Promise<void>;
  redo: () => void | Promise<void>;
}

const LIMIT = 100;
const undoStack: UndoableAction[] = [];
const redoStack: UndoableAction[] = [];

/** Notified whenever either stack changes, so the quick-access toolbar can enable/disable
 *  its Undo/Redo buttons and show the next action's label. */
const changeListeners = new Set<() => void>();
function notifyChange(): void {
  for (const fn of changeListeners) fn();
}

/** Subscribe to stack changes; fires once immediately. Returns unsubscribe. */
export function onUndoChange(fn: () => void): () => void {
  changeListeners.add(fn);
  fn();
  return () => changeListeners.delete(fn);
}

/** Empties both stacks. A project switch invalidates every recorded action — replaying
 *  one would mutate the newly opened database with the old project's values. */
export function clearUndoStacks(): void {
  undoStack.length = 0;
  redoStack.length = 0;
  notifyChange();
}

/** Records a just-performed action so Ctrl+Z can reverse it. Clears the redo branch. */
export function pushUndo(action: UndoableAction): void {
  undoStack.push(action);
  if (undoStack.length > LIMIT) undoStack.shift();
  redoStack.length = 0;
  notifyChange();
}

export async function undo(): Promise<string | null> {
  const action = undoStack.pop();
  if (!action) return null;
  await action.undo();
  redoStack.push(action);
  notifyChange();
  return action.label;
}

export async function redo(): Promise<string | null> {
  const action = redoStack.pop();
  if (!action) return null;
  await action.redo();
  undoStack.push(action);
  notifyChange();
  return action.label;
}

export function undoDepth(): number {
  return undoStack.length;
}

export function redoDepth(): number {
  return redoStack.length;
}

/** Label of the action Ctrl+Z would reverse next (for tooltips); null if none. */
export function nextUndoLabel(): string | null {
  return undoStack[undoStack.length - 1]?.label ?? null;
}

export function nextRedoLabel(): string | null {
  return redoStack[redoStack.length - 1]?.label ?? null;
}

/** True when the event target has its own undo behavior (text inputs, CodeMirror). */
function targetHandlesUndo(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el || !el.closest) return false;
  return !!el.closest("input, textarea, select, [contenteditable], .cm-editor");
}

/** Installs the global Ctrl+Z / Ctrl+Y (or Ctrl+Shift+Z) handlers. */
export function installUndoHotkeys(setStatus: (text: string) => void): void {
  document.addEventListener("keydown", (e) => {
    if (!e.ctrlKey || e.altKey) return;
    if (targetHandlesUndo(e.target)) return;
    const key = e.key.toLowerCase();
    if (key === "z" && !e.shiftKey) {
      e.preventDefault();
      void undo().then((label) => setStatus(label ? `Undo: ${label}` : "Nothing to undo"));
    } else if (key === "y" || (key === "z" && e.shiftKey)) {
      e.preventDefault();
      void redo().then((label) => setStatus(label ? `Redo: ${label}` : "Nothing to redo"));
    }
  });
}
