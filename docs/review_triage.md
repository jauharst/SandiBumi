# Manual test plan — triage

**Generated 2026-07-31** against `docs/manual_test_plan.md` (250 tests, generated 2026-07-22)
and the 589 Rust tests in `src-tauri/src`.

## What this is for

You have 250 manual tests and limited evenings. This document sorts every one of them into four
piles by **who or what should actually do it**, so you spend your time on the tests that only you
can answer and stop re-checking things a machine already checks on every commit.

| Pile | Meaning | Count |
|---|---|---|
| **A** | Already checked by a Rust test that runs on every gate | **21** |
| **B** | Nothing checks it, but a Rust test could — worth writing | **45** |
| **C** | A machine could drive it end-to-end through the app | **147** |
| **D** | Genuinely yours — a judgement on real rock that no assertion can make | **37** |
| | **Total** | **250** |

**Read the piles honestly.** Pile A does not mean "this feature works" — it means the specific
claim that test makes is pinned by arithmetic on synthetic data. Pile C does not mean "already
automated"; it means automatable, and 64 of the 147 carry a blocker that has to be solved first.
Pile D is the number that matters: **37 tests, about one in seven, genuinely need you.**

## Progress — what has actually been retired

**Updated 2026-07-31.** This is the section to check. Everything below is written, passing in
`tools\check.ps1`, and committed — so these manual tests no longer need your evening.

| Pile | Done | Remaining |
|---|---|---|
| **A** — was already pinned before this work started | **21** | — |
| **B** — a Rust test now checks it | **43** | **0** (of 43 — T-IMP-06 and T-RT-18 regraded out) |
| **C** — a machine now drives it | **5 harness tests** | 81 unblocked (+61 blocked) |

### Pile A — the checklist

These were already covered when the triage was written; nothing here was work I did. The
checklist exists so you can see at a glance which are **fully** pinned and which carry a
**residual** — a part of the manual test the automated one does not reach. A `~` is not a
failure; it means the test makes most of the claim and the rest is still yours if you want it.

**Fully pinned (15)** — you can stop hand-checking these.

- [x] T-SHIP-04 — project format stamp (fresh, legacy, and a refused future format)
- [x] T-SHIP-05 — destructive migration backs up first; a normal open writes nothing
- [x] T-PREP-03 — FTEMP negative: TD_BHT ≤ 0 → MISSING, never ±Infinity
- [x] T-PREP-09 — GR hole-size correction, including the no-caliper pass-through
- [x] T-PREP-10 — density hole-size correction, in-gauge and beyond
- [x] T-PREP-15 — MASK machinery: flagged samples leave the percentiles *and* the output
- [x] T-PETRO-05 — vsh_dn degenerate triangle → MISSING, not ±Infinity
- [x] T-PETRO-08 — phi_son Wyllie/RHG + the opt-in compaction correction
- [x] T-PETRO-09 — phimax constant ceiling and the TVDSS-trend ceiling
- [x] T-PETRO-12 — RT ≤ 0 → MISSING in both Archie and Indonesia
- [x] T-ADV-07 — SSPW porosity ladder (PHIE = PHIT − CBW) and the clean-sand end
- [x] T-IMP-01 — LAS batch import: 3 files, names, curves, units, NULL sentinel
- [x] T-IMP-03 — malformed LAS: duplicated depth imports with a warning
- [x] T-IMP-11 — aux import: PERFORATION and XRD land per well, replace on re-import
- [x] T-WELL-17 — degenerate zone override reports honestly rather than "✓ N samples"

**Pinned with a residual (6)** — the arithmetic holds; the noted part is not asserted.

- [~] T-INT-02 / T-IMP-02 — duplicate LAS warns and stays a separate record. **Residual:** the
  display surface (status line, History row) is untested — that is the delivery, not the claim.
- [~] T-PETRO-08 — *also* listed above as full, with one caveat worth knowing: it pins the
  **current un-gated** compaction behaviour, i.e. the audited defect. If the DT_SH > 100 µs/ft
  gate is ever added, this test must change with it — it is not a vote that the behaviour is right.
- [~] T-BATCH-10 — cutoff sensitivity: NTG stays ≤ 1 across a mid-sample zone base.
  **Residual:** step 3's sweep-vs-Pay-Summary agreement is not itself asserted.
- [~] T-BATCH-14 — Monte Carlo seed reproducibility at seed 42, plus the zero-variance case.
  **Residual:** the seed-43 "differs but P50 stays close" step is not asserted.
- [~] T-MLEQ-09 — facies on a well with no usable curve reports an error, with a live control.
  **Residual:** it does not separately check that no FACIES version row was written.
- [~] T-IMP-14 — SCAL import: multi-file, auto-detect, Leverett-J fit, zero-row refusal.
  **Residual:** the sigma guard is a frontend string, not backend arithmetic.

### Pile B — the checklist

**`[x]` here means a Rust test now checks it and runs on every gate. It is NOT your
verification mark** — that lives in `docs/manual_test_plan.md` and nothing automated ever
touches it. A `[x]` below says the arithmetic is pinned; it does not say the feature works on
your wells.

**Done (43 of 43) — pile B is CLOSED**

- [x] **T-PETRO-02** — vsh_gr nonlinear options + version N+1 · `every_vsh_gr_transform_lands_on_its_published_coefficient` (`modules.rs`) + `re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run` (`equations.rs`). All eight transforms evaluated by hand against their published closed forms. **Found that the plan's Larionov labels are reversed** — see finding 21, which is the most consequential thing this pile turned up.
- [x] **T-ADV-17** — SandiMin re-run, lowercase prefix, no shadow rows · `a_re_run_under_a_lowercase_prefix_leaves_no_shadow_rows` (`multimin2.rs`). Both halves of the fix hold: the prefix is canonicalized and the case-insensitive DELETE reclaims prior-casing rows. Clean.
- [x] **T-REP-14** — DB Inspector: browse all 8 tables, page through · `every_inspector_table_returns_the_columns_it_declares` + `the_inspector_pager_lands_exactly_on_the_last_partial_page` (`db.rs`). Every `TABLE_SPECS` entry, and the pager checked on a count that does not divide evenly by the page size.
- [x] **T-REP-16** — DB Inspector negatives: bad input, stale row, read-only Aux · `an_inspector_edit_on_a_row_that_moved_fails_instead_of_reporting_success` + `aux_data_can_be_browsed_but_no_editor_will_write_to_it` (`db.rs`). The three sample editors all guard the 0-row case; the wells editor does not — see finding 20.
- [x] **T-PLOT-19** — Curve Edit negatives (invalid input, stale undo) · `a_set_constant_refuses_a_value_that_is_not_a_number` + `an_undo_replayed_after_the_curve_was_rewritten_splices_stale_values` (`curve_edit.rs`). Both audit findings re-examined: the invalid-input one is half fixed (finding 19), the stale-undo one is fully open and now pinned as-is.
- [x] **T-REP-02** — Composite render: layout, print scale, page size, pagination · `a_metre_of_formation_occupies_its_declared_millimetres_on_the_page` + `the_page_count_follows_the_print_scale_and_the_page_size` (`composite.rs`). The scale is now measured in the ARTWORK — the emitted depth labels — not just asserted as arithmetic.
- [x] **T-REP-09** — "Tables only" mode · `tables_only_drops_the_composite_pages_and_still_dates_the_cover_to_real_rock` + `a_composite_depth_window_re_dates_a_cover_whose_tables_ignore_it` (`report.rs`). One residual — see finding 18, which also explains why the audit's slowness is not a missing `if`.
- [x] **T-AUX-07** — Well-diagram track in Composite/Report + old layouts · `a_well_diagram_draws_its_strings_shoes_and_perforations_at_the_declared_depths` + `a_well_diagram_track_is_redrawn_on_every_composite_page` + `a_layout_saved_before_well_diagram_tracks_opens_as_curves` (`composite.rs`). Clean.
- [x] **T-SHELL-09** — NEGATIVE: project switch refused while a chain runs · `a_registered_chain_holds_the_project_switch_shut_until_it_is_really_finished` + `a_chain_that_never_reports_a_terminal_status_jams_the_guard_permanently` (`chain.rs`). The guard is correct and closes its own pre-flight window. One residual — see finding 17.
- [x] **T-SHELL-07** — Save Project As = backup copy · `save_as_writes_a_backup_copy_and_leaves_the_app_on_the_original` (`project.rs`). Backup-copy semantics confirmed from both sides: the copy is a snapshot, the later edit lands in the original only.
- [x] **T-PREP-18** — Splice Curves at depth · `a_gap_in_the_contributing_run_stays_a_gap` + `a_sample_with_no_depth_is_not_assigned_to_a_side` (`modules.rs`), beside the existing `splice_switches_at_depth`. Clean — a gap in the contributing run is never filled from the other run.
- [x] **T-ADV-13** — Saturation-Height on a deviated well (TVD wiring) · `a_deviated_wells_height_is_measured_from_the_survey_not_along_hole` (`workflow.rs`). **The audit finding is FIXED and the plan step is stale** — `ingest::materialize_tvd_curves` is the producer the audit said did not exist. See finding 14.
- [x] **T-PETRO-13** — zone parameter override: RW in one zone only · `a_zone_parameter_override_moves_that_zone_and_leaves_the_rest_untouched` (`workflow.rs`)
- [x] **T-REP-06** — Report render: cover, methodology, zone params, pay summary · `a_rendered_report_carries_the_plans_page_order_and_a_self_consistent_pay_table` + `a_dense_stringer_is_subtracted_from_the_sand_rows_hpv` (`report.rs`). Two residuals — see findings 15 and 16.

- [x] **T-REP-18** — SQL Query rejects writes · `readonly_query_refuses_every_write_shape_including_a_cte_prefix` (`db.rs`)
- [x] **T-SHIP-03** — missing perm curve fails loudly · `a_missing_curve_fails_by_name_rather_than_computing_on_another` (`lorenz.rs`)
- [x] **T-IMP-15** — LAS export: NaN→null, computed curves, mixed-case name · `export_writes_missing_as_null_and_carries_mixed_case_computed_curves` (`export.rs`)
- [x] **T-IMP-16** — export → re-import round trip · `an_exported_las_reimports_with_the_same_values` (`export.rs`)
- [x] **T-INT-03** — tops → zones, + empty-well negative · `zones_from_tops_are_contiguous_and_absent_tops_make_no_zones` + `a_top_below_the_logged_interval_never_makes_an_inverted_zone` (`db.rs`)
- [x] **T-INT-11** — restore v1, downstream consumes it · `a_restored_log_set_version_feeds_the_next_module_run` (`workflow.rs`)
- [x] **T-PREP-14** — GR normalization anchors per well · `gr_normalization_anchors_each_well_on_its_own_percentiles` (`workflow.rs`)
- [x] **T-PREP-02** — Formation Temperature: both modes land on their anchors · `formation_temperature_lands_on_both_of_its_anchors` (`modules.rs`)
- [x] **T-PREP-11** — a raw degF FTEMP never satisfies the computed-only input · `a_raw_ftemp_never_satisfies_the_computed_only_contract` (`workflow.rs`)
- [~] **T-PREP-13** — Gas Correction negatives · `the_empty_flag_refusal_names_the_users_curve_and_its_remedy_works` (`modules.rs`). **Graded honestly:** I put this in pile B and most of it was already in pile A — `gascorr_guards_stay_missing_or_error` and `gascorr_flag_gate_and_missing_inputs` already pinned the refusal and the all-MISSING-without-precalc behaviour. What was genuinely uncovered, and is now pinned, is the message TEXT (it must name the curve you picked) and the remedy it recommends (EVERYWHERE must actually work).

- [x] **T-PREP-05** — a per-zone gradient override reaches exactly its own samples · `a_per_zone_gradient_override_reaches_exactly_its_own_samples` (`workflow.rs`). **Found a defect while writing it** — see finding 6.
- [x] **T-WELL-16** — a per-zone override actually drives a module run · same test (it is the same claim, so one test retires both)
- [x] **T-PREP-16** — synthetic log: gap fill, raw kept, downward-only repair, and the masked-washout case · `a_synthetic_log_fills_gaps_keeps_raw_and_repairs_only_downward` (`modules.rs`) + `a_masked_washout_defeats_the_very_module_meant_to_repair_it` (`workflow.rs`). The second **pins the audited defect as-is**, not as correct behaviour.

- [x] **T-IMP-04** — malformed LAS: all-null depth **and** the truncated last row · `malformed_las_exemplars_fail_the_documented_way` (already existed) + `a_truncated_las_refuses_rather_than_importing_what_survived` (`example_data_test.rs`). **Your Blocked mark is now answerable** — there was no truncated exemplar to import, so `dataset for test/examples/bad_truncated.las` was added to the generator.
- [x] **T-IMP-08** — a repeated plug depth is dropped, first kept, import never aborts · `a_repeated_plug_depth_is_dropped_not_a_failed_import` (`parsers.rs`)
- [x] **T-IMP-10** — tops CSV: multi-well, unmatched, **blank WELL cells** · `tops_import_multiwell_and_default` (already existed) + `a_blank_well_cell_is_skipped_rather_than_charged_to_the_selected_well` (`ingest.rs`)
- [x] **T-IMP-12** — deviation: TVD/TVDSS **and** duplicate MD · `deviation_import_materializes_tvd_tvdss_curves` + `deviation_import_versions_surveys_and_switching_rebuilds_tvd` (both already existed) + `a_repeated_survey_station_is_dropped_not_a_failed_survey` (`parsers.rs`)

- [x] **T-BATCH-08** — Pay Summary negatives: PERM cutoff, bare well, per-well isolation ·
  `a_well_with_no_perm_at_all_quietly_escapes_an_active_perm_cutoff` +
  `one_unusable_well_cannot_zero_the_whole_pay_summary` (`workflow.rs`). **Found a defect while
  writing it** — see finding 7; the plan's Expected for step 1 was describing behaviour the code
  does not have, and now carries a Known issue line.
- [x] **T-BATCH-16** — Monte Carlo PERM cutoff with chain-produced PERM ·
  `adding_a_permeability_model_to_a_chain_switches_off_the_permeability_cutoff` (`montecarlo.rs`).
  **Pins the audited defect as-is**, with the working chain beside it as the control. Sharpens the
  audit's trigger — see finding 8.
- [x] **T-BATCH-17** — Monte Carlo vs chain with a bad-hole MASK ·
  `the_monte_carlo_chain_ignores_a_step_mask_the_real_chain_honours` (`montecarlo.rs`). **Pins the
  audited defect as-is.** Like T-PREP-16's masked washout it turns out to have **two causes**:
  `run_realization` never blanks, and `build_plans` never even fetches the flag curve, because the
  external-input set is built from LogIn args and MASK is an Option. Both are asserted, so a
  half-fix that touches only the executor will still fail.

- [~] **T-PETRO-14** — Wyllie-Rose variants ·
  `the_wyllie_rose_variants_carry_their_own_constants_and_two_are_one_equation` (`modules.rs`).
  **Graded honestly:** the edge cases were already pile A — `perm_wyllie_rose_edges` and
  `perm_wyllie_rose_negative_phie_missing_across_all_variants` pin PHIE = 0, missing and negative.
  What was genuinely uncovered is that OPT_WR selects different physics at all: the four constants
  at the plan's own domain point, TIXIER being byte-identical to MORRIS_BIGGS_OIL, the decade
  between oil and gas, and the silent fallback to TIMUR on an unknown variant.
- [x] **T-RT-05** — rocktyping with no permeability curve ·
  `rocktyping_without_a_permeability_curve_fails_and_writes_no_curves` (`workflow.rs`), with the
  same well plus permeability as the control. **Found a defect while writing it** — see finding 10.
- [x] **T-RT-07** — RT_LOG ladder and the inconsistent-cutoff case ·
  `an_inverted_cutoff_ladder_is_accepted_and_scatters_the_middle_class` (`rocktyping.rs`). Step 4
  asked what an inverted ladder does; it is worse than silent acceptance — the middle class splits,
  half promoted to BEST and half demoted to non-net in the same run. Pinned as-is.
- [~] **T-RT-08** — Pittman r10–r75 family and the APEX selector ·
  `the_pittman_radius_family_inverts_between_r50_and_r75_in_good_sand` (`rocktyping.rs`). **Graded
  honestly:** `pittman_r35_matches_published_regression`, `pittman_apex_selector_switches_controlling_radius`
  and `pittman_missing_inputs_stay_missing` already pinned RAPEX, the selector and the port class.
  New here: step 3's monotone-ordering claim, which **fails** (finding 9), and that an invalid
  sample blanks all ELEVEN outputs rather than the three the old test checked.

- [~] **T-ADV-10** — RtC/IMTS on an SSPW-only well ·
  `the_sspw_fallback_covers_imts_and_chooses_sample_by_sample` (`lrlc.rs`). **Graded honestly:**
  `rtc_falls_back_to_sspw_curve_names` already pinned the sw_rtc half. New here: **sw_imts**, which
  the manual test asks you to repeat and which nothing checked, and that `prefer` chooses per
  SAMPLE — a section reprocessed through SSPW leaves SSC curves above and below it, and a
  curve-level fallback would either ignore the new work or throw away the old. The fallback is also
  asserted to land on the SAME answer as the SSC path, not merely on *an* answer.
- [~] **T-ADV-11** — RtC with no porosity curve: honest failure ·
  `rtc_without_porosity_under_either_name_is_reported_not_returned_as_success` (`workflow.rs`).
  **Graded honestly:** `all_nan_module_output_reports_error_not_success` already pinned the guard
  on vsh_gr and electrofacies. New here: sw_rtc, which is the case the guard was written for and
  the nastier one — RES_DEEP is healthy, so a full-length SWT_RTC comes back MISSING at every
  depth, and on a saturation curve that is the difference between "no answer" and "no
  hydrocarbon". The control gives the well porosity under the FALLBACK name only, so the refusal
  is provably about absent porosity rather than the module failing to look for the second name.
  Note finding 10 applies here too: the blank curves are still written.

