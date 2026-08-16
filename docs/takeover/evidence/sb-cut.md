# SB-CUT live adjudication receipt

## Execution baseline

- Working tree: D:\XX. SandiBumi; branch: codex/g1-sb-cut-adjudication; planning commit: 25ee7af05f835434468a11e04e39feea3192aeb6.
- Accepted implementation anchor b332026cb498c105f36eade0bf7899bc0c1309f0 is reachable. origin/master and the merge base are both 29833735816d9e5be954afafd9ceb71fd856e3f0.
- Immutable chapter SHA-256: e4972351ae548204a92300f3ab75c0b9415af6f6a2e72709be6119e553e41359. Frozen six-column ledger SHA-256: 40d21a70e16657bda4779deffa66c7d52467458f1aca09eead6623de19e0a3fe.
- Scope guard: exactly 61 source-owned rows, SB-CUT-001 through SB-CUT-061; P0=9, P1=23, P2=22, P3=7; all owned-test fields populated; all live verdict fields unadjudicated at entry.
- Candidate execution on the unchanged implementation tree: workflow 36 passed plus 1 optional-package ignore; Monte Carlo 23 passed; report 13 passed; core-reporting 1 passed; net-flag 8 passed; frontend acceptance 13 passed. These 94 passes are credited only to the limbs they actually observe.
- Manual evidence remains source-owned and unchanged: cutoffs/pay 0/23, workflow 0/23, field dashboard 0/10, Results QC 0/1, Monte Carlo 2/14, and report 6/53. Automation never closes those manual scenarios.
- No production code, test, PRD, research dossier, parameter source, or manual-evidence state changes in this increment.

## Governing boundaries

- A parameter is cited or remains absent. The live cutoff quartet, generic uncertainty-width heuristic, and vendor-derived prior widths are implementation evidence, not authority.
- E-1 closes only historical exposure: the required SD_MULT=2 implementation and T13 regression remain open. O-2 blocks bed-amalgamation tie-breaking. O-3 through O-12, E-3 through E-6, and R-1 through R-13 retain their chapter meanings.
- An internal Result, helper formula, or matching code path is not an observable correctness test. Supporting tests are classified CHARACTERIZATION unless they independently prove the whole contract.
- This receipt distinguishes as-built behavior, release disposition, automated proof, and field evidence. A PRESENT-OK row can remain a PILOT-BLOCKER when its required field exercise or full acceptance proof is still missing.

## Acceptance-test routing

Each chapter intention has one primary receipt owner below. Source-owned cross-support in the
ledger remains unchanged and does not make a partial helper sufficient for a compound contract.

| Test intention | Primary receipt owner |
|---|---|
| SB-CUT-T01 | SB-CUT-001 |
| SB-CUT-T02 | SB-CUT-001 |
| SB-CUT-T02b | SB-CUT-001 |
| SB-CUT-T03 | SB-CUT-001 |
| SB-CUT-T03b | SB-CUT-001 |
| SB-CUT-T03c | SB-CUT-001 |
| SB-CUT-T04 | SB-CUT-007 |
| SB-CUT-T05 | SB-CUT-006 |
| SB-CUT-T06 | SB-CUT-009 |
| SB-CUT-T07 | SB-CUT-010 |
| SB-CUT-T08 | SB-CUT-012 |
| SB-CUT-T09 | SB-CUT-008 |
| SB-CUT-T10 | SB-CUT-011 |
| SB-CUT-T11 | SB-CUT-003 |
| SB-CUT-T12 | SB-CUT-034 |
| SB-CUT-T13 | SB-CUT-031 |
| SB-CUT-T14 | SB-CUT-038 |
| SB-CUT-T15 | SB-CUT-041 |
| SB-CUT-T16 | SB-CUT-044 |
| SB-CUT-T17 | SB-CUT-060 |
| SB-CUT-T18 | SB-CUT-049 |
| SB-CUT-T19 | SB-CUT-043 |
| SB-CUT-T20 | SB-CUT-032 |
| SB-CUT-T21 | SB-CUT-060 |
| SB-CUT-T22 | SB-CUT-005 |
| SB-CUT-T23 | SB-CUT-041 |
| SB-CUT-T24 | SB-CUT-020 |
| SB-CUT-T25 | SB-CUT-041 |
| SB-CUT-T26 | SB-CUT-019 |
| SB-CUT-T27 | SB-CUT-040 |
| SB-CUT-T28 | SB-CUT-037 |
| SB-CUT-T29 | SB-CUT-045 |
| SB-CUT-T30 | SB-CUT-032 |
| SB-CUT-T31 | SB-CUT-013 |
| SB-CUT-T32 | SB-CUT-023 |
| SB-CUT-T33 | SB-CUT-039 |
| SB-CUT-T34 | SB-CUT-051 |
| SB-CUT-T35 | SB-CUT-018 |
| SB-CUT-T36 | SB-CUT-016 |
| SB-CUT-T37 | SB-CUT-055 |
| SB-CUT-T37b | SB-CUT-056 |
| SB-CUT-T37c | SB-CUT-054 |
| SB-CUT-T38 | SB-CUT-057 |
| SB-CUT-T39 | SB-CUT-061 |

## Requirement receipts

### SB-CUT-001 - Make the depth discretisation model an explicit parameter

- **Specified contract:** expose the depth discretisation model as a named parameter with values `CENTRED`, `TOPS` and `BOTTOMS`, **default `CENTRED`**, all three implemented by the single interval-ownership rule plus the shared zone clip `h = max(0, min(Z_bot, b) - max(Z_top, a))`. `sum(h) = Z_bot - Z_top` **MUST** hold exactly under every model, and the rule **MUST have exactly one implementation**, shared by the pay summary, the cut-off sweep and the Monte Carlo path (`14_cutoffs-summation-mc.md:904-919`).
- **Why CENTRED:** four independent vendor votes - IP hard-codes it; Geolog `tp_paysummary.info` L63 `FRAME_REP` defaults `CENTRALISED`; `determin_mc` pins it; Techlog implements an unreachable centred branch. Techlog's min/max clip is the correct IMPLEMENTATION because it is exact when a zone boundary falls between samples and reduces to IP's half-weight rule when it falls on one.
- **Current implementation - verified in code.** No `CENTRED`/`TOPS`/`BOTTOMS` parameter exists anywhere: grep for those tokens returns nothing in `workflow.rs` or `montecarlo.rs`. The clip rule is duplicated - `workflow.rs` carries it at `:2933` and `:3225` among three sites, and `montecarlo.rs` has its own copy, its comment at `:778` recording the same last-in-zone bleed fix independently. The chapter's line numbers are stale but its substance holds exactly.
- **Magnitude:** one half-step per zone-boundary contact - **0.25 ft on a 0.5 ft grid**, with opposite signs at the two contacts. On IP's own fixture the models differ 3.25 vs 3.0 ft.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Manual evidence:** cutoffs 0/24.
- **Source/parameter boundary:** nothing needs inventing. The three model names, the default, the clip formula and the invariant are all cited. The default change is **not** an interpreter decision: the chapter mandates `CENTRED` on four vendor votes, and moving from today's TOPS-with-clip will change net thickness by up to a half-step per zone contact - a real, expected, cited move.
- **Blocker or decision:** `BLOCKED-BOUNDARY`. The requirement is *exactly one implementation shared by* the pay summary, the sweep **and the Monte Carlo path**. Two are in `workflow.rs`, allowed; the third is `montecarlo.rs`, prohibited. Building the shared rule in an allowed file does not help - making `montecarlo.rs` CALL it is itself the prohibited edit, and changing the other two while leaving Monte Carlo on its own copy would make the engines DISAGREE rather than agree, which is worse than today. **`montecarlo.rs` is a new file in the blocked set**, joining `multimin2.rs`, `lrlc.rs`, `multimin.rs` and `satheight.rs`.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; test class `MISSING`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes the narrow `montecarlo.rs` edit alongside the existing SAT-group request. Then build ONE interval-ownership rule with the shared clip, expose the model parameter defaulting to `CENTRED`, route all three consumers through it, and pin: `sum(h) = Z_bot - Z_top` exactly under all three models; the three models DIFFER on a boundary falling between samples; and they AGREE when it falls on one - the arm proving the clip reduces to IP's half-weight rule.

## SB-CUT-002 - Name the discretisation model on every thickness-bearing result

- **Specified contract:** every result record carrying a thickness, a net, a net-to-gross or a thickness-weighted average **MUST** carry the discretisation model that produced it **and the sample interval it was computed on**. A consumer **MUST NOT** have to infer either (`14_cutoffs-summation-mc.md:928-941`).
- **Why:** IP ships **two different definitions of Net in one product** - Cut-off and Summation's half-weight rule and Curve Statistics' `count x step` - under the same column heading, and labels neither. A summation number without its discretisation model is not reproducible. The sample interval is required **separately** because net-to-gross is **not scale-invariant**: 0.55 -> 0.75 -> 1.0 across three blocking steps on the same data.
- **Current implementation - verified in code.** `PaySummaryRow` (`workflow.rs:2637`) carries `well_id`, `well_name`, `zone`, `flag`, `top`, `bottom` and the summation numbers, and **no discretisation model or sample interval**. Monte Carlo is in scope too and equally silent: `montecarlo.rs:272-273` emits `net` and `ntg` percentile bundles per zone with no model recorded.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Manual evidence:** cutoffs 0/24.
- **Source/parameter boundary:** nothing needs inventing - this is custody of a fact the run already knows. Note the row does **not** require SB-CUT-001 to land first: today's model is TOPS-with-clip, and recording *that* is truthful and useful, because the record states what produced **this** number. When SB-CUT-001 exposes the parameter the field simply becomes dynamic.
- **Blocker or decision:** `BLOCKED-BOUNDARY`, on the **same** `montecarlo.rs` authorization as SB-CUT-001. The `PaySummaryRow` half is in `workflow.rs`, allowed, and could be built today. The Monte Carlo half cannot, and the requirement is *every* thickness-bearing record - so shipping only the pay summary would leave the one consumer whose output is a **distribution** unlabelled, which is the harder number to reproduce and therefore the one that most needs its model stated. Held whole for that reason rather than for consistency alone.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Next action:** Jauhar authorizes the narrow `montecarlo.rs` edit - the same one SB-CUT-001 needs. Then add the model and the sample interval to `PaySummaryRow` and to the Monte Carlo result bundle, carry both into the report and the workbook, and pin from both sides: a record states the model and the step it was computed on, and **a consumer reading two records computed at different steps can tell them apart** - which is the whole point, since net-to-gross is not scale-invariant.

## SB-CUT-003

