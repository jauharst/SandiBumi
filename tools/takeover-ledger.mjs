#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md');
const prdDirectory = path.dirname(sourcePath);
const ledgerPath = path.join(repo, 'docs', 'takeover', 'requirements.csv');
const statusPath = path.join(repo, 'docs', 'takeover', 'STATUS.md');
const prdAuditPath = path.join(repo, 'docs', 'takeover', 'evidence', 'prd-integrity.md');
const testEvidencePath = path.join(repo, 'docs', 'takeover', 'test-evidence.csv');

const CHAPTER_STATUSES = new Set([
  'ABSENT',
  'PARTIAL',
  'PRESENT-OK',
  'PRESENT-DIVERGENT',
  'PRESENT-UNVERIFIED',
]);

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

export const CLAIMED_TEST_CLASSES = new Set([
  'CORRECTNESS',
  'CHARACTERIZATION',
  'OPTIONAL-PACKAGE-IGNORED',
  'SPEC-DIVERGENCE-IGNORED',
]);

export const TEST_EVIDENCE_COLUMNS = [
  'requirement_id',
  'test_class',
  'test_path',
  'test_name',
];

const TEST_SOURCE_EXTENSIONS = new Set(['.js', '.mjs', '.rs', '.ts', '.tsx']);
const TEST_DISCOVERY_IGNORES = new Set(['.git', 'dist', 'node_modules', 'target']);

function repositoryPath(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/');
}

function sourceFilesUnder(directory) {
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (TEST_DISCOVERY_IGNORES.has(entry.name)) continue;
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...sourceFilesUnder(entryPath));
    } else if (entry.isFile() && TEST_SOURCE_EXTENSIONS.has(path.extname(entry.name))) {
      files.push(entryPath);
    }
  }
  return files.sort();
}

