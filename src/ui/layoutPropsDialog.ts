import type { CurveStyle, Layout, Track } from "../ipc";
import { openModal } from "./modal";

/** Layout Properties dialog (per the standard-layout prototype): a track list with
 *  insert/delete/duplicate/reorder on the left, and the selected track's settings +
 *  curve style table (color, scale, fill shading) on the right. Operates on a deep
 *  clone; the caller receives the edited layout on Apply/OK. */
export function openLayoutPropsDialog(
  layout: Layout,
  availableCurves: string[],
  onApply: (edited: Layout) => void,
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
  for (const name of availableCurves) {
    const opt = document.createElement("option");
    opt.value = name;
    datalist.appendChild(opt);
  }
  content.appendChild(datalist);

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
        selectInput(track.kind ?? "curves", [["curves", "Curves"], ["well_diagram", "Well diagram"]], (v) => {
          track.kind = v;
          renderDetail();
        }),
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

    const sectionTitle = document.createElement("div");
    sectionTitle.className = "lp-section-title";
    sectionTitle.textContent = "Curves";
    detail.appendChild(sectionTitle);

    const table = document.createElement("table");
    table.className = "lp-curvetable";
    table.innerHTML = `<thead><tr>
      <th>Curve</th><th>Color</th><th>Min</th><th>Max</th>
      <th>Fill</th><th>Fill color</th><th>Opacity</th><th></th>
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
      });
      nameInput.setAttribute("list", datalist.id);
      cell(nameInput);
      cell(colorInput(c.color, (v) => (c.color = v)), "lp-tiny");
      cell(numInput(c.min, (v) => (c.min = v)), "lp-num");
      cell(numInput(c.max, (v) => (c.max = v)), "lp-num");
      cell(
        selectInput(
          c.fill ?? "none",
          [["none", "None"], ["left", "To left edge"], ["right", "To right edge"], ["blocks", "Facies blocks"]],
          (v) => {
            c.fill = v;
          },
        ),
      );
      cell(colorInput(c.fill_color ?? c.color, (v) => (c.fill_color = v)), "lp-tiny");
      const opacity = numInput(c.fill_opacity ?? 0.25, (v) => (c.fill_opacity = Math.max(0, Math.min(1, v))), "0.05");
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
      const style: CurveStyle = {
        curve_name: availableCurves.find((n) => !track.curves.some((c) => c.curve_name === n)) ?? "GR",
        color: /^#[0-9a-fA-F]{3}(?:[0-9a-fA-F]{3})?$/.test(accent) ? accent : "#b5651d",
        min: 0,
        max: 100,
      };
      track.curves.push(style);
      renderAll();
    });
    detail.appendChild(addBtn);
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
  const close = openModal(`Layout Properties — ${working.name}`, wrapper, 880);

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
