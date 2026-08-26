// Renders the offline HTML guidebook under docs/guide/book/ from
// docs/generated/module_manifests.json (the committed dump of modules::list_modules(),
// with the Help-card registry merged in under `help` — kept fresh by
// manifest_reference_test.rs). One chapter per module that has an authored walkthrough
// in docs/guide/chapters/<module>.html, plus an index of every module.
//
//   node tools/gen-guidebook.mjs           regenerate the book
//   node tools/gen-guidebook.mjs --check   fail (exit 1) if any page is stale/orphaned
//
// The split that keeps this honest: everything FACTUAL (doc, equations, references,
// inputs, parameters with their sources, options, outputs, pre-run checks) renders
// from the dump and cannot drift from the running application; the step-by-step
// walkthrough and screenshots are the hand-written fragment, folded in verbatim.
// Internal provenance (PRD sections, Geolog line numbers) appears HERE, in the
// chapter — the in-app Help card carries only the published references.
//
// Offline and self-contained: no CDN, no web fonts, images relative (../img/ resolves
// in both the repo layout and the bundled guide/ resource layout).

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isGeneratedFileCurrent } from './generated-artifact.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dumpPath = path.join(repo, 'docs', 'generated', 'module_manifests.json');
const chaptersDir = path.join(repo, 'docs', 'guide', 'chapters');
const outDir = path.join(repo, 'docs', 'guide', 'book');

const REGEN =
  'GENERATED — do not hand-edit. Regenerate with `node tools/gen-guidebook.mjs` ' +
  '(source: docs/generated/module_manifests.json + docs/guide/chapters/<module>.html).';

const ABSENT_DEFAULT = 'ABSENT';

