import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  checkPrdAudit,
  checkTracker,
  discoverExecutableTests,
  parseCsv,
  parseTestEvidence,
  validateTestEvidence,
} from './takeover-ledger.mjs';

const defaultRepo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

export function commandExitedSuccessfully(result) {
  return result?.status === 0;
}

function hashLines(lines) {
  const canonical = `${[...lines].sort().join('\n')}\n`;
  return crypto.createHash('sha256').update(canonical, 'utf8').digest('hex');
}

export function hashRequirementIds(requirementIds) {
  return hashLines(requirementIds);
}

export function hashPilotDispositions(rows) {
  return hashLines(rows.map((row) => `${row.requirement_id}=${row.release_disposition}`));
}

export function validatePilotScopeManifest(manifest, ledgerRows) {
  const errors = [];
  if (manifest.schema_version !== 1) {
    errors.push('pilot manifest schema version must be 1');
  }
  if (!['PROPOSED', 'APPROVED'].includes(manifest.state)) {
    errors.push('pilot manifest state must be PROPOSED or APPROVED');
  }
  if (manifest.default_excluded_disposition !== 'DEFERRED') {
    errors.push('default excluded disposition must be DEFERRED');
  }

  const ledgerById = new Map(ledgerRows.map((row) => [row.requirement_id, row]));
  const seen = new Set();
  const seenGroups = new Set();
  const requirementIds = [];
  const groups = Array.isArray(manifest.capability_groups) ? manifest.capability_groups : [];
  if (groups.length === 0) {
    errors.push('pilot manifest must contain at least one capability group');
  }
  for (const [groupIndex, group] of groups.entries()) {
    const groupId = typeof group.id === 'string' ? group.id.trim() : '';
    if (groupId === '') {
      errors.push(`capability group ${groupIndex + 1} must have a nonblank id`);
    }
    const groupTitle = typeof group.title === 'string' ? group.title.trim() : '';
    if (groupTitle === '') {
      errors.push(`capability group ${groupId || groupIndex + 1} must have a nonblank title`);
    }
    if (groupId !== '' && seenGroups.has(groupId)) {
      errors.push(`duplicate capability group ${groupId}`);
    } else if (groupId !== '') {
      seenGroups.add(groupId);
    }

    if (!Array.isArray(group.requirement_ids)) {
      errors.push(`capability group ${groupId || groupIndex + 1} must contain a requirement_ids array`);
      continue;
    }
    for (const rawRequirementId of group.requirement_ids) {
      const requirementId = typeof rawRequirementId === 'string' ? rawRequirementId.trim() : '';
      if (requirementId === '') {
        errors.push(`capability group ${groupId || groupIndex + 1} contains a blank requirement id`);
        continue;
      }
      requirementIds.push(requirementId);
      if (seen.has(requirementId)) {
        errors.push(`duplicate pilot requirement ${requirementId}`);
      } else {
        seen.add(requirementId);
      }
      if (!ledgerById.has(requirementId)) {
        errors.push(`unknown pilot requirement ${requirementId}`);
      } else if (ledgerById.get(requirementId).as_built_status === 'UNADJUDICATED') {
        errors.push(`unadjudicated pilot requirement ${requirementId}`);
      }
    }
  }

  if (manifest.included_requirement_count !== requirementIds.length) {
    errors.push(
      `pilot manifest declares ${manifest.included_requirement_count} included requirements but groups contain ${requirementIds.length}`,
    );
  }

  if (errors.length > 0) {
    throw new Error(errors.join('; '));
  }
  return {
    valid: true,
    state: manifest.state,
    requirement_ids: requirementIds,
    approval: manifest.approval ?? {},
  };
}

function conformingDeferredGeoRow(row) {
  return row.requirement_id.startsWith('SB-GEO-')
    && row.as_built_status === 'UNADJUDICATED'
    && row.release_disposition === 'DEFERRED'
    && row.risk_class === 'LATER'
    && row.test_class === 'MISSING-OR-UNCLASSIFIED'
    && row.expected_value_source === ''
    && row.manual_evidence === ''
    && row.dependencies.includes('DEC-011')
    && row.commit_state === 'UNVERIFIED'
    && row.blocking_decision.includes('DEC-011')
    && row.next_action === 'NEXT-VERSION-LIVE-ADJUDICATION'
    && row.last_reverified === '';
}