function decodeSingleQuotedJavascript(value) {
  return value.replace(/\\(['\\])/gu, '$1');
}

export function discoverExecutableTests(root = repo) {
  const catalog = new Map();
  for (const filePath of sourceFilesUnder(root)) {
    const relativePath = repositoryPath(root, filePath);
    const source = fs.readFileSync(filePath, 'utf8');
    if (path.extname(filePath) === '.rs') {
      const rustTest = /((?:^[ \t]*#\[[^\]\r\n]+\][ \t]*\r?\n)+)[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gmu;
      for (const match of source.matchAll(rustTest)) {
        const attributes = match[1];
        if (!/#\[(?:[A-Za-z_][A-Za-z0-9_]*::)?test(?:\]|\()/u.test(attributes)) continue;
        const key = `${relativePath}::${match[2]}`;
        catalog.set(key, { ignored: /#\[ignore(?:\s*=|\])/u.test(attributes) });
      }
      continue;
    }

    const javascriptTest = /\b(?:test|it)(?:\.(skip|todo))?\s*\(\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)")/gu;
    for (const match of source.matchAll(javascriptTest)) {
      const name = match[2] === undefined
        ? JSON.parse(`"${match[3]}"`)
        : decodeSingleQuotedJavascript(match[2]);
      catalog.set(`${relativePath}::${name}`, { ignored: match[1] !== undefined });
    }
  }
  return catalog;
}

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

export function validateTestEvidence({ ledgerRows, evidenceRows, executableTests }) {
  const ledgerById = new Map(ledgerRows.map((row) => [row.requirement_id, row]));
  const evidenceSignatures = new Set();
  for (const evidence of evidenceRows) {
    const ledger = ledgerById.get(evidence.requirement_id);
    if (!ledger) {
      throw new Error(`${evidence.requirement_id} evidence has no ledger requirement`);
    }
    if (evidence.test_class !== ledger.test_class) {
      throw new Error(
        `${evidence.requirement_id} evidence class ${evidence.test_class}`
        + ` does not match ledger class ${ledger.test_class}`,
      );
    }
    if (!CLAIMED_TEST_CLASSES.has(ledger.test_class)) {
      throw new Error(
        `${evidence.requirement_id} has executable evidence while ledger class is ${ledger.test_class}`,
      );
    }
    const key = `${evidence.test_path}::${evidence.test_name}`;
    const signature = `${evidence.requirement_id}::${evidence.test_class}::${key}`;
    if (evidenceSignatures.has(signature)) {
      throw new Error(`duplicate exact test evidence for ${evidence.requirement_id}: ${key}`);
    }
    evidenceSignatures.add(signature);
    const executable = executableTests.get(key);
    if (!executable) {
      throw new Error(
        `${evidence.requirement_id} evidence does not resolve to executable test ${key}`,
      );
    }
    if (ledger.test_class.endsWith('-IGNORED') && !executable.ignored) {
      throw new Error(
        `${evidence.requirement_id} claims ${ledger.test_class} but ${key} is not ignored`,
      );
    }
    if (
      (ledger.test_class === 'CORRECTNESS' || ledger.test_class === 'CHARACTERIZATION')
      && executable.ignored
    ) {
      throw new Error(
        `${evidence.requirement_id} claims ${ledger.test_class} but ${key} is ignored`,
      );
    }
  }
  for (const row of ledgerRows) {
    if (!CLAIMED_TEST_CLASSES.has(row.test_class)) continue;
    if (!evidenceRows.some((evidence) => evidence.requirement_id === row.requirement_id)) {
      throw new Error(
        `${row.requirement_id} claims ${row.test_class} but has no exact executable test evidence`,
      );
    }
  }
  return true;
}

export function parseTestEvidence(text) {
  const rows = parseCsv(text);
  const columns = rows.length > 0 ? Object.keys(rows[0]) : [];
  if (
    columns.length !== TEST_EVIDENCE_COLUMNS.length
    || columns.some((column, index) => column !== TEST_EVIDENCE_COLUMNS[index])
  ) {
    throw new Error(
      `exact test evidence header must be ${TEST_EVIDENCE_COLUMNS.join(',')}`,
    );
  }
  return rows;
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

function countBy(rows, fieldName) {
  const counts = {};
  for (const row of rows) {
    const value = row[fieldName] || '';
    counts[value] = (counts[value] || 0) + 1;
  }
  return counts;
}

function normalizeRollupLabel(value) {
  const codeMatch = /\x60([^\x60]+)\x60/u.exec(value);
  if (codeMatch) return codeMatch[1];
  const plain = value.replaceAll('*', '').trim();
  if (plain === 'Total') return null;
  if (plain === 'Not stated by chapter') return '';
  return plain;
}

function parseRollupTable(lines, heading) {
  const headingIndex = lines.findIndex((line) => line.trim() === heading);
  if (headingIndex < 0) return {};
  const counts = {};
  let tableStarted = false;
  for (const line of lines.slice(headingIndex + 1)) {
    if (!line.trimStart().startsWith('|')) {
      if (tableStarted) break;
      continue;
    }
    tableStarted = true;
    const cells = splitMarkdownRow(line);
    if (cells.length !== 2 || /^[-: ]+$/u.test(cells[0])) continue;
    const label = normalizeRollupLabel(cells[0]);
    if (label === null || label === 'Priority' || label === 'Status') continue;
    const count = Number(cells[1].replace(/[^0-9-]/gu, ''));
    if (!Number.isInteger(count)) throw new Error('invalid roll-up count for ' + cells[0]);
    counts[label] = count;
  }
  return counts;
}

export function parseRollups(markdown) {
  const text = String(markdown);
  const totalMatch = /\*\*Total:\s*([0-9,]+) requirements\.\*\*/u.exec(text);
  return {
    total: totalMatch ? Number(totalMatch[1].replaceAll(',', '')) : null,
    priority: parseRollupTable(text.split(/\r?\n/u), '### By priority'),
    status: parseRollupTable(text.split(/\r?\n/u), '### By status'),
  };
}

function compareCounts(declared, derived) {
  const values = [...new Set([...Object.keys(declared), ...Object.keys(derived)])].sort();
  return values
    .filter((value) => (declared[value] || 0) !== (derived[value] || 0))
    .map((value) => ({
      value,
      declared: declared[value] || 0,
      derived: derived[value] || 0,
    }));
}

function readOptional(filePath) {
  return fs.existsSync(filePath) ? fs.readFileSync(filePath, 'utf8') : '';
}

function parseDocumentMap(markdown) {
  const start = String(markdown).split('### 0.2 Document map')[1] || '';
  const section = start.split('### 0.3 Reading routes')[0];
  return [...section.matchAll(/\x60([^\x60]+\.md)\x60/gu)]
    .map((match) => match[1])
    .filter((file, index, files) => files.indexOf(file) === index);
}

function parseSpineItems(markdown) {
  const text = String(markdown);
  const matches = [...text.matchAll(/^## (SP-[0-9]+) — ([^\r\n]+)/gmu)];
  return matches.map((match, index) => {
    const end = matches[index + 1]?.index ?? text.length;
    const block = text.slice(match.index, end);
    const closed = /\b(?:CLOSED|RESOLVED)\b/iu.test(block);
    return {
      id: match[1],
      title: match[2],
      state: closed ? 'CLOSED-AS-RECORDED' : 'OPEN',
    };
  });
}

function domainChapterFiles(files) {
  return files.filter((file) => /^(?:1[0-9]|2[0-7])_.+\.md$/u.test(file)).sort();
}

export function auditPrd({
  indexMarkdown,
  indexPath,
  prdDirectory: auditDirectory,
  documentMapMarkdown,
  resumeMarkdown,
  spinePendingMarkdown,
}) {
  const rows = parseConsolidatedRequirements(indexMarkdown);
  const files = fs.readdirSync(auditDirectory, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
  const fileSet = new Set(files);
  const rollups = parseRollups(indexMarkdown);
  const derivedPriority = countBy(rows, 'original_priority');
  const derivedStatus = countBy(rows, 'chapter_status');
  const priorityMismatches = compareCounts(rollups.priority, derivedPriority);
  const statusMismatches = compareCounts(rollups.status, derivedStatus);
  const chapterCounts = countBy(rows, 'chapter');
  const chapterReferences = Object.keys(chapterCounts).sort().map((chapter) => ({
    chapter,
    requirementCount: chapterCounts[chapter],
    matches: files.filter((file) => file === chapter).length,
  }));
  const domainsOnDisk = domainChapterFiles(files);
  const representedDomains = new Set(rows.map((row) => row.chapter)
    .filter((chapter) => /^(?:1[0-9]|2[0-7])_.+\.md$/u.test(chapter)));
  const unrepresentedDomainFiles = domainsOnDisk.filter((file) => !representedDomains.has(file));
  const mapMarkdown = documentMapMarkdown
    ?? readOptional(path.join(auditDirectory, '00_INDEX.md'));
  const documentMapArtifacts = parseDocumentMap(mapMarkdown).map((file) => ({
    file,
    present: file.includes('/')
      ? fs.existsSync(path.resolve(auditDirectory, file))
      : fileSet.has(file),
  }));
  const missingDocumentMapArtifacts = documentMapArtifacts
    .filter((entry) => !entry.present)
    .map((entry) => entry.file);
  const resume = resumeMarkdown ?? readOptional(path.join(auditDirectory, 'RESUME.md'));
  const resumeMatch = /### Chapters\s+[—-]\s+([0-9]+) of ([0-9]+) written/u.exec(resume);
  const resumeChapterCount = resumeMatch
    ? {
        claimedWritten: Number(resumeMatch[1]),
        claimedTotal: Number(resumeMatch[2]),
        filesOnDisk: domainsOnDisk.length,
        status: Number(resumeMatch[1]) === domainsOnDisk.length
          && Number(resumeMatch[2]) === domainsOnDisk.length
          ? 'CLOSED-AS-RECORDED'
          : 'INCONSISTENT',
      }
    : null;
  const pending = spinePendingMarkdown
    ?? readOptional(path.join(auditDirectory, '_SPINE_PENDING.md'));
  const blankPriorities = rows.filter((row) => row.original_priority === '')
    .map((row) => row.requirement_id);
  const blankStatuses = rows.filter((row) => row.chapter_status === '')
    .map((row) => row.requirement_id);
  const invalidStatuses = rows
    .filter((row) => row.chapter_status !== '' && !CHAPTER_STATUSES.has(row.chapter_status))
    .map((row) => ({ requirementId: row.requirement_id, value: row.chapter_status }));
  const withoutOwnedTests = rows.filter((row) => row.owned_tests === '')
    .map((row) => row.requirement_id);
  const totalMismatch = rollups.total !== null && rollups.total !== rows.length;

  return {
    indexPath,
    consolidatedRowCount: rows.length,
    uniqueIdCount: new Set(rows.map((row) => row.requirement_id)).size,
    rollups: {
      declaredTotal: rollups.total,
      totalMismatch,
      priority: {
        declared: rollups.priority,
        derived: derivedPriority,
        mismatches: priorityMismatches,
      },
      status: {
        declared: rollups.status,
        derived: derivedStatus,
        mismatches: statusMismatches,
      },
    },
    blankPriorities,
    blankStatuses,
    invalidStatuses,
    withoutOwnedTests,
    chapterReferences,
    domainFilesOnDisk: domainsOnDisk,
    unrepresentedDomainFiles,
    documentMapArtifacts,
    missingDocumentMapArtifacts,
    resumeChapterCount,
    spineItems: parseSpineItems(pending),
    summary: {
      consolidatedRequirements: rows.length,
      rollupMismatches: priorityMismatches.length + statusMismatches.length + (totalMismatch ? 1 : 0),
      blankPriorities: blankPriorities.length,
      blankStatuses: blankStatuses.length,
      invalidStatuses: invalidStatuses.length,
      requirementsWithoutOwnedTest: withoutOwnedTests.length,
      missingPromisedArtifacts: missingDocumentMapArtifacts.length,
      staleResumeClaims: resumeChapterCount?.status === 'INCONSISTENT' ? 1 : 0,
    },
  };
}

function code(value) {
  const tick = String.fromCharCode(96);
  return tick + String(value) + tick;
}

function displayRollupValue(value) {
  return value === '' ? '(blank)' : value;
}

function idList(values) {
  return values.length === 0 ? 'None.' : values.map(code).join(', ') + '.';
}

export function renderPrdAudit(audit) {
  const lines = [
    '# Gate 1 PRD structural-integrity audit',
    '',
    'Generated from ' + code(path.basename(audit.indexPath)) + ' and the checked-out ' + code('docs/PRD_v2') + ' directory.',
    'A recorded discrepancy remains open; generation does not resolve or amend its source.',
    '',
    '## Consolidated requirements',
    '',
    '- Rows: ' + code(audit.consolidatedRowCount) + '.',
    '- Unique IDs: ' + code(audit.uniqueIdCount) + '.',
    '- Declared roll-up total: ' + code(audit.rollups.declaredTotal ?? 'ABSENT') + '.',
    '- Total status: ' + code(audit.rollups.totalMismatch ? 'INCONSISTENT' : 'CLOSED-AS-RECORDED') + '.',
    '',
    '## Roll-up comparisons',
    '',
    '| Dimension | Value | Declared | Derived | State |',
    '|---|---|---:|---:|---|',
  ];
  const mismatchRows = [
    ...audit.rollups.priority.mismatches.map((entry) => ({ dimension: 'Priority', ...entry })),
    ...audit.rollups.status.mismatches.map((entry) => ({ dimension: 'Status', ...entry })),
  ];
  if (mismatchRows.length === 0) {
    lines.push('| All | — | — | — | ' + code('CLOSED-AS-RECORDED') + ' |');
  } else {
    for (const entry of mismatchRows) {
      lines.push('| ' + entry.dimension + ' | ' + code(displayRollupValue(entry.value))
        + ' | ' + entry.declared + ' | ' + entry.derived + ' | ' + code('INCONSISTENT') + ' |');
    }
  }
  lines.push(
    '',
    '## Requirement-shape findings',
    '',
    '- Blank priorities (' + audit.blankPriorities.length + '): ' + idList(audit.blankPriorities),
    '- Blank statuses (' + audit.blankStatuses.length + '): ' + idList(audit.blankStatuses),
    '- Contract-invalid statuses (' + audit.invalidStatuses.length + '): '
      + (audit.invalidStatuses.length === 0
        ? 'None.'
        : audit.invalidStatuses.map((entry) => code(entry.requirementId) + ' = ' + code(entry.value)).join(', ') + '.'),
    '- Requirements without an owned acceptance-test ID (' + audit.withoutOwnedTests.length + '): '
      + idList(audit.withoutOwnedTests),
    '',
    '## Chapter references',
    '',
    '| Chapter | Requirements | Files resolved | State |',
    '|---|---:|---:|---|',
  );
  for (const entry of audit.chapterReferences) {
    lines.push('| ' + code(entry.chapter) + ' | ' + entry.requirementCount + ' | ' + entry.matches
      + ' | ' + code(entry.matches === 1 ? 'CLOSED-AS-RECORDED' : 'OPEN') + ' |');
  }
  lines.push(
    '',
    '- Domain chapter files on disk: ' + code(audit.domainFilesOnDisk.length) + '.',
    '- Domain chapter files not represented by consolidated rows: ' + idList(audit.unrepresentedDomainFiles),
    '',
    '## Document-map artifacts',
    '',
    '| Artifact | Present | State |',
    '|---|---|---|',
  );
  for (const entry of audit.documentMapArtifacts) {
    lines.push('| ' + code(entry.file) + ' | ' + (entry.present ? 'yes' : 'no') + ' | '
      + code(entry.present ? 'CLOSED-AS-RECORDED' : 'OPEN') + ' |');
  }
  lines.push(
    '',
    'Missing promised artifacts: ' + idList(audit.missingDocumentMapArtifacts),
    '',
    '## RESUME chapter-count claim',
    '',
  );
  if (audit.resumeChapterCount) {
    lines.push(
      '- Claimed written: ' + code(audit.resumeChapterCount.claimedWritten) + '.',
      '- Claimed total: ' + code(audit.resumeChapterCount.claimedTotal) + '.',
      '- Domain chapter files on disk: ' + code(audit.resumeChapterCount.filesOnDisk) + '.',
      '- State: ' + code(audit.resumeChapterCount.status) + '.',
    );
  } else {
    lines.push('- State: ' + code('OPEN') + ' — no parseable chapter-count claim.');
  }
  lines.push(
    '',
    '## Spine-pending register',
    '',
    '| Item | Title | State |',
    '|---|---|---|',
  );
  for (const item of audit.spineItems) {
    lines.push('| ' + code(item.id) + ' | ' + item.title.replaceAll('|', '\\|') + ' | ' + code(item.state) + ' |');
  }
  lines.push(
    '',
    '## Dashboard counts',
    '',
    '- Consolidated requirements: ' + code(audit.summary.consolidatedRequirements) + '.',
    '- Roll-up mismatches: ' + code(audit.summary.rollupMismatches) + '.',
    '- Blank priorities: ' + code(audit.summary.blankPriorities) + '.',
    '- Blank statuses: ' + code(audit.summary.blankStatuses) + '.',
    '- Invalid statuses: ' + code(audit.summary.invalidStatuses) + '.',
    '- Requirements without an owned test ID: ' + code(audit.summary.requirementsWithoutOwnedTest) + '.',
    '- Missing promised artifacts: ' + code(audit.summary.missingPromisedArtifacts) + '.',
    '- Stale RESUME claims: ' + code(audit.summary.staleResumeClaims) + '.',
    '',
    '## Interpretation boundary',
    '',
    'This audit reports structural agreement and disagreement. It does not repair PRD text, infer a',
    'missing priority or status, supply a test, or convert an open spine item into a closed one.',
    '',
  );
  return lines.join('\n');
}

function buildPrdAudit(options = {}) {
  const auditSourcePath = options.sourcePath ?? sourcePath;
  const auditDirectory = options.prdDirectory ?? path.dirname(auditSourcePath);
  return auditPrd({
    indexMarkdown: fs.readFileSync(auditSourcePath, 'utf8'),
    indexPath: auditSourcePath,
    prdDirectory: auditDirectory,
  });
}

export function checkPrdAudit(options = {}) {
  const reportPath = options.prdAuditPath ?? prdAuditPath;
  if (!fs.existsSync(reportPath)) throw new Error('missing PRD integrity report: ' + reportPath);
  const expected = renderPrdAudit(buildPrdAudit(options));
  const actual = fs.readFileSync(reportPath, 'utf8');
  if (actual !== expected) throw new Error('PRD integrity report is stale');
  return true;
}

function writePrdAudit(options = {}) {
  const reportPath = options.prdAuditPath ?? prdAuditPath;
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  fs.writeFileSync(reportPath, renderPrdAudit(buildPrdAudit(options)), 'utf8');
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

export function checkTracker(options = {}) {
  const trackerSourcePath = options.sourcePath ?? sourcePath;
  const trackerLedgerPath = options.ledgerPath ?? ledgerPath;
  const trackerStatusPath = options.statusPath ?? statusPath;
  const trackerPrdAuditPath = options.prdAuditPath ?? prdAuditPath;
  const trackerTestEvidencePath = options.testEvidencePath ?? testEvidencePath;
  const trackerPrdDirectory = options.prdDirectory ?? path.dirname(trackerSourcePath);
  const sourceRows = parseConsolidatedRequirements(fs.readFileSync(trackerSourcePath, 'utf8'));
  if (!fs.existsSync(trackerLedgerPath)) throw new Error(`missing ledger: ${trackerLedgerPath}`);
  const ledgerRows = parseCsv(fs.readFileSync(trackerLedgerPath, 'utf8'));
  validateLedger(sourceRows, ledgerRows);
  if (!fs.existsSync(trackerTestEvidencePath)) {
    throw new Error(`missing exact test evidence map: ${trackerTestEvidencePath}`);
  }
  validateTestEvidence({
    ledgerRows,
    evidenceRows: parseTestEvidence(fs.readFileSync(trackerTestEvidencePath, 'utf8')),
    executableTests: options.executableTests ?? discoverExecutableTests(repo),
  });
  if (!fs.existsSync(trackerStatusPath)) throw new Error(`missing dashboard: ${trackerStatusPath}`);
  validateStatus(fs.readFileSync(trackerStatusPath, 'utf8'));
  if (fs.existsSync(trackerPrdAuditPath)) {
    checkPrdAudit({
      sourcePath: trackerSourcePath,
      prdAuditPath: trackerPrdAuditPath,
      prdDirectory: trackerPrdDirectory,
    });
  }
  return true;
}

function parseArgs(argv) {
  const supported = new Set([
    '--initialize',
    '--check',
    '--summary-json',
    '--write-prd-audit',
    '--check-prd-audit',
  ]);
  if (argv.length !== 1 || !supported.has(argv[0])) {
    throw new Error('use exactly one supported takeover-ledger mode');
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
  if (mode === '--write-prd-audit') {
    writePrdAudit();
    return;
  }
  if (mode === '--check-prd-audit') {
    checkPrdAudit();
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
