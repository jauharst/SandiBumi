# SB-CLY Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 55 live `SB-CLY` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing clay-volume math, choosing an endpoint or transform parameter, reading protected vendor data, changing the PRD, or implementing a missing capability.

**Architecture:** This is a documentation-only evidence pass. The PRD remains the immutable statement of intended behavior and historical chapter status. Current source, qualifying observable acceptance tests, manual evidence, and reachable Git history establish a separate live verdict. One domain receipt explains every decision; `docs/takeover/requirements.csv` carries the machine-validated summary. The pass follows the complete clay-volume chain: inputs and endpoint custody, single and double indicators, transform domains, discriminator/absence semantics, combination, percentile picking, Vsh/Vcl typing and bridges, organic pre-correction, units, provenance, and LAS missing-data behavior. Generic normalization, plot-derived parameter writes, module arguments, workflow masks, unit bridges, curve families, run records, and export are supporting seams; none is promoted into a complete CLY contract unless the whole observable requirement is proved.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. This plan is written on `a62da0909a1e29c4f26061d2099401379176b4c6`; `origin/master` and the current merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`. Reverify all four before execution. If any reference moves, stop and reconcile rather than classifying against mixed trees.
- Work only in `D:\XX. SandiBumi`. Leave `D:\XX. SandiBumi-check` untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, tests, and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete CLY chapter, all applicable `docs/record_*.md` files, and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Chapter statuses are historical evidence, not current verdicts.
- All 55 source-owned `owned_tests` fields are populated. Preserve every mapping byte-for-byte. A chapter test ID counts as implemented only after locating its executable body, inspecting the assertion surface and expected-value source, and executing it.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid open-hole-petrophysics pilot. Original P0-P4 priority is evidence, not automatic pilot scope.
- A positive helper or seam does not close a compound requirement. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, an enumerated list, or a cross-layer phrase. One unsatisfied obligation makes the row `PARTIAL` or `PRESENT-DIVERGENT`, never `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence, and uses an independently sourced expected value. A helper, internal `Result`, source-text grep, compile success, or arithmetic copied from the implementation is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. The numeric limb of `SB-CLY-T14` is explicitly characterization evidence.
- A passed test is not field evidence. Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`; unchecked scenarios remain unchecked. At plan time `shale-volume` is `0 / 17`, `conditioning` is `0 / 27`, `workflow` is `0 / 23`, `generic-curve-store` is `0 / 18`, `las-export` is `0 / 2`, and `processing-history` is `0 / 7`. `histogram` is only `5 / 22` and `crossplot` only `6 / 13`; those partial plot exercises do not prove endpoint-picking custody, CLY parameter persistence, or Vsh/Vcl typing.
- The chapter front matter and live ledger agree on 55 requirements with thirteen P0, fifteen P1, nineteen P2, six P3, and two P4. Preserve the section 4 sentence saying "Fourteen are P0" as an internal PRD finding; do not silently use it as scope and do not edit the PRD in this lane.
- The chapter declares 58 parameter rows, fifteen `ABSENT - ships with no default`, and one `NON-ADOPTABLE - cited for verification`. Preserve the section 5.1 prose saying "Six values", "Four", and "two" even though its table currently contains twelve as-built rows with a different disposition split. Treat the table and row-specific chapter requirements as evidence, record the prose mismatch, and do not repair the PRD here.
- The current CLY implementation exposes only `vsh_gr` and `vsh_dn` as domain modules. Presence of two calculations does not prove the twelve-indicator, combination, endpoint-picking, bridge, typed-family, or provenance contracts.
- `vsh_gr` currently carries eight fixed selector IDs and falls through unknown selectors to linear behavior. Inspect selector enumeration, direct dispatch, saved-run compatibility, visible label, warning, run record, and unknown-value refusal separately. Do not let a valid dropdown hide a backend fallback.
- The current Larionov implementation uses rounded parity coefficients. `every_vsh_gr_transform_lands_on_its_published_coefficient` explicitly defends the non-closure at the upper boundary. It is characterization/divergence evidence for the exact-normalized contract, not correctness for `SB-CLY-004`.
- `LARINOV3` remains a shipped selector whose numeric behavior is characterized, while the chapter holds no published provenance. Its keep-warn versus remove decision is still open and saved-run compatibility matters. Adjudication must not make that product-owner decision by relabeling the current option as compliant.
- Stieber and Clavier clamps are currently hard-coded. The Stieber epsilon is an open engineering choice under chapter escalation E5; the chapter deliberately supplies no value. Do not choose one, infer one from floating-point behavior, or treat a current clamp literal as authority.
- `vsh_gr` and `vsh_dn` currently turn degenerate endpoint/geometry cases into `f32::NAN` without the complete flag, provenance token, and run-level report required by the chapter. Internal no-infinity tests are supporting safety evidence only.
- `vsh_dn` implements one specialized density-neutron rearrangement. Algebraic equivalence to the canonical bilinear form can support arithmetic, but it does not prove a shared canonical implementation, constructor, linkage semantics, source-bearing endpoints, units, flags, or reuse by every double indicator.
- `badhole` and `condflag` provide detector arithmetic and tests, but do not automatically implement per-indicator coal branching, two-sided discriminator defaults, substitution provenance, closed CLY absence tokens, or a CLY flag curve.
- The universal workflow mask blanks module inputs before execution and outputs afterward. It is not the per-indicator `Use`/absence model described by CLY, and the existing masked-repair test is a defect characterization, not proof of correct CLY substitution.
- Universal `condition::normalize` correctly has no default reference pair and `gr_normalize` delegates to it for saved-chain compatibility. The hidden legacy `gr_normalize` manifest still carries generic `20 / 120` endpoints and tests that defend them. Do not promote those legacy defaults into CLY endpoint authority. Verify current picker visibility, saved-run behavior, run-record custody, pooling window, and supplied references separately.
- Histogram and crossplot panels can write zone parameters with undoable, provenance-validated plot source notes. That is supporting infrastructure, not proof of the CLY percentile pipeline, two-way percentile/value binding, named pooling group, preset identity, transform-pole warning, or realized endpoint record.
- `ArgSpec::sources_topic`, `param_sourced`, and the parameter-source panel exist, but current CLY arguments do not carry CLY source topics. A generic provenance facility does not close `SB-CLY-050` or `SB-CLY-051` until every applicable parameter is sourced or explicitly absent and the run refuses an unsourced requirement.
- The family registry currently folds raw and normalized/corrected gamma aliases into one `GR` family and registers no distinct CLY Vsh, unclipped Vsh, Vcl, flag, or provenance families. Generic unit bridges and plot quantity checks do not by themselves establish `SB-CLY-041`, `043`, `046`, or `054`.
- LAS export now writes the project-declared sentinel through every registered writer; `-999.25` is only the cited project default, not a writer-owned invariant. Therefore the chapter's historical T35 expectation and its "passes today" note must be reverified against the newer SB-DIO contract, not copied forward.
- LAS import recognizes the standard `-999.25` and `-9999` conventions plus the file-declared null. An undeclared bare `-999` remains data unless an explicit per-channel rule declares it absent; `NoNull` deliberately preserves a matching finite value. Adjudication must keep SB-DIO's declaration discipline intact while recording whether the distinct CLY T44 warning contract is absent, divergent, or superseded. Do not weaken either domain by inventing a global `-999` rule.
- `ssc.rs` is read-only supporting evidence in this adjudication. Its duplicate GR endpoint pair and transform code may demonstrate inconsistency, but the execution must not modify it or treat its result as the CLY oracle.
- A scientific endpoint, coefficient, ratio, cutoff, percentile realization, matrix/fluid value, clean/clay point, clamp epsilon, discriminator threshold, conversion factor, or default is cited or absent. Never infer one from current code, a neighboring vendor, a local study, a plausible textbook value, or model training.
- Preserve these fifteen specified-ABSENT parameter rows exactly: `percentile_clean`, `rho_matrix`, `rho_shale`, `nphi_matrix`, `nphi_shale`, `dt_matrix_sandstone`, `dt_matrix_limestone`, `rho_matrix_limestone`, `rho_fluid_saltwater`, `dt_shale`, `sp_clean | sp_clay`, `res_clean | res_clay`, `gr_kerogen`, `csr`, and `clsr`. Preserve `rho_dry_shale` as `NON-ADOPTABLE - cited for verification`.
- The chapter's local T4 endpoint realization is project-specific verification evidence, not a global product default. The receipt may identify it by chapter section and evidence class without copying any client, field, block, basin, operator, well, or project name.
- Do not read, open, transcribe, screenshot, OCR, or redistribute protected installed-vendor chart files, help resources, templates, lookup arrays, or binary resources. The chapter's citations and recorded findings are the evidence boundary.
- Keep open items 1-16 (with retired item 11), escalations E1-E5, and refusals R1-R9 visible wherever applicable. A live verdict may name one as its blocker; it may not settle it by implication.
- Git reachability proves only that a change is in the accepted tree. Commit messages are locators, never correctness evidence; open the accepted source and test body.
- No branch switch, rebase, merge, push, PR, or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- This planning commit authorizes no ledger verdict. Execution may create only the SB-CLY receipt, adjudicate the 55 SB-CLY ledger rows, and update the dashboard after Jauhar reviews and approves this plan. It does not authorize a source fix, test addition, parameter choice, protected-data read, manual-test checkbox, or product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 55 SB-CLY rows:

```text
SB-CLY-001 SB-CLY-002 SB-CLY-003 SB-CLY-004 SB-CLY-005 SB-CLY-006 SB-CLY-007
SB-CLY-008 SB-CLY-009 SB-CLY-010 SB-CLY-011 SB-CLY-012 SB-CLY-013 SB-CLY-014
SB-CLY-015 SB-CLY-016 SB-CLY-017 SB-CLY-018 SB-CLY-019 SB-CLY-020 SB-CLY-021
SB-CLY-022 SB-CLY-023 SB-CLY-024 SB-CLY-025 SB-CLY-026 SB-CLY-027 SB-CLY-028
SB-CLY-029 SB-CLY-030 SB-CLY-031 SB-CLY-032 SB-CLY-033 SB-CLY-034 SB-CLY-035
SB-CLY-036 SB-CLY-037 SB-CLY-038 SB-CLY-039 SB-CLY-040 SB-CLY-041 SB-CLY-042
SB-CLY-043 SB-CLY-044 SB-CLY-045 SB-CLY-046 SB-CLY-047 SB-CLY-048 SB-CLY-049
SB-CLY-050 SB-CLY-051 SB-CLY-052 SB-CLY-053 SB-CLY-054 SB-CLY-055
```

At plan time all 55 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is thirteen P0, fifteen P1, nineteen P2, six P3, and two P4. Historical chapter status is thirty-four `ABSENT`, thirteen `PARTIAL`, seven `PRESENT-DIVERGENT`, and one `PRESENT-OK`. Gate 1 adjudicates all 55 because historical presence and priority do not establish current completeness, pilot reachability, or proof quality.

Run this guard before and after editing:

```powershell
$clyRows = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-CLY-*' }
$expected = 1..55 | ForEach-Object { 'SB-CLY-{0:D3}' -f $_ }
if ($clyRows.Count -ne 55) { throw "Expected 55 SB-CLY rows, found $($clyRows.Count)" }
if (@(Compare-Object $expected @($clyRows.requirement_id)).Count -ne 0) {
    throw 'The live SB-CLY ID set differs from the approved plan'
}
if (@($clyRows | Where-Object { [string]::IsNullOrWhiteSpace($_.owned_tests) }).Count -ne 0) {
    throw 'A source-owned SB-CLY owned_tests field became blank after planning'
}
if (@($clyRows | Where-Object {
    $_.as_built_status -ne 'UNADJUDICATED' -or
    $_.release_disposition -ne 'UNDECIDED' -or
    $_.risk_class -ne 'UNCLASSIFIED' -or
    $_.test_class -ne 'MISSING-OR-UNCLASSIFIED' -or
    $_.commit_state -ne 'UNVERIFIED' -or
    $_.next_action -ne 'LIVE-ADJUDICATION'
}).Count -ne 0) { throw 'A CLY verdict changed after planning; reconcile before execution' }
```

The mechanical post-execution count is exactly 279 adjudicated and 652 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-cly.md` - complete 55-row evidence receipt, including obligation-by-obligation source findings, test classification, manual evidence, history, verdict, blocker, and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 55 SB-CLY rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary, and next serial-domain handoff.

