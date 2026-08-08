//! LAS 2.0 export: one well's standard + computed curves on the standard depth grid.
//! NaN (missing) writes as the conventional -999.25 null value.

use crate::equations::fetch_curve_frame;
use duckdb::{params, Connection};
use sha2::{Digest, Sha256};
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

const PROVENANCE_PREFIX: &str = "SANDIBUMI_PROVENANCE_V1 ";
const MODEL_PROVENANCE_PREFIX: &str = "SANDIBUMI_MODEL_PROVENANCE_V1 ";

fn collect_model_ids(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(id) = map.get("model_id").and_then(serde_json::Value::as_str) {
                if !out.iter().any(|seen| seen == id) {
                    out.push(id.to_string());
                }
            }
            for child in map.values() {
                collect_model_ids(child, out);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_model_ids(child, out);
            }
        }
        _ => {}
    }
}

/// LAS 2.0 leaves `~O` as free text. SandiBumi's house convention is one compact JSON object per
/// prefixed line: still readable without software, and independently parseable without relying on
/// punctuation in a prose sentence. The prefix versions the convention, not the run record.
fn provenance_lines(
    conn: &Connection,
    well_id: &str,
    curve_names: &[String],
) -> Result<Vec<String>, String> {
    let standard = ["GR", "RES_DEEP", "NPHI", "RHOB", "DT", "SP"];
    let mut lines = Vec::new();

    for name in curve_names {
        let upper = name.trim().to_uppercase();
        if standard.contains(&upper.as_str()) {
            lines.push(format!(
                "{PROVENANCE_PREFIX}{}",
                serde_json::json!({ "curve": upper, "origin": "measured" })
            ));
            continue;
        }

        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves
                 WHERE well_id = ?1 AND upper(curve_name) = ?2",
            )
            .map_err(|e| e.to_string())?;
        let set_ids: Vec<Option<String>> = stmt
            .query_map(params![well_id, upper], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<duckdb::Result<_>>()
            .map_err(|e| e.to_string())?;
        if set_ids.is_empty() {
            lines.push(format!(
                "{PROVENANCE_PREFIX}{}",
                serde_json::json!({ "curve": upper, "origin": "measured" })
            ));
            continue;
        }
        if set_ids.len() != 1 || set_ids[0].is_none() {
            return Err(format!(
                "computed curve '{name}' has no single live ancestry record; export refused because its method and parameters cannot be carried"
            ));
        }
        let set_id = set_ids[0].as_deref().expect("checked above");
        let (set_name, version, module, params_json, inputs_json, created_at): (
            String,
            i64,
            String,
            Option<String>,
            Option<String>,
            String,
        ) = conn
            .query_row(
                "SELECT set_name, version, module, params_json, inputs_json,
                        strftime(created_at, '%Y-%m-%d %H:%M')
                 FROM log_sets WHERE set_id = ?1",
                params![set_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .map_err(|_| {
                format!(
                    "computed curve '{name}' cites missing log-set record '{set_id}'; export refused"
                )
            })?;
        let params_text = params_json
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| format!("computed curve '{name}' has no recorded parameters; export refused"))?;
        let parameters: serde_json::Value = serde_json::from_str(params_text)
            .map_err(|e| format!("computed curve '{name}' has invalid parameter JSON: {e}"))?;
        let inputs: serde_json::Value = match inputs_json.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => serde_json::from_str(text)
                .map_err(|e| format!("computed curve '{name}' has invalid input JSON: {e}"))?,
            _ => serde_json::Value::Array(Vec::new()),
        };
        lines.push(format!(
            "{PROVENANCE_PREFIX}{}",
            serde_json::json!({
                "curve": upper,
                "origin": "computed",
                "method": module,
                "parameters": parameters,
                "inputs": inputs,
                "log_set": set_name,
                "version": version,
                "run_date": created_at,
            })
        ));

        if module.starts_with("ml:") {
            let mut model_ids = Vec::new();
            collect_model_ids(&parameters, &mut model_ids);
            if model_ids.is_empty() {
                return Err(format!(
                    "model-derived curve '{name}' has no saved model identity in its run record; export refused"
                ));
            }
            for model_id in model_ids {
                let (info, artifact) = crate::db::get_ml_model(conn, &model_id).map_err(|_| {
                    format!(
                        "model-derived curve '{name}' cites unavailable model '{model_id}'; export refused"
                    )
                })?;
                let mut record = serde_json::to_value(info).map_err(|e| e.to_string())?;
                let object = record
                    .as_object_mut()
                    .ok_or_else(|| "saved model record did not serialize as an object".to_string())?;
                object.insert(
                    "artifact_sha256".into(),
                    serde_json::Value::String(format!("{:x}", Sha256::digest(&artifact))),
                );
                lines.push(format!(
                    "{MODEL_PROVENANCE_PREFIX}{}",
                    serde_json::json!({ "curve": upper, "record": record })
                ));
            }
        }
    }
    Ok(lines)
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
    let provenance = provenance_lines(conn, well_id, &curve_names)?;
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
        writeln!(w, "~Other Information")?;
        writeln!(w, "# SandiBumi provenance: prefixed JSON Lines; convention version 1")?;
        for line in &provenance {
            writeln!(w, " {line}")?;
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
        let set = crate::equations::create_log_set(
            conn,
            &id.to_string(),
            &crate::equations::LogSetSpec {
                set_name: "INTERP".into(),
                module: "vsh_gr".into(),
                params_json: serde_json::json!({ "gr_clean": 25.0, "gr_shale": 125.0 }).to_string(),
                inputs_json: serde_json::json!(["GR"]).to_string(),
            },
        )
        .unwrap();
        crate::equations::write_computed_curves_versioned(
            conn,
            &id.to_string(),
            &depth,
            &[("Vsh_final", &vsh)],
            &set.0,
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

    fn prefixed_json(text: &str, prefix: &str) -> Vec<serde_json::Value> {
        text.lines()
            .filter_map(|line| line.trim().strip_prefix(prefix))
            .map(|json| serde_json::from_str(json).expect("provenance line must be JSON"))
            .collect()
    }

    /// SB-DIO-051 / SB-DIO-T71..T73. Required fields are specified in
    /// `docs/PRD_v2/21_data-io.md` §4.10 and `04_CORE_REQUIREMENTS.md` SB-CORE-014.
    #[test]
    fn every_las_export_carries_measured_computed_and_model_provenance_in_the_file() {
        // The measured-only side: `~O` is not conditional on there being a computed curve.
        let measured = Connection::open_in_memory().unwrap();
        db::create_schema(&measured).unwrap();
        let measured_id = Uuid::new_v4();
        db::insert_well(&measured, measured_id, "MEASURED-ONLY", None, None, None).unwrap();
        let measured_depth = vec![1000.0_f32, 1000.5];
        db::insert_standard_curves(
            &measured,
            measured_id,
            measured_depth,
            vec![50.0, 55.0],
            vec![2.0; 2],
            vec![0.2; 2],
            vec![2.4; 2],
            vec![f32::NAN; 2],
            vec![f32::NAN; 2],
        )
        .unwrap();
        let measured_dest = tmp_path("measured-provenance");
        export_las(&measured, &measured_id.to_string(), measured_dest.to_str().unwrap()).unwrap();
        let measured_text = crate::parsers::read_text_file(&measured_dest).unwrap();
        let _ = std::fs::remove_file(&measured_dest);
        let measured_rows = prefixed_json(&measured_text, PROVENANCE_PREFIX);
        assert_eq!(measured_rows.len(), 6, "every written standard curve needs a record");
        assert!(measured_rows.iter().all(|row| row["origin"] == "measured"));

        let conn = Connection::open_in_memory().unwrap();
        let (id, _, _) = seed(&conn);
        let well_id = id.to_string();
        let model_bytes = b"synthetic saved model artifact";
        let features = vec!["GR".to_string(), "RHOB".to_string()];
        let trained_on = vec!["TRAIN-A".to_string(), "TRAIN-B".to_string()];
        let (model_id, model_name) = crate::db::insert_ml_model(
            &conn,
            &crate::db::NewMlModel {
                name: "VSH_RF",
                task: "regression",
                algorithm: "rf",
                feature_curves: &features,
                target_curve: Some("VSH"),
                params_json: r#"{"n_estimators":17,"seed":42}"#,
                metrics_json: r#"{"r2_blind":0.61}"#,
                trained_on: &trained_on,
                n_train: 12,
                standardize: true,
                note: None,
                data: model_bytes,
                train_hash: Some("training-row-hash"),
                training_json: Some(r#"{"input_set":"WIRE","wells":["TRAIN-A","TRAIN-B"]}"#),
                runtime_json: Some(r#"{"python":"3.12","sklearn":"1.7"}"#),
                sklearn_version: Some("1.7"),
            },
        )
        .unwrap();
        let run_params = serde_json::json!({
            "model_id": model_id,
            "model_name": model_name,
            "target": "VSH",
            "blind": { "performed": true, "metric": "R2", "value": 0.61,
                       "answers_new_well": true, "n_blind_wells": 1, "n_blind_rows": 4 },
            "train_hash": "training-row-hash",
            "trained_on": ["TRAIN-A", "TRAIN-B"],
        });
        let (set_id, _) = crate::equations::create_log_set(
            &conn,
            &well_id,
            &crate::equations::LogSetSpec {
                set_name: "PREDICTED".into(),
                module: "ml:rf".into(),
                params_json: run_params.to_string(),
                inputs_json: serde_json::json!(["GR", "RHOB"]).to_string(),
            },
        )
        .unwrap();
        let depth: Vec<f32> = (0..6).map(|i| 2000.0 + i as f32 * 0.5).collect();
        let predicted = vec![0.2_f32; depth.len()];
        crate::equations::write_computed_curves_versioned(
            &conn,
            &well_id,
            &depth,
            &[("VSH_PRED", &predicted)],
            &set_id,
        )
        .unwrap();

        let dest = tmp_path("provenance");
        export_las(&conn, &well_id, dest.to_str().unwrap()).unwrap();
        let text = crate::parsers::read_text_file(&dest).unwrap();
        let _ = std::fs::remove_file(&dest);
        assert!(text.contains("~Other Information"));
        let rows = prefixed_json(&text, PROVENANCE_PREFIX);
        let gr = rows.iter().find(|row| row["curve"] == "GR").unwrap();
        assert_eq!(gr["origin"], "measured");
        let vsh = rows.iter().find(|row| row["curve"] == "VSH_FINAL").unwrap();
        assert_eq!(vsh["origin"], "computed");
        assert_eq!(vsh["method"], "vsh_gr");
        assert_eq!(vsh["parameters"]["gr_clean"], 25.0);
        assert_eq!(vsh["parameters"]["gr_shale"], 125.0);

        let models = prefixed_json(&text, MODEL_PROVENANCE_PREFIX);
        let model = models.iter().find(|row| row["curve"] == "VSH_PRED").unwrap();
        assert_eq!(model["record"]["feature_curves"], serde_json::json!(["GR", "RHOB"]));
        assert_eq!(model["record"]["params_json"], r#"{"n_estimators":17,"seed":42}"#);
        assert_eq!(model["record"]["train_hash"], "training-row-hash");
        assert_eq!(model["record"]["runtime_json"], r#"{"python":"3.12","sklearn":"1.7"}"#);
        let artifact_hash = model["record"]["artifact_sha256"].as_str().unwrap();
        assert_eq!(artifact_hash.len(), 64, "the fitted artifact has its own SHA-256 identity");

        // The refusal side: a legacy/unversioned computed curve cannot be relabelled as measured
        // or exported with invented ancestry merely to make the file complete.
        crate::equations::write_computed_curves_batch(
            &conn,
            &well_id,
            &depth,
            &[("LEGACY_NO_PROVENANCE", &predicted)],
        )
        .unwrap();
        let refused_dest = tmp_path("missing-provenance");
        let _ = std::fs::remove_file(&refused_dest);
        let refused = export_las(&conn, &well_id, refused_dest.to_str().unwrap()).unwrap_err();
        assert!(refused.contains("no single live ancestry record"), "{refused}");
        assert!(!refused_dest.exists(), "a refused export must not leave a partial file");
    }
}
