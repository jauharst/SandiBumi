import {
  imageSupport,
  importWellImages,
  probeImageFiles,
  type ImageImportItem,
  type ImageProbe,
  type WellSummary,
} from "../ipc";
import { appState, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";
import { buildFollowCoreRow } from "./followCore";
import { buildPlateDetails } from "./plateDetails";
import { suggestSetName } from "./importSetDialog";

/** Image import wizard: probe → CONFIRM → commit, the same shape as the core-table wizard
 *  and for the same reason. A plate's depth is the one thing nobody can recover later by
 *  looking at it: a thin section hung off the wrong sand is a wrong conclusion, not a
 *  cosmetic error. So the depth guessed from each filename is shown in an editable table
 *  and nothing is stored until it is confirmed.
 *
 *  What is stored is a normalized JPEG display copy (see `src-tauri/src/images.rs`); the
 *  delivered original stays where the lab put it and its path is recorded with every
 *  picture. The long-edge cap sits on the dialog rather than in the code because it is the
 *  user's trade-off between project size and how far they can zoom in.
 */
export async function openImageImportDialog(
  paths: string[],
  well: WellSummary | null,
  onDone: () => void,
): Promise<void> {
  if (!well) {
    setStatus("Select a well first (Wells & Tops panel) — pictures are imported per well");
    return;
  }
  setStatus(`Reading ${paths.length} image file(s)…`);
  let probes: ImageProbe[];
  try {
    probes = await probeImageFiles(paths);
  } catch (err) {
    setStatus(`Could not read the selected files: ${err}`);
    return;
  }
  const pillow = await imageSupport().catch(() => false);
  const usable = probes.filter((p) => !p.error);
  if (usable.length === 0) {
    setStatus(`No usable image: ${probes[0]?.error ?? "unrecognised format"}`);
    return;
  }

  /** Everything still editable before the commit. */
  const rows = probes.map((p) => ({
    probe: p,
    name: p.name,
    depthTop: p.depth_top,
    depthBase: p.depth_base,
    caption: "",
    // Per plate: the delivery-level field of view fills the blanks, this overrules it. Absent in
    // both means no scale was declared for this plate, which is a real answer.
    fovUm: null as number | null,
    include: !p.error,
  }));

  const wrap = document.createElement("div");

  const summary = document.createElement("div");
  summary.className = "form-hint";
  const failed = probes.length - usable.length;
  summary.textContent =
    `${usable.length} of ${probes.length} file(s) readable` +
    (failed > 0 ? `, ${failed} unreadable (hover the file name for the reason)` : "") +
    ` — importing into ${well.well_name}.`;
  wrap.appendChild(summary);

  const datasetInput = document.createElement("input");
  datasetInput.className = "form-control";
  datasetInput.value = "THIN SECTION";
  datasetInput.setAttribute("list", "img-import-datasets");
  const dsList = document.createElement("datalist");
  dsList.id = "img-import-datasets";
  for (const v of ["THIN SECTION", "CORE PHOTO", "SEM", "FMI", "CT SCAN"]) {
    const o = document.createElement("option");
    o.value = v;
    dsList.appendChild(o);
  }
  wrap.appendChild(dsList);
  wrap.appendChild(
    formRow(
      "Dataset",
      datasetInput,
      "What kind of picture this is. A track series draws one dataset, so keep thin sections and core photographs apart.",
    ),
  );

  const setInput = document.createElement("input");
  setInput.className = "form-control";
  setInput.value = suggestSetName(paths) || "RAW";
  wrap.appendChild(
    formRow(
      "Delivery name",
      setInput,
      "Names this delivery. A name already used on this well is auto-suffixed — an import never overwrites an earlier one.",
    ),
  );

  // A thin section is cut from a plug, so when that plug is re-registered the plate belongs with
  // it — but only the user knows whether these depths are the core report's or the log's.
  const followCore = buildFollowCoreRow("the plate depths", "images");
  wrap.appendChild(followCore.el);

  // Scale and preparation, delivered once for the whole delivery. Stored PER PLATE, because
  // magnification varies within a delivery — the per-plate column below overrules this, and the
  // plate editor can correct any of it afterwards.
  const details = buildPlateDetails({
    scaleHint:
      "How wide the whole picture is, for plates that state it. Leave blank otherwise — nothing " +
      "dimensional runs on a plate with no scale, and a guess would be a microscope setting nobody used.",
  });
  wrap.appendChild(details.el);

  const unitSel = document.createElement("select");
  unitSel.className = "form-control";
  for (const [v, label] of [
    ["m", "metres"],
    ["ft", "feet"],
  ] as const) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    unitSel.appendChild(o);
  }
  unitSel.value = appState.displayDepthUnit.get() === "FT" ? "ft" : "m";
  wrap.appendChild(
    formRow("Depths are in", unitSel, "Converted to the project's depth unit on import."),
  );

  const maxPxInput = document.createElement("input");
  maxPxInput.className = "form-control";
  maxPxInput.type = "number";
  maxPxInput.min = "0";
  maxPxInput.step = "100";
  maxPxInput.value = "2400";
  maxPxInput.disabled = !pillow;
  wrap.appendChild(
    formRow(
      "Long edge (px)",
      maxPxInput,
      "Stored size. 2400 px is well past what a plate resolves at track width on paper; 0 keeps full resolution and a much larger project file.",
    ),
  );

  if (!pillow) {
    const warn = document.createElement("div");
    warn.className = "form-hint";
    warn.textContent =
      "Pillow is not installed, so pictures are stored exactly as delivered (no resizing). TIFF cannot be read at all, and anything that is not a JPEG will print as a labelled frame. Install it with: pip install pillow";
    wrap.appendChild(warn);
  }

  // Confirmation table: one row per file, depth editable, unreadable rows dimmed.
  const scroll = document.createElement("div");
  scroll.className = "core-import-preview";
  const table = document.createElement("table");
  const head = document.createElement("tr");
  for (const h of ["", "File", "Pixels", "Name", "Depth", "Base (optional)", "FOV mm", "Caption"]) {
    const th = document.createElement("th");
    th.textContent = h;
    head.appendChild(th);
  }
  table.appendChild(head);

  const cell = (child: HTMLElement): HTMLTableCellElement => {
    const td = document.createElement("td");
    td.appendChild(child);
    return td;
  };

  for (const r of rows) {
    const tr = document.createElement("tr");
    if (r.probe.error) tr.style.opacity = "0.55";

    const chk = document.createElement("input");
    chk.type = "checkbox";
    chk.className = "form-check";
    chk.checked = r.include;
    chk.disabled = !!r.probe.error;
    chk.addEventListener("change", () => (r.include = chk.checked));
    tr.appendChild(cell(chk));

    const file = document.createElement("td");
    file.textContent = r.probe.file_name;
    if (r.probe.error) file.title = r.probe.error;
    tr.appendChild(file);

    const size = document.createElement("td");
    // 0×0 means only Pillow can read the dimensions (TIFF, plain WebP) — show "?" rather
    // than a zero that reads like a measurement.
    size.textContent = r.probe.error ? "—" : r.probe.width > 0 ? `${r.probe.width}×${r.probe.height}` : "?";
    tr.appendChild(size);

    const nameIn = document.createElement("input");
    nameIn.className = "form-control";
    nameIn.value = r.name;
    nameIn.addEventListener("change", () => (r.name = nameIn.value.trim() || r.probe.name));
    tr.appendChild(cell(nameIn));

    const topIn = document.createElement("input");
    topIn.className = "form-control";
    topIn.type = "number";
    topIn.step = "0.01";
    topIn.value = r.depthTop == null ? "" : String(r.depthTop);
    if (r.depthTop == null && !r.probe.error) {
      // Nothing depth-like in the file name. Flag it: an empty cell scrolling past
      // unnoticed is how a plate ends up silently dropped.
      topIn.placeholder = "required";
      topIn.style.borderColor = "var(--warn)";
    }
    topIn.addEventListener("change", () => {
      const v = Number(topIn.value);
      r.depthTop = topIn.value.trim() === "" || !Number.isFinite(v) ? null : v;
      topIn.style.borderColor = r.depthTop == null ? "var(--warn)" : "";
    });
    tr.appendChild(cell(topIn));

    const baseIn = document.createElement("input");
    baseIn.className = "form-control";
    baseIn.type = "number";
    baseIn.step = "0.01";
    baseIn.value = r.depthBase == null ? "" : String(r.depthBase);
    baseIn.title =
      "Leave empty for a sample with no thickness (a thin section). A base depth makes the picture a photographed interval, which a track can then draw to depth scale.";
    baseIn.addEventListener("change", () => {
      const v = Number(baseIn.value);
      r.depthBase = baseIn.value.trim() === "" || !Number.isFinite(v) ? null : v;
    });
    tr.appendChild(cell(baseIn));

    // Blank = use the delivery value above. A plate photographed at another magnification is
    // corrected here rather than by splitting the delivery in two.
    const fovIn = document.createElement("input");
    fovIn.className = "form-control";
    fovIn.type = "number";
    fovIn.step = "0.01";
    fovIn.min = "0";
    fovIn.placeholder = "delivery";
    fovIn.title =
      "Width of this picture in millimetres. Leave blank to use the delivery value, or blank in both if this plate does not state a scale.";
    fovIn.addEventListener("change", () => {
      const v = Number(fovIn.value);
      r.fovUm = fovIn.value.trim() === "" || !Number.isFinite(v) || v <= 0 ? null : v * 1000;
    });
    tr.appendChild(cell(fovIn));

    const capIn = document.createElement("input");
    capIn.className = "form-control";
    capIn.addEventListener("change", () => (r.caption = capIn.value));
    tr.appendChild(cell(capIn));

    table.appendChild(tr);
  }
  scroll.appendChild(table);
  wrap.appendChild(scroll);

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

  const close = openModal("Import Images — confirm depths", wrap, 900);
  cancelBtn.addEventListener("click", () => close());
  okBtn.addEventListener("click", async () => {
    const items: ImageImportItem[] = [];
    const noDepth: string[] = [];
    for (const r of rows) {
      if (!r.include || r.probe.error) continue;
      if (r.depthTop == null) {
        noDepth.push(r.probe.file_name);
        continue;
      }
      items.push({
        path: r.probe.path,
        name: r.name,
        depth_top: r.depthTop,
        depth_base: r.depthBase,
        caption: r.caption.trim() || null,
        fov_um: r.fovUm,
      });
    }
    if (items.length === 0) {
      setStatus("Nothing to import — every selected picture is missing a depth.");
      return;
    }
    okBtn.disabled = true;
    close();
    setStatus(`Importing ${items.length} picture(s)…`);
    try {
      const res = await importWellImages({
        well_id: well.well_id,
        dataset: datasetInput.value,
        set_name: setInput.value,
        depth_unit: unitSel.value,
        max_px: Number(maxPxInput.value) || 0,
        quality: 85,
        follow_core: followCore.checked(),
        fov_um: details.get().fov_um,
        prepared: details.get().prepared || null,
        stain: details.get().stain || null,
        items,
      });
      const mb = (res.bytes / 1048576).toFixed(1);
      const parts = [`Imported ${res.imported} picture(s) as ${res.dataset} / ${res.set_name} (${mb} MB)`];
      if (noDepth.length) parts.push(`${noDepth.length} skipped with no depth`);
      if (res.skipped.length) parts.push(res.skipped.join("; "));
      if (res.note) parts.push(res.note);
      const line = parts.join(" — ");
      setStatus(line);
      recordProcess("Import", line, well.well_name);
      onDone();
    } catch (err) {
      setStatus(`Image import failed: ${err}`);
    }
  });
}
