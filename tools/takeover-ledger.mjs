#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md');
const ledgerPath = path.join(repo, 'docs', 'takeover', 'requirements.csv');
const statusPath = path.join(repo, 'docs', 'takeover', 'STATUS.md');

export const LEDGER_COLUMNS = [
  'requirement_id',
  'chapter',
  'title',
  'original_priority',
  'chapter_status',
  'as_built_status',
  'release_disposition',
  'risk_class',
  'implementation_paths',
  'owned_tests',
  'test_class',
  'expected_value_source',
  'manual_evidence',
  'dependencies',
  'commit_state',
  'blocking_decision',
  'next_action',
  'last_reverified',
];

export const AS_BUILT_STATUSES = new Set([
  'UNADJUDICATED',
  'ABSENT',
  'PARTIAL',
  'PRESENT-OK',
  'PRESENT-DIVERGENT',
  'PRESENT-UNVERIFIED',
]);

export const RELEASE_DISPOSITIONS = new Set([
  'UNDECIDED',
  'PILOT-BLOCKER',
  'DEFERRED',
  'OUT',
]);

export const RISK_CLASSES = new Set([
  'UNCLASSIFIED',
  'SILENT-WRONGNESS',
  'DEGRADED-RESULT',
  'DATA-INTEGRITY',
  'DEPLOYMENT',
  'RECOVERY',
  'FIELD-EVIDENCE',
  'REQUESTED-CAPABILITY',
  'LATER',
]);

export const TEST_CLASSES = new Set([
  'MISSING-OR-UNCLASSIFIED',
  'CORRECTNESS',
  'CHARACTERIZATION',
  'OPTIONAL-PACKAGE-IGNORED',
  'SPEC-DIVERGENCE-IGNORED',
  'MISSING',
]);

export const COMMIT_STATES = new Set([
  'UNVERIFIED',
  'INTEGRATED',
  'CANDIDATE',
  'SUPERSEDED',
  'REJECTED',
  'UNIMPLEMENTED',
]);

function stripOuterCodeTicks(value) {
  const trimmed = value.trim();
  const match = /^`([^`]*)`$/u.exec(trimmed);
  return match ? match[1] : trimmed;
}

export function splitMarkdownRow(line) {
  if (typeof line !== 'string' || !line.trimStart().startsWith('|')) {
    throw new Error('markdown table row must start with |');
  }

  const cells = [];
  let field = '';
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    if (char === '|' && line[index - 1] !== '\\') {
      cells.push(field);
      field = '';
      continue;
    }
    field += char;
  }
  cells.push(field);

  if (cells[0].trim() === '') cells.shift();
  if (cells.at(-1)?.trim() === '') cells.pop();
  return cells.map(stripOuterCodeTicks);
}

function blankAdjudication() {
  return {
    as_built_status: 'UNADJUDICATED',
    release_disposition: 'UNDECIDED',
    risk_class: 'UNCLASSIFIED',
    implementation_paths: '',
    test_class: 'MISSING-OR-UNCLASSIFIED',
    expected_value_source: '',
    manual_evidence: '',
    dependencies: '',
    commit_state: 'UNVERIFIED',
    blocking_decision: '',
    next_action: 'LIVE-ADJUDICATION',
    last_reverified: '',
  };
}

export function parseConsolidatedRequirements(markdown) {
  const lines = String(markdown).split(/\r?\n/u);
  const headingIndex = lines.findIndex((line) => line.trim() === '## Consolidated requirements');
  if (headingIndex < 0) throw new Error('missing consolidated requirements heading');

  const rows = [];
  for (const line of lines.slice(headingIndex + 1)) {
    if (!/^\|\s*`SB-[A-Z]+-\d+`\s*\|/u.test(line)) continue;
    const cells = splitMarkdownRow(line);
    if (cells.length !== 6) {
      throw new Error(`requirement row has ${cells.length} columns instead of 6: ${line}`);
    }
    const [requirementId, title, originalPriority, chapterStatus, chapter, ownedTests] = cells;
    const values = {
      requirement_id: requirementId,
      chapter,
      title,
      original_priority: originalPriority,
      chapter_status: chapterStatus,
      ...blankAdjudication(),
      owned_tests: ownedTests,
    };
    rows.push(Object.fromEntries(LEDGER_COLUMNS.map((column) => [column, values[column]])));
  }
  if (rows.length === 0) throw new Error('consolidated requirements table contains no rows');
  return rows;
}

function encodeCsvField(value) {
  const text = String(value ?? '');
  if (!/[",\r\n]/u.test(text)) return text;
  return `"${text.replaceAll('"', '""')}"`;
}

