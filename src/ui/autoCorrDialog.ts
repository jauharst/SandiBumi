import {
  autocorrelateTop,
  listTops,
  listWells,
  upsertTop,
  type AutoCorrProposal,
  type WellSummary,
} from "../ipc";
import { recordProcess } from "../processLog";
import { appState, bumpDataVersion, filterByActiveGroup, setStatus } from "../state";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";

/** Petrel-style marker autocorrelation: propagate a top picked in the selected well to
 *  other wells by matching the shape of a chosen log (GR by default) around the pick.
 *  Proposals show the correlation coefficient; the user reviews, unticks weak matches,
 *  and applies — one undoable batch. */
export async function openAutoCorrDialog(): Promise<void> {
  const source = appState.selectedWell.get();
  if (!source) {
    setStatus("Select a source well in Wells & Tops first");
    return;
  }

  let tops: { top_name: string }[] = [];
  let wells: WellSummary[] = [];
  try {
    [tops, wells] = await Promise.all([listTops(source.well_id), listWells()]);
  } catch (err) {
    setStatus(`Autocorrelate: ${err}`);
    return;
  }
  if (tops.length === 0) {
    setStatus(`No tops picked in ${source.well_name} yet — pick one in the log view first (🏷)`);
    return;
  }
  const targets = filterByActiveGroup(wells).filter((w) => w.well_id !== source.well_id);
  if (targets.length === 0) {
    setStatus("No other wells (in the active group) to correlate to");
    return;
  }

  const content = document.createElement("div");

  const topSel = document.createElement("select");
  topSel.className = "form-control";
  for (const t of tops) {
    const opt = document.createElement("option");
    opt.value = t.top_name;
    opt.textContent = t.top_name;
    topSel.appendChild(opt);
  }
  const curveInput = document.createElement("input");
  curveInput.className = "form-control";
  curveInput.value = "GR";
  const windowInput = document.createElement("input");
  windowInput.className = "form-control";
  windowInput.type = "number";
  windowInput.value = "10";
  windowInput.step = "1";
  const searchInput = document.createElement("input");
  searchInput.className = "form-control";
  searchInput.type = "number";
  searchInput.value = "25";
  searchInput.step = "1";

  content.appendChild(formRow("Source well", Object.assign(document.createElement("span"), { textContent: source.well_name })));
  content.appendChild(formRow("Top", topSel));
  content.appendChild(formRow("Log", curveInput, "Curve whose shape is matched (GR is standard)"));
  content.appendChild(formRow("Window ± (m)", windowInput, "Half-length of the pattern window around the pick"));
  content.appendChild(formRow("Search ± (m)", searchInput, "How far above/below the initial guess to search in each well"));

  const runBtn = document.createElement("button");
  runBtn.className = "lp-btn";
  runBtn.textContent = `Correlate ${targets.length} well${targets.length > 1 ? "s" : ""}`;
  content.appendChild(runBtn);

  const resultHost = document.createElement("div");
  resultHost.className = "autocorr-results";
  content.appendChild(resultHost);

  const close = openModal("Autocorrelate top", content, 560);

  runBtn.addEventListener("click", () => {
    void (async () => {
      const topName = topSel.value;
      const curve = curveInput.value.trim().toUpperCase() || "GR";
      const halfWindow = parseFloat(windowInput.value) || 10;
      const searchRange = parseFloat(searchInput.value) || 25;
      runBtn.disabled = true;
      runBtn.textContent = "Correlating…";
      let proposals: AutoCorrProposal[] = [];
      try {
        const result = await autocorrelateTop({
          source_well_id: source.well_id,
          top_name: topName,
          curve,
          half_window: halfWindow,
          search_range: searchRange,
          target_well_ids: targets.map((w) => w.well_id),
        });
        if (result.error) {
          setStatus(`Autocorrelate: ${result.error}`);
          resultHost.textContent = result.error;
          return;
        }
        proposals = result.proposals;
      } catch (err) {
        setStatus(`Autocorrelate failed: ${err}`);
        return;
      } finally {
        runBtn.disabled = false;
        runBtn.textContent = `Correlate ${targets.length} well${targets.length > 1 ? "s" : ""}`;
      }

      renderProposals(resultHost, proposals, targets, (apply) => {
        void applyProposals(apply, topName, close);
      });
    })();
  });
}

