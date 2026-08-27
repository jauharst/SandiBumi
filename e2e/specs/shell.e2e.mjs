// The shell: does the application chrome actually come up, and is it wired the way the ribbon
// claims? Covers manual-plan items T-SHELL-01, T-SHELL-02, T-SHELL-03 and T-ADV-01, plus the
// ribbon half of T-RT-16.
//
// Unlike pipeline.e2e.mjs these are DOM tests rather than `invoke` tests, and that carries a
// standing caveat worth stating once: the harness drives a BUILT binary, which embeds whatever
// frontend was compiled into it. A DOM assertion here is a statement about that build, so a
// binary older than the tree will fail on markup the tree has already fixed — rebuild before
// reading a failure as a regression.
//
// Every assertion is deliberately structural (how many tabs, is exactly one panel showing, did
// the label change language) rather than a pixel or a specific button caption, because captions
// are the part of a ribbon that legitimately changes most often.

import assert from 'node:assert/strict'

/** The ribbon's tabs, in the order index.html declares them. */
const TABS = ['project', 'data', 'petro', 'advance', 'plot', 'view']

/**
 * Click a ribbon tab IN THE PAGE, and read back what happened, in one round trip.
 *
 * This is a performance contract, not a style preference. @wdio/tauri-service runs a window-focus
 * probe in `beforeCommand` for exactly `getTitle`, `findElement(s)`, `$`, `$$` and `elementClick`;
 * that probe asks for a Tauri plugin this app does not register, and each failure costs about
 * SEVEN AND A HALF SECONDS. `execute` is not on that list and is effectively free. A six-tab walk
 * written the natural way — `await $(sel)` then `await el.click()` — is twelve of those commands
 * and blew the 180 s mocha timeout; the same walk done through `execute` finishes in under a
 * second.
 *
 * The trade is honest and small: an in-page `el.click()` is not a trusted user gesture, so it
 * would not satisfy anything gated on user activation (fullscreen, clipboard, autoplay). Ribbon
 * tabs are plain click listeners, so for this test the two are equivalent — but do not reach for
 * this helper when the thing under test is the gesture itself.
 */
function clickTab(tab) {
  return browser.execute((t) => {
    const btn = document.querySelector(`.ribbon-tab[data-tab="${t}"]`)
    if (!btn) return { clicked: false }
    btn.click()
    return { clicked: true }
  }, tab)
}

/** Click a tab and assert it actually opened. */
async function openTab(tab) {
  const r = await clickTab(tab)
  assert.ok(r.clicked, `no ribbon tab with data-tab="${tab}"`)
}

/** The visible text of the ribbon tab strip, trimmed, in document order. */
const tabLabels = () =>
  browser.execute(() =>
    Array.from(document.querySelectorAll('.ribbon-tab')).map((b) => (b.textContent ?? '').trim()),
  )

