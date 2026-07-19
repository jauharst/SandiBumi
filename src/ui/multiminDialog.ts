import {
  listWells,
  multiminFluidCalc,
  multiminLibrary,
  runMultimin,
  type MmComponent,
  type MmFluidProps,
  type MultiminRequest,
  type MultiminResult,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, defaultRunWellIds, filterByActiveGroup } from "../state";
import { recordProcess } from "../processLog";
import { openModal } from "./modal";

/** Generalized Multimin dialog — Geolog Multimin / IP Mineral Solver style.
 *
 *  - 27-component library (minerals, clays with CEC, zone-typed fluids: Sxo = flushed,
 *    Sw = unflushed) with editable endpoints and per-component max-volume bounds.
 *  - 16 input logs + user-defined inputs, each with a curve mnemonic and uncertainty σ
 *    (weight = 1/σ²). CT/CXO take a RESISTIVITY curve; the backend converts it to
 *    conductivity and builds the dual-water linear row Ct^(1/w) = Σ v·C^(1/w).
 *  - Constraints are automatic (Geolog program constraints): hard unity over
 *    minerals + unflushed fluids, POROSITY (ΣX = ΣU), BNDWAT (bound water tied to clay
 *    CEC), WATER MUD (Sxo ≥ Sw for WBM), and hard box bounds per component.
 *
 *  Physics defaults are single-sourced in Rust (`multimin_library`); this dialog edits
 *  a working copy. Spec: docs/multimin_geolog_spec.md + docs/multimin_ip_spec.md. */

interface ToolRow {
  key: string;
  label: string;
  curve: string;
  sigma: number;
  on: boolean;
  cond?: boolean;
  custom?: boolean;
}

const BASE_TOOLS: ToolRow[] = [
  { key: "RHOB", label: "Formation Density", curve: "RHOB", sigma: 0.0264, on: true },
  { key: "NPHI", label: "Neutron", curve: "NPHI", sigma: 0.014, on: true },
  { key: "DT", label: "Sonic Transit Time", curve: "DT", sigma: 1.951, on: true },
  { key: "GR", label: "Total Gamma Ray", curve: "GR", sigma: 6, on: true },
  { key: "PEF", label: "Photoelectric (PEF)", curve: "PEF", sigma: 0.3, on: false },
  { key: "U", label: "Photoelectric (U)", curve: "U", sigma: 0.32, on: false },
  { key: "THOR", label: "Spectral Thorium", curve: "THOR", sigma: 0.5, on: false },
  { key: "POTA", label: "Spectral Potassium", curve: "POTA", sigma: 0.2, on: false },
  { key: "URAN", label: "Spectral Uranium", curve: "URAN", sigma: 1.0, on: false },
  { key: "VP", label: "Compressional Velocity", curve: "VP", sigma: 0.11, on: false },
  { key: "VS", label: "Shear Velocity", curve: "VS", sigma: 0.11, on: false },
  { key: "CT", label: "Unflushed Conductivity (from RT)", curve: "RES_DEEP", sigma: 0, on: true, cond: true },
  { key: "CXO", label: "Flushed Conductivity (from RXO)", curve: "RXO", sigma: 0, on: false, cond: true },
  { key: "EPT", label: "EM Propagation (TPL)", curve: "TPL", sigma: 0.6, on: false },
  { key: "EATT", label: "EM Attenuation", curve: "EATT", sigma: 50, on: false },
  { key: "SIGMA", label: "Thermal Neutron Sigma", curve: "SIGM", sigma: 1.1, on: false },
];

const DEFAULT_COMPONENTS = ["Quartz", "Illite", "Water Sxo", "Water Sw"];

const KIND_LABEL: Record<string, string> = { mineral: "Minerals", clay: "Clays", fluid: "Fluids" };

function numInput(value: number, width = 64, step = "any"): HTMLInputElement {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.step = step;
  inp.value = String(value);
  inp.style.width = `${width}px`;
  return inp;
}

