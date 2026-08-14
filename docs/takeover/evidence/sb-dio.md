# Gate 1 SB-DIO live adjudication

- Branch: `codex/g1-sb-dio-adjudication`
- Adjudication start HEAD: `1587592afc8a101a604ca73474d22acc2e359bb4`
- Accepted evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`
- `origin/master` at evidence freeze: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Adjudication date: `2026-08-10`
- Worktree at evidence freeze: clean; `D:\XX. SandiBumi` was the only registered Git worktree.
- Row guard: passed - exactly 63 planned SB-DIO rows, all 63 initially `UNADJUDICATED`; priority mix P0 10, P1 41, P2 11 and P3 1.
- Evidence boundary: this receipt classifies the accepted tree. It does not amend PRD v2, supply a missing parameter or format specification, promote automated evidence to field evidence, or approve pilot scope on Jauhar's behalf.
- Source-navigation boundary: the codebase index was not callable in this task, so exact-file `rg` searches and direct source reads were used as the declared fallback. Consequential negative findings are checked against the expected directories, tests, Git history and generated verification matrix.
- Verification boundary: focused tests named below and the final repository gate are recorded after adjudication. A passing existing worktree is not treated as fresh-clone or field evidence.

## SB-DIO-001 - A single declared sentinel MUST reach every writer.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DIO-T01`, `SB-DIO-T02`.
- **Atomic obligations:** resolve one project sentinel; require it at every registered writer boundary; prevent a writer-owned fallback sentinel.
- **Current source:** `src-tauri/src/export.rs` stores one project setting, makes `WriterSettings` a required non-optional registry argument and routes the sole registered LAS writer through it. `src-tauri/src/lib.rs` exposes only the whitelisted setting commands. No second registered data writer exists outside the registry, and both exact tests enumerate the registry instead of assuming slot zero is complete.
- **Qualifying acceptance tests:** `a_declared_sentinel_reaches_every_registered_writer_and_no_writer_emits_its_own` and `a_registered_writer_cannot_omit_the_required_sentinel_argument` are `CORRECTNESS`; the first loops every registered writer and checks its declared/used sentinel, absence of the project default and successful self-read, while the second loops the same inventory and pins the required function type. The non-default control uses the Baker waveform sentinel cited in chapter section 5.2.
- **Supporting tests:** the LAS round trip and default-format test exercise the same setting but do not replace the writer-inventory assertions.
- **Manual evidence:** the generated matrix currently shows `data-conventions` 4/68 and `las-export` 0/2, but legacy checked Automated entries contaminate its checked count; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** reachable `55927c6` contains the closing writer-registry change; the current `codex/g2-program-plan` increment strengthens the registry-wide proof without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the automated contract; field export remains unexercised.
- **Next action:** retain exact T01/T02, preserve the required registry signature and require every future data writer to enter this inventory before it can ship; Jauhar's independent-reader and representative-field checks remain separate evidence.

## SB-DIO-002 - The default export path MUST NOT be the one that bypasses the sentinel.

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DIO-T03`.
- **Atomic obligations:** expose exactly one default writer; require that default to honour the project sentinel; label any incapable format instead of presenting it as equivalent.
- **Current source:** `export.rs::default_writer` rejects an incapable default, the registry has exactly one shipping default, and `export_formats` exposes both capability and limitation fields through the whitelisted `list_data_export_formats` command and typed IPC response. No second data format ships, so there is no current alternate picker choice.
- **Qualifying acceptance tests:** `the_default_export_format_honours_the_sentinel_and_an_incapable_format_is_marked` passed as `CORRECTNESS`; it pins one default, the real default export with the non-default sentinel sourced to chapter section 5.2, and both capability sides for a synthetic incapable format.
- **Supporting tests:** SB-DIO-001's two registry tests protect the required setting argument.
- **Manual evidence:** `las-export` 0/2 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** reachable `9e55f06` contains the default-selection closure; the current `codex/g2-program-plan` increment reverified it against source and exact T03 without adding a fictitious format or picker.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the current one-format product. A future second format creates a new Visual/Manual obligation; the current synthetic capability test is not rendered-picker evidence.
- **Next action:** retain exact T03 and keep default uniqueness plus the capability declaration when another format is added; Jauhar's default export and Gate 4 representative-delivery checks remain separate evidence.

## SB-DIO-003 - "This channel has no null" MUST be a first-class state.

- **Chapter evidence:** P2; chapter status `ABSENT`; owned tests `SB-DIO-T04`, `SB-DIO-T05`.
- **Atomic obligations:** represent explicit no-null separately from an ordinary null list and from no declaration; preserve a genuine sentinel-shaped amplitude only in the explicit no-null case; expose the distinction in the import result.
- **Current source:** `parsers.rs::ChannelNullMode` owns screening while `ChannelNullResolution` records each actual source channel as `unset`, `no_null` or `values`; `ingest.rs::ImportResult` carries the ordered records through success, attach and post-parse refusal paths, and `ipc.ts` exposes the same typed result.
- **Qualifying acceptance tests:** `a_channel_declared_no_null_preserves_a_sentinel_shaped_amplitude_and_reports_no_null` and `an_unset_channel_screens_the_same_sentinel_shaped_amplitude_and_reports_unset` are `CORRECTNESS`; each enters the real LAS importer, asserts the returned source-channel mode and independently queries the stored sample.
- **Supporting tests:** SB-DIO-005/T09 and SB-DIO-006/T10 retain plural-values and many-to-many rule behavior, but neither substitutes for this result-surface proof.
- **Manual evidence:** `data-conventions` 4/122 and `las-import` 0/90 after regeneration; this exact paired scenario remains unchecked and this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** reachable `23d6b28` introduced the first-class screening enum; the current `codex/g2-program-plan` increment closes the formerly absent result-surface contract.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; D-3 and T04/T05 cite the sentinel-shaped control and require the two distinct states without adding a numerical default.
- **Next action:** retain exact T04/T05 and the three-state result type; Jauhar visually and manually verifies both deliveries and Gate 4 repeats them on a representative sanitized file without promoting synthetic automation to field evidence.

## SB-DIO-004 - Null recognition MUST be one relative-tolerance transform, and recognition MUST NOT rewrite.

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T06`, `SB-DIO-T07`, `SB-DIO-T08`.
- **Atomic obligations:** use one relative comparison with the specified 1.0 floor; survive one f32/f64 representation cycle; convert a recognised sentinel only to internal missing; retain finite nonmatches unchanged.
- **Current source:** `parsers.rs::matches_null` is the single comparison helper used by global, declared and per-channel screening; the former absolute comparison is gone and recognition produces only the internal `f32::NAN` missing state.
- **Qualifying acceptance tests:** `null_recognition_is_one_relative_tolerance_transform_and_recognition_never_rewrites` is `CORRECTNESS`; its tolerance and floor come from chapter section 5.2, it pins near-sentinel and f32/f64 recognition plus a surviving finite value and declared-null import, and exact T08 now reads every Rust source through `read_text_file` to require one parser-owned transform and reject the retired epsilon form.
- **Supporting tests:** per-channel and malformed-input tests exercise the helper through full readers.
- **Manual evidence:** the generated matrix currently shows `data-conventions` 4/71 and `las-import` 0/57, but legacy checked Automated entries contaminate its checked count; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** reachable `29d6d7d` contains the closing transform; the current `codex/g2-program-plan` increment adds the missing executable T08 source inventory without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied P0 contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T06/T07/T08, preserve the one-helper inventory and keep environmental correction separate from import recognition; Jauhar's representative import remains Visual/Manual/Field evidence.

## SB-DIO-005 - Null values MUST be per-channel and plural.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T09`.
- **Atomic obligations:** allow multiple null values per channel and screen each channel only against its own override.
- **Current source:** `ChannelNullValues` maps channel names to plural `ChannelNullMode::Values`, and both LAS parse paths merge and apply those per-channel modes.
- **Qualifying acceptance tests:** `two_channels_with_different_plural_nulls_are_screened_against_their_own_values_only` is `CORRECTNESS`; the three sentinel values are cited in chapter section 5.2, and exact T09 drives both shipping LAS readers while pinning plural own-channel screening and cross-channel survival from both sides.
- **Supporting tests:** SB-DIO-006 exercises rule-derived per-channel modes.
- **Manual evidence:** the generated matrix currently shows `data-conventions` 4/74 and `las-import` 0/60, but legacy checked Automated entries contaminate its checked count; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** reachable `ff58416` contains the closing plural override; the current `codex/g2-program-plan` increment closes the two-reader test gap without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T09 across both shipping LAS readers and keep per-channel overrides attached to source channel identity through every new reader; Jauhar's representative import remains Visual/Manual/Field evidence.

## SB-DIO-006 - The null-exception rule shape MUST be many-to-many.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T10`.
- **Atomic obligations:** retain every name pattern and every null value/no-null declaration in one rule; reject ambiguous or empty rules instead of truncating them.
- **Current source:** `NullExceptionRule` stores `names: Vec<String>` plus one plural/no-null mode; resolution compiles all patterns, rejects overlaps and preserves all matches.
- **Qualifying acceptance tests:** `one_null_exception_entry_keeps_all_six_name_patterns_active_and_no_null_is_not_unset` is `CORRECTNESS`; the six-name shape and explicit no-null case are sourced to chapter section 5.2, and exact T10 now loads the serialized one-entry document before requiring all six patterns to remain active.
- **Supporting tests:** SB-DIO-005 verifies plural values after resolution.
- **Manual evidence:** the generated matrix currently shows `data-conventions` 4/77 and `las-import` 0/60, but legacy checked Automated entries contaminate its checked count; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** reachable `23d6b28` contains the closing rule loader; the current `codex/g2-program-plan` increment closes the serialized-boundary proof without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T10 at the serialized boundary and require future rule loaders to deserialize this same shape rather than flattening it; Jauhar's representative rule-and-delivery exercise remains Visual/Manual/Field evidence.

