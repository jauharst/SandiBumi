# The real-data stress harness — 2026-08-23

**Pass 3, increment 6.** Every number in the performance brief so far comes from wells this
repository generated. The brief said plainly that a synthetic project proves scaling and does not
prove what the application feels like on a real delivery. `pipeline_field_100well_stress` is the
one probe that was supposed to close that gap.

**It was not closing it. It was timing failures and reporting them as timings.**

**Nothing in the shipped application changed.** The only source file touched is
`src-tauri/src/pipeline_field_test.rs`, which `lib.rs` declares under `#[cfg(test)]`, so none of it
is compiled into `sandibumi.exe`.

> **Re-run**, from `src-tauri`, with `SANDIBUMI_FIELD_FIXTURES` pointing at a folder holding
> `las/` and `core/`:
>
> ```
> cargo test --release --lib pipeline_field_test::pipeline_field_100well_stress -- --exact --ignored --nocapture --test-threads=1
> ```
>
> Keep `--exact`. The name above is specific enough that dropping it changes nothing, but the
> shorter filter anyone reaches for — `cargo test pipeline_field` — also selects
> `pipeline_field_full_run` and cargo runs both **at once**, which is the contamination recorded in
> `PERF-VARIANCE-2026-08-23.md`.

## 1. What the test was doing

Measured before any change, on the configured delivery:

```
=== 100-WELL STRESS: built 100 wells × 1562 samples = 156200 samples in 704.3133ms ===
  rayon threads available: 32
  vsh_gr             3.4810735s  (29 wells/s, 100 errors)
  phi_den            5.0804185s  (20 wells/s, 100 errors)
  sw_indo            3.1464822s  (32 wells/s, 100 errors)
  perm_wyllie_rose   1.0076263s  (99 wells/s, 100 errors)
  chain total 12.7156005s → 0.0M sample-evals/s
  pay_summary(100 wells) 14.51ms → 0 rows

test result: ok. 1 passed; 0 failed; ...
```

**It passed.** The failure count was computed, printed, and never asserted on; the pay summary's
`Result` was flattened with `.unwrap_or(0)`, so an `Err` and an empty field rendered identically.
A stopwatch held over work that did not happen reports a duration, and nothing in the output says
the duration is meaningless.

**And the direction of the error is not predictable, which is why it cannot be corrected for.**
That failing chain took **12.7 s**; the working chain below takes **86.7 s** — failing was
**6.8× faster**. `PERF-SCALING-2026-08-22.md` §7 measured the same mistake on a different fixture
and found failing *slower* than working. Same defect, opposite sign. A duration with no output
count beside it is not a measurement in either direction.

Four changes, all in `pipeline_field_test.rs`:

- **The chain asserts, and names the first error.** The count was never the missing part — the
  MESSAGE was. A hundred identical refusals said nothing about why.
- **The pay summary is `expect`ed and asserted non-empty.** An `Err` is now a failed test, not a
  field with no pay in it.
- **A pre-flight check asks the app's own resolver** whether the source well can drive the chain
  at all, before cloning it a hundred times. It walks the chain in order and forgives an input an
  earlier step will write — `VSH` and `PHIE` are absent from any source well by construction —
  using `workflow::resolve_output_names`, the one place output names are resolved. A delivery that
  genuinely cannot drive the chain is a fixture fact, not a code defect, so it **skips by name**;
  a fresh clone pointed at any folder still goes green.
- **The throughput line reads in thousands, not millions.** This chain has never reached
  0.1M sample-evals/s on either fixture, so `{:.1}M` printed `0.0M` every single time — the same
  reading it gave while all 400 runs were failing. A unit that rounds every real answer to zero is
  not a unit.

## 2. Why every run failed: the clone was six columns, not a well

The harness built its hundred wells by reading the source well's six `standard_curves` columns and
writing them onto new well rows. That is a copy of six columns.

On this delivery two of the six arrive filled and two of the ones the chain needs do not:

