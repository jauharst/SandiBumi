//! SB-CORE-011 / SB-CORE-T16 acceptance test.
//!
//! CORRECTNESS: the expected value is exact equality between two independent executions, as
//! required by `docs/PRD_v2/04_CORE_REQUIREMENTS.md` SB-CORE-011/T16. The numeric LAS samples,
//! parameters and cutoffs below are structural fixtures, not adopted scientific defaults.

use crate::{chain, db, ingest, workflow};
use duckdb::Connection;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use uuid::Uuid;

const DBM016_CHILD: &str = "SANDIBUMI_DBM016_CHILD";
const DBM016_PROJECT: &str = "SANDIBUMI_DBM016_PROJECT";
const DBM016_OUTPUT: &str = "SANDIBUMI_DBM016_OUTPUT";
const DBM016_RW: &str = "SANDIBUMI_DBM016_RW";
const DBM016_TEST_NAME: &str = "core_determinism_tests::a_project_run_in_fresh_processes_with_different_hash_orders_produces_identical_curve_bytes_and_aggregate_statistics";

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
    ancestry: BTreeMap<String, crate::ancestry::CurveAncestry>,
    pay_summary: Vec<u8>,
}

#[derive(Debug)]
struct ProcessSnapshot {
    hash_order_witness: Vec<u8>,
    curve_bytes: Vec<u8>,
    aggregate_bytes: Vec<u8>,
}

fn append_u64(bytes: &mut Vec<u8>, value: usize) {
    bytes.extend_from_slice(&(value as u64).to_le_bytes());
}

fn append_text(bytes: &mut Vec<u8>, value: &str) {
    append_u64(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn packed_pay_summary(rows: &[crate::paysummary::PaySummaryRow]) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_u64(&mut bytes, rows.len());
    for row in rows {
        append_text(&mut bytes, &row.well_id);
        append_text(&mut bytes, &row.well_name);
        append_text(&mut bytes, &row.zone);
        append_text(&mut bytes, &row.flag);
        append_text(&mut bytes, row.frame.as_str());
        append_text(&mut bytes, &row.weights_source);
        let values = [
            row.top,
            row.bottom,
            row.gross,
            row.net,
            row.not_net,
            row.unknown,
            row.ntg_known,
            row.residual_absorbed,
            row.ntg,
            row.avg_vsh,
            row.avg_phie,
            row.avg_swe,
            row.hpv,
        ];
        bytes.extend_from_slice(bytemuck::cast_slice(&values));
        bytes.extend_from_slice(&(row.n_classified as u64).to_le_bytes());
        bytes.push(u8::from(row.perm_cutoff_no_data));
    }
    bytes
}

fn packed_curves(curves: &BTreeMap<(String, String), Vec<u8>>) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_u64(&mut bytes, curves.len());
    for ((well_id, curve), samples) in curves {
        append_text(&mut bytes, well_id);
        append_text(&mut bytes, curve);
        append_u64(&mut bytes, samples.len());
        bytes.extend_from_slice(samples);
    }
    bytes
}

fn hash_order_witness() -> Vec<u8> {
    let mut unordered = HashMap::new();
    for index in 0..64 {
        unordered.insert(format!("HASH_ORDER_{index:02}"), index);
    }
    let mut witness = Vec::new();
    for key in unordered.keys() {
        append_text(&mut witness, key);
    }
    witness
}

fn write_process_snapshot(path: &Path, snapshot: &ProcessSnapshot) {
    let mut bytes = b"SBD16\0".to_vec();
    for part in [
        &snapshot.hash_order_witness,
        &snapshot.curve_bytes,
        &snapshot.aggregate_bytes,
    ] {
        append_u64(&mut bytes, part.len());
        bytes.extend_from_slice(part);
    }
    std::fs::write(path, bytes).expect("write the child process binary snapshot");
}

fn take_snapshot_part(bytes: &[u8], cursor: &mut usize) -> Vec<u8> {
    let end = *cursor + 8;
    let len = u64::from_le_bytes(
        bytes[*cursor..end]
            .try_into()
            .expect("snapshot part length is eight bytes"),
    ) as usize;
    *cursor = end;
    let end = *cursor + len;
    let part = bytes
        .get(*cursor..end)
        .expect("snapshot part stays within the artifact")
        .to_vec();
    *cursor = end;
    part
}

