# SandiBumi whole-product takeover design

**Date:** 2026-08-10

**Status:** Direction authorized; written design awaiting Jauhar's review

**Product owner:** Jauhar

**Primary engineering owner:** Codex session model

## 1. Outcome

The takeover converts SandiBumi from a collection of independently completed requirement lanes into
one evidence-governed release program. The immediate product target is a paid, offline,
Windows-first pilot. The long-horizon PRD v2 corpus remains intact, but a requirement does not block
the pilot merely because its current chapter labels it `P0`.

Jauhar authorized re-adjudicating the existing priorities and redefining the first paid-release gate
around:

1. silent-wrongness removal or explicit refusal;
2. installation and offline runtime;
3. project recovery and continuity;
4. a named pilot workflow verified on representative real data; and
5. a frozen release candidate accepted through clean installation, recovery, export and pilot use.

This design does not modify a petrophysical method, choose a parameter, close a requirement, or
change production behavior. It defines how those decisions become safe, reviewable work.

## 2. Why the operating model must change

The 2026-08-10 evidence snapshot is internally inconsistent:

- `91_REQUIREMENTS_INDEX.md` contains 931 requirements, 266 labelled `P0`, and 235 listed as open
  `P0`.
- The `P0` cross-table contains 119 `ABSENT`, 37 `PARTIAL`, 59 `PRESENT-DIVERGENT`, three
  `PRESENT-UNVERIFIED`, 17 status-less and only 31 `PRESENT-OK` rows.
- The index roll-up says 113 `PRESENT-OK` and 111 `PRESENT-DIVERGENT`, while its own consolidated
  rows currently parse to 114 and 110 after the later `SB-CORE-002` adjudication.
- `RESUME.md` says 11 of 18 domain chapters are written although all 18 chapter files exist.
- `00_INDEX.md` names `90_GAP_ANALYSIS.md`, but that file is absent in this worktree.
- `REVIEW.md` currently contains 78 checked and 1,392 unchecked boxes, approximately 5.3 percent
  manually confirmed. The PRD's earlier 75/1,125 and 6.7 percent snapshot is no longer current.
- The current branch starts with three commits not in `master`; another branch contains a separate
  native-grid import commit not in `master`; pre-existing dirty and untracked paths also exist.

These facts do not invalidate PRD v2. They prove that the index, source, tests, branches and field
evidence need a single reconciliation layer before more broad implementation lanes begin.

## 3. Operating charter: the critique that must remain active

### 3.1 Founder failure shields

The product owner and primary agent MUST remind each other of these risks at broad planning and
release decisions:

- A legitimate researched gap does not automatically belong in the first paid release.
- A green automated gate is not field validation.
- A larger PRD or checklist is not progress unless release risk or evidentiary uncertainty falls.
- Atomic commits do not make independently developed branches an integrated product.
- An unsupported customer-facing claim is removed or explicitly qualified; it is not kept because
  demonstrating it would be inconvenient.
- Scope is chosen from a named buyer workflow and deployment reality, not by trying to complete the
  incumbent comparison table.
- Product decisions that materially change licensing, support, target scale, lineage granularity or
  deployment remain Jauhar decisions. Engineering MUST surface them and MUST NOT silently decide
  them.

### 3.2 Primary-agent failure shields

The primary agent MUST:

- reverify current source, owned tests, manual evidence and branch state before trusting a PRD
  status;
- distinguish four different claims: as-built behavior, intended contract, automated evidence and
  real/manual verification;
- treat an owned test as proof only of the sentence its name pins;
- keep unresolved evidence unresolved and never choose an uncited parameter, endpoint, cutoff,
  depth reference or unit convention;
- preserve `f32::NAN`, byte-array IPC, whitelisted writes, Python subprocess isolation, undoable
  edits, delivery-set priority and the PK-less `computed_curves` write discipline;
- preserve an existing refusal until the governing requirement explicitly changes it;
- track whether a commit is integrated, not merely present on some branch;
- report a blocker instead of turning a plausible guess into a shipped invariant; and
- measure progress by closed release risk, not by requirements touched.

## 4. Evidence model

No single artifact answers every question. The takeover uses this explicit evidence model:

