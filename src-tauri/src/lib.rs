mod chain;
mod composite;
mod contacts;
mod curve_edit;
mod curves;
mod db;
mod decimate;
mod deviation;
mod dlis;
mod equations;
mod export;
mod facies;
mod facies_tie;
mod geo;
mod health;
mod hfu;
mod ingest;
mod jobs;
mod layout;
mod lorenz;
mod lrlc;
mod ml;
mod modules;
mod montecarlo;
mod multimin;
mod multimin2;
mod netflag;
mod neutron_charts;
mod parsers;
#[cfg(test)]
mod pipeline_blso_test;
mod project;
mod report;
mod resultsqc;
mod rocktyping;
mod satheight;
mod shf_fit;
mod thomeer;
mod ssc;
mod python_engine;
mod tops;
mod unconventional;
mod workflow;

use duckdb::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// The one live DuckDB connection, behind a Mutex for single-writer safety and an Arc so a
/// long-running command can `clone()` it and move the handle into a background worker thread
/// (`tauri::async_runtime::spawn_blocking`) — the foundation for running heavy jobs off the
/// IPC/main thread (#128). project::switch_project swaps the Connection *inside* this Mutex, so
/// a background job holding an Arc clone transparently follows a project switch; the job
/// registries' `any_active` guards block a switch while a job is still running.
pub struct DbState(pub Arc<Mutex<Connection>>);

/// Checkpoints the DuckDB WAL and copies the project file to `dest_path` ("Save As").
/// Deliberately a backup export: the app KEEPS working on the current file.
#[tauri::command]
fn save_project_as(
    db: tauri::State<DbState>,
    proj: tauri::State<project::ProjectState>,
    dest_path: String,
) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        conn.execute_batch("CHECKPOINT;").map_err(|e| e.to_string())?;
    }
    let src = proj.0.lock().unwrap().clone();
    std::fs::copy(&src, &dest_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// The recent-projects list (most recent first), for the Project ribbon dropdown.
#[tauri::command]
fn list_recent_projects() -> Vec<project::RecentProject> {
    project::list_recents()
}

/// Name + path of the project currently open.
#[tauri::command]
fn current_project(proj: tauri::State<project::ProjectState>) -> project::RecentProject {
    let path = proj.0.lock().unwrap().clone();
    project::RecentProject {
        name: project::project_name(&path),
        path,
        last_opened: 0,
        exists: true,
    }
}

/// Switches the live connection to an EXISTING project file ("IP style" open).
#[tauri::command]
fn open_project(
    db: tauri::State<DbState>,
    proj: tauri::State<project::ProjectState>,
    chains: tauri::State<chain::ChainRegistry>,
    jobs_reg: tauri::State<jobs::JobRegistry>,
    path: String,
) -> Result<project::RecentProject, String> {
    if project::is_current(&proj, &path) {
        return Ok(current_project(proj));
    }
    if chain::any_active(&chains) || jobs::any_active(&jobs_reg) {
        return Err("A background job is still running — wait for it to finish before switching projects".to_string());
    }
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {path}"));
    }
    let info = project::switch_project(&db, &path)?;
    *proj.0.lock().unwrap() = info.path.clone();
    Ok(info)
}

/// Creates a FRESH project file (full schema, no wells) and switches to it.
#[tauri::command]
fn new_project(
    db: tauri::State<DbState>,
    proj: tauri::State<project::ProjectState>,
    chains: tauri::State<chain::ChainRegistry>,
    jobs_reg: tauri::State<jobs::JobRegistry>,
    path: String,
) -> Result<project::RecentProject, String> {
    if chain::any_active(&chains) || jobs::any_active(&jobs_reg) {
        return Err("A background job is still running — wait for it to finish before switching projects".to_string());
    }
    if std::path::Path::new(&path).exists() {
        return Err(format!("{path} already exists — use Open Project to open it"));
    }
    let info = project::switch_project(&db, &path)?;
    *proj.0.lock().unwrap() = info.path.clone();
    Ok(info)
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
async fn import_las_files(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    paths: Vec<String>,
) -> Result<Vec<ingest::ImportResult>, String> {
    // One job item per file (label = basename) so the Processing panel shows "WELL_12.las ✓".
    let items: Vec<(String, String)> = paths
        .iter()
        .map(|p| (p.clone(), p.rsplit(['/', '\\']).next().unwrap_or(p).to_string()))
        .collect();
    let total = paths.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Import LAS", format!("{total} file(s)"), items, total, move |job| {
        let c = conn.lock().unwrap();
        ingest::import_las_files(&c, &paths, Some(&job))
    })
    .await
}

/// Parses a routine-core-analysis CSV and replaces the given well's core plug data
/// (CPOR/CPERM/CGD/CSW, alias-resolved headers, sparse/irregular depths).
#[tauri::command]
async fn import_core_csv(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    path: String,
) -> Result<ingest::CoreImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import core", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_core_csv(&c, &well_id, &path))
    })
    .await
}

/// Imports formation tops from a CSV/TXT file (P2). Files with a WELL column update
/// every matching well; single-well files use `default_well_id` (the selected well).
#[tauri::command]
async fn import_tops_csv(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    default_well_id: Option<String>,
    path: String,
) -> Result<ingest::TopsImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import tops", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_tops_file(&c, default_well_id.as_deref(), &path))
    })
    .await
}

/// Imports a tops-style dataset (PETROGRAPHY / XRD / PERFORATION / custom) for one well,
/// replacing that well's previous rows of the same dataset (P2).
#[tauri::command]
async fn import_aux_data(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    dataset: String,
    path: String,
) -> Result<ingest::AuxImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import dataset", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_aux_file(&c, &well_id, &dataset, &path))
    })
    .await
}

