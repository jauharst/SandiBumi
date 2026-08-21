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

/** SB-DBM-031: a delivery declares the depth datum its depths are quoted in, ONCE, for
 *  the whole set. MD is preselected as the near-universal case; confirming the wizard is
 *  the declaration — the backend refuses an unknown token by naming the vocabulary. */
export function buildDatumSelect(): HTMLSelectElement {
  const sel = document.createElement("select");
  sel.className = "form-control";
  for (const datum of ["MD", "TVD", "TVDSS", "TVDKB", "TWT", "OWT", "CDEPTH"]) {
    const option = document.createElement("option");
    option.value = datum;
    option.textContent = datum;
    sel.appendChild(option);
  }
  sel.value = "MD";
  return sel;
}

/** The value a depth-unit select carries while nobody has said what unit the file is in.
 *  A caller must refuse to import on it — see [`buildDepthUnitSelect`]. */
export const DEPTH_UNIT_UNSTATED = "?";

/** The one depth-unit control for a delivery whose depths have to land on the project's scale.
 *
 *  Audit finding 8: a delivery declares the depth UNIT its depths are quoted in, beside the datum
 *  and for the same reason — the two together are what place a number on the project's own scale.
 *  One helper rather than one copy per wizard: the labels are the only hard-coded depth-unit
 *  strings the frontend is allowed to carry (the acceptance sweep classifies them as a
 *  unit-picker), and three copies would be three places for that classification to drift.
 *
 *  `initial` is what the FILE said, when a probe could read it off a units row or a depth header.
 *  Pass nothing when the file said nothing, or when there is no probe step to ask.
 *
 *  AUDIT-2026-08-20 finding 46. This used to open on "Same as project" in every case, and a
 *  default that is also a valid answer makes a dialog nobody read indistinguishable from one that
 *  was answered — the import could not tell "I checked, it is metres" from "I clicked Import".
 *  The LAS path refuses outright rather than assume, and the doctrine here is the same one this
 *  repo applies to a survey's inclination, a plate's stain and Normalize's reference pair: a
 *  missing declaration is asked for, never substituted. So when the file states no unit the
 *  select opens UNCHOSEN and the caller refuses until somebody chooses — "Same as project" is
 *  still there, it has just stopped being a thing you can select by not looking. Where the file
 *  DID state a unit there is nothing to ask, and it is pre-selected exactly as before.
 *
 *  What is at stake is a factor of 3.28 applied silently to every depth in the delivery. */
export function buildDepthUnitSelect(initial?: string | null): HTMLSelectElement {
  const sel = document.createElement("select");
  sel.className = "form-control";
  const stated = initial ?? "";
  const choices: [string, string][] = [
    ["", "Same as project"],
    ["m", "Metres (m)"],
    ["ft", "Feet (ft)"],
  ];
  if (!stated) choices.unshift([DEPTH_UNIT_UNSTATED, "— the file does not say — choose"]);
  for (const [value, label] of choices) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    sel.appendChild(option);
  }
  sel.value = stated || DEPTH_UNIT_UNSTATED;
  return sel;
}
