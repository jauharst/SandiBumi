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

fn config_dir() -> PathBuf {
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
    let t = std::time::Instant::now();
    let conn = db::init_db_resilient(path).map_err(|e| format!("could not open {path}: {e}"))?;
    eprintln!("[boot] init_db_resilient: {:?}  ({path})", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_standard_curves_to_generic_store(&conn)
        .map_err(|e| format!("curve-store migration failed: {e}"))?;
    eprintln!("[boot] migrate_standard_curves_to_generic_store: {:?}", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_drop_computed_curves_pk(&conn, Some(path))
        .map_err(|e| format!("computed-curves migration failed: {e}"))?;
    eprintln!("[boot] migrate_drop_computed_curves_pk: {:?}", t.elapsed());

    let t = std::time::Instant::now();
    db::migrate_point_data_sets(&conn, Some(path))
        .map_err(|e| format!("point-data set migration failed: {e}"))?;
    eprintln!("[boot] migrate_point_data_sets: {:?}", t.elapsed());
    Ok(conn)
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
        *guard = new_conn;
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
mod tests {
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

    /// Recents round-trip, startup fallback and a live connection swap, in ONE test —
    /// SANDIBUMI_CONFIG_DIR is process-global, so splitting these into parallel tests
    /// would race on it.
    #[test]
    fn recents_roundtrip_and_project_switch() {
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
        let state = DbState(std::sync::Arc::new(Mutex::new(db::init_db_resilient(a.to_str().unwrap()).unwrap())));
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
}
