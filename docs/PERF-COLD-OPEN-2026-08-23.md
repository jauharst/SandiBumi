# What "first project open" is actually timing — 2026-08-23

**Pass 3, increment 7.** `PERF-SCALING-2026-08-22.md` reports `first project open (COLD)` as the
second-largest number in the whole brief — **3.1 minutes at 2000 wells**, growing faster than
linearly — and it is the one a user feels first, because it happens before anything else can.

**Almost none of it is opening a project.** 96.5–96.9% of it is a one-time conversion that a
project built by importing wells **never runs at all** — and §3 builds the same field both ways to
prove it rather than argue it: at 500 wells, **39.5 s written against 174.7 ms imported**, the same
curves in both.

**Nothing in the shipped application changed.** One `#[cfg(test)]` probe and one named test.

## 1. Where the time goes, step by step

The opening sequence is already instrumented — `diagnostics::boot_step` prints every step to
stderr — so this needs no new measurement, only reading what was already in the transcripts.
Five clean runs of `perf_baseline` at 100 wells × 1562 samples, `--exact`, `--nocapture`:

| step | time |
|---|---:|
| `init_db_resilient` (opening the file) | 68.1 ms |
| **`migrate_standard_curves_to_generic_store`** | **3391.9 ms** |
| **`migrate_standard_curves_canonical`** | **1776.2 ms** |
| the other ten migrations, together | 29.6 ms |
| **total accounted** | **5265.9 ms** |
| `first project open (COLD)` as reported | 5336.6 ms |

Reproduced across all five runs:

| run | cold open | the two migrations | share |
|---|---:|---:|---:|
| 1 | 5336.6 ms | 5168.2 ms | 96.8% |
| 2 | 5266.8 ms | 5103.3 ms | 96.9% |
| 3 | 5330.5 ms | 5142.1 ms | 96.5% |
| 4 | 5377.7 ms | 5211.5 ms | 96.9% |
| 5 | 5312.4 ms | 5142.1 ms | 96.8% |

**Actually opening the file takes 68 ms.** The warm re-open reported in the same runs is ~120 ms.

### A trap that would have produced 8% instead of 96%

The other five transcripts on hand — the contaminated `running 4 tests` runs from
`PERF-VARIANCE-2026-08-23.md` — report the two migrations at only **8.1–9.4%** of a *larger* cold
open. The reason matters: **all four probes in that file share one project file per size.** An
earlier probe had already migrated it, so `perf_baseline`'s "cold" open found the work done.

So the missing `--exact` corrupts **state**, not just CPU time. That is a second, independent
reason for the fix recorded in the variance report, and it is worth knowing before anyone reads a
number out of those transcripts.

## 2. Why a real project does not pay it

Both dominant steps are **one-time**, and each is gated:

- `migrate_standard_curves_to_generic_store` (`db.rs`) processes only wells absent from
  `curve_migration_done`. **`ingest.rs` marks every imported well done inside the import
  transaction** — the comment there says why: the import writes both stores from the same decoded
  columns, so a later backfill would "add random duplicate RAW identities and break reproducible
  ancestry across copied projects."
- `migrate_standard_curves_canonical` (`equations.rs`) returns immediately once
  `project_meta.standard_curves_canonical` is set, and it writes that key **unconditionally** at
  the end of its first run — so a project created empty marks itself done before a single well is
  imported.

**The performance harness bypasses `ingest` entirely.** `perf_baseline_test::build_project` calls
`db::insert_standard_curves` directly, so none of its wells is ever marked, and every cold open of
its fixture pays the full backfill. That is the number in the scaling report.

Pinned by `ingest::tests::an_imported_well_never_pays_the_legacy_backfill_and_a_directly_written_one_does`,
from **both sides** — an implementation that stopped marking wells would still pass the
"imported well is complete" half, and one that marked every well unconditionally would still pass
the "not migrated twice" half. Both mutations were applied and both were caught, each by a
different assertion.

## 3. Measured, not reasoned

§2 is an argument from reading the code. This is the experiment: build **the same field twice**,
once the way the harness does it and once the way a user does it, and open both.
`perf_baseline_test::perf_cold_open_construction` (`#[ignore]`d, release profile) writes N wells
through `db::insert_standard_curves`, writes the *same* N wells out as LAS 2.0 and imports them
with `crate::ingest::import_las_files`, then opens each project through
`crate::project::open_and_migrate` and prints both times. `curve_meta` row counts are asserted
equal at the end, so the two projects genuinely hold the same curves.

| wells | **written** first open | **imported** first open | ratio | the two migrations, written |
|---:|---:|---:|---:|---:|
| 10 | 646.8 ms | **109.5 ms** | 5.9× | 82.2% |
| 100 | 6066.1 ms | **120.5 ms** | 50.3× | 98.1% |
| 500 | 39519.9 ms | **174.7 ms** | 226.2× | 99.7% |

On the imported side the two dominant steps are **gone, not smaller**: the generic-store backfill
runs in 0.8–1.4 ms at every size (every well is already in `curve_migration_done`), and the
canonical migration in 0.5–0.7 ms (the flag was written when `open_and_migrate` ran over the empty
file, before a single well existed).

