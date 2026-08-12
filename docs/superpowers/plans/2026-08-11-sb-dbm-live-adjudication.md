# SB-DBM Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 43 live `SB-DBM` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing product behavior or the database write discipline.

**Architecture:** This is a documentation-only evidence pass. The PRD remains an immutable statement of intent and historical chapter status. Current source, qualifying acceptance tests, manual evidence and reachable Git history establish the separate live verdict. One domain receipt explains every decision; `docs/takeover/requirements.csv` carries the machine-validated summary. The pass follows the durable-data chain from format/version gates through run provenance, reproducibility, model custody, store integrity, archive behavior, locking, scoping and honest results. The deliberately PK-less `computed_curves` design is preserved exactly; the adjudication evaluates whether its surrounding discipline is actually enforced.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, database schema, generated product artifacts, `REVIEW.md` or any file under `docs/PRD_v2/**`.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. The branch is stacked on the reviewed SB-DIO adjudication commit `361ee0b1ad87779547e0c24ed64b3907d9022738`; `origin/master` was frozen at `29833735816d9e5be954afafd9ceb71fd856e3f0` when this plan was written. Reverify all three before execution. If an accepted reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected source files, tests and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete database-model chapter, the applicable `docs/record_*.md` files and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. Chapter statuses are historical evidence, not current verdicts.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid pilot. Original P0/P1/P2 is evidence, not automatic pilot scope.
- A positive mechanism does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list, or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence, and uses an independently sourced expected value. A helper, schema snapshot, internal `Result`, source-text grep or compile success is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Do not treat a named test in the PRD as an implemented test until its body is found and executed.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked.
- Preserve `computed_curves` as deliberately PK-less. Never recommend or implement a primary key, `ON CONFLICT`, upsert, duplicate-tolerant writer, or delegated database write discipline. Verify DELETE-then-append and transaction boundaries as they exist.
- Preserve one single writer. `SB-DBM-036` concerns lock hold duration, not adding concurrent writers. A proposal that "fixes" the row by changing the writer model is non-conforming.
- Keep missing sample data as `f32::NAN`, never `Option<f32>` in numeric arrays. `SB-DBM-030`'s chapter wording about `Option<f64>` is a specification statement to adjudicate, not authorization to break the repository's binding array contract.
- The frontend never sends SQL for writes. The read-only SQL console is evidence for refusal boundaries, not an alternate write path.
- A petrophysical value is cited or absent. This chapter ships no petrophysical parameters. Do not use its schema constants as permission to select a rock/fluid endpoint.
- `SAMPLING_STYLE_VERIFY_TOLERANCE`, `INTERACTIVE_SET_CEILING`, `MODULE_VERSION_SOURCE`, `ARTIFACT_HASH_ALGORITHM`, `AUTOSAVE_INTERVAL` and the unresolved UTC migration are named gaps or decisions. Record them; do not invent values or policies.
- `SB-DBM-028` cannot be closed by checking a gap without a cited verification tolerance. The test accepts the tolerance as input; a production default remains absent.
- `SB-DBM-038` cannot be closed by synthetic timing, a source comment or a small fixture. Its chapter test requires a real scale curve at N in `{100, 500, 1000, 2000, 5000}` and the measured ceiling is explicitly absent.
- Model-store rows `SB-DBM-018` through `SB-DBM-022` may require reading `ml.rs`; reading is allowed for evidence, editing is not. This adjudication makes no ML behavior change.
- Git reachability proves only that a change is in the accepted tree. Commit messages are locators, never correctness evidence; open the accepted source and the test body.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- The approved plan authorizes only this plan, the DBM evidence receipt, the 43 ledger-row adjudications and the dashboard handoff. It does not authorize a schema migration, new test, source fix, parameter choice or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 43 SB-DBM rows:

```text
SB-DBM-001 SB-DBM-002 SB-DBM-003 SB-DBM-004 SB-DBM-005 SB-DBM-006 SB-DBM-007
SB-DBM-008 SB-DBM-009 SB-DBM-010 SB-DBM-011 SB-DBM-012 SB-DBM-013 SB-DBM-014
SB-DBM-015 SB-DBM-016 SB-DBM-017 SB-DBM-018 SB-DBM-019 SB-DBM-020 SB-DBM-021
SB-DBM-022 SB-DBM-023 SB-DBM-024 SB-DBM-025 SB-DBM-026 SB-DBM-027 SB-DBM-028
SB-DBM-029 SB-DBM-030 SB-DBM-031 SB-DBM-032 SB-DBM-033 SB-DBM-034 SB-DBM-035
SB-DBM-036 SB-DBM-037 SB-DBM-038 SB-DBM-039 SB-DBM-040 SB-DBM-041 SB-DBM-042
SB-DBM-043
```

