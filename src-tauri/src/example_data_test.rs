//! Executable spec for the example datasets in `dataset for test/examples/`.
//!
//! Those files are the user-facing exemplars of every import format the app accepts
//! (see the README in that folder). This module parses each one with the SAME parser
//! the ribbon import uses, so any parser change that would break the published
//! examples fails the normal `cargo test` gate loudly — the examples can never drift
//! from what the app actually accepts. Regenerate the files with
//! `py -3 tools/make_example_data.py` (deterministic; a clean regeneration is a no-op).

use crate::parsers;
use std::path::PathBuf;

fn examples_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = src-tauri; the examples live beside it in the repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dataset for test/examples")
}

fn example(name: &str) -> String {
    let p = examples_dir().join(name);
    assert!(p.exists(), "example file missing: {} — run py -3 tools/make_example_data.py", p.display());
    p.to_string_lossy().into_owned()
}

#[test]
fn las_examples_import_end_to_end() {
    let db_path = std::env::temp_dir().join("sandibumi_example_data_test.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");

    let paths: Vec<String> =
        ["SANDI-01.las", "SANDI-02.las", "SANDI-03.las"].iter().map(|f| example(f)).collect();
    let results = crate::ingest::import_las_files(&conn, &paths, None);
    assert_eq!(results.len(), 3);
    for r in &results {
        assert!(r.error.is_none(), "{}: {:?}", r.path, r.error);
        assert!(r.rows > 390, "{}: only {} rows", r.path, r.rows);
    }
    assert_eq!(results[0].well_name.as_deref(), Some("SANDI-01"));

    // The six standard curves landed (GR finite over the whole section)…
    let w1 = results[0].well_id.as_deref().unwrap();
    let gr_n: i64 = conn
        .query_row(
            "SELECT count(*) FROM standard_curves WHERE well_id = ?1 AND NOT isnan(gr)",
            duckdb::params![w1],
            |r| r.get(0),
        )
        .unwrap();
    assert!(gr_n > 390, "GR finite samples: {gr_n}");

    // …and the beyond-the-six curves (PEF, CALI) reached the generic store as set RAW,
    // with the deliberate 1-m NPHI/PEF null gap stored as NaN, not as -999.25.
    for mnem in ["PEF", "CALI"] {
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM curve_samples s JOIN curve_meta m ON s.curve_id = m.curve_id
                 WHERE m.well_id = ?1 AND m.set_name = 'RAW' AND upper(m.mnemonic) = ?2",
                duckdb::params![w1, mnem],
                |r| r.get(0),
            )
            .unwrap();
        assert!(n > 390, "{mnem} rows in generic store: {n}");
    }
    let pef_bad: i64 = conn
        .query_row(
            "SELECT count(*) FROM curve_samples s JOIN curve_meta m ON s.curve_id = m.curve_id
             WHERE m.well_id = ?1 AND upper(m.mnemonic) = 'PEF' AND s.value < -900",
            duckdb::params![w1],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pef_bad, 0, "NULL sentinel leaked into stored PEF values");
    drop(conn);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

