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
skips = []
frame_ord = 0

def skip(kind, name, count, rule):
    skips.append({"kind": kind, "name": str(name), "count": int(count), "rule": str(rule)})

def object_name(obj, fallback):
    try:
        value = obj.name
        return str(value) if value is not None and str(value).strip() else fallback
    except Exception:
        return fallback

try:
    batch = dlis.load(path)
except Exception as e:
    print(f"failed to open DLIS: {e}", file=sys.stderr)
    sys.exit(2)

with batch:
    for logical_ord, lf in enumerate(batch):
        for frame in lf.frames:
            run = frame_ord
            frame_ord += 1
            frame_name = object_name(frame, f"logical-file-{logical_ord}/frame-{run}")
            try:
                data = frame.curves()
            except Exception as e:
                skip("frame", frame_name, 1, f"frame.curves failed: {type(e).__name__}: {e}")
                continue
            names = list(data.dtype.names or [])
            if not names:
                skip("frame", frame_name, 1, "frame has no named channels")
                continue
            index_name = frame.index if frame.index in names else names[0]
            try:
                depth = np.asarray(data[index_name], dtype=np.float32)
            except Exception as e:
                skip("frame", frame_name, 1, f"index channel {index_name} cannot convert to float32: {type(e).__name__}: {e}")
                continue
            if depth.ndim != 1:
                skip("frame", frame_name, 1, f"index channel {index_name} is {depth.ndim}-D; a frame index must be 1-D")
                continue
            n = int(depth.shape[0])
            if n == 0:
                skip("frame", frame_name, 1, f"index channel {index_name} has zero rows")
                continue
            unit_by_name = {}
            for ch in frame.channels:
                channel_name = object_name(ch, "unnamed-channel")
                try:
                    unit_by_name[ch.name] = ch.units or ""
                except Exception as e:
                    skip("channel", channel_name, 1, f"UNITS attribute unreadable; channel retained with no unit: {type(e).__name__}: {e}")
            for name in names:
                if name == index_name:
                    continue
                try:
                    col = data[name]
                except Exception as e:
                    skip("channel", name, 1, f"channel data unreadable: {type(e).__name__}: {e}")
                    continue
                if col.ndim != 1 or col.shape[0] != n:
                    skip("channel", name, 1, f"shape {tuple(col.shape)} is not one scalar per each of {n} index rows")
                    continue
                try:
                    vals = np.asarray(col, dtype=np.float32)
                except Exception as e:
                    skip("channel", name, 1, f"values cannot convert to float32: {type(e).__name__}: {e}")
                    continue
                out_curves.append({
                    "mnemonic": str(name).upper(),
                    "unit": unit_by_name.get(name, ""),
                    "index_unit": unit_by_name.get(index_name, ""),
                    "n": n,
                    "run": run,
                })
                buffers.append(depth.tobytes())
                buffers.append(vals.tobytes())

sys.stdout.write(json.dumps({"curves": out_curves, "skips": skips}))
sys.stdout.write("\n")
sys.stdout.flush()
sys.stdout.buffer.write(b"".join(buffers))
"#;

#[derive(Debug, Deserialize)]
struct DlisCurveMeta {
    mnemonic: String,
    unit: String,
    index_unit: String,
    n: usize,
    run: i32,
}