## SB-DIO-007 - Absent MUST be distinguishable from nulled.

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DIO-T11`.
- **Atomic obligations:** preserve source empty-cell versus explicit-sentinel provenance through import and export while both remain `f32::NAN` for arithmetic.
- **Current source:** Intake keeps raw preview strings plus a column-level kind, but import rows reduce empty and explicit-null-shaped cells to `f32::NAN`; curve storage has no per-sample source-state channel and LAS export emits the same project sentinel for every NaN.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** existing Intake tests prove preview and missing-value behavior only; none carries a per-cell empty-versus-explicit-null state into storage or a round-trip deliverable.
- **Manual evidence:** the generated matrix currently shows `delimited-intake` 3/27, `data-conventions` 4/80, `las-export` 0/2 and `verification-stewardship` 6/74, but legacy checked Automated entries contaminate the checked counts; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** no implementation commit exists for the provenance half; commit state is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `BLOCKED` - the chapter requires the distinction to survive but does not select or version its storage and deliverable representation. Choosing a bitset, table column, sidecar or manifest here would invent a data contract; `f32::NAN` must remain the arithmetic value.
- **Next action:** adjudicate one versioned source-cell-state representation and how every supported deliverable carries or accompanies it; then implement exact T11 as the specified import/export round trip without introducing `Option<f32>`.

## SB-DIO-008 - Coverage-aware alias resolution MUST be preserved.

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned tests `SB-DIO-T12`, `SB-DIO-T13` (characterization).
- **Atomic obligations:** choose greatest finite coverage; retain alias-priority order on exact ties by replacing only on strict improvement; remain deterministic.
- **Current source:** the LAS `pick` closure counts finite samples, replaces only when coverage is strictly greater and scans candidates in declared alias order.
- **Qualifying acceptance tests:** `characterizes_greater_finite_coverage_as_winner_and_an_equal_coverage_tie_as_declared_alias_priority` is `CHARACTERIZATION`, matching chapter §6's T12/T13 classification; fixture numbers are row markers, while the expected priority comes from chapter §5.3. It pins the populated-later case plus a reversed-source-order exact tie on two imports.
- **Supporting tests:** SB-DIO-009's correctness test proves that the selected and passed-over coverages reach the import result; it does not own T12/T13's choice behavior.
- **Manual evidence:** the generated matrix currently shows `las-import` 0/60, `generic-curve-store` 0/18 and `data-conventions` 4/80, but legacy checked Automated entries contaminate the checked count; this increment claims no Jauhar-confirmed manual or field exercise.
- **Git evidence:** the algorithm is integrated at the accepted anchor; the current `codex/g2-program-plan` increment adds the missing owned characterization without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied characterization contract); `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the owned characterization; it must not be relabelled as correctness or field evidence.
- **Next action:** retain exact T12/T13 with the source-order reversal and repeat control; Jauhar's rendered choice and representative-delivery assessment remain Visual/Manual/Field evidence.

## SB-DIO-009 - The alias choice MUST be reported.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T14`.
- **Atomic obligations:** report chosen and passed-over mnemonics plus each finite-coverage count whenever aliases compete.
- **Current source:** `AliasDecision` and `AliasCandidateCoverage` travel from the coverage-aware LAS parser through the public serializable `ImportResult`, the whitelisted Tauri command and the typed TypeScript IPC contract.
- **Qualifying acceptance tests:** `the_alias_result_names_the_chosen_and_passed_over_columns_with_both_coverage_counts` is `CORRECTNESS`; its fixture and expectation come from the chapter's T12/T14 contract, and it drives the production import function before asserting one decision, chosen/passed-over identities, both coverage counts and both chosen flags.
- **Supporting tests:** SB-DIO-030 tests the rename/table-entry arm of the same result model.
- **Manual evidence:** the generated matrix currently shows `las-import` 0/60 and `generic-curve-store` 0/18; this increment claims no Jauhar-confirmed visual, manual or field exercise.
- **Git evidence:** reachable `ef5f222` contains the closing report surface; the current `codex/g2-program-plan` increment reverified it against current source and exact T14 without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied audit contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T14 and keep the full candidate list in any future alias-based reader rather than emitting only the winner; Jauhar must still confirm that the desktop result actually renders the decision readably.

## SB-DIO-010 - Prefer a structural index declaration; fall back to names; record which mechanism fired.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DIO-T15`, `SB-DIO-T16`.
- **Atomic obligations:** prefer a structural declaration where present; honour a format-owned positional guarantee before names; otherwise resolve by alias or explicit designation; report the mechanism.
- **Current source:** `parsers.rs::resolve_index_column` implements the ordered mechanisms and returns `IndexResolution`; LAS and core import results carry it through IPC. No production reader or Tauri command imports Geolog flat ASCII or consumes a `.flat_ascii_format` `CLASSES` declaration.
- **Qualifying acceptance tests:** none; test class is `MISSING`. The existing `a_structural_index_wins_and_every_resolution_records_the_mechanism_that_fired` test passed 1/0/0, but only its T16 arm drives the production LAS importer. The T15 arm supplies in-memory headers/classes directly to the resolver, so a tree with no structural file reader still passes.
- **Supporting tests:** SB-DIO-013 exercises the user-designation outcome.
- **Manual evidence:** `las-import` 0/63, `delimited-intake` 3/27 and `data-conventions` 0/80; this re-verification claims no visual, manual or field exercise.
- **Git evidence:** reachable `4a6bc9f` contains the resolver/result change; current Gate 2 re-verification found that its T15 test arm stops at the helper boundary.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED` on DEC-029. Exact T15 requires an actual Geolog flat-ASCII import, while DEC-003 and G2-T04 define a LAS-2/delimited pilot surface. The cited local specs establish `CLASSES = REFERENCE | LOG`, but engineering will not fabricate a test-only reader or silently widen the approved format surface.
- **Next action:** after DEC-029, either implement a source-faithful Geolog flat-ASCII import that returns `IndexResolution::StructuralDeclaration` for a non-first reference column and pins its non-structural fallback, or reconcile the acceptance boundary explicitly; retain the passing production LAS T16 arm unchanged.

## SB-DIO-011 - Index aliases MUST be namespace-aware and MUST have one definition per path.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned test `SB-DIO-T17`.
- **Atomic obligations:** define one sourced alias list per path; keep vendor namespaces separate; retain TVD only in its own reference namespace.
- **Current source:** LAS, core and tops-MD lists are separate and source-commented, and `TVD` exists only in `TOPS_TVD_ALIASES`. A fourth index-bearing path, `DEV_MD_ALIASES = [MD, DEPTH, DEPT, MEASURED_DEPTH]`, has no source comment and no value/source row in chapter §5.3.
- **Qualifying acceptance tests:** none; test class is `MISSING`. The former test named “every” enumerated three known source comments, so adding or retaining an undocumented fourth list still passed.
- **Supporting tests:** `the_three_documented_index_alias_lists_cite_their_sources_and_tvd_is_not_in_an_md_namespace` passed 1/0/0 and pins the three cited chapter lists plus the negative TVD membership controls. Unit-qualified depth-header tests protect the MD lists without restoring positional guessing.
- **Manual evidence:** `data-conventions` 4/84, `las-import` 0/63 and `core-point-import` 0/52; this re-verification claims no visual, manual or field exercise.
- **Git evidence:** reachable `8262c6b` contains the namespace split; `git blame` and `git log -S` trace the uncited deviation list to baseline commit `a659096` but provide no external or chapter authority for its values.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED-SOURCE`. The deviation-survey MD alias list lacks the single documented source SB-DIO-011 requires. Existing code and generic industry familiarity are not sources, and removing or narrowing accepted headers without authority would change import behavior by guess.
- **Next action:** supply a named source for every `DEV_MD_ALIASES` value, cite it beside the declaration, then rewrite exact T17 to discover every index-alias declaration mechanically; retain the passing TVD namespace controls and leave SB-DIO-014's reference semantics separate.

## SB-DIO-012 - A non-monotonic index MUST be detected and reported, never silently accepted.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DIO-T18`.
- **Atomic obligations:** locate the first non-increasing row; block before commit until a user decision; report acceptance without sorting or rewriting.
- **Current source:** `ingest.rs` checks the parsed index before writes, returns the row and requires `NonMonotonicDecision`; accepted data remains in delivered order.
- **Qualifying acceptance tests:** `a_non_increasing_index_is_blocked_at_the_reported_row_until_the_user_accepts_it` passed 1/0/0 and is `CORRECTNESS`; its 400 finite, non-duplicated rows pin the exact decreasing row, zero-write refusal, explicit `AcceptAsDelivered` control and retained warning from T18.
- **Supporting tests:** duplicate-depth policy tests distinguish repeated from decreasing indexes.
- **Manual evidence:** `las-import` 0/63 and `data-conventions` 4/89; this re-verification claims no visual, manual or field exercise.
- **Git evidence:** reachable `cc7c8f5` contains the pre-commit decision gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied structural guard); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T18 and preserve the separate non-increasing and duplicate decisions when adding readers; Jauhar verifies the rendered refusal and accepted-warning surfaces without upgrading automated evidence to field truth.

## SB-DIO-013 - When neither structure nor name resolves an index, the user MUST designate it.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DIO-T19`.
- **Atomic obligations:** make designation mandatory for formats without a positional guarantee; commit nothing before it; record the chosen column and mechanism.
- **Current source:** `resolve_index_column` refuses when structure/name fail and accepts only an explicit designated column; core import propagates the resolution.
- **Qualifying acceptance tests:** exact SB-DIO-T19 `a_delimited_table_without_an_index_name_commits_nothing_until_the_user_designates_one` passed 1/0/0 and is `CORRECTNESS`; the production core-import function commits zero rows for an unresolved `SAMPLE,CPOR` table, then records column zero, `SAMPLE` and `UserDesignation` only after the caller supplies that designation.
- **Supporting tests:** `unit_qualified_and_bare_depth_aliases_resolve_while_an_unrelated_column_is_not_guessed` passed 1/0/0; it protects `Depth (m)`, `DEPTH (FT)` and bare `DEPTH` while proving an unrelated `MEASURE` header is not guessed.
- **Manual evidence:** `core-point-import` 0/55, `delimited-intake` 3/27 and `data-conventions` 4/92; this re-verification claims no visual, manual or field exercise.
- **Git evidence:** reachable `00c22c5`, with follow-up `f02571f`, contains the designation contract and qualified-header regression.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied index guard); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T19 and its qualified-header boundary test for every non-positional delimited reader; Jauhar verifies the rendered designation control against representative deliveries without upgrading automated evidence to field truth.

## SB-DIO-014 - TVD MUST NOT be read as an MD index.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T20`, `SB-DIO-T21`.
- **Atomic obligations:** retain the TVD alias; mark TVD-referenced data as TVD; refuse MD joins/plots/comparisons until a deviation survey exists.
- **Current source:** `TOPS_TVD_ALIASES` remains separate and accepted; `TopsRecord`, `tops.depth_datum`, import and typed IPC retain the source reference. The shared `list_tops` MD-consumer boundary refuses a TVD source without an active survey, maps through the survey only when the TVD has one unique in-range MD solution, and exposes both raw source depth/reference and resolved MD. Log, correlation, composite, zone and autocorrelation consumers all route through that boundary; frontend catches name the refusal instead of presenting a valid-looking empty layer.
- **Qualifying acceptance tests:** exact SB-DIO-T20 `a_tvd_only_tops_table_commits_the_alias_and_records_the_tvd_reference` passed 1/0/0 and is `CORRECTNESS`: raw `900.0` and `TVD` commit together, the alias is retained, and MD-only overwrite plus delete/recreate paths change nothing. Exact SB-DIO-T21 `a_tvd_top_refuses_md_zones_without_a_deviation_survey_and_uses_the_surveyed_md_with_one` passed 1/0/0 and is `CORRECTNESS`: no-survey conversion names TVD, MD and the missing survey with zero zone writes; the literal `900 TVD -> 1000 MD` survey mapping yields an MD zone at `1000`, not `900`.
- **Supporting tests:** the SB-DIO-011 namespace control keeps TVD outside every cited MD list; existing MD tops import, parser and contiguous-zone regressions remain green.
- **Manual evidence:** `correlation-tops` 0/42 and `data-conventions` 4/95; this re-verification claims no visual, manual or field exercise.
- **Git evidence:** this increment extends reachable `8262c6b`'s alias split through production storage, guarded reads and typed IPC; commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied depth-reference guard); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for newly imported MD/TVD tops. Legacy pre-custody tops remain NULL and are not silently migrated; the wider legacy depth-datum declaration gap remains owned by blocked SB-DBM-031.
- **Next action:** retain exact T20/T21 and the single guarded MD-consumer boundary; Jauhar verifies the visible `MD <- TVD` provenance and no-survey refusal with representative tops without promoting automated evidence to field truth.

