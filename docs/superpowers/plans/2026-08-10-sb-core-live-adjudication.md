# SB-CORE Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reverify every one of the 25 live `SB-CORE` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce the ordered follow-up increments without changing product behavior.

**Architecture:** This is a documentation-only evidence pass. The PRD remains an immutable statement of intent and historical chapter status; current source, qualifying acceptance tests, manual evidence and reachable Git history establish the separate as-built classification. One domain receipt explains every verdict, while `docs/takeover/requirements.csv` carries the machine-validated summary. Requirement priority, as-built state, pilot membership and defect state remain separate concepts so a built contract cannot be mistaken for an approved release scope, and an original P1 cannot be silently dismissed from a paid pilot.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, database schema or any generated product artifact.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- This increment MUST NOT edit `docs/PRD_v2/**`, `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`, `ROADMAP.md`, `CLAUDE.md` or `AGENTS.md`.
- The exact accepted evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. Reverify it before execution; if the accepted integration branch or `origin/master` has moved, stop and reconcile the base before classifying a row.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed against exact source files, tests and history because the Rust index would not have been authoritative for macros or `cfg`-gated code anyway.
- Read `AGENTS.md`, all of `CLAUDE.md`, the applicable `docs/record_*.md`, `docs/PRD_v2/CONTRACT.md`, the complete SB-CORE chapter and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. The known invalid chapter statuses on `SB-CORE-030` and `SB-CORE-033` remain reported evidence, not values this lane repairs.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid pilot. A `PILOT-BLOCKER` row may be `PRESENT-OK`; in that case the required pilot contract is satisfied rather than defective.
- `PILOT-BLOCKER` MUST mean the contract is required for the approved paid, offline, Windows-first pilot. `DEFERRED` requires an explicit existing product decision or a clearly later-only dependency. `OUT` requires explicit Jauhar exclusion. Otherwise keep `UNDECIDED` and name the decision that remains.
- Original PRD priority is evidence, not automatic pilot scope. Do not silently promote every P0 or dismiss every P1/P2.
- A positive code path does not close a requirement by itself. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, or an enumerated list. One missing obligation makes the result `PARTIAL` or `PRESENT-DIVERGENT`, not `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's reporting or user-observable contract, has an independently sourced expected result, and maps to the chapter's owned test ID. A helper test or a test that asserts only an internal `Result` is supporting evidence, never closure.
- Where the chapter has no owned test ID, current tests may be listed as supporting evidence but `test_class` remains `MISSING`. Do not write a test in this adjudication increment.
- Classify expected values exactly as `CORRECTNESS` or `CHARACTERIZATION` using `CONTRACT.md` §3 and §6. A present test with an uncited expected value is characterization; do not upgrade it because it passes.
- Manual and field evidence is read from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`. An unchecked scenario stays unchecked. Automated, desktop-harness and build evidence do not become field evidence.
- For petrophysical parameters, cutoffs, endpoints, constants, units and conversions, cited or absent remains absolute. This pass records a missing source; it never supplies a value.
- Git reachability proves that a change is in the accepted tree, not that the behavior is correct. Patch-equivalent or superseded commits are named only where they explain the accepted source.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every write is made with `apply_patch`; stage exact paths only.
- The approved plan authorizes only the evidence receipt, the 25 ledger-row adjudications and the dashboard handoff. It does not authorize any production fix or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 25 SB-CORE rows:

```text
SB-CORE-001 SB-CORE-002 SB-CORE-003 SB-CORE-004 SB-CORE-005
SB-CORE-006 SB-CORE-007 SB-CORE-010 SB-CORE-011 SB-CORE-012
SB-CORE-013 SB-CORE-014 SB-CORE-015 SB-CORE-030 SB-CORE-031
SB-CORE-032 SB-CORE-033 SB-CORE-034 SB-CORE-035 SB-CORE-036
SB-CORE-040 SB-CORE-041 SB-CORE-042 SB-CORE-043 SB-CORE-044
```

At plan time all 25 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. Fourteen have no chapter-owned acceptance-test ID: `003`, `005`, `012`, `013`, `030`, `031`, `032`, `033`, `034`, `035`, `036`, `042`, `043`, `044`.

The original P0 subset is only eight rows: `001`, `002`, `004`, `006`, `007`, `015`, `040`, `041`. Gate 1 nevertheless adjudicates all 25 because release disposition is deliberately independent of the old priority.

Run this guard before and after editing:

```powershell
$core = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-CORE-*' }
$expected = @(
    'SB-CORE-001','SB-CORE-002','SB-CORE-003','SB-CORE-004','SB-CORE-005',
    'SB-CORE-006','SB-CORE-007','SB-CORE-010','SB-CORE-011','SB-CORE-012',
    'SB-CORE-013','SB-CORE-014','SB-CORE-015','SB-CORE-030','SB-CORE-031',
    'SB-CORE-032','SB-CORE-033','SB-CORE-034','SB-CORE-035','SB-CORE-036',
    'SB-CORE-040','SB-CORE-041','SB-CORE-042','SB-CORE-043','SB-CORE-044'
)
if ($core.Count -ne 25) { throw "Expected 25 SB-CORE rows, found $($core.Count)" }
if (@(Compare-Object $expected @($core.requirement_id)).Count -ne 0) {
    throw 'The live SB-CORE ID set differs from the approved plan'
}
```

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-core.md` — complete 25-row evidence receipt, including obligation-by-obligation source findings, tests, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` — only the adjudication-owned fields for the 25 SB-CORE rows.
- `docs/takeover/STATUS.md` — measured row counts, current blocker summary and the next serial domain increment.

### Read-only governing inputs

- `AGENTS.md`
- `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`
- `docs/PRD_v2/03_EVIDENCE_BASE.md`
- `docs/PRD_v2/04_CORE_REQUIREMENTS.md`
- `docs/PRD_v2/06_SEQUENCING_AND_GATES.md`
- `docs/PRD_v2/91_REQUIREMENTS_INDEX.md`
- `docs/record_fixes.md`
- `docs/record_parallel_lanes.md`
- `docs/record_data_tools.md`
- `docs/takeover/DECISIONS.md`
- `docs/takeover/CLAIMS.md`
- `docs/takeover/evidence/branches.md`
- `docs/takeover/evidence/prd-integrity.md`
- `docs/takeover/evidence/field-verification.md`
- `REVIEW.md`
- `verification/capabilities.json`
- `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Files this adjudication MUST NOT change

- every read-only governing input above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/`;
- `REVIEW.md` and its generated matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-core.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness and the 25-row guard result. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-CORE-NNN — exact title