**The scaling changes character, not just magnitude.** Over 10 → 500 wells the written project's
open scales at **exponent 1.05** and the imported project's at **0.12** — which puts it among the
FLAT rows of the scaling table, beside `project re-open (warm)` at 0.06. And it lands on roughly
the same numbers: an imported project's *first* open reads 109.5 / 120.5 / 174.7 ms against that
warm row's 102.6 / 127.1 / 130.0 ms — indistinguishable at 10 and 100 wells (1.07× and 0.95×,
inside the 1.16× variance floor), and 1.34× at 500, which is outside it and is not claimed as
equality. **For a project whose wells arrived by import there is no cold open worth the name** —
the first one costs about what every later one costs, and neither grows.

At 500 wells the written project's open crossed the ten-second threshold and the application said
so by itself, unprompted, through the machinery `CLAUDE.md` describes under open-path hardening:

> `Opening this project took 39s — one-time storage upgrades ran (the project was backed up first); the next open will be fast`

That is `db::boot_note` doing exactly its job. §4's claim that this case is already announced is
not a design intention being quoted back — it fired in the transcript.

**What a user does pay is the import**, and it is the honest place for the cost to sit: 56.6, 46.0
and 53.2 ms per well at the three sizes — flat per well, so linear in the field — happening while
they watch an import they asked for, not while they wait for a window to appear.

### The mistake this probe made first, kept on the record

Written with `db::init_db` in place of `open_and_migrate` for the imported project, the same probe
reported the imported side still paying **1.909 s** of canonical migration at 100 wells, and a
ratio of 3.3× instead of 50.3×. The fixture had skipped the empty-file open that `new_project` →
`project::switch_project` performs, so its "imported" project had never been flagged. The finding
survived — the generic-store backfill collapsed either way — but the number attached to it was
wrong by a factor of fifteen, in the direction that made the application look worse.

It is the same class of error as §2's: **a harness that builds a project by a route no user takes
measures the route, not the product.** That is now stated in a comment at the line that fixes it,
because the probe exists to catch this mistake and had to be caught making it.

## 4. So what should be optimised here

**Nothing, for a user's own project.** Logged as `PERF-ATTEMPTS.md` §3 so a future session does
not spend an increment making a one-time conversion faster.

The cost is real for exactly one case: **a project created by a version of SandiBumi that predates
the generic store**, opened for the first time on a current build. That is a genuine wait, it is
paid once, and the application already announces it — `db::boot_note` surfaces migrations and any
open over ten seconds to the status line and the process history, which is precisely the design
recorded in `CLAUDE.md` under open-path hardening.

## 5. What this does to the published figure

`first project open (COLD)` stays in the scaling table — it is a real measurement of a real thing —
but it is **the cost of converting a legacy project, not the cost of opening one**, and it must not
be quoted as what a user waits for on launch. A dated note now says so at the table.

The exponent finding (0.83 → 1.02 → 1.12 → 1.25 across the four size steps) is therefore an
observation about **the backfill**, not about opening — and §3 measures the difference rather than
asserting it: **1.05 for the written project, 0.12 for the imported one**, over the same 10 → 500
wells. Opening a project does not scale. Converting one does.

**Why it grows faster than linearly is not established, and the obvious answer is wrong.** The
backfill runs `SELECT COUNT(*) FROM curve_meta WHERE well_id = ?1 AND mnemonic = ?2` once per well
per standard column, which looks like a table scan that grows with the project — quadratic. It is
not: `curve_meta` declares `UNIQUE (well_id, set_name, mnemonic, run_no)`, and that index is
**prefixed on `well_id`**, so the lookup is indexed.

That is the second unindexed-scan hypothesis in two increments to die on reading the schema — the
first was `curve_samples` in `PERF-FIELD-FIXTURE-2026-08-23.md`, which carries
`PRIMARY KEY (curve_id, depth)`. **This store is better indexed than a first glance suggests, and
"there must be a missing index" should stop being anyone's first guess about it.** The
super-linearity stays unexplained, and §4 is why it is not worth explaining.

## 6. Limits

- **Nothing here is measured at 2000 wells.** The five-run step attribution in §1 is at 100 wells;
  the written-vs-imported comparison in §3 is at 10, 100 and 500. The 2000-well row stays where the
  scaling report put it. Extending a 0.12 exponent from 500 to 2000 is an extrapolation, and it is
  offered as one: it predicts an imported project opens in roughly 200 ms there, which nothing in
  this document has measured.
- **"Cold" means DuckDB's first open of that file, not a cold operating-system cache.** Both
  projects are opened moments after being written, so both are equally OS-warm. That was already
  true of the published figure — `build_project` then `bench` — so the comparison is like for like,
  but neither number includes the cost of reading a project off a disk that has not touched it
  since a reboot.
- **`boot_step` measures the step, not the whole open.** 70 ms of the 5336 ms is outside the
  instrumented steps — connection tuning, schema creation, the format stamp. That gap is small and
  is not chased here.
- **The written side's absolutes drift between runs.** §3 reports 39.5 s at 500 wells where the
  scaling table reports 32.8 s — 1.20×, just outside the 1.16× variance floor of
  `PERF-VARIANCE-2026-08-23.md`. Different fixture file, different day. Nothing here rests on the
  written absolute; the ratio is measured inside a single run.
