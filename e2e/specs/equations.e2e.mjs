// T-MLEQ-02 (the Inspector shows which Python engine it found) and T-MLEQ-05 step 1 (running an
// unsaved equation is refused).
//
// Both are frontend-only. The backend has no notion of an unsaved equation — `run_equation` takes
// an id, and there is no id to pass — so the guard is a single early return in `handleRun`, and
// nothing else in the repo checks it. The engine note is the same shape: `python_status` is probed
// once per session, and whether its answer reaches the user before they write a script is a
// property of the panel.

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

/**
 * The equation editor's OWN status line.
 *
 * Not the global `#status-bar`: `InspectorPanel.setStatus` writes to `#eq-status`, beside the
 * Save/Run buttons. That is the right place for it — a refusal about the form you are looking at
 * belongs next to the form, not in a corner of the window — and it is also why a test looking at
 * the status bar sees nothing at all.
 */
const eqStatus = () =>
  browser.execute(() => {
    const el = document.querySelector('#eq-status')
    return el && !el.hidden ? el.textContent.trim() : ''
  })

describe('equation editor (T-MLEQ-02, T-MLEQ-05)', () => {
  before(async () => {
    // The equation editor is the Inspector's other tab. Open the panel, then switch to it by its
    // visible label rather than by index — the tab order is a layout choice.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="data"]')?.click()
      document.querySelector('#open-inspector-btn')?.click()
    })

    await browser.waitUntil(
      async () =>
        await browser.execute(() => {
          const tab = Array.from(document.querySelectorAll('button, .tab, [role="tab"]')).find((b) =>
            /^equations?$/i.test((b.textContent ?? '').trim()),
          )
          if (tab) tab.click()
          return !!document.querySelector('#eq-run')
        }),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the equation editor never appeared in the Inspector',
      },
    )
  })

  it('names the Python engine it found, or says plainly that it found none', async () => {
    // T-MLEQ-02. The note is written beside the Language picker so it is read WHILE the script is
    // being written, not after a run has already failed — which is the whole point of probing once
    // per session rather than at run time.
    const note = await browser.execute(
      () => document.querySelector('#eq-lang-note')?.textContent?.trim() ?? '',
    )
    assert.ok(note.length > 0, 'the equation editor must carry a language note')

    const probe = await call('python_status')
    assert.ok(probe.ok, `python_status failed: ${probe.error}`)
    const info = probe.value

    if (info?.path) {
      assert.ok(
        note.includes(info.path),
        `the note must name the interpreter it will actually use; it reads: ${note}`,
      )
      // scipy is OPTIONAL, so its absence is a note and not a warning — the engine is fully usable
      // without it, and calling it a warning would send the user installing something they may not
      // need. The distinction is checked because it is easy to lose in a refactor.
      if (info.scipy === null) {
        assert.ok(
          /no scipy/i.test(note) && !/⚠/.test(note),
          `missing scipy must read as a note, not a warning; it reads: ${note}`,
        )
      } else {
        assert.ok(
          note.includes(String(info.scipy)),
          `the note must name the scipy version it found; it reads: ${note}`,
        )
      }
    } else {
      // The no-Python case must be a WARNING, and must say what to do about it — including the
      // environment variable, which is the only fix when discovery cannot find a working install.
      assert.ok(/⚠/.test(note), `no interpreter must read as a warning; it reads: ${note}`)
      assert.ok(
        /numpy/i.test(note) && /SANDIBUMI_PYTHON/.test(note),
        `and must name numpy and the override variable; it reads: ${note}`,
      )
    }
  })

  it('refuses to run an equation that has never been saved', async () => {
    // T-MLEQ-05 step 1. Fill the form WITHOUT saving, then press Run. The refusal exists because
    // an equation only becomes runnable once it has an id — and without the guard the run would
    // fail somewhere deeper with a message about a missing id, which tells the user nothing about
    // what to do.
    await browser.execute(() => {
      const set = (id, v) => {
        const el = document.querySelector(id)
        if (el) {
          el.value = v
          el.dispatchEvent(new Event('input', { bubbles: true }))
        }
      }
      // "— New equation —" so there is no id behind the form.
      const sel = document.querySelector('#eq-select')
      if (sel) {
        sel.value = ''
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }
      set('#eq-name', 'E2E_UNSAVED')
      set('#eq-inputs', 'GR')
      set('#eq-output', 'E2E_UNSAVED')
      document.querySelector('#eq-run')?.click()
    })

    await browser.waitUntil(async () => /save the equation before running/i.test(await eqStatus()), {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `an unsaved equation was not refused; #eq-status reads: ${await eqStatus()}`,
    })

    // And nothing was written. The message alone would be satisfied by a guard that complains and
    // runs anyway — the same failure shape as the module dialog's two refusals.
    const eqs = await call('list_equations')
    if (eqs.ok) {
      assert.ok(
        !(eqs.value ?? []).some((e) => e.name === 'E2E_UNSAVED'),
        'a refused run must not leave the equation saved as a side effect',
      )
    }
  })
})
