import {
  listImageDatasets,
  listWellImages,
  poreSupport,
  runPoreArea,
  type ImageInfo,
  type PoreColorBand,
  type PoreResult,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { formRow, openModal } from "./modal";

/**
 * Pore area from blue-dyed epoxy (Petrophysics ▸ Petrography ▸ Pore Area…).
 *
 * The first real measurement taken off a plate, and deliberately the dimensionless one: an area
 * fraction needs no micrometres per pixel, so it runs on every plate rather than only the
 * calibrated ones.
 *
 * **The preview is not a convenience, it is the method.** A colour threshold cannot be judged from
 * a number — only by looking at what it selected. So the dialog measures ONE plate as the sliders
 * move and shows the mask drawn over it, and that overlay comes from the backend rather than being
 * redrawn here: putting the segmentation in two languages is how the two drift apart, and what the
 * user tunes against has to be literally what gets measured.
 *
 * **Nothing is written until the user asks.** Measuring and saving are separate buttons, because
 * tuning a threshold means running it many times and a project full of half-judged answers is
 * worse than none.
 */
export async function openPoreAreaDialog(): Promise<void> {
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  openModal(well ? `Pore area — ${well.well_name}` : "Pore area", wrap, 860);

  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent = "Select a well in the Wells pane first — plates are measured one well at a time.";
    wrap.appendChild(none);
    return;
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Measures the blue-epoxy area on each thin section, which estimates pore volume by the Delesse " +
    "relation. Only plates you have declared as impregnated are measured — set that in Plate Details…";
  wrap.appendChild(intro);

  if (!(await poreSupport().catch(() => false))) {
    const warn = document.createElement("div");
    warn.className = "eq-note";
    warn.style.color = "var(--warn)";
    warn.textContent =
      "This needs numpy and Pillow in the Python the app uses. Install them (pip install numpy pillow) " +
      "and reopen — nothing else in the app is affected.";
    wrap.appendChild(warn);
    return;
  }

  const dsSel = document.createElement("select");
  dsSel.className = "form-control";
  const datasets = await listImageDatasets(well.well_id).catch(() => [] as [string, number][]);
  for (const [name, n] of datasets) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = `${name} — ${n} plate(s)`;
    dsSel.appendChild(o);
  }
  wrap.appendChild(formRow("Picture dataset", dsSel));

  if (!datasets.length) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.style.color = "var(--warn)";
    none.textContent = "This well has no pictures. Import some with Data ▸ Import ▸ Images…";
    wrap.appendChild(none);
    return;
  }

  // ---- the plate the band is tuned on -------------------------------------
  const plateSel = document.createElement("select");
  plateSel.className = "form-control";
  let plates: ImageInfo[] = [];
  const loadPlates = async (): Promise<void> => {
    plates = await listWellImages(well.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    plateSel.innerHTML = "";
    for (const p of plates) {
      const o = document.createElement("option");
      o.value = p.image_id;
      // The preparation is shown in the picker because it decides whether the plate can be
      // measured at all — finding that out only after pressing Measure wastes the run.
      const prep = p.prepared === "blue_epoxy" ? "" : p.prepared === "plain" ? " — not impregnated" : " — preparation not stated";
      o.textContent = `${p.name} @ ${p.depth_top}${prep}`;
      o.disabled = p.prepared !== "blue_epoxy";
      plateSel.appendChild(o);
    }
    const first = plates.find((p) => p.prepared === "blue_epoxy");
    if (first) plateSel.value = first.image_id;
  };
  await loadPlates();
  wrap.appendChild(formRow("Tune on plate", plateSel, "The band is judged by eye on one plate, then applied to the delivery."));

  // ---- the colour band ----------------------------------------------------
  const mk = (label: string, value: number, step: number, hint: string): HTMLInputElement => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = String(step);
    i.value = String(value);
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };
  // Round starting numbers, and said to be a starting point: a two-decimal threshold here would
  // look like somebody's regression result, and there is no regression behind it.
  const hueLo = mk("Hue from (°)", 180, 5, "Blue-dyed epoxy sits in the blue-to-violet arc. A starting band, not a calibration — judge it on the picture.");
  const hueHi = mk("Hue to (°)", 260, 5, "Narrow this if grain edges or stain are being caught.");
  const satMin = mk("Saturation at least", 0.15, 0.01, "Rejects greys and near-whites, whose hue is meaningless. Raise it to drop pale, washed-out blue.");
  const valMin = mk("Brightness at least", 0.1, 0.01, "Rejects near-black — cracks, plucked holes and the shadow at a plate edge.");

  const band = (): PoreColorBand => ({
    hue_lo: Number(hueLo.value),
    hue_hi: Number(hueHi.value),
    sat_min: Number(satMin.value),
    val_min: Number(valMin.value),
  });

  // ---- preview ------------------------------------------------------------
  const previewWrap = document.createElement("div");
  previewWrap.style.margin = "8px 0";
  wrap.appendChild(previewWrap);

  const previewFrac = document.createElement("div");
  previewFrac.className = "eq-note";
  wrap.appendChild(previewFrac);

  const img = document.createElement("img");
  img.style.maxWidth = "100%";
  img.style.display = "none";
  previewWrap.appendChild(img);

  let previewSeq = 0;
  const preview = async (): Promise<void> => {
    const id = plateSel.value;
    if (!id) return;
    const seq = ++previewSeq;
    previewFrac.textContent = "Measuring this plate…";
    try {
      const res = await runPoreArea({
        well_id: well.well_id,
        dataset: dsSel.value,
        band: band(),
        preview_image_id: id,
        only_image_id: id,
        // No set_name: tuning writes nothing.
      });
      // A slider moved while the last run was in flight — drop the stale answer rather than let
      // it overwrite the newer one.
      if (seq !== previewSeq) return;
      if (res.preview_png) {
        img.src = `data:image/png;base64,${res.preview_png}`;
        img.style.display = "";
      }
      const p = res.plates[0];
      previewFrac.textContent = p
        ? `Red is what would be counted: ${(p.pore_fraction * 100).toFixed(1)}% of this plate.`
        : res.skipped.join("; ");
    } catch (e) {
      if (seq !== previewSeq) return;
      previewFrac.textContent = String(e);
    }
  };

  const previewBtn = document.createElement("button");
  previewBtn.className = "btn";
  previewBtn.textContent = "Preview this plate";
  previewBtn.addEventListener("click", () => void preview());
  wrap.appendChild(previewBtn);

  for (const el of [hueLo, hueHi, satMin, valMin, plateSel]) {
    el.addEventListener("change", () => void preview());
  }
  dsSel.addEventListener("change", () => {
    void (async () => {
      await loadPlates();
      await preview();
    })();
  });

  // ---- measure the delivery ----------------------------------------------
  const out = document.createElement("div");
  wrap.appendChild(out);

  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Measure every declared plate";
  wrap.appendChild(runBtn);

  let last: PoreResult | null = null;

  const render = (res: PoreResult): void => {
    out.innerHTML = "";
    last = res;

    const table = document.createElement("table");
    table.className = "data-table";
    const head = document.createElement("tr");
    for (const h of ["Plate", "Depth", "Pore area"]) {
      const th = document.createElement("th");
      th.textContent = h;
      head.appendChild(th);
    }
    table.appendChild(head);
    for (const p of res.plates) {
      const tr = document.createElement("tr");
      for (const v of [p.name, String(p.depth_top), `${(p.pore_fraction * 100).toFixed(1)}%`]) {
        const td = document.createElement("td");
        td.textContent = v;
        tr.appendChild(td);
      }
      table.appendChild(tr);
    }
    out.appendChild(table);

    // Named, never a count. A silent subset reads as a complete answer.
    if (res.skipped.length) {
      const sk = document.createElement("div");
      sk.className = "eq-note";
      sk.style.color = "var(--warn)";
      sk.textContent = `Left out: ${res.skipped.join("; ")}`;
      out.appendChild(sk);
    }
    for (const n of res.notes) {
      const d = document.createElement("div");
      d.className = "eq-note";
      d.textContent = n;
      out.appendChild(d);
    }

    const setIn = document.createElement("input");
    setIn.className = "form-control";
    setIn.value = "TS";
    const saveBtn = document.createElement("button");
    saveBtn.className = "btn btn-accent";
    saveBtn.textContent = "Save as point data";
    const saveRow = document.createElement("div");
    saveRow.style.display = "flex";
    saveRow.style.gap = "8px";
    saveRow.appendChild(setIn);
    saveRow.appendChild(saveBtn);
    out.appendChild(
      formRow(
        "Save as delivery",
        saveRow,
        "Stored as point samples at each plate's depth, under the PETROGRAPHY dataset — not as a " +
          "curve, because a section measures the one plug it was cut from."
      )
    );

    saveBtn.addEventListener("click", () => {
      void (async () => {
        saveBtn.disabled = true;
        try {
          const saved = await runPoreArea({
            well_id: well.well_id,
            dataset: dsSel.value,
            band: band(),
            set_name: setIn.value || "TS",
          });
          const [ds, name] = saved.written ?? ["PETROGRAPHY", setIn.value];
          setStatus(`Saved ${saved.plates.length} pore measurement(s) as ${ds} / ${name}`);
          recordProcess("Edit", `Pore area on ${dsSel.value}: ${saved.plates.length} plate(s) → ${ds}/${name}`, well.well_name);
          bumpDataVersion();
        } catch (e) {
          setStatus(String(e));
        } finally {
          saveBtn.disabled = false;
        }
      })();
    });
  };

  runBtn.addEventListener("click", () => {
    void (async () => {
      runBtn.disabled = true;
      runBtn.textContent = "Measuring…";
      try {
        render(
          await runPoreArea({ well_id: well.well_id, dataset: dsSel.value, band: band() })
        );
      } catch (e) {
        out.innerHTML = "";
        const err = document.createElement("div");
        err.className = "eq-note";
        err.style.color = "var(--warn)";
        err.textContent = String(e);
        out.appendChild(err);
      } finally {
        runBtn.disabled = false;
        runBtn.textContent = "Measure every declared plate";
      }
    })();
  });

  void preview();
  void last;
}
