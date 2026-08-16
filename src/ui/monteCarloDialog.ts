import {
  listDocuments,
  listModules,
  listZones,
  runMonteCarlo,
  type ChainStep,
  type McConvergence,
  type McCorrelation,
  type McDistribution,
  type McMetricSet,
  type McParam,
  type McRequest,
  type McResult,
  type McSensParam,
  type McSensZone,
  type McZoneResult,
  type ModuleSpec,
  type Pctl,
} from "../ipc";
import { loadCutoffDefaults } from "./cutoffs";
import { formRow } from "./modal";
import { canvasFont, readTheme } from "./plotCanvas";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";
import { requestRunCustody } from "./runCustody";
import { buildParamSources, PARAM_SOURCE_TOPICS } from "./paramSources";

const WORKFLOW_DOC_TYPE = "workflow";
const DEFAULT_STEPS = ["vsh_gr", "phi_dn", "sw_indo"];

type DistKind = McDistribution["kind"];

interface McRow {
  param: string;
  kind: DistKind;
  a: number; // normal.mean / uniform.lo / triangular.lo
  b: number; // normal.sd   / uniform.hi / triangular.mode
  c: number; // (triangular.hi)
  zone: string; // "" = well-wide; a zone name restricts the draw to that zone
}

/** One rank-correlation pair in the mini-editor (Iman–Conover on the backend). */
interface CorrRow {
  a: string;
  b: string;
  rho: number;
}

let zoneListSeq = 0;

interface ParamCandidate {
  name: string;
  default: number;
  unit: string;
  desc: string;
  sourcesTopic: string;
}

/** One imported default uncertainty width. `pct` → the width is a percentage of the parameter's
 *  own value (IP "%" shift); otherwise it is absolute, in the parameter's own unit (IP "Linear"). */
interface McSeed {
  w: number;
  pct: boolean;
}

/** Per-parameter Monte Carlo widths seeded from IP's `MonteCarloDefaults.par` (Tier-A import —
 *  provenance, mapping table and the adopted σ reading live in `docs/ref_monte_carlo_seeds.md`).
 *  Only the VSH / porosity / saturation parameters that map 1:1 onto a SandiBumi module argument
 *  are imported; everything else falls back to the generic width heuristic in `distDefaults`.
 *  The point is the *units*: a matrix density deserves ±0.03 g/cc, not ±10% of 2.645. */
const IP_MC_SEEDS: Record<string, McSeed> = {
  A: { w: 0.1, pct: false }, // a factor
  M: { w: 0.2, pct: false }, // m exponent
  N: { w: 0.2, pct: false }, // n exponent
  RW: { w: 20, pct: true }, // Rw
  RT_SH: { w: 20, pct: true }, // Res Clay
  GR_MA: { w: 10, pct: false }, // Gr Clean
  GR_SH: { w: 10, pct: false }, // Gr Clay
  RHO_MA: { w: 0.03, pct: false }, // Rho Matrix
  RHO_FL: { w: 0.02, pct: false }, // Rho Fluid
  RHO_SH: { w: 0.05, pct: false }, // Rho Clay
  RHO_DSH: { w: 0.1, pct: false }, // Rho Dry Clay
  NPHI_SH: { w: 0.05, pct: false }, // Neu Clay
};

/** The seeded width for `param` at central value `d`, or NaN when it has no seed (or a `%` seed
 *  resolved against a zero value, which would collapse the row to a point mass). */
function seedWidth(param: string | undefined, d: number): number {
  const s = param ? IP_MC_SEEDS[param] : undefined;
  if (!s) return NaN;
  const w = s.pct ? (Math.abs(d) * s.w) / 100 : s.w;
  return w > 0 ? w : NaN;
}

/** The muted `IP` badge marking a row whose width came from the Tier-A table. Always rendered —
 *  an unseeded row keeps an invisible placeholder so the distribution fields stay column-aligned
 *  with the seeded rows above and below it. */
function seedTag(param: string): HTMLElement {
  const s = document.createElement("span");
  s.className = IP_MC_SEEDS[param] ? "mc-seed-tag" : "mc-seed-tag mc-seed-tag-off";
  s.textContent = "IP";
  if (IP_MC_SEEDS[param]) {
    s.title = "Default width seeded from IP MonteCarloDefaults.par (Tier-A) — see docs/ref_monte_carlo_seeds.md";
  }
  return s;
}

/** User-tweakable look of the HPV histogram (the ⚙ properties panel). `barColor`/height blank →
 *  fall back to the theme accent / CSS default, so a fresh run tracks the active theme. */
interface HistOptions {
  height: number;
  barColor: string; // "" → theme --accent
  showMarkers: boolean; // P10/P50/P90 dashed lines
  showGrid: boolean; // horizontal frequency gridlines
  showYAxis: boolean; // frequency tick labels on the left
}

/** User-tweakable look of the tornado. Height 0 → auto-size to the parameter count. */
interface TornOptions {
  height: number;
  loColor: string; // "" → theme --accent  (the low-side / negative bars)
  hiColor: string; // "" → theme --accent2 (the high-side / positive bars)
  showStripes: boolean; // zebra row backgrounds
  showRho: boolean; // ρ annotations (still gated by significance)
}

function emptyStep(module: string): ChainStep {
  return { module, log_inputs: {}, params: {}, opts: {} };
}

/** Monte Carlo uncertainty (Phase 9): put distributions on model parameters, run N seeded
 *  realizations of a chain in memory, and read the P10/P50/P90 spread of net pay / NTG /
 *  PHIE / SWE / HPV per zone plus an HPV histogram. */