/// The malformed exemplars behave exactly as the manual test plan documents
/// (T-IMP-03 / T-IMP-04): duplicated depths require an explicit policy before commit;
/// an all-NULL depth column still errors cleanly and commits no orphan well.
#[test]
fn malformed_las_exemplars_fail_the_documented_way() {
    let db_path = std::env::temp_dir().join("sandibumi_example_badlas_test.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");

    let dup = &crate::ingest::import_las_files(&conn, &[example("bad_dup_depth.las")], None)[0];
    assert!(
        dup.error.as_deref().is_some_and(|error| {
            error.contains("5 repeated depth row(s)")
                && error.contains("declared duplicate policy")
        }),
        "all five duplicates and the missing policy must be named: {:?}",
        dup.error
    );
    assert_eq!(dup.rows, 0, "an undecided duplicate policy cannot commit rows");
    assert!(dup.well_id.is_none(), "an undecided duplicate policy cannot create a well");

    let null = &crate::ingest::import_las_files(&conn, &[example("bad_null_depth.las")], None)[0];
    assert!(null.error.is_some(), "all-null depth column must be a clean error");
    assert!(null.well_id.is_none(), "no orphan well row may be committed");
    let wells: i64 = conn
        .query_row("SELECT count(*) FROM wells WHERE well_name = 'SANDI-BAD-NULL'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(wells, 0);
    drop(conn);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

/// T-IMP-04's other half: a LAS whose last data row was cut mid-line. Until 2026-07-31 there
/// was no exemplar for this at all, which is why the manual test could only be marked Blocked.
///
/// **A file that ends early looks exactly like a file that was always that short.** Nothing in
/// the ASCII block says "more was coming" — so the only evidence is arithmetic: the leftover
/// tokens do not divide into whole rows. Miss that and the tokens shift every subsequent value
/// one column left, putting GR into the resistivity slot and resistivity into neutron. The
/// numbers stay in range, the curves still plot, and the well is simply wrong.
///
/// Here the truncation is on the LAST row, so the shift has nothing after it to corrupt — and
/// the import still refuses, which is the point. Importing 39 good rows and quietly dropping the
/// 40th would leave a well nobody knows is incomplete. The check is on the file, not on the
/// damage the file happened to do.
#[test]
fn a_truncated_las_refuses_rather_than_importing_what_survived() {
    let db_path = std::env::temp_dir().join("sandibumi_example_trunclas_test.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");

    let res = &crate::ingest::import_las_files(&conn, &[example("bad_truncated.las")], None)[0];
    let err = res.error.as_deref().unwrap_or("");
    assert!(!err.is_empty(), "a truncated ASCII block must be an error, not a warning");
    assert!(
        err.contains("bad_truncated.las") && err.contains("line ") && err.contains("ASCII row"),
        "the message must locate the bad row and name its failed rule: {err}"
    );
    assert!(
        err.contains("truncated or corrupt"),
        "and name the likely cause: {err}"
    );
    assert!(res.well_id.is_none(), "no well row may be committed from a file that failed");
    assert_eq!(res.rows, 0, "no partial rows either — 39 of 40 is a well nobody knows is short");

    let wells: i64 = conn
        .query_row("SELECT count(*) FROM wells WHERE well_name = 'SANDI-BAD-TRUNC'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(wells, 0, "no orphan well");
    let samples: i64 =
        conn.query_row("SELECT count(*) FROM curve_samples", [], |r| r.get(0)).unwrap();
    assert_eq!(samples, 0, "and nothing in the generic store either");

    drop(conn);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

#[test]
fn core_csv_example_parses() {
    let cols = parsers::parse_core_csv(example("core_rcal_SANDI-01.csv")).unwrap();
    assert!(cols.depth.len() >= 10, "plug count: {}", cols.depth.len());
    // Porosity was written in percent; the importer's heuristic must land it in v/v.
    for &p in &cols.cpor {
        assert!(p > 0.05 && p < 0.45, "CPOR not in v/v after % conversion: {p}");
    }
    // Plug depths are deliberately OFF the 0.1524 grid (core stores at native depth).
    let off_grid = cols.depth.iter().any(|d| (d / 0.1524).fract().abs() > 1e-3);
    assert!(off_grid, "example core depths should not sit on the log grid");
}

#[test]
fn scal_examples_parse_all_three_shapes() {
    // Long/flat shape: 3 plugs x 8 pressure points; merged-cell context forward-fills.
    let long = parsers::parse_scal_csv(example("scal_pc_long_SANDI-01.csv")).unwrap();
    assert_eq!(long.len(), 24);
    let plug1: Vec<_> = long.iter().filter(|r| r.sample_no == Some(1)).collect();
    assert_eq!(plug1.len(), 8, "plug 1 context rows forward-filled");
    assert!(plug1.iter().all(|r| r.depth == Some(1522.35)), "depth forward-filled to every row");

    // Wide porous-plate shape: same plugs, header row holds the pressures.
    let wide = parsers::parse_scal_wide_csv(example("scal_porous_plate_wide_SANDI-01.csv")).unwrap();
    assert_eq!(wide.len(), 24);

    // Centrifuge blocks: the table header appears only above the FIRST block; the
    // parser must carry it into the second plug's rows.
    let cf = parsers::parse_scal_centrifuge_csv(example("scal_centrifuge_SANDI-01.csv")).unwrap();
    assert_eq!(cf.len(), 10);
    assert!(cf.iter().any(|r| r.sample_no == Some(5)), "second block rows survived");

    // Sw is stored as v/v in every shape.
    for r in long.iter().chain(wide.iter()).chain(cf.iter()) {
        assert!(r.sw > 0.0 && r.sw <= 1.0, "Sw not in v/v: {}", r.sw);
    }
}

/// Core import v2 exemplars: the multi-well delivery-shaped core CSV probes correctly and
/// routes per well into a project built from the SANDI LAS examples; the tab-delimited
/// multi-well XRD TXT routes through the aux importer the same way.
#[test]
fn multiwell_core_and_aux_examples_import_end_to_end() {
    let db_path = std::env::temp_dir().join("sandibumi_example_multiwell_test.duckdb");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
    let conn = crate::db::init_db(db_path.to_str().unwrap()).expect("init_db");
    let paths: Vec<String> =
        ["SANDI-01.las", "SANDI-02.las", "SANDI-03.las"].iter().map(|f| example(f)).collect();
    let results = crate::ingest::import_las_files(&conn, &paths, None);
    assert!(results.iter().all(|r| r.error.is_none()));

    // Probe sees the delivery shape: WN column, units row, percent porosity.
    let core = example("core_rcal_multiwell.csv");
    let probe = parsers::probe_core_table(&core).unwrap();
    assert_eq!(probe.well, Some(2), "WN is the well column");
    assert!(probe.units_row_skipped);
    assert_eq!(probe.wells.len(), 3, "SANDI-01/02/03 in one file");
    assert!(probe.percent_roles.iter().any(|r| r == "CPOR"));
    assert_eq!(probe.depth_unit_guess.as_deref(), Some("m"), "units row says M");

    // Commit under the probed mapping: every well receives its own plugs.
    // Every column the four core measurements don't claim rides along as point data —
    // the wide-lab-export case (LITH text beside numeric SO), confirmed in the wizard.
    let claimed: Vec<usize> = [probe.well, probe.depth, probe.cpor, probe.cperm, probe.cgd, probe.csw]
        .into_iter()
        .flatten()
        .collect();
    let extras: Vec<usize> = (0..probe.headers.len()).filter(|i| !claimed.contains(i)).collect();
    assert!(
        extras.iter().any(|&i| probe.headers[i] == "LITH"),
        "the exemplar carries a text column the core schema has no slot for"
    );
    let mapping = parsers::CoreMapping {
        well: probe.well,
        depth: probe.depth.unwrap(),
        cpor: probe.cpor,
        cperm: probe.cperm,
        cgd: probe.cgd,
        csw: probe.csw,
        extras,
    };
    let res =
        crate::ingest::import_core_table(&conn, &core, &mapping, probe.depth_unit_guess.as_deref(), None, None, None, false);
    assert!(res.error.is_none(), "{:?}", res.error);
    assert_eq!(res.wells_imported, 3, "all three wells routed by name: {:?}", res.outcomes);
    for r in &results {
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM core_data WHERE well_id = ?1",
                duckdb::params![r.well_id.as_deref().unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert!(n > 5, "{}: {} core rows", r.well_name.as_deref().unwrap(), n);
    }
    // The extra columns landed as point data under the default CORE dataset, typed per
    // cell: SO_1 numeric, LITH text.
    assert!(res.extra_rows > 0, "extra columns stored: {:?}", res.extra_items);
    let aux = crate::db::list_aux_data(&conn, results[0].well_id.as_deref().unwrap(), Some("CORE")).unwrap();
    assert!(aux.iter().any(|r| r.item == "LITH" && r.value_text.is_some()));
    assert!(aux.iter().any(|r| r.item == "SO_1" && r.value_num.is_some()));

    // Tab-delimited multi-well XRD TXT routes through the aux importer.
    let aux = crate::ingest::import_aux_file(
        &conn,
        results[0].well_id.as_deref().unwrap(), // fallback, unused: the file routes itself
        "XRD",
        &example("xrd_multiwell.txt"),
        None,
        false,
    );
    assert!(aux.error.is_none(), "{:?}", aux.error);
    assert_eq!(aux.wells_imported, 3, "rows routed to all three wells: {:?}", aux.notes);
    drop(conn);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("duckdb.wal"));
}

#[test]
fn tops_deviation_locations_examples_parse() {
    let (has_well, tops) = parsers::parse_tops_file(example("tops_multiwell.csv")).unwrap();
    assert!(has_well);
    assert_eq!(tops.len(), 9, "3 wells x 3 tops");
    assert!(tops.iter().any(|t| t.well.as_deref() == Some("SANDI-03") && t.top_name == "TOP_SAND_A"));

    let dev = parsers::parse_deviation_csv(example("deviation_SANDI-02.csv")).unwrap();
    assert_eq!(dev.md.len(), 34);
    assert!(dev.md.windows(2).all(|w| w[0] < w[1]), "stations sorted by MD");
    assert!(dev.inc.iter().cloned().fold(0.0f32, f32::max) > 24.0, "build section reaches 25 deg");

    let (has_well, locs) = parsers::parse_locations_file(example("well_locations.csv")).unwrap();
    assert!(has_well);
    assert_eq!(locs.len(), 3);
    assert!(locs.iter().all(|l| l.zone.as_deref() == Some("UTM 50S")));
}

#[test]
fn aux_interval_examples_parse() {
    let petro = parsers::parse_interval_file(example("petrography_SANDI-01.csv")).unwrap();
    assert_eq!(petro.rows.len(), 6);
    assert!(petro.items.contains(&"LITHOLOGY".to_string()));
    assert!(petro.rows.iter().all(|(_, base, _)| base.is_some()), "petrography rows are intervals");

    let xrd = parsers::parse_interval_file(example("xrd_SANDI-01.csv")).unwrap();
    assert_eq!(xrd.rows.len(), 5);
    assert_eq!(xrd.items.len(), 9, "9 mineral columns");
    assert!(xrd.rows.iter().all(|(_, base, _)| base.is_none()), "XRD rows are point samples");

    let perf = parsers::parse_interval_file(example("perforations_SANDI-01.csv")).unwrap();
    assert_eq!(perf.rows.len(), 3);
    assert!(perf.items.contains(&"STATUS".to_string()));
}

fn malformed_corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dio-malformed")
}

const REGISTERED_FILE_READERS: &[&str] = &[
    "intake::probe",
    "intake::probe_arrays",
    "intake::read_wide",
    "parsers::extract_well_name",
    "parsers::parse_core_csv",
    "parsers::parse_core_csv_with_depth_column",
    "parsers::parse_core_table_mapped",
    "parsers::parse_csv_export",
    "parsers::parse_deviation_csv",
    "parsers::parse_interval_file",
    "parsers::parse_las_2",
    "parsers::parse_las_2_all",
    "parsers::parse_las_2_all_with_channel_nulls",
    "parsers::parse_las_2_all_with_null_rules",
    "parsers::parse_las_2_with_channel_nulls",
    "parsers::parse_las_2_with_null_rules",
    "parsers::parse_las_2_with_unit_designation",
    "parsers::parse_las_directory",
    "parsers::parse_locations_file",
    "parsers::parse_scal_centrifuge_csv",
    "parsers::parse_scal_csv",
    "parsers::parse_scal_wide_csv",
    "parsers::parse_tops_file",
    "parsers::probe_core_table",
    "parsers::read_text_file",
    "parsers::read_text_file_with_encoding",
    "parsers::sniff_scal_format",
];

fn exercise_registered_reader(reader: &str, path: &std::path::Path) -> Result<(), String> {
    let text_path = path.to_string_lossy();
    let mapping = parsers::CoreMapping {
        well: None,
        depth: 0,
        cpor: None,
        cperm: None,
        cgd: None,
        csw: None,
        extras: vec![],
    };
    let opts = crate::intake::TableOptions::default();
    let roles: Vec<String> = Vec::new();
    match reader {
        "parsers::read_text_file" => parsers::read_text_file(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::read_text_file_with_encoding" => {
            parsers::read_text_file_with_encoding(path).map(|_| ()).map_err(|e| e.to_string())
        }
        "parsers::parse_csv_export" => parsers::parse_csv_export(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_las_2" => parsers::parse_las_2(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_las_2_all" => parsers::parse_las_2_all(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_las_2_with_channel_nulls" => {
            parsers::parse_las_2_with_channel_nulls(path, &parsers::ChannelNullValues::new())
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        "parsers::parse_las_2_with_null_rules" => {
            parsers::parse_las_2_with_null_rules(
                path,
                &parsers::ChannelNullValues::new(),
                &[],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
        "parsers::parse_las_2_with_unit_designation" => {
            parsers::parse_las_2_with_unit_designation(
                path,
                &parsers::ChannelNullValues::new(),
                &[],
                None,
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
        "parsers::parse_las_2_all_with_channel_nulls" => {
            parsers::parse_las_2_all_with_channel_nulls(
                path,
                &parsers::ChannelNullValues::new(),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
        "parsers::parse_las_2_all_with_null_rules" => {
            parsers::parse_las_2_all_with_null_rules(
                path,
                &parsers::ChannelNullValues::new(),
                &[],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
        "parsers::extract_well_name" => parsers::extract_well_name(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_core_csv" => parsers::parse_core_csv(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_core_csv_with_depth_column" => {
            parsers::parse_core_csv_with_depth_column(path, None)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        "parsers::parse_scal_csv" => parsers::parse_scal_csv(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_scal_wide_csv" => parsers::parse_scal_wide_csv(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_scal_centrifuge_csv" => {
            parsers::parse_scal_centrifuge_csv(path).map(|_| ()).map_err(|e| e.to_string())
        }
        "parsers::sniff_scal_format" => parsers::sniff_scal_format(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_deviation_csv" => parsers::parse_deviation_csv(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_las_directory" => parsers::parse_las_directory(
            path.parent().expect("a corpus fixture always has a parent directory"),
        )
        .map(|_| ())
        .map_err(|e| e.to_string()),
        "parsers::parse_tops_file" => parsers::parse_tops_file(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_locations_file" => {
            parsers::parse_locations_file(path).map(|_| ()).map_err(|e| e.to_string())
        }
        "parsers::parse_interval_file" => parsers::parse_interval_file(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::probe_core_table" => parsers::probe_core_table(path).map(|_| ()).map_err(|e| e.to_string()),
        "parsers::parse_core_table_mapped" => {
            parsers::parse_core_table_mapped(path, &mapping).map(|_| ()).map_err(|e| e.to_string())
        }
        "intake::probe" => crate::intake::probe(&text_path, &opts).map(|_| ()).map_err(|e| e.to_string()),
        "intake::read_wide" => {
            crate::intake::read_wide(&text_path, &opts, &roles, false).map(|_| ()).map_err(|e| e.to_string())
        }
        "intake::probe_arrays" => crate::intake::probe_arrays(&text_path, &opts, &roles, false)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        _ => Err(format!("reader '{reader}' is registered but has no corpus adapter")),
    }
}

fn discovered_file_readers(module: &str, path: &std::path::Path) -> std::collections::BTreeSet<String> {
    let source = parsers::read_text_file(path).expect("reader source must be readable by the shared text boundary");
    source
        .lines()
        .filter_map(|line| {
            let declaration = line.trim_start().strip_prefix("pub fn ")?;
            let name = declaration.split(['(', '<']).next()?.trim();
            let file_reader = name != "parse_number"
                && (name.starts_with("parse_")
                || name.starts_with("read_")
                || name.starts_with("extract_")
                || name.starts_with("sniff_")
                || name.starts_with("probe_")
                || (module == "intake" && name == "probe"));
            file_reader.then(|| format!("{module}::{name}"))
        })
        .collect()
}

/// SB-DIO-061 / SB-DIO-T91..T94. The four-part malformed-input contract and the
/// synthetic-fixture rule are specified in `docs/PRD_v2/21_data-io.md` §§4.10, 6.10 and 7.1 O-9.
#[test]
fn malformed_input_is_located_counted_named_bounded_and_every_reader_runs_the_corpus_in_ci() {
    let corpus = malformed_corpus_dir();
    let malformed = [
        corpus.join("malformed-short-las-row.las"),
        corpus.join("malformed-bad-cells.csv"),
    ];
    for fixture in &malformed {
        assert!(fixture.is_file(), "malformed corpus fixture is missing: {}", fixture.display());
    }

    // T94: source-derived inventory. A new public file reader changes `discovered`, and CI fails
    // until an adapter is added above; a hand-maintained list cannot silently become stale.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut discovered = discovered_file_readers("parsers", &src.join("parsers.rs"));
    discovered.extend(discovered_file_readers("intake", &src.join("intake.rs")));
    let registered: std::collections::BTreeSet<String> =
        REGISTERED_FILE_READERS.iter().map(|name| (*name).to_string()).collect();
    assert_eq!(discovered, registered, "wire every new reader into the malformed corpus before it ships");

    // T91: the complete matrix runs in a worker with a deadline. Every call is also catch_unwind'd
    // so one malformed fixture cannot take down the test process without naming reader and file.
    let fixtures = malformed.to_vec();
    let (send, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut calls = 0usize;
        for fixture in fixtures {
            for reader in REGISTERED_FILE_READERS {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = exercise_registered_reader(reader, &fixture);
                }));
                if result.is_err() {
                    let _ = send.send(Err(format!("{reader} panicked on {}", fixture.display())));
                    return;
                }
                calls += 1;
            }
        }
        let _ = send.send(Ok(calls));
    });
    let calls = recv
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the malformed corpus reader matrix hung")
        .expect("a malformed fixture panicked a reader");
    assert_eq!(calls, malformed.len() * REGISTERED_FILE_READERS.len());

    // T92: the LAS diagnostic names the file, first bad line, affected count and failed rule.
    let short = &malformed[0];
    for error in [parsers::parse_las_2(short).unwrap_err(), parsers::parse_las_2_all(short).unwrap_err()] {
        let message = error.to_string();
        assert!(message.contains("malformed-short-las-row.las"), "file absent: {message}");
        assert!(message.contains("line 12"), "line absent: {message}");
        assert!(message.contains("2 value(s)"), "affected count absent: {message}");
        assert!(message.contains("~C declares 3 columns"), "failed rule absent: {message}");
    }
    let bad_cells = crate::intake::probe(
        &malformed[1].to_string_lossy(),
        &crate::intake::TableOptions::default(),
    )
    .unwrap();
    assert!(bad_cells.path.ends_with("malformed-bad-cells.csv"));
    assert_eq!(bad_cells.preview_bad, vec![(1, 1), (2, 2)], "row/column location and count");
    assert!(bad_cells.notes.iter().any(|note| note.contains("2 cell(s)") && note.contains("did not read as a number")));

    // T93: one valid, synthetic LAS seed is cut inside its final logical record at 100 distinct
    // byte offsets. Both LAS readers must fail cleanly and locate every cut; none may accept a
    // partial prefix as a shorter delivery.
    let seed = corpus.join("truncation-seed.las");
    assert!(parsers::parse_las_2(&seed).is_ok() && parsers::parse_las_2_all(&seed).is_ok());
    let bytes = std::fs::read(&seed).unwrap();
    let marker = b"\n1001.0000";
    let last_row = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .map(|index| index + 1)
        .expect("seed has a second data record");
    let temp = std::env::temp_dir().join(format!("sandibumi-dio-061-{}", std::process::id()));
    std::fs::create_dir_all(&temp).unwrap();
    for byte_offset in 1..=100usize {
        let path = temp.join(format!("truncated-{byte_offset:03}.las"));
        std::fs::write(&path, &bytes[..last_row + byte_offset]).unwrap();
        for error in [parsers::parse_las_2(&path).unwrap_err(), parsers::parse_las_2_all(&path).unwrap_err()] {
            let message = error.to_string();
            assert!(message.contains(path.file_name().unwrap().to_str().unwrap()), "file absent: {message}");
            assert!(message.contains("line ") && message.contains("ASCII row"), "located rule absent: {message}");
        }
        std::fs::remove_file(&path).unwrap();
    }
    std::fs::remove_dir(&temp).unwrap();
}