- **Specified contract:** Gross equals Net plus NotNet plus Unknown exactly, with each component reported.
- **Current implementation / as-built:** `PaySummaryRow` now carries `not_net` and `unknown` beside `gross` and `net`. `not_net` accumulates in the same loop pass as `net` and takes footage only where the flag was actually EVALUATED and rejected, so a NaN flag cannot land there. `unknown` is DERIVED as `gross - net - not_net`. PRESENT-OK.
- **Why the derivation is the requirement rather than a shortcut:** two different things make footage unjudgeable and only one of them is a sample - an in-zone sample with no VSH/PHIE/SWE to judge, and footage carrying no sample at all, which is a logging gap or the ordinary case of a zone bottomed on a marker below the TD of the run that logged it. Accumulating only the first leaves the identity broken over exactly the intervals where a reader most needs it to close.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_summation_partitions_gross_four_ways_and_books_unjudgeable_footage_as_unknown_not_as_notnet` (`src-tauri/src/workflow.rs`). CORRECTNESS. Pinned from both sides because the invariant ALONE is satisfied by the exact error the requirement names - fold every unjudgeable sample into NotNet and Gross still closes. Arm A: on a zone the samples tile exactly, each component is its own expected footage (10 net / 5 not-net / 5 unknown of 20), so NotNet cannot silently absorb the missing-VSH interval. Arm B: on a zone declared ten units below the logged interval, Unknown is 15 - six unjudgeable sampled units plus nine units nothing logged at all - and the partition still closes. All three summary flags are checked rather than one standing in for the others.
- **Mutation evidence:** two probes, each read for WHICH assertion fired. Removing the evaluated-flag guard so NaN samples fold into NotNet turned arm A red with `not_net` 10.0 against 5.0. Replacing the derivation with a NaN-flag accumulator turned arm B red with `unknown` 6.0 against 15.0 - the nine unlogged units silently gone.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value was adopted. Expected footages are the fixture's own geometry.
- **UI/IPC/provenance surface:** `ipc.ts` `PaySummaryRow` mirrors both fields; `dashboardPanel.ts` `GRID_COLS` adds **Not net** and **Unknown** beside Gross and Net, which drives both the on-screen grid and the CSV export; `core_determinism_tests.rs` `packed_pay_summary` covers them so the re-run determinism hash cannot go blind to a new field.
- **History/reachability:** `n_classified` and `perm_cutoff_no_data` remain what they were - discriminators, not footage categories - and are unchanged.
- **Blocking decision / next action:** cleared. Jauhar field-verifies on a well whose zone bottoms below the logged interval.

### SB-CUT-004

- **Specified contract:** report both N:G and N:(G-Unknown) with unambiguous labels.
- **Current implementation / as-built:** `PaySummaryRow` carries `ntg_known = net / (gross - unknown)` beside the existing `ntg = net / gross`. PRESENT-OK.
- **Why both and not one:** the gap between them is exactly the null fraction. Over a washed-out or partly-logged interval that gap is the whole argument about whether a net-to-gross is defensible, and no incumbent surfaces both - so an interpreter comparing one tool's number with another's cannot tell they are answering different questions.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_summation_reports_net_to_gross_over_all_footage_and_over_only_the_footage_it_could_judge` (`src-tauri/src/workflow.rs`). CORRECTNESS. Three cases, because either ratio alone looks reasonable: the zone the samples tile exactly (0.50 against 0.67 - they differ even there, because some samples had nothing to judge); the zone declared ten units below the log (0.33 against 0.67, and the second is asserted strictly GREATER, so a second number equal to the first fails); and the well nobody interpreted, where the second ratio has no denominator and must come back MISSING.
- **Mutation evidence:** two probes, each read for WHICH assertion fired. Dividing by `gross` instead of `gross - unknown` turned the first arm red with 0.5 against 10/15. Reporting 0.0 instead of MISSING where nothing was judged turned the third arm red with 0 - the value that reads as *none of the judged rock is net* about a well nobody looked at.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value was adopted. Both denominators follow from the SB-CUT-003 partition; no tolerance is invented here (that is SB-CUT-005).
- **UI/IPC/provenance surface:** `ipc.ts` types it `number | null`; `dashboardPanel.ts` `GRID_COLS` adds the labelled column **N/G excl. Unk** beside **N/G**, which drives both the grid and the CSV export; `core_determinism_tests.rs` `packed_pay_summary` covers it.
- **Named limit:** the workbook, report and deck read the same `PaySummaryRow` and were NOT given the second column here. They render their own tables and adding a column to each is its own change with its own layout consequences; the summation itself now reports both, which is what this row requires.
- **History/reachability:** `ntg` is unchanged - the first ratio was never wrong, only alone.
- **Blocking decision / next action:** cleared. Jauhar field-verifies on a partly-logged zone.

### SB-CUT-005