### Read-only governing inputs

- `AGENTS.md`, `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`, `03_EVIDENCE_BASE.md`, `04_CORE_REQUIREMENTS.md`, `06_SEQUENCING_AND_GATES.md`, `10_clay-volume.md`, `20_envcorr-qc.md`, `21_data-io.md`, `23_plotting-interactivity.md`, `91_REQUIREMENTS_INDEX.md`
- applicable `docs/record_*.md` files, especially `record_fixes.md`, `record_data_tools.md`, and `record_parallel_lanes.md`
- `docs/takeover/DECISIONS.md`, `CLAIMS.md`, and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json`, and `docs/VERIFICATION_MATRIX.md`
- current source, current tests, and reachable Git history

### Primary source and test inspection surfaces

- `src-tauri/src/modules.rs`, `condition.rs`, `workflow.rs`, `curves.rs`, `parsers.rs`, `export.rs`, `plotting.rs`, `equations.rs`, `param_sources.rs`, `units.rs`, `ssc.rs`
- persistence and provenance readers reached from those files; `db.rs` is read-only evidence and must not be modified
- `src/ui/moduleDialog.ts`, `paramSources.ts`, `histogramPanel.ts`, `crossplotPanel.ts`, `plotCommon.ts`, `ribbon.ts`, `workflowDialog.ts`, and history/catalog surfaces reached from them
- all Rust/TypeScript tests reached from the 44 chapter intentions

### Files this adjudication MUST NOT change

- every read-only governing input and source/test path above;
- every path under `src/`, `src-tauri/`, `tools/`, and `verification/`;
- any file under `docs/PRD_v2/` or `docs/research_2026-08/`;
- `REVIEW.md` and the generated verification matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-cly.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness, 55-row guard result, 44-test routing result, parameter-state count, and the chapter count/prose findings. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-CLY-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, chapter test intentions, parameters, and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unwired or unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source, class, and exact run result.
- Supporting evidence: helpers, internal checks, generic seams, manual matrix rows, and reachable history that do not prove the whole contract.
- Missing proof or contradiction: the exact obligation not demonstrated, including a stale chapter claim or cross-domain conflict where applicable.
- As-built verdict: one allowed ledger value, justified against every obligation.
- Release disposition and risk: one allowed ledger value, with the pilot-reachability reason kept separate from defect state.
- Commit state: accepted-anchor reachability only; never a correctness claim.
- Blocking decision: cited source, product-owner decision, protected-data boundary, manual evidence, or `NONE`.
- Next action: the smallest bounded follow-up; no adjacent implementation.
```

