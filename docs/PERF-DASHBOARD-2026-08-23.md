# Where the Field Dashboard spends its time — 2026-08-23

> **Read `PERF-VARIANCE-2026-08-23.md` beside this.** It measures how steady the instrument is
> (every heavy operation 1.02×–1.16× between runs), re-adjudicates every claim below against
> that floor, and records a filter defect that inflated some absolutes by ~2.5×.

**Pass 3, increment 4. It measures and changes nothing that affects an answer.**

Pass 2 named the Field Dashboard the **fastest-degrading operation in the whole sweep** — exponent
1.61 over the final segment, 49 s at 2000 wells, and per-well cost rising 5.7 → 24.5 ms as the
field grows. Nothing about a well changes when other wells are added, so that rise is the finding.
This increment attributes it to a named phase, and then to a **named query**.

## 1. How it was measured, and why nothing in the production path was touched

`run_pay_summary`'s per-well loop takes the lock once and makes four reads under it. Every one of
those functions is reachable from inside this crate, so the probe **replays that sequence from
outside** rather than bracketing it in `paysummary.rs`. Unlike increment 2 — where the phases were
interleaved inside one private closure and could only be timed in place — nothing here needed a
`#[cfg(test)]` statement in production code. `paysummary.rs` is untouched by this increment.

`src-tauri/src/perf_baseline_test.rs`, `perf_dashboard_scale`:

```
cargo test --release --lib perf_dashboard_scale -- --ignored --nocapture
```

`SANDIBUMI_PERF_DASH_SIZES` (default `10,100,500`) sets the sweep.

**The caveat, stated rather than buried:** the replay runs after a discarded dashboard pass, so it
and the measured pass see the same warm caches. It cannot be otherwise — a cold measurement of one
phase is a warm measurement of every phase after it. The comparison that matters here is **between
sizes**, which that does not affect.

The probe asserts that every chain step produced an answer and that every well returned a curve
frame, so a timing is never reported for work that did not happen — the failure mode that made
`pipeline_field_100well_stress` print a 59 s chain in which all 400 runs errored.

## 2. The result

Release, 1562 samples per well, four-module chain run first so there is something to summarise:

```
WELLS         WALL      name     alias    curves     zones     READS   compute  PRODUCED
10          56.3ms     3.5ms    16.2ms    30.8ms     3.6ms    54.2ms     2.1ms    30 rows
100        637.8ms    36.8ms   190.3ms   363.8ms    41.0ms   631.9ms     5.9ms   300 rows
500       5532.6ms   211.9ms  2198.3ms  2963.1ms   252.4ms  5625.7ms     0.0ms  1500 rows
```

As shares of the 500-well wall clock:

| phase | what it does | ms | share |
|---|---|---|---|
| `curves` | reads VSH, PHIE, SWE, PERM for the well | 2963.1 | **53.6%** |
| `alias` | works out which log set the well's PHIE belongs to | 2198.3 | **39.7%** |
| `zones` | reads the well's zone tops | 252.4 | 4.6% |
| `well name` | reads the well's name | 211.9 | 3.8% |
| **all reading** | | **5625.7** | **101.7%** |
| the arithmetic | cut-offs, zone sweep, row building | — | **not measurable** |

The reads sum to slightly **more** than the wall clock, because the dashboard ran last and had the
warmest cache. That is the honest reading of "the arithmetic is 0.0 ms": it is not zero, it is
**below what this instrument can separate from noise**. The cut-off classification of 156,200
samples per 100 wells does not register against the cost of fetching them.

## 3. The per-well table, which is the finding

```
PHASE                10       100       500   growth
TOTAL             5.631     6.378    11.065     2.0x
well name         0.354     0.368     0.424     1.2x
alias             1.620     1.903     4.397     2.7x
curves            3.084     3.638     5.926     1.9x
zones             0.361     0.410     0.505     1.4x
  set-id query    1.206     1.570     3.946     3.3x
```

**A well does not change when other wells are added, so anything that rises here is the cause.**
`well name` and `zones` are flat — they read one small row and a handful of tops, and they are not
the problem however the field grows. `alias` and `curves` both rise, and `alias` rises fastest.

## 4. One query is 36% of the Field Dashboard

Inside the `alias` step, one statement accounts for **1973.0 ms of the 2198.3 ms** (89.8% of the
step, 35.7% of the whole operation) — `ancestry::try_resolve_ancestry_input`:

```sql
SELECT DISTINCT CAST(set_id AS VARCHAR) FROM computed_curves
 WHERE well_id = ?1 AND upper(curve_name) = ?2
```

It reads **every one of that well's 1,562 PHIE sample rows** and de-duplicates 1,562 copies of the
same set id, to return one value. The set id is a property of the *curve*; it is stored on every
*sample*.

