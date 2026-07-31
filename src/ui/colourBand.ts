import type { PoreColorBand } from "../ipc";

/**
 * The colour band, as a colour rather than as four numbers.
 *
 * A threshold on hue cannot be judged from a number — 205° means nothing to anyone, and neither
 * does 0.15 of saturation. What a petrographer actually knows is what the epoxy in front of them
 * looks like. So the band is a **hue wheel laid out flat with two draggable ends**, the saturation
 * and brightness floors are sliders whose TRACKS carry the gradient they move along, and the whole
 * thing carries a live swatch of what the band currently accepts.
 *
 * This is the `coreConditionDialog` rule applied to the measurement side: the controls are the
 * thing being controlled wherever they can be. Jauhar, 2026-07-31: "geologist see image not text".
 *
 * **A wrapped band is drawn as two arcs, not refused.** Red sits across 0°, and a band from 340° to
 * 20° is a perfectly ordinary thing to want — the runner's `in_band` already reads it that way, so
 * a control that could not express it would be the one lying.
 *
 * The numbers are still shown, and still typable: a band that came off somebody else's run has to
 * be enterable, and a value that has to be dragged to be set cannot be written down.
 */
export interface ColourBandHandle {
  el: HTMLElement;
  get(): PoreColorBand;
  /** Centres the band on a colour clicked in the picture, keeping its current width. */
  pickFrom(rgb: [number, number, number]): void;
  set(b: PoreColorBand): void;
}

/** Hue, saturation, value from 0-255 RGB — the same conversion the runner does, so a colour picked
 *  off the preview lands where the measurement will put it. */
export function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
  const rr = r / 255;
  const gg = g / 255;
  const bb = b / 255;
  const mx = Math.max(rr, gg, bb);
  const mn = Math.min(rr, gg, bb);
  const d = mx - mn;
  let h = 0;
  if (d > 1e-9) {
    if (mx === rr) h = 60 * (((gg - bb) / d) % 6);
    else if (mx === gg) h = 60 * ((bb - rr) / d + 2);
    else h = 60 * ((rr - gg) / d + 4);
  }
  if (h < 0) h += 360;
  return [h, mx <= 1e-9 ? 0 : d / mx, mx];
}

/** CSS colour for a hue at full saturation and brightness — used to paint the wheel. */
const hueCss = (h: number, s = 1, v = 1): string => `hsl(${h.toFixed(1)} ${(s * 100).toFixed(0)}% ${((1 - s / 2) * v * 100).toFixed(0)}%)`;

/** Is `h` inside the band? Wrapped bands read as two arcs — the runner's own rule. */
export function inBand(h: number, from: number, to: number): boolean {
  return from <= to ? h >= from && h <= to : h >= from || h <= to;
}

