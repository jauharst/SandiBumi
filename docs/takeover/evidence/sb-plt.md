# Gate 1 SB-PLT live adjudication

- Branch: `codex/g1-sb-plt-adjudication`
- Adjudication start HEAD: `6fe716aaf5e4b610a4c3703ad84b2a75800aff07`
- Accepted implementation evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`
- Reviewed stack base: `0cf9480b2b0d8c074f42245edde85c5aace29d48`
- `origin/master` at evidence freeze: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Adjudication date: `2026-08-11`
- Worktree at evidence freeze: clean; `D:\XX. SandiBumi` was the only registered Git worktree.
- Row guard: passed - exactly 35 planned `SB-PLT` rows, all initially `UNADJUDICATED`, in numeric order, with all source-owned `owned_tests` fields blank.
- Evidence boundary: this receipt classifies the accepted implementation tree. It does not amend PRD v2, change plotting behavior, choose a scientific parameter, edit generated chart payloads, decide content rights, or turn an unchecked manual scenario into evidence.
- Source-navigation boundary: the codebase index was not callable in this task, so targeted `rg`, direct source reads, executable tests and reachable Git history were used as the declared fallback. Consequential negative findings were checked in the expected Rust, TypeScript, test and history paths.
- Verification boundary: the 15 focused `plotting::tests` and the one clay-overlay characterization test passed; all 13 frontend acceptance tests passed. Rust dead-code diagnostics independently confirmed that most plotting policy helpers are not product callers. A helper, source-text inventory or internal `Result` is supporting evidence unless it exercises the complete observable contract.
- Manual-evidence boundary: capability counts below are copied from the generated verification matrix. Partial or unexercised capability rows remain field-evidence gaps.

## SB-PLT-001 - Persist semantic intent and concrete resolution separately

- **Chapter evidence:** P0; chapter status `PARTIAL`; no direct whole-contract chapter test is assigned; sections 4.1, 6 and 8.1.
- **Atomic obligations:** persist the semantic request separately from each well's concrete curve ID, mnemonic, quantity, source/display units, conversion, sample count, resolution reason and revision; refuse every unresolved required channel; retain that record in project/template/export state.
- **Current source:** `src-tauri/src/plotting.rs` validates and durably saves typed `PersistedPlotState`; generic document writes cannot bypass the typed plot/session commands. `src/ipc.ts` retains every per-well resolution returned by the resolver. `plotCommon.ts`, `workspace.ts`, the crossplot, histogram, Pickett, correlation and Vega panels capture the represented wells plus exact bindings in project properties, named templates and sessions. Session restore waits for background binding resolution and refuses a changed curve or revision. `plotExport.ts`, `composite.rs` and the PNG/PDF commands retain a separate canonical binding record in SVG, PNG, PDF, clipboard and print artifacts.
- **Qualifying acceptance tests:** `plotting.rs::a_saved_plot_template_and_export_keep_one_request_and_each_wells_distinct_concrete_resolution` is one `CORRECTNESS` proof sourced to chapter section 4.1. It saves and reloads project and template state for two wells with distinct curve IDs, mnemonics, quantities, units, conversions, counts, reasons and revisions; embeds the export record; and proves from both sides that an unresolved required channel neither saves nor exports.
- **Supporting tests:** `plotting.rs::a_plot_binding_keeps_the_request_and_each_wells_concrete_resolution` still proves the resolver value shape and immediate required-channel refusal; the qualifying test does not substitute that internal result for the durable reporting surfaces.
- **Manual evidence:** `crossplot` 6/17, `histogram` 5/26, `pickett` 0/12, `correlation-tops` 0/46, `vega` 0/6 and `project-lifecycle` 3/28 - all partial or unexercised; the new four-scenario section is unchecked.
- **Git evidence:** reachable commit `35e83df` introduced the resolver and in-memory adapter; the current topic-branch increment closes the durable project/template/session/export gap without editing `db.rs` or treating manual review as complete.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2. Visual, manual and representative-field confirmation remain open and are not inferred from the green serialization proof.
- **Next action:** retain the typed save/export boundary, execute the `REVIEW.md` click-through and Gate 4 corpus case separately, and proceed to SB-PLT-002 without weakening this refusal.

## SB-PLT-002 - Resolve axes through one explicit precedence chain

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intentions T01-T02; sections 4.1, 6 and 8.1.
- **Atomic obligations:** apply user, header display, audited family display and finite-data tiers in that order for every axis; exclude validity ranges from display precedence; show the winning tier in UI and export.
- **Current source:** `src/ui/axisRange.ts` is the one live frontend resolver. Crossplot, histogram, Pickett, correlation and generated Vega plots all supply user, typed curve-header, screened unit-family and finite-data candidates through it; log axes refuse non-positive candidates. Each surface renders the winning tier and passes the same axis records into typed plot state and SVG/PNG/PDF export. `plotting.rs` persists the concrete header-display sidecar and validates that every new saved/exported plot carries distinct finite axes and custody tiers; `curveMetaDialog.ts` exposes the header range separately from validity and records exact undo/redo. Custom Vega specifications explicitly refuse persistence/export because their arbitrary scale grammar cannot yet prove custody.
- **Qualifying acceptance test:** `frontend-acceptance.test.mjs::a_user_axis_range_wins_and_without_it_the_header_range_wins_in_the_rendered_label_and_export_while_validity_never_becomes_display` is one `CORRECTNESS` proof sourced to chapter section 4.1 and T01/T02. Unequal discriminator fixtures prove user over header over family/data, header after user removal, rendered and exported tier identity, and validity exclusion. The same test inventories all five live quantitative panels so an unused helper cannot pass while a panel retains private defaults.
- **Supporting tests:** `plotting.rs::a_user_axis_range_wins_and_without_it_the_header_display_range_wins` preserves the backend-side precedence/validity guard, while the durable SB-PLT-001 round-trip proves that the concrete axis range and tier survive save and export.
- **Manual evidence:** `crossplot` 6/21, `histogram` 5/30, `pickett` 0/16, `correlation-tops` 0/50 and `vega` 0/10 - partial or unexercised; the new four-scenario section is unchecked.
- **Git evidence:** reachable commit `3844ae5` added the partial crossplot chain; the current topic-branch increment replaces the five independent live policies with one governed resolver without expanding the screened dossier seed set.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2. Visual readability, runtime Vega scale-name behavior and representative-file interaction remain manual/Gate 4 evidence, not automated claims.
- **Next action:** retain the shared resolver and explicit custom-spec refusal, execute the `REVIEW.md` click-through separately, skip deferred SB-PLT-003, and proceed to pilot blocker SB-PLT-004.

## SB-PLT-003 - Overlay compatibility is quantity-and-unit typed

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intentions T03-T04; sections 4.1, 6 and 8.1.
- **Atomic obligations:** declare X/Y quantity, canonical unit, orientation and admissible transform; convert through a registered rule; reject incompatible axes; never let a mnemonic match alone authorize rendering.
- **Current source:** chartbook overlays in `crossplotPanel.ts` first match mnemonic aliases and then require resolved quantity/unit bindings and a local conversion table. Thomas-Stieber, matrix, rock and core overlays bypass that typed authorization. The separate Rust registry-backed binder is unused by product code, while the active TypeScript registry and transforms can drift from it.
- **Qualifying acceptance tests:** none; T03/T04 do not execute the live renderer, conversion persistence and incompatible refusal across the overlay inventory. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::an_overlay_requires_quantity_compatible_units_and_records_any_registered_conversion` proves only the unused Rust binder.
- **Manual evidence:** `chart-overlays` 16/53 and `crossplot` 6/13 - partial; `verification-stewardship` 0/24.
- **Git evidence:** reachable commit `32b834b` added the chartbook typed gate; accepted source retains untyped overlay families and duplicate registries.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one authoritative registry and an exhaustive active-overlay gate are missing.
- **Next action:** derive every overlay authorization from one quantity/unit registry and implement both incompatible-mnemonic and compatible-conversion controls through render, template and export.