export function renderCsv(rows) {
  if (!Array.isArray(rows) || rows.length === 0) return '';
  const columns = Object.keys(rows[0]);
  const lines = [columns.map(encodeCsvField).join(',')];
  for (const row of rows) {
    const rowColumns = Object.keys(row);
    if (rowColumns.length !== columns.length || rowColumns.some((column, index) => column !== columns[index])) {
      throw new Error('all CSV rows must have the same ordered columns');
    }
    lines.push(columns.map((column) => encodeCsvField(row[column])).join(','));
  }
  return `${lines.join('\n')}\n`;
}

export function parseCsv(text) {
  const records = [];
  let record = [];
  let field = '';
  let quoted = false;
  const input = String(text);

  for (let index = 0; index < input.length; index += 1) {
    const char = input[index];
    if (quoted) {
      if (char === '"' && input[index + 1] === '"') {
        field += '"';
        index += 1;
      } else if (char === '"') {
        quoted = false;
      } else {
        field += char;
      }
      continue;
    }

    if (char === '"') {
      if (field.length !== 0) throw new Error('a quoted CSV field must start with a quote');
      quoted = true;
    } else if (char === ',') {
      record.push(field);
      field = '';
    } else if (char === '\n' || char === '\r') {
      if (char === '\r' && input[index + 1] === '\n') index += 1;
      record.push(field);
      field = '';
      records.push(record);
      record = [];
    } else {
      field += char;
    }
  }

  if (quoted) throw new Error('unterminated quoted CSV field');
  if (field.length > 0 || record.length > 0) {
    record.push(field);
    records.push(record);
  }
  if (records.length === 0) return [];

  const [header, ...dataRows] = records;
  if (new Set(header).size !== header.length) throw new Error('CSV header contains duplicate columns');
  return dataRows.map((values, rowIndex) => {
    if (values.length !== header.length) {
      throw new Error(`CSV row ${rowIndex + 2} has ${values.length} fields instead of ${header.length}`);
    }
    return Object.fromEntries(header.map((column, columnIndex) => [column, values[columnIndex]]));
  });
}

function duplicateId(rows) {
  const seen = new Set();
  for (const row of rows) {
    if (seen.has(row.requirement_id)) return row.requirement_id;
    seen.add(row.requirement_id);
  }
  return null;
}

function assertVocabulary(name, value, allowed, requirementId) {
  if (!allowed.has(value)) {
    throw new Error(`${requirementId} has invalid ${name}: ${value}`);
  }
}

