# SB-INS Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 26 live `SB-INS` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 30 chapter acceptance-test intentions, and preserve the Windows-first, device-wide, offline-deployment, dependency, configuration, unit, evidence-firewall, legal, recovery, provenance, and manual-evidence boundaries without changing production behavior.

**Architecture:** This is a documentation-only evidence pass. Requirements 001-009 cover the MSI, Python independence, truthful capability prerequisites, the capability manifest, interpreter resolution, package preflight/remediation, offline deployment, and runtime attestation. Requirements 010-020 cover immutable templates, reversible migration, corporate policy, precedence, parameter-pack identity/refusal, typed units, raw tokens, missing mappings, registry generation, and pack attestation. Requirements 021-026 cover support reporting, upgrade/uninstall preservation, clean-machine release gating, third-party obligations, the evidence-acquisition firewall, and executable release claims. The immutable PRD supplies the intended contract, 18 parameter rows, six open items, four escalations, six refusals, 30 named test intentions, and 95 section-8 traceability rows. Current source, independently meaningful tests, manual evidence, and reachable Git history supply the separate live verdict. A validator exercised only with invented fixtures is plumbing evidence, not proof of a signed MSI, offline runtime pack, real network isolation, or clean-machine qualification.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, production source, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on GPT-5.6 Sol at xhigh with `superpowers:executing-plans`. Do not delegate or spawn subagents unless Jauhar explicitly authorizes it. Installer qualification, offline-runtime scope, data-integrity classification, legal boundaries, and final sign-off stay with the primary session. Reserve Sol max for the final all-931-row Gate 1 audit.
- Work only in `D:\XX. SandiBumi`. It MUST remain the sole registered Git worktree. The retired `D:\XX. SandiBumi-check` path remains untouched.
- The accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `86dae286e0cff1313532b7af879ba4937c588621`; `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution.
- The local planning branch is `codex/g1-sb-ins-plan`. The serial Gate 1 chain remains local and unpushed; do not merge, rebase, rewrite history, push, or open a pull request. After the planning commit, create `codex/g1-sb-ins-adjudication` in the same worktree.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential Rust/TypeScript absence MUST be confirmed across Tauri bundling, release scripts, command registration, Python consumers, package probes, UI actions, configuration loading, unit ingestion, support/report surfaces, tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/27_ip-install-blockers.md`, `docs/record_data_tools.md`, `docs/record_parallel_lanes.md`, the current verification matrix, takeover receipts/status, and the exact source/tests about to be cited.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 26 ordered rows is `de7fd60a70bde2187130069d4cf16ac522cf6b6b891e1507428e37c1a076f1ce`. The immutable chapter SHA-256 is `44947f129d6b2c2867d10fac707d144a167a0fde71c38839e5b4937538b9e462`.
- The chapter and ledger agree on 26 contiguous requirements: `P0=11` and `blank priority=15`. Historical states are `ABSENT=15`, `PARTIAL=8`, `PRESENT-OK=2`, and `PRESENT-DIVERGENT=1`; all 26 live verdicts remain `UNADJUDICATED`. Reverify live behavior independently rather than copying those labels.
- The chapter defines 30 contiguous test IDs, `SB-INS-T01` through `SB-INS-T30`. All ledger `owned_tests` cells are source-owned blanks, so route the chapter intentions in the receipt without altering those immutable cells.
- Section 5 contains 18 parameter rows, including five deliberate `ABSENT` values. Preserve every absence and source fence. Do not invent a Python/package minimum, signed-pack version, release-lock digest, supported Windows release list, corporate-precedence rule, migration rule, vulnerability decision, licence approval, unit alias, registry dimension, or configuration-pack compatibility range.
- Preserve O-1 through O-6, E-1 through E-4, R-1 through R-6, all 95 traceability rows, and the independent-derivation boundary. The product owner has resolved the deployment direction as per-machine MSI for IT/system deployment with standard-user launch; a separately signed, versioned, application-local qualified Python pack; and every Microsoft-serviced Windows 11 x64 Pro/Enterprise feature release at release time. Those decisions do not create the missing MSI, pack, lock, servicing snapshot, network trace, legal approval, or qualification matrix.
- Linux remains held for a later product version. This Windows-first adjudication MUST NOT claim cross-platform support or create a Linux disposition.
- `f32::NAN`, raw bytemuck array bytes over IPC, `parsers::read_text_file`, subprocess Python with `sys.stdin.buffer`, undoable edits, and the existing DuckDB write discipline remain mandatory. This documentation lane must not change or reinterpret them.
- Do not count an internal `Result`, fixture-populated validator, generated prose helper, storage-only round trip, or source-string assertion as a whole-contract release proof. In particular, `installation_gate.rs` is an example CLI, not a CI or publishing gate; `parameter_pack.rs` has no production caller; and `validate_unit_registry` is test-only.
- Manual evidence remains separate. All 16 SB-INS-specific `REVIEW.md` scenarios are unchecked. Installation/deployment is not listed as a verification-matrix capability and therefore remains `0/0`, not complete. Supporting capability counts remain DLIS `0/11`, data conventions `0/45`, equation engine `0/11`, office deliverables `0/39`, workspace shell `0/159`, project lifecycle `3/24`, security integrity `0/63`, verification stewardship `0/24`, and LAS export `0/2`.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to generic deployment conditions, configuration records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record:

