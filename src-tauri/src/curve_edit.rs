//! Interactive curve editing for the log view's right-click menu (P2-d): wireline
//! shift, set-constant / blank / interpolate / scale over a depth interval.
//!
//! Edits work on whichever store actually holds the curve — a `standard_curves`
//! column, `computed_curves`, or the generic RAW store (`curve_meta`/`curve_samples`,
//! matched by mnemonic then family like the module input resolver). Every op is a
//! read-modify-rewrite of the curve's own rows: values are transformed in memory on
//! the curve's native depth grid, then the rows are deleted and re-appended in one
//! transaction — no floating-point depth matching against SQL literals anywhere.
//!
//! `edit_curve` returns the PREVIOUS (depth, value) pairs of every changed sample so
//! the frontend can push an exact undo; `restore_curve_values` writes such pairs back.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::equations;

#[derive(Debug, Clone, Deserialize)]
pub struct CurveEditRequest {
    pub well_id: String,
    pub curve: String,
    /// "shift" | "set" | "blank" | "interpolate" | "scale"
    pub op: String,
    /// "shift": depth shift in depth units; positive moves the curve DOWN hole.
    #[serde(default)]
    pub delta: f32,
    /// Interval bounds for the interval ops (inclusive; swapped if reversed).
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub bottom: f32,
    /// "set": the constant written over the interval.
    #[serde(default)]
    pub value: f32,
    /// "scale": v' = mul * v + add over the interval.
    #[serde(default = "one")]
    pub mul: f32,
    #[serde(default)]
    pub add: f32
,
    /// Explicit actor and source/reference custody. It is required when the target is a
    /// computed curve because that edit creates a new derived-curve version.
    #[serde(default)]
    pub custody: Option<equations::RunCustody>,
}

fn one() -> f32 {
    1.0
}

/// The pre-edit samples of every CHANGED row (whole curve for a shift, just the
/// interval for interval ops) — the frontend's undo payload. Packed as raw bytes
/// (`depth[n]` then `value[n]`, f32 LE) per this project's IPC rule against bulk JSON
/// number arrays; bytes also carry NaN bit-exactly where JSON cannot.
#[derive(Debug, Clone, Serialize)]
pub struct CurveEditResult {
    pub affected: usize,
    /// "standard" | "computed" | "raw" — shown in the history entry.
    pub store: String,
    pub point_count: usize,
    pub data: Vec<u8>,
}

