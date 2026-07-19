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
    pub error: Option<String>,
}

/// Parses every given LAS file concurrently via `rayon` (CPU-bound), then inserts each
/// well and its curves into DuckDB sequentially — the connection is behind a single lock,
/// so only the parsing step benefits from parallelism, which is also the expensive part.
pub fn import_las_files(conn: &Connection, paths: &[String]) -> Vec<ImportResult> {
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
        .map(|(path, result)| match result {
            Ok((well_name, columns)) => insert_parsed_well(conn, path, well_name, columns),
            Err(e) => ImportResult { path, well_id: None, well_name: None, rows: 0, error: Some(e.to_string()) },
        })
        .collect()
}

fn insert_parsed_well(conn: &Connection, path: String, well_name: String, columns: CurveColumns) -> ImportResult {
    let well_id = Uuid::new_v4();
    let rows = columns.depth.len();

    let result: db::DbResult<()> = (|| {
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
    })();

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
            ImportResult { path, well_id: Some(well_id.to_string()), well_name: Some(well_name), rows, error: None }
        }
        Err(e) => ImportResult { path, well_id: None, well_name: None, rows: 0, error: Some(e.to_string()) },
    }
}

/// Re-reads a LAS file keeping all curves and writes each into `curve_meta`/`curve_samples`
/// as set RAW, tagging family (via the mnemonic dictionary) and normalizing units where a
/// conversion is known. The unit stored is the canonical one when converted, else the
/// file's original unit.
pub fn import_all_curves_into_generic_store(conn: &Connection, well_id: &str, path: &str) -> db::DbResult<()> {
    let frame = match parsers::parse_las_2_all(path) {
        Ok(f) => f,
        Err(e) => return Err(db::DbError::LengthMismatch(format!("parse_las_2_all: {e}"))),
    };
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
        Ok(()) => CoreImportResult { path: path.to_string(), rows, error: None },
        Err(e) => CoreImportResult { path: path.to_string(), rows: 0, error: Some(e.to_string()) },
    }
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

/// Parses a SCAL capillary-pressure CSV, replaces the well's `scal_pc` rows, and fits
/// the Leverett-J function (Sw = A·J^B) over the points at `ift_lab` (sigma·cosθ of the
/// lab fluid system, dyn/cm — e.g. 72 air-brine, 367 air-mercury).
pub fn import_scal_csv(conn: &Connection, well_id: &str, path: &str, ift_lab: f64) -> ScalImportResult {
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", params![well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return ScalImportResult { path: path.to_string(), rows: 0, fit: None, error: Some(format!("unknown well '{well_id}'")) };
    }

    let records = match parsers::parse_scal_csv(path) {
        Ok(r) => r,
        Err(e) => return ScalImportResult { path: path.to_string(), rows: 0, fit: None, error: Some(e.to_string()) },
    };
    let rows: Vec<db::ScalPcRow> = records
        .iter()
        .map(|r| db::ScalPcRow {
            sample_no: r.sample_no,
            depth: r.depth,
            perm: r.perm,
            poro: r.poro,
            pc: r.pc,
            sw: r.sw,
        })
        .collect();
    if let Err(e) = db::insert_scal_pc(conn, well_id, &rows) {
        return ScalImportResult { path: path.to_string(), rows: 0, fit: None, error: Some(e.to_string()) };
    }

    let points: Vec<crate::satheight::ScalPoint> = records
        .iter()
        .map(|r| crate::satheight::ScalPoint { pc: r.pc, sw: r.sw, perm: r.perm, poro: r.poro })
        .collect();
    let fit = crate::satheight::fit_leverett_j(&points, ift_lab);
    ScalImportResult { path: path.to_string(), rows: rows.len(), fit, error: None }
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

        let results = import_las_files(&conn, &paths);
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
