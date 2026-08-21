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
/// How far one line opens or closes a block, counting only braces that ARE block delimiters.
///
/// A brace inside a CHARACTER LITERAL is not one. Found the hard way (AUDIT-2026-08-20 finding
/// 65's increment): a source-scanning test in `db.rs` that looks for a line beginning with a
/// closing brace has to write that brace as `'}'`, and counting it closed this stripper's
/// `#[cfg(test)]` skip one level early — which leaked the whole rest of that test module into the
/// production inventory and reported its fixtures as ancestry bypasses. The failure is loud, but
/// it accuses the wrong file, so it is fixed HERE rather than left as something every future
/// author of a scanning test has to know.
///
/// Deliberately not a Rust lexer: the escape `'\\{'` and braces inside string literals are still
/// counted. This handles the case that actually occurs and stays a line of code, not a parser.
fn block_delimiter_balance(line: &str) -> i64 {
    let code = line.replace("'{'", "").replace("'}'", "");
    code.matches('{').count() as i64 - code.matches('}').count() as i64
}

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
            if line.replace("'{'", "").replace("'}'", "").contains('{') {
                skipping = true;
                depth = block_delimiter_balance(line);
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
            depth += block_delimiter_balance(line);
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

/// AUDIT-2026-08-20 finding 36. `computed_curves` carries no primary key by design, so nothing
/// at the store objects to a duplicate row — uniqueness rests entirely on the writers deleting
/// their target names before appending. `write_computed_curves_with_ancestry_clearing` deleted
/// only the declared stale family and not the curves it was itself writing, which was safe by
/// accident: its one caller passes a family that happens to cover its own outputs.
///
/// Pinned from BOTH sides, because either half alone has a lazy implementation that passes.
/// Deleting only the written curves would satisfy A and lose the retirement; deleting only the
/// declared family would satisfy B and is the defect.
#[test]
fn the_clearing_write_retires_the_declared_family_and_still_replaces_what_it_writes() {
    let conn = db::init_db(":memory:").unwrap();
    crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
    let well_id = "00000000-0000-0000-0000-000000000036";
    insert_fixture_well(&conn, well_id);
    let depth = [1000.0f32, 1000.5];

    let set_for = |outputs: &[&str], name: &str| {
        let mut record = complete_record();
        record.outputs = outputs
            .iter()
            .map(|curve| equations::AncestryOutput {
                curve: (*curve).into(),
                derivation: format!("acceptance_identity({curve})"),
            })
            .collect();
        let spec = equations::CompleteLogSetSpec::try_new(name, record).unwrap();
        equations::create_complete_log_set(&conn, well_id, &spec).unwrap().0
    };
    let rows = |curve: &str| -> Vec<f32> {
        conn.prepare(
            "SELECT value FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = ?2 ORDER BY depth",
        )
        .unwrap()
        .query_map(params![well_id, curve], |row| row.get(0))
        .unwrap()
        .collect::<duckdb::Result<_>>()
        .unwrap()
    };

    // Seed a curve that a later run stops producing - the case the declared family exists for.
    equations::write_computed_curves_with_ancestry_clearing(
        &conn, well_id, &depth, &[("RETIRED", &[0.1f32, 0.2][..])], &[], &set_for(&["RETIRED"], "SEED"),
    )
    .unwrap();
    assert_eq!(rows("RETIRED").len(), 2, "the fixture must actually be stored before it is retired");

    // A. A curve this call WRITES is replaced, not appended beside its old rows - with an EMPTY
    //    family, so nothing but the write itself can be doing the clearing.
    for value in [0.4f32, 0.5] {
        equations::write_computed_curves_with_ancestry_clearing(
            &conn, well_id, &depth, &[("KEPT", &[value, value][..])], &[],
            &set_for(&["KEPT"], "WRITE"),
        )
        .unwrap();
    }
    assert_eq!(
        rows("KEPT"),
        vec![0.5, 0.5],
        "a re-run must REPLACE its own curve; two sets of rows in a PK-less table would double \
         whatever a reader averages"
    );

    // B. And the declared family is still retired, even though it is not among the curves written.
    equations::write_computed_curves_with_ancestry_clearing(
        &conn, well_id, &depth, &[("KEPT", &[0.6f32, 0.6][..])], &["RETIRED".to_string()],
        &set_for(&["KEPT"], "CLEAR"),
    )
    .unwrap();
    assert!(rows("RETIRED").is_empty(), "the declared stale family must still be retired");
    assert_eq!(rows("KEPT"), vec![0.6, 0.6], "and the written curve is still replaced");

    // The archive is append-only throughout: four writes of two depths, none of them deleted.
    let archived: i64 = conn
        .query_row(
            "SELECT count(*) FROM computed_curves_archive WHERE well_id = ?1",
            params![well_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(archived, 8, "clearing the current store must never reach into the archive");
}

/// AUDIT-2026-08-20 finding 32's class: a citation that points at nothing.
///
/// Provenance discipline (CLAUDE.md) requires a default, a rule or a refusal to trace to a named
/// source. A citation naming a file that is not in the tree traces to nothing — and it reads
/// exactly like one that does, which is what lets it survive. Finding 32 caught four sites citing
/// `memory/method_workflow_standards.md`, a path that has never existed here; they were corrected
/// to `docs/workflow_standards.md`, and this is the sweep that stops the next one.
///
/// Pinned from both sides: every cited document must resolve, AND the acknowledged list may
/// shrink but never go stale — a file that arrives must lose its exception rather than keep it.
#[test]
fn every_document_this_code_cites_is_actually_in_the_tree() {
    // The one document cited across this codebase that is NOT on master. It is real and it is
    // 225 lines long, but it lives only on the unmerged branch `docs/prd-and-security-hardening`
    // (commit 18da8b0a, 2026-07-29): RELEASE.md, TARGET_ARCHITECTURE.md and V1_SCOPE.md were all
    // added there and none reached master, though the pull request that carried them was merged.
    //
    // The SUBSTANCE is on master — `docs/PRD_v2/22_database-model.md` states the backup-then-
    // migrate contract that "RELEASE §3.2" is cited for, including that a failed copy aborts. So
    // what dangles is the LABEL, in 34 places, one of which is the rule
    // `every_migration_that_copies_the_project_first_documents_that_a_failed_copy_aborts` enforces.
    //
    // Left as an exception rather than re-pointed: mapping §3.1/§3.2/§3.3/§5 onto PRD_v2 sections
    // would be authoring a governance mapping, and restoring a release policy to master is
    // Jauhar's call. Recorded in the audit triage notes for that decision.
    const CITED_BUT_ABSENT: &[&str] = &["docs/RELEASE.md"];

    let mut sources = Vec::new();
    rust_sources(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(), &mut sources);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();

    let mut dangling: Vec<String> = Vec::new();
    let mut acknowledged_seen: Vec<&str> = Vec::new();
    for path in &sources {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        for (line_no, line) in text.split('\n').enumerate() {
            for token in line.split(|c: char| !(c.is_alphanumeric() || "/._-".contains(c))) {
                if !token.starts_with("docs/") || !token.ends_with(".md") {
                    continue;
                }
                if CITED_BUT_ABSENT.contains(&token) {
                    acknowledged_seen.push(
                        CITED_BUT_ABSENT.iter().find(|known| *known == &token).unwrap(),
                    );
                    continue;
                }
                if !repo.join(token).exists() {
                    dangling.push(format!(
                        "{}:{} cites {token}, which is not in the tree",
                        path.file_name().unwrap().to_string_lossy(),
                        line_no + 1
                    ));
                }
            }
        }
    }
    dangling.sort();
    dangling.dedup();
    assert!(
        dangling.is_empty(),
        "a cited document must exist, or be acknowledged with the reason it does not:\n  {}",
        dangling.join("\n  ")
    );

    // The other side. An exception that has quietly become true is a stale note claiming a gap
    // that closed, and the next reader trusts it.
    for known in CITED_BUT_ABSENT {
        assert!(
            !repo.join(known).exists(),
            "{known} is in the tree now - delete its entry in CITED_BUT_ABSENT, the list may \
             shrink but must never go stale"
        );
        assert!(
            acknowledged_seen.contains(known),
            "{known} is acknowledged as cited-but-absent, but nothing cites it any more - \
             delete the entry rather than carrying it"
        );
    }
}