The receipt MUST end with measured totals, the 44-test classification table, manual-capability evidence, open source/parameter/product decisions, hard refusals, and a cross-check showing every SB-CLY row appears exactly once.

---

## Requirement Evidence Map

The table below is a routing map, not a verdict. "Candidate" means inspect it; it does not mean the requirement is satisfied.

### A. GR transforms, endpoint guards, and clamp custody (`SB-CLY-001` through `010`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Refuse and flag degenerate endpoints, never null silently | `modules.rs::vsh_gr`, `vsh_dn`; current degenerate tests; workflow result reporting | A NaN/no-infinity internal result is not the required flag, provenance token, zone/value message, or pre-write refusal. |
| `002` | Stieber as one generic shape parameter | fixed Stieber selector branches and label/arithmetic tests | Two hard-coded variants do not establish one generic `n`, one computed pole, or imported alias compatibility. |
| `003` | Resolve vendor Stieber labels by alias; fail ambiguous import | parsers/intake, saved parameter-set readers, selector import paths, history | Confirm both positive vendor-qualified resolution and the unqualified refusal; do not guess from spelling or ordinal. |
| `004` | Larionov in exact normalized form | `vsh_gr`; selector labels; transform arithmetic test | The rounded parity test explicitly preserves non-closure and is divergence evidence, not the exact form's oracle. |
| `005` | Decimal Larionov remains reachable for parity only | selector catalog, UI values/labels, saved run IDs, arithmetic test | Prove reachability and parity labeling without treating the rounded form as preferred science. |
| `006` | `LARINOV3` warning and provenance | selector, module result, run record, Processing/history surfaces | Numeric characterization does not prove a visible warning, source absence, or recorded choice. Keep E2 open. |
| `007` | Clavier across its analytic domain | hard-coded clamp and current exact/rounded boundary tests | Separate formula arithmetic, exact domain bound, unlimited output, clipped output, and provenance. |
| `008` | Curved transform | CLY module inventory, selector list, tests, reachable history | Similar or inverse code in another module is not a reachable CLY transform. |
| `009` | Transform-domain clamps computed from parameters | Stieber/Clavier branches and clamp tests | A literal bound or one fixed-parameter test cannot satisfy a parameter-derived universal claim. E5 prevents inventing epsilon. |
| `010` | Mark every clamped sample and report the fraction | CLY outputs, run-result payload, log-set metadata, export | A clipped VSH value alone loses whether it was truly one or was clamped to one. |

