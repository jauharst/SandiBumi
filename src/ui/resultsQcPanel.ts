import {
  getCurveData,
  listZones,
  swMethodSpread,
  type MmFluidProps,
  type SwSpreadResult,
  type WellSummary,
  type ZoneEntry,
} from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";
import {
  attachResizeRedraw,
  canvasFont,
  faciesColor,
  fitCanvasBackingStore,
  PlotCanvas,
  readTheme,
} from "./plotCanvas";
import { type PlotContent } from "./plotCommon";

/** Results-QC dashboard (playbook #8) — the "does this interpretation hold together?" surface.
 *
 *  A per-zone QC scorecard (increment 2) plus a detail view (increment 3):
 *
 *   • **Sw-method spread** — the `sw_method_spread` backend: how far Archie/Simandoux/Indonesia/Juhász
 *     (+Waxman-Smits/Dual-Water when Qv/Swb exist) disagree over the zone. A wide spread means the
 *     model choice changes the answer — the classic fresh-water Mahakam-sand trap.
 *   • **Buckles / bulk-volume-water** — BVW = SWE·PHIE. In rock at irreducible saturation BVW is roughly
 *     constant; a high coefficient of variation flags either a genuine transition or an inconsistent Sw.
 *
 *  The detail view draws, for the selected zone, an **Sw-envelope track** (min/max band + each model,
 *  linked to `appState.hoverDepth`) and a **Buckles crossplot** (Sw vs PHIE with constant-BVW
 *  hyperbolae), and exports the whole scorecard to CSV. Traffic-light and plot colours come from the
 *  theme (`--accent` ok / `--accent2` caution / `--warn` alert) — never hard-coded red/green — so the
 *  panel follows light/dark/branded skins.
 *
 *  Rollup rows for the recon incoherence (#2), cutoff sensitivity, and Monte-Carlo P10/P50/P90 (#1) are
 *  the next thing to slot in as extra check rows. */

type CheckStatus = "ok" | "warn" | "alert" | "na";

/** Coefficient-of-variation thresholds for the Buckles BVW check (heuristic prompts, not physics). */
const BVW_CV_OK = 0.15;
const BVW_CV_WARN = 0.3;
/** Fraction-of-divergent-depths thresholds for the Sw-spread check. */
const SPREAD_FRAC_OK = 0.1;
const SPREAD_FRAC_WARN = 0.4;
/** Model draw order → stable colour per model across zones (Archie first so it reads as the baseline). */
const MODEL_ORDER = ["Archie", "Simandoux", "Indonesia", "Juhasz", "Waxman-Smits", "Dual-Water"];

interface BucklesResult {
  status: CheckStatus;
  detail: string;
  tooltip: string;
  /** Finite SWE / PHIE pairs for the crossplot, and their BVW mean / CV / count. */
  swe: number[];
  phie: number[];
  mean: number;
  cv: number;
  n: number;
}

interface ZoneDatum {
  name: string;
  top: number | null;
  base: number | null;
  spread: SwSpreadResult | null;
  spreadErr: string | null;
  buckles: BucklesResult;
  el: HTMLElement;
}

function modelColor(name: string): string {
  const i = MODEL_ORDER.indexOf(name);
  return faciesColor(i >= 0 ? i : MODEL_ORDER.length);
}

function numInput(value: number, step = "any", width = "5.5em"): HTMLInputElement {
  const i = document.createElement("input");
  i.className = "form-control";
  i.type = "number";
  i.step = step;
  i.style.width = width;
  i.value = String(value);
  return i;
}

function num(input: HTMLInputElement, fallback: number): number {
  const v = parseFloat(input.value);
  return Number.isFinite(v) ? v : fallback;
}

