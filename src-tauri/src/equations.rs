use duckdb::{params, params_from_iter, Connection};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A user-authored petrophysical equation (Rhai script), analogous to a Geolog loglan
/// module or an IP formula: a name, a script body, its declared input curve mnemonics,
/// and the curve it produces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquationDef {
    pub equation_id: String,
    pub name: String,
    pub description: Option<String>,
    pub script: String,
    pub input_curves: Vec<String>,
    pub output_curve: String,
    pub output_units: Option<String>,
    /// "rhai" (per-sample, legacy) or "python" (vectorized numpy).
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "rhai".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct EquationRunResult {
    pub well_id: String,
    pub rows_written: usize,
    pub error: Option<String>,
}

impl EquationRunResult {
    fn success(well_id: String, rows_written: usize) -> Self {
        Self { well_id, rows_written, error: None }
    }
    fn failed(well_id: String, error: String) -> Self {
        Self { well_id, rows_written: 0, error: Some(error) }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveCatalogEntry {
    pub name: String,
    pub units: Option<String>,
    pub source: String, // "Standard" | "Computed"
}

/// A curve's decimated (depth, value) series, packed as raw `bytemuck` bytes rather than a
/// JSON number array — per this project's IPC rule, bulk f32 arrays never cross the Tauri
/// bridge as JSON. `data` is `depth[point_count]` immediately followed by `value[point_count]`.
#[derive(Debug, Clone, Serialize)]
pub struct TrackCurveSeries {
    pub curve_name: String,
    pub point_count: usize,
    pub data: Vec<u8>,
}

/// Fetches and decimates the requested curves for one well, ready for the multi-track
/// viewer: each curve is min/max-decimated independently to `target_pixel_height`.
pub fn fetch_track_data(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
    target_pixel_height: usize,
) -> duckdb::Result<Vec<TrackCurveSeries>> {
    let (depth, columns) = fetch_curve_frame(conn, well_id, curve_names)?;
    Ok(curve_names
        .iter()
        .map(|name| {
            let upper = name.trim().to_uppercase();
            let values = columns.get(&upper).cloned().unwrap_or_default();
            let (dec_depth, dec_value) = crate::decimate::min_max_decimate(&depth, &values, target_pixel_height);
            let point_count = dec_depth.len();
            let mut packed = Vec::with_capacity(point_count * 2);
            packed.extend_from_slice(&dec_depth);
            packed.extend_from_slice(&dec_value);
            TrackCurveSeries { curve_name: upper, point_count, data: bytemuck::cast_slice(&packed).to_vec() }
        })
        .collect())
}

/// Fetches full-resolution (undecimated) curve values for crossplots/histograms/Pickett
/// plots, optionally restricted to a depth interval. Same binary packing as
/// `fetch_track_data`: `depth[point_count]` followed by `value[point_count]`.
pub fn fetch_curve_data(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> duckdb::Result<Vec<TrackCurveSeries>> {
    let (depth, columns) = fetch_curve_frame(conn, well_id, curve_names)?;
    let lo = depth_min.unwrap_or(f32::NEG_INFINITY);
    let hi = depth_max.unwrap_or(f32::INFINITY);
    let keep: Vec<usize> = depth
        .iter()
        .enumerate()
        .filter(|(_, d)| **d >= lo && **d < hi)
        .map(|(i, _)| i)
        .collect();

    Ok(curve_names
        .iter()
        .map(|name| {
            let upper = name.trim().to_uppercase();
            let values = columns.get(&upper).cloned().unwrap_or_default();
            let point_count = keep.len();
            let mut packed = Vec::with_capacity(point_count * 2);
            packed.extend(keep.iter().map(|&i| depth[i]));
            packed.extend(keep.iter().map(|&i| *values.get(i).unwrap_or(&f32::NAN)));
            TrackCurveSeries { curve_name: upper, point_count, data: bytemuck::cast_slice(&packed).to_vec() }
        })
        .collect())
}

/// Fetches core plug data for one well as four independent (depth, value) series —
/// CPOR, CPERM, CGD, CSW — each holding only its own non-NaN samples. Unlike
/// `fetch_curve_frame`, this does NOT align onto the well's standard depth grid: core
/// plug depths are sparse/irregular by nature, so overlay panels plot them at their own
/// depths rather than resampling.
pub fn fetch_core_series(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<TrackCurveSeries>> {
    let mut stmt =
        conn.prepare("SELECT depth, cpor, cperm, cgd, csw FROM core_data WHERE well_id = ?1 ORDER BY depth")?;
    let rows = stmt.query_map(params![well_id], |row| {
        Ok((
            row.get::<_, f32>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f32>(2)?,
            row.get::<_, f32>(3)?,
            row.get::<_, f32>(4)?,
        ))
    })?;

    let (mut depth, mut cpor, mut cperm, mut cgd, mut csw): (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) =
        (vec![], vec![], vec![], vec![], vec![]);
    for r in rows {
        let (d, po, pe, gd, sw) = r?;
        depth.push(d);
        cpor.push(po);
        cperm.push(pe);
        cgd.push(gd);
        csw.push(sw);
    }

    let pack = |name: &str, values: &[f32]| -> TrackCurveSeries {
        let keep: Vec<usize> = values.iter().enumerate().filter(|(_, v)| !v.is_nan()).map(|(i, _)| i).collect();
        let point_count = keep.len();
        let mut packed = Vec::with_capacity(point_count * 2);
        packed.extend(keep.iter().map(|&i| depth[i]));
        packed.extend(keep.iter().map(|&i| values[i]));
        TrackCurveSeries { curve_name: name.to_string(), point_count, data: bytemuck::cast_slice(&packed).to_vec() }
    };

    Ok(vec![pack("CPOR", &cpor), pack("CPERM", &cperm), pack("CGD", &cgd), pack("CSW", &csw)])
}

/// Upserts an equation by (unique) name and returns its authoritative equation_id.
pub fn save_equation(conn: &Connection, def: &EquationDef) -> duckdb::Result<String> {
    let id = if def.equation_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        def.equation_id.clone()
    };
    let input_curves = def.input_curves.join(",");

    conn.execute(
        "INSERT INTO equations (equation_id, name, description, script, input_curves, output_curve, output_units, language, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, now())
         ON CONFLICT (name) DO UPDATE SET
            description = excluded.description,
            script = excluded.script,
            input_curves = excluded.input_curves,
            output_curve = excluded.output_curve,
            output_units = excluded.output_units,
            language = excluded.language,
            updated_at = now()",
        params![id, def.name, def.description, def.script, input_curves, def.output_curve, def.output_units, def.language],
    )?;

    conn.query_row(
        "SELECT equation_id FROM equations WHERE name = ?1",
        params![def.name],
        |row| row.get(0),
    )
}

pub fn list_equations(conn: &Connection) -> duckdb::Result<Vec<EquationDef>> {
    let mut stmt = conn.prepare(
        "SELECT equation_id, name, description, script, input_curves, output_curve, output_units,
                COALESCE(language, 'rhai')
         FROM equations ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        let input_curves_raw: String = row.get(4)?;
        Ok(EquationDef {
            equation_id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            script: row.get(3)?,
            input_curves: input_curves_raw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            output_curve: row.get(5)?,
            output_units: row.get(6)?,
            language: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Curve catalog auto-derived from what's actually in the database: the fixed standard
/// curves plus every distinct computed curve, picking up units from whichever equation
/// produced it. No separate reference file to keep in sync (unlike IP's static CPARMDEF).
pub fn list_curve_catalog(conn: &Connection) -> duckdb::Result<Vec<CurveCatalogEntry>> {
    let mut entries = vec![
        CurveCatalogEntry { name: "GR".into(), units: Some("GAPI".into()), source: "Standard".into() },
        CurveCatalogEntry { name: "RES_DEEP".into(), units: Some("OHMM".into()), source: "Standard".into() },
        CurveCatalogEntry { name: "NPHI".into(), units: Some("V/V".into()), source: "Standard".into() },
        CurveCatalogEntry { name: "RHOB".into(), units: Some("G/C3".into()), source: "Standard".into() },
        CurveCatalogEntry { name: "DT".into(), units: Some("US/F".into()), source: "Standard".into() },
        CurveCatalogEntry { name: "SP".into(), units: Some("MV".into()), source: "Standard".into() },
    ];

    let mut stmt = conn.prepare(
        "SELECT DISTINCT cc.curve_name, e.output_units
         FROM computed_curves cc
         LEFT JOIN equations e ON e.output_curve = cc.curve_name
         ORDER BY cc.curve_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CurveCatalogEntry { name: row.get(0)?, units: row.get(1)?, source: "Computed".into() })
    })?;
    for r in rows {
        entries.push(r?);
    }
    Ok(entries)
}

type CurveFrame = (Vec<f32>, HashMap<String, Vec<f32>>);

/// Reads the requested curve mnemonics for one well, aligned onto that well's standard
/// depth grid. Non-standard names are looked up in `computed_curves` (so equations can
/// chain off previously computed curves).
pub(crate) fn fetch_curve_frame(conn: &Connection, well_id: &str, curve_names: &[String]) -> duckdb::Result<CurveFrame> {
    let mut stmt = conn.prepare(
        "SELECT depth, gr, res_deep, nphi, rhob, dt, sp FROM standard_curves WHERE well_id = ?1 ORDER BY depth",
    )?;
    let mut depth = Vec::new();
    let mut gr = Vec::new();
    let mut res_deep = Vec::new();
    let mut nphi = Vec::new();
    let mut rhob = Vec::new();
    let mut dt = Vec::new();
    let mut sp = Vec::new();

    let rows = stmt.query_map(params![well_id], |row| {
        Ok((
            row.get::<_, f32>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, f32>(2)?,
            row.get::<_, f32>(3)?,
            row.get::<_, f32>(4)?,
            row.get::<_, Option<f32>>(5)?,
            row.get::<_, Option<f32>>(6)?,
        ))
    })?;
    for r in rows {
        let (d, g, rd, np, rb, sdt, ssp) = r?;
        depth.push(d);
        gr.push(g);
        res_deep.push(rd);
        nphi.push(np);
        rhob.push(rb);
        dt.push(sdt.unwrap_or(f32::NAN));
        sp.push(ssp.unwrap_or(f32::NAN));
    }

    let mut columns: HashMap<String, Vec<f32>> = HashMap::new();
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        let std_col = match upper.as_str() {
            "DEPTH" => Some(&depth),
            "GR" => Some(&gr),
            "RES_DEEP" => Some(&res_deep),
            "NPHI" => Some(&nphi),
            "RHOB" => Some(&rhob),
            "DT" => Some(&dt),
            "SP" => Some(&sp),
            _ => None,
        };
        let values = match std_col {
            Some(col) if upper == "DEPTH" || col.iter().any(|v| !v.is_nan()) => col.clone(),
            // An all-NaN standard column means the delivery's mnemonics matched no
            // standard alias at import (e.g. an APS well whose only neutron is APLC):
            // fall through to computed/generic resolution so the family dictionary
            // can still find the curve instead of silently feeding NaN to modules.
            _ => fetch_named_curve_aligned(conn, well_id, &upper, &depth)?,
        };
        columns.insert(upper, values);
    }
    Ok((depth, columns))
}

/// Resolves a non-standard curve name onto the depth grid, trying (1) `computed_curves`
/// (so equations/modules can chain off earlier results), then (2) the generic curve store
/// (set RAW) so imported curves that were never one of the fixed six — PEF, CALI, DRHO,
/// RXO, extra runs — are usable as module/equation inputs. The generic lookup matches on
/// the curve's own mnemonic first, then its resolved family (so a module asking for "CALI"
/// finds an "HCAL" curve whose family is CALI), preferring the base run.
fn fetch_named_curve_aligned(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth_grid: &[f32],
) -> duckdb::Result<Vec<f32>> {
    let computed = fetch_computed_curve_aligned(conn, well_id, curve_name, depth_grid)?;
    if computed.iter().any(|v| !v.is_nan()) {
        return Ok(computed);
    }
    fetch_generic_curve_aligned(conn, well_id, curve_name, depth_grid)
}

fn fetch_computed_curve_aligned(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth_grid: &[f32],
) -> duckdb::Result<Vec<f32>> {
    // Case-insensitive name match: equation outputs may be saved lowercase while
    // module/plot requests arrive uppercased — an exact compare read back all-NaN.
    let mut stmt = conn
        .prepare("SELECT depth, value FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = upper(?2)")?;
    let rows = stmt.query_map(params![well_id, curve_name], |row| {
        Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?))
    })?;

    let mut by_depth: HashMap<u32, f32> = HashMap::new();
    for r in rows {
        let (d, v) = r?;
        by_depth.insert(d.to_bits(), v);
    }
    Ok(depth_grid.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect())
}

/// Looks up a curve in the generic store (`curve_meta`/`curve_samples`, set RAW) by
/// mnemonic-or-family and aligns its samples onto the depth grid. Exact mnemonic matches
/// win over family matches; among equals the base run (lowest `run_no`, NULL first) wins.
fn fetch_generic_curve_aligned(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth_grid: &[f32],
) -> duckdb::Result<Vec<f32>> {
    let upper = curve_name.trim().to_uppercase();
    let curve_id: Option<String> = conn
        .query_row(
            "SELECT curve_id FROM curve_meta
             WHERE well_id = ?1 AND set_name = 'RAW'
               AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
             ORDER BY (upper(mnemonic) = ?2) DESC, run_no NULLS FIRST
             LIMIT 1",
            params![well_id, upper],
            |row| row.get(0),
        )
        .ok();
    let Some(curve_id) = curve_id else {
        return Ok(vec![f32::NAN; depth_grid.len()]);
    };

    let mut stmt = conn.prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1")?;
    let rows = stmt.query_map(params![curve_id], |row| {
        Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?))
    })?;
    let mut by_depth: HashMap<u32, f32> = HashMap::new();
    for r in rows {
        let (d, v) = r?;
        by_depth.insert(d.to_bits(), v);
    }
    Ok(depth_grid.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect())
}

