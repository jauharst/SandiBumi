//! Universal job registry (Phase 11): one progress/cancel channel every long operation
//! reports into, so a single dock panel can show live per-well progress and a Cancel button
//! for ANY of them — workflow chains today, and module runs / imports / SandiMin / Monte
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

/// Aggregate result, separate from lifecycle. A job may finish its scheduled work (`Completed`)
/// while one or more wells are degraded or failed; collapsing those facts into the phase is what
/// made a mixed batch read as a clean "Done" card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JobOutcome {
    Clean,
    Degraded,
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
    /// Whether this job's worker actually polls the cancel flag. A monolithic op run through
    /// [`run_simple_job`] gets no handle and so cannot poll — offering it a Cancel button was a
    /// control that did nothing, the visible half of the cancel-honesty defect. The panel reads
    /// this to decide whether to show the button at all.
    cancellable: bool,
}

/// Serializable snapshot for the `list_jobs` command.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct JobView {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) label: String,
    pub(crate) phase: JobPhase,
    pub(crate) outcome: Option<JobOutcome>,
    pub(crate) total: usize,
    pub(crate) done: usize,
    pub(crate) current: Option<String>,
    pub(crate) items: Vec<JobItem>,
    pub(crate) error: Option<String>,
    pub(crate) seq: u64,
    /// True only when the worker actually observes the cancel flag; the panel shows a Cancel
    /// button on cancellable jobs and an honest "can't be interrupted" tag on the rest.
    pub(crate) cancellable: bool,
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
    /// Set the first time a worker actually OBSERVES the cancel flag as set. Most job kinds
    /// never poll, so the flag alone cannot tell us whether a run really stopped — see
    /// [`JobHandle::cancel_was_observed`] and the finalization in [`run_job`].
    observed_cancel: Arc<AtomicBool>,
}

impl JobHandle {
    fn with_job(&self, f: impl FnOnce(&mut Job)) {
        if let Some(job) = self.reg.lock().unwrap().jobs.get_mut(&self.id) {
            f(job);
        }
    }

    /// True once this job has been cancelled. Off-thread workers wrapped by [`run_job`] poll
    /// this per item to drain fast; chains instead read the shared `cancel` flag directly.
    ///
    /// Observing a set flag is RECORDED, because that observation is the only evidence anyone
    /// acted on the cancel. A worker that never calls this cannot have drained early, so its run
    /// must not be reported as cancelled no matter what the user clicked.
    pub(crate) fn is_cancelled(&self) -> bool {
        let c = self.cancel.load(Ordering::SeqCst);
        if c {
            self.observed_cancel.store(true, Ordering::SeqCst);
        }
        c
    }

    /// Did any worker actually see the cancel? Chains read the raw `cancel` flag rather than
    /// going through [`is_cancelled`], so they mark this explicitly when they break out.
    pub(crate) fn note_cancel_observed(&self) {
        self.observed_cancel.store(true, Ordering::SeqCst);
    }

