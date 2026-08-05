# End-to-end harness — driving the real desktop app

**Added 2026-07-31.** Optional. Never part of the green gate.

`npm run test:e2e` starts the **built** `sandibumi.exe`, drives it through Tauri's WebDriver
route, and checks that a real import, a real module run and a real export actually happened —
against a real DuckDB file, through the real Rust backend.

This is the only automation in the repo that touches a full run. The vite-only browser trick in
`CLAUDE.md` drives real components, but `window.__TAURI__` is undefined there, so every `invoke`
fails and no module, no write and no export is ever exercised. Replacing that gap is the whole
point of this harness — which is also why the WebdriverIO service's "browser mode" (frontend in
plain Chrome against a Vite dev server) is deliberately **not** used.

## What it can and cannot prove

**It can prove** a run completed, that curves landed in the project, that a dialog behaved, that
an exported file appeared on disk and holds the rows the exporter said it wrote.

**It cannot prove a plot looks right.** The log views render through WebGPU onto a canvas, and
WebDriver sees a canvas element, not curves. There are no pixel assertions here and none should
be added to paper over this: a screenshot diff would fail on every legitimate rendering change
while still saying nothing about whether a crossover shaded on the correct side. That check
stays a human one — see the D pile in `docs/review_triage.md`.

**It drives commands, not the file picker.** Tauri's file dialog is an OS window WebDriver
cannot see. But the picker only *collects* paths: `import_las_files(paths, …)` and
`export_las(well_id, dest_path)` take them as arguments, so the harness calls the commands
directly through `window.__TAURI__.core.invoke` (available because `tauri.conf.json` sets
`withGlobalTauri: true`). Everything downstream of the picker is genuinely exercised; the picker
chrome itself is not.

## Setup on a new machine

1. **Build the app — with the Tauri CLI, not with `cargo`.**

   ```
   npm run tauri build -- --no-bundle
   ```

   This distinction is not pedantry, and getting it wrong cost a day. `cargo build --release`
   compiles the same Rust but bakes in `tauri.conf.json`'s **`devUrl`**, so the resulting binary
   loads `http://localhost:1420` instead of its own embedded frontend. The two are the same size
   and the same name. With a Vite dev server running, the dev-pointing one **passes every test** —
   while driving the dev server's frontend rather than the one that ships, which is the entire
   difference this harness exists to check. The `before` hook now refuses such a binary by name;
   see finding 22 in `review_triage.md`. Do not work around that refusal by starting a dev server.

   The harness deliberately does not build the app itself — on the reference machine the build must
   go through the vcvars 14.29 pin (`CLAUDE.md` → "Dev commands"), and burying a slow,
   environment-specific compile inside a test run makes a toolchain problem look like a test
   failure. It refuses to start with a clear message if the binary is missing.

   The built binary **embeds the frontend as it was at build time**. After changing anything in
   `src/`, rebuild before trusting a DOM-level assertion.

2. **`npm install`** — brings in `@wdio/cli` and `@wdio/tauri-service`.

3. **`cargo install tauri-driver --locked`**, through the vcvars 14.29 pin. The service can
   install it itself, but on this machine the default MSVC toolset (14.50) is broken, so do it
   explicitly. `msedgedriver` *is* fetched automatically at run time, matched to the installed
   WebView2 runtime.

4. **`npm run test:e2e`**.

### Why `@wdio/native-utils` is pinned in `overrides`

`@wdio/tauri-service@1.2.0` pins `@wdio/native-utils@2.4.0` exactly, but was built against a
newer one and imports `installMockSyncOverride`, which 2.4.0 does not export — the service fails
to load at all. `package.json` overrides it to `2.5.0`. Remove the override when the service
ships a corrected pin.

### Why `driverProvider: 'external'`

The service defaults to `'embedded'`, which runs a WebDriver server *inside* the app and
therefore requires `tauri-plugin-wdio-webdriver` to be registered in `lib.rs` — a test-only
plugin compiled into the shipping binary. That is a change to the product to serve the tests,
and it should be a deliberate decision rather than an accepted default. `'external'` is the
plain tauri-driver + msedgedriver route and needs nothing from the app.

