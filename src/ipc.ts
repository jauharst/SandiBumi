import { invoke } from "@tauri-apps/api/core";

export interface EquationDef {
  equation_id: string;
  name: string;
  description: string | null;
  script: string;
  input_curves: string[];
  output_curve: string;
  output_units: string | null;
  /** "python" (vectorized numpy — default for new equations) or "rhai" (per-sample, legacy). */
  language: string;
}

export interface EquationRunResult {
  well_id: string;
  rows_written: number;
  error: string | null;
}

export interface CurveCatalogEntry {
  name: string;
  units: string | null;
  source: "Standard" | "Computed";
}

/** Saves (or updates, by unique name) an equation and returns its id. */
export function saveEquation(def: EquationDef): Promise<string> {
  return invoke<string>("save_equation", { def });
}

export function listEquations(): Promise<EquationDef[]> {
  return invoke<EquationDef[]>("list_equations");
}

export function runEquation(equationId: string, wellIds: string[]): Promise<EquationRunResult[]> {
  return invoke<EquationRunResult[]>("run_equation", { equationId, wellIds });
}

export function listCurveCatalog(): Promise<CurveCatalogEntry[]> {
  return invoke<CurveCatalogEntry[]>("list_curve_catalog");
}

export interface WellSummary {
  well_id: string;
  well_name: string;
  field_name: string | null;
}

export function listWells(): Promise<WellSummary[]> {
  return invoke<WellSummary[]>("list_wells");
}

export interface ImportResult {
  path: string;
  well_id: string | null;
  well_name: string | null;
  rows: number;
  error: string | null;
}

export function importLasFiles(paths: string[]): Promise<ImportResult[]> {
  return invoke<ImportResult[]>("import_las_files", { paths });
}

export interface TopEntry {
  top_name: string;
  depth: number;
  color: string | null;
}

export function listTops(wellId: string): Promise<TopEntry[]> {
  return invoke<TopEntry[]>("list_tops", { wellId });
}

/** Crossing warnings: top pairs in this well whose order contradicts most other wells. */
export function checkTopOrder(wellId: string): Promise<string[]> {
  return invoke<string[]>("check_top_order", { wellId });
}

export interface AutoCorrRequest {
  source_well_id: string;
  top_name: string;
  curve: string;
  half_window: number;
  search_range: number;
  target_well_ids: string[];
}

export interface AutoCorrProposal {
  well_id: string;
  current_depth: number | null;
  proposed_depth: number | null;
  correlation: number;
  error: string | null;
}

export interface AutoCorrResult {
  proposals: AutoCorrProposal[];
  error: string | null;
}

export function autocorrelateTop(req: AutoCorrRequest): Promise<AutoCorrResult> {
  return invoke<AutoCorrResult>("autocorrelate_top", { req });
}

export type ScaleType = "linear" | "log";

export interface CurveStyle {
  curve_name: string;
  color: string;
  min: number;
  max: number;
  /** Edge shading, or "blocks" for discrete class curves (facies) rendered as
   *  full-track-width colored intervals instead of a value line. */
  fill?: "none" | "left" | "right" | "blocks";
  fill_color?: string;
  fill_opacity?: number;
}

export interface Track {
  title: string;
  width_weight: number;
  scale_type: ScaleType;
  curves: CurveStyle[];
}

export interface Layout {
  name: string;
  tracks: Track[];
}

export function listLayouts(): Promise<Layout[]> {
  return invoke<Layout[]>("list_layouts");
}

// ---------------------------------------------------------------------------
// Composite log plots (Phase 8 deliverables): layout → vector SVG per page
// ---------------------------------------------------------------------------

export type PageSize = "a4" | "a3" | "letter";

export interface CompositeSpec {
  well_id: string;
  layout: Layout;
  depth_top?: number | null;
  depth_bottom?: number | null;
  scale: number;
  page_size: PageSize;
}

export interface CompositePage {
  svg: string;
  top_depth: number;
  bottom_depth: number;
  index: number;
}

export interface CompositeResult {
  pages: CompositePage[];
  page_width_mm: number;
  page_height_mm: number;
  scale: number;
  well_name: string;
}

/** Renders a composite log plot at a true print scale, returning one vector SVG per page. */
export function renderComposite(spec: CompositeSpec): Promise<CompositeResult> {
  return invoke<CompositeResult>("render_composite", { spec });
}

