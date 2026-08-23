# Why `phi_den` costs more on real data — 2026-08-23

**Pass 3, increment 9.** `PERF-FIELD-FIXTURE-2026-08-23.md` §7 reported `phi_den` at **3.59×** on a
real delivery against the synthetic sweep, called it the largest real-data finding, offered input
*resolution* as the likely mechanism, and named measuring it as the next increment. This is that.

**The mechanism is not resolution, and the 3.59× was partly an artefact of comparing two different
harnesses.** The real figure is **2.90×**, all of it in the WRITE, and it is caused by
**89,600 individual `INSERT` statements recording clamp events** — one per distinct clamp *bound*,
because the bound is formatted into the string that the aggregation groups by.

**Nothing in the shipped application changed.** The instrumentation is `#[cfg(test)]`.

## 1. First: the published ratio compared two harnesses

The real column came from `pipeline_field_100well_stress`; the synthetic column from
`perf_baseline`'s scaling table. Two harnesses, two project constructions, two chain drivers —
and the workload is contention-dominated (`PERF-SPLIT-2026-08-23.md` measured queue time an order
of magnitude above wall time), so a harness difference alone can move a number several-fold.

Making the comparison fair was possible only after **DEC-098**. The stress harness hard-coded
`GR → GRN_CS` / `NPHI → NPHI_COR`, so pointed at generated wells it refused: *"the source well
fills none of GR=GRN_CS"* — on a well carrying a perfectly good `GR`. With both spellings admitted
to their families, the bindings became unnecessary and were removed, and the same harness now runs
on either fixture from manifest defaults alone.

Same harness, same 100 wells × 1562 samples, release profile, **only the source data differs**:

| operation | generated | real delivery | ratio |
|---|---:|---:|---:|
| `vsh_gr` | 9.508 s | 7.513 s | 0.79× |
| **`phi_den`** | **20.864 s** | **60.430 s** | **2.90×** |
| `sw_indo` | 9.786 s | 10.820 s | 1.11× |
| `perm_wyllie_rose` | 3.332 s | 3.179 s | 0.95× |
| chain total | 43.489 s | 81.942 s | 1.88× |
| pay summary | 6.032 s | 6.041 s | **1.00×** |

Two corrections fall out immediately, and both reduce a published claim:

- **`phi_den` is the only module with a real-data penalty.** The other three are 0.79–1.11×,
  inside the 1.02–1.16× variance floor of `PERF-VARIANCE-2026-08-23.md`. `vsh_gr` is *faster* on
  real data.
- **The pay summary's 10× is withdrawn as a real-versus-synthetic effect.** Measured in one
  harness it is **1.00×**. The earlier 659 ms / 6,590 ms pair was `perf_baseline`'s pay summary
  against the stress harness's — the same operation in two harnesses, which differ by ~9× on
  *generated* wells alone. See §5.

## 2. The penalty is entirely in the write

`lock_probe` (already `#[cfg(test)]`, already used by `perf_read_write_split`) split each chain
step. Its counters sum across threads, so WAIT is contention on the one shared connection and
exceeds wall time whenever more than one well is in flight.

| `phi_den` phase | generated | real | ratio |
|---|---:|---:|---:|
| wall | 20.737 s | 54.961 s | 2.65× |
| read | 5.832 s | 5.777 s | **0.99×** |
| **write** | **11.387 s** | **45.706 s** | **4.01×** |
| wait (summed) | 153.5 s | 150.9 s | 0.98× |

**The read is identical.** That refutes the input-resolution hypothesis outright rather than
leaving it merely unsupported — and the reason it was wrong is visible in the test's own output:

```
real       finite in the six standard columns: GR=0 RES_DEEP=1178 NPHI=0 RHOB=1373 DT=0 SP=0
generated  finite in the six standard columns: GR=1562 RES_DEEP=1562 NPHI=1555 RHOB=1562 DT=1562 SP=1562
```

`phi_den` reads `RHOB` and `VSH`. **`RHOB` is populated in the standard column on BOTH fixtures**,
so both take the fast path; `VSH` comes from the batched `computed_curves` read on both; and the
five conditioning flags (`BADHOLE`, `GAS_FLAG`, `COAL_FLAG`, `TIGHT_FLAG`, `COND_FLAG`) are absent
from both and cost the same miss on both. The earlier report's sentence — *"a synthetic well
answers `RHOB` from a `standard_curves` column, a real one answers it after the resolver has been
down the computed-then-generic path"* — is true of `GR` and `NPHI` on that delivery and **not true
of `RHOB`**, which is the only curve `phi_den` actually asks for.

## 3. And it is not the row count

| module (real) | curve rows written | write time |
|---|---:|---:|
| `vsh_gr` | 468,600 (3 curves) | 2.9 s |
| **`phi_den`** | **624,800 (4 curves)** | **45.7 s** |
| `sw_indo` | **624,800 (4 curves)** | 5.3 s |
| `perm_wyllie_rose` | 312,400 (2 curves) | 2.8 s |

