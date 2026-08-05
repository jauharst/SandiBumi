import { getWellImage, type ImageInfo } from "../ipc";

/**
 * A delivery as a row of pictures rather than a dropdown of filenames.
 *
 * Split out of the conditioning workspace so the measuring dialogs get the same thing: a
 * petrographer picking which plate to tune a threshold on is choosing a PICTURE, and a list of
 * names makes them open six of them to find the one they meant. Jauhar, 2026-07-31: "geologist see
 * image not text".
 *
 * **Thumbnails load only when their tile scrolls into view.** A delivery is routinely hundreds of
 * plates at about a megabyte each, and filling a strip nobody has scrolled to would pull hundreds
 * of megabytes through the bridge.
 *
 * **A plate that cannot be measured is shown GREYED with the reason on hover, never hidden.**
 * "MY-14 is there but it was never declared impregnated" is the question the user is about to ask
 * by running the tool; hiding it turns that into a delivery that silently lost a plate.
 */
export interface PlateStripHandle {
  el: HTMLElement;
  /** Rebuild from a new delivery. `disabled` returns a reason, or null when the plate is usable. */
  load(plates: ImageInfo[], disabled?: (p: ImageInfo) => string | null): void;
  /** Highlight one plate without firing the callback. */
  mark(imageId: string): void;
  /** Re-label the tiles in place — a click count, a measured fraction — without refetching a
   *  single thumbnail. Return null to leave a tile showing just its name. */
  annotate(text: (p: ImageInfo) => string | null): void;
  /** Release the object URLs and stop observing. */
  dispose(): void;
}

export function buildPlateStrip(onPick: (id: string) => void): PlateStripHandle {
  const el = document.createElement("div");
  el.className = "cond-strip";

  const urls: string[] = [];
  let plates: ImageInfo[] = [];

  const seen = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        const tile = e.target as HTMLElement;
        seen.unobserve(tile);
        const id = tile.dataset.id ?? "";
        void (async () => {
          try {
            const buf = await getWellImage(id);
            const mime = plates.find((p) => p.image_id === id)?.mime ?? "image/jpeg";
            const url = URL.createObjectURL(new Blob([buf], { type: mime }));
            urls.push(url);
            tile.style.backgroundImage = `url("${url}")`;
          } catch {
            /* a picture the viewer cannot decode still gets its tile and its label */
          }
        })();
      }
    },
    { root: el, rootMargin: "200px" }
  );

  return {
    el,
    load(next, disabled) {
      plates = next;
      el.innerHTML = "";
      for (const p of plates) {
        const tile = document.createElement("div");
        tile.className = "cond-thumb";
        tile.dataset.id = p.image_id;
        const why = disabled?.(p) ?? null;
        if (why) {
          tile.classList.add("is-blocked");
          tile.title = `${p.name} @ ${p.depth_top} — ${why}`;
        } else {
          tile.title = `${p.name} @ ${p.depth_top}`;
        }
        const label = document.createElement("span");
        label.className = "cond-thumb-label";
        label.textContent = p.name;
        tile.appendChild(label);
        // A blocked plate is still clickable: a preview of what the band WOULD claim is how the
        // user works out whether the plate is worth declaring, and refusing the click would leave
        // them with a greyed tile and no way to look at it.
        tile.addEventListener("click", () => onPick(p.image_id));
        el.appendChild(tile);
        seen.observe(tile);
      }
    },
    mark(imageId) {
      for (const tile of Array.from(el.children) as HTMLElement[]) {
        tile.classList.toggle("is-current", tile.dataset.id === imageId);
      }
    },
    annotate(text) {
      for (const tile of Array.from(el.children) as HTMLElement[]) {
        const p = plates.find((q) => q.image_id === tile.dataset.id);
        const label = tile.querySelector(".cond-thumb-label");
        if (!p || !label) continue;
        const extra = text(p);
        label.textContent = extra ? `${p.name} · ${extra}` : p.name;
      }
    },
    dispose() {
      seen.disconnect();
      for (const u of urls) URL.revokeObjectURL(u);
      urls.length = 0;
    },
  };
}
