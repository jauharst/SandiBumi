import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  findBroadDeadCodeAllowances,
  parseCargoWarnings,
  scanIgnoredTests,
  validateHygieneInventory,
} from './gate2-hygiene.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function cargoLine({
  level = 'warning',
  lint = 'dead_code',
  message = 'function `pilot_helper` is never used',
  sourcePath = 'src\\plotting.rs',
  line = 40,
} = {}) {
  return JSON.stringify({
    reason: 'compiler-message',
    target: { kind: ['lib'], name: 'sandibumi' },
    message: {
      level,
      code: lint === null ? null : { code: lint },
      message,
      spans: [{ file_name: sourcePath, is_primary: true, line_start: line }],
    },
  });
}

function ignoredSources() {
  return [{
    sourcePath: 'src/package.rs',
    text: `
#[test]
#[ignore = "needs Pillow"]
fn package_case() {}
`,
  }, {
    sourcePath: 'src/field.rs',
    text: `
#[test]
#[ignore]
fn field_case() {
    let _ = std::env::var("SANDIBUMI_FIELD_FIXTURES");
}
`,
  }, {
    sourcePath: 'src/artifact.rs',
    text: `
#[test]
#[ignore]
fn dump_sample() {}
`,
  }];
}

function validFixture() {
  const compilerWarnings = parseCargoWarnings(cargoLine(), { repoRoot: 'C:/repo/src-tauri' });
  const ignoredTests = scanIgnoredTests(ignoredSources());
  const warningInventory = {
    schema_version: 1,
    expected_warning_count: 1,
    expected_warning_counts_by_file: { 'src/plotting.rs': 1 },
    warnings: [{
      ...compilerWarnings[0],
      owner_requirement: 'SB-PLT-001',
      owner_gate: 'G2',
      disposition: 'CONNECT-OR-REMOVE-DURING-OWNER-REQUIREMENT',
      rationale: 'The pilot plot path must either consume this contract or stop claiming it.',
    }],
  };
  const ignoredInventory = {
    schema_version: 1,
    expected_ignored_test_count: 3,
    expected_category_counts: {
      'OPTIONAL-PACKAGE': 1,
      'CONTROLLED-CORPUS': 1,
      'MANUAL-ARTIFACT': 1,
    },
    tests: [{
      ...ignoredTests.find((item) => item.test_name === 'package_case'),
      category: 'OPTIONAL-PACKAGE',
      owner_requirement: 'SB-INS-006',
      owner_gate: 'G3',
      execution_state: 'NOT-RUN-BY-DEFAULT',
      rationale: 'Qualified offline package execution belongs to Gate 3.',
    }, {
      ...ignoredTests.find((item) => item.test_name === 'field_case'),
      category: 'CONTROLLED-CORPUS',
      owner_requirement: 'SB-CORE-040',
      owner_gate: 'G4',
      execution_state: 'UNVERIFIED',
      rationale: 'Controlled field-corpus execution belongs to Gate 4.',
    }, {
      ...ignoredTests.find((item) => item.test_name === 'dump_sample'),
      category: 'MANUAL-ARTIFACT',
      owner_requirement: 'SB-PLT-023',
      owner_gate: 'G4',
      execution_state: 'MANUAL-ONLY',
      rationale: 'This generates an artifact for visual review and is not an acceptance pass.',
    }],
  };
  return {
    compilerWarnings,
    ignoredTests,
    warningInventory,
    ignoredInventory,
    broadAllowances: [],
  };
}

test('warning_identity_survives_a_line_shift_but_not_a_changed_lint_or_message', () => {
  const first = parseCargoWarnings(cargoLine({ line: 40 }), { repoRoot: 'C:/repo/src-tauri' })[0];
  const shifted = parseCargoWarnings(cargoLine({ line: 400 }), { repoRoot: 'C:/repo/src-tauri' })[0];
  const changed = parseCargoWarnings(cargoLine({ message: 'function `other_helper` is never used' }), {
    repoRoot: 'C:/repo/src-tauri',
  })[0];

  assert.equal(first.key, shifted.key);
  assert.notEqual(first.key, changed.key);
  assert.equal(first.source_path, 'src/plotting.rs');
  assert.equal(first.lint, 'dead_code');
});

