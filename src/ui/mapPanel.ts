/** Field Map pane (Wave E item 22): plots wells at their surface easting/northing and lets
 *  the user draw an editable polygon to select the enclosed wells and drop them into a
 *  well group. Coordinates are raw UTM metres, so the map draws in that space directly (a
 *  single field shares one zone); multi-zone projects are flagged rather than reprojected.
 *
 *  Interaction:
 *   - Pan mode (default): drag to pan, wheel to zoom about the cursor. With a polygon
 *     present, its vertices are draggable handles — dragging re-selects wells live.
 *   - Draw mode ("Draw polygon"): click to drop vertices, click the first vertex again (or
 *     double-click / Enter) to close; Esc cancels. Closing returns to pan/edit mode. */

import { canvasFont, fitCanvasBackingStore, readTheme } from "./plotCanvas";
import { createWellGroup, listWellGroups, listWells, setWellGroupMembers, wellsInPolygon } from "../ipc";
import { recordProcess } from "../processLog";
import { appState, bumpWellGroupsVersion } from "../state";
import { syncWellGroups } from "./wellGroups";
import { formRow, openModal } from "./modal";

interface MapWell {
  id: string;
  name: string;
  x: number;
  y: number;
  zone: string | null;
}

type Vec2 = [number, number];

/** Ray-casting point-in-polygon (mirrors the Rust `geo::point_in_polygon` used for the
 *  authoritative commit) so the live selection highlight is instant while drawing. */
function pointInPolygon(px: number, py: number, poly: Vec2[]): boolean {
  if (poly.length < 3) return false;
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i];
    const [xj, yj] = poly[j];
    if (yi > py !== yj > py) {
      const xInt = xi + ((py - yi) / (yj - yi)) * (xj - xi);
      if (px < xInt) inside = !inside;
    }
  }
  return inside;
}

/** A "nice" round number ≤ target, from the 1/2/5 × 10ⁿ series (scale bar + grid step). */
function niceStep(target: number): number {
  if (!(target > 0)) return 1;
  const pow = Math.pow(10, Math.floor(Math.log10(target)));
  for (const m of [5, 2, 1]) if (m * pow <= target) return m * pow;
  return pow;
}

function fmtDistance(m: number): string {
  return m >= 1000 ? `${(m / 1000).toLocaleString(undefined, { maximumFractionDigits: 1 })} km` : `${Math.round(m)} m`;
}