## SB-PLT-004 - Valid and display ranges remain distinct

- **Chapter evidence:** P0; chapter status `ABSENT`; no direct whole-contract test; T07/T12 provide only cross-cutting support; sections 4.1, 6 and 8.1.
- **Atomic obligations:** keep validity and display ranges distinct; annotate display-hidden counts without changing populations; make validity exclusion opt-in and report its effect on n, statistics and fits across all channels.
- **Current source:** `src/ui/plotRangePolicy.ts` is the shared aligned-channel population policy. Crossplot, Histogram, Pickett, Correlation and generated Vega plots all count non-finite, log-domain, display-hidden and validity-excluded samples through it. Display bounds never remove an analysis index; validity is absent and disabled by default, requires complete finite analyst limits, and changes the statistics/fit population only after explicit opt-in. Each live panel renders the shared summary; Histogram statistics/percentiles/box plots and brush overlays, Pickett fit anchors and hover/brush interaction, Correlation strip scales and traces, and Vega distribution/regression input rows use the screened population. Custom Vega specifications retain the existing refusal because arbitrary grammar cannot prove governed clipping. The replaced Rust-only helper is test-gated rather than left as disconnected production evidence.
- **Qualifying acceptance test:** `frontend-acceptance.test.mjs::display_clipping_counts_hidden_points_without_changing_analysis_while_explicit_validity_changes_and_discloses_n_statistics_and_fit_inputs_on_every_pilot_plot` is one `CORRECTNESS` proof sourced to chapter sections 2.2 and 4.1. Unequal discriminator fixtures prove display `1..3` preserves all five analytical samples and counts two hidden, while explicit validity `2..4` retains three, excludes two, changes the independently calculated mean from two to three and reports statistics/fit inputs. It executes every panel adapter from both sides and inventories every live disclosure call; a temporary Histogram mutation that ignored validity returned the expected `5 !== 3` RED before restoration. TypeScript and cargo check are green; the exact full gate is `1025 passed / 0 failed / 37 ignored` with `47` owned Rust warnings.
- **Supporting tests:** `plotting.rs::display_clipping_counts_hidden_points_while_validity_filtering_changes_the_population_explicitly` preserves the backend arithmetic characterization behind `cfg(test)`; it is not claimed as a live integration proof.
- **Manual evidence:** `crossplot` 6/25, `histogram` 5/34, `pickett` 0/20, `correlation-tops` 0/54 and `vega` 0/14 - partial or unexercised; the new four-scenario section is unchecked.
- **Visual evidence:** pending. A fresh Tauri release build succeeded, but the sandbox capture path failed before producing a frame because the existing E2E pipeline omits the now-required `list_wells` scope, omits workflow custody, and a capture-only session lost its WebDriver session during teardown. No mock/browser-only screenshot is substituted and no PNG is claimed.
- **Git evidence:** reachable commit `e9cc980` integrated the crossplot subset; the current topic-branch increment replaces the private pilot-panel policies with one shared frontend contract without inventing a validity range or default.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual readability, manual interaction and representative-field confirmation remain open and are not inferred from the adapter proof or release build.
- **Next action:** retain the shared policy and custom-spec refusal, execute the `REVIEW.md` click-through separately, and proceed to pilot blocker SB-PLT-005.

## SB-PLT-005 - Unit-limit content is audited before activation

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intention T05; sections 4.1, 6, 7.1 O-2 and 8.1.
- **Atomic obligations:** reject schema-valid content as authority; dimensionally re-derive every converted pair before activation; leave every suspect row disabled with its reason.
- **Current source:** `axisRange.ts::UNIT_LIMIT_ROWS` is the complete source-owned activation registry for the nine shipped family rows plus the chapter's audit-only attenuation refusal. `auditUnitLimitRow` uses generated `UNIT_REGISTRY_RULES` to prove registered quantity kinds, conversion direction and the cited 15% screen before activation. Unknown units and suspect rows preserve a disabled reason, fall back to finite data and carry the audit through labels and exports. Crossplot, Histogram, Pickett, Correlation and governed Vega all execute `resolveBoundAxisRange`; the disconnected Rust-only audit helper was removed rather than presented as a product path.
- **Qualifying acceptance test:** `frontend-acceptance.test.mjs::every_shipped_unit_limit_row_is_source_owned_and_dimensionally_screened_while_the_documented_6_56x_pair_and_unknown_units_stay_disabled_with_reasons` is one `CORRECTNESS` proof sourced to chapter sections 2.2, 4.1, 6 and 7.1 O-2 plus dossier §3.3a. It inventories every active row, proves exact RHOB and screened rounded DT conversions, proves the 6.56× attenuation refusal, proves an unknown unit cannot inherit a family range, and inventories all five live consumers. A deliberate RHOB `2950 -> 3000` mutation returned the expected RED before restoration.
- **Supporting test:** `plotting.rs::the_documented_attenuation_pair_is_6_56x_divergent_and_exceeds_the_cited_screen_in_the_wrong_direction` independently derives T05 from the exact international-foot definition.
- **Manual evidence:** `crossplot` 6/13, `pickett` 0/8 and `verification-stewardship` 0/24.
- **Git evidence:** reachable commit `aefeb6b` added the unused helper; the current topic-branch increment replaces that disconnected claim with the shared frontend activation gate and generated conversion custody.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual readability, manual interaction and representative-field confirmation remain open; no bulk incumbent table is adopted.
- **Next action:** retain the small source-owned registry and refusal boundary, execute the `REVIEW.md` click-through separately, and proceed to pilot blocker SB-PLT-006.

## SB-PLT-006 - One canonical histogram-bin contract

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intentions T06-T07; sections 4.2, 5, 6 and 8.1.
- **Atomic obligations:** use half-open bins with only the final upper endpoint included; count NaN/infinity exclusions separately; make displayed total equal the bin-count sum through every histogram surface.
- **Current source:** `src/distribution.ts::canonicalHistogram` owns the cited 50-bin default, 1-200 normalization, half-open/final-closed arithmetic, exact edges, displayed-total sum and non-finite count. The primary panel, transformed crossplot marginals, pre-binned Vega grammar and log-view micro-glyphs call it; Vega retains raw non-finite X samples until the shared population screen counts them. Canvas SVG/PDF rerun the same static draws and Vega SVG/PNG export the rendered governed view. Rust's matching contract lives in `distribution.rs`; the disconnected plotting-local wrapper is removed and the native-install smoke test points to the real path.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::every_pilot_histogram_uses_half_open_bins_with_a_closed_final_endpoint_counts_non_finite_samples_separately_and_displays_the_sum_of_bin_counts` executes T06/T07 through canonical, primary, marginal and Vega adapters, verifies pre-binned Vega rows and inventories the live screen/vector-export routes. Test class is `CORRECTNESS`.
- **Supporting tests:** `plotting.rs::histogram_bins_are_half_open_except_for_the_final_upper_endpoint_and_non_finite_values_are_counted` executes the real Rust distribution contract. A deliberate frontend mutation changed the final endpoint guard from `> hi` to `>= hi`; the owned test returned `[1,1,1]` versus `[1,1,2]` before restoration.
- **Manual evidence:** `histogram` 5/22, `crossplot` 6/13, `vega` 0/2 and `report` 6/53.
- **Git evidence:** reachable commit `b45427a` migrated only the primary histogram path; the current Gate 2 increment closes the retained marginal, Vega, log-view, Rust and property-affordance divergence.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual readability, manual interaction and representative-field confirmation remain open rather than inferred.
- **Next action:** retain the shared contract and explicit population labels, execute the `REVIEW.md` click-through separately, skip deferred SB-PLT-007/008 and proceed to pilot blocker SB-PLT-009.

