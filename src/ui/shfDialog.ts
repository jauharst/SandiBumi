import {
  listCurveCatalog,
  runCuddyFoil,
  runShfFit,
  type CuddyFoilResult,
  type ShfFitResult,
  type ShfGroupFit,
} from "../ipc";
import { formRow } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";
import { preferredCurveSelect } from "./plotCommon";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/** Saturation-height FUNCTION fitting (playbook #4). Pools computed PHIE/SW/TVDSS (+ PERM / RT)
 *  across the scoped wells and fits a chosen SHF family above the free-water level: Cuddy FOIL
 *  (BVW = a·H^b, + FWL scan), Brooks-Corey, Skelt-Harrison, Thomeer, or a log-driven Leverett-J —
 *  optionally one law per rock type. Writes no curves — it produces the law(s) for the forward
 *  sw_height apply. Method math + Tier-A seeds banked in docs/ref_shf.md. */

/** Callbacks the result renderers use for the draggable free-water level. */
interface FwlCtl {
  status: (text: string) => void;
  getFwl: () => number;
  setFwl: (fwl: number) => void;
}

/** "—" for null/NaN (failed group fits arrive as JSON null over IPC). */
const fmt = (v: number | null | undefined, digits = 4): string =>
  v == null || !Number.isFinite(v) ? "—" : v.toFixed(digits);
const fmtExp = (v: number | null | undefined, digits = 3): string =>
  v == null || !Number.isFinite(v) ? "—" : v.toExponential(digits);

