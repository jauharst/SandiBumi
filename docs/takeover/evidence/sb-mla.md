# SB-MLA live adjudication evidence

Date: 2026-08-12
Branch: `codex/g1-sb-mla-adjudication`
Plan commit: `3bfb5a2cf5179f9e5d494110ade95c10d330d9cc`
Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)
Origin/merge-base anchor: `29833735816d9e5be954afafd9ceb71fd856e3f0`

## Scope and immutable custody

This receipt adjudicates all 65 contiguous requirements, SB-MLA-001 through SB-MLA-065, against the accepted live tree. It changes documentation evidence only. Production code, tests, PRD text, parameter values, manual evidence, and generated artifacts are unchanged.

- Priorities: 10 P0, 34 P1, 17 P2, 4 P3.
- Historical chapter states: 53 PRESENT-OK, 1 PRESENT-DIVERGENT, 2 PARTIAL, 9 ABSENT. These were evidence inputs, not copied verdicts.
- Frozen chapter SHA-256: `6cd4697c611652f66ecdb4a329f9adccf321fecf3aad9aeb8c3a609fad8e219c`.
- Frozen six source-owned ledger columns SHA-256: `82626708eb32931956503fd38daf9d25f179565e718dcba8b6804eaa2244f8dd`.
- The chapter defines T01 through T61. The immutable ledger has 73 ownership references covering all 61 unique IDs, with no ownership gap or unknown test.
- Parameters: 105 rows, including 15 deliberate ABSENT values, 5 NON-ADOPTABLE values, and 3 SB-SHR cross-references.
- Governance: 6 open items, 12 escalations, 13 refusals, 24 no-antecedent requirements, and 218 section-8 traceability rows retained.

## Baseline and executable evidence

- Sole worktree: `D:\XX. SandiBumi`; execution branch created serially from the committed plan.
- Before this domain: 740 adjudicated, 191 unadjudicated, 477 pilot blockers.
- Focused default evidence: ml 67 passed / 0 failed / 7 ignored; native facies 13/0/0; facies tie 7/0/0.
- Separate optional-package run: all 7 ignored ML tests passed. They remain optional-package evidence and do not become default-gate guarantees.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.

## Live result

- As built: ABSENT=8, PARTIAL=14, PRESENT-DIVERGENT=18, PRESENT-OK=24, PRESENT-UNVERIFIED=1.
- Release: DEFERRED=4, PILOT-BLOCKER=44, UNDECIDED=17.
- Test class after the exact-test audit: CHARACTERIZATION=8, CORRECTNESS=14, MISSING=41, OPTIONAL-PACKAGE-IGNORED=2.
- Risk: DATA-INTEGRITY=30, DEGRADED-RESULT=7, DEPLOYMENT=1, FIELD-EVIDENCE=1, LATER=1, REQUESTED-CAPABILITY=7, SILENT-WRONGNESS=18.
- Mechanically after this receipt: 805 adjudicated, 126 unadjudicated, 521 pilot blockers, 262 undecided, 148 deferred.

## Harsh-truth findings

1. The ML subsystem is broad and test-heavy, but internal helpers, source assertions, and optional tests do not prove persisted-output, UI, or default-gate contracts.
2. Unsaved fitted curves can lack a model identifier; runtime warnings and deserialisation failure do not universally occur before apply.
3. Degraded within-well cross-validation still publishes a score, and native numerical guards/convergence are silent.
4. Current cluster-randomness and silhouette outputs are different objects from the specified aggregate/per-cluster quality reports.
5. `_PROB` still names several incompatible quantities; four runner linkages and three UI choices are not the specified five-rule registry.
6. Saved-model apply still holds the global lock across all wells even though the fit path releases between wells.
7. Seven separately passing ignored tests and 7/189 checked ML scenarios are not sufficient release evidence.

## Unique acceptance-test routing

Each T01-T61 intention is routed once below. Shared immutable ownership is noted in requirement sections without creating a second evidence route.

| Test intention | Primary receipt row |
|---|---|
| SB-MLA-T01 | SB-MLA-001 |
| SB-MLA-T02 | SB-MLA-002 |
| SB-MLA-T03 | SB-MLA-003 |
| SB-MLA-T04 | SB-MLA-004 |
| SB-MLA-T05 | SB-MLA-005 |
| SB-MLA-T06 | SB-MLA-006 |
| SB-MLA-T07 | SB-MLA-007 |
| SB-MLA-T08 | SB-MLA-008 |
| SB-MLA-T09 | SB-MLA-009 |
| SB-MLA-T10 | SB-MLA-010 |
| SB-MLA-T11 | SB-MLA-011 |
| SB-MLA-T12 | SB-MLA-012 |
| SB-MLA-T13 | SB-MLA-013 |
| SB-MLA-T14 | SB-MLA-014 |
| SB-MLA-T15 | SB-MLA-015 |
| SB-MLA-T16 | SB-MLA-016 |
| SB-MLA-T17 | SB-MLA-017 |
| SB-MLA-T18 | SB-MLA-018 |
| SB-MLA-T19 | SB-MLA-019 |
| SB-MLA-T20 | SB-MLA-020 |
| SB-MLA-T21 | SB-MLA-021 |
| SB-MLA-T22 | SB-MLA-022 |
| SB-MLA-T23 | SB-MLA-023 |
| SB-MLA-T24 | SB-MLA-024 |
| SB-MLA-T25 | SB-MLA-047 |
| SB-MLA-T26 | SB-MLA-047 |
| SB-MLA-T27 | SB-MLA-026 |
| SB-MLA-T28 | SB-MLA-027 |
| SB-MLA-T29 | SB-MLA-028 |
| SB-MLA-T30 | SB-MLA-029 |
| SB-MLA-T31 | SB-MLA-030 |
| SB-MLA-T32 | SB-MLA-031 |
| SB-MLA-T33 | SB-MLA-032 |
| SB-MLA-T34 | SB-MLA-033 |
| SB-MLA-T35 | SB-MLA-034 |
| SB-MLA-T36 | SB-MLA-035 |
| SB-MLA-T37 | SB-MLA-036 |
| SB-MLA-T38 | SB-MLA-037 |
| SB-MLA-T39 | SB-MLA-038 |
| SB-MLA-T40 | SB-MLA-039 |
| SB-MLA-T41 | SB-MLA-043 |
| SB-MLA-T42 | SB-MLA-043 |
| SB-MLA-T43 | SB-MLA-044 |
| SB-MLA-T44 | SB-MLA-045 |
| SB-MLA-T45 | SB-MLA-041 |
| SB-MLA-T46 | SB-MLA-042 |
| SB-MLA-T47 | SB-MLA-049 |
| SB-MLA-T48 | SB-MLA-049 |
| SB-MLA-T49 | SB-MLA-050 |
| SB-MLA-T50 | SB-MLA-051 |
| SB-MLA-T51 | SB-MLA-052 |
| SB-MLA-T52 | SB-MLA-053 |
| SB-MLA-T53 | SB-MLA-055 |
| SB-MLA-T54 | SB-MLA-054 |
| SB-MLA-T55 | SB-MLA-056 |
| SB-MLA-T56 | SB-MLA-057 |
| SB-MLA-T57 | SB-MLA-060 |
| SB-MLA-T58 | SB-MLA-061 |
| SB-MLA-T59 | SB-MLA-062 |
| SB-MLA-T60 | SB-MLA-063 |
| SB-MLA-T61 | SB-MLA-064 |

Shared custody retained: T23 also informs SB-MLA-025; T25 also informs SB-MLA-048; T37 also informs SB-MLA-046; T57 also informs SB-MLA-058 and SB-MLA-059. The immutable ownership field is unchanged.

## Parameter, open-item, and source custody