1. branch `codex/g1-sb-ins-adjudication`, created serially from the committed plan;
2. one clean worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, `origin/master`, merge base, and anchor reachability;
4. exactly 26 ledger rows, SB-INS-001 through SB-INS-026, with no gap or duplicate;
5. priorities `P0=11` and `blank=15`;
6. historical source states `ABSENT=15`, `PARTIAL=8`, `PRESENT-OK=2`, and `PRESENT-DIVERGENT=1`;
7. all 26 live mutable evidence fields still unadjudicated or placeholder-only;
8. exactly 30 defined chapter test IDs and blank source-owned ledger ownership cells;
9. exactly 18 parameter rows, including five deliberate absent values;
10. exactly six open items, four escalations, six refusals, and 95 section-8 traceability rows;
11. takeover summary `853` adjudicated, `78` unadjudicated, and `562` pilot blockers before SB-INS;
12. no actual signed MSI, qualified Python pack, version/digest release lock, blocked-network trace, Microsoft servicing inventory, clean-machine result set, or release-pipeline invocation in the accepted tree;
13. the manual evidence counts listed above; and
14. fresh focused evidence: installation `10/0/0`, parameter-pack `3/0/0`, Python environment `3/0/0`, typed-unit registry `7/0/0`, and encoding `4/0/1` before the plan edit.

The mechanically predictable post-adjudication ledger counts are `879` adjudicated and `52` unadjudicated. The preliminary release map is 22 rows as `PILOT-BLOCKER` and four unresolved product/governance rows—012, 013, 020, and 025—as `UNDECIDED`; reverify each row before writing it. This preliminary map would produce 584 total pilot blockers, 198 undecided, and 149 deferred. Do not freeze a verdict or disposition merely to make the receipt match this plan.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-ins.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/tauri.conf.json`, `src-tauri/src/installation.rs`, `src-tauri/src/python_engine.rs`, `src-tauri/src/parameter_pack.rs`, `src-tauri/src/curves.rs`, `src-tauri/src/parsers.rs`, `src-tauri/src/dlis.rs`, `src-tauri/src/office.rs`, `src-tauri/src/images.rs`, `src-tauri/src/lib.rs`, `src-tauri/examples/installation_gate.rs`, `src/ui/installationSupportDialog.ts`, installation resources, release/license scripts and copy, applicable tests, manual evidence, immutable source chapter, historical implementation records, and reachable Git history
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected files, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-INS-NNN` section per requirement in numeric order. Every section MUST include the specified contract, current implementation, as-built status, release disposition/risk, exact automated evidence class, manual evidence, parameter/source boundary, deployment/UI/provenance surface, history/reachability, blocking decision/dependency, and next action. Separate every observable limb of a compound requirement. Name whether evidence is correctness, characterization, optional-package, structural, manual, or missing; never inflate a fixture-populated validator, helper-level test, internal `Result`, source-string check, or generated fragment into signed-release or clean-machine proof.

## Requirement Evidence Map

