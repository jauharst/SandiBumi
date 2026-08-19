import {
  listPlugChoices,
  runPlugQc,
  type PlugChoice,
  type PlugQcRequest,
  type PlugQcResult,
  type PlugSource,
} from "../ipc";
import { recordProcess } from "../processLog";
import { buildWellScope } from "./wellScope";
import { buildFitScatter, type FitScatter, type FitScatterLine } from "./fitScatter";
import { formRow } from "./modal";

/** Plug QC — two measurements of the same plug, plotted against each other.
 *
 *  The petrography measurements are numbers nobody has yet checked against anything independent. A
 *  thin-section pore area estimating a volume fraction is a claim; the way to find out whether it
 *  holds on this rock is to put it beside the helium porosity of the plug the section was cut from.
 *  Same for a pore-body diameter beside the throat radii the capillary-pressure curve reports.
 *
 *  Three choices in here are petrophysical rather than cosmetic.
 *
 *  **The reference line defaults to NONE.** A 1:1 line asserts the two axes are the same quantity.
 *  For a pore body against a pore throat that is false by construction — bodies are always larger
 *  than the throats that drain them — so every point would sit below the line and read as a
 *  disagreement when it is the physics. The user turns the line on when they mean it.
 *
 *  **Both correlations are shown, and Spearman is the one that survives a log axis.** Pearson asks
 *  "is this a straight line", Spearman asks "do they move together". Switching an axis to log
 *  changes the picture and changes nothing about Spearman, which is exactly the property wanted
 *  from the number quoted beside it.
 *
 *  **The medians are on the table.** Nothing here converts a unit, because point data is stored
 *  verbatim — so a core porosity delivered in percent beside a thin-section fraction shows up as
 *  18.2 against 0.19, which is a unit mismatch the user can see rather than one that quietly ruins
 *  a 1:1 comparison.
 */
