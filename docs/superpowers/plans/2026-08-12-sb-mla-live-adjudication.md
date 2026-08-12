# SB-MLA Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 65 live `SB-MLA` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 61 chapter acceptance-test intentions, and preserve every model, data-frame, method, runtime, determinism, legal, parameter, provenance, scale, and manual-evidence boundary without changing ML behavior.

**Architecture:** This is a documentation-only evidence pass. Requirements 001-012 cover persisted-model and curve provenance. Requirements 013-022 cover fail-loud behavior, cancellation, cross-validation, sampling, noise, and default-gate coverage. Requirements 023-036 cover shared method identities, evaluation parity, normalization, naming, typed outputs, and enumerations. Requirements 037-050 cover fuzzy, SOM, clustering diagnostics, hierarchical clustering, PCA, nearest-neighbour prediction, and feature scoring. Requirements 051-065 cover tie-in, depth/null custody, the Tier-C boundary, runtime/deployment, lock discipline, caps, registry listing, and portfolio scale. The immutable PRD supplies the intended contract, 105 parameter rows, 6 open items, 12 escalations, 13 refusals, 61 named test intentions, and 218 section-8 traceability rows. Current source, independent tests, manual evidence, and reachable Git history supply the separate live verdict. A private helper, source comment, optional-package test, internal `Result`, returned note, or renderer label does not alone prove a compound persisted-output or UI contract.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, tests, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on GPT-5.6 Sol at xhigh with `superpowers:executing-plans`. Do not delegate or spawn subagents unless Jauhar explicitly authorizes it. ML method interpretation, parameter custody, source boundaries, and final sign-off stay with the primary session. Reserve Sol max for the final all-931-row Gate 1 audit.
- Work only in `D:\XX. SandiBumi`. It MUST remain the sole registered Git worktree. The retired `D:\XX. SandiBumi-check` path remains untouched.
- The accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `aa927f14ac56c072166afb7d9ec53ddf78f3354a`; `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution.
- The local planning branch is `codex/g1-sb-mla-plan`. The serial Gate 1 chain remains local and unpushed; do not merge, rebase, rewrite history, push, or open a pull request. After the planning commit, create `codex/g1-sb-mla-adjudication` in the same worktree.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential Rust/TypeScript absence MUST be confirmed across ML runners, native facies paths, saved-model storage/apply, shared dispatch, IPC/UI, curve metadata, reports/exports, tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/24_ml-advanced.md`, `docs/record_parallel_lanes.md`, the current verification matrix, takeover receipts/status, and the exact source/tests about to be cited. No dedicated ML record currently exists.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 65 ordered rows is `82626708eb32931956503fd38daf9d25f179565e718dcba8b6804eaa2244f8dd`. The immutable chapter SHA-256 is `6cd4697c611652f66ecdb4a329f9adccf321fecf3aad9aeb8c3a609fad8e219c`.
- The chapter and ledger agree on 65 contiguous requirements: `P0=10`, `P1=34`, `P2=17`, `P3=4`. Historical states are `PRESENT-OK=53`, `PRESENT-DIVERGENT=1`, `PARTIAL=2`, and `ABSENT=9`; all 65 live verdicts remain `UNADJUDICATED`. Reverify live behavior independently rather than copying those labels.
- The chapter defines 61 contiguous test IDs, `SB-MLA-T01` through `SB-MLA-T61`. The ledger has 73 ownership references covering all 61 unique IDs, with no ownership gap or unknown ID. Route each unique test intention once and preserve shared ownership without duplicating evidence.
- Section 5 contains 105 parameter rows, including 15 deliberate `ABSENT` values, 5 `NON-ADOPTABLE` values, and 3 cross-references to SB-SHR. Preserve every absence and fence. A scikit-learn default, code literal, vendor screenshot, historical note, or neighboring implementation never becomes parameter authority.
- Preserve O-1 through O-6, E-1 through E-12, R-1 through R-13, all 24 requirements without dossier antecedents, and all 218 traceability rows. No Tier-C capability, vendor model/weight artifact, unprinted KNN weight law, SOM decay, fuzzy default, or absent primary-source method may be inferred from current code.
- `sys.stdin.buffer` and bytemuck byte transport remain mandatory. This lane must not reinterpret passing subprocess tests as permission to change the transport or runtime boundary.
- Optional-package tests remain optional evidence. A test that returns early when Python is missing is not a universal default-gate proof merely because it passed on this machine. The seven ignored ML tests passed separately in the frozen evidence run, but their package dependence and exact contract coverage remain explicit.
- Current source and current PRD conflict in material ways. Record the behavior; do not repair it here and do not soften the contract to match it.
- Manual evidence remains separate: electrofacies `2/26`, machine-learning `7/189`, workflow `0/23`, report `6/53`, office-deliverables `0/39`, LAS-export `0/2`, portfolio-performance `0/50`, processing-history `0/7`, workspace-shell `0/159`, security-integrity `0/63`, and verification-stewardship `0/24`. Automated evidence closes none of these scenarios.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record:

