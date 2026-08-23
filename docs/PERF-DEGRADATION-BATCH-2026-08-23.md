# One appender instead of 89,600 statements — 2026-08-23

Pass 3 of the "how fast is SandiBumi" brief. **The first fix in that pass that survived its own
re-measurement**; the four before it are logged dead in `PERF-ATTEMPTS.md`.

`PERF-PHI-DEN-2026-08-23.md` finished the diagnosis and stopped there. This is the fix it named,
measured against its own before-number on the same machine in the same session.

## 1. What was slow

`phi_den` on the real delivery took **61.7 s** for 100 wells, against 11.6 s for `sw_indo` writing
the **same 624,800 curve rows**. The whole difference was in the write phase: 51.1 s against 5.2 s.

The cause is not the curve rows. It is the **provenance** rows beside them. `phi_den` clamps PHIE to
PHIT, and PHIT is a different number at every sample, so `modules.rs` aggregates the events by
`(kind, detail)` and the detail carries the bound — which means the aggregation never collapses.
896 of 1,562 samples clamp on this delivery, so a 100-well run produced **89,600 degradation rows**
and `ancestry.rs` inserted them **one `conn.execute` at a time**, at roughly half a millisecond
each, holding the single shared DuckDB connection the whole way.

896 clamp notes per well is honest petrophysics — the rock really does hit the bound that often.
Writing them one statement at a time was not.

## 2. The fix

Phase 4 of `write_versioned_rows_batch_raw` now collects the planned rows and writes them through
**one `conn.appender("run_degradations")` plus one `flush()`** — the same shape phases 2 and 3
already used for the curve rows and the archive rows. That is the entire change.

Nothing about the record moves: same rows, same order, same positions, same occurrences, same
`outcome_state` beside them. The row counts in §3 are the proof — they are identical at every step.

### The one hazard the change introduced, and how it is closed

The row-by-row version re-read `MAX(position)` from the table before every insert, so a set that
appeared twice in one batch simply saw its own earlier rows. Batched, those rows are **not in the
table yet**, so a second sighting would read the same seed and plan two rows at the same position —
which the primary key would then reject, failing the whole write.

So the seed is read from the table on first sight of a set and **carried forward in memory**
afterwards. Pinned by the second arm of
`the_batched_degradation_write_records_every_event_in_its_own_position`, which covers both cases
that distinguish the two mechanisms: a *later* batch continuing the sequence (fails if the table
read were dropped) and *one set twice in one batch* (fails if the counter were not carried
forward).

### What was deliberately NOT batched, with the number

The per-well `UPDATE log_sets` and the per-well position lookup stay as they are: **two statements
per well against 896**, so at 100 wells they are ~200 statements out of 89,800 — a fifth of one
percent of what was costing the time. Batching them would also trade the per-well `updated != 1`
check, which names the well that lost its log-set row, for a group count that names nobody.

### The appender is safe here, and that was measured rather than assumed

`run_degradations` carries a `CHECK` on `kind`, a `CHECK` on `occurrences > 0` and a `PRIMARY KEY`
on `(set_id, position)`. A temporary probe confirmed DuckDB's Appender **refuses all three
violations** exactly as the statement did:

```
APPENDER ENFORCEMENT: bad_kind_refused=true bad_occurrences_refused=true duplicate_pk_refused=true rows_left=1
```

The probe was removed once it had answered. Batching therefore cannot weaken the provenance table —
which mattered enough to check, because a faster write that silently accepted a malformed
provenance row would be exactly the trade this brief forbids.

## 3. Measured — paired A/B, same machine, same session

Both states were run back to back today with the same command, release profile,
`--test-threads=1` (concurrent tests distort these numbers — `PERF-VARIANCE-2026-08-23.md` §1).

### Real delivery, 100 wells × 1,562 samples (`pipeline_field_100well_stress`)

| Chain step | BEFORE | AFTER | ratio | degradation rows |
|---|---:|---:|---:|---:|
| vsh_gr | 8.61 s | 8.28 s | 1.04× | 200 → 200 |
| **phi_den** | **61.72 s** | **15.86 s** | **3.89×** | 89,800 → 89,800 |
| ↳ its write phase alone | 51.14 s | 5.34 s | **9.58×** | |
| sw_indo | 11.62 s | 12.99 s | 0.89× | 90,200 → 90,200 |
| perm_wyllie_rose | 3.91 s | 4.79 s | 0.82× | 90,300 → 90,300 |
| **chain total** | **85.86 s** | **41.92 s** | **2.05×** | |

Curve rows written are identical too: 468,600 / 624,800 / 624,800 / 312,400 in both runs, 0 errors
in both runs.

### Generated fixture, 100 wells × 1,562 samples (`perf_baseline`)

| Chain step | BEFORE | AFTER | ratio |
|---|---:|---:|---:|
| vsh_gr | 5.47 s | 5.92 s | 0.92× |
| **phi_den** | **19.70 s** | **14.37 s** | **1.37×** |
| sw_indo | 10.32 s | 11.32 s | 0.91× |
| perm_wyllie_rose | 3.55 s | 4.12 s | 0.86× |
| chain total | 39.04 s | 35.73 s | 1.09× |

## 4. Verdicts

The variance floor for this machine is **1.16×** (`PERF-VARIANCE-2026-08-23.md`); anything inside
it is a different sample, not a result.

| Claim | Verdict |
|---|---|
| `phi_den` is faster on the real delivery | **VERIFIED** — 3.89×, far outside the floor; its write phase 9.58× |
| The whole chain is faster on the real delivery | **VERIFIED** — 2.05× |
| `phi_den` is faster on the generated fixture | **VERIFIED** — 1.37×, outside the floor but much smaller |
| The whole chain is faster on the generated fixture | **INCONCLUSIVE** — 1.09×, inside the floor |
| The three untouched chain steps changed | **NOT CLAIMED** — 0.82×–1.04×, and see below |
| Nothing the run records moved | **VERIFIED** — identical row counts at every step, both tables, 0 errors |

**Why the generated fixture gains so much less, and why that is the expected answer.** The
synthetic wells clamp 177 samples per well where the real ones clamp 896, so there are roughly a
fifth as many rows to write. The size of the win is proportional to how much honest degradation the
rock produces — which is precisely why the brief says a synthetic project proves scaling and does
not prove what Jauhar feels on his own data. Here the synthetic project **understates** the fix by
about 3×.

**The three untouched steps drifted the wrong way, consistently.** All three were slower in the
AFTER run on both fixtures, by 0.82×–0.92×. That is not random: the AFTER runs happened *earlier*
in the session than the BEFORE runs, so whatever the drift is (thermal, disk cache), it favoured
the BEFORE numbers. The reported gains are therefore conservative, and no claim is made about steps
this change does not touch.

## 5. Verification

Two tests in `ancestry.rs`, both **mutation-verified** — each mutation is caught by a *different*
assertion, which is what says the arms are pinning different things:

| mutation | caught by |
|---|---|
| the position counter is not carried forward in memory | arm 2(b), one set twice in one batch |
| a set always starts at position 0 (the table read dropped) | arm 2(a), a later batch continuing |
| no degradation rows are appended at all | arm 1, the content assertion |

`a_batch_carrying_one_impossible_degradation_writes_none_of_them` is the companion: an
`occurrences: 0` event costs the **whole** batch, not just the rows after it. That mattered less
when the loop inserted as it walked; it matters now, because nothing is written until the flush.

Full gate green in 237 s.
