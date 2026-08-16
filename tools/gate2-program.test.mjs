import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  deriveGate2ActionMode,
  validateGate2Program,
} from './gate2-program.mjs';
import { parseCsv } from './takeover-ledger.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function ledgerRow(requirementId, asBuiltStatus, testClass) {
  return {
    requirement_id: requirementId,
    release_disposition: 'PILOT-BLOCKER',
    as_built_status: asBuiltStatus,
    test_class: testClass,
  };
}

function fixture() {
  const rows = [
    ledgerRow('SB-A-001', 'ABSENT', 'MISSING'),
    ledgerRow('SB-A-002', 'PARTIAL', 'CHARACTERIZATION'),
    ledgerRow('SB-A-003', 'PRESENT-UNVERIFIED', 'MISSING'),
    ledgerRow('SB-A-004', 'PRESENT-OK', 'CORRECTNESS'),
    ledgerRow('SB-A-005', 'PRESENT-OK', 'CORRECTNESS'),
  ];
  const pilot = {
    schema_version: 1,
    state: 'APPROVED',
    included_requirement_count: rows.length,
    capability_groups: [{
      id: 'A',
      title: 'Fixture',
      requirement_ids: rows.map((row) => row.requirement_id),
    }],
  };
  const program = {
    schema_version: 1,
    state: 'PLANNED',
    baseline_merge_commit: 'a'.repeat(40),
    approved_scope_sha256: 'fixture',
    gate2_requirement_count: 4,
    later_gate_requirement_count: 1,
    action_mode_counts: {
      'IMPLEMENT-OR-REFUSE': 1,
      REMEDIATE: 1,
      PROVE: 1,
      RETAIN: 1,
    },
    tranches: [{
      id: 'G2-T01',
      title: 'Fixture tranche',
      requirement_ids: rows.slice(0, 4).map((row) => row.requirement_id),
    }],
    completed_requirements: [],
    blocked_requirements: [],
    later_gate_only: [{
      requirement_id: 'SB-A-005',
      owner_gate: 'G3',
      state: 'NOT-OWNED-BY-G2',
      reason: 'Fixture deployment evidence',
    }],
  };
  return { rows, pilot, program };
}

test('gate_two_action_modes_follow_as_built_and_test_evidence_instead_of_row_count', () => {
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-001', 'ABSENT', 'MISSING')), 'IMPLEMENT-OR-REFUSE');
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-002', 'PARTIAL', 'CORRECTNESS')), 'REMEDIATE');
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-003', 'PRESENT-DIVERGENT', 'MISSING')), 'REMEDIATE');
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-004', 'PRESENT-UNVERIFIED', 'MISSING')), 'PROVE');
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-005', 'PRESENT-OK', 'CHARACTERIZATION')), 'PROVE');
  assert.equal(deriveGate2ActionMode(ledgerRow('SB-A-006', 'PRESENT-OK', 'CORRECTNESS')), 'RETAIN');
});

test('the_gate_two_program_accounts_for_every_approved_row_once_without_stealing_later_gate_work', () => {
  const { rows, pilot, program } = fixture();
  const result = validateGate2Program(program, pilot, rows, { approvedScopeHash: 'fixture' });

  assert.equal(result.valid, true);
  assert.equal(result.approved, 5);
  assert.equal(result.gate2, 4);
  assert.equal(result.later, 1);
});

test('a_requirement_routed_twice_is_not_a_complete_gate_two_plan', () => {
  const { rows, pilot, program } = fixture();
  program.later_gate_only[0].requirement_id = 'SB-A-001';

  assert.throws(
    () => validateGate2Program(program, pilot, rows, { approvedScopeHash: 'fixture' }),
    /duplicate routed requirement SB-A-001/u,
  );
});

test('a_missing_approved_requirement_is_not_hidden_by_matching_declared_counts', () => {
  const { rows, pilot, program } = fixture();
  program.tranches[0].requirement_ids.pop();
  program.gate2_requirement_count = 3;
  program.action_mode_counts.RETAIN = 0;

  assert.throws(
    () => validateGate2Program(program, pilot, rows, { approvedScopeHash: 'fixture' }),
    /approved requirement SB-A-004 is not routed/u,
  );
});

test('a_retained_row_requires_both_present_ok_behavior_and_a_correctness_proof', () => {
  const { rows, pilot, program } = fixture();
  program.action_mode_counts.REMEDIATE = 0;
  program.action_mode_counts.RETAIN = 2;

  assert.throws(
    () => validateGate2Program(program, pilot, rows, { approvedScopeHash: 'fixture' }),
    /REMEDIATE count 0 does not match routed 1/u,
  );
});

test('the_live_gate_two_program_routes_the_approved_242_rows_into_222_gate_two_and_20_later_obligations', () => {
  // CORRECTNESS — DEC-018 fixes the exact 242-ID pilot scope; takeover design §§7-10
  // separate engineering truth closure from install qualification and field evidence.
  const rows = parseCsv(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'requirements.csv'), 'utf8'));
  const pilot = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'pilot-scope.json'), 'utf8'));
  const program = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'gate2-program.json'), 'utf8'));
  const result = validateGate2Program(program, pilot, rows);

  assert.equal(result.approved, 242);
  assert.equal(result.gate2, 222);
  assert.equal(result.later, 20);
  assert.deepEqual(result.action_mode_counts, {
    'IMPLEMENT-OR-REFUSE': 21,
    REMEDIATE: 50,
    PROVE: 8,
    RETAIN: 143,
  });
});