### B. Single and double indicators (`SB-CLY-011` through `026`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `011` | SP indicator | module catalog/dispatch, input aliases, parameter manifests | No neighboring linear two-point index counts unless the SP module is reachable and source-bound. |
| `012` | Three neutron forms, no default | module catalog, selector manifest, CLY source topics | One density-neutron double indicator is not a neutron single-indicator family. |
| `013` | Limestone-matrix precondition on neutron indicators | curve metadata, unit/reference declarations, preflight surfaces | A mnemonic or numeric range does not prove matrix-reference identity. |
| `014` | Two-sided warning on neutron clean endpoint | `vsh_dn` endpoint args, warning/flag payloads, T18 candidates | A tolerance on final indicator disagreement is not a two-sided endpoint-source warning. |
| `015` | Four resistivity forms, no default | module catalog/dispatch/options/parameters | Do not infer support from saturation resistivity equations in another domain. |
| `016` | Validate `R_clay < R_clean` before branch selection | preflight/selector dispatch/run write boundary | Refusal after choosing a branch or after writing is not the required guard. |
| `017` | Cite Coriband where used | resistivity form manifest, source topic, run record | If the form is absent, record that; do not add or invent a citation in this lane. |
| `018` | One canonical bilinear form for every double indicator | `vsh_dn`, any shared helper, unit tests, call graph | Algebraic equivalence of one branch supports arithmetic only; verify reuse by every double indicator. |
| `019` | Two-point clean line with explicit constructor | endpoint geometry helpers, UI parameterization, run record | Implicit hard-coded matrix/fluid points are not an explicit constructor with independently movable second point. |
| `020` | Link `c1`, never `c2`; doubles never link to singles | parameter state/persistence, module dialogs, zone overrides | Similar names or shared defaults do not prove edit linkage or non-linkage from both sides. |
| `021` | Refuse and report degenerate crossplot geometry | `vsh_dn` denominator guard, flag outputs, workflow result | Current all-NaN behavior must not be mistaken for the required observable refusal and written flag. |
| `022` | Refuse the printed sonic-density denominator | sonic-density inventory, canonical helper, source notes | No implementation means no accidental adoption, but absence alone does not prove a user-facing refusal if the method is offered elsewhere. |
| `023` | Thorium and Potassium indicators | module inventory, log aliases, parameters | P3 still receives a live verdict; priority is not permission to skip. |
| `024` | EM-propagation indicator with one parameter occurrence | module/UI/exported parameter inventory | Count semantic and ordinal occurrences independently; do not open protected vendor data. |
| `025` | M-N Vsh deliberately not implemented | module inventory, help/refusal surfaces, run history | The intended contract includes a recorded deliberate absence; an unknown-module error alone may be insufficient. Keep E1 open. |
| `026` | NMR clay volume typed as Vcl | module inventory, family/type registry, consuming-module guards | An arithmetic helper or generic volume curve is not typed Vcl with Vsh refusal. |

