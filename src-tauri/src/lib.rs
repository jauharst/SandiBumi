mod chain;
mod composite;
mod condition;
mod contacts;
mod coreimage;
mod curve_edit;
mod curves;
mod db;
mod decimate;
mod deviation;
mod distribution;
mod dlis;
mod equations;
#[cfg(test)]
mod example_data_test;
mod export;
#[cfg(test)]
mod field_fixtures;
mod frame;
mod reframe;
mod facies;
mod facies_tie;
mod geo;
mod health;
mod hfu;
mod images;
mod petrography;
mod plugqc;
mod ingest;
mod intake;
mod jobs;
mod layout;
mod lithology;
mod lorenz;
mod lrlc;
mod ml;
mod modules;
mod montecarlo;
mod multimin;
mod multimin2;
mod netflag;
mod neutron_charts;
mod office;
mod param_sources;
mod parsers;
#[cfg(test)]
mod pipeline_field_test;
mod project;
mod registration;
mod report;
mod resultsqc;
mod rocktyping;
mod satheight;
mod shf_fit;
mod thomeer;
mod ssc;
mod statistics;
mod python_engine;
mod tops;
mod unconventional;
mod units;
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

/// Why the project on disk could not be opened, and what the session is running on instead.
///
/// Startup runs before the window exists, so a failure there cannot be reported the way every
/// other error in this app is. `panic = "abort"` plus `windows_subsystem = "windows"` means an
/// aborting `run()` produces no window, no dialog and no console — the user double-clicks
/// SandiBumi and *nothing happens*, with nothing to read and nothing to send us. So every
/// startup failure becomes one of these instead, and the app opens far enough to say so.
///
/// The likeliest trigger is mundane: DuckDB takes an exclusive lock, so launching a second
/// SandiBumi while the first still has the project open used to kill the second one silently.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StartupProblem {
    /// The project we tried, and failed, to open.
    pub attempted_path: String,
    /// The underlying error verbatim — it names the actual cause (another instance holding the
    /// lock, a read-only volume, a DuckDB file written by a newer version).
    pub message: String,
    /// The throwaway project the session is running on instead. Empty = memory only.
    pub recovered_to: String,
    /// False when even the recovery file could not be created, so **nothing will persist**.
    /// The UI has to say so plainly: the alternative is a petrophysicist doing an afternoon's
    /// interpretation into a database that evaporates on close.
    pub recovery_persists: bool,
}

/// `None` on a normal launch. Read once by the frontend at boot.
pub struct StartupState(pub Mutex<Option<StartupProblem>>);

/// What the background startup open produced, published exactly once.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenOutcome {
    /// Set only when the intended project could NOT be opened (same payload the blocking
    /// dialog already renders).
    pub problem: Option<StartupProblem>,
    /// Wall-clock seconds the open took. The UI uses it to explain a long wait afterwards
    /// ("that took 14 minutes because it upgraded your project's storage").
    pub elapsed_secs: u64,
    /// The project file actually live at the end of it.
    pub path: String,
}

/// Handshake for the startup open, which now runs on a background thread so the WINDOW can
/// exist first (see `run`). The frontend awaits `await_project_open` and builds nothing until
/// it resolves — that gate is what keeps any command from reading the empty placeholder
/// database the window starts on.
pub struct DbInit(pub Arc<(Mutex<Option<OpenOutcome>>, std::sync::Condvar)>);

/// The published slot type behind `DbInit`.
type OpenCell = (Mutex<Option<OpenOutcome>>, std::sync::Condvar);

/// Publishes the outcome and wakes every waiter.
fn store_outcome(cell: &OpenCell, outcome: OpenOutcome) {
    let (lock, cv) = cell;
    *lock.lock().unwrap() = Some(outcome);
    cv.notify_all();
}

/// Blocks until the outcome has been published, then returns a copy.
///
/// The value is STORED, not merely signalled, which is what makes a fast launch work: the
/// normal case is that the open finishes before the frontend ever asks, so the notify fires
/// with nobody listening. A waiter arriving afterwards must find the answer sitting there
/// rather than wait for a signal that already came and went — otherwise every quick launch
/// would hang at the boot overlay forever. Pinned by `fast_open_published_before_the_wait`.
fn wait_for_outcome(cell: &OpenCell) -> OpenOutcome {
    let (lock, cv) = cell;
    let mut slot = lock.lock().unwrap();
    while slot.is_none() {
        slot = cv.wait(slot).unwrap();
    }
    slot.clone().expect("loop exits only once published")
}

/// Blocks until the background open finishes, then reports how it went.
///
/// Async so the wait costs a blocking-pool thread rather than the event loop — the whole point
/// is that the window stays alive and painting while a field-scale project opens.
#[tauri::command]
async fn await_project_open(init: tauri::State<'_, DbInit>) -> Result<OpenOutcome, String> {
    let cell = init.0.clone();
    tauri::async_runtime::spawn_blocking(move || wait_for_outcome(&cell))
        .await
        .map_err(|e| e.to_string())
}

/// What went wrong opening the project at launch, if anything. The frontend calls this on boot
/// and, when it returns something, blocks the workspace behind an explanatory dialog.
#[tauri::command]
fn startup_problem(state: tauri::State<StartupState>) -> Option<StartupProblem> {
    state.0.lock().unwrap().clone()
}

/// Engine-copies the live project to `dest_path` ("Save As"). Deliberately a backup
/// export: the app KEEPS working on the current file. The engine copy (rather than a
/// file copy) writes only live rows, so a Save As is also a compaction — a field project
/// bloated by months of module re-runs exports at its true data size.
/// Async: copying a field-scale project is a multi-minute write, and on the event loop that
/// is a frozen window (see `open_project`).
#[tauri::command]
async fn save_project_as(
    db: tauri::State<'_, DbState>,
    proj: tauri::State<'_, project::ProjectState>,
    dest_path: String,
) -> Result<(), String> {
    let src = proj.0.lock().unwrap().clone();
    if src == dest_path {
        return Err("That is the currently open file — Save As needs a different destination".to_string());
    }
    // The OS save dialog already confirmed an overwrite; a stale same-named file (and any
    // WAL it left) must go first, or ATTACH would merge into it instead of replacing it.
    if std::path::Path::new(&dest_path).exists() {
        std::fs::remove_file(&dest_path).map_err(|e| format!("could not overwrite {dest_path}: {e}"))?;
    }
    let stale_wal = format!("{dest_path}.wal");
    if std::path::Path::new(&stale_wal).exists() {
        std::fs::remove_file(&stale_wal).map_err(|e| format!("could not overwrite {stale_wal}: {e}"))?;
    }
    let handle = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = handle.lock().unwrap();
        db::engine_copy_to(&conn, &dest_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// "Compact Project": rewrites the open project in place, dropping the dead space left by
/// module re-runs (see `project::compact_project`). Blocked while background jobs run —
/// the connection swap must not race a chain mid-write.
#[tauri::command]
async fn compact_project(
    db: tauri::State<'_, DbState>,
    proj: tauri::State<'_, project::ProjectState>,
    chains: tauri::State<'_, chain::ChainRegistry>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
) -> Result<project::CompactReport, String> {
    if chain::any_active(&chains) || jobs::any_active(&jobs_reg) {
        return Err("A background job is still running — wait for it to finish before compacting".to_string());
    }
    let path = proj.0.lock().unwrap().clone();
    let owned = DbState(db.0.clone());
    tauri::async_runtime::spawn_blocking(move || project::compact_project(&owned, &path))
        .await
        .map_err(|e| e.to_string())?
}

/// Drains the queued boot/maintenance notices (one-time migration backups, memory caps,
/// compaction results) for the status line and process history. Each notice is returned
/// exactly once.
#[tauri::command]
fn boot_report() -> Vec<String> {
    db::take_boot_notes()
}

/// The recent-projects list (most recent first), for the Project ribbon dropdown.
#[tauri::command]
fn list_recent_projects() -> Vec<project::RecentProject> {
    project::list_recents()
}

/// Which project is open, as a `RecentProject`. Shared by the command and by the
/// project-switch commands (which cannot call a `State`-taking command from an async body).
fn project_info(proj: &project::ProjectState) -> project::RecentProject {
    let path = proj.0.lock().unwrap().clone();
    project::RecentProject {
        name: project::project_name(&path),
        path,
        last_opened: 0,
        exists: true,
    }
}

/// Name + path of the project currently open.
#[tauri::command]
fn current_project(proj: tauri::State<project::ProjectState>) -> project::RecentProject {
    project_info(&proj)
}

/// Switches the live connection to an EXISTING project file ("IP style" open).
///
/// **Async on purpose.** A sync `#[tauri::command]` runs on the main event-loop thread, so
/// opening a field-scale project — which may run one-time storage migrations, each backing
/// up gigabytes first — froze the whole window for the duration (a real 2.5 GB project took
/// ~15 minutes, during which Windows reports "not responding"). Off-thread, the window keeps
/// painting and the status line's "this can take minutes" message is actually readable.
/// Commands that touch the database still block on its mutex, which is correct: they must
/// not see a half-swapped project.
#[tauri::command]
async fn open_project(
    db: tauri::State<'_, DbState>,
    proj: tauri::State<'_, project::ProjectState>,
    chains: tauri::State<'_, chain::ChainRegistry>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    path: String,
) -> Result<project::RecentProject, String> {
    if project::is_current(&proj, &path) {
        return Ok(project_info(&proj));
    }
    if chain::any_active(&chains) || jobs::any_active(&jobs_reg) {
        return Err("A background job is still running — wait for it to finish before switching projects".to_string());
    }
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {path}"));
    }
    let owned = DbState(db.0.clone());
    let info = tauri::async_runtime::spawn_blocking(move || project::switch_project(&owned, &path))
        .await
        .map_err(|e| e.to_string())??;
    *proj.0.lock().unwrap() = info.path.clone();
    Ok(info)
}

/// Creates a FRESH project file (full schema, no wells) and switches to it.
/// Async for the same reason as `open_project`: the switch closes the OUTGOING project,
/// whose checkpoint can be slow on a big one.
#[tauri::command]
async fn new_project(
    db: tauri::State<'_, DbState>,
    proj: tauri::State<'_, project::ProjectState>,
    chains: tauri::State<'_, chain::ChainRegistry>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    path: String,
) -> Result<project::RecentProject, String> {
    if chain::any_active(&chains) || jobs::any_active(&jobs_reg) {
        return Err("A background job is still running — wait for it to finish before switching projects".to_string());
    }
    if std::path::Path::new(&path).exists() {
        return Err(format!("{path} already exists — use Open Project to open it"));
    }
    let owned = DbState(db.0.clone());
    let info = tauri::async_runtime::spawn_blocking(move || project::switch_project(&owned, &path))
        .await
        .map_err(|e| e.to_string())??;
    *proj.0.lock().unwrap() = info.path.clone();
    Ok(info)
}

/// The project's declared depth unit as a code ("M"/"FT"), and whether it was explicitly
/// declared. The frontend needs it for two different jobs: converting stored depths into
/// the unit the user wants to READ, and deriving the true 1:N print scale (which depends
/// on how long a stored unit physically is, not on what is displayed).
#[tauri::command]
fn get_project_depth_unit(db: tauri::State<DbState>) -> Result<(String, bool), String> {
    let conn = db.0.lock().unwrap();
    let declared = units::project_depth_unit(&conn).map_err(|e| e.to_string())?;
    Ok((declared.unwrap_or_default().code().to_string(), declared.is_some()))
}

/// Declares the project's depth unit.
///
/// Refuses once the project holds wells: their depths are already STORED in the old unit,
/// so re-declaring alone would silently reinterpret every one of them (a 2,438 m well
/// would start reading as 2,438 ft). Converting stored data is a separate, deliberate
/// migration — not a side effect of changing a preference.
#[tauri::command]
fn set_project_depth_unit(db: tauri::State<DbState>, unit: String) -> Result<(), String> {
    let Some(target) = units::DepthUnit::from_code(&unit) else {
        return Err(format!("unknown depth unit '{unit}' (expected M or FT)"));
    };
    let conn = db.0.lock().unwrap();
    units::set_project_depth_unit_checked(&conn, target)
}

/// Quantity families backed by at least one reviewed numeric unit transform.
#[tauri::command]
fn list_convertible_unit_families() -> Vec<String> {
    curves::convertible_unit_families()
}

/// The one project-wide absent-value sentinel supplied to every registered data writer.
#[tauri::command]
fn get_project_null_sentinel(db: tauri::State<DbState>) -> Result<f32, String> {
    let conn = db.0.lock().unwrap();
    export::project_null_sentinel(&conn)
}

/// Declares the project-wide absent-value sentinel. Writer registration makes this a
/// required argument, so no export path can silently fall back to a private value.
#[tauri::command]
fn set_project_null_sentinel(db: tauri::State<DbState>, null_sentinel: f32) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    export::set_project_null_sentinel(&conn, null_sentinel)
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
    set_name: Option<String>,
    attach: Option<bool>,
    file_depth_unit: Option<String>,
    channel_nulls: Option<parsers::ChannelNullValues>,
    null_rules: Option<Vec<parsers::NullExceptionRule>>,
    non_monotonic_index: Option<ingest::NonMonotonicIndexDecision>,
    duplicate_depth_policy: Option<parsers::DuplicateDepthPolicy>,
    ms_per_ft_meanings: Option<std::collections::HashMap<String, curves::MsPerFtMeaning>>,
) -> Result<Vec<ingest::ImportResult>, String> {
    // Import-sets options (T-IMP-02): one set name per batch; attach-by-name defaults ON
    // when the frontend doesn't say otherwise (the dialog always sends it explicitly).
    let opts = ingest::LasImportOptions {
        set_name,
        attach: attach.unwrap_or(true),
        file_depth_unit,
        channel_nulls: channel_nulls.unwrap_or_default(),
        null_rules: null_rules.unwrap_or_default(),
        non_monotonic_index,
        duplicate_depth_policy,
        ms_per_ft_meanings: ms_per_ft_meanings.unwrap_or_default(),
    };
    // One job item per file (label = basename) so the Processing panel shows "WELL_12.las ✓".
    let items: Vec<(String, String)> = paths
        .iter()
        .map(|p| (p.clone(), p.rsplit(['/', '\\']).next().unwrap_or(p).to_string()))
        .collect();
    let total = paths.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Import LAS", format!("{total} file(s)"), items, total, true, move |job| {
        let c = conn.lock().unwrap();
        ingest::import_las_files_with(&c, &paths, Some(&job), &opts)
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
    depth_column: Option<usize>,
) -> Result<ingest::CoreImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import core", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_core_csv_with_depth_column(&c, &well_id, &path, depth_column))
    })
    .await
}

