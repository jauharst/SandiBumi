// T-AUX-03 — the Well Header dialog: a TD-only edit must not lose the surface coordinates.
//
// This is a regression guard on a confirmed bug with an unusually quiet failure mode.
// `appState.selectedWell` is a SNAPSHOT captured on tree-click and is not re-broadcast on a data
// change, so after an import — or after a previous header save — it can carry stale, usually null,
// coordinates. The dialog writes every field unconditionally, so building it from that snapshot
// meant opening the header to correct a TD and silently erasing the well's easting, northing and
// UTM zone on the way out.
//
// Nothing downstream complains: a well with no location simply stops appearing on the map, and the
// deviation/TVD work that needs a datum quietly loses its reference. `handleWellHeader` now
// re-reads the well from the database before building the form, and this test is what keeps that.

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

const wellById = async (id) => (await invokeOk('list_wells')).find((w) => w.well_id === id)

/** Open Well Header… from the Data ribbon's Tools menu, whichever entry carries that label. */
const openWellHeader = () =>
  browser.execute(() => {
    document.querySelector('.ribbon-tab[data-tab="data"]')?.click()
    const buttons = Array.from(
      document.querySelectorAll('.ribbon-panel[data-panel="data"] .ribbon-btn'),
    )
    for (const b of buttons) {
      b.click()
      const item = Array.from(
        document.querySelectorAll('.ribbon-menu:not([hidden]) .ribbon-menu-item'),
      ).find((i) => /well header/i.test((i.textContent ?? '').trim()))
      if (item) {
        item.click()
        return true
      }
    }
    document
      .querySelectorAll('.ribbon-menu:not([hidden])')
      .forEach((m) => m.setAttribute('hidden', ''))
    return false
  })

/** Read the dialog's inputs by the placeholder each one carries. */
const headerFields = () =>
  browser.execute(() => {
    const root = document.querySelector('#modal-root')
    if (!root) return null
    const by = (ph) => root.querySelector(`input[placeholder="${ph}"]`)?.value ?? null
    return {
      td: by('total depth (m)'),
      kb: by('KB elevation (m)'),
      x: by('easting (m)'),
      y: by('northing (m)'),
      zone: by('e.g. 50S'),
    }
  })

describe('well header (T-AUX-03)', () => {
  let well = null
  // NOT named `before` — that would shadow mocha's own `before()` hook in this scope, and the hook
  // call below becomes `null(...)`, which fails at LOAD time with "unable to load spec files"
  // rather than as a test failure. Cost a run to spot.
  let asFound = null

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
    asFound = { ...well }

    // Select it in the tree — the dialog works on the selected well, and refuses by name without
    // one (`needWell.ts`).
    await browser.execute((id) => {
      const sel = document.querySelector('.tree-group-select')
      if (sel) {
        sel.value = ''
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }
      const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === id,
      )
      row?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    }, well.well_id)
  })

  after(async () => {
    // Put the header back as it was found.
    await call('update_well_field', { wellId: well.well_id, field: 'td', value: String(asFound.td ?? '') })
  })

  it('gives the well a location, then keeps it through a TD-only edit', async () => {
    // Step 1: set coordinates through the dialog itself, so the values under test arrived by the
    // same path a user would use.
    assert.ok(await openWellHeader(), 'no Well Header… entry in the Data ribbon')
    await browser.waitUntil(async () => (await headerFields()) !== null, {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: 'the Well Header dialog never opened',
    })

    await browser.execute(() => {
      const root = document.querySelector('#modal-root')
      const set = (ph, v) => {
        const el = root.querySelector(`input[placeholder="${ph}"]`)
        if (el) {
          el.value = v
          el.dispatchEvent(new Event('input', { bubbles: true }))
        }
      }
      set('easting (m)', '512345.6')
      set('northing (m)', '9876543.2')
      set('e.g. 50S', '50S')
      set('total depth (m)', '2500')
      Array.from(root.querySelectorAll('button'))
        .find((b) => (b.textContent ?? '').trim() === 'Save Header')
        .click()
    })

    await browser.waitUntil(
      async () => {
        const w = await wellById(well.well_id)
        return w && w.surface_x !== null && Math.abs(Number(w.surface_x) - 512345.6) < 0.1
      },
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg: 'the header save never stored the surface coordinates',
      },
    )

    // Step 2: reopen and change ONLY the TD. This is the exact sequence that used to erase the
    // coordinates — the dialog is rebuilt from `selectedWell`, which was snapshotted before the
    // save above and therefore still carries the OLD (null) location.
    assert.ok(await openWellHeader())
    await browser.waitUntil(async () => (await headerFields()) !== null, {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: 'the Well Header dialog would not reopen',
    })

    // The reopened form must already SHOW the coordinates. If it does not, it was built from the
    // stale snapshot and pressing Save would write those blanks back — so this assertion catches
    // the bug one step before the damage.
    const shown = await headerFields()
    assert.ok(
      shown.x && Math.abs(Number(shown.x) - 512345.6) < 0.1,
      `the reopened dialog must show the stored easting, not a stale blank; it shows "${shown.x}"`,
    )
    assert.ok(
      shown.y && Math.abs(Number(shown.y) - 9876543.2) < 0.1,
      `and the stored northing; it shows "${shown.y}"`,
    )

    await browser.execute(() => {
      const root = document.querySelector('#modal-root')
      const td = root.querySelector('input[placeholder="total depth (m)"]')
      td.value = '2750'
      td.dispatchEvent(new Event('input', { bubbles: true }))
      Array.from(root.querySelectorAll('button'))
        .find((b) => (b.textContent ?? '').trim() === 'Save Header')
        .click()
    })

    await browser.waitUntil(
      async () => {
        const w = await wellById(well.well_id)
        return w && Math.abs(Number(w.td) - 2750) < 0.1
      },
      { timeout: 20_000, interval: 250, timeoutMsg: 'the TD edit never reached the project' },
    )

    // The claim. A TD-only edit must leave the location exactly as it was.
    const after = await wellById(well.well_id)
    assert.ok(
      after.surface_x !== null && Math.abs(Number(after.surface_x) - 512345.6) < 0.1,
      `a TD-only edit must not erase the easting; it is now ${after.surface_x}`,
    )
    assert.ok(
      after.surface_y !== null && Math.abs(Number(after.surface_y) - 9876543.2) < 0.1,
      `nor the northing; it is now ${after.surface_y}`,
    )
    assert.equal(
      after.utm_zone ?? null,
      '50S',
      `nor the UTM zone; it is now ${after.utm_zone}`,
    )
  })
})