All 105 parameter rows remain exactly as specified. The following 15 values stay absent: facies tie-in acceptance purity; `fuzzy.percentile_error Er`; `fuzzy.weight_bin_by_count`; fuzzy c-means `QQ`; `som.learning_rate_0`; `som.total_iterations`; Geolog neuron/shaking/iteration counts; DYNCLUST `NBCR`/`NBCM`; MRGC electrofacies bounds; KNN `w(d)` and `h`; K.mod training hyperparameters; `nn.epochs`; `nn.cross_validation_pct`; IP neural normalisation scheme; and spherical SOM valid node counts.
The five NON-ADOPTABLE rows remain verification-only: Techlog map size; IP SOM screenshot values; IP cluster-count screenshot value; IP fuzzy screenshot values; and IP sensitivity dither. The three RQI/permeability/HFU rows remain SB-SHR-owned cross-references.
O-1 through O-6, E-1 through E-12, R-1 through R-13, all 24 no-antecedent requirements, and all 218 traceability rows remain source custody. No absent weight law, length scale, decay, threshold, primary equation, vendor artifact, or Tier-C mechanism is adopted.

## Requirement receipts

### SB-MLA-001

- Specified contract: Effective values, default flags, and source IDs for every ML run. Immutable ownership: `SB-MLA-T01`, `SB-MLA-T08`.
- Current implementation: Python runner records effective parameters; native facies defaults still travel through `ModuleOutputs` without an equivalent parameter record.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Record the effective parameter set, not the supplied one contract. Test class MISSING. Supporting source/test evidence is narrower: Python runner records effective parameters; native facies defaults still travel through `ModuleOutputs` without an equivalent parameter record.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python runner records effective parameters; native facies defaults still travel through `ModuleOutputs` without an equivalent parameter record.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python runner records effective parameters; native facies defaults still travel through `ModuleOutputs` without an equivalent parameter record.
- Next action: Complete every missing limb of “Record the effective parameter set, not the supplied one” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-002

- Specified contract: Saved training set identity/version and named drift before apply. Immutable ownership: SB-MLA-T02.
- Current implementation: `TrainingRecord`, picker warnings, apply notes, and named-set drift tests. Confirm warning timing and live-store absence semantics.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T02; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `TrainingRecord`, picker warnings, apply notes, and named-set drift tests. Confirm warning timing and live-store absence semantics.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `TrainingRecord`, picker warnings, apply notes, and named-set drift tests. Confirm warning timing and live-store absence semantics.
- Next action: Retain “A saved model records the input log set it was trained from”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-003

- Specified contract: Stable, discriminating ordered training-matrix fingerprint. Immutable ownership: SB-MLA-T03.
- Current implementation: `training_fingerprint`, model row, curve provenance, and the one-value-change test. Confirm feature/target/order and canonical NaN/-0 coverage.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T03; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `training_fingerprint`, model row, curve provenance, and the one-value-change test. Confirm feature/target/order and canonical NaN/-0 coverage.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `training_fingerprint`, model row, curve provenance, and the one-value-change test. Confirm feature/target/order and canonical NaN/-0 coverage.
- Next action: Retain “A saved model identifies the exact training rows”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-004

- Specified contract: Mask identity/absence and per-well removed counts. Immutable ownership: SB-MLA-T04.
- Current implementation: `TrainingRecord` and mask-effect test. Separate masked, interval-excluded, and incomplete counts; verify the user-visible run message.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T04; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `TrainingRecord` and mask-effect test. Separate masked, interval-excluded, and incomplete counts; verify the user-visible run message.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `TrainingRecord` and mask-effect test. Separate masked, interval-excluded, and incomplete counts; verify the user-visible run message.
- Next action: Retain “A saved model records the exclusion mask and its effect”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-005

- Specified contract: Complete runtime record and pre-apply component warning. Immutable ownership: `SB-MLA-T05`, `SB-MLA-T12`.
- Current implementation: Runtime probe and `runtime_drift` helper exist, but direct apply predicts/writes before the comparison and artifact load failure returns only the Python error tail. Picker-only warning is not an invariant.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A saved model records the runtime that produced it contract. Test class MISSING. Supporting source/test evidence is narrower: Runtime probe and `runtime_drift` helper exist, but direct apply predicts/writes before the comparison and artifact load failure returns only the Python error tail. Picker-only warning is not an invariant.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Runtime probe and `runtime_drift` helper exist, but direct apply predicts/writes before the comparison and artifact load failure returns only the Python error tail. Picker-only warning is not an invariant.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Runtime probe and `runtime_drift` helper exist, but direct apply predicts/writes before the comparison and artifact load failure returns only the Python error tail. Picker-only warning is not an invariant.
- Next action: In Gate 2, reconcile the live behavior with “A saved model records the runtime that produced it” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-006

- Specified contract: Every fitted/applied curve carries a resolvable producing-model ID. Immutable ownership: SB-MLA-T06.
- Current implementation: Saved fits and applies cite IDs; unsaved or failed-save fits still write curves with no model ID. “Not kept” is honest characterization but does not satisfy the universal MUST.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T06; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Saved fits and applies cite IDs; unsaved or failed-save fits still write curves with no model ID. “Not kept” is honest characterization but does not satisfy the universal MUST.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Saved fits and applies cite IDs; unsaved or failed-save fits still write curves with no model ID. “Not kept” is honest characterization but does not satisfy the universal MUST.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Saved fits and applies cite IDs; unsaved or failed-save fits still write curves with no model ID. “Not kept” is honest characterization but does not satisfy the universal MUST.
- Next action: In Gate 2, reconcile the live behavior with “A curve produced by a fitted model names that model” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-007

- Specified contract: Refuse cited deletion; force-delete history and unresolvable mark. Immutable ownership: SB-MLA-T07.
- Current implementation: Backend refuses by default and read-time provenance derives deletion. Force history is frontend-only, so a direct forced command can bypass the permanent record.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A model cited by a stored curve cannot be deleted silently contract. Test class MISSING. Supporting source/test evidence is narrower: Backend refuses by default and read-time provenance derives deletion. Force history is frontend-only, so a direct forced command can bypass the permanent record.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Backend refuses by default and read-time provenance derives deletion. Force history is frontend-only, so a direct forced command can bypass the permanent record.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Backend refuses by default and read-time provenance derives deletion. Force history is frontend-only, so a direct forced command can bypass the permanent record.
- Next action: Complete every missing limb of “A model cited by a stored curve cannot be deleted silently” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-008

- Specified contract: Stored-run replay byte identity for every output and metric. Immutable ownership: `SB-MLA-T01`, `SB-MLA-T08`.
- Current implementation: Ignored all-algorithm runner test passes separately, but it drives a pooled matrix rather than replaying a stored run record; conditional/package evidence is not the whole contract.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `OPTIONAL-PACKAGE-IGNORED` — `SB-MLA-T01`, `SB-MLA-T08`; the whole-path fixture requires the optional Python/scikit-learn/joblib stack and is ignored on the default gate; it passed separately on this machine. Test class OPTIONAL-PACKAGE-IGNORED. Narrow package evidence does not close untested contract limbs.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Ignored all-algorithm runner test passes separately, but it drives a pooled matrix rather than replaying a stored run record; conditional/package evidence is not the whole contract.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Ignored all-algorithm runner test passes separately, but it drives a pooled matrix rather than replaying a stored run record; conditional/package evidence is not the whole contract.
- Next action: Complete every missing limb of “A recorded ML run re-runs to byte-identical curves” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-009

- Specified contract: Blind performance/protocol/held-out count travels with curve, explicit absence otherwise. Immutable ownership: SB-MLA-T09.
- Current implementation: `blind_record`, fit/apply provenance, and the curve-level both-sides test. Keep degraded-protocol behavior under 019 separate.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T09; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `blind_record`, fit/apply provenance, and the curve-level both-sides test. Keep degraded-protocol behavior under 019 separate.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `blind_record`, fit/apply provenance, and the curve-level both-sides test. Keep degraded-protocol behavior under 019 separate.
- Next action: Retain “Blind-well performance travels with the curve”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-010

