import {
  listCurveCatalog,
  statsCurveSummary,
  statsFit,
  statsPairSummary,
  statsThickness,
  statsVersusSets,
  type CurveStatsRow,
  type FitResult,
  type PairStatsRow,
  type ThicknessCondition,
  type ThicknessRow,
  type VersusRow,
} from "../ipc";
import { recordProcess } from "../processLog";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow } from "./modal";
import { buildWellScope } from "./wellScope";

/** **Statistics** — the five table-producing tools in one pane (Jauhar, 2026-08-05).
 *
 *  A pane rather than a popup, per the standing rule: these are read beside a log view and worked
 *  through iteratively — pick a curve, read the table, change a zone, read it again — which is
 *  exactly what a popup covering the workspace prevents.
 *
 *  Tabs rather than five ribbon entries because they share almost all of their controls (a well
 *  scope, an input log set, a by-zone toggle) and the answer to one usually prompts the next: a
 *  Curve Summary showing a suspicious mean leads straight to a Pair Summary against the curve it
 *  should agree with.
 *
 *  **Every table renders a blank as a blank.** `null` from the backend means "there was no answer
 *  here" — a zone the well never entered, a correlation over too few pairs, a true vertical
 *  thickness on a well with no survey — and printing 0 there would be a number nobody measured.
 *  The `office.rs` rule, applied on screen. */
