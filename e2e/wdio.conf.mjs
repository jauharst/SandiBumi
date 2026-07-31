// WebdriverIO configuration for the SandiBumi end-to-end harness.
//
// This drives the REAL BUILT DESKTOP APP through Tauri's WebDriver route (tauri-driver +
// msedgedriver, managed by @wdio/tauri-service). It deliberately does NOT use the service's
// "browser mode", which runs the frontend in plain Chrome against a Vite dev server: in that
// mode `window.__TAURI__` is absent, every `invoke` fails, and no module run, no DuckDB write
// and no export is ever exercised. Proving those is the entire reason this harness exists.
//
// "The real built app" means a binary built by the TAURI CLI, which embeds ../dist. A bare
// `cargo build --release` compiles the same Rust and bakes in `devUrl` instead, so the webview
// loads http://localhost:1420 — and with a Vite dev server up that binary passes every test while
// driving a frontend that is not the one in it. The `before` hook refuses such a binary by name;
// see the origin check there. This is not hypothetical, it is how the harness spent a day looking
// broken.
//
// Every repo path here is ABSOLUTE, because the launcher (`run.mjs`) changes the working
// directory into the sandbox before wdio starts — see sandbox.mjs for why that matters.

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  configDir,
  outDir,
  sandboxDir,
  webviewDir,
  isInsideSandbox,
  expectedProject,
} from './sandbox.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
export const repoRoot = path.resolve(here, '..')

// The release binary. Built by `npm run tauri build` — on the reference machine that must go
// through the vcvars 14.29 pin documented in CLAUDE.md "Dev commands", because toolset 14.50 is
// broken there. The harness does not build it; it refuses to run without it (see run.mjs), which
// keeps a slow, environment-specific compile out of the test path.
const appBinary = path.join(repoRoot, 'src-tauri', 'target', 'release', 'sandibumi.exe')

export const examplesDir = path.join(repoRoot, 'dataset for test', 'examples')

/**
 * Every spec file, as ONE GROUP — the nested array is load-bearing, not a style choice.
 *
 * wdio starts a fresh session per spec FILE, and for this service a session is a launch of
 * `sandibumi.exe`. With two spec files that meant launch, tear down, launch again against the
 * same project — and the intermediate teardown is not clean: the second launch found a WAL it
 * could not replay and `db::init_db_resilient` moved it aside as a `.corrupt-backup-*`. The
 * harness's own WAL check caught it, which is what that check is for.
 *
 * Wrapping the list in an inner array tells wdio to run every file in ONE session, so the app is
 * launched once and shut down once, exactly as it was when there was a single spec file. That is
 * also what `maxInstances: 1` below already intends: one app, one project, one DuckDB writer.
 *
 * The consequence for spec authors: specs SHARE one app and one project, and wdio does not
 * promise an order between files. Write each spec so it establishes what it needs (see the
 * `before` hook in wellgroups.e2e.mjs) and assert changes as before/after differences rather than
 * as absolute state — `pipeline.e2e.mjs` already computes VSH on every well, so "this well has no
 * VSH" is not a safe assertion for anyone who runs after it.
 */
const specFiles = fs
  .readdirSync(path.join(here, 'specs'))
  .filter((f) => f.endsWith('.e2e.mjs'))
  .sort()
  .map((f) => path.join(here, 'specs', f))

/**
 * The layer-3 verdict, carried from `before` to `beforeTest`.
 *
 * It starts as a REFUSAL rather than as null, so a `before` hook that never ran at all — a wdio
 * config edit, a hook renamed, a service that skips it — leaves the sandbox unverified and every
 * test red. Defaulting to "verified" would make deleting the guard look like passing it.
 */
let sandboxVerdict = new Error(
  'REFUSING TO RUN: the sandbox check never ran, so nothing has confirmed which project this ' +
    'app opened. See the `before` hook in e2e/wdio.conf.mjs.',
)

