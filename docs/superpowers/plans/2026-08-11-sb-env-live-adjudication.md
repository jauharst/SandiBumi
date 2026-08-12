# SB-ENV Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 58 live `SB-ENV` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing correction math, choosing a scientific parameter, reading prohibited vendor chart data, changing the PRD or implementing a missing capability.

**Architecture:** This is a documentation-only evidence pass. The PRD remains the immutable statement of intent and historical chapter status. Current source, qualifying observable acceptance tests, manual evidence and reachable Git history establish a separate live verdict. One domain receipt explains every decision; `docs/takeover/requirements.csv` carries the machine-validated summary. The pass traces the complete chain from declarative validity and preflight refusal through correction-step custody, flags and masks, conditioning, formation temperature, resistivity-temperature correction, normalization, depth-unit contracts and image-speed availability. Generic module arguments, per-run ranges, log-set provenance, undo, unit conversion and plotting are evaluated as supporting seams, never promoted into the complete ENV contract they do not demonstrably satisfy.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md` or any file under `docs/PRD_v2/**` or `docs/research_2026-08/**`.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. The plan is written on `0cca8755a620cf10c0a1e210a10852e084e721a5`; `origin/master` and the current merge base were both `29833735816d9e5be954afafd9ceb71fd856e3f0`. Reverify all four before execution. If an accepted reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, tests and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete ENV chapter, all applicable `docs/record_*.md` files and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. Chapter statuses are historical evidence, not current verdicts.
- All 58 source-owned `owned_tests` fields are populated. Preserve every mapping byte-for-byte. A chapter test ID counts as implemented only after locating its actual body, inspecting the assertion surface and expected-value source, and executing it.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid open-hole-petrophysics pilot. Original P0/P1/P2/P3 is evidence, not automatic pilot scope.
- A positive mechanism does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence and uses an independently sourced expected value. A helper, schema snapshot, internal `Result`, source-text grep, compile success or arithmetic copied from the implementation is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Do not treat the chapter's named intention as an implemented test until its executable body is found.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked. At plan time `conditioning` is `0 / 27`, `formation-temperature` is `0 / 0` and not recorded, `processing-history` is `0 / 7`, while `curve-editing` is `5 / 5`. The exercised edit capability cannot close correction, conditioning-provenance or temperature contracts by inference.
- The chapter's front matter is the corrected count authority: 58 requirements, 23 P0, 23 P1, 11 P2, one P3, 83 parameters and 70 tests. Preserve three internal chapter findings rather than smoothing them over: section 4 still says 19 P0; section 6 still says 68 tests and T07 says 31 ABSENT parameters; section 3's BHT summary is superseded by `SB-ENV-047`, which confirms `BHT` and `TD_BHT` are consumed on the BHT branch. Do not edit the PRD in this lane.
- `ArgSpec::sources_topic` and `param_sourced` are current supporting seams, not proof that every ENV parameter carries a non-empty source or explicit `ABSENT` token. Inspect every applicable argument and its rendered, persisted and refused surfaces before classifying `SB-ENV-004`.
- `resolve_param_arrays` now rejects non-finite and out-of-range supplied values and rejects named-zone overrides for well-scoped trends. That does not by itself implement declarative multi-argument validity conditions, source-bearing failures, option-enumeration refusal or four-path preflight. Keep `SB-ENV-001` through `003`, `008` and `009` atomic.
- The current environmental-correction helpers are coefficient-driven analytic approximations. Their comments explicitly describe missing QC inputs as pass-through, and existing tests exercise plausible numeric movement. Numeric movement alone does not prove correction-step custody, source admissibility, per-sample flags, uncertainty parity, chart-interface behavior or a legitimately named corrected output.
- A byte-identical `*_EC` pass-through without an observable uncorrected flag is the defect named by `SB-ENV-006`, not successful graceful degradation. Do not count `env_corrections_move_the_right_way` as proof of the missing-input contract.
- The universal workflow mask currently blanks module inputs before execution and outputs afterward. The existing `a_masked_washout_defeats_the_very_module_meant_to_repair_it` test is an explicit pinned defect/characterization, not correctness evidence for `SB-ENV-027`.
- `badhole` and `condflag` arithmetic, a 0/1 convention and mask UI are supporting seams. They do not automatically supply typed flag polarity, reason channels, signed DRHO reasons, source-bearing presets, curve-unit validation or run provenance.
- Conditioning tests that independently pin thickness windows, missing preservation, open-ended gaps, clip refusals, no self-shadowing and no-default normalization may qualify narrowly. They do not prove persistent kernel metadata, full reversible provenance, all-operation recovery or the declared chain-order contract without the corresponding observable records.
- `ftemp_grad` and `precalc` both currently produce `FTEMP`; inspect their dispatch, UI visibility, saved-chain compatibility, TVD/depth fallbacks, units and downstream consumers separately. Do not let one correct anchor test erase the duplicate-definition and compound-unit obligations.
- The accepted resistivity-temperature constant is the chapter's cited `6.77 degF` / `21.5 degC` pair from two independent implementations. Preserve the rejected `-6 degF` branch as unreachable. Do not re-derive, round or replace either value in this adjudication.
- Normalization percentiles must be exact sorted order statistics. A mapping test using a supplied reference pair may support arithmetic, but a historical generic default or one-well run does not prove an absent reference pair, common interval, reviewable per-well map, separate source identity or UI override record.
- A scientific value, correction coefficient, threshold, bit size, salinity, standoff, mudcake property, endpoint, QC band, uncertainty rule, chart axis count, ordering rule, tolerance or default is cited or absent. Never infer one from current code, a neighboring vendor, a chart shape, a local study, a plausible textbook value or model training.
- Preserve the 32 parameters specified `ABSENT`. Preserve all 29 `SHIPPED-UNCITED` findings as source gaps until each is either removed or cited; do not treat current defaults as authority. Preserve all 16 `NON-ADOPTABLE` values as verification evidence only.
- Do not read, open, transcribe, screenshot, OCR or redistribute installed vendor chart files, vendor `.neu/.ovl` resources, vendor `.itt/.itp` resources or already-digitized chart arrays. Chart-interface evidence must use synthetic tables with no vendor data present.
- `SB-ENV-T19` is explicitly a characterization of an uncited code comment. `SB-ENV-T21` is a verification comparison and does not adopt the comparison coefficient. `SB-ENV-T66` characterizes an internally inverted vendor QC configuration and cannot supply product limits. `SB-ENV-T68` is contract-only and numerically blocked until its named primary sources are held.
- Keep open items `OI-1` through `OI-8`, escalations `ESC-1` through `ESC-16`, refusals `TR-1` through `TR-4` and non-reproduction findings `RF-1` through `RF-10` visible. A live verdict may name one as its blocker; it may not settle it by implication.
- `SB-ENV-058` is P3 and outside the present petrophysics pilot unless Jauhar decides otherwise. Its current release disposition must still be adjudicated; historical P3 does not authorize silently skipping it or predetermining `DEFERRED`.
- Git reachability proves only that a change is in the accepted tree. Commit messages are locators, never correctness evidence; open the accepted source and test body.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- This planning commit authorizes no ledger verdict. Execution may create only the SB-ENV receipt, adjudicate the 58 SB-ENV ledger rows and update the dashboard after Jauhar reviews and approves this plan. It does not authorize a source fix, test addition, parameter choice, vendor-data read or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 58 SB-ENV rows:

```text
SB-ENV-001 SB-ENV-002 SB-ENV-003 SB-ENV-004 SB-ENV-005 SB-ENV-006 SB-ENV-007
SB-ENV-008 SB-ENV-009 SB-ENV-010 SB-ENV-011 SB-ENV-012 SB-ENV-013 SB-ENV-014
SB-ENV-015 SB-ENV-016 SB-ENV-017 SB-ENV-018 SB-ENV-019 SB-ENV-020 SB-ENV-021
SB-ENV-022 SB-ENV-023 SB-ENV-024 SB-ENV-025 SB-ENV-026 SB-ENV-027 SB-ENV-028
SB-ENV-029 SB-ENV-030 SB-ENV-031 SB-ENV-032 SB-ENV-033 SB-ENV-034 SB-ENV-035
SB-ENV-036 SB-ENV-037 SB-ENV-038 SB-ENV-039 SB-ENV-040 SB-ENV-041 SB-ENV-042
SB-ENV-043 SB-ENV-044 SB-ENV-045 SB-ENV-046 SB-ENV-047 SB-ENV-048 SB-ENV-049
SB-ENV-050 SB-ENV-051 SB-ENV-052 SB-ENV-053 SB-ENV-054 SB-ENV-055 SB-ENV-056
SB-ENV-057 SB-ENV-058
```

At plan time all 58 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is twenty-three P0, twenty-three P1, eleven P2 and one P3. Historical chapter status is twenty-three `ABSENT`, eight `PARTIAL`, thirteen `PRESENT-DIVERGENT`, ten `PRESENT-OK` and four `PRESENT-UNVERIFIED`. Gate 1 adjudicates all 58 because historical presence and priority do not establish present completeness, pilot reachability or proof quality.

Run this guard before and after editing:

```powershell
$envRows = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-ENV-*' }
$expected = 1..58 | ForEach-Object { 'SB-ENV-{0:D3}' -f $_ }
if ($envRows.Count -ne 58) { throw "Expected 58 SB-ENV rows, found $($envRows.Count)" }
if (@(Compare-Object $expected @($envRows.requirement_id)).Count -ne 0) {
    throw 'The live SB-ENV ID set differs from the approved plan'
}
if (@($envRows | Where-Object { [string]::IsNullOrWhiteSpace($_.owned_tests) }).Count -ne 0) {
    throw 'A source-owned SB-ENV owned_tests field became blank after planning'
}
if (@($envRows | Where-Object {
    $_.as_built_status -ne 'UNADJUDICATED' -or
    $_.release_disposition -ne 'UNDECIDED' -or
    $_.risk_class -ne 'UNCLASSIFIED' -or
    $_.test_class -ne 'MISSING-OR-UNCLASSIFIED' -or
    $_.commit_state -ne 'UNVERIFIED' -or
    $_.next_action -ne 'LIVE-ADJUDICATION'
}).Count -ne 0) { throw 'An ENV verdict changed after planning; reconcile before execution' }
```

The mechanical post-execution count is exactly 224 adjudicated and 707 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-env.md` - complete 58-row evidence receipt, including obligation-by-obligation source findings, test classification, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 58 SB-ENV rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary and next serial-domain handoff.

### Read-only governing inputs

- `AGENTS.md`, `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, `04_CORE_REQUIREMENTS.md`, `06_SEQUENCING_AND_GATES.md`, `20_envcorr-qc.md`, `91_REQUIREMENTS_INDEX.md`
- applicable `docs/record_*.md` files, especially `record_data_tools.md`, `record_fixes.md`, `record_calibration.md`, `record_core_depth.md` and `record_parallel_lanes.md`
- `docs/takeover/DECISIONS.md`, `CLAIMS.md` and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Primary source and test inspection surfaces

- `src-tauri/src/modules.rs`, `workflow.rs`, `condition.rs`, `frame.rs`, `reframe.rs`, `curve_edit.rs`, `curves.rs`, `units.rs`, `export.rs`, `chain.rs`, `param_sources.rs`
- persistence and provenance readers reached from those files; `db.rs` is read-only evidence and must not be modified
- `src/ui/moduleDialog.ts`, `workflowDialog.ts`, `wellParamsDialog.ts`, `ribbon.ts`, correction/QC/provenance panels and export/history call sites reached from the backend
- all Rust/TypeScript tests reached from the 70 chapter intentions

### Files this adjudication MUST NOT change

- every read-only governing input and source/test path above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/` or `docs/research_2026-08/`;
- `REVIEW.md` and the generated verification matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-env.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness, 58-row guard result and the three chapter count/staleness findings. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-ENV-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, chapter test intentions and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unwired or unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source and class.
- Supporting tests: helper, characterization or partial tests that help but cannot close the complete contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or UNIMPLEMENTED; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, source, parameter, legal boundary, dependency or none.
- Next action: one bounded production, test, source-recovery, field or owner-decision increment.
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

### Declarative validity, preflight, provenance and method selection (`001`-`009`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `001` | Module validity conditions are first-class serializable data, including enumerations and cross-argument conditions. | `ModuleSpec`/`ArgSpec` schema, serde round trip, saved-run representation, UI consumer and T01/T02/T03/T38. | `choices`, numeric bounds, `well_scope` and `sources_topic` exist; no complete declarative validity-condition field has yet been established. |
| `002` | The runner evaluates every declared precondition before module arithmetic through dialog, saved chain, batch and zone-override paths. | All four entry paths, common preflight call, proof body is not entered and T02/T04/T38. | `resolve_param_arrays` enforces scalar ranges and well scope in the runner, but that narrower gate cannot prove all declared conditions or all routes. |
| `003` | Every violated precondition yields a refusal or visibly flagged result containing the condition, offending value, expectation and source; never an unmarked number. | Error/result payload, UI/report propagation, stale-value exclusion and T02-T05. | Current range errors name parameter/value/range but a source-bearing condition record and flagged-result alternative need independent inspection. |
| `004` | Every ENV parameter has a source string or explicit `ABSENT`, delivered atomically with validity metadata. | Full domain argument inventory, `param_sources`, serialized spec, UI rendering, refusal payload and build gates T06/T07. | `sources_topic` exists only on selected arguments; current numeric defaults are not their own sources. Historical status is only `PARTIAL`. |
| `005` | A corrected curve persistently records the ordered list of steps actually applied and each step's parameters/status. | Correction output metadata, log-set/run persistence, restart retrieval and T08-T10. | Generic `log_sets.params_json` stores request parameters, not necessarily the applied/unavailable/disabled/refused step manifest. |
| `006` | A curve named corrected was actually corrected, or every unchanged sample is visibly marked uncorrected. | Inventory of every `*_EC` producer, missing-input cases, byte comparison, flags/manifest and T11/T12. | Current correction helpers explicitly pass through missing-QC samples; do not reinterpret silent copies as success. |
| `007` | Corrections expose a per-sample channel distinguishing corrected, partial and uncorrected states. | Output schema/type, two-thirds-coverage fixture, persistence/export and T11/T13. | Existing outputs are corrected curves without a demonstrated typed correction-state channel. |
| `008` | Validity conditions and source are visible and evaluable before the run. | Module dialog, missing-input preview, saved-chain editor and T14. | Numeric min/max controls and `sources_topic` panels are support; the complete condition/source preview must be observed. |
| `009` | Every unrecognized method selector refuses by name and never falls through or retains a prior-frame value. | Selector inventory, backend validation, all branch dispatches and T03/T15. | `ArgSpec.choices` shapes the UI, but at least one current module test explicitly pins unknown-option fallback; universal backend refusal requires full inventory. |

Manual capability candidates: `workflow`, `processing-history`, `security-integrity`, `verification-stewardship`. None is ENV field evidence by itself.

### Correction chains, source custody, charts and uncertainty (`010`-`020`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `010` | GR borehole correction models hole size, mud weight, tool position and mud type, reporting every unavailable term. | Spec/body, input withholding matrix, step manifest and T08/T16. | Current analytic helper visibly uses hole size/bit size; a plausible GR change does not establish the other three terms. |
| `011` | Neutron correction exposes all ten independently switchable ordered steps and reports an unavailable step. | Step registry, ordering contract, eleven-run fixture, manifest and T08/T09/T17. | `OI-1`, `ESC-5` and `ESC-14` remain open; do not derive canonical order from vendor implementation. |
| `012` | Neutron matrix scale is typed on the curve and validated at every consumer. | Curve metadata, all neutron consumers, unit/scale mismatch refusal and T18/T19. | `condflag` currently documents a scale assumption in prose; T19 is characterization, not a sourced numeric correctness proof. |
| `013` | Density correction includes mudcake and obtains reference diameter from supplied tool/bit inputs. | Spec/body, with/without mudcake fixture, applied-step metadata and T20. | Current helper models hole size around `HD_REF`; historical status is `PARTIAL`, not proof of mudcake or input custody. |
| `014` | Every correction coefficient is cited or absent; uncited shipped values are never adopted by a green arithmetic test. | Full 83-parameter/source inventory, `ArgSpec` defaults, reachability, T06/T07/T21. | T21 compares an uncited shipped coefficient to a house gate solely for verification; it cannot authorize either as the product value. |
| `015` | A chart-lookup interface is specified and testable independently of chart data, with interpolation, clamp/refuse and flags. | Synthetic-table interface, no-chart build, out-of-span policy and T22-T24. | Vendor charts and existing digitized arrays are prohibited evidence; `OI-2`, `ESC-12` and `ESC-13` remain open. |
| `016` | Measured formation/borehole properties have no numeric default and missing required values refuse. | Domain parameter inventory, `param_open` shape, all run routes and T07/T25. | Current correction specs ship defaults for bit size/reference and other measured quantities; retain each uncited/absent finding separately. |
| `017` | Chart baselines/intermediates are separately named single-assignment values. | Correction graph/data model, both baseline branches, mutation inventory and T26. | `ESC-5` keeps neutron baseline semantics unresolved; local variables alone do not prove persistent declared identity. |
| `018` | Conditioning and correction order is declared, checked, recorded and warns on invalid order. | Chain schema/validator, UI warning, provenance and T27/T28. | Workflow preserves entered order, but preservation alone is not a sourced validity contract; `ESC-14` is open. |
| `019` | Uncertainty is computed for exactly the applied correction steps and names that set. | Uncertainty implementation, step-set equality gate, metadata and T09/T29. | `OI-3` leaves the uncertainty-combination model open; mismatched outputs must not be emitted. |
| `020` | A correction QC surface displays original, corrected, per-step contributions and unavailable reasons in curve units. | Backend contribution outputs, UI, unit checks, restart/export behavior and T30. | Generic curve comparison or results QC is not a correction-chain decomposition. |

Manual capability candidates: `conditioning`, `log-view`, `processing-history`, `chart-overlays`. Current checked counts do not close any correction-chain contract.

### Bad-hole flags, masks, units and polarity (`021`-`030`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `021` | Bad-hole detection evaluates whichever valid inputs exist and reports which terms were available; neither-evaluable samples stay missing. | `badhole` input matrix, term status metadata and T31/T32. | Current arithmetic degrades across DRHO/caliper, but no term-availability report is yet established. |
| `022` | Bad-hole output includes a reason channel separating caliper, DRHO and both. | Output type/schema, persistence, decoding surface and T31. | One aggregate `BADHOLE` curve cannot prove reason custody; `OI-7` leaves encoding shape open. |
| `023` | The sign of a DRHO trigger is preserved and reported. | Signed reason output, positive/negative fixture and T31. | Current bad-hole arithmetic uses `abs(DRHO)` and emits one bit, so inspect whether sign survives elsewhere before verdict. |
| `024` | Bad-hole thresholds have no default; only cited named presets may populate them and provenance records the selection. | `ArgSpec`, preset registry/source, empty-run refusal, provenance and T07/T33. | Current defaults are visible; `ESC-1` is an explicit owner/source decision and may not be answered in adjudication. |
| `025` | Bit size is supplied as an input, never substituted from a default. | Spec/body, no-BS fixture, provenance and T33/T34. | Current `BS_DEF=8.5` fallback is explicit historical divergence; never rationalize it as convenience. |
| `026` | DRHO unit is declared on the curve and checked against threshold units in both directions. | Curve metadata resolver, conversion/refusal path and T35. | Canonical family/unit registries are support; mnemonic defaults and threshold labels alone are insufficient. |
| `027` | A repair module can be explicitly exempted from input and output masking, while produced masked-depth samples remain marked. | Repair declaration, both mask passes, output flag/provenance and T36/T37. | The current characterization proves the opposite for `log_predict`; `OI-5` keeps declaration granularity open. |
| `028` | Every run records the applied mask or explicit absence and the step position. | Request, log-set/provenance schema, restart/export and T27/T28. | The runner accepts `opts.MASK`; storing raw request options is not automatically a durable, reviewable mask record. |
| `029` | Conditioning flags validate their neutron scale and density-matrix preconditions before calculating. | Curve scale metadata, cross-consumer gate, mismatch fixture and T18/T19. | Prose warnings and a plausible crossover result do not implement a typed precondition. |
| `030` | Exclusion masks and diagnostic indicators have one typed polarity definition and remain distinguishable. | Flag type definition, every flag emitter, compile-time guard, metadata and T38/T39. | Current 0/1 convention and mask selector are untyped by themselves; historical status is `PRESENT-UNVERIFIED`. |

Manual capability candidates: `conditioning`, `workflow`, `processing-history`, `data-conventions`.

### Conditioning, recovery and output records (`031`-`042`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `031` | Despike displays the method- and branch-specific contamination ceiling live, including the 50% median wall. | Estimator branch state, UI calculation and T40/T69/T70. | The source-derived formulas are in the chapter; no display may hardcode one method's formula across branches. |
| `032` | The MAD consistency constant has one named cited definition and no duplicate literals. | Whole-tree constant inventory and T41. | An existing numeric constant is not self-citing; historical status is divergent. |
| `033` | Degenerate zero-MAD and too-small windows are visibly declared per sample, never silently substituted. | Despike fallback/refusal, flag channel/metadata and T42. | Current tests pin a minimum-window refusal and zero-MAD behavior; inspect the observable declaration, not only arithmetic. |
| `034` | Every window, gap and thickness parameter uses physical thickness in the project's declared depth unit. | All Condition/Frame/module declarations, runtime depth unit, resampled fixture and T43. | `condition.rs`/`frame.rs` use `depth`, while other manifests still use `m` or `m|ft`; T67 separately covers one-token declaration. |
| `035` | Every smoothing/filter operation preserves missing samples and never fills gaps. | Complete operation inventory, missing-interval fixture and T44. | Current `smooth` tests are strong support; universal closure still requires every smoothing/filter path. |
| `036` | Outlier/spurious-population culling exists as a distinct recorded operation. | Module/command registry, chain/order/provenance and T27. | Despike or clip cannot be silently relabelled as culling if their semantics differ. |
| `037` | Every removed/replaced sample is recoverable bit-exactly. | Despike/cull/clip/fill records, restore path, restart behavior and T45. | Output versioning and one edit undo are support; all-operation recovery is a universal claim. |
| `038` | Fill Gaps declares the exact boundary comparison, measures between live anchors, refuses open ends and flags inventions. | Spec/body, four boundary fixtures and T46. | Current implementation appears intentionally aligned; verify the exact named test and persistence of the flag. |
| `039` | Clip refuses with no bounds or reversed bounds and never repairs by swapping. | Both refusal paths and T47. | Current implementation has direct refusals; confirm all entry paths use it. |
| `040` | A conditioning output can never shadow its input mnemonic; refusal happens before execution and names the reason. | Output-name resolver, all Condition outputs, UI/chain paths and T48. | Current framework-level resolver/test is a strong candidate; check whole observable contract. |
| `041` | A filtered output records kernel, normalization, end behavior and gap-edge behavior. | Persisted metadata, restart/export and T49. | Module documentation is not output provenance; historical status is unverified. |
| `042` | Interactive edits have persistent provenance in addition to undo. | Edit command, process/run store, restart and T45. | `curve_edit` returns undo bytes and has a known stale-undo characterization; frontend process history must not be mistaken for durable per-curve provenance. |

Manual capability candidates: `conditioning` (`0 / 27`), `curve-editing` (`5 / 5`), `processing-history` (`0 / 7`). Keep their claims separate.

### Formation temperature and resistivity-temperature correction (`043`-`050`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `043` | Exactly one formation-temperature definition and mnemonic exists; any legacy ID delegates to it without independent arithmetic. | Registry, dispatcher, saved-chain alias, outputs and T50/T51 plus `SB-CORE-T23`. | `ftemp_grad` and `precalc` both currently emit `FTEMP`; inspect whether any delegation now exists before retaining historical divergence. |
| `044` | Formation temperature is a function of true vertical depth, with a defined fallback/refusal policy. | TVD input resolution, deviated-well fixture, both producers and T51/T52. | `precalc` consumes TVDSS with whole-curve MD fallback; `ftemp_grad` reads `DEPTH`. Neither may be called TVD-correct without the full path. |
| `045` | Geothermal gradient has one validated compound unit tied to project depth units. | Typed unit representation, metric/foot conversion/refusal, UI and T52/T53. | Unit strings such as `degC/m` and `deg/ft|m` are not a typed conversion contract. |
| `046` | Offshore temperature supports an explicit mudline/water-bottom branch and rejects mistyped branch names. | Branch enum, mudline datum input, arithmetic and T54. | No branch may be invented from depth fallback; source/owner decisions remain explicit. |
| `047` | Every declared parameter is consumed on at least one reachable branch. | Full domain spec-to-body branch-aware build gate T55. | `BHT` and `TD_BHT` are consumed under `OPT_FT=BHT`; the earlier summary saying otherwise is stale and must not be repeated. |
| `048` | The resistivity-temperature constant has one named cited definition surfaced to the user; rejected alternatives are unreachable. | Constant definitions/call sites, UI/source surface, T56/T57 and branch inventory. | Preserve the chapter's cited 6.77 degF / 21.5 degC pair; do not infer correctness merely because current arithmetic matches. |
| `049` | A superseded module ID remains saved-chain compatible by delegating to the survivor and is hidden from pickers. | Registry/dispatcher alias, saved-chain fixture, UI inventory and T58. | A hidden picker item without backend delegation, or delegation without hidden UI, is only partial. |
| `050` | Depth-trend parameters refuse named-zone overrides while compartment parameters accept them. | `well_scope`, zone resolver, temperature/pressure fixture and T59. | Current runner has a named-zone refusal and pressure-gradient control test; verify both reporting surfaces and source reachability. |

Manual capability candidate: `formation-temperature` is `0 / 0` and `Not recorded`; no runtime field claim may be inferred.

### Normalization, QC limits, depth tokens and image-speed availability (`051`-`058`)

| ID | Contract boundary | Required evidence | Known live candidate or caveat |
|---|---|---|---|
| `051` | Percentiles are exact sorted order statistics, never histogram-bin means. | Implementation, dense-tail fixture, all normalization call sites and T60. | Current normalize sorts before `distribution::percentile`; one helper path does not prove every percentile consumer. |
| `052` | The normalization reference pair ships absent and missing input refuses. | Both Normalize specs, UI defaults, saved-chain behavior and T07/T61. | Generic `condition::normalize` uses open refs, while `gr_normalize` visibly carries numeric defaults; inspect both rather than choosing the favorable one. |
| `053` | Per-well normalization inputs, computed percentiles, map, interval and overrides are recorded and reviewable. | Multi-well run record, UI preview/acceptance, persistence and T62. | A resulting curve and raw request parameters do not prove the review/override contract. |
| `054` | Multi-well normalization uses a declared common interval and warns when wells differ. | Interval model, per-well comparison, warning persistence and T62/T63. | Per-well percentile computation is support but can normalize incomparable intervals silently. |
| `055` | Normalization references and Vsh endpoints are separately named and separately sourced. | Parameter/source keys, dependency inventory, endpoint-change control and T64. | Numeric equality or shared defaults is not semantic separation; historical status is divergent. |
| `056` | Log-QC limits ship absent and one precedence rule governs user/extreme bands. | Limit registry/UI, missing-limit refusal, bracketing validation and T65/T66. | T66 vendor numbers are non-adoptable characterization only; `OI-6` and `ESC-3` remain open. |
| `057` | Every project-depth length parameter uses one token and one validation/conversion path. | Whole-module declaration inventory, UI, runner and T43/T67. | Current tokens include `depth`, `m` and `m|ft`; generic `DepthUnit` conversion alone does not close declaration consistency. |
| `058` | Borehole-image speed correction is independently derived, emits displacement and is reversible. | Lawful primary sources, implementation path, output/provenance and T68. | P3; `ESC-6`/`ESC-7` and `TR-4` remain. T68 has no numeric oracle until the named primary sources are held. |

Manual capability candidates: `conditioning`, `data-conventions`, `image-data`, `processing-history`. None closes the source/legal gate.

---

## Chapter Test-Intent Routing Guard

Route every corrected chapter test intention exactly once as a primary owner. Cross-support never counts as a second implementation.

| Test | Primary row | Contract pinned / cross-support |
|---|---|---|
| `T01` | `001` | validity data survives serialization |
| `T02` | `002` | preflight prevents module entry; also supports `001`/`003` |
| `T03` | `009` | unknown selector refuses; also supports `001`/`003` |
| `T04` | `002` | identical preflight across four launch paths; also supports `003` |
| `T05` | `003` | refusal payload includes source-bearing condition |
| `T06` | `004` | domain-wide source-or-ABSENT build gate |
| `T07` | `016` | corrected set is 32 ABSENT parameters; also supports `004`, `014`, `024`, `025`, `052`, `056` |
| `T08` | `005` | applied-step manifest; also supports `010`/`011` |
| `T09` | `019` | uncertainty step set equals applied step set; also supports `005`/`011` |
| `T10` | `005` | applied-step manifest survives restart |
| `T11` | `006` | no-caliper corrected-name refusal/flag; also supports `007` |
| `T12` | `006` | every corrected-output missing-input case |
| `T13` | `007` | per-sample corrected/partial/uncorrected states |
| `T14` | `008` | pre-run visible condition and source |
| `T15` | `009` | no stale prior-frame value after unknown selector |
| `T16` | `010` | GR term-withholding matrix |
| `T17` | `011` | ten independently reported neutron steps |
| `T18` | `012` | scale mismatch refusal; also supports `029` |
| `T19` | `012` | explicit CHARACTERIZATION of uncited apparent offset; also supports `029` |
| `T20` | `013` | density mudcake/reference-diameter behavior |
| `T21` | `014` | verification ratio only; does not adopt comparison coefficient |
| `T22` | `015` | synthetic in-span interpolation |
| `T23` | `015` | synthetic clamp/refuse out-of-span policies |
| `T24` | `015` | chart interface passes with zero chart data |
| `T25` | `016` | absent measured property refuses |
| `T26` | `017` | named baseline single assignment |
| `T27` | `018` | invalid order warning; also supports `028`/`036` |
| `T28` | `028` | mask and chain position in provenance; also supports `018` |
| `T29` | `019` | uncertainty metadata names covered steps |
| `T30` | `020` | correction-chain QC decomposition |
| `T31` | `022` | reason channel and DRHO sign; also supports `021`/`023` |
| `T32` | `021` | degraded bad-hole input matrix |
| `T33` | `024` | absent thresholds and named cited preset; also supports `025` |
| `T34` | `025` | no bit-size substitution |
| `T35` | `026` | DRHO/threshold unit mismatch both ways |
| `T36` | `027` | masked repair matches unmasked control |
| `T37` | `027` | both mask passes exempted and marked |
| `T38` | `030` | one compile-time polarity/type definition; also supports `001`/`002` |
| `T39` | `030` | exclusion versus diagnostic flag types |
| `T40` | `031` | zero-MAD Hampel ceiling and 50% wall |
| `T41` | `032` | one cited MAD consistency constant |
| `T42` | `033` | zero-MAD and too-small-window declarations |
| `T43` | `034` | thickness invariant under resampling; also supports `057` |
| `T44` | `035` | every smoothing/filter path preserves gaps |
| `T45` | `037` | bit-exact restore across operation family; also supports `042` |
| `T46` | `038` | fill-gap boundary/open-end/flag contract |
| `T47` | `039` | clip refuses no/reversed bounds |
| `T48` | `040` | input-name shadow refusal before run |
| `T49` | `041` | persistent kernel/normalization/end/gap-edge record |
| `T50` | `043` | exactly one temperature producer/delegator |
| `T51` | `043` | duplicate paths converge; also supports `044` |
| `T52` | `045` | metric/foot compound-unit equivalence; also supports `044` |
| `T53` | `045` | mismatched gradient/depth unit refuses |
| `T54` | `046` | mudline branch and unknown-branch refusal; also supports `009` |
| `T55` | `047` | branch-aware declared-parameter usage build gate |
| `T56` | `048` | cited F/C constant equivalence |
| `T57` | `048` | one reachable constant and rejected branch unreachable; also supports `009` |
| `T58` | `049` | saved-chain legacy delegation plus picker hiding |
| `T59` | `050` | trend refuses zone; compartment accepts |
| `T60` | `051` | exact order statistic versus binning |
| `T61` | `052` | missing normalization reference refuses |
| `T62` | `053` | reviewable per-well normalization record; also supports `054` |
| `T63` | `054` | incomparable intervals warn |
| `T64` | `055` | Vsh endpoint change cannot move normalization |
| `T65` | `056` | no QC limits refuses |
| `T66` | `056` | explicit CHARACTERIZATION of inverted non-adoptable bands |
| `T67` | `057` | one project-depth token across all declarations |
| `T68` | `058` | contract-only reversible speed correction; source-blocked numeric oracle |
| `T69` | `031` | positive-MAD Hampel branch shows 50% |
| `T70` | `031` | mean-sigma formula remains estimator-specific |

Execution MUST mechanically verify that `T01` through `T70` each appears once in the primary column, with no gaps or duplicates. The stale section-6 sentence and stale T07 count are findings, not alternate scopes.

---

## Task 1: Freeze Evidence and Create the 58-Row Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-env.md`
- Read: governing inputs and source/test surfaces above

- [ ] Reverify branch, HEAD, accepted anchor, `origin/master`, merge base, worktree cleanliness and anchor reachability.
- [ ] Run the 58-row guard and snapshot every source-owned field before editing.
- [ ] Recount priorities, historical statuses, 83 parameter dispositions and 70 test intentions directly from the live chapter/ledger; record the three chapter staleness findings without modifying the PRD.
- [ ] Read the complete current source for every discovered candidate and record symbol-level locations rather than relying on chapter line numbers.
- [ ] Create all 58 receipt headings in numeric order with every schema field present and no verdict placeholders silently omitted.
- [ ] Record the prohibited-chart/data boundary before any chart search. Directory inventory may establish absence/presence; installed vendor chart contents must never be opened.

## Task 2: Adjudicate Validity and Environmental-Correction Contracts

**Rows:** `SB-ENV-001` through `SB-ENV-020`

- [ ] Trace `ModuleSpec`/`ArgSpec` fields through serialization, module dialog, saved chains, batch runs, zone parameters and backend dispatch. Separate ranges, options, cross-argument conditions, sources and preflight timing.
- [ ] Inventory every ENV parameter and classify source-bearing, explicit ABSENT, SHIPPED-UNCITED or NON-ADOPTABLE exactly as the chapter records; do not infer a source from comments or tests.
- [ ] Trace every `*_EC` output from spec through body, step/result metadata, persistence, restart, UI/QC and export. Exercise missing-input behavior as well as nominal arithmetic.
- [ ] Inventory GR, neutron and density correction obligations term by term; a partial analytic helper may close only the obligations it observably satisfies.
- [ ] Inspect chart-interface candidates with synthetic tables only. Confirm no test depends on chart arrays or vendor resources.
- [ ] Trace correction-order, baseline and uncertainty contracts without settling OI-1/OI-3/ESC-5/ESC-14.
- [ ] Locate and run actual candidates for T01-T30, classifying T19 and T21 exactly as the chapter requires.
- [ ] Write twenty complete receipt verdicts.

## Task 3: Adjudicate Flags, Masks and Conditioning

**Rows:** `SB-ENV-021` through `SB-ENV-042`

- [ ] Trace `badhole` and `condflag` from input availability and units through reason/sign outputs, declared flag type, mask selection, persistence, UI and export.
- [ ] Inspect threshold/bit-size defaults against the ABSENT/source contract; leave ESC-1 open rather than choosing a preset.
- [ ] Trace the universal mask twice: input blanking before module execution and output blanking afterward. Treat the current masked-repair test as characterization until the specified exemption exists.
- [ ] Inventory every Condition/Frame thickness parameter and every project-depth token. Keep physical-thickness behavior distinct from declaration-token consistency.
- [ ] Inspect all smoothing/filtering paths for missing preservation; all removal/replacement paths for bit-exact recovery; and all output paths for kernel/operation/provenance persistence.
- [ ] Distinguish undo payloads, frontend process history, log-set versioning and durable per-curve provenance. Do not let one stand in for another.
- [ ] Locate and run actual candidates for T31-T49 plus T69/T70, classifying pinned defects and source-only grep gates honestly.
- [ ] Write twenty-two complete receipt verdicts.

## Task 4: Adjudicate Temperature, Normalization, QC Limits and Image Availability

**Rows:** `SB-ENV-043` through `SB-ENV-058`

- [ ] Inventory every `FTEMP` producer, dispatcher route, picker entry, saved-chain ID and downstream consumer. Trace TVD/depth reference and compound units separately.
- [ ] Confirm T55 with a branch-aware spec-to-body inventory so `BHT`/`TD_BHT` are not falsely reported unused.
- [ ] Trace the accepted Rw temperature constant and every alternative path; never modify `multimin2.rs` or infer a new constant.
- [ ] Verify legacy-module delegation and trend-versus-compartment zone scope across backend and UI/saved chains.
- [ ] Trace every normalization implementation through percentile selection, absent reference behavior, multi-well interval, per-well review/override, Vsh endpoint independence, persistence and export.
- [ ] Preserve the QC-limit non-adoption and unresolved band semantics. Do not turn T66's inverted vendor values into SandiBumi defaults.
- [ ] Confirm every project-depth token in the full module inventory, not only the cited examples.
- [ ] For image speed correction, record primary-source/legal availability and T68's contract-only state; do not inspect protected artifacts or invent a numeric oracle.
- [ ] Locate and run actual candidates for T50-T68.
- [ ] Write sixteen complete receipt verdicts.

## Task 5: Classify Tests, Manual Evidence and Reachable History

- [ ] For every `T01`-`T70`, record `CORRECTNESS`, `CHARACTERIZATION`, supporting-only, missing proof, build-gate-only or blocked by a named source/parameter/legal decision.
- [ ] For every correctness expected value, name the chapter/public source or independently show the arithmetic. Never use implementation output as its own oracle.
- [ ] Run every discovered candidate test by exact name. Record command, result and assertion surface; compilation or source grep alone is not a behavioral pass.
- [ ] For T06/T07/T55, prove the inventory is complete and branch-aware. A build gate that sees only one branch or a hand-selected reader list is not a whole-domain gate.
- [ ] Confirm all consequential negative findings in expected modules, UI, tests and `git log --all -S/-G` reachable history.
- [ ] Record manual evidence only from the generated capability matrix and checked review scenarios. Do not add or check a scenario in this lane.
- [ ] Ensure receipt text contains no client, asset, field, block, basin, operator, well or project name; describe the physical condition and source class instead.

## Task 6: Update the Ledger Atomically and Self-Review All 58 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-env.md`

- [ ] Prepare all 58 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-ENV rows and all source-owned fields.
- [ ] Enforce that no ENV row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 58 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every characterization is labelled, every block names its dependency and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 224 adjudicated, 707 remaining.

## Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 58-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, hard source/parameter/legal blocks and `707/931` rows remaining.
- [ ] Recommend `G1-DOM-CLY-P` as the next serial planning increment because clay-volume interpretation consumes corrected/conditioned inputs; do not prepare or execute it automatically.

## Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, the PRD-audit check and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff and stage only `docs/takeover/evidence/sb-env.md`, `docs/takeover/requirements.csv` and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-ENV adjudicate 58 SB-ENV requirements`; do not push, merge or begin a production fix.

---

## Plan Self-Review Before Execution

- [ ] Exactly 58 live SB-ENV IDs are covered once; no row can be silently skipped.
- [ ] All twenty-three original P0s, twenty-three P1s, eleven P2s and one P3 are adjudicated without treating old priority as pilot policy.
- [ ] All 70 chapter test intentions are routed once, and all 58 source-owned test mappings remain unchanged.
- [ ] The plan changes no production behavior, test, PRD, research dossier, manual verification record or chart data.
- [ ] The chapter's stale 19-P0, 68-test, 31-ABSENT and BHT-summary statements remain findings rather than alternate scopes.
- [ ] Historical `PRESENT-OK` or `PRESENT-DIVERGENT` is never copied into a live verdict without current source, test and reachability evidence.
- [ ] Ranges, enumerated choices, cross-argument validity, source strings, preflight timing and observable failure payloads remain separate obligations.
- [ ] A correction's plausible numeric movement cannot substitute for applied-step custody, missing-input flags, source admissibility or uncertainty parity.
- [ ] Silent `*_EC` pass-through remains a defect, not graceful degradation.
- [ ] The masked-repair characterization remains a defect alarm; it is never counted as correctness for the required exemption.
- [ ] Flag arithmetic, flag type, reason channel, sign custody, unit validation and provenance remain separate obligations.
- [ ] Thickness behavior and one-token declaration consistency remain separate obligations.
- [ ] Undo, versioning, process history and per-curve provenance remain separate evidence surfaces.
- [ ] Every `FTEMP` producer and consumer is inventoried; one correct arithmetic test cannot hide duplicate definitions or wrong depth/unit semantics.
- [ ] The Rw temperature constant remains exactly the cited chapter pair; rejected alternatives remain unreachable and no new value is introduced.
- [ ] Exact percentiles, absent references, common intervals, reviewable per-well maps and Vsh-endpoint independence are checked separately.
- [ ] All 32 ABSENT parameters remain absent unless a cited source already in the chapter authorizes the value; all 16 NON-ADOPTABLE values remain verification-only.
- [ ] No vendor chart, lookup row, digitized array, binary resource, raster coordinate or client calibration is read or promoted.
- [ ] T19 and T66 remain characterizations; T21 remains a verification comparison; T68 remains contract-only/source-blocked.
- [ ] OI-1 through OI-8, ESC-1 through ESC-16, TR-1 through TR-4 and RF-1 through RF-10 remain visible wherever applicable.
- [ ] Manual evidence, automated tests, accepted Git reachability, source admissibility and pilot field evidence remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 224 adjudicated / 707 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution starts only after Jauhar explicitly approves this plan.