## SB-DIO-015 - An index with no declared unit anywhere MUST refuse.

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DIO-T22`, `SB-DIO-T23`, `SB-DIO-T24`.
- **Atomic obligations:** refuse when both project and file units are absent; when only the file unit is absent, require a per-import declaration rather than inheriting the project unit; name both possible sources; commit nothing before resolution.
- **Current source:** `units.rs::resolve_index_unit` returns errors for both absent-file cases, and LAS/core import accepts an explicit file-unit confirmation before applying the normal adopted/matched/converted action.
- **Qualifying acceptance tests:** exact SB-DIO-T22 `an_index_with_no_file_or_project_unit_refuses_names_both_sources_and_commits_nothing` passed 1/0/0 and is `CORRECTNESS`: both possible sources are named and zero wells commit. Exact SB-DIO-T23 `a_project_unit_never_becomes_an_undeclared_files_unit_without_per_import_confirmation` passed 1/0/0 and is `CORRECTNESS`: the metre project lends no unit and writes zero wells, while explicit `FT` confirmation stores `1000 ft` as `304.8 m` by the cited factor.
- **Supporting tests:** exact SB-DIO-T24 `characterizes_a_declared_feet_index_into_a_metre_project_as_converted_with_a_report` passed 1/0/0 and is explicitly `CHARACTERIZATION` for the current report text; its `304.8 m` numeric assertion remains independently sourced. Unit spelling and DLIS reconciliation tests exercise the shared resolver.
- **Manual evidence:** `data-conventions` 4/98 and `las-import` 0/66; this increment claims no operator or representative-delivery exercise.
- **Git evidence:** reachable `a98c154` contains the closing refusal; this increment separates T22/T23 correctness from T24 characterization and adds pre-write assertions, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied P0 contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain the separate exact T22/T23/T24 proofs and require every new index-bearing reader to call the shared resolver before writes; Jauhar judges the rendered refusal/confirmation messages under Gate 4.

## SB-DIO-016 - The DLIS index unit MUST be read and reconciled.

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DIO-T25`, `SB-DIO-T26`.
- **Atomic obligations:** emit the index channel's `UNITS` from the DLIS sidecar; reconcile it through the shared unit resolver; convert with the cited foot factor or refuse an undeclared index under SB-DIO-015.
- **Current source:** `DLIS_RUNNER` emits `index_unit` for each scalar channel, `resolve_dlis_index_actions` requires consistent per-file units and calls `resolve_index_unit`, and `apply_index_action` converts the depth buffers before commit.
- **Qualifying acceptance tests:** `the_dlis_index_unit_is_read_reconciled_and_an_undeclared_one_is_refused` is `CORRECTNESS`; its 0.3048 factor is the exact NIST SP 811 conversion cited in chapter section 5.1, and it pins convert, refuse and explicit-confirm paths.
- **Supporting tests:** SB-DIO-015 pins the common resolver independently of the sidecar.
- **Manual evidence:** `dlis-import` 0/11 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** reachable `189cceb` contains the closing index-unit sidecar and import path.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied P0 contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** runtime field exercise still depends on a qualified `dlisio` environment, but the automated contract is closed.
- **Next action:** field-exercise metric and feet DLIS files under Gate 4 without weakening the no-unit refusal.

## SB-DIO-017 - The LAS writer MUST write the depth unit it actually used.

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T27`, `SB-DIO-T28`.
- **Atomic obligations:** obtain the stored/project depth unit from the unit model; write it on STRT, STOP, STEP and the index curve; prohibit a writer-owned unit literal; survive metre and feet round trips.
- **Current source:** `export.rs::write_las` resolves the project depth unit and writes its code to every depth declaration; the output validator checks the same declared unit before success.
- **Qualifying acceptance tests:** exact SB-DIO-T27 `a_feet_project_las_round_trip_preserves_depths_and_declares_ft_on_every_depth_header` passed 1/0/0 and is `CORRECTNESS`: every required header declares `FT`, none declares `M`, the fresh project adopts feet and `2000.0` returns unchanged. `a_feet_las_misdeclared_as_metres_fails_its_self_check_before_success` also passed 1/0/0 and refuses the syntactically valid unit lie by naming expected and declared units.
- **Supporting tests:** exact SB-DIO-T28 `characterizes_a_metre_project_las_round_trip_as_preserving_depths_and_declaring_m` passed 1/0/0 and is explicitly `CHARACTERIZATION` because the chapter labels the metre scenario char; it separately pins all four `M` declarations, absence of `FT`, and the current `2000.0` round trip.
- **Manual evidence:** `las-export` 0/5, `las-import` 0/69 and `data-conventions` 4/101; this increment claims no third-party reader or representative-delivery exercise.
- **Git evidence:** reachable `f536578` contains the closing writer change; this increment separates T27 correctness from T28 characterization, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied P0 contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain the separate exact T27/T28 proofs, keep all future depth-writing formats on the shared unit model, and reserve representative-file interoperability for Gate 4.

## SB-DIO-018 - Canonical units MUST have exactly one definition.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T29`, `SB-DIO-T30`.
- **Atomic obligations:** define each canonical family unit once in `curves.rs`; delete the export duplicate; make writers query the canonical table.
- **Current source:** the reviewed `registry/unit-registry.json` is the single source introduced by SB-INS-019; it generates the `curves.rs` `FAMILIES` table, while `export.rs` calls `curves::canonical_unit` and contains no writer-owned `standard_units` table.
- **Qualifying acceptance tests:** exact T29 `the_las_writer_has_no_unit_table_and_queries_the_canonical_family_registry` passed 1/0/0 and checks production source only, so its own assertion text cannot satisfy the positive arm. Exact T30 `every_exported_family_declares_the_canonical_registry_unit_with_exact_case` passed 1/0/0 and compares one file-boundary curve per registered family against the reviewed section 5.1 table with exact spelling and case. Both are `CORRECTNESS`.
- **Supporting tests:** SB-INS-019's generated-consumer drift gate and SB-DIO-017's LAS unit round trips exercise the registry lifecycle and actual file output without replacing T29/T30.
- **Manual evidence:** `data-conventions` 4/104 and `las-export` 0/5; this increment claims no other-reader interoperability or representative-delivery exercise.
- **Git evidence:** reachable `34652cd` contains the single-definition closure; this increment separates T29/T30 and removes the self-satisfying static-test path, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied unit contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** register any new family only in the reviewed source registry, regenerate `curves.rs`, and retain both the production-only duplicate-definition check and exact exported-unit comparison; Jauhar owns representative-reader verification.

## SB-DIO-019 - Changing the project depth unit MUST NOT silently rescale stored data.

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DIO-T31`.
- **Atomic obligations:** either perform an explicit counted migration or refuse a declaration change while committed data exists; never reinterpret or rescale silently.
- **Current source:** `units.rs::set_project_depth_unit_checked` permits no-op/safe empty-project declarations and refuses a changed unit once any well exists; the Tauri command uses this guarded entry point.
- **Qualifying acceptance tests:** exact T31 `changing_the_project_depth_unit_is_refused_while_committed_curves_exist_and_nothing_is_rescaled` passed 1/0/0 and is `CORRECTNESS`. It pins the permitted refusal alternative from both sides: reasserting metres is a safe no-op; changing to feet with one committed well refuses, names the well count and reinterpretation risk, keeps the declaration in metres, and leaves the packed database depth bytes identical to the pre-attempt snapshot.
- **Supporting tests:** project-unit resolver tests cover the empty-project/adoption path; they do not replace T31's committed-data custody proof.
- **Manual evidence:** `project-lifecycle` 3/24 and `data-conventions` 4/104 are partial; this increment claims no operator or reopen/replot exercise.
- **Git evidence:** reachable `4776c9d` contains the guarded declaration path; this increment strengthens exact T31 with a database byte snapshot and same-unit positive control, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied mutation guard); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; an actual bulk migration remains outside this requirement because refusal is an explicitly permitted implementation.
- **Next action:** preserve the guarded entry point, retain the two-sided T31 byte proof, keep display-unit changes separate from stored-unit declarations, and leave any future bulk migration to its own explicit counted contract.

## SB-DIO-020 - Duplicate depths MUST be resolved by a declared policy, and the count reported.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DIO-T32`, `SB-DIO-T33`.
- **Atomic obligations:** detect duplicates before writes; require refuse/keep-first/keep-last/mean; apply the chosen policy in lockstep to every column; report affected-row count.
- **Current source:** `DuplicateDepthPolicy` and the shared row resolver handle standard LAS, generic curves and DLIS frames; ingest refuses without a decision and carries the count/note after resolution.
- **Qualifying acceptance tests:** exact T32 `keep_first_drops_three_repeated_depth_rows_reports_three_and_keeps_first_samples_in_lockstep` passed 1/0/0 and is `CORRECTNESS`: production LAS import reports the specified three affected rows, stores two depths, and retains the first GR and generic PEF sample in lockstep; explicit Refuse also writes no well. Exact T33 `duplicate_depths_commit_nothing_until_a_policy_is_declared` passed 1/0/0 and is separate `CORRECTNESS`: absent policy names the three rows and decision while committing zero wells.
- **Supporting tests:** T32 also exercises KeepLast and Mean through the shared resolver with independent standard and generic companion columns; the non-increasing-index test uses duplicate-free data so these decisions cannot accidentally discharge each other.
- **Manual evidence:** `las-import` 0/72, `dlis-import` 0/14 and `data-conventions` 4/104; this increment claims no operator or representative run-splice exercise.
- **Git evidence:** reachable `82a0448`, with follow-up `86b4b5c`, contains the policy gate and focused fixture correction; this increment separates T32/T33 and adds real standard-plus-generic lockstep custody, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied structural guard); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain separate exact T32/T33, require every new indexed reader to use the shared resolver before transaction commit, and preserve the affected-row count at its public result boundary.

## SB-DIO-021 - Resampling on read MUST be explicit, named, and off by default.

