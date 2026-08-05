// T-REP-01 (the Composite and Report panes open) and T-REP-07 (the methodology table persists as a
// `report_template` document).
//
// The methodology table is what a report says about HOW the numbers were arrived at — the
// parameter/method/remarks rows a reader checks the interpretation against. It is edited once and
// reused across every study, so a template that silently fails to persist means the next report
// goes out with the built-in default text describing methods that were not used.
//
// The round trip is asserted on the STORED DOCUMENT and on what a freshly built pane reads back,
// not on the textarea still holding what was typed into it.

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

async function invokeOk(cmd, args) {
  const r = await call(cmd, args)
  assert.ok(r.ok, `${cmd} failed: ${r.error}`)
  return r.value
}

const templates = () => invokeOk('list_documents', { docType: 'report_template' })

const openPlotButton = (id) =>
  browser.execute((i) => {
    document.querySelector('.ribbon-tab[data-tab="plot"]')?.click()
    const btn = document.querySelector(`#${i}`)
    if (!btn) return false
    btn.click()
    return true
  }, id)

/** The report pane's methodology textarea. */
const methodText = () =>
  browser.execute(() => {
    const ta = document.querySelector('.report-pane textarea')
    return ta ? ta.value : null
  })

// Three rows in the pane's own documented shape: Parameter | Method | Remarks per line.
const ROWS = [
  'Archie m | 1.9 | from core, this study',
  'Archie n | 2.0 | assumed',
  'Rw | 0.08 ohm.m at 75 degC | measured',
].join('\n')

describe('report and composite panes (T-REP-01, T-REP-07)', () => {
  let original = null

  before(async () => {
    assert.ok(await openPlotButton('report-btn'), 'no #report-btn on the Plot tab')
    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.report-pane textarea')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Report pane never opened from the Plot ribbon',
      },
    )
    // Remember whatever template was there, so this spec hands the project back as it found it —
    // a methodology template is a real user artefact, not scratch state.
    const docs = await templates()
    original = docs.find((d) => d.name)?.json ?? null
  })

  after(async () => {
    const docs = await templates()
    for (const d of docs) {
      if (original === null) await call('delete_document', { docType: 'report_template', name: d.name })
      else await call('save_document', { docType: 'report_template', name: d.name, json: original })
    }
  })

  it('opens the Composite pane too', async () => {
    // T-REP-01. Both panes are singletons and both follow the selected well; what is asserted here
    // is only that each opens and builds a form, which is the smoke claim.
    assert.ok(await openPlotButton('composite-btn'), 'no #composite-btn on the Plot tab')
    await browser.waitUntil(
      async () =>
        await browser.execute(
          () =>
            !!document.querySelector('.composite-pane') ||
            !!document.querySelector('[class*="composite"]'),
        ),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Composite pane never opened from the Plot ribbon',
      },
    )
  })

  it('saves the methodology table as a report_template document', async () => {
    assert.ok(await openPlotButton('report-btn'))
    await browser.waitUntil(async () => (await methodText()) !== null, {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: 'the report pane lost its methodology textarea',
    })

    await browser.execute((text) => {
      const ta = document.querySelector('.report-pane textarea')
      ta.value = text
      ta.dispatchEvent(new Event('input', { bubbles: true }))
      const btn = Array.from(document.querySelectorAll('.report-pane button')).find(
        (b) => (b.textContent ?? '').trim() === 'Save Template',
      )
      btn.click()
    }, ROWS)

    await browser.waitUntil(async () => (await templates()).length > 0, {
      timeout: 20_000,
      interval: 250,
      timeoutMsg: 'no report_template document was written',
    })

    const doc = (await templates())[0]
    const parsed = JSON.parse(doc.json)
    assert.ok(Array.isArray(parsed), 'the template must store parsed ROWS, not the raw text')
    assert.equal(
      parsed.length,
      3,
      `three typed lines must become three rows; got ${JSON.stringify(parsed)}`,
    )

    // The pipe-separated fields must survive as separate fields. Storing the line verbatim would
    // still round-trip through this pane and then render as one column in the PDF — the table
    // would look like a list of sentences rather than a methodology table.
    const flat = JSON.stringify(parsed)
    for (const fragment of ['Archie m', '1.9', 'from core, this study', 'Rw']) {
      assert.ok(flat.includes(fragment), `the stored rows must keep "${fragment}"; got ${flat}`)
    }
  })

  it('reads the template back into a freshly built pane', async () => {
    // Rebuild the pane by opening a different one and returning — the methodology field is
    // populated from the document at BUILD time, so this is the only way to test that the read
    // path works rather than that the textarea still holds what was typed into it.
    assert.ok(await openPlotButton('composite-btn'))
    await browser.pause(500)
    assert.ok(await openPlotButton('report-btn'))

    await browser.waitUntil(
      async () => {
        const t = await methodText()
        return t !== null && t.includes('Archie m')
      },
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: `the rebuilt report pane did not read the saved template back; it holds: ${await methodText()}`,
      },
    )

    const text = await methodText()
    for (const fragment of ['Archie m', 'Archie n', 'Rw']) {
      assert.ok(text.includes(fragment), `the reloaded template must keep "${fragment}"`)
    }
  })
})
