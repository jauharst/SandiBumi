import {
  applyCoreLook,
  bakeCoreImages,
  coreImageSupport,
  getWellImage,
  listImageDatasets,
  listImageRecipes,
  listWellImages,
  previewCoreImage,
  type CoreRecipe,
  type CorePreview,
  type CropBox,
  type ImageInfo,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow, openModal } from "./modal";
import { requireWell } from "./needWell";

/**
 * Conditioning a core slab photograph (Data ▸ Tools ▾ ▸ Condition Core Photos…).
 *
 * A core photograph arrives as somebody's snapshot: the box a degree off square on the bench, the
 * tray and the tape in frame, and whatever colour the core shed's lights had that afternoon. None
 * of that is the rock, and all of it goes into a report.
 *
 * **The controls are the picture wherever they can be.** A geologist judges a photograph by looking
 * at it, so the delivery is a strip of thumbnails rather than a dropdown of filenames, the crop is a
 * drag on the image rather than four numbers, the white balance is a click on a grey patch rather
 * than three gains, and each slider's TRACK carries the gradient it moves along — blue to amber,
 * green to magenta, grey to vivid. The readout beside a slider is there to be read back, not to be
 * typed into.
 *
 * **The preview comes from the backend, and that is deliberate.** Re-implementing the pipeline in
 * canvas would be faster to drag but would put the same correction in two languages, and the two
 * would drift — the standing warning that keeps the log view and the composite in step. What is
 * tuned here is literally what gets baked, at a smaller size.
 *
 * **Nothing is written until Apply.** The conditioning is reversible afterwards too: the import is
 * kept the first time a recipe is baked and Reset puts the photograph back byte for byte.
 */