test('an_unclassified_warning_is_a_gate_failure_even_when_the_declared_total_is_adjusted', () => {
  const fixture = validFixture();
  const extra = parseCargoWarnings(cargoLine({
    message: 'function `second_helper` is never used',
    line: 80,
  }), { repoRoot: 'C:/repo/src-tauri' })[0];
  fixture.compilerWarnings.push(extra);
  fixture.warningInventory.expected_warning_count = 2;
  fixture.warningInventory.expected_warning_counts_by_file['src/plotting.rs'] = 2;

  assert.throws(
    () => validateHygieneInventory(fixture),
    /unclassified warning .*second_helper/u,
  );
});

test('a_crate_or_module_wide_dead_code_allowance_cannot_manufacture_a_clean_gate', () => {
  const fixture = validFixture();
  fixture.broadAllowances = findBroadDeadCodeAllowances([{ sourcePath: 'src/lib.rs', text: '#![allow(dead_code)]' }]);

  assert.throws(
    () => validateHygieneInventory(fixture),
    /broad dead_code allowance at src\/lib.rs:1/u,
  );
});

test('a_new_bare_ignored_test_fails_until_it_has_an_explicit_owner_and_execution_class', () => {
  const fixture = validFixture();
  fixture.ignoredTests.push(...scanIgnoredTests([{
    sourcePath: 'src/new.rs',
    text: '#[test]\n#[ignore]\nfn new_bare_case() {}\n',
  }]));
  fixture.ignoredInventory.expected_ignored_test_count = 4;

  assert.throws(
    () => validateHygieneInventory(fixture),
    /bare ignored test src\/new.rs\|new_bare_case has no inventory owner/u,
  );
});

test('an_ignored_test_cannot_have_two_owners_even_when_both_rows_agree', () => {
  const fixture = validFixture();
  fixture.ignoredInventory.tests.push({ ...fixture.ignoredInventory.tests[0] });
  fixture.ignoredInventory.expected_ignored_test_count = 4;
  fixture.ignoredInventory.expected_category_counts['OPTIONAL-PACKAGE'] = 2;

  assert.throws(
    () => validateHygieneInventory(fixture),
    /duplicate ignored-test owner src\/package.rs\|package_case/u,
  );
});

test('a_field_fixture_test_cannot_be_relabelled_as_optional_package_execution', () => {
  const fixture = validFixture();
  const field = fixture.ignoredInventory.tests.find((item) => item.test_name === 'field_case');
  field.category = 'OPTIONAL-PACKAGE';
  field.owner_gate = 'G3';
  fixture.ignoredInventory.expected_category_counts['OPTIONAL-PACKAGE'] = 2;
  fixture.ignoredInventory.expected_category_counts['CONTROLLED-CORPUS'] = 0;

  assert.throws(
    () => validateHygieneInventory(fixture),
    /field signal .*field_case.*CONTROLLED-CORPUS/u,
  );
});

test('the_live_inventory_owns_36_warnings_and_37_ignored_tests_without_counting_them_as_passed', () => {
  // CHARACTERIZATION — the live compiler/test inventory is 36 warnings and 37 ignored tests
  // after SB-PLT-006 removed the disconnected plotting-local histogram wrapper.
  // Owning remaining debt does not prove it passes.
  const warningInventory = JSON.parse(fs.readFileSync(
    path.join(repo, 'docs', 'takeover', 'evidence', 'gate2-warning-inventory.json'),
    'utf8',
  ));
  const ignoredInventory = JSON.parse(fs.readFileSync(
    path.join(repo, 'docs', 'takeover', 'evidence', 'gate2-ignored-test-inventory.json'),
    'utf8',
  ));

  assert.equal(warningInventory.expected_warning_count, 36);
  assert.equal(warningInventory.expected_warning_counts_by_file['src/plotting.rs'], 26);
  assert.equal(ignoredInventory.expected_ignored_test_count, 37);
  assert.deepEqual(ignoredInventory.expected_category_counts, {
    'OPTIONAL-PACKAGE': 27,
    'CONTROLLED-CORPUS': 9,
    'MANUAL-ARTIFACT': 1,
  });
  const executionStates = [
    ...(ignoredInventory.tests ?? []).map((item) => item.execution_state),
    ...(ignoredInventory.test_groups ?? []).flatMap((group) => group.tests.map(() => group.execution_state)),
  ];
  assert.equal(
    executionStates.filter((state) => state === 'PASSED').length,
    0,
  );
});
