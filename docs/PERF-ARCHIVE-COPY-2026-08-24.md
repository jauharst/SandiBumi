# The archive copy is the engine's job, not the appender's (2026-08-24)

Attempt 14. The write-split instrument (`docs/PERF-WRITE-SPLIT-2026-08-24.md`) measured the chain's
write at 58% of a run and found 41.4% of that write going into `computed_curves_archive`. That doc
closed by saying the only remaining speed on the chain would cost the archive's guarantee, and
therefore was not a performance decision to make alone.

It turns out there was a third option nobody had costed: keep every guarantee, and change only
**which code moves the bytes**.

## 1. What was actually expensive

Phases 2 and 3 of `ancestry::write_versioned_rows_batch_raw` were the same loop twice. Phase 2
pushed every sample into `computed_curves` through a DuckDB appender; Phase 3 pushed the identical
values into `computed_curves_archive` through a second appender, re-reading them out of the same
Rust vectors and re-crossing the Rust-to-engine boundary one row at a time.

The rows were already in the database when Phase 3 started. Phase 1 DELETEs exactly the
`(well, curve)` pairs about to be written, and Phase 2 refills them - so at the top of Phase 3,
`computed_curves` filtered by Phase 1's own grouping is precisely the set of rows the archive needs,
each already carrying the `set_id` the appender would have written for that well.

So Phase 3 is now one statement per curve-name group:

```sql
INSERT INTO computed_curves_archive (set_id, well_id, depth, curve_name, value)
SELECT set_id, well_id, depth, curve_name, value FROM computed_curves
WHERE well_id IN (...) AND upper(curve_name) IN (...)
```

Same rows, same table, same transaction, same append-only discipline. A missing sample is SQL NULL
in both stores, so it copies across as NULL untouched rather than being reconstructed from a NaN.

## 2. The measurement

Real 100-well delivery, four-module chain, `pipeline_field_test::pipeline_field_100well_stress`,
two runs before and two after, back to back in ONE session so the comparison is paired.

|                      | before r1 | before r2 | after r1 | after r2 | ratio |
|----------------------|-----------|-----------|----------|----------|-------|
| archive phase        | 5,622 ms  | 5,825 ms  | 642 ms   | 637 ms   | **8.9x** |
| current phase        | 5,666 ms  | 5,742 ms  | 6,818 ms | 6,369 ms | 0.87x |
| write total          | 13,418 ms | 13,842 ms | 9,703 ms | 9,071 ms | **1.45x** |
| chain total          | 23.05 s   | 23.53 s   | 19.84 s  | 18.90 s  | **1.20x** |

The before range [23.05, 23.53] and the after range [18.90, 19.84] do not overlap; the slowest
"after" beats the fastest "before" by 3.2 s. The phase measurement is the load-bearing one and is
not close: 5,724 ms mean against 640 ms mean, with a 203 ms spread on one side and 5 ms on the other.

The archive was 41.4% of the write. It is now about 6.8% of it.

## 3. Part of the win is paid back, and the books say so

The current-store phase got roughly 890 ms SLOWER. That is not noise and it should not be filed as
noise: the engine copy reads `computed_curves` immediately after Phase 2 wrote it, which forces
DuckDB to materialize rows it was previously content to leave buffered until commit.

The accounting closes on that reading. The archive gave back 5,084 ms and the current store took
890 ms of it, for a net 4,194 ms against a measured write delta of 4,243 ms - the ~49 ms difference
is the check/delete/degrade/remainder parts drifting between runs. A change whose parts do not sum
to its whole is a change that has not been understood; this one sums.

## 4. Nothing moved

**No guarantee changed.** The archive still receives every sample of every run, is still append-only,
is still what `restore_log_set` reads back and still what a set-qualified curve read resolves against
(`equations::fetch_computed_only_aligned`). Reframe still writes to it directly. The chain's own
step-to-step handoff, which asks the archive whether an earlier step of this run already produced a
curve, is unaffected because the rows land there exactly as before.

**No number moved.** Across the paired runs every count printed by the stress test is identical:
468,600 / 624,800 / 624,800 / 312,400 rows per step, 1,562 samples per curve on well 1, degradation
rows 200 / 89,800 / 90,200 / 90,300, dashboard 300 rows, pay summary 300 rows.

**GATE GREEN**, 1,244 passed / 0 failed / 45 ignored. The tests that would catch a wrong archive all
ran and passed - notably
`ancestry::tests::archive_updates_and_deletes_are_refused_and_restoring_version_one_creates_version_four_without_changing_versions_one_through_three`
(restores a version out of the archive and checks the earlier ones are untouched),
`db::inspector_tests::batched_versioned_write_is_correct_across_wells_and_reruns` (the exact path
changed here, across wells and re-runs),
`db::inspector_tests::input_set_selection_reads_archived_values` (the set-qualified read that goes to
the archive) and
`ancestry::tests::re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run`.

## 5. What this does NOT do

- **It does not reduce the file.** Every sample is still stored twice, and the archive still grows by
  one full copy of the output on every run. The disk question is unchanged and still has no retention
  policy - `delete_log_set` still refuses by name, saying so.
- **It does not make re-runs cheaper than first runs**, because it did not change what gets written,
  only how it gets there.
- **It does not touch the current store's own appender.** Phase 2 still pushes rows from Rust, which
  is correct: those values exist only in memory at that point, so there is nothing for the engine to
  copy from.
- **It is not a licence to route other bulk writes through SQL.** This worked because the source rows
  were already in the database, in the same transaction, identified by a filter the code had already
  computed. Where that is not true, the appender is still the fastest path.

## 6. Why the scan is affordable

`computed_curves` has no index at all - the primary key was dropped deliberately in Phase 9
increment 5 because its uniqueness index cost about 3.4x on every insert - so the `SELECT` is a full
table scan. It is affordable because the batch write is called ONCE PER CHAIN STEP with every well in
it (`workflow.rs`), not once per well: four scans over a growing table across the whole chain, not
four hundred. A caller that batched one well at a time would turn this win into a loss.

## 7. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** This is attempt 14, KEPT.
