# Gate 1 Baseline Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a versioned, machine-validated takeover tracker and capture the current repository, PRD, branch, verification and customer-claim baseline without changing product behavior.

**Architecture:** A dependency-free Node ESM tool parses the consolidated PRD index, validates an RFC 4180 CSV ledger, emits reproducible audit summaries and checks the human dashboard's required fields. The tracker separates original chapter evidence from reverified as-built status and pilot disposition. Existing `REVIEW.md` and its generated capability matrix remain the sole manual-field-evidence lane; Gate 1 links to them instead of creating a second verification system.

**Tech Stack:** Node.js ESM, built-in `node:test`, PowerShell 5.1, Git, Markdown, RFC 4180 CSV, existing TypeScript/Rust repository gates.

## Global Constraints

- This foundation plan MUST NOT modify production Rust or TypeScript behavior.
- This plan MUST NOT edit `docs/PRD_v2/**`, `REVIEW.md`, `verification/capabilities.json` or `docs/VERIFICATION_MATRIX.md`.
- The PRD chapter status is preserved verbatim; it is never copied into `as_built_status` without live source-and-test adjudication.
- Petrophysical parameters, defaults, endpoints, cutoffs, units and depth references remain cited or absent. This plan chooses none.
- Automated tests, desktop automation and manual/field evidence remain different evidence classes.
- The existing dirty `src-tauri/Cargo.toml` state and untracked `claude-usage-widget/`, `src-tauri/target-ignored-tests/` and `src-tauri/target-plt-p0/` paths remain unstaged and unmodified.
- Code navigation uses the codebase index first when it is callable. In this session it is not callable, so targeted filesystem search is the explicit fallback.
- Historical counts are dated snapshots. The tracker re-measures them from the checked-out tree.
- No branch is merged, deleted, rebased or pushed during this plan.
- Every commit stages exact owned paths; never use `git add -A` or `git add .`.
- A nonzero baseline gate is recorded before repair and stops this foundation plan after the evidence commit. It is not hidden or made green by weakening a test.

---

## File Structure

### New files

- `tools/takeover-ledger.mjs` — dependency-free parser, CSV validator, audit generator and CLI.
- `tools/takeover-ledger.test.mjs` — named unit and checkout-integration tests for the tracker contracts.
- `docs/takeover/requirements.csv` — authoritative one-row-per-requirement evidence and release-disposition ledger.
- `docs/takeover/STATUS.md` — one-minute human dashboard.
- `docs/takeover/DECISIONS.md` — append-only product-owner decision and blocker register.
- `docs/takeover/evidence/2026-08-10-baseline.md` — dated worktree, gate and count receipt.
- `docs/takeover/evidence/branches.md` — reachability and patch-equivalence inventory.
- `docs/takeover/evidence/prd-integrity.md` — generated structural discrepancy report.
- `docs/takeover/evidence/field-verification.md` — current manual-evidence summary linked to the existing matrix.
- `docs/takeover/CLAIMS.md` — customer-facing claim and evidence register.

### Modified files

- `package.json` — adds `test:takeover-ledger` and `check:takeover-ledger` scripts.
- `tools/check.ps1` — runs the takeover-ledger check before the existing verification, frontend and backend stages.

### Read-only inputs

- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/00_INDEX.md`
- `docs/PRD_v2/01_PRODUCT.md`
- `docs/PRD_v2/04_CORE_REQUIREMENTS.md`
- `docs/PRD_v2/06_SEQUENCING_AND_GATES.md`
- `docs/PRD_v2/91_REQUIREMENTS_INDEX.md`
- `docs/PRD_v2/CONTRACT.md`
- `docs/PRD_v2/RESUME.md`
- `docs/PRD_v2/_SPINE_PENDING.md`
- `REVIEW.md`
- `verification/capabilities.json`
- `docs/VERIFICATION_MATRIX.md`
- all Git refs reachable in the local repository after `git fetch origin`

---

### Task 1: G1-I001 — Create the validated tracker and initialize all 931 rows

**Files:**

- Create: `tools/takeover-ledger.mjs`
- Create: `tools/takeover-ledger.test.mjs`
- Create: `docs/takeover/requirements.csv`
- Create: `docs/takeover/STATUS.md`
- Create: `docs/takeover/DECISIONS.md`
- Modify: `package.json`
- Modify: `tools/check.ps1`

**Interfaces:**

- Consumes: `docs/PRD_v2/91_REQUIREMENTS_INDEX.md` consolidated table and the approved takeover design.
- Produces: named exports `splitMarkdownRow(line)`, `parseConsolidatedRequirements(markdown)`, `parseCsv(text)`, `renderCsv(rows)`, `validateLedger(sourceRows, ledgerRows)`, `validateStatus(markdown)` and `summarizeLedger(rows)`.
- Produces CLI modes:
  - `node tools/takeover-ledger.mjs --initialize`
  - `node tools/takeover-ledger.mjs --check`
  - `node tools/takeover-ledger.mjs --summary-json`
- `--initialize` MUST refuse to overwrite an existing ledger.
- `--check` MUST be read-only and exit nonzero on malformed CSV, duplicate IDs, source/ledger drift, invalid adjudicated vocabularies or missing dashboard fields.

- [ ] **Step 1: Capture the pre-change worktree identity in the terminal transcript**

Run:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
```

