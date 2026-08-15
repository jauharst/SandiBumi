# Gate 1 SB-ENV live adjudication

- Branch: `codex/g1-sb-geo-adjudication`
- Adjudication start HEAD: `f3fd0683382738164784cf2ac7a8227bffb43cbb`
- Accepted implementation evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`
- `origin/master` at evidence freeze: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Adjudication date: `2026-08-11`
- Worktree at evidence freeze: clean; `D:\XX. SandiBumi` was the sole registered worktree.
- Row guard: passed - exactly 58 `SB-ENV` rows, IDs 001-058, all initially unadjudicated, in numeric order, with every source-owned `owned_tests` value populated.
- Evidence anchor reachability: `git merge-base --is-ancestor b332026cb498c105f36eade0bf7899bc0c1309f0 HEAD` succeeded.
- Source-navigation boundary: the codebase index was not callable in this task. Targeted source reads, `rg`, exact-filter tests and reachable Git history were the declared fallback. Negative findings were checked across the expected Rust, TypeScript and test paths.
- Protected-data boundary: no installed vendor chart, digitized chart array, descriptor, raster or proprietary lookup resource was opened. Lookup-interface findings use schema/source inventory only and require synthetic tables.
- Chapter-count findings preserved without amending the PRD: the live ledger contains 23 P0 requirements, not the stale section-4 claim of 19; section 6 contains T01-T70, not the stale claim of 68; and section 5 records 32 parameters specified ABSENT, not T07's stale 31. The earlier BHT summary is superseded by SB-ENV-047: both BHT inputs are consumed on their reachable branch.
- Parameter boundary: all 32 specified ABSENT dispositions remain absent requirements; all 29 `SHIPPED-UNCITED` findings remain source violations; all 16 `NON-ADOPTABLE` values remain verification-only. No parameter was selected or inferred in this adjudication.
- Automated-evidence boundary: 25 focused candidates were each run with an exact Cargo filter and each produced exactly one `test ... ok` line. Only the assertion surface named below is credited. T19 and T66 remain characterization intentions, T21 remains verification-only, and T68 remains source-blocked contract-only evidence.
- Manual-evidence boundary: `conditioning` is 0/27 and not exercised, `formation-temperature` is 0/0 and not recorded, `processing-history` is 0/7 and not exercised, `data-conventions` is 0/45 and not exercised, `workflow` is 0/23 and not exercised, `image-data` is 0/30 and not exercised, and `curve-editing` is 5/5 exercised. Automated tests do not close those field-evidence gaps.

## SB-ENV-001 - Declare validity conditions as data on the module spec

- **Chapter evidence:** P1; chapter status `ABSENT`; T01/T02/T03/T38; sections 4.1, 6.1 and 8.
- **Atomic obligations:** serialize enumeration, numeric/input-sample, branch-conditional and required-companion conditions; include units, human meaning and source; preserve them through saved-run data.
- **Current source:** `modules.rs::ValidityCondition` and `ValidityRule` represent enumeration, numeric/input-sample range with unit, branch-conditional range and required companions with a stable id, human statement and source. `ModuleSpec`/`ArgSpec` deserialize as well as serialize, the public runner evaluates declared conditions before module dispatch, and `moduleDialog.ts` renders their meaning/source. `workflow.rs::complete_module_log_spec` now snapshots the exact source-bearing conditions under the versioned `_sandibumi_module_validity_v1` key beside every saved module run; later registry changes therefore cannot rewrite the manifest that governed an earlier result.
- **Qualifying acceptance tests:** `workflow::tests::an_enumeration_validity_condition_survives_the_saved_run_params_json_round_trip`; `a_per_sample_numeric_range_survives_the_saved_run_with_its_unit_meaning_and_source`; `branch_specific_ranges_survive_the_saved_run_without_collapsing_to_one_module_range`; `a_required_companion_condition_survives_the_saved_run_with_the_input_it_requires`; CORRECTNESS. They use only the chapter's explicit NON-ADOPTABLE 8–13/8–18 lb/gal verification fixture and assert exact stored ids, units, meanings, sources, branches and companion input. Retaining only the first condition made the branch test RED before restoration.
- **Supporting tests:** `modules::tests::source_bearing_precondition_shapes_refuse_before_computation_while_a_valid_public_run_still_computes` continues to own direct spec round-trip, public-runner preflight and the already-populated linear-GR controls. The new four-test focused run passed 4 / 0 / 0.
- **Manual evidence:** conditioning 1/34; workflow 0/29; processing-history 0/7; all four SB-ENV-001 scenarios remain unchecked.
- **Git evidence:** current topic-branch worktree; fresh full gate 1041 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the one-requirement commit follows after the final repeated gate.
- **Verdict:** chapter status remains source-owned `ABSENT`; live as-built `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`, Visual/Manual/Field review open.
- **Blocker or decision:** none for SB-ENV-001 representation and persistence. This does not close SB-ENV-002's cross-route evaluator proof, SB-ENV-004's exhaustive source-or-ABSENT registry, or SB-ENV-008's before-run UI state.
- **Next action:** retain the versioned saved-run manifest and four shape proofs; Jauhar executes the unchecked review separately; continue serially to pilot blocker SB-ENV-002 without treating schema-valid as source-authoritative.

## SB-ENV-002 - Evaluate preconditions in the runner, before the module body

- **Chapter evidence:** P1; chapter status `ABSENT`; T02/T04/T38; sections 4.1, 6.1 and 8.
- **Atomic obligations:** evaluate every declared condition before arithmetic, per sample where needed, identically through dialog, saved chain, workflow, zone override, batch and API paths.
- **Current source:** `modules.rs::validate_declared_preconditions` evaluates the source-bearing `ValidityRule` inventory at the public `run_module` boundary before the module match/body. Numeric and relational rules consume the resolved parameter/log array at every sample. `workflow.rs::run_workflow_module_into` is the one dialog/Tauri, single-well, multi-well and zone-resolved path; `chain.rs::run_chain` delegates every saved step to it; `run_module_with_degradations` and direct callers retain the same algorithm boundary.
- **Qualifying acceptance tests:** `modules::tests::source_bearing_precondition_shapes_refuse_before_computation_while_a_valid_public_run_still_computes` owns T02's before-body/valid-other-side contract; `chain::tests::dialog_chain_batch_and_zone_override_routes_report_the_identical_precondition_refusal` owns T04. The latter checks returned `ModuleRunResult`, Processing-panel job items, saved-chain polling status, a two-well batch, one-sample named-zone arrays and zero persisted curves. CORRECTNESS. Temporarily bypassing the shared validator made the exact T04 test RED by allowing the VSH body to return blank arrays; restoration returned GREEN.
- **Supporting tests:** the T04 compile-time route inventory pins dialog → typed IPC → Tauri command → `run_workflow_module_into`; the runtime assertions do not count the direct internal `Result` as the reporting surface. T38's flag-type build gate remains independently owned by SB-ENV-030.
- **Manual evidence:** conditioning 1/38; workflow 0/33; processing-history 0/7; all four SB-ENV-002 scenarios remain unchecked.
- **Git evidence:** current topic-branch worktree; focused RED/GREEN proof complete; TypeScript and cargo check are green; fresh full gate 1042 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the one-requirement commit follows after the exact-candidate repeated gate.
- **Verdict:** chapter status remains source-owned `ABSENT`; live as-built `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`, Visual/Manual/Field review open.
- **Blocker or decision:** none for shared precondition evaluation or route parity. This does not prove that every shipping manifest is scientifically complete or source-correct; SB-ENV-003, SB-ENV-004 and SB-ENV-008 retain those distinct obligations.
- **Next action:** retain the single algorithm-boundary evaluator and route-parity proof; Jauhar executes the unchecked review separately; continue serially to SB-ENV-003 without treating identical routing as complete source population or usable UI wording.

## SB-ENV-003 - A violated precondition produces a refusal or a flagged result, never an unmarked number

- **Chapter evidence:** P0; chapter status `ABSENT`; T02-T05; sections 4.1, 6.1 and 8.
- **Atomic obligations:** name condition, offending value, expected range and source in a refusal, or emit a per-sample flag plus provenance; never emit an unmarked number; retain usable unaffected samples.
- **Current source:** `modules.rs::run_module_with_degradations` defaults to source-bearing refusal and offers an explicit `FLAG_VALID_SAMPLES` policy only for sample-resolvable numeric/relational conditions. It blanks every affected scientific input before dispatch, blanks every scientific output at those samples afterwards, returns a 0/1 companion flag and a structured condition/value/range/source record, and falls back to refusal when no unaffected sample exists. `workflow.rs::run_workflow_module_into` exposes the policy through the dialog and workflow builder, resolves the flag name through the ordinary output-name guard, reports the violation through Processing, and stores the complete affected-sample record plus policy in versioned run provenance. Set allocation, curve writes and the returned success result are controlled by finite scientific output before the finite flag is inserted.
- **Qualifying acceptance test:** `workflow::tests::a_subset_precondition_violation_keeps_only_valid_samples_with_a_companion_flag_and_source_bearing_provenance_while_refusal_stays_available_and_a_flag_alone_never_versions_an_answer`; CORRECTNESS. The fixture defaults to whole-run refusal with no write, then selects the explicit flag policy for one of three samples and asserts `[0, 1, 0]`, `[finite, NaN, finite]`, a degraded Processing item and the exact persisted source-bearing payload. Its second half uses an independently invalid negative-PHIE fixture whose scientific PERM outputs are all missing and proves an all-zero finite flag cannot allocate a set, write a curve or manufacture success.
- **Supporting tests:** T02's public-dispatch refusal, T04's four-route refusal proof and the legacy out-of-range zone rejection remain green. Replacing scientific-answer detection with `answered(outputs) || precondition_flag.is_some()` made the new T05 test RED by returning `Degraded` instead of `Failed`; restoring scientific-only detection returned GREEN. T38's typed flag metadata remains separately owned by SB-ENV-030.
- **Manual evidence:** conditioning 1/42; workflow 0/37; processing-history 0/7; the new SB-ENV-003 scenarios remain unchecked.
- **Git evidence:** current topic-branch worktree; focused RED/GREEN and mutation proof complete; TypeScript and cargo check are green; fresh full gate 1043 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the one-requirement commit follows after the exact-candidate repeated gate.
- **Verdict:** chapter status remains source-owned `ABSENT`; live as-built `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`, Visual/Manual/Field review open.
- **Blocker or decision:** none for the refusal/partial-flag contract. The companion curve is visible, named and durable, but typed flag quantity/family metadata remains an explicit SB-ENV-030 obligation and is not claimed here.
- **Next action:** retain refusal as the default and the explicit partial-result policy; Jauhar executes the unchecked review separately; continue serially to SB-ENV-004 without treating a populated flag as proof that every ENV parameter has a cited source.

## SB-ENV-004 - Every parameter carries a source string, built as one change with the validity field

- **Chapter evidence:** P0; chapter status `PARTIAL`; T06/T07; sections 4.1, 5, 6.1 and 8.
- **Atomic obligations:** every ENV parameter has a citation or explicit `ABSENT`; source and validity share schema, serialization, dialog and persistence; registry-wide build gates have zero exceptions.
- **Current source:** `ArgSpec::default_source` now carries either a named source or exact `ABSENT`; `param_open` has no concealed numeric default; `validate_parameter_sources` rejects an empty source or a value behind `ABSENT`; `moduleDialog.ts::argumentHint` renders both default source and validity conditions; and `workflow.rs::effective_module_parameters` persists the source in ancestry. Those seams were established by SB-CORE-004 and SB-ENV-001. They do not finish this requirement: `module_validity_manifest` snapshots only condition-bearing arguments and omits each argument's default source, so source and validity still do not share one `params_json` record; the chapter's internal `MIN_HAMPEL_SAMPLES` and `DEFAULT_DIVERGENCE` examples also sit outside the module `ArgSpec` registry.
- **Qualifying acceptance tests:** none. T06 requires a complete ENV-domain inventory rather than the whole-registry source gate already owned by SB-CORE-004. T07 cannot be written faithfully: its acceptance row says 31 ABSENT parameters while §5's explicit authoritative count says 32, and the chapter does not provide a canonical 32-entry ArgSpec identity list. Test class `MISSING`.
- **Supporting tests:** `modules::tests::a_registered_default_without_a_source_fails_the_build_gate` and `an_absent_required_parameter_refuses_until_the_interpreter_supplies_a_value` remain green and prove the universal registered-default mechanism. They do not prove that every §5 ENV parameter exists as an ArgSpec, that the domain inventory is complete, or that source and validity share one persisted record.
- **Manual evidence:** verification-stewardship 6/120; conditioning 1/42; workflow 0/37; processing-history 0/7; the blocked SB-ENV-004 source/specification review remains unchecked.
- **Git evidence:** current topic branch after SB-ENV-003; read-only re-verification counted exactly 29 §5 `SHIPPED-UNCITED` rows, split exactly as the chapter states into 10 remove-to-ABSENT and 19 source-required rows. The unchanged full gate remains 1043 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the blocker-only commit follows after the exact-candidate repeated gate.
- **Verdict:** chapter and live as-built remain `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`; Gate 2 `BLOCKED-SOURCE/SPEC`; Visual/Manual/Field review open.
- **Blocker or decision:** the chapter supplies no admissible citation for the 19 rows it explicitly says must acquire a source rather than be emptied. Separately, T07's 31-parameter input conflicts with §5's authoritative 32 count and no exact ArgSpec identity inventory resolves the discrepancy. CONTRACT §2 forbids inventing a source, emptying a source-required value, or guessing which row T07 excludes.
- **Next action:** publish/adjudicate the 19 named sources and reconcile T07's exact 31/32 identity set; then add the domain inventory, one combined source+validity saved-run record, and zero-exception T06/T07 without weakening the existing universal gate.

## SB-ENV-005 - A corrected curve carries the list of steps actually applied

- **Chapter evidence:** P0; chapter status `ABSENT`; T08-T10; sections 4.1, 5, 6.2, 7.1 OI-4 and 8.
- **Atomic obligations:** persist and reload every correction step, status and applied parameter value with the output curve.
- **Current source:** the complete-run writer links every computed curve to one versioned log-set record carrying the module, effective parameters, input ancestry, validity snapshot, outcomes and degradations. That is supporting infrastructure, not an applied-step manifest: neither `workflow.rs` nor `chain.rs` records the correction chain's complete step identities or one of `applied` / `unavailable` / `user-disabled` / `refused` per step, and no reader reconstructs those states after restart. `curve_meta` has no such record either.
- **Qualifying acceptance tests:** none. Exact current/test inventory found no `SB-ENV-T08`, `SB-ENV-T09`, `SB-ENV-T10`, applied-step or correction-manifest implementation. T08 also depends on actual partial correction chains owned by SB-ENV-010/011; T09's uncertainty parity is owned by deferred SB-ENV-019 and its OI-3 model decision; T10 cannot choose a persistence candidate on behalf of OI-4. Test class `MISSING`.
- **Supporting tests:** the versioned log-set, restore and ancestry tests prove that generic records survive and remain curve-linked. They do not prove step status, per-step parameter binding, a complete correction-chain inventory or post-restart applied-step retrieval.
- **Manual evidence:** conditioning 1/46; workflow 0/37; processing-history 0/7; the blocked SB-ENV-005 review remains unchecked.
- **Git evidence:** current topic branch after SB-ENV-004; read-only exact-source audit found the three OI-4 candidates but no later owner decision, step schema or executable acceptance test. The unchanged last full gate is 1043 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the blocker-only exact candidate is re-gated before commit.
- **Verdict:** chapter and live as-built remain `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 `BLOCKED-DECISION/DEPENDENCY`; Visual/Manual/Field review open.
- **Blocker or decision:** OI-4 explicitly leaves log-set archive versus run record versus per-curve metadata open and ties the choice to SB-ENV-028 and SB-ENV-042. Choosing the existing `params_json` blob merely because it is convenient would silently settle a product architecture decision. Exact T08/T09 execution also depends on correction-chain and uncertainty contracts outside this row; no numeric coefficient, measured input, chain identity or status is inferred.
- **Next action:** Jauhar selects OI-4's single persistence owner; source-complete correction chains become available; then add one typed applied-step schema to the existing atomic writer, retrieve it after restart and implement T08-T10. Until then SB-ENV-006/007 must make current unmanifested correction outputs refuse or remain outside the pilot rather than calling this row complete.

