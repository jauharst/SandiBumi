//! SB-CORE-011 / SB-CORE-T16 acceptance test.
//!
//! CORRECTNESS: the expected value is exact equality between two independent executions, as
//! required by `docs/PRD_v2/04_CORE_REQUIREMENTS.md` SB-CORE-011/T16. The numeric LAS samples,
//! parameters and cutoffs below are structural fixtures, not adopted scientific defaults.

use crate::{chain, db, equations, ingest, workflow};
use duckdb::Connection;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

struct TemporaryFiles(Vec<PathBuf>);

impl TemporaryFiles {
    fn track(&mut self, path: PathBuf) -> PathBuf {
        self.0.push(path.clone());
        path
    }
}

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = std::fs::remove_file(path);
            let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        }
    }
}

#[derive(Debug)]
struct ReRunSnapshot {
    curve_blobs: BTreeMap<(String, String), Vec<u8>>,
    ancestry: BTreeMap<String, equations::CurveAncestry>,
    pay_summary: Vec<u8>,
}

fn fixture_las(condition: &str, gr_shift: f32, density_shift: f32) -> String {
    let rows = (0..8)
        .map(|index| {
            let depth = 1000.0 + index as f32 * 0.5;
            let gr = 30.0 + gr_shift + index as f32 * 8.0;
            let rhob = 2.30 + density_shift + index as f32 * 0.025;
            let nphi = 0.14 + index as f32 * 0.012;
            let resistivity = 18.0 - index as f32 * 1.25;
            format!("{depth:.1} {gr:.3} {rhob:.4} {nphi:.4} {resistivity:.3}\n")
        })
        .collect::<String>();
    format!(
        "~VERSION\nVERS. 2.0 :\nWRAP. NO :\n~WELL\nNULL. -999.25 :\nWELL. {condition} :\n\
         ~CURVE\nDEPT.M : depth\nGR.GAPI : gamma ray\nRHOB.G/CC : bulk density\n\
         NPHI.V/V : neutron porosity\nRES_DEEP.OHMM : deep resistivity\n~ASCII\n{rows}"
    )
}

fn representative_steps(rw: f64) -> Vec<chain::ChainStep> {
    vec![
        chain::ChainStep {
            module: "vsh_gr".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([("GR_MA".into(), 20.0), ("GR_SH".into(), 120.0)]),
            opts: HashMap::from([("OPT_GR".into(), "LINEAR".into())]),
        },
        chain::ChainStep {
            module: "phi_dn".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("RHO_SH".into(), 2.45),
                ("NPHI_SH".into(), 0.35),
                ("RHO_DSH".into(), 2.70),
            ]),
            opts: HashMap::from([("OPT_XPLOT".into(), "AVERAGE".into())]),
        },
        chain::ChainStep {
            module: "sw_indo".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("A".into(), 1.0),
                ("M".into(), 2.0),
                ("N".into(), 2.0),
                ("RW".into(), rw),
                ("RT_SH".into(), 4.0),
                ("SWE_IRR".into(), 0.05),
            ]),
            opts: HashMap::from([
                ("OPT_RW".into(), "CONSTANT".into()),
                ("OPT_INDO".into(), "FULL".into()),
            ]),
        },
    ]
}

fn well_ids(conn: &Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT well_id FROM wells ORDER BY well_id").unwrap();
    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn packed_curve_blobs(conn: &Connection) -> BTreeMap<(String, String), Vec<u8>> {
    let mut stmt = conn
        .prepare(
            "SELECT well_id, upper(curve_name), depth, value
             FROM computed_curves
             ORDER BY well_id, upper(curve_name), depth",
        )
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f32>(2)?,
                row.get::<_, f32>(3)?,
            ))
        })
        .unwrap();
    let mut samples: BTreeMap<(String, String), Vec<f32>> = BTreeMap::new();
    for row in rows {
        let (well_id, curve, depth, value) = row.unwrap();
        samples.entry((well_id, curve)).or_default().extend([depth, value]);
    }
    samples
        .into_iter()
        .map(|(identity, values)| (identity, bytemuck::cast_slice(&values).to_vec()))
        .collect()
}

