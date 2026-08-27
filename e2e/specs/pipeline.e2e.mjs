// The proving run: import, compute, export — against the real desktop app, the real Rust
// backend and a real DuckDB file.
//
// These tests reach the backend through `window.__TAURI__.core.invoke`, which is live here
// because tauri.conf.json sets `withGlobalTauri: true`. That is a deliberate design choice, not
// a shortcut. The native file picker is an OS window WebDriver cannot see, but the picker only
// COLLECTS paths — `import_las_files(paths, …)` and `export_las(well_id, dest_path)` take them
// as arguments. Driving the commands exercises the whole of what this harness can honestly
// verify: IPC, the module engine, the write discipline and the file that lands on disk. What it
// skips is the picker chrome, and the docs say so rather than pretending otherwise.
//
// The last test is DOM-level, to prove the real UI actually booted rather than only the backend.

import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { examplesDir } from '../wdio.conf.mjs'
import { outDir, isInsideSandbox } from '../sandbox.mjs'

/** Call a backend command in the running app and return its result. */
const call = (cmd, args) =>
  browser.execute(
    async (c, a) => {
      try {
        return { ok: true, value: await window.__TAURI__.core.invoke(c, a ?? {}) }
      } catch (err) {
        return { ok: false, error: String(err) }
      }
    },
    cmd,
    args,
  )

/** Call and fail the test with the backend's own message if it errored. */
async function invokeOk(cmd, args) {
  const r = await call(cmd, args)
  assert.ok(r.ok, `${cmd} failed: ${r.error}`)
  return r.value
}

const SET_NAME = 'E2E'

