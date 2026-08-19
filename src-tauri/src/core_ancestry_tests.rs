//! SB-CORE-010 acceptance tests.
//!
//! These are correctness tests. Their expected structure comes directly from
//! `docs/PRD_v2/04_CORE_REQUIREMENTS.md` SB-CORE-010 and SB-CORE-T14/T15: module and version,
//! every input and log set, every parameter value and source, zone scope, actor, timestamp and
//! output derivation must be recorded; the same record must survive project Save As/reopen.

use crate::{db, equations};
use duckdb::params;
use serde_json::json;
use std::path::{Path, PathBuf};

fn complete_record() -> equations::CurveAncestry {
    equations::CurveAncestry {
        schema_version: equations::CURVE_ANCESTRY_SCHEMA_VERSION,
        method_derivation: None,
        module: "acceptance_identity".into(),
        module_version: env!("CARGO_PKG_VERSION").into(),
        inputs: vec![equations::AncestryInput {
            well_id: "fixture-well".into(),
            argument: "INPUT".into(),
            curve: "INPUT".into(),
            log_set: "RAW".into(),
            set_version: Some(1),
            set_id: "fixture-import-run".into(),
            chosen_curve_id: Some("fixture-import-run".into()),
            rule: Some(equations::CurveResolutionRule::ExplicitName),
            rejected_candidates: Vec::new(),
        }],
        parameters: vec![equations::AncestryParameter {
            name: "REFERENCE_VALUE".into(),
            value: json!(0.25),
            source: "SB-CORE-T14 structural acceptance fixture; not a scientific default".into(),
            resolution: Some(equations::ParameterResolution::Explicit),
            manifest_version: None,
            decision: None,
        }],
        parameter_state: None,
        zone_scope: equations::AncestryZoneScope::WholeWell,
        actor: equations::AncestryActor {
            kind: equations::AncestryActorKind::Human,
            identity: "acceptance-fixture-operator".into(),
        },
        timestamp_utc_ms: 1,
        outputs: vec![equations::AncestryOutput {
            curve: "OUTPUT".into(),
            derivation: "acceptance_identity(INPUT, REFERENCE_VALUE)".into(),
        }],
        depth_frame: None,
        zone_set: None,
        stochastic: None,
        applied_model: None,
        physics_attributes: Vec::new(),
    }
}

fn insert_fixture_well(conn: &duckdb::Connection, well_id: &str) {
    conn.execute(
        "INSERT INTO wells (well_id, well_name, field_name, td, kb) VALUES (?1, 'VALIDATION', NULL, NULL, NULL)",
        params![well_id],
    )
    .unwrap();
}