```
  source well: 1562 samples; finite in the six standard columns:
    GR=0  RES_DEEP=1178  NPHI=0  RHOB=1373  DT=0  SP=0
```

The delivery's resistivity is `DRES` and its density is `RHOB` — both registered aliases, so both
land in their standard column. Its gamma is `GRN_CS` and its neutron `NPHI_COR`, and neither
reaches one.

**There were two independent causes, and fixing either alone would not have been enough:**

1. **The stress run passed no curve bindings at all** (`log_inputs: HashMap::new()`), so every
   module asked for `GR` and got nothing. Its sibling `pipeline_field_full_run` has always used
   `field_log_inputs` to point the modules at `GRN_CS` and `NPHI_COR`, and has always passed.
   `PERF-SCALING-2026-08-22.md` §7 named this cause and was right about it.
2. **The clone had no generic store**, so even with the bindings there would have been nothing to
   read: a mnemonic that is not one of the six standard columns is resolved from `curve_meta` /
   `curve_samples`, and the clone had neither.

The stress run now uses the same `field_log_inputs` the full run uses, and the clone carries
`curve_meta`, `curve_samples` and the well's own depth unit — which is what makes it a copy of the
well rather than of six columns.

**A wrong turn worth recording**, because it is the mistake the fix is designed against: my first
attempt bound the roles by a family map written inside the test. It resolved neither curve — I had
guessed `RES` for a family actually named `RES_DEEP`, and `GRN_CS`/`NPHI_COR` do not resolve by
family at all — and it was a third opinion about a question `curves.rs` already answers. One
answer, called twice, not two answers.

## 3. Your delivery's gamma and neutron do not arrive under their own names

This is the finding worth your attention, and it is about your data rather than about a test.

Counted across the configured folder — 15 LAS files:

| Channel | Files carrying it | What the dictionary makes of it |
|---|---|---|
| `DEPTH` | 15/15 | index |
| `GRN_CS` | **15/15** | **nothing — not a registered alias of any family** |
| `NPHI_COR` | **15/15** | **family `POR`**, beside PHIE and PHIT — not `NPHI` |
| `DRES` | 15/15 | family `RES_DEEP` ✓ |
| `RHOB` | 15/15 | family `RHOB` ✓ |
| `FTEMP` | 15/15 | temperature |
| `CALI` | 14/15 | family `CALI` ✓ |
| `PEF` | 11/15 | family `PEF` ✓ |

Counts from `os.listdir` case-folded on `.las`, and the channel tallies from parsing each file's
`~CURVE` section. The dictionary facts from `grep -c 'GRN_CS' registry/unit-registry.json` → **0**,
and from reading the `POR` family's alias list in the same file.

**The consequence.** A module opened on one of these wells and run with its manifest defaults asks
for `GR` and `NPHI`. Neither resolves — not from the standard column, and not from the generic
store either, because the fallback in `equations::fetch_curve_frame` resolves by mnemonic **or
family**, and `GRN_CS` matches nothing while `NPHI_COR` matches a porosity family. The refusal is
by name, so nothing is silently wrong; but on this delivery every module needs its gamma and
neutron pointed at by hand, every time.

**And the log view reads through the same resolver.** `equations::fetch_track_frames` sends an
unqualified track name through `fetch_curve_frame`, which is the function above — so a track added
as `GR` on one of these wells resolves to nothing and draws nothing. Adding the track as `GRN_CS`
instead, from the Wells pane's set browsing, draws it. **Read from the code, not clicked**: the two
paths are literally the same function, but this consequence has not been demonstrated in a running
app and should be confirmed before it is quoted as observed behaviour.

**This is not fixed here, and that is deliberate.** Both possible fixes are dictionary decisions,
and a dictionary decision moves numbers on every delivery that uses the spelling:

- Adding `GRN_CS` to family `GR` reads `_CS` as a vendor suffix on a gamma curve. Probably right
  for your data. It is a guess about a vendor convention until someone who knows says so.
