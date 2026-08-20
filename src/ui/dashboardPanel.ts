import { resolveWellScope, runPaySummary, type BackendWellScope, type PaySummaryRow } from "../ipc";
import { appState } from "../state";
import { convertDepth, unitLabel } from "../units";
import { loadCutoffDefaults } from "./cutoffs";
import { escapeHtml } from "./safeDom";
import { reportDashboardCompletion } from "./reportingHonesty";
import { PARAM_SOURCE_TOPICS, withParamSources } from "./paramSources";

/** Field Dashboard: multi-well × zone pay statistics in one panel.
 *  Reuses the existing pay-summary engine (`run_pay_summary`) across every well, then
 *  presents the PAY/RESERVOIR/SAND rows as a sortable grid, a per-zone aggregation table,
 *  and per-zone box plots for a chosen metric, with CSV export. Well-independent (unlike
 *  the histogram/crossplot panels), so it takes no selected well. */

type Metric = "avg_phie" | "avg_swe" | "ntg" | "hpv" | "net";
type SortKey = keyof PaySummaryRow;

/** Every row field that is a LENGTH in the project's stored depth unit — depths, thicknesses,
 *  and HPV, which is a hydrocarbon pore THICKNESS (φe·(1−Sw)·h) and so converts like one.
 *  Everything else on the row (N/G, the volume-fraction averages, sample counts) is
 *  dimensionless and must never be touched by a unit conversion. */
const LENGTH_KEYS = new Set<SortKey>(["top", "bottom", "gross", "net", "not_net", "unknown", "hpv"]);

/** Stored → display conversion for one row field, applied at the moment of DISPLAY only.
 *  The backend returns project-unit values (`workflow.rs` accumulates raw sample thickness);
 *  this panel used to print and export them under a hard-coded "(m)", so a foot-declared
 *  project delivered a CSV whose HPV column was labelled metres and carried feet — a client
 *  deliverable wrong by exactly 3.28084x, with every number plausible. */
const toDisplay = (key: SortKey, value: number): number =>
  LENGTH_KEYS.has(key)
    ? convertDepth(value, appState.projectDepthUnit.get(), appState.displayDepthUnit.get())
    : value;

/** Same conversion for an already-aggregated length (a sum or a weighted mean of one). */
const lenToDisplay = (value: number): number =>
  convertDepth(value, appState.projectDepthUnit.get(), appState.displayDepthUnit.get());

const depthUnit = (): string => unitLabel(appState.displayDepthUnit.get());

/** `len` marks a length-valued column: it carries the depth unit in its heading and its
 *  values go through `toDisplay`. */
const METRICS: { key: Metric; label: string; digits: number; len?: true }[] = [
  { key: "avg_phie", label: "Avg PHIE", digits: 3 },
  { key: "avg_swe", label: "Avg SWE", digits: 3 },
  { key: "ntg", label: "N/G", digits: 2 },
  { key: "hpv", label: "HPV", digits: 2, len: true },
  { key: "net", label: "Net", digits: 1, len: true },
];

/** The metric's heading, with the unit resolved at render time rather than baked in. */
const metricLabel = (m: (typeof METRICS)[number]): string =>
  m.len ? `${m.label} (${depthUnit()})` : m.label;

const GRID_COLS: { key: SortKey; label: string; digits?: number; num: boolean; len?: true }[] = [
  { key: "well_name", label: "Well", num: false },
  { key: "zone", label: "Zone", num: false },
  { key: "top", label: "Top", digits: 1, num: true, len: true },
  { key: "bottom", label: "Bottom", digits: 1, num: true, len: true },
  { key: "gross", label: "Gross", digits: 1, num: true, len: true },
  { key: "net", label: "Net", digits: 1, num: true, len: true },
  // SB-CUT-003: the four-way partition, reported side by side because that is the only way to
  // read it. Gross = Net + Not net + Unknown exactly, and "Unknown" is footage nothing could
  // judge — no VSH/PHIE/SWE at the sample, or no sample at all. A zone that is 40 % net because
  // the rest is shale and one that is 40 % net because the rest was never logged print the same
  // N/G, and only these two columns separate them.
  { key: "not_net", label: "Not net", digits: 1, num: true, len: true },
  { key: "unknown", label: "Unknown", digits: 1, num: true, len: true },
  { key: "ntg", label: "N/G", digits: 2, num: true },
  // SB-CUT-004: the same net measured against only the footage that could be judged. Both are
  // labelled because they answer different questions; where nothing was judged this is MISSING
  // and renders "—", never 0.00.
  { key: "ntg_known", label: "N/G excl. Unk", digits: 2, num: true },
  { key: "avg_vsh", label: "Avg VSH", digits: 2, num: true },
  { key: "avg_phie", label: "Avg PHIE", digits: 3, num: true },
  { key: "avg_swe", label: "Avg SWE", digits: 3, num: true },
  { key: "hpv", label: "HPV", digits: 2, num: true, len: true },
];

