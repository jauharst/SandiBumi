import {
  coreExtraDatasets,
  listCoreReferences,
  listCurveCatalog,
  proposeRegistration,
  shiftCoreData,
  type CoreReference,
  type RegistrationResult,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { fitCanvasBackingStore, readTheme, canvasFont } from "./plotCanvas";

/** Core-to-log depth registration (Data ▸ Core ▸ Register Depth…).
 *
 *  Core arrives on the driller's tally and the log on the wireline's; the difference is real,
 *  routine, and invisible afterwards. Until now the only tool for it was a number typed into
 *  "Shift Core" — you had to already know the answer.
 *
 *  Two things here are deliberate and worth keeping:
 *
 *  **The proposal is never applied.** The backend returns a shift and the whole correlogram, and
 *  the user accepts it. A correlation peak in a repeated sand can be confidently wrong, which is
 *  why the correlogram is drawn rather than summarised — a single sharp peak and a comb of
 *  near-equal ones are the same number and completely different situations.
 *
 *  **What moves is stated before it moves.** A depth shift drags the measurements made on those
 *  plugs along with them, and which point datasets belong to this core is a core-handling
 *  judgement. The checkboxes name them; the app does not decide silently.
 */
export async function openDepthRegDialog(): Promise<void> {
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  const close = openModal(
    well ? `Register core depth — ${well.well_name}` : "Register core depth",
    wrap,
    720
  );

  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent = "Select a well in the Wells pane first — registration is judged one well at a time.";
    wrap.appendChild(none);
    return;
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Finds the constant shift that puts this well's core back on the log's depth scale. " +
    "Nothing is written until you accept it.";
  wrap.appendChild(intro);

  // ---- the log side -------------------------------------------------------
  const logSel = document.createElement("select");
  logSel.className = "form-control";
  const catalog = await listCurveCatalog().catch(() => []);
  for (const c of catalog) {
    const o = document.createElement("option");
    o.value = c.name;
    o.textContent = c.units ? `${c.name} (${c.units})` : c.name;
    logSel.appendChild(o);
  }
  if (catalog.some((c) => c.name.toUpperCase() === "GR")) logSel.value = "GR";
  wrap.appendChild(formRow("Log curve", logSel, "The wireline curve the core is registered against"));

  // ---- the core side ------------------------------------------------------
  const refSel = document.createElement("select");
  refSel.className = "form-control";
  const refs = await listCoreReferences(well.well_id).catch(() => [] as CoreReference[]);
  // The option value is the INDEX, not a composite of kind/dataset/item: a dataset name can
  // contain spaces ("CORE GAMMA"), so any joined key is ambiguous to read back.
  refs.forEach((r, i) => {
    const o = document.createElement("option");
    o.value = String(i);
    o.textContent = r.label;
    refSel.appendChild(o);
  });
  // A delivered core gamma is the strongest reference there is, so pick it when it exists.
  const gammaIdx = refs.findIndex((r) => r.family === "GR");
  if (gammaIdx >= 0) refSel.value = String(gammaIdx);
  wrap.appendChild(
    formRow("Core reference", refSel, "What the core measured at each plug depth")
  );

  const pairNote = document.createElement("div");
  pairNote.className = "eq-note";
  wrap.appendChild(pairNote);

  const describePair = (): void => {
    const ref = refs[Number(refSel.value)];
    const logFam = logSel.value.toUpperCase();
    if (!ref) {
      pairNote.textContent = refs.length
        ? ""
        : "This well has no core measurement with enough samples to correlate. Import a core table, " +
          "or a core gamma as point data, first.";
      return;
    }
    // The dialog cannot resolve families itself — the backend owns that table — so this is
    // phrased as an expectation, and the RESULT states what the pairing actually was.
    if (ref.family && ref.family === logFam) {
      pairNote.textContent =
        `${ref.item} and ${logSel.value} look like the same measurement, so this should be a ` +
        "like-for-like match: they must rise and fall together.";
    } else {
      pairNote.textContent =
        `${ref.item} and ${logSel.value} measure different things. That still registers — the ` +
        "shift is chosen on the strength of the relationship, not its sign — but read the " +
        "coefficient as a shape match rather than agreement.";
    }
  };
  refSel.addEventListener("change", describePair);
  logSel.addEventListener("change", describePair);
  describePair();

  // ---- interval + search --------------------------------------------------
  const topIn = document.createElement("input");
  topIn.className = "form-control";
  topIn.type = "number";
  topIn.step = "0.1";
  const baseIn = document.createElement("input");
  baseIn.className = "form-control";
  baseIn.type = "number";
  baseIn.step = "0.1";
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
    formRow(
      "Interval top / base",
      depthRow,
      sel ? "Seeded from the selected top" : "Optional — leave blank for the whole cored interval, or set one core run"
    )
  );

  const rangeIn = document.createElement("input");
  rangeIn.className = "form-control";
  rangeIn.type = "number";
  rangeIn.step = "0.5";
  rangeIn.value = "5";
  wrap.appendChild(formRow("Search ±", rangeIn, "How far the core might be out, in the project depth unit"));

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Propose a shift";
  runBtn.disabled = refs.length === 0;
  wrap.appendChild(runBtn);

  const out = document.createElement("div");
  wrap.appendChild(out);

  // ---------------------------------------------------------------------------
  // Drawing
  // ---------------------------------------------------------------------------

  /** Depth view: the log as a line, the core as points at its CURRENT depths and again at the
   *  proposed ones. Both series are scaled to their own range — the shapes are what is being
   *  compared, and a porosity against a gamma has no shared axis. */
  function drawOverlay(canvas: HTMLCanvasElement, res: RegistrationResult, delta: number): void {
    const dpr = fitCanvasBackingStore(canvas);
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const theme = readTheme(canvas);
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);

    const pad = { l: 8, r: 8, t: 18, b: 22 };
    const plotW = w - pad.l - pad.r;
    const plotH = h - pad.t - pad.b;
    if (plotW <= 4 || plotH <= 4) return;

    // Depth window: the core, plus the shift, plus a margin — not the whole log, which would
    // squeeze the cored section into a few pixels on a 3000 m well.
    const cd = res.core.map((p) => p.depth);
    if (!cd.length) return;
    const margin = Math.max(2, Math.abs(delta) * 2);
    const dMin = Math.min(...cd) - margin;
    const dMax = Math.max(...cd) + margin;
    const y = (d: number): number => pad.t + ((d - dMin) / (dMax - dMin || 1)) * plotH;

    const span = (vals: number[]): [number, number] => {
      const f = vals.filter((v) => Number.isFinite(v));
      if (!f.length) return [0, 1];
      const lo = Math.min(...f);
      const hi = Math.max(...f);
      return hi > lo ? [lo, hi] : [lo - 1, lo + 1];
    };
    const [logLo, logHi] = span(res.log_value);
    const [coreLo, coreHi] = span(res.core.map((p) => p.value));
    const xLog = (v: number): number => pad.l + ((v - logLo) / (logHi - logLo)) * plotW;
    const xCore = (v: number): number => pad.l + ((v - coreLo) / (coreHi - coreLo)) * plotW;

    // The log, depth-clipped to the window.
    ctx.strokeStyle = theme.axis;
    ctx.lineWidth = 1;
    ctx.beginPath();
    let started = false;
    for (let i = 0; i < res.log_depth.length; i++) {
      const d = res.log_depth[i];
      const v = res.log_value[i];
      if (d < dMin || d > dMax || !Number.isFinite(v)) {
        started = false;
        continue;
      }
      const px = xLog(v);
      const py = y(d);
      if (!started) {
        ctx.moveTo(px, py);
        started = true;
      } else ctx.lineTo(px, py);
    }
    ctx.stroke();

    // Core where it sits now (hollow) and where it would sit (filled).
    for (const p of res.core) {
      const px = xCore(p.value);
      ctx.strokeStyle = theme.grid;
      ctx.beginPath();
      ctx.arc(px, y(p.depth), 2.5, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.fillStyle = theme.accent;
    for (const p of res.core) {
      const d = p.depth + delta;
      if (d < dMin || d > dMax) continue; // off-scale samples are skipped, never clamped
      ctx.beginPath();
      ctx.arc(xCore(p.value), y(d), 2.5, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.font = canvasFont(theme, 11);
    ctx.fillStyle = theme.text;
    ctx.textAlign = "left";
    ctx.fillText(`${res.reference_label} vs ${logSel.value}`, pad.l, 12);
    ctx.textAlign = "right";
    ctx.fillText(`${dMin.toFixed(0)}–${dMax.toFixed(0)}`, w - pad.r, h - 6);
    ctx.textAlign = "left";
    ctx.fillText("hollow = now, solid = proposed", pad.l, h - 6);
  }

  /** The correlogram. This is the honest confidence statement: one sharp peak means the shift is
   *  well determined, several near-equal peaks mean the section repeats and the maximum is a
   *  coin toss. */
  function drawScan(canvas: HTMLCanvasElement, res: RegistrationResult, delta: number): void {
    const dpr = fitCanvasBackingStore(canvas);
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const theme = readTheme(canvas);
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);

    const pad = { l: 34, r: 8, t: 16, b: 22 };
    const plotW = w - pad.l - pad.r;
    const plotH = h - pad.t - pad.b;
    if (plotW <= 4 || plotH <= 4 || !res.scan.length) return;

    const dLo = res.scan[0].delta;
    const dHi = res.scan[res.scan.length - 1].delta;
    const x = (d: number): number => pad.l + ((d - dLo) / (dHi - dLo || 1)) * plotW;
    // Always the full −1..1, never the data's own range: a correlogram cropped to its extremes
    // makes a weak peak look decisive.
    const y = (r: number): number => pad.t + ((1 - r) / 2) * plotH;

    ctx.strokeStyle = theme.grid;
    ctx.lineWidth = 1;
    for (const r of [-1, -0.5, 0, 0.5, 1]) {
      ctx.beginPath();
      ctx.moveTo(pad.l, y(r));
      ctx.lineTo(w - pad.r, y(r));
      ctx.stroke();
    }
    ctx.beginPath();
    ctx.moveTo(x(0), pad.t);
    ctx.lineTo(x(0), pad.t + plotH);
    ctx.stroke();

    ctx.strokeStyle = theme.accent2;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    let started = false;
    for (const p of res.scan) {
      if (!Number.isFinite(p.r)) {
        started = false;
        continue;
      }
      const px = x(p.delta);
      const py = y(p.r);
      if (!started) {
        ctx.moveTo(px, py);
        started = true;
      } else ctx.lineTo(px, py);
    }
    ctx.stroke();

    ctx.strokeStyle = theme.accent;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(x(delta), pad.t);
    ctx.lineTo(x(delta), pad.t + plotH);
    ctx.stroke();

    ctx.font = canvasFont(theme, 11);
    ctx.fillStyle = theme.text;
    ctx.textAlign = "right";
    ctx.fillText("+1", pad.l - 4, y(1) + 4);
    ctx.fillText("0", pad.l - 4, y(0) + 4);
    ctx.fillText("−1", pad.l - 4, y(-1) + 4);
    ctx.textAlign = "center";
    ctx.fillText(`shift (${dLo.toFixed(1)} … ${dHi.toFixed(1)})`, pad.l + plotW / 2, h - 6);
  }

  // ---------------------------------------------------------------------------
  // Run + accept
  // ---------------------------------------------------------------------------

  runBtn.addEventListener("click", () => {
    void (async () => {
      const ref = refs[Number(refSel.value)];
      if (!ref) return;
      runBtn.disabled = true;
      runBtn.textContent = "Scanning…";
      out.innerHTML = "";
      let res: RegistrationResult;
      try {
        res = await proposeRegistration({
          well_id: well.well_id,
          log_curve: logSel.value,
          ref_kind: ref.kind,
          ref_dataset: ref.dataset,
          ref_item: ref.item,
          depth_from: topIn.value ? Number(topIn.value) : null,
          depth_to: baseIn.value ? Number(baseIn.value) : null,
          search_range: Number(rangeIn.value) || 5,
        });
      } finally {
        runBtn.disabled = false;
        runBtn.textContent = "Propose a shift";
      }
      if (res.error) {
        const err = document.createElement("div");
        err.className = "eq-note";
        err.style.color = "var(--warn)";
        err.textContent = res.error;
        out.appendChild(err);
        return;
      }
      renderResult(res);
    })();
  });

  function renderResult(res: RegistrationResult): void {
    out.innerHTML = "";

    const table = document.createElement("table");
    table.className = "data-table";
    const add = (k: string, v: string, hint?: string): void => {
      const tr = document.createElement("tr");
      const th = document.createElement("th");
      th.textContent = k;
      const td = document.createElement("td");
      td.textContent = v;
      if (hint) td.title = hint;
      tr.appendChild(th);
      tr.appendChild(td);
      table.appendChild(tr);
    };
    add("Proposed shift", `${res.proposed_delta > 0 ? "+" : ""}${res.proposed_delta.toFixed(2)}`);
    add(
      "Correlation",
      `${res.correlation.toFixed(3)} (${res.matched_on})`,
      res.like_for_like
        ? "Like-for-like pairing: chosen on the correlation itself."
        : "Proxy pairing: chosen on |r|, so the sign is reported rather than assumed."
    );
    add("Where it sits now", res.current_r.toFixed(3), "Correlation at zero shift, for comparison");
    add("Paired samples", String(res.n_pairs));
    add("Pairing", res.like_for_like ? `like-for-like (${res.log_family})` : "proxy");
    out.appendChild(table);

    const canvasRow = document.createElement("div");
    canvasRow.style.display = "flex";
    canvasRow.style.gap = "8px";
    canvasRow.style.margin = "8px 0";
    const overlay = document.createElement("canvas");
    overlay.className = "plot-canvas";
    overlay.style.flex = "1";
    overlay.style.height = "260px";
    const scan = document.createElement("canvas");
    scan.className = "plot-canvas";
    scan.style.flex = "1";
    scan.style.height = "260px";
    canvasRow.appendChild(overlay);
    canvasRow.appendChild(scan);
    out.appendChild(canvasRow);

    const deltaIn = document.createElement("input");
    deltaIn.className = "form-control";
    deltaIn.type = "number";
    deltaIn.step = "0.05";
    deltaIn.value = res.proposed_delta.toFixed(2);
    out.appendChild(
      formRow("Shift to apply", deltaIn, "+ = the core moves deeper. Override the proposal freely.")
    );

    const repaint = (): void => {
      const d = Number(deltaIn.value);
      drawOverlay(overlay, res, Number.isFinite(d) ? d : res.proposed_delta);
      drawScan(scan, res, Number.isFinite(d) ? d : res.proposed_delta);
    };
    deltaIn.addEventListener("input", repaint);
    // First paint is synchronous: rAF does not fire in a window that is not compositing, and
    // there is no resize fallback here either.
    repaint();

    for (const n of res.notes) {
      const note = document.createElement("div");
      note.className = "eq-note";
      note.textContent = n;
      out.appendChild(note);
    }

    // ---- what rides along ---------------------------------------------------
    const ridersWrap = document.createElement("div");
    out.appendChild(ridersWrap);
    const boxes: HTMLInputElement[] = [];
    void coreExtraDatasets(well!.well_id)
      .then((sets) => {
        if (!sets.length) return;
        const lead = document.createElement("div");
        lead.className = "eq-note";
        lead.textContent =
          "These point datasets were delivered with this core. A measurement made on a plug " +
          "must move with that plug, or it ends up registered against rock it was never taken from.";
        ridersWrap.appendChild(lead);
        for (const [dataset, rows] of sets) {
          const lab = document.createElement("label");
          lab.style.display = "block";
          const cb = document.createElement("input");
          cb.type = "checkbox";
          cb.checked = true;
          cb.value = dataset;
          boxes.push(cb);
          lab.appendChild(cb);
          lab.appendChild(document.createTextNode(` ${dataset} — ${rows} row(s)`));
          ridersWrap.appendChild(lab);
        }
      })
      .catch(() => {});

    const applyBtn = document.createElement("button");
    applyBtn.className = "btn btn-accent";
    applyBtn.textContent = "Apply this shift";
    applyBtn.addEventListener("click", () => {
      const delta = Number(deltaIn.value);
      if (!Number.isFinite(delta) || delta === 0) {
        setStatus("Enter a non-zero shift");
        return;
      }
      const datasets = boxes.filter((b) => b.checked).map((b) => b.value);
      void (async () => {
        const n = await shiftCoreData(well!.well_id, delta, datasets);
        const sign = delta > 0 ? "+" : "";
        setStatus(
          `Shifted ${n.plugs} plug(s) and ${n.extras} point sample(s) of ${well!.well_name} by ${sign}${delta}`
        );
        recordProcess(
          "Edit",
          `Core registration ${sign}${delta} on ${res.reference_label} vs ${logSel.value} (r = ${res.correlation.toFixed(2)})`,
          well!.well_name
        );
        pushUndo({
          label: `core registration ${sign}${delta} (${well!.well_name})`,
          undo: async () => {
            await shiftCoreData(well!.well_id, -delta, datasets);
            bumpDataVersion();
          },
          redo: async () => {
            await shiftCoreData(well!.well_id, delta, datasets);
            bumpDataVersion();
          },
        });
        bumpDataVersion();
        close();
      })();
    });
    out.appendChild(applyBtn);
  }
}