/** Hosted as a dock pane (workspace component "montecarlo"), not a popup. */
export async function buildMonteCarloContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const modules = await listModules().catch(() => [] as ModuleSpec[]);
  const moduleByName = new Map(modules.map((m) => [m.name, m]));
  const scope = await buildWellScope();

  let steps: ChainStep[] = DEFAULT_STEPS.map(emptyStep);
  let mcRows: McRow[] = [];

  // Plot look, held at pane scope so tweaks survive re-runs (renderResults reads these each time).
  const histOpts: HistOptions = { height: 220, barColor: "", showMarkers: true, showGrid: true, showYAxis: true };
  const tornOpts: TornOptions = { height: 0, loColor: "", hiColor: "", showStripes: true, showRho: true };

  const content = document.createElement("div");
  content.className = "mc-dialog";

  // --- Chain source --------------------------------------------------------
  const chainSelect = document.createElement("select");
  chainSelect.className = "form-control";
  const defaultOpt = document.createElement("option");
  defaultOpt.value = "";
  defaultOpt.textContent = "Default: VSH → Porosity → SW-Indo";
  chainSelect.appendChild(defaultOpt);
  const savedDocs = await listDocuments(WORKFLOW_DOC_TYPE).catch(() => []);
  for (const d of savedDocs) {
    const o = document.createElement("option");
    o.value = d.name;
    o.textContent = `Workflow: ${d.name}`;
    chainSelect.appendChild(o);
  }
  const chainNote = document.createElement("div");
  chainNote.className = "mc-chain-note";

  function describeSteps(): void {
    chainNote.textContent = steps.map((s) => moduleByName.get(s.module)?.title ?? s.module).join("  →  ");
  }

  chainSelect.addEventListener("change", async () => {
    if (!chainSelect.value) {
      steps = DEFAULT_STEPS.map(emptyStep);
    } else {
      const docs = await listDocuments(WORKFLOW_DOC_TYPE).catch(() => []);
      const doc = docs.find((d) => d.name === chainSelect.value);
      if (doc) {
        try {
          const parsed = JSON.parse(doc.json) as { steps: ChainStep[] };
          steps = (parsed.steps ?? []).map((s) => ({
            module: s.module,
            log_inputs: s.log_inputs ?? {},
            params: s.params ?? {},
            opts: s.opts ?? {},
          }));
        } catch {
          steps = DEFAULT_STEPS.map(emptyStep);
        }
      }
    }
    describeSteps();
    refreshCandidates();
    renderMcRows();
  });
  content.appendChild(formRow("Chain", chainSelect, "Which module chain each realization runs. Build/save chains in the Workflow Builder."));
  content.appendChild(chainNote);

  // --- Uncertain parameters ------------------------------------------------
  let candidates: ParamCandidate[] = [];
  function refreshCandidates(): void {
    const seen = new Map<string, ParamCandidate>();
    for (const step of steps) {
      const spec = moduleByName.get(step.module);
      if (!spec) continue;
      for (const arg of spec.args) {
        if (arg.kind !== "param" || seen.has(arg.name)) continue;
        seen.set(arg.name, {
          name: arg.name,
          default: step.params[arg.name] ?? parseFloat(arg.default),
          unit: arg.unit,
          desc: arg.desc,
          sourcesTopic: arg.sources_topic ?? "",
        });
      }
    }
    candidates = [...seen.values()];
  }

  const mcList = document.createElement("div");
  mcList.className = "mc-params";
  const addParamBtn = mini("+ Add uncertain parameter", () => {
    const first = candidates[0];
    if (!first) {
      setStatus("This chain has no numeric parameters to vary");
      return;
    }
    mcRows.push(defaultRow(first));
    renderMcRows();
  });

  /** Sensible [a, b, c] for a parameter's central value under a given distribution:
   *  normal → [mean, sd]; uniform → [lo, hi]; triangular → [lo, mode, hi].
   *  `param` (when it has an IP_MC_SEEDS entry) supplies the width in the parameter's own units —
   *  σ for normal, the half-range otherwise. Unseeded parameters keep the generic width off the
   *  value's magnitude, floored so a zero-valued default still spreads. */
  function distDefaults(d0: number, kind: DistKind, param?: string): [number, number, number] {
    const d = Number.isFinite(d0) ? d0 : 0;
    const seeded = seedWidth(param, d);
    const has = Number.isFinite(seeded);
    const spread = has ? seeded : Math.max(Math.abs(d) * 0.1, 0.01);
    const wide = has ? seeded : Math.max(Math.abs(d) * 0.2, 0.02);
    if (kind === "normal") return [d, spread, d + spread];
    if (kind === "uniform") return [d - wide, d + wide, d + wide];
    return [d - wide, d, d + wide]; // triangular: lo, mode, hi
  }

  function defaultRow(c: ParamCandidate): McRow {
    const [a, b, cc] = distDefaults(c.default, "normal", c.name);
    return { param: c.name, kind: "normal", a, b, c: cc, zone: "" };
  }

  // Zone-name suggestions for the per-row zone scope, fetched lazily from the first well in
  // scope (zones are matched per well by NAME on the backend, so one list is a fair guide).
  const zoneListId = `mc-zones-${++zoneListSeq}`;
  const zoneList = document.createElement("datalist");
  zoneList.id = zoneListId;
  content.appendChild(zoneList);
  let zoneFetchWell = "";
  async function refreshZoneOptions(): Promise<void> {
    const ids = scope.getWellIds();
    if (ids.length === 0 || ids[0] === zoneFetchWell) return;
    zoneFetchWell = ids[0];
    const zones = await listZones(ids[0]).catch(() => []);
    zoneList.innerHTML = "";
    for (const z of zones) {
      const o = document.createElement("option");
      o.value = z.zone_name;
      zoneList.appendChild(o);
    }
  }

  function candidateFor(name: string): ParamCandidate | undefined {
    return candidates.find((c) => c.name === name);
  }

  function labelsFor(kind: DistKind): [string, string, string?] {
    if (kind === "normal") return ["mean", "std dev"];
    if (kind === "uniform") return ["min", "max"];
    return ["min", "mode", "max"];
  }

  function renderMcRows(): void {
    mcList.innerHTML = "";
    if (mcRows.length === 0) {
      const empty = document.createElement("div");
      empty.className = "mc-empty";
      empty.textContent = "No uncertain parameters yet — add one to build a distribution.";
      mcList.appendChild(empty);
      return;
    }
    mcRows.forEach((row, i) => {
      const el = document.createElement("div");
      el.className = "mc-param-row";

      const paramSel = document.createElement("select");
      paramSel.className = "mc-param-sel";
      for (const c of candidates) {
        const o = document.createElement("option");
        o.value = c.name;
        o.textContent = c.unit ? `${c.name} [${c.unit}]` : c.name;
        if (c.name === row.param) o.selected = true;
        paramSel.appendChild(o);
      }
      paramSel.addEventListener("change", () => {
        row.param = paramSel.value;
        const c = candidateFor(row.param);
        [row.a, row.b, row.c] = distDefaults(c?.default ?? 0, row.kind, row.param);
        renderMcRows();
      });

      const kindSel = document.createElement("select");
      kindSel.className = "mc-dist-sel";
      for (const k of ["normal", "uniform", "triangular"] as DistKind[]) {
        const o = document.createElement("option");
        o.value = k;
        o.textContent = k;
        if (k === row.kind) o.selected = true;
        kindSel.appendChild(o);
      }
      kindSel.addEventListener("change", () => {
        row.kind = kindSel.value as DistKind;
        const c = candidateFor(row.param);
        [row.a, row.b, row.c] = distDefaults(c?.default ?? row.a, row.kind, row.param);
        renderMcRows();
      });

      const [la, lb, lc] = labelsFor(row.kind);
      const spark = distSparkline();
      const refreshSpark = (): void => spark.redraw(row.kind, row.a, row.b, row.c);
      const inA = numInput(row.a, (v) => ((row.a = v), refreshSpark()), la);
      const inB = numInput(row.b, (v) => ((row.b = v), refreshSpark()), lb);
      const fields = document.createElement("span");
      fields.className = "mc-dist-fields";
      fields.append(wrap(la, inA), wrap(lb, inB));
      if (lc) {
        const inC = numInput(row.c, (v) => ((row.c = v), refreshSpark()), lc);
        fields.append(wrap(lc, inC));
      }
      refreshSpark();

      const zoneInp = document.createElement("input");
      zoneInp.className = "mc-zone-inp";
      zoneInp.placeholder = "zone (all)";
      zoneInp.title = "Restrict this uncertainty to one named zone — blank applies well-wide";
      zoneInp.setAttribute("list", zoneListId);
      zoneInp.value = row.zone;
      zoneInp.addEventListener("focus", () => void refreshZoneOptions());
      zoneInp.addEventListener("change", () => (row.zone = zoneInp.value.trim()));

      const rm = mini("✕", () => {
        mcRows.splice(i, 1);
        renderMcRows();
      });
      rm.classList.add("mc-rm");

      el.append(paramSel, kindSel, seedTag(row.param), fields, spark.el, zoneInp, rm);
      const evidence = buildParamSources(candidateFor(row.param)?.sourcesTopic ?? "");
      evidence.classList.add("mc-param-evidence");
      el.appendChild(evidence);
      mcList.appendChild(el);
    });
    renderCorrRows(); // param renames/removals must reflect in the correlation editor
  }

  const mcHead = document.createElement("div");
  mcHead.className = "mc-params-head";
  mcHead.append(addParamBtn);
  content.appendChild(
    formRow(
      "Uncertainty",
      mcHead,
      "Each row is one distribution — well-wide, or scoped to a named zone via the zone box. Parameters you don't vary follow their zone values.",
    ),
  );
  content.appendChild(mcList);

  // --- Parameter correlations (Iman–Conover rank induction) ------------------
  let corrRows: CorrRow[] = [];
  const corrList = document.createElement("div");
  corrList.className = "mc-corr-list";

  function uniqueParamNames(): string[] {
    return [...new Set(mcRows.map((r) => r.param))];
  }

  const addCorrBtn = mini("+ Add correlation", () => {
    const names = uniqueParamNames();
    if (names.length < 2) {
      setStatus("Add at least two uncertain parameters before correlating them");
      return;
    }
    corrRows.push({ a: names[0], b: names[1], rho: 0.7 });
    renderCorrRows();
  });

  function renderCorrRows(): void {
    corrList.innerHTML = "";
    if (corrRows.length === 0) return;
    const names = uniqueParamNames();
    corrRows.forEach((row, i) => {
      // Drop rows whose parameters vanished from the study.
      if (!names.includes(row.a) || !names.includes(row.b)) {
        corrRows.splice(i, 1);
        renderCorrRows();
        return;
      }
      const el = document.createElement("div");
      el.className = "mc-corr-row";
      const mkSel = (value: string, onChange: (v: string) => void): HTMLSelectElement => {
        const sel = document.createElement("select");
        for (const nm of names) {
          const o = document.createElement("option");
          o.value = nm;
          o.textContent = nm;
          if (nm === value) o.selected = true;
          sel.appendChild(o);
        }
        sel.addEventListener("change", () => onChange(sel.value));
        return sel;
      };
      const selA = mkSel(row.a, (v) => (row.a = v));
      const selB = mkSel(row.b, (v) => (row.b = v));
      const link = document.createElement("span");
      link.className = "mc-corr-link";
      link.textContent = "↔";
      const rhoInp = numInput(row.rho, (v) => (row.rho = Math.max(-0.99, Math.min(0.99, v))), "target Spearman rank correlation (−0.99…0.99)");
      rhoInp.classList.add("mc-corr-rho");
      rhoInp.step = "0.05";
      rhoInp.min = "-0.99";
      rhoInp.max = "0.99";
      const rhoLbl = document.createElement("span");
      rhoLbl.className = "mc-corr-rho-lbl";
      rhoLbl.textContent = "ρ";
      const rm = mini("✕", () => {
        corrRows.splice(i, 1);
        renderCorrRows();
      });
      rm.classList.add("mc-rm");
      el.append(selA, link, selB, rhoLbl, rhoInp, rm);
      corrList.appendChild(el);
    });
  }

  const corrHead = document.createElement("div");
  corrHead.className = "mc-params-head";
  corrHead.append(addCorrBtn);
  content.appendChild(
    formRow(
      "Correlations",
      corrHead,
      "Optional rank correlations between uncertain parameters (e.g. RHO_MA with GR_MA). Draws are reordered to hit the target ρ — the distributions themselves never change.",
    ),
  );
  content.appendChild(corrList);

  // --- Wells (scope, not a checklist) --------------------------------------
  content.appendChild(scope.el);

  // --- Settings (cutoffs + run controls) -----------------------------------
  // Seed the cutoffs from the ONE shared source (saved project defaults → canonical fallback), so
  // an MC net-pay uses exactly the same cutoffs as the deterministic pay summary — the tooltip below
  // says "Cutoffs match the pay summary", and now that is structurally true rather than a stale claim
  // (MC previously hard-coded PHIE ≥ 0.08 / SWE ≤ 0.5 against the summary's 0.1 / 0.6).
  const cuts = await loadCutoffDefaults();
  const iters = numField("Iterations", 1000, 1, 100000);
  const seed = numField("Seed", 42, 0, 1e9);
  const bins = numField("HPV bins", 12, 3, 60);
  const pctlSel = percentileField();
  const vshMax = numField("VSH ≤", cuts.vsh_max, 0, 1);
  const phieMin = numField("PHIE ≥", cuts.phie_min, 0, 1);
  const sweMax = numField("SWE ≤", cuts.swe_max, 0, 1);
  vshMax.el.appendChild(buildParamSources(PARAM_SOURCE_TOPICS.cutoffVshMax));
  phieMin.el.appendChild(buildParamSources(PARAM_SOURCE_TOPICS.cutoffPhieMin));
  sweMax.el.appendChild(buildParamSources(PARAM_SOURCE_TOPICS.cutoffSweMax));
  const permMin = numField("PERM ≥ (blank=off)", cuts.perm_min ?? NaN, 0, 1e6);
  const sampSel = document.createElement("select");
  for (const [v, label] of [
    ["lhs", "Latin Hypercube"],
    ["random", "Random (legacy)"],
  ] as const) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    sampSel.appendChild(o);
  }
  const sampField = wrap("Sampling", sampSel);
  const grid = document.createElement("div");
  grid.className = "mc-settings";
  for (const f of [iters, seed, bins, pctlSel, vshMax, phieMin, sweMax, permMin]) grid.appendChild(f.el);
  grid.appendChild(sampField);
  content.appendChild(
    formRow(
      "Settings",
      grid,
      "Cutoffs match the pay summary; PERM cutoff applies only where a PERM curve exists. The percentile pair drives both the reported spread and the tornado sweep. Latin Hypercube reaches stable percentiles with fewer iterations; Random reproduces pre-upgrade results at the same seed.",
    ),
  );

  // --- Sensitivity + run options -------------------------------------------
  const sensChk = checkbox("Rank sensitivity (Spearman)", true);
  const tornChk = checkbox("Tornado sweep (P10 / P90)", true);
  const convChk = checkbox("Convergence check", false);
  const persistChk = checkbox("Save LOW/BASE/HIGH curves", false);
  const realChk = checkbox("Store realizations (array log)", false);
  const sensBox = document.createElement("div");
  sensBox.className = "mc-sens-opts";
  sensBox.append(sensChk.el, tornChk.el, convChk.el, persistChk.el, realChk.el);
  // Storing realizations rides the same pass over the kept runs as the percentile curves, so
  // it needs them switched on; ticking it alone would silently do nothing.
  const syncReal = (): void => {
    realChk.el.querySelector("input")?.toggleAttribute("disabled", !persistChk.checked());
    realChk.el.style.opacity = persistChk.checked() ? "" : "0.5";
  };
  persistChk.el.addEventListener("change", syncReal);
  syncReal();
  content.appendChild(
    formRow(
      "Options",
      sensBox,
      "Spearman ranks each parameter's pull; the tornado sweeps each to its P10/P90 with the rest at medians. Convergence tracks the running percentiles per batch (Random sampling stops early once stationary). Saving curves writes MC_*_LOW/_P50/_HIGH/_BASE to a fresh version of the MONTECARLO log set. Storing realizations additionally keeps the run's realization matrix as MC_*_REAL, so a log view can draw an adjustable band, spaghetti or a density heat map from it without re-running the study.",
    ),
  );

  // --- Run ------------------------------------------------------------------
  const runBtn = button("Run Monte Carlo");
  runBtn.className = "form-run-btn mc-run-btn";
  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  runRow.append(runBtn, statusLine);
  content.appendChild(runRow);

  // --- Results: histogram (top), tornado (middle), TABLE (bottom, per Jauhar) ----
  const histHost = document.createElement("div");
  histHost.className = "mc-results mc-hist-host";
  content.appendChild(histHost);

  const sensHost = document.createElement("div");
  sensHost.className = "mc-sens";
  content.appendChild(sensHost);

  const convHost = document.createElement("div");
  convHost.className = "mc-conv";
  content.appendChild(convHost);

  const notesHost = document.createElement("div");
  notesHost.className = "mc-notes";
  content.appendChild(notesHost);

  const tableHost = document.createElement("div");
  tableHost.className = "mc-table-host";
  content.appendChild(tableHost);

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    const mcParams: McParam[] = mcRows.map((r) => ({ param: r.param, dist: toDist(r), zone: r.zone ? r.zone : null }));
    const correlations: McCorrelation[] = corrRows
      .filter((r) => r.a && r.b && r.a !== r.b && Number.isFinite(r.rho) && r.rho !== 0)
      .map((r) => ({ param_a: r.a, param_b: r.b, rho: r.rho }));
    const pm = permMin.value();
    const [loP, hiP] = pctlSel.value();
    const persist = persistChk.checked();
    const custody = persist ? await requestRunCustody("Persist Monte Carlo curves") : null;
    if (persist && !custody) return;
    const req: McRequest = {
      well_ids: wellIds,
      steps,
      mc_params: mcParams,
      iterations: Math.round(iters.value()),
      seed: Math.round(seed.value()),
      vsh_max: vshMax.value(),
      phie_min: phieMin.value(),
      swe_max: sweMax.value(),
      perm_min: Number.isFinite(pm) ? pm : null,
      bins: Math.round(bins.value()),
      low_pctl: loP,
      high_pctl: hiP,
      sensitivity: sensChk.checked(),
      tornado: tornChk.checked(),
      sampling: sampSel.value as "lhs" | "random",
      correlations,
      converge: convChk.checked(),
      persist,
      persist_realizations: persist && realChk.checked(),
      custody,
    };
    runBtn.disabled = true;
    statusLine.textContent = `Running ${req.iterations.toLocaleString()} realizations × ${wellIds.length} well(s)…`;
    const t0 = performance.now();
    try {
      const res = await runMonteCarlo(req, scope.backend());
      const ms = Math.round(performance.now() - t0);
      const used = res.zones[0]?.iterations ?? req.iterations;
      const extras = [
        res.sampling === "lhs" ? "LHS" : "random",
        used !== req.iterations ? `stopped at ${used.toLocaleString()}` : "",
        res.persisted.length ? `${res.persisted.length} curves saved` : "",
      ]
        .filter(Boolean)
        .join(" · ");
      statusLine.textContent = `Done in ${ms} ms · ${res.zones.length} well-zone results · ${extras}`;
      renderResults(histHost, tableHost, res, histOpts);
      renderSensitivity(sensHost, res, used, tornOpts);
      renderConvergence(convHost, res);
      renderNotes(notesHost, res);
      setStatus(`Monte Carlo: ${used} realizations across ${wellIds.length} well(s) in ${ms} ms`);
      recordProcess("Monte Carlo", `${used} realizations across ${wellIds.length} well(s) → ${res.zones.length} zone results`);
      if (res.errors.length) console.warn("Monte Carlo warnings:", res.errors);
    } catch (e) {
      statusLine.textContent = `Failed: ${e}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  describeSteps();
  refreshCandidates();
  renderMcRows();
  return {
    el: content,
    dispose: () => {
      scope.dispose();
      disconnectCanvasObservers();
    },
  };
}

// A live percentile-pair picker (drives the reported spread AND the tornado input sweep).
const PCTL_PAIRS: { label: string; lo: number; hi: number }[] = [
  { label: "P10 / P90", lo: 0.1, hi: 0.9 },
  { label: "P25 / P75", lo: 0.25, hi: 0.75 },
  { label: "P5 / P95", lo: 0.05, hi: 0.95 },
  { label: "P1 / P99", lo: 0.01, hi: 0.99 },
];
function percentileField(): { el: HTMLElement; value: () => [number, number] } {
  const sel = document.createElement("select");
  for (const p of PCTL_PAIRS) {
    const o = document.createElement("option");
    o.value = `${p.lo}:${p.hi}`;
    o.textContent = p.label;
    sel.appendChild(o);
  }
  const el = wrap("Percentiles", sel);
  return {
    el,
    value: () => {
      const [lo, hi] = sel.value.split(":").map(Number);
      return [lo, hi];
    },
  };
}

/** Percentile fraction → "P10"/"P90"/"P5" style label. */
function pctLabel(p: number): string {
  return `P${Math.round(p * 100)}`;
}

// Canvas resize/redraw management — the histogram and tornado canvases are CSS-100%-wide, so a
// pane resize must re-rasterize them at the new width (otherwise the browser scales a stale
// bitmap → the blurry "stretch" the user reported). One ResizeObserver per live canvas.
const canvasObservers: ResizeObserver[] = [];
function disconnectCanvasObservers(): void {
  for (const ro of canvasObservers) ro.disconnect();
  canvasObservers.length = 0;
}
function observeCanvas(canvas: HTMLCanvasElement, draw: () => void): void {
  if (typeof ResizeObserver === "undefined") return;
  let w = -1;
  const ro = new ResizeObserver(() => {
    const cw = Math.round(canvas.clientWidth);
    if (cw > 0 && cw !== w) {
      w = cw;
      draw();
    }
  });
  ro.observe(canvas);
  canvasObservers.push(ro);
}

function toDist(r: McRow): McDistribution {
  if (r.kind === "normal") return { kind: "normal", mean: r.a, sd: r.b };
  if (r.kind === "uniform") return { kind: "uniform", lo: r.a, hi: r.b };
  return { kind: "triangular", lo: r.a, mode: r.b, hi: r.c };
}

// --- Results rendering ------------------------------------------------------

interface PctLabels {
  lo: string;
  mid: string;
  hi: string;
}

function fmt(v: number, dp = 2): string {
  return Number.isFinite(v) ? v.toFixed(dp) : "—";
}

/** A right-aligned numeric cell: the P50 as the headline number, with the (lo–hi) band as a
 *  quieter sub-line when `range` is on. Keeps the table scannable instead of cramming three
 *  numbers into every cell. */
function numCell(p: Pctl, dp: number, range: boolean): HTMLTableCellElement {
  const td = document.createElement("td");
  const mid = document.createElement("span");
  mid.className = "mc-cell-mid";
  mid.textContent = fmt(p.mid, dp);
  td.appendChild(mid);
  if (range) {
    const rng = document.createElement("span");
    rng.className = "mc-cell-range";
    rng.textContent = `${fmt(p.lo, dp)} – ${fmt(p.hi, dp)}`;
    td.appendChild(rng);
  }
  return td;
}

function renderResults(histHost: HTMLElement, tableHost: HTMLElement, res: McResult, histOpts: HistOptions): void {
  disconnectCanvasObservers();
  histHost.innerHTML = "";
  tableHost.innerHTML = "";
  if (res.zones.length === 0) {
    histHost.textContent = "No results (no curve data or no zones matched).";
    return;
  }

  const labels: PctLabels = { lo: pctLabel(res.low_pctl), mid: "P50", hi: pctLabel(res.high_pctl) };
  const band = `${labels.lo}–${labels.hi}`;

  // --- Histogram (top) ------------------------------------------------------
  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";
  canvas.style.height = `${histOpts.height}px`;
  let shownZone = res.zones[0];
  const drawHist = () => drawHistogram(canvas, shownZone, labels, histOpts);
  const applyHist = () => {
    canvas.style.height = `${histOpts.height}px`;
    drawHist();
  };

  const histWrap = document.createElement("div");
  histWrap.className = "mc-hist-wrap";
  const histHead = document.createElement("div");
  histHead.className = "mc-plot-head";
  const caption = document.createElement("div");
  caption.className = "mc-hist-caption";
  caption.textContent = `HPV distribution (click a table row) — bars with ${labels.lo} / ${labels.mid} / ${labels.hi} markers`;
  const histGear = gearButton();
  histHead.append(caption, histGear);
  const histProps = buildHistProps(histOpts, applyHist);
  histGear.addEventListener("click", () => (histProps.hidden = !histProps.hidden));
  histWrap.append(histHead, histProps, canvas);
  histHost.appendChild(histWrap);

  // --- Table (bottom) -------------------------------------------------------
  const table = document.createElement("table");
  table.className = "mc-table";
  const thead = document.createElement("thead");
  const head = document.createElement("tr");
  const cols: { label: string; sub?: string }[] = [
    { label: "Well" },
    { label: "Zone" },
    { label: "Gross" },
    { label: "Net pay", sub: band },
    { label: "NTG", sub: "P50" },
    { label: "Avg PHIE", sub: "P50" },
    { label: "Avg SWE", sub: "P50" },
    { label: "HPV", sub: band },
  ];
  for (const c of cols) {
    const th = document.createElement("th");
    th.textContent = c.label;
    if (c.sub) {
      const s = document.createElement("span");
      s.className = "mc-th-sub";
      s.textContent = c.sub;
      th.appendChild(s);
    }
    head.appendChild(th);
  }
  thead.appendChild(head);
  table.appendChild(thead);

  const tbody = document.createElement("tbody");
  res.zones.forEach((z, i) => {
    const tr = document.createElement("tr");
    tr.className = "mc-row-click";
    const wellTd = document.createElement("td");
    wellTd.textContent = z.well_name;
    const zoneTd = document.createElement("td");
    zoneTd.textContent = z.zone;
    const grossTd = document.createElement("td");
    grossTd.className = "mc-cell-num";
    grossTd.textContent = fmt(z.gross, 1);
    tr.append(wellTd, zoneTd, grossTd);
    tr.append(
      numCell(z.net, 1, true),
      numCell(z.ntg, 3, false),
      numCell(z.avg_phie, 3, false),
      numCell(z.avg_swe, 3, false),
      numCell(z.hpv, 2, true),
    );
    tr.addEventListener("click", () => {
      for (const r of table.querySelectorAll(".mc-row-sel")) r.classList.remove("mc-row-sel");
      tr.classList.add("mc-row-sel");
      shownZone = z;
      drawHist();
    });
    if (i === 0) tr.classList.add("mc-row-sel");
    tbody.appendChild(tr);
  });
  table.appendChild(tbody);

  const tableCap = document.createElement("div");
  tableCap.className = "mc-table-caption";
  tableCap.textContent = "Per well-zone volumes — click a row to plot its HPV distribution above";
  const tableWrap = document.createElement("div");
  tableWrap.className = "mc-table-wrap";
  tableWrap.appendChild(table);
  tableHost.append(tableCap, tableWrap);

  applyHist();
  observeCanvas(canvas, drawHist);
}

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
}

/** A CSS colour string coerced to a 6-digit `#rrggbb` (what <input type=color> needs); falls back
 *  when the value is a name/rgb()/var that a colour input can't seed from. */
function toHexColor(v: string, fallback: string): string {
  const s = v.trim();
  if (/^#[0-9a-fA-F]{6}$/.test(s)) return s.toLowerCase();
  if (/^#[0-9a-fA-F]{3}$/.test(s)) return `#${s[1]}${s[1]}${s[2]}${s[2]}${s[3]}${s[3]}`.toLowerCase();
  return fallback;
}

/** "Nice" round step (1/2/5 × 10ⁿ) so ~`target` frequency gridlines land on tidy counts. */
function niceStep(max: number, target = 4): number {
  const raw = max / Math.max(1, target) || 1;
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const norm = raw / mag;
  const step = (norm >= 5 ? 5 : norm >= 2 ? 2 : 1) * mag;
  return step > 0 ? step : 1;
}

function drawHistogram(canvas: HTMLCanvasElement, z: McZoneResult, labels: PctLabels, opts: HistOptions): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth || 600;
  const H = canvas.clientHeight || 200;
  canvas.width = W * dpr;
  canvas.height = H * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);

  const barCol = opts.barColor || cssVar("--accent");
  const accent2 = cssVar("--accent2");
  const text = cssVar("--text-dim");
  const border = cssVar("--border");

  const binVals = z.hpv_hist;
  const maxCount = Math.max(1, ...binVals);
  const step = niceStep(maxCount, 4);
  const yMax = step * Math.ceil(maxCount / step); // tidy ceiling ≥ tallest bar

  const pad = { l: opts.showYAxis ? 40 : 10, r: 12, t: 12, b: 30 };
  const plotW = W - pad.l - pad.r;
  const plotH = H - pad.t - pad.b;
  ctx.font = canvasFont(readTheme(canvas), 10);

  // Frequency gridlines + y-axis tick labels (what makes it read as a real histogram).
  if (opts.showGrid || opts.showYAxis) {
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    for (let t = 0; t <= yMax + 1e-9; t += step) {
      const y = pad.t + plotH - (t / yMax) * plotH;
      if (opts.showGrid) {
        ctx.strokeStyle = border;
        ctx.globalAlpha = 0.4;
        ctx.beginPath();
        ctx.moveTo(pad.l, y + 0.5);
        ctx.lineTo(pad.l + plotW, y + 0.5);
        ctx.stroke();
        ctx.globalAlpha = 1;
      }
      if (opts.showYAxis) {
        ctx.fillStyle = text;
        ctx.fillText(String(Math.round(t)), pad.l - 6, y);
      }
    }
    if (opts.showYAxis) {
      ctx.save();
      ctx.translate(11, pad.t + plotH / 2);
      ctx.rotate(-Math.PI / 2);
      ctx.fillStyle = text;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText("count", 0, 0);
      ctx.restore();
    }
  }

  // Bars.
  const bw = plotW / Math.max(1, binVals.length);
  binVals.forEach((c, i) => {
    const h = (c / yMax) * plotH;
    ctx.fillStyle = barCol;
    ctx.fillRect(pad.l + i * bw + 1, pad.t + plotH - h, Math.max(1, bw - 2), h);
  });

  // Baseline + x tick labels across the HPV range.
  ctx.strokeStyle = border;
  ctx.globalAlpha = 1;
  ctx.beginPath();
  ctx.moveTo(pad.l, pad.t + plotH + 0.5);
  ctx.lineTo(pad.l + plotW, pad.t + plotH + 0.5);
  ctx.stroke();

  const hiVal = z.hist_lo + z.hist_w * binVals.length;
  const midVal = (z.hist_lo + hiVal) / 2;
  ctx.fillStyle = text;
  ctx.textBaseline = "top";
  ctx.textAlign = "left";
  ctx.fillText(z.hist_lo.toFixed(1), pad.l, pad.t + plotH + 5);
  ctx.textAlign = "center";
  ctx.fillText(midVal.toFixed(1), pad.l + plotW / 2, pad.t + plotH + 5);
  ctx.textAlign = "right";
  ctx.fillText(hiVal.toFixed(1), pad.l + plotW, pad.t + plotH + 5);
  ctx.textAlign = "center";
  ctx.fillText("HPV", pad.l + plotW / 2, pad.t + plotH + 17);

  // P10/P50/P90 markers.
  if (opts.showMarkers) {
    const xFor = (v: number): number => {
      const span = z.hist_w * binVals.length || 1;
      return pad.l + ((v - z.hist_lo) / span) * plotW;
    };
    const marks: [number, string][] = [
      [z.hpv.lo, labels.lo],
      [z.hpv.mid, labels.mid],
      [z.hpv.hi, labels.hi],
    ];
    ctx.textBaseline = "top";
    for (const [v, lbl] of marks) {
      if (!Number.isFinite(v)) continue;
      const x = Math.max(pad.l, Math.min(pad.l + plotW, xFor(v)));
      ctx.strokeStyle = accent2;
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(x, pad.t);
      ctx.lineTo(x, pad.t + plotH);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = accent2;
      ctx.textAlign = "center";
      ctx.fillText(lbl, x, pad.t - 1);
    }
  }
}

// --- Sensitivity / tornado rendering ---------------------------------------

type MetricKey = "hpv" | "net" | "ntg" | "avg_phie" | "avg_swe";
const METRICS: { key: MetricKey; label: string; dp: number }[] = [
  { key: "hpv", label: "HPV", dp: 2 },
  { key: "net", label: "Net pay", dp: 1 },
  { key: "ntg", label: "NTG", dp: 3 },
  { key: "avg_phie", label: "Avg PHIE", dp: 3 },
  { key: "avg_swe", label: "Avg SWE", dp: 3 },
];

/** Null-safe metric accessor: the backend serializes NaN as JSON null (no-pay sweep points,
 *  no-spread Spearman), so normalize null → NaN here — downstream `Math.min/max` on a null
 *  would coerce it to 0 and fabricate a bar endpoint. */
function mval(m: McMetricSet | null, k: MetricKey): number {
  const v = m ? m[k] : null;
  return v === null ? NaN : v;
}

/** Below this fraction of the strongest mover's swing, a parameter is treated as not affecting
 *  the metric and hidden. The OAT sweep is deterministic, so a truly non-contributing parameter
 *  (e.g. Rw for PHIE) has a swing of ~0 and is dropped — killing the finite-N noise the user saw. */
const GATE_REL = 0.005;

function renderSensitivity(host: HTMLElement, res: McResult, iterations: number, tornOpts: TornOptions): void {
  host.innerHTML = "";
  const zones = (res.sensitivity ?? []).filter((z) => z.params.length > 0);
  if (zones.length === 0) return;

  const title = document.createElement("div");
  title.className = "mc-sens-title";
  title.textContent = "Parameter sensitivity";

  const controls = document.createElement("div");
  controls.className = "mc-sens-controls";
  const zoneSel = document.createElement("select");
  zones.forEach((z, i) => {
    const o = document.createElement("option");
    o.value = String(i);
    o.textContent = `${z.well_name} · ${z.zone}`;
    zoneSel.appendChild(o);
  });
  const metricSel = document.createElement("select");
  for (const m of METRICS) {
    const o = document.createElement("option");
    o.value = m.key;
    o.textContent = m.label;
    metricSel.appendChild(o);
  }
  const gear = gearButton();
  controls.append(wrap("Zone", zoneSel), wrap("Metric", metricSel), gear);

  const caption = document.createElement("div");
  caption.className = "mc-sens-caption";
  const canvas = document.createElement("canvas");
  canvas.className = "mc-tornado";
  // Height tracks the parameter count so bars are neither squished nor sparse (unless the user
  // pins an explicit height in the ⚙ panel).
  const maxParams = Math.max(...zones.map((z) => z.params.length));
  const autoH = Math.min(360, Math.max(150, 34 + maxParams * 30));
  const applyH = () => {
    canvas.style.height = `${tornOpts.height > 0 ? tornOpts.height : autoH}px`;
  };

  const props = buildTornProps(tornOpts, () => redraw());
  gear.addEventListener("click", () => (props.hidden = !props.hidden));

  host.append(title, controls, props, caption, canvas);

  const currentZone = (): McSensZone => zones[parseInt(zoneSel.value, 10)] ?? zones[0];
  const redraw = (): void => {
    applyH();
    drawTornado(canvas, caption, currentZone(), metricSel.value as MetricKey, iterations, tornOpts);
  };
  zoneSel.addEventListener("change", redraw);
  metricSel.addEventListener("change", redraw);
  applyH();
  redraw();
  observeCanvas(canvas, redraw);
}

/** Rounded horizontal bar (falls back to a plain rect on older webviews). */
function bar(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number): void {
  const r = Math.min(3, Math.abs(w) / 2, h / 2);
  if (typeof ctx.roundRect === "function") {
    ctx.beginPath();
    ctx.roundRect(x, y, w, h, r);
    ctx.fill();
  } else {
    ctx.fillRect(x, y, w, h);
  }
}

/** Horizontal tornado: OAT range bars around a common base when a sweep ran, otherwise signed
 *  Spearman bars. Parameters that don't move the selected metric are hidden (causal gating on
 *  the deterministic OAT swing); ρ annotations show only when statistically significant. */
function drawTornado(
  canvas: HTMLCanvasElement,
  caption: HTMLElement,
  z: McSensZone,
  metric: MetricKey,
  n: number,
  opts: TornOptions,
): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth || 600;
  const H = canvas.clientHeight || 220;
  canvas.width = W * dpr;
  canvas.height = H * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);

  const loCol = opts.loColor || cssVar("--accent");
  const hiCol = opts.hiColor || cssVar("--accent2");
  const text = cssVar("--text-dim");
  const border = cssVar("--border");
  const rowBg = cssVar("--bg-panel-alt");
  ctx.font = canvasFont(readTheme(canvas), 11);

  const meta = METRICS.find((m) => m.key === metric);
  const label = meta?.label ?? metric;
  const dp = meta?.dp ?? 2;
  const pad = { l: 98, r: 60, t: 12, b: 24 };
  const plotW = W - pad.l - pad.r;
  const plotH = H - pad.t - pad.b;
  // 95% two-sided significance floor for a rank correlation at this sample size.
  const sig = 1.96 / Math.sqrt(Math.max(1, n));
  const hasOat = z.params.some((p: McSensParam) => p.oat_base !== null);

  const noneNote = (msg: string): void => {
    caption.textContent = msg;
    ctx.fillStyle = text;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("— no parameter affects this metric —", pad.l + plotW / 2, pad.t + plotH / 2);
  };

  const rowStripe = (i: number, y: number, rowH: number): void => {
    if (opts.showStripes && i % 2 === 1) {
      ctx.fillStyle = rowBg;
      ctx.globalAlpha = 0.5;
      ctx.fillRect(pad.l - 4, y - rowH / 2, plotW + 8, rowH);
      ctx.globalAlpha = 1;
    }
  };

  if (hasOat) {
    const base = mval(z.params[0].oat_base, metric);
    if (!Number.isFinite(base)) {
      // Dry base case (e.g. no pay at the parameter medians → avg PHIE/SWE is null): there is
      // no median-case anchor to split the bars around, so say that instead of crashing on it.
      noneNote(`Tornado — the base case yields no ${label} (no pay at the parameter medians), so there is no anchor for the bars.`);
      return;
    }
    let bars = z.params
      .map((p) => {
        const a = mval(p.oat_low, metric);
        const b = mval(p.oat_high, metric);
        return { name: p.param, lo: Math.min(a, b), hi: Math.max(a, b), corr: mval(p.spearman, metric) };
      })
      .filter((b) => Number.isFinite(b.lo) && Number.isFinite(b.hi));
    const maxSwing = bars.reduce((m, b) => Math.max(m, b.hi - b.lo), 0);
    // Causal gate: drop parameters whose one-at-a-time sweep barely moves this metric.
    const gate = Math.max(1e-9, GATE_REL * maxSwing);
    bars = bars.filter((b) => b.hi - b.lo > gate).sort((a, b) => b.hi - b.lo - (a.hi - a.lo));
    if (bars.length === 0) {
      noneNote(`Tornado — no parameter moves ${label} (every held-out sweep leaves it unchanged).`);
      return;
    }
    const rowH = Math.min(30, plotH / bars.length);

    let mn = base;
    let mx = base;
    for (const b of bars) {
      mn = Math.min(mn, b.lo);
      mx = Math.max(mx, b.hi);
    }
    if (mx <= mn) mx = mn + 1;
    const m = 0.06 * (mx - mn);
    mn -= m;
    mx += m;
    const xFor = (v: number): number => pad.l + ((v - mn) / (mx - mn)) * plotW;
    const bx = xFor(base);

    bars.forEach((_, i) => rowStripe(i, pad.t + i * rowH + rowH / 2, rowH));

    ctx.strokeStyle = border;
    ctx.setLineDash([4, 3]);
    ctx.beginPath();
    ctx.moveTo(bx, pad.t);
    ctx.lineTo(bx, pad.t + plotH);
    ctx.stroke();
    ctx.setLineDash([]);

    ctx.textBaseline = "middle";
    bars.forEach((b, i) => {
      const y = pad.t + i * rowH + rowH / 2;
      const x0 = xFor(b.lo);
      const x1 = xFor(b.hi);
      const bh = rowH * 0.6;
      // Split the bar at the base: [lo, base] one colour, [base, hi] the other. Robust when the
      // metric is non-monotonic (both endpoints on the same side of base → one segment is empty).
      const lStart = Math.min(x0, bx);
      const lW = bx - lStart;
      const rW = Math.max(x1, bx) - bx;
      ctx.fillStyle = loCol;
      if (lW > 0.5) bar(ctx, lStart, y - bh / 2, lW, bh);
      ctx.fillStyle = hiCol;
      if (rW > 0.5) bar(ctx, bx, y - bh / 2, rW, bh);
      ctx.fillStyle = text;
      ctx.textAlign = "right";
      ctx.fillText(b.name, pad.l - 8, y);
      if (opts.showRho && Number.isFinite(b.corr) && Math.abs(b.corr) >= sig) {
        ctx.textAlign = "left";
        ctx.fillText(`ρ ${b.corr >= 0 ? "+" : ""}${b.corr.toFixed(2)}`, pad.l + plotW + 6, y);
      }
    });

    ctx.fillStyle = text;
    ctx.textBaseline = "top";
    ctx.textAlign = "center";
    ctx.fillText(`base ${base.toFixed(dp)}`, bx, pad.t + plotH + 4);
    ctx.textAlign = "left";
    ctx.fillText(mn.toFixed(dp), pad.l, pad.t + plotH + 4);
    ctx.textAlign = "right";
    ctx.fillText(mx.toFixed(dp), pad.l + plotW, pad.t + plotH + 4);
    caption.textContent = `Tornado — ${label} swing as each parameter moves across its uncertainty range (others held at median). Parameters with no effect are hidden; ρ shown where significant.`;
  } else {
    let bars = z.params
      .map((p) => ({ name: p.param, corr: mval(p.spearman, metric) }))
      // Significance gate: hide correlations indistinguishable from noise at this sample size.
      .filter((b) => Number.isFinite(b.corr) && Math.abs(b.corr) >= sig);
    bars = bars.sort((a, b) => Math.abs(b.corr) - Math.abs(a.corr));
    if (bars.length === 0) {
      noneNote(`Rank sensitivity — no parameter correlates with ${label} above the noise floor (|ρ| ≥ ${sig.toFixed(2)}).`);
      return;
    }
    const rowH = Math.min(30, plotH / bars.length);
    const xFor = (v: number): number => pad.l + ((v + 1) / 2) * plotW;
    const zx = xFor(0);

    bars.forEach((_, i) => rowStripe(i, pad.t + i * rowH + rowH / 2, rowH));

    ctx.strokeStyle = border;
    ctx.beginPath();
    ctx.moveTo(zx, pad.t);
    ctx.lineTo(zx, pad.t + plotH);
    ctx.stroke();

    ctx.textBaseline = "middle";
    bars.forEach((b, i) => {
      const y = pad.t + i * rowH + rowH / 2;
      const x = xFor(b.corr);
      const bh = rowH * 0.6;
      ctx.fillStyle = b.corr >= 0 ? hiCol : loCol;
      bar(ctx, Math.min(zx, x), y - bh / 2, Math.max(1, Math.abs(x - zx)), bh);
      ctx.fillStyle = text;
      ctx.textAlign = "right";
      ctx.fillText(b.name, pad.l - 8, y);
      if (opts.showRho) {
        ctx.textAlign = b.corr >= 0 ? "left" : "right";
        ctx.fillText(`${b.corr >= 0 ? "+" : ""}${b.corr.toFixed(2)}`, x + (b.corr >= 0 ? 4 : -4), y);
      }
    });

    ctx.fillStyle = text;
    ctx.textBaseline = "top";
    ctx.textAlign = "left";
    ctx.fillText("−1", pad.l, pad.t + plotH + 4);
    ctx.textAlign = "center";
    ctx.fillText("0", zx, pad.t + plotH + 4);
    ctx.textAlign = "right";
    ctx.fillText("+1", pad.l + plotW, pad.t + plotH + 4);
    caption.textContent = `Rank sensitivity — Spearman correlation of each parameter with ${label}. Correlations below the noise floor (|ρ| < ${sig.toFixed(2)}) are hidden.`;
  }
}

