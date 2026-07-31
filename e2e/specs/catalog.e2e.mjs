// T-MLEQ-16 — the Curve Catalog: rows, live search, header sorting.
//
// The catalog is where a user answers "what is actually in this well, which set does it live in,
// and which version am I looking at". Everything about the versioned write discipline
// (`log_set_versioning_never_overwrites`, the PK-less `computed_curves` contract) is only visible
// to a human THROUGH this table — so a catalog that filters wrongly, or sorts a column by the
// wrong key, hides exactly the state those rules exist to protect.
//
// The rows themselves come from the backend and are pinned there. What is checked here is the
// three things that exist only in the panel: that a row for a real curve appears at all, that the
// search box narrows to matching rows rather than merely highlighting them, and that a sortable
// header reorders the table rather than only drawing an arrow.

import assert from 'node:assert/strict'
import path from 'node:path'
import { examplesDir } from '../wdio.conf.mjs'

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

async function invokeOk(cmd, args) {
  const r = await call(cmd, args)
  assert.ok(r.ok, `${cmd} failed: ${r.error}`)
  return r.value
}

/** The catalog's visible rows: the first cell (curve name plus any badges) of each. */
const catalogNames = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.catalog-table tbody tr'))
      .map((tr) => tr.querySelector('td')?.textContent?.trim() ?? '')
      .filter((t) => t && !/^No curves yet$/i.test(t)),
  )

/** Type into the catalog's search box the way a user does, and let its input handler run. */
const setFilter = (text) =>
  browser.execute((t) => {
    const input = document.querySelector('#catalog-filter')
    if (!input) return false
    input.value = t
    input.dispatchEvent(new Event('input', { bubbles: true }))
    return true
  }, text)

