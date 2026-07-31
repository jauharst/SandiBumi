import { formRow } from "./modal";

/** The "these depths came from the core report" tick-box, shared by every import that can carry
 *  laboratory depths: point data (XRD, CEC, petrography), SCAL plugs and plates.
 *
 *  All three are measured ON core, so all three carry the depths the core report used. Once that
 *  core has been registered against the log, those depths are stale by exactly however far the
 *  core moved — and the samples get attributed to rock they were never measured on.
 *
 *  **Off by default, and it stays the user's declaration.** Nothing in a delimited file or a
 *  plate's filename reliably says which depth scale it uses, so the app must not guess. What it
 *  can do is say afterwards what happened, which every backend here does through its notes:
 *  samples placed, samples that fell outside the cored interval, a core that was never shifted,
 *  and a well with no core at all.
 */
export interface FollowCoreControl {
  /** Append this to the dialog. */
  el: HTMLElement;
  checked: () => boolean;
}

export function buildFollowCoreRow(what: string, idSuffix: string): FollowCoreControl {
  const box = document.createElement("input");
  box.type = "checkbox";
  box.id = `follow-core-${idSuffix}`;

  const label = document.createElement("label");
  label.htmlFor = box.id;
  label.textContent = " These depths came from the core report";

  const row = document.createElement("div");
  row.appendChild(box);
  row.appendChild(label);

  const wrap = document.createElement("div");
  wrap.appendChild(
    formRow(
      "Follow the core",
      row,
      `Tick if ${what} use the depths the core was delivered at. Each one is then placed where that ` +
        "rock now sits, using the core's own record — including where one barrel moved further than " +
        "another. Leave it off for depths already on the log's scale.",
    ),
  );

  const note = document.createElement("div");
  note.className = "eq-note";
  note.style.display = "none";
  note.textContent =
    "Anything above or below the cored interval has nothing to go on — it keeps the nearest " +
    "correction and is reported as such rather than being placed silently.";
  wrap.appendChild(note);
  box.addEventListener("change", () => {
    note.style.display = box.checked ? "" : "none";
  });

  return { el: wrap, checked: () => box.checked };
}
