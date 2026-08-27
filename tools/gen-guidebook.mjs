// Renders the offline HTML guidebook under docs/guide/book/ from
// docs/generated/module_manifests.json (the committed dump of modules::list_modules(),
// with the Help-card registry merged in under `help` — kept fresh by
// manifest_reference_test.rs). One chapter per module that has an authored walkthrough
// in docs/guide/chapters/<module>.html, plus a Contents page of every module.
//
//   node tools/gen-guidebook.mjs           regenerate the book
//   node tools/gen-guidebook.mjs --check   fail (exit 1) if any page is stale/orphaned
//
// The split that keeps this honest: everything FACTUAL (doc, equations, references,
// inputs, parameters with their sources, options, outputs, pre-run checks) renders
// from the dump and cannot drift from the running application; the step-by-step
// walkthrough and screenshots are the hand-written fragment, folded in verbatim.
//
// The shell implements the documentation-tree design from Jauhar's Claude Design
// canvas (SandiBumi Guidebook.dc.html, turn 6, 2026-08-26), built on the Organic
// design-system tokens: a single header row (brand tile, Contents pill, search),
// a 270px disclosure-tree rail of numbered books that collapses to a 48px strip
// (the `[` key toggles it), and the book-contents card table. Books are the
// manifest categories in workflow order; chapters are numbered book.chapter.
// Omitted from the canvas on purpose, because a static offline book has no
// destination for them: the Glossary/Releases nav pills, the "Open app" CTA, the
// dark-theme toggle (the design system ships no dark ramp) and the optional
// reader text-size group.
//
// Offline and self-contained: Figtree is bundled at docs/guide/fonts/ (the same
// variable font the application bundles — field machines are offline, never a
// runtime @import), images relative (../img/ resolves in both the repo layout and
// the bundled guide/ resource layout), and the tree behaviour is a few lines of
// inline script on native <details>.

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

// The launch card echoed in the wide-screen margin. The artwork is the boot
// overlay's own strata drawing (src/bootOverlay.ts) in the brand's colours, and
// the version line reads from package.json the same way the launch screen does,
// so a bump can never leave the book claiming an older build.
const APP_VERSION = JSON.parse(
  fs.readFileSync(path.join(repo, 'package.json'), 'utf8'),
).version;
const LAUNCH_ASIDE = `
<aside class="launch-aside" aria-hidden="true">
  <div class="launch-art">
    <svg viewBox="0 0 320 240" preserveAspectRatio="xMidYMid slice">
      <defs>
        <linearGradient id="bg-strata" x1="0" y1="0" x2="0.35" y2="1">
          <stop offset="0" stop-color="#f5ead8"/><stop offset="1" stop-color="#eadcc2"/>
        </linearGradient>
      </defs>
      <rect width="320" height="240" fill="url(#bg-strata)"/>
      <g fill="none" stroke-linecap="round">
        <path d="M-10 62 Q 80 30 160 58 T 330 44" stroke="#c67139" stroke-width="26" opacity=".85"/>
        <path d="M-10 104 Q 90 74 165 100 T 330 86" stroke="#7a8a5e" stroke-width="18" opacity=".8"/>
        <path d="M-10 138 Q 70 116 158 140 T 330 126" stroke="#c67139" stroke-width="10" opacity=".55"/>
        <path d="M-10 168 Q 100 148 170 172 T 330 158" stroke="#7a8a5e" stroke-width="22" opacity=".55"/>
        <path d="M-10 206 Q 80 186 160 208 T 330 196" stroke="#c67139" stroke-width="16" opacity=".35"/>
      </g>
      <circle cx="228" cy="92" r="7" fill="none" stroke="#3f3428" stroke-width="2.5" opacity=".7"/>
    </svg>
  </div>
  <div class="launch-body">
    <img src="../img/logo-mark.svg" alt="" width="44" height="44" />
    <div class="launch-word">SandiBumi</div>
    <div class="launch-desc">Multi-well petrophysical log analysis</div>
    <div class="launch-foot">SandiBumi 2026 &middot; v${APP_VERSION}<br />&copy; 2026 SandiBumi. All rights reserved.</div>
  </div>
</aside>`;

