# Performance attempt ledger

Every optimisation attempted, **kept and reverted alike**, so a dead idea stays dead and nobody —
including a future session — re-runs an experiment that already failed. Reverted work leaves no
trace in git history, which is exactly why the same idea gets tried again next quarter.

Read this before proposing a performance change.

| # | Idea | Baseline → Result | Verdict | Why |
|---|---|---|---|---|
| 1 | Index `computed_curves(well_id)` to stop a one-well report scanning the project | report render 52.3 ms → **56.3 ms**; chain total 55.9 s → 60.9 s | **REVERTED** | The read did not improve at all, which refutes the premise. See §1. |
| 2 | `LIMIT 2` on the set-id scan that resolves a well's PHIE log set | 1973.0 ms → **2055.7 ms** across 500 wells | **REFUTED BEFORE WRITING** | No win at 10, 100 or 500 wells, and the cheaper form had the colder cache. See §2. |
| 3 | Speed up the `standard_curves` → generic-store backfill, which is 96.8% of `first project open (COLD)` | same field built both ways at 500 wells: harness-written 39.5 s → imported **174.7 ms** | **NOT WORTH DOING** | A project built by importing wells never runs it. The harness's fixture does, because it bypasses `ingest`. See §3. |
| 4 | Speed up `phi_den` on real data by improving CURVE INPUT RESOLUTION, the mechanism `PERF-FIELD-FIXTURE-2026-08-23.md` §7 proposed | read phase 5.832 s generated -> **5.777 s** real (**0.99x**) while the write went 11.4 s -> 45.7 s | **REFUTED BY MEASUREMENT** | The read is identical on both fixtures. `RHOB` - the only curve `phi_den` asks for - is populated in the standard column on BOTH, so neither side takes the fallback path. The penalty is 89,600 degradation INSERTs. See `PERF-PHI-DEN-2026-08-23.md`. |

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

> **The absolutes below are inflated ~1.7x and are not comparable to any other document.**
> This experiment ran with the filter `perf_baseline`, which by then also selected the two
> other probes in the module and ran all three concurrently - see
> `PERF-VARIANCE-2026-08-23.md` §1. Before and after were contaminated equally, so **the
> comparison and the verdict stand**; the milliseconds do not travel.

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

**Precision reduced 2026-08-23:** report render's measured noise floor is **1.14x**, so the
honest claim is *no improvement larger than 14%* rather than *no improvement*. The verdict is
unchanged - a fix for a full table scan would have been far larger than that.

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

---

## §2 — `LIMIT 2` on the ancestry set-id scan (2026-08-23, pass 3 increment 4)

### What it would have fixed

`perf_dashboard_scale` measured one statement at **35.7% of the whole Field Dashboard** —
`ancestry::try_resolve_ancestry_input`, called once per well to say which log set that well's PHIE
belongs to:

```sql
SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves
 WHERE well_id = ?1 AND upper(curve_name) = ?2
```

It reads all 1,562 of that well's PHIE sample rows and de-duplicates 1,562 copies of one set id.

### Why `LIMIT 2` looked free

The caller checks `set_ids.len() != 1` and errors on anything but one, so **a second row is all it
ever needs to see**. Adding `LIMIT 2` cannot change any answer: one row stays one, two-or-more stays
two-or-more, and the error fires identically.

### Measured, with the bias pointed the right way

Both forms were timed per well, the **limited form first** so the unlimited one ran on the warmer
cache — biasing the comparison *against* the cheaper variant.

```
                   unlimited     LIMIT 2
10 wells              12.1ms      12.6ms
100 wells            157.0ms     176.7ms
500 wells           1973.0ms    2055.7ms
```

### Verdict: REFUTED BEFORE WRITING

No win at any size; the limited form is marginally slower at all three, and it had the easier job.
DuckDB materialises the whole distinct aggregate before a limit can apply, so there is nothing for
the limit to short-circuit — it only trims a result that has already been built.

