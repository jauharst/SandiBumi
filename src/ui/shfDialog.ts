import { listCurveCatalog, runCuddyFoil, type CuddyFoilResult } from "../ipc";
import { formRow } from "./modal";
import { readTheme } from "./plotCanvas";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/** Saturation-height FUNCTION fitting — Cuddy FOIL / fractal BVW (Wave B item 8, SHF side).
 *  Pools computed PHIE/SW/TVDSS across the scoped wells, fits BVW = Sw·φ = a·H^b above the
 *  free-water level, and optionally scans for the common FWL (Cuddy 1993 Eq 19). Writes no
 *  curves — it produces the field-wide law (a, b) for the forward sw_height apply. */
export async function buildShfContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const catalog = await listCurveCatalog().catch(() => []);
  const names = catalog.map((c) => c.name);
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "mc-dialog";

  const curveSel = (preferred: string[]): HTMLSelectElement => {
    const sel = document.createElement("select");
    for (const n of names) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      sel.appendChild(o);
    }
    const pick = preferred.find((p) => names.includes(p));
    if (pick) sel.value = pick;
    return sel;
  };
  const phieSel = curveSel(["PHIE", "PHIT"]);
  const swSel = curveSel(["SWE", "SWT", "SW"]);
  const tvdSel = curveSel(["TVDSS", "TVD"]);
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
  const fwlInput = num(0);
  const minPhiInput = num(0.0, "0.01");
  content.appendChild(formRow("Free-water level (TVDSS)", fwlInput, "The common FWL; or the scan centre when scanning."));
  content.appendChild(formRow("Min φ (net cutoff)", minPhiInput, "Exclude non-net porosity below this (Cuddy net-reservoir rule)."));

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
  const mkField = (label: string, el: HTMLElement): HTMLElement => {
    const f = document.createElement("label");
    f.className = "mc-field";
    const s = document.createElement("span");
    s.textContent = label;
    f.append(s, el);
    return f;
  };
  scanWrap.append(scanLabel, mkField("FWL lo", scanLo), mkField("FWL hi", scanHi), mkField("step", scanStep));
  content.appendChild(formRow("FWL scan", scanWrap, "When on, steps a common FWL over [lo, hi] and picks the tightest FOIL fit."));

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

  const results = document.createElement("div");
  results.className = "mc-results";
  content.appendChild(results);

  const hint = document.createElement("div");
  hint.className = "mc-chain-note";
  hint.textContent =
    "BVW = Sw·φ = a·H^b (Cuddy 1993/2017). Log-driven — needs computed PHIE, SW and a TVDSS curve. Export (a, b) into the forward sw_height law.";
  content.appendChild(hint);

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    runBtn.disabled = true;
    statusLine.textContent = "Fitting…";
    const t0 = performance.now();
    try {
      const res = await runCuddyFoil({
        well_ids: wellIds,
        phie_curve: phieSel.value,
        sw_curve: swSel.value,
        tvdss_curve: tvdSel.value,
        fwl: parseFloat(fwlInput.value) || 0,
        min_phi: parseFloat(minPhiInput.value) || 0,
        scan: scanCb.checked,
        scan_lo: parseFloat(scanLo.value) || 0,
        scan_hi: parseFloat(scanHi.value) || 0,
        scan_step: parseFloat(scanStep.value) || 0.5,
      });
      const ms = Math.round(performance.now() - t0);
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        statusLine.textContent =
          `a=${res.a.toExponential(4)}, b=${res.b.toFixed(4)}, R²=${res.r2.toFixed(4)} • ${res.n_points} pts • ${ms} ms` +
          (res.fwl_best != null ? ` • FWL=${res.fwl_best.toFixed(1)}` : "");
        if (res.fwl_best != null) fwlInput.value = res.fwl_best.toFixed(2);
        recordProcess("SHF", `Cuddy FOIL fit: BVW=${res.a.toExponential(3)}·H^${res.b.toFixed(3)} (R²=${res.r2.toFixed(3)})`);
        renderResults(results, res);
      }
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return { el: content, dispose: () => scope.dispose() };
}

function renderResults(host: HTMLElement, res: CuddyFoilResult): void {
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

  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent = "BVW vs height above FWL (log–log) with the fitted FOIL line";
  host.appendChild(cap);
  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";
  host.appendChild(canvas);
  drawFoil(canvas, res);

  if (res.scan.length > 0) {
    const scap = document.createElement("div");
    scap.className = "mc-hist-caption";
    scap.textContent = "FWL scan — fit residual vs candidate free-water level (Cuddy Eq 19)";
    host.appendChild(scap);
    const scanCanvas = document.createElement("canvas");
    scanCanvas.className = "mc-hist";
    host.appendChild(scanCanvas);
    drawScan(scanCanvas, res);
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
  ctx.font = "10px system-ui";
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
  ctx.font = "10px system-ui";
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
