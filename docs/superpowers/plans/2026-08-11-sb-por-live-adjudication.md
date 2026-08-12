# SB-POR Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 62 live `SB-POR` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing porosity math, choosing a petrophysical parameter, resolving an open source conflict, changing the PRD, or implementing a missing capability.

**Architecture:** This is a documentation-only evidence pass. The PRD remains the immutable statement of intended behavior; because the porosity chapter omits all per-requirement statuses and all owned-test mappings, those source-owned blanks remain blank. Current source, qualifying observable acceptance tests, manual evidence, and reachable Git history establish a separate live verdict in `docs/takeover/requirements.csv` and one domain receipt. The pass follows the complete porosity chain: module architecture, typed quantities and naming, shale correction, sonic transforms, neutron-density and neutron-sonic crossplots, hydrocarbon correction, excavation, limits and flags, solver discipline, source custody, units, provenance, comparison curves, vendor-parameter intake, audit reporting, and core comparison. Generic module, workflow, family, provenance, source-panel, plot, conditioning, calibration, and export infrastructure are supporting seams; none is promoted into a complete POR contract unless every observable obligation is proved.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Petrophysical math, parameter custody, and data-integrity judgments remain with the primary session.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. This plan is written on `bba826dd15be67a07126d7d313fdbece04ddf73f`; `origin/master` and the current merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`. Reverify all four before execution. If any reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave `D:\XX. SandiBumi-check` untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, qualifying tests, and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete POR chapter, applicable `docs/record_*.md` files, `docs/method_sonic_porosity.md`, `_SPINE_PENDING.md` entries SP-009 and SP-012 through SP-015, and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Chapter omissions are evidence, not permission to infer values.
- All 62 `chapter_status` fields and all 62 `owned_tests` fields are blank by source. Preserve those blanks byte-for-byte. Route the chapter's test intentions only in the receipt and this plan; do not backfill source ownership.
- Keep SP-009 open. The chapter defines 41 test IDs but deliberately has no numeric `SB-POR-T26` or `SB-POR-T27`; `T14b` and `T18b` are real IDs. Do not invent the missing numeric IDs or renumber any test.
- The chapter front matter and live ledger agree on 62 requirements: seventeen P0, twenty-five P1, seventeen P2, and three P3. Do not infer a status from priority.
- Mechanically, section 5 contains 74 parameter-table rows, 15 rows whose value cell contains `ABSENT`, and 8 containing `NON-ADOPTABLE`. The front matter and closing prose say 18 parameters/rows ship `ABSENT`, while one visible row bundles a three-parameter roll-off set. Preserve this as a structural finding, inspect every row individually, and do not silently normalize either count or edit the PRD.
- Every `ABSENT` parameter remains absent. Every `NON-ADOPTABLE` value remains verification-only. A current code literal, neighboring vendor value, project precedent, range endpoint, average, rounded value, or test fixture is not a citation and cannot become a default.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid open-hole-petrophysics pilot. Original P0-P3 priority is evidence, not automatic pilot scope.
- A positive helper or seam does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list, or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as a qualifying owned acceptance proof only when it exercises the requirement's observable contract and uses an independently sourced expected value. A helper, internal `Result`, source-text grep, compile success, manifest shape, or arithmetic copied from the implementation is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Tests that pin a known current defect are divergence evidence, not proof of the specified behavior.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked. At plan time `porosity` is `0 / 33`, `generic-curve-store` is `0 / 18`, `conditioning` is `0 / 27`, `workflow` is `0 / 23`, `las-export` is `0 / 2`, and `processing-history` is `0 / 7`. `histogram` is only `5 / 22` and `crossplot` only `6 / 13`; those partial plot exercises do not prove POR method correctness, parameter custody, provenance, or pay exclusion.
- The current module framework has a generic `sources_topic` seam plus a source panel, although the chapter's older as-built note says `ModuleSpec` has no source field. Inspect the live argument, backend topic registry, dialog, and run persistence before classifying `SB-POR-007` or `055`; neither the stale note nor the generic seam decides the full contract.
- The current POR catalog exposes `phi_den`, `phi_dn`, and `phi_son`, plus supporting `phimax`, `badhole`, `condflag`, `nphimat`, `gascorr`, `ssc`, and `sspw` paths. Presence in the same category does not prove one family, one limiter, one flag stream, one output contract, or one provenance contract.
- `phi_den` and `phi_dn` currently return unlimited method-specific pairs plus shared limited `PHIE`/`PHIT`; `phi_son` returns only method-specific `PHIT_SON`/`PHIE_SON` and uses different `[0, 1]` limits. Inspect manifest resolution, actual write names, overwrite behavior, and downstream reads separately.
- `PHIE_FLOOR` is currently a compile-time `0.001`, backed by a later direct product-decision record, while `SB-POR-045` requires the conflicting vendor values to ship with no default. The execution receipt MUST state the current behavior, chapter contract, and later product decision separately. It MUST NOT silently decide which evidence supersedes the other.
- SP-012 is open: the current Wyllie compaction path can apply `Cp < 1` and inflate porosity. Jauhar has not chosen clamp versus refusal. Adjudication records the silent behavior and open decision; it does not select or implement a remedy.
- SP-013 is open: the current option named `RHG` is not the three-segment RHG80 transform. Jauhar has not chosen rename-with-source versus implementation of RHG80. Do not collapse the current approximation, the chapter's `FIELD_OBSERVED` proposal, and RHG80 into one method.
- SP-014 is open: one user-visible sonic module description contains a prohibited geographic parenthetical. The physical-condition phrase already carries the meaning. Record the exact surface without reproducing the prohibited proper name in the receipt or plan.
- `docs/method_sonic_porosity.md` is newer primary-source evidence for the `Cp >= 1` direction, the estimator, RHG80's three segments, and the fact RHG removes the compaction correction. It does not authorize an implementation choice or an uncited approximation coefficient.
- The nine Bateman-Konen constants remain `NON-ADOPTABLE` until ESC-POR-8 is closed. A chart-free analytic method cannot be classified shippable merely because current shortcuts or a vendor implementation produce plausible answers.
- ESC-1, ESC-2, ESC-3, ESC-5, ESC-7, and ESC-POR-8 remain explicit source/custody boundaries. Do not open protected vendor binaries or charts, copy vendor chart data, infer missing formula terms, or substitute cross-vendor agreement for a primary source where the chapter refuses that substitution.
- Current `gascorr` is a density-log correction and supporting solver precedent, not the POR hydrocarbon-correction chain. Current `condflag`/`badhole` outputs and `nphimat` conversions are supporting inputs until declared, consumed, and recorded by POR methods.
- `sspw` currently declares three neutron parameters it does not read and uses a gas-conditioning expression that differs from `ssc`. Those are live candidates for `SB-POR-058` and `059`; inspect the manifest, reachable body, exact tests, and history rather than trusting chapter line numbers.
- Core-porosity comparison is a post-check. The live calibration record forbids automatic parameter adjustment. Do not count a generic crossplot, correlation, or calibration engine as `SB-POR-062` unless the observable method-by-method POR report exists and preserves that no-adjustment boundary.
- The receipt and ledger additions MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer to physical conditions and source classes only. Do not open confidential project records merely to restate a source citation already carried by the immutable chapter.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record all of the following:

1. branch `codex/g1-sb-geo-adjudication`;
2. a clean worktree and the sole registered worktree at `D:\XX. SandiBumi`;
3. current HEAD, accepted anchor, `origin/master`, merge base, and accepted-anchor reachability;
4. exactly 62 `SB-POR` ledger rows, covering `001` through `062` with no gap or duplicate;
5. priority counts `P0=17`, `P1=25`, `P2=17`, `P3=3`;
6. `chapter_status` blank for 62 rows, `owned_tests` blank for 62 rows, and `as_built_status=UNADJUDICATED` for 62 rows;
7. exactly 41 chapter test IDs: `T01` through `T25`, `T14b`, `T18b`, and `T28` through `T41`; numeric `T26` and `T27` absent;
8. 74 parameter-table rows, with the literal state counts recorded separately from the chapter's claimed count;
9. takeover summary `279` adjudicated, `652` unadjudicated, and `207` pilot blockers before any POR verdict edit; and
10. the current manual capability counts listed in Global Constraints.

The only mechanically predictable post-adjudication ledger count is `341` adjudicated and `590` unadjudicated. Do not predict as-built, release-disposition, risk, or test-class totals before evidence is classified.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-por.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: current Rust/TypeScript/source/test/history/manual-evidence surfaces required below
- Never modify: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts

## Evidence Receipt Schema

Create one `### SB-POR-NNN` section per requirement in numeric order. Every section MUST include:

