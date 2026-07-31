// The module pane and the two refusals that stop a bad run before it starts.
//
// Covers T-PREP-01 (dialog machinery: the ribbon dropdown, the auto-generated form, the leading
// "(none)" option) and the frontend-validation legs of T-PETRO-03, T-INT-06, T-ADV-18 and
// T-BATCH-06 — all of which are the SAME two guards in `moduleDialog.ts`, reached from different
// modules.
//
// Both guards are frontend-only and cannot be pinned by a Rust test. The backend has no idea a
// dialog exists: hand `run_workflow_module` an out-of-range parameter and it computes with it, and
// hand it an empty well list and it cheerfully does nothing and reports success. What stops either
// reaching the engine is nine lines in a click handler, and nothing else in the repo checks them.
//
// The important assertion in both is NOT the message — it is that NO RUN STARTED. A dialog that
// prints a complaint and then runs anyway is the failure worth catching, and only a before/after
// comparison of what is stored can tell those apart.

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
 * Everything stored in `computed_curves`, project-wide, as one comparable string.
 *
 * Project-wide rather than per well because "no run started" is a claim about the whole scope —
 * a guard that leaked would write to whichever wells the scope resolved to, and checking one well
 * could miss it entirely.
 */
async function projectFingerprint() {
  const page = await invokeOk('run_query', {
    sql:
      'SELECT well_id, curve_name, COUNT(*) AS n, COALESCE(SUM(value), 0) AS total ' +
      'FROM computed_curves GROUP BY well_id, curve_name ORDER BY well_id, curve_name',
    limit: 5000,
  })
  return (page.rows ?? []).map((r) => r.join('|')).join('\n')
}

/** The module pane's one-line result box. */
const resultText = () =>
  browser.execute(
    () => document.querySelector('.module-pane .modal-result')?.textContent?.trim() ?? '(no result box)',
  )

