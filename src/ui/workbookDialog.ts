import { save } from "@tauri-apps/plugin-dialog";
import { exportWorkbook, officeSupport, type OfficeSupport } from "../ipc";
import { appState, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { loadCutoffDefaults } from "./cutoffs";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow, openModal } from "./modal";
import { PARAM_SOURCE_TOPICS, withParamSources } from "./paramSources";
import { buildWellScope } from "./wellScope";

/** Excel workbook export (Plot ▸ Deliverables ▸ Workbook…).
 *
 *  The gap this closes: everything a finished study produces left the app as a PDF, an SVG,
 *  a LAS or a flat CSV, so the table an asset team actually works in was re-typed by hand.
 *
 *  Two things the dialog is careful about:
 *
 *  • It uses the SAME cutoff defaults as the pay summary, the report and Monte Carlo
 *    (`loadCutoffDefaults`), so a workbook can never quote different cutoffs than the PDF
 *    handed over with it.
 *  • The workbook is written by Python's `xlsxwriter` (rule 7 — subprocess, never embedded).
 *    If it is not installed the dialog says so BEFORE the user picks a filename, and names
 *    the interpreter to install it into, rather than failing after the save dialog.
 */
export async function openWorkbookDialog(): Promise<void> {
  const support: OfficeSupport = await officeSupport().catch(() => ({
    python: null,
    xlsxwriter: false,
    docx: false,
    pptx: false,
    openpyxl: false,
    pillow: false,
    matplotlib: false,
    messages: {},
    package_versions: {},
    probe_error: "Office prerequisite probe failed.",
  }));
  const cutoffs = await loadCutoffDefaults();

  const wrap = document.createElement("div");
  const close = openModal("Export workbook (Excel)", wrap, 620);

  // Built before the scope so the scope's live count can retitle it from its first callback.
  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";

  const scope = await buildWellScope({
    onChange: (ids) => {
      runBtn.textContent = `Export ${ids.length} well(s)…`;
      runBtn.disabled = ids.length === 0 || !support.xlsxwriter;
    },
  });
  wrap.appendChild(scope.el);

  const titleIn = document.createElement("input");
  titleIn.className = "form-control";
  const activeWell = appState.selectedWell.get();
  titleIn.value = `Petrophysical Evaluation — ${activeWell?.field_name ?? activeWell?.well_name ?? "Field"}`;
  wrap.appendChild(formRow("Study title", titleIn, "Shown on the workbook's Summary sheet"));

  const numIn = (value: number | null, step: string, placeholder = ""): HTMLInputElement => {
    const el = document.createElement("input");
    el.className = "form-control";
    el.type = "number";
    el.step = step;
    el.placeholder = placeholder;
    el.value = value === null ? "" : String(value);
    return el;
  };
  const vshIn = numIn(cutoffs.vsh_max, "0.01");
  const phieIn = numIn(cutoffs.phie_min, "0.01");
  const sweIn = numIn(cutoffs.swe_max, "0.01");
  const permIn = numIn(cutoffs.perm_min, "0.1", "off");
  wrap.appendChild(formRow("VSH max (v/v)", withParamSources(vshIn, PARAM_SOURCE_TOPICS.cutoffVshMax), "Sand cutoff — the project default"));
  wrap.appendChild(formRow("PHIE min (v/v)", withParamSources(phieIn, PARAM_SOURCE_TOPICS.cutoffPhieMin), "Reservoir cutoff"));
  wrap.appendChild(formRow("SWE max (v/v)", withParamSources(sweIn, PARAM_SOURCE_TOPICS.cutoffSweMax), "Pay cutoff"));
  wrap.appendChild(formRow("PERM min (mD)", permIn, "Optional — leave blank to not apply a permeability floor"));
  // --- Input log set (`logSetPicker.ts`): which VERSION of the curves this reads.
  const setPicker = buildLogSetPicker({ write: false });
  for (const row of setPicker.rows) wrap.appendChild(row);


  const check = (label: string, hint: string): HTMLInputElement => {
    const el = document.createElement("input");
    el.type = "checkbox";
    el.className = "form-check";
    el.checked = true;
    const holder = document.createElement("label");
    holder.appendChild(el);
    wrap.appendChild(formRow(label, holder, hint));
    return el;
  };
  const paySheet = check("Pay Summary sheet", "One row per well, zone and cutoff level — the same table the report PDF prints");
  const fieldSheet = check("Field Summary sheet", "Per-zone roll-up across every well in scope");
  const zoneSheet = check("Zone Parameters sheet", "The interval parameters each interpretation used");

  const hint = document.createElement("div");
  hint.className = "form-hint";
  hint.textContent =
    "Numbers are exported as numbers, so the workbook can be pivoted and re-averaged. " +
    "A blank cell means the well was not interpreted over that zone — it is not a zero. " +
    "Nothing is written back to the project: the pay flags are computed in memory only.";
  wrap.appendChild(hint);

  if (!support.xlsxwriter) {
    const warn = document.createElement("div");
    warn.className = "form-hint";
    warn.style.color = "var(--warn)";
    warn.textContent =
      support.messages.workbook_export ?? support.probe_error ?? "Workbook prerequisite probe failed.";
    wrap.appendChild(warn);
  }

  const status = document.createElement("div");
  status.className = "form-hint";
  wrap.appendChild(status);

  const actions = document.createElement("div");
  actions.className = "form-actions";
  const cancelBtn = document.createElement("button");
  cancelBtn.className = "btn";
  cancelBtn.textContent = "Cancel";
  runBtn.textContent = `Export ${scope.count()} well(s)…`;
  runBtn.disabled = scope.count() === 0 || !support.xlsxwriter;
  actions.append(cancelBtn, runBtn);
  wrap.appendChild(actions);

  const finish = () => {
    scope.dispose();
    close();
  };
  cancelBtn.addEventListener("click", finish);

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) return;
    const permRaw = parseFloat(permIn.value);
    const stem = (titleIn.value.trim() || "petrophysics").replace(/[^\w.-]+/g, "_").slice(0, 60);
    let dest: string | null;
    try {
      dest = await save({
        title: "Export workbook",
        defaultPath: `${stem}.xlsx`,
        filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
      });
    } catch (err) {
      status.textContent = `Save dialog unavailable: ${err}`;
      return;
    }
    if (!dest) return;
    runBtn.disabled = true;
    status.textContent = `Computing ${wellIds.length} well(s)…`;
    try {
      const res = await exportWorkbook(
        {
          well_ids: wellIds,
          vsh_max: parseFloat(vshIn.value),
          phie_min: parseFloat(phieIn.value),
          swe_max: parseFloat(sweIn.value),
          perm_min: Number.isNaN(permRaw) ? null : permRaw,
          input_set: setPicker.inputSet(),
          title: titleIn.value.trim(),
          include_pay: paySheet.checked,
          include_field: fieldSheet.checked,
          include_zone_params: zoneSheet.checked,
        },
        dest,
      );
      // The blind-well count is stated, never swallowed: a workbook whose Summary says
      // "12 of 40 wells with results" is a finding, not a formatting detail.
      const blind = res.wells - res.wells_with_results;
      const note = blind > 0 ? `, ${blind} well(s) not interpreted (named on the Summary sheet)` : "";
      const msg = `Workbook: ${res.sheets} sheet(s), ${res.pay_rows} zone-rows, ${res.wells_with_results}/${res.wells} wells${note} → ${res.path}`;
      status.textContent = msg;
      setStatus(msg);
      recordProcess("Export", `Exported workbook (${res.pay_rows} zone-rows, ${res.wells} wells) → ${res.path}`);
      finish();
    } catch (err) {
      status.textContent = `Workbook export failed: ${err}`;
      runBtn.disabled = false;
    }
  });
}