// --- Convergence + notes rendering -----------------------------------------

/** Per-well convergence traces: a caption (converged / stopped early / advisory) plus a
 *  sparkline of the running P-low / P50 / P-high of total HPV per checkpoint. */
function renderConvergence(host: HTMLElement, res: McResult): void {
  host.innerHTML = "";
  const traces = res.convergence ?? [];
  if (traces.length === 0) return;
  const title = document.createElement("div");
  title.className = "mc-sens-title";
  title.textContent = "Convergence — running percentiles of total HPV";
  host.appendChild(title);
  for (const c of traces) {
    const box = document.createElement("div");
    box.className = "mc-conv-well";
    const cap = document.createElement("div");
    cap.className = c.converged ? "mc-conv-caption" : "mc-conv-caption mc-conv-warn";
    const state = c.converged ? "converged" : "NOT converged";
    cap.textContent =
      `${c.well_name}: ${state} — ${c.used_iterations.toLocaleString()} of ` +
      `${c.requested_iterations.toLocaleString()} realizations` +
      (c.note ? ` · ${c.note}` : "");
    const canvas = document.createElement("canvas");
    canvas.className = "mc-conv-spark";
    box.append(cap, canvas);
    host.appendChild(box);
    const draw = () => drawConvSpark(canvas, c);
    draw();
    observeCanvas(canvas, draw);
  }
}

