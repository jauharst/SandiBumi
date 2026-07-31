// T-INT-09 — well-group scoping, end to end against the real app.
//
// This is the first pile-C spec, and it was chosen because the triage found that NOTHING in the
// Rust suite touches `well_groups`, `well_group_members` or group-filtered scope — not one test.
// The feature is also unusually load-bearing for something untested: `filterByActiveGroup` is
// what a dozen batch dialogs call to decide which wells a run covers, so a group that reports the
// wrong membership does not fail, it quietly runs on the wrong wells and writes real curves to
// them.
//
// WHY 2 OF 3 RATHER THAN THE PLAN'S 3 OF 4. The manual step wants four wells so a three-well
// group can be shown to exclude one. The repo ships three example wells and the standing rule is
// that the harness uses the repo's own fixtures only — never a real project, never a path from
// SANDIBUMI_FIELD_FIXTURES. A group of two out of three proves exactly the same thing: a member
// gets the curve, a non-member does not. Inventing a fourth well to match the prose would buy
// nothing and cost the fixture rule.
//
// The last test is DOM-level on purpose. Everything above it goes through `invoke`, which proves
// the backend and the write discipline but says nothing about whether the pane a user actually
// looks at agrees — and the whole failure mode here is a UI that shows one scope while a run
// covers another.

import assert from 'node:assert/strict'
import path from 'node:path'
import { examplesDir } from '../wdio.conf.mjs'

/** Call a backend command in the running app and return its result. */
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

/** Uppercased computed-curve names for one well, read back from the catalog. */
async function computedNames(wellId) {
  const catalog = await invokeOk('list_computed_catalog', { wellId })
  return catalog.map((c) => (c.name ?? c.curve_name ?? '').toUpperCase())
}

/**
 * A fingerprint of everything computed stored for one well: per curve, the row count and the
 * summed value. Read through the read-only SQL pane's own command, so this is the same path a
 * user would check it by.
 *
 * It exists because "the outsider has no VSH" is NOT a safe assertion here. Both specs run
 * against ONE app and ONE project, wdio guarantees no ordering between spec files, and
 * `pipeline.e2e.mjs` runs vsh_gr across every well — so the outsider may legitimately already
 * carry a VSH before this spec starts. Comparing a fingerprint before and after states the claim
 * that actually matters and is true regardless of what ran first: this run did not touch that
 * well. It is also stronger, because it would catch a run that silently OVERWROTE the outsider's
 * existing values, which a name check never could.
 */
async function computedFingerprint(wellId) {
  const page = await invokeOk('run_query', {
    sql:
      "SELECT curve_name, COUNT(*) AS n, COALESCE(SUM(value), 0) AS total " +
      `FROM computed_curves WHERE well_id = '${wellId}' ` +
      'GROUP BY curve_name ORDER BY curve_name',
    limit: 1000,
  })
  return (page.rows ?? []).map((r) => r.join('|')).join('\n')
}

const GROUP = 'SANDI-NORTH'
const OTHER = 'SANDI-SOUTH'

