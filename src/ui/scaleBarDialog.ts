import { getWellImage, type ImageInfo } from "../ipc";
import { setStatus } from "../state";
import { formRow, openModal } from "./modal";
import { fitCanvasBackingStore, readTheme } from "./plotCanvas";

/**
 * Calibrate a plate by dragging along its own scale bar.
 *
 * The route that makes a plate measurable when it states its scale as a BAR burned into the image
 * rather than as a field of view in the caption — which, on Jauhar's "sometimes yes, sometimes not",
 * is a good share of them.
 *
 * **The measurement is a pure ratio, and that is what makes it safe.** The bar's length is taken as
 * a FRACTION of the picture's width, so the field of view is `bar length / that fraction`. Nothing
 * in it depends on the display zoom, or on the stored copy having been resampled to a long-edge cap
 * — both lengths shrank by the same factor and the ratio did not move. This is the same property
 * that made a field of view the right thing to store rather than micrometres per pixel, and it is
 * why the answer comes out already in the form the store wants.
 *
 * **A slightly crooked drag costs almost nothing.** Off a truly horizontal bar by 5° the measured
 * length is long by 0.4%, because the error is second-order in the angle. So there is no snapping
 * and no constraint to fight — what matters far more is hitting the bar's ends, which is what
 * **Actual size** is for.
 *
 * Resolves to the field of view in micrometres, or `null` if the user closed without accepting.
 */
