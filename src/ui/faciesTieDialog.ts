import { listCurveCatalog, runFaciesConfusion, type FaciesConfusionResult } from "../ipc";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow } from "./modal";
import { preferredCurveSelect } from "./plotCommon";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/** Electrofacies tie-in QC (Wave B item 8, increment 2). Cross-tabulates a predicted log-domain
 *  rock-type curve (e.g. RT_LOG from the cutoff classifier) against a reference/core rock-type
 *  curve and reports the confusion matrix + dominant-class purity — the check that decides whether
 *  the log-based classification faithfully reproduces the core rock types before it is trusted for
 *  uncored intervals. */
export async function buildFaciesTieContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const catalog = await listCurveCatalog().catch(() => []);
  const names = catalog.map((c) => c.name);
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "mc-dialog";

  const predSel = preferredCurveSelect(names, ["RT_LOG"]);
  const refSel = preferredCurveSelect(names, ["RT", "RT_LUCIA", "FACIES", "FACIES_ML"]);
  content.appendChild(formRow("Predicted RT (log)", predSel, "Log-domain rock type, e.g. RT_LOG from the cutoff classifier."));
  content.appendChild(formRow("Reference RT (core)", refSel, "The 'truth' rock type — a core-derived RT or a rock-typing RT."));
  // --- Input log set (`logSetPicker.ts`): which VERSION of the curves this reads.
  const setPicker = buildLogSetPicker({ write: false });
  for (const row of setPicker.rows) content.appendChild(row);

  content.appendChild(scope.el);

  // --- Acceptance threshold (SB-MLA-052). Ships EMPTY and stays empty: the method note this
  // module implements says to accept above a threshold and names no value, and no source the app
  // holds names one either. A pre-filled 0.7 would put SandiBumi's guess behind that silence, and
  // "accepted" is exactly the word a reader will quote without checking what it was measured on.
  const threshInput = document.createElement("input");
  threshInput.type = "number";
  threshInput.className = "form-control";
  threshInput.min = "0";
  threshInput.max = "100";
  threshInput.step = "1";
  threshInput.placeholder = "not set";
  content.appendChild(
    formRow(
      "Accept above (%)",
      threshInput,
      "Your call, not the app's: no published source states a purity threshold for this method. " +
        "Leave it empty and the mapping is reported without a verdict.",
    ),
  );

  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Compare";
  runBtn.classList.add("primary");
  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  runRow.append(runBtn, statusLine);
  content.appendChild(runRow);

  const results = document.createElement("div");
  results.className = "mc-results";
  content.appendChild(results);

  const hint = document.createElement("div");
  hint.className = "mc-chain-note";
  hint.textContent =
    "Rows = reference (core) class, columns = predicted (log) class. High dominant-class purity means the log classification maps cleanly onto the core rock types.";
  content.appendChild(hint);

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    if (predSel.value === refSel.value) {
      setStatus("Pick two different curves (predicted vs reference)");
      return;
    }
    runBtn.disabled = true;
    statusLine.textContent = "Comparing…";
    try {
      // Percent in the box, fraction on the wire — and an empty box stays UNSENT, so "the user
      // did not choose" reaches the backend as absence rather than as a zero that accepts anything.
      const pct = threshInput.value.trim();
      const thresh = pct === "" ? undefined : Number(pct) / 100;
      const res = await runFaciesConfusion({
        well_ids: wellIds,
        pred_curve: predSel.value,
        ref_curve: refSel.value,
        input_set: setPicker.inputSet(),
        accept_threshold: thresh,
      });
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        const purity = `${(res.overall_purity * 100).toFixed(1)}%`;
        const verdict =
          res.accepted == null ? "" : res.accepted ? " — ACCEPTED" : " — below your threshold";
        statusLine.textContent = `Overall purity ${purity} over ${res.n} matched samples${verdict}`;
        // The record carries the bar as well as the score: a stored verdict nobody can read back
        // against its threshold is a verdict nobody can check.
        const bar = res.accept_threshold == null ? "no threshold set" : `vs ${(res.accept_threshold * 100).toFixed(0)}%`;
        recordProcess("RockType", `Facies tie-in: ${predSel.value} vs ${refSel.value}, purity ${purity} (${bar})`);
        renderConfusion(results, res);
      }
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return { el: content, dispose: () => scope.dispose() };
}

/** Which denominator the matrix is currently showing. Never "percent" on its own — the whole
 *  point of SB-MLA-051 is that a bare percentage means two different things. */
type Norm = "count" | "row" | "col";