## SB-PLT-007 - Overplot thresholds expose the comparator

- **Chapter evidence:** P1; chapter status `ABSENT`; chapter intentions T08-T09; sections 4.2, 5, 6 and 8.1.
- **Atomic obligations:** persist threshold value and comparator; translate only by the exact integer relation between `>= D` and `> T`; never call raw threshold numbers equivalent.
- **Current source:** Vega offers a density heatmap, but no plotting source, template, legend or export persists a density threshold/comparator or the exact translation. Targeted Rust/TypeScript/test/history searches recovered no implementation.
- **Qualifying acceptance tests:** none; T08/T09 are absent. Test class is `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** `vega` 0/2 and `crossplot` 6/13.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must decide whether governed density-threshold interchange belongs in the pilot; no comparator or threshold is inferred.
- **Next action:** if selected, add an explicitly typed comparator/value record and implement both T08 and T09 from the chapter's shown integer arithmetic.

## SB-PLT-008 - Percentile probability and range position are different types

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intentions T10-T11; sections 4.2, 5, 6 and 8.1.
- **Atomic obligations:** bound `PercentileP` to 0-100; keep `RangePositionPct` finite but unbounded; name the type in APIs, templates and exports without silently clamping an extrapolated position.
- **Current source:** Rust and TypeScript type helpers enforce the distinct numeric domains. Histogram percentile input uses `PercentileP`; `parseRangePositionPct` has no product caller. Saved options and exports still serialize plain numbers/arrays without the semantic type name.
- **Qualifying acceptance tests:** none; T10/T11 exercise helpers but not API/template/export type persistence. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::percentile_probability_rejects_130_while_range_position_preserves_130_and_minus_5` pins both chapter examples in the unused Rust types.
- **Manual evidence:** `histogram` 5/22, `crossplot` 6/13 and `project-lifecycle` 3/24.
- **Git evidence:** reachable commit `7bac6e6` added the types and helper test; serialization remains untyped at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the range-position path is unwired and serialized type identity is absent.
- **Next action:** carry a tagged percentage kind through API/template/export records and implement T10/T11 across parse, round trip and rendered use.

## SB-PLT-009 - Statistics disclose population, estimator and exclusions

- **Chapter evidence:** P1; chapter status `PARTIAL`; chapter intentions T12-T13; sections 4.2, 6 and 8.1.
- **Atomic obligations:** record active versus pooled population, interval, selection, finite-pair count, all exclusions, percentile interpolation and sample/population standard-deviation choice for every statistic.
- **Current source:** `plotCanvas.ts::buildPlotStatisticsRecord` owns one typed, reconciled statistics record. Histogram, Crossplot, Pickett, Correlation and generated Vega views build the record from their real screened populations, render its disclosure and carry it through SVG/PDF/PNG, clipboard and print exports. Raincloud plots produce one record per displayed group. `plotExport.ts`, `ipc.ts` and `plotting.rs` preserve the record and refuse unbound channels, foreign wells, invalid intervals, unknown estimators and unreconciled counts at the export boundary.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::every_plot_statistic_records_its_population_interval_selection_finite_pairs_exclusions_percentile_interpolation_and_standard_deviation_choice`; expected arithmetic and box semantics are independently sourced from T12/T13. The test executes all five live adapters, active and pooled populations, two-sided and one-sided intervals, sample and population standard deviation, display-only clipping, per-group raincloud records and export custody. Test class is `CORRECTNESS`.
- **Supporting tests:** `plotting.rs::a_plot_statistics_export_preserves_a_reconciled_record_and_refuses_unreconciled_exclusions` round-trips the cited T12 record and refuses unreconciled counts, an unbound channel and a foreign well. A deliberate P5-to-minimum whisker mutation made the T13 acceptance test RED before restoration.
- **Manual evidence:** `histogram` 5/22, `crossplot` 6/13, `vega` 0/2 and `report` 6/53.
- **Git evidence:** current topic-branch worktree; TypeScript and cargo check are green and the exact full gate is 1028 passed / 0 failed / 37 ignored with 42 owned Rust warnings.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** no Gate 2 automation blocker. Visual, Manual and Field evidence remain open rather than inferred from the adapter and export proofs.
- **Next action:** retain the governed record, execute the separate review and Gate 4 scenarios, and proceed serially to SB-PLT-013 without pulling deferred regression or Pickett-fit contracts into this increment.

## SB-PLT-010 - Regression is a versioned scientific result

- **Chapter evidence:** P1; chapter status `PARTIAL`; chapter intentions T14-T16; sections 4.2, 6, 7.1 O-3 and 8.1.
- **Atomic obligations:** persist model, method, transformed space, coefficients, R-squared, pair/exclusion counts, interval, wells and source revisions; support the v1 model/method matrix; display valid goodness metrics without forced clamping; keep unsourced robust/polynomial policy absent.
- **Current source:** crossplot fits linear/power/log-X/exponential with Y-on-X, X-on-Y and RMA and displays equation, R-squared and n. The returned value contains only `a`, `b`, `r2`, `n`; it is neither versioned nor persisted and omits transformed-space, exclusions, interval, wells and revisions. Vega has a separate regression transform and offers `quad`, increasing the governance split.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::characterizes_regression_as_coefficients_without_a_versioned_scientific_record`; T14 supplies the independent arithmetic while the four-field payload is explicitly characterization. Test class is `CHARACTERIZATION`.
- **Supporting tests:** no test covers T15 method/source retention or T16 transformed-domain exclusion disclosure at the product surface.
- **Manual evidence:** `crossplot` 6/13, `vega` 0/2, `processing-history` 0/7 and `verification-stewardship` 0/24.
- **Git evidence:** accepted anchor contains the partial crossplot and Vega fit paths; no versioned fit-record integration exists.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the scientific record and complete T14-T16 proof are missing; O-3 deliberately keeps robust method/tuning absent.
- **Next action:** introduce one versioned fit result used by both renderers, persist its full inputs/exclusions/revisions and implement T14-T16; do not choose a robust default without a source.

## SB-PLT-011 - Pickett states what is and is not identifiable

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intention T17; sections 4.2, 5, 6 and 8.1.
- **Atomic obligations:** identify only the product `a*Rw`; refuse a separate `a` or `Rw` unless its counterpart is independently sourced; require `a`, `m`, `n`, `Rw` and sources before any saturation guides.
- **Current source:** `pickettPanel.ts::fitWaterLine` derives and labels `m` plus `a*Rw`, exposes only M and A_RW write rows, and explicitly says `a` and `Rw` are not separately identified. Saturation guides remain absent, so no unsourced guide is rendered. The stricter live UI does not call `plotting.rs::disclose_pickett_fit`, and there is no complete rendered/save/restore refusal test.
- **Qualifying acceptance tests:** none; T17's observable UI refusal and persistence surface are not exercised. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::a_pickett_fit_without_sourced_a_or_rw_exposes_only_their_product` proves the arithmetic/refusal helper with sourced fixture metadata.
- **Manual evidence:** `pickett` 0/8, `crossplot` 6/13 and `processing-history` 0/7.
- **Git evidence:** reachable commit `f1d6e57` added the disclosure helper and safe UI behavior; no separate parameter is emitted at the accepted anchor.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** integrated T17 and field evidence are missing; the four Pickett scientific inputs remain intentionally absent and are not supplied here.
- **Next action:** add an observable T17 covering fit display, blocked separate writes, save/restore and absent saturation guides, then field-exercise it without selecting defaults.

