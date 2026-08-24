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
| 5 | Write the 89,600 degradation rows through ONE appender instead of one `INSERT` each, the fix attempt 4 named | real delivery: `phi_den` 61.72 s -> **15.86 s** (3.89x), its write phase 51.14 s -> **5.34 s** (9.58x), chain total 85.86 s -> **41.92 s** (2.05x) | **KEPT** | The first surviving fix in pass 3. Paired A/B on the same machine in the same session; identical row counts in both tables at every step. Generated fixture gains only 1.37x on the step and 1.09x on the chain, which is INSIDE the variance floor - synthetic wells clamp 177 samples where real ones clamp 896. See `PERF-DEGRADATION-BATCH-2026-08-23.md`. |
| 6 | #129 **stage 2** - pooled reader connections on the chain's four read paths, for the modelled 1.95x | the queue really did vanish (`wait` 124,407 ms -> **29 ms**) and **99 of 100 wells then failed** on the real fixture | **REVERTED** | A pooled read returns `TransactionContext Error: Failed to commit: PRIMARY KEY or UNIQUE constraint violation` from a SELECT. **Cause named the same day by bisection**: that read path runs a project-wide back-fill WRITE (`ancestry::try_resolve_ancestry_input` -> `db::migrate_standard_curves_to_generic_store`), and N connections each run the whole thing. Stage 1 (the swap catch) stays. See §4. |
| 7 | Take that back-fill off the read path, so #129 stage 2 can be re-attempted - Jauhar's call of *run it at the open*, which the open was already doing | 8 wells on cloned connections: un-backfilled project **7 of 8 failed -> 0 failed**, `curve_meta` rows written by the reads **8 racing threads -> 0**, opened project **8 of 8 resolved** | **KEPT** | Buys no speed by itself and was not meant to - it unblocks the 1.95x. The lazy repair was unreachable in production (the open runs the back-fill; the LAS import marks its own wells done), so no number moved: the 83 fixtures that relied on it now call the same function explicitly through `db::insert_standard_curves_as_opened_project`. See §4. |
| 8 | #129 **stage 2, second attempt** - the same pooled reader connections, now that the write on the read path is gone | real 100-well chain, paired, AFTER arm run FIRST so the cache favoured BEFORE: **38.57 s -> 27.72 s (1.39x)**; lock contention **357,347 ms -> 116 ms**; 0 errors of 100 on all four steps in both arms | **KEPT** | Every row count identical at every step, and every value `pipeline_field_full_run` prints on real wells is unchanged (0 rows differing). The model said 1.95x and the machine says 1.39x, because reading is not free once it overlaps - 13.7 s of serialized read thread-time became 92.7 s of concurrent read thread-time. See `PERF-POOL-STAGE2-2026-08-23.md`. |
| 9 | Run the Field Dashboard's four per-well reads at the same time, through the #129 pool - `PERF-DASHBOARD` measured them as 101.7% of the operation | real 100-well delivery, paired, AFTER arm run FIRST: **963.97 ms -> 219.57 ms (4.39x)**; synthetic probe 5.50x / 5.24x / 3.21x at 10 / 100 / 500 wells | **KEPT** | The summation itself is untouched - same cut-offs, same zone sweep, same serial write under `db.lock()`. 300 pay rows in both arms, `pipeline_field_full_run` unchanged to the last decimal. The writing pay summary moved only 1.10x, INSIDE the floor, and that is the expected result: it writes three FLAG_* curves per well and the write is deliberately still serial. See `PERF-DASHBOARD-PARALLEL-2026-08-24.md`. |
| 10 | Make the multi-well plot overlay's per-well curve fetch **async AND pooled** - a sync `#[tauri::command]` runs inline in the IPC handler, so `context_fetch_concurrency` = 8 described nothing | **real 100-well delivery: 1035.0 ms -> 358.5 ms (2.89x)**, pooled arm run first. Synthetic sweep, two independent sessions: 100 wells 5.35x and 6.55x, 500 wells 6.56x, 10 wells 3.0-3.4x - so the generated fixture OVERSTATES this by about two, and 2.89x is the claim | **KEPT** | Neither half alone is worth anything and that is measured, not argued - the `N locks` arm IS the async-only shape and lands inside the floor of what it would replace, on both fixtures (1.03x real, 0.93x synthetic). See `PERF-PLOT-OVERLAY-2026-08-24.md`. |
| 11 | That the committed `plot data: ALL wells` row was an OPTIMISTIC instrument because it takes one lock where production takes one per well | same probe, five paired measurements: **1.13x, 1.06x, 1.07x, 0.93x, 0.98x** - two of them below 1.0 | **REFUTED BY THE SAME PROBE** | Lock granularity costs nothing here, in either direction. The committed row is a fair measure of the read however the lock is taken, and the instrument error was somewhere else entirely - the webview thread. See section 7. |
| 12 | Split the chain's WRITE phase, which is now 58% of a real run, to find the overhead worth removing | real 100-well chain: appending rows is **83.7%** of the write (current 42.2%, archive 41.4%); the DELETE is **0.2%**, the pre-transaction checks **1.1%** | **NOTHING TO REMOVE** | The instrument is KEPT and the fix it was meant to find does not exist. Every part is work something depends on. The only removable rows are the archive's, and that is a data-integrity decision. See section 8. |
| 13 | That about 93% of the write was something other than putting rows in the table - my own hypothesis, formed before the instrument existed | it counted HALF the rows (the archive gets every row too), and it was a ratio against a probe that reports **972,945 rows/s in one run and 2,874,000 in the next** | **REFUTED BY THE INSTRUMENT BUILT TO TEST IT** | A ratio against a number that moves 3x between runs cannot support a claim smaller than that. Check an instrument's stability before quoting a ratio against it - the rule `PERF-VARIANCE-2026-08-23.md` exists for. |

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