- **Chapter evidence:** P2; chapter status `PRESENT-OK`; owned test `SB-DIO-T34` (characterization).
- **Atomic obligations:** preserve incoming sample intervals by default; allow a change only through an explicit operation named decimate/interpolate/average/nearest.
- **Current source:** the source-discovered file-reader registry is exhaustively classified into sampled and non-sampled readers. Every sampled parser retains the supplied depth sequence; shipping LAS, core-table and WIDE-array paths store it unchanged. Explicit Reframe remains the separate operation that creates an `OWN` frame and records a named method.
- **Qualifying acceptance tests:** exact T34 `characterizes_every_registered_sampled_reader_and_shipping_store_as_preserving_native_depths_until_reframe_is_explicit` passed 1/0/0 and is `CHARACTERIZATION`, as the chapter requires. Its 0.1 m acquisition fixture omits the 1000.2 m station, so both an unclassified new reader and a silent regularizer fail; the three shipping stores retain `1000.0, 1000.1, 1000.3` and create no implicit `OWN` set.
- **Supporting tests:** explicit-set viewer/composite tests prove native generic samples are later read on their owning grids, and SB-DIO-022 independently proves writer defaults.
- **Manual evidence:** `las-import` 0/72, `dlis-import` 0/14, `data-conventions` 4/104 and `reframe` 0/34; this increment claims no operator or representative-delivery exercise.
- **Git evidence:** reachable `2983373` preserves imported LAS sets on native grids; this increment adds the universal reader classification and stored-depth characterization, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied native-grid lock); `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** none; the chapter deliberately classifies T34 as characterization rather than correctness.
- **Next action:** retain exact T34, require every new file reader to enter the sampled/non-sampled registry, and keep any sample-changing operation explicit under Reframe; Jauhar verifies representative files and operation wording.

## SB-DIO-022 - Re-grid on write MUST be named correctly and default OFF.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T35`.
- **Atomic obligations:** default writer-side regridding off; emit stored irregular samples exactly; if later enabled, name it as output resampling and record the changed-sample provenance.
- **Current source:** the registered writer boundary has no regrid argument, and each writer receives the fetched stored rows directly. One LAS 2.0 writer currently ships; no writer-side resample option exists, so the permitted default-off implementation is active.
- **Qualifying acceptance tests:** exact T35 `an_export_at_defaults_writes_the_irregular_stored_samples_without_regridding` passed 1/0/0 and is `CORRECTNESS`; every registered writer receives independently seeded irregular depths plus non-linear GR values and must expose its artifact to a format-specific inspection adapter. A new writer without that adapter fails by name rather than inheriting LAS evidence.
- **Supporting tests:** mandatory self-read validation checks row and curve counts after every registered write, while T35 independently compares artifact values with the database fixture.
- **Manual evidence:** `las-export` 0/5, `data-conventions` 4/104, `reframe` 0/34 and `security-integrity` 3/92; this increment claims no independent-reader or representative-file exercise.
- **Git evidence:** reachable `3291921` contains the default-off lock; this increment strengthens T35 across the writer registry, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied output-fidelity contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none while no writer regrid option ships; its “when on” obligations remain conditional and cannot be claimed as a shipped capability.
- **Next action:** retain exact T35 and its exhaustive registry adapter; if output regridding is ever requested, add explicit UI provenance and a non-default test before implementing it.

## SB-DIO-023 - Numeric columns MUST be validated against physical bounds, not against their labels.

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DIO-T36`, `SB-DIO-T37`, `SB-DIO-T38`.
- **Atomic obligations:** bind a known family, evaluate every finite value against that family's cited plausible bounds, and block for confirmation even when the mnemonic matches perfectly.
- **Current source:** no import-time physical-family bound registry or blocking question exists. Numeric range checks elsewhere are method-specific and cannot be promoted to this contract.
- **Qualifying acceptance tests:** none. T36-T38 are deliberately not implemented or executed because their expected bounds are absent; test class is `MISSING`.
- **Supporting tests:** alias, unit and structural checks cannot catch a correctly labelled but shifted numeric column.
- **Manual evidence:** `data-conventions` 0/45, `las-import` 0/57, `dlis-import` 0/11 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** no implementation commit exists; commit state is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** chapter section 5.6 deliberately ships physical-family ranges absent and section 7.1 O-4 records the block. Only a cited SB-ENV range table can unblock it; no plausible numbers may be supplied here.
- **Next action:** obtain and adjudicate cited family bounds in SB-ENV, then implement a pre-commit report and T36-T38 from those exact sources.

## SB-DIO-024 - Unit conversion MUST NOT be applied silently by default.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DIO-T39`.
- **Atomic obligations:** either disable automatic conversion or report every converted curve with source unit, destination unit and factor.
- **Current source:** conversion remains automatic, but `UnitConversion` records the curve, from/to units, factor and offset; LAS import returns the records and user-visible notes only after the transaction succeeds.
- **Qualifying acceptance tests:** exact T39 `every_converted_curve_reports_its_from_unit_to_unit_and_factor_and_uses_that_transform` passed 1/0/0 and is `CORRECTNESS`; independent DTCO and DTSM channels each produce their own public record and visible note. Stored `30.48` and `45.72` samples are independently derived from source values times the NIST exact-foot factor cited in section 5.1, so a report-only no-op cannot pass.
- **Supporting tests:** affine and conversion-table tests protect offset and family coverage; the two-channel fixture prevents an implementation that reports only the first automatic conversion.
- **Manual evidence:** `data-conventions` 4/104, `las-import` 0/72, `dlis-import` 0/14 and `security-integrity` 3/92; this increment claims no representative LAS/DLIS or operator exercise.
- **Git evidence:** reachable `66a5c0b` contains the conversion audit record; this increment strengthens T39 from one curve to every converted curve in the import, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied conversion-honesty contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain exact T39, keep conversion records transaction-coupled, and add every new reader to the same public result surface; Jauhar verifies representative LAS/DLIS wording and values.

## SB-DIO-025 - Conversion coverage MUST be declared, and an unconvertible unit MUST be reported rather than passed through.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DIO-T40`, `SB-DIO-T41`.
- **Atomic obligations:** expose the exact convertible-family set; retain an unknown declared unit verbatim; flag it as unconverted rather than relabelling it canonical.
- **Current source:** `curves::convertible_unit_families` feeds a registered Tauri command and typed frontend IPC wrapper; `prepare_generic_curves` records `UnconvertedUnit` while preserving the source unit and values.
- **Qualifying acceptance tests:** `an_unknown_declared_unit_is_stored_verbatim_and_flagged_unconverted` and `the_unit_system_reports_the_exact_families_it_can_convert` are `CORRECTNESS`; family membership comes from the chapter-cited code-resident transform table, T41 calls the shipping backend query, and it pins both command registration and the typed frontend invoke route.
- **Supporting tests:** SB-DIO-024 exercises a supported transform end to end.
- **Manual evidence:** `data-conventions` 4/107, `generic-curve-store` 0/21, `las-import` 0/75 and `security-integrity` 3/95; this increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `a19d0a2` contains the declared-coverage and unconverted report; this Gate 2 increment closes the previously unpinned shipping query route, with commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied unit boundary); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** require a sourced transform before adding a family to the queryable set; otherwise retain and flag the source unit.

## SB-DIO-026 - Unit conversion MUST support affine transforms.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T42`.
- **Atomic obligations:** represent conversion as factor plus offset and apply the offset before/with the factor as specified; never treat an affine unit as multiplicative.
- **Current source:** generated `UnitRule` rows carry factor, source-space offset and derivation; `convert_to_canonical` applies `(value + offset) * factor`, and the public `UnitConversion` audit carries both numeric fields.
- **Qualifying acceptance tests:** `a_fahrenheit_temperature_applies_its_affine_offset_before_its_factor` is `CORRECTNESS`; chapter §5.1 supplies the 32-degree offset and T42 supplies `200 °F -> 93.33 °C`, while the fixed point `32 °F -> 0 °C` and explicit rejection of `111.11 °C` independently distinguish affine behavior from multiplication alone.
- **Supporting tests:** the conversion derivation registry tests every transform's arithmetic.
- **Manual evidence:** `data-conventions` 4/110, `generic-curve-store` 0/24, `las-import` 0/78 and `security-integrity` 3/98; this RETAIN increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `a3c8257` contains the affine representation and path; exact T42 remains green on the current Gate 2 head, with the RETAIN commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied conversion contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** prohibit factor-only additions for any unit whose cited transform has a non-zero offset.

## SB-DIO-027 - A vendor alias that is wrong or ambiguous MUST NOT be inherited.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T43`.
- **Atomic obligations:** review aliases against physical quantity; reject a wrong/ambiguous vendor entry; record that designation is required while retaining source data.
- **Current source:** the unit table explicitly rejects `PPG` as density, leaves the curve familyless, records the pressure-gradient ambiguity and retains the delivered samples/unit for a later decision.
- **Qualifying acceptance tests:** `a_ppg_column_is_not_bound_to_density_and_is_flagged_for_designation` is `CORRECTNESS`; chapter §5.1 marks the vendor density alias NON-ADOPTABLE. T43 proves the public rejection record and warning, standard RHOB `NaN`, familyless generic custody, retained `PPG` unit and unchanged `9.5` source value.
- **Supporting tests:** unit-ambiguity and unconverted-unit tests protect neighbouring cases.
- **Manual evidence:** `data-conventions` 4/113, `generic-curve-store` 0/27, `las-import` 0/81 and `security-integrity` 3/101; this RETAIN increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `7bd9966` contains the recorded rejection; exact T43 remains green on the current Gate 2 head, with the RETAIN commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied alias-safety contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the cited instance; future aliases still require per-entry sources.
- **Next action:** keep a review/source field on every added alias and ship ambiguous entries familyless until designated.

## SB-DIO-028 - A conversion factor MUST be correct and MUST show its derivation.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T44`.
- **Atomic obligations:** carry an independent derivation with every factor/offset and verify the table arithmetically; never treat a vendor file as the authority for arithmetic.
- **Current source:** every generated `UnitRule` includes factor, affine offset and a derivation string; `curves.rs` keeps numeric transforms exclusively in that source-bearing registry.
- **Qualifying acceptance tests:** `every_conversion_factor_carries_an_independent_arithmetic_derivation` is `CORRECTNESS`; its independent ten-row table re-derives every expected factor and offset from the exact unit identities named by chapter §5.1, checks the bound families and automatic status, and requires the corresponding arithmetic terms in each runtime derivation. A wrong factor accompanied by a matching wrong sentence cannot pass.
- **Supporting tests:** concrete sonic, Fahrenheit and PPG tests exercise high-risk rows end to end.
- **Manual evidence:** `data-conventions` 4/116 and `verification-stewardship` 6/77; this proof-strengthening increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `bbd43bd` contains the derivation-bearing registry; exact strengthened T44 is green on the current Gate 2 head, with the proof commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied source discipline); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for current transforms.
- **Next action:** keep every future transform in the independent T44 enumeration so a missing row, wrong arithmetic or source-less derivation fails release.

## SB-DIO-029 - An unadjudicable unit ambiguity MUST ship with no default.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T45`.
- **Atomic obligations:** ship no default for `MS/FT`; require a per-file sonic-versus-conductivity answer; record it; never populate the wrong standard family.
- **Current source:** `LasImportOptions.ms_per_ft_meanings` keys explicit decisions by exact source path; import refuses before commit without that file's answer, and `UnitDesignation` records whether the curve becomes sonic or remains familyless conductivity under its source unit.
- **Qualifying acceptance tests:** `an_ms_per_ft_curve_waits_for_a_per_file_quantity_answer_and_records_either_answer` is `CORRECTNESS`; both meanings and the absence of a default come directly from chapter sections 4.5 and 5.1. T45 also imports two files in one batch with opposite explicit decisions, proving a cached or batch-wide answer cannot satisfy the path-scoped contract.
- **Supporting tests:** unconverted-unit tests prove source retention outside known families.
- **Manual evidence:** `data-conventions` 4/119, `las-import` 0/84 and `generic-curve-store` 0/30; this proof-strengthening increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `f20b87e` contains the no-default designation flow; exact strengthened T45 is green on the current Gate 2 head, with the proof commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied ambiguity refusal); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** retain the exact-path decision map and two-file T45 proof; never introduce a batch-wide or project-wide default for this symbol.

