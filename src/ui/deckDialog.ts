import { save } from "@tauri-apps/plugin-dialog";
import { exportDeck, officeSupport, type OfficeSupport } from "../ipc";
import { appState, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { loadCutoffDefaults } from "./cutoffs";
import { formRow, openModal } from "./modal";
import { buildWellScope } from "./wellScope";

const FLAGS = [
  ["PAY", "PAY — sand, reservoir and hydrocarbon"],
  ["RESERVOIR", "RESERVOIR — sand and porosity"],
  ["SAND", "SAND — shale cutoff only"],
] as const;

/** Asset-team deck (Plot ▸ Deliverables ▸ Deck…).
 *
 *  The slides are built from the pay-summary DATA — matplotlib figures drawn from the same
 *  numbers the workbook and the report carry — deliberately NOT from composite log pages. A
 *  composite is drawn at a true print scale, and a picture on a slide stops being at that
 *  scale the moment anyone resizes it.
 *
 *  It needs python-pptx AND matplotlib; the dialog says which is missing before the save
 *  dialog appears, not after.
 */
export async function openDeckDialog(): Promise<void> {
  const support: OfficeSupport = await officeSupport().catch(() => ({
    python: null,
    xlsxwriter: false,
    docx: false,
    pptx: false,
    openpyxl: false,
    matplotlib: false,
  }));
  const ready = support.pptx && support.matplotlib;
  const cutoffs = await loadCutoffDefaults();

  const wrap = document.createElement("div");
  const close = openModal("Export deck (PowerPoint)", wrap, 620);

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";

  const scope = await buildWellScope({
    onChange: (ids) => {
      runBtn.textContent = `Export ${ids.length} well(s)…`;
      runBtn.disabled = ids.length === 0 || !ready;
    },
  });
  wrap.appendChild(scope.el);

  const activeWell = appState.selectedWell.get();
  const titleIn = document.createElement("input");
  titleIn.className = "form-control";
  titleIn.value = `Petrophysical Evaluation — ${activeWell?.field_name ?? activeWell?.well_name ?? "Field"}`;
  wrap.appendChild(formRow("Deck title", titleIn, "Shown on the title slide"));

  const authorIn = document.createElement("input");
  authorIn.className = "form-control";
  authorIn.placeholder = "optional";
  wrap.appendChild(formRow("Presented by", authorIn));

  const flagSel = document.createElement("select");
  flagSel.className = "form-control";
  for (const [value, label] of FLAGS) {
    const o = document.createElement("option");
    o.value = value;
    o.textContent = label;
    flagSel.appendChild(o);
  }
  wrap.appendChild(
    formRow("Summarise at", flagSel, "A deck speaks about one cutoff level and says which; the workbook carries all three"),
  );

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
  wrap.appendChild(formRow("VSH max (v/v)", vshIn, "Sand cutoff — the project default"));
  wrap.appendChild(formRow("PHIE min (v/v)", phieIn, "Reservoir cutoff"));
  wrap.appendChild(formRow("SWE max (v/v)", sweIn, "Pay cutoff"));
  wrap.appendChild(formRow("PERM min (mD)", permIn, "Optional — leave blank to not apply a permeability floor"));

  const hint = document.createElement("div");
  hint.className = "form-hint";
  hint.textContent =
    "Title, scope and cutoffs, field roll-up by zone, net and HPV per zone, N/G–PHIE–SWE distributions, " +
    "a well ranking, and any well that produced nothing, named. Box plots use the app's own statistics, " +
    "so they match the Field Dashboard. Composite log plots stay in the PDF. Nothing is written back to the project.";
  wrap.appendChild(hint);

  if (!ready) {
    const warn = document.createElement("div");
    warn.className = "form-hint";
    warn.style.color = "var(--warn)";
    const missing = [!support.pptx && "python-pptx", !support.matplotlib && "matplotlib"].filter(Boolean).join(" and ");
    warn.textContent = support.python
      ? `${missing} not installed in the Python SandiBumi found (${support.python}). Run: pip install ${missing.replace(" and ", " ")}`
      : "No Python was found. Install Python 3.10+ with python-pptx and matplotlib, or set SANDIBUMI_PYTHON to its python.exe.";
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
  runBtn.disabled = scope.count() === 0 || !ready;
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
        title: "Export deck",
        defaultPath: `${stem}.pptx`,
        filters: [{ name: "PowerPoint deck", extensions: ["pptx"] }],
      });
    } catch (err) {
      status.textContent = `Save dialog unavailable: ${err}`;
      return;
    }
    if (!dest) return;
    runBtn.disabled = true;
    status.textContent = `Computing ${wellIds.length} well(s)…`;
    try {
      const res = await exportDeck(
        {
          well_ids: wellIds,
          vsh_max: parseFloat(vshIn.value),
          phie_min: parseFloat(phieIn.value),
          swe_max: parseFloat(sweIn.value),
          perm_min: Number.isNaN(permRaw) ? null : permRaw,
          title: titleIn.value.trim(),
          author: authorIn.value.trim(),
          flag: flagSel.value,
        },
        dest,
      );
      const blind = res.wells - res.wells_with_results;
      const note = blind > 0 ? `, ${blind} well(s) not interpreted (named on the last slide)` : "";
      const msg = `Deck: ${res.slides} slide(s), ${res.wells_with_results}/${res.wells} wells${note} → ${res.path}`;
      status.textContent = msg;
      setStatus(msg);
      recordProcess("Export", `Exported deck (${res.slides} slides, ${res.wells} wells) → ${res.path}`);
      finish();
    } catch (err) {
      status.textContent = `Deck export failed: ${err}`;
      runBtn.disabled = false;
    }
  });
}