| Question | Governing evidence |
|---|---|
| What does the product do now? | Current source plus a directly exercising test or runtime observation |
| What should it do? | The governing PRD requirement, its cited source and any binding build-record decision |
| What did an automated check prove? | The exact named assertion and fixture; no broader claim |
| What has a user confirmed on representative data? | Capability-indexed `REVIEW.md` evidence |
| Is the work integrated? | Commit reachability from the accepted baseline plus a green integration gate |
| Is it releasable? | All five gate exits, including deployment and pilot acceptance |

`91_REQUIREMENTS_INDEX.md` is a derived projection. A discrepancy between the index and its sources
is an index defect to record; the discrepancy MUST NOT be resolved by pretending either current code
or stale prose is the desired behavior.

## 5. The takeover ledger

Gate 1 will create one row for every PRD v2 requirement. Each row has these required fields:

| Field | Meaning |
|---|---|
| `requirement_id` | Stable `SB-*` identifier |
| `chapter_contract` | Exact chapter and requirement location |
| `original_priority` | Priority currently written in the chapter |
| `chapter_status` | Status currently written in the chapter, including invalid or blank values verbatim |
| `as_built_status` | Status reverified against current source and behavior |
| `release_disposition` | `PILOT-BLOCKER`, `DEFERRED`, `OUT`, or `UNDECIDED` |
| `risk_class` | Silent wrongness, degraded-result honesty, data integrity, deployment, recovery, field evidence, requested capability, or later |
| `implementation_paths` | Current owned production paths; empty if absent |
| `owned_tests` | Tests that exercise the reporting or user-visible contract, not only an internal `Result` |
| `test_class` | Correctness, characterization, optional-package ignored, divergent-spec ignored, or missing |
| `expected_value_source` | Chapter/source/arithmetic citation; empty when no numeric expectation exists |
| `manual_evidence` | Capability-indexed `REVIEW.md` entries and current state |
| `dependencies` | Requirement and product-decision prerequisites |
| `commit_state` | Integrated commit, candidate branch commit, or unimplemented |
| `blocking_decision` | Exact owner decision or missing cited source; empty when none |
| `next_action` | One bounded, reviewable increment |
| `last_reverified` | Date and accepted baseline commit |

The original priority is never overwritten in the ledger. Release disposition is separate so the
takeover can defer a current `P0` without rewriting history or implying the requirement is
unimportant.

## 6. Gate 1 — baseline reconciliation

### Purpose

Establish one accepted, reproducible statement of what exists, what is proved, what remains on a
branch, and what blocks the pilot.

### Work

1. Record the exact worktree state and preserve every pre-existing modification and untracked path.
2. Compare `master`, the takeover branch and every non-contained local/remote feature commit using
   reachability and patch equivalence. Classify each unique commit as accepted candidate, superseded,
   rejected or unresolved; do not merge during the inventory.
3. Run the current TypeScript, Rust and full repository gates to establish a dated baseline. A red
   gate is recorded before any repair and routed to its owning requirement.
4. Parse all 931 index rows and reverify every chapter status in domain-sized passes against source
   and owned tests. Rust and TypeScript negative searches are confirmed because the available index
   is not authoritative for macros and `cfg`-gated code.
5. Reconcile chapter requirement counts, priorities, statuses, owned tests, parameter citations,
   open items, refusals and Tier-C independent-derivation obligations.
6. Convert `REVIEW.md` from round-only evidence into a capability view without marking any unchecked
   item complete.
7. Inventory customer-facing numerical and capability claims, including the unmeasured 2,000-well
   statement.
8. Assign a provisional release disposition and risk class to every row. Only Jauhar approves the
   resulting pilot scope.

### Exit criteria

- All 931 requirements are accounted for exactly once.
- Every row distinguishes chapter status from reverified as-built status.
- Every claimed test, citation, branch commit and manual-verification item resolves to evidence.
- All internal PRD/index discrepancies are listed rather than silently normalized.
- The current full gate result and accepted baseline commit are recorded.
- The pilot-blocker list is small enough to execute as a release program and is approved by Jauhar.
- No production behavior changes as part of reconciliation.

## 7. Gate 2 — silent-wrongness closure