export async function openCoreConditionDialog(): Promise<void> {
  const well = appState.selectedWell.get();
  if (!well) {
    requireWell("Condition core photos");
    return;
  }
  const wrap = document.createElement("div");
  openModal(`Condition core photos — ${well.well_name}`, wrap, 1000);

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Straighten, crop and colour-correct a core photograph. Nothing is destroyed — the picture as " +
    "imported is kept, and Reset puts it back exactly.";
  wrap.appendChild(intro);

  if (!(await coreImageSupport().catch(() => false))) {
    const warn = document.createElement("div");
    warn.className = "eq-note";
    warn.style.color = "var(--warn)";
    warn.textContent =
      "This needs numpy and Pillow in the Python the app uses (pip install numpy pillow). " +
      "Nothing else in the app is affected.";
    wrap.appendChild(warn);
    return;
  }

  const dsSel = document.createElement("select");
  dsSel.className = "form-control";
  const datasets = await listImageDatasets(well.well_id).catch(() => [] as [string, number][]);
  for (const [name, n] of datasets) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = `${name} — ${n} picture(s)`;
    dsSel.appendChild(o);
  }
  // A core-photograph delivery is what this tool is for, so it opens on one where the well has it.
  const photo = datasets.find(([n]) => n.toUpperCase().includes("CORE") || n.toUpperCase().includes("PHOTO"));
  if (photo) dsSel.value = photo[0];
  wrap.appendChild(formRow("Picture dataset", dsSel));

  if (!datasets.length) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.style.color = "var(--warn)";
    none.textContent = "This well has no pictures. Import some with Data ▸ Import ▸ Images…";
    wrap.appendChild(none);
    return;
  }

  // ---- the delivery, as pictures -----------------------------------------
  const strip = document.createElement("div");
  strip.className = "cond-strip";
  wrap.appendChild(strip);

  let plates: ImageInfo[] = [];
  /** Each picture's recipe, edited here and written only on Apply. */
  const recipes = new Map<string, CoreRecipe>();
  let current = "";
  const urls: string[] = [];
  /** Fetches a thumbnail only when its tile scrolls into the strip — a core-photograph delivery is
   *  routinely hundreds of pictures, and loading them all would pull hundreds of megabytes through
   *  the bridge to fill a strip nobody has scrolled to yet. */
  const seen = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        const el = e.target as HTMLElement;
        seen.unobserve(el);
        const id = el.dataset.id ?? "";
        void (async () => {
          try {
            const buf = await getWellImage(id);
            const mime = plates.find((p) => p.image_id === id)?.mime ?? "image/jpeg";
            const url = URL.createObjectURL(new Blob([buf], { type: mime }));
            urls.push(url);
            el.style.backgroundImage = `url("${url}")`;
          } catch {
            /* a picture the viewer cannot decode still gets its tile and its label */
          }
        })();
      }
    },
    { root: strip, rootMargin: "200px" }
  );

  const recipeOf = (id: string): CoreRecipe => {
    let r = recipes.get(id);
    if (!r) {
      r = {};
      recipes.set(id, r);
    }
    return r;
  };

  /** What each picture's recipe is IN THE PROJECT, so the dialog can tell an edit from an applied
   *  change. Without it the status line would read "not yet applied" the moment after Apply. */
  const storedJson = new Map<string, string>();
  /** The colour of the patch picked on each picture this session — for the swatch only. It is not
   *  part of the transform (the gains are), and it is deliberately NOT carried across pictures: a
   *  swatch showing the previous photograph's grey would be a lie about this one. */
  const pickedColour = new Map<string, string>();

  /** A recipe reduced to its meaning, so two can be compared.
   *
   *  Rounded on the way, because a crop composed from two drags carries float noise that no
   *  comparison of raw numbers survives — and a picture would then read as edited for ever. */
  const canon = (r: CoreRecipe): string =>
    JSON.stringify([
      Number((r.rotate_deg ?? 0).toFixed(4)),
      r.crop
        ? [r.crop.x, r.crop.y, r.crop.w, r.crop.h].map((v) => Number(v.toFixed(6)))
        : null,
      r.gain ? r.gain.map((v) => Number(v.toFixed(6))) : null,
      Number((r.warmth ?? 0).toFixed(4)),
      Number((r.tint ?? 0).toFixed(4)),
      Number((r.exposure ?? 0).toFixed(4)),
      Number((r.contrast ?? 0).toFixed(4)),
      Number((r.saturation ?? 0).toFixed(4)),
    ]);
  const PLAIN = canon({});

  /** Where this picture stands: as delivered, applied, or edited and not yet written. Three states
   *  and not two, because "conditioned" and "conditioned in the project" are different facts and
   *  the second is the one that reaches a report. */
  const describe = (): string => {
    if (!current) return "";
    const now = canon(recipeOf(current));
    const wasJson = storedJson.get(current) ?? "";
    let was = PLAIN;
    try {
      was = wasJson ? canon(JSON.parse(wasJson) as CoreRecipe) : PLAIN;
    } catch {
      /* an unreadable stored recipe reads as nothing applied */
    }
    if (now === was) return now === PLAIN ? "As imported." : "Applied — this is what the project holds.";
    return now === PLAIN
      ? "Cleared here — press Reset this photo to put the project back."
      : "Edited — not applied yet.";
  };

  const markStrip = (): void => {
    for (const el of Array.from(strip.children) as HTMLElement[]) {
      const id = el.dataset.id ?? "";
      el.classList.toggle("is-current", id === current);
      // The dot marks what the PROJECT holds, not what is being tried on screen — a strip that
      // lit up while a slider moved would say a photograph had been conditioned when nothing had
      // been written.
      el.classList.toggle("is-conditioned", (storedJson.get(id) ?? "") !== "");
    }
  };

  // ---- the picture --------------------------------------------------------
  const stage = document.createElement("div");
  stage.className = "cond-stage";
  const img = document.createElement("img");
  const cropBox = document.createElement("div");
  cropBox.className = "cond-crop";
  cropBox.hidden = true;
  stage.append(img, cropBox);
  wrap.appendChild(stage);

  const hist = document.createElement("canvas");
  hist.height = 64;
  hist.style.width = "100%";
  hist.style.height = "48px";
  hist.style.margin = "4px 0";
  hist.style.background = "var(--bg-panel-alt)";
  hist.style.borderRadius = "var(--r-sm)";
  hist.title =
    "How the corrected picture's brightness is spread, one line per channel. A wall at either end " +
    "is clipping — detail that is now pure black or pure white and cannot come back.";
  wrap.appendChild(hist);

  const status = document.createElement("div");
  status.className = "eq-note";
  wrap.appendChild(status);

  let last: CorePreview | null = null;

  const drawHist = (p: CorePreview): void => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(hist.clientWidth * dpr));
    const h = Math.round(48 * dpr);
    if (hist.width !== w || hist.height !== h) {
      hist.width = w;
      hist.height = h;
    }
    const ctx = hist.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);
    const chans: [number[], string][] = [
      [p.hist_r, "rgba(200,60,50,0.75)"],
      [p.hist_g, "rgba(60,150,70,0.75)"],
      [p.hist_b, "rgba(60,90,200,0.75)"],
    ];
    // Normalised to the tallest bin across ALL channels, so the three stay comparable — scaling
    // each to its own peak would make a flat channel look as full as a peaked one.
    let peak = 1;
    for (const [c] of chans) for (const v of c) peak = Math.max(peak, v);
    for (const [c, colour] of chans) {
      ctx.beginPath();
      ctx.moveTo(0, h);
      for (let i = 0; i < c.length; i++) {
        ctx.lineTo((i / (c.length - 1)) * w, h - (c[i] / peak) * h);
      }
      ctx.lineTo(w, h);
      ctx.closePath();
      ctx.fillStyle = colour;
      ctx.fill();
    }
  };

  let seq = 0;
  let pending: number | null = null;
  const render = async (pick?: { x: number; y: number }): Promise<void> => {
    if (!current) return;
    const mine = ++seq;
    status.textContent = "Rendering…";
    try {
      const res = await previewCoreImage(current, recipeOf(current), pick?.x, pick?.y);
      // A slider moved while this was in flight — drop the stale answer rather than let it
      // overwrite the newer one.
      if (mine !== seq) return;
      last = res;
      img.src = `data:image/png;base64,${res.png}`;
      drawHist(res);
      if (pick && res.picked_gain) {
        recipeOf(current).gain = res.picked_gain;
        if (res.picked_rgb) {
          pickedColour.set(current, `rgb(${res.picked_rgb[0]},${res.picked_rgb[1]},${res.picked_rgb[2]})`);
        }
        syncControls();
        void render();
        return;
      }
      status.textContent = describe();
      markStrip();
    } catch (e) {
      if (mine !== seq) return;
      status.textContent = String(e);
    }
  };

  /** Sliders fire continuously; the backend renders a real picture. Coalesce, then render. */
  const schedule = (): void => {
    if (pending !== null) window.clearTimeout(pending);
    pending = window.setTimeout(() => {
      pending = null;
      void render();
    }, 160);
  };

  // ---- before / after -----------------------------------------------------
  //
  // Held rather than toggled: a comparison is a glance, and a toggle leaves the user one click away
  // from tuning against the wrong picture without noticing.
  const compare = document.createElement("button");
  compare.className = "btn";
  compare.textContent = "Hold to compare";
  compare.title = "Shows the picture as imported for as long as the button is held.";
  const showBefore = (on: boolean): void => {
    if (!last) return;
    img.src = `data:image/png;base64,${on ? last.before_png : last.png}`;
    cropBox.hidden = on || !recipeOf(current).crop;
  };
  compare.addEventListener("pointerdown", () => showBefore(true));
  for (const ev of ["pointerup", "pointerleave", "pointercancel"]) {
    compare.addEventListener(ev, () => showBefore(false));
  }

  // ---- the sliders --------------------------------------------------------
  const sliderBox = document.createElement("div");
  sliderBox.style.marginTop = "6px";
  wrap.appendChild(sliderBox);

  type Key = "rotate_deg" | "exposure" | "contrast" | "saturation" | "warmth" | "tint";
  interface Ctl {
    key: Key;
    input: HTMLInputElement;
    val: HTMLElement;
    fmt: (v: number) => string;
  }
  const ctls: Ctl[] = [];

  const slider = (
    label: string,
    key: Key,
    min: number,
    max: number,
    step: number,
    track: string,
    fmt: (v: number) => string,
    hint: string
  ): void => {
    const row = document.createElement("div");
    row.className = "cond-row";
    const lab = document.createElement("label");
    lab.className = "form-label";
    lab.textContent = label;
    lab.title = hint;
    const input = document.createElement("input");
    input.type = "range";
    input.className = "cond-slider";
    input.min = String(min);
    input.max = String(max);
    input.step = String(step);
    input.value = "0";
    input.style.background = track;
    input.title = hint;
    const val = document.createElement("span");
    val.className = "cond-val";
    const reset = document.createElement("button");
    reset.className = "btn";
    reset.textContent = "↺";
    reset.title = `Back to no ${label.toLowerCase()}`;
    reset.style.padding = "0 4px";
    row.append(lab, input, val, reset);
    sliderBox.appendChild(row);

    const apply = (v: number): void => {
      recipeOf(current)[key] = v;
      val.textContent = fmt(v);
      schedule();
    };
    input.addEventListener("input", () => apply(Number(input.value)));
    reset.addEventListener("click", () => {
      input.value = "0";
      apply(0);
    });
    ctls.push({ key, input, val, fmt });
  };

  // The tracks carry the gradient each slider moves ALONG, which is the whole point: a geologist
  // dragging "Warmth" should see amber to the right before touching it.
  const grey = "linear-gradient(to right, var(--bg-panel-alt), var(--border-strong))";
  slider("Straighten", "rotate_deg", -10, 10, 0.1, grey, (v) => `${v.toFixed(1)}°`,
    "Rotates the picture clockwise. Applied before the crop, so the empty corners a rotation leaves are cut away rather than printed.");
  slider("Brightness", "exposure", -2, 2, 0.05,
    "linear-gradient(to right, #1a1a1a, #808080, #f2f2f2)", (v) => `${v > 0 ? "+" : ""}${v.toFixed(2)}`,
    "In stops, like a camera. Watch the histogram: a wall at either end is detail that cannot come back.");
  slider("Contrast", "contrast", -1, 1, 0.02,
    "linear-gradient(to right, #8a8a8a, #b0b0b0, #ffffff 50%, #000000 50%, #ffffff)", (v) => v.toFixed(2),
    "Spreads the tones apart around mid grey.");
  slider("Colour", "saturation", -1, 1, 0.02,
    "linear-gradient(to right, #9a9a9a, #b8926a, #d2691e)", (v) => v.toFixed(2),
    "How strong the colours are. Pull it left to judge shape and fabric without the colour arguing.");
  slider("Warmth", "warmth", -1, 1, 0.02,
    "linear-gradient(to right, #4a7fd0, #cfcfcf, #d8a45a)", (v) => v.toFixed(2),
    "Blue to amber. A trim on top of the grey you picked — a core shed rarely has a colour card.");
  slider("Green / magenta", "tint", -1, 1, 0.02,
    "linear-gradient(to right, #6fb36f, #cfcfcf, #c06fb3)", (v) => v.toFixed(2),
    "The other white-balance axis. Fluorescent light usually needs a nudge towards magenta.");

  // ---- white balance, crop, and the two modes -----------------------------
  const swatch = document.createElement("span");
  swatch.className = "cond-swatch";
  swatch.title = "The patch you clicked. Its colour is what the correction is making neutral.";

  const pickBtn = document.createElement("button");
  pickBtn.className = "btn";
  pickBtn.textContent = "Pick a grey";
  pickBtn.title =
    "Then click a patch that should be neutral — the colour card, the grey tray, a white label. " +
    "Everything shifts so that patch reads grey.";
  const clearWb = document.createElement("button");
  clearWb.className = "btn";
  clearWb.textContent = "Clear";
  clearWb.title = "Back to the colours as delivered.";

  const cropBtn = document.createElement("button");
  cropBtn.className = "btn";
  cropBtn.textContent = "Crop";
  cropBtn.title = "Then drag a rectangle on the picture to keep only what is inside it.";
  const clearCrop = document.createElement("button");
  clearCrop.className = "btn";
  clearCrop.textContent = "Clear";
  clearCrop.title = "Back to the whole picture.";

  let mode: "none" | "pick" | "crop" = "none";
  const setMode = (m: typeof mode): void => {
    mode = mode === m ? "none" : m;
    stage.classList.toggle("is-picking", mode === "pick");
    stage.classList.toggle("is-cropping", mode === "crop");
    pickBtn.classList.toggle("btn-accent", mode === "pick");
    cropBtn.classList.toggle("btn-accent", mode === "crop");
  };
  pickBtn.addEventListener("click", () => setMode("pick"));
  cropBtn.addEventListener("click", () => setMode("crop"));
  clearWb.addEventListener("click", () => {
    recipeOf(current).gain = null;
    pickedColour.delete(current);
    syncControls();
    void render();
  });
  clearCrop.addEventListener("click", () => {
    recipeOf(current).crop = null;
    cropBox.hidden = true;
    void render();
  });

  const toolRow = document.createElement("div");
  toolRow.style.display = "flex";
  toolRow.style.gap = "6px";
  toolRow.style.alignItems = "center";
  toolRow.style.flexWrap = "wrap";
  toolRow.style.margin = "6px 0";
  toolRow.append(pickBtn, swatch, clearWb, cropBtn, clearCrop, compare);
  wrap.appendChild(toolRow);

  /** Where a pointer is on the PICTURE, as fractions — never pixels. The displayed size changes
   *  with the window and the stored copy is already capped, so a fraction is the only measure that
   *  means the same thing in the preview and in the full-size bake. */
  const atFraction = (ev: PointerEvent): { x: number; y: number } => {
    const r = img.getBoundingClientRect();
    return {
      x: Math.min(1, Math.max(0, (ev.clientX - r.left) / Math.max(1, r.width))),
      y: Math.min(1, Math.max(0, (ev.clientY - r.top) / Math.max(1, r.height))),
    };
  };

  const placeCropBox = (c: CropBox): void => {
    const r = img.getBoundingClientRect();
    const s = stage.getBoundingClientRect();
    cropBox.style.left = `${r.left - s.left + c.x * r.width}px`;
    cropBox.style.top = `${r.top - s.top + c.y * r.height}px`;
    cropBox.style.width = `${c.w * r.width}px`;
    cropBox.style.height = `${c.h * r.height}px`;
    cropBox.hidden = false;
  };

  let dragFrom: { x: number; y: number } | null = null;
  stage.addEventListener("pointerdown", (ev) => {
    if (!current || mode === "none") return;
    if (mode === "pick") {
      void render(atFraction(ev));
      setMode("none");
      return;
    }
    dragFrom = atFraction(ev);
    stage.setPointerCapture(ev.pointerId);
  });
  stage.addEventListener("pointermove", (ev) => {
    if (!dragFrom) return;
    const to = atFraction(ev);
    placeCropBox({
      x: Math.min(dragFrom.x, to.x),
      y: Math.min(dragFrom.y, to.y),
      w: Math.abs(to.x - dragFrom.x),
      h: Math.abs(to.y - dragFrom.y),
    });
  });
  stage.addEventListener("pointerup", (ev) => {
    if (!dragFrom) return;
    const to = atFraction(ev);
    const c: CropBox = {
      x: Math.min(dragFrom.x, to.x),
      y: Math.min(dragFrom.y, to.y),
      w: Math.abs(to.x - dragFrom.x),
      h: Math.abs(to.y - dragFrom.y),
    };
    dragFrom = null;
    setMode("none");
    // A stray click is not a crop of nothing. Below a couple of per cent in either direction the
    // user missed the drag, and cropping the picture to a speck would look like a broken tool.
    if (c.w < 0.02 || c.h < 0.02) {
      cropBox.hidden = !recipeOf(current).crop;
      return;
    }
    // The crop is stored against the picture the user dragged on, which is already the ROTATED,
    // previously-cropped one — so a second crop composes with the first rather than replacing it.
    const prev = recipeOf(current).crop;
    recipeOf(current).crop = prev
      ? { x: prev.x + c.x * prev.w, y: prev.y + c.y * prev.h, w: c.w * prev.w, h: c.h * prev.h }
      : c;
    cropBox.hidden = true;
    void render();
  });

  /** Puts every control where the current picture's recipe says it should be. */
  const syncControls = (): void => {
    const r = recipeOf(current);
    for (const c of ctls) {
      const v = Number(r[c.key] ?? 0);
      c.input.value = String(v);
      c.val.textContent = c.fmt(v);
    }
    // Per PICTURE. A recipe loaded from the project carries the gains but not the colour they came
    // from, so that case gets a tick rather than a swatch of whatever the last photograph was.
    const colour = pickedColour.get(current);
    swatch.style.background = colour ?? "transparent";
    swatch.textContent = !colour && r.gain ? "\u2713" : "";
    swatch.title = colour
      ? "The patch you clicked. Its colour is what the correction is making neutral."
      : r.gain
        ? "A neutral patch was picked on this photograph in an earlier session."
        : "No white balance set — the colours are as delivered.";
    cropBox.hidden = true;
  };

  // ---- apply --------------------------------------------------------------
  const applyOne = document.createElement("button");
  applyOne.className = "btn btn-accent";
  applyOne.textContent = "Apply to this photo";

  const applyAll = document.createElement("button");
  applyAll.className = "btn";
  applyAll.textContent = "Apply this light to the whole run";
  applyAll.title =
    "Copies the colour half only — the brightness, contrast, colour, warmth and the grey you " +
    "picked. Each picture keeps its own straightening and crop, because the box sits differently " +
    "on the bench in every frame.";

  const resetOne = document.createElement("button");
  resetOne.className = "btn";
  resetOne.textContent = "Reset this photo";
  resetOne.title = "Puts this picture back exactly as it was imported.";

  const applyRow = document.createElement("div");
  applyRow.style.display = "flex";
  applyRow.style.gap = "8px";
  applyRow.style.marginTop = "8px";
  applyRow.append(applyOne, applyAll, resetOne);
  wrap.appendChild(applyRow);

  /** Every picture's recipe as it stands in the project — the state an undo has to put back. */
  const snapshot = async (): Promise<[string, string][]> =>
    listImageRecipes(well.well_id, dsSel.value).catch(() => [] as [string, string][]);

  const restore = async (before: [string, string][]): Promise<void> => {
    // Re-baked from each import, so this is an exact restore rather than an approximate one: the
    // conditioning is a pure function of the photograph and the recipe.
    await bakeCoreImages(
      before.map(([image_id, json]) => ({
        image_id,
        recipe: json ? (JSON.parse(json) as CoreRecipe) : {},
      }))
    );
    await reload();
    bumpDataVersion();
  };

  const runBake = async (
    label: string,
    work: () => Promise<{ conditioned: number; restored: number; skipped: string[] }>
  ): Promise<void> => {
    const before = await snapshot();
    applyOne.disabled = true;
    applyAll.disabled = true;
    resetOne.disabled = true;
    status.textContent = "Applying…";
    try {
      const res = await work();
      const after = await snapshot();
      pushUndo({
        label,
        undo: () => restore(before),
        redo: () => restore(after),
      });
      setStatus(`${label}: ${res.conditioned} conditioned`);
      recordProcess("Edit", `${label} on ${dsSel.value}`, well.well_name);
      bumpDataVersion();
      // AFTER the reload, or the reload's own render would overwrite it and the line would read
      // "not applied yet" the moment after Apply.
      await reload();
      status.textContent =
        `${res.conditioned} conditioned, ${res.restored} put back as imported.` +
        (res.skipped.length ? ` Left alone: ${res.skipped.join("; ")}` : "");
    } catch (e) {
      status.textContent = String(e);
    } finally {
      applyOne.disabled = false;
      applyAll.disabled = false;
      resetOne.disabled = false;
    }
  };

  applyOne.addEventListener("click", () => {
    void runBake("condition core photo", () =>
      bakeCoreImages([{ image_id: current, recipe: recipeOf(current) }])
    );
  });
  applyAll.addEventListener("click", () => {
    void runBake("condition core run", () =>
      // The colour/framing split is made in Rust, so what "the light" means is one rule rather
      // than one per caller.
      applyCoreLook(well.well_id, dsSel.value, recipeOf(current))
    );
  });
  resetOne.addEventListener("click", () => {
    recipes.set(current, {});
    pickedColour.delete(current);
    syncControls();
    void runBake("reset core photo", () => bakeCoreImages([{ image_id: current, recipe: {} }]));
  });

  // ---- loading ------------------------------------------------------------
  const select = (id: string): void => {
    current = id;
    syncControls();
    markStrip();
    void render();
  };

  async function reload(): Promise<void> {
    plates = await listWellImages(well!.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    const stored = await listImageRecipes(well!.well_id, dsSel.value).catch(() => [] as [string, string][]);
    recipes.clear();
    storedJson.clear();
    for (const [id, json] of stored) {
      storedJson.set(id, json ?? "");
      try {
        recipes.set(id, json ? (JSON.parse(json) as CoreRecipe) : {});
      } catch {
        // An unreadable recipe reads as "nothing applied" rather than blocking the picture: the
        // pixels are still there and the user can condition them again.
        recipes.set(id, {});
      }
    }
    strip.textContent = "";
    for (const p of plates) {
      const t = document.createElement("div");
      t.className = "cond-thumb";
      t.dataset.id = p.image_id;
      t.title = `${p.name} @ ${p.depth_top}${p.depth_base != null ? `–${p.depth_base}` : ""}`;
      const cap = document.createElement("span");
      cap.className = "cond-thumb-label";
      cap.textContent = p.name;
      t.appendChild(cap);
      t.addEventListener("click", () => select(p.image_id));
      strip.appendChild(t);
      seen.observe(t);
    }
    if (!plates.some((p) => p.image_id === current)) {
      current = plates[0]?.image_id ?? "";
    }
    syncControls();
    markStrip();
    if (current) void render();
  }

  dsSel.addEventListener("change", () => {
    current = "";
    void reload();
  });
  await reload();

  // Object URLs and the observer outlive the modal unless they are dropped with it.
  const root = document.getElementById("modal-root");
  if (root) {
    const mo = new MutationObserver(() => {
      if (root.contains(wrap)) return;
      for (const u of urls) URL.revokeObjectURL(u);
      urls.length = 0;
      seen.disconnect();
      mo.disconnect();
    });
    mo.observe(root, { childList: true, subtree: true });
  }
}