| ID | Tests | Exact contract focus | Primary live candidates and adjudication guard |
|---|---|---|---|
| `001` | T01 | Qualified signed per-machine MSI, standard-user install/launch, exact identity/version/digest/provenance | Tauri targets MSI and a fixture validator checks the contract, but no built/signed artifact or clean-machine execution evidence exists. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `002` | T02 | No-Python launch, project open, native computation, plotting, and native export | The focused test executes all four native paths with no selected interpreter and pins Python-only unavailability. Candidate `PRESENT-OK`, `PILOT-BLOCKER` until field qualification, `DEPLOYMENT`, test `CORRECTNESS`. |
| `003` | T03 | Every public prerequisite surface derives truthful capability-level copy | Generated README block, installer resource, release prerequisite page, Tauri copy, and support UI agree for the manifest subset, but other public documents retain contradictory/open Python statements. Candidate `PRESENT-DIVERGENT`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `004` | T04 | One complete machine-readable capability/dependency manifest consumed everywhere | A six-capability manifest exists, but it omits actual Python-backed consumers, cites no package minimums, has no release lock, and the test proves only its own expected six-row universe. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `005` | T05-T06 | One session interpreter with explainable precedence and shared probes | Resolver selection/rejection provenance and shared consumer route exist and focused tests cover both lower-valid-candidate and explicit override cases. Candidate `PRESENT-OK`, `PILOT-BLOCKER` until clean-machine evidence, `DEPLOYMENT`, test `CORRECTNESS`. |
| `006` | T07-T09 | Package/version preflight before every costly or destructive Python workflow | DLIS, equation, workbook, and deck surfaces have supporting probes, but the product-wide path is incomplete and document export can still reach work without equivalent UI preflight. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `007` | T07-T09 | Exact package and interpreter, copyable command, and re-probe action in every refusal | A helper builds correct remediation prose, but the real UI does not provide a re-probe control and not every capability message carries the helper output. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `008` | T10 | Signed offline route for every claimed capability, silent install, zero public network | Offline WebView configuration and a fixture validator exist; no qualified pack, release lock, silent-install run, or captured network trace exists. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `009` | T11 | Exact version, digest, licence, vulnerability review, and external-runtime labeling | The chosen offline pack makes this active, but no pack inventory, lock, digest set, vulnerability review, or release evidence exists. Candidate `ABSENT`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `010` | T12 | Immutable installed template plus first-run user copy with origin version/digest | The resource, materializer, Tauri command wiring, and both-side test exist; settings remain empty rather than inventing defaults. Candidate `PRESENT-OK`, `PILOT-BLOCKER` until installed-machine evidence, `DATA-INTEGRITY`, test `CORRECTNESS`. |
| `011` | T13 | Explicit reversible, classified configuration migration | No versioned inventory/migration/report/backup path exists. Candidate `ABSENT`, `PILOT-BLOCKER`, `RECOVERY`, test `MISSING`. |
| `012` | T14 | Read-only signed corporate policy for runtime, packs, locations, and disabled capability | No policy layer exists and the policy authority/format remains an open decision. Candidate `ABSENT`, `UNDECIDED`, `DATA-INTEGRITY`, test `MISSING`. |
| `013` | T14-T15 | Deterministic visible precedence across four layers with shadowed values | No four-layer resolver or support surface exists; O-3/E-2 remain open. Candidate `ABSENT`, `UNDECIDED`, `DATA-INTEGRITY`, test `MISSING`. |
| `014` | T16 | Stable semantic identifier, schema version, and ordinal; duplicate display labels allowed only for unique keys | The typed loader and direct unit test implement the isolated rule, but no production caller activates a parameter pack. Candidate `PARTIAL`, `PILOT-BLOCKER`, `SILENT-WRONGNESS`, whole-contract test `MISSING`; do not promote an unreachable loader test to product proof. |
| `015` | T17-T18 | Refuse identifier/ordinal disagreement, missing ordinal, duplicate/empty key, and unsupported schema before use | The isolated loader names and refuses these fixtures, but no product activation path reaches it. Candidate `PARTIAL`, `PILOT-BLOCKER`, `SILENT-WRONGNESS`, whole-contract test `MISSING`. |
| `016` | T19-T20 | Quantity-kind typing, exact same-kind conversion, and cross-kind refusal | Typed registry and both-side tests cover the permeability/length refusal and cited length/slowness bridges. Candidate `PRESENT-OK`, `PILOT-BLOCKER` until product-wide registry validation is enforced, `SILENT-WRONGNESS`, test `CORRECTNESS`. |
| `017` | T21-T22 | Preserve raw unit token, decoded encoding, and canonical interpretation before explicit aliases | Parser exposes raw tokens and encoding fragments, but they are not preserved end to end and global case folding collapses `mV`/`mv` without an explicit alias. Candidate `PRESENT-DIVERGENT`, `PILOT-BLOCKER`, `DATA-INTEGRITY`, test `CHARACTERIZATION`. |
| `018` | T23 | All missing/empty/placeholder unit representations converge on one explicit missing-unit state | Current registry returns no mapping for the characterized spellings, but no single explicit state or full ingest contract exists. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DATA-INTEGRITY`, test `CHARACTERIZATION`. |
| `019` | T24 | One versioned registry generates runtime, UI, documentation, and tests; release fails on drift | The registry remains code-resident, its validator is test-only, and no generated artifact/release comparison exists. Candidate `ABSENT`, `PILOT-BLOCKER`, `SILENT-WRONGNESS`, test `MISSING`. |
| `020` | T25 | Every configuration pack carries version, digest, source class, time, compatibility, and new provenance on mutation | Only schema/version fragments exist; no full attestation or computation-time event exists and no live pack route currently ships. Candidate `PARTIAL`, `UNDECIDED`, `DATA-INTEGRITY`, test `MISSING`. |
| `021` | T26 | One-action redacted support report with complete build, installer, OS, configuration, interpreter, package, and capability identity | Runtime fragments serialize, but release identity, OS, installer, configuration layers, digests, and redacted diagnostics are absent. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `CHARACTERIZATION`. |
| `022` | T27 | Upgrade, rollback, and uninstall preserve user data unless separate enumerated consent plus recovery exists | No lifecycle preservation or recoverable-removal path exists. Candidate `ABSENT`, `PILOT-BLOCKER`, `RECOVERY`, test `MISSING`. |
| `023` | T28 | Release-time matrix across every serviced Windows 11 x64 Pro/Enterprise target and nine required scenarios | A complete-cross-product validator rejects failing or missing fixture rows, but no Microsoft servicing snapshot, scenario runner, CI/publication gate, or real result set exists. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `024` | T11/T29 | Regenerate distributed dependency inventory, fail on missing licence, enumerate optional runtimes, and record human legal approval | The generator inventories declared dependencies but exits successfully on unknown licences, excludes Python as not distributed, and records neither pack/vulnerability state nor human approval. Candidate `PRESENT-DIVERGENT`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |
| `025` | T30 | Never parse opaque/proprietary artifacts to recover methods/defaults; inventory presence only | Current source search finds no forbidden parser and aligns with the firewall, but no executable guard or owned regression proves the boundary. Candidate `PRESENT-UNVERIFIED`, `UNDECIDED`, `DATA-INTEGRITY`, test `MISSING`. |
| `026` | T03 | Installer feature list, prerequisite table, and in-app matrix derive from manifest plus tested probes and fail release on false claims | Generated fragments share the current incomplete manifest, but no release-claim gate covers every public surface or actual optional capability. Candidate `PARTIAL`, `PILOT-BLOCKER`, `DEPLOYMENT`, test `MISSING`. |

Preliminary as-built totals are six `ABSENT`, 12 `PARTIAL`, four `PRESENT-OK`, three `PRESENT-DIVERGENT`, and one `PRESENT-UNVERIFIED`. Preliminary test classes are four `CORRECTNESS`, three `CHARACTERIZATION`, and 19 `MISSING` qualifying whole-contract proofs. Reverify every total from the final rows rather than hard-coding it into the ledger validator.

## Task 1: Re-establish the accepted SB-INS baseline

**Files:**
- Read: all paths listed above
- Modify: none

- [ ] Confirm the sole worktree, exact branch, clean status, current/anchor/master/merge-base commits, and accepted-anchor reachability.
- [ ] Recompute requirement, priority, historical-state, test, parameter, open/escalation/refusal, traceability, manual-evidence, source-hash, and chapter-hash counts.
- [ ] Search current source and reachable history for actual MSI, offline pack, release lock, release-pipeline hook, servicing inventory, real clean-machine evidence, corporate policy, migration, lifecycle preservation, registry generation, pack attestation, legal approval, support report, and firewall guard.
- [ ] Re-run the five focused test filters and retain exact passed/failed/ignored counts. Record why validator fixtures and source-string assertions do not prove deployment.
- [ ] Stop before editing if source-owned bytes changed, a second worktree exists, the accepted anchor is unreachable, or a cited parameter is required but absent.

## Task 2: Write the complete SB-INS evidence receipt

**Files:**
- Create: `docs/takeover/evidence/sb-ins.md`

- [ ] Record the baseline, deployment-owner decisions, manual boundary, test-class rules, source hashes, and harsh-truth summary.
- [ ] Add exactly 26 requirement sections using the receipt schema and evidence map above.
- [ ] Route T01-T30 exactly once while preserving multi-owner references at T03, T07-T09, T11, and T14.
- [ ] Route all 18 parameter rows, retaining all five deliberate absences and every source/custody distinction.
- [ ] Route O-1 through O-6, E-1 through E-4, R-1 through R-6, and all 95 section-8 traceability rows. Mark owner-resolved direction separately from still-missing executable evidence.
- [ ] State explicitly that Linux is outside this Windows-first increment and that no cross-platform qualification claim is made.
- [ ] State the exact manual evidence: 0/16 SB-INS scenarios and installation/deployment 0/0 not listed, plus the supporting capability counts.
- [ ] Distinguish implementation, automated evidence, manual evidence, and release disposition in every row.

## Task 3: Update the ledger and one-minute dashboard

**Files:**
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`

