//! The module-reference freshness gate.
//!
//! `docs/generated/module_manifests.json` is the machine-readable dump of
//! `modules::list_modules()` — the same manifests the application's own panes are generated
//! from. `tools/gen-module-reference.mjs` renders the user-facing reference pages under
//! `docs/guide/reference/` from that dump, so the reference can never say something the
//! running application does not: the manifests are the single source of truth, and this test
//! is what keeps the committed dump current.
//!
//! Regenerate with:
//!
//! ```text
//! SANDIBUMI_WRITE_MANIFEST_DUMP=1 cargo test --lib the_committed_manifest_dump
//! node tools/gen-module-reference.mjs
//! ```

#![cfg(test)]

use crate::modules;
use std::path::PathBuf;

fn dump_path() -> PathBuf {
    // CARGO_MANIFEST_DIR is src-tauri/, the dump lives beside the other generated docs.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/generated/module_manifests.json")
}

/// The comparison ignores line endings for the same reason `tools/generated-artifact.mjs`
/// does: git's `core.autocrlf` (the Windows default) checks the committed file out with CRLF
/// while the generator writes LF, and a clean checkout must not read as stale.
fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}

#[test]
fn the_committed_manifest_dump_is_what_list_modules_answers() {
    let specs = modules::list_modules();
    assert!(
        specs.len() > 10,
        "list_modules() answered {} specs — the dump would be an empty reference",
        specs.len()
    );
    let fresh = serde_json::to_string_pretty(&specs).expect("manifests serialize") + "\n";

    let path = dump_path();
    if std::env::var("SANDIBUMI_WRITE_MANIFEST_DUMP").is_ok() {
        std::fs::write(&path, &fresh).expect("write module_manifests.json");
        return;
    }

    let committed = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "docs/generated/module_manifests.json is unreadable ({e}). Regenerate it with \
             SANDIBUMI_WRITE_MANIFEST_DUMP=1 cargo test --lib the_committed_manifest_dump, \
             then node tools/gen-module-reference.mjs"
        )
    });
    assert_eq!(
        normalized(&committed),
        normalized(&fresh),
        "docs/generated/module_manifests.json is stale against list_modules(). Regenerate it \
         with SANDIBUMI_WRITE_MANIFEST_DUMP=1 cargo test --lib the_committed_manifest_dump, \
         then node tools/gen-module-reference.mjs to re-render the reference pages"
    );
}