## SB-DIO-030 - An alias rename MUST be reported.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T46`.
- **Atomic obligations:** preserve source and applied mnemonics and identify the exact alias-table entry that fired.
- **Current source:** `AliasDecision` records target, chosen source and `table_entry`; ingest results and warning text carry the rename without changing generic-store source identity.
- **Qualifying acceptance tests:** `an_alias_rename_keeps_both_names_and_records_the_table_entry_that_fired` is `CORRECTNESS`; the chapter's SGR-to-GR case is checked through the public decision and visible note, standard GR sample application, generic SGR identity retention and exact firing-table row.
- **Supporting tests:** SB-DIO-009 covers competed aliases and coverage counts.
- **Manual evidence:** `las-import` 0/87 and `generic-curve-store` 0/33; this RETAIN increment adds no checked operator or representative-delivery evidence.
- **Git evidence:** reachable `0f545cf` contains the rename audit record; exact T46 remains green on the current Gate 2 head, with the RETAIN commit pending at this evidence write.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied audit contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** require every future rename mechanism to preserve source identity while emitting the same source-target-table-entry record on its public result.

## SB-DIO-031 - A different curve's data MUST NOT be supplied under a requested name.

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-DIO-T47`.
- **Atomic obligations:** return unavailable for an absent exact request; never return another curve's samples keyed as the request under any configuration.
- **Current source:** the explicit Reframe substitution path is safe, but `equations.rs::resolve_generic_curve_decision` and `curve_sampling` match `upper(mnemonic) = request OR upper(family) = request`; numeric callers then store the returned values under the requested key. Workflow tests deliberately rely on `HDRA -> DRHO` and `HCAL -> CALI` family fallback. `plotting.rs` also resolves a typed-family match, although it calls the input semantic and returns the concrete mnemonic plus resolution reason. The universal MUST NOT therefore remains violated because exact and semantic intent share one string API.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** `an_accepted_named_substitute_is_recorded_on_the_resulting_curve_as_provenance` proves one explicit path and explicitly keeps the substitute's own name; it cannot prove the general resolver.
- **Manual evidence:** `generic-curve-store` 0/36, `workflow` 0/26, `log-view` 5/40 and `security-integrity` 3/104; the blocker review adds no checked operator or representative-delivery evidence.
- **Git evidence:** the family-fallback behavior is integrated at the accepted anchor; current source was reverified on the Gate 2 head, and no universal closing commit or T47 mapping exists.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED` on DEC-030. Jauhar must approve a non-overlapping exact-mnemonic versus semantic-family request contract; no silent fallback can remain under an exact-request key, and engineering will not break intentional family workflows by guessing every caller's intent.
- **Next action:** after DEC-030, implement the typed request split, make exact-mnemonic absence explicit, return concrete identity for semantic-family resolution, and implement T47 across every resolver.

## SB-DIO-032 - A substitution offered to the user MUST be explicit and recorded.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T48`.
- **Atomic obligations:** allow substitution only for an unavailable explicitly requested curve; require named substitute and explicit acceptance; write it under its own name; record the mapping in ancestry.
- **Current source:** `reframe.rs::resolve_substitutions` enforces every precondition and stores the accepted mapping in the resulting log-set parameters while the output curve keeps the substitute mnemonic.
- **Qualifying acceptance tests:** `an_accepted_named_substitute_is_recorded_on_the_resulting_curve_as_provenance` is `CORRECTNESS`; it pins refusal/no-write and accepted/provenance controls from D-15 and T48.
- **Supporting tests:** SB-DIO-033 proves the requested selection is itself explicit and saved.
- **Manual evidence:** `reframe` 1/41 from prior unrelated review and `processing-history` 0/7; this T48 scenario remains unchecked.
- **Git evidence:** reachable `f417ea7` contains the explicit substitution flow.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated Gate 2 closure; immutable DEC-018 includes explicit Reframe and T48 is green at 1 passed / 0 failed / 0 ignored.
- **Next action:** preserve the two-sided T48 proof and field-exercise the named offer, acceptance, output identity and ancestry in Gate 4.

## SB-DIO-033 - Curve-selection state MUST be explicit and inspectable.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T49`.
- **Atomic obligations:** save a named ordered selection with explicit mode; reload and inspect it; reject hidden blank-means-all or type-implied defaults.
- **Current source:** `CurveSelection` requires `name`, `mode` and ordered exact members; documents persist it, the UI requires a saved selection, and blank selection names refuse.
- **Qualifying acceptance tests:** `a_saved_curve_selection_reloads_as_a_named_object_listing_its_members` is `CORRECTNESS`; it pins persisted order/mode and the missing-mode negative side.
- **Supporting tests:** Reframe request tests consume the saved object rather than an implicit list.
- **Manual evidence:** `reframe` 1/44 and `project-lifecycle` 3/24 from prior unrelated reviews; this T49 scenario remains unchecked.
- **Git evidence:** reachable `216af9d` contains the saved selection object.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated Gate 2 closure; immutable DEC-018 includes the exact Reframe derivation-set contract and T49 is green at 1 passed / 0 failed / 0 ignored.
- **Next action:** preserve T49 and field-exercise save, project reload, inspection and consumption of the named selection in Gate 4.

## SB-DIO-034 - Curves MUST NOT be auto-selected by curve type on read.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T50`.
- **Atomic obligations:** no read API may choose a concrete curve from a type/family classification without stating that concrete choice.
- **Current source:** `equations.rs::fetch_generic_curve_aligned` silently widens an exact-looking request to `family = request` and returns only values, not the chosen curve identity. `plotting.rs` performs a similar typed-family resolution but records its reason and mnemonic. The unreported workflow/read path violates the universal contract.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** current workflow tests characterize family fallback as useful behavior but do not expose the chosen identity, so they defend neither the specified reporting contract nor an exact-name refusal.
- **Manual evidence:** `generic-curve-store` 0/39, `workflow` 0/29, `log-view` 5/43 and `security-integrity` 3/107; this T50 scenario is unavailable and unchecked.
- **Git evidence:** family-based read resolution is integrated at the accepted anchor; no T50 closure exists.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED` on DEC-030, shared with SB-DIO-031. Jauhar must approve non-overlapping exact-mnemonic and semantic-family requests before callers can be classified without silently breaking or preserving family fallback.
- **Next action:** after DEC-030, make family resolution an explicit semantic request returning concrete identity/reason, keep exact requests exact, and add T50 across every read resolver.

## SB-DIO-035 - An import MUST NOT extend an existing object's declared interval.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T51`.
- **Atomic obligations:** detect an incoming DLIS interval outside the held well/set interval; stop before writes; require an explicit decision; leave the existing interval unchanged on refusal.
- **Current source:** `dlis.rs::interval_preflight` computes stored/incoming extents and returns a confirmation-required result before any prepared curve is written; the explicit accept path records the decision.
- **Qualifying acceptance tests:** `a_dlis_outside_an_existing_wells_declared_range_requires_confirmation_before_any_write` is `CORRECTNESS`; it pins refusal/no-write and explicit-confirm controls from D-34/T51.
- **Supporting tests:** DLIS index-unit and multi-well tests protect upstream interval identity.
- **Manual evidence:** `dlis-import` 0/11, `delivery-sets` 0/33 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** reachable `0b0a1d5` contains the interval preflight.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` has not established whether DLIS is in the first pilot workflow; the automated contract is closed.
- **Next action:** field-exercise refusal and confirmation if DLIS enters the named pilot corpus.

## SB-DIO-036 - The duplicate-name policy MUST NOT default to merge.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T52`.
- **Atomic obligations:** stop on every existing mnemonic/frame conflict; offer only explicit keep-separate or skip decisions; record each choice; provide no merge default/path.
- **Current source:** `duplicate_preflight` requires one decision per conflict, records it, and exposes no merge enum variant; keep-separate writes a new set and skip is named.
- **Qualifying acceptance tests:** `an_incoming_dlis_mnemonic_requires_a_recorded_per_curve_choice_and_never_defaults_to_merge` is `CORRECTNESS`; it pins the undecided refusal, both permitted choices and the absence of a merge action.
- **Supporting tests:** import-set tests confirm separate set identity is retained.
- **Manual evidence:** `dlis-import` 0/11, `delivery-sets` 0/33 and `generic-curve-store` 0/18 - unexercised.
- **Git evidence:** reachable `eb6f219` contains the duplicate-name preflight.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot relevance depends on `DEC-003`; no engineering blocker remains.
- **Next action:** keep merge absent from the enum/API and field-exercise both permitted choices if DLIS is selected.

## SB-DIO-037 - Channels that could not be loaded MUST be named, and a partial load MUST NOT be reported as success.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned test `SB-DIO-T53`.
- **Atomic obligations:** name every unreadable/unsupported/encrypted channel and report partial when fewer payload channels load; distinguish a retained attribute warning from omitted data.
- **Current source:** the DLIS sidecar emits structured `DlisSkip` records with `omitted`; `import_status` compares declared and loaded channels; results expose complete/partial/failed plus named skips.
- **Qualifying acceptance tests:** `a_dlis_with_one_unreadable_and_one_readable_channel_is_partial_and_names_the_unreadable_channel` is `CORRECTNESS`; it pins both omitted-partial and retained-warning-complete sides.
- **Supporting tests:** SB-DIO-054 covers all-skipped failure and every skip-kind inventory.
- **Manual evidence:** `dlis-import` 0/11 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** reachable `19bc678` contains the partial-status closure.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot relevance depends on `DEC-003`; field behavior still needs a qualified `dlisio` environment.
- **Next action:** preserve complete/partial/failed semantics and exercise one real partial file if DLIS enters Gate 4.

## SB-DIO-038 - Multi-dimensional channels MUST be imported through the published RP66 container.

- **Chapter evidence:** P2; chapter status `ABSENT`; owned tests `SB-DIO-T54`, `SB-DIO-T55`.
- **Atomic obligations:** parse RP66 multidimensional channels with frame shape and per-axis labels/units; import an all-array delivery as non-empty; use no proprietary tile/model encoding; support the published-container round trip described by section 7.4 D-1.
- **Current source:** `DLIS_RUNNER` accepts only one-dimensional scalar columns and emits a skip for every other shape. No array shape/axis schema or bytemuck IPC/storage path is connected to DLIS arrays; an all-array file is now an honest error under SB-DIO-054, not the required import capability.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** SB-DIO-054 proves arrays are named rather than silently lost, but a correct refusal is not multidimensional support.
- **Manual evidence:** `array-logs` 0/16 and `dlis-import` 0/11 - unexercised.
- **Git evidence:** no implementation commit exists; scalar skip reporting is integrated but does not satisfy this requirement.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** section 7.4 A-1 says the normative API RP66 V1 multidimensional-channel sections are not held locally. The read half may be independently built from the public specification once acquired; the write half must not be inferred from `dlisio` behavior.
- **Next action:** Jauhar decides pilot need under `DEC-003`; if in scope, acquire API RP66 V1, design bytemuck array storage/IPC from it, then implement T54/T55 without proprietary encodings.