Expected: branch `codex/sandibumi-takeover-gate1`; HEAD is the approved design commit or a later plan-only commit; only the four pre-existing dirty/untracked paths are reported. If another path appears, stop and identify its owner before editing.

- [ ] **Step 2: Write the parser and validator tests first**

Create `tools/takeover-ledger.test.mjs` with these contracts:

```js
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  parseConsolidatedRequirements,
  parseCsv,
  renderCsv,
  splitMarkdownRow,
  summarizeLedger,
  validateLedger,
  validateStatus,
} from './takeover-ledger.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('an escaped pipe inside a requirement title does not create an extra column', () => {
  const row = '| `SB-SHR-026` | Pc uses `\\|cos theta\\|` consistently | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T28` |';
  assert.deepEqual(splitMarkdownRow(row), [
    'SB-SHR-026',
    'Pc uses `\\|cos theta\\|` consistently',
    'P1',
    'ABSENT',
    '15_sat-height-rocktyping.md',
    'SB-SHR-T28',
  ]);
});

test('quoted commas quotes and line breaks survive an RFC 4180 round trip', () => {
  const rows = [{ requirement_id: 'SB-CORE-001', blocking_decision: 'source says "refuse", owner decides\nrelease scope' }];
  assert.deepEqual(parseCsv(renderCsv(rows)), rows);
});

test('a duplicate requirement id makes the ledger invalid', () => {
  const source = [{ requirement_id: 'SB-CORE-001' }];
  const ledger = [{ requirement_id: 'SB-CORE-001' }, { requirement_id: 'SB-CORE-001' }];
  assert.throws(() => validateLedger(source, ledger), /duplicate requirement_id: SB-CORE-001/u);
});

test('raw chapter status is preserved while an unadjudicated row remains explicitly unadjudicated', () => {
  const row = parseConsolidatedRequirements([
    '## Consolidated requirements',
    '',
    '| ID | Title | Priority | Status | Chapter | Verified by |',
    '|---|---|---|---|---|---|',
    '| `SB-CORE-030` | Portfolio target | `P1` | `UNMEASURED` | `04_CORE_REQUIREMENTS.md` |  |',
  ].join('\n'))[0];
  assert.equal(row.chapter_status, 'UNMEASURED');
  assert.equal(row.as_built_status, 'UNADJUDICATED');
});

test('the checked out consolidated index contains exactly 931 unique requirements', () => {
  const markdown = fs.readFileSync(path.join(repo, 'docs', 'PRD_v2', '91_REQUIREMENTS_INDEX.md'), 'utf8');
  const rows = parseConsolidatedRequirements(markdown);
  assert.equal(rows.length, 931);
  assert.equal(new Set(rows.map((row) => row.requirement_id)).size, 931);
});

test('the dashboard names one active increment and separates automated from field evidence', () => {
  const status = [
    '# SandiBumi takeover status',
    '',
    '- Current gate: `G1`',
    '- Active increment: `G1-I001`',
    '- Accepted baseline: `b272d1951bd627fa75a0966cd1a94820ec2c3f22`',
    '- Automated gate: `NOT-RUN`',
    '- Pilot field evidence: `OPEN`',
    '- Open blockers: `UNMEASURED`',
    '- Next increment: `G1-I002`',
  ].join('\n');
  assert.doesNotThrow(() => validateStatus(status));
});

