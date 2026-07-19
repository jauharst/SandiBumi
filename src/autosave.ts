import type { SessionSnapshot, Workspace } from "./ui/workspace";

/** Crash resilience (Jauhar field review 2026-07-19, P1):
 *
 *  - While the app runs, a `running` flag sits in localStorage; a clean exit
 *    (pagehide/beforeunload — window close, guarded reload) removes it. If the flag is
 *    still there at the next launch, the previous session died abnormally (crash,
 *    force-kill, power loss).
 *  - Every 10 s (plus on tab-hide and on exit) the full workspace snapshot — dock
 *    layout, active well, every log view's layout — is autosaved to localStorage.
 *  - After an abnormal exit the user chooses: restore the autosaved workspace, or
 *    start in Safe Mode (default layout, nothing restored; the autosave is stashed as
 *    a "Recovered …" session so nothing is silently lost).
 *  - On a NORMAL launch the autosave still improves restore: the dock layout comes
 *    back via the workspace's own localStorage restore, and applyAutosaveExtras()
 *    re-applies the well + log-view layouts that dockview's JSON doesn't carry.
 */

const RUNNING_KEY = "sandibumi.running";
const AUTOSAVE_KEY = "sandibumi.autosave";
const AUTOSAVE_INTERVAL_MS = 10_000;

/** True when the previous session did not exit cleanly. Read BEFORE markSessionRunning. */
export function detectAbnormalExit(): boolean {
  return localStorage.getItem(RUNNING_KEY) !== null;
}

/** Plants the running flag and arranges for a clean exit to remove it. */
export function markSessionRunning(): void {
  localStorage.setItem(RUNNING_KEY, new Date().toISOString());
  const clear = () => localStorage.removeItem(RUNNING_KEY);
  window.addEventListener("pagehide", clear);
  window.addEventListener("beforeunload", clear);
}

export function readAutosave(): SessionSnapshot | null {
  try {
    const raw = localStorage.getItem(AUTOSAVE_KEY);
    return raw ? (JSON.parse(raw) as SessionSnapshot) : null;
  } catch {
    return null;
  }
}

/** Starts the rolling autosave: every 10 s, on tab-hide, and on exit. */
export function installAutosave(workspace: Workspace): void {
  const save = () => {
    try {
      localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(workspace.snapshotSession()));
    } catch {
      /* quota/serialization issues must never break the app */
    }
  };
  window.setInterval(save, AUTOSAVE_INTERVAL_MS);
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") save();
  });
  window.addEventListener("pagehide", save);
  save();
}

/** Blocking choice dialog shown when the previous session crashed. Reuses the
 *  guard-confirm styling from interactionGuard (same "must be deliberate" contract). */
export function showCrashRecoveryDialog(onChoice: (choice: "restore" | "safe") => void): void {
  const scrim = document.createElement("div");
  scrim.className = "guard-confirm-scrim";
  const box = document.createElement("div");
  box.className = "guard-confirm";
  const head = document.createElement("p");
  head.className = "guard-confirm-title";
  head.textContent = "SandiBumi did not close properly last time.";
  const msg = document.createElement("p");
  msg.textContent =
    "You can restore the autosaved workspace (panes, wells and log views as they were " +
    "moments before the exit), or start in Safe Mode with the default layout — the " +
    "autosaved workspace is then kept as a “Recovered” session.";
  const row = document.createElement("div");
  row.className = "guard-confirm-row";
  const safe = document.createElement("button");
  safe.type = "button";
  safe.textContent = "Start in Safe Mode";
  const restore = document.createElement("button");
  restore.type = "button";
  restore.className = "primary";
  restore.textContent = "Restore autosaved workspace";
  const pick = (choice: "restore" | "safe") => {
    scrim.remove();
    onChoice(choice);
  };
  safe.addEventListener("click", () => pick("safe"));
  restore.addEventListener("click", () => pick("restore"));
  row.append(safe, restore);
  box.append(head, msg, row);
  scrim.appendChild(box);
  document.body.appendChild(scrim);
  restore.focus();
}
