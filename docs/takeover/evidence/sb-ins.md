# SB-INS live adjudication evidence

Date: 2026-08-12
Branch: `codex/g1-sb-ins-adjudication`
Plan commit: `57bf948fd4f10179dcc1e48e60f1491904702375`
Plan-column correction commit: `d1df1b59dbd6f535b971858f36e5a07bf91cecc3`
Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)
Origin/merge-base anchor: `29833735816d9e5be954afafd9ceb71fd856e3f0`

## Scope and immutable custody

This receipt adjudicates all 26 contiguous requirements, SB-INS-001 through SB-INS-026, against the accepted live tree. It changes documentation evidence only. Production code, tests, PRD text, parameter values, manual evidence, generated artifacts, and the Windows-first product direction are unchanged.

- Priorities: 11 P0 and 15 blank-priority rows.
- Historical chapter states: 15 ABSENT, 8 PARTIAL, 2 PRESENT-OK, and 1 PRESENT-DIVERGENT. These were evidence inputs, not copied verdicts.
- Frozen chapter SHA-256: `44947f129d6b2c2867d10fac707d144a167a0fde71c38839e5b4937538b9e462`.
- Frozen six source-owned ledger columns SHA-256: `de7fd60a70bde2187130069d4cf16ac522cf6b6b891e1507428e37c1a076f1ce`.
- The chapter defines T01 through T30. Immutable ledger `owned_tests` cells are blank, so this receipt routes the chapter intentions without changing source-owned ownership fields.
- Parameters: 18 rows, including 5 deliberate ABSENT values.
- Governance: 6 open items, 4 escalations, 6 refusals, no Tier-C item, and 95 section-8 traceability rows retained.

## Product-owner direction and evidence boundary

The live direction is Windows-first: a per-machine MSI installed by IT/system context and launched by a standard user; a separately signed, versioned, application-local SandiBumi-qualified Python pack deployed per machine; and every Microsoft-serviced Windows 11 x64 Pro and Enterprise feature release at release time. Linux is held for a later product version.

Those decisions close product-choice ambiguity only. They do not create a signed MSI, pack bytes, exact release lock, digests, vulnerability disposition, legal approval, servicing snapshot, silent-install transcript, blocked-network trace, or clean-machine results. No Linux support or qualification claim is made here.

## Baseline and executable evidence

- Sole worktree: `D:\XX. SandiBumi`; execution branch created serially from committed plan `57bf948`.
- Before this domain: 853 adjudicated, 78 unadjudicated, 562 pilot blockers, 220 undecided, and 149 deferred.
- Current Tauri configuration selects MSI and offline WebView2 installation. `installation.rs` supplies capability, installer, offline-deployment, and clean-machine schemas/validators; `src-tauri/examples/installation_gate.rs` is an example CLI, not a release-pipeline hook.
- No actual signed MSI, qualified Python pack, release-lock artifact, network trace, release-time Microsoft servicing inventory, real clean-machine matrix, automated publication gate, corporate policy, reversible settings migration, upgrade/uninstall preservation run, canonical-registry generator, pack attestation, or human legal approval was found in the accepted tree or reachable history.
- Focused evidence passed: installation 10/0/0, parameter pack 3/0/0, Python environment 3/0/0, typed unit registry 7/0/0, and encoding 4/0/1. Total across those filters: 27 passed, 0 failed, 1 ignored.
- Fixture-populated validators prove acceptance/refusal logic only. They do not prove any release artifact, deployment, network condition, serviced-Windows target, or lifecycle scenario actually existed.
- Manual evidence: 0/16 SB-INS-specific scenarios checked; installation/deployment 0/0 and not listed; DLIS 0/11; data conventions 0/45; equation engine 0/11; office deliverables 0/39; workspace shell 0/159; project lifecycle 3/24; security integrity 0/63; verification stewardship 0/24; LAS export 0/2. Automated evidence closes none of these scenarios.

## Live result

- As built: 6 ABSENT, 11 PARTIAL, 5 PRESENT-OK, 3 PRESENT-DIVERGENT, 1 PRESENT-UNVERIFIED.
- Release: 22 PILOT-BLOCKER, 4 UNDECIDED, 0 DEFERRED.
- Test class: 5 CORRECTNESS, 3 CHARACTERIZATION, 18 MISSING qualifying whole-contract proofs.
- Risk: 13 DEPLOYMENT, 7 DATA-INTEGRITY, 2 RECOVERY, 4 SILENT-WRONGNESS.
- Mechanically after this receipt: 879 adjudicated, 52 unadjudicated, 584 pilot blockers, 198 undecided, 149 deferred.

## Harsh-truth findings

