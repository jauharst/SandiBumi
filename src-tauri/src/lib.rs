mod chain;
mod composite;
mod curves;
mod db;
mod decimate;
mod deviation;
mod dlis;
mod equations;
mod export;
mod facies;
mod ingest;
mod inversion;
mod layout;
mod lrlc;
mod ml;
mod modules;
mod montecarlo;
mod multimin;
mod multimin2;
mod parsers;
#[cfg(test)]
mod pipeline_blso_test;
mod report;
mod satheight;
mod ssc;
mod python_engine;
mod workflow;

use duckdb::Connection;
use std::sync::Mutex;
use uuid::Uuid;

pub struct DbState(pub Mutex<Connection>);

/// Checkpoints the DuckDB WAL and copies the project file to `dest_path` ("Save As").
#[tauri::command]
fn save_project_as(db: tauri::State<DbState>, dest_path: String) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        conn.execute_batch("CHECKPOINT;").map_err(|e| e.to_string())?;
    }
    std::fs::copy("project.duckdb", &dest_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Lists every well in the project, for the object tree panel.
#[tauri::command]
fn list_wells(db: tauri::State<DbState>) -> Result<Vec<db::WellSummary>, String> {
    let conn = db.0.lock().unwrap();
    db::list_wells(&conn).map_err(|e| e.to_string())
}

/// Parses and ingests a batch of LAS 2.0 files (parsed concurrently via `rayon`), inserting
/// one well + its standard curves per file. Per-file failures are reported individually
/// rather than aborting the whole batch.
#[tauri::command]
fn import_las_files(db: tauri::State<DbState>, paths: Vec<String>) -> Vec<ingest::ImportResult> {
    let conn = db.0.lock().unwrap();
    ingest::import_las_files(&conn, &paths)
}

/// Parses a routine-core-analysis CSV and replaces the given well's core plug data
/// (CPOR/CPERM/CGD/CSW, alias-resolved headers, sparse/irregular depths).
#[tauri::command]
fn import_core_csv(db: tauri::State<DbState>, well_id: String, path: String) -> ingest::CoreImportResult {
    let conn = db.0.lock().unwrap();
    ingest::import_core_csv(&conn, &well_id, &path)
}

/// Fetches a well's core plug data as CPOR/CPERM/CGD/CSW series, for overlay onto
/// crossplots/log tracks (see `equations::fetch_core_series` for why this isn't
/// aligned onto the standard depth grid like `get_curve_data`).
#[tauri::command]
fn get_core_data(db: tauri::State<DbState>, well_id: String) -> Result<Vec<equations::TrackCurveSeries>, String> {
    let conn = db.0.lock().unwrap();
    equations::fetch_core_series(&conn, &well_id).map_err(|e| e.to_string())
}

/// Renders a composite log plot for one well at a true print scale, returning one vector
/// SVG per depth page plus page metadata (Phase 8 deliverables).
#[tauri::command]
fn render_composite(db: tauri::State<DbState>, spec: composite::CompositeSpec) -> Result<composite::CompositeResult, String> {
    let conn = db.0.lock().unwrap();
    composite::render_composite(&conn, &spec)
}

/// Renders a composite and writes it to disk as SVG (one file per page when multi-page),
/// returning the paths written.
#[tauri::command]
fn export_composite_svg(db: tauri::State<DbState>, spec: composite::CompositeSpec, dest_path: String) -> Result<Vec<String>, String> {
    let conn = db.0.lock().unwrap();
    let result = composite::render_composite(&conn, &spec)?;
    composite::export_svg_files(&result, &dest_path)
}

/// Renders a composite as a single multi-page PDF and writes it to `dest_path`.
#[tauri::command]
fn export_composite_pdf(db: tauri::State<DbState>, spec: composite::CompositeSpec, dest_path: String) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    let pdf = composite::render_composite_pdf(&conn, &spec)?;
    std::fs::write(&dest_path, pdf).map_err(|e| e.to_string())?;
    Ok(dest_path)
}

/// Renders the full report (cover → methodology table → zone parameters → pay summary →
/// composite log pages) as per-page SVGs for the dialog preview.
#[tauri::command]
fn render_report(db: tauri::State<DbState>, spec: report::ReportSpec) -> Result<composite::CompositeResult, String> {
    report::render_report(&db.0, &spec)
}