- [x] **T-REP-03** — composite depth window + invalid window ·
  `a_depth_window_that_selects_no_rock_is_refused_rather_than_rendered` (`composite.rs`).
  Four ways to select no rock — top below bottom, wholly under TD, wholly above the logged top,
  zero thickness — all refused. The test also pins the two behaviours the pane's page labels
  depend on: a window that OVERLAPS the data is honoured over the overlap rather than refused
  (1150 → 9000 renders 1150 → 1199.5, and the labels say 1199.5, because you cannot render rock
  that was never logged), and a NaN in one field is absorbed to the data bound by `f32::max`/`min`
  before the guard sees it — identical to leaving that field blank, and unreachable over IPC
  anyway since JSON has no NaN literal.

- [x] **T-REP-12** — batch export with a broken well in scope ·
  `one_unrenderable_well_costs_only_itself_in_a_batch_export` (`report.rs`).
  The broken well is listed FIRST, so a loop that gave up on first failure would fail this test.
  Both healthy wells get their own complete PDF, byte-different from each other; the broken well
  leaves no file at all, because `std::fs::write` is only reached after the render returns bytes.
  Also pins the UUID-not-name failure message the plan itself flagged as UX feedback — see
  finding 12 for the second, sharper defect this turned up.

- [x] **T-AUX-17** — equation runtime error mid-batch, per-well isolation ·
  `one_failing_well_does_not_poison_a_multi_well_equation_run` (`equations.rs`),
  `a_python_raise_in_one_well_leaves_the_rest_of_the_batch_intact` (`python_engine.rs`),
  `a_script_that_raises_on_only_some_samples_still_reports_a_clean_success` (`equations.rs`).
  **Both language paths, because they are different functions** — `lib.rs:1073` dispatches on
  `equation.language`, so the Rhai test says nothing about Python. Isolation holds in both, and
  is checked in the DATABASE rather than only in the return value: the failing well leaves zero
  rows, not an all-MISSING curve. The Python test runs for real on any machine with numpy (it
  skips with a printed reason elsewhere, matching its neighbours) and confirms the user's OWN
  `raise` message reaches the run summary. The third test records finding 13. What stays yours
  is step 3 — that the Processing panel *renders* those per-well ✗ marks; the progress calls
  behind it are already pinned by `python_equation_reports_progress_on_every_terminal_branch`.

All three items flagged **silent-wrongness class** (T-REP-18, T-SHIP-03, T-INT-11) are closed.

**Regraded out of pile B (2)**

- **T-IMP-06 (DLIS)** moves to **pile D — genuinely yours.** It needs a real `.dlis` file and the
  `dlisio` package; `dlis.rs::import_real_dlis` already exists for it but is `#[ignore]`d behind
  `SANDIBUMI_TEST_DLIS`, because DLIS is a binary vendor format and there is no honest way to
  synthesise a fixture that exercises sentinel screening and mnemonic collision. Point the
  variable at one of your own files and `cargo test -- --ignored` runs it; nothing automated can
  retire it.

- **T-RT-18 (legacy Multimin RECON_ERR at 3 tools)** cannot be run **or** pinned as written: the
  module it tests is **retired**. `modules::run_module` blocks `multimin` through
  `retired_module`, the solver body was deleted with it, and the spec survives only so a saved
  chain resolves by name. Following the test's steps today gets a loud "use SandiMin" refusal at
  step 3, not a RECON_ERR to read. Already covered by `multimin_is_retired_but_still_cataloged`.
  The *concern* is not obsolete, though, and is now pinned on the module that ships — see
  finding 11.

Pile B is therefore 43 items, not 45.

**Open (0)** — every pile-B item now has a Rust test running on the green gate.

### Pile C — the checklist

`npm run test:e2e` (see `docs/e2e_harness.md`) drives the built app. Optional; **never part of the
green gate**, and never will be — it needs a built binary and a WebDriver stack, and a gate that
can fail for reasons unrelated to the code is a gate people learn to ignore.

**`[x]` here means an end-to-end test drives it against the real app. It is NOT your verification
mark** — same rule as pile B.

**Done (30 of 86)**

- [x] **T-AUX-03** — Well Header: a TD-only edit must not lose the surface coordinates ·
      `wellheader.e2e.mjs`. A regression guard on a confirmed bug with an unusually quiet failure
      mode: `appState.selectedWell` is a SNAPSHOT that is not re-broadcast on a data change, the
      dialog writes every field unconditionally, so building it from that snapshot meant opening
      the header to fix a TD and silently erasing the easting, northing and UTM zone. Nothing
      downstream complains — the well just stops appearing on the map and the TVD work loses its
      datum. The test asserts the REOPENED FORM already shows the stored coordinates, which catches
      the bug one step before the damage, and then that a TD-only save leaves all three intact.

- [x] **T-REP-07** — The methodology table persists as a `report_template` document ·
      `report.e2e.mjs`. Asserted on the STORED DOCUMENT and on a freshly REBUILT pane, not on the
      textarea still holding what was typed: the field is populated from the document at build
      time, so only a rebuild tests the read path. The stored rows must keep their pipe-separated
      FIELDS — storing each line verbatim would still round-trip through this pane and then render
      as one column in the PDF, turning a methodology table into a list of sentences.

- [x] **T-BATCH-07** — Cutoffs & Pay Summary: the table and **the row invariants the triage flagged
      as worth testing** · `paysummary.e2e.mjs`. Net ≤ Gross, N/G within 0..1, PAY-net ≤
      RESERVOIR-net ≤ SAND-net, and HPV ≤ net × avg PHIE. **The test never picks a cutoff** — it
      runs the pane with whatever the pane prefills, because the invariants hold for ANY cutoff and
      inventing a VSH/PHIE/SWE value here would put an unsourced petrophysical number in the repo.
      Two guards make the coverage real rather than vacuous: the nesting check counts the
      comparisons it made and fails if none were available, and the HPV check does the same. An
      uninterpreted row (`n_classified == 0`, shown as "—") is SKIPPED rather than read as 0, which
      is the convention's whole point — a 0 net is byte-identical to a genuine wet zone.
      **Deliberately NOT asserted: that HPV is non-negative.** It is not guaranteed (finding 16, a
      dense stringer inside the net interval is subtracted), so asserting it would either fail on
      correct behaviour or encode a claim the code does not make. **Not covered:** the History
      entry, the status line and the PAYFLAG catalog version.

- [x] **T-RT-01** — Rock Typing ribbon group lists its modules and opens their panes ·
      `rocktyping.e2e.mjs`. The expected titles come from the MANIFESTS, not from a hard-coded
      list, so a rename moves both sides together. The menu is compared as a SET BOTH WAYS: every
      catalogued module offered, and nothing else — a menu with an extra entry is a module the
      catalog does not know about, which is how a retired one comes back. Step 5's singleton claim
      is pinned because a duplicate pane is not cosmetic: two panes for one module each carry their
      own scope and parameters, so editing one and running the other produces a run nobody
      configured, with nothing in the result to say which pane it came from.

- [x] **T-MLEQ-01** — ML pane opens with the full form · `ml.e2e.mjs`. Task and Algorithm (which
      drive the algorithm list and the output name), and the "Save model as" field — the control
      that makes a fitted model an ARTIFACT rather than a by-product, which is the whole point of
      `ml_models`, since a refit on different data is a different model.

- [x] **T-MLEQ-02** — The Inspector names the Python engine it found · `equations.e2e.mjs`. The
      note is checked against `python_status`'s own answer, so it cannot drift into naming an
      interpreter the run would not use. It also pins the distinction that is easy to lose in a
      refactor: **missing scipy is a NOTE, missing Python is a WARNING** — scipy is optional and the
      engine is fully usable without it, so calling its absence a warning would send the user
      installing something they may not need. The no-Python branch must name numpy AND
      `SANDIBUMI_PYTHON`, the only fix when discovery fails.
- [x] **T-MLEQ-05** — Equation negatives · steps 2–3 were already pinned in Rust
      (`python_reports_script_errors`, `worker_survives_a_script_error`,
      `equation_all_nan_output_reports_error`); step 1's "Save the equation before running it" is
      now `equations.e2e.mjs`. Frontend-only by construction: `run_equation` takes an id and there
      is no id to pass, so without the guard the run would fail deeper with a message about a
      missing id, which tells the user nothing. Also checks the refusal writes nothing.

- [x] **T-SHELL-13** — Undo/Redo with live labels · `undo.e2e.mjs`, driven through a real Database
      Inspector cell edit (the cheapest genuinely undoable action). The claim is not that the
      buttons exist: the tooltip must NAME what will be reversed, because "Undo (Ctrl+Z)" alone is
      the empty-stack wording and a button that will not say what it undoes is a dare rather than a
      tool. And the VALUE round trip is asserted, not just the button state — an undo that fires
      its callback without changing the data is indistinguishable from a working one until someone
      reads the number.

- [x] **T-SHELL-15** — History attribution: single-well vs batch module run · `history.e2e.mjs`.
      The contract in `workspace.ts` is sharper than it looks — a single-well run is attributed BY
      NAME, a batch to no well at all, and **neither is ever the globally selected well**. The test
      is built around that last clause: it selects well A, scopes the run to well B alone, and
      requires the row to name B. Attributing to the selected well would be right most of the time
      and silently wrong exactly when it matters, since the History is what answers "where did this
      curve come from" months later — and a row naming a well the run never covered is a wrong
      answer that looks authoritative.

- [x] **T-WELL-03** — A multi-selection feeds a batch pane's Selection scope, LIVE ·
      `scope.e2e.mjs`. The word live is the claim: the pane must follow a selection growing AND
      shrinking without being reopened, because a scope that only reads the selection at open time
      is T-WELL-06's bug one level down. Also checks that Selection with nothing selected resolves
      to NOTHING rather than quietly falling back to All — the dangerous default, where the user
      believes they are running on a handful and covers the whole field.

- [x] **T-BATCH-01** — Workflow Builder smoke, step picker clean · `workflow.e2e.mjs`. The picker
      is grouped by category and offers the catalog; the retired `multimin` is absent under both its
      id and its old display name. A chain built today that quietly wired it up would be refused at
      RUN time by `modules::retired_module` — after the user had arranged the whole recipe.
- [x] **T-BATCH-04** — Save, reload and delete the chain as a `workflow` document · same spec. The
      round trip is asserted on the STORED JSON, not on the dialog looking right: two steps in a
      deliberate order, cleared from the builder, reloaded, re-saved, and the ordered step list
      compared. **Order is the whole content of a chain** — VSH before porosity, porosity before
      saturation — so a round trip that kept the set and lost the sequence would run a different
      recipe and look identical in the list. Both save refusals also check that NO document is left
      behind; a half-saved chain is worse than none.

- [x] **T-WELL-04** — Well Groups manager: create, membership, rename, delete · `wellgroupmanager
      .e2e.mjs`. The rename test asserts the group ID SURVIVES: a rename that quietly created a new
      group and dropped the old one would look identical in the list while silently emptying the
      membership, and every batch dialog scoped to it would then run on nothing. Delete asserts the
      WELLS survive — a group is a view over wells, never a container of them. Caveat: rename goes
      through `window.prompt`, a browser dialog the driver cannot answer, so it is stubbed for the
      click; what is verified is the rename path behind the prompt, not the prompt.
- [x] **T-WELL-05** — Active group scopes the tree and freshly opened batch panes · same spec. The
      tree must show exactly the members, not merely the right NUMBER of rows. The "freshly opened"
      half needs a module NO other spec has opened: module panes are singletons, so asking for one
      already open re-focuses it with whatever scope it was left carrying.
- [x] **T-WELL-06** — NEGATIVE: an already-open batch pane does NOT re-scope · same spec, and it
      asserts the WRONG behaviour on purpose. AUDIT-2026-07-21 records that `wellScope.ts` does not
      follow a group change once a pane is built, so the pane shows the old group's count while the
      user believes they have re-scoped and the run covers the wrong wells. **The day this is fixed
      the test goes red — that is the alarm. Flip the assertion then; do not delete it.**

- [x] **T-REP-17** — SQL Query console · `panels.e2e.mjs`. Opens the pane, requires a runnable
      starter and RUNS it. **This is what caught finding 23** — the starter shipped opening with
      `--` comment lines and the read-only guard tests the first keyword, so the panel's own first
      click was refused. Fixed here; the guard's blindness to comments is pinned as-is and left to
      you. The provenance join in step 2 is not covered.
- [x] **T-AUX-01** — Performance monitor · `panels.e2e.mjs`. Every gauge must be labelled AND show
      a value: a gauge rendering an empty string reads as "measured, and it is nothing" rather than
      "not measured". The live tick and the leak watch stay yours.
- [x] **T-AUX-02** — Help tool · `panels.e2e.mjs`. The modal carries real text and names NO vendor,
      which is the provenance rule checked where a user actually reads it. The right-click route is
      not covered.

- [x] **T-SHELL-10** — Sessions: save, list, delete · `sessions.e2e.mjs`. The snapshot is asserted
      FIELD BY FIELD rather than "it is valid JSON": a snapshot that lost `layout` still parses,
      still restores without error, and simply rebuilds nothing — the workspace comes back empty
      and it looks like the save never worked. `well` must be PRESENT even when null, since its
      absence is how a session silently stops restoring the well it was taken on. Also checks the
      dialog CLOSES on success — one that stays open reads as a save that did not happen.

- [x] **T-MLEQ-16** — Curve Catalog: rows, live search, header sorting · `catalog.e2e.mjs`. The
      rows come from the backend and are pinned there; what is checked is the three things that
      exist only in the panel. The filter must NARROW to matching rows (a filter that merely
      highlights, or drops the wrong rows, also shrinks the table, so a count alone would not catch
      it) and must be reversible — a filter that cannot be cleared leaves a panel stuck on a subset,
      which reads exactly like a well that lost its curves. Sorting is asserted on the SECOND click
      reversing the first exactly, because one click can coincide with the order already there, and
      a header that only draws an arrow looks sorted without being sorted. Found on the way: with no
      active well the catalog renders a plausible static placeholder (GR, RES_DEEP, NPHI, RHOB, DT,
      SP) with no search box — so the spec waits on `#catalog-filter`, not on a row count.

- [x] **T-PREP-01** — Module dialog machinery smoke · `moduledialog.e2e.mjs`. The pane is opened
      the way a user opens it — Petrophysics tab, ribbon dropdown, menu item — and the item is
      found by the module's own manifest TITLE read from `list_modules`, so a rename moves both
      sides together. Asserts the scope control, the numeric parameters, the Outputs note (the
      only place a user is told what a run will write before pressing Run) and the leading
      "(none)" on a curve picker — a picker that lost it would bind the first curve in the list
      instead, and the module would run on a curve nobody chose.
- [x] **T-INT-06** — Negative trio · leg 1 was already pinned in Rust
      (`all_nan_module_output_reports_error_not_success`); legs 2 and 3 are now
      `moduledialog.e2e.mjs`. Both are frontend-only and could not be pinned in Rust: the backend
      computes happily with an out-of-range parameter and reports success on an empty well list,
      so nine lines in a click handler are the only thing stopping either. **The assertion that
      matters in both is that NO RUN STARTED** — compared as a project-wide `computed_curves`
      fingerprint before and after, because a dialog that prints a complaint and then runs anyway
      looks identical from the message alone.

- [x] **T-WELL-15** — Zones pane: add/update/delete, invalid input, per-well isolation ·
      `zones.e2e.mjs` (steps 2–5; From Tops, the History entries and Convert-to-zone are not
      covered). Two of these are FRONTEND-ONLY contracts that no Rust test could pin:
      `db::upsert_zone` has no validation whatsoever and would store an inverted zone, so
      `bottom <= top` is refused by the dialog alone. The refusal being silent is why the
      assertion compares every stored zone byte-for-byte rather than looking for a message — an
      inverted write keeps the name and the row count and swaps only the interval.

- [x] **T-WELL-02** — Multi-select: Ctrl-click, Shift-click range, ⇄ invert, plain-click clear ·
      `wells.e2e.mjs`. All four gestures against the real pane, plus the two things a naive version
      of this test would miss: that ctrl-click is a TOGGLE (a second one removes), and that it does
      not move the active well — which is the whole point of the gesture, since every open view
      follows the active well. Shift takes a range of two out of three deliberately, so an
      implementation that selected everything visible would fail rather than pass.

- [x] **T-INT-09** — Well-group scoping end-to-end · `wellgroups.e2e.mjs`. Create/activate,
      exactly-one-active, membership-replaces, and a group-scoped run that writes to members while
      leaving the outsider's curves byte-identical. A 2-of-3 group rather than the plan's 3-of-4,
      because test data stays the repo's own three example wells; the exclusion claim is the same.
- [x] **T-SHELL-01** — App launch · `shell.e2e.mjs`. Every declared tab has a panel and no panel is
      orphaned, the status bar exists, the dockview workspace was created.
- [x] **T-SHELL-02** — Ribbon tab walk · `shell.e2e.mjs`. Exactly one panel visible per tab, exactly
      one `.active`, every group captioned. Asserted on `checkVisibility()` rather than the `hidden`
      attribute — a CSS `display` rule has overridden `hidden` on these very panels twice, and the
      attribute reads correct in both of those bugs. **The overflow-chevron leg is NOT covered**: it
      needs a window resize, which is a native window operation.
- [x] **T-SHELL-03** — Language EN → ID → SU → JV → EN · `shell.e2e.mjs`. Keyed on "Project", whose
      four forms differ only by diacritics (Project / Proyek / Proyék / Proyèk), so the assertion
      proves the RIGHT dictionary was selected. "Petrophysics" would have proved nothing — it is
      "Petrofisika" in all three translations. Also checks an untranslated term stays English.
- [x] **T-ADV-01** — Advance tab smoke · `shell.e2e.mjs`. All five promoted manifests render, plus
      SandiMin / Calibrate RtC / Calibrate S / ML Models. Tooltip text and the chevron leg are not
      covered.
- [x] **T-SHIP-06** — The green gate from your own shell · `tools\check.ps1` is the test; it is run
      before every commit in this series and its `GATE GREEN` line is the assertion.

**Partially covered**