## SB-DIO-039 - The DLIS sentinel screen MUST be per-channel overridable and MUST count what it deleted.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T56`, `SB-DIO-T57`.
- **Atomic obligations:** retain default LAS-style sentinel screening for DLIS scalar channels; allow named per-channel disable; count screened samples and identify the rule per channel.
- **Current source:** DLIS import builds per-channel modes, honours explicit no-null exceptions, and returns `DlisSentinelScreen` records with channel, count and rule.
- **Qualifying acceptance tests:** `a_named_dlis_sentinel_exception_preserves_minus_999_25_while_the_default_screens_and_counts_it` is `CORRECTNESS`; the sentinel and override behavior come from D-5 and chapter section 5.2, with override and default controls.
- **Supporting tests:** common plural/no-null tests exercise the underlying null-mode representation.
- **Manual evidence:** `dlis-import` 0/11 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** reachable `9b1a37b` contains the per-channel screen record.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot relevance depends on `DEC-003`; no source gap remains.
- **Next action:** exercise the default and exception paths on controlled DLIS data if selected for Gate 4.

## SB-DIO-040 - Wrapped LAS MUST be read; the writer MUST emit unwrapped.

- **Chapter evidence:** P2; chapter status `PRESENT-OK`; owned test `SB-DIO-T58` (characterization).
- **Atomic obligations:** honour `WRAP.YES` by assembling complete logical rows regardless of physical line breaks; keep columns aligned; emit `WRAP.NO` and one row per depth on write.
- **Current source:** both LAS parse paths buffer tokens when wrapped and reject incomplete logical rows; `export.rs` always declares `WRAP.NO` and writes one complete line per depth.
- **Qualifying acceptance tests:** `characterizes_thirty_wrapped_las_curves_as_aligned_and_every_las_export_as_unwrapped` is `CHARACTERIZATION`, matching T58's chapter classification; it drives 30 uniquely valued curves through the real import/store path and then the registered writer.
- **Supporting tests:** SB-DIO-054's malformed-row test retains its three-column positive/refusal controls; T58 adds the missing full-width positional and writer evidence without reclassifying either as field proof.
- **Manual evidence:** `las-import` 0/93 and `las-export` 0/8 after regeneration; this exact wide-file scenario remains unchecked.
- **Git evidence:** wrapped reader and unwrapped writer were already integrated; the current `codex/g2-program-plan` increment adds the owned end-to-end T58 lock without production changes.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated Gate 2 closure; D-24 fixes the 30-curve characterization and unwrapped writer direction.
- **Next action:** retain exact T58; Jauhar manually inspects representative early/middle/late curves and independently opens the unwrapped export in Gate 4.

## SB-DIO-041 - A LAS 3.0 file MUST be recognised, and what is not read MUST be named.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T59`.
- **Atomic obligations:** recognise version 3.0; continue reading supported well/curve/ASCII content; list each encountered unsupported section in the import result.
- **Current source:** both LAS parsers capture `VERS`, classify 3.0, record unknown/associated section headers and return them through `ImportResult.warning`.
- **Qualifying acceptance tests:** `a_las_3_file_is_recognised_as_3_0_and_every_unread_section_is_named_in_the_result` is `CORRECTNESS`; the expected section names are seeded independently from the chapter T59 fixture.
- **Supporting tests:** the malformed corpus exercises section and row failures through both LAS readers.
- **Manual evidence:** `las-import` 0/57 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** reachable `7ee9e97` contains LAS 3 recognition and unread-section reporting.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` has not established LAS 3.0 as a pilot input; associated-section capability remains SB-DIO-042.
- **Next action:** preserve the named-degradation result and field-exercise it only if LAS 3.0 enters the pilot corpus.

## SB-DIO-042 - LAS 3.0 associated sections MUST be read.

- **Chapter evidence:** P3; chapter status `ABSENT`; owned test `SB-DIO-T60`.
- **Atomic obligations:** parse associated-section definitions, `|`-delimited sub-sections and multi-array data into the appropriate core/tops/array models.
- **Current source:** LAS 3.0 associated sections are recorded as unread names and their payloads are not parsed.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** SB-DIO-041 proves honest recognition/refusal, not associated-section support.
- **Manual evidence:** `las-import` 0/57, `core-point-import` 0/52 and `array-logs` 0/16 - unexercised.
- **Git evidence:** no implementation commit exists; commit state is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** section 7.4 A-2 records the CWLS LAS 3.0 specification as not held locally. Implementation may not be derived from another product's parser.
- **Next action:** keep the named unread-section behavior; acquire the CWLS LAS 3.0 specification before a separately authorized later increment.

## SB-DIO-043 - LAS 1.2 MUST be readable and MUST NOT be writable.

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DIO-T61`.
- **Atomic obligations:** accept a valid 1.2 delivery; never expose a 1.2 writer option.
- **Current source:** the LAS readers accept non-3.0 version strings using the common scalar parser, so a normal 1.2 structure is not rejected; `REGISTERED_WRITERS` offers only LAS 2.0. No explicit 1.2 branch documents or tests the supported subset.
- **Qualifying acceptance tests:** no T61 mapping exists; test class is `MISSING`.
- **Supporting tests:** the writer registry proves 1.2 is not offered, but no legacy fixture proves the read half.
- **Manual evidence:** `las-import` 0/57 and `las-export` 0/2 - unexercised.
- **Git evidence:** the generic reader and 2.0-only writer are integrated; no owned 1.2 closure exists.
- **Verdict:** `PRESENT-UNVERIFIED`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` must determine whether a legacy 1.2 delivery belongs in the representative corpus.
- **Next action:** add one source-conformant 1.2 fixture, pin successful read and absence from `export_formats`, then field-exercise if selected.

## SB-DIO-044 - Section-parse strictness MUST be declared and consistent.

- **Chapter evidence:** P2; chapter status `PARTIAL`; owned test `SB-DIO-T62`.
- **Atomic obligations:** declare one policy for malformed headers, unknown sections and out-of-order sections across LAS versions; report every tolerance/refusal outcome.
- **Current source:** `parsers.rs` now routes both complete-curve and all-curve LAS readers through one versioned `las_sections_v1` state machine. It reports unknown, malformed and recognized out-of-order headers in source order, requires a finite numeric `~V` declaration and a `~W` section before `~A`, and does not invent a supported-version range. `ingest.rs` returns the policy plus typed handling records and adds a user-visible warning; `ipc.ts` preserves that structure on the frontend boundary.
- **Qualifying acceptance tests:** `a_single_section_policy_reports_unknown_malformed_and_out_of_order_headers_in_las_2_and_3_and_refuses_data_before_version_or_well` is `CORRECTNESS`. It pins accepted and refused sides across both parser entry points, then drives the accepted LAS 2.0 and 3.0 fixtures through the real importer and inspects serialized structured outcomes.
- **Supporting tests:** the 15 LAS depth tests and 54 ingest tests remain green, including the existing LAS 3 unread-section behavior. Two existing LAS-writer self-check fixtures now declare the mandatory `~W` section so their unchanged malformed-row and wrong-unit assertions still reach their own subjects; SB-DIO-044's refusal was not weakened.
- **Manual evidence:** `las-import` 0/96 and `security-integrity` 3/107 after regeneration; the exact strictness scenario remains unchecked and the prior security checks are unrelated.
- **Git evidence:** the current `codex/g2-program-plan` increment adds the shared policy, typed reporting surface and owned T62 proof; full-gate evidence is recorded in the status receipt before commit.
- **Verdict:** as-built `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated Gate 2 closure. Deferred SB-DIO-042 associated-section parsing and SB-DIO-043 version-support contracts were not imported into this increment.
- **Next action:** retain exact T62; Jauhar visually inspects the policy/handling surface and field-exercises representative tolerated and refused deliveries in Gate 4.

