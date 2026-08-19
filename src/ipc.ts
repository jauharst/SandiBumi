import { invoke } from "@tauri-apps/api/core";
import type { DepthUnit } from "./units";
import type { TrackCurveRequest } from "./trackCurveRequest";
import type { PlotAxisRangeExport } from "./ui/axisRange";
import type { ChartRenderRecord } from "./ui/chartProvenance";
import type { PlotStatisticsRecord } from "./ui/plotCanvas";

/** The project's stored depth unit, plus whether it was explicitly declared (false = a
 *  fresh project that will adopt the unit of its first import). */
export function getProjectDepthUnit(): Promise<[DepthUnit, boolean]> {
  return invoke("get_project_depth_unit");
}

/** Declares the project's stored depth unit. The backend REFUSES once wells exist —
 *  re-declaring would reinterpret every stored depth rather than convert it. */
export function setProjectDepthUnit(unit: DepthUnit): Promise<void> {
  return invoke("set_project_depth_unit", { unit });
}

/** Quantity families backed by at least one reviewed numeric unit transform. */
export function listConvertibleUnitFamilies(): Promise<string[]> {
  return invoke<string[]>("list_convertible_unit_families");
}

/** The one project-wide absent-value sentinel supplied to every data writer. */
export function getProjectNullSentinel(): Promise<number> {
  return invoke("get_project_null_sentinel");
}

/** Declares the project-wide absent-value sentinel. It must be finite. */
export function setProjectNullSentinel(nullSentinel: number): Promise<void> {
  return invoke("set_project_null_sentinel", { nullSentinel });
}

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
  /** Succeeded, but not every sample did — a Rhai script that raised on some depths. Distinct
   *  from `error`: the curve was written and is usable, it just has holes the user should know
   *  about (`docs/review_triage.md` finding 13). */
  note: string | null;
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

