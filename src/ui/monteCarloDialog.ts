import {
  listDocuments,
  listModules,
  listWells,
  runMonteCarlo,
  type ChainStep,
  type McDistribution,
  type McParam,
  type McRequest,
  type McResult,
  type McZoneResult,
  type ModuleSpec,
  type Pctl,
  type WellSummary,
} from "../ipc";
import { appState, filterByActiveGroup } from "../state";
import { formRow, openModal } from "./modal";

const WORKFLOW_DOC_TYPE = "workflow";
const DEFAULT_STEPS = ["vsh_gr", "phi_dn", "sw_indo"];

type DistKind = McDistribution["kind"];

interface McRow {
  param: string;
  kind: DistKind;
  a: number; // normal.mean / uniform.lo / triangular.lo
  b: number; // normal.sd   / uniform.hi / triangular.mode
  c: number; // (triangular.hi)
}

interface ParamCandidate {
  name: string;
  default: number;
  unit: string;
  desc: string;
}

function emptyStep(module: string): ChainStep {
  return { module, log_inputs: {}, params: {}, opts: {} };
}

/** Monte Carlo uncertainty (Phase 9): put distributions on model parameters, run N seeded
 *  realizations of a chain in memory, and read the P10/P50/P90 spread of net pay / NTG /
 *  PHIE / SWE / HPV per zone plus an HPV histogram. */
