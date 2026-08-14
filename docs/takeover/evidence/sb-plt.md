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
- **Current source:** `crossplotPanel.ts::resolveAxisRange` uses the declared order and draws the winning tier, but the live caller always supplies `headerDisplay: null` and exports no tier record. Histogram, Pickett, correlation and Vega construct ranges independently. The equivalent Rust resolver is unused outside tests.
- **Qualifying acceptance tests:** none; T01/T02 are not exercised through a panel with a real header range and export. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::a_user_axis_range_wins_and_without_it_the_header_display_range_wins` proves the unused Rust helper's precedence and validity exclusion.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `vega` 0/2.
- **Git evidence:** reachable commit `3844ae5` added the partial crossplot chain; current independent panel paths remain at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no shared live range resolver, header-range input or export-tier record exists.
- **Next action:** route every plot axis through one governed resolver with real header metadata and add T01/T02 at the rendered/exported surface.

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
- **Current source:** crossplot live code separates the policies and reports non-finite, log-domain, display-hidden and validity-excluded counts. Histogram discloses its visible-bin count versus finite total but has no validity-range surface. Pickett fixed display ranges clip without a hidden count, and other plot families do not share the crossplot policy. The Rust range helper is unused.
- **Qualifying acceptance tests:** none; no test changes only display range and only validity range through every active plot/statistic/fit surface. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::display_clipping_counts_hidden_points_while_validity_filtering_changes_the_population_explicitly` proves the two-sided arithmetic contract in an unused helper.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `chart-overlays` 16/53.
- **Git evidence:** reachable commit `e9cc980` integrated the crossplot subset; no universal panel route exists at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the distinction is not governed across every live plot/statistic/fit path.
- **Next action:** centralize the two policies and implement a rendered two-sided test proving clipping leaves n unchanged while explicit validity filtering changes and discloses n and fit inputs.

## SB-PLT-005 - Unit-limit content is audited before activation

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intention T05; sections 4.1, 6, 7.1 O-2 and 8.1.
- **Atomic obligations:** reject schema-valid content as authority; dimensionally re-derive every converted pair before activation; leave every suspect row disabled with its reason.
- **Current source:** `plotting.rs::audit_unit_limit_pair` implements an exact registered-conversion audit but is unused. `crossplotPanel.ts::axisDefaults` and Pickett defaults remain active hard-coded ranges without a row-level audit registry or preserved disable reason. The chapter explicitly records that the complete row audit is still open.
- **Qualifying acceptance tests:** none; T05 covers one documented divergent pair, not the universal active-limit inventory. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::a_dimensionally_divergent_unit_limit_row_stays_disabled_with_its_reason` proves only one unused audit-helper refusal using the chapter's cited divergence.
- **Manual evidence:** `crossplot` 6/13, `pickett` 0/8 and `verification-stewardship` 0/24.
- **Git evidence:** reachable commit `aefeb6b` added the unused audit helper; accepted product paths still activate unaudited families.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** O-2's row-level dimensional screen and one governed activation registry are missing; no limit may be inferred meanwhile.
- **Next action:** inventory every active unit-limit row, re-derive each from primary unit definitions, disable unproved rows with reasons and implement an exhaustive activation-gate test.

## SB-PLT-006 - One canonical histogram-bin contract

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; chapter intentions T06-T07; sections 4.2, 5, 6 and 8.1.
- **Atomic obligations:** use half-open bins with only the final upper endpoint included; count NaN/infinity exclusions separately; make displayed total equal the bin-count sum through every histogram surface.
- **Current source:** the main histogram panel uses `src/distribution.ts::histogram`, defaults to the cited 50 bins and clamps to 1-200 while reporting non-finite exclusions and count totals. Crossplot marginals reimplement binning without the shared exclusion report, and Vega delegates to an implicit `bin: true` rule. Rust uses a duplicate implementation. The stale HTML attributes still advertise 5-400 even though normalization clamps them.
- **Qualifying acceptance tests:** none; T06/T07 run only through the Rust helper and do not cover the main panel, marginals, Vega and export as one contract. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::histogram_bins_are_half_open_except_for_the_final_upper_endpoint_and_non_finite_values_are_counted` and distribution unit tests pin the shared Rust arithmetic, not the divergent live inventory.
- **Manual evidence:** `histogram` 5/22, `crossplot` 6/13, `vega` 0/2 and `report` 6/53.
- **Git evidence:** reachable commit `b45427a` migrated the primary histogram path; accepted source retains independent marginal and Vega rules.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** multiple active binning definitions and no product-wide T06/T07 proof remain.
- **Next action:** route all plotting histograms through one canonical contract, align the property affordance, and test both endpoints and non-finite disclosure through each live consumer and export.

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
- **Current source:** `plotCanvas.ts::basicStats` returns only count, extrema, mean, selected percentiles and standard deviation. Histogram labels some counts, crossplot labels selected exclusions, and Vega computes additional summaries independently; none returns the complete governed metadata record.
- **Qualifying acceptance tests:** `tools/frontend-acceptance.test.mjs::characterizes_finite_statistics_without_population_or_exclusion_metadata`; expected arithmetic is sourced from T12, while the missing result fields explicitly characterize current behavior. Test class is `CHARACTERIZATION`.
- **Supporting tests:** distribution tests independently pin arithmetic summaries but do not prove disclosure.
- **Manual evidence:** `histogram` 5/22, `crossplot` 6/13, `vega` 0/2 and `report` 6/53.
- **Git evidence:** accepted anchor contains the partial statistics surfaces; no complete record commit is integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** estimator/population/provenance metadata and T13's governed box summary are missing.
- **Next action:** define one statistics result record and render/export every field; implement T12/T13 without using today's result shape as the expected source.

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
- **Current source:** crossplot screens X/Y and applies the TypeScript colour policy for Z with exclusion/clamp/edge counts. Histogram handles non-finite values but not the full channel matrix. Pickett and Vega use separate policies, and array-waveform plotting does not call the shared policy. `plotting.rs::apply_plot_channel_policy` covers all named shapes but is unused outside tests.
- **Qualifying acceptance tests:** none; the chapter has no direct test and no observable inventory test proves all channels, disclosures and source immutability. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::missing_log_xy_z_and_waveform_values_follow_their_own_reported_policies` proves the unused helper from both sides and checks source bits remain unchanged.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8, `log-view` 5/37 and `chart-overlays` 16/53.
- **Git evidence:** reachable commit `cfc468b` added the policy helper and crossplot subset; accepted source retains independent consumers.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no universal product call path or whole-contract acceptance test exists.
- **Next action:** route every X/Y/Z/waveform consumer through one non-mutating policy and add a live inventory test covering both exclusion and clamp/edge disclosure.

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
- **Current source:** `plotTypes.ts::allocateFinitePairBudget` returns one source-index vector and a complete in-memory `ReductionManifest`; `fetchContextLayers` applies it to depth and every series. `describeContextOutcome` visibly reports reduction, stride and forced endpoints. The export manifest retains counts and algorithm but drops stride and forced-endpoint state, and there is no universal consumer inventory.
- **Qualifying acceptance tests:** none; T21/T22 exercise the Rust helper, not the active panel, visible disclosure and exported provenance together. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::decimation_uses_one_shared_index_vector_and_reports_the_forced_final_endpoint` independently pins the cited indices, all channels and complete helper manifest.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `report` 6/53.
- **Git evidence:** reachable commit `f6c9065` integrated shared indices and manifests; accepted export still omits two required provenance fields.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** complete portable provenance and a whole-route acceptance test are missing.
- **Next action:** preserve stride and forced-endpoint state in exported/saved plot records and implement T21/T22 through one panel plus export.

