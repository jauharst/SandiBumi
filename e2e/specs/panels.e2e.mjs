// Panes that open and say the right thing: the SQL console (T-REP-17), the performance monitor
// (T-AUX-01) and contextual Help (T-AUX-02).
//
// Grouped because they share one shape — open it from its real ribbon button, then assert what it
// renders — and because each is small on its own. The claims are deliberately about CONTENT rather
// than about the pane existing: a panel that opens empty, or with a placeholder that looks like
// data, is the failure mode worth catching in all three.

import assert from 'node:assert/strict'

/**
 * Call a backend command and report failure as a VALUE rather than letting it throw.
 *
 * Caught twice on purpose. The page-side `try` is the same one the other specs use — but this is
 * the first spec that calls a command EXPECTING it to be refused, and the rejection turned out to
 * escape anyway: @wdio/tauri-service re-throws page-side errors through its own `__wdio_error__`
 * channel, wdio retries the command three times and then fails the test with the backend's message
 * as a WebDriverError. Which is a confusing way to be told that the refusal you were testing for
 * happened exactly as expected.
 *
 * So the `browser.execute` call is wrapped on the NODE side as well. Testing a refusal needs the
 * refusal to be data, not an exception.
 */
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

/** Click a ribbon button on a given tab, in one round trip. */
const openFrom = (tab, id) =>
  browser.execute(
    (t, i) => {
      document.querySelector(`.ribbon-tab[data-tab="${t}"]`)?.click()
      const btn = document.querySelector(`#${i}`)
      if (!btn) return false
      btn.click()
      return true
    },
    tab,
    id,
  )

