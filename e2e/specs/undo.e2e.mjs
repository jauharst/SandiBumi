// T-SHELL-13 (Undo/Redo with live labels) and T-SHELL-12 (the dirty ● indicator), driven through
// a real Database Inspector cell edit — which is the cheapest genuinely UNDOABLE action in the app.
//
// The claim is not "the buttons exist". It is that the undo stack's LABEL and DEPTH are readable
// before you press anything: the button's tooltip names what will be undone, and its disabled
// state says whether there is anything to undo at all. A user reaches for undo when they have
// just done something they regret, and an undo that does not say what it will reverse is one
// nobody dares press.
//
// The value round trip matters as much as the label: undo must put the OLD value back, and redo
// must put the new one back again. An undo that fires its callback without changing the data is
// indistinguishable from a working one until the next time someone reads the number.

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

/** Undo/redo button state, read from the Project tab's ribbon. */
const undoState = () =>
  browser.execute(() => {
    const u = document.querySelector('#undo-btn')
    const r = document.querySelector('#redo-btn')
    return {
      undoDisabled: !!u?.disabled,
      redoDisabled: !!r?.disabled,
      undoTitle: u?.title ?? '',
      redoTitle: r?.title ?? '',
      projectDirty: !!document
        .querySelector('.ribbon-tab[data-tab="project"]')
        ?.classList.contains('ribbon-tab-dirty'),
    }
  })

const ZONE = 'E2E-UNDO'

