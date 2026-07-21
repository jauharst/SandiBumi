//! Universal job registry (Phase 11): one progress/cancel channel every long operation
//! reports into, so a single dock panel can show live per-well progress and a Cancel button
//! for ANY of them — workflow chains today, and module runs / imports / multimin / Monte
//! Carlo / reports as each is moved off the IPC thread. Generalises the chain-specific
//! registry in `chain.rs`.
//!
//! Same registry-and-poll model as `chain.rs` / `inversion.rs` (no Tauri events): a job is
//! registered up front, runs on its own `std::thread`, mutates its record under a brief
//! registry lock, and the frontend polls `list_jobs` on a timer. `cancel_job` flips the
//! job's shared `AtomicBool`; workers check it cooperatively (per well) and drain fast.
//!
//! The `cancel` flag is passed IN (not created here) so one flag can drive both this
//! registry and the legacy chain registry at once — a Cancel in either panel stops the
//! same run. A `JobHandle` is cheap to clone and `Send + Sync`, so the rayon parallel
//! wells inside `workflow::run_workflow_module_into` can each report progress concurrently;
//! every mutation just takes a brief lock on the shared registry (no nested per-job mutex).

use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Per-item (usually per-well) state as a job runs. `Warned` = finished but with a caveat
/// (e.g. produced no output); `Failed` = errored for that item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ItemState {
    Pending,
    Running,
    Ok,
    Warned,
    Failed,
}

impl ItemState {
    /// Higher = more severe. Used so a well that failed (or was warned on) any step keeps
    /// that status even if a later step of the same chain finishes it cleanly.
    fn severity(self) -> u8 {
        match self {
            ItemState::Pending => 0,
            ItemState::Running => 1,
            ItemState::Ok => 2,
            ItemState::Warned => 3,
            ItemState::Failed => 4,
        }
    }
}

/// Overall job lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JobPhase {
    Queued,
    Running,
    Completed,
    Cancelled,
    Failed,
}

/// One item's live state (serialized to the panel).
#[derive(Debug, Clone, Serialize)]
pub(crate) struct JobItem {
    /// Stable id the runner advances by (well_id).
    pub(crate) key: String,
    /// Human label shown in the panel (well name).
    pub(crate) label: String,
    pub(crate) state: ItemState,
    pub(crate) message: Option<String>,
}

/// Live, mutable job record (kept inside the registry mutex).
struct Job {
    kind: String,
    label: String,
    phase: JobPhase,
    total: usize,
    done: usize,
    current: Option<String>,
    items: Vec<JobItem>,
    item_index: HashMap<String, usize>,
    error: Option<String>,
    seq: u64,
    cancel: Arc<AtomicBool>,
}

/// Serializable snapshot for the `list_jobs` command.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct JobView {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) phase: JobPhase,
    pub(crate) total: usize,
    pub(crate) done: usize,
    pub(crate) current: Option<String>,
    pub(crate) items: Vec<JobItem>,
    pub(crate) error: Option<String>,
    pub(crate) seq: u64,
}

pub(crate) struct JobStore {
    jobs: HashMap<Uuid, Job>,
    next_seq: u64,
}

pub(crate) type JobRegistry = Arc<Mutex<JobStore>>;

/// Keep at most this many finished jobs so a long session doesn't accumulate every run
/// forever. Active jobs are never pruned.
const MAX_FINISHED: usize = 24;

pub(crate) fn new_registry() -> JobRegistry {
    Arc::new(Mutex::new(JobStore { jobs: HashMap::new(), next_seq: 0 }))
}

/// A worker's handle to one job — cheap to clone, `Send + Sync`, safe to share across the
/// rayon parallel wells. Every mutation takes a brief registry lock (same pattern as
/// `chain::set_status`), so there is no nested per-job mutex.
#[derive(Clone)]
pub(crate) struct JobHandle {
    reg: JobRegistry,
    id: Uuid,
    pub(crate) cancel: Arc<AtomicBool>,
}

impl JobHandle {
    fn with_job(&self, f: impl FnOnce(&mut Job)) {
        if let Some(job) = self.reg.lock().unwrap().jobs.get_mut(&self.id) {
            f(job);
        }
    }

