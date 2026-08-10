import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  LEDGER_COLUMNS,
  initializeLedger,
  parseConsolidatedRequirements,
  parseCsv,
  renderCsv,
  splitMarkdownRow,
  summarizeLedger,
  validateLedger,
  validateStatus,
} from './takeover-ledger.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

async function loadAuditApi() {
  const api = await import('./takeover-ledger.mjs');
  for (const name of ['auditPrd', 'checkTracker', 'parseRollups', 'renderPrdAudit']) {
    assert.equal(typeof api[name], 'function', `missing audit export ${name}`);
  }
  return api;
}

function withTemporaryPrd(run) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'sandibumi-prd-audit-'));
  try {
    return run(directory);
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
}

test('an escaped pipe inside a requirement title does not create an extra column', () => {
  const row = '| `SB-SHR-026` | Pc uses `\\|cos theta\\|` consistently | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T28` |';
  assert.deepEqual(splitMarkdownRow(row), [
    'SB-SHR-026',
    'Pc uses `\\|cos theta\\|` consistently',
    'P1',
    'ABSENT',
    '15_sat-height-rocktyping.md',
    'SB-SHR-T28',
  ]);
});

test('quoted commas quotes and line breaks survive an RFC 4180 round trip', () => {
  const rows = [
    {
      requirement_id: 'SB-CORE-001',
      blocking_decision: 'source says "refuse", owner decides\nrelease scope',
    },
  ];
  assert.deepEqual(parseCsv(renderCsv(rows)), rows);
});

test('a duplicate requirement id makes the ledger invalid', () => {
  const source = [{ requirement_id: 'SB-CORE-001' }];
  const ledger = [
    { requirement_id: 'SB-CORE-001' },
    { requirement_id: 'SB-CORE-001' },
  ];
  assert.throws(
    () => validateLedger(source, ledger),
    /duplicate requirement_id: SB-CORE-001/u,
  );
});

