//! Phase timing for the module runner — **`#[cfg(test)]` only, never in a shipped build.**
//!
//! Pass 3 increment 2 of the performance brief. Increment 1 proved that the single shared
//! `Mutex<Connection>` serialises parallel reads (32 rayon threads bought 3%) and that one
//! connection per thread would buy 4–8× on the read it measured. What it could NOT say is how
//! much of a real module run is that read — so it could not turn "4–8× on reads" into "X minutes
//! off a 23-minute chain". This is that measurement.
//!
//! ## Why it lives in production code at all
//!
//! Increment 1 measured from outside, which is always preferable. It ran out of road: the phases
//! it needs to separate are interleaved inside one private closure in `workflow.rs`, and no
//! caller can see the boundary between them. The only honest options were to guess the split or
//! to time it where it happens.
//!
//! The brief's rule is that a profiling harness lives in `tools/` or an `#[ignore]`d test, **never
//! in the shipped app**. So every call site is written `#[cfg(test)] let _g = …`, an attribute on
//! a `let` statement: in `cargo build --release` the statement does not exist, so there is no
//! branch, no atomic, and no timer to optimise away. `cargo test` compiles with `cfg(test)` and
//! the probe is live. This module is likewise `#[cfg(test)] mod` in `lib.rs`.
//!
//! ## What a number here means
//!
//! Every counter is a **sum across wells and across rayon threads**, not wall-clock. That is the
//! right shape for a split — it asks how much WORK each phase costs, independent of how it was
//! scheduled. It also carries its own check: while the loop is serialised the sum tracks the
//! wall-clock, and the day a pool lands they must diverge. If they ever stop matching, that is
//! the parallelism working, not the probe breaking.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Time spent WAITING to acquire the shared connection, before any work begins. This is the queue
/// — pure contention, doing nothing — and it is the phase a connection pool deletes outright.
///
/// It must be timed separately or it is not merely missing but MISATTRIBUTED: COMPUTE is derived
/// as (per-well total minus the timed phases), so an untimed wait lands in COMPUTE and reads as
/// arithmetic. Measured before this counter existed: 466 ms per well of "compute" for a module
/// that costs 52 ms run on its own, which reported reads as 3.5% of the work and a pool as worth
/// 1.03x — the opposite of the truth, carrying a decimal point.
pub(crate) static WAIT_NS: AtomicU64 = AtomicU64::new(0);
/// Time inside a lock scope that only READS, once the lock is HELD. A connection pool parallelises
/// this phase rather than removing it.
pub(crate) static READ_NS: AtomicU64 = AtomicU64::new(0);
/// Time inside the batched WRITE and the log-set bookkeeping around it. DuckDB is single-writer by
/// design, so this phase is serial no matter what happens to connection semantics — it is the
/// floor under any pooling win.
pub(crate) static WRITE_NS: AtomicU64 = AtomicU64::new(0);
/// The five parts of WRITE_NS, so 62% of a real chain stops being one opaque number.
///
/// They are SUBDIVISIONS, not additions: each is inside the WRITE scope, so they must sum to no
/// more than WRITE_NS, and the shortfall is the sixth part - the transaction commit plus the
/// per-well class-curve declaration - derived rather than timed, the same way COMPUTE is.
///
/// Per-well checks before the transaction opens: depth-uniqueness validation over every curve,
/// and the set write discipline. CPU and small reads, no rows written.
pub(crate) static W_VALIDATE_NS: AtomicU64 = AtomicU64::new(0);
/// Phase 1 - the grouped DELETE that makes the write idempotent. This is the cost of the PK-less
/// write discipline: nothing prevents a duplicate except removing the prior rows first.
pub(crate) static W_DELETE_NS: AtomicU64 = AtomicU64::new(0);
/// Phase 2 - appending every sample to `computed_curves`, the readable interpretation.
pub(crate) static W_CURRENT_NS: AtomicU64 = AtomicU64::new(0);
/// Phase 3 - appending every sample AGAIN to `computed_curves_archive`. Every row is written
/// twice by design, which is what makes a re-run non-destructive; this counter is what says
/// whether that design costs half the write or a tenth of it.
pub(crate) static W_ARCHIVE_NS: AtomicU64 = AtomicU64::new(0);
/// Phase 4 - the degradation records that classify the run, in the same transaction as the rows.
pub(crate) static W_DEGRADE_NS: AtomicU64 = AtomicU64::new(0);

/// The five parts of the pay summary's FLAG-curve write, which happens INSIDE its per-well loop
/// rather than once for the run.
///
/// The harness already reports the same summation twice - `stats_only` (no write) at ~170 ms over
/// 100 wells, and the writing variant at ~5.7 s - so the write is almost all of it. That is a
/// SUBTRACTION between two calls with different cache states, though, so it names the block and
/// not the part. These counters time the parts, so the fix is chosen by measurement rather than by
/// the resemblance to a per-well pattern already fixed twice elsewhere.
///
/// They are subdivisions of the per-well write block, so they sum to no more than it.
///
/// Waiting for the connection mutex, per well. The summation loop is serial, so this should be
/// near zero - and if it is NOT, something else holds the connection while a summary runs.
pub(crate) static PS_LOCK_NS: AtomicU64 = AtomicU64::new(0);
/// `complete_curve_run_spec` plus `record_parameter_decisions`: building the provenance record for
/// this well's run. Reads, per well.
pub(crate) static PS_SPEC_NS: AtomicU64 = AtomicU64::new(0);
/// The three `curve_ancestry` lookups that ask whether an identical computation is already current,
/// so previewing and then exporting a report does not create two indistinguishable versions. Three
/// queries per well, and the only part whose cost is paid even when nothing is written.
pub(crate) static PS_ANCESTRY_NS: AtomicU64 = AtomicU64::new(0);
/// `create_complete_log_set`: allocating the PAYFLAG version and its parameter rows, per well.
pub(crate) static PS_SET_NS: AtomicU64 = AtomicU64::new(0);
/// `write_computed_curves_with_ancestry`: the three FLAG curves themselves. This is ONE
/// transaction per well, where a chain step writes every well in one - which is the hypothesis
/// this counter exists to confirm or refute.
pub(crate) static PS_ROWS_NS: AtomicU64 = AtomicU64::new(0);

