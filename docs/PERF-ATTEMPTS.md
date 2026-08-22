# Performance attempt ledger

Every optimisation attempted, **kept and reverted alike**, so a dead idea stays dead and nobody —
including a future session — re-runs an experiment that already failed. Reverted work leaves no
trace in git history, which is exactly why the same idea gets tried again next quarter.

Read this before proposing a performance change.

| # | Idea | Baseline → Result | Verdict | Why |
|---|---|---|---|---|
| 1 | Index `computed_curves(well_id)` to stop a one-well report scanning the project | report render 52.3 ms → **56.3 ms**; chain total 55.9 s → 60.9 s | **REVERTED** | The read did not improve at all, which refutes the premise. See §1. |

---

## §1 — Indexing `computed_curves(well_id)` (2026-08-23, pass 3 increment 3)

### The hypothesis

Pass 2 measured a one-well report growing **41.1 ms → 165.3 ms** across 10 → 2000 wells. A report
for one well should not know how many other wells exist, so the obvious cause was a query missing a
well filter.

Reading the code found something better-looking than that. `ancestry::computed_provenance_groups`
*does* filter (`WHERE cc.well_id = ?1`) — but `computed_curves` carries **no index at all**. The
three-column primary key was deliberately dropped (`ROADMAP.md:398`) because its ART uniqueness
index cost ~3.4× on every insert, taking a 100-well chain from 50 s to 21 s, and nothing replaced
it. So every `WHERE well_id = ?` looked like a full table scan of the whole project.

### What was measured

100 wells × 1562 samples, release, same harness before and after:

```
                              BEFORE      AFTER
report render, 1 well         52.3ms     56.3ms      <- the thing being fixed
chain total (4 modules)      55910ms    60879ms
module vsh_gr, 1 well         67.3ms     87.8ms
plot data: ALL wells         119.9ms    160.3ms
```

### Verdict: REVERTED

**The read did not improve.** That is the decisive result: if the report were paying for a full
scan, an index on the scanned column would have moved it. It did not move at all.

The write-side figures look worse, but run-to-run variance on this machine is large — the same
100-well chain has measured 36 s and 56 s in identical conditions — so *those* numbers are not
strong enough to reject on their own. The absent read improvement is, and it is sufficient.

### What the growth actually is

A follow-up probe timed each part of a one-well report at two project sizes:

```
PART                                  10 wells   500 wells   growth
list_zones (1 well)                      0.3ms       0.4ms     1.3x
curve_ancestry_disclosures (1 well)      4.3ms       5.2ms     1.2x
run_pay_summary (1 well)                11.4ms      17.3ms     1.5x
composite rendering (residual)          12.6ms      19.4ms     1.5x
whole report render (1 well)            28.6ms      42.3ms     1.5x
```

**Fifty times the wells for one-and-a-half times the time, spread evenly across every part.** That
is not the signature of a full scan, which would be roughly linear and concentrated in one query.
It is the ordinary cost of reading from a bigger file: more data, colder caches, more row groups to
consider even when DuckDB's zone maps prune most of them.

So there is **no unfiltered query, and nothing to fix**. The report render is 165 ms at 2000 wells;
the shape is mildly wrong and the magnitude is not a problem.

### What this rules out for anyone who tries again

- **Do not add an index to `computed_curves` to speed up reads.** It was measured and it does not
  work. The table's PK-less state is a deliberate write-path decision and the read cost it was
  assumed to create is not there.
- **The probe that produced the part-by-part table was deleted after it answered its question.** The
  numbers above are the evidence; re-deriving them is ten minutes of work if it is ever doubted.

### Where the real cost is instead

The Field Dashboard, not the report. Pass 2 measured it as the **fastest-degrading operation in the
whole sweep** (exponent 1.61 over the final segment; **49 s at 2000 wells**, 5.7 → 24.5 ms per
well). This probe shows why it is the right target: `run_pay_summary` is already the largest single
part of a *one-well* report at 11–17 ms, and the dashboard is that same operation over every well.
