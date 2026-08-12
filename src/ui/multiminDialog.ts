import { buildLogSetPicker } from "./logSetPicker";
import {
  getCurveData,
  listWells,
  listZones,
  multiminDryClay,
  multiminFluidCalc,
  multiminFluidFromPrecalc,
  multiminLibrary,
  multiminSwModels,
  runMultimin,
  type MmComponent,
  type MmCoreFit,
  type MmFluidProps,
  type MultiminRequest,
  type MultiminResult,
  type SwModel,
  type WellSummary,
  type ZoneEntry,
} from "../ipc";
import { appState, bumpDataVersion } from "../state";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";
import { attachResizeRedraw, canvasFont, faciesColor, readTheme } from "./plotCanvas";

/** Generalized Multimin dialog — commercial mineral-solver style.
 *
 *  - 27-component library (minerals, clays with CEC, zone-typed fluids: Sxo = flushed,
 *    Sw = unflushed) with editable endpoints and per-component max-volume bounds.
 *  - 16 input logs + user-defined inputs, each with a curve mnemonic and uncertainty σ
 *    (weight = 1/σ²). CT/CXO take a RESISTIVITY curve; the backend converts it to
 *    conductivity and builds the dual-water linear row Ct^(1/w) = Σ v·C^(1/w).
 *  - Constraints are automatic (program constraints): hard unity over
 *    minerals + unflushed fluids, POROSITY (ΣX = ΣU), BNDWAT (bound water tied to clay
 *    CEC), WATER MUD (Sxo ≥ Sw for WBM), and hard box bounds per component.
 *
 *  - Wet→dry clay converter (xlsx workflow): wet-clay picks + dry density →
 *    dry endpoints + CEC equivalent so BNDWAT solves v_bw = φ_clay/(1−φ_clay)·v_dryclay.
 *  - Fluid autofill: zone-averaged FTEMP_F / RMF from the precalc module's curves.
 *
 *  Physics defaults are single-sourced in Rust (`multimin_library`); this dialog edits
 *  a working copy. Spec: docs/multimin_ref_spec.md + docs/multimin_ip_spec.md. */

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

/** Model presets (playbook #2c): named GROUPINGS of existing library components — no endpoint
 *  values of their own, so picking one never changes reviewed numbers. Each lists the components
 *  to include; endpoints stay whatever the library/overrides hold. */
const MODEL_PRESETS: { id: string; label: string; components: string[]; note: string }[] = [
  {
    id: "clastic",
    label: "Clastic (quartz–clay–water)",
    components: ["Quartz", "Illite", "Kaolinite", "Water Sxo", "Water Sw", "BoundWater"],
    note: "The delta bread-and-butter: quartz + illite/kaolinite with bound water. Tools: RHOB + NPHI + GR (add DT/CT as available).",
  },
  {
    id: "ssc",
    label: "SSC-style (sand–silt–clay)",
    components: ["Quartz", "Orthoclase", "Clay", "Water Sxo", "Water Sw", "BoundWater"],
    note: "Mirrors the SSC sand/silt/clay split: quartz (sand), feldspar (the silt-fraction marker), generic clay. Compare VOL_* against the SSC module's VSAND/VSILT/VCLAY.",
  },
  {
    id: "carbonate",
    label: "Carbonate (calcite–dolomite–anhydrite)",
    components: ["Calcite", "Dolomite", "Anhydrite", "Water Sxo", "Water Sw"],
    note: "Carbonate stringers: calcite + dolomite with anhydrite as the evaporite end. PEF/U strongly recommended among the tools.",
  },
  {
    id: "organic",
    label: "Organic / coal (feeds unconventional)",
    components: ["Quartz", "Illite", "Coal", "Kerogen", "Water Sxo", "Water Sw", "BoundWater"],
    note: "Organic-rich intervals: coal + kerogen against a quartz-illite background. VOL_KEROGEN feeds the unconventional TOC workflow.",
  },
];

const KIND_LABEL: Record<string, string> = { mineral: "Minerals", clay: "Clays", fluid: "Fluids" };

function numInput(value: number, width = 64, step = "any"): HTMLInputElement {
  const inp = document.createElement("input");
  inp.type = "number";
  inp.step = step;
  inp.value = String(value);
  inp.style.width = `${width}px`;
  return inp;
}

/** A shrinkable + scrollable setup group: a clickable head (with a live count badge) toggles
 *  the body open/closed; long lists cap their height and scroll. Used for the mineral kind
 *  groups and the log-input list so the setup tabs never grow into one endless column. */
function collapsibleGroup(
  title: string,
  opts: { open?: boolean; scroll?: boolean; grid?: boolean } = {},
): { root: HTMLElement; body: HTMLElement; count: HTMLElement } {
  const root = document.createElement("div");
  root.className = "mm-collapse";
  const head = document.createElement("button");
  head.type = "button";
  head.className = "mm-collapse-head";
  const chevron = document.createElement("span");
  chevron.className = "mm-collapse-chevron";
  chevron.textContent = "▾";
  const label = document.createElement("span");
  label.textContent = title;
  const count = document.createElement("span");
  count.className = "mm-collapse-count";
  head.append(chevron, label, count);
  const body = document.createElement("div");
  // grid → multi-column wrap grid (mineral/clay/fluid lists); scroll → capped height with scroll.
  const cls = ["mm-collapse-body"];
  if (opts.scroll) cls.push("mm-collapse-scroll");
  if (opts.grid) cls.push("mm-comp-grid");
  body.className = cls.join(" ");
  const setOpen = (o: boolean): void => {
    body.style.display = o ? "" : "none";
    root.classList.toggle("collapsed", !o);
  };
  setOpen(opts.open ?? true);
  head.addEventListener("click", () => setOpen(body.style.display === "none"));
  root.append(head, body);
  return { root, body, count };
}