- **Specified contract:** the complete observable obligation, split into independently checked limbs.
- **Current implementation:** exact symbols and paths, including negative inventory scope where absent.
- **As-built status:** one legal ledger state with a sentence explaining why stricter states fail.
- **Release disposition and risk:** pilot relevance independent of implementation status.
- **Automated evidence:** exact test names and commands, each classified `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`; name the expected-value source for correctness.
- **Manual evidence:** exact capability/scenario evidence or `NONE`; never infer it from automated checks.
- **Source/parameter boundary:** cited, absent, non-adoptable, conflicting, or not applicable, with unresolved escalation IDs.
- **History/reachability:** accepted commit evidence or confirmed negative history search where consequential.
- **Blocking decision/dependency:** exact source, product decision, UI evidence, or implementation dependency; never a vague “future work”.
- **Next action:** a bounded follow-up or `NONE` when the contract is fully proved and field evidence is not required.

## Requirement Evidence Map

### Group A - Architecture, typing, and seams (`001`-`012`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | One deterministic POR module family, limiting contract, flag contract, and output-naming contract. | POR specs/functions in `modules.rs`; `ssc.rs`; module catalog; `workflow.rs`; generated dialog; output resolution. | A shared category or runner is not a unified POR contract; inventory every reachable method and every write name. |
| `002` | Distinct unlimited and limited pairs for every porosity method. | `phi_den_spec`/`phi_den`; `phi_dn_spec`/`phi_dn`; `phi_son_spec`/`phi_son`; manifest-output test; export. | Pin both pairs and both meanings; a single method-specific pair does not satisfy the all-method claim. |
| `003` | Per-sample branch-and-limit flag for every POR branch and clamp. | POR functions; clamp sites; `badhole`/`condflag`; workflow writes; catalog/export/provenance. | Numeric clamping, NaN, or a generic mask is not the required POR flag stream. |
| `004` | Typed porosity family, method/convention provenance, and collision-free output mnemonics. | `curves.rs` `FAMILIES`; POR manifests; workflow output-name resolver; computed-curve catalog/run metadata. | Check imported-versus-computed identity and sequential runs; family aliases alone do not prevent overwrite. |
| `005` | Separately named correction and forward-model functions wherever both directions exist. | Excavation/HC inventory in Rust, Tauri commands, TS callers, tests, and history. | A sign flag, private algebraic inversion, or round-trip helper fails the naming contract. |
| `006` | Typed VSH/VCL consumption and refusal of untyped or wrong-family input. | `curves.rs`; input resolution in workflow/module paths; POR manifests; CLY bridges; validation/reporting. | Matching mnemonic text is not type proof; inspect both accepted and refused controls. |
| `007` | Each module parameter carries source citation and evidence tier, surfaced in the dialog. | `ArgSpec.sources_topic`; `param_sources.rs`; `moduleDialog.ts`; `paramSources.ts`; run persistence. | A generic topic seam is partial until every POR parameter has source+tier and the run records the choice. |
| `008` | One shared formation-water-based `PHIT_SH`, exported across the CLY seam and kept distinct from shale subtraction. | `phit_sh_at`; `phi_den`; `phi_dn`; `sspw`; CLY bridge; output/export surfaces. | Same mnemonic or similar algebra is not identity; run every current formation path on one parameter set. |
| `009` | `PHIT >= PHIE` by limit-first/rebuild ordering at every sample. | `phi_den`; `phi_dn`; `phi_son`; SSPW/SSC POR outputs; workflow tests. | Test floor, ceiling, shale branch, missing data, and every method; happy-path arithmetic alone is insufficient. |
| `010` | Audit trail records method, full parameters, and exact input curve identities for every POR curve. | module-run records; processing history; computed-curve provenance/catalog; workflow save/reload. | A generic timestamp or module name is not re-derivable provenance. |
| `011` | One shared matrix density across every documented chained module. | POR manifests/functions; `gascorr`; `condflag`; workflow templates and saved overrides. | Equal-looking defaults in some modules do not prove shared custody; trace one override through the whole chain. |
| `012` | CSR bridge exists, refuses absent CSR, preserves four specified relations, and ships no default. | CLY/POR bridge inventory; family typing; dialog/run validation; tests/history. | Do not treat `CSR=1`, a direct VSH/VCL alias, or a generic equation module as implementation. |

