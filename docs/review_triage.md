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
| **B** — a Rust test now checks it | **6** | 39 |
| **C** — a machine now drives it | **5** | 81 unblocked (+61 blocked) |

### Pile B — retired so far

| Manual test | Now checked by | Where |
|---|---|---|
| T-REP-18 | `readonly_query_refuses_every_write_shape_including_a_cte_prefix` | `db.rs` |
| T-SHIP-03 | `a_missing_curve_fails_by_name_rather_than_computing_on_another` | `lorenz.rs` |
| T-IMP-15 | `export_writes_missing_as_null_and_carries_mixed_case_computed_curves` | `export.rs` |
| T-IMP-16 | `an_exported_las_reimports_with_the_same_values` | `export.rs` |
| T-INT-03 | `zones_from_tops_are_contiguous_and_absent_tops_make_no_zones` (+ the inverted-zone guard) | `db.rs` |
| T-INT-11 | `a_restored_log_set_version_feeds_the_next_module_run` | `workflow.rs` |

All three items flagged as **silent-wrongness class** (T-REP-18, T-SHIP-03, T-INT-11) are closed.

### Pile C — covered by the end-to-end harness

`npm run test:e2e` (see `docs/e2e_harness.md`) drives the built app: a sandboxed project, a real
LAS import, a real `vsh_gr` run with the curves read back, a real LAS export, and a frontend-boot
check. Optional; never part of the green gate.

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

## Four things the triage found that are worth fixing regardless

These came out of reading all 250 tests against the current code. Each was verified directly,
not taken on a subagent's word.

### 1. The plan still teaches a client calibration that no longer ships

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

### 2. A known-issue line will make a passing feature look broken

T-MLEQ-14 carries: *"run_ml has no bad-hole/flag MASK support at all… Expect step 3 to Fail."*

It does have it, and two tests run in the gate proving it — `run_ml_mask_excludes_apply_samples`
(`ml.rs:1624`) and `run_ml_mask_excludes_training_outlier` (`ml.rs:1830`). The real remaining gap
is only the missing Mask picker in `mlDialog.ts`. As written, the plan tells you to log a working
backend as a known failure.

### 3. `export.rs` has zero tests

4.4 KB of export code, no test of any kind. T-IMP-15 and T-IMP-16 both landed in pile B because
of it — meaning LAS export is currently proven only by you clicking it.

### 4. The plan cannot cover what shipped after it was written

It was generated 2026-07-22. **21 REVIEW.md sections postdate it.** Grepping the plan:
Workbook — 0 tests. Deck — 0. Core depth registration — 0. Grain size — 1. Petrography — 3.

So the 250 tests are not 250 out of 250. The petrography suite, the office deliverables, the
calibrations, the depth registration and the image tracks have **no systematic test coverage in
this plan at all** — they exist only as prose claims in `REVIEW.md`.

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
