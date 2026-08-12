import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const defaultRepo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ignoredCategories = ['OPTIONAL-PACKAGE', 'CONTROLLED-CORPUS', 'MANUAL-ARTIFACT'];
const ignoredExecutionStates = ['NOT-RUN-BY-DEFAULT', 'UNVERIFIED', 'KNOWN-FAILING', 'MANUAL-ONLY'];

function slash(value) {
  return value.replaceAll('\\', '/');
}

function normalizedRelativePath(fileName, repoRoot) {
  const absolute = path.isAbsolute(fileName) ? fileName : path.resolve(repoRoot, fileName);
  return slash(path.relative(repoRoot, absolute));
}

function warningKey(sourcePath, lint, message) {
  return `${sourcePath}|${lint}|${message}`;
}

export function parseCargoWarnings(ndjson, options = {}) {
  const repoRoot = path.resolve(options.repoRoot ?? path.join(defaultRepo, 'src-tauri'));
  const warnings = [];
  for (const rawLine of ndjson.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (line === '') continue;
    let record;
    try {
      record = JSON.parse(line);
    } catch {
      continue;
    }
    if (record.reason !== 'compiler-message' || record.message?.level !== 'warning') continue;
    const primary = (record.message.spans ?? []).find((span) => span.is_primary);
    if (!primary?.file_name) continue;
    const sourcePath = normalizedRelativePath(primary.file_name, repoRoot);
    const lint = record.message.code?.code ?? 'unclassified-rustc-warning';
    const message = record.message.message ?? '';
    warnings.push({
      key: warningKey(sourcePath, lint, message),
      source_path: sourcePath,
      lint,
      message,
      observed_line: primary.line_start,
    });
  }
  return warnings.sort((a, b) => a.key.localeCompare(b.key));
}

function ignoredTestKey(sourcePath, testName) {
  return `${sourcePath}|${testName}`;
}

function controlledCorpusSignal(testName, reason, functionText) {
  return /(?:SANDIBUMI_FIELD_FIXTURES|SANDIBUMI_TEST_DLIS|SANDIBUMI_E2E_DB|field_fixtures|pipeline_field|real_field|real_dlis|probe_real|full_deterministic_chain|e2e_pef|delivered_book|real_petrography)/iu
    .test(`${testName}\n${reason ?? ''}\n${functionText}`);
}

function manualArtifactSignal(testName) {
  return testName === 'dump_sample';
}

function functionTextAt(sourceText, functionStart) {
  const lineStart = sourceText.lastIndexOf('\n', functionStart - 1) + 1;
  const functionLine = sourceText.slice(lineStart, sourceText.indexOf('\n', lineStart) === -1
    ? sourceText.length
    : sourceText.indexOf('\n', lineStart));
  const indentation = /^(\s*)/u.exec(functionLine)?.[1] ?? '';
  const escapedIndentation = indentation.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
  const closingPattern = new RegExp(`^${escapedIndentation}\\}`, 'mu');
  const afterStart = sourceText.slice(functionStart);
  const closing = closingPattern.exec(afterStart);
  return closing ? afterStart.slice(0, closing.index + closing[0].length) : afterStart;
}

export function scanIgnoredTests(sources) {
  const tests = [];
  const ignorePattern = /^[ \t]*#\s*\[\s*ignore(?:\s*=\s*"([^"]*)")?\s*\][ \t]*$/gmu;
  for (const source of sources) {
    const sourcePath = slash(source.sourcePath);
    for (const match of source.text.matchAll(ignorePattern)) {
      const afterAttribute = match.index + match[0].length;
      const remainder = source.text.slice(afterAttribute);
      const fnMatch = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/u.exec(remainder);
      if (!fnMatch) continue;
      const testName = fnMatch[1];
      const functionStart = afterAttribute + (fnMatch.index ?? 0);
      const functionText = functionTextAt(source.text, functionStart);
      const ignoreReason = match[1] ?? null;
      const observedLine = source.text.slice(0, match.index).split(/\r?\n/u).length;
      tests.push({
        key: ignoredTestKey(sourcePath, testName),
        source_path: sourcePath,
        test_name: testName,
        ignore_reason: ignoreReason,
        bare_ignore: ignoreReason === null,
        controlled_corpus_signal: controlledCorpusSignal(testName, ignoreReason, functionText),
        manual_artifact_signal: manualArtifactSignal(testName),
        observed_line: observedLine,
      });
    }
  }
  return tests.sort((a, b) => a.key.localeCompare(b.key));
}