- **Specified contract:** reconcile the four-way footage partition with a named relative tolerance, record absorbed residuals, and refuse excess residual.
- **Current implementation / as-built:** `PARTITION_TOLERANCE = 1e-7` and `reconcile_partition` in `workflow.rs`, with two structured types - `ReconciledPartition {net, not_net, unknown, absorbed}` and `PartitionResidual {gross, net, not_net, unknown, residual, relative, tolerance}` whose `Display` names every one of them. `run_pay_summary` calls it on every zone-by-flag row and propagates the refusal prefixed with the well, zone and flag. `PaySummaryRow` gains `residual_absorbed`. PRESENT-OK.
- **Why it checks the REPORTED values:** the three f64 sums are each rounded once on the way into the row, so the closure a reader receives is not automatically the closure the arithmetic had. The check is evaluated in f64 on those f32 values, so it adds no rounding of its own. Absorption targets the LARGEST component because a relative correction placed on a small component could move it by a large fraction of itself.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_footage_partition_is_absorbed_into_its_largest_component_and_the_amount_recorded_or_else_refused` (`src-tauri/src/workflow.rs`). CORRECTNESS. Five arms: a within-tolerance residual absorbed into Net when Net is largest; the SAME residual absorbed into Unknown when Unknown is largest, so first-in-order cannot pass for largest; a beyond-tolerance residual REFUSED with a message naming gross, the residual and the tolerance (R-12: the non-reconciling fixture must fail, never be normalized into success); an exactly-zero residual still reconciling and recording zero rather than short-circuiting; and a real `run_pay_summary` row proving the guard is wired into the live path rather than sitting in a test.
- **Mutation evidence:** three probes, one per clause of the MUST, each read for WHICH assertion fired. Absorbing into the first component rather than the largest turned the second arm red at 200000.06. Absorbing but reporting 0.0 - which is exactly Techlog's `print`, behaviourally - turned the first arm red on *the absorbed amount is RECORDED, not printed and lost*. Widening the tolerance so nothing is ever refused turned the third arm red on its `expect_err`.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** `1e-7` is cited to `14_cutoffs-summation-mc.md:2083`, adopted from Techlog's `adjustFinal` with the print-to-result-field refinement. The footages in the test are NUMERICAL fixtures chosen so a residual at the tolerance boundary is exactly representable in f32 - at a realistic gross of tens of metres, 1e-7 relative is far below one ulp and no absorption could be observed at all - and they are not petrophysical quantities.
- **UI/IPC/provenance surface:** `PaySummaryRow.residual_absorbed` and its `ipc.ts` twin; `core_determinism_tests.rs` `packed_pay_summary` covers it.
- **Named limit:** the absorbed amount has NO UI column. It is zero on every row of every ordinary run, and a column of zeros in a dense grid is noise; it is on the record and in the IPC type, which is what the requirement asks (*appear in the result record*, as against Techlog's console). REVIEW.md asks Jauhar whether he wants it surfaced.
- **History/reachability:** nothing prior existed; the negative inventory was correct.
- **Blocking decision / next action:** cleared.

### SB-CUT-006

- **Specified contract:** calculate a general power mean controlled by an explicit per-curve exponent.
- **Current implementation / as-built:** summaries expose fixed arithmetic and PHIE-weighted formulas only; no exponent field or general evaluator exists. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T05 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** p is explicit user input; the chapter-cited p values are verification fixtures, not defaults.
- **UI/IPC/provenance surface:** no request, metadata, result or report field selects or records p.
- **History/reachability:** no general power-mean implementation was found.
- **Blocking decision / next action:** product-select the averaging family, then add the explicit exponent and independently derived p=1,-1,0,1/3 fixtures.

### SB-CUT-007

- **Specified contract:** compute a weight-normalised geometric mean and count excluded non-positive samples.
- **Current implementation / as-built:** no geometric aggregation or exclusion count exists. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T04 and T09 are absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** R-2 forbids adopting the unit-dependent vendor formula; the chapter supplies the independent invariant.
- **UI/IPC/provenance surface:** absent throughout summary and exports.
- **History/reachability:** negative source and history inventory confirmed absence.
- **Blocking decision / next action:** if selected, implement the normalized equation and prove unit invariance plus exclusion counts.

### SB-CUT-008

- **Specified contract:** harmonic averaging skips and counts non-positive samples without aborting the interval.
- **Current implementation / as-built:** no harmonic aggregation exists. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T09 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** the behavior is specified; O-1 keeps any vendor-defect statement scoped to the inspected script.
- **UI/IPC/provenance surface:** no selector, result or exclusion count exists.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, add the guarded harmonic mean and positive/non-positive controls.

### SB-CUT-009

- **Specified contract:** porosity weighting is an explicit property of each averaged curve, never a mnemonic inference.
- **Current implementation / as-built:** `AverageWeighting {Thickness, Porosity}`, `AVERAGED_SLOTS`, `default_weighting`, `weighting_for` and a `WeightedMean` accumulator in `workflow.rs`; `PaySummaryRequest.weighting` is a `BTreeMap` keyed by SLOT, persisted with the rest of the run configuration in `log_sets.params_json`. `run_pay_summary` keeps one accumulator per averaged slot carrying whichever weighting the run declared, replacing the four hand-rolled sums that hard-wired the φ-weighted form to the saturation slot. PRESENT-OK.
- **Both as-built gaps closed:** the φ-weighted form can now be REQUESTED for another curve, and can be SWITCHED OFF.
- **Where the flag lives, and why not on the curve:** the register asked for *typed curve weighting metadata*. The chapter specifies a flag *stored with the curve's averaging configuration*, and the harm it cites is Techlog's summation rule (*"the SW curve is weighted by POR but the SWE is not weighted"*), which is a property of the summation setup rather than of the curve. SB-CUT-006/007/008, which would introduce a richer per-curve averaging configuration, are **not in the pilot scope**, so the run configuration is the configuration the approved scope has - and it is persisted, not an argument that evaporates.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** `zone_averaging_weighting_is_declared_per_curve_and_never_inferred_from_the_curve_name` (`src-tauri/src/workflow.rs`). CORRECTNESS. Four arms on a fixture where φ is deliberately ANTI-correlated with the other curves, so the two weightings give visibly different answers: declaring nothing keeps the vendor-agreed behaviour (Sw 0.30 φ-weighted, Vsh 0.25 and φ 0.20 by thickness); declaring thickness for the saturation slot moves Sw to 0.40, which is the number Techlog silently produces for a curve spelled the wrong way; declaring porosity for the Vsh slot moves it to 0.175 without disturbing the others; and the same rock with its porosity curve stored under a different mnemonic averages identically. A structural arm additionally proves the one resolved MNEMONIC the summation holds (`phie_curve`) never reaches the resolver - the difference between *does not infer from the name* and *happens not to today*.
- **Mutation evidence:** three probes, one per clause, each read for WHICH assertion fired. Ignoring the caller's declaration turned the switch-off arm red at 0.3. Defaulting saturation to thickness turned the default arm red at 0.4. Weighting by `h` instead of `φ·h` turned the default arm red at 0.4 as well - the φ-weighted form collapsing to the thickness-weighted one, which is exactly the silent wrongness.
- **Regression evidence:** the full 1028-test backend suite passes unchanged, which is what says the accumulator restructure moved no existing number.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** the default is cited to `:1041-1042` (three-vendor agreement on `Σ(Sw·φ·h)/Σ(φ·h)`), and it is also what the engine already did, so nothing moves for a caller who declares nothing.
- **UI/IPC/provenance surface:** `ipc.ts` `PaySummaryRequest.weighting?: Record<string, "thickness" | "porosity">`; persisted per run in `log_sets.params_json`.
- **Named limit:** no pane exposes the declaration yet - it is reachable over IPC and recorded in provenance, but the Cutoffs and Summary pane offers no control. Whether an interpreter should be able to change it per run is a product question; REVIEW.md asks it.
- **History/reachability:** the φ-weighted arithmetic was correct and is unchanged; only its custody moved from hard-wired to declared.
- **Blocking decision / next action:** cleared.

### SB-CUT-010

- **Specified contract:** direct HCPV equals Net times PhiAvg times one-minus-SwAvg only under porosity weighting, with a thickness-weighted negative control.
- **Current implementation / as-built:** unchanged arithmetic in both engines - this row is a PROOF, not an implementation. PRESENT-OK. The only production change is visibility: `montecarlo.rs` `zone_metrics`, `ZoneMetrics` and `Cutoffs` become `pub(crate)` so the identity can be asserted against the function that actually emits it.
- **Why against that function and not the run:** percentiles do not commute with a product. The identity is a statement about ONE realization, so asserting it across the P10/P50/P90 bundle would be asserting something false.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** `hydrocarbon_pore_volume_summed_directly_equals_the_volume_rebuilt_from_the_reported_averages_in_both_engines` (`src-tauri/src/workflow.rs`). CORRECTNESS. The expected value is an INDEPENDENT algebraic identity rather than a re-derivation of the code, which is what the register meant by *shared implementation is not an independent proof*: `Net.phi_bar.(1-Sw_bar)` expands to `sum phi.h - sum Sw.phi.h`, which IS `sum phi.h.(1-Sw)`, and it cancels only because `Sw_bar` is phi-weighted. Three arms: every emitted zone and flag closes to 1e-6 relative AND the absolute 1.4 is asserted, so an engine returning zeros could not satisfy it vacuously; the thickness-weighted negative control the chapter demands, where the rebuilt side moves to 1.2 and the identity is asserted to FAIL by more than 1e-3; and the same identity in the Monte Carlo engine.
- **Mutation evidence:** three probes, each read for WHICH assertion fired. Dropping phi from the direct summation turned arm A red at 6 against 1.4. Dropping phi from Monte Carlo's phi-weighted denominator turned arm C red at 1.4 against 1.88. Rebuilding HPV FROM the averages instead of summing it - the register's *shared implementation* made literal - turned arm B red, because the direct side started tracking a weighting choice it must be independent of.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value was adopted; the numbers are the fixture's own arithmetic. **R-13 honoured:** HCPV stays a LENGTH throughout and is never multiplied into a volume.
- **Named limit, stated rather than assumed:** the identity requires phi and Sw to be valid across the whole net interval. Where Sw is missing over part of net, `Net.phi_bar` counts footage `HCPV` cannot, and the identity is not claimed - the engine deliberately normalises each average over the footage ITS OWN curve was valid on, which is a separate pinned rule. T07's fixture is a flagged interval with varying phi and Sw, so the precondition holds there by construction. REVIEW.md tells Jauhar what to expect on a zone with gaps.
- **UI/IPC/provenance surface:** unchanged; the rows already expose every ingredient, and SB-CUT-009 now carries the weighting convention in the run's persisted configuration.
- **History/reachability:** compatible arithmetic was integrated and stays integrated.
- **Blocking decision / next action:** cleared.

### SB-CUT-011

- **Specified contract:** samples outside all zones contribute to no cumulative result.
- **Current implementation / as-built:** unchanged - all three paths already restricted interval overlap to named zones. PRESENT-OK. **No production code was touched by this row**; it is the proof the register asked for.
- **Release disposition and risk:** PILOT-BLOCKER; FIELD-EVIDENCE.
- **Automated evidence:** `a_sample_outside_every_declared_zone_contributes_to_no_summary_statistic_however_well_it_passes_the_cutoffs` (`src-tauri/src/workflow.rs`). CORRECTNESS, and it is the ONE three-path fixture the register asked for: one well, three bands (UPPER declared, LOWER declared, and a band below every zone), exercised through `run_pay_summary`, `run_cutoff_sweep` restricted to UPPER, and `montecarlo::zone_metrics`.
- **The false pass it is built to avoid:** an out-of-zone sample that also fails a cut-off is excluded for the wrong reason and proves nothing. So the test OPENS by asserting those samples return `(1.0, 1.0, 1.0)` from `classify_sample` - they clear SAND, RESERVOIR and PAY on their own merits - and they carry values found nowhere else (φ 0.50 against the zones' 0.30 and 0.10), so any leak moves a number rather than hiding in an average.
- **Mutation evidence:** five probes across the three limbs, each read for WHICH assertion fired. Summary: removing both zone clips, and removing only the base clip, each turned UPPER's net to 25; a third probe that leaked out-of-zone samples into the AVERAGES ONLY left net correct at 10 and moved `avg_phie` to 0.26, which is what proves the summary statistics are pinned independently of net rather than as a by-product of it. Sweep: ignoring the zone bounds turned the series to `[25, 25, 25]`. Monte Carlo: dropping the base clip turned its net to 25.
- **A probe that did NOT fire, recorded rather than quietly dropped:** dropping Monte Carlo's zone TOP clip changed nothing, because this fixture's zone boundaries land exactly on sample boundaries. Straddling behaviour is a separate rule (a sample crossing a boundary contributes its in-zone part) and is not what this row owns; the base clip is the one that gates the below-zone band, and that is the probe kept.
- **Manual evidence:** cutoffs-pay 0/23. Automated only. The register's *field-exercise zone boundaries* stays Jauhar's and is asked in REVIEW.md.
- **Source/parameter boundary:** IP's zone-membership rule is cited; no endpoint invented; every expected number is the fixture's own arithmetic.
- **Supporting test, moved out of the qualifying register:** `cutoff_sweep_ntg_and_dst_mask` remains and still passes, but it is CHARACTERIZATION of the sweep limb alone and no longer the qualifying proof for this row. It is retained, not deleted.
- **UI/IPC/provenance surface:** unchanged; no cumulative out-of-zone bucket exists, and the test asserts none is invented (exactly six rows: two declared zones by three flags).
- **History/reachability:** all three paths were integrated and are unchanged.
- **Blocking decision / next action:** cleared for automated evidence; field evidence remains open.

### SB-CUT-012

- **Specified contract:** depth reference frame is part of result identity.
- **Current implementation / as-built:** `SummationFrame {Md, Tvd, Tvdss, Tst}` and `MD_WEIGHTS_SOURCE` in `workflow.rs`; `PaySummaryRequest.frame` (serde default MD, persisted with the run configuration in `log_sets.params_json`) and `PaySummaryRow.frame` / `.weights_source`. PRESENT-OK.
- **How an ABSENT row closed without building TVD summation:** the requirement does not demand that a TVD summation EXIST. It demands that a result carry its frame, that MD and TVD be separate records, and that a TVD result never be an MD result rescaled. So `run_pay_summary` declares MD on every row and REFUSES any other frame before touching a well, naming the frame and stating what is missing - the weights would be `dz*cos(theta)` from the deviation survey. That is the sanctioned second half of IMPLEMENT-OR-REFUSE, and it is a stronger guarantee than a feature would be: a TVD number cannot be quoted at all until one can honestly be computed.
- **Why the frame is on the ROW and not only in provenance:** it is part of the result's IDENTITY. The per-sample weight is `dz` in MD and `dz*cos(theta)` in TVD, so the weights differ, not merely the totals - by a factor of two in a 60-degree hold, which is why IP says TVD zonal averages *"could be considerably different"*. A net thickness quoted in a deviated field without its frame is a number a reader cannot use.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_summation_declares_the_depth_frame_its_weights_came_from_and_refuses_one_it_cannot_weight` (`src-tauri/src/workflow.rs`). CORRECTNESS. It pins the four-frame vocabulary and its spellings, that MD is the default, that every emitted row carries BOTH the frame and the source its weights were differenced from, and that TVD, TVDSS and TST each refuse with a message naming the frame AND what is missing - a refusal that says only *no* is not actionable.
- **Mutation evidence:** three probes, one per clause, each read for WHICH assertion fired. Serving MD numbers for a non-MD request turned the refusal arm red. Emitting an empty `weights_source` turned the source arm red. Stamping a fixed frame instead of the requested one turned the frame arm red.
- **A dead constant removed rather than declared:** a `SummationFrame::ALL` array existed for the test alone and the repo's own hygiene gate flagged it as unused in production. It is gone; `as_str`'s match is exhaustive and lives in production, so a fifth frame cannot be added without deciding there what it is called - a stronger guard than a list a test could let go stale.
- **Manual evidence:** cutoffs-pay 0/23. Automated only.
- **Source/parameter boundary:** the four-frame vocabulary is the union of Techlog's four and IP's two, as the chapter states. **O-3 is left open and unguessed**, and it cannot bite here: SandiBumi's summation has NO height cutoff at all - `PaySummaryRequest` carries `vsh_max`, `phie_min`, `swe_max` and `perm_min` and nothing else - so there is no `Min Res Height` whose frame could be ambiguous.
- **UI/IPC/provenance surface:** `ipc.ts` carries the frame on both the request and the row; `core_determinism_tests.rs` `packed_pay_summary` covers both new fields so the re-run hash cannot go blind to them.
- **Named limit:** SandiBumi cannot summate in TVD, TVDSS or TST at all, and this row does not change that. The requirement as written is met - results declare their frame and a non-MD request refuses - but whether the pilot NEEDS a TVD summation for deviated wells is a product decision. REVIEW.md puts it to Jauhar.
- **History/reachability:** no frame field existed; the negative inventory was correct.
- **Blocking decision / next action:** cleared.

### SB-CUT-013

- **Specified contract:** bed amalgamation uses three independent thresholds and defined tie-breaking.
- **Current implementation / as-built:** no bed-amalgamation algorithm or threshold fields exist. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T31 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-2 is the largest evidence gap and blocks tie-breaking; no worked-example inference is allowed.
- **UI/IPC/provenance surface:** absent throughout.
- **History/reachability:** negative inventory confirmed absence.
- **Blocking decision / next action:** obtain the O-2 algorithm or explicit product decision before implementation.

### SB-CUT-014

- **Specified contract:** emit bed statistics before and after amalgamation.
- **Current implementation / as-built:** only zone-level pay totals exist; no bed-statistic blocks exist. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T31 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** depends on SB-CUT-013 and O-2.
- **UI/IPC/provenance surface:** no result/report/office representation.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** define the bed model first, then return both blocks in one typed result.

### SB-CUT-015

- **Specified contract:** state the bed-thickness convention explicitly.
- **Current implementation / as-built:** zone gross/net are reported without any bed-thickness convention. ABSENT.
- **Release disposition and risk:** DEFERRED; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T31 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** sample interval is not a substitute; depends on SB-CUT-013/O-2.
- **UI/IPC/provenance surface:** absent from every output.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** add the convention only with the selected bed model and expose it beside both statistic blocks.

### SB-CUT-016

- **Specified contract:** a fresh project ships with no cutoff values.
- **Current implementation / as-built:** cut-offs are absent-capable end to end. PRESENT-OK.
  `PaySummaryRequest.vsh_max/phie_min/swe_max` are `Option<f64>` (`None` = UNFILTERED) with a new `enabled_unset` list; `PaySummaryRow` gains `unfiltered`; `classify_sample` treats an absent cut-off as not filtering; `run_pay_summary` refuses before any work when `enabled_unset` is non-empty. The same widening reaches `CutoffSweepRequest`, `compute_sweep`'s two held cut-offs, `MonteCarloRequest`, `montecarlo::Cutoffs`, `zone_metrics`, `ReportSpec` and both office specs, **so no surface can disagree with another about whether a property was filtered** - which is the failure `cutoffs.ts`'s own comment records from before centralisation, when MC used 0.08/0.5 against the summary's 0.1/0.6.