/** A traffic-light check row: coloured dot + label + detail, with the full notes trail as a tooltip. */
function checkRow(status: CheckStatus, label: string, detail: string, tooltip?: string): HTMLElement {
  const row = document.createElement("div");
  row.className = "rqc-check";
  const dot = document.createElement("span");
  dot.className = `rqc-dot rqc-dot-${status}`;
  const name = document.createElement("span");
  name.className = "rqc-check-label";
  name.textContent = label;
  const det = document.createElement("span");
  det.className = "rqc-check-detail";
  det.textContent = detail;
  if (tooltip) row.title = tooltip;
  row.append(dot, name, det);
  return row;
}

/** BVW = SWE·PHIE over a depth window, aligned by index (both curves share the well grid). */
async function computeBuckles(wellId: string, dmin: number | null, dmax: number | null): Promise<BucklesResult> {
  const empty = (status: CheckStatus, detail: string, tooltip: string): BucklesResult => ({
    status, detail, tooltip, swe: [], phie: [], mean: NaN, cv: NaN, n: 0,
  });
  let series;
  try {
    series = await getCurveData(wellId, ["SWE", "PHIE"], dmin, dmax);
  } catch (err) {
    return empty("na", "curve fetch failed", String(err));
  }
  const swe = series.find((s) => s.curve_name.toUpperCase() === "SWE");
  const phie = series.find((s) => s.curve_name.toUpperCase() === "PHIE");
  if (!swe || !phie) return empty("na", "no SWE/PHIE curve", "Buckles needs both SWE and PHIE.");
  if (swe.value.length !== phie.value.length) {
    return empty("na", "SWE/PHIE grids differ", "The two curves are not on the same depth grid.");
  }
  const sArr: number[] = [];
  const pArr: number[] = [];
  const bvw: number[] = [];
  for (let i = 0; i < swe.value.length; i++) {
    const s = swe.value[i];
    const p = phie.value[i];
    if (Number.isFinite(s) && Number.isFinite(p)) {
      sArr.push(s);
      pArr.push(p);
      bvw.push(s * p);
    }
  }
  if (bvw.length < 5) return empty("na", "too few samples", `only ${bvw.length} finite SWE·PHIE pairs`);
  const mean = bvw.reduce((a, b) => a + b, 0) / bvw.length;
  const variance = bvw.reduce((a, b) => a + (b - mean) * (b - mean), 0) / bvw.length;
  const cv = mean > 0 ? Math.sqrt(variance) / mean : Infinity;
  const status: CheckStatus = cv <= BVW_CV_OK ? "ok" : cv <= BVW_CV_WARN ? "warn" : "alert";
  const detail = `BVW ${mean.toFixed(3)} · CV ${(cv * 100).toFixed(0)}% · n=${bvw.length}`;
  const tooltip =
    status === "ok"
      ? "BVW is tight — consistent with a single irreducible saturation."
      : "BVW varies across the zone — a genuine transition (expected) or an inconsistent Sw. Check the Buckles crossplot.";
  return { status, detail, tooltip, swe: sArr, phie: pArr, mean, cv, n: bvw.length };
}

/** Traffic-light + detail for the Sw-method spread result. */
function spreadCheck(spread: SwSpreadResult): { status: CheckStatus; detail: string; tooltip: string } {
  const models = spread.methods.map((m) => m.name).join(", ");
  if (spread.methods.length < 2 || spread.n_samples === 0) {
    return { status: "na", detail: `${spread.methods.length} model(s) — not comparable`, tooltip: spread.notes.join("\n") };
  }
  const frac = spread.frac_divergent ?? 0;
  const status: CheckStatus = frac <= SPREAD_FRAC_OK ? "ok" : frac <= SPREAD_FRAC_WARN ? "warn" : "alert";
  const mean = spread.mean_spread ?? NaN;
  const max = spread.max_spread ?? NaN;
  const worstAt = Number.isFinite(spread.max_spread_depth ?? NaN) ? ` @ ${(spread.max_spread_depth ?? 0).toFixed(0)} m` : "";
  const detail = `mean ${mean.toFixed(3)} · max ${max.toFixed(3)}${worstAt} · ${(frac * 100).toFixed(0)}% divergent`;
  return { status, detail, tooltip: `Models: ${models}\n${spread.notes.join("\n")}` };
}