## SB-ENV-006 - A curve named "corrected" MUST have been corrected

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T11/T12; sections 4.1, 6.2 and 8.
- **Atomic obligations:** refuse a correction-named output when required input is absent, or mark every unchanged sample and manifest omission; never silently pass through under `*_EC`.
- **Current source:** `ValidityRule::RequiredWhereFinite` is a source-bearing whole-run precondition evaluated by the shared public dispatcher. `gr_hole_corr.CALI` must now be finite at every finite GR sample and `rhob_hole_corr.CALI` at every finite RHOB sample; either missing interval refuses before the private pass-through helper can return or the workflow can allocate/write `*_EC`. Complete inputs remain runnable. `nphi_env_corr` retains its documented salinity-only path because its non-zero salinity term produces a corrected result rather than an input copy.
- **Qualifying acceptance tests:** `workflow::tests::a_gr_correction_with_no_caliper_refuses_and_writes_no_uncorrected_copy` is T11 CORRECTNESS at the reporting/write surface: it requires the condition id, affected sample and contract source in the failed item, zero rows and no output identity, then adds CALI and proves the same request writes a changed `GR_EC`. `modules::tests::every_ec_module_with_a_missing_correction_input_refuses_or_changes_the_curve_and_complete_inputs_still_run` is T12 CORRECTNESS: it discovers every registered `*_EC` producer, rejects a fixture-less future producer, exercises GR/density missing-CALI refusal, exercises the changed neutron salinity-only result and requires complete-input controls to remain runnable. Both tests were RED on the former pass-through and GREEN after the public guard.
- **Supporting tests:** `env_corrections_move_the_right_way` remains explicitly CHARACTERIZATION of the private arithmetic helpers and was not deleted or weakened. The existing computed-only FTEMP regression and SB-ENV-003 partial-precondition regression stay green, proving this correction-specific whole-run guard neither consumes raw degF FTEMP nor converts the general flag policy into blanket refusal.
- **Manual evidence:** conditioning 1/50; workflow 0/41; processing-history 0/7; all four SB-ENV-006 review scenarios remain unchecked.
- **Git evidence:** current topic branch after SB-ENV-005; the two new tests produced exact RED before the manifest/runner change and targeted GREEN afterward. TypeScript and the neighbouring precondition regressions are green; the fresh full gate is 1045 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings.
- **Verdict:** live as-built is now `PRESENT-OK`; `PILOT-BLOCKER` handled; `DEGRADED-RESULT` closed by explicit refusal; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`; Visual/Manual/Field review open.
- **Blocker or decision:** none for SB-ENV-006's refusal branch. No correction coefficient, chart value, default, flag identity or persistence owner was chosen. SB-ENV-005's manifest and SB-ENV-007's per-sample four-state channel remain separate blocked/next obligations rather than being claimed by this refusal.
- **Next action:** retain the source-bearing whole-run guard and universal `*_EC` inventory; execute REVIEW.md separately; continue serially to SB-ENV-007 without calling a refusal a correction-state channel.

## SB-ENV-007 - Per-sample correction flag channel

- **Chapter evidence:** P1; chapter status `ABSENT`; T11/T13; sections 4.1, 5.2, 6.2 and 8.
- **Atomic obligations:** emit full, partial, not-applied and refused states per sample; identify the applied step set at every partial sample; use SB-ENV-030's one typed polarity contract.
- **Current source:** the three environmental-correction helpers emit only corrected numeric `Vec<f32>` curves. `ArgSpec` outputs carry name/description/unit but no flag type. SB-ENV-006 now refuses GR/density runs whenever finite source coverage is not fully matched by CALI; that closes the unmarked-copy defect but cannot exercise T13's partial-caliper result. The universal precondition companion is a binary violation curve and is not a correction-state or step-membership record.
- **Qualifying acceptance tests:** none. Exact T13 has no body. The implemented T11 refusal proves SB-ENV-006's allowed whole-run alternative, not SB-ENV-007's four-state per-sample channel. Test class `MISSING`.
- **Supporting tests:** `a_gr_correction_with_no_caliper_refuses_and_writes_no_uncorrected_copy` proves an all-uncovered input refuses without a correction-named copy. The SB-ENV-003 partial-precondition proof establishes binary `1 = violation` mechanics but carries neither correction state nor step identity.
- **Manual evidence:** conditioning 1/50; workflow 0/41; processing-history 0/7; all four SB-ENV-007 blocker-review scenarios remain unchecked and are not silently assigned to a generated capability-map row.
- **Git evidence:** current topic branch after SB-ENV-006; exact source/spec audit found no typed state output, wire encoding, per-sample step-set record or T13. The unchanged last full gate is 1045 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings; the blocker-only exact candidate is re-gated before commit.
- **Verdict:** chapter and live as-built remain `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 `BLOCKED-DECISION/DEPENDENCY`; Visual/Manual/Field review open.
- **Blocker or decision:** DEC-031 records the non-overlapping representation choice. SB-ENV-030 supplies binary `1 = true`, not four categorical wire codes; choosing arbitrary `f32` values would invent an observable file/IPC contract, while emitting four curves would contradict the singular-channel wording without approval. The partial state also requires the step identity/version and persistence owner still open under SB-ENV-005/OI-4. T13 further requires an explicit partial-execution policy rather than silently weakening SB-ENV-006's all-uncovered refusal.
- **Next action:** Jauhar selects DEC-031's binary one-hot group or categorical state channel, selects OI-4's owner and approves the partial-coverage policy; then implement the typed metadata/storage/IPC seam and exact T13 from both full and partial/refused sides. No correction coefficient or default is needed or inferred.