- [ ] **T-WELL-01** — Object tree: click activates, 📌 pin drives the workspace. `wells.e2e.mjs`
      covers the plain-click activation (exactly one row marked) and the ★ **favourite** pin,
      asserting it reached the project via `list_pinned_wells` rather than that a class toggled —
      a star that looks set and was never written gives a run scope that silently empties on the
      next launch. **Not covered:** the 📌 global well LOCK (a different control from the ★), the
      panel titles following the selection, and ★ persistence across a relaunch.

- [ ] **T-REP-01** — Composite & Report panes open and follow the selected well. `report.e2e.mjs`
      covers the opening half for both. **Not covered:** that they follow the selected well, and the
      placeholder text.

- [ ] **T-MLEQ-14** — ML negatives + the bad-hole Mask. `ml.e2e.mjs` covers step 1 (the
      no-input-curve refusal, plus that nothing was written) and step 3 — **which is where it found
      the plan is stale for the second time; see finding 24.** The Mask control EXISTS; the test
      pins it as present so its removal goes red. **Not covered:** step 2, the blind-well
      comparison needing two training wells.

- [ ] **T-REP-15** — DB Inspector edits persist, refresh, undo. `undo.e2e.mjs` covers the edit
      reaching the project, the undo restoring the old value and the redo reapplying it. **Not
      covered:** the status text, the Log View repaint, and the survives-a-restart check.

- [ ] **T-SHELL-12** — Dirty ● indicators. **NOT covered, and worth knowing why.** A first attempt
      asserted that a Database Inspector edit lights the Project tab's dot. It does not, and that is
      correct: `dirty.ts` tracks **named-save freshness** — workspace arrangement and log-view
      layout edits, the things a Session captures — while a data edit goes straight to DuckDB and is
      already persisted. Marking it dirty would train the user to ignore the dot, which is the only
      warning that their PANE ARRANGEMENT is unsaved. `undo.e2e.mjs` pins the inverse instead (a
      data edit must NOT report unsaved work) plus the placement rule, which holds either way: the
      dot must be a CLASS, never a text prefix, or the tabstrip reflows and shifts every other tab
      under the cursor. Covering the real claim needs a workspace change and a session save, and it
      is order-dependent across specs because `muteDirty()` runs after a save.

- [ ] **T-AUX-15** — Pinned wells as a batch-run scope. `scope.e2e.mjs` covers the ★ scope
      resolving to the pinned set and following a second pin without the pane being reopened, plus
      the mirror case: an emptied pinned set must resolve to nothing, never to All. **Not covered:**
      persistence across a relaunch, and which wells got fresh curves after a run. Note the pins are
      set by CLICKING THE STAR, not by `set_well_pin` — the scope resolves against
      `appState.pinnedWellIds`, which only the tree updates, so invoking the command writes the
      project and leaves the pane reading an empty set.

- [ ] **T-BATCH-06** — Workflow negatives. `workflow.e2e.mjs` covers the two SAVE refusals (no
      name, no steps), both frontend-only — `save_document` would store either quite happily — and
      checks neither leaves a document behind. **Not covered:** the empty-scope refusal on a run,
      and the Processing panel's per-well ⚠/✗ list for one broken well.

- [ ] **T-PETRO-03** — vsh_gr invalid parameters. `moduledialog.e2e.mjs` covers step 1's range
      refusal — the same guard, reached through vsh_gr's own pane — including that the message
      NAMES the parameter and its bounds, since a bare "invalid input" is true and useless on a
      form with several numeric fields. **Not covered:** step 2, the `GR_MA >= GR_SH` guard
      reaching the all-NaN honest-report path. That one is a cheap pile-B test on its own.

- [ ] **T-SHELL-11** — Quiet Ctrl+S re-save + Escape closes ribbon menus. `sessions.e2e.mjs`
      covers the Ctrl+S half: once the session has a name, Ctrl+S must write it again WITHOUT
      putting a dialog in the way, and must name it in the status line. A save that re-prompts
      every time is one people stop using, and the unsaved-state dot then stops meaning anything.
      **Not covered:** Escape closing ribbon menus.

- [ ] **T-RT-16** — Legacy Multimin filtered from the step picker. Now covered at BOTH ends:
      `workflow.e2e.mjs` covers steps 1–3 (the Workflow Builder's own picker offers no `multimin`,
      under either name) and `shell.e2e.mjs` covers step 5 (the ribbon cross-check), and covers it
      well: the retirement rests on two independent
      mechanisms — membership of `ADVANCED_MODULE_IDS`, which filters it out of the Petrophysics
      dropdowns, and a META caption outside `groupOrder`, which keeps it out of the Advance tab —
      so breaking either one puts the button back somewhere, and the test sweeps the whole ribbon.
      **Only step 4 is left** — that SandiMin is not offered as a chain step either.

### Where else to look

- **`git log`** — every increment's commit says what it pinned and why.
- **`tools\check.ps1`** — the count in `test result: ok. N passed` is the whole suite.
- **`tools\testplan-tally.ps1`** — still scores only YOUR marks in the manual plan, deliberately.
  Nothing automated ever ticks a box there.

## Two rules this document follows

**It does not touch your marks.** `REVIEW.md` and `manual_test_plan.md` are unchanged. A `[x]`
in either means *you* ran it on *your* wells, and a passing unit test is a different claim —
it says the arithmetic holds, not that the feature works in your hands. Never let one become
the other.

**Every cited test name was verified to exist.** 157 distinct test names are cited across the
four piles; each was grepped back against the inventory before it was written here. A wrong
"already covered" is the worst thing this document could contain — you would stop checking
something that nothing checks, and nothing downstream would catch it. Where coverage was partial,
the test was graded down rather than up.

---

## Pile A — already pinned by a test that runs on every gate (21)

You can stop hand-checking these. Each cites the test that makes the same claim.

| Test | What it checks | Pinned by |
|---|---|---|
| T-INT-02 | Duplicate LAS re-import warns (negative) | `las_import_warns_on_duplicate_well_name` (src-tauri\src\ingest.rs:1597) asserts exactly this claim: first import does not warn, re-import of the same normalized name warns "already exists", `assert_ne!(well_id)` proves a separate record and no silent auto-merge. Only the display surface (status line / History row) is untested, and that is the delivery vehicle, not the claim. |
| T-SHIP-04 | R-A: the project carries a format stamp | `fresh_project_is_stamped_with_current_format` (db.rs:3452) asserts `format_version` = FORMAT_VERSION and `written_by` starting "SandiBumi " — the exact two rows the manual `SELECT * FROM project_meta` expects. `legacy_project_without_stamp_is_stamped_on_open` (db.rs:3483) covers the invisible stamping of an existing project, and `future_format_is_refused_and_left_unmodified` (db.rs:3497) covers the refusal path the plan itself says is cargo-tested and unreachable by hand. |
| T-SHIP-05 | R-B: destructive migration backs up first; normal opens write nothing | `fresh_project_open_writes_no_backup` (db.rs:4676) asserts step 1's pass condition (absence of any `*-backup` file on a non-migrating open). `destructive_migration_backs_up_the_project_file_first` (db.rs:4615) asserts step 2 in full: the `field.pre-1-backup.duckdb` name from RELEASE §3.2, that it opens as a valid project with the PK still present and every pre-migration row, and that a second destructive run takes a timestamped name rather than overwriting. |
| T-PREP-03 | FTEMP negative: TD_BHT ≤ 0 → MISSING not ±Infinity | `ftemp_grad_bht_nonpositive_td_is_missing` (src-tauri/src/modules.rs:2835) |
| T-PREP-09 | GR Hole-Size Correction | `env_corrections_move_the_right_way` (src-tauri/src/modules.rs:3793) — in-gauge unchanged, +3% at 4 in enlargement (0.75%/in), no-caliper pass-through |
| T-PREP-10 | Density Hole-Size Correction | `env_corrections_move_the_right_way` (src-tauri/src/modules.rs:3793) — RHOB unchanged at CALI ≤ HD_REF, +0.016 g/cc at 4 in beyond it |
| T-PREP-15 | MASK machinery: BADHOLE mask changes percentiles and blanks outputs | `mask_excludes_flagged_samples_from_gr_normalize_percentiles` (src-tauri/src/workflow.rs:1590) — asserts both the blanked flagged sample and the shift in good-hole values from input-side masking |
| T-PETRO-05 | vsh_dn degenerate-triangle regression (no ±Infinity) | `vsh_dn_degenerate_triangle_is_missing_not_inf` (src-tauri/src/modules.rs:2815); the "Warned — no finite output" half is `all_nan_module_output_reports_error_not_success` (src-tauri/src/workflow.rs:1530) |
| T-PETRO-08 | phi_son Wyllie/RHG + OPT_CP compaction correction | `phi_son_wyllie_cp_opt_in_only_scales_wyllie` (src-tauri/src/modules.rs:2741) asserts exactly the plan's predicted outcome — raw Wyllie at OPT_CP OFF, ÷0.9 (≈ +11%) at ON with DT_SH 90, RHG untouched. It pins the CURRENT un-gated behaviour, i.e. the audited defect; if the DT_SH > 100 µs/ft gate is added this test must change with it |
| T-PETRO-09 | phimax constant + TVDSS-trend porosity ceiling | `phimax_constant_caps_and_preserves_missing` (src-tauri/src/modules.rs:2981) and `phimax_linear_trend_falls_with_depth` (3003) — cap behaviour, flat ceiling, MISSING preserved, PHIE_CAP/PHIE_MAX naming from the input curve, ceiling falling with depth |
| T-PETRO-12 | RT = 0 null-streak regression (no +Infinity) | `sw_arch_nonpositive_rt_is_missing_not_inf` (src-tauri/src/modules.rs:3223) and `sw_indo_nonpositive_rt_is_missing_not_inf` (3243) |
| T-ADV-07 | SSPW run: PHR-standard porosity ladder | `sspw_phie_removes_only_clay_bound_water` (src-tauri/src/ssc.rs:629) pins PHIE = PHIT − CBW, PHIFF = PHIE − CAPBW, CBW = VSH·VOL_CBW_SH and the SWIRR identity; `sspw_clean_sand_has_no_bound_water` (651) pins the clean-sand end |
| T-BATCH-10 | Cutoff Sensitivity: NTG stays ≤ 1 with a mid-sample zone/DST boundary | `compute_sweep_clamps_thickness_via_incl_h` (src-tauri\src\workflow.rs:1389) — asserts net is the clamped 1.5 m not 2.0 and NTG never exceeds 1 on a mid-sample zone base; supported by `sample_incl_thickness_clamps_zone_and_dst` (src-tauri\src\workflow.rs:1413). The sweep-vs-Pay-Summary agreement in step 3 is not itself asserted |
| T-BATCH-14 | Monte Carlo seed reproducibility | `hpv_distribution_is_ordered_and_reproducible` (src-tauri\src\montecarlo.rs:1877) — two runs at seed 42 assert identical hpv.mid and hpv.mean; `zero_variance_param_collapses_distribution` (src-tauri\src\montecarlo.rs:1902) covers the degenerate case. Residual: the seed-43 "differs but P50 stays close" step is not asserted |
| T-MLEQ-09 | Facies negative: well with no usable input curves | `all_nan_module_output_reports_error_not_success` (src-tauri\src\workflow.rs:1530) — runs electrofacies on a well whose every curve is NaN and asserts a per-well error rather than a green success, with a live-well positive control. Residual: it does not separately assert that no FACIES version row is written |
| T-IMP-01 | LAS batch import, multiple files at once | las_examples_import_end_to_end (src-tauri\src\example_data_test.rs:25) — 3 files, well names, standard curves, extras as RAW-set rows, NULL sentinel not stored; units pinned by generic_las_import_keeps_all_curves_and_converts_units (src-tauri\src\ingest.rs:1638) |
| T-IMP-02 | Re-import same-named LAS → duplicate warning, separate record | las_import_warns_on_duplicate_well_name (src-tauri\src\ingest.rs:1597) for the warning + separate record; the 2026-07-30 set rebuild is pinned by import_sets_attach_suffix_and_resolution (src-tauri\src\ingest.rs:2051) — attach to one record, auto-suffix, RAW priority |
| T-IMP-03 | Malformed LAS: duplicated depth section imports with warning | malformed_las_exemplars_fail_the_documented_way (src-tauri\src\example_data_test.rs:83) — asserts 35 of 40 rows and a "duplicate depth" warning on bad_dup_depth.las; both stores pinned by duplicate_depth_las_imports_standard_and_generic_curves (src-tauri\src\ingest.rs:1873) |
| T-IMP-11 | Aux data import: PERFORATION and XRD land per-well, replace on re-import | aux_import_xrd_and_perforation (src-tauri\src\ingest.rs:2433) — datasets independent, mixed num/text cells, re-delivery kept beside and counted alone; well routing by aux_import_routes_by_well_column (src-tauri\src\ingest.rs:2012) |
| T-IMP-14 | SCAL Pc/Sw import: multi-file, auto-detect, Leverett-J fit reported | scal_import_files_multi_format_and_replace (src-tauri\src\ingest.rs:2287) for multi-file auto-detect + replace-not-append, scal_import_fits_leverett_j (src-tauri\src\ingest.rs:2243) for the J fit (b<0) and percent conversion, scal_import_zero_rows_leaves_existing_data (src-tauri\src\ingest.rs:2360) for the zero-point refusal; the sigma guard is a frontend string (ribbon.ts:1562) |
| T-WELL-17 | NEGATIVE: degenerate zone override (TD_BHT = 0) reports honestly | ftemp_grad_bht_nonpositive_td_is_missing (src-tauri\src\modules.rs:2835) for MISSING-not-±Infinity, plus all_nan_module_output_reports_error_not_success (src-tauri\src\workflow.rs:1530) for the all-missing run being reported as an error rather than "✓ N samples" |

---

## Pile B — nothing checks it, but a Rust test could (45)

This is the work queue. Each of these retires a manual test permanently, in a test that runs in
seconds on every commit instead of costing you an evening. They are listed in cluster order.

Three are worth pulling forward regardless of the rest, because they are silent-wrongness class —
wrong answers that compile, plot and ship:

- **T-REP-18** — the read-only SQL guard. The existing `readonly_query_selects_and_rejects`
  covers a bare `DELETE` and `SELECT 1; DROP TABLE wells`, but **not** `UPDATE`, `INSERT` or
  `DROP` alone, and **not** the `WITH x AS (…) DELETE` smuggle the manual test specifically
  probes. This one would have been a damaging false "already covered".
- **T-SHIP-03** — a missing permeability curve must fail loudly, never compute on GR. The error
  messages exist in `lorenz.rs`; nothing asserts them.
- **T-INT-11** — a restored log-set version must actually feed the next module run. Steps 1–4
  are pinned; step 5, the one that matters, is not.

