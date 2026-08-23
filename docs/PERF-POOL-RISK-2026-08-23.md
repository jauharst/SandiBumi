# #129, the connection pool — go/no-go assessment, 2026-08-23

The brief says: *"#129 changes DB connection semantics and is HIGH RISK. Measure whether it is worth
doing; do not implement it without asking."* `ROADMAP.md:1267` adds: *"writes must stay
single-writer to protect the WAL/131MB file. Corruption modes must be reasoned explicitly."*

This was that assessment, and Jauhar answered it: **stage 1, then stage 2.** Stage 1 is built and
green. **Stage 2 was built, measured, and REVERTED** - it delivered the modelled speed-up and broke
99 of 100 wells; see M8 below and `PERF-ATTEMPTS.md` §4.

Original framing follows. It is built - see `reader_pool.rs`
and the REVIEW entry of the same date. §3 M1 below carries a correction that building it produced.
Stages 2 and 3, and everything about concurrency, remain unbuilt.

## 1. The earlier number cannot be quoted, and the reason is good news

`PERF-SPLIT-2026-08-23.md` §5 put the pool at **1.76–1.88×**. That was measured *before* the
degradation batching landed, and batching changed the arithmetic the estimate is built on — because
the estimate assumes the write stays serial and only the queue and the reads improve.

Same model, same method (`wall_after = WRITE_sum + READ_sum / N + COMPUTE_sum`), applied to today's
paired runs on the **real delivery**, 100 wells:

| | READ sum | WRITE sum | wall now | pool at 4 readers | pool at 8 readers |
|---|---:|---:|---:|---:|---:|
| **before batching** | 15.80 s | 61.75 s | 85.86 s | 65.98 s → **1.30×** | 64.00 s → **1.34×** |
| **after batching** | 15.92 s | 17.24 s | 41.92 s | 21.49 s → **1.95×** | 19.50 s → **2.15×** |

**Doing the cheap safe fix first made the risky one nearly twice as valuable.** Before batching, a
pool would have bought 1.30×, because 62 seconds of serialized writing sat downstream of it and no
amount of read concurrency touches that. With the writing down to 17 s, the queue is now the
dominant remaining cost and removing it is worth ~2×.

The generated fixture is unchanged at **~1.8×** — its chain barely moved (35.9 s → 35.7 s), because
its wells clamp 177 samples where real ones clamp 896.

**In wall clock: about 20 seconds off a 41.9 s chain over 100 real wells.** The chain's measured
scaling exponent is ~1.1 (`PERF-SCALING-2026-08-22.md`), so it grows slightly faster than the well
count; no figure for a larger project is quoted here because none has been measured.

`COMPUTE_sum` is borrowed from `PERF-SPLIT` (0.265 s at the same 156,200 samples) — it was not
instrumented on the real fixture. At 0.6% of the projected total it changes no conclusion, but it is
borrowed rather than measured and is flagged as such.

**This is a model, not a measurement.** What *was* measured directly is the read half:
`PERF-DIAGNOSIS-2026-08-22.md` experiment C ran one connection per thread and got **4–8× on reads**,
using `Connection::try_clone` — the same primitive a pool would be built from. The end-to-end figure
is that measurement projected through the phase split.

## 2. What a second connection actually does — measured today, not assumed

A temporary probe, run once and removed. Six questions, six answers:

```
Q1  memory_limit seen by the CLONE:                                    512.0 MiB
Q1b after the CLONE set 256MiB, the ORIGINAL sees:                     256.0 MiB
Q2  rows the CLONE sees after the ORIGINAL committed:                  1
Q3  disjoint rows, overlapping transactions:                           all Ok
Q4  delete-then-append on DIFFERENT wells, same table:                 all Ok
Q5  delete-then-append on the SAME well:  TransactionContext Error: Conflict on tuple deletion!
Q6  clone still usable after the original is dropped:                  true
```