- Moving `NPHI_COR` from `POR` to `NPHI` is the larger one. `NPHI_COR` most commonly means a
  neutron log with environmental corrections applied — still a neutron measurement. It currently
  sits in the porosity bucket among SandiBumi's own computed output names (`PHIE_SSC`,
  `PHIFF_SSPW`, `PHIE_DN`…), which is where an app-produced curve belongs. If the same spelling is
  both, one of the two readings is wrong for somebody.

The brief's rule is that a speed change must not move a number, and that anything which would is a
petrophysics decision to be asked about rather than made. This is that.

## 4. A smaller one: the core fixtures have never run

`field_fixtures::core_table()` looks for `<root>/core/*.csv`. The configured folder holds one
file, `Core.xlsx`. It has exactly one consumer — `parsers::probe_real_field_core`, counted with
`grep -rn 'field_fixtures::core_table()' src-tauri/src/` → 1 — and that test has therefore been
skipping every time. It skips correctly and by name, but silently as far as anyone reading a green
gate is concerned. Exporting that sheet to CSV is all it needs.

## 5. The numbers

100 wells × 1562 samples, release build, `--exact`, single test, **run twice**. **Zero errors on
all four modules and 300 pay-summary rows both times** — the first time this probe has measured
work rather than failure.

```
=== 100-WELL STRESS: built 100 wells x 1562 samples = 156200 samples in 4.829s ===
  rayon threads available: 32
  vsh_gr             4.546s   (22 wells/s, 0 errors)
  phi_den           65.738s   ( 2 wells/s, 0 errors)
  sw_indo           12.740s   ( 8 wells/s, 0 errors)
  perm_wyllie_rose   4.123s   (24 wells/s, 0 errors)
  chain total       87.147s -> 7.2k sample-evals/s
  pay_summary(100 wells) 6.881s -> 300 rows
```

| operation | run 1 | run 2 | spread |
|---|---:|---:|---:|
| `vsh_gr` | 4182.1 ms | 4546.1 ms | 1.09× |
| `phi_den` | 63029.4 ms | 65738.4 ms | 1.04× |
| `sw_indo` | 15151.4 ms | 12739.6 ms | 1.19× |
| `perm_wyllie_rose` | 4299.8 ms | 4123.0 ms | 1.04× |
| **chain total** | 86662.6 ms | 87147.1 ms | **1.01×** |
| pay summary | 7682.2 ms | 6880.7 ms | 1.12× |

Every row sits inside the noise floors measured in `PERF-VARIANCE-2026-08-23.md` except `sw_indo`
at 1.19× against its 1.13× floor — and with **n = 2** a max/min spread is a weak estimator, so that
is not evidence of anything. The chain total reproduces to **1%**.

### Real wells against the synthetic sweep

The synthetic column is `PERF-SCALING-2026-08-22.md` at **the same 100 wells and the same 1562
samples**, on the same machine, running the same four modules with the same parameters —
`perf_baseline_test::chain_params` says so in its own comment.

The real column is the **mean of the two runs above**; the synthetic column is a single run, which
is how that harness times a chain step. The asymmetry is stated rather than hidden — it makes the
real column the better-estimated of the two, not the worse.

| operation | synthetic | real delivery | ratio |
|---|---:|---:|---:|
| chain 1/4 `vsh_gr` | 5015.8 ms | 4364.1 ms | **0.87×** |
| chain 2/4 `phi_den` | 17909.3 ms | 64383.9 ms | **3.59×** |
| chain 3/4 `sw_indo` | 10084.4 ms | 13945.5 ms | 1.38× |
| chain 4/4 `perm_wyllie_rose` | 3522.2 ms | 4211.4 ms | 1.20× |
| **CHAIN TOTAL** | 36531.7 ms | 86904.9 ms | **2.38×** |
| pay summary | 659.4 ms | 7281.4 ms | 11.04× † |

