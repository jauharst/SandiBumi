import assert from 'node:assert/strict';
import test from 'node:test';

import {
  classifyProductionChanges,
  commandExitedSuccessfully,
  deriveEvidenceFacts,
  derivePilotProgram,
  deriveRowTruth,
  evaluateGate1Exit,
  hashPilotDispositions,
  hashRequirementIds,
  renderGate1Report,
  validateFullGateReceipt,
  validatePilotScopeManifest,
} from './gate1-final-audit.mjs';

function completeFacts() {
  return {
    inventory: {
      tracker_valid: true,
      total: 931,
      unique: 931,
    },
    row_truth: {
      adjudicated: 931,
      unadjudicated: 0,
      geo_boundary_state: 'NOT-NEEDED',
      geo_deferred_rows: 0,
      geo_rows_conform: true,
    },
    evidence: {
      exact_test_map_valid: true,
      claimed_test_rows: 199,
      mapped_test_rows: 199,
      unresolved_citations: 0,
      unresolved_branch_commits: 0,
      adjudicated_rows_without_manual_evidence: 0,
      adjudicated_rows_with_unverified_commits: 0,
      missing_domain_receipts: [],
      claims_inventory_complete: true,
      manual_matrix_current: true,
    },
    discrepancies: {
      prd_audit_current: true,
      all_findings_listed: true,
    },
    baseline_gate: {
      accepted_baseline_recorded: true,
      accepted_baseline_is_ancestor: true,
      full_gate_receipt_present: true,
      full_gate_fresh: true,
      full_gate_failed: 0,
      tested_commit_is_ancestor: true,
      post_test_changes_are_evidence_only: true,
    },
    pilot_program: {
      manifest_state: 'APPROVED',
      requirements_covered: 931,
      undecided_requirements: 0,
      blocker_program_approved: true,
      blockers_without_next_action: 0,
      blockers_without_dependency: 0,
      blockers_without_owner_decision: 0,
    },
    production_boundary: {
      changed_paths: [],
    },
  };
}

function criterion(report, id) {
  return report.criteria.find((entry) => entry.id === id);
}

test('all_seven_gate_one_exit_criteria_must_pass_before_the_gate_can_close', () => {
  // CORRECTNESS — takeover design §6 defines seven conjunctive Gate 1 exit criteria.
  const report = evaluateGate1Exit(completeFacts());

  assert.equal(report.state, 'PASS');
  assert.equal(report.criteria.length, 7);
  assert.deepEqual([...new Set(report.criteria.map((entry) => entry.state))], ['PASS']);
});

test('a_deferred_row_is_not_silently_counted_as_live_adjudicated', () => {
  // CORRECTNESS — the original Gate 1 criterion requires every row's reverified as-built state.
  const facts = completeFacts();
  facts.row_truth = {
    adjudicated: 879,
    unadjudicated: 52,
    geo_boundary_state: 'PROPOSED',
    geo_deferred_rows: 52,
    geo_rows_conform: true,
  };

  const report = evaluateGate1Exit(facts);

  assert.equal(report.state, 'OPEN');
  assert.equal(criterion(report, 'G1-C2').state, 'OPEN');
  assert.match(criterion(report, 'G1-C2').detail, /52.*unadjudicated.*not approved/iu);
});

test('an_approved_geo_boundary_can_cover_only_the_exact_conforming_deferred_set', () => {
  // CORRECTNESS — DEC-011 permits only the 52 accounted SB-GEO rows to remain visibly deferred.
  const facts = completeFacts();
  facts.row_truth = {
    adjudicated: 879,
    unadjudicated: 52,
    geo_boundary_state: 'APPROVED',
    geo_deferred_rows: 52,
    geo_rows_conform: true,
  };

  const report = evaluateGate1Exit(facts);
  assert.equal(criterion(report, 'G1-C2').state, 'PASS');

  facts.row_truth.geo_deferred_rows = 51;
  const mismatched = evaluateGate1Exit(facts);
  assert.equal(criterion(mismatched, 'G1-C2').state, 'OPEN');
});

