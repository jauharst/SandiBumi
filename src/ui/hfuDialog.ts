import { runHfuCluster, type HfuMethod, type HfuResult } from "../ipc";
import { attachResizeRedraw, canvasFont, faciesColor, readTheme } from "./plotCanvas";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";
import { formRow } from "./modal";

/** Hydraulic Flow Unit clustering (Wave B item 8, increment 2 — data-driven rock typing). Reads
 *  the scoped wells' core φ-k, computes FZI = 0.0314·√(k/φ)/(φ/(1−φ)), and partitions log10(FZI)
 *  into HFUs (Ward exact min-variance, or histogram antimodes). Shows the per-HFU table, the
 *  RQI–φz unit-slope crossplot coloured by HFU, and the log10-FZI histogram with the cluster cuts. */
export async function buildHfuContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const scope = await buildWellScope();

  const content = document.createElement("div");
  content.className = "mc-dialog";
  content.appendChild(scope.el);

  const kInput = document.createElement("input");
  kInput.type = "number";
  kInput.min = "2";
  kInput.max = "12";
  kInput.step = "1";
  kInput.value = "5";
  kInput.style.width = "5rem";

  const methodSel = document.createElement("select");
  for (const [val, label] of [
    ["ward", "Ward (min-variance)"],
    ["histogram", "Histogram (antimodes)"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = label;
    methodSel.appendChild(o);
  }

  content.appendChild(
    formRow("HFUs (K)", kInput, "Target number of hydraulic flow units (2–12). Capped to the distinct FZI levels present."),
  );
  content.appendChild(
    formRow("Method", methodSel, "Ward = exact minimum-variance partition. Histogram = boundaries at the log-FZI histogram valleys."),
  );

  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Cluster HFUs";
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
    "Clusters the core φ-k cloud (routine core analysis) — not log curves — so the HFU boundaries " +
    "come from the data, unlike the fixed GHE bins in the Rock Typing module. HFU 1 = lowest FZI " +
    "(poorest). Each HFU gets the Amaefule perm transform k = 1014.24·FZI_gm²·φ³/(1−φ)²; its R² " +
    "(log-k) shows how tightly that one FZI_gm reproduces the members' measured k. Import core data first.";
  content.appendChild(hint);

  // Cleanup for the current render's canvas resize observers (replaced on each run, run on dispose).
  let detachRender: (() => void) | null = null;

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    const k = Math.round(Number(kInput.value));
    if (!Number.isFinite(k) || k < 2) {
      setStatus("HFUs (K) must be at least 2");
      return;
    }
    const method = methodSel.value as HfuMethod;
    runBtn.disabled = true;
    statusLine.textContent = "Clustering…";
    const t0 = performance.now();
    try {
      const res = await runHfuCluster(wellIds, k, method);
      const ms = Math.round(performance.now() - t0);
      // Tear down the previous render's resize observers before replacing the results DOM.
      detachRender?.();
      detachRender = null;
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        const skipNote = res.skipped > 0 ? `, ${res.skipped} plug(s) skipped` : "";
        statusLine.textContent = `${res.clusters.length} HFU(s) from ${res.n_plugs} plug(s)${skipNote} • ${ms} ms`;
        recordProcess("RockType", `HFU cluster (${res.method}): ${res.clusters.length} unit(s), ${res.n_plugs} plug(s)`);
        detachRender = renderHfu(results, res);
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
      detachRender?.();
      scope.dispose();
    },
  };
}