Four of these change the risk picture:

- **The memory cap is a property of the DATABASE, not the connection** (Q1, Q1b). A pool does **not**
  multiply `db::tune_connection`'s cap. That was the largest field-machine fear — the 8 GB machine
  that prompted the cap in the first place — and it is eliminated by measurement rather than
  argued away. Per-connection statement and result buffers still sit outside that budget and are
  unquantified.
- **A clone sees committed rows immediately, with no handshake** (Q2). Readers do not need to be
  told a write happened; they need only not be inside an open transaction that predates it.
- **The app's own write discipline does not conflict across wells** (Q4). Delete-then-append on two
  different wells in the same table, transactions overlapping, both commit. So a *writer* pool is
  not automatically unsafe — which is a more interesting answer than expected.
- **The same well conflicts, and DuckDB refuses at the statement** (Q5), loudly and by name. Not
  silent corruption.

## 3. Corruption modes, reasoned explicitly

### M1 — the stale handle after a project swap. **Top risk, and silent.**

Measured (Q6): a cloned connection outlives the handle it was cloned from and keeps answering.
`project.rs` replaces the live connection in place on **Open Project, New Project and Compact
Project**, and `lib.rs` does it a third time at startup, when the real project replaces the
in-memory placeholder the window is built on. (**Save As does not belong on that list** and was
listed here in error: it copies the project to a new file with `db::engine_copy_to` and leaves the
current one open, so there is nothing to invalidate. The startup swap was missing from it.)

A pooled reader created before a swap would go on serving rows from the *old* database file, and
those rows look completely normal.

The concrete scenario: the user runs Compact Project on a bloated field, keeps working, and reads
answers out of the pre-compact file for the rest of the session. Nothing errors. Nothing looks
wrong.

**There is no accidental guard, and this paragraph originally said there was.** It claimed that
`compact_project`'s rename would fail on Windows with a pooled handle open, giving a loud failure
behind the silent one. Building stage 1 tested it: a mutation that bumped the generation but never
released the handle ran Compact Project to completion, rename and all. The claim was wrong and is
withdrawn. Every swap path fails the same silent way, and the generation stamp is the only thing
standing in front of all of them.

**Mitigation, as built (stage 1, 2026-08-23):** the pool is generation-stamped and the stamp is
bumped **inside the same critical section that performs the swap** — and the bump is not a step a
caller can forget, because `DbState::install` is the only production route to replacing the live
connection and it is what performs both. `reader_pool.rs` holds the contract and the gate that
refuses any second route.

### M2 — write atomicity, which is a semantic change and not a performance one

Today `write_versioned_rows_batch_raw` takes `&[WellWrite]` and writes **every well of a chain step
in ONE transaction**. A failure rolls the whole step back. Splitting that across pooled writers
splits the transaction: a failure mid-run would leave some wells written and some not.

That is a change to what a failed run *means*, not to how fast it is — and the PK-less
`computed_curves` uniqueness rests entirely on delete-then-append (`CLAUDE.md`, Phase 9 increment
5). Q4 says per-well writes do not conflict, so this would probably *work*; "probably works" is not
the standard for the table that holds every interpretation.

**And it buys nothing measurable.** The projection in §1 already assumes writes stay serial. A
writer pool is risk with no modelled return.

### M3 — memory. **Eliminated by measurement** (Q1/Q1b), except for unquantified per-connection buffers.

### M4 — a reader holding an open transaction across a write

Q2 shows a clone sees committed rows with no handshake, *provided it is not already inside a
transaction*. A pooled reader that held one open across a chain step would serve pre-write data.
This is a discipline rather than a hazard: pooled readers take no explicit transaction and are
returned to the pool between operations.

### M8 — a pooled read returns a COMMIT failure. **Blocking, and unexplained.** (added 2026-08-23)