| Test | What it checks | The test to write |
|---|---|---|
| T-INT-03 | Tops import → zones from tops (+ empty-well negative) | Half pinned: `tops_import_multiwell_and_default` (src-tauri\src\ingest.rs:2388) covers multiwell routing, case-insensitive match, unmatched wells, re-import updating depth. **`db::zones_from_tops` (db.rs:3122) has NO test at all** — write one in `db.rs` asserting zones built from N tops are contiguous top-down with each zone's base = next top (last → TD), and that a well with no tops returns an empty vec rather than phantom zones or a panic. |
| T-INT-11 | Constellation versioning round-trip: two runs, restore v1, downstream consumes it | Steps 1–4 are pinned by `log_set_versioning_never_overwrites` (db.rs:4761), which keeps two versions, restores v1 and asserts the current curve is back at the v1 value. **Step 5 is not**: nothing asserts a downstream module run consumes the RESTORED values. Write it in `workflow.rs` — run `vsh_gr` twice at different GR_SH, restore v1, run `phi_den`, assert PHIE equals the v1-VSH computation and differs from the v2 one. The Constellations UI (version rows, "current" badge, hover params) stays a C. |
| T-SHIP-03 | R30: missing perm curve fails loudly, never computes on GR | The backend messages exist (`lorenz.rs:390` "permeability curve '{}' has no data in this well", plus the `shf_fit.rs` Leverett-J "needs a permeability curve" path) but **nothing asserts them** — `errors_when_no_valid_samples` (lorenz.rs:502) is a different claim (all samples invalid, message unchecked). Write in `lorenz.rs`: `run_lorenz` against a well with no PERM must return an error naming the requested curve and write nothing, with a positive control on a well that has one. Silent-wrongness class, so worth pinning in Rust rather than only in a UI harness; the dropdown-preselect half of the test stays a C. |
| T-PLOT-19 | Curve Edit negative tests (invalid input, stale undo) | in curve_edit.rs: an empty/non-finite `value` is refused rather than coerced to 0.0, and `restore_curve_values` reports a staleness mismatch when the curve was rewritten since the edit (both are open AUDIT findings; `missing_curve_and_bad_op_error_cleanly` at :646 covers neither) |
| T-REP-02 | Composite render: layout, print scale, page size, pagination | in composite.rs: page count scales inversely with print scale (1:200 ≈ 2.5× the pages of 1:500, 1:1000 ≈ half) and A3 gives fewer pages than A4 at one scale. Tiling with no gap/overlap and the header block are already pinned by `composite_paginates_and_renders_structure` (src-tauri\src\composite.rs:2568); `print_scale_is_physically_exact` (:2593) only checks one 10 m window on one A3 page |
| T-REP-03 | Composite depth window + invalid window (negative) | in composite.rs: `render_composite` with top below bottom, and with a window wholly outside the logged interval, returns Err rather than an empty or stale page set |
| T-REP-06 | Report render: cover, methodology, zone params, pay summary | two tests: in report.rs, `report_pages` emits cover → methodology → zone params → pay summary → composite in that order with the zone override present; in workflow.rs, the pay-summary invariants PAY ⊆ RESERVOIR ⊆ SAND, Net ≤ Gross, 0 ≤ NTG ≤ 1, HPV ≥ 0 on synthetic curves. Only the cover is pinned today (`cover_page_carries_title_and_well`, src-tauri\src\report.rs:592) |
| T-REP-09 | "Tables only" mode | in report.rs: `tables_only` yields exactly cover + methodology + zone params + pay summary with no composite pages, and the cover still states the true logged interval (the known slowness is a separate perf item, not an assertion) |
| T-REP-12 | Batch export, one PDF per well, with a broken well in scope | in report.rs: `export_report_batch` into a temp dir writes one `{WELL}_report.pdf` per good well, skips the curve-less well with a named failure, and leaves no partial file — no dialog needed, the folder is an argument |
| T-REP-14 | DB Inspector: browse all 8 tables, page through | in db.rs: `get_table_page` returns the whitelisted columns for EVERY `TABLE_SPECS` entry and the offset/limit arithmetic is right on the last partial page. `table_page_reads_and_cell_updates` (src-tauri\src\db.rs:3745) covers standard_curves only |
| T-REP-16 | DB Inspector negatives: bad input, stale row, read-only Aux | in db.rs: `update_standard_sample` on a depth that no longer exists returns Err (the 0-row case), and aux_data exposes no editable column. `table_page_reads_and_cell_updates` only pins the key-column and non-whitelisted-table refusals |
| T-REP-18 | SQL Query rejects writes (negative) | in db.rs: extend to UPDATE / INSERT / DROP on their own and to the CTE smuggle `WITH x AS (SELECT 1) DELETE FROM tops`, asserting the tops row count is unchanged after each. `readonly_query_selects_and_rejects` (src-tauri\src\db.rs:3704) only covers a bare DELETE and `SELECT 1; DROP TABLE wells` |
| T-AUX-07 | Well-diagram track in Composite/Report output + old layouts | in composite.rs: `draw_well_diagram` emits casing lines, shoe markers and perf ticks at the declared depths with the OD label, and a layout JSON carrying no `kind` deserializes as Curves. Neither is pinned today (`layouts_saved_before_crossover_still_load` at :2012 covers the crossover fields only) |
| T-AUX-17 | Equation runtime error mid-batch with per-well isolation | in equations.rs: a multi-well equation run where one well raises writes NOTHING for that well and commits every healthy well. `python_reports_script_errors` (python_engine.rs:716) and `equation_all_nan_output_reports_error` (equations.rs:1369) cover single-run errors, not batch isolation |
| T-PREP-02 | Formation Temperature: GRADIENT and BHT modes | assert ftemp_grad GRADIENT returns TSURF + TGRAD·depth exactly and BHT reaches BHT at TD_BHT (modules.rs); the BHT interpolation is already asserted inside `ftemp_grad_bht_nonpositive_td_is_missing` (src-tauri/src/modules.rs:2835), GRADIENT mode is not; re-run→v2 is pinned by `log_set_versioning_never_overwrites` (src-tauri/src/db.rs:4761) |
| T-PREP-05 | Pre-Calculation degC mode + per-zone gradient kink | assert a per-zone TEMP_GRAD override kinks precalc's FTEMP exactly at the zone boundary and leaves both segments linear (workflow.rs); the degC/degF/Arps half is pinned by `precalc_degc_mode_converts_for_arps` (src-tauri/src/modules.rs:2957) |
| T-PREP-11 | Neutron Env Correction: computed-only FTEMP contract | assert nphi_env_corr's FTEMP input is declared computed_only, so a raw LAS/generic-store FTEMP is not consumed and only the salinity term applies (modules.rs manifest + workflow.rs input resolution); the correction arithmetic is pinned by `env_corrections_move_the_right_way` (src-tauri/src/modules.rs:3793) but the computed-only contract — the audit finding this test verifies — is not |
| T-PREP-13 | Gas Correction negatives: no condflag → error; no precalc → all-NaN | extend the all-NaN honesty test to gascorr-without-precalc, asserting the run errors with rows_written 0 rather than a green full sample count (workflow.rs); the two guards themselves are pinned by `gascorr_guards_stay_missing_or_error` (src-tauri/src/modules.rs:3949) and `gascorr_flag_gate_and_missing_inputs` (3909), and the framework honesty by `all_nan_module_output_reports_error_not_success` (src-tauri/src/workflow.rs:1530) — which exercises vsh_gr/electrofacies, not gascorr |
| T-PREP-14 | GR Normalization: P3/P97 alignment across two wells | assert gr_normalize anchors percentiles PER WELL, not on the pooled distribution, when run across two wells with different GR spreads (workflow.rs) — the single-well mapping is pinned by `gr_normalize_maps_well_percentiles_to_reference` (src-tauri/src/modules.rs:4068) but a pooled-scope bug would pass it and fail this test |
| T-PREP-16 | Synthetic Log (KNN Predict): fill a gap, then the masked-washout case | an integration test through run_workflow_module asserting what MAX_RAW + MASK=BADHOLE does to the repaired sample inside the mask — the known-open finding (output-masking re-blanks the value the module exists to produce) is exactly what is unpinned (workflow.rs); the module itself is pinned by `log_predict_learns_association_and_fills_gaps` (src-tauri/src/modules.rs:4087) and `log_predict_max_raw_keeps_raw_where_higher` (4107) |
| T-PREP-18 | Splice Curves: run-to-run splice at depth | extend the splice test to assert a MISSING contributor yields MISSING rather than falling back to the other curve (modules.rs); the handover itself is pinned by `splice_switches_at_depth` (src-tauri/src/modules.rs:3390) |
| T-PETRO-02 | vsh_gr nonlinear options + version N+1 | assert each OPT_GR variant at mid-range GR: LARINOV1 ≈ 0.33 and LARINOV2 ≈ 0.22 against LINEAR 0.50, STIEBER1/CLAVIER below LINEAR, endpoints exactly 0 and 1 (modules.rs) — only LINEAR is pinned today (`vsh_gr_linear_and_limits`, src-tauri/src/modules.rs:3175) |
| T-PETRO-13 | zone parameter override: RW in one zone only | assert a zone-scoped RW override moves sw_arch's SWT only inside that zone and by the Archie ratio √(RW_zone/RW_dialog), sample-identical outside (workflow.rs) — the module-run zone-override path is unpinned; `zone_scoped_param_only_moves_its_zone` (src-tauri/src/montecarlo.rs:2154) is the Monte Carlo path, not this one |
| T-PETRO-14 | perm_wyllie_rose, all OPT_WR variants | assert the four variants at φ=0.25 / Swirr=0.15 give the documented magnitudes (TIMUR ≈ 900, MORRIS_BIGGS_OIL ≈ 700, MORRIS_BIGGS_GAS ≈ 70 mD) and that MORRIS_BIGGS_OIL and TIXIER are identical in this port (modules.rs); only the edges are pinned, by `perm_wyllie_rose_edges` (src-tauri/src/modules.rs:2872) and `perm_wyllie_rose_negative_phie_missing_across_all_variants` (2857) |
| T-ADV-10 | RtC/IMTS on an SSPW-only well: SSPW fallback | assert sw_imts falls back per-sample to PHIT_SSPW / CBW_SSPW and returns finite SWT_IMTS when the SSC-named curves are absent (lrlc.rs) — the RtC half is pinned by `rtc_falls_back_to_sspw_curve_names` (src-tauri/src/lrlc.rs:1246), the IMTS half has the same `prefer()` wiring (lrlc.rs:228) and no test |
| T-ADV-11 | RtC on a well with NO porosity curve: honest failure | extend the all-NaN honesty test to sw_rtc on a well carrying RES_DEEP but no PHIT_SSC/PHIT_SSPW, asserting an error and rows_written 0 rather than a full sample count (workflow.rs); `all_nan_module_output_reports_error_not_success` (src-tauri/src/workflow.rs:1530) covers the framework via vsh_gr/electrofacies only |
| T-ADV-13 | Saturation-Height on a DEVIATED well: TVD input is a no-op | an integration test that imports a deviation survey and runs sw_height through run_workflow_module, asserting HAFWL = FWL − TVD rather than FWL − MD — i.e. that a producer for the TVD input actually resolves (workflow.rs/satheight.rs). `sw_height_uses_tvd_and_allows_tvdss_fwl` (src-tauri/src/satheight.rs:335) supplies the TVD curve by hand, which is precisely why it misses this finding; `deviation_import_materializes_tvd_tvdss_curves` (src-tauri/src/ingest.rs:1698) is the other half of the join |
| T-ADV-17 | SandiMin re-run with a lowercase prefix: no shadow rows | assert run_multimin with output_prefix "mm" writes the uppercase MM_* names and that a re-run replaces them case-insensitively, leaving one row per curve at a bumped version (multimin2.rs + db.rs); nothing pins the prefix canonicalization today |
| T-RT-05 | Negative: rocktyping on a well with no permeability curve | assert `run_workflow_module("rocktyping")` on a well lacking PERM returns a per-well error and writes zero RQI/FZI/RT rows (src-tauri\src\workflow.rs) — the sibling guard `all_nan_module_output_reports_error_not_success` (src-tauri\src\workflow.rs:1530) exercises vsh_gr and electrofacies only |
| T-RT-07 | Rock Type from Cutoffs: RT_LOG ladder + inconsistent-cutoff behavior | assert what `rt_cutoff` does when VSH1 (0.50) > VSH2 (0.20) — pin the currently-silent acceptance in src-tauri\src\rocktyping.rs; steps 1–3 are already pinned by `rt_cutoff_ladders_by_vsh_and_phie` (src-tauri\src\rocktyping.rs:534) |
| T-RT-08 | Pittman Pore-Throat Radii: r10–r75 family + APEX selector | assert the radius family decreases monotonically with mercury saturation (PR10 > PR25 > PR35 > PR50 > PR75) on one valid plug in src-tauri\src\rocktyping.rs — `pittman_apex_selector_switches_controlling_radius` (src-tauri\src\rocktyping.rs:505) only checks each radius is finite |
| T-RT-18 | Legacy Multimin RECON_ERR at exactly 3 tools (known QC blindness) | assert in src-tauri\src\multimin.rs that with exactly 3 live tool rows plus the unity row RECON_ERR comes back ~0 even with a deliberately wrong endpoint (e.g. RHOB_SAND=2.75) while VOL_CLAY is badly off — pins the known blindness so the held fix is visible when it lands |
| T-BATCH-08 | Pay Summary negatives: PERM cutoff without PERM, bare well, per-well isolation | add a `run_pay_summary` test scoping a good well together with an uninterpreted one and assert the good well's rows survive (src-tauri\src\workflow.rs) — steps 1 and 2 are already pinned by `classify_sample_nan_propagation` (src-tauri\src\workflow.rs:1181, missing PERM fails an active PERM cutoff) and `pay_summary_marks_an_uninterpreted_well_as_classifying_nothing` (src-tauri\src\workflow.rs:1216), but per-well isolation is not |
| T-BATCH-16 | Monte Carlo PERM cutoff with chain-produced PERM (known bug) | assert in src-tauri\src\montecarlo.rs that a PERM cutoff changes Net/HPV when PERM is read from the DB but (today) does not when PERM is produced by the chain itself — pins `has_perm_cut`'s blind spot so the held fix is visible when it lands |
| T-BATCH-17 | Cross-check: Monte Carlo vs Workflow chain with a bad-hole MASK (known bug) | assert in src-tauri\src\montecarlo.rs that the in-memory chain executor's Net for a masked step matches `run_pay_summary` on the same chain — today it will not, which is exactly the finding to pin |
| T-SHELL-07 | Save Project As = backup copy | a project.rs test that save-as writes a valid copy at the new path while the live connection stays on the ORIGINAL (backup-copy, not switch-to-copy) and later edits land in the original only — only compact_project_shrinks_in_place_and_keeps_the_original (project.rs:444) exists today |
| T-SHELL-09 | NEGATIVE: project switch refused while a chain runs | a lib.rs test that switch/open_project returns the "A background job is still running…" error (lib.rs:268/293) while a chain job is registered active, and leaves the live connection unchanged — that guard has no test (lib.rs only has the three boot-gate tests) |
| T-IMP-04 | Malformed LAS: all-null depth + truncated last row | the all-null half IS pinned (malformed_las_exemplars_fail_the_documented_way, example_data_test.rs:83) but there is NO truncated-row exemplar or test — add `bad_truncated.las` to the examples folder and assert the "ended with N leftover token(s)" error (parsers.rs:313/560) with no orphan well |
| T-IMP-06 | DLIS import: sentinels screened, replaced-count, mnemonic collision | a dlis.rs test feeding a synthetic frame (−999.25 / −9999 / 1e35) through the store-insert side and re-importing it, asserting sentinels become NaN and the replaced-curve count; import_real_dlis (dlis.rs:276) is #[ignore]d so it never runs in the gate |
| T-IMP-08 | Core CSV with a duplicated plug depth imports (first kept) | a parsers/ingest test that a core table with one repeated plug depth imports rows−1, first occurrence kept, never a PK violation — core_import_roundtrip_and_replace (ingest.rs:1507) pins the set/re-delivery half only; the LAS-frame dedup tests do not cover the core CSV path |
| T-IMP-10 | Tops CSV: multi-well WELL column, single-well file, unmatched + blank cells | a tops test with a BLANK WELL cell in a multi-well file asserting the row is skipped, never routed to the selected well — tops_import_multiwell_and_default (ingest.rs:2388) covers routing/unmatched/single-well but has no blank-cell row (locations has locations_import_skips_blank_well_cell_not_default, tops does not) |
| T-IMP-12 | Deviation survey import: TVD/TVDSS, duplicate-MD survives | a parse_deviation_csv test that a duplicated MD station is dropped first-kept and the import still succeeds — TVD/TVDSS and survey versioning are pinned (deviation_import_materializes_tvd_tvdss_curves ingest.rs:1698, deviation_import_versions_surveys_and_switching_rebuilds_tvd ingest.rs:1740), the duplicate-MD rule is not |
| T-IMP-15 | LAS export: NaN→−999.25, computed curves, mixed-case name | export.rs has ZERO tests — write one asserting NaN writes −999.25, every computed curve appears in ~C, and a mixed-case computed name (`Vsh_final`) exports real values rather than an all-null column |
| T-IMP-16 | Round-trip: export → fresh project → re-import → values identical | an export→parsers round-trip test: spot values survive to LAS precision, −999.25 comes back as NaN not a spike, row count and depth range match the export |
| T-WELL-16 | Per-zone parameter override actually drives a module run | a modules/workflow test that vsh_gr with a `GR_MA=60` zone_params override on ZONE_A returns VSH=0 at GR=60 inside the zone and 0.40 outside, with the step exactly at the boundary — the override-beats-dialog-value path has no test (db.rs:3351/3385 pin the storage only, montecarlo.rs:2154 pins the MC zone scope) |

---

## Pile C — a machine could drive it (147)

**But read the split before believing the headline.**

| | Count | Meaning |
|---|---|---|
| No blocker — automatable today | **86** | Pure DOM: menus, dialogs, panels, tables, status text |
| Blocked by the WebGPU canvas | **47** | Log views and plots render to a canvas WebDriver cannot inspect |
| Blocked by the native file dialog | **17** | Tauri's file picker is an OS window, not DOM — gates most imports and exports |

(Three tests carry both blockers, which is why the column sums to more than 147.)

So the honest number for a first harness is **86 tests**, not 147. The canvas blocker has no
clean fix — a screenshot diff would fail on every legitimate rendering change. The file-dialog
blocker is solvable, but only by giving the app a test-only path that bypasses the picker, which
means changing the app to serve the tests. That is a decision to make deliberately.

The 86 unblocked tests are listed below — this is the scope for the end-to-end harness.