**Precision reduced 2026-08-23:** that operation's measured noise floor is **1.06×**, so read this
as *no win larger than 6%*. The verdict is unchanged.

**No production code was ever written for this.** The variant was measured in the probe first, which
is the cheapest possible place to kill an idea.

### What this rules out for anyone who tries again

- **Do not put `LIMIT` on a `DISTINCT` in DuckDB expecting it to stop early.** It does not. If a
  scan needs to stop early, the query has to stop being a `DISTINCT`.
- The 36% is still there and still worth taking. Taking it means **not asking the ancestry question
  at all** — which changes which wells appear in the summary, so it is a provenance decision rather
  than a performance one. `PERF-DASHBOARD-2026-08-23.md` §6 sets out the choice.

---

## §3 — Speeding up the cold-open backfill (2026-08-23, pass 3 increment 7)

### The hypothesis anyone would form

`first project open (COLD)` is the second-largest number in the brief — 3.1 minutes at 2000
wells — and it grows faster than linearly (exponent 1.25 over the 500→2000 step). It is also the
first thing a user waits for. That makes it look like the obvious next target.

### What was measured

The opening sequence is already instrumented, so this took no new probe — only reading the
`[boot]` lines already present in five clean `perf_baseline` transcripts at 100 wells:

```
init_db_resilient                          68.1 ms   <- actually opening the file
migrate_standard_curves_to_generic_store 3391.9 ms
migrate_standard_curves_canonical        1776.2 ms
ten other migrations, together             29.6 ms
```

**96.5–96.9% of the cold open is those two migrations**, reproduced across all five runs.

### Verdict: NOT WORTH DOING

Both are ONE-TIME and both are gated, and **a well that arrived through an import never triggers
either**: `ingest.rs` marks each imported well done inside the import transaction, and the
canonical migration writes its own project-level done flag unconditionally on its first run,
before any well exists. The performance harness builds wells with `db::insert_standard_curves`
and never touches `ingest`, so its fixture is permanently un-migrated — which is the entire
reason the number is large.

Pinned by `ingest::tests::an_imported_well_never_pays_the_legacy_backfill_and_a_directly_written_one_does`,
mutation-verified from both sides.

### And then measured, because a gated code path is easy to reason wrongly about

`perf_baseline_test::perf_cold_open_construction` builds the same field twice — written and
LAS-imported — and opens both, asserting equal `curve_meta` counts so it is the same field:

```
wells    written      imported     ratio
   10     646.8 ms     109.5 ms     5.9x
  100    6066.1 ms     120.5 ms    50.3x
  500   39519.9 ms     174.7 ms   226.2x
```

Scaling exponent over 10 → 500 wells: **1.05 written, 0.12 imported.** An imported project's
FIRST open costs what the warm re-open row costs, at every size measured.

**The first version of this probe was itself wrong**, and in the same direction as the thing it
was measuring: it created the imported project with `db::init_db` rather than
`project::open_and_migrate`, so that project never got the canonical migration's done flag and
still paid 1.909 s of it at 100 wells — reported as 3.3× instead of 50.3×. The finding held; the
size of it did not. A comment at the fixed line records this, because a probe built to catch
"the harness took a route no user takes" had to be caught taking one.

### What this rules out for anyone who tries again

- **Do not optimise this backfill for launch time.** The only project that pays it is one created
  before the generic store existed, opening for the first time on a current build. That wait is
  real, is paid once, and is already announced through `db::boot_note`.
- **Do not read the cold-open row as launch cost.** It is the cost of converting a legacy project.
  `PERF-SCALING-2026-08-22.md` now carries a dated note saying so at the table.
- **"There must be a missing index" is not a good first guess about this store.** The quadratic
  explanation for the 1.25 exponent fails on inspection: `curve_meta` declares
  `UNIQUE (well_id, set_name, mnemonic, run_no)`, whose index is prefixed on `well_id`. That is
  the second such hypothesis in two increments to die on reading the schema.
- Full reasoning: `docs/PERF-COLD-OPEN-2026-08-23.md`.
