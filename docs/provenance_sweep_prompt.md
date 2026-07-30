# SandiBumi Provenance & Exposure Sweep Prompt

### Finds, classifies, and routes everything in the repo that could create contractual, client-confidentiality, or third-party-IP exposure — and applies only the fixes that are unambiguously safe.

**Boundary vs the other prompts** (the table in `stewardship_prompt.md` is authoritative — add a
row for this one): `engineering_review_prompt.md` sweeps *behaviour*, `qc_audit_prompt_template.md`
audits *one tool's correctness*, `stewardship_prompt.md` audits *structure*. This one audits
**provenance** — where every value, name, and file in the repo came from, and whether shipping it
is safe.

**How to use:**
1. Paste everything between `=== PROMPT START ===` / `=== PROMPT END ===` into Claude Code at the
   repo root.
2. Run **Stage 0–2 first** (ground, sweep, classify) and read the register before approving fixes.
3. **Stage 3 applies edits — approve it explicitly.** Stage 4 writes the lawyer packet.
4. Re-run before any release, and after any increment that adds defaults, fixtures, or docs.

**Run cadence:** before first distribution, before each release, and after any work that touches
module manifests, test fixtures, or `docs/`.

---

=== PROMPT START ===

# ROLE & MISSION

You are performing a **provenance and exposure sweep** of SandiBumi — a commercial petrophysics
product built by a solo developer (Jauhar) who has run 50+ consulting studies for Pertamina and
other K3S under confidentiality agreements, and who is preparing to sell the software.

Your mission: find every artifact in this repository whose *origin* could create a problem —
client-confidential material, client-derived values, third-party marks, vendor-derived reference
data, or licence obligations — then **classify, safely fix, and route** them.

You are not a lawyer and you render **no legal conclusions**. You produce evidence and a decision
packet.

# HARD RULES — violating any of these makes the sweep worse than not running it

1. **Never delete an honest provenance record.** If a file header says *"digitized from
   Schlumberger Log Interpretation Charts, 2013"*, that statement is an asset, not a liability —
   removing it does not resolve the licensing question, it destroys the record and looks like
   concealment. **Attribution comes out only when the underlying asset comes out.**
2. **Never silently change a number that affects results.** Any default, endpoint, or constant you
   alter is a behaviour change. Flag it, update the tests that assert it, and say so explicitly in
   the report.
3. **Never write a client identifier into a committed file** — including into this sweep's own
   outputs. Discovered client tokens (operator, block, field, project, well, personnel names) go
   to `docs/commercial/` **only**, which is gitignored.
4. **Never rewrite git history.** Removing a file from the working tree does not remove it from
   history. Surface that as a decision for Jauhar with options; do not execute it.
5. **Distinguish shipped from test.** A value in a module manifest ships to every user. The same
   value in a test asserts the math. These have different exposure and different correct answers.
6. **Do not widen scope.** SandiBumi only. Note SegaraBumi follow-ups; do not act on them.

# SCOPE

`src-tauri/src/`, `src/`, `tests/`, `docs/`, `tools/`, root config, `README.md`, `CLAUDE.md`,
`ROADMAP.md`, `REVIEW.md`, commit messages. Exclude `docs/commercial/` (already local-only),
`node_modules/`, `target/`, `.db-backups/`, `Prompt/`.

---

# STAGE 0 — Ground

Read before sweeping:
- `docs/IP_PROVENANCE.md` — the existing register (vendor IP only; **it has no client-data tier**)
- `docs/PRD.md` §9 risk register — R1/R2/R3 are the already-routed vendor items
- `.gitignore` — what is already excluded and why
- `CLAUDE.md` — the shipping conventions, including how third-party product names are already
  handled in prose

State what the register already covers so you do not re-litigate settled items.

# STAGE 1 — Sweep

Run each category. Report **file:line** for every hit. Use ripgrep; these are seeds, not limits —
follow what you find.

### 1a. Client-derived values
Numeric defaults, endpoints, cutoffs, or calibrations traceable to a specific asset. Signals: a
comment or doc string naming a block, field, operator, study, or well count next to a constant.
```
rg -n "calibration|regional|standard from|wells:|field standard|tuned|derived from" src-tauri/src src
```
For every module manifest (`ModuleSpec` / `param(...)` defaults), ask: *could this number only have
come from someone's wells?*

### 1b. Client identifiers
Operator, block, field, project, study, well, and personnel names anywhere in the tree — including
absolute paths in tests, sample data, comments, and commit messages.
```
rg -n "D:\\\\01\.|\\\\Work\\\\|Delivery Data" src-tauri src tools tests
git log --format="%s%n%b" | rg -n "<tokens discovered above>"
```
**Derive the token list from what you find — do not hardcode it here, and write it only to
`docs/commercial/CLIENT_TOKENS.local.md`.**

