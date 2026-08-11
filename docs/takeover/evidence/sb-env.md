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
- **Current source:** `modules.rs::ArgSpec` serializes choices, scalar min/max, required inputs, `computed_only` and `well_scope`. It has no general condition object, input-sample predicate, branch-conditional range, condition explanation or condition source, and derives `Serialize` only.
- **Qualifying acceptance tests:** none. T01/T02/T03/T38 have no executable whole-contract body; test class `MISSING`.
- **Supporting tests:** `workflow::tests::out_of_range_zone_param_is_rejected_not_clamped` exercises one scalar-range seam, not the condition schema or its round trip.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the partial schema is reachable at the accepted anchor; no complete condition representation was found in current or reachable source. Commit state `INTEGRATED` for the partial mechanism.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the serializable condition model and T01/T02/T03/T38 are missing; sources must come from the chapter, never current defaults.
- **Next action:** add the condition/source schema atomically, route it through serialization and UI, and implement the four named tests with enumeration, per-sample, branch and companion controls.

## SB-ENV-002 - Evaluate preconditions in the runner, before the module body

- **Chapter evidence:** P1; chapter status `ABSENT`; T02/T04/T38; sections 4.1, 6.1 and 8.
- **Atomic obligations:** evaluate every declared condition before arithmetic, per sample where needed, identically through dialog, saved chain, workflow, zone override, batch and API paths.
- **Current source:** `workflow.rs::resolve_param_arrays` rejects supplied non-finite/out-of-range scalars and named-zone overrides for well-scoped parameters before dispatch. It cannot evaluate the absent general conditions, does not validate option enumerations, and is not proof of all launch paths or data-dependent predicates.
- **Qualifying acceptance tests:** none; no four-path preflight fixture exists. Test class `MISSING`.
- **Supporting tests:** the exact-filter range and temperature-scope tests passed, but call the narrow resolver and do not prove all preconditions or paths.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the narrow runner gate is integrated and reachable at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-ENV-001's missing schema prevents a complete common preflight; the four-path observable test is absent.
- **Next action:** introduce one pre-dispatch evaluator used by every launch route and prove body non-entry plus identical refusal on all named routes.

## SB-ENV-003 - A violated precondition produces a refusal or a flagged result, never an unmarked number

- **Chapter evidence:** P0; chapter status `ABSENT`; T02-T05; sections 4.1, 6.1 and 8.
- **Atomic obligations:** name condition, offending value, expected range and source in a refusal, or emit a per-sample flag plus provenance; never emit an unmarked number; retain usable unaffected samples.
- **Current source:** `resolve_param_arrays` names a parameter, supplied value and scalar range in some refusals. It omits the source and condition identity, has no flagged-result alternative, and unknown selectors and correction input gaps can still produce unmarked outputs.
- **Qualifying acceptance tests:** none; T05's source-bearing payload and subset flag path do not exist. Test class `MISSING`.
- **Supporting tests:** the range and well-scope tests pass and prove only two labelled refusal fragments.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** partial refusals are integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** source-bearing conditions, flagged partial results and their provenance are absent.
- **Next action:** carry the declared condition record into one structured refusal/flag payload and prove both whole-run and subset violations without stale output.

## SB-ENV-004 - Every parameter carries a source string, built as one change with the validity field

- **Chapter evidence:** P0; chapter status `PARTIAL`; T06/T07; sections 4.1, 5, 6.1 and 8.
- **Atomic obligations:** every ENV parameter has a citation or explicit `ABSENT`; source and validity share schema, serialization, dialog and persistence; registry-wide build gates have zero exceptions.
- **Current source:** `ArgSpec::sources_topic` exists only for selected competing-value topics; `param_open` encodes no default but no explicit source token. Most ENV defaults have no machine-readable source, and direct-run `params_json` stores supplied numeric values rather than source/validity metadata.
- **Qualifying acceptance tests:** none; the promised domain-wide T06/T07 gates do not exist. Test class `MISSING`.
- **Supporting tests:** UI source rendering is reachable for selected topics only; a source-text or schema inventory cannot substitute for the missing zero-exception gate.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the partial source-topic seam is integrated; the 29 shipped-uncited and 32 specified-absent findings remain open at the anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** cited sources are missing for the 29 shipped defaults; the 32 ABSENT parameters must not acquire invented values.
- **Next action:** add one source-or-ABSENT field with SB-ENV-001 and implement exhaustive branch-aware T06/T07 before enabling any uncited default.

