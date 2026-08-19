export type ChartRenderSurface = "screen" | "save" | "template" | "svg" | "pdf";

/** Portable identity and custody for one chart payload. The record contains metadata only;
 * numeric chart arrays continue to use their existing typed paths. */
export interface ChartRenderRecord {
  chart_id: string;
  title: string;
  chart_type: string;
  x_quantity: string;
  x_unit: string;
  y_quantity: string;
  y_unit: string;
  citation: string;
  publisher: string;
  revision_date: string;
  digitizer: string | null;
  approved_derivation_path: string;
  payload_checksum: string;
  transform_applied: string;
}

const REQUIRED_TEXT_FIELDS: ReadonlyArray<readonly [keyof ChartRenderRecord, string]> = [
  ["chart_id", "chart id"],
  ["title", "chart title"],
  ["chart_type", "chart type"],
  ["x_quantity", "X quantity"],
  ["x_unit", "X unit"],
  ["y_quantity", "Y quantity"],
  ["y_unit", "Y unit"],
  ["citation", "citation"],
  ["publisher", "publisher"],
  ["revision_date", "source revision/date"],
  ["approved_derivation_path", "approved derivation path"],
  ["transform_applied", "transform applied"],
];

/** Validates the record at the user-visible surface that is about to consume it. A persisted
 * snapshot is evidence to carry, never authority for another chart identity. */
export function chartRecordForSurface(
  selectedChartId: string,
  record: ChartRenderRecord | null | undefined,
  surface: ChartRenderSurface,
): ChartRenderRecord {
  const blocked = `${surface} chart rendering is blocked`;
  if (!record) throw new Error(`${blocked}: chart provenance is absent`);
  if (!selectedChartId.trim() || record.chart_id !== selectedChartId) {
    throw new Error(`${blocked}: chart identity does not match the selected payload`);
  }
  for (const [field, label] of REQUIRED_TEXT_FIELDS) {
    const value = record[field];
    if (typeof value !== "string" || !value.trim()) throw new Error(`${blocked}: missing ${label}`);
  }
  if (!/^[0-9a-f]{64}$/iu.test(record.payload_checksum)) {
    throw new Error(`${blocked}: payload checksum is absent or invalid`);
  }
  if (!["licensed_source", "independently_digitized_public_primary_source"].includes(record.approved_derivation_path)) {
    throw new Error(`${blocked}: derivation path is not approved`);
  }
  if (record.approved_derivation_path === "independently_digitized_public_primary_source"
    && !record.digitizer?.trim()) {
    throw new Error(`${blocked}: digitizer is absent`);
  }
  return { ...record };
}
