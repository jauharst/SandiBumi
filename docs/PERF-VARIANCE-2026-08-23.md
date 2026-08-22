# How much of the performance brief is noise — 2026-08-23

**Pass 3, increment 5.** It measures the instrument rather than the application, because a ratio is
only a finding if it is bigger than the wobble underneath it.

The critique that prompted this said the machine showed **25–55% run-to-run variance** and that
several published ratios might therefore be noise. **That claim was wrong, and this is the
measurement that shows it wrong.** It also found a real defect that had nothing to do with variance.

## 1. The defect found on the way — and it is the more important half

The command printed in `PERF-BASELINE-2026-08-22.md`, in `REVIEW.md`, and in the harness's own
doc comment was:

```
cargo test --release --lib perf_baseline -- --ignored --nocapture
```

**That command stopped being correct the moment a second test joined the module.** Cargo's filter is
a SUBSTRING match against the full test path, and `perf_baseline` is a prefix of the module name
`perf_baseline_test` — so it now selects **all four** probes in that file, and cargo runs them
**concurrently**.

Measured: five runs that way produced a 100-well chain of **~83 s** against **~33 s** for the same
work run alone — **2.5× inflated**, from four heavy DuckDB tests fighting for CPU and disk.

**Nothing in the output says so.** No error, no warning, no failed operation — just a plausible table
of plausible numbers, two and a half times too large. Only the extra banner lines betray it.

Fixed at all three sites, which now read:

```
cargo test --release --lib perf_baseline_test::perf_baseline -- --exact --ignored --nocapture
```

`tools/perf-variance.mjs` **refuses** a transcript holding more than one experiment rather than
warning, because the rows look normal and a warning in a wall of cargo output is a warning nobody
reads.

## 2. The floors

Five clean single-test runs, 100 wells × 1562 samples, release, `--exact`, `--test-threads=1`:

```
OPERATION                              MEDIAN        MIN        MAX  NOISE FLOOR
curve catalog (every plot opens)        2.1ms      1.8ms      2.4ms        1.33x
log scroll: 10% depth window            1.0ms      0.9ms      1.1ms        1.22x
well switch: log view, 6 curves         1.1ms      1.0ms      1.2ms        1.20x
chain 4/4 perm_wyllie_rose, all      3109.1ms   2931.3ms   3412.0ms        1.16x
project re-open (warm)                120.4ms    113.7ms    129.9ms        1.14x
report render, 1 well                  46.6ms     41.7ms     47.5ms        1.14x
chain 3/4 sw_indo, all wells         9109.7ms   8819.0ms   9968.0ms        1.13x
log zoom: 1% depth window               1.0ms      0.9ms      1.0ms        1.11x
plot data: 2 curves, FULL res           1.0ms      1.0ms      1.1ms        1.10x
plot data: ALL wells, 2 curves        107.7ms    101.7ms    111.8ms        1.10x
chain 1/4 vsh_gr, all wells          4640.7ms   4431.8ms   4829.4ms        1.09x
module vsh_gr, 1 well                  52.7ms     49.5ms     53.7ms        1.08x
chain 2/4 phi_den, all wells        16523.1ms  15856.4ms  16907.8ms        1.07x
field dashboard (pay summary)         627.1ms    623.3ms    645.1ms        1.03x
first project open (COLD)            5330.5ms   5266.8ms   5377.7ms        1.02x
```

**NOISE FLOOR is max/min across the five runs.** A claimed improvement smaller than an operation's
floor cannot be told apart from changing nothing.

**This machine is steady.** Every heavy operation sits at **1.02× to 1.16×**. The only rows above
1.16× are the three that finish in about a millisecond, where ±0.2 ms is the clock's own resolution
rather than instability — a 1.33× floor on a 2.1 ms operation is 0.6 ms.

## 3. Pass 2's numbers reproduce a day later

The same 100-well column, measured 2026-08-22 and again today on a different branch state:

```
OPERATION                        pass 2      today    ratio
first project open (COLD)       5380.6ms   5330.5ms   0.991x
chain 1/4 vsh_gr                5015.8ms   4640.7ms   0.925x
chain 2/4 phi_den              17909.3ms  16523.1ms   0.923x
chain 3/4 sw_indo              10084.4ms   9109.7ms   0.903x
chain 4/4 perm_wyllie_rose      3522.2ms   3109.1ms   0.883x
field dashboard                  659.4ms    627.1ms   0.951x
report render, 1 well             46.2ms     46.6ms   1.009x
plot data: ALL wells             111.1ms    107.7ms   0.969x
CHAIN TOTAL                    36531.7ms  33382.6ms   0.914x
```

