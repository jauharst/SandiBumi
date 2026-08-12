# SB-POR Pilot Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development for each
> production increment and superpowers:verification-before-completion before every commit. Execute
> serially in the primary session. Do not delegate petrophysical math, parameter custody, write
> discipline, or final verification.

**Goal:** Close or explicitly block all 44 current `SB-POR` pilot-blocker dispositions through a
serial Gate-2 remediation program, while preserving every cited/absent parameter boundary, making
every correction and limit observable, and leaving Jauhar's manual desktop review honestly open.

**Architecture:** Introduce one Rust porosity contract layer consumed by density, density-neutron,
sonic, hydrocarbon, excavation, SSC and SSPW paths. The common layer owns typed method identity,
quantity family, output role, run provenance, user-selected output names, branch/limit flags,
canonical units and shared invariants; each selected method continues to own its source-bound
physics, numerical validity rules and correction limits. Existing `log_sets.params_json` and
`inputs_json` remain the versioned run record, and existing delete-then-append current writes plus
append-only archive remain the only write path. No database schema or `db.rs` change is planned.

**Tech Stack:** Rust, Tauri, TypeScript, DuckDB through existing whitelisted commands, manifest-
generated dock panes, `node:test`, Cargo tests, `bytemuck` IPC where arrays cross the bridge, and the
existing PowerShell full gate.

## Global Constraints

- This file is a banked Gate-2 plan. Gate 1 domain adjudication continues first. Do not execute a
  production task from this plan without a separate explicit authorization.
- Work only in `D:\XX. SandiBumi`, on the branch already assigned for the execution increment.
  Never use the empty `D:\XX. SandiBumi-check` folder as evidence or a worktree.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of
  `docs/PRD_v2/11_porosity.md`, `docs/method_sonic_porosity.md`, the applicable
  `docs/record_*.md`, and this complete plan before production work.
- The codebase-index server was not callable while this plan was written. Reindex and use it first
  if available in the execution task; otherwise state the targeted-filesystem fallback and confirm
  consequential Rust negatives with `rg` because Rust macro/`impl` indexing is not authoritative.
- `docs/PRD_v2/**` is immutable during implementation. A source contradiction is a blocker, not
  permission to amend the specification in a code lane.
- Missing continuous samples remain `f32::NAN`, never `Option<f32>`. A refusal occurs before a
  version or curve is written; a per-sample inability produces MISSING plus the specified flag.
- Raw arrays cross Tauri IPC only as `bytemuck` bytes. Frontend code never sends SQL for a write.
- Do not edit `db.rs` and do not add `ON CONFLICT`, upsert, a uniqueness constraint, or any other
  duplicate-tolerant path to deliberately PK-less `computed_curves`. Reuse the existing versioned
  delete-then-append writer and append-only archive.
- Every text import, if any later becomes necessary, goes through `parsers::read_text_file`. Every
  Python runner, if any later becomes necessary, reads `sys.stdin.buffer`. This plan requires
  neither text import nor Python.
- A parameter is cited or absent. Do not promote current literals, fixtures, vendor neighbors,
  ranges, averages, rounded values, project precedents or the implementation's own output into a
  default. A method with a required absent value refuses and names the missing decision.
- `DEC-012` through `DEC-017` are binding. In particular: `Cp < 1` refuses; output names are user-
  configurable and replacement is explicit/versioned/undoable; arithmetic comparison, RMS
  comparison, SSC/SSPW RMS conditioning, Gaymard-Poupon response and coupled `Sxo`/`Sw` iteration
  are separate contracts; common POR custody never creates universal physics bounds; analytic N-D,
  HC response, excavation and neutron-sonic belong in the product; genuine RHG means the original
  three-segment RHG80 route, not the current approximation.
- `docs/method_sonic_porosity.md` explicitly warns that its scanned-equation transcription is not
  implementation authority. No RHG80 equation may enter Rust until the original scan has been read
  at sufficient resolution, the exact symbols and page citations are recorded, and an independent
  oracle is derived from the paper.
- `SB-POR-020` remains a separate decision: `DEC-017` chooses the original-paper RHG80 route but
  does not nominate IP, Geolog or Techlog as the authoritative vendor rendering. Do not mark
  `SB-POR-020` complete until that separate choice is explicit.
- ESC-POR-8 keeps the nine Bateman-Konen constants non-adoptable. The analytic N-D method does not
  ship until admissible source custody closes that gate. The existing average/RMS curves may be
  retyped as comparisons without waiting for those constants.
- ESC-1, ESC-2, ESC-3, ESC-5 and ESC-7 remain hard boundaries. Do not open a protected binary,
  transcribe vendor chart values, or infer a missing validity endpoint. A blocked requirement is a
  successful truthful result.