/// Renders the full report as one multi-page PDF and writes it to `dest_path`.
#[tauri::command]
fn export_report_pdf(db: tauri::State<DbState>, spec: report::ReportSpec, dest_path: String) -> Result<String, String> {
    let pdf = report::render_report_pdf(&db.0, &spec)?;
    std::fs::write(&dest_path, pdf).map_err(|e| e.to_string())?;
    Ok(dest_path)
}

/// Batch report export: one PDF per well into `dest_dir` (named `<WELL>_report.pdf`).
/// Returns the written paths; per-well failures are reported without aborting the rest.
#[tauri::command]
fn export_report_batch(db: tauri::State<DbState>, spec: report::ReportSpec, well_ids: Vec<String>, dest_dir: String) -> Result<Vec<String>, String> {
    let (written, errors) = report::export_report_batch(&db.0, &spec, &well_ids, &dest_dir)?;
    if !errors.is_empty() {
        return Err(format!("wrote {} file(s); failed: {}", written.len(), errors.join("; ")));
    }
    Ok(written)
}

/// Writes a base64-encoded PNG (rasterized by the frontend from a report/composite SVG
/// page) to the user-picked `dest_path` — same whitelisted-write pattern as the PDF/SVG
/// exports.
#[tauri::command]
fn save_png(dest_path: String, data_base64: String) -> Result<String, String> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|e| format!("bad PNG payload: {e}"))?;
    std::fs::write(&dest_path, bytes).map_err(|e| e.to_string())?;
    Ok(dest_path)
}

/// Parses a SCAL capillary-pressure CSV, replaces the well's `scal_pc` rows, and returns
/// the Leverett-J fit (Sw = A·J^B) at the given lab IFT for use in the sw_height module.
#[tauri::command]
fn import_scal_csv(db: tauri::State<DbState>, well_id: String, path: String, ift_lab: f64) -> ingest::ScalImportResult {
    let conn = db.0.lock().unwrap();
    ingest::import_scal_csv(&conn, &well_id, &path, ift_lab)
}

/// Fetches a well's SCAL Pc/Sw points (for the saturation-height QC plot).
#[tauri::command]
fn get_scal_pc(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ScalPcRow>, String> {
    let conn = db.0.lock().unwrap();
    db::get_scal_pc(&conn, &well_id).map_err(|e| e.to_string())
}

/// Saves (or updates, by unique name) a user-authored equation and returns its id.
#[tauri::command]
fn save_equation(db: tauri::State<DbState>, def: equations::EquationDef) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    equations::save_equation(&conn, &def).map_err(|e| e.to_string())
}

/// Lists every saved equation, for the Equation Editor's picker.
#[tauri::command]
fn list_equations(db: tauri::State<DbState>) -> Result<Vec<equations::EquationDef>, String> {
    let conn = db.0.lock().unwrap();
    equations::list_equations(&conn).map_err(|e| e.to_string())
}

/// Runs a saved equation (looked up by id) across `well_ids` concurrently via `rayon`,
/// writing results into `computed_curves`. Dispatches per the equation's language:
/// "python" → vectorized numpy subprocess engine, anything else → the Rhai path.
#[tauri::command]
fn run_equation(db: tauri::State<DbState>, equation_id: String, well_ids: Vec<String>) -> Result<Vec<equations::EquationRunResult>, String> {
    let equation = {
        let conn = db.0.lock().unwrap();
        equations::list_equations(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|e| e.equation_id == equation_id)
            .ok_or_else(|| format!("equation {equation_id} not found"))?
    };
    if equation.language == "python" {
        Ok(python_engine::run_python_equation(&db.0, &equation, &well_ids))
    } else {
        Ok(equations::run_equation(&db.0, &equation, &well_ids))
    }
}

/// Reports which Python interpreter (with numpy) the equation engine will use, if any —
/// shown in the Equation Editor so a missing install is obvious before a run.
#[tauri::command]
fn python_status() -> Option<String> {
    python_engine::find_python().map(|p| p.to_string_lossy().to_string())
}

/// Lists the curve catalog (standard + computed curves), auto-derived from the database.
#[tauri::command]
fn list_curve_catalog(db: tauri::State<DbState>) -> Result<Vec<equations::CurveCatalogEntry>, String> {
    let conn = db.0.lock().unwrap();
    equations::list_curve_catalog(&conn).map_err(|e| e.to_string())
}