function approvalMetadataPresent(policy) {
  return policy.approved_by === 'Jauhar' && /^\d{4}-\d{2}-\d{2}$/u.test(policy.approved_on);
}

export function deriveRowTruth(rows, geoPolicy = {}) {
  const unadjudicatedRows = rows.filter((row) => row.as_built_status === 'UNADJUDICATED');
  if (unadjudicatedRows.length === 0) {
    return {
      adjudicated: rows.length,
      unadjudicated: 0,
      geo_boundary_state: 'NOT-NEEDED',
      geo_deferred_rows: 0,
      geo_rows_conform: true,
    };
  }

  const exactGeoShape = unadjudicatedRows.length === 52
    && unadjudicatedRows.every(conformingDeferredGeoRow);
  const exactGeoHash = geoPolicy.requirement_ids_sha256 === hashRequirementIds(
    unadjudicatedRows.map((row) => row.requirement_id),
  );
  const geoRowsConform = exactGeoShape && exactGeoHash;
  let state = geoPolicy.state ?? 'MISSING';
  if (state === 'APPROVED' && (!geoRowsConform || !approvalMetadataPresent(geoPolicy))) {
    state = 'INVALID';
  }

  return {
    adjudicated: rows.length - unadjudicatedRows.length,
    unadjudicated: unadjudicatedRows.length,
    geo_boundary_state: state,
    geo_deferred_rows: unadjudicatedRows.filter((row) => row.requirement_id.startsWith('SB-GEO-')).length,
    geo_rows_conform: geoRowsConform,
  };
}

export function derivePilotProgram(rows, pilotPolicy = {}, manifest = null) {
  const blockers = rows.filter((row) => row.release_disposition === 'PILOT-BLOCKER');
  const dispositionHashMatches = pilotPolicy.disposition_sha256 === hashPilotDispositions(rows);
  const blockerCountMatches = pilotPolicy.approved_blocker_count === blockers.length;
  const blockerIds = new Set(blockers.map((row) => row.requirement_id));
  const manifestIds = Array.isArray(manifest?.requirement_ids) ? manifest.requirement_ids : [];
  const manifestMatchesBlockers = manifest?.valid === true
    && manifestIds.length === blockerIds.size
    && manifestIds.every((requirementId) => blockerIds.has(requirementId));
  const manifestHashMatches = manifest?.valid === true
    && pilotPolicy.requirement_ids_sha256 === hashRequirementIds(manifestIds);
  const manifestStateMatches = manifest?.state === pilotPolicy.state;
  const manifestApprovalMatches = pilotPolicy.state !== 'APPROVED'
    || (
      manifest?.approval?.state === 'APPROVED'
      && manifest.approval.approved_by === pilotPolicy.approved_by
      && manifest.approval.approved_on === pilotPolicy.approved_on
    );
  let state = pilotPolicy.state ?? 'MISSING';
  if (
    state === 'APPROVED'
    && (
      manifest?.valid !== true
      || !manifestStateMatches
      || !manifestApprovalMatches
      || !manifestMatchesBlockers
      || !manifestHashMatches
      || !approvalMetadataPresent(pilotPolicy)
      || !dispositionHashMatches
      || !blockerCountMatches
    )
  ) {
    state = 'INVALID';
  }

  return {
    manifest_state: state,
    manifest_matches_blockers: manifestMatchesBlockers,
    manifest_hash_matches: manifestHashMatches,
    requirements_covered: rows.filter((row) => (
      row.release_disposition === 'PILOT-BLOCKER'
      || row.release_disposition === 'DEFERRED'
      || row.release_disposition === 'OUT'
    )).length,
    undecided_requirements: rows.filter((row) => row.release_disposition === 'UNDECIDED').length,
    blocker_program_approved: state === 'APPROVED',
    blockers_without_next_action: blockers.filter((row) => !row.next_action.trim()).length,
    blockers_without_dependency: blockers.filter((row) => !row.dependencies.trim()).length,
    blockers_without_owner_decision: blockers.filter((row) => !row.blocking_decision.trim()).length,
  };
}