---

## §4 — #129 stage 2: pooled readers (2026-08-23, pass 3 increment 11)

### The hypothesis

`PERF-POOL-RISK-2026-08-23.md` modelled **1.95x at 4 readers, 2.15x at 8** on a real 100-well chain,
and stage 1 had already built the safety catch (the generation stamp, `DbState::install` as the
only route to a swap). Stage 2 was the concurrency: `ReaderPool::read` stops holding the connection
mutex for the duration of a read, and the runner's four read paths in `workflow.rs` go through it.

### What was measured

**The mechanism works exactly as modelled.** On the real 100-well fixture, `lock_probe`'s WAIT
counter - the queue, 90.9% of a batch run - went from **124,407 ms to 29 ms** on the first chain
step. That is the 1.95x arriving.

**And 99 of the 100 wells failed.**

```
vsh_gr: 99 of 100 wells produced no answer - first: duckdb error:
TransactionContext Error: Failed to commit: PRIMARY KEY or UNIQUE constraint
violation: duplicate key "98feb98e-0d68-4d6e-9b0a-cf047f9b2bf1"
```

A **commit failure, reported out of a read**. Seven `cargo test --lib` tests fail the same way.

### What was ruled out, each by its own experiment

| Hypothesis | Experiment | Result |
|---|---|---|
| It is not concurrency | `RAYON_NUM_THREADS=1` | **passes** - so it is |
| Some other read path | reverted read site 1 (module inputs + parameters), kept 2-4 pooled | **passes** - site 1 is the trigger |
| Reusing a handle across wells is unsound | pool capacity forced to 0, so every read mints a fresh connection and drops it | **still fails** |
| Minting a connection *during* parallel work is unsound (`PERF-DIAGNOSIS` experiment C cloned every handle BEFORE its loop and was clean) | added `prewarm`, minting every handle up front | **still fails** |
| `try_clone` secretly shares one connection | probe: a clone cannot see the original's uncommitted row (0), cannot COMMIT the original's transaction (false), and two handles hold concurrent transactions (both Ok) | **separate connections** - refuted |
| It is an in-memory-database artifact | the real file-backed 100-well fixture | **fails there too, worse** |