/** Renders and writes the composite to disk as SVG (one file per page when multi-page),
 *  returning the paths written. */
export function exportCompositeSvg(spec: CompositeSpec, destPath: string): Promise<string[]> {
  return invoke<string[]>("export_composite_svg", { spec, destPath });
}

/** Renders and writes the composite as a single multi-page PDF, returning the path. */
export function exportCompositePdf(spec: CompositeSpec, destPath: string): Promise<string> {
  return invoke<string>("export_composite_pdf", { spec, destPath });
}

// ---------------------------------------------------------------------------
// Report generator (Phase 8b): cover + methodology + zone params + pay summary
// + composite pages → one PDF
// ---------------------------------------------------------------------------

export interface MethodRow {
  parameter: string;
  method: string;
  remarks: string;
}

export interface ReportSpec {
  composite: CompositeSpec;
  title: string;
  author: string;
  /** Empty = the built-in default methodology template. */
  methodology: MethodRow[];
  vsh_max: number;
  phie_min: number;
  swe_max: number;
  perm_min?: number | null;
  tables_only?: boolean;
}

/** Renders the full report as per-page SVGs for preview. */
export function renderReport(spec: ReportSpec): Promise<CompositeResult> {
  return invoke<CompositeResult>("render_report", { spec });
}

/** Renders and writes the full report as one multi-page PDF, returning the path. */
export function exportReportPdf(spec: ReportSpec, destPath: string): Promise<string> {
  return invoke<string>("export_report_pdf", { spec, destPath });
}

/** Batch export: one report PDF per well into destDir; returns the written paths. */
export function exportReportBatch(spec: ReportSpec, wellIds: string[], destDir: string): Promise<string[]> {
  return invoke<string[]>("export_report_batch", { spec, wellIds, destDir });
}

/** Writes a base64-encoded PNG (rasterized in the frontend) to a user-picked path. */
export function savePng(destPath: string, dataBase64: string): Promise<string> {
  return invoke<string>("save_png", { destPath, dataBase64 });
}

export interface DocumentEntry {
  name: string;
  json: string;
}

/** Named JSON documents (saved layouts, plot property sets, ...). */
export function saveDocument(docType: string, name: string, json: string): Promise<void> {
  return invoke<void>("save_document", { docType, name, json });
}

export function listDocuments(docType: string): Promise<DocumentEntry[]> {
  return invoke<DocumentEntry[]>("list_documents", { docType });
}

export function deleteDocument(docType: string, name: string): Promise<void> {
  return invoke<void>("delete_document", { docType, name });
}

/** A user-defined named subset of wells, for filtering the workspace and scoping batch
 *  runs when the field has thousands of wells. `active` marks the one group currently
 *  filtering everything (at most one). `well_ids` is the explicit membership. */
export interface WellGroupEntry {
  group_id: string;
  name: string;
  active: boolean;
  member_count: number;
  well_ids: string[];
}

export function listWellGroups(): Promise<WellGroupEntry[]> {
  return invoke<WellGroupEntry[]>("list_well_groups");
}

export function createWellGroup(name: string, wellIds: string[]): Promise<string> {
  return invoke<string>("create_well_group", { name, wellIds });
}

export function renameWellGroup(groupId: string, name: string): Promise<void> {
  return invoke<void>("rename_well_group", { groupId, name });
}

export function deleteWellGroup(groupId: string): Promise<void> {
  return invoke<void>("delete_well_group", { groupId });
}

export function setWellGroupMembers(groupId: string, wellIds: string[]): Promise<void> {
  return invoke<void>("set_well_group_members", { groupId, wellIds });
}

/** Activates a group (null = "All wells" / clear the active group). */
export function setActiveWellGroup(groupId: string | null): Promise<void> {
  return invoke<void>("set_active_well_group", { groupId });
}

export interface TrackCurveSeries {
  curve_name: string;
  depth: Float32Array;
  value: Float32Array;
}

/** Copies raw IPC bytes (which may arrive as a plain number[] rather than a Uint8Array,
 *  depending on the IPC transport) into a fresh, alignment-safe buffer. */
function bytesToFloat32Array(bytes: Uint8Array | number[]): Float32Array {
  const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const copy = new Uint8Array(u8.byteLength);
  copy.set(u8);
  return new Float32Array(copy.buffer);
}

type RawCurveSeries = { curve_name: string; point_count: number; data: Uint8Array | number[] };