- [ ] Update only mutable evidence fields for the 26 SB-INS rows: `as_built_status`, `release_disposition`, `risk_class`, `implementation_paths`, `automated_evidence`, `automated_test_class`, `manual_evidence`, `source_parameter_state`, `ui_ipc_surface`, `provenance_state`, `history_reachability`, `blocking_decision`, `next_action`, `notes`, `evidence_receipt`, `last_reverified`, and `commit_state`.
- [ ] Preserve all source-owned fields byte-for-byte and recompute the frozen source hash.
- [ ] Recompute all ledger totals from the CSV. Expected only if row evidence remains as mapped: `879/931` adjudicated, `52` unadjudicated, `584` pilot blockers, `198` undecided, and `149` deferred.
- [ ] Update `docs/takeover/STATUS.md` with the SB-INS result, exact as-built/test/disposition totals, focused evidence, manual boundary, next serial increment, and current commit language without implying Gate 1 completion.
- [ ] Keep the 52 SB-GEO rows visibly deferred to the next product version rather than mislabeling them as unreviewed work.

## Task 4: Verify, freeze, and commit the execution increment

**Files:**
- Verify: plan, receipt, ledger, dashboard, source-owned bytes, immutable source, and repository state
- Modify: none after the final evidence freeze except a truthful dashboard commit reference if required

