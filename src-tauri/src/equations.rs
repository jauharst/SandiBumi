use crate::ancestry::{
    AncestryZoneScope,
    ComputedProvenanceClass,
    CurveAncestry,
    CurveResolutionRule,
    LEGACY_UNRECORDED,
    RunCustody,
    complete_curve_run_spec,
    create_complete_log_set,
    parse_curve_ancestry,
    write_computed_curves_with_ancestry,
};
// The module-level `#[cfg(test)]` helpers below share this scope, so their ancestry names are
// imported here rather than inside `mod tests`.
#[cfg(test)]
use crate::ancestry::{
    ancestry_timestamp_utc_ms, AncestryActor, AncestryActorKind, AncestryOutput,
    CompleteLogSetSpec, CURVE_ANCESTRY_SCHEMA_VERSION,
};
#[cfg(test)]
use crate::schema_vocab::ProvenanceAbsentState;
use duckdb::{params, params_from_iter, Connection};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use serde::{Deserialize, Serialize};
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
    /// AUDIT-2026-08-20 finding 26. True when the layout draws this curve as CLASS BLOCKS
    /// (`CurveStyle.fill == "blocks"`) - the one fact about a curve that lives in the style
    /// and so cannot be seen from here. `#[serde(default)]` so every older payload still
    /// deserializes as the ordinary continuous curve it was.
    #[serde(default)]
    pub class_curve: bool,
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
/// Packs a JSON header plus raw `f32` COLUMNS for a command whose result is scalars *and*
/// per-depth arrays.
///
/// Rule 3 says a `Vec<f32>` never crosses the bridge as JSON, and [`pack_curve_series`] already
/// serves the pure-curve commands. This is its sibling for a result that also carries metadata a
/// panel needs — model names, notes, summary statistics — which legitimately belong in JSON. The
/// two travel in one response rather than two commands, because a header fetched separately from
/// its columns can be fetched against a different run.
///
/// ```text
/// u32 header_len | header_len bytes of UTF-8 JSON | u32 n_columns | per column: u32 len, len × f32
/// ```
///
/// Column ORDER is the contract and the header names it; the columns themselves are anonymous, so
/// a reader that ignores the header's names cannot silently pair the wrong ones. Floats are
/// native-endian, the same convention `pack_curve_series` uses, because the frontend reads them
/// with `Float32Array` — which is also native-endian — and every platform this ships on is
/// little-endian.
pub fn pack_frame(header_json: &str, columns: &[&[f32]]) -> Vec<u8> {
    let header = header_json.as_bytes();
    let cap = 8 + header.len() + columns.iter().map(|c| 4 + c.len() * 4).sum::<usize>();
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(&(header.len() as u32).to_le_bytes());
    buf.extend_from_slice(header);
    buf.extend_from_slice(&(columns.len() as u32).to_le_bytes());
    for column in columns {
        buf.extend_from_slice(&(column.len() as u32).to_le_bytes());
        buf.extend_from_slice(bytemuck::cast_slice(column));
    }
    buf
}

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
    // fetch_track_frames returns one frame per request, in request order.
    let class_flags: Vec<bool> = curve_requests.iter().map(|r| r.class_curve).collect();
    fetch_track_frames(conn, well_id, curve_requests, depth_min, depth_max)?
        .into_iter()
        .zip(class_flags)
        .map(|(frame, is_class)| {
            // AUDIT-2026-08-20 finding 26. Min/max decimation is honest for a MEASUREMENT - the
            // extremes ARE the envelope, which is why it never averages. A CLASS INDEX has no
            // envelope: the min and max facies in a bucket are two arbitrary numbers, so at
            // whole-well zoom (~13 samples a bucket) one facies-7 sample painted ~2 m of facies
            // 7 and runs of the dominant class shredded into alternating pairs. The PRINT reads
            // the same curve undecimated and was right, so the facies column QC'd on screen was
            // not the column that shipped.
            //
            // A class curve is therefore not decimated at all, rather than decimated by a second
            // rule. Screen and print become identical BY CONSTRUCTION instead of by two
            // algorithms somebody has to keep in step - which is the failure this file is full
            // of warnings about. It costs nothing: the block renderer builds one geometry per
            // RUN, and undecimated data yields fewer runs than shredded data, not more.
            let (dec_depth, dec_value) = if is_class {
                (frame.depth.clone(), frame.value.clone())
            } else {
                crate::decimate::min_max_decimate(&frame.depth, &frame.value, target_pixel_height)
            };
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
        .filter(|(_, d)| *d >= lo && *d < hi)
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
           AND (?3 IS NULL OR depth < ?3)
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
///
/// **Plotted at their own depths is not the same as plotted on their own datum.** The track the
/// overlay lands in is the MD log frame, so a plug depth read as MD when the delivery declared
/// TVD or TVDSS puts the plug beside the wrong rock — which is why the return type widened from
/// `duckdb::Result` to `DbResult`: it has to be able to carry the refusal.
pub fn fetch_core_series(
    conn: &Connection,
    well_id: &str,
) -> crate::db::DbResult<Vec<TrackCurveSeries>> {
    crate::db::refuse_non_md_active_set(conn, "core_sets", well_id, None)?;
    let mut stmt = conn.prepare(
        "SELECT depth, cpor, cperm, cgd, csw FROM core_data
         WHERE well_id = ?1 AND set_name = COALESCE((SELECT set_name FROM core_sets WHERE well_id = ?1
                                                     ORDER BY active DESC, imported_at DESC LIMIT 1), 'RAW')
         ORDER BY depth",
    )?;
    // The four measurement columns are NULLABLE in the schema and a pre-set-era project keeps
    // whatever it had, so each is read as `Option<f32>` and a missing cell becomes NaN — which
    // `pack` below already drops from that property's own series. Reading them straight as `f32`
    // made DuckDB fail the ROW, and one failed row failed the whole command: a plug carrying a
    // real porosity and grain density plotted NOTHING because its permeability cell was NULL.
    // Depth is `NOT NULL` in the schema and stays a plain `f32`; a null there would be a
    // corrupt key, not a missing measurement, and must not be quietly read as absent.
    let rows = stmt.query_map(params![well_id], |row| {
        let opt = |v: Option<f32>| v.unwrap_or(f32::NAN);
        Ok((
            row.get::<_, f32>(0)?,
            opt(row.get::<_, Option<f32>>(1)?),
            opt(row.get::<_, Option<f32>>(2)?),
            opt(row.get::<_, Option<f32>>(3)?),
            opt(row.get::<_, Option<f32>>(4)?),
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
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f32>(1)?,
            row.get::<_, Option<f32>>(2)?.unwrap_or(f32::NAN),
        ))
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
        Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
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
pub(crate) struct GenericCurveCandidate {
    pub(crate) curve_id: String,
    pub(crate) set_name: String,
    pub(crate) set_version: i64,
    pub(crate) mnemonic: String,
    pub(crate) pinned: bool,
    pub(crate) final_flag: bool,
    pub(crate) modified_seq: Option<i64>,
}

/// SB-DIO-031/034 (DEC-030): a resolver request is TYPED. `ExactMnemonic` never falls back -
/// a different curve's data must not be supplied under a requested name - while
/// `SemanticFamily` is the rule-11 alias feature: it may resolve by family, and it always
/// returns the CONCRETE curve identity and the rule that chose it, never a silent stand-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CurveRequest {
    ExactMnemonic,
    SemanticFamily,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericCurveDecision {
    pub(crate) chosen: GenericCurveCandidate,
    pub(crate) rule: Option<CurveResolutionRule>,
    pub(crate) rejected: Vec<GenericCurveCandidate>,
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
pub(crate) fn resolve_generic_curve_decision(
    conn: &Connection,
    well_id: &str,
    curve_name: &str,
    request: CurveRequest,
) -> duckdb::Result<Option<GenericCurveDecision>> {
    let upper = curve_name.trim().to_uppercase();
    // SB-DIO-031: the exact request drops the family arm AT THE SQL, so no later stage can
    // reintroduce a stand-in - an absent mnemonic resolves to nothing, never to a relative.
    let name_filter = match request {
        CurveRequest::ExactMnemonic => "upper(mnemonic) = ?2",
        CurveRequest::SemanticFamily => "(upper(mnemonic) = ?2 OR upper(family) = ?2)",
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT curve_id, set_name, set_version, mnemonic, COALESCE(pinned, 0),
                COALESCE(final_flag, 0), modified_seq
         FROM curve_meta
         WHERE well_id = ?1
           AND {name_filter}
         ORDER BY (set_name = 'RAW') DESC,
                  (upper(mnemonic) = ?2) DESC,
                  (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                  COALESCE(final_flag, 0) DESC,
                  modified_seq DESC NULLS LAST,
                  curve_id",
    ))?;
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
    request: CurveRequest,
) -> duckdb::Result<Option<String>> {
    Ok(resolve_generic_curve_decision(conn, well_id, curve_name, request)?
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
    // SB-DIO-034: module/equation input fetch is the rule-11 SEMANTIC request by design -
    // mnemonic first, family only where the mnemonic is absent - and the concrete identity it
    // chose travels into the run's ancestry record, never a silent substitution.
    let Some(decision) =
        resolve_generic_curve_decision(conn, well_id, curve_name, CurveRequest::SemanticFamily)?
    else {
        return Ok(vec![f32::NAN; depth_grid.len()]);
    };
    let curve_id = decision.chosen.curve_id;

    let mut stmt = conn.prepare("SELECT depth, value FROM curve_samples WHERE curve_id = ?1")?;
    let rows = stmt.query_map(params![curve_id], |row| {
        Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
    })?;
    let mut by_depth: HashMap<u32, f32> = HashMap::new();
    for r in rows {
        let (d, v) = r?;
        by_depth.insert(d.to_bits(), v);
    }
    Ok(depth_grid.iter().map(|d| by_depth.get(&d.to_bits()).copied().unwrap_or(f32::NAN)).collect())
}

/// DEC-089's second half: brings an EXISTING project's standard columns onto the canonical
/// domain the generic store already holds.
///
/// Import stores canonical from now on, but a project imported before that keeps one delivery in
/// two numeric domains - a module reading NPHI 0.30 v/v while the log view reads 30.0 PU off the
/// same curve. This re-projects the six columns from the generic store, which is the same thing
/// import now does, so a migrated project and a re-imported one land identically.
///
/// **A column is replaced ALL AT ONCE or not at all.** The replacement is used only where the
/// generic store resolves a finite value at every depth the stored column has one - otherwise a
/// partially-covered curve would end up half converted and half raw, which is worse than either
/// and is invisible on a log. A well the generic store cannot cover keeps exactly what it had.
///
/// Idempotent by construction: re-projecting an already-canonical column from the same source
/// yields the same numbers. The `project_meta` stamp exists to keep a 2000-well project from
/// paying for that proof on every launch, not to make it safe to run twice.
///
/// Pre-generic-store projects are untouched in practice, and that is not luck:
/// `migrate_standard_curves_to_generic_store` backfilled their generic store FROM these very
/// columns, so both sides already hold the same raw numbers and there is no split to close. The
/// split only ever existed for a delivery imported after the generic store arrived.
pub fn migrate_standard_curves_canonical(conn: &Connection) -> crate::db::DbResult<usize> {
    // A legacy pre-stamp project has no `project_meta` at all, and neither does a bare schema -
    // the table is created by the format stamp, which is a different concern from this one.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_meta (key VARCHAR PRIMARY KEY, value VARCHAR);",
    )?;
    let done: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM project_meta WHERE key = 'standard_curves_canonical'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if done > 0 {
        return Ok(0);
    }

    let mut stmt = conn.prepare("SELECT well_id FROM wells")?;
    let wells: Vec<String> =
        stmt.query_map([], |r| r.get::<_, String>(0))?.filter_map(|r| r.ok()).collect();
    drop(stmt);

    let mut rewritten = 0usize;
    for well_id in wells {
        let mut stmt =
            conn.prepare("SELECT depth FROM standard_curves WHERE well_id = ?1 ORDER BY depth")?;
        let depth: Vec<f32> =
            stmt.query_map(params![well_id], |r| r.get::<_, f32>(0))?.filter_map(|r| r.ok()).collect();
        drop(stmt);
        if depth.is_empty() {
            continue;
        }
        for column in crate::schema_vocab::STANDARD_COLUMNS.iter().filter(|c| c.editable) {
            let col = column.storage_column;
            let mut stmt = conn.prepare(&format!(
                "SELECT {col} FROM standard_curves WHERE well_id = ?1 ORDER BY depth"
            ))?;
            let stored: Vec<f32> = stmt
                .query_map(params![well_id], |r| Ok(r.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN)))?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            if stored.len() != depth.len() || !stored.iter().any(|v| v.is_finite()) {
                continue;
            }
            let Ok(canon) = fetch_generic_curve_aligned(conn, &well_id, column.mnemonic, &depth)
            else {
                continue;
            };
            // Every depth the column answers at, the generic store must answer at too. Anything
            // less and the column would be part converted and part raw.
            let covers = canon.len() == stored.len()
                && stored.iter().zip(&canon).all(|(s, c)| !s.is_finite() || c.is_finite());
            if !covers {
                continue;
            }
            if stored.iter().zip(&canon).all(|(s, c)| s.to_bits() == c.to_bits()) {
                continue; // already canonical - the ordinary case for a metric delivery
            }
            let mut up = conn.prepare(&format!(
                "UPDATE standard_curves SET {col} = ?3 WHERE well_id = ?1 AND depth = ?2"
            ))?;
            for (d, v) in depth.iter().zip(&canon) {
                up.execute(params![well_id, d, if v.is_finite() { Some(*v) } else { None }])?;
            }
            rewritten += 1;
        }
    }
    conn.execute(
        "INSERT INTO project_meta (key, value) VALUES ('standard_curves_canonical', '1')
         ON CONFLICT (key) DO UPDATE SET value = '1'",
        [],
    )?;
    Ok(rewritten)
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
        // SB-DIO-034: the standard-column backfill is a SEMANTIC request - a well delivering
        // GRN under family GR must still fill the GR column, and the decision records which.
        if resolve_generic_curve_decision(conn, well_id, &upper, CurveRequest::SemanticFamily)?
            .is_some()
        {
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
                    Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
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
    // No stored verdict and a stored verdict of "not verified" are the same answer to the only
    // question asked here - has anybody checked this set's sampling? - so they were already
    // refusing in identical words. One pattern states it once.
    let Some((true, effective)) = verdict else {
        return Err(duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': sampling style has not \
             been verified. Reading by frame index assumes every curve in the set shares one \
             depth frame, and nothing has checked that this one does. Re-import the set with \
             its sampling style declared."
        )));
    };
    let style = crate::schema_vocab::SamplingStyle::parse(&effective).ok_or_else(|| {
        duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': the stored sampling \
             verdict '{effective}' is not one this version recognises. It was written by \
             another version or edited by hand, so what its depths mean cannot be \
             established. Re-import the set with its sampling style declared."
        ))
    })?;
    if style == crate::schema_vocab::SamplingStyle::Point {
        return Err(duckdb::Error::InvalidParameterName(format!(
            "frame-indexed read refused for import set '{set_name}': POINT data has no \
             continuous frame. A point delivery sits at the depths somebody sampled and has \
             no spacing between them, so there is no frame to index. Read it as point data \
             instead."
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
            Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
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
            Ok((row.get::<_, f32>(0)?, row.get::<_, Option<f32>>(1)?.unwrap_or(f32::NAN)))
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
    pub mean: Option<f64>,
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
        method_derivation: None,
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

        depth_frame: None,
        zone_set: None,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    };
    let spec = CompleteLogSetSpec::try_new("TEST_FIXTURE", ancestry)
        .expect("complete test fixture ancestry");
    let (set_id, _) =
        create_complete_log_set(conn, well_id, &spec).expect("create test fixture set");
    write_computed_curves_with_ancestry(conn, well_id, depth, curves, &set_id)
        .expect("write complete test fixture");
    Ok(())
}

/// Resolves the declared categorical inputs an equation would consume, per well. This happens
/// before either evaluator starts: arithmetic is a property of the requested operation, not of
/// whether Rhai compiles or a Python interpreter happens to be installed on this machine.
pub(crate) fn categorical_equation_errors(
    db: &Mutex<Connection>,
    equation: &EquationDef,
    well_ids: &[String],
) -> Result<HashMap<String, String>, String> {
    let conn = db.lock().unwrap();
    let mut errors = HashMap::new();
    for well_id in well_ids {
        let declared = crate::db::class_curves_for_well(&conn, well_id).map_err(|error| {
            format!(
                "cannot verify categorical equation inputs for well {well_id}: {error}"
            )
        })?;
        let mut categorical: Vec<String> = equation
            .input_curves
            .iter()
            .map(|curve| curve.trim().to_uppercase())
            .filter(|curve| declared.contains(curve))
            .collect();
        categorical.sort();
        categorical.dedup();
        let message = match categorical.as_slice() {
            [] => continue,
            [curve] => format!(
                "categorical curve '{curve}' cannot be used by equation '{}': arithmetic is refused",
                equation.name
            ),
            curves => format!(
                "categorical curves '{}' cannot be used by equation '{}': arithmetic is refused",
                curves.join("', '"),
                equation.name
            ),
        };
        errors.insert(well_id.clone(), message);
    }
    Ok(errors)
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
    let categorical_errors = match categorical_equation_errors(db, equation, well_ids) {
        Ok(errors) => errors,
        Err(error) => {
            return well_ids
                .iter()
                .map(|well_id| EquationRunResult::failed(well_id.clone(), error.clone()))
                .collect();
        }
    };
    // If every selected well is already refused, do not let script compilation replace the
    // categorical reporting-surface error with an unrelated syntax error. Mixed runs still
    // compile once so their continuous wells can proceed.
    if !well_ids.is_empty() && categorical_errors.len() == well_ids.len() {
        if let Some(progress) = progress {
            for well_id in well_ids {
                progress.finish_item(
                    well_id,
                    crate::jobs::ItemState::Failed,
                    categorical_errors.get(well_id).cloned(),
                );
            }
        }
        return well_ids
            .iter()
            .map(|well_id| {
                EquationRunResult::failed(
                    well_id.clone(),
                    categorical_errors[well_id].clone(),
                )
            })
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
            if let Some(error) = categorical_errors.get(well_id) {
                if let Some(p) = progress {
                    p.finish_item(
                        well_id,
                        crate::jobs::ItemState::Failed,
                        Some(error.clone()),
                    );
                }
                return EquationRunResult::failed(well_id.clone(), error.clone());
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
    values: &[f32],
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
    // Reachable only under cfg(test), where they are declared.
    use crate::ancestry::{
        create_log_set,
        AncestryActor, AncestryInput, AncestryOutput,
        LogSetSpec,
        APPLIED_STEPS_SCHEMA_VERSION,
        AncestryActorKind,
        AppliedStepsRecord,
        CURVE_ANCESTRY_SCHEMA_VERSION,
        CompleteLogSetSpec,
        ancestry_timestamp_utc_ms,
        derive_applied_steps,
        get_applied_steps,
        params_digest_hex,
    };
    /// DEC-089's second half: an EXISTING project is brought onto one numeric domain.
    ///
    /// Import stores canonical from now on, but a project imported before that keeps the split -
    /// a module reading NPHI 0.30 v/v while the log view reads 30.0 PU off the same curve. The
    /// fixture builds that state directly, because it is the only state worth migrating.
    ///
    /// Pinned from both sides. A column the generic store cannot fully cover must be left ALONE,
    /// or the migration would half-convert a curve - part canonical, part raw, invisible on a
    /// log and worse than either. Without that half, "rewrite everything you can reach" would
    /// pass the first assertion.
    #[test]
    fn an_existing_projects_standard_curves_are_brought_onto_one_domain_or_left_whole() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-SPLIT", None, None, None).unwrap();
        let well = id.to_string();
        let depth = [1000.0f32, 1000.5, 1001.0];

        // The pre-DEC-089 state: standard holds the file's raw porosity units, the generic store
        // holds the converted v/v, and nothing reconciles them.
        crate::db::insert_standard_curves(
            &conn,
            id,
            depth.to_vec(),
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![30.0, 28.0, 26.0],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        let curve = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic, unit, family, source)
             VALUES (?1, ?2, 'RAW', 'NPHI', 'v/v', 'NPHI', 'test')",
            duckdb::params![curve, well],
        )
        .unwrap();
        for (d, v) in depth.iter().zip([0.30f32, 0.28, 0.26]) {
            conn.execute(
                "INSERT INTO curve_samples (curve_id, depth, value) VALUES (?1, ?2, ?3)",
                duckdb::params![curve, d, v],
            )
            .unwrap();
        }

        let n = migrate_standard_curves_canonical(&conn).unwrap();
        assert_eq!(n, 1, "exactly the one split column is rewritten");
        let stored: Vec<f32> = conn
            .prepare("SELECT nphi FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(duckdb::params![well], |r| Ok(r.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(
            (stored[0] - 0.30).abs() < 1e-6 && (stored[2] - 0.26).abs() < 1e-6,
            "the projection now reads what the modules read: {stored:?}"
        );

        // Idempotent by CONSTRUCTION, not by the stamp - re-projecting an already-canonical
        // column from the same source yields the same numbers, so a second pass rewrites
        // nothing even with the stamp ignored. (A mutation that disabled the early return left
        // this green, which is how the distinction got stated here rather than assumed.) The
        // stamp is a cost guard, and it is asserted as one.
        assert_eq!(migrate_standard_curves_canonical(&conn).unwrap(), 0, "a second pass is a no-op");
        let stamped: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_meta WHERE key = 'standard_curves_canonical'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stamped, 1, "and the stamp is written so a 2000-well project stops re-scanning");

        // The other side: a column the generic store covers only PARTLY is left whole. A
        // half-converted curve is worse than an unconverted one, because nothing on the log
        // could show which samples were which.
        let conn2 = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn2).unwrap();
        let id2 = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn2, id2, "SANDI-PARTIAL", None, None, None).unwrap();
        let well2 = id2.to_string();
        crate::db::insert_standard_curves(
            &conn2,
            id2,
            depth.to_vec(),
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![30.0, 28.0, 26.0],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
            vec![f32::NAN; 3],
        )
        .unwrap();
        let curve2 = uuid::Uuid::new_v4().to_string();
        conn2
            .execute(
                "INSERT INTO curve_meta (curve_id, well_id, set_name, mnemonic, unit, family, source)
                 VALUES (?1, ?2, 'RAW', 'NPHI', 'v/v', 'NPHI', 'test')",
                duckdb::params![curve2, well2],
            )
            .unwrap();
        // Only the top two depths - the third is missing from the generic store.
        for (d, v) in depth.iter().take(2).zip([0.30f32, 0.28]) {
            conn2
                .execute(
                    "INSERT INTO curve_samples (curve_id, depth, value) VALUES (?1, ?2, ?3)",
                    duckdb::params![curve2, d, v],
                )
                .unwrap();
        }
        assert_eq!(
            migrate_standard_curves_canonical(&conn2).unwrap(),
            0,
            "partial coverage rewrites nothing"
        );
        let kept: Vec<f32> = conn2
            .prepare("SELECT nphi FROM standard_curves WHERE well_id = ?1 ORDER BY depth")
            .unwrap()
            .query_map(duckdb::params![well2], |r| Ok(r.get::<_, Option<f32>>(0)?.unwrap_or(f32::NAN)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(
            (kept[0] - 30.0).abs() < 1e-6 && (kept[2] - 26.0).abs() < 1e-6,
            "the column is left exactly as it was, not half converted: {kept:?}"
        );
    }

    /// Codex whole-repository review, P2: one SQL-NULL cell failed the WHOLE core overlay.
    ///
    /// The four measurement columns are nullable in the schema and the pre-set-era migration
    /// copies whatever a legacy project had. `fetch_core_series` promises four INDEPENDENT
    /// series, each holding only its own non-NaN samples - but it read every cell straight as
    /// `f32`, and DuckDB cannot turn SQL NULL into one. That failed the ROW, and one failed row
    /// failed the command, so a plug carrying a perfectly good porosity and grain density
    /// plotted NOTHING because its permeability cell was NULL. The sibling
    /// `db::get_core_point_series` had the right shape all along.
    ///
    /// Today's writers pass `f32::NAN` rather than NULL, which is why this needs a legacy row to
    /// reproduce and why the fixture inserts the NULLs with raw SQL - the point is precisely that
    /// an OLD project can hold them.
    ///
    /// Pinned from both sides: the properties that ARE present must still come back, and the null
    /// one must come back EMPTY rather than as a point at zero - a plug at 0 mD is a measurement,
    /// and "nobody measured it" is not.
    #[test]
    fn a_null_core_property_empties_its_own_series_instead_of_failing_the_whole_overlay() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-NULLCORE", None, None, None).unwrap();
        let well = id.to_string();

        // A legacy row exactly as the migration would have preserved it: real porosity and grain
        // density, SQL NULL permeability and saturation.
        conn.execute(
            "INSERT INTO core_data (well_id, set_name, depth, cpor, cperm, cgd, csw, depth_orig)              VALUES (?1, 'RAW', 1000.0, 0.20, NULL, 2.65, NULL, 1000.0)",
            duckdb::params![well],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO core_sets (well_id, set_name, active) VALUES (?1, 'RAW', 1)",
            duckdb::params![well],
        )
        .unwrap();

        let series = super::fetch_core_series(&conn, &well)
            .expect("a NULL measurement is missing data, not a failed command");
        let count = |name: &str| {
            series
                .iter()
                .find(|s| s.curve_name == name)
                .unwrap_or_else(|| panic!("{name} series is always returned"))
                .point_count
        };
        assert_eq!(count("CPOR"), 1, "the porosity that WAS measured must still plot");
        assert_eq!(count("CGD"), 1, "and so must the grain density");
        assert_eq!(count("CPERM"), 0, "the unmeasured permeability contributes no point");
        assert_eq!(count("CSW"), 0, "never a point at zero, which would be a measurement");
    }

    /// SB-DIO-031 (DEC-030): an EXACT request never falls back - a different curve's data
    /// must not be supplied under a requested name - while the SEMANTIC request resolves the
    /// same name by family and returns the CONCRETE identity and rule that chose it.
    #[test]
    fn an_exact_request_never_falls_back_to_family_while_a_semantic_request_names_the_curve_it_chose(
    ) {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-TYPED", None, None, None).unwrap();
        let well = id.to_string();
        // The well delivered GRN (family GR). Nothing carries the mnemonic GR itself.
        let curve = crate::db::upsert_curve_meta(
            &conn, &well, "RAW", "GRN", Some("gapi"), Some("GR"), None, None,
        )
        .unwrap();
        crate::db::insert_curve_samples(&conn, &curve, &[1000.0, 1001.0], &[50.0, 60.0])
            .unwrap();
        // A. ExactMnemonic: the requested name is absent, and NOTHING stands in for it.
        let exact =
            resolve_generic_curve_decision(&conn, &well, "GR", CurveRequest::ExactMnemonic)
                .unwrap();
        assert!(
            exact.is_none(),
            "an exact request for an absent mnemonic must resolve to nothing, never a relative"
        );
        // B. SemanticFamily: the same name resolves by family - and the decision names the
        //    CONCRETE curve it chose and the rule, never a silent stand-in.
        let semantic =
            resolve_generic_curve_decision(&conn, &well, "GR", CurveRequest::SemanticFamily)
                .unwrap()
                .expect("the family request is the rule-11 alias feature");
        assert_eq!(semantic.chosen.mnemonic, "GRN", "the concrete identity travels back");
        assert_eq!(semantic.rule, Some(CurveResolutionRule::AliasAutomatic));
        // C. Both request types agree wherever the exact mnemonic EXISTS: the semantic
        //    request prefers it over any family relative (mnemonic first, family only for
        //    what the mnemonic cannot answer).
        let gr = crate::db::upsert_curve_meta(
            &conn, &well, "RAW", "GR", Some("gapi"), Some("GR"), None, None,
        )
        .unwrap();
        crate::db::insert_curve_samples(&conn, &gr, &[1000.0, 1001.0], &[40.0, 41.0]).unwrap();
        for request in [CurveRequest::ExactMnemonic, CurveRequest::SemanticFamily] {
            let decision = resolve_generic_curve_decision(&conn, &well, "GR", request)
                .unwrap()
                .expect("GR now exists");
            assert_eq!(decision.chosen.mnemonic, "GR", "{request:?} must pick the exact curve");
        }
    }

    /// SB-DBM-030's computed-store half: a module output's missing sample (f32::NAN in the
    /// vector, per rule 2) binds SQL NULL in BOTH the current store and the archive - so at the
    /// store "no value" is never representable as a number - and the reader hands back the NaN
    /// convention with data surviving bit for bit.
    #[test]
    fn a_computed_curves_missing_sample_is_sql_null_at_the_store_and_nan_at_the_reader() {
        let conn = duckdb::Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, id, "SANDI-CNULL", None, None, None).unwrap();
        let well = id.to_string();
        let depth = [1000.0f32, 1001.0, 1002.0];
        let nan = vec![f32::NAN; 3];
        crate::db::insert_standard_curves(
            &conn, id, depth.to_vec(), vec![40.0; 3], vec![20.0; 3], vec![0.2; 3],
            vec![2.35; 3], nan.clone(), nan,
        )
        .unwrap();
        let values = [0.5f32, f32::NAN, 0.25];
        write_computed_curves_batch(&conn, &well, &depth, &[("VSH", &values[..])]).unwrap();
        for table in ["computed_curves", "computed_curves_archive"] {
            let nulls: i64 = conn
                .query_row(
                    &format!("SELECT count(*) FROM {table} WHERE well_id = ?1 AND value IS NULL"),
                    duckdb::params![well],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(nulls, 1, "{table}: the NaN sample must bind SQL NULL");
        }
        let (_, columns) =
            fetch_curve_frame(&conn, &well, &["VSH".to_string()]).unwrap();
        let vsh = &columns["VSH"];
        assert_eq!(vsh[0].to_bits(), 0.5f32.to_bits());
        assert!(vsh[1].is_nan(), "the reader hands back the NaN missing convention");
        assert_eq!(vsh[2].to_bits(), 0.25f32.to_bits());
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
            ("sandimin.rs", "SandiminRequest", true),
            ("coreimage.rs", "CoreLogSpec", true),
            ("paysummary.rs", "PaySummaryRequest", false),
            ("paysummary.rs", "CutoffSweepRequest", false),
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
                    "{file}: {name} writes curves but no longer lets the caller name the log set they are versioned into — a hardcoded destination is what this replaced"
                );
            } else {
                assert!(
                    body.contains("pub input_set:"),
                    "{file}: {name} reads curves but no longer takes an input_set, so it silently reads whatever the current values are"
                );
            }
        }
    }

    use super::*;
    use duckdb::Connection;

    /// SB-ENV-005 (DEC-031(b), signed DRAFT_ENV005 under DEC-076): the applied-step
    /// manifest rides the log-set version it describes, in the same transaction.
    #[test]
    fn a_manifest_rides_its_version_atomically_a_legacy_null_reads_unknown_and_an_unknown_schema_version_refuses_while_curves_still_read(
    ) {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let well = "55555555-5555-5555-5555-555555555555";
        conn.execute_batch(&format!(
            "INSERT INTO wells (well_id, well_name) VALUES ('{well}', 'SANDI-M1');"
        ))
        .unwrap();

        // A production (complete) write lands the manifest WITH the version - one INSERT.
        let depths = [1000.0f32, 1000.5, 1001.0];
        let values = [0.21f32, 0.19, 0.23];
        write_computed_curves_batch(&conn, well, &depths, &[("PHIE_M", &values)]).unwrap();
        let (set_id, params_json): (String, String) = conn
            .query_row(
                "SELECT set_id, params_json FROM log_sets WHERE well_id = ?1",
                params![well],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let record = get_applied_steps(&conn, &set_id).unwrap();
        let AppliedStepsRecord::Manifest { manifest } = record else {
            panic!("a manifest-era write must retrieve a manifest, got Unknown");
        };
        assert_eq!(manifest.v, APPLIED_STEPS_SCHEMA_VERSION);
        assert_eq!(manifest.steps.len(), 1, "one run states one step about itself");
        let step = &manifest.steps[0];
        assert_eq!(step.kind, "module");
        assert_eq!(step.module.as_deref(), Some("TEST_FIXTURE"));
        // The digest references the params_json on the SAME row - recomputable, never a copy.
        assert_eq!(
            step.params_digest.as_deref(),
            Some(params_digest_hex(&params_json).as_str()),
            "params_digest must be the SHA-256 of the row's own resolved parameter record"
        );
        assert!(step.outcome.is_none(), "outcome counts are omitted, never invented");

        // The derivation copies resolved inputs WITH set qualification and the rule-11 mask.
        let ancestry_spec = CurveAncestry {
            schema_version: CURVE_ANCESTRY_SCHEMA_VERSION,
            method_derivation: None,
            module: "TEST_FIXTURE".into(),
            module_version: env!("CARGO_PKG_VERSION").into(),
            inputs: vec![AncestryInput {
                well_id: well.into(),
                argument: "NPHI".into(),
                curve: "NPHI".into(),
                log_set: "RAW".into(),
                set_version: None,
                set_id: "imported".into(),
                chosen_curve_id: Some("imported:nphi".into()),
                rule: Some(CurveResolutionRule::ExplicitName),
                rejected_candidates: Vec::new(),
            }],
            parameters: Vec::new(),
            parameter_state: Some(ProvenanceAbsentState::NotApplicable),
            zone_scope: AncestryZoneScope::WholeWell,
            actor: AncestryActor {
                kind: AncestryActorKind::Automated,
                identity: "rust-test-fixture".into(),
            },
            timestamp_utc_ms: ancestry_timestamp_utc_ms().unwrap(),
            outputs: vec![AncestryOutput {
                curve: "OUT_M".into(),
                derivation: "test_fixture:OUT_M".into(),
            }],
            depth_frame: None,
            zone_set: None,
            stochastic: None,
            applied_model: None,
            physics_attributes: Vec::new(),
        };
        let mut spec = CompleteLogSetSpec::try_new("INTERP", ancestry_spec).unwrap();
        spec.storage.params_json = "{\"MASK\":\"BADHOLE\",\"OPT\":1}".into();
        let derived = derive_applied_steps(&spec);
        assert_eq!(derived.steps[0].inputs, vec!["NPHI@RAW".to_string()]);
        assert_eq!(derived.steps[0].mask.as_deref(), Some("BADHOLE"));

        // A pre-contract version reads back UNKNOWN - never an empty step list, because an
        // empty list claims "nothing was applied", which is an answer, not an absence.
        let legacy_spec = LogSetSpec {
            set_name: "LEGACY".into(),
            module: "vsh_gr".into(),
            params_json: "{}".into(),
            inputs_json: "[]".into(),
        };
        let (legacy_id, _) = create_log_set(&conn, well, &legacy_spec).unwrap();
        let legacy = get_applied_steps(&conn, &legacy_id).unwrap();
        assert_eq!(legacy, AppliedStepsRecord::Unknown);
        let view = serde_json::to_value(&legacy).unwrap();
        assert_eq!(view["state"], "unknown");
        assert!(view.get("manifest").is_none(), "unknown must not smuggle an empty step list");

        // A manifest schema version this build does not know REFUSES by name - while the
        // version's curves still read, because nothing else consults the column.
        conn.execute(
            "UPDATE log_sets SET applied_steps_json = '{\"v\":99,\"steps\":[]}' WHERE set_id = ?1",
            params![set_id],
        )
        .unwrap();
        let refusal = get_applied_steps(&conn, &set_id).expect_err("v99 must refuse").to_string();
        assert!(refusal.contains("v99"), "the refusal names the stored version: {refusal}");
        assert!(refusal.contains("SB-ENV-005"), "and cites the contract: {refusal}");
        let still_read: i64 = conn
            .query_row(
                "SELECT count(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'PHIE_M'",
                params![well],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(still_read, 3, "a manifest refusal must never take the curves with it");

        // Structural second side: a manifest exists only ON a version row, so asking for one
        // without its version refuses naming the set.
        let absent = "99999999-9999-9999-9999-999999999999";
        let missing = get_applied_steps(&conn, absent).expect_err("must refuse").to_string();
        assert!(missing.contains("no log-set version"), "{missing}");
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

        let explicit =
            TrackCurveRequest { curve_name: "GR".into(), set_name: Some("WIRE_ALT".into()), class_curve: false };
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
        assert!(visible_packed[..visible_n].iter().all(|d| *d >= 1000.0 && *d < 1002.5));

        let current = TrackCurveRequest { curve_name: "GR".into(), set_name: None, class_curve: false };
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

    /// AUDIT-2026-08-20 finding 26. The screen decimated a FACIES curve with min/max and the
    /// print did not, so the facies column QC'd at whole-well zoom was not the column that
    /// shipped. Min/max is honest for a measurement - the extremes ARE the envelope - but a
    /// class index has no envelope, so the min and max facies in a bucket are two arbitrary
    /// numbers: one facies-7 sample painted metres of facies 7, and runs of the dominant class
    /// shredded into alternating pairs.
    ///
    /// Pinned from BOTH sides, because the tempting fix - decimate a class curve by some SECOND
    /// rule, mode or first-in-bucket - passes the first half and leaves screen and print two
    /// algorithms that have to be kept in step. They must be BIT-IDENTICAL, and an ordinary
    /// continuous curve must still decimate or the viewer loses the reason decimation exists.
    #[test]
    fn a_class_curve_reaches_the_screen_exactly_as_it_reaches_the_print_while_a_measurement_still_decimates() {
        use crate::db;
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well = uuid::Uuid::new_v4();
        db::insert_well(&conn, well, "SANDI-DECIM", None, None, None).unwrap();
        let wid = well.to_string();

        // A whole-well fetch: 4000 samples onto a 400-pixel canvas is ~10 to a bucket, the
        // regime the finding describes. FACIES runs 40 samples of class 2 then one sample of
        // class 7 - a thin bed the min/max rule would smear across the whole bucket.
        let n = 4000usize;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.1).collect();
        let facies: Vec<f32> = (0..n).map(|i| if i % 41 == 40 { 7.0 } else { 2.0 }).collect();
        let gr: Vec<f32> = (0..n).map(|i| 40.0 + (i % 100) as f32).collect();
        db::insert_standard_curves(
            &conn, well, depths.clone(), gr.clone(),
            vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n],
        )
        .unwrap();
        let meta = db::upsert_curve_meta(&conn, &wid, "RAW", "FACIES", None, None, Some("test"), None).unwrap();
        db::insert_curve_samples(&conn, &meta, &depths, &facies).unwrap();

        let req = |name: &str, class_curve: bool| TrackCurveRequest {
            curve_name: name.into(),
            set_name: None,
            class_curve,
        };
        let screen = |r: &[TrackCurveRequest]| fetch_track_data(&conn, &wid, r, 400, None, None).unwrap();
        let print = |r: &[TrackCurveRequest]| fetch_track_frames(&conn, &wid, r, None, None).unwrap();

        // A - the class curve reaches the screen exactly as it reaches the print.
        let requests = [req("FACIES", true)];
        let on_screen = &screen(&requests)[0];
        let in_print = &print(&requests)[0];
        assert_eq!(
            on_screen.point_count,
            in_print.depth.len(),
            "a class curve must not be decimated - screen {} samples, print {}",
            on_screen.point_count,
            in_print.depth.len()
        );
        let packed: &[f32] = bytemuck::cast_slice(&on_screen.data);
        assert_eq!(
            &packed[on_screen.point_count..],
            in_print.value.as_slice(),
            "and the values must be bit-identical, not merely the same length"
        );
        assert_eq!(&packed[..on_screen.point_count], in_print.depth.as_slice());

        // B - and an ordinary MEASUREMENT still decimates, or the viewer has lost the reason
        // decimation exists at all.
        let gr_screen = &screen(&[req("GR", false)])[0];
        assert!(
            gr_screen.point_count < n,
            "GR must still be decimated at whole-well zoom, got {} of {n}",
            gr_screen.point_count
        );
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

        let first = resolve_generic_curve_decision(&conn, &well_id, "CALI", CurveRequest::SemanticFamily)
            .unwrap()
            .expect("the family is present");
        assert_eq!(first.chosen.curve_id, raw_family);
        assert_eq!(first.rule, Some(CurveResolutionRule::WorkingInputSet));
        assert_eq!(first.rejected.len(), 1);
        assert_eq!(first.rejected[0].curve_id, attached_exact);
        let (_, columns) = fetch_curve_frame(&conn, &well_id, &["CALI".into()]).unwrap();
        assert_eq!(columns["CALI"], [8.0, 8.0, 8.0]);

        db::delete_generic_curve(&conn, &raw_family).unwrap();
        let fallback = resolve_generic_curve_decision(&conn, &well_id, "CALI", CurveRequest::SemanticFamily)
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

}