/// Computed-provenance-only resolution for unit-contract inputs (ArgSpec.computed_only,
/// e.g. gascorr's FTEMP/FPRESS): the named input set's archived values first (matching
/// [`fetch_curve_frame_from_set`] semantics, including the own-set precedence), then
/// current `computed_curves` — never the RAW import store, so a Geolog LAS export's
/// degF FTEMP cannot silently masquerade as precalc's degC output.
pub(crate) fn fetch_computed_only_aligned(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth_grid: &[f32],
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> duckdb::Result<Vec<f32>> {
    let upper = curve_name.trim().to_uppercase();
    // An earlier step of this very run wrote the curve: its fresh current values win.
    let own_wrote = match own_set_id {
        Some(own) => conn
            .query_row(
                "SELECT 1 FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2 LIMIT 1",
                params![own, upper],
                |_| Ok(()),
            )
            .is_ok(),
        None => false,
    };
    if !own_wrote {
        if let Some(set_name) = input_set.map(str::trim).filter(|s| !s.is_empty()) {
            let set_id: Option<String> = conn
                .query_row(
                    "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, set_name],
                    |r| r.get(0),
                )
                .ok();
            if let Some(set_id) = set_id {
                let mut stmt = conn.prepare(
                    "SELECT depth, value FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2",
                )?;
                let rows = stmt.query_map(params![set_id, upper], |row| {
                    Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?))
                })?;
                let mut by_depth: HashMap<u32, f32> = HashMap::new();
                for r in rows {
                    let (d, v) = r?;
                    by_depth.insert(d.to_bits(), v);
                }
                if !by_depth.is_empty() {
                    return Ok(depth_grid
                        .iter()
                        .map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN))
                        .collect());
                }
            }
        }
    }
    fetch_computed_curve_aligned(conn, well_id, &upper, depth_grid)
}