const GATE1_AUDIT_PATH_PREFIXES = [
  'docs/takeover/',
  'docs/superpowers/plans/',
];

const GATE1_AUDIT_PATHS = new Set([
  'tools/gate1-final-audit.mjs',
  'tools/gate1-final-audit.test.mjs',
  'tools/takeover-ledger.mjs',
  'tools/takeover-ledger.test.mjs',
]);

function isGate1AuditPath(filePath) {
  const normalized = filePath.replaceAll('\\', '/');
  return GATE1_AUDIT_PATHS.has(normalized)
    || GATE1_AUDIT_PATH_PREFIXES.some((prefix) => normalized.startsWith(prefix));
}

export function classifyProductionChanges(changedPaths) {
  return [...new Set(changedPaths.map((filePath) => filePath.replaceAll('\\', '/')))]
    .filter((filePath) => !isGate1AuditPath(filePath))
    .sort();
}

const POST_TEST_EVIDENCE_PATHS = new Set([
  'docs/takeover/STATUS.md',
  'docs/takeover/evidence/gate1-final-audit.md',
  'docs/takeover/evidence/gate1-full-gate.json',
]);

function postTestChangesAreEvidenceOnly(paths) {
  return paths.every((filePath) => POST_TEST_EVIDENCE_PATHS.has(filePath.replaceAll('\\', '/')));
}

export function validateFullGateReceipt(receipt, gitFacts) {
  if (!receipt) {
    return {
      present: false,
      fresh: false,
      failed: 0,
      tested_commit_is_ancestor: false,
      post_test_changes_are_evidence_only: false,
    };
  }

  const requiredCommands = new Set([
    'npx tsc --noEmit',
    'cargo check',
    'powershell -ExecutionPolicy Bypass -File tools\\check.ps1',
  ]);
  const passingCommands = new Set(
    (receipt.commands ?? [])
      .filter((command) => command.exit_code === 0)
      .map((command) => command.command),
  );
  const commandsPass = [...requiredCommands].every((command) => passingCommands.has(command));
  const fullGateCountsValid = Number.isInteger(receipt.full_gate?.passed)
    && receipt.full_gate.passed >= 0
    && receipt.full_gate?.failed === 0
    && Number.isInteger(receipt.full_gate?.ignored)
    && receipt.full_gate.ignored >= 0;
  const testedCommitValid = /^[0-9a-f]{40}$/u.test(receipt.tested_commit ?? '')
    && gitFacts.tested_commit_exists
    && gitFacts.tested_commit_is_ancestor;
  const evidenceOnly = postTestChangesAreEvidenceOnly(gitFacts.post_test_changed_paths ?? []);
  const fresh = receipt.schema_version === 1
    && commandsPass
    && fullGateCountsValid
    && testedCommitValid
    && evidenceOnly;

  return {
    present: true,
    fresh,
    failed: receipt.full_gate?.failed ?? 0,
    tested_commit_is_ancestor: Boolean(gitFacts.tested_commit_is_ancestor),
    post_test_changes_are_evidence_only: evidenceOnly,
  };
}

const CLAIMED_PROOF_CLASSES = new Set([
  'CORRECTNESS',
  'CHARACTERIZATION',
  'OPTIONAL-PACKAGE-IGNORED',
  'SPEC-DIVERGENCE-IGNORED',
]);

function correctnessSourceIsUnresolved(row) {
  if (row.test_class !== 'CORRECTNESS' && row.test_class !== 'OPTIONAL-PACKAGE-IGNORED') {
    return false;
  }
  const source = row.expected_value_source.trim().toLowerCase();
  return source === '' || /^none\b/u.test(source);
}