- No client, field, block, basin, operator, project or well name appears in code, fixtures, tests,
  comments, Help text or screenshots. Name the physical condition.
- One named correctness test owns each atomic contract below. The name is the sentence it pins.
  Each expected value cites the PRD row, primary source, or independently shown arithmetic. A test
  whose expected value is current behavior is named `characterizes_...` and never closes a
  correctness requirement.
- Do not mark a failing test ignored. Optional-package tests are the only ordinary ignore case, and
  no increment in this plan needs an optional package.
- Every implementation increment follows RED -> minimal GREEN -> focused refactor. Mutation-prove
  the lazy or historically wrong alternative described by the chapter, restore it, then run the
  named test and relevant neighbors.
- Each increment adds one unchecked `REVIEW.md` entry at the top for Jauhar's manual desktop pass.
  Automated DOM or Tauri tests never check that box and never count as field evidence.
- Run `npx tsc --noEmit`, then `cargo check` in `src-tauri`, then
  `powershell -ExecutionPolicy Bypass -File tools\check.ps1` before every implementation commit.
  Require zero failures; report the exact passed/failed/ignored totals.
- One commit per serial increment with the increment ID and covered `SB-POR` IDs in the message.
  Do not push, merge, open a pull request, or begin the next increment automatically.

---

## Serial Increment Map

| Increment | Atomic scope | Requirements | Production gate |
|---|---|---|---|
| `POR-I0` | source and independent-oracle readiness | source gates for `014`, `020`, `021`, `029`-`039` | docs/tests only; no formula enters production |
| `POR-I1` | common POR types, outputs, provenance, flags and UI | `001`-`004`, `006`, `007`, `009`, `010`, `054`-`057` | every method visible under one contract without universal numeric bounds |
| `POR-I2` | shared endpoint custody and clay-bound-water identity | `008`, `011`, `055`, `056` follow-through | one cited/absent parameter surface and one `PHIT_SH` helper |
| `POR-I3` | sonic truthfulness, conventions, compaction and RHG80 | `013`-`018`, `020` | source-ready subset only; unresolved vendor rendering remains blocked |
| `POR-I4` | analytic N-D, basis refusal, comparisons and clamps | `021`, `023`, `024`, `028` | ESC-POR-8 must close for analytic method; comparison cleanup can proceed |
| `POR-I5` | HC response and coupled porosity-`Sxo`/`Sw` solve | `029`-`035`, `038`, `050`-`052` | every absent input refuses; no partial iterate survives |
| `POR-I6` | additive excavation correction | `039` | primary-source/cited-choice gate; multiplied form unreachable |
| `POR-I7` | limiting and conditioning integration | `043`, `045`-`048` | every branch observable; PHIE-floor conflict explicitly adjudicated first |
| `POR-I8` | SSC/SSPW honesty and gas parity | `058`, `059` | no dead controls; identical cited RMS conditioning |
| `POR-I9` | integrated workflow, pay exclusion, UI acceptance and live re-adjudication | all 44 blocker rows | full gate plus Jauhar-owned manual evidence remains open |

The map is deliberately serial. `modules.rs`, `workflow.rs`, `curves.rs`, `ipc.ts`,
`moduleDialog.ts`, the curve catalog and the run record are shared seams; parallel POR branches
would create mechanically clean merges with incompatible flag, provenance and output-role models.

---

## POR-I0: Establish Source and Oracle Readiness

**Files:**

- Modify: `docs/takeover/evidence/sb-por.md`
- Create when source custody is available: `docs/method_porosity_oracles.md`
- Read: the locally controlled original RHG80 scan, Poupon 1971, the admissible Bateman-Konen
  source, and any acquired ESC-1/ESC-2 source
- Test only: `src-tauri/src/porosity.rs` after the source note is independently verified

**Interfaces:** No production interface. This increment creates the evidence boundary later tests
will cite.

- [ ] Locate the original Raymer-Hunt-Gardner 1980 scan supplied by Jauhar. Render the pages that
      contain all three segments, both breakpoints, density/velocity/slowness definitions and any
      stated fluid assumptions. Verify exact subscripts, squares, roots and inequality ownership.
- [ ] Record the exact page citations and independently derive forward and inverse control values
      at one point in each segment plus exactly `phi = 0.37` and `phi = 0.47`. The boundary checks
      must prove continuity from both sides.
- [ ] If the scan is unavailable or typography remains ambiguous, record `SB-POR-014`'s true-RHG80
      production part as BLOCKED and continue with the independently source-ready sonic guards.
      Never code from `docs/method_sonic_porosity.md` alone.
- [ ] Keep the three F3 vendor renderings separate from the original paper. Ask Jauhar for the
      separate `SB-POR-020` authoritative-vendor decision only after presenting their exact source,
      convention and validity differences. Until then, no vendor rendering is the default.