- **Where the violation actually lived:** the backend always REQUIRED values and so shipped no default. The three shipped numbers were in the UI - `DEFAULT_CUTOFFS` and `dashboardPanel`'s three seeded literals - and `cutoffDialog` fell back to them whenever a box was blank. `DEFAULT_CUTOFFS` is now all-null and `mergeCutoffs` keeps a SAVED project value or leaves it absent: a user's own default is theirs, and only ours is forbidden.
- **What this row deliberately did NOT change:** the NaN cascade. A sample with no VSH is unjudgeable whether or not VSH is being used as a cut-off. Making an unfiltered cut-off also stop requiring its curve would let a well with no VSH book pay it never booked, and the requirement says nothing about NaN handling - so the rule stands untouched. Recorded here so it is a stated choice rather than an oversight.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** `no_cutoff_ships_a_value_an_unapplied_one_is_reported_unfiltered_and_an_enabled_blank_one_refuses` (`src-tauri/src/workflow.rs`). CORRECTNESS. Five arms: a source scan proving neither UI file seeds any of the forbidden vendor numbers; a fully specified run where the cut-off bites; an unfiltered run where it does not AND the row reports it; **rock that fails every vendor default counting in full when nothing is set**; and an enabled-but-blank cut-off refusing by name.
- **Mutation evidence:** four probes, one per clause, each read for WHICH assertion fired. Re-seeding a UI box turned the source scan red. Emitting an empty `unfiltered` list turned the reporting arm red. Ignoring `enabled_unset` turned the refusal arm red. **The fourth probe initially PASSED and that was a real hole**: substituting `0.5` for an absent VSH went undetected because the first fixture's Vsh 0.40 clears it anyway. The fourth arm exists because of that miss - Vsh 0.80, φ 0.02, Sw 0.95 fail every vendor default, so a fallback drops net from 10 to 0.
- **Regression evidence:** the full backend suite passes with every existing number unchanged. Before the frontend half was removed the suite stood at 1031 passed / 1 failed, the single failure being this test's own source-scan arm - which is what says the semantic refactor moved nothing.
- **Manual evidence:** cutoffs-pay 0/23; field-dashboard 0/10. Automated only.
- **Source/parameter boundary:** NO value adopted; R-1 honoured. The fixture values are chosen to fail every published vendor default precisely so a silent fallback cannot hide behind them.
- **UI/IPC/provenance surface:** `ipc.ts` carries `number | null` on every cut-off and the new `unfiltered` / `enabled_unset`; the PDF, workbook and deck render the word *unfiltered* through one shared `cutoff_label`, and the workbook writes the WORD rather than a blank cell, because a blank reads as no-data.
- **Named limit:** no pane currently SENDS `enabled_unset` - there is no explicit enable toggle in the UI, so a blank box means unfiltered everywhere today. The backend refusal exists and is proven, so a pane that grows a toggle is already guarded. Whether that toggle should exist is a product question; REVIEW.md asks it.
- **History/reachability:** the centralisation commit that created `DEFAULT_CUTOFFS` was reachable but was never source authority, as the register said.
- **Blocking decision / next action:** cleared.

### SB-CUT-017

- **Specified contract:** every actual default carries a source string.
- **Current implementation / as-built:** `DomainDefault {id, value, owner, source, divergence}`, the `CUT_DOMAIN_DEFAULTS` registry and `validate_domain_defaults` in `param_sources.rs`, run at catalog build beside `validate_parameter_sources` so a violation panics the build. `source_identifies_checkable_artefact` becomes `pub(crate)` so both gates apply ONE predicate rather than two spellings that could drift. PRESENT-OK.
- **The design, and why it is the requirement rather than a convention:** the VALUE lives in the registry entry. A shipped default cannot exist without a source because there is nowhere else to put the number - the same structural property `ArgSpec.default_source` already gives module parameters, extended to the defaults that are NOT module parameters. That was this domain's gap: the pay summary is not a module, so `SB-CORE-004`'s gate never reached it.
- **The cut-off half is the empty set.** SB-CUT-016 removed every shipped cut-off value, so nothing about them needs sourcing - there is nothing to defend about a number that is not shipped.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `every_default_this_domain_ships_carries_a_checkable_source_or_declares_its_absence_and_owner` (`src-tauri/src/param_sources.rs`). CORRECTNESS. Six arms: the live registry passes its own gate and is not vacuously small; the registered value IS the shipped constant, so the disclosure cannot drift from the behaviour; a bare product name FAILS, which is the clause that makes it a gate rather than a formality; a default that declares its source absent but says nothing FAILS, because silence is what is being prevented, not absence; a default naming no owner FAILS; and the unsourced entries are asserted to name a requirement and state a real divergence.
- **Mutation evidence:** three probes, each read for WHICH assertion fired. Dropping the checkable-artefact clause let `"Techlog"` pass and turned that arm red. Letting a declared-absent source stay silent turned the disclosure arm red. Changing the shipped `PARTITION_TOLERANCE` to `1e-6` without touching the registry turned the drift arm red - which is what stops the disclosure from becoming decorative.
- **What is DISCLOSED rather than adopted:** two Monte Carlo values - the auto-stop tolerance `0.005` and the reported percentiles `0.10 / 0.90` - are registered with `source: ABSENT`, their owning requirement `SB-CUT-039`, and a statement of what is known. The chapter's parameter table cites IP's auto-stop tolerance at **0.1 %** against SandiBumi's **0.5 %**, a five-fold divergence with no source of its own. **SB-CUT-039 is outside the Gate 2 scope**, so adopting the cited value here would be doing another gate's work and would change when auto-stop fires; inventing a citation for 0.005 was the alternative and is forbidden. Registering it makes the divergence machine-readable instead of a comment nobody greps.
- **Manual evidence:** cutoffs-pay 0/23. Automated only.
- **Source/parameter boundary:** no value adopted. Three entries carry citations their owning rows already established; two declare their absence.
- **UI/IPC/provenance surface:** unchanged. The registry is a build-time contract, not a user surface.
- **History/reachability:** no source-carrying schema existed; the negative inventory was correct.
- **Blocking decision / next action:** cleared.

### SB-CUT-018

- **Specified contract:** every cutoff entry/display surface resolves one authority.
- **Current implementation / as-built:** every cut-off surface resolves from `./cutoffs`. `dashboardPanel.ts` was the last bypass and now imports `loadCutoffDefaults` like the other seven, seeding all four boxes from the project's saved authority - blank where the authority is absent, which after SB-CUT-016 is the fresh-project state. PRESENT-OK.
- **The enumeration DISCOVERS panes rather than listing them,** and that is the substance. A hand-maintained list goes stale the day somebody adds a pane, which is precisely how six copies of two disagreeing sets came to exist. The test scans `src/ui` for any file naming `vsh_max`/`phie_min`/`swe_max` and requires each to import from `./cutoffs`. Two exemptions, explicit and reviewable: `cutoffs.ts`, which IS the authority, and `paramSources.ts`, which names cut-offs as source topics without carrying values.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `every_pane_that_touches_a_cutoff_resolves_it_from_the_one_shared_authority` (`src-tauri/src/param_sources.rs`). CORRECTNESS. It enumerates by discovery, requires the shared import, forbids a hard-coded cut-off literal on any line that names a cut-off, refuses to pass on fewer than eight discovered surfaces, and asserts the Field Dashboard is among them by name - it was the bypass, so a scan that silently stopped seeing it would be the failure repeating itself.
- **A false positive fixed rather than suppressed:** the literal scan first flagged `sweepMaxIn.value = "0.3"` in `cutoffDialog.ts`. That is a sweep RANGE - a plot bound with a legitimate default - not a cut-off. The scan is now scoped to lines that name a cut-off and skips lines mentioning `sweep`, rather than the file being exempted wholesale, which would have created a place to hide a real literal.
- **Mutation evidence:** three probes, each read for WHICH assertion fired. Removing the shared import from the dashboard turned the authority arm red. Re-seeding `sweIn` with `"0.6"` turned the literal arm red, naming the offending line. Making the discovery filter match nothing turned the vacuity guard red at *found 0* - which is what stops the enumeration from passing by seeing nothing.
- **Manual evidence:** field-dashboard 0/10. Automated only.
- **Source/parameter boundary:** no value adopted. The banned literals are exactly the chapter's two documented copy-pasted sets plus the other vendors' published values, registered as FORBIDDEN rather than as candidates.
- **UI/IPC/provenance surface:** all eight surfaces now read one authority; the dashboard's four boxes included.
- **History/reachability:** the shared loader was integrated and is unchanged; only the bypass is gone.
- **Blocking decision / next action:** cleared.

### SB-CUT-019

- **Specified contract:** cutoff entry requires a unit and converts to canonical storage.
- **Current implementation / as-built:** `CutoffQuantity {VolumeFraction, Permeability}` and the `CutoffEntry {value, unit}` wire type in `workflow.rs`, with `CutoffEntry::canonical(quantity, label)` doing the refusing and the converting. Six request shapes carry `Option<CutoffEntry>`; `run_pay_summary` and `run_cutoff_sweep` canonicalise before computing anything, and `montecarlo::validate_cutoffs` is called from the `run_monte_carlo` Tauri command. `cutoff_label` prints the unit beside the number. PRESENT-OK.
- **Why the unit is required rather than assumed:** IP's own manual expresses the sensitivity-sweep example in **porosity units** and the cut-off default in **v/v** *for the same quantity, on a field with no unit tag*. Entering `35` where `0.1` is meant is a **350x** error, and its symptom is an all-net result - a good-looking well, not a visible failure. Nothing downstream can catch it, which is why the entry refuses instead of guessing.
- **A bare number still DESERIALIZES, and that is deliberate.** A custom untagged `Deserialize` turns a bare `f64` into an entry with an empty unit, which then fails `canonical()` with a message naming the field and explaining the trap. Refusing at the parse layer would have returned serde's *invalid type: floating point* text, which tells an analyst nothing about porosity units, and would have broken every request shape written before this existed. The value is rejected either way; this only controls **which message they get**.
- **Where the Monte Carlo refusal had to live:** the job registry fixes `run_monte_carlo`'s return type, so a refusal inside it has nowhere to go. `validate_cutoffs` is therefore called from the Tauri command, before the job is registered - the same shape the registry already imposes on every other pre-flight check.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_cutoff_is_refused_without_a_unit_and_thirty_five_is_porosity_units_or_out_of_bounds` (`src-tauri/src/workflow.rs`). CORRECTNESS. Six arms: a bare number is refused **for the missing unit rather than for the value**, proven with `0.10` - a perfectly ordinary v/v porosity cut-off that a range check alone would pass; the chapter's own `35` carries a message that states the size of the error it prevents; `35 pu` is 0.35 v/v; `35 v/v` is refused as out of bounds - **the same number as the previous arm, opposite verdict, and only the unit separates them**, which is the requirement in one pair of assertions; both bounds are the quantity's own; permeability has its own unit family and its own bound, so the rule is a property of the QUANTITY and not a hard-coded 0..1; and the summation itself refuses a bare cut-off before computing, because a refusal that exists only in a helper is not a contract.
- **A hole the mutations found, and closed.** The first `nounit` probe (assume the canonical unit instead of refusing) fired on the *message* clause, not on the presence of a refusal - because a bare `35` happens to be out of range in `v/v` anyway. So the arm was proving less than it read as: a bare `0.10` would have been accepted silently. The arm now leads with `0.10`, and the re-run probe fires on that `expect_err` directly.
- **Mutation evidence:** four probes, each read for WHICH assertion fired, and all four at distinct lines. Assuming the canonical unit for a blank one turned the bare-number arm red. Making `pu` an identity conversion turned the `35 pu` arm red. Removing the volume-fraction bound turned the `35 v/v` arm red. Leaving the helper intact but having `run_pay_summary` read `.value` directly turned the wired-in arm red - which is the one that stops the refusal from being a helper nobody calls.
- **An unvalidated constructor removed rather than kept:** an `in_canonical` helper for replaying an already-validated spec had no caller. Unused code that can build a `CutoffEntry` bypassing `canonical()` is a back door into the type this row exists to guard, so it is gone rather than annotated.
- **Manual evidence:** cutoffs-pay 0/23. Automated only.
- **Source/parameter boundary:** no value adopted. The unit vocabulary is a vocabulary; the two bounds are the physical range of a volume fraction and of a permeability.
- **UI/IPC/provenance surface:** nine frontend surfaces send `{value, unit}`; `cutoffs.ts` `CUTOFF_UNITS` is the single place a slot's unit is named. The provenance record and the report label were **tightened** to require the unit, not relaxed to tolerate it.
- **Named limit:** the panes send a fixed unit per slot (v/v for VSH/PHIE/SWE, mD for PERM) rather than offering a selector, so today the refusal is reachable from a saved or scripted request rather than by typing. Adding a unit selector to the entry boxes is a product decision and is Jauhar's, not this row's.
- **History/reachability:** the negative inventory was correct; no canonical conversion layer existed.
- **Blocking decision / next action:** cleared. The register's blocking note - *quantity labels are not unit custody* - was exactly right, and the label is now beside a real unit rather than standing in for one.