    /// True once this job has been cancelled. Off-thread workers wrapped by [`run_job`] poll
    /// this per item to drain fast; chains instead read the shared `cancel` flag directly.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Move the job into the Running phase and set the total unit count (steps × wells for a
    /// chain, wells for a single module).
    pub(crate) fn running(&self, total: usize) {
        self.with_job(|j| {
            j.phase = JobPhase::Running;
            j.total = total;
        });
    }

    /// Human label for what is happening right now, e.g. "Step 2/3: sw_indo".
    pub(crate) fn set_current(&self, current: Option<String>) {
        self.with_job(|j| j.current = current);
    }

    /// Mark an item as actively processing (no progress increment). Never un-marks a
    /// terminal Failed/Warned so a sticky problem stays visible.
    pub(crate) fn start_item(&self, key: &str) {
        self.with_job(|j| {
            if let Some(&i) = j.item_index.get(key) {
                if j.items[i].state.severity() < ItemState::Warned.severity() {
                    j.items[i].state = ItemState::Running;
                }
            }
        });
    }

    /// Mark an item finished and count ONE unit of progress. State only escalates in
    /// severity, so across a chain's steps a well that failed once stays Failed.
    pub(crate) fn finish_item(&self, key: &str, state: ItemState, message: Option<String>) {
        self.with_job(|j| {
            j.done += 1;
            escalate(j, key, state, message);
        });
    }

    /// Update an item's state WITHOUT counting progress (e.g. a late write failure downgrades
    /// a well that already finished its compute).
    pub(crate) fn mark_item(&self, key: &str, state: ItemState, message: Option<String>) {
        self.with_job(|j| escalate(j, key, state, message));
    }

    pub(crate) fn complete(&self) {
        self.finish(JobPhase::Completed, None);
    }

    pub(crate) fn cancelled(&self) {
        self.finish(JobPhase::Cancelled, None);
    }

    /// Wholesale failure (vs per-item): the whole op errored. Used by [`run_job`] when a worker
    /// thread panics, and available to a worker that hits an unrecoverable error.
    pub(crate) fn failed(&self, error: String) {
        self.finish(JobPhase::Failed, Some(error));
    }

    /// Terminal transition: set the final phase, clear `current`, and prune finished jobs so
    /// the cap holds tightly the moment a job ends (not only when the next one registers).
    /// The FIRST terminal transition wins — so [`run_job`]'s automatic `complete` can never
    /// overwrite a wholesale `failed` the worker already recorded, and a double-finalize is a
    /// harmless no-op.
    fn finish(&self, phase: JobPhase, error: Option<String>) {
        let mut store = self.reg.lock().unwrap();
        if let Some(job) = store.jobs.get_mut(&self.id) {
            if matches!(job.phase, JobPhase::Completed | JobPhase::Cancelled | JobPhase::Failed) {
                return;
            }
            job.phase = phase;
            job.current = None;
            if error.is_some() {
                job.error = error;
            }
        }
        prune(&mut store);
    }
}

/// Runs `work` on Tauri's blocking thread-pool as a registered job and returns its value. This
/// is the one entry point every off-thread command uses: the `#[tauri::command]` shim stays
/// `async` (so it never blocks the IPC/event-loop thread), and `work` gets a [`JobHandle`] to
/// report per-item progress and to poll [`JobHandle::is_cancelled`]. The job is registered with
/// `items` (all Pending), moved to Running with `total` units, then finalized automatically —
/// Cancelled if its flag was flipped, otherwise Completed — unless `work` already finished it
/// (a wholesale [`JobHandle::failed`]), in which case that terminal state stands.
///
/// Returns the worker's own value on success; the only `Err` is a worker-thread panic (the
/// ordinary per-item / per-op errors ride inside the returned payload, exactly as before, so a
/// command that keeps its original payload return type sees identical behaviour).
pub(crate) async fn run_job<T, F>(
    reg: JobRegistry,
    kind: impl Into<String>,
    label: impl Into<String>,
    items: Vec<(String, String)>,
    total: usize,
    work: F,
) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(JobHandle) -> T + Send + 'static,
{
    let id = Uuid::new_v4();
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = register(&reg, id, kind, label, items, cancel);
    handle.running(total);
    let finalize = handle.clone();
    match tauri::async_runtime::spawn_blocking(move || work(handle)).await {
        Ok(out) => {
            if finalize.is_cancelled() {
                finalize.cancelled();
            } else {
                finalize.complete();
            }
            Ok(out)
        }
        Err(e) => {
            let msg = format!("worker thread failed: {e}");
            finalize.failed(msg.clone());
            Err(msg)
        }
    }
}

