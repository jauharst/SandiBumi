// The despike pane's live contamination ceiling (SB-ENV-031).
//
// `despike_contamination_preview` resolves its well scope through the backend registry
// (`well_scope::WELL_SCOPE_OPERATIONS`), and the command shipped without its registry row — so
// the card refused every invocation with "unregistered backend well-scope operation" while every
// Rust test stayed green: the registry tests walked registry -> source, and an operation string
// the registry never held was on neither side. The reverse pin now lives in
// `well_scope::tests::every_operation_string_a_command_resolves_is_registered`; this spec is the
// running-app half — the card the user actually reads must show an evaluated ceiling.

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

/** The despike pane is the one whose contamination card is not hidden — the card exists on every
 *  module pane and `buildModuleContent` unhides it only for `spec.name === "despike"`, so this
 *  also keeps working when another spec has left a different module pane open. */
const ceilingText = () =>
  browser.execute(() => {
    const card = Array.from(document.querySelectorAll('.module-contamination')).find(
      (c) => !c.hidden,
    )
    return card?.querySelector('.module-contamination-body')?.textContent?.trim() ?? '(no card)'
  })

describe('despike live contamination ceiling (SB-ENV-031)', () => {
  before(async () => {
    const existing = await invokeOk('list_wells', { scope: { kind: 'all' } })
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      // The import refuses without a declared sampling style (never sniffed from the depths)
      // and, for CONTINUOUS_REGULAR, a declared step tolerance (it belongs to the logging tool,
      // so no default ships). The SANDI examples are synthetic regular-step files; 0.01 m is a
      // test input for them, not a recommended field value.
      await invokeOk('import_las_files', {
        paths,
        setName: 'E2E',
        attach: false,
        samplingStyle: 'CONTINUOUS_REGULAR',
        samplingStyleVerifyTolerance: { value: 0.01, unit: 'M' },
      })
    }

    // Seeding through `invoke` tells the frontend nothing (see the harness doc): the scope
    // control's All resolves the frontend's own well inventory, which is empty until the Wells
    // pane re-reads. Refresh it through the pane's own group select, exactly as wells.e2e.mjs
    // does — dispatched unconditionally, because its handler ends in refresh().
    await browser.execute(() => {
      const sel = document.querySelector('.tree-group-select')
      if (!sel) return
      sel.value = ''
      sel.dispatchEvent(new Event('change', { bubbles: true }))
    })
    await browser.waitUntil(
      async () =>
        await browser.execute(() => document.querySelectorAll('.tree-node.tree-well').length >= 3),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Wells pane never showed the imported wells',
      },
    )

    // Open the pane the way a user does: the manifest title in a ribbon category dropdown.
    // The title comes from `list_modules` (see moduledialog.e2e.mjs for why it is never
    // hard-coded), and "a module pane exists" is not enough of an open check here — another
    // spec's vsh_gr pane would satisfy it — so the check is the unhidden contamination card.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
    })
    const modules = await invokeOk('list_modules')
    const spec = modules.find((m) => m.name === 'despike')
    assert.ok(spec, 'despike must be in the module catalog')

    const opened = await browser.waitUntil(
      async () =>
        await browser.execute((title) => {
          const despikeCard = Array.from(
            document.querySelectorAll('.module-contamination'),
          ).find((c) => !c.hidden)
          if (despikeCard) return true
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
      async () =>
        await browser.execute(() =>
          Array.from(document.querySelectorAll('.module-contamination')).some((c) => !c.hidden),
        ),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the despike pane never built its contamination card',
      },
    )
  })

  it('evaluates a real ceiling instead of refusing the operation as unregistered', async () => {
    // WINDOW deliberately has no default (what counts as a spike is basin-specific), so a fresh
    // pane shows the set-WINDOW-and-K prompt and never calls the backend. Bind the curve and give
    // WINDOW a test thickness through the pane's own controls, so the refresh path under test is
    // the one a user's keystroke takes. 5 depth units covers >= 5 samples at any example-well
    // sampling, which is the Hampel floor; it is a test input, not a recommended value.
    const wired = await browser.execute(() => {
      const card = Array.from(document.querySelectorAll('.module-contamination')).find(
        (c) => !c.hidden,
      )
      const pane = card?.closest('.module-pane')
      if (!pane) return { ok: false, why: 'no despike pane' }
      // Scope on All: the evaluation needs wells in scope, and All is the one mode that cannot
      // be empty while wells exist. Restored expectation for later specs: All is also what
      // moduledialog.e2e.mjs leaves its pane on.
      const all = Array.from(pane.querySelectorAll('.well-scope-mode')).find(
        (b) => (b.textContent ?? '').trim() === 'All',
      )
      all?.click()
      // Form rows label their controls with the manifest ARG NAME (verified on the running
      // app), and `formRow` links label to control via htmlFor — so the arg name is the
      // stable handle, not the descriptive caption.
      const control = (labelText) => {
        const label = Array.from(pane.querySelectorAll('label')).find(
          (l) => (l.textContent ?? '').trim() === labelText,
        )
        if (!label) return null
        return label.htmlFor
          ? document.getElementById(label.htmlFor)
          : label.parentElement?.querySelector('input, select') ?? null
      }
      const curve = control('CURVE')
      if (curve && curve.tagName === 'SELECT') {
        const gr = Array.from(curve.options).find((o) => (o.textContent ?? '').trim() === 'GR')
        if (!gr) return { ok: false, why: 'no GR option in the curve picker' }
        if (curve.value !== gr.value) {
          curve.value = gr.value
          curve.dispatchEvent(new Event('change', { bubbles: true }))
        }
      } else if (!curve) {
        return { ok: false, why: 'no CURVE control' }
      }
      const window_ = control('WINDOW')
      if (!window_) return { ok: false, why: 'no WINDOW input' }
      window_.value = '5'
      window_.dispatchEvent(new Event('input', { bubbles: true }))
      return { ok: true }
    })
    assert.ok(wired.ok, `could not drive the despike form: ${wired.why}`)

    // The claim under test, both sides: the exact shipped refusal is gone, and in its place is an
    // evaluated estimator branch with a percentage — not merely a different message.
    await browser.waitUntil(
      async () => {
        const text = await ceilingText()
        return /True MAD|Mean-deviation fallback/.test(text) && /%/.test(text)
      },
      {
        timeout: 20_000,
        interval: 300,
        timeoutMsg: `the ceiling was never evaluated; the card says: ${await ceilingText()}`,
      },
    )
    const text = await ceilingText()
    assert.ok(
      !text.includes('unregistered backend well-scope operation'),
      `the shipped refusal is still there: ${text}`,
    )
    assert.match(
      text,
      /evaluated sample window/,
      `a branch row must state its sample count; got: ${text}`,
    )
  })
})