fn rust_sources(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Removes every inline `#[cfg(test)]` item. External test modules are excluded by filename below
/// because their cfg guard lives at the declaration site in `lib.rs`. Function-level fixture
/// writers must also be removed: they do not exist in a production build and therefore are not a
/// production ancestry bypass.
fn production_rust(source: &str) -> String {
    let mut kept = String::new();
    let mut pending_test_cfg = false;
    let mut skipping = false;
    let mut depth = 0i64;
    for line in source.lines() {
        if !skipping && line.trim_start().starts_with("#[cfg(test)]") {
            pending_test_cfg = true;
            continue;
        }
        if pending_test_cfg {
            if line.contains('{') {
                skipping = true;
                depth = line.matches('{').count() as i64 - line.matches('}').count() as i64;
                pending_test_cfg = false;
                if depth <= 0 {
                    skipping = false;
                }
                continue;
            }
            if line.trim_end().ends_with(';') {
                pending_test_cfg = false;
                continue;
            }
            // A multi-line cfg(test) function or module signature has not reached its opening
            // brace yet. Keep discarding it rather than leaking fixture code into the scan.
            continue;
        }
        if skipping {
            depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
            if depth <= 0 {
                skipping = false;
            }
            continue;
        }
        kept.push_str(line);
        kept.push('\n');
    }
    kept
}

/// Returns every production call site that could write computed values without first obtaining
/// the opaque complete-ancestry token. SB-CORE-T14 and SB-DBM-T13 share this inventory: the former
/// pins complete custody, while the latter proves no setting or deployment branch has a second
/// writer to switch to.
pub(crate) fn production_ancestry_bypass_violations() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&root, &mut files);
    let mut violations = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if name.ends_with("_test.rs")
            || name.ends_with("_tests.rs")
            || name == "pipeline_field_test.rs"
        {
            continue;
        }
        let source = production_rust(&crate::parsers::read_text_file(&path).unwrap());
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let definition = trimmed.starts_with("pub(crate) fn ") || trimmed.starts_with("fn ");
            for forbidden in [
                "write_computed_curve(",
                "write_computed_curves_batch(",
                "create_log_set(",
                "create_log_sets_batch(",
                "write_computed_curves_versioned(",
                "write_computed_curves_versioned_batch(",
                "db::update_computed_sample(",
            ] {
                if !definition && line.contains(forbidden) {
                    violations.push(format!(
                        "{}:{} uses {forbidden}",
                        path.display(),
                        line_no + 1
                    ));
                }
            }
            // Schema/project migrations are the only non-writer files allowed to mention raw
            // computed-table mutations. A new producer or interactive edit must go through the
            // opaque complete-ancestry API in equations.rs.
            if !matches!(name, "equations.rs" | "db.rs" | "project.rs") {
                for forbidden in [
                    "appender(\"computed_curves\")",
                    "INSERT INTO computed_curves",
                    "UPDATE computed_curves",
                    "DELETE FROM computed_curves",
                ] {
                    if line.contains(forbidden) {
                        violations.push(format!(
                            "{}:{} uses raw {forbidden}",
                            path.display(),
                            line_no + 1
                        ));
                    }
                }
            }
        }
    }
    violations
}

fn code_sources(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            code_sources(&path, out);
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs" | "ts")
        ) {
            out.push(path);
        }
    }
}