### Group B - Shale correction and sonic (`013`-`020`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `013` | Explicit per-method `NORMALISED` versus `SUBTRACTIVE` convention with no mixed method. | `phi_son_spec`; `phi_son`; parameter/run headers; saved workflows. | Infer neither convention from arithmetic; it must be named, selected, and persisted. |
| `014` | Honest sonic method names, parameterized field-observed coefficient, and true vendor-labelled RHG renderings only. | `OPT_SON`; sonic doc/labels; current `RHG` branch; source panel; SP-013 evidence. | Do not choose rename versus RHG80 implementation; classify the current shipped name and formula separately. |
| `015` | Non-Wyllie methods use shale-reduced, matrix-floored slowness and rescale; Wyllie retains subtractive form. | `phi_son`; candidate sonic tests; reachable history. | Raw-DT transforms or a shared shale term cannot stand in for the two explicitly different conventions. |
| `016` | Lithology-selected cited matrix transit-time family with no lithology-agnostic default. | `DT_MA` manifest; source-topic registry; mineral/lithology inputs; saved run. | A numeric range or current 55.5 literal is not a cited choice family. |
| `017` | Wyllie compaction can only reduce porosity; `Cp < 1` is refused or hard-flagged. | `OPT_CP`; `DT_SH`; `phi_son`; SP-012; sonic source note/tests. | Record the open clamp-versus-refusal decision; never turn the current ratio into authority. |
| `018` | Every shale-corrected slowness is floored at `DT_MA` before use. | Sonic reduction functions and every method branch. | An output `[0,1]` clamp after the transform is not the required input-domain floor. |
| `019` | Matrix endpoint and fitted exponent are selected as a cited matched mineral pair. | sonic args/options; parameter-source registry; dialog/run record. | Two independently editable numeric fields do not satisfy pair custody. |
| `020` | Exactly one sourced RHG rendering is default; alternatives are labelled comparisons. | current sonic registry; SP-013; `docs/method_sonic_porosity.md`; comparison-curve typing. | Do not select a rendering during adjudication and do not credit a misnamed approximation. |