## SB-PLT-012 - Hingle uses the negative reciprocal exponent

- **Chapter evidence:** P1; chapter status `ABSENT`; chapter intentions T18-T19; sections 4.2, 6 and 8.1.
- **Atomic obligations:** implement only `Rt^(-1/m)` and reject the reciprocal-sign alternative; offer no compatibility mode that can invert the axis.
- **Current source:** targeted source, tests and reachable-history searches found no Hingle plot, transform, route or compatibility mode.
- **Qualifying acceptance tests:** none; T18/T19 are absent. Test class is `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** `pickett` 0/8, `equation-engine` 0/11 and `verification-stewardship` 0/24.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must decide whether Hingle belongs in the pilot; the equation is specified, but no product surface exists.
- **Next action:** if selected, implement the governed negative-reciprocal transform and both T18/T19 exactly; otherwise keep the capability absent.

## SB-PLT-013 - Missing and out-of-range policy is channel-specific

- **Chapter evidence:** P0; chapter status `PARTIAL`; no direct whole-contract test; T07, T12, T16 and T20 provide cross-cutting support; sections 4.3, 6 and 8.1.
- **Atomic obligations:** exclude and count non-finite values; exclude non-positive log values from plot and statistics; clip/count X/Y overflow; clamp and edge-mark Z overflow; clamp/count waveform overflow; never mutate source samples; disclose every policy.
- **Current source:** `plotTypes.ts::applyPlotChannelPolicy` is the shared TypeScript channel policy. `plotRangePolicy.ts` delegates X/Y finite, log-domain and display-overflow decisions to it; Crossplot, Pickett and generated Vega apply its derived Z values plus low/high edge marks; log-view spaghetti waveforms apply its derived clamp/exclusion record. The composite SVG/PDF spaghetti renderer consumes the parallel Rust `plotting.rs::apply_plot_channel_policy` contract. Histogram's existing finite-only bin contract remains the correct channel-specific path. None of these paths writes the source curve.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::non_finite_log_xy_z_and_waveform_values_follow_one_non_mutating_reported_channel_policy` is CORRECTNESS. The arithmetic fixture proves non-finite and log-domain exclusions, X/Y clip count without replacement, Z and waveform endpoint clamps, low/high edge identity, live Pickett/Vega/log-view/composite adapters and bit-exact source preservation. A deliberate high-to-low edge mutation made the test RED before restoration.
- **Supporting tests:** `plotting.rs::missing_log_xy_z_and_waveform_values_follow_their_own_reported_policies` exercises the Rust policy from both exclusion and clamp sides and checks source bits; `composite.rs::spaghetti_draws_the_asked_for_number_of_traces_and_breaks_them_at_failures` exercises the production composite consumer.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `log-view` 5/37 and `chart-overlays` 16/53.
- **Git evidence:** reachable commit `cfc468b` added the original policy helper and crossplot subset; the Gate 2 completion commit is pending at this evidence write. Four previously owned dead-code warnings disappear because the composite renderer is now a production consumer.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual placement, manual interaction and representative-field evidence remain deliberately open.
- **Next action:** retain the shared non-mutating policy and disclosures, execute the separate review/Gate 4 scenarios, and proceed serially to SB-PLT-015 without importing deferred multi-well allocation scope.

## SB-PLT-014 - Multi-well allocation follows finite-pair screening

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intention T20; sections 4.3, 6 and 8.1.
- **Atomic obligations:** align required channels and screen finite pairs before budget allocation; give zero-pair wells no quota and an explicit reason; retain first and last eligible sample for every represented well.
- **Current source:** `plotCommon.ts::fetchContextLayers` is the active crossplot, histogram and Pickett multi-well path. It reconciles depths, applies the half-open interval, screens finite aligned rows through `allocateFinitePairBudget`, records zero-quota absence, and applies the returned first/final-preserving indices to every channel. The equivalent Rust allocator is unused, and no rendered/exported multi-well fixture exercises the complete TypeScript route.
- **Qualifying acceptance tests:** none; T20 currently exercises only the Rust helper, not the live context loader and visible/exported absence reason. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::an_all_nan_required_channel_consumes_no_quota_while_represented_wells_keep_both_endpoints` pins the chapter fixture and endpoint allocation.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `portfolio-performance` 0/50.
- **Git evidence:** reachable commit `6a5ad1e` integrated the TypeScript context path and Rust support; accepted source contains the complete mechanism.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** whole-route T20 and field evidence are missing.
- **Next action:** implement T20 through one live multi-well panel and its reduction manifest, then exercise a representative all-NaN well without changing the budget.

## SB-PLT-015 - Decimation preserves pairing, endpoints and provenance

- **Chapter evidence:** P0; chapter status `PARTIAL`; chapter intentions T21-T22; sections 4.3, 6 and 8.1.
- **Atomic obligations:** use one shared source-index vector for all mark channels; retain eligible endpoints; record original/displayed counts, algorithm, stride and forced-endpoint state; never call a reduced view complete.
- **Current source:** `plotTypes.ts::allocateFinitePairBudget` returns one source-index vector and a complete in-memory `ReductionManifest`; `fetchContextLayers` applies it to depth and every series. `describeContextOutcome` visibly says reduced and reports original/displayed counts, stride and forced endpoint state. `contextReductionExport` carries those same fields through Crossplot, Histogram and Pickett to `plotExport.ts`, and the whitelisted Rust command validates and canonicalizes the JSON. Non-stride well/legend reductions carry explicit null stride/endpoint fields rather than invented values.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::context_plot_decimation_uses_one_shared_index_retains_both_endpoints_and_exports_counts_algorithm_stride_and_forced_endpoint_without_calling_the_view_complete` is CORRECTNESS. It independently derives indices 0/4/8/10, proves depth/X/Y/Z pairing, checks first/final retention, requires the reduced-not-complete disclosure, verifies every portable field and inventories the three live panel consumers plus the whitelisted exporter. A deliberate forced-endpoint true-to-false mutation made the test RED before restoration.
- **Supporting tests:** `plotting.rs::decimation_uses_one_shared_index_vector_and_reports_the_forced_final_endpoint` independently pins the cited indices, all channels and complete helper manifest; `an_export_after_budget_reduction_includes_original_and_displayed_counts_and_the_algorithm_while_a_hard_maximum_refuses` verifies Rust canonicalization and the hard-limit refusal. The duplicate Rust decimator/result is explicitly test-only; its remaining manifest/stride warnings belong to the still-open SB-PLT-031 allocator path.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `report` 6/53.
- **Git evidence:** reachable commit `f6c9065` integrated shared indices and partial manifests; the Gate 2 completion commit is pending at this evidence write. Two disconnected Rust-helper warnings disappear and two shared allocator warnings remain explicitly owned by SB-PLT-031.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual placement, manual interaction and representative-field evidence remain deliberately open.
- **Next action:** retain the shared-index and portable-manifest contracts, execute the separate review/Gate 4 scenarios, and proceed serially to SB-PLT-016 without importing deferred allocation or implicit DIO resampling scope.