1. branch `codex/g1-sb-mla-adjudication`, created serially from the committed plan;
2. one clean worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, `origin/master`, merge base, and anchor reachability;
4. exactly 65 ledger rows, SB-MLA-001 through SB-MLA-065, with no gap or duplicate;
5. priorities `P0=10`, `P1=34`, `P2=17`, `P3=4`;
6. historical source states `PRESENT-OK=53`, `PRESENT-DIVERGENT=1`, `PARTIAL=2`, `ABSENT=9`;
7. all 65 live mutable evidence fields still unadjudicated or placeholder-only;
8. exactly 61 defined chapter test IDs, 73 ledger ownership references, and all 61 unique IDs owned;
9. exactly 105 parameters, including 15 absent values, 5 non-adoptable values, and 3 SB-SHR cross-references;
10. exactly 6 open items, 12 escalations, 13 refusals, 24 no-antecedent requirements, and 218 section-8 traceability rows;
11. takeover summary `740` adjudicated, `191` unadjudicated, and `477` pilot blockers before SB-MLA;
12. the manual evidence counts listed above; and
13. fresh focused evidence: `ml::tests` at `67 passed / 0 failed / 7 ignored`, `facies::tests` at `13/0/0`, `facies_tie::tests` at `7/0/0`, then the seven ignored ML tests separately at `7 passed / 0 failed`.

The only mechanically predictable post-adjudication ledger count is `805` adjudicated and `126` unadjudicated. Do not freeze as-built, release, test-class, risk, or blocker totals before every row is classified.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-mla.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/ml.rs`, `src-tauri/src/facies.rs`, `src-tauri/src/facies_tie.rs`, `src-tauri/src/python_engine.rs`, `src-tauri/src/installation.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/workflow.rs`, `src-tauri/src/reframe.rs`, `src-tauri/src/frame.rs`, `src-tauri/src/condition.rs`, `src-tauri/src/export.rs`, `src-tauri/src/report.rs`, `src-tauri/src/office.rs`, `src-tauri/src/param_sources.rs`, `src/ipc.ts`, `src/ui/mlDialog.ts`, `src/ui/moduleDialog.ts`, `src/ui/paramSources.ts`, `src/processLog.ts`, current tests, manual evidence, immutable source chapter, historical implementation records, and reachable Git history
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-MLA-NNN` section per requirement in numeric order. Every section MUST include the specified contract, current implementation, as-built status, release disposition/risk, exact automated evidence class, manual evidence, source/parameter boundary, UI/IPC/provenance surface, history/reachability, blocking decision/dependency, and next action. Separate every observable limb of a compound requirement. Name whether evidence is correctness, characterization, optional-package, structural, manual, or missing; never inflate a helper-level test, runner-source assertion, conditional early return, or returned note into a whole-contract proof.