`phi_den` and `sw_indo` write **the same number of rows through the same batched call**, and
`phi_den` takes **12× longer**. Whatever costs is not proportional to the curve data.

## 4. What it actually is: the degradation ledger

Counting `run_degradations` after each chain step, on the real delivery:

| after | rows | added | per well |
|---|---:|---:|---:|
| `vsh_gr` | 200 | +200 | 2 |
| **`phi_den`** | **89,800** | **+89,600** | **896** |
| `sw_indo` | 90,200 | +400 | 4 |
| `perm_wyllie_rose` | 90,300 | +100 | 1 |

Each of those rows is written by its own `conn.execute` inside `ancestry.rs`'s degradation loop,
serialized under the write lock. **≈0.5 ms each on both fixtures** — 45.7 s / 89,600 = 510 µs,
11.4 s / 17,700 = 643 µs — and the generated/real write ratio (4.01×) tracks the row-count ratio
(89,600 / 17,700 = **5.06×**) rather than anything about the curves.

### Why 896 and not 1

`RunDegradation` **is** aggregated — `modules.rs` groups events by `(kind, detail)` and carries an
`occurrences` count, so a repeated event is meant to become one row. The grouping key includes the
`detail` string, and `limit()` formats the bounds into it:

```rust
format!("calculated value was clamped to the existing range [{lo}, {hi}]")
```

Printed side by side, from the test's own output:

```
vsh_gr/CLAMPED   ... clamped to the existing range [0, 1]
phi_den/CLAMPED  ... clamped to the existing range [0.001, 0.0181837320327…]
phi_den/CLAMPED  ... clamped to the existing range [0.001, 0.0209731400012…]
phi_den/CLAMPED  ... clamped to the existing range [0.001, 0.0238905787467…]
```

`vsh_gr` clamps VSH to a **constant** `[0, 1]`, so all 1,562 samples share one detail string and
aggregate to **one row**. `phi_den` clamps PHIE to `[PHIE_FLOOR, PHIT]`, and **PHIT is a different
number at every sample**, printed at full `f64` precision. Every clamped sample therefore produces
a distinct key, and the aggregation that exists to collapse them never collapses anything.

So the count is *"how many samples were clamped"* — 896 of 1,562 on the real delivery against 177
on generated wells. **That difference is honest petrophysics**: real rock hits the porosity ceiling
far more often than the generated section does. The cost is not petrophysics — it is one INSERT
statement per distinct floating-point bound.

## 5. The pay-summary withdrawal, stated separately

The same harness measures the pay summary at **6.032 s generated / 6.041 s real**. The published
10× was `perf_baseline`'s 659 ms against this harness's 6,590 ms — so the ~9× lives between the two
harnesses on identical synthetic data, and **none of it is attributable to real data**. What the
two harnesses do differently is not established here and is not chased: the finding that needed
withdrawing is the real-versus-synthetic claim, and one harness measuring both sides settles it.

`PERF-SCALING-2026-08-22.md` §6 and `PERF-FIELD-FIXTURE-2026-08-23.md` §7 both carry the 10×; both
now need the dated note this document supplies.

## 6. What to do about it — and what not to

**The obvious fix changes no output at all**, which is the only kind this brief permits without
asking: replace the per-event `conn.execute` loop with one batched insert (an Appender, as the
curve write already uses). The rows written stay **byte-identical** — same set_id, same positions,
same kinds, same details, same occurrences — and 89,600 statements become one batch. On the
measured ≈0.5 ms per statement that is most of 45.7 s.

**The tempting fix is a petrophysics decision and is NOT taken here.** Rounding the bound, or
dropping it from the detail, would collapse 896 rows to 1 — and would change what the provenance
record says a run did. It is also arguably *better* provenance ("PHIE clamped to [floor, PHIT] on
896 of 1,562 samples" is more readable than 896 near-identical lines). But it is a change to the
record, and the record is what a study is defended with. That is Jauhar's call, not a performance
one.

Neither is attempted in this increment: this increment measured.

## 7. Limits

- **One delivery, one machine.** Each figure is a single run per fixture; the module timings
  reproduce across the four runs taken here to within the documented variance floor, but no spread
  is estimated.
- **The generated fixture is `make_example_data.py --stress` geology**, so "generated" means that
  section, not a neutral one. Its 177 clamps per well are a property of that rock model.
- **The hundred wells are a hundred copies of one well** on both sides — scale on real data, not
  variety in it. The `x100` in the detail listing is that: the same bound recurring across the
  hundred clones.
- **`lock_probe`'s WAIT sums across threads** and is not a wall-clock quantity; it is reported to
  show contention is unchanged between fixtures (0.98×), not as a duration anyone waits.
