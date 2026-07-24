//! Boot-time notice for a project that would not open.
//!
//! Startup runs before the Tauri window exists, so a failure there cannot use any of the app's
//! normal error paths. With `panic = "abort"` and `windows_subsystem = "windows"` an aborting
//! `run()` produces no window, no dialog and no console — SandiBumi simply does not appear.
//! The backend therefore recovers far enough to open a window and hands the reason over here.

import type { StartupProblem } from "./ipc";

/** Blocks the workspace behind an explanation until acknowledged. Same scrim/box markup as the
 *  crash-recovery dialog, so it inherits that styling and its focus behaviour.
 *
 *  Every value from the backend goes in via `textContent`, never `innerHTML`: the path comes off
 *  disk and the message is a formatted DuckDB error, and neither is ours to trust as markup. */
export function showStartupProblemDialog(p: StartupProblem, onAck: () => void): void {
  const scrim = document.createElement("div");
  scrim.className = "guard-confirm-scrim";
  const box = document.createElement("div");
  box.className = "guard-confirm";

  const head = document.createElement("p");
  head.className = "guard-confirm-title";
  head.textContent = "SandiBumi could not open your project.";

  const which = document.createElement("p");
  which.textContent = `Could not open: ${p.attempted_path}`;

  const why = document.createElement("p");
  why.className = "startup-notice-reason";
  why.textContent = p.message;

  // The reassurance has to be explicit and early. A user who sees an empty well list after a
  // failed open will otherwise assume the project is gone — which is the worst possible
  // misreading, and the one most likely to make them do something destructive to "fix" it.
  const now = document.createElement("p");
  now.textContent = p.recovery_persists
    ? `Your project file has NOT been changed. So the app could start and tell you this, ` +
      `this session is running on an empty temporary project instead ` +
      `(${p.recovered_to}) — nothing you do here will be written back to your project.`
    : `Your project file has NOT been changed. A temporary project could not be created either, ` +
      `so this session is running in memory only: nothing will be saved anywhere.`;

  const fix = document.createElement("p");
  fix.textContent =
    "Most often this means SandiBumi is already open in another window, which holds the " +
    "project file open exclusively. Close that window and start again. Otherwise use " +
    "Project → Open to choose a different project.";

  const row = document.createElement("div");
  row.className = "guard-confirm-row";
  const ok = document.createElement("button");
  ok.type = "button";
  ok.className = "primary";
  ok.textContent = "Continue";
  ok.addEventListener("click", () => {
    scrim.remove();
    onAck();
  });
  row.appendChild(ok);

  box.append(head, which, why, now, fix, row);
  scrim.appendChild(box);
  document.body.appendChild(scrim);
  ok.focus();
}