| Test | What it checks | Note |
|---|---|---|
| T-INT-06 | Negative trio: missing curve, out-of-range param, empty scope | Leg 1 is pinned — `all_nan_module_output_reports_error_not_success` (workflow.rs:1530) asserts an all-NaN run errors with `rows_written == 0` instead of a green success, with live positive controls. Legs 2 (inline "GR_SH: value must be between 0 and 1000." and no run starts) and 3 (empty scope message) are frontend validation in `moduleDialog.ts`/`wellScope.ts` — DOM text and a "no invoke fired" assertion. No blocker. |
| T-INT-09 | Well-group scoping end-to-end (3-well group) | No Rust test touches `well_groups` / `well_group_members` / group-filtered scope at all (grepped). Everything here is DOM: group creation, "Active well group: UAT-North (3 wells)", a newly opened pane defaulting to Group mode with 3 wells, and post-run per-well curve presence (readable via the Curve Catalog or a SELECT in the read-only SQL panel). No blocker. |
| T-INT-10 | Switch active group WITH a batch pane already open (negative) | This is a regression guard on a KNOWN open bug (AUDIT-2026-07-21, group-rescope gap in `wellScope.ts`). A WebDriver test asserting the open pane's scope row after a group switch is exactly the right shape — it goes red the day the bug is fixed and the assertion is flipped. Pure DOM, no blocker. |
| T-PERF-01 | App launch time on the big field project | **A machine timing, not a feel question** — wall clock from process start to the "Ready" status text and to the Wells pane holding 540 rows are both plain DOM waits, and far more reproducible than a stopwatch. Feeds ROADMAP #128/#129 directly. Blocker: none technical, but needs the 540-well fixture project on disk (nothing in the repo builds one; `pipeline_field_100well_stress` (pipeline_field_test.rs:280) is #[ignore]d and only clones one real well 100×). |
| T-PERF-02 | Single-module run across ALL wells: UI responsiveness + Cancel | **Mostly machine timing.** Total run seconds, and "did the main thread block" measured as DOM click round-trip latency sampled during the run — that is the real content of the known issue (`run_workflow_module` on the main thread) and a number beats an impression. Cancel semantics are pinned backend-side by `module_run_skips_all_wells_when_cancelled` (workflow.rs:1857). The one genuinely human leg is dragging the native window title bar, which the latency probe makes redundant. |
| T-PERF-03 | Full workflow chain across all wells: speed + live progress | **Machine timing** — total and per-step seconds against the plan's own "seconds to low minutes, not 30 min" threshold, plus the Processing panel's "Step k/3" and "Writing N well(s)…" text as DOM assertions. Step ordering and completion counts are pinned on synthetic data by `chain_runs_steps_in_order_and_completes` (chain.rs:273); the field-scale number is not, and `pipeline_field_full_run` (pipeline_field_test.rs:66) is #[ignore]d so it never gates. |
| T-PERF-04 | Cancel a 540-well chain mid-run | **Machine timing** — cancel-press to "Cancelled at step N" is a timestamp difference, exactly the "seconds not minutes" claim. `chain_honours_precancellation` (chain.rs:306) only pins the PRE-cancel case (nothing written); the mid-run drain latency and the post-cancel panel refresh are unpinned. Both Cancel buttons driving one flag is a DOM test. |
| T-PERF-05 | Field Dashboard compute on all wells | **Machine timing** for the "seconds, not minutes" claim and the re-compute. The must-NOT-write-FLAG-curves leg is already pinned by `pay_summary_stats_only_persists_nothing` (workflow.rs:1778); the "—" for empty aggregates and no `toFixed` crash are DOM/console assertions. Not A because the timing and grid legs dominate the test. |
| T-PERF-08 | DB size / WAL behavior + force-kill while IDLE (crash recovery) | The corrupt-WAL fallback is pinned by `resilient_open_recovers_from_corrupt_wal` (db.rs:3532) against a real captured crash fixture, but the manual claim is wider: kill while idle, relaunch, abnormal-exit prompt offering restore/Safe Mode, and every well/curve version/history row intact. A harness can kill the process, relaunch, assert the prompt and diff row counts against a pre-kill snapshot. The DB-size figure is an informational note, not a pass condition. |
| T-SHIP-01 | Packaged app launches under the hardened CSP | Not a Rust test and cannot be one — the CSP only exists in a packaged build. `tools\check.ps1` does NOT cover it either (that gate is `npm run build` + `cargo test`, neither of which applies `security.csp`). The harness builds, launches `sandibumi.exe`, asserts the ribbon/dockview rendered and the webview console holds zero CSP violations. Already machine-verified once on 2026-07-29 by exactly this route, which is the proof it automates. |
| T-SHIP-06 | The green gate from your own shell | **Explicitly covered by `tools\check.ps1` itself, not by any Rust test** — the test IS running that script and reading `GATE GREEN` plus a non-zero exit on failure. No WebDriver needed and no blocker; it is a one-line CI/scripted invocation checking the exit code and the count line. Bucketed C rather than A because no named Rust test pins the gate's own behaviour. |
| T-PLOT-05 | Layout Properties dialog + Save Layout | no blocker — dialog is DOM, the saved layout is a `documents` row readable via the SQL pane |
| T-PLOT-13 | Crossplot parameter handle, click-pick, zoom-to-cursor | no blocker for the substance (drag is dispatchable, the two zone-parameter writes land in the DB and in the status line) |
| T-PLOT-18 | Curve Edit dialog: five ops, bit-exact undo, History | no blocker (dialog is DOM, values checkable via SQL). Backend ops already pinned: `shift_moves_curve_and_restore_undoes_it` (src-tauri\src\curve_edit.rs:531), `blank_then_interpolate_bridges_the_gap` (:568), `set_and_scale_route_to_computed_and_generic_stores` (:600) — unpinned part is the status text, History entry and dataVersion refresh |
| T-REP-01 | Composite & Report panes open, follow the selected well | no blocker — pane titles and placeholder text are DOM |
| T-REP-07 | Methodology table persists as report_template document | no blocker — textarea round-trip, the `documents` row via the SQL pane, and an app restart are all scriptable |
| T-REP-10 | Report render writes FLAG_* curves; catalog + plots refresh | no blocker — Curve Catalog rows and the log-view repaint are the claim. Backend half pinned by `pay_summary_versions_flags_with_cutoffs_in_provenance` (src-tauri\src\workflow.rs:1703) |
| T-REP-11 | Composite/report exports in Processing History | no blocker — History pane is DOM (known issue: the entries are not written at all today) |
| T-REP-15 | DB Inspector edits persist, refresh, undo | no blocker — grid edits, status text, Log View repaint and a restart check |
| T-REP-17 | SQL Query: starter query + provenance join | no blocker — results grid is DOM; the "avg_gr plausible" line is a human rider |
| T-REP-19 | Theme switch repaints reporting & DB panes | no blocker — computed styles of the preview surface and grid are readable; the exported PDF is theme-independent by construction |
| T-AUX-01 | Performance monitor pane opens and updates live | no blocker — gauge rows, tooltips and the 1.5 s tick are DOM. Backend snapshot pinned by `snapshot_never_panics_and_percentages_are_sane` (src-tauri\src\health.rs:117); the leak watch is a human rider |
| T-AUX-02 | Help tool: ribbon button and right-click context help | no blocker — modal text, including the no-vendor-name check, is a string assertion on the DOM |
| T-AUX-03 | Well Header: TD/KB prefill, surface X/Y survival | no blocker — the confirmed bug was a stale frontend snapshot, so the dialog round trip is the test; the backend contract (a TD-only update preserves surface_x/y/zone) is separately worth pinning in db.rs |
| T-AUX-05 | Highlight edit / delete / Convert to zone / undo chain | no blocker — dialog, the real zones row after Convert to zone, the History entry and the undo chain are all inspectable |
| T-AUX-10 | Processing panel live detail: progress, step boundary, Cancel | no blocker — job card text, the "Writing N well(s)…" line and Cancel are DOM. Job lifecycle/cancel semantics pinned by `job_lifecycle_reports_progress_and_severity_sticks` (src-tauri\src\jobs.rs:453) and `cancel_counts_as_cancelled_only_once_a_worker_observes_it` (:546) |
| T-AUX-11 | Processing bulk same-failure summary + module-form one-liner | no blocker — one card per failure reason, the first-12-names truncation and the single form line are DOM. Step 6 pinned by `all_nan_module_output_reports_error_not_success` (src-tauri\src\workflow.rs:1530) |
| T-AUX-12 | Workflow Grid: Set-all RW/Mask, amber overrides, saved JSON | no blocker — grid cells, badges, and the saved workflow document compared byte-for-byte after reload |
| T-AUX-13 | Undo/redo depth: full walk down and up, redo invalidation | no blocker — six mixed edits, undo labels and button disabled states are all DOM |
| T-AUX-15 | Pinned wells (★) as a batch-run scope | no blocker — star persistence across a restart, the scope count, and which wells got fresh curves via the Curve Catalog |
| T-AUX-16 | Equation negative: Python in-place output guard | no blocker for the remaining claim (Curve Catalog EQUATION row + dataVersion refresh). Steps 3–5 are already pinned by `python_output_input_name_collision_guard` (src-tauri\src\python_engine.rs:688), which asserts the no-op rejection, the real in-place reassign, and that an output named `np` does not crash the worker |
| T-AUX-19 | Locations blank-WELL rows; SCAL fluid-system guard | no blocker for the unpinned half (the SCAL dialog's preset/σcosθ behaviour). Both backend claims already pinned: `locations_import_skips_blank_well_cell_not_default` (src-tauri\src\ingest.rs:2194) and `scal_import_zero_rows_leaves_existing_data` (:2360) |
| T-AUX-20 | i18n depth: Bahasa inside dialogs + PDF header policy | no blocker — dictionary coverage is a DOM text sweep and the round trip back to English is assertable; the PDF's fixed English headers are a Rust-side constant |
| T-PREP-01 | Module dialog machinery smoke (dropdown, pane form, "(none)") | no blocker — ribbon menu items, pane form fields and the leading "(none)" option are all DOM |
| T-PREP-17 | Depth Shift: block shift + dialog range validation | no blocker — the SHIFT=5000 inline validation message is frontend-only and cannot be pinned in Rust; the shift/resample/end-blanking is pinned by `depth_shift_resamples_onto_grid` (src-tauri/src/modules.rs:3361) |
| T-PETRO-01 | vsh_gr linear smoke run | no blocker — the run, the ✓, the VSH_GR/VSH catalog rows and the History line are DOM; VSH within 0–1 is checkable via the read-only SQL panel; `vsh_gr_linear_and_limits` (src-tauri/src/modules.rs:3175) pins the limited/unlimited arithmetic |
| T-PETRO-03 | vsh_gr invalid parameters (negative) | no blocker — step 1 is a frontend range message; step 2's all-NaN honest report is the framework path pinned by `all_nan_module_output_reports_error_not_success` (src-tauri/src/workflow.rs:1530), but the GR_MA ≥ GR_SH guard reaching it is not asserted |
| T-PETRO-18 | missing input curve + Cancel mid-batch (negative) | no blocker — the per-well ⚠/✗ breakdown, the "X/Y — Z need attention" line, the Cancelling…/Cancelled states and UI responsiveness are DOM; the backend cancel is pinned by `module_run_skips_all_wells_when_cancelled` (src-tauri/src/workflow.rs:1857) |
| T-ADV-01 | Advance tab smoke: all flagship buttons render | no blocker — button set, tooltips, absence of a legacy multimin button and the ›/‹ overflow chevrons are all DOM |
| T-ADV-02 | SSC run with LQR defaults: wiring + cross-checks | no blocker — defaults in the form, the 23-output hint, the Processing ✓, the History entry and the 23 Curve Catalog rows under INTERP v1 are DOM |
| T-ADV-05 | SSC negative tests: bad param, empty scope, no-GR well | no blocker — steps 1–2 are frontend validation messages; step 3's graceful degradation (VSHGR and all eight *_GR blank while the D-N outputs stay finite on a GR-less well) is unpinned in Rust and would be a cheap B on its own |
| T-ADV-15 | SandiMin wet→dry clay converter (KKT ONWJ workflow) | no blocker — what is untested is the Apply wiring (Illite's endpoints rewritten, CEC cell set, BoundWater auto-ticked, both previews refreshing on a temperature edit); the numbers behind it are pinned by `dry_clay_matches_the_kkt_example` (src-tauri/src/multimin2.rs:3019) and `dry_clay_cec_reproduces_the_bndwat_tie` (3040) |
| T-ADV-18 | SandiMin negatives: too few components / under-determined / empty scope | no blocker — the empty-scope message is frontend; `rejects_underdetermined_request` (src-tauri/src/multimin2.rs:2658) pins step 2's refusal, but the "select at least two components" guard (multimin2.rs:1001) has no test and is a cheap B on its own |
| T-RT-01 | Rock Typing ribbon group lists all four modules and opens their panes | no blocker — the ribbon menu, the auto-generated module pane fields and the singleton re-click are all DOM |
| T-RT-02 | Four rock-typing workspace panes in the ＋ add-panel menu | no blocker — menu entries, tab titles and the singleton move are DOM |
| T-RT-04 | winland_port re-run: port classes and version N+1 | no blocker — the catalog Set column and RT min/max are DOM; the R35≈6.5µm→RT 4 binning is already pinned by `winland_r35_and_port_class` (src-tauri\src\rocktyping.rs:425) and versioning by `log_set_versioning_never_overwrites` (src-tauri\src\db.rs:4761) |
| T-RT-10 | HFU Clustering: histogram method, K cap, no-core negative | no blocker — status line, cluster table and the frontend "K must be at least 2" refusal are DOM; the backend halves are pinned by `run_hfu_skips_invalid_and_notes_capped_k` (src-tauri\src\hfu.rs:490), `run_hfu_histogram_keeps_hfu_ids_contiguous_across_empty_gap` (src-tauri\src\hfu.rs:513) and `run_hfu_errors_when_no_valid_plugs` (src-tauri\src\hfu.rs:542) |
| T-RT-14 | Negative: SHF Fit with a starved point cloud | no blocker — the "Failed: …" status line and the cleared results table are DOM; the backend refusals are pinned by `height_fits_reject_too_few_points` (src-tauri\src\shf_fit.rs:1259) and `fit_rejects_degenerate_and_nonpositive` (src-tauri\src\shf_fit.rs:1204) |
| T-RT-16 | Legacy Multimin filtered from the Workflow step picker | no blocker — the picker's option list and both ribbon dropdowns are DOM; the backend half (multimin stays cataloged but retired) is pinned by `multimin_is_retired_but_still_cataloged` (src-tauri\src\modules.rs:2662) but that test does NOT cover the frontend filter |
| T-BATCH-01 | Workflow Builder smoke: pane opens, step picker clean | no blocker — pane singleton, the grouped module picker and the absence of Multimin/SandiMin are DOM |
| T-BATCH-03 | Chain outputs: Curve Catalog provenance, versioning, open-plot refresh | no blocker — the catalog rows and constellation versions are DOM; versioning is pinned by `log_set_versioning_never_overwrites` (src-tauri\src\db.rs:4761) and `batched_versioned_write_is_correct_across_wells_and_reruns` (src-tauri\src\db.rs:4838) |
| T-BATCH-04 | Save, reload, and delete the chain as a workflow document | no blocker — Save/Saved dropdown/Load/List-Grid toggle/Delete are all DOM |
| T-BATCH-05 | Cancel mid-chain: quick stop, honest status, plots still refresh | no blocker — the Cancelling/Cancelled phase, the builder status and the completed wells' curves are all readable; pre-cancellation is pinned by `chain_honours_precancellation` (src-tauri\src\chain.rs:306) for a different case (cancel before start) |
| T-BATCH-06 | Workflow negatives: no steps, empty scope, one broken well | no blocker — the two refusal strings and the Processing panel's per-well ⚠/✗ list are DOM |
| T-BATCH-07 | Cutoffs & Pay Summary: flags, table sanity, History, PAYFLAG version | no blocker — the whole summary table, status line, History entry and catalog version are DOM; PAYFLAG versioning with cutoffs in provenance is pinned by `pay_summary_versions_flags_with_cutoffs_in_provenance` (src-tauri\src\workflow.rs:1703). The row invariants (Net ≤ Gross, PAY-net ≤ RESERVOIR-net ≤ SAND-net, HPV ≤ Net×PHIE) are also worth a Rust test |
| T-BATCH-18 | Monte Carlo negatives: empty scope, dry well, cancel mid-run | no blocker — the two refusal strings, the "—" dry cells and the Processing panel's Cancel button are all DOM |
| T-MLEQ-01 | ML pane opens with the full form (smoke) | no blocker — every control, default and the task-driven algorithm/output-name swap is DOM |
| T-MLEQ-02 | Inspector opens; Python engine status is shown | no blocker — the "(engine: <path>)" suffix and the missing-numpy warning are DOM text; the probe itself is partly pinned by `scipy_is_available_when_installed_and_names_the_fix_when_not` (src-tauri\src\python_engine.rs:535), which is #[ignore]d and so does not run in the gate |
| T-MLEQ-03 | Python equation PHIE_TEST = PHIT × 0.9 on 2 wells | no blocker — save/run status, the catalog row with Set = EQUATION v1 and the two History entries are DOM, and the 0.9× check reads through the DB Inspector; the engine round trip is pinned by `python_vectorized_roundtrip` (src-tauri\src\python_engine.rs:664) |
| T-MLEQ-04 | Rhai equation (legacy per-sample engine) | no blocker — save/run status and the catalog row are DOM. Worth noting: no Rust test asserts the Rhai engine propagates NaN per sample (a NaN input yielding NaN, not 0), which is the claim's domain half |
| T-MLEQ-05 | Equation negatives: unsaved run, syntax error, unresolvable input | no blocker — all three are status-line strings. Steps 2–3 are already pinned by `python_reports_script_errors` (src-tauri\src\python_engine.rs:716), `worker_survives_a_script_error` (src-tauri\src\python_engine.rs:729) and `equation_all_nan_output_reports_error` (src-tauri\src\equations.rs:1369); only the "Save the equation before running it" guard is frontend-only |
| T-MLEQ-11 | ML classification: ML_CLASS + ML_CLASS_PROB | no blocker — the two-curve write, the automatic _PROB suffix, the accuracy/class_counts metrics table and the catalog rows are all DOM; classification behaviour is pinned by `classification_knn_labels_blobs_confidently` (src-tauri\src\ml.rs:1982) |
| T-MLEQ-14 | ML negatives + missing bad-hole Mask | no blocker — the two refusal strings and "is there a Mask control in this pane" are DOM. Important correction: the ML BACKEND does support MASK (`run_ml_mask_excludes_apply_samples`, src-tauri\src\ml.rs:1624; `run_ml_mask_excludes_training_outlier`, src-tauri\src\ml.rs:1830), so the audit line "no MASK support at all" is now true only of the dialog |
| T-MLEQ-15 | ML pane list staleness while open | no blocker — comparing the Input curves / Train wells lists before and after a reopen is pure DOM |
| T-MLEQ-16 | Curve Catalog: provenance, statistics, search/sort | no blocker — rows, columns, the hover provenance tooltip, live search filtering and header sorting are all DOM |
| T-MLEQ-18 | Delete a version: history pruned, current values untouched | no blocker — the two-click confirm, its ~3 s revert timer, the status line and the surviving v2 values are all inspectable |
| T-SHELL-01 | App launch (dev run) | no blocker — ribbon tabs/groups, status bar, sidebar pane titles are all DOM |
| T-SHELL-02 | Ribbon tab walk + overflow chevrons | no blocker — tab clicks, group captions, window resize to force the ›/‹ chevrons |
| T-SHELL-03 | UI language switch EN → ID → SU → JV → EN | no blocker — i18n is exact-phrase DOM substitution; assert label text per language and after relaunch |
| T-SHELL-08 | Clean relaunch restores last project + workspace | no blocker — needs the driver to close and relaunch the app between sessions; recents + autosave extras are DOM/DB |
| T-SHELL-10 | Sessions: Save/Open/delete session | no blocker — in-app modals, sessions stored as `session` documents |
| T-SHELL-11 | Quiet Ctrl+S re-save + Escape closes ribbon menus | no blocker — synthetic key events plus status-line text |
| T-SHELL-12 | Dirty-state ● indicators | no blocker — panel-tab and ribbon-tab dot classes, plus a tab-width assertion |
| T-SHELL-13 | Undo/Redo with live labels (+ History cross-check) | no blocker — button disabled state, tooltip text, History rows |
| T-SHELL-15 | History attribution: single-well vs batch module run | no blocker — the claim is which well name the History row carries; VSH range itself is pinned by vsh_gr_linear_and_limits (modules.rs:3175) |
| T-SHELL-17 | Interaction guards: right-click, reload, armed number fields | native WebView context menu — steps 2–3 assert an OS menu does NOT appear, which WebDriver cannot see; the F5/Ctrl+R confirm and armed number fields have no blocker |
| T-SHELL-18 | Crash resilience: autosave + recovery dialog | no blocker — needs an external process kill between driver sessions; the recovery dialog itself is in-app HTML |
| T-WELL-01 | Object tree: click activates, 📌 pin drives the workspace | no blocker — tree row classes, panel titles, ★ persistence across a relaunch |
| T-WELL-02 | Multi-select: Ctrl-click, Shift-click range, ⇄ invert, plain-click clear | no blocker — modifier clicks, header count text, status line |
| T-WELL-03 | Multi-selection feeds a batch dialog's "Selection" scope, live | no blocker — scope button state, live count, Custom… checklist seeding |
| T-WELL-04 | Well Groups manager: create, edit membership, rename, delete | no blocker — modal CRUD; rename uses an in-app prompt, not a native one |
| T-WELL-05 | Active group scopes the tree and freshly opened batch dialogs | no blocker — tree header text, member list, default scope of a newly opened pane |
| T-WELL-06 | NEGATIVE: already-open batch dialog does NOT re-scope | no blocker — assert the open pane keeps the stale group name and count (known AUDIT finding) |
| T-WELL-13 | Top autocorrelation: propagate by GR shape, untick, apply, batch undo | no blocker — proposals table, r≥0.70 pre-tick, Apply label, single-undo batch are DOM/IPC (engine pinned by autocorrelate_multi_propagates_markers_consistently tops.rs:926); whether the pick is geologically right on real deltaic GR stays a human read |
| T-WELL-14 | Autocorrelation negatives: no tops, bad curve, no targets | no blocker — the three message-pane texts and "nothing written / no History entry"; the backend errors are pinned by autocorrelate_reports_missing_curve_and_top (tops.rs:827) |
| T-WELL-15 | Zones pane: From Tops, add/update/delete, invalid input, History | no blocker — table rows, silent rejection of bottom<top, update-not-duplicate, per-well isolation, History entries |

*(The 61 blocked C tests are not listed here — they become a list worth having only once a
blocker is actually solved.)*

---

## Pile D — genuinely yours (37)

This is the real answer to "what must I check myself". About **one test in seven**. Every one of
these turns on a judgement about real rock, a visual read, or a feel for whether the app responds
— none of which an assertion can make.

| Test | What it checks | Why only you can |
|---|---|---|
| T-INT-05 | Interpretation chain by hand: vsh → phi → sw → perm | The claim is a petrophysical read on real Mahakam logs: VSH high in shales and low in clean sand, PHIE mirroring VSH inversely, SWE ≈1 in wet intervals and low in pay, PERM spanning orders of magnitude while tracking PHIE. `chain_runs_steps_in_order_and_completes` (chain.rs:273) proves the plumbing on synthetic data; no assertion can make the geological judgement. The log-view repaint leg is additionally a WebGPU canvas. |
| T-PERF-06 | Correlation with 20+ strips | **This one is not a timing question.** The pass condition includes "correlatable markers align horizontally after flattening" — a geological judgement on real Mahakam tops that no assertion can make, and the reason a human runs it. The redraw-latency leg IS C-able (frame time during pan/zoom), and the strip rendering is a **canvas** the harness cannot inspect, so automating it would prove the cheap half and skip the point. |
| T-PLOT-02 | Depth scale, zoom, pan (true 1:N) | "pan is smooth on a 15-curve layout with no stutter" is a timing feel, and 1:200 ≈ 5 mm/m on screen is a ruler check |
| T-PLOT-08 | Core-point overlays in the log view | petrophysical judgement — "CPOR diamonds should track the PHIE curve within a few p.u. in good hole", plus log-scale registration read by eye |
| T-PLOT-12 | Crossplot overlays: chartbook, matrix points, core, T-S | the point of the test is a lithology judgement — a clean Mahakam sand must plot on/left of the quartz line on real data |
| T-REP-05 | Composite export PDF, verify against on-screen log view | page-by-page visual comparison against the live log view plus a ruler on the printed page; `print_scale_is_physically_exact` (composite.rs:2593) and `pdf_is_valid_and_multipage` (:2605) cover geometry and PDF validity but not the comparison, and step 5's FACIES block printing is unpinned (`draw_class_blocks` has no test) |
| T-AUX-09 | Contacts MD ↔ TVDSS flattening across a deviated well | needs a real deviated well and a judgement against its survey (MD-vs-TVD displacement consistent with 1/cos(inc)), on top of the known TVDSS-curve exposure gap |
| T-PREP-04 | Pre-Calculation FTEMP/FPRESS/RMF/CT/CXO plausible on a known well | the acceptance is "physically plausible for Mahakam" against the well's own mud-report Rmf — a judgement; the arithmetic (Arps, hydrostatic gradient, CT=1000/RT, MISSING at RT≤0) is pinned by `precalc_kk_fits_and_conductivities` (src-tauri/src/modules.rs:2704) |
| T-PREP-06 | Bad-Hole QC Flag: washout flagging vs DRHO/CALI | "the flagged set visually matches the washouts the caliper shows" is a call on whether the default thresholds suit this hole; the rule itself is pinned by `badhole_flags_washout_and_drho` (src-tauri/src/modules.rs:3405) and the all-(none) honest report by `all_nan_module_output_reports_error_not_success` (src-tauri/src/workflow.rs:1530) |
| T-PREP-07 | Data Conditioning Flags: coal/tight/crossover/shoulder | "flags land on the intervals you would hand-pick" on real coal and tight streaks; the detection rules are already pinned by `condflag_detects_coal_tight_and_crossover` (src-tauri/src/modules.rs:3439), `condflag_washout_is_not_coal_and_xcond_option` (3470), `condflag_min_thick_drops_spikes` (3494), `condflag_shoulder_extends_past_bed_edges` (3523) |
| T-PREP-08 | Neutron Matrix Conversion LS/SS/DOL + chart spot-check | step 4 needs a real gas sand to show crossover at the default XOVER_MIN after conversion, and the hand-chart check is at the user's own reading; the shipped worked example is pinned by `nphimat_reproduces_por5_worked_example` (src-tauri/src/modules.rs:3707) |
| T-PREP-12 | Gas Correction with precalc + condflag: plausible de-gassed density | acceptance is a plausible de-gassed RHOB and GASDEN on a known Mahakam gas well; the physics is pinned by `gascorr_papay_gas_density_pinned` (src-tauri/src/modules.rs:3853) and `gascorr_converges_on_gas_sand_and_skips_water` (3869) |
| T-PETRO-04 | vsh_dn crossplot VSH + VSH_DN_FLAG | the acceptance is flagged streaks across a real gas interval and across kaolinite-rich vs illite intervals — a clay-type judgement; the flag rule (off-triangle, GR divergence, inert without GR) is pinned by `vsh_dn_flags_offmodel_and_gr_divergence` (src-tauri/src/modules.rs:2762) |
| T-PETRO-06 | phi_den density porosity incl. shale branch | "clean Mahakam sand PHIE ≈ 0.20–0.33" is a porosity answer on real rock; NOTE the plan's line "phi_den/phi_dn had zero unit tests — your hand-check here is the coverage" is stale: `phi_den_shale_branch_limits_and_missing` (src-tauri/src/modules.rs:3083) pins the shale branch, both OPT_PHIEMAX caps and missing propagation |
| T-PETRO-07 | phi_dn crossplot porosity, AVERAGE vs GAS_RMS | the claim is about porosity restored in a real gas interval; GAS_RMS > AVERAGE, the shale branch and the shale-reduction clamps are pinned by `phi_dn_crossplot_shale_reduction_and_branches` (src-tauri/src/modules.rs:3121) |
| T-PETRO-10 | sw_arch smoke + coal/tight zero-porosity guard | "≈1 in known wet sands, 0.2–0.5 in pay" is a saturation answer on real rock; the guard and the Archie arithmetic are pinned by `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf` (src-tauri/src/modules.rs:3206) and `sw_arch_clean_sand` (3191) |
| T-PETRO-11 | sw_indo vs sw_sim vs sw_arch on the same interval | the whole claim is a three-model comparison over a real shaly-sand interval; the individual equations are pinned by `sw_indo_full_vs_simple` (src-tauri/src/modules.rs:3302), `sw_sim_matches_quadratic_solution` (3323) and step 4's VSH=1 case by `sw_sim_schlumberger_pure_shale_is_all_water` (3279) |
| T-PETRO-15 | perm_coates default constant | the test is a comparison against the user's own Geolog run plus a sign-off on whether 100 or 70 ships as the default — a judgement, not an assertion; the formula and edges are pinned by `perm_coates_computes_and_handles_edges` (src-tauri/src/modules.rs:2885) |
| T-PETRO-16 | perm_transform vs core φ–k + overflow regression | step 2 is "does the transform track my core cloud" — a petrophysical eyeball on real RCAL data; the overflow half is pinned by `perm_transform_overflow_is_missing` (src-tauri/src/modules.rs:2897) |
| T-PETRO-17 | thin_bed_ts Thomas-Stieber decomposition | needs a known laminated LRLC interval and a clean sand for contrast; the laminated and dispersed end-members are pinned by `thin_bed_ts_pure_laminated_and_dispersed` (src-tauri/src/modules.rs:4127), but PHIE_LAM (> PHIT in laminations, capped at PHI_SD_MAX, MISSING at VSAND ≈ 0) is unpinned and is a cheap B-worthy gap |
| T-ADV-03 | SSC domain validation: closure, porosity ladder, bound-water split | closure and the ladder must be hand-summed at a real clean sand, silty interval and shale, and PHIT_SSC judged plausible for Mahakam; `ssc_clean_sand_is_mostly_sand` (src-tauri/src/ssc.rs:509) and `ssc_shale_point_is_clay_dominated_with_low_phie` (533) pin closure and the ladder on synthetic points — the SWIRR_EFF = 1 (not 0) at a zero-PHIE shale point sub-claim is unpinned and is a cheap B-worthy gap |
| T-ADV-04 | SSC GR-equivalent family (*_GR): eyeball closely | the plan calls this the least-proven output family and asks for an eyeball against the reference-suite run — a domain sign-off. NOTE the plan's Known-issue line is stale: `ssc_gr_equivalent_family_closes_and_guards_degenerate_vwsh` (src-tauri/src/ssc.rs:553) is in the tree, runs in the gate, and already pins closure, both porosity identities and the degenerate-VWSH blanking |
| T-ADV-08 | SW-RtC after SSC: corrected Rt, Sw vs Indonesia | the acceptance is that SWE_RTC reads visibly lower than SWE_INDO in a real high-clay microporous pay interval and agrees in the clean water sand — the method's entire claim, on rock; the mechanics are pinned by `rtc_lowers_sw_versus_archie_when_capillary_water_present` (src-tauri/src/lrlc.rs:1219) and `rtc_correction_is_capped_so_rt_stays_finite` (1284) |
| T-ADV-09 | SW-IMTS after SSC: iterative Waxman-Smits-family Sw | same shape — SWT_IMTS must tell the same geological story as SWT_RTC on a real LRLC pay interval; `imts_converges_and_sits_below_archie_in_clayey_rock` (src-tauri/src/lrlc.rs:1297) and `imts_credits_clay_conductivity_in_pay_zone` (1337) pin the equation |
| T-ADV-12 | Saturation-Height (Leverett) with SCAL-fitted A/B | the acceptance is that SWH broadly tracks the resistivity Sw through a real transition zone, from a lab Pc file chosen in a native file dialog; `leverett_fit_recovers_synthetic_curve` (src-tauri/src/satheight.rs:273), `sw_height_leverett_transition_zone_shape` (300) and `scal_import_fits_leverett_j` (src-tauri/src/ingest.rs:2243) pin the fit and the curve shape |
| T-ADV-16 | SandiMin full run: closure, RECON, vs deterministic answers | MM_PHIT against PHIT_SSC and MM_SWT against SWT_RTC on a real well is a petrophysical reconciliation; closure, the mineral recovery and RECON are pinned on synthetic input by `unity_is_exact_and_bounds_hold` (src-tauri/src/multimin2.rs:2339), `recovers_known_three_mineral_mix` (2317) and `recon_qc_emits_per_tool_curves_and_flags_endpoint_error` (2163) — the real-well test `e2e_pef_and_vsh_on_real_well` (2761) is #[ignore]d and does not run in the gate |
| T-RT-03 | rocktyping GHE run: outputs, catalog provenance, History, plot refresh, domain sanity | steps 7–8 ask whether R35 falls in the Mahakam sand range and whether PERM_RT scatters around 1:1 within half a decade — a rock judgement; the arithmetic is pinned by `fzi_and_rqi_match_amaefule_formula` (src-tauri\src\rocktyping.rs:412) |
| T-RT-06 | Lucia Rock-Fabric Number on a well with carbonate stringers | step 5 asks whether RT_LUCIA populates the carbonate streaks and stays blank in the clastic background — only real stringers answer that; note the "RFN outside 0.5–4 → MISSING" claim is NOT pinned (`lucia_rfn_round_trips_and_classes`, src-tauri\src\rocktyping.rs:463, only checks classes 1 and 3 and invalid φ/k) |
| T-RT-09 | HFU Clustering (Ward): table, crossplot, histogram, highlight, theme repaint | step 9 asks whether the K=5 partition splits an obviously single rock-type population on this well; backend behaviour is pinned by `run_hfu_ward_groups_core_plugs_by_fzi` (src-tauri\src\hfu.rs:458) |
| T-RT-11 | Pc Fit (Thomeer): per-plug fit, Hg standardization, Swanson-k suppression | step 4 judges G, Bv∞≤φ and Swanson k against the measured k on real MICP curves; standardization is pinned by `run_thomeer_standardizes_fluid_systems` (src-tauri\src\thomeer.rs:403) |
| T-RT-12 | SHF Fit — Cuddy FOIL with FWL scan | step 7 compares the scanned FWL against an independent DST/pressure-gradient contact; the scan mechanics are pinned by `fwl_scan_finds_the_true_contact` (src-tauri\src\shf_fit.rs:1213) |
| T-RT-13 | SHF Fit — Brooks-Corey and Skelt-Harrison forms | acceptance is "Swirr plausible for Mahakam shaly sand" and whether the overlay tracks the real cloud; the fits themselves are pinned by `brooks_corey_recovers_synthetic_curve` (src-tauri\src\shf_fit.rs:1228) and `skelt_fits_synthetic_curve_well` (src-tauri\src\shf_fit.rs:1243) |
| T-RT-15 | Facies Tie-in: RT_LOG vs reference RT confusion matrix + purity | step 5 asks whether the best reference class maps to RT_LOG=1 rather than inverted — a geological read that decides whether RT_LOG may be trusted uncored; the matrix arithmetic is pinned by `confusion_tallies_and_scores_purity` (src-tauri\src\facies_tie.rs:215) |
| T-RT-17 | Legacy Multimin v1 run via a pre-existing saved chain: VOL_* + RECON_ERR | step 4–5 acceptance is "in a known clean wet sand VOL_CLAY small and SWT_MM ≈ 1" and RECON_ERR elevated across coals/heavy-mineral streaks — real rock; see the ambiguity note, this test may simply be Blocked |
| T-BATCH-02 | Compose and run the 4-step chain across 5+ wells with live progress | acceptance is a petrophysical read of VSH/PHIE/SWE/PERM against real logs (VSH high in shale, PERM 0.1–1000s mD in sand) plus an app-responsiveness feel; step ordering is pinned by `chain_runs_steps_in_order_and_completes` (src-tauri\src\chain.rs:273) |
| T-MLEQ-10 | ML regression: predict DT from GR/RHOB/NPHI (+ leaderboard) | acceptance is whether DT_SYN overlays real DT within ~±10 µs/ft and whether it is petrophysically plausible on the DT-less well — a rock read; the leaderboard mechanics are pinned by `eval_leaderboard_blind_well_cv_ranks_linear_top` (src-tauri\src\ml.rs:2067) |
| T-IMP-07 | Core CSV import: plugs off the log grid overlay at native depths | the parse/route half is pinned, but the pass criterion is a human comparing plug CPOR against log PHIT (within 2–3 p.u.) and CGD ~2.6–2.7 on real Mahakam core |

---

## Twenty-five things the triage found that are worth fixing regardless

These came out of reading all 250 tests against the current code. Each was verified directly,
not taken on a subagent's word. **Findings 1, 2 and 3 have since been fixed — see the notes
under each.** A fourth of the same shape was found later and is recorded as finding 5.

### 1. The plan still teaches a client calibration that no longer ships — **FIXED 2026-07-31**

`manual_test_plan.md` lines 1470, 1472 and 4723 tell the tester to keep
`GR_LOW_REF 53.68 / GR_HIGH_REF 133.93 — the Rokan 562-well calibration` and to expect GRN's P3
to land on 53.68.

The shipped defaults are **20 / 120** (`modules.rs:2443`). The provenance sweep changed them, and
`gr_normalize_reference_defaults_are_generic_not_a_field_calibration` (`modules.rs:4042`) now
asserts they stay generic — its comment says it outright: *a two-decimal reference is a regression
result*. A stale comment at `workflow.rs:1625` says 53.68/133.93 too.

**Two problems, not one.** A tester following the plan will run on 20/120, see P3 land nowhere
near 53.68, and log a failure that is not one. And a client-fitted calibration named by field is
still sitting in a committed document as an instruction — which is the exact thing
`CLAUDE.md`'s provenance discipline forbids in source. The sweep covered the code; it did not
cover this plan.

**Fixed 2026-07-31.** Both places in the plan now say 20/120 with a dated inline note, and the
stale `workflow.rs:1625` comment is corrected. Instruction text only — no mark was touched.

### 2. A known-issue line will make a passing feature look broken — **FIXED 2026-07-31**

T-MLEQ-14 carries: *"run_ml has no bad-hole/flag MASK support at all… Expect step 3 to Fail."*

It does have it, and two tests run in the gate proving it — `run_ml_mask_excludes_apply_samples`
(`ml.rs:1624`) and `run_ml_mask_excludes_training_outlier` (`ml.rs:1830`). The real remaining gap
is only the missing Mask picker in `mlDialog.ts`. As written, the plan tells you to log a working
backend as a known failure.

**Fixed 2026-07-31.** T-MLEQ-14's known-issue line now says the backend has MASK and names the
two tests, and points the failure at the missing dialog control instead. The distinction matters:
"the capability is absent" tells you not to trust ML over washout, "the picker is absent" tells
you to add a picker.

### 3. `export.rs` has zero tests — **FIXED 2026-07-31**

4.4 KB of export code, no test of any kind. T-IMP-15 and T-IMP-16 both landed in pile B because
of it — meaning LAS export is currently proven only by you clicking it.

**Fixed.** Two tests now: `export_writes_missing_as_null_and_carries_mixed_case_computed_curves`
and `an_exported_las_reimports_with_the_same_values`. Both pile-B items retired.

### 4. The plan cannot cover what shipped after it was written

It was generated 2026-07-22. **21 REVIEW.md sections postdate it.** Grepping the plan:
Workbook — 0 tests. Deck — 0. Core depth registration — 0. Grain size — 1. Petrography — 3.

So the 250 tests are not 250 out of 250. The petrography suite, the office deliverables, the
calibrations, the depth registration and the image tracks have **no systematic test coverage in
this plan at all** — they exist only as prose claims in `REVIEW.md`.

### 5. A second stale known-issue line — found and **FIXED 2026-07-31**

Found while writing T-PREP-13. It said the Round-4 fix for "module runs report ✓ success even
when every output sample is MISSING" was *"uncommitted, in the working tree"* and told you to log
a green ✓ against that known finding rather than as new. It has been committed for some time and
is pinned by `all_nan_module_output_reports_error_not_success`.

**This is the same defect as findings 1 and 2, and three instances is a pattern worth naming.**
The plan was written at a moment when several fixes were in flight, and it froze that moment.
Every "known issue" line in it is a claim about the code that was true on 2026-07-22 and has been
decaying since — and each one costs you the same way, by telling you to accept a real failure or
to log a working feature as broken. The three found here were the ones the triage happened to
walk past. **Nothing has swept the rest.** If a step tells you to expect a failure, check the
claim before you believe it.

The corrected line also adds the case that makes T-PREP-13 subtle: if your gas flag covers only
part of the interval, the unflagged samples pass real densities through, the run is *not*
all-MISSING, and a green ✓ is then **correct**. Step 2 only holds where the flag covers
everything you ran.

**And one that was checked and is still TRUE.** T-PREP-16's known-issue line — the masked
washout — was verified against the current code rather than assumed stale like the other three.
The output masking in `workflow.rs` is still unconditional. It stands as written, and is now
pinned by a test. Three of four stale is not four of four; check, don't assume in either
direction.

### 6. Overriding a temperature gradient per zone makes a STEP, not a kink — **OPEN, your call**

Found while writing T-PREP-05, and it is the first thing in this whole exercise that changes
numbers rather than documentation.

`precalc` computes each sample as `SURF_TEMP + gradient(sample) × depth(sample)`. The gradient is
applied **from surface**, not integrated down through the zones above it. So the moment you give a
lower zone its own gradient, the temperature profile jumps at the boundary instead of bending.

With a 0.03 °C/m well and a 0.035 override below 1500 m, measured in the test:

| depth | FTEMP |
|---|---|
| 1400 m | 67.0 °C |
| 1500 m | **77.5 °C** |

A **10.5 °C step across 100 m**, where the undisturbed trend rises 3.0 °C. Rock temperature is
continuous — a 10 °C discontinuity at a formation top is not a thing the earth does. And it does
not stay in FTEMP: the Arps correction turns temperature into Rw, and Rw goes straight into Sw.

T-PREP-05's own expected result says the trend should **kink**, with *"no discontinuity
artifacts"*. So the plan and the code disagree, and the plan is describing the physical answer.

**Not fixed, deliberately.** Integrating per zone means deciding what temperature each zone
*starts* at — carry the previous zone's value down, or re-anchor on surface — and that is method
math with a cited source, not a refactor I should pick. The current behaviour is pinned exactly
as it is, with the step written into the assertion, so it cannot drift and cannot be changed
silently. Your call on which it should be.

### 7. A well with no permeability is EXEMPTED from the permeability cutoff — **OPEN, your call**

Found while writing T-BATCH-08, and like finding 6 it changes numbers rather than documentation.

`classify_sample` is emphatic about this at the sample level, and there is a confirmed `[x]` in
`REVIEW.md` for it: *a sample with missing PERM must FAIL an active PERM cutoff, not silently
pass.* But whether the cutoff is active at all is decided one line earlier, per WELL:

```rust
let has_perm_cut = req.perm_min.is_some() && perm.iter().any(|v| !v.is_nan());
```

A well carrying **no permeability anywhere** makes that false and switches the cutoff off for
itself. Measured in the test, two wells of identical rock, cutoff PERM ≥ 1000 mD:

| well | permeability | net pay |
|---|---|---|
| measured 1 mD | 1 mD | **0** — correctly excluded |
| measured none | — | **full** — cutoff never applied |

So the two halves of the same rule disagree, and in the damaging direction: **the less permeability
data a well has, the more pay it books.** Both wells report `n_classified > 0`, so nothing
downstream — not the Field Dashboard, not the workbook, not the client PDF — can tell the exempted
row from the honest one. In a field roll-up they simply add together.

T-BATCH-08's own Expected says the opposite of what the code does, so that test would have been
logged as a new failure; its instruction now carries a Known issue line pointing here.

**Not fixed, deliberately.** Whether an uncored well should be excluded from a permeability cutoff
or exempted from it is a petrophysical decision — exclusion is defensible (it cannot be shown to
pass) and so is exemption (a cutoff you have no data for should not silently delete a well) — and
either way it changes reserves. Pinned exactly as it is by
`a_well_with_no_perm_at_all_quietly_escapes_an_active_perm_cutoff`, with the asymmetry written into
the assertions so it cannot drift. Your call.

### 8. The Monte Carlo PERM cutoff — the audit's wording understates the trigger

Not a new finding: AUDIT-2026-07-21 already has it, and T-BATCH-16 carries the Known issue line.
What writing the test refined is **when** it fires.

The audit says the cutoff is ignored "whenever PERM is produced by the Monte Carlo chain itself
(not read from the DB)". The actual discriminator is `build_plans`'s external-input set: PERM
reaches `has_perm_cut` only if some step CONSUMES it and **no step produces it**. So the failure
is not a corner case of an unusual chain — it is triggered by *adding a permeability model*:

| chain | PERM cutoff |
|---|---|
| `vsh_gr → phi_dn → sw_indo → rocktyping` (reads PERM from the project) | works |
| the same chain with `perm_coates` inserted ahead of it | **silently dead** |

A study that models permeability is precisely the study whose permeability cutoff matters. Pinned
as-is by `adding_a_permeability_model_to_a_chain_switches_off_the_permeability_cutoff`, which shows
both chains side by side so the working case is the control.

I also asserted, wrongly, that no module reads PERM as an input — a grep for `log_in("PERM"` misses
`rocktyping.rs` and `satheight.rs`, which sit in their own files. Because it was written as an
assertion rather than a comment, the build rejected it immediately. Worth repeating as a habit:
**a claim about the codebase belongs in an assertion, where it can be wrong out loud.**

### 9. Pittman's r50 and r75 cross over in ordinary sand — **OPEN, needs the paper**

Found while writing T-RT-08. The physics is not in question: mercury enters the widest throats
first, so the radius quoted at 75 % saturation must be SMALLER than the one at 50 %. `PR75 < PR50`,
always, in rock.

The nine Pittman rows are nine **independent regressions**, not a nested family, so nothing in the
arithmetic makes them obey that. In log space,

```
PR50 − PR75 = −0.634 − 0.066·log k + 0.543·log φ%
```

which changes sign at about **79 mD at 25 % porosity**. Measured at φ = 25 %, k = 100 mD:

| | radius |
|---|---|
| PR50 | 2.907 µm |
| PR75 | **2.953 µm** ← larger, which cannot happen |

At 1 mD the same pair is the right way round, so this is not one bad sample — it is the
coefficients. PR10 through PR50 stay monotone throughout, which narrows it to the PR75 row (or,
less likely, PR50).

T-RT-08's Expected says the ordering "holds everywhere both curves are populated". It does not, and
the well where it fails is a good one. It also reaches the outputs: someone selecting `r75` as APEX
in fine rock — which is exactly what the module doc recommends for fine rock — gets RAPEX and
RT_PITT built on the inverted value.

**Not fixed, deliberately.** `pittman_rx_spec`'s own doc already says the full set is transcribed
from Pittman 1992 and flags it *verify before field release*. This is that verification, and it
fails — but correcting a published coefficient requires the paper in hand, and inventing one to
make the ordering come out right is exactly the move the provenance rules forbid. Pinned with the
measured numbers by `the_pittman_radius_family_inverts_between_r50_and_r75_in_good_sand`.

### 10. A run that fails still writes its empty curves into the catalog — **OPEN, your call**

Found while writing T-RT-05. `all_nan_module_output_reports_error_not_success` already pins the
honest half: an all-MISSING run reports an error and a zero sample count instead of a green ✓. What
nobody had checked is what reaches the database.

Phase 2 of `run_workflow_module_into` writes for any well whose outcome is `Computed` with a
non-empty output map — and an all-MISSING output map is still non-empty. So rocktyping on a well
with porosity but no permeability reports its failure **and versions the whole family** — RQI,
PHIZ, FZI, R35, PGEOM, PSTRUC, RT, PERM_RT — into the Curve Catalog as curves that are blank from
top to bottom. Measured: rows written for all eight, finite values in none.

T-RT-05's Expected says the catalog must gain no FZI/RT rows and no half-written curves. It gains
eight of them. The cost is not corruption — the values are honestly MISSING — it is that the
catalog stops distinguishing *"this was never run"* from *"this was run and could not answer"*, and
a log-set version is burned recording the second as though it were an interpretation.

**Not fixed.** Suppressing the write is a one-line filter, but it is a behaviour change for every
module (gascorr without precalc, electrofacies with no usable curve, an equation over a missing
input), and a blank curve is arguably the honest record that a run happened. Pinned as-is by
`rocktyping_without_a_permeability_curve_fails_and_writes_no_curves`, which asserts both halves —
the rows exist, and not one of them is finite.

### 11. The held legacy-multimin RECON_ERR item is already answered — **CLOSE IT**

`REVIEW.md` lists "legacy-multimin RECON_ERR at 3 tools" among the findings **awaiting your
sign-off because they would change interpretation numbers**. It does not need your sign-off. It
was resolved twice over while nobody updated the list, and T-RT-18 still instructs you to run a
module that refuses to start.

**First, the module is gone.** Legacy `multimin` is retired: `run_module` blocks it via
`retired_module`, the solver body was deleted, and the spec survives only so a saved chain
resolves by name and can show its stored parameters while you redo it in SandiMin. T-RT-18 step 3
gets a "use SandiMin" refusal, not a RECON_ERR.

**Second, the concern is real, it was inherited, and SandiMin already handles it.** The blindness
is not a bug anyone can fix — it is linear algebra. With as many equations as components the solve
reproduces the measurements exactly whatever the endpoints are, so the residual says nothing about
them. What SandiMin adds is that it *detects* the condition (`dof == 0`) and returns `dof_note`
saying RECON is forced to ~0 and telling the user to add an input log.

Measured on one well, one set of logs, one set of components, **correct endpoints throughout**:

| tools | dof | RECON |
|---|---|---|
| RHOB + NPHI (+ unity) | 0 | **~0.00** |
| + DT + GR | 2 | **0.62** |

The square figure is arithmetic, not fit quality. Add the 0.4 g/cc illite density error from the
existing endpoint test and the square case still reports ~0 while the clay volume moves materially;
at dof 2 the same error takes RECON from 0.62 to 1.22.

`dof_note_set_when_exactly_determined` already checked the note appears. It never checked the
reason it must be there, which is the whole of T-RT-18 — so
`an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so` now pins it,
with the over-determined run as the control.

**Nothing to decide, and nothing to fix.** What is left is bookkeeping: drop the item from
REVIEW.md's sign-off list, and T-RT-18's instruction now carries a superseded block pointing at
SandiMin. The one judgement worth making is a UI one — the note is returned, and whether the
SandiMin pane makes it hard to miss is worth a look during your click-through, because a warning
nobody reads is the same as no warning.

### 12. Two wells with one name overwrite each other's report, and the count says otherwise — **OPEN, your call**

`export_report_batch` names each file from the well NAME, sanitized (`report.rs:527`). `well_name`
carries no uniqueness constraint, so two wells can share one — and an import with attach OFF
creates a second record under the same name by design. The sanitizer widens the collision
further: every non-alphanumeric maps to `_`, so `SANDI/1` and `SANDI 1` land on one filename too.

When they collide the second write **silently overwrites the first**, and both paths are still
pushed onto `written` — so a 3-well batch reports "wrote 3 file(s)" over 2 files on disk, and the
report you keep is the last well's under the first well's name. Nothing in the status line, the
Processing panel or the folder says a well is missing. Pinned as-is by
`two_wells_with_one_name_silently_overwrite_each_others_report`.

The **same function identifies wells two different ways**, which is the root of it: the success
path looks up the name for the filename (`:518`), the failure path never does and reports the raw
UUID (`:535`). So a failure you can't attribute and a success that silently replaced a file are
the same underlying gap. Your plan already flagged the UUID half as UX feedback.

**Your call because it changes delivered filenames.** Suffixing the duplicate (`SANDI-1_report.pdf`,
`SANDI-1_2_report.pdf`) is the obvious fix; falling back to the well id is the other. Either
changes what lands in a client folder, which is not something to alter under you.

### 13. A script that raises on only SOME samples reports a clean success — **OPEN, your call**

A Rhai error is caught per sample and written as MISSING (`equations.rs:1116`,
`Ok(_) | Err(_) => NAN`). The only thing that converts a script error into a reported failure is
the all-MISSING guard at `:1124` — and it fires **only when every sample failed**.

So a script that raises on half the depths produces a curve with holes and reports success with
the full row count. Nothing tells the user their script threw. Worse, the result is
indistinguishable from a curve whose inputs were simply absent there, which is the ordinary
innocent case — so there is nothing on the log to prompt a second look. Pinned as-is by
`a_script_that_raises_on_only_some_samples_still_reports_a_clean_success`, whose control raises
everywhere and IS caught: the difference between reported and silent is only ever coverage.

Same shape as finding 10 (a failed run still writes its empty curves): in both, the honest signal
exists but is gated on the failure being total.

**Your call because it changes the run summary.** Counting the raises and reporting "N of M
samples failed" is the fix; whether that is a warning or an error — and whether a partially
failed curve should be written at all — is a judgement about how you use the equation editor. If
you make it, this test fails, which is the alarm.

Worth saying what this is NOT: the Python path is fine. `run_python_equation` runs the whole
well's array in one call, so a `raise` fails that well outright and the user's own message reaches
the summary — verified, not assumed, by
`a_python_raise_in_one_well_leaves_the_rest_of_the_batch_intact`. This is specific to Rhai's
per-sample evaluation.

### 14. T-ADV-13 tells you to expect a failure that was fixed — **PLAN IS STALE, no code change needed**

The step says the TVD dropdown on SW — Saturation-Height is a false affordance, that no producer
for a TVD curve exists anywhere in the app, and that you should **"Mark Fail — known"**.

That is no longer true. `ingest::materialize_tvd_curves` (`ingest.rs:469`) resamples the deviation
survey onto the well's log depth grid and writes TVD and TVDSS as fetchable curves, and
`import_deviation_csv` calls it on every import. So a deviated well with its survey loaded gets a
real TVD, `sw_height` consumes it, and HAFWL is measured from true vertical depth.

Verified end to end rather than inferred from the code:
`a_deviated_wells_height_is_measured_from_the_survey_not_along_hole` imports a survey (vertical to
1000 m, building to 60° by 2000 m), runs the module through `run_workflow_module`'s own input
resolution, and reads HAFWL back out of the database. It lands on FWL − TVD at every sample, and
at TD sits more than 500 m above the along-hole answer.

Worth noting **why the suite did not already say so**. Both halves had tests the whole time:
`sw_height_uses_tvd_and_allows_tvdss_fwl` hands the module a TVD array by hand, and
`deviation_import_materializes_tvd_tvdss_curves` checks the survey reaches the log grid. Neither
touches the joint between them, which is precisely where the finding lived. A suite organised by
file tests halves by default.

**Action is on the plan, not the code**: strike the Known-issue paragraph from the T-ADV-13 step so
you do not mark a passing feature as a failure out of deference to it.

### 15. The report's table pages carry no footer, unlike every other surface — **OPEN, your call**

T-REP-06 expects **"Each table page footer: 'Made in SandiBumi'"**. They have none.

The mark is emitted by the report COVER (`report.rs:289`), by every composite page
(`composite.rs:609`), by the Word document (`office.rs:827`) and by the PowerPoint deck
(`office.rs:1332`). But `table_pages` and `note_page` emit no footer at all — so the methodology,
zone-parameter and pay-summary pages of the PDF are the only unmarked surface in the whole
deliverable set. A reader who extracts or photocopies the pay summary gets an unattributed page.

Pinned as-is by `a_rendered_report_carries_the_plans_page_order_and_a_self_consistent_pay_table`,
which asserts the cover IS marked and no table page is. Everything else in that step checks out:
page order is cover → methodology → zone parameters → pay summary, `tables_only` genuinely stops
there, the zone override is listed by name and value, and the printed nets match the computed ones.

**Your call because it is a branding decision on a client document**, not arithmetic — whether the
mark belongs on every page or only on the cover is yours. Either way the plan and the code should
agree; today they do not. Fixing it fails this test, which is the alarm.

### 16. HPV is not guaranteed non-negative — a dense stringer is subtracted — **OPEN, your call**

T-REP-06 lists **HPV ≥ 0** as a domain check. It is not an invariant. The pay summary sums
`PHIE * (1 - SWE) * h` over net with no floor (`workflow.rs:717`), so the row inherits the sign of
PHIE.

The route is ordinary. A tight carbonate streak reads low GR, clears the VSH cutoff and is flagged
SAND; a density porosity computed on a sandstone matrix reads slightly NEGATIVE there, which is a
routine artefact of a vendor PHIE rather than a corrupt curve. Its contribution is then subtracted
from the SAND row's hydrocarbon column.

Measured, not asserted: `a_dense_stringer_is_subtracted_from_the_sand_rows_hpv` puts 2.5 m of
streak at PHIE = −0.05 through a 5 m zone and the SAND row's HPV comes back **more than 20% below**
the same well with the streak floored at zero.

Two things make it easy to miss. The streak fails the porosity cutoff, so **RESERVOIR and PAY are
byte-identical either way** — the two rows anyone checks first agree with each other while the SAND
row quietly does not. And the understatement is in the safe direction, so nothing looks alarming.
Flipping the printed sign takes a porosity far outside anything a log produces, which is why the
test claims the understatement and not a negative number.

**Your call because the fix has two candidate homes**: clamp PHIE at 0 where the porosity modules
write it, or floor the HPV contribution in the pay summary. Those are different statements about
whose job it is to reject a non-physical porosity, and the first changes curves you may want to
see unclamped for QC.

### 17. A chain whose worker thread dies jams the project switch for the rest of the session — **OPEN, your call**

The guard T-SHELL-09 exercises is correct: `chain::register` is called at `lib.rs:2428`, BEFORE
the worker thread is spawned at `:2468`, so the switch is already shut the instant Run is clicked
— there is no window where the command has returned and nothing is registered. Completing and
cancelling both release it. That is all pinned by
`a_registered_chain_holds_the_project_switch_shut_until_it_is_really_finished`.

What has no release is a worker that dies without reaching one of the three terminal `set_status`
calls. **Nothing ever removes an entry from the chain registry** — `register` inserts, `set_status`
mutates, and there is no prune (contrast `jobs.rs`, which prunes finished jobs and has a test for
it). So the job stays `Queued`/`Running` in the map forever and `any_active` keeps answering true.

`lib.rs:2466` already documents the case: a panic inside `run_chain` "stays on this thread (it
can't abort the process); the job simply stops reporting progress". It does more than stop
reporting. **Open Project, New Project and Compact Project are all refused from that moment on**,
each telling the user to wait for a job that will never finish, and the only way out is to restart
the app — which on a field project means paying the reopen cost again.

Pinned as-is by `a_chain_that_never_reports_a_terminal_status_jams_the_guard_permanently`.

**Your call because the fix is a judgement about failure semantics.** The mechanical part is easy:
`catch_unwind` around the `run_chain` call, setting `ChainStatus::Failed` — the variant already
exists and carries `#[allow(dead_code)]` precisely because nothing emits it. The judgement is what
the user should then be told, and whether a chain that died mid-write should let the project be
switched at all or should insist on a restart. Making the change fails this test, which is the
alarm.

### 18. The report cover states the composite's PRINT WINDOW, not the logged interval — **OPEN, your call**

The cover's "Interval: … – … m" is read straight off the composite pagination
(`report.rs:319` — `composite_pages.first().top` / `.last().bot`), and that pagination honours the
render's depth window. So setting a print window re-dates the whole report, **including the tables
the window never touched**: `run_pay_summary` works per zone and knows nothing about the composite
window. A report rendered over 1005–1010 m carries a pay table covering every zone in the well
under a cover announcing a 5 m interval. On a **tables-only** render there are no log pages left to
show the reader that the window was only ever a print setting.

Pinned as-is by `a_composite_depth_window_re_dates_a_cover_whose_tables_ignore_it` (`report.rs`),
which renders a 1005–1010 window over a well whose only zone is 1012–1019 and finds that zone
reported in full under the narrowed cover.

**The same line explains the audit's tables-only slowness, and constrains its fix.**
AUDIT-2026-07-21 (Viz/reporting #3) reads as though `report_pages` forgot an `if`: the composite is
rendered unconditionally at `:314` and only skipped when appending at `:463`. It did not forget.
The comment at `:312` says why — "Composite pages (also gives the true interval for the cover)" —
so the expensive render is what supplies the cover's one remaining fact. Skip it naively and the
cover falls to the `unwrap_or(0.0)` default and prints **"Interval: 0.0 – 0.0 m"** on a client
deliverable.

**Your call, and both halves want the same fix**: give the cover its own cheap logged-interval
query (`MIN`/`MAX` depth), which makes tables-only genuinely fast AND lets a print window be stated
separately from the interval the study covers. Whether the cover should then name the window at all
is a document-design decision, which is why this is yours. Making the change fails that test, which
is the alarm. The correct behaviour of tables-only otherwise is pinned by
`tables_only_drops_the_composite_pages_and_still_dates_the_cover_to_real_rock`.

### 19. Curve Edit's "coerce to 0.0" is HALF fixed, and the surviving half is one line of TypeScript — **OPEN, your call**

The BACKEND guard is correct and tested: `apply_op` refuses a non-finite constant outright
(`curve_edit.rs:417`), writing nothing — pinned by
`a_set_constant_refuses_a_value_that_is_not_a_number`. It is also unreachable for the case the
audit reported.

`curveEditDialog.ts:88` reads every numeric field through
`const v = parseFloat(s); return Number.isFinite(v) ? v : dflt;`. An empty Value field, or `abc`,
gives NaN, which is not finite, so the helper returns its default of **0** — a perfectly finite
number that passes the backend guard and is written over the interval as a real reading. The
comment above that line shows the narrowing was deliberate and stopped one step short: it was
added so `1e999` could not set a curve to +Inf and poison catalog min/max and plot autoscale. It
fixed the Infinity half. `1e999` now writes **0.0** instead, which is the audit's own finding
arriving by the new route.

**The sharp version: 0 is the identity for every field where it is the default except this one.**
An empty `add` falls back to 0 and an empty `mul` to 1 — both no-ops, which is why nobody noticed.
There is no identity for "set a constant", and 0.0 gAPI over an interval does not look like an
error; it looks like a measurement of very clean rock.

**Your call because the fix is a UI decision**: refuse Apply with a hint while the field is empty
or unparseable, or pass the non-finite value through and let the backend's existing refusal
surface. The second is one character (`dflt` to `NaN`, for `value` only) and gives a worse message.

### 20. The Wells grid's editor has no 0-row check, unlike the other three — **OPEN, your call**

`update_standard_sample`, `update_computed_sample` and `update_core_sample` all check the UPDATE's
row count and return an error naming the depth when nothing matched — the fix for the audit's
"DB-inspector edit reports success on a 0-row update", and now pinned by
`an_inspector_edit_on_a_row_that_moved_fails_instead_of_reporting_success`.

`update_well_field` (`db.rs:5140`) does not. It validates the COLUMN and then runs the UPDATE
without checking that anything matched, so an edit against a well that is no longer there returns
`Ok`. The route is the Wells grid left open while the well is deleted in the Wells & Tops pane:
the cell shows the new value, the status bar reports the edit, and an undo entry is pushed for a
change that never happened.

Rarer than a moved curve sample — a well_id does not drift the way a depth does — and the same
silent outcome. Pinned as-is in the test above.

**Your call, though this one is nearly mechanical**: the `n == 0` check the other three already
carry, with a message naming the well rather than a depth.

### 21. T-PETRO-02's Larionov labels are reversed, and the dropdown gives no rock age — **OPEN, and this one is worth reading**

The CODE is right. `modules.rs:349-350`:

- `LARINOV1` = `0.33 * (2^(2*IGR) - 1)` — Larionov (1969) for **older rocks / Mesozoic and older**. At IGR 0.5 it gives **0.330**.
- `LARINOV2` = `0.083 * (2^(3.7*IGR) - 1)` — Larionov (1969) for **Tertiary / unconsolidated**. At IGR 0.5 it gives **0.216**.

Those are the published coefficient sets, and they match the numbers in this document's own pile-B
row. The manual plan has them the other way round: step 1 reads "change `OPT_GR` to **LARINOV1**
(Larionov Tertiary)", and its Expected pairs "Larionov-Tertiary ≈0.33, Larionov-older ≈0.22". Both
associations are backwards relative to what the code computes.

**Why this is the one worth reading.** Mahakam Delta is Miocene deltaic — Tertiary — so the
transform this work usually wants is `LARINOV2`. Selecting `LARINOV1` on the plan's label returns
0.33 where 0.216 belongs: a shale volume more than half again too high through the whole
intermediate-GR interval, which is exactly where the VSH cutoff decides net pay. The curve looks
entirely normal, both endpoints are fine, and nothing downstream can catch it.

**The dropdown cannot settle it either.** `OPT_GR`'s choices are the bare strings `LARINOV1`,
`LARINOV2`, `LARINOV3`, `STIEBER1..3` — no rock age, no coefficient, no tooltip. The plan is the
only place a user is told which is which, and it is wrong.

Now pinned by `every_vsh_gr_transform_lands_on_its_published_coefficient`, which evaluates all
eight transforms by hand at IGR 0.5 and asserts each lands on its published closed form, so the
mapping cannot drift again without failing.

**Two calls, and they are separable.** Correcting the plan text is free. Labelling the dropdown
(`LARINOV1 — Larionov, Mesozoic and older`, `LARINOV2 — Larionov, Tertiary`) is a small UI change
that would make the option self-describing — but the option IDs are stored in `params_json` on
every saved run, so the ids themselves must not be renamed.

**A second correction to the same Expected line.** It says "endpoints 0 and 1 unchanged". At pure
shale that is true for LINEAR, all three Stieber forms and Clavier (which cancels exactly), but
the Larionov forms are empirical fits that were never normalised to close at 1: `LARINOV1` stops
at **0.99**, `LARINOV2` at **0.9957**, and `LARINOV3` overshoots to **1.133**. `VSH` clamps all of
them to 0–1; `VSH_GR` keeps the raw value, which is what that pair of outputs is for. Not a defect
— but read against the plan as written it looks like one.

### 22. The end-to-end harness was driving a dev server, not the built app — **FIXED 2026-08-01**

The harness's opening line claims it drives "the REAL BUILT DESKTOP APP", and that claim was
false for as long as the harness has existed.

`src-tauri/target/release/sandibumi.exe` can be produced two ways. `npm run tauri build` embeds
`../dist` into the binary; a bare `cargo build --release` compiles exactly the same Rust and bakes
in `tauri.conf.json`'s **`devUrl`** instead, so the webview loads `http://localhost:1420`. The
second binary is not distinguishable from the first by size, by name, or by anything the harness
looked at.

**With a Vite dev server up it passes every test.** That is the whole problem: the run is green,
the assertions are real, and what was actually driven is the dev server's frontend — a different
build of the frontend from the one in the binary, with none of the packaged app's CSP or asset
pipeline. Everything the harness is FOR is the difference between those two.

It surfaced only because the dev server happened to stop between one run and the next, at which
point the webview landed on `chrome-error://chromewebdata/` and every test failed at once. Until
then it had passed 10 of 10 twice that morning. A harness that silently tests the wrong artefact
is worse than no harness, and this one gave no signal in either direction.

**The fix is a refusal, not a rebuild.** The `before` hook now reads `location.href` and aborts
unless the app is serving its own embedded frontend (`tauri://` / `http://tauri.localhost`),
naming the cause and the correct build command. A dev-pointing binary can no longer produce a
green run, and — deliberately — the message tells the reader NOT to fix it by starting a dev
server. `e2e/run.mjs`'s "no binary" message now names `npm run tauri build -- --no-bundle` for the
same reason: a bare cargo build is the trap, so the instruction has to rule it out explicitly.

**One thing to re-check, which this casts doubt on.** The pile C note for **T-SHIP-01** ("packaged
app launches under the hardened CSP") records it as *already machine-verified once on 2026-07-29
by exactly this route*. The CSP exists only in a packaged build, so if that verification ran
against a dev-pointing binary it verified nothing. Which binary it used is not recorded, so this
is a doubt rather than a finding — but T-SHIP-01 is cheap to re-run now that the harness refuses
the wrong artefact.

### 23. An ordinary SQL comment breaks the read-only console, two different ways — **STARTER FIXED 2026-08-01, the guard is your call**

The SQL console mishandles `--` comments at BOTH ends of a query, and neither failure is the
user's fault.

**A leading comment is refused.** `db::run_readonly_query` decides whether a query is a read by
lower-casing the trimmed text and testing whether it **starts with** `select` or `with`. A `--`
line in front hides the keyword, so a perfectly valid SELECT comes back *"only SELECT queries are
allowed here"*.

That is what shipped the panel's own starter query broken: it opened with two comment lines naming
the project's tables, so **the first thing a new user clicked in that panel was refused, with a
message telling them their SELECT was not a SELECT.**

**A trailing comment corrupts the query.** The console executes what you typed WRAPPED:

```
SELECT * FROM ({your sql}) __sandibumi_q LIMIT n
```

so a `--` at the end swallows the closing paren and the limit, and DuckDB reports *"syntax error at
end of input"* — against a query that is valid on its own. This is the more confusing half: nothing
on screen says the query was rewritten before it ran.

Both were found by the end-to-end harness, which runs the starter through the pane's own Run
button. Neither is reachable by a Rust test: the guard is pinned by
`readonly_query_refuses_every_write_shape_including_a_cte_prefix` and behaves correctly by its own
definition, the wrapper is an implementation detail no test inspects, and the starter is frontend
text nothing was checking.

**Fixed here:** the starter now begins with `SELECT` and carries its explanation as a closed
`/* … */` block comment, which is safe at both ends. Frontend text only; the write discipline is
untouched.

**NOT fixed, and it is your call**, because both fixes touch a write-discipline path and rule 6
puts that in your hands:

- The leading case wants the guard to skip leading `--` lines and blank lines before testing the
  first keyword. Note this makes it **stricter**, not looser — it would then test the first REAL
  token, and `-- x⏎DELETE …` is rejected either way.
- The trailing case wants the wrapper to put its suffix on a NEW LINE (`…
) __sandibumi_q LIMIT n`),
  which costs nothing and fixes it outright.

Current behaviour is pinned as-is in `panels.e2e.mjs`, with instructions to delete whichever half
gets fixed rather than restoring it.

### 24. T-MLEQ-14's Mask note is stale a SECOND time — **PLAN IS STALE, no code change needed**

Step 3 of T-MLEQ-14 tells you to search the ML pane for a mask picker, expects not to find one,
explains that flagged washout samples therefore "silently bias the scaler, cluster centers, trained
models and PCs", and instructs you to log it against the dialog.

**The control is there.** `mlDialog.ts` builds a `maskSel` and adds a **"Mask (exclude)"** form row,
with a comment saying it is kept visible for ALL tasks because it also governs the unsupervised fit
pool.

This note has now been wrong twice in different ways. It originally said the BACKEND had no mask
support at all; that was corrected on 2026-07-31 when `run_ml_mask_excludes_apply_samples` and
`run_ml_mask_excludes_training_outlier` turned out to pin exactly that. The correction left behind
"what is still missing is only the Mask picker in `mlDialog.ts`" — which is now also untrue.

The cost is the usual one for a stale plan line, and it is not small: a tester following it looks
for a control, finds it, and has to decide whether the plan or their own eyes are right. Worse, the
note tells them what conclusion to draw ("log it against the dialog"), so the likeliest outcome is a
defect filed against working code.

**Fix: correct step 3's Expected and delete the known-issue note.** No code change. Now pinned from
the other side by `ml.e2e.mjs`'s "has a Mask control" test, which goes red the day the control is
removed — the failure mode the note was worried about, caught properly rather than described.

### 25. T-IMP-05 is marked **Fail**, and the behaviour it failed on has since been fixed — **PLAN IS STALE, and worth your attention because your own mark is on it**

`manual_test_plan.md` T-IMP-05 carries **`[x] Fail`** — your mark, from clicking through the
no-well-selected guards. Its Expected reads: *every tool refuses with status `Select a well first
(Wells & Tops panel)` — no dialog opens*.

**Both halves of that sentence are now wrong, and the second one is wrong on purpose.**
`src/ui/needWell.ts` (added 2026-07-31, after your mark) replaced the quiet status line with a
NAMED REFUSAL DIALOG. Its own header explains why in terms that read like the complaint that
produced your Fail:

> A status-bar line is the wrong place to refuse a click. The user picked "Import SCAL…" and
> expected a file dialog; what they got was nothing, with the reason in a corner of the window
> nobody was looking at.

So the step now: shows a modal naming the action and telling you to pick a well, AND still writes
to the status line — because the message belongs in the record of what was attempted, it just
cannot be the only place it appears. The wording is
`"<action> needs a well — select one in the Wells & Tops pane"`, not the string the plan quotes.
The callers are exactly T-IMP-05's list: Export LAS, Import DLIS, Import SCAL, Import deviation,
Import Aux, Import pictures, Data Sets, Shift Core, Well header.

**Two things to do.** Correct T-IMP-05's Expected — "no dialog opens" must become "a named refusal
dialog opens" — and then re-run it, because **the item is very likely a Pass now and your Fail is
the only record saying otherwise.** Nothing else in the plan tracks that a marked item was fixed
afterwards, which is precisely how a fixed defect stays on the books.

**Not covered by the harness**, and the reason is worth stating: driving it needs the app in a
no-well-selected state, and nothing reachable from the DOM clears `appState.selectedWell` once a
well has been clicked. Every other spec selects a well by design. A test-only "clear selection"
path would be a change to the product to serve the tests — the same decision the harness declined
over `driverProvider: 'embedded'` — so it is left to you.

---

## What to do with this

**Work `manual_test_plan.md` for the mature app, and read the top 21 sections of `REVIEW.md`
for everything newer.** They are a snapshot and a running log, not two versions of one thing.

Then, in rough value order:

1. **Fix the three stale spots above** — they cost you wrong results before they cost anything else.
2. **Write pile B** (45 tests). Start with T-REP-18, T-SHIP-03 and T-INT-11; they are silent-wrongness class.
3. **Regenerate the plan** so it absorbs the 21 post-07-22 REVIEW.md sections as new, unticked tests.
4. **Then decide on the harness.** 86 tests are automatable today. Whether that is worth building
   is a judgement about your time, and it should be made with that number rather than with 147.

Pile D — **37 tests** — is yours either way, and always will be.
