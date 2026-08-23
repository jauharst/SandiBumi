//! Stage 1 of #129 (connection pool) — the generation stamp, and nothing else.
//!
//! `PERF-POOL-RISK-2026-08-23.md` measured what a pool is worth (1.95x at 4 readers on a real
//! 100-well chain, up from 1.30x before the degradation batching) and reasoned seven corruption
//! modes. **M1 is the one that is silent**, and it is the only thing this file addresses.
//!
//! Measured, not assumed: `Connection::try_clone` produces a handle that **outlives the handle it
//! was cloned from and keeps answering**. SandiBumi replaces the live connection in place at three
//! sites — `project::compact_project`, `project::switch_project` (Open Project and New Project) and
//! `lib::publish_open_outcome` (the startup swap). A pooled handle minted before one of those would
//! go on serving rows out of the OLD database file, and those rows look entirely normal. The user
//! compacts a project, carries on working, and reads the rest of the session out of the pre-compact
//! file, with nothing to see and nothing to send us.
//!
//! (Save As deliberately does NOT appear in that list. It copies the project to a new file with
//! `db::engine_copy_to` and leaves the current one open, so there is no swap to invalidate.)
//!
//! **This stage buys no speed, by design and by type.** `with_reader` takes `&Connection` — the
//! live one, which a caller can only hold while holding the `DbState` mutex. So a pooled read
//! serialises exactly as it does today, and every variable except the invalidation machinery is
//! held still. Stage 2 relaxes that signature and is where the concurrency arrives; until then a
//! bug here cannot be a concurrency bug, which is the entire point of doing it in this order.
//!
//! **The generation stamp is the whole protection, and there is no second one.** That was assumed
//! to be otherwise and measured to be wrong, which is why it is written down here. The assumption
//! was that an open handle would block `compact_project`'s rename on Windows, giving a loud
//! accidental guard behind the quiet one. It does not: a mutation that bumped the generation but
//! left the handle open ran Compact Project to completion, rename and all. So a pool that stopped
//! stamping would fail silently on every path, with nothing else to catch it.
//!
//! `invalidate` still releases the handle, for a smaller and truer reason: a reader on a database
//! that is being replaced has no further use, and holding it keeps the outgoing DuckDB instance
//! alive past the point `compact_project` expects to have closed it.

use duckdb::Connection;
use std::sync::Mutex;

/// Read handles on the live project. Stage 1 holds at most ONE.
pub struct ReaderPool {
    inner: Mutex<PoolInner>,
}