/// One well's auxiliary dataset rows (all datasets when `dataset` is null).
#[tauri::command]
fn list_aux_data(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: Option<String>,
) -> Result<Vec<db::AuxRow>, String> {
    let conn = db.0.lock().unwrap();
    db::list_aux_data(&conn, &well_id, dataset.as_deref()).map_err(|e| e.to_string())
}

/// Which auxiliary datasets a well has, with row counts.
#[tauri::command]
fn list_aux_datasets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<(String, i64)>, String> {
    let conn = db.0.lock().unwrap();
    db::list_aux_datasets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Fetches a well's core plug data as CPOR/CPERM/CGD/CSW series, for overlay onto
/// crossplots/log tracks (see `equations::fetch_core_series` for why this isn't
/// aligned onto the standard depth grid like `get_curve_data`).
#[tauri::command]
fn get_core_data(db: tauri::State<DbState>, well_id: String) -> Result<tauri::ipc::Response, String> {
    let conn = db.0.lock().unwrap();
    let series = equations::fetch_core_series(&conn, &well_id).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(equations::pack_curve_series(&series)))
}

/// Renders a composite log plot for one well at a true print scale, returning one vector
/// SVG per depth page plus page metadata (Phase 8 deliverables).
#[tauri::command]
async fn render_composite(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: composite::CompositeSpec,
) -> Result<composite::CompositeResult, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Composite", "render composite", move || {
        let c = conn.lock().unwrap();
        composite::render_composite(&c, &spec)
    })
    .await
}

/// Renders a composite and writes it to disk as SVG (one file per page when multi-page),
/// returning the paths written.
#[tauri::command]
async fn export_composite_svg(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: composite::CompositeSpec,
    dest_path: String,
) -> Result<Vec<String>, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Composite", "export SVG", move || {
        let c = conn.lock().unwrap();
        let result = composite::render_composite(&c, &spec)?;
        composite::export_svg_files(&result, &dest_path)
    })
    .await
}

/// Renders a composite as a single multi-page PDF and writes it to `dest_path`.
#[tauri::command]
async fn export_composite_pdf(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: composite::CompositeSpec,
    dest_path: String,
) -> Result<String, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Composite", "export PDF", move || {
        let c = conn.lock().unwrap();
        let pdf = composite::render_composite_pdf(&c, &spec)?;
        std::fs::write(&dest_path, pdf).map_err(|e| e.to_string())?;
        Ok(dest_path)
    })
    .await
}

/// Renders the full report (cover → methodology table → zone parameters → pay summary →
/// composite log pages) as per-page SVGs for the dialog preview.
#[tauri::command]
async fn render_report(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: report::ReportSpec,
) -> Result<composite::CompositeResult, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Report", "render report", move || {
        report::render_report(&conn, &spec)
    })
    .await
}

/// Renders the full report as one multi-page PDF and writes it to `dest_path`.
#[tauri::command]
async fn export_report_pdf(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: report::ReportSpec,
    dest_path: String,
) -> Result<String, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Report", "export PDF", move || {
        let pdf = report::render_report_pdf(&conn, &spec)?;
        std::fs::write(&dest_path, pdf).map_err(|e| e.to_string())?;
        Ok(dest_path)
    })
    .await
}

/// Batch report export: one PDF per well into `dest_dir` (named `<WELL>_report.pdf`).
/// Returns the written paths; per-well failures are reported without aborting the rest.
#[tauri::command]
async fn export_report_batch(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: report::ReportSpec,
    well_ids: Vec<String>,
    dest_dir: String,
) -> Result<Vec<String>, String> {
    let conn = db.0.clone();
    let label = format!("{} report(s)", well_ids.len());
    jobs::run_simple_job(jobs_reg.inner().clone(), "Report batch", label, move || {
        let (written, errors) = report::export_report_batch(&conn, &spec, &well_ids, &dest_dir)?;
        if !errors.is_empty() {
            return Err(format!("wrote {} file(s); failed: {}", written.len(), errors.join("; ")));
        }
        Ok(written)
    })
    .await
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

/// Assembles a single Canvas-2D chart into a one-page PDF from a frontend-built content stream
/// (already in PDF user space — points, bottom-left origin; see `pdfExport.ts`) and writes it to
/// `dest_path`. Same whitelisted-write pattern as `save_png`; the PDF document scaffolding is
/// shared with the composite-log exporter (`composite::assemble_single_page_pdf`).
#[tauri::command]
fn save_plot_pdf(dest_path: String, content: String, width_pt: f64, height_pt: f64) -> Result<String, String> {
    let bytes = composite::assemble_single_page_pdf(&content, width_pt, height_pt);
    std::fs::write(&dest_path, bytes).map_err(|e| e.to_string())?;
    Ok(dest_path)
}

/// Computes a net-reservoir flag curve from a free-form crossplot polygon (see `netflag.rs`) and
/// writes it as a computed curve, returning the inside/evaluated/written counts for a status line.
#[tauri::command]
async fn run_net_flag(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: netflag::NetFlagSpec,
) -> Result<netflag::NetFlagResult, String> {
    let conn = db.0.clone();
    let label = spec.output_curve.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Net flag", label, move || {
        let c = conn.lock().unwrap();
        netflag::run_net_flag(&c, &spec)
    })
    .await
}

/// Parses a SCAL capillary-pressure CSV, replaces the well's `scal_pc` rows, and returns
/// the Leverett-J fit (Sw = A·J^B) at the given lab IFT for use in the sw_height module.
#[tauri::command]
async fn import_scal_csv(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    path: String,
    ift_lab: f64,
) -> Result<ingest::ScalImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import SCAL", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_scal_csv(&c, &well_id, &path, ift_lab))
    })
    .await
}

