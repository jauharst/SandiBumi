# Performance baseline — 2026-08-22

The first measured answer to "is SandiBumi fast enough?". Before this, the honest answer was that
nobody knew: there was no criterion, no `benches/`, and no stored timings anywhere in the repo.

**Pass 1 of three.** This pass built the instruments and changed nothing. Pass 2 builds the scaling
fixture; Pass 3 fixes only what these numbers name.

## 1. The instruments

| Instrument | Kind | Times |
|---|---|---|
| `src-tauri/src/perf_baseline_test.rs` | `#[ignore]`d test, **release profile** | 14 backend operations |
| `tools/perf-frontend.mjs` | ES module, loaded from the vite dev server on demand | 18 paint operations |

Neither ships in the app and neither adds a runtime dependency, per the brief's own rule that a
profiling harness lives in `tools/` or as an `#[ignore]`d test.

**The backend harness builds its own synthetic project** — `SANDI-*` wells, values from a seeded
LCG over a slow shaliness cycle — so a baseline is reproducible on a fresh clone with no field
fixture configured. Size is env-settable:

```
set SANDIBUMI_PERF_WELLS=20
set SANDIBUMI_PERF_SAMPLES=1562
cargo test --release --lib perf_baseline -- --ignored --nocapture
```

The frontend harness never calls the backend by design — it feeds the draw functions typed arrays
directly, so what it reports is paint cost with the data already in hand:

```js
const h = await import('/tools/perf-frontend.mjs');
console.log(h.table((await h.runAll()).results));
console.log(h.table(await h.runLogView()));
```

### Every row prints what it produced

This is the harness's one non-obvious design rule, and it exists because of §2.

## 2. Why the existing stress test could not be the starting point

`pipeline_field_test.rs::pipeline_field_100well_stress`, release, against the real field fixture:

```
vsh_gr   14.23s (100 errors)  |  phi_den          22.98s (100 errors)
sw_indo  16.30s (100 errors)  |  perm_wyllie_rose  5.86s (100 errors)
chain total 59.38s  ;  pay_summary(100 wells) 83.7ms -> 0 rows
```

400 module runs, 400 failures, zero interpreted samples — and 59 seconds reported as though it were
a chain timing. `grep -c assert` over that function's body returns **0**, so nothing catches it.

A failing module run is not free: it still pays the curve fetch, the ancestry lookup and the error
allocation, and only skips the arithmetic. So the duration looks plausible and measures the error
path. **That is why every row of the new harness prints `10/10 wells ok`, or
`N/M FAILED - first: <message>`, beside the milliseconds.** A timing without an output count is not
a measurement.

The cause of the 400 failures is **not identified**. The file's own header comment records that the
configured fixture delivery carries corrected-channel mnemonics rather than the canonical module
inputs, which would leave the cloned wells' `standard_curves.gr` empty — but that is a hypothesis,
not a finding, and confirming it is separate work.

## 3. Backend — release profile, 1,562 samples per well

Median of 3–7 repetitions. Every run succeeded; that is what the PRODUCED column is for.

```
OPERATION                            10 WELLS   20 WELLS   PRODUCED (20w)
== OPEN ==
cold project open                     102.6ms    111.1ms   20 wells
== READ PATH (the backend half of a click) ==
curve catalog (every plot opens)        2.1ms      2.2ms   6 curves
well switch: log view, 6 curves         1.0ms      1.0ms   6 series
log scroll: 10% depth window            0.9ms      0.9ms   6 series
log zoom: 1% depth window               0.9ms      0.9ms   6 series
plot data: 2 curves, FULL res           0.9ms      0.9ms   3,124 values
plot data: ALL wells, 2 curves          9.5ms     18.6ms   62,480 values
== WRITE PATH (module runs) ==
module vsh_gr, 1 well                  50.4ms     49.8ms   1/1 wells ok
chain 1/4 vsh_gr, all wells           436.4ms    860.9ms   20/20 wells ok
chain 2/4 phi_den, all wells         1491.0ms   3188.2ms   20/20 wells ok
chain 3/4 sw_indo, all wells          886.5ms   1868.0ms   20/20 wells ok
chain 4/4 perm_wyllie_rose, all       277.3ms    730.9ms   20/20 wells ok
== DERIVED VIEWS ==
field dashboard (pay summary)          57.0ms    110.1ms   60 rows
report render, 1 well                  41.1ms     40.7ms   19 pages
```

## 4. Frontend — paint cost

WebGPU: **available** on the reference machine. 1,562 points is one logged well; 156,200 is a
hundred wells on one plot, which is what a multi-well overlay actually puts there.

```
2D PLOTS (canvas draw)          1,562      15,620     156,200
histogram                        1.5ms      22.3ms     284.2ms
crossplot                        1.1ms      12.2ms     204.6ms
pickett                          1.7ms      17.8ms     250.9ms

LOG VIEW (WebGPU, interaction to GPU frame done)
well switch (paint)              4.0ms       7.9ms      34.3ms
log scroll                       3.5ms       3.2ms       4.0ms
log zoom                         3.3ms       3.0ms       3.1ms
```

