# Gate 1 SB-CORE live adjudication

- Branch: `codex/g1-sb-core-adjudication`
- Adjudication start HEAD: `95393511425c1b90fc937e1806c15eb8e916bf56`
- Accepted evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`
- `origin/master` at evidence freeze: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Adjudication date: `2026-08-10`
- Worktree at evidence freeze: clean; `D:\XX. SandiBumi` was the only registered Git worktree.
- Row guard: passed - exactly 25 planned SB-CORE rows, all 25 initially `UNADJUDICATED`, with 14 rows lacking a chapter-owned acceptance-test ID.
- Evidence boundary: this receipt classifies the accepted tree. It does not amend PRD v2, supply a missing parameter, promote automated evidence to field evidence, or approve pilot scope on Jauhar's behalf.
- Source-navigation boundary: the codebase index was not callable in this task, so exact-file `rg` searches and direct source reads were used as the declared fallback. Consequential negative findings were checked against the expected directories, tests, Git history and generated verification matrix.
- Verification boundary: focused tests named below passed. The final repository gate is recorded in `STATUS.md` and the adjudication commit receipt; a passing existing worktree is not treated as fresh-clone or field evidence.

## SB-CORE-001 — Depth unit is a first-class, carried property

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-CORE-T01`, `SB-CORE-T01b`, `SB-CORE-T02`.
- **Atomic obligations:** carry the declared depth unit through every import, storage path, depth-dependent computation and export; refuse depth-dependent computation when the project unit is undeclared.
- **Current source:** `src-tauri/src/units.rs` provides `DepthUnit` and the NIST SP 811 exact foot conversion; LAS, Intake and DLIS parsing/reconciliation carry units; `satheight.rs` now normalizes both Leverett-J and Skelt-Harrison inputs; `export.rs` declares and self-checks the LAS unit. The universal refusal is not satisfied: `workflow.rs` refuses an undeclared unit only for `sw_height`, while other modules can fall back to metres. No complete registry marks every depth-dependent module/importer/exporter.
- **Qualifying acceptance tests:** `satheight.rs::saturation_height_is_identical_whichever_unit_the_project_declares`, `satheight.rs::skelt_harrison_is_identical_whichever_unit_the_project_declares`, and `workflow.rs::a_depth_dependent_module_refuses_when_the_project_depth_unit_is_undeclared` passed. These are `CORRECTNESS`; the conversion expectation is sourced to NIST SP 811 and the refusal expectation to the chapter contract.
- **Supporting tests:** `export.rs::the_las_writer_declares_the_unit_it_wrote_for_both_feet_and_metres` and LAS re-import tests exercise the export side but cannot prove the word “every.”
- **Manual evidence:** `data-conventions` 0/45, `las-import` 0/57, `dlis-import` 0/11, `saturation-height` 0/6, `las-export` 0/2 - all unexercised.
- **Git evidence:** `bb807ca` is reachable from the accepted anchor; accepted source still contains the generic-module refusal gap.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** no new numerical source is needed. The missing item is a complete depth-dependency inventory and generic enforcement path.
- **Next action:** add a machine-readable depth-dependency flag to the module/import/export inventories, refuse undeclared project units generically, and pin both dependent and independent modules so the guard cannot become universal by accident.