#[derive(Debug, Deserialize)]
struct DlisHeader {
    curves: Vec<DlisCurveMeta>,
    #[serde(default)]
    skips: Vec<DlisSkip>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisSkip {
    pub kind: String,
    pub name: String,
    pub count: usize,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DlisImportResult {
    pub path: String,
    pub curves_imported: usize,
    pub rows: usize,
    /// Existing RAW curves at the same (mnemonic, run) that this import overwrote — surfaced
    /// so a re-import (or any provenance collision) is never silent.
    pub replaced: usize,
    /// Unit reconciliation and explicit-confirmation record.
    pub notes: Vec<String>,
    /// Every automatic value conversion, including the source unit and applied factor.
    pub unit_conversions: Vec<crate::curves::UnitConversion>,
    /// Every frame/channel/curve/row the reader did not carry, with count and rule.
    pub skipped: Vec<DlisSkip>,
    pub error: Option<String>,
}

fn skip_summary(skipped: &[DlisSkip]) -> String {
    skipped
        .iter()
        .map(|item| format!("{} '{}' x{}: {}", item.kind, item.name, item.count, item.rule))
        .collect::<Vec<_>>()
        .join("; ")
}

fn failed(path: &str, error: String, skipped: Vec<DlisSkip>) -> DlisImportResult {
    DlisImportResult {
        path: path.to_string(),
        curves_imported: 0,
        rows: 0,
        replaced: 0,
        notes: Vec::new(),
        unit_conversions: Vec::new(),
        skipped,
        error: Some(error),
    }
}

fn validate_header(header: &DlisHeader) -> Result<(), String> {
    if !header.curves.is_empty() {
        return Ok(());
    }
    let detail = skip_summary(&header.skips);
    Err(if detail.is_empty() {
        "DLIS import produced no scalar curves and reported no readable frame or channel".into()
    } else {
        format!("DLIS import produced no scalar curves; every candidate was skipped: {detail}")
    })
}

/// Imports every scalar channel of a DLIS file into one existing well's generic curve store.
///
/// `set_name` (import-sets, T-IMP-02/06): None/"RAW" keeps the established behavior — set
/// RAW with per-frame run numbers, same-(mnemonic, run) re-imports REPLACE and are counted
/// in `replaced`. Any other name is auto-suffixed per well (`WIRE` taken → `WIRE_1`,
/// Geolog-style), so duplicates are always KEPT and `replaced` stays 0.
pub fn import_dlis_file(
    conn: &Connection,
    well_id: &str,
    path: &str,
    set_name: Option<&str>,
    confirmed_file_unit: Option<&str>,
) -> DlisImportResult {
    let fail = |e: String| failed(path, e, Vec::new());

    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return fail(format!("unknown well '{well_id}'"));
    }
    let desired = crate::ingest::canonical_set_name(set_name);
    let target_set =
        if desired == "RAW" { desired } else { crate::ingest::resolve_set_name(conn, well_id, &desired) };

    let Some(python) = find_python() else {
        return fail("no Python with numpy found — install Python 3.10+ with numpy and dlisio, or set SANDIBUMI_PYTHON".into());
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
    if let Err(error) = validate_header(&header) {
        return failed(path, error, header.skips);
    }
    let mut skipped = header.skips.clone();
    let payload = &stdout[nl + 1..];

    let confirmed = match confirmed_file_unit {
        Some(raw) => match crate::units::DepthUnit::parse(raw) {
            Some(unit) => Some(unit),
            None => {
                return failed(
                    path,
                    format!("unrecognized confirmed file depth unit '{raw}'"),
                    skipped,
                )
            }
        },
        None => None,
    };
    let project_unit = match crate::units::project_depth_unit(conn) {
        Ok(unit) => unit,
        Err(e) => return failed(path, e.to_string(), skipped),
    };
    let (adopted_unit, index_actions, mut notes) =
        match resolve_dlis_index_actions(project_unit, &header.curves, confirmed) {
            Ok(resolved) => resolved,
            Err(e) => return failed(path, e, skipped),
        };

    // Each curve occupies 2 * n * 4 bytes (depth column then value column).
    let mut offset = 0usize;
    let mut curves_imported = 0usize;
    let mut total_rows = 0usize;
    let mut replaced = 0usize;
    let mut unit_conversions = Vec::new();
    for (meta, index_action) in header.curves.iter().zip(index_actions.iter()) {
        let bytes = meta.n * 4;
        let end = offset + 2 * bytes;
        if end > payload.len() {
            return failed(path, format!("dlis payload truncated at curve '{}'", meta.mnemonic), skipped);
        }
        let mut depth = read_f32(&payload[offset..offset + bytes]);
        let mut values = read_f32(&payload[offset + bytes..end]);
        offset = end;
        apply_index_action(&mut depth, index_action);

        // DLIS absent/sentinel values arrive as non-finite or huge magnitudes; normalize to
        // NaN (the project-wide missing convention). Producers also embed LAS-style
        // -999.25/-9999 sentinels (RP66 has no standard null) — screen them with the same set
        // the LAS paths use, and do it BEFORE unit canonicalization so a survivor can't be
        // unit-scaled into an unrecognizable value.
        let mut nulled = 0usize;
        for v in &mut values {
            if !v.is_finite() || v.abs() > 1e30 || crate::parsers::is_las_null(*v) {
                *v = f32::NAN;
                nulled += 1;
            }
        }
        if nulled > 0 {
            skipped.push(DlisSkip {
                kind: "row".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: nulled,
                rule: "non-finite, magnitude above 1e30, or recognized null sentinel; value stored as missing".into(),
            });
        }

        let fam = crate::curves::family_for(&meta.mnemonic);
        let family = fam.map(|f| f.family);
        let mut unit = if meta.unit.trim().is_empty() { None } else { Some(meta.unit.clone()) };
        if let Some(f) = fam {
            if let Some(conversion) = crate::curves::convert_to_canonical(
                &meta.mnemonic,
                f.family,
                unit.as_deref(),
                &mut values,
            ) {
                unit = Some(f.canonical_unit.to_string());
                notes.push(conversion.note());
                unit_conversions.push(conversion);
            }
        }
        // Give DLIS frames their own run numbering (frame 0 → run 0). The old frame-0 → NULL
        // mapping collided with LAS RAW curves (also run_no NULL), so a DLIS silently
        // overwrote same-mnemonic LAS curves. Using Some(run) keeps both, preserving provenance.
        let run_no = Some(meta.run);

        // Report (don't hide) any genuine overwrite — e.g. re-importing the same DLIS.
        // Only possible in set RAW: a named set was auto-suffixed to a fresh name above.
        let collides: bool = conn
            .query_row(
                "SELECT 1 FROM curve_meta WHERE well_id = ?1 AND set_name = ?4 AND mnemonic = ?2
                 AND run_no IS NOT DISTINCT FROM ?3",
                params![well_id, &meta.mnemonic, run_no, &target_set],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if collides {
            replaced += 1;
        }

        // Sanitize the frame's depth column (drop non-finite + first-occurrence-wins dedup) the
        // same way the LAS paths do, so one bad/duplicate depth sample can't abort the whole DLIS
        // file on the (curve_id, depth) PK. Values follow the kept depth indices.
        let (keep, dreport) = crate::parsers::depth_keep_indices(&depth);
        if dreport.nonfinite > 0 {
            skipped.push(DlisSkip {
                kind: "row".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: dreport.nonfinite,
                rule: "non-finite depth index".into(),
            });
        }
        if dreport.duplicate > 0 {
            skipped.push(DlisSkip {
                kind: "row".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: dreport.duplicate,
                rule: "duplicate depth index; first occurrence kept".into(),
            });
        }
        let (depth, values) = if dreport.is_clean() {
            (depth, values)
        } else {
            (
                keep.iter().map(|&i| depth[i]).collect::<Vec<f32>>(),
                keep.iter().map(|&i| values[i]).collect::<Vec<f32>>(),
            )
        };
        if depth.is_empty() {
            skipped.push(DlisSkip {
                kind: "curve".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: 1,
                rule: "no rows survived depth-index validation".into(),
            });
            continue;
        }

        let res: db::DbResult<()> = (|| {
            let curve_id = db::upsert_curve_meta(conn, well_id, &target_set, &meta.mnemonic, unit.as_deref(), family, Some("DLIS import"), run_no)?;
            db::insert_curve_samples(conn, &curve_id, &depth, &values)?;
            Ok(())
        })();
        match res {
            Ok(()) => {
                curves_imported += 1;
                total_rows += depth.len();
            }
            Err(e) => {
                return failed(path, format!("storing curve '{}': {e}", meta.mnemonic), skipped)
            }
        }
    }

    if curves_imported == 0 {
        return failed(
            path,
            format!("DLIS import produced no curves after validation: {}", skip_summary(&skipped)),
            skipped,
        );
    }

    if project_unit.is_none() {
        if let Some(unit) = adopted_unit {
            if let Err(e) = crate::units::set_project_depth_unit(conn, unit) {
                notes.push(format!("could not record the adopted project depth unit: {e}"));
            }
        }
    }

    DlisImportResult {
        path: path.to_string(),
        curves_imported,
        rows: total_rows,
        replaced,
        notes,
        unit_conversions,
        skipped,
        error: None,
    }
}

fn resolve_dlis_index_actions(
    project_unit: Option<crate::units::DepthUnit>,
    curves: &[DlisCurveMeta],
    confirmed_file_unit: Option<crate::units::DepthUnit>,
) -> Result<
    (
        Option<crate::units::DepthUnit>,
        Vec<crate::units::IndexUnitAction>,
        Vec<String>,
    ),
    String,
> {
    let mut target = project_unit;
    let mut actions = Vec::with_capacity(curves.len());
    let mut notes = Vec::new();
    for meta in curves {
        let declared_file_unit = crate::units::DepthUnit::parse(&meta.index_unit);
        let file_unit = declared_file_unit.or(confirmed_file_unit);
        let action = crate::units::resolve_index_unit(target, file_unit)
            .map_err(|e| format!("DLIS frame {} index: {e}", meta.run))?;
        if declared_file_unit.is_none() {
            if let Some(unit) = confirmed_file_unit {
                let note = format!("DLIS frame {} file depth unit explicitly confirmed as {}", meta.run, unit.code());
                if !notes.contains(&note) {
                    notes.push(note);
                }
            }
        }
        if let Some(note) = action.note() {
            let note = format!("DLIS frame {}: {note}", meta.run);
            if !notes.contains(&note) {
                notes.push(note);
            }
        }
        if target.is_none() {
            target = file_unit;
        }
        actions.push(action);
    }
    Ok((target, actions, notes))
}

fn apply_index_action(depth: &mut [f32], action: &crate::units::IndexUnitAction) {
    if let crate::units::IndexUnitAction::Convert { from, to } = *action {
        crate::units::convert_depths(depth, from, to);
    }
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
    use std::io::Write; // for Vec::write_all in the byte-roundtrip tests
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
        let path = std::env::var("SANDIBUMI_TEST_DLIS").unwrap_or_default();
        if path.is_empty() {
            eprintln!("set SANDIBUMI_TEST_DLIS to a .dlis file to run this");
            return;
        }
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "DLIS-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        let res = import_dlis_file(&conn, &ids, &path, None, None);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert!(res.curves_imported > 0, "expected at least one curve");
        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        assert!(!catalog.is_empty());
        println!("imported {} curves, {} rows", res.curves_imported, res.rows);
    }

    /// SB-DIO-016 / SB-DIO-T25..T26. Index-unit spellings and the 0.3048
    /// international-foot factor are cited in `docs/PRD_v2/21_data-io.md` §5.1.
    #[test]
    fn the_dlis_index_unit_is_read_reconciled_and_an_undeclared_one_is_refused() {
        assert!(DLIS_RUNNER.contains("\"index_unit\": unit_by_name.get(index_name"));
        let meta = |unit: &str| DlisCurveMeta {
            mnemonic: "GR".into(),
            unit: "GAPI".into(),
            index_unit: unit.into(),
            n: 2,
            run: 0,
        };

        let (_, actions, _) = resolve_dlis_index_actions(
            Some(crate::units::DepthUnit::Metres),
            &[meta("FT")],
            None,
        )
        .unwrap();
        let mut depth = vec![1000.0_f32, 1001.0];
        apply_index_action(&mut depth, &actions[0]);
        assert!((depth[0] - 304.8).abs() < 1e-3, "a feet index converts by the cited factor");

        let refused = resolve_dlis_index_actions(
            Some(crate::units::DepthUnit::Metres),
            &[meta("")],
            None,
        )
        .unwrap_err();
        assert!(refused.contains("file index") && refused.contains("project setting is not a file declaration"));

        let (_, confirmed_actions, notes) = resolve_dlis_index_actions(
            Some(crate::units::DepthUnit::Metres),
            &[meta("")],
            Some(crate::units::DepthUnit::Feet),
        )
        .unwrap();
        let mut confirmed_depth = vec![1000.0_f32];
        apply_index_action(&mut confirmed_depth, &confirmed_actions[0]);
        assert!((confirmed_depth[0] - 304.8).abs() < 1e-3);
        assert!(notes.iter().any(|n| n.contains("explicitly confirmed as FT")));
    }

    /// SB-DIO-054 / SB-DIO-T77..T79. Skip-reporting and the located LAS-row failure are
    /// specified in `docs/PRD_v2/21_data-io.md` §§4.10 and 6.10.
    #[test]
    fn every_skipped_frame_channel_curve_and_row_is_counted_named_and_all_skipped_is_an_error() {
        for marker in [
            "skip(\"frame\", frame_name, 1",
            "skip(\"channel\", name, 1",
            "\"skips\": skips",
        ] {
            assert!(DLIS_RUNNER.contains(marker), "runner lost skip record: {marker}");
        }
        let good = DlisCurveMeta {
            mnemonic: "GR".into(),
            unit: "GAPI".into(),
            index_unit: "M".into(),
            n: 2,
            run: 1,
        };
        let one_bad_one_good = DlisHeader {
            curves: vec![good],
            skips: vec![DlisSkip {
                kind: "frame".into(),
                name: "FRAME-BAD".into(),
                count: 1,
                rule: "frame.curves failed: RuntimeError: encrypted".into(),
            }],
        };
        assert!(validate_header(&one_bad_one_good).is_ok(), "one good frame must survive a bad one");
        let all_bad = DlisHeader { curves: vec![], skips: one_bad_one_good.skips.clone() };
        let error = validate_header(&all_bad).unwrap_err();
        assert!(error.contains("FRAME-BAD") && error.contains("x1") && error.contains("encrypted"));

        let las = std::env::temp_dir().join(format!(
            "sandibumi-dio-054-short-row-{}.las",
            std::process::id()
        ));
        std::fs::write(
            &las,
            "~Version\nVERS. 2.0\nWRAP. NO\n~Well\nWELL. SHORT\n~Curve\nDEPT.M\nGR.GAPI\nRHOB.G/C3\n~ASCII\n1000 50 2.4\n1000.5 55\n1001 60\n1001.5 65\n",
        )
        .unwrap();
        for error in [
            crate::parsers::parse_las_2(&las).unwrap_err(),
            crate::parsers::parse_las_2_all(&las).unwrap_err(),
        ] {
            let message = error.to_string();
            assert!(message.contains("line 12"), "the first short row must be located: {message}");
            assert!(message.contains("2 value(s)") && message.contains("3 columns"));
        }

        // The opposite side: an explicit wrapped file is allowed to split one logical row over
        // physical lines, so locating unwrapped truncation must not disable LAS WRAP support.
        std::fs::write(
            &las,
            "~Version\nVERS. 2.0\nWRAP. YES\n~Well\nWELL. WRAPPED\n~Curve\nDEPT.M\nGR.GAPI\nRHOB.G/C3\n~ASCII\n1000 50\n2.4 1000.5\n55 2.5\n",
        )
        .unwrap();
        assert_eq!(crate::parsers::parse_las_2(&las).unwrap().depth.len(), 2);
        assert_eq!(crate::parsers::parse_las_2_all(&las).unwrap().depth.len(), 2);
        let _ = std::fs::remove_file(&las);
    }
}
