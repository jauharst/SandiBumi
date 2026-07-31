import {
  listImageDatasets,
  listWellImages,
  poreSupport,
  runPoreArea,
  stainSchemes,
  type ImageInfo,
  type PoreColorBand,
  type PoreResult,
  type StainBand,
  type StainClass,
  type StainSpec,
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
  // ---- the plate the rest of the delivery is corrected onto ----------------
  //
  // Separate from the tuning plate on purpose: with a reference chosen you want to preview a
  // DIFFERENT plate, to see whether the correction carried the band onto it. One picker doing both
  // jobs could never answer that question.
  const refSel = document.createElement("select");
  refSel.className = "form-control";

  const loadPlates = async (): Promise<void> => {
    plates = await listWellImages(well.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    plateSel.innerHTML = "";
    refSel.innerHTML = "";
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "— none: read every plate as delivered —";
    refSel.appendChild(none);
    for (const p of plates) {
      const o = document.createElement("option");
      o.value = p.image_id;
      // The preparation is shown in the picker because it decides whether the plate can be
      // measured at all — finding that out only after pressing Measure wastes the run.
      const prep = p.prepared === "blue_epoxy" ? "" : p.prepared === "plain" ? " — not impregnated" : " — preparation not stated";
      o.textContent = `${p.name} @ ${p.depth_top}${prep}`;
      o.disabled = p.prepared !== "blue_epoxy";
      plateSel.appendChild(o);
      refSel.appendChild(o.cloneNode(true));
    }
    const first = plates.find((p) => p.prepared === "blue_epoxy");
    if (first) plateSel.value = first.image_id;
  };
  await loadPlates();
  wrap.appendChild(formRow("Tune on plate", plateSel, "The band is judged by eye on one plate, then applied to the delivery."));
  wrap.appendChild(
    formRow(
      "Reference plate",
      refSel,
      "Pick the plate the band reads correctly and every other plate is colour-corrected onto it " +
        "before the band is applied — which is how one band serves a delivery shot under more " +
        "than one light. Leave it at none for a delivery photographed in a single session."
    )
  );
  /** The reference, or nothing. Read at run time so changing the picker needs no rewiring. */
  const refId = (): string | undefined => refSel.value || undefined;

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

  // Pore geometry is opt-in: it needs scipy, and the area fraction must keep working where scipy
  // is not installed.
  const geomChk = document.createElement("input");
  geomChk.type = "checkbox";
  const geomLabel = document.createElement("label");
  geomLabel.appendChild(geomChk);
  geomLabel.appendChild(document.createTextNode(" Also measure each pore's shape and size (needs scipy)"));
  geomLabel.style.display = "block";
  wrap.appendChild(geomLabel);

  const minPx = mk("Smallest pore (pixels)", 20, 1, "Below this a blob is speckle. In PIXELS on purpose — it is what the picture can resolve, not a size in the rock, and it must mean the same on a plate with no scale.");

  // Grains are the other phase of the same segmentation: whatever the pore rule did not claim.
  // Opt-in for the same reason as the pore geometry — it needs scipy.
  const grainChk = document.createElement("input");
  grainChk.type = "checkbox";
  const grainLabel = document.createElement("label");
  grainLabel.appendChild(grainChk);
  grainLabel.appendChild(document.createTextNode(" Also outline each grain and measure its size (needs scipy)"));
  grainLabel.style.display = "block";
  wrap.appendChild(grainLabel);

  const minGrainPx = mk("Smallest grain (pixels)", 50, 1, "Below this a patch of solid is debris or a sliver, not a grain. In PIXELS, same reasoning as the pore floor.");
  const sepPx = mk("Grain separation (pixels)", 20, 1, "How close two grain centres may be before they count as one grain. Over-segmentation is what this gets wrong — watch the yellow outlines in the preview, not the table.");

  const wickChk = document.createElement("input");
  wickChk.type = "checkbox";
  const wickLabel = document.createElement("label");
  wickLabel.appendChild(wickChk);
  wickLabel.appendChild(
    document.createTextNode(" Also report Wicksell-corrected sizes (stored under _W names)")
  );
  wickLabel.style.display = "block";
  wrap.appendChild(wickLabel);

  // The grain fields only mean anything once grains are on. Hidden rather than disabled, because a
  // greyed field still reads as a setting that applies.
  const grainRows = [minGrainPx, sepPx].map((i) => i.closest(".form-row") as HTMLElement | null);
  const syncGrainRows = (): void => {
    for (const r of grainRows) if (r) r.hidden = !grainChk.checked;
    // `style.display` and NOT the `hidden` attribute: this label carries an inline `display:block`,
    // and a display rule beats `hidden` every time — the gotcha the ribbon panels hit twice. Setting
    // the attribute here left the row fully visible at 19px tall.
    wickLabel.style.display = grainChk.checked ? "block" : "none";
  };
  grainChk.addEventListener("change", syncGrainRows);
  syncGrainRows();

  // ---- the stain ----------------------------------------------------------
  // Off by default and it must stay that way: a stain assumed is a mineral fraction invented. The
  // scheme picker offers published identifications (Friedman 1959, Dickson 1966) with generic
  // colour bands — the same split as the epoxy band, and the reason a scheme can ship at all.
  const stainChk = document.createElement("input");
  stainChk.type = "checkbox";
  const stainLabel = document.createElement("label");
  stainLabel.appendChild(stainChk);
  stainLabel.appendChild(document.createTextNode(" Also read the stain (mineral area fractions)"));
  stainLabel.style.display = "block";
  wrap.appendChild(stainLabel);

  const schemes = await stainSchemes().catch(() => [] as [string, StainClass[]][]);
  const schemeSel = document.createElement("select");
  schemeSel.className = "form-control";
  for (const [name] of schemes) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = name;
    schemeSel.appendChild(o);
  }
  const schemeRow = formRow(
    "Stain",
    schemeSel,
    "Must match what each plate says it was stained with — a plate that disagrees is refused by name, because reading the wrong scheme returns mineral fractions that are wrong and entirely plausible.",
  );
  wrap.appendChild(schemeRow);

  // The class list is editable: the identifications are published, the colours are not, and what
  // a stained calcite photographs as depends on the dye batch, the lamp and the scan.
  const classBox = document.createElement("div");
  classBox.className = "eq-note";
  wrap.appendChild(classBox);
  const classInputs: { mineral: string; band: StainBand; inputs: HTMLInputElement[] }[] = [];
  const buildClasses = (): void => {
    classBox.textContent = "";
    classInputs.length = 0;
    const cls = schemes.find(([n]) => n === schemeSel.value)?.[1] ?? [];
    for (const c of cls) {
      const row = document.createElement("div");
      row.style.display = "flex";
      row.style.gap = "6px";
      row.style.alignItems = "center";
      row.style.marginBottom = "3px";
      const name = document.createElement("span");
      name.textContent = c.mineral;
      name.style.minWidth = "9rem";
      row.appendChild(name);
      const inputs: HTMLInputElement[] = [];
      for (const [key, step] of [
        ["hue_lo", 5],
        ["hue_hi", 5],
        ["sat_min", 0.05],
        ["sat_max", 0.05],
      ] as const) {
        const i = document.createElement("input");
        i.className = "form-control";
        i.type = "number";
        i.step = String(step);
        i.style.width = "5rem";
        i.value = String(c.band[key]);
        i.title = key;
        row.appendChild(i);
        inputs.push(i);
      }
      classBox.appendChild(row);
      classInputs.push({ mineral: c.mineral, band: { ...c.band }, inputs });
    }
    const legend = document.createElement("div");
    legend.textContent = "hue from / hue to / saturation at least / at most";
    legend.style.opacity = "0.7";
    classBox.appendChild(legend);
  };
  buildClasses();
  schemeSel.addEventListener("change", buildClasses);

  const stainSpec = (): StainSpec | null => {
    if (!stainChk.checked) return null;
    return {
      stain: schemeSel.value,
      classes: classInputs.map((c) => ({
        mineral: c.mineral,
        band: {
          ...c.band,
          hue_lo: Number(c.inputs[0].value),
          hue_hi: Number(c.inputs[1].value),
          sat_min: Number(c.inputs[2].value),
          sat_max: Number(c.inputs[3].value),
        },
      })),
    };
  };

  const syncStainRows = (): void => {
    schemeRow.hidden = !stainChk.checked;
    // `style.display`, not the `hidden` attribute — classBox has none of its own, but keeping the
    // two toggles the same way is what stops the next one being written the broken way.
    classBox.style.display = stainChk.checked ? "" : "none";
  };
  stainChk.addEventListener("change", syncStainRows);
  syncStainRows();


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
        reference_image_id: refId(),
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
    // The dimensional columns appear only when something was calibrated — an empty µm column on
    // an uncalibrated delivery invites reading the pixel numbers as microns.
    const anyGeom = res.plates.some((p) => p.geometry);
    const anySized = res.plates.some((p) => p.geometry?.d50_um != null);
    // Only on a normalized run — with no reference there is no shift to report, and an empty
    // column would read as "every plate matched" rather than "nothing was compared".
    const anyShift = res.plates.some((p) => Number.isFinite(p.cast_shift));
    const anyGrain = res.plates.some((p) => p.grains);
    const anyGrainSized = res.plates.some((p) => p.grains?.d50_app_um != null);
    const anyW = res.plates.some((p) => p.grains?.d50_w_um != null);
    const cols = ["Plate", "Depth", "Pore area"];
    if (anyShift) cols.push("Shift");
    if (anyGeom) cols.push("Pores", "Aspect", "Roundness");
    if (anySized) cols.push("D10 µm", "D50 µm", "D90 µm");
    if (anyGrain) cols.push("Grains", "Contact");
    // "app" is in the header, not only in the stored item name: a reader looking at the table has
    // to be told which of the two they are looking at.
    if (anyGrainSized) cols.push("GD50 app µm", "Sort app φ");
    if (anyW) cols.push("GD50 W µm", "Sort W φ");
    // Every mineral the run reported, in the scheme's own order, plus the remainder. The remainder
    // column is never optional: it is what says whether the mineral columns are a mineralogy.
    const minerals: string[] = [];
    for (const p of res.plates) {
      for (const [m] of p.stain?.fractions ?? []) if (!minerals.includes(m)) minerals.push(m);
    }
    if (minerals.length) cols.push(...minerals, "Unclassified");
    for (const h of cols) {
      const th = document.createElement("th");
      th.textContent = h;
      head.appendChild(th);
    }
    table.appendChild(head);
    for (const p of res.plates) {
      const tr = document.createElement("tr");
      const g = p.geometry;
      const vals = [p.name, String(p.depth_top), `${(p.pore_fraction * 100).toFixed(1)}%`];
      if (p.scene_dominated) {
        // The number is still shown — it is what the band has to be tuned against — but it must
        // never read as a measurement, because this plate is not going to be stored.
        tr.style.color = "var(--warn)";
        tr.title =
          `This plate is mostly the colour you called pore (its own median hue is ` +
          `${p.scene_hue.toFixed(0)}°, inside the band), so the rule is matching the background. ` +
          `Not a porosity, and not stored — tune the band on this plate.`;
        vals[2] += " ⚠";
      } else if (p.band_missed) {
        // The mirror, and the one that hides: near zero looks exactly like a tight rock, so it
        // would never be queried. Marked the same way, because it is the same kind of non-answer.
        tr.style.color = "var(--warn)";
        tr.title =
          `The band claimed less than one pore's worth of this plate. Either the section is ` +
          `nonporous or the correction did not reach it — its light sat ` +
          `${p.cast_shift.toFixed(0)}° from the reference plate's — and the picture cannot say ` +
          `which. Not stored. Tune a band on this plate, or make it the reference.`;
        vals[2] += " ⚠";
      }
      // The size of the correction, beside the answer it produced. A plate that had to move a long
      // way is one to look at, and it is the only thing on the row that says so.
      if (anyShift) vals.push(Number.isFinite(p.cast_shift) ? `${p.cast_shift.toFixed(0)}°` : "");
      if (anyGeom) {
        vals.push(
          g ? String(g.n) : "",
          g ? g.aspect_p50.toFixed(2) : "",
          g ? g.shape_p50.toFixed(2) : ""
        );
      }
      // A blank, never a zero and never a pixel figure: this plate stated no scale.
      const um = (v: number | null | undefined): string => (v == null ? "" : v.toFixed(0));
      if (anySized) {
        vals.push(um(g?.d10_um), um(g?.d50_um), um(g?.d90_um));
      }
      const gr = p.grains;
      if (anyGrain) {
        vals.push(gr ? String(gr.n) : "", gr ? gr.contact_p50.toFixed(2) : "");
      }
      const phi = (v: number | null | undefined): string => (v == null ? "" : v.toFixed(2));
      if (anyGrainSized) {
        vals.push(um(gr?.d50_app_um), phi(gr?.sort_app_phi));
      }
      if (anyW) {
        vals.push(um(gr?.d50_w_um), phi(gr?.sort_w_phi));
      }
      if (minerals.length) {
        const st = p.stain;
        const pct = (v: number | undefined): string => (v == null ? "" : `${(v * 100).toFixed(1)}%`);
        for (const mname of minerals) {
          vals.push(pct(st?.fractions.find(([m]) => m === mname)?.[1]));
        }
        vals.push(pct(st?.unclassified));
      }
      for (const v of vals) {
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
            reference_image_id: refId(),
            geometry: geomChk.checked,
            min_pore_px: Number(minPx.value) || 20,
            grains: grainChk.checked,
            min_grain_px: Number(minGrainPx.value) || 50,
            grain_sep_px: Number(sepPx.value) || 20,
            wicksell: wickChk.checked,
            stain: stainSpec(),
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
          await runPoreArea({
            well_id: well.well_id,
            dataset: dsSel.value,
            band: band(),
            reference_image_id: refId(),
            geometry: geomChk.checked,
            min_pore_px: Number(minPx.value) || 20,
            grains: grainChk.checked,
            min_grain_px: Number(minGrainPx.value) || 50,
            grain_sep_px: Number(sepPx.value) || 20,
            wicksell: wickChk.checked,
            stain: stainSpec(),
          })
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
