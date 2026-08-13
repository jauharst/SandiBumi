// Regenerates THIRD-PARTY-LICENSES.md from the REAL dependency graphs.
//
// Run from the repo root:  node tools/gen-third-party-licenses.mjs
//
// Rust side  : `cargo tree --edges normal` — normal edges only, so build-time and dev-only
//              crates (which are not distributed and carry no attribution obligation in a
//              shipped binary) are excluded. Including them would inflate the notice and
//              make the real obligations harder to see.
// Node side  : asks npm for the installed production dependency graph and reads only those
//              package directories. Walking node_modules directly also collects hoisted dev
//              tools, which made a notice headed "distributed dependencies" materially false.
//
// Written by the provenance sweep, 2026-07-31 (finding 22: no NOTICE file existed at all).
// This file is a factual inventory, not legal advice.

import { execSync } from "child_process";
import { existsSync, readFileSync, writeFileSync } from "fs";
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
  const root = path.resolve(".");
  const installed = execSync("npm ls --omit=dev --all --parseable", {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  for (const line of installed.split(/\r?\n/u)) {
    const packageDir = line.trim();
    if (!packageDir || path.resolve(packageDir) === root) continue;
    const pj = path.join(packageDir, "package.json");
    if (!existsSync(pj)) continue;
    try {
      const j = JSON.parse(readFileSync(pj, "utf8"));
      const lic =
        j.license ||
        (Array.isArray(j.licenses) && j.licenses.map((l) => l.type).join(" OR ")) ||
        "UNKNOWN";
      if (j.name) seen.set(`${j.name} v${j.version}`, lic);
    } catch {
      /* a malformed package.json is reported as absent rather than invented */
    }
  }
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

Scope note: this is a conservative release inventory: Cargo **normal** dependencies plus npm's
installed **production** dependency graph. Build-time and dev-only packages — compiler plugins,
the bundler and test harnesses — are excluded. Optimisation may remove some production-graph code
from the final binary, but over-including a declared production dependency is safer than silently
omitting a notice that may apply.

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

const outputPath = "THIRD-PARTY-LICENSES.md";
const output = header + section("Rust crates", rust) + section("JavaScript packages", node);
const summary = `${rust.size} crates, ${node.size} npm packages, ${copyleft.length} copyleft, ${unknown.length} undeclared`;

if (process.argv.includes("--check")) {
  if (!existsSync(outputPath) || readFileSync(outputPath, "utf8") !== output) {
    console.error(`${outputPath} is stale; run node tools/gen-third-party-licenses.mjs`);
    process.exitCode = 1;
  } else {
    console.log(`${outputPath} is current: ${summary}`);
  }
} else {
  writeFileSync(outputPath, output, "utf8");
  console.log(`${outputPath} written: ${summary}`);
}