### Purpose

Ensure every known path capable of returning a plausible but wrong result is corrected, disabled or
made to refuse before the pilot can exercise it.

### Work

1. Start from the index's 34 Tier -1 candidates, the cross-cutting truth requirements and any
   additional candidate found during live re-adjudication. Do not invent new violations from
   speculation.
2. Re-derive each contract from its cited evidence. A needed uncited parameter remains absent and
   blocks only the affected path.
3. Prefer dimensional or typed impossibility over warnings where the specification requires it.
4. Require one named test per contract, pinning both sides where a lazier implementation would pass.
5. Verify degraded paths at the reporting surface; an internal `Result` alone is insufficient.
6. Commit one requirement per commit, then run the local checks and integration gate.
7. Add the exact field/manual confirmation to the capability view without checking it on the
   user's behalf.

### Exit criteria

- No known silent-wrongness path remains enabled in the pilot workflow.
- Any unresolved path refuses with an actionable message or is excluded from the pilot capability
  manifest.
- Every numeric expectation names its source or independent arithmetic.
- The full automated gate is green, and remaining manual evidence is explicitly open.

## 8. Gate 3 — Windows, offline deployment and recovery

### Purpose

Make SandiBumi installable, operable and recoverable by client IT without development tools or
network access.

### Fixed direction

- Windows first; Linux remains a later product decision.
- Device-wide managed installation.
- A SandiBumi-qualified offline Python/runtime pack for Python-backed capabilities.
- Native capabilities remain available when Python or an optional package is unavailable.
- Qualification covers every Microsoft-serviced Windows 11 x64 Pro and Enterprise feature release
  that passes SandiBumi's clean-machine matrix at release time.

### Work

1. Reverify the installer, package type, installation scope, signing, settings templates, immutable
   resources and writable data locations.
2. Define the offline runtime manifest, package hashes, interpreter selection, capability preflight,
   remediation and support-report evidence.
3. Exercise install, upgrade, rollback and uninstall on the supported clean-machine matrix.
4. Prove that native work remains usable with no Python and that Python-backed capabilities fail
   before work starts when their runtime is unavailable.
5. Prove fresh-clone build/test, project backup, project recovery and release rollback.
6. Keep licensing, activation, update delivery, support window and response commitment as explicit
   Jauhar decisions; engineering supplies consequences and testable contracts.

### Exit criteria

- A qualified offline package installs device-wide on every declared Windows scenario.
- No development tool or network connection is needed for the declared capability manifest.
- Missing optional runtimes degrade only their declared capabilities.
- Upgrade, rollback, uninstall, backup and project recovery have repeatable evidence.
- Client IT can obtain a support report that excludes project data and secrets.

## 9. Gate 4 — real-data pilot verification

### Purpose

Prove the pilot workflow on representative real data instead of extrapolating from unit, browser or
desktop-harness tests.

### Work

1. Jauhar defines the pilot workflow and supplies a representative, locally controlled corpus.
   Client, field, block, operator, well and project names remain out of committed code, fixtures,
   comments and public artifacts.
2. Map the workflow into capability checkpoints: import, unit/depth integrity, conditioning,
   interpretation, QC, aggregation, plots, exports and recovery as applicable.
3. Pre-register expected outcomes and their evidence before running the workflow. A value without a
   source is not accepted merely because the application produces it.
4. Run the same workflow from raw delivery through deliverable, preserve provenance and record every
   deviation or manual override.
5. Mark `REVIEW.md` only from Jauhar's actual confirmation. Automated evidence remains separately
   labelled.
6. Convert every discovered defect into a bounded requirement-owned increment and repeat the
   affected workflow segment after repair.

### Exit criteria

- Every pilot capability has either accepted real-data evidence or an explicit exclusion.
- The complete workflow is reproducible from retained inputs, parameters, sources and application
  version.
- No clean success surface hides partial or degraded work.
- The capability-indexed verification ratio for the pilot scope is 100 percent; the whole-product
  ratio remains reported separately and honestly.

## 10. Gate 5 — release freeze and pilot acceptance

### Purpose

Turn the verified workflow into a supportable release candidate without adding new capability.

### Work