/// Coarse single-unit off-thread job for a MONOLITHIC op that has no natural per-well/per-file
/// progress (a single render, export, or subprocess). Shows one "running" card + a Cancel button
/// in the Processing panel, marks its one synthetic item Ok or Failed by the op's own `Result`,
/// and returns that result. Use [`run_job`] instead whenever per-item progress is available.
pub(crate) async fn run_simple_job<T, F>(
    reg: JobRegistry,
    kind: impl Into<String>,
    label: impl Into<String>,
    work: F,
) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let label = label.into();
    run_job(reg, kind, label.clone(), vec![(String::from("op"), label)], 1, move |job| {
        let out = work();
        match &out {
            Ok(_) => job.finish_item("op", ItemState::Ok, None),
            Err(e) => job.finish_item("op", ItemState::Failed, Some(e.clone())),
        }
        out
    })
    .await
    .and_then(|r| r)
}

/// Escalate an item's state, keeping the more severe of the current and new states and
/// recording the first non-empty message for that severity.
fn escalate(job: &mut Job, key: &str, state: ItemState, message: Option<String>) {
    if let Some(&i) = job.item_index.get(key) {
        let cur = job.items[i].state;
        if state.severity() >= cur.severity() {
            job.items[i].state = state;
            if message.is_some() {
                job.items[i].message = message;
            }
        }
    }
}

/// Registers a job up front (Queued, all items Pending) and returns a worker handle. The
/// `cancel` flag is shared with the caller so one flag drives both this registry and the
/// legacy chain registry — a Cancel in either panel stops the same run.
pub(crate) fn register(
    reg: &JobRegistry,
    id: Uuid,
    kind: impl Into<String>,
    label: impl Into<String>,
    items: Vec<(String, String)>,
    cancel: Arc<AtomicBool>,
) -> JobHandle {
    let job_items: Vec<JobItem> = items
        .iter()
        .map(|(key, label)| JobItem {
            key: key.clone(),
            label: label.clone(),
            state: ItemState::Pending,
            message: None,
        })
        .collect();
    let item_index: HashMap<String, usize> =
        items.iter().enumerate().map(|(i, (key, _))| (key.clone(), i)).collect();
    let mut store = reg.lock().unwrap();
    let seq = store.next_seq;
    store.next_seq += 1;
    store.jobs.insert(
        id,
        Job {
            kind: kind.into(),
            label: label.into(),
            phase: JobPhase::Queued,
            total: job_items.len(),
            done: 0,
            current: None,
            items: job_items,
            item_index,
            error: None,
            seq,
            cancel: cancel.clone(),
        },
    );
    prune(&mut store);
    JobHandle { reg: reg.clone(), id, cancel }
}

/// Drop the oldest finished jobs when there are too many; active jobs are never pruned.
fn prune(store: &mut JobStore) {
    let mut finished: Vec<(Uuid, u64)> = store
        .jobs
        .iter()
        .filter(|(_, j)| !matches!(j.phase, JobPhase::Queued | JobPhase::Running))
        .map(|(id, j)| (*id, j.seq))
        .collect();
    if finished.len() <= MAX_FINISHED {
        return;
    }
    finished.sort_by_key(|(_, seq)| *seq);
    let drop_n = finished.len() - MAX_FINISHED;
    for (id, _) in finished.into_iter().take(drop_n) {
        store.jobs.remove(&id);
    }
}