/// Replaces any prior values for (well_id, curve_name) with the freshly computed ones.
pub(crate) fn write_computed_curve(conn: &Connection, well_id: &str, depth: &[f32], curve_name: &str, values: &[f32]) -> duckdb::Result<()> {
    write_computed_curves_batch(conn, well_id, depth, &[(curve_name, values)])
}

// ---------------------------------------------------------------------------
// P1-c log-set versioning: run events + append-only history (never overwrite)
// ---------------------------------------------------------------------------

/// Provenance of one run event into a named log set.
#[derive(Debug, Clone)]
pub struct LogSetSpec {
    pub set_name: String,
    pub module: String,
    pub params_json: String,
    pub inputs_json: String,
}

/// One version of a log set as listed in the catalog / Sets manager.
#[derive(Debug, Clone, Serialize)]
pub struct LogSetEntry {
    pub set_id: String,
    pub set_name: String,
    pub version: i64,
    pub module: String,
    pub params_json: Option<String>,
    pub inputs_json: Option<String>,
    pub created_at: String,
    pub curve_names: Vec<String>,
    pub is_current: bool,
}

/// Registers a new run event: version = 1 + the well's highest version of `set_name`
/// (so a re-run NEVER replaces — it becomes version N+1). Returns (set_id, version).
pub(crate) fn create_log_set(conn: &Connection, well_id: &str, spec: &LogSetSpec) -> duckdb::Result<(String, i64)> {
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
        params![well_id, spec.set_name],
        |r| r.get(0),
    )?;
    let set_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO log_sets (set_id, well_id, set_name, version, module, params_json, inputs_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![set_id, well_id, spec.set_name, version, spec.module, spec.params_json, spec.inputs_json],
    )?;
    Ok((set_id, version))
}

