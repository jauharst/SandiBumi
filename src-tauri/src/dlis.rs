//! Phase 6: DLIS import via `dlisio` through the Python subprocess — the same mechanism as
//! the equation engine (`python_engine.rs`), never PyO3. A helper Python script reads every
//! scalar channel from every frame of a DLIS file and streams a JSON header line followed by
//! raw little-endian f32 depth/value columns; Rust writes each channel into the generic
//! curve store (`curve_meta`/`curve_samples`) as set RAW, tagging family and canonicalizing
//! units via `crate::curves`.
//!
//! DLIS attaches curves to an EXISTING well (like core/deviation import), rather than
//! creating one — the user selects the target well, then imports. A missing `dlisio` (or
//! Python) fails the import with a clear message and never affects anything else.

use crate::db;
use crate::python_engine::{find_python, hide_console};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

/// Streams every 1-D scalar channel of every frame. Multi-dimensional channels (image/array
/// logs) are skipped for now — they belong in `array_logs`, a later phase. The index channel
/// of each frame becomes that frame's depth; each other channel is emitted as its own curve
/// paired with that depth, so frames with different sampling stay independent.
const DLIS_RUNNER: &str = r#"
import sys, json
import numpy as np
try:
    from dlisio import dlis
except Exception as e:
    print(f"dlisio not available: {e}", file=sys.stderr)
    sys.exit(3)

path = sys.argv[1]
out_curves = []
buffers = []
frame_ord = 0
try:
    batch = dlis.load(path)
except Exception as e:
    print(f"failed to open DLIS: {e}", file=sys.stderr)
    sys.exit(2)

with batch:
    for lf in batch:
        for frame in lf.frames:
            try:
                data = frame.curves()
            except Exception:
                continue
            names = list(data.dtype.names or [])
            if not names:
                continue
            index_name = frame.index if frame.index in names else names[0]
            try:
                depth = np.asarray(data[index_name], dtype=np.float32)
            except Exception:
                continue
            if depth.ndim != 1:
                continue
            n = int(depth.shape[0])
            if n == 0:
                continue
            unit_by_name = {}
            for ch in frame.channels:
                try:
                    unit_by_name[ch.name] = ch.units or ""
                except Exception:
                    pass
            for name in names:
                if name == index_name:
                    continue
                col = data[name]
                if col.ndim != 1 or col.shape[0] != n:
                    continue  # skip array/multidim channels for now
                vals = np.asarray(col, dtype=np.float32)
                out_curves.append({
                    "mnemonic": str(name).upper(),
                    "unit": unit_by_name.get(name, ""),
                    "n": n,
                    "run": frame_ord,
                })
                buffers.append(depth.tobytes())
                buffers.append(vals.tobytes())
            frame_ord += 1

sys.stdout.write(json.dumps({"curves": out_curves}))
sys.stdout.write("\n")
sys.stdout.flush()
sys.stdout.buffer.write(b"".join(buffers))
"#;

#[derive(Debug, Deserialize)]
struct DlisCurveMeta {
    mnemonic: String,
    unit: String,
    n: usize,
    run: i32,
}

#[derive(Debug, Deserialize)]
struct DlisHeader {
    curves: Vec<DlisCurveMeta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DlisImportResult {
    pub path: String,
    pub curves_imported: usize,
    pub rows: usize,
    /// Existing RAW curves at the same (mnemonic, run) that this import overwrote — surfaced
    /// so a re-import (or any provenance collision) is never silent.
    pub replaced: usize,
    pub error: Option<String>,
}

/// Imports every scalar channel of a DLIS file into one existing well's generic curve store.
pub fn import_dlis_file(conn: &Connection, well_id: &str, path: &str) -> DlisImportResult {
    let fail = |e: String| DlisImportResult { path: path.to_string(), curves_imported: 0, rows: 0, replaced: 0, error: Some(e) };

    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return fail(format!("unknown well '{well_id}'"));
    }

    let Some(python) = find_python() else {
        return fail("no Python with numpy found — install Python 3.10+ with numpy and dlisio, or set ARSHILLA_PYTHON".into());
    };