/** Unpacks the `depth[n]` + `value[n]` raw-byte convention shared by every curve-series
 *  IPC command (per this project's rule against bulk arrays as JSON). */
function unpackCurveSeries(raw: RawCurveSeries[]): TrackCurveSeries[] {
  return raw.map((r) => {
    const floats = bytesToFloat32Array(r.data);
    return {
      curve_name: r.curve_name,
      depth: floats.slice(0, r.point_count),
      value: floats.slice(r.point_count, r.point_count * 2),
    };
  });
}

/**
 * Fetches every curve referenced by a layout's tracks for one well, decimated to
 * `targetPixelHeight`. Numeric data travels as raw bytes (not JSON numbers), per this
 * project's IPC rule — unpacked here into depth/value Float32Arrays per curve.
 */
export async function getTrackData(wellId: string, curveNames: string[], targetPixelHeight: number): Promise<TrackCurveSeries[]> {
  const raw = await invoke<RawCurveSeries[]>("get_track_data", { wellId, curveNames, targetPixelHeight });
  return unpackCurveSeries(raw);
}

// ---------------------------------------------------------------------------
// Deterministic workflow: module manifests, zones, runner, pay summary
// ---------------------------------------------------------------------------

export type ArgKind = "param" | "option" | "log_in" | "log_out";

export interface ArgSpec {
  name: string;
  desc: string;
  unit: string;
  kind: ArgKind;
  default: string;
  choices: string[];
  min: number | null;
  max: number | null;
  required: boolean;
}

export interface ModuleSpec {
  name: string;
  title: string;
  category: string;
  doc: string;
  args: ArgSpec[];
}

export async function listModules(): Promise<ModuleSpec[]> {
  return invoke<ModuleSpec[]>("list_modules");
}

export interface RunModuleRequest {
  module: string;
  well_ids: string[];
  log_inputs: Record<string, string>;
  params: Record<string, number>;
  opts: Record<string, string>;
  /** Log set the outputs are versioned into (re-run = version N+1). Omitted = "INTERP". */
  output_set?: string;
  /** Log set the INPUTS are read from (latest version per well); curves that set wrote
   *  come from its archived values, others fall back. Omitted = current values. */
  input_set?: string;
}

export interface ModuleRunResult {
  well_id: string;
  rows_written: number;
  output_curves: string[];
  error: string | null;
}

export async function runWorkflowModule(req: RunModuleRequest): Promise<ModuleRunResult[]> {
  return invoke<ModuleRunResult[]>("run_workflow_module", { req });
}

// --- Workflow chains (Phase 9) --------------------------------------------

export interface ChainStep {
  module: string;
  log_inputs: Record<string, string>;
  params: Record<string, number>;
  opts: Record<string, string>;
}

/** Tagged union mirroring the Rust `ChainStatus` (serde tag = "state"). */
export type ChainStatus =
  | { state: "queued" }
  | { state: "running"; step: number; total_steps: number; module: string; wells_done: number; wells_total: number }
  | { state: "completed"; steps_run: number; curves_written: number; wells: number; errors: string[] }
  | { state: "cancelled"; at_step: number }
  | { state: "failed"; error: string };

/** Runs a chain; resolves when finished. Poll {@link getChainStatus} with the same jobId
 * for live progress while this promise is pending. `outputSet` names the log set the
 * whole chain run is versioned into (one version per run; default "INTERP"); `inputSet`
 * makes every step read its inputs from that named set (latest version per well). */
export async function runWorkflowChain(
  jobId: string,
  steps: ChainStep[],
  wellIds: string[],
  outputSet?: string,
  inputSet?: string,
): Promise<void> {
  return invoke<void>("run_workflow_chain", { jobId, steps, wellIds, outputSet: outputSet ?? null, inputSet: inputSet ?? null });
}

// --- P1-c log-set versioning (never overwrite) ------------------------------

/** One run event into a named log set (the version history of computed outputs). */
export interface LogSetEntry {
  set_id: string;
  set_name: string;
  version: number;
  module: string;
  params_json: string | null;
  inputs_json: string | null;
  created_at: string;
  curve_names: string[];
  /** True while any current curve value still comes from this version. */
  is_current: boolean;
}

/** A well's version history, newest first per set. */
export function listLogSets(wellId: string): Promise<LogSetEntry[]> {
  return invoke<LogSetEntry[]>("list_log_sets", { wellId });
}