## SB-CORE-002 — A degraded or failed result is never presented as a clean one

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; owned tests `SB-CORE-T03` through `SB-CORE-T09`.
- **Atomic obligations:** each of the original seven reporting surfaces must expose degraded/failing state to the user or persisted run record; each surface also needs a clean or successful control so an implementation that labels everything degraded cannot pass.
- **Current source:** the seven non-overlapping contracts are implemented across `core_reporting_tests.rs`, `ingest.rs`, `report.rs`, `src/ui/reportingHonesty.ts`, `summaryDialog.ts`, `dashboardPanel.ts` and `mlDialog.ts`. The surfaces cover Monte Carlo chain failure, all-channel import failure, pay-summary failure, partial/all-failed ML, uninterpreted versus real-zero pay, statistics-only dashboard output and training wells that contributed no samples.
- **Qualifying acceptance tests:** the Rust T03/T04/T05 reporting tests passed, and the four targeted frontend T06-T09 contracts passed. They are `CORRECTNESS`: expected states come from the chapter's explicit reporting-surface contracts and fixtures assert both failure/degraded and clean/success sides.
- **Supporting tests:** internal `Result` tests were not counted as closure.
- **Manual evidence:** `monte-carlo` 2/14 and `machine-learning` 7/189 are partial; `report` 6/53 is partial; `las-import` 0/57 and `workflow` 0/23 are unexercised.
- **Git evidence:** `78bd21d`, `d25c274` and accepted merge `02b59ea` are reachable from the accepted anchor.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract); `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the automated contract; field exercise remains open.
- **Next action:** make no production change. Preserve T03-T09 and exercise the applicable reporting surfaces during the representative pilot workflow.

## SB-CORE-003 — Validity conditions are enforced preconditions

- **Chapter evidence:** P1; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** every method's validity conditions must be machine-readable, evaluated before computation, and cause an actionable refusal when invalid.
- **Current source:** `modules.rs::ValidityCondition` now serializes source-bearing enumeration, per-sample numeric-range, branch-conditional, required-companion and relational rules. The public `run_module` boundary evaluates them before dispatch, so dialog, saved-chain, batch, Monte Carlo and future callers cannot bypass the same gate. `vsh_gr` carries cited method-id, endpoint-range and endpoint-order conditions; `ipc.ts` and `moduleDialog.ts` carry and display each statement, activation branch and source. Legacy `ArgSpec.min/max` fields are deliberately not promoted into scientific validity conditions without an explicit source.
- **Qualifying acceptance tests:** `modules.rs::source_bearing_precondition_shapes_refuse_before_computation_while_a_valid_public_run_still_computes` is `CORRECTNESS`. It round-trips every rule shape, proves the cited 8-13 versus 8-18 lb/gal branch distinction on a synthetic manifest explicitly marked NON-ADOPTABLE, refuses a missing companion and an empty required parameter frame, pins invalid enumeration/range/endpoint-order paths with source-bearing messages, and proves a valid public VSH run still computes. It introduces no product default and deliberately does not claim the still-open whole-pilot inventory.
- **Supporting tests:** the existing workflow SSPW fallback proof and bounded Monte Carlo reproducibility/scoping/persistence fixtures confirm that all callers still reach the central runner. They do not replace the missing whole-pilot method inventory.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** this Gate 2 increment owns the generic schema, dispatch gate, frontend contract and the first live pilot-method conditions; exact-tree verification is recorded in `STATUS.md` and `REVIEW.md`.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 progress state `BLOCKED`.
- **Blocker or decision:** `DEC-003` is resolved. Closure is blocked by the word **every**: `SB-CORE-004` must finish source/default custody, and the selected ENV/CLY/POR/SAT/CUT methods must populate their own cited validity conditions. The framework supplies no missing endpoint and the recorded 8-13/8-18 vendor ranges remain NON-ADOPTABLE test evidence only.
- **Next action:** populate and prove each selected method's cited valid/invalid conditions during its owning Gate 2 row, then run a whole-pilot registry audit and move this row from `BLOCKED` to `DONE` only when no selected method is unaccounted for.

## SB-CORE-004 — No parameter ships without a source

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-CORE-T10`, `SB-CORE-T11`.
- **Atomic obligations:** every registered default has a machine-readable source string; the build fails on a default with no source; a required source-less parameter ships absent and the run refuses until supplied.
- **Current source:** every registered numeric parameter now carries `ArgSpec::default_source`: a named source beside a finite default or the exact `ABSENT` token beside an empty value. `module_catalog()` runs the universal registry gate, `run_module()` refuses required absent values before dispatch, branch-specific absent requirements activate only for the method that consumes them, and the dialog renders the same custody state.
- **Qualifying acceptance tests:** `a_registered_default_without_a_source_fails_the_build_gate` pins the invalid fixture and complete live registry; `an_absent_required_parameter_refuses_until_the_interpreter_supplies_a_value` pins ABSENT, refusal, supplied-value success and active/inactive branch behavior. Test class is `CORRECTNESS`.
- **Supporting tests:** the affected workflow, chain and Monte Carlo fixtures now supply explicit characterization inputs rather than recovering withdrawn manifest values; the complete Rust and frontend suites remain green.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** the universal source/ABSENT schema, migrated registry, build/runtime enforcement and exact T10/T11 proofs are integrated on the Gate 2 topic branch; the exact-tree receipt is recorded in `STATUS.md` and `REVIEW.md`.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`; Gate 2 progress state `DONE`.
- **Blocker or decision:** none for the automated contract. No replacement value was selected for an uncited parameter; those entries remain visibly absent.
- **Next action:** preserve the registry gate and exercise cited-default plus ABSENT/refusal paths during Gate 4 without converting that pending field evidence into an implementation gap.

## SB-CORE-005 — Vendor-derived defaults are re-sourced to primary literature

- **Chapter evidence:** P1; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** every vendor-derived default used by SandiBumi must be independently sourced to primary literature rather than copied or carried from a neighbouring vendor.
- **Current source:** `multimin2.rs` exposes a 27-entry component library introduced in baseline commit `a659096` already described as merged from two vendor installs; `Component`/`LibRow` has no per-value source field. The preserved IP and reference tables show that individual shipped rows combine values which occur in different vendor libraries, while the later `1216a99` change only scrubbed vendor names and added no construction record. `docs/IP_PROVENANCE.md` records the same primary-literature re-sourcing gap and identifies it as before-first-sale work.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** library-shape and solver tests validate behavior against the current table and therefore cannot prove the table's provenance. A new green test over the same unsourced constants would be a characterization snapshot, not the required custody proof.
- **Manual evidence:** `sandimin` 0/28 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** history re-verification found no pre-merge table or per-value construction map: baseline commit `a659096` is the file's introduction, and the later vendor-name scrub does not recover origin. No production change is admissible from the evidence held; commit state remains `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 progress state `BLOCKED`.
- **Blocker or decision:** exact per-value origin is not recoverable from the recorded evidence. Equal numbers across vendor tables are corroboration, not proof of which library supplied the shipped value, and some values are merged within a physical row. Primary-literature citations are also incomplete, while `CLAIM-012` keeps the curated selection under legal review before first sale. Inventing a generic `VENDOR-DERIVED` source would satisfy the type while defeating the contract.
- **Next action:** rebuild coherent source libraries from held primary references or exact vendor assets, carry provenance per value through UI and deliverables, ship every unresolved value ABSENT, add `SB-MIN-T09`, and obtain counsel's disposition on the curated selection.