At plan time all 43 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is seventeen P0, eighteen P1 and eight P2. Gate 1 adjudicates all 43 because a lower historical priority can still determine whether project state, provenance or recovery is trustworthy.

Run this guard before and after editing:

```powershell
$dbm = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-DBM-*' }
$expected = 1..43 | ForEach-Object { 'SB-DBM-{0:D3}' -f $_ }
if ($dbm.Count -ne 43) { throw "Expected 43 SB-DBM rows, found $($dbm.Count)" }
if (@(Compare-Object $expected @($dbm.requirement_id)).Count -ne 0) {
    throw 'The live SB-DBM ID set differs from the approved plan'
}
```

The mechanical post-execution count is exactly 131 adjudicated and 800 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-dbm.md` - complete 43-row evidence receipt, including obligation-by-obligation source findings, tests, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 43 SB-DBM rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary and next serial-domain handoff.

### Read-only governing inputs

- `AGENTS.md`, `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, `04_CORE_REQUIREMENTS.md`, `06_SEQUENCING_AND_GATES.md`, `22_database-model.md`, `91_REQUIREMENTS_INDEX.md`
- `docs/record_data_tools.md`, `record_fixes.md`, `record_parallel_lanes.md`, `record_calibration.md`, `record_core_depth.md`
- `docs/takeover/DECISIONS.md`, `CLAIMS.md` and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Files this adjudication MUST NOT change

- every read-only governing input above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/`;
- `REVIEW.md` and its generated verification matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-dbm.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness and 43-row guard result. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-DBM-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, owned test IDs and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source and class.
- Supporting tests: tests that help but cannot close the owned contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or UNIMPLEMENTED; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, source, dependency or none.
- Next action: one bounded production, test, field, decision or no-action increment.
```

Copy each receipt verdict into only these adjudication-owned ledger fields:

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

Use this exact implementation verification stamp:

```text
2026-08-11 @ b332026cb498c105f36eade0bf7899bc0c1309f0
```

The docs-only plan or adjudication commit is never implementation evidence.

---

## Requirement Evidence Map

These are inspection maps, not pre-decided verdicts. Each row must be expanded until every atomic obligation is answered.

### Run provenance and structured audit (`001`-`013`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `001` | Every computed value resolves in one hop or is explicitly labelled legacy. | `log_sets`, current/archive writers, legacy reads/exports and T03/T10. | A nullable `set_id` plus a comment does not prove the user sees `LEGACY_UNRECORDED`. |
| `002`, `004`, `005` | Run records pin build-derived module identity, effective parameters/default provenance and method derivation. | `LogSetSpec`, manifest metadata, equation/module writers, registered module metadata and T04/T06/T07/T15. | Names, overrides-only JSON and repository comments are not durable per-run evidence. `MODULE_VERSION_SOURCE` is deliberately absent. |
| `003`, `007` | Parameter value/source/absent state is structured and queryable; empty strings cannot stand for state. | `params_json`, `zone_params`, run writers/readers and T05/T09/T30. | Do not treat one free-form JSON value map as a queryable source registry. |
| `006` | Each input records chosen curve identity/version, decision rule and rejected candidates. | All curve-resolution paths, `inputs_json`, set/version identity and T08. | A requested mnemonic or input set is not a resolved identity. |
| `008`, `009` | Operator, zone-set version and UTC provenance timestamps are durable. | Schema, run creation, display conversion and T11. | The UTC migration for existing local timestamps is a named unresolved decision; do not backfill by assumption. |
| `010` | Provenance reaches computed-curve exports in a machine-readable sidecar and names omissions. | Writer registry, LAS/office export paths, `log_sets` reads and T10. | An in-project inspector does not satisfy deliverable propagation. |
| `011`, `012` | Audit entries/details are relational, controlled and diffable without an external process. | `processLog.ts`, `documents`, audit tables/queries, Save As and T11/T12. | A JSON text blob is supporting history, not structured audit. |
| `013` | Provenance is not optional and its write is atomic with the curve. | Versioned writers, transactions, settings/env inventory and T13. | The atomic path must cover every computed-curve writer, not one module example. |