/// Versioned batch write: refreshes the CURRENT store (same delete-then-append discipline
/// as `write_computed_curves_batch`, rows tagged with `set_id`) and appends the identical
/// rows to the append-only archive. Prior versions' archive rows are untouched — that is
/// the "never overwrite" guarantee; any version can be restored via `restore_log_set`.
pub(crate) fn write_computed_curves_versioned(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &str,
) -> duckdb::Result<()> {
    if curves.is_empty() {
        return Ok(());
    }
    // Atomic: DELETE current + append current + append archive must land as one unit, so a
    // crash can't strand the DELETE with the current-store append lost.
    crate::db::with_txn(conn, |conn| {
        let placeholders = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM computed_curves WHERE well_id = ? AND curve_name IN ({placeholders})");
        let mut del_params: Vec<&str> = Vec::with_capacity(curves.len() + 1);
        del_params.push(well_id);
        for (name, _) in curves {
            del_params.push(name);
        }
        conn.execute(&sql, params_from_iter(del_params))?;

        let mut current = conn.appender("computed_curves")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                current.append_row(params![well_id, d, name, v, set_id])?;
            }
        }
        current.flush()?;

        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                archive.append_row(params![set_id, well_id, d, name, v])?;
            }
        }
        archive.flush()?;
        Ok(())
    })
}