1. SandiBumi has installer-validation code, not a qualified installer. The tests create successful evidence objects in memory; they do not install the final MSI on a clean machine.
2. The offline Python route is a product decision and a prose/schema contract. The signed pack, exact package lock, digests, zero-network capture, vulnerability review, and real probe results do not exist in the repository.
3. The capability manifest covers six named features but not the complete Python-backed product surface. A test that expects those same six rows proves internal consistency, not completeness.
4. “Re-probe” is text, not a user action. The support dialog exposes only Close, and the real capability messages do not all carry the helper's copyable command.
5. Parameter-pack identity loading is now product-reachable through a backend-owned module schema; applying pack values to computation deliberately remains closed until the mismatch, typed-unit, observed-token, generated-registry and attestation/provenance contracts are complete.
6. Raw encoding/unit evidence is not retained end to end. Global case folding currently makes `mV` and `mv` equivalent without an explicit alias, contrary to the specified observable-drift contract.
7. The unit-registry validator is unused outside tests. A correct validator that is never enforced cannot block a bad release registry.
8. The third-party generator reports unknown licences but exits successfully, says Python packages are not distributed, and records neither the chosen offline pack nor human legal approval. Its historical PRESENT-OK label is no longer defensible.
9. Public prerequisite fragments generated from `installation.rs` agree with one another, but other public documents still describe Python distribution/prerequisite decisions as open or user-supplied. One synchronized subset is not one source of truth.
10. All 16 installer-specific manual scenarios remain unchecked, and installation/deployment is absent from the capability matrix. The software has no field evidence for the deployment claim.

## Unique acceptance-test routing

Each T01-T30 intention is routed once. Shared chapter ownership is shown in one row and does not create a second evidence route.

| Test intention | Receipt owner(s) | Live class |
|---|---|---|
| SB-INS-T01 | SB-INS-001 | MISSING |
| SB-INS-T02 | SB-INS-002 | CORRECTNESS |
| SB-INS-T03 | SB-INS-003, SB-INS-026 | MISSING |
| SB-INS-T04 | SB-INS-004 | MISSING |
| SB-INS-T05 | SB-INS-005 | CORRECTNESS |
| SB-INS-T06 | SB-INS-005 | CORRECTNESS |
| SB-INS-T07 | SB-INS-006, SB-INS-007 | MISSING |
| SB-INS-T08 | SB-INS-006, SB-INS-007 | MISSING |
| SB-INS-T09 | SB-INS-006, SB-INS-007 | MISSING |
| SB-INS-T10 | SB-INS-008 | MISSING |
| SB-INS-T11 | SB-INS-009, SB-INS-024 | MISSING |
| SB-INS-T12 | SB-INS-010 | CORRECTNESS |
| SB-INS-T13 | SB-INS-011 | MISSING |
| SB-INS-T14 | SB-INS-012, SB-INS-013 | MISSING |
| SB-INS-T15 | SB-INS-013 | MISSING |
| SB-INS-T16 | SB-INS-014 | CORRECTNESS |
| SB-INS-T17 | SB-INS-015 | MISSING |
| SB-INS-T18 | SB-INS-015 | MISSING |
| SB-INS-T19 | SB-INS-016 | CORRECTNESS |
| SB-INS-T20 | SB-INS-016 | CORRECTNESS |
| SB-INS-T21 | SB-INS-017 | CHARACTERIZATION |
| SB-INS-T22 | SB-INS-017 | CHARACTERIZATION |
| SB-INS-T23 | SB-INS-018 | CHARACTERIZATION |
| SB-INS-T24 | SB-INS-019 | MISSING |
| SB-INS-T25 | SB-INS-020 | MISSING |
| SB-INS-T26 | SB-INS-021 | CHARACTERIZATION |
| SB-INS-T27 | SB-INS-022 | MISSING |
| SB-INS-T28 | SB-INS-023 | MISSING |
| SB-INS-T29 | SB-INS-024 | MISSING |
| SB-INS-T30 | SB-INS-025 | MISSING |

The live requirement-level test total counts one class per requirement, not one per table row above: 5 correctness requirements, 3 characterization requirements, and 18 requirements missing a qualifying whole-contract proof. T05/T06 and T19/T20 remain two test intentions under one correctness-classified requirement.

## Parameter, open-item, and source custody

All 18 source parameter rows remain intact. The 13 cited live values are product version `0.1.0`, identifier `com.sandibumi.petro`, bundle active `true`, historical bundle target `all`, embedded database mode `bundled`, minimum Python `3.10+`, override `SANDIBUMI_PYTHON`, discovery versions 3.13/3.12/3.11/3.10, and the named equation, DLIS, plate, and Office package populations. Current MSI selection is an as-built change; it does not rewrite the immutable chapter row.

The five deliberate source absences remain absent: corporate/user/template precedence, offline runtime distribution mode, supported installer package type, configuration-pack text encoding, and unit-token case policy. The later product-owner MSI/offline-pack decisions are recorded separately from the immutable source table and from still-missing release evidence. No Python/package version, servicing release, signing identity, digest, security disposition, alias, encoding, precedence, or migration value is invented.

