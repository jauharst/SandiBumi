import { listWells, runPaySummary, type PaySummaryRow } from "../ipc";
import { filterByActiveGroup } from "../state";

/** Field Dashboard: multi-well × zone pay statistics in one panel.
 *  Reuses the existing pay-summary engine (`run_pay_summary`) across every well, then
 *  presents the PAY/RESERVOIR/SAND rows as a sortable grid, a per-zone aggregation table,
 *  and per-zone box plots for a chosen metric, with CSV export. Well-independent (unlike
 *  the histogram/crossplot panels), so it takes no selected well. */

type Metric = "avg_phie" | "avg_swe" | "ntg" | "hpv" | "net";
type SortKey = keyof PaySummaryRow;

const METRICS: { key: Metric; label: string; digits: number }[] = [
  { key: "avg_phie", label: "Avg PHIE", digits: 3 },
  { key: "avg_swe", label: "Avg SWE", digits: 3 },
  { key: "ntg", label: "N/G", digits: 2 },
  { key: "hpv", label: "HPV (m)", digits: 2 },
  { key: "net", label: "Net (m)", digits: 1 },
];

const GRID_COLS: { key: SortKey; label: string; digits?: number; num: boolean }[] = [
  { key: "well_name", label: "Well", num: false },
  { key: "zone", label: "Zone", num: false },
  { key: "top", label: "Top", digits: 1, num: true },
  { key: "bottom", label: "Bottom", digits: 1, num: true },
  { key: "gross", label: "Gross", digits: 1, num: true },
  { key: "net", label: "Net", digits: 1, num: true },
  { key: "ntg", label: "N/G", digits: 2, num: true },
  { key: "avg_vsh", label: "Avg VSH", digits: 2, num: true },
  { key: "avg_phie", label: "Avg PHIE", digits: 3, num: true },
  { key: "avg_swe", label: "Avg SWE", digits: 3, num: true },
  { key: "hpv", label: "HPV (m)", digits: 2, num: true },
];

