# The Field Dashboard reads every well at once — measured (2026-08-24)

Pass 3 increment 14. `PERF-DASHBOARD-2026-08-23.md` measured where the Field Dashboard spends its
time and found the answer was not petrophysics: the cut-off classification of 156,200 samples per
100 wells is **below what the instrument can separate from noise**, and 101.7% of the 500-well wall
clock is four database reads per well. This increment makes those reads overlap. Same mechanism as
#129 stage 2, pointed at the one operation `PERF-POOL-STAGE2-2026-08-23.md` §4 explicitly recorded
as *not* on the pooled path.

## 1. What changed

`run_pay_summary`'s per-well loop took `db.lock()` and made four reads under it — the well's name,
which log set its PHIE belongs to, the four curves the cut-offs read, and the zone tops. One well at
a time, on one connection, on one thread.

Those four reads are now `read_well_summation_inputs`, and every well's copy runs at the same time
through the #129 reader pool:

```rust
let read: Result<Vec<Option<WellSummationInputs>>, String> = req
    .well_ids
    .par_iter()
    .map(|well_id| pool.read(db, |conn| Ok(read_well_summation_inputs(conn, well_id, req))))
    .collect();
```

**Everything below the reads is untouched.** The cut-offs, the zone sweep, the discretisation, the
row building and the FLAG_* write all run exactly where they were — serially, in well order, the
write still under `db.lock()`. The struct carries `curve_names` rather than letting the write site
rebuild it, because that list is serialised as the run's `inputs_json`: a provenance record built
twice is a provenance record that can drift from what was actually read.

### The one thing that decides whether a number can move

A `None` from `read_well_summation_inputs` is a well the serial loop would have `continue`d past —
an unresolvable PHIE ancestry, no curve frame, or an unreadable zone list — and it stays a silent
skip, exactly as before.

An `Err` is something else: the read could not be performed at all, because the project was replaced
underneath it. **That fails the whole run** rather than being folded into the skip. Quietly dropping
every well of a summary because the project moved would produce a field total that is simply too
small, with nothing on screen to say so.

Row order is unchanged: rayon's indexed `collect` preserves position, so `prefetched` is
`req.well_ids` order minus the skips.

## 2. The measurement

`perf_baseline_test::perf_dashboard_scale`, release build, one test thread, both arms in the same
session on the same machine, each arm building its own fixture from the same seeds. **The AFTER arm
ran first**, so the disk cache was warmer for BEFORE — the bias points against the change.

The only difference between the arms is the block quoted above. BEFORE is master's shape: a serial
loop taking one lock per well and holding it across all four reads. The struct, the read function,
the signature and the whole summation below it are identical in both, so the delta is attributable
to concurrency and to nothing else.

```
WELLS        before        after      ratio
   10       60.0 ms      10.9 ms      5.50x
  100      864.0 ms     164.8 ms      5.24x
  500     7680.6 ms    2393.1 ms      3.21x
```

Every ratio clears `PERF-VARIANCE-2026-08-23.md`'s measured **1.16x** floor for this machine by a
wide margin. The ratio falls as the field grows, which is the honest reading: the reads get heavier
per well (a bigger file is slower to read out of), and concurrency cannot make a read cheaper — only
simultaneous.

### And on a real delivery, which is the number that counts

`pipeline_field_test::pipeline_field_100well_stress` against `SANDIBUMI_FIELD_FIXTURES` — 100 real
wells, 1,562 samples each — same discipline, AFTER arm first:

```
                                      before        after      ratio
Field Dashboard (stats_only)        963.97 ms    219.57 ms     4.39x
pay summary that WRITES flags        7.7189 s     7.0081 s     1.10x   <- inside the floor
four-module chain                   30.2152 s    29.1945 s     1.03x   <- inside the floor
```

**The last two are not claims, and the second row is the reason this section exists.** That timing
was the only field-scale pay-summary measurement in the repo, and it runs `stats_only: false`: it
writes three FLAG_* curves per well through the single writer. The dashboard never writes. At 7 s
against the dashboard's 219 ms, a read-side change could not have shown up in it however large —
reporting "no measurable change on real data" from that number would have been true and completely
misleading, the wrong instrument rather than a null result.