1. Freeze the accepted commit, dependency/runtime manifests, capability manifest and supported
   Windows matrix.
2. Run TypeScript, Rust and full repository gates from the frozen tree.
3. Repeat clean installation, offline launch, project open, representative workflow, exports,
   backup/recovery and rollback from the release package.
4. Complete the legal, licence, activation, update, support-window and continuity decisions needed
   for the paid pilot.
5. Remove or qualify any customer-facing claim not demonstrated by the release evidence. The
   2,000-well number stays absent until a defined benchmark proves it.
6. Produce release notes, known limitations, capability availability, installation guidance,
   recovery instructions and support boundaries.
7. Obtain Jauhar's acceptance and the pilot user's acceptance against the pre-registered workflow.

### Exit criteria

- The release candidate is byte-identifiable and reproducible from a fresh clone.
- All automated, clean-machine, recovery and pilot-workflow evidence refers to that exact candidate.
- No unresolved pilot blocker, legal blocker or support-boundary decision is hidden in prose.
- The package is accepted for the paid pilot; later requirements remain visible and explicitly
  deferred rather than being implied complete.

## 11. Execution and change control

- Gate work uses a dedicated branch or worktree based on the last accepted integration commit.
- Requirement implementation remains one requirement per commit. Shared infrastructure is assigned
  to the first requirement that needs it and is referenced by dependent rows.
- Documentation-only adjudication and production implementation are separate commits.
- `REVIEW.md` receives one top entry per shipped increment, without marking manual work complete.
- No broad `git add`; stage explicit owned paths. Machine-local corpora and generated target
  directories never enter a commit.
- Before each requirement commit: `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- Before a gate exit: `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from the repository
  root plus the gate-specific evidence named above.
- A failure outside the active requirement's owned files is recorded and routed; it is not repaired
  opportunistically.
- No requirement is declared complete solely because its token budget, branch or session ended.

## 12. Error and blocker handling

| Condition | Required action |
|---|---|
| Needed petrophysical value has no admissible source | Ship it absent or stop the affected requirement; never guess |
| Source and PRD disagree | Record `PRESENT-DIVERGENT`; do not reinterpret the specification silently |
| Existing test asserts a different contract | Stop and adjudicate; do not weaken the assertion |
| Test passes only by reproducing implementation arithmetic | Label characterization or replace with independent evidence |
| Optional package is unavailable | Ignore only the package-dependent test and keep native gates independent |
| Branch contains unique work | Inventory and review before integration; do not infer acceptance from existence |
| Full gate is red outside owned scope | Stop the affected lane and route the failure |
| Pilot evidence is unavailable | Keep Gate 4 open; automation cannot substitute for it |
| Product-owner decision is missing | Present consequences and ask Jauhar; engineering does not choose silently |

## 13. Decisions fixed and decisions still open

### Fixed by Jauhar

- Codex takes primary engineering ownership of the SandiBumi development program.
- Existing priorities may be re-adjudicated for the paid-pilot release disposition.
- The five-gate direction is approved.
- Windows is first; Linux is held for later.
- Installation is device-wide for managed IT deployment.
- Python-backed capabilities use a SandiBumi-qualified offline runtime pack.
- Supported Windows qualification is the serviced Windows 11 x64 Pro/Enterprise matrix that passes
  the release-time clean-machine gate.
- Harsh, evidence-based critique applies to both founder and agent decisions.

### Still open and not safe to infer

- The exact pilot workflow and representative corpus.
- Whether the 2,000-well claim is removed immediately or retained only in non-customer-facing
  planning text until benchmarked; this design recommends removal from customer-facing text.
- Licence unit, activation, commercial model, update delivery, support commitment and supported
  version window.
- Portfolio benchmark operations and performance thresholds.
- Lineage granularity beyond the pilot's audit need.
- Legal conclusions for overlays, vendor names, licence inventory and other recorded legal risks.
- Linux timing and support contract after the Windows pilot.

## 14. Design review boundary

Approval of this document authorizes writing the detailed Gate 1 implementation plan and executing
that plan. It does not pre-approve a petrophysical parameter, a production-code change, a branch
merge, a push, or a paid-release claim. Each remains subject to its own evidence and gate.