const esc = (s) =>
  String(s ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');

const CSS = `
  :root {
    --bg: #f5ead8; --panel: #ffffff; --panel-alt: #efe9dc;
    --text: #2b2118; --dim: #6b5c4a; --brand: #c67139; --accent-text: #8c491a;
    --sage: #7a8a5e; --border: #ded3c0;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0; background: var(--bg); color: var(--text);
    font: 15px/1.55 "Segoe UI", system-ui, sans-serif;
  }
  .layout { max-width: 1150px; margin: 0 auto; padding: 24px 20px 60px;
            display: flex; gap: 18px; align-items: flex-start; }
  .nav { width: 250px; flex: none; position: sticky; top: 16px;
         max-height: calc(100vh - 32px); overflow-y: auto;
         background: var(--panel); border: 1px solid var(--border);
         border-radius: 12px; padding: 12px 14px 16px; font-size: 13px; }
  .nav .nav-home { display: block; font-weight: 700; color: var(--brand);
                   text-decoration: none; padding: 4px 6px 8px; }
  .nav h2 { font-size: 11px; text-transform: uppercase; letter-spacing: .06em;
            color: var(--dim); border: none; margin: 12px 0 3px; padding: 0 6px; }
  .nav a { display: block; padding: 2px 6px; border-radius: 4px;
           color: var(--text); text-decoration: none; }
  .nav a:hover { background: var(--panel-alt); }
  .nav a.here { background: var(--brand); color: #fff; }
  .nav .missing { color: var(--dim); font-style: italic; padding: 2px 6px; }
  .content { flex: 1; min-width: 0; }
  @media (max-width: 920px) { .nav { display: none; } .layout { display: block; } }
  .card { background: var(--panel); border: 1px solid var(--border);
          border-radius: 12px; padding: 28px 32px; }
  .crumb { font-size: 12px; color: var(--dim); margin-bottom: 14px; }
  .crumb a { color: var(--accent-text); }
  h1 { font-size: 30px; color: var(--brand); margin: 0 0 4px; }
  .modmeta { color: var(--dim); font-size: 13px; margin: 0 0 18px; }
  h2 { font-size: 20px; color: var(--accent-text); margin: 30px 0 10px;
       border-bottom: 1px solid var(--border); padding-bottom: 4px; }
  h3 { font-size: 16px; margin: 22px 0 8px; }
  code, pre { font-family: "Cascadia Code", Consolas, monospace; font-size: 13px; }
  pre.equations { background: var(--panel-alt); border: 1px solid var(--border);
                  border-radius: 5px; padding: 10px 14px; white-space: pre-wrap;
                  overflow-x: auto; }
  blockquote { border-left: 3px solid var(--sage); margin: 12px 0; padding: 4px 14px;
               background: var(--panel-alt); border-radius: 0 5px 5px 0; }
  figure { margin: 18px 0; }
  figure img { max-width: 100%; border: 1px solid var(--border); border-radius: 5px; }
  figcaption { font-size: 12px; color: var(--dim); margin-top: 4px; }
  table { border-collapse: collapse; width: 100%; margin: 10px 0; font-size: 13px; }
  th, td { border: 1px solid var(--border); padding: 5px 8px; text-align: left;
           vertical-align: top; }
  th { background: var(--panel-alt); }
  .wrap { overflow-x: auto; }
  ul { padding-left: 22px; }
  .src { font-size: 12px; color: var(--dim); }
  .note { font-size: 13px; color: var(--dim); font-style: italic; }
  .absent { font-weight: 600; }
  .footer { font-size: 11px; color: var(--dim); margin-top: 26px; }
  a { color: var(--accent-text); }
  .toc td.missing { color: var(--dim); font-style: italic; }
`;

function shell(title, nav, body) {
  return `<!doctype html>
<!-- ${REGEN} -->
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>${esc(title)} — SandiBumi Guidebook</title>
<style>${CSS}</style>
</head>
<body>
<div class="layout">
<nav class="nav">
${nav}
</nav>
<div class="content"><div class="card">
${body}
</div></div>
</div>
</body>
</html>
`;
}

// The reading order of the book: the workflow order an interpretation actually
// follows (prepare, condition, frame, then the interpretation shelves), not the
// manifest's own registration order. Categories the list does not know are
// appended in manifest order so a new shelf can never fall out of the nav.
const CATEGORY_ORDER = [
  'Prep', 'Condition', 'Frame', 'VSH', 'Porosity', 'Lithology', 'Saturation',
  'Permeability', 'Rock Typing', 'ThinBeds', 'Facies', 'Unconventional',
];

function orderedCategories(specs) {
  const present = [...new Set(specs.map((m) => m.category))];
  return [
    ...CATEGORY_ORDER.filter((c) => present.includes(c)),
    ...present.filter((c) => !CATEGORY_ORDER.includes(c)),
  ];
}

/** The left navigation pane: every module grouped by category in reading order,
 *  the current page highlighted, unwritten chapters greyed rather than hidden so
 *  the reader always sees the whole shape of the book. */
function navHtml(specs, hasChapter, current) {
  const parts = [];
  parts.push(
    `<a class="nav-home${current === 'index' ? ' here' : ''}" href="index.html">SandiBumi Guidebook</a>`,
  );
  for (const cat of orderedCategories(specs)) {
    parts.push(`<h2>${esc(cat)}</h2>`);
    for (const m of specs.filter((s) => s.category === cat)) {
      if (!hasChapter.has(m.name)) {
        parts.push(`<span class="missing">${esc(m.title)}</span>`);
      } else if (m.name === current) {
        parts.push(`<a class="here" href="${esc(m.name)}.html">${esc(m.title)}</a>`);
      } else {
        parts.push(`<a href="${esc(m.name)}.html">${esc(m.title)}</a>`);
      }
    }
  }
  return parts.join('\n');
}

function conditionText(c) {
  let rule = '';
  if (c.kind === 'numeric_range') {
    rule = ` Range: ${c.min}–${c.max}${c.unit ? ` ${esc(c.unit)}` : ''}.`;
  } else if (c.kind === 'less_than') {
    rule = ` Must be strictly below <code>${esc(c.other)}</code>.`;
  } else if (c.kind === 'greater_than') {
    rule = ` Must be strictly above <code>${esc(c.other)}</code>.`;
  }
  const when = c.when ? ` Applies when: ${esc(c.when)}.` : '';
  return `<code>${esc(c.id)}</code> — ${esc(c.statement)}${rule}${when}
    <div class="src">Source: ${esc(c.source)}</div>`;
}

function renderParam(a) {
  const parts = [];
  parts.push(`<h3>${esc(a.name)}${a.unit ? ` <span class="src">(${esc(a.unit)})</span>` : ''}</h3>`);
  parts.push(`<p>${esc(a.desc)}</p>`);
  const items = [];
  if (a.default_source === ABSENT_DEFAULT || (!a.default && !a.default_source)) {
    items.push(
      '<span class="absent">No shipped default.</span> You supply this value, and a run ' +
        'entering it explicitly must also cite the source that covers it.',
    );
  } else {
    // The provenance behind a default (default_source) is deliberately NOT rendered
    // here: it cites internal files and decision records, which belong in the pane's
    // source tooltip and docs/guide/reference/, not in the client-facing book.
    items.push(`<b>Default:</b> ${esc(a.default) || '—'}`);
  }
  if (a.min !== null || a.max !== null) {
    items.push(`<b>Accepted range:</b> ${a.min ?? '−∞'} to ${a.max ?? '∞'}${a.unit ? ` ${esc(a.unit)}` : ''}`);
  }
  if (a.well_scope) {
    items.push('<b>One value per well</b> — a named-zone override of this parameter is refused.');
  }
  if (a.sources_topic) {
    items.push(
      `Competing shipped values exist for this parameter across installed tools ` +
        `(topic <code>${esc(a.sources_topic)}</code>); the pane lists them with sources at the point of choice.`,
    );
  }
  for (const g of a.guidance ?? []) {
    items.push(`<b>Guidance:</b> ${esc(g.text)} <div class="src">Source: ${esc(g.source)}</div>`);
  }
  for (const c of a.validity_conditions ?? []) {
    items.push(`<b>Checked before the run:</b> ${conditionText(c)}`);
  }
  parts.push(`<ul>${items.map((i) => `<li>${i}</li>`).join('\n')}</ul>`);
  return parts.join('\n');
}

function renderOption(a) {
  const parts = [];
  parts.push(`<h3>${esc(a.name)}</h3>`);
  parts.push(`<p>${esc(a.desc)}</p>`);
  const items = [];
  if (a.kind === 'text') {
    items.push(`Free text.${a.default ? ` Default: <code>${esc(a.default)}</code>` : ''}`);
  } else {
    const rows = a.choices.map((choice, i) => {
      let label = (a.choice_labels ?? [])[i] ?? '';
      if (label.startsWith(`${choice} — `)) label = label.slice(choice.length + 3);
      return `<li><code>${esc(choice)}</code>${label ? ` — ${esc(label)}` : ''}</li>`;
    });
    items.push(`<b>Choices:</b><ul>${rows.join('\n')}</ul>`);
    if (a.default) items.push(`<b>Default:</b> <code>${esc(a.default)}</code>`);
  }
  for (const g of a.guidance ?? []) {
    items.push(`<b>Guidance:</b> ${esc(g.text)} <div class="src">Source: ${esc(g.source)}</div>`);
  }
  for (const c of a.validity_conditions ?? []) {
    items.push(`<b>Checked before the run:</b> ${conditionText(c)}`);
  }
  parts.push(`<ul>${items.map((i) => `<li>${i}</li>`).join('\n')}</ul>`);
  return parts.join('\n');
}

function inputRow(a) {
  const resolves = (a.preferred_aliases ?? []).length
    ? a.preferred_aliases.map((m) => `<code>${esc(m)}</code>`).join(' → ')
    : a.default
      ? `<code>${esc(a.default)}</code>`
      : '—';
  const notes = [];
  if ((a.required_any_of ?? []).length)
    notes.push(`or satisfied by ${a.required_any_of.map((n) => `<code>${esc(n)}</code>`).join(', ')}`);
  if (a.computed_only) notes.push('resolved from computed curves only, never the RAW import store');
  if ((a.accepted_shale_clay_quantities ?? []).length)
    notes.push(`accepts quantity kind: ${esc(a.accepted_shale_clay_quantities.join(', '))}`);
  return `<tr><td>${esc(a.name)}</td><td>${esc(a.desc)}</td><td>${resolves}</td>` +
    `<td>${a.required ? 'yes' : 'no'}</td><td>${notes.join('; ') || '—'}</td></tr>`;
}

function outputRow(a) {
  const flag = a.flag_kind ? ` <span class="src">(flag: ${esc(a.flag_kind)})</span>` : '';
  return `<tr><td>${esc(a.name)}</td><td>${esc(a.desc)}${flag}</td></tr>`;
}

function renderChapter(m, walkthrough, nav) {
  const byKind = (k) => m.args.filter((a) => a.kind === k);
  const params = byKind('param');
  const options = [...byKind('option'), ...byKind('text')];
  const inputs = byKind('log_in');
  const outputs = byKind('log_out');

  const parts = [];
  parts.push(`<div class="crumb"><a href="index.html">SandiBumi Guidebook</a> · ${esc(m.category)}</div>`);
  parts.push(`<h1>${esc(m.title)}</h1>`);
  parts.push(`<p class="modmeta">Module id <code>${esc(m.name)}</code> · category <b>${esc(m.category)}</b></p>`);

  if (m.help) {
    parts.push(`<p>${esc(m.help.summary)}</p>`);
    parts.push(`<pre class="equations">${esc(m.help.equations.join('\n'))}</pre>`);
    if ((m.help.references ?? []).length) {
      parts.push('<h2>References</h2>');
      parts.push(`<ul>${m.help.references.map((r) => `<li>${esc(r)}</li>`).join('\n')}</ul>`);
      if (m.help.note) parts.push(`<p class="note">${esc(m.help.note)}</p>`);
    }
  }

  parts.push(walkthrough.trim());

  parts.push('<h2>What the application enforces</h2>');
  parts.push(
    '<p>Everything below is generated from the same manifest the application builds ' +
      'this module’s pane from — descriptions, defaults, sources, ranges and ' +
      'pre-run checks here are exactly what the running application enforces.</p>',
  );
  parts.push(`<p>${esc(m.doc)}</p>`);
  if (inputs.length) {
    parts.push('<h2>Input curves</h2>');
    parts.push(
      '<div class="wrap"><table><tr><th>Role</th><th>Description</th><th>Resolves to</th>' +
        `<th>Required</th><th>Notes</th></tr>${inputs.map(inputRow).join('\n')}</table></div>`,
    );
  }
  if (params.length) {
    parts.push('<h2>Parameters</h2>');
    parts.push(
      '<p>Whole-well defaults; per-zone values from the Zones pane take precedence inside ' +
        'their zones (except where a parameter is marked one-value-per-well).</p>',
    );
    for (const a of params) parts.push(renderParam(a));
  }
  if (options.length) {
    parts.push('<h2>Options</h2>');
    for (const a of options) parts.push(renderOption(a));
  }
  if (outputs.length) {
    parts.push('<h2>Output curves</h2>');
    parts.push(
      `<div class="wrap"><table><tr><th>Name</th><th>Description</th></tr>` +
        `${outputs.map(outputRow).join('\n')}</table></div>`,
    );
  }
  return shell(m.title, nav, parts.join('\n'));
}

function renderIndex(specs, hasChapter, nav) {
  const parts = [];
  parts.push('<h1>SandiBumi Guidebook</h1>');
  parts.push(
    '<p>One chapter per petrophysics module — the method, its equations and published ' +
      'references, a step-by-step walkthrough with screenshots, and everything the running ' +
      'application enforces for that module. For the workflow these modules live in, start ' +
      'with the first-hour guide (<code>docs/guide/first-hour.md</code>).</p>',
  );
  for (const cat of orderedCategories(specs)) {
    parts.push(`<h2>${esc(cat)}</h2>`);
    const rows = specs
      .filter((s) => s.category === cat)
      .map((m) =>
        hasChapter.has(m.name)
          ? `<tr><td><a href="${esc(m.name)}.html">${esc(m.title)}</a></td><td><code>${esc(m.name)}</code></td></tr>`
          : `<tr><td class="missing">${esc(m.title)} — chapter not yet written</td><td><code>${esc(m.name)}</code></td></tr>`,
      );
    parts.push(`<div class="wrap"><table><tr><th>Module</th><th>Id</th></tr>${rows.join('\n')}</table></div>`);
  }
  return shell('Index', nav, parts.join('\n'));
}

// ---------------------------------------------------------------------------

const check = process.argv.includes('--check');
const specs = JSON.parse(fs.readFileSync(dumpPath, 'utf8'));

const hasChapter = new Set(
  fs.existsSync(chaptersDir)
    ? fs.readdirSync(chaptersDir).filter((n) => n.endsWith('.html')).map((n) => n.slice(0, -5))
    : [],
);
const unknownChapters = [...hasChapter].filter((n) => !specs.some((m) => m.name === n));
if (unknownChapters.length) {
  console.error(`authored chapters name no known module: ${unknownChapters.join(', ')}`);
  process.exit(1);
}

const pages = new Map();
pages.set('index.html', renderIndex(specs, hasChapter, navHtml(specs, hasChapter, 'index')));
for (const m of specs) {
  if (!hasChapter.has(m.name)) continue;
  const walkthrough = fs
    .readFileSync(path.join(chaptersDir, `${m.name}.html`), 'utf8')
    .replaceAll('\r\n', '\n');
  pages.set(`${m.name}.html`, renderChapter(m, walkthrough, navHtml(specs, hasChapter, m.name)));
}

if (check) {
  const problems = [];
  for (const [name, fresh] of pages) {
    const p = path.join(outDir, name);
    if (!fs.existsSync(p)) {
      problems.push(`missing: docs/guide/book/${name}`);
      continue;
    }
    if (!isGeneratedFileCurrent(fs.readFileSync(p, 'utf8'), fresh)) {
      problems.push(`stale: docs/guide/book/${name}`);
    }
  }
  if (fs.existsSync(outDir)) {
    for (const name of fs.readdirSync(outDir)) {
      if (!name.endsWith('.html')) continue;
      if (!pages.has(name)) problems.push(`orphan (no module/chapter behind it): docs/guide/book/${name}`);
    }
  }
  if (problems.length) {
    console.error('guidebook is out of date:');
    for (const p of problems) console.error(`  ${p}`);
    console.error('regenerate with: node tools/gen-guidebook.mjs');
    process.exit(1);
  }
  console.log(`guidebook current: ${pages.size} page(s)`);
} else {
  fs.mkdirSync(outDir, { recursive: true });
  for (const [name, fresh] of pages) fs.writeFileSync(path.join(outDir, name), fresh);
  for (const name of fs.readdirSync(outDir)) {
    if (name.endsWith('.html') && !pages.has(name)) {
      fs.rmSync(path.join(outDir, name));
      console.log(`removed orphan ${name}`);
    }
  }
  console.log(`wrote ${pages.size} page(s) to docs/guide/book/`);
}