/// Core import v2 (T-IMP-07), probe half: reads a core CSV/TXT (delimiter auto-detected)
/// and reports headers, guessed roles (incl. a WELL/WN column), column types, sample rows,
/// distinct wells, percent + depth-unit detection — everything the mapping dialog shows
/// for CONFIRMATION. Writes nothing.
#[tauri::command]
fn probe_core_table(path: String) -> Result<parsers::TableProbe, String> {
    parsers::probe_core_table(&path).map_err(|e| e.to_string())
}

/// Core import v2 commit half: imports one core table under the dialog-confirmed mapping.
/// Rows route per well name (exactly-one-match rule; unmatched/ambiguous reported, never
/// guessed) or all to `fallback_well_id`; depths convert from `depth_unit` to the
/// project's declared unit. Per-well replace-on-reimport semantics. Columns listed in
/// `mapping.extras` are stored as point data under `extras_dataset` (default "CORE").
#[tauri::command]
fn import_core_table(
    db: tauri::State<DbState>,
    path: String,
    mapping: parsers::CoreMapping,
    depth_unit: Option<String>,
    fallback_well_id: Option<String>,
    extras_dataset: Option<String>,
    set_name: Option<String>,
    #[allow(non_snake_case)] followCore: Option<bool>,
) -> Result<ingest::CoreTableImportResult, String> {
    let conn = db.0.lock().unwrap();
    Ok(ingest::import_core_table(
        &conn,
        &path,
        &mapping,
        depth_unit.as_deref(),
        fallback_well_id.as_deref(),
        extras_dataset.as_deref(),
        set_name.as_deref(),
        followCore.unwrap_or(false),
    ))
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

/// Imports a tops-style point dataset (PETROGRAPHY / XRD / CEC / OIL SHOW / PERFORATION /
/// custom) for one well as a NEW named delivery — `set_name` is auto-suffixed per well
/// rather than overwriting an earlier one, and becomes that dataset's live set (P2/T-IMP-08).
#[tauri::command]
async fn import_aux_data(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    dataset: String,
    path: String,
    set_name: Option<String>,
    follow_core: Option<bool>,
) -> Result<ingest::AuxImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    let follow_core = follow_core.unwrap_or(false);
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import dataset", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_aux_file(&c, &well_id, &dataset, &path, set_name.as_deref(), follow_core))
    })
    .await
}

