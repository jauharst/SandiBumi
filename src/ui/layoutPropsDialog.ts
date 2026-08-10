import type { CurveStyle, Layout, Track } from "../ipc";
import { availableTrackSets, hasTrackCurve, trackCurveKey } from "../trackCurveRequest";
import { openModal } from "./modal";

/** One point series the loaded well actually carries — a core plug property, or an item of
 *  a point dataset — used to seed and suggest in the point-track editor. */
export interface PointSuggestion {
  source: "core" | "aux";
  dataset?: string;
  item: string;
}

export interface CurveSuggestion {
  curve_name: string;
  set_name?: string | null;
}

/** Layout Properties dialog (per the standard-layout prototype): a track list with
 *  insert/delete/duplicate/reorder on the left, and the selected track's settings +
 *  curve style table (color, scale, fill shading) on the right. Operates on a deep
 *  clone; the caller receives the edited layout on Apply/OK. */
export function openLayoutPropsDialog(
  layout: Layout,
  availableCurves: CurveSuggestion[],
  onApply: (edited: Layout) => void,
  /** What the loaded well actually carries as measured samples, for the point-track
   *  suggestion lists. Optional so callers that only edit curve tracks need not gather it. */
  availablePoints: PointSuggestion[] = [],
  /** Array-log curve names the loaded well carries (MC_PHIE_REAL, NMR T2 …), for the
   *  array-track picker. Optional for the same reason as `availablePoints`. */
  availableArrays: string[] = [],
  /** Image datasets the loaded well carries (THIN SECTION, CORE PHOTO …), for the
   *  image-track picker. Optional for the same reason as `availablePoints`. */
  availableImages: string[] = [],
): void {
  const working: Layout = structuredClone(layout);
  let selected = 0;

  const content = document.createElement("div");
  content.className = "lp-body";

  const col = document.createElement("div");
  col.className = "lp-col";
  const colHead = document.createElement("div");
  colHead.className = "lp-col-head";
  const trackList = document.createElement("div");
  trackList.className = "lp-list";
  col.appendChild(colHead);
  col.appendChild(trackList);

  const detail = document.createElement("div");
  detail.className = "lp-detail";

  content.appendChild(col);
  content.appendChild(detail);

  const foot = document.createElement("div");
  foot.className = "lp-foot";

  // Shared datalist of known curve names for the curve-name inputs.
  const datalist = document.createElement("datalist");
  datalist.id = `lp-curves-${Date.now().toString(36)}`;
  for (const name of new Set(availableCurves.map((curve) => curve.curve_name))) {
    const opt = document.createElement("option");
    opt.value = name;
    datalist.appendChild(opt);
  }
  content.appendChild(datalist);

  // Suggestion lists for the point-track editor, built from what the well actually carries:
  // core plug properties, aux dataset names, and aux item names.
  const suggestList = (suffix: string, values: Iterable<string>): void => {
    const dl = document.createElement("datalist");
    dl.id = `${datalist.id}-${suffix}`;
    for (const v of new Set(values)) {
      const opt = document.createElement("option");
      opt.value = v;
      dl.appendChild(opt);
    }
    content.appendChild(dl);
  };
  suggestList("core", availablePoints.filter((p) => p.source === "core").map((p) => p.item));
  suggestList("item", availablePoints.filter((p) => p.source === "aux").map((p) => p.item));
  suggestList("ds", availablePoints.flatMap((p) => (p.dataset ? [p.dataset] : [])));
  suggestList("array", availableArrays);
  suggestList("imgds", availableImages);

  const iconBtn = (label: string, title: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.className = "lp-iconbtn";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };

  const newTrack = (): Track => ({
    title: uniqueTitle("New Track"),
    width_weight: 1,
    scale_type: "linear",
    kind: "curves",
    curves: [],
  });

  // `except` lets the rename path dedupe against OTHER tracks while leaving the track's own
  // current name alone (re-typing your own name is a no-op, not "name 2"). Insert/Duplicate
  // pass no `except` and behave exactly as before.
  function uniqueTitle(base: string, except?: Track): string {
    let title = base;
    let i = 2;
    while (working.tracks.some((t) => t !== except && t.title === title)) title = `${base} ${i++}`;
    return title;
  }

  colHead.appendChild(
    iconBtn("＋", "Insert track after the selected one", () => {
      working.tracks.splice(selected + 1, 0, newTrack());
      selected = Math.min(selected + 1, working.tracks.length - 1);
      renderAll();
    }),
  );
  colHead.appendChild(
    iconBtn("✕", "Delete the selected track", () => {
      if (working.tracks.length <= 1) return;
      working.tracks.splice(selected, 1);
      selected = Math.max(0, selected - 1);
      renderAll();
    }),
  );
  colHead.appendChild(
    iconBtn("⧉", "Duplicate the selected track", () => {
      const copy = structuredClone(working.tracks[selected]);
      copy.title = uniqueTitle(copy.title);
      working.tracks.splice(selected + 1, 0, copy);
      selected += 1;
      renderAll();
    }),
  );
  const spacer = document.createElement("div");
  spacer.style.flex = "1";
  colHead.appendChild(spacer);
  colHead.appendChild(
    iconBtn("↑", "Move the selected track left/up", () => {
      if (selected === 0) return;
      const [t] = working.tracks.splice(selected, 1);
      working.tracks.splice(selected - 1, 0, t);
      selected -= 1;
      renderAll();
    }),
  );
  colHead.appendChild(
    iconBtn("↓", "Move the selected track right/down", () => {
      if (selected >= working.tracks.length - 1) return;
      const [t] = working.tracks.splice(selected, 1);
      working.tracks.splice(selected + 1, 0, t);
      selected += 1;
      renderAll();
    }),
  );

  function renderTrackList(): void {
    trackList.innerHTML = "";
    working.tracks.forEach((t, i) => {
      const item = document.createElement("div");
      item.className = "lp-item" + (i === selected ? " selected" : "");
      const name = document.createElement("span");
      name.textContent = t.title;
      const dim = document.createElement("span");
      dim.className = "lp-item-dim";
      dim.textContent = `${t.curves.length} curve${t.curves.length === 1 ? "" : "s"}`;
      item.appendChild(name);
      item.appendChild(dim);
      item.addEventListener("click", () => {
        selected = i;
        renderAll();
      });
      trackList.appendChild(item);
    });
  }

  function field(label: string, control: HTMLElement): HTMLElement {
    const wrap = document.createElement("div");
    wrap.className = "lp-field";
    const lab = document.createElement("label");
    lab.textContent = label;
    wrap.appendChild(lab);
    wrap.appendChild(control);
    return wrap;
  }

  function textInput(value: string, onChange: (v: string) => void): HTMLInputElement {
    const input = document.createElement("input");
    input.type = "text";
    input.value = value;
    input.addEventListener("change", () => onChange(input.value));
    return input;
  }

  function numInput(value: number, onChange: (v: number) => void, step = "any"): HTMLInputElement {
    const input = document.createElement("input");
    input.type = "number";
    input.step = step;
    input.value = String(value);
    input.addEventListener("change", () => {
      const v = parseFloat(input.value);
      if (!Number.isNaN(v)) onChange(v);
    });
    return input;
  }

  function colorInput(value: string, onChange: (v: string) => void): HTMLInputElement {
    const input = document.createElement("input");
    input.type = "color";
    input.value = value;
    input.addEventListener("change", () => onChange(input.value));
    return input;
  }

  function selectInput<T extends string>(value: T, options: [T, string][], onChange: (v: T) => void): HTMLSelectElement {
    const sel = document.createElement("select");
    for (const [v, label] of options) {
      const opt = document.createElement("option");
      opt.value = v;
      opt.textContent = label;
      sel.appendChild(opt);
    }
    sel.value = value;
    sel.addEventListener("change", () => onChange(sel.value as T));
    return sel;
  }

  function renderDetail(): void {
    detail.innerHTML = "";
    const track = working.tracks[selected];
    if (!track) return;

    const grid = document.createElement("div");
    grid.className = "lp-fieldgrid";
    grid.appendChild(
      field(
        "Track title",
        textInput(track.title, (v) => {
          // Track title is the primary key for weights, cursor hit-testing, core overlay and
          // drag-drop — a duplicate collapses two tracks into one. Suffix a colliding rename.
          track.title = uniqueTitle(v || track.title, track);
          renderTrackList();
        }),
      ),
    );
    grid.appendChild(
      field(
        "Track type",
        selectInput(
          track.kind ?? "curves",
          [
            ["curves", "Curves"],
            ["point_data", "Point data"],
            ["array_log", "Array log"],
            ["image", "Images"],
            ["well_diagram", "Well diagram"],
          ],
          (v) => {
            track.kind = v;
            renderDetail();
          },
        ),
      ),
    );
    grid.appendChild(
      field(
        "Value scale",
        selectInput(track.scale_type, [["linear", "Linear"], ["log", "Logarithmic"]], (v) => {
          track.scale_type = v;
        }),
      ),
    );
    grid.appendChild(
      field(
        "Width weight",
        numInput(track.width_weight, (v) => {
          track.width_weight = Math.max(0.2, v);
        }),
      ),
    );
    detail.appendChild(grid);

    // A well-diagram track ignores curves: it draws casing / shoe / tubing / perforations from
    // the well's COMPLETION + PERFORATION aux datasets instead.
    if ((track.kind ?? "curves") === "well_diagram") {
      const note = document.createElement("div");
      note.className = "lp-section-title";
      note.style.fontWeight = "400";
      note.textContent =
        "Draws casing / shoe / tubing / perforations from the well's COMPLETION and PERFORATION datasets (Data ▸ Import aux data). No curves needed.";
      detail.appendChild(note);
      return;
    }

    // A point-data track draws measured samples (core plugs, XRD, CEC, oil show, core
    // extras) rather than a continuous log, so it has its own style block, not `curves`.
    if ((track.kind ?? "curves") === "point_data") {
      renderPointSection(track);
      return;
    }
    if ((track.kind ?? "curves") === "array_log") {
      renderArraySection(track);
      return;
    }
    // An image track draws depth-registered pictures — thin sections, core photographs —
    // which have no value axis at all, so it too has its own style block.
    if ((track.kind ?? "curves") === "image") {
      renderImageSection(track);
      return;
    }

    const sectionTitle = document.createElement("div");
    sectionTitle.className = "lp-section-title";
    sectionTitle.textContent = "Curves";
    detail.appendChild(sectionTitle);

    const table = document.createElement("table");
    table.className = "lp-curvetable";
    table.innerHTML = `<thead><tr>
      <th>Curve</th><th>Set</th><th>Style</th><th>Color</th><th>Min</th><th>Max</th>
      <th>Fill</th><th>To curve</th><th>Shading</th><th>Opacity</th><th></th>
    </tr></thead>`;
    const tbody = document.createElement("tbody");

    track.curves.forEach((c, ci) => {
      const tr = document.createElement("tr");
      const cell = (el: HTMLElement, cls = ""): HTMLTableCellElement => {
        const td = document.createElement("td");
        if (cls) td.className = cls;
        td.appendChild(el);
        tr.appendChild(td);
        return td;
      };
      const nameInput = textInput(c.curve_name, (v) => {
        c.curve_name = v.trim().toUpperCase();
        nameInput.value = c.curve_name;
        if (c.set_name && !hasTrackCurve(availableCurves, c)) c.set_name = undefined;
        // A mnemonic change changes the valid set list. Rebuild the row so a stale source
        // cannot remain selected for a curve that source does not carry.
        renderDetail();
      });
      nameInput.setAttribute("list", datalist.id);
      cell(nameInput);
      const availableSets: [string, string][] = [
        ["", "Current / resolved"],
        ...availableTrackSets(availableCurves, c.curve_name, c.set_name).map(
          (set): [string, string] => [set, set],
        ),
      ];
      cell(
        selectInput(c.set_name ?? "", availableSets, (v) => {
          c.set_name = v || undefined;
        }),
      );
      cell(
        selectInput(c.draw_style ?? "line", [["line", "Continuous"], ["step", "Blocky"]], (v) => {
          c.draw_style = v;
        }),
      );
      cell(colorInput(c.color, (v) => (c.color = v)), "lp-tiny");
      cell(numInput(c.min, (v) => (c.min = v)), "lp-num");
      cell(numInput(c.max, (v) => (c.max = v)), "lp-num");
      const isCrossover = c.fill === "curve";
      cell(
        selectInput(
          c.fill ?? "none",
          [
            ["none", "None"],
            ["left", "To left edge"],
            ["right", "To right edge"],
            ["curve", "Crossover to curve"],
            ["blocks", "Facies blocks"],
          ],
          (v) => {
            c.fill = v;
            // Crossover needs a reference curve and a second colour, so the row's controls
            // change shape — rebuild it rather than leaving dead inputs behind. Seed the two
            // sides with the two curves' OWN colours: that reads immediately (each side is
            // tinted toward whichever curve is out in front) and invents no convention the
            // user has not chosen. Both sides sharing one colour would look like an edge
            // fill and lose the whole point of the display.
            if (v === "curve") {
              const other = track.curves.find((o) => o !== c);
              if (!c.fill_to) c.fill_to = other?.curve_name ?? "";
              if (!c.fill_color) c.fill_color = c.color;
              if (!c.fill_color2) c.fill_color2 = other?.color ?? c.color;
            }
            renderDetail();
          },
        ),
      );
      // Reference curve for crossover shading: another curve in THIS track, because it is
      // positioned with its own min/max. Suggest only siblings, and say so when there are none.
      const toInput = textInput(c.fill_to ?? "", (v) => {
        c.fill_to = v.trim().toUpperCase();
        toInput.value = c.fill_to;
      });
      toInput.disabled = !isCrossover;
      const toWrap = document.createElement("div");
      toWrap.appendChild(toInput);
      if (isCrossover) {
        const siblings = document.createElement("datalist");
        siblings.id = `${datalist.id}-sib-${ci}`;
        for (const o of track.curves) {
          if (o === c) continue;
          const opt = document.createElement("option");
          opt.value = o.curve_name;
          siblings.appendChild(opt);
        }
        toInput.setAttribute("list", siblings.id);
        toInput.title = "Shade between this curve and another curve in the same track";
        toWrap.appendChild(siblings);
      }
      cell(toWrap);
      // One swatch for edge shading; two for crossover — left-of and right-of the reference.
      const shade = document.createElement("div");
      shade.className = "lp-shade";
      shade.appendChild(colorInput(c.fill_color ?? c.color, (v) => (c.fill_color = v)));
      if (isCrossover) {
        const left = shade.firstElementChild as HTMLInputElement;
        left.title = "Where this curve reads LEFT of the reference curve";
        const right = colorInput(c.fill_color2 ?? c.fill_color ?? c.color, (v) => (c.fill_color2 = v));
        right.title = "Where this curve reads RIGHT of the reference curve";
        shade.appendChild(right);
      }
      cell(shade, "lp-tiny");
      const opacity = numInput(
        c.fill_opacity ?? (isCrossover ? 0.3 : 0.25),
        (v) => (c.fill_opacity = Math.max(0, Math.min(1, v))),
        "0.05",
      );
      cell(opacity, "lp-num");
      cell(
        iconBtn("✕", "Remove this curve", () => {
          track.curves.splice(ci, 1);
          renderAll();
        }),
        "lp-tiny",
      );
      tbody.appendChild(tr);
    });
    table.appendChild(tbody);
    detail.appendChild(table);

    const addBtn = document.createElement("button");
    addBtn.className = "lp-btn";
    addBtn.textContent = "＋ Add curve";
    addBtn.addEventListener("click", () => {
      // Seed the persisted default from the LIVE theme accent (hex-validated — this value
      // feeds <input type=color> and is stored), so new curves match the active palette
      // instead of always coming out light-theme terracotta.
      const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim();
      const suggestion = availableCurves.find(
        (candidate) => !track.curves.some((curve) => trackCurveKey(curve) === trackCurveKey(candidate)),
      );
      const style: CurveStyle = {
        curve_name: suggestion?.curve_name ?? "GR",
        set_name: suggestion?.set_name ?? undefined,
        color: /^#[0-9a-fA-F]{3}(?:[0-9a-fA-F]{3})?$/.test(accent) ? accent : "#b5651d",
        min: 0,
        max: 100,
      };
      track.curves.push(style);
      renderAll();
    });
    detail.appendChild(addBtn);
  }

  /** Point-data track editor. One card per series rather than a table row: a point series
   *  carries far more style than a curve (display kind, depth bin, box percentiles, whisker
   *  rule), and only some of it applies to any one display — so the card shows exactly the
   *  controls that mean something for the chosen display and hides the rest. */
  function renderPointSection(track: Track): void {
    const sectionTitle = document.createElement("div");
    sectionTitle.className = "lp-section-title";
    sectionTitle.textContent = "Point data";
    detail.appendChild(sectionTitle);

    const note = document.createElement("div");
    note.className = "lp-note";
    note.textContent =
      "Measured samples — core plugs, XRD, CEC, oil show, core extras — drawn where they were actually sampled. Box and histogram summarise the samples inside each depth bin.";
    detail.appendChild(note);

    track.points ??= [];
    track.points.forEach((p, pi) => {
      const card = document.createElement("div");
      card.className = "lp-point-card";
      const grid = document.createElement("div");
      grid.className = "lp-fieldgrid";
      card.appendChild(grid);

      grid.appendChild(
        field(
          "Source",
          selectInput(p.source, [["core", "Core plugs"], ["aux", "Point dataset"]], (v) => {
            if (v !== p.source) {
              // Item names do not carry across sources — CPOR is a plug column, not an XRD
              // item — so a stale name would silently draw nothing. Re-seed from what this
              // well actually has under the new source.
              const seed = availablePoints.find((s) => s.source === v);
              p.source = v;
              p.dataset = v === "aux" ? seed?.dataset : undefined;
              p.item = seed?.item ?? "";
            }
            renderDetail();
          }),
        ),
      );
      if (p.source === "aux") {
        const ds = textInput(p.dataset ?? "", (v) => {
          p.dataset = v.trim().toUpperCase();
          ds.value = p.dataset;
        });
        ds.setAttribute("list", `${datalist.id}-ds`);
        grid.appendChild(field("Dataset", ds));
      }
      const itemIn = textInput(p.item, (v) => {
        p.item = v.trim().toUpperCase();
        itemIn.value = p.item;
      });
      itemIn.setAttribute("list", p.source === "core" ? `${datalist.id}-core` : `${datalist.id}-item`);
      grid.appendChild(field(p.source === "core" ? "Plug property" : "Item", itemIn));
      grid.appendChild(field("Color", colorInput(p.color, (v) => (p.color = v))));
      grid.appendChild(field("Min", numInput(p.min, (v) => (p.min = v))));
      grid.appendChild(field("Max", numInput(p.max, (v) => (p.max = v))));
      grid.appendChild(
        field(
          "Display",
          selectInput(
            p.display ?? "points",
            [["points", "Points"], ["box", "Box plot"], ["histogram", "Histogram"], ["text", "Text"]],
            (v) => {
              p.display = v;
              renderDetail();
            },
          ),
        ),
      );

      const display = p.display ?? "points";
      if (display === "box" || display === "histogram") {
        // Blank = follow the zoom (a twentieth of the visible window). An explicit height is
        // a fixed depth interval and deliberately does NOT follow the zoom, so the same bin
        // means the same thing at every scale.
        const binIn = document.createElement("input");
        binIn.type = "number";
        binIn.step = "any";
        binIn.placeholder = "auto";
        binIn.value = p.bin != null ? String(p.bin) : "";
        binIn.title = "Depth-bin height. Blank follows the zoom; a value is a fixed interval.";
        binIn.addEventListener("change", () => {
          const v = parseFloat(binIn.value);
          p.bin = Number.isFinite(v) && v > 0 ? v : undefined;
        });
        grid.appendChild(field("Bin height", binIn));
      }
      if (display === "box") {
        grid.appendChild(field("Box low %", numInput(p.box_lo ?? 25, (v) => (p.box_lo = clampPct(v)))));
        grid.appendChild(field("Box high %", numInput(p.box_hi ?? 75, (v) => (p.box_hi = clampPct(v)))));
        grid.appendChild(
          field(
            "Whiskers",
            selectInput(
              p.whisker ?? "tukey",
              [["tukey", "Tukey (k x IQR)"], ["percentile", "Percentiles"], ["minmax", "Full range"]],
              (v) => {
                p.whisker = v;
                renderDetail();
              },
            ),
          ),
        );
        if ((p.whisker ?? "tukey") === "tukey") {
          grid.appendChild(field("Tukey k", numInput(p.whisker_k ?? 1.5, (v) => (p.whisker_k = Math.max(0, v)), "0.1")));
        } else if (p.whisker === "percentile") {
          grid.appendChild(field("Whisker low %", numInput(p.whisker_lo ?? 10, (v) => (p.whisker_lo = clampPct(v)))));
          grid.appendChild(field("Whisker high %", numInput(p.whisker_hi ?? 90, (v) => (p.whisker_hi = clampPct(v)))));
        }
      }
      if (display === "histogram") {
        grid.appendChild(
          field("Value bins", numInput(p.hist_bins ?? 12, (v) => (p.hist_bins = Math.max(2, Math.round(v))), "1"))
        );
      }
      if (display === "box") {
        // Box only: a histogram's bars already ARE the samples, so the option would be
        // offered without meaning anything.
        const chk = document.createElement("input");
        chk.type = "checkbox";
        chk.checked = p.show_samples ?? false;
        chk.title = "Draw the individual samples as ticks above the box";
        chk.addEventListener("change", () => (p.show_samples = chk.checked));
        grid.appendChild(field("Show samples", chk));
      }

      const del = iconBtn("✕", "Remove this point series", () => {
        track.points?.splice(pi, 1);
        renderAll();
      });
      del.className = "lp-iconbtn lp-point-del";
      card.appendChild(del);
      detail.appendChild(card);
    });

    const addBtn = document.createElement("button");
    addBtn.className = "lp-btn";
    addBtn.textContent = "＋ Add point series";
    addBtn.addEventListener("click", () => {
      const seed = availablePoints[0];
      track.points ??= [];
      track.points.push({
        source: seed?.source ?? "core",
        dataset: seed?.dataset,
        item: seed?.item ?? "CPOR",
        color: "#5f7350",
        min: 0,
        max: 0.4,
      });
      renderAll();
    });
    detail.appendChild(addBtn);
  }

  function renderArraySection(track: Track): void {
    const sectionTitle = document.createElement("div");
    sectionTitle.className = "lp-section-title";
    sectionTitle.textContent = "Array log";
    detail.appendChild(sectionTitle);

    const note = document.createElement("div");
    note.className = "lp-note";
    note.textContent =
      availableArrays.length > 0
        ? "Curves holding a whole distribution at every depth — Monte Carlo realizations, NMR T2. All three displays read the SAME stored realizations, so changing the percentiles is a redraw, not a re-run."
        : "This well has no array logs. Run Monte Carlo with 'Store realizations' switched on to produce one.";
    detail.appendChild(note);

    track.arrays ??= [];
    track.arrays.forEach((a, ai) => {
      const card = document.createElement("div");
      card.className = "lp-point-card";
      const grid = document.createElement("div");
      grid.className = "lp-fieldgrid";
      card.appendChild(grid);

      const curveIn = textInput(a.curve_name, (v) => {
        a.curve_name = v.trim().toUpperCase();
        curveIn.value = a.curve_name;
      });
      curveIn.setAttribute("list", `${datalist.id}-array`);
      grid.appendChild(field("Array curve", curveIn));
      grid.appendChild(field("Color", colorInput(a.color, (v) => (a.color = v))));
      grid.appendChild(field("Min", numInput(a.min, (v) => (a.min = v))));
      grid.appendChild(field("Max", numInput(a.max, (v) => (a.max = v))));
      grid.appendChild(
        field(
          "Display",
          selectInput(
            a.display ?? "band",
            [["band", "Uncertainty band"], ["spaghetti", "Spaghetti"], ["heatmap", "Density heat map"]],
            (v) => {
              a.display = v;
              renderDetail();
            },
          ),
        ),
      );

      const display = a.display ?? "band";
      if (display === "band") {
        // The adjustable part: with the realizations stored, these are display settings.
        grid.appendChild(field("Band low %", numInput(a.band_lo ?? 10, (v) => (a.band_lo = clampPct(v)))));
        grid.appendChild(field("Band high %", numInput(a.band_hi ?? 90, (v) => (a.band_hi = clampPct(v)))));
        grid.appendChild(
          field("Shading", numInput(a.fill_opacity ?? 0.3, (v) => (a.fill_opacity = Math.max(0, Math.min(1, v))), "0.05"))
        );
        const chk = document.createElement("input");
        chk.type = "checkbox";
        chk.checked = a.show_median !== false;
        chk.title = "Draw the P50 line inside the band";
        chk.addEventListener("change", () => (a.show_median = chk.checked));
        grid.appendChild(field("Median line", chk));
      } else if (display === "spaghetti") {
        grid.appendChild(
          field(
            "Traces",
            numInput(a.traces ?? 40, (v) => (a.traces = Math.max(1, Math.round(v))), "1"),
          ),
        );
      } else {
        grid.appendChild(
          field("Value bins", numInput(a.hist_bins ?? 32, (v) => (a.hist_bins = Math.max(2, Math.round(v))), "1")),
        );
      }

      const del = iconBtn("✕", "Remove this array series", () => {
        track.arrays?.splice(ai, 1);
        renderAll();
      });
      del.className = "lp-iconbtn lp-point-del";
      card.appendChild(del);
      detail.appendChild(card);
    });

    const addBtn = document.createElement("button");
    addBtn.className = "lp-btn";
    addBtn.textContent = "＋ Add array series";
    addBtn.addEventListener("click", () => {
      track.arrays ??= [];
      track.arrays.push({
        curve_name: availableArrays[0] ?? "",
        color: "#4e79a7",
        min: 0,
        max: 0.4,
      });
      renderAll();
    });
    detail.appendChild(addBtn);
  }

  function renderImageSection(track: Track): void {
    const sectionTitle = document.createElement("div");
    sectionTitle.className = "lp-section-title";
    sectionTitle.textContent = "Images";
    detail.appendChild(sectionTitle);

    const note = document.createElement("div");
    note.className = "lp-note";
    note.textContent =
      availableImages.length > 0
        ? "Depth-registered pictures — thin sections, core photographs, SEM plates. Anchored plates sit at their sample depth at a fixed size; depth-scaled ones span their own top-to-base interval. Where two would overlap the deeper one is skipped, never moved."
        : "This well has no pictures yet. Import them with Data ▸ Import ▸ Images…";
    detail.appendChild(note);

    track.images ??= [];
    track.images.forEach((im, ii) => {
      const card = document.createElement("div");
      card.className = "lp-point-card";
      const grid = document.createElement("div");
      grid.className = "lp-fieldgrid";
      card.appendChild(grid);

      const dsIn = textInput(im.dataset, (v) => {
        im.dataset = v.trim().toUpperCase();
        dsIn.value = im.dataset;
      });
      dsIn.setAttribute("list", `${datalist.id}-imgds`);
      grid.appendChild(field("Dataset", dsIn));
      grid.appendChild(
        field(
          "Placement",
          selectInput(
            im.mode ?? "anchor",
            [["anchor", "Anchored at depth"], ["depth", "Scaled to interval"]],
            (v) => {
              im.mode = v;
              renderDetail();
            },
          ),
        ),
      );
      grid.appendChild(
        field(
          "Width of track",
          numInput(im.size ?? 0.9, (v) => (im.size = Math.max(0.05, Math.min(1, v))), "0.05"),
        ),
      );
      grid.appendChild(
        field(
          "Align",
          selectInput(im.align ?? "center", [["left", "Left"], ["center", "Center"], ["right", "Right"]], (v) => {
            im.align = v;
          }),
        ),
      );
      // Only a depth-scaled plate has a box its own aspect ratio does not already fill, so
      // "fit" is meaningless for an anchored one.
      if ((im.mode ?? "anchor") === "depth") {
        grid.appendChild(
          field(
            "Fit",
            // "Fill the track" is for a depth STRIP, whose height is the depth scale and whose
            // width is the track — neither of them the picture's own, so there is no true shape to
            // preserve. Never for a thin section: a squashed plate misstates grain shape, which is
            // the one thing it is there to show.
            selectInput(
              im.fit ?? "contain",
              [
                ["contain", "Whole picture"],
                ["cover", "Fill and crop"],
                ["stretch", "Fill the track (depth strips)"],
              ],
              (v) => {
                im.fit = v;
              },
            ),
          ),
        );
      }
      const labelChk = document.createElement("input");
      labelChk.type = "checkbox";
      labelChk.checked = im.label !== false;
      labelChk.title = "Draw the picture's name above it";
      labelChk.addEventListener("change", () => (im.label = labelChk.checked));
      grid.appendChild(field("Name label", labelChk));
      const borderChk = document.createElement("input");
      borderChk.type = "checkbox";
      borderChk.checked = im.border !== false;
      borderChk.title = "Hairline frame — a pale core photograph otherwise bleeds into the track";
      borderChk.addEventListener("change", () => (im.border = borderChk.checked));
      grid.appendChild(field("Frame", borderChk));

      const del = iconBtn("✕", "Remove this image series", () => {
        track.images?.splice(ii, 1);
        renderAll();
      });
      del.className = "lp-iconbtn lp-point-del";
      card.appendChild(del);
      detail.appendChild(card);
    });

    const addBtn = document.createElement("button");
    addBtn.className = "lp-btn";
    addBtn.textContent = "＋ Add image series";
    addBtn.addEventListener("click", () => {
      track.images ??= [];
      track.images.push({ dataset: availableImages[0] ?? "THIN SECTION" });
      renderAll();
    });
    detail.appendChild(addBtn);
  }

  function clampPct(v: number): number {
    return Math.max(0, Math.min(100, v));
  }

  function renderAll(): void {
    renderTrackList();
    renderDetail();
  }
  renderAll();

  const wrapper = document.createElement("div");
  wrapper.className = "lp-wrapper";
  wrapper.appendChild(content);
  wrapper.appendChild(foot);
  const close = openModal(`Layout Properties — ${working.name}`, wrapper, 980);

  const footBtn = (label: string, primary: boolean, onClick: () => void): void => {
    const b = document.createElement("button");
    b.className = "lp-btn" + (primary ? " primary" : "");
    b.textContent = label;
    b.addEventListener("click", onClick);
    foot.appendChild(b);
  };
  footBtn("Cancel", false, () => close());
  footBtn("Apply", true, () => onApply(structuredClone(working)));
  footBtn("OK", true, () => {
    onApply(structuredClone(working));
    close();
  });
}