function renderProposals(
  host: HTMLElement,
  proposals: AutoCorrProposal[],
  targets: WellSummary[],
  onApply: (rows: { wellId: string; depth: number; oldDepth: number | null }[]) => void,
): void {
  host.innerHTML = "";
  const names = new Map(targets.map((w) => [w.well_id, w.well_name]));
  const table = document.createElement("table");
  table.className = "dbgrid";
  table.innerHTML = `<thead><tr>
      <th></th><th>Well</th><th>Current</th><th>Proposed</th><th>r</th><th></th>
    </tr></thead>`;
  const tbody = document.createElement("tbody");
  const checks: { box: HTMLInputElement; p: AutoCorrProposal }[] = [];

  for (const p of proposals) {
    const tr = document.createElement("tr");
    const box = document.createElement("input");
    box.type = "checkbox";
    // Strong matches pre-ticked; weak or failed ones left for the user to judge.
    box.checked = p.proposed_depth !== null && p.correlation >= 0.7;
    box.disabled = p.proposed_depth === null;
    const tdBox = document.createElement("td");
    tdBox.appendChild(box);
    tr.appendChild(tdBox);
    const cells = [
      names.get(p.well_id) ?? p.well_id,
      p.current_depth === null ? "—" : p.current_depth.toFixed(1),
      p.proposed_depth === null ? "—" : p.proposed_depth.toFixed(1),
      Number.isFinite(p.correlation) ? p.correlation.toFixed(2) : "—",
      p.error ?? "",
    ];
    for (const text of cells) {
      const td = document.createElement("td");
      td.textContent = text;
      tr.appendChild(td);
    }
    if (p.proposed_depth !== null && p.correlation < 0.7) tr.classList.add("weak-match");
    tbody.appendChild(tr);
    checks.push({ box, p });
  }
  table.appendChild(tbody);
  host.appendChild(table);

  const applyBtn = document.createElement("button");
  applyBtn.className = "lp-btn";
  const countChecked = () => checks.filter((c) => c.box.checked && c.p.proposed_depth !== null).length;
  const refreshLabel = () => (applyBtn.textContent = `Apply ${countChecked()} pick${countChecked() === 1 ? "" : "s"}`);
  refreshLabel();
  for (const c of checks) c.box.addEventListener("change", refreshLabel);
  applyBtn.addEventListener("click", () => {
    const rows = checks
      .filter((c) => c.box.checked && c.p.proposed_depth !== null)
      .map((c) => ({ wellId: c.p.well_id, depth: c.p.proposed_depth!, oldDepth: c.p.current_depth }));
    if (rows.length > 0) onApply(rows);
  });
  host.appendChild(applyBtn);
}

/** Writes the accepted picks as one undoable batch (undo restores/deletes per well). */
async function applyProposals(
  rows: { wellId: string; depth: number; oldDepth: number | null }[],
  topName: string,
  close: () => void,
): Promise<void> {
  const { deleteTop } = await import("../ipc");
  const applyNew = async () => {
    for (const r of rows) await upsertTop(r.wellId, topName, r.depth, null);
    bumpDataVersion();
  };
  const applyOld = async () => {
    for (const r of rows) {
      if (r.oldDepth === null) await deleteTop(r.wellId, topName);
      else await upsertTop(r.wellId, topName, r.oldDepth, null);
    }
    bumpDataVersion();
  };
  try {
    await applyNew();
  } catch (err) {
    setStatus(`Apply picks failed: ${err}`);
    return;
  }
  pushUndo({ label: `autocorrelate ${topName} (${rows.length} wells)`, undo: applyOld, redo: applyNew });
  recordProcess("Tops", `Autocorrelated ${topName} into ${rows.length} well(s)`);
  setStatus(`${topName} picked in ${rows.length} well(s) by autocorrelation`);
  close();
}
