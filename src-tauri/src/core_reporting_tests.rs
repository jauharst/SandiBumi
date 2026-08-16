use crate::chain::ChainStep;
use crate::jobs::{self, ItemState};
use crate::montecarlo::{self, McRequest, Sampling};
use duckdb::Connection;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn seed_gas_correction_inputs(conn: &Connection) -> String {
    let well_id = Uuid::new_v4();
    crate::db::insert_well(conn, well_id, "CHAIN_FAILURE_SURFACE", Some("Synthetic"), None, None)
        .unwrap();
    let depth = vec![1000.0_f32, 1000.5, 1001.0];
    crate::db::insert_standard_curves(
        conn,
        well_id,
        depth.clone(),
        vec![f32::NAN; 3],
        vec![20.0; 3],
        vec![0.18; 3],
        vec![2.30; 3],
        vec![f32::NAN; 3],
        vec![f32::NAN; 3],
    )
    .unwrap();
    crate::equations::write_computed_curve(conn, &well_id.to_string(), &depth, "FTEMP", &[80.0; 3])
        .unwrap();
    crate::equations::write_computed_curve(conn, &well_id.to_string(), &depth, "FPRESS", &[3000.0; 3])
        .unwrap();
    well_id.to_string()
}

fn gas_correction_request(well_id: &str, gate: &str) -> McRequest {
    McRequest {
        well_ids: vec![well_id.to_string()],
        steps: vec![ChainStep {
            module: "gascorr".to_string(),
            log_inputs: HashMap::new(),
            // CHARACTERIZATION fixture: former manifest inputs are explicit so this test reaches
            // its reporting-surface subject (the missing FLAGGED gate). They are not defaults.
            params: HashMap::from([
                ("RHO_MA".to_string(), 2.65),
                ("RHO_FL".to_string(), 1.0),
                ("SG_GAS".to_string(), 0.65),
                ("A".to_string(), 1.0),
                ("M".to_string(), 2.0),
                ("N".to_string(), 2.0),
                ("RW".to_string(), 0.1),
            ]),
            opts: [("OPT_GATE".to_string(), gate.to_string())]
                .into_iter()
                .collect(),
        }],
        mc_params: Vec::new(),
        iterations: 2,
        seed: 7,
        vsh_max: Some(crate::workflow::CutoffEntry { value: 0.5, unit: "v/v".into() }),
        phie_min: Some(crate::workflow::CutoffEntry { value: 0.08, unit: "v/v".into() }),
        swe_max: Some(crate::workflow::CutoffEntry { value: 0.6, unit: "v/v".into() }),
        perm_min: None,
        bins: 4,
        low_pctl: 0.10,
        high_pctl: 0.90,
        sensitivity: false,
        tornado: false,
        sampling: Sampling::Random,
        correlations: Vec::new(),
        converge: false,
        converge_tol: 0.005,
        persist: false,
        persist_realizations: false,
        realization_cap: None
    ,
        custody: Some(crate::workflow::test_run_custody()),
    }
}

fn registered_job(well_id: &str) -> (jobs::JobRegistry, jobs::JobHandle) {
    let registry = jobs::new_registry();
    let handle = jobs::register(
        &registry,
        Uuid::new_v4(),
        "Monte Carlo",
        "Chain reporting",
        vec![(well_id.to_string(), "CHAIN_FAILURE_SURFACE".to_string())],
        Arc::new(AtomicBool::new(false)),
        true,
    );
    handle.running(1);
    (registry, handle)
}

/// SB-CORE-002 / SB-CORE-T03. CORRECTNESS: the expected reporting surfaces are the
/// seven degraded-result contracts recovered in `04_CORE_REQUIREMENTS.md` from R4.
/// The fixture deliberately omits GAS_FLAG, so gascorr's documented FLAGGED refusal is
/// independent evidence for the failure; the EVERYWHERE half prevents an always-fail test.
#[test]
fn a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result() {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::create_schema(&conn).unwrap();
    let well_id = seed_gas_correction_inputs(&conn);
    let db = Mutex::new(conn);

    let (failed_registry, failed_job) = registered_job(&well_id);
    let failed = montecarlo::run_monte_carlo(
        &db,
        &gas_correction_request(&well_id, "FLAGGED"),
        Some(&failed_job),
    );
    assert_eq!(failed.errors.len(), 1, "one failed well must yield one named result error");
    let result_error = &failed.errors[0];
    assert!(result_error.contains(&well_id), "the result error must name the affected well");
    assert!(
        result_error.contains("chain step failed on every realization"),
        "the result must not look like a valid zero-uncertainty study: {result_error}"
    );
    assert!(
        result_error.contains("OPT_GATE = FLAGGED") && result_error.contains("run condflag first"),
        "the underlying actionable refusal must survive: {result_error}"
    );
    let failed_view = jobs::list(&failed_registry).remove(0);
    assert_eq!(failed_view.items[0].state, ItemState::Failed);
    let job_message = failed_view.items[0].message.as_deref().expect("failed job item message");
    assert!(job_message.contains("chain step failed on every realization"));
    assert!(job_message.contains("OPT_GATE = FLAGGED") && job_message.contains("run condflag first"));

    let (ok_registry, ok_job) = registered_job(&well_id);
    let succeeded = montecarlo::run_monte_carlo(
        &db,
        &gas_correction_request(&well_id, "EVERYWHERE"),
        Some(&ok_job),
    );
    assert!(succeeded.errors.is_empty(), "the same inputs with the explicit ungated policy must run");
    let ok_view = jobs::list(&ok_registry).remove(0);
    assert_eq!(ok_view.items[0].state, ItemState::Ok, "a valid control run must not be failed");
    assert!(ok_view.items[0].message.is_none());
}