test('summary counts do not convert undecided rows into completed work', () => {
  const summary = summarizeLedger([
    { release_disposition: 'UNDECIDED', as_built_status: 'UNADJUDICATED' },
    { release_disposition: 'PILOT-BLOCKER', as_built_status: 'PRESENT-DIVERGENT' },
  ]);
  assert.deepEqual(summary, {
    total: 2,
    adjudicated: 1,
    unadjudicated: 1,
    pilot_blockers: 1,
  });
});
```

- [ ] **Step 3: Run the tests to prove the implementation is absent**

Run:

```powershell
node --test tools/takeover-ledger.test.mjs
```

Expected: FAIL because `tools/takeover-ledger.mjs` does not exist.

- [ ] **Step 4: Implement the dependency-free parser and CLI**

Create `tools/takeover-ledger.mjs` with:

```js
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
```

Implement `splitMarkdownRow` as a character scanner that treats `\|` as content and an unescaped `|` as a delimiter. Remove only the table's outer empty cells and surrounding Markdown backticks; do not unescape the title text.

Implement `parseCsv` and `renderCsv` as RFC 4180 readers/writers supporting commas, doubled quotes, CRLF/LF and quoted newlines. Do not split CSV with `String.split(',')`.

`parseConsolidatedRequirements` starts after the exact `## Consolidated requirements` heading and maps every `| \`SB-...` row to all `LEDGER_COLUMNS`. New rows initialize to:

```js
{
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
}
```

Copy the index's `Verified by` cell into `owned_tests` without claiming those tests currently exist or prove the whole requirement.

`validateLedger` MUST verify:

- exact column order;
- unique `requirement_id` values;
- source and ledger have the same IDs;
- source-owned fields match `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`;
- manually adjudicated status/disposition/risk values use the declared vocabularies;
- `last_reverified` is empty only while `as_built_status` is `UNADJUDICATED`;
- `expected_value_source` is never synthesized by this tool.

`validateStatus` MUST require exactly one line for each of:

```text
Current gate
Active increment
Accepted baseline
Automated gate
Pilot field evidence
Open blockers
Next increment
```

- [ ] **Step 5: Run the unit and checkout-integration tests**

Run:

```powershell
node --test tools/takeover-ledger.test.mjs
```

Expected: all seven named tests PASS.

- [ ] **Step 6: Initialize the ledger exactly once**

Run:

```powershell
node tools/takeover-ledger.mjs --initialize
node tools/takeover-ledger.mjs --summary-json
```

Expected: initialization creates `docs/takeover/requirements.csv`; summary reports 931 total, zero adjudicated, 931 unadjudicated and zero pilot blockers. The complete check waits until the dashboard and decision register exist.

- [ ] **Step 7: Create the one-minute dashboard**

Create `docs/takeover/STATUS.md` with these exact sections:

```markdown
# SandiBumi takeover status

This is the one-minute program dashboard. Requirement evidence lives in
`docs/takeover/requirements.csv`; manual field evidence remains in `REVIEW.md` and
`docs/VERIFICATION_MATRIX.md`.

## Now

- Product target: paid offline Windows pilot
- Current gate: `G1 — BASELINE RECONCILIATION`
- Active increment: `G1-I001 — TRACKER FOUNDATION`
- Accepted baseline: `b272d1951bd627fa75a0966cd1a94820ec2c3f22`
- Automated gate: `NOT-RUN FOR GATE 1`
- Pilot field evidence: `OPEN`
- Open blockers: `UNMEASURED — baseline reconciliation not complete`
- Next increment: `G1-I002 — DATED BASELINE RECEIPT`

## Gate dashboard

| Gate | State | Exit evidence |
|---|---|---|
| G1 — Baseline reconciliation | IN PROGRESS | 931 live adjudications, branch inventory, gate receipt, field-evidence and claims audits |
| G2 — Silent-wrongness closure | NOT STARTED | no known pilot-reachable silent-wrongness path remains enabled |
| G3 — Windows/offline deployment and recovery | NOT STARTED | clean-machine, offline-runtime, rollback and recovery matrix |
| G4 — Real-data pilot verification | NOT STARTED | Jauhar-confirmed representative workflow evidence |
| G5 — Release freeze and pilot acceptance | NOT STARTED | one frozen candidate accepted through deployment and pilot use |

## Requirement ledger

The generated summary is re-measured by `node tools/takeover-ledger.mjs --summary-json`.
Do not replace it with an estimated percentage.

## Recent increments

| Increment | State | Evidence | Commit |
|---|---|---|---|
| G1-I001 — Tracker foundation | IN PROGRESS | ledger and validator under construction | latest commit touching this file |

## Decisions needed from Jauhar

See `docs/takeover/DECISIONS.md`. Only rows marked `NEEDS-JAUHAR` require an answer.

## Worktree protection

The pre-existing dirty and untracked paths recorded in the dated baseline receipt are not takeover
inputs and remain unstaged unless Jauhar explicitly assigns them.
```

