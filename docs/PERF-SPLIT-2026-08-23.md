# The read/write split inside a module run — 2026-08-23

**Pass 3, increment 2. It measures and changes nothing that affects an answer.**

Increment 1 (`PERF-DIAGNOSIS-2026-08-22.md`) proved the shared `Mutex<Connection>` serialises
parallel reads and measured that one connection per thread would buy 4–8× **on the read it timed**.
It could not say how much of a real module run that read is, so it could not turn "4–8× on reads"
into "X off a chain". This is that measurement.

## 1. Which modules, and why not one and not all 62

The split is driven by **how many curves a module reads and writes**, not by its arithmetic, so one
module cannot speak for the rest — and Pass 2 measured a **4.2× spread** in per-well cost across
the four chain modules alone (65 / 271 / 260 / 95 ms per well at 2000 wells). Measuring one and
generalising would be presenting a sample as a population fact.

All 62 module specs would be worse, not better: most never appear in a batch workload.

**The four modules of the standard chain are the population, not a sample of it.** They span 1→3
inputs and 1→3 outputs, and they are the workload every other number in this brief was taken on, so
the answer lands against the 23-minute chain directly rather than having to be transferred to it.

## 2. Where the probe lives, and why it cannot ship

`src-tauri/src/lock_probe.rs`. Increment 1 measured from outside, which is always preferable; it
ran out of road, because the phases are interleaved inside one private closure in `workflow.rs` and
no caller can see the boundary between them. The only honest options were to guess or to time it
where it happens.

The brief's rule is that a profiling harness never lives in the shipped app. So the module is
`#[cfg(test)] mod lock_probe;` and every call site is `#[cfg(test)] let _g = …` — an attribute on a
`let` statement, so in a non-test build the statement does not exist at all.

**Verified rather than asserted:** `lock_probe.rs` was deliberately corrupted with a line of
invalid Rust and `cargo check --release` still finished clean, because the file is never compiled
into a shipped build. The file was then restored.

## 3. The instrument was wrong the first time, in the direction that mattered

The first version timed the lock **hold** and not the **wait**. Since COMPUTE is derived as
(per-well total − timed phases), the queueing time did not go missing — it was **misattributed to
COMPUTE**, and read as petrophysical arithmetic.

It reported 466 ms per well of "compute" for a module that costs 52 ms run on its own, and
concluded:

> READ 3.5% — COMPUTE 91.6% — **a pool would be worth 1.03×**

which is the exact opposite of increment 1's finding, stated with a decimal point on it. The fix is
a separate `WAIT_NS` counter bracketing the lock acquisition itself.

## 4. The result

100 wells × 1562 samples, release, four-module chain:

```
MODULE                  WALL       QUEUE        READ     COMPUTE       WRITE
vsh_gr               5072.1ms    36456.1ms     1402.0ms      12.8ms     2834.9ms
phi_den             17549.5ms   135764.0ms     5214.2ms     212.2ms     8786.2ms
sw_indo             10067.3ms    92630.3ms     3538.7ms      31.3ms     4092.9ms
perm_wyllie_rose     3190.3ms    22884.0ms      874.7ms       8.6ms     1679.8ms
TOTAL               35879.1ms   287734.4ms    11029.5ms     265.0ms    17393.9ms
```

Every step produced `100/100 wells ok`. As shares of the summed work:

| phase | share | what a connection pool does to it |
|---|---|---|
| **QUEUE** | **90.9%** | deletes it |
| READ | 3.5% | parallelises it |
| COMPUTE | **0.1%** | nothing — already parallel, and already free |
| WRITE | 5.5% | nothing — DuckDB is single-writer by design |

**Nine tenths of a batch run is threads standing in a queue.** The petrophysics — Larionov, density
porosity, Indonesia saturation, Wyllie-Rose permeability, over 156,200 samples — is **0.1%**.

The four modules agree with each other on the split, so this generalises across them rather than
describing one.

## 5. What that is worth, in wall clock

The phase totals are **sums across wells and threads** (they come to 882% of the wall clock, because
a blocked thread still accumulates time). Quoting the improvement against that summed total would
report **~14×**, which is a true statement about thread-seconds and a false one about how long a
chain takes. In wall clock:

```
queue gone, reads 4x faster:  35.9s -> 20.4s  (1.76x)  - of which write 17.4s (85%)
queue gone, reads 8x faster:  35.9s -> 19.0s  (1.88x)  - of which write 17.4s (91%)
```

**A connection pool is worth roughly 1.8× on a batch run — and then the write is the wall.** At that
point 85–91% of what remains is one batched transaction that cannot be parallelised, because DuckDB
is single-writer by design.

**The ratio is stable even though the absolute is not.** Two runs at 100 wells gave wall clocks of
55.4 s and 35.9 s (ordinary machine load), and predicted **1.87×** and **1.88×**. The prediction is
a ratio of phases measured within the same run, so it survives what the absolute does not — and no
absolute figure here should be quoted without its run.

## 6. What this changes about #129

Increment 1 recommended #129 on the strength of 4–8× on reads. That number was right and its scope
was narrow. **End to end it is ~1.8×**, because reads are only 3.5% of the work and the queue —
which is 90% — is what actually disappears.

1.8× on a 23-minute chain is about ten minutes back. That is worth having. It is not the order of
magnitude "32 cores doing nothing" suggests, and the difference between those two claims is the
whole reason this increment exists.

**Two consequences worth stating plainly:**

- **The safe half of #129 captures the whole win.** A pool of READERS with a single writer removes
  the queue and parallelises the reads. A pool of writers would buy nothing measurable here and is
  the version that could break the PK-less write discipline. They are not the same change.
- **After #129 the next bottleneck is already identified and is not a lock.** The write is 17.4 s of
  a 19–20 s post-fix chain. Anything further has to make the write cheaper, not more concurrent.

Still Jauhar's call, and still subject to `ROADMAP.md`: #129 cannot be signed off without a live
`tauri dev` run on 100+ real wells.

## 7. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** Every attempt lands there, kept and reverted alike —
four ledgers in four documents is four places for the same dead idea to go missing from.