export async function buildMapContent(
  setStatus: (t: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const el = document.createElement("div");
  el.className = "map-pane";

  // --- Toolbar ---------------------------------------------------------------
  const toolbar = document.createElement("div");
  toolbar.className = "map-toolbar";
  const drawBtn = mkBtn("✏ Draw polygon", "Draw a selection polygon (click vertices, click the first again to close)");
  const clearBtn = mkBtn("Clear", "Remove the polygon");
  const assignBtn = mkBtn("Assign to group…", "Put the enclosed wells into a well group");
  const fitBtn = mkBtn("Fit", "Zoom to show every located well");
  const info = document.createElement("span");
  info.className = "map-info";
  toolbar.append(drawBtn, clearBtn, assignBtn, fitBtn, info);
  el.appendChild(toolbar);

  // --- Canvas ----------------------------------------------------------------
  const host = document.createElement("div");
  host.className = "map-canvas-host";
  const canvas = document.createElement("canvas");
  canvas.className = "map-canvas";
  host.appendChild(canvas);
  el.appendChild(host);
  const empty = document.createElement("div");
  empty.className = "map-empty logview-message";
  empty.innerHTML =
    "No wells have surface coordinates yet.<br>Import them from <b>Data ▸ Import Well Locations…</b> " +
    "(a CSV with WELL, EASTING, NORTHING columns), or set X/Y in a well's header.";
  host.appendChild(empty);

  // --- State -----------------------------------------------------------------
  let wells: MapWell[] = [];
  const view = { cx: 0, cy: 0, scale: 1 }; // world centre (metres) + pixels-per-metre
  let poly: Vec2[] = []; // committed ring (world metres); empty = none
  let drawing = false; // actively adding vertices
  let cursorWorld: Vec2 | null = null; // rubber-band endpoint while drawing
  let dragVertex: number | null = null;
  let panning: { x: number; y: number } | null = null;
  let enclosed = new Set<string>();
  let activeIds = new Set<string>();

  const ctx = () => canvas.getContext("2d")!;
  const W = () => canvas.clientWidth || canvas.width;
  const H = () => canvas.clientHeight || canvas.height;
  const toScreen = (wx: number, wy: number): Vec2 => [
    W() / 2 + (wx - view.cx) * view.scale,
    H() / 2 - (wy - view.cy) * view.scale,
  ];
  const toWorld = (sx: number, sy: number): Vec2 => [
    view.cx + (sx - W() / 2) / view.scale,
    view.cy - (sy - H() / 2) / view.scale,
  ];

  const recomputeEnclosed = () => {
    enclosed = new Set<string>();
    if (poly.length >= 3 && !drawing) for (const w of wells) if (pointInPolygon(w.x, w.y, poly)) enclosed.add(w.id);
  };

  const fitView = () => {
    if (!wells.length) return;
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const w of wells) {
      minX = Math.min(minX, w.x); maxX = Math.max(maxX, w.x);
      minY = Math.min(minY, w.y); maxY = Math.max(maxY, w.y);
    }
    view.cx = (minX + maxX) / 2;
    view.cy = (minY + maxY) / 2;
    const pad = 48;
    const spanX = Math.max(maxX - minX, 1);
    const spanY = Math.max(maxY - minY, 1);
    let scale = Math.min((W() - 2 * pad) / spanX, (H() - 2 * pad) / spanY);
    if (!Number.isFinite(scale) || scale <= 0) scale = 0.05;
    view.scale = Math.min(scale, 5); // never blow a single well or tight cluster up past 5 px/m
  };

  const zones = () => [...new Set(wells.map((w) => w.zone).filter((z): z is string => !!z))];

  const updateInfo = () => {
    const zs = zones();
    const zoneLbl = zs.length === 0 ? "no zone set" : zs.length === 1 ? `UTM ${zs[0]}` : `mixed zones (${zs.join(", ")})`;
    const encl = poly.length >= 3 ? ` — polygon encloses ${enclosed.size}` : "";
    info.textContent = `${wells.length} located · ${zoneLbl}${encl}`;
    info.classList.toggle("map-warn", zs.length > 1);
    assignBtn.disabled = !(poly.length >= 3 && enclosed.size > 0);
    clearBtn.disabled = poly.length === 0 && !drawing;
  };

  // --- Rendering -------------------------------------------------------------
  const draw = () => {
    const dpr = fitCanvasBackingStore(canvas);
    const c = ctx();
    const theme = readTheme(canvas);
    const w = W(), h = H();
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    c.fillStyle = theme.bg;
    c.fillRect(0, 0, w, h);
    if (!wells.length) return;

    // Faint coordinate grid, ~90 px spacing, snapped to a nice metre step.
    const step = niceStep(90 / view.scale);
    const [wx0, wy1] = toWorld(0, 0);
    const [wx1, wy0] = toWorld(w, h);
    c.strokeStyle = theme.grid;
    c.fillStyle = theme.text;
    c.lineWidth = 1;
    c.font = canvasFont(theme, 10, 400);
    c.globalAlpha = 0.5;
    for (let gx = Math.ceil(wx0 / step) * step; gx <= wx1; gx += step) {
      const [sx] = toScreen(gx, 0);
      c.beginPath(); c.moveTo(sx, 0); c.lineTo(sx, h); c.stroke();
      c.globalAlpha = 0.8; c.fillText(String(Math.round(gx)), sx + 2, h - 4); c.globalAlpha = 0.5;
    }
    for (let gy = Math.ceil(wy0 / step) * step; gy <= wy1; gy += step) {
      const [, sy] = toScreen(0, gy);
      c.beginPath(); c.moveTo(0, sy); c.lineTo(w, sy); c.stroke();
      c.globalAlpha = 0.8; c.fillText(String(Math.round(gy)), 3, sy - 2); c.globalAlpha = 0.5;
    }
    c.globalAlpha = 1;

    // Polygon (under the wells so markers stay legible).
    if (poly.length) {
      c.strokeStyle = theme.warn;
      c.lineWidth = 1.5;
      c.beginPath();
      poly.forEach(([px, py], i) => {
        const [sx, sy] = toScreen(px, py);
        if (i === 0) c.moveTo(sx, sy); else c.lineTo(sx, sy);
      });
      if (drawing && cursorWorld) {
        const [sx, sy] = toScreen(cursorWorld[0], cursorWorld[1]);
        c.lineTo(sx, sy);
      } else {
        c.closePath();
      }
      c.stroke();
      c.fillStyle = theme.warn;
      c.globalAlpha = 0.08;
      if (!drawing) c.fill();
      c.globalAlpha = 1;
      // Vertex handles (draggable once closed).
      if (!drawing) {
        for (const [px, py] of poly) {
          const [sx, sy] = toScreen(px, py);
          c.fillStyle = theme.bg;
          c.strokeStyle = theme.warn;
          c.fillRect(sx - 3.5, sy - 3.5, 7, 7);
          c.strokeRect(sx - 3.5, sy - 3.5, 7, 7);
        }
      }
    }

    // Wells.
    const showLabels = wells.length <= 80 || view.scale > 0.02;
    c.font = canvasFont(theme, 11, 400);
    for (const wl of wells) {
      const [sx, sy] = toScreen(wl.x, wl.y);
      if (sx < -20 || sx > w + 20 || sy < -20 || sy > h + 20) continue;
      const inEncl = enclosed.has(wl.id);
      const inActive = activeIds.has(wl.id);
      c.beginPath();
      c.arc(sx, sy, inEncl ? 5.5 : 4, 0, Math.PI * 2);
      c.fillStyle = inEncl ? theme.warn : inActive ? theme.accent2 : theme.accent;
      c.fill();
      if (inActive) {
        c.lineWidth = 2;
        c.strokeStyle = theme.accent2;
        c.beginPath();
        c.arc(sx, sy, 7.5, 0, Math.PI * 2);
        c.stroke();
      }
      if (showLabels) {
        c.fillStyle = theme.text;
        c.fillText(wl.name, sx + 7, sy + 3.5);
      }
    }

    // Scale bar (bottom-left).
    const barM = niceStep(140 / view.scale);
    const barPx = barM * view.scale;
    const bx = 16, by = h - 16;
    c.strokeStyle = theme.axis;
    c.fillStyle = theme.text;
    c.lineWidth = 2;
    c.beginPath();
    c.moveTo(bx, by); c.lineTo(bx + barPx, by);
    c.moveTo(bx, by - 4); c.lineTo(bx, by + 4);
    c.moveTo(bx + barPx, by - 4); c.lineTo(bx + barPx, by + 4);
    c.stroke();
    c.font = canvasFont(theme, 10, 400);
    c.fillText(fmtDistance(barM), bx, by - 6);
  };

  const redraw = () => {
    recomputeEnclosed();
    updateInfo();
    draw();
  };

  // --- Data ------------------------------------------------------------------
  // Set once the view has been fit to real (laid-out, non-empty) wells. Guards both the
  // initial open and the case where coordinates first arrive (via import) while the pane is
  // already open — without this a later loadWells(false) would leave markers off-screen
  // under the identity view. Also gated by W() > 0 so a fit at zero canvas width doesn't
  // "count" (the ResizeObserver does the real fit once layout lands).
  let didFit = false;
  const loadWells = async (refit: boolean) => {
    const all = await listWells().catch(() => []);
    wells = all
      .filter((w) => Number.isFinite(w.surface_x ?? NaN) && Number.isFinite(w.surface_y ?? NaN))
      .map((w) => ({ id: w.well_id, name: w.well_name, x: w.surface_x as number, y: w.surface_y as number, zone: w.utm_zone ?? null }));
    activeIds = new Set(appState.activeWellGroup.get()?.well_ids ?? []);
    const hasWells = wells.length > 0;
    host.classList.toggle("map-has-data", hasWells);
    empty.style.display = hasWells ? "none" : "";
    canvas.style.display = hasWells ? "" : "none";
    // Fit on an explicit request, or the first time wells become visible under a real layout.
    if (hasWells && W() > 0 && (refit || !didFit)) {
      didFit = true;
      fitView();
    }
    redraw();
  };

  // --- Pointer interaction ---------------------------------------------------
  const HANDLE_HIT = 8; // px radius to grab a vertex
  const vertexAt = (sx: number, sy: number): number | null => {
    for (let i = 0; i < poly.length; i++) {
      const [px, py] = toScreen(poly[i][0], poly[i][1]);
      if (Math.hypot(px - sx, py - sy) <= HANDLE_HIT) return i;
    }
    return null;
  };
  const localXY = (e: MouseEvent): Vec2 => {
    const r = canvas.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  };

  const onClick = (e: MouseEvent) => {
    if (!drawing) return;
    const [sx, sy] = localXY(e);
    // Click near the first vertex closes the ring (needs ≥3).
    if (poly.length >= 3) {
      const [fx, fy] = toScreen(poly[0][0], poly[0][1]);
      if (Math.hypot(fx - sx, fy - sy) <= HANDLE_HIT * 1.5) {
        closePolygon();
        return;
      }
    }
    poly.push(toWorld(sx, sy));
    redraw();
  };

  const onDblClick = (e: MouseEvent) => {
    if (drawing && poly.length >= 3) {
      e.preventDefault();
      closePolygon();
    }
  };

  const onDown = (e: MouseEvent) => {
    if (drawing) return; // vertices are added on click, not drag
    const [sx, sy] = localXY(e);
    const v = vertexAt(sx, sy);
    if (v !== null) {
      dragVertex = v;
    } else {
      panning = { x: e.clientX, y: e.clientY };
    }
  };

  const onMove = (e: MouseEvent) => {
    const [sx, sy] = localXY(e);
    if (drawing) {
      cursorWorld = toWorld(sx, sy);
      draw();
      return;
    }
    if (dragVertex !== null) {
      poly[dragVertex] = toWorld(sx, sy);
      redraw();
      return;
    }
    if (panning) {
      view.cx -= (e.clientX - panning.x) / view.scale;
      view.cy += (e.clientY - panning.y) / view.scale;
      panning = { x: e.clientX, y: e.clientY };
      draw();
    }
  };

  const onUp = () => {
    dragVertex = null;
    panning = null;
  };

  const onWheel = (e: WheelEvent) => {
    e.preventDefault();
    const [sx, sy] = localXY(e);
    const [wx, wy] = toWorld(sx, sy);
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    view.scale = Math.max(1e-4, Math.min(view.scale * factor, 50));
    // Keep the world point under the cursor fixed.
    view.cx = wx - (sx - W() / 2) / view.scale;
    view.cy = wy + (sy - H() / 2) / view.scale;
    draw();
  };

  const onKey = (e: KeyboardEvent) => {
    if (!drawing) return;
    if (e.key === "Enter" && poly.length >= 3) closePolygon();
    else if (e.key === "Escape") {
      drawing = false;
      poly = [];
      cursorWorld = null;
      drawBtn.classList.remove("map-btn-on");
      redraw();
    }
  };

  const startDrawing = () => {
    drawing = true;
    poly = [];
    cursorWorld = null;
    enclosed = new Set();
    drawBtn.classList.add("map-btn-on");
    canvas.style.cursor = "crosshair";
    setStatus("Draw mode: click to add polygon vertices, click the first vertex (or Enter) to close.");
    redraw();
  };

  const closePolygon = () => {
    drawing = false;
    cursorWorld = null;
    drawBtn.classList.remove("map-btn-on");
    canvas.style.cursor = "";
    redraw();
    setStatus(`Polygon closed — ${enclosed.size} well(s) enclosed. Drag a handle to adjust, or Assign to group.`);
  };

  drawBtn.addEventListener("click", () => (drawing ? closePolygon() : startDrawing()));
  clearBtn.addEventListener("click", () => {
    drawing = false;
    poly = [];
    cursorWorld = null;
    drawBtn.classList.remove("map-btn-on");
    canvas.style.cursor = "";
    redraw();
  });
  fitBtn.addEventListener("click", () => { fitView(); redraw(); });
  assignBtn.addEventListener("click", () => void openAssignDialog(poly, setStatus, () => void loadWells(false)));

  canvas.addEventListener("click", onClick);
  canvas.addEventListener("dblclick", onDblClick);
  canvas.addEventListener("mousedown", onDown);
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
  canvas.addEventListener("wheel", onWheel, { passive: false });
  window.addEventListener("keydown", onKey);

  // Resize + first real layout → fit once (didFit is declared with loadWells above so both
  // the data path and this observer share the same one-shot gate).
  const ro = new ResizeObserver(() => {
    if (!didFit && wells.length && W() > 0) {
      didFit = true;
      fitView();
    }
    redraw();
  });
  ro.observe(host);

  // Repaint on theme change; reload on data/group change (an import brings new coords).
  const unsubTheme = appState.themeVersion.subscribe(() => draw());
  // dataVersion.subscribe fires immediately; the awaited loadWells(true) below already does
  // the first load, so skip that synchronous call and only reload on later data changes.
  let firstData = true;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (firstData) { firstData = false; return; }
    void loadWells(false);
  });
  let firstGroups = true;
  const unsubGroups = appState.wellGroupsVersion.subscribe(() => {
    if (firstGroups) { firstGroups = false; return; }
    activeIds = new Set(appState.activeWellGroup.get()?.well_ids ?? []);
    redraw();
  });

  await loadWells(true);

  return {
    el,
    dispose: () => {
      ro.disconnect();
      unsubTheme();
      unsubData();
      unsubGroups();
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      window.removeEventListener("keydown", onKey);
    },
  };
}

