use duckdb::{params, params_from_iter, Connection, OptionalExt};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A user-authored petrophysical equation (Rhai script), analogous to a Loglan
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
    /// The run succeeded but not every sample did — today, a Rhai script that raised on some
    /// depths. Distinct from `error` on purpose: the curve WAS written and is usable, so calling
    /// it a failure would be wrong, but saying nothing is worse. See [`run_equation`].
    #[serde(default)]
    pub note: Option<String>,
}

impl EquationRunResult {
    pub(crate) fn success(well_id: String, rows_written: usize) -> Self {
        Self { well_id, rows_written, error: None, note: None }
    }
    pub(crate) fn failed(well_id: String, error: String) -> Self {
        Self { well_id, rows_written: 0, error: Some(error), note: None }
    }
    fn with_note(mut self, note: Option<String>) -> Self {
        self.note = note;
        self
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

/// One full-resolution track curve on the depth grid that owns its samples. The interactive
/// viewer decimates this for IPC; SVG/PDF composite output consumes it directly so export and
/// screen resolve the same `(set, mnemonic)` identity.
#[derive(Debug, Clone)]
pub(crate) struct TrackCurveFrame {
    pub curve_name: String,
    pub depth: Vec<f32>,
    pub value: Vec<f32>,
}

/// One curve requested by a log layout. `set_name = None` preserves the application's
/// established standard/computed/RAW resolution. Naming an imported set is explicit and
/// therefore reads that set's own samples without projecting them onto the standard frame.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackCurveRequest {
    pub curve_name: String,
    #[serde(default)]
    pub set_name: Option<String>,
}

/// Stable lookup key mirrored by `src/trackCurveRequest.ts`. Unqualified curves retain their
/// historical upper-case mnemonic key; qualified equal mnemonics can coexist in one layout.
pub fn track_curve_key(request: &TrackCurveRequest) -> String {
    let curve = request.curve_name.trim().to_uppercase();
    match request
        .set_name
        .as_deref()
        .map(str::trim)
        .filter(|set| !set.is_empty())
    {
        Some(set) => format!("{set}\u{001f}{curve}"),
        None => curve,
    }
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
    curve_requests: &[TrackCurveRequest],
    target_pixel_height: usize,
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> duckdb::Result<Vec<TrackCurveSeries>> {
    fetch_track_frames(conn, well_id, curve_requests, depth_min, depth_max)?
        .into_iter()
        .map(|frame| {
            let (dec_depth, dec_value) =
                crate::decimate::min_max_decimate(&frame.depth, &frame.value, target_pixel_height);
            let point_count = dec_depth.len();
            let mut packed = Vec::with_capacity(point_count * 2);
            packed.extend_from_slice(&dec_depth);
            packed.extend_from_slice(&dec_value);
            Ok(TrackCurveSeries {
                curve_name: frame.curve_name,
                point_count,
                data: bytemuck::cast_slice(&packed).to_vec(),
            })
        })
        .collect()
}

/// Resolves every request onto the grid that owns it without display decimation. This is the
/// common identity boundary for the WebGPU viewer and composite SVG/PDF/report renderer.
pub(crate) fn fetch_track_frames(
    conn: &Connection,
    well_id: &str,
    curve_requests: &[TrackCurveRequest],
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> duckdb::Result<Vec<TrackCurveFrame>> {
    let current_names: Vec<String> = curve_requests
        .iter()
        .filter(|request| request.set_name.as_deref().map(str::trim).filter(|set| !set.is_empty()).is_none())
        .map(|request| request.curve_name.trim().to_uppercase())
        .collect();
    let (current_depth, current_columns) = if current_names.is_empty() {
        (Vec::new(), HashMap::new())
    } else {
        fetch_curve_frame(conn, well_id, &current_names)?
    };

    curve_requests
        .iter()
        .map(|request| {
            let upper = request.curve_name.trim().to_uppercase();
            let (depth, values) = match request.set_name.as_deref().map(str::trim).filter(|set| !set.is_empty()) {
                Some(set) => fetch_generic_curve_native(conn, well_id, set, &upper, depth_min, depth_max)?,
                None => {
                    let values = current_columns.get(&upper).cloned().unwrap_or_default();
                    filter_curve_interval(&current_depth, &values, depth_min, depth_max)
                }
            };
            Ok(TrackCurveFrame {
                curve_name: track_curve_key(request),
                depth,
                value: values,
            })
        })
        .collect()
}

fn filter_curve_interval(
    depth: &[f32],
    values: &[f32],
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> (Vec<f32>, Vec<f32>) {
    if depth_min.is_none() && depth_max.is_none() {
        return (depth.to_vec(), values.to_vec());
    }
    let lo = depth_min.unwrap_or(f32::NEG_INFINITY);
    let hi = depth_max.unwrap_or(f32::INFINITY);
    depth
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, d)| *d >= lo && *d <= hi)
        .map(|(index, d)| (d, values.get(index).copied().unwrap_or(f32::NAN)))
        .unzip()
}

/// Resolves one exact mnemonic within one explicit imported set and returns that curve's
/// own ordered samples. Filtering is performed in DuckDB before decimation, so zooming does
/// not repeatedly transfer the whole well and never changes the rows in storage.
fn fetch_generic_curve_native(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    curve_name: &str,
    depth_min: Option<f32>,
    depth_max: Option<f32>,
) -> duckdb::Result<(Vec<f32>, Vec<f32>)> {
    let curve_id: Option<String> = match conn.query_row(
        "SELECT curve_id FROM curve_meta
             WHERE well_id = ?1 AND set_name = ?2 AND upper(mnemonic) = ?3
             ORDER BY COALESCE(pinned, 0) DESC, modified_seq DESC NULLS LAST,
                      run_no DESC NULLS LAST, curve_id
             LIMIT 1",
        params![well_id, set_name, curve_name],
        |row| row.get(0),
    ) {
        Ok(curve_id) => Some(curve_id),
        Err(duckdb::Error::QueryReturnedNoRows) => None,
        Err(error) => return Err(error),
    };
    let Some(curve_id) = curve_id else {
        return Ok((Vec::new(), Vec::new()));
    };

    let mut stmt = conn.prepare(
        "SELECT depth, value FROM curve_samples
         WHERE curve_id = ?1
           AND (?2 IS NULL OR depth >= ?2)
           AND (?3 IS NULL OR depth <= ?3)
         ORDER BY depth",
    )?;
    let rows = stmt.query_map(params![curve_id, depth_min, depth_max], |row| {
        Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
    })?;
    let mut depth = Vec::new();
    let mut values = Vec::new();
    for row in rows {
        let (d, v) = row?;
        depth.push(d);
        values.push(v);
    }
    Ok((depth, values))
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
/// Only the well's ACTIVE core set is plotted — two deliveries of the same plugs overlaid
/// would read as twice the data (see `db::list_core_sets` / the set manager to switch).
pub fn fetch_core_series(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<TrackCurveSeries>> {
    let mut stmt = conn.prepare(
        "SELECT depth, cpor, cperm, cgd, csw FROM core_data
         WHERE well_id = ?1 AND set_name = COALESCE((SELECT set_name FROM core_sets WHERE well_id = ?1
                                                     ORDER BY active DESC, imported_at DESC LIMIT 1), 'RAW')
         ORDER BY depth",
    )?;
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

    // SB-MLA-035. A DECLARED unit (`curve_unit`, written by whatever produced the curve) wins over
    // the equations join, which can only ever answer for a user equation — a curve written by a
    // module or an ML run had no unit anywhere before this. `MAX` because the catalog is
    // project-wide while a declaration is per well: where two wells disagree the catalog picks one
    // rather than listing the curve twice, and the per-well truth stays available through
    // `db::curve_unit_for`.
    let mut stmt = conn.prepare(
        "SELECT DISTINCT cc.curve_name, COALESCE(u.unit, e.output_units)
         FROM computed_curves cc
         LEFT JOIN equations e ON e.output_curve = cc.curve_name
         LEFT JOIN (SELECT upper(curve_name) AS cn, MAX(unit) AS unit FROM curve_unit GROUP BY 1) u
                ON u.cn = upper(cc.curve_name)
         ORDER BY cc.curve_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CurveCatalogEntry { name: row.get(0)?, units: row.get(1)?, source: "Computed".into() })
    })?;
    for r in rows {
        entries.push(r?);
    }

    // The IMPORTED logs — the generic store. Rule 11 has always let a module or an equation consume
    // any imported mnemonic, because `fetch_curve_frame` falls through to
    // `fetch_generic_curve_aligned`; the CATALOG never listed them, so anything that offers the user
    // a list of curves to choose from offered the six standard columns and the computed ones only.
    // A well delivered with PEF, CALI, DRHO, SGR and three resistivities showed none of them, and
    // the picker looked like the product could not read them — the backend could, all along.
    //
    // DISTINCT on the mnemonic because the catalog is PROJECT-WIDE and the store is per
    // (well, set, run): one PEF entry, not one per well per delivery. `MIN(unit)` picks a
    // representative for the same reason the computed join uses MAX — the per-well truth stays
    // available through the frame read, which resolves per well anyway.
    let mut stmt = conn.prepare(
        "SELECT upper(mnemonic), MIN(unit)
         FROM curve_meta
         WHERE upper(mnemonic) <> 'DEPTH'
         GROUP BY upper(mnemonic)
         ORDER BY 1",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CurveCatalogEntry { name: row.get(0)?, units: row.get(1)?, source: "Imported".into() })
    })?;
    for r in rows {
        let e = r?;
        // A name already carried by a standard column or a computed curve keeps THAT entry: those
        // are what `fetch_curve_frame` resolves first, so listing the imported one beside it would
        // offer a choice the reader does not actually have.
        if !entries.iter().any(|x| x.name.eq_ignore_ascii_case(&e.name)) {
            entries.push(e);
        }
    }
    Ok(entries)
}

type CurveFrame = (Vec<f32>, HashMap<String, Vec<f32>>);

/// The `standard_curves` columns, which every curve read resolves FIRST.
///
/// A computed curve stored under one of these names is written, counted and reported — and then
/// invisible, because [`fetch_curve_frame`] hands every reader the raw standard column instead. It
/// is the same shape as the `CPHOTO_*` trace saved at the photograph's own sampling: a run that
/// reports success and a project that holds a curve nothing can open.
///
/// ONE list, consulted by [`crate::workflow::resolve_output_names`] before a run writes anything.
/// It lived in `condition.rs` and again in `frame.rs`, which is two places for a seventh standard
/// column to be forgotten.
/// Reads the requested curve mnemonics for one well, aligned onto that well's standard
/// depth grid. Non-standard names are looked up in `computed_curves` (so equations can
/// chain off previously computed curves).
pub(crate) fn fetch_curve_frame(conn: &Connection, well_id: &str, curve_names: &[String]) -> duckdb::Result<CurveFrame> {
    let projections = crate::schema_vocab::standard_projections();
    let sql = format!(
        "SELECT {} FROM standard_curves WHERE well_id = ?1 ORDER BY depth",
        projections.select_list
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![well_id], |row| {
        (0..crate::schema_vocab::STANDARD_COLUMNS.len())
            .map(|index| row.get::<_, Option<f32>>(index).map(|value| value.unwrap_or(f32::NAN)))
            .collect::<duckdb::Result<Vec<_>>>()
    })?;
    let mut standard = crate::schema_vocab::STANDARD_COLUMNS
        .iter()
        .map(|column| (column.mnemonic, Vec::new()))
        .collect::<HashMap<_, _>>();
    for row in rows {
        for (column, value) in crate::schema_vocab::STANDARD_COLUMNS.iter().zip(row?) {
            standard
                .get_mut(column.mnemonic)
                .expect("registered standard column")
                .push(value);
        }
    }
    let depth = standard
        .get("DEPTH")
        .expect("validated standard depth column")
        .clone();

    let mut columns: HashMap<String, Vec<f32>> = HashMap::new();
    // Names that miss the standard six (or whose standard column is all-NaN because import
    // matched no alias) need computed/generic resolution. Collect them first, then read their
    // computed_curves values in ONE `IN (...)` query instead of a round-trip each — the hot
    // path when a module chains off several previously-computed curves across many wells.
    let mut resolve: Vec<String> = Vec::new();
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        let std_col = standard.get(upper.as_str());
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
        // to the generic store (mnemonic/family aliased) exactly as the per-curve path did
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

/// What one curve's OWN sampling looks like, and how much of it survives the join onto a frame.
///
/// Every read in this module aligns by EXACT depth match. That is correct and cheap when the curves
/// share a grid, which they do whenever they came from one delivery — and it silently returns
/// nothing when they do not. A resistivity delivered on a 0.5 m grid, joined onto a 0.1524 m frame,
/// coincides at no depth at all: the curve is fully logged, fully stored, and reads as absent.
///
/// The diagnosis matters more than the count. "Missing" and "present on a different grid" call for
/// opposite responses — go and find the curve, versus reconcile the sampling — and until now the
/// second reported itself as the first, which sends an interpreter looking for a log they already
/// have.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CurveSampling {
    pub curve: String,
    /// Samples the curve actually holds, on its own depths.
    pub n_own: usize,
    /// Median spacing between its own consecutive depths. `None` below two samples, where a
    /// spacing is undefined rather than zero.
    pub step: Option<f64>,
    pub top: Option<f64>,
    pub base: Option<f64>,
    /// How many of the frame's depths this curve answers for. Zero beside a large `n_own` is the
    /// whole finding.
    pub n_on_frame: usize,
    /// True where the curve was found in the generic store rather than as a standard column or a
    /// computed curve — the only place a foreign grid can come from today.
    pub imported: bool,
}

/// Measures each named curve's own sampling against a frame, for one well.
///
/// Deliberately separate from `fetch_curve_frame`: that answers "give me these curves on this grid",
/// which is what a module wants, and this answers "why did that come back empty", which is what a
/// person wants. Folding the second into the first would put a per-curve extra query on the hot path
/// of every module run in the application.
pub fn curve_sampling(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
    frame: &[f32],
) -> duckdb::Result<Vec<CurveSampling>> {
    let frame_bits: std::collections::HashSet<u32> = frame.iter().map(|d| d.to_bits()).collect();
    let mut out = Vec::new();
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        // Own depths, wherever the curve lives. The generic store is asked first because it is the
        // only source that can carry a grid of its own; a computed curve was written against a
        // frame and a standard column IS the frame.
        let curve_id: Option<String> = conn
            .query_row(
                "SELECT curve_id FROM curve_meta
                 WHERE well_id = ?1 AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
                 ORDER BY (set_name = 'RAW') DESC,
                          (upper(mnemonic) = ?2) DESC,
                          (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                          COALESCE(final_flag, 0) DESC,
                          modified_seq DESC NULLS LAST, curve_id
                 LIMIT 1",
                params![well_id, upper],
                |row| row.get(0),
            )
            .ok();
        let depths: Vec<f32> = match &curve_id {
            Some(id) => {
                let mut stmt = conn.prepare(
                    "SELECT depth FROM curve_samples WHERE curve_id = ?1 AND value IS NOT NULL ORDER BY depth",
                )?;
                stmt.query_map(params![id], |r| r.get::<_, f32>(0))?.collect::<duckdb::Result<_>>()?
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT depth FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = ?2 ORDER BY depth",
                )?;
                stmt.query_map(params![well_id, upper], |r| r.get::<_, f32>(0))?
                    .collect::<duckdb::Result<_>>()?
            }
        };
        let n_own = depths.len();
        let n_on_frame = depths.iter().filter(|d| frame_bits.contains(&d.to_bits())).count();
        // MEDIAN spacing, not mean: one gap across a casing shoe would drag a mean far off the
        // sampling the tool actually ran at, and the sampling is the question.
        let step = if n_own >= 2 {
            let mut gaps: Vec<f64> =
                depths.windows(2).map(|w| (w[1] - w[0]) as f64).filter(|g| *g > 0.0).collect();
            if gaps.is_empty() {
                None
            } else {
                gaps.sort_by(|a, b| a.partial_cmp(b).expect("finite by construction"));
                Some(gaps[gaps.len() / 2])
            }
        } else {
            None
        };
        out.push(CurveSampling {
            curve: upper,
            n_own,
            step,
            top: depths.first().map(|d| *d as f64),
            base: depths.last().map(|d| *d as f64),
            n_on_frame,
            imported: curve_id.is_some(),
        });
    }
    Ok(out)
}