// ---- CSV -------------------------------------------------------------------------------------

function csvCell(v: string | number | null): string {
  if (v === null || v === undefined || (typeof v === "number" && !Number.isFinite(v))) return "";
  const s = typeof v === "number" ? String(v) : v;
  return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
}

function scorecardCsv(well: WellSummary, zones: ZoneDatum[]): string {
  const header = [
    "well", "zone", "top", "base", "models", "mean_spread", "max_spread", "max_spread_depth",
    "frac_divergent", "bvw_mean", "bvw_cv", "bvw_n",
  ];
  const lines = [header.join(",")];
  for (const z of zones) {
    const s = z.spread;
    lines.push(
      [
        csvCell(well.well_name), csvCell(z.name), csvCell(z.top), csvCell(z.base),
        csvCell(s ? s.methods.map((m) => m.name).join(" | ") : z.spreadErr ?? ""),
        csvCell(s?.mean_spread ?? null), csvCell(s?.max_spread ?? null), csvCell(s?.max_spread_depth ?? null),
        csvCell(s?.frac_divergent ?? null), csvCell(z.buckles.mean), csvCell(z.buckles.cv), csvCell(z.buckles.n),
      ].join(","),
    );
  }
  return lines.join("\n");
}

function downloadCsv(filename: string, text: string): void {
  const blob = new Blob([text], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

// ---- Plots -----------------------------------------------------------------------------------

/** Sw-envelope track: depth (Y, inverted) vs Sw (X), a min/max band, one line per model, and a marker
 *  at the crosshair depth. */
function drawEnvelope(canvas: HTMLCanvasElement, spread: SwSpreadResult | null): void {
  fitCanvasBackingStore(canvas);
  const theme = readTheme(canvas);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  if (!spread || spread.depth.length < 2) {
    // clear + hint
    const pc = new PlotCanvas(canvas, { label: "Sw", min: 0, max: 1, log: false, invert: false }, { label: "Depth (m)", min: 0, max: 1, log: false, invert: true });
    pc.drawFrame();
    return;
  }
  const depths = spread.depth;
  let dMin = Infinity;
  let dMax = -Infinity;
  for (const d of depths) {
    if (Number.isFinite(d)) {
      dMin = Math.min(dMin, d);
      dMax = Math.max(dMax, d);
    }
  }
  if (!(dMax > dMin)) dMax = dMin + 1;
  const pc = new PlotCanvas(
    canvas,
    { label: "Sw (v/v)", min: 0, max: 1, log: false, invert: false },
    { label: "Depth (m)", min: dMin, max: dMax, log: false, invert: true },
  );
  pc.drawFrame();

  // min/max envelope band
  const r = pc.plotRect;
  const idx: number[] = [];
  for (let i = 0; i < depths.length; i++) {
    if (Number.isFinite(depths[i]) && Number.isFinite(spread.sw_min[i] ?? NaN) && Number.isFinite(spread.sw_max[i] ?? NaN)) idx.push(i);
  }
  if (idx.length >= 2) {
    ctx.save();
    ctx.beginPath();
    ctx.rect(r.x0, r.y0, r.w, r.h);
    ctx.clip();
    ctx.beginPath();
    idx.forEach((i, k) => {
      const [px, py] = pc.toPx(spread.sw_min[i] as number, depths[i]);
      if (k === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    });
    for (let k = idx.length - 1; k >= 0; k--) {
      const i = idx[k];
      const [px, py] = pc.toPx(spread.sw_max[i] as number, depths[i]);
      ctx.lineTo(px, py);
    }
    ctx.closePath();
    ctx.globalAlpha = 0.16;
    ctx.fillStyle = theme.accent;
    ctx.fill();
    ctx.globalAlpha = 1;
    ctx.restore();
  }

  // one line per model
  for (const m of spread.methods) {
    const pts: [number, number][] = [];
    for (let i = 0; i < depths.length; i++) {
      const v = m.values[i];
      if (v !== null && Number.isFinite(v) && Number.isFinite(depths[i])) pts.push([v, depths[i]]);
    }
    pc.drawLine(pts, modelColor(m.name), 1.3);
  }

  // crosshair depth marker (horizontal)
  const hd = appState.hoverDepth.get();
  if (hd !== null && hd >= dMin && hd <= dMax) {
    const [, py] = pc.toPx(0, hd);
    ctx.save();
    ctx.strokeStyle = theme.warn;
    ctx.lineWidth = 1.2;
    ctx.setLineDash([4, 3]);
    ctx.beginPath();
    ctx.moveTo(r.x0, py);
    ctx.lineTo(r.x0 + r.w, py);
    ctx.stroke();
    ctx.restore();
  }
}

/** Buckles crossplot: Sw (X) vs PHIE (Y), scatter + constant-BVW hyperbolae (Sw·φ = const). */
function drawBucklesPlot(canvas: HTMLCanvasElement, b: BucklesResult): void {
  fitCanvasBackingStore(canvas);
  const theme = readTheme(canvas);
  let phiMax = 0.4;
  for (const p of b.phie) if (Number.isFinite(p)) phiMax = Math.max(phiMax, p);
  phiMax = Math.min(0.6, Math.ceil(phiMax * 10) / 10);
  const pc = new PlotCanvas(
    canvas,
    { label: "Sw (v/v)", min: 0, max: 1, log: false, invert: false },
    { label: "PHIE (v/v)", min: 0, max: phiMax, log: false, invert: false },
  );
  pc.drawFrame();

  // constant bulk-volume-water hyperbolae φ = BVW / Sw
  for (const bvw of [0.02, 0.04, 0.06, 0.08, 0.1]) {
    const pts: [number, number][] = [];
    for (let sw = bvw / phiMax; sw <= 1.0001; sw += 0.01) {
      const phi = bvw / Math.min(sw, 1);
      if (phi <= phiMax) pts.push([Math.min(sw, 1), phi]);
    }
    pc.drawLine(pts, theme.grid, 1, [3, 3]);
    if (pts.length) {
      const ctx = pc.ctx;
      const [px, py] = pc.toPx(pts[0][0], pts[0][1]);
      ctx.save();
      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 9);
      ctx.textAlign = "left";
      ctx.fillText(bvw.toFixed(2), px + 2, py + 9);
      ctx.restore();
    }
  }
  pc.drawScatter(b.swe, b.phie, undefined, 2.0);
}

export async function buildResultsQcContent(
  well: WellSummary,
  setStatus: (text: string) => void,
): Promise<PlotContent> {
  const content = document.createElement("div");
  content.className = "results-qc";

  // ---- Sw parameters (editable defaults; the user confirms them — nothing fabricated) ----
  const controls = document.createElement("div");
  controls.className = "rqc-controls";
  const rwIn = numInput(0.1, "0.0001");
  const rwTIn = numInput(75);
  const ftIn = numInput(210);
  const mIn = numInput(2, "0.01");
  const nIn = numInput(2, "0.01");
  const rshIn = numInput(4, "0.1");
  const aIn = numInput(1, "0.1");
  const divIn = numInput(0.1, "0.01");
  controls.append(
    formRow("Rw", rwIn),
    formRow("Rw °F", rwTIn),
    formRow("Form °F", ftIn),
    formRow("m", mIn),
    formRow("n", nIn),
    formRow("Rsh", rshIn),
    formRow("a", aIn),
    formRow("Diverge", divIn),
  );
  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent rqc-run";
  runBtn.textContent = "Recompute";
  const csvBtn = document.createElement("button");
  csvBtn.className = "plot-export-btn rqc-csv";
  csvBtn.textContent = "⭳ CSV";
  csvBtn.title = "Export the per-zone scorecard as CSV";
  controls.append(runBtn, csvBtn);
  content.append(controls);

  const statusLine = document.createElement("div");
  statusLine.className = "rqc-status";
  content.append(statusLine);

  const body = document.createElement("div");
  body.className = "rqc-body";
  content.append(body);

  // ---- detail view (Sw-envelope track + Buckles crossplot for one zone) ----
  const detail = document.createElement("div");
  detail.className = "rqc-detail";
  const detailControls = document.createElement("div");
  detailControls.className = "rqc-detail-controls";
  const zoneSel = document.createElement("select");
  zoneSel.className = "form-control";
  detailControls.append(formRow("Detail zone", zoneSel));
  const legend = document.createElement("div");
  legend.className = "rqc-legend";
  detailControls.append(legend);
  detail.append(detailControls);
  const plots = document.createElement("div");
  plots.className = "rqc-plots";
  const envWrap = document.createElement("div");
  envWrap.className = "rqc-plot";
  const envTitle = document.createElement("div");
  envTitle.className = "rqc-plot-title";
  envTitle.textContent = "Sw-method envelope";
  const envCanvas = document.createElement("canvas");
  envCanvas.className = "plot-canvas rqc-canvas";
  envWrap.append(envTitle, envCanvas);
  const buckWrap = document.createElement("div");
  buckWrap.className = "rqc-plot";
  const buckTitle = document.createElement("div");
  buckTitle.className = "rqc-plot-title";
  buckTitle.textContent = "Buckles (Sw vs PHIE)";
  const buckCanvas = document.createElement("canvas");
  buckCanvas.className = "plot-canvas rqc-canvas";
  buckWrap.append(buckTitle, buckCanvas);
  plots.append(envWrap, buckWrap);
  detail.append(plots);
  content.append(detail);

  const fluid = (): MmFluidProps => {
    const rw = num(rwIn, 0.1);
    const rwT = num(rwTIn, 75);
    return {
      rw,
      rw_temp_f: rwT,
      rmf: rw, // filtrate props do not affect the Sw envelope (virgin-zone conductivities only)
      rmf_temp_f: rwT,
      ftemp_f: num(ftIn, 210),
      m: num(mIn, 2),
      n: num(nIn, 2),
      mud_type: "WATER",
      rsh: num(rshIn, 4),
      archie_a: num(aIn, 1),
    };
  };

  let zoneData: ZoneDatum[] = [];

  const selectedZone = (): ZoneDatum | undefined => zoneData.find((z) => z.name === zoneSel.value);

  const rebuildLegend = () => {
    legend.textContent = "";
    const z = selectedZone();
    const models = z?.spread?.methods ?? [];
    for (const m of models) {
      const item = document.createElement("span");
      item.className = "rqc-legend-item";
      const sw = document.createElement("span");
      sw.className = "rqc-swatch";
      sw.style.background = modelColor(m.name);
      item.append(sw, document.createTextNode(m.name));
      legend.append(item);
    }
    const bandNote = document.createElement("span");
    bandNote.className = "rqc-legend-note";
    bandNote.textContent = "shaded = min–max envelope · dashed = constant BVW";
    legend.append(bandNote);
  };

  const drawDetail = () => {
    const z = selectedZone();
    drawEnvelope(envCanvas, z?.spread ?? null);
    drawBucklesPlot(buckCanvas, z?.buckles ?? { status: "na", detail: "", tooltip: "", swe: [], phie: [], mean: NaN, cv: NaN, n: 0 });
    rebuildLegend();
  };

  zoneSel.addEventListener("change", drawDetail);

  const compute = async () => {
    runBtn.disabled = true;
    statusLine.textContent = "Computing QC checks…";
    body.textContent = "";
    zoneData = [];

    let zones: ZoneEntry[] = [];
    try {
      zones = await listZones(well.well_id);
    } catch {
      zones = [];
    }
    const targets: { name: string; top: number | null; base: number | null }[] = zones.length
      ? zones.map((z) => ({ name: z.zone_name, top: z.top_depth, base: z.bottom_depth }))
      : [{ name: "All depth", top: null, base: null }];

    const divThreshold = num(divIn, 0.1);
    const f = fluid();
    let flagged = 0;

    for (const t of targets) {
      const card = document.createElement("div");
      card.className = "rqc-card";
      const head = document.createElement("div");
      head.className = "rqc-card-head";
      head.textContent =
        t.top !== null && t.base !== null ? `${t.name} (${t.top.toFixed(0)}–${t.base.toFixed(0)} m)` : t.name;
      card.append(head);

      let spread: SwSpreadResult | null = null;
      let spreadErr: string | null = null;
      try {
        spread = await swMethodSpread({
          well_id: well.well_id,
          depth_min: t.top,
          depth_max: t.base,
          fluid: f,
          divergence_threshold: divThreshold,
        });
        const c = spreadCheck(spread);
        if (c.status === "alert" || c.status === "warn") flagged++;
        card.append(checkRow(c.status, "Sw-method spread", c.detail, c.tooltip));
      } catch (err) {
        spreadErr = String(err);
        card.append(checkRow("alert", "Sw-method spread", "failed", spreadErr));
        flagged++;
      }

      const buckles = await computeBuckles(well.well_id, t.top, t.base);
      if (buckles.status === "alert" || buckles.status === "warn") flagged++;
      card.append(checkRow(buckles.status, "Buckles (BVW)", buckles.detail, buckles.tooltip));

      // Clicking a card focuses the detail plots on that zone.
      card.addEventListener("click", () => {
        zoneSel.value = t.name;
        drawDetail();
      });

      body.append(card);
      zoneData.push({ name: t.name, top: t.top, base: t.base, spread, spreadErr, buckles, el: card });
    }

    // Refresh the detail-zone dropdown, keeping the current pick if it survives.
    const prev = zoneSel.value;
    zoneSel.textContent = "";
    for (const z of zoneData) {
      const o = document.createElement("option");
      o.value = z.name;
      o.textContent = z.name;
      zoneSel.append(o);
    }
    zoneSel.value = zoneData.some((z) => z.name === prev) ? prev : (zoneData[0]?.name ?? "");
    drawDetail();

    statusLine.textContent = `${targets.length} zone(s) · ${flagged} check(s) flagged`;
    setStatus(`Results-QC: ${well.well_name} — ${targets.length} zone(s), ${flagged} flagged`);
    runBtn.disabled = false;
  };

  runBtn.addEventListener("click", () => void compute());
  csvBtn.addEventListener("click", () => {
    if (!zoneData.length) {
      setStatus("Nothing to export yet — Recompute first");
      return;
    }
    downloadCsv(`${well.well_name}_results_qc.csv`, scorecardCsv(well, zoneData));
    setStatus(`Exported ${zoneData.length}-zone scorecard for ${well.well_name}`);
  });
  void compute();

  // Highlight the zone card the crosshair is over, and redraw the envelope's depth marker.
  let rafId = 0;
  const unsubHover = appState.hoverDepth.subscribe((d) => {
    for (const z of zoneData) {
      const active = d !== null && z.top !== null && z.base !== null && d >= z.top && d <= z.base;
      z.el.classList.toggle("rqc-card-active", active);
    }
    if (!rafId) {
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        drawEnvelope(envCanvas, selectedZone()?.spread ?? null);
      });
    }
  });
  const unsubTheme = appState.themeVersion.subscribe(() => drawDetail());
  const disposeEnvResize = attachResizeRedraw(envCanvas, drawDetail);
  const disposeBuckResize = attachResizeRedraw(buckCanvas, drawDetail);

  return {
    el: content,
    dispose: () => {
      unsubHover();
      unsubTheme();
      disposeEnvResize();
      disposeBuckResize();
      if (rafId) cancelAnimationFrame(rafId);
    },
  };
}
