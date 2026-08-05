// T-BATCH-01 (Workflow Builder smoke, step picker clean), T-BATCH-04 (save / reload / delete the
// chain as a `workflow` document) and the two refusals from T-BATCH-06.
//
// A chain is an ordered recipe run across a whole field, so a saved chain that reloads WRONG is
// the quietest failure in the app: it runs, it writes curves, and the only evidence is numbers
// that came from a different recipe than the one on screen. That is why the round trip here is
// asserted on the STORED DOCUMENT and on what comes back after a reload, not on the dialog
// looking right.

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

const CHAIN = 'E2E-CHAIN'

const workflows = () => invokeOk('list_documents', { docType: 'workflow' })

const statusText = () =>
  browser.execute(() => document.querySelector('#status-bar')?.textContent?.trim() ?? '')

/** The builder's current step titles. */
const stepTitles = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.workflow-dialog .workflow-step')).map(
      (li) => li.querySelector('.workflow-step-title')?.textContent?.trim() ?? '',
    ),
  )

/** Click a button in the builder by its exact label. */
const clickBuilder = (label) =>
  browser.execute((t) => {
    const btn = Array.from(document.querySelectorAll('.workflow-dialog button')).find(
      (b) => (b.textContent ?? '').trim() === t,
    )
    if (!btn) return false
    btn.click()
    return true
  }, label)