test('an_unresolved_test_citation_branch_or_manual_claim_keeps_evidence_open', () => {
  // CORRECTNESS — takeover design §6 criterion 3 requires every claimed evidence item to resolve.
  const facts = completeFacts();
  facts.evidence.unresolved_citations = 1;
  facts.evidence.unresolved_branch_commits = 1;
  facts.evidence.adjudicated_rows_without_manual_evidence = 1;

  const report = evaluateGate1Exit(facts);

  assert.equal(criterion(report, 'G1-C3').state, 'OPEN');
  assert.match(criterion(report, 'G1-C3').detail, /citation.*branch.*manual/iu);
});

test('an_unlisted_prd_discrepancy_keeps_the_structural_audit_open', () => {
  // CORRECTNESS — Gate 1 lists discrepancies rather than normalizing them silently.
  const facts = completeFacts();
  facts.discrepancies.all_findings_listed = false;

  const report = evaluateGate1Exit(facts);

  assert.equal(criterion(report, 'G1-C4').state, 'OPEN');
});

test('a_stale_or_failed_full_gate_cannot_prove_the_current_baseline', () => {
  // CORRECTNESS — Gate 1 requires a current full gate tied to the recorded baseline lineage.
  const facts = completeFacts();
  facts.baseline_gate.full_gate_fresh = false;
  facts.baseline_gate.full_gate_failed = 1;

  const report = evaluateGate1Exit(facts);

  assert.equal(criterion(report, 'G1-C5').state, 'OPEN');
  assert.match(criterion(report, 'G1-C5').detail, /stale.*1 failed/iu);
});

test('an_unapproved_or_uncovered_pilot_manifest_keeps_the_blocker_program_open', () => {
  // CORRECTNESS — only Jauhar can approve a complete executable pilot-blocker program.
  const facts = completeFacts();
  facts.pilot_program.manifest_state = 'PROPOSED';
  facts.pilot_program.requirements_covered = 879;
  facts.pilot_program.undecided_requirements = 52;

  const report = evaluateGate1Exit(facts);

  assert.equal(criterion(report, 'G1-C6').state, 'OPEN');
  assert.match(criterion(report, 'G1-C6').detail, /PROPOSED.*879.*52/iu);
});

test('a_production_path_changed_during_reconciliation_keeps_gate_one_open', () => {
  // CORRECTNESS — takeover design §6 forbids production behavior changes in Gate 1 reconciliation.
  const facts = completeFacts();
  facts.production_boundary.changed_paths = ['src-tauri/src/modules.rs'];

  const report = evaluateGate1Exit(facts);

  assert.equal(criterion(report, 'G1-C7').state, 'OPEN');
  assert.match(criterion(report, 'G1-C7').detail, /src-tauri\/src\/modules\.rs/u);
});

test('an_empty_git_response_is_success_only_when_its_exit_status_is_zero', () => {
  // CORRECTNESS — Git predicates commonly emit no stdout; process status is the authority.
  assert.equal(commandExitedSuccessfully({ status: 0, stdout: '' }), true);
  assert.equal(commandExitedSuccessfully({ status: 1, stdout: '' }), false);
  assert.equal(commandExitedSuccessfully({ status: null, stdout: 'looks fine' }), false);
});

function conformingGeoRows() {
  return Array.from({ length: 52 }, (_, index) => ({
    requirement_id: `SB-GEO-${String(index + 1).padStart(3, '0')}`,
    as_built_status: 'UNADJUDICATED',
    release_disposition: 'DEFERRED',
    risk_class: 'LATER',
    test_class: 'MISSING-OR-UNCLASSIFIED',
    expected_value_source: '',
    manual_evidence: '',
    dependencies: 'DEC-011 defers SB-GEO to the next product version.',
    commit_state: 'UNVERIFIED',
    blocking_decision: 'DEC-011: hold geomechanics/PPFG for the next product version.',
    next_action: 'NEXT-VERSION-LIVE-ADJUDICATION',
    last_reverified: '',
  }));
}