function claimsInventoryIsComplete(markdown) {
  const ids = [...String(markdown).matchAll(/^\| (CLAIM-\d{3}) \|/gmu)]
    .map((match) => match[1]);
  const expected = Array.from(
    { length: 29 },
    (_, index) => `CLAIM-${String(index + 1).padStart(3, '0')}`,
  );
  const declaredTotals = [...String(markdown).matchAll(/`(\d+)` claims total/gu)];
  return declaredTotals.length === 1
    && declaredTotals[0][1] === '29'
    && ids.length === 29
    && new Set(ids).size === 29
    && expected.every((id, index) => ids[index] === id);
}

function unresolvedBranchCount(markdown) {
  const match = /- `UNRESOLVED`: `(\d+)`\./u.exec(markdown);
  return match ? Number.parseInt(match[1], 10) : 1;
}

export function deriveEvidenceFacts({
  rows,
  evidenceRows,
  exactTestMapValid,
  branchText,
  missingDomainReceipts,
  claimsText,
  manualMatrixCurrent,
}) {
  const claimedRows = rows.filter((row) => CLAIMED_PROOF_CLASSES.has(row.test_class));
  const mappedRequirementIds = new Set(evidenceRows.map((row) => row.requirement_id));
  const adjudicatedRows = rows.filter((row) => row.as_built_status !== 'UNADJUDICATED');

  return {
    exact_test_map_valid: exactTestMapValid,
    claimed_test_rows: claimedRows.length,
    mapped_test_rows: mappedRequirementIds.size,
    unresolved_citations: rows.filter(correctnessSourceIsUnresolved).length,
    unresolved_branch_commits: unresolvedBranchCount(branchText),
    adjudicated_rows_without_manual_evidence: adjudicatedRows
      .filter((row) => !row.manual_evidence.trim()).length,
    adjudicated_rows_with_unverified_commits: adjudicatedRows
      .filter((row) => row.commit_state === 'UNVERIFIED').length,
    missing_domain_receipts: [...missingDomainReceipts],
    claims_inventory_complete: claimsInventoryIsComplete(claimsText),
    manual_matrix_current: manualMatrixCurrent,
  };
}

function markdownCell(value) {
  return String(value).replaceAll('|', '\\|').replaceAll(/\r?\n/gu, ' ');
}

export function renderGate1Report(report, metadata) {
  const lines = [
    `# Gate 1 final audit — ${report.state}`,
    '',
    `Generated: \`${metadata.generated_at}\``,
    '',
    `HEAD: \`${metadata.head}\``,
    '',
    '| Criterion | State | Exit contract | Evidence |',
    '|---|---|---|---|',
    ...report.criteria.map((entry) => (
      `| ${entry.id} | ${entry.state} | ${markdownCell(entry.title)} | ${markdownCell(entry.detail)} |`
    )),
    '',
  ];

  const open = report.criteria.filter((entry) => entry.state === 'OPEN');
  if (open.length > 0) {
    lines.push('## Open items', '');
    for (const entry of open) lines.push(`- **${entry.id}:** ${entry.detail}`);
    lines.push('');
  }

  if (metadata.diagnostics.length > 0) {
    lines.push('## Diagnostics', '');
    for (const diagnostic of metadata.diagnostics) lines.push(`- ${diagnostic}`);
    lines.push('');
  }

  lines.push(
    report.state === 'PASS'
      ? 'Gate 1 satisfies all seven exit criteria.'
      : 'Gate 1 remains open; no later gate is implied started.',
    '',
  );
  return lines.join('\n');
}