/** Hosted as a dock pane (workspace component "multimin"), not a popup. */
export async function buildMultiminContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const [wells, library, swModels] = await Promise.all([
    listWells().catch(() => [] as WellSummary[]),
    multiminLibrary().catch(() => [] as MmComponent[]),
    multiminSwModels().catch(() => []),
  ]);
  if (library.length === 0 || swModels.length === 0) {
    setStatus("SandiMin library or saturation-equation catalog unavailable (backend not reachable)");
    const msg = document.createElement("div");
    msg.className = "logview-message";
    msg.textContent = "SandiMin library or saturation-equation catalog unavailable — backend not reachable.";
    return { el: msg, dispose: () => {} };
  }
  // Scope selector resolves the run's wells live (group / ★ pinned / selection / all); `wells`
  // is still fetched (unfiltered) for the result-table name lookup and the autofill fallback.
  const scope = await buildWellScope();
  let selectedWell = appState.selectedWell.get();

  // --- Working state --------------------------------------------------------
  const overrides = new Map<string, Map<string, number>>();
  const cecMap = new Map<string, number>();
  const wcpMap = new Map<string, number>();
  const maxMap = new Map<string, number>();
  for (const c of library) {
    overrides.set(c.name, new Map(Object.entries(c.endpoints)));
    cecMap.set(c.name, c.cec);
    wcpMap.set(c.name, c.wet_clay_porosity);
    maxMap.set(c.name, c.max_vol);
  }
  const included = new Set<string>(DEFAULT_COMPONENTS.filter((n) => overrides.has(n)));
  const tools: ToolRow[] = BASE_TOOLS.map((t) => ({ ...t }));

  const content = document.createElement("div");
  content.className = "mc-dialog mm-dialog";

  // Tabbed setup so the pane isn't one long scroll: Minerals (selection + endpoint matrix),
  // Log inputs, Fluid, Clay. The run controls + results sit in a persistent footer BELOW the
  // tabs, so you configure across tabs and run from anywhere without losing your place.
  const tabBar = document.createElement("div");
  tabBar.className = "mm-tabs";
  const tabBody = document.createElement("div");
  tabBody.className = "mm-tab-body";
  content.appendChild(tabBar);
  content.appendChild(tabBody);

  const panels = new Map<string, HTMLElement>();
  const tabBtns = new Map<string, HTMLButtonElement>();
  function addTab(id: string, label: string): HTMLElement {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "mm-tab";
    btn.textContent = label;
    btn.addEventListener("click", () => showTab(id));
    tabBar.appendChild(btn);
    tabBtns.set(id, btn);
    const panel = document.createElement("div");
    panel.className = "mm-tab-panel";
    tabBody.appendChild(panel);
    panels.set(id, panel);
    return panel;
  }
  function showTab(id: string): void {
    for (const [k, p] of panels) p.style.display = k === id ? "" : "none";
    for (const [k, b] of tabBtns) b.classList.toggle("active", k === id);
  }
  // Log inputs first (Jauhar field review): you pick the curves before the mineral model.
  const logsPanel = addTab("logs", "Log inputs");
  const mineralsPanel = addTab("minerals", "Minerals");
  const fluidPanel = addTab("fluid", "Fluid");
  const clayPanel = addTab("clay", "Clay");
  const constraintsPanel = addTab("constraints", "Constraints");

  // --- Components box (grouped) --------------------------------------------
  const compBox = document.createElement("div");
  // Plain container: the preset row sits at the top, then each mineral KIND becomes its own
  // shrinkable + scrollable group (mm-collapse) so the tab opens compact instead of one long list.
  const compChecks = new Map<string, HTMLInputElement>();
  const groupCounts: { kind: string; el: HTMLElement }[] = [];

  // Model presets: replace the included set with a named grouping (existing components only —
  // endpoints/overrides are untouched, so a preset never changes reviewed numbers).
  const presetRow = document.createElement("div");
  presetRow.className = "mm-tool-row";
  const presetLab = document.createElement("span");
  presetLab.textContent = "Preset";
  const presetSel = document.createElement("select");
  const presetNone = document.createElement("option");
  presetNone.value = "";
  presetNone.textContent = "— custom —";
  presetSel.appendChild(presetNone);
  for (const p of MODEL_PRESETS) {
    const o = document.createElement("option");
    o.value = p.id;
    o.textContent = p.label;
    o.title = p.note;
    presetSel.appendChild(o);
  }
  const presetNote = document.createElement("div");
  presetNote.className = "mc-chain-note";
  presetNote.style.display = "none";
  presetSel.addEventListener("change", () => {
    const p = MODEL_PRESETS.find((x) => x.id === presetSel.value);
    if (!p) {
      presetNote.style.display = "none";
      return;
    }
    included.clear();
    for (const name of p.components) {
      if (overrides.has(name)) included.add(name);
    }
    for (const [name, cb] of compChecks) cb.checked = included.has(name);
    presetNote.textContent = p.note;
    presetNote.style.display = "";
    renderTable();
  });
  presetRow.appendChild(presetLab);
  presetRow.appendChild(presetSel);
  compBox.appendChild(presetRow);
  compBox.appendChild(presetNote);
  for (const kind of ["mineral", "clay", "fluid"]) {
    const members = library.filter((c) => c.kind === kind);
    if (members.length === 0) continue;
    // Minerals open by default; clays/fluids start collapsed so the tab opens compact. Multi-column
    // grid so the lists wrap to pane width (scroll both ways) instead of one endless column.
    const grp = collapsibleGroup(KIND_LABEL[kind], { open: kind === "mineral", scroll: true, grid: true });
    for (const c of members) {
      const row = document.createElement("label");
      row.className = "mm-comp-row";
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = included.has(c.name);
      cb.addEventListener("change", () => {
        if (cb.checked) included.add(c.name);
        else included.delete(c.name);
        // A manual tweak leaves preset territory — reflect that in the selector.
        presetSel.value = "";
        presetNote.style.display = "none";
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
      grp.body.appendChild(row);
    }
    groupCounts.push({ kind, el: grp.count });
    compBox.appendChild(grp.root);
  }
  mineralsPanel.appendChild(compBox);

  // Live "selected/total" badge on each group head, refreshed whenever the selection changes
  // (checkbox toggle, preset, dry-clay apply — all route through renderTable).
  function updateGroupCounts(): void {
    for (const g of groupCounts) {
      const total = library.filter((c) => c.kind === g.kind).length;
      const sel = library.filter((c) => c.kind === g.kind && included.has(c.name)).length;
      g.el.textContent = `${sel}/${total}`;
    }
  }

  // --- Tools box (shrinkable + scrollable) ----------------------------------
  const toolsCol = document.createElement("div");
  const toolsGroup = collapsibleGroup("Log inputs", { open: true, scroll: true });
  const toolsBox = document.createElement("div");
  toolsBox.className = "mm-tools";
  toolsGroup.body.appendChild(toolsBox);
  toolsCol.appendChild(toolsGroup.root);

  function updateToolsCount(): void {
    const active = tools.filter((t) => t.on).length;
    toolsGroup.count.textContent = `${active}/${tools.length} on`;
  }

  function renderToolRow(t: ToolRow): void {
    const row = document.createElement("div");
    row.className = "mm-tool-row";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = t.on;
    cb.addEventListener("change", () => {
      t.on = cb.checked;
      updateFluidVisibility();
      updateToolsCount();
      renderTable();
    });
    row.appendChild(cb);
    const lab = document.createElement("span");
    lab.className = "mm-tool-key";
    lab.textContent = t.label;
    lab.title = `${t.label} — ${t.key}`;
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
  updateToolsCount();

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
    updateToolsCount();
    renderTable();
  });
  toolsCol.appendChild(addCustom);
  logsPanel.appendChild(toolsCol);

  // --- Fluid properties (needed for CT/CXO) ---------------------------------
  const fluidBox = document.createElement("div");
  fluidBox.className = "mm-fluid";
  const fluidHead = document.createElement("div");
  fluidHead.className = "mm-group-head";
  fluidHead.textContent = "Fluid properties (CT/CXO — resistivity → conductivity)";
  fluidBox.appendChild(fluidHead);

  // Sw equation: how the deep resistivity becomes water saturation. Linear dual-water (default) is
  // the in-inversion mixing law; Indonesia/Simandoux are post-solve shaly-sand forms.
  const swRow = document.createElement("div");
  swRow.className = "mm-tool-row";
  const swLab = document.createElement("span");
  swLab.textContent = "Sw equation";
  swLab.title = "How the deep resistivity (CT) is turned into water saturation";
  const swModelSel = document.createElement("select");
  for (const { id, label } of swModels) {
    const o = document.createElement("option");
    o.value = id;
    o.textContent = label;
    swModelSel.appendChild(o);
  }
  swRow.appendChild(swLab);
  swRow.appendChild(swModelSel);
  fluidBox.appendChild(swRow);
  // Wet-shale extras. Rsh: Indonesia/Simandoux/Juhász. Archie a: Indonesia/Simandoux. φ_sh (wet-clay
  // porosity): Juhász only. Each cell is shown/hidden per model in syncSwModel.
  const swExtra = document.createElement("div");
  swExtra.className = "mm-fluid-grid";
  const rshInp = numInput(4.0);
  const archieAInp = numInput(1.0);
  const indonesiaKSel = document.createElement("select");
  for (const [value, label] of [
    ["0", "0 — SIMPLE"],
    ["1", "1 — FULL (default)"],
    ["2", "2 — TAR_SAND / Woodhouse"],
  ]) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    indonesiaKSel.appendChild(option);
  }
  indonesiaKSel.value = "1";
  const simandouxCInp = numInput(1.0);
  simandouxCInp.min = "1";
  simandouxCInp.max = "2";
  const phitShInp = numInput(0.1);
  const wsBInp = numInput(0);
  const mkExtraCell = (lab: string, inp: HTMLInputElement | HTMLSelectElement): HTMLLabelElement => {
    const cell = document.createElement("label");
    cell.className = "mm-fluid-cell";
    const sp = document.createElement("span");
    sp.textContent = lab;
    cell.appendChild(sp);
    cell.appendChild(inp);
    swExtra.appendChild(cell);
    return cell;
  };
  const rshCell = mkExtraCell("Rsh (ohmm)", rshInp);
  const archieACell = mkExtraCell("Archie a", archieAInp);
  const indonesiaKCell = mkExtraCell("Indonesia k preset", indonesiaKSel);
  const simandouxCCell = mkExtraCell("Modified-SLB C (1–2)", simandouxCInp);
  const phitShCell = mkExtraCell("Wet-clay φ (φ_sh)", phitShInp);
  const wsBCell = mkExtraCell("B override (0=auto)", wsBInp);
  fluidBox.appendChild(swExtra);
  const swNote = document.createElement("div");
  swNote.className = "mc-chain-note";
  fluidBox.appendChild(swNote);
  function syncSwModel(): void {
    const val = swModelSel.value;
    // Wet-shale inputs by model. Indonesia and both typed Simandoux equations read Rsh + Archie a;
    // Juhász reads Rsh + φ_sh. Only the modified-SLB equation reads C.
    const indonesia = val === "indonesia";
    const bardonPied = val === "simandoux_bardon_pied";
    const modifiedSlb = val === "simandoux_modified_slb";
    const shalySand = indonesia || bardonPied || modifiedSlb;
    const juhasz = val === "juhasz";
    const waxman = val === "waxman_smits";
    // Every model except linear dual-water runs post-solve and shares the note.
    const post = val !== "linear_dw";
    rshCell.style.display = shalySand || juhasz ? "" : "none";
    archieACell.style.display = shalySand ? "" : "none";
    indonesiaKCell.style.display = indonesia ? "" : "none";
    simandouxCCell.style.display = modifiedSlb ? "" : "none";
    phitShCell.style.display = juhasz ? "" : "none";
    wsBCell.style.display = waxman ? "" : "none";
    swExtra.style.display = shalySand || juhasz || waxman ? "" : "none";
    swNote.style.display = post ? "" : "none";
    if (waxman) {
      swNote.textContent =
        "Post-solve, Waxman-Smits (1968): Sw solves Ct = φt^m·(Cw·Swt^n + B·Qv·Swt^(n−1)). Qv comes from " +
        "the solved clay volumes (Σ v_clay·CEC·ρ / φt, meq/mL) and B from the Juhász B(T,Rw) fit unless you " +
        "enter a core-measured B override (the fit overshoots above ~120 °C). Uses your m/n as m*/n*. Needs a " +
        "CT tool + a U-zone hydrocarbon component; clay CEC drives Qv. PHIE/PHIT stay exactly as solved.";
    } else if (val === "juhasz") {
      swNote.textContent =
        "Post-solve, normalized Waxman-Smits (Juhász): the excess clay conductivity is read from the shale " +
        "point — Cwsh = 1/(Rsh·φ_sh^m) weighted by the normalized Qv (Vsh·φ_sh/φt) — instead of a " +
        "temperature-form Cwb, so it uses your wet-shale parameters directly. Needs a CT tool + a U-zone " +
        "hydrocarbon component; set Rsh from a shale pick and φ_sh (wet-clay porosity). PHIE/PHIT stay as solved.";
    } else if (val === "dual_water_nonlinear") {
      swNote.textContent =
        "Post-solve: the mineral solve runs as usual, then the exact Clavier dual-water equation is solved " +
        "for Sw honouring m and n separately (not folded into w). The bound-water saturation comes from the " +
        "solved bound-water volume and the clay-bound-water conductivity from formation temperature, so it " +
        "needs a CT tool + a U-zone hydrocarbon component (bound water optional). PHIE/PHIT stay exactly as solved.";
    } else if (val === "archie_total") {
      swNote.textContent =
        "Post-solve archie_total: Sw = (a·Rw/(φt^m·Rt))^(1/n), with no shale term. It ignores clay " +
        "conductivity, so on shaly sand it reads optimistically high (that's the baseline the shaly-sand " +
        "forms correct). Needs a CT tool + a U-zone hydrocarbon component. PHIE/PHIT stay exactly as solved.";
    } else if (indonesia) {
      swNote.textContent =
        "Post-solve indonesia: 1/√Rt = [Vsh^(1−k·Vsh/2)/√Rsh + √(φe^m/(a·Rw))]·Sw^(n/2). " +
        "The cited k presets are SIMPLE=0, FULL=1, and TAR_SAND/Woodhouse=2. Needs CT, effective porosity, " +
        "VSH, and a U-zone hydrocarbon component; set Rsh from a shale pick.";
    } else if (bardonPied) {
      swNote.textContent =
        "Post-solve simandoux_bardon_pied: 1/Rt = φe^m·Sw^n/(a·Rw) + Vsh·Sw/Rsh. " +
        "This is Geolog MODIFIED and IP's plain Simandoux, but the recorded method is the equation id. " +
        "Needs CT, effective porosity, VSH, and a U-zone hydrocarbon component.";
    } else if (modifiedSlb) {
      swNote.textContent =
        "Post-solve simandoux_modified_slb: 1/Rt = φe^m·Sw^n/[a·Rw·(1−Vsh)] + Vsh^C·Sw/Rsh. " +
        "This is Geolog SCHLUM and IP/Techlog Modified Simandoux; C is cited as 1–2 with default 1. " +
        "Needs CT, effective porosity, VSH, and a U-zone hydrocarbon component.";
    }
  }
  swModelSel.addEventListener("change", syncSwModel);
  syncSwModel();

  const fluidGrid = document.createElement("div");
  fluidGrid.className = "mm-fluid-grid";
  fluidBox.appendChild(fluidGrid);

  const rwInp = numInput(0.43);
  const rwTInp = numInput(77);
  const rmfInp = numInput(0.1);
  const rmfTInp = numInput(62);
  const ftInp = numInput(148);
  // Optional per-depth formation-temperature curve. Blank = use the fixed °F above; a name (e.g.
  // FTEMP_F from Prep) recomputes the T-dependent fluid quantities per sample.
  const ftempCurveInp = document.createElement("input");
  ftempCurveInp.type = "text";
  ftempCurveInp.placeholder = "fixed";
  ftempCurveInp.style.width = "96px";
  ftempCurveInp.title =
    "Per-depth formation temperature (°F) curve name — blank uses the fixed temperature. When set, " +
    "Cw/Cmf/Cbw, the auto CT/CXO σ, the clay bound-water tie and the Waxman-Smits B are recomputed at " +
    "each sample's temperature. A missing or below-freezing sample falls back to the fixed value.";
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
    ["FTEMP curve (opt)", ftempCurveInp],
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

  // --- Autofill FTEMP / RMF from the precalc module's curves (zone-averaged) ---
  const autofillRow = document.createElement("div");
  autofillRow.className = "mm-tool-row";
  const autoLab = document.createElement("span");
  autoLab.className = "mm-tool-key";
  autoLab.textContent = "Autofill from precalc";
  autoLab.title = "Reads the well's FTEMP_F and RMF curves (precalc module outputs), averaged over the chosen zone";
  autofillRow.appendChild(autoLab);
  const zoneSel = document.createElement("select");
  const wholeOpt = document.createElement("option");
  wholeOpt.value = "";
  wholeOpt.textContent = "(whole well)";
  zoneSel.appendChild(wholeOpt);
  let zoneList: ZoneEntry[] = [];
  function refreshZones(): void {
    zoneList = [];
    while (zoneSel.options.length > 1) zoneSel.remove(1);
    zoneSel.value = "";
    if (!selectedWell) return;
    const wid = selectedWell.well_id;
    listZones(wid)
      .then((zs) => {
        if (selectedWell?.well_id !== wid) return;
        zoneList = zs;
        for (const z of zs) {
          const o = document.createElement("option");
          o.value = z.zone_name;
          o.textContent = z.zone_name;
          zoneSel.appendChild(o);
        }
      })
      .catch(() => {});
  }
  // The pane is a persistent singleton — track the well selection like the
  // other panes do (fires immediately, doing the initial zone fill).
  const unsubWell = appState.selectedWell.subscribe((w) => {
    selectedWell = w;
    refreshZones();
  });
  autofillRow.appendChild(zoneSel);
  const autofillBtn = document.createElement("button");
  autofillBtn.type = "button";
  autofillBtn.textContent = "Read";
  autofillBtn.addEventListener("click", async () => {
    const scopeIds = new Set(scope.getWellIds());
    const well = selectedWell ?? wells.find((w) => scopeIds.has(w.well_id));
    if (!well) {
      setStatus("SandiMin autofill: select a well first");
      return;
    }
    const zone = zoneList.find((z) => z.zone_name === zoneSel.value);
    try {
      const pf = await multiminFluidFromPrecalc(well.well_id, zone?.top_depth ?? null, zone?.bottom_depth ?? null);
      // Race guard (mirrors refreshZones): if the active well changed during the await, the form
      // now belongs to a different well — don't stamp this stale well's FTEMP/RMF onto it.
      if (selectedWell && selectedWell.well_id !== well.well_id) return;
      if (pf.ftemp_f === null && pf.rmf === null) {
        setStatus(`SandiMin autofill: no FTEMP_F/RMF samples on ${well.well_name} — run the precalc module first`);
        return;
      }
      if (pf.ftemp_f === null) {
        // An RMF curve resolved without FTEMP_F — a raw import, not a precalc
        // output, so its reference temperature is unknown. Apply nothing.
        setStatus(
          `SandiMin autofill: ${well.well_name} has an RMF curve but no FTEMP_F — ` +
            `not a precalc output, nothing applied (run the precalc module first)`,
        );
        return;
      }
      ftInp.value = pf.ftemp_f.toFixed(1);
      if (pf.rmf !== null) {
        rmfInp.value = pf.rmf.toFixed(4);
        // Precalc's RMF is already at formation temperature — retie the sample
        // temperature only when an RMF value actually came back with it.
        rmfTInp.value = pf.ftemp_f.toFixed(1);
      }
      refreshFluidPreview();
      refreshDryPreview();
      const where = zone ? zone.zone_name : "whole well";
      setStatus(
        `SandiMin autofill (${well.well_name}, ${where}): FTEMP ${pf.ftemp_f.toFixed(1)} °F, ` +
          `RMF ${pf.rmf?.toFixed(4) ?? "—"} ohmm (${pf.n_ftemp}/${pf.n_rmf} samples)`,
      );
    } catch (e) {
      setStatus(`SandiMin autofill failed: ${e}`);
    }
  });
  autofillRow.appendChild(autofillBtn);
  fluidBox.appendChild(autofillRow);
  // A hint keeps the Fluid tab from looking empty when no conductivity tool needs it.
  const fluidHint = document.createElement("div");
  fluidHint.className = "mc-chain-note";
  fluidHint.textContent =
    "Fluid properties feed the CT/CXO conductivity rows. No conductivity tool (CT or CXO) is active right now, so these values won't affect the solve — turn one on in Log inputs to use them.";
  fluidPanel.appendChild(fluidHint);
  fluidPanel.appendChild(fluidBox);

  // --- Wet clay → dry clay converter (xlsx workflow) ---------------
  // Pick wet-clay readings in a shale interval, assume a dry-clay density, and
  // the backend derives the dry endpoints + the CEC that makes the BNDWAT
  // constraint solve bound water as v_bw = φ_clay/(1−φ_clay) · v_dryclay.
  const dryBox = document.createElement("div");
  dryBox.className = "mm-fluid";
  const dryHead = document.createElement("div");
  dryHead.className = "mm-group-head";
  dryHead.textContent = "Wet clay → dry clay (PHIT-basis endpoints)";
  dryBox.appendChild(dryHead);
  const dryGrid = document.createElement("div");
  dryGrid.className = "mm-fluid-grid";
  dryBox.appendChild(dryGrid);

  const wetRhobInp = numInput(2.18);
  const wetNphiInp = numInput(0.49);
  const wetGrInp = numInput(110);
  const wetDtInp = document.createElement("input");
  wetDtInp.type = "number";
  wetDtInp.step = "any";
  wetDtInp.style.width = "64px";
  wetDtInp.placeholder = "(none)";
  const dryRhoInp = numInput(2.7);
  const claySel = document.createElement("select");
  for (const c of library.filter((c) => c.kind === "clay")) {
    const o = document.createElement("option");
    o.value = c.name;
    o.textContent = c.name;
    if (c.name === "Illite") o.selected = true;
    claySel.appendChild(o);
  }
  const dryFields: [string, HTMLElement][] = [
    ["Wet RHOB (g/cc)", wetRhobInp],
    ["Wet NPHI (v/v)", wetNphiInp],
    ["Wet GR (API)", wetGrInp],
    ["Wet DT (µs/ft)", wetDtInp],
    ["Dry clay density (g/cc)", dryRhoInp],
    ["Apply to clay", claySel],
  ];
  for (const [lab, inp] of dryFields) {
    const cell = document.createElement("label");
    cell.className = "mm-fluid-cell";
    const sp = document.createElement("span");
    sp.textContent = lab;
    cell.appendChild(sp);
    cell.appendChild(inp);
    dryGrid.appendChild(cell);
  }
  const dryPreview = document.createElement("div");
  dryPreview.className = "mm-fluid-preview";
  dryBox.appendChild(dryPreview);
  const dryApplyRow = document.createElement("div");
  dryApplyRow.className = "mm-tool-row";
  const dryApplyBtn = document.createElement("button");
  dryApplyBtn.type = "button";
  dryApplyBtn.textContent = "Apply to clay + include BoundWater";
  dryApplyRow.appendChild(dryApplyBtn);
  dryBox.appendChild(dryApplyRow);
  clayPanel.appendChild(dryBox);

  // --- Per-clay wet-clay porosity φ editor ---------------------------------
  // Only consulted when the Porosity Source (Constraints tab) is "Wet Clay Porosity": the BNDWAT
  // tie then solves v_bw = φ/(1−φ)·v_dryclay. Techlog WCLP defaults are pre-filled; smectite's
  // placeholder φ=1.0 is handled by the backend (falls back to CEC), so it's shown read-only-ish.
  const wcpBox = document.createElement("div");
  wcpBox.className = "mm-fluid-box";
  const wcpHead = document.createElement("div");
  wcpHead.className = "mm-group-head";
  wcpHead.textContent = "Wet-clay porosity φ (per clay)";
  wcpBox.appendChild(wcpHead);
  const wcpNote = document.createElement("div");
  wcpNote.className = "mc-chain-note";
  wcpNote.textContent =
    "Used only when Porosity Source = Wet Clay Porosity (Constraints tab). k = φ/(1−φ). " +
    "Dry-clay Apply also updates a clay's φ. Smectite's φ=1.0 is a placeholder — the solver falls back to its CEC there.";
  wcpBox.appendChild(wcpNote);
  const wcpGrid = document.createElement("div");
  wcpGrid.className = "mm-fluid-grid";
  const wcpInputs = new Map<string, HTMLInputElement>();
  for (const c of library.filter((x) => x.kind === "clay")) {
    const cell = document.createElement("label");
    cell.className = "mm-fluid-cell";
    const sp = document.createElement("span");
    sp.textContent = c.name;
    const inp = numInput(wcpMap.get(c.name) ?? 0, 56);
    inp.addEventListener("input", () => wcpMap.set(c.name, Number(inp.value)));
    wcpInputs.set(c.name, inp);
    cell.appendChild(sp);
    cell.appendChild(inp);
    wcpGrid.appendChild(cell);
  }
  wcpBox.appendChild(wcpGrid);
  clayPanel.appendChild(wcpBox);

  function readWetClay() {
    return {
      rhob_wet: Number(wetRhobInp.value) || 0,
      nphi_wet: Number(wetNphiInp.value) || 0,
      gr_wet: Number(wetGrInp.value) || 0,
      dt_wet: wetDtInp.value.trim() === "" ? null : Number(wetDtInp.value),
      rho_dry: Number(dryRhoInp.value) || 0,
      fluid: readFluid(),
    };
  }

  let dryTimer: number | undefined;
  function refreshDryPreview(): void {
    window.clearTimeout(dryTimer);
    dryTimer = window.setTimeout(() => {
      multiminDryClay(readWetClay())
        .then((dc) => {
          const dt = dc.dt_dry === null ? "—" : dc.dt_dry.toFixed(1);
          dryPreview.textContent =
            `φ_clay=${dc.phi_clay.toFixed(4)}  →  RHOB ${dc.rhob_dry.toFixed(3)}  NPHI ${dc.nphi_dry.toFixed(4)}` +
            `  GR ${dc.gr_dry.toFixed(1)}  DT ${dt}  |  v_bw = ${dc.cbw_ratio.toFixed(4)}·v_dryclay` +
            `  (CEC_eq ${dc.cec_equiv.toFixed(4)} meq/g at current fluid T/α)`;
        })
        .catch((e) => {
          dryPreview.textContent = String(e);
        });
    }, 250);
  }
  for (const [, inp] of dryFields) inp.addEventListener("input", refreshDryPreview);
  refreshDryPreview();

  dryApplyBtn.addEventListener("click", async () => {
    let dc;
    try {
      dc = await multiminDryClay(readWetClay());
    } catch (e) {
      setStatus(`Dry-clay conversion: ${e}`);
      return;
    }
    const clay = claySel.value;
    const m = overrides.get(clay);
    if (!m) return;
    m.set("RHOB", Number(dc.rhob_dry.toFixed(3)));
    m.set("NPHI", Number(dc.nphi_dry.toFixed(4)));
    m.set("GR", Number(dc.gr_dry.toFixed(1)));
    if (dc.dt_dry !== null) m.set("DT", Number(dc.dt_dry.toFixed(1)));
    cecMap.set(clay, Number(dc.cec_equiv.toFixed(4)));
    // Keep the Wet-Clay-Porosity source consistent with the converter: it solved this same φ_clay.
    wcpMap.set(clay, Number(dc.phi_clay.toFixed(4)));
    const wcpInp = wcpInputs.get(clay);
    if (wcpInp) wcpInp.value = String(Number(dc.phi_clay.toFixed(4)));
    included.add(clay);
    compChecks.get(clay)!.checked = true;
    // The dry-clay framework needs bound water solved explicitly (PHIT basis).
    if (overrides.has("BoundWater")) {
      included.add("BoundWater");
      compChecks.get("BoundWater")!.checked = true;
    }
    renderTable();
    setStatus(
      `SandiMin: dry-clay endpoints applied to ${clay} — φ_clay ${dc.phi_clay.toFixed(4)}, ` +
        `CEC_eq ${dc.cec_equiv.toFixed(4)} meq/g (re-apply if fluid T/Rw/α or this clay's RHOB endpoint change)`,
    );
  });

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
      rsh: Number(rshInp.value) || 4,
      archie_a: Number(archieAInp.value) || 1,
      indonesia_k: Number(indonesiaKSel.value),
      simandoux_c: Number(simandouxCInp.value) || 1,
      phit_sh: Number(phitShInp.value) || 0.1,
      ws_b: Number(wsBInp.value) || 0,
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
  // Fluid T/Rw changes also move the dry-clay panel's CEC_eq — keep both fresh.
  const refreshBothPreviews = (): void => {
    refreshFluidPreview();
    refreshDryPreview();
  };
  for (const [, inp] of fluidFields) inp.addEventListener("input", refreshBothPreviews);
  mudSel.addEventListener("change", refreshBothPreviews);
  refreshFluidPreview();

  // --- Constraints tab: porosity source + program-constraint enables (Jauhar image 2) ----------
  // Every constraint already runs in the solver; this panel EXPOSES them. Defaults match the reviewed
  // behavior (all on, σ=0.01, CEC), so leaving this tab untouched changes nothing.
  const psBox = document.createElement("div");
  psBox.className = "mm-fluid-box";
  const psHead = document.createElement("div");
  psHead.className = "mm-group-head";
  psHead.textContent = "Porosity source (clay bound water)";
  psBox.appendChild(psHead);
  const psNote = document.createElement("div");
  psNote.className = "mc-chain-note";
  psNote.textContent =
    "What drives the BNDWAT tie v_bw = k·v_dryclay. CEC (default): k = α·96·CEC·ρ/(T+298). " +
    "Wet Clay Porosity: k = φ/(1−φ) from the per-clay φ on the Clay tab — this moves PHIE.";
  psBox.appendChild(psNote);
  const psRow = document.createElement("div");
  psRow.className = "mm-tool-row";
  const psCecLab = document.createElement("label");
  const psCecRadio = document.createElement("input");
  psCecRadio.type = "radio";
  psCecRadio.name = "mm-porosity-source";
  psCecRadio.checked = true;
  psCecLab.appendChild(psCecRadio);
  psCecLab.appendChild(document.createTextNode(" Cation Exchange Capacity"));
  const psWcpLab = document.createElement("label");
  const psWcpRadio = document.createElement("input");
  psWcpRadio.type = "radio";
  psWcpRadio.name = "mm-porosity-source";
  psWcpLab.appendChild(psWcpRadio);
  psWcpLab.appendChild(document.createTextNode(" Wet Clay Porosity"));
  psRow.appendChild(psCecLab);
  psRow.appendChild(psWcpLab);
  psBox.appendChild(psRow);
  constraintsPanel.appendChild(psBox);

  // Program constraints (enable toggles). UNITY lives here now (relocated from the run footer).
  const conBox = document.createElement("div");
  conBox.className = "mm-fluid-box";
  const conHead = document.createElement("div");
  conHead.className = "mm-group-head";
  conHead.textContent = "Program constraints";
  conBox.appendChild(conHead);
  const mkConstraint = (checked: boolean, label: string, note: string): HTMLInputElement => {
    const row = document.createElement("label");
    row.className = "mm-tool-row";
    row.title = note;
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = checked;
    const txt = document.createElement("span");
    txt.textContent = " " + label;
    row.appendChild(cb);
    row.appendChild(txt);
    conBox.appendChild(row);
    return cb;
  };
  const unityCb = mkConstraint(
    true,
    "UNITY — Σ minerals + unflushed fluids = 1 (hard)",
    "Hard unity constraint over the solved volumes.",
  );
  const porosityCb = mkConstraint(
    true,
    "POROSITY — Σ flushed fluids = Σ unflushed fluids",
    "Ties flushed- and virgin-zone porosity (soft).",
  );
  const bndwatCb = mkConstraint(
    true,
    "X&U BNDWAT — bound water tied to clay volume",
    "v_bw = k·v_dryclay via the chosen porosity source (soft).",
  );
  const waterMudCb = mkConstraint(
    true,
    "WATER MUD — flushed water ≥ virgin water (WBM)",
    "For water-based mud, invasion cannot lower water saturation (Sxo ≥ Sw). Ignored for oil-based mud.",
  );
  const sigmaRow = document.createElement("label");
  sigmaRow.className = "mm-tool-row";
  sigmaRow.title = "Soft-constraint tolerance σ; the constraint row weight is 1/σ. Default 0.01.";
  const sigmaLab = document.createElement("span");
  sigmaLab.textContent = "Constraint tolerance σ";
  const sigmaInp = numInput(0.01, 56);
  sigmaRow.appendChild(sigmaLab);
  sigmaRow.appendChild(sigmaInp);
  conBox.appendChild(sigmaRow);
  constraintsPanel.appendChild(conBox);

  function updateFluidVisibility(): void {
    const need = tools.some((t) => t.cond && t.on);
    fluidBox.style.display = need ? "" : "none";
    fluidHint.style.display = need ? "none" : "";
  }
  updateFluidVisibility();

  // --- Endpoints table (lives under the Minerals tab, below the selection list) -----------
  const tableWrap = document.createElement("div");
  tableWrap.className = "mm-table-wrap";
  const tableHead = document.createElement("div");
  tableHead.className = "mm-group-head";
  tableHead.textContent = "Endpoints (selected components × active logs)";
  mineralsPanel.appendChild(tableHead);
  mineralsPanel.appendChild(tableWrap);

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
      if (h === "CEC")
        th.title =
          "Cation exchange capacity (meq/g, clays) — drives the bound-water constraint. " +
          "A converter-derived CEC_eq is paired to the clay's RHOB endpoint — re-Apply after editing either.";
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
    updateGroupCounts();
  }
  renderTable();

  // Start on the Log inputs tab now that every panel is populated.
  showTab("logs");

  // --- Wells + options + Run (persistent section, ABOVE the tabs) -----------
  // Jauhar field review: the apply-wells + Run controls sit on TOP so a run launches without
  // scrolling past every parameter tab. Scope selector (group / ★ pinned / selection / all)
  // instead of a per-well checklist — a 2000-well field can't be ticked one at a time.
  const runSection = document.createElement("div");
  runSection.className = "mm-run-section";
  const wellsHead = document.createElement("div");
  wellsHead.className = "mm-group-head";
  wellsHead.textContent = "Apply to wells";
  runSection.appendChild(wellsHead);
  runSection.appendChild(scope.el);

  const optsRow = document.createElement("div");
  optsRow.className = "mm-tool-row";
  const prefixLab = document.createElement("span");
  prefixLab.textContent = "Output prefix";
  const prefixInp = document.createElement("input");
  prefixInp.className = "mm-tool-curve";
  prefixInp.value = "MM";
  // UNITY, POROSITY, BNDWAT, WATER MUD + the porosity source now live on the Constraints tab.
  const reconLab = document.createElement("label");
  const reconCb = document.createElement("input");
  reconCb.type = "checkbox";
  reconLab.title =
    "Emit per-tool reconstruction curves (<prefix>_<KEY>_REC / _DIF) and show the measured-vs-reconstructed QC after the run";
  reconLab.appendChild(reconCb);
  reconLab.appendChild(document.createTextNode(" Reconstruction QC"));
  optsRow.appendChild(prefixLab);
  optsRow.appendChild(prefixInp);
  optsRow.appendChild(reconLab);
  runSection.appendChild(optsRow);

  // --- Input / output log set (`logSetPicker.ts`). A mineral inversion is only as reproducible
  // as the logs it was solved from: the same components, endpoints and constraints over a
  // re-run porosity return a different mineralogy. Output defaults to SANDIMIN, which is where
  // the volumes always went.
  const setPicker = buildLogSetPicker({ write: "SANDIMIN" });
  for (const row of setPicker.rows) runSection.appendChild(row);

  // --- Run (distinct SandiMin-green button, set apart from other modules' accent runs) -------
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.classList.add("primary", "mm-run-btn");
  runBtn.textContent = "Run";
  runRow.appendChild(runBtn);
  const resultBox = document.createElement("div");
  runSection.appendChild(runRow);
  runSection.appendChild(resultBox);
  // Pin the whole run section above the parameter tabs.
  content.insertBefore(runSection, tabBar);
  // Cleanup for the reconstruction-QC canvas's resize observer (replaced each run).
  let detachRecon: (() => void) | null = null;

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
        wet_clay_porosity: wcpMap.get(c.name) ?? 0,
        max_vol: maxMap.get(c.name) ?? 1,
      }));
    const activeTools = tools
      .filter((t) => t.on && t.curve.trim() !== "")
      .map((t) => ({ key: t.key, curve: t.curve, sigma: t.sigma }));
    const applyWells = scope.getWellIds();
    if (comps.length < 2) {
      setStatus("SandiMin: select at least two components");
      return;
    }
    if (applyWells.length === 0) {
      setStatus("No wells in scope — pick a group, pin/select wells, or choose All");
      return;
    }
    const req: MultiminRequest = {
      components: comps,
      tools: activeTools,
      apply_well_ids: applyWells,
      output_prefix: prefixInp.value.trim() || "MM",
      input_set: setPicker.inputSet(),
      output_set: setPicker.outputSet(),
      unity: unityCb.checked,
      fluid: readFluid(),
      ftemp_curve: ftempCurveInp.value.trim() || undefined,
      recon_qc: reconCb.checked,
      sw_model: swModelSel.value as SwModel,
      porosity_source: psWcpRadio.checked ? "wet_clay_porosity" : "cec",
      enforce_porosity: porosityCb.checked,
      enforce_bndwat: bndwatCb.checked,
      enforce_water_mud: waterMudCb.checked,
      sigma_constraint: Number(sigmaInp.value) || 0.01,
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
    detachRecon?.();
    detachRecon = null;
    resultBox.innerHTML = "";

    // Degrees-of-freedom badge: dof 0 means the reconstruction can't validate the model.
    const dofLine = document.createElement("div");
    dofLine.className = "mc-chain-note";
    if (res.dof <= 0 && res.dof_note) {
      dofLine.style.color = "var(--warn)";
      dofLine.textContent = `DOF ${res.dof} — ${res.dof_note}`;
    } else {
      dofLine.textContent = `Model DOF ${res.dof} (over-determined — RECON/incoherence is a real fit-quality signal).`;
    }
    resultBox.appendChild(dofLine);

    const table = document.createElement("table");
    table.className = "mm-endpoints";
    table.innerHTML =
      "<thead><tr><th>Well</th><th>Samples solved</th><th>Incoherence (σ)</th><th>Note</th></tr></thead>";
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

    // --- Core calibration ---------------------------------------------------
    // Only rendered when some well actually has plugs. RECON above measures the fit to the model's
    // OWN input logs; this measures it against an INDEPENDENT measurement, which is the one that
    // can tell you the endpoints are wrong.
    if (res.wells.some((w) => w.core_phie || w.core_phit || w.core_gd)) {
      const coreCap = document.createElement("div");
      coreCap.className = "mc-chain-note";
      coreCap.textContent =
        "Core calibration — RMS of (model − core) over plugs tied to a solved sample within 1 m; " +
        "bias is the mean signed error, so its sign says which way the model reads. Core φ is shown " +
        "against both porosities because the drying protocol decides which one a plug should match " +
        "(oven-dried drives off clay-bound water → PHIT; humidity-dried retains some → nearer PHIE).";
      const coreTable = document.createElement("table");
      coreTable.className = "mm-endpoints";
      coreTable.innerHTML =
        "<thead><tr><th>Well</th><th>Core φ vs PHIE</th><th>Core φ vs PHIT</th><th>Core ρg (g/cc)</th></tr></thead>";
      const coreBody = document.createElement("tbody");
      const fitCell = (f: MmCoreFit | null): string =>
        f ? `${f.rms.toFixed(3)}  (bias ${f.bias >= 0 ? "+" : ""}${f.bias.toFixed(3)}, n=${f.n})` : "—";
      for (const w of res.wells) {
        if (!w.core_phie && !w.core_phit && !w.core_gd) continue;
        const tr = document.createElement("tr");
        const name = wells.find((x) => x.well_id === w.well_id)?.well_name || w.well_id;
        for (const cell of [name, fitCell(w.core_phie), fitCell(w.core_phit), fitCell(w.core_gd)]) {
          const td = document.createElement("td");
          td.className = "mm-cell";
          td.textContent = cell;
          tr.appendChild(td);
        }
        coreBody.appendChild(tr);
      }
      coreTable.appendChild(coreBody);
      resultBox.append(coreCap, coreTable);
    }

    const okWells = res.wells.filter((w) => !w.error).length;
    if (okWells > 0) {
      recordProcess(
        "Module",
        `SandiMin (${comps.map((c) => c.name).join(", ")}) → ${res.outputs.join(", ")}`,
        applyWells.join(", "),
      );
      bumpDataVersion();
      setStatus(`SandiMin: wrote ${res.outputs.length} curves to ${okWells} well(s)`);
      // Reconstruction QC: for the first solved well, plot measured vs reconstructed per tool.
      if (reconCb.checked) {
        const firstOk = res.wells.find((w) => !w.error && w.rows_solved > 0);
        if (firstOk) {
          const prefix = (prefixInp.value.trim() || "MM").toUpperCase();
          detachRecon = await renderReconQc(resultBox, firstOk.well_id, prefix, activeTools);
        }
      }
    } else {
      setStatus("SandiMin: no well solved — check curves and endpoints");
    }
  });

  return {
    el: content,
    dispose: () => {
      window.clearTimeout(previewTimer);
      window.clearTimeout(dryTimer);
      detachRecon?.();
      unsubWell();
      scope.dispose();
    },
  };
}

