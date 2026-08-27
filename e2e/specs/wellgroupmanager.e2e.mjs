// T-WELL-04 (the Well Groups manager), T-WELL-05 (an active group scopes the tree and freshly
// opened batch dialogs) and T-WELL-06 (the NEGATIVE: an already-open batch dialog does not
// re-scope).
//
// `wellgroups.e2e.mjs` covers what a group DOES to a run. This covers the manager that creates one
// and the scoping it drives — and T-WELL-06 is a regression guard on a bug that is still open
// (AUDIT-2026-07-21, the group-rescope gap in `wellScope.ts`). It therefore asserts the WRONG
// behaviour on purpose: an open pane keeps the stale group. The day that is fixed this test goes
// red, which is the alarm, and the assertion should be flipped rather than deleted.

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

const GROUP = 'E2E-MANAGED'
const RENAMED = 'E2E-RENAMED'

const groups = () => invokeOk('list_well_groups')

/** Open the Well Groups manager from the ⚙ in the Wells pane's group bar. */
const openManager = () =>
  browser.execute(() => {
    const btn = Array.from(document.querySelectorAll('.tree-group-bar .tree-group-manage')).find(
      (b) => (b.textContent ?? '').trim() === '⚙',
    )
    if (!btn) return false
    btn.click()
    return true
  })

/** The manager's group rows: name and member count as displayed. */
const managerRows = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('#modal-root .wg-row'))
      .filter((r) => !r.classList.contains('wg-header'))
      .map((r) => ({
        name: r.querySelector('.wg-name')?.textContent?.trim() ?? '',
        count: r.querySelector('.wg-count')?.textContent?.trim() ?? '',
        active: !!r.querySelector('input[type="radio"]')?.checked,
      })),
  )

const closeModal = async () => {
  await browser.execute(() =>
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })),
  )
  await browser.waitUntil(
    async () =>
      await browser.execute(
        () => (document.querySelector('#modal-root')?.childElementCount ?? 0) === 0,
      ),
    { timeout: 10_000, interval: 200, timeoutMsg: 'a modal would not close on Escape' },
  )
}

/** The visible well rows in the Wells pane. */
const treeWellCount = () =>
  browser.execute(() => document.querySelectorAll('.tree-node.tree-well').length)

