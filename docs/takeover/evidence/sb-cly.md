# SB-CLY live-adjudication evidence receipt

## Execution baseline

- Date: `2026-08-11` (`Asia/Jakarta`).
- Branch: `codex/g1-sb-geo-adjudication`.
- Execution HEAD before adjudication: `82c278672a84037829e303ee90d247b3a992effb`.
- Accepted implementation evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`, reachable from the execution HEAD.
- `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Worktree: clean at entry; sole registered worktree `D:\XX. SandiBumi`.
- Pre-edit ledger: `224 / 931` adjudicated, `707` unadjudicated, `167` pilot blockers.
- Scope guard: exactly 55 source-owned rows, `SB-CLY-001` through `SB-CLY-055`; thirteen P0, fifteen P1, nineteen P2, six P3, two P4; all 55 `owned_tests` fields populated; all 55 live verdict fields untouched at entry.
- Historical chapter status: thirty-four `ABSENT`, thirteen `PARTIAL`, seven `PRESENT-DIVERGENT`, one `PRESENT-OK`. These are evidence, not live verdicts.
- Test-intent guard: `SB-CLY-T01` through `T44`, routed once in the approved plan; source-owned cross-support mappings remain authoritative.
- Parameter guard: 58 chapter rows; fifteen explicitly `ABSENT - ships with no default`; one `NON-ADOPTABLE - cited for verification`. No value is introduced in this receipt.
- Manual evidence boundary: Jauhar owns and will perform manual review. This pass neither exercises nor checks a manual scenario. Existing checked evidence is read only from `REVIEW.md` through `docs/VERIFICATION_MATRIX.md`.
- Retrieval boundary: Clauding retrieval returned a weak match. The live repository, chapter citations already recorded in the PRD, executable tests, generated manual matrix, and reachable history are the adjudication evidence; the retrieval gap is not filled from model memory.

## Chapter and cross-domain findings carried into every row

- Section 4 says fourteen P0 requirements, while front matter and the consolidated ledger both contain thirteen. The thirteen-row ledger set is the execution scope; the PRD is not edited here.
- Section 5.1 says six current values, four contradictions, and two agreements, while its as-built table contains twelve rows with a different split. Row-specific chapter dispositions are preserved; the prose mismatch is not normalized in this lane.
- Current LAS export honors the project-declared sentinel. The cited `-999.25` value remains the project default, not a writer-owned invariant. Chapter T35's historical fixed-sentinel claim is therefore not copied forward unqualified.
- Current LAS import recognizes the standard declared conventions and explicit per-channel rules. Bare undeclared `-999` remains finite data unless a channel rule says otherwise; `NoNull` deliberately preserves matching finite amplitudes. T44 is adjudicated without weakening the data-I/O declaration contract.
- Universal Normalize has no default reference pair, but the hidden saved-chain-compatible `gr_normalize` manifest still carries a legacy endpoint pair. Neither pair is adopted as clay-volume authority.
- Escalations E1 through E5 remain open where applicable: the M-N constant conflict, LARINOV3 product decision, live-vendor verification queue, missing primary transform papers, and the unspecified Stieber clamp epsilon.

## SB-CLY-001 - Refuse and flag degenerate endpoints, never null silently