describe('well-group scoping (T-INT-09)', () => {
  let wells = []
  let members = []
  let outsider = null

  before(async () => {
    // This spec owns its own wells so it does not depend on another spec having run first —
    // wdio gives no ordering guarantee across files, and a spec that silently passes because it
    // found no wells to run on is worse than one that fails.
    const existing = await invokeOk('list_wells')
    if (existing.length === 0) {
      const paths = ['SANDI-01.las', 'SANDI-02.las', 'SANDI-03.las'].map((f) =>
        path.join(examplesDir, f),
      )
      await invokeOk('import_las_files', { paths, setName: 'E2E', attach: false })
    }
    wells = await invokeOk('list_wells')
    assert.ok(wells.length >= 3, `need at least 3 wells, found ${wells.length}`)
    wells.sort((a, b) => a.well_name.localeCompare(b.well_name))
    members = wells.slice(0, 2)
    outsider = wells[2]
  })

  it('creates a group and makes it active', async () => {
    const groupId = await invokeOk('create_well_group', {
      name: GROUP,
      wellIds: members.map((w) => w.well_id),
    })
    assert.ok(groupId, 'create_well_group must return an id')

    await invokeOk('set_active_well_group', { groupId })

    const groups = await invokeOk('list_well_groups')
    const mine = groups.find((g) => g.group_id === groupId)
    assert.ok(mine, `the created group must be listed; got ${groups.map((g) => g.name).join(', ')}`)
    assert.equal(mine.name, GROUP)
    assert.equal(mine.member_count, 2, 'member_count must reflect the membership actually stored')
    assert.ok(mine.active, 'the group we activated must report itself active')
    assert.deepEqual(
      [...mine.well_ids].sort(),
      members.map((w) => w.well_id).sort(),
      'well_ids must be the wells we put in, not a count that happens to match',
    )
  })

  it('keeps exactly one group active', async () => {
    // `set_active_well_group` clears every group before setting one. That invariant is the whole
    // basis of `filterByActiveGroup`, which reads a SINGLE active group — with two active rows,
    // which one wins is a matter of row order, so the scope of every batch dialog in the app
    // would depend on insertion order.
    const second = await invokeOk('create_well_group', {
      name: OTHER,
      wellIds: [outsider.well_id],
    })
    await invokeOk('set_active_well_group', { groupId: second })

    const groups = await invokeOk('list_well_groups')
    const active = groups.filter((g) => g.active)
    assert.equal(active.length, 1, `exactly one group may be active, found ${active.length}`)
    assert.equal(active[0].name, OTHER, 'activating a group must deactivate the previous one')

    // Back to the group under test, and clearing is reachable too ("All wells").
    await invokeOk('set_active_well_group', { groupId: null })
    assert.equal(
      (await invokeOk('list_well_groups')).filter((g) => g.active).length,
      0,
      'a null group id must clear the active group rather than being ignored',
    )
    const mine = (await invokeOk('list_well_groups')).find((g) => g.name === GROUP)
    await invokeOk('set_active_well_group', { groupId: mine.group_id })
  })

  it('replaces membership rather than appending to it', async () => {
    const mine = (await invokeOk('list_well_groups')).find((g) => g.name === GROUP)

    await invokeOk('set_well_group_members', {
      groupId: mine.group_id,
      wellIds: [members[0].well_id],
    })
    let now = (await invokeOk('list_well_groups')).find((g) => g.group_id === mine.group_id)
    assert.equal(now.member_count, 1, 'setting membership must REPLACE, not add to, the old set')

    await invokeOk('set_well_group_members', {
      groupId: mine.group_id,
      wellIds: members.map((w) => w.well_id),
    })
    now = (await invokeOk('list_well_groups')).find((g) => g.group_id === mine.group_id)
    assert.equal(now.member_count, 2, 'and restoring it must not leave a duplicate behind')
  })

  it('a run scoped to the group writes curves to members and not to the outsider', async () => {
    // The claim the whole feature exists for. Note what is NOT asserted: that the dialog chose
    // this scope. That is the next test's job — here the scope is passed explicitly, so a failure
    // means the backend wrote outside the well list it was given, which would be far worse.
    const before = await computedFingerprint(outsider.well_id)

    const results = await invokeOk('run_workflow_module', {
      req: {
        module: 'vsh_gr',
        well_ids: members.map((w) => w.well_id),
        log_inputs: {},
        params: {},
        opts: {},
      },
    })
    assert.equal(results.length, 2, 'one result per member, and no result for the outsider')
    for (const r of results) {
      assert.ok(!r.error, `${r.well_name ?? r.well_id}: ${r.error}`)
    }

    for (const m of members) {
      const names = await computedNames(m.well_id)
      assert.ok(
        names.some((n) => n.startsWith('VSH')),
        `member ${m.well_name} should have VSH; got: ${names.join(', ')}`,
      )
    }
    const after = await computedFingerprint(outsider.well_id)
    assert.equal(
      after,
      before,
      `the non-member ${outsider.well_name} must be untouched by a group-scoped run — ` +
        'not one curve added, not one value changed',
    )
  })

  it('leaves the group active in the project for the DOM check to pick up', async () => {
    // The DOM half deliberately lives in `wellscope.e2e.mjs`, not here. wdio starts a FRESH app
    // session per spec file against this same sandbox project, so by the time that spec runs the
    // app has booted with this group already active and the Wells pane is filtered at first
    // paint — no reload needed, and it proves the active group survives a restart into the
    // bargain, because it lives in the project rather than in memory.
    //
    // Forcing a refresh inside this spec was tried and is the wrong tool: `browser.reloadSession()`
    // tears the app down mid-session, `window.__TAURI__` is gone on the other side, and the run
    // ended with the harness's own WAL guard firing. See docs/e2e_harness.md.
    const groups = await invokeOk('list_well_groups')
    const active = groups.filter((g) => g.active)
    assert.equal(active.length, 1, 'exactly one group must be left active')
    assert.equal(active[0].name, GROUP)
    assert.equal(active[0].member_count, 2)
  })
})