### Group C - Neutron-density and neutron-sonic crossplots (`021`-`028`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `021` | Primary chart-free analytic Bateman-Konen-family N-D method. | `phi_dn`; module catalog; crossplot methods; tests/history. | Average/RMS shortcuts are not analytic crossplot methods; the nine vendor constants remain unavailable under ESC-POR-8. |
| `022` | Chart-derived paths use only SandiBumi-owned gated digitisation and build-time validation. | `nphimat`; `neutron_charts.rs`; chart-generation/validation tests; provenance. | Do not open or copy vendor charts; generic overlay availability is not a POR method. |
| `023` | Average and RMS are labelled comparison curves, never POR methods or pay-default inputs. | `phi_dn` option/doc; curve family/provenance; pay resolver/summation defaults. | Test both registry exclusion and downstream exclusion; renaming only the UI is insufficient. |
| `024` | N-D refuses undeclared neutron matrix units and records the declared basis. | `nphimat`; curve metadata/family resolution; `phi_dn` inputs; run provenance. | Canonical `v/v` alone does not identify limestone/sandstone/dolomite matrix basis. |
| `025` | Salinity-dependent endpoint method evaluates fresh and salt cases and interpolates on fluid density. | N-D method inventory; neutron chart tables; salinity options; source metadata. | A binary salt toggle or one endpoint table is not the specified interpolation. |
| `026` | Gas crossover is surfaced on the POR output. | `condflag` `XOVER_FLAG`; POR input manifests; output flags/provenance. | Existing detector output is supporting-only until the POR run consumes and records it. |
| `027` | Neutron-sonic two-point method exists without the dimensionally defective shale form. | module/Tauri/TS catalog; sonic/crossplot functions; tests/history. | Absence of the forbidden equation does not prove the positive method exists. |
| `028` | Density/neutron shale-reduction clamps are cited parameters and each bind raises the POR flag. | hard-coded clamp sites in `phi_dn`; manifests; source registry; flags/tests. | Pin each lower and upper bound plus a non-binding control; a silent clamp fails. |

### Group D - Hydrocarbon correction (`029`-`038`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `029` | Conventional apparent-HC electron density plus stated validity envelope; incompatible modified form not adopted. | POR HC inventory; `gascorr` only as supporting context; model registry/docs/tests/history. | A gas-density correction is not the electron-density POR chain. |
| `030` | Sourced quadratic HC hydrogen-index model; divergent legacy neutron form not adopted by default. | HC model inventory; options/compat paths; QC outputs; tests/history. | A hidden legacy option must be identified and warned; absence of one bad formula is not positive implementation. |
| `031` | Poupon A/B factor architecture, with density and neutron terms observable term-by-term. | POR HC functions/modules; outputs; provenance; solver code. | An opaque iterative correction or density-only `gascorr` does not satisfy the two-sided architecture. |
| `032` | Mud-filtrate density and hydrogen-loss are explicit parameters, never worked-example literals. | POR HC manifests; parameter sources; run persistence. | Equal current numbers do not establish correct quantity identity or source. |
| `033` | Selected HC model refuses or hard-flags all specified low-density invalid regions. | model dispatch; configuration validation; per-sample flags; warnings/run report. | Pin both invalid and valid sides for every reachable model; clamping to a plausible number fails. |
| `034` | Explicit vendor-named HC model selection, all verification variants, full QC intermediates, and output provenance. | model/options inventory; module outputs; source/run metadata; tests/history. | Comparison availability does not make a rejected variant a default; each intermediate and provenance limb is separate. |
| `035` | Flushed-zone saturation exponent is an explicit no-default user decision. | POR HC args/dialog/validation; run header; saved workflow. | Do not inherit a value from any saturation module, vendor, or current fixture. |
| `036` | Per-zone force-wet branch suppresses all POR HC corrections and raises the POR flag. | zone parameters; HC solver; flag stream; workflow persistence. | A global bypass or water-case arithmetic is not a per-zone declared branch. |
| `037` | Computed HC hydrogen index is checked against the stoichiometric ceiling at every sample. | HC loop/intermediate outputs; validation flags/tests. | The physical ceiling is an independent assertion, not a tunable default or end-of-run summary only. |
| `038` | Density `gascorr` is distinguished from POR HC correction; double correction is explicit; non-convergence remains missing. | `gascorr_spec`/`gascorr`; POR catalog; workflow dependency/provenance; exact guard tests. | Do not count `gascorr` as the missing POR chain; inspect both single- and double-correction routes. |

### Group E - Excavation (`039`-`042`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `039` | Additive neutron excavation rendering with documented evidence basis; multiplied rendering unreachable. | excavation inventory in Rust/TS/tests/history; neutron correction paths. | ESC-1 remains open; do not promote a corroborated exponent into primary-sourced authority. |
| `040` | Excavation correction and forward-model directions are two named functions. | same inventory plus public command/function names and callers. | A round-trip test cannot prove two public, semantically named directions. |
| `041` | Suppression uses resolved tool identities; ambiguous/unresolved tokens fail visibly. | tool registry; neutron metadata; configuration loader; diagnostics/tests. | Never copy or execute the vendor evidence string; preserve the unresolved token as unresolved. |
| `042` | Primary classic-excavation lithology constants are acquired before settling the exponent. | source register and ESC-1 custody; no production candidate expected. | This is source-gated; cross-vendor numerical agreement cannot close it. |

