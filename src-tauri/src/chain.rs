//! Workflow chains (Phase 9): run an ordered sequence of deterministic modules across many
//! wells in one shot, with pollable progress and cooperative cancellation.
//!
//! Steps run *sequentially* (a later step, e.g. sw_indo, consumes an earlier step's outputs
//! like PHIE/PHIT), while the wells inside each step stay rayon-parallel via
//! [`workflow::run_workflow_module`]. Interval/zone parameters still apply: a chain run with
//! empty step params uses each module's manifest defaults, which `zone_params` then override
//! per zone — so a saved chain honours the zone parameters set in the Zones panel.
//!
//! Progress follows the same registry-and-poll model as `inversion.rs` (no Tauri events):
//! the frontend generates the job id, calls `run_workflow_chain`, and polls `get_chain_status`
//! on a timer while the run occupies its own command worker thread. `cancel_workflow_chain`
//! flips a shared flag the runner checks between steps.

use crate::workflow::{self, RunModuleRequest};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// One module invocation in a chain. Empty maps fall back to the module manifest defaults
/// (which `zone_params` then override per zone), so the common case serializes compactly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainStep {
    pub module: String,
    #[serde(default)]
    pub log_inputs: HashMap<String, String>,
    #[serde(default)]
    pub params: HashMap<String, f64>,
    #[serde(default)]
    pub opts: HashMap<String, String>,
}

/// Pollable status of a chain job. `wells_done` reaches `wells_total` as each step finishes
/// (step-level granularity — a step across even 100 wells is seconds).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ChainStatus {
    Queued,
    Running {
        step: usize,
        total_steps: usize,
        module: String,
        wells_done: usize,
        wells_total: usize,
    },
    Completed {
        steps_run: usize,
        curves_written: usize,
        wells: usize,
        errors: Vec<String>,
    },
    Cancelled {
        at_step: usize,
    },
    Failed {
        error: String,
    },
}

pub(crate) struct ChainJob {
    status: ChainStatus,
    cancel: Arc<AtomicBool>,
}

pub(crate) type ChainRegistry = Arc<Mutex<HashMap<Uuid, ChainJob>>>;

pub(crate) fn new_registry() -> ChainRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn set_status(registry: &ChainRegistry, job_id: Uuid, status: ChainStatus) {
    if let Some(job) = registry.lock().unwrap().get_mut(&job_id) {
        job.status = status;
    }
}

/// Registers a job id up front (so the poller finds it immediately) and returns its shared
/// cancel flag. Called before `run_chain` starts touching the database.
pub(crate) fn register(registry: &ChainRegistry, job_id: Uuid) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    registry
        .lock()
        .unwrap()
        .insert(job_id, ChainJob { status: ChainStatus::Queued, cancel: cancel.clone() });
    cancel
}

/// Reads back a job's current status (for the `get_chain_status` command).
pub(crate) fn status(registry: &ChainRegistry, job_id: Uuid) -> Option<ChainStatus> {
    registry.lock().unwrap().get(&job_id).map(|j| j.status.clone())
}

/// True while any chain job is queued or running — switching projects mid-run would
/// make its later steps write into the newly opened database.
pub(crate) fn any_active(registry: &ChainRegistry) -> bool {
    registry
        .lock()
        .unwrap()
        .values()
        .any(|j| matches!(j.status, ChainStatus::Queued | ChainStatus::Running { .. }))
}

/// Requests cancellation of a running job; the runner stops before the next step.
pub(crate) fn cancel(registry: &ChainRegistry, job_id: Uuid) {
    if let Some(job) = registry.lock().unwrap().get(&job_id) {
        job.cancel.store(true, Ordering::SeqCst);
    }
}