- Specified contract: Complete ML provenance block in every report/export carrying an ML curve. Immutable ownership: SB-MLA-T10.
- Current implementation: `ml_provenance`, PDF/Word/LAS paths, saved-model export refusal, and deliverable test. Check all required fields and run date, not section placement alone.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T10; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `ml_provenance`, PDF/Word/LAS paths, saved-model export refusal, and deliverable test. Check all required fields and run date, not section placement alone.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `ml_provenance`, PDF/Word/LAS paths, saved-model export refusal, and deliverable test. Check all required fields and run date, not section placement alone.
- Next action: Retain “The deliverable carries the ML provenance block”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-011

- Specified contract: Per-well training/apply roles and counts, including empty contributor. Immutable ownership: SB-MLA-T11.
- Current implementation: Curve records carry a role and result carries prediction counts, but persisted `training_json` filters zero-row wells and does not form one durable per-well two-role roster.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Training and apply membership are recorded per well contract. Test class MISSING. Supporting source/test evidence is narrower: Curve records carry a role and result carries prediction counts, but persisted `training_json` filters zero-row wells and does not form one durable per-well two-role roster.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Curve records carry a role and result carries prediction counts, but persisted `training_json` filters zero-row wells and does not form one durable per-well two-role roster.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Curve records carry a role and result carries prediction counts, but persisted `training_json` filters zero-row wells and does not form one durable per-well two-role roster.
- Next action: Complete every missing limb of “Training and apply membership are recorded per well” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-012

- Specified contract: Loud artifact runtime skew and truthful substituted algorithm. Immutable ownership: `SB-MLA-T12`, `SB-MLA-T27`.
- Current implementation: Actual estimator ID is stored, but deserialisation failure does not name recorded/current runtime and differing component. Keep substitution and load failure as separate limbs.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Artifact version skew fails loudly, and a substituted algorithm is never silent contract. Test class MISSING. Supporting source/test evidence is narrower: Actual estimator ID is stored, but deserialisation failure does not name recorded/current runtime and differing component. Keep substitution and load failure as separate limbs.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Actual estimator ID is stored, but deserialisation failure does not name recorded/current runtime and differing component. Keep substitution and load failure as separate limbs.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Actual estimator ID is stored, but deserialisation failure does not name recorded/current runtime and differing component. Keep substitution and load failure as separate limbs.
- Next action: In Gate 2, reconcile the live behavior with “Artifact version skew fails loudly, and a substituted algorithm is never silent” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-013

- Specified contract: Every engine refuses unclusterable wells and writes no clean empty curve. Immutable ownership: `SB-MLA-T13`, `SB-MLA-T23`.
- Current implementation: Native both-side tests and ignored Python end-to-end test. Verify module, Python fit/apply, result state, and absence of writes.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DEGRADED-RESULT`.
- Automated evidence: `OPTIONAL-PACKAGE-IGNORED` — `SB-MLA-T13`, `SB-MLA-T23`; the whole-path fixture requires the optional Python/scikit-learn/joblib stack and is ignored on the default gate; it passed separately on this machine. Test class OPTIONAL-PACKAGE-IGNORED. Narrow package evidence does not close untested contract limbs.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Native both-side tests and ignored Python end-to-end test. Verify module, Python fit/apply, result state, and absence of writes.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Native both-side tests and ignored Python end-to-end test. Verify module, Python fit/apply, result state, and absence of writes.
- Next action: Retain “An unclusterable well fails; it never emits a clean empty curve”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-014

- Specified contract: Every effective cluster-count change reports requested/effective/reason. Immutable ownership: SB-MLA-T14.
- Current implementation: Python clamp/short metrics exist; native paths refuse some cases and may not report collapsed/empty components. Do not credit refusal as reporting a completed reduced scheme.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A reduced cluster count is reported, never substituted silently contract. Test class MISSING. Supporting source/test evidence is narrower: Python clamp/short metrics exist; native paths refuse some cases and may not report collapsed/empty components. Do not credit refusal as reporting a completed reduced scheme.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python clamp/short metrics exist; native paths refuse some cases and may not report collapsed/empty components. Do not credit refusal as reporting a completed reduced scheme.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python clamp/short metrics exist; native paths refuse some cases and may not report collapsed/empty components. Do not credit refusal as reporting a completed reduced scheme.
- Next action: Complete every missing limb of “A reduced cluster count is reported, never substituted silently” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-015

- Specified contract: Every numerical mixture guard reports component and count. Immutable ownership: SB-MLA-T15.
- Current implementation: Python reports low-weight components; native GMM hard-codes and silently applies `VAR_FLOOR`. A weight heuristic is not proof of variance-floor reporting.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A floored mixture component is reported contract. Test class MISSING. Supporting source/test evidence is narrower: Python reports low-weight components; native GMM hard-codes and silently applies `VAR_FLOOR`. A weight heuristic is not proof of variance-floor reporting.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Python reports low-weight components; native GMM hard-codes and silently applies `VAR_FLOOR`. A weight heuristic is not proof of variance-floor reporting.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Python reports low-weight components; native GMM hard-codes and silently applies `VAR_FLOOR`. A weight heuristic is not proof of variance-floor reporting.
- Next action: In Gate 2, reconcile the live behavior with “A floored mixture component is reported” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-016

- Specified contract: Convergence versus exhaustion, iterations, and final measure for every iterative fit. Immutable ownership: SB-MLA-T16.
- Current implementation: Python k-means/GMM reports a subset; native GMM retains neither terminal state nor final measure. Source comments are not output.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Convergence and iteration exhaustion are distinguished contract. Test class MISSING. Supporting source/test evidence is narrower: Python k-means/GMM reports a subset; native GMM retains neither terminal state nor final measure. Source comments are not output.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python k-means/GMM reports a subset; native GMM retains neither terminal state nor final measure. Source comments are not output.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python k-means/GMM reports a subset; native GMM retains neither terminal state nor final measure. Source comments are not output.
- Next action: Complete every missing limb of “Convergence and iteration exhaustion are distinguished” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-017

- Specified contract: Cancellation stamps incomplete log sets and reports every well. Immutable ownership: SB-MLA-T17.
- Current implementation: `mark_cancelled_sets`, per-well results, and both-sides persisted-set test. Confirm a completed smaller scope is distinguishable.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T17; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `mark_cancelled_sets`, per-well results, and both-sides persisted-set test. Confirm a completed smaller scope is distinguishable.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `mark_cancelled_sets`, per-well results, and both-sides persisted-set test. Confirm a completed smaller scope is distinguishable.
- Next action: Retain “A cancelled run leaves no partially populated log set”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-018

- Specified contract: UI progress declares non-interruptible fit and cancellable write phase. Immutable ownership: SB-MLA-T18.
- Current implementation: Backend comments/phase text exist, but the job is registered cancellable and the UI exposes Cancel without a phase-specific capability state.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — SB-MLA-T18 has no executable phase-observability test. Backend comments, source strings and a visible Cancel control are source evidence only, not a characterization test.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Backend comments/phase text exist, but the job is registered cancellable and the UI exposes Cancel without a phase-specific capability state. T18 is explicitly characterization.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Backend comments/phase text exist, but the job is registered cancellable and the UI exposes Cancel without a phase-specific capability state. T18 is explicitly characterization.
- Next action: In Gate 2, reconcile the live behavior with “The non-interruptible phase is declared, not hidden” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-019

- Specified contract: Collapsed group CV publishes no score. Immutable ownership: SB-MLA-T19.
- Current implementation: Current runner falls back to within-well shuffled KFold, sets `cv_degraded`, and still publishes the score. A warning beside the number is the forbidden behavior.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A cross-validation protocol that degraded MUST NOT report a score as if it had not contract. Test class MISSING. Supporting source/test evidence is narrower: Current runner falls back to within-well shuffled KFold, sets `cv_degraded`, and still publishes the score. A warning beside the number is the forbidden behavior.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current runner falls back to within-well shuffled KFold, sets `cv_degraded`, and still publishes the score. A warning beside the number is the forbidden behavior.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current runner falls back to within-well shuffled KFold, sets `cv_degraded`, and still publishes the score. A warning beside the number is the forbidden behavior.
- Next action: In Gate 2, reconcile the live behavior with “A cross-validation protocol that degraded MUST NOT report a score as if it had not” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-020

- Specified contract: Subsampled metric has count, flag, and distinct name. Immutable ownership: SB-MLA-T20.
- Current implementation: Python silhouette basis records the cap; native silhouette internally caps its comparison reference but emits ordinary per-sample values with no sample-count/subsample designation.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A metric computed on a subsample says so contract. Test class MISSING. Supporting source/test evidence is narrower: Python silhouette basis records the cap; native silhouette internally caps its comparison reference but emits ordinary per-sample values with no sample-count/subsample designation.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Python silhouette basis records the cap; native silhouette internally caps its comparison reference but emits ordinary per-sample values with no sample-count/subsample designation.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Python silhouette basis records the cap; native silhouette internally caps its comparison reference but emits ordinary per-sample values with no sample-count/subsample designation.
- Next action: In Gate 2, reconcile the live behavior with “A metric computed on a subsample says so” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-021

- Specified contract: Reject/noise class differs from missing in data and rendering. Immutable ownership: SB-MLA-T21.
- Current implementation: Shared reject code, output curve, count, neutral palette, discrete detection, and labels. Verify both numeric and rendered sides.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T21; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared reject code, output curve, count, neutral palette, discrete detection, and labels. Verify both numeric and rendered sides.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared reject code, output curve, count, neutral palette, discrete detection, and labels. Verify both numeric and rendered sides.
- Next action: Retain “Density-based noise is a reported class, not a missing value”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-022

- Specified contract: Ordered-feature refusal and unseen-well apply both run on default gate. Immutable ownership: SB-MLA-T22.
- Current implementation: Structural default test exists, but the two required behavioral tests remain `#[ignore]`. Do not treat structural shape as both behavioral contracts.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete The ordered-feature refusal is verified on the default test gate contract. Test class MISSING. Supporting source/test evidence is narrower: Structural default test exists, but the two required behavioral tests remain `#[ignore]`. Do not treat structural shape as both behavioral contracts.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Structural default test exists, but the two required behavioral tests remain `#[ignore]`. Do not treat structural shape as both behavioral contracts.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Structural default test exists, but the two required behavioral tests remain `#[ignore]`. Do not treat structural shape as both behavioral contracts.
- Next action: In Gate 2, reconcile the live behavior with “The ordered-feature refusal is verified on the default test gate” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-023