/// Phase 6: lists every curve in the generic curve store (`curve_meta`/`curve_samples`)
/// for one well, across RAW/EDIT/FINAL sets — units/family/set-aware, unlike the legacy
/// `list_curve_catalog` above.
#[tauri::command]
fn list_generic_curve_catalog(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::GenericCurveCatalogEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_generic_curve_catalog(&conn, &well_id).map_err(|e| e.to_string())
}

/// Phase 6: reads every sample of one curve from the generic store, ordered by depth.
#[tauri::command]
fn get_generic_curve_samples(db: tauri::State<DbState>, curve_id: String) -> Result<Vec<db::CurveSamplePoint>, String> {
    let conn = db.0.lock().unwrap();
    db::get_curve_samples(&conn, &curve_id).map_err(|e| e.to_string())
}

/// Phase 6: imports a deviation-survey CSV for one well, computing minimum-curvature
/// TVD/TVDSS and storing it in `well_path`.
#[tauri::command]
fn import_deviation_csv(
    db: tauri::State<DbState>,
    well_id: String,
    path: String,
    datum_elevation: Option<f32>,
) -> ingest::CoreImportResult {
    let conn = db.0.lock().unwrap();
    ingest::import_deviation_csv(&conn, &well_id, &path, datum_elevation)
}

/// Phase 6: reads one well's deviation survey (with computed TVD/TVDSS) for TVD-aware views.
#[tauri::command]
fn get_well_path(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::WellPathStation>, String> {
    let conn = db.0.lock().unwrap();
    db::get_well_path(&conn, &well_id).map_err(|e| e.to_string())
}

/// Phase 6: imports every scalar channel of a DLIS file into one existing well's generic
/// curve store (via `dlisio` through the Python subprocess).
#[tauri::command]
fn import_dlis_file(db: tauri::State<DbState>, well_id: String, path: String) -> dlis::DlisImportResult {
    let conn = db.0.lock().unwrap();
    dlis::import_dlis_file(&conn, &well_id, &path)
}

/// Lists every formation top for a well, for the Tops panel.
#[tauri::command]
fn list_tops(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::TopEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_tops(&conn, &well_id).map_err(|e| e.to_string())
}

/// Returns the built-in layout registry (user-saved layouts come via `list_documents`).
#[tauri::command]
fn list_layouts() -> Vec<layout::Layout> {
    layout::list_layouts()
}

/// Saves (or replaces, by (doc_type, name)) one named JSON document — saved layouts,
/// plot property sets, and similar per-item saves.
#[tauri::command]
fn save_document(db: tauri::State<DbState>, doc_type: String, name: String, json: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::save_document(&conn, &doc_type, &name, &json).map_err(|e| e.to_string())
}

/// Lists every saved document of one type (e.g. "layout").
#[tauri::command]
fn list_documents(db: tauri::State<DbState>, doc_type: String) -> Result<Vec<db::DocumentEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_documents(&conn, &doc_type).map_err(|e| e.to_string())
}

/// Deletes one saved document by type + name.
#[tauri::command]
fn delete_document(db: tauri::State<DbState>, doc_type: String, name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_document(&conn, &doc_type, &name).map_err(|e| e.to_string())
}

// --- Well groups: user-defined subsets of wells for filtering + batch scoping ---------

