#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isGeneratedFileCurrent } from './generated-artifact.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function parseArgs(argv) {
  const args = {
    check: false,
    review: path.join(repo, 'REVIEW.md'),
    map: path.join(repo, 'verification', 'capabilities.json'),
    output: path.join(repo, 'docs', 'VERIFICATION_MATRIX.md'),
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--check') {
      args.check = true;
      continue;
    }
    if (arg === '--review' || arg === '--map' || arg === '--output') {
      const value = argv[i + 1];
      if (!value) throw new Error(`${arg} requires a path`);
      args[arg.slice(2)] = path.resolve(value);
      i += 1;
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }
  return args;
}

function reviewSections(markdown) {
  const sections = [];
  let current = null;
  for (const line of markdown.split(/\r?\n/u)) {
    const heading = /^##\s+(.+?)\s*$/u.exec(line);
    if (heading) {
      if (current) sections.push(current);
      const rawTitle = heading[1];
      const date = /\b(20\d{2}-\d{2}-\d{2})\b/u.exec(rawTitle)?.[1] ?? null;
      const title = rawTitle
        .replace(/^20\d{2}-\d{2}-\d{2}\s+[—-]\s+/u, '')
        .replace(/^Round\s+\d+\s+[—-]\s+/iu, '')
        .trim();
      current = { rawTitle, title, date, checked: 0, total: 0 };
      continue;
    }
    if (!current) continue;
    const mark = /^\s*-\s*\[([x ])\]/iu.exec(line);
    if (!mark) continue;
    current.total += 1;
    if (mark[1].toLowerCase() === 'x') current.checked += 1;
  }
  if (current) sections.push(current);
  return sections;
}

function loadMap(jsonText) {
  const parsed = JSON.parse(jsonText);
  if (parsed.schema_version !== 1 || !Array.isArray(parsed.capabilities)) {
    throw new Error('capability map must have schema_version 1 and a capabilities array');
  }
  // REQUIRED, never defaulted to an empty list. This matrix used to check one direction only — it
  // refused a capability matching no review section, and said nothing at all about a review
  // section matching no capability. A new section could therefore count toward nothing, silently,
  // and the only symptom was a total that failed to move. Defaulting an absent key to `[]` would
  // restore exactly that, so an absent key is an error.
  const acknowledged = parsed.unmapped_review_sections;
  if (!Array.isArray(acknowledged) || !acknowledged.every((value) => typeof value === 'string')) {
    throw new Error(
      'capability map must carry unmapped_review_sections as an array of REVIEW.md section titles',
    );
  }
  const duplicate = acknowledged.find(
    (title, index) => acknowledged.indexOf(title) !== index,
  );
  if (duplicate !== undefined) {
    throw new Error(`duplicate unmapped_review_sections entry: "${duplicate}"`);
  }
  const ids = new Set();
  const capabilities = parsed.capabilities.map((capability, index) => {
    const where = `capabilities[${index}]`;
    if (!/^[a-z0-9][a-z0-9-]*$/u.test(capability.id ?? '')) {
      throw new Error(`${where}.id must be a lowercase stable identifier`);
    }
    if (ids.has(capability.id)) throw new Error(`duplicate capability id: ${capability.id}`);
    ids.add(capability.id);
    if (typeof capability.title !== 'string' || !capability.title.trim()) {
      throw new Error(`${where}.title must be non-empty`);
    }
    const exact = capability.review_sections ?? [];
    const patterns = capability.review_section_patterns ?? [];
    const notListed = capability.not_listed === true;
    if (capability.not_listed !== undefined && typeof capability.not_listed !== 'boolean') {
      throw new Error(`${where}.not_listed must be a boolean`);
    }
    if (!Array.isArray(exact) || !exact.every((value) => typeof value === 'string')) {
      throw new Error(`${where}.review_sections must be an array of strings`);
    }
    if (!Array.isArray(patterns) || !patterns.every((value) => typeof value === 'string')) {
      throw new Error(`${where}.review_section_patterns must be an array of strings`);
    }
    if (!notListed && exact.length === 0 && patterns.length === 0) {
      throw new Error(`${where} must map at least one REVIEW.md section`);
    }
    if (notListed && (exact.length > 0 || patterns.length > 0)) {
      throw new Error(`${where} cannot be not_listed and carry REVIEW.md selectors`);
    }
    return {
      id: capability.id,
      title: capability.title.trim(),
      notListed,
      exact: new Set(exact),
      patterns: patterns.map((pattern) => new RegExp(pattern, 'iu')),
    };
  });
  return { capabilities, acknowledgedUnmapped: acknowledged };
}

/** Does any capability claim this section? Selector matching is on the title alone, so a title is
 *  either claimed or unclaimed — never both. A `not_listed` capability carries no selectors and
 *  therefore claims nothing, which is what it means. */
function isMapped(capability, section) {
  return (
    capability.exact.has(section.title)
    || capability.patterns.some((pattern) => pattern.test(section.title))
  );
}

function escapeCell(value) {
  return String(value).replaceAll('|', '\\|').replaceAll('\n', ' ');
}