## SB-ENV-008 - Validity conditions are visible before the run, not only after it

- **Chapter evidence:** P2; chapter status `ABSENT`; T14; sections 4.1, 6.1 and 8.
- **Atomic obligations:** show every condition and source beside its field and pre-mark conditions that cannot be evaluated because inputs are absent.
- **Current source:** every manifest condition is now a visible field-adjacent card containing its stable id, full statement and source. `module_input_availability` resolves each scoped well through the same input-set/native/computed-only curve path as a real run, reduces the result to finite argument-name availability in Rust, and sends no curve arrays through JSON. Scope, selected mnemonic, input log set and project-data changes all refresh the card with a generation guard against stale responses.
- **Qualifying acceptance tests:** `a_missing_required_well_input_is_marked_beside_its_sourced_condition_before_the_run` is T14 CORRECTNESS. It renders the exact sourced condition for a two-state fixture, requires the missing-CALI scope to name CALI and the affected physical-condition identity before launch, then requires the finite-CALI control to say inputs are available and contain no un-evaluable text. The test also requires the live pane to call the scoped backend preflight and route it into the visible renderer.
- **Supporting tests:** the unchanged GR all-uncovered refusal and computed-only FTEMP regression both remain green after factoring their shared input resolver; frontend acceptance is 26/26. They prove the preflight did not create a different raw/computed resolution route or weaken the public guard.
- **Manual evidence:** conditioning 1/50; workflow 0/41; verification-stewardship 6/124; all four SB-ENV-008 review scenarios remain unchecked.
- **Git evidence:** current topic branch after SB-ENV-007; exact T14 was RED because no renderer/preflight function existed and GREEN after implementation. TypeScript, cargo check and focused neighbouring Rust regressions are green; the exact full gate follows before commit.
- **Verdict:** chapter status remains source-owned `ABSENT`; live as-built is now `PRESENT-OK`; `PILOT-BLOCKER` handled; `SILENT-WRONGNESS` closed at the pre-run reporting surface; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`; Visual/Manual/Field review open.
- **Blocker or decision:** none for T14. No petrophysical value, endpoint, fallback mnemonic or default was added. SB-ENV-004's missing parameter-source inventory remains separately blocked; T14 displays every source already declared by the manifest and does not claim those declarations are complete.
- **Next action:** retain the visible source/state card and exact runner-resolver preflight; Jauhar executes the unchecked visual/performance review separately; continue serially to SB-ENV-009 without treating visible metadata as proof that every method selector is backend-validated.

## SB-ENV-009 - A method-selection string that matches no known method is an error

- **Chapter evidence:** P0; chapter status `PRESENT-UNVERIFIED`; T03/T15; sections 4.1, 6.1 and 8.
- **Atomic obligations:** validate every named selector against its closed set; refuse unknown values by name; never fall through or retain prior-frame values.
- **Current source:** `modules.rs::validate_option_value` and `validate_module_options` share the closed-set check used at the direct algorithm boundary. `chain.rs::run_chain` preflights every step's closed-set selectors before `complete_chain_sets`, so an invalid later selector cannot leave an earlier step's output inside the refused saved-chain run.
- **Qualifying acceptance tests:** `modules.rs::an_unknown_method_name_is_refused_with_its_parameter_value_and_permitted_set_before_any_branch_runs` owns T03 across the registered Option inventory; `chain.rs::an_invalid_saved_chain_step_after_a_valid_step_refuses_before_any_previous_value_is_versioned` owns T15 at the saved-chain poll and persisted-output surfaces. Test class `CORRECTNESS`.
- **Supporting tests:** the existing source-bearing precondition and dialog/chain/batch/zone refusal tests remain focused neighbouring controls.
- **Manual evidence:** conditioning 1/50; workflow 0/41; four SB-ENV-009 review scenarios remain unchecked; formation-temperature 0/0 not recorded.
- **Git evidence:** exact T15 was RED as `Completed { steps_run: 2, curves_written: 2, ... }` after the valid first step, then GREEN with a failed poll payload and zero set/current/archive rows. Exact T03 is GREEN from both rejected-unknown and accepted-declared sides; the full repository gate remains a commit precondition.
- **Verdict:** source-owned chapter status remains `PRESENT-UNVERIFIED`; live as-built is `PRESENT-OK`; `PILOT-BLOCKER` handled; `SILENT-WRONGNESS` closed for registered closed-set selectors; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`; Visual/Manual/Field review open.
- **Blocker or decision:** none. No petrophysical parameter, endpoint, fallback method or default was added. This is not a claim that data-dependent saved-chain failures are transactionally atomic.
- **Next action:** retain whole-chain selector preflight plus the direct algorithm-boundary validation; execute the unchecked saved-workflow review separately; continue with the next approved `QC_CONDITIONING` row, SB-ENV-021. SB-ENV-010 remains deferred and outside Gate 2's exact 222-row scope.

## SB-ENV-010 - The GR borehole correction models hole size, mud weight, tool position and mud type

- **Chapter evidence:** P2; chapter status `PARTIAL`; T08/T16; sections 4.2, 5, 6.2 and 8.
- **Atomic obligations:** declare and apply all four term families, refuse/flag missing required terms, and record which terms entered the answer.
- **Current source:** `modules.rs::gr_hole_corr` implements only a coefficient-driven hole-enlargement term and silently passes through missing caliper; mud weight, tool position, mud type and applied-term custody are absent.
- **Qualifying acceptance tests:** none; T08/T16 have no complete body. Test class `MISSING`.
- **Supporting tests:** `env_corrections_move_the_right_way` passed for the one analytic term but its coefficients are fixture inputs and it does not prove source admissibility or term withholding.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the partial helper is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** admissible sources/inputs and the step-manifest dependency are open; no chart data may be inferred or transcribed.
- **Next action:** first implement safe term declarations/refusals and synthetic-interface tests; keep unavailable physics disabled until cited inputs exist.

## SB-ENV-011 - The neutron correction chain exposes all ten steps, and an unavailable step is reported

- **Chapter evidence:** P2; chapter status `PARTIAL`; T08/T09/T17; sections 4.2, 5, 6.2, 7.1 OI-1 and 8.
- **Atomic obligations:** expose ten independently switchable steps; report unavailable steps; keep correction and uncertainty step sets identical.
- **Current source:** `nphi_env_corr` applies only simple temperature and formation-salinity terms, without ten-step switches, unavailable-step reporting, uncertainty or manifest.
- **Qualifying acceptance tests:** none; T08/T09/T17 are missing. Test class `MISSING`.
- **Supporting tests:** the exact nominal NPHI movement test passed but asserts only the two current analytic terms.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the two-term helper is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** OI-1 leaves canonical order open; several measured inputs and correction sources are deliberately ABSENT.
- **Next action:** settle OI-1, implement a ten-state manifest with unavailable steps, and leave every uncited/measured input absent.

## SB-ENV-012 - Neutron matrix scale is a declared property of the curve and is validated at every consumer

- **Chapter evidence:** P0; chapter status `ABSENT`; T18/T19; sections 4.2, 5, 6.2 and 8.
- **Atomic obligations:** persist a closed matrix-scale enum on neutron curves; validate every matrix-dependent consumer; refuse/flag missing, unknown or mismatched scales without a default.
- **Current source:** `condflag` documents a scale assumption but curve metadata and the runner carry no neutron-scale type or validation. No consumer-wide gate exists.
- **Qualifying acceptance tests:** none; T18 is absent and T19 remains only a specified characterization intention. Test class `MISSING`.
- **Supporting tests:** `condflag` fixtures supply numeric NPHI/RHOB only and cannot observe scale metadata.
- **Manual evidence:** data-conventions 0/45; conditioning 0/27.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the enum has a cited source in section 5, but the metadata/persistence/consumer gate is absent; no numeric scale default is authorized.
- **Next action:** add the cited enum to curve metadata and refuse absent/unknown/mismatched pairs at every registered consumer.

## SB-ENV-013 - The density borehole correction models mudcake as well as hole size

- **Chapter evidence:** P2; chapter status `PARTIAL`; T20; sections 4.2, 5, 6.2 and 8.
- **Atomic obligations:** model hole size, mudcake thickness and mudcake density; make reference diameter a declared tool/bit property with no universal default.
- **Current source:** `rhob_hole_corr` implements a one-term hole-size correction with an uncited reference-diameter default; mudcake inputs and terms are absent.
- **Qualifying acceptance tests:** none; T20 is missing. Test class `MISSING`.
- **Supporting tests:** nominal directional movement passed for the one term but uses supplied fixture coefficients and cannot close the measured-property contract.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the partial helper is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** section 5 deliberately ships mudcake properties and reference diameter absent; cited sources/measurements are required.
- **Next action:** remove the universal property default, add explicit absent inputs and implement T20 only after a cited model/source is held.

