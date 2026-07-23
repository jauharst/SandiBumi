use crate::db;
use crate::parsers::{self, CurveColumns, ParseError};
use duckdb::{params, Connection};
use rayon::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub path: String,
    pub well_id: Option<String>,
    pub well_name: Option<String>,
    pub rows: usize,
    /// Non-fatal note for a successful import, e.g. rows dropped for a bad/duplicate depth.
    pub warning: Option<String>,
    pub error: Option<String>,
}

/// Parses every given LAS file concurrently via `rayon` (CPU-bound), then inserts each
/// well and its curves into DuckDB sequentially — the connection is behind a single lock,
/// so only the parsing step benefits from parallelism, which is also the expensive part.
pub fn import_las_files(
    conn: &Connection,
    paths: &[String],
    progress: Option<&crate::jobs::JobHandle>,
) -> Vec<ImportResult> {
    let parsed: Vec<(String, Result<(String, CurveColumns), ParseError>)> = paths
        .par_iter()
        .map(|path| {
            let result = (|| {
                let well_name = parsers::extract_well_name(path)?;
                let columns = parsers::parse_las_2(path)?;
                Ok::<_, ParseError>((well_name, columns))
            })();
            (path.clone(), result)
        })
        .collect();

    parsed
        .into_iter()
        .map(|(path, result)| {
            // Cancel before the DB write, so clicking Cancel actually stops wells being created.
            // Without this the flag was flipped, every remaining file was still inserted, and the
            // job was then labelled "Cancelled" — the user was told the import stopped while the
            // project filled up with unwanted wells. The parse pass above has already run by this
            // point (it is one up-front par_iter), so cancel stops the writes, not the parsing.
            if progress.map_or(false, |p| p.is_cancelled()) {
                if let Some(p) = progress {
                    p.finish_item(&path, crate::jobs::ItemState::Warned, Some("cancelled".into()));
                }
                return ImportResult {
                    path: path.clone(),
                    well_id: None,
                    well_name: None,
                    rows: 0,
                    warning: Some("cancelled before import".into()),
                    error: None,
                };
            }
            if let Some(p) = progress {
                let base = path.rsplit(['/', '\\']).next().unwrap_or(&path);
                p.set_current(Some(format!("Importing {base}")));
                p.start_item(&path);
            }
            let out = match result {
                Ok((well_name, columns)) => insert_parsed_well(conn, path.clone(), well_name, columns),
                Err(e) => ImportResult { path: path.clone(), well_id: None, well_name: None, rows: 0, warning: None, error: Some(e.to_string()) },
            };
            if let Some(p) = progress {
                let (state, msg) = if out.error.is_some() {
                    (crate::jobs::ItemState::Failed, out.error.clone())
                } else if out.warning.is_some() {
                    (crate::jobs::ItemState::Warned, out.warning.clone())
                } else {
                    (crate::jobs::ItemState::Ok, None)
                };
                p.finish_item(&path, state, msg);
            }
            out
        })
        .collect()
}

fn insert_parsed_well(conn: &Connection, path: String, well_name: String, mut columns: CurveColumns) -> ImportResult {
    let well_id = Uuid::new_v4();
    // Drop non-finite / duplicate depths so the (well_id, depth) PK can't trip and abort the
    // whole file (which would also orphan the well row); report what was removed.
    let report = parsers::sanitize_curve_columns(&mut columns);
    let rows = columns.depth.len();

    // Every row dropped (all depths missing/duplicate — e.g. an unrecognized index whose
    // column 0 is entirely the null sentinel): don't commit a curve-less orphan well, error.
    if rows == 0 {
        return ImportResult {
            path,
            well_id: None,
            well_name: None,
            rows: 0,
            warning: None,
            error: Some(format!(
                "no importable rows: {} had missing depth, {} duplicated an earlier depth",
                report.nonfinite, report.duplicate
            )),
        };
    }

    // A LAS index is monotonic by spec; a non-monotonic depth after sanitation usually means
    // column 0 was not the true index (an unrecognized-mnemonic file whose first curve is data,
    // imported as depth via the column-0 fallback) — surface it rather than import silently.
    let non_monotonic = columns.depth.windows(2).any(|w| w[0] < w[1])
        && columns.depth.windows(2).any(|w| w[0] > w[1]);
    let mut notes: Vec<String> = Vec::new();
    if !report.is_clean() {
        notes.push(format!(
            "dropped {} row(s) with missing depth and {} with duplicate depth",
            report.nonfinite, report.duplicate
        ));
    }
    if non_monotonic {
        notes.push("depth index is non-monotonic — column 0 may not be the true depth curve".to_string());
    }
    // A well of the same (normalized) name already exists. LAS import still creates a SEPARATE
    // record here — reuse/merge is a deliberate action that needs a user confirmation flow, not
    // an automatic side effect — but warn so a corrected re-delivery (or the same file picked
    // twice) doesn't silently fragment a well's curves across two disconnected records.
    let name_norm = well_name.trim().to_uppercase();
    let dup_exists = conn
        .query_row(
            "SELECT 1 FROM wells WHERE upper(trim(well_name)) = ?1 LIMIT 1",
            params![name_norm],
            |_| Ok(()),
        )
        .is_ok();
    if dup_exists {
        notes.push(format!(
            "a well named '{well_name}' already exists — imported as a separate record"
        ));
    }
    let warning = (!notes.is_empty()).then(|| notes.join("; "));

    // Well row + standard curves as one transaction: a failure rolls the well row back
    // instead of stranding a curve-less orphan (with_txn = BEGIN/COMMIT/ROLLBACK).
    let result: db::DbResult<()> = db::with_txn(conn, |conn| {
        db::insert_well(conn, well_id, &well_name, None, None, None)?;
        db::insert_standard_curves(
            conn,
            well_id,
            columns.depth,
            columns.gr,
            columns.res,
            columns.nphi,
            columns.rhob,
            columns.dt,
            columns.sp,
        )?;
        Ok(())
    });

    match result {
        Ok(()) => {
            // Phase 6: additionally load *every* curve from the file into the generic
            // store (set RAW), so PEF/CALI/multiple-runs — anything beyond the fixed 6 —
            // is available even though the legacy `standard_curves` path above still feeds
            // the current UI. A failure here must not fail the whole import (the standard
            // curves are already in), so it's logged, not propagated.
            if let Err(e) = import_all_curves_into_generic_store(conn, &well_id.to_string(), &path) {
                eprintln!("warning: generic-store import for {well_name} failed (standard curves still imported): {e}");
            }
            ImportResult { path, well_id: Some(well_id.to_string()), well_name: Some(well_name), rows, warning, error: None }
        }
        Err(e) => ImportResult { path, well_id: None, well_name: None, rows: 0, warning: None, error: Some(e.to_string()) },
    }
}

/// Re-reads a LAS file keeping all curves and writes each into `curve_meta`/`curve_samples`
/// as set RAW, tagging family (via the mnemonic dictionary) and normalizing units where a
/// conversion is known. The unit stored is the canonical one when converted, else the
/// file's original unit.
pub fn import_all_curves_into_generic_store(conn: &Connection, well_id: &str, path: &str) -> db::DbResult<()> {
    let mut frame = match parsers::parse_las_2_all(path) {
        Ok(f) => f,
        Err(e) => return Err(db::DbError::LengthMismatch(format!("parse_las_2_all: {e}"))),
    };
    // curve_samples has PK (curve_id, depth) just like standard_curves, so the same non-finite
    // / duplicate depths the standard-curves path drops would otherwise abort each curve's
    // insert here — silently, since this whole import is best-effort (its Err is only logged).
    // Sanitize depth + every curve in lockstep before writing (identical keep-set to the
    // standard path, so both stores hold the same rows for the same file).
    parsers::sanitize_las_frame(&mut frame);
    if frame.depth.is_empty() {
        return Ok(());
    }

    for raw in &frame.curves {
        let mut values = raw.values.clone();
        // Align to the depth column length (defensive: malformed files can short a column).
        if values.len() != frame.depth.len() {
            values.resize(frame.depth.len(), f32::NAN);
        }
        let fam = crate::curves::family_for(&raw.mnemonic);
        let family = fam.map(|f| f.family);
        let mut unit = raw.unit.clone();
        if let Some(f) = fam {
            if crate::curves::convert_to_canonical(f.family, raw.unit.as_deref(), &mut values) {
                unit = Some(f.canonical_unit.to_string());
            }
        }
        let curve_id =
            db::upsert_curve_meta(conn, well_id, "RAW", &raw.mnemonic, unit.as_deref(), family, Some("LAS import"), None)?;
        db::insert_curve_samples(conn, &curve_id, &frame.depth, &values)?;
    }
    Ok(())
}

