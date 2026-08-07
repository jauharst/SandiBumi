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
pub(crate) const STANDARD_COLUMNS: [&str; 7] = ["DEPTH", "GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];

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

/// Looks up a curve in the generic store (`curve_meta`/`curve_samples`) by
/// mnemonic-or-family and aligns its samples onto the depth grid. Set RAW has ABSOLUTE
/// priority — any RAW match (mnemonic or family) beats any match from another set, so
/// every resolution that worked before import-sets landed is byte-identical. Only when
/// RAW has no candidate at all does the search widen to the well's other sets (attached
/// deliveries like FPROOH/MULTIMIN — T-IMP-02), ordered by set_name for determinism when
/// two sets carry the same mnemonic. Within a set: exact mnemonic matches win over family
/// matches; among exact-mnemonic matches a user-PINNED curve wins, else the base run
/// (lowest `run_no`, NULL first). `pinned` is scoped per (well, set, mnemonic) by
/// `db::promote_generic_curve`, so the tiebreak applies it ONLY when the request is that exact
/// mnemonic (the `CASE WHEN upper(mnemonic)=?2` guard) — a family-name request must rank purely
/// by run_no, or a pin placed on one member mnemonic would hijack a different member of the same
/// family. A final `curve_id` key keeps the pick deterministic when every other key ties (e.g.
/// two same-family base-run curves). Promote a curve in the Curve Catalog to resolve DLIS/LAS
/// shadowing.
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
             WHERE well_id = ?1
               AND (upper(mnemonic) = ?2 OR upper(family) = ?2)
             ORDER BY (set_name = 'RAW') DESC,
                      (upper(mnemonic) = ?2) DESC,
                      (CASE WHEN upper(mnemonic) = ?2 THEN COALESCE(pinned, 0) ELSE 0 END) DESC,
                      set_name,
                      run_no NULLS FIRST,
                      curve_id
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
    let (mut depth, mut columns) = fetch_curve_frame(conn, well_id, curve_names)?;
    let Some(set_name) = input_set.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok((depth, columns));
    };

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
               AND upper(curve_name) IN (SELECT DISTINCT upper(curve_name) FROM computed_curves_archive WHERE set_id = ?1)",
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
            if let Err(e) = write_equation_output(&conn, well_id, &depth, equation, &output) {
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

    /// DLIS/LAS same-mnemonic shadow resolution: the LAS curve (run NULL) wins by DEFAULT
    /// (backward-compatible — nothing pinned); a user PROMOTE flips the winner to the DLIS
    /// curve and is at-most-one-pinned; DELETE of the winner falls back to the surviving
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

        // Two RAW 'PEF' curves colliding on the mnemonic: LAS (run NULL, value 1.0) and
        // DLIS (run 0, value 2.0).
        let las = db::upsert_curve_meta(&conn, &wid, "RAW", "PEF", Some("B/E"), Some("PEF"), Some("LAS import"), None).unwrap();
        db::insert_curve_samples(&conn, &las, &depths, &[1.0, 1.0, 1.0]).unwrap();
        let dlis = db::upsert_curve_meta(&conn, &wid, "RAW", "PEF", Some("B/E"), Some("PEF"), Some("DLIS import"), Some(0)).unwrap();
        db::insert_curve_samples(&conn, &dlis, &depths, &[2.0, 2.0, 2.0]).unwrap();

        let pef = |c: &Connection| fetch_curve_frame(c, &wid, &["PEF".to_string()]).unwrap().1["PEF"][0];

        assert_eq!(pef(&conn), 1.0, "LAS (NULL run) wins by default");
        db::promote_generic_curve(&conn, &dlis).unwrap();
        assert_eq!(pef(&conn), 2.0, "promoted DLIS wins");
        db::promote_generic_curve(&conn, &las).unwrap();
        assert_eq!(pef(&conn), 1.0, "re-promoted LAS wins again (at-most-one-pinned)");
        db::delete_generic_curve(&conn, &las).unwrap();
        assert_eq!(pef(&conn), 2.0, "after deleting the winner, the sibling resolves");

        let cat = db::list_generic_curve_catalog(&conn, &wid).unwrap();
        assert!(cat.iter().all(|c| c.curve_id != las), "deleted curve gone from catalog");
        assert!(cat.iter().any(|c| c.curve_id == dlis && !c.pinned), "DLIS present and unpinned");
    }

    /// A user PIN on one family member must not hijack a FAMILY-name request that resolves a
    /// DIFFERENT member of the same family. `pinned` is scoped per (well, set, mnemonic) by
    /// `db::promote_generic_curve`; the resolver applies it only to exact-mnemonic matches, so a
    /// family request still ranks by run_no and the base run (NULL) keeps winning after a sibling
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
        // family string "CALI": HCAL (base run, NULL) and DCAL (run 0).
        let hcal = db::upsert_curve_meta(&conn, &wid, "RAW", "HCAL", Some("in"), Some("CALI"), Some("LAS import"), None).unwrap();
        db::insert_curve_samples(&conn, &hcal, &depths, &[8.0, 8.0, 8.0]).unwrap();
        let dcal = db::upsert_curve_meta(&conn, &wid, "RAW", "DCAL", Some("in"), Some("CALI"), Some("DLIS import"), Some(0)).unwrap();
        db::insert_curve_samples(&conn, &dcal, &depths, &[9.0, 9.0, 9.0]).unwrap();

        let cali = |c: &Connection| fetch_curve_frame(c, &wid, &["CALI".to_string()]).unwrap().1["CALI"][0];

        // Base run (HCAL, NULL) wins the family bucket by default.
        assert_eq!(cali(&conn), 8.0, "base-run family member wins the family request by default");
        // Promoting DCAL resolves a DCAL-vs-DCAL mnemonic shadow — it must NOT hijack the CALI
        // family request, because DCAL's mnemonic != the requested family name. (Pre-fix this
        // returned 9.0: pinned sorted ahead of run_no even for the family path.)
        db::promote_generic_curve(&conn, &dcal).unwrap();
        assert_eq!(cali(&conn), 8.0, "a pin on a sibling mnemonic must not hijack the family bucket");
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
        let r = run_equation(&dbm, &bad, &[w.clone()], None);
        assert!(r[0].error.is_some(), "all-NaN equation must report an error");

        // A resolvable input computes a real value → success with the full sample count.
        let good = eq("good", "GR", "GRSCALE", "gr / 100.0");
        let r = run_equation(&dbm, &good, &[w], None);
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
        let res = run_equation(&dbm, &eq, &wells, None);

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
        let res = run_equation(&dbm, &eq, &[w.clone()], None);

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
        let res = run_equation(&dbm, &all, &[w], None);
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
        let res = run_equation(&dbm, &eq, &[w], None);

        assert!(res[0].error.is_none(), "{:?}", res[0].error);
        assert_eq!(res[0].rows_written, n);
        assert!(
            res[0].note.is_none(),
            "missing inputs are not a script failure and must not warn: {:?}",
            res[0].note
        );
    }
}