### Group F - Limits, branches, and flags (`043`-`049`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `043` | High-shale kill threshold is a cited parameter, not a literal. | `phi_den`; `phi_dn`; other POR methods; manifests/source panel. | A hard-coded 0.95 with historical provenance still fails parameter custody. |
| `044` | Optional smooth high-shale roll-off with three no-default parameters. | POR limit/ceiling modules and options; dialog/run validation. | Never invent the three defaults or treat `phimax` as the specified Vcl roll-off. |
| `045` | PHIE floor is a documented no-default user decision. | `PHIE_FLOOR`; limit tests; workflow/pay paths; direct product-decision record. | Preserve the evidence conflict; do not weaken the floor safety behavior or declare the chapter silently superseded. |
| `046` | If VSILT exists, its do-not-trust warning is user-visible and recorded. | module/curve catalog; dialog/docs; provenance. | If the index is absent, classify absence; do not create a warning-only stub. |
| `047` | POR methods declare/consume BADHOLE and record its effect in the POR flag. | `badhole`; POR manifests/functions; workflow mask; provenance. | A generic Mask dependency or standalone detector is not wiring. |
| `048` | POR methods declare/consume conditioning flags with defined branch behavior. | `condflag`; POR manifests/functions; output flags and run record. | Check COAL, TIGHT, and combined condition separately; detector quality is not consumer behavior. |
| `049` | No hard-coded lithology-kill literals. | POR/crossplot branch inventory; configuration and tests/history. | Prove both absence of the forbidden literal branch and presence of any intended typed alternative before a positive verdict. |

### Group G - Iteration and solver discipline (`050`-`052`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `050` | Every iterative POR solve exposes cap/tolerance, uses absolute-change inequality, and emits no partial iterate. | `gascorr` precedent; any POR HC/N-D solver; manifests; tests/reporting. | An internal loop guard is supporting-only unless parameterized and observable. |
| `051` | Multi-unknown solver precedence is documented, deterministic, and recorded. | POR solver inventory; configuration and provenance. | Implementation iteration order hidden in code is not a product contract. |
| `052` | Invalid solver combinations are rejected at configuration time. | manifests/dialog validation; Tauri command guard; exact error surface. | A runtime NaN or internal `Err` without a user-facing refusal does not close this row. |

### Group H - Provenance, refusals, and comparison (`053`-`062`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `053` | Crossplot shale porosity uses fluid-minus-matrix span; defective sample-dependent denominator is unreachable. | crossplot/sonic functions, method registry, tests/history. | Prove positive canonical form and negative unreachability from both sides. |
| `054` | Canonical sign convention plus algebraic-identity test for inverted vendor forms. | density/sonic/crossplot helpers and tests; docs/run metadata. | Numerically matching one example is not an identity proof; derive independently. |
| `055` | Every POR parameter carries source+tier; unresolved conflicts ship absent with competing values visible. | all POR `ArgSpec`s; `param_sources.rs`; source panel; run provenance; section 5 inventory. | Audit all 74 parameter rows/quantities; generic source UI or a cited subset cannot satisfy “every.” |
| `056` | Canonical internal POR, transit-time, and density units; display conversion is separate. | curve families/units; import conversion; manifests; module context; UI display. | Matching labels are not conversion proof; run bidirectional unit controls without JSON arrays. |
| `057` | Quick-look comparison curves have distinct structure/visual identity/provenance and are pay-excluded by default. | curve families/catalog; plot styling; pay resolution/summation; provenance. | Visual distinction alone or pay exclusion alone is partial. |
| `058` | No reachable module presents a parameter its body never reads. | every `ModuleSpec`; runner arg access; `sspw_spec`/`sspw`; dead-param tests. | Source grep must understand reachable aliases/branches; a manifest-shape test is not behavioral proof. |
| `059` | SSPW gas conditioning matches SSC's sourced RMS midpoint. | `ssc`; `sspw`; exact gas-branch tests and history. | Re-derive the expected value independently; do not use one implementation as the oracle for the other. |
| `060` | Vendor parameter-set import is cited, tiered, read-only, and never default authority. | intake/import inventory; parameter-source registry; persistence/UI; history. | Generic JSON/profile import is not enough; mutation and defaulting must be refused. |
| `061` | Per-run POR audit report lists methods, sourced parameters, flag counts, and bound limits. | processing history; run records; report/export paths; UI. | A generic module log or curve catalog is incomplete; inventory every required field. |
| `062` | Core-porosity post-check reports method bias/scatter and never auto-adjusts parameters. | core calibration/crossplot/report paths; `record_calibration`; tests/manual evidence. | Generic correlation is supporting-only; preserve the no-automatic-adjustment rule. |

## Test-Intent Routing Guard

Every actual chapter test intention has one primary inspection owner so none is skipped or counted twice. Cross-support is documented separately and does not populate the blank source-owned `owned_tests` field.