Found by building stage 2. With the runner's four read paths on pooled connections, the queue
vanished as modelled (`lock_probe` WAIT **124,407 ms → 29 ms**) and then **99 of 100 wells failed**
on the real fixture with `TransactionContext Error: Failed to commit: PRIMARY KEY or UNIQUE
constraint violation: duplicate key "<uuid>"` — a commit failure reported out of a *read*.

Ruled out by experiment, one at a time: it is concurrency-dependent (serial rayon passes); it is
read site 1 specifically; it is not handle reuse (capacity 0 still fails); it is not lazy minting
(pre-warming still fails); `try_clone` really is a separate connection (measured three ways); and it
is not an in-memory artifact (the file-backed project fails worse). No function on that path writes
anything.

**This is the blocker, and it outranks every number in §1.** The full reproduction and what the next
attempt should do first are in `PERF-ATTEMPTS.md` §4.

### M5 — 203 lock sites, 195 of them in `lib.rs`

Every `db.0.lock()` today means "take THE connection". With a pool each site has to declare read or
write. Declaring a read as a write is harmless (it serializes, as now); declaring a write as a read
is M2. Mechanical, broad, and gated by the compiler only if the pool hands out two *different
types* — which it should, for exactly that reason.

### M6 — `run_readonly_query` stays a security boundary

`SECURITY-REVIEW-2026-08-22.md` establishes that the boundary is the subquery wrap plus DuckDB's own
refusal of data-modifying CTEs, **not** the choice of connection. Running user SQL on a pooled
reader neither strengthens nor weakens it. Worth stating so nobody later "improves" it by relying on
the connection instead.

### M7 — the WAL recovery path

`db::init_db_resilient` moves a corrupt WAL aside at open, before any pool would exist. Connections
on one database instance share one WAL, so this is not a new failure mode — but the recovery path
assumes a single handle at open time and must keep doing so.

## 4. The three options

| | What it is | Worth | Risk |
|---|---|---|---|
| **A** | Pool of READERS, single writer — what `ROADMAP.md` prescribes | **the whole 1.95–2.15×** | M1 (silent), M4, M5-read-half |
| **B** | A + pooled writers | **nothing modelled** | M2 — changes what a failed run means |
| **C** | Do nothing | 0 | 0 |

**B is not recommended and should be ruled out explicitly rather than left open.** Its only argument
was that Q4 shows per-well writes do not conflict, which makes it *feasible*, not *worthwhile*.

## 5. Recommendation

**Superseded 2026-08-23 by M8.** Stage 1 is done. **Stage 2 was attempted and reverted**, and
stages 2-3 are blocked until the commit failure in M8 has a named cause. The 1.95x is still there —
the queue really did disappear — but it is not reachable safely yet.

Original plan follows.

**Do A, in three stages, and only if the swap invalidation is proven first.**

1. **Stage 1 — the pool with exactly ONE reader.** No parallelism, no speed-up, nothing to measure.
   Its entire purpose is to exercise M1 under real use with every other variable held still: build
   the generation stamp, wire it into all four swap paths, and pin it with a test that a pre-swap
   handle is refused. If a stale handle can escape, it escapes here, where nothing else has changed.
2. **Stage 2 — N readers, confined to the chain's read paths in `workflow.rs`**, not to all 195
   `lib.rs` sites. That is where the 15.9 s of reading and the 428 s of summed queue actually live.
   Re-measure against 41.92 s.
3. **Stage 3 — widen only where a measurement names a site**, on the same evidence bar as every
   other attempt in `PERF-ATTEMPTS.md`.

**Sign-off is unchanged and is not mine:** `ROADMAP.md:63` requires a live `tauri dev` run on 100+
real wells before #129 can be marked done.

**And the honest ceiling:** after A, the write is **17.2 s of a 21.5 s chain — 80% of what remains**,
and it is one batched transaction that DuckDB will not parallelise. Anything after this has to make
the write *cheaper*, not more concurrent. A is the last concurrency win available; there is no
second one behind it.
