//! Told once, when a project that came from elsewhere carries code.
//!
//! A LAS file is inert: the worst it can do is give you wrong numbers, and a QC catches that. A
//! project file is not. Alongside the curves it carries **saved equations and saved ML models**,
//! and both are instructions rather than numbers — a model is a joblib pickle, which runs code the
//! moment it is loaded, before any of the checks around it. So a project that arrived from
//! somebody else is an attachment, not a data file.
//!
//! `docs/SECURITY-REVIEW-2026-08-22.md` finding F1. Jauhar's call, put to him as a choice: **warn
//! once, then let it run** — the yellow-bar behaviour every office suite uses, not a gate in front
//! of a daily action. Nothing is blocked and nothing is refused; a person who has been told can
//! decide for themselves.
//!
//! Shown at most once per project file, and never for a project created on this machine.

import { trustProjectCode, type ProjectCodeNotice } from "../ipc";

/** "2 saved equations and 1 saved model" — the count is the point, so it is never rounded away. */
function whatItCarries(notice: ProjectCodeNotice): string {
  const parts: string[] = [];
  if (notice.equations > 0) {
    parts.push(`${notice.equations} saved ${notice.equations === 1 ? "equation" : "equations"}`);
  }
  if (notice.models > 0) {
    parts.push(`${notice.models} saved ${notice.models === 1 ? "model" : "models"}`);
  }
  return parts.join(" and ");
}

/** Shows the notice and records that it has been shown. Resolves once acknowledged.
 *
 *  Same `guard-confirm` markup as `startupNotice.ts` and the crash-recovery dialog, so it inherits
 *  their styling and focus behaviour rather than introducing a third look for the same job.
 *
 *  Every value from the backend goes in through `textContent`, never `innerHTML` — the project
 *  name comes off disk and is not ours to trust as markup. */
export function showProjectCodeNotice(notice: ProjectCodeNotice): Promise<void> {
  return new Promise((resolve) => {
    const scrim = document.createElement("div");
    scrim.className = "guard-confirm-scrim";
    const box = document.createElement("div");
    box.className = "guard-confirm";

    const head = document.createElement("p");
    head.className = "guard-confirm-title";
    head.textContent = "This project was made somewhere else, and it contains code.";

    const what = document.createElement("p");
    what.textContent =
      `"${notice.name}" carries ${whatItCarries(notice)}. Those are instructions, not just ` +
      `numbers: running a saved equation, or applying a saved model, runs that code on this ` +
      `computer with your access to your data.`;

    // The comparison a petrophysicist already has the instinct for. A LAS can only be wrong; this
    // can be something else, and the difference is not obvious from the Wells pane.
    const why = document.createElement("p");
    why.textContent =
      "A LAS file cannot do this — the worst it can do is give you wrong numbers, and you would " +
      "find that in QC. A project file is closer to a spreadsheet with a macro in it.";

    const fix = document.createElement("p");
    fix.textContent =
      "Nothing has run, and nothing is blocked. If you know where this project came from, carry " +
      "on as normal. If you do not, look at what is in the Equation Editor and the Saved models " +
      "list before you run either. You will not be told again for this project.";

    const row = document.createElement("div");
    row.className = "guard-confirm-row";
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "primary";
    ok.textContent = "Continue";
    ok.addEventListener("click", () => {
      scrim.remove();
      // Recorded on acknowledgement, not on display: a notice dismissed by a crash is a notice
      // the user never read, and it should come back.
      void trustProjectCode().catch(() => {});
      resolve();
    });
    row.appendChild(ok);

    box.append(head, what, why, fix, row);
    scrim.appendChild(box);
    document.body.appendChild(scrim);
    ok.focus();
  });
}