## The sandbox — three layers, and why

The harness must never touch a real project. `src-tauri/project.duckdb` is a live working
project on a developer machine, and `project::startup_path()` falls back to `LEGACY_DEFAULT`,
the **relative** path `"project.duckdb"`, whenever the recents list is empty — resolved against
the process working directory. A harness launched from the wrong directory would import test LAS
files straight into real data.

1. **`SANDIBUMI_CONFIG_DIR`** points at a throwaway config directory, so the recents list is
   empty and the developer's real list is never written to.
2. **The working directory is the sandbox.** `@wdio/tauri-service` has no `cwd` option, so
   `e2e/run.mjs` launches wdio *from* the sandbox and the app inherits it. This is why there is
   a launcher at all.
3. **The app is asked what it opened.** `current_project` is read before any test body runs, and
   the run aborts if the answer is not inside the sandbox.

Layers 1 and 2 are configuration and can be broken by an edit elsewhere. Layer 3 is a
measurement. Do not weaken it.

## Teardown, and the rule that outranks tidiness

**Nothing in this harness kills `sandibumi.exe`.** An unclean kill mid-write corrupts
`project.duckdb.wal` (`CLAUDE.md` → "DuckDB WAL resilience"). There is no timeout-kill and no
teardown kill; a hung run is left visible for a human to end deliberately, because a harness
that tidies up by killing the thing it is testing is worse than one that hangs.

Every run is bracketed by a check for new `.corrupt-backup-*` files, which is what
`db::init_db_resilient` leaves behind when a WAL cannot be replayed. If one appears the run
fails and the sandbox is **kept as evidence**.

Two things *are* cleaned up, both before the run rather than after:

- **Orphaned `tauri-driver` / `msedgedriver` processes.** These are drivers, not the app: they
  hold no DuckDB connection and no WAL, so ending one cannot corrupt anything. Reaping happens
  before a run because at teardown a driver may still be closing the app down. The harness
  **refuses to start while `sandibumi.exe` is running** — it cannot tell your session from an
  orphan, and must never terminate the application.
- **Sandboxes from earlier runs.** Windows will not delete an open DuckDB file, so the current
  run's sandbox often survives its own cleanup; the next run sweeps it once the handle is gone.

## Adding a test

Specs live in `e2e/specs/*.e2e.mjs`. The pattern:

```js
const value = await invokeOk('some_command', { someArg: 1 })   // camelCase args
assert.ok(...)                                                  // read the state back
```

Two habits worth keeping:

**Assert on state, not on the return value.** "the command returned" is a weaker claim than "the
curve is in the project". The module test runs `run_workflow_module` and then reads
`list_computed_catalog` back.

**Assert what the code actually writes, not what the convention says.** The export test first
looked for `~VERSION`; `export.rs` writes `~Version Information`. The file was fine and the test
was wrong — which is the direction a harness should fail in, but only if you check the output
rather than assume it.

### Testing a refusal: catch on BOTH sides

The usual `call` helper wraps `invoke` in a page-side `try` and returns `{ok, value|error}`. That
is enough while every command succeeds — but a command you expect to be REFUSED still escapes:
@wdio/tauri-service re-throws page-side errors through its own `__wdio_error__` channel, wdio
retries three times, and the test then fails with the backend's own message as a WebDriverError.
Which is a confusing way to be told that the refusal you were testing for happened exactly as
expected.

Wrap the `browser.execute` call on the Node side too — see `panels.e2e.mjs`. A refusal has to be
data, not an exception.

### Do not rebuild immediately after a run

The link step **deletes** `sandibumi.exe` before writing the new one, and Windows refuses while any
handle is open. A build started while the app is still closing fails with

```
error: failed to remove file …\target\release\sandibumi.exe
```

and — this is the part that wastes time — leaves the OLD binary in place. The next e2e run then
tests the previous build and the change under test appears not to have worked. Check that
`sandibumi.exe` is gone from the task list before rebuilding, or simply rebuild again; orphaned
`tauri-driver` / `msedgedriver` processes are harmless here and the next run reaps them.