/** Copies an archived version back into the current store; returns restored row count. */
export function restoreLogSet(setId: string): Promise<number> {
  return invoke<number>("restore_log_set", { setId });
}

/** Deletes one version's history (current values are kept). */
export function deleteLogSet(setId: string): Promise<void> {
  return invoke<void>("delete_log_set", { setId });
}

/** Current computed curves of a well with provenance + basic statistics. */
export interface ComputedCatalogEntry {
  curve_name: string;
  set_name: string | null;
  version: number | null;
  module: string | null;
  created_at: string | null;
  n_samples: number;
  min: number | null;
  max: number | null;
  mean: number | null;
}

export function listComputedCatalog(wellId: string): Promise<ComputedCatalogEntry[]> {
  return invoke<ComputedCatalogEntry[]>("list_computed_catalog", { wellId });
}

export async function getChainStatus(jobId: string): Promise<ChainStatus | null> {
  return invoke<ChainStatus | null>("get_chain_status", { jobId });
}

export async function cancelWorkflowChain(jobId: string): Promise<void> {
  return invoke<void>("cancel_workflow_chain", { jobId });
}

// --- Monte Carlo uncertainty (Phase 9) ------------------------------------

/** Distribution on one uncertain parameter (mirrors the Rust `Distribution`, serde tag "kind"). */
export type McDistribution =
  | { kind: "normal"; mean: number; sd: number }
  | { kind: "uniform"; lo: number; hi: number }
  | { kind: "triangular"; lo: number; mode: number; hi: number };

export interface McParam {
  param: string;
  dist: McDistribution;
}

export interface McRequest {
  well_ids: string[];
  steps: ChainStep[];
  mc_params: McParam[];
  iterations: number;
  seed: number;
  vsh_max: number;
  phie_min: number;
  swe_max: number;
  perm_min: number | null;
  bins: number;
}

/** P10/P50/P90 + mean/sd of one metric across realizations. */
export interface Pctl {
  p10: number;
  p50: number;
  p90: number;
  mean: number;
  sd: number;
}

export interface McZoneResult {
  well_id: string;
  well_name: string;
  zone: string;
  top: number;
  bottom: number;
  gross: number;
  iterations: number;
  net: Pctl;
  ntg: Pctl;
  avg_phie: Pctl;
  avg_swe: Pctl;
  hpv: Pctl;
  hpv_hist: number[];
  hist_lo: number;
  hist_w: number;
}

export interface McResult {
  zones: McZoneResult[];
  errors: string[];
}

/** Runs a Monte Carlo study; resolves with the per-zone P10/P50/P90 + HPV histograms. Runs
 *  in memory on the backend (no computed_curves writes), so 1000+ realizations are fast. */
export async function runMonteCarlo(req: McRequest): Promise<McResult> {
  return invoke<McResult>("run_monte_carlo", { req });
}

// --- Machine learning (Phase 10-4) ---

export interface MlRequest {
  task: "regression" | "classification" | "clustering" | "reduction";
  algorithm: string;
  params: Record<string, number | string | boolean>;
  feature_curves: string[];
  target_curve: string | null;
  train_well_ids: string[];
  apply_well_ids: string[];
  output_curve: string;
}

export interface MlWellResult {
  well_id: string;
  rows_predicted: number;
  error: string | null;
}

export interface MlResult {
  /** Curve names written (base + suffix, e.g. FACIES_ML, FACIES_ML_PROB, PC1…). */
  outputs: string[];
  metrics: Record<string, unknown> | null;
  wells: MlWellResult[];
  error: string | null;
}

/** Runs a scikit-learn model (subprocess): supervised tasks fit on the train wells'
 *  labelled samples and predict on the apply wells; unsupervised tasks fit on the pooled
 *  apply samples (field-wide, globally consistent cluster ids). */
export function runMl(req: MlRequest): Promise<MlResult> {
  return invoke<MlResult>("run_ml", { req });
}

// --- Generalized Multimin (multi-mineral inversion) -----------------------

/** A mineral, clay, or fluid component (Geolog-style: fluids are zone-typed X/U). */
export interface MmComponent {
  name: string;
  /** "mineral" | "clay" | "fluid" */
  kind: string;
  /** Fluids: "X" (flushed / Sxo), "U" (unflushed / Sw), or "" (both zones). */
  zone: string;
  /** Fluids: "water" | "bound_water" | "oil" | "gas". */
  fluid_type: string;
  endpoints: Record<string, number>;
  /** Cation exchange capacity, meq/g (clays → bound-water constraint). */
  cec: number;
  /** Upper volume bound (1.0 minerals, 0.5 fluids by default). */
  max_vol: number;
}