### SB-CUT-020

- **Specified contract:** a cutoff is a two-sided range with an explicit inclusive/exclusive bounds operator.
- **Current implementation / as-built:** `BoundOperator {Inclusive, Exclusive}`, `CutoffBound`, `CutoffRange` with its `contains`, `CutoffSense` and the `CutoffSpec` wire type in `workflow.rs`. Six request shapes carry `Option<CutoffSpec>`; `classify_sample`, `compute_sweep` and `montecarlo::zone_metrics` all compare through `CutoffRange::contains`, which is now the only place in the product where a cut-off comparison is made. PRESENT-OK.
- **The specification is OURS, and that is the requirement rather than a convenience.** SB-CUT-T24's oracle is *"SandiBumi's own written specification"* because Techlog documents its modes 2 and 3 as outside tests and implements them as inside tests. So the spec lives in `CutoffRange`'s doc comment as a four-row table - side, operator, passes-when, and what happens to a sample exactly on the bound - and the test asserts that table line by line. A boundary convention not tested against its own spec is a coin flip at every sample sitting exactly on the cut-off, which is the population that decides a marginal-pay result.
- **The single-sided forms are the DEGENERATE case, not a parallel mechanism.** An absent bound is an open far bound; `INCLUSIVE` is the default on both sides because that is what `>=` and `<=` have always meant here. A `{value, unit}` object still crosses the wire unchanged and still classifies every sample exactly as it did, so no pane changed and no saved request moved.
- **The comparison happens in `f32`, and that turned out to be load-bearing.** A continuous log is `f32` (rule 2) while a cut-off is entered as a decimal and held as `f64`. Widening the sample instead makes `0.30f32` into `0.30000001192...`, which is strictly GREATER than `0.30f64` - so a sample the user typed `0.30` to sit exactly on never sits on it, and the EXCLUSIVE operator silently excludes **nothing at all**. That is Techlog's mode 7 - a configured cut-off that filters nothing - arrived at by arithmetic rather than by a bug. It was found by the wired-in arm returning identical net for both operators, and it is now pinned from both sides with the premise (`0.30f32 as f64 > 0.30f64`) asserted beside it.
- **An empty window is REFUSED rather than run.** A range whose low bound exceeds its high, or whose bounds meet with either side exclusive, admits no value at all - and the zero net it would book is indistinguishable on the page from tight rock. The refusal names the cut-off.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** `a_sample_exactly_on_a_cutoff_bound_is_included_or_excluded_by_that_bounds_own_declared_operator` (`src-tauri/src/workflow.rs`). CORRECTNESS. Seven arms: the specification table asserted line by line at exactly the bound and away from it; the `f32` precision rule with its premise; an absent bound admitting everything; the degenerate wire form landing inclusively on the slot's own side with the far side open; a genuine two-sided window with a different operator on each side, its units canonicalised per bound; an inverted or touching window refused by name; and the wired-in pair where the SAME well and the SAME number book different net on the operator alone.
- **Mutation evidence:** seven probes over five distinct assertions, each read for WHICH assertion fired. Making the exclusive low bound inclusive turned the `x > min` arm red; the same for the high bound. Comparing in `f64` turned that same low-bound arm red - it shares an assertion with the previous probe rather than proving a separate clause, and both establish that the exclusive-low comparison is load-bearing at the boundary. Making an absent bound reject everything turned the open-far-bound arm red. Landing the degenerate form on the wrong side of the range turned the degenerate arm red, as did restoring the widest-first wire ordering - again a shared assertion, both proving the degenerate form must land inclusively on its slot's own side. Disabling the empty-window refusal turned the refusal arm red.
- **An ordering hazard caught in development, not hypothetically.** Both of the wire `Range` arm's fields are optional, so `#[serde(untagged)]` matches it against ANY object - including `{value, unit}`, which it silently accepted as a range with **no bounds at all**. The first run of the degenerate arm caught exactly that. `Single` is now tried first, with the reason written where the next person will read it.
- **Serialization is the inverse of deserialization**, so a persisted run reloads as the cut-off it was: a degenerate spec writes the object it arrived as, now carrying its operator, and a range writes `{min, max}` with an absent side omitted rather than nulled. The existing provenance assertion was **tightened** to require the operator alongside the unit, never relaxed.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value adopted. The four operator behaviours are the definition of a half-open interval, and the choice that could have been a decision - which side the existing single-sided forms map to - is fixed by the requirement itself ("the single-sided forms MUST be the degenerate case").
- **UI/IPC/provenance surface:** `ipc.ts` gains `BoundOperator`, `CutoffSpecBound` and `CutoffSpec`; every cut-off request field widens to accept either form. The workbook writes a two-sided range as TWO rows, one per bound, each naming its own operator, so the numbers-stay-numbers rule survives a range; `cutoff_label` prints interval notation where the bracket IS the operator; and a new `cutoff_phrase` replaced three hard-coded `>=` fragments in the report and deck prose, which a two-sided or exclusive cut-off would have made untrue.
- **Named limit:** no pane offers an operator control yet - the cut-off boxes still send the degenerate inclusive form, so the operator is reachable from a saved or scripted request rather than by clicking. Whether the entry boxes should grow a min/max pair and an inclusivity toggle is a product decision and is Jauhar's, not this row's.
- **History/reachability:** the negative inventory was correct; the three cut-offs were fixed single-sided comparisons and no operator existed anywhere.
- **Blocking decision / next action:** cleared.

### SB-CUT-021

- **Specified contract:** a cutoff may be a per-sample curve rather than only a scalar.
- **Current implementation / as-built:** pay and MC cutoff fields are scalar only. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T24 has no curve-valued limb.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** no default curve is implied.
- **UI/IPC/provenance surface:** no curve identity or resolver exists for a cutoff operand.
- **History/reachability:** negative inventory confirmed absence.
- **Blocking decision / next action:** if selected, extend the typed cutoff operand to scalar-or-curve while preserving units and bounds semantics.

### SB-CUT-022

- **Specified contract:** cutoff activation is explicit, and reservoir/pay tiers share the same value when both use it.
- **Current implementation / as-built:** `CutoffUse {sand, reservoir, pay}`, `default_cutoff_use`, `cutoff_use_for` and the run-scoped `TierCutoffs` in `workflow.rs`; `PaySummaryRequest.cutoff_use` carries the declaration and is persisted with the rest of the run's configuration. `classify_sample` is now tier-driven: each tier applies exactly the cut-offs declared for it. PRESENT-OK.
- **One value, two independent flags - F-25's shape exactly.** IP ships `Phi Cutoff` as a single ordinal described as *"Porosity cutoff value for Pay and Reservoir report"*, with `Phi Net Use` and `Phi Pay Use` as two separate ordinals beside it. `TierCutoffs` holds one value per property and a `CutoffUse` per property; turning a cut-off off at the reservoir tier moves that tier's net and leaves the pay tier's untouched, and vice versa.
- **Why it has to be a flag rather than an inference:** F-17. Geolog changed the activation trigger between two modules of ONE product - `Determin` fires on the presence of the *curve*, `determin_mc` on the presence of the *value*. Either rule is defensible; what is not defensible is that a result cannot say which one applied. `cutoff_use_for` takes a SLOT and the run's declaration and nothing else - it cannot see a curve or a value - so *never inferred* is a property of the signature rather than of today's body.
- **The ladder moved from nesting to declaration without moving a number.** `classify_sample` used to build reservoir on sand and pay on reservoir. It now evaluates each tier against its own declared set, and the shipped defaults say the same thing: VSH at all three tiers, PHIE at reservoir and pay, SWE and PERM at pay only. All 1,038 existing backend tests passed unchanged, which is the evidence that the restatement is behaviour-preserving.
- **`SWE` is OFF at the reservoir tier by default**, which is F-25 `:494-495` - IP's `Sw Net Use` and `Sw Pay Use` are separate ordinals and Net Reservoir is described as porosity- and clay-driven. That is also SB-CUT-026's whole subject, and this row supplies the mechanism that row will be tested through.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `each_cutoff_declares_the_tiers_it_is_used_at_and_reservoir_and_pay_share_one_value_with_independent_flags` (`src-tauri/src/workflow.rs`). CORRECTNESS. Four arms: the shipped defaults are the ladder, asserted slot by slot with SWE's reservoir-off state called out; the same `phie_min` value read by two tiers, disabled at each in turn, with the OTHER tier asserted not to move; activation resolved from the slot and the declaration alone, with an undeclared slot taking its own documented default rather than its neighbour's declaration; and a cut-off disabled at every tier booking exactly what an absent cut-off books - two different statements about intent that must be the same statement about rock.
- **Mutation evidence:** five probes over four distinct assertions, each read for WHICH assertion fired. Defaulting `SWE` on at the reservoir tier turned the F-25 arm red. Dropping `PHIE` from the pay tier turned the defaults arm red. Making `CutoffUse::at` answer the same for every tier turned the independence arm red. Inferring activation from the presence of a value, and ignoring the caller's declaration entirely, both turned the one-value-two-flags arm red - they share an assertion rather than proving separate clauses, and both establish that the declaration is what decides.
- **Deliberately unchanged: the NaN cascade.** A missing VSH still makes all three tiers unjudgeable, a missing PHIE the reservoir and pay tiers, a missing SWE the pay tier - regardless of any use flag. The flags govern whether a cut-off's VALUE is applied at a tier; whether the tier needs that curve to be judgeable at all is a different question and not a cut-off. This is SB-CUT-016's decision and it is untouched: making an unfiltered cut-off also stop requiring its curve would let a well with no VSH book pay it never booked.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value adopted. The default ladder is the one the engine already applied, restated; its literature backing is T4 Bentley & Ringrose's net sand to net reservoir to net pay (`:1296-1297`) and its Sw clause is F-25.
- **UI/IPC/provenance surface:** `ipc.ts` gains `CutoffUse` and an optional `cutoff_use` map on the pay-summary request. The declaration rides in `log_sets.params_json` with the rest of the run configuration, which is what makes the activation auditable FROM A RESULT rather than re-derivable only by knowing which rule the engine happened to apply.
- **Scope boundary, stated rather than assumed:** the requirement's clause that *additional report tiers MUST each carry their own value* belongs to **SB-CUT-024**, which owns arbitrary named flag tiers over arbitrary cut-off sets and is **outside the Gate 2 manifest**. SandiBumi ships exactly three tiers; reservoir and pay share one value as required, and the third shares it too because there is no mechanism for a tier to carry its own until that row lands. Building one here would be doing an out-of-scope row's work and would change what a SAND flag means.
- **Named limit:** no pane offers the per-tier flags yet - the cut-off panes send no declaration and therefore get the shipped ladder, unchanged. Whether the Cutoffs pane should grow a use matrix is a product decision and is Jauhar's.
- **History/reachability:** the as-built was accurate. The PERM cut-off already had an explicit inactive state and its own `perm_cutoff_no_data` flag; VSH, PHIE and SWE were always active and no per-tier flags existed anywhere.
- **Blocking decision / next action:** cleared.

### SB-CUT-023

- **Specified contract:** cutoff criteria form a boolean expression with AND, OR and parentheses.
- **Current implementation / as-built:** classify_sample uses a fixed cascade; the separate net-flag polygon is not a summary expression engine. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T32 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** the chapter supplies the worked expression; no parser grammar is invented here.
- **UI/IPC/provenance surface:** no expression AST or serialized expression exists.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, define a typed expression tree and prove AND/OR precedence with the cited strict-subset control.

