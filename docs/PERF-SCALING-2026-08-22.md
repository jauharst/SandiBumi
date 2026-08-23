# Scaling to 2000 wells — 2026-08-22

> **Read `PERF-VARIANCE-2026-08-23.md` beside this.** It measures how steady the instrument is
> (every heavy operation 1.02×–1.16× between runs), re-adjudicates every claim below against
> that floor, and records a filter defect that inflated some absolutes by ~2.5×.

**Pass 2 of three.** Pass 1 (`PERF-BASELINE-2026-08-22.md`) built the instrument and measured 10
and 20 wells. This pass builds the generated fixture and takes the same measurements at **10, 20,
100, 500 and 2000 wells** — the largest size that completes.

Everything below is one machine, synthetic data, release profile. **A synthetic project proves
SCALING. It does not prove what SandiBumi feels like on a real delivery** — see §6, which now has
a measured reason for saying so, not just a disclaimer.

## 1. The fixture

`tools/make_example_data.py --stress --wells N [--samples 1562] [--out DIR]`

It shares the committed examples' geology rather than inventing a second one, and reuses
`make_las` unchanged. Three things about it are deliberate:

- **Generated, never committed.** Measured at 1.3 MB per 10 wells, so 2000 wells is ~260 MB of
  LAS. It writes to a temp directory by default, never into the repo.
- **The geology REPEATS down the well** (`fold_depth`). The zone model covers 60 m; a stress well
  is 238 m. Simply extending it would leave everything below 1560 m in flat shale — and that is
  not a neutral simplification, because DuckDB is columnar and a near-constant column compresses
  far better than a varying one. A flat fixture would understate exactly the storage and read cost
  the fixture exists to measure. Folding stacks shale/sand/sand/shale instead, which is also what
  a deltaic section looks like. Verified: 1,562 samples per well, GR spanning 40–114 API, **4 sand
  cycles** per well.
- **Tops travel with it** (`tops_stress.csv`, the same `WELL,TOP,MD` header `make_tops` writes), so
  the generated project can actually be opened and clicked through — a pay summary needs zones.

The committed examples regenerate **byte-identically** after this change; that was checked, not
assumed.

**Why a LAS fixture at all, when the harness builds its own wells?** Because the harness writes
curves straight into a database and skips the import entirely — and import is where the alarming
figure in `SB-CORE-030`'s rationale lives ("540 wells for a 15-minute project open"). It is also
the only form a human can open in the app. §6 shows this distinction turned out to matter more
than expected.

## 2. The scaling table

Median of 3–7 repetitions, release profile. Every run produced its full output — `2000/2000 wells
ok` at every chain step. `exp` is the scaling exponent from 10 to 2000 wells: **1.0 is linear,
0 is free, above 1.0 is worse than linear.**

```
OPERATION                          10       20      100      500     2000    exp
-- FLAT: these do not care how big the project is --------------------------------
project re-open (warm)          102.6    111.1    127.1    130.0    140.8   0.06
curve catalog                     2.1      2.2      2.2      1.8      2.2   0.01
well switch: log view             1.0      1.0      1.3      1.0      1.4   0.06
log scroll 10%                    0.9      0.9      1.1      1.1      1.3   0.07
log zoom 1%                       0.9      0.9      1.0      1.3      1.4   0.07
plot data: 2 curves, 1 well       0.9      0.9      1.0      1.4      1.4   0.08
module vsh_gr, 1 well            50.4     49.8     52.4     53.7     61.4   0.04
-- SCALES WITH THE PROJECT -------------------------------------------------------
first project open (COLD)       589.6   1045.0   5380.6  32808.4 185366.1   1.09
plot data: ALL wells              9.5     18.6    111.1    586.0   3005.6   1.09
chain 1/4 vsh_gr                436.4    860.9   5015.8  26130.0 129879.9   1.08
chain 2/4 phi_den              1491.0   3188.2  17909.3 107623.9 541041.7   1.11
chain 3/4 sw_indo               886.5   1868.0  10084.4  66831.5 519895.1   1.20
chain 4/4 perm_wyllie_rose      277.3    730.9   3522.2  23871.3 189749.1   1.23
field dashboard (pay summary)    57.0    110.1    659.4   5223.9  48966.2   1.28
CHAIN TOTAL (4 modules)        3091.2   6648.0  36531.7 224456.7 1380565.8  1.15
-- SHOULD BE FLAT, IS NOT --------------------------------------------------------
report render, 1 WELL            41.1     40.7     46.2     64.9    165.3   0.26
```