describe('workflow builder (T-BATCH-01, T-BATCH-04, T-BATCH-06)', () => {
  before(async () => {
    await call('delete_document', { docType: 'workflow', name: CHAIN })

    // Open the builder from its real ribbon button.
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
      const btn = Array.from(
        document.querySelectorAll('.ribbon-panel[data-panel="petro"] .ribbon-btn'),
      ).find((b) => /workflow/i.test(b.textContent ?? ''))
      btn?.click()
    })

    await browser.waitUntil(
      async () =>
        await browser.execute(() => !!document.querySelector('.workflow-dialog .workflow-add-row')),
      {
        timeout: 30_000,
        interval: 500,
        timeoutMsg: 'the Workflow Builder never opened from the Petrophysics ribbon',
      },
    )
  })

  after(async () => {
    await call('delete_document', { docType: 'workflow', name: CHAIN })
  })

  it('offers a step picker with no retired module in it', async () => {
    const picker = await browser.execute(() => {
      const sel = document.querySelector('.workflow-dialog .workflow-add-row select')
      return {
        groups: Array.from(sel.querySelectorAll('optgroup')).map((g) => g.label),
        values: Array.from(sel.querySelectorAll('option')).map((o) => o.value),
        labels: Array.from(sel.querySelectorAll('option')).map((o) => o.textContent.trim()),
      }
    })

    assert.ok(picker.groups.length > 0, 'the picker must be grouped by module category')
    assert.ok(picker.values.length > 3, 'the picker must offer the module catalog')

    // T-BATCH-01's negative, and the half of T-RT-16 the ribbon sweep does not reach. The retired
    // fixed-component solver must not be addable to a NEW chain — a chain built today that quietly
    // wired it up would be refused at run time by `modules::retired_module`, after the user had
    // arranged the whole recipe.
    assert.ok(
      !picker.values.includes('multimin'),
      `the retired multimin must not be offered as a chain step; picker has: ${picker.values.join(', ')}`,
    )
    assert.ok(
      !picker.labels.some((l) => /mineral inversion/i.test(l)),
      'nor under its old display name',
    )
  })

  it('refuses to save an unnamed chain, and one with no steps', async () => {
    // Both refusals are frontend-only — `save_document` would happily store either. The order
    // matters: name is checked first, so the no-steps message can only be reached once a name is
    // typed, and a test that set neither would only ever see the first.
    await browser.execute(() => {
      const input = document.querySelector('.workflow-dialog input[placeholder="workflow name"]')
      input.value = ''
      input.dispatchEvent(new Event('input', { bubbles: true }))
    })
    assert.ok(await clickBuilder('Save'), 'no Save button in the builder')

    await browser.waitUntil(async () => /workflow name first/i.test(await statusText()), {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `an unnamed chain was not refused; status reads: ${await statusText()}`,
    })

    // Now name it but leave it empty of steps.
    await browser.execute((name) => {
      const input = document.querySelector('.workflow-dialog input[placeholder="workflow name"]')
      input.value = name
      input.dispatchEvent(new Event('input', { bubbles: true }))
    }, CHAIN)
    assert.ok(await clickBuilder('Save'))

    await browser.waitUntil(async () => /at least one step/i.test(await statusText()), {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `a stepless chain was not refused; status reads: ${await statusText()}`,
    })

    const saved = (await workflows()).map((d) => d.name)
    assert.ok(
      !saved.includes(CHAIN),
      'neither refusal may leave a document behind — a half-saved chain is worse than none',
    )
  })

  it('saves a two-step chain as a workflow document', async () => {
    // Two steps in a deliberate ORDER, because order is the whole content of a chain: VSH before
    // porosity, porosity before saturation. A round trip that preserved the set but not the
    // sequence would still look right in the list.
    const picked = await browser.execute(() => {
      const sel = document.querySelector('.workflow-dialog .workflow-add-row select')
      const add = Array.from(document.querySelectorAll('.workflow-dialog button')).find(
        (b) => (b.textContent ?? '').trim() === '+ Add step',
      )
      const options = Array.from(sel.querySelectorAll('option')).map((o) => o.value)
      const first = options.find((v) => v === 'vsh_gr') ?? options[0]
      const second = options.find((v) => v !== first)
      sel.value = first
      add.click()
      sel.value = second
      add.click()
      return [first, second]
    })

    await browser.waitUntil(async () => (await stepTitles()).length === 2, {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: `the builder did not take two steps; it lists ${(await stepTitles()).join(', ')}`,
    })

    assert.ok(await clickBuilder('Save'))
    await browser.waitUntil(async () => (await workflows()).some((d) => d.name === CHAIN), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the chain was never written to the documents store',
    })

    const doc = (await workflows()).find((d) => d.name === CHAIN)
    const parsed = JSON.parse(doc.json)
    const names = (parsed.steps ?? []).map((s) => s.module ?? s.name)
    assert.deepEqual(
      names,
      picked,
      'the stored chain must carry the steps IN THE ORDER they were added — a chain that keeps ' +
        'the set but loses the sequence runs a different recipe and looks identical in the list',
    )
  })

  it('reloads the saved chain, restoring its steps in order', async () => {
    // Clear the builder first, so a reload that did nothing at all could not pass by leaving the
    // steps that were already on screen.
    await browser.execute(() => {
      Array.from(document.querySelectorAll('.workflow-dialog .workflow-step-ctrls button'))
        .filter((b) => (b.textContent ?? '').trim() === '✕' || /remove|delete/i.test(b.title ?? ''))
        .forEach((b) => b.click())
    })

    const doc = (await workflows()).find((d) => d.name === CHAIN)
    const expected = JSON.parse(doc.json).steps.map((s) => s.module ?? s.name)

    await browser.execute((name) => {
      const sel = document.querySelector('.workflow-dialog .workflow-saved-row select')
      sel.value = name
      sel.dispatchEvent(new Event('change', { bubbles: true }))
      Array.from(document.querySelectorAll('.workflow-dialog .workflow-saved-row button'))
        .find((b) => (b.textContent ?? '').trim() === 'Load')
        .click()
    }, CHAIN)

    await browser.waitUntil(async () => /Loaded workflow/i.test(await statusText()), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: `Load never reported; status reads: ${await statusText()}`,
    })

    await browser.waitUntil(async () => (await stepTitles()).length === expected.length, {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: `the reloaded chain shows ${(await stepTitles()).length} steps, expected ${expected.length}`,
    })

    // Re-save and compare the DOCUMENT, which is the honest round trip: the titles on screen are a
    // rendering, but what runs is the stored JSON.
    assert.ok(await clickBuilder('Save'))
    await browser.waitUntil(
      async () => {
        const again = (await workflows()).find((d) => d.name === CHAIN)
        const names = JSON.parse(again.json).steps.map((s) => s.module ?? s.name)
        return names.join(',') === expected.join(',')
      },
      {
        timeout: 15_000,
        interval: 250,
        timeoutMsg: 'a save → load → save round trip did not return the same ordered steps',
      },
    )
  })

  it('deletes the chain from the store', async () => {
    await browser.execute((name) => {
      const sel = document.querySelector('.workflow-dialog .workflow-saved-row select')
      sel.value = name
      sel.dispatchEvent(new Event('change', { bubbles: true }))
      Array.from(document.querySelectorAll('.workflow-dialog .workflow-saved-row button'))
        .find((b) => (b.textContent ?? '').trim() === 'Delete')
        .click()
    }, CHAIN)

    await browser.waitUntil(async () => !(await workflows()).some((d) => d.name === CHAIN), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the chain was never deleted from the documents store',
    })
  })
})
