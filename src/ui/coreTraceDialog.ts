import {
  buildCoreStrips,
  CORE_STRIP_DATASET,
  detectCoreLanes,
  extractCoreLog,
  getWellImage,
  listDocuments,
  listImageDatasets,
  listWellImages,
  saveDocument,
  DEFAULT_FLUOR,
  type CoreLogResult,
  type FluorClass,
  type ImageInfo,
  type Lane,
  type PlateLayout,
} from "../ipc";
import { buildColourBand } from "./colourBand";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { loadCurveNames } from "./plotCommon";
import { buildPlateStrip } from "./plateStrip";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow } from "./modal";

/**
 * Reading a log off core photographs (Advance ▸ Core Imaging ▸ Photo Log…).
 *
 * **Its own tool, not a section of the conditioning workspace** (Jauhar, 2026-08-01: *"for core
 * image conversion to log, separate it from core photos tools, it should have independent tools"*).
 * They are two jobs with two lifetimes: conditioning is done once per delivery and finished, while
 * a trace is read, compared against GR, re-laid-out and read again. Sharing one pane meant scrolling
 * past a colour picker to reach a depth table.
 *
 * **The columns of a packed plate are BARRELS, and each carries its own depths.** A core-display
 * plate is four columns of core side by side, each labelled with its own top and base, with
 * preserved intervals and part-filled last columns between them. Splitting it into four equal parts
 * of one continuous span — which is all the old lane count could do — places every sample below the
 * first gap at the wrong depth. So the lay-out is a TABLE: where each run of core is, and what
 * interval it covers.
 *
 * **Detection proposes and the user accepts.** The split is measured off the picture's own
 * brightness profile and drawn, because four clean columns and a smear the threshold happened to
 * cut in four are the same answer and completely different situations — the same reason
 * `registration.rs` returns its whole correlogram rather than only the peak. And the DEPTHS are
 * never guessed: nothing in the pixels says what depth a column of rock came from.
 *
 * The lay-out persists as a `corelanes` document keyed `<well_id>/<dataset>`, the way the mineral
 * classifier keeps its clicks — a list anyone can read and correct, rather than a blob.
 */

/** The document type the per-plate lay-outs live under. */
const LAYOUT_DOC = "corelanes";

type Layouts = Record<string, PlateLayout>;

