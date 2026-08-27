// T-WELL-02 (multi-select) and the selection half of T-WELL-01, driven through the real Wells
// pane against the real app.
//
// This is the part of the shell that decides WHICH WELLS A BATCH RUN COVERS. `appState
// .multiSelectedWellIds` is what every batch dialog's "Selection" scope reads, so a multi-select
// that reports the wrong set does not fail — it runs on the wrong wells and writes real curves to
// them. Same shape of risk as the well-group scoping in `wellgroups.e2e.mjs`, one level down.
//
// Everything here goes through the pane's own click handlers rather than through `invoke`,
// because the whole claim IS the click semantics: ctrl toggles without moving the active well,
// shift takes a range from the anchor, a plain click clears. None of that exists in the backend.

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

/**
 * Read the well rows the pane is actually showing.
 *
 * The row carries its well id as the first line of its `title` (objectTree.ts writes
 * `${well_id}\nClick: activate • …`), which is used here rather than adding a `data-` attribute to
 * production markup for the benefit of a test. If that tooltip is ever restructured this helper
 * breaks loudly, which is the right failure — it is reading a real contract, not a coincidence.
 */
const treeRows = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.tree-node.tree-well')).map((n) => ({
      id: (n.title ?? '').split('\n')[0],
      label: n.querySelector('.tree-well-label')?.textContent?.trim() ?? '',
      selected: n.classList.contains('tree-selected'),
      multi: n.classList.contains('tree-multi'),
      pinned: !!n.querySelector('.tree-pin.tree-pinned'),
    })),
  )

/**
 * Click the Nth well row, optionally with modifiers.
 *
 * `el.click()` CANNOT carry modifier keys — it synthesises a plain click and every modifier reads
 * false — so ctrl- and shift-click have to be dispatched as real MouseEvents. That is the whole
 * trap in testing this feature: a ctrl-click written as `el.click()` silently becomes a plain
 * click, the multi-selection is cleared instead of extended, and the test then asserts the
 * behaviour of a completely different gesture.
 *
 * `bubbles: true` matters too — the handler is bound to the row itself here, but the pane's
 * context-menu and pin handlers rely on propagation, so a non-bubbling event would diverge from a
 * real one for no benefit.
 */
const clickRow = (index, mods = {}) =>
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

const multiIds = (rows) => rows.filter((r) => r.multi).map((r) => r.id).sort()

/**
 * Wait for the multi-selection SHOWN IN THE PANE to settle on `expected`, then assert it.
 *
 * The selection lives in `appState.multiSelectedWellIds`; the `tree-multi` class on a row is a
 * rendering of that state which arrives one async `refresh()` later (`setMulti` fires it without
 * awaiting). Reading the DOM immediately after a click therefore tests which of the two won a
 * race — observed directly: the same ctrl-click assertion passed on one run and failed on the
 * next with no code change between them.
 *
 * Waiting is not a workaround here, it is the correct claim: what is being tested is that the pane
 * ENDS UP showing the right set, not how many milliseconds it took. On timeout the last observed
 * value is asserted so the failure still reads as a normal expected-vs-actual diff.
 */
async function expectMulti(expected, message) {
  const want = [...expected].sort()
  let last = []
  try {
    await browser.waitUntil(
      async () => {
        last = multiIds(await treeRows())
        return last.length === want.length && last.every((v, i) => v === want[i])
      },
      { timeout: 10_000, interval: 200 },
    )
  } catch {
    assert.deepEqual(last, want, message)
  }
  return last
}