/** A tool in the inversion. Keys "CT"/"CXO" take a RESISTIVITY curve (the backend converts
 *  to conductivity, mho/m); sigma <= 0 on CT/CXO means auto (0.03·C^(1/w)). */
export interface MmTool {
  key: string;
  curve: string;
  sigma: number;
}

/** Fluid/saturation parameters — required when CT or CXO participates. */
export interface MmFluidProps {
  rw: number;
  rw_temp_f: number;
  rmf: number;
  rmf_temp_f: number;
  ftemp_f: number;
  m: number;
  n: number;
  mud_type: string;
}

/** Derived fluid quantities (w, conductivities, α, auto CT/CXO uncertainties). */
export interface MmFluidCalc {
  w: number;
  cw: number;
  cmf: number;
  cbw: number;
  cbw_x: number;
  cbw_u: number;
  alpha_x: number;
  alpha_u: number;
  salinity_w_ppm: number;
  salinity_mf_ppm: number;
  u_ct: number;
  u_cxo: number;
}

export interface MultiminRequest {
  components: MmComponent[];
  tools: MmTool[];
  apply_well_ids: string[];
  output_prefix: string;
  unity: boolean;
  fluid: MmFluidProps | null;
}

export interface MultiminWellResult {
  well_id: string;
  rows_solved: number;
  mean_recon: number;
  error: string | null;
}

export interface MultiminResult {
  outputs: string[];
  wells: MultiminWellResult[];
  error: string | null;
}

/** The built-in mineral/fluid endpoint library (editable defaults for the dialog). */
export function multiminLibrary(): Promise<MmComponent[]> {
  return invoke<MmComponent[]>("multimin_library");
}

/** Runs the generalized multi-mineral inversion; writes VOL_<component> + derived curves. */
export function runMultimin(req: MultiminRequest): Promise<MultiminResult> {
  return invoke<MultiminResult>("run_multimin", { req });
}

/** Derived fluid quantities (Cw, Cmf, Cbw, α, w, auto CT/CXO σ) for the dialog preview. */
export function multiminFluidCalc(props: MmFluidProps): Promise<MmFluidCalc> {
  return invoke<MmFluidCalc>("multimin_fluid_calc", { props });
}

/** Wet-clay picks + assumed dry-clay density for the wet→dry endpoint conversion. */
export interface MmWetClayInput {
  rhob_wet: number;
  nphi_wet: number;
  gr_wet: number;
  dt_wet: number | null;
  rho_dry: number;
  fluid: MmFluidProps | null;
}

export interface MmDryClayCalc {
  phi_clay: number;
  rhob_dry: number;
  nphi_dry: number;
  gr_dry: number;
  dt_dry: number | null;
  cbw_ratio: number;
  cec_equiv: number;
}

export function multiminDryClay(input: MmWetClayInput): Promise<MmDryClayCalc> {
  return invoke<MmDryClayCalc>("multimin_dry_clay", { input });
}

/** Zone-averaged FTEMP_F / RMF from the precalc module's output curves. */
export interface MmPrecalcFluid {
  ftemp_f: number | null;
  rmf: number | null;
  n_ftemp: number;
  n_rmf: number;
}

export function multiminFluidFromPrecalc(
  wellId: string,
  top: number | null,
  bottom: number | null,
): Promise<MmPrecalcFluid> {
  return invoke<MmPrecalcFluid>("multimin_fluid_from_precalc", { wellId, top, bottom });
}

export interface ZoneEntry {
  zone_name: string;
  top_depth: number;
  bottom_depth: number;
}

export async function listZones(wellId: string): Promise<ZoneEntry[]> {
  return invoke<ZoneEntry[]>("list_zones", { wellId });
}

export async function upsertZone(wellId: string, zoneName: string, topDepth: number, bottomDepth: number): Promise<void> {
  return invoke("upsert_zone", { wellId, zoneName, topDepth, bottomDepth });
}

export async function deleteZone(wellId: string, zoneName: string): Promise<void> {
  return invoke("delete_zone", { wellId, zoneName });
}