### SB-CUT-024

- **Specified contract:** arbitrary named flag tiers can use arbitrary cutoff sets.
- **Current implementation / as-built:** SUMMARY_FLAGS is fixed to SAND, RESERVOIR and PAY over the fixed quartet. PARTIAL.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T36 does not exercise arbitrary tiers.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** names and memberships are user/project configuration, not defaults.
- **UI/IPC/provenance surface:** request/result/report schemas are fixed to three identities.
- **History/reachability:** fixed tiers are integrated.
- **Blocking decision / next action:** decide whether arbitrary tiers enter the pilot, then replace fixed arrays with typed tier definitions.

### SB-CUT-025

- **Specified contract:** lumps are a many-to-one reporting transform over immutable flags.
- **Current implementation / as-built:** no lump schema or transform exists. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T31 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** lumps must not mutate flags or be inferred as bed merging.
- **UI/IPC/provenance surface:** absent throughout.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, define reporting-only grouping after flag/tier architecture is explicit.

### SB-CUT-026

- **Specified contract:** saturation is disabled at reservoir tier by default while pay still applies saturation.
- **Current implementation / as-built:** unchanged and correct. `default_cutoff_use("SWE")` ships `{sand: false, reservoir: false, pay: true}` (SB-CUT-022), so `FLAG_RESERVOIR` applies VSH and PHIE and nothing else, and saturation enters at the pay tier. PRESENT-OK.
- **This is a PROVE row: nothing was implemented, only proved.** The behaviour was already right; what was missing was a test that would notice if it stopped being right. The whole of the evidence is therefore the mutation record below.
- **Why P1 rather than a preference.** The chapter states the consequence outright: getting it wrong **reclassifies wet reservoir as non-reservoir**. A water-bearing sand is still reservoir rock - it is the pay tier that is entitled to care that it is wet. F-25: IP's `Sw Net Use` and `Sw Pay Use` are separate ordinals and Net Reservoir is described as porosity- and clay-driven. Ledger D-5.10 records it as a default the vendor never states plainly.
- **Release disposition and risk:** PILOT-BLOCKER; FIELD-EVIDENCE.
- **Automated evidence:** `a_wet_but_porous_clean_sand_is_reservoir_and_not_pay_because_saturation_enters_at_the_pay_tier` (`src-tauri/src/workflow.rs`). CORRECTNESS. Five arms over a purpose-built fixture - clean at VSH 0.10, porous at PHIE 0.30 and WET at SWE 0.80, which is exactly the rock the row protects. The wet sand books as reservoir IN FULL and as pay NOT AT ALL; the reservoir tier does not move when the saturation cut-off's value moves, while pay does, which distinguishes *does not apply* from *applies but happens not to bite on this fixture*; the reservoir tier IS clay-driven, pinned from the positive side so that a tier applying NOTHING cannot pass; an explicit declaration still reaches the reservoir tier, because the requirement forbids it BY DEFAULT and not at all; and the default is readable off the configuration rather than inferred from a result.
- **The evidence register carried the wrong test, and it has been replaced rather than joined.** `classify_sample_nan_propagation` was registered as this row's proof with class `CHARACTERIZATION`. It is a real test and it stays in the suite, but it pins NaN propagation, not the tier the saturation cut-off enters at - it would pass unchanged with saturation applied at the reservoir tier, which is the entire defect. It is a supporting test, not a qualifying one, and a requirement's registered rows carry the requirement's class.
- **Mutation evidence:** three probes, each read for WHICH assertion fired, all three at distinct assertions. Defaulting `SWE` on at the reservoir tier turned the wet-sand arm red - that is the defect itself, reproduced and caught. Turning `VSH` off at the reservoir tier turned the clay-driven arm red, which is the guard against satisfying this row with a tier that filters nothing. Making `cutoff_use_for` ignore the caller's declaration turned the default-not-prohibition arm red.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed. The risk class is FIELD-EVIDENCE, and this row does not claim to have supplied any: what a synthetic wet sand proves is that the tier rule holds, not that the rule is right for a particular reservoir.
- **Source/parameter boundary:** no value adopted. The cut-off values in the fixture are test inputs chosen to straddle the tiers, not shipped defaults - there are none, since SB-CUT-016.
- **UI/IPC/provenance surface:** unchanged. The default is `default_cutoff_use`, which SB-CUT-022 made declarable per run and persists with the run configuration.
- **History/reachability:** the as-built was accurate; `FLAG_RESERVOIR` applied VSH and PHIE only, and still does.
- **Blocking decision / next action:** cleared.

### SB-CUT-027

- **Specified contract:** impose no arbitrary cap on curves, cutoffs, report tiers or flags.
- **Current implementation / as-built:** no cap exists. PRESENT-OK, following the chapter.
- **The register and the chapter disagreed, and the chapter wins.** The register read the four cut-off fields and the three `SUMMARY_FLAGS` entries as a *"fixed four-cutoff/three-tier schema"* and called the row PRESENT-DIVERGENT. The chapter's own as-built says PRESENT-OK - *"no cap exists, because the surface is not yet general enough to have one. This becomes a live constraint when `SB-CUT-024` is built."* Both read the same code; they differ on whether a fixed schema IS a cap. It is not, and the distinction is the whole row: **a fixed ARITY is not a maximum.** Four cut-off fields exist because four quantities are cut on, and three tiers because three are emitted - neither is a budget a user can exhaust. IP's Curve 10 is a cap because it limits a collection that would otherwise grow.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `a_run_carries_more_curves_than_any_vendor_cap_and_the_fixed_cutoff_and_tier_counts_are_arities_not_maxima` (`src-tauri/src/workflow.rs`). CORRECTNESS. Four arms: **sixty curves on one well resolved in ONE frame**, chosen to clear both vendor numbers the ledger records - past IP's Curve 10 and past its 2025 prose's 50 - with each curve asserted to carry its own values, because a silent truncation IS a cap and a silently reused column would hide one; the four cut-off fields exercised as an arity, with a run using none of them and a run using all of them both valid, so the count is not a resource anything competes for; the emitted tiers asserted to be exactly the declared ones, which is what makes the tier count DATA rather than a `3` written down in several places; and a scan of the engine for a maximum expressed on a COUNT of curves, cut-offs, tiers or flags.
- **The clamps this domain DOES carry are named rather than exempted silently.** Iterations and sweep steps are clamped, and those are compute budgets on a loop - a limit on how long a study runs, not on how much rock it may describe. Saying so in the test is what stops the scan from being quietly widened later to cover a real cap.
- **Mutation evidence:** two probes, each read for WHICH assertion fired, both at distinct assertions. Dropping a tier from the row emission turned the declared-equals-emitted arm red. Introducing a vendor-style `.take(10)` into the engine turned the no-cap scan red - and that probe is deliberately a semantic NO-OP, since three tiers never reach a cap of ten, so **only** the scan can catch it. That is the arm's whole purpose: to catch a cap before it is large enough to bite.
- **NOT mutated, and stated rather than glossed:** the sixty-curve arm. It resolves through `equations::fetch_curve_frame`, which lives in a protected file, so no mutation introducing a truncation there was available to this lane. The arm is a real regression lock - a cap added later would fail it - but its mutation is not available here.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value adopted, and deliberately no replacement maximum invented. The number sixty is a test input chosen to clear both cited vendor numbers, not a new limit.
- **UI/IPC/provenance surface:** unchanged.
- **Scope boundary, stated rather than assumed:** the general case - arbitrary named tiers over arbitrary cut-off sets - is **SB-CUT-024**, which is **outside the Gate 2 manifest**, and the chapter says so in this row's own as-built: the constraint becomes live when that row is built. Generalising the tier list here would be doing an out-of-scope row's work.
- **History/reachability:** dynamic curve infrastructure and the fixed cut-off/tier schema are both integrated, exactly as the register said; the reading of what they mean is what changed.
- **Blocking decision / next action:** cleared.

### SB-CUT-028

- **Specified contract:** emitted saturation identities are SWE or SWT, never bare SW.
- **Current implementation / as-built:** unchanged and correct. `swe_max`, `avg_swe` and explicit `SWE` / `SWT` outputs throughout; the bare `SW` in `contacts` is an input alias, not an emitted result. PRESENT-OK.
- **This is a PROVE row: nothing implemented, only proved.** The register's own note was exactly right and is worth keeping - a module/output parity test *"would also pass if both manifest and output added bare `SW`"*, so parity does not pin a negative registry contract. What was needed was a test that forbids the bare name AND demonstrates the explicit ones, from both sides.
- **The chapter's `Verified by` points at the wrong test, and that is recorded rather than worked around.** SB-CUT-028 cites SB-CUT-T06, but the chapter's own test-to-requirement map assigns T06 to **SB-CUT-009**, and T06 as tabulated pins the phi-weighted average-form identity. That is the CONSEQUENCE of the naming, not the naming: it is why the row is P1, because in Techlog *"the SW curve is weighted by POR but the SWE is not"* - so the ambiguity does not stay an ambiguity, it becomes a different number. The naming contract needed its own test.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** `no_module_output_cutoff_field_or_result_field_is_a_bare_sw_rather_than_swe_or_swt` (`src-tauri/src/workflow.rs`). CORRECTNESS. Four arms: the module registry scanned from BOTH sides - no shipping module emits a bare `SW`, AND some module does emit `SWE` and `SWT`, with a floor on the number of outputs seen so a scan that stopped finding anything cannot pass; the pay-summary result row serialized and its field names checked, because the row outlives the run that made it; the cut-off record round-tripped from the wire, where a bare `sw_max` must NOT be read as the saturation cut-off; and the exemption stated where it belongs.
- **The exemption is NAMED and narrow rather than discovered later as a false positive.** A bare `SW` may appear as an INPUT, because an input names the user's own curve and the requirement governs cut-off records and result fields. What it may never be is an OUTPUT - and the test asserts that a module reading a curve called `SW` does not also emit one, so the boundary is written down instead of being rediscovered by whoever next widens the scan.
- **Mutation evidence:** three probes, each read for WHICH assertion fired, all three at distinct assertions. Renaming a shipping saturation output to bare `SW` turned the registry arm red. Renaming the serialized result field to `avg_sw` turned the result-field arm red. Adding `#[serde(alias = "sw_max")]` to the saturation cut-off turned the cut-off-record arm red - and that one is the subtlest of the three, because an alias is exactly how a bare name gets accepted "for compatibility" without anyone deciding to accept it.
- **Manual evidence:** cutoffs-pay 0/23. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** identity convention only; no numeric parameter, and none invented.
- **UI/IPC/provenance surface:** unchanged. The wire field is `swe_max`, the result field `avg_swe`, and the module outputs `SWE` / `SWT`.
- **History/reachability:** explicit outputs were already integrated; the negative contract had no test.
- **Blocking decision / next action:** cleared.

### SB-CUT-029

- **Specified contract:** null/absence states are carried as typed sibling fields, not inferred from numeric zeros.
- **Current implementation / as-built:** n_classified and perm_cutoff_no_data distinguish two important absences, but other footage/result null states have no typed siblings. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** CHARACTERIZATION; pay_summary_marks_an_uninterpreted_well_as_classifying_nothing, a_well_with_no_perm_fails_the_cutoff_and_says_why, and the rendered zero-versus-absent frontend test prove two limbs only.
- **Manual evidence:** NONE; Results QC is 0/1.
- **Source/parameter boundary:** no null sentinel value is adopted; missing remains NaN plus typed context.
- **UI/IPC/provenance surface:** summary UI honors n_classified, but full result/report/office null custody is incomplete.
- **History/reachability:** both sibling fields and observable rendering regression are integrated.
- **Blocking decision / next action:** inventory every nullable result and add typed reason/status siblings across IPC and exports.

### SB-CUT-030