export function openScaleBarDialog(img: ImageInfo): Promise<number | null> {
  return new Promise((resolve) => {
    const wrap = document.createElement("div");
    let done = false;
    const finish = (v: number | null): void => {
      if (done) return;
      done = true;
      resolve(v);
      close();
    };
    const close = openModal(`Scale bar — ${img.name}`, wrap, 900);
    // openModal has no close hook, and a caller awaiting this must not be left hanging when the
    // user presses Esc or ✕. Detaching the content is the one signal that covers every route out.
    const watcher = new MutationObserver(() => {
      if (!wrap.isConnected) {
        watcher.disconnect();
        finish(null);
      }
    });
    const root = document.querySelector("#modal-root");
    if (root) watcher.observe(root, { childList: true, subtree: true });

    const intro = document.createElement("div");
    intro.className = "eq-note";
    intro.textContent =
      "Drag along the plate's own scale bar, end to end, then type what the bar says. " +
      "Switch to Actual size and scroll to the bar first — hitting its ends is what decides the accuracy.";
    wrap.appendChild(intro);

    // ---- viewer ----------------------------------------------------------
    const modeSel = document.createElement("select");
    modeSel.className = "form-control";
    for (const [v, label] of [
      ["fit", "Fit to window"],
      ["actual", "Actual size (scroll to the bar)"],
    ] as const) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      modeSel.appendChild(o);
    }
    wrap.appendChild(formRow("View", modeSel));

    const scroller = document.createElement("div");
    scroller.style.overflow = "auto";
    scroller.style.maxHeight = "420px";
    scroller.style.border = "1px solid var(--border)";
    wrap.appendChild(scroller);

    const canvas = document.createElement("canvas");
    canvas.style.display = "block";
    canvas.style.cursor = "crosshair";
    scroller.appendChild(canvas);

    const readout = document.createElement("div");
    readout.className = "eq-note";
    wrap.appendChild(readout);

    // Endpoints as fractions of the natural width/height, so they survive both view modes and any
    // window resize — the same reason the answer itself is a ratio.
    let a: { u: number; v: number } | null = null;
    let b: { u: number; v: number } | null = null;
    let bitmap: ImageBitmap | null = null;

    /** Bar length as a fraction of the picture's WIDTH. */
    const widthFraction = (): number | null => {
      if (!a || !b || !bitmap) return null;
      const dx = (b.u - a.u) * bitmap.width;
      const dy = (b.v - a.v) * bitmap.height;
      const len = Math.hypot(dx, dy);
      return len > 0 ? len / bitmap.width : null;
    };

    const layout = (): void => {
      if (!bitmap) return;
      if (modeSel.value === "actual") {
        canvas.style.width = `${bitmap.width}px`;
        canvas.style.height = `${bitmap.height}px`;
      } else {
        canvas.style.width = "100%";
        canvas.style.height = "auto";
        // A percentage width with height:auto leaves the element with no intrinsic ratio until it
        // is painted, so the aspect is set explicitly.
        canvas.style.aspectRatio = `${bitmap.width} / ${bitmap.height}`;
      }
    };

    const draw = (): void => {
      if (!bitmap) return;
      const dpr = fitCanvasBackingStore(canvas);
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      ctx.clearRect(0, 0, w, h);
      ctx.drawImage(bitmap, 0, 0, w, h);

      if (a && b) {
        const theme = readTheme(canvas);
        const p = (q: { u: number; v: number }): [number, number] => [q.u * w, q.v * h];
        const [x1, y1] = p(a);
        const [x2, y2] = p(b);
        // Drawn twice: a dark casing under a bright line, so it reads on a pale quartz grain and
        // on dark epoxy alike. A single colour disappears against one or the other.
        for (const [colour, width] of [["rgba(0,0,0,0.75)", 5], [theme.accent || "#ff2d2d", 2]] as const) {
          ctx.strokeStyle = colour;
          ctx.lineWidth = width;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          ctx.stroke();
          // End caps, so an endpoint that landed short of the bar is visible.
          const nx = (y2 - y1) / Math.hypot(x2 - x1, y2 - y1 || 1);
          const ny = -(x2 - x1) / Math.hypot(x2 - x1, y2 - y1 || 1);
          ctx.beginPath();
          ctx.moveTo(x1 - nx * 7, y1 - ny * 7);
          ctx.lineTo(x1 + nx * 7, y1 + ny * 7);
          ctx.moveTo(x2 - nx * 7, y2 - ny * 7);
          ctx.lineTo(x2 + nx * 7, y2 + ny * 7);
          ctx.stroke();
        }
      }
      report();
    };

    const barIn = document.createElement("input");
    barIn.className = "form-control";
    barIn.type = "number";
    barIn.step = "1";
    barIn.min = "0";
    barIn.placeholder = "e.g. 500";
    const unitSel = document.createElement("select");
    unitSel.className = "form-control";
    for (const [v, label] of [
      ["um", "µm"],
      ["mm", "mm"],
    ] as const) {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = label;
      unitSel.appendChild(o);
    }
    const barRow = document.createElement("div");
    barRow.style.display = "flex";
    barRow.style.gap = "8px";
    barRow.appendChild(barIn);
    barRow.appendChild(unitSel);
    wrap.appendChild(formRow("The bar reads", barRow, "Whatever is printed beside it on the plate."));

    /** The field of view the drag and the typed length imply, in micrometres. */
    const fovUm = (): number | null => {
      const f = widthFraction();
      const n = Number(barIn.value);
      if (f == null || !Number.isFinite(n) || n <= 0) return null;
      const um = unitSel.value === "mm" ? n * 1000 : n;
      return um / f;
    };

    const useBtn = document.createElement("button");
    useBtn.className = "btn btn-accent";
    useBtn.textContent = "Use this scale";
    useBtn.disabled = true;

    function report(): void {
      const f = widthFraction();
      if (f == null) {
        readout.textContent = "Drag along the scale bar.";
        useBtn.disabled = true;
        return;
      }
      const fov = fovUm();
      const px = bitmap ? (f * bitmap.width).toFixed(0) : "?";
      readout.textContent =
        fov == null
          ? `Bar spans ${(f * 100).toFixed(1)}% of the plate (${px} px). Now type what it reads.`
          : `Bar spans ${(f * 100).toFixed(1)}% of the plate (${px} px) → field of view ` +
            `${(fov / 1000).toFixed(3)} mm, ${(fov / (bitmap?.width ?? 1)).toFixed(3)} µm/px on this copy.`;
      useBtn.disabled = fov == null;
    }

    barIn.addEventListener("input", report);
    unitSel.addEventListener("change", report);

    // ---- dragging --------------------------------------------------------
    const at = (e: PointerEvent): { u: number; v: number } => {
      const r = canvas.getBoundingClientRect();
      return {
        u: Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)),
        v: Math.min(1, Math.max(0, (e.clientY - r.top) / r.height)),
      };
    };
    let dragging = false;
    canvas.addEventListener("pointerdown", (e) => {
      dragging = true;
      canvas.setPointerCapture(e.pointerId);
      a = at(e);
      b = a;
      draw();
    });
    canvas.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      b = at(e);
      draw();
    });
    canvas.addEventListener("pointerup", () => {
      dragging = false;
      draw();
    });

    modeSel.addEventListener("change", () => {
      layout();
      draw();
    });

    const applyAll = document.createElement("input");
    applyAll.type = "checkbox";
    const applyLabel = document.createElement("label");
    applyLabel.appendChild(applyAll);
    applyLabel.appendChild(
      document.createTextNode(" Apply to every plate of this delivery (same microscope, same magnification)")
    );
    applyLabel.style.display = "block";
    wrap.appendChild(applyLabel);
    // Read by the caller after the promise resolves — offered because a delivery is usually one
    // sitting at one magnification, but never assumed, because "sometimes" is the whole reason
    // these fields are per plate.
    (openScaleBarDialog as unknown as { lastApplyAll: boolean }).lastApplyAll = false;

    useBtn.addEventListener("click", () => {
      const fov = fovUm();
      if (fov == null) return;
      (openScaleBarDialog as unknown as { lastApplyAll: boolean }).lastApplyAll = applyAll.checked;
      finish(fov);
    });
    wrap.appendChild(useBtn);

    void (async () => {
      try {
        const buf = await getWellImage(img.image_id);
        bitmap = await createImageBitmap(new Blob([buf], { type: img.mime }));
        layout();
        // Synchronous first paint: requestAnimationFrame does not fire in a tab that is not
        // compositing, which would leave this blank in a background window.
        draw();
      } catch (e) {
        readout.textContent = `Could not open the picture: ${e}`;
        setStatus(String(e));
      }
    })();
  });
}

/** Whether the user asked for the last accepted scale to be applied to the whole delivery. */
export function scaleBarAppliedToAll(): boolean {
  return (openScaleBarDialog as unknown as { lastApplyAll?: boolean }).lastApplyAll === true;
}
