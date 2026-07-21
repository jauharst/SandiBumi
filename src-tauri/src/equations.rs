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

/// Packs a set of curve series into ONE length-prefixed binary buffer for raw-IPC transport
/// (a `tauri::ipc::Response` → JS `ArrayBuffer`), instead of letting serde encode each
/// `data: Vec<u8>` as a JSON number array (~4× the bytes + a main-thread `JSON.parse`).
/// Layout, all little-endian, mirrored by `decodeCurveBuffer` in `src/ipc.ts`:
///   [u32 curve_count]
///   repeat curve_count times:
///     [u32 name_byte_len][name utf8][u32 point_count][f32 depth × pc][f32 value × pc]
/// (`depth[pc]` then `value[pc]` are already laid out in each series' `data`.)
pub fn pack_curve_series(series: &[TrackCurveSeries]) -> Vec<u8> {
    let cap = 4 + series.iter().map(|s| 8 + s.curve_name.len() + s.data.len()).sum::<usize>();
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(&(series.len() as u32).to_le_bytes());
    for s in series {
        let name = s.curve_name.as_bytes();
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name);
        buf.extend_from_slice(&(s.point_count as u32).to_le_bytes());
        buf.extend_from_slice(&s.data);
    }
    buf
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
    // Names that miss the standard six (or whose standard column is all-NaN because import
    // matched no alias) need computed/generic resolution. Collect them first, then read their
    // computed_curves values in ONE `IN (...)` query instead of a round-trip each — the hot
    // path when a module chains off several previously-computed curves across many wells.
    let mut resolve: Vec<String> = Vec::new();
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
        match std_col {
            Some(col) if upper == "DEPTH" || col.iter().any(|v| !v.is_nan()) => {
                columns.insert(upper, col.clone());
            }
            // An all-NaN standard column means the delivery's mnemonics matched no standard
            // alias at import (e.g. an APS well whose only neutron is APLC): defer to
            // computed/generic resolution so the family dictionary can still find the curve
            // instead of silently feeding NaN to modules.
            _ => {
                if !resolve.contains(&upper) {
                    resolve.push(upper);
                }
            }
        }
    }

    if !resolve.is_empty() {
        // One batched computed_curves read for every deferred name; then per name, fall back
        // to the generic RAW store (mnemonic/family aliased) exactly as the per-curve path did
        // whenever the computed lookup yields nothing usable.
        let computed = fetch_computed_curves_batch(conn, well_id, &resolve, &depth)?;
        for upper in resolve {
            let values = match computed.get(&upper) {
                Some(col) if col.iter().any(|v| !v.is_nan()) => col.clone(),
                _ => fetch_generic_curve_aligned(conn, well_id, &upper, &depth)?,
            };
            columns.insert(upper, values);
        }
    }
    Ok((depth, columns))
}

/// Batched `computed_curves` read for many names at once: a single `upper(curve_name) IN
/// (...)` query, its rows bucketed per name and each aligned onto the depth grid. Replaces N
/// per-curve [`fetch_computed_curve_aligned`] round-trips inside [`fetch_curve_frame`]. Only
/// names with at least one stored row appear in the returned map; a name absent from it (or
/// present but all-NaN after alignment) is left for the caller to resolve via the generic
/// store — preserving the computed-then-generic precedence the per-curve path had, where a
/// non-standard curve came from the generic (RAW) store when it had no computed values.
fn fetch_computed_curves_batch(
    conn: &Connection,
    well_id: &str,
    names_upper: &[String],
    depth_grid: &[f32],
) -> duckdb::Result<HashMap<String, Vec<f32>>> {
    let placeholders = std::iter::repeat("?").take(names_upper.len()).collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT upper(curve_name), depth, value FROM computed_curves \
         WHERE well_id = ? AND upper(curve_name) IN ({placeholders})"
    );
    let mut qp: Vec<&str> = Vec::with_capacity(names_upper.len() + 1);
    qp.push(well_id);
    for n in names_upper {
        qp.push(n.as_str());
    }
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(qp), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?, row.get::<_, f32>(2)?))
    })?;
    // name → (depth-bits → value); last row wins per depth, matching fetch_computed_curve_aligned.
    let mut by_name: HashMap<String, HashMap<u32, f32>> = HashMap::new();
    for r in rows {
        let (nm, d, v) = r?;
        by_name.entry(nm).or_default().insert(d.to_bits(), v);
    }
    Ok(by_name
        .into_iter()
        .map(|(nm, by_depth)| {
            let col = depth_grid.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect();
            (nm, col)
        })
        .collect())
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

