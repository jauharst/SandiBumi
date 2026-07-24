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
import { saveDocument, startupProblem } from "./ipc";
import { showStartupProblemDialog } from "./startupNotice";
import { applyStoredTheme } from "./theme";
import { Ribbon } from "./ui/ribbon";
import { Workspace } from "./ui/workspace";
import { installUndoHotkeys } from "./undo";
import { loadProcessLog } from "./processLog";

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

  const boot = (mode: "normal" | "restore-autosave" | "safe") => {
    const workspace = new Workspace(dockRoot);
    new Ribbon(ribbonEl, workspace);
    installUndoHotkeys(setStatus);
    // Restore the project's processing history (async; the History panel updates when it lands).
    void loadProcessLog();

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

  // Did the project itself open? This is asked first and answered before anything is built:
  // if the project did not open, which workspace layout to restore is a secondary question, and
  // showing an empty well list before explaining why would read as "my project is gone".
  // The command only clones an Option behind a mutex; if it fails outright we are no worse off
  // than before it existed, so boot normally.
  void startupProblem()
    .catch(() => null)
    .then((problem) => {
      if (problem) showStartupProblemDialog(problem, bootWithWorkspaceChoice);
      else bootWithWorkspaceChoice();
    });
});