export async function buildShfContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const catalog = await listCurveCatalog().catch(() => []);
  const names = catalog.map((c) => c.name);
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "mc-dialog";

  const methodSel = document.createElement("select");
  for (const [val, lbl] of [
    ["foil", "Cuddy FOIL (BVW = a·H^b)"],
    ["brooks_corey", "Brooks-Corey"],
    ["skelt", "Skelt-Harrison"],
    ["thomeer", "Thomeer (log-hyperbola)"],
    ["leverett_j", "Leverett-J (Sw = A·J^B from PERM)"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = lbl;
    methodSel.appendChild(o);
  }
  content.appendChild(
    formRow(
      "SHF form",
      methodSel,
      "FOIL fits bulk-volume water (log-log); Brooks-Corey / Skelt / Thomeer fit Sw vs height; Leverett-J fits Sw vs J from PERM/PHIE.",
    ),
  );

  const phieSel = preferredCurveSelect(names, ["PHIE", "PHIT"]);
  const swSel = preferredCurveSelect(names, ["SWE", "SWT", "SW"]);
  const tvdSel = preferredCurveSelect(names, ["TVDSS", "TVD"]);
  content.appendChild(formRow("Porosity (φ)", phieSel));
  content.appendChild(formRow("Water saturation (Sw)", swSel));
  content.appendChild(formRow("TVDSS", tvdSel, "True vertical depth subsea; height above FWL = FWL − TVDSS."));
  content.appendChild(scope.el);

  const num = (value: number, step = "any"): HTMLInputElement => {
    const i = document.createElement("input");
    i.type = "number";
    i.step = step;
    i.value = String(value);
    return i;
  };
  const mkField = (label: string, el: HTMLElement): HTMLElement => {
    const f = document.createElement("label");
    f.className = "mc-field";
    const s = document.createElement("span");
    s.textContent = label;
    f.append(s, el);
    return f;
  };

  const fwlInput = num(0);
  const minPhiInput = num(0.0, "0.01");
  content.appendChild(formRow("Free-water level (TVDSS)", fwlInput, "The common FWL; or the scan centre when scanning. Drag on the result plot to nudge it."));
  content.appendChild(formRow("Min φ (net cutoff)", minPhiInput, "Exclude non-net porosity below this (Cuddy net-reservoir rule)."));

  // Leverett-J inputs: PERM + fluid props. Seeds are Tier A (per docs/ref_shf.md): σ·cosθ 26
  // dyn/cm = IP cap-pressure Res(Water-Oil) 30·cos30°, Water-Gas 50·cos0°; ρhc 0.7 g/cc from
  // the Techlog sand-summary default — seeds, not field truth; all overridable per run.
  const permSel = preferredCurveSelect(names, ["PERM", "KINT", "K", "KTIM"]);
  const sysSel = document.createElement("select");
  for (const [val, lbl] of [
    ["oil", "Water-Oil (σcosθ 26)"],
    ["gas", "Water-Gas (σcosθ 50)"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = lbl;
    sysSel.appendChild(o);
  }
  const rhoWInput = num(1.0, "0.01");
  const rhoHcInput = num(0.7, "0.01");
  const iftInput = num(26, "1");
  sysSel.addEventListener("change", () => {
    iftInput.value = sysSel.value === "gas" ? "50" : "26";
    rhoHcInput.value = sysSel.value === "gas" ? "0.2" : "0.7";
  });
  const levWrap = document.createElement("div");
  levWrap.className = "mc-settings";
  levWrap.append(
    mkField("System", sysSel),
    mkField("ρw g/cc", rhoWInput),
    mkField("ρhc g/cc", rhoHcInput),
    mkField("σ·cosθ", iftInput),
  );
  const levRow = formRow("Leverett-J", levWrap, "J = 0.21645·Pc/σcosθ·√(k/φ); Pc from height (0.433·Δρ·h_ft). Defaults seeded from the IP/Techlog tables (Tier A).");
  const permRow = formRow("Permeability (k)", permSel, "Working permeability for J (Leverett-J only).");
  content.appendChild(permRow);
  content.appendChild(levRow);

  // Per-rock-type fitting: any family + an RT/facies curve → one law per RT class.
  const rtCb = document.createElement("input");
  rtCb.type = "checkbox";
  const rtSel = preferredCurveSelect(names, ["RT", "RT_GHE", "GHE", "FACIES", "RT35"]);
  const rtWrap = document.createElement("div");
  rtWrap.className = "mc-settings";
  const rtLabel = document.createElement("label");
  rtLabel.className = "mc-field";
  rtLabel.append(rtCb, document.createTextNode(" Fit per rock type"));
  rtWrap.append(rtLabel, mkField("RT curve", rtSel));
  content.appendChild(formRow("Rock typing", rtWrap, "Fits one law per rounded RT class alongside the pooled law — classes that cannot fit are reported, not dropped."));

  const scanCb = document.createElement("input");
  scanCb.type = "checkbox";
  const scanLo = num(0);
  const scanHi = num(0);
  const scanStep = num(0.5, "0.1");
  const scanWrap = document.createElement("div");
  scanWrap.className = "mc-settings";
  const scanLabel = document.createElement("label");
  scanLabel.className = "mc-field";
  scanLabel.append(scanCb, document.createTextNode(" Scan for FWL (Cuddy Eq 19)"));
  scanWrap.append(scanLabel, mkField("FWL lo", scanLo), mkField("FWL hi", scanHi), mkField("step", scanStep));
  const scanRow = formRow("FWL scan", scanWrap, "When on, steps a common FWL over [lo, hi] and picks the tightest FOIL fit. Click the scan plot to pick a FWL by hand.");
  content.appendChild(scanRow);

  // Leverett inputs only for leverett_j; the FWL scan only for FOIL.
  const syncRows = (): void => {
    const m = methodSel.value;
    scanRow.style.display = m === "foil" ? "" : "none";
    permRow.style.display = m === "leverett_j" ? "" : "none";
    levRow.style.display = m === "leverett_j" ? "" : "none";
    runBtn.textContent = m === "foil" ? "Fit FOIL" : "Fit SHF";
  };
  methodSel.addEventListener("change", syncRows);

  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Fit FOIL";
  runBtn.classList.add("primary");
  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  runRow.append(runBtn, statusLine);
  content.appendChild(runRow);
  syncRows();

  const results = document.createElement("div");
  results.className = "mc-results";
  content.appendChild(results);

  const hint = document.createElement("div");
  hint.className = "mc-chain-note";
  hint.textContent =
    "Five families: FOIL (Cuddy 1993/2017), Brooks-Corey 1964, Skelt-Harrison 1995, Thomeer 1960, Leverett 1941. Log-driven — needs computed PHIE, SW and a TVDSS curve (+ PERM for Leverett-J). Export into the forward sw_height law.";
  content.appendChild(hint);

  const fwlCtl: FwlCtl = {
    status: (t) => {
      statusLine.textContent = t;
    },
    getFwl: () => parseFloat(fwlInput.value) || 0,
    setFwl: (f) => {
      fwlInput.value = f.toFixed(2);
      void doRun();
    },
  };

  const doRun = async (): Promise<void> => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    runBtn.disabled = true;
    statusLine.textContent = "Fitting…";
    const t0 = performance.now();
    const method = methodSel.value;
    const rtCurve = rtCb.checked ? rtSel.value : undefined;
    const common = {
      well_ids: wellIds,
      phie_curve: phieSel.value,
      sw_curve: swSel.value,
      tvdss_curve: tvdSel.value,
      fwl: parseFloat(fwlInput.value) || 0,
      min_phi: parseFloat(minPhiInput.value) || 0,
      rt_curve: rtCurve,
    };
    try {
      if (method === "foil") {
        const res = await runCuddyFoil({
          ...common,
          scan: scanCb.checked,
          scan_lo: parseFloat(scanLo.value) || 0,
          scan_hi: parseFloat(scanHi.value) || 0,
          scan_step: parseFloat(scanStep.value) || 0.5,
        });
        const ms = Math.round(performance.now() - t0);
        if (res.error) {
          statusLine.textContent = `Failed: ${res.error}`;
          results.innerHTML = "";
          renderDiagnostics(results, res.excluded, res.notes);
        } else {
          statusLine.textContent =
            `a=${res.a.toExponential(4)}, b=${res.b.toFixed(4)}, R²=${res.r2.toFixed(4)} • ${res.n_points} pts • ${ms} ms` +
            (res.fwl_best != null ? ` • FWL=${res.fwl_best.toFixed(1)}` : "");
          if (res.fwl_best != null) fwlInput.value = res.fwl_best.toFixed(2);
          recordProcess("SHF", `Cuddy FOIL fit: BVW=${res.a.toExponential(3)}·H^${res.b.toFixed(3)} (R²=${res.r2.toFixed(3)})`);
          renderResults(results, res, fwlCtl);
        }
      } else {
        const res = await runShfFit({
          ...common,
          method: method as "brooks_corey" | "skelt" | "thomeer" | "leverett_j",
          perm_curve: method === "leverett_j" ? permSel.value : undefined,
          rho_w: parseFloat(rhoWInput.value) || 1.0,
          rho_hc: parseFloat(rhoHcInput.value) || 0.7,
          ift_res: parseFloat(iftInput.value) || 26,
        });
        const ms = Math.round(performance.now() - t0);
        if (res.error) {
          statusLine.textContent = `Failed: ${res.error}`;
          results.innerHTML = "";
          renderDiagnostics(results, res.excluded, res.notes);
        } else {
          const ps = res.params.map(([k, v]) => `${k}=${fmt(v, 3)}`).join(", ");
          statusLine.textContent = `${res.method}: ${ps} • R²=${res.r2.toFixed(4)} • ${res.n_points} pts • ${ms} ms`;
          recordProcess("SHF", `${res.method} fit: ${ps} (R²=${res.r2.toFixed(3)})`);
          renderShfResults(results, res, fwlCtl);
        }
      }
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  };
  runBtn.addEventListener("click", () => {
    void doRun();
  });

  return { el: content, dispose: () => scope.dispose() };
}

/** Excluded-sample breakdown + honesty notes, shown under every result (and on failures). */
function renderDiagnostics(host: HTMLElement, excluded: [string, number][], notes: string[]): void {
  if ((excluded?.length ?? 0) === 0 && (notes?.length ?? 0) === 0) return;
  const diag = document.createElement("div");
  diag.className = "shf-diag";
  if (excluded?.length) {
    const line = document.createElement("div");
    line.textContent = "Excluded: " + excluded.map(([r, n]) => `${r}: ${n}`).join(" · ");
    diag.appendChild(line);
  }
  for (const n of notes ?? []) {
    const line = document.createElement("div");
    line.className = "warn";
    line.textContent = "⚠ " + n;
    diag.appendChild(line);
  }
  host.appendChild(diag);
}

/** Horizontal drag on a result plot nudges the FWL (0.2 m per px), re-fitting on release. */
function attachFwlDrag(canvas: HTMLCanvasElement, currentFwl: () => number, ctl: FwlCtl): void {
  let startX: number | null = null;
  let startFwl = 0;
  canvas.style.cursor = "ew-resize";
  canvas.title = "Drag horizontally to nudge the FWL (0.2 m per pixel); release to re-fit.";
  canvas.addEventListener("pointerdown", (e) => {
    startX = e.clientX;
    startFwl = currentFwl();
    try {
      canvas.setPointerCapture(e.pointerId);
    } catch {
      /* synthetic events carry no active pointer — drag still works via bubbling */
    }
  });
  canvas.addEventListener("pointermove", (e) => {
    if (startX == null) return;
    ctl.status(`FWL → ${(startFwl + (e.clientX - startX) * 0.2).toFixed(2)} (release to re-fit)`);
  });
  canvas.addEventListener("pointerup", (e) => {
    if (startX == null) return;
    const fwl = startFwl + (e.clientX - startX) * 0.2;
    const moved = Math.abs(e.clientX - startX) > 2;
    startX = null;
    try {
      canvas.releasePointerCapture(e.pointerId);
    } catch {
      /* see above */
    }
    if (moved) ctl.setFwl(fwl);
  });
}

function renderResults(host: HTMLElement, res: CuddyFoilResult, ctl: FwlCtl): void {
  host.innerHTML = "";

  const summary = document.createElement("table");
  summary.className = "mc-table";
  const rows: [string, string][] = [
    ["a (BVW at H=1)", res.a.toExponential(5)],
    ["b (slope)", res.b.toFixed(5)],
    ["R² (log space)", res.r2.toFixed(5)],
    ["points fitted", String(res.n_points)],
    ["FWL used (TVDSS)", res.fwl_used.toFixed(2)],
  ];
  if (res.fwl_best != null) rows.push(["FWL (scan best)", res.fwl_best.toFixed(2)]);
  for (const [k, v] of rows) {
    const tr = document.createElement("tr");
    const th = document.createElement("th");
    th.textContent = k;
    const td = document.createElement("td");
    td.textContent = v;
    tr.append(th, td);
    summary.appendChild(tr);
  }
  host.appendChild(summary);

  // Per-rock-type FOIL laws (null-guarded: failed classes carry null numerics + the reason).
  if (res.groups.length > 0) {
    const gcap = document.createElement("div");
    gcap.className = "mc-hist-caption";
    gcap.textContent = "Per-rock-type FOIL laws";
    host.appendChild(gcap);
    const gt = document.createElement("table");
    gt.className = "mc-table";
    gt.innerHTML = "<tr><th>RT</th><th>a</th><th>b</th><th>R²</th><th>n</th><th></th></tr>";
    for (const gp of res.groups) {
      const tr = document.createElement("tr");
      for (const cell of [String(gp.rt), fmtExp(gp.a), fmt(gp.b), fmt(gp.r2), String(gp.n_points), gp.error ?? ""]) {
        const td = document.createElement("td");
        td.textContent = cell;
        tr.appendChild(td);
      }
      gt.appendChild(tr);
    }
    host.appendChild(gt);
  }

  renderDiagnostics(host, res.excluded, res.notes);

  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = "BVW vs height above FWL (log–log) with the fitted FOIL line — drag to nudge the FWL";
  host.appendChild(cap);
  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";
  host.appendChild(canvas);
  drawFoil(canvas, res);
  attachFwlDrag(canvas, () => res.fwl_used, ctl);

  if (res.scan.length > 0) {
    const scap = document.createElement("div");
    scap.className = "mc-hist-caption";
    scap.textContent = "FWL scan — fit residual vs candidate free-water level (Cuddy Eq 19) — click to pick";
    host.appendChild(scap);
    const scanCanvas = document.createElement("canvas");
    scanCanvas.className = "mc-hist";
    host.appendChild(scanCanvas);
    drawScan(scanCanvas, res);
    // Click-to-pick a candidate FWL straight off the scan curve.
    scanCanvas.style.cursor = "crosshair";
    scanCanvas.addEventListener("click", (e) => {
      const rect = scanCanvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const w = scanCanvas.clientWidth || 360;
      const padL = 48;
      const xs = res.scan.map((p) => p.fwl);
      const xmin = Math.min(...xs);
      const xmax = Math.max(...xs);
      const fwl = xmin + ((x - padL) / (w - padL - 8)) * (xmax - xmin);
      if (Number.isFinite(fwl)) ctl.setFwl(Math.min(xmax, Math.max(xmin, fwl)));
    });
  }
}

/** Log–log scatter of (H, BVW) with the fitted power-law line. */
function drawFoil(canvas: HTMLCanvasElement, res: CuddyFoilResult): void {
  const theme = readTheme(canvas);
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 200;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);

  const pts = res.points.filter((p) => p.h > 0 && p.bvw > 0);
  if (pts.length === 0) return;
  const lx = pts.map((p) => Math.log10(p.h));
  const ly = pts.map((p) => Math.log10(p.bvw));
  const xmin = Math.min(...lx);
  const xmax = Math.max(...lx);
  const ymin = Math.min(...ly);
  const ymax = Math.max(...ly);
  const padL = 44;
  const padB = 22;
  const X = (v: number) => padL + ((v - xmin) / (xmax - xmin || 1)) * (w - padL - 8);
  const Y = (v: number) => 8 + (1 - (v - ymin) / (ymax - ymin || 1)) * (h - padB - 8);

  // Axes.
  ctx.strokeStyle = theme.axis;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(padL, 8);
  ctx.lineTo(padL, h - padB);
  ctx.lineTo(w - 8, h - padB);
  ctx.stroke();
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("log10 H (height above FWL)", (padL + w) / 2, h - 4);
  ctx.save();
  ctx.translate(11, (h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("log10 BVW", 0, 0);
  ctx.restore();

  // Scatter.
  ctx.fillStyle = theme.accent2;
  for (let i = 0; i < pts.length; i++) {
    ctx.beginPath();
    ctx.arc(X(lx[i]), Y(ly[i]), 1.6, 0, Math.PI * 2);
    ctx.fill();
  }

  // Fitted line: log10 BVW = log10 a + b·log10 H.
  const loga = Math.log10(res.a);
  ctx.strokeStyle = theme.accent;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(X(xmin), Y(loga + res.b * xmin));
  ctx.lineTo(X(xmax), Y(loga + res.b * xmax));
  ctx.stroke();
}

/** FWL scan residual curve with the minimum marked. */
function drawScan(canvas: HTMLCanvasElement, res: CuddyFoilResult): void {
  const theme = readTheme(canvas);
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 200;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);

  const s = res.scan;
  if (s.length === 0) return;
  const xs = s.map((p) => p.fwl);
  const ys = s.map((p) => p.residual);
  const xmin = Math.min(...xs);
  const xmax = Math.max(...xs);
  const ymin = Math.min(...ys);
  const ymax = Math.max(...ys);
  const padL = 48;
  const padB = 22;
  const X = (v: number) => padL + ((v - xmin) / (xmax - xmin || 1)) * (w - padL - 8);
  const Y = (v: number) => 8 + (1 - (v - ymin) / (ymax - ymin || 1)) * (h - padB - 8);

  ctx.strokeStyle = theme.axis;
  ctx.beginPath();
  ctx.moveTo(padL, 8);
  ctx.lineTo(padL, h - padB);
  ctx.lineTo(w - 8, h - padB);
  ctx.stroke();
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("candidate FWL (TVDSS)", (padL + w) / 2, h - 4);

  ctx.strokeStyle = theme.accent2;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  s.forEach((p, i) => {
    const x = X(p.fwl);
    const y = Y(p.residual);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();

  if (res.fwl_best != null) {
    ctx.strokeStyle = theme.accent;
    ctx.setLineDash([4, 3]);
    ctx.beginPath();
    ctx.moveTo(X(res.fwl_best), 8);
    ctx.lineTo(X(res.fwl_best), h - padB);
    ctx.stroke();
    ctx.setLineDash([]);
  }
}

/** Height-domain fit results: param table, per-RT tabs, diagnostics, and the Sw-vs-height
 *  crossplot with the selected law's curve. */
function renderShfResults(host: HTMLElement, res: ShfFitResult, ctl: FwlCtl): void {
  host.innerHTML = "";

  const paramTable = (params: [string, number | null][], r2: number | null, n: number): HTMLTableElement => {
    const t = document.createElement("table");
    t.className = "mc-table";
    const rows: [string, string][] = params.map(([k, v]) => [k, fmt(v, 5)] as [string, string]);
    rows.push(["R²", fmt(r2, 5)]);
    rows.push(["points fitted", String(n)]);
    for (const [k, v] of rows) {
      const tr = document.createElement("tr");
      const th = document.createElement("th");
      th.textContent = k;
      const td = document.createElement("td");
      td.textContent = v;
      tr.append(th, td);
      t.appendChild(tr);
    }
    return t;
  };

  const detail = document.createElement("div");

  // Tab strip: the pooled law + one tab per rock-type class (when an RT curve was supplied).
  let tabs: HTMLElement | null = null;
  if (res.groups.length > 0) {
    tabs = document.createElement("div");
    tabs.className = "shf-tabs";
    host.appendChild(tabs);
  }
  host.appendChild(detail);

  const showTab = (gp: ShfGroupFit | null): void => {
    detail.innerHTML = "";
    if (gp && gp.error) {
      detail.appendChild(paramTable([], null, gp.n_points));
      const err = document.createElement("div");
      err.className = "shf-diag";
      const line = document.createElement("div");
      line.className = "warn";
      line.textContent = `⚠ RT ${gp.rt}: ${gp.error}`;
      err.appendChild(line);
      detail.appendChild(err);
      return;
    }
    const params = gp ? gp.params : res.params;
    const r2 = gp ? gp.r2 : res.r2;
    const n = gp ? gp.n_points : res.n_points;
    detail.appendChild(paramTable(params, r2, n));

    const cap = document.createElement("div");
    cap.className = "mc-hist-caption";
    cap.textContent =
      (gp ? `RT ${gp.rt}: ` : "") +
      `Sw vs height above FWL (log H) with the fitted ${res.method} curve — drag to nudge the FWL`;
    detail.appendChild(cap);
    const canvas = document.createElement("canvas");
    canvas.className = "mc-hist";
    detail.appendChild(canvas);
    const pts = gp ? res.points.filter((p) => p.rt === gp.rt) : res.points;
    drawShfFit(canvas, pts, gp ? gp.curve : res.curve);
    attachFwlDrag(canvas, ctl.getFwl, ctl);
  };

  if (tabs) {
    const mkTab = (label: string, gp: ShfGroupFit | null): HTMLButtonElement => {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "shf-tab";
      b.textContent = label;
      b.addEventListener("click", () => {
        tabs.querySelectorAll(".shf-tab").forEach((el) => el.classList.remove("active"));
        b.classList.add("active");
        showTab(gp);
      });
      tabs.appendChild(b);
      return b;
    };
    const allTab = mkTab("All (pooled)", null);
    for (const gp of res.groups) mkTab(gp.error ? `RT ${gp.rt} ⚠` : `RT ${gp.rt}`, gp);
    allTab.classList.add("active");
  }

  showTab(null);
  renderDiagnostics(host, res.excluded, res.notes);
}

/** Sw (linear, 0–1) vs height (log10) scatter + the fitted Sw(H) curve. */
function drawShfFit(
  canvas: HTMLCanvasElement,
  points: { h: number; sw: number }[],
  curve: [number, number][],
): void {
  const theme = readTheme(canvas);
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 200;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);

  const pts = points.filter((p) => p.h > 0 && p.sw > 0);
  const curvePos = curve.filter(([hh]) => hh > 0);
  if (pts.length === 0 && curvePos.length === 0) return;
  const lx = pts.map((p) => Math.log10(p.h));
  const lc = curvePos.map(([hh]) => Math.log10(hh));
  const xmin = Math.min(...lx, ...lc);
  const xmax = Math.max(...lx, ...lc);
  const padL = 40;
  const padB = 22;
  const X = (v: number) => padL + ((v - xmin) / (xmax - xmin || 1)) * (w - padL - 8);
  const Y = (sw: number) => 8 + (1 - Math.min(1, Math.max(0, sw))) * (h - padB - 8); // Sw 0..1

  // Axes.
  ctx.strokeStyle = theme.axis;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(padL, 8);
  ctx.lineTo(padL, h - padB);
  ctx.lineTo(w - 8, h - padB);
  ctx.stroke();
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("log10 H (height above FWL)", (padL + w) / 2, h - 4);
  ctx.save();
  ctx.translate(11, (h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("Sw", 0, 0);
  ctx.restore();
  // Sw gridlines at 0 / 0.5 / 1.
  ctx.strokeStyle = theme.grid;
  ctx.globalAlpha = 0.4;
  for (const sw of [0, 0.5, 1]) {
    ctx.beginPath();
    ctx.moveTo(padL, Y(sw));
    ctx.lineTo(w - 8, Y(sw));
    ctx.stroke();
  }
  ctx.globalAlpha = 1;

  // Scatter.
  ctx.fillStyle = theme.accent2;
  for (let i = 0; i < pts.length; i++) {
    ctx.beginPath();
    ctx.arc(X(lx[i]), Y(pts[i].sw), 1.6, 0, Math.PI * 2);
    ctx.fill();
  }

  // Fitted curve.
  ctx.strokeStyle = theme.accent;
  ctx.lineWidth = 2;
  ctx.beginPath();
  let pen = false;
  for (const [hh, sw] of curvePos) {
    const x = X(Math.log10(hh));
    const y = Y(sw);
    if (pen) ctx.lineTo(x, y);
    else ctx.moveTo(x, y);
    pen = true;
  }
  ctx.stroke();
}
