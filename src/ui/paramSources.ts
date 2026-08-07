import { paramSources, type ParamSource } from "../ipc";

/** The competing shipped values for one parameter, shown at the point of choice (`SB-CORE-013`).
 *
 *  Three packages routinely ship three different values for one constant, and **none of them tells
 *  the interpreter that the others exist** — none of them can, because no vendor can credibly
 *  publish a competitor's defaults. SandiBumi has no such constraint, and surfacing the
 *  disagreement is the whole point rather than a footnote.
 *
 *  One builder rather than a copy per dialog, for the `followCore.ts` reason: the same number is set
 *  in Electrofacies, GMM Facies and the ML pane, and three copies of this panel is three places for
 *  the wording — and eventually the values — to drift apart.
 *
 *  Renders NOTHING when the topic has no entries, so a field can carry a topic key harmlessly and an
 *  empty panel never reads as "nobody disagrees". */
export function buildParamSources(topic: string): HTMLElement {
  const host = document.createElement("div");
  host.className = "param-sources";
  host.hidden = true;
  if (!topic) return host;

  void paramSources(topic)
    .then((rows) => {
      if (rows.length === 0) return;
      host.hidden = false;
      const head = document.createElement("button");
      head.type = "button";
      head.className = "param-sources-head";
      // Collapsed by default: this is context for a decision, not a thing to read on every run.
      // The count is in the label so a user knows there is something here without opening it.
      head.setAttribute("aria-expanded", "false");
      head.textContent = `Shipped values elsewhere (${rows.length}) — this number is not settled`;
      const body = document.createElement("div");
      body.className = "param-sources-body";
      body.hidden = true;
      head.addEventListener("click", () => {
        body.hidden = !body.hidden;
        head.setAttribute("aria-expanded", String(!body.hidden));
      });
      for (const r of rows) body.appendChild(sourceRow(r));
      const foot = document.createElement("div");
      foot.className = "param-sources-foot";
      foot.textContent =
        "Values with their sources, not methods: what each product ships, not how it computes. " +
        "Whichever you pick is recorded with the run, together with which of these it agrees with.";
      body.appendChild(foot);
      host.append(head, body);
    })
    .catch(() => {
      /* A missing panel is the right failure: the field still works, and an error box beside a
         number the user is trying to set would be worse than the absent context. */
    });
  return host;
}

function sourceRow(r: ParamSource): HTMLElement {
  const row = document.createElement("div");
  // SandiBumi's own entry is marked, not hidden and not promoted — a panel that showed three
  // competitors and quietly omitted our own provenance would make exactly the omission it exists
  // to correct.
  row.className = r.product === "SandiBumi" ? "param-source param-source-own" : "param-source";
  const val = document.createElement("span");
  val.className = "param-source-value";
  val.textContent = r.value;
  const who = document.createElement("span");
  who.className = "param-source-product";
  who.textContent = r.product;
  const note = document.createElement("span");
  note.className = "param-source-note";
  note.textContent = r.note;
  row.append(val, who, note);
  if (r.source) {
    const src = document.createElement("span");
    src.className = "param-source-cite";
    src.textContent = r.source;
    row.appendChild(src);
  }
  return row;
}