/** A grid column's heading, with the depth unit resolved at render time. */
const colLabel = (c: (typeof GRID_COLS)[number]): string =>
  c.len ? `${c.label} (${depthUnit()})` : c.label;

// Guards null/undefined/NaN/Infinity → "—". Note: the Rust backend's f64::NAN
// crosses the IPC boundary as JSON `null` (serde_json has no NaN), so a plain
// Number.isNaN check would miss it and `null.toFixed()` would throw.
const fmt = (v: number | null | undefined, d = 2) =>
  typeof v === "number" && Number.isFinite(v) ? v.toFixed(d) : "—";

export async function buildDashboardContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const el = document.createElement("div");
  el.className = "dashboard";

  const num = (value: string, placeholder = "") => {
    const i = document.createElement("input");
    i.type = "number";
    i.step = "any";
    i.value = value;
    i.placeholder = placeholder;
    i.className = "dash-num";
    return i;
  };
  // SB-CUT-016: no cut-off ships a value. A blank box means the summation is UNFILTERED on that
  // property, which the result then reports; it does not mean "use ours".
  // SB-CUT-018: and the values come from the ONE shared authority, never from a literal here. This
  // pane was the last bypass - it seeded its own copies, so it could disagree with every other
  // pane about the same project's cutoffs.
  const saved = await loadCutoffDefaults();
  const seed = (v: number | null) => (v === null ? "" : String(v));
  const vshIn = num(seed(saved.vsh_max), "(unfiltered)");
  const phieIn = num(seed(saved.phie_min), "(unfiltered)");
  const sweIn = num(seed(saved.swe_max), "(unfiltered)");
  const permIn = num(seed(saved.perm_min), "(off)");
  /** SB-CUT-016: a blank cut-off box is ABSENT, never a shipped number. */
  const cutoffOf = (i: HTMLInputElement, unit = "v/v"): { value: number; unit: string } | null => {
    const v = parseFloat(i.value);
    // SB-CUT-019: the unit travels with the number, so the engine never has to guess whether a
    // porosity cut-off was typed in v/v or porosity units - a 350x difference.
    return Number.isFinite(v) ? { value: v, unit } : null;
  };

  // Flag / Metric are Organic segmented pills (design 1b) — same semantics the
  // old <select>s had, one value each, change re-renders from the held rows.
  let flagVal = "PAY";
  let metricVal: Metric = "avg_phie";
  const buildSeg = (opts: { value: string; label: string }[], get: () => string, set: (v: string) => void) => {
    const seg = document.createElement("div");
    seg.className = "seg";
    const paint = () =>
      seg.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.v === get())));
    for (const o of opts) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "seg-opt";
      b.dataset.v = o.value;
      b.textContent = o.label;
      b.addEventListener("click", () => {
        set(o.value);
        paint();
      });
      seg.appendChild(b);
    }
    paint();
    return seg;
  };

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Compute";

  const csvBtn = document.createElement("button");
  csvBtn.className = "btn";
  csvBtn.textContent = "Export CSV";
  csvBtn.disabled = true;

  // Header row (design 1b): display-face title + a group·wells tag, actions
  // right-aligned. The tag stays hidden until a Compute has established what
  // scope the numbers on screen actually describe.
  const header = document.createElement("div");
  header.className = "dashboard-header";
  const title = document.createElement("span");
  title.className = "dashboard-title";
  title.textContent = "Field Dashboard";
  const scopeTag = document.createElement("span");
  scopeTag.className = "dash-tag";
  scopeTag.hidden = true;
  const actions = document.createElement("span");
  actions.className = "dashboard-actions";
  actions.append(csvBtn, runBtn);
  header.append(title, scopeTag, actions);
  el.appendChild(header);

  const controls = document.createElement("div");
  controls.className = "dashboard-controls";
  const field = (label: string, node: HTMLElement) => {
    const w = document.createElement("label");
    w.className = "dash-field";
    w.innerHTML = `<span>${escapeHtml(label)}</span>`;
    w.appendChild(node);
    return w;
  };
  const flagSeg = buildSeg(
    ["PAY", "RESERVOIR", "SAND"].map((f) => ({ value: f, label: f })),
    () => flagVal,
    (v) => {
      flagVal = v;
      render();
    },
  );
  const metricSeg = buildSeg(
    METRICS.map((m) => ({ value: m.key, label: metricLabel(m) })),
    () => metricVal,
    (v) => {
      metricVal = v as Metric;
      render();
    },
  );
  controls.append(
    field("VSH ≤", withParamSources(vshIn, PARAM_SOURCE_TOPICS.cutoffVshMax)),
    field("PHIE ≥", withParamSources(phieIn, PARAM_SOURCE_TOPICS.cutoffPhieMin)),
    field("SWE ≤", withParamSources(sweIn, PARAM_SOURCE_TOPICS.cutoffSweMax)),
    field("PERM ≥", permIn),
    field("Flag", flagSeg),
    field("Metric", metricSeg),
  );
  el.appendChild(controls);

  const statusEl = document.createElement("div");
  statusEl.className = "dashboard-status";
  statusEl.textContent = "Set cutoffs and press Compute to summarize every well.";
  el.appendChild(statusEl);

  const body = document.createElement("div");
  body.className = "dashboard-body";
  el.appendChild(body);

  // Rows for the currently-selected flag, held for re-render on sort / flag / metric change.
  let allRows: PaySummaryRow[] = [];
  let sortKey: SortKey = "hpv";
  let sortDir: 1 | -1 = -1;

  // Rows whose classifier could judge nothing (VSH/PHIE/SWE never computed for that well) carry
  // net/ntg/hpv of exactly 0 — indistinguishable from a genuine wet zone. Here that is worse
  // than a mis-rendered cell: feeding those zeros into the medians and box plots would drag
  // every aggregate down with data that does not exist. So they are excluded from every number
  // and counted — design 1b shows them GREYED at the bottom of the grid (a delivery that
  // silently lost rows reads as complete) with a ZONES EXCLUDED card and a footnote saying so.
  // Returns both lists rather than assigning to an outer variable: the CSV handler calls this
  // outside `render`, so a side-effecting version would leave the on-screen state describing a
  // different selection than the one displayed.
  const rowsForFlag = (): { rows: PaySummaryRow[]; excluded: PaySummaryRow[] } => {
    const forFlag = allRows.filter((r) => r.flag === flagVal);
    const rows = forFlag.filter((r) => r.n_classified > 0);
    return { rows, excluded: forFlag.filter((r) => r.n_classified === 0) };
  };

  // ---- Section: KPI cards (design 1b) — the display face carries the numerals ----
  const sectionKpis = (rows: PaySummaryRow[], excluded: PaySummaryRow[]) => {
    const wrap = document.createElement("div");
    wrap.className = "dash-kpis";
    const kpi = (label: string, value: string, cls: string, suffix?: string) => {
      const card = document.createElement("div");
      card.className = `kpi-card ${cls}`;
      const l = document.createElement("div");
      l.className = "kpi-label";
      l.textContent = label;
      const v = document.createElement("div");
      v.className = "kpi-value";
      v.textContent = value;
      if (suffix) {
        const s = document.createElement("span");
        s.className = "kpi-suffix";
        s.textContent = ` ${suffix}`;
        v.appendChild(s);
      }
      card.append(l, v);
      return card;
    };
    // Same arithmetic the tables below use — the cards are a reading of the
    // aggregation, never a second implementation of it.
    const flagSfx = flagVal === "PAY" ? "PAY" : flagVal;
    wrap.append(
      // Summed in the STORED unit, converted once for display — summing converted values would
      // accumulate the conversion's rounding across every row.
      kpi(`TOTAL NET ${flagSfx}`, fmt(lenToDisplay(sum(rows, "net")), 1), "kpi-accent", depthUnit()),
      kpi("TOTAL HPV", fmt(lenToDisplay(sum(rows, "hpv")), 2), "kpi-accent2", depthUnit()),
      kpi(`MEAN PHIE (${flagVal})`, fmt(weightedMean(rows, "avg_phie", "net"), 3), "kpi-neutral"),
      kpi(`MEAN SWE (${flagVal})`, fmt(weightedMean(rows, "avg_swe", "net"), 3), "kpi-neutral"),
      kpi("ZONES EXCLUDED", String(excluded.length), "kpi-neutral", excluded.length ? "no results" : ""),
    );
    return wrap;
  };

  const render = () => {
    const { rows, excluded } = rowsForFlag();
    body.innerHTML = "";
    if (rows.length === 0) {
      const why =
        excluded.length > 0
          ? `None of the ${excluded.length} ${flagVal} interval(s) could be classified — run VSH/PHIE/SWE for these wells first.`
          : `No ${flagVal} intervals. Check cutoffs, or that VSH/PHIE/SWE are computed.`;
      body.innerHTML = `<div class="dashboard-empty">${escapeHtml(why)}</div>`;
      csvBtn.disabled = true;
      return;
    }
    csvBtn.disabled = false;
    const metric = METRICS.find((m) => m.key === metricVal)!;
    body.appendChild(sectionKpis(rows, excluded));
    // Wells with no PERM curve against an active permeability cutoff. Every sample fails it for
    // want of data, so they contribute a hard zero to every average and box below — which looks
    // exactly like a well that was judged and found wet (`docs/review_triage.md` finding 7). This
    // is the surface where that zero gets summed with real ones, so it is the surface that has to
    // say so.
    const noPerm = [...new Set(rows.filter((r) => r.perm_cutoff_no_data).map((r) => r.well_name))];
    if (noPerm.length > 0) {
      const note = document.createElement("div");
      note.className = "dashboard-empty";
      note.style.color = "var(--warn)";
      note.textContent =
        `${noPerm.length} well(s) carry no permeability curve, so every sample fails the PERM ` +
        `cutoff for want of data: ${noPerm.slice(0, 8).join(", ")}${noPerm.length > 8 ? ", …" : ""}. ` +
        `Their zeros are counted in the averages below and mean "not measured", not "not pay".`;
      body.appendChild(note);
    }
    body.appendChild(sectionByZone(rows, metric));
    body.appendChild(sectionBoxPlots(rows, metric, excluded.length));
    body.appendChild(sectionGrid(rows, excluded));
  };

  // ---- Section: per-zone aggregation ----
  const sectionByZone = (rows: PaySummaryRow[], metric: (typeof METRICS)[number]) => {
    const byZone = groupBy(rows, (r) => r.zone);
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>By zone — ${escapeHtml(flagVal)}</h4>`;
    const table = document.createElement("table");
    table.className = "summary-table";
    const u = escapeHtml(depthUnit());
    table.innerHTML =
      `<thead><tr><th>Zone</th><th>Wells</th><th>Σ Net (${u})</th><th>Σ HPV (${u})</th>` +
      "<th>Mean N/G</th><th>Mean PHIE</th><th>Mean SWE</th></tr></thead>";
    const tb = document.createElement("tbody");
    for (const [zone, zr] of byZone) {
      const netW = (k: keyof PaySummaryRow) => weightedMean(zr, k, "net");
      const tr = document.createElement("tr");
      tr.innerHTML =
        `<td>${zone}</td><td>${new Set(zr.map((r) => r.well_id)).size}</td>` +
        `<td>${fmt(lenToDisplay(sum(zr, "net")), 1)}</td><td>${fmt(lenToDisplay(sum(zr, "hpv")), 2)}</td>` +
        `<td>${fmt(mean(zr, "ntg"), 2)}</td><td>${fmt(netW("avg_phie"), 3)}</td><td>${fmt(netW("avg_swe"), 3)}</td>`;
      tb.appendChild(tr);
    }
    table.appendChild(tb);
    sec.appendChild(table);
    void metric;
    return sec;
  };

  // ---- Section: per-zone box plots for the chosen metric ----
  const sectionBoxPlots = (rows: PaySummaryRow[], metric: (typeof METRICS)[number], nExcluded: number) => {
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>${escapeHtml(metricLabel(metric))} distribution by zone</h4>`;
    const byZone = groupBy(rows, (r) => r.zone);
    // Converted BEFORE the box statistics rather than after: percentiles and whiskers land on
    // real samples (`distribution.ts`), and converting a chosen sample is the same number
    // whichever side of the summary it happens on — but converting first keeps the axis, the
    // box edges and the outlier dots in one unit with no site left behind.
    const stats = Array.from(byZone.entries())
      .map(([zone, zr]) => ({
        zone,
        box: boxStats(zr.map((r) => toDisplay(metric.key, r[metric.key] as number))),
      }))
      .filter((s) => s.box);
    if (stats.length === 0) {
      sec.innerHTML += `<div class="dashboard-empty">No finite ${metricLabel(metric)} values.</div>`;
      return sec;
    }
    const lo = Math.min(...stats.map((s) => s.box!.min));
    const hi = Math.max(...stats.map((s) => s.box!.max));
    sec.appendChild(renderBoxPlots(stats as { zone: string; box: BoxStats }[], lo, hi, metric.digits));
    if (nExcluded > 0) {
      const note = document.createElement("div");
      note.className = "dashboard-footnote";
      note.textContent =
        `${nExcluded} zone(s) with no computed results are excluded and counted — never averaged in as zero.`;
      sec.appendChild(note);
    }
    return sec;
  };

  // ---- Section: sortable interval grid ----
  const sectionGrid = (rows: PaySummaryRow[], excluded: PaySummaryRow[]) => {
    const bySort = (a: PaySummaryRow, b: PaySummaryRow) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (typeof av === "number" && typeof bv === "number") return (av - bv) * sortDir;
      return String(av).localeCompare(String(bv)) * sortDir;
    };
    const sorted = [...rows].sort(bySort);
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>All ${escapeHtml(flagVal)} intervals (${rows.length})</h4>`;
    const wrap = document.createElement("div");
    wrap.className = "summary-table-wrap";
    const table = document.createElement("table");
    table.className = "summary-table dashboard-grid";
    const thead = document.createElement("thead");
    const htr = document.createElement("tr");
    for (const col of GRID_COLS) {
      const th = document.createElement("th");
      th.textContent = colLabel(col) + (sortKey === col.key ? (sortDir === 1 ? " ▲" : " ▼") : "");
      th.classList.add("sortable");
      th.addEventListener("click", () => {
        if (sortKey === col.key) sortDir = sortDir === 1 ? -1 : 1;
        else {
          sortKey = col.key;
          sortDir = col.num ? -1 : 1;
        }
        render();
      });
      htr.appendChild(th);
    }
    thead.appendChild(htr);
    table.appendChild(thead);
    const tb = document.createElement("tbody");
    // On an excluded row the classifier judged nothing, so net/N-G/averages/HPV
    // are 0 for want of an answer — printed they would read as computed zeros.
    // Same rule as the workbook: those cells go blank, GROSS is geometry and
    // stays a number (office.rs "a blank is not a zero").
    const CLASSIFIED_COLS = new Set<SortKey>(["net", "ntg", "avg_vsh", "avg_phie", "avg_swe", "hpv"]);
    const addRow = (r: PaySummaryRow, cls: string) => {
      const tr = document.createElement("tr");
      tr.className = cls;
      tr.innerHTML = GRID_COLS.map((c) => {
        if (!c.num) return `<td>${escapeHtml(String(r[c.key]))}</td>`;
        if (cls === "row-excluded" && CLASSIFIED_COLS.has(c.key)) return "<td>—</td>";
        return `<td>${fmt(toDisplay(c.key, r[c.key] as number), c.digits ?? 2)}</td>`;
      }).join("");
      tb.appendChild(tr);
    };
    // Top row of the current sort is highlighted (design 1b). Excluded rows
    // trail GREYED at the bottom, in the grid but in none of the numbers —
    // a delivery that silently lost rows would read as complete.
    sorted.forEach((r, i) => addRow(r, i === 0 ? "row-top" : ""));
    [...excluded].sort(bySort).forEach((r) => addRow(r, "row-excluded"));
    table.appendChild(tb);
    wrap.appendChild(table);
    sec.appendChild(wrap);
    return sec;
  };

  runBtn.addEventListener("click", async () => {
    let wellIds: string[];
    const displayGroup = appState.activeWellGroup.get();
    const backendScope: BackendWellScope = { kind: "active_group" };
    try {
      wellIds = await resolveWellScope(backendScope);
    } catch (err) {
      statusEl.textContent = `Failed to list wells: ${err}`;
      return;
    }
    if (wellIds.length === 0) {
      statusEl.textContent = "No wells in the project.";
      return;
    }
    const permRaw = parseFloat(permIn.value);
    runBtn.disabled = true;
    statusEl.textContent = `Computing ${wellIds.length} well(s)…`;
    try {
      allRows = await runPaySummary(
        {
          well_ids: wellIds,
          vsh_max: cutoffOf(vshIn),
          phie_min: cutoffOf(phieIn),
          swe_max: cutoffOf(sweIn),
          perm_min: Number.isNaN(permRaw) ? null : { value: permRaw, unit: "mD" },
          // Dashboard is read-only: compute the stats, persist nothing. Skips ~1,600 FLAG-curve
          // write transactions per Compute. Persisting flags stays with Cutoffs & Summary.
          stats_only: true,
        },
        backendScope,
      );
      const flags = new Set(allRows.map((r) => r.flag));
      const resolvedWellCount = new Set(allRows.map((row) => row.well_id)).size;
      reportDashboardCompletion(statusEl, resolvedWellCount, allRows.length, flags.size);
      setStatus(`Field dashboard: ${allRows.length} rows over ${resolvedWellCount} wells`);
      // Scope tag (design 1b): which group these numbers describe, and how many
      // wells actually went in — set only once a Compute has made that true.
      scopeTag.textContent = `${displayGroup ? `Group: ${displayGroup.name}` : "All wells"} · ${resolvedWellCount} well${resolvedWellCount === 1 ? "" : "s"}`;
      scopeTag.hidden = false;
      render();
    } catch (err) {
      statusEl.textContent = `Compute failed: ${err}`;
      allRows = [];
      body.innerHTML = "";
    } finally {
      runBtn.disabled = false;
    }
  });

  // Exports the same usable rows the panel shows — an uninterpreted well's zeros would read as a
  // genuine wet zone in a spreadsheet, where there is no dimmed styling to say otherwise.
  csvBtn.addEventListener("click", () => exportCsv(rowsForFlag().rows, flagVal));

  // Both units are read at render time, so a change to either must repaint: the display unit is
  // switchable from the log view at any moment, and the project unit is re-read on a project
  // switch. Without this the panel would keep the old unit in its headings while the rest of the
  // application had moved - the same mislabelling this increment exists to remove, one step
  // removed. The metric pill is rebuilt too, since its own labels carry the unit.
  // The metric pill's labels are relabelled IN PLACE rather than rebuilt: `buildSeg`'s click
  // handlers close over the element they were built on, so replacing its children would leave
  // the pressed-state painting pointing at a detached node.
  const repaint = () => {
    metricSeg.querySelectorAll("button").forEach((b) => {
      const m = METRICS.find((x) => x.key === b.dataset.v);
      if (m) b.textContent = metricLabel(m);
    });
    if (allRows.length > 0) render();
  };
  const unsubscribe = [
    appState.displayDepthUnit.subscribe(repaint),
    appState.projectDepthUnit.subscribe(repaint),
  ];

  return { el, dispose: () => unsubscribe.forEach((off) => off()) };
}

// ---------------------------------------------------------------------------
// Box-plot rendering (dependency-free inline SVG, theme-aware via CSS vars)
// ---------------------------------------------------------------------------

interface BoxStats {
  min: number;
  q1: number;
  med: number;
  q3: number;
  max: number;
  n: number;
}

function boxStats(values: number[]): BoxStats | null {
  const v = values.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);
  if (v.length === 0) return null;
  const q = (p: number) => {
    const idx = p * (v.length - 1);
    const lo = Math.floor(idx);
    const hi = Math.ceil(idx);
    return v[lo] + (v[hi] - v[lo]) * (idx - lo);
  };
  return { min: v[0], q1: q(0.25), med: q(0.5), q3: q(0.75), max: v[v.length - 1], n: v.length };
}

function renderBoxPlots(stats: { zone: string; box: BoxStats }[], lo: number, hi: number, digits: number): SVGSVGElement {
  const rowH = 26;
  const padL = 96;
  const padR = 56;
  const width = 560;
  const height = stats.length * rowH + 24;
  const span = hi - lo || 1;
  const x = (val: number) => padL + ((val - lo) / span) * (width - padL - padR);

  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.setAttribute("class", "dashboard-boxplot");
  svg.setAttribute("width", "100%");

  const ns = "http://www.w3.org/2000/svg";
  const line = (x1: number, y1: number, x2: number, y2: number, cls: string) => {
    const l = document.createElementNS(ns, "line");
    l.setAttribute("x1", `${x1}`);
    l.setAttribute("y1", `${y1}`);
    l.setAttribute("x2", `${x2}`);
    l.setAttribute("y2", `${y2}`);
    l.setAttribute("class", cls);
    svg.appendChild(l);
  };
  const text = (tx: number, ty: number, s: string, cls: string) => {
    const t = document.createElementNS(ns, "text");
    t.setAttribute("x", `${tx}`);
    t.setAttribute("y", `${ty}`);
    t.setAttribute("class", cls);
    t.textContent = s;
    svg.appendChild(t);
  };

  stats.forEach((s, i) => {
    const cy = 14 + i * rowH + rowH / 2 - 4;
    const b = s.box;
    text(6, cy + 4, s.zone, "bp-label");
    // whiskers
    line(x(b.min), cy, x(b.q1), cy, "bp-whisker");
    line(x(b.q3), cy, x(b.max), cy, "bp-whisker");
    line(x(b.min), cy - 5, x(b.min), cy + 5, "bp-cap");
    line(x(b.max), cy - 5, x(b.max), cy + 5, "bp-cap");
    // box
    const rect = document.createElementNS(ns, "rect");
    rect.setAttribute("x", `${x(b.q1)}`);
    rect.setAttribute("y", `${cy - 8}`);
    rect.setAttribute("width", `${Math.max(1, x(b.q3) - x(b.q1))}`);
    rect.setAttribute("height", "16");
    rect.setAttribute("class", "bp-box");
    svg.appendChild(rect);
    // median
    line(x(b.med), cy - 8, x(b.med), cy + 8, "bp-median");
    text(width - padR + 6, cy + 4, `${b.med.toFixed(digits)} (n${b.n})`, "bp-value");
  });
  // axis min/max labels
  text(padL, height - 4, lo.toFixed(digits), "bp-axis");
  text(width - padR - 20, height - 4, hi.toFixed(digits), "bp-axis");
  return svg;
}

// ---------------------------------------------------------------------------
// CSV export (client-side blob download of the user's own summary)
// ---------------------------------------------------------------------------

/** The CSV text the dashboard exports, separated from the download so the deliverable itself is
 *  testable. The header carries the SAME resolved unit the grid shows and the values go through
 *  the SAME conversion — this file leaves the building, and a header that says metres over feet
 *  is a reserves error nobody downstream can detect, because every number in it is plausible. */
export function buildDashboardCsv(rows: PaySummaryRow[]): string {
  const header = GRID_COLS.map((c) => colLabel(c));
  const lines = [header.join(",")];
  for (const r of rows) {
    lines.push(
      GRID_COLS.map((c) => {
        const v = r[c.key];
        if (v == null) return ""; // null (backend NaN→null) → empty cell, not the string "null"
        if (typeof v !== "number") return `"${String(v).replace(/"/g, '""')}"`;
        return Number.isNaN(v) ? "" : toDisplay(c.key, v);
      }).join(","),
    );
  }
  return lines.join("\r\n");
}