- [ ] Run `git diff --check`.
- [ ] Run the takeover-ledger validator and independent scripts for contiguous IDs, exact counts, allowed enum values, 26 receipt sections, T01-T30 routing, 18 parameters, six opens, four escalations, six refusals, 95 traceability rows, source-owned hash, and byte identity of all non-SB-INS ledger rows.
- [ ] Run `npx tsc --noEmit` from `D:\XX. SandiBumi`.
- [ ] Run `cargo check` from `D:\XX. SandiBumi\src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from `D:\XX. SandiBumi`; require at least the current `946 passed / 0 failed / 36 ignored` baseline.
- [ ] Confirm `git diff --name-only` contains only the receipt, ledger, and dashboard; confirm no PRD, production, test, generated, research, or retired-worktree file changed.
- [ ] Commit once with message `G1-DOM-INS adjudicate 26 SB-INS requirements`. Do not push, merge, rebase, or open a pull request.

## Completion Criteria

- [ ] Every SB-INS requirement has one evidence-backed live verdict, risk class, release disposition, test class, manual boundary, blocker/decision, and next action.
- [ ] Every chapter test, parameter, open item, escalation, refusal, and traceability row is routed exactly as declared, with no uncited value supplied.
- [ ] A fixture validator is never reported as real MSI, offline-pack, legal, network, servicing, or clean-machine evidence.
- [ ] Product-owner deployment decisions are preserved without claiming their missing artifacts exist.
- [ ] Source-owned ledger bytes and non-SB-INS rows are unchanged.
- [ ] The focused checks, TypeScript compiler, Rust compiler, ledger validator, and full gate are green.
- [ ] The serial execution commit is local and unpushed, and Gate 1 remains open for the truthful SB-GEO deferral receipt plus the final Sol-max all-931 audit.
