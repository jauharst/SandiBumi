# SB-MIN live adjudication receipt

## Execution baseline

- Working tree: `D:\XX. SandiBumi`; branch: `codex/g1-sb-min-adjudication`; planning baseline: `f409928c594c9d1ee8407e5345a5e14014204801`.
- Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable). `origin/master` and the merge base are `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Scope: exactly 46 requirements, 44 chapter acceptance-test intentions, and 78 parameter rows from `13_mineral-solver.md`.
- Baseline ledger: 392 adjudicated / 539 unadjudicated of 931; 292 pilot blockers before this increment.
- Candidate-test receipt on the unchanged implementation tree: `multimin2::tests` 41 passed / 0 failed / 1 optional real-data test ignored; the retirement, workflow-refusal, and LAS-provenance candidates each passed. Candidate success is credited only to the observable limb proved.
- No production, test, PRD, parameter, or manual-evidence state changes in this increment.

## Governing boundaries

- A parameter is cited or remains absent. A source-code literal, vendor-neighbour value, average, remembered value, or passing snapshot is not authority.
- Four values ship while source-absent: generic `Clay` CEC `0.00`, generic `Clay` WCLP `0.120`, bound-water density `1.000`, and universal `VP/VS=1.7`. They are silent-wrongness evidence, not adopted defaults.
- `ESC-1` through `ESC-8` remain open. In particular, no agent chooses the conductivity-root convention, matched CEC/WCLP library, WBM cap, Shell constant, source-less fluid constants, clay-triple policy, per-clay `Rsh`, or universal `VP/VS`.
- A bounded solve, a correctly retired runner, generic ancestry, or a self-forward-modelled fixture is not automatically a whole-contract correctness proof.
- Manual evidence remains source-owned: SandiMin 0/28, Results QC 0/1, workflow 0/23, delivery sets 0/33, LAS export 0/2, processing history 0/7, report 6/53, security/integrity 0/63, and verification stewardship 0/24.

## Requirement receipts

### SB-MIN-001

- **Specified contract:** bounded non-negative least squares preserves every named component and never deletes one after an unconstrained negative estimate.
- **Current implementation:** `solve_bounded_lsq` and `run_multimin` keep a fixed component vector with lower/upper bounds; no mineral-deletion heuristic was found.
- **Automated evidence:** `CHARACTERIZATION`; `nonneg_holds_when_truth_is_a_boundary` and `unity_is_exact_and_bounds_hold` pass, but do not pin the specified `-0.04` unconstrained case plus returned names and count.
- **Manual evidence:** `NONE`; SandiMin remains 0/28.
- **Source/parameter and surface boundary:** mathematical structure is present; no parameter is adopted. Names/count are returned internally but lack the owned observable fixture.
- **History/reachability:** bounded-solver history is reachable from the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** the shipped behavior lacks the exact whole-contract regression proof.
- **Next action:** add the independent negative-unconstrained fixture and assert names, count, bounds, and hard unity together.

### SB-MIN-002

- **Specified contract:** the run record discloses bounded constraints and its divergence from the referenced mineral-deletion solver class.
- **Current implementation:** `MultiminResult`, log-set `params_json`, UI result copy, report, and export contain no solver-class/disclosure field.
- **Automated evidence:** `MISSING`; no observable persisted disclosure test exists.
- **Manual evidence:** `NONE`; no checked workflow or report scenario covers the disclosure.
- **Source/parameter and surface boundary:** the factual comparison remains gated by chapter `OPEN-7(e)`; an implementation comment is not a run record.
- **History/reachability:** no complete disclosure was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the solver class and qualification boundary are invisible to users and exports.
- **Next action:** add a stable solver-class field and source-qualified disclosure to run identity, result, report, and export.

### SB-MIN-003

- **Specified contract:** unity is a hard equality over non-X components, while X-only components have exactly zero coefficient in that row.
- **Current implementation:** `unity_of`, `ZoneSets`, and the bounded solve impose hard closure; the positive sum behavior exists.
- **Automated evidence:** `CHARACTERIZATION`; current unity tests prove sum and bounds, not the X-only zero-coefficient side.
- **Manual evidence:** `NONE`; SandiMin remains 0/28.
- **Source/parameter and surface boundary:** no new coefficient or tolerance is chosen; current engineering closure tolerance remains merely shipped behavior.
- **History/reachability:** unity implementation is reachable from the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** the exclusion side of the hard-equality contract has no qualifying proof.
- **Next action:** assert both non-X closure and an exact zero X-only unity coefficient in one observable fixture.

### SB-MIN-004

- **Specified contract:** every misfit statistic states whether unity was hard or represented as a Tool row.
- **Current implementation:** RECON and `dof_note` are emitted, but log-set parameters, Results QC, report, and export omit the unity convention beside them.
- **Automated evidence:** `MISSING`; no test observes the convention adjacent to all misfit outputs.
- **Manual evidence:** `NONE`; Results QC is 0/1 and report evidence is generic.
- **Source/parameter and surface boundary:** not a numeric-default question; it is missing scientific custody across result surfaces.
- **History/reachability:** no complete surface was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** a misfit can be read without knowing which closure convention produced it.
- **Next action:** persist one typed unity-convention value and render it beside every misfit statistic.

### SB-MIN-005

- **Specified contract:** fluid volume upper bounds remain structural after rename and cannot fall back to a solid bound.
- **Current implementation:** `Component.kind`, `classify`, and library `max_vol` encode 0.5, but the dialog `maxMap` and serde fallback can restore 1.0 after rename.
- **Automated evidence:** `CHARACTERIZATION`; `library_has_expected_shape` pins only the positive library row, not renamed request construction.
- **Manual evidence:** `NONE`; no checked renamed-fluid scenario exists.
- **Source/parameter and surface boundary:** the 0.500 ceiling is chapter-cited; the defect is loss of typed custody across UI/IPC.
- **History/reachability:** current kind/library and name-map behavior are integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a label edit can silently widen a physical bound.
- **Next action:** derive the ceiling from immutable component kind and test renamed plus untouched controls end to end.

### SB-MIN-006

- **Specified contract:** the CEC route applies the cited bound-water equation, salinity expansion threshold, and expansion cap.
- **Current implementation:** `fluid_calc_at` and `bound_water_multiplier` implement the CEC route and expansion arithmetic.
- **Automated evidence:** `CHARACTERIZATION`; `fluid_calc_matches_reference` and `bound_water_tracks_clay_volume` pass, but broad/current-helper expectations do not replace T05's independent `0.184106` fixture or T06's three salinity controls.
- **Manual evidence:** `NONE`; SandiMin remains 0/28.
- **Source/parameter and surface boundary:** 96, 298, 20455, and conductivity terms are cited; the `5.0` expansion cap is explicitly an engineering guard.
- **History/reachability:** the bound-water route is integrated and reachable.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** qualifying independent arithmetic coverage remains incomplete.
- **Next action:** add the cited exact fixture and 5000/500/35000 ppm controls without changing constants.

### SB-MIN-007

- **Specified contract:** the CEC route refuses a clay with absent CEC; it never interprets absence as zero, while a valid WCLP route still computes.
- **Current implementation:** generic `Clay` CEC is `0.00`; `bound_water_multiplier` accepts it as real zero and continues.
- **Automated evidence:** `MISSING`; existing WCLP tests do not prove named CEC refusal plus the valid `0.040909` control.
- **Manual evidence:** `NONE`; no refusal scenario is checked.
- **Source/parameter and surface boundary:** generic Clay CEC is `ABSENT — ships with no default`; the currently shipped zero is not authority.
- **History/reachability:** silent-zero behavior is integrated; no refusal was found in history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** missing scientific input becomes a plausible computed answer.
- **Next action:** represent absence explicitly at request resolution and refuse by clay/parameter name before solving.

### SB-MIN-008

- **Specified contract:** CEC and WCLP ship only as a matched pair from one named library within the specified relative agreement gate.
- **Current implementation:** `LIB` combines Geolog-derived CEC and Techlog-derived WCLP values; several clay rows are deliberately recorded as a cross-vendor chimera.
- **Automated evidence:** `CHARACTERIZATION`; `library_has_expected_shape` snapshots the current rows and therefore documents divergence rather than correctness.
- **Manual evidence:** `NONE`; no library-selection or mismatch scenario is checked.
- **Source/parameter and surface boundary:** `ESC-2` remains open; no agent may select Geolog, Techlog, a hybrid, or an average. Missing Chlorite/Glauconite Geolog-pair WCLP values remain absent.
- **History/reachability:** the current mixed library is integrated and reachable.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a SandiBumi-qualified matched library has not been source-adjudicated.
- **Next action:** obtain the missing primary rows, make the library identity explicit, and gate pair agreement without inventing replacements.

### SB-MIN-009

- **Specified contract:** every endpoint value carries its own source string and vendor-derived flag where applicable.
- **Current implementation:** `Component.endpoints` and `LibRow` store numeric columns without per-value source/unit/provenance; generic ancestry records only the run/module.
- **Automated evidence:** `MISSING`; generic LAS-provenance coverage cannot enumerate and verify every endpoint value.
- **Manual evidence:** `NONE`; LAS export is 0/2 and processing history 0/7.
- **Source/parameter and surface boundary:** 16 chapter rows remain vendor-derived pending `SB-CORE-005`; runtime custody is absent.
- **History/reachability:** no per-value endpoint provenance type was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** a numeric endpoint cannot be audited back to its source after selection, solve, or export.
- **Next action:** replace bare endpoint scalars with source-bearing typed values across library, editor, run record, report, and export.

### SB-MIN-010

- **Specified contract:** every clay row and emitted clay curve declares wet or dry convention, and incompatible mixtures refuse.
- **Current implementation:** component kind says only `clay`; library rows, request, curve metadata, report, and export carry no wet/dry convention or mixed-convention refusal.
- **Automated evidence:** `MISSING`; dry-clay helper tests do not observe row/curve declarations or named incompatibility refusal.
- **Manual evidence:** `NONE`; no SandiMin or export scenario is checked.
- **Source/parameter and surface boundary:** convention custody is absent; no density conversion is authorized by inference.
- **History/reachability:** no complete convention field or refusal was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** wet and dry endpoint values can be combined and exported without an observable warning.
- **Next action:** add a required convention enum to clay rows/curves and refuse a mixed set naming the conflicting rows.

### SB-MIN-011

- **Specified contract:** CEC is unit-typed as meq/g and values outside `[0.01, 2.0]` refuse with a unit hint.
- **Current implementation:** `Component.cec` and IPC/UI carry an untyped number with no magnitude validation.
- **Automated evidence:** `MISSING`; neither 16.0 nor 0.001 refusal is pinned.
- **Manual evidence:** `NONE`; no invalid-CEC scenario is checked.
- **Source/parameter and surface boundary:** the chapter marks the window `DERIVED` and `OPEN-7(a)` remains part of its provenance; this lane does not alter it.
- **History/reachability:** no unit-bearing CEC type or boundary refusal was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** a hundredfold unit error remains representable and silent.
- **Next action:** canonicalise CEC units at IPC resolution and test low/high refusal against a valid in-range control.

### SB-MIN-012

- **Specified contract:** photoelectric response is converted to volumetric U before linear mixing, never mixed directly as Pe.
- **Current implementation:** `pef_to_u` is used in response-row construction; the retired raw-Pe path was replaced.
- **Automated evidence:** `CHARACTERIZATION`; `pef_converts_to_u_before_mixing` proves the current route differs from raw Pe but constructs its target from the same live library, so it is not an independent correctness oracle.
- **Manual evidence:** `NONE`; SandiMin remains 0/28.
- **Source/parameter and surface boundary:** conversion constants are chapter-cited; no endpoint is promoted from the current library snapshot.
- **History/reachability:** commit `8fc873b` is reachable and records the R17 correction.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** the exact independent quartz/water discriminator is missing.
- **Next action:** add the cited 50/50 fixture asserting U `1.3821`, wrong-law Pe `1.085`, and separation greater than 0.25.

### SB-MIN-013

- **Specified contract:** RECON follows the stated long-form incoherence equation and is independently pinned.
- **Current implementation:** per-tool reconstruction and residual decomposition exist and reach Results QC.
- **Automated evidence:** `CHARACTERIZATION`; current RECON tests forward-model through the implementation and do not independently establish Eq 79/80 equivalence with `LargestWeight != 1`.
- **Manual evidence:** `NONE`; Results QC remains 0/1.
- **Source/parameter and surface boundary:** the equation source is stated in the chapter; the missing item is independent verification, not a new parameter.
- **History/reachability:** commits `b375d7e` and `a3cd716` are reachable.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** current tests can reproduce the same mistaken formula by construction.
- **Next action:** derive the expected long-form value outside the helper and cross-assert the observable RECON output.

### SB-MIN-014

- **Specified contract:** emit separately labelled IP-comparable and Geolog-comparable misfit statistics beside RECON.
- **Current implementation:** `TOTERR_IP` and `QUALITY` outputs are absent from result, Results QC, run record, report, and export.
- **Automated evidence:** `MISSING`; no three-statistic identity or unity-convention test exists.
- **Manual evidence:** `NONE`; Results QC remains 0/1.
- **Source/parameter and surface boundary:** cited formulas/thresholds remain in the chapter; implementation is not authorized to infer unspecified equivalence.
- **History/reachability:** no complete outputs were found in reachable history.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion of vendor-comparable diagnostics remains a product choice after core RECON safety is secured.
- **Next action:** decide pilot inclusion; if included, implement typed separately labelled statistics with independent cited fixtures.

### SB-MIN-015

- **Specified contract:** report conditioning and refuse to present a numerically unstable solve as trusted.
- **Current implementation:** no `CONDNUM` result, sample trust flag, Results-QC field, report field, or refusal path exists.
- **Automated evidence:** `MISSING`; no collinear-model fixture asserts `CONDNUM>10` and untrusted status.
- **Manual evidence:** `NONE`; no unstable-model scenario is checked.
- **Source/parameter and surface boundary:** the chapter cites the conditioning thresholds; this lane does not tune them.
- **History/reachability:** no conditioning surface was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** a plausible volume set can be displayed without numerical-trust context.
- **Next action:** compute the cited condition measure, surface trust per sample/run, and prove stable versus collinear controls.

### SB-MIN-016

- **Specified contract:** report degrees of freedom and make zero-DOF fits visibly non-validating.
- **Current implementation:** result `dof` and `dof_note` are computed and surfaced; zero-DOF receives a warning.
- **Automated evidence:** `CORRECTNESS`; `dof_note_set_when_exactly_determined`, `an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so`, and `recon_qc_emits_per_tool_curves_and_flags_endpoint_error` independently pin the exact chapter cases: `2+1-3=0` with a note and `4+1-3=2` without one.
- **Manual evidence:** `NONE`; SandiMin and Results QC manual scenarios remain unchecked.
- **Source/parameter and surface boundary:** DOF arithmetic is independently derived from the stated model dimensions; no petrophysical value is adopted.
- **History/reachability:** commit `b375d7e` is reachable.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** automated contract is proved, but paid-pilot UI interpretation still lacks manual evidence.
- **Next action:** execute the named SandiMin/Results-QC manual scenario during Gate 4; no Gate-1 code change.

### SB-MIN-017

- **Specified contract:** `tool off` and `tool weighted to zero` are distinct, with different DOF behavior and persisted state.
- **Current implementation:** the dialog omits an unchecked tool, while `ToolSpec` contains only `sigma`; no explicit active field or zero-weight state survives IPC/run custody.
- **Automated evidence:** `MISSING`; frontend omission is supporting evidence only, and no test proves inactive changes DOF while a near-zero weight retains the row.
- **Manual evidence:** `NONE`; no tool-selection scenario is checked.
- **Source/parameter and surface boundary:** no weight value is invented; the missing typed states are an integrity problem.
- **History/reachability:** no complete active/weight distinction was found in reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** UI omission cannot be audited as a distinct scientific choice after the run.
- **Next action:** add explicit `active` and weight semantics, persist both, and pin inactive versus negligible-weight controls.

### SB-MIN-018

- **Specified contract:** a per-tool weight multiplier is separate from measurement uncertainty and is recorded independently.
- **Current implementation:** `ToolSpec` exposes only `sigma`; no multiplier exists in UI, IPC, solver, run record, report, or export.
- **Automated evidence:** `MISSING`; no 1.0 no-op / 0.25 residual-change fixture exists.
- **Manual evidence:** `NONE`; no weighting scenario is checked.
- **Source/parameter and surface boundary:** the cited 1.0 Elan multiplier is not silently adopted as a product default.
- **History/reachability:** no multiplier path was found in reachable history.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot need for expert weight multipliers remains undecided.
- **Next action:** decide pilot inclusion; if included, add a typed multiplier without mutating recorded sigma.

### SB-MIN-019

- **Specified contract:** each tool stores MIN, MAX, and printed default uncertainty as independent sourced fields.
- **Current implementation:** dialog `BASE_TOOLS` contains one hard-coded sigma per tool and no source/min/max structure.
- **Automated evidence:** `MISSING`; no test pins printed deviations independently from a range rule.
- **Manual evidence:** `NONE`; no uncertainty-editor scenario is checked.
- **Source/parameter and surface boundary:** per-tool defaults are `ABSENT — ships with no default`; code literals are not promoted.
- **History/reachability:** no three-field uncertainty record was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** solver influence cannot be audited to the printed source table.
- **Next action:** source and store the three fields separately, refusing an unresolved default instead of deriving one silently.

### SB-MIN-020

- **Specified contract:** ship a sourced default tool-uncertainty library, retaining printed values and rule-derived labels.
- **Current implementation:** hard-coded `BASE_TOOLS` sigmas have no per-value source or derivation label.
- **Automated evidence:** `MISSING`; no nine-rule-row plus five-deviation library fixture exists.
- **Manual evidence:** `NONE`; no default-library review scenario is checked.
- **Source/parameter and surface boundary:** the 1.5% rule is cited, but per-tool defaults remain absent because source tables disagree.
- **History/reachability:** no qualifying sourced library was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** current source-free sigmas materially steer the solve.
- **Next action:** build a per-row sourced library that distinguishes printed and derived entries; do not overwrite disagreements.

### SB-MIN-021

- **Specified contract:** conductivity-root exponent convention is an explicit named, recorded model input.
- **Current implementation:** `fluid_calc_at` computes `w=0.75m+0.25n` implicitly; related fluid properties may persist, but the convention identity does not.
- **Automated evidence:** `CHARACTERIZATION`; `fluid_calc_matches_reference` pins the current Geolog-form behavior only, not three conventions and persisted identity.
- **Manual evidence:** `NONE`; no convention-selection scenario is checked.
- **Source/parameter and surface boundary:** `ESC-1` remains open among Geolog, Elan, and IP forms; no default is selected here.
- **History/reachability:** current exponent behavior is integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** the equation convention cannot be reconstructed from the run record.
- **Next action:** expose a required named convention, preserve all candidates without a default, and persist the resolved choice.

### SB-MIN-022

- **Specified contract:** the Shell porosity-dependent cementation constant has no default and missing input refuses.
- **Current implementation:** the Shell model/constant is not implemented.
- **Automated evidence:** `MISSING`; no missing-constant refusal or candidate-display test exists.
- **Manual evidence:** `NONE`; no Shell-method scenario is checked.
- **Source/parameter and surface boundary:** `ESC-4` preserves conflicting 0.018, 0.019, and 0.19 sources; none is selected or rounded.
- **History/reachability:** no live Shell path was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the optional model is outside the current pilot until source/product adjudication.
- **Next action:** retain the three candidates as evidence and revisit only if the Shell model enters pilot scope.

### SB-MIN-023

- **Specified contract:** implement variable `m*` with the corroborated coefficient set and source label.
- **Current implementation:** no variable-`m*` path or parameters exist in request, solver, run record, UI, or export.
- **Automated evidence:** `MISSING`; no independent `m*=2.206503` fixture exists.
- **Manual evidence:** `NONE`; no variable-exponent scenario is checked.
- **Source/parameter and surface boundary:** Dual-Water coefficients are corroborated, Waxman-Smits coefficients are single-sourced, and base `m` remains absent; no default is inferred.
- **History/reachability:** no complete implementation was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the advanced exponent route is not required for the current pilot baseline.
- **Next action:** retain source distinctions and revisit after the core fixed-exponent solver is qualified.

### SB-MIN-024

- **Specified contract:** wet/dry conversion consumes an explicit sourced bound-water density rather than a hard-coded value.
- **Current implementation:** `dry_clay_calc` hard-codes `RHO_W=1.0`; only transit time `189.0` has a cited source.
- **Automated evidence:** `CHARACTERIZATION`; dry-clay tests pin current outputs and error guards but not the explicit-density discriminator `2.831325` versus `2.821084`.
- **Manual evidence:** `NONE`; no clay-conversion scenario is checked.
- **Source/parameter and surface boundary:** bound-water density is `ABSENT — ships with no default`; the current `1.0` is not adopted.
- **History/reachability:** hard-coded density behavior is integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** conversion silently uses an uncited physical endpoint.
- **Next action:** require a sourced density in the resolved model and pin two explicit values from the chapter fixture.

### SB-MIN-025

- **Specified contract:** support and persist a per-equation invasion factor rather than one categorical global treatment.
- **Current implementation:** X/U response rows are categorical; no per-equation factor field exists.
- **Automated evidence:** `MISSING`; no mixed 0.5/1.0 equation-factor fixture exists.
- **Manual evidence:** `NONE`; no invasion-factor scenario is checked.
- **Source/parameter and surface boundary:** no default factor is invented.
- **History/reachability:** no per-equation factor was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot need for per-equation invasion scaling remains undecided.
- **Next action:** decide inclusion; if included, add a source-bearing factor to each equation row and persist it.

### SB-MIN-026

- **Specified contract:** neutron response set is a named, selected, recorded model input and never inferred from an imported header.
- **Current implementation:** NPHI endpoint values exist, but no response-set identity crosses library, editor, request, run record, report, or export.
- **Automated evidence:** `MISSING`; no two-response-set volume discriminator exists.
- **Manual evidence:** `NONE`; no response-set scenario is checked.
- **Source/parameter and surface boundary:** vendor-derived endpoint values remain source-gated by `SB-CORE-005`; no response set is named by inference.
- **History/reachability:** no complete identity field was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** different neutron response bases can produce different volumes under the same unlabeled run.
- **Next action:** define source-backed response-set IDs and carry the selected ID through every result surface.

### SB-MIN-027

- **Specified contract:** WCLP is canonical v/v; a p.u. value refuses instead of switching to the CEC route.
- **Current implementation:** `bound_water_multiplier` treats WCLP at or above 0.5 as a trigger to fall back to CEC.
- **Automated evidence:** `CHARACTERIZATION`; `wcp_degenerate_smectite_falls_back_to_cec` proves the divergent silent switch, not the specified refusal; no 10.4/0.104 paired contract test exists.
- **Manual evidence:** `NONE`; no WCLP unit-error scenario is checked.
- **Source/parameter and surface boundary:** v/v is cited; no unit conversion is guessed.
- **History/reachability:** the WCLP route from commit `627e859` is reachable.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a unit error silently changes the physical method.
- **Next action:** validate canonical v/v before route selection, refuse 10.4, and prove 0.104 yields the cited result.

### SB-MIN-028

- **Specified contract:** offer a named endpoint library and surface material disagreement between selectable libraries.
- **Current implementation:** one anonymous merged `LIB` is exposed; no second selection, identity, comparison, values, or sources appear in UI/result/export.
- **Automated evidence:** `MISSING`; current library snapshot cannot prove selection or a greater-than-5% disagreement surface.
- **Manual evidence:** `NONE`; no library-comparison scenario is checked.
- **Source/parameter and surface boundary:** `ESC-2` and `SB-CORE-005` block selection/adoption of a replacement library.
- **History/reachability:** no named multi-library surface was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** users cannot know or compare the endpoint authority driving volumes.
- **Next action:** source two coherent libraries, assign stable IDs, and show value/source disagreement before run acceptance.

### SB-MIN-029

- **Specified contract:** fluid sonic endpoints display their source at the point of use, including alternatives where the source spread matters.
- **Current implementation:** fluid `DT` values are code literals/library columns; the editor and run surfaces do not display value-level source or alternatives.
- **Automated evidence:** `CHARACTERIZATION`; library-shape tests pin literals only and do not prove source display.
- **Manual evidence:** `NONE`; no endpoint-editor scenario is checked.
- **Source/parameter and surface boundary:** water/oil/gas DT rows are vendor-derived; the gas cross-vendor spread remains evidence, not an averaged endpoint.
- **History/reachability:** integrated endpoint literals have no source-bearing UI custody.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** a source-sensitive sonic endpoint appears authoritative without its provenance.
- **Next action:** bind each displayed endpoint to its source record and expose alternatives without silently selecting one.

### SB-MIN-030

- **Specified contract:** silt is first-class, and two different Simandoux equations never share one label.
- **Current implementation:** no SandiMin silt term/Elan-Simandoux row exists; saturation helpers elsewhere use different identities without a shared canonical registry.
- **Automated evidence:** `MISSING`; no Eq 78 no-silt reduction plus displayed-identity test exists.
- **Manual evidence:** `NONE`; no silt/identity scenario is checked.
- **Source/parameter and surface boundary:** cited exponents remain available as specification evidence; no saturation equation is inferred or renamed here.
- **History/reachability:** no complete SandiMin capability was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion of this advanced silt/saturation coupling remains undecided.
- **Next action:** decide inclusion after canonical saturation identities are established; if included, model silt explicitly.

### SB-MIN-031

- **Specified contract:** saturation models can consume a per-clay shale resistivity with explicit zonal fallback.
- **Current implementation:** one zonal/global `Rsh` is used; component/clay rows have no individual `Rsh`.
- **Automated evidence:** `MISSING`; no two-clay plus fallback fixture exists.
- **Manual evidence:** `NONE`; no per-clay resistivity scenario is checked.
- **Source/parameter and surface boundary:** transcribed Techlog `Rsh` values are `NON-ADOPTABLE — cited for verification`; they are not defaults.
- **History/reachability:** no per-clay field was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** capability is deferred and its candidate values are explicitly non-adoptable.
- **Next action:** revisit only with a source-qualified model and user-supplied/cited values.

### SB-MIN-032

- **Specified contract:** persist the fully resolved parameter set so the run can be independently replayed within the stated toolchain boundary.
- **Current implementation:** log-set `params_json` stores only component names, prefix, and Sw model; `inputs_json` stores curve names. Endpoints, constraints, uncertainties, conventions, sources, and resolved defaults are omitted.
- **Automated evidence:** `MISSING`; generic log-set replay/export ancestry is supporting only and cannot reconstruct the resolved SandiMin model.
- **Manual evidence:** `NONE`; processing history is 0/7 and LAS export 0/2.
- **Source/parameter and surface boundary:** `OPEN-9` leaves cross-toolchain bit-identical tolerance undefined; same-build persistence cannot claim broader replay.
- **History/reachability:** generic log-set persistence is integrated; no complete resolved-set history was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** a delivered curve cannot be scientifically reproduced from its run record.
- **Next action:** serialize every resolved value, unit, source, convention, constraint, and route; state the replay/toolchain boundary explicitly.

### SB-MIN-033

- **Specified contract:** an infeasible constraint set names the conflicting rows and depth.
- **Current implementation:** bounded-solver and request errors are generic; they do not return row identities or sample depth for constraint conflicts.
- **Automated evidence:** `MISSING`; no PHIMAX/BVIRR conflict fixture exists.
- **Manual evidence:** `NONE`; no infeasible-model scenario is checked.
- **Source/parameter and surface boundary:** not a parameter-choice issue; structured diagnostic custody is absent from result/UI/report.
- **History/reachability:** no row-specific conflict result was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** users cannot distinguish bad data from mutually impossible constraints.
- **Next action:** return typed conflicting-row IDs and depth, then pin both conflict and feasible controls.

### SB-MIN-034

- **Specified contract:** the water-mud condition is a hard inequality iterated to feasibility and leaves already-valid samples unchanged.
- **Current implementation:** a violation triggers one re-solve with a sigma-weighted equality at zero; valid samples avoid the row, but violating samples are pulled to equality and no iterative feasibility policy exists.
- **Automated evidence:** `CHARACTERIZATION`; current constraint/default tests pin the once-only equality path, not the specified violating/unchanged pair.
- **Manual evidence:** `NONE`; no WBM constraint scenario is checked.
- **Source/parameter and surface boundary:** `ESC-3` leaves iteration cap and cap behavior unresolved; no agent invents them.
- **History/reachability:** commit `a5739c2` is reachable and the divergent path is integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** implementation requires a Jauhar-approved iteration/cap policy before it can satisfy the inequality contract.
- **Next action:** adjudicate `ESC-3`, then implement active inequality enforcement with violating and bit-unchanged valid controls.

### SB-MIN-035

- **Specified contract:** Tool constraints combine hard equality with a pseudo-measurement, emit a tie residual, and affect DOF/misfit correctly.
- **Current implementation:** POROSITY/BNDWAT ties are sigma-weighted soft rows; no separate hard equality or emitted constraint-residual curve exists.
- **Automated evidence:** `CHARACTERIZATION`; current tie tests prove the soft-row behavior only, not 1e-9 closure, residual output, DOF, and misfit together.
- **Manual evidence:** `NONE`; no Tool-constraint scenario is checked.
- **Source/parameter and surface boundary:** the cited Geolog Tool definition governs; current `SIGMA_CONSTRAINT=0.010` is not evidence of hard closure.
- **History/reachability:** commit `a5739c2` is reachable and the soft path is integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a requested hard tie can be visibly close yet scientifically violated.
- **Next action:** separate hard constraint and QC pseudo-row, emit residual, and pin closure/DOF/misfit from both sides.

### SB-MIN-036

- **Specified contract:** emit the complete named output set, no bare `SW`, and declare each clay/shale curve convention.
- **Current implementation:** volume, PHIE/PHIT, saturation, moved-hydrocarbon, VSH, RECON, REC/DIF exist; `SXOE`, `PHIE_X`, and `PHIT_X` plus convention metadata are absent. No bare `SW` is emitted.
- **Automated evidence:** `CHARACTERIZATION`; current output inventories pin the existing set and the positive no-bare-SW limb only.
- **Manual evidence:** `NONE`; delivery sets are 0/33 and LAS export 0/2.
- **Source/parameter and surface boundary:** nomenclature/custody is missing; no curve quantity is inferred from its mnemonic.
- **History/reachability:** current output emitter is integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** required X-zone quantities and wet/dry convention are lost at output.
- **Next action:** add the three named quantities with typed metadata and prove the full positive/negative output inventory.

### SB-MIN-037

- **Specified contract:** propagate endpoint uncertainty by Monte Carlo with an explicit recorded seed and reproducible bands.
- **Current implementation:** SandiMin has no endpoint Monte Carlo path; a separate MC module does not satisfy solver integration or provenance.
- **Automated evidence:** `MISSING`; no same-seed/different-seed plus persistence fixture exists.
- **Manual evidence:** `NONE`; no uncertainty-propagation scenario is checked.
- **Source/parameter and surface boundary:** Monte Carlo seed is `ABSENT — ships with no default`; CPU-clock seeding is explicitly rejected.
- **History/reachability:** no integrated SandiMin path was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** endpoint Monte Carlo is deferred beyond the deterministic pilot baseline.
- **Next action:** revisit after deterministic endpoint custody and solver correctness are qualified.

### SB-MIN-038

- **Specified contract:** report a nonnegative predicted uncertainty per solved volume with the linearisation caveat.
- **Current implementation:** no per-volume uncertainty is calculated, returned, displayed, persisted, reported, or exported.
- **Automated evidence:** `MISSING`; no scaling/nonnegative/caveat fixture exists.
- **Manual evidence:** `NONE`; no uncertainty-display scenario is checked.
- **Source/parameter and surface boundary:** the Geolog expression is cited; no unverified covariance implementation is inferred.
- **History/reachability:** no capability was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion remains undecided after conditioning and misfit controls are implemented.
- **Next action:** decide inclusion; if included, implement typed linearised uncertainty with its exclusion caveat.

### SB-MIN-039

- **Specified contract:** offer balanced pre-solve tool uncertainties and record the derived values separately from source defaults.
- **Current implementation:** no balancing calculation, preview, selection, or run-record field exists.
- **Automated evidence:** `MISSING`; no balanced-output fixture exists.
- **Manual evidence:** `NONE`; no uncertainty-balancing scenario is checked.
- **Source/parameter and surface boundary:** default uncertainties remain unresolved; balancing cannot manufacture authoritative defaults.
- **History/reachability:** no capability was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot need for expert pre-solve balancing remains undecided.
- **Next action:** decide inclusion only after SB-MIN-019/020 source custody is solved.

### SB-MIN-040

- **Specified contract:** CEC and WCLP bound-water routes are mutually exclusive and the chosen route is recorded without silent fallback.
- **Current implementation:** the request selects one route, but WCLP validation can silently fall back to CEC and the complete resolved route/inputs are not persisted.
- **Automated evidence:** `CHARACTERIZATION`; request-default and WCLP tests pin current selection/fallback behavior, not route custody and refusal.
- **Manual evidence:** `NONE`; no route-selection scenario is checked.
- **Source/parameter and surface boundary:** `ESC-2` blocks a default matched library; no route is chosen on the user's behalf.
- **History/reachability:** commits `627e859` and the CEC route history are reachable.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** the selected scientific method can change silently during evaluation.
- **Next action:** eliminate fallback, persist the explicit route and inputs, and test selected-route success plus wrong-unit refusal.

### SB-MIN-041

- **Specified contract:** retired modules remain resolvable and refuse through the real runner while exposing no unshared endpoint default.
- **Current implementation:** `modules` catalogs `multimin` and workflow refusal names SandiMin, but retired `multimin.rs` still exposes `RHOB_CLAY=2.55`, `PEF_CLAY=3.10`, and `SIG_PEF=0.30` without shared live custody/historical labels.
- **Automated evidence:** `MISSING` for the compound contract; `modules::tests::multimin_is_retired_but_still_cataloged` and the workflow refusal qualify only T40, while T41 fails on orphan defaults.
- **Manual evidence:** `NONE`; workflow remains 0/23.
- **Source/parameter and surface boundary:** orphan literals are uncited historical residue and are not promoted to live defaults.
- **History/reachability:** retirement commit `73f952d` and R17 commit `8fc873b` are reachable.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** the refusal is correct, but the rendered retired spec can still teach incompatible endpoint values.
- **Next action:** retain resolution/refusal and label or remove every unshared retired endpoint from user-visible specification surfaces.

### SB-MIN-042

- **Specified contract:** implement the oil-based-mud X/U gas constraint pair.
- **Current implementation:** no OBM inequality rows, request controls, results, run-record fields, or tests exist.
- **Automated evidence:** `MISSING`; no `v_Xgas <= v_Ugas` fixture exists.
- **Manual evidence:** `NONE`; no OBM scenario is checked.
- **Source/parameter and surface boundary:** no constraint values are guessed.
- **History/reachability:** no capability was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot mud-system coverage must be chosen before implementation.
- **Next action:** decide pilot inclusion; if included, implement the sourced inequality with violating and valid controls.

### SB-MIN-043

- **Specified contract:** offer opt-in PHIMAX, BVIRR, and IRRWAT physical ceilings and persist enabled values.
- **Current implementation:** none of the three constraints exists in solver, request, UI, run record, report, or export.
- **Automated evidence:** `MISSING`; no opt-in ceiling fixture exists.
- **Manual evidence:** `NONE`; no ceiling-constraint scenario is checked.
- **Source/parameter and surface boundary:** no ceiling value ships by default; any enabled value must be user/citation supplied.
- **History/reachability:** no capability was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion and interaction with WBM `ESC-3` remain unresolved.
- **Next action:** decide inclusion and interaction policy before implementing opt-in typed constraints.

### SB-MIN-044

- **Specified contract:** canonicalise every unit at the boundary and prove metric/imperial and unit-trap invariance.
- **Current implementation:** some local conversions exist, but component endpoints, CEC, WCLP, fluid inputs, uncertainty, UI, and IPC remain largely raw scalars without one canonical boundary.
- **Automated evidence:** `MISSING`; no metric/imperial equivalence plus four trap controls exist.
- **Manual evidence:** `NONE`; no unit-invariance scenario is checked.
- **Source/parameter and surface boundary:** units stated in the chapter are not equivalent to runtime enforcement.
- **History/reachability:** partial conversions are integrated; no complete boundary type system was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** unit mistakes can enter as plausible scalars and alter volumes silently.
- **Next action:** canonicalise at IPC/request resolution and pin both-system equivalence plus explicit wrong-unit refusals.

### SB-MIN-045

- **Specified contract:** formation temperature is bounded, invalid samples fall back deterministically, and the number of substitutions is returned and persisted.
- **Current implementation:** `FTEMP_MIN_F=32` and `FTEMP_MAX_F=600` guard samples and fallback to the scalar; substitution count is absent from result, UI, run record, report, and export.
- **Automated evidence:** `CHARACTERIZATION`; FTEMP tests pin constant/override/fallback/reconstruction behavior but not the specified 30-of-100 returned and persisted count.
- **Manual evidence:** `NONE`; no FTEMP fallback scenario is checked.
- **Source/parameter and surface boundary:** the window is an explicit engineering guard, not a petrophysical endpoint; no alternative bound is invented.
- **History/reachability:** commit `1e8a837` is reachable.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; commit state `INTEGRATED`.
- **Blocker or decision:** fallback changes scientific inputs without observable magnitude.
- **Next action:** return/persist the substitution count and pin exact fallback count against a no-fallback control.

### SB-MIN-046

- **Specified contract:** gate clay density triples for self-consistency and surface each independent inconsistency.
- **Current implementation:** no density-triple validator, editor warning/refusal, run flag, report field, or test exists.
- **Automated evidence:** `MISSING`; no three-inconsistency fixture exists.
- **Manual evidence:** `NONE`; no endpoint-consistency scenario is checked.
- **Source/parameter and surface boundary:** the 0.01 gate is `DERIVED`; `ESC-6` leaves warn/refuse adoption unresolved and source tables are not silently corrected.
- **History/reachability:** no validation path was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must choose warning versus refusal before the gate becomes product behavior.
- **Next action:** adjudicate `ESC-6`, then independently compute and surface all three cited inconsistencies.

## Acceptance-test ownership summary

- All 44 intentions are routed once: T01→001; T02→002; T03→003; T04→005; T05/T06→006; T07→007; T08→008/040; T09→009; T10→010; T11→011; T12→012; T13→013; T14→004/014; T15→015; T16→016; T17→017; T18→018; T19→019/020; T20→020/039; T21→021; T22→022; T23→023; T24→010/024; T25→025; T26→026; T27→027/040; T28→028/029; T29→029; T30→030; T31→031; T32→032; T33→033; T34→034; T35→035; T36→042/043; T37→036; T38→037; T39→038; T40/T41→041; T42→044; T43→045; T44→046.
- Whole-contract proof classification: 1 `CORRECTNESS` (016), 16 `CHARACTERIZATION` (001, 003, 005, 006, 008, 012, 013, 021, 024, 027, 029, 034, 035, 036, 040, 045), and 29 `MISSING`.
- T40's catalogue/refusal proof is retained as supporting positive evidence, but the compound requirement remains `MISSING` because T41 exposes orphan retired defaults.

## Parameter-custody appendix

- **ABSENT — ships with no default (10):** Chlorite Geolog-pair WCLP; Glauconite Geolog-pair WCLP; generic Clay CEC; generic Clay WCLP; bound-water density; Shell porosity-dependent `m` constant; variable-`m*` base exponent; per-tool default uncertainties; Monte Carlo seed; `VP/VS`.
- **NON-ADOPTABLE — cited for verification (5 rows):** competing Illite CEC; competing Kaolinite CEC; competing Glauconite CEC; per-clay `Rsh` set; Techlog clay `XWater`.
- **VENDOR-DERIVED (16 rows):** shipped Illite CEC, WCLP, and dry density; shipped Kaolinite CEC, WCLP, and dry density; Chlorite dry density; shipped Glauconite CEC, WCLP, and dry density; Montmorillonite dry density; water DT; oil DT; gas DT; oil RHOB/NPHI; gas RHOB/NPHI. These remain subject to `SB-CORE-005`.
- **All remaining chapter rows (47):** fluid and solid ceilings; Tool uncertainty; unity tolerance; active-set cap; WBM trigger; two Pe/U constants; six CEC/fluid terms; CEC plausibility window; WCLP ceiling and unit; pair tolerance/reference temperature; coherent/competing clay-pair values not listed above; bound-water transit time; conductivity exponent and guard; Arps constant; salinity fit and guard; FTEMP window; corroborated/single-source variable-`m*` coefficients; Elan-Simandoux exponents; conditioning/QUALITY/range rule; weight multiplier; MC shift/iterations; clay-triple tolerance; VP conversion; and three retired literals retained only as uncited historical evidence. Each retains its exact chapter class—cited, derived, engineering, primary-source-missing, or historical residue—and none is promoted by this receipt.
- Four absent values nevertheless ship as literals today: generic Clay CEC `0.00`, generic Clay WCLP `0.120`, bound-water density `1.000`, and `VP/VS=1.7`. Their presence is recorded as a defect, never as a default.

## Escalations, acquisition gaps, and open items

- **Escalations (8):** ESC-1 conductivity-root convention; ESC-2 matched CEC/WCLP library; ESC-3 WBM iteration/cap behavior; ESC-4 Shell constant; ESC-5 two source-less fluid constants; ESC-6 clay-density-triple response; ESC-7 per-clay `Rsh` capability/parity; ESC-8 universal `VP/VS`.
- **Acquisition gaps (11):** ACQ-1 GLOBAL parent paper; ACQ-2 Hill/Shirley/Klein clay-water source; ACQ-3 Clavier/Coates/Dumanoir; ACQ-4 Shell paper; ACQ-5 excavation source; ACQ-6 Simandoux lineage; ACQ-7 three saturation citations; ACQ-8 bounded-optimization papers; ACQ-9 geochemical-uncertainty table; ACQ-10 sonic clay-volume crossplot source; ACQ-11 primary Wyllie and Raymer-Hunt-Gardner papers.
- **Execution correction:** the committed planning artifact says 10 acquisition gaps. The live immutable chapter contains ACQ-1 through ACQ-11; this receipt uses 11 and does not rewrite the historical plan or PRD.
- **Open items (10):** OPEN-1 Geolog source ingest; OPEN-2 neutron tables; OPEN-3 global p.u./v/v convention; OPEN-4 unread Elan tables; OPEN-5 fourth endpoint table; OPEN-6 invasion/EQHY identity; OPEN-7 six read-only live-install checks; OPEN-8 stale chapter counts; OPEN-9 replay/toolchain tolerance; OPEN-10 deliberately absent constant-equation bound-water surface.
- OPEN-8 remains explicit: chapter front matter says 34 tests / 63 parameters / 9 P0, while live content is 44 / 78 / 10. The requirement count 46 is correct.

## Reachable history and manual boundary

- Reachable intent commits: `73f952d` retirement; `8fc873b` U mixing; `b375d7e` RECON/DOF; `a3cd716` Results-QC view; `a5739c2` constraints; `627e859` WCLP route; `1e8a837` FTEMP curve; `2b4a30c` core comparison; `b0a1bb8` Waxman-Smits; `857581f` dual water; `254714e` Juhasz. History explains intent but does not override live source/tests.
- No requirement receives manual credit from automation. SandiMin remains 0/28 and all adjacent capability counts remain unchanged.

## Adjudication summary

- As-built: 27 `ABSENT`, 6 `PARTIAL`, 7 `PRESENT-DIVERGENT`, 5 `PRESENT-OK`, 1 `PRESENT-UNVERIFIED`.
- Release: 34 `PILOT-BLOCKER`, 8 `UNDECIDED`, 4 `DEFERRED`.
- Risk: 17 `SILENT-WRONGNESS`, 17 `DATA-INTEGRITY`, 8 `REQUESTED-CAPABILITY`, 4 `LATER`.
- Test class: 1 `CORRECTNESS`, 16 `CHARACTERIZATION`, 29 `MISSING`.
- Manual evidence: unchanged; no requirement is field-validated by this adjudication.
