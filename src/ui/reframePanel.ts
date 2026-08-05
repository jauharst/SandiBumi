import {
  listGenericCurveCatalog,
  listLogSetNames,
  listWells,
  runReframe,
  type ReframeResult,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion } from "../state";
import { formRow } from "./modal";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/**
 * **Reframe** — resample a log set onto a different sampling, as a new set.
 *
 * Jauhar, 2026-08-05: *"i have cons in well A 'wire_input' with 0.1523 sampling, meanwhile other
 * wells 0.5, user should have to resample well A cons to new cons with 0.5 sampling"*.
 *
 * A pane rather than a modal (the standing rule), and the reason applies unusually well here: the
 * probe below is read next to the Wells pane and against the log view, and re-framing a field is a
 * well-by-well job worked through over an afternoon rather than a form filled once.
 *
 * **The probe is the design.** The first thing the pane does is tell the user what each well is
 * ALREADY sampled at — a number nothing else in the app displays, and the one the whole decision
 * turns on. Guessing it from a LAS header is how someone re-frames a 0.1524 m well onto 0.15 and
 * quietly resamples every curve in it for no reason.
 */
export async function buildReframeContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const scope = await buildWellScope();
  let disposed = false;

  const el = document.createElement("div");
  el.className = "module-pane";

  const head = document.createElement("div");
  head.className = "module-head";
  const chip = document.createElement("span");
  chip.className = "module-chip";
  chip.textContent = "R";
  const title = document.createElement("span");
  title.className = "module-title";
  title.textContent = "Reframe";
  head.append(chip, title);
  el.appendChild(head);

  const lead = document.createElement("p");
  lead.className = "modal-hint";
  lead.textContent =
    "Resamples a set onto a different sampling and writes it as a new set. The original is left " +
    "exactly as it is — nothing here overwrites a curve.";
  el.appendChild(lead);

  // `buildWellScope` carries its own RUN ON label (the design 1d segmented pill), so nothing is
  // added above it here — a second heading reads as two controls.
  el.appendChild(scope.el);

  const grid = document.createElement("div");
  grid.className = "module-args";

  // --- Source ---------------------------------------------------------------
  const srcKind = document.createElement("select");
  srcKind.className = "form-control";
  for (const [v, t] of [
    ["logset", "A log set (a versioned result)"],
    ["import", "A delivery set (as imported)"],
    ["standard", "The well's raw standard curves"],
  ]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = t;
    srcKind.appendChild(o);
  }
  const srcName = document.createElement("select");
  srcName.className = "form-control";
  grid.appendChild(formRow("Resample from", srcKind, "Which store the curves are read out of, at their OWN depths."));
  const srcNameRow = formRow("Set", srcName, "The delivery or result to re-frame.");
  grid.appendChild(srcNameRow);

  // --- Target ---------------------------------------------------------------
  const tgtKind = document.createElement("select");
  tgtKind.className = "form-control";
  for (const [v, t] of [
    ["step", "A sampling I give"],
    ["match_well", "Match another well's sampling"],
    ["match_set", "Match another set's sampling"],
  ]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = t;
    tgtKind.appendChild(o);
  }
  const stepInput = document.createElement("input");
  stepInput.className = "form-control";
  stepInput.type = "number";
  stepInput.step = "0.0001";
  stepInput.placeholder = "e.g. 0.5";
  const matchWell = document.createElement("select");
  matchWell.className = "form-control";
  const matchSet = document.createElement("select");
  matchSet.className = "form-control";
  grid.appendChild(formRow("Onto", tgtKind, "The frame to land on."));
  const stepRow = formRow(
    "Sampling",
    stepInput,
    "In the project's depth unit. No default: there is no generic sampling, and a wrong one is " +
      "invisible once the curve is written.",
  );
  const matchWellRow = formRow("Well", matchWell, "Its standard depth grid becomes this well's frame.");
  const matchSetRow = formRow("Set", matchSet, "A set that carries its own frame.");
  grid.append(stepRow, matchWellRow, matchSetRow);

  // --- Method ---------------------------------------------------------------
  const method = document.createElement("select");
  method.className = "form-control";
  for (const [v, t] of [
    ["Auto", "Auto — average a measurement, carry a class code whole"],
    ["Mean", "Mean — arithmetic, for porosity and any volume fraction"],
    ["Geometric", "Geometric — permeability through randomly arranged rock, and anything read on a log scale"],
    ["Harmonic", "Harmonic — permeability across bedding (layers in series with the flow)"],
    ["Median", "Median — an average one spike cannot drag"],
    ["Interpolate", "Interpolate — for upsampling a continuous curve"],
    ["Nearest", "Nearest — the only correct choice for a class or flag curve"],
    ["Mode", "Mode — the class that dominates each output sample"],
  ]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = t;
    method.appendChild(o);
  }
  grid.appendChild(
    formRow(
      "Averaging",
      method,
      "Applies to every curve unless the run reports otherwise. Permeability averaged arithmetically " +
        "is a rock that does not exist — 1000 mD and 0.01 mD are 500 mD arithmetically and 0.02 mD " +
        "harmonically, and the arithmetic answer always reads highest.",
    ),
  );

  const curvesInput = document.createElement("input");
  curvesInput.className = "form-control";
  curvesInput.type = "text";
  curvesInput.placeholder = "all curves in the set";
  grid.appendChild(
    formRow("Curves (optional)", curvesInput, "Comma-separated. Blank carries everything the source holds."),
  );

  const outSet = document.createElement("input");
  outSet.className = "form-control";
  outSet.type = "text";
  outSet.value = "REFRAMED";
  grid.appendChild(
    formRow(
      "Write to log set",
      outSet,
      "A name already in use gets a new VERSION of that same set — the set keeps its name, the " +
        "version number tells the two apart.",
    ),
  );
  el.appendChild(grid);

  const callout = document.createElement("div");
  callout.className = "module-callout";
  callout.textContent =
    "A re-framed set carries its own depths. Point a module's Input log set at it and the whole " +
    "run happens on that sampling — the other curves come along, resampled the same way.";
  el.appendChild(callout);

  // --- Footer ---------------------------------------------------------------
  const probeBtn = document.createElement("button");
  probeBtn.className = "btn";
  probeBtn.textContent = "Check sampling";
  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Reframe";
  const status = document.createElement("div");
  status.className = "modal-result module-status";
  const footer = document.createElement("div");
  footer.className = "module-footer";
  footer.append(probeBtn, runBtn, status);
  el.appendChild(footer);

  const report = document.createElement("div");
  report.className = "reframe-report";
  el.appendChild(report);

  // --- Behaviour ------------------------------------------------------------
  const syncRows = (): void => {
    srcNameRow.hidden = srcKind.value === "standard";
    stepRow.hidden = tgtKind.value !== "step";
    matchWellRow.hidden = tgtKind.value !== "match_well";
    matchSetRow.hidden = tgtKind.value !== "match_set";
  };
  srcKind.addEventListener("change", () => {
    syncRows();
    void refreshNames();
  });
  tgtKind.addEventListener("change", syncRows);
  syncRows();

  const fill = (select: HTMLSelectElement, names: string[], keep: string): void => {
    select.innerHTML = "";
    for (const n of names) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      select.appendChild(o);
    }
    if (names.includes(keep)) select.value = keep;
  };

  // The delivery-set names come from the ACTIVE well's own catalog, because a delivery belongs to
  // a well; log-set names are project-wide, because a result set spans the run that made it.
  let allWells: WellSummary[] = [];
  async function refreshNames(): Promise<void> {
    try {
      const logSets = await listLogSetNames();
      if (disposed) return;
      const well = appState.selectedWell.get();
      let deliveries: string[] = [];
      if (well) {
        const cat = await listGenericCurveCatalog(well.well_id).catch(() => []);
        deliveries = [...new Set(cat.map((c) => c.set_name))].sort();
      }
      if (disposed) return;
      fill(srcName, srcKind.value === "import" ? deliveries : logSets, srcName.value);
      fill(matchSet, logSets, matchSet.value);
      allWells = await listWells().catch(() => allWells);
      if (disposed) return;
      fill(
        matchWell,
        allWells.map((w) => w.well_name),
        matchWell.value,
      );
    } catch {
      // No backend / fresh project: leave the pickers as they are.
    }
  }
  const unsubWell = appState.selectedWell.subscribe(() => void refreshNames());
  const unsubData = appState.dataVersion.subscribe(() => void refreshNames());

  function buildRequest(preview: boolean): Record<string, unknown> | null {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      status.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return null;
    }
    if (tgtKind.value === "step" && !(parseFloat(stepInput.value) > 0)) {
      status.textContent = "Set the sampling to re-frame onto.";
      stepInput.focus();
      return null;
    }
    if (!outSet.value.trim()) {
      status.textContent = "Name the log set to write.";
      outSet.focus();
      return null;
    }
    return {
      well_ids: wellIds,
      source: { kind: srcKind.value, name: srcKind.value === "standard" ? null : srcName.value },
      curves: curvesInput.value
        .split(",")
        .map((s) => s.trim().toUpperCase())
        .filter(Boolean),
      target: {
        kind: tgtKind.value,
        step: tgtKind.value === "step" ? parseFloat(stepInput.value) : null,
        well_id: allWells.find((w) => w.well_name === matchWell.value)?.well_id ?? null,
        set_name: matchSet.value || null,
        top: null,
        base: null,
      },
      methods: {},
      default_method: method.value,
      output_set: outSet.value.trim(),
      preview,
    };
  }

  const num = (v: number): string => (Number.isFinite(v) ? v.toFixed(4).replace(/0+$/, "").replace(/\.$/, "") : "—");

  function renderReport(results: ReframeResult[], preview: boolean): void {
    report.innerHTML = "";
    const table = document.createElement("table");
    table.className = "data-table";
    const head = document.createElement("tr");
    for (const h of ["Well", "Now", "After", "Rows", "Curves", "Note"]) {
      const th = document.createElement("th");
      th.textContent = h;
      head.appendChild(th);
    }
    table.appendChild(head);
    for (const r of results) {
      const tr = document.createElement("tr");
      const cells = [
        r.well_name,
        num(r.source_step),
        num(r.target_step),
        r.error ? "—" : String(r.rows),
        r.error ? "—" : r.curves.map((c) => `${c.name} (${c.method.toLowerCase()})`).join(", "),
        r.error ?? r.notes.join(" "),
      ];
      for (const [i, c] of cells.entries()) {
        const td = document.createElement("td");
        td.textContent = c;
        // A well already at the target sampling is the answer the probe exists to give: it needs
        // no re-frame, and re-framing it anyway resamples every curve for nothing.
        if (i === 2 && !r.error && Math.abs(r.source_step - r.target_step) < 1e-4) {
          td.classList.add("reframe-same");
          td.title = "already at this sampling";
        }
        if (i === 5 && r.error) td.classList.add("reframe-error");
        tr.appendChild(td);
      }
      table.appendChild(tr);
    }
    report.appendChild(table);
    if (preview) {
      const hint = document.createElement("p");
      hint.className = "modal-hint";
      hint.textContent = "Nothing has been written. Press Reframe to commit.";
      report.appendChild(hint);
    }
  }

  async function go(preview: boolean): Promise<void> {
    const req = buildRequest(preview);
    if (!req) return;
    probeBtn.disabled = true;
    runBtn.disabled = true;
    status.textContent = preview ? "Checking…" : "Re-framing…";
    try {
      const results = await runReframe(req);
      if (disposed) return;
      renderReport(results, preview);
      const ok = results.filter((r) => !r.error).length;
      status.textContent = preview
        ? `${results.length} well(s) checked.`
        : `${ok}/${results.length} well(s) written to ${outSet.value.trim().toUpperCase()}.`;
      if (!preview && ok > 0) {
        setStatus(`Reframe: ${ok} well(s) → ${outSet.value.trim().toUpperCase()}`);
        recordProcess("reframe", `${ok} well(s) → ${outSet.value.trim().toUpperCase()}`);
        bumpDataVersion();
      }
    } catch (e) {
      if (!disposed) status.textContent = `Failed: ${e}`;
    } finally {
      probeBtn.disabled = false;
      runBtn.disabled = false;
    }
  }
  probeBtn.addEventListener("click", () => void go(true));
  runBtn.addEventListener("click", () => void go(false));
  void refreshNames();

  return {
    el,
    dispose: () => {
      disposed = true;
      unsubWell();
      unsubData();
      scope.dispose();
    },
  };
}
