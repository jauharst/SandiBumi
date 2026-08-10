# SB-GEO Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 52 live `SB-GEO` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without implementing geomechanics, choosing a scientific parameter, copying a vendor table, broadening a dynamic elastic output or changing the PRD.

**Architecture:** This is a documentation-only evidence pass. The PRD remains the immutable statement of intent and historical chapter status. Current source, qualifying observable acceptance tests, manual evidence and reachable Git history establish a separate live verdict. One domain receipt explains every decision; `docs/takeover/requirements.csv` carries the machine-validated summary. The pass traces the complete chain from independently gated domain units through typed depth/datum, density and vertical stress, normal and abnormal pressure, fracture pressure, elastic/static calibration, horizontal stress, inclined stress frames, failure criteria, stability validation, source and applicability gates, provenance and export. Generic storage, unit conversion, plotting, formation-pressure utilities and the unconventional brittleness module are evaluated as supporting seams, never promoted into a GEO capability they do not implement.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md` or any file under `docs/PRD_v2/**` or `docs/research_2026-08/**`.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. The branch is stacked on the reviewed SB-PLT adjudication commit `57b411ec1534901775cd56913fb138900c9f3908`; `origin/master` was frozen at `29833735816d9e5be954afafd9ceb71fd856e3f0` when this plan was written. Reverify all three before execution. If an accepted reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, tests and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete GEO chapter, both complete GEO evidence dossiers, the applicable `docs/record_*.md` files and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. Chapter statuses are historical evidence, not current verdicts.
- All 52 source-owned `owned_tests` fields are populated. Preserve every mapping byte-for-byte. A chapter test ID counts as implemented only after locating its actual body, inspecting the assertion surface and expected-value source, and executing it.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid pilot. Original P0/P1/P2 is evidence, not automatic pilot scope.
- A positive mechanism does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list, or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence, and uses an independently sourced expected value. A helper, schema snapshot, internal `Result`, source-text grep or compile success is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Do not treat the chapter's named intention as an implemented test until its executable body is found.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked. The current `unconventional` capability is `0 / 4` checked and there is no GEO capability row, so no GEO contract is field-verified by inference.
- There is no registered overburden, pore-pressure, fracture-pressure, horizontal-stress or wellbore-stability module in the accepted module catalog or dispatcher. Confirm that negative in source and reachable history before recording each dependent verdict; one global grep is not enough.
- `precalc`'s generic linear `FPRESS = PSURF + PGRAD*TVDSS` utility is not normal-pressure, pore-pressure or PPFG implementation evidence. Its fallback, parameter scope and unit contract differ from the GEO datum/method/provenance contract.
- Generic `DepthUnit` conversion does not type a datum. Generic scalar-curve carriage does not type pressure, gradient, imported/computed/interpreted identity or a stress tensor. Generic export provenance does not supply the complete GEO run record.
- The unconventional `brittleness` module emits dynamic `YME` in Mpsi and `PR`, and rejects invalid shear/negative Poisson ratio. It does not supply static calibration, a stress consumer, a directional frame or GEO provenance. Keep `dynamic` visible and never treat identity as a dynamic-to-static transform.
- `elastic_bi_from_known_slowness` is at most support for `SB-GEO-T45`: the numeric output alone does not prove persisted `dynamic`, unit and source-run tags. `elastic_rejects_negative_poisson` is at most support for `SB-GEO-T46`: it does not exercise a downstream static consumer.
- Missing sample data remains `f32::NAN`, never `Option<f32>` in numeric arrays. Arrays cross IPC as bytemuck bytes, never JSON numeric arrays.
- The frontend never sends SQL for writes. Any future calibration, parameter, interpretation or edit path must be whitelisted, undoable and provenance-complete. This adjudication cannot invent a write path to satisfy a GEO contract.
- Preserve the deliberately PK-less `computed_curves` table and its one-writer DELETE-then-append discipline. Never recommend or implement a primary key, `ON CONFLICT`, upsert or duplicate-tolerant writer.
- Python remains a subprocess and every runner reads `sys.stdin.buffer`; this planning/adjudication lane does not add or alter runners.
- A scientific value, unit limit, breakpoint, tolerance, cutoff, endpoint, validity range or default is cited or absent. Never infer one from code, a neighboring vendor, a chart shape, a local study, a plausible textbook value or model training.
- Preserve every parameter deliberately shipped absent, including `RHO_WATER`, `GRAD_NORMAL`, `TRAUGOTT_CPG`, `SI_NCT_ABCD`, `EATON_N_SI`, `BOWERS_A`, `BOWERS_B`, `BOWERS_U`, `BOWERS_SIGMA_MAX`, `EHP_CONSTANTS`, `ARCHIE_RW100`, `ARCHIE_OC`, `PR_B_BREAK`, `MK_OBG_TOL`, `K_FIXED`, `T0`, `SIGMA_TEC`, `EPS_HMIN`, `EPS_HMAX`, `YME_D2S`, `PR_D2S`, `SHMAX_SHMIN` and `FAULT_MU`.
- Preserve `PR_POLY_COEFFS`, `MK_COEFFS` and `DAINES_NU_TABLE` as non-adoptable verification evidence until their named primary literature is recovered and independently re-derived. Never transcribe vendor rows, raster coordinates, binary constants or local delivered-study presets.
- `SB-GEO-T35` cannot be implemented with a numeric premise tolerance while `MK_OBG_TOL` is absent. Record the missing cited tolerance; do not choose one merely to make the test executable.
- `SB-GEO-T39` cannot be closed from a vendor lookup file. Its 30-row expectation requires the primary Daines source and source-by-row reconstruction.
- `SB-GEO-T71` is a meta-contract. It closes only when every numeric example is executed through released calculators at the stated tolerance; copying chapter arithmetic into a separate test helper is not proof.
- Git reachability proves only that a change is in the accepted tree. Commit messages are locators, never correctness evidence; open the accepted source and test body.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- This planning commit authorizes no ledger verdict. Execution may create only the SB-GEO receipt, adjudicate the 52 SB-GEO ledger rows and update the dashboard after Jauhar reviews and approves this plan. It does not authorize a source fix, test addition, primary-source reconstruction, parameter choice or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 52 SB-GEO rows:

```text
SB-GEO-001 SB-GEO-002 SB-GEO-003 SB-GEO-004 SB-GEO-005 SB-GEO-006 SB-GEO-007
SB-GEO-008 SB-GEO-009 SB-GEO-010 SB-GEO-011 SB-GEO-012 SB-GEO-013 SB-GEO-014
SB-GEO-015 SB-GEO-016 SB-GEO-017 SB-GEO-018 SB-GEO-019 SB-GEO-020 SB-GEO-021
SB-GEO-022 SB-GEO-023 SB-GEO-024 SB-GEO-025 SB-GEO-026 SB-GEO-027 SB-GEO-028
SB-GEO-029 SB-GEO-030 SB-GEO-031 SB-GEO-032 SB-GEO-033 SB-GEO-034 SB-GEO-035
SB-GEO-036 SB-GEO-037 SB-GEO-038 SB-GEO-039 SB-GEO-040 SB-GEO-041 SB-GEO-042
SB-GEO-043 SB-GEO-044 SB-GEO-045 SB-GEO-046 SB-GEO-047 SB-GEO-048 SB-GEO-049
SB-GEO-050 SB-GEO-051 SB-GEO-052
```

At plan time all 52 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is thirty-three P0, seventeen P1 and two P2. Historical chapter status is fifty `ABSENT` and two `PARTIAL` (`029`, `031`). Gate 1 adjudicates all 52 because a lower historical priority can still control the integrity or availability of a paid-pilot calculation chain.

Run this guard before and after editing:

```powershell
$geo = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-GEO-*' }
$expected = 1..52 | ForEach-Object { 'SB-GEO-{0:D3}' -f $_ }
if ($geo.Count -ne 52) { throw "Expected 52 SB-GEO rows, found $($geo.Count)" }
if (@(Compare-Object $expected @($geo.requirement_id)).Count -ne 0) {
    throw 'The live SB-GEO ID set differs from the approved plan'
}
if (@($geo | Where-Object { [string]::IsNullOrWhiteSpace($_.owned_tests) }).Count -ne 0) {
    throw 'A source-owned SB-GEO owned_tests field became blank after planning'
}
if (@($geo | Where-Object {
    $_.as_built_status -ne 'UNADJUDICATED' -or
    $_.release_disposition -ne 'UNDECIDED' -or
    $_.risk_class -ne 'UNCLASSIFIED' -or
    $_.test_class -ne 'MISSING-OR-UNCLASSIFIED' -or
    $_.commit_state -ne 'UNVERIFIED' -or
    $_.next_action -ne 'LIVE-ADJUDICATION'
}).Count -ne 0) { throw 'A GEO verdict changed after planning; reconcile before execution' }
```

The mechanical post-execution count is exactly 218 adjudicated and 713 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-geo.md` - complete 52-row evidence receipt, including obligation-by-obligation source findings, test classification, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 52 SB-GEO rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary and next serial-domain handoff.

### Read-only governing inputs

- `AGENTS.md`, `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, `04_CORE_REQUIREMENTS.md`, `06_SEQUENCING_AND_GATES.md`, `18_geomech-ppfg.md`, `91_REQUIREMENTS_INDEX.md`
- `docs/research_2026-08/cross_tool/geomech-ppfg.md` and `geomech-ppfg_critique.md`, read completely but never amended
- applicable `docs/record_*.md` files, especially `record_data_tools.md`, `record_fixes.md`, `record_calibration.md`, `record_core_depth.md` and `record_parallel_lanes.md`
- `docs/takeover/DECISIONS.md`, `CLAIMS.md` and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Primary source and test inspection surfaces

- `src-tauri/src/modules.rs`, `lib.rs`, `unconventional.rs`, `units.rs`, `curves.rs`
- `src-tauri/src/workflow.rs`, `export.rs`, `ingest.rs`, `reframe.rs`, `registration.rs`
- frontend module/workflow/parameter, catalog, plot and export call sites reached from those backends
- all Rust/TypeScript tests reached from the 73 chapter intentions

### Files this adjudication MUST NOT change

- every read-only governing input and source/test path above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/` or `docs/research_2026-08/`;
- `REVIEW.md` and the generated verification matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-geo.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness and 52-row guard result. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-GEO-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, chapter test intentions and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unwired or unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source and class.
- Supporting tests: helper or partial tests that help but cannot close the complete contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or UNIMPLEMENTED; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, source, dependency or none.
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

### Domain units, datum, density and applicability (`001`-`008`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `001` | Six independently versioned gates: vertical stress, pore pressure, fracture pressure, horizontal stress, elastic/static calibration and stability. | Module/domain registry, dispatcher, UI availability, per-gate version state and T01/T02. | Current module registry has no GEO family; the `brittleness` module is an unconventional calculation, not one of six GEO gates. |
| `002` | Every depth carries type and datum, and incompatible references refuse before calculation. | Depth/datum types, survey/reference conversion, composition checks and T03/T04. | `DepthUnit` converts ft/m but does not type sea level, rig floor, ground or mudline. |
| `003` | Vertical stress integrates from the physical pressure/density anchor, including the first interval and canonical units. | Density integration code, anchor input, gap policy, exact unit conversions and T05-T07. | No vertical-stress calculator was found; generic cumulative/depth helpers cannot close the pressure integral. |
| `004` | Measured and synthetic density remain distinguishable with retained source identities and unresolved gaps. | Combiner schema, masks, provenance, gap refusal and T08/T09. | Generic curve provenance does not prove a GEO density-combiner or interval-aware stress refusal. |
| `005` | Synthetic density method is selected explicitly or fitted against measured density with recorded evidence. | Method registry, fit interval, objective, residuals, selection record and T10/T11. | No GEO density-synthesis selector or fit record was found. Local constants cannot be promoted to defaults. |
| `006` | Every correlation enforces source, geography, depth, lithology/fluid and other declared applicability metadata. | Correlation registration schema, runtime gate, persisted breach and T12/T13. | Generic module parameters have no complete GEO applicability registry. |
| `007` | Vendor tables, coefficient rows and opaque artifacts are never implementation truth. | Packaged-resource scan, method loader, provenance gate and T14. | Absence of packaged rows is a safety property, not evidence that the methods exist. |
| `008` | One shared run water density is resolved with a labelled correlation-embedded exception. | Typed run parameter resolver, conflict gate, embedded-value metadata and T15/T16. | `RHO_WATER` deliberately ships absent; unrelated fluid-density defaults do not satisfy this contract. |

Manual capability candidates: `data-conventions`, `workflow`, `processing-history`, `verification-stewardship`. None is GEO field evidence by itself.

### Normal pressure, pore pressure and output policy (`009`-`017`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `009` | Normal pressure is anchored at the water/formation boundary with a sourced run gradient. | Anchor/reference schema, typed gradient, calculation, refusal and T17/T18. | `GRAD_NORMAL` is absent; `precalc`'s generic intercept/gradient utility is not this method. |
| `010` | Terzaghi effective stress uses explicit shared Biot alpha. | Total/effective types, shared alpha resolver and T19. | A scalar subtraction helper without state typing and one run value cannot close it. |
| `011` | Eaton sonic, resistivity, velocity and corrected-drilling-exponent forms remain distinct. | Four registrations/equations, ratio direction, method-specific inputs and T20/T21. | No Eaton implementation was found; one formula cannot stand in for all four. |
| `012` | Selected NCT/trend, raw Pp, final Pp and every filter/clamp mask are readable outputs. | Output schema, provenance, UI/export and T22. | Generic curve output and processing history do not expose this method record. |
| `013` | Bowers emits only after coefficients, interval and residual evidence are calibrated. | Calibration record, blocking params, crossplot/residuals and T23/T24. | All run-specific Bowers coefficients deliberately ship absent. |
| `014` | Bowers unloading is algebraically consistent; `U=1` reduces to loading and invalid `U` refuses. | Loading/unloading equations, property tests and T25/T26. | No implementation exists; chapter arithmetic must not be copied into a second ungoverned helper. |
| `015` | A method whose readable primary equation is unavailable remains individually disabled. | Method registry, source resolver, availability result and T27. | Inventory names from manuals or rasters are not equations. |
| `016` | One explicit default-Off pressure-limit policy governs every input path and discloses masks/counts. | Raw/final curves, lower/upper policies, every method route and T28/T29. | No GEO pressure pipeline exists; generic clipping is not pressure limiting. |
| `017` | Pressure and pressure gradient are separate typed curves with a declared datum/depth. | Typed outputs, conversion, metadata, plot/export and T30. | Mnemonics and unit strings alone do not provide distinct physical types. |

Manual capability candidates: `formation-temperature`, `workflow`, `log-view`, `report`, `processing-history`. Generic pressure display does not close PPFG.

### Fracture pressure, coefficient families and envelopes (`018`-`026`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `018` | Generalized fracture pressure is alpha-aware and includes explicit K, tectonic stress and tensile terms. | Governed equation, shared alpha, typed inputs/outputs and T31/T32. | `K_FIXED`, `SIGMA_TEC` and `T0` deliberately ship absent. |
| `019` | Constant-K, Poisson, Daines and other K relationships remain explicit selectable families. | Separate method IDs, input requirements, no fallback and T33. | A hidden coefficient switch or silent default is divergent even if arithmetic is plausible. |
| `020` | Matthews-Kelly coefficients come from independent primary-paper reconstruction. | Paper receipt, derivation, source-by-coefficient record, monotonicity and T14/T34. | `MK_COEFFS` is non-adoptable vendor evidence until re-derived from the 1967 paper. |
| `021` | Matthews-Kelly's overburden premise is measured and warned against a sourced tolerance. | Mean overburden calculation, tolerance source, warning record and T35. | `MK_OBG_TOL` is absent, so no numerical test or enabled method may invent it. |
| `022` | Every regional/source-geography restriction is enforced and persisted. | Source scope model, declared run scope, policy and T12/T36. | A UI label without runtime refusal/warning and stored breach is insufficient. |
| `023` | Every Poisson-polynomial breakpoint and branch inclusivity is exposed and conflicts refuse. | Primary-derived coefficient/breakpoint schema, half-open rule and T37/T38. | `PR_B_BREAK` is absent and `PR_POLY_COEFFS` is non-adoptable. |
| `024` | The Daines table is rebuilt row by row from primary literature. | Daines source, 10-family/30-row derivation, citations and T14/T39. | Vendor table bytes are prohibited implementation truth. |
| `025` | Published fracture-pressure bounds are labelled and plotted as an envelope. | Two governed equations, pressure outputs, plot integration/provenance and T40. | Correct helper arithmetic alone does not prove the visible labelled envelope. |
| `026` | Fracture pressure is capped by overburden only under explicit default-Off policy. | Raw/final FP, policy, mask/count and T41. | Silent unconditional min or display clipping would violate the contract. |

Manual capability candidates: `chart-overlays`, `log-view`, `report`, `verification-stewardship`. No unchecked plot is evidence of scientific correctness.

### Dynamic/static properties, horizontal stress and frames (`027`-`035`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `027` | Minimum and maximum horizontal stresses are computed explicitly with separate tectonic contributions. | Governed equations, direction types, run inputs and T42/T43. | No horizontal-stress implementation was found; `SHMAX_SHMIN` and both strains are absent. |
| `028` | Tectonic strains are named by minimum/maximum horizontal-stress direction and require a mapping frame. | Directional types, frame gate and T44. | Generic x/y parameter names are explicitly insufficient. |
| `029` | Existing YME/PR are reused only as dynamic properties with unit and source-run tags. | `brittleness` source, module spec, persisted metadata/provenance, consumers and T45/T46. | Numeric tests exist, but whole-contract metadata and downstream-static behavior are not yet proven. |
| `030` | Dynamic-to-static transforms are sourced, versioned and required before static stress use. | Transform registry, source/version, refusal and T47. | `YME_D2S` and `PR_D2S` ship absent; identity is forbidden. |
| `031` | Modulus and strength conversions are dimensional across Mpsi, psi, GPa, MPa and bar. | Typed conversion registry, live call sites and T48-T50. | `brittleness` contains one local GPa-to-Mpsi factor; that is not a governed cross-domain conversion system. |
| `032` | Stress and stress gradient remain distinct physical quantities and curves. | Typed output schema, conversion, metadata and T51. | Generic unit strings or shared plotting axes do not establish type safety. |
| `033` | Inclined stresses transform all six total-stress components in a declared right-handed frame and sign convention. | Tensor/frame types, eigensolve/transform, provenance and T52/T53. | No inclined-stress implementation was found. |
| `034` | Every stress input declares total or effective state and incompatible use refuses. | State type, composition gate and T54. | A numeric tensor without state metadata is insufficient. |
| `035` | Thermal, depletion and damage terms remain explicit policy inputs even when omitted. | Optional-term policy schema, refusal and T55. | Absence must remain visible; treating omitted terms as silent zero is not compliant. |

Manual capability candidates: `unconventional`, `workflow`, `log-view`, `report`. The current unconventional row is not exercised and is not a GEO qualification.

### Failure criteria and wellbore stability (`036`-`041`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `036` | Seven public-equation criterion IDs, including two Drucker-Prager forms, are independently gated. | Public sources, registry, parameters, UI/dispatcher and T56/T57. | Vendor raster names are inventory only; no criterion registry was found. |
| `037` | Drucker-Prager solves the published invariant equation numerically with convergence evidence. | Invariant implementation, bracket/root diagnostics and T58. | A transcribed vendor closed form is explicitly prohibited. |
| `038` | Shear-failure mode classification is separate from the binary failure result. | Mode enum, exposed angle boundary, output schema and T59. | `45°` must come from the named source and inclusivity must be declared. |
| `039` | Every stability input is validated before solve without clamping invalid values. | Complete sourced range registry, refusal aggregation and T60. | One range check does not close `every`; uncited ranges remain absent. |
| `040` | Strength correlations retain native input/output units and convert once. | Per-correlation unit metadata, typed conversion and T49/T50/T61. | A numeric result equal after two compensating mistakes is not proof. |
| `041` | Every angle back-transform uses `atan2` under the declared frame and round-trips. | All back-transform call sites, frame definition and T62. | One helper test cannot close the universal statement without a call-site inventory. |

Manual capability candidates: none currently registered for wellbore stability; do not borrow unrelated capability checks.

### Source custody, calibration, identity, export and executable records (`042`-`052`)

| ID | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `042` | Empty sources fail registration and every required ABSENT parameter blocks before allocation. | Source/parameter registry, preflight and T63. | Generic parameter defaults can violate this contract if applied to GEO. |
| `043` | Parameter files are addressed by semantic key and ordinal, and disagreement refuses. | Parser/schema, loader, diagnostics and T64. | Generic JSON/object parsing is not the semantic-plus-ordinal contract. |
| `044` | Local calibration is versioned and named but cannot become the global default. | Calibration store, promotion refusal, undo/provenance and T65. | Existing calibration workflows are supporting patterns, not a GEO calibration implementation. |
| `045` | Every correlation refuses extrapolation outside its declared range and records the breach. | Applicability schema, runtime guard, null output and T66. | Miller's cited 2000-ft ceiling cannot be generalized to other methods. |
| `046` | Raster, binary and opaque coefficient artifacts cannot enable a method. | Registration/resource gates and T27/T67. | The absence of such artifacts is only one side; the enablement path must also refuse them. |
| `047` | Imported, computed and interpreted curves with the same mnemonic remain distinct retrievable identities. | Set/type/version schema, write/read resolution and T68. | Generic `(set_name, mnemonic)` is supporting evidence; verify an explicit interpreted identity and no overwrite. |
| `048` | Export contains the complete GEO run record: inputs, parameters, sources, applicability, transforms, masks/counts and ordered processing. | Export schema and an end-to-end run fixture for T69. | Generic LAS provenance cannot supply method-specific fields that were never computed. |
| `049` | Shared parameters such as water density and Biot alpha are single-valued within one composed run. | Run-scoped resolver, conflict refusal and T15/T70. | Per-module private values are a violation even when numerically equal in one fixture. |
| `050` | Every numeric chapter example executes through released calculators. | Released calculator inventory, source-linked fixtures and T71. | Recomputing examples in test-only code is a circular snapshot, not proof of the product path. |
| `051` | Filtering, limiting and other post-processing are separate visible ordered steps. | Raw/final outputs, operation log, UI/export and T72. | No filter equation may be inferred; Off must preserve raw values. |
| `052` | Each acquisition-dependent method gates independently so one missing source/input does not disable another valid method. | Six-unit registry, method preflight, independent run and T02/T73. | A single monolithic unavailable GEO domain would violate this even while safely refusing. |

Manual capability candidates: `delivery-sets`, `generic-curve-store`, `processing-history`, `report`, `las-export`, `verification-stewardship`. They remain supporting cross-cutting evidence only.

---

## Chapter Test-Intent Routing Guard

Execution MUST account for every `SB-GEO-T01` through `SB-GEO-T73` exactly once as a primary route, while preserving the source-owned cross-links below. A test may support multiple rows, but one partial helper cannot close each linked contract.

| Test | Primary requirement | Required cross-cutting check |
|---|---|---|
| `T01` | `001` | exact six-gate registry and versions |
| `T02` | `001` | independent availability also supports `052` |
| `T03` | `002` | incompatible datum refusal |
| `T04` | `002` | exact ft/m conversion under one datum |
| `T05` | `003` | anchored vertical-stress arithmetic |
| `T06` | `003` | first interval is not discarded |
| `T07` | `003` | cross-unit integration equivalence |
| `T08` | `004` | measured/synthetic mask and source identities |
| `T09` | `004` | unresolved density gap blocks stress |
| `T10` | `005` | independently derived SSE selection |
| `T11` | `005` | no-measurement fit refusal |
| `T12` | `006` | breached scope is persisted; also supports `022` |
| `T13` | `006` | incomplete applicability registration refusal |
| `T14` | `007` | packaged vendor-row absence also supports `020` and `024` |
| `T15` | `008` | one shared typed water density also supports `049` |
| `T16` | `008` | labelled correlation-embedded exception |
| `T17` | `009` | boundary-anchored normal pressure |
| `T18` | `009` | absent normal gradient blocks |
| `T19` | `010` | explicit alpha effective stress |
| `T20` | `011` | Eaton sonic arithmetic |
| `T21` | `011` | all four method-specific monotonic directions |
| `T22` | `012` | trend/raw/final/masks output record |
| `T23` | `013` | absent Bowers coefficient preflight |
| `T24` | `013` | calibration evidence completeness |
| `T25` | `014` | loading/unloading round trip |
| `T26` | `014` | invalid unloading exponent refusal |
| `T27` | `015` | readable-equation gate also supports `046` |
| `T28` | `016` | default-Off lower-limit behavior |
| `T29` | `016` | explicit floor and disclosed clamp |
| `T30` | `017` | pressure/gradient type separation |
| `T31` | `018` | generalized fracture equation with alpha one |
| `T32` | `018` | generalized fracture equation with non-unity alpha |
| `T33` | `019` | explicit K-family requirements and no fallback |
| `T34` | `020` | primary-fit monotonicity gate |
| `T35` | `021` | premise departure; BLOCKED while `MK_OBG_TOL` lacks a cited value |
| `T36` | `022` | source-geography policy and persisted breach |
| `T37` | `023` | exposed breakpoint and branch inclusivity |
| `T38` | `023` | conflicting-source breakpoint refusal |
| `T39` | `024` | primary-derived 10-family/30-row Daines schema |
| `T40` | `025` | published labelled envelope bounds |
| `T41` | `026` | default-Off/On fracture ceiling behavior |
| `T42` | `027` | zero-strain horizontal-stress arithmetic |
| `T43` | `027` | directional strain exchange |
| `T44` | `028` | undeclared frame refusal |
| `T45` | `029` | dynamic values plus persisted type/unit/source-run tags |
| `T46` | `029` | invalid dynamic fixture plus downstream static null |
| `T47` | `030` | absent dynamic-to-static transform blocks |
| `T48` | `031` | Mpsi to psi/GPa dimensional conversion |
| `T49` | `031` | MPa to psi; also supports native-unit contract `040` |
| `T50` | `031` | bar to psi; also supports native-unit contract `040` |
| `T51` | `032` | stress and two gradient representations remain distinct |
| `T52` | `033` | identity-frame principal stresses |
| `T53` | `033` | compression-sign normalization |
| `T54` | `034` | effective/total state mismatch refusal |
| `T55` | `035` | explicit thermal/depletion/damage policies |
| `T56` | `036` | vendor-raster-only criterion registration refusal |
| `T57` | `036` | seven public-equation criterion IDs |
| `T58` | `037` | numerical invariant-root convergence record |
| `T59` | `038` | sourced shear-mode boundary and inclusivity |
| `T60` | `039` | sourced range refusal without clamp |
| `T61` | `040` | psi-native strength output is not double-converted |
| `T62` | `041` | `atan2` quadrant and round trip |
| `T63` | `042` | empty source and required-ABSENT parameter refuse separately |
| `T64` | `043` | semantic/ordinal mismatch diagnostic |
| `T65` | `044` | local calibration cannot become global default |
| `T66` | `045` | cited range breach returns null and record |
| `T67` | `046` | opaque artifact cannot enable disabled method |
| `T68` | `047` | imported/computed/interpreted identities do not overwrite |
| `T69` | `048` | complete GEO run export record |
| `T70` | `049` | private shared-value conflict refuses composition |
| `T71` | `050` | every numeric example through released calculators |
| `T72` | `051` | Off versus explicit ordered post-processing |
| `T73` | `052` | one missing acquisition method leaves valid Eaton runnable |

The two existing unconventional tests are candidates only for partial support of `T45` and `T46`. Execution MUST search current tests and reachable history for every other body and MUST NOT rename an unrelated test into coverage after the fact.

---

### Task 1: Freeze Evidence and Create the 52-Row Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-geo.md`
- Read only: all governing and evidence paths listed above

- [ ] Reverify branch, base, accepted anchor, origin, merge base and cleanliness.
- [ ] Run the exact 52-row guard and byte-compare every source-owned `owned_tests` field against the planning commit.
- [ ] Read both GEO evidence dossiers completely and inventory every cited, absent, conflicting and non-adoptable parameter without copying vendor rows.
- [ ] Extract all 73 chapter test intentions and map each to zero or more actual test bodies.
- [ ] Build exact Rust/TypeScript source, test, manual and Git inventories; no verdict is copied from the chapter.
- [ ] Create one heading for each `SB-GEO-001` through `SB-GEO-052` with `apply_patch`; do not commit an empty skeleton.
- [ ] Machine-check that all 52 headings are unique and in order and all 73 test intentions are accounted for.

### Task 2: Adjudicate Domain Gates, Datum, Density and Pore Pressure

**Rows:** `SB-GEO-001` through `SB-GEO-017`

- [ ] Trace the live module catalog, dispatcher, UI entry points and independent availability behavior. Confirm the absence or presence of each of the six required domain units separately.
- [ ] Inventory every depth and datum type used by candidate paths. Distinguish a numerical ft/m converter from a physical reference contract.
- [ ] Search for every vertical-stress and density-combiner path; inspect anchor semantics, first interval, missing gaps, source masks and provenance.
- [ ] Inventory correlation registration/applicability schemas and vendor-resource gates without opening prohibited artifacts as implementation truth.
- [ ] Trace every candidate normal-pressure, Eaton and Bowers symbol through source, tests, UI, stored outputs and export. Do not promote `precalc::FPRESS` into PPFG.
- [ ] Verify raw/final pressure, limit policy, masks/counts and pressure/gradient typing as separate obligations.
- [ ] Open and run actual candidates for `T01`-`T30`; classify helper and partial tests honestly.
- [ ] Keep all absent/conflicting parameters absent and write seventeen complete receipt verdicts.

### Task 3: Adjudicate Fracture Pressure, Elastic/Static Properties and Stress Frames

**Rows:** `SB-GEO-018` through `SB-GEO-035`

- [ ] Search for the generalized fracture equation, every K family, coefficient source, breakpoint, scope and premise gate in source, tests and reachable history.
- [ ] Preserve the Matthews-Kelly and Daines primary-source blocks; do not count vendor table absence as method implementation.
- [ ] Trace pressure-envelope and cap behavior through computation, plot, export and provenance.
- [ ] Inspect `brittleness` from raw DT/DTS/RHOB inputs through module metadata, workflow storage and every consumer. Keep numeric, unit, dynamic/static and source-run obligations separate.
- [ ] Inventory all modulus/strength conversion paths and call sites; a local constant does not establish a typed registry.
- [ ] Search horizontal-stress and inclined-stress paths for directional strains, all six tensor components, frame, compression sign and total/effective state.
- [ ] Open and run actual candidates for `T31`-`T55`, explicitly recording the `T35` cited-tolerance block and partial nature of existing `T45/T46` candidates.
- [ ] Write eighteen complete receipt verdicts.

### Task 4: Adjudicate Stability, Source Custody, Identity and Complete Run Records

**Rows:** `SB-GEO-036` through `SB-GEO-052`

- [ ] Inventory public-equation failure criteria and distinguish names/raster inventory from executable equations and sourced parameter sets.
- [ ] Trace any Drucker-Prager implementation through invariant construction, bracket, numeric root, convergence record and output.
- [ ] Inventory every stability validation, native-unit correlation and angle back-transform call site; universal claims require complete inventories.
- [ ] Inspect source/parameter registration, semantic-plus-ordinal loading, local calibration versioning, extrapolation and opaque-artifact refusal.
- [ ] Trace imported, computed and interpreted identity separately through writes, lookup, plot and export.
- [ ] Inventory the actual GEO run-record fields, shared-run parameter resolution and ordered post-processing; generic provenance is support only.
- [ ] Verify independent acquisition-dependent method gates and execute actual candidates for `T56`-`T73`.
- [ ] Write seventeen complete receipt verdicts.

### Task 5: Classify Tests, Manual Evidence and Reachable History

- [ ] For every `T01`-`T73`, record `CORRECTNESS`, `CHARACTERIZATION`, supporting-only, missing proof or blocked by a named source/parameter.
- [ ] For every correctness expected value, name the chapter/dossier/public source or independently show the arithmetic. Never use implementation output as its own oracle.
- [ ] Run every discovered candidate test by exact name. Record command, result and assertion surface; compilation alone is not a pass.
- [ ] Confirm all consequential negative source findings in expected modules, UI, tests and `git log --all -S/-G` history.
- [ ] Record manual evidence only from the generated capability matrix and checked review scenarios. Do not create a GEO capability row or check a scenario in this lane.
- [ ] Ensure no receipt text contains a client, asset, field, block, basin, operator, well or project name; describe the physical condition and source class instead.

### Task 6: Update the Ledger Atomically and Self-Review All 52 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-geo.md`

- [ ] Prepare all 52 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-GEO rows and all source-owned fields.
- [ ] Enforce that no GEO row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 52 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every characterization is labelled, every block names its dependency, and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 218 adjudicated, 713 remaining.

### Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 52-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, hard source/parameter blocks and `713/931` rows remaining.
- [ ] Name the next serial domain only as a recommendation; do not start it.

### Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, PRD-audit check and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-geo.md`, `docs/takeover/requirements.csv` and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-GEO adjudicate 52 SB-GEO requirements`; do not push, merge or begin a production fix.

---

## Plan Self-Review Before Execution

- [ ] Exactly 52 live SB-GEO IDs are covered once; no row can be silently skipped.
- [ ] All thirty-three original P0s, seventeen P1s and two P2s are adjudicated without treating old priority as pilot policy.
- [ ] All 73 chapter test intentions are routed and all 52 source-owned test mappings remain unchanged.
- [ ] The plan changes no production behavior, test, PRD or research dossier.
- [ ] Six independent domain gates are judged separately; one partial dynamic seam cannot imply a GEO suite.
- [ ] Unit conversion and datum/reference typing remain separate obligations.
- [ ] `precalc::FPRESS`, generic curve carriage, generic provenance and generic export remain support, not PPFG proof.
- [ ] Dynamic YME/PR remains dynamic; no identity transform or downstream static use is inferred.
- [ ] Pressure, pressure gradient, stress and stress gradient remain distinct typed quantities.
- [ ] All correlation scopes, validity ranges, source geographies, breakpoints and native units are checked at registration and runtime.
- [ ] Every absent parameter remains absent and every non-adoptable table remains non-adoptable.
- [ ] No vendor lookup row, raster coordinate, binary constant or local delivered-study preset is copied.
- [ ] The missing Matthews-Kelly premise tolerance blocks `T35`; it is not supplied from plausibility.
- [ ] Primary-paper recovery for Matthews-Kelly, Daines and other named gaps remains a separate source increment.
- [ ] Every failure criterion needs a public equation and sourced parameters; a registry name is not capability.
- [ ] Every correctness expected value is independently sourced or derived; current output never serves as its own oracle.
- [ ] Manual evidence, automated tests, accepted Git reachability, source admissibility and pilot field evidence remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 218 adjudicated / 713 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution starts only after Jauhar explicitly approves this plan.