### Verdict: REVERTED, and at the time the mechanism was an open question

The failure is reproducible, it is concurrency-dependent, and at this point **it was not
understood**. `CLAUDE.md`'s own standard settles what to do with that: *"probably works is not the
standard for the table that holds every interpretation."* A concurrency change whose failure mode
cannot be explained does not ship, however large the number attached to it.

**One sentence written here was wrong and the bisect below refuted it.** It read: *"none of the
functions on that read path writes anything."* One of them does. That is exactly the shape of
claim reasoning produces and measurement kills - it was true of every function I had read, and
false of the one I had not.

**Stage 1 stays** - the generation stamp, `DbState::install`, and the `list_wells` consumer are
committed and green. It buys no speed and was never meant to; its value is that it made this
discoverable with one variable moving instead of two.

### The bisect - ANSWERED 2026-08-23

The instruction to the next attempt was *"not re-implement this - FIND THE WRITE"*, by running the
runner's own statement sequence on N cloned connections and adding one statement group at a time
until one failed. That was done, and it named one statement.

`workflow.rs::the_only_write_on_the_module_input_read_path_is_the_generic_store_back_fill` -
`#[ignore]`d, prints rather than asserts, 8 wells x 200 samples on a file-backed project:

```
DEPTH 1 resolved_log_args_for_well:              7 of 8 failed - first: duckdb error:
    Constraint Error: Duplicate key "well_id: d28515c7-..." violates primary key constraint.
DEPTH 2 + validate_shale_clay_input_quantities:  0 of 8 failed
DEPTH 3 + validate_neutron_basis_input:          0 of 8 failed
DEPTH 4 + fetch_module_input_logs:               0 of 8 failed
DEPTH 5 + resolve_param_arrays_with_default:     0 of 8 failed
CONFIRM   back-fill run first, then the same concurrent read:   0 of 8 failed
CLEAN-STORE 1..5 (every group, on the migrated project):        0 of 8 failed each
CONTROL   whole sequence serially on the base connection:       0 of 8 failed
```

**The named cause** - `ancestry.rs`, inside `try_resolve_ancestry_input`, reached from
`resolved_log_args_for_well` via `first_available_input_alias`:

```rust
let mut imported = resolve_generic_curve_decision(conn, well_id, &upper, SemanticFamily)?;
if imported.is_none() {
    crate::db::migrate_standard_curves_to_generic_store(conn)?;   // <- a WRITE, on a read path
    imported = resolve_generic_curve_decision(conn, well_id, &upper, SemanticFamily)?;
}
```

A curve missing from the generic store triggers **the whole project's back-fill**, lazily, from
inside what every caller treats as a read. Behind one shared connection it runs once, commits, and
is invisible - which is why `PERF-DIAGNOSIS` experiment C measured 4-8x cleanly on clones: it
called `fetch_curve_frame`, which never reaches this. With N connections, N rayon threads each miss,
each start the same back-fill, and collide on `curve_meta`'s primary key. The duplicate key in the
message is a **well id**, which is what made the error unreadable as a symptom - nothing on the
read path inserts a well, and the back-fill does.

**It is not a live bug today.** The single shared mutex serializes that write, and §3 above already
established the back-fill only has work to do in a project written directly rather than imported.

Three arms exist beyond the walk because a first-failing depth is not a cause on its own: DEPTH's
own depths 2-5 look clean for the wrong reason (depth 1 already ran the back-fill), so CONFIRM
re-runs the identical concurrent read with the back-fill done first, CLEAN-STORE re-walks every
group on the migrated project to show there is no SECOND lazy write further down, and CONTROL runs
the sequence serially to show the fixture is not what is broken.

### The blocker is cleared - route 1, 2026-08-23

Three routes were put to Jauhar and he chose the first: run the back-fill at project open, delete
the lazy call. **It turned out to be half done already** - `project::open_and_migrate` has always
run the back-fill at step 2, and it is the route every production open takes, so route 1 reduced to
deleting the lazy call.