export async function openMonteCarloDialog(setStatus: (text: string) => void): Promise<void> {
  const modules = await listModules().catch(() => [] as ModuleSpec[]);
  const wells = filterByActiveGroup(await listWells().catch(() => [] as WellSummary[]));
  const moduleByName = new Map(modules.map((m) => [m.name, m]));

  let steps: ChainStep[] = DEFAULT_STEPS.map(emptyStep);
  let mcRows: McRow[] = [];

  const content = document.createElement("div");
  content.className = "mc-dialog";

  // --- Chain source --------------------------------------------------------
  const chainSelect = document.createElement("select");
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
   *  normal → [mean, sd]; uniform → [lo, hi]; triangular → [lo, mode, hi]. */
  function distDefaults(d0: number, kind: DistKind): [number, number, number] {
    const d = Number.isFinite(d0) ? d0 : 0;
    const spread = Math.max(Math.abs(d) * 0.1, 0.01);
    const wide = Math.max(Math.abs(d) * 0.2, 0.02);
    if (kind === "normal") return [d, spread, d + spread];
    if (kind === "uniform") return [d - wide, d + wide, d + wide];
    return [d - wide, d, d + wide]; // triangular: lo, mode, hi
  }

  function defaultRow(c: ParamCandidate): McRow {
    const [a, b, cc] = distDefaults(c.default, "normal");
    return { param: c.name, kind: "normal", a, b, c: cc };
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
        [row.a, row.b, row.c] = distDefaults(c?.default ?? 0, row.kind);
        renderMcRows();
      });

      const kindSel = document.createElement("select");
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
        [row.a, row.b, row.c] = distDefaults(c?.default ?? row.a, row.kind);
        renderMcRows();
      });

      const [la, lb, lc] = labelsFor(row.kind);
      const inA = numInput(row.a, (v) => (row.a = v), la);
      const inB = numInput(row.b, (v) => (row.b = v), lb);
      const fields = document.createElement("span");
      fields.className = "mc-dist-fields";
      fields.append(wrap(la, inA), wrap(lb, inB));
      if (lc) {
        const inC = numInput(row.c, (v) => (row.c = v), lc);
        fields.append(wrap(lc, inC));
      }

      const rm = mini("✕", () => {
        mcRows.splice(i, 1);
        renderMcRows();
      });
      rm.classList.add("mc-rm");

      el.append(paramSel, kindSel, fields, rm);
      mcList.appendChild(el);
    });
  }

  const mcHead = document.createElement("div");
  mcHead.className = "mc-params-head";
  mcHead.append(addParamBtn);
  content.appendChild(formRow("Uncertainty", mcHead, "Distributions are sampled well-wide; parameters you don't vary follow their zone values."));
  content.appendChild(mcList);

  // --- Wells ---------------------------------------------------------------
  const wellsBox = document.createElement("div");
  wellsBox.className = "mc-wells";
  const wellChecks = new Map<string, HTMLInputElement>();
  const selected = appState.selectedWell.get();
  for (const w of wells) {
    const label = document.createElement("label");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.value = w.well_id;
    cb.checked = selected ? w.well_id === selected.well_id : true;
    wellChecks.set(w.well_id, cb);
    label.append(cb, document.createTextNode(` ${w.well_name}`));
    wellsBox.appendChild(label);
  }
  content.appendChild(formRow("Wells", wellsBox));

  // --- Settings (cutoffs + run controls) -----------------------------------
  const iters = numField("Iterations", 1000, 1, 100000);
  const seed = numField("Seed", 42, 0, 1e9);
  const bins = numField("HPV bins", 12, 3, 60);
  const vshMax = numField("VSH ≤", 0.5, 0, 1);
  const phieMin = numField("PHIE ≥", 0.08, 0, 1);
  const sweMax = numField("SWE ≤", 0.5, 0, 1);
  const permMin = numField("PERM ≥ (blank=off)", NaN, 0, 1e6);
  const grid = document.createElement("div");
  grid.className = "mc-settings";
  for (const f of [iters, seed, bins, vshMax, phieMin, sweMax, permMin]) grid.appendChild(f.el);
  content.appendChild(formRow("Settings", grid, "Cutoffs match the pay summary; PERM cutoff applies only where a PERM curve exists."));

  // --- Run + results -------------------------------------------------------
  const runBtn = button("Run Monte Carlo");
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

  runBtn.addEventListener("click", async () => {
    const wellIds = [...wellChecks.entries()].filter(([, cb]) => cb.checked).map(([id]) => id);
    if (wellIds.length === 0) {
      setStatus("Select at least one well");
      return;
    }
    const mcParams: McParam[] = mcRows.map((r) => ({ param: r.param, dist: toDist(r) }));
    const pm = permMin.value();
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
    };
    runBtn.disabled = true;
    statusLine.textContent = "Running…";
    const t0 = performance.now();
    try {
      const res = await runMonteCarlo(req);
      const ms = Math.round(performance.now() - t0);
      statusLine.textContent = `Done in ${ms} ms · ${res.zones.length} well-zone results`;
      renderResults(results, res, moduleByName);
      setStatus(`Monte Carlo: ${req.iterations} realizations across ${wellIds.length} well(s) in ${ms} ms`);
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
  openModal("Monte Carlo Uncertainty", content, 640);
}

function toDist(r: McRow): McDistribution {
  if (r.kind === "normal") return { kind: "normal", mean: r.a, sd: r.b };
  if (r.kind === "uniform") return { kind: "uniform", lo: r.a, hi: r.b };
  return { kind: "triangular", lo: r.a, mode: r.b, hi: r.c };
}

// --- Results rendering ------------------------------------------------------

function fmt(v: number, dp = 2): string {
  return Number.isFinite(v) ? v.toFixed(dp) : "—";
}
function cell(p: Pctl, dp = 2): string {
  return `${fmt(p.p50, dp)}  (${fmt(p.p10, dp)}–${fmt(p.p90, dp)})`;
}

function renderResults(host: HTMLElement, res: McResult, moduleByName: Map<string, ModuleSpec>): void {
  void moduleByName;
  host.innerHTML = "";
  if (res.zones.length === 0) {
    host.textContent = "No results (no curve data or no zones matched).";
    return;
  }

  const table = document.createElement("table");
  table.className = "mc-table";
  const head = document.createElement("tr");
  for (const h of ["Well", "Zone", "Net pay P50 (P10–P90)", "NTG", "Avg PHIE", "Avg SWE", "HPV P50 (P10–P90)"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);
  res.zones.forEach((z, i) => {
    const tr = document.createElement("tr");
    tr.className = "mc-row-click";
    const cells = [
      z.well_name,
      z.zone,
      cell(z.net, 1),
      cell(z.ntg, 3),
      cell(z.avg_phie, 3),
      cell(z.avg_swe, 3),
      cell(z.hpv, 2),
    ];
    for (const c of cells) {
      const td = document.createElement("td");
      td.textContent = c;
      tr.appendChild(td);
    }
    tr.addEventListener("click", () => {
      for (const r of table.querySelectorAll(".mc-row-sel")) r.classList.remove("mc-row-sel");
      tr.classList.add("mc-row-sel");
      drawHistogram(canvas, z);
    });
    if (i === 0) tr.classList.add("mc-row-sel");
    table.appendChild(tr);
  });
  host.appendChild(table);

  const histWrap = document.createElement("div");
  histWrap.className = "mc-hist-wrap";
  const caption = document.createElement("div");
  caption.className = "mc-hist-caption";
  caption.textContent = "HPV distribution (click a row) — bars, with P10 / P50 / P90 markers";
  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";
  canvas.width = 600;
  canvas.height = 200;
  histWrap.append(caption, canvas);
  host.appendChild(histWrap);

  drawHistogram(canvas, res.zones[0]);
}

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
}

function drawHistogram(canvas: HTMLCanvasElement, z: McZoneResult): void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const W = canvas.clientWidth || 600;
  const H = 200;
  canvas.width = W * dpr;
  canvas.height = H * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);

  const accent = cssVar("--accent");
  const accent2 = cssVar("--accent2");
  const text = cssVar("--text-dim");
  const border = cssVar("--border");

  const pad = { l: 8, r: 8, t: 10, b: 22 };
  const plotW = W - pad.l - pad.r;
  const plotH = H - pad.t - pad.b;
  const bins = z.hpv_hist;
  const maxCount = Math.max(1, ...bins);
  const bw = plotW / Math.max(1, bins.length);

  // Bars.
  ctx.fillStyle = accent;
  bins.forEach((c, i) => {
    const h = (c / maxCount) * plotH;
    ctx.fillRect(pad.l + i * bw + 1, pad.t + plotH - h, Math.max(1, bw - 2), h);
  });

  // Baseline + axis labels (HPV range).
  ctx.strokeStyle = border;
  ctx.beginPath();
  ctx.moveTo(pad.l, pad.t + plotH + 0.5);
  ctx.lineTo(pad.l + plotW, pad.t + plotH + 0.5);
  ctx.stroke();

  const hi = z.hist_lo + z.hist_w * bins.length;
  ctx.fillStyle = text;
  ctx.font = "500 10px system-ui, sans-serif";
  ctx.textBaseline = "top";
  ctx.textAlign = "left";
  ctx.fillText(`${z.hist_lo.toFixed(1)}`, pad.l, pad.t + plotH + 5);
  ctx.textAlign = "right";
  ctx.fillText(`${hi.toFixed(1)} (HPV)`, pad.l + plotW, pad.t + plotH + 5);

  // P10/P50/P90 markers.
  const xFor = (v: number): number => {
    const span = z.hist_w * bins.length || 1;
    return pad.l + ((v - z.hist_lo) / span) * plotW;
  };
  const marks: [number, string][] = [
    [z.hpv.p10, "P10"],
    [z.hpv.p50, "P50"],
    [z.hpv.p90, "P90"],
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

// --- small DOM helpers ------------------------------------------------------

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
function numField(label: string, def: number, min: number, max: number): { el: HTMLElement; value: () => number } {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.step = "any";
  if (Number.isFinite(min)) inp.min = String(min);
  if (Number.isFinite(max)) inp.max = String(max);
  inp.value = Number.isFinite(def) ? String(def) : "";
  const el = wrap(label, inp);
  return { el, value: () => parseFloat(inp.value) };
}