| Test | Primary row | Contract sentence routed for inspection |
|---|---|---|
| `T01` | `029` | Apparent HC electron density matches the sourced model and envelope. |
| `T02` | `030` | HC hydrogen index matches the sourced gas/oil envelopes. |
| `T03` | `030` | Divergent legacy neutron-side behavior is isolated behind explicit compatibility behavior. |
| `T04` | `039` | Excavation reference case reproduces the sourced additive family. |
| `T05` | `039` | Lithology sensitivity rejects the weaker exponent form. |
| `T06` | `039` | Multiplied excavation rendering remains unreachable. |
| `T07` | `005` | Forward and inverse HC paths round-trip through separately defined directions. |
| `T08` | `021` | Analytic and gated chart branches agree within the specified comparison envelope. |
| `T09` | `056` | Canonical and alternate units produce invariant results. |
| `T10` | `016` | Cited lithology matrix-transit choices remain distinct and no global default is introduced. |
| `T11` | `009` | PHIE is limited before PHIT is rebuilt. |
| `T12` | `028` | Shale-reduction clamp binding is flagged and logged. |
| `T13` | `034` | The compatibility behavior that suppresses excavation is explicit and warning-bearing. |
| `T14` | `041` | Excavation tool gating uses resolved, case-stable identities. |
| `T14b` | `041` | The unresolved tool token remains an explicit diagnostic, never a guessed mapping. |
| `T15` | `008` | PHIT_SH uses formation-water density, not the neighboring fluid term. |
| `T16` | `050` | Non-convergence yields missing plus a flag, never a partial iterate. |
| `T17` | `033` | Every reachable HC form stays within its declared low-density validity discipline. |
| `T18` | `015` | Non-Wyllie sonic methods use shale-reduced slowness rather than the stale document form. |
| `T18b` | `015` | Iterative field-observed sonic seeds from shale-reduced, not raw, slowness. |
| `T19` | `008` | Apparent/effective shale subtraction remains distinct from clay-bound-water porosity. |
| `T20` | `013` | Shale convention is a real named fork and is written to the run header. |
| `T21` | `015` | Wyllie remains the cross-vendor control under the subtractive path. |
| `T22` | `053` | Canonical neutron-sonic shale porosity is bounded and the defective form is unreachable. |
| `T23` | `012` | Missing CSR refuses, while an explicit CSR reproduces and clamps every bridge relation. |
| `T24` | `050` | N-D convergence uses an absolute inequality with parameterized tolerance and cap. |
| `T25` | `035` | Conflicting Sxo exponents remain visible and no default is shipped. |
| `T28` | `014` | No method is named RHG unless it computes a published RHG rendering. |
| `T29` | `017` | Compaction cannot silently inflate porosity when Cp is below one. |
| `T30` | `008` | Every module forms exactly the same PHIT_SH on one parameter set. |
| `T31` | `004` | Sequential POR methods preserve both outputs under collision-free mnemonics. |
| `T32` | `004` | Porosity family typing exists and preserves imported-versus-computed provenance. |
| `T33` | `059` | SSPW and SSC gas conditioning reproduce the independently sourced RMS midpoint. |
| `T34` | `058` | Every declared module parameter is read on a reachable path. |
| `T35` | `011` | One matrix-density decision propagates through a documented chain. |
| `T36` | `023` | Average and RMS are comparison curves, not POR methods or pay defaults. |
| `T37` | `021` | Bateman-Konen and the average shortcut remain measurably distinct. |
| `T38` | `003` | A binding POR ceiling raises the declared per-sample flag. |
| `T39` | `001` | Every POR method uses the same floor/ceiling path and flag contract. |
| `T40` | `045` | PHIE floor is configuration with recorded choice, not a compile-time constant. |
| `T41` | `047` | BADHOLE and conditioning flags are declared, consumed, and recorded by POR methods. |

Cross-support remains mandatory: `001` also uses `T11`, `T31`, and `T39`; `002` uses `T11`, `T19`, `T31`, and `T39`; `003` uses `T12`, `T38`, `T39`, and `T41`; `004` uses `T31` and `T32`; `005` uses `T07`; `006` uses `T23`; `007`, `010`, `055`, `060`, and `061` require direct source/provenance inventory because no test intention alone covers their universal claims; `008` uses `T15`, `T19`, and `T30`; `011` uses `T35`; `013` uses `T20`; `014` uses `T28`; `015` uses `T18`, `T18b`, and `T21`; `016` uses `T10`; `017` uses `T29`; `018` uses `T18` and `T21`; `019` uses the sourced-pair inventory; `020` uses `T28` plus the primary paper; `021` uses `T08` and `T37`; `022` uses gated chart validation; `023` uses `T36`; `024` uses the matrix-basis inventory; `025` uses fresh/salt endpoint inventory; `026` uses `T41`; `027` uses `T22`; `028` uses `T12`; `029` uses `T01`; `030` uses `T02` and `T03`; `031` uses `T01`, `T02`, and `T07`; `032` uses parameter custody; `033` uses `T17`; `034` uses `T03`, `T13`, and `T17`; `035` uses `T25`; `036` uses the force-wet branch inventory; `037` uses the T02/T17 physical-bound limb; `038` uses `T16` and current `gascorr` guards; `039` uses `T04` through `T06`; `040` uses `T07`; `041` uses `T14` and `T14b`; `042` remains ESC-1 source-gated; `043` and `044` use limit inventory; `045` uses `T40`; `046` uses warning inventory; `047` and `048` use `T41`; `049` uses forbidden-branch inventory; `050` uses `T16` and `T24`; `051` uses solver-precedence inventory; `052` uses configuration-refusal inspection; `053` uses `T22`; `054` requires an independent algebraic proof; `056` uses `T09`; `057` uses `T36`; `058` uses `T34`; `059` uses `T33`; and `062` requires the core-comparison/no-auto-adjust evidence. No one test closes a compound row merely because it appears in this list.

Before adjudication, mechanically confirm that the primary table contains exactly the 41 real IDs once each, contains neither `T26` nor `T27`, and leaves every source-owned ledger test field blank.

## Task 1: Freeze the Execution Tree and Create the Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-por.md`
- Read: governing inputs and current Git state