export async function buildStatisticsContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const scope = await buildWellScope();
  const catalog = await listCurveCatalog().catch(() => []);
  const curveNames = catalog.map((c) => c.name);

  const content = document.createElement("div");
  content.className = "module-pane stats-pane";

  const head = document.createElement("div");
  head.className = "module-head";
  const chip = document.createElement("span");
  chip.className = "module-chip";
  chip.textContent = "S";
  const titleEl = document.createElement("span");
  titleEl.className = "module-title";
  titleEl.textContent = "Statistics";
  head.append(chip, titleEl);
  content.appendChild(head);

  content.appendChild(scope.el);

  // --- Tabs -----------------------------------------------------------------
  type TabId = "curve" | "pair" | "fit" | "versus" | "thickness";
  const TABS: [TabId, string][] = [
    ["curve", "Curve Summary"],
    ["pair", "Pair Summary"],
    ["fit", "Fit"],
    ["versus", "Versus"],
    ["thickness", "Thickness"],
  ];
  let active: TabId = "curve";
  const seg = document.createElement("div");
  seg.className = "seg";
  const segBtns = new Map<TabId, HTMLButtonElement>();
  for (const [id, label] of TABS) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "seg-opt";
    b.textContent = label;
    b.addEventListener("click", () => {
      active = id;
      paint();
    });
    seg.appendChild(b);
    segBtns.set(id, b);
  }
  content.appendChild(seg);

  // --- Shared controls ------------------------------------------------------
  const shared = document.createElement("div");
  shared.className = "module-args";
  const setPicker = buildLogSetPicker({ write: false });
  for (const row of setPicker.rows) shared.appendChild(row);

  const byZone = document.createElement("input");
  byZone.type = "checkbox";
  byZone.checked = true;
  shared.appendChild(
    formRow(
      "Per zone",
      byZone,
      "A row per marker interval as well as the whole-well row. Off gives one row per well.",
    ),
  );
  content.appendChild(shared);

  /** Multi-select of curve mnemonics, sized so a handful is visible without scrolling. */
  const curveList = (size = 6): HTMLSelectElement => {
    const s = document.createElement("select");
    s.className = "form-control";
    s.multiple = true;
    s.size = size;
    for (const n of curveNames) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      s.appendChild(o);
    }
    return s;
  };
  const curvePick = (preferred: string[]): HTMLSelectElement => {
    const s = document.createElement("select");
    s.className = "form-control";
    for (const n of curveNames) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      s.appendChild(o);
    }
    const hit = preferred.find((p) => curveNames.includes(p));
    if (hit) s.value = hit;
    return s;
  };
  const picked = (s: HTMLSelectElement): string[] =>
    [...s.selectedOptions].map((o) => o.value);

  const controls = document.createElement("div");
  controls.className = "module-args";
  content.appendChild(controls);

  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.className = "btn btn-accent form-run-btn";
  runRow.appendChild(runBtn);
  content.appendChild(runRow);

  const note = document.createElement("div");
  note.className = "module-status";
  content.appendChild(note);

  const tableWrap = document.createElement("div");
  tableWrap.className = "stats-table-wrap";
  content.appendChild(tableWrap);

  // --- Table rendering ------------------------------------------------------
  /** `null`/`undefined` renders as an EMPTY cell, never 0 — see the module doc. */
  const num = (v: number | null | undefined, dp = 3): string =>
    v === null || v === undefined || !Number.isFinite(v) ? "" : v.toFixed(dp);

  const renderTable = (headers: string[], rows: (string | number | null)[][]): void => {
    tableWrap.innerHTML = "";
    if (rows.length === 0) {
      const empty = document.createElement("div");
      empty.className = "module-status";
      empty.textContent = "No rows — nothing in scope answered this.";
      tableWrap.appendChild(empty);
      return;
    }
    const table = document.createElement("table");
    table.className = "stats-table";
    const thead = document.createElement("thead");
    const hr = document.createElement("tr");
    for (const h of headers) {
      const th = document.createElement("th");
      th.textContent = h;
      hr.appendChild(th);
    }
    thead.appendChild(hr);
    table.appendChild(thead);
    const tbody = document.createElement("tbody");
    for (const r of rows) {
      const tr = document.createElement("tr");
      for (const c of r) {
        const td = document.createElement("td");
        td.textContent = c === null || c === undefined ? "" : String(c);
        tr.appendChild(td);
      }
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);
    tableWrap.appendChild(table);

    // CSV export — the table is the deliverable, and a table you cannot get out of the app is a
    // table you retype. Rendered from the SAME rows, so the file and the screen cannot disagree.
    const csvBtn = document.createElement("button");
    csvBtn.type = "button";
    csvBtn.className = "btn";
    csvBtn.textContent = "Copy as CSV";
    csvBtn.addEventListener("click", () => {
      const esc = (v: unknown): string => {
        const s = v === null || v === undefined ? "" : String(v);
        return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
      };
      const csv = [headers.map(esc).join(","), ...rows.map((r) => r.map(esc).join(","))].join("\n");
      void navigator.clipboard.writeText(csv);
      setStatus(`Copied ${rows.length} row(s) as CSV`);
    });
    tableWrap.appendChild(csvBtn);
  };

  // --- Per-tab controls -----------------------------------------------------
  const curveMulti = curveList();
  const pctIn = document.createElement("input");
  pctIn.className = "form-control";
  pctIn.type = "text";
  pctIn.value = "10, 50, 90";

  const pairX = curvePick(["PHIE", "NPHI"]);
  const pairY = curvePick(["CPOR", "PHIT"]);

  const fitTarget = curvePick(["PERM", "CPERM"]);
  const fitPreds = curveList(5);
  const logTarget = document.createElement("input");
  logTarget.type = "checkbox";
  logTarget.checked = true;
  const logPreds = document.createElement("input");
  logPreds.type = "checkbox";

  const versusA = document.createElement("input");
  versusA.className = "form-control";
  versusA.type = "text";
  versusA.placeholder = "e.g. FINAL";
  const versusCurves = curveList();

  const thickMode = document.createElement("select");
  thickMode.className = "form-control";
  for (const [v, l] of [
    ["FLAG", "FLAG — a 0/1 curve (FLAG_PAY, FLAG_SAND, one you wrote)"],
    ["CLASS", "CLASS — split by a discrete curve (FACIES, rock type)"],
    ["CUTOFF", "CUTOFF — where conditions you type all hold"],
    ["MARKER", "MARKER — gross between tops, no curve needed"],
  ] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = l;
    thickMode.appendChild(o);
  }
  const thickCurve = curvePick(["FLAG_PAY", "FACIES"]);
  const condCurve = curvePick(["PHIE"]);
  const condOp = document.createElement("select");
  condOp.className = "form-control";
  for (const op of [">=", "<=", ">", "<", "=="]) {
    const o = document.createElement("option");
    o.value = op;
    o.textContent = op;
    condOp.appendChild(o);
  }
  const condVal = document.createElement("input");
  condVal.className = "form-control";
  condVal.type = "number";
  condVal.step = "any";
  condVal.value = "0.1";

  function paint(): void {
    for (const [id, btn] of segBtns) btn.classList.toggle("active", id === active);
    controls.innerHTML = "";
    note.textContent = "";
    switch (active) {
      case "curve":
        controls.appendChild(formRow("Curves", curveMulti, "Ctrl-click for several."));
        controls.appendChild(
          formRow("Percentiles", pctIn, "Comma separated, 0–100. Blank uses P10, P50, P90."),
        );
        runBtn.textContent = "Run Curve Summary";
        break;
      case "pair":
        controls.appendChild(formRow("X curve", pairX));
        controls.appendChild(formRow("Y curve", pairY));
        note.textContent =
          "Pearson answers “is this a straight line”, which is the right question only " +
          "when both axes are the same quantity. Spearman answers “do they move together” " +
          "and is the one to read for two different measurements.";
        runBtn.textContent = "Run Pair Summary";
        break;
      case "fit":
        controls.appendChild(formRow("Target", fitTarget));
        controls.appendChild(formRow("Predictors", fitPreds, "Ctrl-click for several."));
        controls.appendChild(formRow("Fit log10(target)", logTarget, "The usual form for permeability."));
        controls.appendChild(formRow("Fit log10(predictors)", logPreds));
        note.textContent =
          "Scored by leave-one-WELL-out, never leave-one-sample-out: neighbouring log samples are " +
          "nearly identical, so a sample-wise split scores the fit on data it has already seen.";
        runBtn.textContent = "Run Fit";
        break;
      case "versus":
        controls.appendChild(
          formRow("Reference log set", versusA, "What you had. Compared against the Input log set above (blank = current values)."),
        );
        controls.appendChild(formRow("Curves", versusCurves, "Ctrl-click for several."));
        runBtn.textContent = "Run Versus";
        break;
      case "thickness":
        controls.appendChild(formRow("Count", thickMode));
        if (thickMode.value === "FLAG" || thickMode.value === "CLASS") {
          controls.appendChild(formRow("Curve", thickCurve));
        }
        if (thickMode.value === "CUTOFF") {
          const wrap = document.createElement("div");
          wrap.className = "ribbon-btn-row";
          wrap.append(condCurve, condOp, condVal);
          controls.appendChild(formRow("Condition", wrap));
        }
        note.textContent =
          "True vertical thickness is reported wherever the well has a TVD curve, and left BLANK " +
          "where it does not — a vertical well and an unsurveyed deviated one look identical " +
          "in the data. Asked for pay, this counts the FLAG_PAY curve the cutoff engine wrote " +
          "rather than re-applying cutoffs, so the project never holds two net-pay numbers.";
        runBtn.textContent = "Run Thickness";
        break;
    }
  }
  thickMode.addEventListener("change", paint);

  // --- Run ------------------------------------------------------------------
  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      note.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return;
    }
    runBtn.disabled = true;
    const common = { input_set: setPicker.inputSet(), by_zone: byZone.checked };
    try {
      if (active === "curve") {
        const curves = picked(curveMulti);
        if (curves.length === 0) {
          note.textContent = "Pick at least one curve.";
          return;
        }
        const pcts = pctIn.value
          .split(",")
          .map((s) => parseFloat(s.trim()))
          .filter((v) => Number.isFinite(v) && v >= 0 && v <= 100);
        const [rows, used] = await statsCurveSummary({ well_ids: wellIds, curves, percentiles: pcts, ...common });
        renderTable(
          ["Well", "Zone", "Curve", "n", "Missing", "Min", "Max", "Mean", "Std", ...used.map((p) => `P${p}`)],
          rows.map((r: CurveStatsRow) => [
            r.well, r.zone, r.curve, r.n, r.n_missing,
            num(r.min), num(r.max), num(r.mean), num(r.std),
            ...r.percentiles.map((p) => num(p)),
          ]),
        );
        setStatus(`Curve summary: ${rows.length} row(s)`);
        recordProcess("Statistics", `Curve summary over ${curves.length} curve(s), ${wellIds.length} well(s)`);
      } else if (active === "pair") {
        const rows = await statsPairSummary({
          well_ids: wellIds, x_curve: pairX.value, y_curve: pairY.value, ...common,
        });
        renderTable(
          ["Well", "Zone", "n pairs", "Pearson", "Spearman", "Bias", "RMS diff", "Slope", "Intercept"],
          rows.map((r: PairStatsRow) => [
            r.well, r.zone, r.n, num(r.pearson), num(r.spearman),
            num(r.bias), num(r.rms_diff), num(r.slope), num(r.intercept),
          ]),
        );
        setStatus(`Pair summary: ${rows.length} row(s)`);
      } else if (active === "fit") {
        const preds = picked(fitPreds);
        if (preds.length === 0) {
          note.textContent = "Pick at least one predictor.";
          return;
        }
        const res: FitResult = await statsFit({
          well_ids: wellIds, predictors: preds, target: fitTarget.value,
          log_target: logTarget.checked, log_predictors: logPreds.checked,
          input_set: setPicker.inputSet(),
        });
        const terms = res.predictors.map((p, i) => `${num(res.coefficients[i + 1], 5)} x ${logPreds.checked ? `log10(${p})` : p}`);
        renderTable(
          ["", "Value"],
          [
            ["Equation", `${logTarget.checked ? `log10(${fitTarget.value})` : fitTarget.value} = ${num(res.coefficients[0], 5)} + ${terms.join(" + ")}`],
            ["Samples", res.n],
            ["R² (on its own data)", num(res.r2, 4)],
            // The blind figure is the one to quote; a null is a stated absence, not a zero.
            ["R² (blind well)", res.r2_blind === null ? "needs 3+ wells" : num(res.r2_blind, 4)],
            ["RMS", num(res.rms, 5)],
            ["Wells used", res.wells_used.join(", ")],
          ],
        );
        note.textContent = res.notes.join(" • ");
        setStatus(`Fit on ${res.n} samples`);
        recordProcess("Statistics", `Fit ${fitTarget.value} on ${preds.join(", ")}`);
      } else if (active === "versus") {
        const curves = picked(versusCurves);
        if (curves.length === 0 || !versusA.value.trim()) {
          note.textContent = "Name a reference log set and pick at least one curve.";
          return;
        }
        const rows = await statsVersusSets({
          well_ids: wellIds, curves, set_a: versusA.value.trim(), set_b: setPicker.inputSet(),
        });
        renderTable(
          ["Well", "Curve", "Both", "Only reference", "Only this", "Changed", "Mean diff", "Max |diff|"],
          rows.map((r: VersusRow) => [
            r.well, r.curve, r.n_common, r.only_a, r.only_b, r.n_changed,
            num(r.mean_diff, 5), num(r.max_abs_diff, 5),
          ]),
        );
        setStatus(`Versus: ${rows.length} row(s)`);
      } else {
        const mode = thickMode.value as "FLAG" | "CLASS" | "CUTOFF" | "MARKER";
        const conditions: ThicknessCondition[] =
          mode === "CUTOFF"
            ? [{ curve: condCurve.value, op: condOp.value as ">=", value: parseFloat(condVal.value) }]
            : [];
        const rows = await statsThickness({
          well_ids: wellIds, mode,
          curve: mode === "FLAG" || mode === "CLASS" ? thickCurve.value : null,
          conditions, ...common,
        });
        renderTable(
          ["Well", "Zone", "Item", "n", "Gross MD", "Net MD", "Gross TVD", "Net TVD", "N/G"],
          rows.map((r: ThicknessRow) => [
            r.well, r.zone, r.item, r.n,
            num(r.gross_md, 2), num(r.net_md, 2), num(r.gross_tvd, 2), num(r.net_tvd, 2), num(r.ntg, 3),
          ]),
        );
        setStatus(`Thickness: ${rows.length} row(s)`);
        recordProcess("Statistics", `Thickness by ${mode} over ${wellIds.length} well(s)`);
      }
    } catch (e) {
      note.textContent = String(e);
    } finally {
      runBtn.disabled = false;
    }
  });

  paint();
  return { el: content, dispose: () => setPicker.dispose() };
}