/// The well's own frame — the depths every read in this module aligns onto.
pub fn well_frame(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<f32>> {
    let mut stmt =
        conn.prepare("SELECT depth FROM standard_curves WHERE well_id = ?1 ORDER BY depth")?;
    stmt.query_map(params![well_id], |r| r.get::<_, f32>(0))?.collect()
}

#[derive(Debug, Clone)]
struct GenericCurveCandidate {
    curve_id: String,
    set_name: String,
    set_version: i64,
    mnemonic: String,
    pinned: bool,
    final_flag: bool,
    modified_seq: Option<i64>,
}

#[derive(Debug, Clone)]
struct GenericCurveDecision {
    chosen: GenericCurveCandidate,
    rule: Option<CurveResolutionRule>,
    rejected: Vec<GenericCurveCandidate>,
}

/// Resolves a generic-store curve once, returning both the winner and the candidates it beat.
/// The same decision feeds the numeric reader and the persisted ancestry record; duplicating the
/// ORDER BY in those two paths would allow a run to record one GR while calculating from another.
///
/// `final_flag` is an explicit, reversible curve-level decision. A set named `FINAL` is not enough:
/// one set can contain several runs of the same family, and treating its label as the decision would
/// recreate the ambiguity this record is meant to expose. RAW is the ordinary working input set
/// and therefore precedes every later filter. Exact mnemonic, user promotion and MRU are
/// deterministic stages within the selected working-set tier.
fn resolve_generic_curve_decision(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
) -> duckdb::Result<Option<GenericCurveDecision>> {
    let upper = curve_name.trim().to_uppercase();
    let mut stmt = conn.prepare(
        "SELECT curve_id, set_name, set_version, mnemonic, COALESCE(pinned, 0),
                COALESCE(final_flag, 0), modified_seq
         FROM curve_meta
         WHERE well_id = ?1
           AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
         ORDER BY (set_name = 'RAW') DESC,
                  (upper(mnemonic) = ?2) DESC,
                  (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                  COALESCE(final_flag, 0) DESC,
                  modified_seq DESC NULLS LAST,
                  curve_id",
    )?;
    let candidates = stmt
        .query_map(params![well_id, upper], |row| {
            Ok(GenericCurveCandidate {
                curve_id: row.get(0)?,
                set_name: row.get(1)?,
                set_version: row.get(2)?,
                mnemonic: row.get(3)?,
                pinned: row.get::<_, i32>(4)? != 0,
                final_flag: row.get::<_, i32>(5)? != 0,
                modified_seq: row.get(6)?,
            })
        })?
        .collect::<duckdb::Result<Vec<_>>>()?;
    let Some(chosen) = candidates.first().cloned() else {
        return Ok(None);
    };
    let rejected = candidates[1..].to_vec();
    let exact = |candidate: &GenericCurveCandidate| candidate.mnemonic.eq_ignore_ascii_case(&upper);
    let working = |candidate: &GenericCurveCandidate| candidate.set_name.eq_ignore_ascii_case("RAW");
    let mut survivors: Vec<&GenericCurveCandidate> = candidates.iter().collect();

    let had_working_choice = survivors.iter().any(|candidate| working(candidate))
        && survivors.iter().any(|candidate| !working(candidate));
    if had_working_choice {
        survivors.retain(|candidate| working(candidate));
    }
    let rule = if had_working_choice && survivors.len() == 1 {
        CurveResolutionRule::WorkingInputSet
    } else {
        let chosen_exact = exact(&chosen);
        let had_exact_choice = survivors.iter().any(|candidate| exact(candidate))
            && survivors.iter().any(|candidate| !exact(candidate));
        if had_exact_choice {
            survivors.retain(|candidate| exact(candidate));
        }
        if survivors.len() == 1 {
            if chosen_exact {
                CurveResolutionRule::ExplicitName
            } else {
                CurveResolutionRule::AliasAutomatic
            }
        } else {
            let chosen_manual = chosen_exact && chosen.pinned;
            survivors.retain(|candidate| (exact(candidate) && candidate.pinned) == chosen_manual);
            if survivors.len() == 1 && chosen_manual {
                CurveResolutionRule::AliasManual
            } else {
                survivors.retain(|candidate| candidate.final_flag == chosen.final_flag);
                if survivors.len() == 1 {
                    CurveResolutionRule::FinalFlag
                } else {
                    CurveResolutionRule::CurveTypeMru
                }
            }
        }
    };
    let rule = if rule == CurveResolutionRule::CurveTypeMru
        && chosen.modified_seq.is_none()
        && survivors.iter().any(|candidate| {
            candidate.curve_id != chosen.curve_id && candidate.modified_seq.is_none()
        })
    {
        None
    } else {
        Some(rule)
    };
    Ok(Some(GenericCurveDecision {
        chosen,
        rule,
        rejected,
    }))
}

/// The chosen native curve id for non-ancestry consumers. Keeping this tiny projection beside the
/// full decision prevents plots and diagnostics from copying the precedence SQL and drifting away
/// from the bytes and provenance used by module runs.
pub(crate) fn resolve_generic_curve_id(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
) -> duckdb::Result<Option<String>> {
    Ok(resolve_generic_curve_decision(conn, well_id, curve_name)?
        .map(|decision| decision.chosen.curve_id))
}

/// Looks up a curve in the generic store (`curve_meta`/`curve_samples`) by the one structured
/// resolution decision above and aligns its samples onto the depth grid.
fn fetch_generic_curve_aligned(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    depth_grid: &[f32],
) -> duckdb::Result<Vec<f32>> {
    let Some(decision) = resolve_generic_curve_decision(conn, well_id, curve_name)? else {
        return Ok(vec![f32::NAN; depth_grid.len()]);
    };
    let curve_id = decision.chosen.curve_id;

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

/// Replaces only legacy standard-column projections with their native stored identities for a
/// calculation input. Ordinary track reads deliberately retain the projection contract; module
/// and input-set reads need this overlay so the numeric bytes and SB-DBM-006 decision record are
/// produced by the same resolver. Curves supplied by an explicitly selected computed set are
/// excluded because that earlier stage already won the documented resolution chain.
fn overlay_resolved_native_standard_inputs(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
    depth: &[f32],
    columns: &mut HashMap<String, Vec<f32>>,
    resolved_from_selected_set: &std::collections::HashSet<String>,
) -> duckdb::Result<()> {
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        if upper == "DEPTH"
            || crate::schema_vocab::standard_column(&upper).is_none()
            || resolved_from_selected_set.contains(&upper)
        {
            continue;
        }
        if resolve_generic_curve_decision(conn, well_id, &upper)?.is_some() {
            columns.insert(
                upper.clone(),
                fetch_generic_curve_aligned(conn, well_id, &upper, depth)?,
            );
        }
    }
    Ok(())
}

/// Computed-provenance-only resolution for unit-contract inputs (ArgSpec.computed_only,
/// e.g. gascorr's FTEMP/FPRESS): the named input set's archived values first (matching
/// [`fetch_curve_frame_from_set`] semantics, including the own-set precedence), then
/// current `computed_curves` — never the RAW import store, so a commercial LAS export's
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
/// Test fixtures may still seed a raw current curve through this helper; production code must
/// create and write a complete ancestry set instead.
#[cfg(test)]
pub(crate) fn write_computed_curve(conn: &Connection, well_id: &str, depth: &[f32], curve_name: &str, values: &[f32]) -> duckdb::Result<()> {
    write_computed_curves_batch(conn, well_id, depth, &[(curve_name, values)])
}

/// Removes a well's computed curve of `name` (case-insensitive). Used so a survey-derived
/// TVD/TVDSS yields to an imported (generic RAW) curve of the same name: `fetch_curve_frame`
/// ranks computed ABOVE generic, so a leftover computed curve would otherwise keep shadowing
/// the authoritative import.
pub(crate) fn delete_computed_curve(conn: &Connection, well_id: &str, name: &str) -> duckdb::Result<()> {
    conn.execute(
        "DELETE FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = upper(?2)",
        params![well_id, name],
    )?;
    Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct SetWriteDiscipline {
    sampling_style: crate::schema_vocab::SamplingStyle,
}

impl Default for SetWriteDiscipline {
    fn default() -> Self {
        // Existing module/equation outputs are continuous. Until SB-DBM-028 verifies regularity,
        // IRREGULAR is the conservative declaration: it promises no increment the writer has not
        // checked, while still enforcing depth uniqueness.
        Self {
            sampling_style: crate::schema_vocab::SamplingStyle::ContinuousIrregular,
        }
    }
}

/// Stable schema key embedded in `log_sets.params_json` without adding a second write path or
/// changing the deliberately PK-less `computed_curves` table. Existing top-level parameter keys
/// remain readable; the complete record travels with the same log-set row every current/archive
/// curve already cites.
pub(crate) const CURVE_ANCESTRY_KEY: &str = "_sandibumi_curve_ancestry_v1";
pub(crate) const CURVE_ANCESTRY_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AncestryActorKind {
    Human,
    Automated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryActor {
    pub kind: AncestryActorKind,
    /// Explicit session identity. It is never inferred from a Windows account and is separate
    /// from a report's optional "Prepared by" field.
    pub identity: String,
}

/// User-supplied custody attached to a computation request. The backend supplies the timestamp and
/// resolves curve/set identities from the project; the frontend cannot fabricate either. One
/// source/reference note may cover the explicit values in a run, while manifest defaults and
/// stored zone/plot values retain their own more specific sources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunCustody {
    pub actor: AncestryActor,
    pub source_note: String,
}

impl RunCustody {
    pub fn validate(&self) -> Result<(), String> {
        if self.actor.identity.trim().is_empty() {
            return Err("run refused: enter the session operator identity before computing".into());
        }
        if self.source_note.trim().is_empty() {
            return Err(
                "run refused: enter a source/reference note for the explicit run values".into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CurveResolutionRule {
    ExplicitName,
    WorkingInputSet,
    AliasOff,
    AliasManual,
    AliasAutomatic,
    FinalFlag,
    CurveTypeMru,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectedCurveCandidate {
    pub curve_id: String,
    pub log_set: String,
    pub set_version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryInput {
    pub well_id: String,
    pub argument: String,
    pub curve: String,
    pub log_set: String,
    pub set_version: Option<i64>,
    pub set_id: String,
    /// The exact stored curve identity that supplied this input. Imported curves use their native
    /// curve UUID; computed curves use the resolvable `computed:<set UUID>:<curve name>` composite
    /// because the current computed store has no standalone curve UUID. Absent only on readable
    /// schema-v1 history written before SB-DBM-006; every schema-v2 writer is fail-closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_curve_id: Option<String>,
    /// The declared resolution stage that selected `chosen_curve_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<CurveResolutionRule>,
    /// Every candidate considered by the same resolver after the winner, in deterministic order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_candidates: Vec<RejectedCurveCandidate>,
}

/// The one controlled vocabulary for provenance that is absent for a known reason. These are
/// states, not substitute values: sample absence remains `f32::NAN`, and a serialization failure
/// remains an error that prevents the run record from being written.
pub use crate::schema_vocab::ProvenanceAbsentState;

pub(crate) const REQUIRED_UNSET_PARAMETER_STATE: &str =
    ProvenanceAbsentState::RequiredUnset.as_str();

pub(crate) fn parameter_state_for(
    parameters: &[AncestryParameter],
) -> Option<ProvenanceAbsentState> {
    parameters.is_empty().then_some(ProvenanceAbsentState::NotApplicable)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParameterResolution {
    Explicit,
    Defaulted,
}

impl ParameterResolution {
    fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "EXPLICIT",
            Self::Defaulted => "DEFAULTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AncestryParameter {
    pub name: String,
    pub value: serde_json::Value,
    pub source: String,
    /// How a declared module parameter obtained its effective value. `None` is retained only for
    /// schema-v1 legacy rows and derived metadata such as `method_id`, neither of which may be
    /// relabelled as a user decision after the fact.
    pub resolution: Option<ParameterResolution>,
    /// Present only for DEFAULTED values and identifies the exact module manifest that supplied
    /// the default. Historical runs therefore cannot be reinterpreted by a later manifest.
    pub manifest_version: Option<String>,
    /// Present only when the corpus records competing positions for this parameter. Optional so
    /// schema-v1 ancestry written before SB-CORE-013 remains readable without being relabelled.
    pub decision: Option<crate::param_sources::ParameterDecision>,
}

#[derive(Serialize, Deserialize)]
struct AncestryParameterWire {
    name: String,
    value: Option<serde_json::Value>,
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state: Option<ProvenanceAbsentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolution: Option<ParameterResolution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    manifest_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decision: Option<crate::param_sources::ParameterDecision>,
}

impl AncestryParameter {
    fn is_required_unset(&self) -> bool {
        self.value.as_str() == Some(crate::modules::ABSENT_DEFAULT_SOURCE)
            && self.source == crate::modules::ABSENT_DEFAULT_SOURCE
    }
}

impl Serialize for AncestryParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let required_unset = self.is_required_unset();
        AncestryParameterWire {
            name: self.name.clone(),
            value: (!required_unset).then(|| self.value.clone()),
            source: (!required_unset).then(|| self.source.clone()),
            state: required_unset.then_some(ProvenanceAbsentState::RequiredUnset),
            resolution: self.resolution,
            manifest_version: self.manifest_version.clone(),
            decision: self.decision.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AncestryParameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = AncestryParameterWire::deserialize(deserializer)?;
        match wire.state {
            Some(ProvenanceAbsentState::RequiredUnset) => {
                if wire.value.is_some()
                    || wire.source.is_some()
                    || wire.resolution.is_some()
                    || wire.manifest_version.is_some()
                {
                    return Err(serde::de::Error::custom(
                        "REQUIRED_UNSET parameter must have null value, source, resolution, and manifest version",
                    ));
                }
                Ok(Self {
                    name: wire.name,
                    value: serde_json::json!(crate::modules::ABSENT_DEFAULT_SOURCE),
                    source: crate::modules::ABSENT_DEFAULT_SOURCE.to_string(),
                    resolution: None,
                    manifest_version: None,
                    decision: wire.decision,
                })
            }
            Some(other) => Err(serde::de::Error::custom(format!(
                "invalid state {other:?} for a named parameter"
            ))),
            None => {
                let value = wire
                    .value
                    .ok_or_else(|| serde::de::Error::custom("sourced parameter is missing value"))?;
                let source = wire
                    .source
                    .ok_or_else(|| serde::de::Error::custom("sourced parameter is missing source"))?;
                match (wire.resolution, wire.manifest_version.as_deref()) {
                    (Some(ParameterResolution::Explicit), Some(_)) => {
                        return Err(serde::de::Error::custom(
                            "EXPLICIT parameter must not name a default manifest version",
                        ));
                    }
                    (Some(ParameterResolution::Defaulted), Some(version))
                        if !version.trim().is_empty() => {}
                    (Some(ParameterResolution::Defaulted), _) => {
                        return Err(serde::de::Error::custom(
                            "DEFAULTED parameter must name a non-empty manifest version",
                        ));
                    }
                    (None, Some(_)) => {
                        return Err(serde::de::Error::custom(
                            "legacy parameter without a resolution cannot name a manifest version",
                        ));
                    }
                    _ => {}
                }
                Ok(Self {
                    name: wire.name,
                    value,
                    source,
                    resolution: wire.resolution,
                    manifest_version: wire.manifest_version,
                    decision: wire.decision,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AncestryZone {
    pub name: String,
    pub top: f32,
    pub base: f32,
    /// Source/reference note for the numeric zone definition. A blank note is not silently
    /// replaced by "operator input" because that would fabricate custody.
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "kind",
    content = "definitions",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum AncestryZoneScope {
    WholeWell,
    Defined(Vec<AncestryZone>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AncestryOutput {
    pub curve: String,
    pub derivation: String,
}

/// Complete SB-CORE-010 record attached to one per-well log-set version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurveAncestry {
    pub schema_version: u32,
    pub module: String,
    pub module_version: String,
    pub inputs: Vec<AncestryInput>,
    pub parameters: Vec<AncestryParameter>,
    /// Present exactly when the current run genuinely has no parameters. Schema-v1/v2 records
    /// omitted this field; readers classify an empty legacy list as `LEGACY_UNRECORDED` rather
    /// than guessing that it meant `NOT_APPLICABLE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_state: Option<ProvenanceAbsentState>,
    pub zone_scope: AncestryZoneScope,
    pub actor: AncestryActor,
    pub timestamp_utc_ms: u64,
    pub outputs: Vec<AncestryOutput>,
}

impl CurveAncestry {
    fn validate(&self) -> Result<(), String> {
        let required = [
            ("module", self.module.as_str()),
            ("module version", self.module_version.as_str()),
            ("actor identity", self.actor.identity.as_str()),
        ];
        for (field, value) in required {
            if value.trim().is_empty() {
                return Err(format!("complete curve ancestry is missing {field}"));
            }
        }
        if !(1..=CURVE_ANCESTRY_SCHEMA_VERSION).contains(&self.schema_version) {
            return Err(format!(
                "unsupported curve ancestry schema version {}",
                self.schema_version
            ));
        }
        if self.timestamp_utc_ms == 0 {
            return Err("complete curve ancestry is missing its timestamp".into());
        }
        for input in &self.inputs {
            for (field, value) in [
                ("input well identity", input.well_id.as_str()),
                ("input argument", input.argument.as_str()),
                ("input curve", input.curve.as_str()),
                ("input log set", input.log_set.as_str()),
                ("input set identity", input.set_id.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(format!("complete curve ancestry is missing {field}"));
                }
            }
            if input.set_version.is_some_and(|version| version < 1) {
                return Err(format!(
                    "input '{}' has an invalid log-set version",
                    input.curve
                ));
            }
            match (input.chosen_curve_id.as_deref(), input.rule.as_ref()) {
                (Some(curve_id), Some(_)) if !curve_id.trim().is_empty() => {}
                (None, None) if self.schema_version == 1 => {}
                (Some(_), None) | (None, Some(_)) => {
                    return Err(format!(
                        "input '{}' has an incomplete curve-resolution decision",
                        input.curve
                    ));
                }
                _ => {
                    return Err(format!(
                        "input '{}' has no chosen curve identity",
                        input.curve
                    ));
                }
            }
            let chosen = input.chosen_curve_id.as_deref();
            if chosen.is_none() && !input.rejected_candidates.is_empty() {
                return Err(format!(
                    "input '{}' has rejected candidates without a chosen curve identity",
                    input.curve
                ));
            }
            let mut rejected_ids = std::collections::HashSet::new();
            for candidate in &input.rejected_candidates {
                if candidate.curve_id.trim().is_empty() || candidate.log_set.trim().is_empty() {
                    return Err(format!(
                        "input '{}' has an incomplete rejected curve identity",
                        input.curve
                    ));
                }
                if candidate.set_version.is_some_and(|version| version < 1) {
                    return Err(format!(
                        "input '{}' has a rejected candidate with an invalid set version",
                        input.curve
                    ));
                }
                if chosen == Some(candidate.curve_id.as_str()) {
                    return Err(format!(
                        "input '{}' lists its chosen curve as rejected",
                        input.curve
                    ));
                }
                if !rejected_ids.insert(candidate.curve_id.as_str()) {
                    return Err(format!(
                        "input '{}' repeats a rejected curve identity",
                        input.curve
                    ));
                }
            }
        }
        for parameter in &self.parameters {
            if parameter.name.trim().is_empty() {
                return Err("complete curve ancestry contains an unnamed parameter".into());
            }
            if parameter.is_required_unset() {
                if parameter.resolution.is_some() || parameter.manifest_version.is_some() {
                    return Err(format!(
                        "parameter '{}' has provenance on a REQUIRED_UNSET state",
                        parameter.name
                    ));
                }
                continue;
            }
            if parameter.source == crate::modules::ABSENT_DEFAULT_SOURCE {
                return Err(format!(
                    "parameter '{}' has an incomplete REQUIRED_UNSET state",
                    parameter.name
                ));
            }
            if parameter.value.is_null() {
                return Err(format!(
                    "parameter '{}' has no recorded value",
                    parameter.name
                ));
            }
            if parameter.source.trim().is_empty() {
                return Err(format!(
                    "parameter '{}' has no source string",
                    parameter.name
                ));
            }
            match (parameter.resolution, parameter.manifest_version.as_deref()) {
                (Some(ParameterResolution::Explicit), Some(_)) => {
                    return Err(format!(
                        "explicit parameter '{}' names a default manifest version",
                        parameter.name
                    ));
                }
                (Some(ParameterResolution::Defaulted), Some(version))
                    if !version.trim().is_empty() => {}
                (Some(ParameterResolution::Defaulted), _) => {
                    return Err(format!(
                        "defaulted parameter '{}' has no manifest version",
                        parameter.name
                    ));
                }
                (None, Some(_)) => {
                    return Err(format!(
                        "legacy parameter '{}' names a manifest version without a resolution",
                        parameter.name
                    ));
                }
                _ => {}
            }
            if parameter
                .value
                .as_f64()
                .is_some_and(|value| !value.is_finite())
            {
                return Err(format!(
                    "parameter '{}' has a non-finite recorded value",
                    parameter.name
                ));
            }
        }
        match (self.parameters.is_empty(), self.parameter_state) {
            (true, Some(ProvenanceAbsentState::NotApplicable)) => {}
            (true, Some(ProvenanceAbsentState::LegacyUnrecorded)) if self.schema_version < 3 => {}
            (true, None) if self.schema_version < 3 => {}
            (true, Some(state)) => {
                return Err(format!(
                    "an empty parameter set has invalid provenance state {state:?}"
                ));
            }
            (true, None) => {
                return Err(
                    "a current empty parameter set must be named NOT_APPLICABLE".into(),
                );
            }
            (false, None) => {}
            (false, Some(state)) => {
                return Err(format!(
                    "a populated parameter set must not also claim absent state {state:?}"
                ));
            }
        }
        if let AncestryZoneScope::Defined(zones) = &self.zone_scope {
            if zones.is_empty() {
                return Err("defined zone ancestry contains no zone definitions".into());
            }
            for zone in zones {
                if zone.name.trim().is_empty()
                    || !zone.top.is_finite()
                    || !zone.base.is_finite()
                    || zone.top >= zone.base
                {
                    return Err(
                        "complete curve ancestry contains an invalid zone definition".into(),
                    );
                }
                if zone.source.trim().is_empty() {
                    return Err(format!("zone '{}' has no source string", zone.name));
                }
            }
        }
        if self.outputs.is_empty() {
            return Err("complete curve ancestry has no output derivations".into());
        }
        for output in &self.outputs {
            if output.curve.trim().is_empty() || output.derivation.trim().is_empty() {
                return Err(
                    "complete curve ancestry contains an incomplete output derivation".into(),
                );
            }
        }
        Ok(())
    }

    /// Whether two records describe the same deterministic computation. The timestamp
    /// identifies the event and is intentionally excluded; every scientifically material
    /// input, value/source, zone/source, actor, output, and implementation identity remains.
    pub(crate) fn same_computation(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.module == other.module
            && self.module_version == other.module_version
            && self.inputs == other.inputs
            && self.parameters == other.parameters
            && self.parameter_state == other.parameter_state
            && self.zone_scope == other.zone_scope
            && self.actor == other.actor
            && self.outputs == other.outputs
    }
}

pub(crate) fn ancestry_timestamp_utc_ms() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("cannot record curve ancestry timestamp: {error}"))?
        .as_millis()
        .try_into()
        .map_err(|_| "curve ancestry timestamp exceeds u64".to_string())
}

/// Resolves one effective input using the same precedence as `fetch_curve_frame`: the current
/// chain/run set, then an explicitly named input set, then the current computed store, then the
/// imported generic store. A standard-only legacy project is migrated through the existing
/// idempotent generic-store migration before the final lookup; no invented RAW identity is used.
pub(crate) fn try_resolve_ancestry_input(
    conn: &Connection,
    well_id: &str,
    argument: &str,
    curve: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<Option<AncestryInput>, String> {
    let upper = curve.trim().to_uppercase();
    let computed_curve_id = |set_id: &str| format!("computed:{set_id}:{upper}");
    let from_log_set = |
        set_id: &str,
        rule: CurveResolutionRule,
        rejected_candidates: Vec<RejectedCurveCandidate>,
    | -> Result<AncestryInput, String> {
        let (set_name, version): (String, i64) = conn
            .query_row(
                "SELECT set_name, version FROM log_sets WHERE set_id = ?1",
                params![set_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| {
                format!("input curve '{curve}' cites a missing log-set record: {error}")
            })?;
        Ok(AncestryInput {
            well_id: well_id.to_string(),
            argument: argument.to_string(),
            curve: upper.clone(),
            log_set: set_name,
            set_version: Some(version),
            set_id: set_id.to_string(),
            chosen_curve_id: Some(computed_curve_id(set_id)),
            rule: Some(rule),
            rejected_candidates,
        })
    };

    if let Some(set_id) = own_set_id {
        let found = conn
            .query_row(
                "SELECT 1 FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2 LIMIT 1",
                params![set_id, upper],
                |_| Ok(()),
            )
            .is_ok();
        if found {
            return from_log_set(
                set_id,
                CurveResolutionRule::WorkingInputSet,
                Vec::new(),
            )
            .map(Some);
        }
    }
    if let Some(set_name) = input_set.map(str::trim).filter(|value| !value.is_empty()) {
        let selected: Vec<(String, String, i64)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT s.set_id, s.set_name, s.version FROM log_sets s
                     WHERE s.well_id = ?1 AND upper(s.set_name) = upper(?2)
                       AND EXISTS (SELECT 1 FROM computed_curves_archive a
                                   WHERE a.set_id = s.set_id AND upper(a.curve_name) = ?3)
                     ORDER BY s.version DESC, s.set_id",
                )
                .map_err(|error| error.to_string())?;
            stmt.query_map(params![well_id, set_name, upper], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<duckdb::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?
        };
        if let Some((set_id, _, _)) = selected.first() {
            let rejected_candidates = selected[1..]
                .iter()
                .map(|(candidate_id, candidate_set, version)| RejectedCurveCandidate {
                    curve_id: computed_curve_id(candidate_id),
                    log_set: candidate_set.clone(),
                    set_version: Some(*version),
                })
                .collect();
            return from_log_set(
                set_id,
                CurveResolutionRule::WorkingInputSet,
                rejected_candidates,
            )
            .map(Some);
        }
    }

    let set_ids: Vec<Option<String>> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves
                 WHERE well_id = ?1 AND upper(curve_name) = ?2",
            )
            .map_err(|error| error.to_string())?;
        stmt.query_map(params![well_id, upper], |row| row.get(0))
            .map_err(|error| error.to_string())?
            .collect::<duckdb::Result<_>>()
            .map_err(|error| error.to_string())?
    };
    if !set_ids.is_empty() {
        if set_ids.len() != 1 || set_ids[0].is_none() {
            return Err(format!(
                "input computed curve '{curve}' has no single live ancestry record"
            ));
        }
        return from_log_set(
            set_ids[0].as_deref().expect("checked above"),
            CurveResolutionRule::ExplicitName,
            Vec::new(),
        )
        .map(Some);
    }

    let mut imported = resolve_generic_curve_decision(conn, well_id, &upper)
        .map_err(|error| error.to_string())?;
    if imported.is_none() {
        crate::db::migrate_standard_curves_to_generic_store(conn)
            .map_err(|error| error.to_string())?;
        imported = resolve_generic_curve_decision(conn, well_id, &upper)
            .map_err(|error| error.to_string())?;
    }
    let Some(GenericCurveDecision {
        chosen,
        rule,
        rejected,
    }) = imported
    else {
        return Ok(None);
    };
    let rule = rule.ok_or_else(|| {
        format!(
            "input curve '{curve}' has tied legacy candidates with no recorded modification order"
        )
    })?;
    let rejected_candidates = rejected
        .into_iter()
        .map(|candidate| RejectedCurveCandidate {
            curve_id: candidate.curve_id,
            log_set: candidate.set_name,
            set_version: Some(candidate.set_version),
        })
        .collect();
    let chosen_curve_id = chosen.curve_id.clone();
    Ok(Some(AncestryInput {
        well_id: well_id.to_string(),
        argument: argument.to_string(),
        curve: upper,
        log_set: chosen.set_name,
        set_version: Some(chosen.set_version),
        set_id: chosen_curve_id.clone(),
        chosen_curve_id: Some(chosen_curve_id),
        rule: Some(rule),
        rejected_candidates,
    }))
}

/// Strict ancestry resolution for a curve that materially participated in a computation.
/// Optional module inputs use [`try_resolve_ancestry_input`] so a declared-but-absent input is
/// not falsely recorded as project data; a present curve with malformed ancestry still errors.
pub(crate) fn resolve_ancestry_input(
    conn: &Connection,
    well_id: &str,
    argument: &str,
    curve: &str,
    input_set: Option<&str>,
    own_set_id: Option<&str>,
) -> Result<AncestryInput, String> {
    try_resolve_ancestry_input(conn, well_id, argument, curve, input_set, own_set_id)?
        .ok_or_else(|| format!("input curve '{curve}' has no resolvable log-set identity"))
}

/// A log-set specification that cannot be constructed until the complete record validates.
/// Production writers accept this type, not raw JSON strings.
#[derive(Debug, Clone)]
pub struct CompleteLogSetSpec {
    storage: LogSetSpec,
    ancestry: CurveAncestry,
    discipline: SetWriteDiscipline,
}

impl CompleteLogSetSpec {
    #[cfg(test)]
    pub fn try_new(set_name: &str, ancestry: CurveAncestry) -> Result<Self, String> {
        Self::try_new_with_legacy(
            set_name,
            ancestry,
            serde_json::Value::Object(Default::default()),
            "[]",
        )
    }

    pub fn try_new_with_legacy(
        set_name: &str,
        ancestry: CurveAncestry,
        legacy_parameters: serde_json::Value,
        legacy_inputs_json: &str,
    ) -> Result<Self, String> {
        ancestry.validate()?;
        if set_name.trim().is_empty() {
            return Err("complete curve ancestry is missing its output log-set name".into());
        }
        let mut parameters = match legacy_parameters {
            serde_json::Value::Object(map) => map,
            other => {
                let mut map = serde_json::Map::new();
                map.insert("legacy_parameters".into(), other);
                map
            }
        };
        if parameters.contains_key(CURVE_ANCESTRY_KEY) {
            return Err(format!(
                "legacy parameters may not replace reserved key '{CURVE_ANCESTRY_KEY}'"
            ));
        }
        parameters.insert(
            CURVE_ANCESTRY_KEY.into(),
            serde_json::to_value(&ancestry)
                .map_err(|error| format!("cannot serialize curve ancestry: {error}"))?,
        );
        let inputs: serde_json::Value = serde_json::from_str(legacy_inputs_json)
            .map_err(|error| format!("cannot record invalid input JSON: {error}"))?;
        let storage = LogSetSpec {
            set_name: set_name.trim().to_string(),
            module: ancestry.module.clone(),
            params_json: serde_json::Value::Object(parameters).to_string(),
            inputs_json: inputs.to_string(),
        };
        Ok(Self {
            storage,
            ancestry,
            discipline: SetWriteDiscipline::default(),
        })
    }

    pub fn ancestry(&self) -> &CurveAncestry {
        &self.ancestry
    }

    #[cfg(test)]
    fn with_sampling_style(
        mut self,
        sampling_style: crate::schema_vocab::SamplingStyle,
    ) -> Self {
        self.discipline = SetWriteDiscipline {
            sampling_style,
        };
        self
    }

    /// Attach source-comparison decisions to named parameters and refresh the already-validated
    /// serialized ancestry in the storage payload. This keeps specialized producers such as the
    /// pay-summary engine on the same whitelisted complete-record path; it does not create a second
    /// writer or any duplicate-tolerant database behavior.
    pub(crate) fn record_parameter_decisions(
        &mut self,
        topics: &[(&str, &str)],
    ) -> Result<(), String> {
        for parameter in &mut self.ancestry.parameters {
            if let Some((_, topic)) = topics
                .iter()
                .find(|(name, _)| parameter.name.eq_ignore_ascii_case(name))
            {
                parameter.decision = crate::param_sources::decision_for(topic, &parameter.value);
            }
        }
        self.ancestry.validate()?;
        let mut stored: serde_json::Value = serde_json::from_str(&self.storage.params_json)
            .map_err(|error| format!("cannot refresh curve ancestry parameter JSON: {error}"))?;
        let object = stored
            .as_object_mut()
            .ok_or_else(|| "cannot refresh curve ancestry in a non-object parameter record".to_string())?;
        object.insert(
            CURVE_ANCESTRY_KEY.into(),
            serde_json::to_value(&self.ancestry)
                .map_err(|error| format!("cannot serialize curve ancestry decision: {error}"))?,
        );
        self.storage.params_json = stored.to_string();
        Ok(())
    }

    /// Retain non-parameter run metadata in the legacy payload while naming the canonical
    /// parameter collection as genuinely not applicable. This is used by user equations: their
    /// definition is provenance, but it is not a configurable petrophysical parameter set.
    fn record_parameters_not_applicable(&mut self) -> Result<(), String> {
        self.ancestry.parameters.clear();
        self.ancestry.parameter_state = Some(ProvenanceAbsentState::NotApplicable);
        self.ancestry.validate()?;
        let mut stored: serde_json::Value = serde_json::from_str(&self.storage.params_json)
            .map_err(|error| format!("cannot name the equation parameter state: {error}"))?;
        let object = stored
            .as_object_mut()
            .ok_or_else(|| "cannot name parameters in a non-object provenance record".to_string())?;
        object.insert(
            CURVE_ANCESTRY_KEY.into(),
            serde_json::to_value(&self.ancestry)
                .map_err(|error| format!("cannot serialize the equation parameter state: {error}"))?,
        );
        self.storage.params_json = stored.to_string();
        Ok(())
    }
}

/// Builds the complete record for a run whose inputs are project curves and whose explicit
/// controls share one user-supplied source/reference note. More specialized producers (for
/// example, a photograph or deviation survey) construct [`CurveAncestry`] directly so their
/// non-curve input identity is recorded truthfully instead of being disguised as a log curve.
pub(crate) fn complete_curve_run_spec(
    conn: &Connection,
    output_well_id: &str,
    set_name: &str,
    module: &str,
    custody: &RunCustody,
    inputs: &[(String, String, String)],
    input_set: Option<&str>,
    legacy_parameters: serde_json::Value,
    zone_scope: AncestryZoneScope,
    outputs: &[String],
) -> Result<CompleteLogSetSpec, String> {
    custody.validate()?;
    if output_well_id.trim().is_empty() {
        return Err("complete curve ancestry is missing its output well identity".into());
    }
    let resolved_inputs = inputs
        .iter()
        .map(|(well_id, argument, curve)| {
            resolve_ancestry_input(conn, well_id, argument, curve, input_set, None)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let parameters = match &legacy_parameters {
        serde_json::Value::Object(values) => values
            .iter()
            .map(|(name, value)| AncestryParameter {
                name: name.clone(),
                value: if value.is_null() {
                    serde_json::json!("ABSENT")
                } else {
                    value.clone()
                },
                source: custody.source_note.trim().to_string(),
                resolution: Some(ParameterResolution::Explicit),
                manifest_version: None,
                decision: None,
            })
            .collect(),
        serde_json::Value::Null => Vec::new(),
        value => vec![AncestryParameter {
            name: "request".into(),
            value: value.clone(),
            source: custody.source_note.trim().to_string(),
            resolution: Some(ParameterResolution::Explicit),
            manifest_version: None,
            decision: None,
        }],
    };
    let parameter_state = parameter_state_for(&parameters);
    let ancestry = CurveAncestry {
        schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
        module: module.trim().to_string(),
        module_version: env!("CARGO_PKG_VERSION").to_string(),
        inputs: resolved_inputs,
        parameters,
        parameter_state,
        zone_scope,
        actor: custody.actor.clone(),
        timestamp_utc_ms: ancestry_timestamp_utc_ms()?,
        outputs: outputs
            .iter()
            .map(|curve| AncestryOutput {
                curve: curve.clone(),
                derivation: format!("{}:{}", module.trim(), curve),
            })
            .collect(),
    };
    let legacy_inputs =
        serde_json::to_string(&inputs.iter().map(|(_, _, curve)| curve).collect::<Vec<_>>())
            .map_err(|error| format!("cannot record run inputs: {error}"))?;
    CompleteLogSetSpec::try_new_with_legacy(set_name, ancestry, legacy_parameters, &legacy_inputs)
}

/// Opaque proof that a stored set has a complete, validated record. No production writer accepts
/// an arbitrary string as a set id.
#[derive(Debug, Clone)]
pub struct CompleteSetId {
    value: String,
    well_id: String,
    outputs: Vec<String>,
}

impl CompleteSetId {
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

pub(crate) struct CompleteWellLogSet {
    pub well_id: String,
    pub spec: CompleteLogSetSpec,
}

pub(crate) struct CompleteWellWrite {
    pub well_id: String,
    pub depth: Vec<f32>,
    pub curves: Vec<(String, Vec<f32>)>,
    pub set_id: CompleteSetId,
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
    pub is_current: bool
,
    pub ancestry: Option<CurveAncestry>,
}

/// Registers a new run event: version = 1 + the well's highest version of `set_name`
/// (so a re-run NEVER replaces — it becomes version N+1). Returns (set_id, version).
fn create_log_set_raw(
    conn: &Connection,
    well_id: &str,
    spec: &LogSetSpec,
    discipline: SetWriteDiscipline,
) -> duckdb::Result<(String, i64)> {
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
        params![well_id, spec.set_name],
        |r| r.get(0),
    )?;
    let set_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO log_sets
            (set_id, well_id, set_name, version, module, params_json, inputs_json, frame,
             sampling_style, duplicate_resolution)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            set_id,
            well_id,
            spec.set_name,
            version,
            spec.module,
            spec.params_json,
            spec.inputs_json,
            crate::schema_vocab::LogSetFrame::Standard.as_str(),
            discipline.sampling_style.as_str(),
            crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str()
        ],
    )?;
    Ok((set_id, version))
}

fn write_run_parameters(
    conn: &Connection,
    set_id: &str,
    parameters: &[AncestryParameter],
) -> duckdb::Result<()> {
    for (position, parameter) in parameters.iter().enumerate() {
        let (value_json, source, state): (Option<String>, Option<&str>, Option<&str>) =
            if parameter.is_required_unset() {
                (None, None, Some(REQUIRED_UNSET_PARAMETER_STATE))
            } else {
                (Some(parameter.value.to_string()), Some(parameter.source.as_str()), None)
            };
        conn.execute(
            "INSERT INTO run_parameters
                (set_id, position, name, value_json, source, state, resolution, manifest_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                set_id,
                position as i64,
                parameter.name,
                value_json,
                source,
                state,
                parameter.resolution.map(ParameterResolution::as_str),
                parameter.manifest_version
            ],
        )?;
    }
    Ok(())
}

/// Legacy test-fixture entry point. Production code is inventoried by SB-CORE-T14 and must use
/// [`create_complete_log_set`] so it cannot obtain a writable set id from partial JSON.
#[cfg(test)]
pub(crate) fn create_log_set(
    conn: &Connection,
    well_id: &str,
    spec: &LogSetSpec,
) -> duckdb::Result<(String, i64)> {
    create_log_set_raw(conn, well_id, spec, SetWriteDiscipline::default())
}

pub(crate) fn create_complete_log_set(
    conn: &Connection,
    well_id: &str,
    spec: &CompleteLogSetSpec,
) -> Result<(CompleteSetId, i64), String> {
    spec.ancestry.validate()?;
    validate_set_write_discipline(spec.discipline)?;
    let (value, version) = crate::db::with_txn(conn, |conn| {
        let created = create_log_set_raw(conn, well_id, &spec.storage, spec.discipline)?;
        write_run_parameters(conn, &created.0, &spec.ancestry.parameters)?;
        Ok::<_, duckdb::Error>(created)
    })
    .map_err(|error| error.to_string())?;
    Ok((
        CompleteSetId {
            value,
            well_id: well_id.to_string(),
            outputs: spec
                .ancestry
                .outputs
                .iter()
                .map(|output| output.curve.to_uppercase())
                .collect(),
        },
        version,
    ))
}

/// Versioned batch write: refreshes the CURRENT store (same delete-then-append discipline
/// as `write_computed_curves_batch`, rows tagged with `set_id`) and appends the identical
/// rows to the append-only archive. Prior versions' archive rows are untouched — that is
/// the "never overwrite" guarantee; any version can be restored via `restore_log_set`.
fn load_set_write_discipline(
    conn: &Connection,
    set_id: &str,
) -> Result<SetWriteDiscipline, String> {
    let stored: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT sampling_style, duplicate_resolution
             FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| format!("log-set write discipline is not live: {error}"))?;
    let sampling_style = stored
        .0
        .as_deref()
        .and_then(crate::schema_vocab::SamplingStyle::parse)
        .ok_or_else(|| {
            "log-set write refused: sampling style is legacy-unrecorded or invalid".to_string()
        })?;
    let duplicate_resolution = stored
        .1
        .as_deref()
        .and_then(crate::schema_vocab::DuplicateDepthResolution::parse)
        .ok_or_else(|| {
            "log-set write refused: duplicate-depth resolution is legacy-unrecorded or invalid"
                .to_string()
        })?;
    if duplicate_resolution != crate::schema_vocab::DuplicateDepthResolution::Refuse {
        return Err("continuous log sets must declare duplicate-depth resolution REFUSE".into());
    }
    let discipline = SetWriteDiscipline { sampling_style };
    validate_set_write_discipline(discipline)?;
    Ok(discipline)
}

fn validate_set_write_discipline(discipline: SetWriteDiscipline) -> Result<(), String> {
    match discipline.sampling_style {
        crate::schema_vocab::SamplingStyle::ContinuousRegular
        | crate::schema_vocab::SamplingStyle::ContinuousIrregular => Ok(()),
        crate::schema_vocab::SamplingStyle::Point => Err(
            "POINT data must use the point-delivery store, which declares and logs its resolution"
                .into(),
        ),
    }
}

fn depth_identity(depth: f32) -> u32 {
    if depth == 0.0 {
        0.0_f32.to_bits()
    } else {
        depth.to_bits()
    }
}

fn validate_continuous_depth_uniqueness(
    depth: &[f32],
    curves: &[(&str, &[f32])],
) -> Result<(), String> {
    for (curve, values) in curves {
        let mut first_rows = HashMap::<u32, usize>::new();
        for (index, value) in depth.iter().take(values.len()).enumerate() {
            let key = depth_identity(*value);
            if let Some(first) = first_rows.insert(key, index) {
                return Err(format!(
                    "continuous depth uniqueness refused for curve '{curve}' at depth {value}: source rows {} and {} share one depth",
                    first + 1,
                    index + 1
                ));
            }
        }
    }
    Ok(())
}

fn validate_archived_continuous_depth_uniqueness(
    conn: &Connection,
    set_id: &str,
) -> Result<(), String> {
    let mut statement = conn
        .prepare(
            "SELECT curve_name, depth,
                    row_number() OVER (
                        PARTITION BY upper(curve_name) ORDER BY rowid
                    ) AS source_row
             FROM computed_curves_archive
             WHERE set_id = ?1
             ORDER BY upper(curve_name), source_row",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![set_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f32>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    let mut first_rows = HashMap::<(String, u32), i64>::new();
    for row in rows {
        let (curve, depth, source_row) = row.map_err(|error| error.to_string())?;
        let curve_key = curve.to_ascii_uppercase();
        let key = (curve_key, depth_identity(depth));
        if let Some(first) = first_rows.insert(key, source_row) {
            return Err(format!(
                "continuous depth uniqueness refused for curve '{curve}' at depth {depth}: source rows {first} and {source_row} share one depth"
            ));
        }
    }
    Ok(())
}

fn write_versioned_rows_raw(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &str,
) -> Result<(), String> {
    if curves.is_empty() {
        return Ok(());
    }
    load_set_write_discipline(conn, set_id)?;
    validate_continuous_depth_uniqueness(depth, curves)?;
    // Atomic: DELETE current + append current + append archive must land as one unit, so a
    // crash can't strand the DELETE with the current-store append lost.
    crate::db::with_txn(conn, |conn| {
        let placeholders = std::iter::repeat("?").take(curves.len()).collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM computed_curves WHERE well_id = ? AND upper(curve_name) IN ({placeholders})");
        // Bind UPPERCASED names so a re-cased write reclaims any prior-casing rows: every reader
        // resolves curve_name case-insensitively via upper(), but an exact-case DELETE would leave
        // a stale shadow row (e.g. old 'phie' after a rewrite to 'PHIE') that can silently win.
        let mut del_params: Vec<String> = Vec::with_capacity(curves.len() + 1);
        del_params.push(well_id.to_string());
        for (name, _) in curves {
            del_params.push(name.to_uppercase());
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
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

/// Legacy test-fixture entry point. A production caller would be able to pair arbitrary rows with
/// arbitrary partial metadata, so SB-CORE-T14 forbids calls outside test code.
#[cfg(test)]
pub(crate) fn write_computed_curves_versioned(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &str,
) -> Result<(), String> {
    write_versioned_rows_raw(conn, well_id, depth, curves, set_id)
}

pub(crate) fn write_computed_curves_with_ancestry(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    set_id: &CompleteSetId,
) -> Result<(), String> {
    if set_id.well_id != well_id {
        return Err("complete ancestry set belongs to a different well".into());
    }
    for (name, _) in curves {
        if !set_id
            .outputs
            .iter()
            .any(|output| output.eq_ignore_ascii_case(name))
        {
            return Err(format!(
                "computed curve '{name}' has no output derivation in its ancestry record"
            ));
        }
    }
    let stored: Option<String> = conn
        .query_row(
            "SELECT params_json FROM log_sets WHERE set_id = ?1 AND well_id = ?2",
            params![set_id.value, well_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("complete ancestry set is not live: {error}"))?;
    parse_curve_ancestry(stored.as_deref().unwrap_or_default())?;
    write_versioned_rows_raw(conn, well_id, depth, curves, &set_id.value)
}

/// Complete write that also retires a declared family of stale current curves in the same
/// transaction. Monte Carlo uses this when a later run stops producing one previously persisted
/// key; the archive remains append-only and no duplicate-tolerant/upsert path is introduced.
pub(crate) fn write_computed_curves_with_ancestry_clearing(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
    clear_names: &[String],
    set_id: &CompleteSetId,
) -> Result<(), String> {
    if set_id.well_id != well_id {
        return Err("complete ancestry set belongs to a different well".into());
    }
    for (name, _) in curves {
        if !set_id
            .outputs
            .iter()
            .any(|output| output.eq_ignore_ascii_case(name))
        {
            return Err(format!(
                "computed curve '{name}' has no output derivation in its ancestry record"
            ));
        }
    }
    let stored: Option<String> = conn
        .query_row(
            "SELECT params_json FROM log_sets WHERE set_id = ?1 AND well_id = ?2",
            params![set_id.value, well_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("complete ancestry set is not live: {error}"))?;
    parse_curve_ancestry(stored.as_deref().unwrap_or_default())?;
    load_set_write_discipline(conn, &set_id.value)?;
    validate_continuous_depth_uniqueness(depth, curves)?;
    crate::db::with_txn(conn, |conn| {
        if !clear_names.is_empty() {
            let placeholders = std::iter::repeat("?").take(clear_names.len()).collect::<Vec<_>>().join(", ");
            let sql = format!(
                "DELETE FROM computed_curves WHERE well_id = ? AND upper(curve_name) IN ({placeholders})"
            );
            let mut values = Vec::with_capacity(clear_names.len() + 1);
            values.push(well_id.to_string());
            values.extend(clear_names.iter().map(|name| name.to_uppercase()));
            conn.execute(&sql, params_from_iter(values))?;
        }
        let mut current = conn.appender("computed_curves")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                current.append_row(params![well_id, d, name, value, set_id.value])?;
            }
        }
        current.flush()?;
        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                archive.append_row(params![set_id.value, well_id, d, name, value])?;
            }
        }
        archive.flush()?;
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

/// Writes a complete version whose archive carries an independent depth frame. Reframe sets must
/// not enter `computed_curves`: that table is aligned to the well's live frame, so doing so would
/// replace a readable interpretation with rows that no current-frame reader can align.
pub(crate) fn write_complete_own_frame(
    conn: &Connection,
    well_id: &str,
    spec: &CompleteLogSetSpec,
    depth: &[f32],
    curves: &[(String, Vec<f32>)],
) -> Result<i64, String> {
    spec.ancestry.validate()?;
    for (curve, _) in curves {
        if !spec
            .ancestry
            .outputs
            .iter()
            .any(|output| output.curve.eq_ignore_ascii_case(curve))
        {
            return Err(format!(
                "computed curve '{curve}' has no output derivation in its ancestry record"
            ));
        }
    }
    validate_set_write_discipline(spec.discipline)?;
    let continuous_curves = curves
        .iter()
        .map(|(name, values)| (name.as_str(), values.as_slice()))
        .collect::<Vec<_>>();
    validate_continuous_depth_uniqueness(depth, &continuous_curves)?;
    crate::db::with_txn(conn, |conn| {
        let (set_id, version) =
            create_log_set_raw(conn, well_id, &spec.storage, spec.discipline)?;
        conn.execute(
            "UPDATE log_sets SET frame = ?2 WHERE set_id = ?1",
            params![set_id, crate::schema_vocab::LogSetFrame::Own.as_str()],
        )?;
        let mut archive = conn.appender("computed_curves_archive")?;
        for (name, values) in curves {
            for (d, value) in depth.iter().zip(values.iter()) {
                archive.append_row(params![set_id, well_id, d, name, value])?;
            }
        }
        archive.flush()?;
        Ok::<i64, duckdb::Error>(version)
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn parse_curve_ancestry(params_json: &str) -> Result<CurveAncestry, String> {
    let parameters: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|error| format!("curve ancestry parameter JSON is invalid: {error}"))?;
    let record = parameters
        .get(CURVE_ANCESTRY_KEY)
        .ok_or_else(|| "computed curve has no complete ancestry record".to_string())?;
    let mut ancestry: CurveAncestry = serde_json::from_value(record.clone())
        .map_err(|error| format!("curve ancestry record is invalid: {error}"))?;
    if ancestry.schema_version < 3
        && ancestry.parameters.is_empty()
        && ancestry.parameter_state.is_none()
    {
        ancestry.parameter_state = Some(ProvenanceAbsentState::LegacyUnrecorded);
    }
    ancestry.validate()?;
    Ok(ancestry)
}

pub(crate) const LEGACY_UNRECORDED: &str = ProvenanceAbsentState::LegacyUnrecorded.as_str();

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputedProvenanceClass {
    Recorded,
    LegacyUnrecorded,
}

#[derive(Debug, Clone)]
pub(crate) struct ComputedProvenanceGroup {
    pub curve_name: String,
    pub set_id: Option<String>,
    pub provenance_class: ComputedProvenanceClass,
    pub row_count: i64,
}

/// Classifies every live computed row by its actual join to `log_sets`. A non-NULL UUID whose
/// target record is missing is no more provenanced than a NULL UUID, so both enter the explicit
/// legacy class. Grouping by set identity preserves the one-hop record for every valid row while
/// retaining an exact count for every unrecorded group.
pub(crate) fn computed_provenance_groups(
    conn: &Connection,
    well_id: &str,
) -> Result<Vec<ComputedProvenanceGroup>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT cc.curve_name, CAST(s.set_id AS VARCHAR),
                    s.set_id IS NOT NULL,
                    COUNT(*)
             FROM computed_curves cc
             LEFT JOIN log_sets s ON s.set_id = cc.set_id
             WHERE cc.well_id = ?1
             GROUP BY cc.curve_name, s.set_id
             ORDER BY upper(cc.curve_name), s.set_id NULLS FIRST",
        )
        .map_err(|error| error.to_string())?;
    stmt.query_map(params![well_id], |row| {
        let recorded = row.get::<_, bool>(2)?;
        Ok(ComputedProvenanceGroup {
            curve_name: row.get(0)?,
            set_id: if recorded { row.get(1)? } else { None },
            provenance_class: if recorded {
                ComputedProvenanceClass::Recorded
            } else {
                ComputedProvenanceClass::LegacyUnrecorded
            },
            row_count: row.get(3)?,
        })
    })
    .map_err(|error| error.to_string())?
    .collect::<duckdb::Result<Vec<_>>>()
    .map_err(|error| error.to_string())
}

/// Resolves the one live record attached to a computed curve. Multiple or NULL set identities are
/// refused rather than selecting whichever row DuckDB happens to return first.
pub(crate) fn curve_ancestry(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
) -> Result<CurveAncestry, String> {
    let groups = computed_provenance_groups(conn, well_id)?
        .into_iter()
        .filter(|group| group.curve_name.eq_ignore_ascii_case(curve_name))
        .collect::<Vec<_>>();
    if groups.len() != 1 || groups[0].provenance_class != ComputedProvenanceClass::Recorded {
        return Err(format!(
            "computed curve '{curve_name}' has no single live ancestry record"
        ));
    }
    let set_id = groups[0].set_id.as_deref().expect("recorded groups carry a set id");
    let params_json: Option<String> = conn
        .query_row(
            "SELECT params_json FROM log_sets WHERE set_id = ?1",
            params![set_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            format!("computed curve '{curve_name}' cites a missing ancestry record: {error}")
        })?;
    parse_curve_ancestry(params_json.as_deref().unwrap_or_default())
}

/// One complete, human-readable ancestry record ready for a catalog or number-carrying
/// deliverable. `set_id` is retained internally by the query but deliberately not exposed here:
/// downstream files need the stable set name/version plus the complete input identities, not an
/// opaque database implementation detail as their only explanation.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CurveAncestryDisclosure {
    pub well_id: String,
    pub curve_name: String,
    pub provenance_class: ComputedProvenanceClass,
    pub provenance_row_count: i64,
    pub set_name: Option<String>,
    pub version: Option<i64>,
    pub ancestry: Option<CurveAncestry>,
}

impl CurveAncestryDisclosure {
    /// Full disclosure columns shared by PDF, Word, workbook and deck surfaces. No field is
    /// summarized away: an input's well/set identity, a value's source, and a zone's source all
    /// remain in the exported text.
    pub(crate) fn cells(&self) -> [String; 7] {
        let Some(ancestry) = self.ancestry.as_ref() else {
            let label = format!(
                "{} / {} ({} rows)",
                self.curve_name, LEGACY_UNRECORDED, self.provenance_row_count
            );
            return [
                label,
                LEGACY_UNRECORDED.into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
                "UNAVAILABLE — LEGACY_UNRECORDED".into(),
            ];
        };
        let inputs = ancestry
            .inputs
            .iter()
            .map(|input| {
                format!(
                    "{}={} [well {}; set {}{}; id {}]",
                    input.argument,
                    input.curve,
                    input.well_id,
                    input.log_set,
                    input
                        .set_version
                        .map(|version| format!(" v{version}"))
                        .unwrap_or_default(),
                    input.set_id,
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let parameters = ancestry
            .parameters
            .iter()
            .map(|parameter| {
                let base = format!(
                    "{}={} [source: {}]",
                    parameter.name, parameter.value, parameter.source
                );
                parameter
                    .decision
                    .as_ref()
                    .map(|decision| format!("{base} [decision: {}]", decision.disclosure()))
                    .unwrap_or(base)
            })
            .collect::<Vec<_>>()
            .join("; ");
        let zones = match &ancestry.zone_scope {
            AncestryZoneScope::WholeWell => "WHOLE WELL".to_string(),
            AncestryZoneScope::Defined(zones) => zones
                .iter()
                .map(|zone| {
                    format!(
                        "{} {}-{} [source: {}]",
                        zone.name, zone.top, zone.base, zone.source
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
        };
        let actor_kind = match ancestry.actor.kind {
            AncestryActorKind::Human => "HUMAN",
            AncestryActorKind::Automated => "AUTOMATED",
        };
        let custody = format!(
            "{} {} at {} UTC-ms",
            actor_kind, ancestry.actor.identity, ancestry.timestamp_utc_ms
        );
        let derivation = ancestry
            .outputs
            .iter()
            .find(|output| output.curve.eq_ignore_ascii_case(&self.curve_name))
            .map(|output| output.derivation.clone())
            .unwrap_or_else(|| {
                ancestry.outputs
                    .iter()
                    .map(|output| format!("{}={}", output.curve, output.derivation))
                    .collect::<Vec<_>>()
                    .join("; ")
            });
        [
            format!(
                "{} / {} v{}",
                self.curve_name,
                self.set_name.as_deref().expect("recorded disclosure has a set name"),
                self.version.expect("recorded disclosure has a version")
            ),
            format!(
                "{} @ {}",
                ancestry.module, ancestry.module_version
            ),
            if inputs.is_empty() {
                "NO CURVE INPUTS".into()
            } else {
                inputs
            },
            if parameters.is_empty() {
                "NO EXPLICIT PARAMETERS".into()
            } else {
                parameters
            },
            zones,
            custody,
            derivation,
        ]
    }
}

fn push_recorded_disclosures(
    disclosures: &mut Vec<CurveAncestryDisclosure>,
    seen: &mut std::collections::BTreeSet<(String, String, String)>,
    well_id: &str,
    set_id: String,
    set_name: String,
    version: i64,
    params_json: Option<String>,
    curves: Vec<(String, i64)>,
) -> Result<(), String> {
    let ancestry = parse_curve_ancestry(params_json.as_deref().unwrap_or_default()).map_err(|error| {
        format!(
            "computed set '{set_name}' v{version} for well '{well_id}' cannot travel into a deliverable: {error}"
        )
    })?;
    for (curve_name, row_count) in curves {
        let key = (
            well_id.to_string(),
            curve_name.to_uppercase(),
            set_id.clone(),
        );
        if !seen.insert(key) {
            continue;
        }
        if !ancestry
            .outputs
            .iter()
            .any(|output| output.curve.eq_ignore_ascii_case(&curve_name))
        {
            return Err(format!(
                "computed curve '{curve_name}' is absent from its set ancestry output derivations"
            ));
        }
        disclosures.push(CurveAncestryDisclosure {
            well_id: well_id.to_string(),
            curve_name,
            provenance_class: ComputedProvenanceClass::Recorded,
            provenance_row_count: row_count,
            set_name: Some(set_name.clone()),
            version: Some(version),
            ancestry: Some(ancestry.clone()),
        });
    }
    Ok(())
}

/// Returns an explicit provenance disclosure for every current computed row in `well_ids`. When a
/// deliverable names an input set, its latest version is included too because those archived values
/// may replace current values while rendering. Rows with no resolvable run record remain visible as
/// `LEGACY_UNRECORDED` with an exact count; no ancestry is inferred for them.
pub(crate) fn curve_ancestry_disclosures(
    conn: &Connection,
    well_ids: &[String],
    input_set: Option<&str>,
) -> Result<Vec<CurveAncestryDisclosure>, String> {
    let mut disclosures = Vec::new();
    let mut seen = std::collections::BTreeSet::<(String, String, String)>::new();

    for well_id in well_ids {
        for group in computed_provenance_groups(conn, well_id)? {
            if group.provenance_class == ComputedProvenanceClass::LegacyUnrecorded {
                let key = (
                    well_id.clone(),
                    group.curve_name.to_uppercase(),
                    LEGACY_UNRECORDED.to_string(),
                );
                if seen.insert(key) {
                    disclosures.push(CurveAncestryDisclosure {
                        well_id: well_id.clone(),
                        curve_name: group.curve_name,
                        provenance_class: ComputedProvenanceClass::LegacyUnrecorded,
                        provenance_row_count: group.row_count,
                        set_name: None,
                        version: None,
                        ancestry: None,
                    });
                }
                continue;
            }
            let set_id = group.set_id.expect("recorded groups carry a set id");
            let (set_name, version, params_json): (String, i64, Option<String>) = conn
                .query_row(
                    "SELECT set_name, version, params_json FROM log_sets WHERE set_id = ?1",
                    params![set_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| error.to_string())?;
            push_recorded_disclosures(
                &mut disclosures,
                &mut seen,
                well_id,
                set_id,
                set_name,
                version,
                params_json,
                vec![(group.curve_name, group.row_count)],
            )?;
        }

        if let Some(input_set) = input_set.map(str::trim).filter(|value| !value.is_empty()) {
            let selected: Option<(String, String, i64, Option<String>)> = conn
                .query_row(
                    "SELECT set_id, set_name, version, params_json FROM log_sets
                     WHERE well_id = ?1 AND upper(set_name) = upper(?2)
                     ORDER BY version DESC LIMIT 1",
                    params![well_id, input_set],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .optional()
                .map_err(|error| error.to_string())?;
            if let Some((set_id, set_name, version, params_json)) = selected {
                let mut stmt = conn
                    .prepare(
                        "SELECT curve_name, COUNT(*) FROM computed_curves_archive
                         WHERE set_id = ?1 GROUP BY curve_name ORDER BY curve_name",
                    )
                    .map_err(|error| error.to_string())?;
                let curves = stmt
                    .query_map(params![set_id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                    })
                    .map_err(|error| error.to_string())?
                    .collect::<duckdb::Result<Vec<_>>>()
                    .map_err(|error| error.to_string())?;
                push_recorded_disclosures(
                    &mut disclosures,
                    &mut seen,
                    well_id,
                    set_id,
                    set_name,
                    version,
                    params_json,
                    curves,
                )?;
            }
        }
    }
    disclosures.sort_by(|a, b| {
        (&a.well_id, &a.curve_name, &a.set_name, &a.version).cmp(&(
            &b.well_id,
            &b.curve_name,
            &b.set_name,
            &b.version,
        ))
    });
    Ok(disclosures)
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
#[cfg(test)]
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
                "INSERT INTO log_sets
                    (set_id, well_id, set_name, version, module, params_json, inputs_json, frame,
                     sampling_style, duplicate_resolution)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    set_id,
                    well_id,
                    spec.set_name,
                    version,
                    spec.module,
                    spec.params_json,
                    spec.inputs_json,
                    crate::schema_vocab::LogSetFrame::Standard.as_str(),
                    SetWriteDiscipline::default().sampling_style.as_str(),
                    crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str()
                ],
            )?;
        }
        Ok::<(), duckdb::Error>(())
    })?;
    Ok(planned.into_iter().map(|(w, _, s)| (w, s)).collect())
}

/// A workflow chain can cite an output from an earlier step in the same not-yet-registered set.
/// Its final stored identity must be exact, but it must also survive a deterministic replay of the
/// same well, set name and version. A UUIDv8-shaped SHA-256 digest supplies that internal key; an
/// ordinary module set without a self-reference keeps the existing random UUID allocation.
fn deterministic_chain_set_id(well_id: &str, set_name: &str, version: i64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(well_id.as_bytes());
    hasher.update([0]);
    hasher.update(set_name.as_bytes());
    hasher.update([0]);
    hasher.update(version.to_le_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes).to_string()
}

/// Per-well complete batch registration. Unlike the legacy batch API, every well carries its own
/// zone/input resolution snapshot while the inserts still share one transaction.
pub(crate) fn create_complete_log_sets_batch(
    conn: &Connection,
    wells: &[CompleteWellLogSet],
) -> Result<HashMap<String, CompleteSetId>, String> {
    let mut planned: Vec<(String, i64, String, CompleteLogSetSpec)> =
        Vec::with_capacity(wells.len());
    for well in wells {
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) + 1 FROM log_sets WHERE well_id = ?1 AND set_name = ?2",
                params![well.well_id, well.spec.storage.set_name],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        let mut spec = well.spec.clone();
        let had_self_inputs = spec
            .ancestry
            .inputs
            .iter()
            .any(|input| input.set_id == "SELF");
        let set_id = if had_self_inputs {
            deterministic_chain_set_id(&well.well_id, &spec.storage.set_name, version)
        } else {
            Uuid::new_v4().to_string()
        };
        if had_self_inputs {
            for input in spec
                .ancestry
                .inputs
                .iter_mut()
                .filter(|input| input.set_id == "SELF")
            {
                input.log_set = spec.storage.set_name.clone();
                input.set_version = Some(version);
                input.set_id = set_id.clone();
                input.chosen_curve_id = Some(format!("computed:{set_id}:{}", input.curve));
            }
            let mut params: serde_json::Value = serde_json::from_str(&spec.storage.params_json)
                .map_err(|error| format!("cannot bind chain input identities: {error}"))?;
            let object = params.as_object_mut().ok_or_else(|| {
                "cannot bind chain input identities in a non-object parameter record".to_string()
            })?;
            object.insert(
                CURVE_ANCESTRY_KEY.into(),
                serde_json::to_value(&spec.ancestry)
                    .map_err(|error| format!("cannot bind chain ancestry: {error}"))?,
            );
            spec.storage.params_json = params.to_string();
            // Workflow-chain legacy input JSON is the same AncestryInput array. Keep it aligned
            // with the complete record rather than persisting the planning-only SELF marker.
            spec.storage.inputs_json = serde_json::to_string(&spec.ancestry.inputs)
                .map_err(|error| format!("cannot bind chain legacy inputs: {error}"))?;
        }
        spec.ancestry.validate()?;
        validate_set_write_discipline(spec.discipline)?;
        planned.push((
            well.well_id.clone(),
            version,
            set_id,
            spec,
        ));
    }
    crate::db::with_txn(conn, |conn| {
        for (well_id, version, set_id, spec) in &planned {
            conn.execute(
                "INSERT INTO log_sets
                    (set_id, well_id, set_name, version, module, params_json, inputs_json, frame,
                     sampling_style, duplicate_resolution)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    set_id,
                    well_id,
                    spec.storage.set_name,
                    version,
                    spec.storage.module,
                    spec.storage.params_json,
                    spec.storage.inputs_json,
                    crate::schema_vocab::LogSetFrame::Standard.as_str(),
                    spec.discipline.sampling_style.as_str(),
                    crate::schema_vocab::DuplicateDepthResolution::Refuse.as_str()
                ],
            )?;
            write_run_parameters(conn, set_id, &spec.ancestry.parameters)?;
        }
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())?;
    Ok(planned
        .into_iter()
        .map(|(well_id, _, value, spec)| {
            let outputs = spec
                .ancestry
                .outputs
                .iter()
                .map(|output| output.curve.to_uppercase())
                .collect();
            (
                well_id.clone(),
                CompleteSetId {
                    value,
                    well_id,
                    outputs,
                },
            )
        })
        .collect())
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
fn write_versioned_rows_batch_raw(conn: &Connection, wells: &[WellWrite]) -> Result<(), String> {
    if wells.iter().all(|w| w.curves.is_empty()) {
        return Ok(());
    }
    for well in wells {
        load_set_write_discipline(conn, &well.set_id)?;
        let curves = well
            .curves
            .iter()
            .map(|(name, values)| (name.as_str(), values.as_slice()))
            .collect::<Vec<_>>();
        validate_continuous_depth_uniqueness(&well.depth, &curves)?;
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
                format!("DELETE FROM computed_curves WHERE well_id IN ({wph}) AND upper(curve_name) IN ({cph})");
            // Uppercase curve names (not well_ids) so a re-cased write reclaims prior-casing rows.
            let mut p: Vec<String> = Vec::with_capacity(well_ids.len() + curves.len());
            p.extend(well_ids.iter().map(|w| w.to_string()));
            p.extend(curves.iter().map(|c| c.to_uppercase()));
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
        Ok::<(), duckdb::Error>(())
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) fn write_computed_curves_versioned_batch(
    conn: &Connection,
    wells: &[WellWrite],
) -> Result<(), String> {
    write_versioned_rows_batch_raw(conn, wells)
}

pub(crate) fn write_computed_curves_with_ancestry_batch(
    conn: &Connection,
    wells: &[CompleteWellWrite],
) -> Result<(), String> {
    let mut raw = Vec::with_capacity(wells.len());
    for well in wells {
        if well.set_id.well_id != well.well_id {
            return Err("complete ancestry set belongs to a different well".into());
        }
        for (curve, _) in &well.curves {
            if !well
                .set_id
                .outputs
                .iter()
                .any(|output| output.eq_ignore_ascii_case(curve))
            {
                return Err(format!(
                    "computed curve '{curve}' has no output derivation in its ancestry record"
                ));
            }
        }
        let params_json: Option<String> = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE set_id = ?1 AND well_id = ?2",
                params![well.set_id.value, well.well_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("complete ancestry set is not live: {error}"))?;
        parse_curve_ancestry(params_json.as_deref().unwrap_or_default())?;
        raw.push(WellWrite {
            well_id: well.well_id.clone(),
            depth: well.depth.clone(),
            curves: well.curves.clone(),
            set_id: well.set_id.value.clone(),
        });
    }
    write_versioned_rows_batch_raw(conn, &raw)
}

fn fetch_verified_import_set_frame(
    conn: &Connection,
    well_id: &str,
    set_name: &str,
    curve_names: &[String],
    source_depth: &[f32],
    source_columns: &HashMap<String, Vec<f32>>,
) -> duckdb::Result<Option<CurveFrame>> {
    let has_curves = conn
        .query_row(
            "SELECT 1 FROM curve_meta
             WHERE well_id = ?1 AND upper(set_name) = upper(?2) LIMIT 1",
            params![well_id, set_name],
            |_| Ok(()),
        )
        .is_ok();
    if !has_curves {
        return Ok(None);
    }
    let verdict: Option<(bool, String)> = conn
        .query_row(
            "SELECT sampling_verified, effective_sampling_style FROM import_sets
             WHERE well_id = ?1 AND upper(set_name) = upper(?2)",
            params![well_id, set_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    let Some((verified, effective)) = verdict else {
        return Err(duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': sampling style has not been verified"
        )));
    };
    if !verified {
        return Err(duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': sampling style has not been verified"
        )));
    }
    let style = crate::schema_vocab::SamplingStyle::parse(&effective).ok_or_else(|| {
        duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': stored sampling verdict '{effective}' is invalid"
        ))
    })?;
    if style == crate::schema_vocab::SamplingStyle::Point {
        return Err(duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': POINT data has no continuous frame"
        )));
    }

    // Always use the verified reference samples themselves, including for a declaration that was
    // contradicted to IRREGULAR. Synthesising `top + row * STEP` is the exact 6.1 m silent shift
    // SB-DBM-T27 exists to prevent.
    let mut depth_statement = conn.prepare(
        "SELECT DISTINCT s.depth
         FROM curve_samples s JOIN curve_meta m ON m.curve_id = s.curve_id
         WHERE m.well_id = ?1 AND upper(m.set_name) = upper(?2)
         ORDER BY s.depth",
    )?;
    let import_depth: Vec<f32> = depth_statement
        .query_map(params![well_id, set_name], |row| row.get(0))?
        .collect::<duckdb::Result<_>>()?;
    let mut columns: HashMap<String, Vec<f32>> = source_columns
        .iter()
        .map(|(name, values)| {
            (
                name.clone(),
                crate::reframe::resample_onto(
                    source_depth,
                    values,
                    &import_depth,
                    crate::reframe::Method::Auto,
                ),
            )
        })
        .collect();
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        let curve_id: Option<String> = conn
            .query_row(
                "SELECT curve_id FROM curve_meta
                 WHERE well_id = ?1 AND upper(set_name) = upper(?2) AND upper(mnemonic) = ?3
                 ORDER BY modified_seq DESC LIMIT 1",
                params![well_id, set_name, upper],
                |row| row.get(0),
            )
            .ok();
        let Some(curve_id) = curve_id else { continue };
        let mut sample_statement =
            conn.prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1")?;
        let rows = sample_statement.query_map(params![curve_id], |row| {
            Ok((row.get::<_, f32>(0)?, row.get::<_, f32>(1)?))
        })?;
        let mut by_depth = HashMap::new();
        for row in rows {
            let (depth, value) = row?;
            by_depth.insert(depth.to_bits(), value);
        }
        columns.insert(
            upper,
            import_depth
                .iter()
                .map(|depth| by_depth.get(&depth.to_bits()).copied().unwrap_or(f32::NAN))
                .collect(),
        );
    }
    Ok(Some((import_depth, columns)))
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
    let (mut depth, mut columns) = fetch_curve_frame(conn, well_id, curve_names)?;
    let Some(set_name) = input_set.map(str::trim).filter(|s| !s.is_empty()) else {
        overlay_resolved_native_standard_inputs(
            conn,
            well_id,
            curve_names,
            &depth,
            &mut columns,
            &Default::default(),
        )?;
        return Ok((depth, columns));
    };

    let computed_set_exists = conn
        .query_row(
            "SELECT 1 FROM log_sets
             WHERE well_id = ?1 AND upper(set_name) = upper(?2) LIMIT 1",
            params![well_id, set_name],
            |_| Ok(()),
        )
        .is_ok();
    if !computed_set_exists {
        if let Some(import_frame) = fetch_verified_import_set_frame(
            conn,
            well_id,
            set_name,
            curve_names,
            &depth,
            &columns,
        )? {
            return Ok(import_frame);
        }
    }

    // A set that carries its OWN depth frame (`log_sets.frame = 'OWN'`, written by
    // `reframe.rs`) REPLACES the run frame, and every curve resolved from elsewhere is
    // resampled onto it.
    //
    // This is the whole point of re-framing. Without it a 0.5 m set on a 0.1523 m well would be
    // read by the exact-depth join below and come back almost entirely MISSING — the same silent
    // shape as the `CPHOTO_*` trace saved at the photograph's own sampling. And the standard
    // curves have to come along, or a run against the re-framed set would read PHIE at 0.5 m and
    // GR at 0.1523 m and quietly pair samples from different rock.
    //
    // Through `reframe::resample_onto`, the same code the tool itself uses, so what the user saw
    // when they built the set and what a module reads out of it cannot differ.
    if let Some(own) = crate::reframe::set_frame(conn, well_id, set_name)? {
        for values in columns.values_mut() {
            *values = crate::reframe::resample_onto(&depth, values, &own, crate::reframe::Method::Auto);
        }
        depth = own;
    }
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
        overlay_resolved_native_standard_inputs(
            conn,
            well_id,
            curve_names,
            &depth,
            &mut columns,
            &Default::default(),
        )?;
        return Ok((depth, columns));
    };

    let mut stmt = conn.prepare(
        "SELECT depth, value FROM computed_curves_archive WHERE set_id = ?1 AND upper(curve_name) = ?2",
    )?;
    let mut resolved_from_selected_set = std::collections::HashSet::new();
    for name in curve_names {
        let upper = name.trim().to_uppercase();
        if own_curves.contains(&upper) {
            resolved_from_selected_set.insert(upper);
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
        resolved_from_selected_set.insert(upper.clone());
        columns.insert(
            upper,
            depth.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect(),
        );
    }
    overlay_resolved_native_standard_inputs(
        conn,
        well_id,
        curve_names,
        &depth,
        &mut columns,
        &resolved_from_selected_set,
    )?;
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
        let params_json: Option<String> = r.get(4)?;
        let ancestry = params_json
            .as_deref()
            .and_then(|text| parse_curve_ancestry(text).ok());
        Ok(LogSetEntry {
            set_id: r.get(0)?,
            set_name: r.get(1)?,
            version: r.get(2)?,
            module: r.get(3)?,
            params_json,
            inputs_json: r.get(5)?,
            created_at: r.get(6)?,
            curve_names: Vec::new(),
            is_current: r.get(7)?
        ,
            ancestry,
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
pub(crate) fn restore_log_set(conn: &Connection, set_id: &str) -> Result<usize, String> {
    load_set_write_discipline(conn, set_id)?;
    validate_archived_continuous_depth_uniqueness(conn, set_id)?;
    // Atomic: the DELETE of current rows and the re-INSERT from the archive must not be split
    // by a crash (which would drop the current rows and leave them un-restored).
    crate::db::with_txn(conn, |conn| {
        conn.execute(
            "DELETE FROM computed_curves
             WHERE well_id = (SELECT well_id FROM log_sets WHERE set_id = ?1)
               AND upper(curve_name) IN (SELECT DISTINCT upper(curve_name) FROM computed_curves_archive WHERE set_id = ?1)",
            params![set_id],
        )?;
        let restored = conn.execute(
            "INSERT INTO computed_curves (well_id, depth, curve_name, value, set_id)
             SELECT well_id, depth, curve_name, value, set_id FROM computed_curves_archive WHERE set_id = ?1",
            params![set_id],
        )?;
        Ok::<usize, duckdb::Error>(restored)
    })
    .map_err(|error| error.to_string())
}

/// Deletes one version's archive rows + its log_sets row. Current values are kept (their
/// provenance tag is cleared) so deleting history can never change any plot or result.
pub(crate) fn delete_log_set(conn: &Connection, set_id: &str) -> duckdb::Result<()> {
    // Atomic: clearing provenance + dropping archive rows + dropping the log_sets row must not
    // be split by a crash (which could orphan archive rows or a dangling set_id reference).
    crate::db::with_txn(conn, |conn| {
        conn.execute("UPDATE computed_curves SET set_id = NULL WHERE set_id = ?1", params![set_id])?;
        conn.execute("DELETE FROM computed_curves_archive WHERE set_id = ?1", params![set_id])?;
        conn.execute("DELETE FROM run_parameters WHERE set_id = ?1", params![set_id])?;
        conn.execute("DELETE FROM log_sets WHERE set_id = ?1", params![set_id])?;
        Ok(())
    })
}

/// Catalog of a well's CURRENT computed curves with per-curve provenance (which set
/// version wrote it, by what module, when) and basic statistics for search/sort.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedCatalogEntry {
    pub curve_name: String,
    pub provenance_class: ComputedProvenanceClass,
    pub provenance_row_count: i64,
    pub set_name: Option<String>,
    pub version: Option<i64>,
    pub module: Option<String>,
    pub created_at: Option<String>,
    pub n_samples: i64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>
,
    pub ancestry: Option<CurveAncestry>,
}

pub(crate) fn list_computed_catalog(conn: &Connection, well_id: &str) -> duckdb::Result<Vec<ComputedCatalogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT cc.curve_name,
                CASE WHEN s.set_id IS NOT NULL THEN 'RECORDED'
                     ELSE ?2 END,
                COUNT(*), s.set_name, s.version, s.module,
                strftime(s.created_at, '%Y-%m-%d %H:%M'),
                COUNT(*) FILTER (WHERE NOT isnan(cc.value)),
                MIN(cc.value) FILTER (WHERE NOT isnan(cc.value)),
                MAX(cc.value) FILTER (WHERE NOT isnan(cc.value)),
                AVG(cc.value) FILTER (WHERE NOT isnan(cc.value)),
                s.params_json
         FROM computed_curves cc
         LEFT JOIN log_sets s ON s.set_id = cc.set_id
         WHERE cc.well_id = ?1
         GROUP BY cc.curve_name, s.set_id, s.set_name, s.version, s.module,
                  s.created_at, s.params_json
         ORDER BY cc.curve_name, s.set_id NULLS FIRST",
    )?;
    let rows = stmt.query_map(params![well_id, LEGACY_UNRECORDED], |r| {
        let provenance_class = match r.get::<_, String>(1)?.as_str() {
            "RECORDED" => ComputedProvenanceClass::Recorded,
            _ => ComputedProvenanceClass::LegacyUnrecorded,
        };
        let params_json: Option<String> = r.get(11)?;
        Ok(ComputedCatalogEntry {
            curve_name: r.get(0)?,
            provenance_class,
            provenance_row_count: r.get(2)?,
            set_name: r.get(3)?,
            version: r.get(4)?,
            module: r.get(5)?,
            created_at: r.get(6)?,
            n_samples: r.get(7)?,
            min: r.get(8)?,
            max: r.get(9)?,
            mean: r.get(10)?,
            ancestry: if provenance_class == ComputedProvenanceClass::Recorded {
                params_json
                    .as_deref()
                    .and_then(|text| parse_curve_ancestry(text).ok())
            } else {
                None
            },
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
#[cfg(test)]
pub(crate) fn write_computed_curves_batch(
    conn: &Connection,
    well_id: &str,
    depth: &[f32],
    curves: &[(&str, &[f32])],
) -> duckdb::Result<()> {
    if curves.is_empty() {
        return Ok(());
    }
    let ancestry = CurveAncestry {
        schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
        module: "TEST_FIXTURE".into(),
        module_version: env!("CARGO_PKG_VERSION").into(),
        inputs: Vec::new(),
        parameters: Vec::new(),
        parameter_state: Some(ProvenanceAbsentState::NotApplicable),
        zone_scope: AncestryZoneScope::WholeWell,
        actor: AncestryActor {
            kind: AncestryActorKind::Automated,
            identity: "rust-test-fixture".into(),
        },
        timestamp_utc_ms: ancestry_timestamp_utc_ms().expect("test fixture timestamp"),
        outputs: curves
            .iter()
            .map(|(name, _)| AncestryOutput {
                curve: (*name).to_string(),
                derivation: format!("test_fixture:{name}"),
            })
            .collect(),
    };
    let spec = CompleteLogSetSpec::try_new("TEST_FIXTURE", ancestry)
        .expect("complete test fixture ancestry");
    let (set_id, _) =
        create_complete_log_set(conn, well_id, &spec).expect("create test fixture set");
    write_computed_curves_with_ancestry(conn, well_id, depth, curves, &set_id)
        .expect("write complete test fixture");
    Ok(())
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
    custody: &RunCustody,
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<EquationRunResult> {
    if let Err(error) = custody.validate() {
        return well_ids
            .iter()
            .map(|well_id| EquationRunResult::failed(well_id.clone(), error.clone()))
            .collect();
    }
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
            // The Python branch of this same command already drains on cancel
            // (`python_engine.rs`), so without this the Cancel button's behaviour depended on
            // the equation's LANGUAGE — same job kind, same button, different outcome.
            if progress.map_or(false, |p| p.is_cancelled()) {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                }
                return EquationRunResult::failed(well_id.clone(), "cancelled".into());
            }
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
            // A Rhai error is caught per sample and written as MISSING, and the only thing that
            // ever converted that into a reported failure was the all-MISSING guard below — which
            // fires only when EVERY sample failed. So a script raising on half the depths produced
            // a holed curve and reported a clean success with the full row count, indistinguishable
            // from the innocent case where the inputs were simply absent there
            // (`docs/review_triage.md` finding 13). Count them instead.
            //
            // Samples whose inputs were already MISSING never reach the evaluator (the `has_nan`
            // short-circuit above), which is exactly what makes these counts mean something: they
            // are depths where the script had real numbers to work with and still could not answer.
            let mut raised = 0usize;
            let mut non_finite = 0usize;
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
                    // Rhai evaluates `1.0/0.0` to infinity, and an in-range f64 like `exp(100)`
                    // overflows to infinity on the `as f32` cast. Downstream only screens for NaN,
                    // so an infinity here would be written to a computed curve and could then be
                    // picked as a predictor — where it poisons z-scores and comparison sorts.
                    // Missing is the honest reading of a non-finite result.
                    Ok(v) if (v as f32).is_finite() => output.push(v as f32),
                    Ok(_) => {
                        non_finite += 1;
                        output.push(f32::NAN);
                    }
                    Err(_) => {
                        raised += 1;
                        output.push(f32::NAN);
                    }
                }
            }

            // A run whose output is entirely MISSING — a typo'd/unresolvable input NaN-poisons
            // every sample via the has_nan short-circuit, or an output name that resolved to
            // nothing — reads as a clean "n rows written" success in the run summary, so surface
            // it as a loud per-well error instead and don't write a useless all-NaN curve.
            if !output.iter().any(|v| v.is_finite()) {
                let msg = "equation produced no finite output — check the input/output curve name(s) resolve to data".to_string();
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Warned, Some(msg.clone()));
                }
                return EquationRunResult::failed(well_id.clone(), msg);
            }

            let conn = db.lock().unwrap();
            if let Err(e) = write_equation_output(&conn, well_id, &depth, equation, &output, custody) {
                if let Some(p) = progress {
                    p.finish_item(well_id, crate::jobs::ItemState::Failed, Some(e.to_string()));
                }
                return EquationRunResult::failed(well_id.clone(), e.to_string());
            }
            // Reported as a WARNING rather than an error, and the curve is still written: the run
            // did produce data, and an equation guarded by a domain check legitimately refuses
            // some depths. What is indefensible is silence — a holed curve with nothing on the log
            // to prompt a second look.
            let note = match (raised, non_finite) {
                (0, 0) => None,
                (r, 0) => Some(format!("the script raised on {r} of {n} sample(s) — those depths are MISSING")),
                (0, f) => {
                    Some(format!("{f} of {n} sample(s) evaluated to a non-finite value — those depths are MISSING"))
                }
                (r, f) => Some(format!(
                    "the script raised on {r} of {n} sample(s), and {f} more evaluated non-finite — those depths are MISSING"
                )),
            };
            if let Some(p) = progress {
                let state =
                    if note.is_some() { crate::jobs::ItemState::Warned } else { crate::jobs::ItemState::Ok };
                p.finish_item(well_id, state, note.clone());
            }
            EquationRunResult::success(well_id.clone(), n).with_note(note)
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
    values: &[f32]
,
    custody: &RunCustody,
) -> Result<(), String> {
    let module= format!("equation:{}", equation.name);
    let inputs = equation.input_curves.iter().map(|curve| (well_id.to_string()
    , curve.clone(), curve.clone()))
        .collect::<Vec<_>>();
    let parameters = serde_json::json!({
        "equation_id": equation.equation_id,
        "language": equation.language,
        "script": equation.script,
        "output_units": equation.output_units,
    });
    let outputs = vec![equation.output_curve.clone()];
    let mut spec = complete_curve_run_spec(
        conn,
        well_id,
        "EQUATION",
        &module,
        custody,
        &inputs,
        None,
        parameters,
        AncestryZoneScope::WholeWell,
        &outputs,
    )?;
    spec.record_parameters_not_applicable()?;
    let (set_id, _) = create_complete_log_set(conn, well_id, &spec)?;
    write_computed_curves_with_ancestry(conn, well_id, depth, &[(equation.output_curve.as_str(), values)], &set_id)
}

#[cfg(test)]
mod tests {
    /// CORRECTNESS — SB-DBM-026 / SB-DBM-T25. Dossier invariant 12 and T-DB-20 require
    /// continuous sets to refuse a duplicate depth with both source rows, while POINT sets keep
    /// legitimate duplicates. F-26 cites IP's explicit 0.01 ft FPRESS perturbation; it is fixture
    /// input here, never a SandiBumi default. The PK-less store rationale is `db.rs:292-305`.
    #[test]
    fn continuous_duplicates_name_both_source_rows_while_point_duplicates_require_and_record_their_resolution() {
        use crate::db;
        use crate::schema_vocab::{DuplicateDepthResolution, SamplingStyle};
        use crate::units::{set_project_depth_unit, DepthOffset, DepthUnit};

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        set_project_depth_unit(&conn, DepthUnit::Feet).unwrap();
        let well_id = uuid::Uuid::new_v4();
        db::insert_well(&conn, well_id, "DUPLICATE-DEPTH-FIXTURE", None, None, Some(0.0)).unwrap();
        let well_id = well_id.to_string();
        let depths = [1000.0_f32, 1000.0];
        let values = [0.17_f32, 0.19];

        let make_spec = |set_name: &str, curve: &str| {
            CompleteLogSetSpec::try_new(
                set_name,
                CurveAncestry {
                    schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
                    module: "SB-DBM-T25 fixture".into(),
                    module_version: "fixture-build".into(),
                    inputs: Vec::new(),
                    parameters: Vec::new(),
                    parameter_state: Some(ProvenanceAbsentState::NotApplicable),
                    zone_scope: AncestryZoneScope::WholeWell,
                    actor: AncestryActor {
                        kind: AncestryActorKind::Automated,
                        identity: "SB-DBM-T25".into(),
                    },
                    timestamp_utc_ms: 1,
                    outputs: vec![AncestryOutput {
                        curve: curve.into(),
                        derivation: "SB-DBM-T25 fixture".into(),
                    }],
                },
            )
            .unwrap()
        };

        for (style, curve) in [
            (SamplingStyle::ContinuousRegular, "REGULAR_DUP"),
            (SamplingStyle::ContinuousIrregular, "IRREGULAR_DUP"),
        ] {
            let spec = make_spec(style.as_str(), curve).with_sampling_style(style);
            let (set_id, _) = create_complete_log_set(&conn, &well_id, &spec).unwrap();
            let error = write_computed_curves_with_ancestry(
                &conn,
                &well_id,
                &depths,
                &[(curve, &values)],
                &set_id,
            )
            .expect_err("a continuous duplicate must be refused");
            assert!(error.contains(curve), "{error}");
            assert!(error.contains("1000"), "{error}");
            assert!(error.contains("source rows 1 and 2"), "{error}");
            let written: i64 = conn
                .query_row(
                    "SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = ?2",
                    duckdb::params![well_id, curve],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(written, 0, "refusal must precede any current-store mutation");
        }

        let restore_curve = "RESTORE_DUP";
        let restore_spec = make_spec("RESTORE_DUP_SET", restore_curve)
            .with_sampling_style(SamplingStyle::ContinuousRegular);
        let (restore_set, _) =
            create_complete_log_set(&conn, &well_id, &restore_spec).unwrap();
        conn.execute(
            "INSERT INTO computed_curves_archive
                (set_id, well_id, depth, curve_name, value)
             VALUES (?1, ?2, 1000.0, ?3, 0.17),
                    (?1, ?2, 1000.0, ?3, 0.19)",
            duckdb::params![restore_set.as_str(), well_id, restore_curve],
        )
        .unwrap();
        let restore_error = restore_log_set(&conn, restore_set.as_str())
            .expect_err("an archive restore is still a continuous write boundary");
        assert!(restore_error.contains(restore_curve), "{restore_error}");
        assert!(restore_error.contains("1000"), "{restore_error}");
        assert!(
            restore_error.contains("source rows 1 and 2"),
            "{restore_error}"
        );
        let restored: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves
                 WHERE well_id = ?1 AND curve_name = ?2",
                duckdb::params![well_id, restore_curve],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restored, 0, "restore refusal must precede current-store mutation");

        let point_rows = [
            db::AuxRow {
                dataset: "PRESSURE".into(),
                depth_top: 1000.0,
                depth_base: None,
                item: "FPRESS".into(),
                value_num: Some(0.17),
                value_text: None,
            },
            db::AuxRow {
                dataset: "PRESSURE".into(),
                depth_top: 1000.0,
                depth_base: None,
                item: "FPRESS".into(),
                value_num: Some(0.19),
                value_text: None,
            },
        ];
        db::insert_aux_data(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_PRESERVED",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
        )
        .expect("the shipped point-data writer must accept legitimate duplicates");
        let preserved_rows: Vec<(f32, f32)> = {
            let mut statement = conn
                .prepare(
                    "SELECT depth_top, value_num FROM aux_data
                     WHERE dataset = 'PRESSURE' AND set_name = 'POINT_PRESERVED'
                     ORDER BY value_num",
                )
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(preserved_rows, vec![(1000.0, 0.17), (1000.0, 0.19)]);
        let preserved_declaration: (String, String, i64, i64) = conn
            .query_row(
                "SELECT s.sampling_style, s.duplicate_resolution,
                        count(r.source_row), count(r.perturbation_value)
                 FROM aux_sets s
                 LEFT JOIN aux_duplicate_depth_resolutions r
                   ON r.well_id = s.well_id AND r.dataset = s.dataset AND r.set_name = s.set_name
                 WHERE s.set_name = 'POINT_PRESERVED'
                 GROUP BY s.sampling_style, s.duplicate_resolution",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            preserved_declaration,
            ("POINT".into(), "PRESERVE".into(), 2, 0),
            "preservation is declared and logged for both duplicate source rows without a made-up offset"
        );

        let missing_offset = db::insert_aux_data_with_resolution(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_NO_DEFAULT",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
            DuplicateDepthResolution::Perturb,
            None,
        )
            .expect_err("perturbation ships with no default");
        assert!(missing_offset.to_string().contains("unit-typed offset"), "{missing_offset}");

        let explicit_offset = DepthOffset {
            value: 0.01,
            unit: DepthUnit::Feet,
        };
        db::insert_aux_data_with_resolution(
            &conn,
            &well_id,
            "PRESSURE",
            "POINT_PERTURBED",
            Some("SB-DBM-T25 fixture"),
            &point_rows,
            DuplicateDepthResolution::Perturb,
            Some(explicit_offset),
        )
        .unwrap();
        let perturbed_depths: Vec<f32> = {
            let mut statement = conn
                .prepare(
                    "SELECT depth_top FROM aux_data
                     WHERE dataset = 'PRESSURE' AND set_name = 'POINT_PERTURBED'
                     ORDER BY value_num",
                )
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(perturbed_depths, vec![1000.0, 1000.01]);
        let log: Vec<(i64, f32, f32, f64, String)> = {
            let mut statement = conn
                .prepare(
                    "SELECT r.source_row, r.original_depth, r.stored_depth,
                            r.perturbation_value, r.perturbation_unit
                     FROM aux_duplicate_depth_resolutions r
                     WHERE r.dataset = 'PRESSURE' AND r.set_name = 'POINT_PERTURBED'
                     ORDER BY source_row",
                )
                .unwrap();
            statement
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                })
                .unwrap()
                .collect::<duckdb::Result<_>>()
                .unwrap()
        };
        assert_eq!(
            log,
            vec![
                (1, 1000.0, 1000.0, 0.01, "FT".into()),
                (2, 1000.0, 1000.01, 0.01, "FT".into()),
            ]
        );
        let point_current: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves WHERE curve_name = 'FPRESS'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(point_current, 0, "POINT duplicates must never enter the current aligned store");
        let primary_keys: i64 = conn
            .query_row(
                "SELECT count(*) FROM duckdb_constraints()
                 WHERE table_name = 'computed_curves' AND constraint_type = 'PRIMARY KEY'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(primary_keys, 0, "the discipline must not be replaced by a computed_curves PK");
    }

    /// **An imported log is offered wherever the product asks the user to pick a curve.**
    ///
    /// `fetch_curve_frame` has resolved the generic store since rule 11, so a module or an equation
    /// could always consume PEF, CALI, DRHO or a second resistivity run. `list_curve_catalog` did
    /// not list them — so every picker built from it (the ML dialog's input checkboxes above all)
    /// showed the six standard columns and the computed curves, and a well delivered with fifteen
    /// logs offered five. The engine could read them the whole time; the list could not say so, and
    /// an input a user cannot see is an input the product does not have.
    ///
    /// Pinned from both sides: the imported name appears, AND a name already carried by a standard
    /// column or a computed curve appears exactly ONCE — `fetch_curve_frame` resolves those first,
    /// so listing the imported twin beside it would offer a choice that does not exist.
    #[test]
    fn an_imported_log_is_offered_as_an_input_not_only_the_standard_six() {
        use crate::db;
        let conn = duckdb::Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-1", None, None, Some(0.0)).unwrap();
        let id = well.to_string();
        for (mn, unit, fam) in
            [("PEF", "B/E", "PEF"), ("CALI", "IN", "CALI"), ("DRHO", "G/C3", "DRHO")]
        {
            db::upsert_curve_meta(&conn, &id, "RAW", mn, Some(unit), Some(fam), Some("LAS import"), None)
                .unwrap();
        }
        // The same mnemonic delivered again in a second set, and once as a standard column.
        db::upsert_curve_meta(&conn, &id, "FPROOH", "PEF", Some("B/E"), Some("PEF"), Some("LAS import"), None)
            .unwrap();
        db::upsert_curve_meta(&conn, &id, "RAW", "GR", Some("GAPI"), Some("GR"), Some("LAS import"), None)
            .unwrap();

        let catalog = super::list_curve_catalog(&conn).unwrap();
        let names: Vec<&str> = catalog.iter().map(|c| c.name.as_str()).collect();
        for want in ["PEF", "CALI", "DRHO"] {
            assert!(names.contains(&want), "an imported log must be pickable: {names:?}");
        }
        assert_eq!(
            names.iter().filter(|n| **n == "PEF").count(),
            1,
            "one entry per mnemonic, not one per well per delivery: {names:?}"
        );
        assert_eq!(
            names.iter().filter(|n| **n == "GR").count(),
            1,
            "the standard column is what a frame read resolves, so it is the entry that stands"
        );
        let pef = catalog.iter().find(|c| c.name == "PEF").unwrap();
        assert_eq!(pef.units.as_deref(), Some("B/E"), "the unit travels, so a picker can show it");
        assert_eq!(pef.source, "Imported", "the reader can tell a delivered log from a computed one");
        assert!(!names.contains(&"DEPTH"), "the index is not an input curve");
    }

    /// **Every tool that reads or writes a curve offers a log set** (Jauhar, 2026-08-05:
    /// *"each tools or modules should give user freedom to define input and output log set ...
    /// and their own curves"*).
    ///
    /// Before this, exactly two surfaces of nineteen did — the module dialog and the workflow
    /// builder — so ML, SandiMin, the saturation-height fit, the cutoff sweep, the pay summary,
    /// the facies tie, Lorenz, the results QC and every deliverable read whatever the current
    /// values happened to be, and the three writers among them hardcoded where their output
    /// landed. None of that is visible in a result: a report quoting last week's porosity looks
    /// exactly like one quoting today's.
    ///
    /// Checked against the SOURCE rather than by calling each command, because the failure this
    /// guards is a request struct that quietly loses the field in a future refactor — which
    /// compiles, runs, and silently reverts the tool to "current values".
    #[test]
    fn every_curve_consuming_request_still_offers_a_log_set() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        // (file, struct, needs an OUTPUT set as well as an input one)
        let contracts: [(&str, &str, bool); 13] = [
            ("ml.rs", "MlRequest", true),
            ("ml.rs", "MlApplyRequest", true),
            ("ml.rs", "MlEvalRequest", false),
            ("multimin2.rs", "MultiminRequest", true),
            ("coreimage.rs", "CoreLogSpec", true),
            ("workflow.rs", "PaySummaryRequest", false),
            ("workflow.rs", "CutoffSweepRequest", false),
            ("netflag.rs", "NetFlagSpec", false),
            ("shf_fit.rs", "ShfFitRequest", false),
            ("shf_fit.rs", "CuddyFoilRequest", false),
            ("facies_tie.rs", "FaciesConfusionRequest", false),
            ("lorenz.rs", "LorenzRequest", false),
            ("resultsqc.rs", "SwSpreadRequest", false),
        ];
        for (file, name, wants_output) in contracts {
            let src = std::fs::read_to_string(dir.join(file))
                .unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
            let head = format!("pub struct {name} {{");
            let start = src.find(&head).unwrap_or_else(|| panic!("{file}: no {name}"));
            // The struct body up to its closing brace at column 0 — enough to see its fields
            // without pulling in the next declaration's.
            let body = &src[start..];
            let end = body.find("
}").unwrap_or(body.len());
            let body = &body[..end];
            if wants_output {
                assert!(
                    body.contains("pub output_set:"),
                    "{file}: {name} writes curves but no longer lets the caller name the log set                      they are versioned into — a hardcoded destination is what this replaced"
                );
            } else {
                assert!(
                    body.contains("pub input_set:"),
                    "{file}: {name} reads curves but no longer takes an input_set, so it silently                      reads whatever the current values are"
                );
            }
        }
    }

    use super::*;
    use duckdb::Connection;

    /// T-PETRO-02, the versioning half. Re-running a module must land as version N+1 and never
    /// overwrite the previous run, because that history is the only way to answer "which OPT_GR
    /// produced the VSH in this report" after the fact. Five runs of `vsh_gr` under one set name
    /// have to be five versions carrying five different parameter records.
    ///
    /// The per-well independence is the part with a real bug behind it. `create_log_sets_batch`
    /// pre-computes each well's next version because reading `MAX(version)` after an INSERT
    /// inside the same transaction trips a DuckDB internal error (`equations.rs:671`) — and a
    /// pre-computation that took ONE number for the whole batch would give a freshly added well
    /// its neighbours' version. Its history would then start at 7, and every earlier version of
    /// it would appear to exist and be missing.
    #[test]
    fn re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let a = "33333333-3333-3333-3333-333333333333";
        let b = "44444444-4444-4444-4444-444444444444";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{a}', 'SANDI-V1'), ('{b}', 'SANDI-V2');"
        ))
        .unwrap();

        let spec = |opt: &str| LogSetSpec {
            set_name: "INTERP".into(),
            module: "vsh_gr".into(),
            params_json: format!("{{\"OPT_GR\":\"{opt}\"}}"),
            inputs_json: "[\"GR\"]".into(),
        };

        // The plan's own sequence: five runs, one per OPT_GR, all into the set named INTERP.
        let opts = ["LINEAR", "LARINOV1", "LARINOV2", "STIEBER1", "CLAVIER"];
        let mut ids = Vec::new();
        for (i, opt) in opts.iter().enumerate() {
            let (set_id, version) = create_log_set(&conn, a, &spec(opt)).unwrap();
            assert_eq!(version, i as i64 + 1, "run {} of vsh_gr must be version {}", i + 1, i + 1);
            ids.push(set_id);
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), opts.len(), "each run needs its own set id, or the runs share history");

        // Every earlier run survives, and each carries the parameters that produced it — which
        // is what makes the version list answerable rather than just countable.
        let sets = list_log_sets(&conn, a).unwrap();
        let interp: Vec<_> = sets.iter().filter(|s| s.set_name == "INTERP").collect();
        assert_eq!(interp.len(), 5, "a re-run must never overwrite the version before it");
        for opt in opts {
            assert!(
                interp.iter().any(|s| s.params_json.as_deref().unwrap_or("").contains(opt)),
                "no version records OPT_GR {opt}; the tooltip could not tell them apart"
            );
        }

        // A DIFFERENT set name on the same well versions independently — INTERP's five runs must
        // not push a first run of FINAL to version 6.
        let (_, first_final) = create_log_set(&conn, a, &LogSetSpec { set_name: "FINAL".into(), ..spec("LINEAR") }).unwrap();
        assert_eq!(first_final, 1, "a set's version counts that set's own runs");

        // And a well that has never been run starts at 1, whatever its neighbours are on.
        let (_, first_b) = create_log_set(&conn, b, &spec("LINEAR")).unwrap();
        assert_eq!(first_b, 1, "version is per well, not per project");

        // The batch path must agree with the single path, per well. Well A is on 5, well B on 1,
        // so one shared number for the batch would be wrong for at least one of them.
        let batch = create_log_sets_batch(&conn, &[a.to_string(), b.to_string()], &spec("LARINOV2")).unwrap();
        assert_eq!(batch.len(), 2);
        let version_of = |well: &str, set_id: &str| -> i64 {
            list_log_sets(&conn, well).unwrap().into_iter().find(|s| s.set_id == *set_id).unwrap().version
        };
        assert_eq!(version_of(a, &batch[a]), 6, "well A had 5 INTERP runs, so the batch is its 6th");
        assert_eq!(version_of(b, &batch[b]), 2, "well B had 1, so the batch is its 2nd");
    }

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

    /// A log-view request that names an imported set is an identity request, not a family
    /// lookup. Its own depth samples must survive even when the well's standard frame has a
    /// different spacing; a viewport bound is applied before display decimation. Leaving the
    /// set blank deliberately retains the established standard/computed/RAW resolution.
    #[test]
    fn explicit_track_set_keeps_its_native_grid_and_filters_before_decimation() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = "25252525-2525-2525-2525-252525252525";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'SANDI-NATIVE');
             INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob) VALUES
                ('{well}', 1000.0000, 10.0, 2.0, 0.10, 2.40),
                ('{well}', 1000.1524, 20.0, 2.0, 0.20, 2.50),
                ('{well}', 1000.3048, 30.0, 2.0, 0.30, 2.60);"
        ))
        .unwrap();
        let curve_id = crate::db::upsert_curve_meta(
            &conn,
            well,
            "WIRE_ALT",
            "GR",
            Some("GAPI"),
            Some("GR"),
            Some("LAS import"),
            None,
        )
        .unwrap();
        let native_depth: Vec<f32> = (0..=8).map(|i| 999.5 + i as f32 * 0.5).collect();
        let native_value: Vec<f32> = (0..=8).map(|i| 100.0 + i as f32).collect();
        crate::db::insert_curve_samples(&conn, &curve_id, &native_depth, &native_value).unwrap();

        let explicit = TrackCurveRequest { curve_name: "GR".into(), set_name: Some("WIRE_ALT".into()) };
        let full = fetch_track_data(&conn, well, &[explicit.clone()], 100, None, None).unwrap();
        assert_eq!(full.len(), 1);
        assert_eq!(full[0].curve_name, track_curve_key(&explicit));
        let packed: &[f32] = bytemuck::cast_slice(&full[0].data);
        let n = full[0].point_count;
        assert_eq!(&packed[..n], native_depth.as_slice(), "explicit set must keep native depths");
        assert_eq!(&packed[n..], native_value.as_slice(), "explicit set must keep native values");

        let visible = fetch_track_data(&conn, well, &[explicit], 1, Some(1000.0), Some(1002.5)).unwrap();
        let visible_packed: &[f32] = bytemuck::cast_slice(&visible[0].data);
        let visible_n = visible[0].point_count;
        assert!(visible_n <= 2, "one pixel bucket emits at most its min/max pair");
        assert!(visible_packed[..visible_n].iter().all(|d| *d >= 1000.0 && *d <= 1002.5));

        let current = TrackCurveRequest { curve_name: "GR".into(), set_name: None };
        let standard = fetch_track_data(&conn, well, &[current], 100, None, None).unwrap();
        let standard_packed: &[f32] = bytemuck::cast_slice(&standard[0].data);
        assert_eq!(&standard_packed[..standard[0].point_count], &[1000.0, 1000.1524, 1000.3048]);
        assert_eq!(&standard_packed[standard[0].point_count..], &[10.0, 20.0, 30.0]);
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

    /// DLIS/LAS same-mnemonic shadow resolution: the most recently stored curve wins by default;
    /// a user PROMOTE flips the winner to the older curve and is at-most-one-pinned; DELETE of
    /// the winner falls back to the surviving
    /// sibling. Exercises the equations resolver + db::promote/delete_generic_curve together.
    #[test]
    fn generic_curve_promote_and_delete_resolve_shadowing() {
        use crate::db;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "SHADOW-1", None, None, None).unwrap();
        let wid = well.to_string();
        let depths = vec![100.0f32, 101.0, 102.0];
        db::insert_standard_curves(
            &conn, well, depths.clone(), vec![10.0f32; 3],
            vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3],
        )
        .unwrap();

        // Two RAW 'PEF' curves colliding on the mnemonic: LAS (stored first, value 1.0) and
        // DLIS (stored second, value 2.0).
        let las = db::upsert_curve_meta(&conn, &wid, "RAW", "PEF", Some("B/E"), Some("PEF"), Some("LAS import"), None).unwrap();
        db::insert_curve_samples(&conn, &las, &depths, &[1.0, 1.0, 1.0]).unwrap();
        let dlis = db::upsert_curve_meta(&conn, &wid, "RAW", "PEF", Some("B/E"), Some("PEF"), Some("DLIS import"), Some(0)).unwrap();
        db::insert_curve_samples(&conn, &dlis, &depths, &[2.0, 2.0, 2.0]).unwrap();

        let pef = |c: &Connection| fetch_curve_frame(c, &wid, &["PEF".to_string()]).unwrap().1["PEF"][0];

        assert_eq!(pef(&conn), 2.0, "most-recently-stored DLIS wins by default");
        db::promote_generic_curve(&conn, &las).unwrap();
        assert_eq!(pef(&conn), 1.0, "promoted LAS wins");
        db::promote_generic_curve(&conn, &dlis).unwrap();
        assert_eq!(pef(&conn), 2.0, "re-promoted DLIS wins again (at-most-one-pinned)");
        db::delete_generic_curve(&conn, &dlis).unwrap();
        assert_eq!(pef(&conn), 1.0, "after deleting the winner, the sibling resolves");

        let cat = db::list_generic_curve_catalog(&conn, &wid).unwrap();
        assert!(cat.iter().all(|c| c.curve_id != dlis), "deleted curve gone from catalog");
        assert!(cat.iter().any(|c| c.curve_id == las && !c.pinned), "LAS present and unpinned");
    }

    /// A user PIN on one family member must not hijack a FAMILY-name request that resolves a
    /// DIFFERENT member of the same family. `pinned` is scoped per (well, set, mnemonic) by
    /// `db::promote_generic_curve`; the resolver applies it only to exact-mnemonic matches, so a
    /// family request still ranks by modification order after a sibling
    /// mnemonic is promoted. Guards the `CASE WHEN upper(mnemonic)=?2 ...` pin gate shared by
    /// `fetch_generic_curve_aligned` and `curve_edit::locate_curve`.
    #[test]
    fn generic_pin_does_not_hijack_family_request() {
        use crate::db;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "FAMILY-1", None, None, None).unwrap();
        let wid = well.to_string();
        let depths = vec![100.0f32, 101.0, 102.0];
        // res_deep all-NaN; CALI is non-standard, so "CALI" always resolves via the generic store.
        db::insert_standard_curves(
            &conn, well, depths.clone(), vec![10.0f32; 3],
            vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3], vec![f32::NAN; 3],
        )
        .unwrap();

        // Two RAW caliper curves in family CALI with DIFFERENT mnemonics — neither equals the
        // family string "CALI": DCAL is stored first, then HCAL is the MRU winner.
        let dcal = db::upsert_curve_meta(&conn, &wid, "RAW", "DCAL", Some("in"), Some("CALI"), Some("DLIS import"), Some(0)).unwrap();
        db::insert_curve_samples(&conn, &dcal, &depths, &[9.0, 9.0, 9.0]).unwrap();
        let hcal = db::upsert_curve_meta(&conn, &wid, "RAW", "HCAL", Some("in"), Some("CALI"), Some("LAS import"), None).unwrap();
        db::insert_curve_samples(&conn, &hcal, &depths, &[8.0, 8.0, 8.0]).unwrap();

        let cali = |c: &Connection| fetch_curve_frame(c, &wid, &["CALI".to_string()]).unwrap().1["CALI"][0];

        // The most recently stored HCAL wins the family bucket by default.
        assert_eq!(cali(&conn), 8.0, "MRU family member wins the family request by default");
        // Promoting DCAL resolves a DCAL-vs-DCAL mnemonic shadow — it must NOT hijack the CALI
        // family request, because DCAL's mnemonic != the requested family name. (Pre-fix this
        // returned 9.0: pinned sorted ahead of the ordinary family tie-break.)
        db::promote_generic_curve(&conn, &dcal).unwrap();
        assert_eq!(cali(&conn), 8.0, "a pin on a sibling mnemonic must not hijack the family bucket");
    }

    /// CORRECTNESS — source: `CLAUDE.md` rule 10a and its import-set build record. RAW has
    /// absolute priority even when an attached set carries the exact requested mnemonic; the
    /// attached exact curve becomes eligible only after RAW no longer carries that family.
    #[test]
    fn a_raw_family_match_beats_an_exact_mnemonic_outside_the_working_set_until_raw_is_absent() {
        use crate::db;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "RAW-PRIORITY-1", None, None, None).unwrap();
        let well_id = well.to_string();
        let depths = vec![100.0_f32, 101.0, 102.0];
        db::insert_standard_curves(
            &conn,
            well,
            depths.clone(),
            vec![10.0_f32; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();

        let raw_family = db::upsert_curve_meta(
            &conn,
            &well_id,
            "RAW",
            "HCAL",
            Some("in"),
            Some("CALI"),
            Some("LAS import"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &raw_family, &depths, &[8.0, 8.0, 8.0]).unwrap();
        let attached_exact = db::upsert_curve_meta(
            &conn,
            &well_id,
            "WIRE",
            "CALI",
            Some("in"),
            Some("CALI"),
            Some("LAS import"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(&conn, &attached_exact, &depths, &[12.0, 12.0, 12.0]).unwrap();

        let first = resolve_generic_curve_decision(&conn, &well_id, "CALI")
            .unwrap()
            .expect("the family is present");
        assert_eq!(first.chosen.curve_id, raw_family);
        assert_eq!(first.rule, Some(CurveResolutionRule::WorkingInputSet));
        assert_eq!(first.rejected.len(), 1);
        assert_eq!(first.rejected[0].curve_id, attached_exact);
        let (_, columns) = fetch_curve_frame(&conn, &well_id, &["CALI".into()]).unwrap();
        assert_eq!(columns["CALI"], [8.0, 8.0, 8.0]);

        db::delete_generic_curve(&conn, &raw_family).unwrap();
        let fallback = resolve_generic_curve_decision(&conn, &well_id, "CALI")
            .unwrap()
            .expect("the attached exact curve remains");
        assert_eq!(fallback.chosen.curve_id, attached_exact);
        assert_eq!(fallback.rule, Some(CurveResolutionRule::ExplicitName));
        assert!(fallback.rejected.is_empty());
        let (_, columns) = fetch_curve_frame(&conn, &well_id, &["CALI".into()]).unwrap();
        assert_eq!(columns["CALI"], [12.0, 12.0, 12.0]);
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

    /// An equation whose input curve resolves to nothing (typo / absent) yields an all-NaN
    /// output — that must surface as a per-well error, not a clean "n rows written" success.
    #[test]
    fn equation_all_nan_output_reports_error() {
        use crate::db;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "EQ-1", None, None, Some(0.0)).unwrap();
        let depths = vec![1000.0f32, 1000.5, 1001.0];
        let n = depths.len();
        db::insert_standard_curves(
            &conn, wid, depths.clone(),
            vec![40.0; n], vec![f32::NAN; n], vec![f32::NAN; n],
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let w = wid.to_string();
        let dbm = Mutex::new(conn);

        let eq = |name: &str, input: &str, output: &str, script: &str| EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            script: script.into(),
            input_curves: vec![input.into()],
            output_curve: output.into(),
            output_units: None,
            language: "rhai".into(),
        };

        // Typo'd input "GRX" resolves to an all-NaN column → output all-NaN → error.
        let bad = eq("bad", "GRX", "VSHX", "grx / 100.0");
        let r = run_equation(&dbm, &bad, &[w.clone()], &crate::workflow::test_run_custody(),
            None);
        assert!(r[0].error.is_some(), "all-NaN equation must report an error");

        // A resolvable input computes a real value → success with the full sample count.
        let good = eq("good", "GR", "GRSCALE", "gr / 100.0");
        let r = run_equation(&dbm, &good, &[w], &crate::workflow::test_run_custody(),
            None);
        assert!(r[0].error.is_none(), "good equation: {:?}", r[0].error);
        assert_eq!(r[0].rows_written, n);
    }

    /// Three wells, one of which lacks the input curve. `equation_all_nan_output_reports_error`
    /// already pins that ONE such well reports an error; what is unpinned — and what T-AUX-17
    /// actually asks for — is that the failure stays inside that well.
    ///
    /// The failing well is listed FIRST and the wells run under `rayon`, so this also checks
    /// that a per-well early return is a return and not an abort of the batch.
    #[test]
    fn one_failing_well_does_not_poison_a_multi_well_equation_run() {
        use crate::db;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // NPHI present or absent; every other curve identical, so the ONLY difference between
        // these wells is the input the equation reads.
        let seed = |name: &str, nphi: f32| -> String {
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, name, None, None, Some(0.0)).unwrap();
            let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
            let n = depths.len();
            db::insert_standard_curves(
                &conn,
                wid,
                depths,
                vec![40.0; n],
                vec![2.0; n],
                vec![nphi; n],
                vec![2.4; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
            )
            .unwrap();
            wid.to_string()
        };
        let no_nphi = seed("SANDI-EQ-BARE", f32::NAN);
        let good_a = seed("SANDI-EQ-A", 0.25);
        let good_b = seed("SANDI-EQ-B", 0.30);
        let dbm = Mutex::new(conn);

        let eq = EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: "PHIN_TEST".into(),
            description: None,
            script: "nphi * 1.0".into(),
            input_curves: vec!["NPHI".into()],
            output_curve: "PHIN_TEST".into(),
            output_units: None,
            language: "rhai".into(),
        };
        let wells = [no_nphi.clone(), good_a.clone(), good_b.clone()];
        let res = run_equation(&dbm, &eq, &wells, &crate::workflow::test_run_custody(),
            None);

        assert_eq!(res.len(), 3);
        let bare = &res[0];
        assert!(bare.error.is_some(), "the NPHI-less well must fail");
        assert!(
            bare.error.as_ref().unwrap().contains("no finite output"),
            "and say why in words a user can act on: {:?}",
            bare.error
        );
        for (label, r) in [("A", &res[1]), ("B", &res[2])] {
            assert!(r.error.is_none(), "healthy well {label} must not inherit the failure: {:?}", r.error);
            assert_eq!(r.rows_written, 4, "healthy well {label} writes every sample");
        }

        // Isolation is about the DATABASE, not just the return value: the failing well must
        // have written nothing at all, and the healthy ones must have written for themselves.
        let conn = dbm.lock().unwrap();
        let rows = |w: &str| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND UPPER(curve_name) = 'PHIN_TEST'",
                duckdb::params![w],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(rows(&no_nphi), 0, "the failed well must leave no curve behind, not an all-MISSING one");
        assert_eq!(rows(&good_a), 4);
        assert_eq!(rows(&good_b), 4);
    }

    /// The other half of T-AUX-17, and the half the manual step does not say out loud: the
    /// all-MISSING guard is the ONLY thing that turns a script error into a reported failure,
    /// and it fires only when EVERY sample failed.
    ///
    /// A Rhai error is caught per sample and written as MISSING (`Ok(_) | Err(_) => NAN`), so a
    /// script that raises on some depths and not others produces a curve with holes and reports
    /// a clean success with the full row count. Nothing tells the user their script threw — and
    /// a curve with holes is indistinguishable from one whose inputs were simply absent there.
    ///
    /// Fixed 2026-08-01 (`docs/review_triage.md` finding 13): the raises are counted and reported
    /// as a WARNING beside a successful run. Not an error — the curve was written and an equation
    /// guarded by a domain check legitimately refuses some depths — and not silence, which is what
    /// made a half-failed script indistinguishable from absent inputs.
    ///
    /// The count is meaningful only because samples whose INPUTS were already MISSING never reach
    /// the evaluator (the `has_nan` short-circuit). This test's fixture has no NaN inputs at all,
    /// so every miss here is a genuine raise; the sibling below adds the mixed case.
    #[test]
    fn a_script_that_raises_on_only_some_samples_says_so_without_calling_the_run_a_failure() {
        use crate::db;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-EQ-PARTIAL", None, None, Some(0.0)).unwrap();
        // GR alternates either side of 60, so the script raises on exactly half the samples.
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
        let gr = vec![40.0f32, 80.0, 40.0, 80.0];
        let n = depths.len();
        db::insert_standard_curves(
            &conn,
            wid,
            depths,
            gr,
            vec![2.0; n],
            vec![0.25; n],
            vec![2.4; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let w = wid.to_string();
        let dbm = Mutex::new(conn);

        let eq = EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: "HALF".into(),
            description: None,
            script: "if gr > 60.0 { throw \"boom\" } gr / 100.0".into(),
            input_curves: vec!["GR".into()],
            output_curve: "HALF".into(),
            output_units: None,
            language: "rhai".into(),
        };
        let res = run_equation(&dbm, &eq, &[w.clone()], &crate::workflow::test_run_custody(),
            None);

        assert!(res[0].error.is_none(), "the curve WAS written, so this is not a failure: {:?}", res[0].error);
        assert_eq!(res[0].rows_written, n, "every depth still gets a row — half of them MISSING");
        let note = res[0].note.as_deref().expect("a half-failing script must say so");
        assert!(
            note.contains("raised on 2 of 4"),
            "the note must give the coverage, not just that something went wrong — 2 of 4 is a \
             different situation from 2 of 4000: {note}"
        );
        assert!(note.contains("MISSING"), "and say what happened to those depths: {note}");

        // The written curve is half MISSING. Counted in Rust, never in SQL: DuckDB gives NaN a
        // total ordering, so `value = value` is TRUE for NaN and would count them as finite.
        let conn = dbm.lock().unwrap();
        let mut st = conn
            .prepare("SELECT value FROM computed_curves WHERE well_id = ?1 AND UPPER(curve_name) = 'HALF'")
            .unwrap();
        let vals: Vec<Option<f32>> = st
            .query_map(duckdb::params![w], |r| r.get::<_, Option<f32>>(0))
            .unwrap()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(vals.len(), n);
        let finite = vals.iter().filter(|v| v.is_some_and(f32::is_finite)).count();
        assert_eq!(finite, 2, "two samples computed, two were swallowed by the raise");

        // And the control that makes the gap sharp: raise on EVERY sample and the same script
        // is refused. The difference between "reported" and "silent" is only ever coverage.
        let all = EquationDef { script: "throw \"boom\"".into(), ..eq.clone() };
        drop(conn);
        let res = run_equation(&dbm, &all, &[w], &crate::workflow::test_run_custody(), None);
        assert!(
            res[0].error.as_ref().is_some_and(|e| e.contains("no finite output")),
            "a script that raises everywhere IS caught: {:?}",
            res[0].error
        );
    }

    /// The control that gives the warning above its meaning: a curve with holes because its
    /// INPUTS were missing there is the ordinary innocent case, and must stay silent.
    ///
    /// This is the whole reason the count is taken at the evaluator rather than from the output.
    /// Counting MISSING output samples would flag every equation ever run over a washed-out
    /// interval, the warning would fire constantly, and a warning that always fires is one nobody
    /// reads — at which point the real half-failed script goes past unnoticed again.
    #[test]
    fn a_curve_holed_by_missing_inputs_is_not_reported_as_a_script_failure() {
        use crate::db;
        use uuid::Uuid;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-EQ-HOLES", None, None, Some(0.0)).unwrap();
        let depths = vec![1000.0f32, 1000.5, 1001.0, 1001.5];
        // Half the GR is absent — a washout, a tool off bottom, an interval nobody logged.
        let gr = vec![40.0f32, f32::NAN, 45.0, f32::NAN];
        let n = depths.len();
        db::insert_standard_curves(
            &conn,
            wid,
            depths,
            gr,
            vec![2.0; n],
            vec![0.25; n],
            vec![2.4; n],
            vec![f32::NAN; n],
            vec![f32::NAN; n],
        )
        .unwrap();
        let w = wid.to_string();
        let dbm = Mutex::new(conn);

        let eq = EquationDef {
            equation_id: Uuid::new_v4().to_string(),
            name: "CLEAN".into(),
            description: None,
            script: "gr / 100.0".into(), // cannot raise
            input_curves: vec!["GR".into()],
            output_curve: "CLEAN".into(),
            output_units: None,
            language: "rhai".into(),
        };
        let res = run_equation(&dbm, &eq, &[w], &crate::workflow::test_run_custody(), None);

        assert!(res[0].error.is_none(), "{:?}", res[0].error);
        assert_eq!(res[0].rows_written, n);
        assert!(
            res[0].note.is_none(),
            "missing inputs are not a script failure and must not warn: {:?}",
            res[0].note
        );
    }

    /// CORRECTNESS — `22_database-model.md` SB-DBM-003 and §6 SB-DBM-T05/T30.
    /// The values are synthetic fixture inputs, not petrophysical defaults. F-11 is the cited
    /// source for keeping an absent parameter distinct from a missing curve sample.
    ///
    /// Removing the relational row write, its state index, the NULL value/source pair, the
    /// positive sourced row, or the write refusal must fail this one contract from opposite sides.
    #[test]
    fn a_parameter_without_a_source_is_queryable_required_unset_and_never_a_number() {
        use crate::db;
        use uuid::Uuid;

        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SOURCE-STATE", None, None, Some(0.0)).unwrap();

        let ancestry = CurveAncestry {
            schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
            module: "synthetic_source_state_fixture".into(),
            module_version: "fixture-build".into(),
            inputs: Vec::new(),
            parameters: vec![
                AncestryParameter {
                    name: "SOURCED_FIXTURE".into(),
                    value: serde_json::json!(2.0),
                    source: "22_database-model.md §6 SB-DBM-T05 fixture input".into(),
                    resolution: Some(ParameterResolution::Explicit),
                    manifest_version: None,
                    decision: None,
                },
                AncestryParameter {
                    name: "REQUIRED_INPUT".into(),
                    value: serde_json::json!("ABSENT"),
                    source: crate::modules::ABSENT_DEFAULT_SOURCE.into(),
                    resolution: None,
                    manifest_version: None,
                    decision: None,
                },
            ],
            parameter_state: None,
            zone_scope: AncestryZoneScope::WholeWell,
            actor: AncestryActor {
                kind: AncestryActorKind::Automated,
                identity: "SB-DBM-T05".into(),
            },
            timestamp_utc_ms: 1,
            outputs: vec![AncestryOutput {
                curve: "SOURCE_STATE_RESULT".into(),
                derivation: "SB-DBM-T05 fixture".into(),
            }],
        };
        let spec = CompleteLogSetSpec::try_new("SOURCE_STATE", ancestry).unwrap();
        let (set_id, _) = create_complete_log_set(&conn, &well_id.to_string(), &spec).unwrap();

        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM duckdb_indexes() WHERE index_name = 'idx_run_parameters_state'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, 1, "the unset-state query key must be indexed");

        let sourced: (String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT value_json, source, state FROM run_parameters
                 WHERE set_id = ?1 AND name = 'SOURCED_FIXTURE'",
                duckdb::params![set_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(&sourced.0).unwrap(), serde_json::json!(2.0));
        assert_eq!(sourced.1.as_deref(), Some("22_database-model.md §6 SB-DBM-T05 fixture input"));
        assert_eq!(sourced.2, None, "a present sourced value is not an absent state");

        let mut unset = conn
            .prepare(
                "SELECT name, value_json, source FROM run_parameters
                 WHERE state = 'REQUIRED_UNSET' ORDER BY set_id, position",
            )
            .unwrap();
        let rows: Vec<(String, Option<String>, Option<String>)> = unset
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<duckdb::Result<_>>()
            .unwrap();
        assert_eq!(rows, vec![("REQUIRED_INPUT".into(), None, None)]);

        let stored: String = conn
            .query_row(
                "SELECT params_json FROM log_sets WHERE set_id = ?1",
                duckdb::params![set_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        let payload: serde_json::Value = serde_json::from_str(&stored).unwrap();
        let required = &payload[CURVE_ANCESTRY_KEY]["parameters"][1];
        assert_eq!(required["state"], "REQUIRED_UNSET");
        assert!(required["value"].is_null(), "no parameter is not a numeric value");
        assert!(required["source"].is_null(), "no parameter has no invented source");

        let mut unsourced = spec.ancestry.clone();
        unsourced.parameters[0].source.clear();
        let error = CompleteLogSetSpec::try_new("SOURCE_STATE", unsourced)
            .expect_err("a UI-supplied numeric value without a source must be refused");
        assert!(error.contains("SOURCED_FIXTURE") && error.contains("source"), "{error}");

        // The migration side: a project written before the relational index existed already has
        // the source state in ancestry JSON. Re-opening must index that fact instead of silently
        // treating only future runs as queryable.
        let legacy = Connection::open_in_memory().unwrap();
        db::create_schema(&legacy).unwrap();
        let legacy_well = Uuid::new_v4();
        db::insert_well(&legacy, legacy_well, "PRE-INDEX-STATE", None, None, Some(0.0)).unwrap();
        legacy.execute_batch("DROP TABLE run_parameters").unwrap();
        let legacy_payload = serde_json::json!({
            CURVE_ANCESTRY_KEY: {
                "parameters": [{
                    "name": "LEGACY_REQUIRED_INPUT",
                    "value": "ABSENT",
                    "source": "ABSENT"
                }]
            }
        });
        let (legacy_set, _) = create_log_set(
            &legacy,
            &legacy_well.to_string(),
            &LogSetSpec {
                set_name: "PRE_INDEX".into(),
                module: "synthetic_pre_index_fixture".into(),
                params_json: legacy_payload.to_string(),
                inputs_json: "[]".into(),
            },
        )
        .unwrap();
        db::create_schema(&legacy).unwrap();
        let migrated: (Option<String>, Option<String>, String) = legacy
            .query_row(
                "SELECT value_json, source, state FROM run_parameters
                 WHERE set_id = ?1 AND name = 'LEGACY_REQUIRED_INPUT'",
                duckdb::params![legacy_set],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(migrated, (None, None, "REQUIRED_UNSET".into()));
    }
}
