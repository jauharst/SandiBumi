//! LAS 2.0 export: one well's standard + computed curves on the standard depth grid.
//! NaN (missing) writes as the conventional -999.25 null value.

use crate::equations::fetch_curve_frame;
use duckdb::{params, Connection};
use std::io::{BufWriter, Write};

const NULL_VALUE: f32 = -999.25;

/// Standard curve units, mirroring the catalog; computed curves pull units from the
/// equation that produced them (blank when unknown).
fn standard_units(name: &str) -> &'static str {
    match name {
        "GR" => "GAPI",
        "RES_DEEP" => "OHMM",
        "NPHI" => "V/V",
        "RHOB" => "G/C3",
        "DT" => "US/F",
        "SP" => "MV",
        _ => "",
    }
}

pub fn export_las(conn: &Connection, well_id: &str, dest_path: &str) -> Result<usize, String> {
    let (well_name, field_name): (String, Option<String>) = conn
        .query_row(
            "SELECT well_name, field_name FROM wells WHERE well_id = ?1",
            params![well_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Every curve this well actually has: the six standard ones + its computed curves.
    let mut curve_names: Vec<String> =
        ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"].iter().map(|s| s.to_string()).collect();
    let mut units: Vec<String> = curve_names.iter().map(|n| standard_units(n).to_string()).collect();
    {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT cc.curve_name, COALESCE(e.output_units, '')
                 FROM computed_curves cc
                 LEFT JOIN equations e ON e.output_curve = cc.curve_name
                 WHERE cc.well_id = ?1 ORDER BY cc.curve_name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![well_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for r in rows {
            let (name, unit) = r.map_err(|e| e.to_string())?;
            // A unit DECLARED by whatever wrote the curve beats one inferred from an equation of
            // the same name (SB-MLA-035). The case this exists for is a prediction fitted in log
            // space: exported with a blank unit — or worse, with the units of the quantity it is a
            // logarithm OF — a log10(mD) column reads as a permeability, and every negative value
            // in it reads as a physically impossible one rather than as a number below 1 mD.
            let unit = crate::db::curve_unit_for(conn, well_id, &name).unwrap_or(unit);
            curve_names.push(name);
            units.push(unit);
        }
    }

    let (depth, columns) = fetch_curve_frame(conn, well_id, &curve_names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("well has no curve data".into());
    }
    let depth_unit = crate::units::project_depth_unit_or_default(conn).code();
    let step = if depth.len() > 1 { depth[1] - depth[0] } else { 0.0 };

    let file = std::fs::File::create(dest_path).map_err(|e| e.to_string())?;
    let mut w = BufWriter::new(file);
    let fmt = |v: f32| -> String {
        if v.is_nan() {
            format!("{NULL_VALUE:.4}")
        } else {
            format!("{v:.4}")
        }
    };

    let write = || -> std::io::Result<()> {
        writeln!(w, "~Version Information")?;
        writeln!(w, " VERS.                 2.0 : CWLS log ASCII Standard - VERSION 2.0")?;
        writeln!(w, " WRAP.                  NO : One line per depth step")?;
        writeln!(w, "~Well Information")?;
        writeln!(w, " STRT.{depth_unit}   {:>12.4} : START DEPTH", depth[0])?;
        writeln!(w, " STOP.{depth_unit}   {:>12.4} : STOP DEPTH", depth[depth.len() - 1])?;
        writeln!(w, " STEP.{depth_unit}   {:>12.4} : STEP", step)?;
        writeln!(w, " NULL.    {:>12.4} : NULL VALUE", NULL_VALUE)?;
        writeln!(w, " WELL.    {} : WELL NAME", well_name)?;
        writeln!(w, " FLD .    {} : FIELD", field_name.unwrap_or_default())?;
        writeln!(w, " SRVC.    SandiBumi : EXPORTED BY")?;
        writeln!(w, "~Curve Information")?;
        writeln!(w, " DEPT.{depth_unit}                     : Depth")?;
        for (name, unit) in curve_names.iter().zip(units.iter()) {
            writeln!(w, " {:<8}.{:<8}          : {}", name, unit, name)?;
        }
        writeln!(w, "~ASCII")?;
        for i in 0..depth.len() {
            let mut line = format!("{:>12.4}", depth[i]);
            for name in &curve_names {
                // fetch_curve_frame keys the column map by name.trim().to_uppercase(); a mixed-case
                // computed curve (e.g. "Vsh_final") missed here and exported an all-NULL column.
                let key = name.trim().to_uppercase();
                let v = columns.get(&key).map(|c| *c.get(i).unwrap_or(&f32::NAN)).unwrap_or(f32::NAN);
                line.push_str(&format!(" {:>12}", fmt(v)));
            }
            writeln!(w, "{line}")?;
        }
        w.flush()
    };
    write().map_err(|e| e.to_string())?;
    Ok(depth.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use uuid::Uuid;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("sandibumi-export-{}-{}.las", tag, std::process::id()))
    }

    /// A well with one missing sample per curve, plus a MIXED-CASE computed curve.
    fn seed(conn: &Connection) -> (Uuid, Vec<f32>, Vec<f32>) {
        db::create_schema(conn).unwrap();
        let id = Uuid::new_v4();
        db::insert_well(conn, id, "SANDI-EXP", Some("Synthetic"), None, None).unwrap();

        let depth: Vec<f32> = (0..6).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let gr = vec![40.0f32, 55.5, f32::NAN, 88.25, 120.0, 33.125];
        let res = vec![2.0f32; 6];
        let nphi = vec![0.30f32; 6];
        let rhob = vec![2.45f32; 6];
        let nan = vec![f32::NAN; 6];
        db::insert_standard_curves(
            conn,
            id,
            depth.clone(),
            gr.clone(),
            res,
            nphi,
            rhob,
            nan.clone(),
            nan,
        )
        .unwrap();

        // Deliberately mixed case — see the regression note below.
        let vsh = vec![0.1f32, 0.25, 0.5, f32::NAN, 0.75, 0.9];
        crate::equations::write_computed_curves_batch(
            conn,
            &id.to_string(),
            &depth,
            &[("Vsh_final", &vsh)],
        )
        .unwrap();

        (id, gr, vsh)
    }

    /// `export.rs` shipped with no tests at all. Three claims matter and none was pinned.
    ///
    /// **Missing writes as the null value, never as a blank or a zero.** A 0.0 where a sample was
    /// absent is a reading nobody took, and it re-imports as real data.
    ///
    /// **Computed curves are exported, not just the six standard ones** — otherwise a delivered
    /// LAS silently omits the entire interpretation.
    ///
    /// **A mixed-case computed name still carries its values.** `fetch_curve_frame` keys its
    /// column map by `trim().to_uppercase()`, so looking it up under the stored spelling misses
    /// and the column exports as ALL NULL — a full-length curve of nothing, in a file that looks
    /// perfectly well formed. The exporter uppercases the key for exactly this reason; this test
    /// is what stops that line being "tidied" away.
    #[test]
    fn export_writes_missing_as_null_and_carries_mixed_case_computed_curves() {
        let conn = Connection::open_in_memory().unwrap();
        let (id, gr, vsh) = seed(&conn);
        let dest = tmp_path("null");

        let rows = export_las(&conn, &id.to_string(), dest.to_str().unwrap()).unwrap();
        assert_eq!(rows, 6, "one row per depth sample");

        let text = std::fs::read_to_string(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);

        assert!(text.contains("NULL."), "the header must declare a null value");
        assert!(
            text.contains("-999.2500"),
            "a missing sample must be written as the declared null, not blank or 0"
        );
        assert!(
            text.contains("Vsh_final"),
            "computed curves must appear in ~Curve Information"
        );

        // The data block, column by column: DEPT, the six standard curves, then Vsh_final.
        let body = text.split("~ASCII").nth(1).unwrap();
        let rows: Vec<Vec<f32>> = body
            .lines()
            .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit() || c == '-'))
            .map(|l| l.split_whitespace().map(|t| t.parse::<f32>().unwrap()).collect())
            .collect();
        assert_eq!(rows.len(), 6);

        let vsh_col: Vec<f32> = rows.iter().map(|r| *r.last().unwrap()).collect();
        assert!(
            vsh_col.iter().any(|v| (*v - NULL_VALUE).abs() > 1e-3),
            "the mixed-case computed curve exported as ALL NULL — the uppercase key lookup broke"
        );
        for (i, expect) in vsh.iter().enumerate() {
            let got = vsh_col[i];
            if expect.is_nan() {
                assert!((got - NULL_VALUE).abs() < 1e-3, "row {i}: missing must be the null value");
            } else {
                assert!((got - expect).abs() < 5e-4, "row {i}: {got} != {expect}");
            }
        }

        // GR is the second column (after DEPT) and carries the missing sample at index 2.
        let gr_col: Vec<f32> = rows.iter().map(|r| r[1]).collect();
        assert!((gr_col[2] - NULL_VALUE).abs() < 1e-3, "GR's missing sample must be null");
        assert!((gr_col[0] - gr[0]).abs() < 5e-4);
    }

    /// The round trip: export a well, import the file into a FRESH project, and the numbers must
    /// come back unchanged.
    ///
    /// This is worth more than either half tested alone, because it is the only check that the
    /// writer and the reader agree about the SAME conventions — the null value, the fixed-width
    /// columns, the well name, the depth grid. A writer and a reader can each be
    /// self-consistently wrong; they cannot both be wrong in a way that survives a round trip.
    ///
    /// Missing must survive as MISSING. If the declared -999.25 came back as the number
    /// -999.25 it would be a porosity of minus a thousand, and it would plot.
    #[test]
    fn an_exported_las_reimports_with_the_same_values() {
        let src = Connection::open_in_memory().unwrap();
        let (id, gr, _) = seed(&src);
        let dest = tmp_path("roundtrip");
        export_las(&src, &id.to_string(), dest.to_str().unwrap()).unwrap();

        // A brand new project — nothing carried over except the file itself.
        let dst = Connection::open_in_memory().unwrap();
        db::create_schema(&dst).unwrap();
        let results =
            crate::ingest::import_las_files(&dst, &[dest.to_str().unwrap().to_string()], None);
        let _ = std::fs::remove_file(&dest);

        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none(), "re-import failed: {:?}", results[0].error);

        let new_id: String = dst
            .query_row("SELECT well_id FROM wells", [], |r| r.get(0))
            .expect("the re-imported well must exist");
        let name: String = dst
            .query_row("SELECT well_name FROM wells", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "SANDI-EXP", "the well name must survive the round trip");

        let (depth, cols) =
            crate::equations::fetch_curve_frame(&dst, &new_id, &["GR".to_string()]).unwrap();
        assert_eq!(depth.len(), 6, "every depth sample must come back");

        let back = cols.get("GR").expect("GR must be present after re-import");
        for (i, expect) in gr.iter().enumerate() {
            if expect.is_nan() {
                assert!(
                    back[i].is_nan(),
                    "row {i}: the declared null must return as MISSING, not as the number -999.25"
                );
            } else {
                assert!((back[i] - expect).abs() < 5e-4, "row {i}: {} != {expect}", back[i]);
            }
        }
    }

    /// SB-DIO-017 / SB-DIO-T27..T28. The LAS unit spellings and the project-unit
    /// source are specified in `docs/PRD_v2/21_data-io.md` §§4 and 5.1.
    #[test]
    fn the_las_writer_declares_the_unit_it_wrote_for_both_feet_and_metres() {
        for (unit, code, tag) in [
            (crate::units::DepthUnit::Feet, "FT", "feet-unit"),
            (crate::units::DepthUnit::Metres, "M", "metre-unit"),
        ] {
            let src = Connection::open_in_memory().unwrap();
            let (id, _, _) = seed(&src);
            crate::units::set_project_depth_unit(&src, unit).unwrap();
            let dest = tmp_path(tag);
            export_las(&src, &id.to_string(), dest.to_str().unwrap()).unwrap();

            let text = crate::parsers::read_text_file(&dest).unwrap();
            for mnemonic in ["STRT", "STOP", "STEP", "DEPT"] {
                assert!(
                    text.lines().any(|line| line.trim_start().starts_with(&format!("{mnemonic}.{code}"))),
                    "{mnemonic} must declare {code} when those are the depths written"
                );
            }
            let other = if code == "FT" { "M" } else { "FT" };
            assert!(
                !text.lines().any(|line| {
                    ["STRT", "STOP", "STEP", "DEPT"]
                        .iter()
                        .any(|mnemonic| line.trim_start().starts_with(&format!("{mnemonic}.{other}")))
                }),
                "the opposite unit must not remain on any depth declaration"
            );

            let dst = Connection::open_in_memory().unwrap();
            db::create_schema(&dst).unwrap();
            let imported = crate::ingest::import_las_files(
                &dst,
                &[dest.to_str().unwrap().to_string()],
                None,
            );
            let _ = std::fs::remove_file(&dest);
            assert!(imported[0].error.is_none(), "{code} round trip failed: {:?}", imported[0].error);
            let imported_unit = crate::units::project_depth_unit(&dst).unwrap();
            assert_eq!(imported_unit, Some(unit));
            let first_depth: f32 = dst
                .query_row("SELECT depth FROM standard_curves ORDER BY depth LIMIT 1", [], |row| row.get(0))
                .unwrap();
            assert!((first_depth - 2000.0).abs() < 1e-4, "{code} depths must survive unchanged");
        }
    }
}
