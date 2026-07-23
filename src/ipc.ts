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
  /** Total depth and Kelly-bushing elevation (metres); null until entered in the well header.
   *  Carried so the header dialog can prefill them instead of opening blank (KB drives TVDSS). */
  td?: number | null;
  kb?: number | null;
  /** Surface easting/northing (UTM metres) and zone label for the Field Map; null until
   *  a location is imported or entered in the well header. */
  surface_x?: number | null;
  surface_y?: number | null;
  utm_zone?: string | null;
}

export function listWells(): Promise<WellSummary[]> {
  return invoke<WellSummary[]>("list_wells");
}

export interface LocationsImportResult {
  path: string;
  wells_located: number;
  unmatched_wells: string[];
  error: string | null;
}

/** Imports well surface locations from a CSV/TXT. Multi-well files (a WELL column) match
 *  by name; single-well files use `defaultWellId`. `defaultZone` fills the UTM zone for
 *  rows without a ZONE column. */
export function importWellLocations(
  defaultWellId: string | null,
  defaultZone: string | null,
  path: string,
): Promise<LocationsImportResult> {
  return invoke<LocationsImportResult>("import_well_locations", { defaultWellId, defaultZone, path });
}

/** Wells whose surface location falls inside `polygon` (ordered [x, y] UTM-metre ring) —
 *  the authoritative hit test behind assigning a map polygon to a well group. */
export function wellsInPolygon(polygon: [number, number][]): Promise<WellSummary[]> {
  return invoke<WellSummary[]>("wells_in_polygon", { polygon });
}