fn execute_recorded_chain(project: &Path, rw: f64) -> ReRunSnapshot {
    let conn = db::init_db(project.to_str().unwrap()).expect("open isolated project copy");
    let ids = well_ids(&conn);
    assert_eq!(ids.len(), 2, "the full rerun fixture must exercise more than one well");
    let db = Mutex::new(conn);
    let registry = chain::new_registry();
    let job_id = Uuid::new_v4();
    let cancel = chain::register(&registry, job_id);
    let custody = workflow::test_run_custody();
    chain::run_chain(
        &db,
        &registry,
        job_id,
        &cancel,
        &representative_steps(rw),
        &ids,
        Some("REPRODUCIBLE_INTERPRETATION"),
        None,
        &custody,
        None,
    );
    match chain::status(&registry, job_id).expect("recorded chain status") {
        chain::ChainStatus::Completed { errors, .. } => {
            assert!(errors.is_empty(), "the recorded chain must complete cleanly: {errors:?}");
        }
        status => panic!("the recorded chain did not complete: {status:?}"),
    }

    let pay = workflow::run_pay_summary(
        &db,
        &workflow::PaySummaryRequest {
            well_ids: ids.clone(),
            vsh_max: 0.55,
            phie_min: 0.10,
            swe_max: 0.65,
            perm_min: None,
            input_set: None,
            skip_version: false,
            stats_only: true,
            custody: None,
        },
    )
    .expect("recorded pay summary");

    let conn = db.lock().unwrap();
    let curve_blobs = packed_curve_blobs(&conn);
    let ancestry = ids
        .iter()
        .map(|well_id| {
            (
                well_id.clone(),
                equations::curve_ancestry(&conn, well_id, "SWE")
                    .expect("the final curve carries the recorded chain ancestry"),
            )
        })
        .collect();
    ReRunSnapshot {
        curve_blobs,
        ancestry,
        pay_summary: serde_json::to_vec(&pay).expect("serialize pay summary deterministically"),
    }
}

#[test]
fn a_recorded_raw_import_to_pay_summary_rerun_produces_byte_identical_curve_blobs_and_an_identical_pay_summary(
) {
    let token = Uuid::new_v4();
    let mut temporary = TemporaryFiles(Vec::new());
    let low = temporary.track(std::env::temp_dir().join(format!("core011_low_{token}.las")));
    let high = temporary.track(std::env::temp_dir().join(format!("core011_high_{token}.las")));
    std::fs::write(&low, fixture_las("LOW_SHALE_RESPONSE", 0.0, 0.0)).unwrap();
    std::fs::write(&high, fixture_las("HIGH_SHALE_RESPONSE", 18.0, 0.08)).unwrap();

    let baseline = temporary.track(std::env::temp_dir().join(format!("core011_base_{token}.duckdb")));
    {
        let conn = db::init_db(baseline.to_str().unwrap()).expect("create baseline project");
        let paths = vec![low.to_string_lossy().to_string(), high.to_string_lossy().to_string()];
        let imported = ingest::import_las_files(&conn, &paths, None);
        assert_eq!(imported.len(), 2);
        assert!(
            imported.iter().all(|result| result.error.is_none() && result.rows == 8),
            "both deterministic LAS fixtures must import completely: {imported:?}"
        );
        let mut stmt = conn
            .prepare(
                "SELECT mnemonic, COALESCE(family, ''), set_name
                 FROM curve_meta
                 ORDER BY mnemonic, family, set_name",
            )
            .unwrap();
        let inventory = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert!(
            inventory
                .iter()
                .any(|(mnemonic, family, set)| mnemonic == "RES_DEEP" && family == "RES_DEEP" && set == "RAW"),
            "the delivered deep-resistivity channel must survive in the native RAW inventory: {inventory:?}"
        );
        conn.execute_batch("CHECKPOINT").unwrap();
    }

    let first_path = temporary.track(std::env::temp_dir().join(format!("core011_first_{token}.duckdb")));
    let second_path = temporary.track(std::env::temp_dir().join(format!("core011_second_{token}.duckdb")));
    let changed_path = temporary.track(std::env::temp_dir().join(format!("core011_changed_{token}.duckdb")));
    std::fs::copy(&baseline, &first_path).unwrap();
    std::fs::copy(&baseline, &second_path).unwrap();
    std::fs::copy(&baseline, &changed_path).unwrap();

    let first = execute_recorded_chain(&first_path, 0.10);
    let second = execute_recorded_chain(&second_path, 0.10);
    assert!(!first.curve_blobs.is_empty(), "an empty output cannot prove determinism");
    for required in ["VSH", "PHIE", "SWE"] {
        assert!(
            first.curve_blobs.keys().any(|(_, curve)| curve == required),
            "the representative chain must actually write {required}"
        );
    }
    assert_eq!(
        first.curve_blobs, second.curve_blobs,
        "SB-CORE-T16 requires byte-identical packed depth/value blobs"
    );
    assert_eq!(
        first.pay_summary, second.pay_summary,
        "SB-CORE-T16 requires the serialized pay summary to be identical"
    );
    assert_eq!(first.ancestry.keys().collect::<Vec<_>>(), second.ancestry.keys().collect::<Vec<_>>());
    for (well_id, original) in &first.ancestry {
        let replay = &second.ancestry[well_id];
        assert!(
            original.same_computation(replay),
            "the replay must retain identical scientifically material ancestry for {well_id}\n\
             original={original:#?}\nreplay={replay:#?}"
        );
        assert!(original.timestamp_utc_ms > 0 && replay.timestamp_utc_ms > 0);
    }

    // Pin the other side: the comparison must be sensitive to a recorded input change rather
    // than passing because it captured no numbers or compared only lengths.
    let changed = execute_recorded_chain(&changed_path, 0.50);
    assert_ne!(first.curve_blobs, changed.curve_blobs, "a changed recorded Rw must move curve bytes");
    assert_ne!(first.pay_summary, changed.pay_summary, "a changed recorded Rw must move the pay result");
}