- [ ] Confirm branch, clean worktree, exact anchors, reachability, merge base, and sole-worktree boundary.
- [ ] Run the 62-row, priority, blank-status, blank-test, parameter-table, test-ID, and manual-capability guards from the Baseline and Count Contract.
- [ ] Confirm `node tools/takeover-ledger.mjs --summary-json` still reports 279 adjudicated and 652 unadjudicated before any verdict edit.
- [ ] Read every governing input listed above; do not use chapter prose or this plan as a substitute for current source.
- [ ] Create all 62 receipt headings in numeric order and the final test, manual-evidence, parameter-conflict, source-escalation, product-decision, and follow-up summaries before classifying any row.
- [ ] Record SP-009, SP-012 through SP-015, the 74-row/ABSENT-count mismatch, the PHIE-floor evidence conflict, ESC-1/2/3/5/7/POR-8, and the manual-evidence boundary at the top of the receipt.

## Task 2: Adjudicate Architecture, Typing, and Sonic Rows `001`-`020`

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Read: module catalog, `modules.rs`, `ssc.rs`, `curves.rs`, workflow/output resolution, source-panel UI, run persistence, relevant tests/history

- [ ] Inventory every deterministic POR method from manifest through dispatch, write-name resolution, persistence, catalog, export, and downstream selection.
- [ ] Split family, unlimited/limited outputs, naming, flags, types, source custody, PHIT_SH, ordering, provenance, matrix density, and CSR into separate observable checks.
- [ ] Run exact candidate tests for manifest-output parity, PHIE flooring, density/D-N branches, workflow ordering, export, family resolution, and source-panel behavior; classify each proof surface honestly.
- [ ] For consequential missing family/type/refusal behavior, confirm the negative in expected Rust, TypeScript, tests, and reachable history.
- [ ] Re-derive PHIT_SH and the PHIT/PHIE ordering from the chapter equations rather than from current implementation output.
- [ ] Inventory every sonic selector, label, formula, shale-reduction step, floor, compaction branch, parameter source, output, and run-record field.
- [ ] Keep current approximation, proposed field-observed name, and RHG80 separate; keep Wyllie subtractive and non-Wyllie normalized paths separate.
- [ ] Record SP-012/013/014/015 exactly. Do not choose a Cp remedy, RHG disposition, coefficient, matrix endpoint, or matched exponent.

## Task 3: Adjudicate Crossplot and Hydrocarbon Rows `021`-`038`

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Read: POR/crossplot modules, chart-validation seams, curve metadata, conditioning flags, HC/gas paths, workflow, provenance, tests/history

- [ ] Prove or disprove the chart-free analytic N-D method independently from average/RMS comparison curves; preserve ESC-POR-8's non-adoptable constants.
- [ ] Inspect SandiBumi-owned chart-validation gates without opening or copying protected vendor chart data.
- [ ] Trace neutron matrix basis from import metadata and `nphimat` through POR input validation and output provenance; inspect wrong-basis refusal from both sides.
- [ ] Trace salinity endpoint behavior, crossover flag wiring, neutron-sonic availability, and all four hard-coded clamp endpoints separately.
- [ ] Inventory the complete POR hydrocarbon chain independently from `gascorr`: model selection, apparent electron density, neutron HI, A/B factors, parameters, validity guards, force-wet branch, physical ceiling, intermediates, solver behavior, and provenance.
- [ ] Run exact current `gascorr`, `condflag`, `nphimat`, module, workflow, and provenance tests only as their assertion surfaces permit; detector or density-correction success is supporting-only for the POR chain.
- [ ] Keep every no-default parameter absent and every rejected/legacy model explicit. Do not infer the flushed-zone exponent or any validity endpoint from a current fixture.

## Task 4: Adjudicate Excavation, Limits, and Solver Rows `039`-`052`

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Read: neutron/excavation inventory, tool register, limit paths, conditioning inputs, solver/configuration paths, tests/history

- [ ] Confirm excavation presence or absence across public modules, helpers, Tauri commands, TS callers, tests, and reachable history.
- [ ] Keep additive versus multiplied form, lithology evidence basis, forward versus correction direction, and tool applicability as separate obligations.
- [ ] Preserve the unresolved tool token and source escalations; never guess a tool identity or open a protected binary.
- [ ] Inventory every POR floor, ceiling, high-shale branch, smooth-rolloff option, VSILT warning, BADHOLE/conditioning branch, and lithology kill.
- [ ] Reconcile the current PHIE-floor safety record with the chapter's no-default contract without changing either. Record a product-owner decision dependency if live evidence cannot establish precedence.
- [ ] Inventory every iterative POR solver and its configuration surface. For each, verify tolerance, cap, inequality, cap-hit output, unknown precedence, invalid combinations, user-facing refusal, and run record separately.

## Task 5: Adjudicate Provenance, Refusal, and Comparison Rows `053`-`062`

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Read: method registries, unit/family conversion, parameter-source system, comparison/pay paths, SSPW/SSC, import/profile inventory, report/history, core comparison/calibration, tests/history

- [ ] Prove the canonical shale-porosity and sign conventions independently and confirm forbidden forms are unreachable.
- [ ] Audit all 74 parameter rows/quantities against live manifests, source topics, dialog display, run persistence, and absent/non-adoptable behavior. Record the chapter count mismatch without editing the PRD.
- [ ] Trace canonical POR/density/transit-time units through imports, storage, module context, outputs, and display conversion.
- [ ] Trace comparison curves through mnemonic/family, visual identity, provenance, and pay selection; none of those surfaces substitutes for another.
- [ ] Audit every reachable module argument against body reads, starting with SSPW's three known candidates. Distinguish an intentionally inactive, visibly labelled arg from a silently dead one.
- [ ] Independently compute the SSC/SSPW RMS reference and run both gas branches; one implementation cannot be the other's correctness oracle.
- [ ] Search for vendor parameter-set intake, POR audit reporting, and method-by-method core comparison. Confirm consequential absences in source, tests, and reachable history.
- [ ] Preserve `record_calibration`'s explicit no-automatic-adjustment contract when assessing core comparison.

