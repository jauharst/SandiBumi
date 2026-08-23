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

### What the next attempt should do FIRST

Deal with the lazy back-fill, and only then re-attempt the concurrency. **This is a decision for
Jauhar, not a performance change** - the three obvious routes are not equivalent:

1. **Run the back-fill once at project open** and delete the lazy call. Honest and simple; it moves
   an already-announced one-time cost to where §3's `[boot]` lines already report it. Costs a cold
   open on a legacy project that would otherwise have paid it later.
2. **Make the lazy call idempotent under concurrency.** Smallest diff, but it is a write discipline
   change on `curve_meta`, and `CLAUDE.md` forbids adding upsert paths casually.
3. **Refuse rather than back-fill** when the curve is missing. Cleanest boundary, changes behaviour
   for legacy projects.

The 1.95x is reachable once one of those lands - the queue really did disappear. The reverted stage
2 patch is preserved against `ff7cecf5` in this session's scratchpad and deliberately not in the
repository, because a broken concurrency change sitting in the tree is exactly the thing somebody
re-enables without re-reading this. The probe stays IN the tree, because verifying whichever route
is chosen means running exactly it.