## SB-ENV-014 - Correction coefficients ship with a source or ship ABSENT

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T06/T07/T21; sections 4.2, 5, 6.2, 7.2 ESC-2 and 8.
- **Atomic obligations:** every coefficient has a named edition/page/file source or no default and a missing-input refusal; vague chartbook language is inadmissible.
- **Current source:** the correction helpers ship multiple numeric coefficients described as pragmatic or chartbook-magnitude approximations without machine-readable sources. The runner accepts them and produces correction-named curves.
- **Qualifying acceptance tests:** none; T06/T07 do not exist and T21 is verification-only, not adoption authority. Test class `MISSING`.
- **Supporting tests:** directional movement proves present arithmetic, not that any coefficient is admissible.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** the uncited defaults are integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** 29 shipped-uncited parameter findings remain; ESC-2 names one missing primary source. Verification comparisons cannot become defaults.
- **Next action:** remove each uncited default or attach its chapter-authorized source, then enforce the zero-exception T06/T07 gate.

## SB-ENV-015 - The correction-chart lookup interface is specified independently of any chart data

- **Chapter evidence:** P1; chapter status `ABSENT`; T22-T24; sections 4.2, 6.2, 7.1 OI-2, 7.2 ESC-12/ESC-13, 7.3 TR-2 and 8.
- **Atomic obligations:** declare axis spans/units, interpolation and off-span policy; forbid extrapolation; flag interpolation/clamp/refusal per sample; test with synthetic data and zero protected chart data.
- **Current source:** no generic correction-chart interface exists. The current correction family is analytic; unrelated lookup code and protected chart arrays are not evidence for this contract.
- **Qualifying acceptance tests:** none; T22-T24 are missing. Test class `MISSING`.
- **Supporting tests:** no protected or digitized chart content was inspected; nominal correction movement cannot exercise lookup custody.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** OI-2 leaves axis cardinality open; ESC-12/ESC-13 and TR-2 preserve interpolation/licensing boundaries. The enforcement interface itself remains implementable with synthetic tables.
- **Next action:** decide OI-2, implement the data-free interface and synthetic span/policy tests, and keep every proprietary table absent.

## SB-ENV-016 - A measured property of the formation or the borehole ships no default

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T07/T25; sections 4.2, 5, 6.2 and 8.
- **Atomic obligations:** measured salinity, standoff, mudcake, mud weight and bit-size properties ship absent; selecting a dependent step without one refuses rather than substitutes.
- **Current source:** correction and bad-hole manifests still carry numeric defaults for measured properties, and bodies consume them when a curve/value is absent.
- **Qualifying acceptance tests:** none; T07/T25 are missing. Test class `MISSING`.
- **Supporting tests:** current arithmetic fixtures demonstrate substitution but cannot authorize the values.
- **Manual evidence:** conditioning 0/27; data-conventions 0/45; processing-history 0/7.
- **Git evidence:** the divergent defaults are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** section 5 deliberately marks these values ABSENT; no replacement number is authorized.
- **Next action:** convert every measured-property default to `param_open`/explicit input and prove term-specific refusal plus independent-term continuation.

## SB-ENV-017 - Chart baselines and intermediates are named, single-assignment quantities

- **Chapter evidence:** P1; chapter status `ABSENT`; T26; sections 4.2, 6.2, 7.2 ESC-5 and 8.
- **Atomic obligations:** assign each baseline/intermediate once and request different references by distinct names.
- **Current source:** no multi-step chart correction chain or typed intermediate model exists; the present analytic helpers cannot demonstrate this contract.
- **Qualifying acceptance tests:** none; T26 is missing. Test class `MISSING`.
- **Supporting tests:** none; numeric helper variables are not a chain-level single-assignment API.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** ESC-5 preserves the unresolved baseline interpretation; pilot inclusion of the full chart chain remains undecided.
- **Next action:** after ESC-5 and pilot scope are decided, introduce typed single-assignment intermediates before any chart values are integrated.

## SB-ENV-018 - Conditioning and correction order is a declared, checkable contract

- **Chapter evidence:** P1; chapter status `ABSENT`; T27/T28; sections 4.2, 6.2, 7.2 ESC-14 and 8.
- **Atomic obligations:** persist actual order; declare prerequisites and invalidations as data; warn with the specific violated relationship.
- **Current source:** workflow chains execute user order, but `chain.rs` records only module IDs and has no ordering contract, prerequisite/invalidation data or violation warning. Direct-run provenance omits mask/options and chain details.
- **Qualifying acceptance tests:** none; T27/T28 are missing. Test class `MISSING`.
- **Supporting tests:** chain execution/version tests prove sequence mechanics only, not semantic validity or persisted context.
- **Manual evidence:** workflow 0/23; processing-history 0/7; conditioning 0/27.
- **Git evidence:** generic chaining is integrated; the declared ordering contract is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** ESC-14 leaves the canonical pipeline placement open.
- **Next action:** settle ESC-14, add prerequisite/invalidation data to each relevant step and persist/warn on one intentionally invalid chain.

## SB-ENV-019 - Per-tool uncertainty is computed over the steps actually applied, and says which

- **Chapter evidence:** P1; chapter status `ABSENT`; T09/T29; sections 4.2, 6.2, 7.1 OI-3, 7.2 ESC-15 and 8.
- **Atomic obligations:** emit per-sample uncertainty over exactly the applied step set; declare that set; refuse a mismatched uncertainty/curve pair.
- **Current source:** no correction uncertainty output, step manifest or step-set equality guard exists.
- **Qualifying acceptance tests:** none; T09/T29 are missing. Test class `MISSING`.
- **Supporting tests:** no current test can observe an absent uncertainty surface.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** OI-3 leaves the uncertainty form open and ESC-15 leaves one model decision open; SB-ENV-005 is prerequisite.
- **Next action:** decide OI-3/ESC-15, then derive uncertainty only from the persisted applied-step set and test a deliberately mismatched pair refusal.

## SB-ENV-020 - Correction-chain QC: what did the corrections actually do?

- **Chapter evidence:** P2; chapter status `ABSENT`; T30; sections 4.2, 6.2 and 8.
- **Atomic obligations:** present uncorrected, corrected and per-step contributions in curve units, plus unavailable steps and reasons.
- **Current source:** no correction-decomposition backend payload or QC view exists; final `*_EC` curves expose no per-step contributions.
- **Qualifying acceptance tests:** none; T30 is missing. Test class `MISSING`.
- **Supporting tests:** plotting and nominal arithmetic do not provide decomposition or unavailable-step custody.
- **Manual evidence:** conditioning 0/27; processing-history 0/7; workflow 0/23.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DEGRADED-RESULT`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion of the correction-decomposition view is not yet decided; SB-ENV-005 is prerequisite.
- **Next action:** if included in the pilot, add one backend decomposition record first, then render and test applied and unavailable steps without inventing correction values.

## SB-ENV-021 - Bad-hole detection degrades to the inputs that exist, and says which it used

- **Chapter evidence:** P1; chapter status `PARTIAL`; T31/T32; sections 4.3, 6.3 and 8.
- **Atomic obligations:** evaluate caliper and density-correction terms independently; use whichever exists; return MISSING when neither is evaluable; record which terms were evaluated.
- **Current source:** `modules.rs::badhole` still evaluates the caliper and DRHO criteria independently and leaves BADHOLE MISSING when neither can run. It now declares and emits `BADHOLE_CALI_EVALUATED` and `BADHOLE_DRHO_EVALUATED` as one-hot availability records beside the unchanged BADHOLE mask.
- **Qualifying acceptance tests:** `modules.rs::a_bad_hole_flag_uses_each_available_term_records_which_was_evaluated_and_stays_missing_when_neither_was_evaluable` owns T32 from both single-input sides plus both/neither and the genuine-good-zero discriminator. Test class `CORRECTNESS`.
- **Supporting tests:** nominal bad-hole arithmetic, manifest/output-key parity, generic-store masking and complete curve ancestry remain focused green controls.
- **Manual evidence:** conditioning 1/54; workflow 0/41; processing-history 0/7; four SB-ENV-021 review scenarios remain unchecked.
- **Git evidence:** exact T32 was RED because the returned output had no evaluated-term keys, then GREEN with explicit per-sample availability and an unchanged MISSING-versus-zero distinction. The full repository gate remains a commit precondition.
- **Verdict:** source-owned chapter status remains `PARTIAL`; live as-built is `PRESENT-OK`; `PILOT-BLOCKER` handled; `DEGRADED-RESULT` closed for criterion availability; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`; Visual/Manual/Field review open.
- **Blocker or decision:** none for SB-ENV-021. No threshold or bit-size value became a default; the test values are explicit fixture inputs. SB-ENV-022's cause channel and SB-ENV-023's DRHO sign remain separate open contracts.
- **Next action:** retain the independent degradation and one-hot availability outputs; execute the unchecked review separately; continue serially to SB-ENV-022 without calling availability a diagnosis.

## SB-ENV-022 - Bad-hole flag carries a reason channel