export async function openMultiminDialog(setStatus: (text: string) => void): Promise<void> {
  const [wells, library] = await Promise.all([
    listWells().then(filterByActiveGroup).catch(() => [] as WellSummary[]),
    multiminLibrary().catch(() => [] as MmComponent[]),
  ]);
  if (library.length === 0) {
    setStatus("SandiMin library unavailable (backend not reachable)");
    return;
  }
  const selectedWell = appState.selectedWell.get();

  // --- Working state --------------------------------------------------------
  const overrides = new Map<string, Map<string, number>>();
  const cecMap = new Map<string, number>();
  const maxMap = new Map<string, number>();
  for (const c of library) {
    overrides.set(c.name, new Map(Object.entries(c.endpoints)));
    cecMap.set(c.name, c.cec);
    maxMap.set(c.name, c.max_vol);
  }
  const included = new Set<string>(DEFAULT_COMPONENTS.filter((n) => overrides.has(n)));
  const tools: ToolRow[] = BASE_TOOLS.map((t) => ({ ...t }));

  const content = document.createElement("div");
  content.className = "mc-dialog";

  const columns = document.createElement("div");
  columns.className = "mm-columns";
  content.appendChild(columns);

  // --- Components box (grouped) --------------------------------------------
  const compBox = document.createElement("div");
  compBox.className = "mm-comp-box";
  const compChecks = new Map<string, HTMLInputElement>();
  for (const kind of ["mineral", "clay", "fluid"]) {
    const head = document.createElement("div");
    head.className = "mm-group-head";
    head.textContent = KIND_LABEL[kind];
    compBox.appendChild(head);
    for (const c of library.filter((c) => c.kind === kind)) {
      const row = document.createElement("label");
      row.className = "mm-comp-row";
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = included.has(c.name);
      cb.addEventListener("change", () => {
        if (cb.checked) included.add(c.name);
        else included.delete(c.name);
        renderTable();
      });
      compChecks.set(c.name, cb);
      row.appendChild(cb);
      const span = document.createElement("span");
      span.textContent = c.name;
      row.appendChild(span);
      if (c.kind === "fluid" && c.zone) {
        const badge = document.createElement("span");
        badge.className = "mm-badge";
        badge.textContent = c.zone === "X" ? "flushed" : "unflushed";
        row.appendChild(badge);
      }
      compBox.appendChild(row);
    }
  }
  columns.appendChild(compBox);

  // --- Tools box ------------------------------------------------------------
  const toolsCol = document.createElement("div");
  const toolsBox = document.createElement("div");
  toolsBox.className = "mm-tools";
  toolsCol.appendChild(toolsBox);

  function renderToolRow(t: ToolRow): void {
    const row = document.createElement("div");
    row.className = "mm-tool-row";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = t.on;
    cb.addEventListener("change", () => {
      t.on = cb.checked;
      updateFluidVisibility();
      renderTable();
    });
    row.appendChild(cb);
    const lab = document.createElement("span");
    lab.className = "mm-tool-key";
    lab.textContent = t.label;
    lab.title = t.key;
    row.appendChild(lab);
    const curve = document.createElement("input");
    curve.className = "mm-tool-curve";
    curve.value = t.curve;
    curve.addEventListener("input", () => (t.curve = curve.value.trim()));
    row.appendChild(curve);
    const sig = document.createElement("input");
    sig.className = "mm-tool-sigma";
    sig.type = "number";
    sig.step = "any";
    if (t.cond && t.sigma <= 0) {
      sig.placeholder = "auto";
    } else {
      sig.value = String(t.sigma);
    }
    sig.title = t.cond ? "Uncertainty (blank = auto: 0.03·C^(1/w))" : "Uncertainty σ (weight = 1/σ²)";
    sig.addEventListener("input", () => (t.sigma = sig.value.trim() === "" ? 0 : Number(sig.value)));
    row.appendChild(sig);
    toolsBox.appendChild(row);
  }
  for (const t of tools) renderToolRow(t);

  const addCustom = document.createElement("button");
  addCustom.type = "button";
  addCustom.className = "mm-add-custom";
  addCustom.textContent = "+ Add user-defined input";
  addCustom.addEventListener("click", () => {
    const name = prompt("Name of the user-defined input log (e.g. TOC, MQUA):", "USR1");
    if (!name) return;
    const key = name.trim().toUpperCase().replace(/[^A-Z0-9]/g, "_");
    if (!key || tools.some((t) => t.key === key)) return;
    const t: ToolRow = { key, label: name.trim(), curve: key, sigma: 0.015, on: true, custom: true };
    tools.push(t);
    for (const m of overrides.values()) m.set(key, 0);
    renderToolRow(t);
    renderTable();
  });
  toolsCol.appendChild(addCustom);
  columns.appendChild(toolsCol);

  // --- Fluid properties (needed for CT/CXO) ---------------------------------
  const fluidBox = document.createElement("div");
  fluidBox.className = "mm-fluid";
  const fluidHead = document.createElement("div");
  fluidHead.className = "mm-group-head";
  fluidHead.textContent = "Fluid properties (CT/CXO — resistivity → conductivity)";
  fluidBox.appendChild(fluidHead);
  const fluidGrid = document.createElement("div");
  fluidGrid.className = "mm-fluid-grid";
  fluidBox.appendChild(fluidGrid);

  const rwInp = numInput(0.43);
  const rwTInp = numInput(77);
  const rmfInp = numInput(0.1);
  const rmfTInp = numInput(62);
  const ftInp = numInput(148);
  const mInp = numInput(2, 52);
  const nInp = numInput(2, 52);
  const mudSel = document.createElement("select");
  for (const v of ["WATER", "OIL"]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = v === "WATER" ? "Water-based mud" : "Oil-based mud";
    mudSel.appendChild(o);
  }
  const fluidFields: [string, HTMLElement][] = [
    ["Rw sample (ohmm)", rwInp],
    ["@ temp (°F)", rwTInp],
    ["Rmf sample (ohmm)", rmfInp],
    ["@ temp (°F)", rmfTInp],
    ["Formation temp (°F)", ftInp],
    ["m", mInp],
    ["n", nInp],
    ["Mud", mudSel],
  ];
  for (const [lab, inp] of fluidFields) {
    const cell = document.createElement("label");
    cell.className = "mm-fluid-cell";
    const sp = document.createElement("span");
    sp.textContent = lab;
    cell.appendChild(sp);
    cell.appendChild(inp);
    fluidGrid.appendChild(cell);
  }
  const fluidPreview = document.createElement("div");
  fluidPreview.className = "mm-fluid-preview";
  fluidBox.appendChild(fluidPreview);
  content.appendChild(fluidBox);

  function readFluid(): MmFluidProps {
    return {
      rw: Number(rwInp.value) || 0.43,
      rw_temp_f: Number(rwTInp.value) || 77,
      rmf: Number(rmfInp.value) || 0.1,
      rmf_temp_f: Number(rmfTInp.value) || 62,
      ftemp_f: Number(ftInp.value) || 148,
      m: Number(mInp.value) || 2,
      n: Number(nInp.value) || 2,
      mud_type: mudSel.value,
    };
  }

  let previewTimer: number | undefined;
  function refreshFluidPreview(): void {
    window.clearTimeout(previewTimer);
    previewTimer = window.setTimeout(() => {
      multiminFluidCalc(readFluid())
        .then((fc) => {
          fluidPreview.textContent =
            `w=${fc.w.toFixed(2)}  Cw=${fc.cw.toFixed(2)}  Cmf=${fc.cmf.toFixed(2)}  Cbw=${fc.cbw.toFixed(2)} mho/m` +
            `  α(x/u)=${fc.alpha_x.toFixed(2)}/${fc.alpha_u.toFixed(2)}` +
            `  σCT=${fc.u_ct.toFixed(3)}  σCXO=${fc.u_cxo.toFixed(3)}`;
        })
        .catch(() => {
          fluidPreview.textContent = "Conductivities computed at run time.";
        });
    }, 250);
  }
  for (const [, inp] of fluidFields) inp.addEventListener("input", refreshFluidPreview);
  mudSel.addEventListener("change", refreshFluidPreview);
  refreshFluidPreview();

  function updateFluidVisibility(): void {
    fluidBox.style.display = tools.some((t) => t.cond && t.on) ? "" : "none";
  }
  updateFluidVisibility();

  // --- Endpoints table ------------------------------------------------------
  const tableWrap = document.createElement("div");
  tableWrap.className = "mm-table-wrap";
  content.appendChild(tableWrap);

  function renderTable(): void {
    tableWrap.innerHTML = "";
    const active = tools.filter((t) => t.on);
    const table = document.createElement("table");
    table.className = "mm-endpoints";
    const thead = document.createElement("thead");
    const hr = document.createElement("tr");
    for (const h of ["Component", ...active.map((t) => t.key), "CEC", "Max"]) {
      const th = document.createElement("th");
      th.textContent = h;
      if (h === "CEC") th.title = "Cation exchange capacity (meq/g, clays) — drives the bound-water constraint";
      if (h === "Max") th.title = "Upper volume bound (hard)";
      hr.appendChild(th);
    }
    thead.appendChild(hr);
    table.appendChild(thead);
    const tbody = document.createElement("tbody");
    for (const c of library.filter((c) => included.has(c.name))) {
      const tr = document.createElement("tr");
      const nameTd = document.createElement("td");
      nameTd.className = "mm-comp-name";
      nameTd.textContent = c.name;
      tr.appendChild(nameTd);
      const m = overrides.get(c.name)!;
      for (const t of active) {
        const td = document.createElement("td");
        td.className = "mm-cell";
        const uZoneFluid = c.kind === "fluid" && c.zone === "U";
        if (t.cond) {
          const inZone = c.kind === "fluid" && (t.key === "CT" ? c.zone !== "X" : c.zone !== "U");
          td.textContent = inZone && c.fluid_type !== "oil" && c.fluid_type !== "gas" ? "auto" : "—";
          td.classList.add("mm-cell-auto");
        } else if (uZoneFluid && !t.custom) {
          td.textContent = "—";
          td.classList.add("mm-cell-auto");
          td.title = "Unflushed-zone fluids are seen only by CT";
        } else {
          const inp = numInput(m.get(t.key) ?? 0, 58);
          inp.addEventListener("input", () => m.set(t.key, Number(inp.value)));
          td.appendChild(inp);
        }
        tr.appendChild(td);
      }
      const cecTd = document.createElement("td");
      cecTd.className = "mm-cell";
      if (c.kind === "clay") {
        const inp = numInput(cecMap.get(c.name) ?? 0, 48);
        inp.addEventListener("input", () => cecMap.set(c.name, Number(inp.value)));
        cecTd.appendChild(inp);
      } else {
        cecTd.textContent = "—";
        cecTd.classList.add("mm-cell-auto");
      }
      tr.appendChild(cecTd);
      const maxTd = document.createElement("td");
      maxTd.className = "mm-cell";
      const maxInp = numInput(maxMap.get(c.name) ?? 1, 48);
      maxInp.addEventListener("input", () => maxMap.set(c.name, Number(maxInp.value)));
      maxTd.appendChild(maxInp);
      tr.appendChild(maxTd);
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);
    tableWrap.appendChild(table);
  }
  renderTable();

  // --- Wells + options ------------------------------------------------------
  const wellBox = document.createElement("div");
  wellBox.className = "mm-tools";
  const wellChecks = new Map<string, HTMLInputElement>();
  const runDefaults = defaultRunWellIds(wells);
  if (runDefaults.size === 0 && selectedWell) runDefaults.add(selectedWell.well_id);
  for (const w of wells) {
    const row = document.createElement("label");
    row.className = "mm-tool-row";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = runDefaults.has(w.well_id);
    wellChecks.set(w.well_id, cb);
    row.appendChild(cb);
    const sp = document.createElement("span");
    sp.textContent = w.well_name || w.well_id;
    row.appendChild(sp);
    wellBox.appendChild(row);
  }
  const wellsHead = document.createElement("div");
  wellsHead.className = "mm-group-head";
  wellsHead.textContent = "Apply to wells";
  content.appendChild(wellsHead);
  content.appendChild(wellBox);

  const optsRow = document.createElement("div");
  optsRow.className = "mm-tool-row";
  const prefixLab = document.createElement("span");
  prefixLab.textContent = "Output prefix";
  const prefixInp = document.createElement("input");
  prefixInp.className = "mm-tool-curve";
  prefixInp.value = "MM";
  const unityLab = document.createElement("label");
  const unityCb = document.createElement("input");
  unityCb.type = "checkbox";
  unityCb.checked = true;
  unityLab.appendChild(unityCb);
  unityLab.appendChild(document.createTextNode(" Hard unity (Σ minerals + unflushed fluids = 1)"));
  optsRow.appendChild(prefixLab);
  optsRow.appendChild(prefixInp);
  optsRow.appendChild(unityLab);
  content.appendChild(optsRow);

  // --- Run ------------------------------------------------------------------
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.classList.add("primary");
  runBtn.textContent = "Run";
  runRow.appendChild(runBtn);
  const resultBox = document.createElement("div");
  content.appendChild(runRow);
  content.appendChild(resultBox);

  const close = openModal("SandiMin — Mineral Solver", content, 940);

  runBtn.addEventListener("click", async () => {
    const comps: MmComponent[] = library
      .filter((c) => included.has(c.name))
      .map((c) => ({
        name: c.name,
        kind: c.kind,
        zone: c.zone,
        fluid_type: c.fluid_type,
        endpoints: Object.fromEntries(overrides.get(c.name)!),
        cec: cecMap.get(c.name) ?? 0,
        max_vol: maxMap.get(c.name) ?? 1,
      }));
    const activeTools = tools
      .filter((t) => t.on && t.curve.trim() !== "")
      .map((t) => ({ key: t.key, curve: t.curve, sigma: t.sigma }));
    const applyWells = wells.filter((w) => wellChecks.get(w.well_id)?.checked).map((w) => w.well_id);
    if (comps.length < 2) {
      setStatus("SandiMin: select at least two components");
      return;
    }
    if (applyWells.length === 0) {
      setStatus("SandiMin: select at least one well");
      return;
    }
    const req: MultiminRequest = {
      components: comps,
      tools: activeTools,
      apply_well_ids: applyWells,
      output_prefix: prefixInp.value.trim() || "MM",
      unity: unityCb.checked,
      fluid: readFluid(),
    };
    runBtn.disabled = true;
    setStatus("SandiMin: running…");
    let res: MultiminResult;
    try {
      res = await runMultimin(req);
    } catch (e) {
      setStatus(`SandiMin failed: ${e}`);
      runBtn.disabled = false;
      return;
    }
    runBtn.disabled = false;
    if (res.error) {
      setStatus(`SandiMin: ${res.error}`);
      return;
    }
    resultBox.innerHTML = "";
    const table = document.createElement("table");
    table.className = "mm-endpoints";
    table.innerHTML =
      "<thead><tr><th>Well</th><th>Samples solved</th><th>Mean recon (σ)</th><th>Note</th></tr></thead>";
    const tb = document.createElement("tbody");
    for (const w of res.wells) {
      const tr = document.createElement("tr");
      const name = wells.find((x) => x.well_id === w.well_id)?.well_name || w.well_id;
      for (const cell of [
        name,
        String(w.rows_solved),
        Number.isFinite(w.mean_recon) ? w.mean_recon.toFixed(3) : "—",
        w.error ?? "—",
      ]) {
        const td = document.createElement("td");
        td.className = "mm-cell";
        td.textContent = cell;
        tr.appendChild(td);
      }
      tb.appendChild(tr);
    }
    table.appendChild(tb);
    resultBox.appendChild(table);
    const okWells = res.wells.filter((w) => !w.error).length;
    if (okWells > 0) {
      recordProcess(
        "Module",
        `SandiMin (${comps.map((c) => c.name).join(", ")}) → ${res.outputs.join(", ")}`,
        applyWells.join(", "),
      );
      bumpDataVersion();
      setStatus(`SandiMin: wrote ${res.outputs.length} curves to ${okWells} well(s)`);
    } else {
      setStatus("SandiMin: no well solved — check curves and endpoints");
    }
  });

  void close;
}