describe('wells pane selection (T-WELL-01, T-WELL-02)', () => {
  let rows = []

  before(async () => {
    // Own the fixtures rather than depending on another spec having run — wdio gives no ordering
    // guarantee across files.
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

    // Two things at once, and the second one is the trap every later DOM spec will hit.
    //
    // (1) Clear any active well group THROUGH THE PANE'S OWN SELECT rather than through
    //     `set_active_well_group`. Invoking that command directly would change the database while
    //     the frontend went on rendering the old scope — `appState.activeWellGroup` is only synced
    //     by `activateWellGroup`, which the select's change handler calls. It matters because
    //     `wellgroups.e2e.mjs` deliberately LEAVES a two-well group active and sorts before this
    //     file, so this is the normal case rather than a corner.
    //
    // (2) DRIVING THE BACKEND THROUGH `invoke` DOES NOT TELL THE FRONTEND ANYTHING. The wells this
    //     spec (and `pipeline.e2e.mjs`) import land in DuckDB, but the Wells pane only re-reads
    //     when the frontend's own code path asks it to — so it happily goes on showing "No wells
    //     ingested yet" over a project with three wells in it. Observed exactly that.
    //
    // The `change` event is therefore dispatched UNCONDITIONALLY, even when the value is already
    // "": its handler ends in `this.refresh()`, which re-fetches the well list. A guard of the
    // shape `if (sel.value !== '')` looks tidier and suppresses the very refresh that is wanted.
    // This is a real user path, not a test hook, which is why it is preferred to reloading.
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (!sel) return
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })

    try {
      await browser.waitUntil(async () => (await treeRows()).length >= 3, {
        timeout: 30_000,
        interval: 500,
      })
    } catch {
      // Report what the pane actually held. "Never showed three wells" is true of a filtered list,
      // a pane that was never mounted and a renamed class alike, and those need different fixes.
      const seen = await browser.execute(() => ({
        wellRows: document.querySelectorAll('.tree-node.tree-well').length,
        anyTreeNodes: document.querySelectorAll('.tree-node').length,
        groupSelects: document.querySelectorAll('.tree-group-select').length,
        groupValue: document.querySelector('.tree-group-select')?.value ?? '(no select)',
        groupLabel: document.querySelector('.tree-group-label')?.textContent?.trim() ?? '(none)',
        emptyNote: document.querySelector('.tree-empty')?.textContent?.trim() ?? '(none)',
      }))
      throw new Error(
        `the Wells pane never showed the three example wells with no group active: ${JSON.stringify(seen)}`,
      )
    }
    rows = await treeRows()
  })

  it('activates one well on a plain click and marks exactly that row', async () => {
    assert.ok(await clickRow(0), 'no well row at index 0')
    const after = await treeRows()

    const selected = after.filter((r) => r.selected).map((r) => r.id)
    assert.deepEqual(
      selected,
      [rows[0].id],
      `a plain click must leave exactly one row selected; got ${selected.length}`,
    )
    assert.equal(multiIds(after).length, 0, 'a plain click must not create a multi-selection')
  })

  it('builds a multi-selection on ctrl-click without moving the active well', async () => {
    // The active well is index 0 from the previous test. Ctrl-clicking two OTHER rows must add
    // them to the multi-selection and leave the active well exactly where it was — that is the
    // entire point of the gesture: assemble a batch set while every open view stays put.
    const before = await treeRows()
    const activeBefore = before.filter((r) => r.selected).map((r) => r.id)

    assert.ok(await clickRow(1, { ctrl: true }))
    assert.ok(await clickRow(2, { ctrl: true }))
    await expectMulti(
      [rows[1].id, rows[2].id],
      'ctrl-click must add exactly the clicked rows to the multi-selection',
    )

    const after = await treeRows()
    assert.deepEqual(
      after.filter((r) => r.selected).map((r) => r.id),
      activeBefore,
      'ctrl-click must NOT move the active well — open views would follow it',
    )

    // Ctrl-click is a TOGGLE, not an add. Clicking one of them again must remove it, and a test
    // that only ever adds would pass on an implementation that cannot.
    assert.ok(await clickRow(2, { ctrl: true }))
    await expectMulti(
      [rows[1].id],
      'a second ctrl-click on the same row must remove it from the selection',
    )
  })

  it('takes a range from the anchor on shift-click', async () => {
    // Set the anchor explicitly rather than inheriting whatever the previous test left. The anchor
    // is wherever the last plain OR ctrl click landed, so a test that assumes it is doing
    // arithmetic on another test's final gesture — this one originally expected a two-row range
    // and got all three, because the ctrl test ends by toggling row 2 back OFF, which still moves
    // the anchor there. The behaviour was right and the expectation was wrong.
    assert.ok(await clickRow(0)) // plain click: activates, clears the selection, anchor = 0
    assert.ok(await clickRow(1, { shift: true }))

    // A range of two out of three, deliberately: an implementation that simply selected every
    // visible well would satisfy "the range is included" and fail this.
    const got = await expectMulti(
      [rows[0].id, rows[1].id],
      'shift-click must select the inclusive range between the anchor and the clicked row, and ' +
        'stop there rather than running to the end of the list',
    )
    assert.ok(!got.includes(rows[2].id), 'shift-click must stop at the clicked row')
  })

  it('inverts the multi-selection within the visible wells', async () => {
    // Establish a selection of exactly one, so BOTH sides of the inversion are non-empty. Inverting
    // a full selection yields nothing, which passes the arithmetic below while proving very little
    // — and it silently starved the next test of anything to clear.
    assert.ok(await clickRow(0))
    assert.ok(await clickRow(0, { ctrl: true }))
    const before = await expectMulti([rows[0].id], 'setting up a one-well selection to invert')

    const clicked = await browser.execute(() => {
      const btn = Array.from(document.querySelectorAll('.tree-group-bar .tree-group-manage')).find(
        (b) => (b.textContent ?? '').trim() === '⇄',
      )
      if (!btn) return false
      btn.click()
      return true
    })
    assert.ok(clicked, 'no invert button in the tree group bar')

    const visible = (await treeRows()).map((r) => r.id).sort()
    const inverted = await expectMulti(
      visible.filter((id) => !before.includes(id)),
      'invert must select exactly the visible wells that were not selected',
    )
    // Stated separately because the assertion above would also be satisfied by an empty result if
    // everything happened to be selected — and an "invert" that quietly selects nothing is the bug
    // worth catching.
    assert.equal(
      inverted.length + before.length,
      visible.length,
      'the selection and its inverse must together account for every visible well, exactly once',
    )
    assert.ok(inverted.length > 0, 'inverting a partial selection must not come back empty')
  })

  it('clears the multi-selection on a plain click', async () => {
    // Own precondition, from a KNOWN starting state. The plain click first is not redundant: the
    // invert test ends with rows 1 and 2 selected, so a bare ctrl-click on row 1 would toggle it
    // OFF and set up the opposite of what this test needs. Ctrl is a toggle, so "add one" is only
    // meaningful once the selection is empty.
    assert.ok(await clickRow(0))
    assert.ok(await clickRow(1, { ctrl: true }))
    assert.ok(
      (await expectMulti([rows[1].id], 'setting up a selection to clear')).length > 0,
      'this test needs a selection to clear',
    )

    assert.ok(await clickRow(0))
    await expectMulti(
      [],
      'a plain click must drop the multi-selection — otherwise a batch run keeps a scope the ' +
        'user thinks they have dismissed',
    )

    const after = await treeRows()
    assert.deepEqual(
      after.filter((r) => r.selected).map((r) => r.id),
      [rows[0].id],
      'and it must still activate the clicked well',
    )
  })

  it('pins a well from the star, and the pin is stored in the project', async () => {
    // T-WELL-01's ★ leg. The claim is not that a class toggled but that the pin PERSISTED: pinned
    // wells are offered as a one-click run scope in every batch tool, so a star that looks set and
    // was never written gives a scope that silently empties on the next launch.
    const target = rows[0].id
    const pinnedBefore = await invokeOk('list_pinned_wells')
    const wasPinned = pinnedBefore.includes(target)

    const clicked = await browser.execute((id) => {
      const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === id,
      )
      const star = row?.querySelector('.tree-pin')
      if (!star) return false
      star.click()
      return true
    }, target)
    assert.ok(clicked, 'no star control on the target well row')

    await browser.waitUntil(
      async () => (await invokeOk('list_pinned_wells')).includes(target) !== wasPinned,
      {
        timeout: 15_000,
        interval: 250,
        timeoutMsg: 'clicking the star never changed what list_pinned_wells reports',
      },
    )

    // The pane's own repaint is a separate async step from the write (`togglePin` fires
    // `this.refresh()` without awaiting it), so the DOM is allowed to lag the database by a tick.
    // Waiting rather than asserting immediately tests the claim — the star ends up agreeing with
    // what was stored — instead of testing which of the two happened to finish first.
    await browser.waitUntil(
      async () => {
        const now = await treeRows()
        return now.find((r) => r.id === target)?.pinned === !wasPinned
      },
      {
        timeout: 15_000,
        interval: 250,
        timeoutMsg: 'the star in the pane never caught up with what the project stored',
      },
    )

    // Put it back, so this spec leaves the pinned set as it found it — a stray pin would change
    // the default scope of any batch dialog a later spec opens.
    await browser.execute((id) => {
      const r = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === id,
      )
      r?.querySelector('.tree-pin')?.click()
    }, target)
    await browser.waitUntil(
      async () => (await invokeOk('list_pinned_wells')).includes(target) === wasPinned,
      { timeout: 15_000, interval: 250, timeoutMsg: 'could not restore the original pin state' },
    )
  })
})
