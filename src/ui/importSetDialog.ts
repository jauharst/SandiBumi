import { formRow, openModal } from "./modal";
import type { LasImportOptions } from "../ipc";

/** The Import LAS "which set?" dialog (T-IMP-02 — the Geolog/IP set model).
 *
 *  A delivery folder is one SET: `blso00025_lapi2023_fprooh.las` and its 543 siblings are
 *  the FPROOH interpretation of the field, and a well's RAW, FPROOH and MULTIMIN curves
 *  belong on ONE well record, not on three same-named ones. This dialog names the set and
 *  decides whether same-named files attach to the existing well.
 *
 *  Non-blocking, per the app's dialog convention (modal.ts): the scrim is
 *  pointer-transparent and only Esc / ✕ / the buttons close it.
 */

/** Filename tokens that carry no set meaning — they appear in every file of a delivery. */
const NOISE_TOKENS = new Set([
  "las", "log", "logs", "final", "data", "well", "wells", "copy", "new", "old", "edit",
]);

/**
 * Derives a set-name suggestion from what the picked filenames have in COMMON.
 *
 * Files are split on `_`, `-`, `.` and space; a token is a candidate only if it appears in
 * EVERY file (so it describes the delivery, not one well), is not purely numeric or a year
 * (well numbers and `lapi2023` dates differ per delivery but say nothing about content),
 * and is not generic noise.
 *
 * A candidate at POSITION 0 is rejected: vendor names run `<well>_<project>_<product>`, so
 * a leading token shared by every file is the well or field prefix, not the set —
 * `SANDI-01/02/03` would otherwise suggest "SANDI" for what is plainly a raw log delivery.
 * Among the rest the LAST wins, which is the product suffix (`fprooh`, `multimin`, `ssc`).
 * Returns "" when nothing survives, and the caller falls back to RAW.
 */
export function suggestSetName(paths: string[]): string {
  if (paths.length === 0) return "";
  // Positions come from the UNFILTERED split, so "position 0" means "first token of the
  // filename" even when earlier tokens were dropped as noise.
  const tokensOf = (p: string): { token: string; index: number }[] => {
    const base = p.replace(/\\/g, "/").split("/").pop() ?? p;
    return base
      .replace(/\.[^.]*$/, "") // strip extension
      .split(/[_\-.\s]+/)
      .map((t, index) => ({ token: t.trim().toUpperCase(), index }))
      .filter(
        ({ token: t }) =>
          t.length >= 2 &&
          !NOISE_TOKENS.has(t.toLowerCase()) &&
          !/^\d+$/.test(t) && // a bare well number
          !/^\d{4}$/.test(t) && // a bare year
          !/^[A-Z]*\d{3,}[A-Z\d]*$/.test(t), // well ids like BLSO00025 / 00358D1
      );
  };
  const first = tokensOf(paths[0]);
  if (first.length === 0) return "";
  const rest = paths.slice(1).map((p) => new Set(tokensOf(p).map((t) => t.token)));
  const common = first.filter(({ token, index }) => index > 0 && rest.every((s) => s.has(token)));
  if (common.length === 0) return "";
  return common[common.length - 1].token;
}

export interface ImportSetChoice extends LasImportOptions {
  setName: string;
  attach: boolean;
}

/**
 * Asks for the set name + attach behaviour. Resolves with the choice, or null if the user
 * cancels (Esc / ✕ / Cancel) — the caller must treat null as "import nothing".
 */
export function openImportSetDialog(paths: string[]): Promise<ImportSetChoice | null> {
  return new Promise((resolve) => {
    const wrap = document.createElement("div");

    const summary = document.createElement("p");
    summary.className = "form-hint";
    const names = paths.slice(0, 3).map((p) => (p.replace(/\\/g, "/").split("/").pop() ?? p));
    summary.textContent =
      `${paths.length} file(s): ${names.join(", ")}${paths.length > 3 ? `, +${paths.length - 3} more` : ""}`;
    wrap.appendChild(summary);

    const setInput = document.createElement("input");
    setInput.type = "text";
    setInput.className = "form-control";
    setInput.value = suggestSetName(paths);
    setInput.placeholder = "RAW";
    setInput.spellcheck = false;
    wrap.appendChild(
      formRow(
        "Set name",
        setInput,
        "One delivery = one set. Curves land under this name so you can tell this run's PHIE from another's. Blank = RAW.",
      ),
    );

    const setHint = document.createElement("p");
    setHint.className = "form-hint";
    setHint.textContent =
      "Upper-cased; spaces become underscores. If a well already has a set with this name, " +
      "the new one is suffixed (FPROOH → FPROOH_1) — an import never overwrites an earlier delivery.";
    wrap.appendChild(setHint);

    const attachBox = document.createElement("input");
    attachBox.type = "checkbox";
    attachBox.className = "form-check";
    attachBox.checked = true;
    wrap.appendChild(
      formRow(
        "Attach to existing wells",
        attachBox,
        "Match by well name: a re-delivery of a well already in the project becomes a new set on THAT well.",
      ),
    );

    const attachHint = document.createElement("p");
    attachHint.className = "form-hint";
    attachHint.textContent =
      "On (recommended): re-importing the same field under a new set name keeps one record per well. " +
      "Off: every file creates its own well record, even when the name already exists. " +
      "A name matching several existing wells is always ambiguous — those import as separate records and say so.";
    wrap.appendChild(attachHint);

    const actions = document.createElement("div");
    actions.className = "form-actions";
    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn";
    cancelBtn.textContent = "Cancel";
    const okBtn = document.createElement("button");
    okBtn.className = "btn btn-accent";
    okBtn.textContent = "Import";
    actions.append(cancelBtn, okBtn);
    wrap.appendChild(actions);

    // `settled` guards the single-resolve contract: close() runs on Esc/✕ too, and without
    // it a user who clicks Import and then presses Escape would resolve the promise twice
    // (the second time as a cancel), silently discarding a running import's choice.
    let settled = false;
    const finish = (choice: ImportSetChoice | null) => {
      if (settled) return;
      settled = true;
      close();
      resolve(choice);
    };

    const close = openModal("Import LAS — curve set", wrap, 560);
    cancelBtn.addEventListener("click", () => finish(null));
    okBtn.addEventListener("click", () =>
      finish({ setName: setInput.value.trim(), attach: attachBox.checked }),
    );
    setInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") okBtn.click();
    });
    // The dialog can also be dismissed by Esc/✕ inside openModal, which does NOT call
    // finish — so watch for the dialog leaving the DOM and resolve as a cancel.
    const root = document.querySelector<HTMLElement>("#modal-root");
    if (root) {
      const observer = new MutationObserver(() => {
        if (!wrap.isConnected) {
          observer.disconnect();
          finish(null);
        }
      });
      observer.observe(root, { childList: true });
    }
    setInput.focus();
    setInput.select();
  });
}