- **Chapter evidence:** P0; historical status `PARTIAL`; T01/T24/T32; sections 4.1, 6 and 8.
- **Atomic obligations:** validate every indicator endpoint pair before arithmetic; distinguish invalid endpoints from missing input; emit `ENDPOINT_INVALID`; name the parameters, zone and offending values in a run message.
- **Current source:** `vsh_gr` now declares a source-bearing endpoint-order precondition. The public runner can refuse it or keep unaffected samples with a generic binary precondition flag and condition message, but that framework records a sample index rather than the required zone and emits no CLY categorical reason. The direct evaluator still collapses invalid endpoints into `f32::NAN`; `vsh_dn` still leaves its value and diagnostic flag `NaN` on degenerate geometry.
- **Qualifying acceptance tests:** none; T01/T24/T32 have no executable whole-contract body. Test class `MISSING`.
- **Supporting tests:** `source_bearing_precondition_shapes_refuse_before_computation_while_a_valid_public_run_still_computes` checks the internal `vsh_gr` refusal, the generic workflow precondition fixture proves only a range violation with a binary companion, and `vsh_dn_degenerate_triangle_is_missing_not_inf` proves only numeric containment. None observes the specified CLY token, zone-bearing message or exported reason.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** current source was reverified at parent `26535ac2122e67137fcb2bae71c8ec261050c423`; no production code or test assertion changed in this blocker increment.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`; Gate 2 `BLOCKED-DECISION/SCOPE` on DEC-036; no false closure.
- **Blocker or decision:** DEC-036 — exact T01 requires SB-CLY-031/032's per-sample categorical provenance custody, but those owning rows are outside the immutable pilot manifest. No stable numeric/LAS codes, closed vocabulary, zone mapping or substitution-separation schema is authorized.
- **Next action:** Jauhar either adds SB-CLY-031/032 to the approved manifest or explicitly authorizes their exact categorical schema as narrow infrastructure. Then implement one shared pre-evaluation result and prove T01/T24/T32 at the persisted/exported reporting surface.

## SB-CLY-002 - Stieber as one generic shape parameter

- **Chapter evidence:** P1; historical status `PARTIAL`; T05-T07; sections 4.1, 6 and 8.
- **Atomic obligations:** one generic `I/(1+n(1-I))` implementation; user-editable positive `n`; named presets for the three cited values; no fixed-only family.
- **Current source:** `vsh_gr` exposes three fixed selector IDs and three separate hard-coded branches. Their arithmetic matches the cited preset forms, but no argument accepts `n` and no shared generic evaluator exists.
- **Qualifying acceptance tests:** none; the generic sweep, intermediate-`n` fixture and derived-clamp fixture are missing. Test class `MISSING`.
- **Supporting tests:** `every_vsh_gr_transform_lands_on_its_published_coefficient` passed exactly once for the fixed variants. It cannot exercise a generic parameter that does not exist.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** fixed variants are integrated; no current or reachable generic Stieber argument or evaluator was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** E5 leaves the single engineering epsilon deliberately unspecified; the chapter does not authorize choosing it here.
- **Next action:** implement the generic evaluator and cited presets together, after Jauhar supplies the E5 engineering decision, then run T05-T07 from both preset and free-parameter paths.

## SB-CLY-003 - Resolve vendor Stieber labels by alias, fail the import if unresolvable

- **Chapter evidence:** P1; historical status `ABSENT`; T12/T13; sections 4.1, 6 and 8.
- **Atomic obligations:** vendor-qualified alias table; imported-label resolution; ambiguous-origin refusal naming every candidate; never guess.
- **Current source:** the catalog contains only SandiBumi's `STIEBER1/2/3` IDs. There is no vendor parameter-set import path, writer-origin metadata or Stieber alias table.
- **Qualifying acceptance tests:** none; both positive vendor-origin resolution and origin-absent refusal are missing. Test class `MISSING`.
- **Supporting tests:** the selector-label test proves current IDs lead their visible labels; it does not import or disambiguate an external label.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** targeted source, test and reachable-history searches found no alias/import implementation.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** import must carry identifiable writer evidence; an unqualified label must remain an error.
- **Next action:** define the chapter's vendor-qualified alias records and refuse an origin-less collision before any value reaches a run request.

## SB-CLY-004 - Larionov in the exact normalised form

- **Chapter evidence:** P1; historical status `PRESENT-DIVERGENT`; T02/T04; sections 4.1, 6 and 8.
- **Atomic obligations:** evaluate both rock-age forms with the exact normalized denominator so both endpoints close exactly.
- **Current source:** `vsh_gr` uses the rounded decimal multipliers in the only Larionov branches. At the upper endpoint the unlimited outputs remain below one, and an existing test explicitly defends that current behavior.
- **Qualifying acceptance tests:** the specified exact-form T02/T04 bodies are absent. Test class `CHARACTERIZATION` because the executable control pins today's rounded result, not the required exact form.
- **Supporting tests:** `the_vsh_gr_labels_agree_with_the_coefficients_they_describe` and `every_vsh_gr_transform_lands_on_its_published_coefficient` each passed exactly once; they prove current label/arithmetic parity only.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the rounded implementation and defending test are integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** no parameter adjudication is required; the chapter cites the exact form. The existing rounded paths must be retained separately for SB-CLY-005 compatibility.
- **Next action:** add exact normalized selectors as the authoritative non-parity paths, migrate visible labels without renaming saved IDs, and prove both endpoints plus the mid-range discriminator.

## SB-CLY-005 - Keep the decimal Larionov reachable, for parity only

- **Chapter evidence:** P2; historical status `PRESENT-OK`; T03; sections 4.1, 6 and 8.
- **Atomic obligations:** retain rounded vendor parity; label it explicitly as parity and disclose non-closure; keep it non-default; record its use per sample.
- **Current source:** rounded arithmetic is reachable and `LINEAR` remains the module default. The Larionov labels are rock-age labels rather than explicit parity disclosures, help text does not state the boundary miss, and no provenance curve records the choice.
- **Qualifying acceptance tests:** none; T03's numeric values are exercised, but no executable test closes label, help, default and provenance together. Test class `MISSING`.
- **Supporting tests:** the transform and label tests passed exactly once and use the cited vendor-decimal arithmetic. They are narrow numeric/UI controls, not the complete parity contract.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** rounded branches and stable saved IDs are integrated; the required explicit parity/provenance surfaces were not found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-031; saved-run compatibility forbids casually renaming the current IDs.
- **Next action:** introduce an explicit compatibility mapping and parity disclosure while preserving stored IDs, then prove non-default selection and round-trippable provenance.

## SB-CLY-006 - `LARINOV3` warns that it has no published provenance

- **Chapter evidence:** P1; historical status `PRESENT-DIVERGENT`; T14; sections 4.1, 6, 7.2 E2 and 8.
- **Atomic obligations:** warn at run level that no published source is held; record selection; imply no equivalent authority or rock age.
- **Current source:** `LARINOV3` is runnable, its label states coefficients and correctly claims no rock age, and a code comment records the source gap. No warning reaches the user and no provenance curve records the selection.
- **Qualifying acceptance tests:** T14's warning/provenance body is absent. Its numeric limb is explicitly `CHARACTERIZATION`; the current transform test pins that limb. Test class `CHARACTERIZATION`.
- **Supporting tests:** the label and transform tests each passed exactly once and prove no rock-age label plus current overshoot behavior.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** the selector, label and numeric characterization are integrated; no reporting/provenance surface was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** E2 requires Jauhar to choose keep-with-warning or remove; this adjudication makes neither choice.
- **Next action:** if retained, add the explicit warning and provenance token without inventing a citation; if removed, preserve actionable saved-run compatibility.

## SB-CLY-007 - Clavier over its full analytic domain

- **Chapter evidence:** P2; historical status `PRESENT-DIVERGENT`; T08/T09; sections 4.1, 6 and 8.
- **Atomic obligations:** cited Clavier equation; runtime-derived exact analytic bounds; no rounded clamp literal.
- **Current source:** the equation is present, but `vsh_gr` clamps the index to hard-coded rounded bounds, truncating the cited valid upper domain.
- **Qualifying acceptance tests:** none; T08/T09's exact-bound and rounded-bound discriminator fixtures are missing. Test class `MISSING`.
- **Supporting tests:** the transform test passed for mid-range and ordinary endpoints, which do not discriminate the rounded clamp from the exact domain.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the hard-coded clamp is integrated and reachable at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** none; the chapter provides the analytic derivation and source. No local realization becomes a default.
- **Next action:** derive both bounds from the transform constants at runtime and add the exact-bound and rounded-literal negative control.

## SB-CLY-008 - Implement the Curved transform

- **Chapter evidence:** P2; historical status `ABSENT`; T11; sections 4.1, 6, 7.1 item 13 and 8.
- **Atomic obligations:** the cited three-branch function, exact boundaries and coefficients, continuous at both joins.
- **Current source:** the selector catalog and dispatch contain no Curved option or branch; unrelated plotting uses of the word curved are not this transform.
- **Qualifying acceptance tests:** none; the two-boundary continuity/discriminator body is missing. Test class `MISSING`.
- **Supporting tests:** none for this transform.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** current, test and reachable-history searches found no CLY Curved implementation.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** E3 can clarify incumbent selectability but does not block SandiBumi's cited implementation; E4 is a stronger-source opportunity, not an implementation dependency.
- **Next action:** decide whether Curved belongs in the pilot, then implement only from the chapter's held witnesses and add the two-sided continuity fixture.

## SB-CLY-009 - Domain clamps computed from transform parameters

- **Chapter evidence:** P0; historical status `PRESENT-DIVERGENT`; T07/T08; sections 4.1, 6, 7.2 E5 and 8.
- **Atomic obligations:** derive every restricted-domain bound from runtime parameters; use one named tested epsilon only for poles; no per-variant literals.
- **Current source:** all three Stieber clamps and the Clavier clamp are hard-coded literals. They cannot generalize to a free `n`, and the Clavier literal is analytically rounded inward.
- **Qualifying acceptance tests:** none; parameter-sweep derivation and exact finite-bound tests are missing. Test class `MISSING`.
- **Supporting tests:** existing transform tests passed but do not derive bounds or expose the clamp source.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** literal clamps are integrated; no shared domain-bound function or named epsilon was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** E5 deliberately withholds the engineering epsilon; it must not be inferred from current literals or floating-point behavior.
- **Next action:** obtain the E5 decision, implement one derived-domain facility, and prove it across the cited parameter sweep plus exact Clavier bound.

## SB-CLY-010 - A clamped sample is marked as clamped

- **Chapter evidence:** P0; historical status `ABSENT`; T10; sections 4.1, 6 and 8.
- **Atomic obligations:** per-sample clamp marker for domain and final clamps; per-zone clamped fraction in the run record; computed one distinguishable from clamped one.
- **Current source:** `vsh_gr` and `vsh_dn` emit unlimited and limited numeric twins, but no clamp reason channel or zone summary. A downstream reader must infer clamping by comparing two curves.
- **Qualifying acceptance tests:** none; T10's marker-count and run-summary fixture is missing. Test class `MISSING`.
- **Supporting tests:** `vsh_gr_linear_and_limits` and the transform inventory passed and prove only that the limited curve is bounded while the unlimited twin survives.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** silent limiting is integrated; no clamp marker/fraction surface was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the closed reason/provenance model of SB-CLY-031/SB-CLY-032.
- **Next action:** add one machine-readable clamp state and derive the zone fraction from it, then prove clamped and naturally computed endpoint samples from both sides.

## SB-CLY-011 - SP indicator

- **Chapter evidence:** P2; historical status `ABSENT`; T15; sections 4.1, 6 and 8.
- **Atomic obligations:** two-endpoint linear SP indicator with ordinary open parameters and no default transform ladder.
- **Current source:** the curve registry recognizes SP, but the module catalog and dispatcher contain no SP clay/shale indicator.
- **Qualifying acceptance tests:** none; the cited linear-index fixture is missing. Test class `MISSING`.
- **Supporting tests:** family recognition is infrastructure only and does not consume SP as a CLY indicator.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** source, test and reachable-history searches found no `vsh_sp` path.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the chapter keeps the two endpoints ABSENT until a user or cited record supplies them.
- **Next action:** decide pilot inclusion; if included, reuse a governed two-endpoint index with no shipped endpoint values and add T15 from the cited fixture.

## SB-CLY-012 - Three neutron single-indicator forms, no default

- **Chapter evidence:** P2; historical status `ABSENT`; T16; sections 4.1, 5, 6 and 8.
- **Atomic obligations:** three separately named and cited forms, no selected default, visible spread under one input.
- **Current source:** `vsh_dn` consumes neutron as part of a double indicator; no neutron-only CLY module, three-form selector, source help or comparison surface exists.
- **Qualifying acceptance tests:** none; the three-result/no-default fixture is missing. Test class `MISSING`.
- **Supporting tests:** N-D arithmetic cannot substitute for any of the three single-indicator forms.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** source, test and reachable-history searches found no neutron single-indicator path.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** competing endpoint/form evidence must remain visible; no default may be selected by implementation convenience.
- **Next action:** decide pilot inclusion, then expose all three cited forms together with an empty selection and a side-by-side discriminator result.

## SB-CLY-013 - Limestone-matrix precondition on neutron indicators

- **Chapter evidence:** P0; historical status `ABSENT`; T17; sections 4.1, 6 and 8.
- **Atomic obligations:** persist neutron matrix reference; refuse unknown reference; refuse mixed references absent an explicit conversion.
- **Current source:** `vsh_dn` is a reachable neutron consumer and accepts a bare NPHI array plus a numeric endpoint. Curve-family metadata and the runner carry no neutron matrix-reference type, so unknown or mixed references are consumed silently.
- **Qualifying acceptance tests:** none; the unknown-reference and mixed-reference refusal controls are missing. Test class `MISSING`.
- **Supporting tests:** generic N-D numeric tests contain no matrix-reference metadata and therefore cannot observe this contract.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** the untyped consumer is integrated; no current or reachable matrix-reference custody was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no conversion or endpoint may be assumed; the missing matrix fact is a hard refusal.
- **Next action:** add the cited matrix-reference enum to curve metadata and one common preflight used by every neutron consumer, then prove unknown, mixed and explicitly converted controls.

## SB-CLY-014 - Two-sided warning on the neutron clean endpoint

- **Chapter evidence:** P2; historical status `ABSENT`; T18; sections 4.1, 5, 6 and 8.
- **Atomic obligations:** warn on either side of the cited witness span and name both artefacts/values.
- **Current source:** `vsh_dn` exposes `NPHI_MA` with only a broad numeric input range and an uncited default. No source-aware two-sided witness warning exists in the manifest, dialog or run result.
- **Qualifying acceptance tests:** none; lower and upper outside-range controls plus in-range silence are missing. Test class `MISSING`.
- **Supporting tests:** N-D flag tests exercise output geometry and GR disagreement, not parameter-source warnings.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the unguarded parameter is integrated; no CLY source topic or witness warning was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the chapter supplies witness bounds for warning only; they are not defaults and must not be promoted as such.
- **Next action:** attach both artefact records to the parameter and add lower, upper and in-range controls while leaving value authority with the user.

## SB-CLY-015 - Four resistivity forms, no default

- **Chapter evidence:** P2; historical status `ABSENT`; T19; sections 4.1, 5, 6 and 8.
- **Atomic obligations:** four separately named/cited forms, no default, live spread display for the current input and parameter set.
- **Current source:** the module catalog, dispatch and UI contain no resistivity CLY indicator or four-form comparison surface.
- **Qualifying acceptance tests:** none; the four-result/no-selection fixture is missing. Test class `MISSING`.
- **Supporting tests:** resistivity-family recognition and saturation modules are different domains and do not satisfy this contract.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** source, test and reachable-history searches found no `vsh_res` path.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** chapter evidence explicitly forbids silently selecting among the four forms; endpoint values remain ABSENT.
- **Next action:** decide pilot inclusion, then implement the four cited forms as a no-default comparison set before enabling a run.

## SB-CLY-016 - Validate `R_clay < R_clean` before branching

- **Chapter evidence:** P0; historical status `ABSENT`; T20; sections 4.1, 6 and 8.
- **Atomic obligations:** ordered endpoint validation before any resistivity branch and an observable refusal naming both values.
- **Current source:** no resistivity CLY indicator exists, so there is no common pre-branch guard or reporting path.
- **Qualifying acceptance tests:** none; T20's deliberately inverted cited pair and non-entry assertion are missing. Test class `MISSING`.
- **Supporting tests:** no neighboring resistivity module is evidence for this absent CLY path.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** no current or reachable `vsh_res` guard was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** this guard must land atomically with any resistivity indicator; no form or endpoint default is authorized.
- **Next action:** implement one pre-dispatch endpoint validator before any resistivity-form branch and prove body non-entry plus actionable refusal.

## SB-CLY-017 - Cite Coriband where the Coriband form is used

- **Chapter evidence:** P2; historical status `ABSENT`; T33; sections 4.1, 6 and 8.
- **Atomic obligations:** where a Coriband-attributable form is offered, name Coriband in help and per-run provenance.
- **Current source:** no Coriband-labelled form, help record, source topic or CLY provenance curve exists.
- **Qualifying acceptance tests:** none; T33's complete sourced-workflow body is missing. Test class `MISSING`.
- **Supporting tests:** none; absence of a current form prevents accidental false attribution but does not implement the required conditional custody.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** current, test and reachable-history searches found no Coriband path.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** attribution becomes mandatory if the corresponding form enters pilot scope.
- **Next action:** bind help and run provenance to the method record when that form is implemented; do not add a label without the cited lineage.

## SB-CLY-018 - One canonical bilinear form for every double indicator

- **Chapter evidence:** P1; historical status `PARTIAL`; T21; sections 4.1, 6 and 8.
- **Atomic obligations:** one shared canonical cross-product evaluator for every double indicator; no per-indicator algebra.
- **Current source:** `vsh_dn` implements one specialized density-neutron rearrangement. The chapter proves algebraic equivalence for that path, but no reusable canonical helper exists and no second double indicator can consume it.
- **Qualifying acceptance tests:** none; the random parameter-grid and two-unit-system equivalence body is missing. Test class `MISSING`.
- **Supporting tests:** `vsh_dn_flags_offmodel_and_gr_divergence` passed exactly once for narrow fixtures; it is not a canonical-form or reuse test.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the specialized implementation is integrated; no shared CLY bilinear helper was found in current or reachable source.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** none; the chapter independently derives the canonical identity. Endpoint/source/type custody remains separate.
- **Next action:** extract the canonical typed evaluator, route N-D through it, and prove equivalence across the cited grid before adding other double indicators.

## SB-CLY-019 - Two-point clean line with an explicit constructor

- **Chapter evidence:** P1; historical status `PRESENT-DIVERGENT`; T22; sections 4.1, 6 and 8.
- **Atomic obligations:** named `c1`/`c2` clean-line points; optional matrix/fluid constructor; unrestricted direct points remain available.
- **Current source:** `vsh_dn` accepts only matrix/fluid scalar endpoints and derives its restricted clean line internally. No direct `c1`/`c2` representation or constructor boundary exists.
- **Qualifying acceptance tests:** none; constructor equivalence and moved-`c2` negative control are missing. Test class `MISSING`.
- **Supporting tests:** current N-D arithmetic exercises only the restricted parameterization.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the restricted path is integrated; no clean-line point type or constructor was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** clean-line points need typed quantities and source custody; no endpoint is inferred.
- **Next action:** make `c1`/`c2` the canonical input and add an explicit matrix/fluid constructor, then pin both equivalent and deliberately non-restricted geometry.

## SB-CLY-020 - Linkage semantics: `c1` linkable, `c2` never; doubles are not singles

- **Chapter evidence:** P2; historical status `ABSENT`; T23; sections 4.1, 6 and 8.
- **Atomic obligations:** opt-in `c1` linkage across double indicators; prohibit `c2` linkage; keep double and single endpoints separate.
- **Current source:** only one double indicator exists and no `c1`/`c2`, endpoint-link or CLY parameter-identity model exists.
- **Qualifying acceptance tests:** none; the three-way edit propagation fixture is missing. Test class `MISSING`.
- **Supporting tests:** generic zone parameters do not express cross-module identity or prohibited linkage.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no current or reachable linkage implementation was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-CLY-019 and on a source-bearing parameter identity, not name equality.
- **Next action:** decide whether multiple double indicators enter the pilot, then implement explicit link identities with a hard prohibition on `c2` and single/double cross-linking.

## SB-CLY-021 - Degenerate crossplot geometry is refused and reported

- **Chapter evidence:** P0; historical status `PARTIAL`; T24/T32; sections 4.1, 6 and 8.
- **Atomic obligations:** detect shale-on-clean-line geometry; refuse; name all three points; write distinct provenance and flag state.
- **Current source:** `vsh_dn` detects a near-zero denominator and avoids infinity, but silently leaves all three outputs `NaN`; the module's own flag is unset and no run message names the geometry.
- **Qualifying acceptance tests:** none; T24/T32's observable flag/reason/message checks are missing. Test class `MISSING`.
- **Supporting tests:** `vsh_dn_degenerate_triangle_is_missing_not_inf` passed exactly once and proves numeric containment only.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** the numeric guard is integrated; reporting and reason custody are absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-030 through SB-CLY-032; the existing numerical tolerance is current code, not adopted source authority.
- **Next action:** return a typed geometry refusal from the canonical evaluator and route it to both the indicator flag and closed provenance vocabulary.

## SB-CLY-022 - Refuse the printed sonic-density denominator

- **Chapter evidence:** P1; historical status `ABSENT`; T25; sections 4.1, 6, 7.1 item 3 and 8.
- **Atomic obligations:** sonic-density through the canonical form; reject the printed sign defect; record the defect disposition in source commentary; module-scoped open matrix travel time.
- **Current source:** no sonic-density CLY indicator, canonical shared evaluator or module-scoped CLY travel-time parameter exists.
- **Qualifying acceptance tests:** none; the finite canonical result versus rejected printed form is missing. Test class `MISSING`.
- **Supporting tests:** no sonic-porosity or unrelated crossplot path substitutes for this indicator.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** source, test and reachable-history searches found no `vsh_sd`/`vsh_ds` implementation.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** E3 may characterize the incumbent, but the chapter explicitly says the answer does not change SandiBumi's canonical implementation.
- **Next action:** build on SB-CLY-018 with the source-comment refusal and no default travel time, then add the sign-discriminating T25 fixture.

## SB-CLY-023 - Thorium and Potassium indicators

- **Chapter evidence:** P3; historical status `ABSENT`; T26; sections 4.1, 6 and 8.
- **Atomic obligations:** separate two-endpoint linear Th and K indicators with ordinary open parameters.
- **Current source:** no spectral-gamma CLY modules, registered families or dispatch branches exist.
- **Qualifying acceptance tests:** none; both cited linear-index fixtures are missing. Test class `MISSING`.
- **Supporting tests:** generic GR/SP family handling is not spectral-gamma indicator support.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18.
- **Git evidence:** current, test and reachable-history searches found no Th/K CLY implementation.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** endpoints remain ordinary user/source values with no shipped defaults; the petrophysics-first pilot can proceed without this P3 extension.
- **Next action:** defer until a later capability increment; when scheduled, add families and linear indicators without importing endpoint values.

## SB-CLY-024 - EM-propagation indicator, parameter named once

- **Chapter evidence:** P4; historical status `ABSENT`; T27; sections 4.1, 6 and 8.
- **Atomic obligations:** if EM propagation is implemented, expose one matrix travel-time parameter exactly once in UI and record.
- **Current source:** no EM/TPL CLY module, manifest, parameter record or UI exists.
- **Qualifying acceptance tests:** none; the manifest/record cardinality test is missing. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** current, test and reachable-history searches found no EM-propagation CLY path.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** no matrix travel-time value is supplied or needed while the P4 capability is deferred.
- **Next action:** keep out of the pilot; if later scheduled, define one semantic parameter key and assert single occurrence across manifest, UI and persisted record.

## SB-CLY-025 - M–N crossplot Vsh is deliberately not implemented

- **Chapter evidence:** P4; historical status `ABSENT`; T33; sections 4.1, 6, 7.2 E1 and 8.
- **Atomic obligations:** do not implement until E1 is resolved; record the deliberate absence and reason in the module library.
- **Current source:** no M-N CLY module or dispatch exists, satisfying the safety half by omission. The module library has no user-visible record that the absence is deliberate or source-blocked.
- **Qualifying acceptance tests:** none; no catalog/documentation control pins deliberate exclusion and reason. Test class `MISSING`.
- **Supporting tests:** unknown module names fail generically, but that response does not name E1 or distinguish a deliberate refusal from an oversight.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no M-N implementation exists in current or reachable source; no deliberate-exclusion record was found.
- **Verdict:** `PARTIAL`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** E1 requires the cited primary chart or a controlled live-vendor fixture; neither conflicting constant may be chosen.
- **Next action:** add a catalog-level deliberate-exclusion explanation now and keep computation absent until E1 is resolved in a separate authorized increment.

## SB-CLY-026 - NMR clay volume is typed as a clay volume

- **Chapter evidence:** P3; historical status `ABSENT`; T28/T43; sections 4.1, 6 and 8.
- **Atomic obligations:** NMR bound-water result typed `VCL`, never aliased to `VSH`; distinct provenance; ordinary naming conventions; wrong-type refusal.
- **Current source:** no NMR CLY module, `VCL` family, type gate or CLY provenance token exists.
- **Qualifying acceptance tests:** none; numeric/type/provenance and wrong-consumer controls are missing. Test class `MISSING`.
- **Supporting tests:** generic NMR/distribution UI elsewhere does not compute or type this clay-volume quantity.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** current, test and reachable-history searches found no `vsh_nmr`/`VCL_NMR` path.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-CLY-043/SB-CLY-046 typing and provenance; no NMR parameter is inferred.
- **Next action:** defer from the pilot; when scheduled, land the typed family and wrong-type refusal before the calculation.

## SB-CLY-027 - Clip each indicator before combining, never after

- **Chapter evidence:** P0; historical status `ABSENT`; T29; sections 4.2, 6 and 8.
- **Atomic obligations:** clip every contributor before combination; never rely on a final-only clip; state the operation order in the run record.
- **Current source:** the CLY catalog and dispatch expose no combination layer. Limited outputs exist per current indicator, but no governed contributor inventory or run-order record combines them.
- **Qualifying acceptance tests:** none; the two clip-order discriminator cases are missing. Test class `MISSING`.
- **Supporting tests:** current indicator tests prove their own limited curves remain bounded, not that a future combiner consumes those limited values in the required order.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** current, test and reachable-history searches found no CLY combiner.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the chapter independently specifies the order; no vendor behavior or numeric parameter needs adjudication.
- **Next action:** implement clip-then-combine as the only workflow order and persist that order, then add both chapter discriminator cases.

## SB-CLY-028 - Only bounded-safe combiners

- **Chapter evidence:** P1; historical status `ABSENT`; T30; sections 4.2, 6 and 8.
- **Atomic obligations:** minimum, mean, median and Lateral pseudomedian; every offered result bounded by clipped contributors; offer no unsafe estimator.
- **Current source:** no CLY combination manifest, evaluator or UI selector exists.
- **Qualifying acceptance tests:** none; the randomized bound-preservation inventory is missing. Test class `MISSING`.
- **Supporting tests:** unrelated averaging/median implementations do not define this contributor contract.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no current or reachable CLY combiner was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** E3 can characterize incumbent ambiguity but does not change the chapter's bounded-safe set.
- **Next action:** implement only the four named estimators behind one contributor interface and prove every catalogued estimator satisfies the randomized bound invariant.

## SB-CLY-029 - A zero is a value, not an absence

- **Chapter evidence:** P0; historical status `ABSENT`; T31; sections 4.2, 6 and 8.
- **Atomic obligations:** include exact zero as a finite contributor; reserve `f32::NAN` for numeric absence; never filter contributors by positivity.
- **Current source:** the project-wide numeric convention correctly uses `f32::NAN` and current CLY arithmetic preserves zero. However no CLY combiner exists, so the exact contributor path the requirement governs is absent.
- **Qualifying acceptance tests:** none; mean and median controls containing both zero and a positive value are missing. Test class `MISSING`.
- **Supporting tests:** the global missing-data convention is a supporting seam, not proof that an unimplemented combiner retains zero.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** the NaN convention is integrated; no current or reachable zero-preserving CLY combination path was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-028; no tolerance or positive threshold may be introduced.
- **Next action:** make contributor inclusion exactly `is_finite`, then pin zero from both sides with arithmetic mean and median fixtures.

## SB-CLY-030 - Three distinct absences, distinguishable in the output

- **Chapter evidence:** P1; historical status `PARTIAL`; T32; sections 4.2, 6 and 8.
- **Atomic obligations:** per-sample distinction among missing input, discriminator rejection and indicator refusal; downstream-readable without input inspection.
- **Current source:** missing inputs, workflow masks and invalid endpoints/geometry all become `f32::NAN`. Mask application works, but no companion reason channel preserves why the numeric value is absent.
- **Qualifying acceptance tests:** none; T32's three-input/three-token workflow fixture is missing. Test class `MISSING`.
- **Supporting tests:** mask and degenerate-geometry tests each passed exactly once, confirming the collapse to the same numeric absence rather than the required distinction.
- **Manual evidence:** shale-volume 0/17; conditioning 0/27; workflow 0/23.
- **Git evidence:** masking and numerical guards are integrated; no closed CLY absence reason was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-031/SB-CLY-032; numeric NaN remains mandatory and cannot itself encode the reason.
- **Next action:** pair every CLY value with one typed reason channel and prove the three states remain distinguishable after persistence and export.

## SB-CLY-031 - Every clay/shale volume carries a provenance curve

- **Chapter evidence:** P0; historical status `ABSENT`; T33/T34; sections 4.2, 6 and 8.
- **Atomic obligations:** per-sample method/transform-or-absence provenance for every Vsh/Vcl module; export beside the value; survive LAS round trip.
- **Current source:** `vsh_gr` emits only unlimited/limited values and `vsh_dn` adds one reliability flag. Neither emits method identity or absence reason; the generic run record stores request parameters, not a per-sample provenance curve.
- **Qualifying acceptance tests:** none; full workflow, export and re-import assertions are missing. Test class `MISSING`.
- **Supporting tests:** output-manifest and generic LAS round-trip tests passed, but no CLY provenance curve exists for them to carry.
- **Manual evidence:** shale-volume 0/17; las-export 0/2; processing-history 0/7.
- **Git evidence:** no `MTH_VSH`, `VSH_PROV` or equivalent CLY output exists in current or reachable source.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** requires SB-CLY-032's closed vocabulary and SB-CLY-046 family registration.
- **Next action:** define one typed per-sample provenance output for every CLY module and make its LAS round trip a release gate.

## SB-CLY-032 - One closed provenance vocabulary, substitution recorded separately

- **Chapter evidence:** P1; historical status `ABSENT`; T34; sections 4.2, 6 and 8.
- **Atomic obligations:** one closed documented token vocabulary; method identity separate from substitution state; independently readable fields.
- **Current source:** no CLY provenance token type, vocabulary, substitution field or module output exists. `VSH_DN_FLAG` is a numeric reliability flag covering two conditions, not method provenance.
- **Qualifying acceptance tests:** none; method-token versus substitution-field independence is missing. Test class `MISSING`.
- **Supporting tests:** detector and mask tests observe numeric flags only.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** current, test and reachable-history searches found no CLY token vocabulary.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** token names are product schema, not petrophysical parameters; they must be defined once before dependent outputs land.
- **Next action:** publish one enum-like vocabulary plus a separate substitution-state channel and reject unknown tokens at persistence/import boundaries.

## SB-CLY-033 - Per-flag override generality

- **Chapter evidence:** P2; historical status `PARTIAL`; T36; sections 4.2, 6 and 8.
- **Atomic obligations:** per-zone/per-indicator rule list of discriminator curve, minimum, maximum and action; no fixed-detector-only design.
- **Current source:** `badhole` and `condflag` hard-code detector families. The workflow `MASK` option can consume an arbitrary resolved curve, but it provides no rule list, numeric window, action or indicator-specific override custody.
- **Qualifying acceptance tests:** none; a configurable two-sided rule applied to one indicator/zone with a control indicator is missing. Test class `MISSING`.
- **Supporting tests:** bad-hole, coal and mask tests passed exactly once and establish the current fixed producer plus generic consumer seams.
- **Manual evidence:** shale-volume 0/17; conditioning 0/27; workflow 0/23.
- **Git evidence:** fixed detectors and generic masking are integrated; no rule-list model was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** rule bounds are user/source data and ship absent; no threshold is introduced by this adjudication.
- **Next action:** add a typed rule list resolved per indicator and zone, preserving the current detectors only as explicit rule producers rather than privileged hard-coded policy.

## SB-CLY-034 - No magic sentinel for a rejected sample

- **Chapter evidence:** P0; historical status `PARTIAL`; T35/T44; sections 4.2, 6 and 8.
- **Atomic obligations:** rejected numeric sample is NaN plus provenance; export only the declared writer sentinel; warn and treat a known undeclared vendor sentinel as absent.
- **Current source:** workflow masks use `f32::NAN` and every registered exporter receives the project-declared sentinel. Import honours file-declared nulls, the two standard conventions and explicit per-channel plural/`NoNull` rules; an undeclared bare `-999` remains finite unless a channel rule names it. No CLY-scoped warning/quarantine exists, and CLY provenance is absent.
- **Qualifying acceptance tests:** none; exact T35's provenance-export arm depends on DEC-036 and exact T44 conflicts with the adopted explicit-`NoNull` control until precedence and identifier scope are adjudicated. Test class `MISSING`.
- **Supporting tests:** the registered-writer sentinel controls, declared-null recognition, plural per-channel screening and explicit `NoNull` survival establish the current safe boundary. They expose the undeclared CLY gap while preventing a global `-999` rewrite from being called correct.
- **Manual evidence:** shale-volume 0/17; las-export 0/2; processing-history 0/7.
- **Git evidence:** current source was reverified at parent `41e603cbdd60736f79e5fb8e95f0500ee868fb89`; no production code or test assertion changed in this blocker increment.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`; Gate 2 `BLOCKED-DECISION/DEPENDENCY` on DEC-036 and DEC-037; no false closure.
- **Blocker or decision:** DEC-036 owns the missing provenance curve/export representation. DEC-037 owns the exact CLY `-999` identification and precedence contract versus SB-DIO's explicit `NoNull`; the number alone is not sufficient evidence of absence.
- **Next action:** settle DEC-037's exact source-controlled identifiers, `NoNull` precedence and blocking-versus-automatic UX, plus DEC-036's categorical export representation; then implement exact T35/T44 with declared-null, standard-sentinel, unrelated-curve and `NoNull` controls from both sides.

