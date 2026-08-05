import {
  getCurveData,
  listAuxData,
  listAuxDatasets,
  listWells,
  listZones,
  runCutoffSweep,
  saveDocument,
  type CutoffSweepResult,
} from "../ipc";
import { recordProcess } from "../processLog";
import { appState, bumpDataVersion } from "../state";
import { DEFAULT_CUTOFFS, loadCutoffDefaults } from "./cutoffs";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow } from "./modal";
import { PlotCanvas, attachResizeRedraw, canvasFont, faciesColor, fitCanvasBackingStore, readTheme, type AxisSpec } from "./plotCanvas";
import { nearestDepthIndex } from "./plotCommon";
import { buildWellScope } from "./wellScope";

/** Cutoff sensitivity (ROADMAP Wave E item 21).
 *  Two ways to pick VSH/PHIE/SWE pay cutoffs against DST-tested rock:
 *   • Sweep — plot a pay metric (net / HPV / N:G) against a swept cutoff, other two held
 *     fixed, so the elbow shows where loosening the cutoff stops adding real pay.
 *   • Crossplot — PHIE vs a shale/Sw curve, all samples dim, DST-interval samples coloured,
 *     with a draggable red crosshair at the candidate cutoffs.
 *  Picked cutoffs can be written into the pay-summary defaults (documents "cutoffs").
 *  Hosted as a dock pane (workspace component "cutoff"). */
