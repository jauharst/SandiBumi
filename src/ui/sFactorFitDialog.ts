import { runSFactorFit, type SFactorFitRequest, type SFactorFitResult } from "../ipc";
import { setStatus } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";
import { buildWellScope } from "./wellScope";
import { buildFitScatter, type FitScatter } from "./fitScatter";
import { buildCalibrationApply } from "./calibrationApply";

/** IMTS S-factor calibration (Advance ▸ Calibrate S…).
 *
 *  `sw_imts` defines S as a measurement — lab CEC divided by the CEC the clay model predicts —
 *  and the app shipped a placeholder for it. S multiplies the whole clay-charge term, so a wrong
 *  S scales Qv_eff directly and moves SwT with nothing on the log to show for it.
 *
 *  The dialog's real job is the pairing question. The clay curves named here must be the ones
 *  the sw_imts RUN will use: calibrate S against XRD weight fractions and then run against a
 *  VDCL-derived VKAOL curve and S is wrong by the ratio between those two estimates of clay,
 *  silently, because both look like clay volumes.
 */
export async function openSFactorFitDialog(): Promise<void> {
  const wrap = document.createElement("div");
  const close = openModal("Calibrate the IMTS S factor from lab CEC", wrap, 660);

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Fits S = measured CEC / theoretical CEC against your own core measurements. The shipped " +
    "0.5 is a placeholder standing in for the method's observation that lab CEC runs below the " +
    "XRD-theoretical value — it was never measured in any rock.";
  wrap.appendChild(intro);

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";

  const scope = await buildWellScope({
    onChange: (ids) => {
      runBtn.textContent = `Fit from ${ids.length} well(s)`;
      runBtn.disabled = ids.length === 0;
    },
  });
  wrap.appendChild(scope.el);
  // First paint. `buildWellScope` deliberately does NOT fire onChange during construction
  // (wellScope.ts: a synchronous first fire would run this callback before the caller's consts
  // exist), so every caller labels the button itself or it opens blank.
  const paintRunBtn = (): void => {
    const n = scope.getWellIds().length;
    runBtn.textContent = `Fit from ${n} well(s)`;
    runBtn.disabled = n === 0;
  };
  paintRunBtn();
  // The backend returns well IDs; the scatter legend has to show the names the user knows.
  const wellName = (id: string): string => scope.namesFor([id])[0] ?? id;

  // ---- where the laboratory CEC lives -------------------------------------
  const mkText = (label: string, def: string, hint?: string) => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.value = def;
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };
  const dsIn = mkText(
    "CEC point dataset",
    "CEC",
    "Read from the ACTIVE delivery of that dataset. Use CORE if the CEC arrived as an extra column on a core table."
  );
  const itemIn = mkText("Item name", "CEC", "The measurement's name within the dataset, in meq/100g");

  // ---- the pairing --------------------------------------------------------
  const pairNote = document.createElement("div");
  pairNote.className = "eq-note";
  pairNote.innerHTML =
    "<b>Name the curves the run will use.</b> S is a ratio against whatever the clay curves " +
    "say, so calibrating it against one estimate of clay and running it against another makes " +
    "S wrong by the difference between them — invisibly, because both are clay volumes.";
  wrap.appendChild(pairNote);

  const vkIn = mkText("VKAOL curve", "VDCL", "Kaolinite volume — sw_imts's own default input");
  const viIn = mkText("VILL curve (optional)", "VILL", "Illite volume; leave blank if you have none");

  const mkNum = (label: string, def: number, step: string, hint?: string) => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = step;
    i.value = String(def);
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };
  const kaolIn = mkNum("Kaolinite CEC (meq/100g)", 8, "0.1", "Literature constant. Held fixed — S multiplies it");
  const illIn = mkNum("Illite CEC (meq/100g)", 25, "0.1", "Literature constant. Held fixed — S multiplies it");
  const tolIn = mkNum(
    "Depth tolerance",
    0.15,
    "0.01",
    "A plug further than this from any log sample is dropped, not snapped. Depth-shift the core first."
  );

  // ---- result -------------------------------------------------------------
  const out = document.createElement("div");
  out.className = "mc-section";
  out.hidden = true;
  wrap.appendChild(out);

  const actions = document.createElement("div");
  actions.className = "modal-actions";
  // Rebuilt on every fit; keep a handle so its ResizeObserver and tooltip are released rather
  // than accumulating one set per run.
  let scatter: FitScatter | null = null;
  const dropScatter = (): void => {
    scatter?.dispose();
    scatter = null;
  };

  const cancel = document.createElement("button");
  cancel.className = "btn";
  cancel.textContent = "Close";
  cancel.addEventListener("click", () => {
    dropScatter();
    scope.dispose();
    close();
  });
  actions.appendChild(cancel);
  actions.appendChild(runBtn);
  wrap.appendChild(actions);

  const num = (el: HTMLInputElement, fallback: number): number => {
    const n = Number(el.value.trim());
    return Number.isFinite(n) ? n : fallback;
  };

  const render = (r: SFactorFitResult): void => {
    out.hidden = false;
    dropScatter();
    out.innerHTML = "";
    if (r.error) {
      const e = document.createElement("div");
      e.className = "eq-note";
      e.style.color = "var(--warn)";
      e.textContent = r.error;
      out.appendChild(e);
      return;
    }
    const f = (v: number, d = 4) => (Number.isFinite(v) ? v.toFixed(d) : "—");
    const rows: [string, string][] = [
      ["S_FACTOR (fitted)", f(r.s_factor, 4)],
      ["Median per-plug ratio", f(r.s_median_ratio, 4)],
      ["Plug ratios P10 → P90", `${f(r.ratio_p10, 3)} → ${f(r.ratio_p90, 3)}`],
      ["R²", f(r.r2, 3)],
      ["RMS residual (meq/100g)", f(r.rms, 4)],
      ["Plugs / wells", `${r.n_points} / ${r.n_wells}`],
      ["Fitted against CEC_KAOL / CEC_ILL", `${f(r.cec_kaol_used, 1)} / ${f(r.cec_ill_used, 1)}`],
    ];
    const tbl = document.createElement("table");
    tbl.className = "mc-table";
    for (const [k, v] of rows) {
      const tr = document.createElement("tr");
      const td1 = document.createElement("td");
      td1.textContent = k;
      const td2 = document.createElement("td");
      td2.textContent = v;
      td2.style.textAlign = "right";
      tr.appendChild(td1);
      tr.appendChild(td2);
      tbl.appendChild(tr);
    }
    out.appendChild(tbl);

    // The regression itself, NOT measured-vs-fitted. With one predictor those two plots carry
    // the same information, but only this one puts clay content on the x axis — which is what
    // turns "the plugs disagree" in the note above into a shape you can name: a curved cloud is
    // S drifting with clay, a fan opening toward the origin is noise on the lean plugs, and a
    // cluster off the line is one core suite. The line is the fit, through the origin.
    if (r.points.length) {
      scatter = buildFitScatter({
        points: r.points.map((p) => ({
          x: p.cec_theo,
          y: p.cec_lab,
          group: wellName(p.well_id),
          detail: [
            `${wellName(p.well_id)} @ ${p.depth.toFixed(2)} (log ${p.log_depth.toFixed(2)})`,
            `lab ${p.cec_lab.toFixed(2)}  model ${p.cec_theo.toFixed(2)} meq/100g`,
            `ratio ${p.ratio.toFixed(3)}   VKAOL ${p.vkaol.toFixed(3)}  VILL ${p.vill.toFixed(3)}`,
          ],
        })),
        xLabel: "Theoretical CEC from the clay model (meq/100g)",
        yLabel: "Laboratory CEC (meq/100g)",
        line: { kind: "origin", slope: r.s_factor },
        caption: `Lab vs modelled CEC — ${r.points.length} plug(s), line is S = ${r.s_factor.toFixed(3)} through the origin`,
        exportName: "imts-s-factor",
      });
      out.appendChild(scatter.el);
      scatter.redraw(); // first paint is synchronous once it is in the document
    }

    // Excluded counts are shown, always — an S quoted from 4 plugs of a 60-plug CEC suite is a
    // different statement, and silence about it is the failure.
    if (r.excluded.length) {
      const ex = document.createElement("div");
      ex.className = "eq-note";
      ex.textContent = "Not fitted — " + r.excluded.map(([why, n]) => `${n} ${why}`).join("; ");
      out.appendChild(ex);
    }
    for (const n of r.notes) {
      const d = document.createElement("div");
      d.className = "eq-note";
      d.textContent = n;
      out.appendChild(d);
    }

    const copy = document.createElement("button");
    copy.className = "btn";
    copy.textContent = "Copy S_FACTOR";
    copy.addEventListener("click", () => {
      // A copy, not an auto-apply — same reasoning as the RtC fit. S is a judgement made after
      // reading the spread and the exclusions, and writing it straight into the module defaults
      // would skip exactly that step.
      // The CEC constants travel with it: S multiplies them, so the three are one setting.
      void navigator.clipboard
        .writeText(
          `S_FACTOR=${r.s_factor}\nCEC_KAOL=${r.cec_kaol_used}\nCEC_ILL=${r.cec_ill_used}`
        )
        .then(() => setStatus("S factor copied — paste it into the sw_imts parameters"));
    });
    out.appendChild(copy);

    // …or write it into `zone_params` directly. The CEC constants ride along: S multiplies them,
    // so the three are one setting and applying S alone would pair it with different rock.
    out.appendChild(
      buildCalibrationApply({
        label: "IMTS S calibration",
        fittedWells: r.wells_fitted,
        scopedWells: scope.getWellIds(),
        wellName,
        params: [
          { name: "S_FACTOR", value: r.s_factor },
          { name: "CEC_KAOL", value: r.cec_kaol_used },
          { name: "CEC_ILL", value: r.cec_ill_used },
        ],
      })
    );
  };

  runBtn.addEventListener("click", () => {
    const req: SFactorFitRequest = {
      well_ids: scope.getWellIds(),
      cec_dataset: dsIn.value.trim(),
      cec_item: itemIn.value.trim(),
      vkaol_curve: vkIn.value.trim(),
      vill_curve: viIn.value.trim(),
      cec_kaol: num(kaolIn, 8),
      cec_ill: num(illIn, 25),
      depth_tol: num(tolIn, 0.15),
    };
    runBtn.disabled = true;
    runBtn.textContent = "Fitting…";
    void runSFactorFit(req)
      .then((r) => {
        render(r);
        if (!r.error) {
          setStatus(`S = ${r.s_factor.toFixed(3)} from ${r.n_points} CEC plug(s)`);
          recordProcess("fit", `IMTS S calibration: ${r.n_points} plugs from ${r.n_wells} well(s)`);
        }
      })
      .catch((e) => render({ error: String(e) } as SFactorFitResult))
      .finally(() => {
        runBtn.disabled = scope.getWellIds().length === 0;
        runBtn.textContent = `Fit from ${scope.getWellIds().length} well(s)`;
      });
  });
}