describe('curve catalog (T-MLEQ-16)', () => {
  let well = null

  before(async () => {
    const existing = await invokeOk('list_wells')
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      await invokeOk('import_las_files', { paths, setName: 'E2E', attach: false })
    }
    const wells = await invokeOk('list_wells')
    wells.sort((a, b) => a.well_name.localeCompare(b.well_name))
    well = wells[0]

    // Make sure the well has at least one COMPUTED curve, so the catalog has something with a set
    // and a version to show. Re-running is harmless — it versions rather than overwrites, which is
    // the contract this panel exists to display.
    await invokeOk('run_workflow_module', {
      req: {
        module: 'vsh_gr',
        well_ids: [well.well_id],
        log_inputs: {},
        params: {},
        opts: {},
      },
    })

    // Open the panel FIRST, then select the well.
    //
    // Order matters and the wrong way round is quiet: with no active well the catalog renders a
    // static "standard + computed reference" placeholder — a plausible-looking table of GR,
    // RES_DEEP, NPHI, RHOB, DT, SP with no search box and no sortable headers. Selecting the well
    // before the panel exists means the panel misses the broadcast and keeps showing that, and a
    // test reading row text alone would think it was looking at a real catalog.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="data"]')?.click()
      document.querySelector('#open-inspector-btn')?.click()
    })

    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.catalog-table')),
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Curve Catalog panel never opened' },
    )

    // Make the Wells pane re-read before trying to click a row in it. Wells imported through
    // `invoke` do not notify the frontend, so the tree can still be showing "No wells ingested
    // yet" over a project with three wells — there would be no row to click and the failure would
    // look like a broken catalog rather than an empty tree. Dispatching `change` on the tree's own
    // group select is the real user path whose handler ends in `refresh()`; see wells.e2e.mjs.
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (!sel) return
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })
    await browser.waitUntil(
      async () =>
        (await browser.execute(() => document.querySelectorAll('.tree-node.tree-well').length)) > 0,
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Wells pane never listed any wells' },
    )

    await browser.execute((id) => {
      const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === id,
      )
      row?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    }, well.well_id)

    // Wait for the REAL catalog, identified by its search box rather than by row count — the
    // placeholder has rows too, which is exactly the trap above.
    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('#catalog-filter')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg:
          'the Curve Catalog never left its no-well placeholder — the panel did not pick up the ' +
          'selected well',
      },
    )
  })

  after(async () => {
    // Leave the search box empty. The filter is panel state, not project state, but a later spec
    // reading this table would otherwise see a filtered view and no explanation for it.
    await setFilter('')
  })

  it('lists the well’s curves, including a computed one with its set and version', async () => {
    const names = await catalogNames()
    assert.ok(names.length > 0, 'the catalog must list rows for a well that has curves')
    assert.ok(
      names.some((n) => /^GR\b/.test(n)),
      `a raw curve every example well carries must be listed; got: ${names.slice(0, 12).join(' | ')}`,
    )
    assert.ok(
      names.some((n) => /VSH/i.test(n)),
      `the computed VSH curve must be listed; got: ${names.slice(0, 12).join(' | ')}`,
    )

    // The set/version column is the visible half of the versioned write discipline: a re-run must
    // become v2 rather than overwrite v1. Assert the column carries a version marker at all — the
    // arithmetic of versioning is pinned in Rust by `log_set_versioning_never_overwrites`, but a
    // panel that stopped SHOWING it would leave a user unable to tell which run they are reading.
    const hasVersion = await browser.execute(() =>
      Array.from(document.querySelectorAll('.catalog-table tbody tr')).some((tr) =>
        /v\d+/.test(tr.textContent ?? ''),
      ),
    )
    assert.ok(hasVersion, 'a computed row must show which version of its set it belongs to')
  })

  it('narrows the table to matching rows as you type', async () => {
    const all = await catalogNames()
    assert.ok(await setFilter('VSH'), 'no #catalog-filter search box in the panel')

    await browser.waitUntil(async () => (await catalogNames()).length < all.length, {
      timeout: 10_000,
      interval: 200,
      timeoutMsg: 'typing in the catalog search never reduced the row count',
    })

    const shown = await catalogNames()
    // Every remaining row must MATCH. A filter that merely highlights, or that drops the wrong
    // rows, still shrinks the table — so the count alone would not catch it.
    assert.ok(
      shown.length > 0 && shown.every((n) => /VSH/i.test(n)),
      `every row left after filtering must match the query; got: ${shown.join(' | ')}`,
    )

    // And clearing it must bring everything back — a filter that cannot be undone is a panel
    // stuck showing a subset, which reads exactly like a well that lost its curves.
    await setFilter('')
    await browser.waitUntil(async () => (await catalogNames()).length === all.length, {
      timeout: 10_000,
      interval: 200,
      timeoutMsg: 'clearing the catalog search did not restore every row',
    })
  })

  it('reorders the table when a sortable header is clicked', async () => {
    const before = await catalogNames()
    assert.ok(before.length > 1, 'sorting needs more than one row to be observable')

    // Click the first sortable header twice: once to sort by it, once to reverse. The second click
    // is the real assertion — the first could coincide with the order the rows already had.
    const clickSort = () =>
      browser.execute(() => {
        const th = document.querySelector('.catalog-table .catalog-sortable')
        if (!th) return null
        th.click()
        return th.dataset.sort
      })

    const key = await clickSort()
    assert.ok(key, 'the catalog must have at least one sortable column header')
    await browser.waitUntil(async () => (await catalogNames()).length === before.length, {
      timeout: 10_000,
      interval: 200,
      timeoutMsg: 'sorting must not add or drop rows',
    })
    const ascending = await catalogNames()

    await clickSort()
    await browser.waitUntil(
      async () => {
        const now = await catalogNames()
        return now.length === ascending.length && now.join('|') !== ascending.join('|')
      },
      {
        timeout: 10_000,
        interval: 200,
        timeoutMsg:
          `clicking the "${key}" header a second time did not reverse the order — a header that ` +
          'only draws an arrow looks sorted without being sorted',
      },
    )

    const descending = await catalogNames()
    assert.deepEqual(
      [...descending].reverse(),
      ascending,
      'the second click must reverse the first ordering exactly, not reshuffle it',
    )
  })
})