test('an_approved_geo_boundary_is_invalidated_when_the_exact_deferred_set_drifts', () => {
  // CORRECTNESS — approval covers the hashed 52-row DEC-011 set, not any future unadjudicated row.
  const geoRows = conformingGeoRows();
  const rows = [
    ...Array.from({ length: 879 }, (_, index) => ({
      requirement_id: `SB-TST-${String(index + 1).padStart(3, '0')}`,
      as_built_status: 'PRESENT-OK',
    })),
    ...geoRows,
  ];
  const policy = {
    state: 'APPROVED',
    approved_by: 'Jauhar',
    approved_on: '2026-08-12',
    requirement_ids_sha256: hashRequirementIds(geoRows.map((row) => row.requirement_id)),
  };

  const exact = deriveRowTruth(rows, policy);
  assert.equal(exact.geo_boundary_state, 'APPROVED');
  assert.equal(exact.geo_rows_conform, true);

  rows.at(-1).release_disposition = 'PILOT-BLOCKER';
  const drifted = deriveRowTruth(rows, policy);
  assert.equal(drifted.geo_boundary_state, 'INVALID');
  assert.equal(drifted.geo_rows_conform, false);
});

test('pilot_approval_is_invalidated_when_any_requirement_disposition_drifts', () => {
  // CORRECTNESS — Jauhar approves one exact release-disposition ledger, not a mutable label.
  const rows = [
    {
      requirement_id: 'SB-TST-001',
      release_disposition: 'PILOT-BLOCKER',
      next_action: 'implement the bounded contract',
      dependencies: 'cited source',
      blocking_decision: 'none',
    },
    {
      requirement_id: 'SB-TST-002',
      release_disposition: 'DEFERRED',
      next_action: 'later',
      dependencies: 'pilot manifest',
      blocking_decision: 'owner approved deferral',
    },
  ];
  const policy = {
    state: 'APPROVED',
    approved_by: 'Jauhar',
    approved_on: '2026-08-12',
    disposition_sha256: hashPilotDispositions(rows),
    requirement_ids_sha256: hashRequirementIds(['SB-TST-001']),
    approved_blocker_count: 1,
  };
  const manifest = {
    valid: true,
    state: 'APPROVED',
    requirement_ids: ['SB-TST-001'],
    approval: {
      state: 'APPROVED',
      approved_by: 'Jauhar',
      approved_on: '2026-08-12',
    },
  };

  const exact = derivePilotProgram(rows, policy, manifest);
  assert.equal(exact.manifest_state, 'APPROVED');
  assert.equal(exact.blocker_program_approved, true);

  rows[1].release_disposition = 'PILOT-BLOCKER';
  const drifted = derivePilotProgram(rows, policy, manifest);
  assert.equal(drifted.manifest_state, 'INVALID');
  assert.equal(drifted.blocker_program_approved, false);
});

test('pilot_approval_covers_the_same_exact_requirements_as_the_machine_manifest', () => {
  // CORRECTNESS — a valid ledger and a valid manifest cannot authorize different pilot products.
  const rows = [
    {
      requirement_id: 'SB-TST-001',
      release_disposition: 'PILOT-BLOCKER',
      next_action: 'implement the bounded contract',
      dependencies: 'cited source',
      blocking_decision: 'none',
    },
    {
      requirement_id: 'SB-TST-002',
      release_disposition: 'DEFERRED',
      next_action: 'later',
      dependencies: 'pilot manifest',
      blocking_decision: 'owner approved deferral',
    },
  ];
  const policy = {
    state: 'APPROVED',
    approved_by: 'Jauhar',
    approved_on: '2026-08-12',
    disposition_sha256: hashPilotDispositions(rows),
    requirement_ids_sha256: hashRequirementIds(['SB-TST-002']),
    approved_blocker_count: 1,
  };
  const mismatchedManifest = {
    valid: true,
    state: 'APPROVED',
    requirement_ids: ['SB-TST-002'],
    approval: {
      state: 'APPROVED',
      approved_by: 'Jauhar',
      approved_on: '2026-08-12',
    },
  };

  const result = derivePilotProgram(rows, policy, mismatchedManifest);

  assert.equal(result.manifest_state, 'INVALID');
  assert.equal(result.manifest_matches_blockers, false);
  assert.equal(result.blocker_program_approved, false);
});