fn read_process_snapshot(path: &Path) -> ProcessSnapshot {
    let bytes = std::fs::read(path).expect("read the child process binary snapshot");
    assert!(
        bytes.starts_with(b"SBD16\0"),
        "the child artifact has the expected binary header"
    );
    let mut cursor = b"SBD16\0".len();
    let snapshot = ProcessSnapshot {
        hash_order_witness: take_snapshot_part(&bytes, &mut cursor),
        curve_bytes: take_snapshot_part(&bytes, &mut cursor),
        aggregate_bytes: take_snapshot_part(&bytes, &mut cursor),
    };
    assert_eq!(
        cursor,
        bytes.len(),
        "the child artifact has no unclassified trailing bytes"
    );
    snapshot
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
            // phi_den, not phi_dn: DEC-070 (2026-08-18) made the D-N quick-look curve
            // visual-only, and this chain's product feeds a pay summary.
            module: "phi_den".into(),
            log_inputs: HashMap::new(),
            params: HashMap::from([
                ("RHO_SH".into(), 2.45),
                ("RHO_DSH".into(), 2.70),
            ]),
            opts: HashMap::new(),
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
    let mut stmt = conn
        .prepare("SELECT well_id FROM wells ORDER BY well_id")
        .unwrap();
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
        samples
            .entry((well_id, curve))
            .or_default()
            .extend([depth, value]);
    }
    samples
        .into_iter()
        .map(|(identity, values)| (identity, bytemuck::cast_slice(&values).to_vec()))
        .collect()
}

fn execute_recorded_chain(project: &Path, rw: f64) -> ReRunSnapshot {
    let conn = db::init_db(project.to_str().unwrap()).expect("open isolated project copy");
    let ids = well_ids(&conn);
    assert_eq!(
        ids.len(),
        2,
        "the full rerun fixture must exercise more than one well"
    );
    let db = Mutex::new(conn);
    let registry = chain::new_registry();
    let job_id = Uuid::new_v4();
    let cancel = chain::register(&registry, job_id);
    let custody = workflow::test_run_custody();
    chain::run_chain(
        &db,
        &crate::reader_pool::ReaderPool::new(),
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
            assert!(
                errors.is_empty(),
                "the recorded chain must complete cleanly: {errors:?}"
            );
        }
        status => panic!("the recorded chain did not complete: {status:?}"),
    }

    let pay = crate::paysummary::run_pay_summary(
        &db,
        &crate::paysummary::PaySummaryRequest {
            discretisation: crate::paysummary::DiscretisationModel::Forward,
            well_ids: ids.clone(),
            vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.55, unit: "v/v".into() }.into()),
            phie_min: Some(crate::paysummary::CutoffEntry { value: 0.10, unit: "v/v".into() }.into()),
            swe_max: Some(crate::paysummary::CutoffEntry { value: 0.65, unit: "v/v".into() }.into()),
            perm_min: None,
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            input_set: None,
            skip_version: false,
            stats_only: true,
            custody: None,
            frame: Default::default(),
            weighting: Default::default(),
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
                crate::ancestry::curve_ancestry(&conn, well_id, "SWE")
                    .expect("the final curve carries the recorded chain ancestry"),
            )
        })
        .collect();
    ReRunSnapshot {
        curve_blobs,
        ancestry,
        pay_summary: packed_pay_summary(&pay),
    }
}

fn run_dbm016_child() {
    let project = PathBuf::from(std::env::var_os(DBM016_PROJECT).expect("child project path"));
    let output = PathBuf::from(std::env::var_os(DBM016_OUTPUT).expect("child output path"));
    let rw = std::env::var(DBM016_RW)
        .expect("child Rw fixture")
        .parse::<f64>()
        .expect("child Rw fixture is numeric");
    let result = execute_recorded_chain(&project, rw);
    assert!(
        !result.curve_blobs.is_empty(),
        "the process proof cannot compare an empty output"
    );
    for required in ["VSH", "PHIE", "SWE"] {
        assert!(
            result
                .curve_blobs
                .keys()
                .any(|(_, curve)| curve == required),
            "the fresh-process chain must write {required}"
        );
    }
    write_process_snapshot(
        &output,
        &ProcessSnapshot {
            hash_order_witness: hash_order_witness(),
            curve_bytes: packed_curves(&result.curve_blobs),
            aggregate_bytes: result.pay_summary,
        },
    );
}

