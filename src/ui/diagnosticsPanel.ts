import { save } from "@tauri-apps/plugin-dialog";
import { diagnosticReport, saveDiagnosticReport } from "../ipc";
import { appState } from "../state";
import { recordProcess } from "../processLog";

/** The one thing a user sends when they report "it was slow" or "the numbers look wrong".
 *
 *  A pane rather than a popup, like its Monitor neighbours History / Processing / Performance —
 *  a support call is worked THROUGH, with the report open beside the log view that looks wrong.
 *
 *  Nothing here transmits anything. The report is built on demand, shown in full so it can be
 *  read before it is sent, and saved only where the user points a save dialog. What travels and
 *  what is redacted is decided in `src-tauri/src/diagnostics.rs`; this pane's job is to make sure
 *  nobody sends it without having seen it.
 *
 *  The sensitivity card is the load-bearing part of the layout. Jauhar's call was that PARAMETER
 *  VALUES travel — without m, n, a, Rw and the cut-offs there is usually no way to say why a
 *  number looks wrong — and the consequence is that this file carries a client's own calibration.
 *  So the caution sits ABOVE the report, where somebody reaching for the Save button reads it,
 *  not in a tooltip. */
export async function buildDiagnosticsContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const wrap = document.createElement("div");
  wrap.className = "module-pane";

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "One file describing what this copy of SandiBumi did — how long the project took to open, " +
    "how long each operation ran, anything that went wrong, and optionally how one well's curves " +
    "were computed. Read it, then send it. Nothing is transmitted from here.";
  wrap.appendChild(intro);

  // Reuses the "hidden cost" card rather than a new class: it is the same job — a real
  // consequence of a choice, stated beside the control that acts on it.
  const caution = document.createElement("div");
  caution.className = "module-contamination";
  const cautionTitle = document.createElement("div");
  cautionTitle.className = "module-contamination-title";
  cautionTitle.textContent = "Check this before you send it";
  const cautionBody = document.createElement("div");
  cautionBody.className = "module-contamination-body";
  for (const line of [
    "It contains PARAMETER VALUES — the m, n, a, Rw, cut-offs and endpoints behind the curves. " +
      "That is analytical work product and may be covered by your confidentiality agreement.",
    "It contains no well names, no field names, no file paths and no curve values. Wells appear " +
      "as WELL-1, WELL-2 and so on.",
  ]) {
    const p = document.createElement("div");
    p.textContent = line;
    cautionBody.appendChild(p);
  }
  caution.append(cautionTitle, cautionBody);
  wrap.appendChild(caution);

  // ---- what to include ----------------------------------------------------
  // `.field-checkbox`, not `.field-label` — a checkbox belongs BESIDE its text, and the small-caps
  // field label is styled to sit above the control it names.
  const provLabel = document.createElement("label");
  provLabel.className = "field-checkbox diagnostics-option";
  const provBox = document.createElement("input");
  provBox.type = "checkbox";
  provBox.checked = true;
  const provText = document.createElement("span");
  provLabel.append(provBox, provText);
  wrap.appendChild(provLabel);

  /** The provenance section is scoped to the SELECTED well — on a 2000-well project the whole
   *  field would be the long half of the report, and a support call is about one well's numbers. */
  const selectedWellId = (): string | null => appState.selectedWell.get()?.well_id ?? null;
  const syncProvLabel = (): void => {
    const well = appState.selectedWell.get();
    provBox.disabled = !well;
    provText.textContent = well
      ? "Include how the selected well's curves were computed (module, inputs, parameter values)"
      : "Select a well in Wells & Tops to include how its curves were computed";
  };
  // `subscribe` fires immediately with the current value, so this also does the initial render.
  const unsubscribe = appState.selectedWell.subscribe(() => syncProvLabel());

  // ---- actions ------------------------------------------------------------
  // Left-aligned: this is a pane, not a dialog. `.guard-confirm-row` pushes its buttons to the
  // right edge, which on a wide pane leaves the primary action far from what it acts on.
  const row = document.createElement("div");
  row.className = "diagnostics-actions";
  const buildBtn = document.createElement("button");
  buildBtn.type = "button";
  buildBtn.className = "btn btn-accent";
  buildBtn.textContent = "Build report";
  const saveBtn = document.createElement("button");
  saveBtn.type = "button";
  saveBtn.className = "btn";
  saveBtn.textContent = "Save…";
  saveBtn.disabled = true;
  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.className = "btn";
  copyBtn.textContent = "Copy";
  copyBtn.disabled = true;
  row.append(buildBtn, saveBtn, copyBtn);
  wrap.appendChild(row);

  const status = document.createElement("div");
  status.className = "eq-note diagnostics-status";
  wrap.appendChild(status);

  const view = document.createElement("pre");
  view.className = "diagnostics-report";
  view.textContent = "";
  wrap.appendChild(view);

  let report = "";
  const setReport = (text: string): void => {
    report = text;
    view.textContent = text;
    saveBtn.disabled = !text;
    copyBtn.disabled = !text;
  };

  buildBtn.addEventListener("click", () => {
    void (async () => {
      buildBtn.disabled = true;
      status.textContent = "Building…";
      try {
        const wellId = provBox.checked ? selectedWellId() : null;
        setReport(await diagnosticReport(wellId));
        status.textContent = "Built. Read it, then Save or Copy.";
      } catch (err) {
        setReport("");
        status.textContent = `Could not build the report: ${err}`;
      } finally {
        buildBtn.disabled = false;
      }
    })();
  });

  saveBtn.addEventListener("click", () => {
    void (async () => {
      let dest: string | null;
      try {
        dest = await save({
          // No well or project name in the filename either — the whole point is that the file
          // can be sent, and a filename travels with it.
          defaultPath: "sandibumi-diagnostics.txt",
          filters: [{ name: "Text", extensions: ["txt"] }],
        });
      } catch (err) {
        status.textContent = `Save dialog unavailable: ${err}`;
        return;
      }
      if (!dest) return;
      try {
        await saveDiagnosticReport(dest, report);
        status.textContent = "Saved.";
        setStatus("Diagnostic report saved.");
        recordProcess("Diagnostics", "Saved a diagnostic report");
      } catch (err) {
        status.textContent = `Could not save: ${err}`;
      }
    })();
  });

  copyBtn.addEventListener("click", () => {
    void navigator.clipboard
      .writeText(report)
      .then(() => {
        status.textContent = "Copied to the clipboard.";
      })
      .catch((err) => {
        status.textContent = `Could not copy: ${err}`;
      });
  });

  return { el: wrap, dispose: () => unsubscribe() };
}
