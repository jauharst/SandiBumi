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
use crate::installation;
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
    print(f"dlis-dependency-missing: {e}", file=sys.stderr)
    sys.exit(3)

path = sys.argv[1]
out_curves = []
buffers = []
skips = []
logical_files = []
frame_ord = 0
channels_declared = 0

def skip(kind, name, count, rule, omitted=True):
    skips.append({"kind": kind, "name": str(name), "count": int(count), "rule": str(rule), "omitted": bool(omitted)})

def object_name(obj, fallback):
    try:
        value = obj.name
        return str(value) if value is not None and str(value).strip() else fallback
    except Exception:
        return fallback

def attr_text(obj, attr):
    try:
        value = getattr(obj, attr)
        return str(value).strip() if value is not None else ""
    except Exception:
        return ""

try:
    batch = dlis.load(path)
except Exception as e:
    print(f"failed to open DLIS: {e}", file=sys.stderr)
    sys.exit(2)

with batch:
    for logical_ord, lf in enumerate(batch):
        try:
            origins = list(lf.origins)
        except Exception as e:
            origins = []
            skip("logical_file", f"logical-file-{logical_ord}", 1, f"ORIGIN directory unreadable: {type(e).__name__}: {e}", omitted=False)
        defining_origin = origins[0] if origins else None
        well_name = attr_text(defining_origin, "well_name") if defining_origin is not None else ""
        well_id = attr_text(defining_origin, "well_id") if defining_origin is not None else ""
        logical_files.append({
            "logical_file": logical_ord,
            "source_well": well_name or well_id,
        })
        for frame in lf.frames:
            run = frame_ord
            frame_ord += 1
            frame_name = object_name(frame, f"logical-file-{logical_ord}/frame-{run}")
            try:
                frame_channels = list(frame.channels)
            except Exception as e:
                frame_channels = []
                skip("frame", frame_name, 1, f"channel directory unreadable; frame data will still be attempted: {type(e).__name__}: {e}", omitted=False)
            frame_channel_names = [object_name(ch, "unnamed-channel") for ch in frame_channels]
            try:
                index_hint = str(frame.index)
            except Exception:
                index_hint = ""
            try:
                data = frame.curves()
            except Exception as e:
                payload_names = [name for name in frame_channel_names if name != index_hint]
                channels_declared += len(payload_names)
                if payload_names:
                    for name in payload_names:
                        skip("channel", name, 1, f"frame {frame_name} unreadable: {type(e).__name__}: {e}")
                else:
                    skip("frame", frame_name, 1, f"frame.curves failed: {type(e).__name__}: {e}")
                continue
            names = list(data.dtype.names or [])
            if not names:
                skip("frame", frame_name, 1, "frame has no named channels")
                continue
            index_name = frame.index if frame.index in names else names[0]
            declared_payload_names = [name for name in frame_channel_names if name != str(index_name)]
            if not declared_payload_names:
                declared_payload_names = [str(name) for name in names if name != index_name]
            channels_declared += len(declared_payload_names)
            for declared in declared_payload_names:
                if declared not in names:
                    skip("channel", declared, 1, f"channel declared in {frame_name} but absent from frame.curves output")
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
            for ch in frame_channels:
                channel_name = object_name(ch, "unnamed-channel")
                try:
                    unit_by_name[ch.name] = ch.units or ""
                except Exception as e:
                    skip("channel", channel_name, 1, f"UNITS attribute unreadable; channel retained with no unit: {type(e).__name__}: {e}", omitted=False)
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
                    "logical_file": logical_ord,
                })
                buffers.append(depth.tobytes())
                buffers.append(vals.tobytes())

sys.stdout.write(json.dumps({"curves": out_curves, "skips": skips, "channels_declared": channels_declared, "logical_files": logical_files}))
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
    #[serde(default)]
    logical_file: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct DlisLogicalFile {
    logical_file: usize,
    source_well: String,
}

#[derive(Debug, Deserialize)]
struct DlisHeader {
    curves: Vec<DlisCurveMeta>,
    #[serde(default)]
    skips: Vec<DlisSkip>,
    #[serde(default)]
    channels_declared: usize,
    #[serde(default)]
    logical_files: Vec<DlisLogicalFile>,
}

struct PreparedDlisCurve {
    well_id: String,
    set_name: String,
    mnemonic: String,
    unit: Option<String>,
    family: Option<&'static str>,
    run_no: Option<i32>,
    depth: Vec<f32>,
    values: Vec<f32>,
}