test('the_live_gate_two_progress_receipt_accounts_for_every_handled_row_once', () => {
  const program = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'gate2-program.json'), 'utf8'));
  const handled = new Set([...program.completed_requirements, ...program.blocked_requirements]);

  assert.equal(program.completed_requirements.length, 145);
  assert.equal(program.blocked_requirements.length, 62);
  assert.equal(handled.size, 207, 'completed and blocked receipts must not overlap');
  assert.equal(program.gate2_requirement_count - handled.size, 15);
  assert.ok(program.completed_requirements.includes('SB-CLY-050'));
  assert.ok(program.completed_requirements.includes('SB-CLY-051'));
  assert.ok(program.completed_requirements.includes('SB-CLY-054'));
  assert.ok(program.blocked_requirements.includes('SB-CLY-055'));
  assert.ok(program.completed_requirements.includes('SB-POR-001'));
  assert.ok(program.blocked_requirements.includes('SB-POR-002'));
  assert.ok(program.blocked_requirements.includes('SB-POR-003'));
  assert.ok(program.completed_requirements.includes('SB-POR-004'));
  assert.ok(program.completed_requirements.includes('SB-POR-006'));
  assert.ok(program.completed_requirements.includes('SB-POR-007'));
  assert.ok(program.completed_requirements.includes('SB-POR-008'));
  assert.ok(program.completed_requirements.includes('SB-POR-009'));
  assert.ok(program.blocked_requirements.includes('SB-POR-010'));
  assert.ok(program.completed_requirements.includes('SB-POR-011'));
  assert.ok(program.blocked_requirements.includes('SB-POR-021'));
  assert.ok(program.completed_requirements.includes('SB-POR-023'));
  assert.ok(program.blocked_requirements.includes('SB-POR-024'));
  assert.ok(program.blocked_requirements.includes('SB-POR-025'));
  assert.ok(program.blocked_requirements.includes('SB-POR-026'));
  assert.ok(program.blocked_requirements.includes('SB-POR-028'));
  assert.ok(program.completed_requirements.includes('SB-POR-043'));
  assert.ok(program.blocked_requirements.includes('SB-POR-044'));
  assert.ok(program.blocked_requirements.includes('SB-POR-045'));
  assert.ok(program.blocked_requirements.includes('SB-POR-047'));
  assert.ok(program.blocked_requirements.includes('SB-POR-048'));
  assert.ok(program.completed_requirements.includes('SB-POR-049'));
  assert.ok(program.blocked_requirements.includes('SB-POR-054'));
  assert.ok(program.blocked_requirements.includes('SB-POR-055'));
  assert.ok(program.completed_requirements.includes('SB-POR-056'));
  assert.ok(program.blocked_requirements.includes('SB-POR-057'));
  assert.ok(program.completed_requirements.includes('SB-SAT-001'));
  assert.ok(program.blocked_requirements.includes('SB-SAT-002'));
  assert.ok(program.completed_requirements.includes('SB-SAT-006'));
  assert.ok(program.blocked_requirements.includes('SB-SAT-023'));
  assert.ok(program.blocked_requirements.includes('SB-SAT-025'));
  assert.ok(program.blocked_requirements.includes('SB-SAT-026'));
  assert.ok(program.blocked_requirements.includes('SB-SAT-027'));
  assert.ok(program.completed_requirements.includes('SB-SAT-028'));
  assert.ok(program.completed_requirements.includes('SB-SAT-029'));
  assert.ok(program.completed_requirements.includes('SB-SAT-030'));
  assert.ok(program.completed_requirements.includes('SB-SAT-031'));
  assert.ok(program.completed_requirements.includes('SB-SAT-034'));
  assert.ok(program.completed_requirements.includes('SB-SAT-038'));
  assert.ok(program.completed_requirements.includes('SB-SAT-047'));
  assert.ok(program.blocked_requirements.includes('SB-CUT-001'));
  assert.ok(program.blocked_requirements.includes('SB-CUT-002'));
  assert.ok(program.completed_requirements.includes('SB-CUT-003'));
  assert.ok(program.completed_requirements.includes('SB-CUT-004'));
  assert.ok(program.completed_requirements.includes('SB-CUT-005'));
  assert.ok(program.completed_requirements.includes('SB-CUT-009'));
  assert.ok(program.completed_requirements.includes('SB-CUT-010'));
  assert.ok(program.completed_requirements.includes('SB-CUT-011'));
  assert.ok(program.completed_requirements.includes('SB-CUT-012'));
});

test('the_integrated_gate_two_blocker_packet_accounts_for_each_live_blocked_requirement_once_and_is_linked_from_the_dashboard', () => {
  // CORRECTNESS — gate2-program.json is the machine-owned live blocker set;
  // STATUS.md is the one-minute dashboard and must route readers to its maintained detail.
  const program = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'gate2-program.json'), 'utf8'));
  const blockerPath = path.join(repo, 'docs', 'takeover', 'GATE2_BLOCKERS.md');
  const blockerDocument = fs.readFileSync(blockerPath, 'utf8');
  const status = fs.readFileSync(path.join(repo, 'docs', 'takeover', 'STATUS.md'), 'utf8');
  const documented = [...blockerDocument.matchAll(/^\| `(SB-[A-Z]+-\d{3})` \|/gmu)]
    .map((match) => match[1]);

  assert.equal(documented.length, new Set(documented).size, 'a blocked requirement must be documented once');
  assert.deepEqual(
    documented.toSorted(),
    [...program.blocked_requirements].toSorted(),
    'the human decision packet must equal the machine-owned blocker set',
  );
  assert.match(status, /\[Gate 2 blocker decision packet\]\(\.\/GATE2_BLOCKERS\.md\)/u);
});