- [ ] **Step 8: Create the product-owner decision register**

Create `docs/takeover/DECISIONS.md` with this schema and initial rows:

```markdown
# SandiBumi takeover decisions

This register separates product-owner policy from engineering fact. `OPEN` means no decision has
been inferred. A decision row changes only from explicit Jauhar direction or named external evidence.

| ID | Decision | State | Current direction | What settles it | Blocks |
|---|---|---|---|---|---|
| DEC-001 | Re-adjudicate original P0 priorities for the paid pilot | DECIDED | Authorized 2026-08-10 | Jauhar authorization | none |
| DEC-002 | Release program | DECIDED | Five gates; Windows-first paid offline pilot | Jauhar authorization | none |
| DEC-003 | Pilot workflow and representative corpus | NEEDS-JAUHAR | OPEN | Jauhar names the workflow and supplies locally controlled representative data | G4 |
| DEC-004 | Customer-facing 2,000-well statement | NEEDS-JAUHAR | Design recommends removal until a defined benchmark proves it | Jauhar chooses removal now or an explicitly non-customer-facing hold | G5 |
| DEC-005 | Licence unit and activation | NEEDS-JAUHAR | OPEN | Commercial decision informed by deployment constraints | G5 |
| DEC-006 | Commercial model and support commitment | NEEDS-JAUHAR | OPEN | Commercial decision with written hours and escalation boundary | G5 |
| DEC-007 | Update delivery and supported-version window | NEEDS-JAUHAR | OPEN | Commercial and deployment decision | G5 |
| DEC-008 | Portfolio benchmark operations and thresholds | NEEDS-JAUHAR | OPEN | Named operations, fixture and hardware profile | later scale claim |
| DEC-009 | Lineage granularity beyond the pilot audit need | NEEDS-JAUHAR | OPEN | Audit requirement from the pilot or buyer | later lineage design |
| DEC-010 | Linux product timing and support contract | DEFERRED | Revisit after the Windows pilot | Named opportunity and support capacity | no Windows-pilot block |
```

- [ ] **Step 9: Add package scripts and the full-gate stage**

Add to `package.json`:

```json
"test:takeover-ledger": "node --test tools/takeover-ledger.test.mjs",
"check:takeover-ledger": "node tools/takeover-ledger.mjs --check"
```

Modify `tools/check.ps1` so the takeover ledger is stage 1 and existing stages become 2 through 4:

```powershell
Write-Host "[1/4] takeover ledger: source and tracker agree..." -ForegroundColor Cyan
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Push-Location $repo
& npm run test:takeover-ledger
$code = $LASTEXITCODE
if ($code -eq 0) {
    & npm run check:takeover-ledger
    $code = $LASTEXITCODE
}
Pop-Location
if ($code -ne 0) { Fail "takeover ledger" $code }
Write-Host ("[1/4] takeover ledger green in {0:n0}s" -f $sw.Elapsed.TotalSeconds) -ForegroundColor Green
```

Do not change the commands inside the existing verification, frontend or backend stages.

- [ ] **Step 10: Run the focused and repository checks**

Run in order:

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
npx tsc --noEmit
Push-Location src-tauri
cargo check
Pop-Location
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: every command exits 0. Record the full gate's actual passed/failed/ignored numbers in Task 2; do not reuse an older count.

- [ ] **Step 11: Mark G1-I001 complete in the dashboard**

Change only these dashboard fields:

```text
Active increment: G1-I002 — DATED BASELINE RECEIPT
Next increment: G1-I003 — BRANCH RECONCILIATION
G1-I001 state: DONE
G1-I001 evidence: 931-row ledger, seven named tracker tests, ledger check and full gate green
```

