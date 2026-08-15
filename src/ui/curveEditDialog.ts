import { editCurve, restoreCurveValues, type CurveEditRequest } from "../ipc";
import { bumpDataVersion, setStatus } from "../state";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { requestRunCustody } from "./runCustody";

const OPS: { id: CurveEditRequest["op"]; label: string; hint: string }[] = [
  { id: "shift", label: "Wireline shift", hint: "Move the whole curve in depth (resampled onto its own grid; + is down hole)" },
  { id: "set", label: "Set constant", hint: "Write one value over the interval" },
  { id: "blank", label: "Blank (erase)", hint: "Set the interval to missing (NaN)" },
  { id: "interpolate", label: "Interpolate across", hint: "Bridge the interval linearly between its edges (gap fill / despike)" },
  { id: "scale", label: "Scale a·v + b", hint: "Linear recalibration over the interval" },
];

/** Right-click → "Edit CURVE…" from a log-view track: manual log editing with the
 *  standard ops. Every apply is one undoable action — the backend returns the changed
 *  samples' previous values and Ctrl+Z writes them back bit-exactly. */
export function openCurveEditDialog(wellId: string, wellName: string, curveName: string, clickedDepth: number): void {
  const content = document.createElement("div");

  const opSel = document.createElement("select");
  opSel.className = "form-control";
  for (const op of OPS) {
    const opt = document.createElement("option");
    opt.value = op.id;
    opt.textContent = op.label;
    opSel.appendChild(opt);
  }

  const num = (value: number, step = 0.1): HTMLInputElement => {
    const input = document.createElement("input");
    input.className = "form-control";
    input.type = "number";
    input.step = String(step);
    input.value = String(Math.round(value * 100) / 100);
    return input;
  };
  const deltaInput = num(0.5);
  const topInput = num(clickedDepth - 1);
  const bottomInput = num(clickedDepth + 1);
  const valueInput = num(0);
  const mulInput = num(1, 0.01);
  const addInput = num(0, 0.01);

  const hintEl = document.createElement("div");
  hintEl.className = "form-hint";

  const rows = {
    delta: formRow("Shift (m)", deltaInput, "Positive moves the curve deeper"),
    top: formRow("Top (m)", topInput),
    bottom: formRow("Bottom (m)", bottomInput),
    value: formRow("Value", valueInput),
    mul: formRow("Multiplier a", mulInput),
    add: formRow("Offset b", addInput),
  };
  content.appendChild(formRow("Operation", opSel));
  content.appendChild(hintEl);
  for (const row of Object.values(rows)) content.appendChild(row);

  const VISIBLE: Record<CurveEditRequest["op"], (keyof typeof rows)[]> = {
    shift: ["delta"],
    set: ["top", "bottom", "value"],
    blank: ["top", "bottom"],
    interpolate: ["top", "bottom"],
    scale: ["top", "bottom", "mul", "add"],
  };
  const syncVisibility = () => {
    const op = opSel.value as CurveEditRequest["op"];
    hintEl.textContent = OPS.find((o) => o.id === op)?.hint ?? "";
    const visible = new Set<string>(VISIBLE[op]);
    for (const [key, row] of Object.entries(rows)) row.hidden = !visible.has(key);
  };
  opSel.addEventListener("change", syncVisibility);
  syncVisibility();

  // Refusing a bad field HERE rather than in the status bar, for the `needWell.ts` reason: the
  // user is looking at this dialog, and a message in a corner of the window is one they will not
  // read before clicking Apply again.
  const errEl = document.createElement("div");
  errEl.className = "form-hint";
  errEl.style.color = "var(--warn)";
  errEl.style.display = "none"; // not the `hidden` attribute — a `display` rule would beat it
  content.appendChild(errEl);

  const applyBtn = document.createElement("button");
  applyBtn.className = "lp-btn";
  applyBtn.textContent = "Apply";
  content.appendChild(applyBtn);

  const close = openModal(`Edit ${curveName} — ${wellName}`, content, 420);

  applyBtn.addEventListener("click", () => {
    void (async () => {
      const op = opSel.value as CurveEditRequest["op"];
      // `x || 0` lets a non-finite through (Infinity is truthy — "1e999" or "Infinity" would set
      // the constant to +Inf and poison catalog min/max + plot autoscale). Require finite.
      //
      // But narrowing to finite only fixed the Infinity half, and the surviving half is worse:
      // an empty or unparseable field falls back to the default, and for three of these fields
      // 0 is NOT the identity (`docs/review_triage.md` finding 19). `1e999` stopped writing +Inf
      // and started writing 0.0 — a perfectly finite number that clears the backend's own guard
      // and lands on the log as a real reading. 0.0 gAPI over an interval does not look like an
      // error; it looks like a measurement of very clean rock. An empty Top is the same trap by a
      // different route: it does not mean "no interval", it means from surface.
      //
      // `mul` (1) and `add` (0) keep the fallback — there the default really is the identity, so
      // the worst case is an op that does nothing and says so.
      const num = (s: string, dflt = 0): number => {
        const v = parseFloat(s);
        return Number.isFinite(v) ? v : dflt;
      };
      const REQUIRED: Partial<Record<keyof typeof rows, [HTMLInputElement, string]>> = {
        top: [topInput, "Top (m)"],
        bottom: [bottomInput, "Bottom (m)"],
        value: [valueInput, "Value"],
      };
      const missing = VISIBLE[op]
        .map((k) => REQUIRED[k])
        .filter((f): f is [HTMLInputElement, string] => !!f && !Number.isFinite(parseFloat(f[0].value)));
      if (missing.length > 0) {
        const labels = missing.map(([, label]) => label);
        errEl.textContent = `${labels.join(" and ")} need${labels.length > 1 ? "" : "s"} a number — nothing was written.`;
        errEl.style.display = "";
        missing[0][0].focus();
        return;
      }
      errEl.style.display = "none";
      const custody = await requestRunCustody(`Apply ${curveName} edit`);
      if (!custody) return;
      const req: CurveEditRequest = {
        well_id: wellId,
        curve: curveName,
        op,
        delta: num(deltaInput.value),
        top: num(topInput.value),
        bottom: num(bottomInput.value),
        value: num(valueInput.value),
        mul: num(mulInput.value, 1),
        add: num(addInput.value),
        custody,
      };
      applyBtn.disabled = true;
      try {
        const res = await editCurve(req);
        if (res.affected === 0) {
          setStatus(`${curveName}: nothing changed`);
          return;
        }
        // The packed previous-sample bytes ARE the undo payload; redo re-runs the
        // same deterministic edit on the restored curve.
        const prevBytes = Array.from(res.data instanceof Uint8Array ? res.data : new Uint8Array(res.data));
        const pointCount = res.point_count;
        const opLabel = OPS.find((o) => o.id === op)?.label ?? op;
        const label = `${opLabel} ${curveName} (${wellName})`;
        const undoCustody = {
          actor: custody.actor,
          source_note: `Undo of prior curve edit; original source/reference: ${custody.source_note}`,
        };
        let undoVersion = {
          editId: res.edit_id,
          curveSha256: res.curve_sha256,
        };
        pushUndo({
          label,
          undo: async () => {
            await restoreCurveValues(
              wellId,
              curveName,
              pointCount,
              prevBytes,
              undoVersion.editId,
              undoVersion.curveSha256,
              undoCustody,
            );
            bumpDataVersion();
          },
          redo: async () => {
            const reapplied = await editCurve(req);
            if (reapplied.affected === 0 || !reapplied.edit_id) {
              throw new Error("redo did not recreate the curve edit");
            }
            undoVersion = {
              editId: reapplied.edit_id,
              curveSha256: reapplied.curve_sha256,
            };
            bumpDataVersion();
          },
        });
        bumpDataVersion();
        setStatus(`${label} — ${res.affected} samples changed (Ctrl+Z undoes)`);
        close();
      } catch (err) {
        setStatus(`Edit failed: ${err}`);
      } finally {
        applyBtn.disabled = false;
      }
    })();
  });
}