export async function buildCutoffContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const wells = await listWells();

  const root = document.createElement("div");
  root.className = "cutoff-pane";

  // --- Well scope (group / ★ pinned / selection / all) instead of a per-well checklist.
  // The zone/DST pickers rebuild whenever the scope changes; `wells` is kept (unfiltered) for
  // the crossplot's per-well name lookup.
  const scope = await buildWellScope({ onChange: () => void refreshZoneDst() });
  root.appendChild(scope.el);
  const checkedWellIds = (): string[] => scope.getWellIds();

  // --- Zone + DST dataset pickers (union over the currently-checked wells) ---
  const zoneSelect = document.createElement("select");
  zoneSelect.className = "form-control";
  const dstSelect = document.createElement("select");
  dstSelect.className = "form-control";
  const opt = (sel: HTMLSelectElement, value: string, text: string) => {
    const o = document.createElement("option");
    o.value = value;
    o.textContent = text;
    sel.appendChild(o);
  };
  // Repopulated whenever the well selection changes so the lists never go stale; the
  // current choice is preserved if it still exists, else a DST/perf set is auto-picked.
  let optionsEpoch = 0;
  let dstUserTouched = false; // once the user picks a DST scope, stop auto-overriding it
  async function refreshZoneDst(): Promise<void> {
    const epoch = ++optionsEpoch;
    // Union over ALL checked wells (Compute runs on all of them): a zone or DST set that
    // exists only on a later well must still be selectable, not silently hidden.
    const seed = checkedWellIds();
    const zoneNames = new Set<string>();
    const dstNames = new Set<string>();
    await Promise.all(
      seed.map(async (id) => {
        try {
          for (const z of await listZones(id)) zoneNames.add(z.zone_name);
          for (const [ds] of await listAuxDatasets(id)) dstNames.add(ds);
        } catch {
          /* a well with no zones/aux just contributes nothing */
        }
      }),
    );
    if (epoch !== optionsEpoch) return; // a newer refresh already won
    const keepZone = zoneSelect.value;
    const keepDst = dstSelect.value;
    zoneSelect.innerHTML = "";
    dstSelect.innerHTML = "";
    opt(zoneSelect, "", "(whole well)");
    for (const z of [...zoneNames].sort()) opt(zoneSelect, z, z);
    opt(dstSelect, "", "(all samples)");
    for (const d of [...dstNames].sort()) opt(dstSelect, d, d);
    zoneSelect.value = [...zoneNames].includes(keepZone) ? keepZone : "";
    if (dstUserTouched) {
      // Preserve the user's explicit scope; "(all samples)" (value "") is always valid, so
      // a deliberate whole-well choice is no longer clobbered when the well set changes.
      dstSelect.value = keepDst === "" || dstNames.has(keepDst) ? keepDst : "";
    } else {
      // Not yet chosen: default to a perforation/DST set when one exists.
      const preferred = [...dstNames].find((d) => /DST|PERF/i.test(d));
      dstSelect.value = preferred ?? "";
    }
  }
  await refreshZoneDst();
  root.appendChild(formRow("Zone", zoneSelect, "Restrict the analysis to one zone"));
  root.appendChild(formRow("DST / perf set", dstSelect, "Only samples inside these test intervals count"));
  // The scope selector's onChange (wired at construction) re-runs refreshZoneDst on any change.
  dstSelect.addEventListener("change", () => {
    dstUserTouched = true;
  });

  // --- Shared cutoff fields (the fixed cutoffs + where picks are written) ----
  const numInput = (value: string, cls = "form-control"): HTMLInputElement => {
    const i = document.createElement("input");
    i.className = cls;
    i.type = "number";
    i.step = "any";
    i.value = value;
    return i;
  };
  // Seed from the shared cutoff source so reopening reflects the project's saved defaults, not a
  // frozen literal — the same set the pay summary, Monte Carlo and the report all open with.
  const cuts = await loadCutoffDefaults();
  const vshIn = numInput(String(cuts.vsh_max));
  const phieIn = numInput(String(cuts.phie_min));
  const sweIn = numInput(String(cuts.swe_max));
  const permIn = numInput(cuts.perm_min != null ? String(cuts.perm_min) : "");
  permIn.placeholder = "(off)";
  root.appendChild(formRow("VSH ≤", vshIn, "Sand cutoff"));
  root.appendChild(formRow("PHIE ≥", phieIn, "Reservoir cutoff"));
  root.appendChild(formRow("SWE ≤", sweIn, "Pay cutoff"));
  root.appendChild(formRow("PERM ≥ (optional)", permIn, "Extra pay cutoff, needs a PERM curve"));
  const cutFor = (p: "VSH" | "PHIE" | "SWE"): HTMLInputElement =>
    p === "VSH" ? vshIn : p === "PHIE" ? phieIn : sweIn;
  const numOf = (i: HTMLInputElement, fallback: number): number => {
    const v = parseFloat(i.value);
    return Number.isFinite(v) ? v : fallback;
  };

  // --- Mode toggle ----------------------------------------------------------
  let mode: "sweep" | "crossplot" = "sweep";
  const modeBar = document.createElement("div");
  modeBar.className = "cutoff-seg";
  const sweepModeBtn = segButton(modeBar, "Sweep", true);
  const xplotModeBtn = segButton(modeBar, "DST Crossplot", false);
  root.appendChild(formRow("Method", modeBar));

  // --- Sweep controls -------------------------------------------------------
  const sweepControls = document.createElement("div");
  sweepControls.className = "cutoff-mode-controls";
  const propBar = document.createElement("div");
  propBar.className = "cutoff-seg";
  const propBtns: Record<"VSH" | "PHIE" | "SWE", HTMLButtonElement> = {
    VSH: segButton(propBar, "VSH", true),
    PHIE: segButton(propBar, "PHIE", false),
    SWE: segButton(propBar, "SWE", false),
  };
  let property: "VSH" | "PHIE" | "SWE" = "VSH";
  sweepControls.appendChild(formRow("Sweep", propBar, "Which cutoff to vary"));
  const sweepMinIn = numInput("0");
  const sweepMaxIn = numInput("1");
  const stepsIn = numInput("60");
  sweepControls.appendChild(formRow("From → to", rowPair(sweepMinIn, sweepMaxIn), "Candidate cutoff range"));
  sweepControls.appendChild(formRow("Steps", stepsIn));
  const metricBar = document.createElement("div");
  metricBar.className = "cutoff-seg";
  const metricBtns: Record<"NET" | "HPV" | "NTG", HTMLButtonElement> = {
    NET: segButton(metricBar, "Net", true),
    HPV: segButton(metricBar, "HPV", false),
    NTG: segButton(metricBar, "N:G", false),
  };
  let metric: "NET" | "HPV" | "NTG" = "NET";
  sweepControls.appendChild(formRow("Metric", metricBar, "Net thickness · HC pore-thickness · net-to-gross"));
  const normChk = document.createElement("input");
  normChk.type = "checkbox";
  normChk.checked = true;
  const normLabel = document.createElement("label");
  normLabel.className = "cutoff-inline-check";
  normLabel.appendChild(normChk);
  normLabel.appendChild(document.createTextNode(" Normalise each well to its own peak"));
  sweepControls.appendChild(formRow("", normLabel));
  root.appendChild(sweepControls);

  // Default the sweep range to sensible bounds per property.
  const applyPropDefaults = () => {
    if (property === "PHIE") {
      sweepMinIn.value = "0";
      sweepMaxIn.value = "0.3";
    } else {
      sweepMinIn.value = "0";
      sweepMaxIn.value = "1";
    }
  };

  // --- Crossplot controls ---------------------------------------------------
  const xplotControls = document.createElement("div");
  xplotControls.className = "cutoff-mode-controls";
  xplotControls.style.display = "none";
  const xCurveIn = document.createElement("input");
  xCurveIn.className = "form-control";
  xCurveIn.value = "VOL_WETCLAY";
  const yCurveIn = document.createElement("input");
  yCurveIn.className = "form-control";
  yCurveIn.value = "PHIE";
  xplotControls.appendChild(formRow("X curve", xCurveIn, "Shale (Vclay) or Sw curve"));
  xplotControls.appendChild(formRow("Y curve", yCurveIn, "Porosity (PHIE) curve"));
  const presetBar = document.createElement("div");
  presetBar.className = "cutoff-presets";
  const vclayPreset = presetButton(presetBar, "PHIE vs Vclay");
  const swPreset = presetButton(presetBar, "PHIE vs Sw");
  xplotControls.appendChild(formRow("Presets", presetBar));
  // --- Input log set (`logSetPicker.ts`): which VERSION of the curves this reads.
  const setPicker = buildLogSetPicker({ write: false });
  for (const row of setPicker.rows) xplotControls.appendChild(row);

  root.appendChild(xplotControls);

  // --- Run + canvas + readout ----------------------------------------------
  const runBtn = document.createElement("button");
  runBtn.className = "form-run-btn";
  runBtn.textContent = "Compute";
  root.appendChild(runBtn);

  const canvas = document.createElement("canvas");
  canvas.className = "plot-canvas cutoff-canvas";
  root.appendChild(canvas);

  const readout = document.createElement("div");
  readout.className = "cutoff-readout";
  root.appendChild(readout);

  const legend = document.createElement("div");
  legend.className = "cutoff-legend";
  root.appendChild(legend);

  const pickBar = document.createElement("div");
  pickBar.className = "cutoff-pickbar";
  const usePickBtn = document.createElement("button");
  usePickBtn.className = "cutoff-btn";
  const saveDefaultBtn = document.createElement("button");
  saveDefaultBtn.className = "cutoff-btn";
  saveDefaultBtn.textContent = "Save as pay-summary default";
  pickBar.appendChild(usePickBtn);
  pickBar.appendChild(saveDefaultBtn);
  root.appendChild(pickBar);

  // --- Plot state -----------------------------------------------------------
  let pc: PlotCanvas | null = null; // the PlotCanvas from the most recent redraw (hit-testing)
  let sweep: CutoffSweepResult | null = null;
  let sweepReq: { property: "VSH" | "PHIE" | "SWE"; sweepMin: number; sweepMax: number; metric: string } | null = null;
  let pickedCutoff: number | null = null;
  // Crossplot per-well point sets + the crosshair.
  interface XSet {
    name: string;
    x: number[];
    y: number[];
    dst: boolean[];
  }
  let xsets: XSet[] = [];
  let xAxisLabel = "VOL_WETCLAY";
  let yAxisLabel = "PHIE";
  let xCut = 0.5;
  let yCut = 0.1;

  const setModeButtons = () => {
    sweepModeBtn.classList.toggle("active", mode === "sweep");
    xplotModeBtn.classList.toggle("active", mode === "crossplot");
    sweepControls.style.display = mode === "sweep" ? "" : "none";
    xplotControls.style.display = mode === "crossplot" ? "" : "none";
    usePickBtn.textContent = mode === "sweep" ? `Use pick as ${property} cutoff` : "Apply crosshair → cutoffs";
  };

  // === Rendering ============================================================
  const redraw = () => {
    fitCanvasBackingStore(canvas);
    if (mode === "sweep") drawSweep();
    else drawCrossplot();
  };

  function niceRange(lo: number, hi: number): [number, number] {
    if (!(hi > lo)) return [lo - 1, lo + 1];
    const pad = (hi - lo) * 0.05;
    return [lo - pad, hi + pad];
  }

  function drawSweep(): void {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    if (!sweep || !sweepReq) {
      pc = null;
      hint(ctx, "Choose wells and a swept cutoff, then Compute.");
      return;
    }
    const normalise = normChk.checked;
    let yMax = 0;
    for (const s of sweep.series) {
      for (const v of s.values) {
        const y = normalise ? (s.peak > 0 ? v / s.peak : 0) : v;
        if (y > yMax) yMax = y;
      }
    }
    if (yMax <= 0) yMax = 1;
    const x: AxisSpec = {
      label: `${sweepReq.property} cutoff`,
      min: sweepReq.sweepMin,
      max: sweepReq.sweepMax,
      log: false,
      invert: false,
    };
    const y: AxisSpec = {
      label: normalise ? `${sweep.metric} (fraction of peak)` : metricLabel(sweep.metric),
      min: 0,
      max: yMax * 1.02,
      log: false,
      invert: false,
    };
    pc = new PlotCanvas(canvas, x, y);
    pc.drawFrame();
    sweep.series.forEach((s, i) => {
      if (s.cutoffs.length < 2) return;
      const pts: [number, number][] = s.cutoffs.map((c, k) => {
        const v = s.values[k];
        return [c, normalise ? (s.peak > 0 ? v / s.peak : 0) : v];
      });
      pc!.drawLine(pts, faciesColor(i), 1.8);
    });
    if (pickedCutoff !== null) {
      pc.drawVMarker(pickedCutoff, pc.theme.warn, pickedCutoff.toFixed(3));
    }
    renderSweepLegend();
    renderSweepReadout();
  }

  function drawCrossplot(): void {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    if (xsets.length === 0) {
      pc = null;
      hint(ctx, "Choose wells and X/Y curves, then Compute.");
      return;
    }
    let xlo = Infinity;
    let xhi = -Infinity;
    let ylo = Infinity;
    let yhi = -Infinity;
    for (const s of xsets) {
      for (let i = 0; i < s.x.length; i++) {
        const vx = s.x[i];
        const vy = s.y[i];
        if (Number.isNaN(vx) || Number.isNaN(vy)) continue;
        if (vx < xlo) xlo = vx;
        if (vx > xhi) xhi = vx;
        if (vy < ylo) ylo = vy;
        if (vy > yhi) yhi = vy;
      }
    }
    if (!Number.isFinite(xlo)) {
      pc = null;
      hint(ctx, "No overlapping X/Y samples for the selected wells.");
      renderXplotLegend();
      return;
    }
    const [x0, x1] = niceRange(Math.min(0, xlo), xhi);
    const [y0, y1] = niceRange(Math.min(0, ylo), yhi);
    // Keep the crosshair inside the auto-ranged axes: a seed from the VSH/SWE field (e.g. 0.5)
    // can exceed the data range, and drawLine clips off-plot — an invisible, ungrabbable line
    // whose readout still claims a value. Clamping keeps line, readout and hit-test consistent.
    xCut = clampAxis(xCut, x0, x1);
    yCut = clampAxis(yCut, y0, y1);
    const x: AxisSpec = { label: xAxisLabel, min: x0, max: x1, log: false, invert: false };
    const y: AxisSpec = { label: yAxisLabel, min: y0, max: y1, log: false, invert: false };
    pc = new PlotCanvas(canvas, x, y);
    pc.drawFrame();
    // Non-DST first (dim), DST on top (coloured per well) so tested rock stands out.
    const dimX: number[] = [];
    const dimY: number[] = [];
    for (const s of xsets) {
      for (let i = 0; i < s.x.length; i++) {
        if (!s.dst[i]) {
          dimX.push(s.x[i]);
          dimY.push(s.y[i]);
        }
      }
    }
    // Dim (untested) samples in the axis-text colour; DST samples get the per-well colour.
    const dimColor = pc.theme.text;
    pc.drawScatter(
      dimX,
      dimY,
      dimX.map(() => dimColor),
      1.3,
    );
    xsets.forEach((s, i) => {
      const cx: number[] = [];
      const cy: number[] = [];
      for (let k = 0; k < s.x.length; k++) {
        if (s.dst[k]) {
          cx.push(s.x[k]);
          cy.push(s.y[k]);
        }
      }
      pc!.drawScatter(
        cx,
        cy,
        cx.map(() => faciesColor(i)),
        2.6,
      );
    });
    // Crosshair.
    const warn = pc.theme.warn;
    pc.drawLine(
      [
        [xCut, y0],
        [xCut, y1],
      ],
      warn,
      1.5,
      [5, 3],
    );
    pc.drawLine(
      [
        [x0, yCut],
        [x1, yCut],
      ],
      warn,
      1.5,
      [5, 3],
    );
    renderXplotLegend();
    renderXplotReadout();
  }

  function hint(ctx: CanvasRenderingContext2D, msg: string): void {
    const theme = readTheme(canvas);
    const w = canvas.clientWidth || canvas.width;
    const h = canvas.clientHeight || canvas.height;
    // Scale the context by dpr and draw in CSS-pixel coordinates (fitCanvasBackingStore sized
    // the backing store to w*dpr), so the empty-state text stays centred and 13px on HiDPI.
    const dpr = Math.min(window.devicePixelRatio || 1, 2.5);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = theme.text;
    ctx.font = canvasFont(theme, 13, 400);
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(msg, w / 2, h / 2);
    legend.innerHTML = "";
    readout.textContent = "";
  }

  function metricLabel(m: string): string {
    return m === "HPV" ? "HC pore-thickness (m)" : m === "NTG" ? "Net-to-gross" : "Net thickness (m)";
  }

  function renderSweepLegend(): void {
    legend.innerHTML = "";
    if (!sweep) return;
    sweep.series.forEach((s, i) => {
      const chip = document.createElement("span");
      chip.className = "cutoff-chip";
      const dot = document.createElement("span");
      dot.className = "cutoff-dot";
      dot.style.background = faciesColor(i);
      chip.appendChild(dot);
      // 0 samples ⇒ zone/DST absent; samples but zero peak ⇒ no pay at any cutoff (often
      // VSH/PHIE/SWE not computed yet), which a flat line alone would not distinguish.
      const warn = s.n_samples === 0 ? " (0 samples)" : s.peak <= 0 ? " (no pay)" : "";
      chip.appendChild(document.createTextNode(`${s.well_name}${warn}`));
      legend.appendChild(chip);
    });
  }

  function renderXplotLegend(): void {
    legend.innerHTML = "";
    xsets.forEach((s, i) => {
      const nDst = s.dst.reduce((a, b) => a + (b ? 1 : 0), 0);
      const chip = document.createElement("span");
      chip.className = "cutoff-chip";
      const dot = document.createElement("span");
      dot.className = "cutoff-dot";
      dot.style.background = faciesColor(i);
      chip.appendChild(dot);
      chip.appendChild(document.createTextNode(`${s.name} · ${nDst} DST pts`));
      legend.appendChild(chip);
    });
  }

  function renderSweepReadout(): void {
    if (pickedCutoff === null || !sweep || !sweepReq) {
      readout.textContent = "Click or drag on the plot to place a candidate cutoff.";
      return;
    }
    const m = sweep.metric;
    const parts = sweep.series.map((s) => {
      if (s.cutoffs.length < 2) return `${s.well_name}: —`;
      const v = interpAt(s.cutoffs, s.values, pickedCutoff!);
      return `${s.well_name}: ${m === "NTG" ? v.toFixed(3) : v.toFixed(2)}`;
    });
    readout.textContent = `${sweepReq.property} = ${pickedCutoff.toFixed(3)} → ${metricLabel(sweep.metric)}  |  ${parts.join("  ·  ")}`;
  }

  function renderXplotReadout(): void {
    readout.textContent = `Crosshair: ${xAxisLabel} = ${xCut.toFixed(3)}, ${yAxisLabel} = ${yCut.toFixed(3)}. Drag the red lines to adjust.`;
  }

  // Linear interpolation of a monotone-x sweep series at an arbitrary cutoff.
  function interpAt(xs: number[], ys: number[], x: number): number {
    if (x <= xs[0]) return ys[0];
    if (x >= xs[xs.length - 1]) return ys[ys.length - 1];
    for (let i = 1; i < xs.length; i++) {
      if (x <= xs[i]) {
        const t = (x - xs[i - 1]) / (xs[i] - xs[i - 1] || 1);
        return ys[i - 1] + t * (ys[i] - ys[i - 1]);
      }
    }
    return ys[ys.length - 1];
  }

  // === Compute ==============================================================
  async function computeSweep(): Promise<void> {
    const wellIds = checkedWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All.");
      return;
    }
    const sweepMin = numOf(sweepMinIn, 0);
    const sweepMax = numOf(sweepMaxIn, 1);
    if (!(sweepMax > sweepMin)) {
      readout.textContent = "Sweep range invalid: 'to' must exceed 'from'.";
      return;
    }
    const permRaw = parseFloat(permIn.value);
    runBtn.disabled = true;
    readout.textContent = "Computing sweep…";
    try {
      const res = await runCutoffSweep({
        input_set: setPicker.inputSet(),
        well_ids: wellIds,
        property,
        vsh_max: numOf(vshIn, DEFAULT_CUTOFFS.vsh_max),
        phie_min: numOf(phieIn, DEFAULT_CUTOFFS.phie_min),
        swe_max: numOf(sweIn, DEFAULT_CUTOFFS.swe_max),
        perm_min: Number.isFinite(permRaw) ? permRaw : null,
        sweep_min: sweepMin,
        sweep_max: sweepMax,
        steps: Math.round(numOf(stepsIn, 60)),
        metric,
        zone: zoneSelect.value || null,
        dst_dataset: dstSelect.value || null,
      });
      sweep = res;
      sweepReq = { property, sweepMin, sweepMax, metric };
      // Seed the pick at the property's current fixed cutoff (clamped into range).
      const seed = numOf(cutFor(property), (sweepMin + sweepMax) / 2);
      pickedCutoff = Math.min(sweepMax, Math.max(sweepMin, seed));
      const empties = res.series.filter((s) => s.n_samples === 0).length;
      const noPay = res.series.filter((s) => s.n_samples > 0 && s.peak <= 0).length;
      const notes: string[] = [];
      if (empties) notes.push(`${empties} with 0 samples (zone/DST not present)`);
      if (noPay) notes.push(`${noPay} with no pay at any cutoff — check VSH/PHIE/SWE are computed`);
      setStatus(
        `Cutoff sweep: ${res.series.length} well(s), ${property} × ${metric}` +
          (notes.length ? `; ${notes.join("; ")}` : ""),
      );
      redraw();
    } catch (err) {
      readout.textContent = `Sweep failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  }

  async function computeCrossplot(): Promise<void> {
    const ids = new Set(scope.getWellIds());
    const checked = wells.filter((w) => ids.has(w.well_id));
    if (checked.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All.");
      return;
    }
    const xCurve = xCurveIn.value.trim() || "VOL_WETCLAY";
    const yCurve = yCurveIn.value.trim() || "PHIE";
    xAxisLabel = xCurve;
    yAxisLabel = yCurve;
    const zone = zoneSelect.value || null;
    const dstDs = dstSelect.value || null;
    runBtn.disabled = true;
    readout.textContent = "Loading crossplot data…";
    try {
      const built: XSet[] = [];
      for (const well of checked) {
        const series = await getCurveData(well.well_id, [xCurve, yCurve], null, null);
        const xs = series.find((s) => s.curve_name === xCurve);
        const ys = series.find((s) => s.curve_name === yCurve);
        if (!xs || !ys || xs.depth.length === 0 || ys.depth.length === 0) {
          built.push({ name: well.well_name, x: [], y: [], dst: [] });
          continue;
        }
        // Zone window + DST intervals for this well.
        let zTop = -Infinity;
        let zBot = Infinity;
        if (zone) {
          try {
            const z = (await listZones(well.well_id)).find((zz) => zz.zone_name === zone);
            if (z) {
              zTop = z.top_depth;
              zBot = z.bottom_depth;
            } else {
              zTop = Infinity; // zone absent → no samples from this well
            }
          } catch {
            /* leave whole-well */
          }
        }
        const intervals: [number, number][] = [];
        if (dstDs) {
          try {
            for (const r of await listAuxData(well.well_id, dstDs)) {
              if (r.depth_base != null && r.depth_base > r.depth_top) intervals.push([r.depth_top, r.depth_base]);
            }
          } catch {
            /* no aux rows */
          }
        }
        const inDst = (d: number): boolean =>
          !dstDs ? false : intervals.some(([t, b]) => d >= t && d < b);
        // Median x-step as the pairing tolerance (curves may sit on slightly different grids).
        const tol = medianStep(xs.depth);
        const x: number[] = [];
        const y: number[] = [];
        const dst: boolean[] = [];
        for (let i = 0; i < xs.depth.length; i++) {
          const d = xs.depth[i];
          if (d < zTop || d >= zBot) continue;
          const vx = xs.value[i];
          const j = nearestDepthIndex(ys.depth, d);
          if (j < 0) continue;
          if (Math.abs(ys.depth[j] - d) > tol) continue;
          const vy = ys.value[j];
          if (Number.isNaN(vx) || Number.isNaN(vy)) continue;
          x.push(vx);
          y.push(vy);
          dst.push(inDst(d));
        }
        built.push({ name: well.well_name, x, y, dst });
      }
      xsets = built;
      // Seed the crosshair from the current cutoff fields (Y=PHIE, X=VSH/SWE by family).
      yCut = numOf(phieIn, DEFAULT_CUTOFFS.phie_min);
      xCut = /sw/i.test(xCurve) ? numOf(sweIn, DEFAULT_CUTOFFS.swe_max) : numOf(vshIn, DEFAULT_CUTOFFS.vsh_max);
      const totalDst = built.reduce((a, s) => a + s.dst.reduce((p, q) => p + (q ? 1 : 0), 0), 0);
      setStatus(`DST crossplot: ${built.length} well(s), ${xCurve} vs ${yCurve}, ${totalDst} DST-tested points`);
      redraw();
    } catch (err) {
      readout.textContent = `Crossplot failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  }

  function medianStep(depth: Float32Array): number {
    if (depth.length < 2) return 0.5;
    const steps: number[] = [];
    for (let i = 1; i < depth.length && steps.length < 200; i++) {
      const d = Math.abs(depth[i] - depth[i - 1]);
      if (d > 0) steps.push(d);
    }
    if (steps.length === 0) return 0.5;
    steps.sort((a, b) => a - b);
    return steps[steps.length >> 1] * 0.75 + 1e-6;
  }

  // === Pointer interaction (place / drag cutoff) ============================
  let dragging: "none" | "sweep" | "xCut" | "yCut" | "both" = "none";
  const localXY = (e: PointerEvent): [number, number] => {
    const r = canvas.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  };
  const onDown = (e: PointerEvent) => {
    if (!pc) return;
    const [px, py] = localXY(e);
    if (!pc.inPlot(px, py)) return;
    canvas.setPointerCapture(e.pointerId);
    if (mode === "sweep") {
      dragging = "sweep";
      const [dx] = pc.toData(px, py);
      pickedCutoff = clampX(dx);
      redraw();
    } else {
      const [vpx] = pc.toPx(xCut, pc.y.min);
      const [, hpy] = pc.toPx(pc.x.min, yCut);
      const nearV = Math.abs(px - vpx) < 12;
      const nearH = Math.abs(py - hpy) < 12;
      dragging = nearV && !nearH ? "xCut" : nearH && !nearV ? "yCut" : "both";
      applyDrag(px, py);
    }
  };
  const onMove = (e: PointerEvent) => {
    if (dragging === "none" || !pc) return;
    const [px, py] = localXY(e);
    if (mode === "sweep") {
      const [dx] = pc.toData(px, py);
      pickedCutoff = clampX(dx);
      redraw();
    } else {
      applyDrag(px, py);
    }
  };
  const onUp = (e: PointerEvent) => {
    dragging = "none";
    try {
      canvas.releasePointerCapture(e.pointerId);
    } catch {
      /* capture may already be gone */
    }
  };
  function clampX(v: number): number {
    if (!sweepReq) return v;
    return Math.min(sweepReq.sweepMax, Math.max(sweepReq.sweepMin, v));
  }
  function applyDrag(px: number, py: number): void {
    if (!pc) return;
    const [dx, dy] = pc.toData(px, py);
    if (dragging === "xCut" || dragging === "both") xCut = clampAxis(dx, pc.x.min, pc.x.max);
    if (dragging === "yCut" || dragging === "both") yCut = clampAxis(dy, pc.y.min, pc.y.max);
    redraw();
  }
  function clampAxis(v: number, lo: number, hi: number): number {
    return Math.min(Math.max(v, Math.min(lo, hi)), Math.max(lo, hi));
  }
  canvas.addEventListener("pointerdown", onDown);
  canvas.addEventListener("pointermove", onMove);
  canvas.addEventListener("pointerup", onUp);
  // pointercancel also implicitly releases capture; without this the drag state would stick and
  // the crosshair/pick would then follow the bare cursor with no button held.
  canvas.addEventListener("pointercancel", onUp);

  // === Buttons / wiring =====================================================
  runBtn.addEventListener("click", () => {
    if (mode === "sweep") void computeSweep();
    else void computeCrossplot();
  });

  usePickBtn.addEventListener("click", () => {
    if (mode === "sweep") {
      if (pickedCutoff === null) {
        setStatus("Place a cutoff on the plot first.");
        return;
      }
      cutFor(property).value = String(round3(pickedCutoff));
      setStatus(`${property} cutoff set to ${round3(pickedCutoff)}. Save it as the pay-summary default, or Compute again.`);
    } else {
      phieIn.value = String(round3(yCut));
      const xIsSw = /sw/i.test(xAxisLabel);
      if (xIsSw) {
        sweIn.value = String(round3(xCut));
        setStatus(`Applied crosshair: PHIE ≥ ${round3(yCut)}, SWE ≤ ${round3(xCut)}.`);
      } else {
        vshIn.value = String(round3(xCut));
        setStatus(`Applied crosshair: PHIE ≥ ${round3(yCut)}, VSH ≤ ${round3(xCut)} (from ${xAxisLabel}).`);
      }
    }
  });

  saveDefaultBtn.addEventListener("click", async () => {
    const permRaw = parseFloat(permIn.value);
    const payload = {
      vsh_max: numOf(vshIn, DEFAULT_CUTOFFS.vsh_max),
      phie_min: numOf(phieIn, DEFAULT_CUTOFFS.phie_min),
      swe_max: numOf(sweIn, DEFAULT_CUTOFFS.swe_max),
      perm_min: Number.isFinite(permRaw) ? permRaw : null,
    };
    try {
      await saveDocument("cutoffs", "__default__", JSON.stringify(payload));
      bumpDataVersion();
      recordProcess("Cutoffs", `Saved default cutoffs (VSH ≤ ${payload.vsh_max}, PHIE ≥ ${payload.phie_min}, SWE ≤ ${payload.swe_max})`);
      setStatus(
        `Saved pay-summary default cutoffs (VSH ≤ ${payload.vsh_max}, PHIE ≥ ${payload.phie_min}, SWE ≤ ${payload.swe_max}). Open Cutoffs & Pay Summary to use them.`,
      );
    } catch (err) {
      setStatus(`Could not save cutoffs: ${err}`);
    }
  });

  // Changing what is swept or measured makes the plotted sweep stale: the pick sits on the
  // old axis and the readout describes the old run. Clear it so "Use pick" can't write into a
  // property/metric it wasn't computed for (which would persist a nonsensical pay cutoff).
  const invalidateSweep = () => {
    if (!sweep && !sweepReq && pickedCutoff === null) return;
    sweep = null;
    sweepReq = null;
    pickedCutoff = null;
    if (mode === "sweep") redraw();
  };
  for (const p of ["VSH", "PHIE", "SWE"] as const) {
    propBtns[p].addEventListener("click", () => {
      property = p;
      for (const q of ["VSH", "PHIE", "SWE"] as const) propBtns[q].classList.toggle("active", q === p);
      applyPropDefaults();
      usePickBtn.textContent = `Use pick as ${property} cutoff`;
      invalidateSweep();
    });
  }
  for (const m of ["NET", "HPV", "NTG"] as const) {
    metricBtns[m].addEventListener("click", () => {
      metric = m;
      for (const q of ["NET", "HPV", "NTG"] as const) metricBtns[q].classList.toggle("active", q === m);
      invalidateSweep();
    });
  }
  sweepModeBtn.addEventListener("click", () => {
    mode = "sweep";
    setModeButtons();
    redraw();
  });
  xplotModeBtn.addEventListener("click", () => {
    mode = "crossplot";
    setModeButtons();
    redraw();
  });
  normChk.addEventListener("change", () => redraw());
  vclayPreset.addEventListener("click", () => {
    xCurveIn.value = "VOL_WETCLAY";
    yCurveIn.value = "PHIE";
  });
  swPreset.addEventListener("click", () => {
    xCurveIn.value = "SW";
    yCurveIn.value = "PHIE";
  });

  setModeButtons();
  const disposeResize = attachResizeRedraw(canvas, redraw);
  // Repaint on live theme swaps like every other Canvas-2D pane, else the plot keeps the old
  // palette until an unrelated interaction; the returned unsub is released in dispose.
  const unsubTheme = appState.themeVersion.subscribe(() => redraw());
  redraw();

  return {
    el: root,
    dispose: () => {
      disposeResize();
      unsubTheme();
      scope.dispose();
      canvas.removeEventListener("pointerdown", onDown);
      canvas.removeEventListener("pointermove", onMove);
      canvas.removeEventListener("pointerup", onUp);
      canvas.removeEventListener("pointercancel", onUp);
    },
  };
}

// --- small DOM helpers ------------------------------------------------------
function segButton(bar: HTMLElement, text: string, active: boolean): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "cutoff-seg-btn" + (active ? " active" : "");
  b.type = "button";
  b.textContent = text;
  bar.appendChild(b);
  return b;
}

function presetButton(bar: HTMLElement, text: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "cutoff-btn";
  b.type = "button";
  b.textContent = text;
  bar.appendChild(b);
  return b;
}

function rowPair(a: HTMLElement, b: HTMLElement): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "cutoff-pair";
  wrap.appendChild(a);
  wrap.appendChild(b);
  return wrap;
}

function round3(v: number): number {
  return Math.round(v * 1000) / 1000;
}