/// Enumerates the production app-preference and environment read surfaces used by SB-DBM-T13.
/// The inventory is generated from the whole Rust and TypeScript corpus rather than a hand-picked
/// setting list, so a new preference is included automatically in the no-bypass proof.
pub(crate) fn production_configuration_read_inventory() -> Vec<String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let roots = [manifest.join("src"), manifest.join("../src")];
    let mut files = Vec::new();
    for root in roots {
        code_sources(&root, &mut files);
    }
    let mut inventory = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if name.ends_with("_test.rs")
            || name.ends_with("_tests.rs")
            || name.ends_with(".test.ts")
            || name.ends_with(".spec.ts")
            || name == "pipeline_field_test.rs"
        {
            continue;
        }
        let text = crate::parsers::read_text_file(&path).unwrap();
        let source = if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            production_rust(&text)
        } else {
            text
        };
        for (line_no, line) in source.lines().enumerate() {
            if [
                "std::env::var(",
                "std::env::var_os(",
                "localStorage.getItem(",
                "sessionStorage.getItem(",
                "current_setting(",
                "FROM documents",
                "read_user_settings(",
            ]
            .iter()
            .any(|marker| line.contains(marker))
            {
                inventory.push(format!(
                    "{}:{} {}",
                    path.display(),
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }
    inventory.sort();
    inventory
}

#[test]
fn every_computed_curve_written_by_any_module_has_a_complete_ancestry_record() {
    // CORRECTNESS — required fields and refusal behavior are SB-CORE-010 / SB-CORE-T14.
    let conn = db::init_db(":memory:").unwrap();
    crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
    let well_id = "00000000-0000-0000-0000-000000000010";
    insert_fixture_well(&conn, well_id);
    db::insert_standard_curves(
        &conn,
        uuid::Uuid::parse_str(well_id).unwrap(),
        vec![1000.0, 1000.5],
        vec![50.0, 50.0],
        vec![f32::NAN; 2],
        vec![f32::NAN; 2],
        vec![f32::NAN; 2],
        vec![f32::NAN; 2],
        vec![f32::NAN; 2],
    )
    .unwrap();

    let mut missing_actor = complete_record();
    missing_actor.actor.identity.clear();
    let refused = equations::CompleteLogSetSpec::try_new("VALIDATION", missing_actor)
        .expect_err("an unnamed actor must refuse before a log-set or curve row is written");
    assert!(
        refused.contains("actor"),
        "the refusal must name the missing custody field: {refused}"
    );

    let mut missing_source = complete_record();
    missing_source.parameters[0].source.clear();
    let refused = equations::CompleteLogSetSpec::try_new("VALIDATION", missing_source)
        .expect_err("an unsourced value must refuse before a log-set or curve row is written");
    assert!(
        refused.contains("source"),
        "the refusal must name the missing custody field: {refused}"
    );

    let before: (i64, i64) = conn
        .query_row(
            "SELECT (SELECT count(*) FROM log_sets), (SELECT count(*) FROM computed_curves)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        before,
        (0, 0),
        "refused custody must not allocate a version or write a curve"
    );

    let spec = equations::CompleteLogSetSpec::try_new("VALIDATION", complete_record()).unwrap();
    let (set_id, _) = equations::create_complete_log_set(&conn, well_id, &spec).unwrap();
    equations::write_computed_curves_with_ancestry(
        &conn,
        well_id,
        &[1000.0, 1000.5],
        &[("OUTPUT", &[0.25, 0.26])],
        &set_id,
    )
    .unwrap();

    let ancestry = equations::curve_ancestry(&conn, well_id, "OUTPUT").unwrap();
    assert_eq!(
        ancestry,
        complete_record(),
        "the stored record must answer every SB-CORE-T14 field"
    );
    let disclosures = equations::curve_ancestry_disclosures(&conn, &[well_id.to_string()], None)
        .expect("the same complete record must be available to UI and deliverable surfaces");
    assert_eq!(
        disclosures.len(),
        1,
        "one live computed curve must disclose one ancestry record"
    );
    assert_eq!(disclosures[0].curve_name, "OUTPUT");
    assert_eq!(disclosures[0].set_name.as_deref(), Some("VALIDATION"));
    assert_eq!(disclosures[0].version, Some(1));
    assert_eq!(disclosures[0].ancestry, Some(complete_record()));
    let las_path =
        std::env::temp_dir().join(format!("sandibumi-ancestry-{}.las", std::process::id()));
    let _ = std::fs::remove_file(&las_path);
    crate::export::export_las(&conn, well_id, las_path.to_str().unwrap())
        .expect("a complete curve must export with its ancestry inside the LAS header");
    let las = crate::parsers::read_text_file(&las_path).unwrap();
    let exported = las
        .lines()
        .filter_map(|line| line.trim().strip_prefix("SANDIBUMI_PROVENANCE_V1 "))
        .map(|json| serde_json::from_str::<serde_json::Value>(json).unwrap())
        .find(|row| row["curve"] == "OUTPUT")
        .expect("the LAS header must carry the computed curve record");
    assert_eq!(
        exported["ancestry"],
        serde_json::to_value(complete_record()).unwrap(),
        "the LAS header must carry the complete record, not only a method name or partial params",
    );
    let _ = std::fs::remove_file(&las_path);

    let mut layout = crate::layout::standard_layout();
    layout.tracks[0].curves[0].curve_name = "OUTPUT".into();
    let composite_spec = crate::composite::CompositeSpec {
        well_id: well_id.to_string(),
        layout,
        depth_top: None,
        depth_bottom: None,
        scale: 500,
        page_size: crate::composite::PageSize::A4,
    };
    let composite = crate::composite::render_composite(&conn, &composite_spec)
        .expect("the standalone SVG composite must carry computed-curve ancestry");
    assert_eq!(composite.ancestry, disclosures);
    assert!(
        composite.pages[0]
            .svg
            .contains("sandibumi-curve-ancestry-v1"),
        "the SVG itself must embed the complete record rather than relying on live project state",
    );
    let composite_pdf = crate::composite::render_composite_pdf(&conn, &composite_spec)
        .expect("the standalone PDF composite must carry computed-curve ancestry");
    assert!(
        composite_pdf
            .windows(b"SANDIBUMI_CURVE_ANCESTRY_V1_BASE64".len())
            .any(|window| window == b"SANDIBUMI_CURVE_ANCESTRY_V1_BASE64"),
        "the PDF bytes must embed the complete record rather than relying on live project state",
    );
    let plot_ancestry_json = serde_json::to_string(&disclosures).unwrap();
    let plot_svg = crate::composite::embed_ancestry_json_in_svg(
        "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
        &plot_ancestry_json,
    )
    .unwrap();
    assert!(
        plot_svg.contains("sandibumi-curve-ancestry-v1"),
        "an ordinary SVG plot must carry the same complete record as a composite"
    );
    let plot_pdf = crate::composite::embed_ancestry_json_in_pdf(
        crate::composite::assemble_single_page_pdf("", 100.0, 100.0),
        &plot_ancestry_json,
    )
    .unwrap();
    assert!(
        plot_pdf
            .windows(b"SANDIBUMI_CURVE_ANCESTRY_V1_BASE64".len())
            .any(|window| window == b"SANDIBUMI_CURVE_ANCESTRY_V1_BASE64"),
        "an ordinary PDF plot must carry the same complete record as a composite"
    );
    // Minimal structurally valid PNG: signature followed by IEND. The ancestry helper inserts a
    // standards-compliant tEXt chunk before IEND; the image pixels and IEND remain untouched.
    let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x00IEND\xae\x42\x60\x82";
    let plot_png = crate::composite::embed_ancestry_json_in_png(png, &plot_ancestry_json).unwrap();
    assert!(
        plot_png
            .windows(b"SandiBumiCurveAncestry".len())
            .any(|window| window == b"SandiBumiCurveAncestry"),
        "an ordinary PNG plot must carry the complete record in a PNG metadata chunk"
    );
    let rows: (i64, i64) = conn
        .query_row(
            "SELECT count(*), count(set_id) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'OUTPUT'",
            params![well_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        rows,
        (2, 2),
        "a computed row cannot exist without one live ancestry identity"
    );

    // The opaque complete-set API is not enough if a production module can still reach a legacy
    // writer. Scan every Rust production surface, not a hand-picked module list. This catches a new
    // producer on the same commit that introduces it.
    let violations = production_ancestry_bypass_violations();
    assert!(
        violations.is_empty(),
        "production computed writers must require complete ancestry:\n{}",
        violations.join("\n")
    );
}

#[test]
fn a_complete_ancestry_record_round_trips_through_project_save_and_load() {
    // CORRECTNESS — persistence through project save/load is SB-CORE-010 / SB-CORE-T15.
    let root = std::env::temp_dir().join(format!("sandibumi-ancestry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let original = root.join("original.duckdb");
    let backup = root.join("backup.duckdb");
    let original_text = original.to_str().unwrap();
    let backup_text = backup.to_str().unwrap();
    let well_id = "00000000-0000-0000-0000-000000000015";

    let conn = db::init_db_resilient(original_text).unwrap();
    insert_fixture_well(&conn, well_id);
    let expected = complete_record();
    let spec = equations::CompleteLogSetSpec::try_new("VALIDATION", expected.clone()).unwrap();
    let (set_id, _) = equations::create_complete_log_set(&conn, well_id, &spec).unwrap();
    equations::write_computed_curves_with_ancestry(
        &conn,
        well_id,
        &[1000.0, 1000.5],
        &[("OUTPUT", &[0.25, 0.30])],
        &set_id,
    )
    .unwrap();
    db::engine_copy_to(&conn, backup_text).expect("Save As must engine-copy the live project");
    drop(conn);

    let reopened =
        db::init_db_resilient(backup_text).expect("the Save As copy must reopen normally");
    assert_eq!(
        equations::curve_ancestry(&reopened, well_id, "OUTPUT").unwrap(),
        expected,
        "the complete record must survive engine copy and normal reopen byte-for-byte"
    );
    let values: Vec<f32> = reopened
        .prepare("SELECT value FROM computed_curves WHERE well_id = ?1 AND curve_name = 'OUTPUT' ORDER BY depth")
        .unwrap()
        .query_map(params![well_id], |row| row.get(0))
        .unwrap()
        .collect::<duckdb::Result<_>>()
        .unwrap();
    assert_eq!(
        values,
        vec![0.25, 0.30],
        "the record must remain attached to the copied curve numbers"
    );
    drop(reopened);
    let _ = std::fs::remove_dir_all(&root);
}
