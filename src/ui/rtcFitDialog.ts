import { runRtcFit, type RtcFitRequest, type RtcFitResult } from "../ipc";
import { appState, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { formRow } from "./modal";
import { buildWellScope } from "./wellScope";
import { buildFitScatter, type FitScatter } from "./fitScatter";
import { buildCalibrationApply } from "./calibrationApply";

/** RtC calibration (Advance ▸ Calibrate RtC…).
 *
 *  A dock PANE, not a popup. A calibration is not one click: you fit, read R² and the excluded
 *  samples, look at the scatter, change the interval or a curve and fit again — with the log view
 *  beside you, because deciding which interval is wet is done by looking at one. Standing rule
 *  from Jauhar (2026-08-01): tools open as working panes.
 *
 *  `sw_rtc` has always told the user to "recalibrate per field from water-zone excess
 *  conductivity" and never given them a way to do it, so in practice one study's coefficients
 *  ran on every field. This fits A_CAP / B_QV / C0 to the user's OWN water leg.
 *
 *  The dialog's real job is the water-zone question. Everything else has a sensible default;
 *  that one field cannot, because the fit assumes Sw = 1 and there is no way to find a water
 *  zone without already knowing the saturation the calibration is for. So the interval is
 *  required, the backend refuses without it, and this dialog says why in plain language rather
 *  than presenting it as one input among nine.
 */
export async function buildRtcFitContent(): Promise<{ el: HTMLElement; dispose: () => void }> {
  const wrap = document.createElement("div");
  wrap.className = "module-pane";

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Fits the excess-conductivity coefficients to rock you know is water-bearing, where Sw = 1. " +
    "The result replaces the shipped defaults, which came from one study in one field and are " +
    "the wrong starting point anywhere else.";
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
  // exist), so every caller labels the button itself or it opens blank and disabled.
  {
    const n = scope.getWellIds().length;
    runBtn.textContent = `Fit from ${n} well(s)`;
    runBtn.disabled = n === 0;
  }
  // The backend returns well IDs; the scatter legend has to show the names the user knows.
  const wellName = (id: string): string => scope.namesFor([id])[0] ?? id;

  // ---- the water zone -----------------------------------------------------
  const zoneNote = document.createElement("div");
  zoneNote.className = "eq-note";
  zoneNote.innerHTML =
    "<b>Which interval is wet?</b> Required. Over hydrocarbon-bearing rock the missing " +
    "conductivity is the hydrocarbon, and fitting it hands that to the clay and capillary " +
    "terms — the calibration then under-corrects Rt and reads Sw too high, erasing pay in " +
    "exactly the intervals this method exists to find.";
  wrap.appendChild(zoneNote);

  const topIn = document.createElement("input");
  topIn.className = "form-control";
  topIn.type = "number";
  topIn.step = "0.1";
  const baseIn = document.createElement("input");
  baseIn.className = "form-control";
  baseIn.type = "number";
  baseIn.step = "0.1";

  // Seed from the interval selected in the Tops pane, the same convention every other panel
  // follows — the user has usually already clicked the water sand.
  const sel = appState.selectedInterval.get();
  if (sel) {
    if (sel.depthMin != null) topIn.value = String(sel.depthMin);
    if (sel.depthMax != null) baseIn.value = String(sel.depthMax);
  }
  const depthRow = document.createElement("div");
  depthRow.style.display = "flex";
  depthRow.style.gap = "8px";
  depthRow.appendChild(topIn);
  depthRow.appendChild(baseIn);
  wrap.appendChild(
    formRow("Water zone top / base", depthRow, sel ? "Seeded from the selected top" : "Depth in the project unit")
  );

  const flagIn = document.createElement("input");
  flagIn.className = "form-control";
  flagIn.placeholder = "optional, e.g. WETFLAG";
  wrap.appendChild(
    formRow("…or a wet-flag curve", flagIn, "Non-zero = use the sample. A missing value is NOT treated as wet.")
  );

  // ---- curves -------------------------------------------------------------
  const mkCurve = (label: string, def: string, hint?: string) => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.value = def;
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };
  const rtIn = mkCurve("RT curve", "RES_DEEP");
  const phitIn = mkCurve("PHIT curve", "PHIT_SSC", "SSC's total porosity, or PHIT_SSPW");
  const capIn = mkCurve("CAPBW curve", "CWSH", "Capillary-bound water — SSC's CWSH or SSPW's CAPBW_SSPW");
  const qvIn = mkCurve("QV curve (optional)", "QV", "Leave blank to build Qv from CEC below");

  // ---- parameters ---------------------------------------------------------
  const mkNum = (label: string, def: number, step: string, hint?: string) => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = step;
    i.value = String(def);
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };
  const rwIn = mkNum("Rw (ohm.m)", 0.3, "0.001", "Must be the Rw the run will use — it sets the clean baseline");
  const mIn = mkNum("M", 2.0, "0.01");
  const rsfIn = mkNum("RSF", 2.25, "0.01", "Held FIXED during the fit — the coefficients belong to this RSF");
  const cecIn = mkNum("CEC (meq/100g, if no QV log)", 0, "0.1");
  const rhogIn = mkNum("Grain density (g/cc)", 2.65, "0.01");

  // ---- result -------------------------------------------------------------
  const out = document.createElement("div");
  out.className = "mc-section";
  out.hidden = true;
  wrap.appendChild(out);

  const actions = document.createElement("div");
  actions.className = "module-footer";
  // The QC scatter is rebuilt on every fit; keep a handle so its ResizeObserver and tooltip are
  // released rather than accumulating one set per run.
  let scatter: FitScatter | null = null;
  const dropScatter = (): void => {
    scatter?.dispose();
    scatter = null;
  };

  actions.appendChild(runBtn);
  wrap.appendChild(actions);

  const num = (el: HTMLInputElement): number | null => {
    const v = el.value.trim();
    if (!v) return null;
    const n = Number(v);
    return Number.isFinite(n) ? n : null;
  };

  const render = (r: RtcFitResult): void => {
    out.hidden = false;
    dropScatter();
    if (r.error) {
      out.innerHTML = "";
      const e = document.createElement("div");
      e.className = "eq-note";
      e.style.color = "var(--warn)";
      e.textContent = r.error;
      out.appendChild(e);
      return;
    }
    const f = (v: number, d = 4) => (Number.isFinite(v) ? v.toFixed(d) : "—");
    const rows = [
      ["A_CAP (capillary water)", f(r.a_cap, 4)],
      ["B_QV (clay chemistry)", f(r.b_qv, 6)],
      ["C0 (intercept)", f(r.c0, 6)],
      ["RSF (held fixed)", f(r.rsf_used, 2)],
      ["R²", f(r.r2, 3)],
      ["RMS residual", f(r.rms, 5)],
      ["Samples / wells", `${r.n_points} / ${r.n_wells}`],
    ];
    out.innerHTML = "";
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

    // Measured against fitted excess conductivity, with the 1:1 line. R² says how much scatter
    // there is; only the picture says what KIND — curvature (the two paths are not linear over
    // this interval), or one well parked off the trend, which R² averages away.
    if (r.points.length) {
      scatter = buildFitScatter({
        points: r.points.map((p) => ({
          x: p.y_fit,
          y: p.y,
          group: wellName(p.well_id),
          detail: [
            `${wellName(p.well_id)} @ ${p.depth.toFixed(2)}`,
            `measured ${p.y.toFixed(5)}  fitted ${p.y_fit.toFixed(5)}`,
            `CAPBW ${p.capbw.toFixed(3)}   Qv ${p.qv.toFixed(3)}`,
          ],
        })),
        xLabel: "Fitted excess / (PHIT·RSF)",
        yLabel: "Measured excess / (PHIT·RSF)",
        line: { kind: "identity" },
        caption: `Measured vs fitted excess conductivity — ${r.points.length} sample(s), 1:1 dashed`,
        exportName: "rtc-calibration",
      });
      out.appendChild(scatter.el);
      scatter.redraw(); // first paint is synchronous once it is in the document
    }

    // Excluded counts are shown, always. A calibration quoted from 12 samples of an interval
    // the user believed held 500 is a different statement, and silence about it is the failure.
    if (r.excluded.length) {
      const ex = document.createElement("div");
      ex.className = "eq-note";
      ex.textContent =
        "Not fitted — " + r.excluded.map(([why, n]) => `${n} ${why}`).join("; ");
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
    copy.textContent = "Copy A_CAP / B_QV / C0";
    copy.addEventListener("click", () => {
      // Deliberately a COPY, not an auto-apply. A calibration is a judgement the user makes
      // after reading R² and the excluded counts above it — writing it straight into the
      // module defaults would skip exactly the step that matters.
      void navigator.clipboard
        .writeText(`A_CAP=${r.a_cap}\nB_QV=${r.b_qv}\nC0=${r.c0}\nRSF=${r.rsf_used}`)
        .then(() => setStatus("RtC coefficients copied — paste them into the sw_rtc parameters"));
    });
    out.appendChild(copy);

    // …or write them straight into `zone_params`, which is where a per-well parameter already
    // belongs, so the next sw_rtc run and every workflow chain picks them up. RSF rides along
    // because the coefficients are only valid for the RSF they were fitted with.
    out.appendChild(
      buildCalibrationApply({
        label: "RtC calibration",
        fittedWells: r.wells_fitted,
        scopedWells: scope.getWellIds(),
        wellName,
        params: [
          { name: "A_CAP", value: r.a_cap },
          { name: "B_QV", value: r.b_qv },
          { name: "C0", value: r.c0 },
          { name: "RSF", value: r.rsf_used },
        ],
      })
    );
  };

  runBtn.addEventListener("click", () => {
    const req: RtcFitRequest = {
      well_ids: scope.getWellIds(),
      rt_curve: rtIn.value.trim(),
      phit_curve: phitIn.value.trim(),
      capbw_curve: capIn.value.trim(),
      qv_curve: qvIn.value.trim(),
      cec: num(cecIn) ?? 0,
      rhog: num(rhogIn) ?? 2.65,
      rw: num(rwIn) ?? 0.3,
      m: num(mIn) ?? 2.0,
      rsf: num(rsfIn) ?? 2.25,
      depth_min: num(topIn),
      depth_max: num(baseIn),
      wet_flag_curve: flagIn.value.trim(),
    };
    runBtn.disabled = true;
    runBtn.textContent = "Fitting…";
    void runRtcFit(req)
      .then((r) => {
        render(r);
        if (!r.error) {
          setStatus(`RtC calibrated from ${r.n_points} water-zone samples (R² ${r.r2.toFixed(2)})`);
          recordProcess("fit", `RtC calibration: ${r.n_points} samples from ${r.n_wells} well(s)`);
        }
      })
      .catch((e) => render({ error: String(e) } as RtcFitResult))
      .finally(() => {
        runBtn.disabled = scope.getWellIds().length === 0;
        runBtn.textContent = `Fit from ${scope.getWellIds().length} well(s)`;
      });
  });

  // A pane is closed by the dock, not by a Close button, so the two things that outlive the
  // element — the scatter's ResizeObserver and the well scope's subscriptions — are released here.
  return {
    el: wrap,
    dispose: () => {
      dropScatter();
      scope.dispose();
    },
  };
}
