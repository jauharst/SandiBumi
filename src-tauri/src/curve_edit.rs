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

use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

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
    pub add: f32,
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
            "SELECT curve_id FROM curve_meta
             WHERE well_id = ?1 AND set_name = 'RAW'
               AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
             ORDER BY (upper(mnemonic) = ?2) DESC, run_no NULLS FIRST
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
        CurveStore::Computed(name) => {
            // Keep each row's set_id (P1-c versioning tag) across the rewrite.
            let mut stmt = conn
                .prepare("SELECT depth, set_id FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2 ORDER BY depth")
                .map_err(|e| e.to_string())?;
            let rows: Vec<(f32, Option<String>)> = stmt
                .query_map(params![well_id, name], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            if rows.len() != depth.len() {
                return Err("curve changed while editing — retry".into());
            }
            conn.execute(
                "DELETE FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2",
                params![well_id, name],
            )
            .map_err(|e| e.to_string())?;
            let mut appender = conn.appender("computed_curves").map_err(|e| e.to_string())?;
            for (i, (d, set_id)) in rows.into_iter().enumerate() {
                appender
                    .append_row(params![well_id, d, name, new_values[i], set_id])
                    .map_err(|e| e.to_string())?;
            }
            appender.flush().map_err(|e| e.to_string())?;
        }
        CurveStore::Generic(curve_id) => {
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

    write_curve(conn, &store, &req.well_id, &depth, &new)?;
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
    values: &[f32],
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
    for (i, d) in depth.iter().enumerate() {
        if let Some(&v) = restore.get(&d.to_bits()) {
            value[i] = v;
            n += 1;
        }
    }
    if n == 0 {
        return Ok(0);
    }
    write_curve(conn, &store, well_id, &depth, &value)?;
    Ok(n)
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
            add: 0.0,
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
        let n = restore_curve_values(&conn, &w, "GR", &prev_depth, &prev_value).unwrap();
        assert_eq!(n, res.affected);
        let (depth, gr) = read_gr(&conn, &w);
        for (d, v) in depth.iter().zip(gr.iter()) {
            assert_eq!(d.to_bits(), v.to_bits(), "restore must be bit-exact");
        }
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
            add: 0.0,
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

        // Computed curve with a set_id tag that must survive the rewrite.
        conn.execute_batch(&format!(
            "INSERT INTO computed_curves VALUES ('{w}', 1000.0, 'VSH', 0.30, 'aaaaaaaa-0000-0000-0000-000000000000');
             INSERT INTO computed_curves VALUES ('{w}', 1001.0, 'VSH', 0.40, 'aaaaaaaa-0000-0000-0000-000000000000');"
        ))
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
            add: 0.0,
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
        assert_eq!(set_id.as_deref(), Some("aaaaaaaa-0000-0000-0000-000000000000"));

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
            add: 0.0,
        };
        assert!(edit_curve(&conn, &req).is_err());
        let req = CurveEditRequest { curve: "GR".into(), op: "explode".into(), ..req };
        assert!(edit_curve(&conn, &req).unwrap_err().contains("unknown edit op"));
    }
}