Manual capability candidates: `delivery-sets`, `generic-curve-store`, `workflow`, `processing-history`, `las-export`, `office-deliverables`, `security-integrity`, `verification-stewardship`.

### Reproducibility and physics-driving state (`014`-`017`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `014` | Every stochastic run stores root seed, derivation rule and generator identity sufficient for bitwise replay. | Monte Carlo and ML seed handling, run record and T14/T15. | A seed exposed in a dialog is not a stored general seeding rule. |
| `015` | One complete re-run manifest exists and refuses each unresolved element by name. | Run schema, re-run command, module/parameter/input/frame/zone/seed/model/attribute identity and T15/T16. | Individually present fragments do not create an enumerated manifest. |
| `016` | Output is byte-stable across unordered traversal and query row-order changes. | Explicit `ORDER BY`, deterministic collection handling, aggregate order and T16. | One deterministic fixture in one process is supporting evidence only. |
| `017` | Physics-driving attributes are declared module inputs, stored, stale outputs on change and fail named when unset. | Well/set/curve metadata, manifest declarations, invalidation and T17. | Do not invent a contractor/tool-response default to make the test executable. |

Manual capability candidates: `workflow`, `monte-carlo`, `machine-learning`, `well-scope`, `verification-stewardship`.

### Learned-model store and custody (`018`-`022`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `018` | Training identity uses stable well ids, intervals and training-curve set versions. | `ml_models`, train/save/apply requests, rename/delete behavior and T18/T20. | Well names and `n_train` cannot distinguish intervals or survive a rename. |
| `019` | Model rows store seed/generator, full numerical library set and verified artifact hash. | model schema, Python runner metadata, artifact write/load and T19/T21. | `ARTIFACT_HASH_ALGORITHM` is deliberately absent; do not choose one in adjudication. |
| `020` | Train-and-apply and apply-saved both persist and stamp a resolvable `model_id`. | both `ml.rs` paths, `log_sets`, `ml_models` and T20. | An algorithm name is not a model identity. |
| `021` | Only native artifacts can enter/apply; origin is non-user-settable and every external path refuses. | schema, commands/dialogs, read-only SQL boundary, artifact loader and T21. | Absence of an import button alone does not make origin schema-enforced. |
| `022` | Apply verifies feature membership and order and fails each well by name. | stored ordered features, resolve/apply loop and T22. | A missing-feature test does not prove reordered complete features are refused. |

Manual capability candidates: `machine-learning`, `generic-curve-store`, `security-integrity`, `verification-stewardship`.

### One definition, capacity and registered constants (`023`-`025`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `023` | Schema vocabularies have one registry and every projection derives from it. | exact inventory of `STANDARD_COLUMNS`, frame/sampling/datum/audit/absent literals and T23. | Two lists that currently agree would still violate derivation; exclusions such as DEPTH must be derived. |
| `024` | Limits/tolerances are unit-typed, sourced and generate their documentation. | source constant inventory, doc generation and T24. | Current bare literals are characterization evidence, not conformance. |
| `025` | Cross-module petrophysical constants resolve through a sourced registry. | modules/registry inventory and T23/T24. | This requirement specifies a mechanism only; do not populate it with values from this chapter. |

Manual capability candidates: `data-conventions`, `security-integrity`, `verification-stewardship`.

### Store integrity, frame identity and archive (`026`-`035`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `026`, `027` | Continuous duplicates refuse, POINT duplicates remain legal by declaration, and a checker reports every dangling/duplicate class including zero counts. | every computed/native/array writer, set-type schema, integrity commands and T25/T26. | The current delete-first discipline does not prove duplicate refusal; do not add a PK or upsert. |
| `028` | Sampling style is stored and verified against the reference column before frame-indexed reads. | ingest preflight, log-set schema, read guard and T27. | `SAMPLING_STYLE_VERIFY_TOLERANCE` ships absent. A hard-coded tolerance is forbidden. |
| `029` | Module output cannot mutate the reference column; a new basis creates `OWN`. | output-name resolver, edit paths, Reframe boundary and T28. | Shadow-name refusal may protect one path while another edit list omits DEPTH. |
| `030` | Large-negative null detection is threshold-based and sample absence stays distinct from parameter absence across layers. | parser/store/IPC/UI/export paths and T29/T30. | Reconcile the chapter test with binding `f32::NAN` array IPC; never introduce `Option<f32>`. |
| `031` | Every depth carries datum and cross-datum comparison refuses without a frame. | schema, survey/reference transforms, tops/contacts/plot joins and T31. | A depth unit is not a datum. |
| `032` | Persisted parameters use ordinal plus semantic key and retain unit/tilt; mismatch is a hard error. | parameter file/store/load paths and T32. | Zone-name plus param-name storage is not the required dual handle. |
| `033` | Categorical curves have distinct storage/arithmetic/resampling behavior. | curve types, resampler, equation/module inputs and T33. | An integer-looking float heuristic is not a categorical type. |
| `034` | Every bulk operation returns matched/unmatched/ambiguous and queues every exception. | inventory all bulk import/paste/match commands and T34. | One import result with unmatched wells cannot close the universal claim. |
| `035` | Archive is append-only and restore creates a new version recording its source. | all archive SQL, restore commands, version writer and T35. | A comment saying restore is possible is not a restore operation or an append-only guard. |

