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
        let values = match upper.as_str() {
            "DEPTH" => depth.clone(),
            "GR" => gr.clone(),
            "RES_DEEP" => res_deep.clone(),
            "NPHI" => nphi.clone(),
            "RHOB" => rhob.clone(),
            "DT" => dt.clone(),
            "SP" => sp.clone(),
            other => fetch_named_curve_aligned(conn, well_id, other, &depth)?,
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
    let mut stmt = conn.prepare("SELECT depth, value FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2")?;
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

/// Replaces any prior values for (well_id, curve_name) with the freshly computed ones.
pub(crate) fn write_computed_curve(conn: &Connection, well_id: &str, depth: &[f32], curve_name: &str, values: &[f32]) -> duckdb::Result<()> {
    write_computed_curves_batch(conn, well_id, depth, &[(curve_name, values)])
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
            appender.append_row(params![well_id, d, name, v])?;
        }
    }
    appender.flush()?;
    Ok(())
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
            if let Err(e) = write_computed_curve(&conn, well_id, &depth, &equation.output_curve, &output) {
                return EquationRunResult::failed(well_id.clone(), e.to_string());
            }
            EquationRunResult::success(well_id.clone(), n)
        })
        .collect()
}