## Task 6: Classify All 41 Test Intentions, Manual Evidence, and Reachable History

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Read: all discovered tests, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md`, accepted/reachable Git history

- [ ] Route all 41 real test IDs exactly once through the primary table; explicitly record that numeric T26/T27 do not exist and are not invented.
- [ ] For every correctness expected value, name the chapter/public source or independently show the arithmetic. Never use implementation output as its own oracle.
- [ ] Treat T28 through T41 as specified-behavior tests that currently target documented defects unless live code proves closure; a current-value pin is divergence/characterization evidence, not correctness.
- [ ] Run every discovered candidate test by exact name. Record command, result, expected-value source, and assertion surface; compilation or source grep alone is not a behavioral pass.
- [ ] Confirm consequential negative findings in expected modules, UI, tests, and `git log --all -S/-G` reachable history.
- [ ] Record manual evidence only from the generated capability matrix and checked review scenarios. Do not add, alter, or check a scenario in this lane.
- [ ] Ensure receipt text contains no prohibited names and does not quote confidential project records.

## Task 7: Update the Ledger Atomically and Self-Review All 62 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-por.md`

- [ ] Prepare all 62 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-POR rows and every source-owned field.
- [ ] Enforce that no POR row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 62 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every characterization is labelled, every block names its dependency, and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 341 adjudicated, 590 remaining.

## Task 8: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 62-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged, and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, source/parameter/product/manual blocks, and `590/931` rows remaining.
- [ ] Recommend `G1-DOM-SAT-P` as the next serial planning increment because saturation consumes the now-adjudicated porosity contract; do not prepare or execute it automatically.

## Task 9: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, the PRD-audit check, and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-por.md`, `docs/takeover/requirements.csv`, and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-POR adjudicate 62 SB-POR requirements`; do not push, merge, or begin a production fix.

## Plan Self-Review Before Execution

- [ ] Exactly 62 live SB-POR IDs are covered once; no row can be silently skipped.
- [ ] All seventeen P0, twenty-five P1, seventeen P2, and three P3 rows are adjudicated without treating old priority as pilot policy.
- [ ] All 41 real chapter test intentions are routed once; numeric T26/T27 remain absent; all 62 source-owned test/status fields remain blank.
- [ ] The plan changes no production behavior, test, PRD, research dossier, manual verification record, generated artifact, protected vendor data, or ledger verdict.
- [ ] The section 5 claim of 18 ABSENT parameters/rows and the mechanically visible 15 ABSENT-bearing rows remain an explicit source finding, never a guessed normalization.
- [ ] Historical chapter reconnaissance is never copied into a live verdict without current source, test, and reachability evidence.
- [ ] Module category, shared runner, shared dialog, shared outputs, shared helper, and shared curve family remain distinct obligations.
- [ ] Unlimited versus limited, PHIE versus PHIT, apparent versus effective, shale subtraction versus clay-bound water, VSH versus VCL, imported versus computed, and method versus comparison quantities remain distinct.
- [ ] Every clamp and branch is checked for both numeric behavior and the required observable flag/provenance surface.
- [ ] Current generic `sources_topic` infrastructure cannot substitute for every POR parameter carrying source, tier, visible competing values, and run custody.
- [ ] All ABSENT parameters stay absent and all NON-ADOPTABLE values stay verification-only.
- [ ] No current literal, vendor neighbor, protected chart, project precedent, average, rounding, or fixture becomes a default or source.
- [ ] The PHIE-floor chapter contract, current code, and later direct product decision remain separately visible until Jauhar adjudicates precedence.
- [ ] SP-012's clamp-versus-refusal choice and SP-013's rename-versus-RHG80 choice remain open; the plan implements neither.
- [ ] The prohibited user-visible geographic parenthetical is recorded without repeating it in new committed text.
- [ ] The Bateman-Konen constants remain unavailable for shipping until ESC-POR-8 closes.
- [ ] Average/RMS shortcuts cannot substitute for a chart-free analytic crossplot method and are checked for pay exclusion.
- [ ] `gascorr` cannot substitute for the POR HC chain; detector modules cannot substitute for POR flag consumption.
- [ ] Absence of a forbidden method never substitutes for presence of the required method.
- [ ] A generic mask, NaN, clamp, internal `Err`, or source grep cannot substitute for an observable POR refusal/flag/report.
- [ ] Tool identity is resolved through SandiBumi's own register; unresolved/ambiguous vendor tokens are never guessed.
- [ ] SSPW/SSC expected arithmetic is independently sourced rather than circularly cross-compared.
- [ ] Core comparison remains a post-check and never auto-adjusts a parameter.
- [ ] Manual evidence, automated tests, accepted Git reachability, source admissibility, product decisions, and pilot release disposition remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 341 adjudicated / 590 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution starts only after Jauhar explicitly approves this plan.