/// Parses a deviation-survey CSV (columns MD/INC/AZI, alias-tolerant) and stores the
/// computed minimum-curvature TVD/TVDSS in `well_path` for one well. `datum_elevation`
/// (KB above MSL) is used for TVDSS; if omitted, the well's `kb` is used, else 0.
pub fn import_deviation_csv(
    conn: &Connection,
    well_id: &str,
    path: &str,
    datum_elevation: Option<f32>,
) -> CoreImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")) };
    }

    let survey = match parsers::parse_deviation_csv(path) {
        Ok(s) => s,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()) },
    };
    if survey.md.is_empty() {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some("no survey stations found".into()) };
    }

    let datum = datum_elevation.unwrap_or_else(|| {
        conn.query_row("SELECT kb FROM wells WHERE well_id = ?1", params![well_id], |r| r.get::<_, Option<f32>>(0))
            .ok()
            .flatten()
            .unwrap_or(0.0)
    });
    let stations = crate::deviation::minimum_curvature(&survey.md, &survey.inc, &survey.azi, datum);
    let rows = stations.len();
    match db::insert_well_path(conn, well_id, &stations) {
        Ok(()) => {
            // Materialize TVD/TVDSS onto the log grid so height modules (sw_height, the SHF
            // fits, the TVDSS correlation view) can fetch them by name. Best-effort: the
            // survey itself is already saved; a well with no logs yet is a no-op (0 samples)
            // and the user can recompute via `materialize_tvd` after importing logs.
            let _ = materialize_tvd_curves(conn, well_id);
            CoreImportResult { path: path.to_string(), rows, error: None }
        }
        Err(e) => CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()) },
    }
}

/// Resamples a well's deviation survey (`well_path`) onto its standard-curve depth grid and
/// writes the result as fetchable `TVD` and `TVDSS` computed curves — the bridge that lets
/// `sw_height`'s TVD input, the SHF-fitting modules' TVDSS input, and the correlation TVDSS
/// depth-mode consume the survey. Returns the number of samples written (per curve); 0 when
/// the well has no survey (vertical — TVD == MD, and callers already fall back to MD) or no
/// logs yet (no depth grid to hang the curves on). Refreshes its OWN prior computed TVD/TVDSS
/// in place, but NEVER overwrites a TVD/TVDSS the user imported from a vendor LAS/DLIS (see the
/// import guard below), so a re-import or a KB edit + recompute is safe.
pub fn materialize_tvd_curves(conn: &Connection, well_id: &str) -> db::DbResult<usize> {
    let stations = db::get_well_path(conn, well_id)?;
    if stations.is_empty() {
        return Ok(0);
    }
    let path: Vec<crate::deviation::Station> = stations
        .iter()
        .map(|s| crate::deviation::Station { md: s.md, inc: s.inc, azi: s.azi, tvd: s.tvd, tvdss: s.tvdss })
        .collect();
    // Empty name list → just the standard depth grid for this well.
    let (depth, _cols) = crate::equations::fetch_curve_frame(conn, well_id, &[])?;
    if depth.is_empty() {
        return Ok(0);
    }
    let mut tvd = Vec::with_capacity(depth.len());
    let mut tvdss = Vec::with_capacity(depth.len());
    for &d in &depth {
        let (t, ss) = crate::deviation::sample_at(&path, d);
        tvd.push(t);
        tvdss.push(ss);
    }
    // A survey-derived COMPUTED curve outranks the generic RAW store in fetch_curve_frame, so
    // writing TVD/TVDSS unconditionally would SILENTLY shadow an authoritative curve the user
    // imported from a vendor LAS/DLIS — with a possibly wrong datum (a well with no KB falls
    // back to a sea-level datum → TVDSS = −TVD) or NaN outside the survey's MD range, and no
    // recourse via the Curve Catalog's Promote (it is disabled on a "served by computed" row).
    // So: only materialize a name the well does NOT already resolve from an import, and clear
    // any prior survey-derived computed curve when an import IS present, so the import wins.
    let mut written = 0usize;
    for (name, values) in [("TVD", &tvd), ("TVDSS", &tvdss)] {
        let imported: bool = conn
            .query_row(
                "SELECT 1 FROM curve_meta WHERE well_id = ?1 AND set_name = 'RAW'
                   AND (upper(mnemonic) = upper(?2) OR upper(family) = upper(?2)) LIMIT 1",
                params![well_id, name],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if imported {
            crate::equations::delete_computed_curve(conn, well_id, name)?;
        } else {
            crate::equations::write_computed_curve(conn, well_id, &depth, name, values)?;
            written = depth.len();
        }
    }
    Ok(written)
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreImportResult {
    pub path: String,
    pub rows: usize,
    pub error: Option<String>,
}

/// Parses a routine-core-analysis CSV and replaces the given well's core plug data.
/// Unlike LAS import, this attaches to an existing well rather than creating one.
pub fn import_core_csv(conn: &Connection, well_id: &str, path: &str) -> CoreImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return CoreImportResult { path: path.to_string(), rows: 0, error: Some(format!("unknown well '{well_id}'")) };
    }

    let columns = match parsers::parse_core_csv(path) {
        Ok(c) => c,
        Err(e) => return CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()) },
    };
    let rows = columns.depth.len();
    match db::insert_core_data(conn, well_id, &columns.depth, &columns.cpor, &columns.cperm, &columns.cgd, &columns.csw) {
        Ok(()) => CoreImportResult { path: path.to_string(), rows, error: None },
        Err(e) => CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()) },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalImportResult {
    pub path: String,
    pub rows: usize,
    /// Leverett-J fit over the imported points at the given lab IFT, when solvable —
    /// reported straight back to the import dialog so the user can carry SWH_A/SWH_B
    /// into the sw_height module.
    pub fit: Option<crate::satheight::LeverettFit>,
    pub error: Option<String>,
}

/// Parses a SCAL capillary-pressure CSV (flat/long shape), replaces the well's `scal_pc`
/// rows, and fits the Leverett-J function (Sw = A·J^B) over the points at `ift_lab`
/// (sigma·cosθ of the lab fluid system, dyn/cm — e.g. 72 air-brine, 367 air-mercury).
pub fn import_scal_csv(conn: &Connection, well_id: &str, path: &str, ift_lab: f64) -> ScalImportResult {
    import_scal_files(conn, well_id, &[path.to_string()], "long", "", ift_lab)
}