function mkBtn(label: string, title: string): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "map-btn";
  b.textContent = label;
  b.title = title;
  return b;
}

/** The "put the enclosed wells into a group" dialog: pick an existing group (union) or
 *  name a new one. Selection is re-derived from the backend so the group matches the DB. */
async function openAssignDialog(
  poly: Vec2[],
  setStatus: (t: string) => void,
  onDone: () => void,
): Promise<void> {
  let selected: Awaited<ReturnType<typeof wellsInPolygon>> = [];
  try {
    selected = await wellsInPolygon(poly);
  } catch (err) {
    setStatus(`Well selection failed: ${err}`);
    return;
  }
  if (!selected.length) {
    setStatus("No wells inside the polygon.");
    return;
  }
  const ids = selected.map((w) => w.well_id);

  const groups = await listWellGroups().catch(() => []);
  const content = document.createElement("div");
  const doc = document.createElement("p");
  doc.className = "modal-doc";
  doc.textContent =
    `${selected.length} well(s) inside the polygon: ${selected.slice(0, 8).map((w) => w.well_name).join(", ")}` +
    `${selected.length > 8 ? "…" : ""}. Add them to a new group, or an existing one.`;
  content.appendChild(doc);

  const targetSel = document.createElement("select");
  targetSel.className = "form-control";
  const newOpt = document.createElement("option");
  newOpt.value = "__new__";
  newOpt.textContent = "＋ New group…";
  targetSel.appendChild(newOpt);
  for (const g of groups) {
    const o = document.createElement("option");
    o.value = g.group_id;
    o.textContent = `${g.name} (${g.member_count})`;
    targetSel.appendChild(o);
  }
  content.appendChild(formRow("Target", targetSel));

  const nameInput = document.createElement("input");
  nameInput.className = "form-control";
  nameInput.type = "text";
  nameInput.placeholder = "New group name…";
  const nameRow = formRow("Name", nameInput);
  content.appendChild(nameRow);
  targetSel.addEventListener("change", () => {
    nameRow.style.display = targetSel.value === "__new__" ? "" : "none";
  });

  const apply = document.createElement("button");
  apply.className = "form-run-btn";
  apply.textContent = "Assign";
  content.appendChild(apply);

  const close = openModal("Assign wells to group", content, 460);
  apply.addEventListener("click", async () => {
    apply.disabled = true;
    try {
      if (targetSel.value === "__new__") {
        const name = nameInput.value.trim();
        if (!name) {
          setStatus("Enter a group name.");
          apply.disabled = false;
          return;
        }
        await createWellGroup(name, ids);
        setStatus(`Created group “${name}” with ${ids.length} well(s).`);
        recordProcess("Group", `Created group "${name}" from map polygon (${ids.length} wells)`);
      } else {
        const g = groups.find((x) => x.group_id === targetSel.value)!;
        const union = [...new Set([...g.well_ids, ...ids])];
        await setWellGroupMembers(g.group_id, union);
        setStatus(`Group “${g.name}” now has ${union.length} well(s) (+${union.length - g.well_ids.length}).`);
        recordProcess("Group", `Assigned ${ids.length} well(s) to "${g.name}" from map polygon`);
      }
      await syncWellGroups();
      bumpWellGroupsVersion();
      onDone();
      close();
    } catch (err) {
      setStatus(`Assign failed: ${err}`);
      apply.disabled = false;
    }
  });
  nameInput.focus();
}