- Chapter evidence: priority, verbatim chapter status, owned test IDs and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, source of expected value and class.
- Supporting tests: tests that help but cannot close the owned contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or `UNIMPLEMENTED`; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, dependency or `none`.
- Next action: one bounded production, test, field, decision or no-action increment.
```

For every row, copy the summary into these adjudication-owned ledger fields only:

```text
as_built_status
release_disposition
risk_class
implementation_paths
test_class
expected_value_source
manual_evidence
dependencies
commit_state
blocking_decision
next_action
last_reverified
```

Use this exact verification stamp for the accepted anchor:

```text
2026-08-10 @ b332026cb498c105f36eade0bf7899bc0c1309f0
```

If HEAD changes before execution, replace the stamp with the newly approved accepted integration commit; never stamp the planning commit as implementation evidence.

---

## Requirement Evidence Map

The following paths are candidates to inspect, not pre-decided verdicts. Each task must expand its searches until every atomic obligation is answered.

### Truth and data-integrity group

| ID | Atomic contract to adjudicate | Required source/test evidence | Manual capabilities | Known history or caveat |
|---|---|---|---|---|
| `SB-CORE-001` | Depth unit is carried through every import, storage, depth-dependent computation and export; undeclared units refuse. | `src-tauri/src/units.rs`, `parsers.rs`, `ingest.rs`, `intake.rs`, `dlis.rs`, `reframe.rs`, `satheight.rs`, `workflow.rs`, `export.rs`; exact T01/T01b/T02 tests plus LAS/DLIS round-trip/refusal tests. | `data-conventions`, `las-import`, `dlis-import`, `saturation-height`, `las-export` | `bb807ca` added core unit behavior, but a few examples cannot prove the word “every.” Preserve native imported depth samples and explicit Reframe from `record_data_tools.md`. |
| `SB-CORE-002` | Every one of the original seven degraded-result reporting surfaces is honest, with positive and clean controls. | `src-tauri/src/core_reporting_tests.rs`, `ingest.rs`, `report.rs`, `src/ui/reportingHonesty.ts`, `summaryDialog.ts`, `dashboardPanel.ts`, `mlDialog.ts`, `tools/frontend-acceptance.test.mjs`; exact T03–T09 test sentences. | `monte-carlo`, `las-import`, `report`, `machine-learning`, `workflow` | `78bd21d`, `d25c274` and accepted merge `02b59ea`; do not count an internal `Result` without the user/job/batch surface. |
| `SB-CORE-004` | Every shipped default carries a machine-readable source; empty-source defaults fail the build and required source-less parameters refuse at runtime. | `src-tauri/src/modules.rs` registry and manifest validation, `workflow.rs` runtime validation, `param_sources.rs`, relevant registry tests and all default-bearing module registrations; find the actual implementations of T10 and T11, not just conceptual IDs. | `verification-stewardship`, `workflow`; select any method-specific additions from the committed capability map, never from an invented label | Chapter says `PARTIAL`; the registry structure alone does not prove every default or both enforcement sides. Missing citations stay absent. |
| `SB-CORE-006` | One method name resolves to one equation across module and solver engines; emitted curve mnemonic, UI label and documentation agree. | `modules.rs`, `workflow.rs`, `equations.rs`, `distribution.rs`, shared-method solver modules, UI method selectors and output metadata; locate real T17/T18 coverage for both numeric parity and naming. | `workflow`, `equation-engine`, `sandimin`, `histogram`; add only exact affected IDs from `verification/capabilities.json` | Chapter says `PRESENT-DIVERGENT`; a shared helper for one method cannot close the universal contract. |
| `SB-CORE-007` | Every petrophysical constant/transform has one definition, deliberate duplicate transforms agree on a shared vector, and multi-producer output defaults are identical. | `modules.rs`, `equations.rs`, `units.rs`, default-bearing solver/module files, output mnemonic registrations, UI defaults and tests implementing T19/T20/T23. | `verification-stewardship`, `workflow`, `equation-engine`, `sandimin`; add only exact affected IDs from the committed capability map | Chapter says `PRESENT-DIVERGENT`; search constants and transforms across the whole source tree. Never infer a canonical number. |
| `SB-CORE-015` | Every shipped writer is accepted by its own reader and non-default fixtures round-trip, including declared depth units and DLIS. | Writer/reader inventory in `export.rs`, `dlis.rs`, `ingest.rs`, `parsers.rs`, `intake.rs`, `office.rs`, `project.rs`; actual T14/T15/T16 mapping and writer-registration corpus test. | `las-export`, `las-import`, `dlis-import`, `project-lifecycle`, `report`, `office-deliverables` | Chapter says `PRESENT-DIVERGENT`; LAS success does not prove DLIS or every writer. T14–T16 overlap lineage IDs elsewhere, so map exact test intent rather than ID alone. |
| `SB-CORE-040` | A capability-indexed matrix is generated from committed sources and its freshness is enforced by the gate. | `verification/capabilities.json`, `tools/generate-verification-matrix.mjs`, `tools/check.ps1`, `docs/VERIFICATION_MATRIX.md`, `src-tauri/tests/verification_matrix.rs::a_capability_matrix_is_generated_from_review_and_a_capability_map_and_checked_by_the_gate`. | `verification-stewardship` | `34ee79e`/patch-equivalent `8b420e5` is reachable; chapter `ABSENT` is demonstrably stale, but actual gate hookup still must be run. |
| `SB-CORE-041` | A genuinely fresh clone resolves dependencies, builds and runs the complete gate without local caches or untracked inputs. | `package.json`, lockfiles, `src-tauri/Cargo.toml`, `Cargo.lock`, `CONTRIBUTING.md`, `tools/check.ps1`, installation/build scripts and any CI or clean-clone receipt; find an actual T13 test or external clean-machine evidence. | `verification-stewardship`, `security-integrity`, `project-lifecycle` | A green existing worktree is not fresh-clone proof. Treat optional-package ignores and local caches explicitly. |

### Validity, provenance and reproducibility group

| ID | Atomic contract to adjudicate | Required source/test evidence | Manual capabilities | Known history or caveat |
|---|---|---|---|---|
| `SB-CORE-003` | Each method’s validity conditions are machine-readable preconditions and invalid runs refuse before computation. | `modules.rs` argument/manifest schema, `workflow.rs` validators, method-specific guards and UI surfacing; inventory whether conditions are data, executable guards or prose. | `workflow`, `verification-stewardship`; add only exact method IDs from `verification/capabilities.json` | No owned test ID. Method-specific guards are supporting evidence only; a universal machine-readable contract needs registry coverage. |
| `SB-CORE-005` | Every vendor-derived default used by SandiBumi is re-sourced to primary literature, not copied from a neighbouring vendor. | `modules.rs`, `param_sources.rs`, `parameter_pack.rs`, default-bearing modules, source comments/metadata and `docs/PRD_v2/03_EVIDENCE_BASE.md`; compare all shipped default sources to the Tier-A/Tier-B requirement. | `verification-stewardship`; add only exact method IDs from `verification/capabilities.json` | No owned test ID. Do not search the web or invent replacement values in this pass; record uncited/vendor-only defaults as gaps. |
| `SB-CORE-010` | Every computed curve carries complete ancestry and the record survives project save/load. | Read-only inspection of `db.rs` schema/write discipline, `workflow.rs`, module write paths, curve catalog/metadata IPC, `project.rs` and project serialization; map actual T14/T15 tests if they exist. | `processing-history`, `project-lifecycle`, `database-tools`, `verification-stewardship` | `DEC-009` governs lineage beyond the pilot audit need. Do not add a provenance write path or change `computed_curves`. |
| `SB-CORE-011` | A complete project rerun produces byte-identical curve blobs and an exact provenance record. | `project.rs`, workflow/module execution, deterministic seeds/order, byte serialization and an actual full-rerun T16 fixture; distinguish deterministic math from timestamps, IDs and export formatting. | `project-lifecycle`, `workflow`, `processing-history` | Chapter says `PARTIAL`; a single deterministic chain or same-process rerun may not prove full-project byte identity. |
| `SB-CORE-012` | Named interpretation scenarios can be rerun side by side with a persisted A/B difference. | Project/model/scenario stores, workflow UI, result comparison and processing history; confirm positive or negative source evidence and search history for a rejected/superseded implementation. | `project-lifecycle`, `workflow`, `results-qc` | No owned test ID and chapter says `ABSENT`; do not mistake ad-hoc plot comparison for a named persisted scenario. |
| `SB-CORE-013` | Where the corpus records vendor disagreement, competing sourced values are visible at the exact parameter choice and the chosen value is recorded. | `src-tauri/src/param_sources.rs`, `modules.rs` fields `sources_topic`/`param_sourced`, `facies.rs`, `ml.rs`, `lib.rs`; `src/ipc.ts`, `src/ui/paramSources.ts`, `moduleDialog.ts`, `mlDialog.ts`, `src/styles.css`; source-display and run-record tests. | `verification-stewardship`, `machine-learning`, affected module UI capabilities | Current source is clearly newer than chapter `ABSENT`, but one cluster-count topic may make the universal requirement only `PARTIAL`. No owned test ID. |
| `SB-CORE-014` | A learned model records every enumerated training-provenance element and refuses replay when any required element is absent. | Read-only inspection of `ml.rs`, model persistence schema in `db.rs`, ML IPC/UI, `export.rs` and actual T21/T22 replay/refusal tests. | `machine-learning`, `processing-history`, `project-lifecycle` | Chapter says `ABSENT`; export metadata is not sufficient if the saved model cannot reproduce the prediction. Do not modify the ML lane. |

### Scale, operations and responsiveness group

| ID | Atomic contract to adjudicate | Required source/test evidence | Manual capabilities | Known history or caveat |
|---|---|---|---|---|
| `SB-CORE-030` | A portfolio target names operations, fixture, hardware and thresholds and is measured by that exact benchmark. | `docs/takeover/CLAIMS.md`, `DECISIONS.md`, benchmark scripts/tests, historical 100-well/540-well observations and any reproducible receipt. | `portfolio-performance` | Chapter status `UNMEASURED` is outside the contract vocabulary and remains a structural finding. `DEC-004`/`DEC-008` block a numeric claim; do not manufacture a 2,000-well proof. |
| `SB-CORE-031` | The defined portfolio benchmark harness exists and runs inside the green gate. | `tools/check.ps1`, package scripts, `src-tauri/src/pipeline_field_test.rs`, ignored-test configuration, benchmark directories and CI files. | `portfolio-performance`, `verification-stewardship` | No owned test ID. A deterministic fixture or ignored field test is not automatically a benchmark, and a harness outside the gate does not satisfy this contract. |
| `SB-CORE-032` | Long compute work does not hold the global DuckDB mutex; snapshot-read, compute and short commit phases are distinguishable. | Read-only lock-scope trace through `db.rs`, `lib.rs`, `workflow.rs`, `jobs.rs`, long-running module commands and transaction boundaries; inspect concurrency/lock tests. | `workflow`, `portfolio-performance`, `database-tools`; add only exact long-running capability IDs from the committed map | Do not change the single-writer discipline. Chapter says `PRESENT-DIVERGENT`; absence of an observed stall is not proof of lock duration. |
| `SB-CORE-033` | Compute results are cached by complete content identity and invalidated when any input changes. | Search cache keys/stores in backend and frontend, workflow/module fingerprints, project persistence and invalidation tests. | `portfolio-performance`, `workflow` | Chapter status `ABSENT — designed, parked` is outside the contract vocabulary. `DEC-008` and measured need determine timing; UI render caches do not satisfy compute caching. |
| `SB-CORE-034` | Interactive portfolio surfaces stay within the chapter’s declared responsiveness contract under the declared benchmark. | `decimate.rs`, plotting/data-query paths, workspace/tree/plot UI, worker scheduling and performance receipts; map observed timings only to their exact fixture/hardware. | `portfolio-performance`, `workspace-shell`, `log-view`, `crossplot`, `vega`, `database-tools` | No owned test ID. Chapter says `PRESENT-DIVERGENT`; without the SB-CORE-030 benchmark definition, broad portfolio responsiveness cannot be proven. |
| `SB-CORE-035` | Well scope is authorized and enforced in backend commands, not merely filtered by frontend selection. | `src/ui/wellScope.ts`, every scoped IPC wrapper in `src/ipc.ts`, Tauri commands in `lib.rs`, and backend query/compute/export paths in `workflow.rs`, `statistics.rs`, `report.rs`, `export.rs`; refusal/isolation tests. | `well-scope`, `workflow`, `report`, relevant analysis capabilities | No owned test ID. A correct shared UI scope control does not prove backend enforcement on every command. |
| `SB-CORE-036` | Cancellation reports the truth and leaves no hidden partial result beyond the contract explicitly reported to the user. | `jobs.rs`, `workflow.rs`, import runners, long-running Rust modules, `src/ui/processingPanel.ts`, `workflowDialog.ts`, `ribbon.ts`; cancellation state, write-boundary and user-reporting tests. | `workflow`, `processing-history`, `las-import`, `dlis-import`, `monte-carlo`, `machine-learning` | No owned test ID. Cooperative cancel checks on one chain cannot prove all cancellable jobs or rollback semantics. |

### Governance and stewardship group

| ID | Atomic contract to adjudicate | Required source/test evidence | Manual capabilities | Known history or caveat |
|---|---|---|---|---|
| `SB-CORE-042` | Build, lint and tests are automatically machine-enforced on every change rather than manually invoked. | `tools/check.ps1`, package scripts, repository hooks and `.github/workflows/**`; `src-tauri/tests/governance_contracts.rs::characterizes_the_green_gate_as_machine_enforced_but_still_manually_invoked`. | `verification-stewardship` | `bb0c488` is reachable. The current test is characterization and explicitly records the manual invocation gap; it does not close the specified behavior. |
| `SB-CORE-043` | Architecture and product/engineering decisions have a maintained, discoverable record in the tree. | Exact inventory of architecture, ADR, design and decision paths, including `docs/superpowers/specs/**`, `docs/takeover/DECISIONS.md` and build records; confirm negative ADR search and ownership/update rules. | `verification-stewardship` | No owned test ID. Recent takeover design/decision files may make the old `ABSENT` status stale, but a one-off plan is not necessarily a maintained architecture record. |
| `SB-CORE-044` | Tier-C material is excluded from shipped defaults unless an explicit, auditable, asset-specific design-around route exists. | `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, source/provenance registries and `src-tauri/tests/governance_contracts.rs::characterizes_the_tier_c_register_as_shipped_policy_with_asset_specific_design_around_routes`. | `verification-stewardship` | `3abf150` is reachable. The existing test is characterization; it proves recorded policy state, not exhaustive enforcement across every shipped parameter. |

---

### Task 1: Freeze the Evidence Base and Create the Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-core.md`
- Read only: all governing and source paths listed above

- [ ] **Step 1: Reverify branch, base, origin and cleanliness**

Run:

```powershell
git fetch origin --prune
git branch --show-current
git rev-parse HEAD
git rev-parse origin/master
git merge-base HEAD origin/master
git status --short
git worktree list --porcelain
```

Expected: branch `codex/g1-sb-core-adjudication`; the accepted integration anchor remains an ancestor; only `D:\XX. SandiBumi` is a registered worktree; no unexplained tracked or untracked file is present. If `origin/master` moved, record it and stop for baseline reconciliation rather than silently rebasing.

- [ ] **Step 2: Re-read the governing documents and build records completely**

Read in the repository-prescribed order. Do not rely on the planning receipt as a substitute for the live files.

- [ ] **Step 3: Run the 25-row baseline guard**

Run the exact PowerShell guard in “Baseline and Count Contract,” then assert:

```powershell
if (@($core | Where-Object { $_.as_built_status -ne 'UNADJUDICATED' }).Count -ne 0) {
    throw 'An SB-CORE row was already adjudicated; reconcile the plan before proceeding'
}
```

- [ ] **Step 4: Create the evidence receipt header and 25 empty evidence headings**

Use `apply_patch`. The headings are not verdicts. Include all 25 IDs exactly once and no unfinished marker, guessed state or dummy value.

- [ ] **Step 5: Machine-check heading coverage before adding verdicts**

Run:

```powershell
$headings = Select-String -LiteralPath 'docs\takeover\evidence\sb-core.md' -Pattern '^## SB-CORE-[0-9]{3} — '
$ids = @($headings | ForEach-Object { [regex]::Match($_.Line, 'SB-CORE-[0-9]{3}').Value })
if ($ids.Count -ne 25 -or @($ids | Sort-Object -Unique).Count -ne 25) {
    throw 'The SB-CORE evidence receipt does not contain exactly 25 unique requirement headings'
}
if (@(Compare-Object $expected $ids).Count -ne 0) {
    throw 'The evidence receipt ID set does not match the ledger'
}
```

Do not commit the empty skeleton; continue to Task 2.

---

### Task 2: Adjudicate Truth and Data-Integrity Requirements

**Rows:** `SB-CORE-001`, `002`, `004`, `006`, `007`, `015`, `040`, `041`

**Files:**

- Modify: `docs/takeover/evidence/sb-core.md`
- Read only: the truth/data-integrity source and test paths in the evidence map

- [ ] **Step 1: Trace every atomic obligation through current source**

Use `rg` to locate candidate symbols, then open each exact file/function. For universal contracts (`every`, `all`, `one definition`) inventory the registry or all producers; do not extrapolate from one example.

- [ ] **Step 2: Run the existing focused correctness tests**

From `src-tauri`:

```powershell
cargo test saturation_height_is_identical_whichever_unit_the_project_declares -- --nocapture
cargo test skelt_harrison_is_identical_whichever_unit_the_project_declares -- --nocapture
cargo test a_depth_dependent_module_refuses_when_the_project_depth_unit_is_undeclared -- --nocapture
cargo test a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result -- --nocapture
cargo test an_all_channel_import_failure_returns_a_named_error_and_commits_no_partial_well -- --nocapture
cargo test a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record -- --nocapture
cargo test a_capability_matrix_is_generated_from_review_and_a_capability_map_and_checked_by_the_gate -- --nocapture
```

From the repository root:

```powershell
node --test --test-name-pattern='an_uninterpreted_pay_summary|a_partial_ml_run|a_stats_only_dashboard|a_training_well' tools/frontend-acceptance.test.mjs
node tools/generate-verification-matrix.mjs --check
```

If a named filter matches zero tests, record the missing executable mapping; do not call it passing.

- [ ] **Step 3: Verify owned test identity and expected-value source**

For T01–T20/T23, distinguish exact executable tests from prose intentions. Record `CORRECTNESS`, `CHARACTERIZATION`, `SPEC-DIVERGENCE-IGNORED`, `OPTIONAL-PACKAGE-IGNORED` or `MISSING` using only the current test body and its cited source.

- [ ] **Step 4: Verify commit reachability**

For each claimed implementation commit:

```powershell
git merge-base --is-ancestor <commit> b332026cb498c105f36eade0bf7899bc0c1309f0
git show --stat --oneline <commit>
```

Record accepted patch equivalence where the exact commit is not the ancestor but its patch is included. Never use branch-name presence as integration proof.

- [ ] **Step 5: Record manual evidence without altering it**

Copy the relevant generated matrix state and checked/total count into each receipt row. If a capability has zero checks, say so explicitly.

- [ ] **Step 6: Write eight complete verdicts**

Each verdict must name satisfied and unsatisfied clauses. Keep release disposition `UNDECIDED` wherever it would require a new pilot-scope decision. Do not implement a fix.

---

### Task 3: Adjudicate Validity, Provenance and Reproducibility Requirements

**Rows:** `SB-CORE-003`, `005`, `010`, `011`, `012`, `013`, `014`

**Files:**

- Modify: `docs/takeover/evidence/sb-core.md`
- Read only: the validity/provenance source and test paths in the evidence map

- [ ] **Step 1: Separate structural capability from complete coverage**

For `003`, `004`, `005` and `013`, inventory the relevant manifest fields and every registry entry that claims the behavior. A type field proves the capability exists; only populated and enforced entries prove coverage.

- [ ] **Step 2: Trace one computed curve and one saved learned model end to end**

Follow write, metadata/provenance, project save/load, replay and refusal surfaces. This is an audit trace, not a representative proof of all producers. Use it to locate the global inventory boundary for `010`, `011` and `014`.

- [ ] **Step 3: Search for executable T14/T15/T16/T21/T22 contracts**

Run:

```powershell
rg -n "SB-CORE-T(14|15|16|21|22)|ancestry|byte.identical|training provenance|re.runs to an identical curve" src src-tauri tools
```

Open every match. A LAS round trip assigned to `SB-CORE-015` does not automatically prove ancestry under `SB-CORE-010`; test IDs are disambiguated by their sentence and exercised surface.

- [ ] **Step 4: Audit SB-CORE-013 from both sides**

Verify that the parameter-source UI actually appears at the choice point and that the selected value/source reaches the run record. Then inventory how many contested topics are declared. Record `PARTIAL` if the mechanism is real but the requirement’s full corpus coverage is not.

- [ ] **Step 5: Apply the no-owned-test rule**

Rows `003`, `005`, `012` and `013` remain `test_class=MISSING` because the chapter owns no acceptance-test ID. Supporting tests go only in the evidence receipt and `implementation_paths`/`expected_value_source` fields; do not mutate `owned_tests`.

- [ ] **Step 6: Write seven complete verdicts**

Reference `DEC-009` for lineage granularity beyond the pilot audit need. If the minimal pilot lineage contract itself is unclear, keep release disposition `UNDECIDED` and name the exact question; do not invent a provenance schema.

---

### Task 4: Adjudicate Scale, Operations and Responsiveness Requirements

**Rows:** `SB-CORE-030`, `031`, `032`, `033`, `034`, `035`, `036`

**Files:**

- Modify: `docs/takeover/evidence/sb-core.md`
- Read only: the scale/operations source and test paths in the evidence map

- [ ] **Step 1: Establish the benchmark evidence boundary**

Read `CLAIM-001`, `DEC-004` and `DEC-008`. Inventory current benchmark scripts and receipts. State separately what the 100-well chain, the 540-well observation and any ignored field test actually prove. None is renamed a 2,000-well qualification.

- [ ] **Step 2: Confirm whether a harness is in the gate**

Trace every command in `tools/check.ps1` to its invoked tests. A file named `pipeline_field_test.rs` is evidence only if its subject, fixture, thresholds and gate invocation satisfy `SB-CORE-031`.

- [ ] **Step 3: Trace lock scope, cancellation and backend well scope**

Use call-chain inspection from Tauri command to database snapshot, compute, commit and result reporting. Inspect both successful and interrupted paths. Never propose relaxing the DuckDB single-writer discipline or adding an upsert.

- [ ] **Step 4: Search for compute caching, not render caching**

Inventory content fingerprints, cache persistence and invalidation tests. Explicitly exclude cached DOM rows, plot dimensions and UI memoization from `SB-CORE-033` unless they cache the specified compute result on full content identity.

- [ ] **Step 5: Record the manual performance boundary**

Use the exact `portfolio-performance` and related matrix rows. No unchecked scenario becomes measured. If hardware, operation or threshold is missing, the broad responsiveness contract remains unproven.

- [ ] **Step 6: Apply decision constraints and write seven verdicts**

`SB-CORE-030` and `033` must cite the existing owner decisions. Do not choose a portfolio number, latency, memory ceiling, cache policy or supported hardware. All seven have no owned test ID and therefore use `test_class=MISSING` even if supporting tests exist.

---

### Task 5: Adjudicate Governance and Stewardship Requirements

**Rows:** `SB-CORE-042`, `043`, `044`

**Files:**

- Modify: `docs/takeover/evidence/sb-core.md`
- Read only: governance paths in the evidence map

- [ ] **Step 1: Run the existing characterization tests**

From `src-tauri`:

```powershell
cargo test characterizes_the_green_gate_as_machine_enforced_but_still_manually_invoked -- --nocapture
cargo test characterizes_the_tier_c_register_as_shipped_policy_with_asset_specific_design_around_routes -- --nocapture
```

Both are supporting `CHARACTERIZATION` evidence. Because their requirements have blank owned-test IDs, the ledger test class remains `MISSING`.

- [ ] **Step 2: Confirm automation and architecture searches from both sides**

Run:

```powershell
rg --files -g '.github/workflows/**' -g '*ARCHITECTURE*' -g '*ADR*' -g '*adr*' -g '*decision*' -g 'docs/superpowers/specs/**'
rg -n "tools\\check.ps1|npm run|cargo test|architecture|decision record|Tier-C|design-around" package.json tools docs src src-tauri
```

Open positive matches and exact parent directories. A zero-result glob must be confirmed with `Get-ChildItem` on the expected directory and Git history.

- [ ] **Step 3: Distinguish policy, register and enforcement**

For `044`, separately record the written Tier-C rule, the asset-specific design-around register and actual runtime/build enforcement. Policy text alone cannot prove “shipped, auditable policy” across every default.

- [ ] **Step 4: Write three complete verdicts**

Do not call takeover plans an architecture system unless they satisfy the chapter’s maintenance, discoverability and decision-record obligations. Do not call a manually invoked script automatic enforcement.

---

### Task 6: Update the Ledger Atomically and Self-Review All 25 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-core.md`

- [ ] **Step 1: Prepare all 25 ledger changes as one RFC 4180-safe edit**

Use a temporary review copy or a purpose-built PowerShell object transform only to calculate the patch; make the repository edit with `apply_patch`. Preserve column order, quoting, source-owned fields and every non-SB-CORE row byte-for-byte where practical.

- [ ] **Step 2: Enforce the row-completeness rules**

Run:

```powershell
$core = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-CORE-*' }
if (@($core | Where-Object { $_.as_built_status -eq 'UNADJUDICATED' }).Count -ne 0) {
    throw 'At least one SB-CORE row was silently skipped'
}
foreach ($row in $core) {
    foreach ($field in @('release_disposition','risk_class','implementation_paths','test_class','commit_state','next_action','last_reverified')) {
        if ([string]::IsNullOrWhiteSpace($row.$field)) {
            throw "$($row.requirement_id) is missing $field"
        }
    }
}
```

`blocking_decision`, `dependencies`, `expected_value_source` and `manual_evidence` may be empty only when the receipt explicitly says why they do not apply; prefer the literal `none` so absence is deliberate.

- [ ] **Step 3: Enforce source-owned-field immutability**

Run:

```powershell
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
```

Expected: both exit 0. These checks reject drift in chapter, title, priority, chapter status or owned-test IDs.

- [ ] **Step 4: Cross-check the evidence receipt and ledger**

For every row, verify the receipt verdict exactly matches all adjudication-owned CSV fields. Check that:

- all 25 IDs appear exactly once;
- all 14 blank-owned-test rows use `test_class=MISSING`;
- no internal-only test is counted as acceptance closure;
- every `CORRECTNESS` entry names its independent expected-value source;
- every `CHARACTERIZATION` entry says so;
- every `PRESENT-OK` verdict addresses every atomic obligation;
- every universal contract has inventory evidence, not one example;
- every `PILOT-BLOCKER`, `DEFERRED`, `OUT` or `UNDECIDED` value follows the release-disposition rules;
- no manual checkbox or field claim was promoted;
- no petrophysical number, endpoint or default was introduced.

- [ ] **Step 5: Generate the post-adjudication summary**

Run:

```powershell
node tools/takeover-ledger.mjs --summary-json
```

Expected: total remains 931; adjudicated increases by exactly 25; unadjudicated falls from 931 to 906. Report the actual pilot-disposition counts rather than predicting them in this plan.

---

### Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] **Step 1: Replace planning state with measured adjudication state**

Set:

```text
Current gate: G1 — BASELINE RECONCILIATION
Active increment: G1-DOM-CORE — SB-CORE LIVE ADJUDICATION
Accepted baseline: the eventual adjudication commit atop b332026
Pilot field evidence: OPEN
Open blockers: 906 live domain adjudications plus the measured SB-CORE findings and existing structural/claim blockers
Next increment: the next serial domain-adjudication plan selected after Jauhar reviews SB-CORE dispositions
```

Do not call Gate 1 complete. Add one recent-increment row saying `25/25 SB-CORE rows adjudicated`, the exact as-built/disposition counts and `906` rows remaining.

- [ ] **Step 2: Correct the worktree-protection statement**

State that `D:\XX. SandiBumi` is the only registered development worktree; the empty locked `D:\XX. SandiBumi-check` folder remains untouched by explicit direction and is not a worktree; previously authorized auxiliary folders have been removed. Do not repeat the stale claim that dirty contents or registered auxiliary worktrees remain.

- [ ] **Step 3: Keep evidence classes distinct**

The automated gate line receives the current run and actual counts. `Pilot field evidence` remains `OPEN` regardless of a green automated gate.

---

### Task 8: Verify, Commit the Domain Adjudication, and Stop

**Files committed:**

- `docs/takeover/evidence/sb-core.md`
- `docs/takeover/requirements.csv`
- `docs/takeover/STATUS.md`

- [ ] **Step 1: Run focused tracker and evidence checks**

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
node tools/generate-verification-matrix.mjs --check
```

- [ ] **Step 2: Run required compile checks in order**

```powershell
npx tsc --noEmit
Push-Location src-tauri
cargo check
Pop-Location
```

- [ ] **Step 3: Run the full repository gate**

```powershell
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: zero failed. Record the actual takeover-ledger, frontend, Rust passed and ignored counts. A red gate whose fix is outside these three documentation files stops this increment; do not edit production code to make an adjudication commit green.

- [ ] **Step 4: Review the complete diff**

```powershell
git diff --check
git diff --stat
git diff -- docs/takeover/evidence/sb-core.md docs/takeover/requirements.csv docs/takeover/STATUS.md
git status --short
```

Assert that no production file, PRD file, `REVIEW.md`, generated matrix or unrelated path changed. Search the receipt for unfinished markers, dummy values, invented parameter values and unqualified “verified” claims.

- [ ] **Step 5: Stage exact files and commit once**

```powershell
git add -- docs/takeover/evidence/sb-core.md docs/takeover/requirements.csv docs/takeover/STATUS.md
git diff --cached --check
git diff --cached --name-only
git commit -m "G1-DOM-CORE adjudicate 25 SB-CORE requirements"
git status --short
```

This is one domain reconciliation commit, not 25 behavior commits. Any later production work remains one requirement per serial topic branch and commit, with its own named tests and full gate. Do not push.

- [ ] **Step 6: Stop for Jauhar review**

Report:

- every SB-CORE ID and its as-built status, release disposition and bounded next action;
- the exact 25/25 adjudicated and 906/931 remaining counts;
- all owner decisions still `UNDECIDED`;
- exact automated gate counts;
- manual evidence still open;
- files changed and commit hash;
- production fixes deliberately not started.

---

## Plan Self-Review Before Approval

- [ ] Exactly 25 live SB-CORE IDs are covered once; none of the 931-row ledger is silently dropped or broadened into this domain.
- [ ] All eight original P0s and all seventeen lower/blank-scope candidates are adjudicated without treating old priority as pilot policy.
- [ ] The plan changes no production behavior and does not amend PRD v2.
- [ ] The chapter's two invalid statuses and fourteen missing owned-test IDs remain evidence, not silently repaired metadata.
- [ ] SB-CORE-040 and SB-CORE-013 are reverified from current source rather than copied from stale `ABSENT` chapter labels.
- [ ] SB-CORE-041 requires fresh-clone evidence; an existing-worktree green gate is not accepted as a substitute.
- [ ] SB-CORE-042 distinguishes a machine-runnable gate from automatic per-change enforcement.
- [ ] SB-CORE-044 distinguishes written policy, a register and exhaustive enforcement.
- [ ] Universal claims require inventory evidence and both refusal/success sides where laziness could otherwise pass.
- [ ] No test that asserts only an internal `Result` closes a reporting contract.
- [ ] Every expected value is sourced or explicitly characterized; no petrophysical value is selected.
- [ ] Git reachability, automated tests, manual checks and pilot field evidence remain separate columns of evidence.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] Only explicit Jauhar direction can set `OUT` or resolve the open portfolio/lineage decisions.
- [ ] The plan predicts no verdict count other than the mechanical 25 adjudicated / 906 remaining result.
- [ ] The plan commit itself changes zero ledger rows; execution starts only after Jauhar approves it.