## Requirement Evidence Map

| ID | Tests | Exact contract focus | Primary live candidates and adjudication guard |
|---|---|---|---|
| `001` | T01 | Effective values, default flags, and source IDs for every ML run | Python runner records effective parameters; native facies defaults still travel through `ModuleOutputs` without an equivalent parameter record. |
| `002` | T02 | Saved training set identity/version and named drift before apply | `TrainingRecord`, picker warnings, apply notes, and named-set drift tests. Confirm warning timing and live-store absence semantics. |
| `003` | T03 | Stable, discriminating ordered training-matrix fingerprint | `training_fingerprint`, model row, curve provenance, and the one-value-change test. Confirm feature/target/order and canonical NaN/-0 coverage. |
| `004` | T04 | Mask identity/absence and per-well removed counts | `TrainingRecord` and mask-effect test. Separate masked, interval-excluded, and incomplete counts; verify the user-visible run message. |
| `005` | T05 | Complete runtime record and pre-apply component warning | Runtime probe and `runtime_drift` helper exist, but direct apply predicts/writes before the comparison and artifact load failure returns only the Python error tail. Picker-only warning is not an invariant. |
| `006` | T06 | Every fitted/applied curve carries a resolvable producing-model ID | Saved fits and applies cite IDs; unsaved or failed-save fits still write curves with no model ID. “Not kept” is honest characterization but does not satisfy the universal MUST. |
| `007` | T07 | Refuse cited deletion; force-delete history and unresolvable mark | Backend refuses by default and read-time provenance derives deletion. Force history is frontend-only, so a direct forced command can bypass the permanent record. |
| `008` | T08 | Stored-run replay byte identity for every output and metric | Ignored all-algorithm runner test passes separately, but it drives a pooled matrix rather than replaying a stored run record; conditional/package evidence is not the whole contract. |
| `009` | T09 | Blind performance/protocol/held-out count travels with curve, explicit absence otherwise | `blind_record`, fit/apply provenance, and the curve-level both-sides test. Keep degraded-protocol behavior under 019 separate. |
| `010` | T10 | Complete ML provenance block in every report/export carrying an ML curve | `ml_provenance`, PDF/Word/LAS paths, saved-model export refusal, and deliverable test. Check all required fields and run date, not section placement alone. |
| `011` | T11 | Per-well training/apply roles and counts, including empty contributor | Curve records carry a role and result carries prediction counts, but persisted `training_json` filters zero-row wells and does not form one durable per-well two-role roster. |
| `012` | T12 | Loud artifact runtime skew and truthful substituted algorithm | Actual estimator ID is stored, but deserialisation failure does not name recorded/current runtime and differing component. Keep substitution and load failure as separate limbs. |
| `013` | T13 | Every engine refuses unclusterable wells and writes no clean empty curve | Native both-side tests and ignored Python end-to-end test. Verify module, Python fit/apply, result state, and absence of writes. |
| `014` | T14 | Every effective cluster-count change reports requested/effective/reason | Python clamp/short metrics exist; native paths refuse some cases and may not report collapsed/empty components. Do not credit refusal as reporting a completed reduced scheme. |
| `015` | T15 | Every numerical mixture guard reports component and count | Python reports low-weight components; native GMM hard-codes and silently applies `VAR_FLOOR`. A weight heuristic is not proof of variance-floor reporting. |
| `016` | T16 | Convergence versus exhaustion, iterations, and final measure for every iterative fit | Python k-means/GMM reports a subset; native GMM retains neither terminal state nor final measure. Source comments are not output. |
| `017` | T17 | Cancellation stamps incomplete log sets and reports every well | `mark_cancelled_sets`, per-well results, and both-sides persisted-set test. Confirm a completed smaller scope is distinguishable. |
| `018` | T18 | UI progress declares non-interruptible fit and cancellable write phase | Backend comments/phase text exist, but the job is registered cancellable and the UI exposes Cancel without a phase-specific capability state. T18 is explicitly characterization. |
| `019` | T19 | Collapsed group CV publishes no score | Current runner falls back to within-well shuffled KFold, sets `cv_degraded`, and still publishes the score. A warning beside the number is the forbidden behavior. |
| `020` | T20 | Subsampled metric has count, flag, and distinct name | Python silhouette basis records the cap; native silhouette internally caps its comparison reference but emits ordinary per-sample values with no sample-count/subsample designation. |
| `021` | T21 | Reject/noise class differs from missing in data and rendering | Shared reject code, output curve, count, neutral palette, discrete detection, and labels. Verify both numeric and rendered sides. |
| `022` | T22 | Ordered-feature refusal and unseen-well apply both run on default gate | Structural default test exists, but the two required behavioral tests remain `#[ignore]`. Do not treat structural shape as both behavioral contracts. |
| `023` | T23 | One k-means definition and exact cross-engine conformance | Constants are shared and a default test compares a small well-separated fixture, but the test can return early without Python and explicitly disclaims general identity required by the specified nontrivial fixture. |
| `024` | T24 | One seed/default plus identifier stability and disagreement across seeds | Shared seed constant and deterministic ordering exist; the three-seed disagreement-rate contract is absent. |
| `025` | T23,T37 | One Ward implementation with free/sorted/depth variants and provenance | Sorted-value/depth routes share `WardDp`; free ordering still uses scikit-learn's separate agglomerative implementation and variant provenance is incomplete. |
| `026` | T27 | Leaderboard and run construct identical estimators for every algorithm/configuration | Shared `ML_BUILD_MODEL` and structural/default tests are strong; verify xgboost-present/absent substitution and all non-default hyperparameters rather than one polynomial case. |
| `027` | T28 | Every score carries a distinct protocol identity | `score_protocols`, split metadata, and leaderboard displays. Do not let 019's degraded score become compliant merely because its fallback protocol is described. |
| `028` | T29 | Every learned transform fits inside each fold | Shared estimator/pipeline construction, source assertion, and shifted-well test. Confirm importance and every transform, not scaling alone. |
| `029` | T30 | Every class mnemonic identifies engine and different methods cannot collide | Native k-means still writes generic `FACIES`; Python output base remains user-editable. Disjoint native engine sets do not prove the universal naming rule. |
| `030` | T31 | Probability quantity/normalisation types and disjoint mnemonic conventions | Definitions are recorded, but posterior, membership strength, and classifier probability still share `_PROB`. Descriptive metadata does not satisfy the MUST NOT share convention. |
| `031` | T32 | Competing vendor defaults shown at point of choice and decision recorded | Source registry and backend decision note exist; the separate ML dialog shows only one `K=5` field without competing values/sources. A run-time note is not point-of-choice evidence. |
| `032` | T33 | Explicit named normalisation basis in every distance-based output provenance | Python and native choices/records exist. Verify every method and native generic-workflow provenance, not only one runner metric. |
| `033` | T34 | Stored fixed limits plus reported rescaling under data-derived basis | Python fixed basis and movement warning exist; native facies remains data-derived without fixed limits. A whole-product requirement cannot be closed by one engine. |
| `034` | T35 | Every automatic transform announced per curve, including none | Feature-transform records and run notes. Distinguish explicit user transforms from automatic family behavior and verify the no-transform side. |
| `035` | T36 | Transformed quantity has distinct mnemonic/unit and explicit back-transform | Separate log/back curves, curve-unit registry, metric-space note, and direct test. Export is not the assertion vehicle. |
| `036` | T37 | Canonical IDs, labels, vendor aliases, and hard-fail unknown | Current options use canonical-like IDs and reject unknown linkage/scaling, but the specified vendor aliases and complete enumeration/update-rule registry are absent. |
| `037` | T33,T38 | Cuddy reciprocal fuzzy combination, never product | No fuzzy predictor is registered. Keep the printed equation and regression fixture as future correctness evidence only. |
| `038` | T39 | Equal-population binning reports achieved populations | No fuzzy binning route exists. |
| `039` | T40 | Fuzzy uncertainty edge fallback and fired record | No fuzzy band route exists. |
| `040` | T38 | Visible no-default bin-count weighting and declared Cuddy deviation | No fuzzy weighting UI/parameter exists; the absent default remains absent. |
| `041` | T45 | Required total-iteration SOM decay and refusal of degenerate form | No SOM route exists; `som.total_iterations` remains absent/required and E-1 stays open. |
| `042` | T46 | Defined SOM distortion with lower-is-better and radius | No SOM route exists. |
| `043` | T41,T42 | Aggregate cluster randomness index matching cited equation and random anchor | Current code emits per-cluster `observed_mean_run*(1-p)` as a depth curve. T41 specifies one aggregate `Av/Random` value; the current direction/anchor test characterizes a different formula. |
| `044` | T43 | Per-cluster and overall geometric quality with sample count | Native path emits per-sample silhouette curves only; its test compares direction and does not exercise per-cluster/overall reporting or explicit count. |
| `045` | T44 | Restart objective distribution, retained-hit count, and caveat | Restarts retain only the best objective; no spread or hit-frequency report exists. |
| `046` | T37 | Five linkage rules/update equations, Ward default, corroborated source | Runner accepts four linkages, UI offers three, and no five-rule update/source registry exists. Do not call partial enumerations five. |
| `047` | T25,T26 | PCA loadings plus correlation-circle coordinates | Loadings and explained variance ship; correlation-circle coordinates/eigenvalue scaling are absent despite source-backed T26. |
| `048` | T25 | Fixed documented PC sign convention | First-feature non-negative convention is implemented; verify scores and loadings together and do not borrow optional all-algorithm determinism as the only proof. |
| `049` | T47,T48 | Normalised KNN average with SandiBumi-owned weight function and first-class length scale | scikit-learn KNN exists with `uniform`/`distance`; no SandiBumi-owned `w(d),h` contract or length-scale parameter exists. Section-5 absence remains binding. |
| `050` | T49 | Held-out frame excluded from neighbour search and zero self-score refused | `log_predict` skips self and has a breaking-test fixture; leaderboard feature scoring uses held-out wells. Keep dedicated vendor-style ranking absence distinct from the safety contract. |
| `051` | T50 | Raw, row-normalised, and column-normalised contingency with named axes | Backend matrices and UI modes exist; non-square/non-symmetric test pins direction and sums. |
| `052` | T51 | Acceptance threshold absent, visible, required, and persisted when chosen | Optional backend field, empty UI, explicit no-source caption, result/process record, and both-sides test. No default may be inferred. |
| `053` | T52 | Spread-multiple names its statistic; bare tolerance is ambiguous | DBSCAN records `eps_unit`, but the specified imported bare-tolerance refusal and `tolerance_sd` outlier configuration are absent. Do not redefine the test around `eps`. |
| `054` | T54 | Every resampling/snap decision recorded per curve | Main ML uses exact-depth equality and reports zero-contribution off-grid curves; facies tie records nearest-match tolerance/counts. Verify the chapter's explicit no-resampling interpretation without claiming partial overlaps are documented. |
| `055` | T53 | Categorical registry prevents every interpolation/average path | Curve-class registry, workflow declarations, reframe substitution, block/smooth/despike refusals, and both-sides tests. Probability curves remain continuous. |
| `056` | T55 | NaN/null discipline through every ML path with no opt-out | Finite-row filters exist, but the exact declared-sentinel import-to-ML/equation fixture and settings-surface search are not one whole-contract test. Never treat an undeclared finite number as null. |
| `057` | T56 | Threshold absence is distinct; missing sentinel cannot be stored as value | Shared Python parameter guard rejects known sentinels, but the specified set/import both-sides test is absent. Confirm all parameter doors, not only runner source text. |
| `058` | T57 | Tier-C capabilities named and never approximated | Static dependency/source boundary test. A name search is supporting evidence; preserve all register semantics and primary-source requirements. |
| `059` | T57 | Any design-around independently derived, labelled, cited, and provenanced | No design-around is specified or built; do not invent one during adjudication. |
| `060` | T57 | No vendor model, weight, or tile artifact reader | Static dependency/import/path/extension guard and ordinary-curve interchange boundary. Keep SandiBumi-owned joblib blobs distinct. |
| `061` | T58 | Missing component names component, inspected interpreter, exact command; native routes remain | Shared resolver is richer, but ML's hard-coded missing-Python/sklearn messages omit the exact inspected interpreter or interpreter-qualified install command; joblib failure is post-fit only. |
| `062` | T59 | No lock across subprocess/unbounded wait and release between every well | Fit path computes lock-free and writes per well; saved-model apply acquires one lock before the loop and holds it across all apply wells. |
| `063` | T60 | Every binding cap reports request, cap, and dropped work | Combination/t-SNE/silhouette caps are surfaced; native `K>12` now refuses while T60 specifies a reported clamp. Record the specified-test divergence without weakening refusal safety. |
| `064` | T61 | Model listing never materialises artifacts | Listing query omits blob and test characterizes that behavior. T61 is explicitly characterization. |
| `065` | T17,T18,T59,T60 | Portfolio progress, phase cancellation truth, bounds, and per-well outcome classes | Per-well results, cancellation/partial stamps, and some caps exist; non-interruptible phase UI, apply lock, fit bound, real-scale evidence, and complete outcome taxonomy remain open. |