/// One well's versioned output, for the batched multi-well writer.
pub(crate) struct WellWrite {
    pub well_id: String,
    pub depth: Vec<f32>,
    pub curves: Vec<(String, Vec<f32>)>,
    pub set_id: String,
}

/// Batched [`create_log_set`]: registers one run event per well inside a SINGLE transaction
/// instead of one auto-committed INSERT (= one WAL fsync) per well. Returns well_id → set_id.
/// Versioning is identical (each well gets 1 + its own MAX(version) for `set_name`).
pub(crate) fn create_log_sets_batch(
    conn: &Connection,
    well_ids: &[String],
    spec: &LogSetSpec,
) -> duckdb::Result<HashMap<String, String>> {
    // Plan every well's version from the CURRENT committed state FIRST (reads only — wells are
    // distinct so there is no cross-well dependency), THEN INSERT them all in one transaction.
    // Reading MAX(version) *after* an INSERT inside the same transaction trips a DuckDB internal
    // error, so the reads deliberately precede all writes.
    let mut planned: Vec<(String, i64, String)> = Vec::with_capacity(well_ids.len());
    for well_id in well_ids {
        let version: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
            params![well_id, spec.set_name],
            |r| r.get(0),
        )?;
        planned.push((well_id.clone(), version, Uuid::new_v4().to_string()));
    }
    crate::db::with_txn(conn, |conn| {
        for (well_id, version, set_id) in &planned {
            conn.execute(
                "INSERT INTO log_sets (set_id, well_id, set_name, version, module, params_json, inputs_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![set_id, well_id, spec.set_name, version, spec.module, spec.params_json, spec.inputs_json],
            )?;
        }
        Ok::<(), duckdb::Error>(())
    })?;
    Ok(planned.into_iter().map(|(w, _, s)| (w, s)).collect())
}