**And it is the fastest-growing thing measured**: 1.206 → 3.946 ms per well, 3.3× across 50× the
wells, which is a steeper rise than any other phase.

## 5. `LIMIT 2` does not fix it — measured, not assumed

The caller only ever needs to know whether the count is exactly one; it errors on anything else. So
a second row is all it ever has to see, and `LIMIT 2` is semantically free.

It was timed alongside the unlimited form, with the **limited form deliberately run first** so the
unlimited one had the warmer cache — biasing the comparison *against* the cheaper variant, which is
the direction that makes a win believable.

```
                   unlimited     LIMIT 2
10 wells              12.1ms      12.6ms
100 wells            157.0ms     176.7ms
500 wells           1973.0ms    2055.7ms
```

**No win at any size — the limited form is marginally slower.** DuckDB builds the whole distinct
aggregate before a limit can apply, so there is nothing to short-circuit. Logged in
`PERF-ATTEMPTS.md` as attempt 2, refuted **before** any production code was written.

## 6. What could be done, and why it is not a performance decision

`run_pay_summary` calls the alias resolver with a candidate list of exactly one name, `["PHIE"]`,
and keeps only whether it returned something. The full provenance record it builds — set name,
version, rejected candidates — is discarded. **So 36% of the Field Dashboard builds a provenance
record to answer a yes/no question.**

A cheaper existence check would change one behaviour, and it is not a performance question:

- Today, `try_resolve_ancestry_input` **errors** when a well's PHIE does not resolve to exactly one
  live log set (more than one, or a NULL set id from before log sets existed).
  `first_available_input_alias` propagates that with `?`, and `run_pay_summary` turns it into
  `Err(_) => continue`.
- So a well whose PHIE carries no single recorded log set is **dropped from the Field Dashboard,
  with no message**. An existence check that merely asked "are there PHIE rows?" would include it.

**This paragraph is read from the code, not measured** — no fixture in this increment produced such
a well. It is stated as a code reading and flagged for verification, not reported as an observed
behaviour.

Either way the brief's rule applies: *if a speed-up would change any output, stop and ask.* Whether
a well with ambiguous curve ancestry belongs in a field summary — and whether being dropped from one
silently is acceptable — is a petrophysics and provenance decision, not a performance one.

## 7. What the ceiling is, if that decision goes the fast way

Removing the set-id scan entirely takes the 500-well dashboard from 5532.6 ms to about 3559.6 ms —
**1.55×**. On pass 2's 49 s at 2000 wells that is roughly 32 s.

**What is left is not plumbing.** The remaining 54% is `fetch_curve_frame_from_set` reading the
6,248 curve values per well that the summary exists to summarise. That is real work, and it grows
1.9× per well across the sweep for the same reason everything else does — a bigger file is slower to
read out of. There is no second 36% behind this one.

## 8. Two runs, and which numbers are the claim

The probe was run twice, the second time with the set-id split added:

```
PER-WELL          run 1 (10/100/500)        run 2 (10/100/500)
TOTAL          5.514  6.191  13.952      5.631  6.378  11.065
alias          1.645  1.980   5.356      1.620  1.903   4.397
curves         3.225  3.480   6.578      3.084  3.638   5.926
```

They agree on shape and on which phase is worst; they differ by 10–20% on the 500-well absolutes,
and pass 2's own figure (10.4 ms/well) sits inside that spread. **The shares and the growth ratios
are the claim of this document. The absolutes are not**, and no absolute here should be quoted
without its run — the same discipline increment 2 applied to the 55.4 s / 35.9 s pair.

Run 1 also reported the residual "compute" growing 124×, from 0.010 to 1.187 ms per well. Run 2 put
it at zero. **A residual that starts at ten microseconds can grow enormously in ratio while meaning
nothing in seconds**, and two runs disagreeing about it by that much is the tell. It is not reported
as a finding.

## 9. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** Attempt 2 (the `LIMIT 2` variant) is recorded there.

## 10. What was done about it - 2026-08-24

`PERF-DASHBOARD-PARALLEL-2026-08-24.md`. The finding above is that the reads ARE the operation, so
the fix is not a faster query: the four per-well reads now run at the same time through the #129
reader pool, and the summation below them is untouched. Paired on this same probe, AFTER arm first:
**5.50x at 10 wells, 5.24x at 100, 3.21x at 500**.

Two things in this document to re-read with that in mind:

- **The `compute` column now clamps to zero by construction**, not by measurement. It is derived by
  subtracting the SERIAL replay from the wall clock, and the operation is no longer serial. §2's
  reading - that the arithmetic is below what the instrument can separate from noise - was measured
  before that was true and still stands; the column itself no longer means what its name says.
- **§4-§7 are unchanged and still open.** The set-id scan is still there and still cannot be removed
  as a performance change, for the provenance reason §6 gives. It is now that share of a smaller
  number.