const DOMAIN_RECEIPTS = new Map([
  ['SB-CLY', 'docs/takeover/evidence/sb-cly.md'],
  ['SB-CORE', 'docs/takeover/evidence/sb-core.md'],
  ['SB-CUT', 'docs/takeover/evidence/sb-cut.md'],
  ['SB-DBM', 'docs/takeover/evidence/sb-dbm.md'],
  ['SB-DIO', 'docs/takeover/evidence/sb-dio.md'],
  ['SB-ENV', 'docs/takeover/evidence/sb-env.md'],
  ['SB-GEO', 'docs/takeover/evidence/sb-geo-deferral.md'],
  ['SB-INS', 'docs/takeover/evidence/sb-ins.md'],
  ['SB-MIN', 'docs/takeover/evidence/sb-min.md'],
  ['SB-MLA', 'docs/takeover/evidence/sb-mla.md'],
  ['SB-NMR', 'docs/takeover/evidence/sb-nmr.md'],
  ['SB-PLG', 'docs/takeover/evidence/sb-plg.md'],
  ['SB-PLT', 'docs/takeover/evidence/sb-plt.md'],
  ['SB-POR', 'docs/takeover/evidence/sb-por.md'],
  ['SB-RPH', 'docs/takeover/evidence/sb-rph.md'],
  ['SB-SAT', 'docs/takeover/evidence/sb-sat.md'],
  ['SB-SHR', 'docs/takeover/evidence/sb-shr.md'],
  ['SB-TBD', 'docs/takeover/evidence/sb-tbd.md'],
  ['SB-TOC', 'docs/takeover/evidence/sb-toc.md'],
]);

function run(repoRoot, command, args) {
  return spawnSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    windowsHide: true,
  });
}

function git(repoRoot, args) {
  return run(repoRoot, 'git', args);
}

function outputLines(result) {
  if (!commandExitedSuccessfully(result)) return [];
  return result.stdout.split(/\r?\n/u).map((line) => line.trim()).filter(Boolean);
}

function readJsonOrNull(filePath, diagnostics, label) {
  if (!fs.existsSync(filePath)) return null;
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    diagnostics.push(`${label} is not valid JSON: ${error.message}`);
    return null;
  }
}

function currentWorktreePaths(repoRoot) {
  return [
    ...outputLines(git(repoRoot, ['diff', '--name-only'])),
    ...outputLines(git(repoRoot, ['diff', '--cached', '--name-only'])),
    ...outputLines(git(repoRoot, ['ls-files', '--others', '--exclude-standard'])),
  ];
}

function exactTestMapStatus(repoRoot, rows, evidenceRows, diagnostics) {
  try {
    validateTestEvidence({
      ledgerRows: rows,
      evidenceRows,
      executableTests: discoverExecutableTests(repoRoot),
    });
    return true;
  } catch (error) {
    diagnostics.push(`exact test evidence: ${error.message}`);
    return false;
  }
}

function trackerStatus(diagnostics) {
  try {
    checkTracker();
    return true;
  } catch (error) {
    diagnostics.push(`takeover tracker: ${error.message}`);
    return false;
  }
}

function prdAuditStatus(diagnostics) {
  try {
    checkPrdAudit();
    return true;
  } catch (error) {
    diagnostics.push(`PRD audit: ${error.message}`);
    return false;
  }
}