Manual capability candidates: `data-conventions`, `delivery-sets`, `reframe`, `core-depth-registration`, `correlation-tops`, `curve-editing`, `security-integrity`, `project-lifecycle`.

### Concurrency, scope, scale, honest results and recovery (`036`-`043`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `036` | Single writer remains, but latency of an interactive command does not scale with batch size. | command/lock inventory, long-operation boundaries and T36/T38. | Counting lock calls is evidence of exposure, not the required latency distribution. |
| `037` | Every backend well iterator enforces active-group scope or declares itself project-wide. | active-group store, command/query inventory and T37. | A frontend-filtered list does not prove direct backend invocation is scoped. |
| `038` | Project queries materialize only the interactive set and publish a real scale curve. | project open/list/facet/first-paint paths and T38. | Hard block on real scale evidence and a measured ceiling; no inherited 2,000-well default. |
| `039` | Clean, warned and failed are distinct and degradation persists after job pruning. | `jobs.rs`, workflow/equation results, `log_sets`, prune behavior and T39/T41. | A transient `Warned` state does not satisfy durable provenance. |
| `040` | Cancellation honesty is pinned from both halves. | worker-observed cancel, `cancellable` view/UI and T40. | Verify existing tests match the complete sentence; do not count an internal flag alone. |
| `041` | Total count has one meaning and the inspector exposes every provenance table needed to trace a curve. | `TablePage`, inspector whitelist/queries and T41/T42. | Fixing the count without the table inventory remains partial. |
| `042` | Newer-version refusal is byte-preserving; destructive backup is fail-closed/non-overwriting/user-visible; additive migration skips backup; filename names source version. | format gate, every migration, backup/copy code and T01/T02/T43. | The shipped safety mechanism may be OK while the naming clause diverges; classify the whole row accordingly. |
| `043` | Deterministic uncapped sweep stores every trial under full provenance. | sweep/trial schema and runners, depth-cap search and T44. | Method-specific tuning is outside this domain; no generic store means absent. |

Manual capability candidates: `portfolio-performance`, `well-scope`, `processing-history`, `database-tools`, `project-lifecycle`, `workflow`, `security-integrity`, `verification-stewardship`.

---

### Task 1: Freeze Evidence and Create the 43-Row Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-dbm.md`
- Read only: all governing and evidence paths listed above

- [ ] Reverify branch, base, accepted anchor, origin and cleanliness.
- [ ] Run the exact 43-row guard and assert every row is still `UNADJUDICATED`.
- [ ] Build exact source/test/manual/Git inventories; no verdict is copied from the chapter.
- [ ] Create one heading for each `SB-DBM-001` through `SB-DBM-043` with `apply_patch`; do not commit an empty skeleton.
- [ ] Machine-check that all 43 headings are unique and in order.

### Task 2: Adjudicate Run Provenance, Audit and Reproducibility

**Rows:** `SB-DBM-001` through `SB-DBM-017`

- [ ] Trace every computed-curve writer and every run-record reader/exporter; inventory nullable legacy states.
- [ ] Open every candidate implementation of T03-T17, classify its expected-value source and run every qualifying filter. A zero-test filter is missing evidence.
- [ ] Separate effective values from overrides, method source from parameter source, requested mnemonic from resolved curve identity, and in-project history from deliverable provenance.
- [ ] Verify atomicity and configuration inventory for `013` across all writers.
- [ ] Preserve named gaps for module-version form and UTC migration; do not choose either.
- [ ] Write seventeen complete receipt verdicts.