export const config = {
  runner: 'local',
  specs: [specFiles],
  maxInstances: 1, // One app, one project file, one DuckDB writer. Never parallelise this.
  framework: 'mocha',
  reporters: ['spec'],
  logLevel: 'warn',

  // A real module run over real curves is not fast, and a timeout here would end the run by
  // tearing the session down mid-write — the one thing this harness must never do.
  mochaOpts: { ui: 'bdd', timeout: 180_000 },

  services: [
    [
      '@wdio/tauri-service',
      {
        // Layer 1 of the sandbox: a throwaway config directory means the recents list the app
        // reads is empty, and the developer's real per-user list is never written to.
        env: {
          SANDIBUMI_CONFIG_DIR: configDir,
          // Layer 4: a private WebView2 profile, so `localStorage` is this run's own. Without it
          // the harness shares `%LOCALAPPDATA%\com.sandibumi.petro\EBWebView` with the developer's
          // real SandiBumi — and `autosave.ts` keeps its crash flag there, so ONE unclean shutdown
          // puts every later launch of BOTH into crash recovery, permanently. See sandbox.mjs.
          WEBVIEW2_USER_DATA_FOLDER: webviewDir,
          // Keep the engine's memory cap modest; the harness works on three synthetic wells and
          // has no business reserving gigabytes on the machine running it.
          SANDIBUMI_DB_MEMORY: '512MB',
        },
        // 'external' is the tauri-driver + msedgedriver route. The service's DEFAULT is
        // 'embedded', which runs a WebDriver server inside the app — but that requires
        // `tauri-plugin-wdio-webdriver` to be registered in lib.rs, i.e. a test-only plugin
        // compiled into the shipping binary. That is a change to the product to serve the
        // tests, and a decision to take deliberately rather than by accepting a default.
        driverProvider: 'external',
        // Both drivers are fetched at RUN time by the service, not by an npm install script,
        // which is what lets this work on a machine whose npm policy blocks install scripts.
        autoInstallTauriDriver: true,
        autoDownloadEdgeDriver: true,
        captureBackendLogs: true,
        logDir: outDir,
        startTimeout: 120_000,
      },
    ],
  ],

  capabilities: [
    {
      browserName: 'tauri',
      'tauri:options': { application: appBinary },
    },
  ],

  /**
   * Layer 3 of the sandbox, and the only layer that is a measurement rather than a setting.
   *
   * The app is asked which project it actually opened. If that is not inside this run's sandbox,
   * the run is aborted before a single test body executes — no import, no module run, no write.
   * Layers 1 and 2 are configuration and can be got wrong by an edit somewhere else; this check
   * cannot be got wrong by anything except the app lying about its own state.
   *
   * It does NOT enforce on its own, though — see `beforeTest`. An exception thrown here is LOGGED
   * by wdio and the run then continues into the test bodies, so a guard that lives only in this
   * hook is advisory. Observed: `window.__TAURI__` was momentarily undefined, this hook threw,
   * wdio printed the stack and ran all ten tests anyway. The verdict is therefore recorded and
   * re-thrown where wdio does treat a failure as fatal to the test.
   */
  before: async function () {
    try {
      // Wait until the app can actually be TALKED to, by making a real call — not by checking
      // that `window.__TAURI__` exists.
      //
      // Those are different conditions, and the difference is the whole reason this harness looked
      // broken for a day. `window.__TAURI__` is injected by an initialisation script, so it is
      // present before the webview has finished navigating to the app document; during that window
      // the document still sits on an opaque origin, and Tauri's IPC layer rejects every call with
      // "Origin header is not a valid URL". That message reads like a WebDriver transport fault and
      // is nothing of the kind — @wdio/tauri-service marshals a page-side error back through its
      // `__wdio_error__` channel and rethrows it, so the text belongs to Tauri, not to the driver.
      //
      // A liveness probe cannot be faked by an init script, so it is the honest precondition: the
      // tests need IPC to work, so wait for IPC to work.
      let lastProbe = 'no probe ran'
      try {
        await browser.waitUntil(
          async () => {
            const probe = await browser.execute(async () => {
              // The page's own identity comes back with every probe. "IPC is refused" and "the
              // webview is sitting on about:blank" look identical from outside and need completely
              // different fixes, and without this the difference costs an afternoon.
              const where =
                `url=${location.href} readyState=${document.readyState} ` +
                `title=${JSON.stringify(document.title)} bodyChildren=${document.body?.childElementCount ?? -1}`
              if (typeof window.__TAURI__?.core?.invoke !== 'function') {
                return `no window.__TAURI__ | ${where}`
              }
              try {
                await window.__TAURI__.core.invoke('current_project')
                return 'ok'
              } catch (err) {
                return `${String(err?.message ?? err)} | ${where}`
              }
            })
            lastProbe = probe
            return probe === 'ok'
          },
          { timeout: 60_000, interval: 250 },
        )
      } catch {
        // `timeoutMsg` is a fixed string, so the diagnosis is built here where the last probe's own
        // words are still in hand. Which case it was decides where to look next, and making the
        // harness say so beats making the next reader rediscover it.
        //
        // The dev-binary case is named FIRST because it is the one that wastes a day: a webview
        // parked on chrome-error:// with the title "localhost" is not a broken app, it is a binary
        // that was built with `cargo build --release` and is looking for a dev server that is not
        // there. The probe's own url is the evidence, so it is quoted rather than inferred.
        const devBinary = /chrome-error:|localhost:1420/.test(lastProbe)
        throw new Error(
          [
            `The app never answered IPC. Last probe said: ${lastProbe}`,
            '',
            ...(devBinary
              ? [
                  'That url is the signature of a binary built with `cargo build --release`, which',
                  "bakes in tauri.conf.json's devUrl instead of embedding ../dist. Rebuild with the",
                  'Tauri CLI — `npm run tauri build -- --no-bundle` — and do NOT work around it by',
                  'starting a dev server; the tests would then pass against a frontend that is not',
                  'the one in the binary. See finding 22 in docs/review_triage.md.',
                ]
              : [
                  '  "no window.__TAURI__" — check tauri.conf.json still sets withGlobalTauri, and',
                  '      that src-tauri/target/release/sandibumi.exe was built from this tree.',
                  '  "Origin header is not a valid URL" — the webview never finished navigating to',
                  '      the app document, so it is still on an opaque origin and Tauri is refusing',
                  '      IPC. That is a Tauri message surfaced through the driver, NOT a WebDriver',
                  '      fault: @wdio/tauri-service rethrows page-side errors verbatim.',
                ]),
          ].join('\n'),
        )
      }

      // REFUSE a binary that is not a production build.
      //
      // `cargo build --release` produces an exe that still points at `devUrl`
      // (http://localhost:1420) — only the Tauri CLI's own build embeds `frontendDist`. Such a
      // binary does not fail loudly: with a Vite dev server up it loads and every test passes,
      // while what was actually driven is the dev server's frontend rather than the one that
      // ships. This harness's whole claim is that it drives the real built app, so a run that
      // quietly tests something else is worse than no run. With no dev server the webview lands on
      // chrome-error:// instead, which is how this was finally caught.
      const page = await browser.execute(() => location.href)
      if (!page.startsWith('tauri://') && !page.startsWith('http://tauri.localhost')) {
        throw new Error(
          [
            `REFUSING TO RUN: the app is not serving its own embedded frontend (url: ${page}).`,
            '',
            'That means the binary was built with a bare `cargo build --release`, which bakes in',
            "tauri.conf.json's devUrl instead of embedding ../dist. Rebuild through the Tauri CLI:",
            '',
            '  npm run tauri build -- --no-bundle',
            '',
            'On the reference machine that must go through the vcvars 14.29 pin — see CLAUDE.md',
            '"Dev commands". Do not "fix" this by starting a dev server: the tests would then pass',
            'against a frontend that is not the one in the binary.',
          ].join('\n'),
        )
      }

      // `main.ts` must await the project-open gate before it builds any panel, so the harness
      // waits on the same gate rather than racing it with a sleep.
      await browser.execute(async () => {
        await window.__TAURI__.core.invoke('await_project_open')
      })

      const current = await browser.execute(async () =>
        window.__TAURI__.core.invoke('current_project'),
      )
      const openedPath = current?.path

      if (!isInsideSandbox(openedPath)) {
        throw new Error(
          [
            'REFUSING TO RUN: the app did not open a sandboxed project.',
            `  opened:   ${openedPath}`,
            `  expected: inside ${sandboxDir} (e.g. ${expectedProject})`,
            '',
            'This guard exists because SandiBumi falls back to the RELATIVE path "project.duckdb"',
            'when its recents list is empty, which resolves against the working directory. Running',
            'on regardless could import test data into a real project. Fix the sandbox, do not',
            'weaken this check.',
          ].join('\n'),
        )
      }
      sandboxVerdict = null
    } catch (err) {
      // Recorded rather than re-thrown here, because throwing here does not stop anything.
      sandboxVerdict = err instanceof Error ? err : new Error(String(err))
    }
  },

  /**
   * Where the layer-3 verdict is actually enforced.
   *
   * wdio treats a `beforeTest` failure as a failure OF THAT TEST, so an unverified sandbox turns
   * every test red with the reason printed, instead of a warning scrolling past above a green
   * suite. The check itself still lives in `before` — it runs once, against one app launch. This
   * only carries its answer to somewhere that cannot be ignored.
   */
  beforeTest: async function () {
    if (sandboxVerdict) throw sandboxVerdict
  },
}