- Specified contract: One k-means definition and exact cross-engine conformance. Immutable ownership: SB-MLA-T23.
- Current implementation: Constants are shared and a default test compares a small well-separated fixture, but the test can return early without Python and explicitly disclaims general identity required by the specified nontrivial fixture.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T23; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Constants are shared and a default test compares a small well-separated fixture, but the test can return early without Python and explicitly disclaims general identity required by the specified nontrivial fixture.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Constants are shared and a default test compares a small well-separated fixture, but the test can return early without Python and explicitly disclaims general identity required by the specified nontrivial fixture.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Constants are shared and a default test compares a small well-separated fixture, but the test can return early without Python and explicitly disclaims general identity required by the specified nontrivial fixture.
- Next action: In Gate 2, reconcile the live behavior with “One k-means, one definition” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-024

- Specified contract: One seed/default plus identifier stability and disagreement across seeds. Immutable ownership: SB-MLA-T24.
- Current implementation: Shared seed constant and deterministic ordering exist; the three-seed disagreement-rate contract is absent.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete One seed concept, one default contract. Test class MISSING. Supporting source/test evidence is narrower: Shared seed constant and deterministic ordering exist; the three-seed disagreement-rate contract is absent.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Shared seed constant and deterministic ordering exist; the three-seed disagreement-rate contract is absent.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Shared seed constant and deterministic ordering exist; the three-seed disagreement-rate contract is absent.
- Next action: Complete every missing limb of “One seed concept, one default” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-025

- Specified contract: One Ward implementation with free/sorted/depth variants and provenance. Immutable ownership: SB-MLA-T25.
- Current implementation: Sorted-value/depth routes share `WardDp`; free ordering still uses scikit-learn's separate agglomerative implementation and variant provenance is incomplete.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete One within-cluster-sum-of-squares partition, three declared applications contract. Test class MISSING. Supporting source/test evidence is narrower: Sorted-value/depth routes share `WardDp`; free ordering still uses scikit-learn's separate agglomerative implementation and variant provenance is incomplete.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Sorted-value/depth routes share `WardDp`; free ordering still uses scikit-learn's separate agglomerative implementation and variant provenance is incomplete.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Sorted-value/depth routes share `WardDp`; free ordering still uses scikit-learn's separate agglomerative implementation and variant provenance is incomplete.
- Next action: Complete every missing limb of “One within-cluster-sum-of-squares partition, three declared applications” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-026

- Specified contract: Leaderboard and run construct identical estimators for every algorithm/configuration. Immutable ownership: SB-MLA-T27.
- Current implementation: Shared `ML_BUILD_MODEL` and structural/default tests are strong; verify xgboost-present/absent substitution and all non-default hyperparameters rather than one polynomial case.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T27; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Shared `ML_BUILD_MODEL` and structural/default tests are strong; verify xgboost-present/absent substitution and all non-default hyperparameters rather than one polynomial case.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared `ML_BUILD_MODEL` and structural/default tests are strong; verify xgboost-present/absent substitution and all non-default hyperparameters rather than one polynomial case.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared `ML_BUILD_MODEL` and structural/default tests are strong; verify xgboost-present/absent substitution and all non-default hyperparameters rather than one polynomial case.
- Next action: Retain “The leaderboard evaluates the model the run will fit”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-027

- Specified contract: Every score carries a distinct protocol identity. Immutable ownership: SB-MLA-T28.
- Current implementation: `score_protocols`, split metadata, and leaderboard displays. Do not let 019's degraded score become compliant merely because its fallback protocol is described.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Every reported score names its protocol contract. Test class MISSING. Supporting source/test evidence is narrower: `score_protocols`, split metadata, and leaderboard displays. Do not let 019's degraded score become compliant merely because its fallback protocol is described.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `score_protocols`, split metadata, and leaderboard displays. Do not let 019's degraded score become compliant merely because its fallback protocol is described.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `score_protocols`, split metadata, and leaderboard displays. Do not let 019's degraded score become compliant merely because its fallback protocol is described.
- Next action: Retain “Every reported score names its protocol”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-028

- Specified contract: Every learned transform fits inside each fold. Immutable ownership: SB-MLA-T29.
- Current implementation: Shared estimator/pipeline construction, source assertion, and shifted-well test. Confirm importance and every transform, not scaling alone.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T29; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared estimator/pipeline construction, source assertion, and shifted-well test. Confirm importance and every transform, not scaling alone.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared estimator/pipeline construction, source assertion, and shifted-well test. Confirm importance and every transform, not scaling alone.
- Next action: Retain “Every fitted transform is fitted inside the fold”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-029

- Specified contract: Every class mnemonic identifies engine and different methods cannot collide. Immutable ownership: SB-MLA-T30.
- Current implementation: Native k-means still writes generic `FACIES`; Python output base remains user-editable. Disjoint native engine sets do not prove the universal naming rule.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A facies mnemonic names the engine that produced it contract. Test class MISSING. Supporting source/test evidence is narrower: Native k-means still writes generic `FACIES`; Python output base remains user-editable. Disjoint native engine sets do not prove the universal naming rule.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Native k-means still writes generic `FACIES`; Python output base remains user-editable. Disjoint native engine sets do not prove the universal naming rule.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Native k-means still writes generic `FACIES`; Python output base remains user-editable. Disjoint native engine sets do not prove the universal naming rule.
- Next action: Complete every missing limb of “A facies mnemonic names the engine that produced it” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-030