export function collectLiveGate1Facts(repoRoot = defaultRepo) {
  const diagnostics = [];
  const takeover = path.join(repoRoot, 'docs', 'takeover');
  const ledgerPath = path.join(takeover, 'requirements.csv');
  const evidencePath = path.join(takeover, 'test-evidence.csv');
  const policyPath = path.join(takeover, 'gate1-policy.json');
  const rows = parseCsv(fs.readFileSync(ledgerPath, 'utf8'));
  const evidenceRows = parseTestEvidence(fs.readFileSync(evidencePath, 'utf8'));
  const policy = readJsonOrNull(policyPath, diagnostics, 'Gate 1 policy') ?? {};
  if (!fs.existsSync(policyPath)) diagnostics.push('Gate 1 policy is absent.');

  const trackerValid = trackerStatus(diagnostics);
  const exactMapValid = exactTestMapStatus(repoRoot, rows, evidenceRows, diagnostics);
  const prdAuditCurrent = prdAuditStatus(diagnostics);
  const domains = new Set(rows.map((row) => row.requirement_id.replace(/-\d+$/u, '')));
  const missingDomainReceipts = [...domains]
    .map((domain) => DOMAIN_RECEIPTS.get(domain) ?? `UNROUTED:${domain}`)
    .filter((receipt) => receipt.startsWith('UNROUTED:') || !fs.existsSync(path.join(repoRoot, receipt)))
    .sort();
  const branchText = fs.readFileSync(path.join(takeover, 'evidence', 'branches.md'), 'utf8');
  const claimsText = fs.readFileSync(path.join(takeover, 'CLAIMS.md'), 'utf8');
  const manualResult = run(
    repoRoot,
    process.execPath,
    ['tools/generate-verification-matrix.mjs', '--check'],
  );
  if (!commandExitedSuccessfully(manualResult)) {
    diagnostics.push(`manual verification matrix check failed: ${manualResult.stderr.trim()}`);
  }

  const acceptedBaseline = policy.accepted_baseline ?? '';
  const baselineExists = /^[0-9a-f]{40}$/u.test(acceptedBaseline)
    && commandExitedSuccessfully(git(repoRoot, ['cat-file', '-e', `${acceptedBaseline}^{commit}`]));
  const baselineIsAncestor = baselineExists
    && commandExitedSuccessfully(git(repoRoot, ['merge-base', '--is-ancestor', acceptedBaseline, 'HEAD']));
  const headResult = git(repoRoot, ['rev-parse', 'HEAD']);
  const head = commandExitedSuccessfully(headResult) ? headResult.stdout.trim() : 'UNRESOLVED';
  const worktreePaths = currentWorktreePaths(repoRoot);
  const baselinePaths = baselineExists
    ? outputLines(git(repoRoot, ['diff', '--name-only', `${acceptedBaseline}..HEAD`]))
    : ['<accepted-baseline-unavailable>'];
  const changedPaths = [...new Set([...baselinePaths, ...worktreePaths])];

  const receiptRelative = policy.full_gate_receipt
    ?? 'docs/takeover/evidence/gate1-full-gate.json';
  const receiptPath = path.join(repoRoot, receiptRelative);
  const receipt = readJsonOrNull(receiptPath, diagnostics, 'Gate 1 full-gate receipt');
  const testedCommit = receipt?.tested_commit ?? '';
  const testedCommitExists = /^[0-9a-f]{40}$/u.test(testedCommit)
    && commandExitedSuccessfully(git(repoRoot, ['cat-file', '-e', `${testedCommit}^{commit}`]));
  const testedCommitIsAncestor = testedCommitExists
    && commandExitedSuccessfully(git(repoRoot, ['merge-base', '--is-ancestor', testedCommit, 'HEAD']));
  const postTestCommittedPaths = testedCommitExists
    ? outputLines(git(repoRoot, ['diff', '--name-only', `${testedCommit}..HEAD`]))
    : [];
  const gateReceipt = validateFullGateReceipt(receipt, {
    tested_commit_exists: testedCommitExists,
    tested_commit_is_ancestor: testedCommitIsAncestor,
    post_test_changed_paths: [...new Set([...postTestCommittedPaths, ...worktreePaths])],
  });

  const pilotPolicy = policy.pilot_program ?? {};
  const manifestPath = pilotPolicy.manifest_path
    ? path.join(repoRoot, pilotPolicy.manifest_path)
    : '';
  let manifest = null;
  if (manifestPath === '' || !fs.existsSync(manifestPath)) {
    diagnostics.push('Pilot scope manifest is absent.');
  } else {
    const manifestJson = readJsonOrNull(manifestPath, diagnostics, 'Pilot scope manifest');
    if (manifestJson !== null) {
      try {
        manifest = validatePilotScopeManifest(manifestJson, rows);
      } catch (error) {
        diagnostics.push(`pilot scope manifest: ${error.message}`);
      }
    }
  }

  const facts = {
    inventory: {
      tracker_valid: trackerValid,
      total: rows.length,
      unique: new Set(rows.map((row) => row.requirement_id)).size,
    },
    row_truth: deriveRowTruth(rows, policy.geo_boundary),
    evidence: deriveEvidenceFacts({
      rows,
      evidenceRows,
      exactTestMapValid: exactMapValid,
      branchText,
      missingDomainReceipts,
      claimsText,
      manualMatrixCurrent: commandExitedSuccessfully(manualResult),
    }),
    discrepancies: {
      prd_audit_current: prdAuditCurrent,
      all_findings_listed: prdAuditCurrent,
    },
    baseline_gate: {
      accepted_baseline_recorded: baselineExists,
      accepted_baseline_is_ancestor: baselineIsAncestor,
      full_gate_receipt_present: gateReceipt.present,
      full_gate_fresh: gateReceipt.fresh,
      full_gate_failed: gateReceipt.failed,
      tested_commit_is_ancestor: gateReceipt.tested_commit_is_ancestor,
      post_test_changes_are_evidence_only: gateReceipt.post_test_changes_are_evidence_only,
    },
    pilot_program: derivePilotProgram(rows, pilotPolicy, manifest),
    production_boundary: {
      changed_paths: classifyProductionChanges(changedPaths),
    },
  };

  return {
    facts,
    metadata: {
      generated_at: new Date().toISOString(),
      head,
      diagnostics,
    },
  };
}

