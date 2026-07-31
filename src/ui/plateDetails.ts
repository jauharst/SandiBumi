import { formRow } from "./modal";

/** How big the rock in a plate is, and how the section was prepared. */
export interface PlateDetails {
  /** Width of the WHOLE picture in micrometres. null = not declared. */
  fov_um: number | null;
  /** "" = unknown, "blue_epoxy", "plain". */
  prepared: string;
  /** As the laboratory report names it; "" = none or not stated. */
  stain: string;
}

/**
 * The scale-and-preparation control, shared by the image import wizard and the plate editor.
 *
 * One control rather than two copies, for the same reason `followCore.ts` is one: it is the same
 * declaration, and two copies is two places for the wording to drift.
 *
 * **Scale is entered as a FIELD OF VIEW WIDTH, not as micrometres per pixel.** The stored copy of
 * a plate is resampled to a long-edge cap, so a µm/px belongs to whichever copy it was measured
 * on and nothing in the number says which — while "this picture is 2.5 mm across" is true of every
 * copy of it. µm/px for any copy is then `fov_um / that copy's pixel width`, which is what the
 * readout shows. It is also the form a petrography caption already states.
 *
 * **Everything defaults to absent, and absent is a real answer** (Jauhar, 2026-07-31: "sometimes
 * yes, sometimes not" for the scale, "sometimes stained and epoxy, sometimes not" for the
 * preparation). A plate with no declared scale is one nothing dimensional may run on; a section
 * whose preparation is unknown is one a blue-epoxy pore rule must refuse rather than guess at,
 * because that rule returns a plausible porosity on an unimpregnated section instead of failing.
 */
export function buildPlateDetails(opts?: {
  /** Pixel width used for the µm/px readout. Omit when the plates differ — no readout is shown. */
  pixelWidth?: number | null;
  /** Wording for the delivery-wide case. */
  scaleHint?: string;
}): {
  el: HTMLElement;
  get(): PlateDetails;
  set(d: PlateDetails): void;
} {
  const el = document.createElement("div");

  const fovIn = document.createElement("input");
  fovIn.className = "form-control";
  fovIn.type = "number";
  fovIn.step = "0.01";
  fovIn.min = "0";
  fovIn.placeholder = "not stated";

  const scaleWrap = document.createElement("div");
  scaleWrap.style.display = "flex";
  scaleWrap.style.gap = "8px";
  scaleWrap.style.alignItems = "center";
  scaleWrap.appendChild(fovIn);

  const derived = document.createElement("span");
  derived.className = "eq-note";
  derived.style.whiteSpace = "nowrap";
  scaleWrap.appendChild(derived);

  const showDerived = (): void => {
    const mm = Number(fovIn.value);
    const px = opts?.pixelWidth ?? 0;
    if (!Number.isFinite(mm) || mm <= 0) {
      derived.textContent = "";
      return;
    }
    if (!px) {
      // Without a pixel width there is no ratio to show. Showing one anyway would mean picking a
      // plate's width to stand for all of them, which is the ambiguity this entry form avoids.
      derived.textContent = "";
      return;
    }
    derived.textContent = `= ${((mm * 1000) / px).toFixed(3)} µm/px`;
  };
  fovIn.addEventListener("input", showDerived);

  el.appendChild(
    formRow(
      "Field of view width (mm)",
      scaleWrap,
      opts?.scaleHint ??
        "How wide the whole picture is. Leave blank if the plate does not state it — grain and pore " +
          "size cannot be measured without it, and a guess would be a microscope setting nobody used."
    )
  );

  const prepSel = document.createElement("select");
  prepSel.className = "form-control";
  for (const [v, label] of [
    ["", "Unknown"],
    ["blue_epoxy", "Blue-dyed epoxy"],
    ["plain", "Not impregnated"],
  ] as const) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    prepSel.appendChild(o);
  }
  el.appendChild(
    formRow(
      "Impregnation",
      prepSel,
      "Blue epoxy separates pore from solid by colour. Left unknown, the pore measurement refuses " +
        "the plate rather than returning a porosity assembled from blue-ish grains."
    )
  );

  const stainIn = document.createElement("input");
  stainIn.className = "form-control";
  stainIn.type = "text";
  stainIn.placeholder = "none";
  el.appendChild(
    formRow(
      "Stain",
      stainIn,
      "As your laboratory report names it — the protocol is their fact, not something this app " +
        "should offer a menu of."
    )
  );

  return {
    el,
    get(): PlateDetails {
      const mm = Number(fovIn.value);
      return {
        fov_um: fovIn.value.trim() && Number.isFinite(mm) && mm > 0 ? mm * 1000 : null,
        prepared: prepSel.value,
        stain: stainIn.value.trim(),
      };
    },
    set(d: PlateDetails): void {
      fovIn.value = d.fov_um != null ? String(d.fov_um / 1000) : "";
      prepSel.value = d.prepared || "";
      stainIn.value = d.stain || "";
      showDerived();
    },
  };
}
