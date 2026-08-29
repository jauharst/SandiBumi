import { runThomeerFit, type ThomeerResult, type ThomeerSampleFit } from "../ipc";
import { attachResizeRedraw, canvasFont, readTheme } from "./plotCanvas";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";

/** Thomeer Pc hyperbola fitting (Wave B item 8, increment 2 — MICP side). Fits
 *  Bv = Bv∞·exp(−G/log10(Pc/Pd)) per plug over the scoped wells' imported SCAL Pc
 *  points (Bv = φ·(1−Sw)) and shows the per-plug parameter table, the Pc–Bv curve QC
 *  plot for the selected plug, and the Pd–G plane used for Thomeer-class rock typing. */
export async function buildThomeerContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "mc-dialog";
  content.appendChild(scope.el);

  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Fit Thomeer";
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
    "One pore system per plug (multi-modal stacking is a later increment). Needs imported SCAL Pc " +
    "points with plug porosity — Bv = φ·(1−Sw). Pc is standardized to Hg-air equivalent (×367/σcosθ " +
    "from the import's fluid system) so plugs from different lab systems share one Pd–G plane; " +
    "'(raw)' rows lack a recorded σcosθ and fit unconverted. G typ. 0.1–1, lower = better sorted. " +
    "Swanson k constants are literature values — verify before field release.";
  content.appendChild(hint);

  // One handle for the plots currently on screen. Each run replaces the results, so the previous
  // render's resize observers are released first rather than stacking one set per fit.
  let detachPlots: (() => void) | null = null;

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
      const res = await runThomeerFit(scope.backend());
      const ms = Math.round(performance.now() - t0);
      detachPlots?.();
      detachPlots = null;
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        const skipNote = res.skipped > 0 ? ` (${res.skipped} plug(s) skipped — no φ or too few points)` : "";
        statusLine.textContent = `${res.fits.length} plug(s) fitted${skipNote} • ${ms} ms`;
        recordProcess("RockType", `Thomeer fit: ${res.fits.length} plug(s)${skipNote}`);
        detachPlots = renderThomeer(results, res);
      }
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return {
    el: content,
    dispose: () => {
      detachPlots?.();
      scope.dispose();
    },
  };
}