## SB-CORE-006 — One name, one equation

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-CORE-T17`, `SB-CORE-T18`.
- **Atomic obligations:** one method name must resolve to one equation across engines; emitted mnemonic, UI label and documentation must identify that same method.
- **Current source:** `modules.rs` registers `sw_sim` with one modified Simandoux expression, while `multimin2.rs` documents and computes “Modified Simandoux (Bardon-Pied)” with an additional `(1-VSH)` denominator; the UI presents the latter as “Simandoux (modified).” Thus one name still selects two equations. No shared method identity binds numeric behavior and labels.
- **Qualifying acceptance tests:** no executable T17 numeric-parity or T18 naming-parity mapping was found; test class is `MISSING`.
- **Supporting tests:** per-engine numeric tests only prove each implementation against itself and cannot adjudicate which name/equation is correct.
- **Manual evidence:** `workflow` 0/23, `equation-engine` 0/11 and `sandimin` 0/28 - unexercised.
- **Git evidence:** the divergent implementations are integrated at the accepted anchor; no closing commit is present.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the cited method identity must be adjudicated from its named source; this pass does not alter or choose an equation.
- **Next action:** assign distinct source-backed identifiers where the equations differ, route labels/mnemonics through those identifiers, and add independent numeric plus UI/export naming tests.

## SB-CORE-007 — One definition for every constant and every transform

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-CORE-T19`, `SB-CORE-T20`, `SB-CORE-T23`.
- **Atomic obligations:** each physical constant and transform has one canonical definition; deliberate duplicate implementations agree on a shared vector; every producer of the same output uses the same sourced default.
- **Current source:** module/solver inventories still carry conflicting GR endpoint pairs, two gas-correction expressions, and duplicated mineral-density constants with different precision. The SSC and SSPW paths also contain separate gas branches. No canonical registry or cross-producer inventory resolves those differences.
- **Qualifying acceptance tests:** no executable T19/T20/T23 mapping proving all constants, transforms and output producers was found; test class is `MISSING`.
- **Supporting tests:** method-local fixtures validate isolated copies and cannot prove global parity.
- **Manual evidence:** `workflow` 0/23, `equation-engine` 0/11, `sandimin` 0/28 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** divergent definitions are integrated at the accepted anchor; no universal closing commit is present.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** each canonical value or transform still requires its own cited authority. No existing copy is promoted merely because it is common.
- **Next action:** inventory every producer, adjudicate one source-backed definition per concept, route duplicates through it, and pin shared-vector parity plus default/mnemonic parity from both sides.

## SB-CORE-010 — Every computed curve answers "how was I made?"

- **Chapter evidence:** P1; chapter status `ABSENT`; owned tests `SB-CORE-T14`, `SB-CORE-T15` as defined by the ancestry sentences in this section.
- **Atomic obligations:** every computed curve records module and version, every input and log set, every parameter and source, zone, operator and timestamp; the UI shows it on demand; the complete ancestry travels into every deliverable; it survives project save/load.
- **Current source:** versioned `log_sets` record module, `params_json`, `inputs_json` and creation time, and the UI exposes log-set history. LAS export carries method/parameters/inputs/set/version/date and refuses computed curves without a live ancestry record. Missing universal elements include module version, parameter-source strings, zone definition and operator; complete ancestry is not carried through all reports/office artifacts; not every computed writer has been inventoried.
- **Qualifying acceptance tests:** no executable T14 every-writer ancestry test or T15 project round-trip test was found; test class is `MISSING`.
- **Supporting tests:** LAS provenance tests cover one deliverable and log-set tests cover one store path; neither proves every producer and deliverable.
- **Manual evidence:** `processing-history` 0/7, `database-tools` 0/2 and `verification-stewardship` 0/24 are unexercised; `project-lifecycle` is partial at 3/24.
- **Git evidence:** the partial log-set/LAS foundation is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `SB-CORE-004` is a prerequisite and `DEC-003` must identify the representative pilot workflow. `SB-CORE-010` itself defines the minimum ancestry record; `DEC-009` governs only additional lineage beyond that minimum.
- **Next action:** after `DEC-003`, inventory every pilot curve writer and number-carrying deliverable, implement the exact `SB-CORE-010` ancestry fields, and add complete-ancestry plus project/deliverable round-trip acceptance tests without changing the `computed_curves` write discipline. Route only additional lineage to `DEC-009`.

