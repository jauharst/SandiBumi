// T-SHELL-10 — Sessions: save, list, delete; plus the quiet Ctrl+S re-save from T-SHELL-11.
//
// A session is a named snapshot of the WORKSPACE — which panels are open, how they are arranged,
// and which well was active. It is deliberately not a copy of the project (that is Save Project
// As), so a session that saves the wrong thing loses an afternoon's arrangement without losing any
// data, and says nothing at the time.
//
// The part worth testing here is the round trip through the `documents` store and the shape of
// what is written. `snapshotSession` captures `{version, layout, well}` and separately captures
// each log view's chosen Layout, because dockview's own `toJSON` does not serialise it — a
// snapshot that quietly dropped a field would restore a workspace that looks right and is not.

import assert from 'node:assert/strict'

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

const NAME = 'E2E Session'

const sessions = () => invokeOk('list_documents', { docType: 'session' })

/** The status bar's current text — where the quiet save reports itself. */
const statusText = () =>
  browser.execute(() => document.querySelector('#status-bar')?.textContent?.trim() ?? '')

/** Whether a modal is currently on screen, and its title. */
const modalTitle = () =>
  browser.execute(() => {
    const root = document.querySelector('#modal-root')
    if (!root || root.childElementCount === 0) return null
    return root.querySelector('.modal-title, h3, h2')?.textContent?.trim() ?? '(untitled modal)'
  })

describe('sessions (T-SHELL-10, T-SHELL-11 quiet save)', () => {
  before(async () => {
    // Start clean so "saved" is unambiguous — a leftover from an earlier run would make the first
    // assertion pass without anything having been written this time.
    await call('delete_document', { docType: 'session', name: NAME })
    await browser.execute(() => {
      document.querySelector('.ribbon-tab[data-tab="project"]')?.click()
    })
  })

  after(async () => {
    await call('delete_document', { docType: 'session', name: NAME })
  })

  it('saves the workspace under a name', async () => {
    const before = (await sessions()).map((d) => d.name)
    assert.ok(!before.includes(NAME), 'the test session must not exist before this test')

    await browser.execute(() => document.querySelector('#save-session-btn')?.click())
    await browser.waitUntil(async () => (await modalTitle()) !== null, {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: 'Save Session… opened no dialog',
    })

    await browser.execute((name) => {
      const root = document.querySelector('#modal-root')
      const input = root.querySelector('input.form-control')
      input.value = name
      const save = Array.from(root.querySelectorAll('button')).find(
        (b) => (b.textContent ?? '').trim() === 'Save',
      )
      save.click()
    }, NAME)

    await browser.waitUntil(
      async () => (await sessions()).some((d) => d.name === NAME),
      {
        timeout: 15_000,
        interval: 250,
        timeoutMsg: 'the session was never written to the documents store',
      },
    )

    // The dialog must close itself on a successful save. One that stays open reads as a save that
    // did not happen, and the usual next move is to press Save again.
    await browser.waitUntil(async () => (await modalTitle()) === null, {
      timeout: 10_000,
      interval: 200,
      timeoutMsg: 'the Save Session dialog stayed open after a successful save',
    })
  })

  it('writes a snapshot that carries the layout and the active well', async () => {
    const doc = (await sessions()).find((d) => d.name === NAME)
    assert.ok(doc, 'the saved session must be listed')

    const snap = JSON.parse(doc.json)
    // Asserted field by field rather than "it is valid JSON". A snapshot that lost `layout` still
    // parses, still restores without error, and simply rebuilds nothing — the workspace comes back
    // empty and the session looks like it was never saved properly.
    assert.ok(typeof snap.version === 'number', 'a session must record its snapshot version')
    assert.ok(snap.layout && typeof snap.layout === 'object', 'a session must carry the dock layout')
    assert.ok(
      Array.isArray(snap.layout.grid?.root?.data ?? null) ||
        typeof snap.layout.grid === 'object',
      'the layout must be dockview’s own serialised grid, not an empty placeholder',
    )

    // `well` may legitimately be null if nothing is selected, but the FIELD must be present — its
    // absence is how a snapshot silently stops restoring the well it was taken on.
    assert.ok('well' in snap, 'a session must record which well was active, even if that is none')
  })

  it('lists the saved session in the Open Session dialog', async () => {
    await browser.execute(() => document.querySelector('#open-session-btn')?.click())
    await browser.waitUntil(async () => (await modalTitle()) !== null, {
      timeout: 15_000,
      interval: 200,
      timeoutMsg: 'Open Session… opened no dialog',
    })

    const listed = await browser.execute(
      () => document.querySelector('#modal-root .session-list')?.textContent ?? '',
    )
    assert.ok(
      listed.includes(NAME),
      `the Open Session dialog must list the saved session; it showed: ${listed.slice(0, 200)}`,
    )

    // Close it without opening anything — restoring a workspace mid-run would tear down the panes
    // the other specs are driving.
    await browser.execute(() => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    })
    await browser.waitUntil(async () => (await modalTitle()) === null, {
      timeout: 10_000,
      interval: 200,
      timeoutMsg: 'Escape did not close the Open Session dialog',
    })
  })

  it('re-saves quietly on Ctrl+S once the session has a name', async () => {
    // The claim is QUIET: having named the session once, Ctrl+S must write it again without
    // putting a dialog in the way. A save that re-prompts every time is one people stop using,
    // and the unsaved-state dot then stops meaning anything.
    await browser.execute(() => {
      document.dispatchEvent(
        new KeyboardEvent('keydown', { key: 's', ctrlKey: true, bubbles: true, cancelable: true }),
      )
    })

    await browser.waitUntil(async () => /session .*saved/i.test(await statusText()), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: `Ctrl+S never reported a save; the status bar reads: ${await statusText()}`,
    })

    assert.equal(
      await modalTitle(),
      null,
      'Ctrl+S must re-save in place, not reopen the naming dialog',
    )
    assert.ok(
      (await statusText()).includes(NAME),
      `the status line must name the session it re-saved; it reads: ${await statusText()}`,
    )
  })

  it('deletes a session from the store', async () => {
    await invokeOk('delete_document', { docType: 'session', name: NAME })
    const after = (await sessions()).map((d) => d.name)
    assert.ok(!after.includes(NAME), 'a deleted session must be gone from the documents store')
  })
})
