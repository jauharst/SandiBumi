import {
  importCoreTable,
  probeCoreTable,
  type CoreMapping,
  type CoreTableImportResult,
  type TableProbe,
} from "../ipc";
import { setStatus } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";

/** Core import v2 wizard (T-IMP-07): probe → CONFIRM → commit.
 *
 *  A core delivery is a wide lab export whose meaning must be confirmed, not assumed:
 *  which column is the well name (BLSO writes WN, Duri writes WELL NAME beside a numeric
 *  WELL), which is depth and in what unit (feet vs the project's metres is a silent 3.28×
 *  error), which columns are porosity/perm/grain-density/Sw, and whether porosity is in
 *  percent. The dialog shows everything the probe detected, lets the user fix it, and
 *  only then writes — per well, replace-on-reimport.
 *
 *  Multi-file: the mapping is confirmed ONCE by HEADER NAME and re-resolved to a column
 *  index per file, so a delivery of one-CSV-per-well (the BLSO shape) imports in one go
 *  even if some files order their columns differently. A file missing a mapped header
 *  reports and skips, never guesses.
 */

interface RoleSpec {
  key: keyof CoreMapping;
  label: string;
  hint: string;
  required?: boolean;
}

const ROLES: RoleSpec[] = [
  { key: "well", label: "Well name", hint: "Rows route to project wells by this column's value. '—' sends every row to the selected well." },
  { key: "depth", label: "Depth", hint: "Plug depth (MD). Required.", required: true },
  { key: "cpor", label: "Porosity (CPOR)", hint: "Core porosity. Percent is detected and converted to v/v." },
  { key: "cperm", label: "Permeability (CPERM)", hint: "Stays in mD." },
  { key: "cgd", label: "Grain density (CGD)", hint: "g/cc." },
  { key: "csw", label: "Water saturation (CSW)", hint: "Percent is detected and converted to v/v." },
];

/** The dialog's confirmed choice: header NAME per role (re-resolved per file). */
export interface CoreImportChoice {
  headers: Partial<Record<keyof CoreMapping, string>>;
  depthUnit: string | null; // "ft" | "m" | null = already the project unit
}

/** Resolves a by-name choice against one file's probed headers. Returns null (with a
 *  reason) when the depth header is missing in this file. */
export function mappingForFile(
  choice: CoreImportChoice,
  probe: TableProbe,
): { mapping: CoreMapping; missing: string[] } | { error: string } {
  const idx = (name: string | undefined): number | null => {
    if (!name) return null;
    const i = probe.headers.indexOf(name);
    return i >= 0 ? i : null;
  };
  const depth = idx(choice.headers.depth);
  if (depth === null) {
    return { error: `no '${choice.headers.depth ?? "?"}' column in this file` };
  }
  const missing: string[] = [];
  const opt = (key: keyof CoreMapping): number | null => {
    const name = choice.headers[key];
    if (!name) return null;
    const i = idx(name);
    if (i === null) missing.push(name);
    return i;
  };
  return {
    mapping: {
      well: opt("well"),
      depth,
      cpor: opt("cpor"),
      cperm: opt("cperm"),
      cgd: opt("cgd"),
      csw: opt("csw"),
    },
    missing,
  };
}

/** Aggregates per-file results into the status line + History entries. */
function report(results: { path: string; res: CoreTableImportResult | null; skipped?: string }[], well: { well_name: string } | null): void {
  let rows = 0;
  let wells = new Set<string>();
  const problems: string[] = [];
  for (const { path, res, skipped } of results) {
    const base = path.replace(/\\/g, "/").split("/").pop() ?? path;
    if (skipped) {
      problems.push(`${base}: ${skipped}`);
      continue;
    }
    if (!res) continue;
    if (res.error) {
      problems.push(`${base}: ${res.error}`);
      continue;
    }
    rows += res.rows_imported;
    for (const o of res.outcomes) {
      if (o.imported > 0) wells.add(o.well_name);
      else if (o.problem) problems.push(`${o.well_name}: ${o.problem}`);
    }
    if (res.skipped_blank_well > 0) problems.push(`${base}: ${res.skipped_blank_well} blank-well row(s) skipped`);
    recordProcess("Import", `Imported ${res.rows_imported} core sample(s) into ${res.wells_imported} well(s) ← ${path}`);
  }
  const probNote = problems.length ? ` ${problems.length} issue(s): ${problems.slice(0, 3).join("; ")}${problems.length > 3 ? "; …" : ""}` : "";
  if (rows === 0) {
    setStatus(`Core import: nothing imported.${probNote || " (no matching wells?)"}`);
  } else {
    setStatus(`Imported ${rows} core sample(s) into ${wells.size} well(s).${probNote}`);
  }
  if (problems.length) {
    for (const p of problems) recordProcess("Import", `Core import issue — ${p}`, well?.well_name);
  }
}