## 5. What the numbers say

**Nothing a user clicks is slow at field-normal sizes.** A crossplot on one well is ~4 ms of
backend plus ~1 ms of drawing. A well switch in a log view is 1 ms of backend plus 4 ms of paint.
The threshold at which a delay is noticed is ~200 ms; the app sits two orders of magnitude inside
it.

**The log view is flat in well size, and that is the WebGPU design working as intended.** Scroll
and zoom cost 3–4 ms whether the well carries 1,562 samples or 156,200, because geometry is
uploaded once on load and scrolling only moves a transform. The only cost that grows with the well
is the first paint after a well switch (4.0 → 34.3 ms), which is the geometry build.

**The one interaction measured above 200 ms is the multi-well crossplot overlay.** At 100 wells:
~47 ms to read (extrapolated from the measured 18.6 ms at 62,480 values, which is linear) plus
204.6 ms to draw ≈ **250 ms**.

### The finding: a chain does not get faster with more cores

```
              1 well   10 wells   20 wells   per well @20
vsh_gr         50.4ms    436.4ms    860.9ms       43.0ms
```

`workflow.rs` runs the well loop as `req.well_ids.par_iter()` and the harness reports **32 rayon
threads available**. Were those threads doing the work, 20 wells would cost roughly what 1 well
costs. Instead 20 wells cost 20× one well — 43.0 ms per well at 20 wells against 49.8 ms for a
single well run alone. Doubling 10 → 20 wells multiplied the four chain steps by 1.97, 2.14, 2.11
and 2.64. Dead linear.

The compute parallelises; something downstream of it does not. The structural suspect is the single
`Mutex<Connection>` — `CLAUDE.md` records DuckDB single-writer as fundamental — and it is exactly
what open item **#129 (connection pool)** proposes to address.

**This measures the effect, not the cause.** Isolating the serialisation point is the first Pass 3
job, and #129 changes DB connection semantics, so it is not to be implemented without asking.

Extrapolating the 4-module chain — 3.09 s at 10 wells, 6.65 s at 20 — projects to **~33 s at 100
wells**, consistent with the 21 s recorded at `ROADMAP.md:398` on real data with different per-well
sample counts, and to roughly 11 minutes at 2,000 wells. Both are extrapolations. Turning them into
measurements is Pass 2.

### Measure in release, always

The same 4-module chain over 10 wells: **16.03 s debug against 3.09 s release**, a factor of 5.2.
A plain `cargo test` is a debug build, so any timing taken that way is five times pessimistic. The
harness prints its own profile on the first line for that reason.

## 6. What these numbers are not

- **The backend figures are the backend half only.** A real click is this plus the Tauri IPC hop
  plus the canvas paint. Every backend number is a **lower bound** on what is felt, and the harness
  prints that caveat itself rather than leaving a reader to assume otherwise.
- **The log-view figures exclude the wait for the next vsync.** A browser pauses `requestAnimationFrame`
  in a hidden tab, so the harness drives `render()` directly and awaits the GPU queue. Add up to one
  frame interval (~16 ms at 60 Hz) to each.
- **This is synthetic data on one machine.** It proves the SHAPE of the scaling. It does not prove
  what the app feels like on a real delivery — that run belongs to whoever holds the data.

## 7. What this answers in the PRD

Three core requirements bear on this work, and one of them **predicted the finding in §5**.

| Requirement | Status before | What Pass 1 changes |
|---|---|---|
| `SB-CORE-031` — A benchmark harness exists and is part of the gate | **ABSENT** | **Half satisfied.** The harness is now committed to the tree instead of living in prose, which is the requirement's stated rationale. The *part of the gate* half is NOT delivered: a pass/fail threshold needs a declared budget, which is `SB-CORE-030`. |
| `SB-CORE-030` — Portfolio-scale target is declared and measured | **UNMEASURED** | Still unmeasured, and correctly so — it is blocked on Jauhar declaring the target (`06_SEQUENCING_AND_GATES.md` §26 decision 5), and the number cannot be invented here. Pass 2 supplies the measurement half once the target exists. |
| `SB-CORE-032` — The compute path does not hold the global lock across long work | **PRESENT-DIVERGENT** | **First evidence.** The requirement reads: *no operation whose duration scales with well count or sample count may hold the global database mutex for its duration.* §5 measures exactly the symptom that divergence predicts — a rayon-parallel loop over 32 threads scaling dead linear in well count. |

`SB-CORE-032` was written from an estimate (*"a 30-well solver run holds it across an inversion
estimated at up to ~800,000 bounded-least-squares calls"*). It now has a measurement behind it
instead of an estimate. That does not prove the mutex is the serialisation point — it proves the
symptom is real and matches the prediction. Confirming the mechanism is Pass 3's first job.

## 8. The attempt ledger

Kept per the `performance-optimization` skill, so a dead idea stays dead and is not re-run. Pass 1
optimized nothing, so it is empty by design; Pass 3 fills it, reverts included.

| Idea | Baseline → Result | Verdict | Why |
|---|---|---|---|
| _(none yet — Pass 1 measures only)_ | | | |
