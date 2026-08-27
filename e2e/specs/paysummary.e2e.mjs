// T-BATCH-07 — the Cutoffs & Pay Summary table, and the row invariants that must hold whatever
// cutoffs are used.
//
// This is the table a client report is built from, so a row that violates its own arithmetic is
// the most expensive kind of wrong: it is plausible, it is quoted, and nothing downstream checks
// it. The invariants below are true for ANY cutoff values, which is exactly why they are worth
// asserting — the test never picks a cutoff. It runs the pane with whatever the pane prefills and
// checks the relationships between the numbers that come back.
//
// That is deliberate: VSH/PHIE/SWE cutoffs are petrophysical parameters, and inventing one here to
// make a test pass would put a number in the repo that nobody sourced.

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

const ZONE = 'E2E-PAY'

/** The rendered summary table, as objects. "—" (uninterpreted) becomes null, never 0. */
const summaryRows = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.summary-table tbody tr')).map((tr) => {
      const c = Array.from(tr.querySelectorAll('td')).map((td) => td.textContent.trim())
      const num = (s) => (s === '—' || s === '' ? null : Number(s))
      return {
        well: c[0],
        zone: c[1],
        flag: c[2],
        top: num(c[3]),
        bottom: num(c[4]),
        gross: num(c[5]),
        net: num(c[6]),
        ntg: num(c[7]),
        avgVsh: num(c[8]),
        avgPhie: num(c[9]),
        avgSwe: num(c[10]),
        hpv: num(c[11]),
      }
    }),
  )

