// T-WELL-03 (a multi-selection feeds a batch pane's "Selection" scope, live) and T-AUX-15 (pinned
// wells as a one-click run scope).
//
// This is the last link in the chain the other specs cover from both ends: `wells.e2e.mjs` proves
// the Wells pane builds the right SET, `wellgroups.e2e.mjs` proves a run honours the scope it is
// given. What is left is whether the pane in between resolves the set the user is looking at —
// and getting that wrong does not fail, it runs on the wrong wells and writes real curves.
//
// The assertion throughout is the RESOLVED WELL COUNT the pane displays, because that is the
// number a user reads before pressing Run. A mode that highlights correctly while resolving to
// something else is the failure worth catching.

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

/** The LAST module pane's scope readout — several panes can be open at once. */
const scopeCount = () =>
  browser.execute(() => {
    const els = document.querySelectorAll('.module-pane .well-scope-count')
    return els.length ? els[els.length - 1].textContent.trim() : '(no scope readout)'
  })

/** Click a scope mode button by its label, in the last module pane. */
const setMode = (label) =>
  browser.execute((t) => {
    const panes = document.querySelectorAll('.module-pane')
    const pane = panes[panes.length - 1]
    if (!pane) return false
    const btn = Array.from(pane.querySelectorAll('.well-scope-mode')).find(
      (b) => (b.textContent ?? '').trim() === t,
    )
    if (!btn) return false
    btn.click()
    return true
  }, label)

/** Wait for the scope readout to name `n` wells, then return it. */
async function expectScope(n, message) {
  let last = ''
  try {
    await browser.waitUntil(
      async () => {
        last = await scopeCount()
        return new RegExp(`\\b${n}\\b`).test(last)
      },
      { timeout: 15_000, interval: 250 },
    )
  } catch {
    assert.fail(`${message} — the pane reads "${last}", expected ${n}`)
  }
  return last
}

/** Click a well row's ★ by well id — the only path that also updates `appState.pinnedWellIds`. */
const clickStar = (wellId) =>
  browser.execute((id) => {
    const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
      (n) => (n.title ?? '').split('\n')[0] === id,
    )
    const star = row?.querySelector('.tree-pin')
    if (!star) return false
    star.click()
    return true
  }, wellId)

const clickWell = (index, mods = {}) =>
  browser.execute(
    (i, m) => {
      const rows = Array.from(document.querySelectorAll('.tree-node.tree-well'))
      const row = rows[i]
      if (!row) return false
      row.dispatchEvent(
        new MouseEvent('click', {
          bubbles: true,
          cancelable: true,
          ctrlKey: !!m.ctrl,
          shiftKey: !!m.shift,
        }),
      )
      return true
    },
    index,
    mods,
  )

describe('batch scope modes (T-WELL-03, T-AUX-15)', () => {
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

    // No active group, and a tree that has actually re-read (imports through `invoke` do not tell
    // the frontend anything).
    await call('set_active_well_group', { groupId: null })
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (!sel) return
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })
    await browser.waitUntil(
      async () =>
        (await browser.execute(() => document.querySelectorAll('.tree-node.tree-well').length)) >= 3,
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Wells pane never listed the wells' },
    )

    // Start with nothing pinned, so the ★ scope means what this spec sets.
    for (const id of await invokeOk('list_pinned_wells')) {
      await call('set_well_pin', { wellId: id, pinned: false })
    }

    // A module pane to read the scope from. vsh_gr's pane may already exist from another spec —
    // that is fine here, since every test sets the mode it needs rather than relying on the
    // pane's initial state.
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
      async () =>
        await browser.execute(() => !!document.querySelector('.module-pane .well-scope-count')),
      { timeout: 30_000, interval: 500, timeoutMsg: 'no module pane with a scope readout' },
    )
  })

  after(async () => {
    for (const id of await invokeOk('list_pinned_wells')) {
      await clickStar(id)
    }
    await browser.execute(() => {
      const rows = Array.from(document.querySelectorAll('.tree-node.tree-well'))
      rows[0]?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
      const panes = document.querySelectorAll('.module-pane')
      const pane = panes[panes.length - 1]
      Array.from(pane?.querySelectorAll('.well-scope-mode') ?? [])
        .find((b) => (b.textContent ?? '').trim() === 'All')
        ?.click()
    })
  })

  it('resolves All to every well in the project', async () => {
    assert.ok(await setMode('All'), 'no All scope mode in the module pane')
    await expectScope(wells.length, 'All must resolve to every well')
  })

  it('follows the Wells pane multi-selection LIVE', async () => {
    assert.ok(await setMode('Selection'), 'no Selection scope mode')

    // Nothing selected yet — the count must say so rather than quietly falling back to All, which
    // is the dangerous default: the user thinks they are running on a handful and covers the field.
    await browser.execute(() => {
      const rows = Array.from(document.querySelectorAll('.tree-node.tree-well'))
      rows[0]?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    })
    await expectScope(0, 'Selection with nothing selected must resolve to nothing, never to All')

    // Now ctrl-click two wells and require the OPEN pane to follow without being reopened. That
    // "live" part is the whole claim of T-WELL-03 — a scope that only reads the selection at open
    // time is the same bug as T-WELL-06, one level down.
    assert.ok(await clickWell(0, { ctrl: true }))
    assert.ok(await clickWell(1, { ctrl: true }))
    await expectScope(2, 'the open pane must follow a growing multi-selection')

    assert.ok(await clickWell(1, { ctrl: true }))
    await expectScope(1, 'and follow it shrinking again')
  })

  it('resolves the star scope to the pinned wells', async () => {
    // T-AUX-15. Pinned wells are a persisted favourites set offered as a one-click run scope in
    // every batch tool.
    //
    // Pinned through the TREE'S OWN STAR, not through `set_well_pin`. The scope resolves against
    // `appState.pinnedWellIds`, which is frontend state that only `togglePin` updates — invoking
    // the command writes the project and leaves the pane reading an empty set, so a test that
    // pinned that way would report the scope broken when it is the test that skipped a step. Same
    // rule as the Wells pane not hearing about imported wells.
    assert.ok(await clickStar(wells[2].well_id), 'no star control on the target well row')
    await browser.waitUntil(
      async () => (await invokeOk('list_pinned_wells')).includes(wells[2].well_id),
      { timeout: 15_000, interval: 250, timeoutMsg: 'the pin never reached the project' },
    )

    assert.ok(await setMode('★ Pinned'), 'no pinned scope mode in the module pane')
    await expectScope(1, 'the star scope must resolve to the pinned well')

    assert.ok(await clickStar(wells[0].well_id))
    await expectScope(2, 'and follow a second pin without the pane being reopened')
  })

  it('leaves the scope showing nothing when the pinned set is emptied', async () => {
    // The mirror of the Selection case, and the same risk: a scope that silently fell back to All
    // when its set went empty would run the whole field from a button labelled with a star.
    for (const id of await invokeOk('list_pinned_wells')) {
      await clickStar(id)
    }
    await browser.waitUntil(async () => (await invokeOk('list_pinned_wells')).length === 0, {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'could not clear the pinned set through the tree',
    })
    await expectScope(0, 'an empty pinned set must resolve to nothing, never to All')
  })
})
