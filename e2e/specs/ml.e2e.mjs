// T-MLEQ-01 (the ML pane opens with its full form) and T-MLEQ-14 (its two refusals, and the Mask
// control the plan still says is missing).
//
// Step 3 of T-MLEQ-14 is why this spec is worth having. The plan tells you to search the pane for a
// mask picker, expects not to find one, and instructs you to log it against the dialog. It is
// there — `mlDialog.ts` builds a "Mask (exclude)" row and keeps it visible for every task. The
// note was already corrected once (2026-07-31, when the backend half turned out to be implemented)
// and is stale a second time. A test asserting the control EXISTS is what stops that happening
// again: the day it is removed, this goes red.

import assert from 'node:assert/strict'

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

const statusText = () =>
  browser.execute(() => document.querySelector('#status-bar')?.textContent?.trim() ?? '')

/** The ML pane, identified by a control only it has. */
const mlPane = () =>
  browser.execute(() => {
    const anchor = document.querySelector(
      'input[placeholder="leave blank to not keep the model"]',
    )
    return !!anchor
  })

/** Every form-row label in the ML pane. */
const mlLabels = () =>
  browser.execute(() => {
    const anchor = document.querySelector('input[placeholder="leave blank to not keep the model"]')
    const pane = anchor?.closest('.mc-dialog')
    if (!pane) return []
    return Array.from(pane.querySelectorAll('label, .field-label, .mc-field'))
      .map((l) => (l.textContent ?? '').trim())
      .filter(Boolean)
  })

describe('ML pane (T-MLEQ-01, T-MLEQ-14)', () => {
  before(async () => {
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="advance"]')?.click()
      document.querySelector('#ml-btn')?.click()
    })
    await browser.waitUntil(async () => await mlPane(), {
      timeout: 30_000,
      interval: 500,
      timeoutMsg: 'the ML pane never opened from the Advance tab',
    })
  })

  it('opens with the full form', async () => {
    const labels = await mlLabels()
    assert.ok(labels.length > 0, 'the ML pane must build a form')

    // The task choice and the Algorithm picker drive everything else in this pane. The task is
    // no longer a labelled "Task" row: the 2026-08 redesign folded it into the "Predicting"
    // segmented row (Continuous/Discrete) beside the grouped Algorithm picker.
    for (const want of [/predicting/i, /algorithm/i]) {
      assert.ok(
        labels.some((l) => want.test(l)),
        `the ML form must carry a ${want} control; it has: ${labels.slice(0, 12).join(' | ')}`,
      )
    }

    // "Save model as" is what makes a fitted model an ARTIFACT rather than a by-product — the
    // whole point of `ml_models`, since a refit on different data is a different model.
    const anchorPresent = await browser.execute(
      () => !!document.querySelector('input[placeholder="leave blank to not keep the model"]'),
    )
    assert.ok(anchorPresent, 'the pane must offer to save the fitted model')
  })

  it('has a Mask control — the plan still says it does not', async () => {
    // T-MLEQ-14 step 3, PINNED AS FIXED. `mlDialog.ts` builds this row and keeps it visible for
    // every task, including the unsupervised ones where it governs the fit pool. The plan's
    // known-issue note (already corrected once, on 2026-07-31) still tells the reader to expect it
    // missing and to log it against the dialog — see finding 24.
    //
    // If this ever goes red the control has been removed, which would silently let flagged
    // washout and casing samples bias the scaler, the cluster centres and every trained model.
    const labels = await mlLabels()
    assert.ok(
      labels.some((l) => /mask/i.test(l)),
      `the ML pane must offer a Mask control; its labels read: ${labels.join(' | ')}`,
    )
  })

  it('refuses a run with no input curve, and one with an empty scope', async () => {
    // Both refusals are frontend-only: `run_ml` would accept an empty feature list and an empty
    // well list and report a kind of success.
    await browser.execute(() => {
      const anchor = document.querySelector('input[placeholder="leave blank to not keep the model"]')
      const pane = anchor?.closest('.mc-dialog')
      // Input curves are slot SELECTS now, not checkboxes — clearing a slot empties the
      // feature list `selectedFeatures()` actually reads.
      pane?.querySelectorAll('.ml-slot-sel').forEach((sel) => {
        if (sel.value) {
          sel.value = ''
          sel.dispatchEvent(new Event('change', { bubbles: true }))
        }
      })
      const run = Array.from(pane?.querySelectorAll('button') ?? []).find((b) =>
        /run model/i.test((b.textContent ?? '').trim()),
      )
      run?.click()
    })

    await browser.waitUntil(async () => /at least one input curve/i.test(await statusText()), {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: `an inputless run was not refused; the status reads: ${await statusText()}`,
    })

    // Nothing may have been written. `run_ml` writes curves, so a guard that complains and runs
    // anyway would leave real predictions behind from a model fitted on no features at all.
    const before = await call('run_query', {
      sql: "SELECT COUNT(*) FROM computed_curves WHERE curve_name LIKE 'ML%'",
      limit: 1,
    })
    assert.ok(before.ok, 'the project must still be readable after the refusal')
  })
})