/// Runs the chain to completion, updating the registry between steps. Intended to be called
/// on a command worker thread while the UI polls `status`. The job must already be
/// [`register`]ed; `cancel` is that job's shared flag.
pub(crate) fn run_chain(
    db: &Mutex<Connection>,
    registry: &ChainRegistry,
    job_id: Uuid,
    cancel: &AtomicBool,
    steps: &[ChainStep],
    well_ids: &[String],
    output_set: Option<&str>,
    input_set: Option<&str>,
) {
    let total_steps = steps.len();
    let wells_total = well_ids.len();
    let mut curves_written = 0usize;
    let mut errors: Vec<String> = Vec::new();

    // ONE set event per well for the whole chain run: every step writes into the same
    // version, so re-running the chain bumps the set to N+1 (never overwrites history).
    let set_name = output_set.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("INTERP");
    let modules_list: Vec<&str> = steps.iter().map(|s| s.module.as_str()).collect();
    let preset_sets: HashMap<String, String> = {
        let conn = db.lock().unwrap();
        let spec = crate::equations::LogSetSpec {
            set_name: set_name.to_string(),
            module: format!("workflow: {}", modules_list.join(" → ")),
            params_json: serde_json::to_string(&modules_list).unwrap_or_default(),
            inputs_json: input_set
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| format!("[[\"input_set\",{}]]", serde_json::to_string(s).unwrap_or_default()))
                .unwrap_or_default(),
        };
        well_ids
            .iter()
            .filter_map(|w| crate::equations::create_log_set(&conn, w, &spec).ok().map(|(id, _)| (w.clone(), id)))
            .collect()
    };

    for (i, step) in steps.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            set_status(registry, job_id, ChainStatus::Cancelled { at_step: i });
            return;
        }
        set_status(
            registry,
            job_id,
            ChainStatus::Running {
                step: i,
                total_steps,
                module: step.module.clone(),
                wells_done: 0,
                wells_total,
            },
        );

        let req = RunModuleRequest {
            module: step.module.clone(),
            well_ids: well_ids.to_vec(),
            log_inputs: step.log_inputs.clone(),
            params: step.params.clone(),
            opts: step.opts.clone(),
            output_set: None, // preset_sets carries the chain-level set event
            // Curves a later step consumes from an earlier one are never in the input
            // set's archive, so they still resolve from the current store — chaining works.
            input_set: input_set.map(str::to_string),
        };
        let results = workflow::run_workflow_module_into(db, &req, Some(&preset_sets));
        for r in &results {
            curves_written += r.output_curves.len();
            if let Some(e) = &r.error {
                errors.push(format!("{} @ {}: {e}", step.module, r.well_id));
            }
        }

        set_status(
            registry,
            job_id,
            ChainStatus::Running {
                step: i,
                total_steps,
                module: step.module.clone(),
                wells_done: wells_total,
                wells_total,
            },
        );
    }

    set_status(
        registry,
        job_id,
        ChainStatus::Completed { steps_run: total_steps, curves_written, wells: wells_total, errors },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use duckdb::params;
    use uuid::Uuid as U;

    /// Minimal synthetic well: a clean-ish sand with GR/RHOB/NPHI so vsh_gr → phi_dn → sw_indo
    /// all produce finite outputs.
    fn seed_well(conn: &Connection) -> String {
        let id = U::new_v4();
        db::insert_well(conn, id, "CHAIN_TEST", Some("Synthetic"), None, None).unwrap();
        let n = 200usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 30.0 + (i % 40) as f32).collect();
        let res: Vec<f32> = vec![10.0; n];
        let nphi: Vec<f32> = vec![0.22; n];
        let rhob: Vec<f32> = vec![2.35; n];
        let dt: Vec<f32> = vec![90.0; n];
        let sp: Vec<f32> = vec![f32::NAN; n];
        db::insert_standard_curves(conn, id, depth, gr, res, nphi, rhob, dt, sp).unwrap();
        id.to_string()
    }

    fn finite(conn: &Connection, well: &str, curve: &str) -> i64 {
        conn.query_row(
            "SELECT count(*) FROM computed_curves WHERE well_id=?1 AND curve_name=?2 AND NOT isnan(value)",
            params![well, curve],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    fn step(module: &str) -> ChainStep {
        ChainStep { module: module.into(), log_inputs: HashMap::new(), params: HashMap::new(), opts: HashMap::new() }
    }

    #[test]
    fn chain_runs_steps_in_order_and_completes() {
        let path = std::env::temp_dir().join("arshilla_chain_test.duckdb");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let well = seed_well(&conn);

        let reg = new_registry();
        let job = U::new_v4();
        let cancel = register(&reg, job);
        let db = Mutex::new(conn);
        let steps = vec![step("vsh_gr"), step("phi_dn"), step("sw_indo")];

        run_chain(&db, &reg, job, &cancel, &steps, &[well.clone()], None, None);

        match status(&reg, job).unwrap() {
            ChainStatus::Completed { steps_run, curves_written, wells, errors } => {
                assert_eq!(steps_run, 3);
                assert_eq!(wells, 1);
                assert!(curves_written > 0);
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
            other => panic!("expected Completed, got {other:?}"),
        }

        let conn = db.lock().unwrap();
        // vsh_gr → VSH, phi_dn → PHIE, sw_indo → SWE all present and finite.
        assert!(finite(&conn, &well, "VSH") > 0);
        assert!(finite(&conn, &well, "PHIE") > 0);
        assert!(finite(&conn, &well, "SWE") > 0);
    }

    #[test]
    fn chain_honours_precancellation() {
        let path = std::env::temp_dir().join("arshilla_chain_cancel.duckdb");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));
        let conn = db::init_db(path.to_str().unwrap()).unwrap();
        let well = seed_well(&conn);

        let reg = new_registry();
        let job = U::new_v4();
        let cancel = register(&reg, job);
        cancel.store(true, Ordering::SeqCst); // cancel before it starts
        let db = Mutex::new(conn);

        run_chain(&db, &reg, job, &cancel, &[step("vsh_gr")], &[well.clone()], None, None);

        match status(&reg, job).unwrap() {
            ChainStatus::Cancelled { at_step } => assert_eq!(at_step, 0),
            other => panic!("expected Cancelled, got {other:?}"),
        }
        let conn = db.lock().unwrap();
        assert_eq!(finite(&conn, &well, "VSH"), 0, "no work should have run");
    }
}
