import {
  applyCoreLook,
  bakeCoreImages,
  buildCoreStrips,
  CORE_STRIP_DATASET,
  coreImageSupport,
  extractCoreLog,
  getWellImage,
  listImageDatasets,
  listImageRecipes,
  listWellImages,
  previewCoreImage,
  type CoreRecipe,
  type CorePreview,
  type CoreLogResult,
  type CropBox,
  type ImageInfo,
  type Quad,
} from "../ipc";
import { loadCurveNames } from "./plotCommon";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow } from "./modal";

/** Which kind of picture the workspace was opened for. */
export type ConditionSubject = "core" | "plate";

/**
 * Conditioning a picture of rock — a core slab photograph (Advance ▸ Core Imaging ▸ Core Photos…)
 * or a thin section (Advance ▸ Petrography ▸ Condition Plates…).
 *
 * **A dock PANE, not a popup, and one pane per KIND of picture.** Conditioning is a long sitting
 * job: crop, straighten, pick a grey, judge it, move to the next box. A modal covers the log view
 * the result is read against, and the two subjects get their own pane because a core photograph
 * and a thin section are two deliveries with two recipes — sharing one pane would mean correcting
 * one loses your place in the other. Standing rule from Jauhar (2026-08-01): tools open as
 * working panes.
 *
 * A core photograph arrives as somebody's snapshot: the box a degree off square on the bench, the
 * tray and the tape in frame, and whatever colour the core shed's lights had that afternoon. None
 * of that is the rock, and all of it goes into a report. **A thin section arrives with exactly the
 * same problems** — lifted out of a workbook at whatever angle it was scanned, under whatever lamp
 * the microscope had — so it gets the same workspace rather than a second one written to look like
 * it. Two dialogs would be two places for the wording, the gamma and the white-balance rule to
 * drift, which is the `followCore.ts` argument.
 *
 * The one real difference is at the bottom: the proxy trace and the depth strips are core-only, and
 * not by omission. A thin section is cut from ONE plug and covers no interval, so there is no axis
 * to read a log along and nothing to stretch a strip over.
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
export async function buildCoreConditionContent(
  subject: ConditionSubject = "core",
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const plate = subject === "plate";
  const what = plate ? "plates" : "core photos";
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  wrap.className = "module-pane";

  // A pane says this itself rather than calling `requireWell`. That refusal exists for a click
  // where nothing visible happens — here the pane opens, so it is the place to say why it is
  // empty, and it fills in the moment a well is selected.
  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent =
      `Select a well in the Wells pane first — ${what} are conditioned one well at a time.`;
    wrap.appendChild(none);
    return { el: wrap };
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent = plate
    ? "Straighten, crop and colour-correct a thin section — the same tools the core photographs " +
      "use, because the job is the same one. Nothing is destroyed: the picture as imported is kept, " +
      "and Reset puts it back exactly. What you apply here is what Pore Area measures."
    : "Straighten, crop and colour-correct a core photograph. Nothing is destroyed — the picture as " +
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
    return { el: wrap };
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
  // Opens on the kind of delivery it was asked for, so neither entry point starts on the other's
  // pictures. A plate delivery is named for what it is; a core one is named for the core.
  const want = plate
    ? (n: string) => /THIN|SECTION|PLATE|SEM|PETROG/.test(n)
    : (n: string) => n.includes("CORE") || n.includes("PHOTO");
  const first = datasets.find(([n]) => want(n.toUpperCase()));
  if (first) dsSel.value = first[0];
  wrap.appendChild(formRow("Picture dataset", dsSel));

  if (!datasets.length) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.style.color = "var(--warn)";
    none.textContent = "This well has no pictures. Import some with Data ▸ Import ▸ Images…";
    wrap.appendChild(none);
    return { el: wrap };
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
      r.quad ? r.quad.map((p) => p.map((v) => Number(v.toFixed(6)))) : null,
      r.crop
        ? [r.crop.x, r.crop.y, r.crop.w, r.crop.h].map((v) => Number(v.toFixed(6)))
        : null,
      r.gain ? r.gain.map((v) => Number(v.toFixed(6))) : null,
      Number((r.warmth ?? 0).toFixed(4)),
      Number((r.tint ?? 0).toFixed(4)),
      Number((r.exposure ?? 0).toFixed(4)),
      Number((r.contrast ?? 0).toFixed(4)),
      Number((r.saturation ?? 0).toFixed(4)),
      Number((r.denoise ?? 0).toFixed(4)),
      Number((r.clarity ?? 0).toFixed(4)),
      Number((r.sharpen ?? 0).toFixed(4)),
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

  // ---- the four corners of the box ----------------------------------------
  //
  // A box photographed from one end is a trapezoid: the far end is drawn shorter than the near end,
  // so a depth read straight down the frame runs fast at one end and slow at the other. Straighten
  // cannot touch that — rotating a trapezoid gives a rotated trapezoid — which is why this is four
  // draggable corners rather than another slider.
  const SVG_NS = "http://www.w3.org/2000/svg";
  const quadLayer = document.createElement("div");
  quadLayer.className = "cond-quad";
  quadLayer.hidden = true;
  const quadSvg = document.createElementNS(SVG_NS, "svg");
  const quadPoly = document.createElementNS(SVG_NS, "polygon");
  quadSvg.appendChild(quadPoly);
  quadLayer.appendChild(quadSvg);
  const CORNER = ["top-left", "top-right", "bottom-right", "bottom-left"];
  const handles: HTMLElement[] = CORNER.map((name, i) => {
    const h = document.createElement("div");
    h.className = "cond-handle";
    h.dataset.i = String(i);
    h.title = `Drag onto the ${name} corner of the core itself`;
    quadLayer.appendChild(h);
    return h;
  });

  stage.append(img, cropBox, quadLayer);
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

  /** The recipe the PICTURE is drawn with, which is not always the recipe being edited.
   *
   *  While the corners are being dragged the picture has to be shown unrectified and uncropped —
   *  you cannot point at the box's corner in a photograph that has already been squared up to it,
   *  and a crop would have cut the corners off. Everything else stays on, because the light is what
   *  makes the box edge findable in the first place. */
  const viewRecipe = (): CoreRecipe =>
    mode === "quad" ? { ...recipeOf(current), quad: null, crop: null } : recipeOf(current);

  let seq = 0;
  let pending: number | null = null;
  const render = async (pick?: { x: number; y: number }): Promise<void> => {
    if (!current) return;
    const mine = ++seq;
    status.textContent = "Rendering…";
    try {
      const res = await previewCoreImage(current, viewRecipe(), pick?.x, pick?.y);
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

  // ---- the other light ----------------------------------------------------
  //
  // A core shed shoots the same box twice, once in white light and once under ultraviolet, and the
  // UV frame is where an oil show lives — fluorescence that is simply not in the white-light
  // picture. So the two have to be looked at TOGETHER: a bright patch under UV means nothing until
  // you can see what rock it sits on, and a sand that looks clean in white light says nothing about
  // whether it has oil in it.
  //
  // Held, not toggled, for the same reason the before/after is: the answer is a glance, and a
  // toggle leaves you tuning the wrong picture.
  const pairSel = document.createElement("select");
  pairSel.className = "form-control";
  pairSel.style.maxWidth = "16rem";
  const pairBtn = document.createElement("button");
  pairBtn.className = "btn";
  pairBtn.textContent = "Hold for the pair";
  pairBtn.disabled = true;
  pairBtn.title =
    "Shows the same depth from the other delivery — the UV frame beside the white-light one — for " +
    "as long as the button is held. Each is shown with its OWN conditioning.";

  /** Every picture of the paired dataset, and its stored recipe. Loaded when the pair is chosen. */
  let pairPlates: ImageInfo[] = [];
  const pairRecipes = new Map<string, CoreRecipe>();
  let pairPng: string | null = null;

  /** The paired picture covering the same rock. Matched on the depth INTERVAL rather than on the
   *  name, because the two deliveries are two different cameras' filenames for one box — and
   *  matched on overlap rather than on nearest top, so a UV frame shot in two halves still finds
   *  the white-light box it belongs to. */
  const pairFor = (info: ImageInfo): ImageInfo | null => {
    const aTop = info.depth_top;
    const aBot = info.depth_base ?? info.depth_top;
    let best: ImageInfo | null = null;
    let bestOverlap = 0;
    for (const p of pairPlates) {
      const bTop = p.depth_top;
      const bBot = p.depth_base ?? p.depth_top;
      const overlap = Math.min(aBot, bBot) - Math.max(aTop, bTop);
      // A zero-thickness sample still matches when it falls inside the other's interval.
      const score = overlap > 0 ? overlap : aBot === aTop || bBot === bTop ? (Math.abs(aTop - bTop) < 0.5 ? 1e-6 : 0) : 0;
      if (score > bestOverlap) {
        bestOverlap = score;
        best = p;
      }
    }
    return best;
  };

  const loadPair = async (): Promise<void> => {
    pairPlates = [];
    pairRecipes.clear();
    pairPng = null;
    pairBtn.disabled = true;
    const ds = pairSel.value;
    if (!ds) return;
    pairPlates = await listWellImages(well.well_id, ds).catch(() => [] as ImageInfo[]);
    for (const [id, json] of await listImageRecipes(well.well_id, ds).catch(() => [] as [string, string][])) {
      if (!json) continue;
      try {
        pairRecipes.set(id, JSON.parse(json) as CoreRecipe);
      } catch {
        /* an unreadable stored recipe reads as nothing applied */
      }
    }
    pairBtn.disabled = pairPlates.length === 0;
  };

  const showPair = async (on: boolean): Promise<void> => {
    if (!on) {
      if (last) img.src = `data:image/png;base64,${last.png}`;
      return;
    }
    const info = plates.find((p) => p.image_id === current);
    const mate = info ? pairFor(info) : null;
    if (!mate) {
      status.textContent = "Nothing in that delivery covers this depth.";
      return;
    }
    if (!pairPng) {
      // Rendered through the SAME pipeline with the MATE's own recipe — not this picture's. A UV
      // frame shown under a white-light photograph's white balance would be a picture of the
      // correction rather than of the fluorescence.
      const res = await previewCoreImage(mate.image_id, pairRecipes.get(mate.image_id) ?? {}).catch(() => null);
      if (!res) return;
      pairPng = res.png;
    }
    img.src = `data:image/png;base64,${pairPng}`;
    status.textContent = `${mate.name} — ${pairSel.value}`;
  };
  pairBtn.addEventListener("pointerdown", () => void showPair(true));
  for (const ev of ["pointerup", "pointerleave", "pointercancel"]) {
    pairBtn.addEventListener(ev, () => void showPair(false));
  }
  pairSel.addEventListener("change", () => void loadPair());

  // ---- the sliders --------------------------------------------------------
  const sliderBox = document.createElement("div");
  sliderBox.style.marginTop = "6px";
  wrap.appendChild(sliderBox);

  type Key =
    | "rotate_deg"
    | "exposure"
    | "contrast"
    | "saturation"
    | "warmth"
    | "tint"
    | "clarity"
    | "sharpen"
    | "denoise";
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
      syncDetailNote();
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

  // The three below are a different kind of correction and are grouped apart on purpose — see the
  // note under them.
  const detailHead = document.createElement("div");
  detailHead.className = "eq-note";
  detailHead.style.margin = "10px 0 4px";
  detailHead.textContent = "Detail";
  sliderBox.appendChild(detailHead);

  slider("Local contrast", "clarity", 0, 1, 0.02,
    "linear-gradient(to right, #7a7a7a, #a8a8a8 45%, #1c1c1c 50%, #f0f0f0 55%, #e0e0e0)", (v) => v.toFixed(2),
    "Lifts the shadowed end of a box towards the lit end, tile by tile, instead of brightening the whole picture. What to reach for when one lamp lit the box from one side.");
  slider("Denoise", "denoise", 0, 1, 0.02,
    "linear-gradient(to right, #8a8a8a, #b4b4b4)", (v) => v.toFixed(2),
    "Takes out speckle and dust without softening the grain boundary beside it. Its reach follows the picture's size, so the preview and the saved copy remove the same thing.");
  slider("Sharpen", "sharpen", 0, 1, 0.02,
    "linear-gradient(to right, #9a9a9a, #d8d8d8)", (v) => v.toFixed(2),
    "Lifts real edges — bedding, grain boundaries, fractures. Applied after the denoise, or it would sharpen the speckle.");

  const detailNote = document.createElement("div");
  detailNote.className = "eq-note";
  detailNote.style.margin = "2px 0 0";
  detailNote.textContent =
    "These three move a pixel's NEIGHBOURS rather than its colour, which is what makes a " +
    "photograph readable — and what changes Read the trace. Local contrast roughly halves the " +
    "darkness contrast between clean sand and mudstone, so an equalised box and a plain one no " +
    "longer read on the same scale; sharpening inflates TEX and denoising suppresses it. Read a " +
    "trace off photographs corrected for light and framing only.";
  sliderBox.appendChild(detailNote);

  /** Coloured only while one of the three is actually doing something. A warning that is always red
   *  is a warning nobody reads by the third photograph — so this has to follow the slider as it
   *  moves, not only the picture as it changes. */
  const syncDetailNote = (): void => {
    const r = recipeOf(current);
    detailNote.style.color = r.denoise || r.clarity || r.sharpen ? "var(--warn)" : "";
  };

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

  const quadBtn = document.createElement("button");
  quadBtn.className = "btn";
  quadBtn.textContent = "Square up";
  quadBtn.title =
    "For a box photographed from an angle. Drag the four handles onto the corners of the core " +
    "itself, then press Square up again. The picture is stretched back to the shape the box " +
    "really is — which straightening cannot do, because a tilted trapezoid is still a trapezoid.";
  const clearQuad = document.createElement("button");
  clearQuad.className = "btn";
  clearQuad.textContent = "Clear";
  clearQuad.title = "Back to the picture as the camera framed it.";

  const FRAME: Quad = [
    [0, 0],
    [1, 0],
    [1, 1],
    [0, 1],
  ];
  const copyQuad = (q: Quad): Quad => q.map((p) => [p[0], p[1]]) as Quad;
  let editQuad: Quad = copyQuad(FRAME);

  /** Puts the polygon and its four handles over the picture, in whatever size it is drawn at. */
  const placeQuad = (): void => {
    const r = img.getBoundingClientRect();
    const s = stage.getBoundingClientRect();
    if (r.width < 2 || r.height < 2) return;
    const ox = r.left - s.left;
    const oy = r.top - s.top;
    quadSvg.setAttribute("width", String(Math.round(s.width)));
    quadSvg.setAttribute("height", String(Math.round(s.height)));
    quadPoly.setAttribute(
      "points",
      editQuad.map(([x, y]) => `${ox + x * r.width},${oy + y * r.height}`).join(" ")
    );
    for (let i = 0; i < 4; i++) {
      handles[i].style.left = `${ox + editQuad[i][0] * r.width}px`;
      handles[i].style.top = `${oy + editQuad[i][1] * r.height}px`;
    }
  };
  // The picture changes size when a new one loads or the window moves; the corners are stored as
  // fractions precisely so they survive that, but they still have to be re-drawn.
  img.addEventListener("load", () => {
    if (!quadLayer.hidden) placeQuad();
  });

  let mode: "none" | "pick" | "crop" | "quad" = "none";
  const setMode = (m: typeof mode): void => {
    const was = mode;
    mode = mode === m ? "none" : m;
    stage.classList.toggle("is-picking", mode === "pick");
    stage.classList.toggle("is-cropping", mode === "crop");
    pickBtn.classList.toggle("btn-accent", mode === "pick");
    cropBtn.classList.toggle("btn-accent", mode === "crop");
    quadBtn.classList.toggle("btn-accent", mode === "quad");
    quadBtn.textContent = mode === "quad" ? "Done" : "Square up";
    quadLayer.hidden = mode !== "quad";
    if (mode === "quad") {
      editQuad = copyQuad(recipeOf(current).quad ?? FRAME);
    }
    // Entering or leaving corner mode swaps the picture between rectified and as-framed, so it has
    // to be re-rendered; the other modes only change the cursor.
    if (was === "quad" || mode === "quad") void render();
  };
  pickBtn.addEventListener("click", () => setMode("pick"));
  cropBtn.addEventListener("click", () => setMode("crop"));
  quadBtn.addEventListener("click", () => setMode("quad"));
  clearQuad.addEventListener("click", () => {
    recipeOf(current).quad = null;
    editQuad = copyQuad(FRAME);
    if (mode === "quad") placeQuad();
    void render();
  });

  // ---- dragging a corner ---------------------------------------------------
  let dragHandle: number | null = null;
  quadLayer.addEventListener("pointerdown", (ev) => {
    const i = Number((ev.target as HTMLElement).dataset?.i ?? NaN);
    if (!Number.isFinite(i)) return;
    dragHandle = i;
    quadLayer.setPointerCapture(ev.pointerId);
    ev.preventDefault();
  });
  quadLayer.addEventListener("pointermove", (ev) => {
    if (dragHandle === null) return;
    const p = atFraction(ev);
    editQuad[dragHandle] = [p.x, p.y];
    placeQuad();
  });
  for (const evName of ["pointerup", "pointercancel"]) {
    quadLayer.addEventListener(evName, () => {
      if (dragHandle === null) return;
      dragHandle = null;
      // Stored even while the unrectified picture is still on screen: the polygon IS the feedback
      // here, and re-rendering rectified on every corner would take the corners off screen.
      recipeOf(current).quad = copyQuad(editQuad);
      status.textContent = describe();
      markStrip();
    });
  }
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
  toolRow.append(pickBtn, swatch, clearWb, cropBtn, clearCrop, quadBtn, clearQuad, compare, pairSel, pairBtn);
  wrap.appendChild(toolRow);

  // The pair picker offers every OTHER delivery this well has. It opens on one whose name says
  // ultraviolet, because that is what the control is for — but it is a picker rather than a rule,
  // since a shed labels its deliveries whatever it labels them.
  /** Rebuilt whenever the source changes, because a delivery paired with ITSELF is not a pair — it
   *  would show the same picture and read as a control that does nothing. */
  const refreshPairOptions = (): void => {
    const keep = pairSel.value;
    pairSel.innerHTML = "";
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "— no paired delivery —";
    pairSel.appendChild(none);
    for (const [name] of datasets) {
      if (name === dsSel.value) continue;
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      pairSel.appendChild(o);
    }
    const uv = datasets.find(([n]) => n !== dsSel.value && /\bUV\b|ULTRAVIOLET|FLUOR/i.test(n));
    pairSel.value = keep && keep !== dsSel.value ? keep : (uv?.[0] ?? "");
    void loadPair();
  };
  refreshPairOptions();
  dsSel.addEventListener("change", refreshPairOptions);

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
    // Corner mode owns its own drags, on the layer above — without this a miss between two handles
    // would silently start a crop.
    if (!current || mode === "none" || mode === "quad") return;
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
    editQuad = copyQuad(r.quad ?? FRAME);
    if (!quadLayer.hidden) placeQuad();
    syncDetailNote();
  };

  // ---- apply --------------------------------------------------------------
  const applyOne = document.createElement("button");
  applyOne.className = "btn btn-accent";
  applyOne.textContent = "Apply to this photo";

  const applyAll = document.createElement("button");
  applyAll.className = "btn";
  applyAll.textContent = plate ? "Apply this light to the whole delivery" : "Apply this light to the whole run";
  applyAll.title =
    "Copies the colour half only — the brightness, contrast, colour, warmth and the grey you " +
    "picked. Each picture keeps its own straightening and crop, because " +
    (plate
      ? "a section sits differently under the microscope every time."
      : "the box sits differently on the bench in every frame.");

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

  // ---- reading a log off the photographs ----------------------------------
  //
  // The second half of what a core photograph is for. A conditioned box is a picture of the rock at
  // a known depth, and averaging its pixels down the core gives a continuous trace that can sit in a
  // log track beside GR.
  //
  // Everything here is chosen by looking at the photograph rather than by typing: which way the
  // depth runs, how many rows of core are in the frame, and then a drawn trace to judge.
  // The trace and the depth strips are core-only, and not by omission: a thin section is cut from
  // ONE plug and covers no interval, so there is no axis to read a log along and nothing to stretch
  // a strip over — the same reason `extract_core_log` refuses a picture with no base depth.
  const logBox = document.createElement("div");
  logBox.hidden = plate;
  logBox.style.borderTop = "1px solid var(--border)";
  logBox.style.marginTop = "10px";
  logBox.style.paddingTop = "8px";
  wrap.appendChild(logBox);

  const logTitle = document.createElement("div");
  logTitle.className = "eq-note";
  logTitle.textContent =
    "Read a trace off these photographs \u2014 darkness, redness and texture down the core. They are " +
    "IMAGE measures, not petrophysical properties: darkness follows shale in most clastic sections " +
    "without being a shale volume, which is why nothing here is called VSH.";
  logBox.appendChild(logTitle);

  /** A row of buttons behaving as one choice. More legible than a dropdown for three or four
   *  options, and it shows every option at once \u2014 which is what you want when the answer is read
   *  off the picture above rather than remembered. */
  const segmented = (
    options: { value: string; label: string; title: string }[],
    initial: string,
    onPick: (v: string) => void
  ): { el: HTMLElement; get: () => string } => {
    const row = document.createElement("div");
    row.style.display = "flex";
    row.style.gap = "2px";
    let value = initial;
    const btns: HTMLButtonElement[] = [];
    const paint = (): void => {
      for (const b of btns) b.classList.toggle("btn-accent", b.dataset.value === value);
    };
    for (const o of options) {
      const b = document.createElement("button");
      b.className = "btn";
      b.textContent = o.label;
      b.title = o.title;
      b.dataset.value = o.value;
      b.addEventListener("click", () => {
        value = o.value;
        paint();
        onPick(value);
      });
      btns.push(b);
      row.appendChild(b);
    }
    paint();
    return { el: row, get: () => value };
  };

  const axisPick = segmented(
    [
      { value: "x", label: "\u2192 across", title: "Depth runs along the width of the picture \u2014 a core box laid out left to right." },
      { value: "y", label: "\u2193 down", title: "Depth runs down the picture \u2014 a single vertical strip." },
    ],
    "x",
    () => {}
  );
  logBox.appendChild(formRow("Depth runs", axisPick.el, "Read it off the photograph above."));

  const revChk = document.createElement("input");
  revChk.type = "checkbox";
  const revLabel = document.createElement("label");
  revLabel.appendChild(revChk);
  revLabel.appendChild(document.createTextNode(" Deepest end first (the box is the other way round)"));
  revLabel.style.display = "block";
  logBox.appendChild(revLabel);

  const lanePick = segmented(
    [1, 2, 3, 4, 5, 6].map((n) => ({
      value: String(n),
      label: String(n),
      title:
        n === 1
          ? "One run of core in the frame."
          : `${n} rows of core, read top to bottom. Equal lanes are an approximation \u2014 a real box has unequal rows and gaps \u2014 so for a careful job crop to one row and run this per row.`,
    })),
    "1",
    () => {}
  );
  logBox.appendChild(formRow("Rows of core", lanePick.el, "How many runs of core are laid out in one photograph."));

  const cmpSel = document.createElement("select");
  cmpSel.className = "form-control";
  {
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "\u2014 none: do not check \u2014";
    cmpSel.appendChild(none);
    const names = await loadCurveNames().catch(() => [] as string[]);
    for (const n of names) {
      const o = document.createElement("option");
      o.value = n;
      o.textContent = n;
      cmpSel.appendChild(o);
    }
    // GR by default where the well has it: a trace nobody thought to check is exactly the one that
    // ships, and darkness against GR is the check this measure exists to pass.
    if (names.includes("GR")) cmpSel.value = "GR";
  }
  logBox.appendChild(
    formRow(
      "Check against",
      cmpSel,
      "Reports how each measure tracks a real log over the same interval. It is the only thing that " +
        "says whether the trace is about the rock \u2014 and a strongly NEGATIVE darkness usually means " +
        "the depth axis is the other way round."
    )
  );

  const readBtn = document.createElement("button");
  readBtn.className = "btn btn-accent";
  readBtn.textContent = "Read the trace";
  const writeBtn = document.createElement("button");
  writeBtn.className = "btn";
  writeBtn.textContent = "Save as curves";
  writeBtn.disabled = true;
  // The strip uses the SAME lay-out as the trace, which is why it lives beside it rather than in a
  // dialog of its own: one statement of how the box is laid out, two things read off it.
  const stripBtn = document.createElement("button");
  stripBtn.className = "btn";
  stripBtn.textContent = "Build depth strips";
  stripBtn.title =
    "Cuts every box into its rows and stacks them into one tall picture per box, with the core " +
    "running down it. Put an image track on it in depth mode to see it beside the logs. Building " +
    "again into the same name replaces the last one.";

  // Where the strips land. Visible and editable rather than fixed, because a white-light delivery
  // and a UV one both want strips and one name would have the second quietly replace the first —
  // the same box, twice, and only the second light left.
  const stripTarget = document.createElement("input");
  stripTarget.className = "form-control";
  stripTarget.style.maxWidth = "12rem";
  stripTarget.title = "The picture dataset the strips are written to.";
  /** `CORE STRIP`, plus whatever the source delivery is called beyond "CORE PHOTO" — so a delivery
   *  named CORE PHOTO UV suggests CORE STRIP UV without the user having to think about it. */
  const suggestTarget = (): void => {
    const src = dsSel.value.toUpperCase();
    const extra = src.replace(/CORE|PHOTO|PHOTOS|SLAB/g, "").replace(/\s+/g, " ").trim();
    stripTarget.value = extra ? `${CORE_STRIP_DATASET} ${extra}` : CORE_STRIP_DATASET;
  };
  suggestTarget();
  dsSel.addEventListener("change", suggestTarget);

  const readRow = document.createElement("div");
  readRow.style.display = "flex";
  readRow.style.gap = "8px";
  readRow.style.margin = "6px 0";
  readRow.style.flexWrap = "wrap";
  readRow.append(readBtn, writeBtn, stripBtn, stripTarget);
  logBox.appendChild(readRow);

  const trace = document.createElement("canvas");
  trace.style.width = "100%";
  trace.style.height = "220px";
  trace.style.background = "var(--bg-panel-alt)";
  trace.style.borderRadius = "var(--r-sm)";
  trace.hidden = true;
  logBox.appendChild(trace);

  const logNote = document.createElement("div");
  logNote.className = "eq-note";
  logBox.appendChild(logNote);

  /** Draws the measures as three tracks down depth, the way they will look in a log view.
   *
   *  A table of percentiles cannot say whether a trace has bedding in it, and bedding is the whole
   *  question. Each track is scaled to its OWN range \u2014 darkness, redness and texture are three
   *  different quantities and one shared axis would flatten two of them to a line. */
  const drawTrace = (res: CoreLogResult): void => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(trace.clientWidth * dpr));
    const h = Math.max(1, Math.round(220 * dpr));
    if (trace.width !== w || trace.height !== h) {
      trace.width = w;
      trace.height = h;
    }
    const ctx = trace.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, w, h);
    const d = res.preview_depth;
    if (d.length < 2) return;
    const pad = 18 * dpr;
    const cols = res.curves.length;
    const cw = (w - pad * (cols + 1)) / cols;
    const dmin = d[0];
    const dmax = d[d.length - 1];
    const colours = ["#5a5a5a", "#a83e2c", "#5f7350"];
    ctx.font = `${11 * dpr}px sans-serif`;
    ctx.textBaseline = "top";
    for (let k = 0; k < cols; k++) {
      const cv = res.curves[k];
      const x0 = pad + k * (cw + pad);
      const fin = cv.preview.filter((v) => Number.isFinite(v));
      if (!fin.length) continue;
      let lo = Math.min(...fin);
      let hi = Math.max(...fin);
      if (hi - lo < 1e-9) {
        lo -= 0.5;
        hi += 0.5;
      }
      ctx.strokeStyle = "rgba(128,128,128,0.35)";
      ctx.strokeRect(x0, pad, cw, h - pad * 1.6);
      ctx.fillStyle = "var(--text)";
      ctx.fillStyle = colours[k % colours.length];
      ctx.fillText(cv.name.replace("CPHOTO_", ""), x0, 2 * dpr);
      ctx.beginPath();
      for (let i = 0; i < cv.preview.length; i++) {
        const v = cv.preview[i];
        if (!Number.isFinite(v)) continue;
        const x = x0 + ((v - lo) / (hi - lo)) * cw;
        const y = pad + ((d[i] - dmin) / Math.max(1e-9, dmax - dmin)) * (h - pad * 1.6);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.strokeStyle = colours[k % colours.length];
      ctx.lineWidth = 1 * dpr;
      ctx.stroke();
    }
    ctx.fillStyle = "rgba(128,128,128,0.9)";
    ctx.fillText(`${dmin.toFixed(1)}`, 2 * dpr, pad);
    ctx.fillText(`${dmax.toFixed(1)}`, 2 * dpr, h - pad);
  };

  const describeRun = (res: CoreLogResult): string => {
    const bits = res.curves.map((c) =>
      Number.isFinite(c.correlation)
        ? `${c.name.replace("CPHOTO_", "")} ${c.correlation >= 0 ? "+" : ""}${c.correlation.toFixed(2)}`
        : `${c.name.replace("CPHOTO_", "")} \u2014`
    );
    const head =
      `${res.samples} sample(s) from ${res.photographs} photograph(s), ` +
      `${res.depth_min.toFixed(1)} to ${res.depth_max.toFixed(1)}.`;
    const agree = cmpSel.value ? ` Against ${cmpSel.value}: ${bits.join(", ")}.` : "";
    return head + agree + (res.notes.length ? " " + res.notes.join(" ") : "") +
      (res.skipped.length ? ` Left out: ${res.skipped.join("; ")}` : "");
  };

  const buildSpec = (write: boolean) => ({
    well_id: well.well_id,
    dataset: dsSel.value,
    axis: axisPick.get() as "x" | "y",
    reverse: revChk.checked,
    lanes: Number(lanePick.get()) || 1,
    compare_curve: cmpSel.value || null,
    write,
  });

  const runRead = async (write: boolean): Promise<void> => {
    readBtn.disabled = true;
    writeBtn.disabled = true;
    logNote.textContent = write ? "Saving\u2026" : "Reading\u2026";
    try {
      const res = await extractCoreLog(buildSpec(write));
      trace.hidden = false;
      drawTrace(res);
      logNote.textContent =
        (write ? `Saved ${res.written.join(", ")}. ` : "") + describeRun(res);
      if (write) {
        setStatus(`Read ${res.written.length} curve(s) off ${dsSel.value}`);
        recordProcess("Edit", `Core photo log on ${dsSel.value}: ${res.written.join(", ")}`, well.well_name);
        bumpDataVersion();
      }
      writeBtn.disabled = false;
    } catch (e) {
      logNote.textContent = String(e);
    } finally {
      readBtn.disabled = false;
    }
  };

  readBtn.addEventListener("click", () => void runRead(false));
  writeBtn.addEventListener("click", () => void runRead(true));

  stripBtn.addEventListener("click", () => {
    void (async () => {
      stripBtn.disabled = true;
      logNote.textContent = "Building…";
      try {
        const res = await buildCoreStrips({
          well_id: well.well_id,
          dataset: dsSel.value,
          axis: axisPick.get() as "x" | "y",
          reverse: revChk.checked,
          lanes: Number(lanePick.get()) || 1,
          target: stripTarget.value.trim() || null,
        });
        logNote.textContent =
          `${res.built} strip(s) in ${res.dataset}. ` +
          res.notes.join(" ") +
          (res.skipped.length ? ` Left out: ${res.skipped.join("; ")}` : "");
        setStatus(`${res.built} depth strip(s) built in ${res.dataset}`);
        recordProcess("Edit", `Depth strips from ${dsSel.value} into ${res.dataset}`, well.well_name);
        // A new picture delivery: the Wells pane, the layout editor's dataset list and any open log
        // view all need to hear about it.
        bumpDataVersion();
      } catch (e) {
        logNote.textContent = String(e);
      } finally {
        stripBtn.disabled = false;
      }
    })();
  });

  // ---- loading ------------------------------------------------------------
  const select = (id: string): void => {
    current = id;
    // The paired frame belongs to the picture being looked at, so it is dropped rather than left
    // to be shown beside the next box.
    pairPng = null;
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

  // Object URLs and the observer outlive the element unless they are dropped with it. As a modal
  // this had to watch #modal-root for its own content being detached, because openModal offers no
  // close hook; a pane is handed a real teardown and the whole mechanism goes away.
  return {
    el: wrap,
    dispose: () => {
      for (const u of urls) URL.revokeObjectURL(u);
      urls.length = 0;
      seen.disconnect();
    },
  };
}