/** Renders the results and returns a cleanup that detaches the plots' resize observers. */
function renderHfu(host: HTMLElement, res: HfuResult): () => void {
  host.innerHTML = "";
  let selected = -1; // -1 = none highlighted

  if (res.note) {
    const note = document.createElement("div");
    note.className = "mc-chain-note";
    note.style.color = "var(--warn)";
    note.textContent = `Note: ${res.note}`;
    host.appendChild(note);
  }

  const table = document.createElement("table");
  table.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["", "HFU", "n", "FZI min", "FZI max", "FZI gm", "φ mean", "k(φ) R²"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const rqiCanvas = document.createElement("canvas");
  rqiCanvas.className = "mc-hist";
  const histCanvas = document.createElement("canvas");
  histCanvas.className = "mc-hist";

  const bodyRows: HTMLTableRowElement[] = [];
  const redraw = () => {
    bodyRows.forEach((tr, i) => tr.classList.toggle("ml-diag", i === selected));
    drawRqiPhiz(rqiCanvas, res, selected);
    drawFziHist(histCanvas, res, selected);
  };

  res.clusters.forEach((c, i) => {
    const tr = document.createElement("tr");
    tr.style.cursor = "pointer";
    // Colour swatch matching the plots.
    const swatchTd = document.createElement("td");
    const sw = document.createElement("span");
    sw.style.display = "inline-block";
    sw.style.width = "0.8rem";
    sw.style.height = "0.8rem";
    sw.style.borderRadius = "2px";
    sw.style.background = faciesColor(c.hfu - 1);
    swatchTd.appendChild(sw);
    tr.appendChild(swatchTd);

    const cells = [
      String(c.hfu),
      String(c.n),
      c.fzi_min.toFixed(3),
      c.fzi_max.toFixed(3),
      c.fzi_gm.toFixed(3),
      c.poro_mean.toFixed(3),
      c.perm_r2.toFixed(3),
    ];
    for (const cell of cells) {
      const td = document.createElement("td");
      td.textContent = cell;
      tr.appendChild(td);
    }
    tr.addEventListener("click", () => {
      selected = selected === i ? -1 : i;
      redraw();
    });
    bodyRows.push(tr);
    table.appendChild(tr);
  });
  host.appendChild(table);

  const cap1 = document.createElement("div");
  cap1.className = "mc-hist-caption";
  cap1.textContent = "RQI vs φz (log-log) coloured by HFU, with each HFU's unit-slope FZI_gm line";
  host.appendChild(cap1);
  host.appendChild(rqiCanvas);

  const cap2 = document.createElement("div");
  cap2.className = "mc-hist-caption";
  cap2.textContent = "log10 FZI histogram with the HFU cut(s); click a table row to highlight one unit";
  host.appendChild(cap2);
  host.appendChild(histCanvas);

  redraw();

  // Repaint the canvases at the new pixel size when the pane is resized (otherwise the fixed
  // backing store stretches and blurs until the next row click). Each reads the live `selected`.
  const detachers = [
    attachResizeRedraw(rqiCanvas, () => drawRqiPhiz(rqiCanvas, res, selected)),
    attachResizeRedraw(histCanvas, () => drawFziHist(histCanvas, res, selected)),
  ];
  return () => detachers.forEach((d) => d());
}

/** Prepares a DPR-scaled 2D context and paints the background; returns null if unavailable. */
function setupCanvas(canvas: HTMLCanvasElement, bg: string): { ctx: CanvasRenderingContext2D; w: number; h: number } | null {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 200;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, w, h);
  return { ctx, w, h };
}