export function validateLedger(sourceRows, ledgerRows) {
  const duplicateLedgerId = duplicateId(ledgerRows);
  if (duplicateLedgerId) throw new Error(`duplicate requirement_id: ${duplicateLedgerId}`);
  const duplicateSourceId = duplicateId(sourceRows);
  if (duplicateSourceId) throw new Error(`duplicate source requirement_id: ${duplicateSourceId}`);

  if (ledgerRows.length === 0) throw new Error('ledger contains no rows');
  const columns = Object.keys(ledgerRows[0]);
  if (columns.length !== LEDGER_COLUMNS.length || columns.some((column, index) => column !== LEDGER_COLUMNS[index])) {
    throw new Error(`ledger columns must be: ${LEDGER_COLUMNS.join(',')}`);
  }
  for (const row of ledgerRows) {
    const rowColumns = Object.keys(row);
    if (rowColumns.length !== LEDGER_COLUMNS.length || rowColumns.some((column, index) => column !== LEDGER_COLUMNS[index])) {
      throw new Error(`${row.requirement_id} has inconsistent ledger columns`);
    }
  }

  const sourceById = new Map(sourceRows.map((row) => [row.requirement_id, row]));
  const ledgerById = new Map(ledgerRows.map((row) => [row.requirement_id, row]));
  const missing = [...sourceById.keys()].filter((id) => !ledgerById.has(id));
  const extra = [...ledgerById.keys()].filter((id) => !sourceById.has(id));
  if (missing.length > 0 || extra.length > 0) {
    throw new Error(`source/ledger id drift; missing=${missing.join(';')} extra=${extra.join(';')}`);
  }

  const sourceOwnedFields = [
    'chapter',
    'title',
    'original_priority',
    'chapter_status',
    'owned_tests',
  ];
  for (const row of ledgerRows) {
    const source = sourceById.get(row.requirement_id);
    for (const fieldName of sourceOwnedFields) {
      if (row[fieldName] !== source[fieldName]) {
        throw new Error(`${row.requirement_id} source-owned field drift: ${fieldName}`);
      }
    }
    assertVocabulary('as_built_status', row.as_built_status, AS_BUILT_STATUSES, row.requirement_id);
    assertVocabulary('release_disposition', row.release_disposition, RELEASE_DISPOSITIONS, row.requirement_id);
    assertVocabulary('risk_class', row.risk_class, RISK_CLASSES, row.requirement_id);
    assertVocabulary('test_class', row.test_class, TEST_CLASSES, row.requirement_id);
    assertVocabulary('commit_state', row.commit_state, COMMIT_STATES, row.requirement_id);

    if (row.as_built_status === 'UNADJUDICATED' && row.last_reverified !== '') {
      throw new Error(`${row.requirement_id} is unadjudicated but carries last_reverified`);
    }
    if (row.as_built_status !== 'UNADJUDICATED' && row.last_reverified === '') {
      throw new Error(`${row.requirement_id} is adjudicated without last_reverified`);
    }
  }
  return true;
}

export function validateStatus(markdown) {
  const requiredFields = [
    'Current gate',
    'Active increment',
    'Accepted baseline',
    'Automated gate',
    'Pilot field evidence',
    'Open blockers',
    'Next increment',
  ];
  const lines = String(markdown).split(/\r?\n/u);
  for (const fieldName of requiredFields) {
    const matches = lines.filter((line) => line.startsWith(`- ${fieldName}:`));
    if (matches.length !== 1) {
      throw new Error(`dashboard must contain exactly one ${fieldName} field`);
    }
  }
  return true;
}

export function summarizeLedger(rows) {
  return {
    total: rows.length,
    adjudicated: rows.filter((row) => row.as_built_status !== 'UNADJUDICATED').length,
    unadjudicated: rows.filter((row) => row.as_built_status === 'UNADJUDICATED').length,
    pilot_blockers: rows.filter((row) => row.release_disposition === 'PILOT-BLOCKER').length,
  };
}

function readSourceRows() {
  return parseConsolidatedRequirements(fs.readFileSync(sourcePath, 'utf8'));
}

function readLedgerRows() {
  if (!fs.existsSync(ledgerPath)) throw new Error(`missing ledger: ${ledgerPath}`);
  return parseCsv(fs.readFileSync(ledgerPath, 'utf8'));
}

export function initializeLedger(rows, targetPath = ledgerPath) {
  if (fs.existsSync(targetPath)) {
    throw new Error(`${path.relative(repo, targetPath)} already exists; initialization refuses to overwrite it`);
  }
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  fs.writeFileSync(targetPath, renderCsv(rows), 'utf8');
}

function checkTracker() {
  validateLedger(readSourceRows(), readLedgerRows());
  if (!fs.existsSync(statusPath)) throw new Error(`missing dashboard: ${statusPath}`);
  validateStatus(fs.readFileSync(statusPath, 'utf8'));
}

function parseArgs(argv) {
  const supported = new Set(['--initialize', '--check', '--summary-json']);
  if (argv.length !== 1 || !supported.has(argv[0])) {
    throw new Error('use exactly one of --initialize, --check or --summary-json');
  }
  return argv[0];
}

function main() {
  const mode = parseArgs(process.argv.slice(2));
  if (mode === '--initialize') {
    initializeLedger(readSourceRows());
    return;
  }
  if (mode === '--check') {
    checkTracker();
    return;
  }
  process.stdout.write(`${JSON.stringify(summarizeLedger(readLedgerRows()), null, 2)}\n`);
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`takeover ledger: ${error instanceof Error ? error.message : error}`);
    process.exitCode = 1;
  }
}