describe('well group manager and scoping (T-WELL-04, T-WELL-05, T-WELL-06)', () => {
  let wells = []

  before(async () => {
    const existing = await invokeOk('list_wells', { scope: { kind: 'all' } })
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      // The import refuses without a declared sampling style + step tolerance (see
      // despike.e2e.mjs); 0.01 m is a test input for the synthetic examples, not a field value.
      const imported = await invokeOk('import_las_files', {
        paths,
        setName: 'E2E',
        attach: false,
        samplingStyle: 'CONTINUOUS_REGULAR',
        samplingStyleVerifyTolerance: { value: 0.01, unit: 'M' },
      })
      // A refused file is not an invoke error — the command succeeds and reports per file.
      for (const r of imported) {
        assert.ok(r.well_id != null, `import failed for ${r.path}: ${r.error}`)
      }
    }
    wells = await invokeOk('list_wells', { scope: { kind: 'all' } })
    wells.sort((a, b) => a.well_name.localeCompare(b.well_name))
    assert.ok(wells.length >= 3, `need at least 3 wells, found ${wells.length}`)

    // Remove any leftovers from an earlier run of this spec, under either name.
    for (const g of await groups()) {
      if (g.name === GROUP || g.name === RENAMED) {
        await call('delete_well_group', { groupId: g.group_id })
      }
    }

    // Make the tree re-read (imports through `invoke` do not notify the frontend) and start with
    // no active group, so the counts below mean what they say.
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (!sel) return
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })
    await browser.waitUntil(async () => (await treeWellCount()) >= 3, {
      timeout: 30_000,
      interval: 500,
      timeoutMsg: 'the Wells pane never listed the example wells with no group active',
    })
  })

  after(async () => {
    for (const g of await groups()) {
      if (g.name === GROUP || g.name === RENAMED) {
        await call('delete_well_group', { groupId: g.group_id })
      }
    }
    // Hand the workspace back unscoped, and let the tree hear about it.
    await call('set_active_well_group', { groupId: null })
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (sel) {
        sel.value = ''
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }
    })
  })

  it('creates a group from the manager', async () => {
    assert.ok(await openManager(), 'no ⚙ Manage well groups button in the Wells pane')
    await browser.waitUntil(
      async () =>
        await browser.execute(() => !!document.querySelector('#modal-root .wg-new-row')),
      { timeout: 15_000, interval: 250, timeoutMsg: 'the Well Groups manager never opened' },
    )

    await browser.execute((name) => {
      const row = document.querySelector('#modal-root .wg-new-row')
      const input = row.querySelector('input.form-control')
      input.value = name
      Array.from(row.querySelectorAll('button'))
        .find((b) => (b.textContent ?? '').trim() === 'Create')
        .click()
    }, GROUP)

    await browser.waitUntil(async () => (await groups()).some((g) => g.name === GROUP), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'Create never wrote the group',
    })

    const rows = await managerRows()
    assert.ok(
      rows.some((r) => r.name === GROUP),
      `the manager must list the new group; it shows: ${rows.map((r) => r.name).join(', ')}`,
    )
  })

  it('renames a group in place, keeping its identity and membership', async () => {
    const before = (await groups()).find((g) => g.name === GROUP)
    assert.ok(before, 'the group created above must still exist')

    // Rename is a double-click that opens `window.prompt` — a BROWSER dialog the WebDriver session
    // cannot see or answer, so it is stubbed for the duration of the click. That is a real
    // limitation being worked around, not a claim about the UI: what is verified is the rename
    // path behind the prompt, not the prompt itself.
    await browser.execute(
      (oldName, newName) => {
        const original = window.prompt
        window.prompt = () => newName
        const row = Array.from(document.querySelectorAll('#modal-root .wg-row')).find(
          (r) => r.querySelector('.wg-name')?.textContent?.trim() === oldName,
        )
        row.querySelector('.wg-name').dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
        window.prompt = original
      },
      GROUP,
      RENAMED,
    )

    await browser.waitUntil(async () => (await groups()).some((g) => g.name === RENAMED), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the rename never reached the project',
    })

    const after = (await groups()).find((g) => g.name === RENAMED)
    // The IDENTITY must survive. A rename that quietly created a new group and dropped the old one
    // would look identical in the list and would silently empty the membership — and every batch
    // dialog scoped to it would then run on nothing.
    assert.equal(after.group_id, before.group_id, 'a rename must keep the same group id')
    assert.equal(
      after.member_count,
      before.member_count,
      'a rename must not touch membership',
    )
    assert.ok(
      !(await groups()).some((g) => g.name === GROUP),
      'the old name must be gone, not duplicated',
    )
  })

  it('scopes the Wells pane to the active group', async () => {
    const all = await treeWellCount()
    assert.ok(all >= 3, 'this test needs the unfiltered tree first')

    const g = (await groups()).find((x) => x.name === RENAMED)
    await invokeOk('set_well_group_members', {
      groupId: g.group_id,
      wellIds: [wells[0].well_id, wells[1].well_id],
    })

    // Activate through the manager's own radio, which is the user path and the one that also syncs
    // `appState.activeWellGroup`.
    await browser.execute((name) => {
      const row = Array.from(document.querySelectorAll('#modal-root .wg-row')).find(
        (r) => r.querySelector('.wg-name')?.textContent?.trim() === name,
      )
      const radio = row.querySelector('input[type="radio"]')
      radio.checked = true
      radio.dispatchEvent(new Event('change', { bubbles: true }))
    }, RENAMED)

    await browser.waitUntil(async () => (await treeWellCount()) === 2, {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: `the Wells pane never narrowed to the group's 2 members (it shows ${await treeWellCount()})`,
    })

    // And the members shown are the RIGHT two, not just two of them.
    const shown = await browser.execute(() =>
      Array.from(document.querySelectorAll('.tree-node.tree-well')).map(
        (n) => (n.title ?? '').split('\n')[0],
      ),
    )
    assert.deepEqual(
      [...shown].sort(),
      [wells[0].well_id, wells[1].well_id].sort(),
      'the tree must show exactly the group members',
    )

    await closeModal()
  })

  it('gives a NEWLY opened batch pane the active group as its scope', async () => {
    // T-WELL-05's second half. A pane opened while a group is active must start scoped to it —
    // otherwise the user sets a group, opens a tool, and runs on the whole field.
    //
    // A module NO OTHER SPEC OPENS, deliberately. Module panes are singletons: asking for one that
    // is already open re-focuses it, keeping whatever scope it was left with — so using vsh_gr here
    // measured the pane `moduledialog.e2e.mjs` had left on "Selection: 0 wells" and called it a
    // newly opened pane. The claim is about a FRESH pane, so the test has to get a fresh one.
    const modules = await invokeOk('list_modules')
    const spec =
      modules.find((m) => m.name === 'sw_arch') ??
      modules.find((m) => m.name !== 'vsh_gr' && m.category)
    assert.ok(spec, 'need a second module to open a fresh pane with')

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

    // Identify the pane by its OWN manifest outputs rather than taking "the last .module-pane".
    // Two reasons that shortcut is wrong: several module panes can be open at once, and dockview
    // DETACHES inactive tabs from the DOM, so which panes are present at all depends on what other
    // specs left focused. Reading the wrong pane made this test report a 3-well scope for a 2-well
    // group — a failure that says nothing about the feature. The WAIT must run on the identified
    // pane too: a generic `.well-scope-count` wait is satisfied by any earlier spec's pane while
    // this one is still building its outputs grid.
    const outputs = (spec.args ?? []).filter((a) => a.kind === 'log_out').map((a) => a.name)
    const readCount = (outs) => {
      const panes = Array.from(document.querySelectorAll('.module-pane'))
      // The outputs section's labels carry "Declared as <ARG>" titles — the manifest identity
      // the old "Outputs:" hint used to state.
      const mine = panes.find((p) => {
        const titles = Array.from(p.querySelectorAll('.module-outputs .module-output-label')).map(
          (l) => l.title ?? '',
        )
        return outs.every((o) =>
          titles.some((t) => t === `Declared as ${o}` || t.startsWith(`Declared as ${o};`)),
        )
      })
      return mine?.querySelector('.well-scope-count')?.textContent?.trim() ?? '(pane not found)'
    }
    await browser.waitUntil(
      async () => (await browser.execute(readCount, outputs)) !== '(pane not found)',
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: `the ${spec.name} pane never built its outputs section`,
      },
    )
    const count = await browser.execute(readCount, outputs)

    assert.match(
      count,
      /\b2\b/,
      `a pane opened under a 2-well group must scope to 2; ${spec.name}'s count reads "${count}"`,
    )
  })

  it('does NOT re-scope a batch pane that is already open (known open bug)', async () => {
    // T-WELL-06, and it asserts the WRONG behaviour deliberately.
    //
    // AUDIT-2026-07-21 records that `wellScope.ts` does not follow a group change once a pane is
    // built. The consequence is quiet and expensive: the pane goes on displaying the old group's
    // name and count while the user believes they have re-scoped, and the run covers the wrong
    // wells. Pinning it means the day it is fixed this test goes RED — which is the alarm. Flip
    // the assertion then; do not delete it.
    // Same identification problem as the previous test: read THIS module's pane, not whichever
    // .module-pane happens to be attached.
    const outs = await browser.execute(() => {
      const panes = Array.from(document.querySelectorAll('.module-pane'))
      return panes.map((p) => p.querySelector('.well-scope-count')?.textContent?.trim() ?? '')
    })
    const before = outs.join('|')

    // Drop back to All wells through the tree's own select — a real group change, made while the
    // pane above is open.
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })
    await browser.waitUntil(async () => (await treeWellCount()) >= 3, {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the tree never came back to the unfiltered list',
    })

    const afterList = await browser.execute(() => {
      const panes = Array.from(document.querySelectorAll('.module-pane'))
      return panes.map((p) => p.querySelector('.well-scope-count')?.textContent?.trim() ?? '')
    })
    const after = afterList.join('|')
    assert.equal(
      after,
      before,
      'DOCUMENTED CURRENT BEHAVIOUR (open bug): an already-open batch pane keeps the stale group ' +
        'scope after the active group changes. If this now differs, the bug is FIXED — flip this ' +
        'assertion to require the pane to follow, rather than removing it.',
    )
  })

  it('deletes a group from the manager', async () => {
    assert.ok(await openManager(), 'could not reopen the Well Groups manager')
    await browser.waitUntil(
      async () => (await managerRows()).some((r) => r.name === RENAMED),
      { timeout: 15_000, interval: 250, timeoutMsg: 'the manager never listed the group to delete' },
    )

    await browser.execute((name) => {
      const row = Array.from(document.querySelectorAll('#modal-root .wg-row')).find(
        (r) => r.querySelector('.wg-name')?.textContent?.trim() === name,
      )
      Array.from(row.querySelectorAll('button'))
        .find((b) => (b.textContent ?? '').trim() === 'Delete')
        .click()
    }, RENAMED)

    await browser.waitUntil(async () => !(await groups()).some((g) => g.name === RENAMED), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the group was never deleted',
    })

    // The wells must still be there. A group is a view over wells, not a container of them —
    // deleting one must never take its members with it.
    const stillThere = await invokeOk('list_wells', { scope: { kind: 'all' } })
    assert.ok(
      stillThere.length >= 3,
      `deleting a group must not delete its wells; ${stillThere.length} remain`,
    )

    await closeModal()
  })
})