- **Specified contract:** accumulation, flag testing and presentation clamps are distinct and declared.
- **Current implementation / as-built:** floored_phie is shared by deterministic summary/sweep, module limited/unlimited curves coexist, and MC plausibility inspects unlimited companions; however stage identity is not explicit and MC accumulates limited curves. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** CHARACTERIZATION; flooring_phie_leaves_missing_missing and impossible-combination tests prove isolated guards, not the three-stage contract. T15/T23/T25 are absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** the existing PHIE floor is a recorded product decision; R-4 still forbids pre-accumulation clamp in MC.
- **UI/IPC/provenance surface:** requests/results do not name stage or clamp policy.
- **History/reachability:** scattered clamp behavior is integrated.
- **Blocking decision / next action:** model stage policy explicitly and route MC accumulation from unlimited scientific outputs while retaining separate flag/presentation rules.

### SB-CUT-031

- **Specified contract:** shift width carries an explicit mandatory sigma multiple, with SD_MULT=2 for IP-sourced widths.
- **Current implementation / as-built:** IP_MC_SEEDS passes each vendor width directly to the normal distribution as sigma; the cited m width 0.2 therefore becomes 0.20 instead of 0.10. PRESENT-DIVERGENT.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** MISSING; T13 is absent.
- **Manual evidence:** NONE; E-1 confirms the bad width did not reach a delivered uncertainty band, but that is not implementation proof.
- **Source/parameter boundary:** SD_MULT=2 and the m verification row are cited; vendor widths remain NON-ADOPTABLE defaults.
- **UI/IPC/provenance surface:** McSeed stores w/pct only; McParam and McResult do not record source width or sigma multiple.
- **History/reachability:** direct-width implementation is integrated; E-1 is prospective exposure adjudication only.
- **Blocking decision / next action:** require SD_MULT on imported widths, calculate sigma=w/2, record it, and pin 0.10 versus 0.20 from both sides.

### SB-CUT-032

- **Specified contract:** store shift type with width and refuse reciprocal Rec input when the sampler cannot represent it.
- **Current implementation / as-built:** McSeed has only pct boolean versus absolute; Distribution has linear normal/uniform/triangular forms and no reciprocal type or importer refusal. ABSENT.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T20 and T30 are absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-10 leaves Rec behavior near zero open; R-9 forbids coercion to Linear.
- **UI/IPC/provenance surface:** no typed shift enum crosses import, editor, request, result or persistence.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** add a shift-type enum and refuse unsupported Rec at load before any draw occurs.

### SB-CUT-033

- **Specified contract:** import measurement priors as well as model-parameter priors.
- **Current implementation / as-built:** the UI lists numeric module parameters only and no structured prior importer exists. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T20 has no importer body.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** E-4 forbids inventing a house Rw prior; imported priors require their own source and units.
- **UI/IPC/provenance surface:** no measurement-prior identity or source record exists.
- **History/reachability:** negative inventory confirmed absence.
- **Blocking decision / next action:** product-select the importer, then distinguish measurement and parameter priors without supplying any new default.

### SB-CUT-034

- **Specified contract:** seed is mandatory and appears on the returned/reportable result record.
- **Current implementation / as-built:** McRequest requires seed and persisted log-set parameters include it when persistence is enabled; McResult itself has no seed field. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** CHARACTERIZATION; hpv_distribution_is_ordered_and_reproducible and draw-matrix checks prove same-seed determinism, but T12 does not observe seed on the result.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** seed is a mandatory run identity, not a default value.
- **UI/IPC/provenance surface:** editor sends seed; non-persisted result, report and ordinary job surface cannot read it back.
- **History/reachability:** request/persistence custody is integrated.
- **Blocking decision / next action:** echo seed in McResult and every report/job identity, then prove same/different seeds and returned custody together.

### SB-CUT-035

- **Specified contract:** provide log-domain probability distributions.
- **Current implementation / as-built:** Distribution supports normal, uniform and triangular only on the linear variable. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T14 does not exercise a log-domain distribution.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** no distribution is chosen as a default; O-8 preserves vendor conflict.
- **UI/IPC/provenance surface:** editor and serde enum have no log-domain variant.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, add explicit log-domain variants with typed base/unit custody and independent quantile fixtures.

### SB-CUT-036

- **Specified contract:** every prior carries value, basis, sigma multiple and units.
- **Current implementation / as-built:** Distribution carries numeric parameters and McParam carries name/zone; McSeed carries width and pct, but basis, sigma multiple, units, source and shift type are incomplete. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T13 and T21 are absent as triple-and-unit custody tests.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** 11 vendor rows are NON-ADOPTABLE; E-3 keeps B unit null and E-4 keeps Rw prior absent; R-11 forbids filling either gap.
- **UI/IPC/provenance surface:** editor labels some module units but request/result/persistence do not carry the full prior record.
- **History/reachability:** partial numeric distribution schema is integrated.
- **Blocking decision / next action:** define one prior record with nullable sourced unit and explicit basis/SD_MULT; preserve unknown units as unknown.

### SB-CUT-037

- **Specified contract:** each prior stores its centring rule.
- **Current implementation / as-built:** normal implies mean, uniform midpoint and triangular mode from field position, but no centring field is stored. ABSENT.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T28 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-4 keeps asymmetric-Gaussian centring unresolved; a typed field can represent the future decision without guessing it.
- **UI/IPC/provenance surface:** no centring rule appears in editor, request, result, persisted params or report.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** add a required centring enum and use the cited triangular fixture without resolving O-4 by assumption.

### SB-CUT-038

- **Specified contract:** Gaussian draws are truncated at the cited boundary and the realised variance deficit is reported.
- **Current implementation / as-built:** normal draws use Box-Muller without truncation and no variance-deficit result exists. ABSENT.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** MISSING; T14 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** the 2.5-sigma truncation rule is cited; no corrective variance factor is invented.
- **UI/IPC/provenance surface:** request and result carry neither truncation nor deficit.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** implement explicit truncated draws and report measured variance loss, then pin both maximum deviation and deficit.

### SB-CUT-039

- **Specified contract:** use the cited 2000-iteration default and auto-stop on the percentile actually reported.
- **Current implementation / as-built:** UI defaults to 1000; convergence is opt-in, uses an uncited default tolerance, and LHS never truncates. PRESENT-DIVERGENT.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** CHARACTERIZATION; convergence_early_stops_on_a_stationary_series and lhs_design_never_truncates prove current mechanics, not T33's cited default and clear/marginal cases.
- **Manual evidence:** NONE; Monte Carlo is 2/14 without this scenario.
- **Source/parameter boundary:** 2000 is cited; O-7 leaves iteration ceiling open and current 0.005 is not promoted to authority.
- **UI/IPC/provenance surface:** request/result record requested/used counts and traces, but fresh UI state and stopping policy diverge.
- **History/reachability:** convergence implementation is integrated.
- **Blocking decision / next action:** set only the cited default, make the stopping target explicit, and add T33 without inventing an iteration ceiling.

### SB-CUT-040

- **Specified contract:** one offset is drawn per section per iteration, constant vertically within that section.
- **Current implementation / as-built:** build_draws creates one draw per McParam per realization and zone spans apply that scalar over the contiguous section. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; FIELD-EVIDENCE.
- **Automated evidence:** CHARACTERIZATION; zone_scoped_param_only_moves_its_zone proves scoping, but T27's vertical-versus-horizontal spread and N=100 ratio are absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** VERTICAL is cited; R-6 forbids per-depth draws for summation inputs.
- **UI/IPC/provenance surface:** McParam.zone records section name but result does not declare draw regime.
- **History/reachability:** scalar-per-param realization behavior is integrated.
- **Blocking decision / next action:** expose the draw regime and add the independent vertical/horizontal variance fixture.

### SB-CUT-041

- **Specified contract:** never clamp scientific values before volumetric accumulation.
- **Current implementation / as-built:** run_realization feeds module limited PHIE/SWE outputs into zone_metrics; unlimited companions are inspected only for plausibility. PRESENT-DIVERGENT.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** MISSING; T15, T23 and T25 are absent as accumulator tests. Impossible-combination tests do not prove unbiased accumulation.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** no-clamp is specified and R-4 binding; T25 supplies the independent negative bias expression.
- **UI/IPC/provenance surface:** result does not say whether accumulated curves were limited.
- **History/reachability:** limited-curve accumulation is integrated.
- **Blocking decision / next action:** route accumulation from unlimited companions with explicit missing/physical diagnostics, preserving separate display clamps.

### SB-CUT-042

- **Specified contract:** perturb cutoff values independently per zone during Monte Carlo.
- **Current implementation / as-built:** Cutoffs is one scalar quartet for the request; McParam zone scoping varies module parameters only. ABSENT.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T18 has no cutoff-draw body.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** cutoff priors ship absent and IP cutoff widths remain NON-ADOPTABLE verification data.
- **UI/IPC/provenance surface:** no cutoff-prior rows, zone draw identity or realised values are returned.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, extend the cited prior schema to cutoff-by-zone without creating defaults.

### SB-CUT-043

- **Specified contract:** derived ratios are calculated inside each realization before percentile summarization.
- **Current implementation / as-built:** zone_metrics calculates ntg per realization and summarize operates on the resulting ntg vector. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** MISSING; code shape is not executable evidence, and current distribution tests do not exercise T19's varying-gross ratio-of-percentiles discriminator.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** independent arithmetic is the oracle; R-8 forbids ratio-of-marginal-percentiles.
- **UI/IPC/provenance surface:** McZoneResult exposes ntg percentiles but not calculation method.
- **History/reachability:** inside-iteration ratio is integrated.
- **Blocking decision / next action:** add the varying-gross fixture that fails any ratio-of-percentiles implementation.

### SB-CUT-044

- **Specified contract:** store per-iteration joint records and expose iteration-consistent percentile cases.
- **Current implementation / as-built:** per_real holds joint MetricSet values transiently, then McResult returns only marginal summaries and optional per-sample matrices; no labelled joint case record exists. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T16 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** the marginal-versus-joint fixture is specified; no reserves probability mapping is inferred.
- **UI/IPC/provenance surface:** result, report and office outputs cannot recover which realization supplies a percentile case.
- **History/reachability:** transient joint vectors are integrated.
- **Blocking decision / next action:** return stable per-realization/case identity and prove joint P50 differs from the product of marginals.

### SB-CUT-045

- **Specified contract:** withhold a statistic whose sample-size or other precondition fails and emit a machine-readable reason.
- **Current implementation / as-built:** summarize emits interpolated tails for any non-empty finite vector, including five realizations; no refusal reason is returned. PRESENT-DIVERGENT.
- **Release disposition and risk:** PILOT-BLOCKER; DEGRADED-RESULT.
- **Automated evidence:** MISSING; T29 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** refusal precedent is cited; R-7 forbids emitting unsupported statistics.
- **UI/IPC/provenance surface:** Pctl fields have no availability/reason sibling.
- **History/reachability:** unconditional summarization is integrated.
- **Blocking decision / next action:** add per-statistic availability and reason fields and pin the five-iteration refusal.

### SB-CUT-046

- **Specified contract:** name the percentile interpolation method on the output record.
- **Current implementation / as-built:** percentile implements linear type-7 arithmetic internally, but McResult/Pctl do not identify the method. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; helper inspection is not an executable output-record assertion, and T16 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** type-7 is the explicit SandiBumi specification, not a guessed vendor method.
- **UI/IPC/provenance surface:** method is absent from result, persistence, report and office export.
- **History/reachability:** arithmetic is integrated.
- **Blocking decision / next action:** add an immutable percentile_method field and assert it beside the joint case record.

### SB-CUT-047

- **Specified contract:** percentile cases use reserves-category labels with actual probabilities and per-quantity direction.
- **Current implementation / as-built:** results expose generic low/mid/high percentile fractions; no reserves category or direction field exists. ABSENT.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T16 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-5 keeps Sw direction open; the schema must store direction rather than guess it.
- **UI/IPC/provenance surface:** generic labels propagate to UI and reportable results.
- **History/reachability:** no category implementation found.
- **Blocking decision / next action:** define probability and direction per quantity, leaving unresolved directions unset until sourced.