    let mut cmd = Command::new(&python);
    cmd.args(["-c", DLIS_RUNNER, path]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => return fail(format!("failed to start python: {e}")),
    };
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("dlis import failed");
        return fail(last.trim().to_string());
    }

    // stdout = one JSON header line, then raw f32 payload (depth+values per curve).
    let stdout = output.stdout;
    let nl = match stdout.iter().position(|&b| b == b'\n') {
        Some(i) => i,
        None => return fail("malformed dlis runner output (no header line)".into()),
    };
    let header: DlisHeader = match serde_json::from_slice(&stdout[..nl]) {
        Ok(h) => h,
        Err(e) => return fail(format!("bad dlis header: {e}")),
    };
    let payload = &stdout[nl + 1..];

    // Each curve occupies 2 * n * 4 bytes (depth column then value column).
    let mut offset = 0usize;
    let mut curves_imported = 0usize;
    let mut total_rows = 0usize;
    let mut replaced = 0usize;
    for meta in &header.curves {
        let bytes = meta.n * 4;
        let end = offset + 2 * bytes;
        if end > payload.len() {
            return fail(format!("dlis payload truncated at curve '{}'", meta.mnemonic));
        }
        let depth = read_f32(&payload[offset..offset + bytes]);
        let mut values = read_f32(&payload[offset + bytes..end]);
        offset = end;

        // DLIS absent/sentinel values arrive as non-finite or huge magnitudes; normalize to
        // NaN (the project-wide missing convention). Producers also embed LAS-style
        // -999.25/-9999 sentinels (RP66 has no standard null) — screen them with the same set
        // the LAS paths use, and do it BEFORE unit canonicalization so a survivor can't be
        // unit-scaled into an unrecognizable value.
        for v in &mut values {
            if !v.is_finite() || v.abs() > 1e30 || crate::parsers::is_las_null(*v) {
                *v = f32::NAN;
            }
        }

        let fam = crate::curves::family_for(&meta.mnemonic);
        let family = fam.map(|f| f.family);
        let mut unit = if meta.unit.trim().is_empty() { None } else { Some(meta.unit.clone()) };
        if let Some(f) = fam {
            if crate::curves::convert_to_canonical(f.family, unit.as_deref(), &mut values) {
                unit = Some(f.canonical_unit.to_string());
            }
        }
        // Give DLIS frames their own run numbering (frame 0 → run 0). The old frame-0 → NULL
        // mapping collided with LAS RAW curves (also run_no NULL), so a DLIS silently
        // overwrote same-mnemonic LAS curves. Using Some(run) keeps both, preserving provenance.
        let run_no = Some(meta.run);

        // Report (don't hide) any genuine overwrite — e.g. re-importing the same DLIS.
        let collides: bool = conn
            .query_row(
                "SELECT 1 FROM curve_meta WHERE well_id = ?1 AND set_name = 'RAW' AND mnemonic = ?2
                 AND run_no IS NOT DISTINCT FROM ?3",
                params![well_id, &meta.mnemonic, run_no],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if collides {
            replaced += 1;
        }

        let res: db::DbResult<()> = (|| {
            let curve_id = db::upsert_curve_meta(conn, well_id, "RAW", &meta.mnemonic, unit.as_deref(), family, Some("DLIS import"), run_no)?;
            db::insert_curve_samples(conn, &curve_id, &depth, &values)?;
            Ok(())
        })();
        match res {
            Ok(()) => {
                curves_imported += 1;
                total_rows += meta.n;
            }
            Err(e) => return fail(format!("storing curve '{}': {e}", meta.mnemonic)),
        }
    }

    DlisImportResult { path: path.to_string(), curves_imported, rows: total_rows, replaced, error: None }
}

fn read_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn read_f32_roundtrip() {
        let vals = [1.0f32, 2.5, -3.25, f32::NAN];
        let mut bytes = Vec::new();
        for v in &vals {
            bytes.write_all(&v.to_le_bytes()).unwrap();
        }
        let back = read_f32(&bytes);
        assert_eq!(back[0], 1.0);
        assert_eq!(back[2], -3.25);
        assert!(back[3].is_nan());
    }

    /// Full DLIS import path, gated on a real DLIS file being present (ignored by default —
    /// the CI/synthetic path is covered by `read_f32_roundtrip` + the dlisio runner is
    /// exercised manually). Run with `--ignored` once a sample .dlis is dropped at the path.
    #[test]
    #[ignore]
    fn import_real_dlis() {
        let path = std::env::var("ARSHILLA_TEST_DLIS").unwrap_or_default();
        if path.is_empty() {
            eprintln!("set ARSHILLA_TEST_DLIS to a .dlis file to run this");
            return;
        }
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "DLIS-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        let res = import_dlis_file(&conn, &ids, &path);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(res.curves_imported > 0, "expected at least one curve");
        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        assert!(!catalog.is_empty());
        println!("imported {} curves, {} rows", res.curves_imported, res.rows);
    }
}