### Task 3: Adjudicate Learned Models and One-Definition Contracts

**Rows:** `SB-DBM-018` through `SB-DBM-025`

- [ ] Read both ML train/apply paths and model schema without editing them.
- [ ] Verify stable ids, intervals, training set versions, seed/library/hash metadata, origin custody and feature order independently.
- [ ] Open and run T18-T24 candidates; distinguish missing-feature support from reordered-feature proof.
- [ ] Inventory schema vocabularies and bare limits across Rust, TypeScript and docs. A static comparison is supporting evidence unless projections derive from one registry.
- [ ] Keep every rock/fluid value outside this chapter; `025` can be judged only as a registry mechanism.
- [ ] Write eight complete receipt verdicts.

### Task 4: Adjudicate Store Integrity and Archive Behavior

**Rows:** `SB-DBM-026` through `SB-DBM-035`

- [ ] Inventory every store's key posture and every writer. Preserve the measured reason for PK-less `computed_curves` and examine only the enforcing discipline around it.
- [ ] Open/run T25-T35 candidates. Pin duplicate refusal versus POINT acceptance, named zero-count integrity classes, sampling-style contradiction, DEPTH refusal, null boundary, datum refusal, dual-handle failure, categorical behavior, bulk exception counts and archive restore.
- [ ] Treat `SAMPLING_STYLE_VERIFY_TOLERANCE` as absent; a reader or fixture may take one as input, but no production default is inferred.
- [ ] Reconcile `SB-DBM-030` with binding `f32::NAN`/byte-IPC rules explicitly in the receipt; record specification friction rather than changing code or PRD.
- [ ] Search for all archive UPDATE/DELETE and all restore paths; a comment is not a command.
- [ ] Write ten complete receipt verdicts.

### Task 5: Adjudicate Concurrency, Scope, Scale, Honest Results and Recovery

**Rows:** `SB-DBM-036` through `SB-DBM-043`

- [ ] Rebuild lock and well-iterator inventories from current source; distinguish single-writer correctness from unbounded hold time.
- [ ] Open/run T36-T44 candidates. Do not substitute source counts or synthetic timings for the required real-well scale curve.
- [ ] Trace degradation from transient jobs into durable records, and cancellation from worker observation into UI controls.
- [ ] Verify both halves of the inspector requirement and every clause of the format/backup requirement, including byte identity and source-version naming.
- [ ] Search for sweep/trial stores and caps; record absence without inventing method parameters.
- [ ] Write eight complete receipt verdicts.

### Task 6: Update the Ledger Atomically and Self-Review All 43 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-dbm.md`

- [ ] Prepare all 43 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-DBM rows and source-owned fields.
- [ ] Enforce that no DBM row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 43 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every block names its dependency, and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 131 adjudicated, 800 remaining.

### Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 43-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, hard evidence blocks and `800/931` rows remaining.
- [ ] Name the next serial domain only as a recommendation; do not start it.

### Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, PRD-audit check and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-dbm.md`, `docs/takeover/requirements.csv` and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-DBM adjudicate 43 SB-DBM requirements`; do not push, merge or begin a production fix.

---

## Plan Self-Review Before Execution

- [ ] Exactly 43 live SB-DBM IDs are covered once; no row can be silently skipped.
- [ ] All seventeen original P0s and twenty-six lower-priority rows are adjudicated without treating old priority as pilot policy.
- [ ] The plan changes no production behavior and does not amend PRD v2.
- [ ] The deliberately PK-less `computed_curves` design and single-writer model are preserved.
- [ ] Run provenance, parameter provenance, method provenance and deliverable provenance remain separate obligations.
- [ ] `SAMPLING_STYLE_VERIFY_TOLERANCE` and `INTERACTIVE_SET_CEILING` remain absent until their specified evidence exists.
- [ ] `MODULE_VERSION_SOURCE`, `ARTIFACT_HASH_ALGORITHM` and UTC legacy migration remain named implementation/owner decisions, not guessed defaults.
- [ ] The ML source is read-only and model-artifact custody is assessed at the store boundary.
- [ ] No helper, schema comment, compile check or Git message closes an observable contract by itself.
- [ ] Every expected value is sourced or explicitly characterized; no petrophysical value is selected.
- [ ] Manual evidence, automated tests, accepted Git reachability and pilot field evidence remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 131 adjudicated / 800 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution proceeds only under Jauhar's existing approval.