### `invoke` does not tell the frontend anything

Seeding data through `invoke` writes to DuckDB and nothing else. The panes only re-read when the
frontend's own code path asks them to, so a spec that imports three wells by command and then
looks at the Wells pane finds it still showing **"No wells ingested yet"** — observed exactly
that.

Force a refresh through a real user path rather than a test hook. `wells.e2e.mjs` dispatches
`change` on the tree's own group `<select>`, whose handler ends in `refresh()`; note it dispatches
**unconditionally**, even when the value is unchanged, because a tidy-looking
`if (sel.value !== '')` guard suppresses the very refresh that is wanted.

### The DOM lags the state it renders

`appState` is the truth; a CSS class on a row is a rendering of it that arrives one async
`refresh()` later (`setMulti`, `togglePin` and friends fire it without awaiting). Asserting
immediately after a click therefore tests which of the two won a race — the same ctrl-click
assertion passed on one run and failed on the next with no code change in between.

Wait for the state to settle, then assert. `wells.e2e.mjs`'s `expectMulti` is the pattern: poll
until the pane shows the expected set, and on timeout assert the last observed value so the
failure still reads as an ordinary expected-vs-actual diff. This is not a workaround — the claim
being tested is that the pane *ends up* right, not how fast it got there.

### Modifier clicks must be dispatched, not `.click()`ed

`el.click()` synthesises a plain click and every modifier reads false, so a ctrl-click written
that way silently becomes a plain click — which in this app CLEARS the multi-selection instead of
extending it, and the test then measures a different gesture entirely. Dispatch a real
`MouseEvent` with `ctrlKey` / `shiftKey` and `bubbles: true`.

### Cost per command — why specs batch their DOM work