#[tauri::command]
fn list_well_groups(db: tauri::State<DbState>) -> Result<Vec<db::WellGroupEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_well_groups(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_well_group(db: tauri::State<DbState>, name: String, well_ids: Vec<String>) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    db::create_well_group(&conn, &name, &well_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_well_group(db: tauri::State<DbState>, group_id: String, name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::rename_well_group(&conn, &group_id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_well_group(db: tauri::State<DbState>, group_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_well_group(&conn, &group_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_well_group_members(db: tauri::State<DbState>, group_id: String, well_ids: Vec<String>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_well_group_members(&conn, &group_id, &well_ids).map_err(|e| e.to_string())
}

/// Activates one group (filters the workspace to its members) or clears the active group
/// when `group_id` is None ("All wells").
#[tauri::command]
fn set_active_well_group(db: tauri::State<DbState>, group_id: Option<String>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_active_well_group(&conn, group_id.as_deref()).map_err(|e| e.to_string())
}

/// Fetches every curve referenced by a layout's tracks for one well, decimated to
/// `target_pixel_height` per curve — the data source for the multi-track viewer.
#[tauri::command]
fn get_track_data(
    db: tauri::State<DbState>,
    well_id: String,
    curve_names: Vec<String>,
    target_pixel_height: usize,
) -> Result<Vec<equations::TrackCurveSeries>, String> {
    let conn = db.0.lock().unwrap();
    equations::fetch_track_data(&conn, &well_id, &curve_names, target_pixel_height).map_err(|e| e.to_string())
}

/// Fetches full-resolution curve data for parameter-selection plots (histogram,
/// crossplot, Pickett), optionally windowed to a depth interval.
#[tauri::command]
fn get_curve_data(
    db: tauri::State<DbState>,
    well_id: String,
    curve_names: Vec<String>,
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> Result<Vec<equations::TrackCurveSeries>, String> {
    let conn = db.0.lock().unwrap();
    equations::fetch_curve_data(&conn, &well_id, &curve_names, depth_min, depth_max).map_err(|e| e.to_string())
}

/// Lists every deterministic module manifest — the frontend auto-generates each module's
/// parameter dialog from these (Geolog .info equivalent).
#[tauri::command]
fn list_modules() -> Vec<modules::ModuleSpec> {
    modules::list_modules()
}

/// Runs one deterministic module across the given wells (rayon-parallel), resolving
/// interval parameters per zone and writing outputs to computed_curves.
#[tauri::command]
fn run_workflow_module(db: tauri::State<DbState>, req: workflow::RunModuleRequest) -> Vec<workflow::ModuleRunResult> {
    workflow::run_workflow_module(&db.0, &req)
}

/// Computes the cutoff/lumping pay summary per well per zone, writing FLAG_* curves.
#[tauri::command]
fn run_pay_summary(db: tauri::State<DbState>, req: workflow::PaySummaryRequest) -> Result<Vec<workflow::PaySummaryRow>, String> {
    workflow::run_pay_summary(&db.0, &req)
}

/// Monte Carlo uncertainty: N seeded realizations of a chain with parameter distributions,
/// returning P10/P50/P90 net pay / NTG / PHIE / SWE / HPV + an HPV histogram per zone. Runs
/// entirely in memory (no computed_curves writes).
#[tauri::command]
fn run_monte_carlo(db: tauri::State<DbState>, req: montecarlo::McRequest) -> montecarlo::McResult {
    montecarlo::run_monte_carlo(&db.0, &req)
}

/// Phase 10-4: machine-learning bridge — supervised regression/classification and
/// unsupervised clustering/dimensionality reduction via scikit-learn subprocess.
#[tauri::command]
fn run_ml(db: tauri::State<DbState>, req: ml::MlRequest) -> ml::MlResult {
    ml::run_ml(&db.0, &req)
}

/// Generalized multi-mineral inversion: N user-defined components against N tools, with
/// hard unity + non-negativity. Writes VOL_<component> + derived PHIT/VSH/SWT/RECON curves.
#[tauri::command]
fn run_multimin(db: tauri::State<DbState>, req: multimin2::MultiminRequest) -> multimin2::MultiminResult {
    multimin2::run_multimin(&db.0, &req)
}

/// The built-in mineral/fluid endpoint library (editable defaults for the Multimin dialog).
#[tauri::command]
fn multimin_library() -> Vec<multimin2::Component> {
    multimin2::multimin_library()
}

/// Derived fluid quantities (Cw, Cmf, Cbw, α, w, CT/CXO auto-uncertainties) for the
/// Multimin dialog's fluid-properties preview.
#[tauri::command]
fn multimin_fluid_calc(props: multimin2::FluidProps) -> multimin2::FluidCalc {
    multimin2::fluid_calc(&props)
}

/// Lists the zones defined for a well.
#[tauri::command]
fn list_zones(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ZoneEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_zones(&conn, &well_id).map_err(|e| e.to_string())
}

/// Creates or updates a zone.
#[tauri::command]
fn upsert_zone(db: tauri::State<DbState>, well_id: String, zone_name: String, top_depth: f32, bottom_depth: f32) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::upsert_zone(&conn, &well_id, &zone_name, top_depth, bottom_depth).map_err(|e| e.to_string())
}

/// Deletes a zone and its parameters.
#[tauri::command]
fn delete_zone(db: tauri::State<DbState>, well_id: String, zone_name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_zone(&conn, &well_id, &zone_name).map_err(|e| e.to_string())
}

/// Rebuilds a well's zones from its formation tops (each top starts a zone).
#[tauri::command]
fn zones_from_tops(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ZoneEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::zones_from_tops(&conn, &well_id).map_err(|e| e.to_string())
}

/// Lists every per-zone parameter value for a well.
#[tauri::command]
fn list_zone_params(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ZoneParamEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_zone_params(&conn, &well_id).map_err(|e| e.to_string())
}

/// Sets (or clears, when both values are null) one per-zone parameter value.
#[tauri::command]
fn set_zone_param(
    db: tauri::State<DbState>,
    well_id: String,
    zone_name: String,
    param_name: String,
    value_num: Option<f32>,
    value_text: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_zone_param(&conn, &well_id, &zone_name, &param_name, value_num, value_text.as_deref()).map_err(|e| e.to_string())
}

/// One page of a whitelisted table for the Database Inspector, every cell as VARCHAR.
#[tauri::command]
fn get_table_page(
    db: tauri::State<DbState>,
    table: String,
    well_id: Option<String>,
    offset: usize,
    limit: usize,
) -> Result<db::TablePage, String> {
    let conn = db.0.lock().unwrap();
    db::get_table_page(&conn, &table, well_id.as_deref(), offset, limit)
}

/// Edits one wells-table field (well_name/field_name as text, td/kb as numbers).
#[tauri::command]
fn update_well_field(db: tauri::State<DbState>, well_id: String, field: String, value: Option<String>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::update_well_field(&conn, &well_id, &field, value.as_deref())
}

/// Edits one standard-curve sample (NaN = missing).
#[tauri::command]
fn update_standard_sample(db: tauri::State<DbState>, well_id: String, depth: f32, column: String, value: f32) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::update_standard_sample(&conn, &well_id, depth, &column, value)
}

/// Edits one computed-curve sample.
#[tauri::command]
fn update_computed_sample(db: tauri::State<DbState>, well_id: String, depth: f32, curve_name: String, value: f32) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::update_computed_sample(&conn, &well_id, depth, &curve_name, value)
}

/// Edits one core-plug sample (NaN = missing).
#[tauri::command]
fn update_core_sample(db: tauri::State<DbState>, well_id: String, depth: f32, column: String, value: f32) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::update_core_sample(&conn, &well_id, depth, &column, value)
}

/// Shifts every core plug of a well by `delta` metres (core-to-log alignment).
#[tauri::command]
fn shift_core_data(db: tauri::State<DbState>, well_id: String, delta: f32) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::shift_core_depths(&conn, &well_id, delta).map_err(|e| e.to_string())
}

/// Creates or updates a formation top.
#[tauri::command]
fn upsert_top(db: tauri::State<DbState>, well_id: String, top_name: String, depth: f32, color: Option<String>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::upsert_top(&conn, &well_id, &top_name, depth, color.as_deref()).map_err(|e| e.to_string())
}

/// Deletes a formation top.
#[tauri::command]
fn delete_top(db: tauri::State<DbState>, well_id: String, top_name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_top(&conn, &well_id, &top_name).map_err(|e| e.to_string())
}

/// Read-only SQL over the project database (full DuckDB SQL, SELECT-only).
#[tauri::command]
fn run_query(db: tauri::State<DbState>, sql: String, limit: usize) -> Result<db::TablePage, String> {
    let conn = db.0.lock().unwrap();
    db::run_readonly_query(&conn, &sql, limit)
}

/// Exports one well (standard + computed curves) as a LAS 2.0 file; returns row count.
#[tauri::command]
fn export_las(db: tauri::State<DbState>, well_id: String, dest_path: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    export::export_las(&conn, &well_id, &dest_path)
}

/// Kicks off a long-running stochastic multi-mineral inversion on a background thread and
/// returns immediately with a job id; the Tauri UI thread is never blocked.
#[tauri::command]
fn start_inversion(registry: tauri::State<inversion::JobRegistry>, iterations: u32) -> String {
    inversion::dispatch_inversion(registry.inner().clone(), iterations).to_string()
}

/// Polls the status/result of a previously dispatched inversion job.
#[tauri::command]
fn get_inversion_status(
    registry: tauri::State<inversion::JobRegistry>,
    job_id: String,
) -> Option<inversion::InversionStatus> {
    let uuid = Uuid::parse_str(&job_id).ok()?;
    registry.lock().unwrap().get(&uuid).cloned()
}

/// Runs a saved workflow chain (ordered modules) across the given wells. The frontend
/// supplies the `job_id` up front so it can poll `get_chain_status` for live progress while
/// this command runs on its own worker thread. Returns when the chain finishes; progress and
/// per-well errors are read via the status poll.
#[tauri::command]
fn run_workflow_chain(
    db: tauri::State<DbState>,
    registry: tauri::State<chain::ChainRegistry>,
    job_id: String,
    steps: Vec<chain::ChainStep>,
    well_ids: Vec<String>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&job_id).map_err(|e| format!("bad job id: {e}"))?;
    if steps.is_empty() {
        return Err("workflow has no steps".into());
    }
    if well_ids.is_empty() {
        return Err("no wells selected".into());
    }
    let cancel = chain::register(registry.inner(), uuid);
    chain::run_chain(&db.0, registry.inner(), uuid, &cancel, &steps, &well_ids);
    Ok(())
}

/// Polls the progress/result of a running workflow chain.
#[tauri::command]
fn get_chain_status(
    registry: tauri::State<chain::ChainRegistry>,
    job_id: String,
) -> Option<chain::ChainStatus> {
    let uuid = Uuid::parse_str(&job_id).ok()?;
    chain::status(registry.inner(), uuid)
}

/// Requests cancellation of a running workflow chain; it stops before the next step.
#[tauri::command]
fn cancel_workflow_chain(registry: tauri::State<chain::ChainRegistry>, job_id: String) {
    if let Ok(uuid) = Uuid::parse_str(&job_id) {
        chain::cancel(registry.inner(), uuid);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // `tauri dev` restarts this binary on every source-file change, so a WAL replay
    // failure (killed mid-write) is expected to happen occasionally during
    // development — self-heal it rather than crash-looping until a human notices.
    let conn = db::init_db_resilient("project.duckdb").expect("failed to initialize DuckDB");
    db::migrate_standard_curves_to_generic_store(&conn).expect("failed to migrate curves into the generic curve store");
    db::migrate_drop_computed_curves_pk(&conn).expect("failed to drop legacy computed_curves primary key");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DbState(Mutex::new(conn)))
        .manage(inversion::new_registry())
        .manage(chain::new_registry())
        .invoke_handler(tauri::generate_handler![
            save_project_as,
            list_wells,
            import_las_files,
            save_equation,
            list_equations,
            run_equation,
            list_curve_catalog,
            list_generic_curve_catalog,
            get_generic_curve_samples,
            import_deviation_csv,
            get_well_path,
            import_dlis_file,
            list_tops,
            list_layouts,
            save_document,
            list_documents,
            delete_document,
            list_well_groups,
            create_well_group,
            rename_well_group,
            delete_well_group,
            set_well_group_members,
            set_active_well_group,
            get_track_data,
            get_curve_data,
            list_modules,
            run_workflow_module,
            run_pay_summary,
            run_monte_carlo,
            list_zones,
            upsert_zone,
            delete_zone,
            zones_from_tops,
            list_zone_params,
            set_zone_param,
            get_table_page,
            update_well_field,
            update_standard_sample,
            update_computed_sample,
            update_core_sample,
            shift_core_data,
            upsert_top,
            delete_top,
            run_query,
            export_las,
            python_status,
            run_ml,
            run_multimin,
            multimin_library,
            multimin_fluid_calc,
            start_inversion,
            get_inversion_status,
            run_workflow_chain,
            get_chain_status,
            cancel_workflow_chain,
            import_core_csv,
            import_scal_csv,
            get_scal_pc,
            render_composite,
            export_composite_svg,
            export_composite_pdf,
            render_report,
            export_report_pdf,
            export_report_batch,
            save_png,
            get_core_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