## SB-DIO-045 - A multi-well container MUST produce multiple wells, never one merged well.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned tests `SB-DIO-T63`, `SB-DIO-T64`.
- **Atomic obligations:** derive one project well per source well in a multi-well container; never merge those sources; show the source-to-project mapping before commit.
- **Current source:** `src-tauri/src/dlis.rs` groups logical files by source well identity, refuses logical files that cannot be separated, returns the mapping in preview and commits each resolved group to a distinct well.
- **Qualifying acceptance tests:** `a_three_well_dlis_shows_its_logical_file_mapping_before_commit_and_creates_three_wells_without_merging` is `CORRECTNESS`; one fixture pins both the visible pre-commit mapping and the three separate committed wells.
- **Supporting tests:** the skipped-frame accounting tests prove per-frame degradation but do not replace the cross-logical-file identity assertion.
- **Manual evidence:** `dlis-import` 0/11 and `delivery-sets` 0/33 - unexercised.
- **Git evidence:** reachable `da40d16` contains the multi-logical-file grouping, preview and regression test.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` has not selected a multi-well DLIS for the pilot corpus; this does not diminish the automated contract.
- **Next action:** field-exercise a representative multi-well container if `DEC-003` selects one, preserving the pre-commit mapping evidence.

## SB-DIO-046 - A missing interpreter or library MUST produce a named, actionable, per-format refusal.

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned test `SB-DIO-T65`.
- **Atomic obligations:** disable only the affected format; name the missing interpreter or library and the remedy; refuse before a partial import or write.
- **Current source:** `src-tauri/src/dlis.rs` resolves `CAPABILITY_DLIS_IMPORT` through the shared Python capability manifest; `src-tauri/src/office.rs` does the same for workbook/document/deck export, and the resolver supplies capability-specific installation text.
- **Qualifying acceptance tests:** no T65 mapping simulates both missing Python and missing per-format packages for DLIS and workbook export; test class is `MISSING`.
- **Supporting tests:** capability-manifest and installation tests validate individual messages and the interpreter probe, but they do not execute both format boundaries under the T65 absence conditions.
- **Manual evidence:** `dlis-import` 0/11 and `office-deliverables` 0/39 are unexercised; the matrix has no dedicated installer-capability row.
- **Git evidence:** the per-capability resolver is integrated; no owned end-to-end refusal proof was found.
- **Verdict:** `PRESENT-UNVERIFIED`; `UNDECIDED`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no source parameter is missing; the gap is an owned boundary test, while release packaging remains an SB-INS decision.
- **Next action:** add T65 with an isolated capability environment and assert distinct DLIS and workbook refusals name their package and remedy before work starts.

## SB-DIO-047 - Storage precision MUST be declared and MUST NOT silently truncate.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned test `SB-DIO-T66`.
- **Atomic obligations:** declare internal sample precision; surface every import or export precision reduction in the operation result.
- **Current source:** `ingest.rs` parses core numeric text as f64 before the deliberate f32 store and returns the source/destination labels plus the count that actually changed. `export.rs` independently counts f32 samples altered by fixed-decimal-4 LAS representation, returns the report and writes `SANDIBUMI_PRECISION_V1` into the file. `coreImportDialog.ts` and `ribbon.ts` render the corresponding import/export result instead of hiding it in IPC.
- **Qualifying acceptance tests:** `a_float64_core_import_and_a_four_decimal_las_export_state_their_precision_reductions` is current `CORRECTNESS`, green at 1 passed / 0 failed / 0 ignored. It proves both lossy boundaries, controls exactly represented values from false positives, queries the stored f32 cast, inspects the returned reports and reads the declaration back from the LAS.
- **Supporting tests:** ordinary core import and registered LAS export/self-read tests exercise the same production paths; they remain supporting evidence rather than substitutes for exact T66.
- **Manual evidence:** `core-point-import` 0/55, `las-export` 0/11 and `data-conventions` 4/122 after regeneration; the exact precision scenario remains unchecked and the four data-convention checks are unrelated.
- **Git evidence:** reachable `173c7d2` contains the behavior and owned proof; the current Gate 2 increment reverifies and retains them without production change.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the automated contract; manual deliverable review remains open.
- **Next action:** retain exact T66 and both rendered declarations; Jauhar compares representative source, stored and exported values in Gate 4 without promoting synthetic evidence to field proof.

## SB-DIO-048 - Well identity in a container MUST come from the container, never from the filename.

- **Chapter evidence:** P2; chapter status `PARTIAL`; owned test `SB-DIO-T67`.
- **Atomic obligations:** use the container's well-identifying field; offer a filename only as a user-confirmable default; never select it silently.
- **Current source:** `src-tauri/src/parsers.rs::probe_las_well_identity` uses the mandatory byte-tolerant reader and returns container identity separately from a filename proposal. `src-tauri/src/ingest.rs` always prefers the parsed container value, ignores a contradictory confirmation when one exists, and refuses before writing when the source value is absent and no exact-path confirmation was supplied. `src-tauri/src/lib.rs`, `src/ipc.ts`, `src/ui/ribbon.ts` and `src/ui/importSetDialog.ts` expose the typed preflight and show a required confirmation input only for identity-absent files.
- **Qualifying acceptance tests:** `a_las_header_well_identity_overrides_the_filename_and_an_absent_header_only_offers_the_filename_until_confirmed` is `CORRECTNESS`, green at 1 passed / 0 failed / 0 ignored. It pins a colonless container value against a contradictory confirmation, suppresses the proposal when source identity exists, exposes only a proposal when absent, proves the refusal writes no well, and proves an explicit non-empty confirmation can commit.
- **Supporting tests:** the complete ingest module is green at 54 passed / 0 failed / 1 optional-data ignored, including colonless `WELL` controls across null and encoding fixtures. The registered malformed-reader corpus is green at 1 passed / 0 failed / 0 ignored with the new probe registered; neither supporting result replaces exact T67.
- **Manual evidence:** `las-import` 0/102, `data-conventions` 4/122 and `security-integrity` 3/107 after regeneration; the new visual/manual/field scenario remains unchecked.
- **Git evidence:** current topic-branch worktree carries the source-first resolver, typed preflight, confirmation UI and exact T67 pending this increment's commit.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. No parameter or deferred capability is required.
- **Next action:** retain exact T67; Jauhar visually inspects the identified and identity-absent dialog states and field-exercises representative deliveries in Gate 4 without promoting synthetic proof to field evidence.

## SB-DIO-049 - Writing a file our own reader would reject MUST be an error, not a warning.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned tests `SB-DIO-T68`, `SB-DIO-T69`.
- **Atomic obligations:** within the approved LAS/delimited pilot surface, route every registered DIO data writer's artifact through its own registered reader before success; make rejection fatal; catch a misdeclared depth unit before success. The chapter's section 7.2 E-3 separately records that a product-wide artifact contract has not been minted.
- **Current source:** `src-tauri/src/export.rs` makes `self_read` a required field of every `RegisteredWriter`, and `export_with_writer` calls it before setting `self_checked` or returning success. `src-tauri/src/lib.rs` exposes one DIO data-export command in the approved surface, `export_las`, and it reaches success only through that wrapper. `validate_las_output` uses the same full-curve LAS reader as import, compares the declared depth unit with the project unit, and validates reported row and curve counts.
- **Qualifying acceptance tests:** `every_registered_writer_reads_its_output_and_a_rejected_round_trip_is_an_error` and `a_feet_las_misdeclared_as_metres_fails_its_self_check_before_success` are `CORRECTNESS`, independently green at 1 passed / 0 failed / 0 ignored each. T68 enumerates the registry, requires positive `self_checked` evidence, then proves a syntactically corrupt artifact turns apparent writer success into an actionable error. T69 proves a readable feet-as-metres unit lie also refuses before success.
- **Supporting tests:** the public DIO command and registry inventory were reverified by direct source inspection. Ordinary LAS export success is not counted as proof because it cannot show that rejection is fatal. Office, report, plot, browser-CSV, model and backup writers are not counted as supporting evidence for this pilot-bounded row.
- **Manual evidence:** `las-export` 0/15, `las-import` 0/106 and `data-conventions` 4/122 after regeneration; the exact representative artifact/read-back exercise remains unchecked.
- **Git evidence:** reachable `2f4d4a2` contains the round-trip and unit self-check closure; the current Gate 2 increment reverifies the exact tree and records the pilot-versus-product boundary without changing production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the approved pilot DIO writer. A new pilot DIO writer must register its own reader and extend the inventory proof. Product-wide closure remains open under chapter section 7.2 E-3 and cannot be inferred from the LAS registry.
- **Next action:** preserve T68/T69; Jauhar exports and independently reopens a representative feet-based pilot LAS in Gate 4. A future coordinator-owned product contract must inventory and qualify every other artifact writer rather than laundering them through this LAS result.

## SB-DIO-050 - A re-gridded input MUST be detectable at import.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T70`.
- **Atomic obligations:** for exact T70, compare declared `STEP` with every genuinely adjacent finite source-depth token; flag the first disagreement as possibly re-gridded; do not flag a matching declaration; do not create a mismatch by reducing the tokens to f32 or by bridging a missing index row.
- **Current source:** `src-tauri/src/parsers.rs` parses the declared step and source depth tokens into normalized exact decimals before f32 storage, compares each adjacent pair until the first disagreement, and returns a typed `declared_step_mismatch_note`. A missing or unparseable depth resets adjacency. `src-tauri/src/ingest.rs` carries that note into the successful import result without rewriting or rejecting the samples.
- **Qualifying acceptance tests:** `a_declared_step_that_disagrees_with_actual_spacing_is_flagged_as_possibly_regridded_and_a_matching_step_is_not` is `CORRECTNESS`, green at 1 passed / 0 failed / 0 ignored. It names the mismatching declaration, observed interval and first row pair; proves an otherwise identical match is unflagged; proves exact 0.15240 source decimals at deep measured depth are not falsely flagged by f32 reduction; and proves a missing index row breaks adjacency.
- **Supporting tests:** none is promoted into T70. The exact test itself drives the real LAS importer and both positive and negative controls; a parser-helper-only comparison would be weaker evidence.
- **Manual evidence:** `las-import` 0/111, `data-conventions` 4/122 and `reframe` 1/44 after regeneration; the known-regridded versus matching delivery exercise remains unchecked.
- **Git evidence:** reachable `29d7504` contains the import warning and owned test.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for exact T70's declared-versus-actual contract. The chapter supplies neither a universal suspicious-round-interval threshold nor an acquisition-step source, so that separate detector stays absent and is not inferred from a tidy number.
- **Next action:** retain T70; Jauhar imports one known re-gridded delivery plus its matching control in Gate 4 and confirms `possibly` remains uncertainty rather than fabricated acquisition provenance.

## SB-DIO-051 - Provenance MUST be carried into the deliverable.

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DIO-T71`, `SB-DIO-T72`, `SB-DIO-T73`.
- **Atomic obligations:** classify every exported curve as measured, computed or model-derived; record method plus parameter values for computed curves; carry the saved-model record; encode the record inside LAS `~O`.
- **Current source:** `src-tauri/src/export.rs` builds a provenance record for every selected curve, requires a saved model for model-derived output, and writes the complete record into `~O` before its own-reader validation.
- **Qualifying acceptance tests:** `every_las_export_carries_measured_computed_and_model_provenance_in_the_file` is `CORRECTNESS`; its measured, computed and saved-model cases collectively map T71-T73 and inspect the file rather than only the return value.
- **Supporting tests:** the final/working-curve test proves status marking, not method/model provenance.
- **Manual evidence:** `las-export` 0/2, `processing-history` 0/7 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** reachable `b940fcb` contains the in-file provenance record and owned test.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a model curve without a saved model is deliberately refused; weakening that refusal would violate the record contract.
- **Next action:** preserve the refusal and verify the `~O` record in a representative client-facing pilot deliverable.

## SB-DIO-052 - Final and working curves MUST be distinguishable in an export.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DIO-T74`.
- **Atomic obligations:** allow both working and final curves to be exported while marking each status inside the file.
- **Current source:** `src-tauri/src/export.rs` carries the curve-status metadata into the LAS provenance section for every exported curve.
- **Qualifying acceptance tests:** `a_working_and_final_phie_are_both_exported_and_each_is_marked_in_the_file` is `CORRECTNESS`; it proves both curves are present and differently labelled, so omission or one default label cannot pass.
- **Supporting tests:** general provenance coverage exercises the same writer but does not substitute for the paired status assertion.
- **Manual evidence:** `las-export` 0/2 and `processing-history` 0/7 - unexercised.
- **Git evidence:** reachable `ba53311` contains the working/final status closure.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the automated file contract.
- **Next action:** include a paired working/final export in pilot deliverable review.

## SB-DIO-053 - Well-header fields MUST be mapped explicitly and identity MUST NOT be invented.

- **Chapter evidence:** P2; chapter status `PARTIAL`; owned tests `SB-DIO-T75`, `SB-DIO-T76`.
- **Atomic obligations:** use a documented header mapping; preserve unmapped headers verbatim; never synthesise any identity field.
- **Current source:** `src-tauri/src/parsers.rs` maps a selected subset of LAS well fields into typed metadata but does not carry all unmapped `~W` records verbatim. SB-DIO-048 now separates an absent `WELL` value from a filename proposal and requires explicit confirmation; the broader raw-header and all-identity-field contract remains incomplete.
- **Qualifying acceptance tests:** none maps T75 or T76; test class is `MISSING`.
- **Supporting tests:** known-header parsing and alias tests prove selected mappings only and cannot prove preservation of unknown fields or absence of invented identity.
- **Manual evidence:** `las-import` 0/102, `data-conventions` 4/122 and `security-integrity` 3/107 after regeneration; no SB-DIO-053 scenario is checked.
- **Git evidence:** the partial typed mapping and SB-DIO-048 `WELL` boundary are integrated; verbatim preservation and complete no-synthesis closure are absent.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no source parameter is missing. Unmapped headers remain dropped, and T76 still lacks a whole-contract proof for every identity field; the narrower `WELL` filename violation is closed under SB-DIO-048.
- **Next action:** preserve the complete raw `~W` map alongside typed fields and add T75/T76 with unknown-header and no-UWI controls without reintroducing any identity fallback.