- Specified contract: Probability quantity/normalisation types and disjoint mnemonic conventions. Immutable ownership: SB-MLA-T31.
- Current implementation: Definitions are recorded, but posterior, membership strength, and classifier probability still share `_PROB`. Descriptive metadata does not satisfy the MUST NOT share convention.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Probability outputs are typed contract. Test class MISSING. Supporting source/test evidence is narrower: Definitions are recorded, but posterior, membership strength, and classifier probability still share `_PROB`. Descriptive metadata does not satisfy the MUST NOT share convention.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Definitions are recorded, but posterior, membership strength, and classifier probability still share `_PROB`. Descriptive metadata does not satisfy the MUST NOT share convention.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Definitions are recorded, but posterior, membership strength, and classifier probability still share `_PROB`. Descriptive metadata does not satisfy the MUST NOT share convention.
- Next action: In Gate 2, reconcile the live behavior with “Probability outputs are typed” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-031

- Specified contract: Competing vendor defaults shown at point of choice and decision recorded. Immutable ownership: SB-MLA-T32.
- Current implementation: Source registry and backend decision note exist; the separate ML dialog shows only one `K=5` field without competing values/sources. A run-time note is not point-of-choice evidence.
- Verdict: `PARTIAL`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Shipped vendor defaults are surfaced at the point of choice contract. Test class MISSING. Supporting source/test evidence is narrower: Source registry and backend decision note exist; the separate ML dialog shows only one `K=5` field without competing values/sources. A run-time note is not point-of-choice evidence.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Source registry and backend decision note exist; the separate ML dialog shows only one `K=5` field without competing values/sources. A run-time note is not point-of-choice evidence.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Source registry and backend decision note exist; the separate ML dialog shows only one `K=5` field without competing values/sources. A run-time note is not point-of-choice evidence.
- Next action: Complete every missing limb of “Shipped vendor defaults are surfaced at the point of choice” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-032

- Specified contract: Explicit named normalisation basis in every distance-based output provenance. Immutable ownership: SB-MLA-T33.
- Current implementation: Python and native choices/records exist. Verify every method and native generic-workflow provenance, not only one runner metric.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete The normalisation basis is a recorded choice, not an implicit one contract. Test class MISSING. Supporting source/test evidence is narrower: Python and native choices/records exist. Verify every method and native generic-workflow provenance, not only one runner metric.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Python and native choices/records exist. Verify every method and native generic-workflow provenance, not only one runner metric.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Python and native choices/records exist. Verify every method and native generic-workflow provenance, not only one runner metric.
- Next action: Retain “The normalisation basis is a recorded choice, not an implicit one”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-033

- Specified contract: Stored fixed limits plus reported rescaling under data-derived basis. Immutable ownership: SB-MLA-T34.
- Current implementation: Python fixed basis and movement warning exist; native facies remains data-derived without fixed limits. A whole-product requirement cannot be closed by one engine.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A fixed normalisation basis is available, so adding a well does not move existing boundaries contract. Test class MISSING. Supporting source/test evidence is narrower: Python fixed basis and movement warning exist; native facies remains data-derived without fixed limits. A whole-product requirement cannot be closed by one engine.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python fixed basis and movement warning exist; native facies remains data-derived without fixed limits. A whole-product requirement cannot be closed by one engine.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Python fixed basis and movement warning exist; native facies remains data-derived without fixed limits. A whole-product requirement cannot be closed by one engine.
- Next action: Complete every missing limb of “A fixed normalisation basis is available, so adding a well does not move existing boundaries” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-034

- Specified contract: Every automatic transform announced per curve, including none. Immutable ownership: SB-MLA-T35.
- Current implementation: Feature-transform records and run notes. Distinguish explicit user transforms from automatic family behavior and verify the no-transform side.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — transform-resolution tests do not assert both the enabled and disabled announcements on the run's reporting surface required by SB-MLA-T35.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Feature-transform records and run notes. Distinguish explicit user transforms from automatic family behavior and verify the no-transform side.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Feature-transform records and run notes. Distinguish explicit user transforms from automatic family behavior and verify the no-transform side.
- Next action: Retain “Every automatic pre-transform is announced”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-035

- Specified contract: Transformed quantity has distinct mnemonic/unit and explicit back-transform. Immutable ownership: SB-MLA-T36.
- Current implementation: Separate log/back curves, curve-unit registry, metric-space note, and direct test. Export is not the assertion vehicle.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T36; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Separate log/back curves, curve-unit registry, metric-space note, and direct test. Export is not the assertion vehicle.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Separate log/back curves, curve-unit registry, metric-space note, and direct test. Export is not the assertion vehicle.
- Next action: Retain “A transformed quantity is a distinct quantity with its own name and unit”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-036

- Specified contract: Canonical IDs, labels, vendor aliases, and hard-fail unknown. Immutable ownership: SB-MLA-T37.
- Current implementation: Current options use canonical-like IDs and reject unknown linkage/scaling, but the specified vendor aliases and complete enumeration/update-rule registry are absent.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Enumerated methods are addressed by id, never by display string contract. Test class MISSING. Supporting source/test evidence is narrower: Current options use canonical-like IDs and reject unknown linkage/scaling, but the specified vendor aliases and complete enumeration/update-rule registry are absent.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current options use canonical-like IDs and reject unknown linkage/scaling, but the specified vendor aliases and complete enumeration/update-rule registry are absent.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current options use canonical-like IDs and reject unknown linkage/scaling, but the specified vendor aliases and complete enumeration/update-rule registry are absent.
- Next action: In Gate 2, reconcile the live behavior with “Enumerated methods are addressed by id, never by display string” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-037

- Specified contract: Cuddy reciprocal fuzzy combination, never product. Immutable ownership: SB-MLA-T38.
- Current implementation: No fuzzy predictor is registered. Keep the printed equation and regression fixture as future correctness evidence only.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Fuzzy combination across curves is the reciprocal sum contract. Test class MISSING. Supporting source/test evidence is narrower: No fuzzy predictor is registered. Keep the printed equation and regression fixture as future correctness evidence only.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy predictor is registered. Keep the printed equation and regression fixture as future correctness evidence only.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy predictor is registered. Keep the printed equation and regression fixture as future correctness evidence only.
- Next action: Decide pilot inclusion; if included, implement “Fuzzy combination across curves is the reciprocal sum” only from chapter-authorized sources and add SB-MLA-T38.

### SB-MLA-038

- Specified contract: Equal-population binning reports achieved populations. Immutable ownership: SB-MLA-T39.
- Current implementation: No fuzzy binning route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Equal-population binning reports its actual populations contract. Test class MISSING. Supporting source/test evidence is narrower: No fuzzy binning route exists.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy binning route exists.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy binning route exists.
- Next action: Decide pilot inclusion; if included, implement “Equal-population binning reports its actual populations” only from chapter-authorized sources and add SB-MLA-T39.

### SB-MLA-039

- Specified contract: Fuzzy uncertainty edge fallback and fired record. Immutable ownership: SB-MLA-T40.
- Current implementation: No fuzzy band route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete The fuzzy uncertainty band has a defined edge behaviour contract. Test class MISSING. Supporting source/test evidence is narrower: No fuzzy band route exists.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy band route exists.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy band route exists.
- Next action: Decide pilot inclusion; if included, implement “The fuzzy uncertainty band has a defined edge behaviour” only from chapter-authorized sources and add SB-MLA-T40.

### SB-MLA-040