describe('panels that open and report (T-REP-17, T-AUX-01, T-AUX-02)', () => {
  it('opens the SQL console with a runnable starter query', async () => {
    assert.ok(await openFrom('data', 'sql-query-btn'), 'no #sql-query-btn on the Data tab')

    await browser.waitUntil(
      async () => await browser.execute(() => !!document.querySelector('.sql-input')),
      { timeout: 30_000, interval: 500, timeoutMsg: 'the SQL Query pane never opened' },
    )

    const starter = await browser.execute(
      () => document.querySelector('.sql-input')?.value?.trim() ?? '',
    )
    // The starter query is not decoration: it is the only place a user is shown what the project's
    // tables are called before writing anything. An empty box would mean guessing at schema names.
    assert.ok(starter.length > 0, 'the SQL pane must open with a starter query, not an empty box')
    assert.match(
      starter,
      /select/i,
      `the starter must be a runnable SELECT; it reads: ${starter.slice(0, 120)}`,
    )

    // Run it and require ROWS. "The pane rendered a grid" is satisfied by an error message too, so
    // the assertion is on what the query returned through the same command the pane uses.
    await browser.execute(() => document.querySelector('.sql-run')?.click())
    await browser.waitUntil(
      async () =>
        await browser.execute(
          () => !/^running/i.test(document.querySelector('.sql-info')?.textContent?.trim() ?? ''),
        ),
      { timeout: 30_000, interval: 250, timeoutMsg: 'the starter query never finished' },
    )

    const grid = await browser.execute(() => ({
      rows: document.querySelectorAll('.sql-grid table tbody tr').length,
      info: document.querySelector('.sql-info')?.textContent?.trim() ?? '',
      text: document.querySelector('.sql-grid')?.textContent?.trim()?.slice(0, 160) ?? '',
    }))
    assert.ok(
      grid.rows > 0,
      `the starter query must return rows on a project with wells; the pane shows: ${grid.text}`,
    )
  })

  it('refuses a write, and runs a SELECT that carries comments', async () => {
    // The read-only rule itself is pinned in Rust
    // (`readonly_query_refuses_every_write_shape_including_a_cte_prefix`). What is checked here is
    // that the refusal really reaches this path, and — the reason this test exists — how the guard
    // decides.
    const write = await call('run_query', {
      sql: 'DELETE FROM wells WHERE well_id IS NOT NULL',
      limit: 10,
    })
    assert.ok(!write.ok, 'a DELETE through the query command must be refused, not executed')
    assert.ok((await call('list_wells', { scope: { kind: 'all' } })).ok, 'the project must still be readable after the refusal')

    // Finding 23, BOTH halves fixed in `run_readonly_query` and pinned here from the fixed side:
    // the keyword guard skips leading `--` lines (the token it inspects is the one DuckDB will
    // execute), and the wrapper's closing suffix sits on its own line so a trailing comment
    // cannot swallow it. An ordinary SQL comment must never break a valid SELECT.
    const leading = await call('run_query', {
      sql: '-- a perfectly ordinary comment\nSELECT COUNT(*) AS n FROM wells',
      limit: 10,
    })
    assert.ok(leading.ok, `a SELECT behind a leading comment must run; it was refused: ${leading.error}`)

    const trailing = await call('run_query', {
      sql: 'SELECT COUNT(*) AS n FROM wells -- how many wells',
      limit: 10,
    })
    assert.ok(trailing.ok, `a SELECT with a trailing comment must run; it was refused: ${trailing.error}`)
  })

  it('opens the performance monitor with live gauges', async () => {
    assert.ok(await openFrom('project', 'health-btn'), 'no #health-btn on the Project tab')

    await browser.waitUntil(
      async () =>
        (await browser.execute(() => document.querySelectorAll('.health-row').length)) > 0,
      { timeout: 30_000, interval: 500, timeoutMsg: 'the Performance pane never rendered its rows' },
    )

    const gauges = await browser.execute(() =>
      Array.from(document.querySelectorAll('.health-row')).map((r) => ({
        label: r.querySelector('.health-label')?.textContent?.trim() ?? '',
        value: r.querySelector('.health-val')?.textContent?.trim() ?? '',
      })),
    )
    assert.ok(gauges.length > 0, 'the monitor must show at least one gauge')
    assert.ok(
      gauges.every((g) => g.label.length > 0),
      `every gauge must be labelled; got ${JSON.stringify(gauges)}`,
    )
    // A gauge showing an empty string is worse than no gauge: it reads as "measured, and it is
    // nothing" rather than "not measured".
    assert.ok(
      gauges.every((g) => g.value.length > 0),
      `every gauge must show a value, not a blank; got ${JSON.stringify(gauges)}`,
    )
  })

  it('opens contextual help for the active panel, naming no vendor', async () => {
    // Help is per PANEL, so something must be active first — with nothing focused the button
    // correctly refuses with a status line instead, which is a different claim.
    assert.ok(await openFrom('project', 'help-btn'), 'no #help-btn on the Project tab')

    const shown = await browser.waitUntil(
      async () =>
        await browser.execute(() => {
          const root = document.querySelector('#modal-root')
          return root && root.childElementCount > 0 ? root.textContent.trim() : false
        }),
      {
        timeout: 15_000,
        interval: 250,
        timeoutMsg:
          'Help opened no dialog. It is per-panel: with no active panel it reports to the status ' +
          'bar instead, so this needs a panel focused first.',
      },
    )

    assert.ok(shown.length > 20, `the help modal must carry real text; it read: ${shown}`)

    // The provenance rule, checked where a user actually reads it. The app's own documentation
    // must not carry a vendor's name — see docs/IP_PROVENANCE.md. Attribution belongs in comments
    // and in the register beside the asset it describes, not in shipped help text.
    for (const vendor of ['Schlumberger', 'Halliburton', 'Techlog', 'Geolog', 'Interactive Petrophysics']) {
      assert.ok(
        !shown.includes(vendor),
        `the help text must not name ${vendor}; it read: ${shown.slice(0, 200)}`,
      )
    }

    await browser.execute(() => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    })
  })
})