/// Batched [`write_computed_curves_versioned`]: writes MANY wells' outputs in ONE transaction.
///
/// The earlier version mirrored the single-well path per well — one DELETE + a fresh current
/// appender + a fresh archive appender for every well. At field scale (544 wells) that is 544
/// full-table DELETE scans of `computed_curves` plus 1088 appender open/flush/drop cycles, and it
/// dominated the between-step "pause" the user saw. This version keeps the identical semantics
/// (same delete-then-append discipline, same current+archive double-write, each well's rows still
/// carrying that well's own `set_id`) but restructures the work so it runs in seconds:
///
///   Phase 1 — clear the CURRENT store. Wells are grouped by their exact curve-set (every well in
///   a workflow step writes the same curves, so this is normally ONE group), and each group is
///   cleared with a single `DELETE ... WHERE well_id IN (…) AND curve_name IN (…)` — one table
///   pass for the whole batch instead of one per well. Deleting the exact (wells × curves) cross
///   product is safe *because every well in a group has exactly that curve-set*.
///
///   Phase 2/3 — append. With every DELETE already done, ONE appender per table may span all
///   wells: the DuckDB "appender can't span DML on the same table" constraint only forbids
///   interleaving a DELETE while an appender is open, which never happens here.
pub(crate) fn write_computed_curves_versioned_batch(conn: &Connection, wells: &[WellWrite]) -> duckdb::Result<()> {
    if wells.iter().all(|w| w.curves.is_empty()) {
        return Ok(());
    }
    crate::db::with_txn(conn, |conn| {
        // Phase 1: group wells by identical curve-set, then one DELETE per group.
        use std::collections::BTreeMap;
        let mut groups: BTreeMap<Vec<&str>, Vec<&str>> = BTreeMap::new();
        for w in wells {
            if w.curves.is_empty() {
                continue;
            }
            let mut names: Vec<&str> = w.curves.iter().map(|(n, _)| n.as_str()).collect();
            names.sort_unstable();
            names.dedup();
            groups.entry(names).or_default().push(w.well_id.as_str());
        }
        for (curves, well_ids) in &groups {
            let wph = std::iter::repeat("?").take(well_ids.len()).collect::<Vec<_>>().join(", ");
            let cph = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
            let sql =
                format!("DELETE FROM computed_curves WHERE well_id IN ({wph}) AND curve_name IN ({cph})");
            let mut p: Vec<&str> = Vec::with_capacity(well_ids.len() + curves.len());
            p.extend(well_ids.iter().copied());
            p.extend(curves.iter().copied());
            conn.execute(&sql, params_from_iter(p))?;
        }

        // Phase 2: one appender for the CURRENT store across every well.
        {
            let mut current = conn.appender("computed_curves")?;
            for w in wells {
                for (name, values) in &w.curves {
                    for (d, v) in w.depth.iter().zip(values.iter()) {
                        current.append_row(params![w.well_id, d, name, v, w.set_id])?;
                    }
                }
            }
            current.flush()?;
        }

        // Phase 3: one appender for the append-only ARCHIVE across every well.
        {
            let mut archive = conn.appender("computed_curves_archive")?;
            for w in wells {
                for (name, values) in &w.curves {
                    for (d, v) in w.depth.iter().zip(values.iter()) {
                        archive.append_row(params![w.set_id, w.well_id, d, name, v])?;
                    }
                }
            }
            archive.flush()?;
        }
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

/// Distinct constellation (log-set) names across the whole project, alphabetical. The
/// module and workflow dialogs run across many wells at once, so their input/output
/// pickers need the project-wide name list — a single well's `list_log_sets` would miss
/// names that only exist on other wells.
pub(crate) fn list_log_set_names(conn: &Connection) -> duckdb::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT set_name FROM log_sets ORDER BY set_name")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut names = Vec::new();
    for r in rows {
        names.push(r?);
    }
    Ok(names)
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
pub fn run_equation(
    db: &Mutex<Connection>,
    equation: &EquationDef,
    well_ids: &[String],
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<EquationRunResult> {
    let engine = Engine::new();
    let ast: AST = match engine.compile(&equation.script) {
        Ok(ast) => ast,
        Err(e) => {
            if let Some(p) = progress {
                for w in well_ids {
                    p.finish_item(w, crate::jobs::ItemState::Failed, Some(format!("script error: {e}")));
                }
            }
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
            if let Some(p) = progress {
                p.start_item(well_id);
            }
            let (depth, columns) = {
                let conn = db.lock().unwrap();
                match fetch_curve_frame(&conn, well_id, &equation.input_curves) {
                    Ok(v) => v,
                    Err(e) => {
                        if let Some(p) = progress {
                            p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                        }
                        return EquationRunResult::failed(well_id.clone(), e.to_string());
                    }
                }
            };
            if depth.is_empty() {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some("no curve data for well".into()));
                }
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
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                }
                return EquationRunResult::failed(well_id.clone(), e.to_string());
            }
            if let Some(p) = progress {
                p.finish_item(well_id, crate::jobs::ItemState::Ok, None);
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

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    /// The batched `fetch_curve_frame` must return byte-for-byte what the old per-curve path
    /// did: standard columns straight through, computed curves matched case-insensitively and
    /// aligned onto the depth grid (missing depths → NaN), and a name with no computed rows
    /// falling through to the generic store (here: all-NaN, since none is registered).
    #[test]
    fn fetch_curve_frame_batches_computed_and_preserves_semantics() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = "22222222-2222-2222-2222-222222222222";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'BATCH-1');
             INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob) VALUES
                ('{well}', 100.0, 10.0, 2.0, 0.10, 2.40),
                ('{well}', 101.0, 20.0, 2.0, 0.20, 2.50),
                ('{well}', 102.0, 30.0, 2.0, 0.30, 2.60);
             -- VSH present on the full grid; 'phie' stored LOWERCASE and only at 100/102 so
             -- 101 must align to NaN and the uppercased request must still find it.
             INSERT INTO computed_curves (well_id, depth, curve_name, value) VALUES
                ('{well}', 100.0, 'VSH', 0.11),
                ('{well}', 101.0, 'VSH', 0.22),
                ('{well}', 102.0, 'VSH', 0.33),
                ('{well}', 100.0, 'phie', 0.15),
                ('{well}', 102.0, 'phie', 0.25);"
        ))
        .unwrap();

        let names: Vec<String> =
            ["DEPTH", "GR", "VSH", "PHIE", "MADEUP"].iter().map(|s| s.to_string()).collect();
        let (depth, cols) = fetch_curve_frame(&conn, well, &names).unwrap();

        let approx = |a: f32, b: f32| (a - b).abs() < 1e-4;
        assert_eq!(depth, vec![100.0, 101.0, 102.0]);
        // Every requested name is present as a column (callers rely on this).
        for n in ["DEPTH", "GR", "VSH", "PHIE", "MADEUP"] {
            assert!(cols.contains_key(n), "missing column {n}");
        }
        assert_eq!(cols["DEPTH"], depth);
        assert!(cols["GR"].iter().zip([10.0, 20.0, 30.0]).all(|(a, b)| approx(*a, b)));
        assert!(cols["VSH"].iter().zip([0.11, 0.22, 0.33]).all(|(a, b)| approx(*a, b)), "{:?}", cols["VSH"]);
        // Case-insensitive match + off-grid depth 101 → NaN.
        assert!(approx(cols["PHIE"][0], 0.15), "{:?}", cols["PHIE"]);
        assert!(cols["PHIE"][1].is_nan(), "off-grid depth must be NaN, got {}", cols["PHIE"][1]);
        assert!(approx(cols["PHIE"][2], 0.25));
        // No computed rows and no generic curve registered → all NaN (generic fallback).
        assert!(cols["MADEUP"].iter().all(|v| v.is_nan()), "absent curve should be all-NaN");
    }

    /// A single-name request still works through the `IN (?)` builder (placeholder count 1).
    #[test]
    fn fetch_curve_frame_single_computed_name() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = "33333333-3333-3333-3333-333333333333";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'BATCH-2');
             INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob) VALUES
                ('{well}', 500.0, 5.0, 1.0, 0.05, 2.65);
             INSERT INTO computed_curves (well_id, depth, curve_name, value) VALUES
                ('{well}', 500.0, 'PERM', 12.5);"
        ))
        .unwrap();
        let (_d, cols) = fetch_curve_frame(&conn, well, &["PERM".to_string()]).unwrap();
        assert!((cols["PERM"][0] - 12.5).abs() < 1e-4);
    }

    /// The raw-IPC buffer must round-trip: pack two series, decode by hand exactly the way
    /// `decodeCurveBuffer` in src/ipc.ts does, and recover names / point counts / values
    /// (incl. NaN). Floats are read via `from_le_bytes` — NOT `cast_slice` — because the
    /// name bytes leave the data blocks at an arbitrary (non-4-aligned) offset.
    #[test]
    fn pack_curve_series_roundtrips() {
        let mk = |name: &str, depth: &[f32], value: &[f32]| {
            let mut packed: Vec<f32> = Vec::new();
            packed.extend_from_slice(depth);
            packed.extend_from_slice(value);
            TrackCurveSeries { curve_name: name.into(), point_count: depth.len(), data: bytemuck::cast_slice(&packed).to_vec() }
        };
        let a = mk("GR", &[100.0, 101.0], &[12.0, 34.0]);
        let b = mk("PHIE", &[200.0, 201.0, 202.0], &[0.1, 0.2, f32::NAN]);
        let buf = pack_curve_series(&[a, b]);

        let u32_at = |off: usize| u32::from_le_bytes(buf[off..off + 4].try_into().unwrap());
        let mut off = 0usize;
        let count = u32_at(off) as usize;
        off += 4;
        assert_eq!(count, 2);

        let mut got: Vec<(String, Vec<f32>, Vec<f32>)> = Vec::new();
        for _ in 0..count {
            let nlen = u32_at(off) as usize;
            off += 4;
            let name = String::from_utf8(buf[off..off + nlen].to_vec()).unwrap();
            off += nlen;
            let pc = u32_at(off) as usize;
            off += 4;
            let mut floats = Vec::with_capacity(pc * 2);
            for k in 0..pc * 2 {
                let p = off + k * 4;
                floats.push(f32::from_le_bytes(buf[p..p + 4].try_into().unwrap()));
            }
            off += pc * 8;
            let (d, v) = floats.split_at(pc);
            got.push((name, d.to_vec(), v.to_vec()));
        }
        assert_eq!(off, buf.len(), "no trailing bytes");
        assert_eq!(got[0].0, "GR");
        assert_eq!(got[0].1, vec![100.0, 101.0]);
        assert_eq!(got[0].2, vec![12.0, 34.0]);
        assert_eq!(got[1].0, "PHIE");
        assert_eq!(got[1].1, vec![200.0, 201.0, 202.0]);
        assert!((got[1].2[1] - 0.2).abs() < 1e-6);
        assert!(got[1].2[2].is_nan(), "NaN value survives the pack");
    }
}