### C. Combination, absence, discriminator, and provenance (`SB-CLY-027` through `036`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `027` | Clip each contributor before combining | module inventory, combiners, run-record operation order | Clipping a final VSH in `vsh_gr` does not prove combination order. |
| `028` | Only bounded-safe combiners | module catalog/options, arithmetic helpers, fuzz/property tests | A generic mean/median elsewhere does not establish the closed CLY combiner set. |
| `029` | Zero is a value, not absence | combiner input filtering, missing checks, tests | Pin zero and missing from both sides; finite-value filtering alone can still drop zero by policy. |
| `030` | Three distinct absence states remain distinguishable | CLY output types, flag/provenance vocabulary, downstream readers | `f32::NAN` is the numeric absence representation, but one NaN cannot carry three reasons without a companion channel. |
| `031` | Every Vsh/Vcl carries a provenance curve | module outputs, curve catalog, run records, LAS export | Run-level JSON alone is not the required per-sample provenance curve. |
| `032` | Closed provenance vocabulary; substitution separate | token declarations, serializers, substitution fields/readers | A free-text note or one combined token cannot prove a closed vocabulary plus independent substitution custody. |
| `033` | Per-flag override generality | zone overrides, workflow mask, discriminator configuration | A universal mask is not one override per discriminator/indicator flag. |
| `034` | No numeric magic sentinel for rejected samples | CLY modules, parser null policy, export writer settings, T35/T44 | Preserve `f32::NAN` internally and the declared external sentinel; do not globally reinterpret undeclared `-999`. |
| `035` | Discriminator tests two-sided by default | `badhole`, `condflag`, parameter defaults/tests | One-sided washout detection or an absolute-value helper is not proof of every required discriminator direction. |
| `036` | Per-indicator coal branch with own token | `condflag` coal detector, CLY indicator dispatch, provenance outputs | Detecting coal is supporting evidence only; verify branch enable/disable, bad-hole precedence, value, and token. |

### D. Endpoint picking (`SB-CLY-037` through `042`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `037` | Complete percentile endpoint pipeline | `condition::normalize`, histogram picks, zone writes, workflow scope | Verify pooled pre-clipped data, named group, realized values, preset identity, source, run record, and consumption by CLY. |
| `038` | Two-way percentile/value binding | histogram/crossplot pick rows, parameter state, editor persistence | One-way click-to-value writing is not value-to-percentile synchronization or authority recording. |
| `039` | P3/P97 is cited and recorded as a preset | normalize manifests, hidden legacy module, run params/history | Percentile defaults alone do not prove cited preset identity, pooling group, or realized endpoints. Do not adopt a local realization globally. |
| `040` | Warn when picked endpoint reaches a transform pole | endpoint UI, transform-domain helper, run warning | Generic range validation cannot detect sample fraction inside a transform-specific clamp region. |
| `041` | Prefer corrected aliases uniformly | family registry, curve resolution, module defaults, ENV outputs | Folding raw and corrected names into one family may select a raw curve; inspect precedence and provenance, not just alias recognition. |
| `042` | Picking conventions are help, never defaults | docs rendered in module/pick panels, manifests, empty-state refusal | Helpful prose plus a populated numeric input still ships a default. |

### E. Vsh/Vcl bridge, organic correction, sources, units, and LAS (`SB-CLY-043` through `055`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `043` | Vsh and Vcl are distinct typed quantities | module args/outputs, family registry, unit/quantity checks | Similar 0-1 units or mnemonic heuristics cannot substitute for distinct quantity types. |
| `044` | Both bridge forms named; no default ratio | module catalog, selector labels, open parameters, refusal path | Do not select a ratio or migrate one form's cited value onto the other. |
| `045` | Explicit endpoint conversion identities | shared conversion helpers, unit records, run provenance | A correct numeric coincidence is not proof that the named identity and source unit were used. |
| `046` | Register all Vsh/Vcl families distinctly | `curves.rs::FAMILIES`, `family_for`, import resolution, tests | Inventory clipped Vsh, unlimited Vsh, Vcl, flag, and provenance separately; raw/corrected GR precedence is cross-support only. |
| `047` | Organic-shale pre-correction in renormalized form | module inventory, organic-volume outputs, CLY inputs | A kerogen module in another domain is not the required pre-indicator renormalization. |
| `048` | Guard the renormalization denominator | same pre-correction path and observable failures | An absent feature does not supply a guard; if no path is reachable, say so. |
| `049` | Do not iterate organic/heavy-mineral volumes inside indicator | CLY call graph, module docs/refusals, history | Confirm both the absence of iteration and the intended explicit boundary; do not infer an algorithm from vendor ambiguity. |
| `050` | Conflicting vendor values surface; no default | CLY manifests, `param_sources.rs`, module dialog, run preflight | Current uncited defaults are adverse evidence. Generic source-panel infrastructure with no CLY topics is not compliance. |
| `051` | Primary vendor artifact path is the parameter source string | CLY argument/source metadata, run record/export provenance | A chapter citation not persisted with the run does not satisfy runtime custody. Do not copy protected content. |
| `052` | Import parameters by ordinal and semantic key | parameter-set importers, aliases, round-trip tests | A mnemonic-only import or order-only import is insufficient; pin both and ambiguous/conflicting controls. |
| `053` | Matrix travel time is module-scoped and source-bearing | sonic-density parameter manifest, import/export record | A globally shared or silently inherited value violates the contract even if numerically cited elsewhere. |
| `054` | Unit-typed quantities; no magic scale constants | unit bridge registry, module arg units, conversions, provenance | Generic unit conversion supports the seam; verify each CLY quantity, source unit, conversion record, and wrong-quantity refusal. |
| `055` | LAS null discipline for every domain curve | project sentinel, writer registry, self-read, import round trip, CLY provenance curves | Reconcile the newer project-declared sentinel contract with T35; verify every CLY curve, not just GR or one computed curve. |