## SB-PLT-016 - Depth-step reconciliation is explicit and conservative

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intentions T23-T26; sections 4.3, 6, 7.3 R-8 and 8.4 OPEN-X7.
- **Atomic obligations:** keep equal steps unchanged; decimate exact integer multiples to the coarsest step with factor disclosure; refuse non-integer ratios and route to explicit DIO resampling; retain half-open intervals.
- **Current source:** `plotTypes.ts::reconcileDepthChannels` validates every complete input grid before its identical-grid shortcut, keeps equal regular grids unchanged, decimates exact integer multiples to the coarsest grid with per-input factors and refuses irregular or non-integer grids with typed `DepthGridReconciliationError` metadata. Crossplot, Pickett, Histogram and shared context loading render the same `Open Reframe` control; Ribbon routes the user's click to the existing Reframe workspace. No plot consumer calls Reframe or resamples automatically. Half-open slicing remains `[lo,hi)`.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::equal_and_exact_multiple_regular_depth_grids_proceed_with_reported_factors_while_non_integer_or_irregular_grids_refuse_with_an_explicit_reframe_action_and_intervals_stay_half_open` is CORRECTNESS. It executes equal and 0.5/1.0 grids, both irregular-identical and 0.5/0.8 refusals, the `[100,101)` fixture, typed handoff construction, the visible control and event route, inventories every pilot consumer and proves no panel invokes Reframe itself. Deliberately changing the event name made the exact test RED before restoration.
- **Supporting tests:** `plotting.rs::equal_and_integer_multiple_depth_steps_proceed_but_non_integer_steps_route_to_dio_and_intervals_stay_half_open` independently pins the cited arithmetic and interval fixture. Its duplicate reconciliation oracle is test-only, removing two disconnected production warnings without weakening the Rust proof.
- **Manual evidence:** `reframe` 0/34, `crossplot` 6/13, `pickett` 0/8 and `histogram` 5/22.
- **Git evidence:** reachable commit `16cfcb1` integrated the original conservative helpers; the Gate 2 completion commit is pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual placement, manual interaction and representative-field evidence remain deliberately open.
- **Next action:** retain exact regular-grid validation, the non-mutating handoff and half-open intervals; execute the separate review/Gate 4 scenarios; proceed serially to SB-PLT-017 without inventing a viewport fetch policy.

## SB-PLT-017 - Zoom beyond loaded data triggers an identified refetch

- **Chapter evidence:** P1; chapter status `ABSENT`; chapter intentions T27-T28; sections 4.3, 6 and 8.1.
- **Atomic obligations:** detect viewport crossing of the loaded interval; issue one new half-open identified request with a generation token; discard stale responses; never imply that stretched/empty old samples are newly loaded data.
- **Current source:** `viewportRefetch.ts::ViewportRefetchCoordinator` carries one source/interval/pixel-density identity, requests when the view crosses that interval or requires denser samples, collapses an identical in-flight request and attaches a monotonic local generation before loading. Only the current generation can apply. `logViewPanel.ts` seeds the identity from the structural whole-well extent, routes settled depth views through the coordinator, preserves the extent while replacing only visible series, and displays both provisional-data and failure notices instead of implying old samples are newly loaded. `equations.rs` applies the same half-open high-bound exclusion to current and explicit-set track reads before decimation; no row or sampling is written.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::a_viewport_crossing_its_loaded_high_bound_issues_one_generation_tagged_half_open_refetch_and_only_the_newest_reverse_order_response_renders` is CORRECTNESS. It executes contained, crossed-bound, duplicate, reverse-order, pending and failure paths, checks the exact tagged interval, and inventories the live panel route. Inverting the newest-generation guard made the test RED before restoration.
- **Supporting tests:** `equations.rs::explicit_track_set_keeps_its_native_grid_and_filters_before_decimation` exercises the real pre-decimation query and excludes the high endpoint; `plotting.rs::equal_and_integer_multiple_depth_steps_proceed_but_non_integer_steps_route_to_dio_and_intervals_stay_half_open` retains the independent cited interval oracle as test-only code, removing its disconnected production warning.
- **Manual evidence:** `log-view` 5/47; visual, manual and representative-field checks are open in `REVIEW.md`.
- **Git evidence:** the prior local generation guard was unowned and did not track loaded density or disclose provisional/failure state; the Gate 2 completion commit is pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for Gate 2 automation. Visual clarity, interaction timing and representative-field evidence remain deliberately open.
- **Next action:** retain the identified disposable-load contract, half-open query and visible pending/failure states; execute the separate review/Gate 4 scenarios; skip deferred SB-PLT-018 and proceed serially to SB-PLT-019.

## SB-PLT-018 - Linked selections are named, typed and persistable

- **Chapter evidence:** P1; chapter status `PARTIAL`; chapter intention T29; sections 4.4, 6, 7.1 O-6 and 8.1.
- **Atomic obligations:** persist ID, label, colour, well set, predicate or exact membership, creation source and data revision; permit multiple coexisting selections; persist any selection used by computation while allowing hover to remain ephemeral.
- **Current source:** `state.ts` holds one process-memory `BrushSelection` containing only `wellId` and exact depth set; a new brush replaces the previous one. Plot provenance hashes that ephemeral set but does not persist a selection record. No database/project owner, multiple-selection collection, label, colour, predicate, source or data revision exists.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::characterizes_linked_brushing_as_one_ephemeral_scope_with_exact_depth_membership` explicitly characterizes replacement and the two-field shape rather than defending it. Test class is `CHARACTERIZATION`.
- **Supporting tests:** panel brush rendering proves cross-panel visibility only.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `vega` 0/2 and `project-lifecycle` 3/24.
- **Git evidence:** accepted anchor contains the divergent one-selection state; the persistence owner recorded by O-6 is absent.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** O-6 requires a governed persistent record; the current computation provenance can reference an ephemeral selection that cannot be recovered.
- **Next action:** define the DBM-owned selection record, support multiple active IDs and implement T29 across save/reload and a computation consumer.

