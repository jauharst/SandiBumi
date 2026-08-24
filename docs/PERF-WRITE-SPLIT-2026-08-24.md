# The chain's write is 58% of it, and 84% of that is writing rows (2026-08-24)

Pass 3 increment 16. **Instrument only — this changes no behaviour and buys no speed.** It exists
because the write had become the majority of a real chain and nothing could say what was inside it.

After #129's pooled reads and the degradation batching, the queue is gone (`wait` is 120 ms summed
across 400 well-steps, against 357,347 ms before the pool) and the reads overlap. What is left is
the write, and DuckDB is single-writer by design, so that phase is serial no matter what happens
to connection semantics. Before touching a contract that protects the project file, it was worth
knowing whether any of it was avoidable.

## 1. What was measured

`lock_probe` gains five counters that subdivide `WRITE_NS`, timed where the phases happen inside
`ancestry::write_versioned_rows_batch_raw`. A sixth part — the transaction commit plus the per-well
class-curve declaration — is reported as a REMAINDER, derived rather than timed, and labelled as
one, exactly as `COMPUTE` already is.

Same discipline as the existing counters: every call site is a `#[cfg(test)]`-attributed `let`
binding with a `#[cfg(test)]` drop, so a shipped build contains no branch, no atomic and no timer.
**Verified, not assumed**: `cargo check --release --lib` passes, and it could not if any reference
to a `#[cfg(test)] mod` sat outside a `#[cfg(test)]` guard.

## 2. The answer

Real 100-well delivery, four-module chain, `pipeline_field_test::pipeline_field_100well_stress`:

```
PART                                   ms     share of the write
appending to computed_curves         6101              42.2%
appending to computed_curves_archive 5983              41.4%
commit + class declaration (rest)    1376               9.5%
degradation records                   794               5.5%
pre-transaction checks                154               1.1%
the DELETE that makes a re-run idempotent 36             0.2%
WRITE                               14444             100.0%

chain total 24.8 s, of which the write is 14.4 s = 58%
```

**83.7% of the write is genuinely appending rows** — 4,061,200 of them, because every sample is
written twice: once to the readable store and once to the append-only archive.

## 3. The hypothesis this refutes, which was mine

Before the instrument existed I compared the chain's apparent write rate against the WRITE-COST
probe in the same run and concluded that **about 93% of the write was something other than putting
rows in the table** — most likely the DELETE, or per-well transaction overhead, or bookkeeping.

Both halves of that were wrong.

- **It counted half the rows.** `computed_curves` row counts ignore the archive, and the archive
  gets every row. The chain writes 4.06M rows, not 2.03M.
- **The probe it was measured against is not stable enough to support the claim.** The same
  WRITE-COST probe, same code, reports **972,945 rows/s in one run and 2,874,000 in the next** —
  3x apart, which is more than the effect being claimed. A ratio against a number that moves by
  more than the thing it is measuring is not evidence. This should have been checked before the
  comparison was quoted, the way `PERF-VARIANCE-2026-08-23.md` checks everything else.

The direct split makes the comparison unnecessary. The DELETE that was the leading suspect costs
**0.2%**. The per-well validation that looked worth moving out of the writer's lock costs **1.1%**.

## 4. So there is nothing here to remove

Every part of the write is doing work that something depends on:

- **The two appends are the data.** They are 84% of the phase and they are not overhead.
- **The DELETE is what upholds uniqueness** on a deliberately PK-less table (`ROADMAP.md`'s phase 9
  increment 5). It costs 36 ms across the whole chain. It is not worth a thought.
- **The degradation records classify the run** and were already batched (`PERF-ATTEMPTS` attempt 5,
  which took `phi_den` from 61.7 s to 15.9 s). 5.5% is what remains after that fix.
- **The commit is the commit.**

**The only lever that would move this number is writing fewer rows, and the only removable rows are
the archive's** — 41.4% of the write, about 24% of the whole chain. That is not a performance
decision. The archive is what makes a re-run non-destructive and what a log-set restore reads, so
removing or deferring it trades a data-integrity guarantee for speed. It goes to Jauhar as a
question or not at all.

## 5. The variance note, because it is the lesson

This run measured the chain at **24.8 s**; the run an hour earlier measured the same code at
**49.8 s**. The WRITE share was **58%** and **62%**.

The share is the stable claim. The absolutes are not, and neither is any ratio taken against a
figure from a different run. That is the same rule `PERF-DASHBOARD-2026-08-23.md` §8 and
`PERF-VARIANCE-2026-08-23.md` already state, and it is exactly the rule the refuted hypothesis in
§3 broke.

## 6. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** This is attempt 12 (the instrument, KEPT) and
attempt 13 (my own hypothesis, REFUTED by it). The refuted one gets its own row because a
dead idea that is not findable gets re-run.