---

## Test-Intent Routing Guard

Every chapter intention must have one primary inspection owner so none is skipped or counted twice. Cross-support remains listed in the source-owned ledger and receipt; this table does not change `owned_tests`.

| Test | Primary row | Contract sentence routed for inspection |
|---|---|---|
| `T01` | `001` | Inverted endpoints produce no value plus an explicit invalid-endpoint token and report. |
| `T02` | `004` | Exact Larionov reaches one at the upper boundary. |
| `T03` | `005` | Rounded decimal Larionov remains reachable as parity behavior. |
| `T04` | `004` | Exact and parity rock-age forms remain distinguishable at an interior point. |
| `T05` | `002` | Generic Stieber with the cited shape reproduces the legacy first form. |
| `T06` | `002` | A non-legacy Stieber shape uses the generic formula. |
| `T07` | `009` | Stieber's clamp is derived from `n` and one named epsilon. |
| `T08` | `007` | Clavier reaches the exact analytic-domain boundary. |
| `T09` | `007` | The rounded historical Clavier clamp remains a measured divergent control. |
| `T10` | `010` | Clamped samples and their interval fraction are observable. |
| `T11` | `008` | Curved is continuous across both cited breaks. |
| `T12` | `003` | Vendor-qualified Stieber aliases resolve to the vendor-specific shape. |
| `T13` | `003` | An unqualified ambiguous Stieber label is refused. |
| `T14` | `006` | LARINOV3 warns and records provenance; its numeric limb is CHARACTERIZATION. |
| `T15` | `011` | SP two-endpoint indicator arithmetic. |
| `T16` | `012` | Three neutron forms remain distinct and none is selected by default. |
| `T17` | `013` | Missing or mixed neutron matrix reference is refused. |
| `T18` | `014` | Conflicting endpoint witnesses remain visible and the warning is two-sided. |
| `T19` | `015` | Four resistivity forms remain distinct and un-defaulted. |
| `T20` | `016` | Degenerate resistivity endpoints are refused before branch selection. |
| `T21` | `018` | Canonical bilinear and the shipped N-D rearrangement agree across units. |
| `T22` | `019` | Explicit clean-line constructor reproduces the restricted geometry. |
| `T23` | `020` | Linkage updates only allowed shared points and never singles. |
| `T24` | `021` | Degenerate double-indicator geometry writes an explicit failure channel. |
| `T25` | `022` | Corrected sonic-density form stays finite and rejects the printed sign error. |
| `T26` | `023` | Thorium and Potassium are ordinary no-default two-point indicators. |
| `T27` | `024` | EM matrix travel time appears once in UI and once in the exported record. |
| `T28` | `026` | NMR output is typed Vcl and refused where Vsh is required. |
| `T29` | `027` | Clip-then-combine differs from combine-then-clip and is recorded. |
| `T30` | `028` | Every allowed combiner remains inside contributor bounds. |
| `T31` | `029` | Legitimate zero participates in mean and median. |
| `T32` | `030` | Missing input, discriminator mask, and invalid endpoints carry distinct reasons. |
| `T33` | `031` | Full workflow emits per-sample provenance and source-bearing run parameters. |
| `T34` | `032` | Method token and substitution record remain independently readable. |
| `T35` | `055` | Rejected samples round-trip through the declared LAS null and provenance export. |
| `T36` | `035` | Under-gauge and over-gauge behavior pins the two-sided discriminator contract. |
| `T37` | `036` | Coal branch is optional, per-indicator, bad-hole-aware, and provenance-bearing. |
| `T38` | `037` | Named pooled percentile pipeline records preset, group, and realized endpoints. |
| `T39` | `038` | Percentile and value edits synchronize both ways and record authority. |
| `T40` | `040` | Endpoint picking reports the measured fraction in the clamped region. |
| `T41` | `044` | Both Vsh-to-Vcl forms are named and refuse an absent ratio. |
| `T42` | `045` | Named endpoint and unit conversions are explicit and recorded. |
| `T43` | `046` | All CLY outputs resolve to distinct families and wrong-type inputs refuse. |
| `T44` | `034` | Bare undeclared `-999` follows the chapter's documented warning contract without weakening DIO declarations. |

Rows without a unique primary test remain mandatory and use the source-owned cross-support: `017` and `051` use T33 source custody; `025` uses T33 deliberate-absence provenance; `033` uses T36 override behavior; `039` uses T38 preset custody; `041` and `043` use T43; `042`, `047`, `048`, `049`, and `052` use the relevant T33 workflow/inventory limbs; `050` uses T18-T20 conflicts; `053` uses T25; `054` uses T21/T42. These mappings do not make one test sufficient for a compound row.

Before adjudication, mechanically confirm the primary table contains exactly T01-T44 once and that every source-owned ledger mapping remains nonblank.

---

## Task 1: Freeze the Execution Tree and Create the Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-cly.md`
- Read: governing inputs and current Git state