/// Multi-file, multi-format SCAL Pc import (increment 2): flat/long CSVs, Corelab-style
/// porous-plate wide tables, and per-plug centrifuge block files ("auto" sniffs each
/// file). All files land in ONE combined replace-write of the well's `scal_pc` rows,
/// with the Leverett-J fit over the pooled points.
#[tauri::command]
async fn import_scal_files(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    paths: Vec<String>,
    format: String,
    system: String,
    ift_lab: f64,
) -> Result<ingest::ScalImportResult, String> {
    let conn = db.0.clone();
    let detail = if paths.len() == 1 {
        paths[0].rsplit(['/', '\\']).next().unwrap_or(&paths[0]).to_string()
    } else {
        format!("{} files", paths.len())
    };
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import SCAL", detail, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_scal_files(&c, &well_id, &paths, &format, &system, ift_lab))
    })
    .await
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
async fn run_equation(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    equation_id: String,
    well_ids: Vec<String>,
) -> Result<Vec<equations::EquationRunResult>, String> {
    let (equation, items) = {
        let conn = db.0.lock().unwrap();
        let equation = equations::list_equations(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|e| e.equation_id == equation_id)
            .ok_or_else(|| format!("equation {equation_id} not found"))?;
        let items = well_items(&conn, &well_ids);
        (equation, items)
    };
    let total = well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    let label = format!("equation: {}", equation.name);
    jobs::run_job(reg, "Equation", label, items, total, move |job| {
        if equation.language == "python" {
            python_engine::run_python_equation(&conn, &equation, &well_ids, Some(&job))
        } else {
            equations::run_equation(&conn, &equation, &well_ids, Some(&job))
        }
    })
    .await
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

/// P1-c: every log-set run event for one well (set/version/module/params/when + curves),
/// newest first — the version history behind "re-run = N+1, never overwrite".
#[tauri::command]
fn list_log_sets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<equations::LogSetEntry>, String> {
    let conn = db.0.lock().unwrap();
    equations::list_log_sets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Distinct constellation (log-set) names across the project — powers the input/output
/// constellation pickers in the module and workflow dialogs (which run across many wells).
#[tauri::command]
fn list_log_set_names(db: tauri::State<DbState>) -> Result<Vec<String>, String> {
    let conn = db.0.lock().unwrap();
    equations::list_log_set_names(&conn).map_err(|e| e.to_string())
}

/// P1-c: copies one archived set version back into the current store (its curves become
/// the values every panel shows again). Returns the number of restored rows.
#[tauri::command]
fn restore_log_set(db: tauri::State<DbState>, set_id: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    equations::restore_log_set(&conn, &set_id).map_err(|e| e.to_string())
}

/// P1-c: deletes one set version's history rows (current values are kept, provenance tag
/// cleared) — for pruning old versions once they're no longer needed.
#[tauri::command]
fn delete_log_set(db: tauri::State<DbState>, set_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    equations::delete_log_set(&conn, &set_id).map_err(|e| e.to_string())
}

/// P1-c: per-well catalog of current computed curves with provenance (set/version/module/
/// when) and basic statistics (n/min/max/mean) for the searchable catalog.
#[tauri::command]
fn list_computed_catalog(db: tauri::State<DbState>, well_id: String) -> Result<Vec<equations::ComputedCatalogEntry>, String> {
    let conn = db.0.lock().unwrap();
    equations::list_computed_catalog(&conn, &well_id).map_err(|e| e.to_string())
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

/// Curve Catalog: deletes one generic-store curve (meta + samples) by id — removes a
/// shadowing/duplicate imported curve. Irreversible.
#[tauri::command]
fn delete_generic_curve(db: tauri::State<DbState>, curve_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_generic_curve(&conn, &curve_id).map_err(|e| e.to_string())
}

/// Curve Catalog: promotes one generic curve so it wins its (well, set, mnemonic) group in
/// curve resolution — the fix for DLIS/LAS same-mnemonic shadowing.
#[tauri::command]
fn promote_generic_curve(db: tauri::State<DbState>, curve_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::promote_generic_curve(&conn, &curve_id).map_err(|e| e.to_string())
}

/// Phase 6: imports a deviation-survey CSV for one well, computing minimum-curvature
/// TVD/TVDSS and storing it in `well_path`.
#[tauri::command]
async fn import_deviation_csv(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    path: String,
    datum_elevation: Option<f32>,
) -> Result<ingest::CoreImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import deviation", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_deviation_csv(&c, &well_id, &path, datum_elevation))
    })
    .await
}

/// Phase 6: reads one well's deviation survey (with computed TVD/TVDSS) for TVD-aware views.
#[tauri::command]
fn get_well_path(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::WellPathStation>, String> {
    let conn = db.0.lock().unwrap();
    db::get_well_path(&conn, &well_id).map_err(|e| e.to_string())
}

/// Per-well outcome of `materialize_tvd`.
#[derive(serde::Serialize)]
struct TvdMaterialize {
    well_id: String,
    well_name: String,
    /// Samples written for each of TVD and TVDSS; 0 = no survey or no logs yet.
    samples: usize,
    has_survey: bool,
}

/// Materializes TVD/TVDSS computed curves from each well's deviation survey onto its log
/// depth grid, so `sw_height`'s TVD input, the SHF-fitting panes, and the TVDSS correlation
/// depth-mode can consume them by name. Deviation import already does this automatically;
/// this command re-runs it (e.g. after importing logs later or editing the KB datum). Wells
/// with no survey or no logs report `samples = 0`.
#[tauri::command]
fn materialize_tvd(db: tauri::State<DbState>, well_ids: Vec<String>) -> Result<Vec<TvdMaterialize>, String> {
    let conn = db.0.lock().unwrap();
    let mut out = Vec::with_capacity(well_ids.len());
    for wid in &well_ids {
        let well_name: String = conn
            .query_row("SELECT well_name FROM wells WHERE well_id = ?1", [wid], |r| r.get(0))
            .unwrap_or_else(|_| wid.clone());
        let has_survey = !db::get_well_path(&conn, wid).map_err(|e| e.to_string())?.is_empty();
        let samples = ingest::materialize_tvd_curves(&conn, wid).map_err(|e| e.to_string())?;
        out.push(TvdMaterialize { well_id: wid.clone(), well_name, samples, has_survey });
    }
    Ok(out)
}

/// Phase 6: imports every scalar channel of a DLIS file into one existing well's generic
/// curve store (via `dlisio` through the Python subprocess).
#[tauri::command]
async fn import_dlis_file(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    path: String,
) -> Result<dlis::DlisImportResult, String> {
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import DLIS", base, move || {
        let c = conn.lock().unwrap();
        Ok(dlis::import_dlis_file(&c, &well_id, &path))
    })
    .await
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