/// Every point-data delivery of a well (XRD, CEC, oil show, core extras …), grouped by
/// dataset — for the set manager and the Wells tree.
#[tauri::command]
fn list_aux_sets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::AuxSetInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_aux_sets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Makes one point-data delivery the live one for its dataset (others untouched).
#[tauri::command]
fn set_active_aux_set(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
    set_name: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_active_aux_set(&conn, &well_id, &dataset, &set_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_aux_set(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
    set_name: String,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_aux_set(&conn, &well_id, &dataset, &set_name).map_err(|e| e.to_string())
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

/// Every measurement name in the project's point data, so a dialog can offer what exists
/// instead of asking the user to type a name and read an error. Project-wide by design — see
/// `db::list_aux_item_catalog`.
#[tauri::command]
fn list_aux_item_catalog(db: tauri::State<DbState>) -> Result<Vec<db::AuxItemInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_aux_item_catalog(&conn).map_err(|e| e.to_string())
}

// --- Depth-registered images (thin sections, core photographs) ----------------------

/// Reads the headers of the selected image files: what they are, their true pixel size, and
/// the depth guessed from each filename. Read-only — nothing is stored until the wizard's
/// commit, so a wrong guess is corrected on screen rather than in the database.
#[tauri::command]
async fn probe_image_files(paths: Vec<String>) -> Result<Vec<images::ImageProbe>, String> {
    tauri::async_runtime::spawn_blocking(move || images::probe_image_files(&paths))
        .await
        .map_err(|e| e.to_string())
}

/// Lifts the plates out of one or more petrography workbooks so the normal import wizard can take
/// them. A petrography delivery arrives as a workbook with one worksheet per plate — the well, the
/// depth and the magnification in cells, the pictures anchored on top — which a file picker cannot
/// read at all. Extracts to a temporary folder; nothing is stored until the wizard's commit.
#[tauri::command]
async fn probe_plate_workbooks(paths: Vec<String>) -> Result<images::WorkbookProbe, String> {
    tauri::async_runtime::spawn_blocking(move || {
        images::probe_plate_workbooks(&paths, &images::workbook_scratch_dir())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Is Pillow reachable? Decides whether the wizard offers TIFF and whether it warns that
/// pictures will print as labelled frames.
#[tauri::command]
async fn image_support() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(images::pillow_available).await.map_err(|e| e.to_string())
}

/// Commits a confirmed image delivery. Long-running (it shells out to Pillow and writes
/// megabytes), so it is async + `spawn_blocking`: a sync command would freeze the window.
#[tauri::command]
async fn import_well_images(
    db: tauri::State<'_, DbState>,
    req: images::ImageImportRequest,
) -> Result<images::ImageImportResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let c = conn.lock().unwrap();
        images::import_images(&c, &req)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Picture METADATA for a well, from the ACTIVE delivery of each dataset. Never the pixels —
/// a well with 300 core photographs must list in kilobytes.
#[tauri::command]
fn list_well_images(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: Option<String>,
) -> Result<Vec<db::ImageInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_well_images(&conn, &well_id, dataset.as_deref()).map_err(|e| e.to_string())
}

/// Which image datasets a well has, with the ACTIVE delivery's counts.
#[tauri::command]
fn list_image_datasets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<(String, i64)>, String> {
    let conn = db.0.lock().unwrap();
    db::list_image_datasets(&conn, &well_id).map_err(|e| e.to_string())
}

/// The pixels of one picture, as raw bytes (rule 3 — never a JSON array). The frontend wraps
/// them in a Blob of the returned mime type; the mime rides in a header the caller already
/// has from `list_well_images`.
#[tauri::command]
fn get_well_image(db: tauri::State<DbState>, image_id: String) -> Result<tauri::ipc::Response, String> {
    let conn = db.0.lock().unwrap();
    let (_mime, data) = db::get_well_image(&conn, &image_id).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(data))
}

/// Every image delivery of a well, for the set manager and the Wells tree.
#[tauri::command]
fn list_image_sets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ImageSetInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_image_sets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Makes one image delivery the live one for its dataset (others untouched).
#[tauri::command]
fn set_active_image_set(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
    set_name: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_active_image_set(&conn, &well_id, &dataset, &set_name).map_err(|e| e.to_string())
}

/// Deletes one image delivery; the newest survivor of that dataset takes over.
#[tauri::command]
fn delete_image_set(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
    set_name: String,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_image_set(&conn, &well_id, &dataset, &set_name).map_err(|e| e.to_string())
}

/// Deletes one picture.
#[tauri::command]
fn delete_well_image(db: tauri::State<DbState>, image_id: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_well_image(&conn, &image_id).map_err(|e| e.to_string())
}

/// Re-registers one picture: core-to-log alignment and labelling for pictures.
#[tauri::command]
fn update_well_image(
    db: tauri::State<DbState>,
    image_id: String,
    depth_top: f32,
    depth_base: Option<f32>,
    name: String,
    caption: Option<String>,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::update_well_image(&conn, &image_id, depth_top, depth_base, &name, caption.as_deref())
        .map_err(|e| e.to_string())
}

/// Which array logs a well carries, for the layout dialog's picker and the object tree.
#[tauri::command]
fn list_array_curves(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ArrayCurveInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_array_curves(&conn, &well_id).map_err(|e| e.to_string())
}

/// Drops one array log. Array logs are the only output whose size scales with iteration count,
/// so there has to be a way to reclaim one without deleting the study that produced it.
#[tauri::command]
fn delete_array_log(
    db: tauri::State<DbState>,
    well_id: String,
    set_name: String,
    curve_name: String,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_array_log(&conn, &well_id, &set_name, &curve_name).map_err(|e| e.to_string())
}

/// Fetches one array log — a whole DISTRIBUTION at every depth — as a raw byte buffer.
///
/// Raw IPC rather than JSON is not an optimization here, it is the only workable path: a
/// 2000-sample well with 256 realizations is half a million floats, which serde would encode
/// as a JSON number array of roughly 4 MB for the frontend to `JSON.parse` on the main thread.
///
/// Layout, all little-endian, mirrored by `decodeArrayLog` in `src\ipc.ts`:
///   [u32 depth_count][u32 width][f32 depth x depth_count][f32 values x depth_count*width]
///
/// The value block is ROW-MAJOR by depth and padded with NaN to a uniform `width`. Padding
/// rather than a ragged encoding keeps the frontend able to index `row * width + r` directly,
/// and NaN is already this project's only missing-value marker, so a padded cell is dropped by
/// exactly the same code that drops a realization that failed to converge.
#[tauri::command]
fn get_array_log(
    db: tauri::State<DbState>,
    well_id: String,
    set_name: Option<String>,
    curve_name: String,
) -> Result<tauri::ipc::Response, String> {
    let conn = db.0.lock().unwrap();
    let rows = db::read_array_log(&conn, &well_id, set_name.as_deref(), &curve_name).map_err(|e| e.to_string())?;
    let width = rows.iter().map(|r| r.samples.len()).max().unwrap_or(0);
    let mut packed: Vec<f32> = Vec::with_capacity(rows.len() * (width + 1));
    packed.extend(rows.iter().map(|r| r.depth));
    for r in &rows {
        packed.extend_from_slice(&r.samples);
        packed.extend(std::iter::repeat(f32::NAN).take(width - r.samples.len()));
    }
    let mut out = Vec::with_capacity(8 + packed.len() * 4);
    out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    out.extend_from_slice(&(width as u32).to_le_bytes());
    out.extend_from_slice(bytemuck::cast_slice(&packed));
    Ok(tauri::ipc::Response::new(out))
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

/// Which office-document packages the discovered Python can import. Asked when the workbook
/// dialog opens so a button that cannot work explains itself instead of failing at save time.
#[tauri::command]
async fn office_support() -> Result<office::OfficeSupport, String> {
    tauri::async_runtime::spawn_blocking(office::office_support).await.map_err(|e| e.to_string())
}

/// The asset-team deck. Slides are built from the pay-summary DATA (matplotlib figures), not
/// from composite pages — a log plot at 1:200 pasted into a slide stops being at 1:200.
#[tauri::command]
async fn export_deck(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: office::DeckSpec,
    dest_path: String,
) -> Result<office::DeckResult, String> {
    let conn = db.0.clone();
    let label = format!("{} well(s)", spec.well_ids.len());
    jobs::run_simple_job(jobs_reg.inner().clone(), "Deck", label, move || {
        office::export_deck(&conn, &spec, &dest_path)
    })
    .await
}

/// The EDITABLE Word twin of the report PDF — same title, author, methodology, cutoffs and
/// tables, so it can be adapted into a client's own template. The native PDF stays the default
/// deliverable and keeps the composite log pages.
#[tauri::command]
async fn export_report_docx(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: report::ReportSpec,
    dest_path: String,
) -> Result<String, String> {
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Report", "export Word", move || {
        office::export_report_docx(&conn, &spec, &dest_path)
    })
    .await
}

/// One `.docx` per well into `dest_dir`. Per-well failures are reported without aborting.
#[tauri::command]
async fn export_report_docx_batch(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: report::ReportSpec,
    well_ids: Vec<String>,
    dest_dir: String,
) -> Result<Vec<String>, String> {
    let conn = db.0.clone();
    let label = format!("{} Word report(s)", well_ids.len());
    jobs::run_simple_job(jobs_reg.inner().clone(), "Report batch", label, move || {
        let (written, errors) = office::export_report_docx_batch(&conn, &spec, &well_ids, &dest_dir)?;
        if !errors.is_empty() {
            return Err(format!("wrote {} file(s); failed: {}", written.len(), errors.join("; ")));
        }
        Ok(written)
    })
    .await
}

/// Writes the study as a formatted multi-sheet Excel workbook. Runs as a job: a field-scale
/// pay summary is minutes of work, and the Processing monitor should be able to see it.
#[tauri::command]
async fn export_workbook(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    spec: office::WorkbookSpec,
    dest_path: String,
) -> Result<office::WorkbookResult, String> {
    let conn = db.0.clone();
    let label = format!("{} well(s)", spec.well_ids.len());
    jobs::run_simple_job(jobs_reg.inner().clone(), "Workbook", label, move || {
        office::export_workbook(&conn, &spec, &dest_path)
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
/// file). The selected files form ONE delivery, stored as the named SCAL set (auto-suffixed
/// rather than overwriting an earlier report) with the Leverett-J fit over the pooled points.
#[tauri::command]
async fn import_scal_files(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    paths: Vec<String>,
    format: String,
    system: String,
    ift_lab: f64,
    set_name: Option<String>,
    follow_core: Option<bool>,
) -> Result<ingest::ScalImportResult, String> {
    let conn = db.0.clone();
    let follow_core = follow_core.unwrap_or(false);
    let detail = if paths.len() == 1 {
        paths[0].rsplit(['/', '\\']).next().unwrap_or(&paths[0]).to_string()
    } else {
        format!("{} files", paths.len())
    };
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import SCAL", detail, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_scal_files(&c, &well_id, &paths, &format, &system, ift_lab, set_name.as_deref(), follow_core))
    })
    .await
}

/// Fetches a well's SCAL Pc/Sw points from its ACTIVE delivery (saturation-height QC plot).
#[tauri::command]
fn get_scal_pc(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ScalPcRow>, String> {
    let conn = db.0.lock().unwrap();
    db::get_scal_pc(&conn, &well_id).map_err(|e| e.to_string())
}

/// A well's SCAL deliveries, for the set manager and the Wells tree.
#[tauri::command]
fn list_scal_sets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ScalSetInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_scal_sets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Makes one SCAL delivery live — Pc/Sw QC, Leverett-J and Thomeer fits all follow it.
#[tauri::command]
fn set_active_scal_set(db: tauri::State<DbState>, well_id: String, set_name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_active_scal_set(&conn, &well_id, &set_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_scal_set(db: tauri::State<DbState>, well_id: String, set_name: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_scal_set(&conn, &well_id, &set_name).map_err(|e| e.to_string())
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
    jobs::run_job(reg, "Equation", label, items, total, true, move |job| {
        if equation.language == "python" {
            python_engine::run_python_equation(&conn, &equation, &well_ids, Some(&job))
        } else {
            equations::run_equation(&conn, &equation, &well_ids, Some(&job))
        }
    })
    .await
}

/// Reports which Python interpreter (with numpy) the equation engine will use, and whether
/// the optional scipy is importable in it — shown in the Equation Editor so a missing install
/// is obvious while writing the script, not after it is queued across ninety wells.
#[tauri::command]
fn python_status() -> python_engine::PythonStatus {
    python_engine::python_status()
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

/// Renames / re-units / re-families one imported curve, returning its PREVIOUS identity so
/// the caller can push an undo. Metadata only — no sample is touched — but the mnemonic and
/// family are what module inputs resolve by, so this repoints what modules read.
#[tauri::command]
fn update_curve_meta(
    db: tauri::State<DbState>,
    curve_id: String,
    mnemonic: String,
    unit: Option<String>,
    family: Option<String>,
) -> Result<db::CurveMetaEdit, String> {
    let conn = db.0.lock().unwrap();
    db::update_curve_meta_fields(&conn, &curve_id, &mnemonic, unit.as_deref(), family.as_deref())
        .map_err(|e| e.to_string())
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
    survey_name: Option<String>,
) -> Result<ingest::CoreImportResult, String> {
    let conn = db.0.clone();
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import deviation", base, move || {
        let c = conn.lock().unwrap();
        Ok(ingest::import_deviation_csv(&c, &well_id, &path, datum_elevation, survey_name.as_deref()))
    })
    .await
}

/// One well's core deliveries and deviation surveys, for the set manager (T-IMP-08/-12).
#[tauri::command]
fn list_core_sets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::CoreSetInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_core_sets(&conn, &well_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_surveys(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::SurveyInfo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_surveys(&conn, &well_id).map_err(|e| e.to_string())
}

/// Makes one core delivery the live one — every core reader (log overlay, φ-k clouds,
/// SandiMin calibration, DB Inspector edits) follows it from here on.
#[tauri::command]
fn set_active_core_set(db: tauri::State<DbState>, well_id: String, set_name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    db::set_active_core_set(&conn, &well_id, &set_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_core_set(db: tauri::State<DbState>, well_id: String, set_name: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::delete_core_set(&conn, &well_id, &set_name).map_err(|e| e.to_string())
}

/// Makes one survey the live one and RE-MATERIALIZES TVD/TVDSS from it — the stored
/// curves must never keep the previous survey's geometry (they feed every height
/// calculation). Returns the number of samples rewritten.
#[tauri::command]
fn set_active_survey(db: tauri::State<DbState>, well_id: String, survey_name: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::set_active_survey(&conn, &well_id, &survey_name).map_err(|e| e.to_string())?;
    ingest::materialize_tvd_curves(&conn, &well_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_survey(db: tauri::State<DbState>, well_id: String, survey_name: String) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    let removed = db::delete_survey(&conn, &well_id, &survey_name).map_err(|e| e.to_string())?;
    // Whatever survey took over (or none) now owns TVD/TVDSS.
    let _ = ingest::materialize_tvd_curves(&conn, &well_id);
    Ok(removed)
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
/// Async: this runs over EVERY selected well, so at field scale it is a minutes-long write —
/// on the event loop that would freeze the window (see `open_project`).
#[tauri::command]
async fn materialize_tvd(
    db: tauri::State<'_, DbState>,
    well_ids: Vec<String>,
) -> Result<Vec<TvdMaterialize>, String> {
    let handle = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = handle.lock().unwrap();
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
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Phase 6: imports every scalar channel of a DLIS file into one existing well's generic
/// curve store (via `dlisio` through the Python subprocess). `set_name` (import-sets):
/// omitted/RAW = legacy replace-with-count semantics; anything else auto-suffixes per
/// well so duplicates are kept.
#[tauri::command]
async fn import_dlis_file(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: Option<String>,
    path: String,
    set_name: Option<String>,
    file_depth_unit: Option<String>,
    ms_per_ft_meaning: Option<curves::MsPerFtMeaning>,
    outside_interval_decision: Option<dlis::DlisOutsideIntervalDecision>,
    duplicate_decisions: Option<Vec<dlis::DlisDuplicateDecision>>,
    las_sentinel_exceptions: Option<Vec<String>>,
    confirmed_well_mappings: Option<Vec<dlis::DlisWellMapping>>,
) -> Result<dlis::DlisImportResult, String> {
    let base = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
    let conn = db.0.clone();
    jobs::run_simple_job(jobs_reg.inner().clone(), "Import DLIS", base, move || {
        let c = conn.lock().unwrap();
        Ok(dlis::import_dlis_file_with_unit_designation(
            &c,
            well_id.as_deref(),
            &path,
            set_name.as_deref(),
            file_depth_unit.as_deref(),
            ms_per_ft_meaning,
            outside_interval_decision,
            duplicate_decisions.as_deref().unwrap_or(&[]),
            las_sentinel_exceptions.as_deref().unwrap_or(&[]),
            confirmed_well_mappings.as_deref().unwrap_or(&[]),
        ))
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

/// The curve names a module would write, given the inputs and renames chosen so far — what the
/// module pane's output grid is filled from.
///
/// A pure question about the manifest, so no database and no job: it is answered by the same code
/// the runner uses (`workflow::preview_output_names`), which is the whole point. Expanding a
/// `log_out_as` pattern in the frontend instead would be a second copy of a naming rule, and this
/// app has the `composite.rs`-versus-renderer scar to show for that pattern.
#[tauri::command]
fn module_output_names(
    module: String,
    log_inputs: std::collections::HashMap<String, String>,
    opts: std::collections::HashMap<String, String>,
) -> Result<Vec<workflow::OutputName>, String> {
    workflow::preview_output_names(&module, &log_inputs, &opts)
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
    jobs::run_job(reg, "Module", req.module.clone(), items, total, true, move |job| {
        workflow::run_workflow_module_into(&conn, &req, None, Some(&job.cancel), Some(&job))
    })
    .await
}


// --- Intake (intake.rs): one importer for any delimited text -------------------------------

/// Writes pasted text to a temp file and hands back its path.
///
/// So a pasted table and the same table on disk take the IDENTICAL parse and commit path — one
/// parser, one write, and no way for a paste to behave differently. Doing it in Rust also keeps
/// the frontend free of a filesystem plugin it otherwise needs for nothing.
#[tauri::command]
async fn intake_paste(text: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut path = std::env::temp_dir();
        // Named rather than randomised so a user who wants to look at what they pasted can, and
        // so a second paste replaces the first instead of filling the temp folder.
        path.push("sandibumi-intake-paste.txt");
        std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Reads a delimited file and reports everything the Intake pane needs to confirm a mapping —
/// column kinds, proposed roles with their reasons, the decimal convention, a preview grid.
/// **Writes nothing**, so a wrong guess is seen rather than discovered afterwards.
#[tauri::command]
async fn intake_probe(
    path: String,
    opts: intake::TableOptions,
) -> Result<intake::IntakeProbe, String> {
    tauri::async_runtime::spawn_blocking(move || intake::probe(&path, &opts).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// Commits a confirmed mapping. Deliberately routed through `ingest::import_core_table` rather
/// than a second write path: that function already owns well routing, the unit conversion, the
/// percent rule, per-well replace, depth dedup and carrying every unclaimed column into
/// `aux_data`. Two implementations of those rules would eventually disagree, silently.
#[tauri::command]
async fn intake_commit(
    db: tauri::State<'_, DbState>,
    req: intake::IntakeCommit,
) -> Result<Vec<ingest::CoreTableImportResult>, String> {
    let mapping = intake::mapping_from_roles(&req.roles)?;
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = conn.lock().map_err(|_| "database busy".to_string())?;
        let mut out = Vec::new();
        for path in &req.paths {
            out.push(ingest::import_core_table(
                &conn,
                path,
                &mapping,
                req.depth_unit.as_deref(),
                req.fallback_well_id.as_deref(),
                req.extras_dataset.as_deref(),
                req.set_name.as_deref(),
                req.follow_core,
            ));
        }
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Reads a WIDE or BLOCK table for the pane, writing nothing.
///
/// The long path has had a preview since it shipped; the array path did not, so a duplicated depth
/// — which the array store REFUSES, one vector per depth — was only named once the import had
/// already run and half-written. Same `read_wide` the commit uses, so the two cannot disagree
/// about what the file says.
#[tauri::command]
async fn intake_probe_arrays(
    path: String,
    opts: intake::TableOptions,
    roles: Vec<String>,
    block: bool,
) -> Result<intake::ArrayPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        intake::probe_arrays(&path, &opts, &roles, block).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Imports a WIDE or BLOCK table into the array store (`intake::commit_arrays`).
///
/// Separate from `intake_commit` because the destination is a different store with a different
/// shape, not because the front end is different — the pane, the grid and the roles are the same.
/// The layout is the user's DECLARATION: a wide table and a long one are both rectangles of
/// numbers and nothing in the characters says which is which.
#[tauri::command]
async fn intake_commit_arrays(
    db: tauri::State<'_, DbState>,
    req: intake::ArrayCommit,
) -> Result<Vec<intake::ArrayImportResult>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = conn.lock().map_err(|_| "database busy".to_string())?;
        Ok(intake::commit_arrays(&conn, &req))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Imports columns marked CURVE as continuous logs into the generic curve store.
///
/// The route a delimited file of logs had no way in by — Import LAS reads LAS, and everything else
/// the pane produces is point data. A GR every 15 cm stored in `aux_data` would be invisible to
/// every module, plot and export.
#[tauri::command]
async fn intake_commit_curves(
    db: tauri::State<'_, DbState>,
    req: intake::CurveCommit,
) -> Result<Vec<intake::CurveImportResult>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = conn.lock().map_err(|_| "database busy".to_string())?;
        Ok(intake::commit_curves(&conn, &req))
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Statistics (statistics.rs) -------------------------------------------------------------
// Every one is a pure READ — nothing here writes a curve, a flag or a log set — so each runs
// silently off-thread rather than posting a job card the user never asked for, the same rule a
// stats-only pay summary already follows.

/// One row per well x zone x curve: n, missing, min/max, mean, spread and the user's percentiles.
/// Returns the percentile list actually used, so a table can label its own columns rather than
/// assuming the default.
#[tauri::command]
async fn stats_curve_summary(
    db: tauri::State<'_, DbState>,
    req: statistics::CurveStatsRequest,
) -> Result<(Vec<statistics::CurveStatsRow>, Vec<f32>), String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || statistics::curve_summary(&conn, &req))
        .await
        .map_err(|e| e.to_string())?
}

/// Two curves against each other per well x zone: Pearson, Spearman, bias, RMS and the OLS line.
#[tauri::command]
async fn stats_pair_summary(
    db: tauri::State<'_, DbState>,
    req: statistics::PairStatsRequest,
) -> Result<Vec<statistics::PairStatsRow>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || statistics::pair_summary(&conn, &req))
        .await
        .map_err(|e| e.to_string())?
}

/// The same curves in two log sets — what a re-run actually changed, including where it gained
/// or lost coverage.
#[tauri::command]
async fn stats_versus_sets(
    db: tauri::State<'_, DbState>,
    req: statistics::VersusRequest,
) -> Result<Vec<statistics::VersusRow>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || statistics::versus_sets(&conn, &req))
        .await
        .map_err(|e| e.to_string())?
}

/// Thickness of whatever satisfies a condition — a flag, a class, a cutoff or a marker interval.
#[tauri::command]
async fn stats_thickness(
    db: tauri::State<'_, DbState>,
    req: statistics::ThicknessRequest,
) -> Result<Vec<statistics::ThicknessRow>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || statistics::thickness(&conn, &req))
        .await
        .map_err(|e| e.to_string())?
}

/// Least squares on 1..n predictors, scored by leave-one-WELL-out.
#[tauri::command]
async fn stats_fit(
    db: tauri::State<'_, DbState>,
    req: statistics::FitRequest,
) -> Result<statistics::FitResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || statistics::fit_curves(&conn, &req))
        .await
        .map_err(|e| e.to_string())?
}

/// Computes the cutoff/lumping pay summary per well per zone, writing FLAG_* curves.
#[tauri::command]
async fn run_pay_summary(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: workflow::PaySummaryRequest,
) -> Result<Vec<workflow::PaySummaryRow>, String> {
    let conn = db.0.clone();
    // A stats-only pay summary persists nothing (workflow.rs gates every FLAG_* write behind
    // !stats_only), so it is a pure read — run it silently off-thread rather than posting a
    // "Pay summary" job card the user never asked for. The Field Dashboard is the only stats-only
    // caller and it reports its own progress in its status line, so a card would be redundant and,
    // labelled "cutoffs & pay", misleading (nothing is written). A persisting pay summary — an
    // explicit Cutoffs & Summary run, or a report render — still shows a job.
    if req.stats_only {
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
    jobs::run_job(reg, "Monte Carlo", "uncertainty".to_string(), items, total, true, move |job| {
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
    jobs::run_job(reg, "Machine learning", req.algorithm.clone(), items, total, true, move |job| {
        ml::run_ml(&conn, &req, Some(&job))
    })
    .await
}

/// Applies a SAVED model to wells it has never seen. Nothing is refitted — that is the point:
/// a refit on different data is a different model.
#[tauri::command]
async fn apply_ml_model(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    req: ml::MlApplyRequest,
) -> Result<ml::MlResult, String> {
    let items = {
        let conn = db.0.lock().unwrap();
        well_items(&conn, &req.apply_well_ids)
    };
    let total = req.apply_well_ids.len();
    let conn = db.0.clone();
    let reg = jobs_reg.inner().clone();
    jobs::run_job(reg, "Machine learning", String::from("apply saved model"), items, total, true, move |job| {
        ml::apply_ml_model(&conn, &req, Some(&job))
    })
    .await
}

/// Every saved model, newest first. Never carries the model bytes — a random forest is megabytes
/// and the picker only needs the description.
#[tauri::command]
async fn list_ml_models(db: tauri::State<'_, DbState>) -> Result<Vec<db::MlModelInfo>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let c = conn.lock().unwrap();
        db::list_ml_models(&c).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// SB-MLA-002 + SB-MLA-005 — what each saved model would be warned about IF it were applied now,
/// answered before it is.
///
/// Both requirements ask for a warning "before the model is applied", and an apply run cannot give
/// either one early: it learns the runtime from a reply header that arrives after the prediction,
/// and by then the curves are written. So the same two checks the apply path runs are asked here,
/// at the moment somebody is looking at a list of models deciding which to push across a field.
///
/// Computed in Rust rather than compared in the picker so there is ONE implementation of each check
/// and one wording. A model list is short and the runtime probe is cached, so this is one query and
/// no subprocess after the first call.
/// How each curve is SAMPLED, against the frame every read aligns onto.
///
/// Answers the question the coverage numbers cannot: a curve reported as blank everywhere is either
/// absent or delivered on a grid that coincides with the frame at no depth, and those call for
/// opposite responses. Measured per well because sampling is a property of the delivery, and a field
/// where one well came from a different vendor is exactly where this bites.
/// The competing shipped values the corpus records for one parameter (`SB-CORE-013`).
///
/// Synchronous and connection-free on purpose: it is a static table, it is read while a dialog is
/// being built, and a round-trip through `spawn_blocking` for four struct literals would be the only
/// slow thing about opening a parameter editor.
#[tauri::command]
fn param_sources(topic: String) -> Vec<param_sources::ParamSource> {
    param_sources::sources_for(&topic).to_vec()
}

#[tauri::command]
async fn curve_sampling(
    db: tauri::State<'_, DbState>,
    well_ids: Vec<String>,
    curves: Vec<String>,
) -> Result<Vec<(String, Vec<equations::CurveSampling>)>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let c = conn.lock().unwrap();
        let mut out = Vec::new();
        for id in &well_ids {
            let frame = equations::well_frame(&c, id).unwrap_or_default();
            match equations::curve_sampling(&c, id, &curves, &frame) {
                Ok(rows) => out.push((id.clone(), rows)),
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// SB-MLA-008 — what about THIS configuration would not reproduce elsewhere, before it is run.
///
/// Scoped to what the product can observe in its own code rather than to second-hand claims about
/// library determinism: today that is the `gbdt` estimator substitution. `None` is the ordinary
/// answer and means the run is reproducible from its own record under the same runtime.
#[tauri::command]
async fn ml_determinism_note(task: String, algorithm: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || ml::determinism_note(&task, &algorithm))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ml_model_warnings(db: tauri::State<'_, DbState>) -> Result<Vec<ml::ModelWarnings>, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let c = conn.lock().unwrap();
        Ok(ml::model_warnings(&c))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn rename_ml_model(db: tauri::State<'_, DbState>, model_id: String, new_name: String) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    db::rename_ml_model(&conn, &model_id, &new_name).map_err(|e| e.to_string())
}

/// SB-MLA-007. Which delivered curves name this model as the thing that produced them.
#[tauri::command]
fn ml_model_citations(db: tauri::State<'_, DbState>, model_id: String) -> Vec<ml::ModelCitation> {
    let conn = db.0.lock().unwrap();
    ml::model_citations(&conn, &model_id)
}

/// SB-MLA-007 — a model a delivered curve cites cannot be deleted SILENTLY.
///
/// Deleting one corrupts nothing; the curve keeps its numbers. It does something quieter: the curve
/// goes on citing a model id that resolves to nothing, so the provenance block in a report names a
/// model nobody can produce, and the failure surfaces in front of a client months later as a
/// question that cannot be answered.
///
/// It REFUSES by default and names what would be orphaned. `force` is the user's own decision, taken
/// after reading that list — which is the difference between a deletion that was chosen and one that
/// merely happened.
#[tauri::command]
fn delete_ml_model(
    db: tauri::State<'_, DbState>,
    model_id: String,
    force: Option<bool>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    if !force.unwrap_or(false) {
        let cited = ml::model_citations(&conn, &model_id);
        if !cited.is_empty() {
            let where_ = cited
                .iter()
                .take(4)
                .map(|c| format!("{} ({}: {})", c.well_name, c.set_name, c.curves.join(", ")))
                .collect::<Vec<_>>()
                .join("; ");
            let more = cited.len().saturating_sub(4);
            return Err(format!(
                "{} live curve set(s) name this model as what produced them - {where_}{}. Deleting it \
                 leaves those curves citing a model that no longer exists, which is a provenance \
                 block in a report naming something nobody can produce. Delete it anyway only if you \
                 mean to.",
                cited.len(),
                if more > 0 { format!(", and {more} more") } else { String::new() }
            ));
        }
    }
    db::delete_ml_model(&conn, &model_id).map_err(|e| e.to_string())
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

/// Fits the RtC excess-conductivity coefficients (A_CAP / B_QV / C0) to the user's OWN
/// water-bearing interval, so `sw_rtc` stops running on a calibration from somebody else's
/// field. Refuses unless a water zone is declared — see `lrlc::run_rtc_fit`. Off-thread;
/// writes no curves.
#[tauri::command]
async fn run_rtc_fit(
    db: tauri::State<'_, DbState>,
    req: lrlc::RtcFitRequest,
) -> Result<lrlc::RtcFitResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || lrlc::run_rtc_fit(&conn, &req))
        .await
        .map_err(|e| e.to_string())
}

/// Fits `sw_imts`'s CEC scaling factor S to the user's OWN laboratory CEC measurements, against
/// the clay content of the very curves the run will use — see `lrlc::run_s_factor_fit`.
/// Off-thread; writes no curves.
#[tauri::command]
async fn run_s_factor_fit(
    db: tauri::State<'_, DbState>,
    req: lrlc::SFactorFitRequest,
) -> Result<lrlc::SFactorFitResult, String> {
    let conn = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || lrlc::run_s_factor_fit(&conn, &req))
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
    jobs::run_job(reg, "SandiMin", "mineral solver".to_string(), items, total, true, move |job| {
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

/// Wet-clay → dry-clay endpoint conversion (wet/dry clay xlsx workflow) for the
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
    compartment: Option<String>,
    zones: Option<Vec<String>>,
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
        compartment.as_deref(),
        &zones.unwrap_or_default(),
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

/// Every whole-well parameter override in the project, for the per-well parameter grid.
#[tauri::command]
fn list_well_param_overrides(db: tauri::State<DbState>) -> Result<Vec<db::WellParamOverride>, String> {
    let conn = db.0.lock().unwrap();
    db::list_well_param_overrides(&conn).map_err(|e| e.to_string())
}

/// Applies a batch of whole-well parameter overrides atomically; a null value clears one.
/// The grid's fill/paste actions and their undo all come through here.
#[tauri::command]
fn set_well_param_overrides(
    db: tauri::State<DbState>,
    entries: Vec<(String, String, Option<f32>)>,
) -> Result<usize, String> {
    let mut conn = db.0.lock().unwrap();
    db::set_well_param_overrides(&mut conn, &entries).map_err(|e| e.to_string())
}

/// Applies a batch of parameter overrides at one zone scope atomically (`*` = whole well).
/// An accepted calibration comes through here, so every well it is written to gets the whole
/// coefficient set or none of it.
#[tauri::command]
fn set_zone_param_batch(
    db: tauri::State<DbState>,
    zone_name: String,
    entries: Vec<(String, String, Option<f32>)>,
) -> Result<usize, String> {
    let mut conn = db.0.lock().unwrap();
    db::set_zone_param_batch(&mut conn, &zone_name, &entries).map_err(|e| e.to_string())
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

/// Shifts a well's ACTIVE core delivery by `delta` (core-to-log alignment) — the plugs and the
/// extras that rode in with them, together. Returns both counts so the caller can say so.
#[tauri::command]
fn shift_core_data(
    db: tauri::State<DbState>,
    well_id: String,
    delta: f32,
    targets: Option<db::ShiftTargets>,
    note: Option<db::RegistrationNote>,
) -> Result<db::CoreShiftCounts, String> {
    let mut conn = db.0.lock().unwrap();
    // Nothing given = the extras that provably came in on this core. An explicit empty set means
    // "plugs only", and must stay distinguishable from "not specified".
    let targets = match targets {
        Some(t) => t,
        None => db::ShiftTargets::aux(
            db::core_extra_datasets(&conn, &well_id)
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|(d, _)| d)
                .collect(),
        ),
    };
    db::shift_core_depths(&mut conn, &well_id, delta, &targets, &note.unwrap_or_default())
        .map_err(|e| e.to_string())
}

/// Applies per-barrel (or finer) corrections to a well's active core delivery. Refuses any set
/// that would reorder the core, and changes nothing when it does.
#[tauri::command]
fn apply_core_run_shifts(
    db: tauri::State<DbState>,
    well_id: String,
    runs: Vec<db::RunShift>,
    targets: Option<db::ShiftTargets>,
    note: Option<db::RegistrationNote>,
) -> Result<db::CoreShiftCounts, String> {
    let mut conn = db.0.lock().unwrap();
    let targets = match targets {
        Some(t) => t,
        None => db::ShiftTargets::aux(
            db::core_extra_datasets(&conn, &well_id)
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|(d, _)| d)
                .collect(),
        ),
    };
    db::apply_core_run_shifts(&mut conn, &well_id, &runs, &targets, &note.unwrap_or_default())
}

/// A well's core depth history: every shift ever applied, why, and how well it agreed. An event
/// log, so an undo appears as its own reversal rather than erasing what it reversed.
#[tauri::command]
fn list_core_registrations(
    db: tauri::State<DbState>,
    well_id: String,
) -> Result<Vec<db::RegistrationEntry>, String> {
    let conn = db.0.lock().unwrap();
    db::list_core_registrations(&conn, &well_id).map_err(|e| e.to_string())
}

/// Everything in a well that a core registration could carry with it, with whether each delivery
/// was imported as sitting on the core depth scale.
#[tauri::command]
fn core_shift_candidates(db: tauri::State<DbState>, well_id: String) -> Result<Vec<db::ShiftCandidate>, String> {
    let conn = db.0.lock().unwrap();
    db::core_shift_candidates(&conn, &well_id).map_err(|e| e.to_string())
}

/// A well's core depth record: `(depth the lab wrote, depth it sits at now)` per plug.
#[tauri::command]
fn core_depth_pairs(db: tauri::State<DbState>, well_id: String) -> Result<Vec<(f32, f32)>, String> {
    let conn = db.0.lock().unwrap();
    db::core_depth_pairs(&conn, &well_id).map_err(|e| e.to_string())
}

/// Maps depths written by a laboratory onto where that rock now sits, using the core's own
/// record. Returns one `(depth, extrapolated)` per input, so a caller can show which samples fell
/// outside the cored interval and were therefore guessed rather than measured.
#[tauri::command]
fn map_core_depths(
    db: tauri::State<DbState>,
    well_id: String,
    depths: Vec<f32>,
) -> Result<Vec<(f32, bool)>, String> {
    let conn = db.0.lock().unwrap();
    let pairs = db::core_depth_pairs(&conn, &well_id).map_err(|e| e.to_string())?;
    Ok(depths.into_iter().map(|d| db::map_core_depth(&pairs, d)).collect())
}

/// Moves a whole plate delivery by a constant depth (re-registering pictures).
#[tauri::command]
fn shift_well_images(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: Option<String>,
    delta: f32,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::shift_well_images(&conn, &well_id, dataset.as_deref(), delta).map_err(|e| e.to_string())
}

/// Can the pore measurement run? Probed once so a dialog can say what is missing before a run,
/// rather than failing at the end of one.
#[tauri::command]
async fn pore_support() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(petrography::pore_support)
        .await
        .map_err(|e| e.to_string())?
}

/// Pore area from blue-dyed epoxy, over a well's live image delivery. Refuses any plate not
/// declared impregnated, by name — the measurement would otherwise succeed on it and return a
/// porosity assembled from blue-ish grains.
#[tauri::command]
async fn run_pore_area(
    db: tauri::State<'_, DbState>,
    spec: petrography::PoreSpec,
) -> Result<petrography::PoreResult, String> {
    let conn = db.0.lock().unwrap();
    petrography::run_pore_area(&conn, &spec)
}

/// Is scikit-learn reachable? Probed so the dialog can say what is missing before a run.
#[tauri::command]
async fn classify_support() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(petrography::classify_support)
        .await
        .map_err(|e| e.to_string())?
}

/// Trains a per-pixel mineral classifier on the user's own clicks and applies it to the delivery.
#[tauri::command]
async fn run_plate_classifier(
    db: tauri::State<'_, DbState>,
    spec: petrography::ClassifySpec,
) -> Result<petrography::ClassifyResult, String> {
    let conn = db.0.lock().unwrap();
    petrography::run_plate_classifier(&conn, &spec)
}

/// The published stain schemes this build ships, as (name, classes). Mineral identifications are
/// standard carbonate petrography; the colour bands are round starting points for visual tuning.
#[tauri::command]
fn stain_schemes() -> Vec<(String, Vec<petrography::StainClass>)> {
    petrography::stain_scheme_names()
        .into_iter()
        .filter_map(|n| petrography::stain_scheme(&n).map(|c| (n, c)))
        .collect()
}

/// What the two axis pickers of the plug QC pane can offer over the wells in scope.
#[tauri::command]
fn list_plug_choices(
    db: tauri::State<DbState>,
    well_ids: Vec<String>,
) -> Result<Vec<plugqc::PlugChoice>, String> {
    let conn = db.0.lock().unwrap();
    plugqc::list_plug_choices(&conn, &well_ids)
}

/// Pairs two plug-scale measurements by depth across the scoped wells. Long enough on a field of
/// cored wells to be worth keeping off the event loop.
#[tauri::command]
async fn run_plug_qc(
    db: tauri::State<'_, DbState>,
    req: plugqc::PlugQcRequest,
) -> Result<plugqc::PlugQcResult, String> {
    let conn = db.0.lock().unwrap();
    plugqc::run_plug_qc(&conn, &req)
}

/// Resamples a log set onto a different sampling as a NEW set (`reframe.rs`).
///
/// `preview` computes and reports without writing, which is how the pane can show the source's own
/// sampling beside the target before anything is committed — the number the user is actually
/// deciding against, and one nothing else in the app displays.
#[tauri::command]
async fn run_reframe(
    db: tauri::State<'_, DbState>,
    req: reframe::ReframeRequest,
) -> Result<Vec<reframe::ReframeResult>, String> {
    let conn = db.0.lock().unwrap();
    Ok(reframe::run_reframe(&conn, &req))
}

/// Exact mnemonics the selected Reframe source can offer as a substitute. This list never expands
/// by family/type: the person must be shown the actual curve whose data would be used.
#[tauri::command]
fn reframe_source_curves(
    db: tauri::State<DbState>,
    well_id: String,
    source: reframe::SourceSpec,
) -> Result<Vec<String>, String> {
    let conn = db.0.lock().unwrap();
    reframe::source_curve_names(&conn, &well_id, &source)
}

#[tauri::command]
fn save_curve_selection(
    db: tauri::State<DbState>,
    selection: reframe::CurveSelection,
) -> Result<reframe::CurveSelection, String> {
    let conn = db.0.lock().unwrap();
    reframe::save_curve_selection(&conn, &selection)
}

#[tauri::command]
fn list_curve_selections(db: tauri::State<DbState>) -> Result<Vec<reframe::CurveSelection>, String> {
    let conn = db.0.lock().unwrap();
    reframe::list_curve_selections(&conn)
}

#[tauri::command]
fn delete_curve_selection(db: tauri::State<DbState>, name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    reframe::delete_curve_selection(&conn, &name)
}

/// Are numpy and Pillow reachable? Probed once so the conditioning workspace can say what is
/// missing before a photograph is opened rather than after a slider is moved.
#[tauri::command]
async fn core_image_support() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(coreimage::core_image_support)
        .await
        .map_err(|e| e.to_string())?
}

/// One core photograph rendered at preview size under a recipe, with the un-conditioned proxy and
/// a histogram beside it. Writes nothing — tuning must not leave half-judged pictures in a project.
#[tauri::command]
async fn preview_core_image(
    db: tauri::State<'_, DbState>,
    image_id: String,
    recipe: coreimage::CoreRecipe,
    pick_x: Option<f32>,
    pick_y: Option<f32>,
) -> Result<coreimage::CorePreview, String> {
    let conn = db.0.lock().unwrap();
    let pick = pick_x.zip(pick_y);
    coreimage::preview_core_image(&conn, &image_id, &recipe, pick)
}

/// Bakes recipes into pictures, keeping each import so the conditioning stays reversible.
#[tauri::command]
async fn bake_core_images(
    db: tauri::State<'_, DbState>,
    items: Vec<coreimage::BakeItem>,
) -> Result<coreimage::BakeResult, String> {
    let conn = db.0.lock().unwrap();
    coreimage::bake_core_images(&conn, &items)
}

/// Reads the proxy measures off a well's live core-photograph delivery, and (optionally) writes
/// them as curves. Long enough on a full core to be worth keeping off the event loop.
#[tauri::command]
async fn extract_core_log(
    db: tauri::State<'_, DbState>,
    spec: coreimage::CoreLogSpec,
) -> Result<coreimage::CoreLogResult, String> {
    let conn = db.0.lock().unwrap();
    coreimage::extract_core_log(&conn, &spec)
}

/// Proposes where the runs of core are inside one packed photograph — the columns of a core-display
/// plate, the rows of a core box. Proposes only: the lay-out lands in an editable table.
#[tauri::command]
async fn detect_core_lanes(
    db: tauri::State<'_, DbState>,
    image_id: String,
    axis: String,
    reverse: bool,
) -> Result<coreimage::LaneDetection, String> {
    let conn = db.0.lock().unwrap();
    coreimage::detect_core_lanes(&conn, &image_id, &axis, reverse)
}

/// Measures a delivery and proposes conditioning for each picture, with the reason for every value.
/// Decodes every picture, so it stays off the event loop.
#[tauri::command]
async fn recommend_core_recipe(
    db: tauri::State<'_, DbState>,
    image_ids: Vec<String>,
) -> Result<Vec<coreimage::RecipeAdvice>, String> {
    let conn = db.0.lock().unwrap();
    coreimage::recommend_core_recipe(&conn, &image_ids)
}

/// Cuts a core-photograph delivery into rows and stacks each box into one depth-registered strip.
/// Decodes and re-encodes every picture in the delivery, so it stays off the event loop.
#[tauri::command]
async fn build_core_strips(
    db: tauri::State<'_, DbState>,
    spec: coreimage::StripSpec,
) -> Result<coreimage::StripResult, String> {
    let conn = db.0.lock().unwrap();
    coreimage::build_core_strips(&conn, &spec)
}

/// Copies one photograph's look across a whole live delivery, keeping each picture's own framing.
#[tauri::command]
async fn apply_core_look(
    db: tauri::State<'_, DbState>,
    well_id: String,
    dataset: String,
    look: coreimage::CoreRecipe,
) -> Result<coreimage::BakeResult, String> {
    let conn = db.0.lock().unwrap();
    coreimage::apply_look_to_delivery(&conn, &well_id, &dataset, &look)
}

/// The conditioning recipe of every picture in a dataset's live delivery. Never reads a blob, so
/// the workspace can show which photographs have been touched without fetching any of them.
#[tauri::command]
fn list_image_recipes(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
) -> Result<Vec<(String, String)>, String> {
    let conn = db.0.lock().unwrap();
    db::list_image_recipes(&conn, &well_id, &dataset).map_err(|e| e.to_string())
}

/// One plate's field of view and preparation. Every value is written as given, `null` included —
/// a scale typed by mistake has to be clearable.
#[tauri::command]
fn set_image_details(
    db: tauri::State<DbState>,
    image_id: String,
    fov_um: Option<f32>,
    prepared: Option<String>,
    stain: Option<String>,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::set_image_details(&conn, &image_id, fov_um, prepared.as_deref(), stain.as_deref())
        .map_err(|e| e.to_string())
}

/// The same three facts across a whole live image delivery, in one statement.
#[tauri::command]
fn set_image_delivery_details(
    db: tauri::State<DbState>,
    well_id: String,
    dataset: String,
    fov_um: Option<f32>,
    prepared: Option<String>,
    stain: Option<String>,
) -> Result<usize, String> {
    let conn = db.0.lock().unwrap();
    db::set_image_delivery_details(&conn, &well_id, &dataset, fov_um, prepared.as_deref(), stain.as_deref())
        .map_err(|e| e.to_string())
}

/// Point datasets delivered as part of this well's active core table — what a depth shift
/// should move along with the plugs.
#[tauri::command]
fn core_extra_datasets(db: tauri::State<DbState>, well_id: String) -> Result<Vec<(String, i64)>, String> {
    let conn = db.0.lock().unwrap();
    db::core_extra_datasets(&conn, &well_id).map_err(|e| e.to_string())
}

/// Everything in a well that could anchor a core-to-log registration.
#[tauri::command]
fn list_core_references(db: tauri::State<DbState>, well_id: String) -> Result<Vec<registration::CoreReference>, String> {
    let conn = db.0.lock().unwrap();
    registration::list_core_references(&conn, &well_id)
}

/// Proposes the depth shift that best aligns a well's core with a log. Writes nothing.
#[tauri::command]
async fn propose_registration(
    db: tauri::State<'_, DbState>,
    req: registration::RegistrationRequest,
) -> Result<registration::RegistrationResult, String> {
    let mx = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || registration::propose_registration(&mx, &req))
        .await
        .map_err(|e| e.to_string())
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

/// Cross-well consistency for one contact type IN ONE MARKER: fits a flat-TVDSS surface through
/// the picked contacts and flags wells that disagree. `zone_name` omitted checks the contacts that
/// state no marker — it does not mean "every marker". Read-only.
#[tauri::command]
fn check_contact_consistency(
    db: tauri::State<DbState>,
    contact_type: String,
    compartment: Option<String>,
    zones: Option<Vec<String>>,
    flag_abs: Option<f32>,
) -> Result<contacts::ContactConsistency, String> {
    let conn = db.0.lock().unwrap();
    Ok(contacts::check_contact_consistency(
        &conn,
        &contact_type,
        compartment.as_deref(),
        &zones.unwrap_or_default(),
        flag_abs.unwrap_or(3.0),
    ))
}

/// Every (contact type, marker) pair in the project, so a QC pane can check them all.
#[tauri::command]
fn contact_groups(db: tauri::State<DbState>) -> Result<Vec<contacts::ContactGroup>, String> {
    let conn = db.0.lock().unwrap();
    Ok(contacts::contact_groups(&conn))
}

/// Compares each marker-tagged FWL contact against the parameter a saturation-height run reads.
#[tauri::command]
fn check_fwl_agreement(
    db: tauri::State<DbState>,
    tolerance: Option<f32>,
) -> Result<Vec<contacts::FwlCheck>, String> {
    let conn = db.0.lock().unwrap();
    Ok(contacts::check_fwl_agreement(&conn, tolerance.unwrap_or(0.1)))
}

/// Copies picked FWL contacts into `zone_params`, so the arithmetic reads what the panel draws.
/// One transaction per marker, and undoable from the caller like any other parameter write.
#[tauri::command]
fn apply_fwl_to_zone_params(
    db: tauri::State<DbState>,
    picks: Vec<(String, String, f32)>,
) -> Result<usize, String> {
    let mut conn = db.0.lock().unwrap();
    contacts::apply_fwl_to_zone_params(&mut conn, &picks)
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

/// Read-only SQL over the project database (full DuckDB SQL, SELECT-only). Async: the user
/// writes the query, so its cost is unbounded — a join over a field-scale `computed_curves`
/// must not freeze the window (see `open_project`).
#[tauri::command]
async fn run_query(
    db: tauri::State<'_, DbState>,
    sql: String,
    limit: usize,
) -> Result<db::TablePage, String> {
    let handle = db.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = handle.lock().unwrap();
        db::run_readonly_query(&conn, &sql, limit)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Export formats and their sentinel capability, including the one unambiguous default.
#[tauri::command]
fn list_data_export_formats() -> Vec<export::ExportFormatInfo> {
    export::export_formats()
}

/// Exports one well as LAS 2.0 and returns rows, held/written curve counts and named omissions.
#[tauri::command]
async fn export_las(
    db: tauri::State<'_, DbState>,
    jobs_reg: tauri::State<'_, jobs::JobRegistry>,
    well_id: String,
    dest_path: String,
) -> Result<export::LasExportResult, String> {
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
    // Cancellable: the chain re-reads the raw flag between steps and `run_workflow_module_into`
    // drains the wells on it, marking the observation via `note_cancel_observed`.
    let job = jobs::register(jobs_reg.inner(), uuid, "Workflow chain", label, items, cancel.clone(), true);
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
    // rayon under this same Mutex, so cross-thread use is sound.
    //
    // A panic inside stays on this thread (it can't abort the process) — but "the job simply
    // stops reporting progress" understated it badly. The chain registry has no prune, so an
    // entry that never reaches a terminal status stays Running forever and `chain::any_active`
    // shuts Open/New/Compact Project for the rest of the session (review_triage finding 17). So
    // catch it: report the panic on both registries, which tells the user AND releases the guard.
    //
    // Honest limit: if the panic happened while the DB mutex was held, that mutex is now poisoned
    // and the next `lock().unwrap()` anywhere panics in turn. Catching here cannot rescue that
    // case — it rescues every panic that was not holding the lock, and makes the rest report.
    std::thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
        }));
        if let Err(payload) = outcome {
            // `panic!("literal")` carries a &str, `panic!("{x}")` a String; anything else has no
            // readable message at all. Say so rather than printing a type name.
            let detail = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "no message".to_string());
            let msg = format!("the workflow stopped unexpectedly ({detail}) — its results are incomplete");
            chain::failed(&registry, uuid, msg.clone());
            job.failed(msg);
        }
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

/// Opens the startup project and installs it as the live connection. Runs on a background
/// thread so the window exists first — see `run` for why that matters.
///
/// Everything here used to happen BEFORE `tauri::Builder`, which is what made a slow first
/// open look like a dead application: on a 2.5 GB field project the one-time storage
/// migrations took ~15 minutes, and for all of it the user had double-clicked SandiBumi and
/// gotten no window at all. The recovery ladder is unchanged (project → temp recovery file →
/// memory-only), it just publishes its outcome instead of returning it.
fn open_startup_project(handle: tauri::AppHandle, startup: String) {
    // `tauri dev` restarts this binary on every source-file change, so a WAL replay failure
    // (killed mid-write) happens occasionally during development — self-heal rather than
    // crash-loop. The per-step timings inside `open_and_migrate` say which step dominates.
    let boot = std::time::Instant::now();
    let (conn, problem) = match project::open_and_migrate(&startup) {
        Ok(conn) => {
            project::register_recent(&startup);
            (conn, None)
        }
        Err(message) => {
            // Never abort — see `StartupProblem`. The file that failed is left completely
            // untouched, and deliberately NOT registered in the recents: a project that would
            // not open should not be the first thing we try again at the next launch.
            eprintln!("[boot] {message}");
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let recovery = std::env::temp_dir()
                .join(format!("sandibumi-recovery-{stamp}.duckdb"))
                .to_string_lossy()
                .into_owned();
            match project::open_and_migrate(&recovery) {
                Ok(conn) => (
                    conn,
                    Some(StartupProblem {
                        attempted_path: startup.clone(),
                        message,
                        recovered_to: recovery,
                        recovery_persists: true,
                    }),
                ),
                Err(second) => {
                    // Even the temp directory is unusable. Memory-only still beats a silent
                    // death: the user gets a window, can read why, and can open a different
                    // project. The UI is told nothing will persist. This keeps the placeholder
                    // the window already booted on, so there is nothing to swap in.
                    eprintln!("[boot] recovery project also failed: {second}");
                    let problem = StartupProblem {
                        attempted_path: startup.clone(),
                        message,
                        recovered_to: String::new(),
                        recovery_persists: false,
                    };
                    publish_open_outcome(&handle, None, Some(problem), String::new(), boot);
                    return;
                }
            }
        }
    };
    eprintln!("[boot] total background DB init: {:?}", boot.elapsed());

    // "Save As" copies whatever file the connection is actually on, so this follows the
    // recovery rather than the intent — otherwise a recovered session would copy the project
    // that never opened.
    let path = match &problem {
        Some(p) if p.recovery_persists => p.recovered_to.clone(),
        Some(_) => String::new(),
        None => project::absolute(&startup),
    };
    publish_open_outcome(&handle, Some(conn), problem, path, boot);
}

/// Installs the freshly opened connection (if any) and wakes `await_project_open`. Ordering
/// matters: the connection, project path and problem are all in place BEFORE the outcome is
/// published, so a frontend that resolves and immediately queries can never race the swap.
fn publish_open_outcome(
    handle: &tauri::AppHandle,
    conn: Option<Connection>,
    problem: Option<StartupProblem>,
    path: String,
    boot: std::time::Instant,
) {
    use tauri::Manager as _;
    if let Some(conn) = conn {
        if let Some(db) = handle.try_state::<DbState>() {
            *db.0.lock().unwrap() = conn;
        }
    }
    if let Some(proj) = handle.try_state::<project::ProjectState>() {
        *proj.0.lock().unwrap() = path.clone();
    }
    if let Some(state) = handle.try_state::<StartupState>() {
        *state.0.lock().unwrap() = problem.clone();
    }
    if let Some(init) = handle.try_state::<DbInit>() {
        store_outcome(
            &init.0,
            OpenOutcome { problem, elapsed_secs: boot.elapsed().as_secs(), path },
        );
    }
}

#[cfg(test)]
mod startup_gate_tests {
    use super::*;

    fn cell() -> Arc<OpenCell> {
        Arc::new((Mutex::new(None), std::sync::Condvar::new()))
    }

    fn outcome(path: &str) -> OpenOutcome {
        OpenOutcome { problem: None, elapsed_secs: 3, path: path.to_string() }
    }

    /// THE launch-hang guard. On a normal (fast) launch the background open finishes before
    /// the frontend calls `await_project_open` at all, so the wake-up fires with no waiter.
    /// A waiter arriving afterwards must return IMMEDIATELY from the stored value. If this
    /// ever regresses to a pure signal, every quick launch hangs on the boot overlay — with
    /// the project perfectly healthy behind it.
    #[test]
    fn fast_open_published_before_the_wait() {
        let c = cell();
        store_outcome(&c, outcome("already-open.duckdb"));
        let got = wait_for_outcome(&c);
        assert_eq!(got.path, "already-open.duckdb");
        // ...and it stays readable: the frontend may ask more than once.
        assert_eq!(wait_for_outcome(&c).path, "already-open.duckdb");
    }

    /// The slow case: the waiter arrives first and must block until the open publishes,
    /// then see the real answer (not a default).
    #[test]
    fn slow_open_wakes_a_waiter_already_blocked() {
        let c = cell();
        let writer = {
            let c = c.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(120));
                store_outcome(&c, outcome("field.duckdb"));
            })
        };
        let started = std::time::Instant::now();
        let got = wait_for_outcome(&c); // must actually block, not spin-return None
        assert!(started.elapsed().as_millis() >= 100, "must wait for the open, not return early");
        assert_eq!(got.path, "field.duckdb");
        writer.join().unwrap();
    }

    /// A failed open still publishes — otherwise the boot overlay would sit there forever
    /// instead of handing the frontend a problem to render.
    #[test]
    fn a_failed_open_still_releases_the_gate() {
        let c = cell();
        store_outcome(
            &c,
            OpenOutcome {
                problem: Some(StartupProblem {
                    attempted_path: "gone.duckdb".into(),
                    message: "could not open".into(),
                    recovered_to: String::new(),
                    recovery_persists: false,
                }),
                elapsed_secs: 0,
                path: String::new(),
            },
        );
        let got = wait_for_outcome(&c);
        assert!(got.problem.is_some(), "the frontend needs the problem to explain it");
        assert!(!got.problem.unwrap().recovery_persists);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The most recently opened project that still exists, else the legacy
    // `project.duckdb` in the cwd — so existing installs open exactly as before.
    let startup = project::startup_path();

    // The window is built on an EMPTY IN-MEMORY database and the real project is opened on a
    // background thread. This is what turns a slow first open from "the app didn't launch"
    // into "the app is open and telling me what it's doing". Nothing reads this placeholder:
    // the frontend awaits `await_project_open` before it builds a single panel, and until then
    // issues no other command. Creating it cannot fail (no filesystem involved), which is why
    // these are the only two expects left on the startup path.
    let placeholder =
        Connection::open_in_memory().expect("opening an in-memory DuckDB cannot fail");
    db::create_schema(&placeholder)
        .expect("creating the schema in a fresh in-memory DuckDB cannot fail");

    // No opener plugin: the app never hands a URL or path to the OS, and a granted-but-unused
    // capability is exactly what an enterprise security review asks about. If a future feature
    // needs to open an external link, re-add the plugin AND the `opener:default` capability
    // together, and revisit the zero-network-egress claim in docs/PRD.md section 7.5.
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DbState(Arc::new(Mutex::new(placeholder))))
        .manage(project::ProjectState(Mutex::new(String::new())))
        .manage(StartupState(Mutex::new(None)))
        .manage(DbInit(Arc::new((Mutex::new(None), std::sync::Condvar::new()))))
        .manage(chain::new_registry())
        .manage(jobs::new_registry())
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || open_startup_project(handle, startup));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            startup_problem,
            await_project_open,
            get_project_depth_unit,
            set_project_depth_unit,
            list_convertible_unit_families,
            get_project_null_sentinel,
            set_project_null_sentinel,
            save_project_as,
            compact_project,
            boot_report,
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
            update_curve_meta,
            import_deviation_csv,
            list_core_sets,
            list_surveys,
            set_active_core_set,
            delete_core_set,
            set_active_survey,
            delete_survey,
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
            module_output_names,
            run_reframe,
            reframe_source_curves,
            save_curve_selection,
            list_curve_selections,
            delete_curve_selection,
            run_pay_summary,
            stats_curve_summary,
            stats_pair_summary,
            stats_versus_sets,
            stats_thickness,
            stats_fit,
            intake_probe,
            intake_paste,
            intake_commit,
            intake_commit_arrays,
            intake_probe_arrays,
            intake_commit_curves,
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
            list_well_param_overrides,
            set_well_param_overrides,
            set_zone_param_batch,
            get_table_page,
            update_well_field,
            update_standard_sample,
            update_computed_sample,
            update_core_sample,
            shift_core_data,
            core_extra_datasets,
            core_shift_candidates,
            shift_well_images,
            apply_core_run_shifts,
            list_core_registrations,
            set_image_details,
            core_image_support,
            preview_core_image,
            bake_core_images,
            apply_core_look,
            extract_core_log,
            detect_core_lanes,
            recommend_core_recipe,
            build_core_strips,
            list_image_recipes,
            set_image_delivery_details,
            pore_support,
            run_pore_area,
            stain_schemes,
            classify_support,
            run_plate_classifier,
            list_plug_choices,
            run_plug_qc,
            core_depth_pairs,
            map_core_depths,
            list_core_references,
            propose_registration,
            edit_curve,
            restore_curve_values,
            upsert_top,
            delete_top,
            check_top_order,
            autocorrelate_top,
            autocorrelate_multi,
            suggest_contacts,
            check_contact_consistency,
            contact_groups,
            check_fwl_agreement,
            apply_fwl_to_zone_params,
            sw_method_spread,
            run_query,
            list_data_export_formats,
            export_las,
            python_status,
            run_ml,
            apply_ml_model,
            list_ml_models,
            ml_model_warnings,
            ml_determinism_note,
            curve_sampling,
            param_sources,
            rename_ml_model,
            delete_ml_model,
            ml_model_citations,
            run_ml_eval,
            run_cuddy_foil,
            run_shf_fit,
            run_rtc_fit,
            run_s_factor_fit,
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
            probe_core_table,
            import_core_table,
            import_tops_csv,
            import_well_locations,
            wells_in_polygon,
            import_aux_data,
            list_aux_data,
            list_aux_datasets,
            list_aux_item_catalog,
            probe_image_files,
            probe_plate_workbooks,
            image_support,
            import_well_images,
            list_well_images,
            list_image_datasets,
            get_well_image,
            list_image_sets,
            set_active_image_set,
            delete_image_set,
            delete_well_image,
            update_well_image,
            list_array_curves,
            get_array_log,
            delete_array_log,
            list_aux_sets,
            set_active_aux_set,
            delete_aux_set,
            import_scal_csv,
            import_scal_files,
            get_scal_pc,
            list_scal_sets,
            set_active_scal_set,
            delete_scal_set,
            render_composite,
            export_composite_svg,
            export_composite_pdf,
            render_report,
            export_report_pdf,
            export_report_batch,
            office_support,
            export_workbook,
            export_report_docx,
            export_report_docx_batch,
            export_deck,
            save_png,
            save_plot_pdf,
            get_core_data
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // R-C (data-loss on plain window close, found 2026-07-29 by the packaged-build
            // verification): Tauri exits via std::process::exit, which skips Rust
            // destructors — the DuckDB connection never closes cleanly, so every close
            // leaves an unflushed WAL. Reproduced twice: import 20 rows, close with the
            // window ✕, relaunch → WAL fails replay, recovery moves it aside as
            // `.corrupt-backup-<ts>`, and the import is silently gone. Writes below the
            // auto-checkpoint threshold live ONLY in the WAL, so small edits (an import,
            // a parameter change) are exactly the writes at risk. Flushing here turns
            // every graceful exit into a clean checkpoint; force-kills stay covered by
            // `init_db_resilient` as before.
            if let tauri::RunEvent::Exit = event {
                use tauri::Manager as _;
                if let Some(db) = app_handle.try_state::<DbState>() {
                    if let Ok(conn) = db.0.lock() {
                        let _ = conn.execute_batch("CHECKPOINT;");
                    }
                }
            }
        });
}