/// Packs (depth, value) into the shared `depth[n] + value[n]` f32-LE byte convention.
pub fn pack_pairs(depth: &[f32], value: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity((depth.len() + value.len()) * 4);
    for d in depth {
        out.extend_from_slice(&d.to_le_bytes());
    }
    for v in value {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// Inverse of `pack_pairs`.
pub fn unpack_pairs(point_count: usize, data: &[u8]) -> Result<(Vec<f32>, Vec<f32>), String> {
    if data.len() != point_count * 8 {
        return Err("malformed curve-edit payload".into());
    }
    let read = |off: usize| f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
    let depth = (0..point_count).map(|i| read(i * 4)).collect();
    let value = (0..point_count).map(|i| read((point_count + i) * 4)).collect();
    Ok((depth, value))
}

const STANDARD_COLUMNS: &[(&str, &str)] = &[
    ("GR", "gr"),
    ("RES_DEEP", "res_deep"),
    ("NPHI", "nphi"),
    ("RHOB", "rhob"),
    ("DT", "dt"),
    ("SP", "sp"),
];

enum CurveStore {
    /// Column name within `standard_curves`.
    Standard(&'static str),
    /// Exact stored `curve_name` in `computed_curves`.
    Computed(String),
    /// `curve_id` in the generic store.
    Generic(String),
}

impl CurveStore {
    fn label(&self) -> &'static str {
        match self {
            CurveStore::Standard(_) => "standard",
            CurveStore::Computed(_) => "computed",
            CurveStore::Generic(_) => "raw",
        }
    }
}

/// Resolves which store holds `curve` for this well, in the same precedence order the
/// viewer reads them: standard column, then computed, then generic RAW (mnemonic
/// before family, base run first).
fn locate_curve(conn: &Connection, well_id: &str, curve: &str) -> Result<CurveStore, String> {
    let upper = curve.trim().to_uppercase();

    if let Some((_, col)) = STANDARD_COLUMNS.iter().find(|(m, _)| *m == upper) {
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM standard_curves WHERE well_id = ?1",
                params![well_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if n > 0 {
            return Ok(CurveStore::Standard(col));
        }
    }

    let computed: Option<String> = conn
        .query_row(
            // ORDER BY makes the pick deterministic if a case-duplicate shadow row still exists
            // (the write-side case-normalization prevents new ones); no effect in the normal
            // one-row-per-upper(name) case.
            "SELECT curve_name FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = ?2 \
             ORDER BY curve_name LIMIT 1",
            params![well_id, upper],
            |r| r.get(0),
        )
        .ok();
    if let Some(name) = computed {
        return Ok(CurveStore::Computed(name));
    }

    let generic: Option<String> = conn
        .query_row(
            // pinned is scoped per mnemonic (db::promote_generic_curve), so gate it behind an
            // exact-mnemonic match — a family-name request must rank by run_no, not by a pin on a
            // different member mnemonic. curve_id is the final deterministic tiebreak. Mirrors
            // equations::fetch_generic_curve_aligned.
            "SELECT curve_id FROM curve_meta
             WHERE well_id = ?1 AND set_name = 'RAW'
               AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
             ORDER BY (upper(mnemonic) = ?2) DESC,
                      (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                      run_no NULLS FIRST,
                      curve_id
             LIMIT 1",
            params![well_id, upper],
            |r| r.get(0),
        )
        .ok();
    if let Some(curve_id) = generic {
        return Ok(CurveStore::Generic(curve_id));
    }

    Err(format!("curve '{curve}' has no data in this well"))
}

/// Reads the curve's native samples, sorted by depth (NULL → NaN).
fn read_curve(conn: &Connection, store: &CurveStore, well_id: &str) -> Result<(Vec<f32>, Vec<f32>), String> {
    let (sql, bind_well) = match store {
        CurveStore::Standard(col) => (
            format!("SELECT depth, {col} FROM standard_curves WHERE well_id = ?1 ORDER BY depth"),
            true,
        ),
        CurveStore::Computed(name) => (
            format!(
                "SELECT depth, value FROM computed_curves WHERE well_id = ?1 AND curve_name = '{}' ORDER BY depth",
                name.replace('\'', "''")
            ),
            true,
        ),
        CurveStore::Generic(curve_id) => (
            format!(
                "SELECT depth, value FROM curve_samples WHERE curve_id = '{}' ORDER BY depth",
                curve_id.replace('\'', "''")
            ),
            false,
        ),
    };
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let map_row = |row: &duckdb::Row| {
        Ok((
            row.get::<_, f32>(0)?,
            row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN),
        ))
    };
    let rows = if bind_well {
        stmt.query_map(params![well_id], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| e.to_string())?;

    let mut depth = Vec::new();
    let mut value = Vec::new();
    for r in rows {
        let (d, v) = r.map_err(|e| e.to_string())?;
        depth.push(d);
        value.push(v);
    }
    Ok((depth, value))
}

/// Rewrites the curve with `new_values` (same depth order as `read_curve` returned):
/// delete + re-append inside one transaction, preserving every other column.
fn write_curve(
    conn: &Connection,
    store: &CurveStore,
    well_id: &str,
    depth: &[f32],
    new_values: &[f32],
) -> Result<(), String> {
    if matches!(store, CurveStore::Computed(_)) {
        return Err(
            "computed curve edit refused: a new ancestry-bearing version is required".into(),
        );
    }
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    let result = write_curve_inner(conn, store, well_id, depth, new_values);
    match result {
        Ok(()) => conn.execute_batch("COMMIT").map_err(|e| e.to_string()),
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

fn write_curve_inner(
    conn: &Connection,
    store: &CurveStore,
    well_id: &str,
    depth: &[f32],
    new_values: &[f32],
) -> Result<(), String> {
    match store {
        CurveStore::Standard(col) => {
            // Read every column of the well's grid, patch the edited one, rewrite whole
            // rows. NaN in a nullable column is stored as NULL to keep the import
            // discipline (dt/sp arrive as NULL where absent).
            let mut stmt = conn
                .prepare("SELECT depth, gr, res_deep, nphi, rhob, dt, sp FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
                .map_err(|e| e.to_string())?;
            type Row = (f32, Option<f32>, Option<f32>, Option<f32>, Option<f32>, Option<f32>, Option<f32>);
            let rows: Vec<Row> = stmt
                .query_map(params![well_id], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            if rows.len() != depth.len() {
                return Err("curve changed while editing — retry".into());
            }
            let col_idx = ["gr", "res_deep", "nphi", "rhob", "dt", "sp"]
                .iter()
                .position(|c| c == col)
                .expect("known column");

            conn.execute("DELETE FROM standard_curves WHERE well_id = ?1", params![well_id])
                .map_err(|e| e.to_string())?;
            let mut appender = conn.appender("standard_curves").map_err(|e| e.to_string())?;
            for (i, (d, gr, res, nphi, rhob, dt, sp)) in rows.into_iter().enumerate() {
                let mut cols = [gr, res, nphi, rhob, dt, sp];
                cols[col_idx] = if new_values[i].is_nan() { None } else { Some(new_values[i]) };
                appender
                    .append_row(params![well_id, d, cols[0], cols[1], cols[2], cols[3], cols[4], cols[5]])
                    .map_err(|e| e.to_string())?;
            }
            appender.flush().map_err(|e| e.to_string())?;
        }
        CurveStore::Computed(_) => unreachable!("computed writers require complete ancestry")
                , CurveStore::Generic(curve_id) => {
            let mut stmt = conn
                .prepare("SELECT depth FROM curve_samples WHERE curve_id = ?1 ORDER BY depth")
                .map_err(|e| e.to_string())?;
            let depths: Vec<f32> = stmt
                .query_map(params![curve_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            if depths.len() != depth.len() {
                return Err("curve changed while editing — retry".into());
            }
            conn.execute("DELETE FROM curve_samples WHERE curve_id = ?1", params![curve_id])
                .map_err(|e| e.to_string())?;
            let mut appender = conn.appender("curve_samples").map_err(|e| e.to_string())?;
            for (i, d) in depths.into_iter().enumerate() {
                appender
                    .append_row(params![curve_id, d, new_values[i]])
                    .map_err(|e| e.to_string())?;
            }
            appender.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn write_computed_revision(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth: &[f32],
    new_values: &[f32],
    custody: &equations::RunCustody,
    parameters: serde_json::Value,
    zone_scope: equations::AncestryZoneScope,
) -> Result<(), String> {
    let output = curve_name.to_string();
    let spec = equations::complete_curve_run_spec(
        conn,
        well_id,
        "CURVE_EDIT",
        "CURVE_EDIT",
        custody,
        &[(
            well_id.to_string(),
            "edited_curve".into(),
            curve_name.to_string(),
        )],
        None,
        parameters,
        zone_scope,
        std::slice::from_ref(&output),
    )?;
    let (set_id, _) = equations::create_complete_log_set(conn, well_id, &spec)?;
    equations::write_computed_curves_with_ancestry(
        conn,
        well_id,
        depth,
        &[(curve_name, new_values)],
        &set_id,
    )
}

fn edit_parameters(req: &CurveEditRequest) -> serde_json::Value {
    let (top, bottom) = if req.top <= req.bottom {
        (req.top, req.bottom)
    } else {
        (req.bottom, req.top)
    };
    match req.op.as_str() {
        "shift" => serde_json::json!({ "operation": "shift", "delta": req.delta }),
        "set" => serde_json::json!({
            "operation": "set",
            "top": top,
            "bottom": bottom,
            "value": req.value,
        }),
        "blank" | "interpolate" => serde_json::json!({
            "operation": req.op,
            "top": top,
            "bottom": bottom,
        }),
        "scale" => serde_json::json!({
            "operation": "scale",
            "top": top,
            "bottom": bottom,
            "multiplier": req.mul,
            "offset": req.add,
        }),
        _ => serde_json::json!({ "operation": req.op }),
    }
}

fn edit_zone_scope(
    req: &CurveEditRequest,
    custody: &equations::RunCustody,
) -> equations::AncestryZoneScope {
    if req.op == "shift" {
        return equations::AncestryZoneScope::WholeWell;
    }
    let (top, bottom) = if req.top <= req.bottom {
        (req.top, req.bottom)
    } else {
        (req.bottom, req.top)
    };
    if top < bottom {
        equations::AncestryZoneScope::Defined(vec![equations::AncestryZone {
            name: "CURVE_EDIT_INTERVAL".into(),
            top,
            base: bottom,
            source: custody.source_note.trim().to_string(),
        }])
    } else {
        // A single-sample edit has no non-zero interval. Its exact depth remains a sourced
        // parameter; labelling it as a geological zone would be false custody.
        equations::AncestryZoneScope::WholeWell
    }
}

// ---------------------------------------------------------------------------
// The transforms (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Linear interpolation of (depth, value) at `target`. NaN outside coverage or when
/// either bracketing sample is NaN — a shift never invents data across gaps.
fn interp_at(depth: &[f32], value: &[f32], target: f32) -> f32 {
    let n = depth.len();
    if n == 0 || target < depth[0] || target > depth[n - 1] {
        return f32::NAN;
    }
    let i = depth.partition_point(|&d| d < target);
    if i < n && depth[i] == target {
        return value[i];
    }
    if i == 0 || i >= n {
        return f32::NAN;
    }
    let (d0, d1) = (depth[i - 1], depth[i]);
    let (v0, v1) = (value[i - 1], value[i]);
    if !v0.is_finite() || !v1.is_finite() || d1 == d0 {
        return f32::NAN;
    }
    v0 + (v1 - v0) * (target - d0) / (d1 - d0)
}

/// Wireline shift: the value originally logged at depth d appears at d + delta, so the
/// new value on the (unchanged) grid is the old curve sampled at d - delta.
pub fn apply_shift(depth: &[f32], value: &[f32], delta: f32) -> Vec<f32> {
    depth.iter().map(|&d| interp_at(depth, value, d - delta)).collect()
}

fn apply_in_range(depth: &[f32], value: &[f32], top: f32, bottom: f32, f: impl Fn(f32) -> f32) -> Vec<f32> {
    depth
        .iter()
        .zip(value.iter())
        .map(|(&d, &v)| if d >= top && d <= bottom { f(v) } else { v })
        .collect()
}

/// Replaces the samples strictly inside (top, bottom) with a straight line between the
/// nearest FINITE samples at-or-outside each edge — the standard gap-bridge / despike.
pub fn apply_interpolate(depth: &[f32], value: &[f32], top: f32, bottom: f32) -> Result<Vec<f32>, String> {
    let above = depth
        .iter()
        .zip(value.iter())
        .filter(|(&d, &v)| d <= top && v.is_finite())
        .last()
        .map(|(&d, &v)| (d, v));
    let below = depth
        .iter()
        .zip(value.iter())
        .find(|(&d, &v)| d >= bottom && v.is_finite())
        .map(|(&d, &v)| (d, v));
    let (Some((d0, v0)), Some((d1, v1))) = (above, below) else {
        return Err("no finite data at the interval edges to interpolate between".into());
    };
    Ok(depth
        .iter()
        .zip(value.iter())
        .map(|(&d, &v)| {
            if d > top && d < bottom {
                if d1 == d0 { v0 } else { v0 + (v1 - v0) * (d - d0) / (d1 - d0) }
            } else {
                v
            }
        })
        .collect())
}

fn apply_op(req: &CurveEditRequest, depth: &[f32], value: &[f32]) -> Result<Vec<f32>, String> {
    let (top, bottom) = if req.top <= req.bottom { (req.top, req.bottom) } else { (req.bottom, req.top) };
    match req.op.as_str() {
        "shift" => {
            if req.delta == 0.0 || !req.delta.is_finite() {
                return Err("shift needs a non-zero delta".into());
            }
            Ok(apply_shift(depth, value, req.delta))
        }
        "set" => {
            if !req.value.is_finite() {
                return Err("set needs a finite value (use blank to erase)".into());
            }
            Ok(apply_in_range(depth, value, top, bottom, |_| req.value))
        }
        "blank" => Ok(apply_in_range(depth, value, top, bottom, |_| f32::NAN)),
        "interpolate" => apply_interpolate(depth, value, top, bottom),
        "scale" => {
            if !req.mul.is_finite() || !req.add.is_finite() {
                return Err("scale needs finite factors".into());
            }
            Ok(apply_in_range(depth, value, top, bottom, |v| req.mul * v + req.add))
        }
        other => Err(format!("unknown edit op '{other}'")),
    }
}

// ---------------------------------------------------------------------------
// Entry points (called by the Tauri commands in lib.rs)
// ---------------------------------------------------------------------------

pub fn edit_curve(conn: &Connection, req: &CurveEditRequest) -> Result<CurveEditResult, String> {
    let store = locate_curve(conn, &req.well_id, &req.curve)?;
    let (depth, old) = read_curve(conn, &store, &req.well_id)?;
    if depth.is_empty() {
        return Err(format!("curve '{}' has no samples", req.curve));
    }
    let new = apply_op(req, &depth, &old)?;

    // NaN-aware change detection: NaN → NaN is "unchanged".
    let changed: Vec<usize> = (0..depth.len())
        .filter(|&i| old[i].to_bits() != new[i].to_bits() && !(old[i].is_nan() && new[i].is_nan()))
        .collect();
    if changed.is_empty() {
        return Ok(CurveEditResult { affected: 0, store: store.label().into(), point_count: 0, data: vec![] });
    }

    match &store {
        CurveStore::Computed(name) => {
            let custody = req.custody.as_ref().ok_or_else(|| {
                "computed curve edit refused: enter the session operator and source/reference"
                    .to_string()
            })?;
            write_computed_revision(
                conn,
                &req.well_id,
                name,
                &depth,
                &new,
                custody,
                edit_parameters(req),
                edit_zone_scope(req, custody),
            )?;
        }
        _ => write_curve(conn, &store, &req.well_id, &depth, &new)?}
    let prev_depth: Vec<f32> = changed.iter().map(|&i| depth[i]).collect();
    let prev_value: Vec<f32> = changed.iter().map(|&i| old[i]).collect();
    Ok(CurveEditResult {
        affected: changed.len(),
        store: store.label().into(),
        point_count: changed.len(),
        data: pack_pairs(&prev_depth, &prev_value),
    })
}

/// Writes explicit (depth, value) pairs back into a curve — the undo path for
/// `edit_curve`. Depths are matched bit-exactly (the packed bytes round-trip the f32
/// bits untouched); NaN values restore to NaN/NULL. Returns how many samples matched.
pub fn restore_curve_values(
    conn: &Connection,
    well_id: &str,
    curve: &str,
    depths: &[f32],
    values: &[f32]
,
    custody: Option<&equations::RunCustody>,
) -> Result<usize, String> {
    if depths.len() != values.len() {
        return Err("depth/value length mismatch".into());
    }
    let store = locate_curve(conn, well_id, curve)?;
    let (depth, mut value) = read_curve(conn, &store, well_id)?;
    let restore: std::collections::HashMap<u32, f32> = depths
        .iter()
        .zip(values.iter())
        .map(|(d, v)| (d.to_bits(), *v))
        .collect();
    let mut n = 0usize;
    let mut matched_depth = Vec::new();
    let mut matched_value = Vec::new();
    for (i, d) in depth.iter().enumerate() {
        if let Some(&v) = restore.get(&d.to_bits()) {
            value[i] = v;
            n += 1;
        matched_depth.push(*d);
            matched_value.push(v);
        }
    }
    if n == 0 {
        return Ok(0);
    }
    match &store {
        CurveStore::Computed(name) => {
            let custody = custody.ok_or_else(|| {
                "computed curve undo refused: enter the session operator and source/reference"
                    .to_string()
            })?;
            let payload = B64.encode(pack_pairs(&matched_depth, &matched_value));
            write_computed_revision(
                conn,
                well_id,
                name,
                &depth,
                &value,
                custody,
                serde_json::json!({
                    "operation": "undo",
                    "restored_samples": n,
                    "restored_pairs_f32_le_base64": payload,
                }),
                equations::AncestryZoneScope::WholeWell,
            )?;
        }
        _ => write_curve(conn, &store, well_id, &depth, &value)?}
    Ok(n)
}

/// Replaces one sample by writing a complete new version of the computed curve. The exact
/// prior curve is the ancestry input; a spreadsheet edit never mutates a historical run in place.
pub fn update_computed_sample(
    conn: &Connection,
    well_id: &str,
    requested_depth: f32,
    curve_name: &str,
    new_value: f32,
    custody: &equations::RunCustody,
) -> Result<(), String> {
    if !requested_depth.is_finite() {
        return Err("computed sample edit refused: depth must be finite".into());
    }
    let store = locate_curve(conn, well_id, curve_name)?;
    let CurveStore::Computed(stored_name) = &store else {
        return Err(format!("curve '{curve_name}' is not a computed curve"));
    };
    let (depth, mut values) = read_curve(conn, &store, well_id)?;
    let index = depth
        .iter()
        .position(|value| value.to_bits() == requested_depth.to_bits())
        .ok_or_else(|| {
            format!("computed curve '{curve_name}' has no sample at depth {requested_depth}")
        })?;
    values[index] = new_value;
    let recorded_value = if new_value.is_nan() {
        serde_json::json!("NaN (missing)")
    } else if new_value.is_finite() {
        serde_json::json!(new_value)
    } else {
        return Err("computed sample edit refused: value must be finite or NaN (missing)".into());
    };
    write_computed_revision(
        conn,
        well_id,
        stored_name,
        &depth,
        &values,
        custody,
        serde_json::json!({
            "operation": "sample_edit",
            "depth": requested_depth,
            "value": recorded_value,
        }),
        equations::AncestryZoneScope::WholeWell,
    )
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn open_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        conn
    }

    /// Well whose GR equals its depth (an identity ramp), 1000–1100 m at 0.5 m step.
    fn seed_ramp_well(conn: &Connection) -> String {
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "EDIT-1", Some("Synthetic"), None, None).unwrap();
        let depth: Vec<f32> = (0..201).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr = depth.clone();
        let nan = vec![f32::NAN; depth.len()];
        db::insert_standard_curves(conn, id, depth, gr, nan.clone(), nan.clone(), nan.clone(), nan.clone(), nan)
            .unwrap();
        id.to_string()
    }

    fn read_gr(conn: &Connection, well_id: &str) -> (Vec<f32>, Vec<f32>) {
        let store = locate_curve(conn, well_id, "GR").unwrap();
        read_curve(conn, &store, well_id).unwrap()
    }

    fn write_test_computed(
        conn: &Connection,
        well_id: &str,
        depth: &[f32],
        curve: &str,
        values: &[f32],
        fixture_step: &str,
    ) {
        let custody = crate::workflow::test_run_custody();
        let spec = equations::complete_curve_run_spec(
            conn,
            well_id,
            "TEST_COMPUTED",
            "TEST_COMPUTED",
            &custody,
            &[(well_id.to_string(), "fixture_input".into(), "GR".into())],
            None,
            serde_json::json!({ "fixture_step": fixture_step }),
            equations::AncestryZoneScope::WholeWell,
            &[curve.to_string()],
        )
        .unwrap();
        let (set_id, _) = equations::create_complete_log_set(conn, well_id, &spec).unwrap();
        equations::write_computed_curves_with_ancestry(
            conn,
            well_id,
            depth,
            &[(curve, values)],
            &set_id,
        )
        .unwrap();
    }

    #[test]
    fn shift_moves_curve_and_restore_undoes_it() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);

        let req = CurveEditRequest {
            well_id: w.clone(),
            curve: "GR".into(),
            op: "shift".into(),
            delta: 5.0,
            top: 0.0,
            bottom: 0.0,
            value: 0.0,
            mul: 1.0,
            add: 0.0
        ,
            custody: None,
        };
        let res = edit_curve(&conn, &req).unwrap();
        assert_eq!(res.store, "standard");
        assert!(res.affected > 0);

        let (depth, gr) = read_gr(&conn, &w);
        // Interior samples: the ramp shifted down by 5 → gr(d) = d - 5.
        let i = depth.iter().position(|&d| d == 1050.0).unwrap();
        assert!((gr[i] - 1045.0).abs() < 1e-3, "gr at 1050 = {}", gr[i]);
        // The first 5 m have no source data above the well → NaN.
        assert!(gr[0].is_nan() && gr[9].is_nan());

        // Undo: restore the returned previous samples → exact original ramp.
        let (prev_depth, prev_value) = unpack_pairs(res.point_count, &res.data).unwrap();
        let n = restore_curve_values(&conn, &w, "GR", &prev_depth, &prev_value, None).unwrap();
        assert_eq!(n, res.affected);
        let (depth, gr) = read_gr(&conn, &w);
        for (d, v) in depth.iter().zip(gr.iter()) {
            assert_eq!(d.to_bits(), v.to_bits(), "restore must be bit-exact");
        }
    }

    /// T-PLOT-19, the invalid-input half. The BACKEND guard is correct: a "set constant" whose
    /// value is not a real number is refused outright, and nothing is written. This matters
    /// because 0 is not a neutral value for this op — for `scale` an empty factor falls back to
    /// mul 1 / add 0, which is the identity and harmless, but for `set` there is no identity, and
    /// 0.0 gAPI is a reading, not a no-op. Written over an interval it looks exactly like a real
    /// measurement of very clean rock.
    ///
    /// The curve is re-read after each refusal, because "returns Err" and "changed nothing" are
    /// different claims and only the second one protects the data.
    #[test]
    fn a_set_constant_refuses_a_value_that_is_not_a_number() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);
        let before = read_gr(&conn, &w).1;

        let base = CurveEditRequest {
            well_id: w.clone(),
            curve: "GR".into(),
            op: "set".into(),
            delta: 0.0,
            top: 1010.0,
            bottom: 1020.0,
            value: 0.0,
            mul: 1.0,
            add: 0.0
        ,
            custody: None,
        };
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let req = CurveEditRequest { value: bad, ..base.clone() };
            let err = edit_curve(&conn, &req).expect_err("a non-finite constant must be refused");
            assert!(err.contains("finite"), "the refusal must say why: {err}");
            let after = read_gr(&conn, &w).1;
            for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
                assert_eq!(a.to_bits(), b.to_bits(), "sample {i} changed on a refused edit");
            }
        }

        // The control: a real value in the same request DOES write, so the assertions above are
        // about the value and not about some unrelated reason the edit could not run.
        let ok = edit_curve(&conn, &CurveEditRequest { value: 12.5, ..base }).unwrap();
        assert_eq!(ok.affected, 21, "1010..1020 inclusive at 0.5 m");
    }

    /// KNOWN GAP, pinned AS-IS and not endorsed. T-PLOT-19 step 3: `restore_curve_values` — the
    /// undo path — has no staleness check of any kind. It matches depths bit-exactly against
    /// whatever is in the curve NOW and writes the old values in.
    ///
    /// So the sequence the plan describes (edit a VSH interval, re-run the VSH module, press
    /// Ctrl+Z) splices pre-edit values over the freshly computed curve and reports success. The
    /// result is a VSH curve that is partly one vintage and partly another, with nothing on the
    /// log, in the catalog or in the provenance to say where the boundary falls.
    ///
    /// Both failure modes are silent and they are opposites, which is why both are pinned here:
    /// when the depths still match, the undo writes stale values over new ones; when they no
    /// longer match (the curve was depth-shifted since), it matches nothing and returns Ok(0) —
    /// an undo that did nothing at all, also reported as success.
    ///
    /// A fix would carry the curve's version or a content hash in the undo entry and refuse a
    /// mismatch. Making that change fails this test, which is the alarm.
    #[test]
    fn an_undo_replayed_after_the_curve_was_rewritten_splices_stale_values() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);

        // VSH on a 0.5 m grid — a COMPUTED curve, which is what the plan's scenario edits and
        // re-runs. It matters that this is not `standard_curves`: that table's depth grid comes
        // from the import and a module cannot move it, whereas a computed curve is DELETEd and
        // re-appended on every run, so its sampling can genuinely change under an undo entry.
        let vsh: Vec<f32> = (0..21).map(|i| 0.10 + i as f32 * 0.01).collect();
        let depth: Vec<f32> = (0..21).map(|i| 1000.0 + i as f32 * 0.5).collect();
        write_test_computed(&conn, &w, &depth, "VSH", &vsh, "initial");
        let read_vsh = |conn: &Connection| -> (Vec<f32>, Vec<f32>) {
            let store = locate_curve(conn, &w, "VSH").unwrap();
            read_curve(conn, &store, &w).unwrap()
        };

        // Edit: set 1002–1006 to a constant, keeping the undo payload the dialog would keep.
        let res = edit_curve(
            &conn,
            &CurveEditRequest {
                well_id: w.clone(),
                curve: "VSH".into(),
                op: "set".into(),
                delta: 0.0,
                top: 1002.0,
                bottom: 1006.0,
                value: 0.99,
                mul: 1.0,
                add: 0.0
            ,
                custody: Some(crate::workflow::test_run_custody()),
            },
        )
        .unwrap();
        assert_eq!(res.store, "computed");
        let (undo_depth, undo_value) = unpack_pairs(res.point_count, &res.data).unwrap();

        // The module is now RE-RUN: same grid, every sample recomputed to a new answer.
        write_test_computed(&conn, &w, &depth, "VSH", &vec![0.77f32; depth.len()],
            "same-grid rerun",
        );

        // Now the stale undo runs. It reports success and a full match count.
        let undo_custody = crate::workflow::test_run_custody();
        let n = restore_curve_values(&conn, &w, "VSH", &undo_depth, &undo_value,
            Some(&undo_custody),
        ).unwrap();
        assert_eq!(n, res.affected, "pinned AS-IS, not endorsed: the stale undo matches and writes");

        let (d, v) = read_vsh(&conn);
        let inside = d.iter().position(|&x| x == 1004.0).unwrap();
        let outside = d.iter().position(|&x| x == 1009.0).unwrap();
        assert!(
            (v[inside] - (0.10 + 8.0 * 0.01)).abs() < 1e-5,
            "the edited window came back to its PRE-EDIT value, over the recomputed 0.77: {}",
            v[inside]
        );
        assert!(
            (v[outside] - 0.77).abs() < 1e-5,
            "while the rest of the curve is the fresh computation: {}",
            v[outside]
        );
        // That is the damage stated plainly: one curve, two vintages, no marker between them.

        // The mirror. The module re-runs on an OFFSET grid — a re-import or a depth-shifted
        // run — which a computed curve's DELETE-and-append write really does allow. The samples
        // are half a step off, so no stored depth matches the undo entry's: the same undo now
        // matches nothing, writes nothing, and still returns Ok. (A merely FINER grid would not
        // show this: 0.25 m contains every 0.5 m depth, so the stale splice above still lands —
        // into a curve that now has twice as many samples.)
        let offset: Vec<f32> = (0..21).map(|i| 1000.25 + i as f32 * 0.5).collect();
        write_test_computed(&conn, &w, &offset, "VSH", &vec![0.77f32; offset.len()],
            "offset-grid rerun",
        );
        let n = restore_curve_values(&conn, &w, "VSH", &undo_depth, &undo_value,
            Some(&undo_custody),
        )
            .expect("pinned AS-IS: a depth mismatch is not reported as an error either");
        assert_eq!(n, 0, "nothing matched, and the only sign of it is a zero count");
    }

    #[test]
    fn blank_then_interpolate_bridges_the_gap() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);
        let base = CurveEditRequest {
            well_id: w.clone(),
            curve: "GR".into(),
            op: "blank".into(),
            delta: 0.0,
            top: 1010.0,
            bottom: 1020.0,
            value: 0.0,
            mul: 1.0,
            add: 0.0
        ,
            custody: None,
        };
        let res = edit_curve(&conn, &base).unwrap();
        assert_eq!(res.affected, 21); // inclusive 1010..1020 at 0.5 m step
        let (depth, gr) = read_gr(&conn, &w);
        let i = depth.iter().position(|&d| d == 1015.0).unwrap();
        assert!(gr[i].is_nan());

        // Interpolate across the blanked hole: the ramp is linear, so the bridge
        // reproduces it exactly (anchors at 1009.5 and 1020.5).
        let interp = CurveEditRequest { op: "interpolate".into(), top: 1009.5, bottom: 1020.5, ..base };
        let res = edit_curve(&conn, &interp).unwrap();
        assert_eq!(res.affected, 21);
        let (depth, gr) = read_gr(&conn, &w);
        for (d, v) in depth.iter().zip(gr.iter()) {
            assert!((d - v).abs() < 1e-3, "bridge at {d} gave {v}");
        }
    }

    #[test]
    fn set_and_scale_route_to_computed_and_generic_stores() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);

        // A computed edit is a new run, never a rewrite under the producer's old set identity.
        write_test_computed(
            &conn,
            &w,
            &[1000.0, 1001.0],
            "VSH",
            &[0.30, 0.40],
            "route fixture",
        );
        let prior_set_id: String = conn.query_row("SELECT CAST(set_id AS VARCHAR) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'VSH' LIMIT 1",
                params![w],
                |row| row.get(
            0))
        .unwrap();
        let req = CurveEditRequest {
            well_id: w.clone(),
            curve: "vsh".into(),
            op: "set".into(),
            delta: 0.0,
            top: 1000.5,
            bottom: 1002.0,
            value: 0.99,
            mul: 1.0,
            add: 0.0
        ,
            custody: Some(crate::workflow::test_run_custody()),
        };
        let res = edit_curve(&conn, &req).unwrap();
        assert_eq!((res.store.as_str(), res.affected), ("computed", 1));
        let (v, set_id): (f32, Option<String>) = conn
            .query_row(
                "SELECT value, set_id FROM computed_curves WHERE well_id = ?1 AND depth = 1001.0",
                params![w],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((v - 0.99).abs() < 1e-6);
        let edited_set_id = set_id.expect("the edit must have a complete ancestry identity");
        assert_ne!(
            edited_set_id, prior_set_id,
            "an edit must create a new version rather than falsify the old run"
        );
        assert_eq!(
            equations::curve_ancestry(&conn, &w, "VSH").unwrap().module, "CURVE_EDIT"
        );

        // Generic-store curve, addressed by FAMILY (PEF ← mnemonic PEFZ).
        let curve_id = db::upsert_curve_meta(&conn, &w, "RAW", "PEFZ", Some("b/e"), Some("PEF"), None, None).unwrap();
        db::insert_curve_samples(&conn, &curve_id, &[1000.0, 1001.0], &[3.0, 4.0]).unwrap();
        let req = CurveEditRequest { curve: "PEF".into(), op: "scale".into(), top: 999.0, bottom: 1002.0, mul: 2.0, add: 1.0, ..req };
        let res = edit_curve(&conn, &req).unwrap();
        assert_eq!((res.store.as_str(), res.affected), ("raw", 2));
        let v: f32 = conn
            .query_row("SELECT value FROM curve_samples WHERE curve_id = ?1 AND depth = 1001.0", params![curve_id], |r| r.get(0))
            .unwrap();
        assert!((v - 9.0).abs() < 1e-6); // 2*4 + 1
    }

    #[test]
    fn missing_curve_and_bad_op_error_cleanly() {
        let conn = open_db();
        let w = seed_ramp_well(&conn);
        let req = CurveEditRequest {
            well_id: w.clone(),
            curve: "NOSUCH".into(),
            op: "shift".into(),
            delta: 1.0,
            top: 0.0,
            bottom: 0.0,
            value: 0.0,
            mul: 1.0,
            add: 0.0
        ,
            custody: None,
        };
        assert!(edit_curve(&conn, &req).is_err());
        let req = CurveEditRequest { curve: "GR".into(), op: "explode".into(), ..req };
        assert!(edit_curve(&conn, &req).unwrap_err().contains("unknown edit op"));
    }
}