// --- Pinned wells: a persisted favourites subset, reused as a run-scope shortcut -------

#[tauri::command]
fn list_pinned_wells(db: tauri::State<DbState>) -> Result<Vec<String>, String> {
    let conn = db.0.lock().unwrap();
    db::list_pinned_wells(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_well_pin(db: tauri::State<DbState>, well_id: String, pinned: bool) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_well_pin(&conn, &well_id, pinned).map_err(|e| e.to_string())
}

/// Replaces the whole pinned set at once ("pin selection" / "clear pins").
#[tauri::command]
fn set_pinned_wells(db: tauri::State<DbState>, well_ids: Vec<String>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_pinned_wells(&conn, &well_ids).map_err(|e| e.to_string())
}

/// Fetches every curve referenced by a layout's tracks for one well, decimated to
/// `target_pixel_height` per curve — the data source for the multi-track viewer.
#[tauri::command]
fn get_track_data(
    db: tauri::State<DbState>,
    well_id: String,
    curve_names: Vec<String>,
    target_pixel_height: usize,
) -> Result<tauri::ipc::Response, String> {
    let conn = db.0.lock().unwrap();
    let series = equations::fetch_track_data(&conn, &well_id, &curve_names, target_pixel_height)
        .map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(equations::pack_curve_series(&series)))
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
) -> Result<tauri::ipc::Response, String> {
    let conn = db.0.lock().unwrap();
    let series = equations::fetch_curve_data(&conn, &well_id, &curve_names, depth_min, depth_max)
        .map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(equations::pack_curve_series(&series)))
}

/// Lists every deterministic module manifest — the frontend auto-generates each module's
/// parameter dialog from these (module-manifest model).
#[tauri::command]
fn list_modules() -> Vec<modules::ModuleSpec> {
    modules::list_modules()
}

/// well_id → (well_id, well_name) pairs for a job's item list, so the Processing panel shows
/// well names instead of UUIDs. One cheap query; ids without a matching row fall back to the id.
fn well_items(conn: &duckdb::Connection, well_ids: &[String]) -> Vec<(String, String)> {
    let mut names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT well_id, well_name FROM wells") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
            for row in rows.flatten() {
                names.insert(row.0, row.1);
            }
        }
    }
    well_ids
        .iter()
        .map(|id| (id.clone(), names.get(id).cloned().unwrap_or_else(|| id.clone())))
        .collect()
}

/// Runs one deterministic module across the given wells (rayon-parallel), resolving interval
/// parameters per zone and writing outputs to computed_curves. Async + off-thread via the job
/// registry, so it reports live per-well progress and a Cancel in the Processing panel and never
/// blocks the IPC thread. Returns the same per-well result list as before.
#[tauri::command]
async fn run_workflow_module(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: workflow::RunModuleRequest,
) -> Result<Vec<workflow::ModuleRunResult>, String> {
    let items = {
        let conn = db.0.lock().unwrap();
        well_items(&conn, &req.well_ids)
    };
    let total = req.well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Module", req.module.clone(), items, total, move |job| {
        workflow::run_workflow_module_into(&conn, &req, None, Some(&job.cancel), Some(&job))
    })
    .await
}

/// Computes the cutoff/lumping pay summary per well per zone, writing FLAG_* curves.
#[tauri::command]
async fn run_pay_summary(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: workflow::PaySummaryRequest,
) -> Result<Vec<workflow::PaySummaryRow>, String> {
    let conn = db.0.clone();
    // The Field Dashboard's automatic field-wide QC pass (stats_only + skip_version) runs on
    // every refresh — keep it silent (off-thread, but no job card) so it doesn't flood the
    // Processing panel. A user-initiated pay summary still shows a job.
    if req.stats_only && req.skip_version {
        return tauri::async_runtime::spawn_blocking(move || workflow::run_pay_summary(&conn, &req))
            .await
            .map_err(|e| e.to_string())?;
    }
    jobs::run_simple_job(jobs_reg.inner().clone(), "Pay summary", "cutoffs & pay", move || {
        workflow::run_pay_summary(&conn, &req)
    })
    .await
}

/// Cutoff-sensitivity sweep (Method 1): pay metric vs a swept cutoff, per well. Reads
/// VSH/PHIE/SWE/PERM and writes nothing.
#[tauri::command]
async fn run_cutoff_sweep(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: workflow::CutoffSweepRequest,
) -> Result<workflow::CutoffSweepResult, String> {
    let conn = db.0.clone();
    let label = format!("cutoff sweep: {}", req.property);
    jobs::run_simple_job(jobs_reg.inner().clone(), "Cutoff sweep", label, move || {
        workflow::run_cutoff_sweep(&conn, &req)
    })
    .await
}

