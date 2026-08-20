import { runPaySummary, type PaySummaryRow } from "../ipc";
import { bumpDataVersion } from "../state";
import { shownDepthLabel, toShownDepth } from "../depthUnitPref";
import { recordProcess } from "../processLog";
import { loadCutoffDefaults } from "./cutoffs";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow } from "./modal";
import { PARAM_SOURCE_TOPICS, withParamSources } from "./paramSources";
import { buildWellScope } from "./wellScope";
import { requestRunCustody } from "./runCustody";

/** Cutoffs & Pay Summary (pay-summary model): VSH/PHIE/SWE (+ optional PERM)
 *  cutoffs → SAND / RESERVOIR / PAY flags → per-well per-zone statistics table.
 *  Also writes FLAG_SAND / FLAG_RESERVOIR / FLAG_PAY curves for the layout.
 *  Hosted as a dock pane (workspace component "paysummary"), not a popup. */
export async function buildSummaryContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const scope = await buildWellScope();
  const content = document.createElement("div");
  content.className = "summary-pane";

  // Scope selector (group / ★ pinned / selection / all) instead of a per-well checklist.
  content.appendChild(scope.el);

  const numInput = (value: string): HTMLInputElement => {
    const input = document.createElement("input");
    input.className = "form-control";
    input.type = "number";
    input.step = "any";
    input.value = value;
    return input;
  };
  const cuts = await loadCutoffDefaults();
  const vshIn = numInput(String(cuts.vsh_max));
  const phieIn = numInput(String(cuts.phie_min));
  const sweIn = numInput(String(cuts.swe_max));
  const permIn = numInput(cuts.perm_min != null ? String(cuts.perm_min) : "");
  permIn.placeholder = "(off)";
  content.appendChild(formRow("VSH ≤", withParamSources(vshIn, PARAM_SOURCE_TOPICS.cutoffVshMax), "Sand cutoff"));
  content.appendChild(formRow("PHIE ≥", withParamSources(phieIn, PARAM_SOURCE_TOPICS.cutoffPhieMin), "Reservoir cutoff"));
  content.appendChild(formRow("SWE ≤", withParamSources(sweIn, PARAM_SOURCE_TOPICS.cutoffSweMax), "Pay cutoff"));
  content.appendChild(formRow("PERM ≥ (optional)", permIn, "Extra pay cutoff, needs a computed PERM curve"));
  // --- Input log set (`logSetPicker.ts`): which VERSION of the curves this reads.
  const setPicker = buildLogSetPicker({ write: false });
  for (const row of setPicker.rows) content.appendChild(row);


  const runBtn = document.createElement("button");
  runBtn.className = "form-run-btn";
  runBtn.textContent = "Compute Summary";
  content.appendChild(runBtn);

  const resultBox = document.createElement("div");
  resultBox.className = "modal-result";
  content.appendChild(resultBox);

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      resultBox.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return;
    }
    const permRaw = parseFloat(permIn.value);
    const custody = await requestRunCustody("Compute and write pay flags");
    if (!custody) return;
    runBtn.disabled = true;
    resultBox.textContent = "Computing…";
    const cutOf = (i: HTMLInputElement, unit: string) => {
      const v = parseFloat(i.value);
      return Number.isFinite(v) ? { value: v, unit } : null;
    };
    try {
      const rows = await runPaySummary(
        {
          well_ids: wellIds,
          // SB-CUT-019: entered with a unit; a blank box is ABSENT, not a bare number.
          vsh_max: cutOf(vshIn, "v/v"),
          phie_min: cutOf(phieIn, "v/v"),
          swe_max: cutOf(sweIn, "v/v"),
          perm_min: Number.isNaN(permRaw) ? null : { value: permRaw, unit: "mD" },
          input_set: setPicker.inputSet(),
          custody,
        },
        scope.backend(),
      );
      renderPaySummaryTable(resultBox, rows);
      setStatus(`Pay summary: ${rows.length} rows; FLAG curves written`);
      // The explicit Compute Summary versions FLAG_SAND/RESERVOIR/PAY into a PAYFLAG log set —
      // a persisting write, so it earns a History entry like every other module output.
      recordProcess(
        "Pay Summary",
        `VSH≤${vshIn.value} PHIE≥${phieIn.value} SWE≤${sweIn.value}: ${rows.length} row(s) across ${wellIds.length} well(s)`,
      );
      bumpDataVersion();
    } catch (err) {
      resultBox.textContent = `Summary failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return { el: content, dispose: () => scope.dispose() };
}

function fmt(v: number | null | undefined, digits = 2): string {
  // f64::NAN from the Rust pay-summary crosses IPC as JSON `null` (serde_json has
  // no NaN), so guard null/undefined too — a bare Number.isNaN would let it reach
  // null.toFixed() and throw.
  return typeof v === "number" && Number.isFinite(v) ? v.toFixed(digits) : "—";
}

export function renderPaySummaryTable(container: HTMLElement, rows: PaySummaryRow[]): void {
  container.innerHTML = "";
  if (rows.length === 0) {
    container.textContent = "No results — check that VSH/PHIE/SWE have been computed for the selected wells.";
    return;
  }
  const wrap = document.createElement("div");
  wrap.className = "summary-table-wrap";
  const table = document.createElement("table");
  table.className = "summary-table";
  // Every length column carries the unit it is being READ in, and its value is converted to
  // match. Only HPV used to be labelled, and it said metres over whatever the project stored;
  // Top through Net said nothing at all, which on a mixed-unit desk is the same problem quieter.
  // Not escaped, deliberately: `unitLabel` returns "m" or "ft" and nothing else, so there is no
  // untrusted text here to escape — unlike the well and zone names below, which come from data.
  const u = shownDepthLabel();
  table.innerHTML =
    `<thead><tr><th>Well</th><th>Zone</th><th>Flag</th><th>Top (${u})</th><th>Bottom (${u})</th>` +
    `<th>Gross (${u})</th><th>Net (${u})</th><th>N/G</th><th>Avg VSH</th><th>Avg PHIE</th>` +
    `<th>Avg SWE</th><th>HPV (${u})</th></tr></thead>`;
  const tbody = document.createElement("tbody");
  let uninterpreted = 0;
  for (const r of rows) {
    const tr = document.createElement("tr");
    tr.className = `flag-${r.flag.toLowerCase()}`;
    // n_classified === 0 means the classifier could not judge a single in-zone sample, i.e.
    // VSH/PHIE/SWE were never computed for this well. That leaves net/ntg/hpv at exactly 0,
    // which is byte-identical to a genuine wet zone — so show "—" and say why below, rather
    // than printing a zero the run cannot actually support.
    const none = r.n_classified === 0;
    if (none) {
      uninterpreted++;
      tr.classList.add("row-uninterpreted");
      tr.title = "VSH/PHIE/SWE not computed for this well — no sample could be classified";
    }
    const cell = (v: number, d: number) => (none ? "—" : fmt(v, d));
    // `len` marks the depth-dimensioned values. N/G and the volume-fraction averages are
    // dimensionless and go through `fmt` untouched — converting a net-to-gross of 0.50 into
    // 0.15 would still look like a legal number.
    const len = (v: number) => toShownDepth(v);
    tr.innerHTML =
      `<td>${r.well_name}</td><td>${r.zone}</td><td>${r.flag}</td>` +
      `<td>${fmt(len(r.top), 1)}</td><td>${fmt(len(r.bottom), 1)}</td><td>${fmt(len(r.gross), 1)}</td>` +
      `<td>${cell(len(r.net), 1)}</td><td>${cell(r.ntg, 2)}</td><td>${fmt(r.avg_vsh)}</td>` +
      `<td>${fmt(r.avg_phie, 3)}</td><td>${fmt(r.avg_swe)}</td><td>${cell(len(r.hpv), 2)}</td>`;
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  wrap.appendChild(table);
  container.appendChild(wrap);
  if (uninterpreted > 0) {
    const note = document.createElement("p");
    note.className = "summary-note";
    note.textContent =
      `${uninterpreted} of ${rows.length} row(s) show "—" for Net, N/G and HPV: no sample could ` +
      `be classified, so those wells have no interpretation yet. Run VSH/PHIE/SWE first.`;
    container.appendChild(note);
  }
}
