import type { WellSummary } from "../ipc";
import { appState, setStatus } from "../state";
import { openModal } from "./modal";

/** The shared refusal for an action that needs a selected well (ROADMAP T-IMP-05).
 *
 *  **A status-bar line is the wrong place to refuse a click.** The user picked "Import SCAL…" and
 *  expected a file dialog; what they got was nothing, with the reason in a corner of the window
 *  nobody was looking at. "Nothing happened" is indistinguishable from a broken button, and the
 *  usual next move is to click it again — which does nothing again.
 *
 *  It is the same family as every other refusal in this app: an undeclared stain, an unimpregnated
 *  plate, a plug with no partner in the depth tolerance. Each of those is refused BY NAME with the
 *  fix stated, and none of them is refused quietly. This is the one place that was still quiet.
 *
 *  One helper rather than a copy per handler, for the `followCore.ts` reason: it is the same
 *  decision, and eight copies is eight places for the wording to drift.
 */
export function requireWell(action: string): WellSummary | null {
  const well = appState.selectedWell.get();
  if (well) return well;

  // The status line still gets it: the message belongs in the history of what was attempted, it
  // just cannot be the only place it appears.
  setStatus(`${action} needs a well — select one in the Wells & Tops pane`);

  const wrap = document.createElement("div");
  const msg = document.createElement("div");
  msg.className = "eq-note";
  msg.textContent =
    `${action} works on one well at a time, and no well is selected. Click a well in the ` +
    `Wells & Tops pane on the left, then try again.`;
  wrap.appendChild(msg);

  const actions = document.createElement("div");
  actions.className = "modal-actions";
  const ok = document.createElement("button");
  ok.type = "button";
  ok.className = "btn btn-accent";
  ok.textContent = "OK";
  actions.appendChild(ok);
  wrap.appendChild(actions);

  const close = openModal(action, wrap, 420);
  ok.addEventListener("click", close);
  return null;
}
