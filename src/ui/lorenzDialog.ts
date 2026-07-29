import { listCurveCatalog, listWells, runLorenz, type LorenzResult } from "../ipc";
import { appState, filterByActiveGroup } from "../state";
import { formRow } from "./modal";
import { attachResizeRedraw, canvasFont, faciesColor, readTheme } from "./plotCanvas";
import { preferredCurveSelect } from "./plotCommon";
import { recordProcess } from "../processLog";

/** Stratigraphic Modified Lorenz Plot (playbook #3, increment 3c — the visual for the 3a solver).
 *  One well's φ + k logs walked in depth order: cumulative flow capacity Σ(k·h) vs storage
 *  capacity Σ(φ·h), segmented into flow units (Gunter 1997 SMLP). Shows the SMLP curve coloured
 *  by flow unit against the 45° homogeneous diagonal, the per-unit table (speed zone / baffle),
 *  and the Lorenz heterogeneity coefficient. Reads curves through the standard→computed→generic
 *  resolver, so PERM can be an imported KLOGH, a computed PERM, or the rock-typing PERM_RT. */
export async function buildLorenzContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const [catalog, wells] = await Promise.all([
    listCurveCatalog().catch(() => []),
    listWells().catch(() => []),
  ]);
  const names = catalog.map((c) => c.name);
  const scoped = filterByActiveGroup(wells);

  const content = document.createElement("div");
  content.className = "mc-dialog";

  const wellSel = document.createElement("select");
  for (const w of scoped) {
    const o = document.createElement("option");
    o.value = w.well_id;
    o.textContent = w.well_name;
    wellSel.appendChild(o);
  }
  const selected = appState.selectedWell.get();
  if (selected && scoped.some((w) => w.well_id === selected.well_id)) {
    wellSel.value = selected.well_id;
  }
  content.appendChild(formRow("Well", wellSel, "The SMLP is a single-well stratigraphic walk — one column at a time."));

  const phiSel = preferredCurveSelect(names, ["PHIE", "MM_PHIE", "PHIT", "MM_PHIT"]);
  const permSel = preferredCurveSelect(names, ["PERM", "KLOGH", "PERM_RT", "KH", "K"]);
  content.appendChild(formRow("Porosity (φ)", phiSel));
  content.appendChild(
    formRow("Permeability (k)", permSel, "mD — an imported KLOGH, a computed PERM, or the rock-typing PERM_RT estimate."),
  );

  const num = (value: string, step = "any", width = "6.5rem"): HTMLInputElement => {
    const i = document.createElement("input");
    i.type = "number";
    i.step = step;
    i.value = value;
    i.style.width = width;
    return i;
  };
  const kInput = num("0", "1", "5rem");
  kInput.min = "0";
  kInput.max = "12";
  content.appendChild(
    formRow("Flow units (K)", kInput, "0 = auto (split while a new boundary explains ≥2 % of the slope variance, max 12); 1–12 forces exactly K."),
  );
  const fromInput = num("");
  const toInput = num("");
  const windowWrap = document.createElement("div");
  windowWrap.className = "mc-settings";
  const mkField = (label: string, el: HTMLElement): HTMLElement => {
    const f = document.createElement("label");
    f.className = "mc-field";
    const s = document.createElement("span");
    s.textContent = label;
    f.append(s, el);
    return f;
  };
  windowWrap.append(mkField("Depth from", fromInput), mkField("to", toInput));
  content.appendChild(formRow("Window (MD)", windowWrap, "Optional — leave blank for the whole well; use a zone's top/base to Lorenz one zone."));

  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.textContent = "Build Lorenz Plot";
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
    "Walks the column top-down accumulating flow capacity Σ(k·h) against storage capacity Σ(φ·h) " +
    "(Gunter 1997 SMLP). Steep segments (slope > 1) are speed zones — they deliver more flow than " +
    "their share of storage; flat segments (< 1) are baffles. The dashed 45° diagonal is a perfectly " +
    "homogeneous column; the Lorenz coefficient is 0 on it and → 1 as flow concentrates in thin intervals.";
  content.appendChild(hint);

  let detachRender: (() => void) | null = null;

  runBtn.addEventListener("click", async () => {
    const wellId = wellSel.value;
    if (!wellId) {
      setStatus("No well selected — import a well first");
      return;
    }
    const k = Math.round(Number(kInput.value));
    if (!Number.isFinite(k) || k < 0 || k > 12) {
      setStatus("Flow units (K) must be 0 (auto) or 1–12");
      return;
    }
    const from = fromInput.value.trim() === "" ? undefined : Number(fromInput.value);
    const to = toInput.value.trim() === "" ? undefined : Number(toInput.value);
    runBtn.disabled = true;
    statusLine.textContent = "Computing…";
    const t0 = performance.now();
    try {
      const res = await runLorenz(wellId, phiSel.value, permSel.value, k, from, to);
      const ms = Math.round(performance.now() - t0);
      detachRender?.();
      detachRender = null;
      if (res.error) {
        statusLine.textContent = `Failed: ${res.error}`;
        results.innerHTML = "";
      } else {
        const skipNote = res.skipped > 0 ? `, ${res.skipped} skipped` : "";
        statusLine.textContent = `${res.units.length} flow unit(s) from ${res.n_samples} sample(s)${skipNote} • ${ms} ms`;
        const wellName = scoped.find((w) => w.well_id === wellId)?.well_name ?? wellId;
        recordProcess("RockType", `Lorenz (SMLP) ${wellName}: ${res.units.length} unit(s), Lc=${fmtLc(res.lorenz_coefficient)}`);
        detachRender = renderLorenz(results, res);
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
    },
  };
}

function fmtLc(lc: number | null): string {
  return lc == null || !Number.isFinite(lc) ? "—" : lc.toFixed(3);
}

/** Renders the results and returns a cleanup that detaches the plot's resize observer. */
function renderLorenz(host: HTMLElement, res: LorenzResult): () => void {
  host.innerHTML = "";
  let selected = -1; // index into res.units; -1 = none highlighted

  if (res.note) {
    const note = document.createElement("div");
    note.className = "mc-chain-note";
    note.style.color = "var(--warn)";
    note.textContent = `Note: ${res.note}`;
    host.appendChild(note);
  }

  // Headline: the heterogeneity number + capacity totals.
  const headline = document.createElement("div");
  headline.className = "mc-hist-caption";
  headline.textContent =
    `Lorenz coefficient ${fmtLc(res.lorenz_coefficient)} (0 homogeneous … 1 heterogeneous) • ` +
    `Σk·h ${res.total_kh.toPrecision(4)} mD·m • Σφ·h ${res.total_phih.toPrecision(4)} m`;
  host.appendChild(headline);

  const table = document.createElement("table");
  table.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["", "Unit", "Top", "Base", "n", "Storage %", "Flow %", "Slope", "Character", "φ mean", "k mean (mD)"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";

  const bodyRows: HTMLTableRowElement[] = [];
  const redraw = () => {
    bodyRows.forEach((tr, i) => tr.classList.toggle("ml-diag", i === selected));
    drawSmlp(canvas, res, selected);
  };

  res.units.forEach((u, i) => {
    const tr = document.createElement("tr");
    tr.style.cursor = "pointer";
    const swatchTd = document.createElement("td");
    const sw = document.createElement("span");
    sw.style.display = "inline-block";
    sw.style.width = "0.8rem";
    sw.style.height = "0.8rem";
    sw.style.borderRadius = "2px";
    sw.style.background = faciesColor(u.unit - 1);
    swatchTd.appendChild(sw);
    tr.appendChild(swatchTd);

    const cells = [
      String(u.unit),
      u.depth_top.toFixed(1),
      u.depth_base.toFixed(1),
      String(u.n),
      (u.storage_frac * 100).toFixed(1),
      (u.flow_frac * 100).toFixed(1),
      u.slope.toFixed(2),
      u.character,
      u.phi_mean.toFixed(3),
      u.perm_mean.toPrecision(3),
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

  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent =
    "SMLP — cumulative flow capacity vs storage capacity in depth order, coloured by flow unit; " +
    "dashed diagonal = homogeneous. Click a table row to highlight one unit.";
  host.appendChild(cap);
  host.appendChild(canvas);

  redraw();

  const detachers = [attachResizeRedraw(canvas, () => drawSmlp(canvas, res, selected))];
  return () => detachers.forEach((d) => d());
}

/** Prepares a DPR-scaled 2D context and paints the background; returns null if unavailable. */
function setupCanvas(canvas: HTMLCanvasElement, bg: string): { ctx: CanvasRenderingContext2D; w: number; h: number } | null {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 260;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, w, h);
  return { ctx, w, h };
}

/** The SMLP curve: unit square, 45° dashed diagonal, cumulative polyline coloured by flow unit. */
function drawSmlp(canvas: HTMLCanvasElement, res: LorenzResult, selected: number): void {
  const theme = readTheme(canvas);
  const s = setupCanvas(canvas, theme.bg);
  if (!s) return;
  const { ctx, w, h } = s;
  if (res.points.length === 0) return;

  const padL = 44;
  const padB = 28;
  const padT = 8;
  const padR = 10;
  const X = (x: number) => padL + x * (w - padL - padR);
  const Y = (y: number) => padT + (1 - y) * (h - padT - padB);

  // Quarter gridlines + frame.
  ctx.strokeStyle = theme.grid;
  ctx.lineWidth = 1;
  for (const g of [0.25, 0.5, 0.75]) {
    ctx.beginPath();
    ctx.moveTo(X(g), Y(0));
    ctx.lineTo(X(g), Y(1));
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(X(0), Y(g));
    ctx.lineTo(X(1), Y(g));
    ctx.stroke();
  }
  ctx.strokeStyle = theme.axis;
  ctx.strokeRect(X(0), Y(1), w - padL - padR, h - padT - padB);

  // Homogeneous diagonal.
  ctx.strokeStyle = theme.warn;
  ctx.setLineDash([5, 4]);
  ctx.beginPath();
  ctx.moveTo(X(0), Y(0));
  ctx.lineTo(X(1), Y(1));
  ctx.stroke();
  ctx.setLineDash([]);

  // The stratigraphic walk, one line segment per sample, coloured by its flow unit. The selected
  // unit stays vivid while the rest dim (same interaction as the HFU plots).
  const selUnit = selected >= 0 ? res.units[selected]?.unit : -1;
  let px = 0;
  let py = 0;
  ctx.lineWidth = 2;
  for (const p of res.points) {
    const dim = selUnit !== -1 && p.unit !== selUnit;
    ctx.strokeStyle = faciesColor(p.unit - 1);
    ctx.globalAlpha = dim ? 0.2 : 0.95;
    ctx.beginPath();
    ctx.moveTo(X(px), Y(py));
    ctx.lineTo(X(p.cum_storage), Y(p.cum_flow));
    ctx.stroke();
    px = p.cum_storage;
    py = p.cum_flow;
  }
  ctx.globalAlpha = 1;
  ctx.lineWidth = 1;

  // Unit-boundary markers: a dot at the last point of each unit (except the final overall point,
  // which is always (1,1)).
  ctx.fillStyle = theme.text;
  for (let i = 0; i + 1 < res.points.length; i++) {
    if (res.points[i].unit !== res.points[i + 1].unit) {
      ctx.beginPath();
      ctx.arc(X(res.points[i].cum_storage), Y(res.points[i].cum_flow), 3, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  ctx.fillStyle = theme.text;
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "center";
  ctx.fillText("cumulative storage capacity Σ(φ·h)", (padL + w - padR) / 2, h - 6);
  ctx.save();
  ctx.translate(12, (h - padB + padT) / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.fillText("cumulative flow capacity Σ(k·h)", 0, 0);
  ctx.restore();
}
