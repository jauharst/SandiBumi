import {
  autocorrelateTop,
  autocorrelateMulti,
  listTops,
  listWells,
  resolveWellScope,
  upsertTop,
  type AutoCorrProposal,
  type BackendWellScope,
  type MultiWellProposal,
  type WellSummary,
} from "../ipc";
import { storedDepthLabel } from "../depthUnitPref";
import { recordProcess } from "../processLog";
import { bumpDataVersion, filterByActiveGroup } from "../state";
import { pushUndo } from "../undo";
import { formRow } from "./modal";
import { escapeHtml } from "./safeDom";

/** Petrel-style marker autocorrelation: propagate a top (or several tops together) picked
 *  in the source well to other wells by matching the shape of a chosen log (GR by default)
 *  around each pick. Rigid best-lag or an elastic depth warp; proposals show the correlation
 *  coefficient; the user reviews, unticks weak matches, and applies — one undoable batch.
 *  Hosted as a dock pane (workspace component "autocorr") that follows the selected well. */
export async function buildAutoCorrContent(
  well: WellSummary,
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const content = document.createElement("div");
  content.className = "autocorr-pane";
  const messagePane = (text: string): { el: HTMLElement } => {
    const div = document.createElement("div");
    div.className = "logview-message";
    div.textContent = text;
    content.appendChild(div);
    return { el: content };
  };

  let tops: { top_name: string }[] = [];
  let wells: WellSummary[] = [];
  try {
    [tops, wells] = await Promise.all([listTops(well.well_id), listWells()]);
  } catch (err) {
    return messagePane(`Autocorrelate: ${err}`);
  }
  if (tops.length === 0) {
    return messagePane(`No tops picked in ${well.well_name} yet — pick one in the log view first (🏷)`);
  }
  let targets = filterByActiveGroup(wells).filter((w) => w.well_id !== well.well_id);
  if (targets.length === 0) {
    return messagePane("No other wells (in the active group) to correlate to");
  }

  // Tops as a checkbox list — tick one to correlate a single top, several to propagate a
  // consistent set together (monotone, no crossings).
  const topsBox = document.createElement("div");
  topsBox.className = "autocorr-tops";
  const topChecks: HTMLInputElement[] = [];
  tops.forEach((t, i) => {
    const label = document.createElement("label");
    label.className = "autocorr-top";
    const box = document.createElement("input");
    box.type = "checkbox";
    box.value = t.top_name;
    box.checked = i === 0; // default: single-top mode, first top
    box.addEventListener("change", syncControls);
    label.appendChild(box);
    label.appendChild(document.createTextNode(` ${t.top_name}`));
    topsBox.appendChild(label);
    topChecks.push(box);
  });
  const selectedTops = (): string[] => topChecks.filter((b) => b.checked).map((b) => b.value);

  const allToggle = document.createElement("button");
  allToggle.className = "lp-btn lp-btn-sm";
  allToggle.type = "button";
  allToggle.textContent = "All";
  allToggle.addEventListener("click", () => {
    const target = topChecks.some((b) => !b.checked);
    topChecks.forEach((b) => (b.checked = target));
    syncControls();
  });

  const curveInput = document.createElement("input");
  curveInput.className = "form-control";
  curveInput.value = "GR";

  const methodSel = document.createElement("select");
  methodSel.className = "form-control";
  for (const [value, text] of [
    ["shift", "Rigid shift (fast)"],
    ["warp", "Elastic warp"],
  ] as const) {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = text;
    methodSel.appendChild(opt);
  }
  methodSel.addEventListener("change", syncControls);

  const maxStretchInput = document.createElement("input");
  maxStretchInput.className = "form-control";
  maxStretchInput.type = "number";
  maxStretchInput.value = "1.5";
  maxStretchInput.step = "0.1";
  maxStretchInput.min = "1";

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

  content.appendChild(formRow("Source well", Object.assign(document.createElement("span"), { textContent: well.well_name })));
  const topsRow = document.createElement("div");
  topsRow.className = "autocorr-tops-row";
  topsRow.appendChild(allToggle);
  topsRow.appendChild(topsBox);
  content.appendChild(formRow("Tops", topsRow, "One top = single correlation; several = propagate a consistent set"));
  content.appendChild(formRow("Log", curveInput, "Curve whose shape is matched (GR is standard)"));
  content.appendChild(formRow("Method", methodSel, "Rigid best-lag, or an elastic depth warp that follows local stretch"));
  const maxStretchRow = formRow("Max stretch ×", maxStretchInput, "Warp elasticity — how much local stretch/compression to allow");
  content.appendChild(maxStretchRow);
  // `AutoCorrRequest::half_window` and `search_range` are documented "depth units" and are used
  // against the stored grid without conversion, so both read in the project's own unit.
  const du = storedDepthLabel();
  const windowRow = formRow(`Window ± (${du})`, windowInput, "Half-length of the pattern window (single top; auto for several)");
  content.appendChild(windowRow);
  content.appendChild(formRow(`Search ± (${du})`, searchInput, "How far above/below the initial guess to search in each well"));

  const runBtn = document.createElement("button");
  runBtn.className = "lp-btn";
  content.appendChild(runBtn);

  const resultHost = document.createElement("div");
  resultHost.className = "autocorr-results";
  content.appendChild(resultHost);

  function syncControls(): void {
    const count = selectedTops().length;
    maxStretchRow.style.display = methodSel.value === "warp" ? "" : "none";
    windowRow.style.display = count === 1 ? "" : "none";
    const n = targets.length;
    const wellWord = `${n} well${n === 1 ? "" : "s"}`;
    runBtn.textContent = count > 1 ? `Correlate ${count} tops → ${wellWord}` : `Correlate ${wellWord}`;
    runBtn.disabled = count === 0;
  }
  syncControls();

  runBtn.addEventListener("click", () => {
    void (async () => {
      const picks = selectedTops();
      if (picks.length === 0) {
        setStatus("Select at least one top");
        return;
      }
      const curve = curveInput.value.trim().toUpperCase() || "GR";
      const method = methodSel.value as "shift" | "warp";
      const maxStretch = parseFloat(maxStretchInput.value) || 1.5;
      const searchRange = parseFloat(searchInput.value) || 25;
      const backendScope: BackendWellScope = { kind: "active_group" };
      const restore = () => syncControls();
      runBtn.disabled = true;
      runBtn.textContent = "Correlating…";
      try {
        const [latestWells, targetIds] = await Promise.all([listWells(), resolveWellScope(backendScope)]);
        const allowed = new Set(targetIds.filter((wellId) => wellId !== well.well_id));
        targets = latestWells.filter((candidate) => allowed.has(candidate.well_id));
        if (targets.length === 0) {
          setStatus("No other wells in the current active group to correlate to");
          return;
        }
        if (picks.length === 1) {
          const halfWindow = parseFloat(windowInput.value) || 10;
          const result = await autocorrelateTop(
            {
              source_well_id: well.well_id,
              top_name: picks[0],
              curve,
              half_window: halfWindow,
              search_range: searchRange,
              target_well_ids: [...allowed],
              method,
              max_stretch: maxStretch,
            },
            backendScope,
          );
          if (result.error) {
            setStatus(`Autocorrelate: ${result.error}`);
            resultHost.textContent = result.error;
            return;
          }
          renderProposals(resultHost, result.proposals, picks[0], targets, (rows) => {
            void applyProposals(rows, backendScope, setStatus, () => (resultHost.innerHTML = ""));
          });
        } else {
          const result = await autocorrelateMulti(
            {
              source_well_id: well.well_id,
              top_names: picks,
              curve,
              search_range: searchRange,
              max_stretch: maxStretch,
              method,
              target_well_ids: [...allowed],
            },
            backendScope,
          );
          if (result.error) {
            setStatus(`Autocorrelate: ${result.error}`);
            resultHost.textContent = result.error;
            return;
          }
          renderMultiProposals(resultHost, result.proposals, targets, (rows) => {
            void applyMultiProposals(rows, backendScope, setStatus, () => (resultHost.innerHTML = ""));
          });
        }
      } catch (err) {
        setStatus(`Autocorrelate failed: ${err}`);
      } finally {
        runBtn.disabled = false;
        restore();
      }
    })();
  });

  return { el: content };
}