At 2000 wells, in the units a person uses: **first open 3.1 minutes, four-module chain 23.0
minutes, Field Dashboard 49 seconds.** A log view still scrolls in 1.3 ms.

> **Added 2026-08-23 — `first project open (COLD)` is mislabelled and must not be read as launch
> cost.** The `[boot]` breakdown, over five clean runs at 100 wells, puts **96.5–96.9%** of it in
> two ONE-TIME migrations; opening the file itself takes **68 ms**. A project whose wells arrived
> through an import never runs either migration — the harness's fixture does only because
> `build_project` bypasses `ingest`. So this row measures **converting a legacy project**, once,
> and 3.1 minutes is not something a user waits for on launch. Measured by building the same field
> both ways and opening each: at 500 wells, **39.5 s written against 174.7 ms imported**, and the
> imported project's scaling exponent is **0.12** — this row belongs with the FLAT block above,
> not here.
> `PERF-COLD-OPEN-2026-08-23.md` has the attribution and the contract test; `PERF-ATTEMPTS.md` §3
> records why speeding it up is not worth an increment. **The other rows are unaffected.**

## 3. Where it stops being linear — the knee

The knee is not a cliff. It is a **steady worsening that begins between 100 and 500 wells and
accelerates after 500.** Exponent per segment:

```
OPERATION                     10->20   20->100  100->500  500->2000
first project open (COLD)       0.83      1.02      1.12      1.25
CHAIN TOTAL (4 modules)         1.10      1.06      1.13      1.31
field dashboard (pay summary)   0.95      1.11      1.29      1.61
plot data: ALL wells            0.97      1.11      1.03      1.18
report render, 1 WELL          -0.01      0.08      0.21      0.67
```

Every one of them rises monotonically. Said as cost per well, which is the same fact without the
logarithms:

```
PER-WELL COST (ms/well)         10       20      100      500     2000
first project open (COLD)     59.0     52.2     53.8     65.6     92.7
CHAIN TOTAL (4 modules)      309.1    332.4    365.3    448.9    690.3
field dashboard (pay summary)  5.7      5.5      6.6     10.4     24.5
```

A well costs **2.2× more to interpret** in a 2000-well project than in a 10-well one, and **4.3×
more to summarise**. Nothing about the well changed.

**The Field Dashboard is the worst row in the table** (1.61 in the final segment) and it is the
one a user sits and waits for.

## 4. Findings, ranked by measured cost

1. **A four-module chain over 2000 wells takes 23 minutes**, and the per-well cost rises with
   project size rather than staying flat. This is Pass 1's finding extended: the wells are handed
   to 32 cores and behave as though handed to one. `SB-CORE-032` (PRESENT-DIVERGENT) predicts
   exactly this, and #129 (connection pool) is the open item against it. **Not touched here.**
2. **First project open is 3.1 minutes at 2000 wells and superlinear (1.25).** The *warm re-open*
   is 140 ms and essentially flat, so this is one-time-per-launch cost, not per-click cost — but
   it is the first thing anyone experiences.
3. **The Field Dashboard is the fastest-degrading operation measured** (exponent 1.61 over the last
   segment; 49 s at 2000 wells).
4. **Rendering a report for ONE well costs 4× more in a 2000-well project than in a 10-well one**
   (41.1 → 165.3 ms). It is small in absolute terms and it is the *shape* that is wrong: a
   single-well deliverable should not know how many other wells exist. Cheapest of the four to
   investigate and the most likely to be a single unfiltered query.