- [ ] Reverify Poupon A-5/A-6/A-8/A-9/A-10 expectations used by T01-T03 directly against the held
      primary source. Record the page/equation citations and the independent arithmetic vectors.
- [ ] Confirm whether ESC-2 supplies a validity range. If it does not, the model-specific bound
      remains absent; stoichiometry may guard only the quantity it independently constrains.
- [ ] Confirm ESC-POR-8 status. If the nine Bateman-Konen constants still lack admissible custody,
      keep `SB-POR-021` blocked and prohibit copying them from the vendor executable/chart.
- [ ] Confirm ESC-1 status before making the excavation exponent authoritative. If still open,
      expose only the chapter-admissible cited choices with no silent default.
- [ ] Run `git diff --check` and the takeover-ledger tests. Commit only the source receipt as
      `POR-I0 record POR source and oracle readiness`.

---

## POR-I1: Build the Common POR Contract and Observable UI

**Files:**

- Create: `src-tauri/src/porosity.rs`
- Create: `src-tauri/src/porosity_contract_tests.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/curves.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src-tauri/src/equations.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src/ipc.ts`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `src/ui/workflowDialog.ts`
- Modify: `src/ui/wellParamsDialog.ts`
- Modify: `src/styles.css`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Interfaces:**

- Add Rust `PorosityMethodId`, `PorosityVolumeConvention`, `PorosityOutputRole`,
  `PorosityQuantity`, `PorosityFlagSet` and `PorosityRunMeta` types in `porosity.rs`.
- Keep method identity and convention serialized as stable strings in the existing versioned run
  JSON. Keep per-sample flags as exact integer-valued `f32` output samples so the continuous-log
  missing contract remains `f32::NAN` and no new IPC array representation is invented.
- Add manifest metadata sufficient to declare POR method, quantity, output role, volume input type,
  source topic and active/inactive state without hard-coding module names in the frontend.
- Extend curve-catalog entries with imported/computed origin plus POR quantity/method/output-role
  metadata derived from the run record; do not add a schema or write path.
- Reuse `workflow::resolve_output_names`. Distinct user-selected names preserve parallel results;
  an intentional same-name run remains one explicit versioned replacement under `DEC-013`.

**Owned acceptance tests:**

- `every_porosity_method_uses_one_common_contract_without_borrowing_another_methods_limits`
  (`SB-POR-001`, `DEC-015`).
- `every_porosity_method_emits_distinct_unlimited_and_limited_pairs` (`SB-POR-002`).
- `every_porosity_sample_records_the_branch_and_each_limit_that_bound_it` (`SB-POR-003`).
- `porosity_curves_carry_family_method_convention_and_noncolliding_user_chosen_names`
  (`SB-POR-004`, chapter T31/T32, `DEC-013`).
- `a_porosity_method_refuses_an_untyped_shale_or_clay_volume` (`SB-POR-006`).
- `every_porosity_parameter_shows_its_source_and_tier_at_the_point_of_choice` (`SB-POR-007`).
- `limited_total_porosity_is_rebuilt_after_effective_porosity_is_limited` (`SB-POR-009`, T11).
- `a_saved_porosity_run_can_be_rederived_from_method_parameters_and_input_identities`
  (`SB-POR-010`).
- `canonical_porosity_transform_signs_match_both_inverted_vendor_notations` (`SB-POR-054`).
- `porosity_density_and_transit_time_are_canonical_inside_the_engine_and_convert_only_at_the_boundary`
  (`SB-POR-056`, T09).
- `comparison_porosity_curves_are_visually_distinct_and_excluded_from_pay_by_default`
  (`SB-POR-057`, T36).

- [ ] Write the eleven named tests first. Use synthetic physical conditions and independent chapter
      arithmetic. Pin both sides of each lazy alternative: distinct limit paths without borrowed
      bounds, typed VSH accepted/untyped refused, imported/computed identities distinguishable,
      comparison excluded/primary included, user-distinct names preserved/intentional same-name
      replacement versioned.
- [ ] Add the common typed structs and stable serialization. Keep method-specific limit values out
      of the common type; it owns identity and observability, not physics.
- [ ] Extend manifest declarations for POR metadata and active/inactive parameters. Update all
      manifest consumers together so a module pane, saved workflow, well-parameter grid and restored
      pane show the same source, role and inactive state.
- [ ] Add a porosity family in `curves.rs` covering at minimum `PHIE`, `PHIT`, `PHIA`, `DPHI`,
      `NPHI_COR`, limited outputs and comparison outputs. Preserve exact mnemonic identity within a
      family so family resolution never makes two methods interchangeable.