- [ ] Confirm branch `codex/g1-sb-geo-adjudication`, a clean worktree, the accepted anchor's reachability, current `origin/master`, current merge base, and the sole worktree at `D:\XX. SandiBumi`.
- [ ] Run the 55-row guard and capture the exact priority, historical-status, owned-test, and parameter-state counts.
- [ ] Confirm `node tools/takeover-ledger.mjs --summary-json` still reports 224 adjudicated and 707 unadjudicated before any verdict edit.
- [ ] Read every governing input listed above; do not use chapter prose or this plan as a substitute for current source.
- [ ] Create all 55 receipt headings in numeric order and the final test/manual/blocker summary sections before classifying any row.
- [ ] Record the two chapter-internal count/prose findings, the current DIO sentinel conflict, the hidden legacy-normalization default, and the E1-E5 decision boundaries at the top of the receipt.

## Task 2: Adjudicate Transform and Indicator Rows `001`-`026`

**Files:**

- Modify: `docs/takeover/evidence/sb-cly.md`
- Read: `modules.rs`, parser/import paths, module UI, run persistence, relevant tests/history

- [ ] Inventory every CLY module and selector from catalog through backend dispatch and saved-run resolution. Confirm negative findings with both manifest and dispatch searches.
- [ ] Split every transform row into formula, domain, clip, unlimited output, marker, warning, label, source, saved-run, and run-record obligations as applicable.
- [ ] Run exact candidate tests for current VSH labels/arithmetic, degeneracy, flags, and UI selector persistence; classify each as correctness, characterization, supporting-only, or missing.
- [ ] For `001`, `021`, and `024`, inspect reporting surfaces outside the internal function. Do not count an internal NaN as an observable refusal.
- [ ] For `002`-`009`, keep exact published form, parity form, unproven LARINOV3, computed clamp, and engineering epsilon distinct.
- [ ] For `011`-`017` and `022`-`026`, prove absence or presence across catalog, dispatch, UI, import, family typing, tests, and reachable history; no neighboring domain substitutes.
- [ ] For `018`-`020`, independently re-derive the chapter's algebra from cited equations, then inspect implementation reuse and edit-link behavior separately.
- [ ] Record E1, E2, E3, E4, or E5 as a blocker only where the exact row depends on it; never choose the missing value or product decision.

## Task 3: Adjudicate Combination, Absence, and Provenance Rows `027`-`036`

**Files:**

- Modify: `docs/takeover/evidence/sb-cly.md`
- Read: module/workflow/flag/provenance/export paths and their tests

- [ ] Inventory any CLY combiner and contributor preprocessing; trace exact operation order and allowed method set.
- [ ] For zero/missing behavior, test or inspect positive and negative controls separately. A finite-value filter must be checked for zero retention rather than assumed safe.
- [ ] Inventory every per-sample CLY companion output and closed token vocabulary; distinguish numeric NaN from the reason channel.
- [ ] Trace substitution, discriminator, coal, bad-hole, and per-zone override custody independently from a generic workflow mask.
- [ ] Inspect run records, curve catalog, log-set metadata, Processing/history, and LAS export for the exact provenance obligations; one surface cannot stand in for all of them.
- [ ] Reconcile `034` with current DIO null rules without changing either domain. Record the source conflict and current observable behavior exactly.
- [ ] Run exact candidate tests for bad-hole direction, coal detection, mask behavior, plot-derived writes, provenance validation, and export; classify helper-level tests as supporting-only where appropriate.

## Task 4: Adjudicate Endpoint, Type, Bridge, Source, Unit, and LAS Rows `037`-`055`

**Files:**

- Modify: `docs/takeover/evidence/sb-cly.md`
- Read: normalization, plot picking, family/unit, source-panel, organic, parser/export, UI, and persistence paths

- [ ] Trace endpoint selection from source curve and pooling scope through percentile calculation, displayed value, zone parameter, run input, saved source note, realized endpoint, and downstream CLY consumption.
- [ ] Inspect both directions of percentile/value editing and which representation is authoritative; one-way zone writes are not two-way binding.
- [ ] Keep the universal no-default Normalize contract separate from the hidden legacy GR preset and its saved-chain compatibility.
- [ ] Inventory distinct Vsh, unlimited Vsh, Vcl, flag, and provenance quantities and families; confirm wrong-type refusal and corrected-alias precedence from both sides.
- [ ] Confirm whether either Vsh/Vcl bridge or organic pre-correction is reachable. If absent, still inspect for accidental magic ratios, cross-domain reuse, or unsupported iteration.
- [ ] Inventory every CLY numeric argument's default, unit, source topic, persisted source, and refusal behavior. Preserve all fifteen ABSENT rows and the one NON-ADOPTABLE row.
- [ ] For `051`-`053`, inspect import/export semantics, semantic keys, ordinals, module scoping, and source strings without opening protected artifacts.
- [ ] For `054`, distinguish generic conversion infrastructure from complete CLY quantity typing and source-unit custody.
- [ ] For `055`, run the current declared-sentinel and round-trip tests, inspect every writer's required settings, and determine how the newer DIO contract changes the chapter's historical T35 claim.

## Task 5: Classify All 44 Test Intentions, Manual Evidence, and History

**Files:**