interface ApplyRow {
  wellId: string;
  topName: string;
  depth: number;
  oldDepth: number | null;
}

/** Single-top result: one row per target well. */
function renderProposals(
  host: HTMLElement,
  proposals: AutoCorrProposal[],
  topName: string,
  targets: WellSummary[],
  onApply: (rows: ApplyRow[]) => void,
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

  const applyBtn = makeApplyButton(
    () => checks.filter((c) => c.box.checked && c.p.proposed_depth !== null).length,
    checks.map((c) => c.box),
    () => {
      const rows = checks
        .filter((c) => c.box.checked && c.p.proposed_depth !== null)
        .map((c) => ({ wellId: c.p.well_id, topName, depth: c.p.proposed_depth!, oldDepth: c.p.current_depth }));
      if (rows.length > 0) onApply(rows);
    },
  );
  host.appendChild(applyBtn);
}

/** Multi-top result: one row per (well, marker), grouped by well. */
function renderMultiProposals(
  host: HTMLElement,
  proposals: MultiWellProposal[],
  targets: WellSummary[],
  onApply: (rows: ApplyRow[]) => void,
): void {
  host.innerHTML = "";
  const names = new Map(targets.map((w) => [w.well_id, w.well_name]));
  const table = document.createElement("table");
  table.className = "dbgrid";
  table.innerHTML = `<thead><tr>
      <th></th><th>Well</th><th>Top</th><th>Current</th><th>Proposed</th><th>r</th>
    </tr></thead>`;
  const tbody = document.createElement("tbody");
  const checks: { box: HTMLInputElement; wellId: string; topName: string; depth: number; oldDepth: number | null }[] = [];

  for (const wp of proposals) {
    const wellName = names.get(wp.well_id) ?? wp.well_id;
    if (wp.error) {
      const tr = document.createElement("tr");
      tr.classList.add("weak-match");
      // wellName is the LAS-supplied ~W WELL value, stored verbatim (see R9) — escape it and the
      // backend error string so a hostile header can't inject markup into this row.
      tr.innerHTML = `<td></td><td>${escapeHtml(wellName)}</td><td colspan="4">${escapeHtml(String(wp.error))}</td>`;
      tbody.appendChild(tr);
      continue;
    }
    wp.markers.forEach((mk, i) => {
      const tr = document.createElement("tr");
      const box = document.createElement("input");
      box.type = "checkbox";
      box.checked = mk.proposed_depth !== null && mk.correlation >= 0.7;
      box.disabled = mk.proposed_depth === null;
      const tdBox = document.createElement("td");
      tdBox.appendChild(box);
      tr.appendChild(tdBox);
      const cells = [
        i === 0 ? wellName : "", // only label the well on its first marker row
        mk.top_name,
        mk.current_depth === null ? "—" : mk.current_depth.toFixed(1),
        mk.proposed_depth === null ? "—" : mk.proposed_depth.toFixed(1),
        Number.isFinite(mk.correlation) ? mk.correlation.toFixed(2) : "—",
      ];
      for (const text of cells) {
        const td = document.createElement("td");
        td.textContent = text;
        tr.appendChild(td);
      }
      if (mk.proposed_depth !== null && mk.correlation < 0.7) tr.classList.add("weak-match");
      tbody.appendChild(tr);
      if (mk.proposed_depth !== null) {
        checks.push({ box, wellId: wp.well_id, topName: mk.top_name, depth: mk.proposed_depth, oldDepth: mk.current_depth });
      }
    });
  }
  table.appendChild(tbody);
  host.appendChild(table);

  const applyBtn = makeApplyButton(
    () => checks.filter((c) => c.box.checked).length,
    checks.map((c) => c.box),
    () => {
      const rows = checks
        .filter((c) => c.box.checked)
        .map((c) => ({ wellId: c.wellId, topName: c.topName, depth: c.depth, oldDepth: c.oldDepth }));
      if (rows.length > 0) onApply(rows);
    },
  );
  host.appendChild(applyBtn);
}

