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

/// Zeroes every counter. A caller measuring one module must call this first, or it reads the
/// previous module's total as well as its own.
pub(crate) fn reset() {
    WAIT_NS.store(0, Ordering::Relaxed);
    READ_NS.store(0, Ordering::Relaxed);
    WRITE_NS.store(0, Ordering::Relaxed);
    WELL_NS.store(0, Ordering::Relaxed);
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