Everything a user clicks — scroll, zoom, well switch, opening a plot on one well — is **flat to
2000 wells**. The interactive surfaces are not the problem. `SB-CORE-034` ("interactive surfaces
stay responsive at portfolio scale", P2, PRESENT-DIVERGENT) now has evidence on the *responsive*
side; the divergence, if it remains, is not in these operations.

## 5. Two corrections to Pass 1

**The row labelled "cold project open" was reporting a WARM re-open.** `bench` times every
repetition with no warm-up, so only the first open pays for a cold file cache — and the median of
three is one of the warm ones. At 500 wells that was a 130 ms median beside a 32.8 s maximum. The
harness now reports `first project open (COLD)` and `project re-open (warm)` as separate rows,
because they are separate operations and the interesting one was hiding in the MAX column.

**The Pass 1 extrapolation to 2000 wells was wrong, and wrong in the optimistic direction.** It
projected ~11 minutes for the chain by extending a straight line from 10 and 20 wells. The measured
answer is **23.0 minutes** — a factor of two — because the curve is not straight. The prediction
recorded in the Pass 1 PR ("I expect no knee... I expect the finding to be that there is nothing to
find there") was **wrong**: there is a knee, it starts around 100 wells, and it steepens.

## 6. Why a synthetic number is still not Jauhar's number

This is normally a disclaimer. Here it is a measurement.

Running the *same* `pipeline_field_100well_stress` over 100 wells two ways:

| | pay summary, 100 wells | rows |
|---|---|---|
| harness-built wells (curves written directly) | **659 ms** | 300 |
| LAS-imported wells (curves via the import path) | **6,590 ms** | 300 |

Same operation, same row count, **10× the time**, differing only in how the wells got there. The
mechanism is not established here — the two paths differ in which store the curves land in and how
they are resolved — but the consequence is: **every number in §2 is a lower bound for a project
that was imported rather than generated**, on top of already being a backend-only lower bound.

So a real 2000-well delivery is slower than 23 minutes, not faster. How much slower is his to
measure on his own data.

> **Added 2026-08-23 — this 10× reproduced, and it is narrower than the sentence above implies.**
> `PERF-FIELD-FIXTURE-2026-08-23.md` measured the pay summary at **7,682 ms** on a third fixture,
> 1.17× from the 6,590 ms here, so the pay-summary finding is solid. But the **chain** had never
> been measured on real curve data, and now it has been: **2.37×**, not 10×, with only `phi_den`
> badly affected. "Every number in §2 is a lower bound" is still true; reading 10× as the size of
> that bound for the 23-minute chain figure overstates it about fourfold.

> **Withdrawn 2026-08-23 — the 10× was the HARNESS, not the data.** The table above compares
> `perf_baseline`'s pay summary (659 ms) with `pipeline_field_100well_stress`'s (6,590 ms), which
> are two different harnesses. Run inside ONE harness with only the source data changed, the pay
> summary measures **6,032 ms on generated wells against 6,041 ms on the real delivery — 1.00×**.
> So the ~9× lives between the two harnesses on identical synthetic data and none of it is
> attributable to real data. The "reproduced at 7,682 ms" note above is consistent with this and
> does not rescue the claim: all three of those figures are the stress harness measuring itself.
> **`phi_den` is the only operation with a genuine real-data penalty** (2.90×), and
> `PERF-PHI-DEN-2026-08-23.md` attributes it — entirely to the write, and specifically to 89,600
> individual degradation INSERTs, not to curve resolution.

## 7. A confirmed diagnosis, and a number withdrawn

**`pipeline_field_100well_stress`'s 400 failures are caused by the fixture's curve naming.** Pass 1
recorded this as an explicitly-unconfirmed hypothesis. Pointing `SANDIBUMI_FIELD_FIXTURES` at
synthetic wells carrying canonical mnemonics, and changing nothing else, took every module from
**100 errors to 0 errors** and the pay summary from **0 rows to 300**. The module runner is not
broken; the test was feeding it curves it could not resolve.

Note the direction: the all-failing run reported **59.4 s** for the chain and the all-succeeding
run **44.3 s**. Failing was *slower* than working, which is why a duration with no output count
beside it is not a measurement.

> **Added 2026-08-23 — right, but one of two causes.** Swapping the fixture cured the symptom and
> so hid a second fault underneath it: the stress run also **cloned its hundred wells from the six
> `standard_curves` columns alone**, with no generic store, so on any delivery whose curves live
> outside those six columns the copies were empty whatever the test asked for. Both are fixed in
> `PERF-FIELD-FIXTURE-2026-08-23.md`, and the run now asserts on failures instead of printing a
> count — which is what let this stand for as long as it did.

**The write-cost probe's "13.2× PK overhead" quoted in Pass 1 is withdrawn as unstable.** The same
probe, same code, reported 840 ms / 63 ms on one run and 48 ms / 13 ms on another — a 17× swing on
a probe that writes its own rows and should not depend on the fixture at all. It ran once each
time. Nothing here rests on it, and it should not be quoted until it is repeated.

## 8. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** Every attempt lands there, kept and reverted alike —
four ledgers in four documents is four places for the same dead idea to go missing from.