- O-INS-1 is direction-resolved to per-machine MSI/system deployment with standard-user launch; actual signed-artifact qualification remains open.
- O-INS-2 is direction-resolved to a separately signed application-local Python pack; pack, lock, digests, and offline evidence remain open.
- O-INS-3 remains open; no corporate/template/user precedence is selected.
- O-INS-4 is policy-resolved to every Microsoft-serviced Windows 11 x64 Pro/Enterprise feature release at release time; the release-time inventory and real matrix remain open.
- O-INS-5 remains open and format-specific; CP1252 evidence does not authorize a universal encoding or case rule.
- O-INS-6 remains open; generated inventory is not human legal approval.
- E-INS-1 is partly resolved by O-INS-1/O-INS-2 direction, while O-INS-3 remains with deployment/security ownership.
- E-INS-2 remains open for signing, policy-pack trust, configuration attestation, and support-report redaction.
- E-INS-3 is active because the chosen deployment route distributes a Python pack.
- E-INS-4 remains routed to the scientific domains; installation work must not acquire or infer their parameters.

All six refusals remain binding: cross-dimension unit bridges, blank mapping success, name-only positional joins, mutable-user-file defaults, binary-derived methods/defaults, and false dependency-free claims are refused.

Section 8 custody remains complete: 57 rows in §8.1, 15 rows in §8.2, 15 rows in §8.3, and 8 rows in §8.4, for 95 traceability rows. Scientific values stay with their owning domains; opaque artifacts remain behind the evidence firewall; corrected dossier counts/withdrawals remain corrected; and every SB-INS requirement/test family remains routed without converting traceability into implementation evidence.

## Requirement receipts

### SB-INS-001

- Specified contract: Ship a qualified native Windows installer. Chapter test: SB-INS-T01.
- Current implementation: `tauri.conf.json` selects MSI and `installation.rs` validates identity, per-machine scope, signature, digest, build commit, install, and launch evidence supplied to it. No final MSI or clean-machine execution receipt exists.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — `a_signed_clean_machine_msi_must_match_the_installed_identity_and_version` uses an invented qualification fixture and proves validator behavior, not T01's real artifact/install/launch contract.
- Manual evidence: SB-INS 0/16; installation/deployment 0/0 and not listed; supporting capabilities retain the counts in the baseline section.
- Source/parameter boundary: Per-machine MSI/system deployment is owner-resolved; signature identity, artifact digest, build provenance, and successful clean-machine install/launch must come from the final release.
- Deployment/UI/provenance surface: Configuration and validator are present; installer UI, installed-product evidence, manifest receipt, and publication enforcement are absent.
- History/reachability: Accepted anchor is reachable; current/reachable-history search found schemas and tests but no qualifying MSI artifact or result set.
- Decision/dependency: A checked-in validator cannot certify bytes it never saw.
- Next action: Build and sign the exact MSI, run T01 on a clean supported target, retain digest/build/identity/install/launch evidence, and wire that evidence into the publication gate.

### SB-INS-002

- Specified contract: Keep native core launch independent of Python. Chapter test: SB-INS-T02.
- Current implementation: Application support resolves without Python; real native project open, module dispatch, histogram construction, and LAS-format discovery execute independently while manifest Python capabilities report unavailable.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `CORRECTNESS` — `missing_python_does_not_block_project_open_native_computation_plotting_or_native_export` exercises all four named native paths and the Python-only unavailable side from the chapter contract.
- Manual evidence: SB-INS 0/16; the no-Python manual scenario is unchecked; installation/deployment remains 0/0 and not listed.
- Source/parameter boundary: No interpreter or package default is invented; absence disables only declared Python capabilities.
- Deployment/UI/provenance surface: Native seams and support payload are covered; a real installed no-Python Windows run remains unverified.
- History/reachability: Accepted anchor is reachable and the current implementation contains the qualifying automated path.
- Decision/dependency: Automated correctness does not replace the clean-machine field run.
- Next action: Preserve the guard and close the no-Python installed-machine scenario during Windows qualification.

### SB-INS-003

- Specified contract: Publish truthful capability-level prerequisites. Chapter test: SB-INS-T03, shared with SB-INS-026.
- Current implementation: Generated README, installer resource, prerequisite document, Tauri description, and support UI agree for the six-row manifest. Other public documents still describe Python distribution/prerequisite decisions inconsistently.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — the existing test proves only the generated subset and absence of one retired blanket sentence; it does not inventory every public surface or complete Python-backed capability.
- Manual evidence: SB-INS 0/16; the prerequisite-surface scenario is unchecked; installation/deployment remains 0/0 and not listed.
- Source/parameter boundary: Capability claims must derive from executable manifest/probe evidence; prose cannot create package versions or availability.
- Deployment/UI/provenance surface: One synchronized copy path exists, but product-wide claim generation and release refusal are incomplete.
- History/reachability: Accepted anchor is reachable; contradictory public copy remains in the current tree.
- Decision/dependency: Truthfulness is whole-product; a correct subset does not excuse stale adjacent claims.
- Next action: Inventory every public prerequisite surface and actual Python consumer, generate the complete copy set from the manifest/probes, and make divergence fail release.

