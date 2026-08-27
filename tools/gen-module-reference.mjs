// Renders the per-module reference pages under docs/guide/reference/ from
// docs/generated/module_manifests.json — the committed dump of modules::list_modules(),
// kept fresh by manifest_reference_test.rs. The manifests are the single source of truth:
// these pages carry the same descriptions, defaults, sources and validity conditions the
// application's own panes are generated from, so the reference cannot drift from the app.
//
//   node tools/gen-module-reference.mjs           regenerate the pages
//   node tools/gen-module-reference.mjs --check   fail (exit 1) if any page is stale/orphaned
//
// Hand-written workflow prose does NOT go into these files (they are overwritten wholesale):
// put it in docs/guide/reference/notes/<module>.md and the generator includes it under a
// "Working notes" heading.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isGeneratedFileCurrent } from './generated-artifact.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dumpPath = path.join(repo, 'docs', 'generated', 'module_manifests.json');
const outDir = path.join(repo, 'docs', 'guide', 'reference');
const notesDir = path.join(outDir, 'notes');

const REGEN =
  'GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` ' +
  '(source: `docs/generated/module_manifests.json`, kept fresh by ' +
  '`manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`.';

const ABSENT_DEFAULT = 'ABSENT';

/** Escape a value for a markdown table cell. */
const cell = (s) => String(s ?? '').replaceAll('|', '\\|').replaceAll('\n', ' ').trim();

/** First sentence of a doc string, for the index. */
function summary(doc) {
  const text = doc.replaceAll('\n', ' ').trim();
  const stop = text.search(/\.\s|\.$/u);
  return stop === -1 ? text : text.slice(0, stop + 1);
}

function renderCondition(c) {
  let rule = '';
  if (c.kind === 'numeric_range') {
    const unit = c.unit ? ` ${c.unit}` : '';
    rule = ` Range: ${c.min}–${c.max}${unit}.`;
  } else if (c.kind === 'less_than') {
    rule = ` Must be strictly below \`${c.other}\`.`;
  } else if (c.kind === 'greater_than') {
    rule = ` Must be strictly above \`${c.other}\`.`;
  }
  const when = c.when ? ` Applies when: ${c.when}.` : '';
  return `- \`${c.id}\` — ${c.statement}${rule}${when}\n  Source: ${c.source}`;
}

function renderParam(a) {
  const lines = [];
  const unit = a.unit ? ` *(${a.unit})*` : '';
  lines.push(`### ${a.name}${unit}`);
  lines.push('');
  lines.push(a.desc);
  lines.push('');
  if (a.default_source === ABSENT_DEFAULT || (!a.default && !a.default_source)) {
    lines.push(
      '- **No shipped default.** You supply this value, and a run entering it explicitly ' +
        'must also cite the source that covers it.',
    );
  } else {
    lines.push(`- **Default:** ${a.default || '—'} — source: ${a.default_source}`);
  }
  if (a.min !== null || a.max !== null) {
    lines.push(`- **Accepted range:** ${a.min ?? '−∞'} to ${a.max ?? '∞'}${a.unit ? ` ${a.unit}` : ''}`);
  }
  if (a.well_scope) {
    lines.push('- **One value per well** — a named-zone override of this parameter is refused.');
  }
  if (a.sources_topic) {
    lines.push(
      `- Competing shipped values exist for this parameter across installed tools ` +
        `(topic \`${a.sources_topic}\`); the pane lists them with sources at the point of choice.`,
    );
  }
  for (const g of a.guidance ?? []) {
    lines.push(`- **Guidance:** ${g.text}\n  Source: ${g.source}`);
  }
  if ((a.validity_conditions ?? []).length) {
    lines.push('- **Checked before the run:**');
    for (const c of a.validity_conditions) lines.push(renderCondition(c).replace(/^-/u, '  -').replaceAll('\n  ', '\n    '));
  }
  return lines.join('\n');
}

function renderOption(a) {
  const lines = [];
  lines.push(`### ${a.name}`);
  lines.push('');
  lines.push(a.desc);
  lines.push('');
  if (a.kind === 'text') {
    lines.push(`- Free text.${a.default ? ` Default: \`${a.default}\`` : ''}`);
  } else {
    lines.push('- **Choices:**');
    a.choices.forEach((choice, i) => {
      // Labels usually repeat their own id ("LINEAR — VSH = IGR"); print the id once.
      let label = (a.choice_labels ?? [])[i] ?? '';
      if (label.startsWith(`${choice} — `)) label = label.slice(choice.length + 3);
      lines.push(`  - \`${choice}\`${label ? ` — ${label}` : ''}`);
    });
    if (a.default) lines.push(`- **Default:** \`${a.default}\``);
  }
  for (const g of a.guidance ?? []) {
    lines.push(`- **Guidance:** ${g.text}\n  Source: ${g.source}`);
  }
  if ((a.validity_conditions ?? []).length) {
    lines.push('- **Checked before the run:**');
    for (const c of a.validity_conditions) lines.push(renderCondition(c).replace(/^-/u, '  -').replaceAll('\n  ', '\n    '));
  }
  return lines.join('\n');
}