export async function zonesFromTops(wellId: string): Promise<ZoneEntry[]> {
  return invoke<ZoneEntry[]>("zones_from_tops", { wellId });
}

export interface ZoneParamEntry {
  zone_name: string;
  param_name: string;
  value_num: number | null;
  value_text: string | null;
}

export async function listZoneParams(wellId: string): Promise<ZoneParamEntry[]> {
  return invoke<ZoneParamEntry[]>("list_zone_params", { wellId });
}

export async function setZoneParam(
  wellId: string,
  zoneName: string,
  paramName: string,
  valueNum: number | null,
  valueText: string | null,
): Promise<void> {
  return invoke("set_zone_param", { wellId, zoneName, paramName, valueNum, valueText });
}

export interface PaySummaryRequest {
  well_ids: string[];
  vsh_max: number;
  phie_min: number;
  swe_max: number;
  perm_min: number | null;
}

export interface PaySummaryRow {
  well_id: string;
  well_name: string;
  zone: string;
  flag: string;
  top: number;
  bottom: number;
  gross: number;
  net: number;
  ntg: number;
  avg_vsh: number;
  avg_phie: number;
  avg_swe: number;
  hpv: number;
}

export async function runPaySummary(req: PaySummaryRequest): Promise<PaySummaryRow[]> {
  return invoke<PaySummaryRow[]>("run_pay_summary", { req });
}

/** Cutoff-sensitivity sweep (Method 1 of the cutoff study): sweep one cutoff over a range,
 *  holding the other two fixed, and report the pay metric per well at each step. */
export interface CutoffSweepRequest {
  well_ids: string[];
  property: "VSH" | "PHIE" | "SWE";
  vsh_max: number;
  phie_min: number;
  swe_max: number;
  perm_min: number | null;
  sweep_min: number;
  sweep_max: number;
  steps: number;
  metric: "NET" | "HPV" | "NTG";
  zone: string | null;
  dst_dataset: string | null;
}

export interface CutoffSweepSeries {
  well_id: string;
  well_name: string;
  cutoffs: number[];
  values: number[];
  peak: number;
  gross: number;
  n_samples: number;
}

export interface CutoffSweepResult {
  series: CutoffSweepSeries[];
  property: string;
  metric: string;
}

export async function runCutoffSweep(req: CutoffSweepRequest): Promise<CutoffSweepResult> {
  return invoke<CutoffSweepResult>("run_cutoff_sweep", { req });
}

/** Full-resolution curve data for parameter-selection plots, optionally windowed to a
 *  depth interval. Binary transport, unpacked to Float32Arrays like getTrackData. */
export async function getCurveData(
  wellId: string,
  curveNames: string[],
  depthMin: number | null,
  depthMax: number | null,
): Promise<TrackCurveSeries[]> {
  const raw = await invoke<RawCurveSeries[]>("get_curve_data", { wellId, curveNames, depthMin, depthMax });
  return unpackCurveSeries(raw);
}

// ---------------------------------------------------------------------------
// Core plug data (routine core analysis): sparse/irregular depths, imported per
// well by CSV and overlaid onto crossplots/log tracks — not aligned onto the
// standard depth grid like log curves.
// ---------------------------------------------------------------------------

export interface CoreImportResult {
  path: string;
  rows: number;
  error: string | null;
}

/** Parses a core CSV (alias-resolved headers: DEPTH, CPOR/POR, CPERM/PERM, CGD, CSW)
 *  and replaces the given well's core plug data. */
export function importCoreCsv(wellId: string, path: string): Promise<CoreImportResult> {
  return invoke<CoreImportResult>("import_core_csv", { wellId, path });
}

// --- P2 tops-style imports: tops CSV/TXT + petrography/XRD/perforation ------

export interface TopsImportResult {
  path: string;
  tops_written: number;
  wells_matched: number;
  unmatched_wells: string[];
  error: string | null;
}

/** Imports formation tops from CSV/TXT. Multi-well files match wells by name;
 *  files without a WELL column land in `defaultWellId` (the selected well). */
export function importTopsCsv(defaultWellId: string | null, path: string): Promise<TopsImportResult> {
  return invoke<TopsImportResult>("import_tops_csv", { defaultWellId, path });
}

export interface AuxImportResult {
  path: string;
  dataset: string;
  rows: number;
  items: string[];
  error: string | null;
}

