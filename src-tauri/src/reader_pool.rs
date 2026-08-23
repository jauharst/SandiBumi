//! Read handles on the live project — #129, the connection pool.
//!
//! `PERF-POOL-RISK-2026-08-23.md` measured what this is worth (**1.95x at 4 readers, 2.15x at 8**
//! on a real 100-well chain) and reasoned seven corruption modes. The value is almost entirely in
//! deleting the QUEUE: 90% of a batch run is rayon threads standing in line for one shared
//! connection, while the petrophysics itself is 0.1%.
//!
//! Stage 1 (2026-08-23) built the safety catch with a single handle and no concurrency at all, so
//! that if the catch were wrong it would surface with every other variable held still. Stage 2 is
//! the concurrency.
//!
//! ## M1, the silent one
//!
//! Measured, not assumed: `Connection::try_clone` produces a handle that **outlives the handle it
//! was cloned from and keeps answering**. SandiBumi replaces the live connection in place at three
//! sites — `project::compact_project`, `project::switch_project` (Open Project and New Project)
//! and `lib::publish_open_outcome` (the startup swap, which replaces the in-memory placeholder the
//! window is built on). A pooled handle minted before one of those would go on serving rows out of
//! the OLD database file, and those rows look entirely normal.
//!
//! (Save As deliberately does NOT appear in that list. It copies the project to a new file with
//! `db::engine_copy_to` and leaves the current one open, so there is no swap to invalidate.)
//!
//! **The generation stamp is the whole protection, and there is no second one.** That was assumed
//! to be otherwise and measured to be wrong, which is why it is written down here. The assumption
//! was that an open handle would block `compact_project`'s rename on Windows, giving a loud
//! accidental guard behind the quiet one. It does not: a mutation that bumped the generation but
//! left the handle open ran Compact Project to completion, rename and all.
//!
//! `invalidate` still releases the handles, for a smaller and truer reason: a reader on a database
//! that is being replaced has no further use, and holding one keeps the outgoing DuckDB instance
//! alive past the point `compact_project` expects to have closed it.
//!
//! ## A read that is in flight when the project changes
//!
//! Stage 1 could not have one: a read held the `DbState` mutex for its whole duration, and that is
//! the mutex a swap needs. Stage 2 deliberately gives that up — it is the queue, and the queue is
//! the thing worth deleting — so the window exists, and is closed two ways.
//!
//! **Structurally**, a swap cannot start while a job is running: `open_project`, `new_project` and
//! `compact_project` each refuse with `chain::any_active(..) || jobs::any_active(..)`, and pooled
//! reads in the runner only happen inside a job. The startup swap predates every job.
//!
//! **And loudly**, because that guard is a check followed by an action rather than one atomic
//! step: the generation is re-checked when the read finishes, and a read whose project moved
//! underneath returns an ERROR instead of its answer. A chain step then fails and says so, which
//! is the outcome to want — the alternative is an interpretation computed from one project and
//! written into another, which nothing downstream could ever detect.
//!
//! ## Lock order — the one rule
//!
//! **The pool's own lock may be taken alone, or while the connection mutex is held. Never the
//! reverse.** `DbState::install` performs a swap holding the connection mutex and calls
//! `invalidate`, which takes the pool lock — so a `read` that held the pool lock while reaching
//! for the connection would deadlock the two against each other. `read` therefore releases the
//! pool lock before it touches the connection, and takes the connection FIRST when it has to mint.

use duckdb::Connection;
use std::sync::Mutex;

/// How many idle handles are kept. The measurement modelled 4 and 8 readers (1.95x and 2.15x), and
/// past that the write is 80%+ of what remains, so more handles buy progressively nothing while
/// each one costs DuckDB per-connection buffers on a field machine.
const MAX_IDLE: usize = 8;

pub struct ReaderPool {
    inner: Mutex<PoolInner>,
    capacity: usize,
}

struct PoolInner {
    /// Bumped every time the live connection is replaced.
    generation: u64,
    idle: Vec<Stamped>,
}

struct Stamped {
    generation: u64,
    conn: Connection,
}