- [ ] Add one framework-owned POR flag output per POR method. Define stable software bit meanings in
      `porosity.rs` for floor, ceiling, high-shale, bad-hole, coal, tight, conditioning, crossover,
      forced-wet, model-validity and non-convergence. These are identifiers, not physical limits;
      no bit assignment may encode or imply an uncited threshold.
- [ ] Serialize `PorosityRunMeta` beside existing parameters and inputs in the versioned run JSON.
      Prove save/list/restore preserves method, convention, roles, source topics, input identities,
      output names and flags without editing `db.rs`.
- [ ] Show source/tier, no-default state, method identity, volume convention, output role and flag
      legend in the docked module pane. A blank required parameter says “set a cited value” and
      refuses Run; it never renders a plausible placeholder.
- [ ] Give comparison outputs a distinct structural role and visible badge/style in curve pickers
      and POR pane summaries. Pay continues to consume an explicitly selected primary `PHIE`; a
      comparison curve is absent from default pay selection even if its numerical values are valid.
- [ ] Mutation-prove a borrowed universal clamp, an untyped volume acceptance, a lost run field and
      a comparison curve entering default pay. Restore each mutation and rerun the owned tests.
- [ ] Add one unchecked REVIEW entry covering the docked POR pane, source display, output naming,
      flag legend, version history and comparison visual treatment.
- [ ] Run the three mandatory gates and commit as
      `POR-I1 SB-POR-001-010 054-057 add common observable POR contract`.

---

## POR-I2: Unify Parameter Custody, Matrix Density and `PHIT_SH`

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/ssc.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `every_module_uses_the_same_formation_water_phit_sh_definition` (`SB-POR-008`, T15/T30).
- `a_chained_porosity_workflow_uses_one_recorded_matrix_density` (`SB-POR-011`, T35).
- `every_porosity_parameter_is_cited_or_absent_with_competing_values_visible` (`SB-POR-055`).

- [ ] Write the three named tests first. T15 independently derives `0.1667` from formation-water
      density and distinguishes the wrong `0.1573` fluid-density route. T30 drives every reachable
      `PHIT_SH` producer through one explicit parameter set. T35 runs the real documented chain.
- [ ] Move the `PHIT_SH = (RHO_DSH-RHO_SH)/(RHO_DSH-RHO_W)` arithmetic into one public POR helper
      and call it from density, D-N, SSC/SSPW and the CLY seam. Keep shale subtraction as a different
      named helper and quantity.
- [ ] Replace uncited numeric defaults for `RHO_SH`, `RHO_DSH`, `NPHI_SH`, `DT_SH` and `RHO_MA`
      with open required fields wherever section 5 says absent. Show all competing cited values and
      tiers through existing `param_sources`; do not select one for the user.
- [ ] Make a chained workflow share an explicitly selected `RHO_MA` value and record that selection
      at each stage. Do not create a global lithology-agnostic default or silently copy the first
      module's old literal.
- [ ] Mutation-prove the `RHO_FL` substitution, an SSPW-local duplicate helper, a missing source
      display and the old 2.65/2.645 chained mismatch. Restore and rerun.
- [ ] Add one unchecked REVIEW entry for competing-source display, required blanks and a chain run
      inspected through version history.
- [ ] Run the mandatory gates and commit as
      `POR-I2 SB-POR-008 011 055 unify POR endpoint custody`.

---