- Specified contract: Visible no-default bin-count weighting and declared Cuddy deviation. Immutable ownership: SB-MLA-T38.
- Current implementation: No fuzzy weighting UI/parameter exists; the absent default remains absent.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete The bin-count weighting is explicit, with no hidden default contract. Test class MISSING. Supporting source/test evidence is narrower: No fuzzy weighting UI/parameter exists; the absent default remains absent.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy weighting UI/parameter exists; the absent default remains absent.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No fuzzy weighting UI/parameter exists; the absent default remains absent.
- Next action: Decide pilot inclusion; if included, implement “The bin-count weighting is explicit, with no hidden default” only from chapter-authorized sources and add SB-MLA-T38.

### SB-MLA-041

- Specified contract: Required total-iteration SOM decay and refusal of degenerate form. Immutable ownership: SB-MLA-T45.
- Current implementation: No SOM route exists; `som.total_iterations` remains absent/required and E-1 stays open.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete SOM decay is parameterised by total iterations, and the degenerate form is refused contract. Test class MISSING. Supporting source/test evidence is narrower: No SOM route exists; `som.total_iterations` remains absent/required and E-1 stays open.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No SOM route exists; `som.total_iterations` remains absent/required and E-1 stays open.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No SOM route exists; `som.total_iterations` remains absent/required and E-1 stays open.
- Next action: Decide pilot inclusion; if included, implement “SOM decay is parameterised by total iterations, and the degenerate form is refused” only from chapter-authorized sources and add SB-MLA-T45.

### SB-MLA-042

- Specified contract: Defined SOM distortion with lower-is-better and radius. Immutable ownership: SB-MLA-T46.
- Current implementation: No SOM route exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Map quality is reported by a defined distortion measure contract. Test class MISSING. Supporting source/test evidence is narrower: No SOM route exists.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No SOM route exists.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No SOM route exists.
- Next action: Decide pilot inclusion; if included, implement “Map quality is reported by a defined distortion measure” only from chapter-authorized sources and add SB-MLA-T46.

### SB-MLA-043

- Specified contract: Aggregate cluster randomness index matching cited equation and random anchor. Immutable ownership: `SB-MLA-T41`, `SB-MLA-T42`.
- Current implementation: Current code emits per-cluster `observed_mean_run*(1-p)` as a depth curve. T41 specifies one aggregate `Av/Random` value; the current direction/anchor test characterizes a different formula.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — `SB-MLA-T41`, `SB-MLA-T42`; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Current code emits per-cluster `observed_mean_run*(1-p)` as a depth curve. T41 specifies one aggregate `Av/Random` value; the current direction/anchor test characterizes a different formula.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current code emits per-cluster `observed_mean_run*(1-p)` as a depth curve. T41 specifies one aggregate `Av/Random` value; the current direction/anchor test characterizes a different formula.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Current code emits per-cluster `observed_mean_run*(1-p)` as a depth curve. T41 specifies one aggregate `Av/Random` value; the current direction/anchor test characterizes a different formula.
- Next action: In Gate 2, reconcile the live behavior with “The cluster randomness index ships” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-044

- Specified contract: Per-cluster and overall geometric quality with sample count. Immutable ownership: SB-MLA-T43.
- Current implementation: Native path emits per-sample silhouette curves only; its test compares direction and does not exercise per-cluster/overall reporting or explicit count.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T43; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Native path emits per-sample silhouette curves only; its test compares direction and does not exercise per-cluster/overall reporting or explicit count.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Native path emits per-sample silhouette curves only; its test compares direction and does not exercise per-cluster/overall reporting or explicit count.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Native path emits per-sample silhouette curves only; its test compares direction and does not exercise per-cluster/overall reporting or explicit count.
- Next action: In Gate 2, reconcile the live behavior with “The native clustering path reports cluster quality” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-045

- Specified contract: Restart objective distribution, retained-hit count, and caveat. Immutable ownership: SB-MLA-T44.
- Current implementation: Restarts retain only the best objective; no spread or hit-frequency report exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Restart spread is reported as a convergence diagnostic contract. Test class MISSING. Supporting source/test evidence is narrower: Restarts retain only the best objective; no spread or hit-frequency report exists.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. Restarts retain only the best objective; no spread or hit-frequency report exists.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. Restarts retain only the best objective; no spread or hit-frequency report exists.
- Next action: Decide pilot inclusion; if included, implement “Restart spread is reported as a convergence diagnostic” only from chapter-authorized sources and add SB-MLA-T44.

### SB-MLA-046

- Specified contract: Five linkage rules/update equations, Ward default, corroborated source. Immutable ownership: SB-MLA-T37.
- Current implementation: Runner accepts four linkages, UI offers three, and no five-rule update/source registry exists. Do not call partial enumerations five.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Hierarchical linkage is a named enumeration with a sourced default contract. Test class MISSING. Supporting source/test evidence is narrower: Runner accepts four linkages, UI offers three, and no five-rule update/source registry exists. Do not call partial enumerations five.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Runner accepts four linkages, UI offers three, and no five-rule update/source registry exists. Do not call partial enumerations five.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Runner accepts four linkages, UI offers three, and no five-rule update/source registry exists. Do not call partial enumerations five.
- Next action: In Gate 2, reconcile the live behavior with “Hierarchical linkage is a named enumeration with a sourced default” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-047

- Specified contract: PCA loadings plus correlation-circle coordinates. Immutable ownership: SB-MLA-T26.
- Current implementation: Loadings and explained variance ship; correlation-circle coordinates/eigenvalue scaling are absent despite source-backed T26.
- Verdict: `PARTIAL`; release `UNDECIDED`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — none; no executable test proves the complete PCA reports loadings and correlation-circle coordinates contract. Test class MISSING. Supporting source/test evidence is narrower: Loadings and explained variance ship; correlation-circle coordinates/eigenvalue scaling are absent despite source-backed T26.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Loadings and explained variance ship; correlation-circle coordinates/eigenvalue scaling are absent despite source-backed T26.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Loadings and explained variance ship; correlation-circle coordinates/eigenvalue scaling are absent despite source-backed T26.
- Next action: Complete every missing limb of “PCA reports loadings and correlation-circle coordinates” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-048

- Specified contract: Fixed documented PC sign convention. Immutable ownership: SB-MLA-T08.
- Current implementation: First-feature non-negative convention is implemented; verify scores and loadings together and do not borrow optional all-algorithm determinism as the only proof.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Component sign is fixed by a stated convention contract. Test class MISSING. Supporting source/test evidence is narrower: First-feature non-negative convention is implemented; verify scores and loadings together and do not borrow optional all-algorithm determinism as the only proof.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. First-feature non-negative convention is implemented; verify scores and loadings together and do not borrow optional all-algorithm determinism as the only proof.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. First-feature non-negative convention is implemented; verify scores and loadings together and do not borrow optional all-algorithm determinism as the only proof.
- Next action: Retain “Component sign is fixed by a stated convention”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-049

- Specified contract: Normalised KNN average with SandiBumi-owned weight function and first-class length scale. Immutable ownership: `SB-MLA-T47`, `SB-MLA-T48`.
- Current implementation: scikit-learn KNN exists with `uniform`/`distance`; no SandiBumi-owned `w(d),h` contract or length-scale parameter exists. Section-5 absence remains binding.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Nearest-neighbour prediction is a normalised weighted average, and its weight function is SandiBumi's contract. Test class MISSING. Supporting source/test evidence is narrower: scikit-learn KNN exists with `uniform`/`distance`; no SandiBumi-owned `w(d),h` contract or length-scale parameter exists. Section-5 absence remains binding.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. scikit-learn KNN exists with `uniform`/`distance`; no SandiBumi-owned `w(d),h` contract or length-scale parameter exists. Section-5 absence remains binding.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. scikit-learn KNN exists with `uniform`/`distance`; no SandiBumi-owned `w(d),h` contract or length-scale parameter exists. Section-5 absence remains binding.
- Next action: In Gate 2, reconcile the live behavior with “Nearest-neighbour prediction is a normalised weighted average, and its weight function is SandiBumi's” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-050