## SB-PLT-019 - Every plot subscribes to the same invalidation contract

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; chapter intention T32; sections 4.4, 6 and 8.1.
- **Atomic obligations:** redraw on theme, data revision, interval, selection and size changes; dispose every subscription and cancel pending work for every plot.
- **Current source:** histogram, crossplot, Pickett and Vega subscribe to theme/data/selection and dispose their handlers; correlation subscribes to theme/data/size but not linked selection. Each panel implements its own event set rather than one shared contract. Several panel-specific generation counters cancel stale effects, but no exhaustive registration inventory proves every pending task is cancelled.
- **Qualifying acceptance tests:** none; T32's four-open-panel theme scenario and universal disposal/event inventory are missing. Test class is `MISSING`.
- **Supporting tests:** `tools/frontend-acceptance.test.mjs::a_superseded_async_plot_build_is_disposed_before_it_can_replace_the_active_panel` covers the workspace build seam, not all invalidations.
- **Manual evidence:** `themes-language-accessibility` 2/52, `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `vega` 0/2 and `correlation-tops` 0/36.
- **Git evidence:** accepted anchor contains broad but independently maintained subscriptions and the correlation selection gap.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one shared registration contract and T32 are missing.
- **Next action:** make the five plot kinds declare one invalidation/disposal inventory and implement T32 plus data/interval/selection/size controls.

## SB-PLT-020 - Plot-derived parameter writes carry full provenance

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intentions T30-T31; sections 4.4, 6 and 8.1.
- **Atomic obligations:** make every handle, marker, polygon or fit write undoable; carry plot identity/type, typed axis bindings/units/revisions, viewport, selection, interval, method, fit record where applicable, user and UTC timestamp; reject null source metadata.
- **Current source:** `plotCommon.ts::writePlotParameter` validates complete provenance through the backend before the whitelisted zone-parameter write and registers exact undo/redo. Histogram markers, crossplot handles/Thomas-Stieber endpoints and Pickett M/A_RW rows use it. The crossplot polygon calls `runNetFlag` directly, carries no plot-provenance record and has no UI undo path, contradicting the explicit polygon clause.
- **Qualifying acceptance tests:** none; T30/T31 do not exercise every writer and the polygon counter-path. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::a_plot_derived_parameter_write_is_undoable_and_requires_complete_non_null_provenance` validates the backend record and source-orders the shared adapter, but does not execute the complete writer inventory.
- **Manual evidence:** `curve-editing` 5/5, `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `processing-history` 0/7.
- **Git evidence:** reachable commit `f4a27cb` integrated the safe parameter writer; the accepted polygon path remains a provenance/undo bypass.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the polygon writer violates the universal contract; whole-inventory T30/T31 are missing.
- **Next action:** route the polygon result through an undoable provenance-complete writer or disable it, then execute T30/T31 against every plot-derived write route.

## SB-PLT-021 - Expression-valued channels are sandboxed and reproducible

- **Chapter evidence:** P2; chapter status `PARTIAL`; no direct whole-contract chapter test; sections 4.4, 6 and 8.1.
- **Atomic obligations:** use only the governed equation runtime; persist expression, dependencies and data revision; unit-check outputs; prevent arbitrary panel scripting from becoming a scientific calculation path.
- **Current source:** the governed Rust equation runtime exists outside plotting, but plot panels do not route expression channels through it. `vegaPanel.ts` lets a user apply an arbitrary Vega-Lite JSON override containing transform/calculation expressions; the override is process memory, is not dependency/unit/revision recorded and is executed by Vega rather than the governed equation runtime.
- **Qualifying acceptance tests:** none; the chapter assigns no direct test and no refusal/reproducibility test exists. Test class is `MISSING`.
- **Supporting tests:** equation-runtime tests do not constrain the Vega specification surface.
- **Manual evidence:** `vega` 0/2, `equation-engine` 0/11, `processing-history` 0/7 and `security-integrity` 0/63.
- **Git evidence:** accepted anchor contains both the governed equation engine and the divergent Vega override; they are not connected.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** an active scientific transform path bypasses governed expression, dependency, unit and revision records.
- **Next action:** restrict Vega overrides to presentation grammar or route scientific transforms through the governed runtime, then add a two-sided arbitrary-expression refusal and reproducible-expression record test.

## SB-PLT-022 - Faceting precedes decimation

- **Chapter evidence:** P2; chapter status `ABSENT`; chapter intention T34; sections 4.4, 6 and 8.1.
- **Atomic obligations:** partition by facet before assigning point budgets; preserve small groups; report original/displayed counts per facet.
- **Current source:** no governed plot faceting/budget implementation or facet reduction manifest exists. Vega's editable grammar can request facets, but the plotting engine does not own, allocate or report those groups. Targeted source/test/history searches recovered no T34 route.
- **Qualifying acceptance tests:** none; T34 is absent. Test class is `MISSING`.
- **Supporting tests:** multi-well allocation is by well, not by facet, and cannot substitute for T34.
- **Manual evidence:** `vega` 0/2, `crossplot` 6/13 and `portfolio-performance` 0/50.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** no facet record or selected pilot need exists.
- **Next action:** keep faceting out of the pilot until selected; then partition before budget and implement T34 with visible/exported per-facet counts.

## SB-PLT-023 - Every rendered chart is provenance-complete

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intention T35; sections 4.5, 6, 7.1 O-5 and 8.1.
- **Atomic obligations:** persist chart ID/title/type, typed X/Y quantity and unit, citation, publisher, revision/date, digitizer when required, approved derivation path, payload checksum and applied transform; block every deliverable render with any mandatory field missing.
- **Current source:** `crossplotPanel.ts::authorizeProvenancedChart` constructs the complete record only after typed-axis authorization and visibly refuses missing/invalid source metadata. The same `drawStatic` path feeds screen, SVG and PDF. Current generated chart definitions provide no accepted provenance, so those overlays are blocked rather than rendered. The record is retained in last-used options/getState, but named template/project portability and a complete positive deliverable path are not proven.
- **Qualifying acceptance tests:** none; T35's deliverable refusal is represented only by an internal validator and source-order assertion, not an executed screen/export/save route. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::a_chart_record_missing_its_source_revision_cannot_render_in_a_deliverable` checks every record field, an invalid revision and authorization-before-draw source order.
- **Manual evidence:** `chart-overlays` 16/53, `report` 6/53, `office-deliverables` 0/39 and `project-lifecycle` 3/24.
- **Git evidence:** reachable commit `113916c` added the fail-closed gate; accepted source contains no known provenance bypass for the chartbook draw call.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** T35's observable refusal/positive route and O-5's rights/provenance evidence are missing.
- **Next action:** add a screen/save/template/SVG/PDF T35 fixture with one complete public-primary record and one missing revision; keep current unapproved overlays blocked.

## SB-PLT-024 - Vendor chart payloads are never transcribed

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; no direct whole-contract chapter test; sections 4.5, 6, 7.1 O-5 and 8.1.
- **Atomic obligations:** keep vendor tables, vertices and lookup payloads out of the repository/product; use only a licensed source or independently digitized published primary source with its own provenance; allow metadata-only inventories.
- **Current source:** generated `src/ui/chartOverlays.ts` declares digitization from a commercial chartbook and contains 19 chart definitions with numeric curve/point/line/region payloads. `docs/IP_PROVENANCE.md` section 2.1 states that these extracted numeric values ship and calls the item the product's highest-exposure legal question. `docs/takeover/CLAIMS.md` CLAIM-013 is `LEGAL-REVIEW` and a first-sale blocker. No payload coordinates are reproduced in this receipt.
- **Qualifying acceptance tests:** none; the chapter supplies no direct test and repository inventory is factual evidence, not a legal entitlement. Test class is `MISSING`.
- **Supporting tests:** generated-file and build checks establish presence/reachability only; they cannot decide copyright or licence scope.
- **Manual evidence:** `chart-overlays` 16/53, `security-integrity` 0/63 and `verification-stewardship` 0/24.
- **Git evidence:** accepted anchor integrates the generated payload and renderer; Git reachability proves shipment, not permission.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** legal counsel must select a permitted route under O-5/CLAIM-013 before first sale; engineering cannot declare the payload licensed.
- **Next action:** obtain counsel's disposition and then either document a licensed source, re-derive from independently published primary sources with full provenance, or remove the payload from the paid build while preserving only lawful metadata/tooling.

## SB-PLT-025 - Plot templates are schema-versioned and scope-aware