### SB-INS-004

- Specified contract: Maintain one dependency/capability manifest. Chapter test: SB-INS-T04.
- Current implementation: `capabilities.json` maps six capabilities to one interpreter and cited package names, and several detectors consume it. It omits other Python-backed product consumers, has no package minimums, and points to a release lock that does not exist.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — `each_optional_capability_maps_to_the_cited_packages_and_no_detector_carries_a_second_package_list` expects the same six-row fixture universe and cannot prove the inventory is complete.
- Manual evidence: SB-INS 0/16; the manifest/detection scenario is unchecked; supporting Python capabilities remain manually open.
- Source/parameter boundary: Exact minimum versions remain absent until the qualified release lock cites them.
- Deployment/UI/provenance surface: Manifest consumption exists in installation/DLIS/image/Office paths, but the complete runtime/capability universe and offline/package-version evidence do not.
- History/reachability: Accepted anchor is reachable; source search finds Python consumers outside the six-row completeness assertion.
- Decision/dependency: One file is not one source of truth when it omits live consumers.
- Next action: Derive a complete consumer inventory, add only sourced version rows and offline routes, and add a source-derived completeness test that fails on an unregistered Python runner.

### SB-INS-005

- Specified contract: Resolve one interpreter with explainable precedence. Chapter tests: SB-INS-T05 and T06.
- Current implementation: `python_engine.rs` resolves one cached session interpreter, records every candidate/rule/rejection, and all searched Python consumers call the shared resolver.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `CORRECTNESS` — the two named environment tests independently pin lower-valid-candidate selection and exact override reuse by every manifest probe.
- Manual evidence: SB-INS 0/16; the interpreter-resolution scenario is unchecked; installation/deployment remains 0/0 and not listed.
- Source/parameter boundary: Discovery order, `SANDIBUMI_PYTHON`, and Python 3.10 minimum are source-cited; no pack path or version is inferred.
- Deployment/UI/provenance surface: Resolution provenance is in the support payload and UI; real pack/external-interpreter installed-machine behavior remains unverified.
- History/reachability: Accepted anchor is reachable and the qualifying resolver/tests are in the live tree.
- Decision/dependency: Keep one session interpreter; do not allow capability-local fallback.
- Next action: Preserve the resolver contract and exercise both qualified-pack and supported-external-Python paths in the clean-machine matrix.

### SB-INS-006

- Specified contract: Probe packages and versions before work begins. Chapter tests: SB-INS-T07 through T09.
- Current implementation: Equation, DLIS, workbook, and deck paths have supporting probes; DLIS refuses before parsing. Product-wide action gating is incomplete, including the document-deliverable surface.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — the DLIS order test and helper tests prove narrower seams, not every capability/action and not the UI behavior of T07-T09.
- Manual evidence: SB-INS 0/16; the missing-package preflight scenario is unchecked; DLIS 0/11, equation 0/11, and office deliverables 0/39.
- Source/parameter boundary: Probe only packages and version constraints declared in the complete manifest; uncited minimums remain absent.
- Deployment/UI/provenance surface: Backend probes exist, but all affected UI actions are not consistently blocked before costly/save work.
- History/reachability: Accepted anchor is reachable; current UI/source search confirms incomplete preflight coverage.
- Decision/dependency: A late subprocess error is not preflight.
- Next action: Route every Python-backed UI action through manifest-derived preflight and add observable T07-T09 coverage for each affected action.

### SB-INS-007

- Specified contract: Give interpreter-specific remediation. Chapter tests: SB-INS-T07 through T09.
- Current implementation: `package_remediation` builds exact executable/package/pip text and says to re-probe, but real capability messages do not all use it and the support dialog exposes no re-probe control.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — the helper unit test proves generated text for package rows; it does not prove every visible refusal or a working re-probe action.
- Manual evidence: SB-INS 0/16; the remediation scenario is unchecked; related DLIS/equation/office counts remain open.
- Source/parameter boundary: The command must target the selected executable and named distribution; it must not choose a new interpreter or version.
- Deployment/UI/provenance surface: Copyable command helper exists; UI wiring and re-probe interaction are incomplete.
- History/reachability: Accepted anchor is reachable; current dialog source contains Close but no re-probe event/control.
- Decision/dependency: Prose saying “re-probe” is not a re-probe action.
- Next action: Use the helper on every visible missing-package surface, add one real re-probe control, and test both message and state refresh.

### SB-INS-008