impl Default for ReaderPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderPool {
    pub fn new() -> Self {
        let capacity = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, MAX_IDLE);
        Self { inner: Mutex::new(PoolInner { generation: 0, idle: Vec::new() }), capacity }
    }

    /// Announce that the live connection has been replaced.
    ///
    /// **Must be called while the connection mutex is held, before the old connection is dropped.**
    /// Holding it is what makes this atomic against a reader that is minting: minting takes the
    /// connection first, so it cannot be part-way through while this runs.
    ///
    /// A poisoned lock is recovered rather than propagated. A panic elsewhere must not turn every
    /// later project switch into a second failure — and under `panic = "abort"` a shipped build
    /// never reaches this at all (`CLAUDE.md`, the diagnostic-report section).
    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.generation = inner.generation.wrapping_add(1);
        // Dropped HERE, inside the caller's connection guard, so no handle on the outgoing
        // database outlives the swap.
        inner.idle.clear();
    }

    /// Run one read on the live project, without queueing behind every other reader.
    ///
    /// `live` is the same `Mutex<Connection>` every caller already holds a reference to; it is
    /// locked ONLY to mint a new handle, never for the duration of the read itself. That is the
    /// whole speed-up: `PERF-SPLIT-2026-08-23.md` measured 90.9% of a batch run as time spent
    /// waiting for this lock, against 3.5% spent reading through it.
    pub fn read<T>(
        &self,
        live: &Mutex<Connection>,
        read: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        // Step 1: an idle handle, if one from the current generation is going spare. The pool lock
        // is taken alone and released before anything else is touched - see the lock-order note.
        let reusable = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            let generation = inner.generation;
            loop {
                match inner.idle.pop() {
                    Some(handle) if handle.generation == generation => break Some(handle),
                    // Minted under an older project. Dropped here rather than served; reaching this
                    // means a swap site skipped `invalidate`, and the stamp catches it anyway.
                    Some(_stale) => continue,
                    None => break None,
                }
            }
        };

        // Step 2: mint if there was nothing to reuse. The connection is taken FIRST and the pool
        // lock second, matching the order `DbState::install` uses - and holding the connection is
        // what makes the generation read here trustworthy, because a swap cannot be in progress.
        let stamped = match reusable {
            Some(handle) => handle,
            None => {
                #[cfg(test)]
                let _phase_wait = crate::lock_probe::wait();
                let guard = live.lock().map_err(|_| {
                    "read refused: the project connection is unavailable because an earlier \
                     operation left it in an unknown state. Reopen the project."
                        .to_string()
                })?;
                let generation = self.inner.lock().unwrap_or_else(|e| e.into_inner()).generation;
                let conn = guard
                    .try_clone()
                    .map_err(|e| format!("could not open a read handle on the project: {e}"))?;
                Stamped { generation, conn }
            }
        };

        // Step 3: the read itself, holding no lock at all. This is the part that now runs in
        // parallel across wells.
        let answer = {
            #[cfg(test)]
            let _phase_read = crate::lock_probe::read();
            read(&stamped.conn)
        };

        // Step 4: was it still the same project when this finished? A swap cannot normally happen
        // here (see the module note on the job guards), but that guard is a check followed by an
        // action, so this closes the gap rather than trusting it. An error is the right outcome:
        // the alternative is an answer computed from a project that is no longer open.
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if stamped.generation != inner.generation {
            return Err("read refused: the project was replaced while this read was in flight, so \
                        its answer describes the project that was open before. Nothing has been \
                        written. Run it again now the switch has finished."
                .to_string());
        }
        if inner.idle.len() < self.capacity {
            inner.idle.push(stamped);
        }
        answer
    }

    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.inner.lock().unwrap().generation
    }

    #[cfg(test)]
    pub(crate) fn idle_handles(&self) -> usize {
        self.inner.lock().unwrap().idle.len()
    }

    #[cfg(test)]
    pub(crate) fn holds_idle_handle(&self) -> bool {
        self.idle_handles() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with(well: &str) -> Mutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(&format!(
            "CREATE TABLE wells (well_name VARCHAR); INSERT INTO wells VALUES ('{well}');"
        ))
        .unwrap();
        Mutex::new(conn)
    }

    fn only_well(conn: &Connection) -> Result<String, String> {
        conn.query_row("SELECT well_name FROM wells", [], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())
    }

    /// M1, the one silent corruption mode in `PERF-POOL-RISK-2026-08-23.md` §3. A cloned handle
    /// outlives the handle it came from and keeps answering — measured, which is why this exists —
    /// so a reader minted before a project swap would serve the OLD project's rows, and they look
    /// exactly like the right answer.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. A pool that
    /// re-cloned on every single read would satisfy "never serves a stale answer" perfectly and
    /// would be no pool at all; a pool that reused forever would satisfy "reuses its handle" and be
    /// the bug. So: **it reuses within a generation, and refuses across one.**
    #[test]
    fn a_reader_minted_before_a_project_swap_is_never_served_after_it() {
        let pool = ReaderPool::new();
        let mut live = project_with("SANDI-1");

        // Arm 1 - it really does pool. Without this the "refuses across a generation" half below
        // is a statement about nothing.
        assert_eq!(pool.read(&live, only_well).unwrap(), "SANDI-1");
        assert!(pool.holds_idle_handle(), "the first read must leave its handle in the pool");
        assert_eq!(pool.read(&live, only_well).unwrap(), "SANDI-1");

        // Arm 2 - the swap. This is what `DbState::install` does: invalidate under the connection
        // guard, then replace the connection.
        let before = pool.generation();
        pool.invalidate();
        assert!(!pool.holds_idle_handle(), "invalidate must release every handle");
        assert_eq!(pool.generation(), before + 1, "a swap must bump the generation");
        live = project_with("SANDI-2");

        // The reader now answers from the NEW project. Without the stamp it would still be
        // answering SANDI-1, and nothing about that answer would look wrong.
        assert_eq!(
            pool.read(&live, only_well).unwrap(),
            "SANDI-2",
            "a pooled reader served the previous project after a swap"
        );
    }

    /// The belt to the braces. `invalidate` releasing the handles is what stops a reader on the
    /// outgoing database outliving the swap; the STAMP is what still refuses one if a future swap
    /// site forgets to call it. Simulated by bumping the generation behind the pool's back — which
    /// is precisely what a forgotten `invalidate` leaves: idle handles from an older world.
    #[test]
    fn the_stamp_refuses_a_stale_handle_even_if_a_swap_site_forgot_to_invalidate() {
        let pool = ReaderPool::new();
        let live = project_with("SANDI-1");
        assert_eq!(pool.read(&live, only_well).unwrap(), "SANDI-1");
        assert!(pool.holds_idle_handle());

        {
            let mut inner = pool.inner.lock().unwrap();
            inner.generation += 1;
        }
        let replacement = project_with("SANDI-2");
        assert_eq!(
            pool.read(&replacement, only_well).unwrap(),
            "SANDI-2",
            "the stamp is the only thing between a forgotten invalidate and a stale answer"
        );
    }

    /// Stage 2 gives up holding the connection for the duration of a read, which is where the
    /// speed comes from and which opens a window stage 1 could not have: the project changing
    /// while a read is still running.
    ///
    /// The structural guard is that `open_project`, `new_project` and `compact_project` all refuse
    /// while a job is active. But that is a check followed by an action, not one atomic step, so
    /// the answer must not be usable if the window is ever hit. **An error is the required
    /// outcome, not a stale answer and not a silent empty one** — an interpretation computed from
    /// one project and written into another is undetectable afterwards.
    #[test]
    fn a_read_whose_project_changed_underneath_it_returns_an_error_not_an_answer() {
        let pool = ReaderPool::new();
        let live = project_with("SANDI-1");

        let outcome = pool.read(&live, |conn| {
            let answer = only_well(conn)?;
            // The swap lands mid-read. Nothing else in the process could tell afterwards.
            pool.invalidate();
            Ok(answer)
        });

        let error = outcome.expect_err("a read that spanned a project swap must not return rows");
        assert!(
            error.contains("replaced while this read was in flight"),
            "the refusal must say what happened, got: {error}"
        );
        assert!(
            error.contains("Nothing has been written"),
            "a refusal a chain step will surface must say whether it left anything behind: {error}"
        );
        assert!(!pool.holds_idle_handle(), "the handle from the old project must not be kept");
    }

    /// The pool is bounded. Handles beyond the cap are dropped rather than accumulated: each one
    /// costs DuckDB per-connection buffers, and past 8 readers the write is 80%+ of what remains
    /// (`PERF-POOL-RISK-2026-08-23.md` §5), so more of them buy progressively nothing.
    #[test]
    fn the_pool_keeps_at_most_its_capacity_and_drops_the_rest() {
        let pool = ReaderPool::new();
        let live = project_with("SANDI-1");
        assert!(pool.capacity <= MAX_IDLE, "the cap must not exceed {MAX_IDLE}");

        // Nested reads each hold their own handle, so this forces more of them into existence at
        // once than a sequential loop ever would.
        fn nest(pool: &ReaderPool, live: &Mutex<Connection>, depth: usize) -> Result<(), String> {
            if depth == 0 {
                return Ok(());
            }
            pool.read(live, |_| nest(pool, live, depth - 1))
        }
        nest(&pool, &live, MAX_IDLE + 4).unwrap();

        assert_eq!(
            pool.idle_handles(),
            pool.capacity,
            "the pool must fill to its cap and keep no more"
        );
    }

    /// The gate behind `DbState::install`. The guarantee above dies not from a wrong line but from
    /// a MISSING one, at a swap site written six months from now by somebody who has never read
    /// this file.
    ///
    /// So the invalidation is not a step a caller can forget: there is exactly ONE call to it in
    /// production code, inside `install`, and `install` is the only production route to replacing
    /// the live connection. This test refuses both halves of any attempt to write a second route.
    ///
    /// Pinned from both sides, because either half alone passes for the wrong reason. "Exactly one
    /// invalidate" is satisfied by a codebase where nothing swaps the connection at all; "install
    /// invalidates" is satisfied while three other sites quietly assign around it. So: the call
    /// count is pinned AND the sites are pinned AND `install` is asserted to still contain the call.
    ///
    /// Needles are assembled from pieces so this test is never an occurrence of what it counts.
    #[test]
    fn a_swap_of_the_live_connection_cannot_be_written_without_invalidating_the_pool() {
        let bump = [".invalid", "ate()"].concat();
        let installer = ["pub fn ", "install"].concat();
        let assign = ["*guard", " = "].concat();
        let replace = ["mem::", "replace"].concat();

        let production = |source: &str| -> String {
            source.split("\nmod tests").next().expect("a split always yields one piece").to_string()
        };

        // Arm 1 - the call count, across every production file that could reach the pool. A second
        // call site anywhere is a second thing to keep in step with the swap, which is the state
        // this design exists to avoid.
        let files: [(&str, &str); 4] = [
            ("lib.rs", include_str!("lib.rs")),
            ("project.rs", include_str!("project.rs")),
            ("db.rs", include_str!("db.rs")),
            ("reader_pool.rs", include_str!("reader_pool.rs")),
        ];
        let calls: Vec<String> = files
            .iter()
            .flat_map(|(name, source)| {
                production(source)
                    .lines()
                    .filter(|line| {
                        line.contains(bump.as_str()) && !line.trim_start().starts_with("//")
                    })
                    .map(|line| format!("{name}: {}", line.trim()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            calls.len(),
            1,
            "the pool must be invalidated in exactly one production place; found {calls:?}"
        );
        assert!(calls[0].starts_with("lib.rs"), "the one call belongs in DbState::install");

        // Arm 2 - and that one place is `install`. Without this, arm 1 passes just as happily with
        // the call moved somewhere no swap goes through.
        let lib = production(include_str!("lib.rs"));
        let at = lib.find(installer.as_str()).expect("DbState::install is declared");
        let body_end = lib[at..].find("\n    }").expect("its body closes") + at;
        assert!(
            lib[at..body_end].contains(bump.as_str()),
            "the invalidation has moved out of {installer}, so a swap can be written without it"
        );

        // Arm 3 - no other production route to the assignment, in the two files that own the live
        // connection. Every match must sit inside `install`.
        for (name, source) in
            [("lib.rs", include_str!("lib.rs")), ("project.rs", include_str!("project.rs"))]
        {
            let body = production(source);
            let stray: Vec<&str> = body
                .lines()
                .filter(|line| line.contains(assign.as_str()) || line.contains(replace.as_str()))
                .filter(|line| !line.trim_start().starts_with("//"))
                .filter(|line| !(name == "lib.rs" && line.contains("live, incoming")))
                .collect();
            assert!(
                stray.is_empty(),
                "{name} replaces the live connection outside DbState::install, so the reader pool \
                 is not invalidated on that path: {stray:?}"
            );
        }
    }
}
