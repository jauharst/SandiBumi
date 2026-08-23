# #129 stage 2 — reader connections, measured (2026-08-23)

Pass 3 increment 13. Stage 1 built the safety catch with one handle and no concurrency;
`PERF-ATTEMPTS.md` §4 records the first attempt at this stage, which delivered the speed and broke
99 of 100 wells, and the bisect that named why. Route 1 of the three fixes cleared it. This is the
re-attempt, and its measurement.

## 1. What changed

`ReaderPool::read` locks the connection mutex **only to mint a handle**, never for the duration of
the read. The four reads inside the module runner's per-well `rayon` loop go through it:
input resolution and parameters, the run mask, the declared neutron basis, and the zone list behind
the clay-provenance message.

**The write path is untouched.** Every write still takes `db.lock()` and goes through
`ancestry::write_versioned_rows_batch_raw`. DuckDB single-writer is not negotiable and nothing here
questions it — the measurement below shows write thread-time unmoved, which is the check on that
claim rather than the claim itself.

`prewarm` from the first attempt is **not** in this version. It was written to test the hypothesis
that minting handles inside the parallel loop was the fault; the bisect refuted that, so it was
20 lines of unmeasured complexity and it went.

## 2. The measurement

Real 100-well fixture (`SANDIBUMI_FIELD_FIXTURES`, 100 wells x 1,562 samples = 156,200 samples), a
four-module chain, `pipeline_field_test::pipeline_field_100well_stress`, release build, one test
thread, both arms in the same session on the same machine. **The AFTER arm ran first**, so the disk
cache was warmer for BEFORE — the bias points against the change.

```
step                  before        after      ratio
vsh_gr                6.614 s      4.270 s     1.55x
phi_den              15.705 s     10.440 s     1.50x
sw_indo              11.976 s      9.357 s     1.28x
perm_wyllie_rose      4.270 s      3.658 s     1.17x   <- AT the noise floor
CHAIN TOTAL          38.565 s     27.725 s     1.39x
```

`perm_wyllie_rose` is inside `PERF-VARIANCE-2026-08-23.md`'s measured **1.16x** floor for this
machine, so it is **not established on its own**; it is reported because dropping a step that did
not clear the floor would flatter the total. The chain total clears the floor comfortably.

### The queue, which is the whole mechanism

`lock_probe`'s counters SUM ACROSS THREADS, so `wait` is contention rather than wall clock.

```
                     before        after
wait  (contention)  357,347 ms      116 ms
read  (doing work)   13,718 ms   92,713 ms
write (serialized)   16,090 ms   16,358 ms
```

Three readings, and each one matters:

- **`wait` collapsed to nothing.** That is the queue being deleted, which is what the pool is for.
- **`read` went UP 6.8x, and that is the proof the reads overlap.** Under one shared connection a
  read counter can never exceed wall clock, because only one thread can be reading. 92,713 ms of
  read over a 27,725 ms chain is **~3.3 threads busy reading on average**.
- **`write` did not move** (1.02x, inside the floor). The single-writer discipline is intact, and
  this is the number that says so.

### Nothing moved

Identical in both arms, at every step: **468,600 / 624,800 / 624,800 / 312,400** curve rows and
**200 / 89,800 / 90,200 / 90,300** degradation rows. **0 errors of 100 wells** on all four steps in
both arms.

`pipeline_field_full_run` was run on both arms as well, and every value it prints is unchanged -
the three validation means and all four wells' net, N/G, PHIE, SWE and HPV. **0 rows differing.**
That is the check the brief's hard rule asks for: a speed change must not move a number.

## 3. The model said 1.95x. The machine says 1.39x.

`PERF-POOL-RISK-2026-08-23.md` §1 modelled **1.95x at 4 readers, 2.15x at 8** by treating the
measured read cost as fixed and deleting the queue around it. That is not what happens.

**Reading is not free once it runs concurrently.** 13.7 s of serialized read thread-time became
92.7 s of concurrent read thread-time for the same work: eight readers competing for the same
memory bandwidth and the same DuckDB internals make each individual read slower. The win is real —
92.7 s spread over ~3.3 effective threads beats 13.7 s plus 357 s of queueing — but "delete the
queue and keep everything else" was never available.

**So do not expect more from raising the capacity.** The pool caps at 8 idle handles and the
machine offers 32 rayon threads, yet the measured concurrency is ~3.3. The limit is not the number
of handles. Raising `MAX_IDLE` would cost per-connection DuckDB buffers on a field machine and buy
approximately nothing; if somebody wants to try it anyway, the number to beat is 1.39x on this
fixture, and it must be a PAIRED measurement.

## 4. Where the remaining time is

After this, the same chain spends **16.4 s of thread-time writing** against 27.7 s of wall clock.
The write is now the largest single serialized cost in a batch run, and `PERF-POOL-RISK` §4 already
recommends AGAINST a writer pool by name: there is no modelled return, and it would split the one
transaction that covers a chain step.

`pay_summary` (7.17 s -> 7.55 s, 1.05x, inside the floor) is **not** on the pooled path. It is the
Field Dashboard's operation, and `PERF-DASHBOARD-2026-08-23.md` is where that thread continues.

## 5. Safety, and what is still true

The corruption reasoning is `PERF-POOL-RISK-2026-08-23.md` §3 (M1-M8) and has not changed. The two
things stage 2 specifically depends on:

- **A swap cannot begin while a job runs.** `open_project`, `new_project` and `compact_project` each
  refuse with `chain::any_active(..) || jobs::any_active(..)`; pooled reads in the runner happen
  only inside a job; the startup swap predates every job.
- **And that guard is a check followed by an action**, so `read` re-checks the generation stamp when
  it finishes and returns an ERROR rather than an answer if the project moved underneath. A chain
  step that fails and says so is the outcome to want; the alternative is an interpretation computed
  from one project and written into another, which nothing downstream could detect.

Pinned by `reader_pool.rs`'s own tests, including
`a_read_whose_project_changed_underneath_it_returns_an_error_not_an_answer` and the source gate
`a_swap_of_the_live_connection_cannot_be_written_without_invalidating_the_pool`.
