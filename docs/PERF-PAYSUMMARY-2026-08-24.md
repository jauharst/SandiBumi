# The pay summary paid a hundred transaction commits to write three curves each (2026-08-24)

Attempts 15 (the instrument) and 16 (the fix). After the chain came down to 19.4 s,
`run_pay_summary` over 100 wells was **5.6 s** and had never been instrumented. It sits behind the
Field Dashboard, the workbook, the deck and every report, so it is waited on far more often than a
chain is.

## 1. Why an instrument, when the harness already reported the answer

The harness runs the same summation twice: `stats_only` (no write) at ~170 ms, and the writing
variant at ~5.6 s. It is tempting to call the difference "the write" and start optimising.

That number is a **subtraction across two calls with different cache states** - the read-only one
runs first, deliberately, on the colder cache. It names the block, not the part. Two hypotheses
formed from code shape had already been refuted this week by the instruments built to test them
(`PERF-ATTEMPTS.md` rows 11 and 13), so the parts were timed rather than reasoned about.

`lock_probe` gains five counters over the per-well write block: `PS_LOCK_NS`, `PS_SPEC_NS`,
`PS_ANCESTRY_NS`, `PS_SET_NS`, `PS_ROWS_NS`. The sixth part - the reads and the summation
arithmetic - is a derived REMAINDER and is labelled as one, exactly as the write split's is.
Every call site is a `#[cfg(test)]` binding with a `#[cfg(test)]` drop; `cargo check --release
--lib` passes, which it could not if any reference to a `cfg(test)` item sat outside a guard.

## 2. What it measured

Real 100-well delivery, two runs:

```
PART                                        ms    share   per well
writing the FLAG rows                     3031    54.2%    30.3 ms
creating the log-set version              1236    22.1%    12.4 ms
building the provenance record             722    12.9%     7.2 ms
the three ancestry lookups                 443     7.9%     4.4 ms
reads + summation (remainder)              175     3.1%        -
waiting for the connection mutex             0     0.0%     0.0 ms
TOTAL                                     5605   100.0%
```

Two things this settles that a code reading would have got wrong:

- **The lock is exactly zero.** The loop is serial, so nothing queues behind anything. This is not
  the Field Dashboard's contention problem in another costume.
- **The petrophysics is 3%.** Classifying every sample against four cut-offs and summing net, N/G,
  PHIE, SWE and HPV over every zone costs 175 ms across a hundred wells. It was never the cost.

**And the 54% label is misleading on its own.** 30.3 ms per well writes 4,686 rows - about 155k
rows/s, against roughly 2,000k rows/s for the same table in this harness's own WRITE-COST probe. So
only ~2.3 ms of that 30 was putting rows anywhere. **The rest was the transaction commit**, paid a
hundred times to store three curves at a time. Dividing a share by its row count is what turned
"the row write is expensive" into "the commit is expensive", and they call for different fixes.

It was also a hundred separate archive copies, each scanning `computed_curves` - precisely the
caller shape `PERF-ARCHIVE-COPY-2026-08-24.md` warns turns that win into a loss. (Measured: pay
summary was 5.72 s before that change and 5.73 s after, so it did not regress. It simply never
benefited.)

## 3. The change

Every well's three FLAG curves are collected during the loop and written **once** afterwards
through `write_computed_curves_with_ancestry_batch` - the same batched path a chain step has always
used. One transaction instead of a hundred; one archive copy instead of a hundred.

```
PART                            before    after
writing the FLAG rows             3031     1639
creating the log-set version      1236      656
building the provenance record     722      741
the three ancestry lookups         443      502
reads + summation (remainder)      175      194
TOTAL                             5605     3730   = 1.50x
```

**468,600 FLAG rows across 100 wells** - 100 wells x 3 curves x 1,562 samples, exactly. The harness
now asserts the well count, because a batch that silently dropped wells would look identical to a
speed-up, and a duration with no row count beside it cannot tell the two apart.

## 4. One result is unexplained, and is reported as unexplained

**The log-set step got 1.9x faster without being touched** - same function, same call count, same
per-well loop. The plausible reading is that its small inserts were previously interleaved between a
hundred large transaction commits and were paying for that write-ahead-log churn. That is a
hypothesis, not a measurement. It is recorded here as an observed side effect rather than claimed as
a designed one, and nothing downstream should be built on the explanation until someone measures it.

## 5. Two behaviour notes, neither of them buried

**The run classification is deliberately still absent.** The batched path stamps `outcome_state` on
the log-set version; the per-well path it replaces never did, so PAYFLAG versions have always been
unclassified. Batching naively would have quietly started marking them CLEAN in the catalog - a
catalog change arriving inside a performance change. So `CompleteWellWrite`'s degradation fields
became `Option`, the chain passes `Some` and is classified exactly as before, and the pay summary
passes `None`. Whether PAYFLAG runs *should* be classified is a real question and a separate one.