## Test-Intention Routing Contract

- Route T01-T12 to persisted parameter/model/curve/run/deliverable provenance and artifact failure.
- Route T13-T22 to unclusterable behavior, effective counts, numerical guards, convergence, cancellation, phase truth, CV collapse, subsampling, reject classes, and default-gate model apply.
- Route T23-T37 to shared k-means/Ward/estimator definitions, seed, PCA, score protocols, fold transforms, names, probability types, vendor defaults, normalization, transform/unit custody, and enumerations.
- Route T38-T49 to fuzzy/SOM, cluster randomness/geometric quality/restarts, hierarchical linkage, KNN weighting, and held-out scoring.
- Route T50-T61 to contingency, absent threshold, typed tolerance, class-safe resampling, depth/null/sentinel custody, Tier-C/vendor-artifact boundaries, actionable runtime failure, lock discipline, caps, and registry listing.
- Every T01-T61 identifier MUST appear in the receipt exactly once as a routed intention even where multiple requirements own it. Ownership duplication in the immutable ledger is not duplicated evidence.
- Keep the chapter's four explicit characterization labels visible: whole T18, whole T61, the “predictions still complete” limb of T58, and the “no all-missing curve is written” limb of T13. Additional current-behavior tests may be classified characterization where their expected value is the implementation rather than an independent source.