† The synthetic pay-summary figure is a median of three calls within one process; the real one is
the mean of two calls that were each the first in their own process, so warm-versus-cold sits
inside that ratio. **But it is not an artifact**, because it
reproduces an earlier independent measurement: `PERF-SCALING-2026-08-22.md` §6 timed the same pay
summary at **659 ms on harness-built wells and 6,590 ms on LAS-imported ones**, same 300 rows,
and called it 10×. Today's **7,281 ms** on wells carrying a real generic store lands **1.10×**
from that 6,590 ms. Two different fixtures, two sessions, the same answer — the pay summary really
does cost about ten times more once the curves are in the generic store.

**The finding is `phi_den`.** It is not a uniform slowdown: `vsh_gr` is slightly *faster* on real
wells, and the other two are within 1.4×. Inside the real run `phi_den` costs **15× what `vsh_gr`
costs**, where synthetically it costs 3.6× — so whatever this is, it is specific to that module on
this data.

**The cause is not diagnosed, but the measurements do point somewhere.** Counting each chain
module's declared log inputs from the manifest and setting them beside the ratio:

| module | log inputs | log outputs | synthetic | real (mean) | ratio |
|---|---:|---:|---:|---:|---:|
| `vsh_gr` | 1 | 3 | 5015.8 ms | 4364.1 ms | 0.87× |
| `perm_wyllie_rose` | 1 | 2 | 3522.2 ms | 4211.4 ms | 1.20× |
| `sw_indo` | 3 | 4 | 10084.4 ms | 13945.5 ms | 1.38× |
| `phi_den` | 7 | 4 | 17909.3 ms | 64383.9 ms | 3.59× |

**The penalty rises monotonically with the number of declared inputs, and the output count does
not track it at all.** That is a correlation across **four points**, not a cause — but it is
consistent with input *resolution* being what costs, which is exactly what differs between the two
fixtures: a synthetic well answers `RHOB` from a `standard_curves` column, a real one answers it
after the resolver has been down the computed-then-generic path. Five of `phi_den`'s seven inputs
are optional conditioning flags (`BADHOLE`, `GAS_FLAG`, `COAL_FLAG`, `TIGHT_FLAG`, `COND_FLAG`)
that do not exist on these wells, so each is resolved, found absent, and paid for.

**The obvious mechanism does not survive reading the schema**, which is why this stops at
correlation: an unindexed scan of the generic store would explain it, and `curve_samples` carries
`PRIMARY KEY (curve_id, depth)`, so those lookups are indexed. Measuring where those 64 seconds
actually go is the next increment.

### What this settles about the synthetic numbers

The brief said a synthetic project proves scaling and does not prove what the application feels
like on a real delivery. Now there is a number for that gap: **a four-module chain over a hundred
real wells costs 2.38× what the same chain costs over a hundred invented ones**, and that ratio
had never been measured before — §6 of the scaling report only ever compared the pay summary.

Two things follow, and they pull in opposite directions, which is why both belong here:

- **The 10× warning stands, and is now reproduced.** The scaling report said every published
  figure is a lower bound for an imported project. It is.
- **But 10× was the pay summary, not the chain.** Reading it as a blanket multiplier on the whole
  23-minute 2000-well chain figure would overstate it by about four times. The chain's own
  measured multiplier is **2.38×**, and within that only `phi_den` is badly affected.

**Neither number licenses a 2000-well claim.** Both were measured at 100 wells, and the whole
point of the scaling report is that this application gets worse per well as the project grows.

## 6. Limits

- **One delivery, one machine, two runs.** Two is enough to show the chain total reproduces to
  1%; it is not enough to estimate a spread. The variance floors in `PERF-VARIANCE-2026-08-23.md`
  were measured on synthetic wells and are assumed, not shown, to carry over here.
- **The hundred wells are a hundred copies of one well.** That is what makes the numbers
  comparable with the synthetic sweep, and it means the spread of real wells — different lengths,
  different gaps, different channel sets — is not exercised. The stress run measures scale on real
  data, not variety in it.
- **The delivery is processed, not raw.** Its channels are corrected outputs, so this is a
  measurement of the app on a client's interpreted deliverable, which is one of the two shapes
  that arrive, not both.
