import { open } from "@tauri-apps/plugin-dialog";
import {
  intakeCommit,
  intakeCommitArrays,
  intakeCommitCurves,
  listDocuments,
  saveDocument,
  deleteDocument,
  intakePaste,
  intakeProbe,
  intakeProbeArrays,
  listWells,
  type ArrayPreview,
  type IntakeColumn,
  type IntakeProbe,
  type IntakeRole,
} from "../ipc";
import { recordProcess } from "../processLog";
import { bumpDataVersion } from "../state";
import { buildDatumSelect, buildFollowCoreRow } from "./followCore";
import { formRow } from "./modal";

/** **Intake** — one importer for any delimited text, replacing the five table-shaped dialogs
 *  (Jauhar, 2026-08-05).
 *
 *  **The grid IS the control.** Click a column header to give it a role and the column tints to
 *  match — the reference importer's idea, and the right one: a mapping is a statement about
 *  columns, and making it in a list beside the data means reading two things at once. What is
 *  The grid shows the file's OWN text — a user needs to see what was delivered — but every cell
 *  that sits in a numeric column and did not parse is painted, so a stray unit, a spreadsheet's
 *  `#N/A` or a depth read under the wrong decimal convention is visible BEFORE anything is
 *  stored. That is the one thing the five dialogs this replaces could only report afterwards.
 *
 *  **Every guess is a proposal with its reason on hover**, and changing the delimiter, the skipped
 *  lines or the decimal convention re-reads live. Nothing here is inferred from a bad result
 *  afterwards, which is the one thing the five dialogs could not do.
 *
 *  A pane rather than a modal: an import is worked through — look at the grid, fix a role, look
 *  again — and a popup covering the workspace is exactly what stops that. */