/** Three thin polylines (lo/mid/hi) over the checkpoint series; flat lines = stationary. */
function drawConvSpark(canvas: HTMLCanvasElement, c: McConvergence): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth || 600;
  const H = canvas.clientHeight || 56;
  canvas.width = W * dpr;
  canvas.height = H * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);

  const accent = cssVar("--accent");
  const accent2 = cssVar("--accent2");
  const text = cssVar("--text-dim");
  // Checkpoints can be null-valued for a dry well (NaN → JSON null) — keep only fully
  // finite ones rather than coercing.
  const pts = c.checks.filter((k) => Number.isFinite(k.lo) && Number.isFinite(k.mid) && Number.isFinite(k.hi));
  if (pts.length < 2) {
    ctx.fillStyle = text;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.font = canvasFont(readTheme(canvas), 10);
    ctx.fillText("— not enough checkpoints to plot —", W / 2, H / 2);
    return;
  }
  let mn = Infinity;
  let mx = -Infinity;
  for (const k of pts) {
    mn = Math.min(mn, k.lo);
    mx = Math.max(mx, k.hi);
  }
  if (mx <= mn) {
    mx = mn + 1;
  }
  const m = 0.08 * (mx - mn);
  mn -= m;
  mx += m;
  const pad = { l: 4, r: 44, t: 4, b: 4 };
  const lastAt = pts[pts.length - 1].at;
  const xFor = (at: number): number => pad.l + (at / lastAt) * (W - pad.l - pad.r);
  const yFor = (v: number): number => pad.t + (1 - (v - mn) / (mx - mn)) * (H - pad.t - pad.b);
  const line = (get: (k: (typeof pts)[number]) => number, color: string, width: number): void => {
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    pts.forEach((k, i) => {
      const x = xFor(k.at);
      const y = yFor(get(k));
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();
  };
  ctx.globalAlpha = 0.7;
  line((k) => k.lo, accent2, 1);
  line((k) => k.hi, accent2, 1);
  ctx.globalAlpha = 1;
  line((k) => k.mid, accent, 1.5);
  const last = pts[pts.length - 1];
  ctx.fillStyle = text;
  ctx.font = canvasFont(readTheme(canvas), 9);
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillText(last.mid.toFixed(2), W - pad.r + 4, yFor(last.mid));
}

/** Backend advisories (skipped correlation pairs, persist confirmations) + errors. */
function renderNotes(host: HTMLElement, res: McResult): void {
  host.innerHTML = "";
  const errs = res.errors ?? [];
  const notes = res.notes ?? [];
  const plaus = res.plausibility ?? [];
  if (errs.length === 0 && notes.length === 0 && plaus.length === 0) return;
  for (const e of errs) {
    const d = document.createElement("div");
    d.className = "mc-note mc-note-err";
    d.textContent = `⚠ ${e}`;
    host.appendChild(d);
  }
  for (const nt of notes) {
    const d = document.createElement("div");
    d.className = "mc-note";
    d.textContent = `ℹ ${nt}`;
    host.appendChild(d);
  }
  // Physical-plausibility line per well — never a silent pass: ✓ when every realization stayed in
  // bounds, ⚠ when a sampled combo produced an impossible Sw>1 / PHIE<0. Reported only: the P10/P50/
  // P90 above are unchanged (the module limits already clamp these draws to the correct volumetrics).
  for (const p of plaus) {
    const bad = p.impossible_realizations > 0;
    const d = document.createElement("div");
    d.className = bad ? "mc-note mc-note-err" : "mc-note";
    const pct = (p.fraction * 100).toFixed(1);
    if (bad) {
      d.textContent = `⚠ ${p.well_name}: ${pct}% of realizations hit impossible petrophysics (${p.detail}) — reported, not excluded; consider narrowing the input ranges`;
    } else if (!p.checked) {
      d.textContent = `• ${p.well_name}: ${p.detail}`;
    } else {
      d.textContent = `✓ ${p.well_name}: all realizations within physical bounds`;
    }
    host.appendChild(d);
  }
}

// --- small DOM helpers ------------------------------------------------------

function checkbox(label: string, def: boolean): { el: HTMLElement; checked: () => boolean } {
  const inp = document.createElement("input");
  inp.type = "checkbox";
  inp.checked = def;
  const l = document.createElement("label");
  l.className = "mc-check";
  l.append(inp, document.createTextNode(` ${label}`));
  return { el: l, checked: () => inp.checked };
}

function button(text: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.textContent = text;
  return b;
}
function mini(text: string, onClick: () => void): HTMLButtonElement {
  const b = button(text);
  b.className = "mc-mini";
  b.addEventListener("click", onClick);
  return b;
}
/** The ⚙ toggle that shows/hides a plot's properties panel. */
function gearButton(): HTMLButtonElement {
  const b = button("⚙");
  b.className = "mc-gear";
  b.title = "Plot properties (size, colour, axes)";
  return b;
}
function wrap(label: string, input: HTMLElement): HTMLElement {
  const s = document.createElement("label");
  s.className = "mc-field";
  const t = document.createElement("span");
  t.textContent = label;
  s.append(t, input);
  return s;
}
function numInput(value: number, onChange: (v: number) => void, title: string): HTMLInputElement {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.step = "any";
  inp.title = title;
  inp.value = Number.isFinite(value) ? String(value) : "";
  inp.addEventListener("change", () => {
    const v = parseFloat(inp.value);
    if (Number.isFinite(v)) onChange(v);
  });
  return inp;
}

const SVG_NS = "http://www.w3.org/2000/svg";
const sparkClamp01 = (t: number): number => (t < 0 ? 0 : t > 1 ? 1 : t);
const sparkRound = (v: number): number => Math.round(v * 100) / 100;

/** A small inline SVG that previews one uncertain parameter's distribution shape.
 *  Purely informational (never feeds the sampler) — `redraw` reads (kind, a, b, c) with the
 *  same field semantics as the row editor: normal → mean/sd, uniform → lo/hi, triangular →
 *  lo/mode/hi. Theme colours come from the `.mc-dist-spark` CSS class, not from JS. */
function distSparkline(): { el: SVGSVGElement; redraw: (kind: DistKind, a: number, b: number, c: number) => void } {
  const W = 60;
  const H = 22;
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  svg.setAttribute("preserveAspectRatio", "none");
  svg.setAttribute("class", "mc-dist-spark");
  const path = document.createElementNS(SVG_NS, "path");
  path.setAttribute("class", "mc-spark-fill");
  const title = document.createElementNS(SVG_NS, "title");
  svg.append(title, path);
  const redraw = (kind: DistKind, a: number, b: number, c: number): void => {
    const { d, label } = distSparkPath(kind, a, b, c, W, H);
    path.setAttribute("d", d);
    title.textContent = label;
  };
  return { el: svg, redraw };
}

/** The `<path d>` (a closed PDF area over a fixed W×H box) plus a `<title>` label for one
 *  distribution. A collapsed spread (sd≤0, lo==hi) renders as a narrow point-mass spike so the
 *  preview never goes blank. */
function distSparkPath(
  kind: DistKind,
  a: number,
  b: number,
  c: number,
  W: number,
  H: number,
): { d: string; label: string } {
  const pad = 2;
  const baseY = H - pad;
  const topY = pad;
  const x = (t: number): number => sparkRound(pad + sparkClamp01(t) * (W - 2 * pad));
  const y = (yn: number): number => sparkRound(baseY - sparkClamp01(yn) * (baseY - topY));
  const num = (v: number): string => (Number.isFinite(v) ? String(sparkRound(v)) : "?");
  const cx = W / 2;
  const spike = `M${cx - 1.4},${baseY} L${cx},${topY} L${cx + 1.4},${baseY} Z`;

  if (kind === "normal") {
    const m = a;
    const s = b;
    if (!Number.isFinite(m) || !Number.isFinite(s) || s <= 0) {
      return { d: spike, label: `normal(mean=${num(m)}, sd=${num(s)}) — no spread` };
    }
    const x0 = m - 3.2 * s;
    const x1 = m + 3.2 * s;
    const K = 48;
    let d = `M${x(0)},${baseY}`;
    for (let i = 0; i <= K; i++) {
      const t = i / K;
      const z = (x0 + t * (x1 - x0) - m) / s;
      d += ` L${x(t)},${y(Math.exp(-0.5 * z * z))}`;
    }
    d += ` L${x(1)},${baseY} Z`;
    return { d, label: `normal(mean=${num(m)}, sd=${num(s)})` };
  }

  if (kind === "uniform") {
    let lo = a;
    let hi = b;
    if (Number.isFinite(lo) && Number.isFinite(hi) && lo > hi) [lo, hi] = [hi, lo];
    if (!Number.isFinite(lo) || !Number.isFinite(hi) || hi <= lo) {
      return { d: spike, label: `uniform(min=${num(a)}, max=${num(b)}) — no spread` };
    }
    const w = hi - lo;
    const x0 = lo - 0.18 * w;
    const x1 = hi + 0.18 * w;
    const tLo = (lo - x0) / (x1 - x0);
    const tHi = (hi - x0) / (x1 - x0);
    const d =
      `M${x(0)},${baseY} L${x(tLo)},${baseY} L${x(tLo)},${topY} ` +
      `L${x(tHi)},${topY} L${x(tHi)},${baseY} L${x(1)},${baseY} Z`;
    return { d, label: `uniform(min=${num(lo)}, max=${num(hi)})` };
  }

  // triangular: lo=a, mode=b, hi=c
  let lo = a;
  let hi = c;
  if (Number.isFinite(lo) && Number.isFinite(hi) && lo > hi) [lo, hi] = [hi, lo];
  if (!Number.isFinite(lo) || !Number.isFinite(hi) || hi <= lo) {
    return { d: spike, label: `triangular(min=${num(a)}, mode=${num(b)}, max=${num(c)}) — no spread` };
  }
  let mode = b;
  if (!Number.isFinite(mode)) mode = 0.5 * (lo + hi);
  mode = Math.min(hi, Math.max(lo, mode));
  const tMode = (mode - lo) / (hi - lo);
  const d = `M${x(0)},${baseY} L${x(tMode)},${topY} L${x(1)},${baseY} Z`;
  return { d, label: `triangular(min=${num(lo)}, mode=${num(mode)}, max=${num(hi)})` };
}

// SB-CUT-016: `def` is absent-capable, because a cut-off has no shipped value. A null opens the
// field BLANK rather than pre-filling somebody's number.
function numField(label: string, def: number | null, min: number, max: number): { el: HTMLElement; value: () => number } {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.step = "any";
  if (Number.isFinite(min)) inp.min = String(min);
  if (Number.isFinite(max)) inp.max = String(max);
  inp.value = typeof def === "number" && Number.isFinite(def) ? String(def) : "";
  const el = wrap(label, inp);
  return { el, value: () => parseFloat(inp.value) };
}

// --- plot properties (⚙ panels) --------------------------------------------

/** A labelled height/number control inside a ⚙ properties panel. */
function propNum(label: string, value: number, min: number, max: number, onChange: (v: number) => void): HTMLElement {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.className = "mc-prop-num";
  inp.min = String(min);
  inp.max = String(max);
  inp.step = "10";
  inp.value = String(value);
  inp.addEventListener("change", () => {
    const v = Math.round(parseFloat(inp.value));
    if (Number.isFinite(v)) onChange(Math.min(max, Math.max(min, v)));
  });
  const l = document.createElement("label");
  l.className = "mc-prop";
  const t = document.createElement("span");
  t.textContent = label;
  l.append(t, inp);
  return l;
}

/** A labelled colour swatch inside a ⚙ properties panel. `seedVar` is the theme var used as the
 *  swatch's initial value when the option is still blank (= "follow theme"). */
function propColor(label: string, current: string, seedVar: string, fallback: string, onChange: (hex: string) => void): HTMLElement {
  const inp = document.createElement("input");
  inp.type = "color";
  inp.className = "mc-prop-color";
  inp.value = current ? toHexColor(current, fallback) : toHexColor(cssVar(seedVar), fallback);
  inp.addEventListener("input", () => onChange(inp.value));
  const l = document.createElement("label");
  l.className = "mc-prop";
  const t = document.createElement("span");
  t.textContent = label;
  l.append(t, inp);
  return l;
}

/** A labelled checkbox inside a ⚙ properties panel. */
function propCheck(label: string, checked: boolean, onChange: (v: boolean) => void): HTMLElement {
  const inp = document.createElement("input");
  inp.type = "checkbox";
  inp.checked = checked;
  inp.addEventListener("change", () => onChange(inp.checked));
  const l = document.createElement("label");
  l.className = "mc-prop mc-prop-check";
  l.append(inp, document.createTextNode(` ${label}`));
  return l;
}

function buildHistProps(opts: HistOptions, onChange: () => void): HTMLElement {
  const panel = document.createElement("div");
  panel.className = "mc-props";
  panel.hidden = true;
  panel.append(
    propNum("Height", opts.height, 120, 600, (v) => {
      opts.height = v;
      onChange();
    }),
    propColor("Bars", opts.barColor, "--accent", "#b5651d", (hex) => {
      opts.barColor = hex;
      onChange();
    }),
    propCheck("P10/50/90 markers", opts.showMarkers, (v) => {
      opts.showMarkers = v;
      onChange();
    }),
    propCheck("Gridlines", opts.showGrid, (v) => {
      opts.showGrid = v;
      onChange();
    }),
    propCheck("Y-axis (count)", opts.showYAxis, (v) => {
      opts.showYAxis = v;
      onChange();
    }),
  );
  return panel;
}

function buildTornProps(opts: TornOptions, onChange: () => void): HTMLElement {
  const panel = document.createElement("div");
  panel.className = "mc-props";
  panel.hidden = true;
  panel.append(
    propNum("Height", opts.height || 0, 0, 700, (v) => {
      opts.height = v; // 0 = auto
      onChange();
    }),
    propColor("Low side", opts.loColor, "--accent", "#b5651d", (hex) => {
      opts.loColor = hex;
      onChange();
    }),
    propColor("High side", opts.hiColor, "--accent2", "#5a7d4f", (hex) => {
      opts.hiColor = hex;
      onChange();
    }),
    propCheck("Row stripes", opts.showStripes, (v) => {
      opts.showStripes = v;
      onChange();
    }),
    propCheck("ρ labels", opts.showRho, (v) => {
      opts.showRho = v;
      onChange();
    }),
  );
  return panel;
}
