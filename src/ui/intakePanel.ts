import { open } from "@tauri-apps/plugin-dialog";
import {
  intakeCommit,
  intakePaste,
  intakeProbe,
  listWells,
  type IntakeColumn,
  type IntakeProbe,
  type IntakeRole,
} from "../ipc";
import { recordProcess } from "../processLog";
import { bumpDataVersion } from "../state";
import { buildFollowCoreRow } from "./followCore";
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
    IGNORE: "transparent",
  };

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

  // --- Destination ----------------------------------------------------------
  const dest = document.createElement("div");
  dest.className = "module-args";
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
  void listWells()
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
      "Delivery set",
      setIn,
      "The files chosen together are ONE delivery. A name already used on a well is auto-suffixed — an import never overwrites.",
    ),
  );
  dest.appendChild(
    formRow(
      "Point-data set",
      dsIn,
      "Where columns no measurement role claimed land. They are carried, never dropped.",
    ),
  );
  dest.appendChild(formRow("Depths are in", unitSel, "Converted to the project's unit on import."));
  dest.appendChild(
    formRow("If no Well column", wellSel, "Only used when the table carries no well name of its own."),
  );
  const follow = buildFollowCoreRow("these rows", "intake");
  dest.appendChild(follow.el);
  content.appendChild(dest);

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
      th.style.background = TINT[roles[i]];
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
      });
      rt.appendChild(sel);
      rt.style.background = TINT[roles[i]];
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

  function validate(): void {
    const hasDepth = roles.includes("DEPTH");
    runBtn.disabled = !probe || !hasDepth;
    if (probe && !hasDepth) {
      // Refused in the pane, where the user is looking (the needWell.ts rule), rather than as an
      // error after the import button.
      notes.textContent =
        "Mark one column DEPTH before importing — every row has to land at a depth, and there is " +
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
    try {
      const res = await intakeCommit({
        paths,
        roles,
        depth_unit: unitSel.value || undefined,
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
