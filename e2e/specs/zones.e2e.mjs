// T-WELL-15 — the Zones pane: add, update, delete, invalid input, per-well isolation.
//
// Zones are the unit almost every later answer is quoted per: zone parameters override module
// defaults, the pay summary reports per zone, the Field Dashboard aggregates per zone, and the
// report's tables are laid out per zone. A zone written to the wrong well, silently duplicated,
// or left behind after a delete does not announce itself — it just changes numbers in a
// deliverable.
//
// Driven through the real pane rather than through `upsert_zone`, because two of the five claims
// exist ONLY in the dialog: the `bottom <= top` refusal is a frontend guard (`db::upsert_zone`
// has no validation at all and would happily store an inverted zone), and "update, do not
// duplicate" is a property of the ON CONFLICT clause that a UI round trip is the honest way to
// exercise.

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

const ZONE = 'E2E-SAND'

/** Type into the pane's add row and press Add / Update Zone, all in one round trip. */
const addZone = (name, top, bottom) =>
  browser.execute(
    (n, t, b) => {
      const row = document.querySelector('.zones-pane .zone-add-row')
      if (!row) return { ok: false, why: 'no zone add row — is the Zones pane open?' }
      const [nameIn, topIn, botIn] = Array.from(row.querySelectorAll('input'))
      const btn = row.querySelector('button')
      if (!nameIn || !topIn || !botIn || !btn) return { ok: false, why: 'add row is missing controls' }
      // Assigning .value does not fire the events a framework might listen for; this dialog reads
      // the values directly in its click handler, so a plain assignment is what it actually sees.
      nameIn.value = n
      topIn.value = String(t)
      botIn.value = String(b)
      btn.click()
      return { ok: true }
    },
    name,
    top,
    bottom,
  )

/** The zone rows the pane is showing: [name, top, bottom] per row. */
const zoneRows = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.zones-pane .zone-table tbody tr'))
      .map((tr) => Array.from(tr.querySelectorAll('td')).map((td) => td.textContent.trim()))
      .filter((cells) => cells.length >= 3 && cells[0] !== ''),
  )

/**
 * Wait until `list_zones` does or does not carry `name` for a well, then assert it.
 *
 * Membership rather than an exact list, deliberately: these specs share one project, "From Tops"
 * and other work can legitimately leave zones behind, and a test that demanded an exactly-empty
 * well would fail for a reason that has nothing to do with what it is checking.
 *
 * Same waiting discipline as the Wells pane's `expectMulti`: the dialog's handlers are async
 * (`await upsertZone(...)` then `await refresh()`), so reading straight after the click measures a
 * race. The claim is that the zone ends up stored, not how quickly it got there.
 */
async function expectZone(wellId, name, present, message) {
  let last = []
  try {
    await browser.waitUntil(
      async () => {
        last = (await invokeOk('list_zones', { wellId })).map((z) => z.zone_name)
        return last.includes(name) === present
      },
      { timeout: 10_000, interval: 200 },
    )
  } catch {
    assert.equal(last.includes(name), present, `${message} (well holds: ${last.join(', ') || 'none'})`)
  }
  return last
}