**The write is now ALL-OR-NOTHING, and that is the one thing that differs.** Before, a failure at
well 50 left wells 1-49 carrying committed flags while the run returned an error - a field left
half-flagged by a summary that reported failure. Now nothing commits unless all of it does. A
partially flagged field is the worse outcome, because it looks complete to every reader downstream,
and this is what a chain step has always done. It is still a behaviour change on the failure path
and is stated as one.

## 6. What is left

Provenance (741 ms) and the three ancestry lookups (502 ms) did not move and are now **33% of what
remains**. Both are READS, done one well at a time, while the write lock is held. Pay summary
already reads its inputs through the parallel reader pool (`PERF-DASHBOARD-2026-08-23.md`); these
two never joined it. That is the shape of a next increment, if it earns one - and it is a hypothesis
about a shape, which is exactly the kind of thing that gets instrumented before it gets fixed.

## 7. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** This is attempt 15 (the instrument, KEPT) and attempt 16
(the batching, KEPT).

---

# Part 2: the provenance reads were an N+1, and pooling them treats the symptom

Attempt 17. Section 6 above named provenance (741 ms) and the three ancestry lookups (502 ms) as
33% of what remained, both READS taken one well at a time under the write lock, in a function that
already prefetches its INPUTS through the pool. That is what this does.

## 8. The change

`run_pay_summary` now works in four stages instead of one loop:

1. the summation loop, which no longer touches the project at all - it collects a `PendingFlagRun`
   per well carrying the rows to write and the three things a provenance record is built from;
2. **every well's provenance record and already-current check at once, through the pool** - reads;
3. log-set version allocation, still serial under the one connection, because it is a WRITE and
   allocating versions concurrently is how two wells claim the same one;
4. the single batched row write from part 1.

```
PART                              part 1    part 2
provenance (summed over threads)     741      6260
ancestry-check (summed)              502      3543
  -> those two, WALL-CLOCK          1243      ~305
creating the log-set version         656       574
writing the FLAG rows               1639      1484
TOTAL ELAPSED                       3730      2557   = 1.46x
```

468,600 FLAG rows across 100 wells in both runs. Diffed against part 1's harness output ignoring
timings, the only differences are the row-count line this work added and the WRITE-COST probe's own
unstable ratio (ledger row 13).

**`PS_SPEC_NS` and `PS_ANCESTRY_NS` are now summed across pool threads**, so they exceed elapsed and
push the derived `rest` remainder NEGATIVE. That is the signature of successful overlap, not a bug,
and the harness says so on its own line - it must never be compared against elapsed, only against
its own previous SUM.

## 9. The part to be honest about: this spreads the work, it does not reduce it

Provenance went from **741 ms of serial work to 6,260 ms of thread time** to produce ~305 ms of
wall-clock. Eight threads each running far slower than one thread did alone - they contend, most
likely on DuckDB's buffer manager, since they all scan the same tables at once. Roughly **8x the
total work for 4x the wall-clock**.

The wall-clock is what a user waits for and it is measured twice, so this is kept. But the reason it
is inefficient is the reason a better fix exists and has not been done:

**These are N+1 queries.** One `resolve_ancestry_input` per input curve per well, plus one
`curve_ancestry` per output per well: roughly 700 separate questions for a 100-well field.
Parallelising N+1 queries makes them finish sooner without making them FEWER. The right fix is to
ask **two** questions - one returning every well's input versions, one returning every well's
current curve ancestry - which would cut the work rather than spread it, and would very likely beat
2.56 s on a single core.

That change lives inside `ancestry.rs`, where a wrong answer is a misattributed provenance record
rather than a compile error, so it is a separate and more careful increment. It is recorded here so
the cheaper win cannot be mistaken for the area being finished.

## 10. Boundaries this had to respect

- **The pool gets reads only.** `create_complete_log_set` allocates a version and stays serial under
  the connection mutex - the same boundary `reader_pool.rs` draws for the chain, and the reason the
  write path is deliberately not pooled.
- **An error in the parallel pass FAILS THE RUN and is never folded into a skip.** The input
  prefetch a few lines above may treat a `None` as a silent skip because that is a well with no
  curves; there is no equivalent here, because every well in this list has already been summed and
  is expecting flags. A field quietly missing FLAG curves looks exactly like a field where nothing
  passed the cut-offs.
- **A run of spaces inside a message is indistinguishable from a dropped line continuation.**
  `source_hygiene_tests::a_message_an_operator_reads_never_carries_a_dropped_line_continuation`
  failed the first gate on this work's own harness print, where three spaces were deliberate column
  alignment. The legend moved to its own line rather than the rule gaining an exception - a rule with
  an "I meant it" clause is one somebody has to adjudicate every time.

## 11. The attempt ledger

Attempt 17, KEPT - with the N+1 collapse recorded as the better fix still outstanding.
