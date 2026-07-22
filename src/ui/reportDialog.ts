import { open, save } from "@tauri-apps/plugin-dialog";
import {
  exportReportBatch,
  exportReportPdf,
  listDocuments,
  listLayouts,
  renderReport,
  saveDocument,
  savePng,
  type CompositeResult,
  type Layout,
  type MethodRow,
  type PageSize,
  type ReportSpec,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion } from "../state";
import { formRow } from "./modal";
import { buildWellScope } from "./wellScope";

const TEMPLATE_DOC_TYPE = "report_template";
const TEMPLATE_NAME = "default";

/** Serializes methodology rows to the editable "Parameter | Method | Remarks" lines. */
function rowsToText(rows: MethodRow[]): string {
  return rows.map((r) => `${r.parameter} | ${r.method} | ${r.remarks}`).join("\n");
}

function textToRows(text: string): MethodRow[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
    .map((l) => {
      const [parameter = "", method = "", ...rest] = l.split("|").map((s) => s.trim());
      return { parameter, method, remarks: rest.join(" | ") };
    });
}

/** Report generator (Phase 8b): cover + methodology table + zone parameters +
 *  pay summary + composite pages → one PDF, previewed page by page. The methodology
 *  table is editable and persists as a `report_template` document.
 *  Hosted as a dock pane (workspace component "report") that follows the selected
 *  well, not a popup. */