export interface ImportResult {
  path: string;
  well_id: string | null;
  well_name: string | null;
  rows: number;
  /** Non-fatal note for a successful import (e.g. rows dropped for a bad/duplicate depth). */
  warning: string | null;
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
  /** "shift" (rigid best-lag, default) or "warp" (elastic depth-warp refinement). */
  method?: "shift" | "warp";
  /** Warp only: max local stretch/compression factor (≥1). Default 1.5. */
  max_stretch?: number;
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

export interface MultiAutoCorrRequest {
  source_well_id: string;
  /** Markers to propagate together. Empty ⇒ all tops picked in the source well. */
  top_names: string[];
  curve: string;
  search_range: number;
  /** Warp elasticity (≥1). Default 1.5. */
  max_stretch?: number;
  /** "shift" or "warp" (default). Either way markers stay in order. */
  method?: "shift" | "warp";
  target_well_ids: string[];
}

export interface MultiMarkerProposal {
  top_name: string;
  current_depth: number | null;
  proposed_depth: number | null;
  /** Local shape correlation around this marker after warping (per-interval confidence). */
  correlation: number;
}

export interface MultiWellProposal {
  well_id: string;
  markers: MultiMarkerProposal[];
  error: string | null;
}

export interface MultiAutoCorrResult {
  proposals: MultiWellProposal[];
  error: string | null;
}

/** Propagate several markers together with one consistent (monotone) depth warp. */
export function autocorrelateMulti(req: MultiAutoCorrRequest): Promise<MultiAutoCorrResult> {
  return invoke<MultiAutoCorrResult>("autocorrelate_multi", { req });
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
  /** "curves" (normal log track) or "well_diagram" (casing/shoe/perforations). Optional for
   *  backward compat with saved layouts; absent = "curves". */
  kind?: "curves" | "well_diagram";
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

/** Assembles a single-chart PDF from a frontend-built content stream (points, bottom-left origin;
 *  see pdfExport.ts) and writes it to `destPath`. Returns the written path. */
export function savePlotPdf(destPath: string, content: string, widthPt: number, heightPt: number): Promise<string> {
  return invoke<string>("save_plot_pdf", { destPath, content, widthPt, heightPt });
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

// --- Pinned wells: a persisted favourites subset, reused as a run-scope shortcut ---------

/** The pinned well ids (persisted, independent of groups). */
export function listPinnedWells(): Promise<string[]> {
  return invoke<string[]>("list_pinned_wells");
}

/** Pins or unpins a single well. */
export function setWellPin(wellId: string, pinned: boolean): Promise<void> {
  return invoke<void>("set_well_pin", { wellId, pinned });
}

/** Replaces the whole pinned set ("pin selection" / "clear pins"). */
export function setPinnedWells(wellIds: string[]): Promise<void> {
  return invoke<void>("set_pinned_wells", { wellIds });
}

export interface TrackCurveSeries {
  curve_name: string;
  depth: Float32Array;
  value: Float32Array;
}

/** Decodes the length-prefixed multi-curve binary buffer produced by Rust's
 *  `pack_curve_series` (returned as a raw `tauri::ipc::Response` → `ArrayBuffer`, so the
 *  f32 bytes never travel as a JSON number array). Layout, all little-endian:
 *    [u32 curve_count]
 *    repeat: [u32 name_len][name utf8][u32 point_count][f32 depth×pc][f32 value×pc]
 *  Each data block is `slice()`d into a fresh buffer because the preceding name bytes leave
 *  the read offset at an arbitrary (non-4-aligned) position — a Float32Array view needs a
 *  4-aligned offset, and slice() copies into a 0-aligned buffer. */
function decodeCurveBuffer(buf: ArrayBuffer): TrackCurveSeries[] {
  const view = new DataView(buf);
  const dec = new TextDecoder();
  let off = 0;
  const count = view.getUint32(off, true);
  off += 4;
  const out: TrackCurveSeries[] = [];
  for (let i = 0; i < count; i++) {
    const nameLen = view.getUint32(off, true);
    off += 4;
    const curve_name = dec.decode(new Uint8Array(buf, off, nameLen));
    off += nameLen;
    const pointCount = view.getUint32(off, true);
    off += 4;
    const byteLen = pointCount * 2 * 4;
    const floats = new Float32Array(buf.slice(off, off + byteLen));
    off += byteLen;
    out.push({ curve_name, depth: floats.slice(0, pointCount), value: floats.slice(pointCount, pointCount * 2) });
  }
  return out;
}

/**
 * Fetches every curve referenced by a layout's tracks for one well, decimated to
 * `targetPixelHeight`. Numeric data travels as raw bytes (not JSON numbers), per this
 * project's IPC rule — unpacked here into depth/value Float32Arrays per curve.
 */
export async function getTrackData(wellId: string, curveNames: string[], targetPixelHeight: number): Promise<TrackCurveSeries[]> {
  const buf = await invoke<ArrayBuffer>("get_track_data", { wellId, curveNames, targetPixelHeight });
  return decodeCurveBuffer(buf);
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

/** Distinct constellation (log-set) names across the project — for the input/output
 *  constellation pickers in the module and workflow dialogs (which span many wells). */
export function listLogSetNames(): Promise<string[]> {
  return invoke<string[]>("list_log_set_names");
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

// --- Universal jobs (Phase 11): one progress/cancel channel for every long op -----------

/** One item (usually a well) within a job. Mirrors Rust `jobs::JobItem`. */
export interface JobItem {
  key: string;
  label: string;
  state: "pending" | "running" | "ok" | "warned" | "failed";
  message: string | null;
}

/** A running or finished job for the universal Processing panel. Mirrors `jobs::JobView`. */
export interface JobView {
  id: string;
  kind: string;
  label: string;
  phase: "queued" | "running" | "completed" | "cancelled" | "failed";
  total: number;
  done: number;
  current: string | null;
  items: JobItem[];
  error: string | null;
  seq: number;
}

/** Snapshot of every job, most recent first — polled by the Processing panel. */
export async function listJobs(): Promise<JobView[]> {
  return invoke<JobView[]>("list_jobs");
}

/** Requests cancellation of one job (flips the shared flag the runner checks per well). */
export async function cancelJob(jobId: string): Promise<void> {
  return invoke<void>("cancel_job", { jobId });
}

// --- Performance Monitor (Phase 11) -----------------------------------------------------

/** System-resource snapshot for the Performance panel. Percentages are 0..100; null =
 *  unavailable on this platform. Mirrors Rust `health::HealthSnapshot`. */
export interface HealthSnapshot {
  mem_system: number | null;
  cpu_load: number | null;
  user_objects: number | null;
  gdi_objects: number | null;
  user_count: number | null;
  gdi_count: number | null;
}

/** One cheap system-resource reading (memory / GPU memory / USER + GDI object counts). */
export async function healthSnapshot(): Promise<HealthSnapshot> {
  return invoke<HealthSnapshot>("health_snapshot");
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
  /** Restrict this entry's draw to one named zone (omit/null = well-wide). The same parameter
   *  may appear in several entries with different zones; a later entry wins where zones
   *  overlap. Unknown zone names come back as a note and leave the parameter at base values. */
  zone?: string | null;
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
  /** Low / high output percentiles (fractions in (0,1); default 0.10 / 0.90). One control drives
   *  both the reported spread and the tornado's input sweep. The median is always reported. */
  low_pctl?: number;
  high_pctl?: number;
  /** Retain draws and report Spearman rank correlation of each param vs each output metric. */
  sensitivity?: boolean;
  /** Run a one-at-a-time low/median/high sweep per parameter (the classic tornado range). */
  tornado?: boolean;
  /** Draw-matrix scheme: "lhs" (default — stratified Latin Hypercube, tighter CDF coverage at
   *  the same N) or "random" (legacy independent draws, byte-identical to pre-LHS results). */
  sampling?: "lhs" | "random";
  /** Target Spearman rank correlations between parameter pairs (Iman–Conover induction;
   *  marginals are only reordered, never altered). */
  correlations?: McCorrelation[];
  /** Track running lo/mid/hi convergence of per-well total HPV; in "random" mode the run stops
   *  early once the series is stationary (LHS designs always run to their full size). */
  converge?: boolean;
  /** Relative stationarity tolerance for the convergence check (default 0.005 = 0.5%). */
  converge_tol?: number;
  /** Persist per-sample uncertainty curves (MC_<KEY>_LOW/_P50/_HIGH/_BASE for each tracked
   *  output the chain produces) to a fresh version of the MONTECARLO log set per well. */
  persist?: boolean;
}

/** Target rank correlation between two MC parameters (rho clamped to ±0.995 backend-side). */
export interface McCorrelation {
  param_a: string;
  param_b: string;
  rho: number;
}

/** Low / median / high percentile + mean/sd of one metric across realizations. The lo/hi
 *  percentiles are the request's `low_pctl` / `high_pctl` (echoed on `McResult`). */
export interface Pctl {
  lo: number;
  mid: number;
  hi: number;
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

/** One output-metric bundle (net / NTG / avg PHIE / avg SWE / HPV) for a sweep point or a
 *  Spearman row. Fields are null when the backend value is NaN (Rust f32::NAN → JSON null):
 *  avg_phie/avg_swe of a no-pay sweep point, or a no-spread Spearman. Guard before math —
 *  `Math.min(null, x)` silently coerces null to 0. */
export interface McMetricSet {
  net: number | null;
  ntg: number | null;
  avg_phie: number | null;
  avg_swe: number | null;
  hpv: number | null;
}

/** Sensitivity of one Monte Carlo parameter within one zone. */
export interface McSensParam {
  param: string;
  /** Spearman rank correlation (−1..+1) of the param vs each metric; null unless requested. */
  spearman: McMetricSet | null;
  /** Output metrics at the param's P10 / median / P90 (others held at base); null unless tornado. */
  oat_low: McMetricSet | null;
  oat_base: McMetricSet | null;
  oat_high: McMetricSet | null;
}

export interface McSensZone {
  well_id: string;
  well_name: string;
  zone: string;
  params: McSensParam[];
}

/** One convergence checkpoint: running lo/mid/hi of per-well total HPV after `at` realizations. */
export interface McConvCheck {
  at: number;
  lo: number;
  mid: number;
  hi: number;
}

/** Per-well convergence trace (present only when `converge` was requested). */
export interface McConvergence {
  well_id: string;
  well_name: string;
  checks: McConvCheck[];
  converged: boolean;
  used_iterations: number;
  requested_iterations: number;
  note: string | null;
}

export interface McResult {
  zones: McZoneResult[];
  /** Per-zone parameter sensitivity (empty unless sensitivity/tornado was requested). */
  sensitivity: McSensZone[];
  /** Output percentiles actually used (echoed + clamped) so the UI can label the lo/hi columns. */
  low_pctl: number;
  high_pctl: number;
  /** Sampling scheme actually used ("lhs" / "random") — the UI badge. */
  sampling: string;
  /** Per-well convergence traces (empty unless `converge` was requested). */
  convergence: McConvergence[];
  /** Curve names written to the versioned MONTECARLO log set (empty unless `persist`). */
  persisted: string[];
  /** Non-fatal advisories (skipped correlation pairs, degenerate targets, …). */
  notes: string[];
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
  /** Optional flag curve: samples where the mask = 1 are excluded from training and left blank
   *  (NaN) in the prediction. Null = no masking. */
  mask_curve: string | null;
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

/** Model-comparison leaderboard (Wave B item 3). */
export interface MlEvalRequest {
  task: "regression" | "classification";
  feature_curves: string[];
  target_curve: string;
  train_well_ids: string[];
  algorithms: string[];
  /** Feature subsets to try (each a subset of feature_curves); empty → full set only. */
  subsets: string[][];
  standardize: boolean;
  seed: number;
  folds: number;
  /** Optional flag curve: masked (= 1) samples are excluded from the CV pool so the leaderboard
   *  scores the same population the real run trains on. Omit / null for no masking. */
  mask_curve?: string | null;
}

export interface MlEvalRow {
  algorithm: string;
  features: string[];
  /** Blind-well CV score: R² (regression) or accuracy (classification); null if it errored. */
  score: number | null;
  score_std: number | null;
  metrics: Record<string, unknown>;
  importances: number[];
  confusion: number[][] | null;
  labels: number[] | null;
  error: string | null;
}

export interface MlEvalResult {
  rows: MlEvalRow[];
  n_train: number;
  n_groups: number;
  cv: string;
  n_splits: number;
  note: string | null;
  error: string | null;
}

/** Ranks algorithm × feature-subset combos by blind-well (GroupKFold) CV, with permutation
 *  importance + confusion matrix. Evaluation only — writes no curves. */
export function runMlEval(req: MlEvalRequest): Promise<MlEvalResult> {
  return invoke<MlEvalResult>("run_ml_eval", { req });
}

/** Cuddy FOIL / BVW saturation-height fit (Wave B item 8, SHF side). */
export interface CuddyFoilRequest {
  well_ids: string[];
  phie_curve: string;
  sw_curve: string;
  tvdss_curve: string;
  /** Optional rock-type curve — when set, also fits one FOIL law per rounded RT class. */
  rt_curve?: string;
  fwl: number;
  min_phi: number;
  scan: boolean;
  scan_lo: number;
  scan_hi: number;
  scan_step: number;
}

export interface FoilPoint {
  h: number;
  bvw: number;
  well_id: string;
  /** Rounded rock-type class of the sample (null when no RT curve / RT is NaN). */
  rt: number | null;
}

/** One per-rock-type FOIL law (BVW = a·H^b over that RT class only). When the group's fit
 *  failed (`error` set), the numerics are Rust NaN → JSON null — guard before formatting. */
export interface FoilGroupFit {
  rt: number;
  a: number | null;
  b: number | null;
  r2: number | null;
  n_points: number;
  error: string | null;
}

export interface CuddyFoilResult {
  a: number;
  b: number;
  r2: number;
  n_points: number;
  fwl_used: number;
  fwl_best: number | null;
  points: FoilPoint[];
  scan: { fwl: number; residual: number }[];
  /** Per-rock-type fits when an RT curve was supplied (ascending RT class). */
  groups: FoilGroupFit[];
  /** [reason, count] of candidate samples excluded from the fit. */
  excluded: [string, number][];
  notes: string[];
  error: string | null;
}

/** Fits BVW = a·H^b (Cuddy FOIL) from computed PHIE/SW/TVDSS across wells; optionally scans the
 *  common FWL (Cuddy Eq 19). Evaluation only — writes no curves. */
export function runCuddyFoil(req: CuddyFoilRequest): Promise<CuddyFoilResult> {
  return invoke<CuddyFoilResult>("run_cuddy_foil", { req });
}

/** Height-domain SHF fit (Brooks-Corey / Skelt-Harrison) to the log-derived Sw-vs-height cloud. */
export interface ShfFitRequest {
  well_ids: string[];
  phie_curve: string;
  sw_curve: string;
  tvdss_curve: string;
  /** Working permeability curve — required by leverett_j only. */
  perm_curve?: string;
  /** Fluid props for height→Pc→J (leverett_j). Defaults: 1.0 / 0.7 g/cc, σ·cosθ 26 dyn/cm. */
  rho_w?: number;
  rho_hc?: number;
  ift_res?: number;
  /** Optional rock-type curve — when set, also fits one law per rounded RT class. */
  rt_curve?: string;
  fwl: number;
  min_phi: number;
  method: "brooks_corey" | "skelt" | "thomeer" | "leverett_j";
}

export interface ShfPoint {
  h: number;
  sw: number;
  well_id: string;
  /** Rounded rock-type class of the sample (null when no RT curve / RT is NaN). */
  rt: number | null;
}

/** One per-rock-type SHF law of the requested family. When the group's fit failed (`error`
 *  set), `params`/`curve` are empty and `r2` is Rust NaN → JSON null — guard before formatting. */
export interface ShfGroupFit {
  rt: number;
  params: [string, number][];
  r2: number | null;
  n_points: number;
  /** Sampled fitted Sw(H) curve over this group's height range. */
  curve: [number, number][];
  error: string | null;
}

export interface ShfFitResult {
  method: string;
  /** Named fitted parameters, e.g. [["swirr", …], ["he", …], ["lambda", …]]. */
  params: [string, number][];
  r2: number;
  n_points: number;
  points: ShfPoint[];
  /** Sampled fitted Sw(H) curve as [H, Sw] pairs. */
  curve: [number, number][];
  /** Per-rock-type fits when an RT curve was supplied (ascending RT class). */
  groups: ShfGroupFit[];
  /** [reason, count] of candidate samples excluded from the fit. */
  excluded: [string, number][];
  notes: string[];
  error: string | null;
}

export function runShfFit(req: ShfFitRequest): Promise<ShfFitResult> {
  return invoke<ShfFitResult>("run_shf_fit", { req });
}

/** Electrofacies tie-in QC: confusion matrix of a predicted log RT curve vs a reference/core RT. */
export interface FaciesConfusionRequest {
  well_ids: string[];
  pred_curve: string;
  ref_curve: string;
}

export interface RefClassRow {
  ref_label: number;
  dominant_pred: number;
  purity: number;
  count: number;
}

export interface FaciesConfusionResult {
  ref_labels: number[];
  pred_labels: number[];
  /** matrix[i][j] = count where reference == ref_labels[i] and prediction == pred_labels[j]. */
  matrix: number[][];
  per_ref: RefClassRow[];
  overall_purity: number;
  n: number;
  /** ANOVA variance reduction of log10(core k) grouped by the predicted class (1 − SS_within/
   *  SS_total): 1 = the typing explains all core-perm variance, 0 = none. `null` when no core
   *  plugs match or fewer than 2 classes carry plugs (Rust f64::NAN → JSON null). */
  k_var_reduction: number | null;
  /** Core plugs that contributed to `k_var_reduction`. */
  n_core_plugs: number;
  error: string | null;
}

export function runFaciesConfusion(req: FaciesConfusionRequest): Promise<FaciesConfusionResult> {
  return invoke<FaciesConfusionResult>("run_facies_confusion", { req });
}

// --- Generalized Multimin (multi-mineral inversion) -----------------------

/** A mineral, clay, or fluid component (fluids are zone-typed X/U). */
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
  /** Wet-clay porosity φ_clay (clays only, minerals 0). Drives the bound-water tie when the
   *  porosity source is `wet_clay_porosity`: k = φ/(1−φ). Techlog WCLP defaults on the library. */
  wet_clay_porosity: number;
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
  /** Shale resistivity (ohmm) at formation temperature — the 100%-shale Rt for the shaly-sand Sw
   *  models (Indonesia/Simandoux). Ignored by the dual-water model. Backend default 4.0. */
  rsh?: number;
  /** Archie tortuosity factor a (Indonesia/Simandoux). Dual-water uses a=1. Backend default 1.0. */
  archie_a?: number;
  /** Wet-clay (shale) total porosity φ_sh for Juhász's normalized Qv (Vsh·φ_sh/φt) and shale-point
   *  conductivity (1/(Rsh·φ_sh^m)). Only Juhász reads it. Backend default 0.10. */
  phit_sh?: number;
  /** Core-measured Waxman-Smits B override (mho·mL/(m·meq)). 0/blank ⇒ compute B(T,Rw) from the
   *  Juhász fit. Only the `waxman_smits` model reads it. Backend default 0. */
  ws_b?: number;
}

/** Saturation model for the conductivity tools. `linear_dw` (default) is the in-inversion linearised
 *  dual-water; the rest are post-solve forms (Sw from Rt + the solved volumes): `dual_water_nonlinear`
 *  and `archie` (total-porosity), `indonesia`/`simandoux`/`juhasz`/`waxman_smits` (shaly-sand). */
export type SwModel =
  | "linear_dw"
  | "dual_water_nonlinear"
  | "archie"
  | "indonesia"
  | "simandoux"
  | "juhasz"
  | "waxman_smits";

/** What drives the clay bound-water (BNDWAT) constraint. `cec` (default) uses
 *  α·96·CEC·ρ/(T+298); `wet_clay_porosity` uses the geometric k = φ/(1−φ) from each clay's
 *  `wet_clay_porosity`. The two agree for smectite (a degenerate WCLP falls back to CEC). */
export type PorositySource = "cec" | "wet_clay_porosity";

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
  /** Optional per-depth formation-temperature curve name (°F). When set and finite at a depth, the
   *  temperature-dependent fluid quantities (Cw, Cmf, Cbw, auto CT/CXO σ, BNDWAT k, Waxman-Smits B)
   *  are recomputed for that sample; a missing or out-of-range sample (a ±999.25 null) falls back to
   *  `fluid.ftemp_f`. */
  ftemp_curve?: string;
  /** Emit per-tool reconstruction QC curves: `<prefix>_<KEY>_REC` (measurement rebuilt from the
   *  solved volumes, display units) + `<prefix>_<KEY>_DIF` (σ-unit residual, that tool's RECON term). */
  recon_qc?: boolean;
  /** Saturation model for the conductivity tools (default `linear_dw` — nothing moves). */
  sw_model?: SwModel;
  /** What drives the BNDWAT constraint (default `cec` — nothing moves). */
  porosity_source?: PorositySource;
  /** Constraint enables (all default true / on — omit to leave the solver unchanged). POROSITY ties
   *  flushed/virgin porosity; BNDWAT ties clay bound water; WATER MUD keeps flushed water ≥ virgin
   *  water for water-based mud. UNITY is the `unity` flag above. */
  enforce_porosity?: boolean;
  enforce_bndwat?: boolean;
  enforce_water_mud?: boolean;
  /** Soft-constraint tolerance σ (row weight = 1/σ). Default 0.01; non-positive falls back to it. */
  sigma_constraint?: number;
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
  /** Model degrees of freedom = (tools + soft constraints + unity) − components. 0 = exactly
   *  determined (RECON forced to ~0, can't validate the model); >0 = a real fit-quality signal. */
  dof: number;
  /** Set when dof == 0 — a heads-up that the reconstruction can't discriminate the model. */
  dof_note: string | null;
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

export interface HighlightEntry {
  highlight_id: string;
  top_depth: number;
  bottom_depth: number;
  color: string | null;
  label: string | null;
}

export function listHighlights(wellId: string): Promise<HighlightEntry[]> {
  return invoke<HighlightEntry[]>("list_highlights", { wellId });
}

export function upsertHighlight(
  wellId: string,
  highlightId: string,
  topDepth: number,
  bottomDepth: number,
  color: string | null,
  label: string | null,
): Promise<void> {
  return invoke("upsert_highlight", { wellId, highlightId, topDepth, bottomDepth, color, label });
}

export function deleteHighlight(wellId: string, highlightId: string): Promise<void> {
  return invoke("delete_highlight", { wellId, highlightId });
}

/** A fluid contact (OWC/GWC/GOC/GDT/ODT/FWL). Scope: well_id set → that well; field_name set
 *  (well_id null) → every well in that field; both null → a global datum on every well. */
export interface FluidContact {
  contact_id: string;
  field_name: string | null;
  well_id: string | null;
  contact_type: string;
  depth: number;
  /** true → depth is TVDSS (draws flat across wells); false → measured depth. */
  is_tvdss: boolean;
  color: string | null;
  label: string | null;
}

export function listFluidContacts(): Promise<FluidContact[]> {
  return invoke<FluidContact[]>("list_fluid_contacts", {});
}

export function upsertFluidContact(c: FluidContact): Promise<void> {
  return invoke("upsert_fluid_contact", {
    contactId: c.contact_id,
    fieldName: c.field_name,
    wellId: c.well_id,
    contactType: c.contact_type,
    depth: c.depth,
    isTvdss: c.is_tvdss,
    color: c.color,
    label: c.label,
  });
}

export function deleteFluidContact(contactId: string): Promise<void> {
  return invoke("delete_fluid_contact", { contactId });
}

export interface ContactSuggestRequest {
  well_id: string;
  zone_top: number;
  zone_base: number;
  sw_curve?: string;
  res_curve?: string;
  nphi_curve?: string;
  rhob_curve?: string;
  sw_cutoff?: number;
}

export interface ContactCandidate {
  contact_type: string;
  depth: number;
  method: string;
  confidence: number;
  detail: string;
}

export interface ContactSuggestResult {
  candidates: ContactCandidate[];
  error: string | null;
}

/** Suggest a contact depth from logs (Sw crossover, resistivity drop, density-neutron). */
export function suggestContacts(req: ContactSuggestRequest): Promise<ContactSuggestResult> {
  return invoke<ContactSuggestResult>("suggest_contacts", { req });
}

export interface ContactWellResidual {
  well_id: string;
  well_name: string;
  tvdss: number;
  predicted: number;
  residual: number;
  flagged: boolean;
}

export interface ContactConsistency {
  contact_type: string;
  n: number;
  mean_tvdss: number;
  rms: number;
  /** [a, b, c] of z = a + b·x + c·y (dip plane), or null when the flat mean is used. */
  plane: [number, number, number] | null;
  wells: ContactWellResidual[];
  error: string | null;
}

/** Check whether every well's pick of a contact type agrees on a flat TVDSS surface. */
export function checkContactConsistency(contactType: string, flagAbs?: number): Promise<ContactConsistency> {
  return invoke<ContactConsistency>("check_contact_consistency", { contactType, flagAbs });
}

// --- Results-QC: Sw-method spread ---------------------------------------------------------------

/** Request for the per-depth Sw-method envelope. Curve names are optional — the backend tries a
 *  candidate list (first present wins) when a field is left blank. Qv/Swb curves are what pull the
 *  Waxman-Smits / Dual-Water models into the envelope; without them those two are skipped, never faked. */
export interface SwSpreadRequest {
  well_id: string;
  depth_min?: number | null;
  depth_max?: number | null;
  rt_curve?: string | null;
  phie_curve?: string | null;
  phit_curve?: string | null;
  vsh_curve?: string | null;
  qv_curve?: string | null;
  swb_curve?: string | null;
  fluid: MmFluidProps;
  /** Sw-unit gap above which a depth is flagged divergent. Backend default 0.10. */
  divergence_threshold?: number | null;
}

export interface SwMethodSeries {
  name: string;
  /** Sw per depth for this model; NaN → null where a sample was non-physical. */
  values: (number | null)[];
}

export interface SwSpreadResult {
  depth: number[];
  methods: SwMethodSeries[];
  sw_min: (number | null)[];
  sw_max: (number | null)[];
  spread: (number | null)[];
  mean_spread: number | null;
  max_spread: number | null;
  max_spread_depth: number | null;
  /** Fraction of comparable depths (≥2 models) whose spread exceeds the threshold. */
  frac_divergent: number | null;
  /** Comparable depths used in the summary stats. */
  n_samples: number;
  notes: string[];
}

/** Per-depth water-saturation envelope across the app's Sw models. Read-only — computes nothing to disk. */
export function swMethodSpread(req: SwSpreadRequest): Promise<SwSpreadResult> {
  return invoke<SwSpreadResult>("sw_method_spread", { req });
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
  /** Field Dashboard sets this true so its field-wide QC pass writes FLAG_* in place instead
   *  of versioning the pay flags (with the cutoffs in provenance) per well on every refresh. */
  skip_version?: boolean;
  /** Compute + return the per-zone stats WITHOUT persisting any FLAG_* curves. The Field
   *  Dashboard sets this: it only reads the returned rows, so writing flags per well on every
   *  cutoff tweak was the dominant cost. Flag persistence stays with Cutoffs & Summary. */
  stats_only?: boolean;
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
  // The Rust engine emits f32::NAN for zone×flag rows with no valid in-zone samples, and
  // Tauri/serde_json encodes non-finite floats as JSON null — so these arrive as null, not NaN.
  avg_vsh: number | null;
  avg_phie: number | null;
  avg_swe: number | null;
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
  const buf = await invoke<ArrayBuffer>("get_curve_data", { wellId, curveNames, depthMin, depthMax });
  return decodeCurveBuffer(buf);
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
  /** Lab fluid system ('air_brine', 'hg_air', 'oil_brine', ...); null = legacy import. */
  system: string | null;
  /** sigma·cosθ of that system (dyn/cm) as entered at import. */
  ift: number | null;
}

/** Imports a SCAL Pc/Sw CSV for the well and returns the Leverett-J fit (Sw = A·J^B)
 *  at the given lab sigma·cosθ, for carrying into the sw_height module. */
export function importScalCsv(wellId: string, path: string, iftLab: number): Promise<ScalImportResult> {
  return invoke<ScalImportResult>("import_scal_csv", { wellId, path, iftLab });
}

/** SCAL Pc file shapes the importer understands: "long" flat Pc/Sw rows, "porous_plate"
 *  Corelab-style wide tables (pressure columns × plug rows), "centrifuge" per-plug
 *  key-value blocks + Pc/Sw tables, or "auto" to sniff each file. */
export type ScalFormat = "auto" | "long" | "porous_plate" | "centrifuge";

/** Multi-file SCAL Pc import (e.g. a set of single-plug centrifuge exports): all files
 *  land in ONE combined replace-write of the well's scal_pc rows, with the Leverett-J
 *  fit over the pooled points. `system` labels every stored point with the lab fluid
 *  system ('air_brine', 'hg_air', 'oil_brine', ...) alongside the entered sigma·cosθ. */
export function importScalFiles(
  wellId: string,
  paths: string[],
  format: ScalFormat,
  system: string,
  iftLab: number,
): Promise<ScalImportResult> {
  return invoke<ScalImportResult>("import_scal_files", { wellId, paths, format, system, iftLab });
}

export interface ThomeerSampleFit {
  well_name: string;
  sample_no: number | null;
  /** null when the plug has no depth (serde encodes NaN floats as null too). */
  depth: number | null;
  /** null when the plug has no permeability (Rust NaN → JSON null). */
  perm: number | null;
  poro: number;
  /** Displacement pressure, psi — Hg-air EQUIVALENT when `standardized`. */
  pd: number;
  g: number;
  bv_inf: number;
  r2: number;
  n: number;
  /** Pd pinned at a search bound (entry-truncated curve) — an artifact, not a real Pd. */
  pd_at_bound: boolean;
  /** Lab fluid system of this plug's points (from import). */
  system: string | null;
  /** Every point carried a σcosθ and Pc was converted to Hg-air equivalent. */
  standardized: boolean;
  apex_bv_pc: number;
  /** Swanson k (mD); null when unstandardized or no apex (Rust NaN → JSON null). */
  swanson_k: number | null;
  /** (Pc, Bv) data points, Pc ascending (Hg-air equivalent when standardized). */
  scatter: [number, number][];
  /** Fitted (Pc, Bv) curve, log-spaced from just above Pd. */
  curve: [number, number][];
}

export interface ThomeerResult {
  fits: ThomeerSampleFit[];
  skipped: number;
  error: string | null;
}

/** Thomeer Pc hyperbola fit per plug over the selected wells' scal_pc points:
 *  Bv = Bv∞·exp(−G/log10(Pc/Pd)) — the (Pd, G) plane for Thomeer-class rock typing. */
export function runThomeerFit(wellIds: string[]): Promise<ThomeerResult> {
  return invoke<ThomeerResult>("run_thomeer_fit", { req: { well_ids: wellIds } });
}

export interface HfuCluster {
  /** 1..K, ascending FZI (HFU 1 = lowest FZI = poorest quality). */
  hfu: number;
  n: number;
  fzi_min: number;
  fzi_max: number;
  /** Geometric-mean FZI = the unit-slope RQI–φz line intercept at φz = 1. */
  fzi_gm: number;
  poro_mean: number;
  /** R² (log-k) of the per-HFU Amaefule perm transform vs measured k (1.0 for a single plug). */
  perm_r2: number;
}

export interface HfuPoint {
  well_name: string;
  depth: number | null;
  poro: number;
  perm: number;
  rqi: number;
  phiz: number;
  fzi: number;
  hfu: number;
}

export interface HfuResult {
  clusters: HfuCluster[];
  points: HfuPoint[];
  /** K−1 FZI cut values (ascending) separating the HFUs. */
  boundaries: number[];
  method: string;
  n_plugs: number;
  skipped: number;
  /** Set when the result deviated from the request (fewer clusters than asked, etc.). */
  note: string | null;
  error: string | null;
}

export type HfuMethod = "ward" | "histogram";

/** Cluster the scoped wells' core φ-k cloud into hydraulic flow units on log10(FZI). */
export function runHfuCluster(wellIds: string[], nClusters: number, method: HfuMethod): Promise<HfuResult> {
  return invoke<HfuResult>("run_hfu_cluster", {
    req: { well_ids: wellIds, n_clusters: nClusters, method },
  });
}

export interface LorenzPoint {
  depth: number;
  phi: number;
  perm: number;
  /** Cumulative storage fraction Σ(φh)/Σφh at this sample, 0..1 (depth order). */
  cum_storage: number;
  /** Cumulative flow fraction Σ(kh)/Σkh at this sample, 0..1 (depth order). */
  cum_flow: number;
  /** Local SMLP slope (k/φ normalized); 1 = the well-average k/φ (the 45° line). */
  slope: number;
  /** Flow-unit id, 1..K in depth order (unit 1 = shallowest). */
  unit: number;
}

export interface LorenzUnit {
  unit: number;
  depth_top: number;
  depth_base: number;
  n: number;
  /** Share of the well's total storage capacity (Σφh) in this unit. */
  storage_frac: number;
  /** Share of the well's total flow capacity (Σkh) in this unit. */
  flow_frac: number;
  /** Unit SMLP slope = flow_frac / storage_frac; >1 speed zone, <1 baffle. */
  slope: number;
  phi_mean: number;
  perm_mean: number;
  /** Advisory character: "speed" (slope>1), "baffle" (<1), "balanced", or "n/a". */
  character: string;
}

export interface LorenzResult {
  points: LorenzPoint[];
  units: LorenzUnit[];
  /** Lorenz heterogeneity coefficient (0 homogeneous … 1 heterogeneous). `null` on the error
   *  path (Rust emits f64::NAN → JSON null), same NaN→null convention as PaySummaryRow.avg_*. */
  lorenz_coefficient: number | null;
  total_kh: number;
  total_phih: number;
  n_samples: number;
  skipped: number;
  note: string | null;
  error: string | null;
}

/** Stratigraphic Modified Lorenz Plot: depth-ordered flow/storage-capacity curve for one well,
 *  segmented into flow units, with the Lorenz heterogeneity coefficient. `nUnits` 0 = auto. */
export function runLorenz(
  wellId: string,
  phiCurve: string,
  permCurve: string,
  nUnits: number,
  depthFrom?: number,
  depthTo?: number,
): Promise<LorenzResult> {
  return invoke<LorenzResult>("run_lorenz", {
    req: {
      well_id: wellId,
      phi_curve: phiCurve,
      perm_curve: permCurve,
      n_units: nUnits,
      depth_from: depthFrom ?? null,
      depth_to: depthTo ?? null,
    },
  });
}

export function getScalPc(wellId: string): Promise<ScalPcRow[]> {
  return invoke<ScalPcRow[]>("get_scal_pc", { wellId });
}

/** Fetches a well's core plug data as CPOR/CPERM/CGD/CSW series (each only its own
 *  non-NaN samples, at their own depths — not resampled onto the log depth grid). */
export async function getCoreData(wellId: string): Promise<TrackCurveSeries[]> {
  const buf = await invoke<ArrayBuffer>("get_core_data", { wellId });
  return decodeCurveBuffer(buf);
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
  /** True when the user promoted this curve to win its (well, set, mnemonic) group. */
  pinned: boolean;
}

/** Every curve in the generic store for one well, across RAW/EDIT/FINAL sets. */
export function listGenericCurveCatalog(wellId: string): Promise<GenericCurveCatalogEntry[]> {
  return invoke<GenericCurveCatalogEntry[]>("list_generic_curve_catalog", { wellId });
}

/** Deletes one generic-store curve (meta + samples) — removes a shadowing/duplicate import. */
export function deleteGenericCurve(curveId: string): Promise<void> {
  return invoke<void>("delete_generic_curve", { curveId });
}

/** Promotes one generic curve so it wins its (well, set, mnemonic) group in curve resolution
 *  (the DLIS/LAS same-mnemonic shadow tiebreak). */
export function promoteGenericCurve(curveId: string): Promise<void> {
  return invoke<void>("promote_generic_curve", { curveId });
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

/** Per-well outcome of `materializeTvd`. `samples` = points written for each of TVD/TVDSS
 *  (0 = no survey or no logs yet); `has_survey` distinguishes "no survey" from "survey, no logs". */
export interface TvdMaterializeResult {
  well_id: string;
  well_name: string;
  samples: number;
  has_survey: boolean;
}

/** Rebuilds TVD/TVDSS computed curves from each well's deviation survey onto its log depth
 *  grid, so sw_height's TVD input, the SHF fits, and the TVDSS correlation view can fetch them.
 *  Deviation import already does this automatically; use this after importing logs later or
 *  editing the KB datum. Wells with no survey or no logs report samples = 0. */
export function materializeTvd(wellIds: string[]): Promise<TvdMaterializeResult[]> {
  return invoke<TvdMaterializeResult[]>("materialize_tvd", { wellIds });
}

export interface DlisImportResult {
  path: string;
  curves_imported: number;
  rows: number;
  /** Existing RAW curves at the same (mnemonic, run) that this import overwrote. */
  replaced: number;
  error: string | null;
}

/** Imports every scalar channel of a DLIS file into the selected well's generic store
 *  (via dlisio through the Python subprocess). */
export function importDlisFile(wellId: string, path: string): Promise<DlisImportResult> {
  return invoke<DlisImportResult>("import_dlis_file", { wellId, path });
}