- Specified contract: Held-out frame excluded from neighbour search and zero self-score refused. Immutable ownership: SB-MLA-T49.
- Current implementation: `log_predict` skips self and has a breaking-test fixture; leaderboard feature scoring uses held-out wells. Keep dedicated vendor-style ranking absence distinct from the safety contract.
- Verdict: `PRESENT-OK`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T49; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `log_predict` skips self and has a breaking-test fixture; leaderboard feature scoring uses held-out wells. Keep dedicated vendor-style ranking absence distinct from the safety contract.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. `log_predict` skips self and has a breaking-test fixture; leaderboard feature scoring uses held-out wells. Keep dedicated vendor-style ranking absence distinct from the safety contract.
- Next action: Retain “Feature scoring by leave-one-out excludes the held-out frame”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-051

- Specified contract: Raw, row-normalised, and column-normalised contingency with named axes. Immutable ownership: SB-MLA-T50.
- Current implementation: Backend matrices and UI modes exist; non-square/non-symmetric test pins direction and sums.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T50; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Backend matrices and UI modes exist; non-square/non-symmetric test pins direction and sums.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Backend matrices and UI modes exist; non-square/non-symmetric test pins direction and sums.
- Next action: Retain “A contingency table carries both normalisations, each labelled with its axis”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-052

- Specified contract: Acceptance threshold absent, visible, required, and persisted when chosen. Immutable ownership: SB-MLA-T51.
- Current implementation: Optional backend field, empty UI, explicit no-source caption, result/process record, and both-sides test. No default may be inferred.
- Verdict: `PRESENT-OK`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T51; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Optional backend field, empty UI, explicit no-source caption, result/process record, and both-sides test. No default may be inferred.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Optional backend field, empty UI, explicit no-source caption, result/process record, and both-sides test. No default may be inferred.
- Next action: Retain “The tie-in acceptance threshold ships absent and visible”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-053

- Specified contract: Spread-multiple names its statistic; bare tolerance is ambiguous. Immutable ownership: SB-MLA-T52.
- Current implementation: DBSCAN records `eps_unit`, but the specified imported bare-tolerance refusal and `tolerance_sd` outlier configuration are absent. Do not redefine the test around `eps`.
- Verdict: `PARTIAL`; release `DEFERRED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A tolerance expressed in standard deviations is named for its unit contract. Test class MISSING. Supporting source/test evidence is narrower: DBSCAN records `eps_unit`, but the specified imported bare-tolerance refusal and `tolerance_sd` outlier configuration are absent. Do not redefine the test around `eps`.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. DBSCAN records `eps_unit`, but the specified imported bare-tolerance refusal and `tolerance_sd` outlier configuration are absent. Do not redefine the test around `eps`.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. DBSCAN records `eps_unit`, but the specified imported bare-tolerance refusal and `tolerance_sd` outlier configuration are absent. Do not redefine the test around `eps`.
- Next action: Complete every missing limb of “A tolerance expressed in standard deviations is named for its unit” across all applicable engines/surfaces and add one qualifying whole-contract test.

### SB-MLA-054

- Specified contract: Every resampling/snap decision recorded per curve. Immutable ownership: SB-MLA-T54.
- Current implementation: Main ML uses exact-depth equality and reports zero-contribution off-grid curves; facies tie records nearest-match tolerance/counts. Verify the chapter's explicit no-resampling interpretation without claiming partial overlaps are documented.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T54; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Main ML uses exact-depth equality and reports zero-contribution off-grid curves; facies tie records nearest-match tolerance/counts. Verify the chapter's explicit no-resampling interpretation without claiming partial overlaps are documented.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Main ML uses exact-depth equality and reports zero-contribution off-grid curves; facies tie records nearest-match tolerance/counts. Verify the chapter's explicit no-resampling interpretation without claiming partial overlaps are documented.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Main ML uses exact-depth equality and reports zero-contribution off-grid curves; facies tie records nearest-match tolerance/counts. Verify the chapter's explicit no-resampling interpretation without claiming partial overlaps are documented.
- Next action: Retain “The depth-resampling decision is logged for every ML input”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-055

- Specified contract: Categorical registry prevents every interpolation/average path. Immutable ownership: SB-MLA-T53.
- Current implementation: Curve-class registry, workflow declarations, reframe substitution, block/smooth/despike refusals, and both-sides tests. Probability curves remain continuous.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T53; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Curve-class registry, workflow declarations, reframe substitution, block/smooth/despike refusals, and both-sides tests. Probability curves remain continuous.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Curve-class registry, workflow declarations, reframe substitution, block/smooth/despike refusals, and both-sides tests. Probability curves remain continuous.
- Next action: Retain “A class label is never interpolated”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-056

- Specified contract: NaN/null discipline through every ML path with no opt-out. Immutable ownership: SB-MLA-T55.
- Current implementation: Finite-row filters exist, but the exact declared-sentinel import-to-ML/equation fixture and settings-surface search are not one whole-contract test. Never treat an undeclared finite number as null.
- Verdict: `PRESENT-UNVERIFIED`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Null discipline holds through the ML path with no opt-out contract. Test class MISSING. Supporting source/test evidence is narrower: Finite-row filters exist, but the exact declared-sentinel import-to-ML/equation fixture and settings-surface search are not one whole-contract test. Never treat an undeclared finite number as null.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The architecture appears to satisfy the contract, but no qualifying whole-contract proof exists. Finite-row filters exist, but the exact declared-sentinel import-to-ML/equation fixture and settings-surface search are not one whole-contract test. Never treat an undeclared finite number as null.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The architecture appears to satisfy the contract, but no qualifying whole-contract proof exists. Finite-row filters exist, but the exact declared-sentinel import-to-ML/equation fixture and settings-surface search are not one whole-contract test. Never treat an undeclared finite number as null.
- Next action: Add a qualifying independently sourced whole-contract test for “Null discipline holds through the ML path with no opt-out” before treating it as pilot-proven.

### SB-MLA-057

- Specified contract: Threshold absence is distinct; missing sentinel cannot be stored as value. Immutable ownership: SB-MLA-T56.
- Current implementation: Shared Python parameter guard rejects known sentinels, but the specified set/import both-sides test is absent. Confirm all parameter doors, not only runner source text.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A threshold value can never be confused with a missing value contract. Test class MISSING. Supporting source/test evidence is narrower: Shared Python parameter guard rejects known sentinels, but the specified set/import both-sides test is absent. Confirm all parameter doors, not only runner source text.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared Python parameter guard rejects known sentinels, but the specified set/import both-sides test is absent. Confirm all parameter doors, not only runner source text.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Shared Python parameter guard rejects known sentinels, but the specified set/import both-sides test is absent. Confirm all parameter doors, not only runner source text.
- Next action: Retain “A threshold value can never be confused with a missing value”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-058

- Specified contract: Tier-C capabilities named and never approximated. Immutable ownership: SB-MLA-T57.
- Current implementation: Static dependency/source boundary test. A name search is supporting evidence; preserve all register semantics and primary-source requirements.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — the executable governance test explicitly characterizes the shipped Tier-C register policy; it does not independently prove every named capability or design-around route.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Static dependency/source boundary test. A name search is supporting evidence; preserve all register semantics and primary-source requirements.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Static dependency/source boundary test. A name search is supporting evidence; preserve all register semantics and primary-source requirements.
- Next action: Retain “Tier-C capabilities are named, never approximated”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-059

- Specified contract: Any design-around independently derived, labelled, cited, and provenanced. Immutable ownership: SB-MLA-T57.
- Current implementation: No design-around is specified or built; do not invent one during adjudication.
- Verdict: `ABSENT`; release `DEFERRED`; risk `LATER`.
- Automated evidence: `MISSING` — none; no executable test proves the complete The user need behind a Tier-C capability may be served by an independently derived feature contract. Test class MISSING. Supporting source/test evidence is narrower: No design-around is specified or built; do not invent one during adjudication.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No design-around is specified or built; do not invent one during adjudication.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The capability remains unbuilt; retain chapter-cited source and absent-parameter gates. No design-around is specified or built; do not invent one during adjudication.
- Next action: Decide pilot inclusion; if included, implement “The user need behind a Tier-C capability may be served by an independently derived feature” only from chapter-authorized sources and add SB-MLA-T57.

### SB-MLA-060

- Specified contract: No vendor model, weight, or tile artifact reader. Immutable ownership: SB-MLA-T57.
- Current implementation: Static dependency/import/path/extension guard and ordinary-curve interchange boundary. Keep SandiBumi-owned joblib blobs distinct.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CORRECTNESS` — SB-MLA-T57; expected behavior/value comes from the chapter's named input, expected result and cited source, independently asserted at the relevant stored/output surface. Test class CORRECTNESS.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Static dependency/import/path/extension guard and ordinary-curve interchange boundary. Keep SandiBumi-owned joblib blobs distinct.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Static dependency/import/path/extension guard and ordinary-curve interchange boundary. Keep SandiBumi-owned joblib blobs distinct.
- Next action: Retain “No vendor model or weight file is read, converted or imported”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-061