- Specified contract: Support offline and managed deployment. Chapter test: SB-INS-T10.
- Current implementation: Tauri uses offline WebView2 installation and `installation.rs` validates an offline qualification object for the chosen per-machine pack route. The pack, lock, installs, and captured network evidence are absent.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — `an_offline_clean_machine_makes_no_network_request_and_every_claimed_capability_passes_its_probe` populates zero requests and successful installs in a fixture; it does not observe a machine or network.
- Manual evidence: SB-INS 0/16; the end-to-end offline deployment scenario is unchecked; installation/deployment remains 0/0 and not listed.
- Source/parameter boundary: The signed application-local pack route is owner-resolved; exact runtime/packages/digests must come from the release lock and artifact bytes.
- Deployment/UI/provenance surface: Validation schema exists; packaging, deployment automation, detection result, trace retention, and release enforcement do not.
- History/reachability: Accepted anchor is reachable; no pack/lock/network artifact exists in current or reachable history.
- Decision/dependency: A schema describing offline evidence is not offline evidence.
- Next action: Build the signed pack and lock, deploy MSI plus pack with the public network blocked, retain the trace and probes, and make T10 a release gate.

### SB-INS-009

- Specified contract: Pin and attest optional runtime contents. Chapter test: SB-INS-T11, shared with SB-INS-024.
- Current implementation: No distributed Python pack inventory, exact lock, artifact/package digests, vulnerability review, external-runtime label record, or legal disposition exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — no executable release-artifact test supplies the T11 inputs or expected inventory equality.
- Manual evidence: SB-INS 0/16; installation/deployment 0/0 and not listed.
- Source/parameter boundary: No package/runtime version or security status may be selected without the qualified release lock and review evidence.
- Deployment/UI/provenance surface: Manifest placeholders name a future lock; actual release manifest, support labeling, and licence inventory are absent.
- History/reachability: Accepted anchor is reachable; no qualifying runtime artifact was found.
- Decision/dependency: The offline-pack decision activates this obligation; it does not satisfy it.
- Next action: Create a signed reproducible pack and lock with exact versions/digests/licences/vulnerability status, distinguish external runtimes, and implement T11.

### SB-INS-010

- Specified contract: Separate immutable templates from user configuration. Chapter test: SB-INS-T12.
- Current implementation: A bundled empty settings template is materialized once into user configuration with template version/digest provenance; later user edits survive and the installed bytes remain unchanged. The command is wired in `lib.rs`.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — the named test pins first-run creation, later-edit preservation, template byte identity, version/digest origin, and absence of invented defaults.
- Manual evidence: SB-INS 0/16; the installed-settings scenario is unchecked; project lifecycle remains 3/24.
- Source/parameter boundary: The settings map stays empty because no factory values are cited.
- Deployment/UI/provenance surface: Runtime materialization/provenance exists; installed-profile behavior still needs a clean-user run.
- History/reachability: Accepted anchor is reachable and the production command/test are live.
- Decision/dependency: Preserve the installed/user separation during future migration work.
- Next action: Retain the guard and verify T12 on the final installed candidate.

### SB-INS-011

- Specified contract: Migrate configuration explicitly and reversibly. Chapter test: SB-INS-T13.
- Current implementation: No prior-version inventory, presented migration set, backup, versioned transform, or accepted/renamed/defaulted/rejected report exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `RECOVERY`.
- Automated evidence: `MISSING` — no executable migration contract or regression test exists.
- Manual evidence: SB-INS 0/16; project lifecycle 3/24; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Migration rules and precedence remain absent; copying an entire profile is refused.
- Deployment/UI/provenance surface: Installed/user split exists, but no upgrade migration UI, backup, transform, or report exists.
- History/reachability: Accepted anchor is reachable; no qualifying configuration migration path was found.
- Decision/dependency: Clean-machine qualification includes upgrade/rollback, so this is a pilot dependency even if first install works.
- Next action: Adjudicate source-backed migration rules, implement reversible versioned transforms and reporting, then add T13.

### SB-INS-012

- Specified contract: Provide a corporate policy layer. Chapter test: SB-INS-T14, shared with SB-INS-013.
- Current implementation: No signed read-only policy format, trust root, administrator deployment path, enforced runtime/pack/location rule, or capability disable exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — no executable policy-layer test exists.
- Manual evidence: SB-INS 0/16; workspace shell 0/159; security integrity 0/63; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Policy authority, signing, and precedence are not source-adjudicated and remain absent.
- Deployment/UI/provenance surface: No load, enforcement, winning-layer display, override refusal, or provenance surface exists.
- History/reachability: Accepted anchor is reachable; no policy-layer implementation was found.
- Decision/dependency: IT deployment does not by itself choose whether signed corporate policy is mandatory for the first pilot.
- Next action: Jauhar plus deployment/security ownership must decide pilot necessity and trust/precedence before implementation.

### SB-INS-013

- Specified contract: Make precedence visible and deterministic. Chapter tests: SB-INS-T14 and T15.
- Current implementation: No resolver spans shipped template, corporate policy, migrated user settings, and current user settings; no shadowed-value report exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — no four-layer resolution or UI test exists.
- Manual evidence: SB-INS 0/16; workspace shell 0/159; security integrity 0/63.
- Source/parameter boundary: The corporate/user/template precedence parameter deliberately remains absent.
- Deployment/UI/provenance surface: No effective-value provenance or shadowed-layer UI exists.
- History/reachability: Accepted anchor is reachable; no qualifying layer resolver was found.
- Decision/dependency: O-INS-3/E-INS-2 must be settled; choosing an order here would invent product policy.
- Next action: Obtain the signed precedence decision, then implement the resolver/report and T14-T15 from both winner and shadowed-value sides.

