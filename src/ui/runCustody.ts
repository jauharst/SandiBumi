import type { AncestryActorKind, RunCustody } from "../ipc";
import { formRow, openModal } from "./modal";

const SESSION_OPERATOR_KEY = "sandibumi.sessionOperator";
const SESSION_ACTOR_KIND_KEY = "sandibumi.sessionActorKind";

export interface RunCustodyControls {
  rows: HTMLElement[];
  collect: () => RunCustody;
}

/** Explicit run custody shared by every computation pane.
 *
 * The identity is remembered only for this browser session and prefilled on the next run. It is
 * deliberately separate from report authorship and is never inferred from the Windows account.
 * The source/reference note belongs to this run and is not retained as a default: carrying one
 * study's source into another would be worse than asking again.
 */
export function buildRunCustodyControls(): RunCustodyControls {
  const kind = document.createElement("select");
  kind.className = "form-control";
  for (const [value, label] of [
    ["HUMAN", "Human operator"],
    ["AUTOMATED", "Declared automated actor"],
  ] as const) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    kind.appendChild(option);
  }
  const rememberedKind = sessionStorage.getItem(SESSION_ACTOR_KIND_KEY);
  if (rememberedKind === "HUMAN" || rememberedKind === "AUTOMATED") kind.value = rememberedKind;

  const identity = document.createElement("input");
  identity.className = "form-control";
  identity.placeholder = "explicit session identity";
  identity.value = sessionStorage.getItem(SESSION_OPERATOR_KEY) ?? "";

  const source = document.createElement("textarea");
  source.className = "form-control";
  source.rows = 2;
  source.placeholder = "Cited method, study/calibration record, plot provenance, or other exact reference";

  const collect = (): RunCustody => {
    const actorIdentity = identity.value.trim();
    const sourceNote = source.value.trim();
    if (!actorIdentity) {
      identity.focus();
      throw new Error("Enter the session operator identity before computing.");
    }
    if (!sourceNote) {
      source.focus();
      throw new Error("Enter the source/reference covering this run's explicit values.");
    }
    const actorKind = kind.value as AncestryActorKind;
    sessionStorage.setItem(SESSION_OPERATOR_KEY, actorIdentity);
    sessionStorage.setItem(SESSION_ACTOR_KIND_KEY, actorKind);
    return {
      actor: { kind: actorKind, identity: actorIdentity },
      source_note: sourceNote,
    };
  };

  return {
    rows: [
      formRow("Actor kind", kind, "Human or a stable declared automated identity; never inferred from Windows."),
      formRow("Session operator", identity, "Recorded on every computed curve; separate from report Prepared by."),
      formRow(
        "Run source / reference",
        source,
        "Must identify the authority for explicit values and zone definitions used by this run.",
      ),
    ],
    collect,
  };
}

/** Requests one run's explicit custody without using browser prompts. The operator identity may be
 * remembered for this app session; the source/reference never is. Closing or cancelling resolves
 * `null`, so a write action cannot continue behind a dismissed dialog. */
/** SB-DBM-011: the session operator for AUDITED edit surfaces (zone parameters, curve
 *  identity). Reuses the identity/kind already entered for a run this session; prompts a
 *  minimal dialog once when none exists yet. Returns null when dismissed - the edit does
 *  not proceed, because DEC-020 forbids inferring an operator. */
export function ensureSessionOperator(
  action: string,
): Promise<{ identity: string; kind: AncestryActorKind } | null> {
  const stored = sessionStorage.getItem(SESSION_OPERATOR_KEY)?.trim();
  const kind = (sessionStorage.getItem(SESSION_ACTOR_KIND_KEY) as AncestryActorKind) || "HUMAN";
  if (stored) return Promise.resolve({ identity: stored, kind });
  return requestRunCustody(action).then((custody) =>
    custody ? { identity: custody.actor.identity, kind: custody.actor.kind } : null,
  );
}

export function requestRunCustody(action: string): Promise<RunCustody | null> {
  return new Promise((resolve) => {
    const content = document.createElement("div");
    const intro = document.createElement("p");
    intro.className = "modal-hint";
    intro.textContent =
      "These details travel with every output curve and into its deliverables. Report authorship and the Windows account are not used as substitutes.";
    content.appendChild(intro);
    const controls = buildRunCustodyControls();
    content.append(...controls.rows);

    const error = document.createElement("p");
    error.className = "modal-hint";
    error.style.color = "var(--danger)";
    error.hidden = true;
    content.appendChild(error);

    const actions = document.createElement("div");
    actions.className = "modal-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "btn";
    cancel.textContent = "Cancel";
    const proceed = document.createElement("button");
    proceed.type = "button";
    proceed.className = "btn btn-accent";
    proceed.textContent = action;
    actions.append(cancel, proceed);
    content.appendChild(actions);

    let settled = false;
    const finish = (value: RunCustody | null) => {
      if (settled) return;
      settled = true;
      resolve(value);
    };
    let close = () => {};
    close = openModal(`Run custody — ${action}`, content, 560, () => finish(null));
    cancel.addEventListener("click", close);
    proceed.addEventListener("click", () => {
      try {
        const custody = controls.collect();
        finish(custody);
        close();
      } catch (cause) {
        error.hidden = false;
        error.textContent = cause instanceof Error ? cause.message : String(cause);
      }
    });
  });
}