export async function buildCoreTraceContent(): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  wrap.className = "module-pane";

  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent =
      "Select a well in the Wells pane first — a trace is read one well at a time.";
    wrap.appendChild(none);
    return { el: wrap };
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Reads darkness, redness and texture down the core. They are IMAGE measures, not petrophysical " +
    "properties: darkness follows shale in most clastic sections without being a shale volume, " +
    "which is why nothing here is called VSH. Condition the pictures first — a darkness compared " +
    "across boxes shot under two lamps is a comparison of the lamps.";
  wrap.appendChild(intro);

  // ---- the delivery -------------------------------------------------------
  const dsSel = document.createElement("select");
  dsSel.className = "form-control";
  const datasets = await listImageDatasets(well.well_id).catch(() => [] as [string, number][]);
  for (const [name, n] of datasets) {
    const o = document.createElement("option");
    o.value = name;
    o.textContent = `${name} — ${n} picture(s)`;
    dsSel.appendChild(o);
  }
  const core = datasets.find(([n]) => /CORE|PHOTO|SLAB/.test(n.toUpperCase()));
  if (core) dsSel.value = core[0];
  wrap.appendChild(formRow("Picture dataset", dsSel));

  if (!datasets.length) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.style.color = "var(--warn)";
    none.textContent = "This well has no pictures. Import some with Data ▸ Import ▸ Images…";
    wrap.appendChild(none);
    return { el: wrap };
  }

  let plates: ImageInfo[] = [];
  let current = "";
  let layouts: Layouts = {};

  // ---- the delivery, as pictures -----------------------------------------
  const filmstrip = buildPlateStrip((id) => {
    current = id;
    filmstrip.mark(id);
    void showPlate();
  });
  wrap.appendChild(filmstrip.el);

  // ---- how the picture is laid out ---------------------------------------
  /** A row of buttons behaving as one choice — every option visible at once, which is what you want
   *  when the answer is read off the picture rather than remembered. */
  const segmented = (
    options: { value: string; label: string; title: string }[],
    initial: string
  ): { el: HTMLElement; get: () => string; set: (v: string) => void } => {
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
      });
      btns.push(b);
      row.appendChild(b);
    }
    paint();
    return {
      el: row,
      get: () => value,
      set: (v) => {
        value = v;
        paint();
      },
    };
  };

  const axisPick = segmented(
    [
      {
        value: "x",
        label: "→ across",
        title: "Depth runs along the width — a core box laid out left to right.",
      },
      {
        value: "y",
        label: "↓ down",
        title:
          "Depth runs down the picture — a single vertical strip, or the columns of a core-display plate.",
      },
    ],
    "x"
  );
  wrap.appendChild(formRow("Depth runs", axisPick.el, "Read it off the picture above."));

  // ---- which light, and what counts as fluorescence -----------------------
  //
  // DECLARED, never detected. A UV frame is dark, and so is a daylight photograph of dark shale in a
  // shadowed box — the evidence for "this is ultraviolet" would be the brightness about to be
  // measured, which is the circle that makes an impregnated thin section something the user states
  // rather than something the pixels are asked about.
  const lightPick = segmented(
    [
      {
        value: "white",
        label: "☀ Daylight",
        title:
          "White light — darkness, redness and texture: proxy measures for shale, stain and lamination.",
      },
      {
        value: "uv",
        label: "🔦 Ultraviolet",
        title:
          "A UV frame — how much of each slab fluoresces. An INFERRED SHOW, never a pay flag: minerals, drilling-fluid additives and dead oil all fluoresce.",
      },
    ],
    "white"
  );
  wrap.appendChild(
    formRow("Light", lightPick.el, "The two lights are two deliveries; pick the one this is.")
  );

  // --- Output log set (`logSetPicker.ts`). The photograph traces were the one module output with
  // no log set at all, so "which conditioning produced this CPHOTO_DARK" had no answer and each
  // re-read silently replaced the last. Defaults to CPHOTO.
  const setPicker = buildLogSetPicker({ read: false, write: "CPHOTO" });
  for (const row of setPicker.rows) wrap.appendChild(row);

  const fluorBox = document.createElement("div");
  fluorBox.style.display = "none";
  let classes: FluorClass[] = [{ ...DEFAULT_FLUOR }];
  let bands: { row: HTMLElement; read: () => FluorClass }[] = [];

  const paintFluor = (): void => {
    fluorBox.innerHTML = "";
    bands = [];
    const note = document.createElement("div");
    note.className = "eq-note";
    note.textContent =
      "Tune this against the preview on ONE photograph, and judge it by whether it agrees with " +
      "your own show descriptions — never by whether the average looks about right. Show " +
      "descriptions often separate bright yellow-green from dull blue-white; add a second kind if " +
      "yours do, and each gets its own curve. Nothing here assumes what that split means.";
    fluorBox.appendChild(note);

    classes.forEach((c, i) => {
      const row = document.createElement("div");
      row.className = "fluor-class";

      const head = document.createElement("div");
      head.className = "fluor-class-head";
      const name = document.createElement("input");
      name.className = "field-label";
      name.value = c.name;
      name.placeholder = "SHOW";
      name.title = "Becomes the curve suffix when there is more than one kind.";
      head.appendChild(name);
      if (classes.length > 1) {
        const del = document.createElement("button");
        del.className = "btn";
        del.textContent = "✕";
        del.title = "Remove this kind of fluorescence.";
        del.addEventListener("click", () => {
          // Read every card BEFORE dropping one. Filtering first and then indexing `bands` by the
          // new position reads each surviving card off its neighbour's control — which looks right
          // when you delete the last one and silently swaps the edits when you delete the first.
          classes = bands.map((b) => b.read()).filter((_, k) => k !== i);
          paintFluor();
        });
        head.appendChild(del);
      }
      row.appendChild(head);

      // The hue window and the two floors come from the SHARED control, so the fluorescence band
      // and the pore band can never drift apart about what a wrapped band means.
      const band = buildColourBand(
        { hue_lo: c.hue_lo, hue_hi: c.hue_hi, sat_min: c.sat_min, val_min: c.val_min },
        () => {}
      );
      row.appendChild(band.el);

      // The one thing a PoreColorBand has no room for. A floor cannot express "dull blue-white",
      // because white is the absence of colour.
      const ceil = document.createElement("input");
      ceil.type = "range";
      ceil.min = "0";
      ceil.max = "1";
      ceil.step = "0.01";
      ceil.value = String(c.sat_max ?? 1);
      const ceilOut = document.createElement("span");
      ceilOut.className = "eq-note";
      const paintCeil = (): void => {
        ceilOut.textContent =
          Number(ceil.value) >= 0.999 ? "no limit" : `≤ ${Number(ceil.value).toFixed(2)}`;
      };
      ceil.addEventListener("input", paintCeil);
      paintCeil();
      const ceilWrap = document.createElement("div");
      ceilWrap.style.display = "flex";
      ceilWrap.style.gap = "8px";
      ceilWrap.style.alignItems = "center";
      ceil.style.flex = "1";
      ceilWrap.appendChild(ceil);
      ceilWrap.appendChild(ceilOut);
      row.appendChild(
        formRow(
          "Pale limit",
          ceilWrap,
          "How washed-out a colour still counts. Lower it for a DULL BLUE-WHITE description — white is the absence of colour, so it cannot be written as a floor."
        )
      );

      bands.push({
        row,
        read: () => ({ ...band.get(), name: name.value.trim() || "SHOW", sat_max: Number(ceil.value) }),
      });
      fluorBox.appendChild(row);
    });

    const add = document.createElement("button");
    add.className = "btn";
    add.textContent = "+ Another kind of fluorescence";
    add.addEventListener("click", () => {
      classes = bands.map((b) => b.read());
      classes.push({ ...DEFAULT_FLUOR, name: `SHOW${classes.length + 1}` });
      paintFluor();
    });
    fluorBox.appendChild(add);
  };
  paintFluor();
  wrap.appendChild(fluorBox);

  const isUv = (): boolean => lightPick.get() === "uv";
  const readClasses = (): FluorClass[] => bands.map((b) => b.read());
  // Declared later (the sand/shale row sits below), so it is called through a holder rather than
  // by name — the light toggle has to reach both.
  let syncLith: () => void = () => {};
  const syncLight = (): void => {
    fluorBox.style.display = isUv() ? "" : "none";
    syncLith();
  };
  for (const b of Array.from(lightPick.el.querySelectorAll("button"))) {
    b.addEventListener("click", syncLight);
  }
  syncLight();

  const revChk = document.createElement("input");
  revChk.type = "checkbox";
  const revLabel = document.createElement("label");
  revLabel.appendChild(revChk);
  revLabel.appendChild(document.createTextNode(" Deepest end first (the box is the other way round)"));
  revLabel.style.display = "block";
  wrap.appendChild(revLabel);

  const lanePick = segmented(
    [1, 2, 3, 4, 5, 6].map((n) => ({
      value: String(n),
      label: String(n),
      title:
        n === 1
          ? "One run of core in the frame."
          : `${n} runs of core, read in order over ONE continuous interval. Equal lanes are an approximation — use the column table below where the runs are separate barrels.`,
    })),
    "1"
  );
  wrap.appendChild(
    formRow(
      "Runs of core",
      lanePick.el,
      "The fall-back for a picture with no column table: equal lanes over the picture's own interval."
    )
  );

  // ---- the column table ---------------------------------------------------
  const colBox = document.createElement("div");
  colBox.style.borderTop = "1px solid var(--border)";
  colBox.style.marginTop = "10px";
  colBox.style.paddingTop = "8px";
  wrap.appendChild(colBox);

  const colHead = document.createElement("div");
  colHead.className = "field-label";
  colHead.textContent = "Columns of this picture";
  colBox.appendChild(colHead);

  const colNote = document.createElement("div");
  colNote.className = "eq-note";
  colNote.textContent =
    "A core-display plate carries several barrels side by side, each with its own depths. Detect " +
    "them, then type the depths its caption states — nothing in the picture says what depth a " +
    "column of rock came from. Leave every depth blank to have the picture's own interval shared " +
    "out across the columns instead.";
  colBox.appendChild(colNote);

  const detectBtn = document.createElement("button");
  detectBtn.className = "btn";
  detectBtn.textContent = "Detect columns";
  detectBtn.title =
    "Measures the picture's brightness across the frame and proposes where the runs of core are. " +
    "Nothing is applied — the proposal lands in the table below.";
  const addBtn = document.createElement("button");
  addBtn.className = "btn";
  addBtn.textContent = "Add column";
  const clearBtn = document.createElement("button");
  clearBtn.className = "btn";
  clearBtn.textContent = "Clear";
  clearBtn.title = "Drops this picture's column table, so it falls back to equal lanes.";
  const colBtns = document.createElement("div");
  colBtns.style.display = "flex";
  colBtns.style.gap = "8px";
  colBtns.style.margin = "6px 0";
  colBtns.append(detectBtn, addBtn, clearBtn);
  colBox.appendChild(colBtns);

  /** The picture with the detected columns drawn on it. The table is the record; this is how it is
   *  judged, because a column table read as eight numbers says nothing about whether it landed on
   *  the rock. */
  const shot = document.createElement("canvas");
  shot.style.width = "100%";
  shot.style.maxHeight = "260px";
  shot.style.objectFit = "contain";
  shot.style.background = "var(--bg-panel-alt)";
  shot.style.borderRadius = "var(--r-sm)";
  colBox.appendChild(shot);

  const colTable = document.createElement("div");
  colBox.appendChild(colTable);

  // OUTSIDE the table, and updated on every depth change rather than on a redraw. It lived inside
  // `drawColumns` first, which meant it only appeared when the table was rebuilt — so typing a
  // depth into one row, which is exactly how a plate becomes half-labelled, never raised it. Caught
  // in the browser, not by the compiler.
  const colWarn = document.createElement("div");
  colWarn.className = "eq-note";
  colWarn.style.color = "var(--warn)";
  colBox.appendChild(colWarn);

  const colStatus = document.createElement("div");
  colStatus.className = "eq-note";
  colBox.appendChild(colStatus);

  /** The all-or-nothing rule, said HERE rather than after a run: half a plate labelled cannot be
   *  placed without assuming the core runs on without a break. */
  const syncWarn = (): void => {
    const lay = layouts[current];
    const lanes = lay?.lanes ?? [];
    const labelled = lanes.filter((l) => l.depth_top != null || l.depth_base != null).length;
    colWarn.textContent =
      labelled > 0 && labelled < lanes.length
        ? `${labelled} of ${lanes.length} columns carry a depth. Fill the rest in, or clear them ` +
          "all — placing the blank ones would mean assuming the core runs on without a break, " +
          "which is exactly what a preserved interval is not."
        : "";
  };

  let bitmap: ImageBitmap | null = null;

  /** Redraws the current plate with its columns and window marked. */
  const paintShot = (): void => {
    const ctx = shot.getContext("2d");
    if (!ctx) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const cw = Math.max(1, Math.round(shot.clientWidth * dpr));
    const ch = Math.max(1, Math.round(260 * dpr));
    if (shot.width !== cw || shot.height !== ch) {
      shot.width = cw;
      shot.height = ch;
    }
    ctx.clearRect(0, 0, cw, ch);
    if (!bitmap) return;
    // Fit, never stretch: a column drawn at the wrong aspect ratio is a column in the wrong place.
    const k = Math.min(cw / bitmap.width, ch / bitmap.height);
    const dw = bitmap.width * k;
    const dh = bitmap.height * k;
    const ox = (cw - dw) / 2;
    const oy = (ch - dh) / 2;
    ctx.drawImage(bitmap, ox, oy, dw, dh);

    const lay = layouts[current];
    if (!lay) return;
    // The across axis of the LAY-OUT is the picture's width when depth runs down it, and its height
    // when depth runs across — the same frame the reader works in.
    const down = axisPick.get() === "y";
    ctx.lineWidth = 2 * dpr;
    ctx.strokeStyle = "rgba(198,113,57,0.95)";
    ctx.fillStyle = "rgba(198,113,57,0.16)";
    for (const l of lay.lanes) {
      if (down) {
        ctx.fillRect(ox + l.start * dw, oy, (l.end - l.start) * dw, dh);
        ctx.strokeRect(ox + l.start * dw, oy, (l.end - l.start) * dw, dh);
      } else {
        ctx.fillRect(ox, oy + l.start * dh, dw, (l.end - l.start) * dh);
        ctx.strokeRect(ox, oy + l.start * dh, dw, (l.end - l.start) * dh);
      }
    }
    if (lay.span) {
      ctx.strokeStyle = "rgba(122,138,94,0.95)";
      ctx.setLineDash([6 * dpr, 4 * dpr]);
      const [a, b] = lay.span;
      if (down) {
        ctx.strokeRect(ox, oy + a * dh, dw, (b - a) * dh);
      } else {
        ctx.strokeRect(ox + a * dw, oy, (b - a) * dw, dh);
      }
      ctx.setLineDash([]);
    }
  };

  const numField = (
    value: number | null | undefined,
    onSet: (v: number | null) => void
  ): HTMLInputElement => {
    const inp = document.createElement("input");
    inp.className = "form-control";
    inp.type = "number";
    inp.step = "0.01";
    inp.style.width = "6.5rem";
    inp.value = value == null || !Number.isFinite(value) ? "" : String(value);
    inp.addEventListener("change", () => {
      const t = inp.value.trim();
      const n = Number(t);
      // A blank is "not stated", never 0 — a depth silently pinned to zero is a wrong answer that
      // keeps computing, the same rule the zone-parameter batch follows.
      onSet(t === "" || !Number.isFinite(n) ? null : n);
      void persist();
      syncWarn();
      paintShot();
    });
    return inp;
  };

  const drawColumns = (): void => {
    colTable.innerHTML = "";
    const lay = layouts[current];
    if (!lay || !lay.lanes.length) {
      const none = document.createElement("div");
      none.className = "eq-note";
      none.textContent =
        "No column table for this picture, so it is read as " +
        `${lanePick.get()} equal lane(s) over its own interval.`;
      colTable.appendChild(none);
      syncWarn();
      return;
    }
    const table = document.createElement("table");
    table.className = "data-table";
    const hrow = document.createElement("tr");
    for (const h of ["#", "From", "To", "Depth top", "Depth base", ""]) {
      const th = document.createElement("th");
      th.textContent = h;
      hrow.appendChild(th);
    }
    table.appendChild(hrow);

    lay.lanes.forEach((l, i) => {
      const tr = document.createElement("tr");
      const cell = (el: HTMLElement | string): void => {
        const td = document.createElement("td");
        if (typeof el === "string") td.textContent = el;
        else td.appendChild(el);
        tr.appendChild(td);
      };
      cell(String(i + 1));
      cell(`${(l.start * 100).toFixed(1)}%`);
      cell(`${(l.end * 100).toFixed(1)}%`);
      cell(
        numField(l.depth_top, (v) => {
          l.depth_top = v;
        })
      );
      cell(
        numField(l.depth_base, (v) => {
          l.depth_base = v;
        })
      );
      const del = document.createElement("button");
      del.className = "btn";
      del.textContent = "✕";
      del.title = "Drop this column";
      del.addEventListener("click", () => {
        lay.lanes.splice(i, 1);
        void persist();
        drawColumns();
        paintShot();
      });
      cell(del);
      table.appendChild(tr);
    });
    colTable.appendChild(table);
    syncWarn();
  };

  /** Keeps the lay-outs with the project. Debounced by the fact that it only runs on a committed
   *  change (a `change` event, a detect, an add), never on every keystroke. */
  const persist = async (): Promise<void> => {
    try {
      await saveDocument(LAYOUT_DOC, `${well.well_id}/${dsSel.value}`, JSON.stringify(layouts));
    } catch {
      // A lay-out that could not be stored is still usable for this run; saying so in the status
      // line beats blocking the work.
      setStatus("Could not save the column table — it still applies to this run");
    }
  };

  const loadLayouts = async (): Promise<void> => {
    layouts = {};
    try {
      const docs = await listDocuments(LAYOUT_DOC);
      const mine = docs.find((d) => d.name === `${well.well_id}/${dsSel.value}`);
      if (mine?.json) layouts = JSON.parse(mine.json) as Layouts;
    } catch {
      layouts = {};
    }
  };

  async function showPlate(): Promise<void> {
    bitmap?.close();
    bitmap = null;
    if (current) {
      try {
        const buf = await getWellImage(current);
        const mime = plates.find((p) => p.image_id === current)?.mime ?? "image/jpeg";
        bitmap = await createImageBitmap(new Blob([buf], { type: mime }));
      } catch {
        /* a picture the viewer cannot decode still gets its row in the table */
      }
    }
    drawColumns();
    paintShot();
  }

  detectBtn.addEventListener("click", () => {
    if (!current) {
      colStatus.textContent = "Pick a picture in the strip above first.";
      return;
    }
    void (async () => {
      detectBtn.disabled = true;
      colStatus.textContent = "Measuring…";
      try {
        const det = await detectCoreLanes(current, axisPick.get() as "x" | "y", revChk.checked);
        // The depths already typed are KEPT where a proposed column lands on the same place — a
        // re-detect after nudging the axis must not throw away a table somebody typed by hand.
        const old = layouts[current]?.lanes ?? [];
        const lanes: Lane[] = det.lanes.map((l) => {
          const mid = (l.start + l.end) / 2;
          const was = old.find((o) => mid >= o.start && mid <= o.end);
          return { ...l, depth_top: was?.depth_top ?? null, depth_base: was?.depth_base ?? null };
        });
        layouts[current] = { span: det.span, lanes };
        await persist();
        drawColumns();
        paintShot();
        colStatus.textContent = det.notes.join(" ");
      } catch (e) {
        colStatus.textContent = String(e);
      } finally {
        detectBtn.disabled = false;
      }
    })();
  });

  addBtn.addEventListener("click", () => {
    if (!current) return;
    const lay = (layouts[current] ??= { span: null, lanes: [] });
    const last = lay.lanes[lay.lanes.length - 1];
    const start = last ? Math.min(0.95, last.end + 0.01) : 0.0;
    lay.lanes.push({ start, end: Math.min(1, start + 0.15), depth_top: null, depth_base: null });
    void persist();
    drawColumns();
    paintShot();
  });

  clearBtn.addEventListener("click", () => {
    if (!current) return;
    delete layouts[current];
    void persist();
    drawColumns();
    paintShot();
  });

  // ---- reading it ---------------------------------------------------------
  const runBox = document.createElement("div");
  runBox.style.borderTop = "1px solid var(--border)";
  runBox.style.marginTop = "10px";
  runBox.style.paddingTop = "8px";
  wrap.appendChild(runBox);

  const cmpSel = document.createElement("select");
  cmpSel.className = "form-control";
  {
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "— none: do not check —";
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
  runBox.appendChild(
    formRow(
      "Check against",
      cmpSel,
      "Reports how each measure tracks a real log over the same interval. It is the only thing " +
        "that says whether the trace is about the rock — and a strongly NEGATIVE darkness usually " +
        "means the depth axis is the other way round."
    )
  );

  // --- Unfold for dipping beds -----------------------------------------------
  // A slab average runs ACROSS the core, so a contact crossing it diagonally is averaged with the
  // rock either side over the whole width and comes back as a ramp. Stated as a depth DROP rather
  // than an angle: an angle needs the core's diameter, which nothing here stores, while the drop
  // is read straight off the picture by noting one contact's depth at each edge.
  const unfoldIn = document.createElement("input");
  unfoldIn.className = "form-control";
  unfoldIn.type = "number";
  unfoldIn.step = "0.01";
  unfoldIn.placeholder = "0 — beds read flat";
  // Proposing one, `registration.rs`'s way: scan a range of dips, score each by how sharply the
  // core reads at it, and hand back the WHOLE scan. A peak is a proposal, never an application —
  // the user types the number in, exactly as they accept a depth-registration shift.
  const proposeBtn = document.createElement("button");
  proposeBtn.type = "button";
  proposeBtn.className = "btn";
  proposeBtn.textContent = "Propose…";
  const unfoldRow = document.createElement("div");
  unfoldRow.className = "intake-template-row";
  unfoldRow.append(unfoldIn, proposeBtn);
  runBox.appendChild(
    formRow(
      "Unfold dipping beds",
      unfoldRow,
      "How much DEEPER the bedding sits at the RIGHT edge of the core than at the left, in the " +
        "project's depth unit. Read it off the picture: note one contact's depth at each edge and " +
        "subtract. Propose… scans a range of dips and shows how sharply the core reads at each — " +
        "one peak means the dip is determined, a flat scan means this core has no bedding " +
        "contrast to find one from and the maximum is noise.",
    ),
  );
  // The scan lives under the field it fills, and is drawn rather than reduced to its peak: a
  // sharp maximum and a flat plateau give the same number and mean completely different things.
  const scanBox = document.createElement("div");
  scanBox.className = "unfold-scan";
  scanBox.hidden = true;
  runBox.appendChild(scanBox);

  // --- Sand/shale curve ------------------------------------------------------
  // Jauhar's remaining item from the UV round: a DISCRETE curve off the white-light trace, because
  // a correlation panel can consume a class curve and cannot consume a continuous proxy.
  const lithChk = document.createElement("input");
  lithChk.type = "checkbox";
  const lithCut = document.createElement("input");
  lithCut.className = "form-control";
  lithCut.type = "number";
  lithCut.step = "0.01";
  lithCut.placeholder = "Otsu, from this core";
  const lithRow = document.createElement("div");
  lithRow.className = "intake-template-row";
  const lithLab = document.createElement("label");
  lithLab.style.display = "flex";
  lithLab.style.alignItems = "center";
  lithLab.style.gap = "6px";
  lithLab.append(lithChk, document.createTextNode("Write CPHOTO_LITH"));
  const lithMinBed = document.createElement("input");
  lithMinBed.className = "form-control";
  lithMinBed.type = "number";
  lithMinBed.step = "0.01";
  lithMinBed.min = "0";
  lithMinBed.placeholder = "no minimum — every flicker kept";
  lithRow.append(lithLab, lithCut, lithMinBed);
  runBox.appendChild(
    formRow(
      "Sand / shale",
      lithRow,
      "A two-class curve cut out of the darkness trace — 0 lighter, 1 darker. It is a reading of " +
        "DARKNESS, not a shale volume: the same dark band is mudstone in one core, oil stain in " +
        "another, which is why it is never called VSH. Leave the cut blank and Otsu proposes one " +
        "from this core's own trace. The third box is the thinnest bed to keep, in the project's " +
        "depth unit — beds thinner than it are absorbed into the rock around them. Blank keeps " +
        "every flicker: there is no thickness that is right in two cores, so none is assumed.",
    ),
  );
  // Under UV the brightness IS the fluorescence, so there is no darkness to cut.
  syncLith = (): void => {
    const row = lithRow.closest<HTMLElement>(".form-row");
    if (row) row.hidden = isUv();
  };
  syncLith();

  const readBtn = document.createElement("button");
  readBtn.className = "btn btn-accent";
  readBtn.textContent = "Read the trace";
  const writeBtn = document.createElement("button");
  writeBtn.className = "btn";
  writeBtn.textContent = "Save as curves";
  writeBtn.disabled = true;
  const stripBtn = document.createElement("button");
  stripBtn.className = "btn";
  stripBtn.textContent = "Build depth strips";
  stripBtn.title =
    "Cuts every box into its runs and stacks them into one tall picture per box, core running down " +
    "it. Put an image track on it in depth mode to see it beside the logs.";

  // Where the strips land. Visible and editable rather than fixed, because a white-light delivery
  // and a UV one both want strips and one name would have the second quietly replace the first.
  const stripTarget = document.createElement("input");
  stripTarget.className = "form-control";
  stripTarget.style.maxWidth = "12rem";
  stripTarget.title = "The picture dataset the strips are written to.";
  const suggestTarget = (): void => {
    const src = dsSel.value.toUpperCase();
    const extra = src.replace(/CORE|PHOTO|PHOTOS|SLAB/g, "").replace(/\s+/g, " ").trim();
    stripTarget.value = extra ? `${CORE_STRIP_DATASET} ${extra}` : CORE_STRIP_DATASET;
  };
  suggestTarget();

  const readRow = document.createElement("div");
  readRow.style.display = "flex";
  readRow.style.gap = "8px";
  readRow.style.margin = "6px 0";
  readRow.style.flexWrap = "wrap";
  readRow.append(readBtn, writeBtn, stripBtn, stripTarget);
  runBox.appendChild(readRow);

  const trace = document.createElement("canvas");
  trace.style.width = "100%";
  trace.style.height = "220px";
  trace.style.background = "var(--bg-panel-alt)";
  trace.style.borderRadius = "var(--r-sm)";
  trace.hidden = true;
  runBox.appendChild(trace);

  const logNote = document.createElement("div");
  logNote.className = "eq-note";
  runBox.appendChild(logNote);

  /** Draws the measures as three tracks down depth, the way they will look in a log view.
   *
   *  A table of percentiles cannot say whether a trace has bedding in it, and bedding is the whole
   *  question. Each track is scaled to its OWN range — darkness, redness and texture are three
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
        : `${c.name.replace("CPHOTO_", "")} —`
    );
    const head =
      `${res.samples} sample(s) from ${res.photographs} photograph(s), ` +
      `${res.depth_min.toFixed(1)} to ${res.depth_max.toFixed(1)}.`;
    const agree = cmpSel.value ? ` Against ${cmpSel.value}: ${bits.join(", ")}.` : "";
    return (
      head +
      agree +
      (res.notes.length ? " " + res.notes.join(" ") : "") +
      (res.skipped.length ? ` Left out: ${res.skipped.join("; ")}` : "")
    );
  };

  const runRead = async (write: boolean): Promise<void> => {
    readBtn.disabled = true;
    writeBtn.disabled = true;
    logNote.textContent = write ? "Saving…" : "Reading…";
    try {
      const res = await extractCoreLog({
        well_id: well.well_id,
        dataset: dsSel.value,
        axis: axisPick.get() as "x" | "y",
        reverse: revChk.checked,
        lanes: Number(lanePick.get()) || 1,
        layouts,
        light: isUv() ? "uv" : "white",
        fluor: isUv() ? readClasses() : [],
        compare_curve: cmpSel.value || null,
        output_set: setPicker.outputSet(),
        unfold: unfoldIn.value.trim() ? Number(unfoldIn.value) : null,
        lith: lithChk.checked && !isUv(),
        lith_cut: lithCut.value.trim() ? Number(lithCut.value) : null,
        lith_min_bed: lithMinBed.value.trim() ? Number(lithMinBed.value) : null,
        write,
      });
      trace.hidden = false;
      drawTrace(res);
      logNote.textContent = (write ? `Saved ${res.written.join(", ")}. ` : "") + describeRun(res);
      if (write) {
        setStatus(`Read ${res.written.length} curve(s) off ${dsSel.value}`);
        recordProcess(
          "Edit",
          `Core photo log on ${dsSel.value}: ${res.written.join(", ")}`,
          well.well_name
        );
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

  /**
   * Draws the scan and offers its peak. Drawn, never reduced to a number: a sharp maximum and a
   * flat plateau return the same figure and mean completely different things, which is the whole
   * reason `registration.rs` returns its correlogram rather than only its shift.
   *
   * The score axis runs from ZERO rather than from the scan's own minimum. Cropping to the data
   * makes a 2% wobble fill the box and read as a decisive peak — the same trap the depth
   * registration's fixed −1..1 axis avoids.
   */
  const drawScan = (scan: NonNullable<CoreLogResult["unfold_scan"]>): void => {
    scanBox.hidden = false;
    scanBox.textContent = "";
    const live = scan.scores.filter((s) => Number.isFinite(s));
    const top = live.length ? Math.max(...live) : 1;
    const bars = document.createElement("div");
    bars.className = "unfold-scan-bars";
    const bestAt = scan.best ?? null;
    scan.drops.forEach((d, i) => {
      const s = scan.scores[i];
      const bar = document.createElement("div");
      bar.className = "unfold-scan-bar";
      // A candidate that sheared away too much core is not a low score, it is NO score — drawn as
      // an empty slot rather than a short bar, which would read as "tried and poor".
      if (!Number.isFinite(s)) {
        bar.classList.add("unfold-scan-none");
        bar.title = `${d.toFixed(2)} — not scored, too little core left at this shear`;
      } else {
        bar.style.height = `${Math.max(2, (s / (top || 1)) * 100)}%`;
        bar.title = `${d.toFixed(2)} → ${s.toFixed(4)}`;
        if (bestAt !== null && d === bestAt) bar.classList.add("unfold-scan-best");
      }
      bars.appendChild(bar);
    });
    scanBox.appendChild(bars);
    const foot = document.createElement("div");
    foot.className = "unfold-scan-foot";
    if (bestAt !== null) {
      const take = document.createElement("button");
      take.type = "button";
      take.className = "btn btn-accent";
      take.textContent = `Use ${bestAt.toFixed(2)}`;
      // Filling the box is the whole of "accept": the run still has to be pressed, so a proposal
      // can never become a measurement by having been looked at.
      take.addEventListener("click", () => {
        unfoldIn.value = String(bestAt);
        logNote.textContent = `Unfold set to ${bestAt.toFixed(2)} — read the trace to apply it.`;
      });
      foot.appendChild(take);
    }
    const why = document.createElement("span");
    why.className = "unfold-scan-note";
    why.textContent = scan.notes.join(" ");
    foot.appendChild(why);
    scanBox.appendChild(foot);
  };

  proposeBtn.addEventListener("click", () => {
    void (async () => {
      proposeBtn.disabled = true;
      logNote.textContent = "Scanning for a dip…";
      try {
        // The search width: whatever is typed, or a tenth of the trace's own depth range, which is
        // a dip steep enough to matter and shallow enough to leave most of every barrel intact.
        const typed = Number(unfoldIn.value);
        const width = Number.isFinite(typed) && typed !== 0 ? Math.abs(typed) * 2 : 0.5;
        const res = await extractCoreLog({
          well_id: well.well_id,
          dataset: dsSel.value,
          axis: axisPick.get() as "x" | "y",
          reverse: revChk.checked,
          lanes: Number(lanePick.get()) || 1,
          layouts,
          light: isUv() ? "uv" : "white",
          fluor: isUv() ? readClasses() : [],
          output_set: setPicker.outputSet(),
          unfold_scan: width,
          write: false,
        });
        if (res.unfold_scan) {
          drawScan(res.unfold_scan);
          logNote.textContent =
            res.unfold_scan.best !== null && res.unfold_scan.best !== undefined
              ? `Sharpest at ${res.unfold_scan.best.toFixed(2)}. ${res.unfold_scan.notes.join(" ")}`
              : res.unfold_scan.notes.join(" ");
        } else {
          logNote.textContent = "Nothing came back to scan.";
        }
      } catch (e) {
        logNote.textContent = String(e);
      } finally {
        proposeBtn.disabled = false;
      }
    })();
  });

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
        bumpDataVersion();
      } catch (e) {
        logNote.textContent = String(e);
      } finally {
        stripBtn.disabled = false;
      }
    })();
  });

  // ---- loading ------------------------------------------------------------
  async function reload(): Promise<void> {
    plates = await listWellImages(well!.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    await loadLayouts();
    filmstrip.load(plates);
    if (!plates.some((p) => p.image_id === current)) current = plates[0]?.image_id ?? "";
    filmstrip.mark(current);
    // A picture already laid out says so on its tile, so working through a delivery plate by plate
    // does not mean opening each one to find out which are done — the classifier's argument.
    filmstrip.annotate((p) => {
      const lay = layouts[p.image_id];
      if (!lay?.lanes.length) return null;
      const labelled = lay.lanes.filter((l) => l.depth_top != null && l.depth_base != null).length;
      return labelled === lay.lanes.length
        ? `${lay.lanes.length} barrels`
        : `${lay.lanes.length} cols`;
    });
    suggestTarget();
    await showPlate();
  }

  dsSel.addEventListener("change", () => {
    current = "";
    void reload();
  });
  await reload();

  return {
    el: wrap,
    dispose: () => {
      filmstrip.dispose();
      bitmap?.close();
      bitmap = null;
    },
  };
}
