import type { PaperExportBounds, PaperExportRecord, PlotAncestryScope } from "../ipc";

// SB-PLT-026 cites the shipped print path, whose paper rule was already 12 mm. Reuse that
// source-owned value for vector and raster custody; do not choose an A4/Letter page default.
export const PAPER_MARGIN_PT = (12 * 72) / 25.4;
// The shipped provenance block was already 8 pt monospace in plotExport.ts.
export const PAPER_PROVENANCE_FONT_PT = 8;

export interface PaperTextMetrics {
  width: number;
  left: number;
  right: number;
  ascent: number;
  descent: number;
}

let textMeasureContext: CanvasRenderingContext2D | null | undefined;

function fontSizePx(font: string): number {
  const match = /(\d+(?:\.\d+)?)px/.exec(font);
  return match ? Number.parseFloat(match[1]) : 10;
}

function fallbackTextMetrics(
  font: string,
  text: string,
  align: CanvasTextAlign,
): PaperTextMetrics {
  const size = fontSizePx(font);
  // A missing Canvas TextMetrics implementation must enlarge the page, never
  // recreate the old under-measurement. Two em per Unicode scalar is deliberately
  // conservative for the test/runtime fallback; browsers use their real glyph bounds.
  const width = Array.from(text).length * size * 2;
  const left = align === "center" ? width / 2 : align === "right" || align === "end" ? width : 0;
  return { width, left, right: width - left, ascent: size, descent: size / 2 };
}

/** Measures the glyph box used to certify a paper page. Production browsers supply
 * actual TextMetrics bounds; a runtime without them gets a deliberately oversized box
 * so it can waste whitespace but can never certify a character-count underestimate. */
export function measurePaperText(
  font: string,
  text: string,
  align: CanvasTextAlign,
  baseline: CanvasTextBaseline,
): PaperTextMetrics {
  if (textMeasureContext === undefined) {
    const canvas = document.createElement("canvas");
    textMeasureContext = canvas.getContext?.("2d") ?? null;
  }
  const fallback = fallbackTextMetrics(font, text, align);
  const context = textMeasureContext;
  if (!context) return fallback;
  context.font = font;
  context.textAlign = align;
  context.textBaseline = baseline;
  const measured = context.measureText(text);
  const values = {
    width: measured.width,
    left: measured.actualBoundingBoxLeft,
    right: measured.actualBoundingBoxRight,
    ascent: measured.actualBoundingBoxAscent,
    descent: measured.actualBoundingBoxDescent,
  };
  return Object.values(values).every(Number.isFinite) ? values : fallback;
}

export function paperProvenanceFooter(scope: PlotAncestryScope): string {
  let excluded = 0;
  let hidden = 0;
  for (const record of scope.statisticsRecords ?? []) {
    const counts = record.exclusions;
    excluded += counts.non_finite + counts.log_domain + counts.validity + counts.selection + counts.unpaired_or_unclassified;
    hidden += counts.display_hidden;
  }
  return (
    `SandiBumi provenance: wells=${scope.wellIds.length}; bindings=${scope.plotBindings?.length ?? 0}; ` +
    `axes=${scope.axisRanges?.length ?? 0}; statistics=${scope.statisticsRecords?.length ?? 0}; ` +
    `excluded=${excluded}; display-hidden=${hidden}; full records embedded`
  );
}

function finiteBounds(bounds: PaperExportBounds): boolean {
  return (
    Number.isFinite(bounds.min_x) &&
    Number.isFinite(bounds.min_y) &&
    Number.isFinite(bounds.max_x) &&
    Number.isFinite(bounds.max_y) &&
    bounds.max_x >= bounds.min_x &&
    bounds.max_y >= bounds.min_y
  );
}

export function buildPaperExportRecord(
  medium: PaperExportRecord["medium"],
  sourceWidth: number,
  sourceHeight: number,
  contentBounds: PaperExportBounds,
  provenanceFooter: string,
): PaperExportRecord {
  if (!(sourceWidth > 0) || !(sourceHeight > 0) || !finiteBounds(contentBounds)) {
    throw new Error("paper export requires positive source geometry and finite ordered content bounds");
  }
  if (!provenanceFooter.trim()) throw new Error("paper export requires a provenance footer");
  const raster = medium === "print-raster";
  const pageBounds: PaperExportBounds = raster
    ? { ...contentBounds }
    : {
        min_x: contentBounds.min_x - PAPER_MARGIN_PT,
        min_y: contentBounds.min_y - PAPER_MARGIN_PT,
        max_x: contentBounds.max_x + PAPER_MARGIN_PT,
        max_y: contentBounds.max_y + PAPER_MARGIN_PT,
      };
  const record: PaperExportRecord = {
    schema_version: 1,
    medium,
    unit: raster ? "px" : "pt",
    source_width: sourceWidth,
    source_height: sourceHeight,
    margin_pt: PAPER_MARGIN_PT,
    content_bounds: { ...contentBounds },
    page_bounds: pageBounds,
    provenance_footer: provenanceFooter,
    crop_proof: raster
      ? "raster_pixels_preserved_before_browser_print_layout"
      : "all_recorded_bounds_inside_page",
  };
  validatePaperExportRecord(record);
  return record;
}

export function paperPageWidth(record: PaperExportRecord): number {
  return record.page_bounds.max_x - record.page_bounds.min_x;
}

export function paperPageHeight(record: PaperExportRecord): number {
  return record.page_bounds.max_y - record.page_bounds.min_y;
}

export function validatePaperExportRecord(record: PaperExportRecord): void {
  const raster = record.medium === "print-raster";
  if (record.schema_version !== 1 || record.unit !== (raster ? "px" : "pt")) {
    throw new Error("unsupported paper export schema or medium-specific unit");
  }
  if (!(record.margin_pt > 0) || !finiteBounds(record.content_bounds) || !finiteBounds(record.page_bounds)) {
    throw new Error("paper export has invalid bounds or margin");
  }
  if (
    record.content_bounds.min_x > 0 ||
    record.content_bounds.min_y > 0 ||
    record.content_bounds.max_x < record.source_width ||
    record.content_bounds.max_y < record.source_height
  ) {
    throw new Error("paper export source canvas is cropped by its declared content bounds");
  }
  if (
    record.page_bounds.min_x > record.content_bounds.min_x ||
    record.page_bounds.min_y > record.content_bounds.min_y ||
    record.page_bounds.max_x < record.content_bounds.max_x ||
    record.page_bounds.max_y < record.content_bounds.max_y
  ) {
    throw new Error("paper export content is cropped by its declared page");
  }
  const expectedProof = raster
    ? "raster_pixels_preserved_before_browser_print_layout"
    : "all_recorded_bounds_inside_page";
  if (!record.provenance_footer.trim() || record.crop_proof !== expectedProof) {
    throw new Error("paper export lacks its provenance footer or crop proof");
  }
}