## SB-CORE-011 — A project re-runs byte-identically

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-CORE-T16` as defined here.
- **Atomic obligations:** rerun from raw import through pay summary produces byte-identical output blobs and identical provenance; every deliberate non-determinism is seeded and the seed recorded.
- **Current source:** Monte Carlo and ML paths use deterministic seeds, several method tests pin deterministic behavior, and `training_fingerprint` identifies ML training matrices. The ignored `test_full_deterministic_chain` runs a real-data chain but does not compare full-project output bytes, provenance and pay summary across two independent reruns. No gate test implements T16's complete boundary.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** deterministic module/seed tests and the ignored real-data chain prove narrower claims. The ignored chain needs optional local field fixtures and is not renamed into byte-identical proof.
- **Manual evidence:** `project-lifecycle` 3/24 is partial; `workflow` 0/23 and `processing-history` 0/7 are unexercised.
- **Git evidence:** deterministic foundations are integrated at the accepted anchor; the complete rerun contract is unimplemented.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `SB-CORE-010` and `DEC-003` are prerequisites; the representative workflow/fixture has not been selected.
- **Next action:** define the representative pilot chain, run it twice from raw import in isolated projects, and compare curve bytes, recorded provenance and pay-summary bytes while recording every seed.

## SB-CORE-012 — Named interpretation scenarios with A/B diff

- **Chapter evidence:** P2; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** persist a named parameter set, rerun it, compare it with another named set, and report both changed parameters and changed numerical results.
- **Current source:** no persisted interpretation-scenario entity, scenario rerun command, or independent A/B parameter-and-result diff was found. Side-by-side plot and UI comparisons are transient views, not named scenarios.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** none closes the persisted scenario contract.
- **Manual evidence:** `project-lifecycle` 3/24 is partial; `workflow` 0/23 and `results-qc` 0/1 are unexercised.
- **Git evidence:** no implementation or superseded accepted implementation was found; commit state `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must decide whether named A/B scenarios belong in the paid pilot after `DEC-003` defines the workflow.
- **Next action:** if approved, specify one persisted scenario record and an independently computed parameter/result diff, then implement save, rerun and comparison as a separate requirement increment.

## SB-CORE-013 — Vendor divergence is visible at the point of choice

- **Chapter evidence:** P2; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** every corpus-recorded vendor disagreement is visible with values, sources and tiers at the exact editor choice; the interpreter's selected value/source is persisted as a decision.
- **Current source:** `param_sources.rs` contains one contested topic, cluster count, with four sourced vendor positions plus SandiBumi's own disclosed default. `moduleDialog.ts` displays `sources_topic` at the field; ML presents the cluster-count sources and returns a decision note. Only one corpus topic is represented, and the ML decision note is returned in transient `MlResult.notes` rather than persisted into the curve/log-set run record. `param_sourced()` is used only twice.
- **Qualifying acceptance tests:** none; by the no-owned-test rule, test class is `MISSING`.
- **Supporting tests:** `every_competing_value_names_its_product_and_the_absence_of_one_is_itself_shown` and `the_recorded_choice_says_which_cited_values_it_agrees_with_and_when_it_agrees_with_none` prove the one-topic data and note generation, not full corpus coverage or persistence.
- **Manual evidence:** `machine-learning` 7/189 is partial; `verification-stewardship` 0/24 is unexercised.
- **Git evidence:** the one-topic UI and note mechanism is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` determines which contested pilot parameters need first coverage; source entries remain limited to corpus evidence.
- **Next action:** inventory all recorded disagreements for the chosen pilot methods, persist the selected value and source in the run record, and test both the point-of-choice display and saved decision.

## SB-CORE-014 — A learned model carries its training provenance

- **Chapter evidence:** P1; chapter status `ABSENT`; owned tests `SB-CORE-T21`, `SB-CORE-T22`.
- **Atomic obligations:** record training rows by well/depth/log set, ordered features, every effective hyperparameter, random seed, fitted preprocessing state, artifact identity hash and library versions; reproduce from that record alone or refuse naming any missing element.
- **Current source:** `ml.rs::TrainingRecord` stores contributing wells, row/mask/incomplete counts and log-set id/version; `params_json` stores effective defaults and seeds; `metrics_json` stores interval/output resolution; the joblib blob stores scaler, model, ordered features and feature transforms; `runtime_json` stores Python/numpy/scipy/sklearn/joblib/xgboost versions; saved-model apply never refits and checks feature order. The model row does not persist the artifact's content hash (LAS export computes one later), apply still takes an interval from its request instead of restoring a complete prediction record, and missing legacy training/runtime fields are often treated as unknown or warned rather than refused. No replay-from-record-only path proves every enumerated field.
- **Qualifying acceptance tests:** no complete executable T21 replay or T22 any-missing-element refusal was found; test class is `MISSING`.
- **Supporting tests:** training-fingerprint stability, log-set drift, runtime drift, ordered-feature, saved-model application and optional-package ignored end-to-end ML tests prove important subsets only.
- **Manual evidence:** `machine-learning` 7/189 and `project-lifecycle` 3/24 are partial; `processing-history` 0/7 is unexercised.
- **Git evidence:** the substantial saved-model provenance foundation is integrated at the accepted anchor.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` decides whether learned models are in the pilot; `DEC-009` defines required lineage granularity. No ML production edit is authorized in this increment.
- **Next action:** persist an artifact hash and complete prediction/training selection, make replay consume the record rather than caller restatement, refuse each missing required element by name, and add exhaustive T21/T22 tests in the owning ML increment.