/// Monte Carlo uncertainty: N seeded realizations of a chain with parameter distributions,
/// returning P10/P50/P90 net pay / NTG / PHIE / SWE / HPV + an HPV histogram per zone. Runs
/// entirely in memory (no computed_curves writes). Async + off-thread with live per-well
/// progress + Cancel in the Processing panel.
#[tauri::command]
async fn run_monte_carlo(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: montecarlo::McRequest,
) -> Result<montecarlo::McResult, String> {
    let items = {
        let conn = db.0.lock().unwrap();
        well_items(&conn, &req.well_ids)
    };
    let total = req.well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Monte Carlo", "uncertainty".to_string(), items, total, move |job| {
        montecarlo::run_monte_carlo(&conn, &req, Some(&job))
    })
    .await
}

/// Phase 10-4: machine-learning bridge — supervised regression/classification and unsupervised
/// clustering/dimensionality reduction via scikit-learn subprocess. Async + off-thread; the
/// Processing panel shows the training phase then the per-well writeback.
#[tauri::command]
async fn run_ml(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: ml::MlRequest,
) -> Result<ml::MlResult, String> {
    let items = {
        let conn = db.0.lock().unwrap();
        well_items(&conn, &req.apply_well_ids)
    };
    let total = req.apply_well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Machine learning", req.algorithm.clone(), items, total, move |job| {
        ml::run_ml(&conn, &req, Some(&job))
    })
    .await
}

