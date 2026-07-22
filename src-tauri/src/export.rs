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
            curve_names.push(name);
            units.push(unit);
        }
    }

    let (depth, columns) = fetch_curve_frame(conn, well_id, &curve_names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("well has no curve data".into());
    }
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
        writeln!(w, " STRT.M   {:>12.4} : START DEPTH", depth[0])?;
        writeln!(w, " STOP.M   {:>12.4} : STOP DEPTH", depth[depth.len() - 1])?;
        writeln!(w, " STEP.M   {:>12.4} : STEP", step)?;
        writeln!(w, " NULL.    {:>12.4} : NULL VALUE", NULL_VALUE)?;
        writeln!(w, " WELL.    {} : WELL NAME", well_name)?;
        writeln!(w, " FLD .    {} : FIELD", field_name.unwrap_or_default())?;
        writeln!(w, " SRVC.    SandiBumi : EXPORTED BY")?;
        writeln!(w, "~Curve Information")?;
        writeln!(w, " DEPT.M                     : Depth")?;
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
