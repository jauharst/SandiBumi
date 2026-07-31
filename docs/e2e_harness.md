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

Fifteen tests across three spec files. Specs share ONE app launch and one project (see the spec
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

Test data is `dataset for test/examples/` (`SANDI-*`) only. Never a real client project, never a
path from `SANDIBUMI_FIELD_FIXTURES`.
