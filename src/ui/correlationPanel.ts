import {
  deleteFluidContact,
  getTrackData,
  listFluidContacts,
  listTops,
  listWells,
  upsertFluidContact,
  type FluidContact,
  type TopEntry,
  type TrackCurveSeries,
  type WellSummary,
} from "../ipc";
import { appState, filterByActiveGroup } from "../state";
import { openModal } from "./modal";
import { canvasFont, percentile, readTheme } from "./plotCanvas";
import { curveSelect, loadCurveNames, loadPlotProps, savePlotProps, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

/** Default marker colors per contact type (a stored color overrides these). */
const CONTACT_COLORS: Record<string, string> = {
  OWC: "#2f6fed",
  GWC: "#e0483d",
  GOC: "#e08a1e",
  GDT: "#b58a2b",
  ODT: "#3b7a57",
  FWL: "#8e44ad",
};
const CONTACT_TYPES = ["OWC", "GWC", "GOC", "GDT", "ODT", "FWL"];

/** Multi-well correlation view (well-correlation view): the included
 *  wells drawn as side-by-side strips of one shared curve, formation tops connected
 *  between adjacent wells, optionally flattened on a datum top. */

export interface CorrelationOptions {
  curve: string;
  /** Shared strip scale; null = auto (global P2–P98 across the included wells). */
  min: number | null;
  max: number | null;
  /** Top name to flatten on; "" = measured depth. */
  datum: string;
  /** Depth axis: measured depth, or true vertical depth subsea (contacts are flat in TVDSS). */
  depthMode: "md" | "tvdss";
  /** Draw fluid contacts (OWC/GWC/…) as horizontal lines across the strips. */
  showContacts: boolean;
}

export const DEFAULT_CORRELATION_OPTIONS: CorrelationOptions = {
  curve: "GR",
  min: 0,
  max: 150,
  datum: "",
  depthMode: "md",
  showContacts: true,
};

/** Finite (MD, TVDSS) pairs for one well, ascending in MD (hence in TVDSS). */
interface TvdssMap {
  md: Float64Array;
  ss: Float64Array;
}

interface WellStrip {
  well: WellSummary;
  series: TrackCurveSeries | null;
  tops: TopEntry[];
  /** MD→TVDSS lookup built from the well's TVDSS curve; null → treat MD as TVDSS (vertical well). */
  tv: TvdssMap | null;
  /** Display depth = displayOf(MD) - shift (flattening); 0 when the well lacks the datum top. */
  shift: number;
  hasDatum: boolean;
}

/** Linear interpolation on an ascending x-grid, clamped flat beyond the ends. */
function interpAsc(xs: Float64Array, ys: Float64Array, x: number): number {
  const n = xs.length;
  if (n === 0) return x;
  if (x <= xs[0]) return ys[0];
  if (x >= xs[n - 1]) return ys[n - 1];
  let lo = 0;
  let hi = n - 1;
  while (hi - lo > 1) {
    const m = (lo + hi) >> 1;
    if (xs[m] <= x) lo = m;
    else hi = m;
  }
  const t = (x - xs[lo]) / (xs[hi] - xs[lo]);
  return ys[lo] + t * (ys[hi] - ys[lo]);
}

/** Builds the finite (MD, TVDSS) lookup for a well from its TVDSS curve, or null if unusable. */
function buildTvdssMap(series: TrackCurveSeries | null): TvdssMap | null {
  if (!series) return null;
  const md: number[] = [];
  const ss: number[] = [];
  for (let i = 0; i < series.depth.length; i++) {
    const d = series.depth[i];
    const v = series.value[i];
    if (Number.isFinite(d) && Number.isFinite(v)) {
      md.push(d);
      ss.push(v);
    }
  }
  return md.length >= 2 ? { md: Float64Array.from(md), ss: Float64Array.from(ss) } : null;
}

const AXIS_W = 52;
const HEADER_H = 30;

export async function buildCorrelationContent(
  _well: WellSummary | null,
  setStatus: (text: string) => void,
): Promise<PlotContent> {
  const opts: CorrelationOptions = { ...DEFAULT_CORRELATION_OPTIONS, ...(await loadPlotProps<CorrelationOptions>("correlation")) };
  const persist = () => savePlotProps("correlation", opts);

  let wells: WellSummary[] = [];
  try {
    wells = filterByActiveGroup(await listWells());
  } catch {
    wells = [];
  }
  const included = new Set(wells.map((w) => w.well_id));
  let strips: WellStrip[] = [];
  let curveNames: string[] = [];
  try {
    curveNames = await loadCurveNames();
  } catch {
    curveNames = [opts.curve];
  }

  // --- DOM scaffold ---
  const el = document.createElement("div");
  el.className = "correlation-panel";
  const props = document.createElement("div");
  props.className = "plot-props";
  const canvasHost = document.createElement("div");
  canvasHost.className = "correlation-canvas-host";
  const canvas = document.createElement("canvas");
  canvas.className = "plot-canvas";
  canvasHost.appendChild(canvas);
  el.appendChild(props);
  el.appendChild(canvasHost);

  // --- View state (display-depth space) ---
  let viewTop = 0;
  let pxPerUnit = 1;
  let hoverY: number | null = null;
  /** All fluid contacts in the project; each strip renders the ones that apply to it. */
  let contacts: FluidContact[] = [];

  // --- Depth-mode helpers: measured depth vs TVDSS -----------------------------------------
  /** MD → TVDSS via the well's TVDSS curve (identity when the well has none — vertical well). */
  const mdToTvdss = (s: WellStrip, md: number): number => (s.tv ? interpAsc(s.tv.md, s.tv.ss, md) : md);
  /** TVDSS → MD (inverse of the above; TVDSS rises monotonically with MD). */
  const tvdssToMd = (s: WellStrip, ss: number): number => (s.tv ? interpAsc(s.tv.ss, s.tv.md, ss) : ss);
  /** Raw display depth (before flattening) for a measured depth, in the active depth mode. */
  const displayOf = (s: WellStrip, md: number): number => (opts.depthMode === "tvdss" ? mdToTvdss(s, md) : md);
  /** A contact's display depth (after flattening) inside one strip. A TVDSS contact in TVDSS
   *  mode round-trips back to its own depth for every well → the line is perfectly flat. */
  const contactDisplay = (s: WellStrip, c: FluidContact): number => {
    const md = c.is_tvdss ? tvdssToMd(s, c.depth) : c.depth;
    return displayOf(s, md) - s.shift;
  };
  /** Whether a contact applies to a well: explicit well, else field, else global. */
  const contactApplies = (c: FluidContact, well: WellSummary): boolean => {
    if (c.well_id) return c.well_id === well.well_id;
    if (c.field_name) return c.field_name === well.field_name;
    return true;
  };
  const contactColor = (c: FluidContact): string => c.color || CONTACT_COLORS[c.contact_type] || "#888";

  const displayExtent = (): [number, number] => {
    let lo = Infinity;
    let hi = -Infinity;
    for (const s of strips) {
      if (!s.series || s.series.depth.length === 0) continue;
      const a = displayOf(s, s.series.depth[0]) - s.shift;
      const b = displayOf(s, s.series.depth[s.series.depth.length - 1]) - s.shift;
      lo = Math.min(lo, a, b);
      hi = Math.max(hi, a, b);
    }
    return lo < hi ? [lo, hi] : [0, 100];
  };

  const fit = () => {
    const [lo, hi] = displayExtent();
    const h = Math.max(50, canvas.clientHeight - HEADER_H);
    pxPerUnit = h / (hi - lo);
    viewTop = lo;
    draw();
  };

  /** Global strip scale: manual values, else P2–P98 pooled over every included well. */
  const stripScale = (): [number, number] => {
    if (opts.min !== null && opts.max !== null && opts.max !== opts.min) return [opts.min, opts.max];
    const pool: number[] = [];
    for (const s of strips) {
      if (!s.series) continue;
      for (let i = 0; i < s.series.value.length; i++) {
        const v = s.series.value[i];
        if (Number.isFinite(v)) pool.push(v);
      }
    }
    if (pool.length < 2) return [0, 1];
    const lo = percentile(pool, 2);
    const hi = percentile(pool, 98);
    return hi > lo ? [lo, hi] : [lo, lo + 1];
  };

  /** Nice tick step so depth labels sit ≥ 45px apart. */
  const tickStep = (): number => {
    const target = 45 / pxPerUnit;
    const pow = Math.pow(10, Math.floor(Math.log10(target)));
    for (const m of [1, 2, 5, 10]) {
      if (m * pow >= target) return m * pow;
    }
    return 10 * pow;
  };

  function draw(): void {
    const dpr = window.devicePixelRatio || 1;
    const w = canvasHost.clientWidth;
    const h = canvasHost.clientHeight;
    if (w === 0 || h === 0) return;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const theme = readTheme(el);
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);

    const active = strips.filter((s) => included.has(s.well.well_id));
    if (active.length === 0) {
      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 13);
      ctx.fillText("No wells included — pick some under Wells…", AXIS_W + 10, 40);
      return;
    }

    const plotH = h - HEADER_H;
    const yOf = (disp: number) => HEADER_H + (disp - viewTop) * pxPerUnit;
    const [vMin, vMax] = stripScale();
    const slot = (w - AXIS_W) / active.length;
    const gap = Math.min(46, slot * 0.28);
    const stripW = slot - gap;
    const stripLeft = (i: number) => AXIS_W + i * slot + gap / 2;

    // Depth axis (display depth: MD, or relative to the datum when flattened).
    ctx.strokeStyle = theme.grid;
    ctx.fillStyle = theme.text;
    ctx.font = canvasFont(theme, 10);
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    const step = tickStep();
    const first = Math.ceil(viewTop / step) * step;
    for (let d = first; yOf(d) < h; d += step) {
      const y = yOf(d);
      if (y < HEADER_H) continue;
      ctx.beginPath();
      ctx.moveTo(AXIS_W - 4, y);
      ctx.lineTo(w, y);
      ctx.globalAlpha = 0.35;
      ctx.stroke();
      ctx.globalAlpha = 1;
      ctx.fillText(String(Math.round(d)), AXIS_W - 7, y);
    }

    // Flattening datum line at display depth 0.
    if (opts.datum) {
      const y = yOf(0);
      if (y >= HEADER_H && y <= h) {
        ctx.strokeStyle = theme.accent;
        ctx.setLineDash([6, 4]);
        ctx.beginPath();
        ctx.moveTo(AXIS_W, y);
        ctx.lineTo(w, y);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }

    // Strips: frame, header, curve.
    active.forEach((s, i) => {
      const left = stripLeft(i);
      ctx.strokeStyle = theme.axis;
      ctx.strokeRect(left, HEADER_H, stripW, plotH);

      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 11, 600);
      ctx.textAlign = "center";
      ctx.textBaseline = "alphabetic";
      const label = opts.datum && !s.hasDatum ? `${s.well.well_name} (no datum)` : s.well.well_name;
      ctx.fillText(label, left + stripW / 2, 12, stripW + gap - 6);
      ctx.fillStyle = theme.text;
      ctx.font = canvasFont(theme, 10);
      ctx.fillText(opts.curve, left + stripW / 2, 24, stripW - 4);

      if (!s.series || s.series.depth.length === 0) return;
      ctx.save();
      ctx.beginPath();
      ctx.rect(left, HEADER_H, stripW, plotH);
      ctx.clip();
      ctx.strokeStyle = theme.accent2;
      ctx.lineWidth = 1;
      ctx.beginPath();
      let pen = false;
      for (let k = 0; k < s.series.depth.length; k++) {
        const v = s.series.value[k];
        if (!Number.isFinite(v)) {
          pen = false;
          continue;
        }
        const frac = Math.min(1, Math.max(0, (v - vMin) / (vMax - vMin)));
        const x = left + frac * stripW;
        const y = yOf(displayOf(s, s.series.depth[k]) - s.shift);
        if (pen) ctx.lineTo(x, y);
        else ctx.moveTo(x, y);
        pen = true;
      }
      ctx.stroke();
      ctx.restore();
    });

    // Tops: marker line inside each strip, then connectors between adjacent wells.
    const topY = (s: WellStrip, name: string): number | null => {
      const top = s.tops.find((t) => t.top_name === name);
      if (!top) return null;
      const y = yOf(displayOf(s, top.depth) - s.shift);
      return y >= HEADER_H && y <= h ? y : null;
    };
    const allTopNames = Array.from(new Set(active.flatMap((s) => s.tops.map((t) => t.top_name))));
    ctx.lineWidth = 1.5;
    ctx.font = canvasFont(theme, 10);
    ctx.textBaseline = "bottom";
    for (const name of allTopNames) {
      const color = active.flatMap((s) => s.tops).find((t) => t.top_name === name)?.color || theme.warn;
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      let labeled = false;
      active.forEach((s, i) => {
        const y = topY(s, name);
        if (y === null) return;
        const left = stripLeft(i);
        ctx.beginPath();
        ctx.moveTo(left, y);
        ctx.lineTo(left + stripW, y);
        ctx.stroke();
        if (!labeled) {
          ctx.textAlign = "left";
          ctx.fillText(name, left + 2, y - 1);
          labeled = true;
        }
      });
      // Dashed connectors bridge the gaps between adjacent strips that both have the top.
      ctx.setLineDash([4, 3]);
      for (let i = 0; i + 1 < active.length; i++) {
        const y1 = topY(active[i], name);
        const y2 = topY(active[i + 1], name);
        if (y1 === null || y2 === null) continue;
        ctx.beginPath();
        ctx.moveTo(stripLeft(i) + stripW, y1);
        ctx.lineTo(stripLeft(i + 1), y2);
        ctx.stroke();
      }
      ctx.setLineDash([]);
    }

    // Fluid contacts: solid horizontal markers across each applicable strip, with a small
    // triangle at the left edge and dashed cross-well connectors. A TVDSS contact drawn in
    // TVDSS mode round-trips to its own depth in every well, so its line is perfectly flat.
    if (opts.showContacts && contacts.length) {
      ctx.font = canvasFont(theme, 10, 600);
      ctx.textBaseline = "bottom";
      for (const c of contacts) {
        if (!active.some((s) => contactApplies(c, s.well))) continue;
        const color = contactColor(c);
        ctx.strokeStyle = color;
        ctx.fillStyle = color;
        const ys = active.map((s) => {
          if (!contactApplies(c, s.well)) return null;
          const y = yOf(contactDisplay(s, c));
          return y >= HEADER_H && y <= h ? y : null;
        });
        let labeled = false;
        active.forEach((_s, i) => {
          const y = ys[i];
          if (y === null) return;
          const left = stripLeft(i);
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(left, y);
          ctx.lineTo(left + stripW, y);
          ctx.stroke();
          // Left-edge triangle marker distinguishes contacts from tops.
          ctx.beginPath();
          ctx.moveTo(left, y - 4);
          ctx.lineTo(left, y + 4);
          ctx.lineTo(left + 7, y);
          ctx.closePath();
          ctx.fill();
          if (!labeled) {
            ctx.textAlign = "left";
            const lbl = c.label || `${c.contact_type} ${Math.round(c.depth)}${c.is_tvdss ? "ss" : ""}`;
            ctx.fillText(lbl, left + 9, y - 1);
            labeled = true;
          }
        });
        ctx.setLineDash([5, 3]);
        ctx.lineWidth = 1.2;
        for (let i = 0; i + 1 < active.length; i++) {
          const y1 = ys[i];
          const y2 = ys[i + 1];
          if (y1 === null || y2 === null) continue;
          ctx.beginPath();
          ctx.moveTo(stripLeft(i) + stripW, y1);
          ctx.lineTo(stripLeft(i + 1), y2);
          ctx.stroke();
        }
        ctx.setLineDash([]);
      }
    }

    // Hover crosshair.
    if (hoverY !== null && hoverY > HEADER_H) {
      ctx.strokeStyle = theme.accent;
      ctx.globalAlpha = 0.75;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(AXIS_W, hoverY);
      ctx.lineTo(w, hoverY);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
    }
  }

  // --- Data loading ---
  // Monotonic token: a rapid well-toggle / curve change / dataVersion bump can leave an
  // older Promise.all in flight; whichever reload started last wins, so a stale set of
  // strips can't replace the current one. reload() preserves the pan/zoom viewport.
  let reloadGen = 0;
  async function reload(): Promise<void> {
    const gen = ++reloadGen;
    const chosen = wells.filter((w) => included.has(w.well_id));
    // TVDSS rides along in the same batch read so a TVDSS-mode switch needs no refetch.
    const names = Array.from(new Set([opts.curve, "TVDSS"]));
    const [loaded, loadedContacts] = await Promise.all([
      Promise.all(
        chosen.map(async (well): Promise<WellStrip> => {
          let series: TrackCurveSeries | null = null;
          let tv: TvdssMap | null = null;
          let tops: TopEntry[] = [];
          try {
            const data = await getTrackData(well.well_id, names, 1400);
            series = data.find((s) => s.curve_name === opts.curve) ?? null;
            tv = buildTvdssMap(data.find((s) => s.curve_name === "TVDSS") ?? null);
          } catch {
            series = null;
          }
          try {
            tops = await listTops(well.well_id);
          } catch {
            tops = [];
          }
          return { well, series, tops, tv, shift: 0, hasDatum: false };
        }),
      ),
      listFluidContacts().catch(() => [] as FluidContact[]),
    ]);
    if (gen !== reloadGen) return; // a newer reload started while we awaited
    strips = loaded;
    contacts = loadedContacts;
    applyDatum();
    refreshDatumChoices();
  }

  /** Re-fetches the well list so the Wells menu and strips track the current project after
   *  an import, delete, or active-group change — reload() alone only re-reads curves for the
   *  wells already included, so a freshly imported well never appeared. New wells join the
   *  included set (they show as strips immediately); wells that no longer exist drop out. */
  async function refreshWells(): Promise<void> {
    let latest: WellSummary[];
    try {
      latest = filterByActiveGroup(await listWells());
    } catch {
      return; // keep the current list if the fetch fails
    }
    const known = new Set(wells.map((w) => w.well_id));
    const live = new Set(latest.map((w) => w.well_id));
    for (const w of latest) if (!known.has(w.well_id)) included.add(w.well_id);
    for (const id of Array.from(included)) if (!live.has(id)) included.delete(id);
    wells = latest;
    refreshWellsBtn();
  }

  /** Recomputes per-well flattening shifts from the chosen datum top. */
  function applyDatum(): void {
    for (const s of strips) {
      const top = opts.datum ? s.tops.find((t) => t.top_name === opts.datum) : undefined;
      s.hasDatum = !!top;
      // Shift is in display space, so re-derive it whenever the depth mode changes too.
      s.shift = top ? displayOf(s, top.depth) : 0;
    }
  }

  // --- Property row ---
  const wellsBtn = document.createElement("button");
  wellsBtn.className = "form-control";
  const refreshWellsBtn = () => {
    wellsBtn.textContent = `Wells (${included.size}/${wells.length})…`;
  };
  refreshWellsBtn();
  wellsBtn.addEventListener("click", () => {
    const menu = document.createElement("div");
    menu.className = "dock-add-menu";
    const rect = wellsBtn.getBoundingClientRect();
    menu.style.left = `${rect.left}px`;
    menu.style.top = `${rect.bottom + 2}px`;
    for (const well of wells) {
      const row = document.createElement("label");
      row.className = "well-check";
      const box = document.createElement("input");
      box.type = "checkbox";
      box.checked = included.has(well.well_id);
      box.addEventListener("change", () => {
        if (box.checked) included.add(well.well_id);
        else included.delete(well.well_id);
        refreshWellsBtn();
        void reload().then(draw);
      });
      row.appendChild(box);
      row.appendChild(document.createTextNode(well.well_name));
      menu.appendChild(row);
    }
    document.body.appendChild(menu);
    const close = (e: MouseEvent) => {
      if (!menu.contains(e.target as Node) && e.target !== wellsBtn) {
        menu.remove();
        document.removeEventListener("mousedown", close);
      }
    };
    setTimeout(() => document.addEventListener("mousedown", close), 0);
  });

  const curveSel = curveSelect(curveNames, opts.curve);
  curveSel.addEventListener("change", () => {
    opts.curve = curveSel.value;
    persist();
    void reload().then(draw);
  });

  const numField = (placeholder: string, value: number | null, onChange: (v: number | null) => void): HTMLInputElement => {
    const input = document.createElement("input");
    input.type = "number";
    input.className = "form-control num-field";
    input.placeholder = placeholder;
    if (value !== null) input.value = String(value);
    input.addEventListener("change", () => {
      const v = input.value.trim() === "" ? null : Number(input.value);
      onChange(v !== null && Number.isFinite(v) ? v : null);
      persist();
      draw();
    });
    return input;
  };

  const datumSel = document.createElement("select");
  datumSel.className = "form-control";
  function refreshDatumChoices(): void {
    const names = Array.from(new Set(strips.flatMap((s) => s.tops.map((t) => t.top_name)))).sort();
    datumSel.innerHTML = "";
    const md = document.createElement("option");
    md.value = "";
    md.textContent = "Measured depth";
    datumSel.appendChild(md);
    for (const name of names) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = `Flatten on ${name}`;
      datumSel.appendChild(option);
    }
    datumSel.value = names.includes(opts.datum) ? opts.datum : "";
  }
  datumSel.addEventListener("change", () => {
    opts.datum = datumSel.value;
    persist();
    applyDatum();
    fit();
  });

  const mkBtn = (label: string, title: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.className = "form-control";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };

  const depthModeSel = document.createElement("select");
  depthModeSel.className = "form-control";
  depthModeSel.title = "Depth axis — measured depth, or TVDSS (fluid contacts are flat in TVDSS)";
  for (const [val, lbl] of [
    ["md", "MD"],
    ["tvdss", "TVDSS"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = lbl;
    depthModeSel.appendChild(o);
  }
  depthModeSel.value = opts.depthMode;
  depthModeSel.addEventListener("change", () => {
    opts.depthMode = depthModeSel.value === "tvdss" ? "tvdss" : "md";
    persist();
    applyDatum(); // shift is in display space → re-derive for the new mode
    fit();
  });

  // --- Fluid-contacts editor ---
  const scopeValue = (c: FluidContact): string =>
    c.well_id ? `well:${c.well_id}` : c.field_name ? `field:${c.field_name}` : "";
  const applyScope = (c: FluidContact, value: string): void => {
    if (value.startsWith("well:")) {
      c.well_id = value.slice(5);
      c.field_name = null;
    } else if (value.startsWith("field:")) {
      c.field_name = value.slice(6);
      c.well_id = null;
    } else {
      c.well_id = null;
      c.field_name = null;
    }
  };

  function openContactsEditor(): void {
    const body = document.createElement("div");
    body.className = "contacts-editor";

    const showRow = document.createElement("label");
    showRow.className = "contacts-show";
    const showBox = document.createElement("input");
    showBox.type = "checkbox";
    showBox.checked = opts.showContacts;
    showBox.addEventListener("change", () => {
      opts.showContacts = showBox.checked;
      persist();
      draw();
    });
    showRow.append(showBox, document.createTextNode(" Show contacts in the view"));
    body.appendChild(showRow);

    const table = document.createElement("div");
    table.className = "contacts-table";
    body.appendChild(table);

    const fields = Array.from(new Set(wells.map((w) => w.field_name).filter((f): f is string => !!f))).sort();

    const save = async (c: FluidContact): Promise<void> => {
      await upsertFluidContact(c).catch((e) => setStatus(`Contact save failed: ${e}`));
      draw();
    };

    const renderRows = (): void => {
      table.innerHTML = "";
      if (!contacts.length) {
        const empty = document.createElement("div");
        empty.className = "contacts-empty";
        empty.textContent = "No fluid contacts yet — add one below.";
        table.appendChild(empty);
      }
      for (const c of contacts) {
        const row = document.createElement("div");
        row.className = "contacts-row";

        const typeSel = document.createElement("select");
        typeSel.className = "form-control";
        for (const t of CONTACT_TYPES) {
          const o = document.createElement("option");
          o.value = t;
          o.textContent = t;
          typeSel.appendChild(o);
        }
        if (!CONTACT_TYPES.includes(c.contact_type)) {
          const o = document.createElement("option");
          o.value = c.contact_type;
          o.textContent = c.contact_type;
          typeSel.appendChild(o);
        }
        typeSel.value = c.contact_type;
        typeSel.addEventListener("change", () => {
          c.contact_type = typeSel.value;
          void save(c);
        });

        const depthInput = document.createElement("input");
        depthInput.type = "number";
        depthInput.className = "form-control num-field";
        depthInput.value = String(c.depth);
        depthInput.addEventListener("change", () => {
          const v = Number(depthInput.value);
          if (Number.isFinite(v)) {
            c.depth = v;
            void save(c);
          }
        });

        const ssLabel = document.createElement("label");
        ssLabel.className = "contacts-ss";
        const ssBox = document.createElement("input");
        ssBox.type = "checkbox";
        ssBox.checked = c.is_tvdss;
        ssBox.addEventListener("change", () => {
          c.is_tvdss = ssBox.checked;
          void save(c);
        });
        ssLabel.append(ssBox, document.createTextNode(" TVDSS"));

        const scopeSel = document.createElement("select");
        scopeSel.className = "form-control";
        const gOpt = document.createElement("option");
        gOpt.value = "";
        gOpt.textContent = "All wells";
        scopeSel.appendChild(gOpt);
        for (const f of fields) {
          const o = document.createElement("option");
          o.value = `field:${f}`;
          o.textContent = `Field: ${f}`;
          scopeSel.appendChild(o);
        }
        for (const w of wells) {
          const o = document.createElement("option");
          o.value = `well:${w.well_id}`;
          o.textContent = `Well: ${w.well_name}`;
          scopeSel.appendChild(o);
        }
        scopeSel.value = scopeValue(c);
        scopeSel.addEventListener("change", () => {
          applyScope(c, scopeSel.value);
          void save(c);
        });

        const colorInput = document.createElement("input");
        colorInput.type = "color";
        colorInput.className = "contacts-color";
        colorInput.value = contactColor(c);
        colorInput.title = "Marker color";
        colorInput.addEventListener("change", () => {
          c.color = colorInput.value;
          void save(c);
        });

        const del = document.createElement("button");
        del.className = "form-control contacts-del";
        del.textContent = "✕";
        del.title = "Delete contact";
        del.addEventListener("click", () => {
          void deleteFluidContact(c.contact_id).then(() => {
            contacts = contacts.filter((x) => x.contact_id !== c.contact_id);
            renderRows();
            draw();
          });
        });

        row.append(typeSel, depthInput, ssLabel, scopeSel, colorInput, del);
        table.appendChild(row);
      }
    };

    const addBtn = document.createElement("button");
    addBtn.className = "form-control contacts-add";
    addBtn.textContent = "＋ Add contact";
    addBtn.addEventListener("click", () => {
      const c: FluidContact = {
        contact_id: crypto.randomUUID(),
        field_name: null,
        well_id: null,
        contact_type: "OWC",
        depth: Math.round(viewTop + 50),
        is_tvdss: opts.depthMode === "tvdss",
        color: null,
        label: null,
      };
      contacts.push(c);
      void upsertFluidContact(c).then(() => {
        renderRows();
        draw();
      });
    });
    body.appendChild(addBtn);

    renderRows();
    openModal("Fluid contacts", body, 640);
  }

  props.appendChild(wellsBtn);
  props.appendChild(curveSel);
  props.appendChild(numField("min", opts.min, (v) => (opts.min = v)));
  props.appendChild(numField("max", opts.max, (v) => (opts.max = v)));
  props.appendChild(datumSel);
  props.appendChild(depthModeSel);
  props.appendChild(mkBtn("Contacts…", "Add / edit fluid contacts (OWC, GWC, …)", openContactsEditor));
  props.appendChild(mkBtn("Fit", "Fit all wells vertically", fit));
  props.appendChild(mkBtn("＋", "Zoom in", () => {
    zoomAtCenter(1.25);
  }));
  props.appendChild(mkBtn("−", "Zoom out", () => {
    zoomAtCenter(1 / 1.25);
  }));
  props.appendChild(buildImageExportButtons(() => canvas, "Correlation", setStatus));

  function zoomAtCenter(factor: number): void {
    const plotH = Math.max(50, canvas.clientHeight - HEADER_H);
    const mid = viewTop + plotH / 2 / pxPerUnit;
    pxPerUnit *= factor;
    viewTop = mid - plotH / 2 / pxPerUnit;
    draw();
  }

  // --- Interactions: wheel/drag pan, hover broadcast ---
  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      if (e.ctrlKey || e.metaKey) {
        // Ctrl/Cmd+wheel zooms about the cursor depth — same convention (and factors) as
        // attachZoomPan on the other plots (in = shrink the depth window). Plain wheel keeps
        // panning through depth (there's no competing page scroll inside a dock pane).
        const rect = canvas.getBoundingClientRect();
        const y = Math.max(HEADER_H, e.clientY - rect.top);
        const anchor = viewTop + (y - HEADER_H) / pxPerUnit;
        const f = e.deltaY < 0 ? 0.83 : 1.2;
        pxPerUnit /= f;
        viewTop = anchor - (y - HEADER_H) / pxPerUnit;
      } else {
        viewTop += e.deltaY / pxPerUnit;
      }
      draw();
    },
    { passive: false },
  );
  let dragging = false;
  let lastY = 0;
  canvas.addEventListener("pointerdown", (e) => {
    dragging = true;
    lastY = e.clientY;
  });
  window.addEventListener("pointerup", () => (dragging = false));
  canvas.addEventListener("pointermove", (e) => {
    const rect = canvas.getBoundingClientRect();
    const y = e.clientY - rect.top;
    if (dragging) {
      viewTop -= (e.clientY - lastY) / pxPerUnit;
      lastY = e.clientY;
    }
    hoverY = y;
    // Broadcast the hovered STRIP's measured depth so the well's other views sync.
    const active = strips.filter((s) => included.has(s.well.well_id));
    const x = e.clientX - rect.left;
    const slot = (rect.width - AXIS_W) / Math.max(1, active.length);
    const idx = Math.floor((x - AXIS_W) / slot);
    const disp = viewTop + (y - HEADER_H) / pxPerUnit;
    if (idx >= 0 && idx < active.length && y > HEADER_H) {
      const s = active[idx];
      const unflattened = disp + s.shift; // display depth without flattening
      // Other views expect measured depth, so undo the TVDSS mapping before broadcasting.
      appState.hoverDepth.set(opts.depthMode === "tvdss" ? tvdssToMd(s, unflattened) : unflattened);
    } else {
      appState.hoverDepth.set(null);
    }
    draw();
  });
  canvas.addEventListener("pointerleave", () => {
    hoverY = null;
    appState.hoverDepth.set(null);
    draw();
  });

  const resizeObserver = new ResizeObserver(() => draw());
  resizeObserver.observe(canvasHost);
  // The primed flag drops subscribe's immediate fire so the trailing `await reload()`
  // below stays the only build-time load (no double fetch at construction).
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true;
      return;
    }
    // Refresh the well list first (imports/deletions/group change), THEN re-read curves —
    // reload() alone left a stale Wells menu that never showed a newly imported well.
    void refreshWells()
      .then(() => reload())
      .then(draw);
  });
  // Colours come from CSS vars at draw time; a theme switch only needs a repaint.
  const unsubTheme = appState.themeVersion.subscribe(() => draw());

  try {
    await reload();
  } catch (err) {
    setStatus(`Correlation load failed: ${err}`);
  }
  // Initial fit once the panel has a size (dock lays out after mount).
  setTimeout(fit, 50);

  return {
    el,
    dispose: () => {
      resizeObserver.disconnect();
      unsubData();
      unsubTheme();
    },
  };
}