- [ ] **Step 12: Commit the tracker foundation**

Run:

```powershell
git add -- tools/takeover-ledger.mjs tools/takeover-ledger.test.mjs docs/takeover/requirements.csv docs/takeover/STATUS.md docs/takeover/DECISIONS.md package.json tools/check.ps1
git diff --cached --check
git commit -m "G1-I001 establish the takeover tracker"
```

Expected: one commit containing only the five new tracker paths and the two package/gate files; pre-existing dirty/untracked paths remain unstaged.

---

### Task 2: G1-I002 — Record the dated baseline receipt

**Files:**

- Create: `docs/takeover/evidence/2026-08-10-baseline.md`
- Modify: `docs/takeover/STATUS.md`

**Interfaces:**

- Consumes: current Git identity, tracker JSON summary, `REVIEW.md`, generated capability matrix and the full gate output.
- Produces: one dated evidence receipt containing measured facts only.

- [ ] **Step 1: Re-measure Git and tracker state**

Run:

```powershell
git branch --show-current
git rev-parse HEAD
git rev-list --left-right --count master...HEAD
git status --short
node tools/takeover-ledger.mjs --summary-json
```

Expected: the takeover branch is current; the tracker summary reports 931 unadjudicated rows; only pre-existing dirty/untracked paths remain outside the tracker commit.

- [ ] **Step 2: Re-measure manual checklist counts without editing it**

Run:

```powershell
$reviewLines = Get-Content -LiteralPath REVIEW.md
$checked = ($reviewLines | Select-String '\[x\]' -AllMatches).Matches.Count
$unchecked = ($reviewLines | Select-String '\[ \]' -AllMatches).Matches.Count
[pscustomobject]@{
    Checked = $checked
    Unchecked = $unchecked
    Total = $checked + $unchecked
    RatioPercent = [math]::Round(100 * $checked / [math]::Max(1, $checked + $unchecked), 1)
} | Format-List
node tools/generate-verification-matrix.mjs --check
```

Expected: the generator exits 0. Copy the measured counts; do not assume the earlier 78/1,470 snapshot still holds.

- [ ] **Step 3: Run the full gate and retain the exact final summary**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: exit 0. If nonzero, record the failing stage and exact test in the baseline receipt, set the dashboard automated gate to `RED`, and stop this plan after committing the evidence receipt. Do not continue to branch or PRD adjudication with an unexplained red baseline.

- [ ] **Step 4: Write the baseline receipt**

Create `docs/takeover/evidence/2026-08-10-baseline.md` with these headings and measured content:

```markdown
# Gate 1 baseline receipt — 2026-08-10

## Identity

- Branch
- HEAD
- `master...HEAD` left/right counts
- Worktree paths present before Gate 1

## Requirement ledger

- Source rows
- Unique IDs
- Original priority counts
- Raw chapter-status counts
- Live-adjudication count
- Pilot-disposition count

## Manual and field evidence

- Checked scenarios
- Unchecked scenarios
- Total scenarios
- Ratio
- Capability-matrix recorded and fully exercised counts

## Automated gate

- Command
- Exit code
- Frontend result
- Rust passed/failed/ignored result
- Commit tested

## Interpretation boundary

This receipt describes repository and evidence state. It does not declare a petrophysical result
correct and does not mark an unchecked manual scenario complete.
```

Every bullet receives the exact observed value or the exact word `FAILED` plus its evidence. Do not write estimates.

- [ ] **Step 5: Update the dashboard**

Set:

```text
Automated gate: GREEN or RED, with date and tested commit
Active increment: G1-I003 — BRANCH RECONCILIATION, only when the gate is green
Next increment: G1-I004 — PRD STRUCTURAL INTEGRITY
G1-I002 state: DONE or BLOCKED
```

- [ ] **Step 6: Validate and commit the receipt**