function renderConfusion(host: HTMLElement, res: FaciesConfusionResult): void {
  host.innerHTML = "";

  // The verdict, or the honest absence of one.
  if (res.accepted != null) {
    const v = document.createElement("div");
    v.className = "mc-hist-caption";
    v.textContent = res.accepted
      ? `Accepted — overall purity ${(res.overall_purity * 100).toFixed(1)}% is at or above the ${(res.accept_threshold! * 100).toFixed(0)}% you set.`
      : `Not accepted — overall purity ${(res.overall_purity * 100).toFixed(1)}% is below the ${(res.accept_threshold! * 100).toFixed(0)}% you set.`;
    host.appendChild(v);
  } else if (res.accept_note) {
    const v = document.createElement("div");
    v.className = "mc-chain-note";
    v.textContent = res.accept_note;
    host.appendChild(v);
  }

  // Does the predicted typing explain core permeability? (variance reduction of log10 k by class).
  if (res.n_core_plugs > 0 && res.k_var_reduction != null && Number.isFinite(res.k_var_reduction)) {
    const kvr = document.createElement("div");
    kvr.className = "mc-hist-caption";
    kvr.textContent =
      `k variance reduction ${(res.k_var_reduction * 100).toFixed(1)}% — how much of the core log10(k) ` +
      `spread the predicted class explains, over ${res.n_core_plugs} plug(s). Higher = the rock types ` +
      `separate permeability better.`;
    host.appendChild(kvr);
  }
  // How the plugs were put on the log's frame, and how many never got there (SB-MLA-054).
  if (res.n_core_plugs > 0 || res.n_core_unmatched > 0) {
    const j = document.createElement("div");
    j.className = "mc-chain-note";
    j.textContent =
      res.n_core_unmatched > 0
        ? `${res.core_match_note}. ${res.n_core_unmatched} plug(s) had no sample in range and are in no statistic above.`
        : `${res.core_match_note}. Every plug found a sample.`;
    host.appendChild(j);
  }

  // Per-reference-class dominant mapping (ROW axis) and its column-wise twin. Two tables rather
  // than one, because they answer different questions and a shared header would blur them.
  host.appendChild(
    summaryTable(
      "By reference class — does the model FIND this rock?",
      res.row_axis,
      ["Ref class", "→ dominant pred", "found", "n"],
      res.per_ref.map((r) => [String(r.ref_label), String(r.dominant_pred), pct(r.purity), String(r.count)]),
    ),
  );
  host.appendChild(
    summaryTable(
      "By predicted class — can this LABEL be trusted?",
      res.col_axis,
      ["Pred class", "→ dominant ref", "correct", "n"],
      res.per_pred.map((r) => [String(r.pred_label), String(r.dominant_ref), pct(r.recognition), String(r.count)]),
    ),
  );

  // Confusion matrix: rows = reference, cols = predicted; dominant cell per row emphasized.
  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = "Confusion matrix (row = reference, col = predicted)";
  host.appendChild(cap);

  // The denominator is a CHOICE the reader makes, and the caption under it always names what
  // they picked — so no number leaves this pane without its axis attached.
  let norm: Norm = "count";
  const seg = document.createElement("div");
  seg.className = "seg";
  const axisNote = document.createElement("div");
  axisNote.className = "mc-chain-note";
  const t = document.createElement("table");
  t.className = "mc-table ml-confusion";

  const draw = (): void => {
    for (const b of Array.from(seg.children)) {
      // `.seg-opt` paints its selected state off aria-pressed, so the accessible state and the
      // visible one cannot drift apart.
      b.setAttribute("aria-pressed", String((b as HTMLElement).dataset.norm === norm));
    }
    axisNote.textContent =
      norm === "count" ? "Raw matched-sample counts." : norm === "row" ? res.row_axis : res.col_axis;
    t.innerHTML = "";
    const hr = document.createElement("tr");
    hr.appendChild(document.createElement("th"));
    for (const pl of res.pred_labels) {
      const th = document.createElement("th");
      th.textContent = String(pl);
      hr.appendChild(th);
    }
    t.appendChild(hr);
    res.matrix.forEach((row, i) => {
      const tr = document.createElement("tr");
      const rh = document.createElement("th");
      rh.textContent = String(res.ref_labels[i]);
      tr.appendChild(rh);
      const dom = res.per_ref[i]?.dominant_pred;
      row.forEach((n, j) => {
        const td = document.createElement("td");
        td.textContent =
          norm === "count" ? String(n) : pct((norm === "row" ? res.row_pct : res.col_pct)?.[i]?.[j] ?? 0);
        if (res.pred_labels[j] === dom) td.className = "ml-diag";
        tr.appendChild(td);
      });
      t.appendChild(tr);
    });
  };

  for (const [id, label] of [
    ["count", "Counts"],
    ["row", "% of reference"],
    ["col", "% of predicted"],
  ] as [Norm, string][]) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.dataset.norm = id;
    b.textContent = label;
    b.addEventListener("click", () => {
      norm = id;
      draw();
    });
    seg.appendChild(b);
  }
  host.append(seg, axisNote, t);
  draw();
}

const pct = (v: number): string => `${(v * 100).toFixed(1)}%`;

function summaryTable(title: string, axis: string, headers: string[], rows: string[][]): HTMLElement {
  const wrap = document.createElement("div");
  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = title;
  const note = document.createElement("div");
  note.className = "mc-chain-note";
  note.textContent = axis;
  const table = document.createElement("table");
  table.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of headers) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);
  for (const r of rows) {
    const tr = document.createElement("tr");
    for (const c of r) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    table.appendChild(tr);
  }
  wrap.append(cap, note, table);
  return wrap;
}