/// Snapshot of every job, most recent first — for the `list_jobs` poll.
pub(crate) fn list(reg: &JobRegistry) -> Vec<JobView> {
    let store = reg.lock().unwrap();
    let mut views: Vec<JobView> = store
        .jobs
        .iter()
        .map(|(id, j)| JobView {
            id: id.to_string(),
            kind: j.kind.clone(),
            label: j.label.clone(),
            phase: j.phase,
            total: j.total,
            done: j.done,
            current: j.current.clone(),
            items: j.items.clone(),
            error: j.error.clone(),
            seq: j.seq,
        })
        .collect();
    views.sort_by(|a, b| b.seq.cmp(&a.seq));
    views
}

/// Requests cancellation of one job (flips its shared flag).
pub(crate) fn cancel(reg: &JobRegistry, id: Uuid) {
    if let Some(job) = reg.lock().unwrap().jobs.get(&id) {
        job.cancel.store(true, Ordering::SeqCst);
    }
}

/// True while any job is queued or running — guards project switches mid-run (a background
/// job holds an Arc DB handle that would otherwise follow the switch).
pub(crate) fn any_active(reg: &JobRegistry) -> bool {
    reg.lock().unwrap().jobs.values().any(|j| matches!(j.phase, JobPhase::Queued | JobPhase::Running))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_lifecycle_reports_progress_and_severity_sticks() {
        let reg = new_registry();
        let id = Uuid::new_v4();
        let flag = Arc::new(AtomicBool::new(false));
        let items = vec![
            ("w1".to_string(), "WELL_1".to_string()),
            ("w2".to_string(), "WELL_2".to_string()),
        ];
        let h = register(&reg, id, "Workflow chain", "vsh_gr → phi_dn", items, flag.clone());

        // Registered = queued, both items pending.
        let v = list(&reg).remove(0);
        assert_eq!(v.phase, JobPhase::Queued);
        assert_eq!(v.total, 2);
        assert!(v.items.iter().all(|it| it.state == ItemState::Pending));

        // A 2-step chain over 2 wells = 4 units.
        h.running(4);
        // Step 1: w1 fails, w2 ok.
        h.start_item("w1");
        h.finish_item("w1", ItemState::Failed, Some("no GR".into()));
        h.start_item("w2");
        h.finish_item("w2", ItemState::Ok, None);
        // Step 2: both compute cleanly — but w1's failure must STICK.
        h.start_item("w1");
        h.finish_item("w1", ItemState::Ok, None);
        h.start_item("w2");
        h.finish_item("w2", ItemState::Ok, None);
        h.complete();

        let v = list(&reg).remove(0);
        assert_eq!(v.phase, JobPhase::Completed);
        assert_eq!(v.done, 4);
        assert_eq!(v.items[0].state, ItemState::Failed, "step-1 failure is sticky");
        assert_eq!(v.items[0].message.as_deref(), Some("no GR"));
        assert_eq!(v.items[1].state, ItemState::Ok);
    }

    #[test]
    fn cancel_via_registry_flips_shared_flag() {
        let reg = new_registry();
        let id = Uuid::new_v4();
        let flag = Arc::new(AtomicBool::new(false));
        let h = register(&reg, id, "Module", "sw_indo", vec![], flag.clone());
        assert!(any_active(&reg));
        assert!(!h.is_cancelled());
        cancel(&reg, id);
        assert!(h.is_cancelled());
        assert!(flag.load(Ordering::SeqCst), "the caller's flag is the job's flag");
    }

    #[test]
    fn finished_jobs_are_pruned_but_active_ones_survive() {
        let reg = new_registry();
        let flag = Arc::new(AtomicBool::new(false));
        // One active job that must never be pruned.
        let active = Uuid::new_v4();
        let ah = register(&reg, active, "Chain", "keep-me", vec![], flag.clone());
        ah.running(1);
        // Flood with finished jobs beyond the cap.
        for _ in 0..(MAX_FINISHED + 10) {
            let id = Uuid::new_v4();
            let h = register(&reg, id, "Module", "x", vec![], Arc::new(AtomicBool::new(false)));
            h.complete();
        }
        let views = list(&reg);
        let finished = views.iter().filter(|v| v.phase == JobPhase::Completed).count();
        assert!(finished <= MAX_FINISHED, "finished jobs capped at {MAX_FINISHED}");
        assert!(views.iter().any(|v| v.id == active.to_string()), "active job survives pruning");
    }
}