- **Chapter evidence:** P1; chapter status `ABSENT`; T31; sections 4.3, 6.3, 7.1 OI-7 and 8.
- **Atomic obligations:** identify caliper, density correction, both or neither-evaluable per sample.
- **Current source:** `badhole` emits the numeric `BADHOLE` mask plus the SB-ENV-021 `BADHOLE_CALI_EVALUATED` and `BADHOLE_DRHO_EVALUATED` availability companions. It still discards whether each evaluated criterion actually fired; availability cannot distinguish caliper-only, DRHO-only or both causes.
- **Qualifying acceptance tests:** none. Exact T31 has no executable body, and no passing availability/arithmetic test is counted as cause-channel proof. Test class `MISSING`.
- **Supporting tests:** SB-ENV-021 proves both availability sides and the neither-evaluable MISSING distinction; nominal arithmetic proves the combined mask. Neither can recover the firing criterion from persisted outputs.
- **Manual evidence:** conditioning 1/58; workflow 0/41; processing-history 0/7; all four SB-ENV-022 blocker-review scenarios remain unchecked.
- **Git evidence:** current topic branch after SB-ENV-021; exact source/spec audit found no reason output or T31. TypeScript and cargo check are green; the fresh full gate is 1049 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings.
- **Verdict:** chapter and live as-built remain `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 `BLOCKED-DECISION`; Visual/Manual/Field review open.
- **Blocker or decision:** OI-7 requires the same encoded-curve-versus-boolean-group choice as SB-ENV-007, while DEC-031 remains open and SB-ENV-030 defines only binary `1 = true`. DEC-032 records the bad-hole-specific cause/sign state matrix. Inventing numeric reason codes or silently treating two new curves as one singular channel would create an unapproved storage/IPC/export contract.
- **Next action:** Jauhar settles DEC-031/DEC-032 by approving the typed binary group or every exact categorical wire value; then implement T31 from caliper-only, positive/negative DRHO-only, both, evaluated-good and neither-evaluable sides and prove persistence/export preserves it.

## SB-ENV-023 - The density correction's sign is preserved and reported

- **Chapter evidence:** P1; chapter status `ABSENT`; T31; sections 4.3, 6.3 and 8.
- **Atomic obligations:** preserve the sign of each density-correction exceedance in the reason output.
- **Current source:** `badhole` compares `abs(DRHO)` against the supplied threshold and collapses either sign into the same combined BADHOLE bit. The SB-ENV-021 DRHO availability companion is also 1 for either sign and cannot preserve the cause direction.
- **Qualifying acceptance tests:** none. Exact T31 has no executable equal-magnitude opposite-sign control and no signed reason output to assert. Test class `MISSING`.
- **Supporting tests:** nominal bad-hole arithmetic and SB-ENV-021 availability prove only magnitude-based alarm and criterion availability; the raw input DRHO curve is not output reason custody and cannot close this contract.
- **Manual evidence:** conditioning 1/62; workflow 0/41; processing-history 0/7; all four SB-ENV-023 blocker-review scenarios remain unchecked.
- **Git evidence:** current topic branch after blocker commit SB-ENV-022; exact source/spec audit found no signed reason representation or T31. The blocker-only candidate retains the measured 1049 passed / 0 failed / 37 ignored baseline and is re-gated before commit.
- **Verdict:** chapter and live as-built remain `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 `BLOCKED-DECISION/DEPENDENCY`; Visual/Manual/Field review open.
- **Blocker or decision:** SB-ENV-023 is an arm of DEC-032: `-1/0/+1`, a signed raw exceedance and separate positive/negative booleans are different public contracts. OI-7/DEC-031 also require the reason representation family to remain shared with SB-ENV-007. Engineering cannot choose one by implementation convenience.
- **Next action:** Jauhar settles DEC-032 together with DEC-031; then add exact T31 with equal-magnitude positive and negative DRHO-only samples, both caliper combinations, evaluated-good and neither-evaluable controls, plus persistence/export proof.

## SB-ENV-024 - Bad-hole thresholds ship ABSENT with cited presets

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T07/T33; sections 4.3, 5, 6.3, 7.2 ESC-1 and 8.
- **Atomic obligations:** ship both thresholds absent; optionally expose only named, cited presets; persist the chosen preset.
- **Current source:** `badhole_spec` already declares both thresholds through required `param_open` arguments. Each has an empty default and the exact `ABSENT` source token; public dispatch refuses before arithmetic when either threshold is absent. No named preset selector is exposed while ESC-1 is unresolved.
- **Qualifying acceptance tests:** `modules::tests::both_bad_hole_thresholds_ship_absent_and_each_must_be_explicitly_supplied_before_the_algorithm_can_run` owns the mandatory T07/T33 contract from both missing-threshold sides, then uses only the chapter-cited 0.02 g/cc and 2 in values as explicit inputs for a below/above-threshold arithmetic control. Test class `CORRECTNESS`.
- **Supporting tests:** the registry-wide sourced/default audit and generic ABSENT-parameter refusal remain supporting only; nominal bad-hole tests continue to use explicit fixture thresholds and are not authority for shipped defaults.
- **Manual evidence:** conditioning 1/66; processing-history 0/7; all four SB-ENV-024 review scenarios remain unchecked.
- **Git evidence:** current topic branch after SB-ENV-023; focused exact proof is green. TypeScript and cargo check are green; the fresh full gate is 1050 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings.
- **Verdict:** source-owned chapter status remains `PRESENT-DIVERGENT`; live as-built is now `PRESENT-OK`; `PILOT-BLOCKER` handled; `SILENT-WRONGNESS` closed for shipped defaults and both mandatory refusal sides; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 `DONE`; Visual/Manual/Field review open.
- **Blocker or decision:** none for the mandatory contract. The requirement says the application MAY offer named presets, so no preset is manufactured. ESC-1 remains open and must be answered before any named preset is added; therefore its conditional provenance obligation is not triggered by the current product.
- **Next action:** retain both required ABSENT thresholds; do not relabel explicit entry as a preset; execute the unchecked review separately and continue SB-ENV-025.

## SB-ENV-025 - Bit size is an input, never a default

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T33/T34; sections 4.3, 5, 6.3 and 8.
- **Atomic obligations:** obtain bit size from curve/header/explicit entry; never default it; report the caliper term unavailable and continue with density correction when absent.
- **Current source:** `badhole` substitutes `BS_DEF` whenever the bit-size curve is missing, so the caliper term silently runs on invented geometry.
- **Qualifying acceptance tests:** none; T33/T34 are missing. Test class `MISSING`.
- **Supporting tests:** nominal bad-hole arithmetic does not exercise absent bit size without the fallback.
- **Manual evidence:** conditioning 0/27; data-conventions 0/45.
- **Git evidence:** the divergent fallback is integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** section 5 specifies bit size ABSENT; no replacement default may be chosen.
- **Next action:** remove `BS_DEF`, make the caliper term explicitly unavailable without geometry and prove the density term still operates.

## SB-ENV-026 - DRHO's unit is declared on the curve and validated at the threshold

- **Chapter evidence:** P0; chapter status `ABSENT`; T35; sections 4.3, 6.3 and 8.
- **Atomic obligations:** persist density-correction curve units; reconcile them with threshold units; refuse missing or incompatible declarations, in both mismatch directions.
- **Current source:** the manifest labels the expected log/threshold unit, and a generic curve-unit registry exists, but `badhole` receives plain numeric arrays and never validates the actual curve unit against the threshold.
- **Qualifying acceptance tests:** none; T35 is missing. Test class `MISSING`.
- **Supporting tests:** generic unit-conversion tests do not enter the bad-hole threshold path.
- **Manual evidence:** data-conventions 0/45; conditioning 0/27.
- **Git evidence:** `UNIMPLEMENTED` at the bad-hole consumer boundary.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** no scientific value is missing; the absent piece is typed metadata propagation and refusal.
- **Next action:** carry curve units into module resolution and implement compatible conversion plus both incompatible/missing-unit refusals.

## SB-ENV-027 - A module whose purpose is to produce a value where the mask says there is none MUST be exempt from the mask

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T36/T37; sections 4.3, 6.3, 7.1 OI-5 and 8.
- **Atomic obligations:** declare a justified repair exemption; bypass both input and output mask passes; mark each reconstructed masked sample.
- **Current source:** `workflow.rs` blanks all module inputs before execution and all outputs afterward. No exemption or reconstructed-sample marker exists.
- **Qualifying acceptance tests:** `workflow::tests::a_masked_washout_defeats_the_very_module_meant_to_repair_it` passed with one actual test line. It deliberately pins the two-pass defect plus an unmasked working control, so it is `CHARACTERIZATION`.
- **Supporting tests:** ordinary mask-exclusion tests prove the general mask, not the required repair exception.
- **Manual evidence:** conditioning 0/27; workflow 0/23.
- **Git evidence:** the divergent mask runner and characterization are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** OI-5 leaves the exemption declaration shape open.
- **Next action:** settle OI-5, exempt both mask passes for declared repair modules, emit a reconstructed marker and invert the characterization into T36/T37 correctness.

## SB-ENV-028 - The mask is recorded in the run's provenance

- **Chapter evidence:** P1; chapter status `ABSENT`; T27/T28; sections 4.3, 6.3 and 8.
- **Atomic obligations:** persist the applied mask identity or explicit none so masked and unmasked outputs remain distinguishable.
- **Current source:** `MASK` is carried in request options and used by the runner, but direct-run `params_json` and `inputs_json` omit options; chain provenance records only module IDs. No persisted mask identity exists.
- **Qualifying acceptance tests:** none; T27/T28 are missing. Test class `MISSING`.
- **Supporting tests:** mask behavior and generic log-set provenance tests do not retrieve the mask after the run.
- **Manual evidence:** processing-history 0/7; conditioning 0/27; workflow 0/23.
- **Git evidence:** mask execution is integrated; provenance custody is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** no owner decision is required; the run identity is simply omitted.
- **Next action:** persist `MASK` including explicit none in direct and chain records and prove reload distinguishes otherwise identical runs.

## SB-ENV-029 - Conditioning flags validate their own stated preconditions

- **Chapter evidence:** P1; chapter status `ABSENT`; T18/T19; sections 4.3, 6.3 and 8.
- **Atomic obligations:** validate the documented neutron matrix-scale pairing before crossover arithmetic and refuse/flag a mismatch.
- **Current source:** `condflag_spec` contains a prose warning; `condflag` consumes numeric curves and matrix parameters without matrix-scale metadata or validation.
- **Qualifying acceptance tests:** none; T18 is missing and T19 remains a specified characterization only. Test class `MISSING`.
- **Supporting tests:** condflag detection tests exercise numerical branches with no scale metadata.
- **Manual evidence:** conditioning 0/27; data-conventions 0/45.
- **Git evidence:** `UNIMPLEMENTED` at the consumer precondition boundary.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-ENV-012's typed neutron-scale metadata, not on an invented numeric offset.
- **Next action:** implement the metadata contract first, then add matched, mismatched, absent and unknown scale controls at `condflag`.

## SB-ENV-030 - One flag polarity, defined once, as a type

- **Chapter evidence:** P0; chapter status `PRESENT-UNVERIFIED`; T38/T39; sections 4.3, 6.1 and 8.
- **Atomic obligations:** enforce one polarity at compile time, define it once as a type, and persist flag kind so exclusion masks and diagnostic indicators are distinct.
- **Current source:** current ENV/Condition producers consistently use numeric 1 for the flagged state, but each site constructs raw `f32` curves. No enum/newtype, central polarity definition, flag-kind metadata or validator exists.
- **Qualifying acceptance tests:** none; the promised compile-time inventory and type distinction T38/T39 are absent. Test class `MISSING`.
- **Supporting tests:** bad-hole, condition and mask tests separately assert 0/1 values; agreement by convention is not compile-time impossibility.
- **Manual evidence:** conditioning 0/27; data-conventions 0/45; workflow 0/23.
- **Git evidence:** the consistent but untyped convention is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one typed flag registry and persisted kind are missing.
- **Next action:** introduce the single polarity/type definition, migrate every ENV emitter and add a whole-registry compile/build gate plus mask/indicator control.

