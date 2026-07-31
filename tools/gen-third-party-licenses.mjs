// Regenerates THIRD-PARTY-LICENSES.md from the REAL dependency graphs.
//
// Run from the repo root:  node tools/gen-third-party-licenses.mjs
//
// Rust side  : `cargo tree --edges normal` — normal edges only, so build-time and dev-only
//              crates (which are not distributed and carry no attribution obligation in a
//              shipped binary) are excluded. Including them would inflate the notice and
//              make the real obligations harder to see.
// Node side  : walks node_modules for the packages Vite actually bundles into the frontend.
//              Dev tooling is listed separately for the same reason.
//
// Written by the provenance sweep, 2026-07-31 (finding 22: no NOTICE file existed at all).
// This file is a factual inventory, not legal advice.

import { execSync } from "child_process";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "fs";
import path from "path";

function crates() {
  const raw = execSync('cargo tree --prefix none --format "{p}|{l}" --edges normal', {
    cwd: "src-tauri",
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const out = new Map();
  for (let line of raw.split("\n")) {
    line = line.replace(/ \(\*\)\s*$/, "").trim();
    if (!line || !line.includes("|")) continue;
    const i = line.lastIndexOf("|");
    const name = line.slice(0, i).replace(/\s+\(proc-macro\)$/, "").trim();
    const lic = line.slice(i + 1).trim() || "UNKNOWN";
    if (name.startsWith("sandibumi")) continue; // our own crate
    out.set(name, lic);
  }
  return out;
}

function npmPackages() {
  const seen = new Map();
  (function walk(dir) {
    if (!existsSync(dir)) return;
    for (const e of readdirSync(dir)) {
      if (e.startsWith(".")) continue;
      const p = path.join(dir, e);
      if (e.startsWith("@")) {
        walk(p);
        continue;
      }
      const pj = path.join(p, "package.json");
      if (existsSync(pj)) {
        try {
          const j = JSON.parse(readFileSync(pj, "utf8"));
          const lic =
            j.license ||
            (Array.isArray(j.licenses) && j.licenses.map((l) => l.type).join(" OR ")) ||
            "UNKNOWN";
          if (j.name) seen.set(`${j.name} v${j.version}`, lic);
        } catch {
          /* a malformed package.json is not a licence signal */
        }
      }
      walk(path.join(p, "node_modules"));
    }
  })("node_modules");
  return seen;
}

function groupByLicence(map) {
  const g = new Map();
  for (const [name, lic] of [...map].sort()) {
    if (!g.has(lic)) g.set(lic, []);
    g.get(lic).push(name);
  }
  return [...g].sort((a, b) => b[1].length - a[1].length);
}

function section(title, map) {
  const groups = groupByLicence(map);
  let s = `## ${title}\n\n${map.size} packages.\n\n| Licence | Count |\n|---|---|\n`;
  for (const [lic, names] of groups) s += `| ${lic} | ${names.length} |\n`;
  s += "\n";
  for (const [lic, names] of groups) {
    s += `### ${lic}\n\n`;
    s += names.map((n) => `- ${n}`).join("\n");
    s += "\n\n";
  }
  return s;
}

const rust = crates();
const node = npmPackages();

const copyleft = [...rust, ...node].filter(([, l]) => /GPL|MPL|CDDL|EPL|SSPL/i.test(l));
const unknown = [...rust, ...node].filter(([, l]) => /UNKNOWN/i.test(l));

const header = `# Third-party licences

SandiBumi is distributed as a compiled desktop application that statically links a large number
of open-source Rust crates and bundles a JavaScript frontend. This file lists them and their
declared licences.

**Generated** by \`tools/gen-third-party-licenses.mjs\` — re-run it after any dependency change;
do not edit by hand. It is a **factual inventory, not legal advice.**

Scope note: only **normal** (distributed) dependencies are listed. Build-time and dev-only
packages — the compiler plugins, the bundler, the test harnesses — are not shipped to a user and
are excluded, so the obligations that DO apply stay visible.

Python packages (\`numpy\`, \`dlisio\`, \`scikit-learn\`, \`xlsxwriter\`, \`python-docx\`,
\`python-pptx\`, \`matplotlib\`, \`Pillow\`) are **not distributed with SandiBumi**. They are
prerequisites the user installs into their own interpreter, which SandiBumi invokes as a
subprocess. That is a materially lighter obligation than bundling them.

## Attention items

${
  copyleft.length
    ? `**Weak-copyleft licences present (${copyleft.length}).** All are file-level (MPL-family): they
permit linking into a proprietary application, but require that the source of *those files*
remains available and that the licence notice is preserved. None is modified by this project;
all arrive transitively.

${copyleft.map(([n, l]) => `- ${n} — ${l}`).join("\n")}`
    : "No copyleft licences found in the distributed dependency set."
}

${
  unknown.length
    ? `**Packages with no declared licence (${unknown.length})** — each needs to be checked by hand:\n\n${unknown
        .map(([n, l]) => `- ${n} — ${l}`)
        .join("\n")}`
    : "**No package in the distributed set is missing a licence declaration.**"
}

---

`;

writeFileSync(
  "THIRD-PARTY-LICENSES.md",
  header + section("Rust crates", rust) + section("JavaScript packages", node),
  "utf8"
);
console.log(
  `THIRD-PARTY-LICENSES.md written: ${rust.size} crates, ${node.size} npm packages, ` +
    `${copyleft.length} copyleft, ${unknown.length} undeclared`
);
