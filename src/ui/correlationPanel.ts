import { getTrackData, listTops, listWells, type TopEntry, type TrackCurveSeries, type WellSummary } from "../ipc";
import { appState, filterByActiveGroup } from "../state";
import { percentile, readTheme } from "./plotCanvas";
import { curveSelect, loadCurveNames, loadPlotProps, savePlotProps, type PlotContent } from "./plotCommon";
import { buildImageExportButtons } from "./plotExport";

/** Multi-well correlation view (Geolog Well Correlation equivalent): the included
 *  wells drawn as side-by-side strips of one shared curve, formation tops connected
 *  between adjacent wells, optionally flattened on a datum top. */

export interface CorrelationOptions {
  curve: string;
  /** Shared strip scale; null = auto (global P2–P98 across the included wells). */
  min: number | null;
  max: number | null;
  /** Top name to flatten on; "" = measured depth. */
  datum: string;
}

export const DEFAULT_CORRELATION_OPTIONS: CorrelationOptions = {
  curve: "GR",
  min: 0,
  max: 150,
  datum: "",
};

interface WellStrip {
  well: WellSummary;
  series: TrackCurveSeries | null;
  tops: TopEntry[];
  /** Display depth = MD - shift (flattening); 0 when the well lacks the datum top. */
  shift: number;
  hasDatum: boolean;
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

  const displayExtent = (): [number, number] => {
    let lo = Infinity;
    let hi = -Infinity;
    for (const s of strips) {
      if (!s.series || s.series.depth.length === 0) continue;
      lo = Math.min(lo, s.series.depth[0] - s.shift);
      hi = Math.max(hi, s.series.depth[s.series.depth.length - 1] - s.shift);
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
      ctx.font = "500 13px system-ui";
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
    ctx.font = "500 10px system-ui";
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
      ctx.font = "600 11px system-ui";
      ctx.textAlign = "center";
      ctx.textBaseline = "alphabetic";
      const label = opts.datum && !s.hasDatum ? `${s.well.well_name} (no datum)` : s.well.well_name;
      ctx.fillText(label, left + stripW / 2, 12, stripW + gap - 6);
      ctx.fillStyle = theme.text;
      ctx.font = "500 10px system-ui";
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
        const y = yOf(s.series.depth[k] - s.shift);
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
      const y = yOf(top.depth - s.shift);
      return y >= HEADER_H && y <= h ? y : null;
    };
    const allTopNames = Array.from(new Set(active.flatMap((s) => s.tops.map((t) => t.top_name))));
    ctx.lineWidth = 1.5;
    ctx.font = "500 10px system-ui";
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
    const loaded = await Promise.all(
      chosen.map(async (well): Promise<WellStrip> => {
        let series: TrackCurveSeries | null = null;
        let tops: TopEntry[] = [];
        try {
          const data = await getTrackData(well.well_id, [opts.curve], 1400);
          series = data.find((s) => s.curve_name === opts.curve) ?? null;
        } catch {
          series = null;
        }
        try {
          tops = await listTops(well.well_id);
        } catch {
          tops = [];
        }
        return { well, series, tops, shift: 0, hasDatum: false };
      }),
    );
    if (gen !== reloadGen) return; // a newer reload started while we awaited
    strips = loaded;
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
      s.shift = top ? top.depth : 0;
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

  props.appendChild(wellsBtn);
  props.appendChild(curveSel);
  props.appendChild(numField("min", opts.min, (v) => (opts.min = v)));
  props.appendChild(numField("max", opts.max, (v) => (opts.max = v)));
  props.appendChild(datumSel);
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
      appState.hoverDepth.set(disp + active[idx].shift);
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