## SB-ENV-031 - The despike cutoff shows its contamination ceiling, live

- **Chapter evidence:** P1; chapter status `ABSENT`; T40/T69/T70; sections 4.4, 6.4, 7.2 ESC-16 and 8.
- **Atomic obligations:** display the running estimator's contamination ceiling live, show the 50 percent wall, and keep estimator-specific formulas distinct.
- **Current source:** `condition::despike` has no contamination-ceiling calculation or UI surface. The dialog exposes method/parameters only.
- **Qualifying acceptance tests:** none; T40/T69/T70 are missing. Test class `MISSING`.
- **Supporting tests:** current Hampel behavior tests do not compute or render contamination bounds.
- **Manual evidence:** conditioning 0/27.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DEGRADED-RESULT`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** ESC-16 preserves the uncited shipped threshold concern; pilot inclusion of the live ceiling remains undecided.
- **Next action:** decide pilot inclusion, then derive each estimator's ceiling from the chapter contract and render/test positive-MAD, zero-MAD and mean-sigma branches without adopting a new threshold.

## SB-ENV-032 - The MAD consistency constant is defined once, named, and cited

- **Chapter evidence:** P2; chapter status `PRESENT-DIVERGENT`; T41; sections 4.4, 5, 6.4 and 8.
- **Atomic obligations:** one named cited consistency constant is shared by every MAD consumer.
- **Current source:** the same literal appears independently in `condition.rs::window_spread` and `frame.rs`; it is neither named nor connected to a machine-readable source.
- **Qualifying acceptance tests:** none; T41 is missing. Test class `MISSING`.
- **Supporting tests:** despike/frame behavior can pass with duplicated literals and therefore cannot prove single ownership.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** duplicate literals are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the chapter supplies the required citation; no new numeric choice is needed.
- **Next action:** define one cited constant, route all consumers through it and add the whole-tree single-definition T41 gate.

## SB-ENV-033 - A degenerate window is declared, not silently substituted

- **Chapter evidence:** P2; chapter status `PRESENT-DIVERGENT`; T42; sections 4.4, 6.4 and 8.
- **Atomic obligations:** declare zero-spread and too-small-window behavior in output/provenance; never silently substitute an estimator.
- **Current source:** too-small Hampel windows refuse with an actionable message, but zero MAD silently falls back to mean absolute deviation and emits no per-run/per-sample declaration.
- **Qualifying acceptance tests:** `condition::tests::a_spike_in_a_quiet_interval_is_still_a_spike` and the narrow-window refusal passed. The first pins today's fallback and its comment identifies the divergence, so row test class is `CHARACTERIZATION`.
- **Supporting tests:** the narrow-window refusal is correct for only one degenerate branch; it cannot close silent zero-MAD substitution.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** mixed refusal/fallback behavior is integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the fallback reporting surface is absent; no parameter choice is required.
- **Next action:** make estimator substitution explicit in the result/provenance and convert T42 into two correctness controls: declared fallback and declared refusal.

## SB-ENV-034 - Every window, gap and thickness parameter is a thickness in the project's depth unit

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; T43; sections 4.4, 6.4 and 8.
- **Atomic obligations:** no sample-count windows; every conditioning/framing window, gap, bed/shoulder and filter length resolves as physical thickness against its own depth frame.
- **Current source:** Condition and Frame specs use physical-thickness parameters and their algorithms resolve depth windows from actual samples; `condflag` thickness/shoulder arithmetic also uses depth differences. The declaration tokens remain inconsistent under SB-ENV-057.
- **Qualifying acceptance tests:** none for the universal inventory; T43 is not implemented as a whole-registry gate. Test class `MISSING`.
- **Supporting tests:** `a_despike_window_covers_the_same_rock_at_any_sampling` passed and proves one resampling-invariant path, not every declaration/caller.
- **Manual evidence:** conditioning 0/27; data-conventions 0/45.
- **Git evidence:** the physical-thickness mechanisms are integrated and no sample-count ENV window was found at the accepted anchor.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** exhaustive registry proof is missing; SB-ENV-057 separately blocks token consistency.
- **Next action:** add T43 as an exhaustive declaration/behavior inventory with two samplings per operation, without changing the existing physical-width semantics.

## SB-ENV-035 - Smoothing never bridges a gap, and never invents a sample

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; T44; sections 4.4, 6.4 and 8.
- **Atomic obligations:** every smoothing/filter/average path preserves input MISSING samples; only explicit gap filling may create values.
- **Current source:** `condition::smooth` clones the input, skips missing centres and shares that preservation rule across mean, median and Savitzky-Golay branches.
- **Qualifying acceptance tests:** `condition::tests::a_smoothed_curve_never_fills_a_gap` passed exactly once and loops over all three live smoothing methods, asserting both missing preservation and a finite live-sample control. Expected behavior comes from T44; test class `CORRECTNESS`.
- **Supporting tests:** the quadratic-preservation test differentiates smoothing methods but is not needed for the gap contract.
- **Manual evidence:** conditioning 0/27.
- **Git evidence:** behavior and test are integrated at the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; automated proof passes, but conditioning field evidence remains 0/27.
- **Next action:** preserve T44 unchanged and exercise all three methods on representative pilot data before release acceptance.

## SB-ENV-036 - Outlier and spurious-population culling exists as a distinct operation

- **Chapter evidence:** P2; chapter status `ABSENT`; T27; sections 4.4, 6.2 and 8.
- **Atomic obligations:** provide population-level culling distinct from local despiking; declare cull-before-despike order; emit a reversible record.
- **Current source:** the Condition family provides despike, smooth, clip, fill, flip and normalize; no population cull operation, ordering declaration or cull recovery record exists.
- **Qualifying acceptance tests:** none; T27 is missing. Test class `MISSING`.
- **Supporting tests:** local despike tests cannot prove a distinct population operation.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion of population culling is not yet decided; SB-ENV-018/SB-ENV-037 are prerequisites.
- **Next action:** if included, specify a source-backed culling method without thresholds/defaults, then implement order and exact recovery as one increment.

## SB-ENV-037 - Every removed or replaced sample is recoverable

- **Chapter evidence:** P1; chapter status `PARTIAL`; T45; sections 4.4, 6.4 and 8.
- **Atomic obligations:** despike, cull, clip and fill each emit an exact restoration record and exercise bit-exact restore.
- **Current source:** batch Condition modules keep the input curve and may emit change flags, but do not persist original changed values or a restoration record. Interactive curve editing returns undo pairs and can restore them; the missing cull operation cannot comply.
- **Qualifying acceptance tests:** none across the operation family; T45 is missing. Test class `MISSING`.
- **Supporting tests:** `curve_edit::tests::shift_moves_curve_and_restore_undoes_it` passed for one interactive path only; retaining a separate input curve is not the required per-operation recovery record.
- **Manual evidence:** conditioning 0/27; curve-editing 5/5; processing-history 0/7.
- **Git evidence:** interactive undo is integrated; universal recovery is incomplete.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `RECOVERY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** cull is absent and batch restoration payload/persistence is missing.
- **Next action:** define one bit-exact change record for all four operation families and exercise restore after persistence, including missing values.

## SB-ENV-038 - Gap filling states its boundary comparison and refuses an open-ended gap

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; T46; sections 4.4, 6.4 and 8.
- **Atomic obligations:** document and test the exact-equality boundary; skip both open ends; measure between live anchors; flag every inserted sample.
- **Current source:** `fill_gaps_spec` says gaps no wider than the limit are filled; `fill_gaps` uses `span > max` to skip, so equality fills, rejects open-ended runs and flags inserted samples.
- **Qualifying acceptance tests:** none for the exact-boundary clause. The existing focused test passed for inside, outside and both open ends, but does not put a gap exactly on `MAX_GAP`; test class `MISSING`.
- **Supporting tests:** `fill_gaps_bridges_only_a_bounded_hole_inside_the_limit` proves the other four obligations and a flag-count control.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** behavior is integrated at the accepted anchor; proof is incomplete.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the exact-equality regression required by T46 is missing.
- **Next action:** add one exact-boundary fixture beside the existing inside/outside/open-end controls; do not change the documented less-than-or-equal behavior.

## SB-ENV-039 - Clip refuses rather than repairs

- **Chapter evidence:** P2; chapter status `PRESENT-OK`; T47; sections 4.4, 6.4 and 8.
- **Atomic obligations:** refuse no bounds and reversed pairs; preserve a genuine one-sided bound; never silently swap.
- **Current source:** `condition::clip` implements both refusals and honors one-sided bounds.
- **Qualifying acceptance tests:** `condition::tests::clipping_can_blank_or_clamp_and_an_empty_side_is_not_a_bound` passed exactly once; it asserts no-bound and reversed refusals plus one-sided and valid-pair controls. Expected behavior comes from T47; test class `CORRECTNESS`.
- **Supporting tests:** none needed beyond the two-sided control already in the test.
- **Manual evidence:** conditioning 0/27.
- **Git evidence:** behavior and test are integrated at the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; automated proof passes, but conditioning field evidence remains open.
- **Next action:** preserve T47 and field-exercise blank, clamp and one-sided modes before pilot acceptance.

## SB-ENV-040 - A conditioning output is never the input's own mnemonic

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; T48; sections 4.4, 6.4 and 8.
- **Atomic obligations:** refuse standard-mnemonic shadowing by name and reason before any module runs; also reject output collisions while allowing a safe rename.
- **Current source:** `workflow.rs::resolve_output_names` performs one pre-run check for every module and rejects shadowed, colliding and malformed output names.
- **Qualifying acceptance tests:** `workflow::tests::an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs` passed exactly once and asserts standard-name, second-output, collision and malformed refusals plus an accepted-name control. T48 is the source; test class `CORRECTNESS`.
- **Supporting tests:** per-module default-name tests support naming shape but are not needed to close refusal timing.
- **Manual evidence:** conditioning 0/27; workflow 0/23.
- **Git evidence:** behavior and test are integrated at the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; automated proof passes, but workflow/conditioning field evidence is open.
- **Next action:** preserve the central preflight and field-exercise an attempted standard-name overwrite plus a safe rename.

## SB-ENV-041 - The filter kernel and its normalisation are declared in the output

