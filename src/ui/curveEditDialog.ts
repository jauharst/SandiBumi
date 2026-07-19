import { editCurve, restoreCurveValues, type CurveEditRequest } from "../ipc";
import { recordProcess } from "../processLog";
import { bumpDataVersion, setStatus } from "../state";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";

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

  const applyBtn = document.createElement("button");
  applyBtn.className = "lp-btn";
  applyBtn.textContent = "Apply";
  content.appendChild(applyBtn);

  const close = openModal(`Edit ${curveName} — ${wellName}`, content, 420);

  applyBtn.addEventListener("click", () => {
    void (async () => {
      const op = opSel.value as CurveEditRequest["op"];
      const req: CurveEditRequest = {
        well_id: wellId,
        curve: curveName,
        op,
        delta: parseFloat(deltaInput.value) || 0,
        top: parseFloat(topInput.value) || 0,
        bottom: parseFloat(bottomInput.value) || 0,
        value: parseFloat(valueInput.value) || 0,
        mul: parseFloat(mulInput.value),
        add: parseFloat(addInput.value) || 0,
      };
      if (!Number.isFinite(req.mul!)) req.mul = 1;
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
        pushUndo({
          label,
          undo: async () => {
            await restoreCurveValues(wellId, curveName, pointCount, prevBytes);
            bumpDataVersion();
          },
          redo: async () => {
            await editCurve(req);
            bumpDataVersion();
          },
        });
        recordProcess("Edit", `${label}: ${res.affected} samples (${res.store} store)`, wellName);
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