fn launch_dbm016_child(project: &Path, output: &Path, rw: f64) -> ProcessSnapshot {
    let result = Command::new(std::env::current_exe().expect("current Rust test executable"))
        .args(["--exact", DBM016_TEST_NAME, "--nocapture"])
        .env(DBM016_CHILD, "1")
        .env(DBM016_PROJECT, project)
        .env(DBM016_OUTPUT, output)
        .env(DBM016_RW, rw.to_string())
        .output()
        .expect("launch a fresh Rust test process");
    assert!(
        result.status.success(),
        "fresh-process run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    read_process_snapshot(output)
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

    let baseline =
        temporary.track(std::env::temp_dir().join(format!("core011_base_{token}.duckdb")));
    {
        let conn = db::init_db(baseline.to_str().unwrap()).expect("create baseline project");
        let paths = vec![
            low.to_string_lossy().to_string(),
            high.to_string_lossy().to_string(),
        ];
        let imported = ingest::import_las_files(&conn, &paths, None);
        assert_eq!(imported.len(), 2);
        assert!(
            imported
                .iter()
                .all(|result| result.error.is_none() && result.rows == 8),
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

    let first_path =
        temporary.track(std::env::temp_dir().join(format!("core011_first_{token}.duckdb")));
    let second_path =
        temporary.track(std::env::temp_dir().join(format!("core011_second_{token}.duckdb")));
    let changed_path =
        temporary.track(std::env::temp_dir().join(format!("core011_changed_{token}.duckdb")));
    std::fs::copy(&baseline, &first_path).unwrap();
    std::fs::copy(&baseline, &second_path).unwrap();
    std::fs::copy(&baseline, &changed_path).unwrap();

    let first = execute_recorded_chain(&first_path, 0.10);
    let second = execute_recorded_chain(&second_path, 0.10);
    assert!(
        !first.curve_blobs.is_empty(),
        "an empty output cannot prove determinism"
    );
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
    assert_eq!(
        first.ancestry.keys().collect::<Vec<_>>(),
        second.ancestry.keys().collect::<Vec<_>>()
    );
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
    assert_ne!(
        first.curve_blobs, changed.curve_blobs,
        "a changed recorded Rw must move curve bytes"
    );
    assert_ne!(
        first.pay_summary, changed.pay_summary,
        "a changed recorded Rw must move the pay result"
    );
}

#[test]
fn a_project_run_in_fresh_processes_with_different_hash_orders_produces_identical_curve_bytes_and_aggregate_statistics(
) {
    // CORRECTNESS — exact equality is required by 22_database-model.md SB-DBM-T16; the
    // fixture numbers are structural inputs and are not adopted petrophysical defaults.
    if std::env::var_os(DBM016_CHILD).is_some() {
        run_dbm016_child();
        return;
    }

    let token = Uuid::new_v4();
    let mut temporary = TemporaryFiles(Vec::new());
    let low = temporary.track(std::env::temp_dir().join(format!("dbm016_low_{token}.las")));
    let high = temporary.track(std::env::temp_dir().join(format!("dbm016_high_{token}.las")));
    std::fs::write(&low, fixture_las("LOW_SHALE_RESPONSE", 0.0, 0.0)).unwrap();
    std::fs::write(&high, fixture_las("HIGH_SHALE_RESPONSE", 18.0, 0.08)).unwrap();

    let baseline =
        temporary.track(std::env::temp_dir().join(format!("dbm016_base_{token}.duckdb")));
    {
        let conn =
            db::init_db(baseline.to_str().unwrap()).expect("create DBM-016 baseline project");
        let paths = vec![
            low.to_string_lossy().to_string(),
            high.to_string_lossy().to_string(),
        ];
        let imported = ingest::import_las_files(&conn, &paths, None);
        assert_eq!(imported.len(), 2);
        assert!(
            imported
                .iter()
                .all(|result| result.error.is_none() && result.rows == 8),
            "both structural LAS fixtures must import completely: {imported:?}"
        );
        conn.execute_batch("CHECKPOINT").unwrap();
    }

    let first_project =
        temporary.track(std::env::temp_dir().join(format!("dbm016_first_{token}.duckdb")));
    let second_project =
        temporary.track(std::env::temp_dir().join(format!("dbm016_second_{token}.duckdb")));
    let changed_project =
        temporary.track(std::env::temp_dir().join(format!("dbm016_changed_{token}.duckdb")));
    let first_output =
        temporary.track(std::env::temp_dir().join(format!("dbm016_first_{token}.bin")));
    let second_output =
        temporary.track(std::env::temp_dir().join(format!("dbm016_second_{token}.bin")));
    let changed_output =
        temporary.track(std::env::temp_dir().join(format!("dbm016_changed_{token}.bin")));
    for destination in [&first_project, &second_project, &changed_project] {
        std::fs::copy(&baseline, destination).unwrap();
    }

    let first = launch_dbm016_child(&first_project, &first_output, 0.10);
    let second = launch_dbm016_child(&second_project, &second_output, 0.10);
    assert_ne!(
        first.hash_order_witness, second.hash_order_witness,
        "the two fresh processes must demonstrate different randomized HashMap iteration orders"
    );
    assert_eq!(
        first.curve_bytes, second.curve_bytes,
        "SB-DBM-T16 requires every packed output curve to be byte-identical"
    );
    assert_eq!(
        first.aggregate_bytes, second.aggregate_bytes,
        "SB-DBM-T16 requires every aggregate-statistic field to be byte-identical"
    );

    // Pin the other side: the comparison must observe scientific output rather than a constant,
    // empty or metadata-only artifact.
    let changed = launch_dbm016_child(&changed_project, &changed_output, 0.50);
    assert_ne!(
        first.curve_bytes, changed.curve_bytes,
        "a changed recorded Rw must move curve bytes"
    );
    assert_ne!(
        first.aggregate_bytes, changed.aggregate_bytes,
        "a changed recorded Rw must move aggregate statistics"
    );
}