## Execution Tasks

### Task 1: Re-freeze the accepted tree and inventories

Re-run branch/worktree/hash/count checks, the focused normal suites, the seven ignored ML tests separately, manual-evidence counts, and targeted source/history searches. Stop on branch/worktree drift, source hash drift, or a count mismatch.

### Task 2: Adjudicate SB-MLA-001 through SB-MLA-012

Classify persisted parameters, training identity/content/mask/runtime, curve model identity, deletion, determinism, blind metrics, deliverable provenance, per-well roles, and artifact/substitution failure. Test actual stored/output surfaces, not helper JSON alone.

### Task 3: Adjudicate SB-MLA-013 through SB-MLA-022

Classify unclusterable wells, effective K, numerical guards, convergence, cancellation, phase truth, CV collapse, subsampling, reject classes, and default-gate behavioral coverage. Preserve required optional-package status without converting an internal early return into an ignored-test count.

### Task 4: Adjudicate SB-MLA-023 through SB-MLA-036

Classify method identity, seed, Ward variants, estimator parity, protocol naming, fold isolation, mnemonics, probability types, vendor-default UI, normalization, transforms, units, and enumerations. Verify both native and Python engines and the separate ML dialog.

### Task 5: Adjudicate SB-MLA-037 through SB-MLA-050