function renderThomeer(host: HTMLElement, res: ThomeerResult): () => void {
  host.innerHTML = "";
  let selected = 0;

  // Per-plug parameter table; a row click selects the plug for the QC plot.
  const table = document.createElement("table");
  table.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["Well", "Sample", "Depth", "φ", "k mD", "System", "Pd psi", "G", "Bv∞", "R²", "n", "Swanson k"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const pcCanvas = document.createElement("canvas");
  pcCanvas.className = "mc-hist";
  const pdgCanvas = document.createElement("canvas");
  // Both axes of the Pd-G plane are data, so it is a crossplot and is marked as one: it stays
  // roughly square instead of filling a wide pane. The Pc curve above it is read left to right
  // against a single axis and is free to fill.
  pdgCanvas.className = "mc-hist mc-hist-plane";

  const bodyRows: HTMLTableRowElement[] = [];
  const redraw = () => {
    bodyRows.forEach((tr, i) => tr.classList.toggle("ml-diag", i === selected));
    drawPcCurve(pcCanvas, res.fits[selected]);
    drawPdG(pdgCanvas, res.fits, selected);
  };
  res.fits.forEach((f, i) => {
    const tr = document.createElement("tr");
    tr.style.cursor = "pointer";
    const cells = [
      f.well_name,
      f.sample_no != null ? String(f.sample_no) : "—",
      f.depth != null ? f.depth.toFixed(1) : "—",
      Number.isFinite(f.poro) ? f.poro.toFixed(3) : "—",
      f.perm != null && Number.isFinite(f.perm) ? f.perm.toFixed(1) : "—",
      // "(raw)" = no σcosθ recorded on the points → Pc NOT standardized to Hg-air.
      (f.system ?? "—") + (f.standardized ? "" : " (raw)"),
      // ⚠ = Pd pinned at a search bound (entry-truncated curve) — an artifact.
      f.pd.toFixed(2) + (f.pd_at_bound ? " ⚠" : ""),
      f.g.toFixed(3),
      f.bv_inf.toFixed(4),
      f.r2.toFixed(3),
      String(f.n),
      f.swanson_k != null && Number.isFinite(f.swanson_k) ? f.swanson_k.toFixed(1) : "—",
    ];
    for (const c of cells) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    if (f.pd_at_bound) {
      tr.title = "Pd pinned at a search bound — the curve is entry-truncated or barely invaded; treat Pd as unresolved.";
    }
    tr.addEventListener("click", () => {
      selected = i;
      redraw();
    });
    bodyRows.push(tr);
    table.appendChild(tr);
  });
  host.appendChild(table);

  const cap1 = document.createElement("div");
  cap1.className = "mc-hist-caption";
  cap1.textContent = "Selected plug: Bv vs Pc (log) with the fitted Thomeer hyperbola";
  host.appendChild(cap1);
  host.appendChild(pcCanvas);

  const cap2 = document.createElement("div");
  cap2.className = "mc-hist-caption";
  cap2.textContent = "Pd–G plane (all plugs; selected highlighted) — the Thomeer rock-typing crossplot";
  host.appendChild(cap2);
  host.appendChild(pdgCanvas);

  redraw();

  // The pane no longer caps at the form-column width, so these canvases really do change size.
  // A canvas whose CSS box grows without a redraw is a scaled bitmap, not a bigger plot.
  return attachAll([
    attachResizeRedraw(pcCanvas, () => drawPcCurve(pcCanvas, res.fits[selected])),
    attachResizeRedraw(pdgCanvas, () => drawPdG(pdgCanvas, res.fits, selected)),
  ]);
}

/** Collapse several detachers into one, so a caller holds a single handle per render. */
function attachAll(detachers: Array<() => void>): () => void {
  return () => {
    for (const d of detachers) d();
  };
}

/** Selected plug: Bv (linear) vs log10 Pc scatter + the fitted hyperbola. */
function drawPcCurve(canvas: HTMLCanvasElement, fit: ThomeerSampleFit | undefined): void {
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
  if (!fit) return;

  const pcs = [...fit.scatter, ...fit.curve].map(([pc]) => pc).filter((pc) => pc > 0);
  if (pcs.length === 0) return;
  const xmin = Math.log10(Math.min(...pcs));
  const xmax = Math.log10(Math.max(...pcs));
  const ymax = Math.max(...fit.scatter.map(([, bv]) => bv), fit.bv_inf) * 1.05 || 1;
  const padL = 44;
  const padB = 22;
  const X = (pc: number) => padL + ((Math.log10(pc) - xmin) / (xmax - xmin || 1)) * (w - padL - 8);
  const Y = (bv: number) => 8 + (1 - bv / ymax) * (h - padB - 8);

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
  ctx.fillText("log10 Pc (psi)", (padL + w) / 2, h - 4);
  ctx.save();
  ctx.translate(11, (h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("Bv (v/v bulk)", 0, 0);
  ctx.restore();

  // Entry pressure marker.
  ctx.strokeStyle = theme.grid;
  ctx.setLineDash([4, 3]);
  ctx.beginPath();
  ctx.moveTo(X(fit.pd), 8);
  ctx.lineTo(X(fit.pd), h - padB);
  ctx.stroke();
  ctx.setLineDash([]);

  // Scatter.
  ctx.fillStyle = theme.accent2;
  for (const [pc, bv] of fit.scatter) {
    if (!(pc > 0)) continue;
    ctx.beginPath();
    ctx.arc(X(pc), Y(bv), 1.8, 0, Math.PI * 2);
    ctx.fill();
  }

  // Fitted hyperbola.
  ctx.strokeStyle = theme.accent;
  ctx.lineWidth = 2;
  ctx.beginPath();
  let pen = false;
  for (const [pc, bv] of fit.curve) {
    if (!(pc > 0)) continue;
    const x = X(pc);
    const y = Y(bv);
    if (pen) ctx.lineTo(x, y);
    else ctx.moveTo(x, y);
    pen = true;
  }
  ctx.stroke();
}

/** All plugs on the (log10 Pd, G) plane; the selected plug ringed. */
function drawPdG(canvas: HTMLCanvasElement, fits: ThomeerSampleFit[], selected: number): void {
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
  if (fits.length === 0) return;

  const lx = fits.map((f) => Math.log10(f.pd));
  const gy = fits.map((f) => f.g);
  const xmin = Math.min(...lx) - 0.2;
  const xmax = Math.max(...lx) + 0.2;
  const ymin = 0;
  const ymax = Math.max(...gy, 1) * 1.1;
  const padL = 40;
  const padB = 22;
  const X = (v: number) => padL + ((v - xmin) / (xmax - xmin || 1)) * (w - padL - 8);
  const Y = (v: number) => 8 + (1 - (v - ymin) / (ymax - ymin || 1)) * (h - padB - 8);

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
  ctx.fillText("log10 Pd (psi)", (padL + w) / 2, h - 4);
  ctx.save();
  ctx.translate(11, (h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("G", 0, 0);
  ctx.restore();

  fits.forEach((f, i) => {
    const x = X(Math.log10(f.pd));
    const y = Y(f.g);
    ctx.fillStyle = i === selected ? theme.accent : theme.accent2;
    ctx.beginPath();
    ctx.arc(x, y, i === selected ? 4 : 2.4, 0, Math.PI * 2);
    ctx.fill();
    if (i === selected) {
      ctx.strokeStyle = theme.accent;
      ctx.beginPath();
      ctx.arc(x, y, 7, 0, Math.PI * 2);
      ctx.stroke();
    }
  });
}