/**
 * Runs the whole wizard: probes `paths`, shows the confirm dialog, commits on Import.
 * `fallbackWell` (the selected well) receives rows only when no well column is mapped.
 * Calls `onDone` after a successful commit so the workspace can refresh.
 */
export async function openCoreImportWizard(
  paths: string[],
  fallbackWell: { well_id: string; well_name: string } | null,
  onDone: () => void,
): Promise<void> {
  setStatus(`Reading ${paths.length} file(s)…`);
  // Probe every file; the FIRST successful probe seeds the mapping UI.
  const probes: { path: string; probe: TableProbe | null; err?: string }[] = [];
  for (const p of paths) {
    try {
      probes.push({ path: p, probe: await probeCoreTable(p) });
    } catch (err) {
      probes.push({ path: p, probe: null, err: String(err) });
    }
  }
  const first = probes.find((p) => p.probe);
  if (!first?.probe) {
    setStatus(`Core import failed: ${probes[0]?.err ?? "no readable file"}`);
    return;
  }
  const lead = first.probe;

  const wrap = document.createElement("div");

  const summary = document.createElement("p");
  summary.className = "form-hint";
  const unreadable = probes.filter((p) => !p.probe).length;
  summary.textContent =
    `${paths.length} file(s), ${probes.reduce((n, p) => n + (p.probe?.n_rows ?? 0), 0)} data row(s)` +
    (unreadable ? ` — ${unreadable} unreadable file(s) will be skipped` : "") +
    (lead.units_row_skipped ? " — a units row under the headers was detected and is skipped" : "");
  wrap.appendChild(summary);

  // --- Role → header selects, seeded from the lead probe's guesses. ---
  const selects = new Map<keyof CoreMapping, HTMLSelectElement>();
  const headerName = (i: number | null): string => (i === null ? "" : lead.headers[i] ?? "");
  for (const role of ROLES) {
    const sel = document.createElement("select");
    sel.className = "form-control";
    if (!role.required) {
      const none = document.createElement("option");
      none.value = "";
      none.textContent = "—";
      sel.appendChild(none);
    }
    lead.headers.forEach((h, i) => {
      const opt = document.createElement("option");
      opt.value = h;
      const kind = lead.column_kind[i];
      opt.textContent = kind && kind !== "number" ? `${h} (${kind})` : h;
      sel.appendChild(opt);
    });
    sel.value = headerName((lead[role.key as keyof TableProbe] as number | null) ?? null);
    selects.set(role.key, sel);
    wrap.appendChild(formRow(role.label, sel, role.hint));
  }

  // --- Depth unit (the silent-3.28× guard). ---
  const unitSel = document.createElement("select");
  unitSel.className = "form-control";
  for (const [v, label] of [["", "Same as project"], ["m", "Metres (m)"], ["ft", "Feet (ft)"]] as const) {
    const opt = document.createElement("option");
    opt.value = v;
    opt.textContent = label;
    unitSel.appendChild(opt);
  }
  unitSel.value = lead.depth_unit_guess ?? "";
  wrap.appendChild(
    formRow(
      "Depth unit in file",
      unitSel,
      "Converted to the project's depth unit on import. Detected from the units row or the depth header when possible.",
    ),
  );

  // --- Routing + percent notes (live: follows the well select). ---
  const routing = document.createElement("p");
  routing.className = "form-hint";
  const updateRouting = () => {
    const wellHeader = selects.get("well")?.value ?? "";
    if (!wellHeader) {
      routing.textContent = fallbackWell
        ? `No well column — every row goes to the selected well: ${fallbackWell.well_name}.`
        : "No well column and NO WELL SELECTED — pick a well column or select a well first.";
      return;
    }
    if (wellHeader === headerName(lead.well) && lead.wells.length > 0) {
      const names = lead.wells.slice(0, 6).map((w) => `${w.name} (${w.rows})`).join(", ");
      routing.textContent =
        `Routing by ${wellHeader} — ${lead.wells.length} well name(s) in the first file: ${names}` +
        (lead.wells.length > 6 ? ", …" : "") +
        ". Names must match project wells exactly (case-blind); unmatched names are reported, never guessed.";
    } else {
      routing.textContent = `Routing by ${wellHeader}. Unmatched names are reported, never guessed.`;
    }
  };
  selects.get("well")?.addEventListener("change", updateRouting);
  updateRouting();
  wrap.appendChild(routing);

  if (lead.percent_roles.length > 0) {
    const pct = document.createElement("p");
    pct.className = "form-hint";
    pct.textContent = `${lead.percent_roles.join(" and ")} read as PERCENT — values are divided by 100 to v/v on import.`;
    wrap.appendChild(pct);
  }

  // --- Sample preview (first file, first 5 rows). ---
  if (lead.sample_rows.length > 0) {
    const scroll = document.createElement("div");
    scroll.className = "core-import-preview";
    const table = document.createElement("table");
    const thead = document.createElement("tr");
    for (const h of lead.headers) {
      const th = document.createElement("th");
      th.textContent = h;
      thead.appendChild(th);
    }
    table.appendChild(thead);
    for (const row of lead.sample_rows) {
      const tr = document.createElement("tr");
      lead.headers.forEach((_, i) => {
        const td = document.createElement("td");
        td.textContent = row[i] ?? "";
        tr.appendChild(td);
      });
      table.appendChild(tr);
    }
    scroll.appendChild(table);
    wrap.appendChild(scroll);
  }

  const actions = document.createElement("div");
  actions.className = "form-actions";
  const cancelBtn = document.createElement("button");
  cancelBtn.className = "btn";
  cancelBtn.textContent = "Cancel";
  const okBtn = document.createElement("button");
  okBtn.className = "btn btn-accent";
  okBtn.textContent = "Import";
  actions.append(cancelBtn, okBtn);
  wrap.appendChild(actions);

  const close = openModal("Import Core — confirm mapping", wrap, 640);
  cancelBtn.addEventListener("click", () => close());
  okBtn.addEventListener("click", async () => {
    const choice: CoreImportChoice = {
      headers: {},
      depthUnit: unitSel.value || null,
    };
    for (const role of ROLES) {
      const v = selects.get(role.key)?.value ?? "";
      if (v) choice.headers[role.key] = v;
    }
    if (!choice.headers.depth) {
      setStatus("Pick a depth column first");
      return;
    }
    if (!choice.headers.well && !fallbackWell) {
      setStatus("No well column mapped and no well selected — nothing to route rows to");
      return;
    }
    okBtn.disabled = true;
    close();
    setStatus(`Importing core data from ${paths.length} file(s)…`);

    const results: { path: string; res: CoreTableImportResult | null; skipped?: string }[] = [];
    for (const { path, probe, err } of probes) {
      if (!probe) {
        results.push({ path, res: null, skipped: err ?? "unreadable" });
        continue;
      }
      const m = mappingForFile(choice, probe);
      if ("error" in m) {
        results.push({ path, res: null, skipped: m.error });
        continue;
      }
      try {
        const res = await importCoreTable(path, m.mapping, choice.depthUnit, fallbackWell?.well_id ?? null);
        if (m.missing.length && !res.error) {
          res.outcomes.push({
            well_name: "(this file)",
            rows: 0,
            imported: 0,
            problem: `column(s) not in this file, left empty: ${m.missing.join(", ")}`,
          });
        }
        results.push({ path, res });
      } catch (e) {
        results.push({ path, res: null, skipped: String(e) });
      }
    }
    report(results, fallbackWell);
    onDone();
  });
}