export async function buildPlugQcContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const content = document.createElement("div");
  // A pane, so it takes the pane form treatment (module-pane + the .module-args
  // grid below) rather than the dialog-era side-label rows it was built with:
  // a fixed 180px label column squeezed against a full-width control reads as
  // clipped labels next to enormous selects at pane widths.
  content.className = "module-pane";

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Pairs two measurements made on the same plug and plots one against the other. Samples are " +
    "matched by depth within the tolerance below; a sample with no partner inside it is dropped " +
    "and counted, never snapped to the nearest one.";
  content.appendChild(intro);

  // ---- scope --------------------------------------------------------------
  let choices: PlugChoice[] = [];
  const runBtn = document.createElement("button");
  runBtn.type = "button";
  runBtn.classList.add("primary");

  const xSel = document.createElement("select");
  const ySel = document.createElement("select");
  xSel.className = "form-control";
  ySel.className = "form-control";

  /** Refills both pickers from what the scoped wells actually hold, keeping the current picks. */
  const reloadChoices = async (ids: string[]): Promise<void> => {
    choices = ids.length ? await listPlugChoices(scope.backend()).catch(() => [] as PlugChoice[]) : [];
    for (const sel of [xSel, ySel]) {
      const had = sel.value;
      sel.textContent = "";
      if (!choices.length) {
        const o = document.createElement("option");
        o.value = "";
        o.textContent = "(no plug measurements in the scoped wells)";
        o.disabled = true;
        o.selected = true;
        sel.appendChild(o);
        continue;
      }
      for (const c of choices) {
        const o = document.createElement("option");
        o.value = keyOf(c);
        o.textContent = c.label + (c.wells > 1 ? ` — ${c.wells} wells` : "");
        sel.appendChild(o);
      }
      if (choices.some((c) => keyOf(c) === had)) sel.value = had;
    }
    // A first-run default that answers the question this pane was built for: the section against
    // the plug. Only applied when nothing has been picked yet, so it never overrides the user.
    if (choices.length && !xSel.dataset.touched) {
      const core = choices.find((c) => c.kind === "core" && c.item.toUpperCase() === "CPOR");
      const ts = choices.find((c) => c.item.toUpperCase() === "VPORE_TS");
      if (core) xSel.value = keyOf(core);
      if (ts) ySel.value = keyOf(ts);
    }
    paintRunBtn();
  };

  const scope = await buildWellScope({
    onChange: (ids) => {
      void reloadChoices(ids);
    },
  });
  content.appendChild(scope.el);
  const wellName = (id: string): string => scope.namesFor([id])[0] ?? id;

  const paintRunBtn = (): void => {
    const n = scope.getWellIds().length;
    runBtn.textContent = `Compare across ${n} well(s)`;
    runBtn.disabled = n === 0 || choices.length === 0;
  };

  // ---- the two axes -------------------------------------------------------
  xSel.addEventListener("change", () => {
    xSel.dataset.touched = "1";
  });
  ySel.addEventListener("change", () => {
    xSel.dataset.touched = "1";
  });
  const args = document.createElement("div");
  args.className = "module-args";
  content.appendChild(args);
  args.appendChild(formRow("X measurement", xSel, "Read from the ACTIVE delivery of each store"));
  args.appendChild(formRow("Y measurement", ySel, "Read from the ACTIVE delivery of each store"));

  // Only meaningful when one of the axes is the SCAL throat radius, so it hides itself otherwise.
  const satIn = document.createElement("input");
  satIn.type = "number";
  satIn.className = "form-control";
  satIn.step = "1";
  satIn.min = "1";
  satIn.max = "99";
  satIn.value = "35";
  const satRow = formRow(
    "Mercury saturation (%)",
    satIn,
    "The saturation the throat radius is read at. 35% is the Kolodzie/Winland r35 convention, which is what the R35 curve predicts.",
  );
  args.appendChild(satRow);

  const tolIn = document.createElement("input");
  tolIn.type = "number";
  tolIn.className = "form-control";
  tolIn.step = "0.01";
  tolIn.value = "0.15";
  args.appendChild(
    formRow(
      "Depth tolerance",
      tolIn,
      "One standard 6-inch sample. If the two deliveries disagree by more than this, register the core rather than widening it.",
    ),
  );

  const lineSel = document.createElement("select");
  lineSel.className = "form-control";
  for (const [v, label] of [
    ["none", "None — two different quantities"],
    ["identity", "1:1 — the same quantity measured twice"],
  ] as const) {
    const o = document.createElement("option");
    o.value = v;
    o.textContent = label;
    lineSel.appendChild(o);
  }
  args.appendChild(
    formRow(
      "Reference line",
      lineSel,
      "A 1:1 line claims the axes are the same quantity. Pore bodies against pore throats are not.",
    ),
  );

  const logX = document.createElement("input");
  logX.type = "checkbox";
  const logY = document.createElement("input");
  logY.type = "checkbox";
  const axisRow = document.createElement("div");
  axisRow.className = "mc-run-row";
  for (const [box, label] of [
    [logX, "Log X"],
    [logY, "Log Y"],
  ] as const) {
    const l = document.createElement("label");
    l.className = "mc-check";
    l.appendChild(box);
    l.appendChild(document.createTextNode(` ${label}`));
    axisRow.appendChild(l);
  }
  content.appendChild(axisRow);

  const statusLine = document.createElement("div");
  statusLine.className = "mc-status";
  const runRow = document.createElement("div");
  runRow.className = "mc-run-row";
  runRow.append(runBtn, statusLine);
  content.appendChild(runRow);

  const results = document.createElement("div");
  results.className = "mc-results";
  content.appendChild(results);

  await reloadChoices(scope.getWellIds());

  const syncSatRow = (): void => {
    const scal = sourceOf(xSel.value).kind === "scal_throat" || sourceOf(ySel.value).kind === "scal_throat";
    satRow.hidden = !scal;
  };
  syncSatRow();
  xSel.addEventListener("change", syncSatRow);
  ySel.addEventListener("change", syncSatRow);

  // ---- run ----------------------------------------------------------------
  let scatter: FitScatter | null = null;
  let last: PlugQcResult | null = null;
  const dropScatter = (): void => {
    scatter?.dispose();
    scatter = null;
  };

  const num = (el: HTMLInputElement, fallback: number): number => {
    const n = Number(el.value.trim());
    return Number.isFinite(n) ? n : fallback;
  };

  const render = (r: PlugQcResult): void => {
    last = r;
    dropScatter();
    results.textContent = "";

    const f = (v: number, d = 3): string => (Number.isFinite(v) ? v.toFixed(d) : "—");
    const rows: [string, string][] = [
      ["Pairs / wells", `${r.n_pairs} / ${r.n_wells}`],
      ["Pearson r (straight line)", f(r.pearson)],
      ["Spearman ρ (rank)", f(r.spearman)],
      ["Median X", f(r.x_median, 4)],
      ["Median Y", f(r.y_median, 4)],
    ];
    const tbl = document.createElement("table");
    tbl.className = "mc-table";
    for (const [k, v] of rows) {
      const tr = document.createElement("tr");
      const td1 = document.createElement("td");
      td1.textContent = k;
      const td2 = document.createElement("td");
      td2.textContent = v;
      td2.style.textAlign = "right";
      tr.append(td1, td2);
      tbl.appendChild(tr);
    }
    results.appendChild(tbl);

    if (r.points.length) {
      const line: FitScatterLine = lineSel.value === "identity" ? { kind: "identity" } : { kind: "none" };
      scatter = buildFitScatter({
        points: r.points.map((p) => ({
          x: p.x,
          y: p.y,
          group: wellName(p.well_id),
          detail: [
            `${wellName(p.well_id)} @ ${p.x_depth.toFixed(2)}`,
            `X ${p.x.toPrecision(4)}   Y ${p.y.toPrecision(4)}`,
            // Only worth a line when the two deliveries do not sit on the same depth: that gap is
            // a registration question and it is invisible from the values alone.
            Math.abs(p.x_depth - p.y_depth) > 1e-4
              ? `paired across ${(p.y_depth - p.x_depth).toFixed(3)}`
              : "same depth",
          ],
        })),
        xLabel: r.x_label,
        yLabel: r.y_label,
        line,
        logX: logX.checked,
        logY: logY.checked,
        caption: `${r.n_pairs} paired plug(s) from ${r.n_wells} well(s)`,
        exportName: "plug-qc",
      });
      results.appendChild(scatter.el);
      scatter.redraw(); // first paint is synchronous once it is in the document
    }

    if (r.excluded.length) {
      const ex = document.createElement("div");
      ex.className = "eq-note";
      ex.textContent = "Not paired — " + r.excluded.map(([why, n]) => `${n} ${why}`).join("; ");
      results.appendChild(ex);
    }
    for (const n of r.notes) {
      const d = document.createElement("div");
      d.className = "eq-note";
      d.textContent = n;
      results.appendChild(d);
    }
  };

  // The line and the axis scales are DISPLAY choices — redrawing from the pairs already in hand
  // beats a round trip, and re-pairing would be the same answer arrived at more slowly.
  for (const el of [lineSel, logX, logY]) {
    el.addEventListener("change", () => {
      if (last) render(last);
    });
  }

  runBtn.addEventListener("click", () => {
    const req: PlugQcRequest = {
      well_ids: scope.getWellIds(),
      x: sourceOf(xSel.value, num(satIn, 35) / 100),
      y: sourceOf(ySel.value, num(satIn, 35) / 100),
      depth_tol: num(tolIn, 0.15),
    };
    runBtn.disabled = true;
    statusLine.textContent = "Pairing…";
    void runPlugQc(req, scope.backend())
      .then((r) => {
        render(r);
        statusLine.textContent = `${r.n_pairs} pair(s)`;
        setStatus(`Plug QC: ${r.n_pairs} paired plug(s) from ${r.n_wells} well(s)`);
        recordProcess("qc", `Plug QC: ${r.x_label} vs ${r.y_label} — ${r.n_pairs} pair(s)`);
      })
      .catch((e) => {
        statusLine.textContent = String(e);
      })
      .finally(paintRunBtn);
  });

  return {
    el: content,
    dispose: () => {
      dropScatter();
      scope.dispose();
    },
  };
}

/** Stable select value for one choice. Split on a control character, never a space: dataset names
 *  come from the user's own imports and "THIN SECTION" is a perfectly ordinary one. */
const SEP = "\u0001";

function keyOf(c: PlugChoice): string {
  return [c.kind, c.dataset, c.item].join(SEP);
}

function sourceOf(key: string, saturation = 0.35): PlugSource {
  const [kind, dataset, item] = key.split(SEP);
  return { kind: kind ?? "", dataset: dataset ?? "", item: item ?? "", saturation };
}