Run:

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
git add -- docs/takeover/evidence/2026-08-10-baseline.md docs/takeover/STATUS.md
git diff --cached --check
git commit -m "G1-I002 record the dated baseline receipt"
```

Expected: tracker checks pass; commit contains only the receipt and dashboard.

---

### Task 3: G1-I003 — Reconcile branch reachability and patch equivalence

**Files:**

- Create: `docs/takeover/evidence/branches.md`
- Modify: `docs/takeover/STATUS.md`

**Interfaces:**

- Consumes: local and fetched `origin` refs, accepted baseline, `git cherry` patch equivalence and exact commit diffs.
- Produces: classification `PATCH-EQUIVALENT`, `ACCEPTED-CANDIDATE`, `SUPERSEDED`, `REJECTED` or `UNRESOLVED` for every non-contained commit.

- [ ] **Step 1: Refresh remote knowledge without changing branches**

Run:

```powershell
git fetch origin
git branch --show-current
```

Expected: fetch succeeds and the takeover branch remains checked out. Do not use `--prune` in this inventory task.

- [ ] **Step 2: Enumerate local and remote non-contained refs**

Run a PowerShell loop over `git for-each-ref --format='%(refname:short)' refs/heads refs/remotes/origin`, excluding symbolic `origin/HEAD`, and capture:

```text
ref name
behind master count
ahead master count
head hash
head date
head subject
```

Expected: the report includes the current takeover branch and any remote branch with commits not reachable from `master`.

- [ ] **Step 3: Distinguish unique patches from cherry-picked equivalents**

For every ref with a positive ahead count, run:

```powershell
git cherry -v master <ref-name>
```

Interpret `-` as patch-equivalent and `+` as a candidate unique patch. Do not classify by commit hash or subject alone.

- [ ] **Step 4: Inspect every `+` commit without merging it**

For each unique patch, run:

```powershell
git show --stat --summary <commit>
git show --format= --name-only <commit>
```

Then inspect its exact diff. Verify whether its governing requirement, owned tests and full-gate evidence exist. A unique patch remains `UNRESOLVED` when acceptance evidence is missing.

- [ ] **Step 5: Write the branch inventory**

Create `docs/takeover/evidence/branches.md` with:

```markdown
# Gate 1 branch and commit inventory

## Accepted baseline

## Patch-equivalent refs

## Unique candidate commits

| Commit | Ref | Subject | Owned paths | Requirement | Evidence | Classification | Reason |
|---|---|---|---|---|---|---|---|

## Explicit exclusions

## Integration actions deferred
```

Every unique commit receives a row. Do not call work accepted merely because a previous task said it was done.

- [ ] **Step 6: Update the dashboard and validate**

Record the exact number of patch-equivalent, accepted-candidate, superseded, rejected and unresolved commits. Set the next active increment only when every non-contained commit is classified.

Run:

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
```

Expected: both pass.

- [ ] **Step 7: Commit without integrating any candidate**

Run:

```powershell
git add -- docs/takeover/evidence/branches.md docs/takeover/STATUS.md
git diff --cached --check
git commit -m "G1-I003 reconcile branch and commit evidence"
```

Expected: documentation-only commit; no merge, cherry-pick, rebase, deletion or push.

---

### Task 4: G1-I004 — Generate the PRD structural-integrity audit

**Files:**

- Modify: `tools/takeover-ledger.mjs`
- Modify: `tools/takeover-ledger.test.mjs`
- Create: `docs/takeover/evidence/prd-integrity.md`
- Modify: `docs/takeover/STATUS.md`

**Interfaces:**

- Consumes: PRD spine files and all chapter paths referenced by the consolidated index.
- Produces CLI modes:
  - `node tools/takeover-ledger.mjs --write-prd-audit`
  - `node tools/takeover-ledger.mjs --check-prd-audit`
- The generated report records discrepancies and exits 0 when the report matches current facts. It does not claim the discrepancies are resolved.

- [ ] **Step 1: Add failing tests for the audit rules**

Add named tests:

```js
test('rollup status totals are compared with the consolidated rows rather than trusted', () => {});
test('a chapter named by the document map but absent from disk is reported', () => {});
test('a resume chapter count that disagrees with files on disk is reported', () => {});
test('blank and out of vocabulary chapter statuses remain visible findings', () => {});
test('every chapter reference in the consolidated index resolves to one file', () => {});
test('the generated PRD integrity report is byte current', () => {});
test('the complete tracker check rejects a stale PRD audit once the report exists', () => {});
```

Use in-memory Markdown fixtures for the first four tests and the checked-out PRD for the last three.

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```powershell
npm run test:takeover-ledger
```

Expected: the seven new tests FAIL because audit functions and CLI modes do not exist.

- [ ] **Step 3: Implement the audit without editing the PRD**

Add exports:

```js
export function parseRollups(markdown) {}
export function auditPrd({ indexMarkdown, indexPath, prdDirectory }) {}
export function renderPrdAudit(audit) {}
```

`auditPrd` MUST report:

- consolidated row count and unique-ID count;
- priority roll-up versus row-derived counts;
- status roll-up versus row-derived counts;
- blank and contract-invalid status values;
- blank priorities;
- requirements without an owned acceptance-test ID;
- chapter files referenced by rows and whether each resolves;
- domain chapter files on disk not represented by rows;
- `RESUME.md` chapter-count claim versus files on disk;
- document-map artifacts named but absent on disk;
- `_SPINE_PENDING.md` items and their stated open/closed labels.

Do not infer that a discrepancy is fixed. Emit `OPEN`, `CLOSED-AS-RECORDED` or `INCONSISTENT` from explicit evidence only.

Extend the existing `--check` mode so it also performs the byte-current PRD-audit check whenever
`docs/takeover/evidence/prd-integrity.md` exists. Before Task 4 creates that file, `--check` validates
only the ledger and dashboard. This keeps Task 1 bootstrappable and makes the full repository gate
enforce the audit after it becomes part of the tracker.

- [ ] **Step 4: Generate and check the report**

Run:

```powershell
node tools/takeover-ledger.mjs --write-prd-audit
node tools/takeover-ledger.mjs --check-prd-audit
npm run test:takeover-ledger
```

Expected: all commands exit 0 and `docs/takeover/evidence/prd-integrity.md` records the current inconsistencies rather than modifying their sources.

- [ ] **Step 5: Update dashboard findings**

Add exact measured counts for:

```text
consolidated requirements
roll-up mismatches
blank priorities
blank statuses
invalid statuses
requirements without an owned test ID
missing promised artifacts
stale resume claims
```

Set `G1-I004` to `DONE` when the generated report is current. These findings remain open for later adjudication.

- [ ] **Step 6: Run checks and commit**

Run:

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
npx tsc --noEmit
Push-Location src-tauri
cargo check
Pop-Location
git add -- tools/takeover-ledger.mjs tools/takeover-ledger.test.mjs docs/takeover/evidence/prd-integrity.md docs/takeover/STATUS.md
git diff --cached --check
git commit -m "G1-I004 audit PRD structural integrity"
```

Expected: four owned paths plus the dashboard are committed; PRD v2 remains untouched.

---

### Task 5: G1-I005 — Record the manual and field-verification baseline

**Files:**

- Create: `docs/takeover/evidence/field-verification.md`
- Modify: `docs/takeover/STATUS.md`

**Interfaces:**

- Consumes: `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md` and `tools/generate-verification-matrix.mjs`.
- Produces: a dated summary of manual scenarios and capability coverage without checking any box.

- [ ] **Step 1: Verify the existing generated matrix is current**

Run:

```powershell
node tools/generate-verification-matrix.mjs --check
```

Expected: exit 0. If it fails, record the drift and stop this task; do not regenerate or edit manual evidence until the source of drift is understood.

- [ ] **Step 2: Measure checklist and capability counts**

Count checked and unchecked `REVIEW.md` scenarios directly. Parse the generated matrix header for:

```text
total mapped capabilities
capabilities with recorded exercise
fully exercised capabilities
```

Also count capabilities with `Not listed`, `Not recorded`, `Not exercised`, `Partially exercised` and `Exercised` states from the generated rows.

- [ ] **Step 3: Write the field-verification evidence report**

Create `docs/takeover/evidence/field-verification.md` with:

```markdown
# Gate 1 manual and field-verification baseline

## Evidence boundary

## Scenario counts

## Capability matrix

## Capabilities with recorded exercise

## Capabilities not fully exercised

## Mapping gaps