So the `stats_only` timing was ADDED to that same `#[ignore]`d test, ahead of the writing one so it
runs on the colder cache and reports the pessimistic figure. The chain total is carried for the same
reason stage 2 carried the write counter: it is the check that nothing else moved.

**The absolutes are not the claim, the pair is.** `PERF-DASHBOARD` §8 already recorded this probe's
500-well wall clock varying 5532.6 / 13952 ms across two runs of the same code; 7680.6 ms sits
inside that spread. Only numbers measured in one session against each other mean anything here.

### The reads overlap, and the probe proves it from the other side

The probe replays the same four reads serially, from outside, before it times the operation. In the
AFTER arm that replay costs **6919.0 ms** while `run_pay_summary` itself finishes in **2393.1 ms**.
A serial reader can never beat the sum of its own reads; a concurrent one can, and that gap is the
mechanism, measured independently of the before/after pair.

It also breaks one of the probe's own columns, which is worth saying rather than quietly enjoying.
`compute` is derived by subtracting the serial replay from the wall clock, so it now clamps to zero
**by construction** rather than by measurement. It already read zero at 500 wells before this change
for an unrelated and genuine reason (`PERF-DASHBOARD` §2: the arithmetic really is below the noise),
so nothing is lost — but a column whose name no longer describes how it was derived is exactly the
thing that gets quoted later as if it meant something.

## 3. Nothing moved

`cargo test --lib`: **1244 passed, 0 failed, 44 ignored** — the same counts as before the change.

On the real delivery, `pipeline_field_full_run` prints exactly what it printed for stage 2 — the
three validation means (VSH 0.6588, PHIE 0.1023, SWE 0.5969) and all four wells' net, N/G, PHIE, SWE
and HPV. **300 pay rows in both arms** of the 100-well stress test, and the four-module chain
reported 0 errors of 100 on every step in both.

The order of the rows is the other half of that check and it is structural rather than measured:
rayon's indexed `collect` preserves position, so `prefetched` is `req.well_ids` order minus the same
skips the serial loop made. A summary whose rows silently reordered would still total correctly.

## 4. Safety: which half of the pool's guard covers this

`PERF-POOL-RISK-2026-08-23.md` §3 M1 is the silent corruption mode — a pooled handle that survives a
project swap goes on serving rows out of the old file. Two things guard it, and **only one of them
covers this call site**:

- **Structurally, a swap cannot begin while a job runs.** That is the guard stage 2 leans on, and it
  does not apply here: the Field Dashboard's `stats_only` pay summary is deliberately NOT a job. It
  runs silently off-thread precisely so it never posts a job card the user did not ask for.
- **The generation stamp is re-checked when each read finishes**, and a read whose project moved
  underneath returns an ERROR instead of an answer. This is what covers the dashboard, and it is why
  `reader_pool.rs` describes the stamp as the whole protection rather than a backstop.

**That is not a regression — it is an improvement, and the reason is worth stating.** The old serial
loop took `db.lock()` afresh for every well, so a swap part-way through would have handed later
wells the NEW connection: one field summary built from two different projects, with no error and no
mark on any row. The new path refuses instead.

The pool used is the SESSION's (`db.1`), never one made on the spot, at every call site threaded
here — the pay-summary command, report render / PDF / batch, the workbook, the deck, and the Word
report and its batch. A pool of its own would never be invalidated, which is M1 again.

## 5. What this does NOT do, and why it needs a decision rather than a commit

`PERF-DASHBOARD` §4 found that **one query is 36% of the Field Dashboard** — the set-id scan that
resolves which log set a well's PHIE belongs to. It is still there, and it is still roughly that
share; this increment made it that share of a smaller number.

It cannot be removed as a performance change, because removing it changes **which wells appear in
the summary**: today `try_resolve_ancestry_input` errors when a well's PHIE does not resolve to
exactly one live log set, `first_available_input_alias` propagates that, and `run_pay_summary` turns
it into a silent skip. A cheaper existence check would include such a well.

Whether a well with ambiguous curve ancestry belongs in a field summary — and whether being dropped
from one silently is acceptable — is a petrophysics and provenance decision. It is Jauhar's, and it
is not taken here.

## 6. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** This is attempt 9.