function exportCsv(rows: PaySummaryRow[], flag: string): void {
  const blob = new Blob([buildDashboardCsv(rows)], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `field-dashboard-${flag.toLowerCase()}.csv`;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

// ---------------------------------------------------------------------------
// Small aggregation helpers
// ---------------------------------------------------------------------------

function groupBy<T>(rows: T[], key: (r: T) => string): Map<string, T[]> {
  const m = new Map<string, T[]>();
  for (const r of rows) {
    const k = key(r);
    (m.get(k) ?? m.set(k, []).get(k)!).push(r);
  }
  return m;
}

function sum(rows: PaySummaryRow[], k: keyof PaySummaryRow): number {
  return rows.reduce((s, r) => s + (Number.isFinite(r[k] as number) ? (r[k] as number) : 0), 0);
}

function mean(rows: PaySummaryRow[], k: keyof PaySummaryRow): number {
  const vals = rows.map((r) => r[k] as number).filter((v) => Number.isFinite(v));
  return vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : NaN;
}

/** Net-thickness-weighted mean (the petrophysically meaningful average for PHIE/SWE). */
function weightedMean(rows: PaySummaryRow[], k: keyof PaySummaryRow, weightKey: keyof PaySummaryRow): number {
  let wsum = 0;
  let vsum = 0;
  for (const r of rows) {
    const v = r[k] as number;
    const w = r[weightKey] as number;
    if (Number.isFinite(v) && Number.isFinite(w) && w > 0) {
      vsum += v * w;
      wsum += w;
    }
  }
  return wsum > 0 ? vsum / wsum : NaN;
}