### SB-INS-014

- Specified contract: Key parameters by semantic identifier and ordinal. Chapter test: SB-INS-T16.
- Current implementation: `parameter_pack.rs` derives a backend-owned schema from the shipping module manifest, reuses stable module/argument wire IDs, assigns configurable-row ordinals, deterministically versions that manifest, and exposes schema/load commands through registered Tauri and typed TypeScript IPC. Duplicate labels never participate in selection.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CORRECTNESS` — `two_identically_labelled_loaded_parameter_rows_remain_separately_addressable_by_semantic_identifier_and_ordinal` loads the T16 fixture through the product function, proves both exact keys and the crossed-key refusal, and pins both IPC registrations.
- Manual evidence: SB-INS 0/16; the parameter-pack identity scenario is unchecked; security integrity 0/63.
- Source/parameter boundary: Display labels never become selectors; semantic identity, ordinal and schema version all come from the shipping module manifest. Fixture values are opaque markers and no pack value becomes a default.
- Deployment/UI/provenance surface: The governed load boundary is product-reachable; no selection UI or automatic computation application is claimed.
- History/reachability: The prior orphan is closed by registered backend commands and typed frontend wrappers, both pinned by T16.
- Decision/dependency: Applying loaded values remains gated by SB-INS-015 through SB-INS-020; this increment does not bypass those safety contracts.
- Next action: Retain T16 and continue with observable ambiguity refusals through the same governed load path under SB-INS-015.

### SB-INS-015

- Specified contract: Refuse registry mismatch and ambiguity. Chapter tests: SB-INS-T17 and T18.
- Current implementation: The isolated loader refuses ID/ordinal disagreement, missing ordinal, duplicate key, unsupported schema, and empty key, naming file/row conflicts. No live pack activation calls it.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` at the whole-product contract — two direct tests cover the loader's refusal matrix but not a customer-reachable activation-before-computation boundary.
- Manual evidence: SB-INS 0/16; the ambiguous-pack refusal scenario is unchecked; security integrity 0/63.
- Source/parameter boundary: No positional/name guess and no partial activation are allowed.
- Deployment/UI/provenance surface: Internal refusal exists; product selection, visible error, run prevention, and audit/provenance are absent.
- History/reachability: Accepted anchor is reachable; no production caller closes the contract.
- Decision/dependency: The refusal must sit on the actual activation path, not beside it.
- Next action: Route all pack activation through the loader, surface the exact refusal before computation, and implement observable T17-T18.

### SB-INS-016