## SB-CORE-015 — No artifact ships that SandiBumi's own reader rejects

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned tests `SB-CORE-T14`, `SB-CORE-T15`, `SB-CORE-T16` as defined by this writer-round-trip section.
- **Atomic obligations:** every file writer's output is accepted by its own reader; values round-trip; declared units/null/index conventions match bytes; every writer is tested on a non-default fixture.
- **Current source:** `export.rs` has a registered LAS 2.0 writer whose shared wrapper invokes the SandiBumi reader and treats a rejected round trip as an error. LAS tests cover feet/metres, nulls/index and non-default conventions. The writer inventory covers only the LAS data writer; office/project/report artifacts are outside it, and no DLIS export/T15 round trip exists. Therefore the universal “every file” inventory is incomplete even though the LAS defect is closed.
- **Qualifying acceptance tests:** `an_exported_las_reimports_with_the_same_values`, `the_las_writer_declares_the_unit_it_wrote_for_both_feet_and_metres`, and `every_registered_writer_reads_its_output_and_a_rejected_round_trip_is_an_error` are `CORRECTNESS`, using the declared fixture values and NIST SP 811 conversion. T15 remains missing.
- **Supporting tests:** optional-package office round trips are ignored correctly but do not provide an always-green universal writer inventory.
- **Manual evidence:** `las-export` 0/2, `las-import` 0/57, `dlis-import` 0/11 and `office-deliverables` 0/39 are unexercised; `project-lifecycle` 3/24 and `report` 6/53 are partial.
- **Git evidence:** the LAS writer/reader discipline is integrated at the accepted anchor; no complete all-writer/DLIS closing commit exists.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** a complete shipped-writer inventory and the intended DLIS export boundary are unresolved; no format convention is invented here.
- **Next action:** register every shipped number-carrying writer or define its external-reader contract, add non-default round trips, and implement T15 only if a DLIS writer is an approved shipped capability.

## SB-CORE-030 — Portfolio-scale target is declared and measured