function renderMatrix(reviewText, mapText) {
  const sections = reviewSections(reviewText);
  const undatedExercise = sections.find((section) => section.checked > 0 && !section.date);
  if (undatedExercise) {
    throw new Error(
      `checked REVIEW.md section "${undatedExercise.rawTitle}" has no ledger date`,
    );
  }
  const { capabilities, acknowledgedUnmapped } = loadMap(mapText);

  // The reverse direction of the `capability matches no REVIEW.md section` check below. A section
  // nothing claims contributes to no capability's count, which is a defensible state for the
  // historical entries that predate this map — but only while it is WRITTEN DOWN. An unlisted one
  // is refused by name, so a new review section cannot quietly count toward nothing.
  const unmappedTitles = new Set(
    sections
      .filter((section) => !capabilities.some((capability) => isMapped(capability, section)))
      .map((section) => section.title),
  );
  const unacknowledged = [...unmappedTitles].filter((title) => !acknowledgedUnmapped.includes(title));
  if (unacknowledged.length > 0) {
    throw new Error(
      `${unacknowledged.length} REVIEW.md section(s) match no capability and are not acknowledged in `
        + 'verification/capabilities.json: map each to a capability\'s selectors, or add its exact '
        + `title to unmapped_review_sections: ${unacknowledged.map((title) => `"${title}"`).join(', ')}`,
    );
  }
  // And the list cannot rot into a blanket exemption: an entry survives only while its section is
  // still present and still unclaimed. Retitle a section, or give it a capability, and its stale
  // acknowledgement has to go — otherwise the list would slowly stop describing anything and would
  // keep excusing sections nobody had looked at in a year.
  const staleAcknowledgements = acknowledgedUnmapped.filter((title) => !unmappedTitles.has(title));
  if (staleAcknowledgements.length > 0) {
    throw new Error(
      `${staleAcknowledgements.length} unmapped_review_sections entr(ies) no longer name an unmapped `
        + 'REVIEW.md section (retitled, removed, or since mapped) and must be deleted: '
        + `${staleAcknowledgements.map((title) => `"${title}"`).join(', ')}`,
    );
  }

  const rows = capabilities.map((capability) => {
    if (capability.notListed) {
      return {
        ...capability,
        checked: 0,
        total: 0,
        status: 'Not listed',
        ledgerDate: '—',
        sections: 0,
      };
    }
    const matched = sections.filter((section) => isMapped(capability, section));
    if (matched.length === 0) {
      throw new Error(`capability ${capability.id} matches no REVIEW.md section`);
    }
    const withScenarios = matched.filter((section) => section.total > 0);
    if (withScenarios.length === 0) {
      return {
        ...capability,
        checked: 0,
        total: 0,
        status: 'Not recorded',
        ledgerDate: '—',
        sections: matched.length,
      };
    }
    const checked = withScenarios.reduce((sum, section) => sum + section.checked, 0);
    const total = withScenarios.reduce((sum, section) => sum + section.total, 0);
    const dates = withScenarios
      .filter((section) => section.checked > 0 && section.date)
      .map((section) => section.date)
      .sort();
    const status = checked === 0
      ? 'Not exercised'
      : checked === total && withScenarios.length === matched.length
        ? 'Exercised'
        : 'Partially exercised';
    return {
      ...capability,
      checked,
      total,
      status,
      ledgerDate: dates.at(-1) ?? '—',
      sections: matched.length,
    };
  });

  const exercised = rows.filter((row) => row.checked > 0).length;
  const fullyExercised = rows.filter((row) => row.status === 'Exercised').length;
  const lines = [
    '# Capability verification matrix',
    '',
    '<!-- Generated by tools/generate-verification-matrix.mjs. Do not edit by hand. -->',
    '',
    'This matrix is generated mechanically from `REVIEW.md` and',
    '`verification/capabilities.json`. A checked review scenario means a human exercised it',
    'against real well data. **Ledger date** is the date on the newest dated review section',
    'containing a checked scenario; it is not a more precise test timestamp.',
    '',
    `Capabilities with recorded exercise: **${exercised} / ${rows.length}**. Fully exercised: **${fullyExercised} / ${rows.length}**.`,
    '',
    `Review sections counted toward no capability: **${unmappedTitles.size}** of ${sections.length},`,
    'each named in `unmapped_review_sections` in the capability map. They contribute to no count in',
    'this table, so every figure above reads low by whatever those sections cover.',
    '',
    '| Capability ID | Capability | Status | Checked scenarios | Ledger date | Review sections |',
    '|---|---|---|---:|---|---:|',
    ...rows.map(
      (row) =>
        `| \`${row.id}\` | ${escapeCell(row.title)} | ${row.status} | ${row.checked} / ${row.total} | ${row.ledgerDate} | ${row.sections} |`,
    ),
    '',
  ];
  return lines.join('\n');
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const expected = renderMatrix(
    fs.readFileSync(args.review, 'utf8'),
    fs.readFileSync(args.map, 'utf8'),
  );
  if (args.check) {
    const actual = fs.existsSync(args.output) ? fs.readFileSync(args.output, 'utf8') : '';
    if (!isGeneratedFileCurrent(actual, expected)) {
      throw new Error(
        `${path.relative(repo, args.output)} is out of date; run node tools/generate-verification-matrix.mjs`,
      );
    }
    return;
  }
  fs.mkdirSync(path.dirname(args.output), { recursive: true });
  fs.writeFileSync(args.output, expected, 'utf8');
}

try {
  main();
} catch (error) {
  console.error(`verification matrix: ${error instanceof Error ? error.message : error}`);
  process.exitCode = 1;
}