## POR-I3: Make Sonic Methods Truthful and Source-Bound

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src/ipc.ts`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `each_sonic_method_uses_one_named_shale_convention_and_records_it` (`SB-POR-013`, T20/T21).
- `a_raymer_hunt_gardner_label_names_only_the_verified_published_rendering` (`SB-POR-014`, T28).
- `non_wyllie_sonic_methods_reduce_floor_and_rescale_while_wyllie_stays_subtractive`
  (`SB-POR-015`, T18/T18b/T21).
- `sonic_matrix_transit_time_is_a_cited_lithology_choice_with_no_global_default`
  (`SB-POR-016`, T10).
- `a_wyllie_compaction_factor_below_one_is_refused_before_any_curve_is_written`
  (`SB-POR-017`, T29, `DEC-012`).
- `shale_corrected_slowness_never_falls_below_the_selected_matrix_slowness`
  (`SB-POR-018`, F5).
- `exactly_one_verified_rhg_rendering_is_authoritative_and_every_other_is_comparison`
  (`SB-POR-020`; remains blocked until its separate vendor choice is explicit).

- [ ] Start with the seven named tests. Preserve Wyllie `0.1917604` as the control. Test the exact
      T20 normalised/subtractive split from both sides and prove the selected convention survives in
      version history.
- [ ] Remove the false `RHG` identity from the current branch. If retained, expose it only as
      `FIELD_OBSERVED`, make `CFO` a cited parameter, and apply exactly one source-matched shale
      convention. Do not use it as a fallback when RHG80 is unavailable.
- [ ] Implement the original three-segment RHG80 route only after POR-I0 has verified the scan. Use
      the paper-derived controls in all three regions and at both joins; never calibrate against IP's
      proprietary bridge or any vendor closed form.
- [ ] Keep `OPT_CP` reachable only for Wyllie. Validate the complete request before any well runs;
      `Cp < 1` or `DT_SH <= 100 us/ft` with correction enabled returns a named refusal and writes no
      log-set version. `Cp == 1` is the no-inflation boundary control.
- [ ] Replace the one numeric `DT_MA` default with cited lithology/method choices from section 5.
      Conflicting sandstone Wyllie values remain a visible no-default choice; matched AFF pairs stay
      atomic and are not split into freely combinable endpoint/exponent values.
- [ ] For every non-Wyllie route, form shale-reduced slowness, floor at the selected matrix value,
      evaluate the transform and rescale by `1-VSH`. Keep Wyllie on the cited subtractive path.
- [ ] If the SB-POR-020 vendor choice is still absent, ship no vendor default and record the
      requirement BLOCKED. Do not commit a permanently failing test, ignore it, weaken it, or switch
      branches to hide it. The verified original RHG80 path may still land under DEC-017.
- [ ] Mutation-prove raw-DT seeding, shared post-subtraction, `Cp=0.90` inflation, a global DTMA
      default and the old RHG label. Restore and rerun.
- [ ] Add one unchecked REVIEW entry for method labels, source details, convention switch, empty
      DTMA choice, `Cp < 1` refusal and RHG80 boundary behavior.
- [ ] Run the mandatory gates and commit the source-ready subset as
      `POR-I3 SB-POR-013-020 harden sourced sonic methods`; list any blocked row explicitly.

---

## POR-I4: Replace N-D Shortcuts with a Real Analytic Method

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/curves.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src-tauri/src/neutron_charts.rs` only for SandiBumi-owned validation calls, never chart data
- Modify: `src/ipc.ts`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `src/styles.css`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `the_analytic_nd_method_matches_the_independent_chart_gate_within_one_porosity_unit`
  (`SB-POR-021`, T08/T37; blocked while ESC-POR-8 is open).
- `arithmetic_and_rms_curves_are_labelled_comparisons_and_never_primary_methods`
  (`SB-POR-023`, T36).
- `nd_porosity_refuses_neutron_data_without_a_declared_matrix_basis` (`SB-POR-024`).
- `each_nd_endpoint_clamp_is_source_bound_and_raises_the_porosity_flag`
  (`SB-POR-028`, T12/T38).

- [ ] Write the four named tests first. The matrix-basis test accepts an explicitly declared
      basis and refuses the same numerical NPHI when metadata is absent. The comparison test proves
      both visible availability and default pay exclusion.
- [ ] Remove the claim that average/RMS is a chart-equivalent method. Keep arithmetic and RMS as
      separately named quick-look comparison outputs under `DEC-014` and the common output-role
      contract.
- [ ] Require an NPHI matrix basis on the real input identity and record it in provenance. Do not
      infer basis from mnemonic alone. Reuse `nphimat` for explicit conversion, not as a silent
      auto-fix.
- [ ] If ESC-POR-8 closes, implement Bateman-Konen from the admissible source in `porosity.rs` and
      validate against SandiBumi's own gated digitisation without copying a vendor lookup table. If
      it remains open, mark `SB-POR-021` BLOCKED and land only the comparison/basis/clamp work.
- [ ] Replace hard-coded shale-reduction clamp endpoints with source-bound parameters. An absent
      endpoint refuses; every bound hit records the correct flag and preserves the unlimited value.
- [ ] Mutation-prove an inferred limestone basis, a comparison output presented as primary, a
      silent ceiling and the average shortcut substituted for Bateman-Konen. Restore and rerun.
- [ ] Add one unchecked REVIEW entry for basis refusal, comparison styling, analytic-method source
      panel, unlimited/limited pair and clamp flag.
- [ ] Run the mandatory gates and commit the source-ready subset as
      `POR-I4 SB-POR-021 023 024 028 establish honest ND porosity`.

---

## POR-I5: Implement the Source-Gated HC Response and Coupled Solve

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src/ipc.ts`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `src/ui/workflowDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `the_conventional_hc_electron_density_matches_poupon_within_the_cited_envelope`
  (`SB-POR-029`, T01).
- `the_gaymard_poupon_hydrogen_index_matches_the_primary_source_and_rejects_the_legacy_alpha`
  (`SB-POR-030`, T02/T03).
- `the_hydrocarbon_chain_exposes_separate_a_and_b_factors` (`SB-POR-031`).
- `mud_filtrate_density_and_hydrogen_loss_are_explicit_parameters_without_example_defaults`
  (`SB-POR-032`).