- **Chapter evidence:** P1; chapter status `UNMEASURED` (invalid as-built vocabulary retained verbatim); no chapter-owned acceptance-test ID.
- **Atomic obligations:** state well count, project size, named operations, fixture, hardware and usability thresholds; demonstrate that exact target before customer-facing copy claims it.
- **Current source:** `CLAIM-001` records the 2,000+ statement as unmeasured. The ignored 100-well stress test and historical 540-well project-open observation prove different, narrower scenarios; neither defines the required fixture, hardware, operations and thresholds. No approved target exists.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** `pipeline_field_100well_stress` is ignored and prints observations without acceptance thresholds; it is not renamed into portfolio qualification.
- **Manual evidence:** `portfolio-performance` 0/50 - unexercised.
- **Git evidence:** no declared-and-measured target implementation exists; commit state `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `UNDECIDED`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must resolve `DEC-004` and `DEC-008`: remove/hold the customer claim or define the exact target, operations, fixture, hardware and thresholds. No number is selected here.
- **Next action:** make the owner decision first; if the claim remains, write and execute the named benchmark receipt before any customer copy uses the scale statement.

## SB-CORE-031 — A benchmark harness exists and is part of the gate

- **Chapter evidence:** P1; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** performance figures come from a repeatable committed harness, and that harness runs in the green gate.
- **Current source:** no benchmark/performance harness exists under `tools` or a benchmark directory. `pipeline_field_test.rs` contains ignored real-data and 100-well stress tests, but they depend on local fixtures, print timings without accepted thresholds, and default `cargo test` does not run them. `tools/check.ps1` runs ledger, matrix, frontend and default Rust tests only.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** ignored field/stress tests are diagnostic observations, not a benchmark contract.
- **Manual evidence:** `portfolio-performance` 0/50 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** no qualifying harness is integrated; commit state `UNIMPLEMENTED`.
- **Verdict:** `ABSENT`; `UNDECIDED`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on `SB-CORE-030` and `DEC-008`; without named operations/fixture/hardware/thresholds, a harness would encode invented policy.
- **Next action:** after the target is approved, implement one repeatable benchmark command with machine-readable thresholds and invoke it from the release gate.

## SB-CORE-032 — The compute path does not hold the global lock across long work

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** work scaling with well/sample count must not hold the global DuckDB mutex for its duration; long work runs off the main event-loop thread while preserving the single-writer discipline.
- **Current source:** `lib.rs` currently has 137 textual `db.0.lock()` sites. Several commands are off-thread, but lock duration remains wrong: LAS import locks one connection across all files, and core import locks across parsing/import work that scales with sample count. The code does not consistently separate short snapshot read, lock-free compute and short transactional commit phases.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** job/off-thread tests prove event-loop scheduling for selected commands, not mutex hold duration or concurrency.
- **Manual evidence:** `portfolio-performance` 0/50, `workflow` 0/23 and `database-tools` 0/2 - unexercised.
- **Git evidence:** off-thread job infrastructure is integrated, but the accepted source still contains the lock-scope divergence.
- **Verdict:** `PRESENT-DIVERGENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` must identify the pilot's long operations. The single-writer mutex and PK-less `computed_curves` discipline are not candidates for relaxation.
- **Next action:** select one pilot-critical long operation, split it into bounded snapshot/compute/commit phases, and add a concurrency test proving another read can progress while computation runs.

## SB-CORE-033 — Compute results are cached on content, not recomputed

- **Chapter evidence:** P2; chapter status `ABSENT — designed, parked` (invalid as-built vocabulary retained verbatim); no chapter-owned acceptance-test ID.
- **Atomic obligations:** reuse a computed result only when complete input content, parameters and module version match; invalidate it when any identity element changes.
- **Current source:** no general compute-result cache, content-key store or invalidation test exists. ML training fingerprints identify rows but do not reuse module results. Python/runtime caches and frontend render/bitmap/memoized-color caches were explicitly excluded because they do not cache the specified computation.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** training-fingerprint tests prove identity sensitivity for one ML record, not result reuse.
- **Manual evidence:** `portfolio-performance` 0/50 and `workflow` 0/23 - unexercised.
- **Git evidence:** the compute-cache contract is `UNIMPLEMENTED`; a design document exists but was deliberately parked.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** existing owner direction parks the design until `DEC-008` establishes a measured portfolio need. Its assumptions must be reverified before use.
- **Next action:** do not implement now. When `DEC-008` reopens it, re-audit the parked DAG/cache design against current chains and write complete-key/invalidation tests before production code.

## SB-CORE-034 — Interactive surfaces stay responsive at portfolio scale

- **Chapter evidence:** P2; chapter status `PRESENT-DIVERGENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** pan, zoom and hover remain responsive with the declared portfolio open, and no interaction recomputes more than one frame requires.
- **Current source:** current UI is newer than the chapter: crossplot, histogram and Vega paths use `requestAnimationFrame`; shared plotting applies point budgets/decimation; heavy crossplot color work is memoized; view caches are bounded in selected panels. Coverage is not universal, no target/hardware/threshold exists, and no portfolio harness measures frame latency, memory retention or every interactive surface.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** decimation and plotting tests prove bounded data transformations, not end-user responsiveness at portfolio scale.
- **Manual evidence:** `portfolio-performance` 0/50, `workspace-shell` 0/159 and `vega` 0/2 are unexercised; `log-view` 5/37 and `crossplot` 6/13 are partial.
- **Git evidence:** several mitigation paths are integrated at the accepted anchor, but the broad responsiveness contract is unproved.
- **Verdict:** `PARTIAL`; `DEFERRED`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `SB-CORE-030`, `SB-CORE-031` and `DEC-008` must define the portfolio, operations, hardware and thresholds. No latency or memory limit is invented.
- **Next action:** after that decision, benchmark named surfaces, retain one-frame scheduling and bounded decimation, and fix only measured leaks/recomputation with a repeatable receipt.

