import {
  listImageDatasets,
  listPlugChoices,
  listWellImages,
  poreSupport,
  runPoreArea,
  stainSchemes,
  type ImageInfo,
  type PlugChoice,
  type PlugSource,
  type PoreColorBand,
  type ReferenceZone,
  type PoreResult,
  type StainBand,
  type StainClass,
  type StainSpec,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { buildColourBand } from "./colourBand";
import { buildPlateStrip } from "./plateStrip";
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

  const plateOption = (p: ImageInfo): HTMLOptionElement => {
    const o = document.createElement("option");
    o.value = p.image_id;
    // The preparation is shown in the picker because it decides whether the plate can be measured
    // at all — finding that out only after pressing Measure wastes the run.
    const prep = p.prepared === "blue_epoxy" ? "" : p.prepared === "plain" ? " — not impregnated" : " — preparation not stated";
    o.textContent = `${p.name} @ ${p.depth_top}${prep}`;
    o.disabled = p.prepared !== "blue_epoxy";
    return o;
  };

  /** Refill a reference picker, keeping the current choice if this dataset still has that plate. */
  const fillPlates = (sel: HTMLSelectElement, emptyLabel: string): void => {
    const keep = sel.value;
    sel.innerHTML = "";
    const none = document.createElement("option");
    none.value = "";
    none.textContent = emptyLabel;
    sel.appendChild(none);
    for (const p of plates) sel.appendChild(plateOption(p));
    if (keep) sel.value = keep;
  };

  /** Every per-interval picker, so a dataset change refills them all. */
  const zoneSelects: HTMLSelectElement[] = [];

  const loadPlates = async (): Promise<void> => {
    plates = await listWellImages(well.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    plateSel.innerHTML = "";
    for (const p of plates) plateSel.appendChild(plateOption(p));
    const first = plates.find((p) => p.prepared === "blue_epoxy");
    if (first) plateSel.value = first.image_id;
    fillPlates(refSel, "— none: read every plate as delivered —");
    for (const s of zoneSelects) fillPlates(s, "— choose a plate —");
    // The preparation decides whether a plate can be measured at all, so it is on the tile rather
    // than discovered after pressing Measure. Greyed, never hidden.
    filmstrip.load(plates, (pl) =>
      pl.prepared === "blue_epoxy"
        ? null
        : pl.prepared === "plain"
          ? "not impregnated — a blue rule over an unimpregnated section returns a porosity assembled from blue-ish feldspar and edge artefact"
          : "preparation not stated — declare it in Plate Details, it cannot be read off the pixels"
    );
    filmstrip.mark(plateSel.value);
  };

  // The delivery as PICTURES. A petrographer choosing which plate to tune a threshold on is
  // choosing a picture; a list of names makes them open six to find the one they meant.
  const filmstrip = buildPlateStrip((id) => {
    plateSel.value = id;
    filmstrip.mark(id);
    void preview();
  });
  wrap.appendChild(filmstrip.el);

  await loadPlates();
  wrap.appendChild(formRow("Tune on plate", plateSel, "Or click one in the strip above. The band is judged by eye on one plate, then applied to the delivery."));
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

  // ---- a reference per cored interval -------------------------------------
  //
  // A delivery spanning two cored intervals is two different rocks, usually photographed on two
  // different days, and one reference plate serves both only by accident. Giving each interval its
  // own lifted agreement with core porosity in BOTH on a real delivery — a refinement rather than a
  // rescue, and now one the Check against figure below can settle instead of it being taken on
  // trust.
  const zoneBox = document.createElement("div");
  const zoneRows: { row: HTMLElement; top: HTMLInputElement; base: HTMLInputElement; ref: HTMLSelectElement }[] = [];

  const addZone = (): void => {
    const row = document.createElement("div");
    row.style.display = "flex";
    row.style.gap = "6px";
    row.style.alignItems = "center";
    row.style.marginBottom = "3px";
    const num = (ph: string): HTMLInputElement => {
      const i = document.createElement("input");
      i.className = "form-control";
      i.type = "number";
      i.step = "any";
      i.placeholder = ph;
      i.style.width = "8rem";
      return i;
    };
    // A blank end is not a missing value — "from the top of the well" and "down to total depth" is
    // how a cored interval at either end of the well is actually described.
    const top = num("from (blank = top)");
    const base = num("to (blank = TD)");
    const ref = document.createElement("select");
    ref.className = "form-control";
    ref.style.flex = "1";
    fillPlates(ref, "— choose a plate —");
    zoneSelects.push(ref);
    const del = document.createElement("button");
    del.className = "btn";
    del.textContent = "✕";
    del.title = "Remove this interval";
    del.addEventListener("click", () => {
      const i = zoneRows.findIndex((z) => z.row === row);
      if (i >= 0) zoneRows.splice(i, 1);
      const j = zoneSelects.indexOf(ref);
      if (j >= 0) zoneSelects.splice(j, 1);
      row.remove();
    });
    row.append(top, base, ref, del);
    zoneBox.appendChild(row);
    zoneRows.push({ row, top, base, ref });
  };

  const addZoneBtn = document.createElement("button");
  addZoneBtn.className = "btn";
  addZoneBtn.textContent = "+ Interval with its own reference";
  addZoneBtn.addEventListener("click", addZone);
  const zoneWrap = document.createElement("div");
  zoneWrap.append(zoneBox, addZoneBtn);
  wrap.appendChild(
    formRow(
      "Per-interval references",
      zoneWrap,
      "Overrules the plate above inside a depth range. Intervals may touch but not cross — across " +
        "an overlap, which reference a section got would come down to the order of this list. A " +
        "section no interval reaches falls back to the plate above; with none set it is refused by " +
        "name rather than measured uncorrected alongside corrected ones."
    )
  );

  /** The declared intervals. A row with no plate chosen is sent as it stands, so the run refuses it
   *  by name — silently ignoring a half-filled interval would look exactly like one that applied. */
  const zoneList = (): ReferenceZone[] =>
    zoneRows.map((z) => ({
      top: z.top.value === "" ? null : Number(z.top.value),
      base: z.base.value === "" ? null : Number(z.base.value),
      image_id: z.ref.value,
    }));

  // ---- the yardstick ------------------------------------------------------
  //
  // Directly under the reference plate because this is the dial for that knob. Choosing a reference
  // plate moved rank agreement with core porosity by 3.5x on a real delivery — more than the colour
  // band did — and the worst of three picks was worse than not correcting at all. None of which is
  // visible in the preview, which shows only what the band claimed and not whether what it claimed
  // is the rock.
  const checkSel = document.createElement("select");
  checkSel.className = "form-control";
  const choices = await listPlugChoices([well.well_id]).catch(() => [] as PlugChoice[]);
  const noCheck = document.createElement("option");
  noCheck.value = "";
  noCheck.textContent = choices.length
    ? "— none: do not check —"
    : "— this well has no plug measurements —";
  checkSel.appendChild(noCheck);
  for (const [i, c] of choices.entries()) {
    const o = document.createElement("option");
    o.value = String(i);
    o.textContent = c.label;
    checkSel.appendChild(o);
  }
  // Core porosity is picked by default where the well has it. The check is worth having by default
  // rather than on request — a setting nobody thought to verify is exactly the one that ships.
  const cpor = choices.findIndex((c) => c.kind === "core" && c.item.toUpperCase() === "CPOR");
  if (cpor >= 0) checkSel.value = String(cpor);
  wrap.appendChild(
    formRow(
      "Check against",
      checkSel,
      "Scores the run against a measurement of the same plugs that this app did not produce — " +
        "usually the laboratory's core porosity. It is the only way to tell a good reference plate " +
        "from a bad one; the preview cannot say."
    )
  );
  /** The chosen yardstick, or nothing. */
  const checkSrc = (): PlugSource | undefined => {
    const c = choices[Number(checkSel.value)];
    return c ? { kind: c.kind, dataset: c.dataset, item: c.item } : undefined;
  };

  // ---- the colour band, as a colour ---------------------------------------
  //
  // A hue threshold cannot be judged from a number: 205 degrees means nothing to anyone, and the
  // thing a petrographer actually knows is what the epoxy in front of them looks like. So the band
  // is a wheel with two draggable ends and the floors are sliders carrying the gradient they move
  // along - the conditioning workspace's rule, applied to the measurement.
  //
  // Round starting numbers, and said to be a starting point: a two-decimal threshold here would
  // look like somebody's regression result, and there is no regression behind it.
  const bandCtl = buildColourBand({ hue_lo: 180, hue_hi: 260, sat_min: 0.15, val_min: 0.1 }, () => void preview());
  wrap.appendChild(
    formRow(
      "Pore colour",
      bandCtl.el,
      "Drag the ends of the band onto the colour your blue epoxy actually is, or press Pick the " +
        "pore colour and click a pore on the plate. A starting band, not a calibration."
    )
  );

  const mk = (label: string, value: number, step: number, hint: string): HTMLInputElement => {
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = String(step);
    i.value = String(value);
    wrap.appendChild(formRow(label, i, hint));
    return i;
  };

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


  const band = (): PoreColorBand => bandCtl.get();

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

  /** The plate WITHOUT the mask on it, at the same size — what the eyedropper reads and what Hold
   *  to compare shows. Both need the CORRECTED pixels the band is applied to, not the delivered
   *  ones, which is why the runner sends it beside the overlay rather than the viewer fetching the
   *  stored picture. */
  let plainPng: string | null = null;
  let overlayPng: string | null = null;

  const compare = document.createElement("button");
  compare.className = "btn";
  compare.textContent = "Hold to compare";
  compare.disabled = true;
  compare.title = "Shows the plate without the mask, so you can see what the band claimed against what is actually there.";
  const showPlain = (on: boolean): void => {
    const src = on ? plainPng : overlayPng;
    if (src) img.src = `data:image/png;base64,${src}`;
  };
  compare.addEventListener("pointerdown", () => showPlain(true));
  for (const e of ["pointerup", "pointerleave", "pointercancel"]) {
    compare.addEventListener(e, () => showPlain(false));
  }

  // ---- the eyedropper -----------------------------------------------------
  //
  // The "pick a grey" idea from the conditioning workspace, pointed the other way: there the click
  // says "this should be neutral", here it says "this is pore". Both replace a number nobody can
  // picture with the thing itself.
  const pickBtn = document.createElement("button");
  pickBtn.className = "btn";
  pickBtn.textContent = "Pick the pore colour";
  pickBtn.disabled = true;
  pickBtn.title = "Then click a pore on the plate below. The band re-centres on that colour, keeping the width you have set.";
  let picking = false;
  const setPicking = (on: boolean): void => {
    picking = on;
    pickBtn.classList.toggle("btn-accent", on);
    img.style.cursor = on ? "crosshair" : "";
  };
  pickBtn.addEventListener("click", () => setPicking(!picking));
  img.addEventListener("click", (ev) => {
    if (!picking || !plainPng) return;
    // Read the colour out of the UN-MASKED copy: clicking inside the red overlay would otherwise
    // sample the overlay and re-centre the band on the mask's own colour, which is circular.
    const probe = new Image();
    probe.onload = () => {
      const r = img.getBoundingClientRect();
      const fx = (ev.clientX - r.left) / Math.max(1, r.width);
      const fy = (ev.clientY - r.top) / Math.max(1, r.height);
      const cv = document.createElement("canvas");
      cv.width = probe.naturalWidth;
      cv.height = probe.naturalHeight;
      const ctx = cv.getContext("2d");
      if (!ctx) return;
      ctx.drawImage(probe, 0, 0);
      // A small patch, and its MEDIAN — a single pixel on a scanned plate is as likely to be a
      // speck as the epoxy, the same reason the white-balance pick takes a median.
      const px = Math.round(fx * (cv.width - 1));
      const py = Math.round(fy * (cv.height - 1));
      const rad = Math.max(2, Math.round(0.006 * Math.max(cv.width, cv.height)));
      const x0 = Math.max(0, px - rad);
      const y0 = Math.max(0, py - rad);
      const d = ctx.getImageData(x0, y0, Math.min(2 * rad + 1, cv.width - x0), Math.min(2 * rad + 1, cv.height - y0)).data;
      const rs: number[] = [];
      const gs: number[] = [];
      const bs: number[] = [];
      for (let i = 0; i < d.length; i += 4) {
        rs.push(d[i]);
        gs.push(d[i + 1]);
        bs.push(d[i + 2]);
      }
      const med = (a: number[]): number => {
        a.sort((x, y) => x - y);
        return a[Math.floor(a.length / 2)] ?? 0;
      };
      bandCtl.pickFrom([med(rs), med(gs), med(bs)]);
      setPicking(false);
    };
    probe.src = `data:image/png;base64,${plainPng}`;
  });

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
        reference_zones: zoneList(),
        preview_image_id: id,
        only_image_id: id,
        // No set_name: tuning writes nothing.
      });
      // A slider moved while the last run was in flight — drop the stale answer rather than let
      // it overwrite the newer one.
      if (seq !== previewSeq) return;
      if (res.preview_png) {
        overlayPng = res.preview_png;
        plainPng = res.plain_png ?? null;
        img.src = `data:image/png;base64,${res.preview_png}`;
        img.style.display = "";
        pickBtn.disabled = !plainPng;
        compare.disabled = !plainPng;
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
  const previewRow = document.createElement("div");
  previewRow.style.display = "flex";
  previewRow.style.gap = "6px";
  previewRow.style.flexWrap = "wrap";
  previewRow.style.margin = "4px 0";
  previewRow.append(pickBtn, compare);
  previewWrap.parentElement?.insertBefore(previewRow, previewWrap);
  previewBtn.addEventListener("click", () => void preview());
  wrap.appendChild(previewBtn);

  plateSel.addEventListener("change", () => {
    filmstrip.mark(plateSel.value);
    void preview();
  });
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

  /** Which reference plate and band a run used, in one phrase. Recorded beside its score so the
   *  comparison table below identifies what was actually changed between two runs. */
  const settingLabel = (): string => {
    const ref = refSel.value ? (refSel.selectedOptions[0]?.textContent ?? "?") : "none";
    const n = zoneRows.length;
    // The intervals are named by their count rather than listed: the row has to stay readable, and
    // what changed between two runs is almost always how many intervals there were, not their ends.
    const head = n ? `${n} interval${n > 1 ? "s" : ""}${refSel.value ? ` + ${ref}` : ""}` : ref;
    const b = bandCtl.get();
    return `${head} · band ${Math.round(b.hue_lo)}–${Math.round(b.hue_hi)}°`;
  };

  /** Every scored run this session. Kept here rather than persisted because it describes an
   *  afternoon's tuning, not the project — but kept at all because a single coefficient answers
   *  nothing: 0.24 is a poor result next to 0.53 and a good one next to 0.11, and the only way to
   *  know which is to have seen the alternatives. */
  const tried: { setting: string; n: number; spearman: number }[] = [];

  const renderAgreement = (res: PoreResult): void => {
    const a = res.agreement;
    if (!a) return;
    const box = document.createElement("div");
    box.className = "eq-note";

    if (!a.n_pairs || !Number.isFinite(a.spearman)) {
      box.style.color = "var(--warn)";
      box.textContent = a.notes.join(" ") || "Nothing could be paired, so this run was not scored.";
      out.appendChild(box);
      return;
    }

    tried.push({ setting: settingLabel(), n: a.n_pairs, spearman: a.spearman });
    box.innerHTML = "";
    const strong = document.createElement("strong");
    strong.textContent = `Agreement with ${a.reference_label}: rank ${a.spearman.toFixed(2)} over ${a.n_pairs} plug(s).`;
    box.appendChild(strong);
    box.appendChild(
      document.createTextNode(
        ` Straight-line ${a.pearson.toFixed(2)}. Medians ${a.measured_median.toFixed(3)} measured ` +
          `against ${a.reference_median.toFixed(3)}. Compare settings on the RANK figure — a ` +
          `section reads systematically below a plug's porosity without being wrong about which ` +
          `plug is the better rock, and only the rank figure ignores that offset.`
      )
    );
    out.appendChild(box);

    // A number with nothing to compare it to is not yet a decision. Once there are two, the table
    // is the whole point of the feature.
    if (tried.length > 1) {
      // Best among the rows that are actually comparable with the latest run — a row scored on a
      // different set of plugs is not in the running, whatever its number. Bolding the highest
      // figure regardless would recommend the row the next line goes on to say cannot be read
      // straight across, and the bold is the part people act on.
      const here = tried.filter((t) => t.n === a.n_pairs);
      const best = Math.max(...here.map((t) => t.spearman));
      const t = document.createElement("table");
      t.className = "data-table";
      const hr = document.createElement("tr");
      for (const h of ["Setting tried", "Plugs", "Rank agreement"]) {
        const th = document.createElement("th");
        th.textContent = h;
        hr.appendChild(th);
      }
      t.appendChild(hr);
      for (const row of tried) {
        const tr = document.createElement("tr");
        if (row.n === a.n_pairs && row.spearman === best) tr.style.fontWeight = "bold";
        // Two runs that scored a different number of plugs were scored on different rock, so their
        // coefficients are not a fair comparison. Marked rather than hidden: the run is still
        // informative, it just cannot be read straight across.
        if (row.n !== a.n_pairs) {
          tr.style.color = "var(--warn)";
          tr.title =
            "Scored on a different set of plugs from the latest run, because a different set of " +
            "plates was refused. Not directly comparable with the rows that share the current count.";
        }
        for (const v of [row.setting, String(row.n), row.spearman.toFixed(2)]) {
          const td = document.createElement("td");
          td.textContent = v;
          tr.appendChild(td);
        }
        t.appendChild(tr);
      }
      out.appendChild(t);
      const hint = document.createElement("div");
      hint.className = "eq-note";
      hint.textContent =
        "Settings tried this session, best in bold. If changing the reference plate moves this " +
        "column a long way, the delivery was not photographed under one light and wants measuring " +
        "in groups. If a reference scores below the run with no reference at all, it is making " +
        "things worse.";
      out.appendChild(hint);
    }
  };

  const render = (res: PoreResult): void => {
    out.innerHTML = "";
    last = res;
    // Before the plate table, because it is the verdict on the settings above and the table is the
    // detail behind it.
    renderAgreement(res);

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
    // Only once more than one plate served as a reference — with a single one the column would
    // repeat the picker above on every row, and a shift is unambiguous anyway.
    const anyRef = new Set(res.plates.map((p) => p.reference_name).filter(Boolean)).size > 1;
    const anyGrain = res.plates.some((p) => p.grains);
    const anyGrainSized = res.plates.some((p) => p.grains?.d50_app_um != null);
    const anyW = res.plates.some((p) => p.grains?.d50_w_um != null);
    const cols = ["Plate", "Depth", "Pore area"];
    if (anyRef) cols.push("Reference");
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
      // Which plate this section was corrected onto, beside the size of that correction. With two
      // references in play a shift of 40 degrees means nothing until you know what it is 40 from.
      if (anyRef) vals.push(p.reference_name);
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
            reference_zones: zoneList(),
            geometry: geomChk.checked,
            min_pore_px: Number(minPx.value) || 20,
            grains: grainChk.checked,
            min_grain_px: Number(minGrainPx.value) || 50,
            grain_sep_px: Number(sepPx.value) || 20,
            wicksell: wickChk.checked,
            stain: stainSpec(),
            check_against: checkSrc(),
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
            reference_zones: zoneList(),
            geometry: geomChk.checked,
            min_pore_px: Number(minPx.value) || 20,
            grains: grainChk.checked,
            min_grain_px: Number(minGrainPx.value) || 50,
            grain_sep_px: Number(sepPx.value) || 20,
            wicksell: wickChk.checked,
            stain: stainSpec(),
            check_against: checkSrc(),
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