/// Input-set selection: like [`fetch_curve_frame`], but any requested curve that the named
/// log set wrote (latest version per well, name matched case-insensitively) is read from
/// that set's ARCHIVED values instead of the current store — so a module can consume
/// "VSH from FINAL" even after later runs replaced the current VSH. Curves the set never
/// wrote fall back to normal resolution (raw stores, current computed), and an unknown /
/// empty set name degrades to plain [`fetch_curve_frame`].
///
/// `own_set_id` is the running job's own output set event (workflow chains): curves that
/// an EARLIER step of the same run already wrote keep their fresh current values — the
/// input set must never shadow this run's own intermediate outputs.
pub(crate) fn fetch_curve_frame_from_set(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> duckdb::Result<CurveFrame> {
    let (depth, mut columns) = fetch_curve_frame(conn, well_id, curve_names)?;
    let Some(set_name) = input_set.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok((depth, columns));
    };
    let own_curves: std::collections::HashSet<String> = match own_set_id {
        Some(own) => {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT upper(curve_name) FROM computed_curves_archive WHERE set_id = ?1",
            )?;
            let rows = stmt.query_map(params![own], |row| row.get::<_, String>(0))?;
            rows.collect::<duckdb::Result<_>>()?
        }
        None => Default::default(),
    };
    let set_id: Option<String> = conn
        .query_row(
            "SELECT set_id FROM log_sets WHERE well_id = ?1 AND upper(set_name) = upper(?2)
             ORDER BY version DESC LIMIT 1",
            params![well_id, set_name],
            |r| r.get(0),
        )
        .ok();
    let Some(set_id) = set_id else {
        return Ok((depth, columns));
    };

    let mut stmt = conn.prepare(
        "SELECT depth, value FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2",
    )?;
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        if own_curves.contains(&upper) {
            continue; // written by an earlier step of this very run — keep the fresh values
        }
        let rows = stmt.query_map(params![set_id, upper], |row| {
            Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?))
        })?;
        let mut by_depth: HashMap<u32, f32> = HashMap::new();
        for r in rows {
            let (d, v) = r?;
            by_depth.insert(d.to_bits(), v);
        }
        if by_depth.is_empty() {
            continue; // set never wrote this curve — keep the fallback resolution
        }
        columns.insert(
            upper,
            depth.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect(),
        );
    }
    Ok((depth, columns))
}

