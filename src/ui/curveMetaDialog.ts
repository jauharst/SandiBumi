import {
  getCurveHeaderDisplayRange,
  setCurveHeaderDisplayRange,
  updateCurveMeta,
  type GenericCurveCatalogEntry,
  type GenericCurveInventoryEntry,
} from "../ipc";
import { setStatus } from "../state";
import { bumpDataVersion } from "../state";
import { pushUndo } from "../undo";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";
import { ensureSessionOperator } from "./runCustody";

/** Edit one imported curve's identity — name, unit, family — from the Wells pane.
 *
 *  This is metadata only: not a single sample moves, so it is exactly reversible and lands on
 *  the undo stack. It is NOT cosmetic though, and the dialog says so: modules resolve their
 *  inputs by mnemonic and then by family, so renaming GRN_CS to GR is what makes a module
 *  that wants GR actually read this curve. That is usually the point — a delivery whose
 *  mnemonics don't match the standard names is otherwise invisible to every module.
 */
export function openCurveMetaDialog(
  curve: GenericCurveInventoryEntry & Partial<Pick<GenericCurveCatalogEntry, "n_samples">>,
  onChanged: () => void,
): void {
  const content = document.createElement("div");

  const doc = document.createElement("p");
  doc.className = "modal-doc";
  doc.textContent =
    "Renames this curve where it is stored. No values change. Modules find their inputs by " +
    "mnemonic first and family second, so this also decides which module reads this curve — " +
    "renaming a delivery's GRN_CS to GR makes every GR-based module see it.";
  content.appendChild(doc);

  const mnemonic = document.createElement("input");
  mnemonic.className = "form-control";
  mnemonic.value = curve.mnemonic;
  mnemonic.spellcheck = false;
  content.appendChild(
    formRow("Name (mnemonic)", mnemonic, "Stored upper-cased, the way imports store them"),
  );

  const unit = document.createElement("input");
  unit.className = "form-control";
  unit.value = curve.unit ?? "";
  unit.spellcheck = false;
  unit.placeholder = "(none)";
  content.appendChild(
    formRow("Unit", unit, "Free text, verbatim from the file — e.g. GAPI, V/V, OHMM. Blank = no unit."),
  );

  const family = document.createElement("input");
  family.className = "form-control";
  family.value = curve.family ?? "";
  family.spellcheck = false;
  family.placeholder = "(none)";
  content.appendChild(
    formRow("Family", family, "The curve TYPE used as a fallback when no mnemonic matches — GR, RES, NPHI, RHOB, DT, SP…"),
  );

  const displayMin = document.createElement("input");
  displayMin.className = "form-control";
  displayMin.type = "number";
  displayMin.step = "any";
  displayMin.placeholder = "unset";
  displayMin.disabled = true;
  const displayMax = document.createElement("input");
  displayMax.className = "form-control";
  displayMax.type = "number";
  displayMax.step = "any";
  displayMax.placeholder = "unset";
  displayMax.disabled = true;
  const displayRange = document.createElement("div");
  displayRange.style.display = "grid";
  displayRange.style.gridTemplateColumns = "1fr auto 1fr";
  displayRange.style.gap = "6px";
  displayRange.style.alignItems = "center";
  displayRange.append(displayMin, document.createTextNode("→"), displayMax);
  content.appendChild(formRow(
    "Header display range",
    displayRange,
    "Optional display metadata for plots. It is not a scientific validity range and never filters samples.",
  ));

  const saveDisplay = document.createElement("button");
  saveDisplay.className = "btn";
  saveDisplay.textContent = "Save header range";
  saveDisplay.disabled = true;
  content.appendChild(saveDisplay);

  void getCurveHeaderDisplayRange(curve.curve_id)
    .then((range) => {
      displayMin.value = range ? String(range.low) : "";
      displayMax.value = range ? String(range.high) : "";
      displayMin.disabled = false;
      displayMax.disabled = false;
      saveDisplay.disabled = false;
    })
    .catch((error) => setStatus(`Curve header display range unavailable: ${error}`));

  const applyDisplayRange = (range: { low: number; high: number } | null): Promise<void> =>
    setCurveHeaderDisplayRange(curve.curve_id, range).then(() => {
      bumpDataVersion();
    });

  saveDisplay.addEventListener("click", () => {
    const lowText = displayMin.value.trim();
    const highText = displayMax.value.trim();
    if ((lowText === "") !== (highText === "")) {
      setStatus("Header display range needs both limits, or both fields blank to remove it");
      return;
    }
    const next = lowText === "" ? null : { low: Number(lowText), high: Number(highText) };
    if (next && (!Number.isFinite(next.low) || !Number.isFinite(next.high) || next.low === next.high)) {
      setStatus("Header display range needs two distinct finite limits");
      return;
    }
    saveDisplay.disabled = true;
    setCurveHeaderDisplayRange(curve.curve_id, next)
      .then((previous) => {
        bumpDataVersion();
        setStatus(next ? `Header display range saved for ${curve.mnemonic}` : `Header display range removed for ${curve.mnemonic}`);
        recordProcess("Curve", `${curve.mnemonic} header display range ${next ? `${next.low} → ${next.high}` : "removed"}`);
        pushUndo({
          label: `edit ${curve.mnemonic} header display range`,
          undo: () => applyDisplayRange(previous),
          redo: () => applyDisplayRange(next),
        });
      })
      .catch((error) => setStatus(`Header display range not saved: ${error}`))
      .finally(() => {
        saveDisplay.disabled = false;
      });
  });

  const note = document.createElement("p");
  note.className = "modal-doc";
  note.textContent = `Set ${curve.set_name}${curve.n_samples == null ? "" : ` • ${curve.n_samples} samples`}${
    curve.source ? ` • ${curve.source}` : ""
  }`;
  content.appendChild(note);

  const actions = document.createElement("div");
  actions.className = "form-actions";
  const save = document.createElement("button");
  save.className = "btn btn-accent";
  save.textContent = "Save";
  const cancel = document.createElement("button");
  cancel.className = "btn";
  cancel.textContent = "Cancel";
  actions.append(save, cancel);
  content.appendChild(actions);

  const close = openModal(`Edit curve — ${curve.mnemonic}`, content, 460);
  cancel.addEventListener("click", () => close());
  mnemonic.focus();
  mnemonic.select();

  save.addEventListener("click", () => {
    const nextName = mnemonic.value.trim();
    if (!nextName) {
      setStatus("A curve must keep a name");
      mnemonic.focus();
      return;
    }
    save.disabled = true;
    const apply = async (name: string, u: string | null, f: string | null): Promise<unknown> => {
      // SB-DBM-011: a rename/attribute edit is audited with the explicit session operator.
      const op = await ensureSessionOperator("Edit curve identity");
      if (!op) throw new Error("edit cancelled: no session operator entered");
      return updateCurveMeta(curve.curve_id, name, u, f, op, "Curve Catalog").then(() => {
        bumpDataVersion(); // log views, plots and module pickers re-resolve by name
        onChanged();
      });
    };

    apply(nextName, unit.value.trim() || null, family.value.trim() || null)
      .then(() => {
        const from = curve.mnemonic;
        setStatus(from === nextName.toUpperCase() ? `Curve ${from} updated` : `Curve ${from} renamed to ${nextName.toUpperCase()}`);
        recordProcess("Curve", `Edited curve ${from} → ${nextName.toUpperCase()} (set ${curve.set_name})`);
        // Exactly reversible: the previous identity is restored verbatim.
        pushUndo({
          label: `edit curve ${from}`,
          undo: () => apply(from, curve.unit, curve.family).then(() => {}),
          redo: () => apply(nextName, unit.value.trim() || null, family.value.trim() || null).then(() => {}),
        });
        close();
      })
      .catch((err) => {
        setStatus(`Curve edit failed: ${err}`);
        save.disabled = false;
      });
  });
}