/** Measured-vs-reconstructed QC for one well: for each active tool with a `<prefix>_<KEY>_REC`
 *  curve, plot (measured, reconstructed) points coloured by tool against the 1:1 line — points on
 *  the diagonal are a perfect reconstruction, scatter off it is the tool's incoherence. Returns a
 *  cleanup that detaches the canvas resize observer (null if nothing could be drawn). */
async function renderReconQc(
  host: HTMLElement,
  wellId: string,
  prefix: string,
  tools: { key: string; curve: string }[],
): Promise<(() => void) | null> {
  // Curve-safe token matching the backend curve_token() (uppercase, non-alphanumeric → '_').
  const token = (s: string) =>
    s.trim().toUpperCase().replace(/[^A-Z0-9]/g, "_").replace(/^_+|_+$/g, "");
  const series: { label: string; meas: number[]; rec: number[] }[] = [];
  for (const t of tools) {
    const recName = `${prefix}_${token(t.key)}_REC`;
    try {
      const got = await getCurveData(wellId, [t.curve.trim().toUpperCase(), recName], null, null);
      const measC = got.find((g) => g.curve_name.toUpperCase() === t.curve.trim().toUpperCase());
      const recC = got.find((g) => g.curve_name.toUpperCase() === recName);
      if (!measC || !recC) continue;
      const meas: number[] = [];
      const rec: number[] = [];
      const n = Math.min(measC.value.length, recC.value.length);
      for (let i = 0; i < n; i++) {
        const m = measC.value[i];
        const r = recC.value[i];
        if (Number.isFinite(m) && Number.isFinite(r)) {
          meas.push(m);
          rec.push(r);
        }
      }
      if (meas.length > 0) series.push({ label: t.key, meas, rec });
    } catch {
      // A missing/unreadable curve just drops that tool from the QC plot.
    }
  }
  if (series.length === 0) return null;

  const cap = document.createElement("div");
  cap.className = "mc-hist-caption";
  cap.textContent =
    "Reconstruction QC — measured (x) vs reconstructed (y) per tool, normalized to each tool's range; " +
    "the dashed 1:1 line is a perfect fit. Off-diagonal scatter is that tool's incoherence.";
  host.appendChild(cap);
  const canvas = document.createElement("canvas");
  canvas.className = "mc-hist";
  host.appendChild(canvas);

  const draw = () => drawReconScatter(canvas, series);
  draw();
  return attachResizeRedraw(canvas, draw);
}