/// Multi-file, multi-format SCAL Pc import. Each file is parsed with `format` — "long"
/// (flat Pc/Sw CSV), "porous_plate" (Corelab-style wide table: pressure columns × plug
/// rows), "centrifuge" (per-plug key-value blocks + Pc/Sw tables), or "auto" to sniff
/// each file — so a set of single-plug centrifuge exports imports in one shot. The
/// combined records REPLACE the well's `scal_pc` rows (same discipline as re-import),
/// then the Leverett-J function is fitted over all points at `ift_lab`. `system` labels
/// every stored point with the lab fluid system ('air_brine', 'hg_air', ...; "" = not
/// recorded) alongside `ift_lab`, so later standardization (Thomeer, J-from-SCAL) knows
/// which system each point was measured in.
pub fn import_scal_files(
    conn: &Connection,
    well_id: &str,
    paths: &[String],
    format: &str,
    system: &str,
    ift_lab: f64,
) -> ScalImportResult {
    let joined = paths.join("; ");
    let fail = |error: String| ScalImportResult { path: joined.clone(), rows: 0, fit: None, error: Some(error) };

    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return fail(format!("unknown well '{well_id}'"));
    }
    if paths.is_empty() {
        return fail("no files selected".into());
    }

    let mut records: Vec<parsers::ScalPcRecord> = Vec::new();
    for path in paths {
        let base = path.rsplit(['/', '\\']).next().unwrap_or(path);
        let fmt = if format == "auto" {
            match parsers::sniff_scal_format(path) {
                Ok(f) => f,
                Err(e) => return fail(format!("{base}: {e}")),
            }
        } else {
            format
        };
        let parsed = match fmt {
            "long" => parsers::parse_scal_csv(path),
            "porous_plate" => parsers::parse_scal_wide_csv(path),
            "centrifuge" => parsers::parse_scal_centrifuge_csv(path),
            other => return fail(format!("unknown SCAL format '{other}'")),
        };
        match parsed {
            Ok(mut r) => records.append(&mut r),
            Err(e) => return fail(format!("{base} ({fmt}): {e}")),
        }
    }
    // A structurally-valid file can still yield zero points (header-only export, cells in
    // a format no rule parses). Refuse the replace-write then — otherwise a degenerate
    // re-import would silently DELETE the well's existing SCAL dataset.
    if records.is_empty() {
        return fail(
            "no Pc/Sw data rows parsed from the selected file(s) — nothing was imported and the well's existing SCAL points are untouched (check the file format choice)".into(),
        );
    }

    let sys: Option<String> = if system.trim().is_empty() { None } else { Some(system.trim().to_string()) };
    let rows: Vec<db::ScalPcRow> = records
        .iter()
        .map(|r| db::ScalPcRow {
            sample_no: r.sample_no,
            depth: r.depth,
            perm: r.perm,
            poro: r.poro,
            pc: r.pc,
            sw: r.sw,
            system: sys.clone(),
            ift: Some(ift_lab as f32),
        })
        .collect();
    if let Err(e) = db::insert_scal_pc(conn, well_id, &rows) {
        return fail(e.to_string());
    }

    let points: Vec<crate::satheight::ScalPoint> = records
        .iter()
        .map(|r| crate::satheight::ScalPoint { pc: r.pc, sw: r.sw, perm: r.perm, poro: r.poro })
        .collect();
    let fit = crate::satheight::fit_leverett_j(&points, ift_lab);
    ScalImportResult { path: joined, rows: rows.len(), fit, error: None }
}

#[derive(Debug, Clone, Serialize)]
pub struct TopsImportResult {
    pub path: String,
    pub tops_written: usize,
    pub wells_matched: usize,
    /// Well names in the file that matched nothing in the project (rows skipped).
    pub unmatched_wells: Vec<String>,
    pub error: Option<String>,
}