`@wdio/tauri-service` runs a window-focus probe in `beforeCommand` for exactly `getTitle`,
`findElement`, `findElements`, `$`, `$$` and `elementClick`. The probe asks for
`@wdio/tauri-plugin`, which this app deliberately does not register (see "Why
`driverProvider: 'external'`"), and each failure costs about **7.5 seconds**. `execute` is not on
that list and is effectively free.

So a six-tab ribbon walk written the obvious way — `await $(sel)` then `await el.click()` — is
twelve of those commands and blows the 180 s mocha timeout. The same walk driven through one
`browser.execute` finishes in under a second. `shell.e2e.mjs`'s `clickTab` documents the pattern.

The trade is small and worth stating: an in-page `el.click()` is not a trusted user gesture, so it
cannot exercise anything gated on user activation (fullscreen, clipboard, autoplay). For a plain
click listener the two are equivalent — but when the *gesture* is the thing under test, pay the
7.5 s and use `$`.

## Current coverage

Seventy-seven tests across twenty spec files. Specs share ONE app launch and one project (see the spec
grouping note in `wdio.conf.mjs`), so write each one to establish what it needs and to assert
changes as before/after differences rather than as absolute state.

`pipeline.e2e.mjs` — five tests, chosen to prove the harness rather than to chase coverage:

| Test | What it proves |
|---|---|
| Sandboxed project, empty | The isolation held and the project is fresh |
| Import three example LAS as a named set | Real ingest into real DuckDB, three wells |
| Run `vsh_gr`, read the catalog back | The module engine ran and curves were written |
| Export LAS | A real file on disk, with the row count the exporter reported |
| Ribbon and dock populated | The frontend booted, not just the backend |

`shell.e2e.mjs` — the application chrome (manual plan T-SHELL-01/02/03, T-ADV-01, T-RT-16 step 5):

| Test | What it proves |
|---|---|
| Ribbon, status bar, workspace render | Every declared tab has a panel and none is orphaned |
| One panel at a time, captioned groups | Asserted on `checkVisibility()`, not the `hidden` attribute |
| Advance tab flagships + calibration tools | The five promoted manifests resolved; SandiMin/RtC/S/ML present |
| Legacy multimin has no ribbon button | Both retirement mechanisms, swept across the whole ribbon |
| Language EN → ID → SU → JV → EN | The right dictionary each time; untranslated terms stay English |

`wellgroups.e2e.mjs` — well-group scoping (T-INT-09):

| Test | What it proves |
|---|---|
| Create and activate a group | The group exists and is the active one |
| Exactly one group active | `set_active_well_group` clears the others |
| Membership replaces | A second write is not an append |
| A scoped run writes only to members | The outsider's curves are byte-identical before and after |

`wells.e2e.mjs` — Wells pane selection (T-WELL-02 in full, the selection half of T-WELL-01):

| Test | What it proves |
|---|---|
| Plain click activates | Exactly one row marked, no multi-selection created |
| Ctrl-click builds a selection | Adds, TOGGLES off, and never moves the active well |
| Shift-click takes a range | Inclusive from the anchor, and stops — two of three, not all |
| ⇄ inverts within the visible wells | Selection and inverse partition the visible list exactly |
| Plain click clears | The scope a user thinks they dismissed is really gone |
| ★ pins a well | The pin reached the project, read back via `list_pinned_wells` |

`zones.e2e.mjs` — the Zones pane (T-WELL-15 steps 2–5):

| Test | What it proves |
|---|---|
| Add a zone | Stored against the well and shown in the pane at the entered depths |
| Re-add under the same name | UPDATES in place; exactly one row, never a duplicate |
| `bottom <= top` is refused | A dialog-only guard — the backend has none — asserted on stored state |
| Zones stay on their own well | No leak to a neighbour that nobody edited |
| Delete from the pane | Gone from the project and from the table |

`moduledialog.e2e.mjs` — the module pane and its two refusals (T-PREP-01, T-INT-06 legs 2-3,
T-PETRO-03 step 1):

| Test | What it proves |
|---|---|
| The pane form is built from the manifest | Scope, parameters, Outputs note, and a curve picker leading with "(none)" |
| An out-of-range parameter is refused | The message names the parameter AND its bounds, and nothing was written |
| An empty scope is refused | The message says how to fix it, and nothing was written |

Both refusals compare a project-wide `computed_curves` fingerprint before and after. That is the
real claim: a dialog that prints a complaint and then runs anyway is indistinguishable from a
correct one if you only read the message.

`catalog.e2e.mjs` — the Curve Catalog (T-MLEQ-16):

| Test | What it proves |
|---|---|
| Rows list raw and computed curves | Including the set/version marker, the visible half of the write discipline |
| Search narrows the table | Every surviving row matches, and clearing restores all of them |
| A sortable header reorders | The second click reverses the first exactly |

`sessions.e2e.mjs` — named workspace snapshots (T-SHELL-10, the Ctrl+S half of T-SHELL-11):

| Test | What it proves |
|---|---|
| Save under a name | Written to the `documents` store, and the dialog closes itself |
| The snapshot's shape | `version`, `layout` and `well` all present — checked field by field |
| Listed in Open Session | The dialog can find what was saved |
| Ctrl+S re-saves quietly | No dialog reopens; the status line names the session |
| Delete | Gone from the store |

`panels.e2e.mjs` — panes that open and report (T-REP-17, T-AUX-01, T-AUX-02):

| Test | What it proves |
|---|---|
| The SQL console opens with a runnable starter | And it is RUN — which is how finding 23 was caught |
| A write is refused, and so is a commented SELECT | The second is pinned as-is, not endorsed |
| The performance monitor shows gauges | Every one labelled AND carrying a value |
| Help opens and names no vendor | The provenance rule, where a user reads it |

`wellgroupmanager.e2e.mjs` — the manager and the scoping it drives (T-WELL-04/05/06):

| Test | What it proves |
|---|---|
| Create from the manager | Written, and listed |
| Rename in place | The group ID and the membership both survive |
| The tree scopes to the active group | Exactly the members, not merely the right count |
| A newly opened pane inherits the scope | Uses a module no other spec opens — panes are singletons |
| An already-open pane does NOT re-scope | Pinned as the known open bug; goes red when fixed |
| Delete | The group goes, the wells stay |

`workflow.e2e.mjs` — the chain builder (T-BATCH-01, T-BATCH-04, T-BATCH-06 save refusals):

| Test | What it proves |
|---|---|
| The step picker is clean | Grouped by category, and the retired multimin is absent |
| Both save refusals | Unnamed and stepless — and neither leaves a document behind |
| A two-step chain saves | The stored JSON carries the steps in the order they were added |
| It reloads in order | Cleared, reloaded, re-saved, compared as JSON |
| Delete | Gone from the store |

`scope.e2e.mjs` — how a batch pane resolves its scope (T-WELL-03, T-AUX-15):

| Test | What it proves |
|---|---|
| All resolves to every well | The baseline |
| Selection follows the tree LIVE | Growing and shrinking, without reopening the pane |
| The star scope resolves to the pinned set | And follows a second pin |
| An empty set resolves to nothing | Never a silent fallback to All |

`history.e2e.mjs` — Processing History attribution (T-SHELL-15):

| Test | What it proves |
|---|---|
| A single-well run names the well that RAN | Selected A, scoped to B, the row must say B |
| A batch names no single well | And states how many it covered |

`undo.e2e.mjs` — undo/redo through a real DB Inspector edit (T-SHELL-13, part of T-REP-15):

| Test | What it proves |
|---|---|
| The undo button is labelled after an edit | The tooltip names the action, not just "Undo" |
| Undo restores the VALUE | Not merely the button state |
| Redo reapplies it | The full round trip |
| A data edit is not "unsaved work" | The dirty dot means named-save freshness, nothing else |

`equations.e2e.mjs` — the equation editor (T-MLEQ-02, T-MLEQ-05 step 1):

| Test | What it proves |
|---|---|
| The engine note matches `python_status` | Missing scipy is a note; missing Python is a warning |
| An unsaved equation will not run | And the refusal writes nothing |

`ml.e2e.mjs` — the ML pane (T-MLEQ-01, T-MLEQ-14 steps 1 and 3):

| Test | What it proves |
|---|---|
| The pane opens with its form | Task, Algorithm, and the save-the-model field |
| A Mask control exists | The plan says it does not — finding 24 |
| A run with no input curve is refused | And nothing is written |

`rocktyping.e2e.mjs` — the Rock Typing ribbon group (T-RT-01):

| Test | What it proves |
|---|---|
| The menu lists exactly its catalogued modules | Compared as a set both ways |
| Re-clicking focuses, never duplicates | Two panes for one module = a run nobody configured |
| The pane is built from its manifest | Including an Outputs note naming the real outputs |

`paysummary.e2e.mjs` — the pay summary and its row invariants (T-BATCH-07):

| Test | What it proves |
|---|---|
| The pane produces a table | Every row names its cutoff level |
| Net within gross, levels nested | PAY ≤ RESERVOIR ≤ SAND, counted so it cannot pass vacuously |
| HPV within net × porosity | The ceiling the arithmetic implies |

It never picks a cutoff — the invariants hold for any, and a VSH/PHIE/SWE value invented to make a
test pass would be an unsourced petrophysical number in the repo.

`report.e2e.mjs` — the report corner (T-REP-07, the opening half of T-REP-01):

| Test | What it proves |
|---|---|
| The Composite pane opens | The smoke claim |
| The methodology table saves | As parsed ROWS keeping their pipe-separated fields |
| A rebuilt pane reads it back | The read path, not the textarea's own memory |

`wellheader.e2e.mjs` — the Well Header round trip (T-AUX-03):

| Test | What it proves |
|---|---|
| A TD-only edit keeps the location | The reopened form shows stored values, and the save preserves them |

### Do not name a variable `before` inside a `describe`

It shadows mocha's own `before()` hook, so the hook call becomes `null(...)` and the whole file
fails at LOAD time with "unable to load spec files quite likely because they rely on `browser`" —
a message that points nowhere near the cause. Same for `after`.

Test data is `dataset for test/examples/` (`SANDI-*`) only. Never a real client project, never a
path from `SANDIBUMI_FIELD_FIXTURES`.