## SB-ENV-005 - A corrected curve carries the list of steps actually applied

- **Chapter evidence:** P0; chapter status `ABSENT`; T08-T10; sections 4.1, 5, 6.2, 7.1 OI-4 and 8.
- **Atomic obligations:** persist and reload every correction step, status and applied parameter value with the output curve.
- **Current source:** `workflow.rs` records only request numeric parameters and input bindings for direct runs; `chain.rs` records only the module ID sequence for a chain. Neither records applied/unavailable/disabled/refused steps, actual per-step parameters or a restart-retrievable correction manifest.
- **Qualifying acceptance tests:** none; T08-T10 are missing. Test class `MISSING`.
- **Supporting tests:** generic log-set version tests prove version retrieval, not correction-step custody.
- **Manual evidence:** processing-history 0/7; conditioning 0/27; workflow 0/23.
- **Git evidence:** generic provenance is integrated, but the required manifest is `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** OI-4 leaves the exact persistence location open; the manifest content itself is fully specified.
- **Next action:** settle OI-4, persist one applied-step manifest with the curve, and prove partial input plus restart retrieval.

## SB-ENV-006 - A curve named "corrected" MUST have been corrected

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T11/T12; sections 4.1, 6.2 and 8.
- **Atomic obligations:** refuse a correction-named output when required input is absent, or mark every unchanged sample and manifest omission; never silently pass through under `*_EC`.
- **Current source:** `gr_hole_corr` and `rhob_hole_corr` can copy input values into correction-named outputs when caliper is absent, with no companion state or step manifest. The modules are catalogued and picker-reachable.
- **Qualifying acceptance tests:** `modules::tests::env_corrections_move_the_right_way` passed and explicitly pins the no-caliper pass-through. Its oracle is current behavior, so this is `CHARACTERIZATION`, not correctness.
- **Supporting tests:** nominal directional-movement assertions prove arithmetic movement only and cannot justify the missing-input contract.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the divergent helpers are integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-ENV-005 and SB-ENV-007 are absent; no missing-input refusal/flag proof exists.
- **Next action:** refuse or visibly mark every unchanged correction sample and persist the omitted step, while retaining the current nominal control.

## SB-ENV-007 - Per-sample correction flag channel

- **Chapter evidence:** P1; chapter status `ABSENT`; T11/T13; sections 4.1, 6.2 and 8.
- **Atomic obligations:** emit full, partial, not-applied and refused states per sample, typed through SB-ENV-030 and linked to the step set.
- **Current source:** the three environmental-correction helpers emit only corrected numeric curves; no companion correction-state output exists.
- **Qualifying acceptance tests:** none; T11/T13 are missing. Test class `MISSING`.
- **Supporting tests:** nominal correction arithmetic does not observe state custody.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the missing applied-step model and typed flag model.
- **Next action:** define the typed correction-state channel and exercise all four states with partial-coverage inputs.

## SB-ENV-008 - Validity conditions are visible before the run, not only after it

- **Chapter evidence:** P2; chapter status `ABSENT`; T14; sections 4.1, 6.1 and 8.
- **Atomic obligations:** show every condition and source beside its field and pre-mark conditions that cannot be evaluated because inputs are absent.
- **Current source:** `moduleDialog.ts` renders numeric min/max controls and selected `sources_topic` content, but no general condition record exists and the dialog does not preflight required-input availability or show an un-evaluable state.
- **Qualifying acceptance tests:** none; no dialog-level T14 body exists. Test class `MISSING`.
- **Supporting tests:** frontend compilation and selected source rendering do not exercise the required pre-run state.
- **Manual evidence:** conditioning 0/27; workflow 0/23.
- **Git evidence:** partial UI seams are integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-ENV-001/SB-ENV-004 data and the input-availability preflight are absent.
- **Next action:** render the shared condition/source model and add a dialog test with both evaluable and missing-companion controls.

## SB-ENV-009 - A method-selection string that matches no known method is an error

- **Chapter evidence:** P0; chapter status `PRESENT-UNVERIFIED`; T03/T15; sections 4.1, 6.1 and 8.
- **Atomic obligations:** validate every named selector against its closed set; refuse unknown values by name; never fall through or retain prior-frame values.
- **Current source:** dialogs constrain choices, but backend `ctx.o()` receives strings without a common enumeration check. Multiple module bodies implement `== known` plus an `else` branch, so an API/saved value outside the manifest can select a fallback rather than refuse.
- **Qualifying acceptance tests:** none; T03/T15 are missing. Test class `MISSING`.
- **Supporting tests:** choice-label and manifest-shape tests do not inject an invalid backend selector.
- **Manual evidence:** conditioning 0/27; workflow 0/23; formation-temperature 0/0 not recorded.
- **Git evidence:** the divergent string dispatch is integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** one backend option validator and stale-output control are missing.
- **Next action:** validate manifest choices before every dispatch and test unknown strings after a valid frame through direct and saved-chain paths.

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
- **Current source:** `modules.rs::badhole` tracks `any` and `bad`, evaluates the two terms independently and leaves the result MISSING when neither can run. It emits no evaluated-term or reason record.
- **Qualifying acceptance tests:** none; T31/T32's complete degradation-plus-custody contract has no executable body. Test class `MISSING`.
- **Supporting tests:** `modules::tests::badhole_flags_washout_and_drho` passed and proves nominal two-term arithmetic, but not every availability combination or the missing term record.
- **Manual evidence:** conditioning 0/27; workflow 0/23; processing-history 0/7.
- **Git evidence:** the partial degradation logic is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-ENV-022's reason channel is absent.
- **Next action:** emit the evaluated-term state and add both single-input controls plus the neither-evaluable MISSING case.

## SB-ENV-022 - Bad-hole flag carries a reason channel

- **Chapter evidence:** P1; chapter status `ABSENT`; T31; sections 4.3, 6.3, 7.1 OI-7 and 8.
- **Atomic obligations:** identify caliper, density correction, both or neither-evaluable per sample.
- **Current source:** `badhole` emits only one untyped numeric `BADHOLE` curve; the information used to set it is discarded.
- **Qualifying acceptance tests:** none; T31 is missing. Test class `MISSING`.
- **Supporting tests:** the arithmetic test cannot recover which criterion fired from the output.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** OI-7 leaves one encoded reason curve versus several typed booleans open.
- **Next action:** settle OI-7, emit the four reason states and prove they remain distinguishable after persistence/export.

## SB-ENV-023 - The density correction's sign is preserved and reported

- **Chapter evidence:** P1; chapter status `ABSENT`; T31; sections 4.3, 6.3 and 8.
- **Atomic obligations:** preserve the sign of each density-correction exceedance in the reason output.
- **Current source:** `badhole` compares `abs(DRHO)` and emits only 0/1, so the sign is irrecoverably discarded.
- **Qualifying acceptance tests:** none; T31 is missing. Test class `MISSING`.
- **Supporting tests:** the existing bad-hole test covers magnitude only and supplies no positive/negative reason assertion.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** `UNIMPLEMENTED` at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the reason-channel representation selected under OI-7.
- **Next action:** preserve positive and negative exceedances as distinct typed reasons, with equal-magnitude opposite-sign controls.

## SB-ENV-024 - Bad-hole thresholds ship ABSENT with cited presets

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; T07/T33; sections 4.3, 5, 6.3, 7.2 ESC-1 and 8.
- **Atomic obligations:** ship both thresholds absent; optionally expose only named, cited presets; persist the chosen preset.
- **Current source:** `badhole_spec` ships numeric threshold defaults with no source/preset identity, and run provenance cannot name a selected preset.
- **Qualifying acceptance tests:** none; T07/T33 are missing. Test class `MISSING`.
- **Supporting tests:** `badhole_flags_washout_and_drho` uses supplied fixture thresholds and is not authority for defaults.
- **Manual evidence:** conditioning 0/27; processing-history 0/7.
- **Git evidence:** the divergent defaults are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** ESC-1 leaves which cited presets, if any, ship; the default values are not authorized.
- **Next action:** remove both defaults now; add preset identities only after ESC-1 is answered from cited study records.

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