- **Chapter evidence:** P2; chapter status `PRESENT-UNVERIFIED`; T49; sections 4.4, 6.4 and 8.
- **Atomic obligations:** persist kernel, normalization, end behavior and gap-edge behavior with each smoothed output.
- **Current source:** the selected smoothing method and window reach the run request, but output/log-set provenance stores numeric parameters and input bindings while omitting option/kernel identity and the normalization/end/gap-edge policy. The curve itself carries no such record.
- **Qualifying acceptance tests:** none; T49 is missing. Test class `MISSING`.
- **Supporting tests:** smooth-method arithmetic tests prove behavior, not persistent declaration/retrieval.
- **Manual evidence:** processing-history 0/7; conditioning 0/27.
- **Git evidence:** smoothing and generic provenance are integrated, but the required output declaration is incomplete.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the option/policy record and restart retrieval are missing.
- **Next action:** persist the complete kernel policy with the output and add a restart test that differentiates two kernels sharing the same window.

## SB-ENV-042 - Interactive edits carry provenance, not only undo

- **Chapter evidence:** P1; chapter status `PARTIAL`; T45; sections 4.4, 6.4 and 8.
- **Atomic obligations:** persist operation, interval, parameters and time for every edit, retrievable without the session undo stack.
- **Current source:** `curve_edit.rs` returns byte-packed prior values for frontend undo, and `processLog` provides a UI history surface, but no durable per-curve edit record with version/content identity is stored. A stale undo can overwrite newer samples or report success without matching them.
- **Qualifying acceptance tests:** none for persistent edit provenance; T45 is missing. Test class `MISSING`.
- **Supporting tests:** exact shift/restore and `an_undo_replayed_after_the_curve_was_rewritten_splices_stale_values` both passed; the latter is explicit as-is characterization of undo's staleness, not proof of a durable audit trail.
- **Manual evidence:** curve-editing 5/5 exercised; processing-history 0/7 not exercised.
- **Git evidence:** undo and process UI are integrated; persistent edit provenance is absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** durable edit identity/history and its observable test are missing.
- **Next action:** write an immutable per-edit record tied to curve version/content, retrieve it after restart and make stale undo refuse rather than splice.

## SB-ENV-043 - One formation-temperature definition, one mnemonic

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; SB-CORE-T23 and T50/T51; sections 4.5, 6.5 and 8.
- **Atomic obligations:** exactly one temperature implementation owns `FTEMP`; every legacy entry delegates; no two independent paths emit the same mnemonic.
- **Current source:** `modules.rs::ftemp_grad` and `modules.rs::precalc` are separately dispatched implementations and both emit `FTEMP`; neither delegates. Their manifests carry independent defaults and depth semantics.
- **Qualifying acceptance tests:** none; T50/T51 and the no-parameter duplicate-producer gate are not implemented for the live registry. Test class `MISSING`.
- **Supporting tests:** both formation-temperature anchor tests passed on their own supplied parameters; shared equation fixtures cannot detect divergent defaults or ownership.
- **Manual evidence:** formation-temperature 0/0 and not recorded; workflow 0/23.
- **Git evidence:** both producers are integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one canonical producer/delegation decision remains; existing defaults cannot select the winner because several are uncited.
- **Next action:** choose the canonical contract from cited requirements, delegate the legacy ID without breaking saved chains and add the registry ownership plus convergence controls.

## SB-ENV-044 - Formation temperature is a function of true vertical depth

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T51/T52; sections 4.5, 6.5 and 8.
- **Atomic obligations:** evaluate the geotherm on true vertical depth; refuse or visibly flag measured-depth substitution.
- **Current source:** `ftemp_grad` always reads measured `DEPTH`; `precalc` uses TVDSS only when any finite TVDSS exists and silently falls back to the whole measured-depth curve otherwise. Neither reports substitution.
- **Qualifying acceptance tests:** none; T51/T52 are missing. Test class `MISSING`.
- **Supporting tests:** `precalc_rmf_trend_and_depth_fallback` passed and intentionally pins silent measured-depth fallback, but it does not label itself as the required reported substitution.
- **Manual evidence:** formation-temperature 0/0 and not recorded; workflow 0/23; data-conventions 0/45.
- **Git evidence:** both divergent depth paths are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** canonical producer work under SB-ENV-043 is prerequisite; no substitute depth value is needed.
- **Next action:** make TVD the required canonical input and exercise explicit refusal/flag plus measured-depth and TVD controls on a deviated trajectory.

## SB-ENV-045 - The geothermal gradient carries a declared, validated compound unit

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T52/T53; sections 4.5, 6.5 and 8.
- **Atomic obligations:** declare temperature and length units together; validate length denominator against project depth; reject a bare or mismatched gradient.
- **Current source:** the two temperature manifests use different compound-unit strings. The runner carries `DepthUnit` but does not parse or reconcile the gradient denominator; both bodies multiply bare numeric gradients by native depth.
- **Qualifying acceptance tests:** none; metric/foot equivalence and both mismatch refusals T52/T53 are absent. Test class `MISSING`.
- **Supporting tests:** generic project/file depth-unit tests passed, but no gradient consumer uses that conversion path.
- **Manual evidence:** formation-temperature 0/0 and not recorded; data-conventions 0/45.
- **Git evidence:** divergent declarations and unchecked arithmetic are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one canonical unit representation is required; no numeric gradient may be inferred from current defaults.
- **Next action:** introduce a typed compound unit, validate/convert against project depth and test equivalent metric/foot fixtures plus both mismatches.

## SB-ENV-046 - A mudline / water-bottom branch exists for offshore wells

- **Chapter evidence:** P2; chapter status `ABSENT`; T54; sections 4.5, 6.5 and 8.
- **Atomic obligations:** offer a mudline-referenced branch with declared mudline depth and a backend-validated enumeration; refuse unknown branch values.
- **Current source:** both temperature modules provide only surface-referenced trends; no mudline parameter/branch or backend selector refusal exists.
- **Qualifying acceptance tests:** none; T54 is missing. Test class `MISSING`.
- **Supporting tests:** surface/BHT anchor tests cannot establish a mudline geotherm.
- **Manual evidence:** formation-temperature 0/0 and not recorded; workflow 0/23.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** inclusion of offshore mudline temperature in the Windows pilot has not been decided; SB-ENV-009 and SB-ENV-043 are prerequisites.
- **Next action:** if included, add the branch without a default mudline depth, validate its selector in the runner and exercise surface, mudline, missing-depth and unknown-selector cases.

## SB-ENV-047 - A declared parameter that does not enter the answer is removed or used

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; T55; sections 4.5, 6.5 and 8.
- **Atomic obligations:** every declared ENV parameter must be consumed on a reachable branch; a branch-aware build gate prevents drift.
- **Current source:** branch-aware inspection confirmed that `ftemp_grad` consumes BHT and TD_BHT in BHT mode and that current ENV manifest parameters have reachable consumers. No executable registry-to-body T55 inventory was found.
- **Qualifying acceptance tests:** none; T55 is promised by the chapter but absent from the suite/build gate. Test class `MISSING`.
- **Supporting tests:** `formation_temperature_lands_on_both_of_its_anchors` and the nonpositive-TD guard passed and prove the formerly misreported BHT branch only.
- **Manual evidence:** formation-temperature 0/0 and not recorded; workflow 0/23.
- **Git evidence:** current parameter use is integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** a branch-aware zero-unused-declaration gate is missing; the old chapter claim of an unused BHT input is closed by source evidence and must not be revived.
- **Next action:** implement T55 by inventorying each manifest argument against all reachable option branches, with a deliberately unused test module as the failing control.

## SB-ENV-048 - The resistivity temperature constant is defined once, cited, and surfaced

- **Chapter evidence:** P0; chapter status `PRESENT-UNVERIFIED`; T56/T57; sections 4.5, 5, 6.5, 7.3 RF-1 and 8.
- **Atomic obligations:** define one named cited constant in one unit system; derive the other unit; surface it at every temperature-corrected Rw path; keep the rejected alternative unreachable.
- **Current source:** the accepted Celsius/Fahrenheit pair appears as duplicated literals in `modules.rs::resolve_rw`, `precalc` tests and `multimin2.rs`; there is no named single definition or UI/source surface. No rejected negative-offset branch was found reachable.
- **Qualifying acceptance tests:** none; existing tests repeat the literals in their expected arithmetic, so they cannot prove single ownership, unit derivation, source display or rejected-branch absence. Test class `MISSING`.
- **Supporting tests:** `precalc_degc_mode_converts_for_arps` passed and supports current unit arithmetic only; the chapter supplies the independent two-source equivalence.
- **Manual evidence:** formation-temperature 0/0 and not recorded; data-conventions 0/45.
- **Git evidence:** accepted arithmetic is integrated, while the single-source/surfacing contract diverges at the anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no scientific value is open: the chapter fixes the cited pair. The missing work is one definition, derived conversion, source surface and whole-tree guard.
- **Next action:** centralize the cited constant, derive the alternate unit and add T56/T57 across every consumer plus a whole-tree rejected-alternative absence check.

## SB-ENV-049 - A superseded module delegates to the survivor and says so

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; T58; sections 4.5, 6.5 and 8.
- **Atomic obligations:** keep the legacy ID runnable; delegate to the survivor; hide it from pickers; preserve the source rationale.
- **Current source:** `modules.rs::gr_normalize` maps legacy names/options onto `condition::normalize`, remains in the dispatcher/catalog and documents why; `ribbon.ts` and `workflowDialog.ts` hide it from new-module pickers.
- **Qualifying acceptance tests:** none; no single T58 test runs a saved legacy step, compares it with the survivor and inspects both picker inventories. Test class `MISSING`.
- **Supporting tests:** legacy normalization arithmetic passed through the delegator, but it does not prove UI hiding or equality with a direct survivor run.
- **Manual evidence:** workflow 0/23; processing-history 0/7; conditioning 0/27.
- **Git evidence:** delegation and hiding are integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** only the promised observable compatibility/picker regression proof is missing.
- **Next action:** add T58 with a serialized legacy chain, direct-survivor equality, legacy catalog reachability and absence from both picker surfaces.