- `each_hydrocarbon_model_refuses_or_flags_outside_its_own_validity_domain`
  (`SB-POR-033`, T17).
- `the_selected_vendor_hydrocarbon_model_and_intermediates_survive_in_provenance`
  (`SB-POR-034`).
- `the_hydrocarbon_loop_refuses_until_the_sxo_exponent_is_explicitly_selected`
  (`SB-POR-035`, T25).
- `density_log_gascorr_and_porosity_hydrocarbon_correction_cannot_be_silently_double_applied`
  (`SB-POR-038`).
- `every_porosity_iteration_uses_parameterized_tolerance_and_cap_and_returns_missing_on_exhaustion`
  (`SB-POR-050`, T16/T24).
- `a_multi_unknown_porosity_solve_follows_the_documented_precedence` (`SB-POR-051`).
- `an_invalid_variable_sxo_configuration_is_refused_before_the_run_starts` (`SB-POR-052`).

- [ ] Write the eleven tests first using POR-I0's primary-source oracle. T01/T02 sweep every chapter
      point. T03 proves the legacy divergence is reachable only through an explicit labelled
      verification mode. T07 forward/inverse round-trip is supporting evidence, not its own oracle.
- [ ] Implement pure Conventional apparent-electron-density and Gaymard-Poupon hydrogen-index
      functions. Keep all vendor variants explicitly named; no generic “HC correction” selector.
- [ ] Expose `A_FACTOR`, `B_FACTOR`, `RHOHC_APP`, `HI_HC`, corrected density/neutron curves, `SXO`,
      iteration count and convergence/validity flags as QC outputs with typed roles.
- [ ] Make `RHO_MF`, `P_MF`, model choice, `Sxo` exponent, tolerance and iteration cap explicit
      parameters. Worked-example values are not defaults. A blank required field refuses at
      configuration time.
- [ ] Apply each model's own cited validity domain under `DEC-015`. A bound from another rendering
      cannot gate it. If ESC-2 leaves a bound absent, that method remains unavailable rather than
      borrowing the stoichiometric ceiling as a vendor-validity claim.
- [ ] Implement the deterministic unknown precedence recorded by SB-POR-051. Reject invalid variable
      combinations before the job starts. Terminate on `abs(delta) <= tolerance`; cap exhaustion
      returns MISSING plus flag and never the last iterate.
- [ ] Add an explicit input-correction-state contract so density-log `gascorr` and the POR HC chain
      cannot both apply silently. The pane explains which curve is already corrected and the run
      record preserves that declaration.
- [ ] Mutation-prove the Geolog alpha substitution, an example default, an omitted model identity,
      equality-only convergence, partial-iterate return, reordered unknowns and double correction.
      Restore and rerun.
- [ ] Add one unchecked REVIEW entry for model selection, missing-input refusals, intermediate/QC
      curves, convergence display, double-correction refusal and versioned provenance.
- [ ] Run the mandatory gates and commit as
      `POR-I5 SB-POR-029-035 038 050-052 add sourced HC response solve`.

---

## POR-I6: Add the Additive Excavation Correction

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance test:**

- `the_additive_excavation_term_matches_the_cited_reference_case_and_the_multiplied_form_is_unreachable`
  (`SB-POR-039`, T04-T06).

- [ ] Write the named test first. Independently derive the 2.71 p.u. reference, 1.1729 lithology
      ratio and multiplied-form failure discriminator from the chapter sources.
- [ ] Implement the additive term as a pure forward-model function and a separately named correction
      function; never a sign flag. This also advances DEC-016 and the non-pilot SB-POR-005/040
      product contract without silently changing their release disposition.
- [ ] Expose the chapter's two independently corroborated lithology renderings as cited choices if
      ESC-1 remains open; do not choose an exponent default. Record which rendering ran.
- [ ] Keep tool suppression out of this increment unless SandiBumi's own tool register resolves the
      identity. Never copy Techlog's blacklist string; unresolved `APSC` remains unresolved.
- [ ] Emit `DPHI_EX` with forward/correction direction, source and applicability flags. Missing
      required inputs remain MISSING, not zero correction.
- [ ] Mutation-prove the multiplied bracket, square-root lithology sensitivity and sign-flag API.
      Restore and rerun.
- [ ] Add one unchecked REVIEW entry for direction labels, cited-choice display, reference case and
      unresolved tool identity.
- [ ] Run the mandatory gates and commit the source-ready subset as
      `POR-I6 SB-POR-039 add sourced additive excavation correction`.

---

## POR-I7: Make Limits and Conditioning Explicit

**Files:**

