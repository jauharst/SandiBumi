import {
  deleteAuxSet,
  deleteCoreSet,
  deleteScalSet,
  deleteSurvey,
  deleteImageSet,
  listAuxSets,
  listCoreSets,
  listImageSets,
  listScalSets,
  listSurveys,
  setActiveAuxSet,
  setActiveCoreSet,
  setActiveImageSet,
  setActiveScalSet,
  setActiveSurvey,
  type AuxSetInfo,
  type CoreSetInfo,
  type ImageSetInfo,
  type ScalSetInfo,
  type SurveyInfo,
} from "../ipc";
import { renameDeliverySet } from "../ipc";
import { setStatus } from "../state";
import { recordProcess } from "../processLog";
import { openModal } from "./modal";
import { ensureSessionOperator } from "./runCustody";

/** Data-set manager: core, surveys and every point dataset (T-IMP-08 / T-IMP-12).
 *
 *  Curves learned the set model first: one delivery = one named set, and an import never
 *  overwrites an earlier one. Core plugs, deviation surveys and ALL point data — XRD, CEC,
 *  oil show, petrography, perforations, core extras — now work the same way, but with a
 *  different resolution rule, and the difference matters. Two curve sets can BOTH be read (a
 *  set supplies mnemonics RAW lacks); two deliveries of the same plugs or the same samples
 *  would just double every count. So exactly ONE core set, ONE survey and one set per point
 *  dataset are ACTIVE, and this dialog is where that choice is made and the rest are kept,
 *  not lost.
 *
 *  Switching a survey re-materializes TVD/TVDSS immediately — leaving the old geometry in
 *  the stored curves would silently feed every height calculation the survey you just
 *  switched away from.
 */

function fmtDate(s: string | null): string {
  if (!s) return "—";
  // DuckDB stamps "2026-07-30 14:05:12.123"; the date and minute are what a user reads.
  return s.slice(0, 16).replace("T", " ");
}

function fileOf(p: string | null): string {
  if (!p) return "—";
  return p.replace(/\\/g, "/").split("/").pop() ?? p;
}