function runCli() {
  const live = collectLiveGate1Facts();
  const report = evaluateGate1Exit(live.facts);
  if (process.argv.includes('--json')) {
    process.stdout.write(`${JSON.stringify({ ...live.metadata, ...report }, null, 2)}\n`);
  } else {
    process.stdout.write(renderGate1Report(report, live.metadata));
  }
  if (process.argv.includes('--check') && report.state !== 'PASS') process.exitCode = 1;
}

const invokedPath = process.argv[1] ? pathToFileURL(path.resolve(process.argv[1])).href : '';
if (invokedPath === import.meta.url) runCli();

function result(id, title, pass, detail) {
  return {
    id,
    title,
    state: pass ? 'PASS' : 'OPEN',
    detail,
  };
}

export function evaluateGate1Exit(facts) {
  const inventory = facts.inventory;
  const inventoryPass = inventory.tracker_valid
    && inventory.total === 931
    && inventory.unique === 931;

  const rowTruth = facts.row_truth;
  const allRowsAdjudicated = rowTruth.adjudicated === 931 && rowTruth.unadjudicated === 0;
  const approvedGeoBoundary = rowTruth.adjudicated === 879
    && rowTruth.unadjudicated === 52
    && rowTruth.geo_boundary_state === 'APPROVED'
    && rowTruth.geo_deferred_rows === 52
    && rowTruth.geo_rows_conform;
  const rowTruthPass = allRowsAdjudicated || approvedGeoBoundary;

  const evidence = facts.evidence;
  const evidencePass = evidence.exact_test_map_valid
    && evidence.claimed_test_rows === evidence.mapped_test_rows
    && evidence.unresolved_citations === 0
    && evidence.unresolved_branch_commits === 0
    && evidence.adjudicated_rows_without_manual_evidence === 0
    && evidence.adjudicated_rows_with_unverified_commits === 0
    && evidence.missing_domain_receipts.length === 0
    && evidence.claims_inventory_complete
    && evidence.manual_matrix_current;

  const discrepancies = facts.discrepancies;
  const discrepanciesPass = discrepancies.prd_audit_current
    && discrepancies.all_findings_listed;

  const baselineGate = facts.baseline_gate;
  const baselineGatePass = baselineGate.accepted_baseline_recorded
    && baselineGate.accepted_baseline_is_ancestor
    && baselineGate.full_gate_receipt_present
    && baselineGate.full_gate_fresh
    && baselineGate.full_gate_failed === 0
    && baselineGate.tested_commit_is_ancestor
    && baselineGate.post_test_changes_are_evidence_only;

  const pilot = facts.pilot_program;
  const pilotPass = pilot.manifest_state === 'APPROVED'
    && pilot.requirements_covered === 931
    && pilot.undecided_requirements === 0
    && pilot.blocker_program_approved
    && pilot.blockers_without_next_action === 0
    && pilot.blockers_without_dependency === 0
    && pilot.blockers_without_owner_decision === 0;

  const productionBoundary = facts.production_boundary;
  const productionPass = productionBoundary.changed_paths.length === 0;

  const criteria = [
    result(
      'G1-C1',
      'All 931 requirements are accounted for exactly once',
      inventoryPass,
      inventoryPass
        ? 'Tracker is valid with 931 rows and 931 unique requirement IDs.'
        : `Tracker valid=${inventory.tracker_valid}; rows=${inventory.total}; unique=${inventory.unique}.`,
    ),
    result(
      'G1-C2',
      'Every row distinguishes chapter status from reverified as-built status',
      rowTruthPass,
      allRowsAdjudicated
        ? 'All 931 rows carry a live-adjudicated as-built state.'
        : approvedGeoBoundary
          ? '879 rows are live-adjudicated and the exact 52-row conforming SB-GEO set is covered by the approved next-version boundary.'
          : `${rowTruth.unadjudicated} unadjudicated rows remain; GEO boundary is ${rowTruth.geo_boundary_state}, not approved for this exact conforming set of ${rowTruth.geo_deferred_rows}.`,
    ),
    result(
      'G1-C3',
      'Every claimed test, citation, branch commit, and manual item resolves to evidence',
      evidencePass,
      evidencePass
        ? `${evidence.mapped_test_rows} claimed proof rows resolve exactly; citation, branch, manual, commit, receipt, claim-register, and matrix checks have no gap.`
        : `Evidence gaps: citation=${evidence.unresolved_citations}; branch=${evidence.unresolved_branch_commits}; manual=${evidence.adjudicated_rows_without_manual_evidence}; unverified commits=${evidence.adjudicated_rows_with_unverified_commits}; missing receipts=${evidence.missing_domain_receipts.length}; exact map valid=${evidence.exact_test_map_valid}; mapped=${evidence.mapped_test_rows}/${evidence.claimed_test_rows}; claims complete=${evidence.claims_inventory_complete}; manual matrix current=${evidence.manual_matrix_current}.`,
    ),
    result(
      'G1-C4',
      'All internal PRD and index discrepancies are listed',
      discrepanciesPass,
      discrepanciesPass
        ? 'The byte-current PRD audit records every measured structural discrepancy without normalizing it.'
        : `PRD audit current=${discrepancies.prd_audit_current}; all findings listed=${discrepancies.all_findings_listed}.`,
    ),
    result(
      'G1-C5',
      'The current full gate result and accepted baseline are recorded',
      baselineGatePass,
      baselineGatePass
        ? 'Accepted baseline and tested commit are in current lineage; the fresh full gate has zero failures and later changes are evidence-only.'
        : `Baseline recorded=${baselineGate.accepted_baseline_recorded}; baseline ancestor=${baselineGate.accepted_baseline_is_ancestor}; full gate receipt=${baselineGate.full_gate_receipt_present}; full gate ${baselineGate.full_gate_fresh ? 'fresh' : 'stale'}; ${baselineGate.full_gate_failed} failed; tested commit ancestor=${baselineGate.tested_commit_is_ancestor}; post-test evidence-only=${baselineGate.post_test_changes_are_evidence_only}.`,
    ),
    result(
      'G1-C6',
      'The pilot-blocker program is executable and approved by Jauhar',
      pilotPass,
      pilotPass
        ? 'The approved manifest covers all 931 requirements, leaves none undecided, and every retained blocker has an action, dependency, and owner-decision boundary.'
        : `Manifest ${pilot.manifest_state}; covers ${pilot.requirements_covered}/931 requirements; ${pilot.undecided_requirements} undecided; blocker program approved=${pilot.blocker_program_approved}; missing actions=${pilot.blockers_without_next_action}; missing dependencies=${pilot.blockers_without_dependency}; missing owner decisions=${pilot.blockers_without_owner_decision}.`,
    ),
    result(
      'G1-C7',
      'Reconciliation changes no production behavior',
      productionPass,
      productionPass
        ? 'No production path differs from the accepted baseline.'
        : `Production paths changed during Gate 1: ${productionBoundary.changed_paths.join(', ')}.`,
    ),
  ];

  return {
    gate: 'G1',
    state: criteria.every((entry) => entry.state === 'PASS') ? 'PASS' : 'OPEN',
    criteria,
  };
}