## SB-PLT-016 - Depth-step reconciliation is explicit and conservative

- **Chapter evidence:** P0; chapter status `ABSENT`; chapter intentions T23-T26; sections 4.3, 6, 7.1 O-7 and 8.1.
- **Atomic obligations:** keep equal steps unchanged; decimate exact integer multiples to the coarsest step with factor disclosure; refuse non-integer ratios and route to explicit DIO resampling; retain half-open intervals.
- **Current source:** `plotTypes.ts::reconcileDepthChannels` is used by crossplot, Pickett and context loading and implements exact-depth alignment, integer-factor decimation and half-open slicing. A non-integer ratio produces an error string naming DIO but no actionable DIO route. The identical-grid shortcut does not validate the full grid's regular step, and independent Rust helpers are unused.
- **Qualifying acceptance tests:** none; T23-T26 do not run through the active TypeScript panel/refusal route. Test class is `MISSING`.
- **Supporting tests:** `plotting.rs::equal_and_integer_multiple_depth_steps_proceed_but_non_integer_steps_route_to_dio_and_intervals_stay_half_open` pins the chapter's shown arithmetic and interval fixture.
- **Manual evidence:** `reframe` 0/34, `crossplot` 6/13, `pickett` 0/8 and `histogram` 5/22.
- **Git evidence:** reachable commit `16cfcb1` integrated the conservative helpers and panel use; the actionable DIO handoff and complete grid validation remain absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the refusal is not an explicit resampling workflow handoff, and the active route lacks T23-T26.
- **Next action:** validate every regular grid, expose a bounded DIO action on refusal and implement all four fixtures through the panel without introducing a tolerance or implicit Reframe.

## SB-PLT-017 - Zoom beyond loaded data triggers an identified refetch

- **Chapter evidence:** P1; chapter status `ABSENT`; chapter intention T27; sections 4.3, 6 and 8.1.
- **Atomic obligations:** detect viewport crossing of the loaded interval; issue one new half-open identified request with a generation token; discard stale responses; never imply that stretched/empty old samples are newly loaded data.
- **Current source:** `plotCanvas.ts::attachZoomPan` and keyboard controls mutate only an in-memory viewport and redraw. Histogram, crossplot and Pickett fetch on curve/zone/data-revision changes, not viewport crossing. Targeted source, tests and reachable history recovered no loaded-interval tracker or viewport-triggered refetch.
- **Qualifying acceptance tests:** none; T27 is absent. Test class is `MISSING`.
- **Supporting tests:** generation tests for ordinary reloads and panel replacement do not issue the required viewport request.
- **Manual evidence:** `crossplot` 6/13, `histogram` 5/22, `pickett` 0/8 and `portfolio-performance` 0/50.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying history is reachable at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the product cannot distinguish panning beyond loaded data from a local view change.
- **Next action:** track the identified loaded interval, issue generation-tagged half-open refetches on boundary crossing and implement T27 with a stale-response control.

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