struct PoolInner {
    /// Bumped every time the live connection is replaced.
    generation: u64,
    idle: Option<Stamped>,
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
        Self { inner: Mutex::new(PoolInner { generation: 0, idle: None }) }
    }

    /// Announce that the live connection has been replaced.
    ///
    /// **Must be called while the `DbState` mutex is held, before the old connection is dropped.**
    /// Holding that mutex is what makes this atomic with respect to every reader: a reader can only
    /// lease a handle while holding it too, so there is no window in which a handle is minted
    /// against a connection that is already being replaced.
    ///
    /// A poisoned lock is recovered rather than propagated. A panic elsewhere must not turn every
    /// later project switch into a second failure — and under `panic = "abort"` a shipped build
    /// never reaches this at all (`CLAUDE.md`, the diagnostic-report section).
    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.generation = inner.generation.wrapping_add(1);
        // Dropped HERE, inside the lock and inside the caller's `DbState` guard, so the file
        // handle is gone before the swap renames or closes anything.
        inner.idle = None;
    }

    /// Run one read through a pooled handle on the live project.
    ///
    /// `live` is the connection behind the `DbState` mutex, which the caller must already hold —
    /// see the module note on why stage 1 is deliberately shaped this way.
    pub fn with_reader<T>(
        &self,
        live: &Connection,
        read: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let stamped = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            let generation = inner.generation;
            match inner.idle.take() {
                // A handle from this generation is still good.
                Some(handle) if handle.generation == generation => handle,
                // A handle from an earlier one is dropped rather than served. Reaching this arm
                // means a swap site forgot to `invalidate`; the stamp catches it anyway.
                _ => Stamped {
                    generation,
                    conn: live
                        .try_clone()
                        .map_err(|e| format!("could not open a read handle on the project: {e}"))?,
                },
            }
        };

        let answer = read(&stamped.conn);

        // Return it only if it is still current. A panic in `read` simply drops it, which costs a
        // re-clone and nothing else - so no drop guard is needed to keep this correct.
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if stamped.generation == inner.generation {
            inner.idle = Some(stamped);
        }
        answer
    }

    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.inner.lock().unwrap().generation
    }

    #[cfg(test)]
    pub(crate) fn holds_idle_handle(&self) -> bool {
        self.inner.lock().unwrap().idle.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with(well: &str) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(&format!(
            "CREATE TABLE wells (well_name VARCHAR); INSERT INTO wells VALUES ('{well}');"
        ))
        .unwrap();
        conn
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

        // Arm 1 - it really does pool. The second read must reuse the handle the first left idle,
        // or the "refuses across a generation" half below is a statement about nothing.
        assert_eq!(pool.with_reader(&live, only_well).unwrap(), "SANDI-1");
        assert!(pool.holds_idle_handle(), "the first read must leave its handle in the pool");
        assert_eq!(pool.with_reader(&live, only_well).unwrap(), "SANDI-1");

        // Arm 2 - the swap. This is what `project.rs` does: invalidate under the guard, then
        // replace the live connection.
        let before = pool.generation();
        pool.invalidate();
        assert!(!pool.holds_idle_handle(), "invalidate must release the handle before the swap");
        assert_eq!(pool.generation(), before + 1, "a swap must bump the generation");
        live = project_with("SANDI-2");

        // The reader now answers from the NEW project. Without the stamp it would still be
        // answering SANDI-1, and nothing about that answer would look wrong.
        assert_eq!(
            pool.with_reader(&live, only_well).unwrap(),
            "SANDI-2",
            "a pooled reader served the previous project after a swap"
        );
    }

    /// The belt to the braces. `invalidate` releasing the handle is what makes Compact Project's
    /// rename work on Windows; the STAMP is what still refuses a stale handle if a future swap site
    /// forgets to call it. Simulated by bumping the generation behind the pool's back — which is
    /// precisely what a forgotten `invalidate` would leave: an idle handle from an older world.
    #[test]
    fn the_stamp_refuses_a_stale_handle_even_if_a_swap_site_forgot_to_invalidate() {
        let pool = ReaderPool::new();
        let live = project_with("SANDI-1");
        assert_eq!(pool.with_reader(&live, only_well).unwrap(), "SANDI-1");
        assert!(pool.holds_idle_handle());

        // A swap that bumped the generation but left the handle behind - the state a forgotten
        // `invalidate` cannot produce today and a new swap site could produce tomorrow.
        {
            let mut inner = pool.inner.lock().unwrap();
            inner.generation += 1;
        }
        let replacement = project_with("SANDI-2");
        assert_eq!(
            pool.with_reader(&replacement, only_well).unwrap(),
            "SANDI-2",
            "the stamp is the only thing standing between a forgotten invalidate and a stale answer"
        );
    }

    /// The gate behind `DbState::install`. Stage 1 exists to prove a pooled handle cannot survive
    /// a project swap, and the way that guarantee dies is not a wrong line - it is a MISSING one,
    /// at a swap site written six months from now by somebody who has never read this file.
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

        // Arm 1 - the call count, across every production file in the crate. A second call site
        // anywhere is a second thing to keep in step with the swap, which is the state this design
        // exists to avoid.
        let files: [(&str, &str); 4] = [
            ("lib.rs", include_str!("lib.rs")),
            ("project.rs", include_str!("project.rs")),
            ("db.rs", include_str!("db.rs")),
            ("reader_pool.rs", include_str!("reader_pool.rs")),
        ];
        let calls: Vec<(&str, &str)> = files
            .iter()
            .flat_map(|(name, source)| {
                production(source)
                    .lines()
                    .filter(|line| line.contains(bump.as_str()) && !line.trim_start().starts_with("//"))
                    .map(|line| (*name, line.trim().to_string()))
                    .collect::<Vec<_>>()
            })
            .map(|(name, line)| (name, Box::leak(line.into_boxed_str()) as &str))
            .collect();
        assert_eq!(
            calls.len(),
            1,
            "the pool must be invalidated in exactly one production place; found {calls:?}"
        );
        assert_eq!(calls[0].0, "lib.rs", "the one call belongs in DbState::install");

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
        for (name, source) in [("lib.rs", include_str!("lib.rs")), ("project.rs", include_str!("project.rs"))] {
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