## Gate 4 consequence
```

State explicitly that automated and desktop-harness evidence does not close an unchecked manual scenario. Link to the existing generated matrix rather than reproducing all 54 rows.

- [ ] **Step 4: Update the dashboard**

Record the re-measured scenario and capability counts. Keep `Pilot field evidence: OPEN`; Gate 4 cannot close before Jauhar defines and confirms the pilot workflow.

- [ ] **Step 5: Validate and commit**

Run:

```powershell
node tools/generate-verification-matrix.mjs --check
npm run test:takeover-ledger
npm run check:takeover-ledger
git add -- docs/takeover/evidence/field-verification.md docs/takeover/STATUS.md
git diff --cached --check
git commit -m "G1-I005 record field verification evidence"
```

Expected: documentation-only commit; neither `REVIEW.md` nor the generated matrix changes.

---

### Task 6: G1-I006 — Inventory customer-facing claims and close the foundation checkpoint

**Files:**

- Create: `docs/takeover/CLAIMS.md`
- Modify: `docs/takeover/STATUS.md`

**Interfaces:**

- Consumes: current customer-facing source and documentation, measured benchmark evidence, legal-risk register and the approved design.
- Produces: claim states `PROVEN`, `QUALIFIED`, `UNMEASURED`, `REMOVE-RECOMMENDED`, `LEGAL-REVIEW` or `UNDECIDED`.

- [ ] **Step 1: Search customer-facing and governing surfaces**

Run targeted searches for:

```text
2000+
2,000
only tool
best
most complete
field-verified
offline
no Python
Windows
Linux
byte-identical
audit trail
```

Search `README.md`, `CLAUDE.md`, `src/`, `public/`, package metadata and the product/strategy spine. Exclude `docs/research_2026-08/` and generated build directories. A search hit is a candidate claim, not proof that the surface is customer-facing.

- [ ] **Step 2: Trace each claim to evidence**

For every candidate, record:

```text
claim text
surface and line
audience
evidence source
current state
release action
owner
blocking decision
```

The 2,000-well statement is `UNMEASURED` unless the checked-out tree contains the defined passing benchmark the PRD requires. Do not substitute the 540-well observation.

- [ ] **Step 3: Write the claim register**

Create `docs/takeover/CLAIMS.md` with:

```markdown
# SandiBumi customer-facing claim register

## Rules

## Claims

| ID | Claim | Surface | Audience | Evidence | State | Release action | Owner/blocker |
|---|---|---|---|---|---|---|---|

## Claims deliberately absent

## Legal-review claims
```

Assign stable IDs `CLAIM-001`, `CLAIM-002` in source-path and line order. Do not edit the claiming surfaces in this foundation task.

- [ ] **Step 4: Mark the foundation checkpoint**

Update `docs/takeover/STATUS.md`:

```text
G1-I006 — CLAIM INVENTORY: DONE
Baseline foundation: COMPLETE
Current gate: G1 — BASELINE RECONCILIATION
Active increment: G1-DOM-CORE — SB-CORE LIVE ADJUDICATION PLAN
Next increment: write and approve the domain-adjudication implementation plan
```

Do not say Gate 1 is complete. All 931 rows still require live domain adjudication and cross-domain release disposition.

- [ ] **Step 5: Run the full foundation gate**

Run:

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
node tools/generate-verification-matrix.mjs --check
npx tsc --noEmit
Push-Location src-tauri
cargo check
Pop-Location
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: every command exits 0. Record the actual full-gate counts in the dashboard and baseline receipt if they changed; never reuse a historical count.

- [ ] **Step 6: Confirm only owned paths are staged and commit**

Run:

```powershell
git add -- docs/takeover/CLAIMS.md docs/takeover/STATUS.md
git diff --cached --check
git commit -m "G1-I006 inventory release claims"
git status --short
```

Expected: claim register and dashboard committed; pre-existing dirty/untracked paths still appear and no other path is staged.

---

## Foundation Exit Review

Before writing the domain-adjudication plan, verify:

- [ ] `docs/takeover/STATUS.md` answers current gate, active increment, evidence, blockers and next action in under one minute.
- [ ] `docs/takeover/requirements.csv` contains exactly the source requirement IDs and preserves original priority/status verbatim.
- [ ] No row is called adjudicated merely because its chapter had a status.
- [ ] `docs/takeover/DECISIONS.md` contains only explicit decisions and clearly marked open questions.
- [ ] The current automated gate has a dated receipt tied to one commit.
- [ ] Every non-contained commit has an evidence classification; none was integrated during inventory.
- [ ] PRD structural inconsistencies are reported without editing the PRD.
- [ ] Manual evidence is linked to the existing capability matrix and no checkbox was changed.
- [ ] Customer-facing claims are registered without silently editing or approving them.
- [ ] The full gate is green, or the plan has stopped on a recorded red baseline.
- [ ] The next plan is limited to domain-by-domain live adjudication of the 931 ledger rows.
