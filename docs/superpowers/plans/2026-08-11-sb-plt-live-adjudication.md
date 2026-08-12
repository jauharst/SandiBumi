# SB-PLT Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 35 live `SB-PLT` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing plotting behavior, scientific defaults, generated chart payloads or the PRD.

**Architecture:** This is a documentation-only evidence pass. The PRD remains an immutable statement of intent and historical chapter status. Current source, qualifying observable acceptance tests, manual evidence and reachable Git history establish the separate live verdict. One domain receipt explains every decision; `docs/takeover/requirements.csv` carries the machine-validated summary. The pass traces the complete plotting contract from semantic curve binding through axes, ranges, binning, scientific overlays, multi-well allocation, interaction state, provenance, export, accessibility and performance. Rust helpers and frontend implementations are evaluated separately and then at their actual integration seams; agreement between duplicate helpers is not treated as a single governed implementation.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, generated chart data, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md` or any file under `docs/PRD_v2/**`.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. The branch is stacked on the reviewed SB-DBM adjudication commit `0cf9480b2b0d8c074f42245edde85c5aace29d48`; `origin/master` was frozen at `29833735816d9e5be954afafd9ceb71fd856e3f0` when this plan was written. Reverify all three before execution. If an accepted reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, tests and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete plotting chapter, the applicable `docs/record_*.md` files and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. Chapter statuses are historical evidence, not current verdicts.
- All 35 source-owned `owned_tests` fields are blank even though plotting chapter section 6 lists 43 test intentions, `SB-PLT-T01` through `SB-PLT-T43`. Do not backfill the immutable ledger field. Map candidate implementations in the receipt and classify whether each is a whole-contract correctness test, characterization, supporting test or missing proof.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid pilot. Original P0/P1/P2 is evidence, not automatic pilot scope.
- A positive mechanism does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list, or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence, and uses an independently sourced expected value. A Rust helper, TypeScript normalization function, schema snapshot, internal `Result`, source-text grep or compile success is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Do not treat a named test in the PRD as implemented until its body is found, its assertion surface is inspected and the test is executed.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked. A partial capability row cannot be promoted to complete by inference.
- Preserve the distinction between data validity, display clipping and scientific exclusion. A displayed range MUST NOT become an implicit validity filter or change the reported population.
- Preserve semantic binding separately from concrete resolution. A mnemonic, alias or requested semantic role is not proof that the concrete well/set/curve/version used by a rendered plot was persisted durably.
- Preserve native LAS samples and `(set_name, mnemonic)` identity. Reframe or resampling remains explicit and cannot be introduced as a plotting convenience.
- Keep missing sample data as `f32::NAN`, never `Option<f32>` in numeric arrays. Arrays cross IPC as bytemuck bytes, never JSON numeric arrays.
- The frontend never sends SQL for writes. Every plot-derived data edit must use a whitelisted backend operation and remain undoable with complete non-null provenance. This adjudication cannot invent a write path to satisfy a plotting contract.
- Preserve the deliberately PK-less `computed_curves` table and its one-writer DELETE-then-append discipline. Never recommend or implement a primary key, `ON CONFLICT`, upsert or duplicate-tolerant writer.
- Python remains a subprocess and every runner reads `sys.stdin.buffer`; this planning/adjudication lane does not add or alter runners.
- A petrophysical value, unit limit, range, tolerance, cutoff or scientific default is cited or absent. Never infer one from code, a neighboring vendor, a chart shape, a plausible textbook value or model training.
- Pickett `m`, `n`, `a` and `Rw` remain absent unless a named source in the project supplies them for the relevant interpretation. The current identifiable `a*Rw` product cannot be split by assumption.
- `TERNARY_SUM_TOL`, `INTERACTION_GATE`, `FIRST_PAINT_GATE` and any robust-regression method/tuning default are deliberately absent. Record the gap; do not choose values or policies.
- `SB-PLT-032` cannot be closed by a synthetic micro-benchmark, source comment or developer-machine timing. Its evidence must name the qualified hardware, dataset and acceptance thresholds required by the chapter.
- `SB-PLT-024` has a live evidence conflict: the chapter says `PRESENT-OK`, while generated `src/ui/chartOverlays.ts`, `docs/IP_PROVENANCE.md` and `docs/takeover/CLAIMS.md` identify shipped digitized vendor-chart payloads and a legal-review boundary. Engineering cannot adjudicate legal permission. Inspect and record the conflict without editing the generated file, deleting payloads, transcribing coordinates, or declaring the legal issue resolved.
- `src/ui/chartOverlays.ts` is generated and read-only. Never hand-edit it. No coordinate, curve, polygon, line or chart payload may be copied into the receipt or any other artifact; record only identifiers, counts and provenance conclusions needed for adjudication.
- `SB-PLT-035` requires a governed batch equation. Numerical agreement between a duplicated interactive formula and a batch formula is characterization evidence, not proof of one governed implementation.
- `SB-PLT-033` depends on the geomechanics datum/sign contract owned by chapter 18. Do not invent that convention in the plotting adjudication.
- Git reachability proves only that a change is in the accepted tree. Commit messages are locators, never correctness evidence; open the accepted source and test body.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- This planning commit authorizes no ledger verdict. Execution may create only the SB-PLT receipt, adjudicate the 35 SB-PLT ledger rows and update the dashboard after Jauhar reviews and approves this plan. It does not authorize a source fix, test addition, legal conclusion, parameter choice or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 35 SB-PLT rows:

```text
SB-PLT-001 SB-PLT-002 SB-PLT-003 SB-PLT-004 SB-PLT-005 SB-PLT-006 SB-PLT-007
SB-PLT-008 SB-PLT-009 SB-PLT-010 SB-PLT-011 SB-PLT-012 SB-PLT-013 SB-PLT-014
SB-PLT-015 SB-PLT-016 SB-PLT-017 SB-PLT-018 SB-PLT-019 SB-PLT-020 SB-PLT-021
SB-PLT-022 SB-PLT-023 SB-PLT-024 SB-PLT-025 SB-PLT-026 SB-PLT-027 SB-PLT-028
SB-PLT-029 SB-PLT-030 SB-PLT-031 SB-PLT-032 SB-PLT-033 SB-PLT-034 SB-PLT-035
```

At plan time all 35 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is eighteen P0, thirteen P1 and four P2. Gate 1 adjudicates all 35 because a lower historical priority can still determine whether a scientific plot, export, interaction or legal surface is trustworthy in the paid pilot.

Run this guard before and after editing:

```powershell
$plt = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-PLT-*' }
$expected = 1..35 | ForEach-Object { 'SB-PLT-{0:D3}' -f $_ }
if ($plt.Count -ne 35) { throw "Expected 35 SB-PLT rows, found $($plt.Count)" }
if (@(Compare-Object $expected @($plt.requirement_id)).Count -ne 0) {
    throw 'The live SB-PLT ID set differs from the approved plan'
}
if (@($plt | Where-Object { -not [string]::IsNullOrWhiteSpace($_.owned_tests) }).Count -ne 0) {
    throw 'A source-owned SB-PLT owned_tests field changed after planning'
}
```

The mechanical post-execution count is exactly 166 adjudicated and 765 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-plt.md` - complete 35-row evidence receipt, including obligation-by-obligation source findings, test classification, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 35 SB-PLT rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary and next serial-domain handoff.

### Read-only governing inputs

- `AGENTS.md`, `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, `04_CORE_REQUIREMENTS.md`, `06_SEQUENCING_AND_GATES.md`, `23_plotting-interactivity.md`, `91_REQUIREMENTS_INDEX.md`
- applicable `docs/record_*.md` files, especially `record_data_tools.md` and `record_fixes.md`
- `docs/IP_PROVENANCE.md`, `docs/takeover/DECISIONS.md`, `CLAIMS.md` and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Primary source and test inspection surfaces

- `src-tauri/src/plotting.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/lib.rs`
- `src/ipc.ts`, `src/distribution.ts`
- `src/ui/plotTypes.ts`, `plotCommon.ts`, `plotCanvas.ts`, `plotExport.ts`, `workspace.ts`
- `src/ui/histogramPanel.ts`, `crossplotPanel.ts`, `pickettPanel.ts`, `vegaPanel.ts`, `correlationPanel.ts`, `compositeDialog.ts`
- generated, read-only `src/ui/chartOverlays.ts`
- `tools/frontend-acceptance.test.mjs` and all Rust/TypeScript tests reached from the 43 chapter intentions

### Files this adjudication MUST NOT change

- every read-only governing input and source/test path above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/`;
- `docs/IP_PROVENANCE.md`, `REVIEW.md` and the generated verification matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-plt.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness and 35-row guard result. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-PLT-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, chapter test intentions and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unwired or unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source and class.
- Supporting tests: helper or partial tests that help but cannot close the complete contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or UNIMPLEMENTED; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, source, dependency, legal decision or none.
- Next action: one bounded production, test, field, legal/owner decision or no-action increment.
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

### Semantic binding, axes, ranges and typed overlays (`001`-`005`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `001` | A plot persists semantic intent separately from each well's concrete set/curve/version resolution. | `plotting.rs::resolve_plot_bindings`, Tauri command registration, `ipc.ts::getCurveData`, binding registry, template/project persistence and a recovered whole-contract test. | The current in-memory binding registry and source hash support resolution, but do not by themselves prove durable project/template persistence. |
| `002` | Every axis follows one declared precedence chain and records the winning source. | Rust range resolver, every panel's actual range construction, template restore, exports and T01/T02. | A helper test can pass while panels continue using hard-coded or data-derived ranges independently. |
| `003` | Overlay admission is typed by physical quantity and compatible units, with governed conversions. | Rust type registry, frontend overlay declarations/conversions, actual overlay authorization and T03/T04. | Two registries that agree today still drift unless one derives from the other; mnemonic alias matching is not dimensional compatibility. |
| `004` | Validity filtering and display clipping remain separate and both disclose their effects. | range-policy helper, statistics population, plot render/export summaries, support from T07/T12 and a recovered whole-contract test. | The Rust helper is supporting evidence unless the active plot path uses or faithfully implements the same contract. |
| `005` | Every unit-limit row is dimensionally audited before activation, with disabled reasons preserved. | complete unit-limit inventory, audit registry, UI activation paths and T05. | One deliberately disabled row does not prove the universal inventory or activation gate. |

Manual capability candidates: `histogram`, `crossplot`, `pickett`, `chart-overlays`, `themes-language-accessibility`, `verification-stewardship`.

### Binning, statistics and scientific interpretation (`006`-`012`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `006` | One canonical histogram contract defines bin count, boundaries, final-endpoint inclusion and non-finite handling. | `src/distribution.ts`, `histogramPanel.ts`, Rust distribution/plotting helpers, templates/exports and T06/T07. | The default migrated to 50 and the shared TypeScript routine appears wired; verify no panel or export retains a second rule. |
| `007` | Overplot thresholds expose the exact comparator and threshold source. | crossplot/plot rendering density paths, settings/templates/legend/export and T08/T09. | No qualifying comparator surface was found during planning; confirm the consequential negative before verdict. |
| `008` | Percentile probability and range position are distinct types with distinct validation and serialized names. | Rust/TypeScript types, all parsers/callers, templates, exports and T10/T11. | Type helpers exist, but the range-position parser appears unwired; helper coverage cannot prove the product surface. |
| `009` | Every statistic discloses population, estimator and exclusions. | histogram/crossplot/Pickett/Vega/composite statistics, UI labels, exports and T12/T13. | Existing frontend coverage explicitly characterizes finite statistics without the required disclosure. |
| `010` | Regression output is a versioned scientific record with method, tuning, inputs, exclusions and uncertainty. | regression implementations, result storage, UI/export, provenance and T14/T15/T16. | Current coverage characterizes coefficient-only output; robust method and tuning defaults are intentionally absent. |
| `011` | Pickett exposes only identifiable quantities and never silently separates `a` from `Rw`. | `pickettPanel.ts`, Rust fit helper, stored writes/provenance, guides/export and T17. | The `a*Rw` product is support; verify every guide, label, save and restore path before closing the universal statement. |
| `012` | Hingle uses the specified negative reciprocal exponent and makes source/parameters explicit. | all Hingle symbols/routes, equation seam, UI/export and T18/T19. | No Hingle panel or product symbol was found during planning; confirm source, tests and reachable history before recording absence. |

Manual capability candidates: `histogram`, `crossplot`, `pickett`, `equation-engine`, `processing-history`, `verification-stewardship`.

### Missing policy, multi-well capacity and depth reconciliation (`013`-`017`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `013` | Missing and out-of-range behavior is explicit per X/Y/Z/waveform channel and disclosed to the user/export. | Rust/TypeScript policy types, every plot's channel application, legends/manifests, support from T07/T12/T16/T20 and a recovered whole-contract test. | The shared type contract exists, but active use appears narrower than all required channels. |
| `014` | Multi-well allocation occurs after finite-pair screening, so all-NaN wells consume no quota and represented wells retain endpoints. | `plotCommon.ts::fetchContextLayers`, Rust allocation helper, every multi-well panel and T20. | One shared context helper does not prove every active multi-well plot uses it. |
| `015` | Decimation preserves one paired index vector, endpoints and provenance, including any forced point. | TypeScript/Rust decimators, active render/export routes, manifests and T21/T22. | Helper equality is not proof that every consumer uses the returned shared indices and reports forced endpoints. |
| `016` | Equal or integer-multiple steps proceed, non-integer steps route to explicit DIO reconciliation, and intervals remain half-open. | depth-reconciliation helpers, context loaders, Reframe routing, interval boundaries and T23/T24/T25/T26. | Do not add implicit resampling or inherit an uncited step tolerance. |
| `017` | Zoom outside the loaded interval triggers a generation-safe refetch with the requested interval identified. | viewport events, query parameters, caches, stale-request disposal and T27. | Local canvas zoom is not evidence of an identified data refetch; confirm all relevant load paths. |

Manual capability candidates: `crossplot`, `correlation-tops`, `composite`, `log-view`, `reframe`, `portfolio-performance`.

### Selection, invalidation, writes, expressions and faceting (`018`-`022`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `018` | Linked selections are named, typed, scope-aware and persistable with exact depth membership. | selection stores/events, project/template serialization, plot subscriptions and T29. | Existing frontend coverage explicitly characterizes a single ephemeral scope, not durable named selections. |
| `019` | Every plot subscribes to one invalidation contract and disposes all subscriptions. | inventory histogram, crossplot, Pickett, Vega, correlation, composite and exports; T32. | The chapter says `PRESENT-OK`; verify the universal inventory rather than sampling one panel. |
| `020` | Every plot-derived parameter write is whitelisted, undoable and carries complete non-null provenance. | `plotCommon.ts::writePlotParameter`, backend finalization, `setZoneParam`, every handle/fit/polygon/marker writer and T30/T31. | The central path is strong evidence only if no plot writer bypasses it and persisted output retains the provenance. |
| `021` | Expression-valued channels execute only through a governed sandbox and serialize reproducibly. | Vega/equation seams, accepted expression grammar, IPC/write boundaries, templates and a recovered whole-contract test. | Arbitrary panel JavaScript is prohibited; an expression-looking field without a governed executor cannot close the contract. |
| `022` | Faceting partitions the scientific population before per-facet decimation. | Vega/facet construction, context budgeting, manifests/exports and T34. | Separate facet and reduction features do not establish their required order. |

Manual capability candidates: `crossplot`, `vega`, `correlation-tops`, `composite`, `curve-editing`, `workflow`, `processing-history`.

### Chart provenance, legal boundary, templates and export (`023`-`027`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `023` | A chart may render in a deliverable only when source revision, rights/disposition and transformation provenance are complete. | provenance schema, `authorizeProvenancedChart`, generated definitions, project/template/export persistence and T35. | The refusal helper is support; all current chart records appear to lack the required provenance, so visible blocking and all bypass paths must be verified. |
| `024` | Vendor chart coordinates/payloads are not transcribed into the product. | generated-file header/content class, generator inputs, `IP_PROVENANCE`, claim register, build/package reachability and a recovered whole-contract test. | Direct conflict: the chapter says present-OK while current generated code and legal register identify shipped digitized vendor payloads. Do not resolve the legal question or reproduce payload data in the receipt. |
| `025` | Plot templates are schema-versioned, scope-aware, migratable and preserve unknown fields and provenance dependencies. | template state/normalizers, save/restore, version/migration/diff behavior and T36. | A passing unknown-field normalization test pins only one clause of the compound requirement. |
| `026` | Export reruns the scientific draw at paper scale and labels vector/raster output with population, exclusions, provenance and reduction metadata. | `plotExport.ts`, SVG/PDF/PNG/print paths, crop/scale behavior, manifests and T37/T38. | Existing coverage characterizes vector labeling while the PNG/print path is not equivalently labelled. |
| `027` | Plot state is portable while restricted chart payloads remain external and unresolved references fail visibly. | template/project serialization, chart IDs versus payloads, missing/unknown chart behavior, support from T35/T36 and a recovered whole-contract test. | Storing only an ID helps portability, but silently clearing an unavailable overlay violates visible refusal; repository-shipped payload remains a separate legal issue. |

Manual capability candidates: `chart-overlays`, `report`, `office-deliverables`, `project-lifecycle`, `security-integrity`, `verification-stewardship`.

### Rendering lifecycle, accessibility, truncation and performance (`028`-`032`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `028` | Static scientific draw and interaction overlays have separate invalidation scopes. | crossplot/canvas/render lifecycle, resize/data/style/selection events, support from T28/T32/T33 and a recovered whole-contract test. | Current test coverage is explicitly characterization of separate subsets, not necessarily the specified complete event matrix. |
| `029` | Every asynchronous plot load is generation-safe and disposes superseded work before replacement. | `workspace.ts::createPlot`, every async panel builder/loader, cleanup and T28/T33. | One correctness test covers the shared replacement helper; inventory consumers and bypasses before accepting the universal claim. |
| `030` | Interactive canvases are keyboard and assistive-technology reachable, expose non-pointer properties and remain exportable. | `plotCanvas.ts`, focus/ARIA/keyboard lifecycle, property panels, export routes and T39. | Existing correctness coverage pins keyboard lifecycle but may not pin all non-pointer and export clauses. |
| `031` | No load, facet, legend or render silently truncates records; every reduction is declared and hard limits refuse. | context budgets/manifests, panel-specific caps, exports, legends, T40 and supporting T20/T21/T22/T34. | The context manifest covers several reductions, but universal inventories and hard-limit bypasses remain to be checked. |
| `032` | Interaction and first-useful-paint performance are qualified on declared Windows hardware/datasets with explicit gates. | benchmark harness, release hardware receipt, datasets, measured distributions, T43 and manual capability evidence. | Hard block: latency and first-paint gates are absent and current manual performance capability is unexercised. Never invent thresholds. |

Manual capability candidates: `themes-language-accessibility`, `portfolio-performance`, `crossplot`, `histogram`, `chart-overlays`, `verification-stewardship`.

### Domain plot shells and governed equations (`033`-`035`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `033` | Pressure-gradient crossplots preserve the governed geomechanics datum and sign convention. | plot/panel inventory, chapter-18 dependency, axis/transform labels, templates/exports and a recovered whole-contract test. | No qualifying plot was found during planning; the convention is owned by geomechanics and cannot be invented here. |
| `034` | Ternary plots visibly normalize valid sums and visibly refuse or flag invalid sums under a sourced tolerance. | plot/panel inventory, normalization display, tolerance source and T41/T42. | No ternary surface was found and the invalid-sum tolerance deliberately ships absent. |
| `035` | Interactive clay-volume plots call the same governed equation as batch computation. | `modules.rs`, interactive panel route, batch module/equation route and a recovered whole-contract test. | The existing test explicitly characterizes a duplicated formula matching endpoints; numerical equality is not single-definition governance. |

Manual capability candidates: `crossplot`, `equation-engine`, `shale-volume`, `verification-stewardship`.

---

## Chapter Test-Intent Routing Guard

Chapter section 6 claims that its 43 intentions cover all 35 requirements, but it does not assign an owned test ID to each requirement and the consolidated ledger preserves every SB-PLT `owned_tests` field as blank. Use this table only to locate candidate evidence. A candidate may support more than one row; it closes none until its observable assertion surface proves every atomic obligation. Requirements not named in the primary-route column require an explicit `missing whole-contract proof` finding unless another independently sourced observable test is recovered.

| Chapter intentions | Primary contract route | Cross-cutting support that must not be overclaimed |
|---|---|---|
| `T01`-`T02` | `002` axis precedence and winning tier | May expose range provenance, but do not substitute for durable semantic binding in `001`. |
| `T03`-`T04` | `003` quantity/unit compatibility and registered conversion | Conversion persistence supports provenance only if the rendered/template/export path retains it. |
| `T05` | `005` documented converted-unit divergence | One row cannot prove the universal unit-limit inventory. |
| `T06`-`T07` | `006` canonical histogram boundaries and non-finite counts | `T07` may support `004`/`013`, but does not prove their range/channel universals. |
| `T08`-`T09` | `007` threshold conversion and explicit comparator | Verify the comparator is exposed at the product surface, not just calculated. |
| `T10`-`T11` | `008` distinct percentage types | Parser tests do not prove template/export type persistence. |
| `T12`-`T13` | `009` population, exclusions, estimator/quantile policy | `T12` may support `004`/`013`; finite arithmetic alone is not disclosure. |
| `T14`-`T16` | `010` regression result and transform exclusions | `T16` may support `013`; coefficients alone do not make a versioned record. |
| `T17` | `011` Pickett identifiability/refusal | Does not source `m`, `n`, `a` or `Rw`. |
| `T18`-`T19` | `012` Hingle exponent and reciprocal-sign refusal | Arithmetic must be independently derived as shown in the chapter. |
| `T20` | `014` finite-pair allocation and absent well | Supports `013` only for the named channel/shape. |
| `T21`-`T22` | `015` shared-index decimation, endpoints and manifest | Does not prove every consumer or export uses the same indices. |
| `T23`-`T26` | `016` depth-step reconciliation and half-open interval | No implicit resampling tolerance may be inferred. |
| `T27` | `017` identified out-of-bounds refetch | Generation safety is separately exercised by `T28`. |
| `T28`, `T33` | `029` newest-generation rendering and stale-build disposal | May support `017`/`028`, but neither test alone proves those complete contracts. |
| `T29` | `018` two named persistent selections | Must prove ID, colour, membership and persistence together. |
| `T30`-`T31` | `020` provenance-complete write and null-metadata refusal | Must cover whitelisting and undo as well as provenance. |
| `T32` | `019` shared invalidation across named open panels | Theme invalidation does not automatically prove all data/viewport/disposal clauses. |
| `T34` | `022` facet-before-budget behavior | Per-facet counts must be visible, not reconstructed only in a test. |
| `T35` | `023` incomplete chart-provenance refusal | May support `027`, but says nothing about whether restricted payload ships. |
| `T36` | `025` unknown-field preservation or explicit migration refusal | One future field does not prove versioning, scope or provenance dependencies. |
| `T37`-`T38` | `026` vector and raster export obligations | Each path must retain labels, provenance and uncropped layout independently. |
| `T39` | `030` keyboard pan/zoom and current accessible label | Non-pointer properties and export remain separate clauses. |
| `T40` | `031` export disclosure after reduction | Does not prove every load/facet/legend/render cap is non-silent. |
| `T41`-`T42` | `034` valid ternary normalization and invalid-negative refusal | The absent `TERNARY_SUM_TOL` still blocks any uncited invalid-sum boundary. |
| `T43` | `032` named-hardware performance report | The absent `INTERACTION_GATE` and `FIRST_PAINT_GATE` cannot be filled by the test run itself without an owner decision. |

The plan-time routing exposes no direct whole-contract chapter test for `001`, `004`, `013`, `021`, `024`, `027`, `028`, `033` or `035`. Execution MUST search current tests and reachable history before confirming that finding; it MUST NOT force an unrelated intention to “cover” the row merely to preserve the chapter's aggregate claim.

---

### Task 1: Freeze Evidence and Create the 35-Row Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-plt.md`
- Read only: all governing and evidence paths listed above

- [ ] Reverify branch, base, accepted anchor, origin, merge base and cleanliness.
- [ ] Run the exact 35-row guard and assert every row is still `UNADJUDICATED` with a blank source-owned `owned_tests` field.
- [ ] Extract all 43 chapter test intentions and map each to zero or more actual test bodies without writing those IDs into the source-owned ledger field.
- [ ] Build exact Rust/TypeScript source, test, manual and Git inventories; no verdict is copied from the chapter.
- [ ] Create one heading for each `SB-PLT-001` through `SB-PLT-035` with `apply_patch`; do not commit an empty skeleton.
- [ ] Machine-check that all 35 headings are unique and in order.

### Task 2: Adjudicate Binding, Axes, Ranges, Binning and Scientific Interpretation

**Rows:** `SB-PLT-001` through `SB-PLT-012`

- [ ] Trace semantic requests from panel/template state through `resolve_plot_bindings`, `getCurveData`, concrete resolution, cache/persistence and export.
- [ ] Inventory every actual axis/range/overlay path. Compare Rust support helpers with live TypeScript consumers rather than assuming one governs the other.
- [ ] Verify validity filtering and display clipping from both sides: changing display range must not change valid population, while a validity rule must change it and disclose the exclusion.
- [ ] Inventory all histogram implementations and call sites; prove or refute one 50-bin, half-open/final-inclusive, non-finite-aware contract.
- [ ] Open and run the `T01`-`T19` candidates routed above. Classify the relevant helper and frontend characterization tests honestly.
- [ ] Keep overplot, regression, Pickett and Hingle scientific defaults absent unless a named source supplies them. Never split `a*Rw` without sourced `a` or `Rw`.
- [ ] Write twelve complete receipt verdicts.

### Task 3: Adjudicate Missing Policy, Multi-Well Capacity, Depth and Interaction State

**Rows:** `SB-PLT-013` through `SB-PLT-022`

- [ ] Inventory X, Y, Z and waveform missing/out-of-range behavior across all active panels, labels and exports.
- [ ] Trace all multi-well routes through finite-pair screening, allocation, shared-index decimation, endpoint preservation and reduction manifests.
- [ ] Verify depth-step and interval logic without introducing positional guessing, implicit Reframe or an uncited tolerance.
- [ ] Trace viewport expansion from input event to identified backend refetch, including generation and cancellation behavior.
- [ ] Inventory selection naming/type/scope/persistence and every plot subscription/disposal path.
- [ ] Inventory every plot-derived parameter writer and prove whitelisting, undo and complete provenance together.
- [ ] Inspect expression and faceting paths for governed execution and facet-before-decimation ordering.
- [ ] Open and run the candidates routed to these rows: `T07`, `T12`, `T16`, `T20`-`T34`. Treat every cross-cutting reuse as support unless the whole observable sentence is asserted.
- [ ] Write ten complete receipt verdicts.

### Task 4: Adjudicate Chart Provenance, Legal Boundaries, Templates and Export

**Rows:** `SB-PLT-023` through `SB-PLT-027`

- [ ] Inspect chart authorization at render, save, template, export and deliverable boundaries. A schema-level refusal cannot close an unguarded renderer.
- [ ] Inspect generated chart artifacts and generator/provenance records without copying any chart coordinate or payload into the receipt.
- [ ] Reconcile the chapter's `PRESENT-OK` statement for `024` with live generated-code and legal-register evidence. Record the engineering facts and preserve legal disposition as a named owner decision.
- [ ] Trace template schema version, scope, unknown fields, migration, diff, chart references and provenance dependencies independently.
- [ ] Exercise SVG, PDF, PNG and print/export paths and inspect their scientific metadata, scale/crop behavior and reduction disclosure.
- [ ] Open and run `T35`-`T38` and any recovered whole-contract candidates for `023`-`027`; do not treat ternary `T41` as chart-portability evidence merely because both concern plot records.
- [ ] Write five complete receipt verdicts.

### Task 5: Adjudicate Rendering Lifecycle, Accessibility, Performance and Domain Plots

**Rows:** `SB-PLT-028` through `SB-PLT-035`

- [ ] Inventory static versus interaction invalidation and every asynchronous plot loader/disposer.
- [ ] Verify keyboard, focus, assistive description, non-pointer property and export obligations as separate clauses.
- [ ] Inventory all record, facet, legend and render caps. Require visible manifests or hard refusal for every reduction; do not accept silent slicing.
- [ ] Preserve `SB-PLT-032` as evidence-gated unless the named Windows hardware/dataset measurements and sourced thresholds actually exist.
- [ ] Confirm pressure-gradient and ternary product surfaces through source, tests and history; preserve chapter-18 and tolerance dependencies.
- [ ] For `035`, distinguish duplicated-formula endpoint agreement from invocation of the governed batch equation.
- [ ] Open and run `T28`, `T32`, `T33`, `T39`-`T43` and every recovered whole-contract candidate for `028`-`035`.
- [ ] Write eight complete receipt verdicts.

### Task 6: Update the Ledger Atomically and Self-Review All 35 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-plt.md`

- [ ] Prepare all 35 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-PLT rows and all source-owned fields, including blank `owned_tests`.
- [ ] Enforce that no PLT row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 35 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every characterization is labelled, every block names its dependency, and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 166 adjudicated, 765 remaining.

### Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 35-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, hard evidence blocks and `765/931` rows remaining.
- [ ] Name the next serial domain only as a recommendation; do not start it.

### Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, PRD-audit check and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-plt.md`, `docs/takeover/requirements.csv` and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-PLT adjudicate 35 SB-PLT requirements`; do not push, merge or begin a production fix.

---

## Plan Self-Review Before Execution

- [ ] Exactly 35 live SB-PLT IDs are covered once; no row can be silently skipped.
- [ ] All eighteen original P0s, thirteen P1s and four P2s are adjudicated without treating old priority as pilot policy.
- [ ] All 43 chapter test intentions are mapped, while all 35 blank source-owned `owned_tests` fields remain unchanged.
- [ ] The plan changes no production behavior, generated chart payload or PRD v2 file.
- [ ] Semantic intent, concrete per-well resolution and durable plot-state persistence remain separate obligations.
- [ ] Rust support helpers and live TypeScript product paths are traced separately; duplicate implementations do not prove one definition.
- [ ] Validity, display clipping, finite-pair screening, decimation and export populations remain distinct and disclosed.
- [ ] Native samples and `(set_name, mnemonic)` identity remain intact; Reframe stays explicit.
- [ ] Pickett `m`, `n`, `a`, `Rw`, ternary tolerance, robust-regression tuning and performance gates remain absent until their specified evidence exists.
- [ ] The chart-payload conflict is recorded without transcribing payloads, editing generated code or making a legal determination.
- [ ] Every plot-derived write remains whitelisted, undoable and provenance-complete; no frontend SQL or database-discipline change is proposed.
- [ ] No helper, compile check, source grep, Git message or chapter status closes an observable contract by itself.
- [ ] Every expected value is sourced or explicitly characterized; no petrophysical value or scientific threshold is selected.
- [ ] Manual evidence, automated tests, accepted Git reachability, legal disposition and pilot field evidence remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 166 adjudicated / 765 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution starts only after Jauhar explicitly approves this plan.