const fmt = (v: number, d = 2) => (Number.isNaN(v) ? "—" : v.toFixed(d));

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
  const vshIn = num("0.5");
  const phieIn = num("0.1");
  const sweIn = num("0.6");
  const permIn = num("", "(off)");

  const flagSel = document.createElement("select");
  flagSel.className = "dash-sel";
  flagSel.innerHTML = ["PAY", "RESERVOIR", "SAND"].map((f) => `<option value="${f}">${f}</option>`).join("");

  const metricSel = document.createElement("select");
  metricSel.className = "dash-sel";
  metricSel.innerHTML = METRICS.map((m) => `<option value="${m.key}">${m.label}</option>`).join("");

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Compute";

  const csvBtn = document.createElement("button");
  csvBtn.className = "btn";
  csvBtn.textContent = "Export CSV";
  csvBtn.disabled = true;

  const controls = document.createElement("div");
  controls.className = "dashboard-controls";
  const field = (label: string, node: HTMLElement) => {
    const w = document.createElement("label");
    w.className = "dash-field";
    w.innerHTML = `<span>${label}</span>`;
    w.appendChild(node);
    return w;
  };
  controls.append(
    field("VSH ≤", vshIn),
    field("PHIE ≥", phieIn),
    field("SWE ≤", sweIn),
    field("PERM ≥", permIn),
    field("Flag", flagSel),
    field("Metric", metricSel),
    runBtn,
    csvBtn,
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

  const rowsForFlag = () => allRows.filter((r) => r.flag === flagSel.value);

  const render = () => {
    const rows = rowsForFlag();
    body.innerHTML = "";
    if (rows.length === 0) {
      body.innerHTML = `<div class="dashboard-empty">No ${flagSel.value} intervals. Check cutoffs, or that VSH/PHIE/SWE are computed.</div>`;
      csvBtn.disabled = true;
      return;
    }
    csvBtn.disabled = false;
    const metric = METRICS.find((m) => m.key === metricSel.value)!;
    body.appendChild(sectionByZone(rows, metric));
    body.appendChild(sectionBoxPlots(rows, metric));
    body.appendChild(sectionGrid(rows));
  };

  // ---- Section: per-zone aggregation ----
  const sectionByZone = (rows: PaySummaryRow[], metric: (typeof METRICS)[number]) => {
    const byZone = groupBy(rows, (r) => r.zone);
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>By zone — ${flagSel.value}</h4>`;
    const table = document.createElement("table");
    table.className = "summary-table";
    table.innerHTML =
      "<thead><tr><th>Zone</th><th>Wells</th><th>Σ Net (m)</th><th>Σ HPV (m)</th>" +
      "<th>Mean N/G</th><th>Mean PHIE</th><th>Mean SWE</th></tr></thead>";
    const tb = document.createElement("tbody");
    for (const [zone, zr] of byZone) {
      const netW = (k: keyof PaySummaryRow) => weightedMean(zr, k, "net");
      const tr = document.createElement("tr");
      tr.innerHTML =
        `<td>${zone}</td><td>${new Set(zr.map((r) => r.well_id)).size}</td>` +
        `<td>${fmt(sum(zr, "net"), 1)}</td><td>${fmt(sum(zr, "hpv"), 2)}</td>` +
        `<td>${fmt(mean(zr, "ntg"), 2)}</td><td>${fmt(netW("avg_phie"), 3)}</td><td>${fmt(netW("avg_swe"), 3)}</td>`;
      tb.appendChild(tr);
    }
    table.appendChild(tb);
    sec.appendChild(table);
    void metric;
    return sec;
  };

  // ---- Section: per-zone box plots for the chosen metric ----
  const sectionBoxPlots = (rows: PaySummaryRow[], metric: (typeof METRICS)[number]) => {
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>${metric.label} distribution by zone</h4>`;
    const byZone = groupBy(rows, (r) => r.zone);
    const stats = Array.from(byZone.entries())
      .map(([zone, zr]) => ({ zone, box: boxStats(zr.map((r) => r[metric.key] as number)) }))
      .filter((s) => s.box);
    if (stats.length === 0) {
      sec.innerHTML += `<div class="dashboard-empty">No finite ${metric.label} values.</div>`;
      return sec;
    }
    const lo = Math.min(...stats.map((s) => s.box!.min));
    const hi = Math.max(...stats.map((s) => s.box!.max));
    sec.appendChild(renderBoxPlots(stats as { zone: string; box: BoxStats }[], lo, hi, metric.digits));
    return sec;
  };

  // ---- Section: sortable interval grid ----
  const sectionGrid = (rows: PaySummaryRow[]) => {
    const sorted = [...rows].sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (typeof av === "number" && typeof bv === "number") return (av - bv) * sortDir;
      return String(av).localeCompare(String(bv)) * sortDir;
    });
    const sec = document.createElement("div");
    sec.className = "dashboard-section";
    sec.innerHTML = `<h4>All ${flagSel.value} intervals (${rows.length})</h4>`;
    const wrap = document.createElement("div");
    wrap.className = "summary-table-wrap";
    const table = document.createElement("table");
    table.className = "summary-table dashboard-grid";
    const thead = document.createElement("thead");
    const htr = document.createElement("tr");
    for (const col of GRID_COLS) {
      const th = document.createElement("th");
      th.textContent = col.label + (sortKey === col.key ? (sortDir === 1 ? " ▲" : " ▼") : "");
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
    for (const r of sorted) {
      const tr = document.createElement("tr");
      tr.className = `flag-${r.flag.toLowerCase()}`;
      tr.innerHTML = GRID_COLS.map((c) =>
        c.num ? `<td>${fmt(r[c.key] as number, c.digits ?? 2)}</td>` : `<td>${escapeHtml(String(r[c.key]))}</td>`,
      ).join("");
      tb.appendChild(tr);
    }
    table.appendChild(tb);
    wrap.appendChild(table);
    sec.appendChild(wrap);
    return sec;
  };

  runBtn.addEventListener("click", async () => {
    let wellIds: string[];
    try {
      wellIds = filterByActiveGroup(await listWells()).map((w) => w.well_id);
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
      allRows = await runPaySummary({
        well_ids: wellIds,
        vsh_max: parseFloat(vshIn.value),
        phie_min: parseFloat(phieIn.value),
        swe_max: parseFloat(sweIn.value),
        perm_min: Number.isNaN(permRaw) ? null : permRaw,
        skip_version: true, // field-wide QC — overwrite FLAG_* in place, don't version per well each refresh
      });
      const flags = new Set(allRows.map((r) => r.flag));
      statusEl.textContent = `${wellIds.length} well(s) · ${allRows.length} zone-rows across ${flags.size} flag level(s). FLAG curves written.`;
      setStatus(`Field dashboard: ${allRows.length} rows over ${wellIds.length} wells`);
      render();
    } catch (err) {
      statusEl.textContent = `Compute failed: ${err}`;
      allRows = [];
      body.innerHTML = "";
    } finally {
      runBtn.disabled = false;
    }
  });

  flagSel.addEventListener("change", render);
  metricSel.addEventListener("change", render);
  csvBtn.addEventListener("click", () => exportCsv(rowsForFlag(), flagSel.value));

  return { el };
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

function exportCsv(rows: PaySummaryRow[], flag: string): void {
  const header = GRID_COLS.map((c) => c.label);
  const lines = [header.join(",")];
  for (const r of rows) {
    lines.push(
      GRID_COLS.map((c) => {
        const v = r[c.key];
        return typeof v === "number" ? (Number.isNaN(v) ? "" : v) : `"${String(v).replace(/"/g, '""')}"`;
      }).join(","),
    );
  }
  const blob = new Blob([lines.join("\r\n")], { type: "text/csv;charset=utf-8" });
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

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}
