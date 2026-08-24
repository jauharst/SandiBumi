# The multi-well plot overlay stopped queueing — measured (2026-08-24)

Pass 3 increment 15. The Field Dashboard was the last operation on the pooled path
(`PERF-DASHBOARD-PARALLEL-2026-08-24.md`); this is the interactive one. When a crossplot,
histogram or Pickett plot is asked to show the rest of the field behind the selected well, the
frontend fetches every context well's curves — and it already asks for eight at a time.

## 1. What was wrong, and it is not what I expected

`plotCommon.ts::fetchContextLayers` runs `context_fetch_concurrency` = 8 concurrent workers, each
calling `get_curve_data` for one well. That command was:

```rust
#[tauri::command]                       // <- not async
fn get_curve_data(db: tauri::State<DbState>, …) -> … {
    let conn = db.0.lock().unwrap();    // <- not the pool
```

Two things there could defeat the frontend's concurrency, and **the probe was built to test both.
One of them is refuted.**

### Refuted: the lock count

The hypothesis was that taking the connection mutex once per well costs materially more than
taking it once for a whole loop, which would make `perf_baseline`'s `plot data: ALL wells` row —
one lock, serial loop inside it — an optimistic instrument. Measured over 100 wells, one lock
against one-lock-per-well on eight threads: **1.06×**, and 1.13× at ten wells. Both inside this
machine's **1.16×** variance floor (`PERF-VARIANCE-2026-08-23.md`).

So lock granularity costs nothing here, and the committed row is a fair measure of the read
however the lock is taken. That claim is withdrawn.

### Confirmed, from the dependency's own source: the webview thread

A default `#[tauri::command]` on a **sync** fn expands through `tauri-macros`' `body_blocking`,
which runs the function body INLINE in the IPC handler; only `body_async` reaches
`respond_async_serialized`. So the eight concurrent invokes were handled one after another on the
webview thread, with the window unable to paint between them — the frontend's concurrency limit
described nothing, and `context_fetch_concurrency` = 8 was a setting with no effect.

This half is read off `tauri-macros-2.6.3/src/command/wrapper.rs`, not measured: the IPC leg is
not reachable from a unit test, which `perf_baseline_test`'s own header already says of every
number in it.

## 2. The measurement

Three arms, defined once and run on BOTH fixtures - the real delivery through
`pipeline_field_test::pipeline_field_100well_stress` and the synthetic sweep through
`perf_baseline_test::perf_plot_overlay_shape`, which share the three helpers rather than copying
them so the two cannot drift apart. Release, one test thread, the same
wells, the same two curves, and — asserted, not assumed — the same total point count:

- **`1 lock`** — what `perf_baseline`'s committed row measures: one lock, serial loop inside it.
- **`N locks`** — 8 workers, `db.lock()` per well. This is also, exactly, what the app would cost
  if the command were made async and NOTHING else.
- **`pooled`** — 8 workers, `ReaderPool::read` per well: the change.

`OVERLAY_WORKERS` is 8, duplicated from `context_fetch_concurrency` rather than read from it,
because a harness that silently follows a product limit stops measuring the limit it thinks it is.
The **pooled arm runs FIRST**, on the coldest cache, and the rayon pool construction sits inside
its timed region and not the serial arm's — both biases point against the change.

### On the real delivery, which is the number that counts

`pipeline_field_test::pipeline_field_100well_stress` against `SANDIBUMI_FIELD_FIXTURES` - 100 real
wells - runs the same three arms, pooled first, before anything else reads:

```
100 REAL wells      1 lock    N locks     pooled       N/1    pool/N
                  1002.4ms   1035.0ms    358.5ms     1.03x     2.89x
```

**2.89x is the claim, not the 6.55x below it.** The synthetic field overstates this change by
about two at the same well count, which is exactly the divergence `2c` warns about and this repo
has measured before in the other direction (`PERF-ATTEMPTS` attempt 5: 1.37x generated against
3.89x real). All three arms read the same 312,400 values, so the real wells carry both NPHI and
RHOB at the full sample count - the gap is not a difference in what was read.

`N/1` is **1.03x** here too, so the refuted lock-granularity hypothesis stays refuted on real
data and not only on synthetic.

Why the real win is smaller: reading a real well is genuinely more expensive - 1002 ms serial
against 316 ms for the same well count synthetic - and concurrency cannot make a read cheaper,
only simultaneous. The more of the time is unavoidable bytes-off-disk, the less of it a pool can
remove. That is the same shape stage 2 recorded when a modelled 1.95x measured 1.39x.

### And on the synthetic field, for the scaling shape

Two independent sessions:

```
                 1 lock    N locks     pooled       N/1    pool/N
run A, 10          9.7ms     11.0ms      3.6ms     1.13x     3.02x
run A, 100       131.5ms    139.2ms     26.0ms     1.06x     5.35x

run B, 10         23.6ms     25.3ms      7.5ms     1.07x     3.38x
run B, 100       315.9ms    292.4ms     44.6ms     0.93x     6.55x
run B, 500      1466.9ms   1441.0ms    219.6ms     0.98x     6.56x
```

**The ratio is the result; the absolutes are not.** 100 wells reads 131.5 ms in one session and
315.9 ms in the next — 2.4x apart for identical code, the same instability `PERF-DASHBOARD` §8
recorded when the same probe's 500-well figure came out 5532.6 ms and 13952 ms. Only arms measured
against each other in one session mean anything here, and both sessions agree: **~3.0-3.4x at ten
wells, 5.4-6.6x at a hundred, 6.6x at five hundred.**

