import {
  listImageDatasets,
  listWellImages,
  setImageDeliveryDetails,
  setImageDetails,
  shiftWellImages,
  updateWellImage,
  type ImageInfo,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { buildPlateDetails } from "./plateDetails";
import { openScaleBarDialog, scaleBarAppliedToAll } from "./scaleBarDialog";

/** Plate depth, scale and preparation editing (Data ▸ Tools ▾ ▸ Plate Details…).
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
  openModal(well ? `Plates — ${well.well_name}` : "Plates", wrap, 900);

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

  // ---- scale and preparation, delivery-wide ------------------------------
  // A delivery is usually one microscope and one impregnation run, so correcting it plate by
  // plate would be hundreds of round trips to apply one decision — the same argument the bulk
  // shift is built on. The per-plate columns below stay for the delivery that is genuinely mixed,
  // which is the case that made these fields per-plate in the first place.
  const details = buildPlateDetails({
    scaleHint:
      "Applies to every plate of the dataset selected above. Leave blank to clear it — a scale " +
      "nobody can remove is worse than one never entered.",
  });
  wrap.appendChild(details.el);
  const applyDetails = document.createElement("button");
  applyDetails.className = "btn";
  applyDetails.textContent = "Apply to whole delivery";
  wrap.appendChild(applyDetails);

  applyDetails.addEventListener("click", () => {
    const ds = dsSel.value;
    if (!ds) {
      // A delivery-wide write needs a delivery. "All datasets" would silently give a core
      // photograph the thin sections' magnification.
      setStatus("Choose one dataset first — scale and preparation belong to a delivery");
      return;
    }
    const d = details.get();
    const before = rows.filter((r) => r.dataset === ds).map((r) => ({ ...r }));
    void (async () => {
      const n = await setImageDeliveryDetails(well!.well_id, ds, d.fov_um, d.prepared || null, d.stain || null);
      setStatus(`Set scale and preparation on ${n} plate(s) of ${ds}`);
      recordProcess("Edit", `Plate details on ${ds} (${n} plates)`, well!.well_name);
      pushUndo({
        label: `plate details ${ds} (${well!.well_name})`,
        // Restored plate by plate: they need not have agreed before, and writing one value back
        // across the delivery would invent a uniformity that was not there.
        undo: async () => {
          for (const r of before) await setImageDetails(r.image_id, r.fov_um, r.prepared || null, r.stain || null);
          await refresh();
          bumpDataVersion();
        },
        redo: async () => {
          await setImageDeliveryDetails(well!.well_id, ds, d.fov_um, d.prepared || null, d.stain || null);
          await refresh();
          bumpDataVersion();
        },
      });
      await refresh();
      bumpDataVersion();
    })();
  });

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
    for (const h of ["Dataset", "Name", "Top", "Base", "Kind", "FOV mm", "Prep", "Caption", ""]) {
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

      // Blank means no scale was declared for this plate, and that is a real answer — nothing
      // dimensional will run on it, which is the point.
      const fovIn = document.createElement("input");
      fovIn.className = "form-control";
      fovIn.type = "number";
      fovIn.step = "0.01";
      fovIn.min = "0";
      fovIn.placeholder = "none";
      fovIn.value = img.fov_um == null ? "" : String(img.fov_um / 1000);
      fovIn.title =
        img.fov_um == null
          ? "No scale declared. Grain and pore size cannot be measured on this plate."
          : `${(img.fov_um / img.width).toFixed(3)} µm/px on the stored copy (${img.width} px wide)`;

      // The route for a plate that states its scale as a BAR rather than in the caption. It only
      // FILLS the box — the row's own Save is still what writes it, so a calibration is reviewed
      // like any other typed value.
      const calBtn = document.createElement("button");
      calBtn.className = "btn";
      calBtn.textContent = "⇹";
      calBtn.title = "Measure the plate's own scale bar";
      calBtn.addEventListener("click", () => {
        void (async () => {
          const fov = await openScaleBarDialog(img);
          if (fov == null) return;
          fovIn.value = (fov / 1000).toFixed(4);
          setStatus(`${img.name}: field of view ${(fov / 1000).toFixed(3)} mm — press Save to keep it`);
          if (!scaleBarAppliedToAll()) return;
          // Applied across the delivery row by row rather than in one statement, because each
          // plate keeps its OWN preparation and stain: a scale must not quietly overwrite what
          // the section was made of. Slower, and the only version that is right.
          const mine = rows.filter((r) => r.dataset === img.dataset);
          const before = mine.map((r) => ({ ...r }));
          for (const r of mine) await setImageDetails(r.image_id, fov, r.prepared || null, r.stain || null);
          setStatus(`Field of view ${(fov / 1000).toFixed(3)} mm on ${mine.length} plate(s) of ${img.dataset}`);
          recordProcess("Edit", `Scale bar on ${img.dataset}: ${mine.length} plate(s)`, well!.well_name);
          pushUndo({
            label: `plate scale ${img.dataset} (${well!.well_name})`,
            undo: async () => {
              for (const r of before) await setImageDetails(r.image_id, r.fov_um, r.prepared || null, r.stain || null);
              await refresh();
              bumpDataVersion();
            },
            redo: async () => {
              for (const r of before) await setImageDetails(r.image_id, fov, r.prepared || null, r.stain || null);
              await refresh();
              bumpDataVersion();
            },
          });
          await refresh();
          bumpDataVersion();
        })();
      });
      const fovCell = document.createElement("div");
      fovCell.style.display = "flex";
      fovCell.style.gap = "4px";
      fovCell.appendChild(fovIn);
      fovCell.appendChild(calBtn);
      cell(fovCell);

      const prepIn = document.createElement("select");
      prepIn.className = "form-control";
      for (const [v, label] of [["", "?"], ["blue_epoxy", "Blue epoxy"], ["plain", "Plain"]] as const) {
        const o = document.createElement("option");
        o.value = v;
        o.textContent = label;
        prepIn.appendChild(o);
      }
      prepIn.value = img.prepared || "";
      prepIn.title =
        "Unknown is refused by the pore measurement rather than assumed — a blue-epoxy rule run on " +
        "an unimpregnated section returns a porosity built from blue-ish grains.";
      cell(prepIn);

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
        const fovMm = Number(fovIn.value);
        const before = {
          top: img.depth_top, base: img.depth_base, name: img.name, caption: img.caption,
          fov: img.fov_um, prep: img.prepared || null,
        };
        const after = {
          top, base, name: nameIn.value, caption: capIn.value || null,
          // Blank clears the scale. A wrongly typed field of view has to be removable — one that
          // cannot be cleared is worse than one never entered, because everything downstream
          // believes it.
          fov: fovIn.value.trim() === "" || !Number.isFinite(fovMm) || fovMm <= 0 ? null : fovMm * 1000,
          prep: prepIn.value || null,
        };
        void (async () => {
          await updateWellImage(img.image_id, after.top, after.base, after.name, after.caption);
          await setImageDetails(img.image_id, after.fov, after.prep, img.stain || null);
          setStatus(`Re-registered ${after.name} at ${after.top}`);
          recordProcess("Edit", `Plate ${after.name} → ${after.top}`, well!.well_name);
          pushUndo({
            label: `plate depth ${after.name} (${well!.well_name})`,
            undo: async () => {
              await updateWellImage(img.image_id, before.top, before.base, before.name, before.caption);
              await setImageDetails(img.image_id, before.fov, before.prep, img.stain || null);
              await refresh();
              bumpDataVersion();
            },
            redo: async () => {
              await updateWellImage(img.image_id, after.top, after.base, after.name, after.caption);
              await setImageDetails(img.image_id, after.fov, after.prep, img.stain || null);
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