Which raised the obvious question: if the open already does it, what was the lazy call FOR? The
counted answer is nothing. The one production writer of `standard_curves` is the LAS import, which
marks its own wells done in the same transaction (`ingest.rs`); DLIS writes no standard columns;
every other call site in the crate is `#[cfg(test)]`. The repair could only fire in a state
production does not produce.

**83 tests were in exactly that state** and had been relying on the read to repair them. They now
ask for it by name - `db::insert_standard_curves_as_opened_project`, the same function called
explicitly and earlier, so no number moved. Two fixtures had to stay on the raw door because they
write their own generic curves and are therefore IMPORTED projects, where a back-fill invents a
second identity; both showed up as identity failures, which is where an invented identity would.

Measured on 8 wells over cloned connections:

```
                          before            after
un-backfilled project     7 of 8 failed     0 failed, 0 resolved
back-filled project       0 failed          0 failed, 8 of 8 resolved
curve_meta rows written   from 8 threads    0
```

`0 resolved` is the honest half - a read that resolves nothing cannot collide with anything - and
is why the probe now counts resolutions rather than failures alone.

**Stage 2 is unblocked but NOT re-attempted here**, deliberately: the last attempt moved two things
at once and cost a day. One variable at a time.

The 1.95x is reachable now - the queue really did disappear. The reverted stage
2 patch is preserved against `ff7cecf5` in this session's scratchpad and deliberately not in the
repository, because a broken concurrency change sitting in the tree is exactly the thing somebody
re-enables without re-reading this. The probe stays IN the tree, because verifying whichever route
is chosen means running exactly it.


---

## §5 — #129 stage 2, second attempt (2026-08-23, pass 3 increment 13)

The first attempt is §4 above: it delivered the speed and broke 99 of 100 wells, and the bisect
named the cause as a project-wide back-fill WRITE running from inside the module-input READ. Route 1
removed it. This is the same change, re-attempted with one variable moved instead of two.

**It works.** Real 100-well fixture, paired in one session, AFTER arm run FIRST so the disk cache
favoured BEFORE:

```
                      before        after      ratio
CHAIN TOTAL          38.565 s     27.725 s     1.39x
wait  (contention)  357,347 ms      116 ms
read  (doing work)   13,718 ms   92,713 ms     <- reads now overlap
write (serialized)   16,090 ms   16,358 ms     <- untouched, as designed
```

`cargo test --lib`: **1244 passed, 0 failed** - the first attempt failed 7 the same way it failed
the wells. Row counts identical at every step, and `pipeline_field_full_run` on real wells prints
identical values in both arms, **0 rows differing**.

**The honest shortfall: modelled 1.95x, measured 1.39x.** The model treated the read cost as fixed
and deleted the queue around it; in fact 13.7 s of serialized reading became 92.7 s of concurrent
reading, which over a 27.7 s chain is only ~3.3 threads busy. Eight handles and 32 rayon threads do
not produce eight-fold reading, so **raising the capacity is not the lever** - the full argument is
`PERF-POOL-STAGE2-2026-08-23.md` §3.

`prewarm` from the first attempt was DROPPED rather than carried forward. It existed to test the
hypothesis that lazy minting was the fault, the bisect refuted that, and the perf rule that governs
this whole ledger says unmeasured complexity does not ship. Removing it cost nothing to check,
because this increment was being measured anyway.

## §6 - the Field Dashboard's reads, in parallel (2026-08-24, pass 3 increment 14)

§2 above refuted the obvious fix for this operation (`LIMIT 2` on the set-id scan) before any code
was written, and `PERF-DASHBOARD-2026-08-23.md` §6 ruled the real fix - dropping that scan - OUT OF
SCOPE for a performance change, because it would alter which wells appear in the summary. Neither
verdict has moved. This attempt goes at the other 64%.

**It works, and on real wells.** Paired in one session, AFTER arm first so the cache favoured BEFORE:

```
100 REAL wells                        before        after      ratio
Field Dashboard (stats_only)        963.97 ms    219.57 ms     4.39x
pay summary that WRITES flags        7.7189 s     7.0081 s     1.10x   <- inside the floor
four-module chain                   30.2152 s    29.1945 s     1.03x   <- inside the floor

synthetic probe                       before        after      ratio
10 wells                             60.0 ms      10.9 ms      5.50x
100 wells                           864.0 ms     164.8 ms      5.24x
500 wells                          7680.6 ms    2393.1 ms      3.21x
```

**What this ledger entry is really here to record is an instrument mistake that was nearly made.**
The only field-scale pay-summary timing in the repo runs `stats_only: false` - it writes three
FLAG_* curves per well - and it moved 1.10x, inside the noise floor. Reported on its own that reads
as "no measurable change on real data". It is not a null result, it is the WRONG INSTRUMENT: the
dashboard never writes, and at 7 s against the dashboard's 219 ms no read-side change of any size
could have surfaced in it. The `stats_only` timing was added to the same `#[ignore]`d test, ahead
of the writing one so it runs on the colder cache.

Anyone tempted to conclude "the pay summary is write-bound so reads do not matter" should read those
two rows side by side: same code, same wells, same cut-offs, 32x apart.

**Do not expect the same from raising the pool capacity** - `PERF-POOL-STAGE2-2026-08-23.md` §3
measured ~3.3 effective concurrent readers against 8 handles and 32 rayon threads, and that ceiling
is unchanged here. The win came from the reads no longer queueing, not from more of them.

## §7 - the multi-well plot overlay, async AND pooled (2026-08-24, pass 3 increment 15)

`plot data: ALL wells` was the last scaling row in `PERF-SCALING-2026-08-22.md` §2 that nothing
had been done to. Two hypotheses about why it is slow were written down BEFORE the probe, and the
probe was built with an arm for each. **One of them died, which is the reason this section is
worth reading.**

### The one that died: lock granularity

`perf_baseline`'s committed row takes ONE lock and loops every well inside it. Production takes one
per well - `plotCommon.ts::fetchContextLayers` runs `context_fetch_concurrency` = 8 concurrent
`get_curve_data` calls and each took `db.0.lock()` afresh. So the committed row looked like an
optimistic instrument, the same class of error as timing the pay summary that WRITES when the
question was about the one that does not (§6 above).

Five paired measurements across two sessions, one lock against one-per-well on 8 threads:

```
   10 wells    1.13x     1.07x
  100 wells    1.06x     0.93x
  500 wells              0.98x
```

**Two of them below 1.0.** Against a 1.16x floor that is not a small effect, it is no effect in
either direction. Lock granularity costs nothing here, the committed row is a fair measure of the
read, and the hypothesis is withdrawn rather than quietly dropped.

### The one that held: a sync command runs on the webview thread

Read off `tauri-macros-2.6.3/src/command/wrapper.rs`: a default `#[tauri::command]` on a sync fn
compiles through `body_blocking`, which runs the body INLINE in the IPC handler. Only `body_async`
reaches `respond_async_serialized`. So those eight concurrent invokes were handled one after
another on the thread that repaints the window, and `context_fetch_concurrency` = 8 was a product
limit with no effect whatsoever.

**This half is argued from source, not measured** - the IPC leg is not reachable from a unit test,
which `perf_baseline_test`'s own header says of every number in it. What IS measured is that the
fix needs both halves.

### Why it is one change and not two

The probe's `N locks` arm is exactly the async-only shape: 8 real threads, every one queueing on
the connection mutex. It measured **292.4 ms against the 315.9 ms it would replace** - inside the
floor. And pooling alone changes nothing while the eight never overlap.

```
100 wells        serial    async-only     both
REAL           1002.4ms      1035.0ms  358.5ms   -> 2.89x   <- the claim
synthetic A     131.5ms       139.2ms   26.0ms   -> 5.35x
synthetic B     315.9ms       292.4ms   44.6ms   -> 6.55x
```

