import { listDocuments, runPaySummary, type PaySummaryRow } from "../ipc";
import { bumpDataVersion } from "../state";
import { recordProcess } from "../processLog";
import { formRow } from "./modal";
import { buildWellScope } from "./wellScope";

/** Cutoffs picked in the Cutoff Sensitivity pane are saved as documents "cutoffs"/"__default__";
 *  the pay summary preloads them so a picked set flows straight into the report defaults. */
async function loadDefaultCutoffs(): Promise<{ vsh_max?: number; phie_min?: number; swe_max?: number; perm_min?: number | null } | null> {
  try {
    const docs = await listDocuments("cutoffs");
    const doc = docs.find((d) => d.name === "__default__");
    return doc ? JSON.parse(doc.json) : null;
  } catch {
    return null;
  }
}

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
  const saved = await loadDefaultCutoffs();
  const vshIn = numInput(saved?.vsh_max != null ? String(saved.vsh_max) : "0.5");
  const phieIn = numInput(saved?.phie_min != null ? String(saved.phie_min) : "0.1");
  const sweIn = numInput(saved?.swe_max != null ? String(saved.swe_max) : "0.6");
  const permIn = numInput(saved?.perm_min != null ? String(saved.perm_min) : "");
  permIn.placeholder = "(off)";
  content.appendChild(formRow("VSH ≤", vshIn, "Sand cutoff"));
  content.appendChild(formRow("PHIE ≥", phieIn, "Reservoir cutoff"));
  content.appendChild(formRow("SWE ≤", sweIn, "Pay cutoff"));
  content.appendChild(formRow("PERM ≥ (optional)", permIn, "Extra pay cutoff, needs a computed PERM curve"));

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
    runBtn.disabled = true;
    resultBox.textContent = "Computing…";
    try {
      const rows = await runPaySummary({
        well_ids: wellIds,
        vsh_max: parseFloat(vshIn.value),
        phie_min: parseFloat(phieIn.value),
        swe_max: parseFloat(sweIn.value),
        perm_min: Number.isNaN(permRaw) ? null : permRaw,
      });
      renderTable(resultBox, rows);
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

function renderTable(container: HTMLElement, rows: PaySummaryRow[]): void {
  container.innerHTML = "";
  if (rows.length === 0) {
    container.textContent = "No results — check that VSH/PHIE/SWE have been computed for the selected wells.";
    return;
  }
  const wrap = document.createElement("div");
  wrap.className = "summary-table-wrap";
  const table = document.createElement("table");
  table.className = "summary-table";
  table.innerHTML =
    "<thead><tr><th>Well</th><th>Zone</th><th>Flag</th><th>Top</th><th>Bottom</th><th>Gross</th>" +
    "<th>Net</th><th>N/G</th><th>Avg VSH</th><th>Avg PHIE</th><th>Avg SWE</th><th>HPV (m)</th></tr></thead>";
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
    tr.innerHTML =
      `<td>${r.well_name}</td><td>${r.zone}</td><td>${r.flag}</td>` +
      `<td>${fmt(r.top, 1)}</td><td>${fmt(r.bottom, 1)}</td><td>${fmt(r.gross, 1)}</td>` +
      `<td>${cell(r.net, 1)}</td><td>${cell(r.ntg, 2)}</td><td>${fmt(r.avg_vsh)}</td>` +
      `<td>${fmt(r.avg_phie, 3)}</td><td>${fmt(r.avg_swe)}</td><td>${cell(r.hpv, 2)}</td>`;
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  wrap.appendChild(table);
  container.appendChild(wrap);
  if (uninterpreted > 0) {
    const note = document.createElement("p");
    note.className = "summary-note";
    note.textContent =
      `${uninterpreted} of ${rows.length} row(s) show — because no sample could be classified: ` +
      `run VSH/PHIE/SWE for those wells first.`;
    container.appendChild(note);
  }
}