## SB-CORE-035 — Well scoping is enforced in the backend

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** every operation claiming active-group scope must have the backend resolve/authorize that scope; frontend selection alone cannot define authority; membership changes cannot leave a stale execution set.
- **Current source:** `wellScope.ts` resolves active/group/pinned/selection/custom scopes in the frontend and sends explicit `well_ids`. Backend commands accept those ids directly. The database stores active group membership, but no backend execution path re-resolves the active group or refuses ids outside it. Thus the shared UI control is useful but not enforcement.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** UI scope serialization and group CRUD tests do not prove backend isolation or stale-membership refusal.
- **Manual evidence:** `well-scope` 3/9 and `report` 6/53 are partial; `workflow` 0/23 is unexercised.
- **Git evidence:** frontend scoping/group storage are integrated, while backend enforcement remains divergent.
- **Verdict:** `PRESENT-DIVERGENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` must identify which scoped operations are required in the pilot before the first backend contract is bounded.
- **Next action:** pass an explicit backend scope identity, resolve current membership transactionally, refuse unauthorized/stale ids, and add positive/negative isolation tests for one pilot-critical operation before inventorying the rest.

## SB-CORE-036 — Cancellation is honest

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** every displayed Cancel control has a worker that observes it; non-interruptible work offers no control; “Cancelled” is reported only when work actually stopped, with partial results explicitly surfaced.
- **Current source:** all `run_simple_job` call sites are registered non-cancellable. The seven current `run_job(..., true, ...)` families route to LAS ingest, equation, workflow module, Monte Carlo, ML and SandiMin workers that poll a `JobHandle` or raw shared flag; workflow chain marks raw-flag observation explicitly. `run_job` finalizes as Cancelled only after a worker observed the set flag, and the processing panel shows Cancel only when `JobView.cancellable` is true. Per-item states and ML cancelled-log-set metadata expose completed partial work.
- **Qualifying acceptance tests:** none because the chapter owns no test ID; ledger test class remains `MISSING`.
- **Supporting tests:** `cancellable_flag_reaches_the_view_both_ways`, `cancel_counts_as_cancelled_only_once_a_worker_observes_it`, `note_cancel_observed_marks_it_for_raw_flag_readers`, and `module_run_skips_all_wells_when_cancelled` passed. They are correctness-supporting tests but are not chapter-owned acceptance closure.
- **Manual evidence:** `monte-carlo` 2/14 and `machine-learning` 7/189 are partial; `workflow` 0/23, `processing-history` 0/7, `las-import` 0/57 and `dlis-import` 0/11 are unexercised.
- **Git evidence:** current cancellation-honesty implementation is integrated at the accepted anchor; the stale chapter divergence no longer matches current source.
- **Verdict:** `PRESENT-OK`; `UNDECIDED`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** automated source/test evidence is complete for the current registration inventory, but `DEC-003` determines whether cancellation is a paid-pilot release blocker and field interruption remains unexercised.
- **Next action:** make no production change. Add an owned inventory-level acceptance mapping and field-exercise a mid-write cancellation, confirming the UI and persisted partial-result qualification agree.

## SB-CORE-040 — Verification is indexed by capability

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-CORE-T12`.
- **Atomic obligations:** maintain a capability-indexed real-data verification matrix, mechanically generated from committed sources, with freshness enforced by the gate.
- **Current source:** `verification/capabilities.json` defines the capability map; `tools/generate-verification-matrix.mjs` combines it with `REVIEW.md`; `docs/VERIFICATION_MATRIX.md` is generated; `tools/check.ps1` runs the generator in `--check` mode before build/test.
- **Qualifying acceptance tests:** `src-tauri/tests/verification_matrix.rs::a_capability_matrix_is_generated_from_review_and_a_capability_map_and_checked_by_the_gate` passed, and `node tools/generate-verification-matrix.mjs --check` passed. This is `CORRECTNESS`; the expected matrix is derived from the two committed sources named by T12.
- **Supporting tests:** takeover ledger checks are complementary governance evidence, not T12.
- **Manual evidence:** the matrix itself records 78/1479 scenarios checked, 14/54 capabilities with any recorded exercise and 1/54 complete; `verification-stewardship` is 0/24. Generation does not close the field checks.
- **Git evidence:** reachable `8b420e5` contains the patch-equivalent integration of original candidate `34ee79e`.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied evidence contract); `FIELD-EVIDENCE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for generation/freshness; the matrix truthfully remains mostly open.
- **Next action:** make no production change. Preserve the freshness gate and use the matrix to drive Gate 4 field execution rather than claiming automated checks as field evidence.

## SB-CORE-041 — The tree builds and tests from a fresh clone