export async function buildIntakeContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const ROLES: [IntakeRole, string][] = [
    ["WELL", "Well"],
    ["DEPTH", "Depth"],
    ["DEPTH_BASE", "Depth base"],
    ["CPOR", "Porosity"],
    ["CPERM", "Permeability"],
    ["CGD", "Grain density"],
    ["CSW", "Saturation"],
    ["ITEM", "Point item"],
    ["CURVE", "Log curve"],
    ["IGNORE", "Ignore"],
  ];
  /** One tint per role, so a mapping is read at a glance rather than column by column. Drawn
   *  from the accent ramp with an alpha, so every client skin gets its own family for free and
   *  none of them can collide with the table's own borders. */
  const TINT: Record<IntakeRole, string> = {
    WELL: "color-mix(in srgb, var(--accent) 22%, transparent)",
    DEPTH: "color-mix(in srgb, var(--accent) 38%, transparent)",
    DEPTH_BASE: "color-mix(in srgb, var(--accent) 30%, transparent)",
    CPOR: "color-mix(in srgb, var(--ok, #7a8a5e) 30%, transparent)",
    CPERM: "color-mix(in srgb, var(--ok, #7a8a5e) 20%, transparent)",
    CGD: "color-mix(in srgb, var(--warn, #b4642a) 20%, transparent)",
    CSW: "color-mix(in srgb, var(--warn, #b4642a) 30%, transparent)",
    ITEM: "color-mix(in srgb, var(--text-dim) 12%, transparent)",
    CURVE: "color-mix(in srgb, var(--accent2, #7a8a5e) 34%, transparent)",
    IGNORE: "transparent",
  };
  /** The header a tint is painted ON. Every TINT is translucent by design — a role tint has to
   *  read as a tint OF the header surface, not as a colour of its own — and that only works over
   *  something opaque. This header is `position: sticky`, and an inline background beats the
   *  stylesheet, so setting the tint alone leaves nothing beneath it and the scrolling rows show
   *  straight through the header. The tint therefore goes on as a gradient LAYER over the same
   *  --bg-panel-alt the CSS rule paints for a header cell that carries no tint. */
  const headerBg = (role: IntakeRole): string =>
    TINT[role] === "transparent"
      ? "var(--bg-panel-alt)"
      : `linear-gradient(${TINT[role]}, ${TINT[role]}), var(--bg-panel-alt)`;

  let paths: string[] = [];
  let probe: IntakeProbe | null = null;
  let roles: IntakeRole[] = [];

  const content = document.createElement("div");
  content.className = "module-pane intake-pane";

  const head = document.createElement("div");
  head.className = "module-head";
  const chip = document.createElement("span");
  chip.className = "module-chip";
  chip.textContent = "I";
  const titleEl = document.createElement("span");
  titleEl.className = "module-title";
  titleEl.textContent = "Intake";
  head.append(chip, titleEl);
  content.appendChild(head);

  // --- Source ---------------------------------------------------------------
  const fileBar = document.createElement("div");
  fileBar.className = "ribbon-btn-row";
  const pickBtn = document.createElement("button");
  pickBtn.type = "button";
  pickBtn.className = "btn";
  pickBtn.textContent = "Choose files…";
  const pasteBtn = document.createElement("button");
  pasteBtn.type = "button";
  pasteBtn.className = "btn";
  pasteBtn.textContent = "Paste from clipboard";
  const fileLabel = document.createElement("span");
  fileLabel.className = "module-status";
  fileLabel.textContent = "No file chosen.";
  fileBar.append(pickBtn, pasteBtn, fileLabel);
  content.appendChild(fileBar);

  // --- Reading options: every one of these is a GUESS the user can overrule, and each change
  // re-reads the grid so the effect is seen rather than deduced.
  const opts = document.createElement("div");
  opts.className = "module-args";
  const delimSel = document.createElement("select");
  delimSel.className = "form-control";
  for (const [v, l] of [
    ["", "(detect)"],
    [",", "comma"],
    [";", "semicolon"],
    ["\t", "tab"],
    ["ws", "whitespace"],
  ] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = l;
    delimSel.appendChild(o);
  }
  const skipIn = document.createElement("input");
  skipIn.className = "form-control";
  skipIn.type = "number";
  skipIn.min = "0";
  skipIn.value = "0";
  const decSel = document.createElement("select");
  decSel.className = "form-control";
  for (const [v, l] of [
    ["", "(decide per value)"],
    ["dot", "1234.56 — dot is the decimal"],
    ["comma", "1234,56 — comma is the decimal"],
  ] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = l;
    decSel.appendChild(o);
  }
  opts.appendChild(formRow("Delimiter", delimSel));
  opts.appendChild(formRow("Skip lines before header", skipIn, "For a title block or a banner above the column names."));
  opts.appendChild(
    formRow(
      "Decimal",
      decSel,
      "One delivery can use both conventions. Left to decide per value, the rightmost separator " +
        "is taken as the decimal, and a genuinely ambiguous value is reported.",
    ),
  );
  content.appendChild(opts);

  const notes = document.createElement("div");
  notes.className = "module-status";
  content.appendChild(notes);

  // --- The grid -------------------------------------------------------------
  const gridWrap = document.createElement("div");
  gridWrap.className = "intake-grid-wrap";
  content.appendChild(gridWrap);

  // --- What a WIDE or BLOCK read produced ------------------------------------
  // The grid above shows the file's own text; this shows what reading it AS an array made of it —
  // which depth each sample landed on, and what the header row became as an axis. For a block file
  // those depths came from captions the grid displays as ordinary lines, so without this there is
  // nothing on screen that says a caption was understood.
  const arrayPreview = document.createElement("div");
  arrayPreview.className = "intake-array-preview";
  arrayPreview.hidden = true;
  content.appendChild(arrayPreview);

  // --- Destination ----------------------------------------------------------
  const dest = document.createElement("div");
  dest.className = "module-args";
  // The LAYOUT is a declaration, never a sniff. A wide table and a long one are both rectangles
  // of numbers and nothing in the characters says which is which — reading a long Pc table as
  // wide would take its column headers for pressures and store an array of column indices.
  const layoutSel = document.createElement("select");
  layoutSel.className = "form-control";
  for (const [v, l] of [
    ["long", "Long — one row per point (the usual table)"],
    ["wide", "Wide — one row per sample, the header row is the axis"],
    ["block", "Block — several tables stacked, the header repeated"],
  ] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = l;
    layoutSel.appendChild(o);
  }
  const arrayName = document.createElement("input");
  arrayName.className = "form-control";
  arrayName.type = "text";
  arrayName.placeholder = "e.g. PC_SW, T2";
  const setIn = document.createElement("input");
  setIn.className = "form-control";
  setIn.type = "text";
  setIn.placeholder = "e.g. RCA-2026";
  const dsIn = document.createElement("input");
  dsIn.className = "form-control";
  dsIn.type = "text";
  dsIn.value = "CORE";
  const unitSel = document.createElement("select");
  unitSel.className = "form-control";
  for (const [v, l] of [
    ["", "(as delivered)"],
    ["m", "metres"],
    ["ft", "feet"],
  ] as [string, string][]) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = l;
    unitSel.appendChild(o);
  }
  const wellSel = document.createElement("select");
  wellSel.className = "form-control";
  const noneOpt = document.createElement("option");
  noneOpt.value = "";
  noneOpt.textContent = "(route by the Well column)";
  wellSel.appendChild(noneOpt);
  void listWells({ kind: "all" })
    .then((wells) => {
      for (const w of wells) {
        const o = document.createElement("option");
        o.value = w.well_id;
        o.textContent = w.well_name;
        wellSel.appendChild(o);
      }
    })
    .catch(() => {});
  dest.appendChild(
    formRow(
      "Layout",
      layoutSel,
      "Declared, never guessed: a wide table and a long one are both rectangles of numbers, and reading one as the other stores an array of column indices.",
    ),
  );
  const arrayRow = formRow(
    "Array name",
    arrayName,
    "Wide and Block store an ARRAY per sample — a Pc curve, an NMR T2 distribution — under this name, with the header row as its axis.",
  );
  arrayRow.hidden = true;
  dest.appendChild(arrayRow);
  // Point data and following the core belong to the LONG path; an array carries neither.
  const pointRows: HTMLElement[] = [];
  layoutSel.addEventListener("change", () => {
    const arrays = layoutSel.value !== "long";
    arrayRow.hidden = !arrays;
    for (const r of pointRows) r.hidden = arrays;
    // The layout is the declaration that decides how the file is read at all, so it re-reads.
    lastPreview = null;
    void refreshArrayPreview();
  });
  dest.appendChild(
    formRow(
      "Delivery set",
      setIn,
      "The files chosen together are ONE delivery. A name already used on a well is auto-suffixed — an import never overwrites.",
    ),
  );
  const pointSetRow = formRow(
    "Point-data set",
    dsIn,
    "Where columns no measurement role claimed land. They are carried, never dropped.",
  );
  pointRows.push(pointSetRow);
  dest.appendChild(pointSetRow);
  dest.appendChild(formRow("Depths are in", unitSel, "Converted to the project's unit on import."));
  dest.appendChild(
    formRow("If no Well column", wellSel, "Only used when the table carries no well name of its own."),
  );
  const datumSel = buildDatumSelect();
  const datumRow = formRow(
    "Depth datum",
    datumSel,
    "The datum the delivery's depths are quoted in (declared once for the whole delivery).",
  );
  pointRows.push(datumRow);
  dest.appendChild(datumRow);
  const follow = buildFollowCoreRow("these rows", "intake");
  pointRows.push(follow.el);
  dest.appendChild(follow.el);
  content.appendChild(dest);

  // --- Saved mappings -------------------------------------------------------
  //
  // A recurring delivery arrives in the same shape every quarter, and re-declaring nine column
  // roles each time is both tedious and a place to differ from last time without noticing — which
  // is the worse half: two quarters of one core imported under different mappings look like one
  // consistent dataset.
  //
  // Stored as a `documents` row (`intaketmpl`), the plot-template precedent. It holds the
  // DECISIONS — roles, layout, delimiter, decimal, units, destination names — and never the
  // file: a template is how to read a shape, not what was in one of them.
  //
  // **Applied by HEADER NAME, never by position.** A delivery that gains a column would otherwise
  // shift every role one to the right, silently, and a saved mapping exists precisely for the
  // deliveries nobody re-checks. Columns the template does not name keep whatever Intake proposed
  // for them and are reported, so a new column is visible rather than quietly IGNOREd.
  const TMPL_DOC = "intaketmpl";
  interface IntakeTemplate {
    headers: string[];
    roles: string[];
    layout: string;
    delimiter: string;
    skip: number;
    decimal: string;
    depthUnit: string;
    set: string;
    dataset: string;
    arrayName: string;
  }

  const tmplSelect = document.createElement("select");
  tmplSelect.className = "form-control";
  const tmplName = document.createElement("input");
  tmplName.className = "form-control";
  tmplName.type = "text";
  tmplName.placeholder = "name this mapping";
  const tmplSave = document.createElement("button");
  tmplSave.type = "button";
  tmplSave.className = "btn";
  tmplSave.textContent = "Save";
  const tmplApply = document.createElement("button");
  tmplApply.type = "button";
  tmplApply.className = "btn";
  tmplApply.textContent = "Apply";
  const tmplDel = document.createElement("button");
  tmplDel.type = "button";
  tmplDel.className = "btn";
  tmplDel.textContent = "Delete";

  async function refreshTemplates(keep?: string): Promise<void> {
    const docs = await listDocuments(TMPL_DOC).catch(() => []);
    tmplSelect.innerHTML = "";
    const ph = document.createElement("option");
    ph.value = "";
    ph.textContent = docs.length ? "— saved mappings —" : "(none saved)";
    tmplSelect.appendChild(ph);
    for (const d of docs) {
      const o = document.createElement("option");
      o.value = d.name;
      o.textContent = d.name;
      tmplSelect.appendChild(o);
    }
    if (keep) tmplSelect.value = keep;
  }

  tmplSave.addEventListener("click", async () => {
    const name = tmplName.value.trim();
    if (!name) {
      result.textContent = "Name the mapping before saving it.";
      tmplName.focus();
      return;
    }
    if (!probe) {
      result.textContent = "Read a file first — a mapping is saved against its column names.";
      return;
    }
    const doc: IntakeTemplate = {
      headers: probe.columns.map((c) => c.header.trim().toUpperCase()),
      roles: [...roles],
      layout: layoutSel.value,
      delimiter: delimSel.value,
      skip: Number(skipIn.value) || 0,
      decimal: decSel.value,
      depthUnit: unitSel.value,
      set: setIn.value,
      dataset: dsIn.value,
      arrayName: arrayName.value,
    };
    await saveDocument(TMPL_DOC, name, JSON.stringify(doc));
    await refreshTemplates(name);
    result.textContent = `Saved mapping "${name}".`;
  });

  tmplApply.addEventListener("click", async () => {
    const name = tmplSelect.value;
    if (!name || !probe) return;
    const docs = await listDocuments(TMPL_DOC).catch(() => []);
    const doc = docs.find((d) => d.name === name);
    if (!doc) return;
    let t: IntakeTemplate;
    try {
      t = JSON.parse(doc.json) as IntakeTemplate;
    } catch {
      result.textContent = `Mapping "${name}" could not be read.`;
      return;
    }
    // By header NAME. A delivery that gained a column would shift every role one to the right if
    // this went by position, and it would look exactly like a correct import.
    const byHeader = new Map<string, string>();
    (t.headers ?? []).forEach((h, i) => byHeader.set(h, (t.roles ?? [])[i] ?? "IGNORE"));
    const unknown: string[] = [];
    probe.columns.forEach((c, i) => {
      const want = byHeader.get(c.header.trim().toUpperCase());
      if (want) roles[i] = want as IntakeRole;
      else unknown.push(c.header);
    });
    layoutSel.value = t.layout || "long";
    layoutSel.dispatchEvent(new Event("change", { bubbles: true }));
    delimSel.value = t.delimiter ?? "";
    skipIn.value = String(t.skip ?? 0);
    decSel.value = t.decimal ?? "";
    unitSel.value = t.depthUnit ?? "";
    setIn.value = t.set ?? "";
    dsIn.value = t.dataset ?? dsIn.value;
    arrayName.value = t.arrayName ?? "";
    renderGrid();
    // Named, not silent: a new column in a recurring delivery is exactly what a saved mapping
    // stops anyone from looking at.
    result.textContent = unknown.length
      ? `Applied "${name}". ${unknown.length} column(s) the mapping does not name kept their proposed role: ${unknown.join(", ")}`
      : `Applied "${name}".`;
  });

  tmplDel.addEventListener("click", async () => {
    const name = tmplSelect.value;
    if (!name) return;
    await deleteDocument(TMPL_DOC, name);
    await refreshTemplates();
    result.textContent = `Deleted mapping "${name}".`;
  });

  const tmplRow = document.createElement("div");
  tmplRow.className = "intake-template-row";
  tmplRow.append(tmplSelect, tmplApply, tmplName, tmplSave, tmplDel);
  content.appendChild(formRow("Saved mapping", tmplRow, "Applied by column NAME, never by position — a delivery that gains a column would otherwise shift every role one to the right."));
  void refreshTemplates();

  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.className = "btn btn-accent form-run-btn";
  runBtn.textContent = "Import";
  runBtn.disabled = true;
  runRow.appendChild(runBtn);
  content.appendChild(runRow);

  const result = document.createElement("div");
  result.className = "module-status";
  content.appendChild(result);

  // --- Rendering ------------------------------------------------------------
  function renderGrid(): void {
    gridWrap.innerHTML = "";
    if (!probe) {
      const empty = document.createElement("div");
      empty.className = "module-status";
      empty.textContent = "Choose a file, or paste a table, to see it here.";
      gridWrap.appendChild(empty);
      return;
    }
    const table = document.createElement("table");
    table.className = "intake-grid";

    // Header row: the column name, its sniffed kind, and a role picker whose value tints the
    // whole column. The picker is IN the header rather than in a separate list — the mapping is
    // a statement about a column, so it belongs on the column.
    const thead = document.createElement("thead");
    const nameRow = document.createElement("tr");
    const roleRow = document.createElement("tr");
    probe.columns.forEach((col: IntakeColumn, i: number) => {
      const th = document.createElement("th");
      const nm = document.createElement("div");
      nm.className = "intake-col-name";
      nm.textContent = col.header || `(column ${i + 1})`;
      const kind = document.createElement("div");
      kind.className = "intake-col-kind";
      kind.textContent = `${col.kind} · ${col.filled} filled`;
      th.append(nm, kind);
      th.style.background = headerBg(roles[i]);
      nameRow.appendChild(th);

      const rt = document.createElement("th");
      const sel = document.createElement("select");
      sel.className = "form-control intake-role";
      for (const [v, l] of ROLES) {
        const o = document.createElement("option");
        o.value = v;
        o.textContent = l;
        if (v === roles[i]) o.selected = true;
        sel.appendChild(o);
      }
      // The reason lives on the picker, so a proposal can be argued with rather than accepted
      // because it is the only thing on offer.
      sel.title = col.reason;
      sel.addEventListener("change", () => {
        roles[i] = sel.value as IntakeRole;
        renderGrid();
        validate();
        // WELL and DEPTH change which columns are axis bins, so an array read is a different read.
        void refreshArrayPreview();
      });
      rt.appendChild(sel);
      rt.style.background = headerBg(roles[i]);
      roleRow.appendChild(rt);
    });
    thead.append(nameRow, roleRow);
    table.appendChild(thead);

    // Cells that failed to parse in a numeric column, keyed for an O(1) lookup per cell rather
    // than a scan of the list per cell — a 200-row preview of a wide table is thousands of cells.
    const bad = new Set(probe.preview_bad.map(([r, c]) => `${r},${c}`));
    const tbody = document.createElement("tbody");
    probe.preview.forEach((row, r) => {
      const tr = document.createElement("tr");
      row.forEach((cell, i) => {
        const td = document.createElement("td");
        td.textContent = cell;
        td.style.background = TINT[roles[i]];
        if (bad.has(`${r},${i}`)) {
          td.classList.add("intake-bad");
          td.title = "This column reads as numbers and this cell does not.";
        }
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);
    gridWrap.appendChild(table);

    const foot = document.createElement("div");
    foot.className = "module-status";
    const shown = Math.min(probe.preview.length, probe.n_rows);
    foot.textContent =
      `${probe.n_rows} row(s), ${probe.columns.length} column(s), ${probe.delimiter}-delimited` +
      (shown < probe.n_rows ? ` — showing the first ${shown}.` : ".");
    gridWrap.appendChild(foot);
  }

  // Stale answers are dropped by sequence rather than cancelled: a role click can outrun a file
  // read, and a preview of the mapping before last is worse than none — it would be a picture of a
  // decision the user has already changed.
  let previewSeq = 0;
  let lastPreview: ArrayPreview | null = null;

  /** How many of a sample's bins the preview draws before it says "and the rest". */
  const SHOWN_BINS = 8;

  async function refreshArrayPreview(): Promise<void> {
    const block = layoutSel.value === "block";
    if (!probe || layoutSel.value === "long" || !paths[0]) {
      arrayPreview.hidden = true;
      arrayPreview.innerHTML = "";
      return;
    }
    const seq = ++previewSeq;
    let pv: ArrayPreview;
    try {
      pv = await intakeProbeArrays(
        paths[0],
        {
          delimiter: delimSel.value || undefined,
          skip_lines: Math.max(0, parseInt(skipIn.value, 10) || 0),
          decimal: decSel.value || undefined,
        },
        roles,
        block,
      );
    } catch (e) {
      if (seq !== previewSeq) return;
      arrayPreview.hidden = false;
      arrayPreview.innerHTML = "";
      const err = document.createElement("div");
      err.className = "module-status";
      err.style.color = "var(--warn)";
      err.textContent = String(e);
      arrayPreview.appendChild(err);
      return;
    }
    if (seq !== previewSeq) return;
    renderArrayPreview(pv);
  }

  function renderArrayPreview(pv: ArrayPreview): void {
    lastPreview = pv;
    // One-way: the preview settles whether a BLOCK file has depths at all, so validate() reads it
    // and must never be what triggers the fetch — that is a loop with a file read in it.
    validate();
    arrayPreview.hidden = false;
    arrayPreview.innerHTML = "";

    const head = document.createElement("div");
    head.className = "module-status";
    const axisFrom = pv.axis_labels[0] ?? "";
    const axisTo = pv.axis_labels[pv.axis_labels.length - 1] ?? "";
    head.textContent =
      `${pv.n_rows} sample(s) × ${pv.axis.length} bin(s)` +
      (pv.axis.length
        ? `, axis ${pv.axis[0]} to ${pv.axis[pv.axis.length - 1]}` +
          // The header TEXT beside the number it parsed to: `100 psi` reading as 100 is the kind
          // of thing that is obviously right once seen and impossible to check otherwise.
          (axisFrom !== String(pv.axis[0]) || axisTo !== String(pv.axis[pv.axis.length - 1])
            ? ` (read from "${axisFrom}" … "${axisTo}")`
            : "")
        : "");
    arrayPreview.appendChild(head);

    for (const n of pv.notes) {
      const line = document.createElement("div");
      line.className = "module-status";
      // A duplicate is not commentary — it names samples the store would refuse — so it reads as a
      // warning while the ordinary "2 block(s) keyed by a label line" stays plain.
      if (n.includes("carry more than one sample") || n.includes("key more than one block")) {
        line.style.color = "var(--warn)";
      }
      line.textContent = n;
      arrayPreview.appendChild(line);
    }

    if (!pv.rows.length) return;

    const clashing = (r: ArrayPreview["rows"][number]): boolean => {
      if (r.depth == null) return false;
      const w = r.well_name ? r.well_name.trim().toUpperCase() : null;
      return pv.clashes.some(
        (c) => Math.abs(c.depth - r.depth!) < 1e-9 && (c.well === null || c.well === w),
      );
    };
    const anyWell = pv.rows.some((r) => r.well_name);

    const wrap = document.createElement("div");
    wrap.className = "intake-grid-wrap";
    const table = document.createElement("table");
    table.className = "intake-grid";
    const thead = document.createElement("thead");
    const hr = document.createElement("tr");
    const cols = ["#", ...(anyWell ? ["Well"] : []), "Depth"];
    const shown = pv.axis_labels.slice(0, SHOWN_BINS);
    for (const c of [...cols, ...shown, ...(pv.axis_labels.length > SHOWN_BINS ? ["…"] : [])]) {
      const th = document.createElement("th");
      th.textContent = c;
      hr.appendChild(th);
    }
    thead.appendChild(hr);
    table.appendChild(thead);

    const tbody = document.createElement("tbody");
    pv.rows.forEach((r, i) => {
      const tr = document.createElement("tr");
      const bad = clashing(r);
      if (bad) tr.classList.add("intake-dupe");
      const cells: string[] = [
        // The row's place in the FILE, not in this table — a duplicate pulled in from beyond the
        // cap would otherwise appear to follow the row above it.
        String((pv.row_index[i] ?? i) + 1),
        ...(anyWell ? [r.well_name ?? ""] : []),
        // An empty depth is left EMPTY rather than shown as 0: a sample with no depth has nowhere
        // to go and is not stored, which a zero would misreport as the top of the well.
        r.depth == null ? "" : r.depth.toFixed(2),
      ];
      for (const v of r.values.slice(0, SHOWN_BINS)) {
        cells.push(Number.isFinite(v) ? String(v) : "");
      }
      if (pv.axis_labels.length > SHOWN_BINS) cells.push("…");
      for (const c of cells) {
        const td = document.createElement("td");
        td.textContent = c;
        tr.appendChild(td);
      }
      if (bad) {
        tr.title =
          "Another sample of this well sits at this depth. Only one measurement can be stored " +
          "per depth, so the rest would be refused.";
      }
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);
    wrap.appendChild(table);
    arrayPreview.appendChild(wrap);

    if (pv.n_rows > pv.rows.length) {
      const foot = document.createElement("div");
      foot.className = "module-status";
      foot.textContent = `Showing ${pv.rows.length} of ${pv.n_rows} sample(s) — every duplicate is included whatever its place in the file.`;
      arrayPreview.appendChild(foot);
    }
  }

  function validate(): void {
    // A BLOCK file keyed by captions has NO depth column — that is the whole point of reading the
    // captions — so requiring one here made that path unreachable from the pane: the reader would
    // resolve every block correctly and the Import button stayed disabled. The preview is what
    // settles it, because whether the captions actually yielded depths is a fact about the file
    // rather than about the roles.
    const captioned =
      layoutSel.value === "block" && !!lastPreview && lastPreview.rows.some((r) => r.depth != null);
    const hasDepth = roles.includes("DEPTH") || captioned;
    runBtn.disabled = !probe || !hasDepth;
    if (probe && !hasDepth) {
      // Refused in the pane, where the user is looking (the needWell.ts rule), rather than as an
      // error after the import button.
      notes.textContent =
        layoutSel.value === "block"
          ? "Mark one column DEPTH, or caption each block with a depth carrying a UNIT " +
            "(`PLUG 12  4633.5 ft`) — every sample has to land at a depth, and nothing here says " +
            "where these came from."
          : "Mark one column DEPTH before importing — every row has to land at a depth, and there is " +
            "nothing to store without one.";
      notes.style.color = "var(--warn)";
    } else if (probe) {
      notes.textContent = probe.notes.join(" • ");
      notes.style.color = "";
    }
  }

  async function reprobe(path: string): Promise<void> {
    try {
      probe = await intakeProbe(path, {
        delimiter: delimSel.value || undefined,
        skip_lines: Math.max(0, parseInt(skipIn.value, 10) || 0),
        decimal: decSel.value || undefined,
      });
      roles = probe.columns.map((c) => c.role);
      if (probe.depth_unit_guess && !unitSel.value) unitSel.value = probe.depth_unit_guess;
      renderGrid();
      validate();
      void refreshArrayPreview();
    } catch (e) {
      notes.textContent = String(e);
      notes.style.color = "var(--warn)";
    }
  }

  for (const el of [delimSel, decSel, skipIn]) {
    el.addEventListener("change", () => {
      if (paths[0]) void reprobe(paths[0]);
    });
  }

  pickBtn.addEventListener("click", async () => {
    const picked = await open({
      multiple: true,
      filters: [{ name: "Delimited text", extensions: ["csv", "txt", "tsv", "dat", "asc"] }],
    });
    if (!picked) return;
    paths = Array.isArray(picked) ? picked : [picked];
    fileLabel.textContent =
      paths.length === 1 ? paths[0] : `${paths.length} files — the first is shown below.`;
    // Only the FIRST file is probed, and the confirmed mapping is applied to all of them. That
    // is the coreImportDialog rule: a mapping is confirmed once BY HEADER NAME, because a
    // delivery split across files is one delivery with one shape.
    await reprobe(paths[0]);
  });

  pasteBtn.addEventListener("click", async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) {
        notes.textContent = "The clipboard holds no text.";
        return;
      }
      // Pasted text becomes a file and takes the identical path — one parser, one commit, so a
      // paste cannot behave differently from the same table on disk.
      const full = await intakePaste(text);
      paths = [full];
      fileLabel.textContent = "Pasted table";
      await reprobe(full);
    } catch (e) {
      notes.textContent = `Could not read the clipboard: ${e}`;
    }
  });

  runBtn.addEventListener("click", async () => {
    if (!probe) return;
    runBtn.disabled = true;
    result.textContent = "Importing…";
    // Cleared per run, or a warning colour outlives the import that earned it and a clean second
    // attempt still reads as failed.
    result.style.color = "";
    try {
      // A WIDE or BLOCK table is an ARRAY per sample — a Pc curve, an NMR T2 distribution — and
      // goes to the array store with its axis. LONG is point data and takes the ordinary path.
      if (layoutSel.value !== "long") {
        if (!arrayName.value.trim()) {
          result.textContent = "Name the array before importing it — it is stored under that name.";
          arrayName.focus();
          runBtn.disabled = false;
          return;
        }
        const ares = await intakeCommitArrays({
          paths,
          roles,
          layout: layoutSel.value,
          curve_name: arrayName.value.trim(),
          set_name: setIn.value.trim() || undefined,
          depth_unit: unitSel.value || undefined,
          fallback_well_id: wellSel.value || undefined,
        });
        const samples = ares.reduce((a, r) => a + r.samples, 0);
        const wells = ares.reduce((a, r) => a + r.wells, 0);
        const bins = ares[0]?.bins ?? 0;
        const anotes = ares.flatMap((r) => r.notes);
        const aerrs = ares.filter((r) => r.error).map((r) => `${r.path}: ${r.error}`);
        result.textContent =
          `${samples} sample(s) x ${bins} bin(s) into ${wells} well(s)` +
          (ares[0] ? `, axis ${ares[0].axis_first} to ${ares[0].axis_last}` : "") +
          (aerrs.length ? ` — ${aerrs.join("; ")}` : ".") +
          (anotes.length ? ` ${anotes.join(" ")}` : "");
        // A duplicate depth is not commentary — an array holds one vector per depth, so it names
        // samples that could not be stored. It reads as a warning or it gets skimmed past with the
        // sample count, which is exactly the number it contradicts.
        const clashed = anotes.some(
          (n) => n.includes("carry more than one sample") || n.includes("key more than one block"),
        );
        result.style.color = aerrs.length || clashed ? "var(--warn)" : "";
        setStatus(`Intake: ${samples} array sample(s) into ${wells} well(s)`);
        recordProcess("Import", `Intake arrays: ${samples} sample(s) from ${paths.length} file(s)`);
        bumpDataVersion();
        runBtn.disabled = false;
        return;
      }
      // Columns marked CURVE are continuous logs and go to the curve store, at the same depths,
      // in the same delivery. Run BEFORE the point-data commit so one file can carry both — a
      // wireline export with a lithology description beside it is one delivery, not two.
      const curveCols = roles.filter((r) => r === "CURVE").length;
      if (curveCols > 0) {
        const cres = await intakeCommitCurves({
          paths,
          roles,
          set_name: setIn.value.trim() || undefined,
          depth_unit: unitSel.value || undefined,
          fallback_well_id: wellSel.value || undefined,
        });
        const cs = cres.reduce((a, r) => a + r.samples, 0);
        const names = [...new Set(cres.flatMap((r) => r.curves))];
        const cerr = cres.filter((r) => r.error).map((r) => `${r.path}: ${r.error}`);
        result.textContent =
          `${cs} sample(s) of ${names.join(", ") || "no curve"} into the curve store` +
          (cerr.length ? ` — ${cerr.join("; ")}` : ".");
        setStatus(`Intake: ${cs} curve sample(s), ${names.length} curve(s)`);
        recordProcess("Import", `Intake curves: ${names.join(", ")}`);
        bumpDataVersion();
        // A file of pure logs has nothing left for the point-data path.
        if (!roles.some((r) => r !== "CURVE" && r !== "IGNORE" && r !== "DEPTH" && r !== "WELL")) {
          runBtn.disabled = false;
          return;
        }
      }
      const res = await intakeCommit({
        paths,
        roles,
        depth_unit: unitSel.value || undefined,
        depth_datum: datumSel.value,
        set_name: setIn.value.trim() || undefined,
        extras_dataset: dsIn.value.trim() || undefined,
        fallback_well_id: wellSel.value || undefined,
        follow_core: follow.checked(),
      });
      const rows = res.reduce((a, r) => a + r.rows_imported, 0);
      const wells = res.reduce((a, r) => a + r.wells_imported, 0);
      const extras = res.reduce((a, r) => a + r.extra_rows, 0);
      const errs = res.filter((r) => r.error).map((r) => `${r.path}: ${r.error}`);
      result.textContent =
        `${rows} row(s) into ${wells} well(s)` +
        (extras ? `, ${extras} point-data row(s) carried` : "") +
        (errs.length ? ` — ${errs.join("; ")}` : ".");
      setStatus(`Intake: ${rows} row(s) into ${wells} well(s)`);
      recordProcess("Import", `Intake: ${rows} row(s) from ${paths.length} file(s)`);
      bumpDataVersion();
    } catch (e) {
      result.textContent = String(e);
    } finally {
      runBtn.disabled = false;
    }
  });

  renderGrid();
  return { el: content, dispose: () => {} };
}