**The synthetic field overstates this by about two**, which is why the real arm exists: `2c` says a
synthetic project proves scaling and not what a delivery feels like, and attempt 5 above measured
the divergence running the other way (1.37x generated, 3.89x real). Reading a real well is simply
more expensive - 1002 ms serial against 316 ms for the same well count - and concurrency makes a
read simultaneous, never cheaper, so the larger the unavoidable bytes-off-disk share, the less a
pool can remove.

**The absolutes do not travel** - 131.5 ms and 315.9 ms are the same code in two sessions, 2.4x
apart, exactly as §8 of `PERF-DASHBOARD-2026-08-23.md` recorded for the same probe family. The
ratio is the result.

### What this rules out for anyone who tries again

- **Do not expect a win from changing how a read takes the connection mutex.** Once per well and
  once per loop measure the same. If a read path is slow it is not the lock acquisition.
- **Do not read `context_fetch_concurrency`, or any frontend concurrency limit, as evidence that
  work overlaps.** Check whether the command behind it is `async`. A sync one makes the limit
  decorative.
- **Do not convert the remaining 154 sync lock-taking commands as a sweep.** `lib.rs` declares 259
  commands - 89 async, 155 sync-with-lock, 15 sync-without - and most of the 155 are small
  metadata reads where the webview thread cannot matter. No measurement justifies the diff.
- **`grep -c '#\[tauri::command\]' src/lib.rs` overcounts.** It returns 262 against a real 259,
  because three doc comments quote the literal - one of them added by this very change. Count with
  the attribute required to START the line, and assert the parts sum to the total.

## §8 - what is inside the chain's write (2026-08-24, pass 3 increment 16)

Instrument only. After the pooled reads and the degradation batching, the write is the majority of
a real chain and nothing could say what was in it. Five counters now subdivide `WRITE_NS` where the
phases happen, with a sixth reported as a derived REMAINDER.

```
PART                                       ms    share
appending to computed_curves             6101    42.2%
appending to computed_curves_archive     5983    41.4%
commit + class declaration (remainder)   1376     9.5%
degradation records                       794     5.5%
pre-transaction checks                    154     1.1%
the DELETE that keeps a re-run idempotent  36     0.2%
WRITE                                   14444   100.0%     <- 58% of a 24.8 s chain
```

### The fix this was looking for does not exist

Every part is work something depends on. The DELETE that was the leading suspect is **0.2%**. The
per-well depth validation that looked worth moving out of the writer's lock is **1.1%**. The two
appends are the data, and they are 84%.

**The only lever is writing fewer rows, and the only removable rows are the archive's** - 41.4% of
the write, ~24% of the chain. Every sample is written twice by design: once readable, once to the
append-only archive that makes a re-run non-destructive and that a log-set restore reads. Trading
that for speed is a data-integrity decision and goes to Jauhar as a question or not at all.

### How the hypothesis got formed wrong, which is the part worth keeping

It counted the rows in `computed_curves` and ignored the archive, so it credited the write with
half the work it does. Then it divided by the WRITE-COST probe's rate from the same run.

**That probe is not stable enough to divide by.** Same code, two runs: **972,945 rows/s** and
**2,874,000 rows/s**. A 3x-unstable denominator cannot support a claim about a 7x or 15x gap. The
repo already has the rule - `PERF-VARIANCE-2026-08-23.md` measures the instrument before trusting
it, and `PERF-DASHBOARD-2026-08-23.md` section 8 records the same probe family swinging 2.5x - and
this broke it.

The direct split needed no comparison at all.

### What this rules out for anyone who tries again

- **Do not look for overhead in the chain's write.** It was measured part by part and there is
  none: 84% is appending rows and the largest non-append part is the commit.
- **Do not propose removing the DELETE** to speed up writes. It is 0.2% and it is the only thing
  upholding uniqueness on a deliberately PK-less table.
- **Do not move the per-well validation out of the write lock** expecting a win. It is 1.1%.
- **Do not divide by the WRITE-COST probe.** It varies 3x run to run. It is a sanity check that the
  PK-less table beats the PK'd one, which it does in both runs, and nothing finer.