## SB-CLY-035 - Discriminator tests are two-sided by default

- **Chapter evidence:** P2; historical status `PARTIAL`; T36; sections 4.2, 6 and 8.
- **Atomic obligations:** every rule supports minimum and maximum; caliper bad-hole detection covers over-gauge and under-gauge.
- **Current source:** `badhole` now applies the explicit `DCAL_MAX` as a strict maximum absolute departure, `|CALI - bit size| > DCAL_MAX`, so one supplied magnitude defines symmetric minimum/maximum accepted gauge bounds. Its manifest and visible help use the same absolute-departure contract. The arbitrary per-indicator/per-zone rule-list generality remains owned by deferred SB-CLY-033 rather than being smuggled into this row.
- **Qualifying acceptance tests:** `modules::tests::under_gauge_and_over_gauge_hole_both_fire_while_both_strict_boundaries_and_in_gauge_do_not` was witnessed RED with the cited T36 under-gauge sample returning clear, then passed after the absolute-departure change. It pins cited under-gauge, symmetric over-gauge, both strict boundaries, in-gauge and evaluated-state controls; test class `CORRECTNESS`.
- **Supporting tests:** `badhole_flags_washout_and_drho`, the independent-term availability test and bit-size-source test remain green for DRHO, over-gauge, missing-geometry and measured/explicit bit-size paths.
- **Manual evidence:** conditioning 0/27; shale-volume 0/17.
- **Git evidence:** implementation and exact T36 are prepared on current parent `bf35be67fb1f27c7ed42f44af2d1b3332e7bb39b`; no threshold magnitude, default, unit or DRHO behavior changed.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied built-in caliper contract); `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for exact T36. `DCAL_MAX` remains an explicit user/source value with no shipped default; SB-CLY-033 still owns arbitrary discriminator-rule lists and is not claimed closed here.
- **Next action:** preserve exact T36 and the strict-boundary controls, perform Visual/Manual/Field review separately, and continue SB-CLY-041.

## SB-CLY-036 - Per-indicator coal branch with its own provenance token

- **Chapter evidence:** P2; historical status `PARTIAL`; T37; sections 4.2, 6 and 8.
- **Atomic obligations:** optional branch on every indicator; default off; coal sets Vsh to zero; distinct `COAL` provenance; bad hole vetoes coal.
- **Current source:** `condflag` computes a coal detector and correctly vetoes it on bad-hole samples. Neither CLY indicator exposes an opt-in coal branch, zeroes its output, records `COAL`, or declares an off default.
- **Qualifying acceptance tests:** none; enabled, disabled and bad-hole-veto indicator outputs with provenance are missing. Test class `MISSING`.
- **Supporting tests:** both coal detector tests passed exactly once and prove detection/veto only; their current thresholds are fixture inputs, not authorization for a CLY branch default.
- **Manual evidence:** shale-volume 0/17; conditioning 0/27; workflow 0/23.
- **Git evidence:** detector code is integrated; no `OPT_COAL` or CLY coal-provenance path exists in current or reachable source.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-031/SB-CLY-032 and explicit detector-rule custody; no threshold is adopted here.
- **Next action:** wire an off-by-default branch through every indicator, consume the explicit detector result, and prove ordinary, coal and bad-hole samples with distinct provenance.

## SB-CLY-037 - A complete percentile endpoint pipeline

- **Chapter evidence:** P1; historical status `ABSENT`; T38; sections 4.3, 6 and 8.
- **Atomic obligations:** selectable well/zone/named-set pool; optional pre-statistic clip; clean/shale percentile pair; explicit out-of-range extrapolation with warning; realized endpoints and scope recorded.
- **Current source:** universal Normalize computes percentiles over the run after workflow masking and the hidden legacy GR preset supplies a fixed percentile pair. Histogram/crossplot can write a picked zone value with undoable plot provenance. No path turns these seams into a CLY endpoint record with pooling identity, pre-clip policy, extrapolation warning and downstream consumption.
- **Qualifying acceptance tests:** none; T38's complete pooled endpoint/run-record fixture is missing. Test class `MISSING`.
- **Supporting tests:** normalization mapping, per-well anchoring, masked-percentile exclusion and plot-write provenance each passed exactly once. They prove isolated seams only.
- **Manual evidence:** shale-volume 0/17; histogram 5/22; crossplot 6/13; workflow 0/23.
- **Git evidence:** normalization and plot-write infrastructure are integrated; no end-to-end CLY endpoint pipeline was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** realized endpoints are study data, not product defaults; all source and pooling scope must persist explicitly.
- **Next action:** define one endpoint record and thread source curve, pool, pre-clip, percentile/extrapolation, realized value and downstream run through it before adding UI convenience.

## SB-CLY-038 - Two-way binding between percentile and value

- **Chapter evidence:** P2; historical status `ABSENT`; T39; sections 4.3, 6 and 8.
- **Atomic obligations:** percentile edit recomputes value; value edit recomputes displayed percentile; persist which representation was authoritative.
- **Current source:** histogram and crossplot provide one-way writes of a numeric zone parameter. There is no CLY endpoint entity, reverse percentile lookup, two-way control or authority record.
- **Qualifying acceptance tests:** none; both edit orders and authority assertions are missing. Test class `MISSING`.
- **Supporting tests:** plot-derived writes are undoable and source-validated, but they store only the chosen value, not its percentile binding.
- **Manual evidence:** shale-volume 0/17; histogram 5/22; crossplot 6/13.
- **Git evidence:** one-way plot writes are integrated; no two-way CLY binding was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-CLY-037's persisted endpoint distribution and scope; neither side may be reconstructed from an unspecified population.
- **Next action:** make authority explicit in the endpoint record and add both edit-order controls against the same declared distribution.

## SB-CLY-039 - The P3/P97 house preset is a cited, recorded preset

- **Chapter evidence:** P2; historical status `PARTIAL`; T38; sections 4.3, 5, 6 and 8.
- **Atomic obligations:** if offered, named preset with source; visibly a convention, not constant; record preset identity, pool and realized values.
- **Current source:** the hidden saved-chain-compatible `gr_normalize` manifest still ships P3/P97 as cited numeric defaults and identifies the convention. It has no preset identity and ordinary run parameters do not record the pool or realized percentiles as endpoint custody; the unrelated GR reference pair now ships absent.
- **Qualifying acceptance tests:** T38's cited preset-identity, pool and realized-value record remains absent. The GR reference-pair absence test is an SB-CORE-004 control, not proof of this P3/P97 contract, so row test class is `MISSING`.
- **Supporting tests:** mapping arithmetic and universal Normalize's absent-reference refusal remain narrower than the complete preset-custody contract.
- **Manual evidence:** shale-volume 0/17; histogram 5/22; workflow 0/23; processing-history 0/7.
- **Git evidence:** the legacy preset is integrated for saved-run compatibility and hidden from normal ribbon discovery; source/preset custody is absent.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the chapter cites the convention but not any realized endpoint; saved-chain compatibility must not turn legacy defaults into physical authority.
- **Next action:** represent P3/P97 as an explicitly cited CLY preset, record realized values/scope, and migrate legacy runs without silently promoting the old manifest.

## SB-CLY-040 - Warn where a percentile endpoint lands near a transform pole

- **Chapter evidence:** P2; historical status `ABSENT`; T10/T40; sections 4.3, 6 and 8.
- **Atomic obligations:** evaluate proposed percentile endpoints against selected transform domain/pole; warn with affected interval fraction.
- **Current source:** no CLY endpoint pipeline, derived transform-domain service, clamp marker or interactive warning surface connects endpoint selection to transform behavior.
- **Qualifying acceptance tests:** none; the cited affected-fraction fixture is missing. Test class `MISSING`.
- **Supporting tests:** plot percentile lines and limited/unlimited curves operate independently and do not measure the interaction.
- **Manual evidence:** shale-volume 0/17; histogram 5/22; crossplot 6/13.
- **Git evidence:** no current or reachable endpoint-versus-pole warning path was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-CLY-009, SB-CLY-010 and SB-CLY-037; E5 prevents inventing a Stieber pole tolerance.
- **Next action:** after those dependencies exist, compute the warning from the declared domain/tolerance and the declared population, then pin the fraction from both warning and no-warning sides.

## SB-CLY-041 - Prefer the corrected input alias, uniformly

- **Chapter evidence:** P2; historical status `PRESENT-DIVERGENT`; T43; sections 4.3, 6 and 8.
- **Atomic obligations:** ordered aliases prefer corrected over raw across every indicator; persist the resolved mnemonic.
- **Current source:** VSH manifests now declare ordered per-input aliases: `GR_COR -> GR_EC -> GR`, `RHO_COR -> RHOB_EC -> RHOB` and `NPHI_COR -> NPHI_EC -> NPHI`. The shared per-well resolver is used by direct runs, preflight and saved-chain ancestry; an explicit interpreter selection still wins. Module and Workflow controls expose a distinct Auto choice and state that the resolved curve is recorded.
- **Qualifying acceptance tests:** exact T43 `corrected_aliases_win_over_raw_and_normalized_inputs_raw_remains_the_fallback_and_each_resolved_curve_is_recorded` passed 1/0/0 after witnessed RED and a raw-first mutation. It independently computes corrected and raw VSH-GR values, uses T18's cited density-neutron witness, includes `GRN` as a losing control, proves vendor/native/raw fallback order and reads exact direct plus saved-chain ancestry. Test class `CORRECTNESS`.
- **Supporting tests:** TypeScript compilation and cargo check are green. The fresh full project gate passed 1027 / 0 / 37 with 31 owned warnings, exercising the typed manifest/UI and all existing workflow routes without changing the unordered semantic-family registry. The full Monte Carlo module suite also passed 23 / 0 / 0 after the resolver kept raw manifest defaults for direct-context compatibility.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** current topic-branch worktree; one requirement-scoped commit will carry manifest, resolver, direct/chain custody, UI and evidence changes together.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated T43; Visual, Manual and Field evidence remain open.
- **Next action:** preserve exact T43 and explicit-selection precedence, perform the mixed-well review separately, and continue SB-CLY-042 without turning picking advice into a default.

## SB-CLY-042 - Picking conventions stated as help text, not encoded as defaults

- **Chapter evidence:** P3; historical status `ABSENT`; T33; sections 4.3, 5, 6 and 8.
- **Atomic obligations:** sourced picking advice attached to the parameter; never convert advice into a numeric default.
- **Current source:** `ArgSpec.guidance` serializes advice and source separately from `default`. `vsh_gr`, `vsh_dn` and `gr_normalize` attach the F15/F17 crossplot, percentile and common-reference procedures to each relevant field. Module and Workflow editors render the same source-bearing hint. Endpoint/reference values remain empty with source state `ABSENT`; the separately cited RHO_MA/RHO_FL/NPHI_FL defaults and named P3/P97 percentile preset remain intact.
- **Qualifying acceptance tests:** `documented_picking_conventions_are_sourced_help_and_never_numeric_defaults` was witnessed RED, passed 1/0/0, turned RED when `vsh_gr.GR_SH` lost guidance, and turned RED when the cited RHO_MA positive control was changed to ABSENT. It inventories every shipping endpoint/reference field, requires non-empty advice and source, and pins the value/default distinction from both sides. Test class `CORRECTNESS`.
- **Supporting tests:** `source_bearing_picking_guidance_is_rendered_beside_the_parameter_without_becoming_its_value` was witnessed RED, passed 1/0/0, and turned RED when the renderer omitted guidance. TypeScript compilation is green; Module and Workflow use one hint renderer.
- **Manual evidence:** shale-volume 0/17; histogram 5/22; crossplot 6/13.
- **Git evidence:** current topic-branch worktree; one requirement-scoped commit will carry the typed manifest field, CLY guidance, shared rendering, tests and evidence together.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated SB-CLY-042; Visual, Manual and Field evidence remain open.
- **Next action:** preserve the source-bearing guidance/default separation, execute visual/manual review separately, and continue SB-CLY-043 without inferring quantity type from mnemonic.

## SB-CLY-043 - Shale volume and clay volume are distinct typed quantities

- **Chapter evidence:** P0; historical status `ABSENT`; T28/T43; sections 4.4, 6 and 8.
- **Atomic obligations:** distinct VSH/VCL types; wrong-type refusal; consumers accepting both record which quantity they received.
- **Current source:** the family registry contains neither quantity. Current CLY modules emit unregistered `VSH`; downstream modules bind by mnemonic and cannot distinguish a renamed `VCL`. No run record stores a quantity type.
- **Qualifying acceptance tests:** none; family/type refusal and dual-acceptance custody are missing. Test class `MISSING`.
- **Supporting tests:** generic family resolution passes only for existing families and confirms CLY names resolve to none by inventory.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** untyped outputs/consumers are integrated; no VSH/VCL quantity type was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** type identity must be metadata-backed, not inferred from a mutable mnemonic.
- **Next action:** register distinct quantity types first, enforce them in runner input resolution, and add wrong-type plus explicit dual-type controls.

## SB-CLY-044 - Both bridges named; no default ratio

- **Chapter evidence:** P1; historical status `PARTIAL`; T41; sections 4.4, 5, 6 and 8.
- **Atomic obligations:** named ratio bridge in both directions; explicit ratio with no default; prefer structural bridge where available; record bridge identity.
- **Current source:** no indicator-layer Vsh/Vcl ratio bridge or ratio parameter exists. A structural calculation inside the SSC solver derives related quantities, but it is not exposed as a typed bridge to CLY consumers and produces no bridge-selection run record.
- **Qualifying acceptance tests:** none; unset-ratio refusal and two named-form discriminator are missing. Test class `MISSING`.
- **Supporting tests:** solver-internal relationships are supporting architecture only and cannot close an absent public bridge contract.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** no `CSR`/clay-shale-ratio path exists in current or reachable indicator code.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** both ratio parameters remain explicitly ABSENT; the two vendor forms must remain separately named rather than averaged or merged.
- **Next action:** expose a typed structural bridge where available and two explicit no-default ratio forms elsewhere, with bridge identity in the run record.

## SB-CLY-045 - Endpoint conversion identities are explicit and tested

- **Chapter evidence:** P2; historical status `ABSENT`; T42; sections 4.4, 6 and 8.
- **Atomic obligations:** named tested Vsh/Vcl endpoint identities; never reuse an endpoint across types unchanged.
- **Current source:** no Vsh/Vcl types, bridges or endpoint conversion facility exists; bare numeric parameters cannot declare which quantity an endpoint describes.
- **Qualifying acceptance tests:** none; converted-versus-direct-reuse discriminator is missing. Test class `MISSING`.
- **Supporting tests:** generic physical-unit conversions do not convert between Vsh and Vcl semantics.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** no current or reachable endpoint identity was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-CLY-043/SB-CLY-044; no ratio may be supplied by default.
- **Next action:** implement conversions as named typed identities and require explicit bridge inputs before any cross-quantity endpoint can be consumed.

## SB-CLY-046 - Register the Vsh/Vcl curve families

- **Chapter evidence:** P1; historical status `ABSENT`; T43; sections 4.4, 6 and 8.
- **Atomic obligations:** distinct clipped Vsh, unlimited Vsh, Vcl and flag/provenance families with vendor aliases; no emitted curve resolves to none.
- **Current source:** `curves.rs::FAMILIES` has no CLY family despite `vsh_gr`/`vsh_dn` emitting four CLY names. Raw/corrected/normalized GR aliases are also folded together.
- **Qualifying acceptance tests:** none; the complete emitted/vendor mnemonic inventory and four-family distinctness test is missing. Test class `MISSING`.
- **Supporting tests:** `families_resolve_common_mnemonics` passed and proves the existing registry behavior, including merged GR aliases; it does not include CLY outputs.
- **Manual evidence:** generic-curve-store 0/18; shale-volume 0/17.
- **Git evidence:** current and reachable family registries contain no VSH/VCL/provenance entries.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** aliases must come from the chapter's held evidence; family identity must remain distinct from mnemonic preference.
- **Next action:** add the four distinct families and exhaustive alias inventory before enforcing SB-CLY-043 or exporting provenance.

## SB-CLY-047 - Organic-shale pre-correction in renormalised form

- **Chapter evidence:** P3; historical status `ABSENT`; T33; sections 4.4, 5, 6 and 8.
- **Atomic obligations:** renormalize over non-organic fraction and emit corrected input as a reviewable curve.
- **Current source:** no CLY kerogen/heavy-mineral input correction or emitted corrected indicator input exists. The separate unconventional kerogen module is not this pre-correction.
- **Qualifying acceptance tests:** none; corrected-input arithmetic, emitted-curve and provenance assertions are missing. Test class `MISSING`.
- **Supporting tests:** none from neighboring mineral/unconventional code qualify.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** current, test and reachable-history searches found no CLY organic pre-correction.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `gr_kerogen` remains specified ABSENT; no correction endpoint is invented for this deferred capability.
- **Next action:** defer from the pilot; later implement only with explicit supplied fractions and an emitted review curve.

## SB-CLY-048 - Guard the renormalisation denominator

- **Chapter evidence:** P3; historical status `ABSENT`; T33; sections 4.4, 6 and 8.
- **Atomic obligations:** reject a non-positive organic-correction denominator; never clamp; report the condition.
- **Current source:** the organic pre-correction is absent, so no denominator guard or report exists.
- **Qualifying acceptance tests:** none; valid and non-positive denominator controls are missing. Test class `MISSING`.
- **Supporting tests:** generic numeric guards elsewhere do not implement this contract.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no current or reachable renormalization guard was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** must land atomically with SB-CLY-047; no epsilon or clamp is authorized.
- **Next action:** keep absent with the parent capability; when implemented, return a named refusal before division and pin both sides.

## SB-CLY-049 - Do not iterate kerogen and heavy-mineral volumes inside the indicator

- **Chapter evidence:** P3; historical status `ABSENT`; T33; sections 4.4, 6, 7.1 item 16 and 8.
- **Atomic obligations:** accept explicit kerogen/heavy-mineral inputs; never solve them internally; refuse when absent.
- **Current source:** no organic pre-correction exists, so there is neither an explicit-input seam nor an accidental internal iteration.
- **Qualifying acceptance tests:** none; supplied-input use and absent-input refusal are missing. Test class `MISSING`.
- **Supporting tests:** a separate mineral or kerogen solver does not prove the CLY boundary.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no current or reachable CLY iteration/input seam was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** E3 leaves the incumbent iteration behavior open, but SandiBumi's no-iteration boundary is already specified.
- **Next action:** when SB-CLY-047 is scheduled, make both fractions required typed inputs and test body non-entry when either is absent.

## SB-CLY-050 - Where the vendors disagree, ship no default and surface the conflict

- **Chapter evidence:** P0; historical status `PRESENT-DIVERGENT`; T18-T20; sections 4.5, 5, 6 and 8.
- **Atomic obligations:** no default for disputed values; refuse until set; show every competing value and artefact at entry; never interpolate or select silently.
- **Current source:** `vsh_gr` and `vsh_dn` ship multiple numeric endpoint defaults, including values the chapter marks uncited or disputed. `ArgSpec::sources_topic` and a generic source panel exist, but every current CLY argument leaves the topic empty, so the dialog displays neither conflict nor source and runs without explicit entry.
- **Qualifying acceptance tests:** none; the no-default refusal and competing-source UI inventory are missing. Test class `MISSING`.
- **Supporting tests:** N-D and GR arithmetic tests use supplied/current values but do not establish source admissibility; the generic source-panel mechanism is unused by CLY.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** uncited defaults are integrated; the generic disagreement infrastructure commit is reachable but not wired to this domain.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** all fifteen chapter-ABSENT rows stay absent and the NON-ADOPTABLE value remains verification-only; no current default becomes a source.
- **Next action:** withdraw every disputed/uncited default, wire each open parameter to its artefact records and make unsourced evaluation fail before module arithmetic.

## SB-CLY-051 - The vendor artefact path is the primary source string

- **Chapter evidence:** P1; historical status `ABSENT`; T33; sections 4.5, 5, 6 and 8.
- **Atomic obligations:** every shipped default carries a specific artefact/publication/record locator; product name alone is rejected; persist source with the run.
- **Current source:** `ArgSpec` has a generic topic key, not a per-value primary source field. Current CLY parameters leave it empty, ship defaults, and serialize only numeric request values; no CLY source locator reaches the run record.
- **Qualifying acceptance tests:** none; zero-exception source inventory, generic-name rejection and run persistence are missing. Test class `MISSING`.
- **Supporting tests:** the generic parameter-source panel can render other domains' topics, but it is not evidence for any CLY default.
- **Manual evidence:** shale-volume 0/17; workflow 0/23; processing-history 0/7.
- **Git evidence:** generic source infrastructure is integrated; domain adoption is absent and no CLY source locator was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** protected artefacts were not opened; the chapter's recorded locator is evidence, and missing locators keep values absent.
- **Next action:** add a source-or-ABSENT field to CLY parameter custody and reject generic vendor labels and missing source at both entry and persisted-run validation.

## SB-CLY-052 - Import by ordinal **and** semantic key

- **Chapter evidence:** P2; historical status `ABSENT`; T33; sections 4.5, 6 and 8.
- **Atomic obligations:** vendor parameter import matches both ordinal and semantic key; disagreement refuses before assignment.
- **Current source:** no vendor parameter-set importer, ordinal schema or CLY semantic-key mapping exists.
- **Qualifying acceptance tests:** none; matching and shifted-ordinal refusal fixtures are missing. Test class `MISSING`.
- **Supporting tests:** generic text/LAS import paths do not import vendor CLY parameter sets.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** current, test and reachable-history searches found no vendor CLY parameter import.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** requires a supported import-format decision and source identity; no ordinal map is inferred from protected data.
- **Next action:** decide pilot migration scope, then define an explicit versioned schema and fail any ordinal/key disagreement before producing a run request.

## SB-CLY-053 - Matrix travel time is module-scoped and carries its artefact

- **Chapter evidence:** P2; historical status `ABSENT`; T25; sections 4.5, 5, 6 and 8.
- **Atomic obligations:** travel time scoped to its consuming module; persist artefact; no shared default where witnesses disagree.
- **Current source:** no sonic-bearing CLY module or travel-time argument exists, so no module scope, source field or disagreement refusal exists.
- **Qualifying acceptance tests:** none; two-module non-sharing and source/no-default assertions are missing. Test class `MISSING`.
- **Supporting tests:** sonic porosity parameters in another module are not CLY source custody and must not be reused by name.
- **Manual evidence:** shale-volume 0/17; workflow 0/23.
- **Git evidence:** no current or reachable CLY travel-time parameter was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the chapter records conflicting witnesses and explicitly ships all matrix travel-time defaults ABSENT.
- **Next action:** if SB-CLY-022 enters the pilot, define a module-local open value plus artefact identity and prove it cannot leak into another module.

## SB-CLY-054 - Unit-typed quantities; no magic scale constants

- **Chapter evidence:** P0; historical status `PARTIAL`; T21/T42; sections 4.5, 5, 6 and 8.
- **Atomic obligations:** every quantity has a manifest unit; named tested conversions; persist source unit and conversion; no unexplained scale factor.
- **Current source:** current CLY args/outputs carry unit strings and the shared curve layer has named density/neutron/sonic conversions. However CLY parameter values do not carry source units or conversion records; a current density default appears only in house units after an unstated scale conversion.
- **Qualifying acceptance tests:** none; complete CLY quantity inventory and source-unit run record are missing. Test class `MISSING`.
- **Supporting tests:** `unit_conversions_only_when_needed` passed exactly once, including the independently known density-unit conversion, but no CLY parameter invokes or records it.
- **Manual evidence:** shale-volume 0/17; generic-curve-store 0/18; workflow 0/23.
- **Git evidence:** manifest units and generic conversion identities are integrated; CLY source-unit custody is absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no scale constant is open; the known unit identity may be used only through the named conversion with source-unit evidence.
- **Next action:** type every CLY parameter/output, retain artefact units on entry, and persist the named conversion alongside the canonical value.

## SB-CLY-055 - LAS null discipline on every domain curve

- **Chapter evidence:** P1; historical status `PARTIAL`; T35/T44; sections 4.5, 6 and 8.
- **Atomic obligations:** values, flags and provenance round-trip with absences intact; declared sentinel in header; provenance tokens survive as a curve.
- **Current source:** every registered exporter requires writer settings and emits the project-declared sentinel; the default LAS path re-imports numeric values correctly and parser rules preserve declared/per-channel null semantics. CLY has no provenance curve or family and no domain-wide values/flags/provenance round-trip fixture.
- **Qualifying acceptance tests:** none; generic numeric round trip does not close missing CLY provenance or the T44 conflict. Test class `MISSING`.
- **Supporting tests:** all three export controls and all three parser null-policy controls passed exactly once.
- **Manual evidence:** las-export 0/2; shale-volume 0/17; processing-history 0/7.
- **Git evidence:** generic declared-sentinel custody is integrated; CLY provenance export/import is unimplemented.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-CLY-031, SB-CLY-032 and SB-CLY-046; the writer sentinel remains project-declared, never hard-coded per format.
- **Next action:** after typed provenance exists, add one all-output LAS round trip using explicit writer settings and prove values, NaNs, flags and tokens survive unchanged.

## Test-intent classification summary

- All 44 chapter intentions remain routed exactly once as primary evidence: T01→001, T02→004, T03→005, T04→004, T05→002, T06→002, T07→009, T08→007, T09→007, T10→010, T11→008, T12→003, T13→003, T14→006, T15→011, T16→012, T17→013, T18→014, T19→015, T20→016, T21→018, T22→019, T23→020, T24→021, T25→022, T26→023, T27→024, T28→026, T29→027, T30→028, T31→029, T32→030, T33→031, T34→032, T35→055, T36→035, T37→036, T38→037, T39→038, T40→040, T41→044, T42→045, T43→046, T44→034.
- Cross-support is preserved without double-counting: 017/025/042/047/048/049/051 use T33; 033 uses T36; 039 uses T38; 041/043 use T43; 050 uses T18-T20; 053 uses T25; 054 uses T21/T42; 055 uses T35/T44.
- No executable body closes any complete T01-T44 sentence. Row-level test classes are therefore 52 `MISSING` and 3 `CHARACTERIZATION` (004's rounded Larionov behavior, 006's T14 numeric limb, and 039's legacy preset behavior); there is no whole-contract `CORRECTNESS` row.
- Twenty-four focused candidate tests were compiled and run by full Cargo path with `--exact`; every credited filter produced exactly one `test ... ok` line. The three initial parser short names selected zero tests and were rejected as evidence; their nested full names were resolved from `cargo test -- --list` and rerun successfully.

| Exact candidate | Result | Credited surface and oracle class |
|---|---:|---|
| `modules::tests::vsh_gr_linear_and_limits` | 1 passed | supplied-endpoint linear arithmetic and limited/unlimited twins; narrow correctness support only |
| `modules::tests::the_vsh_gr_labels_agree_with_the_coefficients_they_describe` | 1 passed | stable IDs, rock-age labels and no rock age on `LARINOV3`; cited mapping support, no run provenance |
| `modules::tests::every_vsh_gr_transform_lands_on_its_published_coefficient` | 1 passed | current fixed transforms and rounded Larionov closure miss; characterization where it conflicts with exact normalization |
| `modules::tests::vsh_dn_flags_offmodel_and_gr_divergence` | 1 passed | current N-D reliability flag on supplied fixtures; no canonical-form or source proof |
| `modules::tests::vsh_dn_degenerate_triangle_is_missing_not_inf` | 1 passed | internal NaN/no-infinity safety; not an observable refusal |
| `modules::tests::badhole_flags_washout_and_drho` | 1 passed | good, DRHO and over-gauge current detector branches; no under-gauge control |
| `modules::tests::condflag_detects_coal_tight_and_crossover` | 1 passed | current detector arithmetic from explicit fixture inputs; no CLY coal branch |
| `modules::tests::condflag_washout_is_not_coal_and_xcond_option` | 1 passed | current bad-hole veto and mask option; no per-indicator provenance |
| `modules::tests::gr_normalize_reference_defaults_are_generic_not_a_field_calibration` | 1 passed | legacy manifest defaults derived from current code; characterization, not parameter authority |
| `modules::tests::gr_normalize_maps_well_percentiles_to_reference` | 1 passed | supplied-reference affine arithmetic; no endpoint record |
| `condition::tests::normalize_refuses_a_reference_pair_it_was_not_given` | 1 passed | universal no-default refusal; correctness support for a neighboring seam |
| `workflow::tests::mask_excludes_flagged_samples_from_gr_normalize_percentiles` | 1 passed | current pre-statistic masking; no CLY absence reason |
| `workflow::tests::gr_normalization_anchors_each_well_on_its_own_percentiles` | 1 passed | per-well normalization isolation; no named endpoint pool/record |
| `workflow::tests::every_module_returns_the_output_keys_its_manifest_declares` | 1 passed | generic output-manifest parity; absent CLY outputs cannot be credited |
| `workflow::tests::a_masked_washout_defeats_the_very_module_meant_to_repair_it` | 1 passed | explicit current defect characterization; not correct CLY masking |
| `plotting::tests::a_plot_derived_parameter_write_is_undoable_and_requires_complete_non_null_provenance` | 1 passed | generic undoable/source-complete plot write; no CLY percentile/value binding |
| `curves::tests::families_resolve_common_mnemonics` | 1 passed | current family inventory and merged GR aliases; CLY families absent |
| `curves::tests::unit_conversions_only_when_needed` | 1 passed | independently derived generic unit conversions; no CLY source-unit custody |
| `export::tests::a_declared_sentinel_reaches_every_registered_writer_and_no_writer_emits_its_own` | 1 passed | current DIO writer-sentinel correctness; no CLY provenance curve |
| `export::tests::the_default_export_format_honours_the_sentinel_and_an_incapable_format_is_marked` | 1 passed | declared-sentinel/default-format capability correctness |
| `export::tests::an_exported_las_reimports_with_the_same_values` | 1 passed | generic numeric LAS round trip; not values/flags/provenance inventory |
| `parsers::las_depth_tests::null_recognition_is_one_relative_tolerance_transform_and_recognition_never_rewrites` | 1 passed | declaration-based recognition without amplitude rewrite |
| `parsers::las_depth_tests::two_channels_with_different_plural_nulls_are_screened_against_their_own_values_only` | 1 passed | per-channel null-rule isolation |
| `parsers::las_depth_tests::one_null_exception_entry_keeps_all_six_name_patterns_active_and_no_null_is_not_unset` | 1 passed | explicit exception and `NoNull` behavior; prevents a global `-999` shortcut |

## Manual capability evidence

- Jauhar explicitly owns and will perform all manual review. This documentation-only increment checked no scenario and did not modify `REVIEW.md` or the generated matrix.
- `shale-volume`: not exercised, 0/17.
- `conditioning`: not exercised, 0/27.
- `workflow`: not exercised, 0/23.
- `generic-curve-store`: not exercised, 0/18.
- `las-export`: not exercised, 0/2.
- `processing-history`: not exercised, 0/7.
- `histogram`: partially exercised, 5/22; those checks do not prove CLY endpoint custody.
- `crossplot`: partially exercised, 6/13; those checks do not prove two-way endpoint binding, type safety or run provenance.
- Consequently pilot field evidence remains `OPEN` even if the automated gate is green. UI availability, compile success and a desktop harness do not replace Jauhar's actual workflow review.

## Open decisions, source gaps, and hard refusals

- **E1:** M-N remains computation-free. The conflicting line constant is neither selected nor repeated into runtime authority; a primary chart or controlled live-vendor fixture is required. The deliberate-exclusion explanation is still missing from the module library.
- **E2:** `LARINOV3` remains a product-owner decision between warned parity retention and removal with saved-run handling. This receipt records current divergence but chooses neither path.
- **E3:** live-vendor questions remain queued. They can improve parity evidence but do not override the independently specified SandiBumi guards, clip order, zero handling or canonical form.
- **E4:** the three primary transform papers remain an acquisition opportunity. The chapter states no implementation requirement depends on obtaining them; held vendor artefacts remain the current source tier.
- **E5:** the Stieber engineering epsilon remains absent. No value was inferred from current hard-coded clamps, floating-point limits or model knowledge.
- All fifteen chapter parameters marked `ABSENT - ships with no default` remain absent requirements. The one `NON-ADOPTABLE - cited for verification` value remains verification-only.
- The undeclared bare-`-999` clause in T44 conflicts with the newer declaration-based SB-DIO contract. This increment neither weakens DIO nor silently declares CLY correct; it records a pilot-blocking cross-domain adjudication need.
- Protected installed-vendor charts/resources were not opened, transcribed, imaged or used as defaults. The chapter's recorded source locators and independently executable current tests are the boundary of this receipt.
- No current default, neighboring module value, local realization or implementation literal was treated as a citation.

## Measured totals and completeness guard

- As-built: 27 `ABSENT`, 15 `PARTIAL`, 13 `PRESENT-DIVERGENT`, 0 `PRESENT-UNVERIFIED`, 0 `PRESENT-OK`.
- Release disposition: 40 `PILOT-BLOCKER`, 8 `UNDECIDED`, 7 `DEFERRED`, 0 `OUT`.
- Risk class: 21 `DATA-INTEGRITY`, 19 `SILENT-WRONGNESS`, 7 `REQUESTED-CAPABILITY`, 7 `LATER`, 1 `DEGRADED-RESULT`.
- Test class: 3 `CHARACTERIZATION`, 52 `MISSING`, 0 whole-contract `CORRECTNESS`.
- Commit state: 27 `INTEGRATED`, 28 `UNIMPLEMENTED`.
- Ledger after this domain: 279/931 adjudicated, 652 unadjudicated and 207 total pilot-blocker dispositions.
- Completeness: all 55 IDs appear once; no CLY row remains `UNADJUDICATED`; every adjudication-owned mandatory field is populated; source-owned PRD audit passed; all T01-T44 intentions remain routed once.
- Boundary: no production code, executable test, PRD, research dossier, protected vendor resource, manual verification record or petrophysical value changed in this increment.