test('raw chapter status is preserved while an unadjudicated row remains explicitly unadjudicated', () => {
  const row = parseConsolidatedRequirements([
    '## Consolidated requirements',
    '',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-CORE-030` | Portfolio target | `P1` | `UNMEASURED` | `04_CORE_REQUIREMENTS.md` |  |',
  ].join('\n'))[0];

  assert.equal(row.chapter_status, 'UNMEASURED');
  assert.equal(row.as_built_status, 'UNADJUDICATED');
});

test('the checked out consolidated index contains exactly 931 unique requirements', () => {
  const markdown = fs.readFileSync(
    path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md'),
    'utf8',
  );
  const rows = parseConsolidatedRequirements(markdown);

  assert.equal(rows.length, 931);
  assert.equal(new Set(rows.map((row) => row.requirement_id)).size, 931);
});

test('source rows are initialized in authoritative ledger column order', () => {
  const markdown = fs.readFileSync(
    path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md'),
    'utf8',
  );
  const [first] = parseConsolidatedRequirements(markdown);

  assert.deepEqual(Object.keys(first), LEDGER_COLUMNS);
});

test('initialization refuses to overwrite an existing ledger', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'sandibumi-takeover-ledger-'));
  const target = path.join(directory, 'requirements.csv');
  fs.writeFileSync(target, 'existing adjudication\n', 'utf8');

  try {
    assert.throws(
      () => initializeLedger([{ requirement_id: 'SB-CORE-001' }], target),
      /already exists; initialization refuses to overwrite it/u,
    );
    assert.equal(fs.readFileSync(target, 'utf8'), 'existing adjudication\n');
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});

test('the dashboard names one active increment and separates automated from field evidence', () => {
  const status = [
    '# SandiBumi takeover status',
    '',
    '- Current gate: `G1`',
    '- Active increment: `G1-I001`',
    '- Accepted baseline: `b272d1951bd627fa75a0966cd1a94820ec2c3f22`',
    '- Automated gate: `NOT-RUN`',
    '- Pilot field evidence: `OPEN`',
    '- Open blockers: `UNMEASURED`',
    '- Next increment: `G1-I002`',
  ].join('\n');

  assert.doesNotThrow(() => validateStatus(status));
});

test('summary counts do not convert undecided rows into completed work', () => {
  const summary = summarizeLedger([
    {
      release_disposition: 'UNDECIDED',
      as_built_status: 'UNADJUDICATED',
    },
    {
      release_disposition: 'PILOT-BLOCKER',
      as_built_status: 'PRESENT-DIVERGENT',
    },
  ]);

  assert.deepEqual(summary, {
    total: 2,
    adjudicated: 1,
    unadjudicated: 1,
    pilot_blockers: 1,
  });
});

test('rollup status totals are compared with the consolidated rows rather than trusted', async () => {
  // CORRECTNESS — CONTRACT.md §3 makes each requirement row authoritative for its stated status.
  const { auditPrd } = await loadAuditApi();
  const indexMarkdown = [
    '## Roll-ups',
    '',
    '**Total: 2 requirements.**',
    '',
    '### By priority',
    '| Priority | Count |',
    '|---|---:|',
    '| `P0` | 2 |',
    '| **Total** | **2** |',
    '',
    '### By status',
    '| Status | Count |',
    '|---|---:|',
    '| `PRESENT-OK` | 1 |',
    '| `PRESENT-DIVERGENT` | 1 |',
    '| **Total** | **2** |',
    '',
    '## Consolidated requirements',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-TST-001` | First | `P0` | `PRESENT-OK` | `10_test.md` | `SB-TST-T01` |',
    '| `SB-TST-002` | Second | `P0` | `PRESENT-OK` | `10_test.md` | `SB-TST-T02` |',
  ].join('\n');

  const audit = withTemporaryPrd((prdDirectory) => {
    fs.writeFileSync(path.join(prdDirectory, '10_test.md'), '# Test\n', 'utf8');
    return auditPrd({
      indexMarkdown,
      indexPath: path.join(prdDirectory, '91_REQUIREMENTS_INDEX.md'),
      prdDirectory,
      documentMapMarkdown: '',
      resumeMarkdown: '',
      spinePendingMarkdown: '',
    });
  });

  assert.deepEqual(audit.rollups.status.mismatches, [
    { value: 'PRESENT-DIVERGENT', declared: 1, derived: 0 },
    { value: 'PRESENT-OK', declared: 1, derived: 2 },
  ]);
});

test('a chapter named by the document map but absent from disk is reported', async () => {
  // CORRECTNESS — 00_INDEX.md §0.2 says its document map names promised PRD artifacts.
  const { auditPrd } = await loadAuditApi();
  const indexMarkdown = [
    '## Consolidated requirements',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-TST-001` | First | `P0` | `PRESENT-OK` | `10_test.md` | `SB-TST-T01` |',
  ].join('\n');
  const documentMapMarkdown = [
    '### 0.2 Document map',
    '| File | Holds |',
    '|---|---|',
    '| `00_INDEX.md` | map |',
    '| `90_GAP_ANALYSIS.md` | promised roll-up |',
  ].join('\n');

  const audit = withTemporaryPrd((prdDirectory) => {
    fs.writeFileSync(path.join(prdDirectory, '00_INDEX.md'), documentMapMarkdown, 'utf8');
    fs.writeFileSync(path.join(prdDirectory, '10_test.md'), '# Test\n', 'utf8');
    return auditPrd({
      indexMarkdown,
      indexPath: path.join(prdDirectory, '91_REQUIREMENTS_INDEX.md'),
      prdDirectory,
      documentMapMarkdown,
      resumeMarkdown: '',
      spinePendingMarkdown: '',
    });
  });

  assert.deepEqual(audit.missingDocumentMapArtifacts, ['90_GAP_ANALYSIS.md']);
});

test('a resume chapter count that disagrees with files on disk is reported', async () => {
  // CORRECTNESS — RESUME.md states a written/total count that can be checked against domain files.
  const { auditPrd } = await loadAuditApi();
  const indexMarkdown = [
    '## Consolidated requirements',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-TST-001` | First | `P0` | `PRESENT-OK` | `10_one.md` | `SB-TST-T01` |',
    '| `SB-TST-002` | Second | `P0` | `PRESENT-OK` | `11_two.md` | `SB-TST-T02` |',
  ].join('\n');

  const audit = withTemporaryPrd((prdDirectory) => {
    fs.writeFileSync(path.join(prdDirectory, '10_one.md'), '# One\n', 'utf8');
    fs.writeFileSync(path.join(prdDirectory, '11_two.md'), '# Two\n', 'utf8');
    return auditPrd({
      indexMarkdown,
      indexPath: path.join(prdDirectory, '91_REQUIREMENTS_INDEX.md'),
      prdDirectory,
      documentMapMarkdown: '',
      resumeMarkdown: '### Chapters — 1 of 2 written\n',
      spinePendingMarkdown: '',
    });
  });

  assert.deepEqual(audit.resumeChapterCount, {
    claimedWritten: 1,
    claimedTotal: 2,
    filesOnDisk: 2,
    status: 'INCONSISTENT',
  });
});

test('blank and out of vocabulary chapter statuses remain visible findings', async () => {
  // CORRECTNESS — CONTRACT.md §3 defines exactly five valid chapter statuses.
  const { auditPrd } = await loadAuditApi();
  const indexMarkdown = [
    '## Consolidated requirements',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-TST-001` | Blank | `P0` |  | `10_test.md` | `SB-TST-T01` |',
    '| `SB-TST-002` | Invalid | `P0` | `UNMEASURED` | `10_test.md` | `SB-TST-T02` |',
  ].join('\n');

  const audit = withTemporaryPrd((prdDirectory) => {
    fs.writeFileSync(path.join(prdDirectory, '10_test.md'), '# Test\n', 'utf8');
    return auditPrd({
      indexMarkdown,
      indexPath: path.join(prdDirectory, '91_REQUIREMENTS_INDEX.md'),
      prdDirectory,
      documentMapMarkdown: '',
      resumeMarkdown: '',
      spinePendingMarkdown: '',
    });
  });

  assert.deepEqual(audit.blankStatuses, ['SB-TST-001']);
  assert.deepEqual(audit.invalidStatuses, [
    { requirementId: 'SB-TST-002', value: 'UNMEASURED' },
  ]);
});

test('every chapter reference in the consolidated index resolves to one file', async () => {
  // CORRECTNESS — checked-out 91_REQUIREMENTS_INDEX.md is the source of the chapter references.
  const { auditPrd } = await loadAuditApi();
  const indexPath = path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md');
  const audit = auditPrd({
    indexMarkdown: fs.readFileSync(indexPath, 'utf8'),
    indexPath,
    prdDirectory: path.dirname(indexPath),
  });

  assert.ok(audit.chapterReferences.length > 0);
  assert.equal(new Set(audit.chapterReferences.map((entry) => entry.chapter)).size, audit.chapterReferences.length);
  assert.deepEqual(audit.chapterReferences.filter((entry) => entry.matches !== 1), []);
});

test('the generated PRD integrity report is byte current', async () => {
  // CORRECTNESS — generated evidence must equal a fresh render of the checked-out PRD facts.
  const { auditPrd, renderPrdAudit } = await loadAuditApi();
  const indexPath = path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md');
  const reportPath = path.join(repo, 'docs', 'takeover', 'evidence', 'prd-integrity.md');
  const expected = renderPrdAudit(auditPrd({
    indexMarkdown: fs.readFileSync(indexPath, 'utf8'),
    indexPath,
    prdDirectory: path.dirname(indexPath),
  }));

  assert.equal(fs.readFileSync(reportPath, 'utf8'), expected);
});

test('the complete tracker check rejects a stale PRD audit once the report exists', async () => {
  // CORRECTNESS — Gate 1 design makes an existing PRD audit a byte-current tracker dependency.
  const { auditPrd, checkTracker, renderPrdAudit } = await loadAuditApi();
  const indexPath = path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md');

  withTemporaryPrd((directory) => {
    const reportPath = path.join(directory, 'prd-integrity.md');
    fs.writeFileSync(reportPath, renderPrdAudit(auditPrd({
      indexMarkdown: fs.readFileSync(indexPath, 'utf8'),
      indexPath,
      prdDirectory: path.dirname(indexPath),
    })), 'utf8');
    const options = {
      sourcePath: indexPath,
      ledgerPath: path.join(repo, 'docs', 'takeover', 'requirements.csv'),
      statusPath: path.join(repo, 'docs', 'takeover', 'STATUS.md'),
      prdAuditPath: reportPath,
      prdDirectory: path.dirname(indexPath),
    };

    assert.doesNotThrow(() => checkTracker(options));
    fs.appendFileSync(reportPath, 'stale\n', 'utf8');
    assert.throws(() => checkTracker(options), /PRD integrity report is stale/u);
  });
});