- **Chapter evidence:** P1; chapter status `PARTIAL`; chapter intention T36; sections 4.5, 6 and 8.1.
- **Atomic obligations:** declare application scope, schema version, migration path, semantic bindings, parameters and provenance dependencies; show an apply diff; preserve unknown fields or refuse migration explicitly.
- **Current source:** `plotCommon.ts` stores named raw JSON documents per plot kind and recalls them without a schema envelope, migration registry, semantic binding record, provenance dependency list or apply diff. Crossplot normalization spreads unknown fields but deliberately clears stale chart provenance for safe reconstruction; last-used property saves swallow write failures.
- **Qualifying acceptance tests:** none; T36's full save/reload or migration-refusal route is not exercised, and one normalizer clause cannot close the compound contract. Test class is `MISSING`.
- **Supporting tests:** `tools/frontend-acceptance.test.mjs::an_unknown_future_template_field_survives_crossplot_option_normalization` is correctness evidence for unknown-field preservation in one normalizer only.
- **Manual evidence:** `project-lifecycle` 3/24, `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `verification-stewardship` 0/24.
- **Git evidence:** accepted anchor contains the raw per-kind document mechanism; no versioned template contract is integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** schema/scope/migration/binding/provenance/diff clauses and whole T36 are missing.
- **Next action:** define a versioned template envelope and migration/diff contract, then implement T36 through durable save/reload with semantic and provenance dependencies.

## SB-PLT-026 - Export reruns the scientific draw at paper scale

- **Chapter evidence:** P1; chapter status `PARTIAL`; chapter intentions T37-T38; sections 4.5, 6 and 8.1.
- **Atomic obligations:** rerun the same plot state/vector draw for SVG/PDF at paper scale; retain axes, legends, annotations, exclusion counts and provenance footer; prove no crop; explicitly label raster print.
- **Current source:** histogram, crossplot and Pickett rerun their shared `drawStatic` callbacks through SVG/PDF recording contexts. Correlation offers only shared raster actions; Vega has its own SVG path and no shared PDF route. Exports use live pixel dimensions rather than a declared paper-scale contract, provenance footers/crop proofs are incomplete, and `printCanvas` opens a PNG while the action label says only `Print`.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::characterizes_vector_exports_as_labelled_while_the_png_print_path_is_not_labelled_raster` explicitly characterizes the current split instead of asserting completion. Test class is `CHARACTERIZATION`.
- **Supporting tests:** vector-render tests elsewhere prove drawing mechanics, not T37's long-label/legend/crop/provenance fixture or T38's raster label/footer.
- **Manual evidence:** `report` 6/53, `office-deliverables` 0/39, `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `vega` 0/2.
- **Git evidence:** accepted anchor contains the partial shared vector and divergent raster paths.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** paper-scale state, complete metadata/footer/crop proof and an explicitly labelled raster route are missing.
- **Next action:** define one export record/draw contract across plot kinds and implement T37/T38 with long labels, outside legend, provenance footer, reduction metadata and raster labelling.

## SB-PLT-027 - Plot state is portable without embedding restricted payloads

- **Chapter evidence:** P1; chapter status `PARTIAL`; no direct whole-contract chapter test; T35-T36 provide only cross-cutting support; sections 4.5, 6, 7.1 O-5 and 8.1.
- **Atomic obligations:** save approved chart IDs and checksums without embedding restricted payloads; visibly fail missing referenced content; keep project/template state portable.
- **Current source:** crossplot options/templates refer to `chartOverlay` by ID and may carry a reconstructed provenance record, not the numeric overlay arrays. Unknown/missing IDs are silently normalized to an empty string, so content loss is not visible. The repository/binary still supplies the generated payload addressed by that ID, leaving O-5 as a separate first-sale issue.
- **Qualifying acceptance tests:** none; no test saves a referenced ID/checksum, removes the content and observes a visible failure. Test class is `MISSING`.
- **Supporting tests:** T35's provenance refusal and T36's unknown option-field preservation do not prove reference portability or missing-content behavior.
- **Manual evidence:** `project-lifecycle` 3/24, `chart-overlays` 16/53, `report` 6/53 and `security-integrity` 0/63.
- **Git evidence:** accepted anchor integrates ID-based option state and the silent-clear counter-path; no portable content registry is integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** missing references fail silently, checksums are not a portable registry key, and O-5/CLAIM-013 remains unresolved.
- **Next action:** introduce an approved ID/checksum content registry with visible unresolved-reference state and test project/template transfer without embedding any restricted payload.

## SB-PLT-028 - Static and interaction layers have separate invalidation

- **Chapter evidence:** P1; chapter status `PARTIAL`; no direct whole-contract chapter test; T28/T32/T33 provide cross-cutting support; sections 4.6, 6 and 8.1.
- **Atomic obligations:** separate cached axes/grids/invariant overlays from hover/brush/drag feedback; memoize ranges, sorted quantiles and transformed arrays by data revision and plot options.
- **Current source:** crossplot separates `drawStatic` from transient feedback and memoizes Z colours by data/options revision. Every hover/brush redraw still reruns `drawStatic`, axes, grids and invariant overlays; ranges/quantiles/transforms are not comprehensively memoized. Other plot kinds have independent redraw lifecycles.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::characterizes_crossplot_static_draw_and_z_colours_as_separately_invalidated_subsets` explicitly labels the current subset rather than the specified complete event/cache matrix. Test class is `CHARACTERIZATION`.
- **Supporting tests:** async generation/disposal and theme redraw tests concern lifecycle safety, not cache separation.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `vega` 0/2 and `portfolio-performance` 0/50.
- **Git evidence:** accepted anchor contains partial function/memo boundaries; no complete cache layer is integrated.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DEGRADED-RESULT`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot need depends on measured SB-PLT-032 results; no performance threshold is inferred from source structure.
- **Next action:** after release hardware/gates are defined, measure the event matrix and implement only the evidenced static/range/quantile/transform caches with observable invalidation tests.

## SB-PLT-029 - Asynchronous plot loads are generation-safe

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; chapter intentions T28 and T33; sections 4.6, 6 and 8.1.
- **Atomic obligations:** attach a generation token to every asynchronous build/refetch; dispose every superseded result before it can mutate the active panel.
- **Current source:** `workspace.ts::createPlot` and well-bound builders generation-guard async panel replacement and dispose stale content. Histogram, crossplot, Pickett, correlation, Vega and context loaders each carry generation/disposed checks; Vega finalizes stale views. There is no executable inventory proving every asynchronous branch, including dynamic editor/export dependencies, remains registered.
- **Qualifying acceptance tests:** none; the available T28/T33 test inspects the shared build function rather than resolving real competing panel promises across the complete async inventory. Test class is `MISSING`.
- **Supporting tests:** `tools/frontend-acceptance.test.mjs::a_superseded_async_plot_build_is_disposed_before_it_can_replace_the_active_panel` is correctness evidence for the shared workspace seam; focused source review supports panel-local guards.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `vega` 0/2, `correlation-tops` 0/36 and `workflow` 0/23.
- **Git evidence:** accepted anchor contains broad generation-safe mechanisms; no current stale-write counter-path was recovered.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** an executable registration inventory and reverse-order end-to-end T28/T33 are missing.
- **Next action:** register every async plot operation and implement reverse-order completion/disposal tests through actual panel replacement and data-revision refetch.

## SB-PLT-030 - Interactive canvases remain keyboard and assistive-technology reachable

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; chapter intention T39; sections 4.6, 6 and 8.1.
- **Atomic obligations:** give every canvas a current accessible label and keyboard focus/pan/zoom; provide non-pointer routes to properties and export.
- **Current source:** histogram, crossplot and Pickett use `makeCanvasAccessible`, keyboard pan/zoom and toolbar/context-menu actions. Correlation's hand-built canvas lacks the shared accessibility/keyboard helpers, and Vega's generated canvas has no SandiBumi keyboard-pan contract. The shared workspace provides export menus for some but not all vector paths.
- **Qualifying acceptance tests:** none; T39 exercises one fake canvas/helper, not every active canvas and non-pointer property/export route. Test class is `MISSING`.
- **Supporting tests:** `tools/frontend-acceptance.test.mjs::a_focused_accessible_canvas_changes_view_by_keyboard_and_removes_the_handler_on_dispose` is correctness evidence for the shared helper from focus through disposal.
- **Manual evidence:** `themes-language-accessibility` 2/52, `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `correlation-tops` 0/36 and `vega` 0/2.
- **Git evidence:** accepted anchor contains compliant and noncompliant canvas families.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** correlation/Vega and the universal non-pointer/export inventory are not covered.
- **Next action:** bring every plot canvas under one accessible interaction contract and implement T39 as an inventory test with current-label and non-pointer properties/export controls.

## SB-PLT-031 - No silent record truncation