    pub(crate) fn cancel_was_observed(&self) -> bool {
        self.observed_cancel.load(Ordering::SeqCst)
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
    // Does `work` poll [`JobHandle::is_cancelled`] (or the raw flag)? Stated explicitly at the
    // call site rather than inferred, so a future worker that does NOT poll is forced to pass
    // `false` and cannot silently inherit a Cancel button that would do nothing.
    cancellable: bool,
    work: F,
) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(JobHandle) -> T + Send + 'static,
{
    let id = Uuid::new_v4();
    let cancel = Arc::new(AtomicBool::new(false));
    let handle = register(&reg, id, kind, label, items, cancel, cancellable);
    handle.running(total);
    let finalize = handle.clone();
    match tauri::async_runtime::spawn_blocking(move || work(handle)).await {
        Ok(out) => {
            // "Cancelled" must mean the work actually STOPPED, not merely that the user clicked.
            // Only a handful of job kinds poll the flag; the rest run to completion and commit
            // their writes, and labelling those Cancelled told the user the opposite of what
            // happened — an import that wrote 120 wells reported as cancelled, every item ticked
            // green. A worker that never observed the flag cannot have drained early, so the
            // honest report for it is Completed: the cancel simply arrived too late to matter.
            if finalize.cancel_was_observed() {
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
    // `false`: a simple job's worker is a bare `FnOnce() -> Result` with no `JobHandle`, so it
    // structurally cannot observe the cancel flag. This is the class of op the Processing panel
    // used to offer an inert Cancel button on — a render, an export, a single subprocess.
    run_job(reg, kind, label.clone(), vec![(String::from("op"), label)], 1, false, move |job| {
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
    cancellable: bool,
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
    let observed_cancel = Arc::new(AtomicBool::new(false));
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
            cancellable,
        },
    );
    prune(&mut store);
    JobHandle { reg: reg.clone(), id, cancel, observed_cancel }
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
            outcome: aggregate_outcome(j),
            total: j.total,
            done: j.done,
            current: j.current.clone(),
            items: j.items.clone(),
            error: j.error.clone(),
            seq: j.seq,
            cancellable: j.cancellable,
        })
        .collect();
    views.sort_by(|a, b| b.seq.cmp(&a.seq));
    views
}

fn aggregate_outcome(job: &Job) -> Option<JobOutcome> {
    if job.phase == JobPhase::Failed || job.items.iter().any(|item| item.state == ItemState::Failed)
    {
        Some(JobOutcome::Failed)
    } else if job.items.iter().any(|item| item.state == ItemState::Warned) {
        Some(JobOutcome::Degraded)
    } else if job.phase == JobPhase::Completed {
        Some(JobOutcome::Clean)
    } else {
        None
    }
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
        let h = register(&reg, id, "Workflow chain", "vsh_gr → phi_dn", items, flag.clone(), true);

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
        let h = register(&reg, id, "Module", "sw_indo", vec![], flag.clone(), true);
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
        let ah = register(&reg, active, "Chain", "keep-me", vec![], flag.clone(), true);
        ah.running(1);
        // Flood with finished jobs beyond the cap.
        for _ in 0..(MAX_FINISHED + 10) {
            let id = Uuid::new_v4();
            let h = register(&reg, id, "Module", "x", vec![], Arc::new(AtomicBool::new(false)), true);
            h.complete();
        }
        let views = list(&reg);
        let finished = views.iter().filter(|v| v.phase == JobPhase::Completed).count();
        assert!(finished <= MAX_FINISHED, "finished jobs capped at {MAX_FINISHED}");
        assert!(views.iter().any(|v| v.id == active.to_string()), "active job survives pruning");
    }

    /// `cancellable` rides through to the JobView both ways, so the panel can hide the Cancel
    /// button on a job whose worker never observes the flag — the visible half of the same
    /// cancel-honesty defect: a button that does nothing is as much a lie as a false "Cancelled".
    #[test]
    fn cancellable_flag_reaches_the_view_both_ways() {
        let reg = new_registry();
        let no_poll = Uuid::new_v4();
        register(&reg, no_poll, "Report", "render", vec![], Arc::new(AtomicBool::new(false)), false);
        let polls = Uuid::new_v4();
        register(&reg, polls, "Module", "sw_indo", vec![], Arc::new(AtomicBool::new(false)), true);

        let views = list(&reg);
        let get = |id: Uuid| views.iter().find(|v| v.id == id.to_string()).expect("job present");
        assert!(!get(no_poll).cancellable, "a monolithic op must report not-cancellable");
        assert!(get(polls).cancellable, "a polling worker must report cancellable");
    }

    /// The flag being set is NOT evidence the work stopped. Only ~5 of the ~27 job kinds poll it;
    /// the rest ran to completion, committed their writes, and were then reported "Cancelled"
    /// with every item ticked green. `run_job` now finalizes on the OBSERVATION instead, so this
    /// is the distinction the whole fix rests on.
    #[test]
    fn cancel_counts_as_cancelled_only_once_a_worker_observes_it() {
        let reg = new_registry();
        let cancel = Arc::new(AtomicBool::new(false));
        let h = register(
            &reg,
            Uuid::new_v4(),
            "Test",
            "unit",
            vec![("a".into(), "A".into())],
            cancel.clone(),
            true,
        );

        assert!(!h.cancel_was_observed(), "nothing observed before anything happens");
        assert!(!h.is_cancelled(), "polling an unset flag must not mark it observed");
        assert!(!h.cancel_was_observed());

        // The user clicks Cancel.
        cancel.store(true, Ordering::SeqCst);
        assert!(
            !h.cancel_was_observed(),
            "a set flag alone is not evidence: a worker that never polls cannot have stopped"
        );

        // A worker polls — now, and only now, the run really did drain.
        assert!(h.is_cancelled());
        assert!(h.cancel_was_observed(), "polling a set flag records the observation");

        // The observation is shared across clones, because workers poll from rayon threads
        // holding their own clone of the handle.
        let worker = h.clone();
        assert!(worker.cancel_was_observed(), "clones share the observation");
    }

    /// The raw-flag paths (chain steps, module runs) never call `is_cancelled`, so they mark the
    /// observation explicitly. Without that a genuinely drained run would report Completed — the
    /// same lie in the opposite direction.
    #[test]
    fn note_cancel_observed_marks_it_for_raw_flag_readers() {
        let reg = new_registry();
        let cancel = Arc::new(AtomicBool::new(false));
        let h = register(
            &reg,
            Uuid::new_v4(),
            "Test",
            "unit",
            vec![("a".into(), "A".into())],
            cancel.clone(),
            true,
        );
        cancel.store(true, Ordering::SeqCst);
        assert!(!h.cancel_was_observed());
        h.note_cancel_observed();
        assert!(h.cancel_was_observed());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn every_displayed_cancel_reaches_an_observing_worker_and_completed_work_is_never_reported_cancelled(
    ) {
        // CORRECTNESS for SB-CORE-036: its 2026-08-07 correction is the normative source that
        // Cancel is offered only for an observing worker and a click cannot relabel committed work.
        // CHARACTERIZATION for SB-DBM-040 / SB-DBM-T40: the chapter explicitly classifies this
        // same shipped three-job behavior as characterization rather than a newly derived result.
        let reg = new_registry();

        run_job(
            reg.clone(),
            "Test",
            "clicked after completion",
            vec![],
            0,
            true,
            |job| {
                job.cancel.store(true, Ordering::SeqCst);
            },
        )
        .await
        .expect("the non-observing worker completes");

        run_job(
            reg.clone(),
            "Test",
            "worker observed cancellation",
            vec![],
            0,
            true,
            |job| {
                job.cancel.store(true, Ordering::SeqCst);
                assert!(job.is_cancelled(), "the worker observes and honours the request");
            },
        )
        .await
        .expect("the observing worker drains cleanly");

        run_simple_job(reg.clone(), "Test", "monolithic", || Ok::<_, String>(()))
            .await
            .expect("the monolithic worker completes");

        let views = list(&reg);
        let view = |label: &str| {
            views
                .iter()
                .find(|view| view.label == label)
                .unwrap_or_else(|| panic!("job {label:?} is present"))
        };
        assert_eq!(
            view("clicked after completion").phase,
            JobPhase::Completed,
            "a request no worker observed did not stop the completed work"
        );
        assert!(view("clicked after completion").cancellable);
        assert_eq!(
            view("worker observed cancellation").phase,
            JobPhase::Cancelled,
            "an observed request reports the work as cancelled"
        );
        assert!(view("worker observed cancellation").cancellable);
        assert_eq!(view("monolithic").phase, JobPhase::Completed);
        assert!(
            !view("monolithic").cancellable,
            "a worker with no cancellation handle cannot offer Cancel"
        );

        let panel = include_str!("../../src/ui/processingPanel.ts");
        assert!(
            panel.contains("if (active && job.cancellable)"),
            "the Processing panel exposes Cancel only for an active observing worker"
        );
        assert!(
            panel.contains("tag.textContent = \"can't be interrupted\""),
            "active monolithic work states its non-interruptible limit"
        );

        let lib = include_str!("lib.rs");
        let cancellable_registrations = [
            "ingest::import_las_files_with(&c, &paths, Some(&job), &opts)",
            "python_engine::run_python_equation(&conn, &equation, &well_ids, &custody, Some(&job))",
            "equations::run_equation(&conn, &equation, &well_ids, &custody, Some(&job))",
            "workflow::run_workflow_module_into(&conn, &req, None, Some(&job.cancel), Some(&job))",
            "montecarlo::run_monte_carlo(&conn, &req, Some(&job))",
            "ml::run_ml(&conn, &req, Some(&job))",
            "ml::apply_ml_model(&conn, &req, Some(&job))",
            "sandimin::run_sandimin(&conn, &req, Some(&job))",
        ];
        assert_eq!(
            lib.matches("jobs::run_job(").count(),
            7,
            "every live run_job registration must be reviewed by this inventory"
        );
        for registration in cancellable_registrations {
            assert!(
                lib.contains(registration),
                "cancellable registration must still route its observing handle: {registration}"
            );
        }
        assert_eq!(
            lib.matches("jobs::register(").count(),
            1,
            "every manual job registration must be reviewed by this inventory"
        );
        assert!(lib.contains(
            "jobs::register(jobs_reg.inner(), uuid, \"Workflow chain\", label, items, cancel.clone(), true)"
        ));
        assert!(lib.contains("chain::cancel(registry.inner(), uuid);"));

        let observers = [
            (include_str!("ingest.rs"), "p.is_cancelled()", "LAS import"),
            (include_str!("equations.rs"), "p.is_cancelled()", "Rhai equation"),
            (include_str!("python_engine.rs"), "p.is_cancelled()", "Python equation"),
            (include_str!("workflow.rs"), "p.note_cancel_observed()", "workflow module"),
            (include_str!("montecarlo.rs"), "p.is_cancelled()", "Monte Carlo"),
            (include_str!("ml.rs"), "p.is_cancelled()", "machine learning"),
            (include_str!("sandimin.rs"), "p.is_cancelled()", "SandiMin"),
        ];
        for (source, observer, family) in observers {
            assert!(
                source.contains(observer),
                "{family} is advertised cancellable only while its worker observes cancellation"
            );
        }

        let chain = include_str!("chain.rs");
        assert!(
            chain.contains("j.note_cancel_observed();"),
            "the chain loop records a cancellation that prevents a later step from starting"
        );
        assert!(
            chain.contains("j.cancel_was_observed()"),
            "the final chain check distinguishes a drained last step from a late click"
        );
        let workflow_dialog = include_str!("../../src/ui/workflowDialog.ts");
        assert!(workflow_dialog.contains("cancelBtn.disabled = false;"));
        assert!(workflow_dialog.contains("await cancelWorkflowChain(currentJob).catch(() => {});"));
    }
}