const esc = (s) =>
  String(s ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');

// Organic design-system tokens (docs/design_organic/ · the Organic DS project's
// styles.css). Values are restated here so the book stays self-contained.
const CSS = `
  :root {
    --color-bg: #f5ead8; --color-surface: #ebddc5; --color-text: #201e1d;
    --color-card: #fffdf8;
    --color-neutral-100: #f9f4ed; --color-neutral-200: #eee7db;
    --color-neutral-300: #dcd3c4; --color-neutral-500: #a19786;
    --color-neutral-700: #645c50; --color-neutral-800: #474238;
    --color-neutral-900: #2e2b25;
    --color-accent-100: #fff2eb; --color-accent-200: #ffe1d0;
    --color-accent-300: #ffc6a5; --color-accent-600: #b2622d;
    --color-accent-700: #8c491a; --color-accent-800: #643312;
    --color-accent-900: #402310;
    --color-accent-2-200: #e1eecc; --color-accent-2-700: #56633f;
    --color-accent-2-800: #3d472b;
    --on-accent-700: #fff8ef;
    --line: rgba(32, 30, 29, .14); --line-strong: rgba(32, 30, 29, .24);
    --row-line: rgba(32, 30, 29, .09);
    --font-body: "Figtree", system-ui, sans-serif;
    --mono: ui-monospace, "Cascadia Code", Consolas, monospace;
  }
  @font-face {
    font-family: "Figtree"; font-style: normal; font-weight: 300 900;
    font-display: swap; src: url("../fonts/figtree-var.woff2") format("woff2");
  }
  * { box-sizing: border-box; }
  body {
    margin: 0; background: var(--color-bg); color: var(--color-text);
    font: 400 15px/1.55 var(--font-body);
  }
  a { color: var(--color-accent-700); text-underline-offset: 3px; }
  :focus { outline: none; }
  :focus-visible { outline: 2px solid var(--color-accent-600); outline-offset: 2px; }

  /* ── header row ── */
  .hdr {
    position: sticky; top: 0; z-index: 20; display: flex; align-items: center;
    gap: 14px; padding: 8px 16px; background: var(--color-card);
    border-bottom: 1px solid rgba(32, 30, 29, .16);
  }
  .hdr-brand { display: flex; align-items: center; gap: 8px; text-decoration: none; }
  .hdr-mark {
    width: 24px; height: 24px; border-radius: 7px; background: var(--color-accent-700);
    display: grid; place-items: center; font: 700 11px/1 var(--font-body);
    color: var(--on-accent-700);
  }
  .hdr-word { font: 700 14px/1 var(--font-body); letter-spacing: -.01em; color: var(--color-text); }
  .hdr-pill {
    display: flex; align-items: center; height: 30px; padding: 0 10px;
    border-radius: 999px; font: 400 12.5px/1 var(--font-body);
    color: var(--color-neutral-800); text-decoration: underline; text-underline-offset: 3px;
  }
  .hdr-pill[aria-current="page"] {
    background: var(--color-accent-100); color: var(--color-accent-800); font-weight: 600;
  }
  .hdr-search {
    flex: 0 1 288px; display: flex; align-items: center; gap: 7px;
    background: var(--color-card); border: 1.5px solid var(--line-strong);
    border-radius: 999px; padding: 0 12px; height: 32px;
  }
  .hdr-search input {
    flex: 1; min-width: 0; border: 0; background: none; font: 400 13px/1 var(--font-body);
    color: var(--color-text); padding: 0;
  }
  .hdr-search input::placeholder { color: var(--color-neutral-800); }
  .hdr-search input:focus { outline: none; }
  kbd {
    font: 600 11px/1 var(--mono); color: var(--color-neutral-800);
    border: 1px solid rgba(32, 30, 29, .3); border-radius: 4px; padding: 2px 4px;
  }
  .hdr-skip {
    margin-left: auto; display: flex; align-items: center; height: 30px; padding: 0 10px;
    border-radius: 999px; font: 600 12px/1 var(--font-body);
    color: var(--color-accent-800); background: var(--color-accent-100);
    text-decoration: underline; text-underline-offset: 3px;
  }

  /* ── frame ── */
  .frame { display: flex; align-items: stretch; min-height: calc(100vh - 49px); }

  /* ── the disclosure-tree rail ── */
  .rail {
    width: 270px; flex: none; background: var(--color-surface);
    border-right: 1px solid var(--line); padding: 12px 0 16px 10px;
    display: flex; flex-direction: column; gap: 2px;
    position: sticky; top: 49px; max-height: calc(100vh - 49px); overflow-y: auto;
  }
  .rail-head {
    display: flex; align-items: center; gap: 6px; margin: 0 8px 8px 8px;
  }
  .rail-head h2 {
    flex: 1; font: 600 11.5px/1 var(--font-body); letter-spacing: .05em;
    text-transform: uppercase; color: var(--color-neutral-800); margin: 0;
  }
  .rail-btn {
    height: 26px; padding: 0 9px; border: 1px solid rgba(32, 30, 29, .24);
    border-radius: 999px; background: var(--color-card);
    font: 600 11px/1 var(--font-body); color: var(--color-neutral-800); cursor: pointer;
  }
  .rail-btn:hover { background: var(--color-accent-100); }
  details.book { margin-right: 10px; }
  details.book > summary {
    display: flex; align-items: center; gap: 9px; min-height: 36px; padding: 2px 10px;
    border-radius: 10px; cursor: pointer; list-style: none;
    border-left: 3px solid transparent;
  }
  details.book > summary::-webkit-details-marker { display: none; }
  details.book > summary:hover { background: var(--color-neutral-200); }
  .bnum { font: 700 12px/1 var(--font-body); width: 20px; flex: none; }
  .btitle { flex: 1; font: 600 13px/1.2 var(--font-body); color: var(--color-neutral-900); }
  .bcount { font: 400 11.5px/1 var(--font-body); color: var(--color-neutral-800); }
  .bchev { flex: none; transition: transform .12s; color: var(--color-neutral-700); }
  details.book[open] > .bchev, details.book[open] > summary .bchev { transform: rotate(90deg); }
  .b-t0 .bnum { color: var(--color-neutral-700); }
  .b-t1 .bnum { color: var(--color-accent-2-700); }
  .b-t2 .bnum { color: var(--color-accent-800); }
  details.book.here > summary {
    background: var(--color-accent-200); border-left-color: var(--color-accent-700);
  }
  details.book.here > summary .btitle { font-weight: 700; color: var(--color-text); }
  .chapters { display: flex; flex-direction: column; gap: 1px; padding: 3px 0 6px 14px; }
  .ch {
    display: flex; align-items: baseline; gap: 8px; padding: 4px 10px;
    border-radius: 9px; font: 600 12.5px/1.35 var(--font-body);
    color: var(--color-text); text-decoration: underline; text-underline-offset: 3px;
  }
  .ch:hover { background: var(--color-neutral-200); }
  .chno { font: 600 11px/1.35 var(--mono); color: var(--color-neutral-700); flex: none; min-width: 26px; }
  .ch.here {
    background: var(--color-accent-700); color: var(--on-accent-700);
    text-decoration: none; font-weight: 700;
  }
  .ch.here .chno { color: var(--on-accent-700); }
  .ch.missing {
    color: var(--color-neutral-700); font-weight: 400; font-style: italic;
    text-decoration: none;
  }
  .rail-hint { margin: 10px 8px 0; font: 400 11.5px/1.4 var(--font-body); color: var(--color-neutral-800); }

  /* ── the collapsed 48px strip ── */
  .strip {
    display: none; width: 48px; flex: none; background: var(--color-surface);
    border-right: 1px solid var(--line); flex-direction: column; align-items: center;
    gap: 6px; padding: 10px 0; position: sticky; top: 49px;
    max-height: calc(100vh - 49px); overflow-y: auto;
  }
  .strip-btn {
    width: 28px; height: 28px; padding: 0; border: 1px solid rgba(32, 30, 29, .24);
    border-radius: 999px; background: var(--color-card); display: grid;
    place-items: center; cursor: pointer; color: var(--color-neutral-800);
  }
  .strip-rule { width: 28px; height: 1px; background: rgba(32, 30, 29, .16); margin: 2px 0; }
  .strip a {
    width: 30px; height: 30px; border-radius: 9px; display: grid; place-items: center;
    font: 700 11.5px/1 var(--font-body); text-decoration: none;
  }
  .strip a.b-t0 { background: var(--color-neutral-200); color: var(--color-neutral-800); }
  .strip a.b-t1 { background: var(--color-accent-2-200); color: var(--color-accent-2-800); }
  .strip a.b-t2 { background: #f8d9cf; color: var(--color-accent-900); }
  .strip a.here {
    width: 32px; height: 32px; border-radius: 10px;
    background: var(--color-accent-700); color: var(--on-accent-700);
  }
  body.rail-collapsed .rail { display: none; }
  body.rail-collapsed .strip { display: flex; }

  /* ── main column ── */
  .main { flex: 1; min-width: 0; padding: 16px 20px 40px; }
  .main-inner { max-width: 900px; }
  .crumb { font: 400 12px/1.4 var(--font-body); color: var(--color-neutral-800); margin: 0 0 8px; }
  .crumb a { color: var(--color-accent-800); }
  .titlerow { display: flex; align-items: baseline; gap: 12px; flex-wrap: wrap; margin: 0 0 6px; }
  h1 {
    font: 700 22px/1.2 var(--font-body); letter-spacing: -.02em; margin: 0;
    color: var(--color-text);
  }
  .chip {
    font: 600 11.5px/1 var(--font-body); color: var(--color-accent-800);
    background: var(--color-accent-100); border: 1px solid var(--color-accent-300);
    padding: 5px 9px; border-radius: 999px; white-space: nowrap;
  }
  .lead { font: 400 13.5px/1.6 var(--font-body); color: var(--color-neutral-800); max-width: 680px; }

  /* ── the content card ── */
  .page {
    background: var(--color-card); border: 1px solid rgba(32, 30, 29, .16);
    border-radius: 14px; padding: 22px 28px 26px; margin-top: 12px;
  }
  .page h2 {
    font: 700 17px/1.25 var(--font-body); letter-spacing: -.01em;
    color: var(--color-accent-800); margin: 30px 0 10px;
    border-bottom: 1px solid var(--row-line); padding-bottom: 5px;
  }
  .page h2:first-child { margin-top: 0; }
  .page h3 { font: 700 14.5px/1.3 var(--font-body); margin: 22px 0 6px; }
  code { font-family: var(--mono); font-size: 12.5px; }
  pre.equations {
    background: var(--color-neutral-100); border: 1px solid var(--line);
    border-radius: 10px; padding: 10px 14px; white-space: pre-wrap;
    overflow-x: auto; font: 400 12.5px/1.55 var(--mono);
  }
  blockquote {
    border-left: 3px solid var(--color-accent-2-700); margin: 12px 0;
    padding: 4px 14px; background: var(--color-neutral-100); border-radius: 0 10px 10px 0;
  }
  figure { margin: 18px 0; }
  figure img { max-width: 100%; border: 1px solid var(--line); border-radius: 10px; display: block; }
  figcaption { font: 400 12px/1.5 var(--font-body); color: var(--color-neutral-800); margin-top: 5px; }
  .wrap { overflow-x: auto; }
  table { border-collapse: collapse; width: 100%; margin: 10px 0; font-size: 13px; }
  th {
    text-align: left; font: 600 11.5px/1.3 var(--font-body); letter-spacing: .05em;
    text-transform: uppercase; color: var(--color-neutral-900);
    background: var(--color-neutral-300); padding: 7px 8px;
    border: 1px solid var(--line);
  }
  td { border: 1px solid var(--row-line); padding: 6px 8px; vertical-align: top; }
  ul { padding-left: 22px; }
  .src { font-size: 12px; color: var(--color-neutral-800); }
  .note { font-size: 13px; color: var(--color-neutral-800); font-style: italic; }
  .absent { font-weight: 600; }

  /* ── the Contents page ── */
  .toc-status { font: 600 12.5px/1.4 var(--font-body); color: var(--color-neutral-900); margin: 0; }
  .toc-book { margin-top: 22px; }
  .toc-bhead { display: flex; align-items: baseline; gap: 10px; margin: 0 0 8px; }
  .toc-bnum {
    width: 30px; height: 30px; border-radius: 9px; display: grid; place-items: center;
    font: 700 12px/1 var(--font-body); flex: none; align-self: center;
  }
  .toc-book .b-t0 { background: var(--color-neutral-200); color: var(--color-neutral-800); }
  .toc-book .b-t1 { background: var(--color-accent-2-200); color: var(--color-accent-2-800); }
  .toc-book .b-t2 { background: #f8d9cf; color: var(--color-accent-900); }
  .toc-bhead h2 { font: 700 17px/1.2 var(--font-body); letter-spacing: -.01em; margin: 0; }
  .toc-bhead .bcount { font: 400 12px/1 var(--font-body); }
  .toc-card {
    background: var(--color-card); border: 1px solid rgba(32, 30, 29, .16);
    border-radius: 14px; overflow: hidden;
  }
  .toc-card table { margin: 0; border-collapse: collapse; }
  .toc-card th { border: none; border-bottom: 1px solid var(--line); }
  .toc-card td { border: none; border-bottom: 1px solid var(--row-line); padding: 9px 8px; }
  .toc-card tr:last-child td { border-bottom: none; }
  .toc-card td:first-child { font: 600 12px/1.2 var(--mono); color: var(--color-neutral-800); padding-left: 14px; width: 44px; }
  .toc-card td a { font: 600 13.5px/1.3 var(--font-body); color: var(--color-text); }
  .toc-card td code { color: var(--color-neutral-800); }
  .toc-card td.missing { color: var(--color-neutral-700); font-style: italic; }

  @media (max-width: 860px) {
    .rail, .strip { display: none !important; }
    .hdr-search { display: none; }
  }

  /* ── the launch card, echoed in the wide-screen margin ──
     On a wide monitor the 900px reading column leaves a bare cream field to its
     right. Rather than leave it empty, the margin carries the same identity card
     the application shows while a project opens: the strata artwork, the mark,
     the wordmark, the edition. Decorative only (aria-hidden, no pointer events),
     fixed so it never enters the reading flow, and shown only when the viewport
     is wide enough that it cannot crowd the column. */
  .launch-aside {
    display: none; position: fixed; right: 32px; top: 50%;
    transform: translateY(-50%); width: 236px; pointer-events: none;
    background: var(--color-card); border: 1px solid rgba(32, 30, 29, .16);
    border-radius: 14px; overflow: hidden;
  }
  @media (min-width: 1600px) { .launch-aside { display: block; } }
  .launch-art svg { display: block; width: 100%; height: auto; }
  .launch-body { padding: 16px 18px 18px; text-align: center; }
  .launch-body img { width: 44px; height: 44px; margin: 0 auto 8px; display: block; }
  .launch-word {
    font: 700 19px/1.2 var(--font-body); letter-spacing: -.02em;
    color: var(--color-accent-700);
  }
  .launch-desc { font: 400 11.5px/1.5 var(--font-body); color: var(--color-neutral-800); margin-top: 3px; }
  .launch-foot {
    font: 400 10.5px/1.6 var(--font-body); color: var(--color-neutral-700);
    margin-top: 12px; border-top: 1px solid var(--row-line); padding-top: 10px;
  }
`;

const JS = `
  (function () {
    var KEY = 'sbguide-rail';
    function collapsed() { return document.body.classList.contains('rail-collapsed'); }
    function setRail(c) {
      document.body.classList.toggle('rail-collapsed', c);
      try { localStorage.setItem(KEY, c ? '1' : '0'); } catch (e) {}
    }
    try { if (localStorage.getItem(KEY) === '1') setRail(true); } catch (e) {}
    var toRail = document.getElementById('rail-collapse');
    var toTree = document.getElementById('rail-expand');
    if (toRail) toRail.addEventListener('click', function () { setRail(true); });
    if (toTree) toTree.addEventListener('click', function () { setRail(false); });
    var closeAll = document.getElementById('rail-closeall');
    if (closeAll) closeAll.addEventListener('click', function () {
      document.querySelectorAll('details.book').forEach(function (d) { d.open = false; });
    });
    document.addEventListener('keydown', function (e) {
      var t = document.activeElement && document.activeElement.tagName;
      if (t === 'INPUT' || t === 'TEXTAREA') return;
      if (e.key === '[') setRail(!collapsed());
      if (e.key === '/') {
        var s = document.getElementById('guide-search');
        if (s) { e.preventDefault(); setRail(false); s.focus(); }
      }
    });
    var search = document.getElementById('guide-search');
    if (search) search.addEventListener('input', function () {
      var q = search.value.trim().toLowerCase();
      document.querySelectorAll('details.book').forEach(function (book) {
        var any = false;
        book.querySelectorAll('.ch').forEach(function (row) {
          var hit = !q || (row.dataset.k || '').indexOf(q) !== -1;
          row.hidden = !hit;
          if (hit) any = true;
        });
        book.hidden = !!q && !any;
        if (q && any) book.open = true;
      });
    });
  })();
`;

function shell(title, nav, body, { isIndex = false } = {}) {
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
<header class="hdr">
<a class="hdr-brand" href="index.html"><span class="hdr-mark">SB</span><span class="hdr-word">Guidebook</span></a>
<nav aria-label="Sections"><a class="hdr-pill" href="index.html"${isIndex ? ' aria-current="page"' : ''}>Contents</a></nav>
<label class="hdr-search" aria-label="Search the guidebook">
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#8c491a" stroke-width="2.75" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="7"></circle><path d="M20 20l-3.5-3.5"></path></svg>
<input id="guide-search" type="search" placeholder="Title, method or mnemonic" />
<kbd aria-hidden="true">/</kbd>
</label>
<a class="hdr-skip" href="#content">Skip to contents</a>
</header>
<div class="frame">
${nav}
<main class="main" id="content">
<div class="main-inner">
${body}
</div>
</main>
</div>
${LAUNCH_ASIDE}
<script>${JS}</script>
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

// The tool books: chapters for the working panes and plot panels that are not
// petrophysics modules and therefore have no manifest to render from. The whole
// campaign is registered up front so the Contents page shows what is coming
// ("chapter not yet written", the same convention the module chapters used);
// an authored file is docs/guide/chapters/<name>.html, names prefixed tool_ to
// keep them out of the module namespace. A tool chapter is entirely hand-written
// (there is no dump to gate it against), so unlike a module chapter it carries
// no generated "What the application enforces" section.
const TOOL_BOOKS = [
  { cat: 'Plots & Views', tools: [
    ['tool_log_view', 'Log View'],
    ['tool_histogram', 'Histogram'],
    ['tool_crossplot', 'Crossplot'],
    ['tool_pickett', 'Pickett'],
    ['tool_correlation', 'Correlation'],
    ['tool_vega', 'Vega Chart'],
  ]},
  { cat: 'Data & Sets', tools: [
    ['tool_intake', 'Intake (import any table)'],
    ['tool_statistics', 'Statistics (tables)'],
    ['tool_reframe', 'Reframe (resample a set)'],
    ['tool_data_sets', 'Data Sets (deliveries)'],
    ['tool_versions', 'Versions (labels + purge)'],
    ['tool_inspector', 'Inspector'],
    ['tool_db_inspector', 'Database Inspector'],
    ['tool_sql_query', 'SQL Query'],
  ]},
  { cat: 'Batch & Field', tools: [
    ['tool_workflow_builder', 'Workflow Builder'],
    ['tool_monte_carlo', 'Monte Carlo'],
    ['tool_pay_summary', 'Cutoffs & Pay Summary'],
    ['tool_cutoff_sensitivity', 'Cutoff Sensitivity'],
    ['tool_field_dashboard', 'Field Dashboard'],
    ['tool_field_map', 'Field Map'],
    ['tool_results_qc', 'Results QC (Sw spread)'],
  ]},
  { cat: 'Fits & Analysis', tools: [
    ['tool_sandimin', 'SandiMin Solver'],
    ['tool_ml', 'Machine Learning'],
    ['tool_shf_fit', 'SHF Fit (Cuddy / FOIL)'],
    ['tool_thomeer_fit', 'Pc Fit (Thomeer)'],
    ['tool_hfu', 'HFU Clustering (FZI)'],
    ['tool_lorenz', 'Lorenz Plot (flow units)'],
    ['tool_facies_tie', 'Facies Tie-in'],
    ['tool_fluid_contacts', 'Fluid Contacts'],
  ]},
  { cat: 'Core & Petrography', tools: [
    ['tool_depth_reg', 'Register Core Depth'],
    ['tool_core_photos', 'Condition Core Photos'],
    ['tool_photo_log', 'Photo Log (core → curves)'],
    ['tool_condition_plates', 'Condition Plates'],
    ['tool_plate_details', 'Plate Details'],
    ['tool_pore_area', 'Pore Area (thin sections)'],
    ['tool_mineral_classifier', 'Mineral Classifier'],
    ['tool_plug_qc', 'Plug QC'],
  ]},
  { cat: 'Workspace & Project', tools: [
    ['tool_zones', 'Zones'],
    ['tool_tops_editor', 'Tops & Autocorrelation'],
    ['tool_equation_editor', 'Equation Editor'],
    ['tool_sessions_layouts', 'Sessions & Layouts'],
    ['tool_processing_history', 'Processing History'],
    ['tool_composite', 'Composite Log'],
    ['tool_report', 'Report & Deliverables'],
    ['tool_diagnostics', 'Diagnostics'],
  ]},
];
// The front book: the first-hour walkthrough, ahead of every module book because
// it is the path the module chapters assume you have walked. Rendered exactly
// like a tool chapter (hand-authored, no manifest), with its own chip text.
const FRONT_BOOKS = [
  { cat: 'Getting Started', chip: 'walkthrough', tools: [
    ['first_hour', 'Your First Hour'],
  ]},
];

const TOOL_NAMES = new Set(
  [...FRONT_BOOKS, ...TOOL_BOOKS].flatMap((b) => b.tools.map(([name]) => name)),
);

function orderedCategories(specs) {
  const present = [...new Set(specs.map((m) => m.category))];
  return [
    ...CATEGORY_ORDER.filter((c) => present.includes(c)),
    ...present.filter((c) => !CATEGORY_ORDER.includes(c)),
  ];
}

/** Books = categories in reading order, then the tool books; chapters numbered
 *  book.chapter. A tool chapter's `m` is a synthesized minimal spec (name, title,
 *  category, isTool) so the nav and index render it exactly like a module row. */
function bookModel(specs, hasChapter) {
  const handBook = (b, i) => ({
    num: String(i + 1).padStart(2, '0'),
    tint: `b-t${i % 3}`,
    cat: b.cat,
    chapters: b.tools.map(([name, title], j) => ({
      no: `${i + 1}.${j + 1}`,
      m: { name, title, category: b.cat, isTool: true, chip: b.chip },
      written: hasChapter.has(name),
    })),
  });
  const front = FRONT_BOOKS.map((b, i) => handBook(b, i));
  const moduleBooks = orderedCategories(specs).map((cat, k) => {
    const i = front.length + k;
    return {
      num: String(i + 1).padStart(2, '0'),
      tint: `b-t${i % 3}`,
      cat,
      chapters: specs
        .filter((s) => s.category === cat)
        .map((m, j) => ({ no: `${i + 1}.${j + 1}`, m, written: hasChapter.has(m.name) })),
    };
  });
  const base = front.length + moduleBooks.length;
  const toolBooks = TOOL_BOOKS.map((b, i) => handBook(b, base + i));
  return [...front, ...moduleBooks, ...toolBooks];
}

const CHEV =
  '<svg class="bchev" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 6l6 6-6 6"></path></svg>';

/** The left rail (disclosure tree) plus its collapsed 48px strip. `current` is a
 *  module name or 'index'; the current book opens, the current chapter fills. */
function navHtml(books, current) {
  const total = books.reduce((n, b) => n + b.chapters.length, 0);
  const tree = [];
  tree.push('<nav class="rail" id="rail" aria-label="Guidebook contents">');
  tree.push('<div class="rail-head">');
  tree.push(`<h2>${books.length} books · ${total} chapters</h2>`);
  tree.push('<button type="button" class="rail-btn" id="rail-closeall">Collapse all</button>');
  tree.push(
    '<button type="button" class="rail-btn" id="rail-collapse" aria-label="Collapse the contents rail" title="Collapse the rail ([)">‹</button>',
  );
  tree.push('</div>');
  for (const b of books) {
    const isHere = b.chapters.some((c) => c.m.name === current);
    tree.push(
      `<details class="book ${b.tint}${isHere ? ' here' : ''}"${isHere ? ' open' : ''}>` +
        `<summary><span class="bnum">${b.num}</span><span class="btitle">${esc(b.cat)}</span>` +
        `<span class="bcount">${b.chapters.length}</span>${CHEV}</summary>`,
    );
    tree.push('<div class="chapters">');
    for (const c of b.chapters) {
      const key = esc(`${c.m.title} ${c.m.name}`.toLowerCase());
      if (!c.written) {
        tree.push(
          `<span class="ch missing" data-k="${key}"><span class="chno">${c.no}</span>${esc(c.m.title)}</span>`,
        );
      } else if (c.m.name === current) {
        tree.push(
          `<a class="ch here" data-k="${key}" href="${esc(c.m.name)}.html" aria-current="page"><span class="chno">${c.no}</span>${esc(c.m.title)}</a>`,
        );
      } else {
        tree.push(
          `<a class="ch" data-k="${key}" href="${esc(c.m.name)}.html"><span class="chno">${c.no}</span>${esc(c.m.title)}</a>`,
        );
      }
    }
    tree.push('</div></details>');
  }
  tree.push('<p class="rail-hint"><kbd>[</kbd> hides the rail · <kbd>/</kbd> searches</p>');
  tree.push('</nav>');

  const strip = [];
  strip.push('<nav class="strip" aria-label="Guidebook contents, collapsed">');
  strip.push(
    '<button type="button" class="strip-btn" id="rail-expand" aria-label="Expand the contents rail" title="Expand the rail ([)">' +
      '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 6l6 6-6 6"></path></svg></button>',
  );
  strip.push('<span class="strip-rule"></span>');
  for (const b of books) {
    const isHere = b.chapters.some((c) => c.m.name === current);
    strip.push(
      `<a class="${b.tint}${isHere ? ' here' : ''}" href="index.html#b${b.num}"` +
        `${isHere ? ' aria-current="true"' : ''} aria-label="Book ${b.num}, ${esc(b.cat)}">${b.num}</a>`,
    );
  }
  strip.push('</nav>');
  return tree.join('\n') + '\n' + strip.join('\n');
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

function renderChapter(m, chapterNo, walkthrough, nav) {
  const byKind = (k) => m.args.filter((a) => a.kind === k);
  const params = byKind('param');
  const options = [...byKind('option'), ...byKind('text')];
  const inputs = byKind('log_in');
  const outputs = byKind('log_out');
  const bookNum = chapterNo.split('.')[0].padStart(2, '0');

  const parts = [];
  parts.push(
    `<p class="crumb"><a href="index.html">Guidebook</a> · <a href="index.html#b${bookNum}">${bookNum} — ${esc(m.category)}</a></p>`,
  );
  parts.push(
    `<div class="titlerow"><h1>${esc(m.title)}</h1>` +
      `<span class="chip">${chapterNo} · <code>${esc(m.name)}</code></span></div>`,
  );

  const inner = [];
  if (m.help) {
    inner.push(`<p>${esc(m.help.summary)}</p>`);
    inner.push(`<pre class="equations">${esc(m.help.equations.join('\n'))}</pre>`);
    if ((m.help.references ?? []).length) {
      inner.push('<h2>References</h2>');
      inner.push(`<ul>${m.help.references.map((r) => `<li>${esc(r)}</li>`).join('\n')}</ul>`);
      if (m.help.note) inner.push(`<p class="note">${esc(m.help.note)}</p>`);
    }
  }

  inner.push(walkthrough.trim());

  inner.push('<h2>What the application enforces</h2>');
  inner.push(
    '<p>Everything below is generated from the same manifest the application builds ' +
      'this module’s pane from — descriptions, defaults, sources, ranges and ' +
      'pre-run checks here are exactly what the running application enforces.</p>',
  );
  inner.push(`<p>${esc(m.doc)}</p>`);
  if (inputs.length) {
    inner.push('<h2>Input curves</h2>');
    inner.push(
      '<div class="wrap"><table><tr><th>Role</th><th>Description</th><th>Resolves to</th>' +
        `<th>Required</th><th>Notes</th></tr>${inputs.map(inputRow).join('\n')}</table></div>`,
    );
  }
  if (params.length) {
    inner.push('<h2>Parameters</h2>');
    inner.push(
      '<p>Whole-well defaults; per-zone values from the Zones pane take precedence inside ' +
        'their zones (except where a parameter is marked one-value-per-well).</p>',
    );
    for (const a of params) inner.push(renderParam(a));
  }
  if (options.length) {
    inner.push('<h2>Options</h2>');
    for (const a of options) inner.push(renderOption(a));
  }
  if (outputs.length) {
    inner.push('<h2>Output curves</h2>');
    inner.push(
      `<div class="wrap"><table><tr><th>Name</th><th>Description</th></tr>` +
        `${outputs.map(outputRow).join('\n')}</table></div>`,
    );
  }
  parts.push(`<div class="page">${inner.join('\n')}</div>`);
  return shell(m.title, nav, parts.join('\n'));
}

/** A tool chapter is the walkthrough alone: no manifest exists to generate the
 *  factual sections from, so nothing here pretends to be generated. */
function renderToolChapter(t, chapterNo, walkthrough, nav) {
  const bookNum = chapterNo.split('.')[0].padStart(2, '0');
  const parts = [];
  parts.push(
    `<p class="crumb"><a href="index.html">Guidebook</a> · <a href="index.html#b${bookNum}">${bookNum} — ${esc(t.category)}</a></p>`,
  );
  parts.push(
    `<div class="titlerow"><h1>${esc(t.title)}</h1>` +
      `<span class="chip">${chapterNo} · ${esc(t.chip ?? 'pane')}</span></div>`,
  );
  parts.push(`<div class="page">${walkthrough.trim()}</div>`);
  return shell(t.title, nav, parts.join('\n'));
}

function renderIndex(books, nav) {
  const total = books.reduce((n, b) => n + b.chapters.length, 0);
  const written = books.reduce((n, b) => n + b.chapters.filter((c) => c.written).length, 0);
  const parts = [];
  parts.push('<p class="crumb">Guidebook</p>');
  const status =
    written === total ? `${total} chapters` : `${total} chapters · ${written} written so far`;
  parts.push('<div class="titlerow"><h1>Contents</h1>' +
    `<p class="toc-status">${status}</p></div>`);
  parts.push(
    '<p class="lead">One chapter per petrophysics module — the method, its equations and published ' +
      'references, a step-by-step walkthrough with screenshots, and everything the running ' +
      'application enforces for that module — followed by the tool books: the plot panels, ' +
      'data tools, batch machinery and working panes the modules are driven from. For the ' +
      'workflow these all live in, start with ' +
      '<a href="first_hour.html">Your First Hour</a> — book 01.</p>',
  );
  for (const b of books) {
    parts.push(`<section class="toc-book" id="b${b.num}">`);
    parts.push(
      `<div class="toc-bhead"><span class="toc-bnum ${b.tint}">${b.num}</span>` +
        `<h2>${esc(b.cat)}</h2><span class="bcount">${b.chapters.length} chapter${b.chapters.length === 1 ? '' : 's'}</span></div>`,
    );
    const rows = b.chapters.map((c) =>
      c.written
        ? `<tr><td>${c.no}</td><td><a href="${esc(c.m.name)}.html">${esc(c.m.title)}</a></td><td><code>${esc(c.m.name)}</code></td></tr>`
        : `<tr><td>${c.no}</td><td class="missing">${esc(c.m.title)} — chapter not yet written</td><td><code>${esc(c.m.name)}</code></td></tr>`,
    );
    parts.push(
      `<div class="toc-card"><table><thead><tr><th style="width:44px">No.</th><th>Chapter</th>` +
        `<th style="width:170px">Module id</th></tr></thead><tbody>${rows.join('\n')}</tbody></table></div>`,
    );
    parts.push('</section>');
  }
  return shell('Contents', nav, parts.join('\n'), { isIndex: true });
}

// ---------------------------------------------------------------------------

const check = process.argv.includes('--check');
const specs = JSON.parse(fs.readFileSync(dumpPath, 'utf8'));

const hasChapter = new Set(
  fs.existsSync(chaptersDir)
    ? fs.readdirSync(chaptersDir).filter((n) => n.endsWith('.html')).map((n) => n.slice(0, -5))
    : [],
);
const unknownChapters = [...hasChapter].filter(
  (n) => !specs.some((m) => m.name === n) && !TOOL_NAMES.has(n),
);
if (unknownChapters.length) {
  console.error(
    `authored chapters name no known module or registered tool: ${unknownChapters.join(', ')}`,
  );
  process.exit(1);
}

const books = bookModel(specs, hasChapter);
const chapterNo = new Map();
for (const b of books) for (const c of b.chapters) chapterNo.set(c.m.name, c.no);

const pages = new Map();
pages.set('index.html', renderIndex(books, navHtml(books, 'index')));
for (const m of specs) {
  if (!hasChapter.has(m.name)) continue;
  const walkthrough = fs
    .readFileSync(path.join(chaptersDir, `${m.name}.html`), 'utf8')
    .replaceAll('\r\n', '\n');
  pages.set(
    `${m.name}.html`,
    renderChapter(m, chapterNo.get(m.name), walkthrough, navHtml(books, m.name)),
  );
}
for (const b of books) {
  for (const c of b.chapters) {
    if (!c.m.isTool || !c.written) continue;
    const walkthrough = fs
      .readFileSync(path.join(chaptersDir, `${c.m.name}.html`), 'utf8')
      .replaceAll('\r\n', '\n');
    pages.set(
      `${c.m.name}.html`,
      renderToolChapter(c.m, c.no, walkthrough, navHtml(books, c.m.name)),
    );
  }
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