describe('the application shell', () => {
  after(async () => {
    // Specs share one app and one session, so a language left switched would be inherited by
    // whatever runs next — and every later DOM assertion would then be reading Sundanese. Put it
    // back regardless of how this file ended.
    await browser.execute(() => {
      const sel = document.querySelector('#language-select')
      if (sel && sel.value !== 'en') {
        sel.value = 'en'
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }
    })
  })

  // T-SHELL-01 — the shell rendered at all.
  it('renders the ribbon, the status bar and the workspace', async () => {
    const shell = await browser.execute(() => ({
      tabs: Array.from(document.querySelectorAll('.ribbon-tab')).map((b) => b.dataset.tab),
      panels: Array.from(document.querySelectorAll('.ribbon-panel')).map((s) => s.dataset.panel),
      status: document.querySelector('#status-bar')?.textContent?.trim() ?? null,
      dockChildren: document.querySelector('#dock-root')?.childElementCount ?? 0,
    }))

    assert.deepEqual(shell.tabs, TABS, 'the ribbon must carry exactly the declared tabs, in order')
    assert.deepEqual(
      shell.panels,
      TABS,
      'every tab must have a panel and no panel may be orphaned — a tab whose panel is missing ' +
        'is a tab that opens onto nothing',
    )
    assert.ok(shell.status !== null, 'the status bar must exist')
    assert.ok(shell.dockChildren > 0, 'the dockview workspace must have been created')
  })

  // T-SHELL-02 — walking the tabs.
  it('shows exactly one ribbon panel at a time, and each one has captioned groups', async () => {
    // The whole walk is one round trip, for the reason documented on `clickTab`. Tab switching is
    // a synchronous class/attribute toggle, so there is nothing to wait for between the click and
    // the read — and doing it this way means the six tabs are measured under identical conditions
    // rather than seconds apart.
    const walk = await browser.execute((tabs) => {
      const out = []
      for (const t of tabs) {
        const btn = document.querySelector(`.ribbon-tab[data-tab="${t}"]`)
        if (!btn) {
          out.push({ tab: t, missing: true })
          continue
        }
        btn.click()
        // checkVisibility() is the RENDERED answer, not the `hidden` attribute. That distinction is
        // the whole point here: CLAUDE.md records that a CSS `display` rule overrode `hidden` on
        // these very panels twice, which leaves the attribute correct and the panel on screen.
        // Reading the attribute would have passed on both of those bugs.
        const panels = Array.from(document.querySelectorAll('.ribbon-panel'))
        const mine = document.querySelector(`.ribbon-panel[data-panel="${t}"]`)
        out.push({
          tab: t,
          missing: false,
          visible: panels.filter((p) => p.checkVisibility()).map((p) => p.dataset.panel),
          active: Array.from(document.querySelectorAll('.ribbon-tab.active')).map(
            (b) => b.dataset.tab,
          ),
          captions: Array.from(mine?.querySelectorAll('.ribbon-group-caption') ?? []).map((c) =>
            (c.textContent ?? '').trim(),
          ),
        })
      }
      return out
    }, TABS)

    for (const state of walk) {
      assert.ok(!state.missing, `no ribbon tab with data-tab="${state.tab}"`)
      assert.deepEqual(
        state.visible,
        [state.tab],
        `clicking ${state.tab} must leave exactly that panel on screen; visible: ${state.visible.join(', ')}`,
      )
      assert.deepEqual(state.active, [state.tab], `exactly one tab may carry .active (${state.tab})`)
      assert.ok(
        state.captions.length > 0,
        `the ${state.tab} panel has no captioned groups — an empty ribbon tab is a dead tab`,
      )
      assert.ok(
        state.captions.every((c) => c.length > 0),
        `the ${state.tab} panel has a group with a blank caption: ${JSON.stringify(state.captions)}`,
      )
    }
  })

  // T-ADV-01 — the flagship methods are promoted out of the auto-generated dropdowns.
  it('fills the Advance tab with the flagship methods and the calibration tools', async () => {
    await openTab('advance')

    // These buttons are built from module manifests fetched over IPC, so the container is empty
    // for a moment after the tab first renders. Waiting for the count is the difference between
    // testing the ribbon and testing how fast this machine answered `list_modules`.
    await browser.waitUntil(
      async () =>
        (await browser.execute(
          () => document.querySelectorAll('#advance-modules .ribbon-btn').length,
        )) >= 5,
      {
        timeout: 30_000,
        timeoutMsg: 'the Advance tab never filled with its promoted module buttons',
      },
    )

    const advance = await browser.execute(() => ({
      // The five promoted module buttons are generated from manifests into #advance-modules.
      generated: Array.from(
        document.querySelectorAll('#advance-modules .ribbon-btn .ribbon-label'),
      ).map((s) => (s.textContent ?? '').trim()),
      // The hand-written buttons in the same panel.
      // sandimin-btn since the 2026-08-20 rename: only the retired module id and workspace
      // component id are frozen for saved chains/sessions; the ribbon button id moved with it.
      byId: ['sandimin-btn', 'rtc-fit-btn', 'sfactor-fit-btn', 'ml-btn'].filter((id) =>
        document.querySelector(`#${id}`),
      ),
    }))

    // Asserted as a SET rather than in order: which flagship sits leftmost is a layout choice,
    // but one of them going missing means a manifest stopped being found.
    assert.deepEqual(
      [...advance.generated].sort(),
      ['IMTS', 'RtC', 'SSC', 'SSPW', 'Thin Beds'],
      `the Advance tab must carry all five promoted methods; got: ${advance.generated.join(', ')}`,
    )
    assert.deepEqual(
      advance.byId,
      ['sandimin-btn', 'rtc-fit-btn', 'sfactor-fit-btn', 'ml-btn'],
      'SandiMin, both calibration tools and ML Models must all be on the Advance tab',
    )
  })

  // T-RT-16 (ribbon half) — the retired fixed-component solver has no button anywhere.
  it('gives the legacy fixed multimin no button in any ribbon tab', async () => {
    // The retirement is upheld by TWO independent mechanisms, and breaking either one puts the
    // button back in a different place: `multimin` is listed in ADVANCED_MODULE_IDS, which filters
    // it OUT of the auto-generated Petrophysics dropdowns, and its META caption is "(hidden)",
    // which is outside groupOrder so renderAdvancedModules never emits it. Sweeping the whole
    // ribbon catches both, where checking one tab would catch one.
    //
    // "SandiMin" is the CURRENT generalised solver (multimin2.rs) and must still be there — the
    // preceding test asserts that — so the label is matched exactly rather than by substring.
    const found = await browser.execute(() => {
      const labels = Array.from(document.querySelectorAll('#ribbon .ribbon-label'))
      return labels
        .map((s) => (s.textContent ?? '').trim())
        .filter((t) => t === 'Mineral Inv' || t === 'Multimin' || t === 'Multimin…')
    })
    assert.deepEqual(
      found,
      [],
      `the retired legacy multimin must not appear as a ribbon button; found: ${found.join(', ')}`,
    )
  })

  // T-SHELL-03 — the language round trip.
  it('switches UI language EN to ID to SU to JV and back', async () => {
    await openTab('project')

    // "Project" is the discriminator on purpose. Its four forms differ only by diacritics
    // (Project / Proyek / Proyék / Proyèk), so asserting each one proves the RIGHT dictionary was
    // selected. "Petrophysics" would have been useless here — it is "Petrofisika" in all three
    // translations, so a test built on it passes while serving Sundanese to an Indonesian user.
    const expected = { en: 'Project', id: 'Proyek', su: 'Proyék', jv: 'Proyèk' }

    for (const locale of ['id', 'su', 'jv', 'en']) {
      await browser.execute((l) => {
        const sel = document.querySelector('#language-select')
        sel.value = l
        sel.dispatchEvent(new Event('change', { bubbles: true }))
      }, locale)

      // i18n substitutes text through a MutationObserver, so the DOM settles a tick after the
      // change event rather than synchronously with it.
      await browser.waitUntil(
        async () => (await tabLabels()).includes(expected[locale]),
        {
          timeout: 10_000,
          timeoutMsg:
            `the ribbon never showed "${expected[locale]}" after switching to ${locale}; ` +
            `tabs read: ${(await tabLabels()).join(', ')}`,
        },
      )

      const labels = await tabLabels()
      assert.ok(
        labels.includes(expected[locale]),
        `${locale}: expected a tab reading "${expected[locale]}", got ${labels.join(', ')}`,
      )
      // The technical terms stay English BY DESIGN (CLAUDE.md: mnemonics, Monte Carlo, Pickett,
      // Thin Beds and the like are Jauhar's explicit request). "Advance" carries no dictionary
      // entry, so it is the check that translation is targeted rather than blanket.
      assert.ok(
        labels.includes('Advance'),
        `${locale}: untranslated terms must stay English; tabs read ${labels.join(', ')}`,
      )
    }

    const back = await tabLabels()
    assert.ok(back.includes('Project'), 'switching back to English must restore the English tabs')
  })
})