/// Model-comparison leaderboard (Wave B item 3): blind-well GroupKFold CV over algorithm ×
/// feature-subset combos, with permutation importance + confusion matrix. Evaluation only — it
/// writes no curves. Off-thread so the fit/predict sweep doesn't freeze the IPC thread.
#[tauri::command]
async fn run_ml_eval(
    db: tauri::State<'_, DbState>,
    req: ml::MlEvalRequest,
) -> Result<ml::MlEvalResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || ml::run_ml_eval(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Cuddy FOIL / fractal BVW saturation-height fit (Wave B item 8, SHF side): pools computed
/// PHIE/SW/TVDSS across wells, fits BVW = a·H^b above the FWL, and optionally scans for the common
/// free-water level (Cuddy 1993 Eq 19). Off-thread; writes no curves.
#[tauri::command]
async fn run_cuddy_foil(
    db: tauri::State<'_, DbState>,
    req: shf_fit::CuddyFoilRequest,
) -> Result<shf_fit::CuddyFoilResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || shf_fit::run_cuddy_foil(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Height-domain SHF fit (Wave B item 8, increment 2): Brooks-Corey or Skelt-Harrison fitted to
/// the log-derived Sw-vs-height cloud. Off-thread; writes no curves.
#[tauri::command]
async fn run_shf_fit(
    db: tauri::State<'_, DbState>,
    req: shf_fit::ShfFitRequest,
) -> Result<shf_fit::ShfFitResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || shf_fit::run_shf_fit(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Thomeer Pc hyperbola fit (Wave B item 8, increment 2): per-plug (Pd, G, Bv∞) over the
/// selected wells' scal_pc points, for the Pd-G rock-typing plane + Swanson apex.
/// Off-thread; writes no curves.
#[tauri::command]
async fn run_thomeer_fit(
    db: tauri::State<'_, DbState>,
    req: thomeer::ThomeerRequest,
) -> Result<thomeer::ThomeerResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || thomeer::run_thomeer_fit(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Hydraulic-flow-unit clustering (Wave B item 8, increment 2): partitions the scoped wells' core
/// φ-k cloud into HFUs on log10(FZI) — Ward (exact min-variance) or histogram antimodes — with the
/// per-HFU Amaefule perm transform. Off-thread; writes no curves.
#[tauri::command]
async fn run_hfu_cluster(
    db: tauri::State<'_, DbState>,
    req: hfu::HfuRequest,
) -> Result<hfu::HfuResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || hfu::run_hfu_cluster(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Stratigraphic Modified Lorenz Plot (playbook #3, increment 3a): builds the depth-ordered
/// flow/storage-capacity curve for one well from its φ + k logs, segments it into flow units, and
/// returns the Lorenz heterogeneity coefficient. Off-thread; writes no curves.
#[tauri::command]
async fn run_lorenz(
    db: tauri::State<'_, DbState>,
    req: lorenz::LorenzRequest,
) -> Result<lorenz::LorenzResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || lorenz::run_lorenz(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Electrofacies tie-in QC (Wave B item 8, increment 2): confusion matrix + dominant-class purity
/// of a predicted log rock-type curve against a reference/core rock-type curve. Off-thread.
#[tauri::command]
async fn run_facies_confusion(
    db: tauri::State<'_, DbState>,
    req: facies_tie::FaciesConfusionRequest,
) -> Result<facies_tie::FaciesConfusionResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || facies_tie::run_facies_confusion(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Generalized multi-mineral inversion: N user-defined components against N tools, with hard
/// unity + non-negativity. Writes VOL_<component> + derived PHIT/VSH/SWT/RECON curves. Async +
/// off-thread via the job registry — the solve no longer freezes the IPC thread, and the
/// Processing panel shows live per-well progress + Cancel. Same per-well result payload.
#[tauri::command]
async fn run_multimin(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: multimin2::MultiminRequest,
) -> Result<multimin2::MultiminResult, String> {
    let items = {
        let conn = db.0.lock().unwrap();
        well_items(&conn, &req.apply_well_ids)
    };
    let total = req.apply_well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "SandiMin", "mineral solver".to_string(), items, total, move |job| {
        multimin2::run_multimin(&conn, &req, Some(&job))
    })
    .await
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

/// Wet-clay → dry-clay endpoint conversion (KKT ONWJ xlsx workflow) for the
/// SandiMin dialog's converter panel.
#[tauri::command]
fn multimin_dry_clay(input: multimin2::WetClayInput) -> Result<multimin2::DryClayCalc, String> {
    multimin2::dry_clay_calc(&input)
}

/// Zone-averaged FTEMP_F / RMF read from the precalc module's output curves, for
/// the SandiMin fluid-properties autofill.
#[tauri::command]
fn multimin_fluid_from_precalc(
    db: tauri::State<DbState>,
    well_id: String,
    top: Option<f64>,
    bottom: Option<f64>,
) -> Result<multimin2::PrecalcFluid, String> {
    multimin2::fluid_from_precalc(&db.0, &well_id, top, bottom)
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

/// Lists the informal colored highlights for a well.
#[tauri::command]
fn list_highlights(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::HighlightEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_highlights(&conn, &well_id).map_err(|e| e.to_string())
}

/// Creates or updates a highlight (keyed by client-generated id).
#[tauri::command]
fn upsert_highlight(
    db: tauri::State<DbState>,
    well_id: String,
    highlight_id: String,
    top_depth: f32,
    bottom_depth: f32,
    color: Option<String>,
    label: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::upsert_highlight(&conn, &well_id, &highlight_id, top_depth, bottom_depth, color.as_deref(), label.as_deref())
        .map_err(|e| e.to_string())
}

/// Deletes a highlight.
#[tauri::command]
fn delete_highlight(db: tauri::State<DbState>, well_id: String, highlight_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_highlight(&conn, &well_id, &highlight_id).map_err(|e| e.to_string())
}

/// Lists every fluid contact in the project (the correlation view filters per well).
#[tauri::command]
fn list_fluid_contacts(db: tauri::State<DbState>) -> Result<Vec<db::FluidContact>, String> {
    let conn = db.0.lock().unwrap();
    db::list_fluid_contacts(&conn).map_err(|e| e.to_string())
}

/// Creates or updates a fluid contact (keyed by client-generated id).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn upsert_fluid_contact(
    db: tauri::State<DbState>,
    contact_id: String,
    field_name: Option<String>,
    well_id: Option<String>,
    contact_type: String,
    depth: f64,
    is_tvdss: bool,
    color: Option<String>,
    label: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::upsert_fluid_contact(
        &conn,
        &contact_id,
        field_name.as_deref(),
        well_id.as_deref(),
        &contact_type,
        depth,
        is_tvdss,
        color.as_deref(),
        label.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Deletes a fluid contact.
#[tauri::command]
fn delete_fluid_contact(db: tauri::State<DbState>, contact_id: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::delete_fluid_contact(&conn, &contact_id).map_err(|e| e.to_string())
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

/// Imports well surface locations from a CSV/TXT (Field Map, Wave E item 22). Multi-well
/// files (a WELL column) match project wells by name; single-well files use
/// `default_well_id`. `default_zone` fills the UTM zone for rows without a ZONE column.
#[tauri::command]
async fn import_well_locations(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    default_well_id: Option<String>,
    default_zone: Option<String>,
    path: String,
) -> Result<ingest::LocationsImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import locations", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_locations_file(&c, default_well_id.as_deref(), default_zone.as_deref(), &path))
    })
    .await
}

/// Returns the wells whose surface location falls inside `polygon` (an ordered
/// `[[x, y], …]` ring in UTM metres) — the "draw a polygon → select wells" hit test
/// behind assigning a map lasso to a well group. Wells without coordinates are excluded.
#[tauri::command]
fn wells_in_polygon(db: tauri::State<DbState>, polygon: Vec<[f64; 2]>) -> Result<Vec<db::WellSummary>, String> {
    let conn = db.0.lock().unwrap();
    let wells = db::list_wells(&conn).map_err(|e| e.to_string())?;
    let ring: Vec<(f64, f64)> = polygon.iter().map(|p| (p[0], p[1])).collect();
    Ok(geo::wells_in_polygon(&wells, &ring))
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

/// Interactive curve edit from the log view's right-click menu: wireline shift or an
/// interval op (set/blank/interpolate/scale). Returns the previous samples for undo.
#[tauri::command]
fn edit_curve(db: tauri::State<DbState>, req: curve_edit::CurveEditRequest) -> Result<curve_edit::CurveEditResult, String> {
    let conn = db.0.lock().unwrap();
    curve_edit::edit_curve(&conn, &req)
}

/// Undo path for `edit_curve`: writes back the (depth, value) pairs a prior edit
/// returned, in the same packed `depth[n] + value[n]` f32-LE byte convention.
#[tauri::command]
fn restore_curve_values(
    db: tauri::State<DbState>,
    well_id: String,
    curve: String,
    point_count: usize,
    data: Vec<u8>,
) -> Result<usize, String> {
    let (depth, values) = curve_edit::unpack_pairs(point_count, &data)?;
    let conn = db.0.lock().unwrap();
    curve_edit::restore_curve_values(&conn, &well_id, &curve, &depth, &values)
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

/// Stratigraphic crossing check: warnings for top pairs in this well whose depth order
/// contradicts the majority of other wells (run after every interactive pick/drag).
#[tauri::command]
fn check_top_order(db: tauri::State<DbState>, well_id: String) -> Result<Vec<String>, String> {
    let conn = db.0.lock().unwrap();
    tops::check_top_order(&conn, &well_id)
}

/// Proposes marker depths in target wells by correlating a log shape around the source
/// well's pick (Petrel-style autocorrelation). Read-only — the dialog applies picks.
#[tauri::command]
async fn autocorrelate_top(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: tops::AutoCorrRequest,
) -> Result<tops::AutoCorrResult, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Autocorrelate", "top correlation", move || {
        let c = conn.lock().unwrap();
        Ok(tops::autocorrelate_top(&c, &req))
    })
    .await
}

/// Propagates SEVERAL markers together into target wells with one consistent depth warp
/// (monotone — no crossings), each with its own per-interval confidence. Read-only.
#[tauri::command]
async fn autocorrelate_multi(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: tops::MultiAutoCorrRequest,
) -> Result<tops::MultiAutoCorrResult, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Autocorrelate", "multi-marker correlation", move || {
        let c = conn.lock().unwrap();
        Ok(tops::autocorrelate_multi(&c, &req))
    })
    .await
}

/// Suggests a fluid-contact depth in one well from its logs (Sw=0.5 crossover, deep-
/// resistivity drop, density-neutron gas base), each with a confidence. Read-only.
#[tauri::command]
fn suggest_contacts(
    db: tauri::State<DbState>,
    req: contacts::ContactSuggestRequest,
) -> Result<contacts::ContactSuggestResult, String> {
    let conn = db.0.lock().unwrap();
    Ok(contacts::suggest_contacts(&conn, &req))
}

/// Cross-well consistency for a contact type: fits a flat-TVDSS surface through the picked
/// contacts and flags wells that disagree. Read-only.
#[tauri::command]
fn check_contact_consistency(
    db: tauri::State<DbState>,
    contact_type: String,
    flag_abs: Option<f32>,
) -> Result<contacts::ContactConsistency, String> {
    let conn = db.0.lock().unwrap();
    Ok(contacts::check_contact_consistency(&conn, &contact_type, flag_abs.unwrap_or(3.0)))
}

/// Per-depth water-saturation envelope across the app's Sw models (Archie/Simandoux/Indonesia/
/// Juhász, plus Waxman-Smits/Dual-Water when a Qv/Swb curve is present) — the Results-QC "does the
/// model choice change the answer?" metric. Read-only.
#[tauri::command]
fn sw_method_spread(
    db: tauri::State<DbState>,
    req: resultsqc::SwSpreadRequest,
) -> Result<resultsqc::SwSpreadResult, String> {
    let conn = db.0.lock().unwrap();
    resultsqc::sw_method_spread(&conn, &req)
}

/// Read-only SQL over the project database (full DuckDB SQL, SELECT-only).
#[tauri::command]
fn run_query(db: tauri::State<DbState>, sql: String, limit: usize) -> Result<db::TablePage, String> {
    let conn = db.0.lock().unwrap();
    db::run_readonly_query(&conn, &sql, limit)
}

/// Exports one well (standard + computed curves) as a LAS 2.0 file; returns row count.
#[tauri::command]
async fn export_las(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    dest_path: String,
) -> Result<usize, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Export LAS", "write LAS", move || {
        let c = conn.lock().unwrap();
        export::export_las(&c, &well_id, &dest_path)
    })
    .await
}

/// Runs a saved workflow chain (ordered modules) across the given wells. The frontend
/// supplies the `job_id` up front so it can poll `get_chain_status` for live progress while
/// this command runs on its own worker thread. Returns when the chain finishes; progress and
/// per-well errors are read via the status poll.
#[tauri::command]
fn run_workflow_chain(
    db: tauri::State<DbState>,
    registry: tauri::State<chain::ChainRegistry>,
    jobs_reg: tauri::State<jobs::JobRegistry>,
    job_id: String,
    steps: Vec<chain::ChainStep>,
    well_ids: Vec<String>,
    output_set: Option<String>,
    input_set: Option<String>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&job_id).map_err(|e| format!("bad job id: {e}"))?;
    if steps.is_empty() {
        return Err("workflow has no steps".into());
    }
    if well_ids.is_empty() {
        return Err("no wells selected".into());
    }
    let cancel = chain::register(registry.inner(), uuid);
    // Universal Processing-panel job: ONE shared cancel flag drives both registries, so Cancel
    // works from the Workflow Builder or the universal panel. Item labels are well names so the
    // panel shows "WELL_12" rather than a UUID.
    let items: Vec<(String, String)> = {
        let conn = db.0.lock().unwrap();
        let mut names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if let Ok(mut stmt) = conn.prepare("SELECT well_id, well_name FROM wells") {
            if let Ok(rows) =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                for row in rows.flatten() {
                    names.insert(row.0, row.1);
                }
            }
        }
        well_ids
            .iter()
            .map(|id| (id.clone(), names.get(id).cloned().unwrap_or_else(|| id.clone())))
            .collect()
    };
    let label = steps.iter().map(|s| s.module.as_str()).collect::<Vec<_>>().join(" → ");
    let job = jobs::register(jobs_reg.inner(), uuid, "Workflow chain", label, items, cancel.clone());
    // Run OFF the IPC/main thread so the window stays responsive and the frontend's
    // get_chain_status poll + Cancel button are actually serviced *during* the run. As a sync
    // command this blocked the event loop for the whole multi-minute chain — which is exactly
    // why the existing progress bar sat frozen at "Starting…". db.0 is an Arc<Mutex<Connection>>
    // so we clone the handle into the worker; the chain registry is already an Arc.
    let db = db.0.clone();
    let registry = registry.inner().clone();
    // Plain OS thread, NOT tokio::spawn_blocking: a sync #[tauri::command] runs on the main
    // event-loop thread, which is NOT a Tokio runtime worker, so tokio::task::spawn_blocking
    // panics there ("must be called from the context of a Tokio runtime") and aborts the app
    // the instant Run is clicked. std::thread has no runtime-context requirement. run_chain is
    // ordinary blocking code and every captured value (Arc DB handle, Arc registry, Arc cancel
    // flag, the owned Vecs) is Send + 'static; the DB connection is already used off-thread by
    // rayon under this same Mutex, so cross-thread use is sound. A panic inside stays on this
    // thread (it can't abort the process); the job simply stops reporting progress.
    std::thread::spawn(move || {
        chain::run_chain(
            &db,
            &registry,
            uuid,
            &cancel,
            &steps,
            &well_ids,
            output_set.as_deref(),
            input_set.as_deref(),
            Some(&job),
        );
    });
    Ok(())
}