- Specified contract: Missing component names component, inspected interpreter, exact command; native routes remain. Immutable ownership: SB-MLA-T58.
- Current implementation: Shared resolver is richer, but ML's hard-coded missing-Python/sklearn messages omit the exact inspected interpreter or interpreter-qualified install command; joblib failure is post-fit only.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DEPLOYMENT`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A missing interpreter is a named, actionable failure contract. Test class MISSING. Supporting source/test evidence is narrower: Shared resolver is richer, but ML's hard-coded missing-Python/sklearn messages omit the exact inspected interpreter or interpreter-qualified install command; joblib failure is post-fit only.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Shared resolver is richer, but ML's hard-coded missing-Python/sklearn messages omit the exact inspected interpreter or interpreter-qualified install command; joblib failure is post-fit only.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Shared resolver is richer, but ML's hard-coded missing-Python/sklearn messages omit the exact inspected interpreter or interpreter-qualified install command; joblib failure is post-fit only.
- Next action: In Gate 2, reconcile the live behavior with “A missing interpreter is a named, actionable failure” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-062

- Specified contract: No lock across subprocess/unbounded wait and release between every well. Immutable ownership: SB-MLA-T59.
- Current implementation: Fit path computes lock-free and writes per well; saved-model apply acquires one lock before the loop and holds it across all apply wells.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A long fit does not hold the global write lock contract. Test class MISSING. Supporting source/test evidence is narrower: Fit path computes lock-free and writes per well; saved-model apply acquires one lock before the loop and holds it across all apply wells.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Fit path computes lock-free and writes per well; saved-model apply acquires one lock before the loop and holds it across all apply wells.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Fit path computes lock-free and writes per well; saved-model apply acquires one lock before the loop and holds it across all apply wells.
- Next action: In Gate 2, reconcile the live behavior with “A long fit does not hold the global write lock” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-063

- Specified contract: Every binding cap reports request, cap, and dropped work. Immutable ownership: SB-MLA-T60.
- Current implementation: Combination/t-SNE/silhouette caps are surfaced; native `K>12` now refuses while T60 specifies a reported clamp. Record the specified-test divergence without weakening refusal safety.
- Verdict: `PRESENT-DIVERGENT`; release `UNDECIDED`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — none; no executable test proves the complete Every capacity cap is a declared limit, not a silent truncation contract. Test class MISSING. Supporting source/test evidence is narrower: Combination/t-SNE/silhouette caps are surfaced; native `K>12` now refuses while T60 specifies a reported clamp. Record the specified-test divergence without weakening refusal safety.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Combination/t-SNE/silhouette caps are surfaced; native `K>12` now refuses while T60 specifies a reported clamp. Record the specified-test divergence without weakening refusal safety.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The shipped path conflicts with a specified limb and requires a Gate 2 remediation decision/test, not a documentation reinterpretation. Combination/t-SNE/silhouette caps are surfaced; native `K>12` now refuses while T60 specifies a reported clamp. Record the specified-test divergence without weakening refusal safety.
- Next action: In Gate 2, reconcile the live behavior with “Every capacity cap is a declared limit, not a silent truncation” and add one qualifying observable whole-contract test without weakening existing refusals.

### SB-MLA-064

- Specified contract: Model listing never materialises artifacts. Immutable ownership: SB-MLA-T61.
- Current implementation: Listing query omits blob and test characterizes that behavior. T61 is explicitly characterization.
- Verdict: `PRESENT-OK`; release `UNDECIDED`; risk `DEGRADED-RESULT`.
- Automated evidence: `CHARACTERIZATION` — SB-MLA-T61; expected behavior is what the current implementation does and is not an independent scientific or product proof. Test class CHARACTERIZATION. Listing query omits blob and test characterizes that behavior. T61 is explicitly characterization.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Listing query omits blob and test characterizes that behavior. T61 is explicitly characterization.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: The current contract is implemented, but automated evidence does not replace the domain's open manual and field evidence. Listing query omits blob and test characterizes that behavior. T61 is explicitly characterization.
- Next action: Retain “The model registry lists without materialising artifacts”; preserve its current guard and add any missing manual/field evidence before release.

### SB-MLA-065

- Specified contract: Portfolio progress, phase cancellation truth, bounds, and per-well outcome classes. Immutable ownership: `SB-MLA-T17`, `SB-MLA-T18`.
- Current implementation: Per-well results, cancellation/partial stamps, and some caps exist; non-interruptible phase UI, apply lock, fit bound, real-scale evidence, and complete outcome taxonomy remain open.
- Verdict: `PARTIAL`; release `UNDECIDED`; risk `FIELD-EVIDENCE`.
- Automated evidence: `MISSING` — none; no executable test proves the complete A portfolio-scale ML run is bounded, cancellable and honestly reported contract. Test class MISSING. Supporting source/test evidence is narrower: Per-well results, cancellation/partial stamps, and some caps exist; non-interruptible phase UI, apply lock, fit bound, real-scale evidence, and complete outcome taxonomy remain open.
- Manual evidence: electrofacies 2/26; machine-learning 7/189; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; portfolio-performance 0/50; processing-history 0/7; workspace-shell 0/159; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Per-well results, cancellation/partial stamps, and some caps exist; non-interruptible phase UI, apply lock, fit bound, real-scale evidence, and complete outcome taxonomy remain open.
- UI/IPC/provenance surface: The row was checked across every applicable runner, native engine, saved-model, UI, persisted curve/log-set, report/export, and default-gate surface named above; helper-only evidence was not promoted.
- History/reachability: accepted implementation anchor `b332026cb498c105f36eade0bf7899bc0c1309f0` is reachable; targeted reachable-history review found no later implementation that closes the recorded missing or divergent limbs.
- Decision/dependency: One or more observable limbs remain incomplete across runner, native, UI, persistence, report/export, or default-gate surfaces. Per-well results, cancellation/partial stamps, and some caps exist; non-interruptible phase UI, apply lock, fit bound, real-scale evidence, and complete outcome taxonomy remain open.
- Next action: Complete every missing limb of “A portfolio-scale ML run is bounded, cancellable and honestly reported” across all applicable engines/surfaces and add one qualifying whole-contract test.

## Verification boundary

The execution increment must finish with `npx tsc --noEmit`, `cargo check`, and the full `tools\check.ps1` gate green. Those automated checks prove repository health only; they do not change any manual or field-evidence count.