- Modify: `src-tauri/src/porosity.rs`
- Modify: `src-tauri/src/modules.rs`
- Modify: `src-tauri/src/workflow.rs`
- Modify: `src-tauri/src/param_sources.rs`
- Modify: `src/ipc.ts`
- Modify: `src/ui/moduleDialog.ts`
- Modify: `src/ui/workflowDialog.ts`
- Modify: `src/styles.css`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `the_high_shale_kill_threshold_is_a_source_bound_parameter_and_raises_a_flag`
  (`SB-POR-043`).
- `the_phie_floor_has_no_compile_time_default_and_records_the_user_choice`
  (`SB-POR-045`, T40).
- `a_vsilt_curve_carries_the_do_not_trust_warning_where_it_is_read` (`SB-POR-046`).
- `badhole_is_a_declared_porosity_input_and_its_effect_is_recorded_per_sample`
  (`SB-POR-047`, T41).
- `coal_tight_and_conditioning_flags_have_explicit_porosity_branches`
  (`SB-POR-048`, T41).

- [ ] Before code, adjudicate the PHIE-floor precedence conflict: chapter no-default contract versus
      the later direct `0.001` product decision recorded in `docs/record_fixes.md`. Do not silently
      treat either as superseding the other. If no precedence decision is available, keep
      `SB-POR-045` BLOCKED and do not remove the existing safety floor in this increment.
- [ ] Write the five named tests first. Each must assert both numeric behavior and the visible flag/
      provenance surface. BADHOLE/COAL/TIGHT/COND accepted and absent/untyped controls distinguish
      declared integration from a generic optional mask.
- [ ] Replace hard-coded high-shale thresholds with cited/open manifest parameters. Hitting a
      threshold records the branch and leaves the unlimited curve intact.
- [ ] Once precedence is decided, replace compile-time floor ownership with the source-bound run
      contract and keep the pay path consistent with the selected, recorded primary PHIE. Preserve
      the NaN guard so MISSING never becomes a floor value.
- [ ] Surface the VSILT warning beside every selector, legend and report field where VSILT is read;
      a Help-only note is insufficient.
- [ ] Add BADHOLE, COAL_FLAG, TIGHT_FLAG and COND_FLAG as declared typed inputs with explicit branch
      behavior. Record their effect in the POR flag stream; do not depend on the analyst remembering
      the generic Mask option.
- [ ] Mutation-prove a silent high-shale step, a missing NaN guard, Help-only VSILT warning, generic-
      mask-only BADHOLE and ignored conditioning inputs. Restore and rerun.
- [ ] Add one unchecked REVIEW entry covering limit choices, flags, the PHIE floor, VSILT warning and
      conditioning behavior in the docked pane and plotted outputs.
- [ ] Run the mandatory gates and commit the unblocked subset as
      `POR-I7 SB-POR-043 045-048 expose POR limits and conditioning`.

---

## POR-I8: Remove Dead SSPW Controls and Restore RMS Parity

**Files:**