/** Per-tool measured-vs-reconstructed scatter, each tool min-max normalized to [0,1] so tools with
 *  very different units (RHOB ~2.5 vs DT ~90) share one square with a single 1:1 reference line. */
function drawReconScatter(canvas: HTMLCanvasElement, series: { label: string; meas: number[]; rec: number[] }[]): void {
  const theme = readTheme(canvas);
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth || 360;
  const h = canvas.clientHeight || 240;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);

  const padL = 34;
  const padB = 24;
  const padT = 8;
  const padR = 10;
  const X = (x: number) => padL + x * (w - padL - padR);
  const Y = (y: number) => padT + (1 - y) * (h - padT - padB);

  ctx.strokeStyle = theme.axis;
  ctx.lineWidth = 1;
  ctx.strokeRect(X(0), Y(1), w - padL - padR, h - padT - padB);
  // 1:1 line.
  ctx.strokeStyle = theme.warn;
  ctx.setLineDash([5, 4]);
  ctx.beginPath();
  ctx.moveTo(X(0), Y(0));
  ctx.lineTo(X(1), Y(1));
  ctx.stroke();
  ctx.setLineDash([]);

  series.forEach((s, si) => {
    // Shared min/max over measured AND reconstructed so a perfect fit lands on the diagonal.
    let lo = Infinity;
    let hi = -Infinity;
    for (const v of s.meas) {
      lo = Math.min(lo, v);
      hi = Math.max(hi, v);
    }
    for (const v of s.rec) {
      lo = Math.min(lo, v);
      hi = Math.max(hi, v);
    }
    const span = hi - lo || 1;
    ctx.fillStyle = faciesColor(si);
    for (let i = 0; i < s.meas.length; i++) {
      const nx = (s.meas[i] - lo) / span;
      const ny = (s.rec[i] - lo) / span;
      ctx.beginPath();
      ctx.arc(X(nx), Y(ny), 1.8, 0, Math.PI * 2);
      ctx.fill();
    }
  });

  // Legend + axis labels.
  ctx.font = canvasFont(theme, 10, 400);
  ctx.textAlign = "left";
  let lx = padL + 4;
  for (let si = 0; si < series.length; si++) {
    ctx.fillStyle = faciesColor(si);
    ctx.fillRect(lx, padT + 2, 8, 8);
    ctx.fillStyle = theme.text;
    ctx.fillText(series[si].label, lx + 11, padT + 10);
    lx += 11 + ctx.measureText(series[si].label).width + 12;
  }
  ctx.fillStyle = theme.text;
  ctx.textAlign = "center";
  ctx.fillText("measured (normalized)", (padL + w - padR) / 2, h - 4);
}