test('an_approved_policy_cannot_promote_a_manifest_that_still_records_pending_owner_approval', () => {
  // CORRECTNESS — owner approval must be explicit and consistent in both approval records.
  const rows = [{
    requirement_id: 'SB-TST-001',
    release_disposition: 'PILOT-BLOCKER',
    next_action: 'implement the bounded contract',
    dependencies: 'cited source',
    blocking_decision: 'none',
  }];
  const policy = {
    state: 'APPROVED',
    approved_by: 'Jauhar',
    approved_on: '2026-08-12',
    disposition_sha256: hashPilotDispositions(rows),
    requirement_ids_sha256: hashRequirementIds(['SB-TST-001']),
    approved_blocker_count: 1,
  };
  const proposedManifest = {
    valid: true,
    state: 'PROPOSED',
    requirement_ids: ['SB-TST-001'],
    approval: {
      state: 'PENDING',
      approved_by: '',
      approved_on: '',
    },
  };

  assert.equal(derivePilotProgram(rows, policy, proposedManifest).manifest_state, 'INVALID');
});

test('pilot_approval_is_invalidated_when_the_exact_manifest_requirement_hash_drifts', () => {
  // CORRECTNESS — approval binds the exact machine-readable ID set, not a mutable file path.
  const rows = [{
    requirement_id: 'SB-TST-001',
    release_disposition: 'PILOT-BLOCKER',
    next_action: 'implement the bounded contract',
    dependencies: 'cited source',
    blocking_decision: 'none',
  }];
  const manifest = {
    valid: true,
    state: 'APPROVED',
    requirement_ids: ['SB-TST-001'],
    approval: {
      state: 'APPROVED',
      approved_by: 'Jauhar',
      approved_on: '2026-08-12',
    },
  };
  const policy = {
    state: 'APPROVED',
    approved_by: 'Jauhar',
    approved_on: '2026-08-12',
    disposition_sha256: hashPilotDispositions(rows),
    requirement_ids_sha256: hashRequirementIds(['SB-TST-999']),
    approved_blocker_count: 1,
  };

  const result = derivePilotProgram(rows, policy, manifest);

  assert.equal(result.manifest_state, 'INVALID');
  assert.equal(result.manifest_hash_matches, false);
});

test('only_named_gate_one_audit_paths_are_excluded_from_the_production_diff', () => {
  // CORRECTNESS — an audit whitelist cannot grow to absorb an application change.
  const changed = classifyProductionChanges([
    'docs/takeover/STATUS.md',
    'docs/superpowers/plans/2026-08-12-audit.md',
    'tools/gate1-final-audit.mjs',
    'src-tauri/src/modules.rs',
    'package.json',
  ]);

  assert.deepEqual(changed, ['package.json', 'src-tauri/src/modules.rs']);
});

test('a_full_gate_receipt_is_fresh_by_lineage_and_evidence_only_diff_not_by_age', () => {
  // CORRECTNESS — no uncited time limit defines freshness; source lineage and changed paths do.
  const receipt = {
    schema_version: 1,
    tested_commit: 'a'.repeat(40),
    commands: [
      { command: 'npx tsc --noEmit', exit_code: 0 },
      { command: 'cargo check', exit_code: 0 },
      {
        command: 'powershell -ExecutionPolicy Bypass -File tools\\check.ps1',
        exit_code: 0,
      },
    ],
    full_gate: { passed: 946, failed: 0, ignored: 36 },
  };

  assert.deepEqual(validateFullGateReceipt(receipt, {
    tested_commit_exists: true,
    tested_commit_is_ancestor: true,
    post_test_changed_paths: ['docs/takeover/evidence/gate1-full-gate.json'],
  }), {
    present: true,
    fresh: true,
    failed: 0,
    tested_commit_is_ancestor: true,
    post_test_changes_are_evidence_only: true,
  });

  assert.equal(validateFullGateReceipt(receipt, {
    tested_commit_exists: true,
    tested_commit_is_ancestor: true,
    post_test_changed_paths: ['src/main.ts'],
  }).fresh, false);
});