Keep fuzzy/SOM absent values and source gaps absent. Compare current cluster-randomness and silhouette outputs to the exact aggregate contracts. Classify restart spread, linkage count, PCA coordinates/sign, KNN weighting/length-scale, and held-out scoring without importing vendor defaults or inventing `h`.

### Task 6: Adjudicate SB-MLA-051 through SB-MLA-065

Classify tie-in axes/thresholds, tolerance naming, resampling/class/null custody, Tier-C boundary, runtime messages, both fit and apply locking, every cap, listing behavior, and portfolio-scale honesty. Keep manual evidence and deployment evidence separate.

### Task 7: Write receipt and update ledger atomically

Create `docs/takeover/evidence/sb-mla.md`, update only the 65 mutable ledger fields, preserve the six source-owned fields/hash, and update `docs/takeover/STATUS.md`. Include exact pre/post totals, row-level status/disposition/test/risk totals, all 61 tests, 105 parameters, 6 opens, 12 escalations, 13 refusals, 24 no-antecedent requirements, all 218 traceability rows, manual counts, and the no-production-change boundary.

### Task 8: Self-review before the gate

Verify 65 ordered receipt sections, 65 non-placeholder ledger rows, allowed vocabularies, exact unique-test routing, no invented parameter, no source-owned drift, no prohibited identifiers, no unrelated file, and no claim that automated evidence closed manual/field evidence. Recompute global totals rather than estimating them.

