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
import { type PlotContent } from "./plotCommon";

/** Results-QC dashboard (playbook #8) — the "does this interpretation hold together?" surface.
 *
 *  Increment 1 (this file): a per-zone QC scorecard. For every zone of the selected well it runs two
 *  cheap, read-only checks and shows a traffic-light per check:
 *
 *   • **Sw-method spread** — the new `sw_method_spread` backend: how far Archie/Simandoux/Indonesia/
 *     Juhász (+Waxman-Smits/Dual-Water when Qv/Swb exist) disagree over the zone. A wide spread means
 *     the model choice changes the answer — the classic fresh-water Mahakam-sand trap.
 *   • **Buckles / bulk-volume-water** — BVW = SWE·PHIE. In a rock at irreducible saturation BVW is
 *     roughly constant; a high coefficient of variation flags either a genuine transition zone or an
 *     inconsistent Sw. This is a prompt to open the crossplot (increment 3), not a verdict.
 *
 *  The Sw-envelope track, the Buckles crossplot, CSV export, and the recon/cutoff/Monte-Carlo rollup
 *  rows land in the next increments; the scorecard is built so those slot in as extra check rows.
 *
 *  Traffic-light colours come from the theme (`--accent` ok, `--accent2` caution, `--warn` alert) —
 *  never hard-coded red/green — so the card follows light/dark/branded skins. */

type CheckStatus = "ok" | "warn" | "alert" | "na";

/** Coefficient-of-variation thresholds for the Buckles BVW check (heuristic prompts, not physics). */
const BVW_CV_OK = 0.15;
const BVW_CV_WARN = 0.3;
/** Fraction-of-divergent-depths thresholds for the Sw-spread check. */
const SPREAD_FRAC_OK = 0.1;
const SPREAD_FRAC_WARN = 0.4;

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

/** BVW = SWE·PHIE statistics over a depth window, aligned by index (both curves share the well grid). */
async function computeBuckles(
  wellId: string,
  dmin: number | null,
  dmax: number | null,
): Promise<{ status: CheckStatus; detail: string; tooltip: string }> {
  let series;
  try {
    series = await getCurveData(wellId, ["SWE", "PHIE"], dmin, dmax);
  } catch (err) {
    return { status: "na", detail: "curve fetch failed", tooltip: String(err) };
  }
  const swe = series.find((s) => s.curve_name.toUpperCase() === "SWE");
  const phie = series.find((s) => s.curve_name.toUpperCase() === "PHIE");
  if (!swe || !phie) {
    return { status: "na", detail: "no SWE/PHIE curve", tooltip: "Buckles needs both SWE and PHIE." };
  }
  if (swe.value.length !== phie.value.length) {
    return { status: "na", detail: "SWE/PHIE grids differ", tooltip: "The two curves are not on the same depth grid." };
  }
  const bvw: number[] = [];
  for (let i = 0; i < swe.value.length; i++) {
    const s = swe.value[i];
    const p = phie.value[i];
    if (Number.isFinite(s) && Number.isFinite(p)) bvw.push(s * p);
  }
  if (bvw.length < 5) {
    return { status: "na", detail: "too few samples", tooltip: `only ${bvw.length} finite SWE·PHIE pairs` };
  }
  const mean = bvw.reduce((a, b) => a + b, 0) / bvw.length;
  const variance = bvw.reduce((a, b) => a + (b - mean) * (b - mean), 0) / bvw.length;
  const cv = mean > 0 ? Math.sqrt(variance) / mean : Infinity;
  const status: CheckStatus = cv <= BVW_CV_OK ? "ok" : cv <= BVW_CV_WARN ? "warn" : "alert";
  const detail = `BVW ${mean.toFixed(3)} · CV ${(cv * 100).toFixed(0)}% · n=${bvw.length}`;
  const tooltip =
    status === "ok"
      ? "BVW is tight — consistent with a single irreducible saturation."
      : "BVW varies across the zone — a genuine transition (expected) or an inconsistent Sw. Check the Buckles crossplot.";
  return { status, detail, tooltip };
}

/** Traffic-light + detail for the Sw-method spread result. */
function spreadCheck(spread: SwSpreadResult): { status: CheckStatus; detail: string; tooltip: string } {
  const models = spread.methods.map((m) => m.name).join(", ");
  if (spread.methods.length < 2 || spread.n_samples === 0) {
    return {
      status: "na",
      detail: `${spread.methods.length} model(s) — not comparable`,
      tooltip: spread.notes.join("\n"),
    };
  }
  const frac = spread.frac_divergent ?? 0;
  const status: CheckStatus = frac <= SPREAD_FRAC_OK ? "ok" : frac <= SPREAD_FRAC_WARN ? "warn" : "alert";
  const mean = spread.mean_spread ?? NaN;
  const max = spread.max_spread ?? NaN;
  const worstAt = Number.isFinite(spread.max_spread_depth ?? NaN) ? ` @ ${(spread.max_spread_depth ?? 0).toFixed(0)} m` : "";
  const detail = `mean ${mean.toFixed(3)} · max ${max.toFixed(3)}${worstAt} · ${(frac * 100).toFixed(0)}% divergent`;
  const tooltip = `Models: ${models}\n${spread.notes.join("\n")}`;
  return { status, detail, tooltip };
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
  controls.append(runBtn);
  content.append(controls);

  const statusLine = document.createElement("div");
  statusLine.className = "rqc-status";
  content.append(statusLine);

  const body = document.createElement("div");
  body.className = "rqc-body";
  content.append(body);

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

  // Cards keep their depth range so hoverDepth can highlight the active zone.
  let cards: { top: number | null; base: number | null; el: HTMLElement }[] = [];

  const compute = async () => {
    runBtn.disabled = true;
    statusLine.textContent = "Computing QC checks…";
    body.textContent = "";
    cards = [];

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

      // Sw-method spread
      try {
        const spread = await swMethodSpread({
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
        card.append(checkRow("alert", "Sw-method spread", "failed", String(err)));
        flagged++;
      }

      // Buckles / BVW
      const b = await computeBuckles(well.well_id, t.top, t.base);
      if (b.status === "alert" || b.status === "warn") flagged++;
      card.append(checkRow(b.status, "Buckles (BVW)", b.detail, b.tooltip));

      body.append(card);
      cards.push({ top: t.top, base: t.base, el: card });
    }

    statusLine.textContent = `${targets.length} zone(s) · ${flagged} check(s) flagged`;
    setStatus(`Results-QC: ${well.well_name} — ${targets.length} zone(s), ${flagged} flagged`);
    runBtn.disabled = false;
  };

  runBtn.addEventListener("click", () => void compute());
  void compute();

  // Highlight the zone card the crosshair is over.
  const unsubHover = appState.hoverDepth.subscribe((d) => {
    for (const c of cards) {
      const active =
        d !== null && c.top !== null && c.base !== null && d >= c.top && d <= c.base;
      c.el.classList.toggle("rqc-card-active", active);
    }
  });

  return { el: content, dispose: () => unsubHover() };
}
