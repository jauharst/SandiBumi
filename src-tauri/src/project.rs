//! Project management (ROADMAP §4c item 2, "IP style"): which DuckDB file is open,
//! plus a recent-projects list kept OUTSIDE the project database — in the per-user
//! config dir — so it survives switching projects and lists them at next launch.
//!
//! The live connection is swapped in place under the existing `DbState` mutex: every
//! command locks that mutex per call, so a swap between calls is invisible to them.
//! The old database is CHECKPOINTed before the swap so its WAL is flushed.

use crate::db;
use crate::DbState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Fallback project file when there is no usable recents entry — the pre-picker
/// behaviour (relative to the process cwd, i.e. `src-tauri/` in dev).
pub const LEGACY_DEFAULT: &str = "project.duckdb";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    /// Unix seconds of the last successful open (display only).
    pub last_opened: u64,
    /// Whether the file still exists on disk — recomputed on every list, never trusted
    /// from the stored JSON.
    #[serde(default)]
    pub exists: bool,
}

/// Managed Tauri state: absolute path of the currently open project file.
pub struct ProjectState(pub std::sync::Mutex<String>);

pub(crate) fn config_dir() -> PathBuf {
    // Test override — unit tests point this at a temp dir so they never touch the
    // real per-user list.
    if let Ok(dir) = std::env::var("SANDIBUMI_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("SandiBumi")
}

fn recents_path() -> PathBuf {
    config_dir().join("projects.json")
}

fn load_recents() -> Vec<RecentProject> {
    let Ok(text) = std::fs::read_to_string(recents_path()) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn save_recents(list: &[RecentProject]) {
    if std::fs::create_dir_all(config_dir()).is_err() {
        return; // recents are a convenience — never fail an open over them
    }
    if let Ok(json) = serde_json::to_string_pretty(list) {
        let _ = std::fs::write(recents_path(), json);
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Display name of a project = its file stem ("balam.duckdb" → "balam").
pub fn project_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Absolute form of `path` for stable recents entries. `canonicalize` on Windows
/// returns an extended-length `\\?\C:\…` path — strip that prefix for display.
pub fn absolute(path: &str) -> String {
    match std::fs::canonicalize(path) {
        Ok(p) => {
            let s = p.to_string_lossy().into_owned();
            s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s)
        }
        Err(_) => path.to_string(),
    }
}

fn same_path(a: &str, b: &str) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        // At least one side doesn't exist — fall back to a case-insensitive compare
        // (Windows filesystems are case-insensitive).
        _ => a.eq_ignore_ascii_case(b),
    }
}

/// Moves/adds `path` to the top of the recents list (deduped, capped at 12).
pub fn register_recent(path: &str) {
    let abs = absolute(path);
    let mut list = load_recents();
    list.retain(|r| !same_path(&r.path, &abs));
    list.insert(
        0,
        RecentProject {
            name: project_name(&abs),
            path: abs,
            last_opened: now_secs(),
            exists: true,
        },
    );
    list.truncate(12);
    save_recents(&list);
}

/// The recents list for the UI, `exists` recomputed per entry.
pub fn list_recents() -> Vec<RecentProject> {
    let mut list = load_recents();
    for r in &mut list {
        r.exists = Path::new(&r.path).exists();
    }
    list
}

fn trusted_path() -> PathBuf {
    config_dir().join("trusted-code.json")
}

fn load_trusted() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(trusted_path()) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Whether this machine has already been told what code lives inside `path`.
///
/// A project file is not inert. Alongside the curves it carries saved equations and saved ML
/// models, and both are INSTRUCTIONS rather than numbers — a model is a joblib pickle, which
/// executes code the moment it is loaded, before any of the checks around it run. So a project
/// that arrived from somebody else is an attachment, not a data file, and the app says so once
/// before running anything out of it (SECURITY-REVIEW-2026-08-22 finding F1).
///
/// **The list lives HERE, on this machine, and deliberately NOT inside the project.** A marker
/// written into the file would travel with it, so a project passed between two operators would
/// carry a trace of where it had been. Nothing about who made a project may ride along inside it
/// — the same rule the repository keeps about client identifiers, pointed at file metadata.
pub fn project_code_is_trusted(path: &str) -> bool {
    let abs = absolute(path);
    load_trusted().iter().any(|known| same_path(known, &abs))
}

/// Record that this machine has been told about `path`'s saved code, so the notice is shown once
/// rather than every time. Called when the user acknowledges it, and when the app CREATES a
/// project — code you saved into your own new project is not code that arrived from anywhere.
pub fn trust_project_code(path: &str) {
    let abs = absolute(path);
    let mut list = load_trusted();
    if list.iter().any(|known| same_path(known, &abs)) {
        return;
    }
    list.push(abs);
    // Same convenience rule as the recents: never fail an operation over this file.
    if std::fs::create_dir_all(config_dir()).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string_pretty(&list) {
        let _ = std::fs::write(trusted_path(), json);
    }
}

/// The path the app opens at startup: the most recently opened project that still
/// exists, else the legacy `project.duckdb` next to the process cwd.
pub fn startup_path() -> String {
    for r in load_recents() {
        if Path::new(&r.path).exists() {
            return r.path;
        }
    }
    LEGACY_DEFAULT.to_string()
}

/// Opens (or, for a fresh file, creates) the DuckDB at `path` and brings it up to the current
/// schema. Shared by the runtime project switch and by startup, so the same file failing reports
/// the same reason either way — and, more importantly, so **startup can treat that failure as a
/// value instead of a panic**. A panic in `run()` happens before any window exists, and with
/// `panic = "abort"` plus `windows_subsystem = "windows"` it kills the process with no window, no
/// dialog and no console: the user double-clicks SandiBumi and nothing happens at all.
///
/// The per-step timings are the diagnostic for the ~5-minute open on the 540-well / ~2 GB file:
/// they say whether the DB open, the standard-curves backfill or the PK-drop check dominates.
/// They live here rather than at the call site so switching projects is measured too.
pub fn open_and_migrate(path: &str) -> Result<duckdb::Connection, String> {
    let t0 = std::time::Instant::now();
    let t = t0;
    let conn = db::init_db_resilient(path).map_err(|e| format!("could not open {path}: {e}"))?;
    // The only step not using `diagnostics::boot_step`: its console line names the project PATH,
    // which the diagnostic report must never carry. One measurement, printed with the path and
    // recorded without it.
    let opened = t.elapsed();
    eprintln!("[boot] init_db_resilient: {opened:?}  ({path})");
    crate::diagnostics::record_boot_step("init_db_resilient", opened.as_millis());

    let t = std::time::Instant::now();
    db::migrate_standard_curves_to_generic_store(&conn)
        .map_err(|e| format!("curve-store migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_standard_curves_to_generic_store", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_drop_computed_curves_pk(&conn, Some(path))
        .map_err(|e| format!("computed-curves migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_drop_computed_curves_pk", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_point_data_sets(&conn, Some(path))
        .map_err(|e| format!("point-data set migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_point_data_sets", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_array_logs_store(&conn).map_err(|e| format!("array-log migration failed: {e}"))?;
    // SB-DBM-009 / DEC-022: legacy WIB provenance timestamps become UTC instants, once.
    db::migrate_run_degradations_endpoint_invalid(&conn)
        .map_err(|e| format!("run-degradation vocabulary migration failed: {e}"))?;
    db::migrate_log_set_timestamps_to_utc(&conn)
        .map_err(|e| format!("timestamp migration failed: {e}"))?;
    // Provenance: scrubs the study-named Phase-0 stub table out of old project files.
    db::migrate_drop_study_named_stub(&conn)
        .map_err(|e| format!("stub-table migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_array_logs_store", t.elapsed());

    // DEC-089's second half. Must run AFTER migrate_standard_curves_to_generic_store, which is
    // what gives a pre-generic-store project a generic store to project FROM.
    let t = std::time::Instant::now();
    match crate::equations::migrate_standard_curves_canonical(&conn) {
        Ok(0) => {}
        Ok(n) => db::boot_note(format!(
            "Brought {n} standard curve(s) onto the project's canonical units, so the log view, plots and modules now read the same numbers (DEC-089)."
        )),
        // A project that cannot be re-projected still OPENS - it simply keeps the split it had,
        // which is exactly what it has been doing. Failing the launch over a display-domain
        // tidy-up would be the worse trade.
        Err(e) => db::boot_note(format!(
            "Standard-curve unit migration skipped: {e}. The project opens unchanged."
        )),
    }
    crate::diagnostics::boot_step("migrate_standard_curves_canonical", t.elapsed());

    // Must run AFTER migrate_point_data_sets, which rebuilds core_data for its primary key —
    // a column added before that rebuild would be dropped by it.
    let t = std::time::Instant::now();
    db::migrate_core_depth_orig(&conn).map_err(|e| format!("core depth-record migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_core_depth_orig", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_delivery_depth_basis(&conn)
        .map_err(|e| format!("delivery depth-basis migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_delivery_depth_basis", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_plate_scale_and_prep(&conn)
        .map_err(|e| format!("plate scale migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_plate_scale_and_prep", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_core_image_recipe(&conn)
        .map_err(|e| format!("core image recipe migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_core_image_recipe", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_fluid_contact_zone(&conn)
        .map_err(|e| format!("fluid contact marker migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_fluid_contact_zone", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_log_set_frame(&conn).map_err(|e| format!("log set frame migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_log_set_frame", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_array_log_axis(&conn).map_err(|e| format!("array log axis migration failed: {e}"))?;
    crate::diagnostics::boot_step("migrate_array_log_axis", t.elapsed());

    // A long open is almost always the one-time storage upgrades above (each backs up the
    // whole project first). Tell the user so — from their chair a silent 15-minute open on
    // a field-scale file is indistinguishable from a hang.
    let total = t0.elapsed();
    if total.as_secs() >= 10 {
        db::boot_note(format!(
            "Opening this project took {}s — one-time storage upgrades ran (the project was backed up first); the next open will be fast",
            total.as_secs()
        ));
    }

    // The dead-space meter: DuckDB never shrinks a file on DELETE, so a long-lived project
    // quietly drags (BLSO reached ~4x its live size before anyone noticed). Measured from
    // the engine's own block accounting at the one moment every project passes through.
    // Never fatal - a project that opens without the note is strictly better than one that
    // refuses to open over a meter reading.
    if let Ok((total_bytes, free_bytes)) = db::dead_space(&conn) {
        if let Some(note) = db::compact_suggestion(total_bytes, free_bytes) {
            db::boot_note(note);
        }
    }
    Ok(conn)
}

/// Result of "Compact Project": file sizes around the rewrite, plus where the original
/// (bloated) file was parked so the user can delete it once satisfied.
#[derive(Debug, Clone, Serialize)]
pub struct CompactReport {
    pub bytes_before: u64,
    pub bytes_after: u64,
    /// The pre-compaction original, kept beside the project as `<name>.pre-compact-<ts>.duckdb`.
    pub old_file: String,
}

/// Row-count comparison between the live database and the freshly written copy, over
/// EVERY table the live project holds (enumerated from the catalog, so a new store can
/// never silently fall outside the check). Cheap: DuckDB answers `count(*)` from table
/// metadata, not a scan.
fn verify_copy_counts(live: &duckdb::Connection, copy_path: &str) -> Result<(), String> {
    let copy = duckdb::Connection::open(copy_path).map_err(|e| format!("could not reopen the copy: {e}"))?;
    let tables: Vec<String> = {
        let mut stmt = live
            .prepare(
                "SELECT table_name FROM duckdb_tables()
                 WHERE database_name = current_database() AND NOT internal
                 ORDER BY table_name",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect()
    };
    for t in &tables {
        let q = format!("SELECT count(*) FROM \"{}\"", t.replace('"', "\"\""));
        let a: i64 = live.query_row(&q, [], |r| r.get(0)).map_err(|e| e.to_string())?;
        let b: i64 = copy.query_row(&q, [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if a != b {
            return Err(format!("copy verification failed: {t} has {b} rows in the copy but {a} in the project"));
        }
    }
    Ok(())
}

/// Rewrites the open project into a fresh file containing only live rows, then swaps it in
/// at the SAME path. Months of module re-runs (each a DELETE + append) leave dead space the
/// engine never returns to the file — one field project measured ~75% dead (2.5 GB file,
/// ~0.6 GB live) — and a bloated file drags every scan. The original is verified against the
/// copy (row counts), then parked beside the project as `.pre-compact-<ts>.duckdb`, never
/// deleted by us; on ANY failure the original is put back and the report says so.
pub fn compact_project(db: &DbState, path: &str) -> Result<CompactReport, String> {
    let bytes_before = std::fs::metadata(path).map_err(|e| format!("could not stat {path}: {e}"))?.len();
    let stem = path.strip_suffix(".duckdb").unwrap_or(path);
    let ts = now_secs();
    let tmp = format!("{stem}.compact-tmp-{ts}.duckdb");
    let old = format!("{stem}.pre-compact-{ts}.duckdb");

    // The mutex is held for the whole operation: every IPC command locks it per call, so
    // nothing can observe the placeholder connection mid-swap.
    let mut guard = db.0.lock().unwrap();
    db::engine_copy_to(&guard, &tmp).map_err(|e| format!("compact copy failed: {e}"))?;
    if let Err(e) = verify_copy_counts(&guard, &tmp) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    // Release the file handle so the swap can rename it: park a throwaway in-memory DB in
    // the state. Dropping the old connection closes gracefully (checkpoint + WAL removal).
    // Swap site 1 of 3 (#129 M1). The pool release matters here for a second reason beyond the
    // generation stamp: the rename below cannot move a file that something still holds open, so an
    // idle pooled handle would fail Compact Project outright on Windows.
    let placeholder = duckdb::Connection::open_in_memory().map_err(|e| e.to_string())?;
    drop(db.install(&mut guard, placeholder));

    if let Err(e) = std::fs::rename(path, &old) {
        let _ = std::fs::remove_file(&tmp);
        drop(db.install(&mut guard, open_and_migrate(path)?));
        return Err(format!("compact aborted (could not move the old file): {e} — the project is unchanged"));
    }
    // A leftover WAL belongs to the ORIGINAL file — it must follow it, or the compacted
    // file at this path would adopt and replay a foreign WAL on open.
    let wal = format!("{path}.wal");
    if Path::new(&wal).exists() {
        let _ = std::fs::rename(&wal, format!("{old}.wal"));
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::rename(format!("{old}.wal"), &wal);
        let _ = std::fs::rename(&old, path);
        drop(db.install(&mut guard, open_and_migrate(path)?));
        return Err(format!("compact aborted (could not move the compacted file into place): {e} — the project is unchanged"));
    }

    match open_and_migrate(path) {
        Ok(conn) => drop(db.install(&mut guard, conn)),
        Err(e) => {
            // The compacted copy verified on counts but failed to open — put everything back.
            let _ = std::fs::rename(path, &tmp);
            let _ = std::fs::rename(&old, path);
            let _ = std::fs::rename(format!("{old}.wal"), &wal);
            let reopened = open_and_migrate(path)
                .map_err(|e2| format!("compact failed ({e}) and reopening the original also failed: {e2} — the original file is intact at {path}; restart SandiBumi"))?;
            drop(db.install(&mut guard, reopened));
            return Err(format!("compact aborted (the compacted copy failed to open: {e}); the project is unchanged"));
        }
    }
    let bytes_after = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    db::boot_note(format!(
        "Compacted project: {:.0} MB → {:.0} MB; the original is kept as {} — delete it once you are happy with the result",
        bytes_before as f64 / 1_048_576.0,
        bytes_after as f64 / 1_048_576.0,
        old
    ));
    Ok(CompactReport { bytes_before, bytes_after, old_file: old })
}

/// Opens (or, for a fresh file, creates) the DuckDB at `path`, runs the launch
/// migrations, swaps it in as the live connection and records it in the recents.
/// On any error the current project stays open untouched.
pub fn switch_project(db: &DbState, path: &str) -> Result<RecentProject, String> {
    let new_conn = open_and_migrate(path)?;
    {
        let mut guard = db.0.lock().unwrap();
        // Flush the outgoing project's WAL; failure is not fatal to the switch
        // (the WAL simply replays on that file's next open).
        let _ = guard.execute_batch("CHECKPOINT;");
        // Swap site 2 of 3 (#129 M1) - Open Project and New Project both land here.
        drop(db.install(&mut guard, new_conn));
    }
    register_recent(path);
    Ok(RecentProject {
        name: project_name(path),
        path: absolute(path),
        last_opened: now_secs(),
        exists: true,
    })
}

/// True when `path` is the project that is already open (switching would be a no-op
/// and, worse, DuckDB refuses a second read-write open of the same file).
pub fn is_current(state: &ProjectState, path: &str) -> bool {
    same_path(&state.0.lock().unwrap(), path)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The whole startup-recovery design rests on this: a project that cannot be opened must
    /// come back as an `Err` VALUE, never a panic. `run()` calls this before the Tauri window
    /// exists, and with `panic = "abort"` plus `windows_subsystem = "windows"` a panic there
    /// kills the process with no window, no dialog and no console — the app simply never
    /// appears. If this ever starts panicking instead of returning, that symptom comes back.
    #[test]
    fn open_and_migrate_reports_an_unopenable_path_instead_of_panicking() {
        // A directory: it exists, so this takes the genuine open-failure branch rather than the
        // create-a-new-file branch, and no filesystem state is disturbed either way.
        let dir = std::env::temp_dir();
        let err = open_and_migrate(dir.to_str().unwrap())
            .expect_err("a directory is not a project file and must not open");
        assert!(
            err.contains("could not open"),
            "the message must name what failed so the startup dialog can show it: {err}"
        );
    }

    /// ...and the other half: the fallback needs somewhere to fall back TO, so creating a fresh
    /// project at a writable path must work. This is exactly what `run()` does with its
    /// `sandibumi-recovery-<stamp>.duckdb` when the real project will not open.
    #[test]
    fn open_and_migrate_creates_a_fresh_recovery_project() {
        let p = std::env::temp_dir()
            .join(format!("sandibumi-recovery-test-{}.duckdb", std::process::id()));
        let _ = std::fs::remove_file(&p);
        let conn = open_and_migrate(p.to_str().unwrap())
            .expect("a fresh project file at a writable path must open");
        // The schema is really there — a recovery session has to be usable, not just openable.
        conn.execute_batch("SELECT count(*) FROM wells;").expect("schema created");
        drop(conn);
        assert!(p.exists(), "the recovery project is a real file, so Save As can copy it");
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(p.with_extension("duckdb.wal"));
    }

    /// #129 stage 1, end to end: `reader_pool.rs` pins the pool, this pins that the two swap
    /// sites in THIS file actually call it. A pool that is correct and never invalidated is
    /// corruption mode M1 with extra steps.
    ///
    /// Both sites are covered because both fail the same silent way - the reader keeps serving the
    /// PREVIOUS project's wells, and they look exactly like wells.
    ///
    /// It was expected that `compact_project` would fail more loudly, on the theory that Windows
    /// will not rename a file something still holds open. Measured: it will. A mutation that
    /// bumped the generation but never released the pooled handle ran Compact Project to
    /// completion. There is no accidental filesystem guard behind the stamp, which is worth
    /// knowing rather than assuming - see the note in `reader_pool.rs`.
    ///
    /// This lives here rather than beside the pool because it needs a real file on a real
    /// filesystem and the two real swap functions, not a hand-made pair of connections.
    #[test]
    fn a_pooled_reader_follows_the_project_across_both_swap_sites() {
        let _guard = CONFIG_DIR.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("sandibumi-pool-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("SANDIBUMI_CONFIG_DIR", tmp.join("cfg").to_str().unwrap());

        let make = |path: &std::path::Path, well: &str| {
            let conn = db::init_db_resilient(path.to_str().unwrap()).unwrap();
            conn.execute_batch(&format!(
                "INSERT INTO wells (well_id, well_name, field_name, td, kb) \
                 VALUES (gen_random_uuid(), '{well}', NULL, NULL, NULL);"
            ))
            .unwrap();
            conn
        };
        let a = tmp.join("alpha.duckdb");
        let b = tmp.join("beta.duckdb");
        let state = DbState::new(make(&a, "SANDI-A"));
        drop(make(&b, "SANDI-B"));

        let pooled_well = |state: &DbState| -> String {
            state
                .1
                .read(&state.0, |reader| {
                    reader
                        .query_row("SELECT well_name FROM wells", [], |r| r.get::<_, String>(0))
                        .map_err(|e| e.to_string())
                })
                .unwrap()
        };

        // A pooled read leaves its handle behind - without that there is nothing stale to serve
        // and the rest of this test would pass for the wrong reason.
        assert_eq!(pooled_well(&state), "SANDI-A");
        assert!(state.1.holds_idle_handle(), "the read must leave a handle in the pool");

        // Swap site 2 of 3.
        switch_project(&state, b.to_str().unwrap()).expect("switch");
        assert_eq!(
            pooled_well(&state),
            "SANDI-B",
            "the pooled reader served the previous project after Open Project"
        );

        // Swap site 1 of 3. Compact Project rewrites the file and swaps the connection under it,
        // so a reader that survived would answer from a file that is no longer the project.
        assert!(state.1.holds_idle_handle(), "the read above must have left a handle to release");
        compact_project(&state, b.to_str().unwrap()).expect("compact must still succeed");
        assert_eq!(
            pooled_well(&state),
            "SANDI-B",
            "the pooled reader lost the project across Compact Project"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// `SANDIBUMI_CONFIG_DIR` is process-global, so every test that redirects it takes this
    /// first. It is what lets the trust-list contract below have a name of its own instead of
    /// being appended to a test about something else.
    pub(crate) static CONFIG_DIR: Mutex<()> = Mutex::new(());

    /// Recents round-trip, startup fallback and a live connection swap, in ONE test —
    /// they share the redirected config dir and the same `DbState`.
    #[test]
    fn recents_roundtrip_and_project_switch() {
        let _guard = CONFIG_DIR.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("sandibumi-proj-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("SANDIBUMI_CONFIG_DIR", tmp.join("cfg").to_str().unwrap());

        // Empty recents → legacy fallback.
        assert_eq!(startup_path(), LEGACY_DEFAULT);

        // Two projects; B opened last → startup picks B; re-registering A moves it up.
        let a = tmp.join("alpha.duckdb");
        let b = tmp.join("beta.duckdb");
        let conn_a = db::init_db_resilient(a.to_str().unwrap()).unwrap();
        conn_a
            .execute_batch(
                "INSERT INTO wells (well_id, well_name, field_name, td, kb) \
                 VALUES (gen_random_uuid(), 'WELL-A', NULL, NULL, NULL);",
            )
            .unwrap();
        drop(conn_a);
        register_recent(a.to_str().unwrap());
        register_recent(b.to_str().unwrap()); // not created yet — register still works
        let list = list_recents();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "beta");
        assert!(!list[0].exists, "beta.duckdb not created yet");
        assert!(list[1].exists);
        // beta doesn't exist on disk → startup falls through to alpha.
        assert_eq!(startup_path(), absolute(a.to_str().unwrap()));

        // Live swap: state starts on A (1 well), switches to fresh B (0 wells), back to A.
        let state = DbState::new(db::init_db_resilient(a.to_str().unwrap()).unwrap());
        let wells = |s: &DbState| -> i64 {
            s.0.lock()
                .unwrap()
                .query_row("SELECT count(*) FROM wells", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(wells(&state), 1);
        let info = switch_project(&state, b.to_str().unwrap()).unwrap();
        assert_eq!(info.name, "beta");
        assert_eq!(wells(&state), 0, "fresh project must be empty");
        switch_project(&state, a.to_str().unwrap()).unwrap();
        assert_eq!(wells(&state), 1, "switching back sees the original data");

        // Recents now lead with A (last switched-to), and is_current matches it.
        assert_eq!(list_recents()[0].name, "alpha");
        let proj = ProjectState(Mutex::new(absolute(a.to_str().unwrap())));
        assert!(is_current(&proj, a.to_str().unwrap()));
        assert!(!is_current(&proj, b.to_str().unwrap()));

        std::env::remove_var("SANDIBUMI_CONFIG_DIR");
        // Windows can't delete an open DuckDB; state still holds A — drop first.
        drop(state);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// A project that arrived from elsewhere is announced ONCE, and the record of that lives on
    /// this machine rather than inside the project.
    ///
    /// Both halves are load-bearing and neither implies the other. Announcing every time trains
    /// the user to click past it; announcing never is the state this replaced. And writing the
    /// acknowledgement INTO the `.duckdb` would be the easy implementation - it would also make
    /// the file carry a trace of every machine that had opened it, so a project passed between
    /// two operators would leak where it had been. That is the same rule this repository keeps
    /// about client identifiers, pointed at file metadata (SECURITY-REVIEW-2026-08-22, F1).
    #[test]
    fn a_project_from_elsewhere_is_announced_once_and_nothing_is_written_into_it() {
        let _guard = CONFIG_DIR.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("sandibumi-trust-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("SANDIBUMI_CONFIG_DIR", tmp.join("cfg").to_str().unwrap());

        let foreign = tmp.join("from-a-partner.duckdb");
        let path = foreign.to_str().unwrap();
        drop(db::init_db_resilient(path).unwrap());
        let bytes_before = std::fs::read(&foreign).unwrap();

        // Side A: unknown until acknowledged, then known - so the notice fires once, not always.
        assert!(!project_code_is_trusted(path), "a project never opened here is not yet trusted");
        trust_project_code(path);
        assert!(project_code_is_trusted(path), "acknowledging it must be remembered");
        trust_project_code(path); // idempotent - a second acknowledgement adds no second entry
        let recorded = std::fs::read_to_string(trusted_path()).unwrap();
        assert_eq!(recorded.matches("from-a-partner").count(), 1, "recorded once, not twice");

        // Side B: the project file is untouched. An implementation that stamped the acknowledgement
        // inside the .duckdb would satisfy Side A just as well, and would leak where the file had
        // been - so the bytes are compared, not just the behaviour.
        assert_eq!(
            std::fs::read(&foreign).unwrap(),
            bytes_before,
            "the acknowledgement must live on this machine, never inside the project"
        );

        std::env::remove_var("SANDIBUMI_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// "Compact Project" must (1) actually shrink a file bloated the way the field bloats
    /// it — module outputs appended then deleted on every re-run — (2) keep every LIVE row
    /// through the swap, (3) park the original beside the project rather than deleting it,
    /// and (4) leave the swapped-in connection writable at the SAME path. Loss here would
    /// be silent (a compacted project opens fine with rows missing), hence the belt of
    /// row-count assertions through the live state.
    #[test]
    fn compact_project_shrinks_in_place_and_keeps_the_original() {
        let tmp = std::env::temp_dir().join(format!("sandibumi-compact-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let p = tmp.join("field.duckdb");
        let path = p.to_str().unwrap().to_string();

        let conn = db::init_db_resilient(&path).unwrap();
        conn.execute_batch(
            "INSERT INTO wells (well_id, well_name, field_name, td, kb)
                 VALUES (gen_random_uuid(), 'W1', NULL, NULL, NULL);
             -- A re-run's lifecycle: a big output written, deleted, rewritten smaller.
             INSERT INTO computed_curves (well_id, depth, curve_name, value)
                 SELECT (SELECT well_id FROM wells), 1000.0 + i * 0.1, 'PHIE', 0.2
                 FROM range(500000) t(i);
             DELETE FROM computed_curves;
             INSERT INTO computed_curves (well_id, depth, curve_name, value)
                 SELECT (SELECT well_id FROM wells), 1000.0 + i * 0.5, 'PHIE', 0.21
                 FROM range(1000) t(i);
             CHECKPOINT;",
        )
        .unwrap();
        let state = DbState::new(conn);

        let rep = compact_project(&state, &path).expect("compact must succeed");
        assert!(
            std::path::Path::new(&rep.old_file).exists(),
            "the original must be parked beside the project, not deleted: {}",
            rep.old_file
        );
        assert!(
            rep.bytes_after < rep.bytes_before,
            "500k dead rows must not survive the rewrite: {} -> {} bytes",
            rep.bytes_before,
            rep.bytes_after
        );
        {
            let conn = state.0.lock().unwrap();
            let rows: i64 =
                conn.query_row("SELECT count(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
            assert_eq!(rows, 1000, "every live row crosses the swap");
            let wells: i64 = conn.query_row("SELECT count(*) FROM wells", [], |r| r.get(0)).unwrap();
            assert_eq!(wells, 1);
            // The swapped-in connection is the file at `path`, live and writable.
            conn.execute(
                "INSERT INTO computed_curves (well_id, depth, curve_name, value)
                     SELECT well_id, 999.0, 'TEST', 1.0 FROM wells",
                [],
            )
            .unwrap();
        }
        drop(state);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// T-SHELL-07. "Save Project As…" is a BACKUP COPY, deliberately not the IP-style
    /// switch-to-copy: the app keeps working on the original file. That distinction is invisible
    /// in the moment — both write a file and both report success — and it is only discovered a
    /// week later, when the work done after the Save As turns out to be in whichever file the
    /// app actually stayed on. Getting it backwards silently splits a study across two projects.
    ///
    /// `save_project_as` (`lib.rs:168`) is the command, and the whole of its substance is
    /// `engine_copy_to` plus the stale-destination cleanup — it never assigns `ProjectState`,
    /// which is what makes it a backup. This drives that same sequence against a real file and
    /// checks the claim from BOTH sides: the copy must hold the state at the moment it was
    /// taken, and every later edit must land in the original and nowhere else.
    #[test]
    fn save_as_writes_a_backup_copy_and_leaves_the_app_on_the_original() {
        let tmp = std::env::temp_dir().join(format!("sandibumi-saveas-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let original = tmp.join("study.duckdb").to_str().unwrap().to_string();
        let backup = tmp.join("backup-uat.duckdb").to_str().unwrap().to_string();

        let conn = db::init_db_resilient(&original).unwrap();
        conn.execute_batch(
            "INSERT INTO wells (well_id, well_name, field_name, td, kb)
                 VALUES (gen_random_uuid(), 'SANDI-01', NULL, NULL, NULL);
             -- Dead space of the kind a month of module re-runs leaves behind.
             INSERT INTO computed_curves (well_id, depth, curve_name, value)
                 SELECT (SELECT well_id FROM wells), 1000.0 + i * 0.1, 'PHIE', 0.2
                 FROM range(200000) t(i);
             DELETE FROM computed_curves;
             INSERT INTO computed_curves (well_id, depth, curve_name, value)
                 SELECT (SELECT well_id FROM wells), 1000.0 + i * 0.5, 'PHIE', 0.21
                 FROM range(1000) t(i);
             CHECKPOINT;",
        )
        .unwrap();
        let state = DbState::new(conn);

        {
            let conn = state.0.lock().unwrap();
            db::engine_copy_to(&conn, &backup).expect("save as must write the copy");
        }
        assert!(Path::new(&backup).exists(), "the copy must exist at the chosen path");

        // Step 3 of the plan: a small change made AFTER the Save As.
        {
            let conn = state.0.lock().unwrap();
            conn.execute(
                "INSERT INTO wells (well_id, well_name, field_name, td, kb)
                     VALUES (gen_random_uuid(), 'SANDI-02-ADDED-AFTER', NULL, NULL, NULL)",
                [],
            )
            .unwrap();
        }

        // The live connection is still the ORIGINAL file, and holds the change.
        {
            let conn = state.0.lock().unwrap();
            let wells: i64 = conn.query_row("SELECT count(*) FROM wells", [], |r| r.get(0)).unwrap();
            assert_eq!(wells, 2, "the app kept working on the original, so the change is here");
        }

        // Windows will not let a second connection open a file this process holds open for
        // write, so close the live one before reading either file back.
        drop(state);

        // The copy is a valid project, opens through the normal path, and holds the state as
        // it was WHEN IT WAS TAKEN — the later well is not in it.
        let copy = open_and_migrate(&backup).expect("the copy must open as a project");
        let wells: i64 = copy.query_row("SELECT count(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(wells, 1, "the copy is a snapshot: the well added afterwards must not be in it");
        let name: String =
            copy.query_row("SELECT well_name FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "SANDI-01");
        let rows: i64 =
            copy.query_row("SELECT count(*) FROM computed_curves", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 1000, "every live row crosses into the copy");
        drop(copy);

        // And the original really did keep the change — read back from disk, not from the
        // connection that made it.
        let reopened = open_and_migrate(&original).expect("the original must still be a project");
        let wells: i64 =
            reopened.query_row("SELECT count(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(wells, 2, "the post-Save-As change is durable in the ORIGINAL file");
        drop(reopened);

        // A Save As is also a compaction: the engine copy writes live rows only, so 200k dead
        // ones do not ride along into the backup.
        let before = std::fs::metadata(&original).unwrap().len();
        let after = std::fs::metadata(&backup).unwrap().len();
        assert!(after < before, "the copy carries live rows only: {before} -> {after} bytes");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
