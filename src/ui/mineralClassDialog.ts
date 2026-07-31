import {
  classifySupport,
  getWellImage,
  listDocuments,
  listImageDatasets,
  listWellImages,
  runPlateClassifier,
  saveDocument,
  type ClassifyResult,
  type ImageInfo,
  type PlateLabel,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { faciesColor } from "./plotCanvas";
import { formRow, openModal } from "./modal";

/** Mineral classifier from your own point counts (Petrophysics ▸ Petrography ▸ Mineral Classifier…).
 *
 *  Quartz against feldspar in plane light is not a colour problem, and any code claiming otherwise
 *  produces numbers with the shape of a modal analysis and none of the content. So there is no
 *  shipped model and there will not be one: the labels are yours, clicked on your own plates under
 *  your own lamp.
 *
 *  **Clicking IS the method, and it is the workflow you already have.** Point counting is a
 *  petrographer moving a stage and naming what is under the crosshair; this is the same act, and
 *  what it produces is training data rather than a tally.
 *
 *  **The labels are the artefact, not the model.** They are saved with the project and the model is
 *  refitted from them, seeded, on every run. A stored model blob cannot be read, argued with, or
 *  corrected; a list of clicks can be all three, and the answer stays reproducible from it.
 */
export async function openMineralClassDialog(): Promise<void> {
  const well = appState.selectedWell.get();
  const wrap = document.createElement("div");
  openModal(well ? `Mineral classifier — ${well.well_name}` : "Mineral classifier", wrap, 900);

  if (!well) {
    const none = document.createElement("div");
    none.className = "eq-note";
    none.textContent = "Select a well in the Wells pane first — plates are classified one well at a time.";
    wrap.appendChild(none);
    return;
  }

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Click on the plate to label what is under the pointer, the way you would point count. The " +
    "model is trained on those clicks and applied to the whole delivery. Nothing is shipped " +
    "pre-trained — the lamp, the white balance and the scanner are part of what it learns.";
  wrap.appendChild(intro);

  if (!(await classifySupport().catch(() => false))) {
    const warn = document.createElement("div");
    warn.className = "eq-note";
    warn.style.color = "var(--warn)";
    warn.textContent =
      "This needs numpy, Pillow, scipy and scikit-learn in the Python the app uses " +
      "(pip install numpy pillow scipy scikit-learn). Nothing else in the app is affected.";
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

  // ---- classes ------------------------------------------------------------
  let minerals: string[] = [];
  let active = 0;
  let labels: PlateLabel[] = [];

  const classBar = document.createElement("div");
  classBar.className = "mc-run-row";
  classBar.style.flexWrap = "wrap";
  wrap.appendChild(classBar);

  const newName = document.createElement("input");
  newName.className = "form-control";
  newName.placeholder = "Mineral name";
  newName.style.width = "10rem";
  const addBtn = document.createElement("button");
  addBtn.type = "button";
  addBtn.className = "btn";
  addBtn.textContent = "Add mineral";

  const countFor = (m: string): number => labels.filter((l) => l.mineral === m).length;

  const drawClasses = (): void => {
    classBar.textContent = "";
    minerals.forEach((m, i) => {
      const b = document.createElement("button");
      b.type = "button";
      b.className = i === active ? "btn btn-accent" : "btn";
      const sw = document.createElement("span");
      sw.style.cssText = `display:inline-block;width:10px;height:10px;border-radius:50%;margin-right:5px;background:${faciesColor(i)}`;
      b.appendChild(sw);
      b.appendChild(document.createTextNode(`${m} (${countFor(m)})`));
      b.addEventListener("click", () => {
        active = i;
        drawClasses();
      });
      classBar.appendChild(b);
    });
    classBar.appendChild(newName);
    classBar.appendChild(addBtn);
  };
  addBtn.addEventListener("click", () => {
    const n = newName.value.trim();
    if (!n || minerals.includes(n)) return;
    minerals.push(n);
    active = minerals.length - 1;
    newName.value = "";
    drawClasses();
  });

  // ---- the plate ----------------------------------------------------------
  const plateSel = document.createElement("select");
  plateSel.className = "form-control";
  let plates: ImageInfo[] = [];
  const stage = document.createElement("div");
  stage.style.cssText = "position:relative;display:inline-block;max-width:100%;margin:6px 0";
  const img = document.createElement("img");
  img.style.cssText = "max-width:100%;display:block;cursor:crosshair";
  const dots = document.createElement("div");
  dots.style.cssText = "position:absolute;inset:0;pointer-events:none";
  stage.append(img, dots);

  const drawDots = (): void => {
    dots.textContent = "";
    for (const l of labels) {
      if (l.image_id !== plateSel.value) continue;
      const d = document.createElement("div");
      const i = Math.max(0, minerals.indexOf(l.mineral));
      d.style.cssText =
        `position:absolute;left:${l.x * 100}%;top:${l.y * 100}%;width:9px;height:9px;` +
        `margin:-4.5px 0 0 -4.5px;border-radius:50%;border:1.5px solid #fff;background:${faciesColor(i)}`;
      dots.appendChild(d);
    }
  };

  img.addEventListener("click", (ev) => {
    if (!minerals.length) {
      setStatus("Add a mineral name first — a click has to be a label for something");
      return;
    }
    const r = img.getBoundingClientRect();
    // Stored as a FRACTION of the picture, so a label survives the display size, a window resize
    // and the stored copy having been resampled — the scale-bar argument again.
    const x = (ev.clientX - r.left) / r.width;
    const y = (ev.clientY - r.top) / r.height;
    if (x < 0 || x > 1 || y < 0 || y > 1) return;
    labels.push({ image_id: plateSel.value, x, y, mineral: minerals[active] });
    drawDots();
    drawClasses();
  });

  let objectUrl: string | null = null;
  const loadPlate = async (): Promise<void> => {
    const id = plateSel.value;
    if (!id) return;
    const info = plates.find((p) => p.image_id === id);
    try {
      const buf = await getWellImage(id);
      // Revoked on the next load: a delivery of three hundred plates clicked through would
      // otherwise hold every one of them alive for the life of the dialog.
      if (objectUrl) URL.revokeObjectURL(objectUrl);
      objectUrl = URL.createObjectURL(new Blob([buf], { type: info?.mime ?? "image/jpeg" }));
      img.src = objectUrl;
    } catch {
      img.removeAttribute("src");
    }
    drawDots();
  };

  const loadPlates = async (): Promise<void> => {
    plates = await listWellImages(well.well_id, dsSel.value).catch(() => [] as ImageInfo[]);
    plateSel.textContent = "";
    for (const p of plates) {
      const o = document.createElement("option");
      o.value = p.image_id;
      o.textContent = `${p.name} @ ${p.depth_top}`;
      plateSel.appendChild(o);
    }
    if (plates.length) plateSel.value = plates[0].image_id;
    await loadPlate();
  };

  // Labels persist with the project, keyed by well and delivery.
  const docName = (): string => `${well.well_id}/${dsSel.value}`;
  const loadLabels = async (): Promise<void> => {
    const docs = await listDocuments("platelabels").catch(() => []);
    const mine = docs.find((d) => d.name === docName());
    labels = [];
    minerals = [];
    if (mine) {
      try {
        const parsed = JSON.parse(mine.json) as { minerals: string[]; labels: PlateLabel[] };
        minerals = parsed.minerals ?? [];
        labels = parsed.labels ?? [];
      } catch {
        /* a corrupt document starts empty rather than stopping the dialog */
      }
    }
    active = 0;
    drawClasses();
    drawDots();
  };

  dsSel.addEventListener("change", () => {
    void loadPlates().then(loadLabels);
  });
  plateSel.addEventListener("change", () => void loadPlate());
  wrap.appendChild(formRow("Label on plate", plateSel, "Click anywhere on the picture to place a label."));
  wrap.appendChild(stage);
  await loadPlates();
  await loadLabels();

  const tools = document.createElement("div");
  tools.className = "mc-run-row";
  const undoBtn = document.createElement("button");
  undoBtn.type = "button";
  undoBtn.className = "btn";
  undoBtn.textContent = "Undo last label";
  undoBtn.addEventListener("click", () => {
    labels.pop();
    drawDots();
    drawClasses();
  });
  const clearBtn = document.createElement("button");
  clearBtn.type = "button";
  clearBtn.className = "btn";
  clearBtn.textContent = "Clear this plate";
  clearBtn.addEventListener("click", () => {
    labels = labels.filter((l) => l.image_id !== plateSel.value);
    drawDots();
    drawClasses();
  });
  const saveLabelsBtn = document.createElement("button");
  saveLabelsBtn.type = "button";
  saveLabelsBtn.className = "btn";
  saveLabelsBtn.textContent = "Save labels";
  saveLabelsBtn.addEventListener("click", () => {
    void saveDocument("platelabels", docName(), JSON.stringify({ minerals, labels })).then(() =>
      setStatus(`${labels.length} label(s) saved with the project`)
    );
  });
  tools.append(undoBtn, clearBtn, saveLabelsBtn);
  wrap.appendChild(tools);

  const setIn = document.createElement("input");
  setIn.className = "form-control";
  setIn.value = "CLS";
  wrap.appendChild(
    formRow("Save results as delivery", setIn, "Stored as point data under CLS_<mineral> in the PETROGRAPHY dataset.")
  );

  // ---- run ----------------------------------------------------------------
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.className = "btn btn-accent";
  runBtn.textContent = "Train and apply";
  const saveBtn = document.createElement("button");
  saveBtn.type = "button";
  saveBtn.className = "btn";
  saveBtn.textContent = "Train, apply and save";
  const status = document.createElement("div");
  status.className = "mc-status";
  runRow.append(runBtn, saveBtn, status);
  wrap.appendChild(runRow);

  const out = document.createElement("div");
  out.className = "mc-results";
  wrap.appendChild(out);

  const render = (res: ClassifyResult): void => {
    out.textContent = "";

    // The accuracy goes FIRST and per class, because an overall number can sit comfortably on top
    // of one mineral the model cannot see at all.
    const perf = document.createElement("table");
    perf.className = "mc-table";
    const ph = document.createElement("tr");
    for (const h of ["Mineral", "Clicks", "Held-out recall"]) {
      const th = document.createElement("th");
      th.textContent = h;
      ph.appendChild(th);
    }
    perf.appendChild(ph);
    for (const c of res.per_class) {
      const tr = document.createElement("tr");
      for (const v of [c.mineral, String(c.clicks), c.recall < 0 ? "not checked" : c.recall.toFixed(2)]) {
        const td = document.createElement("td");
        td.textContent = v;
        tr.appendChild(td);
      }
      if (c.recall >= 0 && c.recall < 0.7) (tr as HTMLElement).style.color = "var(--warn)";
      perf.appendChild(tr);
    }
    out.appendChild(perf);

    const acc = document.createElement("div");
    acc.className = "eq-note";
    acc.textContent = Number.isFinite(res.accuracy)
      ? `Overall held-out accuracy ${(res.accuracy * 100).toFixed(0)}%, cross-validated by click.`
      : "Not enough clicks per mineral to check the model.";
    out.appendChild(acc);

    if (res.plates.length) {
      const names: string[] = [];
      for (const p of res.plates) for (const [m] of p.fractions) if (!names.includes(m)) names.push(m);
      const tbl = document.createElement("table");
      tbl.className = "data-table";
      const head = document.createElement("tr");
      for (const h of ["Plate", "Depth", ...names]) {
        const th = document.createElement("th");
        th.textContent = h;
        head.appendChild(th);
      }
      tbl.appendChild(head);
      for (const p of res.plates) {
        const tr = document.createElement("tr");
        const vals = [p.name, String(p.depth_top)];
        for (const n of names) {
          const f = p.fractions.find(([m]) => m === n)?.[1];
          vals.push(f == null ? "" : `${(f * 100).toFixed(1)}%`);
        }
        for (const v of vals) {
          const td = document.createElement("td");
          td.textContent = v;
          tr.appendChild(td);
        }
        tbl.appendChild(tr);
      }
      out.appendChild(tbl);
    }

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
  };

  const go = async (save: boolean): Promise<void> => {
    runBtn.disabled = true;
    saveBtn.disabled = true;
    status.textContent = "Training…";
    try {
      const res = await runPlateClassifier({
        well_id: well.well_id,
        dataset: dsSel.value,
        labels,
        set_name: save ? setIn.value || "CLS" : null,
        preview_image_id: null,
      });
      render(res);
      status.textContent = `${res.plates.length} plate(s)`;
      if (save && res.written) {
        const [ds, name] = res.written;
        setStatus(`Saved ${res.plates.length} classified plate(s) as ${ds} / ${name}`);
        recordProcess("Edit", `Mineral classifier on ${dsSel.value}: ${res.plates.length} plate(s) → ${ds}/${name}`, well.well_name);
        bumpDataVersion();
      }
    } catch (e) {
      out.textContent = "";
      const err = document.createElement("div");
      err.className = "eq-note";
      err.style.color = "var(--warn)";
      err.textContent = String(e);
      out.appendChild(err);
      status.textContent = "";
    } finally {
      runBtn.disabled = false;
      saveBtn.disabled = false;
    }
  };
  runBtn.addEventListener("click", () => void go(false));
  saveBtn.addEventListener("click", () => void go(true));

  drawClasses();
}