- Modify: `src-tauri/src/ssc.rs`
- Modify: `src-tauri/src/modules.rs` only if the shared manifest audit lives there
- Modify: `src/ui/moduleDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `REVIEW.md`

**Owned acceptance tests:**

- `every_active_porosity_parameter_is_read_and_every_unread_parameter_is_visibly_inactive`
  (`SB-POR-058`, T34).
- `sspw_gas_conditioning_uses_the_same_rms_midpoint_as_ssc` (`SB-POR-059`, T33).

- [ ] Write T33 from the chapter's independent RMS arithmetic and T34 against the real manifest
      plus reachable body paths. Do not make SSC call SSPW or vice versa as the correctness oracle.
- [ ] Bring SSPW's gas branch to `sqrt((phi_d^2+nphi^2)/2)` and pin the 0.1903943 reference from
      both modules independently.
- [ ] Re-port only the source-ready SSPW parameters. Until Jauhar signs off the held `sspw.lls`
      re-port, remove unread controls from the active manifest or mark them visibly inactive with a
      reason. A disabled control must not enter saved run parameters as though it affected output.
- [ ] Keep arithmetic comparison, RMS comparison, SSPW/SSC RMS conditioning, Gaymard-Poupon
      response and coupled `Sxo`/`Sw` solve separate under `DEC-014`.
- [ ] Mutation-prove the old 0.2/0.8 weighting and an enabled-but-unread parameter. Restore and rerun.
- [ ] Add one unchecked REVIEW entry for SSPW gas parity and inactive-control honesty.
- [ ] Run the mandatory gates and commit as
      `POR-I8 SB-POR-058 059 make SSPW controls and RMS conditioning honest`.

---

## POR-I9: Integrate, Re-Adjudicate and Hand Off Manual Review

**Files:**

- Create: `src-tauri/src/porosity_integration_tests.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `tools/frontend-acceptance.test.mjs`
- Modify: `docs/takeover/evidence/sb-por.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Modify: `REVIEW.md`

**Interfaces:** Full manifest -> docked pane -> workflow -> versioned curve catalog -> plot/selector
-> pay-summary path. No internal helper result alone qualifies.

- [ ] Run one synthetic but source-derived density workflow, one sonic workflow, one N-D workflow,
      one HC workflow and one excavation workflow through the real runner. Inspect current curves,
      archive versions, params, input identities, output roles, flags and refusals.
- [ ] Prove two differently named density and D-N runs survive together. Prove deliberate same-name
      replacement creates a new version and is restorable. Prove comparison curves never become the
      default pay PHIE.
- [ ] Prove every finite POR result has method identity, convention, source/tier for every parameter,
      input identities, units, output role and flag legend. Prove a required absent parameter writes
      neither a version nor an all-MISSING curve.
- [ ] Run all 44 owned acceptance tests by exact name, all chapter T01-T41 source-ready tests, and
      every relevant existing module/workflow/frontend test. Report blocked tests as blocked
      requirements; do not ignore them or manufacture expectations.
- [ ] Re-adjudicate each of the 44 pilot-blocker rows against observable production behavior and
      independently sourced tests. A compound row remains PARTIAL if any UI, provenance, flag,
      refusal, persistence or manual obligation is unproved.
- [ ] Preserve all source-owned blank status/test fields in `requirements.csv`; update only
      adjudication-owned fields and the POR receipt. Run takeover ledger and PRD-audit checks.
- [ ] Add one consolidated unchecked manual plan at the top of `REVIEW.md` for Jauhar: source/no-
      default presentation, output naming/replacement, flag visibility, method labels, RHG joins,
      N-D basis, HC intermediate curves, convergence/refusal, excavation direction, conditioning,
      comparison/pay exclusion and archive restore.
- [ ] Run `npx tsc --noEmit`.
- [ ] Run `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` and require zero failures.
- [ ] Run `git diff --check`, inspect every path and stage no unrelated or protected material.
- [ ] Commit as `POR-I9 re-adjudicate integrated SB-POR pilot contracts`; do not push or start SAT
      implementation.

---

## Explicitly Outside This Pilot-Blocker Plan

- `SB-POR-005`, `012`, `019`, `022`, `025`-`027`, `036`, `037`, `040`-`042`, `044`, `049`, `053`,
  `060`-`062` are not silently promoted merely because adjacent code is touched. `DEC-016` records
  product inclusion for analytic N-D, HC response, excavation and neutron-sonic, but separate source
  gates and current release dispositions remain visible.
- Neutron-sonic (`SB-POR-027/053`) is planned only after its interface with SAT has been adjudicated.
  Its canonical fluid-minus-matrix shale term and the forbidden Techlog form remain binding.
- Smooth high-shale roll-off remains absent until the user supplies all three no-default parameters.
- Vendor parameter-deck import, POR audit report and core post-check stay deferred. Core comparison
  never auto-adjusts a petrophysical parameter.
- Manual review is Jauhar's evidence lane. Engineering prepares observable UI and unchecked
  scenarios; it does not mark manual evidence complete on his behalf.

## Plan Self-Review

- [ ] Exactly 44 current POR pilot-blocker requirements appear once in the owned-test lists.
- [ ] Every test name is a sentence and every expected value has a chapter/source/independent-
      arithmetic oracle.
- [ ] DEC-012 through DEC-017 are represented without converting a product decision into an
      invented parameter.
- [ ] The original RHG80 scan gate is explicit and `docs/method_sonic_porosity.md` is never treated
      as implementation authority.
- [ ] The original-paper RHG80 choice and the separate SB-POR-020 vendor-rendering choice remain
      non-overlapping.
- [ ] ESC-POR-8 prevents the analytic N-D constants from being copied from protected vendor data.
- [ ] Common POR types standardize custody and observability, not method-specific numeric bounds.
- [ ] Unlimited/limited, PHIE/PHIT, apparent/effective, imported/computed, method/comparison,
      correction/forward and VSH/VCL identities remain distinct.
- [ ] Every silent clamp and branch has a per-sample flag and a preserved unlimited value.
- [ ] Every refusal happens before log-set creation or curve write.
- [ ] Versioned delete-then-append writes and PK-less `computed_curves` remain unchanged.
- [ ] No database schema, `db.rs`, frontend write SQL, JSON array IPC, embedded Python, client name,
      protected chart value or uncited endpoint is introduced.
- [ ] Every production increment runs TypeScript, Cargo check and the full PowerShell gate before
      its commit, with zero failures and exact totals reported.
- [ ] Every increment adds an unchecked manual scenario but never claims Jauhar performed it.
- [ ] Gate 1 adjudication remains the immediate program step; this plan does not silently begin
      Gate 2 production work.