Everything within **12%**, and the chain steps are consistently a few per cent *faster* today rather
than scattered either side — which looks like a small systematic difference between the two build
states, not noise. Either way it is far below any published finding.

## 4. Re-adjudicating every published claim

A ratio measured ACROSS project sizes carries the floor of both endpoints, so the bar is roughly
floor × floor.

| Claim | Ratio claimed | Bar | Verdict |
|---|---|---|---|
| Cold open costs more per well as the field grows (59.0 → 92.7 ms/well) | 1.57× | 1.04× | **SURVIVES** |
| A well costs more to interpret in a big project (309 → 690 ms/well) | 2.23× | 1.35× | **SURVIVES** |
| A well costs more to summarise (5.7 → 24.5 ms/well) | 4.30× | 1.06× | **SURVIVES** |
| A one-well report costs more in a big project (41.1 → 165.3 ms) | 4.02× | 1.30× | **SURVIVES** |
| One connection per thread is 4–8× on the read it timed (597 → 74 ms) | 8.07× | 1.21× | **SURVIVES** |
| The set-id query is ~36% of the Field Dashboard | share, twice | 1.06× | **SURVIVES** |
| Indexing `computed_curves(well_id)` does nothing for reads | 1.08× *worse* | 1.14× | **STANDS, with its precision reduced — see §5** |
| `LIMIT 2` on the set-id scan does nothing | 1.04× *worse* | 1.06× | **STANDS, same reduction** |

**Not one published growth finding is inside the noise.** The critique's suggestion that several
might be was over-cautious, and stating it as measured fact was wrong.

## 5. What the floors DO reduce: the two refutations' precision

Both refuted fixes were rejected on "it did not improve". The floors say how small an improvement
would have been invisible:

- The **index**: report render's floor is **1.14×**. The observed result was 1.08× *worse*. So the
  honest statement is *"the index produced no improvement larger than 14%"*, not *"the index
  produced no improvement"*. The conclusion is unchanged — a fix for a full table scan would have
  been far larger than 14% — but the claim was stated with more precision than it had.
- **`LIMIT 2`**: same shape, at 1.06×.

Also worth recording: **the write-side numbers in `PERF-ATTEMPTS.md` §1 were correctly not used to
reject.** Those chain figures moved 1.09× where the chain floors are 1.07–1.16×, i.e. inside the
noise. Declining to reject on them was right for the right reason.

## 6. The correction I owe, and where the 55 s came from

The critique's "25–55% variance" rested on one pair of numbers: `perf_read_write_split` giving 55.4 s
and 35.9 s for the same 100-well work (1.54×). Everything measured here sits at **1.16× or better**.

So that spread belongs to **one test**, not to the machine. That test is also the only one carrying
`lock_probe` instrumentation — nine `Instant::now()` sites per well, per module. **Whether the
instrumentation causes the spread is not measured here and is not claimed.** What is established is
that the noisiest thing in the brief is a probe, and generalising it to "the machine is unreliable"
was unwarranted.

**`PERF-ATTEMPTS.md` §1's absolutes are separately unusable.** That experiment reported a 100-well
chain of 55.9 s against today's 33.4 s — 1.67×, consistent with the concurrent-test defect of §1,
because it was run with the bad filter while the module already held three tests. Before and after
were contaminated equally, so **the comparison holds and the verdict stands**; the absolute
milliseconds should not be compared to any other document.

## 7. Limits of this measurement — stated, not buried

- **Floors are measured at 100 wells only** and are assumed, not shown, to hold at other sizes. A
  2000-well run does more I/O and sits closer to the memory cap, and could well be less steady.
- **n = 5, and max/min grows with sample size.** Every floor here is a **lower bound** on the true
  spread; a sixth run can only widen it.
- **The 2000-well column is still a single run.** Nothing here changes that, and it carries the
  brief's most-quoted figures.
- One machine, one day, no CPU pinning, ordinary desktop load.

## 8. The tool

`tools/perf-variance.mjs` — parses transcripts, never spawns cargo, so it cannot become a second
harness that disagrees with the first. Three refusals, each pinned by a named test in
`tools/perf-variance.test.mjs` and run by the gate:

- more than one experiment in a transcript (§1) — **and a `== SECTION ==` title is not an
  experiment**, or every clean transcript would be refused;
- runs of different project sizes, whose spread is scale and not variance;
- any run containing a failed operation, which is a stopwatch on a failure.

```
node tools/perf-variance.mjs run1.txt run2.txt run3.txt run4.txt run5.txt
```

## 9. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** Nothing was optimised here.
