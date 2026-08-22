# Where the time goes — 2026-08-22

> **Read `PERF-VARIANCE-2026-08-23.md` beside this.** It measures how steady the instrument is
> (every heavy operation 1.02×–1.16× between runs), re-adjudicates every claim below against
> that floor, and records a filter defect that inflated some absolutes by ~2.5×.

**Pass 3, increment 1. It diagnoses and changes nothing.**

Passes 1 and 2 established the *symptom*: a chain over N wells costs N times one well, even though
`workflow.rs` hands the wells to `par_iter` and this machine reports 32 rayon threads. They did not
establish the *cause*, and #129 (connection pool) was named as a suspect on structural grounds
only. Fixing on a suspicion is the guessing this brief exists to avoid, so this increment proves
the mechanism first.

## 1. What was suspected, and why

The per-well closure in `run_workflow_module_into` takes the single `Mutex<Connection>` **four
separate times**: once for input resolution (also the largest read — log-arg resolution, two
quantity validations, the curve fetch and the parameter arrays all sit inside one lock scope), once
for the run mask, once for the neutron-basis declaration, and once to write.

`SB-CORE-032` predicts precisely this and is already recorded `PRESENT-DIVERGENT`:

> No operation whose duration scales with well count or sample count may hold the global database
> mutex for its duration.

It was written from an estimate. This is the measurement.

## 2. The experiment

`perf_where_time_goes` in `perf_baseline_test.rs` reads the same six curves for every well **three
ways, changing only how the connection is shared**:

| | |
|---|---|
| **A** | serial, one shared connection — the reference |
| **B** | parallel, one shared connection — **what the app does today** |
| **C** | parallel, one connection per thread — **what #129 would do** |

`Connection::try_clone` gives a second handle on the already-open database, which is the primitive
a pool would be built from — so **C measures what #129 would buy without implementing it**, which
is exactly what the brief asked for. Clones are made outside the timer, because a real pool builds
them once per session rather than per read.

All three variants assert they read an identical sample count, so the comparison is one job three
ways rather than three different jobs. A warm-up pass is discarded: the first read of a freshly
built file pays for a cold OS cache, and charging that to whichever variant ran first would decide
the answer by ordering.

Read-only on purpose. DuckDB is single-writer by design, so writes are legitimately serial; the
read half is the half that *could* be parallel, and therefore the half worth measuring.

## 3. The result

```
wells    A serial   B parallel   C parallel    threads    pool
                      (shared)     (pooled)       buy      buys
   10      11.0ms       10.4ms        3.0ms     1.06x     3.48x
  100     109.2ms      105.7ms       23.9ms     1.03x     4.43x
  500     597.5ms      609.7ms       73.7ms     0.98x     8.27x
```

**Thirty-two threads buy three percent.** At 500 wells they buy *less than nothing* — parallel is
2% slower than serial, because lock contention costs more than the concurrency returns.

**One connection per thread buys 3.5× to 8.3×, and the benefit grows with project size.** Not 32×:
DuckDB parallelises internally and the disk is shared, so per-connection scaling saturates well
below the thread count. **#129 is worth roughly 4–8× on reads, not 32×**, and quoting a thread
count as a speed-up would be the same class of error as extrapolating a straight line through a
bending curve.

## 4. An independent confirmation

If the per-well work were fully serialised, a chain over N wells would cost exactly one well's cost
× N. Comparing the two independently measured rows from Pass 2 — `module vsh_gr, 1 well` and
`chain 1/4 vsh_gr, all wells`:

```
wells   1-well x N    measured chain    ratio
   20       996.0ms          860.9ms     0.86
  100      5240.0ms         5015.8ms     0.96
  500     26850.0ms        26130.0ms     0.97
 2000    122800.0ms       129879.9ms     1.06
```

Within ±14% across a 100× size range, and within 4% over most of it. **A chain behaves as though it
runs one well at a time**, arrived at from entirely different data than §3.

## 5. What this does NOT establish

- **It does not measure how much of a module run is under the lock.** §3 times one specific read
  (`fetch_curve_frame` over `standard_curves`). The production path does considerably more DB work
  inside the same lock scope — log-arg resolution, two quantity validations, a generic-store-aware
  curve fetch, parameter arrays, the mask, the neutron-basis declaration — plus the write.
  §3's figure is therefore a **lower bound on the under-lock time per well**, not a share of it.
- **It therefore does not predict an end-to-end chain speed-up.** Reads parallelise 4–8×; writes do
  not parallelise at all. What a chain would actually gain depends on the read/write split inside a
  module run, which is not measured here. Measuring it needs timing inside `workflow.rs` — that is
  production instrumentation, which this increment deliberately does not add.
- **It is not a design for #129.** A pool has real questions this experiment does not touch:
  transaction scope, who owns the write connection, what happens to the single-writer guarantee,
  and whether `try_clone`'s handles are safe to hold across a migration.

## 6. Recommendation

**#129 is worth doing on this evidence, and it remains Jauhar's call.** The brief says it changes DB
connection semantics, is high risk, and must not be implemented without asking. It measures as
worth 4–8× on the read half and the benefit grows with project size — but the end-to-end figure is
unknown until the read/write split inside a module run is measured, and that is the honest next
step before any design work.

The cheapest independent finding from Pass 2 remains untouched and does not depend on any of this:
**a report render for ONE well grew 41.1 → 165.3 ms across the sweep**, which is almost certainly a
single query missing a well filter.

## 7. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** Every attempt lands there, kept and reverted alike —
four ledgers in four documents is four places for the same dead idea to go missing from.
