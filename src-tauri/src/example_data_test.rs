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
