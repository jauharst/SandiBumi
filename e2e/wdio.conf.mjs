// WebdriverIO configuration for the SandiBumi end-to-end harness.
//
// This drives the REAL BUILT DESKTOP APP through Tauri's WebDriver route (tauri-driver +
// msedgedriver, managed by @wdio/tauri-service). It deliberately does NOT use the service's
// "browser mode", which runs the frontend in plain Chrome against a Vite dev server: in that
// mode `window.__TAURI__` is absent, every `invoke` fails, and no module run, no DuckDB write
// and no export is ever exercised. Proving those is the entire reason this harness exists.
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
   */
  before: async function () {
    // `main.ts` must await the project-open gate before it builds any panel, so the harness waits
    // on the same gate rather than racing it with a sleep.
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
  },
}