describe('undo, redo and the dirty marker (T-SHELL-13, T-SHELL-12)', () => {
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

    // A zone to edit. `zones` is one of the Inspector's editable tables and a zone row is small,
    // self-contained and easy to read back — unlike a curve sample, which needs a depth match.
    await invokeOk('upsert_zone', {
      wellId: well.well_id,
      zoneName: ZONE,
      topDepth: 1500,
      bottomDepth: 1600,
    })
  })

  after(async () => {
    await call('delete_zone', { wellId: well.well_id, zoneName: ZONE })
  })

  it('starts with a labelled, usable undo button after an edit', async () => {
    // Open the Database Inspector and put it on the zones table.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="data"]')?.click()
      document.querySelector('#db-inspector-btn')?.click()
    })
    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.dbinspector')),
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Database Inspector never opened' },
    )

    // The panel has ONE select — the table. The WELL comes from `appState.selectedWell`, so it is
    // chosen in the Wells tree rather than in this panel; a first attempt drove every select it
    // could find and set the table picker to a well id.
    await browser.execute((wellId) => {
      const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === wellId,
      )
      row?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
      const sel = document.querySelector('.dbinspector .dbi-table')
      if (sel) {
        const opt = Array.from(sel.options).find(
          (o) => o.value === 'zones' || /^zones$/i.test(o.textContent.trim()),
        )
        if (opt) {
          sel.value = opt.value
          sel.dispatchEvent(new Event('change', { bubbles: true }))
        }
      }
    }, well.well_id)

    const ready = await browser.waitUntil(
      async () =>
        await browser.execute(
          (zoneName) =>
            Array.from(document.querySelectorAll('.dbinspector td')).some(
              (td) => td.textContent.trim() === zoneName,
            ),
          ZONE,
        ),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: `the Inspector never showed the ${ZONE} row in the zones table`,
      },
    )
    assert.ok(ready)

    // Edit the zone's top depth: double-click the cell, type, Enter.
    const edited = await browser.execute((zoneName) => {
      const rows = Array.from(document.querySelectorAll('.dbinspector table tbody tr'))
      const row = rows.find((tr) =>
        Array.from(tr.querySelectorAll('td')).some((td) => td.textContent.trim() === zoneName),
      )
      if (!row) return null
      const cell = Array.from(row.querySelectorAll('td.editable')).find((td) =>
        /^\d+(\.\d+)?$/.test(td.textContent.trim()),
      )
      if (!cell) return null
      const before = cell.textContent.trim()
      cell.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
      const input = cell.querySelector('input.dbgrid-edit')
      if (!input) return null
      input.value = '1234'
      input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
      return before
    }, ZONE)
    assert.ok(edited !== null, 'could not edit a numeric cell on the zone row')

    await browser.waitUntil(async () => !(await undoState()).undoDisabled, {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: 'the undo button never became usable after an edit',
    })

    const st = await undoState()
    // The tooltip must NAME the action. "Undo (Ctrl+Z)" alone is the empty-stack wording; once
    // there is something to undo it has to say what, or the button is a dare rather than a tool.
    assert.match(
      st.undoTitle,
      /^Undo .+\(Ctrl\+Z\)$/,
      `the undo tooltip must name what it will reverse; it reads "${st.undoTitle}"`,
    )
    assert.ok(
      /edit/i.test(st.undoTitle),
      `and name the edit; it reads "${st.undoTitle}"`,
    )
    assert.ok(st.redoDisabled, 'redo must stay disabled until something has been undone')
  })

  it('undoes the value, not just the button state', async () => {
    const beforeUndo = await invokeOk('list_zones', { wellId: well.well_id })
    const mine = beforeUndo.find((z) => z.zone_name === ZONE)
    assert.ok(Math.abs(mine.top_depth - 1234) < 0.01, 'the edit must have reached the project first')

    await browser.execute(() => document.querySelector('#undo-btn')?.click())

    await browser.waitUntil(
      async () => {
        const zones = await invokeOk('list_zones', { wellId: well.well_id })
        const z = zones.find((x) => x.zone_name === ZONE)
        return z && Math.abs(z.top_depth - 1500) < 0.01
      },
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg:
          'undo did not restore the old value. An undo that fires without changing the data is ' +
          'indistinguishable from a working one until someone reads the number.',
      },
    )

    const st = await undoState()
    assert.ok(!st.redoDisabled, 'redo must become available once something has been undone')
    assert.match(
      st.redoTitle,
      /^Redo .+\(Ctrl\+Y\)$/,
      `the redo tooltip must name what it will reapply; it reads "${st.redoTitle}"`,
    )
  })

  it('redoes it again', async () => {
    await browser.execute(() => document.querySelector('#redo-btn')?.click())

    await browser.waitUntil(
      async () => {
        const zones = await invokeOk('list_zones', { wellId: well.well_id })
        const z = zones.find((x) => x.zone_name === ZONE)
        return z && Math.abs(z.top_depth - 1234) < 0.01
      },
      { timeout: 20_000, interval: 250, timeoutMsg: 'redo did not reapply the edit' },
    )
  })

  it('does NOT mark the project dirty for a data edit', async () => {
    // The inverse of T-SHELL-12, and it was worth getting wrong once to learn: this test first
    // asserted that the edit above SHOULD light the Project tab's ● . It does not, and that is
    // correct.
    //
    // `dirty.ts` tracks **named-save freshness** — workspace arrangement and log-view layout
    // edits, the things a Session captures. A Database Inspector edit goes straight to DuckDB and
    // is already persisted, so there is nothing unsaved about it. Marking it dirty would train the
    // user to ignore the dot, which is the only warning that their PANE ARRANGEMENT is unsaved.
    //
    // The dot's placement rule is checked here too, because it holds either way: it must be a
    // CLASS, never a text prefix — a tabstrip that reflows when work goes dirty shifts every other
    // tab under the cursor.
    const st = await undoState()
    assert.equal(
      st.projectDirty,
      false,
      'a data edit is already saved and must not be reported as unsaved work',
    )

    const tabText = await browser.execute(
      () => document.querySelector('.ribbon-tab[data-tab="project"]')?.textContent?.trim() ?? '',
    )
    assert.equal(
      tabText,
      'Project',
      `the dirty state must be a class, not a text prefix; the tab reads "${tabText}"`,
    )
  })
})
