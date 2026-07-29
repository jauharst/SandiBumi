import { listCurveCatalog, runFaciesConfusion, type FaciesConfusionResult } from "../ipc";
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
  content.appendChild(scope.el);

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
      const res = await runFaciesConfusion({ well_ids: wellIds, pred_curve: predSel.value, ref_curve: refSel.value });
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        statusLine.textContent = `Overall purity ${(res.overall_purity * 100).toFixed(1)}% over ${res.n} matched samples`;
        recordProcess("RockType", `Facies tie-in: ${predSel.value} vs ${refSel.value}, purity ${(res.overall_purity * 100).toFixed(1)}%`);
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

function renderConfusion(host: HTMLElement, res: FaciesConfusionResult): void {
  host.innerHTML = "";

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

  // Per-reference-class dominant mapping + purity.
  const perTable = document.createElement("table");
  perTable.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["Ref class", "→ dominant pred", "purity", "n"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  perTable.appendChild(head);
  for (const r of res.per_ref) {
    const tr = document.createElement("tr");
    for (const c of [String(r.ref_label), String(r.dominant_pred), `${(r.purity * 100).toFixed(1)}%`, String(r.count)]) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    perTable.appendChild(tr);
  }
  host.appendChild(perTable);

  // Confusion matrix: rows = reference, cols = predicted; dominant cell per row emphasized.
  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = "Confusion matrix (row = reference, col = predicted)";
  host.appendChild(cap);
  const t = document.createElement("table");
  t.className = "mc-table ml-confusion";
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
      td.textContent = String(n);
      if (res.pred_labels[j] === dom) td.className = "ml-diag";
      tr.appendChild(td);
    });
    t.appendChild(tr);
  });
  host.appendChild(t);
}
