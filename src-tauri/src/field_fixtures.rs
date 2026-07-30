//! Locating the real field deliveries that the `#[ignore]`d integration tests run against.
//!
//! Those tests exist because synthetic data cannot reproduce what a real delivery does to a
//! parser — the malformed headers, the placeholder columns, the stray encoding bytes. They are
//! genuinely valuable and they stay.
//!
//! What changed (provenance sweep, 2026-07-31): the deliveries used to be named by **absolute
//! path**, twenty of them, each spelling out the operator, the contract, the project number and
//! the well identifier of work done under a confidentiality agreement. That put a client's
//! delivery manifest into a repository intended for licensing.
//!
//! So the tests no longer name a file. They ask this module for a directory and take whatever is
//! in it. Set `SANDIBUMI_FIELD_FIXTURES` to a folder holding:
//!
//! ```text
//!   <root>/las/     any number of .las files   — import, chain and pipeline tests
//!   <root>/core/    any number of .csv files   — core-table probe tests
//! ```
//!
//! Unset, or missing, and every test that needs one skips with a printed reason rather than
//! failing: a fresh clone has no field data and must still go green.
//!
//! This is also better as a test. A test that names one specific delivered file proves the
//! importer works on *that* file; a test that reads whatever the folder holds proves it works on
//! a delivery — which is the claim actually being made.

use std::path::{Path, PathBuf};

/// The environment variable naming the field-fixture root.
pub const FIELD_FIXTURE_ENV: &str = "SANDIBUMI_FIELD_FIXTURES";

/// The configured fixture root, if it is set and exists.
pub fn root() -> Option<PathBuf> {
    let raw = std::env::var(FIELD_FIXTURE_ENV).ok()?;
    let p = PathBuf::from(raw.trim());
    if p.is_dir() {
        Some(p)
    } else {
        None
    }
}

/// Files with `ext` under `<root>/<sub>`, sorted by name so a run is reproducible.
/// Sorting matters: several of these tests assert on "the first well", and directory
/// order is not stable across machines.
fn files_in(sub: &str, ext: &str) -> Vec<PathBuf> {
    let Some(dir) = root().map(|r| r.join(sub)) else {
        return Vec::new();
    };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case(ext))
        })
        .collect();
    out.sort();
    out
}

/// Up to `max` real LAS deliveries. Empty when no fixture root is configured.
pub fn las_files(max: usize) -> Vec<String> {
    files_in("las", "las")
        .into_iter()
        .take(max)
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

/// The first real core table, if one is configured.
pub fn core_table() -> Option<PathBuf> {
    files_in("core", "csv").into_iter().next()
}

/// Prints why a test is skipping and returns `true` when it should. Keeps the reason
/// visible under `--nocapture`, so a silently-passing test never masks a missing fixture.
pub fn skip(what: &str, found: usize, need: usize) -> bool {
    if found >= need {
        return false;
    }
    match root() {
        None => eprintln!(
            "SKIP {what}: set {FIELD_FIXTURE_ENV} to a folder with las/ and core/ subfolders \
             of real deliveries to run this"
        ),
        Some(r) => eprintln!(
            "SKIP {what}: {} holds {found} usable file(s), need {need}",
            r.display()
        ),
    }
    true
}

/// A temp database path for an integration test, cleared if a previous run left one.
pub fn temp_db(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("sandibumi_{name}.duckdb"));
    let _ = std::fs::remove_file(&p);
    p
}

/// True when `p` exists; prints the same skip reason otherwise.
pub fn have(p: &Path, what: &str) -> bool {
    if p.exists() {
        return true;
    }
    eprintln!("SKIP {what}: {} is not on this machine", p.display());
    false
}
