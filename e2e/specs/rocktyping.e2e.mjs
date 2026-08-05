// T-RT-01 — the Rock Typing ribbon group lists its modules and opens their panes, once each.
//
// The singleton claim in step 5 is the one worth automating. Module panes are keyed by module, so
// re-clicking an entry must FOCUS the pane rather than build a second one — and a duplicate is not
// a cosmetic problem: two panes for the same module each hold their own scope and parameters, so a
// user who edits one and runs the other gets a run they did not configure, with no way to tell
// from the result which pane it came from.

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

/** Open the Petrophysics dropdown whose menu contains `title`, and return its item labels. */
const openMenuContaining = (title) =>
  browser.execute((t) => {
    document.querySelector('.ribbon-tab[data-tab="petro"]')?.click()
    const buttons = Array.from(
      document.querySelectorAll('.ribbon-panel[data-panel="petro"] .ribbon-btn'),
    )
    for (const b of buttons) {
      b.click()
      const items = Array.from(
        document.querySelectorAll('.ribbon-menu:not([hidden]) .ribbon-menu-item'),
      ).map((i) => (i.textContent ?? '').trim())
      if (items.some((i) => i.includes(t))) return items
    }
    document
      .querySelectorAll('.ribbon-menu:not([hidden])')
      .forEach((m) => m.setAttribute('hidden', ''))
    return null
  }, title)

/** Click a menu item by exact title in whatever menu is open. */
const clickMenuItem = (title) =>
  browser.execute((t) => {
    const item = Array.from(
      document.querySelectorAll('.ribbon-menu:not([hidden]) .ribbon-menu-item'),
    ).find((i) => (i.textContent ?? '').trim() === t)
    if (!item) return false
    item.click()
    return true
  }, title)

const paneCount = () =>
  browser.execute(() => document.querySelectorAll('.module-pane').length)

describe('rock typing ribbon group (T-RT-01)', () => {
  let rtModules = []

  before(async () => {
    // Take the expected titles from the MANIFESTS rather than hard-coding them: the menu is
    // generated from the same source, so a rename moves both sides together instead of leaving
    // this test hunting a string that no longer exists.
    const modules = await invokeOk('list_modules')
    rtModules = modules.filter((m) => /rock typing/i.test(m.category ?? ''))
    assert.ok(
      rtModules.length > 0,
      `no modules are catalogued under a Rock Typing category; categories seen: ${[
        ...new Set(modules.map((m) => m.category)),
      ].join(', ')}`,
    )
  })

  it('lists exactly the catalogued Rock Typing modules', async () => {
    const items = await openMenuContaining(rtModules[0].title)
    assert.ok(items, `no ribbon dropdown contains "${rtModules[0].title}"`)

    // Set comparison both ways: every catalogued module must be offered, and the menu must offer
    // nothing else. A menu with an extra entry is a module the catalog does not know about, which
    // is how a retired one comes back.
    const expected = rtModules.map((m) => m.title).sort()
    assert.deepEqual(
      [...items].sort(),
      expected,
      `the Rock Typing menu must list exactly its catalogued modules.\n  menu: ${items.join(' | ')}\n  catalog: ${expected.join(' | ')}`,
    )
  })

  it('opens a pane per module, and never a second one for the same module', async () => {
    const target = rtModules[0]

    // Open it once.
    assert.ok(await openMenuContaining(target.title))
    assert.ok(await clickMenuItem(target.title), `could not click "${target.title}"`)
    await browser.waitUntil(async () => (await paneCount()) > 0, {
      timeout: 30_000,
      interval: 500,
      timeoutMsg: `no module pane opened for "${target.title}"`,
    })

    const after = await paneCount()

    // Open it again — the count must not move. Step 5's singleton claim, and the reason it matters:
    // two panes for one module each carry their own scope and parameters, so editing one and
    // running the other produces a run nobody configured.
    assert.ok(await openMenuContaining(target.title))
    assert.ok(await clickMenuItem(target.title))

    // Give a duplicate a chance to appear before concluding it did not.
    await browser.pause(1000)
    assert.equal(
      await paneCount(),
      after,
      `re-clicking "${target.title}" must focus the existing pane, not open a second one`,
    )
  })

  it('builds that pane from its manifest, outputs note and all', async () => {
    const target = rtModules[0]
    const outputs = (target.args ?? [])
      .filter((a) => a.kind === 'log_out')
      .map((a) => a.name)

    const form = await browser.execute(() => {
      const panes = document.querySelectorAll('.module-pane')
      const pane = panes[panes.length - 1]
      if (!pane) return null
      return {
        hasScope: !!pane.querySelector('.well-scope'),
        hasRun: !!pane.querySelector('.form-run-btn'),
        outputsNote:
          Array.from(pane.querySelectorAll('.modal-hint'))
            .map((p) => p.textContent.trim())
            .find((t) => t.startsWith('Outputs:')) ?? null,
      }
    })
    assert.ok(form, 'no module pane to inspect')
    assert.ok(form.hasScope, 'the pane must carry a well-scope control')
    assert.ok(form.hasRun, 'the pane must carry a Run button')
    assert.ok(form.outputsNote, 'the pane must state which curves the run will write')

    // The note must name the manifest's OWN outputs. A stale note is worse than none: it tells the
    // user which curves to expect, and they will go looking for the ones it named.
    for (const name of outputs) {
      assert.ok(
        form.outputsNote.includes(name),
        `the Outputs note must name ${name}; it reads: ${form.outputsNote}`,
      )
    }
  })
})