### 1c. Client data files
Any `.las`, `.dlis`, `.csv`, `.xlsx`, image, or project file in the tree that is not demonstrably
synthetic. Cross-check against `tools/make_example_data.py` output and the `.gitignore` fixture
exceptions. Report size, path, and whether it is tracked by git.

### 1d. Vendor-derived reference data
Chart digitizations, mineral endpoints, alias catalogs, parameter defaults sourced from a vendor
install. Most are already registered (R1/R2) — report only additions or drift since the register
was written.

### 1e. Third-party marks
Vendor and product names in shipped code, identifiers, UI strings, docs, and commit messages.
Include theme ids, env var names, bundle ids, and anything a trademark holder would recognise as
their mark.
```
rg -ni "schlumberger|halliburton|slb|techlog|geolog|aspen|petrel|<others found>" src src-tauri docs README.md
```
Note which are **client-branded palettes** (a deliberate feature) versus incidental.

### 1f. Method attribution
Every implemented method should cite the primary publication, not a vendor implementation. Flag
any method whose only citation is a vendor manual, install tree, or decompiled source.

### 1g. Dependency licence obligations
Enumerate crates and npm packages with attribution requirements or copyleft terms. Report whether
a `NOTICE`/`THIRD-PARTY-LICENSES` file exists and whether it is complete.
```
cargo tree --prefix none --format "{p} {l}" | sort -u
```

### 1h. Stale brand and path leakage
Old product names and machine-specific absolute paths that will break or embarrass on another
machine (e.g. superseded env var names, prior branding, personal directory paths).

# STAGE 2 — Classify

One row per finding:

| # | Category | file:line | What it is | Shipped or test? | Tier | Proposed action |
|---|---|---|---|---|---|---|

**Tiers:**
- **A — Safe to fix now.** No legal question; the fix is mechanical and loses nothing. *(Absolute
  client paths in tests, stale brand strings, machine-specific paths, missing NOTICE entries.)*
- **B — Fix with a behaviour note.** Mechanically safe but changes what a user sees or a module
  computes. Requires a test update and an explicit callout. *(Field-specific values shipping as
  manifest defaults.)*
- **C — Route, do not touch.** A real legal question. Document precisely; change nothing. *(Chart
  data, vendor endpoint defaults, third-party marks, client-derived analytical work product.)*
- **D — Decision required.** Options exist, all with consequences; Jauhar chooses. *(Git history
  containing client tokens; whether a client-branded theme keeps its id.)*

# STAGE 3 — Apply (requires explicit approval)

Apply **Tier A** and **Tier B** only.

- Tier B rule: when removing a field-specific default, replace it with a neutral value **and**
  rewrite the surrounding doc string to describe the *method* rather than one field's calibration.
  A calibration from one basin is the wrong default in another — this improves the product, not
  just the exposure.
- Preserve the real values as a **local preset** outside the repo where they are still useful.
- Update every test that asserts a changed value. A test asserting the math with a real-world value
  is usually legitimate — decide per case and record the reasoning.
- Touch **nothing** in Tier C or D.

# STAGE 4 — Route

Update `docs/IP_PROVENANCE.md`: add a **client-data tier** alongside the existing vendor tiers,
using the same treatment — what it is, the derivation path, the precise question, and costed
fallbacks.

Write `docs/commercial/LAWYER_PACKET.local.md` (gitignored): every Tier C and D item as a single
numbered question a lawyer can answer without reading the codebase, with the evidence attached.
Merge with the questions already open in `docs/commercial/PLAN.md` §8 so it is **one engagement**,
not several.

# STAGE 5 — Verify and report

- `npx tsc --noEmit` and `cd src-tauri && cargo check`
- If any Tier B change landed: `powershell -ExecutionPolicy Bypass -File tools\check.ps1`
- Report test results honestly, including failures.

**Final report:**
1. Counts by tier, and what was applied versus routed.
2. Every behaviour change, with the old and new values.
3. The lawyer packet's question count.
4. **What this sweep could not see** — git history contents, binaries in `target/`, anything
   outside the repo, and any judgment that requires reading a signed agreement.
5. A `REVIEW.md` entry for the applied changes.

# OUTPUT CONTRACT

- No client identifier appears in any committed file, including your own outputs.
- No Tier C or D item was modified.
- No provenance statement was removed while its underlying asset remained.
- Every changed number is listed with its test update.

=== PROMPT END ===

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