describe('pay summary invariants (T-BATCH-07)', () => {
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

    // A summary needs VSH, PHIE and SWE to exist, or every row comes back uninterpreted and the
    // invariants would hold vacuously. Run the standard three through their own modules. The
    // no-default parameters each module now requires are supplied from named sources, none of
    // them a field calibration:
    //  - GR_MA 45 / GR_SH 110 and RHO_SH 2.45: the synthetic generator's own clean-sand and
    //    cap-shale means (tools/make_example_data.py ZONES).
    //  - A=1, M=2, N=2: Archie (1942), the constants the module doc itself cites.
    //  - SWT_IRR 0: no irreducible saturation declared for the synthetic fixture (the manifest's
    //    own lower bound), so the term is the identity.
    //  - RW 0.11 ohmm: back-computed from the generator's water sand with those same constants
    //    (SAND-B: ILD 3.2, RHOB 2.34 -> phi_D = (2.65-2.34)/1.65 = 0.188; Rw = Rt*phi^2 = 0.113).
    const MODULE_PARAMS = {
      vsh_gr: { GR_MA: 45, GR_SH: 110 },
      phi_den: { RHO_SH: 2.45 },
      sw_arch: { A: 1, M: 2, N: 2, SWT_IRR: 0, RW: 0.11 },
    }
    const ids = wells.map((w) => w.well_id)
    const catalog = await invokeOk('list_modules')
    for (const name of ['vsh_gr', 'phi_den', 'sw_arch']) {
      if (!catalog.some((m) => m.name === name)) continue
      // A run now declares its custody (operator + source note; AUTOMATED names the harness
      // honestly) and resolves its wells from a backend scope selector.
      await call('run_workflow_module', {
        req: {
          module: name,
          well_ids: ids,
          log_inputs: {},
          params: MODULE_PARAMS[name],
          opts: {},
          custody: {
            actor: { kind: 'AUTOMATED', identity: 'e2e-harness' },
            source_note: 'e2e fixture run; manifest defaults, no explicit values',
          },
        },
        scope: { kind: 'explicit', well_ids: ids },
      })
    }

    // A zone to report over, on every well.
    for (const w of wells) {
      await call('upsert_zone', {
        wellId: w.well_id,
        zoneName: ZONE,
        topDepth: 1000,
        bottomDepth: 2000,
        // MD is what the frontend's own zone paths send for log measured depths.
        depthDatum: 'MD',
      })
    }

    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
      const btn = Array.from(
        document.querySelectorAll('.ribbon-panel[data-panel="petro"] .ribbon-btn'),
      ).find((b) => /cutoffs\s*&\s*summary/i.test((b.textContent ?? '').replace(/\s+/g, ' ')))
      btn?.click()
    })

    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.summary-pane')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Cutoffs & Summary pane never opened from the Petrophysics ribbon',
      },
    )
  })

  after(async () => {
    for (const w of wells) {
      await call('delete_zone', { wellId: w.well_id, zoneName: ZONE })
    }
  })

  it('produces a summary table from the pane', async () => {
    // Run with the pane's OWN prefilled cutoffs. Nothing here chooses a cutoff — see the header.
    await browser.execute(() => {
      const pane = document.querySelector('.summary-pane')
      pane?.querySelector('.form-run-btn')?.click()
    })

    // Compute Summary writes FLAG curves, so it asks for run custody in a modal before running.
    // Answer it the way an operator would: identity, source note, then the action button.
    await browser.waitUntil(
      async () =>
        await browser.execute(() => {
          const root = document.querySelector('#modal-root')
          const proceed = Array.from(root?.querySelectorAll('button') ?? []).find((b) =>
            /compute and write pay flags/i.test((b.textContent ?? '').trim()),
          )
          if (!proceed) return false
          const identity = root.querySelector('input.form-control')
          const source = root.querySelector('textarea.form-control')
          if (!identity || !source) return false
          identity.value = 'e2e-harness'
          identity.dispatchEvent(new Event('input', { bubbles: true }))
          source.value = 'e2e fixture run over the synthetic examples'
          source.dispatchEvent(new Event('input', { bubbles: true }))
          proceed.click()
          return true
        }),
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg: 'the run-custody dialog never appeared after Compute Summary',
      },
    )

    await browser.waitUntil(async () => (await summaryRows()).length > 0, {
      timeout: 120_000,
      interval: 1000,
      timeoutMsg: `the summary produced no rows; the pane says: ${await browser.execute(
        () => document.querySelector('.summary-pane .modal-result')?.textContent?.trim() ?? '',
      )}`,
    })

    const rows = await summaryRows()
    assert.ok(rows.length > 0, 'the summary must produce rows')
    // Every row must carry one of the three cutoff levels. A blank flag column would make the
    // whole table unreadable — net at which cutoff is the question it exists to answer.
    for (const r of rows) {
      assert.ok(
        ['SAND', 'RESERVOIR', 'PAY'].includes(r.flag),
        `every row must name its cutoff level; got "${r.flag}" for ${r.well}/${r.zone}`,
      )
    }
  })

  it('keeps net within gross, and the three cutoff levels nested', async () => {
    const rows = await summaryRows()

    for (const r of rows) {
      // Gross is GEOMETRY and is always a number, even where nothing was interpreted.
      assert.ok(r.gross !== null && r.gross >= 0, `gross must be a non-negative number (${r.well})`)

      // An uninterpreted row shows "—" rather than 0, which is the whole point of that convention:
      // 0 net is byte-identical to a genuine wet zone, and printing it would state a result the run
      // cannot support. Nothing further can be asserted about such a row.
      if (r.net === null) continue

      assert.ok(
        r.net <= r.gross + 1e-3,
        `net must never exceed gross (${r.well}/${r.zone}/${r.flag}: net ${r.net} > gross ${r.gross})`,
      )
      assert.ok(r.net >= 0, `net must not be negative (${r.well}/${r.zone}/${r.flag})`)
      if (r.ntg !== null) {
        assert.ok(
          r.ntg >= -1e-6 && r.ntg <= 1 + 1e-6,
          `N/G must lie in 0..1 (${r.well}/${r.zone}/${r.flag}: ${r.ntg})`,
        )
      }
    }

    // The nesting: PAY is the strictest cutoff, SAND the loosest, so PAY-net <= RESERVOIR-net <=
    // SAND-net for the same well and zone. This is the invariant a cutoff bug breaks first, and it
    // is invisible in any single row — only the comparison shows it.
    const byKey = new Map()
    for (const r of rows) {
      if (r.net === null) continue
      const key = `${r.well}|${r.zone}`
      if (!byKey.has(key)) byKey.set(key, {})
      byKey.get(key)[r.flag] = r.net
    }
    let compared = 0
    for (const [key, lv] of byKey) {
      if (lv.SAND !== undefined && lv.RESERVOIR !== undefined) {
        compared++
        assert.ok(
          lv.RESERVOIR <= lv.SAND + 1e-3,
          `RESERVOIR net must not exceed SAND net (${key}: ${lv.RESERVOIR} > ${lv.SAND})`,
        )
      }
      if (lv.RESERVOIR !== undefined && lv.PAY !== undefined) {
        compared++
        assert.ok(
          lv.PAY <= lv.RESERVOIR + 1e-3,
          `PAY net must not exceed RESERVOIR net (${key}: ${lv.PAY} > ${lv.RESERVOIR})`,
        )
      }
    }
    assert.ok(
      compared > 0,
      'no well/zone had two cutoff levels to compare — the nesting invariant was never exercised',
    )
  })

  it('keeps HPV within net times porosity', async () => {
    const rows = await summaryRows()
    let checked = 0

    for (const r of rows) {
      if (r.net === null || r.hpv === null || r.avgPhie === null) continue
      checked++
      // HPV is the sum of PHIE*(1-SWE)*thickness over net, so it can never exceed net * PHIE — the
      // water saturation term is a fraction of one. An HPV above that ceiling means the hydrocarbon
      // volume was accumulated over samples the net calculation did not count.
      const ceiling = r.net * r.avgPhie + 1e-2
      assert.ok(
        r.hpv <= ceiling,
        `HPV must not exceed net x avg PHIE (${r.well}/${r.zone}/${r.flag}: ` +
          `hpv ${r.hpv} > ${r.net} x ${r.avgPhie} = ${ceiling.toFixed(3)})`,
      )
    }

    assert.ok(checked > 0, 'no interpreted row was available to check HPV against')

    // NOT asserted: that HPV is non-negative. It is not guaranteed — see finding 16, where a dense
    // stringer inside the net interval is subtracted. Asserting it here would either fail on
    // correct behaviour or quietly encode a claim the code does not make.
  })
})