function inputRow(a) {
  const resolves = (a.preferred_aliases ?? []).length
    ? a.preferred_aliases.map((m) => `\`${m}\``).join(' → ')
    : a.default
      ? `\`${a.default}\``
      : '—';
  const notes = [];
  if ((a.required_any_of ?? []).length) notes.push(`or satisfied by ${a.required_any_of.map((n) => `\`${n}\``).join(', ')}`);
  if (a.computed_only) notes.push('resolved from computed curves only, never the RAW import store');
  if ((a.accepted_shale_clay_quantities ?? []).length)
    notes.push(`accepts quantity kind: ${a.accepted_shale_clay_quantities.join(', ')}`);
  return `| ${cell(a.name)} | ${cell(a.desc)} | ${resolves} | ${a.required ? 'yes' : 'no'} | ${cell(notes.join('; ')) || '—'} |`;
}

function outputRow(a) {
  const flag = a.flag_kind ? ` *(flag: ${a.flag_kind})*` : '';
  return `| ${cell(a.name)} | ${cell(a.desc)}${flag} |`;
}

function renderModule(m) {
  const byKind = (k) => m.args.filter((a) => a.kind === k);
  const params = byKind('param');
  const options = [...byKind('option'), ...byKind('text')];
  const inputs = byKind('log_in');
  const outputs = byKind('log_out');

  const parts = [];
  parts.push(`<!-- ${REGEN} -->`);
  parts.push('');
  parts.push(`# ${m.title}`);
  parts.push('');
  parts.push(`Module id \`${m.name}\` · category **${m.category}** · [reference index](README.md)`);
  parts.push('');
  parts.push(m.doc);
  if (inputs.length) {
    parts.push('');
    parts.push('## Input curves');
    parts.push('');
    parts.push('| Role | Description | Resolves to | Required | Notes |');
    parts.push('|---|---|---|---|---|');
    for (const a of inputs) parts.push(inputRow(a));
  }
  if (params.length) {
    parts.push('');
    parts.push('## Parameters');
    parts.push('');
    parts.push(
      'Whole-well defaults; per-zone values from the Zones pane take precedence inside ' +
        'their zones (except where a parameter is marked one-value-per-well).',
    );
    for (const a of params) {
      parts.push('');
      parts.push(renderParam(a));
    }
  }
  if (options.length) {
    parts.push('');
    parts.push('## Options');
    for (const a of options) {
      parts.push('');
      parts.push(renderOption(a));
    }
  }
  if (outputs.length) {
    parts.push('');
    parts.push('## Output curves');
    parts.push('');
    parts.push('| Name | Description |');
    parts.push('|---|---|');
    for (const a of outputs) parts.push(outputRow(a));
  }
  const notesPath = path.join(notesDir, `${m.name}.md`);
  if (fs.existsSync(notesPath)) {
    parts.push('');
    parts.push('## Working notes');
    parts.push('');
    parts.push(fs.readFileSync(notesPath, 'utf8').replaceAll('\r\n', '\n').trim());
  }
  parts.push('');
  return parts.join('\n');
}

function renderIndex(specs) {
  const parts = [];
  parts.push(`<!-- ${REGEN} -->`);
  parts.push('');
  parts.push('# Module reference');
  parts.push('');
  parts.push(
    'One page per petrophysics module, generated from the same manifests the application ' +
      'builds its parameter panes from — descriptions, defaults, sources, ranges and ' +
      'pre-run checks here are exactly what the running application enforces. For the ' +
      'workflow these modules live in, start with the [first hour guide](../book/first_hour.html).',
  );
  const categories = [...new Set(specs.map((m) => m.category))].sort();
  for (const cat of categories) {
    parts.push('');
    parts.push(`## ${cat}`);
    parts.push('');
    parts.push('| Module | Title | What it does |');
    parts.push('|---|---|---|');
    for (const m of specs.filter((s) => s.category === cat)) {
      parts.push(`| [\`${m.name}\`](${m.name}.md) | ${cell(m.title)} | ${cell(summary(m.doc))} |`);
    }
  }
  parts.push('');
  return parts.join('\n');
}

// ---------------------------------------------------------------------------

const check = process.argv.includes('--check');
const specs = JSON.parse(fs.readFileSync(dumpPath, 'utf8'));

const pages = new Map();
pages.set('README.md', renderIndex(specs));
for (const m of specs) pages.set(`${m.name}.md`, renderModule(m));

if (check) {
  const problems = [];
  for (const [name, fresh] of pages) {
    const p = path.join(outDir, name);
    if (!fs.existsSync(p)) {
      problems.push(`missing: docs/guide/reference/${name}`);
      continue;
    }
    if (!isGeneratedFileCurrent(fs.readFileSync(p, 'utf8'), fresh)) {
      problems.push(`stale: docs/guide/reference/${name}`);
    }
  }
  for (const name of fs.readdirSync(outDir)) {
    if (!name.endsWith('.md')) continue;
    if (!pages.has(name)) problems.push(`orphan (module no longer exists): docs/guide/reference/${name}`);
  }
  if (problems.length) {
    console.error('module reference is out of date:');
    for (const p of problems) console.error(`  ${p}`);
    console.error('regenerate with: node tools/gen-module-reference.mjs');
    console.error('(if the manifests themselves changed, first: SANDIBUMI_WRITE_MANIFEST_DUMP=1 cargo test --lib the_committed_manifest_dump)');
    process.exit(1);
  }
  console.log(`module reference current: ${pages.size} page(s)`);
} else {
  fs.mkdirSync(outDir, { recursive: true });
  for (const [name, fresh] of pages) fs.writeFileSync(path.join(outDir, name), fresh);
  for (const name of fs.readdirSync(outDir)) {
    if (name.endsWith('.md') && !pages.has(name)) {
      fs.rmSync(path.join(outDir, name));
      console.log(`removed orphan ${name}`);
    }
  }
  console.log(`wrote ${pages.size} page(s) to docs/guide/reference/`);
}