- Specified contract: Use a canonical typed unit registry. Chapter tests: SB-INS-T19 and T20.
- Current implementation: `curves.rs` resolves quantity kinds/canonical units, refuses permeability-to-length, and performs cited length/slowness conversions while preserving missing values.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CORRECTNESS` — the named tests pin the wrong-kind refusal before arithmetic and the independently derived same-kind factors from both sides.
- Manual evidence: SB-INS 0/16; the typed-unit scenario is unchecked; data conventions 0/45.
- Source/parameter boundary: Only registered, source-derived same-kind bridges compute; no physical-family value is invented.
- Deployment/UI/provenance surface: Registry and conversions are live in ingest paths; release-wide registry validation remains unenforced because `validate_unit_registry` is otherwise unused.
- History/reachability: Accepted anchor is reachable and current tests prove the numerical/refusal core.
- Decision/dependency: Preserve the typed guard, then enforce validation at startup/build/release rather than tests only.
- Next action: Keep T19-T20 and connect registry validation to a product/release gate without weakening unknown-unit preservation.

### SB-INS-017

- Specified contract: Preserve observed unit and encoding tokens. Chapter tests: SB-INS-T21 and T22.
- Current implementation: Parser results expose raw unit spelling and selected decoded encoding, but persistence/export provenance is incomplete; global normalization case-folds `mV` and `mv` without an explicit alias.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — `characterizes_raw_unit_and_encoding_preservation_before_the_current_case_fold` explicitly records the current partial/raw behavior and divergence; it is not correctness proof.
- Manual evidence: SB-INS 0/16; the raw-token/encoding scenario is unchecked; data conventions 0/45 and LAS export 0/2.
- Source/parameter boundary: No universal encoding or case policy is selected; raw evidence must precede format-specific aliases.
- Deployment/UI/provenance surface: Parse-time observation exists; stored provenance, drift warning, explicit alias record, and export round trip are absent.
- History/reachability: Accepted anchor is reachable; current source confirms global case folding.
- Decision/dependency: Silent case equivalence erases evidence even when numerical samples survive.
- Next action: Preserve raw bytes/token/encoding through storage and export, require explicit format aliases, emit drift warnings, and add correctness T21-T22.

### SB-INS-018

- Specified contract: Reject missing and empty unit mappings. Chapter test: SB-INS-T23.
- Current implementation: Characterized absent/empty/placeholder spellings resolve to no registry mapping. One typed missing-unit state across all intake/mapping forms does not exist.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — `characterizes_all_missing_unit_spellings_as_no_registry_mapping` pins current no-mapping behavior and explicitly does not claim the richer specified state.
- Manual evidence: SB-INS 0/16; the missing-unit scenario is unchecked; data conventions 0/45.
- Source/parameter boundary: Empty-to-empty mappings and placeholder tokens must never become successful conversions.
- Deployment/UI/provenance surface: Registry lookup refuses implicitly; explicit state, ingest convergence, warning/provenance, and export representation remain incomplete.
- History/reachability: Accepted anchor is reachable; no unified missing-unit type was found.
- Decision/dependency: “No mapping” is safer than a false mapping but is not the complete observable contract.
- Next action: Add one explicit missing-unit representation through intake/storage/UI/export and implement T23 across all four fixtures.

### SB-INS-019

- Specified contract: Generate aliases, families, and units from one registry. Chapter test: SB-INS-T24.
- Current implementation: Unit families/aliases remain code-resident; no versioned canonical source generates runtime, UI, documentation, and tests. `validate_unit_registry` is test-only and no release drift gate exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — no generated-population equality or release-failure test exists.
- Manual evidence: SB-INS 0/16; data conventions 0/45; verification stewardship 0/24.
- Source/parameter boundary: Registry content and aliases need reviewed source custody; current code tables cannot self-certify completeness.
- Deployment/UI/provenance surface: Runtime lookup exists, but shared version identity, generated UI/docs, and release enforcement do not.
- History/reachability: Accepted anchor is reachable; no generator or checked artifact family was found.
- Decision/dependency: Multiple hand-maintained vocabularies can drift silently while unit tests remain green.
- Next action: Establish one reviewed versioned registry source, generate every consumer artifact, and add T24 as a release gate.

### SB-INS-020

- Specified contract: Version and attest configuration packs. Chapter test: SB-INS-T25.
- Current implementation: Parameter-pack schema/version fragments exist; content digest, source class, creation time, compatibility range, signed identity, and mutation-triggered provenance event do not.
- Verdict: `PARTIAL`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — no signed-pack mutation test or computation-time provenance check exists.
- Manual evidence: SB-INS 0/16; security integrity 0/63; processing history 0/7.
- Source/parameter boundary: Pack versions, digests, trust, and compatibility cannot be inferred from filenames or mutable profiles.
- Deployment/UI/provenance surface: Structural loader exists; attestation, trust UI, activation, run provenance, and mutation handling are absent.
- History/reachability: Accepted anchor is reachable; no live configuration-pack distribution/activation route was found.
- Decision/dependency: Pilot disposition depends on whether configuration packs become pilot-reachable; security review E-INS-2 remains open.
- Next action: Decide pilot reachability and trust model; if in scope, implement full attestation/provenance and T25 before enabling packs.

### SB-INS-021

- Specified contract: Produce a reproducible support report. Chapter test: SB-INS-T26.
- Current implementation: Installation support serializes selected interpreter/rule, candidates, capabilities, package status, and observed versions. Application/build digest, installer type, OS architecture, configuration layers/digests, and redacted diagnostics are absent.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `CHARACTERIZATION` — the named characterization test asserts present fragments and absent required fields; it explicitly is not a correctness claim.
- Manual evidence: SB-INS 0/16; the support-report scenario is unchecked; workspace shell 0/159 and security integrity 0/63.
- Source/parameter boundary: Report observed facts and digests only; never include project samples, secrets, or inferred release identity.
- Deployment/UI/provenance surface: In-app view exists for fragments; one-action export, redaction policy, configuration/install/build identity, and reproducibility are incomplete.
- History/reachability: Accepted anchor is reachable; current serialized type confirms the missing fields.
- Decision/dependency: Useful diagnostics are not a reproducible environment record until all required identities share one report.
- Next action: Define the security-reviewed redaction schema, add the missing observed fields/digests, export one report, and implement T26.

### SB-INS-022

- Specified contract: Preserve user data through upgrade and uninstall. Chapter test: SB-INS-T27.
- Current implementation: No installer lifecycle boundary proves project/settings byte preservation, separate removal consent, enumeration, or recoverable backup through upgrade, rollback, and uninstall.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `RECOVERY`.
- Automated evidence: `MISSING` — no lifecycle harness or byte-identity regression exists.
- Manual evidence: SB-INS 0/16; project lifecycle 3/24; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Default action is preservation; no deletion scope or migration rule is inferred.
- Deployment/UI/provenance surface: No lifecycle consent, backup, recovery receipt, or final-artifact test surface exists.
- History/reachability: Accepted anchor is reachable; no installer lifecycle evidence was found.
- Decision/dependency: An uninstall path is destructive until its exact data boundary is proven.
- Next action: Implement preservation-by-default and separate recoverable removal consent, then run T27 on the signed candidate.

### SB-INS-023

- Specified contract: Gate releases on clean-machine scenarios. Chapter test: SB-INS-T28.
- Current implementation: A validator enforces the cross-product of supplied serviced targets and nine supplied scenarios, refusing failure/omission. No release-time Microsoft inventory, scenario runner, CI/publication hook, or real result set exists.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — `one_failing_clean_machine_scenario_blocks_release_and_names_the_scenario` uses fictional release names and evidence booleans; it proves validator logic only.
- Manual evidence: SB-INS 0/16; the serviced-Windows matrix scenario is unchecked; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Target policy is every Microsoft-serviced Windows 11 x64 Pro/Enterprise release at release time; the actual release list must be captured from Microsoft then, not hard-coded now.
- Deployment/UI/provenance surface: Schema/validator/example CLI exist; automated environment provisioning, scenario execution, artifact binding, preserved logs, and publication refusal do not.
- History/reachability: Accepted anchor is reachable; no real matrix artifact or release workflow was found.
- Decision/dependency: A validator that trusts a fixture's `passed: true` cannot prove a Windows scenario ran.
- Next action: Build the release-time inventory and clean-machine runner, bind all results to the exact MSI/pack/commit, and enforce T28 in publication.

### SB-INS-024

- Specified contract: Generate and review third-party obligations. Chapter tests: SB-INS-T11 and T29.
- Current implementation: The generator inventories normal Rust/frontend dependencies and labels output as factual, not legal advice. It emits unknown licences but exits success, excludes Python as user-supplied, does not enumerate the chosen pack/runtime, and records no vulnerability or human approval state.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — no T29 fixture makes an unknown licence fail generation/release, and no T11 release-pack inventory equality test exists.
- Manual evidence: SB-INS 0/16; security integrity 0/63; verification stewardship 0/24; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Licence facts, vulnerability status, and approval must be recorded from actual distributed bytes and human review; generation is not legal advice.
- Deployment/UI/provenance surface: Factual generator exists; failure semantics, Python-pack scope, approval record, and release enforcement diverge.
- History/reachability: Accepted anchor is reachable; current generator source directly confirms success-on-unknown and user-supplied Python wording.
- Decision/dependency: The offline-pack decision invalidates the generator's present distribution-scope statement.
- Next action: Extend inventory to the exact pack/runtime, fail on undeclared licence, record vulnerability/legal dispositions separately, and implement T11/T29.

### SB-INS-025

- Specified contract: Enforce the evidence-acquisition firewall. Chapter test: SB-INS-T30.
- Current implementation: Targeted source/history search found no parser that decodes proprietary keys, compiled libraries, opaque weights, or vendor chart payloads to recover methods/defaults. No executable inventory/migration guard or owned test proves the negative boundary.
- Verdict: `PRESENT-UNVERIFIED`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — no T30 fixture demonstrates presence-only inventory while refusing content interpretation and method/default output.
- Manual evidence: SB-INS 0/16; security integrity 0/63; verification stewardship 0/24.
- Source/parameter boundary: CONTRACT evidence tiers and all six chapter refusals remain binding; opaque artifacts never authorize scientific values.
- Deployment/UI/provenance surface: Current absence is consistent with the firewall; no explicit tooling policy, inventory record, refusal surface, or regression exists.
- History/reachability: Accepted anchor is reachable; negative source search is evidence but not an executable invariant.
- Decision/dependency: Decide whether an explicit migration/inventory route is pilot-reachable; never add decoding to make the test possible.
- Next action: Add a narrow presence-only inventory/refusal contract if such artifacts enter migration scope, then implement T30 without parsing content.

### SB-INS-026

- Specified contract: Keep release claims derived from executable evidence. Chapter test: SB-INS-T03, shared with SB-INS-003.
- Current implementation: Several generated prerequisite fragments derive from the six-capability manifest, but the manifest is incomplete and no release gate compares every installer/documentation/in-app claim with actual tested runtime paths.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — current tests compare a generated subset and one no-Python payload; they do not fail release for a false claim across all public surfaces and actual capabilities.
- Manual evidence: SB-INS 0/16; prerequisite and manifest scenarios are unchecked; installation/deployment 0/0 and not listed.
- Source/parameter boundary: Claims may report only manifest-declared, probe-tested capability states; no capability or version is inferred from copy.
- Deployment/UI/provenance surface: Generation helpers exist; complete claim registry, live-probe binding, and publication failure are incomplete.
- History/reachability: Accepted anchor is reachable; current source contains both synchronized fragments and stale/contradictory adjacent claims.
- Decision/dependency: Claim generation cannot be stronger than the completeness and truth of its manifest/probe evidence.
- Next action: Complete the manifest, register every public claim surface, compare it with executable probe results, and make any mismatch fail release.
