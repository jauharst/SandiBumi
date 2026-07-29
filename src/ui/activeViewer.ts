/** Which viewer pane is the "working pane" while the 📌 pin is OFF.
 *
 *  Pin OFF is the Petrel-style working-pane model: viewers keep the well they are
 *  showing and only the ACTIVE one follows a new well selection, which is how
 *  side-by-side multi-well comparison works.
 *
 *  Asking dockview `panel.api.isActive` at selection time answers the wrong question.
 *  A well is selected by CLICKING IT IN THE WELLS & TOPS TREE, and that click activates
 *  the tree's own pane — so at the instant `selectedWell` fires, no viewer is active and
 *  NOTHING follows the selection at all (Jauhar field review 2026-07-29, T-SHELL-16:
 *  "Pin off, never follow well even for active panel").
 *
 *  What "the active panel" means to the user is the last VIEWER they worked in — the
 *  tree is a browser, not a viewer. Viewers therefore report their activation here and
 *  the follow gate reads this instead of dockview's instantaneous active panel.
 */

let workingId: string | null = null;

/** Called by a viewer pane when it becomes the active dock panel. */
export function markActiveViewer(panelId: string): void {
  workingId = panelId;
}

/** Called when a viewer pane closes, so a dead id can't stay the working pane. */
export function forgetViewer(panelId: string): void {
  if (workingId === panelId) workingId = null;
}

/** True when this pane is the working pane, i.e. the one that follows the selection
 *  while the pin is OFF. With no viewer yet recorded (nothing has ever been activated —
 *  a restored session can arrive that way) the first pane to ask claims the role, so
 *  "pin off" can never mean "no pane follows anything". */
export function isWorkingPane(panelId: string): boolean {
  if (workingId === null) {
    workingId = panelId;
    return true;
  }
  return workingId === panelId;
}

/** The working pane's dock id, or null before any viewer has been active. */
export function workingPaneId(): string | null {
  return workingId;
}