export interface AuxRow {
  dataset: string;
  depth_top: number;
  depth_base: number | null;
  item: string;
  value_num: number | null;
  value_text: string | null;
}

/** Imports a tops-style dataset (PETROGRAPHY / XRD / PERFORATION / custom) for one
 *  well, replacing that well's previous rows of the same dataset. */
export function importAuxData(wellId: string, dataset: string, path: string): Promise<AuxImportResult> {
  return invoke<AuxImportResult>("import_aux_data", { wellId, dataset, path });
}

export function listAuxData(wellId: string, dataset: string | null): Promise<AuxRow[]> {
  return invoke<AuxRow[]>("list_aux_data", { wellId, dataset });
}

export function listAuxDatasets(wellId: string): Promise<[string, number][]> {
  return invoke<[string, number][]>("list_aux_datasets", { wellId });
}

export interface LeverettFit {
  a: number;
  b: number;
  r2: number;
  n_points: number;
}

export interface ScalImportResult {
  path: string;
  rows: number;
  fit: LeverettFit | null;
  error: string | null;
}

export interface ScalPcRow {
  sample_no: number | null;
  depth: number | null;
  perm: number;
  poro: number;
  pc: number;
  sw: number;
}

/** Imports a SCAL Pc/Sw CSV for the well and returns the Leverett-J fit (Sw = A·J^B)
 *  at the given lab sigma·cosθ, for carrying into the sw_height module. */
export function importScalCsv(wellId: string, path: string, iftLab: number): Promise<ScalImportResult> {
  return invoke<ScalImportResult>("import_scal_csv", { wellId, path, iftLab });
}

export function getScalPc(wellId: string): Promise<ScalPcRow[]> {
  return invoke<ScalPcRow[]>("get_scal_pc", { wellId });
}

/** Fetches a well's core plug data as CPOR/CPERM/CGD/CSW series (each only its own
 *  non-NaN samples, at their own depths — not resampled onto the log depth grid). */
export async function getCoreData(wellId: string): Promise<TrackCurveSeries[]> {
  const raw = await invoke<RawCurveSeries[]>("get_core_data", { wellId });
  return unpackCurveSeries(raw);
}

export function updateCoreSample(wellId: string, depth: number, column: string, value: number): Promise<void> {
  return invoke("update_core_sample", { wellId, depth, column, value });
}

/** Shifts every core plug of a well by `delta` metres (core-to-log alignment);
 *  returns the number of plugs moved. Exactly reversible with -delta. */
export function shiftCoreData(wellId: string, delta: number): Promise<number> {
  return invoke<number>("shift_core_data", { wellId, delta });
}

// --- Interactive curve editing (P2-d: log-view right-click menu) ---

export interface CurveEditRequest {
  well_id: string;
  curve: string;
  /** "shift" (wireline depth shift) | "set" | "blank" | "interpolate" | "scale". */
  op: "shift" | "set" | "blank" | "interpolate" | "scale";
  delta?: number;
  top?: number;
  bottom?: number;
  value?: number;
  mul?: number;
  add?: number;
}

/** `data` holds the CHANGED rows' previous samples as packed `depth[n] + value[n]`
 *  f32-LE bytes — pass it back to {@link restoreCurveValues} verbatim to undo. */
export interface CurveEditResult {
  affected: number;
  store: string;
  point_count: number;
  data: Uint8Array | number[];
}

export function editCurve(req: CurveEditRequest): Promise<CurveEditResult> {
  return invoke<CurveEditResult>("edit_curve", { req });
}

export function restoreCurveValues(wellId: string, curve: string, pointCount: number, data: number[]): Promise<number> {
  return invoke<number>("restore_curve_values", { wellId, curve, pointCount, data });
}

/** Checkpoints the DuckDB project database and copies it to `destPath`. */
export async function saveProjectAs(destPath: string): Promise<void> {
  return invoke("save_project_as", { destPath });
}

// ---------------------------------------------------------------------------
// Projects (open / new / recent list — "IP style" project switching)
// ---------------------------------------------------------------------------

export interface RecentProject {
  path: string;
  name: string;
  /** Unix seconds of the last successful open (0 = unknown). */
  last_opened: number;
  exists: boolean;
}

export async function listRecentProjects(): Promise<RecentProject[]> {
  return invoke("list_recent_projects");
}

export async function currentProject(): Promise<RecentProject> {
  return invoke("current_project");
}

