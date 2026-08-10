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