export function buildColourBand(initial: PoreColorBand, onChange: () => void): ColourBandHandle {
  const el = document.createElement("div");
  el.className = "cband";

  let band: PoreColorBand = { ...initial };

  // ---- the wheel, flat -----------------------------------------------------
  //
  // Painted on a canvas rather than assembled from CSS gradients and overlay panels. A band that
  // WRAPS through red is two arcs, and dimming everything outside two arcs with layered divs is
  // three special cases that each have to be got right; one degree per column is none.
  const wheel = document.createElement("div");
  wheel.className = "cband-wheel";
  const wheelCv = document.createElement("canvas");
  wheelCv.className = "cband-canvas";
  const handleLo = document.createElement("div");
  const handleHi = document.createElement("div");
  handleLo.className = "cband-handle";
  handleHi.className = "cband-handle";
  handleLo.title = "The blue end of the band, in degrees round the colour wheel";
  handleHi.title = "The other end. Drag it past the first and the band wraps through red, which is legal.";
  wheel.append(wheelCv, handleLo, handleHi);
  el.appendChild(wheel);

  /** One column per degree: bright inside the band, dimmed outside it. The crop-rectangle idiom —
   *  what is KEPT is the bright part. */
  const paintWheel = (): void => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(wheel.clientWidth * dpr));
    const h = Math.max(1, Math.round(wheel.clientHeight * dpr));
    if (wheelCv.width !== w || wheelCv.height !== h) {
      wheelCv.width = w;
      wheelCv.height = h;
    }
    const ctx = wheelCv.getContext("2d");
    if (!ctx) return;
    for (let x = 0; x < w; x++) {
      const hue = (x / w) * 360;
      const on = inBand(hue, band.hue_lo, band.hue_hi);
      ctx.fillStyle = hueCss(hue, on ? 1 : 0.35, on ? 1 : 0.42);
      ctx.fillRect(x, 0, 1, h);
    }
  };

  // ---- the two floors ------------------------------------------------------
  const satIn = document.createElement("input");
  const valIn = document.createElement("input");
  const satVal = document.createElement("span");
  const valVal = document.createElement("span");
  const floorRow = (label: string, input: HTMLInputElement, out: HTMLElement, hint: string): HTMLElement => {
    const row = document.createElement("div");
    row.className = "cond-row";
    const lab = document.createElement("label");
    lab.className = "form-label";
    lab.textContent = label;
    lab.title = hint;
    input.type = "range";
    input.className = "cond-slider";
    input.min = "0";
    input.max = "1";
    input.step = "0.01";
    input.title = hint;
    out.className = "cond-val";
    const spacer = document.createElement("span");
    row.append(lab, input, out, spacer);
    return row;
  };
  el.appendChild(
    floorRow(
      "At least this vivid",
      satIn,
      satVal,
      "Rejects greys and near-whites, whose hue is meaningless. Raise it to drop pale, washed-out blue that is really a grain edge."
    )
  );
  el.appendChild(
    floorRow(
      "At least this bright",
      valIn,
      valVal,
      "Rejects near-black — cracks, plucked holes, and the shadow at a plate edge."
    )
  );

  // ---- the numbers, still there --------------------------------------------
  const nums = document.createElement("div");
  nums.className = "cband-nums";
  const mkNum = (label: string, step: number): HTMLInputElement => {
    const w = document.createElement("label");
    w.className = "cband-num";
    const t = document.createElement("span");
    t.textContent = label;
    const i = document.createElement("input");
    i.className = "form-control";
    i.type = "number";
    i.step = String(step);
    w.append(t, i);
    nums.appendChild(w);
    return i;
  };
  const hueLoNum = mkNum("from °", 1);
  const hueHiNum = mkNum("to °", 1);
  const swatch = document.createElement("span");
  swatch.className = "cband-swatch";
  swatch.title = "The middle of the band at the vividness and brightness you have set — roughly what a pixel has to look like to be counted.";
  nums.appendChild(swatch);
  el.appendChild(nums);

  // ---- painting ------------------------------------------------------------
  const frac = (h: number): number => Math.min(1, Math.max(0, h / 360));
  const paint = (): void => {
    const lo = band.hue_lo;
    const hi = band.hue_hi;
    handleLo.style.left = `${frac(lo) * 100}%`;
    handleHi.style.left = `${frac(hi) * 100}%`;
    handleLo.style.background = hueCss(lo);
    handleHi.style.background = hueCss(hi);
    paintWheel();
    // The middle of the band, the short way round — so a wrapped band's swatch is the red it is
    // actually selecting rather than the cyan on the far side.
    const span = lo <= hi ? hi - lo : 360 - lo + hi;
    const mid = (lo + span / 2) % 360;
    swatch.style.background = hueCss(mid, Math.max(0.25, band.sat_min), Math.max(0.35, band.val_min));
    hueLoNum.value = String(Math.round(lo));
    hueHiNum.value = String(Math.round(hi));
    satIn.value = String(band.sat_min);
    valIn.value = String(band.val_min);
    satVal.textContent = band.sat_min.toFixed(2);
    valVal.textContent = band.val_min.toFixed(2);
    // The floors' tracks show what they are moving through, at the band's own hue.
    satIn.style.background = `linear-gradient(to right, ${hueCss(mid, 0.02, 0.85)}, ${hueCss(mid, 1, 0.85)})`;
    valIn.style.background = `linear-gradient(to right, #000, ${hueCss(mid, Math.max(0.25, band.sat_min), 1)})`;
  };

  // ---- dragging ------------------------------------------------------------
  let dragging: "lo" | "hi" | null = null;
  const hueAt = (ev: PointerEvent): number => {
    const r = wheel.getBoundingClientRect();
    return Math.min(360, Math.max(0, ((ev.clientX - r.left) / Math.max(1, r.width)) * 360));
  };
  const startDrag = (which: "lo" | "hi") => (ev: PointerEvent) => {
    dragging = which;
    wheel.setPointerCapture(ev.pointerId);
    ev.preventDefault();
    ev.stopPropagation();
  };
  handleLo.addEventListener("pointerdown", startDrag("lo"));
  handleHi.addEventListener("pointerdown", startDrag("hi"));
  wheel.addEventListener("pointerdown", (ev) => {
    if (dragging) return;
    // A press on the wheel itself grabs the NEARER end — the whole band is then draggable without
    // hunting for a 12-pixel handle.
    const h = hueAt(ev);
    const dl = Math.abs(h - band.hue_lo);
    const dh = Math.abs(h - band.hue_hi);
    dragging = dl <= dh ? "lo" : "hi";
    wheel.setPointerCapture(ev.pointerId);
    if (dragging === "lo") band.hue_lo = h;
    else band.hue_hi = h;
    paint();
    onChange();
  });
  wheel.addEventListener("pointermove", (ev) => {
    if (!dragging) return;
    const h = hueAt(ev);
    if (dragging === "lo") band.hue_lo = h;
    else band.hue_hi = h;
    paint();
  });
  for (const e of ["pointerup", "pointercancel"]) {
    wheel.addEventListener(e, () => {
      if (!dragging) return;
      dragging = null;
      onChange();
    });
  }

  const num = (input: HTMLInputElement, apply: (v: number) => void): void => {
    input.addEventListener("change", () => {
      const v = Number(input.value);
      if (!Number.isFinite(v)) return;
      apply(v);
      paint();
      onChange();
    });
  };
  num(hueLoNum, (v) => (band.hue_lo = ((v % 360) + 360) % 360));
  num(hueHiNum, (v) => (band.hue_hi = ((v % 360) + 360) % 360));
  for (const [input, key] of [
    [satIn, "sat_min"],
    [valIn, "val_min"],
  ] as [HTMLInputElement, "sat_min" | "val_min"][]) {
    input.addEventListener("input", () => {
      band[key] = Number(input.value);
      paint();
    });
    input.addEventListener("change", () => onChange());
  }

  paint();

  return {
    el,
    get: () => ({ ...band }),
    set: (b) => {
      band = { ...b };
      paint();
    },
    pickFrom: ([r, g, b]) => {
      const [h, s, v] = rgbToHsv(r, g, b);
      // The band keeps its WIDTH and moves its centre. A click says "this colour is pore", not
      // "this is the only colour that is pore" — and a band collapsed onto one hue would find
      // almost nothing, which reads as a broken tool rather than as a narrow band.
      const span = band.hue_lo <= band.hue_hi ? band.hue_hi - band.hue_lo : 360 - band.hue_lo + band.hue_hi;
      const half = Math.max(5, Math.min(90, span / 2));
      band.hue_lo = ((h - half) % 360 + 360) % 360;
      band.hue_hi = (h + half) % 360;
      // The floors drop to just under what was clicked, so the clicked pixel is inside the band it
      // just defined. Raising them is a decision; excluding the very pixel the user pointed at is
      // not a decision anyone made.
      band.sat_min = Math.min(band.sat_min, Math.max(0.02, s * 0.7));
      band.val_min = Math.min(band.val_min, Math.max(0.02, v * 0.5));
      paint();
      onChange();
    },
  };
}