fn write_prepared_dlis(
    conn: &Connection,
    mappings: &[DlisWellMapping],
    curves: &[PreparedDlisCurve],
    stored_depth_unit: Option<crate::units::DepthUnit>,
) -> db::DbResult<()> {
    for mapping in mappings.iter().filter(|mapping| mapping.will_create) {
        let id = uuid::Uuid::parse_str(
            mapping
                .target_well_id
                .as_deref()
                .ok_or_else(|| db::DbError::LengthMismatch("a committed mapping has no target well id".into()))?,
        )
        .map_err(|error| db::DbError::LengthMismatch(error.to_string()))?;
        db::insert_well(conn, id, &mapping.target_well_name, None, None, None)?;
        if let Some(unit) = stored_depth_unit {
            conn.execute(
                "UPDATE wells SET depth_unit = ?2 WHERE well_id = ?1",
                params![id.to_string(), unit.code()],
            )?;
        }
    }
    for curve in curves {
        let curve_id = db::upsert_curve_meta(
            conn,
            &curve.well_id,
            &curve.set_name,
            &curve.mnemonic,
            curve.unit.as_deref(),
            curve.family,
            Some("DLIS import"),
            curve.run_no,
        )?;
        db::insert_curve_samples(conn, &curve_id, &curve.depth, &curve.values)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisSkip {
    pub kind: String,
    pub name: String,
    pub count: usize,
    pub rule: String,
    /// True when the named frame/channel/curve was not loaded. False for a row-level screen or an
    /// attribute warning on a channel that was retained.
    #[serde(default)]
    pub omitted: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlisImportStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisWellMapping {
    pub source_well: String,
    pub logical_files: Vec<usize>,
    pub target_well_name: String,
    pub target_well_id: Option<String>,
    pub will_create: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlisOutsideIntervalDecision {
    AcceptOutsideDeclaredInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DlisIntervalConflict {
    /// `well` or `set`; stated so an existing well interval and a narrower delivery interval are
    /// not collapsed into one ambiguous warning.
    pub scope: String,
    pub name: String,
    pub declared_top: f32,
    pub declared_base: f32,
    pub incoming_top: f32,
    pub incoming_base: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlisDuplicateAction {
    KeepSeparate,
    SkipIncoming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisDuplicateDecision {
    pub mnemonic: String,
    pub run: i32,
    pub action: DlisDuplicateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisDuplicateConflict {
    pub mnemonic: String,
    pub run: i32,
    /// Existing identities are stated as `SET/run N` (or `SET/run none`); no existing curve is
    /// silently elected as the one an incoming channel would modify.
    pub existing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlisDuplicateDecisionRecord {
    pub mnemonic: String,
    pub run: i32,
    pub action: DlisDuplicateAction,
    pub existing: Vec<String>,
    pub target_set: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DlisImportResult {
    pub path: String,
    pub status: DlisImportStatus,
    pub channels_declared: usize,
    pub curves_imported: usize,
    pub rows: usize,
    /// Legacy result field, now always zero: duplicate mnemonics require keep-separate or skip,
    /// and no DLIS path merges/replaces an existing curve by default.
    pub replaced: usize,
    /// Unit reconciliation and explicit-confirmation record.
    pub notes: Vec<String>,
    /// Every automatic value conversion, including the source unit and applied factor.
    pub unit_conversions: Vec<crate::curves::UnitConversion>,
    /// Declared units that were preserved because no reviewed conversion applied.
    pub unconverted_units: Vec<crate::curves::UnconvertedUnit>,
    /// Per-file answers to genuinely ambiguous unit symbols.
    pub unit_designations: Vec<crate::curves::UnitDesignation>,
    /// Every frame/channel/curve/row the reader did not carry, with count and rule.
    pub skipped: Vec<DlisSkip>,
    /// Exact channel mnemonics for which the user disabled the LAS-derived sentinel fallback.
    pub sentinel_exceptions: Vec<String>,
    /// Source-well grouping and its proposed/committed project target. For a multi-well
    /// container this is populated before any write and must be echoed back to confirm it.
    pub well_mappings: Vec<DlisWellMapping>,
    pub mapping_confirmation_required: bool,
    /// Incoming extents outside an existing well/set extent. Empty means there was no conflict;
    /// populated beside an error means the required decision was absent.
    pub interval_conflicts: Vec<DlisIntervalConflict>,
    /// Duplicate mnemonic questions found before commit and the explicit per-curve answers used.
    pub duplicate_conflicts: Vec<DlisDuplicateConflict>,
    pub duplicate_decisions: Vec<DlisDuplicateDecisionRecord>,
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
        status: DlisImportStatus::Failed,
        channels_declared: 0,
        curves_imported: 0,
        rows: 0,
        replaced: 0,
        notes: Vec::new(),
        unit_conversions: Vec::new(),
        unconverted_units: Vec::new(),
        unit_designations: Vec::new(),
        skipped,
        sentinel_exceptions: Vec::new(),
        well_mappings: Vec::new(),
        mapping_confirmation_required: false,
        interval_conflicts: Vec::new(),
        duplicate_conflicts: Vec::new(),
        duplicate_decisions: Vec::new(),
        error: Some(error),
    }
}

fn failed_interval(
    path: &str,
    conflicts: Vec<DlisIntervalConflict>,
    skipped: Vec<DlisSkip>,
) -> DlisImportResult {
    let detail = conflicts
        .iter()
        .map(|conflict| {
            format!(
                "{} '{}' is {:.4}-{:.4}, incoming DLIS is {:.4}-{:.4}",
                conflict.scope,
                conflict.name,
                conflict.declared_top,
                conflict.declared_base,
                conflict.incoming_top,
                conflict.incoming_base
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let mut result = failed(
        path,
        format!(
            "incoming DLIS falls outside an existing declared interval ({detail}); explicit confirmation is required before commit"
        ),
        skipped,
    );
    result.interval_conflicts = conflicts;
    result
}

fn failed_mapping(path: &str, mapping: Vec<DlisWellMapping>, skipped: Vec<DlisSkip>) -> DlisImportResult {
    let detail = mapping
        .iter()
        .map(|item| {
            format!(
                "{} (logical files {:?}) -> new project well {}",
                item.source_well, item.logical_files, item.target_well_name
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let mut result = failed(
        path,
        format!(
            "multi-well DLIS requires confirmation of the container-to-well mapping before commit: {detail}"
        ),
        skipped,
    );
    result.well_mappings = mapping;
    result.mapping_confirmation_required = true;
    result
}

fn failed_duplicates(
    path: &str,
    conflicts: Vec<DlisDuplicateConflict>,
    skipped: Vec<DlisSkip>,
) -> DlisImportResult {
    let detail = conflicts
        .iter()
        .map(|conflict| {
            format!(
                "{} frame {} already exists as {}",
                conflict.mnemonic,
                conflict.run,
                conflict.existing.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let mut result = failed(
        path,
        format!(
            "incoming DLIS repeats mnemonic(s) already held by this well ({detail}); a per-curve keep-separate or skip decision is required before commit"
        ),
        skipped,
    );
    result.duplicate_conflicts = conflicts;
    result
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

fn import_status(
    header: &DlisHeader,
    curves_imported: usize,
    skipped: &[DlisSkip],
) -> DlisImportStatus {
    if curves_imported == 0 {
        DlisImportStatus::Failed
    } else if curves_imported < header.channels_declared
        || skipped.iter().any(|item| item.omitted)
    {
        DlisImportStatus::Partial
    } else {
        DlisImportStatus::Complete
    }
}

fn multi_well_plan(header: &DlisHeader) -> Result<Vec<DlisWellMapping>, String> {
    if header.logical_files.len() <= 1 {
        return Ok(Vec::new());
    }
    let unnamed: Vec<usize> = header
        .logical_files
        .iter()
        .filter(|logical| logical.source_well.trim().is_empty())
        .map(|logical| logical.logical_file)
        .collect();
    if !unnamed.is_empty() {
        return Err(format!(
            "multi-logical-file DLIS has no source WELL-NAME or WELL-ID for logical file(s) {unnamed:?}; the wells cannot be separated without source identity"
        ));
    }

    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for logical in &header.logical_files {
        if let Some((_, files)) = groups
            .iter_mut()
            .find(|(name, _)| name.eq_ignore_ascii_case(logical.source_well.trim()))
        {
            files.push(logical.logical_file);
        } else {
            groups.push((
                logical.source_well.trim().to_string(),
                vec![logical.logical_file],
            ));
        }
    }
    if groups.len() <= 1 {
        return Ok(Vec::new());
    }
    Ok(groups
        .into_iter()
        .map(|(source_well, logical_files)| DlisWellMapping {
            target_well_name: source_well.clone(),
            source_well,
            logical_files,
            target_well_id: None,
            will_create: true,
        })
        .collect())
}

fn mapping_confirmation_matches(
    proposed: &[DlisWellMapping],
    confirmed: &[DlisWellMapping],
) -> bool {
    proposed.len() == confirmed.len()
        && proposed.iter().zip(confirmed).all(|(expected, received)| {
            expected.source_well == received.source_well
                && expected.logical_files == received.logical_files
                && expected.target_well_name == received.target_well_name
                && received.target_well_id.is_none()
                && expected.will_create == received.will_create
        })
}

fn screen_dlis_values(
    mnemonic: &str,
    channel_name: &str,
    values: &mut [f32],
    disable_las_sentinel_for: &[String],
) -> Vec<DlisSkip> {
    let las_sentinel_disabled = disable_las_sentinel_for
        .iter()
        .any(|name| name.trim().eq_ignore_ascii_case(mnemonic));
    let mut nonfinite = 0usize;
    let mut excessive_magnitude = 0usize;
    let mut las_sentinel = 0usize;
    for value in values {
        if !value.is_finite() {
            *value = f32::NAN;
            nonfinite += 1;
        } else if value.abs() > 1e30 {
            *value = f32::NAN;
            excessive_magnitude += 1;
        } else if !las_sentinel_disabled && crate::parsers::is_las_null(*value) {
            *value = f32::NAN;
            las_sentinel += 1;
        }
    }

    let mut screened = Vec::new();
    for (count, rule) in [
        (nonfinite, "non-finite value; stored as missing"),
        (excessive_magnitude, "absolute magnitude above 1e30; stored as missing"),
        (las_sentinel, "recognized LAS sentinel fallback; stored as missing"),
    ] {
        if count > 0 {
            screened.push(DlisSkip {
                kind: "sample".into(),
                name: channel_name.into(),
                count,
                rule: rule.into(),
                omitted: false,
            });
        }
    }
    screened
}

fn stored_interval(conn: &Connection, well_id: &str, set_name: Option<&str>) -> Option<(f32, f32)> {
    let row: Option<(Option<f32>, Option<f32>)> = match set_name {
        Some(set) => conn
            .query_row(
                "SELECT CAST(MIN(s.depth) AS FLOAT), CAST(MAX(s.depth) AS FLOAT)
                 FROM curve_samples s
                 JOIN curve_meta m ON m.curve_id = s.curve_id
                 WHERE m.well_id = ?1 AND upper(m.set_name) = upper(?2)",
                params![well_id, set],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok(),
        None => conn
            .query_row(
                "SELECT CAST(MIN(depth) AS FLOAT), CAST(MAX(depth) AS FLOAT)
                 FROM (
                     SELECT depth FROM standard_curves WHERE well_id = ?1
                     UNION ALL
                     SELECT s.depth FROM curve_samples s
                     JOIN curve_meta m ON m.curve_id = s.curve_id WHERE m.well_id = ?1
                     UNION ALL
                     SELECT depth FROM computed_curves WHERE well_id = ?1
                 ) held",
                params![well_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok(),
    };
    row.and_then(|(top, base)| top.zip(base))
}

fn detect_interval_conflicts(
    conn: &Connection,
    well_id: &str,
    target_set: &str,
    incoming_top: f32,
    incoming_base: f32,
) -> Vec<DlisIntervalConflict> {
    let mut conflicts = Vec::new();
    if let Some((top, base)) = stored_interval(conn, well_id, None) {
        if incoming_top < top || incoming_base > base {
            conflicts.push(DlisIntervalConflict {
                scope: "well".into(),
                name: well_id.to_string(),
                declared_top: top,
                declared_base: base,
                incoming_top,
                incoming_base,
            });
        }
    }
    if let Some((top, base)) = stored_interval(conn, well_id, Some(target_set)) {
        if incoming_top < top || incoming_base > base {
            conflicts.push(DlisIntervalConflict {
                scope: "set".into(),
                name: target_set.to_string(),
                declared_top: top,
                declared_base: base,
                incoming_top,
                incoming_base,
            });
        }
    }
    conflicts
}

fn incoming_interval(
    header: &DlisHeader,
    payload: &[u8],
    index_actions: &[crate::units::IndexUnitAction],
    duplicate_decisions: &[DlisDuplicateDecisionRecord],
) -> Result<Option<(f32, f32)>, String> {
    let mut offset = 0usize;
    let mut top = f32::INFINITY;
    let mut base = f32::NEG_INFINITY;
    for (meta, index_action) in header.curves.iter().zip(index_actions.iter()) {
        let bytes = meta.n * 4;
        let end = offset + 2 * bytes;
        if end > payload.len() {
            return Err(format!("dlis payload truncated at curve '{}'", meta.mnemonic));
        }
        let skipped = duplicate_decisions.iter().any(|decision| {
            decision.mnemonic.eq_ignore_ascii_case(&meta.mnemonic)
                && decision.run == meta.run
                && decision.action == DlisDuplicateAction::SkipIncoming
        });
        if skipped {
            offset = end;
            continue;
        }
        let mut depth = read_f32(&payload[offset..offset + bytes]);
        apply_index_action(&mut depth, index_action);
        for value in depth.into_iter().filter(|value| value.is_finite()) {
            top = top.min(value);
            base = base.max(value);
        }
        offset = end;
    }
    Ok((top.is_finite() && base.is_finite()).then_some((top, base)))
}

fn interval_preflight(
    conn: &Connection,
    well_id: &str,
    target_set: &str,
    incoming_top: f32,
    incoming_base: f32,
    decision: Option<DlisOutsideIntervalDecision>,
) -> Result<Vec<DlisIntervalConflict>, Vec<DlisIntervalConflict>> {
    let conflicts = detect_interval_conflicts(conn, well_id, target_set, incoming_top, incoming_base);
    if conflicts.is_empty() || decision == Some(DlisOutsideIntervalDecision::AcceptOutsideDeclaredInterval) {
        Ok(conflicts)
    } else {
        Err(conflicts)
    }
}

fn duplicate_conflicts(
    conn: &Connection,
    well_id: &str,
    curves: &[DlisCurveMeta],
) -> Vec<DlisDuplicateConflict> {
    let mut conflicts = Vec::new();
    for meta in curves {
        let mut stmt = match conn.prepare(
            "SELECT set_name, run_no FROM curve_meta
             WHERE well_id = ?1 AND upper(mnemonic) = upper(?2)
             ORDER BY set_name, run_no NULLS FIRST, curve_id",
        ) {
            Ok(stmt) => stmt,
            Err(_) => continue,
        };
        let existing: Vec<String> = match stmt.query_map(params![well_id, &meta.mnemonic], |r| {
            let set: String = r.get(0)?;
            let run: Option<i32> = r.get(1)?;
            Ok(format!("{set}/run {}", run.map(|n| n.to_string()).unwrap_or_else(|| "none".into())))
        }) {
            Ok(rows) => rows.flatten().collect(),
            Err(_) => Vec::new(),
        };
        if !existing.is_empty() {
            conflicts.push(DlisDuplicateConflict {
                mnemonic: meta.mnemonic.trim().to_uppercase(),
                run: meta.run,
                existing,
            });
        }
    }
    conflicts
}

fn duplicate_preflight(
    conn: &Connection,
    well_id: &str,
    desired_set: &str,
    initial_target_set: &str,
    curves: &[DlisCurveMeta],
    decisions: &[DlisDuplicateDecision],
) -> Result<(String, Vec<DlisDuplicateDecisionRecord>), Vec<DlisDuplicateConflict>> {
    let conflicts = duplicate_conflicts(conn, well_id, curves);
    let unresolved: Vec<DlisDuplicateConflict> = conflicts
        .iter()
        .filter(|conflict| {
            decisions
                .iter()
                .filter(|decision| {
                    decision.mnemonic.trim().eq_ignore_ascii_case(&conflict.mnemonic)
                        && decision.run == conflict.run
                })
                .count()
                != 1
        })
        .cloned()
        .collect();
    if !unresolved.is_empty() {
        return Err(unresolved);
    }

    let needs_fresh_set = conflicts.iter().any(|conflict| {
        let action = decisions
            .iter()
            .find(|decision| {
                decision.mnemonic.trim().eq_ignore_ascii_case(&conflict.mnemonic)
                    && decision.run == conflict.run
            })
            .map(|decision| decision.action);
        action == Some(DlisDuplicateAction::KeepSeparate)
            && conn
                .query_row(
                    "SELECT 1 FROM curve_meta
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                       AND upper(mnemonic) = upper(?3) AND run_no IS NOT DISTINCT FROM ?4",
                    params![well_id, desired_set, &conflict.mnemonic, Some(conflict.run)],
                    |_| Ok(()),
                )
                .is_ok()
    });
    let target_set = if needs_fresh_set {
        crate::ingest::resolve_set_name(conn, well_id, desired_set)
    } else {
        initial_target_set.to_string()
    };

    let records = conflicts
        .into_iter()
        .map(|conflict| {
            let action = decisions
                .iter()
                .find(|decision| {
                    decision.mnemonic.trim().eq_ignore_ascii_case(&conflict.mnemonic)
                        && decision.run == conflict.run
                })
                .expect("unresolved decisions returned above")
                .action;
            DlisDuplicateDecisionRecord {
                mnemonic: conflict.mnemonic,
                run: conflict.run,
                action,
                existing: conflict.existing,
                target_set: (action == DlisDuplicateAction::KeepSeparate).then(|| target_set.clone()),
            }
        })
        .collect();
    Ok((target_set, records))
}

/// Imports scalar DLIS channels. A one-source container targets one existing well; a container
/// naming multiple source wells returns a source-to-project mapping first, then creates and routes
/// one project well per source only when the exact map is confirmed.
///
/// `set_name` (import-sets, T-IMP-02/06): named sets are auto-suffixed per well (`WIRE` taken
/// -> `WIRE_1`, Geolog-style). A mnemonic already held anywhere on the well stops before commit
/// until every incoming `(mnemonic, frame)` has a keep-separate or skip decision; no merge action
/// exists on this path.
#[allow(dead_code)] // compatibility entry point; the command supplies an explicit ambiguity answer
pub fn import_dlis_file(
    conn: &Connection,
    well_id: &str,
    path: &str,
    set_name: Option<&str>,
    confirmed_file_unit: Option<&str>,
) -> DlisImportResult {
    import_dlis_file_with_unit_designation(
        conn,
        Some(well_id),
        path,
        set_name,
        confirmed_file_unit,
        None,
        None,
        &[],
        &[],
        &[],
    )
}

pub fn import_dlis_file_with_unit_designation(
    conn: &Connection,
    well_id: Option<&str>,
    path: &str,
    set_name: Option<&str>,
    confirmed_file_unit: Option<&str>,
    ms_per_ft_meaning: Option<crate::curves::MsPerFtMeaning>,
    outside_interval_decision: Option<DlisOutsideIntervalDecision>,
    duplicate_decisions: &[DlisDuplicateDecision],
    las_sentinel_exceptions: &[String],
    confirmed_well_mappings: &[DlisWellMapping],
) -> DlisImportResult {
    let fail = |e: String| failed(path, e, Vec::new());

    let Some(python) = find_python() else {
        return fail(installation::capability_message(
            installation::CAPABILITY_DLIS_IMPORT,
            None,
            None,
        ));
    };
    if let Err(error) = installation::require_python_capability(
        &python,
        installation::CAPABILITY_DLIS_IMPORT,
    ) {
        return fail(error);
    }

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
        return fail(if last.contains("dlis-dependency-missing") {
            installation::capability_message(
                installation::CAPABILITY_DLIS_IMPORT,
                Some(&python),
                None,
            )
        } else {
            last.trim().to_string()
        });
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
    let proposed_well_mappings = match multi_well_plan(&header) {
        Ok(mapping) => mapping,
        Err(error) => return failed(path, error, header.skips),
    };
    let is_multi_well = !proposed_well_mappings.is_empty();
    if is_multi_well
        && !mapping_confirmation_matches(&proposed_well_mappings, confirmed_well_mappings)
    {
        return failed_mapping(path, proposed_well_mappings, header.skips);
    }
    let selected_well_id = if is_multi_well {
        well_id.unwrap_or("")
    } else {
        let Some(well_id) = well_id else {
            return failed(
                path,
                "single-well DLIS requires a selected project well; no data was written".into(),
                header.skips,
            );
        };
        let exists: bool = conn
            .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
            .unwrap_or(false);
        if !exists {
            return failed(path, format!("unknown well '{well_id}'"), header.skips);
        }
        well_id
    };
    let desired = crate::ingest::canonical_set_name(set_name);
    let initial_target_set = if is_multi_well || desired == "RAW" {
        desired.clone()
    } else {
        crate::ingest::resolve_set_name(conn, selected_well_id, &desired)
    };
    let ambiguous: Vec<&DlisCurveMeta> = header
        .curves
        .iter()
        .filter(|meta| crate::curves::is_ms_per_ft(Some(&meta.unit)))
        .collect();
    if let Some(first) = ambiguous.first() {
        let Some(meaning) = ms_per_ft_meaning else {
            return failed(
                path,
                format!(
                    "curve {} declares {}; this can mean microseconds per foot or millisiemens per foot, so a per-file user designation is required before commit",
                    first.mnemonic, first.unit
                ),
                header.skips,
            );
        };
        // The answer is recorded below after `notes` is initialized; keeping the
        // ambiguity check here guarantees no curve can be written first.
        debug_assert!(ambiguous.iter().all(|meta| crate::curves::is_ms_per_ft(Some(&meta.unit))));
        let _ = meaning;
    }
    let mut skipped = header.skips.clone();
    let (target_set, duplicate_decision_records) = if is_multi_well {
        (desired.clone(), Vec::new())
    } else {
        match duplicate_preflight(
            conn,
            selected_well_id,
            &desired,
            &initial_target_set,
            &header.curves,
            duplicate_decisions,
        ) {
            Ok(resolved) => resolved,
            Err(conflicts) => return failed_duplicates(path, conflicts, skipped),
        }
    };
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
    let mut committed_well_mappings = proposed_well_mappings;
    if is_multi_well {
        for mapping in &mut committed_well_mappings {
            mapping.target_well_id = Some(uuid::Uuid::new_v4().to_string());
        }
    }
    let interval_conflicts = if is_multi_well {
        Vec::new()
    } else {
        match incoming_interval(
            &header,
            payload,
            &index_actions,
            &duplicate_decision_records,
        ) {
            Ok(Some((top, base))) => match interval_preflight(
                conn,
                selected_well_id,
                &target_set,
                top,
                base,
                outside_interval_decision,
            ) {
                Ok(conflicts) => conflicts,
                Err(conflicts) => return failed_interval(path, conflicts, skipped),
            },
            Ok(None) => Vec::new(),
            Err(error) => return failed(path, error, skipped),
        }
    };
    notes.extend(interval_conflicts.iter().map(|conflict| {
        format!(
            "Accepted DLIS interval conflict for {} '{}': declared {:.4}-{:.4}, incoming {:.4}-{:.4}",
            conflict.scope,
            conflict.name,
            conflict.declared_top,
            conflict.declared_base,
            conflict.incoming_top,
            conflict.incoming_base
        )
    }));
    notes.extend(duplicate_decision_records.iter().map(|record| match record.action {
        DlisDuplicateAction::KeepSeparate => format!(
            "Duplicate {} frame {} kept separate in set {} (existing: {})",
            record.mnemonic,
            record.run,
            record.target_set.as_deref().unwrap_or(""),
            record.existing.join(", ")
        ),
        DlisDuplicateAction::SkipIncoming => format!(
            "Duplicate {} frame {} skipped by explicit choice (existing: {})",
            record.mnemonic,
            record.run,
            record.existing.join(", ")
        ),
    }));
    notes.extend(committed_well_mappings.iter().map(|mapping| {
        format!(
            "DLIS source well {} (logical files {:?}) created as project well {}",
            mapping.source_well, mapping.logical_files, mapping.target_well_name
        )
    }));
    let unit_designations: Vec<crate::curves::UnitDesignation> = ms_per_ft_meaning
        .map(|meaning| {
            ambiguous
                .iter()
                .map(|meta| crate::curves::ms_per_ft_designation(&meta.mnemonic, &meta.unit, meaning))
                .collect()
        })
        .unwrap_or_default();
    notes.extend(unit_designations.iter().map(crate::curves::UnitDesignation::note));

    // Each curve occupies 2 * n * 4 bytes (depth column then value column).
    let mut offset = 0usize;
    let mut curves_imported = 0usize;
    let mut total_rows = 0usize;
    let replaced = 0usize;
    let mut unit_conversions = Vec::new();
    let mut unconverted_units = Vec::new();
    let mut prepared_curves = Vec::new();
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

        if duplicate_decision_records.iter().any(|record| {
            record.mnemonic.eq_ignore_ascii_case(&meta.mnemonic)
                && record.run == meta.run
                && record.action == DlisDuplicateAction::SkipIncoming
        }) {
            skipped.push(DlisSkip {
                kind: "curve".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: 1,
                rule: "incoming duplicate skipped by explicit per-curve choice".into(),
                omitted: true,
            });
            continue;
        }

        // DLIS absent/sentinel values arrive as non-finite or huge magnitudes; normalize to
        // NaN (the project-wide missing convention). Producers also embed LAS-style
        // -999.25/-9999 sentinels (RP66 has no standard null), but that fallback can be disabled
        // for an exact channel. Screen BEFORE unit canonicalization so a sentinel cannot be
        // unit-scaled into an unrecognizable value.
        let channel_name = format!("frame {} curve {}", meta.run, meta.mnemonic);
        skipped.extend(screen_dlis_values(
            &meta.mnemonic,
            &channel_name,
            &mut values,
            las_sentinel_exceptions,
        ));

        let mut unit = if meta.unit.trim().is_empty() { None } else { Some(meta.unit.clone()) };
        let resolved_ms_per_ft = crate::curves::is_ms_per_ft(Some(&meta.unit));
        let (fam, rejected_alias) = if resolved_ms_per_ft {
            match ms_per_ft_meaning.expect("ambiguity checked before writes") {
                crate::curves::MsPerFtMeaning::MicrosecondsPerFoot => {
                    let family = crate::curves::family_for(&meta.mnemonic)
                        .filter(|family| matches!(family.family, "DT" | "DTS"));
                    unit = Some("us/ft".to_string());
                    (family, None)
                }
                crate::curves::MsPerFtMeaning::MillisiemensPerFoot => (None, None),
            }
        } else {
            crate::curves::family_for_import(&meta.mnemonic, Some(&meta.unit))
        };
        let family = fam.map(|f| f.family);
        if let Some(rejected) = rejected_alias {
            notes.push(rejected.note());
            unconverted_units.push(rejected);
        } else if resolved_ms_per_ft {
            // The explicit designation above owns both the label and family decision.
        } else if let Some(f) = fam {
            if let Some(conversion) = crate::curves::convert_to_canonical(
                &meta.mnemonic,
                f.family,
                unit.as_deref(),
                &mut values,
            ) {
                unit = Some(f.canonical_unit.to_string());
                notes.push(conversion.note());
                unit_conversions.push(conversion);
            } else if let Some(unconverted) = crate::curves::unconverted_unit(
                &meta.mnemonic,
                Some(f.family),
                unit.as_deref(),
            ) {
                notes.push(unconverted.note());
                unconverted_units.push(unconverted);
            }
        } else if let Some(unconverted) =
            crate::curves::unconverted_unit(&meta.mnemonic, None, unit.as_deref())
        {
            notes.push(unconverted.note());
            unconverted_units.push(unconverted);
        }
        // Give DLIS frames their own run numbering (frame 0 → run 0). The old frame-0 → NULL
        // mapping collided with LAS RAW curves (also run_no NULL), so a DLIS silently
        // overwrote same-mnemonic LAS curves. Using Some(run) keeps both, preserving provenance.
        let run_no = Some(meta.run);

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
                omitted: false,
            });
        }
        if dreport.duplicate > 0 {
            skipped.push(DlisSkip {
                kind: "row".into(),
                name: format!("frame {} curve {}", meta.run, meta.mnemonic),
                count: dreport.duplicate,
                rule: "duplicate depth index; first occurrence kept".into(),
                omitted: false,
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
                omitted: true,
            });
            continue;
        }

        let (curve_well_id, curve_set) = if is_multi_well {
            let Some(mapping) = committed_well_mappings
                .iter()
                .find(|mapping| mapping.logical_files.contains(&meta.logical_file))
            else {
                return failed(
                    path,
                    format!(
                        "DLIS curve '{}' belongs to logical file {}, which is absent from the confirmed well mapping",
                        meta.mnemonic, meta.logical_file
                    ),
                    skipped,
                );
            };
            (
                mapping
                    .target_well_id
                    .as_deref()
                    .expect("a confirmed multi-well mapping has a generated target id"),
                desired.as_str(),
            )
        } else {
            (selected_well_id, target_set.as_str())
        };
        total_rows += depth.len();
        curves_imported += 1;
        prepared_curves.push(PreparedDlisCurve {
            well_id: curve_well_id.to_string(),
            set_name: curve_set.to_string(),
            mnemonic: meta.mnemonic.clone(),
            unit,
            family,
            run_no,
            depth,
            values,
        });
    }

    if curves_imported == 0 {
        return failed(
            path,
            format!("DLIS import produced no curves after validation: {}", skip_summary(&skipped)),
            skipped,
        );
    }

    let write_result = write_prepared_dlis(
        conn,
        &committed_well_mappings,
        &prepared_curves,
        project_unit.or(adopted_unit),
    );
    if let Err(error) = write_result {
        return failed(path, format!("storing DLIS curves: {error}"), skipped);
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
        status: import_status(&header, curves_imported, &skipped),
        channels_declared: header.channels_declared,
        curves_imported,
        rows: total_rows,
        replaced,
        notes,
        unit_conversions,
        unconverted_units,
        unit_designations,
        skipped,
        sentinel_exceptions: las_sentinel_exceptions
            .iter()
            .map(|name| name.trim().to_ascii_uppercase())
            .filter(|name| !name.is_empty())
            .collect(),
        well_mappings: committed_well_mappings,
        mapping_confirmation_required: false,
        interval_conflicts,
        duplicate_conflicts: Vec::new(),
        duplicate_decisions: duplicate_decision_records,
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

    /// **A DLIS outside an existing well's declared range requires confirmation before any
    /// write.** `SB-DIO-035` / T51, sourced to data-I/O finding D-34. Pinned from both sides: an
    /// inside interval passes without a decision, an outside interval is returned as a conflict
    /// and leaves the held extent unchanged, and only the named acceptance releases that same
    /// conflict.
    #[test]
    fn a_dlis_outside_an_existing_wells_declared_range_requires_confirmation_before_any_write() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "DLIS-RANGE", None, None, None).unwrap();
        db::insert_standard_curves(
            &conn,
            well_id,
            vec![1000.0, 1001.0, 1002.0],
            vec![1.0, 2.0, 3.0],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        let well = well_id.to_string();
        assert_eq!(stored_interval(&conn, &well, None), Some((1000.0, 1002.0)));
        assert!(
            interval_preflight(&conn, &well, "WIRE", 1000.0, 1002.0, None)
                .unwrap()
                .is_empty(),
            "an interval exactly inside the declaration needs no confirmation"
        );

        let conflict = interval_preflight(&conn, &well, "WIRE", 999.0, 1003.0, None).unwrap_err();
        assert_eq!(conflict.len(), 1);
        assert_eq!(conflict[0].scope, "well");
        assert_eq!((conflict[0].declared_top, conflict[0].declared_base), (1000.0, 1002.0));
        assert_eq!((conflict[0].incoming_top, conflict[0].incoming_base), (999.0, 1003.0));
        assert_eq!(
            stored_interval(&conn, &well, None),
            Some((1000.0, 1002.0)),
            "the refusal happens before any sample can widen the held interval"
        );

        let accepted = interval_preflight(
            &conn,
            &well,
            "WIRE",
            999.0,
            1003.0,
            Some(DlisOutsideIntervalDecision::AcceptOutsideDeclaredInterval),
        )
        .unwrap();
        assert_eq!(accepted, conflict, "the accepted run retains the exact conflict as its audit record");
    }

    /// **An incoming DLIS mnemonic requires a recorded per-curve choice and never defaults to
    /// merge.** `SB-DIO-036` / T52, sourced to data-I/O finding D-34. The undecided preflight is a
    /// refusal with the old samples intact; `keep_separate` resolves an exact RAW collision to a
    /// fresh set, while `skip_incoming` is recorded against that exact mnemonic and frame.
    #[test]
    fn an_incoming_dlis_mnemonic_requires_a_recorded_per_curve_choice_and_never_defaults_to_merge() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "DLIS-DUP", None, None, None).unwrap();
        let well = well_id.to_string();
        let existing = db::upsert_curve_meta(
            &conn,
            &well,
            "RAW",
            "GR",
            Some("GAPI"),
            Some("GR"),
            Some("earlier delivery"),
            Some(0),
        )
        .unwrap();
        db::insert_curve_samples(&conn, &existing, &[1000.0, 1001.0], &[41.0, 42.0]).unwrap();
        let incoming = vec![DlisCurveMeta {
            mnemonic: "GR".into(),
            unit: "GAPI".into(),
            index_unit: "M".into(),
            n: 2,
            run: 0,
            logical_file: 0,
        }];

        let unresolved = duplicate_preflight(&conn, &well, "RAW", "RAW", &incoming, &[]).unwrap_err();
        assert_eq!(unresolved.len(), 1);
        assert_eq!((unresolved[0].mnemonic.as_str(), unresolved[0].run), ("GR", 0));
        assert_eq!(unresolved[0].existing, vec!["RAW/run 0"]);
        let old = db::get_curve_samples(&conn, &existing).unwrap();
        assert_eq!(old.iter().map(|sample| sample.value).collect::<Vec<_>>(), vec![41.0, 42.0]);

        let keep = [DlisDuplicateDecision {
            mnemonic: "GR".into(),
            run: 0,
            action: DlisDuplicateAction::KeepSeparate,
        }];
        let (target, records) = duplicate_preflight(&conn, &well, "RAW", "RAW", &incoming, &keep).unwrap();
        assert_eq!(target, "RAW_1", "keep-separate must not reuse the colliding identity");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, DlisDuplicateAction::KeepSeparate);
        assert_eq!(records[0].target_set.as_deref(), Some("RAW_1"));

        let skip = [DlisDuplicateDecision {
            mnemonic: "GR".into(),
            run: 0,
            action: DlisDuplicateAction::SkipIncoming,
        }];
        let (target, records) = duplicate_preflight(&conn, &well, "RAW", "RAW", &incoming, &skip).unwrap();
        assert_eq!(target, "RAW");
        assert_eq!(records[0].action, DlisDuplicateAction::SkipIncoming);
        assert!(records[0].target_set.is_none(), "a skipped curve has no invented destination");
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
            logical_file: 0,
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
            logical_file: 0,
        };
        let one_bad_one_good = DlisHeader {
            curves: vec![good],
            skips: vec![DlisSkip {
                kind: "frame".into(),
                name: "FRAME-BAD".into(),
                count: 1,
                rule: "frame.curves failed: RuntimeError: encrypted".into(),
                omitted: true,
            }],
            channels_declared: 2,
            logical_files: Vec::new(),
        };
        assert!(validate_header(&one_bad_one_good).is_ok(), "one good frame must survive a bad one");
        let all_bad = DlisHeader {
            curves: vec![],
            skips: one_bad_one_good.skips.clone(),
            channels_declared: 1,
            logical_files: Vec::new(),
        };
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

    /// SB-DIO-037 / SB-DIO-T53. Partial-file status and named unreadable channels are
    /// specified in `docs/PRD_v2/21_data-io.md` §§4.7 and 6.7 (DLIS D-34).
    #[test]
    fn a_dlis_with_one_unreadable_and_one_readable_channel_is_partial_and_names_the_unreadable_channel() {
        let readable = DlisCurveMeta {
            mnemonic: "GR".into(),
            unit: "GAPI".into(),
            index_unit: "M".into(),
            n: 2,
            run: 0,
            logical_file: 0,
        };
        let unreadable = DlisSkip {
            kind: "channel".into(),
            name: "LOCKED_RES".into(),
            count: 1,
            rule: "frame MAIN unreadable: RuntimeError: encrypted".into(),
            omitted: true,
        };
        let partial = DlisHeader {
            curves: vec![readable],
            skips: vec![unreadable],
            channels_declared: 2,
            logical_files: Vec::new(),
        };
        assert_eq!(import_status(&partial, 1, &partial.skips), DlisImportStatus::Partial);
        assert_eq!(partial.skips[0].name, "LOCKED_RES");

        // The opposite side: a retained channel's attribute warning is reported, but it does
        // not turn a fully loaded delivery into a partial load.
        let retained_warning = DlisSkip {
            kind: "channel".into(),
            name: "GR".into(),
            count: 1,
            rule: "UNITS attribute unreadable; channel retained with no unit".into(),
            omitted: false,
        };
        let complete = DlisHeader {
            curves: partial.curves,
            skips: vec![retained_warning],
            channels_declared: 1,
            logical_files: Vec::new(),
        };
        assert_eq!(import_status(&complete, 1, &complete.skips), DlisImportStatus::Complete);
        assert!(DLIS_RUNNER.contains("for name in payload_names"));
        assert!(DLIS_RUNNER.contains("\"omitted\": bool(omitted)"));
    }

    /// SB-DIO-045 / SB-DIO-T63..T64. Multi-well separation and the pre-commit mapping are
    /// specified in `docs/PRD_v2/21_data-io.md` §§4.9 and 6.9 (D-27).
    #[test]
    fn a_three_well_dlis_shows_its_logical_file_mapping_before_commit_and_creates_three_wells_without_merging() {
        assert!(DLIS_RUNNER.contains("well_name = attr_text(defining_origin, \"well_name\")"));
        assert!(DLIS_RUNNER.contains("\"logical_file\": logical_ord"));
        let logical_files = ["SANDI-A", "SANDI-B", "SANDI-C"]
            .into_iter()
            .enumerate()
            .map(|(logical_file, source_well)| DlisLogicalFile {
                logical_file,
                source_well: source_well.into(),
            })
            .collect::<Vec<_>>();
        let curves = (0..3)
            .map(|logical_file| DlisCurveMeta {
                mnemonic: "GR".into(),
                unit: "GAPI".into(),
                index_unit: "M".into(),
                n: 2,
                run: logical_file as i32,
                logical_file,
            })
            .collect();
        let header = DlisHeader {
            curves,
            skips: Vec::new(),
            channels_declared: 3,
            logical_files,
        };
        let proposed = multi_well_plan(&header).unwrap();
        assert_eq!(proposed.len(), 3);
        assert_eq!(proposed[0].source_well, "SANDI-A");
        assert_eq!(proposed[2].logical_files, vec![2]);

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let preview = failed_mapping("three-well.dlis", proposed.clone(), Vec::new());
        assert!(preview.mapping_confirmation_required);
        assert_eq!(preview.well_mappings, proposed);
        let before: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        assert_eq!(before, 0, "showing the mapping cannot commit even the well shells");
        assert!(mapping_confirmation_matches(&proposed, &preview.well_mappings));

        let mut committed = proposed;
        for mapping in &mut committed {
            mapping.target_well_id = Some(Uuid::new_v4().to_string());
        }
        let prepared = committed
            .iter()
            .enumerate()
            .map(|(logical_file, mapping)| PreparedDlisCurve {
                well_id: mapping.target_well_id.clone().unwrap(),
                set_name: "RAW".into(),
                mnemonic: "GR".into(),
                unit: Some("GAPI".into()),
                family: Some("GR"),
                run_no: Some(logical_file as i32),
                depth: vec![1000.0, 1000.5],
                values: vec![50.0 + logical_file as f32, 51.0 + logical_file as f32],
            })
            .collect::<Vec<_>>();
        write_prepared_dlis(
            &conn,
            &committed,
            &prepared,
            Some(crate::units::DepthUnit::Metres),
        )
        .unwrap();
        let well_count: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |row| row.get(0)).unwrap();
        let curve_wells: i64 = conn
            .query_row("SELECT COUNT(DISTINCT well_id) FROM curve_meta", [], |row| row.get(0))
            .unwrap();
        let most_curves_on_one_well: i64 = conn
            .query_row(
                "SELECT MAX(curves) FROM (SELECT COUNT(*) curves FROM curve_meta GROUP BY well_id)",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((well_count, curve_wells, most_curves_on_one_well), (3, 3, 1));

        // The opposite side: two logical files for the same declared source well are two runs,
        // not two project wells. Grouping by logical-file ordinal alone would fail this.
        let same_well = DlisHeader {
            curves: Vec::new(),
            skips: Vec::new(),
            channels_declared: 0,
            logical_files: vec![
                DlisLogicalFile { logical_file: 0, source_well: "SANDI-A".into() },
                DlisLogicalFile { logical_file: 1, source_well: "sandi-a".into() },
            ],
        };
        assert!(multi_well_plan(&same_well).unwrap().is_empty());
    }

    /// SB-DIO-039 / SB-DIO-T56..T57. The per-channel exception and per-rule deletion count
    /// are specified in `docs/PRD_v2/21_data-io.md` §§4.7 and 6.7 (D-5).
    #[test]
    fn a_named_dlis_sentinel_exception_preserves_minus_999_25_while_the_default_screens_and_counts_it() {
        let exceptions = vec!["AMPLITUDE".to_string()];
        let mut excepted = vec![-999.25_f32, 12.0];
        let excepted_report = screen_dlis_values(
            "AMPLITUDE",
            "frame 2 curve AMPLITUDE",
            &mut excepted,
            &exceptions,
        );
        assert_eq!(excepted[0], -999.25, "the explicitly excepted finite sample remains data");
        assert!(excepted_report.is_empty());

        let mut screened = vec![-999.25_f32, 12.0];
        let screened_report = screen_dlis_values(
            "AMPLITUDE",
            "frame 2 curve AMPLITUDE",
            &mut screened,
            &[],
        );
        assert!(screened[0].is_nan());
        assert_eq!(screened_report.len(), 1);
        assert_eq!(screened_report[0].name, "frame 2 curve AMPLITUDE");
        assert_eq!(screened_report[0].count, 1);
        assert_eq!(screened_report[0].rule, "recognized LAS sentinel fallback; stored as missing");

        // The opposite side: disabling only the LAS-derived fallback must not disable the
        // separately sourced non-finite and magnitude screens.
        let mut still_invalid = vec![f32::INFINITY, 1.1e30_f32];
        let invalid_report = screen_dlis_values(
            "AMPLITUDE",
            "frame 2 curve AMPLITUDE",
            &mut still_invalid,
            &exceptions,
        );
        assert!(still_invalid.iter().all(|value| value.is_nan()));
        assert_eq!(invalid_report.iter().map(|item| item.count).sum::<usize>(), 2);
        assert!(invalid_report.iter().any(|item| item.rule.contains("non-finite")));
        assert!(invalid_report.iter().any(|item| item.rule.contains("1e30")));
    }
}