/// Every run event for a well, newest first, with the curves it wrote and whether any of
/// its rows still provide the current values.
pub(crate) fn list_log_sets(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<LogSetEntry>> {
    let mut stmt = conn.prepare(
        "SELECT s.set_id, s.set_name, s.version, s.module, s.params_json, s.inputs_json,
                strftime(s.created_at, '%Y-%m-%d %H:%M'),
                EXISTS (SELECT 1 FROM computed_curves cc WHERE cc.set_id = s.set_id)
         FROM log_sets s
         WHERE s.well_id = ?1
         ORDER BY s.set_name, s.version DESC",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(LogSetEntry {
            set_id: r.get(0)?,
            set_name: r.get(1)?,
            version: r.get(2)?,
            module: r.get(3)?,
            params_json: r.get(4)?,
            inputs_json: r.get(5)?,
            created_at: r.get(6)?,
            curve_names: Vec::new(),
            is_current: r.get(7)?,
        })
    })?;
    let mut entries = Vec::new();
    for r in rows {
        entries.push(r?);
    }
    // Curve names per set from the archive (one query, folded in Rust).
    let mut stmt = conn.prepare(
        "SELECT DISTINCT set_id, curve_name FROM computed_curves_archive WHERE well_id = ?1 ORDER BY curve_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut by_set: HashMap<String, Vec<String>> = HashMap::new();
    for r in rows {
        let (sid, name) = r?;
        by_set.entry(sid).or_default().push(name);
    }
    for e in &mut entries {
        if let Some(names) = by_set.remove(&e.set_id) {
            e.curve_names = names;
        }
    }
    Ok(entries)
}