### Task 9: Verify and commit

Run, in order:

```powershell
npx tsc --noEmit
Set-Location src-tauri
cargo check
Set-Location ..
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Require zero failures. Record exact passed/failed/ignored counts and ledger totals, then commit the execution increment locally with a message naming G1-DOM-MLA. Do not push. Continue serially to SB-PLG and SB-INS at Sol xhigh; use Sol max only for the final 931-row audit.

## Mandatory Harsh-Truth Review

- The ML subsystem is broad and heavily tested, but test volume is not whole-contract coverage. Several specified output/UI/default-gate obligations are only internal helpers, source checks, optional tests, or comments.
- A warning after prediction is not a warning before apply. A picker warning is not an invariant when the backend command can be called without the picker.
- “The fit was not saved” is truthful, but it still means a produced curve lacks the model identifier the universal contract requires.
- A degraded cross-validation score remains a score. Labelling it degraded does not satisfy a MUST NOT report.
- A per-depth or per-cluster diagnostic is not the specified aggregate. Directional tests can pass while the equation, denominator, and reported object are wrong.
- `_PROB` metadata cannot undo a shared mnemonic convention that the requirement explicitly prohibits.
- Four linkages in the runner and three in the UI are not five linkage rules with sourced update equations.
- Releasing the fit-path lock per well does not close an apply path that still holds one lock across the portfolio.
- Seven successful ignored tests prove narrow optional-package fixtures on this machine. They do not make those behaviors default-gate guarantees or close 182 unchecked ML manual scenarios.
- Gate 1 records these facts; it must not repair production behavior, soften the PRD, invent a parameter, or pre-spend Gate 2 product decisions.

## Completion Contract

The SB-MLA execution increment is complete only when all 65 rows are adjudicated, all 61 test intentions and 105 parameter rows are routed, all open/escalation/refusal/traceability custody is explicit, the source-owned hash is unchanged, manual evidence remains honestly separate, the full gate is green, and the work is committed locally. The final Gate 1 audit remains a later Sol-max increment over all 931 rows.
