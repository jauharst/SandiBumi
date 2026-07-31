// Launcher for the end-to-end harness. `npm run test:e2e` runs this, never wdio directly.
//
// It exists for one reason that cannot be expressed in wdio's own config: @wdio/tauri-service
// has no `cwd` option, so the app inherits the working directory of the process that launched
// it — and SandiBumi's startup fallback is the RELATIVE path "project.duckdb". Launching wdio
// from the repo would therefore point the app at a real project. This runs wdio with its
// working directory set to a throwaway sandbox instead, which is layer 2 of the three described
// in sandbox.mjs.
//
// It also brackets the run with the WAL check, so a teardown that ever starts killing the app
// fails the run instead of quietly corrupting a database.

import { spawn, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  createSandbox,
  removeSandbox,
  sandboxDir,
  countCorruptBackups,
  sweepOldSandboxes,
  SANDBOX_ENV,
} from './sandbox.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..')
const appBinary = path.join(repoRoot, 'src-tauri', 'target', 'release', 'sandibumi.exe')
const wdioConfig = path.join(here, 'wdio.conf.mjs')

function fail(lines) {
  console.error(`\n${lines.join('\n')}\n`)
  process.exit(1)
}

// The harness deliberately does not build the app. A Tauri release build on the reference
// machine has to go through the vcvars 14.29 pin (CLAUDE.md "Dev commands") and takes minutes;
// burying that inside a test run makes a slow toolchain problem look like a test failure.
if (!fs.existsSync(appBinary)) {
  fail([
    'No release binary to test.',
    `  expected: ${appBinary}`,
    '',
    'Build it first. On the reference machine that must go through the vcvars 14.29 pin:',
    '  see CLAUDE.md "Dev commands" for the full command, then re-run this.',
  ])
}

/**
 * Clear driver processes orphaned by an earlier run.
 *
 * Each run spawns a tauri-driver and an msedgedriver; if a run ends badly they survive, and the
 * next run then fails with "tauri-driver exited unexpectedly (code: 1)" because the port is
 * taken. Observed, not hypothesised — two runs left four orphans behind.
 *
 * THE DISTINCTION THAT MATTERS: these are DRIVERS, not the application. They hold no DuckDB
 * connection and no write-ahead log, so ending one cannot corrupt a project. The prohibition on
 * force-killing is about `sandibumi.exe` mid-write, and this function must never touch it — so
 * it refuses to run at all while the app is up, which also stops the harness from stamping on a
 * session the developer has open.
 *
 * It runs BEFORE the tests, never after: at teardown a driver may still be shutting the app
 * down, and reaping there would create exactly the unclean kill this harness must avoid.
 */
function reapOrphanDrivers() {
  if (process.platform !== 'win32') return
  const running = (name) =>
    spawnSync('tasklist', ['/FI', `IMAGENAME eq ${name}`, '/NH'], { encoding: 'utf8' })
      .stdout?.includes(name) ?? false

  if (running('sandibumi.exe')) {
    fail([
      'SandiBumi is already running.',
      '',
      'The harness will not start while the app is up: it cannot tell your session apart from',
      'an orphan, and it must never terminate the application. Close SandiBumi and re-run.',
    ])
  }

  for (const name of ['tauri-driver.exe', 'msedgedriver.exe']) {
    if (running(name)) {
      console.log(`clearing orphaned ${name} from a previous run`)
      spawnSync('taskkill', ['/F', '/IM', name], { stdio: 'ignore' })
    }
  }
}

reapOrphanDrivers()

const before = countCorruptBackups(repoRoot)
createSandbox()
const swept = sweepOldSandboxes()
if (swept > 0) console.log(`swept ${swept} sandbox(es) left by earlier runs`)
console.log(`e2e sandbox: ${sandboxDir}`)

// Run wdio FROM the sandbox. Everything it needs from the repo is passed as an absolute path.
//
// wdio's JS entry point is invoked with this same node binary rather than through `npx`. On
// Windows `npx` is a .cmd shim, and Node refuses to spawn one without `shell: true` (EINVAL) —
// and turning the shell on would push an absolute path containing a space ("D:\XX. SandiBumi")
// through the command-line parser. Calling the entry point directly avoids both.
const wdioBin = path.join(repoRoot, 'node_modules', '@wdio', 'cli', 'bin', 'wdio.js')
if (!fs.existsSync(wdioBin)) {
  fail([
    'WebdriverIO is not installed.',
    `  expected: ${wdioBin}`,
    '',
    'Run `npm install` in the repo root, then re-run this.',
  ])
}

const child = spawn(process.execPath, [wdioBin, 'run', wdioConfig], {
  cwd: sandboxDir,
  stdio: 'inherit',
  shell: false,
  // Publish the sandbox path so the wdio WORKER resolves the same directory this launcher
  // created. Without it each process derives its own name from its own pid, and the safety
  // assertion compares the app's real (correct) project against a path that never existed.
  env: { ...process.env, [SANDBOX_ENV]: sandboxDir },
})

// No timeout, and no kill path anywhere in this file. An unclean kill mid-write is exactly the
// failure that produces a corrupt DuckDB WAL, so a hung run is left for a human to look at and
// end deliberately. A harness that tidies up by killing the thing it is testing is worse than
// one that hangs visibly.
child.on('exit', (code) => {
  const after = countCorruptBackups(repoRoot)
  const fresh = after.filter((f) => !before.includes(f))

  if (fresh.length > 0) {
    console.error(
      [
        '',
        'WAL CORRUPTION DETECTED — the app did not shut down cleanly.',
        ...fresh.map((f) => `  ${f}`),
        '',
        'db::init_db_resilient moves an unreplayable WAL aside like this when something killed',
        'the app mid-write. Do not delete these and re-run: find what is terminating the app.',
      ].join('\n'),
    )
    // The sandbox is left in place on this path — it is evidence.
    process.exit(1)
  }

  const cleanup = removeSandbox()
  if (!cleanup.removed) {
    console.warn(`note: sandbox left behind (${cleanup.reason}): ${sandboxDir}`)
  }
  process.exit(code ?? 1)
})