export function runEquation(
  equationId: string,
  scope: BackendWellScope,
  custody: RunCustody,
): Promise<EquationRunResult[]> {
  return invoke<EquationRunResult[]>("run_equation", { equationId, scope, custody });
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

export function listWells(scope: BackendWellScope = { kind: "active_group" }): Promise<WellSummary[]> {
  return invoke<WellSummary[]>("list_wells", { scope });
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
export function wellsInPolygon(
  polygon: [number, number][],
  scope: BackendWellScope = { kind: "active_group" },
): Promise<WellSummary[]> {
  return invoke<WellSummary[]>("wells_in_polygon", { polygon, scope });
}

export interface ImportResult {
  path: string;
  well_id: string | null;
  well_name: string | null;
  /** Every non-comment LAS ~W record in source order; unknown mnemonics remain raw. */
  well_headers: Array<{
    raw: string;
    mnemonic: string;
    mapped_field:
      | "well_name"
      | "uwi"
      | "country"
      | "state"
      | "well_status"
      | "null_value"
      | "step"
      | null;
  }>;
  rows: number;
  /** Encoding selected by the mandatory byte-tolerant text reader. */
  text_encoding: string | null;
  /** Non-fatal note for a successful import (e.g. rows dropped for a bad/duplicate depth). */
  warning: string | null;
  error: string | null;
  /** Set the curves landed under when this file ATTACHED to an existing well instead of
   *  creating a new record. null = a new well was created. */
  attached_set: string | null;
  /** Every standard target for which multiple LAS aliases competed, including coverage. */
  alias_decisions: Array<{
    target: string;
    chosen: string;
    candidates: Array<{ mnemonic: string; finite_samples: number; chosen: boolean }>;
    table_entry: string | null;
  }>;
  /** Effective per-source-channel null handling; unset still uses ordinary LAS screening. */
  null_resolutions: Array<{
    channel: string;
    mode: "unset" | "no_null" | "values";
    values: number[];
  }>;
  index_resolution: {
    column: number;
    mnemonic: string;
    mechanism: "structural_declaration" | "positional_guarantee" | "name_alias" | "user_designation";
  } | null;
  /** Versioned policy governing LAS section order and tolerated malformed/unknown headers. */
  section_policy: string;
  /** Every non-fatal section tolerance that fired, in source order. */
  section_handling: Array<{
    line: number;
    header: string;
    action: "unknown_section_ignored" | "malformed_header_ignored" | "out_of_order_section_accepted";
  }>;
  /** Every automatic value conversion performed by the importer. */
  unit_conversions: Array<{
    curve: string;
    from_unit: string;
    to_unit: string;
    factor: number;
    /** Source-space offset: canonical = (source + offset) × factor. */
    offset: number;
    derivation: string;
  }>;
  /** Declared units retained verbatim because no reviewed conversion applied. */
  unconverted_units: Array<{
    curve: string;
    declared_unit: string;
    family: string | null;
    reason: string;
    designation_required: boolean;
    rejected_entry: string | null;
  }>;
  /** Per-file answers to genuinely ambiguous unit symbols. */
  unit_designations: Array<{
    curve: string;
    declared_unit: string;
    meaning: "microseconds_per_foot" | "millisiemens_per_foot";
    recorded_unit: string;
    family: string | null;
  }>;
  /** Verbatim observed unit spellings and only their explicitly registered interpretations. */
  unit_tokens: Array<{
    curve: string;
    state: "missing_unit" | "recognized" | "unrecognized";
    raw_token: string | null;
    canonical_unit: string | null;
    quantity_kind:
      | "gamma_ray"
      | "electric_potential"
      | "length"
      | "bulk_density"
      | "photoelectric_factor"
      | "fraction"
      | "slowness"
      | "temperature"
      | "resistivity"
      | "charge_per_volume"
      | "permeability"
      | null;
    explicit_alias: string | null;
  }>;
  /** Look-alike spellings kept distinct because the registry declares no alias. */
  unit_token_warnings: string[];
  /** SB-CLY-034 (DEC-037): set when the import BLOCKED on undeclared vendor-sentinel values.
   *  Nothing from this file was written; re-invoke with `undeclaredSentinelDecision`. */
  sentinel_question: {
    value: number;
    curves: Array<{ mnemonic: string; samples: number }>;
  } | null;
}

/** Container-owned identity plus a filename proposal that is never silently selected. */
export interface LasWellIdentityProbe {
  path: string;
  container_well_name: string | null;
  filename_proposal: string | null;
}

export function probeLasWellIdentities(paths: string[]): Promise<LasWellIdentityProbe[]> {
  return invoke<LasWellIdentityProbe[]>("probe_las_well_identities", { paths });
}

/** Import-sets choices from the Import LAS dialog (T-IMP-02, the Geolog/IP set model). */
export interface LasImportOptions {
  /** Set name for every curve of this batch; auto-suffixed per well (`FPROOH` → `FPROOH_1`)
   *  so a re-import never overwrites an earlier delivery. Empty/omitted → RAW. */
  setName?: string | null;
  /** Attach a file whose well name already exists to that well as a new set, instead of
   *  creating a duplicate well record. Defaults to true backend-side. */
  attach?: boolean;
  /** Explicit unit for files whose index has no usable declaration. Omit to refuse them. */
  fileDepthUnit?: "M" | "FT" | null;
  /** Resolved per-channel plural nulls. A named channel is screened only against its own list. */
  channelNulls?: Record<string, number[] | "NoNull">;
  /** One vendor exception entry may own many regex names and plural nulls, or explicit NoNull. */
  nullRules?: Array<{ names: string[]; nulls: number[] | "NoNull" }>;
  /** Required only when the index descends; absent means block before commit. */
  nonMonotonicIndex?: "accept_as_delivered" | null;
  /** Required only when repeated depths are present; absent means block before commit. */
  duplicateDepthPolicy?: "keep-first" | "keep-last" | "mean" | "refuse" | null;
  /** Explicit MS/FT meanings keyed by the exact source path; no entry means refuse. */
  msPerFtMeanings?: Record<string, "microseconds_per_foot" | "millisiemens_per_foot">;
  /** Explicit unit for DRHO-family channels whose source declaration is absent. */
  undeclaredDrhoUnit?: "g/cc" | "kg/m3" | null;
  /** Explicit confirmations keyed by source path, consulted only when the container has no identity. */
  confirmedWellNames?: Record<string, string>;
  /** Required set-level declaration; it is never inferred from coincidentally regular depths. */
  samplingStyle?: "CONTINUOUS_REGULAR" | "CONTINUOUS_IRREGULAR" | null;
  /** Required only for regular sets. No production default ships. */
  samplingStyleVerifyTolerance?: { value: number; unit: "M" | "FT" } | null;
  /** SB-CLY-034 (DEC-037): the user's answer to the undeclared-sentinel question. Absent
   *  while candidates exist blocks the import with the question; nothing converts on
   *  magnitude alone. */
  undeclaredSentinelDecision?: "convert" | "keep" | null;
}

export function importLasFiles(paths: string[], opts?: LasImportOptions): Promise<ImportResult[]> {
  return invoke<ImportResult[]>("import_las_files", {
    paths,
    setName: opts?.setName ?? null,
    attach: opts?.attach ?? null,
    fileDepthUnit: opts?.fileDepthUnit ?? null,
    channelNulls: opts?.channelNulls ?? null,
    nullRules: opts?.nullRules ?? null,
    nonMonotonicIndex: opts?.nonMonotonicIndex ?? null,
    duplicateDepthPolicy: opts?.duplicateDepthPolicy ?? null,
    msPerFtMeanings: opts?.msPerFtMeanings ?? null,
    undeclaredDrhoUnit: opts?.undeclaredDrhoUnit ?? null,
    confirmedWellNames: opts?.confirmedWellNames ?? null,
    samplingStyle: opts?.samplingStyle ?? null,
    samplingStyleVerifyTolerance: opts?.samplingStyleVerifyTolerance ?? null,
    undeclaredSentinelDecision: opts?.undeclaredSentinelDecision ?? null,
  });
}

export interface TopEntry {
  top_name: string;
  /** MD depth consumed by existing log/correlation/zone surfaces. */
  depth: number;
  /** Delivered source value and its recorded reference; null marks a pre-custody legacy row. */
  source_depth: number;
  source_depth_datum: DepthDatum | null;
  color: string | null;
}

export function listTops(wellId: string): Promise<TopEntry[]> {
  return invoke<TopEntry[]>("list_tops", { wellId });
}

/** Crossing warnings: top pairs in this well whose order contradicts most other wells. */
export function checkTopOrder(
  wellId: string,
  scope: BackendWellScope = { kind: "active_group" },
): Promise<string[]> {
  return invoke<string[]>("check_top_order", { wellId, scope });
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

export function autocorrelateTop(req: AutoCorrRequest, scope: BackendWellScope): Promise<AutoCorrResult> {
  return invoke<AutoCorrResult>("autocorrelate_top", { req, scope });
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
export function autocorrelateMulti(req: MultiAutoCorrRequest, scope: BackendWellScope): Promise<MultiAutoCorrResult> {
  return invoke<MultiAutoCorrResult>("autocorrelate_multi", { req, scope });
}

export type ScaleType = "linear" | "log";

export interface CurveStyle {
  curve_name: string;
  /** Explicit imported set: display this curve on that set's native depth grid. Blank keeps
   *  the established current standard/computed/RAW resolution. */
  set_name?: string;
  color: string;
  min: number;
  max: number;
  /** "line" (default) joins consecutive sample centres; "step" holds each sample's value
   *  across its whole sampling interval and then jumps — the blocky display for anything
   *  genuinely piecewise-constant (block-averaged logs, zone-constant parameter curves). */
  draw_style?: "line" | "step";
  /** Edge shading, "curve" for crossover shading against `fill_to`, or "blocks" for discrete
   *  class curves (facies) rendered as full-track-width colored intervals. */
  fill?: "none" | "left" | "right" | "curve" | "blocks";
  /** `fill: "curve"` — the reference curve to shade against. It must be another curve in the
   *  SAME track, and it is positioned with ITS OWN min/max (compatible scaling). */
  fill_to?: string;
  fill_color?: string;
  /** `fill: "curve"` — colour where this curve reads to the RIGHT of `fill_to`;
   *  `fill_color` covers the left side. */
  fill_color2?: string;
  fill_opacity?: number;
}

/** One measured-sample series drawn in a `point_data` track. Kept separate from CurveStyle
 *  on purpose: a curve has a value at every depth, a point series has values only where
 *  somebody sampled, and joining core plugs with a line would state a continuity the data
 *  does not have. */
export interface PointStyle {
  /** "core" reads a plug property (CPOR, CPERM, CGD, CSW …) from the ACTIVE core set;
   *  "aux" reads an item from a point dataset (XRD, CEC, oil show, core extras …). */
  source: "core" | "aux";
  /** For `source: "aux"`, which dataset. Ignored for core. */
  dataset?: string;
  /** The property (core) or item (aux) name within that source. */
  item: string;
  color: string;
  min: number;
  max: number;
  /** "points" (default) | "box" | "histogram" | "text". */
  display?: "points" | "box" | "histogram" | "text";
  /** Depth-bin height for "box" / "histogram", in the project's depth unit. */
  bin?: number;
  /** Box edges as percentiles (defaults 25 / 75). */
  box_lo?: number;
  box_hi?: number;
  /** "tukey" (default) | "percentile" | "minmax" — an interpretive choice, so it is stored
   *  with the layout rather than assumed. */
  whisker?: "tukey" | "percentile" | "minmax";
  /** Tukey multiplier (default 1.5). */
  whisker_k?: number;
  /** Percentile pair when `whisker: "percentile"` (defaults 10 / 90). */
  whisker_lo?: number;
  whisker_hi?: number;
  /** Bin count across the value axis for "histogram" (default 12). */
  hist_bins?: number;
  /** Draw the individual samples on top of a box/histogram glyph. */
  show_samples?: boolean;
}

/** One array log — a curve holding a whole DISTRIBUTION at every depth (Monte Carlo
 *  realizations, an NMR T2 distribution, a sonic waveform) rather than a single reading.
 *
 *  The three displays are three readings of the SAME stored matrix: with the realizations on
 *  disk, moving P10 to P5 is a redraw, not a re-run of the study. */
export interface ArrayStyle {
  curve_name: string;
  /** Which array set. Absent = whichever set holds the curve. */
  set_name?: string;
  color: string;
  min: number;
  max: number;
  /** "band" (default) | "spaghetti" | "heatmap". */
  display?: "band" | "spaghetti" | "heatmap";
  /** Band edges as percentiles (defaults 10 / 90) — the adjustable part. */
  band_lo?: number;
  band_hi?: number;
  /** Draw the P50 line inside the band (default true). */
  show_median?: boolean;
  fill_opacity?: number;
  /** "spaghetti": how many realizations to draw (default 40). */
  traces?: number;
  /** "heatmap": bins across the value axis (default 32). */
  hist_bins?: number;
}

/** One picture series in an `image` track — thin sections, core photographs, SEM plates.
 *
 *  A picture has no value axis, so this shares nothing with a curve or a point series except
 *  the depth column. What it shares is their honesty about depth: a plate is drawn where it
 *  was sampled, and when two would overlap the second is skipped rather than nudged. */
export interface ImageStyle {
  /** Which image dataset ('THIN SECTION', 'CORE PHOTO' …); the ACTIVE delivery is drawn. */
  dataset: string;
  /** "anchor" (default) centres a fixed-size plate on its depth — the honest display for a
   *  thin section, which has no thickness. "depth" stretches the picture across its
   *  depth_top..depth_base interval, which a core photograph genuinely occupies. */
  mode?: "anchor" | "depth";
  /** Width as a fraction of the track (0.05..1, default 0.9). */
  size?: number;
  /** "contain" (default, whole picture visible) | "cover" (fill the box, crop the overhang).
   *  There is no stretch option: a distorted thin section misstates grain shape. */
  /** "contain" fits the whole picture in its box, "cover" crops it to fill, "stretch" fills it
   *  exactly. "stretch" is for depth STRIPS only — a picture whose height is the depth scale and
   *  whose width is the track has no shape of its own to preserve. A thin section is never
   *  stretched: its delivered shape is the truth. */
  fit?: "contain" | "cover" | "stretch";
  align?: "left" | "center" | "right";
  label?: boolean;
  border?: boolean;
}

export interface Track {
  title: string;
  width_weight: number;
  scale_type: ScaleType;
  /** "curves" (normal log track), "well_diagram" (casing/shoe/perforations), "point_data"
   *  (measured samples — core plugs, XRD, CEC …), "array_log" (a distribution at every
   *  depth) or "image" (depth-registered pictures). Optional for backward compat with saved
   *  layouts; absent = "curves". */
  kind?: "curves" | "well_diagram" | "point_data" | "array_log" | "image";
  curves: CurveStyle[];
  /** Drawn only when `kind: "point_data"`. Absent in every layout saved before point tracks. */
  points?: PointStyle[];
  /** Drawn only when `kind: "array_log"`. Absent in every layout saved before array tracks. */
  arrays?: ArrayStyle[];
  /** Drawn only when `kind: "image"`. Absent in every layout saved before image tracks. */
  images?: ImageStyle[];
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
  ancestry: CurveAncestryDisclosure[];
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
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  /** Explicit operator and source/reference for the PDF pay-summary run. */
  custody?: RunCustody;
  composite: CompositeSpec;
  title: string;
  author: string;
  /** Empty = the built-in default methodology template. */
  methodology: MethodRow[];
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min?: CutoffSpec | null;
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
export function exportReportBatch(spec: ReportSpec, scope: BackendWellScope, destDir: string): Promise<string[]> {
  return invoke<string[]>("export_report_batch", { spec, scope, destDir });
}

// --- Office deliverables (office.rs, Python subprocess) ---------------------

/** Which office-document packages the Python SandiBumi found can actually import. `python` is
 *  the interpreter path, so a "not installed" message can name the environment to install into. */
export interface OfficeSupport {
  python: string | null;
  xlsxwriter: boolean;
  docx: boolean;
  pptx: boolean;
  openpyxl: boolean;
  pillow: boolean;
  /** The deck needs BOTH pptx and matplotlib — python-pptx assembles the slides, matplotlib
   *  draws the figures they carry. */
  matplotlib: boolean;
  messages: Record<string, string>;
  package_versions: Record<string, string | null>;
  probe_error: string | null;
}

export interface DeckSpec {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  well_ids: string[];
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min?: CutoffSpec | null;
  title?: string;
  author?: string;
  /** Which cutoff level the deck summarises (default PAY). */
  flag?: string;
}

export interface DeckResult {
  path: string;
  slides: number;
  wells: number;
  wells_with_results: number;
  bytes: number;
}

/** Asset-team deck built from the pay-summary DATA (matplotlib figures), not from composite
 *  pages — a log plot at 1:200 stops being at 1:200 once it is a picture on a slide. */
export function exportDeck(spec: DeckSpec, scope: BackendWellScope, destPath: string): Promise<DeckResult> {
  return invoke<DeckResult>("export_deck", { spec, scope, destPath });
}

export interface WorkbookSpec {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  well_ids: string[];
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min?: CutoffSpec | null;
  title?: string;
  include_pay?: boolean;
  include_field?: boolean;
  include_zone_params?: boolean;
}

export interface WorkbookResult {
  path: string;
  sheets: number;
  wells: number;
  /** Wells that produced at least one interpreted zone row — the rest are named on the Summary
   *  sheet rather than silently missing. */
  wells_with_results: number;
  pay_rows: number;
  bytes: number;
}

export function officeSupport(): Promise<OfficeSupport> {
  return invoke<OfficeSupport>("office_support");
}

/** The EDITABLE Word twin of the report PDF — same title, author, methodology and cutoffs.
 *  The PDF stays the default deliverable and keeps the composite log pages. */
export function exportReportDocx(spec: ReportSpec, destPath: string): Promise<string> {
  return invoke<string>("export_report_docx", { spec, destPath });
}

export function exportReportDocxBatch(spec: ReportSpec, scope: BackendWellScope, destDir: string): Promise<string[]> {
  return invoke<string[]>("export_report_docx_batch", { spec, scope, destDir });
}

/** Writes the study as a formatted multi-sheet .xlsx. Never persists FLAG curves — an export
 *  must not churn the project (see office.rs). */
export function exportWorkbook(spec: WorkbookSpec, scope: BackendWellScope, destPath: string): Promise<WorkbookResult> {
  return invoke<WorkbookResult>("export_workbook", { spec, scope, destPath });
}

/** Exact project scope whose current computed curves contribute numbers to an exported plot.
 *  `curves: []` explicitly means the plot carries no project curve; `allProject` is reserved for
 *  free-form charts whose query cannot be narrowed without inventing a binding. */
export interface PaperExportBounds {
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

/** Machine-checkable paper/raster custody embedded with a plot artifact. Vector page geometry is
 *  recorded in physical points; raster backing geometry stays honestly labelled pixels. Neither
 *  route invents an A4/Letter default. */
export interface PaperExportRecord {
  schema_version: 1;
  medium: "svg-vector" | "pdf-vector" | "print-raster";
  unit: "pt" | "px";
  source_width: number;
  source_height: number;
  margin_pt: number;
  content_bounds: PaperExportBounds;
  page_bounds: PaperExportBounds;
  provenance_footer: string;
  crop_proof: "all_recorded_bounds_inside_page" | "raster_pixels_preserved_before_browser_print_layout";
}

export interface PlotAncestryScope {
  wellIds: string[];
  curves?: string[];
  allProject?: boolean;
  /** Exact binding record captured by the reads that produced the visible marks. */
  plotBindings?: PlotChannelBinding[];
  /** Exact displayed limits and the precedence tier that supplied each quantitative axis. */
  axisRanges?: PlotAxisRangeExport[];
  /** Complete statistics custody for every numeric summary visible on the plot. */
  statisticsRecords?: PlotStatisticsRecord[];
  /** Complete identity and source custody for an optional chart payload. */
  chartRenderRecord?: ChartRenderRecord;
  /** Paper geometry, crop proof, output medium and visible footer for a plot deliverable. */
  paperExportRecord?: PaperExportRecord;
}

function ancestryArgs(scope?: PlotAncestryScope): Record<string, unknown> {
  return {
    ancestryWellIds: scope ? scope.wellIds : null,
    ancestryCurveNames: scope?.curves ?? null,
    ancestryAllProject: scope?.allProject ?? false,
    plotBindings: scope?.plotBindings ?? null,
    axisRanges: scope?.axisRanges ?? null,
    statisticsRecords: scope?.statisticsRecords ?? null,
    chartRenderRecord: scope?.chartRenderRecord ?? null,
    paperExportRecord: scope?.paperExportRecord ?? null,
  };
}

/** Writes a base64-encoded PNG or SVG. When scoped, the backend resolves and embeds ancestry. */
export function savePng(destPath: string, dataBase64: string, scope?: PlotAncestryScope): Promise<string> {
  return invoke<string>("save_png", { destPath, dataBase64, ...ancestryArgs(scope) });
}

/** Assembles a single-chart PDF from a frontend-built content stream (points, bottom-left origin;
 *  see pdfExport.ts) and writes it to `destPath`. Returns the written path. */
export function savePlotPdf(
  destPath: string,
  content: string,
  widthPt: number,
  heightPt: number,
  scope?: PlotAncestryScope,
): Promise<string> {
  return invoke<string>("save_plot_pdf", { destPath, content, widthPt, heightPt, ...ancestryArgs(scope) });
}

/** Resolves the same backend-owned metadata for clipboard and print artifacts. */
export function getCurveAncestryDisclosures(scope: PlotAncestryScope): Promise<CurveAncestryDisclosure[]> {
  return invoke<CurveAncestryDisclosure[]>("get_curve_ancestry_disclosures", ancestryArgs(scope));
}

/** Writes a backend-validated plot reduction manifest to a user-picked path. */
export function savePlotReductionManifest(destPath: string, content: string): Promise<string> {
  return invoke<string>("save_plot_reduction_manifest", { destPath, content });
}

/** A free-form net-reservoir polygon drawn on a crossplot: vertices in DATA space (axis order),
 *  the axes' log flags, and the output curve name. Inside → 1, outside → 0, undefined → NaN. */
/** Field names are snake_case because they cross the wire into `netflag.rs`'s serde structs,
 *  which carry no `rename_all` — same convention as every other DTO here (see LorenzResult,
 *  ZoneParamEntry, HighlightEntry). Tauri camel-cases only the top-level command ARGUMENT key
 *  (`{ spec }`), never the fields inside it; `rename_all` is used only on enums, for their
 *  string tag values. `netflag.rs` has a test that reads this interface and fails on drift. */
export interface NetFlagSpec {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  well_id: string;
  x_curve: string;
  y_curve: string;
  x_log: boolean;
  y_log: boolean;
  polygon: [number, number][];
  output_curve: string;
  depth_top: number | null;
  depth_bottom: number | null;
  custody: RunCustody;
}

export interface NetFlagResult {
  output_curve: string;
  inside: number;
  evaluated: number;
  written: number;
}

/** Computes a net-flag curve from a crossplot polygon and writes it as a computed curve. */
export function runNetFlag(spec: NetFlagSpec): Promise<NetFlagResult> {
  return invoke<NetFlagResult>("run_net_flag", { spec });
}

export interface DocumentEntry {
  name: string;
  json: string;
}

/** Named JSON documents (saved layouts, plot property sets, ...). */
export function saveDocument(docType: string, name: string, json: string): Promise<void> {
  return invoke<void>("save_document", { docType, name, json });
}

export function savePlotState(docType: string, name: string, state: PersistedPlotState): Promise<void> {
  return invoke<void>("save_plot_state", { docType, name, state });
}

export function saveSessionDocument(name: string, json: string): Promise<void> {
  return invoke<void>("save_session_document", { name, json });
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
export async function getTrackData(
  wellId: string,
  curves: Array<string | TrackCurveRequest>,
  targetPixelHeight: number,
  depthMin?: number,
  depthMax?: number,
): Promise<TrackCurveSeries[]> {
  const curveRequests: TrackCurveRequest[] = curves.map((curve) =>
    typeof curve === "string" ? { curve_name: curve } : curve,
  );
  const buf = await invoke<ArrayBuffer>("get_track_data", {
    wellId,
    curveRequests,
    targetPixelHeight,
    depthMin,
    depthMax,
  });
  return decodeCurveBuffer(buf);
}

// ---------------------------------------------------------------------------
// Deterministic workflow: module manifests, zones, runner, pay summary
// ---------------------------------------------------------------------------

/** `text` is a free-typed run option — same `opts` channel as `option`, but the valid values are
 *  not a list the manifest can hold (the Condition family's user-named output curve). */
export type ArgKind = "param" | "option" | "text" | "log_in" | "log_out";
export type FlagKind = "EXCLUSION_MASK" | "DIAGNOSTIC_INDICATOR";

export type PorosityModuleRole =
  | "DETERMINISTIC_METHOD"
  | "COMPARISON_PRODUCER"
  // DEC-038 (2026-08-17): ssc/sspw are separately typed workflows.
  | "TYPED_WORKFLOW"
  | "LIMIT_PRODUCER";
export type PorosityOutputRole =
  | "UNLIMITED_EFFECTIVE"
  | "UNLIMITED_TOTAL"
  | "LIMITED_EFFECTIVE"
  | "LIMITED_TOTAL"
  | "COMPARISON_UNLIMITED_EFFECTIVE"
  | "COMPARISON_UNLIMITED_TOTAL"
  | "COMPARISON_LIMITED_EFFECTIVE"
  | "COMPARISON_LIMITED_TOTAL"
  | "EFFECTIVE"
  | "TOTAL"
  | "FREE_FLUID"
  | "CAPPED"
  | "CEILING";
export type PorosityFlagEmission = "PENDING_SB_POR_003";

export interface PorosityOutputContract {
  family: string;
  module_role: PorosityModuleRole;
  method: string;
  convention: string;
  output_role: PorosityOutputRole;
  limiting_contract: string;
  limiting_policy: string;
  limiting_policy_source: string;
  flag_contract: string;
  flag_emission: PorosityFlagEmission;
  output_naming_contract: string;
}

export interface ValidityBranch {
  argument: string;
  equals: string;
}

/** Backend-owned multi-well scope identity. Group and All carry no frontend snapshot: Rust resolves
 * their current membership at operation start. Explicit is the intentional Active/Pinned/
 * Selection/Custom alternative and Rust verifies that every identity still exists. */
export type BackendWellScope =
  | { kind: "active_group" }
  | { kind: "group"; group_id: string }
  | { kind: "all" }
  | { kind: "explicit"; well_ids: string[] };

/** Resolves a scope in Rust. Multi-well reads use this to avoid drawing a stale group snapshot;
 * mutating/export commands take the same selector and resolve it inside their own command. */
export function resolveWellScope(scope: BackendWellScope): Promise<string[]> {
  return invoke<string[]>("resolve_well_scope", { scope });
}

export type ValidityCondition = {
  /** Stable id repeated in runner refusals and persisted manifests. */
  id: string;
  /** Human explanation shown beside the affected field. */
  statement: string;
  /** Named source for the condition; never an implied or guessed endpoint. */
  source: string;
} & (
  | { kind: "enumeration" }
  | { kind: "numeric_range"; min: number | null; max: number | null; unit: string; when?: ValidityBranch | null }
  | { kind: "required_companion"; any_of: string[]; when?: ValidityBranch | null }
  | { kind: "required_value"; when?: ValidityBranch | null }
  | { kind: "required_where_finite"; input: string }
  | { kind: "less_than"; other: string }
);

export interface ArgSpec {
  name: string;
  desc: string;
  unit: string;
  kind: ArgKind;
  /** Semantic role of a binary flag output; absent for ordinary numeric/class channels. */
  flag_kind?: FlagKind | null;
  /** Explicit physical quantities accepted by this input. VSH and VCL both use v/v, so neither
   *  the unit nor the selected mnemonic is allowed to supply this identity. */
  accepted_shale_clay_quantities?: Array<"VSH" | "VCL">;
  /** Producer-owned physical identity recorded for this output after output renaming. */
  output_shale_clay_quantity?: "VSH" | "VCL" | null;
  /** SB-POR-001 common family envelope plus the producing method's own limit/convention policy. */
  porosity_output?: PorosityOutputContract | null;
  default: string;
  /** Ordered automatic LogIn aliases; absent means the single manifest default is used. */
  preferred_aliases?: string[];
  /** Source-bearing advice shown beside the field. It never populates the argument value. */
  guidance?: Array<{ text: string; source: string }>;
  /** Named source for a numeric default, or exact `ABSENT` when no numeric default ships. */
  default_source: string;
  /** Source-unit value and the named generated-registry conversion that produced a numeric
   * default. Absent parameters have no custody object; explicit values receive run custody. */
  default_unit_custody?: {
    artefact_value: number;
    artefact_unit: string;
    canonical_value: number;
    canonical_unit: string;
    conversion: {
      identity: string;
      from_unit: string;
      to_unit: string;
      factor: number;
      offset: number;
      derivation: string;
    };
  } | null;
  /** Option ids. **Stored in `params_json` on every saved run — never render-and-submit anything
   *  else as the value.** */
  choices: string[];
  /** Display text parallel to `choices`; empty or short means "show the id". Exists because a
   *  dropdown of bare ids (`LARINOV1`, `LARINOV2`, …) carries no rock age and no coefficient, and
   *  picking the wrong Larionov returns a shale volume more than half again too high right where
   *  the VSH cutoff decides net pay (`docs/review_triage.md` finding 21). */
  choice_labels?: string[];
  /** `SB-CORE-013`: this parameter is one the corpus records COMPETING shipped values for, so the
   *  editor shows them with their sources at the point of choice. A topic key rather than an
   *  embedded list, because the values belong to the topic — electrofacies, GMM facies and the ML
   *  dialog must not be able to show three different answers for the same number. Empty for almost
   *  every arg; fetch with `paramSources`. */
  sources_topic?: string;
  /** Source-bearing preconditions evaluated by the public module runner before computation. */
  validity_conditions?: ValidityCondition[];
  min: number | null;
  max: number | null;
  required: boolean;
  /** Other declared LogIn argument names that may satisfy this required input role. */
  required_any_of?: string[];
}

/** One product's shipped or advised value for a parameter (`SB-CORE-013`). */
export interface ParamSource {
  /** The product that ships or advises it — including SandiBumi, which is never listed first. */
  product: string;
  /** The value as its source states it. A STRING because "15-20" and "none stated" are both real
   *  answers and neither is a number. */
  value: string;
  /** What the value is FOR, where the source distinguishes stages or modules. */
  note: string;
  /** Where the claim was read, including this repository for SandiBumi's own position. */
  source: string;
  /** Evidence tier exactly as refined by the owning PRD chapter (for example T1a or T3). */
  tier: string;
}

/** The competing shipped values recorded for a parameter topic, or empty where there are none.
 *
 *  Three packages routinely ship three different values for one constant and none of them tells the
 *  interpreter the others exist — none of them can, because no vendor can credibly publish a
 *  competitor's defaults. Showing the disagreement is the point. */
export function paramSources(topic: string): Promise<ParamSource[]> {
  return invoke<ParamSource[]>("param_sources", { topic });
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

export interface ParameterSchemaEntry {
  semantic_id: string;
  ordinal: number;
}

export interface ParameterModuleSchema {
  module_schema_version: string;
  parameters: ParameterSchemaEntry[];
}

export interface ParameterPackRow {
  semantic_id: string;
  module_schema_version: string;
  ordinal: number;
  display_label: string;
  value: unknown;
}

export interface ParameterPack {
  source_file: string;
  text_provenance: {
    declared_encoding: string | null;
    decoded_encoding: string;
    /** Reversible original-byte representation; never a JSON numeric array. */
    original_bytes_hex: string;
  };
  rows: ParameterPackRow[];
}

/** Read the backend-owned semantic identifiers, ordinals, and exact manifest version. */
export function getParameterModuleSchema(moduleName: string): Promise<ParameterModuleSchema> {
  return invoke<ParameterModuleSchema>("get_parameter_module_schema", { moduleName });
}

/** Load and validate a pack against the selected shipping module's backend-owned schema. */
export function loadParameterPack(moduleName: string, path: string): Promise<ParameterPack> {
  return invoke<ParameterPack>("load_parameter_pack", { moduleName, path });
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
  custody: RunCustody;
}

export interface ModuleInputAvailability {
  well_id: string;
  /** Manifest LogIn argument names with at least one finite sample on the runner's resolved frame. */
  available_arguments: string[];
  /** A read failure is distinct from an ordinarily absent required input. */
  error: string | null;
}

/** Read-only preflight for a module pane. Numeric arrays remain in Rust; only availability
 * metadata crosses IPC so this cannot become a second JSON curve-data path. */
export function moduleInputAvailability(
  moduleName: string,
  scope: BackendWellScope,
  logInputs: Record<string, string>,
  inputSet?: string,
): Promise<ModuleInputAvailability[]> {
  return invoke<ModuleInputAvailability[]>("module_input_availability", {
    module: moduleName,
    scope,
    logInputs,
    inputSet,
  });
}

export type DespikeEstimator = "TRUE_MAD" | "MEAN_DEVIATION_FALLBACK" | "MEAN_SIGMA_POPULATION";

export interface DespikeContaminationBranch {
  estimator: DespikeEstimator;
  ceiling_pct: number;
  sample_count: number;
}

export interface DespikeContaminationPreview {
  branches: DespikeContaminationBranch[];
  evaluated_wells: number;
  unavailable_well_ids: string[];
  issues: Array<{ well_id: string; error: string }>;
}

/** Read-only SB-ENV-031 preflight. The selected curve arrays stay in Rust; only estimator names,
 * mathematical ceilings and counts cross IPC. */
export function despikeContaminationPreview(
  scope: BackendWellScope,
  logInputs: Record<string, string>,
  params: Record<string, number>,
  opts: Record<string, string>,
  inputSet?: string,
): Promise<DespikeContaminationPreview> {
  return invoke<DespikeContaminationPreview>("despike_contamination_preview", {
    scope,
    logInputs,
    params,
    opts,
    inputSet,
  });
}

export type AncestryActorKind = "HUMAN" | "AUTOMATED";

export interface RunCustody {
  actor: {
    kind: AncestryActorKind;
    identity: string;
  };
  source_note: string;
}

export interface AncestryInput {
  well_id: string;
  argument: string;
  curve: string;
  log_set: string;
  set_version: number | null;
  set_id: string;
  /** Exact stored curve identity (native imported UUID or computed set/name composite);
   *  absent only on readable schema-v1 history. */
  chosen_curve_id?: string;
  /** Controlled resolution stage that selected the stored identity. */
  rule?:
    | "EXPLICIT_NAME"
    | "WORKING_INPUT_SET"
    | "ALIAS_OFF"
    | "ALIAS_MANUAL"
    | "ALIAS_AUTOMATIC"
    | "FINAL_FLAG"
    | "CURVE_TYPE_MRU";
  /** Candidates considered by the same resolver and not selected. */
  rejected_candidates?: Array<{
    curve_id: string;
    log_set: string;
    set_version: number | null;
  }>;
}

export interface AncestryParameter {
  name: string;
  value: unknown;
  source: string | null;
  /** Present only when no numeric value exists; a sourced value has no absent-state token. */
  state?: "REQUIRED_UNSET";
  /** Whether the effective value came from this run request or the module manifest. */
  resolution?: "EXPLICIT" | "DEFAULTED";
  /** Exact module-manifest identity for a DEFAULTED value; absent for explicit and legacy rows. */
  manifest_version?: string;
  decision?: {
    topic: string;
    parameter: string;
    alternatives: Array<{
      product: string;
      value: string;
      note: string;
      source: string;
      tier: string;
    }>;
    selected_matches: string[];
  };
}

export type AncestryZoneScope =
  | { kind: "WHOLE_WELL" }
  | { kind: "DEFINED"; definitions: Array<{ name: string; top: number; base: number; source: string }> };

export interface CurveAncestry {
  schema_version: number;
  module: string;
  module_version: string;
  inputs: AncestryInput[];
  parameters: AncestryParameter[];
  zone_scope: AncestryZoneScope;
  actor: { kind: AncestryActorKind; identity: string };
  timestamp_utc_ms: number;
  outputs: Array<{ curve: string; derivation: string }>;
}

export interface CurveAncestryDisclosure {
  well_id: string;
  curve_name: string;
  provenance_class: "RECORDED" | "LEGACY_UNRECORDED";
  provenance_row_count: number;
  set_name: string | null;
  version: number | null;
  ancestry: CurveAncestry | null;
}

export interface ModuleRunResult {
  well_id: string;
  rows_written: number;
  output_curves: string[];
  error: string | null;
  outcome: "clean" | "degraded" | "failed" | "skipped";
  degradations: RunDegradation[];
}

export interface RunDegradation {
  kind: "CLAMPED" | "DEFAULTED" | "TRUNCATED" | "SUBSTITUTED_INPUT" | "ENDPOINT_INVALID";
  detail: string;
  occurrences: number;
}

/** One declared output and the curve name a run with the current settings would write it under. */
export interface OutputName {
  arg: string;
  desc: string;
  unit: string;
  name: string;
  flag_kind?: FlagKind | null;
}

/**
 * The names a module would write, asked of the backend rather than worked out here.
 *
 * A module's default output name can be built from the run's own choices (`{CURVE}_C`,
 * `{TARGET}_SYN`), and expanding those patterns in TypeScript would be a second copy of a naming
 * rule — the preview would agree with the run right up until somebody changed one of them.
 */
export async function moduleOutputNames(
  module: string,
  logInputs: Record<string, string>,
  opts: Record<string, string>,
): Promise<OutputName[]> {
  return invoke("module_output_names", { module, logInputs, opts });
}

export async function runWorkflowModule(
  req: RunModuleRequest,
  scope: BackendWellScope,
): Promise<ModuleRunResult[]> {
  return invoke<ModuleRunResult[]>("run_workflow_module", { req, scope });
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
  scope: BackendWellScope,
  custody: RunCustody,
  outputSet?: string,
  inputSet?: string,
): Promise<void> {
  return invoke<void>("run_workflow_chain", {
    jobId,
    steps,
    scope,
    custody,
    outputSet: outputSet ?? null,
    inputSet: inputSet ?? null,
  });
}

// --- P1-c log-set versioning (never overwrite) ------------------------------

export interface LogSetRestoreRecord {
  schema_version: number;
  source_set_id: string;
  source_version: number;
}

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
  /** Complete SB-CORE-010 record; null only for a pre-contract legacy project. */
  ancestry: CurveAncestry | null;
  /** Present only when this run appended a prior immutable version back as a new version. */
  restored_from: LogSetRestoreRecord | null;
  /** Null only for a pre-contract run that cannot honestly be classified after the fact. */
  outcome_state: "CLEAN" | "DEGRADED" | null;
  degradations: Array<RunDegradation & { module: string }>;
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

export interface RestoreLogSetResult {
  rows_restored: number;
  new_set_id: string;
  new_version: number;
  restored_from: LogSetRestoreRecord;
}

/** Restores an archived version as a new append-only version and returns its source link. */
export function restoreLogSet(setId: string): Promise<RestoreLogSetResult> {
  return invoke<RestoreLogSetResult>("restore_log_set", { setId });
}

/** Current computed curves of a well with provenance + basic statistics. */
export interface ComputedCatalogEntry {
  curve_name: string;
  provenance_class: "RECORDED" | "LEGACY_UNRECORDED";
  provenance_row_count: number;
  set_name: string | null;
  version: number | null;
  module: string | null;
  created_at: string | null;
  n_samples: number;
  min: number | null;
  max: number | null;
  mean: number | null;
  /** Complete SB-CORE-010 record exposed on demand in the catalog. */
  ancestry: CurveAncestry | null;
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
  /** Aggregate result is separate from lifecycle: completed work may still be degraded. */
  outcome: "clean" | "degraded" | "failed" | null;
  total: number;
  done: number;
  current: string | null;
  items: JobItem[];
  error: string | null;
  seq: number;
  /** Whether this job's worker actually polls the cancel flag. The panel shows a Cancel button
   *  only when true; a monolithic op (a render, an export, a single subprocess) reports false and
   *  gets an honest "can't be interrupted" tag instead of a button that would do nothing. */
  cancellable: boolean;
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
  /** SB-CUT-001 (DEC-071): "CENTRED" (default) or "TOPS" (the legacy forward rule). */
  discretisation?: "CENTRED" | "TOPS";
  well_ids: string[];
  steps: ChainStep[];
  mc_params: McParam[];
  iterations: number;
  seed: number;
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min: CutoffSpec | null;
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
  /** Also store the per-sample REALIZATION MATRIX as MC_<KEY>_REAL in `array_logs`, so a log
   *  view can draw an adjustable band / spaghetti / heat map from one stored run. Requires
   *  `persist`; off by default because it is the only output whose size scales with iterations. */
  persist_realizations?: boolean;
  /** How many realizations to store per depth (default 256, clamped 8..1024). */
  realization_cap?: number;
  /** Required only when `persist` writes computed curves. */
  custody?: RunCustody | null;
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

/** Per-well physical-plausibility diagnostic: how often a sampled combo produced an impossible
 *  Sw>1 / PHIE<0 (on the chain's unlimited porosity/saturation curves). Reported, not excluded —
 *  the module limits clamp these to the correct volumetrics, so they stay valid P10/P90 tails. */
export interface McPlausibility {
  well_id: string;
  well_name: string;
  impossible_realizations: number;
  realizations: number;
  fraction: number;
  /** False when the well produced no finite porosity/saturation samples to judge. */
  checked: boolean;
  detail: string;
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
  /** Per-well physical-plausibility (impossible Sw>1 / PHIE<0 fraction; reported, not excluded). */
  plausibility: McPlausibility[];
  /** Non-fatal advisories (skipped correlation pairs, degenerate targets, …). */
  notes: string[];
  errors: string[];
}

/** Runs a Monte Carlo study; resolves with the per-zone P10/P50/P90 + HPV histograms. Runs
 *  in memory on the backend (no computed_curves writes), so 1000+ realizations are fast. */
export async function runMonteCarlo(req: McRequest, scope: BackendWellScope): Promise<McResult> {
  return invoke<McResult>("run_monte_carlo", { req, scope });
}

// --- Machine learning (Phase 10-4) ---

export interface MlRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  /** Version the outputs into this log set; omit for the tool's own default. */
  output_set?: string;
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
  /** Keep the fitted model under this name so it can be applied to other wells later. Supervised
   *  tasks only — clustering and reduction are fitted on the wells they are applied to. */
  save_model_as?: string | null;
  model_note?: string | null;
  /** Hold roughly this fraction of the pooled training SAMPLES back from the fit and score the
   *  model on them. A share of the data, held back as WHOLE WELLS: splitting pooled samples would
   *  put consecutive depths from one well on both sides and the blind score would come back
   *  optimistic, so the fraction is a target the well subset is chosen to approach rather than a
   *  quantity it can hit exactly. Omit for no split (every training well is fitted on, exactly as
   *  before). */
  blind_fraction?: number | null;
  /** Seed for the draw, so the same request re-runs to the same split. */
  split_seed?: number | null;
  /** `"well"` (default) holds back whole wells — the only split that cannot leak, and the one that
   *  answers "will this work on the next well?". `"sample"` draws individual rows stratified on the
   *  target — exact in its percentage, balanced in its statistics, and optimistic on log data
   *  because consecutive depths are near-duplicates. */
  split_mode?: string | null;
  /** Fit the target in a transformed space. `"log10"` is the one transform on offer, for
   *  permeability and anything else spanning decades. A transformed quantity is a DIFFERENT
   *  quantity, so the run writes two curves: `<output_curve>_LOG10` (the model's own prediction, in
   *  log units) and `<output_curve>` (its back-transform, in the target's units). Every reported
   *  score is in the fitted space. Regression only. Omit or null to fit the target as measured. */
  target_transform?: string | null;
  /** Fit one model per pattern of available inputs instead of one model over the depths where every
   *  input exists.
   *
   *  Off, a curve logged over half the interval deletes the other half of every other input too,
   *  because a row reaches the fit only where all of them have a value. On, each depth is predicted
   *  by the largest model whose curves it carries — so the short curve is used where it exists and
   *  the rest of the rock is predicted without it. Each segment keeps its OWN blind score and its own
   *  saved model (suffixed `_<n>CURVE`), because they are different models on different feature sets
   *  and one number over both would describe neither. Supervised only. */
  coverage_segments?: boolean;
  /** Write the prediction at this vertical resolution: one value per `output_step`-thick interval,
   *  held across the interval, on the well's own depths.
   *
   *  A model fitted against a target sampled every 0.5 m predicts at every INPUT depth, so it emits
   *  a value every 0.1524 m — a curve claiming three times the resolution anything it learned from
   *  ever had. The depth FRAME is unchanged (computed curves are read back by exact depth match, so
   *  writing at a coarser sampling would make the curve read back all-missing); only the values are
   *  held in blocks, which is why such a curve wants `draw_style: "step"` in the layout.
   *
   *  Omit or null to write at the input frame, which is what every run did before this existed. A
   *  saved model records it, and applying that model inherits it. */
  output_step?: number | null;
  /** Confine BOTH the fit and the prediction to this depth window (Jauhar: *"it should be tops
   *  bounded as well by user"*). A model fitted over a whole well learns one relation for every
   *  formation it passed through. Omit for the whole logged interval. */
  interval?: DepthWindow;
  /** What the feature space is normalized AGAINST, when `standardize` is on.
   *
   *  Omit or null for the data-derived basis every run used before this existed: mean and spread
   *  computed from the samples in hand. `"limits"` uses the fixed per-curve ranges in `norm_limits`.
   *
   *  **This is the add-a-well trap.** On a data-derived basis, adding one well recomputes every mean
   *  and scale, which moves every boundary expressed in them — in the wells that were *already
   *  there*. Nothing reports that anything changed and both answers look reasonable. A basis fixed
   *  to limits the analyst chose is stable across wells. */
  norm_basis?: string | null;
  /** Fixed per-curve limits for `norm_basis: "limits"`. Matched to features by name and re-ordered
   *  into the resolved feature order by the backend, so a caller can no more reorder a basis than it
   *  can reorder the features. Never filled in for you: a run whose inputs are not all covered is
   *  refused, because the same curve over two ranges gives two answers and both look right. */
  norm_limits?: CurveLimit[];
  /** Per-input transforms applied before the fit and **stored inside the saved model**, so applying
   *  it later cannot use a different one. Matched to features by name and re-ordered into the
   *  resolved feature order by the backend. Send only the curves actually changed — an omitted
   *  curve means "as measured", which keeps every payload written before this existed identical. */
  feature_transforms?: CurveTransform[];
  custody: RunCustody;
}

/** One input's transform. `transform` is `none` | `log10` | `ln` | `sqrt`.
 *
 *  A short list on purpose: each is one a petrophysicist already applies by hand to these curves,
 *  and an arbitrary expression could not be re-applied from a saved artifact with any confidence. */
export interface CurveTransform {
  curve: string;
  transform: string;
}

/** One curve's fixed normalization range. */
export interface CurveLimit {
  curve: string;
  low: number;
  high: number;
}

/** One feature subset a coverage-segmented run fitted a model for, or declined to. Reported per
 *  segment and never averaged: the curve is one curve, and how well it is known varies down it. */
export interface CoverageSegment {
  features: string[];
  /** Depths this segment predicted. 0 where it was skipped. */
  n_predicted: number;
  n_train: number;
  /** This segment's own blind record, carrying `performed: false` where nothing was held back. */
  blind: Record<string, unknown> | null;
  /** The saved artifact's name, where the run was asked to save one. */
  model_name: string | null;
  /** Why no model was fitted, stated in full. Null on a segment that ran. */
  skipped: string | null;
}

/** The split as it was actually performed, not as it was requested — the requested share is kept
 *  beside the achieved one because whole wells rarely divide the samples exactly, and the gap is
 *  what the blind score is really a score of. */
export interface SplitReport {
  /** Empty in `sample` mode — every well is on both sides, so naming them would say nothing. */
  fit_wells: string[];
  blind_wells: string[];
  /** Usable training samples on each side — what the fraction is really a fraction of. */
  fit_rows: number;
  blind_rows: number;
  requested_fraction: number;
  /** `blind_rows / (fit_rows + blind_rows)`. Whole wells rarely divide the data exactly, so
   *  this is what was actually reached — never a restatement of the request. */
  achieved_fraction: number;
  seed: number;
  /** `"well"` or `"sample"` — the two are different claims, and a score quoted without it cannot
   *  be read. */
  mode: string;
  /** How many wells contributed rows. The answer to "how much rock is this?" in `sample` mode,
   *  where the well lists are empty. */
  wells_pooled: number;
}

/** How alike the fit and blind sides are, per feature and on the target — the evidence for a
 *  stratified draw's "similar statistics" claim. Reported rather than asserted: a pair that does
 *  NOT match is the signal that the strata were too thin to divide representatively. */
/** One side of a split, as a whole distribution rather than a centre and a width. */
export interface BalanceShape {
  n: number;
  mean: number;
  sd: number;
  p10: number;
  p50: number;
  p90: number;
  /** Midpoint of the tallest bin of a 64-bin histogram over BOTH sides' combined range — the two
   *  sides share one binning, because modes read off different binnings are not a comparison. */
  mode: number;
  /** Fisher-Pearson g1, the population form — the same number `scipy.stats.skew` returns. */
  skew: number;
}

export interface SplitBalance {
  name: string;
  fit_mean: number;
  blind_mean: number;
  fit_sd: number;
  blind_sd: number;
  /** The whole shape of each side. Two sets can agree exactly on mean and sd and still be a
   *  unimodal clean sand against a bimodal sand-shale pair, so the four flat fields above cannot
   *  answer whether a draw was representative. Optional so a metrics blob written before this
   *  existed still renders. */
  fit?: BalanceShape;
  blind?: BalanceShape;
  /** Centre shift in FIT standard deviations — the one form comparable across curves in different
   *  units. Signed: which way the blind side sits matters. */
  mean_shift_sd?: number;
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
  /** Advisories that qualify a successful run — e.g. training wells that contributed no usable
   *  samples, so the model was fit on fewer wells than were selected. Empty on a clean run. */
  notes: string[];
  /** Set when the fit was kept as a reusable artifact. */
  model_id: string | null;
  /** The name it was actually stored under — an existing name is auto-suffixed, never
   *  overwritten, so this can differ from what was asked for. */
  model_name: string | null;
  /** Which wells were fitted on and which were held blind; null when no split was asked for. */
  split: SplitReport | null;
  error: string | null;
}

/** Runs a scikit-learn model (subprocess): supervised tasks fit on the train wells'
 *  labelled samples and predict on the apply wells; unsupervised tasks fit on the pooled
 *  apply samples (field-wide, globally consistent cluster ids). */
export function runMl(req: MlRequest, scope: BackendWellScope): Promise<MlResult> {
  return invoke<MlResult>("run_ml", { req, scope });
}

/** A saved, re-runnable model. `feature_curves` is ORDERED — the order is part of the apply
 *  contract, not a display detail. */
export interface MlModelInfo {
  model_id: string;
  name: string;
  task: string;
  algorithm: string;
  feature_curves: string[];
  target_curve: string | null;
  params_json: string;
  metrics_json: string;
  /** Names of the wells that actually contributed samples. */
  trained_on: string[];
  n_train: number;
  standardize: boolean;
  sklearn_version: string | null;
  note: string | null;
  created_at: string;
  bytes: number;
  /** Fingerprint of the exact training matrix — feature names in order, feature and target values,
   *  row order. The well list and the sample count cannot tell "the same wells at a later log-set
   *  version" from "the same rows"; this can. Null on a model saved before it was recorded, which
   *  is the honest answer for such a model rather than a hash that means nothing. */
  train_hash: string | null;
  /** SB-MLA-002 + SB-MLA-004 — JSON array of `TrainWellRecord`: per contributing well, the rows it
   *  gave, the rows the mask removed, the rows that were incomplete, and the log set (name, id,
   *  version) its frame was read from. `trained_on` says which wells; this says which rock. */
  training_json: string | null;
  /** SB-MLA-005 — JSON object of the interpreter and library versions that fitted and serialised
   *  this artifact. The blob is a pickle, so it is loadable only under a compatible set, and
   *  `joblib` — the serialiser itself — is in here for that reason. */
  runtime_json: string | null;
}

/** A depth window a run is confined to.
 *
 *  Each side is independent and an open side stays open: no base means run to TD, exactly as the
 *  last top in a well does. Omit the whole object for the full logged interval. */
export interface DepthWindow {
  top?: number | null;
  base?: number | null;
}

export interface MlApplyRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  /** Version the outputs into this log set; omit for the tool's own default. */
  output_set?: string;
  model_id: string;
  apply_well_ids: string[];
  output_curve: string;
  mask_curve?: string | null;
  /** Confine the prediction to this depth window. NOT inherited from the model: where a model
   *  LEARNED and where you choose to propagate it are separate decisions. */
  interval?: DepthWindow;
  custody: RunCustody;
}

export function listMlModels(): Promise<MlModelInfo[]> {
  return invoke<MlModelInfo[]>("list_ml_models");
}

/** SB-MLA-002 + SB-MLA-005 — what each saved model would be warned about IF applied now.
 *
 *  Both requirements say the warning must come BEFORE the model is applied, and an apply run cannot
 *  give either early: it learns the runtime from a reply header that arrives after the prediction,
 *  and by then the curves are written. So the picker asks first.
 *
 *  The comparison itself lives in Rust (`ml::model_warnings`) rather than here, so the list and the
 *  run result cannot word the same problem two different ways. Only models with something to say are
 *  returned. */
export interface ModelWarnings {
  model_id: string;
  notes: string[];
}

export function mlModelWarnings(): Promise<ModelWarnings[]> {
  return invoke<ModelWarnings[]>("ml_model_warnings");
}

/** SB-MLA-008 — what about a chosen task/algorithm would not reproduce on another machine, asked
 *  BEFORE the run. `null` is the ordinary answer and means the run reproduces from its own record
 *  under the same runtime.
 *
 *  Deliberately scoped to what the product can observe in its own code — today the `gbdt` estimator
 *  substitution — rather than to second-hand claims about which library is deterministic where. */
export function mlDeterminismNote(task: string, algorithm: string): Promise<string | null> {
  return invoke<string | null>("ml_determinism_note", { task, algorithm });
}

/** What one curve's OWN sampling looks like, and how much of it survives the join onto the well's
 *  frame. `n_own` large beside `n_on_frame` zero is the finding: the curve is fully logged and
 *  coincides with the frame at no depth, so every read of it returns blank. */
export interface CurveSampling {
  curve: string;
  n_own: number;
  /** Median spacing between the curve's own depths — median, not mean, so one gap across a casing
   *  shoe cannot drag it away from the sampling the tool actually ran at. */
  step: number | null;
  top: number | null;
  base: number | null;
  n_on_frame: number;
  imported: boolean;
}

/** Per well, how each named curve is sampled against that well's frame. */
export function curveSampling(
  scope: BackendWellScope,
  curves: string[],
): Promise<[string, CurveSampling[]][]> {
  return invoke<[string, CurveSampling[]][]>("curve_sampling", { scope, curves });
}

/** Applies a saved model to wells it has never seen. Nothing is refitted — a refit on different
 *  data is a different model. The model's own curve list drives the inputs, so the caller cannot
 *  reorder them. */
export function applyMlModel(req: MlApplyRequest, scope: BackendWellScope): Promise<MlResult> {
  return invoke<MlResult>("apply_ml_model", { req, scope });
}

export function renameMlModel(modelId: string, newName: string): Promise<string> {
  return invoke<string>("rename_ml_model", { modelId, newName });
}

/** One live curve set that names a saved model as what produced it (SB-MLA-007). */
export interface ModelCitation {
  well_name: string;
  set_name: string;
  curves: string[];
}

/** Which delivered curves would be orphaned by deleting this model. */
export function mlModelCitations(modelId: string): Promise<ModelCitation[]> {
  return invoke<ModelCitation[]>("ml_model_citations", { modelId });
}

/** Deletes a saved model. REFUSES when a live curve cites it, naming what would be orphaned;
 *  `force` is the user's own decision taken after reading that list. */
export function deleteMlModel(modelId: string, force = false): Promise<void> {
  return invoke<void>("delete_ml_model", { modelId, force });
}

/** Model-comparison leaderboard (Wave B item 3). */
export interface MlEvalRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  task: "regression" | "classification";
  feature_curves: string[];
  target_curve: string;
  train_well_ids: string[];
  algorithms: string[];
  /** The same hyperparameters the run will be given, so a row describes the model you will fit. */
  params?: Record<string, number | string | boolean>;
  /** Which algorithm `params` belongs to. Every other row is scored at its defaults, which is what
   *  the run would fit for them — an `C` set for SVR must not silently re-rank logistic regression. */
  params_for?: string | null;
  /** Feature subsets to try (each a subset of feature_curves); empty → full set only. */
  subsets: string[][];
  /** Score EVERY non-empty combination of the curves instead of `subsets`. Answers "which curves
   *  do I need" rather than "which model is best" — overrides `subsets` when set. */
  enumerate_subsets?: boolean;
  standardize: boolean;
  seed: number;
  folds: number;
  /** Optional flag curve: masked (= 1) samples are excluded from the CV pool so the leaderboard
   *  scores the same population the real run trains on. Omit / null for no masking. */
  mask_curve?: string | null;
  /** Score the candidates in a transformed target space — the same value the run will be given.
   *  A model fitted on log10(k) is a different model from one fitted on k, and in linear space an
   *  R² over four decades of permeability is dominated by the few highest values, so the winner
   *  there is routinely not the winner in log space. Regression only. */
  target_transform?: string | null;
}

export interface MlEvalRow {
  algorithm: string;
  features: string[];
  /** The MEAN of the per-fold scores — R² (regression) or accuracy (classification), each fold
   *  scored against its own held-out well. Null if the row errored.
   *
   *  This is the same estimator a training run reports as `r2_cv`, so a model does not change value
   *  between the table it was chosen FROM and the run it was chosen FOR — and it is the number
   *  `score_std` is the spread of. The table used to show `score_pooled` here with this one's
   *  spread beside it, so a model reading "0.327 ± 0.094" scored 0.216 on a typical well. */
  score: number | null;
  /** Spread of the per-fold scores. Qualifies `score` and nothing else. */
  score_std: number | null;
  /** One score over every out-of-fold row at once, against the GLOBAL mean.
   *
   *  Answers "how good is the field-wide curve", which is a real question but not the one a
   *  leaderboard is read for. Runs HIGHER than `score` whenever the wells differ in level, because
   *  between-well contrast counts as variance the model is credited with explaining. */
  score_pooled: number | null;
  /** Folds `score` and `score_std` are computed over. Below `n_splits`, some fold produced no
   *  score and the mean is over fewer wells than it looks. */
  n_score_folds: number;
  /** Folds where the optimiser hit its iteration limit instead of converging. A candidate that
   *  gave up is not one that merely lost, and the score cannot tell them apart. */
  n_unconverged: number;
  /** scikit-learn's own words for the last such warning, so the fix it suggests is not lost. */
  converge_note: string;
  metrics: Record<string, unknown>;
  /**
   * Permutation importance measured on each fold's HELD-OUT rows, then averaged — the same
   * population `score` is measured on, so the two can be read in one row.
   */
  importances: number[];
  /**
   * Spread of `importances` across folds. A feature that carried in one well and nowhere else has
   * a large one, and is not a predictor however high its mean.
   */
  importances_std: number[];
  /** Folds that contributed an importance. Below `n_splits`, some fold could not be permuted. */
  n_imp_folds: number;
  confusion: number[][] | null;
  labels: number[] | null;
  /** This model's OUT-OF-FOLD prediction per sampled row, aligned with `MlEvalResult.blind_actual`.
   *
   *  Out-of-fold, so every point was predicted by a model that had not seen that row — the crossplot
   *  answers the same question the score does. A scatter of fitted values would look better and mean
   *  nothing. `null` where no fold could predict that row; never 0, which is a value. */
  blind_pred: (number | null)[];
  error: string | null;
}

export interface MlEvalResult {
  rows: MlEvalRow[];
  n_train: number;
  n_groups: number;
  cv: string;
  n_splits: number;
  /** What the two score columns each are, in one sentence, carried once because it is the same for
   *  every row. Shipped from `ml.rs` so the table cannot word them differently from the code that
   *  computed them. */
  score_protocol: string;
  note: string | null;
  /** Which row was scored with the settings on screen; every other row is at library defaults. */
  params_for: string | null;
  /** The measured value at each sampled row — the x-axis every model's crossplot shares. Carried
   *  once rather than per row: it is the same column for all of them. */
  blind_actual: (number | null)[];
  /** Which well each sampled row came from, by name. A crossplot coloured by well shows a model
   *  carried by two wells and failing on the third, which the aggregate R² above it cannot. */
  blind_well: string[];
  /** Points drawn, and points there were. The sample is capped, and a scatter that silently showed
   *  2,000 of 60,000 would read as all of them — density is the first thing anybody judges. */
  blind_sampled: number;
  blind_total: number;
  /** What each input curve is worth, measured by dropping it. Empty unless at least two curve
   *  combinations were scored, because with one there is nothing to compare. */
  curve_value?: CurveValue[];
  error: string | null;
}

/** What one input curve is WORTH, measured by dropping it — the question that decides whether the
 *  next well runs a tool. A curve can be in the winning model and still be worth nothing. */
export interface CurveValue {
  curve: string;
  /** Best blind score among scored combinations that INCLUDE this curve. */
  best_with: number | null;
  /** Best among those that EXCLUDE it. `null` where every scored combination carried it — the
   *  question was never asked, which is not the same as the answer being zero. */
  best_without: number | null;
  /** `best_with − best_without`: what having this curve buys, in the score's own units. */
  gain: number | null;
  /** Whether the single best-scoring combination overall uses it. */
  in_best: boolean;
}

/** Ranks algorithm × feature-subset combos by blind-well (GroupKFold) CV, with permutation
 *  importance + confusion matrix. Evaluation only — writes no curves. */
export function runMlEval(req: MlEvalRequest, scope: BackendWellScope): Promise<MlEvalResult> {
  return invoke<MlEvalResult>("run_ml_eval", { req, scope });
}

/** Cuddy FOIL / BVW saturation-height fit (Wave B item 8, SHF side). */
export interface CuddyFoilRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
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
export function runCuddyFoil(req: CuddyFoilRequest, scope: BackendWellScope): Promise<CuddyFoilResult> {
  return invoke<CuddyFoilResult>("run_cuddy_foil", { req, scope });
}

/** Height-domain SHF fit (Brooks-Corey / Skelt-Harrison) to the log-derived Sw-vs-height cloud. */
export interface ShfFitRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
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

export function runShfFit(req: ShfFitRequest, scope: BackendWellScope): Promise<ShfFitResult> {
  return invoke<ShfFitResult>("run_shf_fit", { req, scope });
}

/** Electrofacies tie-in QC: confusion matrix of a predicted log RT curve vs a reference/core RT. */
export interface FaciesConfusionRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  well_ids: string[];
  pred_curve: string;
  ref_curve: string;
  /** Dominant-class purity (0..1) at or above which the mapping is accepted. **No default** — omit
   *  it and the result is reported unjudged (SB-MLA-052). The method note states a threshold is
   *  required and states no value; neither does any source the app holds. */
  accept_threshold?: number;
}

export interface RefClassRow {
  ref_label: number;
  dominant_pred: number;
  /** ROW-normalised — of this reference class's samples, the fraction in `dominant_pred`. */
  purity: number;
  count: number;
}

/** The column-wise counterpart of `RefClassRow` — the "recognition rate" axis. */
export interface PredClassRow {
  pred_label: number;
  dominant_ref: number;
  /** COLUMN-normalised — of the samples called `pred_label`, the fraction really `dominant_ref`. */
  recognition: number;
  count: number;
}

export interface FaciesConfusionResult {
  ref_labels: number[];
  pred_labels: number[];
  /** matrix[i][j] = count where reference == ref_labels[i] and prediction == pred_labels[j]. */
  matrix: number[][];
  /** `matrix` over its ROW sums, as fractions 0..1. Read it with `row_axis`, never bare. */
  row_pct: number[][];
  /** `matrix` over its COLUMN sums, as fractions 0..1. Read it with `col_axis`, never bare. */
  col_pct: number[][];
  /** Prose statement of what `row_pct` and `per_ref[].purity` divide by (SB-MLA-051). */
  row_axis: string;
  /** Prose statement of what `col_pct` and `per_pred[].recognition` divide by. */
  col_axis: string;
  per_ref: RefClassRow[];
  per_pred: PredClassRow[];
  /** ROW-normalised: Σ dominant-cell counts / total pairs. */
  overall_purity: number;
  /** The threshold the USER stated, echoed back, or `null` when they stated none. */
  accept_threshold: number | null;
  /** `null` when no threshold was stated — a mapping is never judged against a number the app
   *  chose for itself (SB-MLA-052). */
  accepted: boolean | null;
  /** Why there is no verdict, when there is none. */
  accept_note: string | null;
  n: number;
  /** ANOVA variance reduction of log10(core k) grouped by the predicted class (1 − SS_within/
   *  SS_total): 1 = the typing explains all core-perm variance, 0 = none. `null` when no core
   *  plugs match or fewer than 2 classes carry plugs (Rust f64::NAN → JSON null). */
  k_var_reduction: number | null;
  /** Core plugs that contributed to `k_var_reduction`. */
  n_core_plugs: number;
  /** Plugs with usable permeability that found no log sample inside the match tolerance, and so
   *  contributed to nothing. Reported so a statistic over 9 of 90 plugs cannot read as one over
   *  90 (SB-MLA-054). */
  n_core_unmatched: number;
  /** How a core plug was put on the log's depth frame — the method and its tolerance, in words. */
  core_match_note: string;
  error: string | null;
}

export function runFaciesConfusion(req: FaciesConfusionRequest, scope: BackendWellScope): Promise<FaciesConfusionResult> {
  return invoke<FaciesConfusionResult>("run_facies_confusion", { req, scope });
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
  /** Indonesia shale exponent coefficient k: SIMPLE=0, FULL=1 (default), TAR_SAND=2. */
  indonesia_k?: number;
  /** simandoux_modified_slb shale exponent C. Cited default 1 and valid range 1..2. */
  simandoux_c?: number;
  /** Wet-clay (shale) total porosity φ_sh for Juhász's normalized Qv (Vsh·φ_sh/φt) and shale-point
   *  conductivity (1/(Rsh·φ_sh^m)). Only Juhász reads it. Backend default 0.10. */
  phit_sh?: number;
  /** Core-measured Waxman-Smits B override (mho·mL/(m·meq)). 0/blank ⇒ compute B(T,Rw) from the
   *  Juhász fit. Only the `waxman_smits` model reads it. Backend default 0. */
  ws_b?: number;
}

/** Saturation model for the conductivity tools. `linear_dw` (default) is the in-inversion linearised
 *  dual-water; the rest are post-solve forms (Sw from Rt + the solved volumes): `dual_water_nonlinear`
 *  and `archie_total` (total-porosity), `indonesia`, the two typed Simandoux equations,
 *  `juhasz`, and `waxman_smits` (shaly-sand). */
export type SwModel =
  | "linear_dw"
  | "dual_water_nonlinear"
  | "archie_total"
  | "indonesia"
  | "simandoux_bardon_pied"
  | "simandoux_modified_slb"
  | "juhasz"
  | "waxman_smits";

export interface SwModelChoice {
  id: SwModel;
  label: string;
  /** Stable categorical value written to SW_METHOD; resolve it through this catalog, never as a quantity. */
  flag_code: number;
}

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
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  /** Version the outputs into this log set; omit for the tool's own default. */
  output_set?: string;
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
  custody: RunCustody;
}

/** Agreement between a solved output and a routine-core-analysis measurement, over the plugs that
 *  tied to a solved sample. `bias` is the mean signed (model − core), so its sign says which way
 *  the model reads. Absent (null) when no plug matched — never a zero standing in for "no data". */
export interface MmCoreFit {
  n: number;
  rms: number;
  bias: number;
}

export interface MultiminWellResult {
  well_id: string;
  rows_solved: number;
  mean_recon: number;
  /** Core calibration. RECON says the model reproduces its own input LOGS; these say whether it
   *  reproduces an INDEPENDENT measurement. Core φ comes against both PHIE and PHIT because which
   *  one a plug should match depends on the drying protocol (oven-dried → PHIT; humidity-dried →
   *  nearer PHIE), so the analyst reads the bracket rather than being handed one interpretation. */
  core_phie: MmCoreFit | null;
  core_phit: MmCoreFit | null;
  /** Solved grain density vs core ρg — a check on the MINERAL model specifically. */
  core_gd: MmCoreFit | null;
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

/** Canonical saturation equation ids and their backend-owned display labels. */
export function multiminSwModels(): Promise<SwModelChoice[]> {
  return invoke<SwModelChoice[]>("multimin_sw_models");
}

/** Runs the generalized multi-mineral inversion; writes VOL_<component> + derived curves. */
export function runMultimin(req: MultiminRequest, scope: BackendWellScope): Promise<MultiminResult> {
  return invoke<MultiminResult>("run_multimin", { req, scope });
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

export type DepthDatum = "MD" | "TVD" | "TVDSS" | "TVDKB" | "TWT" | "OWT" | "CDEPTH";

export interface ZoneEntry {
  zone_name: string;
  top_depth: number;
  bottom_depth: number;
  depth_datum: DepthDatum;
}

export async function listZones(wellId: string): Promise<ZoneEntry[]> {
  return invoke<ZoneEntry[]>("list_zones", { wellId });
}

export async function upsertZone(
  wellId: string,
  zoneName: string,
  topDepth: number,
  bottomDepth: number,
  depthDatum: DepthDatum,
): Promise<void> {
  return invoke("upsert_zone", { wellId, zoneName, topDepth, bottomDepth, depthDatum });
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
  /** Explicit stored datum. Older in-memory callers may omit it; writes derive only from the
   *  caller's existing MD/TVDSS switch, never from the numeric value or its unit. */
  depth_datum?: DepthDatum;
  /** true → depth is TVDSS (draws flat across wells); false → measured depth. */
  is_tvdss: boolean;
  color: string | null;
  label: string | null;
  /** The named fault block or segment. null = not stated. Two compartments are not in pressure
   *  communication, so they have no reason to sit on the same contact — and pooling them into one
   *  QC fit produces a surface neither is on. */
  compartment?: string | null;
  /** The markers this contact governs, sorted. EMPTY = none stated (a field-wide datum cuts across
   *  markers). SEVERAL = stacked sands in one hydraulic unit sharing ONE contact — the case a
   *  single marker field cannot express. */
  zones?: string[];
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
    depthDatum: c.depth_datum ?? (c.is_tvdss ? "TVDSS" : "MD"),
    color: c.color,
    label: c.label,
    compartment: c.compartment ?? null,
    zones: c.zones ?? [],
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
  compartment: string | null;
  zones: string[];
  n: number;
  mean_tvdss: number;
  rms: number;
  /** [a, b, c] of z = a + b·x + c·y (dip plane), or null when the flat mean is used. */
  plane: [number, number, number] | null;
  wells: ContactWellResidual[];
  error: string | null;
}

/** One QC group: a contact type, in one compartment, governing one set of markers.
 *
 *  All three parts of the key earn their place — two stacked sands can have two contacts, several
 *  stacked sands can SHARE one, and two fault blocks have no reason to share anything. */
export interface ContactGroup {
  contact_type: string;
  compartment: string | null;
  zones: string[];
  n: number;
  /** Well-scoped contacts — the only ones the consistency check can use. */
  n_well: number;
}

/** Every (type, compartment, marker set) in the project, so the QC can check them all. */
export function contactGroups(): Promise<ContactGroup[]> {
  return invoke<ContactGroup[]>("contact_groups", {});
}

/** Check whether the wells sharing ONE contact agree on a flat TVDSS surface.
 *
 *  The compartment and the marker set are part of the GROUP, not filters you may omit: omitting
 *  them checks the contacts that state none, never "all of them". */
export function checkContactConsistency(
  contactType: string,
  compartment?: string | null,
  zones?: string[],
  flagAbs?: number,
  scope: BackendWellScope = { kind: "active_group" },
): Promise<ContactConsistency> {
  return invoke<ContactConsistency>("check_contact_consistency", {
    contactType,
    compartment: compartment ?? null,
    zones: zones ?? [],
    flagAbs,
    scope,
  });
}

/** One well/marker where the picked FWL and the FWL the arithmetic reads do not agree. */
export interface FwlCheck {
  well_id: string;
  well_name: string;
  zone_name: string;
  contact_depth: number;
  contact_is_tvdss: boolean;
  param_value: number | null;
  /** contact − parameter; NaN when the two cannot be compared. */
  difference: number;
  verdict: string;
  can_apply: boolean;
}

/** Compares each marker-tagged FWL contact against the parameter `sw_height` computes from. */
export function checkFwlAgreement(
  tolerance?: number,
  scope: BackendWellScope = { kind: "active_group" },
): Promise<FwlCheck[]> {
  return invoke<FwlCheck[]>("check_fwl_agreement", { tolerance, scope });
}

/** Copies picked FWL contacts into `zone_params`, so the arithmetic reads what the panel draws.
 *  An explicit, undoable copy — never a live read at calculation time. */
export function applyFwlToZoneParams(picks: [string, string, number][]): Promise<number> {
  return invoke<number>("apply_fwl_to_zone_params", { picks });
}

// --- Results-QC: Sw-method spread ---------------------------------------------------------------

/** Request for the per-depth Sw-method envelope. Curve names are optional — the backend tries a
 *  candidate list (first present wins) when a field is left blank. Qv/Swb curves are what pull the
 *  Waxman-Smits / Dual-Water models into the envelope; without them those two are skipped, never faked. */
export interface SwSpreadRequest {
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
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

/** SB-DBM-011: an audited surface - the backend writes the value and its structured audit
 *  entry as one gesture record, so the operator is part of the call, never inferred. */
export async function setZoneParam(
  wellId: string,
  zoneName: string,
  paramName: string,
  valueNum: number | null,
  valueText: string | null,
  operator: { identity: string; kind: AncestryActorKind },
  view?: string,
): Promise<void> {
  return invoke("set_zone_param", {
    wellId,
    zoneName,
    paramName,
    valueNum,
    valueText,
    operator: operator.identity,
    operatorKind: operator.kind,
    view: view ?? null,
  });
}

/** A whole-well override of a module parameter: a `zone_params` row whose zone is `*`.
 *  At run time it fills the whole curve before any named zone overrides it, so it sits
 *  between the workflow step's value and the per-zone values. */
export interface WellParamOverride {
  well_id: string;
  param_name: string;
  value_num: number;
}

/** Every whole-well parameter override in the project (one query, not one per well). */
export async function listWellParamOverrides(): Promise<WellParamOverride[]> {
  return invoke<WellParamOverride[]>("list_well_param_overrides");
}

/** Applies a batch of whole-well overrides atomically; a null value clears one. Returns the
 *  number of rows written or cleared. Used by the per-well grid's edits AND their undo, so a
 *  fill-column sweep and its reversal are the same single transaction shape. */
export async function setWellParamOverrides(
  entries: [string, string, number | null][],
  scope: BackendWellScope,
): Promise<number> {
  return invoke<number>("set_well_param_overrides", { entries, scope });
}

/** Applies a batch of parameter overrides at ONE zone scope in ONE transaction — `"*"` for the
 *  whole well, or a zone name. A null value clears that row rather than writing zero.
 *
 *  An accepted calibration comes through here, so every well it touches gets the whole
 *  coefficient set or none of it: a half-applied saturation calibration would leave a field with
 *  two different answers and nothing on the log to say where the boundary fell. */
export async function setZoneParamBatch(
  zoneName: string,
  entries: [string, string, number | null][],
): Promise<number> {
  return invoke<number>("set_zone_param_batch", { zoneName, entries });
}

/** SB-CUT-019. A cut-off AS ENTERED: the number and the unit it was typed in.
 *
 *  The unit is not decoration. IP's own manual expresses one quantity in porosity units in one
 *  place and `v/v` in another, with no unit tag on the field, so `35` where `0.1` is meant is a
 *  350x error whose symptom is a plausible all-net well. A bare number is REFUSED by the backend
 *  rather than guessed at. Volume fractions accept `v/v`, `pu` or `%`; permeability `mD` or `D`. */
export interface CutoffEntry {
  value: number;
  unit: string;
}

/** SB-CUT-020. Which side of a bound a sample sitting exactly ON it falls.
 *
 *  Spelled as a word rather than `>=` / `>`, because this is the one setting whose misreading is
 *  invisible: it changes the verdict only for samples exactly on the cut-off, which is precisely
 *  the population a marginal-pay result turns on. */
export type BoundOperator = "INCLUSIVE" | "EXCLUSIVE";

/** SB-CUT-020. One side of a cut-off range, as entered. `operator` defaults to `INCLUSIVE`. */
export interface CutoffSpecBound extends CutoffEntry {
  operator?: BoundOperator;
}

/** SB-CUT-020. A cut-off, which may be single-sided or a two-sided range.
 *
 *  The single-sided form is the DEGENERATE case with an open far bound, not a separate mechanism,
 *  so sending a bare `{value, unit}` keeps meaning exactly what it always meant: `>=` in a `_min`
 *  slot, `<=` in a `_max` slot, inclusive on the boundary. Send `{min, max}` for a real window —
 *  a porosity window that excludes both tight rock and bad-hole spikes, say. A range that can
 *  admit no value is refused by the backend rather than quietly booking zero net. */
export type CutoffSpec = CutoffEntry | { min?: CutoffSpecBound; max?: CutoffSpecBound };

/** SB-CUT-022. Which report tiers a cut-off is USED at.
 *
 *  An explicit flag per tier, never an inference. Geolog changed this trigger between two modules
 *  of one product - one fires on the presence of the CURVE, the other on the presence of the
 *  VALUE - and an inferred rule cannot be audited from a result, because the result does not
 *  record what was inferred.
 *
 *  Omitting a slot takes the shipped ladder: VSH at all three tiers, PHIE at reservoir and pay,
 *  SWE and PERM at pay only. Reservoir and pay share ONE value with independent flags. */
export interface CutoffUse {
  sand: boolean;
  reservoir: boolean;
  pay: boolean;
}

export interface PaySummaryRequest {
  /** SB-CUT-001 (DEC-071): "CENTRED" (default) or "TOPS" (the legacy forward rule). */
  discretisation?: "CENTRED" | "TOPS";
  /** Read this run's input curves from this log set (latest version per well); omit for the current values. */
  input_set?: string;
  well_ids: string[];
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min: CutoffSpec | null;
  /** SB-CUT-009. Per-curve averaging weighting, keyed by the SLOT the curve fills — `"VSH"`,
   *  `"PHIE"` or `"SWE"` — which is the ROLE that curve plays in the summation, never the
   *  mnemonic it happens to be stored under. Omit a slot to take the cited default: saturation
   *  is porosity-weighted `Σ(Sw·φ·h)/Σ(φ·h)`, which all three vendors agree on, and the rest is
   *  thickness-weighted. Omit the whole object for exactly the behaviour that shipped before. */
  weighting?: Record<string, "thickness" | "porosity">;
  /** SB-CUT-012. Depth frame to summate in; defaults to MD. Anything else is REFUSED with a
   *  message naming the frame and what is missing — never served as MD numbers relabelled. */
  frame?: "MD" | "TVD" | "TVDSS" | "TST";
  /** SB-CUT-016. Cut-offs the user switched ON and left blank. Any name here REFUSES the run —
   *  "I am not filtering on Sw" and "I meant to and have not said what" are different statements,
   *  and only one may produce a number. */
  enabled_unset?: string[];
  /** Write FLAG_* in place without creating a versioned log set, instead of versioning the pay
   *  flags (with the cutoffs in provenance) per well. Set by the report/composite render pass,
   *  whose flags are a render side-effect that should not churn the archive with a version per
   *  render. The explicit Cutoffs & Summary run leaves this false so its flags are versioned. */
  skip_version?: boolean;
  /** Compute + return the per-zone stats WITHOUT persisting any FLAG_* curves. The Field
   *  Dashboard sets this: it only reads the returned rows, so writing flags per well on every
   *  cutoff tweak was the dominant cost. Flag persistence stays with Cutoffs & Summary. */
  stats_only?: boolean;
  /** Required for the explicit flag-writing run; read-only summaries omit it. */
  custody?: RunCustody | null;
  /** SB-CUT-022. Which report tiers each cutoff is used at, keyed by SLOT (`VSH`, `PHIE`, `SWE`,
   *  `PERM`). Omit a slot for the shipped ladder; omit the whole map and nothing changes. */
  cutoff_use?: Record<string, CutoffUse>;
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
  /** SB-CUT-003. Footage the classifier EVALUATED and rejected — it saw the sample and the sample
   *  failed a cutoff. Deliberately separate from `unknown`: a zone reading 40 % net-to-gross
   *  because 60 % is shale and one reading 40 % because 55 % was never logged print the same
   *  number and are completely different rock. */
  not_net: number;
  /** SB-CUT-003. Footage whose flag could not be EVALUATED, so that
   *  `gross === net + not_net + unknown` holds exactly. Covers both an in-zone sample with no
   *  VSH/PHIE/SWE to judge AND footage carrying no sample at all — a logging gap, or a zone
   *  bottomed on a marker below the TD of the run that logged it. */
  unknown: number;
  /** SB-CUT-004. Net-to-gross over the footage that could actually be judged —
   *  `net / (gross - unknown)`, the chapter's `N:(G-Unknown)`. Reported BESIDE `ntg`, never
   *  instead of it: the gap between the two is the null fraction, and over a washed-out or
   *  partly-logged interval that gap is the whole argument about whether a net-to-gross is
   *  defensible.
   *
   *  `null` where nothing was judged — there is no denominator, and a printed 0.00 would be a
   *  claim about rock nobody looked at. (Backend f32::NAN crosses as JSON null.) */
  ntg_known: number | null;
  /** SB-CUT-005. Footage moved into the largest component so that
   *  `gross === net + not_net + unknown` closes — reported rather than printed, which is the
   *  point: a reconciliation whose correction is not recorded is indistinguishable from no
   *  reconciliation. Zero on any run whose partition already closed, which is every ordinary run.
   *  A residual beyond 1e-7 relative fails the summation instead of arriving here. */
  residual_absorbed: number;
  /** SB-CUT-030. True when a zonal average falls outside its quantity's physical bounds.
   *  The value is emitted AS COMPUTED, not corrected - a corrected average is a number nobody
   *  derived. Render it as a warning beside the value, never by rewriting the value. */
  out_of_range?: boolean;
  /** SB-CUT-012. The depth frame these weights were measured in — `"MD"`, `"TVD"`, `"TVDSS"` or
   *  `"TST"`. Part of the result's IDENTITY, not a display option: the per-sample weight is `Δz`
   *  in MD and `Δz·cos θ` in TVD, so the weights differ, by a factor of two in a 60° hold. Today
   *  the engine computes MD only and refuses any other frame rather than relabelling. */
  frame: "MD" | "TVD" | "TVDSS" | "TST";
  /** SB-CUT-012. What the per-sample weights were differenced from. Naming the frame alone does
   *  not say which depths produced the increments. */
  weights_source: string;
  /** SB-CUT-016. Cut-offs NOT applied to this summation, in VSH/PHIE/SWE/PERM order. An
   *  unfiltered summation is reported AS unfiltered — a net that quietly stopped being filtered,
   *  with nothing on the result to say so, is the failure this prevents. */
  unfiltered: string[];
  ntg: number;
  // The Rust engine emits f32::NAN for zone×flag rows with no valid in-zone samples, and
  // Tauri/serde_json encodes non-finite floats as JSON null — so these arrive as null, not NaN.
  avg_vsh: number | null;
  avg_phie: number | null;
  avg_swe: number | null;
  hpv: number;
  /** In-zone samples the classifier could judge. **0 means the well was never interpreted**
   *  (VSH/PHIE/SWE resolved to all-NaN) — which produces net/ntg/hpv of exactly 0, identical to
   *  a genuine zero-net result. Render "—" rather than 0.00 when this is 0. */
  n_classified: number;
  /** A permeability cutoff is active and this well carries no PERM anywhere, so every sample
   *  failed it for want of data. Per well, so it is the same on every zone row of that well.
   *
   *  The zero net pay this well reports is an absence of evidence, not a dry reservoir, and
   *  nothing else on the row distinguishes the two — `n_classified` is above zero either way.
   *  Show it wherever pay is read or summed (`docs/review_triage.md` finding 7).
   *
   *  It means "a cutoff was requested and this well has nothing to answer it with", never "this
   *  well has no permeability" — with no cutoff asked for there is nothing to report. */
  perm_cutoff_no_data: boolean;
  /** SB-POR-057 / DEC-070: this well's only porosity is the quick-look comparison curve,
   *  deliberately not summed - render the zeros as "not interpreted", never as wet. */
  quicklook_phie_excluded: boolean;
}

export async function runPaySummary(
  req: PaySummaryRequest,
  scope: BackendWellScope,
): Promise<PaySummaryRow[]> {
  return invoke<PaySummaryRow[]>("run_pay_summary", { req, scope });
}

/** Cutoff-sensitivity sweep (Method 1 of the cutoff study): sweep one cutoff over a range,
 *  holding the other two fixed, and report the pay metric per well at each step. */
export interface CutoffSweepRequest {
  /** SB-CUT-001 (DEC-071): "CENTRED" (default) or "TOPS" (the legacy forward rule). */
  discretisation?: "CENTRED" | "TOPS";
  /** Sweep against this log set's stored curves; omit for the current values. */
  input_set?: string;
  well_ids: string[];
  property: "VSH" | "PHIE" | "SWE";
  /** SB-CUT-016. `null` = UNFILTERED on this property; the result reports it as such. There is no
   *  default: four shipped vendor sets disagree, two of them from one vendor, and delivered work
   *  spans a wide range even within one field. */
  vsh_max: CutoffSpec | null;
  phie_min: CutoffSpec | null;
  swe_max: CutoffSpec | null;
  perm_min: CutoffSpec | null;
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

export async function runCutoffSweep(
  req: CutoffSweepRequest,
  scope: BackendWellScope,
): Promise<CutoffSweepResult> {
  return invoke<CutoffSweepResult>("run_cutoff_sweep", { req, scope });
}

export interface DepthComparison {
  left: number;
  right: number;
  difference: number;
  datum: DepthDatum;
}

export function compareZoneTopToContact(
  wellId: string,
  zoneName: string,
  contactId: string,
): Promise<DepthComparison> {
  return invoke<DepthComparison>("compare_zone_top_to_contact", { wellId, zoneName, contactId });
}

/** Full-resolution curve data for parameter-selection plots, optionally windowed to a
 *  depth interval. Binary transport, unpacked to Float32Arrays like getTrackData. */
export async function getCurveData(
  wellId: string,
  curveNames: string[],
  depthMin: number | null,
  depthMax: number | null,
): Promise<TrackCurveSeries[]> {
  await resolvePlotBindings(
    curveNames.map((semantic_request, index) => ({
      channel: `curve:${index}`,
      semantic_request,
      required: true,
    })),
    { kind: "explicit", well_ids: [wellId] },
  );
  const buf = await invoke<ArrayBuffer>("get_curve_data", { wellId, curveNames, depthMin, depthMax });
  return decodeCurveBuffer(buf);
}

export interface PlotChannelIntent {
  channel: string;
  semantic_request: string;
  required: boolean;
}

export interface ResolvedPlotCurve {
  well_id: string;
  curve_id: string;
  mnemonic: string;
  quantity: string;
  source_unit: string;
  display_unit: string;
  conversion: string;
  sample_count: number;
  resolution_reason: string;
  source_revision: string;
  header_display?: StoredDisplayRange;
}

export interface StoredDisplayRange {
  low: number;
  high: number;
}

export interface PlotChannelBinding {
  intent: PlotChannelIntent;
  resolved: ResolvedPlotCurve[];
}

export interface PersistedPlotState {
  schema_version: 1;
  plot_type: string;
  well_ids: string[];
  options: Record<string, unknown>;
  bindings: PlotChannelBinding[];
  /** Absent only in a pre-SB-PLT-002 legacy document; every new write carries this. */
  axis_ranges: PlotAxisRangeExport[];
}

export interface PlotBindingExport {
  schema_version: 1;
  well_ids: string[];
  bindings: PlotChannelBinding[];
  axis_ranges: PlotAxisRangeExport[];
}

const plotBindingRegistry = new Map<string, PlotChannelBinding>();

function rememberPlotBindings(bindings: PlotChannelBinding[]): void {
  for (const binding of bindings) {
    for (const resolved of binding.resolved) {
      plotBindingRegistry.set(
        `${resolved.well_id}\u0000${binding.intent.semantic_request.toUpperCase()}`,
        { intent: binding.intent, resolved: [resolved] },
      );
    }
  }
}

/** Rebuilds a channel-labelled binding record only from the concrete answers captured
 * by the reads that produced the current plot. Missing required answers stay empty so
 * typed persistence/export validation refuses them rather than resolving again. */
export function plotBindingSnapshotForChannels(
  wellIds: string[],
  intents: PlotChannelIntent[],
): PlotChannelBinding[] {
  return intents.map((intent) => {
    const resolved: ResolvedPlotCurve[] = [];
    for (const wellId of wellIds) {
      const binding = plotBindingRegistry.get(
        `${wellId}\u0000${intent.semantic_request.toUpperCase()}`,
      );
      if (binding) resolved.push(...binding.resolved);
    }
    return { intent, resolved };
  });
}

/** Concrete bindings accumulated by the plot reads in this session, suitable for
 * persisted plot state and provenance records without another curve-resolution pass. */
export function plotBindingSnapshot(wellIds: string[], curveNames: string[]): PlotChannelBinding[] {
  return plotBindingSnapshotForChannels(
    wellIds,
    curveNames.map((curveName) => ({
      channel: curveName,
      semantic_request: curveName,
      required: true,
    })),
  );
}

/** Resolves and validates the semantic plot request before numeric curve bytes are read. */
export async function resolvePlotBindings(
  intents: PlotChannelIntent[],
  scope: BackendWellScope,
): Promise<PlotChannelBinding[]> {
  const bindings = await invoke<PlotChannelBinding[]>("resolve_plot_bindings", { intents, scope });
  rememberPlotBindings(bindings);
  return bindings;
}

export function serializePlotBindingExport(
  wellIds: string[],
  bindings: PlotChannelBinding[],
  axisRanges: PlotAxisRangeExport[],
  statisticsRecords: PlotStatisticsRecord[] = [],
): Promise<string> {
  return invoke<string>("serialize_plot_binding_export", { wellIds, bindings, axisRanges, statisticsRecords });
}

export function getCurveHeaderDisplayRange(curveId: string): Promise<StoredDisplayRange | null> {
  return invoke<StoredDisplayRange | null>("get_curve_header_display_range", { curveId });
}

export function setCurveHeaderDisplayRange(
  curveId: string,
  range: StoredDisplayRange | null,
): Promise<StoredDisplayRange | null> {
  return invoke<StoredDisplayRange | null>("set_curve_header_display_range", { curveId, range });
}

export interface PlotWriteAxisBinding {
  channel: string;
  curve_id: string;
  mnemonic: string;
  quantity: string;
  source_unit: string;
  display_unit: string;
  conversion: string;
  source_revision: string;
}

export interface PlotWriteProvenanceInput {
  plot_id: string;
  plot_type: string;
  x_axis: PlotWriteAxisBinding;
  y_axis: PlotWriteAxisBinding;
  z_axis: PlotWriteAxisBinding | null;
  viewport: {
    x_min: number;
    x_max: number;
    y_min: number;
    y_max: number;
    x_log: boolean;
    y_log: boolean;
  };
  selection: {
    kind: string;
    selection_id: string | null;
    member_count: number;
    revision: string | null;
  };
  interval: {
    low: number | null;
    high: number | null;
    closure: "[lo,hi)";
  };
  method: string;
  fit_record: unknown | null;
  target: {
    well_id: string;
    zone_name: string;
    parameter_name: string;
    value: number;
  };
}

/** Backend adds the actual OS user and UTC timestamp, validates all mandatory fields,
 * and returns the canonical JSON stored as the zone parameter's source note. */
export function finalizePlotWriteProvenance(source: PlotWriteProvenanceInput | null): Promise<string> {
  return invoke<string>("finalize_plot_write_provenance", { source });
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
  index_resolution: ImportResult["index_resolution"];
}

/** Parses a core CSV (alias-resolved headers: DEPTH, CPOR/POR, CPERM/PERM, CGD, CSW)
 *  and replaces the given well's core plug data. */
export function importCoreCsv(
  wellId: string,
  path: string,
  depthColumn: number | null,
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user. */
  depthDatum: string,
): Promise<CoreImportResult> {
  return invoke<CoreImportResult>("import_core_csv", { wellId, path, depthColumn: depthColumn ?? null, depthDatum });
}

// --- Core import v2 (T-IMP-07): probe → confirm mapping → commit -------------

export interface WellRowCount {
  name: string;
  rows: number;
}

/** Everything the mapping dialog shows before anything is written. */
export interface TableProbe {
  headers: string[];
  n_rows: number;
  well: number | null;
  depth: number | null;
  cpor: number | null;
  cperm: number | null;
  cgd: number | null;
  csw: number | null;
  /** "number" | "text" | "empty" per column. */
  column_kind: string[];
  sample_rows: string[][];
  wells: WellRowCount[];
  /** Roles ("CPOR"/"CSW") whose values read as percent — divided to v/v on import. */
  percent_roles: string[];
  /** "ft" / "m" when the units row or depth header names one. */
  depth_unit_guess: string | null;
  units_row_skipped: boolean;
}

/** Dialog-confirmed column mapping (indices into the file's columns). */
export interface CoreMapping {
  well: number | null;
  depth: number;
  cpor: number | null;
  cperm: number | null;
  cgd: number | null;
  csw: number | null;
  /** Columns beyond the four core measurements, stored as point data (aux_data). */
  extras: number[];
}

export interface CoreWellOutcome {
  well_name: string;
  rows: number;
  imported: number;
  /** The core SET the plugs landed in on this well (auto-suffixed if the name was taken). */
  set_name: string | null;
  problem: string | null;
}

/** Declares a numeric write boundary and counts only values that actually changed. */
export interface SamplePrecisionReport {
  source_precision: string;
  destination_precision: string;
  reduced: boolean;
  values_reduced: number;
}

export interface CoreTableImportResult {
  path: string;
  rows_imported: number;
  wells_imported: number;
  outcomes: CoreWellOutcome[];
  skipped_blank_well: number;
  /** Point-data rows written from the file's extra columns, and which columns they
   *  came from (empty when no extras were mapped). */
  extra_rows: number;
  extra_items: string[];
  precision: SamplePrecisionReport;
  error: string | null;
}

/** Reads a core CSV/TXT (delimiter auto-detected) and reports headers, guessed roles,
 *  sample rows, distinct wells, percent + depth-unit detection. Writes nothing. */
export function probeCoreTable(path: string): Promise<TableProbe> {
  return invoke<TableProbe>("probe_core_table", { path });
}

/** Commits one core table under the confirmed mapping: rows route per well name
 *  (unmatched/ambiguous reported, never guessed) or all to `fallbackWellId`; depths
 *  convert from `depthUnit` ("ft"/"m"; null = already project unit). Columns in
 *  `mapping.extras` land as point data under `extrasDataset` (null = "CORE"). `setName`
 *  names the delivery: resolved per well, auto-suffixed rather than overwriting an earlier
 *  one, and the new set becomes that well's active core. */
export function importCoreTable(
  path: string,
  mapping: CoreMapping,
  depthUnit: string | null,
  fallbackWellId: string | null,
  extrasDataset: string | null = null,
  setName: string | null = null,
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user. */
  depthDatum = "MD",
): Promise<CoreTableImportResult> {
  return invoke<CoreTableImportResult>("import_core_table", {
    path,
    mapping,
    depthUnit,
    fallbackWellId,
    extrasDataset,
    setName,
    depthDatum,
  });
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
  /** Wells that received rows (multi-well files route by their WELL column). */
  wells_imported: number;
  /** Routing story: unmatched/ambiguous names, blank-well rows skipped. */
  notes: string | null;
  /** Set name(s) the delivery landed in — several when some wells already carried that
   *  name and theirs was suffixed. */
  sets: string[];
  error: string | null;
}

/** One point-data delivery (T-IMP-08 applied to every dataset). Exactly one set per
 *  (well, dataset) is active; readers and counts follow it. */
export interface AuxSetInfo {
  dataset: string;
  set_name: string;
  rows: number;
  active: boolean;
  source: string | null;
  imported_at: string | null;
}

export function listAuxSets(wellId: string): Promise<AuxSetInfo[]> {
  return invoke<AuxSetInfo[]>("list_aux_sets", { wellId });
}

export function setActiveAuxSet(wellId: string, dataset: string, setName: string): Promise<void> {
  return invoke<void>("set_active_aux_set", { wellId, dataset, setName });
}

export function deleteAuxSet(wellId: string, dataset: string, setName: string): Promise<number> {
  return invoke<number>("delete_aux_set", { wellId, dataset, setName });
}

export interface AuxRow {
  dataset: string;
  depth_top: number;
  depth_base: number | null;
  item: string;
  value_num: number | null;
  value_text: string | null;
}

/** Imports a tops-style point dataset (PETROGRAPHY / XRD / CEC / OIL SHOW / PERFORATION /
 *  custom) as a NEW named delivery: `setName` is auto-suffixed per well rather than
 *  overwriting an earlier one, and becomes that dataset's live set. */
export function importAuxData(
  wellId: string,
  dataset: string,
  path: string,
  setName: string | null = null,
  /** Treat the file's depths as the ones the original core report used, and place them through
   *  the target well's core depth record. Off by default: a file written on the log's scale must
   *  not be moved. The result's notes say what happened, including samples that fell outside the
   *  cored interval and wells with no core to follow. */
  followCore = false,
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user. */
  depthDatum = "MD",
): Promise<AuxImportResult> {
  return invoke<AuxImportResult>("import_aux_data", { wellId, dataset, path, setName, followCore, depthDatum });
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
  /** The SCAL delivery these points landed in (auto-suffixed if the name was taken). */
  set_name: string | null;
  /** What following the core depth record did, when it was asked for. */
  note: string | null;
  error: string | null;
}

/** One SCAL delivery of a well. Exactly one is active; Pc QC, Leverett-J and Thomeer
 *  fits all read it. */
export interface ScalSetInfo {
  set_name: string;
  rows: number;
  active: boolean;
  source: string | null;
  imported_at: string | null;
}

export function listScalSets(wellId: string): Promise<ScalSetInfo[]> {
  return invoke<ScalSetInfo[]>("list_scal_sets", { wellId });
}

export function setActiveScalSet(wellId: string, setName: string): Promise<void> {
  return invoke<void>("set_active_scal_set", { wellId, setName });
}

export function deleteScalSet(wellId: string, setName: string): Promise<number> {
  return invoke<number>("delete_scal_set", { wellId, setName });
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
export function importScalCsv(
  wellId: string,
  path: string,
  iftLab: number,
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user. */
  depthDatum = "MD",
): Promise<ScalImportResult> {
  return invoke<ScalImportResult>("import_scal_csv", { wellId, path, iftLab, depthDatum });
}

/** SCAL Pc file shapes the importer understands: "long" flat Pc/Sw rows, "porous_plate"
 *  Corelab-style wide tables (pressure columns × plug rows), "centrifuge" per-plug
 *  key-value blocks + Pc/Sw tables, or "auto" to sniff each file. */
export type ScalFormat = "auto" | "long" | "porous_plate" | "centrifuge";

/** Multi-file SCAL Pc import (e.g. a set of single-plug centrifuge exports): the files
 *  selected together form ONE delivery, stored as the named SCAL set (auto-suffixed rather
 *  than overwriting an earlier report) and made live, with the Leverett-J fit over the
 *  pooled points. `system` labels every stored point with the lab fluid system
 *  ('air_brine', 'hg_air', 'oil_brine', ...) alongside the entered sigma·cosθ. */
export function importScalFiles(
  wellId: string,
  paths: string[],
  format: ScalFormat,
  system: string,
  iftLab: number,
  setName: string | null = null,
  /** Treat the plug depths as the ones the original core report used, and place them through the
   *  well's core depth record. SCAL plugs ARE core plugs, so they move with the core. */
  followCore = false,
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user. */
  depthDatum = "MD",
): Promise<ScalImportResult> {
  return invoke<ScalImportResult>("import_scal_files", { wellId, paths, format, system, iftLab, setName, followCore, depthDatum });
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
export function runThomeerFit(scope: BackendWellScope): Promise<ThomeerResult> {
  return invoke<ThomeerResult>("run_thomeer_fit", { req: { well_ids: [] }, scope });
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
export function runHfuCluster(scope: BackendWellScope, nClusters: number, method: HfuMethod): Promise<HfuResult> {
  return invoke<HfuResult>("run_hfu_cluster", {
    req: { well_ids: [], n_clusters: nClusters, method },
    scope,
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
  scope: BackendWellScope = { kind: "explicit", well_ids: [wellId] },
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
    scope,
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

// ---------------------------------------------------------------------------
// Array logs: a whole distribution at every depth (Monte Carlo realizations,
// NMR T2 distributions, waveforms) rather than one reading per depth.
// ---------------------------------------------------------------------------

export interface ArrayCurveInfo {
  set_name: string;
  curve_name: string;
  /** Number of depths the array covers. */
  depths: number;
  /** Values per depth in the widest row (realizations, T2 bins, …). */
  width: number;
  depth_min: number;
  depth_max: number;
}

/** One array log in memory: depths, and a row-major matrix `width` values wide.
 *  A padded or failed slot is NaN, which every consumer already drops. */
export interface ArrayLog {
  depth: Float32Array;
  values: Float32Array;
  width: number;
}

export function listArrayCurves(wellId: string): Promise<ArrayCurveInfo[]> {
  return invoke<ArrayCurveInfo[]>("list_array_curves", { wellId });
}

export function deleteArrayLog(wellId: string, setName: string, curveName: string): Promise<number> {
  return invoke<number>("delete_array_log", { wellId, setName, curveName });
}

/** Decodes the raw array-log buffer. Mirrors the layout documented on `get_array_log` in
 *  `src-tauri/src/lib.rs`: [u32 depth_count][u32 width][f32 depth × dc][f32 values × dc*width].
 *
 *  Copied out of the response rather than aliased: the two f32 blocks start at byte offset 8,
 *  which is 4-byte aligned, but slicing keeps the whole IPC buffer alive for as long as either
 *  view is held — a costly thing to leak for a matrix this size. */
export function decodeArrayLog(buf: ArrayBuffer): ArrayLog {
  const view = new DataView(buf);
  const depthCount = view.getUint32(0, true);
  const width = view.getUint32(4, true);
  const depth = new Float32Array(buf.slice(8, 8 + depthCount * 4));
  const values = new Float32Array(buf.slice(8 + depthCount * 4, 8 + depthCount * 4 + depthCount * width * 4));
  return { depth, values, width };
}

export async function getArrayLog(wellId: string, setName: string | null, curveName: string): Promise<ArrayLog> {
  const buf = await invoke<ArrayBuffer>("get_array_log", { wellId, setName, curveName });
  return decodeArrayLog(buf);
}

// ---------------------------------------------------------------------------
// Depth-registered images (thin sections, core photographs)
// ---------------------------------------------------------------------------

/** One picture's METADATA — never its pixels. A well carrying 300 core photographs has to
 *  list in kilobytes, so the bytes are fetched one at a time by `getWellImage`. */
export interface ImageInfo {
  image_id: string;
  dataset: string;
  set_name: string;
  depth_top: number;
  /** null = a POINT sample: a thin section is cut from one plug and has no thickness. */
  depth_base: number | null;
  name: string;
  caption: string | null;
  mime: string;
  width: number;
  height: number;
  src_width: number | null;
  src_height: number | null;
  source_path: string | null;
  /** false = the viewer shows it but the PDF exporter cannot embed it (see images.rs). */
  printable: boolean;
  bytes: number;
  /** Width of the WHOLE picture in micrometres. null = no scale was declared, and nothing
   *  dimensional may run on this plate. um/px of any copy = fov_um / that copy pixel width. */
  fov_um: number | null;
  /** "" = unknown (refused, never assumed), "blue_epoxy", "plain". */
  prepared: string;
  /** As the laboratory report names it; "" = none or not stated. */
  stain: string;
}

export interface ImageSetInfo {
  dataset: string;
  set_name: string;
  images: number;
  active: boolean;
  source: string | null;
  imported_at: string | null;
  bytes: number;
}

/** One selected file as the import wizard shows it, before anything is stored. */
export interface ImageProbe {
  path: string;
  file_name: string;
  name: string;
  mime: string;
  /** 0 when only Pillow can tell (TIFF, plain WebP). */
  width: number;
  height: number;
  bytes: number;
  depth_top: number | null;
  depth_base: number | null;
  error: string | null;
}

export interface ImageImportItem {
  path: string;
  name: string;
  depth_top: number;
  depth_base?: number | null;
  caption?: string | null;
  /** Overrides the delivery value; absent falls back to it. Magnification genuinely varies
   *  within one delivery, which is why this is per plate. */
  fov_um?: number | null;
}

export interface ImageImportRequest {
  well_id: string;
  dataset: string;
  set_name: string;
  depth_unit?: string | null;
  max_px?: number | null;
  quality?: number | null;
  /** Place the plate depths through the well's core depth record — a section is cut from a plug,
   *  so when that plug is re-registered the plate belongs with it. */
  follow_core?: boolean;
  /** SB-DBM-031: the datum the plates' depths are quoted in, declared in the wizard. */
  depth_datum: string;
  /** Delivery-level defaults. All absent by default, and absent is a real answer. */
  fov_um?: number | null;
  prepared?: string | null;
  stain?: string | null;
  items: ImageImportItem[];
}

/** One plate field of view and preparation. Every value is written as given, null included —
 *  a scale typed by mistake has to be clearable. */
export function setImageDetails(
  imageId: string,
  fovUm: number | null,
  prepared: string | null,
  stain: string | null,
): Promise<number> {
  return invoke<number>("set_image_details", { imageId, fovUm, prepared, stain });
}

/** The same three facts across a whole live delivery, in one statement. */
export function setImageDeliveryDetails(
  wellId: string,
  dataset: string,
  fovUm: number | null,
  prepared: string | null,
  stain: string | null,
): Promise<number> {
  return invoke<number>("set_image_delivery_details", { wellId, dataset, fovUm, prepared, stain });
}

export interface ImageImportResult {
  dataset: string;
  set_name: string;
  imported: number;
  skipped: string[];
  bytes: number;
  note: string | null;
}

export function probeImageFiles(paths: string[]): Promise<ImageProbe[]> {
  return invoke<ImageProbe[]>("probe_image_files", { paths });
}

/** One plate lifted out of a petrography workbook. */
export interface WorkbookPlate {
  /** A temporary file the normal importer then reads — one import path, not two. */
  path: string;
  name: string;
  sheet: string;
  /** A, B, C… in the order the pictures were anchored on the sheet. */
  panel: string;
  width: number;
  height: number;
  /** From the sheet's own header CELL, never from a file name. `null` when the sheet stated none. */
  depth_top: number | null;
  depth_base: number | null;
  /** 'ft' or 'm', as the sheet wrote it. */
  unit: string | null;
  /** As stated on the sheet ('10x'). Never converted to a scale — that needs the camera and tube. */
  magnification: string | null;
  bytes: number;
}

export interface WorkbookProbe {
  plates: WorkbookPlate[];
  /** The one depth unit, when every sheet that stated one agreed; null means ask. */
  depth_unit: string | null;
  notes: string[];
}

/** Lifts the plates out of petrography workbooks so the import wizard can take them.
 *
 *  A petrography delivery arrives as a workbook with one worksheet per plate — the well, the depth
 *  and the magnification typed into cells, the pictures anchored on top — which a file picker
 *  cannot read at all. Only `.xlsx`/`.xlsm`; the old `.xls` is refused by name with the fix. */
export function probePlateWorkbooks(paths: string[]): Promise<WorkbookProbe> {
  return invoke<WorkbookProbe>("probe_plate_workbooks", { paths });
}

export interface PackageRuntimeSupport {
  distribution: string;
  selected_interpreter: string | null;
  available: boolean;
  version: string | null;
  message: string;
}

/** Pillow status and manifest-derived remediation for the image wizard. */
export function imageSupport(): Promise<PackageRuntimeSupport> {
  return invoke<PackageRuntimeSupport>("image_support");
}

export function importWellImages(req: ImageImportRequest): Promise<ImageImportResult> {
  return invoke<ImageImportResult>("import_well_images", { req });
}

export function listWellImages(wellId: string, dataset?: string | null): Promise<ImageInfo[]> {
  return invoke<ImageInfo[]>("list_well_images", { wellId, dataset: dataset ?? null });
}

export function listImageDatasets(wellId: string): Promise<[string, number][]> {
  return invoke<[string, number][]>("list_image_datasets", { wellId });
}

/** The pixels of one picture, as raw bytes (rule 3 — never a JSON array). Wrapped into a
 *  Blob by the caller, which already knows the mime type from `listWellImages`. */
// ---------------------------------------------------------------------------
// Core slab photograph conditioning (coreimage.rs)
// ---------------------------------------------------------------------------

/** A rectangle as FRACTIONS of the picture it was drawn on, never pixels — the stored copy is
 *  already resampled to a long-edge cap, so a pixel rectangle would belong to whichever copy it
 *  was dragged on. It is also what lets the preview and the full-size bake describe the same
 *  rectangle. Taken on the ROTATED picture, because that is what the user dragged across. */
export interface CropBox {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** What was done to one photograph. Every field defaults to "no change", so an empty recipe is
 *  exactly the imported picture. */
export interface CoreRecipe {
  /** Deskew, degrees CLOCKWISE. Applied before the crop, so a rotation's empty corners are cut
   *  away rather than printed. */
  rotate_deg?: number;
  /** The four corners of the box as the camera saw them, FRACTIONS, in reading order: top-left,
   *  top-right, bottom-right, bottom-left. Rectifying deliberately changes the aspect ratio — a box
   *  shot from one end arrives with its shape already wrong, and the output takes its proportions
   *  from these corners rather than from the frame. */
  quad?: Quad | null;
  crop?: CropBox | null;
  /** Per-channel gains from a neutral patch the user clicked. Normalised so the LARGEST is 1, so
   *  it can only darken and never clips the brightest pixels. */
  gain?: [number, number, number] | null;
  /** Manual white-balance trim on top of the picked patch: blue-to-amber. */
  warmth?: number;
  /** The other axis: green-to-magenta. */
  tint?: number;
  /** Stops. */
  exposure?: number;
  contrast?: number;
  saturation?: number;
  /** Speckle removal, 0..1 — a median filter whose radius is a fraction of the long edge, so the
   *  preview and the full-size bake take the same thing out of the rock. */
  denoise?: number;
  /** Local contrast, 0..1 (contrast-limited adaptive histogram equalisation). */
  clarity?: number;
  /** Unsharp mask, 0..1. */
  sharpen?: number;
}

/** Four corners as fractions, top-left, top-right, bottom-right, bottom-left. */
export type Quad = [[number, number], [number, number], [number, number], [number, number]];

export interface CorePreview {
  /** The conditioned proxy, base64 PNG. */
  png: string;
  width: number;
  height: number;
  /** The same proxy with nothing applied — sent together so before and after are the same decode
   *  of the same picture, and a stale one can never linger beside a fresh one. */
  before_png: string;
  before_width: number;
  before_height: number;
  hist_r: number[];
  hist_g: number[];
  hist_b: number[];
  /** Present when the call carried a pick: the gains that neutralise the patch clicked, and the
   *  colour it actually is, so a swatch can be shown instead of three numbers. */
  picked_gain?: [number, number, number] | null;
  picked_rgb?: [number, number, number] | null;
}

export interface BakeItem {
  image_id: string;
  recipe: CoreRecipe;
}

export interface BakeResult {
  conditioned: number;
  /** Pictures whose recipe was cleared, so the import was restored. */
  restored: number;
  skipped: string[];
  notes: string[];
}

/** Are numpy and Pillow reachable? Probed once so the workspace can say what is missing before a
 *  photograph is opened rather than after a slider is moved. */
export async function coreImageSupport(): Promise<boolean> {
  return invoke<boolean>("core_image_support");
}

/** One photograph at preview size under a recipe, with the un-conditioned proxy and a histogram.
 *  Writes nothing. `pickX`/`pickY` are fractions of the rotated, cropped picture — where the user
 *  clicked on what they can see — and the gains come back computed BEFORE any colour operation, so
 *  clicking the same grey twice gives the same answer rather than compounding. */
export async function previewCoreImage(
  imageId: string,
  recipe: CoreRecipe,
  pickX?: number,
  pickY?: number
): Promise<CorePreview> {
  return invoke<CorePreview>("preview_core_image", {
    imageId,
    recipe,
    pickX: pickX ?? null,
    pickY: pickY ?? null,
  });
}

/** Bakes recipes into pictures, keeping each import so the conditioning stays reversible. An empty
 *  recipe restores the photograph as delivered. */
export async function bakeCoreImages(items: BakeItem[]): Promise<BakeResult> {
  return invoke<BakeResult>("bake_core_images", { items });
}

/** Copies one photograph's LIGHT across a whole live delivery, keeping each picture's own framing.
 *  The merge happens in Rust so what "the look" means is one rule rather than one per caller. */
export async function applyCoreLook(
  wellId: string,
  dataset: string,
  look: CoreRecipe
): Promise<BakeResult> {
  return invoke<BakeResult>("apply_core_look", { wellId, dataset, look });
}

/** Curve-name prefix for every measure read off a core photograph.
 *
 *  Deliberately NOT `VSH`. A photograph's darkness co-varies with shale in most clastic sections,
 *  which is not the same statement as being a shale volume — the same dark band is organic mudstone
 *  in one core, oil stain in another. A curve called VSH is read by every module downstream as a
 *  shale volume, and an uncalibrated one under that name is a wrong answer that computes and plots.
 *  The same reason `GRAIN_D50_APP` is not `GRAIN_D50`. */
export const CORE_LOG_PREFIX = "CPHOTO";

/** One run of core inside a packed photograph — one column of a core-display plate, one row of a
 *  core box. `start`/`end` are fractions of the ACROSS-core axis, in reading order.
 *
 *  The depths are the barrel's OWN, and they are all-or-nothing across a picture's lanes: a plate
 *  carries four separate barrels with preserved intervals and part-filled columns between them,
 *  none of which one span divided in four can express. Where none is given the picture's own
 *  interval is shared out by lane length, which is what an equal split always did. */
export interface Lane {
  start: number;
  end: number;
  depth_top?: number | null;
  depth_base?: number | null;
}

/** How ONE picture is laid out. Per picture, because every plate of a delivery carries different
 *  barrels. Held as a `corelanes` document, the way the mineral classifier holds its clicks. */
export interface PlateLayout {
  /** The fraction of the down-core axis that is core, so a title block above the columns and a
   *  caption below them are not read as the shallowest and deepest rock in the barrel. */
  span?: [number, number] | null;
  lanes: Lane[];
}

/** What a column-detection pass found in one picture. */
export interface LaneDetection {
  /** In reading order, WITHOUT depths — nothing in the pixels says what depth a column came from. */
  lanes: Lane[];
  span: [number, number];
  /** The across-axis brightness profile the split was made from, decimated for drawing. Returned
   *  for the reason `registration.rs` returns its whole correlogram: four clean columns and a smear
   *  cut in four are the same answer and completely different situations. */
  profile: number[];
  threshold: number;
  notes: string[];
}

/** A proposed recipe for one picture, and the measurement behind every value. */
export interface RecipeAdvice {
  image_id: string;
  name: string;
  recipe: CoreRecipe;
  reasons: string[];
  notes: string[];
  error?: string | null;
}

/** Proposes where the runs of core are in one packed photograph. Proposes only. */
export function detectCoreLanes(
  imageId: string,
  axis: "x" | "y",
  reverse: boolean,
): Promise<LaneDetection> {
  return invoke<LaneDetection>("detect_core_lanes", { imageId, axis, reverse });
}

/** Measures a delivery and proposes conditioning for each picture, with reasons. Never applies. */
export function recommendCoreRecipe(imageIds: string[]): Promise<RecipeAdvice[]> {
  return invoke<RecipeAdvice[]>("recommend_core_recipe", { imageIds });
}

export interface CoreLogSpec {
  /** Version the outputs into this log set; omit for the tool's own default. */
  output_set?: string;
  well_id: string;
  dataset: string;
  /** Which way depth runs across the conditioned picture: "x" along the width, "y" down it. */
  axis?: "x" | "y";
  /** The picture is laid out deepest-first. */
  reverse?: boolean;
  /** Rows of core in one photograph, split into equal lanes and read in order. An APPROXIMATION —
   *  a real box has unequal rows and gaps — so the default is 1 and nobody gets it without asking. */
  lanes?: number;
  /** Per-picture lay-outs, keyed by image id — the columns of a core-display plate and the barrel
   *  each one covers. A picture named here uses its own columns; anything else falls back to
   *  `lanes` equal lanes over its own interval. */
  layouts?: Record<string, PlateLayout>;
  /** Depth step of the output curve, in the project's depth unit. */
  step?: number;
  /** Which light this delivery was shot under. DECLARED, never detected: a UV frame is dark and
   *  so is a daylight photograph of dark shale in a shadowed box, and the evidence for "this is
   *  ultraviolet" would be the brightness about to be measured. */
  light?: "white" | "uv";
  /** What counts as fluorescence, when `light` is "uv". Empty falls back to one generic band. */
  fluor?: FluorClass[];
  /** Report how each measure tracks this curve, usually GR. It is the only thing that says whether
   *  the trace is about the rock. */
  compare_curve?: string | null;
  /** Also write `CPHOTO_LITH`, a two-class curve cut out of the darkness trace — 0 lighter,
   *  1 darker. White light only: under UV the brightness IS the fluorescence. It is a reading of
   *  DARKNESS and never a shale volume, which is why it keeps the CPHOTO prefix. */
  lith?: boolean;
  /** The darkness at which the class changes. Omitted proposes Otsu's cut on this core's own
   *  trace — a method, not a calibration carried from anybody else's rock. */
  lith_cut?: number | null;
  /** Unfold for dipping beds: how much DEEPER the bedding sits at the right edge of the core than
   *  at the left, in the project's depth unit. An angle would need the core's diameter, which
   *  nothing here stores; the drop is read straight off the picture. */
  unfold?: number | null;
  /** The thinnest bed `CPHOTO_LITH` keeps, in the project's depth unit. Omitted leaves the cut
   *  exactly as the threshold made it. No default on purpose: a minimum bed thickness is a
   *  statement about the rock and about what the study is for, and no value is right in two
   *  cores. Counted in samples, so an unphotographed gap adds no thickness. */
  lith_min_bed?: number | null;
  /** Propose an unfold: the widest drop to search, in the project's depth unit. Omitted runs no
   *  scan. The whole scan comes back in `CoreLogResult.unfold_scan` and nothing is applied. */
  unfold_scan?: number | null;
  /** Write the curves. Omit to measure without writing, so a lay-out can be tried first. */
  write?: boolean;
  /** Required only when `write` persists CPHOTO curves. */
  custody?: RunCustody | null;
}

/**
 * How sharply the core reads at each candidate dip — the whole curve, not only its peak.
 *
 * `registration.rs`'s contract, and it is the reason this is a shape rather than a number. One
 * sharp peak means the dip is determined. A flat scan means the core carries no bedding contrast
 * to find a dip from, so the maximum is whichever candidate the noise favoured. A comb of
 * near-equal peaks means the section repeats. All three return a number.
 */
export interface UnfoldScan {
  /** The candidate drops tried, in the project's depth unit. Signed: a bed can dip either way. */
  drops: number[];
  /** One score per candidate — the trace's own contrast. NaN where the candidate sheared away too
   *  much of the core to be compared with the rest. */
  scores: number[];
  /** The best-scoring candidate. A PROPOSAL: read it beside the scan and type it in. */
  best?: number | null;
  /** Rival peaks within 5% of the best, away from it. */
  rivals: number;
  notes: string[];
}

/**
 * One kind of fluorescence, as the user describes it.
 *
 * Structurally a `PoreColorBand` plus a name and a saturation CEILING, so the shared colour-band
 * control drives the hue window and the two floors unchanged.
 *
 * **The ceiling is not decoration.** Fluorescence is routinely described as *dull blue-white*, and
 * white is the ABSENCE of colour — it cannot be written as a floor. Same distinction that makes
 * `StainBand` carry one so dolomite can be identified by staying colourless.
 */
export interface FluorClass {
  /** Becomes the curve suffix, upper-cased: `SHOW` gives `CPHOTO_FLUOR_SHOW`. */
  name: string;
  hue_lo: number;
  hue_hi: number;
  sat_min: number;
  /** 1 is no ceiling. Lower it to reach the pale end of a description. */
  sat_max?: number;
  val_min: number;
}

/** The shipped band: generic round numbers to start a VISUAL tuning from, never a calibration —
 *  what a fluorescing oil photographs as depends on the lamp, the camera and the exposure. Kept in
 *  step with `coreimage.rs::default_fluor`. */
export const DEFAULT_FLUOR: FluorClass = {
  name: "SHOW",
  hue_lo: 40,
  hue_hi: 200,
  sat_min: 0.2,
  sat_max: 1,
  val_min: 0.35,
};

export interface CoreLogCurve {
  name: string;
  n: number;
  p10: number;
  p50: number;
  p90: number;
  /** SIGNED agreement with the compared curve. Darkness and GR should both rise into shale, so a
   *  strongly negative value on DARK is a finding rather than a weak result — most often the depth
   *  axis is the other way round. NaN when nothing was compared. */
  correlation: number;
  pairs: number;
  /** Evenly spread down the interval for drawing — never the first N, which would be the top of
   *  the core rather than the core. */
  preview: number[];
}

export interface CoreLogResult {
  photographs: number;
  samples: number;
  depth_min: number;
  depth_max: number;
  curves: CoreLogCurve[];
  preview_depth: number[];
  written: string[];
  skipped: string[];
  notes: string[];
  /** Present only when `unfold_scan` asked for a proposal. Never applied. */
  unfold_scan?: UnfoldScan | null;
}

/** Reads the proxy measures off a well's live core-photograph delivery, and optionally writes them
 *  as curves. Reads the CONDITIONED pictures, so a darkness is comparable across boxes. */
export async function extractCoreLog(spec: CoreLogSpec): Promise<CoreLogResult> {
  return invoke<CoreLogResult>("extract_core_log", { spec });
}

/** Where depth strips are written unless the caller names another dataset. */
export const CORE_STRIP_DATASET = "CORE STRIP";

/** How a box is laid out — the SAME vocabulary the trace uses, because the strip and the trace read
 *  the box the same way and are built from one statement of that lay-out. */
export interface StripSpec {
  well_id: string;
  /** The photographs to cut up: the live delivery of this dataset. */
  dataset: string;
  /** "x" — the core runs across the frame; "y" — down it. */
  axis?: "x" | "y";
  lanes?: number;
  reverse?: boolean;
  /** Where the strips land. Defaults to CORE STRIP; may not be the source dataset. */
  target?: string | null;
}

export interface StripResult {
  dataset: string;
  set_name: string;
  built: number;
  skipped: string[];
  notes: string[];
}

/** Cuts each box of a delivery into its rows and stacks them into ONE tall depth-registered picture
 *  per box, so an ordinary image track in depth mode shows the core running beside the logs.
 *
 *  The lay-out happens here rather than at draw time: doing it while drawing would mean writing the
 *  same rotation and re-stacking three times — the log view, the SVG export and the PDF export —
 *  with nothing to stop the three drifting apart. Rebuilding REPLACES: a strip is derived, not
 *  delivered. */
export async function buildCoreStrips(spec: StripSpec): Promise<StripResult> {
  return invoke<StripResult>("build_core_strips", { spec });
}

/** The conditioning recipe of every picture in a dataset's live delivery, as (image_id, json).
 *  Empty string where a picture is exactly as imported. Never reads a blob. */
export async function listImageRecipes(
  wellId: string,
  dataset: string
): Promise<[string, string][]> {
  return invoke<[string, string][]>("list_image_recipes", { wellId, dataset });
}

export async function getWellImage(imageId: string): Promise<ArrayBuffer> {
  return invoke<ArrayBuffer>("get_well_image", { imageId });
}

export function listImageSets(wellId: string): Promise<ImageSetInfo[]> {
  return invoke<ImageSetInfo[]>("list_image_sets", { wellId });
}

export function setActiveImageSet(wellId: string, dataset: string, setName: string): Promise<void> {
  return invoke("set_active_image_set", { wellId, dataset, setName });
}

export function deleteImageSet(wellId: string, dataset: string, setName: string): Promise<number> {
  return invoke<number>("delete_image_set", { wellId, dataset, setName });
}

export function deleteWellImage(imageId: string): Promise<number> {
  return invoke<number>("delete_well_image", { imageId });
}

/** Re-registers one picture: core-to-log alignment and labelling, for pictures. */
export function updateWellImage(
  imageId: string,
  depthTop: number,
  depthBase: number | null,
  name: string,
  caption: string | null,
): Promise<number> {
  return invoke<number>("update_well_image", { imageId, depthTop, depthBase, name, caption });
}

/** Moves a whole plate delivery by a constant depth. `dataset` null = every live plate in the
 *  well. A plate with no base stays a POINT sample; an interval keeps its thickness. */
export function shiftWellImages(wellId: string, dataset: string | null, delta: number): Promise<number> {
  return invoke<number>("shift_well_images", { wellId, dataset, delta });
}

export function updateCoreSample(wellId: string, depth: number, column: string, value: number): Promise<void> {
  return invoke("update_core_sample", { wellId, depth, column, value });
}

export interface CoreShiftCounts {
  plugs: number;
  /** Point-data rows moved along with them. */
  extras: number;
  /** SCAL Pc rows moved. */
  scal: number;
  /** Pictures moved. */
  plates: number;
  /** The ranges that put this operation back, in the depths that exist AFTER it. Use these for
   *  undo rather than negating your own deltas — barrels moved by different amounts can produce
   *  overlapping ranges, and the first match wins. Empty for a whole-well shift. */
  inverse: RunShift[];
}

/** Shifts a well's ACTIVE core delivery by `delta` (core-to-log alignment). The plugs and the
 *  point measurements made ON those plugs move together — pass `datasets` to say which point
 *  datasets ride along (omit for the ones delivered with this core, `[]` for plugs only).
 *  Exactly reversible with -delta, which is what makes it undoable. */
export function shiftCoreData(
  wellId: string,
  delta: number,
  targets?: ShiftTargets,
  note?: RegistrationNote,
): Promise<CoreShiftCounts> {
  return invoke<CoreShiftCounts>("shift_core_data", { wellId, delta, targets, note });
}

/** Why a shift is being applied. Travels WITH the shift rather than being logged afterwards: a
 *  depth registration that committed without its reason is the state the record exists to
 *  prevent, and the backend writes both in one transaction. */
export interface RegistrationNote {
  /** "proposed" (correlation-backed), "manual" (a typed amount), "undo". */
  kind: string;
  log_curve?: string;
  reference?: string;
  pairing?: string;
  /** Agreement at the shift ACTUALLY applied — not the peak of the scan. */
  correlation?: number | null;
  n_pairs?: number | null;
  note?: string;
}

/** One line of a core's depth history. */
export interface RegistrationEntry {
  set_name: string;
  seq: number;
  applied_at: string | null;
  kind: string;
  /** null for a whole-core shift: no range was declared. */
  top: number | null;
  base: number | null;
  delta: number;
  log_curve: string;
  reference: string;
  pairing: string;
  correlation: number | null;
  n_pairs: number | null;
  note: string;
}

/** A well's core depth history, newest first. An event log — an undo shows up as its own
 *  reversal rather than erasing the row it reversed. */
export function listCoreRegistrations(wellId: string): Promise<RegistrationEntry[]> {
  return invoke<RegistrationEntry[]>("list_core_registrations", { wellId });
}

/** One barrel's correction: everything currently between `top` and `base` moves by `delta`.
 *  Ranges are CURRENT depths — what you read off the log view. */
export interface RunShift {
  top: number;
  base: number;
  delta: number;
  /** Agreement at THIS range's own shift, for the depth record. Omit when the range was not
   *  proposed against anything — absent means "not measured", never zero. */
  correlation?: number | null;
  n_pairs?: number | null;
}

/** Applies per-barrel (or finer) corrections to the active core delivery. Rejects — and changes
 *  nothing for — any set that would put deeper rock above shallower rock. */
export function applyCoreRunShifts(
  wellId: string,
  runs: RunShift[],
  targets?: ShiftTargets,
  note?: RegistrationNote,
): Promise<CoreShiftCounts> {
  return invoke<CoreShiftCounts>("apply_core_run_shifts", { wellId, runs, targets, note });
}

/** What a core registration should carry with it. Omit entirely to move the point data that
 *  provably came in with the core table; pass an empty object for plugs only. */
export interface ShiftTargets {
  aux_datasets?: string[];
  scal?: boolean;
  image_datasets?: string[];
}

/** One delivery a core shift could carry. `on_core_depths` marks the ones imported as sitting on
 *  the core depth scale — those are pre-ticked, the rest are offered but left alone. */
export interface ShiftCandidate {
  kind: "aux" | "scal" | "image";
  dataset: string;
  set_name: string;
  rows: number;
  on_core_depths: boolean;
}

export function coreShiftCandidates(wellId: string): Promise<ShiftCandidate[]> {
  return invoke<ShiftCandidate[]>("core_shift_candidates", { wellId });
}

/** The well's core depth record: `[depth the lab wrote, depth it sits at now]` per plug. */
export function coreDepthPairs(wellId: string): Promise<[number, number][]> {
  return invoke<[number, number][]>("core_depth_pairs", { wellId });
}

/** Maps lab-written depths onto where that rock now sits. Each result is
 *  `[depth, extrapolated]`; `extrapolated` marks samples outside the cored interval, where the
 *  correction is held from the nearest end rather than measured. */
export function mapCoreDepths(wellId: string, depths: number[]): Promise<[number, boolean][]> {
  return invoke<[number, boolean][]>("map_core_depths", { wellId, depths });
}

/** Point datasets delivered as part of this well's active core table, with their row counts. */
export function coreExtraDatasets(wellId: string): Promise<[string, number][]> {
  return invoke<[string, number][]>("core_extra_datasets", { wellId });
}

// --- Core-to-log depth registration ---

export interface CoreReference {
  /** "core" = a plug-table column; "aux" = an item of a point dataset; "curve" = the core
   *  photograph's own proxy trace, which is the densest reference this dialog has. */
  kind: string;
  dataset: string;
  item: string;
  label: string;
  n: number;
  /** Resolved family, "" when the name is not recognised. */
  family: string;
}

export function listCoreReferences(wellId: string): Promise<CoreReference[]> {
  return invoke<CoreReference[]>("list_core_references", { wellId });
}

export interface RegistrationRequest {
  well_id: string;
  log_curve: string;
  ref_kind: string;
  ref_dataset?: string;
  ref_item: string;
  depth_from?: number | null;
  depth_to?: number | null;
  search_range?: number;
  step?: number;
}

export interface RegPoint {
  depth: number;
  value: number;
}

/** One rung of the correlogram: agreement if the core moved by `delta`. */
export interface LagPoint {
  delta: number;
  r: number;
  n: number;
}

export interface RegistrationResult {
  core: RegPoint[];
  log_depth: number[];
  log_value: number[];
  proposed_delta: number;
  correlation: number;
  current_r: number;
  n_pairs: number;
  like_for_like: boolean;
  /** "direct" | "inverse". */
  matched_on: string;
  log_family: string;
  ref_family: string;
  reference_label: string;
  scan: LagPoint[];
  notes: string[];
  error: string | null;
}

/** Proposes the shift that best aligns a well's core with a log. Writes nothing. */
export function proposeRegistration(req: RegistrationRequest): Promise<RegistrationResult> {
  return invoke<RegistrationResult>("propose_registration", { req });
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
  /** Required by the backend when the target resolves to a computed curve. */
  custody?: RunCustody;
}

/** `data` holds the CHANGED rows' previous samples as packed `depth[n] + value[n]`
 *  f32-LE bytes — pass it back to {@link restoreCurveValues} verbatim to undo. */
export interface CurveEditResult {
  affected: number;
  store: string;
  point_count: number;
  data: Uint8Array | number[];
  edit_id: string;
  curve_sha256: string;
}

export type CurveEditInterval =
  | { kind: "WHOLE_CURVE" }
  | { kind: "INCLUSIVE_DEPTH"; top: number; bottom: number };

export interface CurveEditRecord {
  edit_id: string;
  well_id: string;
  well_name: string;
  requested_curve: string;
  curve: string;
  store: string;
  storage_identity: string;
  operation: string;
  interval: CurveEditInterval;
  parameters: Record<string, unknown>;
  timestamp_utc_ms: number;
  actor: string | null;
  source_note: string | null;
  before_sha256: string;
  after_sha256: string;
}

export function editCurve(req: CurveEditRequest): Promise<CurveEditResult> {
  return invoke<CurveEditResult>("edit_curve", { req });
}

export function restoreCurveValues(
  wellId: string,
  curve: string,
  pointCount: number,
  data: number[],
  restoresEditId: string,
  expectedCurveSha256: string,
  custody: RunCustody,
): Promise<number> {
  return invoke<number>("restore_curve_values", {
    wellId,
    curve,
    pointCount,
    data,
    restoresEditId,
    expectedCurveSha256,
    custody,
  });
}

export function listCurveEditRecords(): Promise<CurveEditRecord[]> {
  return invoke<CurveEditRecord[]>("list_curve_edit_records");
}

/** Engine-copies the project database to `destPath` (live rows only, so the export is
 *  also compacted). The app keeps working on the current file. */
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

/** Why the project on disk could not be opened at launch, and what this session is running on
 *  instead. `null` on a normal launch. Startup happens before the window exists, so a failure
 *  there used to abort the process with no window and no message at all — see `StartupProblem`
 *  in `lib.rs`. Field names are snake_case: they cross the wire from a plain serde struct. */
export interface StartupProblem {
  /** The project we tried, and failed, to open. */
  attempted_path: string;
  /** The underlying error verbatim — it names the real cause. */
  message: string;
  /** The throwaway project actually in use; `""` = memory only. */
  recovered_to: string;
  /** False = nothing will persist anywhere, not even to a recovery file. */
  recovery_persists: boolean;
}

export async function startupProblem(): Promise<StartupProblem | null> {
  return invoke("startup_problem");
}

/** How the background startup open went. Field names are snake_case (serde struct). */
export interface OpenOutcome {
  /** Set only when the intended project could not be opened. */
  problem: StartupProblem | null;
  /** Seconds the open took — used to explain a long wait after the fact. */
  elapsed_secs: number;
  /** The project file actually live. */
  path: string;
}

/** Resolves when the project database is open and installed. The window comes up before the
 *  project does, so NOTHING may query the database until this resolves — until then the
 *  connection is an empty in-memory placeholder. */
export async function awaitProjectOpen(): Promise<OpenOutcome> {
  return invoke("await_project_open");
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

/** Result of "Compact Project": sizes around the rewrite + where the original was parked. */
export interface CompactReport {
  bytes_before: number;
  bytes_after: number;
  old_file: string;
}

/** Rewrites the project file keeping only live rows (drops re-run dead space) and swaps it
 *  in at the same path. The original is kept beside it as `.pre-compact-<ts>.duckdb`. */
export async function compactProject(): Promise<CompactReport> {
  return invoke("compact_project");
}

/** One-time boot/maintenance notices (migration backups, memory caps, compaction results).
 *  Each notice is returned exactly once — record what comes back. */
export async function bootReport(): Promise<string[]> {
  return invoke("boot_report");
}

// ---------------------------------------------------------------------------
// Database inspector (whitelisted tables; cells travel as strings)
// ---------------------------------------------------------------------------

export interface TablePage {
  columns: string[];
  rows: (string | null)[][];
  total_rows: number;
  /** Always false on the inspector path; total_rows is a real COUNT(*). */
  truncated: boolean;
}

export interface QueryPage {
  columns: string[];
  rows: (string | null)[][];
  returned_rows: number;
  /** Explicitly false: returned_rows is a page count, not a true query-result total. */
  count_is_total: false;
  truncated: boolean;
}

export function getTablePage(table: string, wellId: string | null, offset: number, limit: number): Promise<TablePage> {
  return invoke<TablePage>("get_table_page", { table, wellId, offset, limit });
}

export interface IntegrityClassReport {
  class_id: string;
  name: string;
  count: number;
  prunable_count: number;
  can_prune: boolean;
  action: string;
}

export interface IntegrityPruneOffer {
  offered: boolean;
  prunable_findings: number;
  class_ids: string[];
  recovery: string;
}

export interface RecoverableIntegrityPrune {
  batch_id: string;
  created_at: string;
  class_ids: string[];
  pruned_findings: number;
}

export interface IntegrityReport {
  scope: "PROJECT_WIDE";
  wells_touched: number;
  classes: IntegrityClassReport[];
  checked_class_count: number;
  finding_count: number;
  summary: string;
  prune: IntegrityPruneOffer;
  recoverable_prunes: RecoverableIntegrityPrune[];
}

export interface IntegrityPruneClassReceipt {
  class_id: string;
  pruned_findings: number;
}

export interface IntegrityPruneReceipt {
  batch_id: string;
  pruned_findings: number;
  classes: IntegrityPruneClassReceipt[];
}

/** Read-only complete class inventory; never collapses a zero-finding result to bare "clean". */
export function checkReferentialIntegrity(): Promise<IntegrityReport> {
  return invoke<IntegrityReport>("check_referential_integrity");
}

/** Quarantines selected backend-whitelisted orphan classes. No SQL or row payload crosses IPC. */
export function pruneReferentialIntegrity(classIds: string[]): Promise<IntegrityPruneReceipt> {
  return invoke<IntegrityPruneReceipt>("prune_referential_integrity", { classIds });
}

/** Restores one persisted typed-quarantine batch exactly. */
export function restoreReferentialIntegrityPrune(batchId: string): Promise<number> {
  return invoke<number>("restore_referential_integrity_prune", { batchId });
}

/** Reapplies the exact restored batch; refuses if intervening edits changed its identities. */
export function reapplyReferentialIntegrityPrune(batchId: string): Promise<number> {
  return invoke<number>("reapply_referential_integrity_prune", { batchId });
}

export function updateWellField(wellId: string, field: string, value: string | null): Promise<void> {
  return invoke("update_well_field", { wellId, field, value });
}

/** NaN = missing. */
export function updateStandardSample(wellId: string, depth: number, column: string, value: number): Promise<void> {
  return invoke("update_standard_sample", { wellId, depth, column, value });
}

export function updateComputedSample(
  wellId: string,
  depth: number,
  curveName: string,
  value: number,
  custody: RunCustody,
): Promise<void> {
  return invoke("update_computed_sample", { wellId, depth, curveName, value, custody });
}

export function upsertTop(
  wellId: string,
  topName: string,
  depth: number,
  color: string | null,
  scope?: BackendWellScope,
): Promise<void> {
  return invoke("upsert_top", { wellId, topName, depth, color, scope });
}

export function deleteTop(wellId: string, topName: string): Promise<void> {
  return invoke("delete_top", { wellId, topName });
}

/** Read-only SQL over the project database (full DuckDB SQL, SELECT-only). */
export function runQuery(sql: string, limit = 1000): Promise<QueryPage> {
  return invoke<QueryPage>("run_query", { sql, limit });
}

export interface LasOmission {
  curve: string;
  reason: string;
}

export interface LasCurveState {
  export_curve: string;
  source_curve: string;
  set_name: string;
  state: "working" | "final";
}

export interface LasExportResult {
  rows: number;
  curves_written: number;
  curves_held: number;
  omitted: LasOmission[];
  curve_states: LasCurveState[];
  legacy_unrecorded_curves: number;
  precision: SamplePrecisionReport;
  /** Set only after SandiBumi's own LAS reader accepts the completed file. */
  self_checked: boolean;
  /**
   * Set when the index is not uniformly sampled, so `STEP` was written as `0` — LAS 2.0's
   * own declaration for a non-uniform index. Names the depth where the spacing changes and
   * both spacings. `null` means the index is uniform and `STEP` carries its real value.
   */
  nonuniform_step?: string | null;
}

export interface DataExportFormat {
  id: string;
  label: string;
  extension: string;
  is_default: boolean;
  honours_project_sentinel: boolean;
  /** Why a fixed-null format cannot honour the project declaration; shown by a format picker. */
  sentinel_limitation: string | null;
}

/** Lists export formats with their default and sentinel capability declared. */
export function listDataExportFormats(): Promise<DataExportFormat[]> {
  return invoke<DataExportFormat[]>("list_data_export_formats");
}

/** Exports one well as LAS 2.0, including exact held/written counts and named omissions. */
export function exportLas(wellId: string, destPath: string): Promise<LasExportResult> {
  return invoke<LasExportResult>("export_las", { wellId, destPath });
}

/** One water-zone sample that entered the RtC calibration fit. */
export interface RtcFitPoint {
  well_id: string;
  depth: number;
  capbw: number;
  qv: number;
  /** Measured excess conductivity normalized by PHIT·RSF — the regression's y. */
  y: number;
  y_fit: number;
}

/** Fits A_CAP / B_QV / C0 to the user's OWN water leg. */
export interface RtcFitRequest {
  well_ids: string[];
  rt_curve: string;
  phit_curve: string;
  capbw_curve: string;
  qv_curve?: string;
  cec?: number;
  rhog?: number;
  /** Must match the parameters the sw_rtc run will use — they define the clean baseline. */
  rw: number;
  m: number;
  /** Held fixed; the fitted coefficients belong to this RSF only. */
  rsf: number;
  /** The water-bearing interval. At least one of these, or a wet-flag curve, is REQUIRED —
   *  the fit assumes Sw = 1 and refuses to guess where that is true. */
  depth_min?: number | null;
  depth_max?: number | null;
  wet_flag_curve?: string;
}

export interface RtcFitResult {
  a_cap: number;
  b_qv: number;
  c0: number;
  rsf_used: number;
  r2: number;
  rms: number;
  n_points: number;
  n_wells: number;
  /** Wells that actually contributed a sample. Reported separately from `points`, which is
   *  decimated for display — "apply to the wells it was fitted from" must not read a sample. */
  wells_fitted: string[];
  points: RtcFitPoint[];
  /** (reason, count) for every candidate sample not fitted. */
  excluded: [string, number][];
  notes: string[];
  error: string | null;
}

/** Fits the RtC excess-conductivity coefficients to the selected water-bearing interval. */
export function runRtcFit(req: RtcFitRequest, scope: BackendWellScope): Promise<RtcFitResult> {
  return invoke<RtcFitResult>("run_rtc_fit", { req, scope });
}

/** One laboratory CEC plug paired with the clay content of the curves the run will use. */
export interface SFactorPoint {
  well_id: string;
  depth: number;
  /** Depth of the log sample it was paired with — so a suspicious pairing is visible. */
  log_depth: number;
  vkaol: number;
  vill: number;
  /** Theoretical bulk CEC from the clay model — the regression's x. */
  cec_theo: number;
  /** Measured laboratory CEC — the regression's y. */
  cec_lab: number;
  ratio: number;
}

/** Fits sw_imts's CEC scaling factor S to the user's OWN laboratory CEC measurements. */
export interface SFactorFitRequest {
  well_ids: string[];
  /** Point dataset holding the lab CEC ("CEC", or "CORE" for a core-table extra column). */
  cec_dataset: string;
  cec_item: string;
  /** The clay curves the sw_imts RUN will use — not the XRD table the lab CEC came from.
   *  Calibrating against one clay estimate and running against another makes S wrong by the
   *  ratio between them, and both look like clay volumes. */
  vkaol_curve: string;
  vill_curve?: string;
  /** Held fixed; S multiplies these, so the fitted S belongs to them only. */
  cec_kaol?: number;
  cec_ill?: number;
  /** How far a plug may sit from the nearest log sample and still be paired with it. */
  depth_tol?: number;
}

export interface SFactorFitResult {
  s_factor: number;
  s_median_ratio: number;
  ratio_p10: number;
  ratio_p90: number;
  r2: number;
  rms: number;
  n_points: number;
  n_wells: number;
  /** Wells that actually contributed a plug. See `RtcFitResult.wells_fitted`. */
  wells_fitted: string[];
  cec_kaol_used: number;
  cec_ill_used: number;
  points: SFactorPoint[];
  excluded: [string, number][];
  notes: string[];
  error: string | null;
}

/** Fits the IMTS CEC scaling factor S to the selected wells' laboratory CEC measurements. */
export function runSFactorFit(req: SFactorFitRequest, scope: BackendWellScope): Promise<SFactorFitResult> {
  return invoke<SFactorFitResult>("run_s_factor_fit", { req, scope });
}

/** One measurement name inside a point dataset, with what is actually stored under it. */
export interface AuxItemInfo {
  dataset: string;
  item: string;
  /** Wells carrying it in their ACTIVE delivery. */
  wells: number;
  rows: number;
  /** Rows whose value is a NUMBER — a descriptive item has none and cannot be fitted against. */
  numeric_rows: number;
}

/** Every measurement name in the project's point data, from the ACTIVE delivery of each dataset.
 *  Project-wide by design: one grouped scan beats N round trips or an IN-list long enough to hit
 *  a binding limit. Lets a dialog offer what exists instead of asking for a typed name. */
export function listAuxItemCatalog(): Promise<AuxItemInfo[]> {
  return invoke<AuxItemInfo[]>("list_aux_item_catalog");
}

/** What the Python equation engine can offer, probed once per session. */
export interface PythonStatus {
  /** Interpreter the engine will use; null when no Python with numpy was found. */
  path: string | null;
  /** scipy version when importable in that interpreter; null when scipy is absent.
   *  scipy is OPTIONAL — numpy alone is a fully working engine. */
  scipy: string | null;
  /** Manifest-derived equation capability status/remediation. */
  message: string;
  /** Manifest-derived optional SciPy remediation. */
  scipy_message: string;
}

/** Interpreter + optional-package status for the equation engine. */
export function pythonStatus(): Promise<PythonStatus> {
  return invoke<PythonStatus>("python_status");
}

export interface CapabilityPackageRequirement {
  distribution: string;
  import_name: string;
  required: boolean;
  /** Null until a qualified release lock supplies the supported version. */
  minimum_supported_version: string | null;
  version_source: string;
}

export interface CapabilitySupport {
  id: string;
  display_name: string;
  owning_domain: string;
  packages: CapabilityPackageRequirement[];
  /** False = known unavailable; null = the interpreter exists but probing is pending. */
  available: boolean | null;
  reason: string;
  package_status: Array<{
    distribution: string;
    import_name: string;
    available: boolean;
    version: string | null;
    error: string | null;
  }>;
}

export interface InstallationSupport {
  manifest_schema_version: number;
  interpreter_minimum_version: string;
  selected_interpreter: string | null;
  selected_interpreter_rule: string | null;
  interpreter_candidates: Array<{
    candidate: string;
    precedence_rule: string;
    resolved_executable: string | null;
    accepted: boolean;
    reason: string;
  }>;
  capabilities: CapabilitySupport[];
}

/** Capability-level prerequisite status generated from the bundled manifest. */
export function installationSupport(): Promise<InstallationSupport> {
  return invoke<InstallationSupport>("installation_support");
}

// ---------------------------------------------------------------------------
// Phase 6: generic curve store (curve_meta/curve_samples), deviation surveys,
// DLIS import. The generic store holds ANY curve (PEF, CALI, multiple runs, ...)
// per well across RAW/EDIT/FINAL sets, unlike the fixed-6 standard_curves.
// ---------------------------------------------------------------------------

export interface GenericCurveInventoryEntry {
  curve_id: string;
  mnemonic: string;
  unit: string | null;
  family: string | null;
  set_name: string;
  source: string | null;
  run_no: number | null;
  /** Version of this stored curve-set identity; changes when resolving metadata changes. */
  set_version: number;
  /** Curve-level Final designation used by the declared resolver. */
  final_flag: boolean;
  /** Monotonic metadata revision; null only on pre-SB-DBM-006 history. */
  modified_seq: number | null;
  /** True when the user promoted this curve to win its (well, set, mnemonic) group. */
  pinned: boolean;
}

export interface GenericCurveCatalogEntry extends GenericCurveInventoryEntry {
  /** Every stored row, including missing values. */
  n_samples: number;
  /** Finite values included in min/max/mean. */
  n_valid: number;
  n_missing: number;
  min: number | null;
  max: number | null;
  mean: number | null;
}

/** Every curve in the generic store for one well, across RAW/EDIT/FINAL sets. */
export function listGenericCurveCatalog(wellId: string): Promise<GenericCurveCatalogEntry[]> {
  return invoke<GenericCurveCatalogEntry[]>("list_generic_curve_catalog", { wellId });
}

/** Metadata-only curve list for trees and set pickers; never scans sample rows. */
export function listGenericCurveInventory(wellId: string): Promise<GenericCurveInventoryEntry[]> {
  return invoke<GenericCurveInventoryEntry[]>("list_generic_curve_inventory", { wellId });
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

/** Changes the curve-level Final designation and returns the previously Final identity, if any. */
export function setGenericCurveFinal(curveId: string, isFinal: boolean): Promise<string | null> {
  return invoke<string | null>("set_generic_curve_final", { curveId, isFinal });
}

/** A curve's editable identity — returned by `updateCurveMeta` as it was BEFORE the edit,
 *  which is exactly what an undo needs. */
export interface CurveMetaEdit {
  mnemonic: string;
  unit: string | null;
  family: string | null;
}

/** Renames / re-units / re-families one imported curve. Metadata only — no sample changes —
 *  but modules resolve their inputs by mnemonic and family, so this repoints what they read.
 *  Returns the previous values so the caller can push an undo. */
export function updateCurveMeta(
  curveId: string,
  mnemonic: string,
  unit: string | null,
  family: string | null,
  operator: { identity: string; kind: AncestryActorKind },
  view?: string,
): Promise<CurveMetaEdit> {
  return invoke<CurveMetaEdit>("update_curve_meta", {
    curveId,
    mnemonic,
    unit,
    family,
    operator: operator.identity,
    operatorKind: operator.kind,
    view: view ?? null,
  });
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
 *  `datumElevation` (KB above MSL) is used for TVDSS; null falls back to the well's KB.
 *  `surveyName` versions the survey — a second import lands BESIDE the first (auto-suffixed
 *  if the name is taken) and becomes the active one, never overwriting it. */
export function importDeviationCsv(
  wellId: string,
  path: string,
  datumElevation: number | null,
  surveyName: string | null = null,
): Promise<CoreImportResult> {
  return invoke<CoreImportResult>("import_deviation_csv", { wellId, path, datumElevation, surveyName });
}

/** One core delivery of a well (T-IMP-08). Exactly one is `active`; every core reader —
 *  log overlay, φ-k clouds, SandiMin calibration, DB Inspector edits — follows it. */
export interface CoreSetInfo {
  set_name: string;
  rows: number;
  active: boolean;
  source: string | null;
  imported_at: string | null;
}

/** One deviation survey of a well (T-IMP-12). The active one drives TVD/TVDSS. */
export interface SurveyInfo {
  survey_name: string;
  stations: number;
  active: boolean;
  source: string | null;
  datum: number | null;
  imported_at: string | null;
}

export function listCoreSets(wellId: string): Promise<CoreSetInfo[]> {
  return invoke<CoreSetInfo[]>("list_core_sets", { wellId });
}

export function listSurveys(wellId: string): Promise<SurveyInfo[]> {
  return invoke<SurveyInfo[]>("list_surveys", { wellId });
}

export function setActiveCoreSet(wellId: string, setName: string): Promise<void> {
  return invoke<void>("set_active_core_set", { wellId, setName });
}

export function deleteCoreSet(wellId: string, setName: string): Promise<number> {
  return invoke<number>("delete_core_set", { wellId, setName });
}

/** Activates a survey AND rebuilds TVD/TVDSS from it; returns the samples rewritten. */
export function setActiveSurvey(wellId: string, surveyName: string): Promise<number> {
  return invoke<number>("set_active_survey", { wellId, surveyName });
}

export function deleteSurvey(wellId: string, surveyName: string): Promise<number> {
  return invoke<number>("delete_survey", { wellId, surveyName });
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
export function materializeTvd(
  scope: BackendWellScope = { kind: "active_group" },
): Promise<TvdMaterializeResult[]> {
  return invoke<TvdMaterializeResult[]>("materialize_tvd", { scope });
}

export interface DlisImportResult {
  path: string;
  status: "complete" | "partial" | "failed";
  /** Payload channels named by the file, excluding frame index channels. */
  channels_declared: number;
  curves_imported: number;
  rows: number;
  /** Legacy field; always zero now that duplicate curves require keep-separate or skip. */
  replaced: number;
  notes: string[];
  /** Every automatic value conversion performed by the importer. */
  unit_conversions: ImportResult["unit_conversions"];
  unconverted_units: ImportResult["unconverted_units"];
  unit_designations: ImportResult["unit_designations"];
  skipped: Array<{ kind: string; name: string; count: number; rule: string; omitted: boolean }>;
  /** Exact channel mnemonics whose LAS-sentinel fallback was disabled for this import. */
  sentinel_exceptions: string[];
  well_mappings: DlisWellMapping[];
  mapping_confirmation_required: boolean;
  interval_conflicts: Array<{
    scope: "well" | "set";
    name: string;
    declared_top: number;
    declared_base: number;
    incoming_top: number;
    incoming_base: number;
  }>;
  duplicate_conflicts: DlisDuplicateConflict[];
  duplicate_decisions: DlisDuplicateDecisionRecord[];
  error: string | null;
}

export type DlisDuplicateAction = "keep_separate" | "skip_incoming";

export interface DlisDuplicateDecision {
  mnemonic: string;
  run: number;
  action: DlisDuplicateAction;
}

export interface DlisDuplicateConflict {
  mnemonic: string;
  run: number;
  existing: string[];
}

export interface DlisDuplicateDecisionRecord extends DlisDuplicateDecision {
  existing: string[];
  target_set: string | null;
}

export interface DlisWellMapping {
  source_well: string;
  logical_files: number[];
  target_well_name: string;
  /** Null in the pre-commit proposal; populated only after the project well is created. */
  target_well_id: string | null;
  will_create: boolean;
}

/** Imports scalar DLIS channels through the Python subprocess. A single source well targets
 *  `wellId`; a multi-well container may omit it and requires its returned mapping to be echoed. */
export function importDlisFile(
  wellId: string | null,
  path: string,
  setName?: string | null,
  fileDepthUnit?: "M" | "FT" | null,
  msPerFtMeaning?: "microseconds_per_foot" | "millisiemens_per_foot" | null,
  undeclaredDrhoUnit?: "g/cc" | "kg/m3" | null,
  outsideIntervalDecision?: "accept_outside_declared_interval" | null,
  duplicateDecisions?: DlisDuplicateDecision[] | null,
  lasSentinelExceptions?: string[] | null,
  confirmedWellMappings?: DlisWellMapping[] | null,
): Promise<DlisImportResult> {
  return invoke<DlisImportResult>("import_dlis_file", {
    wellId,
    path,
    setName: setName ?? null,
    fileDepthUnit: fileDepthUnit ?? null,
    msPerFtMeaning: msPerFtMeaning ?? null,
    undeclaredDrhoUnit: undeclaredDrhoUnit ?? null,
    outsideIntervalDecision: outsideIntervalDecision ?? null,
    duplicateDecisions: duplicateDecisions ?? null,
    lasSentinelExceptions: lasSentinelExceptions ?? null,
    confirmedWellMappings: confirmedWellMappings ?? null,
  });
}

// --- Petrography: pore area from blue-dyed epoxy (plan_image_analysis A1) ---

/** The colour rule, in HSV. Hue in degrees, saturation and value 0..1.
 *  The backend's defaults are a plain blue band offered as a STARTING POINT for a visual tuning
 *  task — not a calibration. Tune them against the preview, which is drawn by the same code that
 *  does the measuring. */
export interface PoreColorBand {
  hue_lo: number;
  hue_hi: number;
  sat_min: number;
  val_min: number;
}

/** One depth interval and the plate every section inside it is corrected onto. */
export interface ReferenceZone {
  /** Shallowest depth this reference serves. Omit to reach up to the top of the well. */
  top?: number | null;
  /** Deepest. Omit to reach down to total depth. */
  base?: number | null;
  image_id: string;
}

export interface PoreSpec {
  well_id: string;
  dataset: string;
  band?: PoreColorBand;
  /** Draw the mask over this plate and send the picture back. */
  preview_image_id?: string | null;
  /** Measure only this plate — so moving a slider does not re-measure the whole delivery. */
  only_image_id?: string | null;
  /** The plate the band was tuned on. Every other plate is colour-corrected onto it before the
   *  band is applied, which is what lets one band serve a delivery photographed under more than
   *  one light. Omit to read every plate exactly as delivered. Naming one also turns on the
   *  empty-measurement refusal — see `band_missed`. */
  reference_image_id?: string | null;
  /** References for particular depth intervals, overruling `reference_image_id` where they reach.
   *  Omit for the delivery-wide behaviour.
   *
   *  A delivery spanning two cored intervals is two different rocks, usually photographed on two
   *  different days, and one reference serves both only by accident. A plate no interval covers
   *  falls back to `reference_image_id`; where that is absent too the plate is REFUSED by name
   *  rather than read as delivered — one stored set holding both corrected and uncorrected
   *  fractions would be two measurements under one name. */
  reference_zones?: ReferenceZone[];
  /** Store the results as point data under this delivery name. Omit to measure without writing:
   *  tuning must not leave a trail of half-judged answers in the project. */
  set_name?: string | null;
  /** Also measure each individual pore. Needs scipy. */
  geometry?: boolean;
  /** Smallest thing counted as a pore, in PIXELS — a statement about what the picture can
   *  resolve, which has to mean the same thing on a plate carrying no scale. */
  min_pore_px?: number;
  /** Also outline each GRAIN and measure its size. Needs scipy, and inherits the blue-epoxy
   *  refusal: the grain phase is whatever the pore rule did not claim. */
  grains?: boolean;
  /** Smallest thing counted as a grain, in PIXELS. */
  min_grain_px?: number;
  /** How far apart two grain centres must be before they count as two grains, in PIXELS. The knob
   *  for over-segmentation — judged against the preview, not from the table. */
  grain_sep_px?: number;
  /** Also report the Wicksell-corrected sizes beside the apparent ones. Off by default; the two
   *  are stored under DIFFERENT item names so neither can be mistaken for the other. */
  wicksell?: boolean;
  /** Read the stain too. Omit for no stain — a stain assumed is a mineral fraction invented. */
  stain?: StainSpec | null;
  /** Score this run against an independent measurement of the same plugs — usually the core
   *  porosity the laboratory measured on the plug each section was cut from. Omit to skip.
   *
   *  `reference_image_id` turned out to be a bigger lever on the answer than the colour band is,
   *  and until now the dialog offered nothing to tell a good choice from a bad one except the
   *  preview. A setting judged by eye against a picture is judged on how the picture looks; this
   *  is the number that says whether it also tracks the rock. */
  check_against?: PlugSource | null;
  /** Two measurements further apart than this are not the same plug. Omit for the standard
   *  6-inch sample the rest of the app pairs on. */
  check_depth_tol?: number;
}

/** An HSV window. Richer than PoreColorBand because a stain scheme has to be able to say
 *  UNSTAINED — dolomite under alizarin red S is identified by staying colourless, which is a
 *  saturation ceiling and cannot be written as a floor. */
export interface StainBand {
  hue_lo: number;
  hue_hi: number;
  sat_min: number;
  sat_max: number;
  val_min: number;
  val_max: number;
}

export interface StainClass {
  mineral: string;
  band: StainBand;
}

export interface StainSpec {
  /** Matched against each plate's OWN declared stain; a plate that disagrees is refused by name. */
  stain: string;
  /** Tested in order, first match wins — a pixel is one mineral. */
  classes: StainClass[];
}

/** Mineral area fractions on one plate, as fractions of the WHOLE plate: pore + minerals +
 *  unclassified is 1. */
export interface PlateStain {
  fractions: [string, number][];
  /** Solid that fell in no band. The honesty number — a section where a third of the rock matched
   *  nothing has not been given a mineralogy, whatever the other rows say. */
  unclassified: number;
}

/** The published stain schemes this build ships, as [name, classes]. Mineral identifications are
 *  standard carbonate petrography (Friedman 1959, Dickson 1966); the colour bands are round
 *  starting points for visual tuning, like the epoxy band. */
export function stainSchemes(): Promise<[string, StainClass[]][]> {
  return invoke<[string, StainClass[]][]>("stain_schemes");
}

// ---------------------------------------------------------------------------
// A3 — the trained mineral classifier
// ---------------------------------------------------------------------------

/** One point the user clicked, and what they called it. Position is a FRACTION of the picture,
 *  never a pixel: the stored copy is resampled, so a pixel coordinate belongs to whichever copy it
 *  was taken on and nothing in the number says which. */
export interface PlateLabel {
  image_id: string;
  x: number;
  y: number;
  mineral: string;
}

export interface ClassifySpec {
  well_id: string;
  dataset: string;
  labels: PlateLabel[];
  patch_px?: number;
  set_name?: string | null;
  preview_image_id?: string | null;
}

export interface ClassPerf {
  mineral: string;
  /** Fraction of held-out clicks the model got right. **−1 means it could not be checked.** A low
   *  recall means that mineral's fraction is noise. */
  recall: number;
  clicks: number;
}

export interface PlateClasses {
  image_id: string;
  name: string;
  depth_top: number;
  depth_base: number | null;
  fractions: [string, number][];
}

export interface ClassifyResult {
  plates: PlateClasses[];
  /** Overall held-out accuracy, cross-validated BY CLICK. */
  accuracy: number;
  per_class: ClassPerf[];
  skipped: string[];
  preview_png: string | null;
  preview_width: number;
  preview_height: number;
  written: [string, string] | null;
  notes: string[];
}

/** Is scikit-learn reachable? Probed so the dialog can say what is missing before a run. */
export function classifySupport(): Promise<boolean> {
  return invoke<boolean>("classify_support");
}

/** Trains a per-pixel mineral classifier on the user's own clicks and applies it to the delivery.
 *  There is no shipped model: quartz against feldspar in plane light is not a colour problem, and
 *  a model trained on somebody else's sections under somebody else's lamp would produce numbers
 *  with the shape of a modal analysis and none of the content. */
export function runPlateClassifier(spec: ClassifySpec): Promise<ClassifyResult> {
  return invoke<ClassifyResult>("run_plate_classifier", { spec });
}

export interface PlatePore {
  image_id: string;
  name: string;
  depth_top: number;
  depth_base: number | null;
  /** Pore area as a fraction of the plate, v/v. */
  pore_fraction: number;
  /** The plate's own median hue in degrees — what colour this picture mostly is. */
  scene_hue: number;
  /** True when that median hue falls inside the declared pore band, so the band is matching the
   *  background rather than the pores. The fraction is still shown — tuning the band is how it
   *  gets fixed — but the plate is left out of the write. */
  scene_dominated: boolean;
  /** How far this photograph's light sat from the reference plate's, in degrees of hue — the size
   *  of the correction applied. NaN when no reference was named. Diagnostic, never a threshold. */
  cast_shift: number;
  /** The plate this one was corrected onto, by name; empty when nothing was. Reported because with
   *  more than one reference in play, a shift of 40° means nothing until you know which plate it is
   *  40° from. */
  reference_name: string;
  /** True when the band, carried onto this plate, claimed less than one resolvable pore. Only ever
   *  set on a normalized run: near zero reads as a tight rock, so it is refused rather than stored,
   *  but only once a reference plate has established that the band finds epoxy somewhere. */
  band_missed: boolean;
  pixels: number;
  geometry?: PoreGeometry;
  grains?: GrainStats;
  stain?: PlateStain;
}

/** Size of the individual grains on one plate.
 *
 *  Everything dimensional here is APPARENT unless its name says `_w`. A random plane rarely cuts a
 *  grain through its centre, so section diameters run small and section sorting runs worse than
 *  the rock's — which is why the apparent and corrected numbers are stored under different item
 *  names rather than one name and a flag. */
export interface GrainStats {
  n: number;
  n_edge: number;
  n_small: number;
  aspect_p50: number;
  /** Median fraction of a grain's outline that is a contact with another grain rather than open
   *  pore. The honesty number: where the rock is cemented there is nothing in the picture to
   *  separate two grains, and the boundary was placed rather than seen. */
  contact_p50: number;
  d10_app_um: number | null;
  d50_app_um: number | null;
  d90_app_um: number | null;
  /** Folk & Ward inclusive graphic standard deviation, phi units. Needs a scale, like the
   *  diameters — phi is a logarithm of millimetres. */
  sort_app_phi: number | null;
  d10_w_um: number | null;
  d50_w_um: number | null;
  d90_w_um: number | null;
  sort_w_phi: number | null;
  /** Saltykov classes that unfolded to a negative population and were clamped. Several of them
   *  means the correction is unstable on that plate. */
  w_clamped: number;
}

/** Shape and size of the individual pores on one plate. */
export interface PoreGeometry {
  /** Pores measured: big enough, and not cut by the frame. */
  n: number;
  /** Dropped for touching the plate edge — their true size is unknown, so including them would
   *  bias the distribution small. */
  n_edge: number;
  /** Dropped as too small to be anything but speckle. */
  n_small: number;
  aspect_p50: number;
  aspect_p90: number;
  /** Median circularity, 4·pi·A/P². 1 is a circle. */
  shape_p50: number;
  /** Equivalent-circle diameter in MICROMETRES, area-weighted. null on a plate with no declared
   *  scale — a diameter in pixels is not a diameter. */
  d10_um: number | null;
  d50_um: number | null;
  d90_um: number | null;
}

export interface PoreResult {
  plates: PlatePore[];
  /** Plates left out and why, one entry each — never a silent subset. */
  skipped: string[];
  preview_png: string | null;
  /** The same plate at the same size WITHOUT the mask — what the eyedropper reads and what Hold to
   *  compare shows. Sent with the overlay so the two can never be one plate's mask over another
   *  plate's pixels. */
  plain_png?: string | null;
  preview_width: number;
  preview_height: number;
  /** [dataset, delivery] written, when a set name was given. */
  written: [string, string] | null;
  /** How the STORABLE plates agreed with an independent plug measurement, when one was named.
   *  Computed whether or not the run was saved, so a setting can be judged before it is kept. */
  agreement?: Agreement | null;
  notes: string[];
}

/** How a run agrees with a measurement this app did not produce. */
export interface Agreement {
  reference_label: string;
  /** Plugs carrying BOTH — not the number of plates measured. Two runs that refused different
   *  plates are scored on different rock, so this has to be read beside every coefficient. */
  n_pairs: number;
  /** Measurements on either side that found no partner inside the tolerance. */
  n_unpaired: number;
  /** Straight-line agreement — the right question when both axes are porosity. */
  pearson: number;
  /** Rank agreement: does it order the plugs the way the laboratory does. The number to choose a
   *  setting on, because it survives the systematic offset a section-versus-plug comparison always
   *  carries. */
  spearman: number;
  measured_median: number;
  reference_median: number;
  notes: string[];
}

/** Is numpy + Pillow reachable? Probed once so the dialog can say what is missing before a run. */
export function poreSupport(): Promise<boolean> {
  return invoke<boolean>("pore_support");
}

/** Measures pore area on a well's live image delivery. Refuses any plate not declared as
 *  blue-dyed epoxy, by name. */
export function runPoreArea(spec: PoreSpec): Promise<PoreResult> {
  return invoke<PoreResult>("run_pore_area", { spec });
}

// ---------------------------------------------------------------------------
// Plug QC — two measurements of the same plug, plotted against each other
// ---------------------------------------------------------------------------

/** Where one axis of the comparison comes from. */
export interface PlugSource {
  /** "core" | "aux" | "scal_throat" */
  kind: string;
  /** aux only — the point dataset, whose ACTIVE delivery is read. */
  dataset?: string;
  /** core: CPOR/CPERM/CGD/CSW. aux: the measurement name. */
  item?: string;
  /** scal_throat only — the mercury saturation the radius is quoted at (0.35 = the r35 convention). */
  saturation?: number;
}

export interface PlugChoice {
  kind: string;
  dataset: string;
  item: string;
  label: string;
  n: number;
  wells: number;
}

export interface PlugQcRequest {
  well_ids: string[];
  x: PlugSource;
  y: PlugSource;
  depth_tol: number;
}

export interface PlugQcPoint {
  well_id: string;
  x: number;
  y: number;
  x_depth: number;
  y_depth: number;
}

export interface PlugQcResult {
  points: PlugQcPoint[];
  /** Pairs found — NOT points.length once the cloud is decimated for the wire. */
  n_pairs: number;
  n_wells: number;
  pearson: number;
  spearman: number;
  x_label: string;
  y_label: string;
  x_median: number;
  y_median: number;
  excluded: [string, number][];
  notes: string[];
}

/** What the two axis pickers can offer over the wells in scope. */
export function listPlugChoices(scope: BackendWellScope): Promise<PlugChoice[]> {
  return invoke<PlugChoice[]>("list_plug_choices", { scope });
}

/** Pairs two plug-scale measurements by depth across the scoped wells. */
export function runPlugQc(req: PlugQcRequest, scope: BackendWellScope): Promise<PlugQcResult> {
  return invoke<PlugQcResult>("run_plug_qc", { req, scope });
}

// ---------------------------------------------------------------------------
// Statistics (statistics.rs) — the table-producing family. Every one is a pure read.
// ---------------------------------------------------------------------------

export interface CurveStatsRequest {
  well_ids: string[];
  curves: string[];
  input_set?: string;
  by_zone?: boolean;
  /** Percentiles to report (0–100). Empty falls back to P10/P50/P90. */
  percentiles?: number[];
  mask_curve?: string | null;
}

export interface CurveStatsRow {
  well: string;
  zone: string;
  curve: string;
  n: number;
  /** Samples inside the interval with no value — a mean over 12 of 400 is not the zone's mean. */
  n_missing: number;
  min: number | null;
  max: number | null;
  mean: number | null;
  /** Geometric and harmonic means, beside the arithmetic one. Null where any sample is
   *  non-positive — a geometric mean over "the samples that had a logarithm" describes a
   *  different set from the arithmetic mean next to it, and the two get read straight across. */
  mean_geom: number | null;
  mean_harm: number | null;
  std: number | null;
  percentiles: (number | null)[];
}

/** Returns the rows and the percentile list actually used, so a table labels its own columns. */
export async function statsCurveSummary(
  req: CurveStatsRequest,
  scope: BackendWellScope,
): Promise<[CurveStatsRow[], number[]]> {
  return invoke<[CurveStatsRow[], number[]]>("stats_curve_summary", { req, scope });
}

export interface PairStatsRequest {
  well_ids: string[];
  x_curve: string;
  y_curve: string;
  input_set?: string;
  by_zone?: boolean;
  mask_curve?: string | null;
}

export interface PairStatsRow {
  well: string;
  zone: string;
  n: number;
  pearson: number | null;
  spearman: number | null;
  bias: number | null;
  rms_diff: number | null;
  slope: number | null;
  intercept: number | null;
}

export async function statsPairSummary(req: PairStatsRequest, scope: BackendWellScope): Promise<PairStatsRow[]> {
  return invoke<PairStatsRow[]>("stats_pair_summary", { req, scope });
}

export interface VersusRequest {
  well_ids: string[];
  curves: string[];
  /** The reference version. */
  set_a: string;
  /** The version under test; omit for the current values. */
  set_b?: string;
}

export interface VersusRow {
  well: string;
  curve: string;
  n_common: number;
  only_a: number;
  only_b: number;
  n_changed: number;
  mean_diff: number | null;
  max_abs_diff: number | null;
}

export async function statsVersusSets(req: VersusRequest, scope: BackendWellScope): Promise<VersusRow[]> {
  return invoke<VersusRow[]>("stats_versus_sets", { req, scope });
}

export interface ThicknessCondition {
  curve: string;
  op: ">=" | "<=" | ">" | "<" | "==";
  value: number;
}

export interface ThicknessRequest {
  well_ids: string[];
  mode: "FLAG" | "CLASS" | "CUTOFF" | "MARKER";
  input_set?: string;
  curve?: string | null;
  conditions?: ThicknessCondition[];
  by_zone?: boolean;
}

export interface ThicknessRow {
  well: string;
  zone: string;
  item: string;
  n: number;
  gross_md: number;
  net_md: number;
  /** Blank where the well has no TVD curve — never a copy of the measured value. */
  gross_tvd: number | null;
  net_tvd: number | null;
  ntg: number | null;
}

export async function statsThickness(req: ThicknessRequest, scope: BackendWellScope): Promise<ThicknessRow[]> {
  return invoke<ThicknessRow[]>("stats_thickness", { req, scope });
}

export interface FitRequest {
  well_ids: string[];
  predictors: string[];
  target: string;
  input_set?: string;
  log_target?: boolean;
  log_predictors?: boolean;
  mask_curve?: string | null;
}

export interface FitResult {
  /** Intercept first, then one per predictor in the order given. */
  coefficients: number[];
  predictors: string[];
  n: number;
  r2: number;
  rms: number;
  /** Leave-one-WELL-out R² — the number to quote. Null with fewer than three wells. */
  r2_blind: number | null;
  wells_used: string[];
  notes: string[];
}

export async function statsFit(req: FitRequest, scope: BackendWellScope): Promise<FitResult> {
  return invoke<FitResult>("stats_fit", { req, scope });
}

// ---------------------------------------------------------------------------
// Intake (intake.rs) — one importer for any delimited text.
// ---------------------------------------------------------------------------

export interface TableOptions {
  /** "," ";" "\t" "ws" — omit to auto-detect. */
  delimiter?: string;
  /** Lines to skip before the header (a title block). */
  skip_lines?: number;
  /** "dot" | "comma" — omit to decide per token. */
  decimal?: string;
}

export type IntakeRole =
  | "WELL" | "DEPTH" | "DEPTH_BASE" | "CPOR" | "CPERM" | "CGD" | "CSW" | "ITEM" | "CURVE" | "IGNORE";

export interface IntakeColumn {
  header: string;
  /** "number" | "text" | "empty" */
  kind: string;
  role: IntakeRole;
  /** Why that role was proposed — a guess nobody can argue with is a guess that gets accepted. */
  reason: string;
  filled: number;
}

export interface FormatDetection {
  detected_format: string;
  recognition: string;
  choice_report: string;
  extension_disagreement: string | null;
}

export interface IntakeProbe {
  path: string;
  format: FormatDetection;
  text_encoding: string;
  columns: IntakeColumn[];
  n_rows: number;
  preview: string[][];
  delimiter: string;
  units_row_skipped: boolean;
  depth_unit_guess: string | null;
  decimal: string;
  /** [row, column] of preview cells in a NUMBER column that did not parse — painted in the grid. */
  preview_bad: [number, number][];
  ambiguous_numbers: number;
  notes: string[];
}

/** Writes pasted text to a temp file and returns its path, so a paste and a file take the
 *  identical parse and commit path. */
export async function intakePaste(text: string): Promise<string> {
  return invoke<string>("intake_paste", { text });
}

export async function intakeProbe(path: string, opts: TableOptions): Promise<IntakeProbe> {
  return invoke<IntakeProbe>("intake_probe", { path, opts });
}

export interface IntakeCommit {
  paths: string[];
  roles: string[];
  depth_unit?: string;
  set_name?: string;
  extras_dataset?: string;
  fallback_well_id?: string;
  follow_core?: boolean;
  /** SB-DBM-031: the datum the delivery's depths are quoted in, declared in the pane. */
  depth_datum: string;
}

export async function intakeCommit(req: IntakeCommit): Promise<CoreTableImportResult[]> {
  return invoke<CoreTableImportResult[]>("intake_commit", { req });
}

// ---------------------------------------------------------------------------
// Reframe — resampling a log set onto a different sampling as a new set
// ---------------------------------------------------------------------------

/** One output sample whose categorical resampling support spans unlike source codes. This is
 * sparse reporting metadata, not a curve array; full depth/value arrays stay on the bytemuck IPC
 * path. */
export interface CategoryBoundaryCrossing {
  output_depth: number;
  source_start_depth: number;
  source_end_depth: number;
  from_code: number;
  to_code: number;
}

/** One curve carried onto the new frame, with the averaging that was actually used. */
export interface ReframeCurve {
  name: string;
  method: string;
  samples_in: number;
  samples_out: number;
  category_boundary_crossings: CategoryBoundaryCrossing[];
}

export interface ReframeResult {
  well_id: string;
  well_name: string;
  /** Median spacing of the source — the number the decision turns on, and one nothing else shows. */
  source_step: number;
  target_step: number;
  depth_top: number;
  depth_base: number;
  rows: number;
  curves: ReframeCurve[];
  version: number | null;
  notes: string[];
  error: string | null;
}

export interface ReframeSourceSpec {
  kind: "logset" | "import" | "standard";
  name: string | null;
}

export type ReframeMethod =
  | "Mean"
  | "Geometric"
  | "Harmonic"
  | "Median"
  | "Interpolate"
  | "Nearest"
  | "Mode"
  | "Auto";

export interface ReframeRequest {
  well_ids: string[];
  source: ReframeSourceSpec;
  selection_name: string;
  substitutions: Array<{ requested: string; substitute: string; accepted: boolean }>;
  target: {
    kind: "step" | "regularize" | "match_well" | "match_set";
    step: number | null;
    align: boolean;
    well_id: string | null;
    set_name: string | null;
    top: number | null;
    base: number | null;
  };
  methods: Record<string, ReframeMethod>;
  default_method: ReframeMethod;
  output_set: string;
  preview: boolean;
  /** Required only when `preview` is false and a new set is written. */
  custody?: RunCustody | null;
}

export interface CurveSelection {
  name: string;
  mode: "selected";
  /** Ordered exact mnemonics; this order survives save/reload. */
  members: string[];
}

export function saveCurveSelection(selection: CurveSelection): Promise<CurveSelection> {
  return invoke<CurveSelection>("save_curve_selection", { selection });
}

export function listCurveSelections(): Promise<CurveSelection[]> {
  return invoke<CurveSelection[]>("list_curve_selections");
}

export function deleteCurveSelection(name: string): Promise<void> {
  return invoke<void>("delete_curve_selection", { name });
}

/** Exact source mnemonics offered for an explicit substitution; never family/type-expanded. */
export function reframeSourceCurves(wellId: string, source: ReframeSourceSpec): Promise<string[]> {
  return invoke<string[]>("reframe_source_curves", { wellId, source });
}

/** Resamples a set onto a different sampling as a NEW set. `preview: true` reports without writing. */
export function runReframe(req: ReframeRequest, scope: BackendWellScope): Promise<ReframeResult[]> {
  return invoke<ReframeResult[]>("run_reframe", { req, scope });
}

// ---------------------------------------------------------------------------
// Intake — WIDE / BLOCK array layouts
// ---------------------------------------------------------------------------

/** What one wide/block import wrote. */
export interface ArrayImportResult {
  path: string;
  curve: string;
  wells: number;
  samples: number;
  bins: number;
  /** The two ends of the axis read off the header row. */
  axis_first: number;
  axis_last: number;
  /** Sets actually written — the suffixed name where the chosen one was already taken. */
  sets: string[];
  unmatched: string[];
  notes: string[];
  error: string | null;
}

export interface ArrayCommitRequest {
  paths: string[];
  roles: string[];
  /** `"wide"` or `"block"`. Declared by the user, never sniffed. */
  layout: string;
  curve_name: string;
  set_name?: string;
  depth_unit?: string;
  fallback_well_id?: string;
}

/** Imports a WIDE or BLOCK table into the array store, with the header row as its axis. */
export function intakeCommitArrays(req: ArrayCommitRequest): Promise<ArrayImportResult[]> {
  return invoke<ArrayImportResult[]>("intake_commit_arrays", { req });
}

/**
 * One depth that more than one sample landed on. The array store holds ONE vector per depth, so
 * the extras would be refused — `well` is null where the file has no WELL column and the whole
 * delivery falls back to the selected well.
 */
export interface DepthClash {
  well: string | null;
  depth: number;
}

/** One sample of a wide/block table: where it sits, and its values across the axis. */
export interface ArrayPreviewRow {
  well_name: string | null;
  depth: number | null;
  sample_no: number | null;
  values: number[];
}

/** What a wide/block file says, read for the pane without writing anything. */
export interface ArrayPreview {
  axis: number[];
  /** The header TEXT each axis value was read from — `100 psi` reading as 100 is worth seeing. */
  axis_labels: string[];
  /** Headers that are not numbers, so cannot be axis values. Named, never silently dropped. */
  non_axis: string[];
  blocks_joined: number;
  clashes: DepthClash[];
  notes: string[];
  /** Samples the FILE holds, not the count drawn below. */
  n_rows: number;
  rows: ArrayPreviewRow[];
  /** Where each drawn row sits in the file — a duplicate pulled in from beyond the cap says so. */
  row_index: number[];
}

/**
 * Reads a WIDE or BLOCK table for the pane, writing nothing. The same reader the import runs, so
 * the preview cannot disagree with the import about what the file says.
 */
export function intakeProbeArrays(
  path: string,
  opts: TableOptions,
  roles: string[],
  block: boolean,
): Promise<ArrayPreview> {
  return invoke<ArrayPreview>("intake_probe_arrays", { path, opts, roles, block });
}

export interface CurveCommitRequest {
  paths: string[];
  roles: string[];
  set_name?: string;
  depth_unit?: string;
  fallback_well_id?: string;
}

export interface CurveImportResult {
  path: string;
  wells: number;
  curves: string[];
  samples: number;
  sets: string[];
  unmatched: string[];
  notes: string[];
  error: string | null;
}

/** Imports columns marked CURVE as continuous logs into the generic curve store. */
export function intakeCommitCurves(req: CurveCommitRequest): Promise<CurveImportResult[]> {
  return invoke<CurveImportResult[]>("intake_commit_curves", { req });
}