/** Switches the live connection to an existing project file. */
export async function openProject(path: string): Promise<RecentProject> {
  return invoke("open_project", { path });
}

/** Creates a fresh, empty project file and switches to it. */
export async function newProject(path: string): Promise<RecentProject> {
  return invoke("new_project", { path });
}

// ---------------------------------------------------------------------------
// Database inspector (whitelisted tables; cells travel as strings)
// ---------------------------------------------------------------------------

export interface TablePage {
  columns: string[];
  rows: (string | null)[][];
  total_rows: number;
}

export function getTablePage(table: string, wellId: string | null, offset: number, limit: number): Promise<TablePage> {
  return invoke<TablePage>("get_table_page", { table, wellId, offset, limit });
}

export function updateWellField(wellId: string, field: string, value: string | null): Promise<void> {
  return invoke("update_well_field", { wellId, field, value });
}

/** NaN = missing. */
export function updateStandardSample(wellId: string, depth: number, column: string, value: number): Promise<void> {
  return invoke("update_standard_sample", { wellId, depth, column, value });
}

export function updateComputedSample(wellId: string, depth: number, curveName: string, value: number): Promise<void> {
  return invoke("update_computed_sample", { wellId, depth, curveName, value });
}

export function upsertTop(wellId: string, topName: string, depth: number, color: string | null): Promise<void> {
  return invoke("upsert_top", { wellId, topName, depth, color });
}

export function deleteTop(wellId: string, topName: string): Promise<void> {
  return invoke("delete_top", { wellId, topName });
}

/** Read-only SQL over the project database (full DuckDB SQL, SELECT-only). */
export function runQuery(sql: string, limit = 1000): Promise<TablePage> {
  return invoke<TablePage>("run_query", { sql, limit });
}

/** Exports one well's standard + computed curves as LAS 2.0; returns row count. */
export function exportLas(wellId: string, destPath: string): Promise<number> {
  return invoke<number>("export_las", { wellId, destPath });
}

/** Path of the Python interpreter the equation engine will use (null = none found). */
export function pythonStatus(): Promise<string | null> {
  return invoke<string | null>("python_status");
}

// ---------------------------------------------------------------------------
// Phase 6: generic curve store (curve_meta/curve_samples), deviation surveys,
// DLIS import. The generic store holds ANY curve (PEF, CALI, multiple runs, ...)
// per well across RAW/EDIT/FINAL sets, unlike the fixed-6 standard_curves.
// ---------------------------------------------------------------------------

export interface GenericCurveCatalogEntry {
  curve_id: string;
  mnemonic: string;
  unit: string | null;
  family: string | null;
  set_name: string;
  source: string | null;
  run_no: number | null;
  n_samples: number;
}

/** Every curve in the generic store for one well, across RAW/EDIT/FINAL sets. */
export function listGenericCurveCatalog(wellId: string): Promise<GenericCurveCatalogEntry[]> {
  return invoke<GenericCurveCatalogEntry[]>("list_generic_curve_catalog", { wellId });
}

export interface CurveSamplePoint {
  depth: number;
  value: number;
}

export function getGenericCurveSamples(curveId: string): Promise<CurveSamplePoint[]> {
  return invoke<CurveSamplePoint[]>("get_generic_curve_samples", { curveId });
}

export interface WellPathStation {
  md: number;
  inc: number;
  azi: number;
  tvd: number;
  tvdss: number;
}

/** Imports a deviation survey CSV (MD/INC/AZI) and computes minimum-curvature TVD/TVDSS.
 *  `datumElevation` (KB above MSL) is used for TVDSS; null falls back to the well's KB. */
export function importDeviationCsv(wellId: string, path: string, datumElevation: number | null): Promise<CoreImportResult> {
  return invoke<CoreImportResult>("import_deviation_csv", { wellId, path, datumElevation });
}

export function getWellPath(wellId: string): Promise<WellPathStation[]> {
  return invoke<WellPathStation[]>("get_well_path", { wellId });
}

export interface DlisImportResult {
  path: string;
  curves_imported: number;
  rows: number;
  error: string | null;
}

/** Imports every scalar channel of a DLIS file into the selected well's generic store
 *  (via dlisio through the Python subprocess). */
export function importDlisFile(wellId: string, path: string): Promise<DlisImportResult> {
  return invoke<DlisImportResult>("import_dlis_file", { wellId, path });
}
