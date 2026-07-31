import { setStatus } from "./state";
import { initI18n } from "./i18n";
import { installInteractionGuards } from "./interactionGuard";
import {
  detectAbnormalExit,
  installAutosave,
  markSessionRunning,
  readAutosave,
  showCrashRecoveryDialog,
} from "./autosave";
import { awaitProjectOpen, bootReport, saveDocument } from "./ipc";
import { recordProcess } from "./processLog";
import { showBootOverlay } from "./bootOverlay";
import { showStartupProblemDialog } from "./startupNotice";
import { applyStoredTheme } from "./theme";
import { Ribbon } from "./ui/ribbon";
import { Workspace } from "./ui/workspace";
import { installUndoHotkeys } from "./undo";
import { loadProcessLog } from "./processLog";
import { syncDepthUnits } from "./depthUnitPref";

// Apply the stored theme before first paint to avoid a flash of the wrong palette.
applyStoredTheme();

window.addEventListener("DOMContentLoaded", () => {
  const dockRoot = document.querySelector<HTMLElement>("#dock-root");
  const ribbonEl = document.querySelector<HTMLElement>("#ribbon");
  if (!dockRoot || !ribbonEl) {
    console.error("App shell markup missing");
    return;
  }

  // i18n first: its observer then covers everything the workspace/ribbon build.
  initI18n();
  // Interaction safety (right-click/reload lockdown, double-click-to-edit) before any
  // panel exists, so no early control escapes the guards.
  installInteractionGuards();

  // Crash detection must read the flag BEFORE this session plants its own.
  const crashed = detectAbnormalExit();
  const autosave = readAutosave();
  markSessionRunning();

  /** Notices the boot overlay drained while the database was still opening. They can only be
   *  written to the processing history (which lives IN the project) once it is open. */
  let pendingBootNotes: string[] = [];

  const boot = (mode: "normal" | "restore-autosave" | "safe") => {
    const workspace = new Workspace(dockRoot);
    new Ribbon(ribbonEl, workspace);
    installUndoHotkeys(setStatus);
    // Which unit this project's depths are stored in, and which to show them in. Async:
    // panels open on the metre default and re-render when it lands, which is correct for
    // a metric project and a one-frame correction for a foot one.
    void syncDepthUnits();
    // Restore the project's processing history, then append anything noteworthy the
    // backend did while opening (one-time migration backups, the memory cap, a slow open
    // explained) — invisible in a built exe otherwise. The last notice also lands in the
    // status line so a "why did that take 15 minutes" has an answer on screen, not just
    // in the History panel. Sequenced after the load so the notes append to, rather than
    // race, the restored history.
    void loadProcessLog().then(() =>
      bootReport()
        .then((late) => {
          // What the overlay already drained, plus anything queued since.
          const notes = [...pendingBootNotes, ...late];
          pendingBootNotes = [];
          for (const n of notes) recordProcess("Project", n);
          const visible = notes.filter((n) => !n.startsWith("DuckDB memory"));
          if (visible.length > 0) setStatus(visible[visible.length - 1]);
        })
        .catch(() => {}),
    );

    if (mode === "restore-autosave" && autosave) {
      workspace.applySession(autosave);
      setStatus("Workspace restored from the crash autosave");
    } else if (mode === "safe") {
      workspace.resetWorkspace();
      // Nothing silently lost: stash the autosaved workspace as a reopenable session.
      if (autosave) {
        const stamp = new Date().toISOString().slice(0, 16).replace("T", " ");
        void saveDocument("session", `Recovered ${stamp}`, JSON.stringify(autosave))
          .then(() => setStatus(`Safe Mode — previous workspace kept as session "Recovered ${stamp}"`))
          .catch(() => setStatus("Safe Mode — default workspace"));
      } else {
        setStatus("Safe Mode — default workspace");
      }
    } else if (autosave) {
      // Normal launch: the dock layout came back via the workspace's own restore; the
      // autosave adds the parts that restore can't carry (well, log-view layouts).
      workspace.applyAutosaveExtras(autosave);
    }
    installAutosave(workspace);
  };

  const bootWithWorkspaceChoice = () => {
    if (crashed && autosave) {
      showCrashRecoveryDialog((choice) => boot(choice === "restore" ? "restore-autosave" : "safe"));
    } else {
      boot("normal");
    }
  };

  // THE GATE. The window now exists before the project database does — the backend opens it on
  // a background thread — so nothing may be built and no command may be issued until this
  // resolves; until then the live connection is an empty in-memory placeholder and every query
  // would truthfully answer "no wells". The overlay covers the wait and reports what the
  // backend is doing (a first open after an update runs one-time storage upgrades).
  //
  // The answer also carries whether the project opened AT ALL: if it did not, which workspace
  // layout to restore is a secondary question, and showing an empty well list before explaining
  // why would read as "my project is gone".
  const overlay = showBootOverlay();
  void awaitProjectOpen()
    .catch(() => null)
    .then((outcome) => {
      overlay.finish();
      pendingBootNotes = overlay.notes;
      // A long open, explained after the fact rather than left as a mystery.
      if (outcome && outcome.elapsed_secs >= 10) {
        pendingBootNotes.push(
          `Opening this project took ${outcome.elapsed_secs}s (one-time storage upgrades run on the first open after an update)`,
        );
      }
      const problem = outcome?.problem ?? null;
      if (problem) showStartupProblemDialog(problem, bootWithWorkspaceChoice);
      else bootWithWorkspaceChoice();
    });
});