/** One section (core sets or surveys), rebuilt in place after every mutation. */
function buildSection<T>(opts: {
  title: string;
  empty: string;
  countLabel: (row: T) => string;
  nameOf: (row: T) => string;
  isActive: (row: T) => boolean;
  sourceOf: (row: T) => string | null;
  dateOf: (row: T) => string | null;
  extra?: (row: T) => string;
  /** Point data only: the dataset a row belongs to. Rows are grouped under it, and it is
   *  passed back on activate/remove — activation is per dataset, not per well. */
  groupOf?: (row: T) => string;
  load: () => Promise<T[]>;
  activate: (name: string, group?: string) => Promise<void>;
  remove: (name: string, group?: string) => Promise<void>;
  /** Present = the rows offer Rename (every section does; the callback carries its kind). */
  rename?: (oldName: string, newName: string, group?: string) => Promise<void>;
}): { root: HTMLElement; refresh: () => Promise<void> } {
  const root = document.createElement("div");
  const h = document.createElement("p");
  h.className = "props-section";
  h.textContent = opts.title;
  root.appendChild(h);
  const body = document.createElement("div");
  root.appendChild(body);

  const refresh = async (): Promise<void> => {
    body.replaceChildren();
    let rows: T[];
    try {
      rows = await opts.load();
    } catch (err) {
      const p = document.createElement("p");
      p.className = "form-hint";
      p.textContent = String(err);
      body.appendChild(p);
      return;
    }
    if (rows.length === 0) {
      const p = document.createElement("p");
      p.className = "form-hint";
      p.textContent = opts.empty;
      body.appendChild(p);
      return;
    }
    const table = document.createElement("table");
    table.className = "set-table";
    const head = document.createElement("tr");
    for (const label of ["", "Name", "Rows", "From file", "Imported", ""]) {
      const th = document.createElement("th");
      th.textContent = label;
      head.appendChild(th);
    }
    table.appendChild(head);

    let lastGroup: string | null = null;
    for (const row of rows) {
      const name = opts.nameOf(row);
      const group = opts.groupOf?.(row);
      const active = opts.isActive(row);
      // A sub-header per dataset, since the backend already orders by dataset.
      if (group !== undefined && group !== lastGroup) {
        lastGroup = group;
        const gr = document.createElement("tr");
        const gc = document.createElement("td");
        gc.colSpan = 6;
        gc.className = "set-group";
        gc.textContent = group;
        gr.appendChild(gc);
        table.appendChild(gr);
      }
      const tr = document.createElement("tr");
      if (active) tr.className = "set-active";

      const mark = document.createElement("td");
      mark.textContent = active ? "●" : "";
      mark.title = active ? "Active — this is what every panel reads" : "";
      tr.appendChild(mark);

      const nameCell = document.createElement("td");
      nameCell.textContent = name + (opts.extra ? opts.extra(row) : "");
      tr.appendChild(nameCell);

      const rowsCell = document.createElement("td");
      rowsCell.textContent = opts.countLabel(row);
      tr.appendChild(rowsCell);

      const srcCell = document.createElement("td");
      srcCell.textContent = fileOf(opts.sourceOf(row));
      srcCell.title = opts.sourceOf(row) ?? "";
      tr.appendChild(srcCell);

      const dateCell = document.createElement("td");
      dateCell.textContent = fmtDate(opts.dateOf(row));
      tr.appendChild(dateCell);

      const actions = document.createElement("td");
      if (!active) {
        const use = document.createElement("button");
        use.className = "btn";
        use.textContent = "Use";
        use.title = "Make this the active one";
        use.addEventListener("click", () => {
          void opts
            .activate(name, group)
            .then(() => refresh())
            .catch((err) => setStatus(String(err)));
        });
        actions.appendChild(use);
      }
      if (opts.rename) {
        const ren = document.createElement("button");
        ren.className = "btn";
        ren.textContent = "Rename";
        ren.title = "Renames the delivery everywhere its name is read - audited";
        ren.addEventListener("click", () => {
          const entered = window.prompt(`Rename ${name} to:`, name);
          const newName = entered?.trim();
          if (!newName || newName === name) return;
          void opts
            .rename!(name, newName, group)
            .then(() => refresh())
            .catch((err) => setStatus(String(err)));
        });
        actions.appendChild(ren);
      }
      const del = document.createElement("button");
      del.className = "btn";
      del.textContent = "Delete";
      // Deleting data is irreversible here (no undo entry), so it asks first and names
      // what goes — a mis-click must not silently drop a lab delivery.
      del.addEventListener("click", () => {
        const what = group ? `${group} / ${name}` : name;
        if (!window.confirm(`Delete ${what} (${opts.countLabel(row)})? This cannot be undone.`)) return;
        void opts
          .remove(name, group)
          .then(() => refresh())
          .catch((err) => setStatus(String(err)));
      });
      actions.appendChild(del);
      tr.appendChild(actions);
      table.appendChild(tr);
    }
    body.appendChild(table);
  };

  return { root, refresh };
}

/**
 * Opens the manager for one well. `onChanged` fires after any activation or deletion so the
 * workspace can repaint (core overlays and TVD-aware views follow the active set).
 */