- Modify: `docs/takeover/evidence/sb-cly.md`
- Read: all discovered tests, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md`, and reachable Git history

- [ ] Route T01-T44 exactly once through the primary table, while retaining all source-owned cross-support mappings.
- [ ] For every correctness expected value, name the chapter/public source or independently show the arithmetic. Never use implementation output as its own oracle.
- [ ] Mark T14's numeric limb `CHARACTERIZATION`; inspect whether its warning/provenance limb exists separately.
- [ ] Treat the current rounded-Larionov and rounded-Clavier controls as characterization/divergence evidence where they contradict the specified exact forms.
- [ ] Reassess T35 and T44 against current DIO behavior; never label an older fixed-sentinel assertion as current correctness without reconciling the declared-sentinel contract.
- [ ] Run every discovered candidate test by exact name. Record command, result, expected-value source, and assertion surface; compilation or source grep alone is not a behavioral pass.
- [ ] Confirm consequential negative findings in expected modules, UI, tests, and `git log --all -S/-G` reachable history.
- [ ] Record manual evidence only from the generated capability matrix and checked review scenarios. Do not add, alter, or check a scenario in this lane.
- [ ] Ensure receipt text contains no client, asset, field, block, basin, operator, well, or project name; describe the physical condition and source class instead.

## Task 6: Update the Ledger Atomically and Self-Review All 55 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-cly.md`

- [ ] Prepare all 55 RFC 4180-safe row changes as one `apply_patch`; preserve all non-SB-CLY rows and all source-owned fields.
- [ ] Enforce that no CLY row remains `UNADJUDICATED` and every adjudication-owned mandatory field is populated.
- [ ] Run `npm run check:takeover-ledger` and `node tools/takeover-ledger.mjs --check-prd-audit` to prove source-owned-field immutability.
- [ ] Cross-check all 55 receipt verdicts against the ledger: every universal claim has inventory evidence, every correctness test has an independent source, every characterization is labeled, every block names its dependency, and no manual checkbox is promoted.
- [ ] Generate the measured summary with `node tools/takeover-ledger.mjs --summary-json`. Expected mechanical count only: 279 adjudicated, 652 remaining.

## Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 55-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged, and worktree protection unchanged.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, hard source/parameter/product/manual blocks, and `652/931` rows remaining.
- [ ] Recommend `G1-DOM-POR-P` as the next serial planning increment because porosity consumes the now-adjudicated clay-volume contract; do not prepare or execute it automatically.

## Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, the PRD-audit check, and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-cly.md`, `docs/takeover/requirements.csv`, and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-CLY adjudicate 55 SB-CLY requirements`; do not push, merge, or begin a production fix.

---

## Plan Self-Review Before Execution

- [ ] Exactly 55 live SB-CLY IDs are covered once; no row can be silently skipped.
- [ ] All thirteen P0, fifteen P1, nineteen P2, six P3, and two P4 rows are adjudicated without treating old priority as pilot policy.
- [ ] All 44 chapter test intentions are routed once, and all 55 source-owned test mappings remain unchanged.
- [ ] The plan changes no production behavior, test, PRD, research dossier, manual verification record, protected vendor data, or ledger verdict.
- [ ] The section 4 P0 mismatch and section 5.1 prose/table mismatch remain findings rather than alternate scopes.
- [ ] Historical `PRESENT-OK` or `PRESENT-DIVERGENT` is never copied into a live verdict without current source, test, and reachability evidence.
- [ ] Formula, selector, label, domain, clip, marker, warning, source, saved-run, and run-record obligations remain separate.
- [ ] Rounded parity arithmetic cannot substitute for exact normalized transforms.
- [ ] LARINOV3's numeric characterization cannot substitute for a warning or published provenance, and E2 remains open.
- [ ] The Stieber epsilon remains absent until Jauhar supplies the engineering decision; no value is inferred.
- [ ] A NaN/no-infinity helper result cannot substitute for an observable refusal, companion flag, provenance token, and named run message.
- [ ] One specialized N-D implementation cannot substitute for the canonical shared double-indicator contract.
- [ ] Detector arithmetic, discriminator direction, per-indicator branch, substitution, override, and provenance remain separate obligations.
- [ ] Generic workflow masking is not the CLY per-indicator absence model.
- [ ] Zero, missing, masked, invalid-endpoint, clamped, coal, and substituted states are checked independently.
- [ ] Generic Normalize, histogram picks, crossplot writes, undo, and plot provenance remain supporting seams until the complete endpoint pipeline is proved.
- [ ] The hidden legacy GR normalization defaults are never promoted into CLY endpoint authority.
- [ ] Vsh, unlimited Vsh, Vcl, flag, and provenance quantities/families remain distinct in the analysis.
- [ ] All fifteen ABSENT parameters remain absent; the one NON-ADOPTABLE value remains verification-only.
- [ ] Every current CLY default is checked for a source string and runtime source custody; current code is never treated as a citation.
- [ ] No local realization, neighboring vendor value, duplicate endpoint pair, or protected chart/resource becomes a default.
- [ ] Project-declared LAS sentinel behavior and bare undeclared `-999` behavior are reconciled without weakening DIO or CLY.
- [ ] T14 remains partly characterization; rounded transform controls remain divergence evidence where applicable; T35/T44 are reclassified against current source rather than chapter history.
- [ ] Manual evidence, automated tests, accepted Git reachability, source admissibility, and pilot field evidence remain separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 279 adjudicated / 652 remaining result, not verdict totals.
- [ ] The planning commit changes zero ledger verdict rows; execution starts only after Jauhar explicitly approves this plan.
