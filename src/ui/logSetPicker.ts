import { listLogSetNames } from "../ipc";
import { formRow } from "./modal";

/** The ONE input/output log-set control, shared by every tool that reads or writes a curve.
 *
 *  Jauhar, 2026-08-05: *"each tools or modules should give user freedom to define input and output
 *  log set ... and their own curves"* — and, in the same breath, *"i forgot what set refer to in
 *  sandibumi"*. Both halves of that are addressed here.
 *
 *  **The vocabulary is now one word.** The store, the backend, `ROADMAP.md` and every other
 *  petrophysics package say **log set**; the UI alone said "constellation", abbreviated to "cons"
 *  on the two dialogs that had it. A user cannot map "Input cons" onto anything they have read
 *  about the project, which is exactly what happened. Nothing about the data model changed —
 *  `log_sets`, `input_set` and `output_set` were always the names underneath.
 *
 *  **One control rather than nineteen copies**, the `followCore.ts` argument: this is a single
 *  decision (which version do I read, which version do I write) and a copy per dialog is a place
 *  for the wording, the fallback rule and the refresh behaviour to drift. Before this, exactly two
 *  surfaces of nineteen offered it — the module dialog and the workflow builder — so ML, SandiMin,
 *  the saturation-height fit, the cutoff engine, the facies tie and every plot silently read
 *  whatever happened to be the current values, with no way to say "run this against FINAL" or
 *  "against what the vendor delivered".
 *
 *  Asymmetry worth keeping: the INPUT picker is a strict dropdown, because you can only read from
 *  a set that exists, while the OUTPUT is an editable combobox, because naming a new one is the
 *  ordinary act. A free-typed input would fall back to current values on a typo and the run would
 *  look like it had honoured the choice.
 */
export interface LogSetPicker {
  /** Rows to append into a form grid, in order. */
  rows: HTMLElement[];
  /** The chosen input set, or `undefined` for "current values". */
  inputSet: () => string | undefined;
  /** The chosen output set, or `undefined` when this picker is read-only. */
  outputSet: () => string | undefined;
  /** Re-read the project's set names (a run or a well switch can add one). */
  refresh: () => void;
  dispose: () => void;
}

export interface LogSetPickerOptions {
  /** Offer an input set. Default true — a tool that reads no curve should not build a picker. */
  read?: boolean;
  /** Output set default name, or `false` for a tool that writes nothing. */
  write?: string | false;
  /** Overrides the input hint for a tool whose fallback is worth spelling out differently. */
  readHint?: string;
  /** Overrides the output hint. */
  writeHint?: string;
  /** Prefills the input picker (a pane restoring its own state). */
  initialInput?: string;
  /** Called when the readable source set changes, so a condition/QC preflight can re-evaluate
   * against the same stored inputs the eventual operation will consume. */
  onInputChange?: () => void;
}

const READ_HINT =
  "Read inputs from this log set's stored values (latest version per well). Curves it never " +
  "wrote fall back to the usual sources. Blank = whatever the current values are.";
const WRITE_HINT =
  "Outputs are versioned into this log set — a re-run becomes version N+1 and never overwrites. " +
  "Pick an existing one or type a new name. Manage versions in the Curve Catalog.";

/** Shared datalist for every output combobox on the page. One node, refreshed by whichever
 *  pickers are alive — two panes open at once must not each append their own. */
const DATALIST_ID = "log-set-names";

function datalist(): HTMLDataListElement {
  let list = document.querySelector<HTMLDataListElement>(`#${DATALIST_ID}`);
  if (!list) {
    list = document.createElement("datalist");
    list.id = DATALIST_ID;
    document.body.appendChild(list);
  }
  return list;
}

export function buildLogSetPicker(opts: LogSetPickerOptions = {}): LogSetPicker {
  const wantRead = opts.read !== false;
  const wantWrite = opts.write !== false && opts.write !== undefined;
  const rows: HTMLElement[] = [];
  let disposed = false;

  // Input: strict dropdown. "(current values)" is a real choice, not an empty state, so it is
  // spelled out rather than left as a blank first option.
  const inSelect = document.createElement("select");
  inSelect.className = "form-control";
  if (wantRead) {
    const latest = document.createElement("option");
    latest.value = "";
    latest.textContent = "(current values)";
    inSelect.appendChild(latest);
    if (opts.initialInput) inSelect.value = opts.initialInput;
    inSelect.addEventListener("change", () => opts.onInputChange?.());
    rows.push(formRow("Input log set", inSelect, opts.readHint ?? READ_HINT));
  }

  // Output: editable combobox — an existing name or a new one.
  const outInput = document.createElement("input");
  outInput.className = "form-control";
  outInput.type = "text";
  if (wantWrite) {
    outInput.value = typeof opts.write === "string" ? opts.write : "INTERP";
    outInput.setAttribute("list", DATALIST_ID);
    rows.push(formRow("Output log set", outInput, opts.writeHint ?? WRITE_HINT));
  }

  const refresh = (): void => {
    if (disposed || (!wantRead && !wantWrite)) return;
    void listLogSetNames()
      .then((names) => {
        if (disposed) return;
        if (wantRead) {
          // Keep the user's choice across a refresh — a run finishing must not silently move a
          // pane back to "current values" after they deliberately picked a version.
          const keep = inSelect.value;
          while (inSelect.options.length > 1) inSelect.remove(1);
          for (const n of names) {
            const o = document.createElement("option");
            o.value = n;
            o.textContent = n;
            inSelect.appendChild(o);
          }
          if ([...inSelect.options].some((o) => o.value === keep)) inSelect.value = keep;
        }
        if (wantWrite) {
          const list = datalist();
          const seeds = [...new Set(["INTERP", "FINAL", "TEST", ...names])];
          list.innerHTML = "";
          for (const n of seeds) {
            const o = document.createElement("option");
            o.value = n;
            list.appendChild(o);
          }
        }
      })
      .catch(() => {});
  };
  refresh();

  return {
    rows,
    inputSet: () => (wantRead ? inSelect.value.trim() || undefined : undefined),
    outputSet: () => (wantWrite ? outInput.value.trim() || undefined : undefined),
    refresh,
    dispose: () => {
      disposed = true;
    },
  };
}