describe('zones pane (T-WELL-15)', () => {
  let wells = []
  let target = null
  let other = null

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
    assert.ok(wells.length >= 2, `need at least 2 wells, found ${wells.length}`)
    target = wells[0]
    other = wells[1]

    // Start from a known state. A previous run of this spec in the same project would otherwise
    // leave its zone behind and turn "add" into "update" without saying so.
    await call('delete_zone', { wellId: target.well_id, zoneName: ZONE })
    await call('delete_zone', { wellId: other.well_id, zoneName: ZONE })

    // The pane follows the ACTIVE well, so select the target through the tree first — the same
    // path a user takes, and the only one that also updates `appState.selectedWell`.
    await browser.execute((id) => {
      const row = Array.from(document.querySelectorAll('.tree-node.tree-well')).find(
        (n) => (n.title ?? '').split('\n')[0] === id,
      )
      row?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    }, target.well_id)

    // Open the pane from its real ribbon button.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
      document.querySelector('#zones-btn')?.click()
    })

    await browser.waitUntil(
      async () =>
        await browser.execute(() => !!document.querySelector('.zones-pane .zone-add-row')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Zones pane never opened after clicking Zones… on the Petrophysics tab',
      },
    )
  })

  after(async () => {
    // Leave the project as found — a stray zone would change what a later spec's pay summary or
    // zone-parameter read sees.
    await call('delete_zone', { wellId: target.well_id, zoneName: ZONE })
    await call('delete_zone', { wellId: other.well_id, zoneName: ZONE })
  })

  it('adds a zone and shows it in the pane', async () => {
    const r = await addZone(ZONE, 1000, 1050)
    assert.ok(r.ok, r.why)

    await expectZone(target.well_id, ZONE, true, 'the added zone must be stored against the well')

    const rows = await zoneRows()
    const mine = rows.find((c) => c[0] === ZONE)
    assert.ok(mine, `the pane must list the new zone; rows: ${JSON.stringify(rows)}`)
    assert.deepEqual(
      [mine[1], mine[2]],
      ['1000.0', '1050.0'],
      'the pane must show the depths that were entered, to one decimal',
    )
  })

  it('updates an existing zone rather than duplicating it', async () => {
    const r = await addZone(ZONE, 1010, 1080)
    assert.ok(r.ok, r.why)

    // The claim is the COUNT as much as the values: `upsert_zone`'s ON CONFLICT is what stops a
    // second entry appearing, and a duplicated zone would be reported twice in every pay summary
    // and every per-zone average downstream.
    await browser.waitUntil(
      async () => {
        const zones = await invokeOk('list_zones', { wellId: target.well_id })
        const mine = zones.filter((z) => z.zone_name === ZONE)
        return mine.length === 1 && Math.abs(mine[0].top_depth - 1010) < 0.01
      },
      {
        timeout: 10_000,
        interval: 200,
        timeoutMsg: 'the second add must UPDATE the zone in place, leaving exactly one row',
      },
    )

    const zones = await invokeOk('list_zones', { wellId: target.well_id })
    const mine = zones.filter((z) => z.zone_name === ZONE)
    assert.equal(mine.length, 1, 'a re-add under the same name must not create a second zone')
    assert.ok(Math.abs(mine[0].bottom_depth - 1080) < 0.01, 'the new bottom depth must be stored')
  })

  it('silently refuses a zone whose bottom is not below its top', async () => {
    const before = await invokeOk('list_zones', { wellId: target.well_id })

    // `bottom <= top` is refused in the DIALOG (`zonesDialog.ts` returns early), not in the
    // backend — `db::upsert_zone` has no validation and would store an inverted zone quite
    // happily. So this is a frontend-only contract and could not be pinned by a Rust test.
    //
    // The refusal is silent by design, which is why the assertion is on the STORED state: there is
    // no message to look for, and the only observable claim is that nothing changed.
    const r = await addZone(ZONE, 1200, 1100)
    assert.ok(r.ok, r.why)

    // Equality is checked, not just "the zone still exists": an inverted write would keep the same
    // name and the same row count while silently swapping the interval underneath it.
    const after = await invokeOk('list_zones', { wellId: target.well_id })
    assert.deepEqual(
      after.map((z) => [z.zone_name, z.top_depth, z.bottom_depth]),
      before.map((z) => [z.zone_name, z.top_depth, z.bottom_depth]),
      'an inverted interval must leave every stored zone exactly as it was',
    )

    // And the equal case, which `bottom <= top` also covers — a zero-thickness zone is not a zone.
    assert.ok((await addZone(ZONE, 1300, 1300)).ok)
    const afterEqual = await invokeOk('list_zones', { wellId: target.well_id })
    assert.deepEqual(
      afterEqual.map((z) => [z.zone_name, z.top_depth, z.bottom_depth]),
      before.map((z) => [z.zone_name, z.top_depth, z.bottom_depth]),
      'a zero-thickness interval must be refused too',
    )
  })

  it('keeps zones on the well they were entered against', async () => {
    // Per-well isolation. `zones` is keyed by (well_id, zone_name), so a zone leaking to a
    // neighbour would be a wrong reservoir interval on a well nobody edited — and every pay
    // number quoted for it afterwards would be about rock that was never picked.
    const otherZones = await invokeOk('list_zones', { wellId: other.well_id })
    assert.ok(
      !otherZones.some((z) => z.zone_name === ZONE),
      `${ZONE} must not appear on ${other.well_name}; it holds ${otherZones
        .map((z) => z.zone_name)
        .join(', ')}`,
    )
  })

  it('deletes a zone from the pane', async () => {
    const clicked = await browser.execute((name) => {
      const rows = Array.from(document.querySelectorAll('.zones-pane .zone-table tbody tr'))
      const row = rows.find((tr) => tr.querySelector('td')?.textContent?.trim() === name)
      const del = row?.querySelector('.zone-del')
      if (!del) return false
      del.click()
      return true
    }, ZONE)
    assert.ok(clicked, `no delete control on the ${ZONE} row`)

    await expectZone(target.well_id, ZONE, false, 'the deleted zone must be gone from the project')

    const rows = await zoneRows()
    assert.ok(
      !rows.some((c) => c[0] === ZONE),
      `the pane must stop listing the deleted zone; rows: ${JSON.stringify(rows)}`,
    )
  })
})