describe('module pane and its refusals (T-PREP-01, T-PETRO-03, T-INT-06)', () => {
  before(async () => {
    const existing = await invokeOk('list_wells')
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      await invokeOk('import_las_files', { paths, setName: 'E2E', attach: false })
    }

    // Open a module pane the way a user does: the Petrophysics tab, a ribbon dropdown, an item.
    // Driving `openModulePane` directly would skip the dropdown, which is half of T-PREP-01.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
    })

    // The menu item's text is the module's own manifest TITLE, so read it from the manifest rather
    // than hard-coding a guess at the wording — a first attempt searched for "Shale Volume" and
    // found nothing, because vsh_gr is titled "VSH from Gamma Ray". Taking the title from
    // `list_modules` also means a future rename moves both sides together instead of leaving a
    // test hunting for a string that no longer exists.
    const modules = await invokeOk('list_modules')
    const spec = modules.find((m) => m.name === 'vsh_gr')
    assert.ok(spec, 'vsh_gr must be in the module catalog')

    const opened = await browser.waitUntil(
      async () =>
        await browser.execute((title) => {
          if (document.querySelector('.module-pane')) return true
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
              return true
            }
          }
          // Close whatever was left open rather than leaving a menu over the ribbon.
          document
            .querySelectorAll('.ribbon-menu:not([hidden])')
            .forEach((m) => m.setAttribute('hidden', ''))
          return false
        }, spec.title),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: `no "${spec.title}" entry found in any Petrophysics ribbon dropdown`,
      },
    )
    assert.ok(opened)

    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.module-pane .form-run-btn')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the module pane never built its form',
      },
    )
  })

  it('builds the pane form from the manifest', async () => {
    const form = await browser.execute(() => {
      const pane = document.querySelector('.module-pane')
      const selects = Array.from(pane.querySelectorAll('select'))
      return {
        hasScope: !!pane.querySelector('.well-scope'),
        hasRun: !!pane.querySelector('.form-run-btn'),
        numberInputs: pane.querySelectorAll('input[type="number"]').length,
        outputsNote:
          Array.from(pane.querySelectorAll('.modal-hint'))
            .map((p) => p.textContent.trim())
            .find((t) => t.startsWith('Outputs:')) ?? null,
        // Curve pickers lead with "(none)" so a run can leave an optional input unbound. A picker
        // that lost it would silently bind the first curve in the list instead — the module would
        // run, on a curve nobody chose.
        curveSelectsWithNone: selects.filter(
          (s) => (s.options[0]?.textContent ?? '').trim() === '(none)',
        ).length,
      }
    })

    assert.ok(form.hasScope, 'the pane must carry a well-scope control')
    assert.ok(form.hasRun, 'the pane must carry a Run button')
    assert.ok(form.numberInputs > 0, 'a vsh_gr form must expose its numeric parameters')
    assert.ok(
      form.outputsNote,
      'the pane must state which curves the run writes — that note is the only place a user is ' +
        'told before pressing Run',
    )
    assert.ok(
      form.curveSelectsWithNone > 0,
      'at least one curve picker must lead with "(none)"',
    )
  })

  it('refuses an out-of-range parameter, and starts no run', async () => {
    const before = await projectFingerprint()

    const set = await browser.execute(() => {
      const input = document.querySelector('.module-pane input[type="number"]')
      if (!input) return null
      input.value = '999999'
      input.dispatchEvent(new Event('input', { bubbles: true }))
      document.querySelector('.module-pane .form-run-btn').click()
      return true
    })
    assert.ok(set, 'no numeric parameter input in the module pane')

    await browser.waitUntil(async () => /value must be between/i.test(await resultText()), {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `the pane never refused the out-of-range value; it said: ${await resultText()}`,
    })

    const msg = await resultText()
    // The message must NAME the parameter and its bounds. "Invalid input" would be true and
    // useless: the form has several numeric fields and the user has to know which one to fix.
    assert.match(
      msg,
      /^\S+: value must be between .+ and .+\.$/,
      `the refusal must name the parameter and its range; got: ${msg}`,
    )

    // The claim that matters. A dialog that complains and runs anyway looks identical from the
    // message alone.
    assert.equal(
      await projectFingerprint(),
      before,
      'an out-of-range parameter must stop the run before anything is written',
    )
  })

  it('refuses an empty scope, and starts no run', async () => {
    const before = await projectFingerprint()

    // Put the scope on Selection with nothing selected. This is the reachable empty scope — "All"
    // cannot be empty while wells exist, and it is exactly the state a user lands in after
    // clearing a multi-selection and then pressing Run out of habit.
    await browser.execute(() => {
      const rows = Array.from(document.querySelectorAll('.tree-node.tree-well'))
      // A plain click clears any multi-selection (see wells.e2e.mjs).
      rows[0]?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
      const mode = Array.from(document.querySelectorAll('.module-pane .well-scope-mode')).find(
        (b) => (b.textContent ?? '').trim() === 'Selection',
      )
      mode?.click()
    })

    // Restore a valid parameter first, or this would be re-testing the previous guard: the range
    // check runs AFTER the scope check, so a still-invalid field would never be reached — but
    // leaving it invalid would make this test pass for the wrong reason if the order ever changed.
    await browser.execute(() => {
      const input = document.querySelector('.module-pane input[type="number"]')
      if (input) {
        input.value = '10'
        input.dispatchEvent(new Event('input', { bubbles: true }))
      }
      document.querySelector('.module-pane .form-run-btn').click()
    })

    await browser.waitUntil(async () => /no wells in scope/i.test(await resultText()), {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `the pane never refused the empty scope; it said: ${await resultText()}`,
    })

    const msg = await resultText()
    // The message must say what to DO about it. A bare "no wells selected" leaves the user looking
    // at a Run button that does nothing, which is the complaint this wording exists to answer.
    assert.match(
      msg,
      /pick a group|select wells|choose All/i,
      `the refusal must tell the user how to fix it; got: ${msg}`,
    )

    assert.equal(
      await projectFingerprint(),
      before,
      'an empty scope must stop the run before anything is written',
    )
  })
})