### SB-CUT-048

- **Specified contract:** cross-zone roll-up merges iteration cases before calculating percentiles.
- **Current implementation / as-built:** McResult reports zones independently and has no field-level case roll-up. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T16 has no cross-zone body.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-11 preserves incumbent-comparison residue; it does not authorize marginal-statistic merging.
- **UI/IPC/provenance surface:** no rolled-up joint case/result exists.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** if selected, merge aligned realization records from SB-CUT-044 and summarize only afterward.

### SB-CUT-049

- **Specified contract:** report realised correlation beside every requested correlation.
- **Current implementation / as-built:** McRequest accepts requested McCorrelation and Iman-Conover induces rank correlation; results report parameter-output Spearman only, not realised parameter-pair correlation. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** CHARACTERIZATION; iman_conover_induces_target_rank_correlation proves induction internally, but T18's returned requested/realised pair is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-9 leaves measurement-correlation comparison low priority; R-10 forbids silently approximating a non-positive-definite matrix.
- **UI/IPC/provenance surface:** requested pairs cross IPC; realised pairs are not returned or rendered.
- **History/reachability:** induction and parameter-output sensitivity are integrated.
- **Blocking decision / next action:** return requested and measured pairwise rank correlations with explicit refusal/advisory state.

### SB-CUT-050

- **Specified contract:** data-picked parameters are re-derived per realization rather than merely correlated.
- **Current implementation / as-built:** MC draws configured scalar parameters and reruns the module chain; it has no identity or operation for a data-picked parameter derivation. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T18 has no re-derivation limb.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** O-9 retains the measurement-correlation question; no substitute formula is invented.
- **UI/IPC/provenance surface:** prior rows cannot declare derived-from-data behavior.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** define a typed re-derivation plan only when the owning parameter workflow is selected.

### SB-CUT-051

- **Specified contract:** tornado bars carry absolute output values and units, with iteration-count context.
- **Current implementation / as-built:** MetricSet stores absolute low/base/high output values and the UI plots them, but no units field accompanies the metrics and run-count comparability is not a typed bar attribute. PARTIAL.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** CHARACTERIZATION; tornado_low_base_high_are_ordered proves ordering only. T34's 750-versus-5000 absolute stability and percentage divergence is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** units come from each output quantity; no display unit is inferred from pixels.
- **UI/IPC/provenance surface:** renderer knows metric names and overall iterations, but result bundles lack explicit units/count custody.
- **History/reachability:** OAT metrics and renderer are integrated.
- **Blocking decision / next action:** add quantity/unit/iteration metadata to each bar and run the independent stability fixture.

### SB-CUT-052

- **Specified contract:** a newly added prior starts with perturbation disabled.
- **Current implementation / as-built:** defaultRow immediately assigns a non-zero vendor seed or generic max(abs(value)*0.1,0.01) width; there is no enabled flag. PRESENT-DIVERGENT.
- **Release disposition and risk:** PILOT-BLOCKER; SILENT-WRONGNESS.
- **Automated evidence:** MISSING; T15 and T23 are absent as off-by-default controls.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** perturbation-off is specified; generic widths and vendor rows are not adopted defaults.
- **UI/IPC/provenance surface:** adding a row silently activates a distribution before the user supplies a sourced width.
- **History/reachability:** current auto-width behavior is integrated.
- **Blocking decision / next action:** add explicit enabled state defaulting false and keep all unsourced width fields absent.

### SB-CUT-053

- **Specified contract:** impossible realizations are counted/reported and remain in percentile accumulation.
- **Current implementation / as-built:** McPlausibility counts unlimited-curve violations, all realizations remain in per_real, and no exclusion path was found. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; FIELD-EVIDENCE.
- **Automated evidence:** CHARACTERIZATION; impossible_combo_guard_flags_negative_porosity, impossible_combo_guard_flags_supersaturation and impossible_combo_guard_clean_run_reports_zero prove diagnostics, but T25's accumulator bias fixture is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** R-4 forbids exclusion/clamp before accumulation; this row does not cure SB-CUT-041's use of limited curves.
- **UI/IPC/provenance surface:** plausibility result carries count, denominator, fraction, checked and detail.
- **History/reachability:** diagnostics and retention are integrated.
- **Blocking decision / next action:** retain diagnostics, add an observable proof that impossible records remain in percentile population, and field-exercise warnings.

### SB-CUT-054

- **Specified contract:** if every realization fails a chain step, the study is a failed job item with source error and never a zero-uncertainty result.
- **Current implementation / as-built:** per-well step failure is retained and the job/result surface reports failure without clean percentile output. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; DEGRADED-RESULT.
- **Automated evidence:** CORRECTNESS; core_reporting_tests::a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result asserts the observable job/result surfaces, not only an internal Result.
- **Manual evidence:** NONE; the automated lock does not close a representative field workflow.
- **Source/parameter boundary:** no numeric expected value; SB-CORE-002 observable refusal is the oracle.
- **UI/IPC/provenance surface:** failure survives the job boundary with the originating error.
- **History/reachability:** regression lock d25c274 and implementation are reachable.
- **Blocking decision / next action:** preserve the test unchanged and field-exercise one all-realization failure before pilot release.

### SB-CUT-055

- **Specified contract:** an uninterpreted well renders absent while a genuinely classified zero remains numeric zero.
- **Current implementation / as-built:** n_classified distinguishes the states and summaryDialog renders an em dash only for uninterpreted rows. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; DEGRADED-RESULT.
- **Automated evidence:** CORRECTNESS; frontend test an_uninterpreted_pay_summary_renders_absent_values_while_a_real_zero_net_zone_renders_zero pins both sides on rendered rows.
- **Manual evidence:** NONE; cutoffs/pay and Results QC scenarios remain unchecked.
- **Source/parameter boundary:** no sentinel value is introduced; zero and absence retain distinct typed evidence.
- **UI/IPC/provenance surface:** Rust sibling field crosses IPC and controls client-visible rendering.
- **History/reachability:** regression lock d25c274 and implementation are reachable.
- **Blocking decision / next action:** preserve the lock and field-exercise summary, dashboard and exported representations.

### SB-CUT-056

- **Specified contract:** a failed pay computation leaves a named section in every emitted PDF and a named degradation in the batch result.
- **Current implementation / as-built:** report generation emits a Pay Summary note page and records per-well degradation instead of omitting the section. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; DEGRADED-RESULT.
- **Automated evidence:** CORRECTNESS; report::tests::a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record inspects emitted PDF bytes and the batch record.
- **Manual evidence:** NONE specific to this failure; report is 6/53 overall.
- **Source/parameter boundary:** no numeric parameter.
- **UI/IPC/provenance surface:** PDF and batch/job outputs both preserve the failed section identity.
- **History/reachability:** regression lock d25c274 and report change are reachable.
- **Blocking decision / next action:** preserve the lock and field-exercise one batch with a real pay-data failure.

### SB-CUT-057

- **Specified contract:** nested net-flag IPC uses exact snake_case names and rejects unknown or case-drift fields.
- **Current implementation / as-built:** NetFlagSpec uses deny_unknown_fields; TypeScript sends snake_case; result serialization uses the declared snake_case key set. PRESENT-OK.
- **Release disposition and risk:** PILOT-BLOCKER; DATA-INTEGRITY.
- **Automated evidence:** CORRECTNESS; spec_deserializes_from_the_exact_json_the_frontend_sends, result_serializes_under_the_names_the_frontend_reads and ipc_ts_declares_the_same_wire_names_as_the_rust_structs pin positive and negative wire shapes.
- **Manual evidence:** NONE specific to the IPC refusal.
- **Source/parameter boundary:** exact wire schema is the expected value; no numeric source.
- **UI/IPC/provenance surface:** positive exact payload works; camelCase and unknown probe keys fail.
- **History/reachability:** wire fix and three regression locks are integrated.
- **Blocking decision / next action:** preserve tests and field-exercise one polygon run before pilot release.

### SB-CUT-058

- **Specified contract:** sweep more than one cutoff simultaneously.
- **Current implementation / as-built:** CutoffSweepRequest selects one property while holding the other scalar cutoffs fixed; the UI is one-axis. PARTIAL.
- **Release disposition and risk:** UNDECIDED; REQUESTED-CAPABILITY.
- **Automated evidence:** CHARACTERIZATION; cutoff_sweep_vsh_monotone and cutoff_sweep_ntg_and_dst_mask prove the one-axis implementation, not T32's multi-cutoff grid.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** no grid resolution/default is invented.
- **UI/IPC/provenance surface:** request/result carry a single varying property and series.
- **History/reachability:** one-dimensional sweep is integrated.
- **Blocking decision / next action:** decide whether multi-axis sensitivity enters the pilot, then define an explicit grid without arbitrary caps.

### SB-CUT-059

- **Specified contract:** solve cutoff values backwards from a target response.
- **Current implementation / as-built:** only forward sweeps exist; no inverse objective, solver or result exists. ABSENT.
- **Release disposition and risk:** DEFERRED; REQUESTED-CAPABILITY.
- **Automated evidence:** MISSING; T32 has no inverse-solve body.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** no optimizer tolerance or target-selection guidance is cited here and none is invented.
- **UI/IPC/provenance surface:** absent throughout.
- **History/reachability:** negative source/UI/history inventory confirmed absence.
- **Blocking decision / next action:** defer until product scope and cited solver controls exist.

### SB-CUT-060

- **Specified contract:** imported parameters are addressed by block, ordinal and semantic key, with mismatch refusal.
- **Current implementation / as-built:** no structured cutoff/Monte Carlo parameter importer exists. ABSENT.
- **Release disposition and risk:** UNDECIDED; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T17 and T21 are absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** E-6 preserves the CONTRACT transcription boundary; R-11 keeps B unit unknown; the protected ordinal map is not copied or inferred.
- **UI/IPC/provenance surface:** no import address, mismatch error or source record exists.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** adjudicate E-6, then build a source-addressed importer that refuses bare ordinal and semantic mismatch.

### SB-CUT-061

- **Specified contract:** validate requested display precision against field width before rendering.
- **Current implementation / as-built:** reports and office exports use formatting functions, but no precision-width pair validator or refusal exists. ABSENT.
- **Release disposition and risk:** DEFERRED; DATA-INTEGRITY.
- **Automated evidence:** MISSING; T39 is absent.
- **Manual evidence:** NONE.
- **Source/parameter boundary:** formatting relationship is specified; no field width or precision default is invented.
- **UI/IPC/provenance surface:** formatting is applied at render time without preflight validation.
- **History/reachability:** no implementation found.
- **Blocking decision / next action:** when configurable formatting enters scope, add typed width/precision validation and refuse overflow before render.

## Execution totals and retained gaps

- As-built verdicts: ABSENT 28; PARTIAL 14; PRESENT-DIVERGENT 8; PRESENT-OK 10; PRESENT-UNVERIFIED 1.
- Release dispositions: PILOT-BLOCKER 42; UNDECIDED 9; DEFERRED 10; OUT 0.
- Automated proof classes after the exact-test audit: CORRECTNESS 4; CHARACTERIZATION 11; MISSING 46.
- All 44 chapter test intentions are routed in the committed execution plan. The four correctness closures are T37, T37b, T37c and T38; every other claimed passing test is supporting/characterization unless the owned whole-contract oracle is added.
- All 44 parameter rows remain cited, absent or explicitly non-adoptable. No live literal was promoted to authority.
- O-1 through O-12, E-1 through E-6 and R-1 through R-13 remain accounted for. E-1 is closed only as an exposure ruling; O-2 still blocks SB-CUT-013 through SB-CUT-015; E-3/E-4/E-6 remain unresolved source or governance seams.