export function findBroadDeadCodeAllowances(sources) {
  const allowances = [];
  const inner = /^\s*#!\s*\[\s*allow\s*\(\s*(?:dead_code|unused)\s*\)\s*\]/u;
  const outer = /^\s*#\s*\[\s*allow\s*\(\s*(?:dead_code|unused)\s*\)\s*\]/u;
  for (const source of sources) {
    const sourcePath = slash(source.sourcePath);
    const lines = source.text.split(/\r?\n/u);
    for (let index = 0; index < lines.length; index += 1) {
      if (inner.test(lines[index])) {
        allowances.push({ source_path: sourcePath, line: index + 1, scope: 'crate-or-module' });
        continue;
      }
      if (!outer.test(lines[index])) continue;
      let next = index + 1;
      while (next < lines.length && (/^\s*$/u.test(lines[next]) || /^\s*\/\//u.test(lines[next]))) next += 1;
      if (/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+/u.test(lines[next] ?? '')) {
        allowances.push({ source_path: sourcePath, line: index + 1, scope: 'module' });
      }
    }
  }
  return allowances;
}

function duplicateKeys(items) {
  const seen = new Set();
  const duplicates = new Set();
  for (const item of items) {
    if (seen.has(item.key)) duplicates.add(item.key);
    seen.add(item.key);
  }
  return [...duplicates];
}

function countBy(items, selector) {
  const counts = {};
  for (const item of items) {
    const key = selector(item);
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return counts;
}

function compareCounts(label, declared, observed, errors) {
  const keys = new Set([...Object.keys(declared ?? {}), ...Object.keys(observed)]);
  for (const key of [...keys].sort()) {
    if ((declared?.[key] ?? 0) !== (observed[key] ?? 0)) {
      errors.push(`${label} ${key} declares ${declared?.[key] ?? 0} but observed ${observed[key] ?? 0}`);
    }
  }
}

function validRequirementId(value) {
  return /^SB-[A-Z]+-[0-9]{3}$/u.test(value ?? '');
}

function warningOwnersFromInventory(inventory) {
  const direct = Array.isArray(inventory.warnings) ? inventory.warnings : [];
  const grouped = (inventory.warning_groups ?? []).flatMap((group) => {
    const { warning_keys: warningKeys = [], ...ownership } = group;
    return warningKeys.map((key) => ({ key, ...ownership }));
  });
  return [...direct, ...grouped];
}

function ignoredOwnersFromInventory(inventory) {
  const direct = Array.isArray(inventory.tests) ? inventory.tests : [];
  const grouped = (inventory.test_groups ?? []).flatMap((group) => {
    const { tests = [], ...ownership } = group;
    return tests.map((test) => {
      const separator = test.key.lastIndexOf('|');
      return {
        source_path: test.key.slice(0, separator),
        test_name: test.key.slice(separator + 1),
        ...test,
        ...ownership,
      };
    });
  });
  return [...direct, ...grouped];
}

export function validateHygieneInventory({
  compilerWarnings,
  ignoredTests,
  warningInventory,
  ignoredInventory,
  broadAllowances,
}) {
  const errors = [];
  if (warningInventory.schema_version !== 1) errors.push('warning inventory schema version must be 1');
  if (ignoredInventory.schema_version !== 1) errors.push('ignored-test inventory schema version must be 1');

  for (const allowance of broadAllowances) {
    errors.push(`broad dead_code allowance at ${allowance.source_path}:${allowance.line}`);
  }

  const warningOwners = warningOwnersFromInventory(warningInventory);
  for (const key of duplicateKeys(warningOwners)) errors.push(`duplicate warning owner ${key}`);
  for (const key of duplicateKeys(compilerWarnings)) errors.push(`duplicate live warning ${key}`);
  const warningOwnerByKey = new Map(warningOwners.map((item) => [item.key, item]));
  const liveWarningByKey = new Map(compilerWarnings.map((item) => [item.key, item]));
  for (const warning of compilerWarnings) {
    const owner = warningOwnerByKey.get(warning.key);
    if (!owner) {
      errors.push(`unclassified warning ${warning.key}`);
      continue;
    }
    for (const field of ['source_path', 'lint', 'message']) {
      if (owner[field] !== undefined && owner[field] !== warning[field]) {
        errors.push(`warning owner ${warning.key} has mismatched ${field}`);
      }
    }
    if (!validRequirementId(owner.owner_requirement)) {
      errors.push(`warning ${warning.key} has invalid owner requirement ${owner.owner_requirement}`);
    }
    if (!['G2', 'G3', 'G4', 'G5', 'LATER'].includes(owner.owner_gate)) {
      errors.push(`warning ${warning.key} has invalid owner gate ${owner.owner_gate}`);
    }
    if (typeof owner.disposition !== 'string' || owner.disposition.trim() === '') {
      errors.push(`warning ${warning.key} has no disposition`);
    }
    if (typeof owner.rationale !== 'string' || owner.rationale.trim() === '') {
      errors.push(`warning ${warning.key} has no rationale`);
    }
  }
  for (const owner of warningOwners) {
    if (!liveWarningByKey.has(owner.key)) errors.push(`stale warning owner ${owner.key}`);
  }
  if (warningInventory.expected_warning_count !== compilerWarnings.length) {
    errors.push(`warning inventory declares ${warningInventory.expected_warning_count} but observed ${compilerWarnings.length}`);
  }
  if (warningOwners.length !== compilerWarnings.length) {
    errors.push(`warning owner count ${warningOwners.length} does not match observed ${compilerWarnings.length}`);
  }
  compareCounts(
    'warning file',
    warningInventory.expected_warning_counts_by_file,
    countBy(compilerWarnings, (item) => item.source_path),
    errors,
  );

  const ignoredOwners = ignoredOwnersFromInventory(ignoredInventory);
  for (const key of duplicateKeys(ignoredOwners)) errors.push(`duplicate ignored-test owner ${key}`);
  for (const key of duplicateKeys(ignoredTests)) errors.push(`duplicate live ignored test ${key}`);
  const ignoredOwnerByKey = new Map(ignoredOwners.map((item) => [item.key, item]));
  const liveIgnoredByKey = new Map(ignoredTests.map((item) => [item.key, item]));
  for (const ignored of ignoredTests) {
    const owner = ignoredOwnerByKey.get(ignored.key);
    if (!owner) {
      errors.push(ignored.bare_ignore
        ? `bare ignored test ${ignored.key} has no inventory owner`
        : `unclassified ignored test ${ignored.key}`);
      continue;
    }
    if (owner.source_path !== ignored.source_path || owner.test_name !== ignored.test_name) {
      errors.push(`ignored-test owner identity does not match ${ignored.key}`);
    }
    if (owner.ignore_reason !== ignored.ignore_reason || owner.bare_ignore !== ignored.bare_ignore) {
      errors.push(`ignored-test attribute changed for ${ignored.key}`);
    }
    if (!ignoredCategories.includes(owner.category)) {
      errors.push(`ignored test ${ignored.key} has invalid category ${owner.category}`);
    }
    if (!ignoredExecutionStates.includes(owner.execution_state)) {
      errors.push(`ignored test ${ignored.key} has invalid execution state ${owner.execution_state}`);
    }
    if (!validRequirementId(owner.owner_requirement)) {
      errors.push(`ignored test ${ignored.key} has invalid owner requirement ${owner.owner_requirement}`);
    }
    if (owner.category === 'OPTIONAL-PACKAGE' && owner.owner_gate !== 'G3') {
      errors.push(`optional-package test ${ignored.key} must be owned by G3`);
    }
    if (owner.category === 'CONTROLLED-CORPUS' && owner.owner_gate !== 'G4') {
      errors.push(`controlled-corpus test ${ignored.key} must be owned by G4`);
    }
    if (owner.category === 'MANUAL-ARTIFACT' && owner.owner_gate !== 'G4') {
      errors.push(`manual artifact ${ignored.key} must be owned by G4`);
    }
    if (ignored.controlled_corpus_signal && owner.category !== 'CONTROLLED-CORPUS') {
      errors.push(`field signal for ${ignored.key} requires CONTROLLED-CORPUS, not ${owner.category}`);
    }
    if (ignored.manual_artifact_signal && owner.category !== 'MANUAL-ARTIFACT') {
      errors.push(`manual artifact signal for ${ignored.key} requires MANUAL-ARTIFACT`);
    }
    if (owner.execution_state === 'PASSED') {
      errors.push(`ignored test ${ignored.key} cannot be counted PASSED`);
    }
    if (typeof owner.rationale !== 'string' || owner.rationale.trim() === '') {
      errors.push(`ignored test ${ignored.key} has no rationale`);
    }
  }
  for (const owner of ignoredOwners) {
    if (!liveIgnoredByKey.has(owner.key)) errors.push(`stale ignored-test owner ${owner.key}`);
  }
  if (ignoredInventory.expected_ignored_test_count !== ignoredTests.length) {
    errors.push(`ignored-test inventory declares ${ignoredInventory.expected_ignored_test_count} but observed ${ignoredTests.length}`);
  }
  if (ignoredOwners.length !== ignoredTests.length) {
    errors.push(`ignored-test owner count ${ignoredOwners.length} does not match observed ${ignoredTests.length}`);
  }
  compareCounts(
    'ignored category',
    ignoredInventory.expected_category_counts,
    countBy(ignoredOwners, (item) => item.category),
    errors,
  );

  for (const key of ignoredInventory.known_failing_tests ?? []) {
    const owner = ignoredOwnerByKey.get(key);
    if (!owner) errors.push(`known failing ignored test ${key} has no owner`);
    else if (owner.execution_state !== 'KNOWN-FAILING') {
      errors.push(`known failing ignored test ${key} is labelled ${owner.execution_state}`);
    }
  }

  if (errors.length > 0) throw new Error(errors.join('; '));
  return {
    warnings: compilerWarnings.length,
    ignored: ignoredTests.length,
    ignored_category_counts: countBy(ignoredOwners, (item) => item.category),
  };
}

export function readRustSources(root) {
  const sources = [];
  const visit = (directory) => {
    if (!fs.existsSync(directory)) return;
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const fullPath = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(fullPath);
      else if (entry.isFile() && entry.name.endsWith('.rs')) {
        sources.push({
          sourcePath: slash(path.relative(path.join(root, 'src-tauri'), fullPath)),
          text: fs.readFileSync(fullPath, 'utf8'),
        });
      }
    }
  };
  visit(path.join(root, 'src-tauri', 'src'));
  visit(path.join(root, 'src-tauri', 'tests'));
  return sources.sort((a, b) => a.sourcePath.localeCompare(b.sourcePath));
}

export function collectCargoWarnings(repo = defaultRepo) {
  const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
  const result = spawnSync(cargo, ['check', '--message-format=json'], {
    cwd: path.join(repo, 'src-tauri'),
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`cargo check failed while collecting Gate 2 warnings:\n${result.stderr || result.stdout}`);
  }
  return parseCargoWarnings(result.stdout, { repoRoot: path.join(repo, 'src-tauri') });
}

export function checkLiveHygiene(repo = defaultRepo, options = {}) {
  const evidence = path.join(repo, 'docs', 'takeover', 'evidence');
  const warningInventory = JSON.parse(fs.readFileSync(path.join(evidence, 'gate2-warning-inventory.json'), 'utf8'));
  const ignoredInventory = JSON.parse(fs.readFileSync(path.join(evidence, 'gate2-ignored-test-inventory.json'), 'utf8'));
  const sources = readRustSources(repo);
  return validateHygieneInventory({
    compilerWarnings: options.compilerWarnings ?? collectCargoWarnings(repo),
    ignoredTests: scanIgnoredTests(sources),
    warningInventory,
    ignoredInventory,
    broadAllowances: findBroadDeadCodeAllowances(sources),
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = checkLiveHygiene();
    console.log(
      `Gate 2 hygiene valid: ${result.warnings} owned Rust warnings, ${result.ignored} owned ignored tests `
      + `(${result.ignored_category_counts['OPTIONAL-PACKAGE']} package, `
      + `${result.ignored_category_counts['CONTROLLED-CORPUS']} corpus, `
      + `${result.ignored_category_counts['MANUAL-ARTIFACT']} artifact)`,
    );
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
