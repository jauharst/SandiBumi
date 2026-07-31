import {
  listImageDatasets,
  listWellImages,
  shiftWellImages,
  updateWellImage,
  type ImageInfo,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";

/** Plate depth editing (Data ▸ Tools ▾ ▸ Plate Depths…).
 *
 *  `update_well_image` has existed and been tested since the image track shipped, and nothing
 *  called it: a thin section delivered at the wrong depth could only be corrected by deleting the
 *  delivery and importing it again. This is that missing caller.
 *
 *  Two rules are load-bearing and both come from what a plate IS:
 *
 *  **An empty base means a POINT sample, and stays one.** A thin section is cut from one plug and
 *  has no thickness, which is why `depth_base IS NULL` is a petrophysical statement in the store
 *  rather than a missing field. Typing a base here is a deliberate claim that the picture spans an
 *  interval; leaving it blank must never be "helpfully" filled in from the plate below.
 *
 *  **A whole delivery moves in one statement.** Plates read off one mis-registered core tally are
 *  wrong by one number, so the bulk shift is the normal repair and the per-plate edit is the
 *  exception. It is also the only practical form: a core-photograph delivery is routinely hundreds
 *  of plates.
 */
export async function openPlateDepthDialog(): Promise<void> {
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  openModal(well ? `Plate depths — ${well.well_name}` : "Plate depths", wrap, 780);

  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent = "Select a well in the Wells pane first.";
    wrap.appendChild(none);
    return;
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Re-registers pictures already in the project: thin sections, core photographs, SEM plates. " +
    "Shows the live delivery of each dataset — switch deliveries in Data Sets…";
  wrap.appendChild(intro);

  const dsSel = document.createElement("select");
  dsSel.className = "form-control";
  const all = document.createElement("option");
  all.value = "";
  all.textContent = "All datasets";
  dsSel.appendChild(all);
  const datasets = await listImageDatasets(well.well_id).catch(() => [] as [string, number][]);
  for (const [name, n] of datasets) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = `${name} — ${n} plate(s)`;
    dsSel.appendChild(o);
  }
  wrap.appendChild(formRow("Dataset", dsSel));

  if (!datasets.length) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.style.color = "var(--warn)";
    none.textContent = "This well has no pictures yet. Import some with Data ▸ Import ▸ Images…";
    wrap.appendChild(none);
    return;
  }

  // ---- bulk shift ---------------------------------------------------------
  const shiftIn = document.createElement("input");
  shiftIn.className = "form-control";
  shiftIn.type = "number";
  shiftIn.step = "0.1";
  shiftIn.placeholder = "e.g. -2.5";
  const shiftBtn = document.createElement("button");
  shiftBtn.className = "btn";
  shiftBtn.textContent = "Shift delivery";
  const shiftRow = document.createElement("div");
  shiftRow.style.display = "flex";
  shiftRow.style.gap = "8px";
  shiftRow.appendChild(shiftIn);
  shiftRow.appendChild(shiftBtn);
  wrap.appendChild(
    formRow("Shift every plate by", shiftRow, "+ = deeper. Applies to the dataset selected above.")
  );

  const tableWrap = document.createElement("div");
  tableWrap.style.maxHeight = "340px";
  tableWrap.style.overflow = "auto";
  wrap.appendChild(tableWrap);

  let rows: ImageInfo[] = [];

  const refresh = async (): Promise<void> => {
    const ds = dsSel.value || null;
    rows = await listWellImages(well.well_id, ds).catch(() => [] as ImageInfo[]);
    rows.sort((a, b) => a.depth_top - b.depth_top);
    render();
  };

  function render(): void {
    tableWrap.innerHTML = "";
    const table = document.createElement("table");
    table.className = "data-table";
    const head = document.createElement("tr");
    for (const h of ["Dataset", "Name", "Top", "Base", "Kind", "Caption", ""]) {
      const th = document.createElement("th");
      th.textContent = h;
      head.appendChild(th);
    }
    table.appendChild(head);

    for (const img of rows) {
      const tr = document.createElement("tr");
      const cell = (el: HTMLElement): void => {
        const td = document.createElement("td");
        td.appendChild(el);
        tr.appendChild(td);
      };
      const text = (s: string): HTMLElement => {
        const d = document.createElement("span");
        d.textContent = s;
        return d;
      };
      cell(text(img.dataset));

      const nameIn = document.createElement("input");
      nameIn.className = "form-control";
      nameIn.value = img.name;
      cell(nameIn);

      const topIn = document.createElement("input");
      topIn.className = "form-control";
      topIn.type = "number";
      topIn.step = "0.01";
      topIn.value = String(img.depth_top);
      cell(topIn);

      const baseIn = document.createElement("input");
      baseIn.className = "form-control";
      baseIn.type = "number";
      baseIn.step = "0.01";
      baseIn.value = img.depth_base == null ? "" : String(img.depth_base);
      baseIn.placeholder = "point";
      cell(baseIn);

      const kind = text(img.depth_base == null ? "point" : "interval");
      kind.title =
        img.depth_base == null
          ? "Cut from one plug — anchored at its depth, with no thickness."
          : "Spans a real interval.";
      cell(kind);

      const capIn = document.createElement("input");
      capIn.className = "form-control";
      capIn.value = img.caption ?? "";
      cell(capIn);

      const save = document.createElement("button");
      save.className = "btn";
      save.textContent = "Save";
      save.addEventListener("click", () => {
        const top = Number(topIn.value);
        if (!Number.isFinite(top)) {
          setStatus("Top depth must be a number");
          return;
        }
        // Blank stays blank: an empty base is "no thickness", not "unknown thickness".
        const base = baseIn.value.trim() === "" ? null : Number(baseIn.value);
        if (base != null && !Number.isFinite(base)) {
          setStatus("Base depth must be a number, or blank for a point sample");
          return;
        }
        if (base != null && base < top) {
          // Not swapped silently — a reversed pair is a typo or a wrong column, and guessing
          // which would hide it.
          setStatus(`${nameIn.value}: base is above top — check the depths`);
          return;
        }
        const before = { top: img.depth_top, base: img.depth_base, name: img.name, caption: img.caption };
        const after = { top, base, name: nameIn.value, caption: capIn.value || null };
        void (async () => {
          await updateWellImage(img.image_id, after.top, after.base, after.name, after.caption);
          setStatus(`Re-registered ${after.name} at ${after.top}`);
          recordProcess("Edit", `Plate ${after.name} → ${after.top}`, well!.well_name);
          pushUndo({
            label: `plate depth ${after.name} (${well!.well_name})`,
            undo: async () => {
              await updateWellImage(img.image_id, before.top, before.base, before.name, before.caption);
              await refresh();
              bumpDataVersion();
            },
            redo: async () => {
              await updateWellImage(img.image_id, after.top, after.base, after.name, after.caption);
              await refresh();
              bumpDataVersion();
            },
          });
          await refresh();
          bumpDataVersion();
        })();
      });
      cell(save);
      table.appendChild(tr);
    }
    tableWrap.appendChild(table);

    const count = document.createElement("div");
    count.className = "eq-note";
    count.textContent = `${rows.length} plate(s) in the live delivery.`;
    tableWrap.appendChild(count);
  }

  shiftBtn.addEventListener("click", () => {
    const delta = Number(shiftIn.value);
    if (!Number.isFinite(delta) || delta === 0) {
      setStatus("Enter a non-zero shift");
      return;
    }
    const ds = dsSel.value || null;
    void (async () => {
      const n = await shiftWellImages(well!.well_id, ds, delta);
      const sign = delta > 0 ? "+" : "";
      setStatus(`Moved ${n} plate(s) by ${sign}${delta}`);
      recordProcess("Edit", `Plate shift ${sign}${delta} (${ds ?? "all datasets"}, ${n} plates)`, well!.well_name);
      pushUndo({
        label: `plate shift ${sign}${delta} (${well!.well_name})`,
        undo: async () => {
          await shiftWellImages(well!.well_id, ds, -delta);
          await refresh();
          bumpDataVersion();
        },
        redo: async () => {
          await shiftWellImages(well!.well_id, ds, delta);
          await refresh();
          bumpDataVersion();
        },
      });
      await refresh();
      bumpDataVersion();
    })();
  });

  dsSel.addEventListener("change", () => void refresh());
  await refresh();
}