export async function buildReportContent(
  well: WellSummary,
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const builtins = await listLayouts().catch(() => [] as Layout[]);
  const active = appState.activeLayout.get();
  const layouts: Layout[] = [...builtins];
  if (active && !layouts.some((l) => l.name === active.name)) layouts.unshift(active);
  // Batch export runs over the scope selector (group / ★ pinned / selection / all); the single
  // Render/PDF/PNG buttons still target this pane's own well. The Batch button's count tracks
  // the live scope.
  const scope = await buildWellScope({
    onChange: (ids) => {
      batchBtn.textContent = `Batch (${ids.length} wells)…`;
    },
  });

  const content = document.createElement("div");
  content.className = "report-pane";

  const titleIn = document.createElement("input");
  titleIn.className = "form-control";
  titleIn.value = `Petrophysical Evaluation — ${well.field_name ?? well.well_name}`;
  const authorIn = document.createElement("input");
  authorIn.className = "form-control";
  authorIn.placeholder = "optional";

  const layoutSel = document.createElement("select");
  layoutSel.className = "form-control";
  for (const l of layouts) {
    const o = document.createElement("option");
    o.value = l.name;
    o.textContent = l.name;
    layoutSel.appendChild(o);
  }
  if (active) layoutSel.value = active.name;

  const scaleSel = document.createElement("select");
  scaleSel.className = "form-control";
  for (const sc of [200, 500, 1000]) {
    const o = document.createElement("option");
    o.value = String(sc);
    o.textContent = `1:${sc}`;
    if (sc === 500) o.selected = true;
    scaleSel.appendChild(o);
  }
  const pageSel = document.createElement("select");
  pageSel.className = "form-control";
  for (const [val, label] of [
    ["a4", "A4 (210×297)"],
    ["a3", "A3 (297×420)"],
    ["letter", "Letter (216×279)"],
  ] as const) {
    const o = document.createElement("option");
    o.value = val;
    o.textContent = label;
    pageSel.appendChild(o);
  }
  const scalePageWrap = document.createElement("div");
  scalePageWrap.style.display = "flex";
  scalePageWrap.style.gap = "6px";
  scalePageWrap.appendChild(scaleSel);
  scalePageWrap.appendChild(pageSel);

  // Cutoffs (pay-summary convention).
  const cutoffWrap = document.createElement("div");
  cutoffWrap.style.display = "flex";
  cutoffWrap.style.gap = "6px";
  const mkNum = (value: string, title: string): HTMLInputElement => {
    const el = document.createElement("input");
    el.type = "number";
    el.step = "any";
    el.className = "form-control";
    el.value = value;
    el.title = title;
    return el;
  };
  const vshIn = mkNum("0.5", "VSH ≤ (sand)");
  const phieIn = mkNum("0.1", "PHIE ≥ (reservoir)");
  const sweIn = mkNum("0.6", "SWE ≤ (pay)");
  const permIn = mkNum("", "PERM ≥ mD (optional)");
  permIn.placeholder = "PERM (off)";
  cutoffWrap.appendChild(vshIn);
  cutoffWrap.appendChild(phieIn);
  cutoffWrap.appendChild(sweIn);
  cutoffWrap.appendChild(permIn);

  const tablesOnly = document.createElement("input");
  tablesOnly.type = "checkbox";

  // Methodology editor with template persistence.
  const methodTa = document.createElement("textarea");
  methodTa.className = "form-control";
  methodTa.rows = 6;
  methodTa.placeholder = "One row per line: Parameter | Method | Remarks\n(blank = built-in default template)";
  methodTa.classList.add("mono-input");
  try {
    const docs = await listDocuments(TEMPLATE_DOC_TYPE);
    const tpl = docs.find((d) => d.name === TEMPLATE_NAME);
    if (tpl) methodTa.value = rowsToText(JSON.parse(tpl.json) as MethodRow[]);
  } catch {
    /* no template yet */
  }
  const saveTplBtn = document.createElement("button");
  saveTplBtn.className = "form-run-btn";
  saveTplBtn.textContent = "Save Template";
  saveTplBtn.addEventListener("click", async () => {
    try {
      await saveDocument(TEMPLATE_DOC_TYPE, TEMPLATE_NAME, JSON.stringify(textToRows(methodTa.value)));
      status.textContent = "Methodology template saved.";
    } catch (err) {
      status.textContent = `Template save failed: ${err}`;
    }
  });

  content.appendChild(formRow("Study title", titleIn));
  content.appendChild(formRow("Prepared by", authorIn));
  content.appendChild(formRow("Layout", layoutSel));
  content.appendChild(formRow("Scale / page", scalePageWrap));
  content.appendChild(formRow("Cutoffs VSH/PHIE/SWE/PERM", cutoffWrap, "Pay summary flags: VSH ≤, PHIE ≥, SWE ≤, PERM ≥ (blank = off)"));
  content.appendChild(formRow("Tables only (no composite)", tablesOnly));
  content.appendChild(formRow("Methodology table", methodTa, "Parameter | Method | Remarks per line; blank = default"));

  // Scope for the Batch export below (Render/PDF/PNG act on this pane's single well).
  content.appendChild(scope.el);

  const btnRow = document.createElement("div");
  btnRow.className = "pick-row";
  const renderBtn = document.createElement("button");
  renderBtn.className = "form-run-btn";
  renderBtn.textContent = "Render";
  const pdfBtn = document.createElement("button");
  pdfBtn.className = "form-run-btn";
  pdfBtn.textContent = "Save PDF…";
  pdfBtn.disabled = true;
  const pngBtn = document.createElement("button");
  pngBtn.className = "form-run-btn";
  pngBtn.textContent = "Save PNG (page)…";
  pngBtn.disabled = true;
  const batchBtn = document.createElement("button");
  batchBtn.className = "form-run-btn";
  batchBtn.textContent = `Batch (${scope.getWellIds().length} wells)…`;
  btnRow.appendChild(renderBtn);
  btnRow.appendChild(pdfBtn);
  btnRow.appendChild(pngBtn);
  btnRow.appendChild(saveTplBtn);
  btnRow.appendChild(batchBtn);
  content.appendChild(btnRow);

  const status = document.createElement("div");
  status.className = "modal-result";
  content.appendChild(status);

  const nav = document.createElement("div");
  nav.className = "pick-row";
  nav.style.display = "none";
  const prevBtn = document.createElement("button");
  prevBtn.className = "ribbon-stepbtn";
  prevBtn.textContent = "◀";
  const pageLabel = document.createElement("span");
  pageLabel.className = "pick-label";
  const nextBtn = document.createElement("button");
  nextBtn.className = "ribbon-stepbtn";
  nextBtn.textContent = "▶";
  nav.appendChild(prevBtn);
  nav.appendChild(pageLabel);
  nav.appendChild(nextBtn);
  content.appendChild(nav);

  const preview = document.createElement("div");
  preview.className = "composite-preview";
  content.appendChild(preview);

  let result: CompositeResult | null = null;
  let pageIdx = 0;

  const buildSpec = (): ReportSpec | string => {
    const layout = layouts.find((l) => l.name === layoutSel.value);
    if (!layout) return "No layout selected.";
    const vsh = Number(vshIn.value);
    const phie = Number(phieIn.value);
    const swe = Number(sweIn.value);
    if ([vsh, phie, swe].some(Number.isNaN)) return "Cutoffs must be numbers.";
    const perm = permIn.value.trim() === "" ? null : Number(permIn.value);
    if (perm !== null && Number.isNaN(perm)) return "PERM cutoff must be a number or blank.";
    return {
      composite: {
        well_id: well.well_id,
        layout,
        depth_top: null,
        depth_bottom: null,
        scale: Number(scaleSel.value),
        page_size: pageSel.value as PageSize,
      },
      title: titleIn.value.trim() || "Petrophysical Evaluation",
      author: authorIn.value.trim(),
      methodology: textToRows(methodTa.value),
      vsh_max: vsh,
      phie_min: phie,
      swe_max: swe,
      perm_min: perm,
      tables_only: tablesOnly.checked,
    };
  };

  const showPage = () => {
    if (!result) return;
    preview.innerHTML = result.pages[pageIdx].svg;
    const svgEl = preview.querySelector("svg");
    if (svgEl) {
      svgEl.removeAttribute("width");
      svgEl.setAttribute("height", "auto");
      (svgEl as SVGElement).style.width = "100%";
      (svgEl as SVGElement).style.height = "auto";
      (svgEl as SVGElement).style.border = "1px solid var(--border, #ccc)";
    }
    pageLabel.textContent = `Page ${pageIdx + 1} / ${result.pages.length}`;
    prevBtn.disabled = pageIdx === 0;
    nextBtn.disabled = pageIdx === result.pages.length - 1;
  };

  prevBtn.addEventListener("click", () => {
    if (pageIdx > 0) {
      pageIdx--;
      showPage();
    }
  });
  nextBtn.addEventListener("click", () => {
    if (result && pageIdx < result.pages.length - 1) {
      pageIdx++;
      showPage();
    }
  });

  renderBtn.addEventListener("click", async () => {
    const spec = buildSpec();
    if (typeof spec === "string") {
      status.textContent = spec;
      return;
    }
    renderBtn.disabled = true;
    status.textContent = "Rendering report…";
    try {
      result = await renderReport(spec);
      pageIdx = 0;
      nav.style.display = result.pages.length > 1 ? "flex" : "none";
      pdfBtn.disabled = false;
      pngBtn.disabled = false;
      status.textContent = `${result.well_name}: ${result.pages.length} report page(s).`;
      showPage();
      // Rendering a report writes FLAG_SAND/RESERVOIR/PAY in place (its pay-summary pass), so
      // refresh any open plots/log views showing those flag curves.
      bumpDataVersion();
    } catch (err) {
      status.textContent = `Render failed: ${err}`;
      result = null;
      pdfBtn.disabled = true;
      pngBtn.disabled = true;
    } finally {
      renderBtn.disabled = false;
    }
  });

  pdfBtn.addEventListener("click", async () => {
    const spec = buildSpec();
    if (typeof spec === "string") {
      status.textContent = spec;
      return;
    }
    let dest: string | null;
    try {
      dest = await save({
        defaultPath: `${well.well_name}_report.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
    } catch (err) {
      status.textContent = `Save dialog unavailable: ${err}`;
      return;
    }
    if (!dest) return;
    pdfBtn.disabled = true;
    status.textContent = "Writing PDF…";
    try {
      const path = await exportReportPdf(spec, dest);
      status.textContent = `Wrote ${path.split(/[\\/]/).pop()}`;
      setStatus(`Report PDF exported for ${well.well_name}.`);
      bumpDataVersion(); // the export's pay-summary pass writes FLAG curves in place

    } catch (err) {
      status.textContent = `Report export failed: ${err}`;
    } finally {
      pdfBtn.disabled = false;
    }
  });

  // PNG of the CURRENT preview page, rasterized at ~150 dpi for slides.
  pngBtn.addEventListener("click", async () => {
    if (!result) return;
    let dest: string | null;
    try {
      dest = await save({
        defaultPath: `${well.well_name}_report_p${pageIdx + 1}.png`,
        filters: [{ name: "PNG", extensions: ["png"] }],
      });
    } catch (err) {
      status.textContent = `Save dialog unavailable: ${err}`;
      return;
    }
    if (!dest) return;
    pngBtn.disabled = true;
    status.textContent = "Rasterizing PNG…";
    try {
      const pxPerMm = 150 / 25.4;
      const w = Math.round(result.page_width_mm * pxPerMm);
      const h = Math.round(result.page_height_mm * pxPerMm);
      const img = new Image();
      const svgBlob = new Blob([result.pages[pageIdx].svg], { type: "image/svg+xml" });
      const url = URL.createObjectURL(svgBlob);
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error("SVG rasterization failed"));
        img.src = url;
      });
      const canvas = document.createElement("canvas");
      canvas.width = w;
      canvas.height = h;
      const g = canvas.getContext("2d");
      if (!g) throw new Error("no 2d context");
      g.fillStyle = "#ffffff";
      g.fillRect(0, 0, w, h);
      g.drawImage(img, 0, 0, w, h);
      URL.revokeObjectURL(url);
      const base64 = canvas.toDataURL("image/png").split(",")[1];
      const path = await savePng(dest, base64);
      status.textContent = `Wrote ${path.split(/[\\/]/).pop()}`;
      setStatus(`Report page PNG exported for ${well.well_name}.`);
    } catch (err) {
      status.textContent = `PNG export failed: ${err}`;
    } finally {
      pngBtn.disabled = false;
    }
  });

  batchBtn.addEventListener("click", async () => {
    const spec = buildSpec();
    if (typeof spec === "string") {
      status.textContent = spec;
      return;
    }
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      status.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return;
    }
    let dir: string | null;
    try {
      const picked = await open({ directory: true, multiple: false });
      dir = typeof picked === "string" ? picked : null;
    } catch (err) {
      status.textContent = `Folder dialog unavailable: ${err}`;
      return;
    }
    if (!dir) return;
    batchBtn.disabled = true;
    status.textContent = `Exporting ${wellIds.length} report(s)…`;
    try {
      const paths = await exportReportBatch(spec, wellIds, dir);
      status.textContent = `Wrote ${paths.length} report PDF(s) to ${dir}`;
      setStatus(`Batch report export: ${paths.length} well(s).`);
      bumpDataVersion(); // batch reports write FLAG curves per well — refresh open views
    } catch (err) {
      status.textContent = `Batch export: ${err}`;
    } finally {
      batchBtn.disabled = false;
    }
  });

  return { el: content, dispose: () => scope.dispose() };
}