/// Copies a version's archived rows back into the current store (delete-then-append on
/// exactly the curve names that version wrote). Returns the number of restored rows.
pub(crate) fn restore_log_set(conn: &Connection, set_id: &str) -> duckdb::Result<usize> {
    // Atomic: the DELETE of current rows and the re-INSERT from the archive must not be split
    // by a crash (which would drop the current rows and leave them un-restored).
    crate::db::with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM computed_curves
             WHERE well_id = (SELECT well_id FROM log_sets WHERE set_id = ?1)
               AND curve_name IN (SELECT DISTINCT curve_name FROM computed_curves_archive WHERE set_id = ?1)",
            params![set_id],
        )?;
        let restored = conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             SELECT well_id, depth, curve_name, value, set_id FROM computed_curves_archive WHERE set_id = ?1",
            params![set_id],
        )?;
        Ok(restored)
    })
}

/// Deletes one version's archive rows + its log_sets row. Current values are kept (their
/// provenance tag is cleared) so deleting history can never change any plot or result.
pub(crate) fn delete_log_set(conn: &Connection, set_id: &str) -> duckdb::Result<()> {
    // Atomic: clearing provenance + dropping archive rows + dropping the log_sets row must not
    // be split by a crash (which could orphan archive rows or a dangling set_id reference).
    crate::db::with_txn(conn, |conn| {
        conn.execute("UPDATE computed_curves SET set_id = NULL WHERE set_id = ?1", params![set_id])?;
        conn.execute("DELETE FROM computed_curves_archive WHERE set_id = ?1", params![set_id])?;
        conn.execute("DELETE FROM log_sets WHERE set_id = ?1", params![set_id])?;
        Ok(())
    })
}

/// Catalog of a well's CURRENT computed curves with per-curve provenance (which set
/// version wrote it, by what module, when) and basic statistics for search/sort.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedCatalogEntry {
    pub curve_name: String,
    pub set_name: Option<String>,
    pub version: Option<i64>,
    pub module: Option<String>,
    pub created_at: Option<String>,
    pub n_samples: i64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
}

pub(crate) fn list_computed_catalog(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<ComputedCatalogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT cc.curve_name, s.set_name, s.version, s.module,
                strftime(s.created_at, '%Y-%m-%d %H:%M'),
                COUNT(*) FILTER (WHERE NOT isnan(cc.value)),
                MIN(cc.value) FILTER (WHERE NOT isnan(cc.value)),
                MAX(cc.value) FILTER (WHERE NOT isnan(cc.value)),
                AVG(cc.value) FILTER (WHERE NOT isnan(cc.value))
         FROM computed_curves cc
         LEFT JOIN log_sets s ON s.set_id = cc.set_id
         WHERE cc.well_id = ?1
         GROUP BY cc.curve_name, s.set_name, s.version, s.module, s.created_at
         ORDER BY cc.curve_name",
    )?;
    let rows = stmt.query_map(params![well_id], |r| {
        Ok(ComputedCatalogEntry {
            curve_name: r.get(0)?,
            set_name: r.get(1)?,
            version: r.get(2)?,
            module: r.get(3)?,
            created_at: r.get(4)?,
            n_samples: r.get(5)?,
            min: r.get(6)?,
            max: r.get(7)?,
            mean: r.get(8)?,
        })
    })?;
    let mut entries = Vec::new();
    for r in rows {
        entries.push(r?);
    }
    Ok(entries)
}