function evidenceFixture() {
  const rows = Array.from({ length: 29 }, (_, index) => ({
    requirement_id: `SB-TST-${String(index + 1).padStart(3, '0')}`,
    as_built_status: 'PRESENT-OK',
    test_class: index === 0 ? 'CORRECTNESS' : 'MISSING',
    expected_value_source: index === 0 ? 'named chapter source' : 'none - no numeric oracle claimed',
    manual_evidence: 'workflow 0/1; automation closes none',
    commit_state: 'INTEGRATED',
  }));
  const claims = [
    '# Claims',
    '',
    ...Array.from({ length: 29 }, (_, index) => (
      `| CLAIM-${String(index + 1).padStart(3, '0')} | claim | surface | audience | evidence | PROVEN | action | owner |`
    )),
    '',
    'State totals: `29` claims total.',
  ].join('\n');
  return {
    rows,
    evidenceRows: [{ requirement_id: 'SB-TST-001' }],
    exactTestMapValid: true,
    branchText: '- `UNRESOLVED`: `0`.',
    missingDomainReceipts: [],
    claimsText: claims,
    manualMatrixCurrent: true,
  };
}

test('a_correctness_claim_without_a_named_expected_source_is_unresolved', () => {
  // CORRECTNESS — CONTRACT.md §3 requires the source of every correctness expectation.
  const fixture = evidenceFixture();
  fixture.rows[0].expected_value_source = 'none';

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.unresolved_citations, 1);
});

test('a_branch_inventory_without_an_explicit_zero_unresolved_total_is_not_complete', () => {
  // CORRECTNESS — an absent total cannot be interpreted as zero unresolved branch work.
  const fixture = evidenceFixture();
  fixture.branchText = '# branch inventory without a measured unresolved total';

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.unresolved_branch_commits, 1);
});

test('the_claim_register_must_retain_all_twenty_nine_stable_inventory_ids', () => {
  // CORRECTNESS — Gate 1 inventoried 29 customer-facing claims and each stable ID is evidence.
  const fixture = evidenceFixture();
  fixture.claimsText = fixture.claimsText.replace('CLAIM-029', 'CLAIM-028');

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.claims_inventory_complete, false);
});

test('the_claim_total_can_follow_its_state_breakdown_without_becoming_unresolved', () => {
  // CORRECTNESS — CLAIMS.md records state counts before its parenthesized 29-row total.
  const fixture = evidenceFixture();
  fixture.claimsText = fixture.claimsText.replace(
    'State totals: `29` claims total.',
    'State totals: `5 PROVEN`, `23 QUALIFIED`, `1 UNDECIDED` (`29` claims total).',
  );

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.claims_inventory_complete, true);
});

test('a_wrapped_claim_state_total_still_resolves_to_the_same_twenty_nine_rows', () => {
  // CORRECTNESS — the live register wraps its state breakdown before the total.
  const fixture = evidenceFixture();
  fixture.claimsText = fixture.claimsText.replace(
    'State totals: `29` claims total.',
    [
      'State totals: `5 PROVEN`, `6 QUALIFIED`, `3 UNMEASURED`, `11 REMOVE-RECOMMENDED`,',
      '`3 LEGAL-REVIEW`, `1 UNDECIDED` (`29` claims total).',
    ].join('\n'),
  );

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.claims_inventory_complete, true);
});

test('an_adjudicated_row_without_manual_or_commit_evidence_remains_an_evidence_gap', () => {
  // CORRECTNESS — automated adjudication does not erase manual state or commit reachability.
  const fixture = evidenceFixture();
  fixture.rows[0].manual_evidence = '';
  fixture.rows[1].commit_state = 'UNVERIFIED';
  fixture.missingDomainReceipts = ['docs/takeover/evidence/sb-tst.md'];

  const evidence = deriveEvidenceFacts(fixture);

  assert.equal(evidence.adjudicated_rows_without_manual_evidence, 1);
  assert.equal(evidence.adjudicated_rows_with_unverified_commits, 1);
  assert.deepEqual(evidence.missing_domain_receipts, ['docs/takeover/evidence/sb-tst.md']);
});