- **Chapter evidence:** P0; chapter status `PARTIAL`; chapter intention T40 with T20-T22/T34 support; sections 4.6, 6 and 8.1.
- **Atomic obligations:** disclose before/after counts for every load, point, well, facet, legend and visual limit; export the reduction manifest; refuse hard maxima instead of returning a prefix.
- **Current source:** context point budgets, scope-name preview and well legends expose counts/reasons and produce an exportable manifest; too-small endpoint budgets refuse. Vega refuses more than its group maximum rather than slicing. The manifest omits some reduction details, faceting is absent, and visual text truncation plus the complete limit inventory are not represented by portable before/after records.
- **Qualifying acceptance tests:** none; T40 exercises a Rust manifest/helper plus source strings, not every active limit and export route. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::an_export_after_budget_reduction_includes_original_and_displayed_counts_and_the_algorithm_while_a_hard_maximum_refuses` pins the cited reduction/refusal and inventories several UI adapters.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `vega` 0/2, `report` 6/53 and `portfolio-performance` 0/50.
- **Git evidence:** reachable commit `cdd444f` integrated the reduction-manifest path; accepted source retains unregistered visual/facet limits.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** an exhaustive limit registry and complete manifest/export test are missing.
- **Next action:** register every plot limit and its refusal/reduction record, include all portable reduction fields, and implement T40 against each registered consumer.

## SB-PLT-032 - Plot performance is gated on declared hardware

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intention T43; sections 4.6, 5, 6, 7.1 O-1 and 8.1.
- **Atomic obligations:** gate cold load, first useful paint, pan/zoom, selection, memory and export for single/multi-well fixtures; name hardware, dataset, curve/point/well counts and software revision; use approved thresholds.
- **Current source:** no release benchmark harness or qualified report covers the named metric matrix. `portfolio-performance` is unexercised. `INTERACTION_GATE` and `FIRST_PAINT_GATE` are intentionally absent under O-1; developer-machine source timing cannot replace them.
- **Qualifying acceptance tests:** none; T43 is absent. Test class is `MISSING`.
- **Supporting tests:** ordinary functional and ignored stress tests do not name the release hardware, complete metric set or approved gates.
- **Manual evidence:** `portfolio-performance` 0/50 and `verification-stewardship` 0/24.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying release benchmark/report is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** O-1 requires declared Windows release hardware, representative fixtures and owner-adopted interaction/first-paint gates; no number may be guessed.
- **Next action:** Jauhar defines the release-hardware/fixture claim and adopts measured gates, then engineering implements and runs T43 twice with all required metadata.

## SB-PLT-033 - Pressure-gradient crossplots preserve the geomechanics sign convention

- **Chapter evidence:** P1; chapter status `ABSENT`; no direct whole-contract chapter test; sections 4.7, 6, 7.1 O-7 and 8.1.
- **Atomic obligations:** accept typed pressure/depth channels; show selected datum/sign; persist picks with full provenance; perform no pressure/fracture-gradient calculation in the plotting shell.
- **Current source:** generic crossplot and correlation surfaces can display selected curves/depth frames but expose no typed pressure-gradient shell, governed datum/sign label or provenance-complete pressure pick. Targeted source, tests and history recovered no qualifying implementation. Chapter 18 remains the equation/datum owner.
- **Qualifying acceptance tests:** none; the chapter supplies no direct test. Test class is `MISSING`.
- **Supporting tests:** generic crossplot and TVDSS correlation behavior do not establish the geomechanics sign/datum contract.
- **Manual evidence:** `crossplot` 6/13, `correlation-tops` 0/36 and `verification-stewardship` 0/24.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** O-7 and the chapter-18 datum/sign contract must close, and DEC-003 must select a pilot geomechanics workflow; no convention is invented here.
- **Next action:** if selected after chapter 18 closes, implement a display-only typed shell with provenance-complete picks and an owned cross-domain acceptance test.

## SB-PLT-034 - Ternary plots normalize visibly

- **Chapter evidence:** P2; chapter status `ABSENT`; chapter intentions T41-T42; sections 4.7, 5, 6, 7.1 O-4 and 8.1.
- **Atomic obligations:** declare units and normalization; flag non-finite/negative components; show pre-normalization sum; preserve originals; never silently turn invalid volumes into a plausible point.
- **Current source:** targeted source, tests and reachable history found no ternary plot, normalization record or invalid-component surface. `TERNARY_SUM_TOL` deliberately ships absent under O-4.
- **Qualifying acceptance tests:** none; T41/T42 are absent. Test class is `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** `equation-engine` 0/11, `chart-overlays` 16/53 and `verification-stewardship` 0/24.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** O-4 requires a house exact-sum policy or cited numerical tolerance; no plausible tolerance is selected.
- **Next action:** keep ternary plotting absent until that policy is sourced, then implement T41/T42 with preserved originals and visible pre-sum/invalid flags.

## SB-PLT-035 - Clay-volume interactive plots use the governed equation

- **Chapter evidence:** P2; chapter status `PARTIAL`; no direct whole-contract chapter test; sections 4.7, 6 and 8.1.
- **Atomic obligations:** call the same versioned equation and parameter schema as batch computation; route every endpoint write through SB-PLT-020; contain no hidden duplicate formula in the UI.
- **Current source:** `crossplotPanel.ts::drawTsOverlay` duplicates the Thomas-Stieber equations in TypeScript rather than calling `modules.rs::thin_bed_ts`. Endpoint handle writes use the provenance-complete parameter adapter, but the duplicated draw can drift independently. The polygon bypass recorded in SB-PLT-020 remains separate.
- **Qualifying acceptance tests:** `modules.rs::characterizes_the_interactive_clay_overlay_as_a_duplicate_formula_matching_batch_endpoints` explicitly characterizes algebraic endpoint agreement and asserts that the UI does not call the governed module. Test class is `CHARACTERIZATION`.
- **Supporting tests:** batch module tests prove its equation behavior, not single-definition governance.
- **Manual evidence:** `shale-volume` 0/17, `crossplot` 6/13, `equation-engine` 0/11 and `processing-history` 0/7.
- **Git evidence:** accepted anchor integrates both the batch equation and divergent UI duplicate; no shared governed call is present.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** numerical agreement today is a snapshot, not proof that interactive and batch paths cannot drift.
- **Next action:** make the interactive overlay consume the governed equation/schema and add an observable test that fails if a second formula or non-SB-PLT-020 endpoint writer reappears.

## Domain result

- All 35 live SB-PLT rows were adjudicated exactly once against the accepted implementation anchor.
- As-built: 0 `PRESENT-OK`, 4 `PRESENT-UNVERIFIED`, 10 `PRESENT-DIVERGENT`, 14 `PARTIAL`, 7 `ABSENT`.
- Release disposition: 29 `PILOT-BLOCKER`, 4 `UNDECIDED`, 2 `DEFERRED`, 0 `OUT`.
- Test evidence: 6 `CHARACTERIZATION`; 29 `MISSING` qualifying whole-contract proofs. The 15 focused plotting tests, one clay characterization and relevant frontend tests passed but are not overclaimed beyond their inspected assertion surfaces.
- Integrated state: 28 rows have current mechanisms or divergences in reachable source; 7 rows are `UNIMPLEMENTED`.
- Hard evidence blocks preserved: O-1 release hardware and `INTERACTION_GATE`/`FIRST_PAINT_GATE`; O-2 row-level unit-limit audit; O-3 robust-regression method/tuning; O-4 `TERNARY_SUM_TOL`; O-5/CLAIM-013 chart rights; O-6 persistent selection ownership; O-7 geomechanics datum/sign ownership; Pickett `a`, `m`, `n`, `Rw`; and DEC-003 pilot workflow selection.
- No production code, test, generated chart payload, PRD, scientific parameter, legal conclusion or manual verification result was changed.