Those are the SCALING shape and nothing more: the ratio holds as the field grows,
which is what a synthetic field is good for. What a user feels is the 2.89x
above.

`N/1` lands at 1.13, 1.06, 1.07, 0.93 and 0.98 — twice BELOW one. Lock granularity is not merely
inside the floor, it is indistinguishable from noise in both directions.

And because `N locks` is the async-only shape, that column says what half a fix would have bought:
**nothing**. 292.4 ms against the 315.9 ms it would replace, well inside the floor.

## 3. The fix, and why it is one change rather than two

```rust
#[tauri::command]
async fn get_curve_data(db: tauri::State<'_, DbState>, …) -> … {
    let owned = db.share();
    let series = tauri::async_runtime::spawn_blocking(move || {
        owned.1.read(&owned.0, |conn| { … })
    }).await…
```

`async` + `spawn_blocking` puts each call on the blocking pool so the eight really do overlap;
reading through the SESSION pool (`db.1`) stops them queueing on the connection mutex once they
do. **Neither half alone moves anything, and that is measured rather than argued** — the `N locks`
column below is exactly what the app would cost if the command were made async and nothing else,
and it sits inside the floor of the serial figure it would replace.

It follows the house idiom (`async fn` + `db.share()` + `spawn_blocking`) that `compact_project`
and `await_project_open` already use. `#[tauri::command(async)]` would have been shorter and
appears nowhere in this crate; a second idiom for the same job is how the next reader learns the
wrong one.

Nothing on the frontend changed. `ipc.ts::getCurveData` already awaited the invoke, and
`tauri::ipc::Response` is raw bytes whichever context runs it, so the `ArrayBuffer` contract #131
established is untouched.

## 4. Safety

Same trade the Field Dashboard takes, for the same reason. A read whose project was replaced
mid-flight returns an ERROR instead of the old project's curves — the pool's generation stamp,
which `PERF-POOL-RISK-2026-08-23.md` §3 M1 names as the whole protection. The job guard does not
apply here: drawing a plot is deliberately not a job.

That is an improvement rather than a regression. The old command held `db.0.lock()` for its whole
body, so a swap between two of the eight fetches would have handed later wells the NEW connection
and drawn one cloud from two projects, with nothing on screen to say so.

## 5. Nothing moved

`tools\check.ps1`: **GATE GREEN**, `cargo test --lib` **1244 passed, 0 failed, 45 ignored** -
the same 1244 as before the change, with the ignored count up by the probe added here and declared
in `docs/takeover/evidence/gate2-ignored-test-inventory.json` under SB-CORE-032.

`get_curve_data` returns the same bytes it always did. The change is which thread runs the read
and which connection it runs on; `equations::fetch_curve_data` and `pack_curve_series` are
untouched, so the `ArrayBuffer` contract #131 established is byte-for-byte what it was.

The probe carries the correctness half itself: all three arms `assert_eq!` their total point
count, at every size — 31,240 values at 10 wells, 312,400 at 100, 1,562,000 at 500, identical
across one lock, N locks and pooled. An arm that read less would fail rather than look fast.

Nothing on the frontend changed at all, so every existing display contract stands unexamined
because it is unexercised: viewport filtering still precedes decimation, structural endpoints are
still retained, min/max decimation still never averages, and neither the screen nor SVG/PDF
resamples. This increment cannot have touched them — it changed no TypeScript and no rendering
code.

## 6. What this does NOT do

**A zone-scoped overlay still pays N serialized `list_zones` calls.** `contextZoneWindow` is
called once per well in the same loop, and `list_zones` is one of the 141 sync commands that take
`db.0.lock()`. `PERF-ATTEMPTS.md` §1's part-by-part table measured it at **0.3 ms for one well at
10 wells and 0.4 ms at 500** — so roughly 35 ms across 100 wells, which is an ESTIMATE scaled from
a one-well measurement, not something measured here. After this change that would be the larger
half of a zone-scoped overlay. The all-depths overlay (`"*"`, the default) makes no zone call at
all and takes the full win.

It is left alone deliberately: one variable at a time is the rule this whole pass runs on, and it
is the same one-line change of the same shape if it is wanted.

**And the other 154 sync lock-taking commands are untouched.** `lib.rs` declares **259**
commands: **89** async, **155** sync that take `db.0.lock()`, **15** sync that touch no
connection. Counted by a script that requires the attribute to START a line and then walks to the
next `fn`, and the three figures are asserted to sum to the total — because a naive
`grep -c '#\[tauri::command\]' src/lib.rs` returns **262**, counting the three doc comments that
quote the literal, one of which is the comment added by this very change.

Most of the 155 are small metadata reads where being on the webview thread cannot matter.
Converting them as a sweep would be a large diff justified by no measurement, which is what
`PERF-ATTEMPTS.md` exists to prevent. The one to look at next if any is `list_zones`, named
above, because it is the only other command on this per-well loop.

## 7. The attempt ledger

**One ledger, in `docs/PERF-ATTEMPTS.md`.** This is attempt 10 — and the refuted half above is
recorded there as its own row, because a hypothesis that dies is the cheapest kind of result and
the one most likely to be re-run.