/// Whole per-well work, from the top of the runner's per-well closure to its end. COMPUTE is this
/// minus the read scopes inside it — derived rather than timed directly, because the arithmetic is
/// scattered through the closure and bracketing it would need far more call sites than the answer
/// is worth.
pub(crate) static WELL_NS: AtomicU64 = AtomicU64::new(0);

/// Accumulates its lifetime into one counter when it drops, so a phase is bracketed by the scope
/// it already has rather than by a matched pair of calls somebody can forget to close.
pub(crate) struct Phase(&'static AtomicU64, Instant);

impl Drop for Phase {
    fn drop(&mut self) {
        self.0.fetch_add(self.1.elapsed().as_nanos() as u64, Ordering::Relaxed);
    }
}

pub(crate) fn wait() -> Phase {
    Phase(&WAIT_NS, Instant::now())
}

pub(crate) fn read() -> Phase {
    Phase(&READ_NS, Instant::now())
}

pub(crate) fn write() -> Phase {
    Phase(&WRITE_NS, Instant::now())
}

pub(crate) fn well() -> Phase {
    Phase(&WELL_NS, Instant::now())
}

pub(crate) fn w_validate() -> Phase {
    Phase(&W_VALIDATE_NS, Instant::now())
}

pub(crate) fn w_delete() -> Phase {
    Phase(&W_DELETE_NS, Instant::now())
}

pub(crate) fn w_current() -> Phase {
    Phase(&W_CURRENT_NS, Instant::now())
}

pub(crate) fn w_archive() -> Phase {
    Phase(&W_ARCHIVE_NS, Instant::now())
}

pub(crate) fn w_degrade() -> Phase {
    Phase(&W_DEGRADE_NS, Instant::now())
}

pub(crate) fn ps_lock() -> Phase {
    Phase(&PS_LOCK_NS, Instant::now())
}

pub(crate) fn ps_spec() -> Phase {
    Phase(&PS_SPEC_NS, Instant::now())
}

pub(crate) fn ps_ancestry() -> Phase {
    Phase(&PS_ANCESTRY_NS, Instant::now())
}

pub(crate) fn ps_set() -> Phase {
    Phase(&PS_SET_NS, Instant::now())
}

pub(crate) fn ps_rows() -> Phase {
    Phase(&PS_ROWS_NS, Instant::now())
}

/// Zeroes every counter. A caller measuring one module must call this first, or it reads the
/// previous module's total as well as its own.
pub(crate) fn reset() {
    WAIT_NS.store(0, Ordering::Relaxed);
    READ_NS.store(0, Ordering::Relaxed);
    WRITE_NS.store(0, Ordering::Relaxed);
    WELL_NS.store(0, Ordering::Relaxed);
    W_VALIDATE_NS.store(0, Ordering::Relaxed);
    W_DELETE_NS.store(0, Ordering::Relaxed);
    W_CURRENT_NS.store(0, Ordering::Relaxed);
    W_ARCHIVE_NS.store(0, Ordering::Relaxed);
    W_DEGRADE_NS.store(0, Ordering::Relaxed);
    PS_LOCK_NS.store(0, Ordering::Relaxed);
    PS_SPEC_NS.store(0, Ordering::Relaxed);
    PS_ANCESTRY_NS.store(0, Ordering::Relaxed);
    PS_SET_NS.store(0, Ordering::Relaxed);
    PS_ROWS_NS.store(0, Ordering::Relaxed);
}

/// `(lock, spec, ancestry, set, rows)` in nanoseconds - the five timed parts of the pay summary's
/// per-well FLAG write. Whatever the caller's own elapsed time is minus their sum is the summation
/// arithmetic plus the reads, and it is a REMAINDER, derived rather than timed, exactly as the
/// write split's sixth part is.
pub(crate) fn pay_summary_split() -> (u64, u64, u64, u64, u64) {
    (
        PS_LOCK_NS.load(Ordering::Relaxed),
        PS_SPEC_NS.load(Ordering::Relaxed),
        PS_ANCESTRY_NS.load(Ordering::Relaxed),
        PS_SET_NS.load(Ordering::Relaxed),
        PS_ROWS_NS.load(Ordering::Relaxed),
    )
}

/// `(validate, delete, current, archive, degrade)` in nanoseconds - the five timed parts of the
/// write. The sixth, commit plus class declaration, is `WRITE_NS` minus their sum; it is reported
/// as a remainder so it can never be mistaken for something that was measured.
pub(crate) fn write_split() -> (u64, u64, u64, u64, u64) {
    (
        W_VALIDATE_NS.load(Ordering::Relaxed),
        W_DELETE_NS.load(Ordering::Relaxed),
        W_CURRENT_NS.load(Ordering::Relaxed),
        W_ARCHIVE_NS.load(Ordering::Relaxed),
        W_DEGRADE_NS.load(Ordering::Relaxed),
    )
}

/// `(wait, read, write, per-well total)` in nanoseconds.
pub(crate) fn snapshot() -> (u64, u64, u64, u64) {
    (
        WAIT_NS.load(Ordering::Relaxed),
        READ_NS.load(Ordering::Relaxed),
        WRITE_NS.load(Ordering::Relaxed),
        WELL_NS.load(Ordering::Relaxed),
    )
}