test('the_gate_one_report_names_every_criterion_and_never_hides_an_open_result', () => {
  // CORRECTNESS — the final audit must expose each exit criterion independently.
  const facts = completeFacts();
  facts.pilot_program.manifest_state = 'PROPOSED';
  const report = evaluateGate1Exit(facts);

  const markdown = renderGate1Report(report, {
    generated_at: '2026-08-12T12:00:00.000Z',
    head: 'a'.repeat(40),
    diagnostics: [],
  });

  assert.match(markdown, /^# Gate 1 final audit — OPEN$/mu);
  for (let index = 1; index <= 7; index += 1) {
    assert.match(markdown, new RegExp(`\\| G1-C${index} \\|`));
  }
  assert.match(markdown, /G1-C6[^\n]*OPEN/u);
});

function pilotManifestFixture() {
  return {
    schema_version: 1,
    program_id: 'G1-PILOT-TEST',
    state: 'PROPOSED',
    default_excluded_disposition: 'DEFERRED',
    included_requirement_count: 2,
    capability_groups: [
      {
        id: 'GROUP_A',
        title: 'first group',
        requirement_ids: ['SB-TST-001'],
      },
      {
        id: 'GROUP_B',
        title: 'second group',
        requirement_ids: ['SB-TST-002'],
      },
    ],
  };
}

test('every_pilot_requirement_must_exist_in_the_ledger_and_appear_exactly_once', () => {
  // CORRECTNESS — an exact owner approval cannot cover duplicate or unknown requirement IDs.
  const ledger = [
    { requirement_id: 'SB-TST-001' },
    { requirement_id: 'SB-TST-002' },
  ];
  const manifest = pilotManifestFixture();
  manifest.capability_groups[1].requirement_ids = ['SB-TST-001', 'SB-TST-999'];
  manifest.included_requirement_count = 3;

  assert.throws(
    () => validatePilotScopeManifest(manifest, ledger),
    /duplicate pilot requirement SB-TST-001.*unknown pilot requirement SB-TST-999/su,
  );
});

test('the_pilot_manifest_count_must_equal_the_exact_group_union', () => {
  // CORRECTNESS — a prose total cannot disagree with the machine-readable requirement set.
  const manifest = pilotManifestFixture();
  manifest.included_requirement_count = 3;

  assert.throws(
    () => validatePilotScopeManifest(manifest, [
      { requirement_id: 'SB-TST-001' },
      { requirement_id: 'SB-TST-002' },
    ]),
    /declares 3 included requirements but groups contain 2/u,
  );
});

test('requirements_outside_the_first_pilot_remain_explicitly_deferred_not_silently_dropped', () => {
  // CORRECTNESS — Gate 5 requires later requirements to remain visible and explicitly deferred.
  const manifest = pilotManifestFixture();
  manifest.default_excluded_disposition = 'OUT';

  assert.throws(
    () => validatePilotScopeManifest(manifest, [
      { requirement_id: 'SB-TST-001' },
      { requirement_id: 'SB-TST-002' },
    ]),
    /default excluded disposition must be DEFERRED/u,
  );
});

test('the_pilot_manifest_schema_state_and_capability_groups_must_be_explicit_and_unambiguous', () => {
  // CORRECTNESS — malformed structure cannot carry an exact product-scope approval.
  const manifest = pilotManifestFixture();
  manifest.schema_version = 2;
  manifest.state = 'DRAFT';
  manifest.capability_groups[0].title = '';
  manifest.capability_groups[1].id = 'GROUP_A';

  assert.throws(
    () => validatePilotScopeManifest(manifest, [
      { requirement_id: 'SB-TST-001', as_built_status: 'PRESENT-OK' },
      { requirement_id: 'SB-TST-002', as_built_status: 'PRESENT-OK' },
    ]),
    /schema version must be 1.*state must be PROPOSED or APPROVED.*nonblank title.*duplicate capability group GROUP_A/su,
  );
});

test('an_unadjudicated_requirement_cannot_enter_the_first_pilot_manifest', () => {
  // CORRECTNESS — the approved release program cannot conceal an unknown as-built state.
  const manifest = pilotManifestFixture();

  assert.throws(
    () => validatePilotScopeManifest(manifest, [
      { requirement_id: 'SB-TST-001', as_built_status: 'PRESENT-OK' },
      { requirement_id: 'SB-TST-002', as_built_status: 'UNADJUDICATED' },
    ]),
    /unadjudicated pilot requirement SB-TST-002/u,
  );
});