/// Imports formation tops from a CSV/TXT file. Files with a WELL column update every
/// matching well (name match, case-insensitive); files without one need
/// `default_well_id` (the selected well). Tops upsert by (well, name) — re-import
/// updates depths, existing colors are kept.
pub fn import_tops_file(conn: &Connection, default_well_id: Option<&str>, path: &str) -> TopsImportResult {
    let fail = |e: String| TopsImportResult {
        path: path.to_string(),
        tops_written: 0,
        wells_matched: 0,
        unmatched_wells: vec![],
        error: Some(e),
    };
    let (has_well_column, records) = match parsers::parse_tops_file(path) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };

    // Project well-name → id map (upper-trimmed).
    let mut name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut stmt = match conn.prepare("SELECT well_name, well_id FROM wells") {
            Ok(s) => s,
            Err(e) => return fail(e.to_string()),
        };
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        });
        match rows {
            Ok(rows) => {
                for r in rows.flatten() {
                    name_to_id.insert(r.0.trim().to_uppercase(), r.1);
                }
            }
            Err(e) => return fail(e.to_string()),
        }
    }

    // All-or-nothing: a mid-file DB error must not leave some tops written and others not
    // (which would otherwise report tops_written=0 while rows are already persisted). Mirrors
    // import_locations_file below.
    if let Err(e) = conn.execute_batch("BEGIN") {
        return fail(e.to_string());
    }
    let mut written = 0usize;
    let mut wells_hit: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unmatched: Vec<String> = Vec::new();
    let mut blank_rows = 0usize;
    for rec in &records {
        let well_id = match &rec.well {
            Some(name) => match name_to_id.get(&name.trim().to_uppercase()) {
                Some(id) => id.clone(),
                None => {
                    let label = name.trim().to_string();
                    if !unmatched.contains(&label) {
                        unmatched.push(label);
                    }
                    continue;
                }
            },
            // File HAS a WELL column but this row's cell is blank/ragged — skip it; misrouting a
            // blank-cell top to the selected well would silently attach it to an unrelated well.
            None if has_well_column => {
                blank_rows += 1;
                continue;
            }
            // Genuinely column-less (single-well) file → the dialog's selected well.
            None => match default_well_id {
                Some(id) => id.to_string(),
                None => {
                    let _ = conn.execute_batch("ROLLBACK");
                    return fail("file has no WELL column — select a well first".into());
                }
            },
        };
        match db::upsert_top(conn, &well_id, &rec.top_name, rec.depth, None) {
            Ok(()) => {
                written += 1;
                wells_hit.insert(well_id);
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return fail(e.to_string());
            }
        }
    }
    if let Err(e) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return fail(e.to_string());
    }
    // Surface dropped blank-WELL rows so the skip is never silent.
    if blank_rows > 0 {
        unmatched.push(format!("{blank_rows} blank-WELL row(s)"));
    }
    TopsImportResult {
        path: path.to_string(),
        tops_written: written,
        wells_matched: wells_hit.len(),
        unmatched_wells: unmatched,
        error: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LocationsImportResult {
    pub path: String,
    pub wells_located: usize,
    /// Well names in the file that matched nothing in the project (rows skipped).
    pub unmatched_wells: Vec<String>,
    pub error: Option<String>,
}

/// Imports well surface locations from a CSV/TXT file. Files with a WELL column locate
/// every matching well (name match, case-insensitive); files without one locate
/// `default_well_id` (the selected well). `default_zone` fills the UTM zone for rows that
/// carry no ZONE column value (the dialog's chosen zone). Re-import overwrites a well's
/// previous location.
pub fn import_locations_file(
    conn: &Connection,
    default_well_id: Option<&str>,
    default_zone: Option<&str>,
    path: &str,
) -> LocationsImportResult {
    let fail = |e: String| LocationsImportResult {
        path: path.to_string(),
        wells_located: 0,
        unmatched_wells: vec![],
        error: Some(e),
    };
    let (has_well_column, records) = match parsers::parse_locations_file(path) {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };

    // Project well-name → id map (upper-trimmed), same convention as the tops importer.
    let mut name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut stmt = match conn.prepare("SELECT well_name, well_id FROM wells") {
            Ok(s) => s,
            Err(e) => return fail(e.to_string()),
        };
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)));
        match rows {
            Ok(rows) => {
                for r in rows.flatten() {
                    name_to_id.insert(r.0.trim().to_uppercase(), r.1);
                }
            }
            Err(e) => return fail(e.to_string()),
        }
    }

    // All-or-nothing: a mid-file DB error must not leave some wells relocated and others not
    // (which would otherwise report wells_located = 0 while rows are already persisted).
    if let Err(e) = conn.execute_batch("BEGIN") {
        return fail(e.to_string());
    }
    let mut located: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unmatched: Vec<String> = Vec::new();
    let mut blank_rows = 0usize;
    for rec in &records {
        let well_id = match &rec.well {
            Some(name) => match name_to_id.get(&name.trim().to_uppercase()) {
                Some(id) => id.clone(),
                None => {
                    let label = name.trim().to_string();
                    if !unmatched.contains(&label) {
                        unmatched.push(label);
                    }
                    continue;
                }
            },
            // File HAS a WELL column but this row's cell is blank/ragged — a dropped row, not
            // a single-well file. Skip it; misrouting it to the selected well would silently
            // overwrite an unrelated well's real surface location.
            None if has_well_column => {
                blank_rows += 1;
                continue;
            }
            // Genuinely column-less (single-well) file → the dialog's selected well.
            None => match default_well_id {
                Some(id) => id.to_string(),
                None => {
                    let _ = conn.execute_batch("ROLLBACK");
                    return fail("file has no WELL column — select a well first".into());
                }
            },
        };
        let zone = rec.zone.as_deref().or(default_zone);
        if let Err(e) = db::set_well_location(conn, &well_id, Some(rec.x), Some(rec.y), zone) {
            let _ = conn.execute_batch("ROLLBACK");
            return fail(e.to_string());
        }
        located.insert(well_id);
    }
    if let Err(e) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return fail(e.to_string());
    }
    // Surface dropped blank-WELL rows so the skip is never silent.
    if blank_rows > 0 {
        unmatched.push(format!("{blank_rows} blank-WELL row(s)"));
    }
    LocationsImportResult {
        path: path.to_string(),
        wells_located: located.len(),
        unmatched_wells: unmatched,
        error: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuxImportResult {
    pub path: String,
    pub dataset: String,
    pub rows: usize,
    /// Value columns found in the file (QUARTZ, STATUS, …).
    pub items: Vec<String>,
    pub error: Option<String>,
}

/// Imports a tops-style dataset (petrography / XRD / perforations) for one well,
/// replacing that well's previous rows of the same dataset. Numeric cells land in
/// value_num, everything else in value_text.
pub fn import_aux_file(conn: &Connection, well_id: &str, dataset: &str, path: &str) -> AuxImportResult {
    let fail = |e: String| AuxImportResult {
        path: path.to_string(),
        dataset: dataset.to_string(),
        rows: 0,
        items: vec![],
        error: Some(e),
    };
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return fail(format!("unknown well '{well_id}'"));
    }
    let dataset = dataset.trim().to_uppercase();
    if dataset.is_empty() {
        return fail("dataset name is empty".into());
    }

    let data = match parsers::parse_interval_file(path) {
        Ok(d) => d,
        Err(e) => return fail(e.to_string()),
    };
    let mut rows: Vec<db::AuxRow> = Vec::new();
    for (top, base, values) in &data.rows {
        for (item, raw) in data.items.iter().zip(values) {
            let Some(raw) = raw else { continue };
            let num = raw.replace(',', ".").parse::<f32>().ok();
            rows.push(db::AuxRow {
                dataset: dataset.clone(),
                depth_top: *top,
                depth_base: *base,
                item: item.clone(),
                value_num: num,
                value_text: if num.is_some() { None } else { Some(raw.clone()) },
            });
        }
    }
    let n = rows.len();
    match db::insert_aux_data(conn, well_id, &dataset, &rows) {
        Ok(()) => AuxImportResult {
            path: path.to_string(),
            dataset,
            rows: n,
            items: data.items,
            error: None,
        },
        Err(e) => fail(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn core_import_roundtrip_and_replace() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "BALAM-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        let path = std::env::temp_dir().join("arshilla_core_roundtrip.csv");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"DEPTH,CPOR (%),KAIR (mD)\n2001.0,22.5,150\n2002.0,18.0,20\n").unwrap();
        drop(f);
        let csv = path.to_str().unwrap();

        let result = import_core_csv(&conn, &ids, csv);
        assert!(result.error.is_none(), "{:?}", result.error);
        assert_eq!(result.rows, 2);

        let (n, cpor0): (i64, f32) = conn
            .query_row(
                "SELECT COUNT(*), MIN(cpor) FROM core_data WHERE well_id = ?1",
                params![ids],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(n, 2);
        assert!((cpor0 - 0.18).abs() < 1e-6, "percent porosity must land as v/v, got {cpor0}");

        // Re-import replaces rather than duplicates.
        let again = import_core_csv(&conn, &ids, csv);
        assert!(again.error.is_none());
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM core_data WHERE well_id = ?1", params![ids], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2, "re-import must replace, not append");

        // Unknown well is rejected cleanly.
        let bad = import_core_csv(&conn, "no-such-well", csv);
        assert!(bad.error.is_some());

        // Core-to-log shift moves every plug by the same delta and reverses exactly.
        let shifted = db::shift_core_depths(&conn, &ids, 2.5).unwrap();
        assert_eq!(shifted, 2);
        let min_depth: f32 = conn
            .query_row("SELECT MIN(depth) FROM core_data WHERE well_id = ?1", params![ids], |r| r.get(0))
            .unwrap();
        assert!((min_depth - 2003.5).abs() < 1e-4);
        db::shift_core_depths(&conn, &ids, -2.5).unwrap();
        let min_depth: f32 = conn
            .query_row("SELECT MIN(depth) FROM core_data WHERE well_id = ?1", params![ids], |r| r.get(0))
            .unwrap();
        assert!((min_depth - 2001.0).abs() < 1e-4);
        std::fs::remove_file(&path).ok();
    }

    /// A second LAS import of a well whose name already exists still creates a separate record
    /// (auto-merge needs a confirmation flow) but must surface a warning, not silently fragment.
    #[test]
    fn las_import_warns_on_duplicate_well_name() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        let cols = || CurveColumns {
            depth: vec![1000.0, 1000.5, 1001.0],
            gr: vec![40.0, 45.0, 50.0],
            res: vec![f32::NAN; 3],
            nphi: vec![f32::NAN; 3],
            rhob: vec![f32::NAN; 3],
            dt: vec![f32::NAN; 3],
            sp: vec![f32::NAN; 3],
        };

        // First import: a fresh well, no duplicate warning.
        let r1 = insert_parsed_well(&conn, "a.las".into(), "DUP-1".into(), cols());
        assert!(r1.error.is_none(), "{:?}", r1.error);
        assert!(
            r1.warning.as_deref().map_or(true, |w| !w.contains("already exists")),
            "first import must not warn about a duplicate, got {:?}",
            r1.warning
        );

        // Second import of the SAME well name (normalized: lower-case + trailing space): a
        // separate record, but a duplicate warning.
        let r2 = insert_parsed_well(&conn, "b.las".into(), "dup-1  ".into(), cols());
        assert!(r2.error.is_none(), "{:?}", r2.error);
        assert!(
            r2.warning.as_deref().unwrap_or("").contains("already exists"),
            "re-import of a same-named (normalized) well must warn, got {:?}",
            r2.warning
        );
        assert_ne!(r1.well_id, r2.well_id, "still two distinct records (no auto-merge)");
    }

    /// Phase 6b: a full LAS with curves beyond the fixed 6 (PEF, CALI, a metric-unit
    /// sonic) must import whole into the generic store, with families tagged and units
    /// canonicalized. Also exercises the deviation-survey → minimum-curvature → well_path
    /// path end to end.
    #[test]
    fn generic_las_import_keeps_all_curves_and_converts_units() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "PEF-1", None, None, Some(25.0)).unwrap();
        let ids = well_id.to_string();

        // A minimal LAS 2.0 with DEPT, GR, PEF, HCAL (caliper), and DTCO given in us/m.
        let las = "~Version\n\
                   VERS. 2.0 :\n\
                   ~Curve\n\
                   DEPT .M    : depth\n\
                   GR   .GAPI : gamma\n\
                   PEF  .B/E  : photoelectric\n\
                   HCAL .IN   : caliper\n\
                   DTCO .US/M : sonic\n\
                   ~ASCII\n\
                   1000.0 55.0 5.1 8.5 656.0\n\
                   1000.5 60.0 5.2 8.6 660.0\n\
                   1001.0 -999.25 5.0 8.4 650.0\n";
        let path = std::env::temp_dir().join(format!("arshilla_pef_test_{ids}.las"));
        std::fs::write(&path, las).unwrap();

        import_all_curves_into_generic_store(&conn, &ids, path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();

        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        // PEF and CALI (family of HCAL) must be present with correct families.
        let pef = catalog.iter().find(|c| c.mnemonic == "PEF").expect("PEF imported");
        assert_eq!(pef.family.as_deref(), Some("PEF"));
        assert_eq!(pef.n_samples, 3);
        let cali = catalog.iter().find(|c| c.mnemonic == "HCAL").expect("HCAL imported");
        assert_eq!(cali.family.as_deref(), Some("CALI"));

        // DTCO in us/m must have been converted to us/ft and relabeled.
        let dt = catalog.iter().find(|c| c.mnemonic == "DTCO").expect("DTCO imported");
        assert_eq!(dt.unit.as_deref(), Some("us/ft"));
        let dt_samples = db::get_curve_samples(&conn, &dt.curve_id).unwrap();
        assert!((dt_samples[0].value - 656.0 * 0.3048).abs() < 0.5, "us/m→us/ft, got {}", dt_samples[0].value);

        // The LAS null (-999.25) in GR must be NaN in the store.
        let gr = catalog.iter().find(|c| c.mnemonic == "GR").expect("GR imported");
        let gr_samples = db::get_curve_samples(&conn, &gr.curve_id).unwrap();
        assert!(gr_samples[2].value.is_nan(), "LAS null must become NaN");

        // Deviation survey → TVD/TVDSS.
        let dev = std::env::temp_dir().join(format!("arshilla_dev_test_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0));
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 3);
        let path = db::get_well_path(&conn, &ids).unwrap();
        assert_eq!(path.len(), 3);
        assert!((path[1].tvd - 1000.0).abs() < 1e-2, "vertical section TVD == MD");
        assert!(path[2].tvd < path[2].md, "deviated station TVD shallower than MD");
        assert!((path[1].tvdss - (25.0 - 1000.0)).abs() < 1e-2, "TVDSS = datum - TVD");
    }

    #[test]
    fn deviation_import_materializes_tvd_tvdss_curves() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-1", None, None, None).unwrap();
        let ids = wid.to_string();

        // A log depth (MD) grid spanning the whole survey, incl. a deviated section.
        let depth = vec![0.0f32, 1000.0, 1500.0, 2000.0, 3000.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f.clone(),
        )
        .unwrap();

        // Vertical to 1000, build to 60° by 2000, hold to 3000.
        let dev = std::env::temp_dir().join(format!("arshilla_devmat_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0));
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);

        // Import auto-materialized TVD + TVDSS onto the log grid, fetchable by name.
        let (grid, cols) =
            crate::equations::fetch_curve_frame(&conn, &ids, &["TVD".to_string(), "TVDSS".to_string()]).unwrap();
        assert_eq!(grid, depth, "curves land on the standard depth grid");
        let (tvd, tvdss) = (&cols["TVD"], &cols["TVDSS"]);
        let i1000 = grid.iter().position(|&d| d == 1000.0).unwrap();
        assert!((tvd[i1000] - 1000.0).abs() < 1e-1, "vertical section TVD == MD: {}", tvd[i1000]);
        let i3000 = grid.iter().position(|&d| d == 3000.0).unwrap();
        assert!(tvd[i3000] < 2900.0, "deviated section TVD shallower than MD: {}", tvd[i3000]);
        // TVDSS = datum(25) − TVD everywhere (the interpolation preserves the affine relation).
        for (t, ss) in tvd.iter().zip(tvdss.iter()) {
            assert!((ss - (25.0 - t)).abs() < 1e-1, "TVDSS = 25 - TVD: {ss} vs {}", 25.0 - t);
        }
    }

    #[test]
    fn materialize_tvd_no_survey_writes_nothing() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-2", None, None, None).unwrap();
        let ids = wid.to_string();
        let depth = vec![0.0f32, 100.0, 200.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f,
        )
        .unwrap();
        // No survey → 0 samples written, and TVD stays absent (all-NaN via generic fallback).
        assert_eq!(materialize_tvd_curves(&conn, &ids).unwrap(), 0);
        let (_d, cols) = crate::equations::fetch_curve_frame(&conn, &ids, &["TVD".to_string()]).unwrap();
        assert!(cols["TVD"].iter().all(|v| v.is_nan()), "no survey → no TVD curve");
    }

    /// A vendor TVDSS imported into the generic RAW store must stay authoritative after a
    /// deviation survey is imported — the survey-derived COMPUTED TVDSS (which outranks the
    /// generic store in fetch_curve_frame) must not silently shadow it. TVD, with no import,
    /// is still materialized. Guards the cross-feature seam between TVD materialization and the
    /// standard→computed→generic resolution precedence.
    #[test]
    fn materialize_tvd_keeps_imported_tvdss_authoritative() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "DEV-MAT-3", None, None, None).unwrap();
        let ids = wid.to_string();
        let depth = vec![0.0f32, 1000.0, 2000.0, 3000.0];
        let f = vec![1.0f32; depth.len()];
        crate::db::insert_standard_curves(
            &conn, wid, depth.clone(), vec![50.0f32; depth.len()],
            f.clone(), f.clone(), f.clone(), f.clone(), f,
        )
        .unwrap();

        // A vendor TVDSS in the generic RAW store: a constant sentinel no survey-derived TVDSS
        // could produce, so we can tell which one resolves.
        let cid = crate::db::upsert_curve_meta(
            &conn, &ids, "RAW", "TVDSS", Some("m"), Some("TVDSS"), Some("LAS import"), None,
        )
        .unwrap();
        crate::db::insert_curve_samples(&conn, &cid, &depth, &vec![-777.0f32; depth.len()]).unwrap();

        // Import a deviated survey (would compute a very DIFFERENT TVDSS = 25 − TVD).
        let dev = std::env::temp_dir().join(format!("arshilla_devmat3_{ids}.csv"));
        std::fs::write(&dev, "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n3000,60,45\n").unwrap();
        let res = import_deviation_csv(&conn, &ids, dev.to_str().unwrap(), Some(25.0));
        std::fs::remove_file(&dev).ok();
        assert!(res.error.is_none(), "{:?}", res.error);

        let (_g, cols) = crate::equations::fetch_curve_frame(
            &conn, &ids, &["TVDSS".to_string(), "TVD".to_string()],
        )
        .unwrap();
        // Imported TVDSS still wins — NOT overwritten by the survey-derived computed curve.
        assert!(
            cols["TVDSS"].iter().all(|&v| (v - (-777.0)).abs() < 1e-3),
            "imported TVDSS must remain authoritative, got {:?}",
            cols["TVDSS"]
        );
        // TVD had no import → it IS materialized from the survey (shallower than MD when deviated).
        assert!(cols["TVD"].iter().any(|v| !v.is_nan()), "TVD still materialized from the survey");

        // And the stale-cleanup path: even if a computed TVDSS already existed (a survey was
        // materialized BEFORE the vendor curve arrived), a recompute clears it so the import wins.
        crate::equations::write_computed_curve(&conn, &ids, &depth, "TVDSS", &vec![9.9f32; depth.len()]).unwrap();
        materialize_tvd_curves(&conn, &ids).unwrap();
        let (_g2, cols2) = crate::equations::fetch_curve_frame(&conn, &ids, &["TVDSS".to_string()]).unwrap();
        assert!(
            cols2["TVDSS"].iter().all(|&v| (v - (-777.0)).abs() < 1e-3),
            "recompute must clear a stale survey TVDSS so the import wins, got {:?}",
            cols2["TVDSS"]
        );
    }

    /// #118 follow-up: a spliced LAS with a duplicate depth must import cleanly on BOTH the
    /// standard-curves AND the generic-store path. The generic path re-parses the file and
    /// writes curve_samples (curve_id, depth) PK, so without the same depth dedup it aborts
    /// silently (Err only logged), leaving the well missing its PEF/extra curves.
    #[test]
    fn duplicate_depth_las_imports_standard_and_generic_curves() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // Two rows at 1000.0 (a re-spliced section) plus PEF beyond the standard 6.
        let las = "~Version\nVERS. 2.0 :\n\
                   ~Curve\nDEPT .M : depth\nGR .GAPI : gamma\nPEF .B/E : pe\n\
                   ~ASCII\n1000.0 55.0 5.1\n1000.0 56.0 5.2\n1000.5 60.0 5.0\n";
        let path = std::env::temp_dir().join("arshilla_dupdepth_test.las");
        std::fs::write(&path, las).unwrap();

        let results = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None);
        std::fs::remove_file(&path).ok();

        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert!(r.error.is_none(), "import must succeed, got {:?}", r.error);
        assert_eq!(r.rows, 2, "duplicate 1000.0 row dropped → 2 unique depths");
        assert!(
            r.warning.as_deref().unwrap_or("").contains("duplicate"),
            "duplicate-depth warning surfaced, got {:?}",
            r.warning
        );

        let ids = r.well_id.clone().unwrap();
        let n_std: i64 = conn
            .query_row("SELECT COUNT(*) FROM standard_curves WHERE well_id = ?1", params![ids], |r| r.get(0))
            .unwrap();
        assert_eq!(n_std, 2, "standard_curves deduped to 2 rows");
        // The generic store must ALSO carry PEF — not silently missing from a PK abort.
        let catalog = db::list_generic_curve_catalog(&conn, &ids).unwrap();
        let pef = catalog.iter().find(|c| c.mnemonic == "PEF").expect("PEF must reach the generic store");
        assert_eq!(pef.n_samples, 2, "generic PEF deduped to 2 rows, not aborted");
    }

    /// #118 follow-up: a file whose (unrecognized) index column is entirely the null sentinel
    /// leaves zero rows after depth sanitation. That must ERROR — not commit a curve-less
    /// orphan well — and must create no wells row.
    #[test]
    fn all_null_depth_las_errors_without_creating_well() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();

        // XREF (unrecognized index) at column 0, every value the -999.25 null sentinel.
        let las = "~Version\nVERS. 2.0 :\n~Well\nNULL. -999.25 :\n\
                   ~Curve\nXREF .M : idx\nGR .GAPI : gamma\n\
                   ~ASCII\n-999.25 55.0\n-999.25 60.0\n";
        let path = std::env::temp_dir().join("arshilla_allnull_depth_test.las");
        std::fs::write(&path, las).unwrap();

        let results = import_las_files(&conn, &[path.to_str().unwrap().to_string()], None);
        std::fs::remove_file(&path).ok();

        let r = &results[0];
        assert!(r.error.is_some(), "all-null depth must error, not create an empty well");
        assert!(r.well_id.is_none(), "no well_id on the errored import");
        let n_wells: i64 = conn.query_row("SELECT COUNT(*) FROM wells", [], |r| r.get(0)).unwrap();
        assert_eq!(n_wells, 0, "no orphan well row created");
    }

    /// Well-locations CSV: alias-resolved EASTING/NORTHING/ZONE headers, name→well match,
    /// per-row zone overriding the dialog default, unmatched names reported, and re-import
    /// overwriting a previous fix.
    #[test]
    fn locations_import_matches_zones_and_overwrites() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        db::insert_well(&conn, a, "MHK-1", Some("Mahakam"), None, None).unwrap();
        db::insert_well(&conn, b, "MHK-2", Some("Mahakam"), None, None).unwrap();

        // MHK-1 carries its own zone column value; MHK-2's is blank → dialog default; the
        // third row's well isn't in the project → unmatched. Southern-hemisphere northings.
        let path = std::env::temp_dir().join(format!("arshilla_loc_{a}.csv"));
        std::fs::write(
            &path,
            "WELL,EASTING,NORTHING,ZONE\nMHK-1,485000.0,9450000.0,50S\nMHK-2,486200.5,9451750.0,\nGHOST,1,2,50S\n",
        )
        .unwrap();

        let res = import_locations_file(&conn, None, Some("50M"), path.to_str().unwrap());
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_located, 2);
        assert_eq!(res.unmatched_wells, vec!["GHOST".to_string()]);

        let read = |id: &Uuid| -> (f64, f64, String) {
            conn.query_row(
                "SELECT surface_x, surface_y, utm_zone FROM wells WHERE well_id = ?1",
                params![id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, String>(2)?)),
            )
            .unwrap()
        };
        let (x1, y1, z1) = read(&a);
        assert!((x1 - 485000.0).abs() < 1e-6 && (y1 - 9450000.0).abs() < 1e-6);
        assert_eq!(z1, "50S", "explicit ZONE cell wins");
        let (_, _, z2) = read(&b);
        assert_eq!(z2, "50M", "blank ZONE cell falls back to the dialog default");

        // Re-import with a new easting overwrites rather than erroring or duplicating.
        std::fs::write(&path, "WELL,X,Y\nMHK-1,490000.0,9460000.0\n").unwrap();
        let res2 = import_locations_file(&conn, None, Some("50S"), path.to_str().unwrap());
        assert!(res2.error.is_none());
        assert_eq!(res2.wells_located, 1);
        let (x1b, _, _) = read(&a);
        assert!((x1b - 490000.0).abs() < 1e-6, "re-import overwrote the location");
        std::fs::remove_file(&path).ok();

        // A file with no X/Y column fails cleanly.
        let bad = std::env::temp_dir().join(format!("arshilla_loc_bad_{a}.csv"));
        std::fs::write(&bad, "WELL,DEPTH\nMHK-1,1000\n").unwrap();
        let res3 = import_locations_file(&conn, None, None, bad.to_str().unwrap());
        assert!(res3.error.is_some(), "missing coordinate columns must error");
        std::fs::remove_file(&bad).ok();
    }

    /// A blank WELL cell in a multi-well file must NOT be routed to the selected well (that
    /// would silently corrupt an unrelated well's surface location) — it is skipped and
    /// surfaced. The headerless single-well fallback must still route to the selected well.
    #[test]
    fn locations_import_skips_blank_well_cell_not_default() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        db::insert_well(&conn, a, "MHK-1", Some("Mahakam"), None, None).unwrap();
        db::insert_well(&conn, b, "MHK-2", Some("Mahakam"), None, None).unwrap();

        // Multi-well file (HAS a WELL column) whose 2nd row's WELL cell is blank but carries
        // valid coordinates. MHK-1 is "selected" (default_well_id = a); the blank row must
        // not overwrite MHK-1 — MHK-1 never appears in the file.
        let path = std::env::temp_dir().join(format!("arshilla_locblank_{a}.csv"));
        std::fs::write(&path, "WELL,EASTING,NORTHING\nMHK-2,486000.0,9451000.0\n,999999.0,888888.0\n").unwrap();
        let res = import_locations_file(&conn, Some(&a.to_string()), Some("50S"), path.to_str().unwrap());
        std::fs::remove_file(&path).ok();

        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.wells_located, 1, "only MHK-2 located; the blank row is skipped, not routed to MHK-1");
        assert!(
            res.unmatched_wells.iter().any(|s| s.contains("blank")),
            "blank row must be surfaced, got {:?}",
            res.unmatched_wells
        );
        let x1: Option<f64> = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![a.to_string()], |r| r.get(0))
            .unwrap();
        assert!(x1.is_none(), "selected well must be untouched by a blank-WELL row, got {x1:?}");
        let x2: f64 = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![b.to_string()], |r| r.get(0))
            .unwrap();
        assert!((x2 - 486000.0).abs() < 1e-6, "MHK-2 located from its named row");

        // A genuinely headerless (no WELL column) single-well file still routes to the
        // selected well — the fallback the fix must NOT break.
        let path2 = std::env::temp_dir().join(format!("arshilla_locnohdr_{a}.csv"));
        std::fs::write(&path2, "EASTING,NORTHING\n500000.0,9400000.0\n").unwrap();
        let res2 = import_locations_file(&conn, Some(&a.to_string()), Some("50S"), path2.to_str().unwrap());
        std::fs::remove_file(&path2).ok();
        assert!(res2.error.is_none(), "{:?}", res2.error);
        assert_eq!(res2.wells_located, 1, "headerless file routes to the selected well");
        let x1b: f64 = conn
            .query_row("SELECT surface_x FROM wells WHERE well_id = ?1", params![a.to_string()], |r| r.get(0))
            .unwrap();
        assert!((x1b - 500000.0).abs() < 1e-6, "selected well located from the headerless file");
    }

    /// SCAL Pc CSV import: alias headers, percent Sw/poro detection, replace-on-reimport,
    /// and the Leverett-J fit coming back with the import result.
    #[test]
    fn scal_import_fits_leverett_j() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-1", None, None, None).unwrap();
        let ids = well_id.to_string();

        // Synthesize points on Sw = 0.4 * J^-0.5 at IFT 72 (like the satheight unit test),
        // but delivered the way labs do: Sw in percent, headers with units.
        let mut body = String::from("Sample,Depth (m),Kair (mD),CPOR (%),Pc (psi),Sw (%)\n");
        for i in 1..=12 {
            let pc = i as f64 * 3.0;
            let j = 0.21645 * pc / 72.0 * (150.0f64 / 0.22).sqrt();
            let sw = (0.4 * j.powf(-0.5)).min(1.0) * 100.0;
            body.push_str(&format!("1,2000.5,150,22,{pc},{sw:.2}\n"));
        }
        let path = std::env::temp_dir().join(format!("arshilla_scal_test_{ids}.csv"));
        std::fs::write(&path, &body).unwrap();

        let res = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 12);
        let fit = res.fit.expect("fit should solve");
        assert!((fit.b - -0.5).abs() < 0.05, "b={}", fit.b);
        assert!((fit.a - 0.4).abs() < 0.1, "a={}", fit.a);

        // Re-import replaces rather than duplicates; rows readable back.
        let res2 = import_scal_csv(&conn, &ids, path.to_str().unwrap(), 72.0);
        std::fs::remove_file(&path).ok();
        assert_eq!(res2.rows, 12);
        let rows = db::get_scal_pc(&conn, &ids).unwrap();
        assert_eq!(rows.len(), 12);
        assert!((rows[0].poro - 0.22).abs() < 1e-4, "percent poro converted to v/v");
        assert!(rows.iter().all(|r| r.sw <= 1.0), "percent Sw converted to v/v");

        // Unknown well errors cleanly.
        let bad = import_scal_csv(&conn, "nope", "x.csv", 72.0);
        assert!(bad.error.is_some());
    }

    /// Multi-file SCAL import (increment 2): two single-plug centrifuge exports sniffed
    /// by "auto" land in one combined replace-write; a later porous-plate import REPLACES
    /// them (not appends); a bad file fails the whole import with the filename named.
    #[test]
    fn scal_import_files_multi_format_and_replace() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-2", None, None, None).unwrap();
        let ids = well_id.to_string();

        let cf = |sample: &str, depth: f32| {
            format!(
                "SAMPLE,{sample}\nDEPTH,{depth}\nPERM,45.0\nPORO,18.0\n\
                 Speed (RPM),Pc (psi),Sw (%PV)\n500,2.1,95.0\n1000,8.4,78.2\n2000,33.6,55.4\n4000,120.0,41.0\n"
            )
        };
        let p1 = std::env::temp_dir().join(format!("sandibumi_scal_cf1_{ids}.csv"));
        let p2 = std::env::temp_dir().join(format!("sandibumi_scal_cf2_{ids}.csv"));
        std::fs::write(&p1, cf("12A", 2695.3)).unwrap();
        std::fs::write(&p2, cf("S-16A", 2701.8)).unwrap();
        let paths = vec![p1.to_str().unwrap().to_string(), p2.to_str().unwrap().to_string()];

        let res = import_scal_files(&conn, &ids, &paths, "auto", "air_brine", 72.0);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.rows, 8, "both plugs land in one combined import");
        assert!(res.fit.is_some(), "J-fit solves over the pooled points");
        let rows = db::get_scal_pc(&conn, &ids).unwrap();
        assert_eq!(rows.len(), 8);
        assert!(rows.iter().any(|r| r.sample_no == Some(12)) && rows.iter().any(|r| r.sample_no == Some(16)));
        assert!(
            rows.iter().all(|r| r.system.as_deref() == Some("air_brine") && r.ift == Some(72.0)),
            "fluid system + IFT stored on every point"
        );

        // A porous-plate re-import replaces the centrifuge set (write discipline).
        let wide = "Sample,Depth (m),Perm (mD),Poro (%),1,2,4,8\n4,2001.5,150.0,22.5,98.5,95.2,88.1,79.4\n";
        let p3 = std::env::temp_dir().join(format!("sandibumi_scal_pp_{ids}.csv"));
        std::fs::write(&p3, wide).unwrap();
        let res2 =
            import_scal_files(&conn, &ids, &[p3.to_str().unwrap().to_string()], "porous_plate", "air_brine", 72.0);
        assert!(res2.error.is_none(), "{:?}", res2.error);
        assert_eq!(res2.rows, 4);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 4, "replace, not append");

        // One bad file fails the whole import and names the file.
        let res3 = import_scal_files(
            &conn,
            &ids,
            &[p1.to_str().unwrap().to_string(), "missing_dir/nope.csv".to_string()],
            "auto",
            "air_brine",
            72.0,
        );
        assert!(res3.error.as_deref().is_some_and(|e| e.contains("nope.csv")));
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 4, "failed import leaves prior rows intact");

        for p in [&p1, &p2, &p3] {
            std::fs::remove_file(p).ok();
        }
    }

    /// Post-review hardening: a structurally-valid file that parses to ZERO points must
    /// refuse the replace-write instead of silently wiping the well's existing SCAL data.
    #[test]
    fn scal_import_zero_rows_leaves_existing_data() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = Uuid::new_v4();
        db::insert_well(&conn, well_id, "SCAL-3", None, None, None).unwrap();
        let ids = well_id.to_string();

        let good = std::env::temp_dir().join(format!("sandibumi_scal_good_{ids}.csv"));
        std::fs::write(&good, "PC,SW\n5,0.55\n10,0.45\n20,0.35\n").unwrap();
        let res = import_scal_files(&conn, &ids, &[good.to_str().unwrap().to_string()], "long", "hg_air", 367.0);
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 3);

        // Header-only export (e.g. a filtered/template sheet) → error, data intact.
        let empty = std::env::temp_dir().join(format!("sandibumi_scal_empty_{ids}.csv"));
        std::fs::write(&empty, "PC,SW\n").unwrap();
        let res2 = import_scal_files(&conn, &ids, &[empty.to_str().unwrap().to_string()], "auto", "hg_air", 367.0);
        assert!(res2.error.as_deref().is_some_and(|e| e.contains("untouched")), "{:?}", res2.error);
        assert_eq!(db::get_scal_pc(&conn, &ids).unwrap().len(), 3, "existing points survive");

        for p in [&good, &empty] {
            std::fs::remove_file(p).ok();
        }
    }

    /// P2 tops import: multi-well CSV matches wells by name, no-well-column file needs
    /// the selected well, re-import updates depth but keeps an existing color.
    #[test]
    fn tops_import_multiwell_and_default() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let w1 = Uuid::new_v4();
        let w2 = Uuid::new_v4();
        db::insert_well(&conn, w1, "BALAM-1", None, None, None).unwrap();
        db::insert_well(&conn, w2, "BALAM-2", None, None, None).unwrap();
        let id1 = w1.to_string();

        let path = std::env::temp_dir().join("arshilla_tops_import.csv");
        std::fs::write(
            &path,
            "WELL,TOP,MD\nbalam-1,TOP_A,1000.0\nBALAM-1,TOP_B,1100.0\nBALAM-2,TOP_A,1010.0\nGHOST-9,TOP_A,900.0\n",
        )
        .unwrap();
        let res = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.tops_written, 3);
        assert_eq!(res.wells_matched, 2, "case-insensitive well matching");
        assert_eq!(res.unmatched_wells, vec!["GHOST-9".to_string()]);

        // Give TOP_A a color, then re-import a new depth: depth moves, color survives.
        db::upsert_top(&conn, &id1, "TOP_A", 1000.0, Some("#ff0000")).unwrap();
        std::fs::write(&path, "WELL,TOP,MD\nBALAM-1,TOP_A,1005.0\n").unwrap();
        let res2 = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(res2.error.is_none());
        let tops = db::list_tops(&conn, &id1).unwrap();
        let a = tops.iter().find(|t| t.top_name == "TOP_A").unwrap();
        assert!((a.depth - 1005.0).abs() < 1e-3, "re-import updates depth");
        assert_eq!(a.color.as_deref(), Some("#ff0000"), "existing color kept");

        // No WELL column: needs a default well; with one it lands there.
        std::fs::write(&path, "TOP,DEPTH\nTOP_C,1200.0\n").unwrap();
        let need = import_tops_file(&conn, None, path.to_str().unwrap());
        assert!(need.error.is_some(), "no well column and no selection must error");
        let ok = import_tops_file(&conn, Some(&id1), path.to_str().unwrap());
        assert!(ok.error.is_none());
        assert!(db::list_tops(&conn, &id1).unwrap().iter().any(|t| t.top_name == "TOP_C"));
        std::fs::remove_file(&path).ok();
    }

    /// P2 aux import: XRD point data (numeric + text cells) and perforation intervals
    /// land in aux_data long format; re-import replaces per (well, dataset); datasets
    /// are independent.
    #[test]
    fn aux_import_xrd_and_perforation() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let w = Uuid::new_v4();
        db::insert_well(&conn, w, "AUX-1", None, None, None).unwrap();
        let ids = w.to_string();

        let xrd = std::env::temp_dir().join("arshilla_aux_xrd.csv");
        std::fs::write(&xrd, "Depth,Quartz,Illite,Remarks\n2000.0,45.2,12.1,clean\n2001.0,40.0,,silty\n").unwrap();
        let res = import_aux_file(&conn, &ids, "xrd", xrd.to_str().unwrap());
        assert!(res.error.is_none(), "{:?}", res.error);
        assert_eq!(res.dataset, "XRD", "dataset name normalized upper");
        assert_eq!(res.rows, 5, "empty cell is skipped, text cell kept");

        let rows = db::list_aux_data(&conn, &ids, Some("XRD")).unwrap();
        assert_eq!(rows.len(), 5);
        let quartz0 = rows.iter().find(|r| r.item == "QUARTZ" && r.depth_top == 2000.0).unwrap();
        assert!((quartz0.value_num.unwrap() - 45.2).abs() < 1e-3);
        assert!(quartz0.value_text.is_none());
        let remark = rows.iter().find(|r| r.item == "REMARKS" && r.depth_top == 2001.0).unwrap();
        assert_eq!(remark.value_text.as_deref(), Some("silty"));
        assert!(remark.value_num.is_none());

        // Perforation intervals in a second dataset; both coexist.
        let perf = std::env::temp_dir().join("arshilla_aux_perf.csv");
        std::fs::write(&perf, "FROM,TO,STATUS\n2050.0,2055.0,OPEN\n2100.0,2104.0,SQUEEZED\n").unwrap();
        let res2 = import_aux_file(&conn, &ids, "PERFORATION", perf.to_str().unwrap());
        assert!(res2.error.is_none());
        assert_eq!(res2.rows, 2);
        let perfs = db::list_aux_data(&conn, &ids, Some("PERFORATION")).unwrap();
        assert_eq!(perfs[0].depth_base, Some(2055.0), "interval BASE kept");
        assert_eq!(perfs[1].value_text.as_deref(), Some("SQUEEZED"));
        let sets = db::list_aux_datasets(&conn, &ids).unwrap();
        assert_eq!(sets, vec![("PERFORATION".to_string(), 2i64), ("XRD".to_string(), 5i64)]);

        // Re-import of XRD replaces only XRD.
        std::fs::write(&xrd, "Depth,Quartz\n2000.0,50.0\n").unwrap();
        let res3 = import_aux_file(&conn, &ids, "XRD", xrd.to_str().unwrap());
        assert!(res3.error.is_none());
        let sets = db::list_aux_datasets(&conn, &ids).unwrap();
        assert_eq!(sets, vec![("PERFORATION".to_string(), 2i64), ("XRD".to_string(), 1i64)]);
        std::fs::remove_file(&xrd).ok();
        std::fs::remove_file(&perf).ok();

        // Unknown well errors cleanly.
        let bad = import_aux_file(&conn, "nope", "XRD", "x.csv");
        assert!(bad.error.is_some());
    }

    /// Ad-hoc verification against real field LAS files. Ignored by default since it
    /// depends on absolute paths that only exist on this machine; run explicitly with
    /// `cargo test --release -- --ignored --nocapture test_import_real_field_files`.
    #[test]
    #[ignore]
    fn test_import_real_field_files() {
        let paths: Vec<String> = vec![
            r"D:\01. Work\00. Guidebook\02. Guidebook Geolog\Loglan\mina01060d1_study_minas_itb2022_final.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00008_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00009_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00010_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00011_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00012_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00001_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00002_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00004_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00005_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00006_lapi2023_fprooh.las",
            r"D:\01. Work\2023\10. LQR Balam South - PHR Rokan\13. Delivery Data\01. Final Log\BLSO_LAPI2023_FPROOH\blso00007_lapi2023_fprooh.las",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let db_path = std::env::temp_dir().join("arshilla_import_test.duckdb");
        let _ = std::fs::remove_file(&db_path);
        let conn = db::init_db(db_path.to_str().unwrap()).expect("init_db failed");

        let results = import_las_files(&conn, &paths, None);
        for r in &results {
            println!(
                "{} -> well_name={:?} rows={} error={:?}",
                r.path, r.well_name, r.rows, r.error
            );
        }

        let failures: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
        assert!(failures.is_empty(), "{failures:?}");

        let well_count: i64 = conn
            .query_row("SELECT count(*) FROM wells", [], |row| row.get(0))
            .unwrap();
        assert_eq!(well_count, paths.len() as i64);
    }
}