/** RQI (log) vs φz (log) scatter coloured by HFU, plus one unit-slope line per HFU at FZI_gm. */
function drawRqiPhiz(canvas: HTMLCanvasElement, res: HfuResult, selected: number): void {
  const theme = readTheme(canvas);
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;
  const pts = res.points.filter((p) => p.phiz > 0 && p.rqi > 0);
  if (pts.length === 0) return;

  const lxs = pts.map((p) => Math.log10(p.phiz));
  const lys = pts.map((p) => Math.log10(p.rqi));
  // Include the FZI_gm lines' reach so they never clip.
  const xmin = Math.min(...lxs) - 0.15;
  const xmax = Math.max(...lxs) + 0.15;
  const ymin = Math.min(...lys) - 0.15;
  const ymax = Math.max(...lys) + 0.15;
  const padL = 48;
  const padB = 26;
  const X = (lx: number) => padL + ((lx - xmin) / (xmax - xmin || 1)) * (w - padL - 10);
  const Y = (ly: number) => 8 + (1 - (ly - ymin) / (ymax - ymin || 1)) * (h - padB - 8);

  // Axes + decade gridlines.
  ctx.strokeStyle = theme.grid;
  ctx.lineWidth = 1;
  for (let d = Math.ceil(xmin); d <= Math.floor(xmax); d++) {
    ctx.beginPath();
    ctx.moveTo(X(d), 8);
    ctx.lineTo(X(d), h - padB);
    ctx.stroke();
  }
  for (let d = Math.ceil(ymin); d <= Math.floor(ymax); d++) {
    ctx.beginPath();
    ctx.moveTo(padL, Y(d));
    ctx.lineTo(w - 10, Y(d));
    ctx.stroke();
  }
  ctx.strokeStyle = theme.axis;
  ctx.beginPath();
  ctx.moveTo(padL, 8);
  ctx.lineTo(padL, h - padB);
  ctx.lineTo(w - 10, h - padB);
  ctx.stroke();
  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("log10 φz", (padL + w) / 2, h - 4);
  ctx.save();
  ctx.translate(12, (h - padB) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("log10 RQI (µm)", 0, 0);
  ctx.restore();

  // Clip the data region: an FZI_gm line's slope-1 extension can overshoot a cluster's own φz
  // coverage vertically, so without this it would paint over the axis label / frame.
  ctx.save();
  ctx.beginPath();
  ctx.rect(padL, 8, w - 10 - padL, h - padB - 8);
  ctx.clip();

  // Unit-slope FZI_gm lines: RQI = FZI_gm·φz → log RQI = log FZI_gm + log φz.
  for (const c of res.clusters) {
    const dim = selected >= 0 && res.clusters[selected]?.hfu !== c.hfu;
    ctx.strokeStyle = faciesColor(c.hfu - 1);
    ctx.globalAlpha = dim ? 0.25 : 0.9;
    ctx.lineWidth = dim ? 1 : 2;
    const lg = Math.log10(c.fzi_gm);
    ctx.beginPath();
    ctx.moveTo(X(xmin), Y(lg + xmin));
    ctx.lineTo(X(xmax), Y(lg + xmax));
    ctx.stroke();
  }
  ctx.globalAlpha = 1;
  ctx.lineWidth = 1;

  // Scatter.
  for (const p of pts) {
    const dim = selected >= 0 && res.clusters[selected]?.hfu !== p.hfu;
    ctx.fillStyle = faciesColor(p.hfu - 1);
    ctx.globalAlpha = dim ? 0.2 : 1;
    ctx.beginPath();
    ctx.arc(X(Math.log10(p.phiz)), Y(Math.log10(p.rqi)), dim ? 1.6 : 2.4, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.globalAlpha = 1;
  ctx.restore(); // end plot-region clip
}

/** Histogram of log10 FZI, bars coloured by the HFU their centre falls in, with cut lines. */
function drawFziHist(canvas: HTMLCanvasElement, res: HfuResult, selected: number): void {
  const theme = readTheme(canvas);
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;
  const xs = res.points.filter((p) => p.fzi > 0).map((p) => Math.log10(p.fzi));
  if (xs.length === 0) return;

  let xmin = Math.min(...xs);
  let xmax = Math.max(...xs);
  if (!(xmax > xmin)) {
    xmin -= 0.5;
    xmax += 0.5;
  }
  // Same clamp as the backend histogram_boundaries (8..40) so bars and cut lines share resolution.
  const bins = Math.min(40, Math.max(8, Math.round(Math.sqrt(xs.length))));
  const width = (xmax - xmin) / bins;
  const counts = new Array<number>(bins).fill(0);
  for (const x of xs) {
    const b = Math.min(bins - 1, Math.floor((x - xmin) / width));
    counts[b]++;
  }
  const cmax = Math.max(...counts, 1);
  const padL = 40;
  const padB = 26;
  const X = (x: number) => padL + ((x - xmin) / (xmax - xmin || 1)) * (w - padL - 10);
  const Y = (c: number) => 8 + (1 - c / cmax) * (h - padB - 8);
  const lnBoundaries = res.boundaries.map((b) => Math.log10(b));
  // HFU (1-based) whose interval contains x: 1 + count of boundaries at or below x.
  const hfuAt = (x: number) => 1 + lnBoundaries.filter((b) => x >= b).length;

  ctx.strokeStyle = theme.axis;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(padL, 8);
  ctx.lineTo(padL, h - padB);
  ctx.lineTo(w - 10, h - padB);
  ctx.stroke();

  for (let b = 0; b < bins; b++) {
    if (counts[b] === 0) continue;
    const cx = xmin + (b + 0.5) * width;
    const hfu = hfuAt(cx);
    const dim = selected >= 0 && res.clusters[selected]?.hfu !== hfu;
    const x0 = X(xmin + b * width);
    const x1 = X(xmin + (b + 1) * width);
    ctx.fillStyle = faciesColor(hfu - 1);
    ctx.globalAlpha = dim ? 0.25 : 0.9;
    ctx.fillRect(x0 + 0.5, Y(counts[b]), Math.max(1, x1 - x0 - 1), h - padB - Y(counts[b]));
  }
  ctx.globalAlpha = 1;

  // Cut lines.
  ctx.strokeStyle = theme.warn;
  ctx.setLineDash([4, 3]);
  for (const lb of lnBoundaries) {
    if (lb < xmin || lb > xmax) continue;
    ctx.beginPath();
    ctx.moveTo(X(lb), 8);
    ctx.lineTo(X(lb), h - padB);
    ctx.stroke();
  }
  ctx.setLineDash([]);

  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("log10 FZI (µm)", (padL + w) / 2, h - 4);
}