/** Apply button whose label tracks the number of ticked picks. */
function makeApplyButton(count: () => number, boxes: HTMLInputElement[], onClick: () => void): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.className = "lp-btn";
  const refresh = () => (btn.textContent = `Apply ${count()} pick${count() === 1 ? "" : "s"}`);
  refresh();
  for (const b of boxes) b.addEventListener("change", refresh);
  btn.addEventListener("click", onClick);
  return btn;
}

/** Writes the accepted picks as one undoable batch (undo restores/deletes per well). */
async function applyProposals(
  rows: ApplyRow[],
  scope: BackendWellScope,
  setStatus: (text: string) => void,
  onApplied: () => void,
): Promise<void> {
  const topName = rows[0]?.topName ?? "top";
  await applyRows(rows, scope, `autocorrelate ${topName} (${rows.length} wells)`, `Autocorrelated ${topName} into ${rows.length} well(s)`, setStatus, onApplied);
}

/** Multi-top apply: every accepted (well, marker) pick in one undoable batch. */
async function applyMultiProposals(
  rows: ApplyRow[],
  scope: BackendWellScope,
  setStatus: (text: string) => void,
  onApplied: () => void,
): Promise<void> {
  const wells = new Set(rows.map((r) => r.wellId)).size;
  const label = `autocorrelate ${rows.length} picks (${wells} wells)`;
  await applyRows(rows, scope, label, `Autocorrelated ${rows.length} marker(s) across ${wells} well(s)`, setStatus, onApplied);
}

async function applyRows(
  rows: ApplyRow[],
  scope: BackendWellScope,
  undoLabel: string,
  processMsg: string,
  setStatus: (text: string) => void,
  onApplied: () => void,
): Promise<void> {
  const { deleteTop } = await import("../ipc");
  const exactScope: BackendWellScope = {
    kind: "explicit",
    well_ids: [...new Set(rows.map((row) => row.wellId))],
  };
  const applyNew = async (backendScope: BackendWellScope) => {
    for (const r of rows) await upsertTop(r.wellId, r.topName, r.depth, null, backendScope);
    bumpDataVersion();
  };
  const applyOld = async () => {
    for (const r of rows) {
      if (r.oldDepth === null) await deleteTop(r.wellId, r.topName);
      else await upsertTop(r.wellId, r.topName, r.oldDepth, null, exactScope);
    }
    bumpDataVersion();
  };
  try {
    await applyNew(scope);
  } catch (err) {
    setStatus(`Apply picks failed: ${err}`);
    return;
  }
  pushUndo({ label: undoLabel, undo: applyOld, redo: () => applyNew(exactScope) });
  recordProcess("Tops", processMsg);
  setStatus(processMsg);
  onApplied();
}