## SB-DIO-054 - Every skipped frame, channel, curve and row MUST be counted and named.

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T77`, `SB-DIO-T78`, `SB-DIO-T79`.
- **Atomic obligations:** name and count every skipped frame/channel/curve/row with its rule; allow partial success only with that record; turn all-skipped input into an error.
- **Current source:** `src-tauri/src/dlis.rs` accumulates structured skip records through logical files, frames, channels, curves and rows; its LAS bridge reports malformed short rows with location, and all-skipped input returns an error.
- **Qualifying acceptance tests:** `every_skipped_frame_channel_curve_and_row_is_counted_named_and_all_skipped_is_an_error` is `CORRECTNESS`; it covers the good-plus-bad, all-bad and short-row obligations rather than asserting only a summary count.
- **Supporting tests:** malformed-corpus coverage checks reader behavior broadly but does not replace the DLIS item-level inventory.
- **Manual evidence:** `dlis-import` 0/11, `las-import` 0/57 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** reachable `d922ba8` contains the named skip accounting and owned test.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated degradation reporting; real malformed deliveries remain unexercised.
- **Next action:** retain the all-skipped refusal and capture partial-import notes against a representative malformed pilot artifact if available.

## SB-DIO-055 - An export that omits data MUST say what it omitted.

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T80`, `SB-DIO-T81`.
- **Atomic obligations:** write every held curve or name every omitted curve and reason in both the user result and file; report written and held counts.
- **Current source:** `src-tauri/src/export.rs` inventories all held curves, writes every eligible curve, and serialises any held item with the same reason into the operation result and LAS `~O` section.
- **Qualifying acceptance tests:** `every_held_curve_is_written_or_named_with_the_same_reason_in_the_file_and_result` is `CORRECTNESS`; it uses more curves than the old fixed selection and cross-checks names, reasons and counts across both reporting surfaces.
- **Supporting tests:** the writer round trip proves readability, not completeness or disclosed omission.
- **Manual evidence:** `las-export` 0/2, `generic-curve-store` 0/18 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** reachable `3276f27` contains the all-held-curve inventory and disclosure test.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated completeness reporting.
- **Next action:** verify the written/held inventory in a many-curve pilot export.

## SB-DIO-056 - A declared `STEP` MUST be verified across the whole index.

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DIO-T82`, `SB-DIO-T83`.
- **Atomic obligations:** compare every adjacent interval; emit the uniform step only when all intervals agree within the stated tolerance; otherwise write zero; never infer `STEP` from only the first pair.
- **Current source:** `src-tauri/src/export.rs` still calculates `STEP` as `depth[1] - depth[0]` and does not inspect the remaining index, so a later interval change is silently misdeclared.
- **Qualifying acceptance tests:** none maps T82/T83; test class is `MISSING`.
- **Supporting tests:** export round trips use uniform grids and therefore cannot expose the first-interval defect.
- **Manual evidence:** `las-export` 0/2, `reframe` 0/34 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** the divergent first-interval implementation is integrated; no closure commit exists.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the requirement says `within the stated tolerance`, but chapter section 5 supplies no STEP-uniformity tolerance. CONTRACT section 2 forbids selecting one by plausibility; an explicit exact-equality rule or cited tolerance is required before implementation.
- **Next action:** adjudicate and cite the tolerance (or explicitly specify exact equality), then scan the full finite index and add uniform/irregular T82-T83 controls.

## SB-DIO-057 - A zero on a log-scale curve MUST NOT be committed as a reading.

- **Chapter evidence:** P1; chapter status `ABSENT`; owned tests `SB-DIO-T84`, `SB-DIO-T85`.
- **Atomic obligations:** identify logarithmic curve families; count exact zeros before commit; require an explicit keep/convert decision; never rewrite automatically; record the decision.
- **Current source:** intake can report suspicious values generically, but no authoritative log-family registry classifies gas, resistivity and permeability families for this contract and no zero-decision commit record exists.
- **Qualifying acceptance tests:** none; T84/T85 are intentionally absent because testing them would require inventing the family membership; test class is `MISSING`.
- **Supporting tests:** generic missing-value and null-policy tests do not establish logarithmic family membership.
- **Manual evidence:** `delimited-intake` 3/27, `data-conventions` 0/45 and `security-integrity` 0/63 - unexercised for this decision.
- **Git evidence:** no implementation commit exists; commit state is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** chapter section 7.1 O-5 explicitly records log-family membership as unclassified. A family registry is a cited parameter and cannot be inferred from mnemonic intuition.
- **Next action:** supply an authoritative, versioned log-family registry and sources; then implement pre-commit counting, explicit keep/convert recording and T84-T85.

## SB-DIO-058 - Old `.xls` plate workbooks MUST be read from the published specification.

- **Chapter evidence:** P2; chapter status `PARTIAL`; owned tests `SB-DIO-T86`, `SB-DIO-T87`.
- **Atomic obligations:** read BIFF cell and drawing records from `[MS-XLS]`; preserve worksheet-to-picture association; drop and name an unresolved anchor; never guess its depth.
- **Current source:** `src-tauri/src/intake.rs` recognises BIFF signatures, while `src-tauri/src/images.rs` deliberately refuses legacy `.xls` plate extraction because the cell/drawing parser and anchor resolution do not exist.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** signature detection proves recognition only, and the correct refusal proves no guessed association; neither reads a plate workbook.
- **Manual evidence:** `image-data` 0/30, `office-deliverables` 0/39 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** the safe refusal is integrated; the requested reader is unimplemented.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the published `[MS-XLS]` route is named, but `DEC-003` has not selected legacy plate workbooks for the pilot and no implementation is present.
- **Next action:** if `DEC-003` selects the capability, implement from `[MS-XLS]` in a separate authorized increment and add T86/T87 without weakening the existing refusal first.

## SB-DIO-059 - Tabular `.xls` MUST be readable without the drawing layer.

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DIO-T88`.
- **Atomic obligations:** parse legacy BIFF cell records for a table without requiring any drawing-layer support.
- **Current source:** Intake can identify a BIFF stream but has no BIFF worksheet/cell-record parser; office readers support modern ZIP-based workbooks instead.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** BIFF signature disambiguation does not extract cell values.
- **Manual evidence:** `office-deliverables` 0/39, `core-point-import` 0/52 and `delimited-intake` 3/27 - the legacy table path is unexercised.
- **Git evidence:** no tabular BIFF reader commit exists; commit state is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-003` must decide whether legacy tabular `.xls` belongs in the paid pilot; the requirement already names `[MS-XLS]`, so no substitute decoder may be improvised.
- **Next action:** if selected, implement the smaller cell-record reader independently of drawings and add T88 against a licence-safe fixture.

## SB-DIO-060 - Format MUST be recognised by signature, and signature collisions MUST be handled.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DIO-T89`, `SB-DIO-T90`.
- **Atomic obligations:** choose readers from content signatures; report extension disagreement; disambiguate shared signatures using structure and report the selected format.
- **Current source:** `src-tauri/src/intake.rs` recognises BIFF stream variants, inspects shared ZIP containers by structure and routes text that is misnamed `.las` to the delimited reader while preserving a disagreement note.
- **Qualifying acceptance tests:** `a_biff5_stream_named_xls_is_chosen_by_signature_and_a_shared_zip_signature_is_disambiguated_by_structure` and `a_delimited_text_file_named_las_is_read_as_delimited_and_the_extension_disagreement_is_reported` are `CORRECTNESS`; together they pin a collision and an extension mismatch.
- **Supporting tests:** generic intake probes exercise ordinary extensions but do not replace the adverse controls.
- **Manual evidence:** `delimited-intake` 3/27, `workbooks` 0/19 and `las-import` 0/57.
- **Git evidence:** reachable `0a7281f` contains the signature-first routing and both owned tests.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the registered signature inventory; newly supported colliding containers must extend it.
- **Next action:** preserve signature-first routing and capture one extension-disagreement result in pilot intake evidence.

## SB-DIO-061 - Malformed input MUST be located, counted, named, and regression-tested against a corpus.

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DIO-T91`, `SB-DIO-T92`, `SB-DIO-T93`, `SB-DIO-T94`.
- **Atomic obligations:** locate file plus line/record plus failed rule; count affected items; forbid silent drop/coercion, panic, hang and unbounded allocation; run every registered reader over the in-repo malformed corpus in CI.
- **Current source:** `src-tauri/src/example_data_test.rs` derives the reader inventory from source adapters and drives every malformed fixture through each applicable reader with bounded truncation cases and diagnostic assertions.
- **Qualifying acceptance tests:** `malformed_input_is_located_counted_named_bounded_and_every_reader_runs_the_corpus_in_ci` is `CORRECTNESS`; it maps T91-T94 and fails when a reader is added without a corpus adapter.
- **Supporting tests:** individual parser refusals add local detail but do not replace the cross-reader inventory.
- **Manual evidence:** `security-integrity` 0/63, `las-import` 0/57, `dlis-import` 0/11 and `delimited-intake` 3/27.
- **Git evidence:** reachable `3b7b654` introduced the corpus contract; reachable follow-ups `aaa4172`, `f02571f`, `9f99e69` and `86b4b5c` keep the adapter inventory aligned with later readers.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the current registered-reader inventory; real-delivery malformed evidence remains separate from CI proof.
- **Next action:** keep the source-derived inventory mandatory and add every future malformed recurrence to the corpus before closing its defect.

## SB-DIO-062 - Text encoding MUST be detected, not assumed.

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DIO-T95`.
- **Atomic obligations:** detect UTF-8, UTF-16LE/BE with and without BOM, and Windows single-byte text; decode without rejecting a whole delivery for one stray byte; report the selected encoding.
- **Current source:** `src-tauri/src/parsers.rs::read_text_file_with_encoding` performs the shared byte-level detection and fallback, and all text-import paths route through `read_text_file` rather than direct UTF-8 file reads.
- **Qualifying acceptance tests:** `utf8_utf16_in_both_byte_orders_with_and_without_boms_and_windows_1252_are_imported_and_reported` is `CORRECTNESS`; it covers both BOM sides, no-BOM controls and Windows-1252 while checking the reported choice.
- **Supporting tests:** lower-level parser encoding tests exercise decoder details but do not replace the full import/result assertion.
- **Manual evidence:** `las-import` 0/57, `delimited-intake` 3/27 and `data-conventions` 0/45.
- **Git evidence:** reachable `e8b88a3` contains the shared detector, path migration and owned test.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the required encoding inventory.
- **Next action:** preserve the universal reader inventory and record detected encoding for representative Windows-origin pilot files.

## SB-DIO-063 - Non-ASCII paths and payloads MUST survive every sidecar boundary.

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned test `SB-DIO-T96`.
- **Atomic obligations:** preserve non-ASCII path and payload bytes unchanged through every Python-backed DLIS, office and image boundary; never decode piped stdin with the Windows ANSI code page.
- **Current source:** `src-tauri/src/office.rs` and `src-tauri/src/images.rs` parse requests from `sys.stdin.buffer`; the shared engine uses byte streams; the DLIS sidecar receives the source path as an argument rather than through text stdin. No embedded Python path exists.
- **Qualifying acceptance tests:** no T96 test sends the same non-ASCII path and payload through all three named boundaries; test class is `MISSING`.
- **Supporting tests:** `a_word_document_keeps_non_ascii_text_intact` proves the office payload path, and static runner tests require `sys.stdin.buffer`, but neither proves the DLIS and images end-to-end halves.
- **Manual evidence:** `dlis-import` 0/11, `office-deliverables` 0/39, `image-data` 0/30 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** the byte-safe runner implementations are integrated; the cross-sidecar owned proof is absent.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no parameter is missing; the remaining gap is end-to-end proof across the complete sidecar inventory.
- **Next action:** add T96 with a non-ASCII temporary path and payload through DLIS, office and images, ignoring only an optional-package subcase under the repository's package rule.