## SB-ENV-050 - A depth-trend parameter is well-scoped, and a compartment parameter is not

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; T59; sections 4.5, 6.5 and 8.
- **Atomic obligations:** refuse named-zone overrides for continuous trends; accept well-wide overrides; allow boundary-stepping compartment parameters; preserve the physical justification per parameter.
- **Current source:** `ArgSpec::well_scope`, `param_well` and `resolve_param_arrays` implement the distinction. Temperature trend parameters are well-scoped; pressure trend parameters remain zone-capable; the source comments record why.
- **Qualifying acceptance tests:** `workflow::tests::a_geothermal_gradient_is_refused_per_zone_and_accepted_per_well` and `a_per_zone_pressure_gradient_reaches_exactly_its_own_samples` each passed exactly once. Together they pin refusal, well-wide control and compartment acceptance from T59; test class `CORRECTNESS`.
- **Supporting tests:** none needed beyond the two-sided physical control.
- **Manual evidence:** formation-temperature 0/0 and not recorded; workflow 0/23.
- **Git evidence:** behavior and tests are integrated at the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; automated proof passes, but the workflow/temperature capability has no field exercise.
- **Next action:** preserve T59 and field-exercise one temperature-trend refusal and one compartment override before pilot acceptance.

## SB-ENV-051 - Percentiles are exact order statistics, never histogram bin means

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; T60; sections 4.6, 6.6 and 8.
- **Atomic obligations:** sort finite values and compute exact order statistics at every normalization call site; never use histogram-bin means.
- **Current source:** `condition::normalize` sorts before `distribution::percentile`; `gr_normalize` delegates to it rather than retaining a second implementation.
- **Qualifying acceptance tests:** `condition::tests::a_two_point_map_lands_the_wells_own_percentiles_on_the_reference_pair` passed exactly once with deliberately permuted depth order and both endpoint assertions. The expected map is independently derived from supplied fixture endpoints and T60; test class `CORRECTNESS`.
- **Supporting tests:** the multi-entity workflow normalization test also passed but reads legacy defaults and is supporting-only for per-entity isolation.
- **Manual evidence:** conditioning 0/27; workflow 0/23.
- **Git evidence:** behavior and the anti-depth-order regression are integrated at the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; automated proof passes, but normalization has no field evidence and adjacent reference/provenance contracts remain open.
- **Next action:** preserve T60 and field-exercise normalization on two representative curves while keeping SB-ENV-052 through 055 separately open.

## SB-ENV-052 - The normalisation reference pair ships ABSENT

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; T07/T61; sections 4.6, 5, 6.6 and 8.
- **Atomic obligations:** every normalization entry ships without a reference pair and refuses until the user supplies one.
- **Current source:** the survivor `condition::normalize` uses open references and refuses without them, but runnable legacy `gr_normalize` still ships numeric reference defaults; saved legacy chains can therefore normalize without an explicit pair.
- **Qualifying acceptance tests:** `condition::tests::normalize_refuses_a_reference_pair_it_was_not_given` passed for the survivor. Legacy default tests also passed and derive their expected values from the current manifest. Because the full inventory violates T61, row test class is `CHARACTERIZATION`.
- **Supporting tests:** survivor refusal is correct on one side; the legacy default tests pin the opposite side and are not scientific authority.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** mixed open/default behavior is integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** section 5 deliberately specifies the pair ABSENT; no default may be retained or replaced.
- **Next action:** remove legacy numeric defaults while preserving saved explicit values, make missing legacy values refuse visibly and turn T61 into an all-entry-path correctness test.

## SB-ENV-053 - Normalisation is recorded, reviewable and overridable per well

- **Chapter evidence:** P1; chapter status `ABSENT`; T62; sections 4.6, 6.6 and 8.
- **Atomic obligations:** persist per-entity reference pair, computed percentiles, linear map, interval and manual override; expose review before acceptance.
- **Current source:** the runner writes the resulting curve and supplied numeric parameters. It does not persist computed percentiles/map/interval, provide a per-entity review-and-accept surface or record an override identity.
- **Qualifying acceptance tests:** none; T62 is missing. Test class `MISSING`.
- **Supporting tests:** per-entity normalization arithmetic proves isolated computation, not reviewable custody.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the review/provenance boundary.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** requires a persisted normalization record and acceptance/override workflow; no reference value may be inferred.
- **Next action:** persist the complete per-entity map first, then add preview/accept/override and a reload test with two distinct distributions.

## SB-ENV-054 - Normalisation percentiles are computed over a declared common interval

- **Chapter evidence:** P1; chapter status `PARTIAL`; T62/T63; sections 4.6, 6.6 and 8.
- **Atomic obligations:** record each percentile interval and warn when intervals across a set are not comparable.
- **Current source:** the universal mask can restrict samples before percentile computation, and a focused test proves masked samples do not anchor the map. No declared interval is persisted, compared or warned across entities.
- **Qualifying acceptance tests:** none; T62/T63 are missing. Test class `MISSING`.
- **Supporting tests:** `mask_excludes_flagged_samples_from_gr_normalize_percentiles` and the per-entity normalization test support the computation seam only.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** masked computation is integrated; interval custody/comparison is absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-ENV-053's record is prerequisite; comparability semantics need an explicit declared interval, not an inferred depth overlap.
- **Next action:** persist each declared interval and implement T62/T63 with matching and deliberately mismatched intervals plus a visible warning.

## SB-ENV-055 - A normalisation reference pair is named and sourced separately from a `Vsh` endpoint pair

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; T64; sections 4.6, 5, 6.6 and 8.
- **Atomic obligations:** keep normalization references and Vsh endpoints distinct, separately named and separately sourced even when chosen values coincide.
- **Current source:** normalization uses distinct runtime names from Vsh endpoints, so changing a Vsh parameter does not mechanically change normalization. However the legacy manifest ships uncited reference defaults and explicitly describes them as matching Vsh endpoints; neither pair has the required per-parameter source custody.
- **Qualifying acceptance tests:** none; T64's endpoint-change independence plus separate-source assertion is missing. Test class `MISSING`.
- **Supporting tests:** the legacy default characterization proves numeric equality only; equality is permitted by the requirement and does not prove or disprove semantic independence.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** separate names are integrated; source separation/default discipline is incomplete at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** machine-readable sources are absent and SB-ENV-052 forbids the legacy default; the old chapter status overstates runtime coupling and is preserved as historical evidence only.
- **Next action:** remove the legacy defaults, attach independent source identities when user values are supplied and add T64 proving a Vsh endpoint change cannot move normalization.

## SB-ENV-056 - Log-QC limits ship ABSENT, and band precedence is specified once

- **Chapter evidence:** P1; chapter status `ABSENT`; T65/T66; sections 4.6, 5, 6.6, 7.1 OI-6, 7.2 ESC-3 and 8.
- **Atomic obligations:** ship user/extreme bands with no defaults; define precedence once; require the extreme band to bracket the user band; refuse inversion at entry.
- **Current source:** no log-QC limit registry, editor, precedence validator or refusal surface exists.
- **Qualifying acceptance tests:** none; T65 is missing and T66 remains non-adoptable characterization only. Test class `MISSING`.
- **Supporting tests:** no vendor band values were adopted or opened; generic clip bounds are not a QC-band precedence facility.
- **Manual evidence:** data-conventions 0/45; conditioning 0/27.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** OI-6 leaves the between-band state open and ESC-3 leaves conflicting vendor semantics unresolved; all numeric limits remain ABSENT.
- **Next action:** decide OI-6 without adopting vendor numbers, implement an empty band registry and inversion refusal, then characterize the non-adoptable conflict separately.

## SB-ENV-057 - One token for "a length in the project's depth unit", validated once

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T43/T67; sections 4.6, 6.6 and 8.
- **Atomic obligations:** define one unit token for project-depth lengths; forbid false fixed-unit labels; validate through one conversion path.
- **Current source:** live manifests use at least `depth`, `m|ft` and `m` for native-depth arithmetic. `depth_shift` and splice declare metres while applying values in the project depth unit; generic `DepthUnit` conversion does not validate these ArgSpec strings.
- **Qualifying acceptance tests:** none; T67's complete declaration inventory is missing. Test class `MISSING`.
- **Supporting tests:** generic project/file conversion passed, and the physical-window test supports behavior, but neither prevents false manifest labels.
- **Manual evidence:** data-conventions 0/45; conditioning 0/27; workflow 0/23.
- **Git evidence:** the divergent tokens are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one canonical token/validator is missing; no conversion factor is open.
- **Next action:** define the single project-depth-length token, migrate every declaration/doc string and add a whole-registry T67 plus metric/foot UI controls.

## SB-ENV-058 - Borehole-image speed correction, derived independently

- **Chapter evidence:** P3; chapter status `ABSENT`; T68; sections 4.7, 6.6, 7.2 ESC-6/ESC-7, 7.3 TR-4 and 8.
- **Atomic obligations:** independently derive speed correction from lawful primary sources, emit displacement and a reversible record, and prove the contract without proprietary assets.
- **Current source:** no borehole-image speed-correction implementation, displacement output or recovery record exists.
- **Qualifying acceptance tests:** none; T68 is contract-only and has no numeric oracle until the named primary sources are held. Test class `MISSING`.
- **Supporting tests:** image ingest/display paths and unrelated generated overlays do not implement speed correction; protected descriptors/resources were not inspected.
- **Manual evidence:** image-data 0/30; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** ESC-6 classification, ESC-7 primary-source acquisition and TR-4 legal/non-reproduction boundaries remain open. The petrophysics-first pilot explicitly leaves this P3 capability for later.
- **Next action:** do not implement now; acquire/adjudicate lawful primary sources first, then write a separate plan with a reversible synthetic-oracle test.

## Receipt totals

- As-built: 19 `ABSENT`, 15 `PARTIAL`, 15 `PRESENT-DIVERGENT`, 4 `PRESENT-UNVERIFIED`, 5 `PRESENT-OK`.
- Release disposition: 50 `PILOT-BLOCKER`, 7 `UNDECIDED`, 1 `DEFERRED`, 0 `OUT`.
- Risk class: 32 `SILENT-WRONGNESS`, 16 `DATA-INTEGRITY`, 5 `DEGRADED-RESULT`, 1 `RECOVERY`, 3 `REQUESTED-CAPABILITY`, 1 `LATER`.
- Test class: 5 `CORRECTNESS`, 4 `CHARACTERIZATION`, 49 `MISSING`.
- Commit state: 39 `INTEGRATED`, 19 `UNIMPLEMENTED`.
- Chapter test routing: T01-T70 each remains routed exactly once by the approved plan; no missing executable intention was treated as an implemented test.
- Open decisions preserved: OI-1 through OI-8, ESC-1 through ESC-16, TR-1 through TR-4 and RF-1 through RF-10 remain unresolved except where a row merely names the dependency. No row settles one by implication.
