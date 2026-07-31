import {
  applyCoreRunShifts,
  coreDepthPairs,
  coreShiftCandidates,
  listCoreReferences,
  listCoreRegistrations,
  listCurveCatalog,
  proposeRegistration,
  shiftCoreData,
  type CoreReference,
  type ShiftCandidate,
  type ShiftTargets,
  type RegistrationNote,
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

  const barrels = document.createElement("div");
  wrap.appendChild(barrels);

  const history = document.createElement("div");
  wrap.appendChild(history);

  // ---- what rides along ---------------------------------------------------
  // A core registration moves rock that other deliveries were measured on. Which ones belong to
  // the core is recorded at import (the "these depths came from the core report" tick-box), so
  // those are pre-ticked here — but everything live is LISTED, because the flag only exists for
  // deliveries imported since it did, and an older project would otherwise look empty.
  const ridersWrap = document.createElement("div");
  out.appendChild(ridersWrap);
  const boxes: { el: HTMLInputElement; cand: ShiftCandidate }[] = [];
  void coreShiftCandidates(well!.well_id)
    .then((cands) => {
      if (!cands.length) return;
      const lead = document.createElement("div");
      lead.className = "eq-note";
      lead.textContent =
        "These deliveries can move with the core. A measurement made on a plug must move with " +
        "that plug, or it ends up registered against rock it was never taken from — and anything " +
        "already on the log's own depth scale must not move at all. Ticked ones were imported as " +
        "core-depth data.";
      ridersWrap.appendChild(lead);
      const label = (c: ShiftCandidate): string => {
        const what =
          c.kind === "scal" ? `SCAL ${c.set_name}` : `${c.dataset} (${c.set_name})`;
        const unit = c.kind === "image" ? "picture(s)" : c.kind === "scal" ? "Pc point(s)" : "row(s)";
        const basis = c.on_core_depths ? "" : " — not marked as core-depth data";
        return ` ${what} — ${c.rows} ${unit}${basis}`;
      };
      for (const c of cands) {
        const lab = document.createElement("label");
        lab.style.display = "block";
        const cb = document.createElement("input");
        cb.type = "checkbox";
        cb.checked = c.on_core_depths;
        boxes.push({ el: cb, cand: c });
        lab.appendChild(cb);
        lab.appendChild(document.createTextNode(label(c)));
        ridersWrap.appendChild(lab);
      }
    })
    .catch(() => {});

  /** The ticked deliveries, in the shape the backend takes. */
  const chosenTargets = (): ShiftTargets => {
    const picked = boxes.filter((b) => b.el.checked).map((b) => b.cand);
    return {
      aux_datasets: picked.filter((c) => c.kind === "aux").map((c) => c.dataset),
      scal: picked.some((c) => c.kind === "scal"),
      image_datasets: picked.filter((c) => c.kind === "image").map((c) => c.dataset),
    };
  };

  /** Agreement at the shift the user is ACTUALLY applying, read off the scan. They are free to
   *  overrule the proposal, and recording the peak instead would describe an alignment nobody
   *  chose — a good number filed against a shift it does not belong to. */
  function correlationAt(res: RegistrationResult, delta: number): { r: number | null; n: number | null } {
    let best: { r: number; n: number; gap: number } | null = null;
    for (const p of res.scan) {
      const gap = Math.abs(p.delta - delta);
      if (!best || gap < best.gap) best = { r: p.r, n: p.n, gap };
    }
    // Outside the scanned window there is no measured agreement, and inventing one by
    // extrapolation would put a number in the record that was never computed.
    if (!best || best.gap > 0.51) return { r: null, n: null };
    return { r: best.r, n: best.n };
  }

  /** Why this shift is being applied, in the shape the record takes. */
  function noteFor(res: RegistrationResult | null, delta: number, kind = "proposed"): RegistrationNote {
    if (!res) return { kind: "manual", note: "typed without a proposal on screen" };
    const at = correlationAt(res, delta);
    return {
      kind,
      log_curve: logSel.value,
      reference: res.reference_label,
      pairing: res.like_for_like ? "like-for-like" : `proxy (${res.matched_on})`,
      correlation: at.r,
      n_pairs: at.n,
    };
  }


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

    const applyBtn = document.createElement("button");
    applyBtn.className = "btn btn-accent";
    applyBtn.textContent = "Apply this shift";
    applyBtn.addEventListener("click", () => {
      const delta = Number(deltaIn.value);
      if (!Number.isFinite(delta) || delta === 0) {
        setStatus("Enter a non-zero shift");
        return;
      }
      const targets = chosenTargets();
      void (async () => {
        const n = await shiftCoreData(well!.well_id, delta, targets, noteFor(res, delta));
        const sign = delta > 0 ? "+" : "";
        setStatus(
          `Shifted ${n.plugs} plug(s), ${n.extras} point sample(s), ${n.scal} Pc point(s) and ${n.plates} picture(s) of ${well!.well_name} by ${sign}${delta}`
        );
        recordProcess(
          "Edit",
          `Core registration ${sign}${delta} on ${res.reference_label} vs ${logSel.value} (r = ${res.correlation.toFixed(2)})`,
          well!.well_name
        );
        pushUndo({
          label: `core registration ${sign}${delta} (${well!.well_name})`,
          undo: async () => {
            // The reversal is recorded too. A core that was registered and put back is not the
            // same as one nobody touched, and only the log still knows the difference.
            await shiftCoreData(well!.well_id, -delta, targets, {
              kind: "undo",
              note: `reverses ${sign}${delta}`,
            });
            bumpDataVersion();
          },
          redo: async () => {
            await shiftCoreData(well!.well_id, delta, targets, noteFor(res, delta));
            bumpDataVersion();
          },
        });
        bumpDataVersion();
        close();
      })();
    });
    out.appendChild(applyBtn);
  }

  // ---------------------------------------------------------------------------
  // One shift per barrel
  // ---------------------------------------------------------------------------

  /** Core comes up a barrel at a time and each barrel carries its own tally error, so one number
   *  for a whole well is right in the middle of the cored interval and wrong at both ends. Pieces
   *  can also move INSIDE a barrel between the core face and the lab, which is why these are free
   *  intervals rather than a fixed barrel length — split one row into two and shift each half. */
  async function buildBarrels(): Promise<void> {
    barrels.innerHTML = "";
    const head = document.createElement("div");
    head.className = "eq-note";
    head.innerHTML =
      "<b>One shift per barrel</b> — core comes up a barrel at a time and each one is out by its " +
      "own amount, so a single number for the whole well is right in the middle and wrong at both " +
      "ends. Propose each range separately. Pieces that moved inside a barrel just mean a shorter " +
      "range: split the row and shift each part.";
    barrels.appendChild(head);

    // What the core already carries, so a second pass is not applied on top of a forgotten first.
    const pairs = await coreDepthPairs(well!.well_id).catch(() => [] as [number, number][]);
    if (pairs.length) {
      const offs = pairs.map(([o, d]) => d - o);
      const lo = Math.min(...offs);
      const hi = Math.max(...offs);
      const rec = document.createElement("div");
      rec.className = "eq-note";
      rec.textContent =
        Math.abs(hi - lo) < 1e-3
          ? Math.abs(hi) < 1e-3
            ? "This core is still exactly where the lab put it — no shift recorded yet."
            : `Already moved by ${hi.toFixed(2)} throughout. Anything you apply now adds to that.`
          : `Already moved by between ${lo.toFixed(2)} and ${hi.toFixed(2)} down the hole. ` +
            "Anything you apply now adds to that.";
      barrels.appendChild(rec);
    }

    const table = document.createElement("table");
    table.className = "data-table";
    const hrow = document.createElement("tr");
    for (const h of ["Top", "Base", "Shift", "", ""]) {
      const th = document.createElement("th");
      th.textContent = h;
      hrow.appendChild(th);
    }
    table.appendChild(hrow);
    barrels.appendChild(table);

    // `res` is this barrel's OWN proposal, kept so the depth record can carry the agreement that
    // justified this range rather than one number for the whole apply.
    const rows: {
      top: HTMLInputElement;
      base: HTMLInputElement;
      delta: HTMLInputElement;
      res: RegistrationResult | null;
    }[] = [];

    const addRow = (top = "", base = "", delta = ""): void => {
      const tr = document.createElement("tr");
      const mk = (v: string, step: string): HTMLInputElement => {
        const i = document.createElement("input");
        i.className = "form-control";
        i.type = "number";
        i.step = step;
        i.value = v;
        const td = document.createElement("td");
        td.appendChild(i);
        tr.appendChild(td);
        return i;
      };
      const topIn2 = mk(top, "0.1");
      const baseIn2 = mk(base, "0.1");
      const deltaIn2 = mk(delta, "0.05");

      const prop = document.createElement("button");
      prop.className = "btn";
      prop.textContent = "Propose";
      prop.addEventListener("click", () => {
        const ref = refs[Number(refSel.value)];
        if (!ref) return;
        prop.disabled = true;
        prop.textContent = "…";
        void (async () => {
          try {
            const res = await proposeRegistration({
              well_id: well!.well_id,
              log_curve: logSel.value,
              ref_kind: ref.kind,
              ref_dataset: ref.dataset,
              ref_item: ref.item,
              depth_from: topIn2.value ? Number(topIn2.value) : null,
              depth_to: baseIn2.value ? Number(baseIn2.value) : null,
              search_range: Number(rangeIn.value) || 5,
            });
            if (res.error) {
              setStatus(res.error);
              return;
            }
            deltaIn2.value = res.proposed_delta.toFixed(2);
            const mine = rows.find((r) => r.top === topIn2);
            if (mine) mine.res = res;
            // Show this barrel's own correlogram — each range is judged on its own evidence.
            renderResult(res);
          } finally {
            prop.disabled = false;
            prop.textContent = "Propose";
          }
        })();
      });
      const td1 = document.createElement("td");
      td1.appendChild(prop);
      tr.appendChild(td1);

      const del = document.createElement("button");
      del.className = "btn";
      del.textContent = "✕";
      del.title = "Remove this range";
      del.addEventListener("click", () => {
        const i = rows.findIndex((r) => r.top === topIn2);
        if (i >= 0) rows.splice(i, 1);
        tr.remove();
      });
      const td2 = document.createElement("td");
      td2.appendChild(del);
      tr.appendChild(td2);

      table.appendChild(tr);
      rows.push({ top: topIn2, base: baseIn2, delta: deltaIn2, res: null });
    };

    const addBtn = document.createElement("button");
    addBtn.className = "btn";
    addBtn.textContent = "Add a barrel";
    addBtn.addEventListener("click", () => addRow());
    barrels.appendChild(addBtn);

    const applyAll = document.createElement("button");
    applyAll.className = "btn btn-accent";
    applyAll.textContent = "Apply all barrels";
    applyAll.addEventListener("click", () => {
      const runs = rows
        .map((r) => {
          const delta = Number(r.delta.value);
          // Each barrel carries the evidence for its OWN range. A row the user typed without
          // proposing carries none, which the record shows as blank rather than as a zero.
          const at = r.res ? correlationAt(r.res, delta) : { r: null, n: null };
          return {
            top: Number(r.top.value),
            base: Number(r.base.value),
            delta,
            correlation: at.r,
            n_pairs: at.n,
          };
        })
        .filter((r) => Number.isFinite(r.top) && Number.isFinite(r.base) && Number.isFinite(r.delta) && r.delta !== 0);
      if (!runs.length) {
        setStatus("Fill in at least one range with a non-zero shift");
        return;
      }
      void (async () => {
        try {
          const anyProposed = rows.some((r) => r.res);
          const barrelNote: RegistrationNote = {
            kind: anyProposed ? "proposed" : "manual",
            log_curve: anyProposed ? logSel.value : "",
            reference: anyProposed ? (rows.find((r) => r.res)?.res?.reference_label ?? "") : "",
            note: `${runs.length} barrel(s)`,
          };
          const n = await applyCoreRunShifts(well!.well_id, runs, chosenTargets(), barrelNote);
          setStatus(`Moved ${n.plugs} plug(s), ${n.extras} point sample(s), ${n.scal} Pc point(s) and ${n.plates} picture(s) across ${runs.length} barrel(s)`);
          recordProcess(
            "Edit",
            `Core registration, ${runs.length} barrel(s): ${runs.map((r) => `${r.top}-${r.base} ${r.delta > 0 ? "+" : ""}${r.delta}`).join(", ")}`,
            well!.well_name
          );
          pushUndo({
            label: `core barrel shifts (${well!.well_name})`,
            // The backend hands back the ranges that undo this, because it knows where the plugs
            // landed. Negating the deltas and shifting these ranges here looks equivalent and is
            // not: two barrels moved by different amounts can produce overlapping ranges, and the
            // first match wins, so some plugs would come back by the wrong correction.
            undo: async () => {
              await applyCoreRunShifts(well!.well_id, n.inverse, chosenTargets(), {
                kind: "undo",
                note: `reverses ${runs.length} barrel(s)`,
              });
              await buildBarrels();
              await buildHistory();
              bumpDataVersion();
            },
            redo: async () => {
              await applyCoreRunShifts(well!.well_id, runs, chosenTargets(), barrelNote);
              await buildBarrels();
              await buildHistory();
              bumpDataVersion();
            },
          });
          await buildBarrels();
          await buildHistory();
          bumpDataVersion();
        } catch (err) {
          // The backend refuses anything that would reorder the core and changes nothing, so
          // this is a message to read, not a failure to recover from.
          setStatus(String(err));
        }
      })();
    });
    barrels.appendChild(applyAll);

    if (!rows.length) addRow();
  }

  // ---------------------------------------------------------------------------
  // Why this core sits where it does
  // ---------------------------------------------------------------------------

  /** The well's depth history. An EVENT LOG, not a summary of the current position: an undo
   *  appears as its own reversal, because a core that was registered, judged wrong and put back
   *  is not the same as a core nobody ever touched, and next year only this can tell them
   *  apart. */
  async function buildHistory(): Promise<void> {
    history.innerHTML = "";
    let log;
    try {
      log = await listCoreRegistrations(well!.well_id);
    } catch {
      return;
    }

    const head = document.createElement("div");
    head.className = "field-label";
    head.textContent = "Why this core sits where it does";
    history.appendChild(head);

    if (!log.length) {
      const none = document.createElement("div");
      none.className = "eq-note";
      none.textContent =
        "This core has never been shifted. Its plugs are at the depths the laboratory delivered.";
      history.appendChild(none);
      return;
    }

    const table = document.createElement("table");
    table.className = "data-table";
    const hrow = document.createElement("tr");
    for (const h of ["When", "Set", "Interval", "Shift", "Matched", "Against", "r", "n"]) {
      const th = document.createElement("th");
      th.textContent = h;
      hrow.appendChild(th);
    }
    table.appendChild(hrow);

    for (const e of log) {
      const tr = document.createElement("tr");
      const cell = (text: string, hint?: string): void => {
        const td = document.createElement("td");
        td.textContent = text;
        if (hint) td.title = hint;
        tr.appendChild(td);
      };
      // The timestamp is the database's; only the date and minute are worth the width.
      cell((e.applied_at ?? "").replace("T", " ").slice(0, 16));
      cell(e.set_name);
      // A whole-core shift declared no range. "whole core" is the honest label — it is a
      // statement about what was done, not a missing value.
      cell(e.top === null ? "whole core" : `${e.top.toFixed(1)} – ${(e.base ?? 0).toFixed(1)}`);
      const sign = e.delta > 0 ? "+" : "";
      cell(`${sign}${e.delta.toFixed(2)}`, e.kind === "undo" ? e.note : undefined);
      cell(e.kind === "undo" ? "undo" : e.pairing || "typed by hand");
      cell(e.reference ? `${e.reference} vs ${e.log_curve}` : "—");
      // A blank r is "not measured", never zero: a hand-typed shift was judged by eye, and a
      // 0.00 there would read as a registration that matched nothing.
      cell(e.correlation === null ? "" : e.correlation.toFixed(2));
      cell(e.n_pairs === null ? "" : String(e.n_pairs));
      if (e.kind === "undo") tr.style.opacity = "0.65";
      table.appendChild(tr);
    }
    history.appendChild(table);
  }

  await buildBarrels();
  await buildHistory();
}
