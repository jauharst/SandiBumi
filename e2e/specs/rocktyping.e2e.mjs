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

/** How many panes are open FOR THIS MODULE, identified by its own declared outputs.
 *
 * Deliberately not the global `.module-pane` count. By the time this spec runs, several module
 * panes are open, and dockview attaches and detaches them as the active tab changes — so a global
 * count moves for reasons that have nothing to do with this module. It measured a 2 -> 3
 * "duplicate" that was another spec's pane re-attaching, while the app was behaving correctly
 * (verified against the running app: one tab, one pane, before and after the re-click). The
 * singleton claim is about THIS module, so the count has to be too — the same identification rule
 * wellgroupmanager.e2e.mjs settled on. */
const paneCountFor = (outputs) =>
  browser.execute(
    (outs) =>
      Array.from(document.querySelectorAll('.module-pane')).filter((p) => {
        const declared = Array.from(p.querySelectorAll('.module-outputs .module-output-label'))
          .map((l) => l.title ?? '')
          .filter((t) => t.startsWith('Declared as '))
          .map((t) => t.slice('Declared as '.length).split(';')[0])
        return outs.every((o) => declared.includes(o))
      }).length,
    outputs,
  )

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
    const targetOutputs = (target.args ?? [])
      .filter((a) => a.kind === 'log_out')
      .map((a) => a.name)

    // Open it once.
    assert.ok(await openMenuContaining(target.title))
    assert.ok(await clickMenuItem(target.title), `could not click "${target.title}"`)
    await browser.waitUntil(async () => (await paneCountFor(targetOutputs)) > 0, {
      timeout: 30_000,
      interval: 500,
      timeoutMsg: `no module pane opened for "${target.title}"`,
    })

    const after = await paneCountFor(targetOutputs)
    assert.equal(after, 1, `"${target.title}" must open exactly one pane; found ${after}`)

    // Open it again — the count must not move. Step 5's singleton claim, and the reason it matters:
    // two panes for one module each carry their own scope and parameters, so editing one and
    // running the other produces a run nobody configured.
    assert.ok(await openMenuContaining(target.title))
    assert.ok(await clickMenuItem(target.title))

    // Give a duplicate a chance to appear before concluding it did not.
    await browser.pause(1000)
    assert.equal(
      await paneCountFor(targetOutputs),
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
        // The pane's primary pill lives in its footer now (design 1d) — `.form-run-btn` is gone.
        hasRun: !!pane.querySelector('.module-footer .btn'),
        // The old "Outputs:" hint became the editable outputs section: each output-name label
        // carries a title "Declared as <ARG>" (longer for porosity outputs, `;`-separated).
        outputTitles: Array.from(pane.querySelectorAll('.module-outputs .module-output-label')).map(
          (l) => l.title ?? '',
        ),
      }
    })
    assert.ok(form, 'no module pane to inspect')
    assert.ok(form.hasScope, 'the pane must carry a well-scope control')
    assert.ok(form.hasRun, 'the pane must carry a Run button')
    assert.ok(form.outputTitles.length > 0, 'the pane must state which curves the run will write')

    // The section must declare the manifest's OWN outputs. A stale list is worse than none: it
    // tells the user which curves to expect, and they will go looking for the ones it named.
    for (const name of outputs) {
      assert.ok(
        form.outputTitles.some(
          (t) => t === `Declared as ${name}` || t.startsWith(`Declared as ${name};`),
        ),
        `the outputs section must declare ${name}; it declares: ${form.outputTitles.join(' | ')}`,
      )
    }
  })
})
