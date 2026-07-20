import { listDocuments, listWells, runPaySummary, type PaySummaryRow, type WellSummary } from "../ipc";
import { appState, bumpDataVersion, defaultRunWellIds, filterByActiveGroup } from "../state";
import { formRow } from "./modal";

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

/** Cutoffs & Pay Summary (Geolog .paysum model): VSH/PHIE/SWE (+ optional PERM)
 *  cutoffs → SAND / RESERVOIR / PAY flags → per-well per-zone statistics table.
 *  Also writes FLAG_SAND / FLAG_RESERVOIR / FLAG_PAY curves for the layout.
 *  Hosted as a dock pane (workspace component "paysummary"), not a popup. */
export async function buildSummaryContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const selectedWell = appState.selectedWell.get();
  const wells = filterByActiveGroup(await listWells());
  const content = document.createElement("div");
  content.className = "summary-pane";

  const wellBox = document.createElement("div");
  wellBox.className = "well-checklist";
  const wellChecks: { well: WellSummary; input: HTMLInputElement }[] = [];
  const runDefaults = defaultRunWellIds(wells);
  if (runDefaults.size === 0 && selectedWell) runDefaults.add(selectedWell.well_id);
  for (const well of wells) {
    const label = document.createElement("label");
    label.className = "well-check";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.checked = runDefaults.has(well.well_id);
    label.appendChild(input);
    label.appendChild(document.createTextNode(well.well_name));
    wellBox.appendChild(label);
    wellChecks.push({ well, input });
  }
  content.appendChild(formRow("Wells", wellBox));

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
    const wellIds = wellChecks.filter((w) => w.input.checked).map((w) => w.well.well_id);
    if (wellIds.length === 0) {
      resultBox.textContent = "Select at least one well.";
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
      bumpDataVersion();
    } catch (err) {
      resultBox.textContent = `Summary failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return { el: content, dispose: () => {} };
}

function fmt(v: number, digits = 2): string {
  return Number.isNaN(v) ? "—" : v.toFixed(digits);
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
  for (const r of rows) {
    const tr = document.createElement("tr");
    tr.className = `flag-${r.flag.toLowerCase()}`;
    tr.innerHTML =
      `<td>${r.well_name}</td><td>${r.zone}</td><td>${r.flag}</td>` +
      `<td>${fmt(r.top, 1)}</td><td>${fmt(r.bottom, 1)}</td><td>${fmt(r.gross, 1)}</td>` +
      `<td>${fmt(r.net, 1)}</td><td>${fmt(r.ntg)}</td><td>${fmt(r.avg_vsh)}</td>` +
      `<td>${fmt(r.avg_phie, 3)}</td><td>${fmt(r.avg_swe)}</td><td>${fmt(r.hpv, 2)}</td>`;
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  wrap.appendChild(table);
  container.appendChild(wrap);
}
