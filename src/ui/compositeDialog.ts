import { save } from "@tauri-apps/plugin-dialog";
import {
  exportCompositePdf,
  exportCompositeSvg,
  listLayouts,
  renderComposite,
  type CompositeResult,
  type CompositeSpec,
  type Layout,
  type PageSize,
  type WellSummary,
} from "../ipc";
import { appState } from "../state";
import { formRow } from "./modal";

/** Composite log deliverable: pick a layout, print scale, page size and depth
 *  window, preview the rendered vector pages, and export to SVG. The composite is
 *  rendered in Rust (`render_composite`) at a physically exact print scale.
 *  Hosted as a dock pane (workspace component "composite") that follows the
 *  selected well, not a popup. */
export async function buildCompositeContent(
  well: WellSummary,
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  // Layout choices: built-ins plus the currently active layout (deduped by name).
  const builtins = await listLayouts().catch(() => [] as Layout[]);
  const active = appState.activeLayout.get();
  const layouts: Layout[] = [...builtins];
  if (active && !layouts.some((l) => l.name === active.name)) layouts.unshift(active);

  const content = document.createElement("div");
  content.className = "composite-pane";

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

  const topIn = document.createElement("input");
  topIn.type = "number";
  topIn.step = "any";
  topIn.className = "form-control";
  topIn.placeholder = "full";
  const botIn = document.createElement("input");
  botIn.type = "number";
  botIn.step = "any";
  botIn.className = "form-control";
  botIn.placeholder = "full";
  const rangeWrap = document.createElement("div");
  rangeWrap.style.display = "flex";
  rangeWrap.style.gap = "6px";
  rangeWrap.appendChild(topIn);
  rangeWrap.appendChild(botIn);

  content.appendChild(formRow("Layout", layoutSel));
  content.appendChild(formRow("Print scale", scaleSel));
  content.appendChild(formRow("Page size", pageSel));
  content.appendChild(formRow("Depth top / bottom (m)", rangeWrap, "Blank = full logged interval"));

  const btnRow = document.createElement("div");
  btnRow.className = "pick-row";
  const renderBtn = document.createElement("button");
  renderBtn.className = "form-run-btn";
  renderBtn.textContent = "Render";
  const saveBtn = document.createElement("button");
  saveBtn.className = "form-run-btn";
  saveBtn.textContent = "Save SVG…";
  saveBtn.disabled = true;
  const pdfBtn = document.createElement("button");
  pdfBtn.className = "form-run-btn";
  pdfBtn.textContent = "Save PDF…";
  pdfBtn.disabled = true;
  btnRow.appendChild(renderBtn);
  btnRow.appendChild(saveBtn);
  btnRow.appendChild(pdfBtn);
  content.appendChild(btnRow);

  const status = document.createElement("div");
  status.className = "modal-result";
  content.appendChild(status);

  // Page navigation + preview surface.
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

  const buildSpec = (): CompositeSpec | string => {
    const layout = layouts.find((l) => l.name === layoutSel.value);
    if (!layout) return "No layout selected.";
    const top = topIn.value.trim() === "" ? null : Number(topIn.value);
    const bottom = botIn.value.trim() === "" ? null : Number(botIn.value);
    if (top !== null && Number.isNaN(top)) return "Depth top must be a number or blank.";
    if (bottom !== null && Number.isNaN(bottom)) return "Depth bottom must be a number or blank.";
    return {
      well_id: well.well_id,
      layout,
      depth_top: top,
      depth_bottom: bottom,
      scale: Number(scaleSel.value),
      page_size: pageSel.value as PageSize,
    };
  };

  const showPage = () => {
    if (!result) return;
    const page = result.pages[pageIdx];
    preview.innerHTML = page.svg;
    const svgEl = preview.querySelector("svg");
    if (svgEl) {
      // Fit the mm-sized page into the preview column; viewBox keeps it crisp.
      svgEl.removeAttribute("width");
      svgEl.setAttribute("height", "auto");
      (svgEl as SVGElement).style.width = "100%";
      (svgEl as SVGElement).style.height = "auto";
      (svgEl as SVGElement).style.border = "1px solid var(--border, #ccc)";
    }
    pageLabel.textContent = `Page ${pageIdx + 1} / ${result.pages.length}  ·  ${page.top_depth.toFixed(1)}–${page.bottom_depth.toFixed(1)} m`;
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
    status.textContent = "Rendering…";
    try {
      result = await renderComposite(spec);
      pageIdx = 0;
      nav.style.display = result.pages.length > 1 ? "flex" : "none";
      saveBtn.disabled = false;
      pdfBtn.disabled = false;
      status.textContent = `${result.well_name}: ${result.pages.length} page(s) at 1:${result.scale}.`;
      showPage();
    } catch (err) {
      status.textContent = `Render failed: ${err}`;
      result = null;
      saveBtn.disabled = true;
      pdfBtn.disabled = true;
    } finally {
      renderBtn.disabled = false;
    }
  });

  saveBtn.addEventListener("click", async () => {
    const spec = buildSpec();
    if (typeof spec === "string") {
      status.textContent = spec;
      return;
    }
    let dest: string | null;
    try {
      dest = await save({
        defaultPath: `${well.well_name}_composite.svg`,
        filters: [{ name: "SVG", extensions: ["svg"] }],
      });
    } catch (err) {
      status.textContent = `Save dialog unavailable: ${err}`;
      return;
    }
    if (!dest) return;
    saveBtn.disabled = true;
    status.textContent = "Writing SVG…";
    try {
      const paths = await exportCompositeSvg(spec, dest);
      status.textContent = `Wrote ${paths.length} file(s): ${paths.map((p) => p.split(/[\\/]/).pop()).join(", ")}`;
      setStatus(`Composite exported: ${paths.length} SVG page(s) for ${well.well_name}.`);
    } catch (err) {
      status.textContent = `Export failed: ${err}`;
    } finally {
      saveBtn.disabled = false;
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
        defaultPath: `${well.well_name}_composite.pdf`,
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
      const path = await exportCompositePdf(spec, dest);
      status.textContent = `Wrote ${path.split(/[\\/]/).pop()}`;
      setStatus(`Composite PDF exported for ${well.well_name}.`);
    } catch (err) {
      status.textContent = `PDF export failed: ${err}`;
    } finally {
      pdfBtn.disabled = false;
    }
  });

  return { el: content };
}