export function openDataSetsDialog(
  well: { well_id: string; well_name: string },
  onChanged: () => void,
): void {
  const wrap = document.createElement("div");

  // One rename routine for every section: custody first (a rename is audited, so the
  // operator is demanded before anything moves), then the backend moves every row that
  // carries the name — riders included — or refuses by name.
  const renameSet =
    (kind: string, label: string) =>
    async (oldName: string, newName: string, group?: string): Promise<void> => {
      const operator = await ensureSessionOperator("Rename delivery set");
      if (!operator) return;
      const receipt = await renameDeliverySet(
        kind,
        well.well_id,
        group ?? null,
        oldName,
        newName,
        operator.identity,
        operator.kind,
        "Data Sets",
      );
      const rider =
        receipt.rider_rows_moved > 0 ? ` (+${receipt.rider_rows_moved} rider row(s) moved with it)` : "";
      setStatus(`Renamed ${label} ${oldName} → ${newName} (${receipt.rows_moved} row(s))${rider} — audited.`);
      recordProcess("Edit", `Renamed ${label} set ${oldName} → ${newName}`, well.well_name);
      onChanged();
    };

  const doc = document.createElement("p");
  doc.className = "modal-doc";
  doc.textContent =
    "Every delivery ever imported for this well — core, SCAL, deviation surveys and point data (XRD, CEC, oil show, …). " +
    "One of each is ACTIVE (●) — that is what log overlays, φ-k plots, Pc/J-fits, calibration, TVD/TVDSS and the data " +
    "panels read. The rest are kept, not lost.";
  wrap.appendChild(doc);

  const core = buildSection<CoreSetInfo>({
    title: "Core sets",
    empty: "No core imported for this well yet.",
    nameOf: (r) => r.set_name,
    isActive: (r) => r.active,
    countLabel: (r) => `${r.rows} plug(s)`,
    sourceOf: (r) => r.source,
    dateOf: (r) => r.imported_at,
    load: () => listCoreSets(well.well_id),
    rename: renameSet("core", "core"),
    activate: async (name) => {
      await setActiveCoreSet(well.well_id, name);
      setStatus(`Core set ${name} is now active for ${well.well_name}.`);
      recordProcess("Edit", `Active core set → ${name}`, well.well_name);
      onChanged();
    },
    remove: async (name) => {
      const n = await deleteCoreSet(well.well_id, name);
      setStatus(`Deleted core set ${name} (${n} plug(s)) from ${well.well_name}.`);
      recordProcess("Edit", `Deleted core set ${name} (${n} plugs)`, well.well_name);
      onChanged();
    },
  });
  wrap.appendChild(core.root);

  const scal = buildSection<ScalSetInfo>({
    title: "SCAL (capillary pressure)",
    empty: "No SCAL Pc data imported for this well yet.",
    nameOf: (r) => r.set_name,
    isActive: (r) => r.active,
    countLabel: (r) => `${r.rows} point(s)`,
    sourceOf: (r) => r.source,
    dateOf: (r) => r.imported_at,
    load: () => listScalSets(well.well_id),
    rename: renameSet("scal", "SCAL"),
    activate: async (name) => {
      await setActiveScalSet(well.well_id, name);
      setStatus(`SCAL set ${name} is now active for ${well.well_name}.`);
      recordProcess("Edit", `Active SCAL set → ${name}`, well.well_name);
      onChanged();
    },
    remove: async (name) => {
      const n = await deleteScalSet(well.well_id, name);
      setStatus(`Deleted SCAL set ${name} (${n} point(s)) from ${well.well_name}.`);
      recordProcess("Edit", `Deleted SCAL set ${name} (${n} points)`, well.well_name);
      onChanged();
    },
  });
  wrap.appendChild(scal.root);

  const surveys = buildSection<SurveyInfo>({
    title: "Deviation surveys",
    empty: "No deviation survey imported — the well is treated as vertical (TVD = MD).",
    nameOf: (r) => r.survey_name,
    isActive: (r) => r.active,
    countLabel: (r) => `${r.stations} station(s)`,
    sourceOf: (r) => r.source,
    dateOf: (r) => r.imported_at,
    extra: (r) => (r.datum === null ? "" : `  (datum ${r.datum})`),
    load: () => listSurveys(well.well_id),
    rename: renameSet("survey", "survey"),
    activate: async (name) => {
      const samples = await setActiveSurvey(well.well_id, name);
      setStatus(`Survey ${name} is now active; TVD/TVDSS rebuilt (${samples} sample(s)).`);
      recordProcess("Edit", `Active survey → ${name}; TVD/TVDSS rebuilt (${samples} samples)`, well.well_name);
      onChanged();
    },
    remove: async (name) => {
      const n = await deleteSurvey(well.well_id, name);
      setStatus(`Deleted survey ${name} (${n} station(s)) from ${well.well_name}.`);
      recordProcess("Edit", `Deleted survey ${name} (${n} stations)`, well.well_name);
      onChanged();
    },
  });
  wrap.appendChild(surveys.root);

  // Point data: one section, grouped by dataset. The dataset is part of the row label
  // because activation is per dataset — switching XRD leaves CEC and oil show alone.
  const aux = buildSection<AuxSetInfo>({
    title: "Point data (XRD, CEC, oil show, core extras …)",
    empty: "No point data imported for this well yet.",
    nameOf: (r) => r.set_name,
    isActive: (r) => r.active,
    countLabel: (r) => `${r.rows} value(s)`,
    sourceOf: (r) => r.source,
    dateOf: (r) => r.imported_at,
    groupOf: (r) => r.dataset,
    load: () => listAuxSets(well.well_id),
    rename: renameSet("aux", "point-data"),
    activate: async (name, group) => {
      await setActiveAuxSet(well.well_id, group ?? "", name);
      setStatus(`${group}: set ${name} is now active for ${well.well_name}.`);
      recordProcess("Edit", `Active ${group} set → ${name}`, well.well_name);
      onChanged();
    },
    remove: async (name, group) => {
      const n = await deleteAuxSet(well.well_id, group ?? "", name);
      setStatus(`Deleted ${group} set ${name} (${n} value(s)) from ${well.well_name}.`);
      recordProcess("Edit", `Deleted ${group} set ${name} (${n} values)`, well.well_name);
      onChanged();
    },
  });
  wrap.appendChild(aux.root);

  // Pictures: grouped by dataset like point data, and the only section that shows SIZE —
  // a core photo run is the one delivery whose cost a user needs before deciding to keep it.
  const images = buildSection<ImageSetInfo>({
    title: "Images (thin sections, core photos …)",
    empty: "No pictures imported for this well yet.",
    nameOf: (r) => r.set_name,
    isActive: (r) => r.active,
    countLabel: (r) => `${r.images} picture(s), ${(r.bytes / 1048576).toFixed(1)} MB`,
    sourceOf: (r) => r.source,
    dateOf: (r) => r.imported_at,
    groupOf: (r) => r.dataset,
    load: () => listImageSets(well.well_id),
    rename: renameSet("image", "image"),
    activate: async (name, group) => {
      await setActiveImageSet(well.well_id, group ?? "", name);
      setStatus(`${group}: image set ${name} is now active for ${well.well_name}.`);
      recordProcess("Edit", `Active ${group} image set → ${name}`, well.well_name);
      onChanged();
    },
    remove: async (name, group) => {
      const n = await deleteImageSet(well.well_id, group ?? "", name);
      setStatus(`Deleted ${group} image set ${name} (${n} picture(s)) from ${well.well_name}.`);
      recordProcess("Edit", `Deleted ${group} image set ${name} (${n} pictures)`, well.well_name);
      onChanged();
    },
  });
  wrap.appendChild(images.root);

  const note = document.createElement("p");
  note.className = "form-hint";
  note.textContent =
    "Switching a survey recomputes TVD/TVDSS from it straight away, so height calculations never keep the old geometry. " +
    "Deleting the active one hands over to the next newest.";
  wrap.appendChild(note);

  openModal(`Data Sets — ${well.well_name}`, wrap, 620);
  void core.refresh();
  void scal.refresh();
  void surveys.refresh();
  void aux.refresh();
  void images.refresh();
}