/// Replaces any prior values for a well's given curves in a single DELETE + one Appender.
///
/// Every curve name passed is fully overwritten (its old rows for this well are deleted),
/// then all the new rows are appended together. Batching a well's whole module output this
/// way collapses N per-curve DELETE/append/flush cycles into one — and since
/// `computed_curves` carries no uniqueness index (see `db::create_schema`), the delete-then-
/// append IS what keeps (well_id, depth, curve_name) unique. Depth/values are zipped, so a
/// short `values` slice simply writes fewer rows (matching the old per-curve behaviour).
pub(crate) fn write_computed_curves_batch(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
) -> duckdb::Result<()> {
    if curves.is_empty() {
        return Ok(());
    }
    // Atomic delete-then-append: an unclean kill mid-write must not leave the DELETE committed
    // with the never-flushed append lost (this unversioned path has no archive to recover from).
    crate::db::with_txn(conn, |conn| {
        // One DELETE covering exactly the curve names about to be rewritten.
        let placeholders = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM computed_curves WHERE well_id = ? AND curve_name IN ({placeholders})");
        let mut del_params: Vec<&str> = Vec::with_capacity(curves.len() + 1);
        del_params.push(well_id);
        for (name, _) in curves {
            del_params.push(name);
        }
        conn.execute(&sql, params_from_iter(del_params))?;

        let mut appender = conn.appender("computed_curves")?;
        for (name, values) in curves {
            for (d, v) in depth.iter().zip(values.iter()) {
                // 5th column: no set_id — this is the legacy/unversioned write path.
                appender.append_row(params![well_id, d, name, v, None::<String>])?;
            }
        }
        appender.flush()?;
        Ok(())
    })
}

/// Runs `equation` across every well in `well_ids` concurrently via `rayon`. The Rhai
/// script is compiled once and shared (via `Arc`, using rhai's `sync` feature) across
/// worker threads; each depth sample gets a fresh `Scope` with input curve values bound
/// as lowercase variables. Any NaN input short-circuits straight to a NaN output, so the
/// script itself never has to special-case missing data.
pub fn run_equation(db: &Mutex<Connection>, equation: &EquationDef, well_ids: &[String]) -> Vec<EquationRunResult> {
    let engine = Engine::new();
    let ast: AST = match engine.compile(&equation.script) {
        Ok(ast) => ast,
        Err(e) => {
            return well_ids
                .iter()
                .map(|w| EquationRunResult::failed(w.clone(), format!("script error: {e}")))
                .collect();
        }
    };
    let engine = Arc::new(engine);
    let ast = Arc::new(ast);

    well_ids
        .par_iter()
        .map(|well_id| {
            let (depth, columns) = {
                let conn = db.lock().unwrap();
                match fetch_curve_frame(&conn, well_id, &equation.input_curves) {
                    Ok(v) => v,
                    Err(e) => return EquationRunResult::failed(well_id.clone(), e.to_string()),
                }
            };
            if depth.is_empty() {
                return EquationRunResult::failed(well_id.clone(), "no curve data for well".into());
            }

            let n = depth.len();
            let mut output = Vec::with_capacity(n);
            for i in 0..n {
                let mut scope = Scope::new();
                let mut has_nan = false;
                for (name, values) in &columns {
                    let v = values[i];
                    if v.is_nan() {
                        has_nan = true;
                    }
                    scope.push(name.to_lowercase(), v as f64);
                }

                if has_nan {
                    output.push(f32::NAN);
                    continue;
                }
                match engine.eval_ast_with_scope::<f64>(&mut scope, &ast) {
                    Ok(v) => output.push(v as f32),
                    Err(_) => output.push(f32::NAN),
                }
            }

            let conn = db.lock().unwrap();
            if let Err(e) = write_equation_output(&conn, well_id, &depth, equation, &output) {
                return EquationRunResult::failed(well_id.clone(), e.to_string());
            }
            EquationRunResult::success(well_id.clone(), n)
        })
        .collect()
}

/// Versioned write of one equation run's output: registers a run event in set "EQUATION"
/// (module "equation:<name>", inputs recorded) and writes current + archive.
pub(crate) fn write_equation_output(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    equation: &EquationDef,
    values: &[f32],
) -> duckdb::Result<()> {
    let spec = LogSetSpec {
        set_name: "EQUATION".into(),
        module: format!("equation:{}", equation.name),
        params_json: String::new(),
        inputs_json: serde_json::to_string(&equation.input_curves).unwrap_or_default(),
    };
    let (set_id, _) = create_log_set(conn, well_id, &spec)?;
    write_computed_curves_versioned(conn, well_id, depth, &[(equation.output_curve.as_str(), values)], &set_id)
}