describe('SandiBumi end-to-end', () => {
  it('opened a sandboxed project, and it is empty', async () => {
    // The before hook already refused to continue otherwise; this states it as a test so a
    // regression in the guard shows up as a named failure rather than as a silent skip.
    const current = await invokeOk('current_project')
    assert.ok(
      isInsideSandbox(current.path),
      `must be sandboxed, opened: ${current.path}`,
    )

    const wells = await invokeOk('list_wells', { scope: { kind: 'all' } })
    assert.equal(wells.length, 0, 'a fresh sandbox project must start with no wells')
  })

  it('imports the example LAS files as a named set', async () => {
    const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
      path.join(examplesDir, f),
    )
    for (const p of paths) {
      assert.ok(fs.existsSync(p), `example data missing: ${p}`)
    }

    // The import refuses without a declared sampling style + step tolerance (see
    // despike.e2e.mjs); 0.01 m is a test input for the synthetic examples, not a field value.
    const results = await invokeOk('import_las_files', {
      paths,
      setName: SET_NAME,
      attach: false,
      samplingStyle: 'CONTINUOUS_REGULAR',
      samplingStyleVerifyTolerance: { value: 0.01, unit: 'M' },
    })
    assert.equal(results.length, 3, 'one result per file')
    // A refused file is not an invoke error — the command succeeds and reports per file.
    for (const r of results) {
      assert.ok(r.well_id != null, `import failed for ${r.path}: ${r.error}`)
    }

    const wells = await invokeOk('list_wells', { scope: { kind: 'all' } })
    assert.equal(wells.length, 3, 'three wells after importing three files')
    const names = wells.map((w) => w.well_name).sort()
    assert.deepEqual(names, ['SANDI-01', 'SANDI-02', 'SANDI-03'])
  })

  it('runs a module and writes curves into the project', async () => {
    const wells = await invokeOk('list_wells', { scope: { kind: 'all' } })
    const wellIds = wells.map((w) => w.well_id)

    // vsh_gr is the cheapest real module with a real numeric output: it needs only GR, which
    // every example well carries.
    // A run now declares its custody (operator + source note; AUTOMATED names the harness
    // honestly) and resolves its wells from a backend scope selector.
    const results = await invokeOk('run_workflow_module', {
      req: {
        module: 'vsh_gr',
        well_ids: wellIds,
        log_inputs: {},
        // The GR endpoints ship with no default (provenance rule — an endpoint is basin-specific).
        // 45/110 are the synthetic generator's own clean-sand and cap-shale GR means
        // (tools/make_example_data.py ZONES), not a recommended field calibration.
        params: { GR_MA: 45, GR_SH: 110 },
        opts: {},
        custody: {
          actor: { kind: 'AUTOMATED', identity: 'e2e-harness' },
          source_note: 'e2e fixture run; manifest defaults, no explicit values',
        },
      },
      scope: { kind: 'explicit', well_ids: wellIds },
    })
    assert.equal(results.length, 3, 'one result per well')
    for (const r of results) {
      assert.ok(!r.error, `${r.well_name ?? r.well_id}: ${r.error}`)
    }

    // The claim is not "the command returned" but "the curve is in the project" — so read it
    // back from the catalog rather than trusting the return value.
    const catalog = await invokeOk('list_computed_catalog', { wellId: wellIds[0] })
    const produced = catalog.map((c) => (c.name ?? c.curve_name ?? '').toUpperCase())
    assert.ok(
      produced.some((n) => n.startsWith('VSH')),
      `a VSH curve must be in the computed catalog; got: ${produced.join(', ')}`,
    )
  })

  it('exports a LAS file that actually appears on disk', async () => {
    const wells = await invokeOk('list_wells', { scope: { kind: 'all' } })
    const well = wells[0]
    const dest = path.join(outDir, `${well.well_name}-export.las`)

    const written = await invokeOk('export_las', {
      wellId: well.well_id,
      destPath: dest,
    })

    // export_las returns a full LasExportResult now; `rows` is the row count the old command
    // returned bare.
    assert.ok(fs.existsSync(dest), `export_las reported ${written.rows} rows but wrote no file at ${dest}`)
    const text = fs.readFileSync(dest, 'utf8')

    // The section headers as export.rs actually writes them ("~Version Information", not the
    // all-caps "~VERSION" a LAS spec is usually quoted with). Asserting the real output rather
    // than the convention is the point: this assertion was wrong on the first run and the file
    // was fine, which is exactly the direction a harness should fail in.
    for (const marker of ['~Version Information', '~Well Information', '~Curve Information', '~ASCII']) {
      assert.ok(text.includes(marker), `exported LAS is missing the ${marker} section`)
    }
    assert.ok(text.includes(well.well_name), 'the exported LAS must name the well it came from')

    // A header with no data under it is the failure mode worth catching: the ~ASCII section
    // must be followed by at least one numeric row.
    const body = text.split('~ASCII')[1] ?? ''
    const dataRows = body.split('\n').filter((l) => /^\s*-?\d/.test(l))
    assert.ok(dataRows.length > 0, 'the exported LAS has a header but no data rows')
    assert.equal(
      dataRows.length,
      written.rows,
      `export_las reported ${written.rows} rows but the file holds ${dataRows.length}`,
    )
  })

  it('booted the real UI, not just the backend', async () => {
    // Cheap but load-bearing: it distinguishes "the Rust side answered IPC" from "the frontend
    // actually came up".
    //
    // The assertion is that the ribbon has been POPULATED, not that some particular button
    // exists. `#ribbon` and `#dock-root` are empty containers in index.html, so their mere
    // presence proves only that static HTML parsed; children appear only once main.ts has run
    // past the project-open gate. Naming a specific button id instead would tie this test to
    // one version of the markup — and the harness drives a BUILT binary, which embeds whatever
    // frontend existed when it was compiled, so the DOM here can legitimately lag the repo.
    const ribbon = await $('#ribbon')
    await ribbon.waitForExist({ timeout: 30_000 })

    const built = await browser.execute(() => ({
      ribbonChildren: document.querySelector('#ribbon')?.childElementCount ?? 0,
      dockChildren: document.querySelector('#dock-root')?.childElementCount ?? 0,
    }))
    assert.ok(built.ribbonChildren > 0, 'main.ts must have populated the ribbon')
    assert.ok(built.dockChildren > 0, 'the dockview workspace must have been created')
  })
})