- **Chapter evidence:** P0; chapter status `PRESENT-DIVERGENT`; owned test `SB-CORE-T13`.
- **Atomic obligations:** a genuinely fresh clone resolves all dependencies, builds and passes the complete suite without manually placed or untracked fixtures.
- **Current source:** package/Cargo lockfiles, build instructions and a machine-runnable full gate exist, and the current worktree is green. There is no CI workflow or dated clean-clone receipt, and T13 is not implemented. Local caches and already-present optional tools mean this worktree cannot prove the fresh-clone clause.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** the full gate proves the accepted working tree only. Ignored optional-package/field-fixture tests are explicit and do not become required fresh-clone inputs.
- **Manual evidence:** `verification-stewardship` 0/24 and `security-integrity` 0/63 are unexercised; `project-lifecycle` 3/24 is partial.
- **Git evidence:** build files and gate are integrated at the accepted anchor; fresh-clone behavior remains `PRESENT-UNVERIFIED` rather than failed or proven.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** a clean checkout with empty dependency/build caches has not been observed; no missing fixture is assumed from historical prose.
- **Next action:** clone the accepted commit into a new clean directory/machine, install only documented prerequisites, run the full gate, preserve the receipt, and then implement T13 as CI clean-clone evidence.

## SB-CORE-042 — A green gate that a machine enforces

- **Chapter evidence:** P1; chapter status `PARTIAL`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** build, lint and tests run automatically on every change rather than depending on manual invocation.
- **Current source:** `tools/check.ps1` is a machine-runnable four-stage gate covering ledger, verification matrix, frontend acceptance/build and default Rust tests. `.github/workflows` is absent, and no repository automation invokes the gate on each change.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** `characterizes_the_green_gate_as_machine_enforced_but_still_manually_invoked` passed. It explicitly declares `CHARACTERIZATION` and pins the manual-invocation gap; it is not compliance evidence.
- **Manual evidence:** `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** reachable `0244bc6` contains the patch-equivalent integration of original characterization candidate `bb0c488`.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DEPLOYMENT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on `SB-CORE-041`; Jauhar must decide whether automatic per-change CI is required before the paid pilot or whether a manually enforced release freeze is temporarily sufficient.
- **Next action:** after that decision and clean-clone proof, add one repository workflow that runs the unchanged full gate for every proposed change and records the result.

## SB-CORE-043 — Architecture and decisions are written down

- **Chapter evidence:** P1; chapter status `ABSENT`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** a current, discoverable `ARCHITECTURE.md` exists; decision records exist and have an ownership/update rule.
- **Current source:** `docs/takeover/DECISIONS.md`, the takeover design, execution plans and `docs/record_*.md` provide substantial decision history. The required root `ARCHITECTURE.md` is absent, no ADR directory/system was found, and the mixed plan/build-record collection has no single maintenance contract proving current architecture coverage.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** none converts a one-off plan into maintained architecture evidence.
- **Manual evidence:** `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** decision records are integrated at the accepted anchor; the architecture half remains absent.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `RECOVERY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** Jauhar must decide whether architecture/onboarding continuity is required before the pilot or belongs after first-field validation.
- **Next action:** if approved, create one maintained architecture map and ADR index with explicit owners/update triggers, linking rather than duplicating the authoritative build records.

## SB-CORE-044 — Tier-C boundary is a shipped, auditable policy

- **Chapter evidence:** P1; chapter status `PARTIAL`; no chapter-owned acceptance-test ID.
- **Atomic obligations:** maintain the Tier-C register in the repository; every similar shipped capability has an asset-specific design-around with its own primary sources; un-cleared Tier-C material is excluded.
- **Current source:** `docs/IP_PROVENANCE.md` defines tiers, a same-increment maintenance rule, blocking treatment and known asset/fallback routes; `CONTRACT.md` defines independent derivation and primary-source requirements. The register itself records unresolved digitized-chart, vendor-merged endpoint and trademark/dependency questions. No exhaustive build/runtime inventory proves every shipped default/asset has a registered route and primary-source design-around.
- **Qualifying acceptance tests:** none; test class is `MISSING`.
- **Supporting tests:** `characterizes_the_tier_c_register_as_shipped_policy_with_asset_specific_design_around_routes` passed. It explicitly declares `CHARACTERIZATION` and proves policy text/known routes, not exhaustive enforcement.
- **Manual evidence:** `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** reachable `43fddd3` contains the patch-equivalent integration of original characterization candidate `3abf150`.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** primary-source and legal dispositions remain open for the asset classes named by the register; `SB-CORE-005` is one concrete dependency. No clearance is inferred from a passing test.
- **Next action:** inventory every shipped external-derived asset/default against the register, block any unregistered Tier-C route, require primary-source design-around evidence where similar capability ships, and obtain counsel disposition before first sale.
