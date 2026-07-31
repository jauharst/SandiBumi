// The sandbox is the whole safety story of this harness, so it lives in one file.
//
// SandiBumi picks its startup project from `project::startup_path()`: the first entry in the
// per-user recents list that still exists on disk, and failing that the LEGACY_DEFAULT, which is
// the RELATIVE path "project.duckdb" — resolved against the process working directory.
//
// That relative fallback is the danger. `src-tauri/project.duckdb` is a real working project on a
// developer machine, so a harness launched with the wrong cwd would import test LAS files straight
// into live data. Three independent layers keep that from being possible, and no single one of
// them is trusted on its own:
//
//   1. SANDIBUMI_CONFIG_DIR points at a throwaway config dir, so the recents list is EMPTY and
//      also so the real per-user list is never written to.
//   2. The process working directory is the sandbox, so the LEGACY_DEFAULT fallback resolves to
//      <sandbox>/project.duckdb rather than to any real one.
//   3. After launch the harness ASKS the running app which project it opened (`current_project`)
//      and aborts the entire run if the answer is not inside the sandbox. Layers 1 and 2 are
//      configuration and can be got wrong; layer 3 is a measurement and cannot.
//
// Nothing here deletes anything outside the sandbox directory it created itself.

import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

/**
 * Absolute path of this run's sandbox.
 *
 * The launcher publishes it through the environment and every later process reads it back. It is
 * deliberately NOT recomputed per process: wdio runs specs in a WORKER, so a name derived from
 * `process.pid` gives the launcher and the spec two different answers — the app then opens the
 * launcher's sandbox while the spec checks a directory that was never created, and the safety
 * assertion fails on a run that was actually safe. A guard that cries wolf gets weakened, so it
 * has to be right about the ordinary case.
 */
export const sandboxDir =
  process.env.SANDIBUMI_E2E_SANDBOX ?? path.join(os.tmpdir(), `sandibumi-e2e-${process.pid}`)

/** Name of the variable above, so the launcher and this file cannot disagree about it. */
export const SANDBOX_ENV = 'SANDIBUMI_E2E_SANDBOX'

export const configDir = path.join(sandboxDir, 'cfg')
export const outDir = path.join(sandboxDir, 'out')

/** Where the app will land given layers 1 and 2 above. Layer 3 checks it actually did. */
export const expectedProject = path.join(sandboxDir, 'project.duckdb')

export function createSandbox() {
  fs.rmSync(sandboxDir, { recursive: true, force: true })
  fs.mkdirSync(configDir, { recursive: true })
  fs.mkdirSync(outDir, { recursive: true })
  return sandboxDir
}

/**
 * Remove the sandbox. Best-effort on purpose: Windows will not delete a DuckDB file whose handle
 * is still open, and a sandbox left behind in the temp directory is a great deal less harmful
 * than a teardown that starts killing processes to win the race. It is reported, not forced.
 */
export function removeSandbox() {
  try {
    fs.rmSync(sandboxDir, { recursive: true, force: true })
    return { removed: true }
  } catch (err) {
    return { removed: false, reason: String(err?.message ?? err) }
  }
}

/**
 * Remove sandboxes left by earlier runs.
 *
 * Windows will not delete an open DuckDB file, and the app's handle is often still closing when
 * the launcher exits — so `removeSandbox` legitimately fails and the directory survives. Rather
 * than wait (or kill something to win the race) the next run sweeps them, by which time the
 * handles are long gone. Only directories matching this harness's own prefix inside the system
 * temp directory are considered, and the current run's sandbox is never touched.
 */
export function sweepOldSandboxes() {
  const tmp = os.tmpdir()
  let removed = 0
  let entries = []
  try {
    entries = fs.readdirSync(tmp)
  } catch {
    return 0
  }
  for (const name of entries) {
    if (!name.startsWith('sandibumi-e2e-')) continue
    const full = path.join(tmp, name)
    if (path.resolve(full) === path.resolve(sandboxDir)) continue
    try {
      fs.rmSync(full, { recursive: true, force: true })
      removed += 1
    } catch {
      // Still locked, or not ours to remove. Leave it; a stale temp directory is harmless.
    }
  }
  return removed
}

/** True when `p` is inside the sandbox — the layer-3 check. Case-insensitive for Windows. */
export function isInsideSandbox(p) {
  if (!p) return false
  const rel = path.relative(sandboxDir.toLowerCase(), path.resolve(p).toLowerCase())
  return rel !== '' && !rel.startsWith('..') && !path.isAbsolute(rel)
}

/**
 * The WAL check. `db::init_db_resilient` moves a WAL it cannot replay aside as a timestamped
 * `.corrupt-backup-*` file rather than letting the app panic on startup. Those files are
 * therefore the visible evidence that something killed the app mid-write — which is the exact
 * failure this harness is forbidden from causing. The run counts them before and after and
 * fails if the number went up, so a teardown regression cannot pass quietly.
 */
export function countCorruptBackups(repoRoot) {
  const dirs = [path.join(repoRoot, 'src-tauri'), sandboxDir]
  let found = []
  for (const dir of dirs) {
    let entries = []
    try {
      entries = fs.readdirSync(dir)
    } catch {
      continue
    }
    found = found.concat(
      entries.filter((f) => f.includes('.corrupt-backup-')).map((f) => path.join(dir, f)),
    )
  }
  return found
}
