use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Status of a background multi-mineral (LRLC) stochastic inversion job.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state")]
pub enum InversionStatus {
    Queued,
    Running { progress: f32 },
    Completed { result: Vec<f32> },
    Failed { error: String },
}

pub type JobRegistry = Arc<Mutex<HashMap<Uuid, InversionStatus>>>;

pub fn new_registry() -> JobRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn set_status(registry: &JobRegistry, job_id: Uuid, status: InversionStatus) {
    registry.lock().unwrap().insert(job_id, status);
}

/// Placeholder for a real iterative multi-mineral inversion (e.g. simulated annealing over
/// mineral/fluid volume fractions). Runs entirely synchronously on its own blocking-pool
/// thread and reports incremental progress into the registry as it iterates.
fn run_stochastic_inversion(registry: &JobRegistry, job_id: Uuid, iterations: u32) -> Vec<f32> {
    let model = [0.25f32, 0.15, 0.20, 0.40]; // [quartz, clay, porosity, water] fractions
    let report_every = (iterations / 20).max(1);
    for i in 0..iterations {
        // Real solver step (annealing/MCMC update) goes here.
        std::thread::sleep(std::time::Duration::from_millis(1));
        if i % report_every == 0 {
            set_status(
                registry,
                job_id,
                InversionStatus::Running { progress: i as f32 / iterations as f32 },
            );
        }
    }
    model.to_vec()
}

/// Dispatches a long-running inversion to a `tokio::task::spawn_blocking` thread and returns
/// immediately with a job id, so the Tauri UI thread never blocks while the solver runs.
/// Poll `get_inversion_status` (Tauri command) with the returned id for progress/result.
pub fn dispatch_inversion(registry: JobRegistry, iterations: u32) -> Uuid {
    let job_id = Uuid::new_v4();
    set_status(&registry, job_id, InversionStatus::Queued);

    tokio::spawn(async move {
        let reg = registry.clone();
        let handle = tokio::task::spawn_blocking(move || run_stochastic_inversion(&reg, job_id, iterations));

        match handle.await {
            Ok(result) => set_status(&registry, job_id, InversionStatus::Completed { result }),
            Err(e) => set_status(&registry, job_id, InversionStatus::Failed { error: e.to_string() }),
        }
    });

    job_id
}
