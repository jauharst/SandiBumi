import { setStatus } from "./state";
import { initI18n } from "./i18n";
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
  const workspace = new Workspace(dockRoot);
  new Ribbon(ribbonEl, workspace);
  installUndoHotkeys(setStatus);
  // Restore the project's processing history (async; the History panel updates when it lands).
  void loadProcessLog();
});