/// Hardware Health Monitor snapshot — system memory %, this process's USER/GDI object %, and
/// GPU video-memory %. Cheap; the Health panel polls it on a timer. Windows-only metrics
/// (other targets return all-None → the panel shows "n/a").
#[tauri::command]
fn health_snapshot() -> health::HealthSnapshot {
    health::snapshot()
}

/// Snapshot of every job for the universal Processing panel — most recent first. Reads only
/// the (separate) job registry mutex, never the DB, so the poll stays responsive even while a
/// heavy chain holds the DB lock on its worker thread.
#[tauri::command]
fn list_jobs(jobs_reg: tauri::State<jobs::JobRegistry>) -> Vec<jobs::JobView> {
    jobs::list(jobs_reg.inner())
}

/// Requests cancellation of one job by id (flips the shared flag the runner checks per well),
/// so a Cancel from the universal panel stops the same run the Workflow Builder started.
#[tauri::command]
fn cancel_job(jobs_reg: tauri::State<jobs::JobRegistry>, job_id: String) {
    if let Ok(uuid) = Uuid::parse_str(&job_id) {
        jobs::cancel(jobs_reg.inner(), uuid);
    }
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
    // The most recently opened project that still exists, else the legacy
    // `project.duckdb` in the cwd — so existing installs open exactly as before.
    let startup = project::startup_path();
    // `tauri dev` restarts this binary on every source-file change, so a WAL replay
    // failure (killed mid-write) is expected to happen occasionally during
    // development — self-heal it rather than crash-looping until a human notices.
    // Boot timing (stderr, visible in the `tauri dev` terminal). These three steps run BEFORE
    // the window is created, so their sum IS the black-screen time on a large project. The logs
    // pinpoint which one dominates the ~5-min open on the 540-well / ~2 GB file so the fix can
    // target it precisely (DB open vs the standard-curves backfill vs the PK-drop check).
    let boot = std::time::Instant::now();
    let conn = db::init_db_resilient(&startup).expect("failed to initialize DuckDB");
    eprintln!("[boot] init_db_resilient: {:?}  ({startup})", boot.elapsed());
    let t = std::time::Instant::now();
    db::migrate_standard_curves_to_generic_store(&conn).expect("failed to migrate curves into the generic curve store");
    eprintln!("[boot] migrate_standard_curves_to_generic_store: {:?}", t.elapsed());
    let t = std::time::Instant::now();
    db::migrate_drop_computed_curves_pk(&conn).expect("failed to drop legacy computed_curves primary key");
    eprintln!("[boot] migrate_drop_computed_curves_pk: {:?}", t.elapsed());
    eprintln!("[boot] total pre-window DB init: {:?}", boot.elapsed());
    project::register_recent(&startup);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DbState(Arc::new(Mutex::new(conn))))
        .manage(project::ProjectState(Mutex::new(project::absolute(&startup))))
        .manage(chain::new_registry())
        .manage(jobs::new_registry())
        .invoke_handler(tauri::generate_handler![
            save_project_as,
            list_recent_projects,
            current_project,
            open_project,
            new_project,
            list_wells,
            import_las_files,
            save_equation,
            list_equations,
            run_equation,
            list_curve_catalog,
            list_log_sets,
            list_log_set_names,
            restore_log_set,
            delete_log_set,
            list_computed_catalog,
            list_generic_curve_catalog,
            get_generic_curve_samples,
            delete_generic_curve,
            promote_generic_curve,
            import_deviation_csv,
            get_well_path,
            materialize_tvd,
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
            list_pinned_wells,
            set_well_pin,
            set_pinned_wells,
            get_track_data,
            get_curve_data,
            list_modules,
            run_workflow_module,
            run_pay_summary,
            run_cutoff_sweep,
            run_monte_carlo,
            list_zones,
            upsert_zone,
            delete_zone,
            list_highlights,
            upsert_highlight,
            delete_highlight,
            list_fluid_contacts,
            upsert_fluid_contact,
            delete_fluid_contact,
            zones_from_tops,
            list_zone_params,
            set_zone_param,
            get_table_page,
            update_well_field,
            update_standard_sample,
            update_computed_sample,
            update_core_sample,
            shift_core_data,
            edit_curve,
            restore_curve_values,
            upsert_top,
            delete_top,
            check_top_order,
            autocorrelate_top,
            autocorrelate_multi,
            suggest_contacts,
            check_contact_consistency,
            sw_method_spread,
            run_query,
            export_las,
            python_status,
            run_ml,
            run_ml_eval,
            run_cuddy_foil,
            run_shf_fit,
            run_thomeer_fit,
            run_hfu_cluster,
            run_lorenz,
            run_net_flag,
            run_facies_confusion,
            run_multimin,
            multimin_library,
            multimin_fluid_calc,
            multimin_dry_clay,
            multimin_fluid_from_precalc,
            run_workflow_chain,
            get_chain_status,
            cancel_workflow_chain,
            list_jobs,
            cancel_job,
            health_snapshot,
            import_core_csv,
            import_tops_csv,
            import_well_locations,
            wells_in_polygon,
            import_aux_data,
            list_aux_data,
            list_aux_datasets,
            import_scal_csv,
            import_scal_files,
            get_scal_pc,
            render_composite,
            export_composite_svg,
            export_composite_pdf,
            render_report,
            export_report_pdf,
            export_report_batch,
            save_png,
            save_plot_pdf,
            get_core_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
