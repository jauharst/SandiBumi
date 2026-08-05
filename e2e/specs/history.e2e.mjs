// T-SHELL-15 — Processing History attribution: which well a module run is recorded against.
//
// The contract is stated in `workspace.ts` and it is sharper than it looks: a single-well run is
// attributed BY NAME, a genuine multi-well batch is attributed to no well at all, and neither is
// ever the globally "selected" well — because a scoped run may not have touched that well.
//
// That last clause is the whole test. Attributing to the selected well would be right most of the
// time and silently wrong exactly when it matters: the History is what a user reads months later
// to answer "where did this curve come from", and a row naming a well the run never covered is a
// wrong answer that looks authoritative.

import assert from 'node:assert/strict'
import path from 'node:path'
import { examplesDir } from '../wdio.conf.mjs'

async function call(cmd, args) {
  try {
    return await browser.execute(
      async (c, a) => {
        try {
          return { ok: true, value: await window.__TAURI__.core.invoke(c, a ?? {}) }
        } catch (err) {
          return { ok: false, error: String(err?.message ?? err) }
        }
      },
      cmd,
      args,
    )
  } catch (err) {
    return { ok: false, error: String(err?.message ?? err) }
  }
}

async function invokeOk(cmd, args) {
  const r = await call(cmd, args)
  assert.ok(r.ok, `${cmd} failed: ${r.error}`)
  return r.value
}

/** The newest History rows, newest first, as displayed. */
const historyRows = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.history-row')).map(
      (r) => r.querySelector('.history-detail')?.textContent?.trim() ?? '',
    ),
  )

/** The module pane's result line. */
const resultText = () =>
  browser.execute(() => {
    const els = document.querySelectorAll('.module-pane .modal-result')
    return els.length ? els[els.length - 1].textContent.trim() : ''
  })

const setMode = (label) =>
  browser.execute((t) => {
    const panes = document.querySelectorAll('.module-pane')
    const pane = panes[panes.length - 1]
    const btn = Array.from(pane?.querySelectorAll('.well-scope-mode') ?? []).find(
      (b) => (b.textContent ?? '').trim() === t,
    )
    if (!btn) return false
    btn.click()
    return true
  }, label)

const runModule = () =>
  browser.execute(() => {
    const panes = document.querySelectorAll('.module-pane')
    const pane = panes[panes.length - 1]
    pane?.querySelector('.form-run-btn')?.click()
  })

describe('history attribution (T-SHELL-15)', () => {
  let wells = []

  before(async () => {
    const existing = await invokeOk('list_wells')
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      await invokeOk('import_las_files', { paths, setName: 'E2E', attach: false })
    }
    wells = await invokeOk('list_wells')
    wells.sort((a, b) => a.well_name.localeCompare(b.well_name))
    assert.ok(wells.length >= 3, `need at least 3 wells, found ${wells.length}`)

    await call('set_active_well_group', { groupId: null })
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (sel) {
        sel.value = ''
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }
      document.querySelector('.ribbon-tab[data-tab="project"]')?.click()
      document.querySelector('#history-btn')?.click()
    })

    await browser.waitUntil(
      async () =>
        (await browser.execute(() => document.querySelectorAll('.tree-node.tree-well').length)) >= 3,
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Wells pane never listed the wells' },
    )

    // A module pane to run from.
    const modules = await invokeOk('list_modules')
    const spec = modules.find((m) => m.name === 'vsh_gr')
    await browser.execute((title) => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
      const buttons = Array.from(
        document.querySelectorAll('.ribbon-panel[data-panel="petro"] .ribbon-btn'),
      )
      for (const b of buttons) {
        b.click()
        const item = Array.from(
          document.querySelectorAll('.ribbon-menu:not([hidden]) .ribbon-menu-item'),
        ).find((i) => (i.textContent ?? '').trim().includes(title))
        if (item) {
          item.click()
          return
        }
      }
      document
        .querySelectorAll('.ribbon-menu:not([hidden])')
        .forEach((m) => m.setAttribute('hidden', ''))
    }, spec.title)

    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.module-pane .form-run-btn')),
      { timeout: 30_000, interval: 500, timeoutMsg: 'no module pane to run from' },
    )
  })

  after(async () => {
    await setMode('All')
  })

  it('attributes a single-well run to the well that RAN, not the one selected', async () => {
    const selected = wells[0]
    const target = wells[1]

    // Select well A globally, then scope the run to well B alone. This is the case the contract
    // exists for: a row naming A would be wrong about rock A was never touched for.
    await browser.execute(
      (selId, tgtId) => {
        const rows = Array.from(document.querySelectorAll('.tree-node.tree-well'))
        const find = (id) => rows.find((n) => (n.title ?? '').split('\n')[0] === id)
        // Plain click on A: activates it and clears any multi-selection.
        find(selId)?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
        // Ctrl-click B: adds it to the multi-selection WITHOUT moving the active well.
        find(tgtId)?.dispatchEvent(
          new MouseEvent('click', { bubbles: true, cancelable: true, ctrlKey: true }),
        )
      },
      selected.well_id,
      target.well_id,
    )

    assert.ok(await setMode('Selection'), 'no Selection scope mode')
    await browser.waitUntil(
      async () =>
        /\b1\b/.test(
          await browser.execute(() => {
            const els = document.querySelectorAll('.module-pane .well-scope-count')
            return els.length ? els[els.length - 1].textContent.trim() : ''
          }),
        ),
      { timeout: 15_000, interval: 250, timeoutMsg: 'the scope never resolved to the single well' },
    )

    await runModule()
    await browser.waitUntil(async () => /computed/i.test(await resultText()), {
      timeout: 60_000,
      interval: 500,
      timeoutMsg: `the run never reported; the pane says: ${await resultText()}`,
    })

    await browser.waitUntil(
      async () => (await historyRows()).some((r) => /^.*: Ran /.test(r)),
      { timeout: 20_000, interval: 250, timeoutMsg: 'no module run appeared in the History' },
    )

    const newest = (await historyRows()).find((r) => / Ran /.test(r))
    assert.ok(
      newest.startsWith(`${target.well_name}:`),
      `the History row must name the well that RAN (${target.well_name}); it reads: ${newest}`,
    )
    assert.ok(
      !newest.startsWith(`${selected.well_name}:`),
      `and must NOT name the globally selected well (${selected.well_name}); it reads: ${newest}`,
    )
  })

  it('attributes a multi-well batch to no single well, and says how many', async () => {
    assert.ok(await setMode('All'))
    await runModule()
    await browser.waitUntil(async () => /computed/i.test(await resultText()), {
      timeout: 60_000,
      interval: 500,
      timeoutMsg: `the batch run never reported; the pane says: ${await resultText()}`,
    })

    await browser.waitUntil(
      async () => (await historyRows()).some((r) => /Ran .* on \d+ wells/.test(r)),
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg: `no batch row in the History; newest rows: ${(await historyRows()).slice(0, 3).join(' | ')}`,
      },
    )

    const batch = (await historyRows()).find((r) => /Ran .* on \d+ wells/.test(r))
    // A batch row must carry NO well prefix. Naming one of the wells would read as "this run was
    // about that well", which is exactly the wrong answer when the run covered the whole field —
    // and the other wells' curves would then have no History entry pointing at them at all.
    for (const w of wells) {
      assert.ok(
        !batch.startsWith(`${w.well_name}:`),
        `a multi-well row must not be attributed to one well; it reads: ${batch}`,
      )
    }
    assert.match(
      batch,
      new RegExp(`on ${wells.length} wells`),
      `the batch row must state how many wells it covered; it reads: ${batch}`,
    )
  })
})
